pub mod ast;
pub mod codegen;
mod codegen_basic;
mod codegen_c;
pub mod diagnostics;
pub mod lexer;
pub mod parser;
pub mod records;
pub mod resolver;

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use codegen::CodeGenerator;
pub use codegen::Target;
use diagnostics::Diagnostic;
use lexer::{Lexer, TokenKind};
use parser::Parser;

#[derive(Debug, Clone)]
pub struct CompileOptions {
    pub library_dirs: Vec<PathBuf>,
    pub libraries: Vec<String>,
    /// Number every output line (BASCOM strict mode). When false, only lines
    /// that are branch targets receive a line number.
    pub line_numbers: bool,
    /// Which backend to generate code for. Defaults to `Target::Basic` --
    /// the only backend `compile_file` can actually produce output for
    /// today; `Target::C` always fails with a "not supported" diagnostic
    /// (see `codegen_c::generate`).
    pub target: Target,
    /// Pascal-style mandatory variable declaration, opt-in and off by
    /// default -- turning it on is *not* a superset of BASIC any more, so
    /// it never applies unless asked for. An identifier used as a plain
    /// scalar/array variable (not a call, not a builtin) that was never
    /// introduced by `dim`/`declare`, `const`, a `for` loop's own counter,
    /// or a function/procedure parameter is rejected. Checked only against
    /// the root program's own statements and functions -- never a
    /// `require`d library's, which may not itself be written this way (see
    /// `resolver::check_strict_vars`). Mutually exclusive in effect with
    /// `strict_vars_warn`; if both are set, this one wins.
    pub strict_vars: bool,
    /// Same check as `strict_vars`, but every finding is printed to stderr
    /// as a warning instead of failing the compile -- for trying strict
    /// mode against an existing program without committing to it yet.
    pub strict_vars_warn: bool,
}

impl CompileOptions {
    pub fn new() -> Self {
        Self {
            library_dirs: Vec::new(),
            libraries: Vec::new(),
            line_numbers: false,
            target: Target::Basic,
            strict_vars: false,
            strict_vars_warn: false,
        }
    }
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self::new()
    }
}

pub fn compile_source(
    filename: impl Into<String>,
    source: &str,
) -> Result<String, Vec<Diagnostic>> {
    let filename = filename.into();
    let program = parse_source(filename, source)?;
    let (mut program, synthesized_buffer_names) = records::lower(program)?;
    inject_mid_assign_helper_if_used(&mut program)?;
    resolver::validate(&program)?;
    print_legacy_form_warnings(&program);
    let conflicts = codegen::check_generated_name_conflicts(&program);
    if !conflicts.is_empty() {
        return Err(conflicts);
    }
    CodeGenerator::new()
        .with_synthesized_buffer_names(synthesized_buffer_names)
        .generate(&program)
}

/// The generated `Target::C` runtime-support file's fixed name -- see
/// `codegen_c::GeneratedC`'s own doc comment. Always the same name
/// (rather than derived from the input file, the way the app `.c` file's
/// own name is) so repeated builds in the same directory overwrite it
/// cleanly instead of accumulating garbage, and so the app file's own
/// `#include "bcc_runtime.h"` line never has to be templated per program.
pub const C_RUNTIME_FILE_NAME: &str = "bcc_runtime.h";

/// Same as `compile_file`, but for `Target::C` callers that need the
/// paired runtime-support file too (see `codegen_c::GeneratedC`) -- the
/// CLI (to write/link it alongside the app file) and this crate's own
/// C-backend tests (to check helper-definition text in the right place).
/// For `Target::Basic`, `runtime` is always `None` -- that backend never
/// splits its output. For `Target::C`, `runtime` is `None` exactly when
/// the program needs no `bcc_*` helper at all (see `GeneratedC`'s own
/// doc comment) -- `Some` otherwise, meant to be written out as a sibling
/// file named `C_RUNTIME_FILE_NAME` next to the app file.
pub fn compile_file_with_runtime(
    input: &Path,
    options: &CompileOptions,
) -> Result<(String, Option<String>), Vec<Diagnostic>> {
    let mut options = options.clone();
    if let Some(parent) = input.parent() {
        let parent = parent.to_path_buf();
        if !options.library_dirs.contains(&parent) {
            options.library_dirs.insert(0, parent);
        }
    }
    let options = &options;

    if options.strict_vars || options.strict_vars_warn {
        // Checked against the root file's own parse, on its own -- not the
        // merged `program` below, whose `require`d functions (BASCAL's own
        // `com.bascal.stdlib` included) were never written to satisfy this,
        // and not the DSL-lowered form, which invents buffer/scalar
        // variables no one is expected to `dim` by hand. See resolver.rs's
        // own `check_strict_vars` doc comment.
        let source = fs::read_to_string(input).map_err(|err| {
            vec![Diagnostic::error(
                diagnostics::SourcePos::new(input.display().to_string(), 1, 1),
                format!("failed to read source file: {err}"),
            )]
        })?;
        let root_only = parse_source(input.display().to_string(), &source)?;
        let findings = resolver::check_strict_vars(&root_only, options.strict_vars_warn);
        if options.strict_vars {
            if !findings.is_empty() {
                return Err(findings);
            }
        } else {
            for finding in &findings {
                eprintln!("{finding}");
            }
        }
    }

    let mut visited = HashSet::new();
    let mut program = load_program_recursive(input, true, options, &mut visited)?;

    // Resolve the shared COMMON block if the program declares one.
    if let Some(shared_name) = program
        .program_decl
        .as_ref()
        .and_then(|d| d.shared.as_deref())
        .map(str::to_string)
    {
        if let Some(shared_path) = resolve_shared_path(&shared_name, input, options) {
            program.common = load_shared_file(&shared_path, &shared_name)?;
        }
        // Shared file not found → compile without COMMON (silent; it may not exist yet).
    }

    let (mut program, synthesized_buffer_names) = records::lower(program)?;
    inject_mid_assign_helper_if_used(&mut program)?;
    resolver::validate(&program)?;
    print_legacy_form_warnings(&program);
    match options.target {
        Target::Basic => {
            let basic = CodeGenerator::new()
                .with_line_numbers(options.line_numbers)
                .with_synthesized_buffer_names(synthesized_buffer_names)
                .generate(&program)?;
            Ok((basic, None))
        }
        Target::C => {
            let generated = codegen_c::generate(&program)?;
            let runtime = if generated.runtime.is_empty() {
                None
            } else {
                Some(generated.runtime)
            };
            Ok((generated.app, runtime))
        }
    }
}

/// The most common case: just the primary generated file (the whole
/// `.bas` for `Target::Basic`, or the app-only `.c` for `Target::C` --
/// see `compile_file_with_runtime`'s own doc comment for why that's not
/// necessarily everything a `Target::C` program needs to actually build).
pub fn compile_file(input: &Path, options: &CompileOptions) -> Result<String, Vec<Diagnostic>> {
    Ok(compile_file_with_runtime(input, options)?.0)
}

/// If `program` uses `MID$` statement-form assignment anywhere (top-level
/// or inside any function body) and hasn't already defined or required its
/// own `midAssign$`, splices in `com.bascal.stdlib.midAssign` -- resolved
/// via `stdlib_search_roots()`, the same on-disk library `require
/// com.bascal.stdlib.*` resolves against, just triggered by the AST shape
/// instead of an explicit `require` line, since nothing in the user's own
/// source ever names this function -- the transpiler synthesizes the call
/// (see `codegen::MID_ASSIGN_HELPER_NAME`).
fn inject_mid_assign_helper_if_used(program: &mut ast::Program) -> Result<(), Vec<Diagnostic>> {
    let already_defined = program.functions.iter().any(|f| {
        f.name
            .name
            .eq_ignore_ascii_case(codegen::MID_ASSIGN_HELPER_NAME)
    });
    if already_defined || !program_uses_mid_assign(program) {
        return Ok(());
    }

    let symbol = format!("com.bascal.stdlib.{}", codegen::MID_ASSIGN_HELPER_NAME);
    let relative = required_symbol_to_path(&symbol);
    let path = stdlib_search_roots()
        .into_iter()
        .map(|root| root.join(&relative))
        .find(|candidate| candidate.exists())
        .ok_or_else(|| {
            vec![Diagnostic::error(
                diagnostics::SourcePos::new("<transpiler-internal>", 1, 1),
                format!(
                    "internal error: this program uses MID$ statement-form assignment, which \
                     needs BASCAL's own {symbol} helper, but {} could not be found -- this \
                     looks like a broken install; check that `com/` shipped alongside `bcc`",
                    relative.display()
                ),
            )]
        })?;

    let source = fs::read_to_string(&path).map_err(|err| {
        vec![Diagnostic::error(
            diagnostics::SourcePos::new("<transpiler-internal>", 1, 1),
            format!("internal error: failed to read {}: {err}", path.display()),
        )]
    })?;
    let filename = path.display().to_string();
    let helper_program = parse_source(filename.clone(), &source)?;
    let [function]: [ast::FunctionDef; 1] =
        helper_program
            .functions
            .try_into()
            .unwrap_or_else(|functions: Vec<ast::FunctionDef>| {
                panic!(
                    "BASCAL bug: {filename} must declare exactly one function, found {}",
                    functions.len()
                )
            });
    program.functions.push(function);
    Ok(())
}

/// Prints every legacy-form finding from `resolver::check_legacy_forms` to
/// stderr as a warning -- advisory only, so unlike `resolver::validate`
/// this never turns into an `Err`; a legacy BASIC form with a BASCAL
/// equivalent still compiles, it just gets named so new/edited source can
/// be steered toward the structured spelling (see resolver.rs's own doc
/// comment on `check_legacy_forms`).
fn print_legacy_form_warnings(program: &ast::Program) {
    for finding in resolver::check_legacy_forms(program) {
        eprintln!("{finding}");
    }
}

fn program_uses_mid_assign(program: &ast::Program) -> bool {
    statements_use_mid_assign(&program.statements)
        || program
            .functions
            .iter()
            .any(|f| statements_use_mid_assign(&f.body))
}

fn statements_use_mid_assign(statements: &[ast::Stmt]) -> bool {
    statements.iter().any(statement_uses_mid_assign)
}

fn statement_uses_mid_assign(statement: &ast::Stmt) -> bool {
    use ast::Statement::*;
    match &statement.kind {
        MidAssign { .. } => true,
        If {
            then_body,
            else_body,
            ..
        } => statements_use_mid_assign(then_body) || statements_use_mid_assign(else_body),
        For { body, .. } | While { body, .. } | Do { body, .. } => statements_use_mid_assign(body),
        SelectCase {
            cases, else_body, ..
        } => {
            cases.iter().any(|c| statements_use_mid_assign(&c.body))
                || statements_use_mid_assign(else_body)
        }
        _ => false,
    }
}

pub fn default_output_path(input: &Path, target: Target) -> std::path::PathBuf {
    let extension = match target {
        Target::Basic => "bas",
        Target::C => "c",
    };
    input.with_extension(extension)
}

fn parse_source(filename: String, source: &str) -> Result<ast::Program, Vec<Diagnostic>> {
    let tokens = Lexer::new(&filename, source).lex();
    reject_underscored_identifiers(&tokens)?;
    let mut parser = Parser::new(filename, tokens);
    parser.parse_program()
}

/// An identifier with an underscore is a syntax error on real MBASIC/BASCOM
/// whenever it's read as an expression operand (it's only tolerated as an
/// assignment target) -- discovered by compiling against a real BASCOM 2.00
/// transpiler. Since almost every variable gets read somewhere, and BASCAL
/// can't safely rewrite a user's own chosen name, the underscore is rejected
/// outright at parse time, with camelCase suggested as the fix -- matching
/// the convention the transpiler's own generated names already use.
fn reject_underscored_identifiers(tokens: &[lexer::Token]) -> Result<(), Vec<Diagnostic>> {
    let diagnostics: Vec<Diagnostic> = tokens
        .iter()
        .filter_map(|token| match &token.kind {
            TokenKind::Ident(name) if name.contains('_') => Some(Diagnostic::error(
                token.pos.clone(),
                format!(
                    "identifier `{name}` contains an underscore, which real MBASIC/BASCOM \
                     rejects as a syntax error wherever the name is read (not just assigned) -- \
                     use camelCase instead (e.g. `{}`)",
                    to_suggested_camel_case(name)
                ),
            )),
            _ => None,
        })
        .collect();
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

fn to_suggested_camel_case(name: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;
    for ch in name.chars() {
        if ch == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.extend(ch.to_uppercase());
            capitalize_next = false;
        } else {
            result.push(ch);
        }
    }
    result
}

fn load_program_recursive(
    input: &Path,
    is_root: bool,
    options: &CompileOptions,
    visited: &mut HashSet<PathBuf>,
) -> Result<ast::Program, Vec<Diagnostic>> {
    let input = normalize_path(input);
    if !visited.insert(input.clone()) {
        return Ok(ast::Program {
            program_decl: None,
            library_decl: None,
            shared_decl: None,
            declarations: Vec::new(),
            common: Vec::new(),
            statements: Vec::new(),
            functions: Vec::new(),
            records: Vec::new(),
        });
    }

    let source = fs::read_to_string(&input).map_err(|err| {
        vec![Diagnostic::error(
            diagnostics::SourcePos::new(input.display().to_string(), 1, 1),
            format!("failed to read source file: {err}"),
        )]
    })?;
    let program = parse_source(input.display().to_string(), &source)?;

    let mut errors = Vec::new();

    if !is_root && program.program_decl.is_some() {
        errors.push(Diagnostic::error(
            diagnostics::SourcePos::new(input.display().to_string(), 1, 1),
            format!(
                "`program` declaration is not allowed in library modules (`{}`)",
                input.display()
            ),
        ));
    }

    if is_root && program.library_decl.is_some() {
        errors.push(Diagnostic::error(
            diagnostics::SourcePos::new(input.display().to_string(), 1, 1),
            format!(
                "`library` declaration is not allowed in the root program file (`{}`) -- only files loaded via `require`/`import` may declare `library`",
                input.display()
            ),
        ));
    }

    if program.shared_decl.is_some() {
        errors.push(Diagnostic::error(
            diagnostics::SourcePos::new(input.display().to_string(), 1, 1),
            format!(
                "`shared` declaration is only valid in shared-variable files, not in `{}`",
                input.display()
            ),
        ));
    }

    // Every file must declare exactly one of `program`/`library`/`shared`.
    // Gated on `shared_decl.is_none()` so a file that already errored above
    // for a stray `shared` header doesn't also get a confusing second error
    // about a missing `program`/`library` header.
    if program.shared_decl.is_none() {
        if is_root && program.program_decl.is_none() {
            errors.push(Diagnostic::error(
                diagnostics::SourcePos::new(input.display().to_string(), 1, 1),
                format!(
                    "file `{}` must start with `program <name>` -- only files loaded via `require`/`import` may omit it (and only if they declare `library <name>` instead)",
                    input.display()
                ),
            ));
        } else if !is_root && program.library_decl.is_none() && program.program_decl.is_none() {
            errors.push(Diagnostic::error(
                diagnostics::SourcePos::new(input.display().to_string(), 1, 1),
                format!(
                    "required file `{}` must declare `library <name>` -- only files declared `library` may be `require`d/`import`ed",
                    input.display()
                ),
            ));
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    let mut merged = ast::Program {
        program_decl: program.program_decl,
        library_decl: None,
        shared_decl: None,
        declarations: Vec::new(),
        common: Vec::new(),
        statements: Vec::new(),
        functions: Vec::new(),
        records: Vec::new(),
    };

    for declaration in &program.declarations {
        match declaration {
            ast::DependencyDecl::Require(symbol) | ast::DependencyDecl::Import(symbol) => {
                let dependency_path = resolve_required_symbol(&symbol.raw, &input, options)?;
                let dependency = load_program_recursive(&dependency_path, false, options, visited)?;
                merged.statements.extend(dependency.statements);
                merged.functions.extend(dependency.functions);
                merged.records.extend(dependency.records);
            }
        }
    }

    merged.statements.extend(program.statements);
    merged.functions.extend(program.functions);
    merged.records.extend(program.records);
    Ok(merged)
}

fn load_shared_file(
    path: &Path,
    shared_name: &str,
) -> Result<Vec<ast::CommonBlock>, Vec<Diagnostic>> {
    let source = fs::read_to_string(path).map_err(|err| {
        vec![Diagnostic::error(
            diagnostics::SourcePos::new(path.display().to_string(), 1, 1),
            format!("failed to read shared file: {err}"),
        )]
    })?;
    let program = parse_source(path.display().to_string(), &source)?;

    let pos = diagnostics::SourcePos::new(path.display().to_string(), 1, 1);
    let mut errors = Vec::new();

    // Every top-level `dim` becomes a CommonVar below -- a `shared <name>`
    // file's variables are COMMON by default, with no separate keyword to
    // opt in.
    if program.statements.iter().any(|s| match &s.kind {
        ast::Statement::BlankLine
        | ast::Statement::BlockComment(_)
        | ast::Statement::Dim { .. } => false,
        ast::Statement::Raw(text) => !text.trim_start().starts_with('\''),
        _ => true,
    }) {
        errors.push(Diagnostic::error(
            pos.clone(),
            format!(
                "shared file `{}` may only contain DIM declarations (no other statements)",
                path.display()
            ),
        ));
    }
    if !program.functions.is_empty() {
        errors.push(Diagnostic::error(
            pos.clone(),
            format!(
                "shared file `{}` may only contain DIM declarations (no functions)",
                path.display()
            ),
        ));
    }
    if !program.declarations.is_empty() {
        errors.push(Diagnostic::error(
            pos.clone(),
            format!(
                "shared file `{}` may only contain DIM declarations (no require/import)",
                path.display()
            ),
        ));
    }
    if program.program_decl.is_some() {
        errors.push(Diagnostic::error(
            pos.clone(),
            format!(
                "shared file `{}` may only contain DIM declarations (no program declaration)",
                path.display()
            ),
        ));
    }
    if program.library_decl.is_some() {
        errors.push(Diagnostic::error(
            pos.clone(),
            format!(
                "shared file `{}` may only contain DIM declarations (no library declaration)",
                path.display()
            ),
        ));
    }
    // The `shared <name>` header is mandatory -- every file must declare
    // exactly one of `program`/`library`/`shared` -- and it must name the
    // same shared file this was actually resolved as, catching a
    // copy-pasted header pointing at the wrong filename.
    match &program.shared_decl {
        None => errors.push(Diagnostic::error(
            pos.clone(),
            format!("shared file `{}` must declare `shared {shared_name}`", path.display()),
        )),
        Some(declared) if declared != shared_name => errors.push(Diagnostic::error(
            pos.clone(),
            format!(
                "shared file `{}` declares `shared {declared}`, but was loaded as `{shared_name}` -- its filename must be `{declared}.bcl`",
                path.display()
            ),
        )),
        Some(_) => {}
    }

    // Every `dim name[()]` in the file becomes one more shared variable,
    // collected into a single COMMON block (declaration order matters for
    // CHAIN).
    let dim_vars: Vec<ast::CommonVar> = program
        .statements
        .iter()
        .filter_map(|s| match &s.kind {
            ast::Statement::Dim { name, is_array, .. } => Some(ast::CommonVar {
                name: name.clone(),
                is_array: *is_array,
            }),
            _ => None,
        })
        .collect();

    if dim_vars.is_empty() {
        errors.push(Diagnostic::error(
            pos,
            format!(
                "shared file `{}` contains no DIM declarations",
                path.display()
            ),
        ));
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(vec![ast::CommonBlock { vars: dim_vars }])
}

fn resolve_shared_path(
    shared_name: &str,
    source_file: &Path,
    options: &CompileOptions,
) -> Option<PathBuf> {
    let filename = format!("{shared_name}.bcl");
    for root in search_roots(source_file, options) {
        let candidate = root.join(&filename);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

fn resolve_required_symbol(
    raw: &str,
    source_file: &Path,
    options: &CompileOptions,
) -> Result<PathBuf, Vec<Diagnostic>> {
    let relative = required_symbol_to_path(raw);
    for root in search_roots(source_file, options) {
        let candidate = root.join(&relative);
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    Err(vec![Diagnostic::error(
        diagnostics::SourcePos::new(source_file.display().to_string(), 1, 1),
        format!(
            "failed to resolve required BASCAL symbol `{raw}` as {}",
            relative.display()
        ),
    )])
}

fn required_symbol_to_path(raw: &str) -> PathBuf {
    let mut path = raw.split('.').collect::<PathBuf>();
    path.set_extension("bcl");
    path
}

fn search_roots(source_file: &Path, options: &CompileOptions) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(parent) = source_file.parent() {
        roots.push(parent.to_path_buf());
    }
    roots.extend(options.library_dirs.iter().cloned());
    roots.extend(stdlib_search_roots());
    roots
}

/// Where the bundled `com.bascal.stdlib.*` library lives, checked last so an
/// explicit `-L` (or a same-named file next to the source) can still shadow
/// it. Two on-disk layouts are supported, since release packages don't all
/// place the binary and its data the same way:
///   - portable (zip/tarball): `com/` sits right next to `bcc`.
///   - FHS (deb/rpm): `bcc` installs to `.../bin/bcc` and `com/` installs to
///     `.../share/bascal/com/`, the standard split those packages expect --
///     reached from the binary via `../share/bascal`, the same relative hop
///     tools like `git` and `gcc` use to find their own bundled data.
/// `CARGO_MANIFEST_DIR` covers `cargo build`/`cargo test`, since it's baked
/// in at compile time from wherever this crate was built.
fn stdlib_search_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            roots.push(dir.to_path_buf());
            roots.push(dir.join("../share/bascal"));
        }
    }
    roots.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")));
    roots
}

fn normalize_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trailing_fixed_parameter_defaults_are_inserted_at_call_sites() {
        let source = r#"const punctuation$ = "!"

function decorate$(text$, suffix$ = punctuation$)
    return text$ + suffix$
end function

procedure announce(text$, suffix$ = "!")
    print text$ + suffix$
end procedure

result$ = decorate$("hello")
announce("ready")
print result$
end
"#;
        let basic = compile_source("defaults.bcl", source).expect("defaults should compile");
        assert!(basic.contains("decorateSuffix0$ = punctuation$"), "{basic}");
        assert!(basic.contains("announceSuffix0$ = \"!\""), "{basic}");

        let c = compile_source_via_c_target(source);
        assert!(c.contains("bf_s_decorate(\"hello\", bv_s_punctuation,"), "{c}");
        assert!(c.contains("bf_i_announce(\"ready\", \"!\");"), "{c}");
    }

    #[test]
    fn basic_target_lowers_scalar_method_calls_and_chains() {
        let source = r#"method$ capitalize$()
    return self$
end method

method$ pad$(width%)
    return self$
end method

result$ = name$.capitalize().pad(20)
end
"#;
        let output = compile_source("methods.bcl", source).expect("methods should lower to BASIC");
        assert!(output.contains("capitalizeSelf0$ = name$"), "{output}");
        assert!(output.contains("GOSUB"), "{output}");
        assert!(output.contains("result$ = padResult0$"), "{output}");
    }

    #[test]
    fn fixed_parameter_defaults_reject_dynamic_and_non_trailing_values() {
        let dynamic = r#"function choose%(value% = timer)
    return value%
end function
end
"#;
        let dynamic_error = compile_source("dynamic_default.bcl", dynamic)
            .expect_err("a dynamic default must be rejected");
        assert!(dynamic_error.iter().any(|d| d.message.contains("literal or a top-level `const`")));

        let non_trailing = r#"function choose%(first% = 1, second%)
    return first% + second%
end function
end
"#;
        let trailing_error = compile_source("non_trailing_default.bcl", non_trailing)
            .expect_err("a required parameter after a default must be rejected");
        assert!(trailing_error.iter().any(|d| d.message.contains("required but follows")));

        let signed = r#"function offset%(value% = -1)
    return value%
end function
print offset%()
end
"#;
        compile_source("signed_default.bcl", signed)
            .expect("a signed numeric literal is a fixed default");
    }

    #[test]
    fn compiles_sort_driver_sample() {
        let source = include_str!("../tutorial/sort_driver.bcl");
        let output =
            compile_source("tutorial/sort_driver.bcl", source).expect("sample should compile");
        assert!(output.contains("' require com.bascal.sort.bubbleSort"));
        // Without the sort library bubbleSort% is not in the symbol table;
        // it is emitted lowercase like any other user symbol, not uppercased.
        assert!(output.contains("bubblesort%(bubbledata%)"));
        assert!(output.contains("END"));
    }

    #[test]
    fn lowers_functions_to_labels_and_gosub() {
        // Mixed-case: function name, params, and caller variable are normalised to lowercase.
        let source = r#"function Add%(Left%, Right%)
    return Left% + Right%
end function

Total% = Add%(10, 20)
PRINT Total%
END
"#;

        let output = compile_source("add.bcl", source).expect("sample should compile");
        assert!(
            output.contains("' function add%"),
            "spec comment should be emitted"
        );
        assert!(
            !output.lines().any(|l| {
                let p = l
                    .trim_start()
                    .trim_start_matches(|c: char| c.is_ascii_digit())
                    .trim_start();
                !p.starts_with('\'') && p.to_ascii_lowercase().contains("function ")
            }),
            "should not emit BASCOM function declarations"
        );
        assert!(
            output.contains("' end function add%"),
            "end function comment should be emitted"
        );
        assert!(
            !output.lines().any(|l| {
                let p = l
                    .trim_start()
                    .trim_start_matches(|c: char| c.is_ascii_digit())
                    .trim_start();
                !p.starts_with('\'') && p.to_ascii_lowercase().starts_with("end function")
            }),
            "should not emit BASCOM end function declarations"
        );
        assert!(output.contains("addLeft0% = 10"));
        assert!(output.contains("addRight0% = 20"));
        assert!(output.contains("GOSUB "));
        assert!(output.contains("total% = addResult0%"));
        assert!(!output.contains("FN_add"));
        assert!(output.contains("addResult0% = addLeft0% + addRight0%"));
    }

    #[test]
    fn lowers_one_argument_suffix_functions() {
        let source = r#"function double%(value%)
    return value% * 2
end function

answer% = double%(21)
END
"#;

        let output = compile_source("double.bcl", source).expect("sample should compile");
        assert!(output.contains("doubleValue0% = 21"));
        assert!(output.contains("GOSUB "));
        assert!(!output.contains("FN_double"));
        assert!(output.contains("answer% = doubleResult0%"));
        assert!(output.contains("doubleResult0% = doubleValue0% * 2"));
    }

    #[test]
    fn assigns_repeated_function_results_to_variables() {
        let source = include_str!("../tutorial/07_functions.bcl");
        let output =
            compile_source("tutorial/07_functions.bcl", source).expect("sample should compile");

        // repeat$ is called twice; each result must be captured in a$ and b$ separately
        assert!(output.contains("GOSUB "));
        assert!(output.contains("a$ = repeatResult0$"));
        assert!(output.contains("b$ = repeatResult0$"));
    }

    #[test]
    fn lowers_procedures_to_gosub_without_result_variable() {
        let source = r#"procedure greet(name$)
    PRINT "Hello, " + name$
end procedure

greet("World")
END
"#;
        let output = compile_source("greet.bcl", source).expect("procedure should compile");
        assert!(
            output.contains("GOSUB "),
            "should emit GOSUB for procedure call"
        );
        assert!(
            !output.contains("greet_result"),
            "procedures must not emit a result variable"
        );
        assert!(
            output.contains("' procedure greet("),
            "should annotate as procedure"
        );
        assert!(
            output.contains("' end procedure greet"),
            "should close annotation as procedure"
        );
    }

    #[test]
    fn comment_text_matching_a_label_name_is_left_untouched() {
        let source = r#"// jump to done when finished
goto done

print "before"

done:
print "after"
end
"#;
        let output = compile_source("done.bcl", source).expect("sample should compile");
        assert!(
            output.contains("' jump to done when finished"),
            "comment text must not be rewritten just because it contains a label's name:\n{output}"
        );
        assert!(
            output.contains("GOTO 10"),
            "goto should still resolve to the label's line number"
        );
    }

    #[test]
    fn function_local_const_declares_and_reads_the_same_lowered_name() {
        // A `const` declared inside a function/procedure body must be
        // renamed to its local BASIC name (like `dim`/assignment already
        // are) so the assignment and every later read of it agree --
        // otherwise the declaration binds one name while reads resolve to a
        // fresh, never-assigned local of a different name. Real MBASIC/
        // BASCOM has no CONST statement at all, so `const` always lowers to
        // a plain assignment, never a CONST line.
        let source = r#"procedure show()
    const n% = 5
    print n%
end procedure

show()
end
"#;
        let output = compile_source("const_local.bcl", source).expect("should compile");
        assert!(
            !output.contains("CONST "),
            "CONST isn't valid on real MBASIC/BASCOM:\n{output}"
        );
        assert!(
            output.contains("showN0% = 5"),
            "unexpected const line:\n{output}"
        );
        assert!(
            output.contains("PRINT showN0%"),
            "PRINT should read back the same local name the const line declared:\n{output}"
        );
    }

    #[test]
    fn top_level_const_resolves_globally_inside_procedure() {
        // A top-level `const` referenced inside a `procedure` body must
        // resolve to the real top-level name, not a fresh per-function
        // local -- otherwise the reference reads an unassigned local
        // instead of the actual constant.
        let source = r#"const col% = 20

procedure show()
    locate 1, col%
end procedure

show()
end
"#;
        let output = compile_source("const_global_proc.bcl", source).expect("should compile");
        assert!(
            output.contains("LOCATE 1, col%"),
            "reference inside the procedure should resolve to the real top-level const, not a synthesized showCol-style local:\n{output}"
        );
        assert!(
            !output.contains("showCol"),
            "no per-function local should be synthesized for a top-level const:\n{output}"
        );
    }

    #[test]
    fn inkey_inside_function_resolves_to_builtin() {
        // INKEY$ referenced inside a `function` body must resolve to the
        // real builtin (via known_callables), not a fresh per-function
        // local -- otherwise the assignment reads an unassigned local
        // forever, hanging any `loop until` that polls it.
        let source = r#"function readKey$()
    global lastKey$
    lastKey$ = inkey$
    return lastKey$
end function

k$ = readKey$()
print k$
end
"#;
        let output = compile_source("inkey_in_function.bcl", source).expect("should compile");
        assert!(
            output.contains("= INKEY$"),
            "INKEY$ inside a function should lower to the real builtin, not a synthesized local:\n{output}"
        );
    }

    #[test]
    fn input_dollar_inside_function_resolves_to_builtin() {
        // INPUT$(n) referenced inside a `function` body must resolve to
        // the real builtin call, not a fresh per-function local array --
        // "input" was missing from BASIC_BUILTINS the same way "inkey"
        // was, so the Expr::Call codegen path fell through to treating
        // `input$(1)` as an unassigned local array reference.
        let source = r#"function readOne$()
    x$ = input$(1)
    return x$
end function

print readOne$()
end
"#;
        let output =
            compile_source("input_dollar_in_function.bcl", source).expect("should compile");
        assert!(
            output.contains("= INPUT$(1)"),
            "INPUT$ inside a function should lower to the real builtin, not a synthesized local array:\n{output}"
        );
    }

    #[test]
    fn err_and_erl_inside_procedure_resolve_to_builtins() {
        // ERR and ERL, the numeric error-handler pseudo-variables, must
        // resolve globally when referenced inside a `procedure` body,
        // the same as DATE$/TIME$/TIMER/INKEY$ -- they were missing from
        // BASIC_BUILTINS, so a reference inside a procedure resolved to
        // an unassigned local instead of the real system variable.
        let source = r#"procedure checkErr()
    if err = 53 then
        print "no file"
    end if
    print erl
end procedure

on error goto handler
error 53
goto after
handler:
checkErr()
resume next
after:
end
"#;
        let output = compile_source("err_erl_in_procedure.bcl", source).expect("should compile");
        assert!(
            output.contains("ERR = 53"),
            "ERR inside a procedure should resolve to the real system variable:\n{output}"
        );
        assert!(
            output.contains("PRINT ERL"),
            "ERL inside a procedure should resolve to the real system variable:\n{output}"
        );
    }

    #[test]
    fn top_level_const_resolves_globally_inside_function() {
        // Same as the procedure case above, but for `function`, to cover
        // both callable kinds -- ident() resolution doesn't distinguish
        // between them, but the regression this guards against was only
        // ever demonstrated against `procedure`.
        let source = r#"const factor% = 3

function scale%(n%)
    return n% * factor%
end function

print scale%(5)
end
"#;
        let output = compile_source("const_global_func.bcl", source).expect("should compile");
        assert!(
            output.contains("factor%"),
            "reference inside the function should resolve to the real top-level const:\n{output}"
        );
        assert!(
            !output.contains("scaleFactor"),
            "no per-function local should be synthesized for a top-level const:\n{output}"
        );
    }

    #[test]
    fn top_level_const_and_explicit_global_coexist_in_same_function() {
        // A top-level const referenced alongside an explicit
        // `global`-declared unrelated variable in the same body must not
        // interfere with each other -- the const resolves globally
        // without a `global` declaration, and the explicit global keeps
        // working exactly as before.
        let source = r#"const limit% = 10
dim total%

procedure accumulate()
    global total%
    total% = total% + limit%
end procedure

accumulate()
print total%
end
"#;
        let output = compile_source("const_and_global.bcl", source).expect("should compile");
        assert!(
            output.contains("limit%"),
            "the const should still resolve to the real top-level name:\n{output}"
        );
        assert!(
            output.contains("total% = total% + limit%"),
            "the explicit global and the const should resolve together correctly:\n{output}"
        );
    }

    #[test]
    fn top_level_const_resolves_identically_across_multiple_procedures() {
        // A const referenced both at top level and inside multiple
        // different procedures in the same program must resolve to the
        // same real name everywhere -- no per-function duplication or
        // divergence.
        let source = r#"const rate% = 7

procedure showA()
    print rate%
end procedure

procedure showB()
    print rate% * 2
end procedure

print rate%
showA()
showB()
end
"#;
        let output = compile_source("const_multi_proc.bcl", source).expect("should compile");
        let occurrences = output.matches("rate%").count();
        assert!(
            occurrences >= 4,
            "every reference (top level + both procedures) should use the same real `rate%` name:\n{output}"
        );
        assert!(
            !output.contains("showARate") && !output.contains("showBRate"),
            "no per-function local should be synthesized for the const in either procedure:\n{output}"
        );
    }

    #[test]
    fn procedure_early_return_emits_bare_return() {
        let source = r#"procedure sayIfPositive(n%)
    if n% <= 0 then
        return
    end if
    PRINT STR$(n%)
end procedure

sayIfPositive(5)
sayIfPositive(-1)
END
"#;
        let output =
            compile_source("early.bcl", source).expect("procedure with return should compile");
        assert!(output.contains("RETURN"), "should emit RETURN");
        assert!(
            !output.contains("sayIfPositive_result"),
            "no result variable for procedure"
        );
    }

    #[test]
    fn block_comment_preserves_internal_blank_lines() {
        let source = "/*\nFirst paragraph.\n\nSecond paragraph.\n*/\nEND\n";
        let output = compile_source("comment.bcl", source).expect("should compile");
        let lines: Vec<&str> = output.lines().collect();
        let first = lines
            .iter()
            .position(|l| l.contains("First paragraph."))
            .unwrap();
        let second = lines
            .iter()
            .position(|l| l.contains("Second paragraph."))
            .unwrap();
        assert!(
            second > first + 1,
            "blank line should separate the two comment paragraphs"
        );
        assert!(
            lines[first + 1].trim().is_empty(),
            "line between paragraphs should be blank"
        );
    }

    #[test]
    fn compile_file_recursively_includes_required_bcl_files() {
        let input = Path::new(env!("CARGO_MANIFEST_DIR")).join("tutorial/sort_driver.bcl");
        let output =
            compile_file(&input, &CompileOptions::new()).expect("sort driver should compile");

        assert!(!output.contains("' require com.bascal.sort.bubbleSort%"));
        assert!(output.contains("' Sort driver for the BASCAL example sort library."));
        assert!(output.contains("' In-place bubble sort."));
        // Mixed-case source names are normalised to lowercase in comments.
        assert!(output.contains("' function bubblesort%(data%)"));
        assert!(output.contains("' function shellsort%(data%)"));
        assert!(output.contains("' function touch%(value%)"));
        assert!(!output.contains("placeholder"));
        assert!(
            !output.contains("BCC_COPY%"),
            "hardcoded BCC_COPY% loop var should not appear"
        );
        // sort_driver.bcl uses mixed-case `bubbleData%`; output normalises to lowercase.
        assert!(output
            .lines()
            .any(|l| l.contains("bubblesortData0%(") && l.contains(") = bubbledata%(")));
        assert!(output
            .lines()
            .any(|l| l.contains("bubbledata%(") && l.contains(") = bubblesortData0%(")));
        assert!(output
            .contains("bubblesortData0%(bubblesortJ0%) = bubblesortData0%(bubblesortJ0% + 1)"));
        assert!(
            output.contains("quicksortData0%(quicksortWall0%) = quicksortData0%(quicksortQHigh0%)")
        );
        assert!(output.contains("GOSUB "));
    }

    #[test]
    fn basic_target_lowers_methods_from_required_libraries() {
        let dir = tempfile::tempdir().expect("temp directory");
        let lib_dir = dir.path().join("com/example");
        fs::create_dir_all(&lib_dir).expect("library directory");
        fs::write(
            lib_dir.join("text.bcl"),
            "library com.example.text\nmethod$ capitalize$()\nreturn self$\nend method\n",
        )
        .expect("library source");
        let input = dir.path().join("main.bcl");
        fs::write(
            &input,
            "program main\nrequire com.example.text\nresult$ = name$.capitalize()\nend\n",
        )
        .expect("program source");
        let output = compile_file(&input, &CompileOptions::new()).expect("required method should compile");
        assert!(output.contains("capitalizeSelf0$ = name$"), "{output}");
        assert!(output.contains("result$ = capitalizeResult0$"), "{output}");
        let mut c_options = CompileOptions::new();
        c_options.target = Target::C;
        let (c_output, _) = compile_file_with_runtime(&input, &c_options)
            .expect("required method should compile for C");
        assert!(c_output.contains("bf_s_capitalize"), "{c_output}");
    }

    #[test]
    fn c_target_lowers_scalar_method_calls_and_chains() {
        let source = r#"method$ capitalize$()
    return self$
end method

method! negate!()
    return -self!
end method

result$ = name$.capitalize()
answer! = value!.negate()
end
"#;
        let output = compile_source_via_c_target(source);
        assert!(output.contains("bf_s_capitalize_s(bv_s_name"), "{output}");
        assert!(output.contains("bf_f_negate_f(bv_f_value"), "{output}");
    }

    #[test]
    fn program_shared_loads_common_block() {
        let dir = tempfile::tempdir().unwrap();
        let prog_path = dir.path().join("myapp.bcl");
        let shared_path = dir.path().join("mystate.bcl");

        std::fs::write(
            &prog_path,
            "program myapp shared mystate\nPRINT \"hello\"\nEND\n",
        )
        .unwrap();
        std::fs::write(
            &shared_path,
            "shared mystate\n\ndim score%\ndim level%\ndim name$\n",
        )
        .unwrap();

        let output = compile_file(&prog_path, &CompileOptions::new())
            .expect("program with shared file should compile");

        assert!(output.contains("COMMON score%, level%, name$"));
        assert!(output.contains("PRINT \"hello\""));
    }

    #[test]
    fn common_keyword_is_rejected_everywhere() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.bcl");
        std::fs::write(&path, "program bad\ncommon score%\nPRINT 1\nEND\n").unwrap();

        let result = compile_file(&path, &CompileOptions::new());
        assert!(result.is_err());
        let msg = result
            .unwrap_err()
            .into_iter()
            .map(|d| d.to_string())
            .collect::<String>();
        assert!(msg.contains("the `common` keyword has been removed"));
    }

    #[test]
    fn declare_compiles_identically_to_dim() {
        let dim_out = compile_source("d.bcl", "dim x%, y%(20)\nx% = 5\nprint x%\nend\n")
            .expect("dim should compile");
        let declare_out = compile_source("e.bcl", "declare x%, y%(20)\nx% = 5\nprint x%\nend\n")
            .expect("declare should compile");
        assert_eq!(dim_out, declare_out);
        assert!(declare_out.contains("DIM x%"), "unexpected output:\n{declare_out}");
    }

    #[test]
    fn function_cannot_shadow_a_builtin() {
        let source = "function sqr%(x%)\n    return x% * x%\nend function\nprint sqr%(4)\nend\n";
        let diagnostics =
            compile_source("shadow.bcl", source).expect_err("shadowing SQR should fail");
        let msg = diagnostics
            .into_iter()
            .map(|d| d.to_string())
            .collect::<String>();
        assert!(
            msg.contains("same name as the built-in `SQR`"),
            "unexpected diagnostics: {msg}"
        );
    }

    #[test]
    fn procedure_cannot_shadow_a_builtin() {
        let source = "procedure len(s$)\n    print s$\nend procedure\nlen(\"hi\")\nend\n";
        let diagnostics =
            compile_source("shadow2.bcl", source).expect_err("shadowing LEN should fail");
        let msg = diagnostics
            .into_iter()
            .map(|d| d.to_string())
            .collect::<String>();
        assert!(
            msg.contains("same name as the built-in `LEN`"),
            "unexpected diagnostics: {msg}"
        );
    }

    #[test]
    fn ordinary_function_names_are_unaffected_by_the_builtin_check() {
        let source = "function larger%(a%, b%)\n    if a% > b% then\n        return a%\n    end if\n    return b%\nend function\nprint larger%(1, 2)\nend\n";
        compile_source("ok.bcl", source).expect("a non-colliding function name should compile");
    }

    // ── --strict-vars / --strict-vars-warn ──────────────────────────────

    #[test]
    fn strict_vars_rejects_an_undeclared_variable() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("typo.bcl");
        std::fs::write(
            &path,
            "program typo\ndim score%\nscore% = 10\nprint scroe%\nend\n",
        )
        .unwrap();

        let options = CompileOptions {
            strict_vars: true,
            ..CompileOptions::new()
        };
        let result = compile_file(&path, &options);
        assert!(result.is_err(), "a misspelled, undeclared name should fail");
        let msg = result
            .unwrap_err()
            .into_iter()
            .map(|d| d.to_string())
            .collect::<String>();
        assert!(msg.contains("`scroe%` is used without a"), "unexpected: {msg}");
    }

    #[test]
    fn strict_vars_accepts_dim_declare_const_for_and_params() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("clean.bcl");
        std::fs::write(
            &path,
            "program clean\n\
             declare score%, y%(20)\n\
             dim total%\n\
             const bonus% = 5\n\
             score% = 10\n\
             y%(1) = 5\n\
             total% = score% + y%(1) + bonus%\n\
             for i% = 1 to 5\n\
             \x20   print i%\n\
             end for\n\
             function double%(n%)\n\
             \x20   declare result%\n\
             \x20   result% = n% * 2\n\
             \x20   return result%\n\
             end function\n\
             print double%(total%)\n\
             end\n",
        )
        .unwrap();

        let options = CompileOptions {
            strict_vars: true,
            ..CompileOptions::new()
        };
        compile_file(&path, &options).expect(
            "dim/declare'd names, const, a for-loop counter, and a parameter should all pass",
        );
    }

    #[test]
    fn strict_vars_ignores_a_required_librarys_own_body() {
        // com.bascal.stdlib.ucase itself doesn't dim its locals -- turning
        // on --strict-vars for a program that merely requires it should
        // never fail because of that library's own internals.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("uses_stdlib.bcl");
        std::fs::write(
            &path,
            "program usesstdlib\n\
             require com.bascal.stdlib.ucase\n\
             dim s$\n\
             s$ = \"hello\"\n\
             print ucase$(s$)\n\
             end\n",
        )
        .unwrap();

        let options = CompileOptions {
            strict_vars: true,
            ..CompileOptions::new()
        };
        compile_file(&path, &options)
            .expect("a required stdlib call should not be misread as an undeclared variable");
    }

    #[test]
    fn strict_vars_warn_prints_but_does_not_fail() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("typo2.bcl");
        std::fs::write(
            &path,
            "program typo2\ndim score%\nscore% = 10\nprint scroe%\nend\n",
        )
        .unwrap();

        let options = CompileOptions {
            strict_vars_warn: true,
            ..CompileOptions::new()
        };
        compile_file(&path, &options)
            .expect("--strict-vars-warn should still succeed despite the finding");
    }

    #[test]
    fn without_strict_vars_an_undeclared_variable_is_fine() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("implicit.bcl");
        std::fs::write(&path, "program implicit\nx% = 5\nprint x%\nend\n").unwrap();
        compile_file(&path, &CompileOptions::new())
            .expect("implicit variable creation must still work when strict_vars is off");
    }

    #[test]
    fn shared_file_with_statements_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let prog_path = dir.path().join("prog.bcl");
        let bad_shared = dir.path().join("badstate.bcl");

        std::fs::write(&prog_path, "program prog shared badstate\nEND\n").unwrap();
        std::fs::write(&bad_shared, "shared badstate\n\ndim score%\nPRINT 1\n").unwrap();

        let result = compile_file(&prog_path, &CompileOptions::new());
        assert!(result.is_err());
        let msg = result
            .unwrap_err()
            .into_iter()
            .map(|d| d.to_string())
            .collect::<String>();
        assert!(msg.contains("may only contain DIM declarations"));
    }

    #[test]
    fn shared_header_declares_common_vars() {
        let dir = tempfile::tempdir().unwrap();
        let prog_path = dir.path().join("show.bcl");
        let shared_path = dir.path().join("state.bcl");

        std::fs::write(&prog_path, "program show shared state\nprint count%\nend\n").unwrap();
        std::fs::write(
            &shared_path,
            "shared state\n\ndim count%\ndim label$\ndim scores%()\n",
        )
        .unwrap();

        let output = compile_file(&prog_path, &CompileOptions::new())
            .expect("shared header + dim should compile");
        assert!(output.contains("COMMON count%, label$, scores%()"));
    }

    #[test]
    fn shared_header_name_must_match_the_filename_it_was_loaded_as() {
        let dir = tempfile::tempdir().unwrap();
        let prog_path = dir.path().join("show.bcl");
        let shared_path = dir.path().join("state.bcl");

        std::fs::write(&prog_path, "program show shared state\nend\n").unwrap();
        std::fs::write(&shared_path, "shared wrongname\n\ndim count%\n").unwrap();

        let result = compile_file(&prog_path, &CompileOptions::new());
        assert!(result.is_err());
        let msg = result
            .unwrap_err()
            .into_iter()
            .map(|d| d.to_string())
            .collect::<String>();
        assert!(msg.contains("declares `shared wrongname`"));
    }

    #[test]
    fn a_file_cannot_have_both_program_and_shared_declarations() {
        let source = "program foo\nshared bar\nend\n";
        let result = compile_source("both.bcl", source);
        assert!(result.is_err());
        let msg = result
            .unwrap_err()
            .into_iter()
            .map(|d| d.to_string())
            .collect::<String>();
        assert!(msg.contains("cannot have both a `program` declaration and a `shared` declaration"));
    }

    #[test]
    fn shared_declaration_is_rejected_outside_a_shared_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("stray.bcl");
        std::fs::write(&path, "shared lonely\n\ndim x%\n").unwrap();

        let result = compile_file(&path, &CompileOptions::new());
        assert!(result.is_err());
        let msg = result
            .unwrap_err()
            .into_iter()
            .map(|d| d.to_string())
            .collect::<String>();
        assert!(msg.contains("`shared` declaration is only valid in shared-variable files"));
    }

    #[test]
    fn dim_with_empty_parens_stays_an_array_in_generated_output() {
        // Regression test: `dim arr%()` used to collapse to the same
        // `DIM arr%` as a plain scalar `dim arr%`, because Statement::Dim
        // only tracked `sizes` (empty either way) with no separate signal
        // for "parens were written at all". Surfaced by the shared/dim work
        // above, since a shared file needs to tell scalar and unbounded-array
        // dims apart to emit `arr%()` vs `arr%` in the COMMON line -- fixed
        // by giving Statement::Dim its own `is_array` field.
        let source = "dim x%\ndim arr%()\nend\n";
        let output = compile_source("dimarr.bcl", source).expect("should compile");
        assert!(output.contains("DIM x%\n"));
        assert!(output.contains("DIM arr%()"));
    }

    #[test]
    fn lowers_basic_file_io_statements() {
        // Mixed-case keywords and variable names: transpiler normalises vars to lowercase.
        let source = r#"OPEN InputFile$ FOR INPUT AS #1
LINE INPUT #1, CurrentLine$
PRINT #2, CurrentLine$
CLOSE #1
END
"#;

        let output = compile_source("io.bcl", source).expect("sample should compile");
        assert!(output.contains("OPEN inputfile$ FOR INPUT AS #1"));
        assert!(output.contains("LINE INPUT #1, currentline$"));
        assert!(output.contains("PRINT #2, currentline$"));
        assert!(output.contains("CLOSE #1"));
    }

    #[test]
    fn compiles_random_access_file_io() {
        let source = r#"open DataFile$ for random as #1 len = 128
field #1, 4 as RecNum$, 124 as RecData$
lset RecNum$ = mki%(1)
lset RecData$ = "hello"
put #1, 1
get #1, 1
seek #1, 2
close #1
end
"#;
        let output =
            compile_source("random.bcl", source).expect("random-access sample should compile");
        assert!(output.contains("OPEN datafile$ FOR RANDOM AS #1 LEN = 128"));
        assert!(output.contains("FIELD #1, 4 AS recnum$, 124 AS recdata$"));
        assert!(output.contains("LSET recnum$ = MKI%(1)"));
        assert!(output.contains("LSET recdata$ = \"hello\""));
        assert!(output.contains("PUT #1, 1"));
        assert!(output.contains("GET #1, 1"));
        assert!(output.contains("SEEK #1, 2"));
        assert!(output.contains("CLOSE #1"));
    }

    // ── record/file DSL ────────────────────────────────────────────────

    fn record_dsl_source() -> &'static str {
        r#"record Student
    id:    int16
    name:  string(20)
    score: float64
end record

file db as Student = open("tutorial_students.dat")

db[1] = { id: 1, name: "Alice", score: 95.0 }

for i = 3 downto 1
    let s = db[i]
    print "[" + s.id + "] " + s.name + " " + s.score
end for

db[2].score = 61.5

db.close()

end
"#
    }

    #[test]
    fn record_file_declares_open_and_field() {
        let output = compile_source("rec.bcl", record_dsl_source()).expect("should compile");
        assert!(output.contains(r#"OPEN "tutorial_students.dat" FOR RANDOM AS #1 LEN = 30"#));
        assert!(output.contains("FIELD #1, 2 AS dbIdBuf$, 20 AS dbNameBuf$, 8 AS dbScoreBuf$"));
    }

    #[test]
    fn record_whole_write_lowers_to_lset_and_put() {
        let output = compile_source("rec.bcl", record_dsl_source()).expect("should compile");
        // MKI$/MKL$/MKS$/MKD$ always return a string -- a destination-type
        // suffix like MKI% or MKD# isn't a real MBASIC/BASCOM function.
        assert!(output.contains("LSET dbIdBuf$ = MKI$(1)"));
        assert!(output.contains(r#"LSET dbNameBuf$ = "Alice""#));
        assert!(output.contains("LSET dbScoreBuf$ = MKD$(95)"));
        assert!(output.contains("PUT #1, 1"));
    }

    #[test]
    fn record_whole_read_lowers_to_get_and_unpack() {
        let output = compile_source("rec.bcl", record_dsl_source()).expect("should compile");
        assert!(output.contains("GET #1, i"));
        // CVI/CVL/CVS/CVD take no suffix at all on real MBASIC/BASCOM.
        assert!(output.contains("sid% = CVI(dbIdBuf$)"));
        // RTRIM$ isn't a real MBASIC/BASCOM builtin either -- string fields
        // are unpacked through an inline LEN/MID$/LEFT$ trim loop instead.
        assert!(
            output.contains("snametrimi% = LEN(dbNameBuf$)"),
            "string field unpacking should trim inline, not call RTRIM$:\n{output}"
        );
        assert!(
            !output.to_ascii_uppercase().contains("RTRIM$"),
            "RTRIM$ isn't valid on real MBASIC/BASCOM:\n{output}"
        );
        assert!(output.contains("sname$ = LEFT$(dbNameBuf$, snametrimi%)"));
        assert!(output.contains("sscore# = CVD(dbScoreBuf$)"));
    }

    #[test]
    fn record_dotted_field_access_resolves_to_unpacked_scalar() {
        let output = compile_source("rec.bcl", record_dsl_source()).expect("should compile");
        // s.name is already a string — must not be STR$()-wrapped.
        assert!(output.contains("+ sname$"));
        assert!(!output.contains("STR$(sname$)"));
        // s.id / s.score are numeric and combined with strings via `+` — must be wrapped.
        assert!(output.contains("STR$(sid%)"));
        assert!(output.contains("STR$(sscore#)"));
    }

    #[test]
    fn record_partial_update_lowers_to_get_lset_put() {
        let output = compile_source("rec.bcl", record_dsl_source()).expect("should compile");
        assert!(output.contains("GET #1, 2"));
        assert!(output.contains("LSET dbScoreBuf$ = MKD$(61.5)"));
        assert!(output.contains("PUT #1, 2"));
    }

    #[test]
    fn record_close_lowers_to_close_statement() {
        let output = compile_source("rec.bcl", record_dsl_source()).expect("should compile");
        assert!(output.contains("CLOSE #1"));
    }

    #[test]
    fn record_field_buffers_stay_global_inside_procedures() {
        // db[...]/let-bound record access used from inside a procedure body
        // must LSET/GET the exact same FIELD-bound buffer names the
        // top-level `file` declaration bound -- not per-procedure locals
        // that were never FIELD-bound and so silently never touch the file.
        let source = r#"record Item
    name: string(10)
    qty:  int16
end record

file items as Item = open("probe.dat")

procedure addItem(n$, q%)
    items[1] = { name: n$, qty: q% }
end procedure

procedure showItem()
    let s = items[1]
    print s.name + " " + str$(s.qty)
end procedure

addItem("widget", 5)
showItem()
items.close()
end
"#;
        let output = compile_source("probe.bcl", source).expect("should compile");
        assert!(
            output.contains("FIELD #1, 10 AS itemsNameBuf$, 2 AS itemsQtyBuf$"),
            "unexpected FIELD line:\n{output}"
        );
        assert!(
            output.contains("LSET itemsNameBuf$ = "),
            "addItem should LSET the top-level FIELD buffer, not a per-procedure local:\n{output}"
        );
        assert!(
            output.contains("LSET itemsQtyBuf$ = "),
            "addItem should LSET the top-level FIELD buffer, not a per-procedure local:\n{output}"
        );
        assert!(
            // RTRIM$ isn't a real MBASIC/BASCOM builtin -- string fields are
            // unpacked through an inline LEN/MID$/LEFT$ trim loop instead,
            // and CVI/CVL/CVS/CVD take no suffix at all.
            output.contains("LEN(itemsNameBuf$)") && output.contains("CVI(itemsQtyBuf$)"),
            "showItem should read back from the same top-level FIELD buffers:\n{output}"
        );
        assert!(
            !output.contains("additemItemsNameBuf") && !output.contains("showitemItemsNameBuf"),
            "FIELD buffer names must never be re-namespaced per procedure:\n{output}"
        );
    }

    #[test]
    fn record_downto_lowers_to_step_negative_one() {
        let output = compile_source("rec.bcl", record_dsl_source()).expect("should compile");
        assert!(output.contains("FOR i = 3 TO 1 STEP -1"));
    }

    #[test]
    fn record_multiple_files_allocate_sequential_channels() {
        let source = r#"record A
    n: int16
end record

record B
    n: int16
end record

file first as A = open("a.dat")
file second as B = open("b.dat")

first.close()
second.close()
end
"#;
        let output = compile_source("multi.bcl", source).expect("should compile");
        assert!(output.contains("OPEN \"a.dat\" FOR RANDOM AS #1 LEN = 2"));
        assert!(output.contains("OPEN \"b.dat\" FOR RANDOM AS #2 LEN = 2"));
        assert!(output.contains("CLOSE #1"));
        assert!(output.contains("CLOSE #2"));
    }

    #[test]
    fn record_batched_field_mutation_is_one_get_one_put() {
        // `s.field = value` after `let s = db[i]` must touch only the
        // in-memory scalar (no GET/PUT); the write-back happens exactly
        // once, at `db[i] = s`.
        let source = r#"record Student
    id:    int16
    name:  string(20)
    score: float64
end record

file db as Student = open("students.dat")

let s = db[1]
s.name = "Alicia"
s.score = 99.0
db[1] = s

end
"#;
        let output = compile_source("batch.bcl", source).expect("should compile");
        assert_eq!(
            output.matches("GET #1").count(),
            1,
            "exactly one GET for the whole batch"
        );
        assert_eq!(
            output.matches("PUT #1").count(),
            1,
            "exactly one PUT for the whole batch"
        );
        assert!(
            output.contains(r#"sname$ = "Alicia""#),
            "field mutation is a plain in-memory assignment"
        );
        assert!(
            output.contains("sscore# = 99"),
            "field mutation is a plain in-memory assignment"
        );
        assert!(output.contains("LSET dbIdBuf$ = MKI$(sid%)"));
        assert!(output.contains("LSET dbNameBuf$ = sname$"));
        assert!(output.contains("LSET dbScoreBuf$ = MKD$(sscore#)"));
    }

    #[test]
    fn record_write_back_rejects_mismatched_record_type() {
        let source = r#"record A
    n: int16
end record

record B
    n: int16
end record

file fa as A = open("a.dat")
file fb as B = open("b.dat")

let s = fa[1]
fb[1] = s
end
"#;
        let err = compile_source("mismatch.bcl", source).expect_err("should reject type mismatch");
        assert!(err.iter().any(|d| d.message.contains("holds `B` records")));
    }

    #[test]
    fn record_partial_literal_missing_fields_inserts_get() {
        let source = r#"record Student
    id:    int16
    name:  string(20)
    score: float64
end record
file db as Student = open("students.dat")
db[2] = ?{ score: 61.5 }
end
"#;
        let output = compile_source("partial.bcl", source).expect("should compile");
        assert!(
            output.contains("GET #1, 2"),
            "partial write covering only some fields needs a GET"
        );
        assert!(output.contains("LSET dbScoreBuf$ = MKD$(61.5)"));
        assert!(
            !output.contains("LSET dbIdBuf$"),
            "unmentioned fields must not be LSET"
        );
        assert!(
            !output.contains("LSET dbNameBuf$"),
            "unmentioned fields must not be LSET"
        );
        assert!(output.contains("PUT #1, 2"));
    }

    #[test]
    fn record_partial_write_requires_an_existing_record_in_basic_output() {
        let source = r#"record Student
    id: int16
    name: string(20)
    score: float64
    faculty: string(20)
end record
file db as Student = open("students.dat")
db[2] = ?{ name: "Bob" }
end
"#;
        let output =
            compile_source("partial_exists.bcl", source).expect("partial update should compile");
        assert!(
            output.contains("IF LOF(#1) < (2) * 50 THEN ERROR 63"),
            "a partial update must reject a missing fixed-length record before GET:\n{output}"
        );
    }

    #[test]
    fn record_partial_literal_covering_every_field_skips_get() {
        // Static analysis: a `?{ ... }` that happens to name every declared
        // field needs no GET, exactly like a plain `{ ... }` literal.
        let source = r#"record Student
    id:    int16
    name:  string(20)
    score: float64
end record
file db as Student = open("students.dat")
db[3] = ?{ id: 3, name: "Carol", score: 78.0 }
end
"#;
        let output = compile_source("partial_full.bcl", source).expect("should compile");
        assert!(
            !output.contains("GET #1"),
            "covering every field needs no GET"
        );
        assert!(output.contains("LSET dbIdBuf$ = MKI$(3)"));
        assert!(output.contains("LSET dbNameBuf$ = \"Carol\""));
        assert!(output.contains("LSET dbScoreBuf$ = MKD$(78)"));
        assert!(output.contains("PUT #1, 3"));
    }

    #[test]
    fn record_full_literal_still_rejects_missing_field() {
        // `{ ... }` (no `?`) keeps the completeness safety net.
        let source = r#"record A
    n: int16
    m: int16
end record
file db as A = open("a.dat")
db[1] = { n: 1 }
end
"#;
        let err =
            compile_source("full.bcl", source).expect_err("should reject incomplete full literal");
        assert!(err.iter().any(|d| d.message.contains("missing field `m`")));
        assert!(
            err.iter().any(|d| d.message.contains("?{")),
            "error should point at the partial alternative"
        );
    }

    #[test]
    fn record_partial_literal_still_rejects_unknown_field() {
        let source = r#"record A
    n: int16
end record
file db as A = open("a.dat")
db[1] = ?{ bogus: 1 }
end
"#;
        let err = compile_source("bogus.bcl", source)
            .expect_err("should reject unknown field even in a partial literal");
        assert!(err.iter().any(|d| d.message.contains("bogus")));
    }

    #[test]
    fn record_rejects_unknown_field_in_literal() {
        let source = r#"record A
    n: int16
end record
file db as A = open("a.dat")
db[1] = { n: 1, bogus: 2 }
end
"#;
        let err = compile_source("bad.bcl", source).expect_err("should reject unknown field");
        assert!(err.iter().any(|d| d.message.contains("bogus")));
    }

    #[test]
    fn record_rejects_missing_field_in_literal() {
        let source = r#"record A
    n: int16
    m: int16
end record
file db as A = open("a.dat")
db[1] = { n: 1 }
end
"#;
        let err = compile_source("bad.bcl", source).expect_err("should reject missing field");
        assert!(err.iter().any(|d| d.message.contains("missing field")));
    }

    #[test]
    fn record_rejects_oversized_string_literal() {
        let source = r#"record A
    name: string(3)
end record
file db as A = open("a.dat")
db[1] = { name: "TooLong" }
end
"#;
        let err =
            compile_source("bad.bcl", source).expect_err("should reject oversized string literal");
        assert!(err.iter().any(|d| d.message.contains("exceeds string(3)")));
    }

    #[test]
    fn record_rejects_string_literal_for_numeric_field() {
        let source = r#"record A
    n: int16
end record
file db as A = open("a.dat")
db[1] = { n: "oops" }
end
"#;
        let err =
            compile_source("bad.bcl", source).expect_err("should reject string for numeric field");
        assert!(err.iter().any(|d| d.message.contains("numeric")));
    }

    #[test]
    fn record_rejects_numeric_literal_for_string_field() {
        let source = r#"record A
    name: string(10)
end record
file db as A = open("a.dat")
db[1] = { name: 5 }
end
"#;
        let err =
            compile_source("bad.bcl", source).expect_err("should reject numeric for string field");
        assert!(err.iter().any(|d| d.message.contains("string(N)")));
    }

    #[test]
    fn record_rejects_unknown_record_type() {
        let source = r#"file db as Nope = open("a.dat")
end
"#;
        let err = compile_source("bad.bcl", source).expect_err("should reject unknown record type");
        assert!(err
            .iter()
            .any(|d| d.message.contains("unknown record type")));
    }

    #[test]
    fn record_rejects_undeclared_file_var() {
        let source = r#"record A
    n: int16
end record
db[1] = { n: 1 }
end
"#;
        let err = compile_source("bad.bcl", source).expect_err("should reject undeclared file var");
        assert!(err
            .iter()
            .any(|d| d.message.contains("not a declared `file`")));
    }

    #[test]
    fn record_rejects_bare_field_read_without_let() {
        let source = r#"record A
    n: int16
end record
file db as A = open("a.dat")
print db[1].n
end
"#;
        let err = compile_source("bad.bcl", source).expect_err("should reject bare field read");
        assert!(!err.is_empty());
    }

    #[test]
    fn print_supports_semicolon_separator_and_direct_numeric() {
        // Semicolons between items: no gap, no trailing newline when trailing.
        // Commas between items: tab-zone advance.
        // Numeric expressions printed directly without str$().
        let source = r#"x% = 42
print "value: "; x%
print "a"; "b"; "c"
print "col1", "col2"
print "no newline";
print x%, "done"
end
"#;
        let output = compile_source("print.bcl", source).expect("should compile");
        assert!(output.contains(r#"PRINT "value: "; x%"#));
        assert!(output.contains(r#"PRINT "a"; "b"; "c""#));
        assert!(output.contains(r#"PRINT "col1", "col2""#));
        assert!(output.contains(r#"PRINT "no newline";"#));
        assert!(output.contains(r#"PRINT x%, "done""#));
    }

    #[test]
    fn print_using_formats_output() {
        let source = "amount! = 1234.5\n\
count% = 7\n\
print using \"####.##\"; amount!\n\
print using \"Item ##\"; count%\n\
lprint using \"####.##\"; amount!\n\
open \"out.txt\" for output as #1\n\
print #1, using \"####.##\"; amount!\n\
close #1\n\
end\n";
        let output = compile_source("fmt.bcl", source).expect("should compile");
        assert!(output.contains("PRINT USING \"####.##\"; amount!"));
        assert!(output.contains("PRINT USING \"Item ##\"; count%"));
        assert!(output.contains("LPRINT USING \"####.##\"; amount!"));
        assert!(output.contains("PRINT #1, USING \"####.##\"; amount!"));
    }

    #[test]
    fn option_base_and_erase() {
        let source = r#"option base 1
dim scores%(10)
dim names$(10)
dim grid%(4, 4)
' ... use arrays ...
erase scores%
erase names$, grid%
end
"#;
        let output = compile_source("ob.bcl", source).expect("should compile");
        assert!(output.contains("OPTION BASE 1"));
        assert!(output.contains("ERASE scores%"));
        assert!(output.contains("ERASE names$, grid%"));
    }

    #[test]
    fn error_handling_statements() {
        // `on error goto`/`resume` targets must be labels (not raw line
        // numbers) — BASCAL manages line numbers itself. `on error goto 0`
        // is the one numeric exception: the sentinel that disables the trap.
        let source = r#"' set and clear error trap
on error goto handler
on error goto 0
' resume forms
resume
resume next
resume handler
' trigger a synthetic error
error 53
end

handler:
print "handled"
"#;
        let output = compile_source("err.bcl", source).expect("should compile");
        let handler_line = output
            .lines()
            .find(|line| line.contains("PRINT \"handled\""))
            .and_then(|line| line.split_whitespace().next())
            .expect("handler line should be numbered");
        assert!(output.contains(&format!("ON ERROR GOTO {handler_line}")));
        assert!(output.contains(&format!("RESUME {handler_line}")));
        assert!(output.contains("ON ERROR GOTO 0"));
        assert!(output.contains("RESUME\n") || output.ends_with("RESUME"));
        assert!(output.contains("RESUME NEXT"));
        assert!(output.contains("ERROR 53"));
    }

    #[test]
    fn do_loop_until_runs_body_before_testing_the_condition() {
        // Post-check form: `loop until` tests after the body, so it must
        // jump back to the top of the loop -- not skip the body the way a
        // pre-check `do until` would for an already-true condition.
        let source = r#"k% = 99
do
    print k%
    k% = k% + 1
loop until k% > 3
end
"#;
        let output = compile_source("postcheck.bcl", source).expect("should compile");
        // No pre-check guard before the body: the very first statement
        // inside the loop is PRINT, not an IF testing the condition.
        let print_idx = output.find("PRINT k%").expect("body should be present");
        let first_if_idx = output.find("IF (");
        assert!(
            first_if_idx.is_none_or(|i| i > print_idx),
            "post-check loop must not test the condition before running the body"
        );
        // The condition jump must loop back, i.e. target a label that
        // resolves to a line number at or before PRINT k%'s own line.
        assert!(output.contains("k% > 3"));
    }

    #[test]
    fn do_loop_while_repeats_while_condition_holds() {
        let source = r#"j% = 1
do
    print j%
    j% = j% + 1
loop while j% <= 3
end
"#;
        let output = compile_source("postcheck_while.bcl", source).expect("should compile");
        assert!(
            output.contains("<> 0 THEN GOTO"),
            "loop while repeats when the condition is still true"
        );
    }

    #[test]
    fn bare_exit_resolves_to_the_innermost_enclosing_loop_kind() {
        // for/next is a native BASIC FOR...NEXT block, so `exit` inside it
        // must become EXIT FOR; while/do are GOTO chains, so `exit` there
        // must become a GOTO to the loop's own end label -- and a nested
        // do inside a for must resolve to the do's GOTO, not the outer
        // for's EXIT FOR, since exit always targets the innermost loop.
        let source = r#"for i% = 1 to 5
    do
        print i%
        exit
    end do
    if i% = 3 then
        exit
    end if
end for
end
"#;
        let output = compile_source("nested_exit.bcl", source).expect("should compile");
        assert!(
            output.contains("EXIT FOR"),
            "exit directly inside for must become EXIT FOR"
        );
        // The exit inside the inner `do` must NOT have produced a second
        // EXIT FOR -- only exactly one EXIT FOR should appear anywhere.
        assert_eq!(output.matches("EXIT FOR").count(), 1);
        assert!(
            output.contains("GOTO"),
            "exit inside the inner do must become a GOTO, not EXIT FOR"
        );
    }

    #[test]
    fn exit_outside_any_loop_is_a_soft_warning_not_a_hard_error() {
        let source = "print \"before\"\nexit\nprint \"after\"\nend\n";
        let output = compile_source("bad_exit.bcl", source).expect("should compile");
        assert!(output.contains("warning: EXIT outside of a loop"));
    }

    #[test]
    fn single_line_if_else_does_not_get_swallowed_by_a_greedy_print_statement() {
        // Regression test: PRINT's own argument-list loop only used to stop
        // at a real newline/colon/EOF, so `print "a" else print "b"` used
        // to parse `else`, `print`, and `"b"` as three more PRINT
        // arguments instead of recognizing `else` as the boundary --
        // corrupting the output into one garbled PRINT with no real
        // else-branch at all.
        let source = "if x% > 100 then print \"big\" else print \"small\"\nend\n";
        let output = compile_source("single_line_else.bcl", source).expect("should compile");
        assert!(output.contains(r#"PRINT "big""#));
        assert!(output.contains(r#"PRINT "small""#));
        assert!(
            !output.contains("else"),
            "else must not leak into the generated output"
        );
    }

    #[test]
    fn compound_assignment_desugars_to_binary_self_assignment() {
        let source = "a% = 1\na% += 4\na% -= 1\na% *= 3\na% /= 2\nprint a%\nend\n";
        let output = compile_source("compound.bcl", source).expect("should compile");
        assert!(output.contains("a% = a% + 4"));
        assert!(output.contains("a% = a% - 1"));
        assert!(output.contains("a% = a% * 3"));
        assert!(output.contains("a% = a% / 2"));
    }

    #[test]
    fn compound_assignment_works_on_array_elements() {
        let source = "dim scores%(10)\nscores%(2) = 5\nscores%(2) += 3\nend\n";
        let output = compile_source("compound_arr.bcl", source).expect("should compile");
        assert!(output.contains("scores%(2) = scores%(2) + 3"));
    }

    #[test]
    fn true_and_false_are_sugar_for_minus_one_and_zero() {
        let source =
            "found% = TRUE\ndone% = FALSE\nif found% = TRUE then\n    print \"yes\"\nend if\nend\n";
        let output = compile_source("boolsugar.bcl", source).expect("should compile");
        assert!(output.contains("found% = -1"));
        assert!(output.contains("done% = 0"));
        assert!(output.contains("(found% = -1)"));
    }

    #[test]
    fn multi_name_dim_splits_into_separate_dim_statements() {
        let source = "dim a%, b%(3), c$\na% = 1\nb%(0) = 2\nc$ = \"hi\"\nend\n";
        let output = compile_source("multidim.bcl", source).expect("should compile");
        assert!(output.contains("DIM a%"));
        assert!(output.contains("DIM b%(3)"));
        assert!(output.contains("DIM c$"));
    }

    #[test]
    fn multi_name_dim_inside_single_line_if_stays_inside_the_if_body() {
        // Regression test: multi-name DIM desugars into more than one
        // Statement via the parser's pending-statement queue. A single-line
        // `if`'s body loop only used to check its stopping condition right
        // after the *first* dequeued statement, so a queued second/third
        // DIM used to leak out and attach to the wrong (outer) block instead
        // of staying inside the `if`.
        let source =
            "x% = 5\nif x% > 0 then dim p%, q% : p% = 1 : q% = 2 : print p% + q%\nprint \"after\"\nend\n";
        let output = compile_source("dim_in_if.bcl", source).expect("should compile");
        let if_line = output
            .lines()
            .position(|l| l.contains("IF ("))
            .expect("if line");
        let end_if_line = output
            .lines()
            .position(|l| l.contains("REM END IF"))
            .expect("end if line");
        let body: Vec<&str> = output.lines().collect::<Vec<_>>()[if_line..end_if_line].to_vec();
        assert!(body.iter().any(|l| l.contains("DIM p%")));
        assert!(body.iter().any(|l| l.contains("DIM q%")));
    }

    #[test]
    fn goto_label_resolves_to_a_real_line_number() {
        let source = r#"print "before"
goto skip
print "should not print"
skip:
print "after"
end
"#;
        let output = compile_source("label.bcl", source).expect("should compile");
        assert!(
            !output.contains("skip"),
            "label text must not leak into generated output"
        );
        let skip_line = output
            .lines()
            .find(|line| line.contains("PRINT \"after\""))
            .and_then(|line| line.split_whitespace().next())
            .expect("skip target should be numbered");
        assert!(output.contains(&format!("GOTO {skip_line}")));
    }

    #[test]
    fn label_name_matching_string_literal_text_is_not_corrupted() {
        // Regression test: label -> line-number substitution used to be a
        // blind `str::replace` across the whole line, which was safe only
        // because transpiler-internal labels use distinctive prefixed names
        // (WHILE_0001_TOP, ...). User labels are short, ordinary words, so
        // a label named `done` must not corrupt `PRINT "...done..."` text
        // on some unrelated line that just happens to contain that word.
        let source = r#"goto done
print "we are done, done, done!"
done:
print "finished"
end
"#;
        let output = compile_source("collide.bcl", source).expect("should compile");
        assert!(output.contains(r#"PRINT "we are done, done, done!""#));
    }

    #[test]
    fn mid_statement_form() {
        // MID$(str$, start[, length]) = replacement$ -- same-length substring
        // replace. Real MBASIC/BASCOM 2.00 has no MID$ assignment statement
        // at all (it's a later QuickBASIC-era addition), so this must not
        // pass through as raw `MID$(...) = ...` -- it needs to transpile to a
        // call to BASCAL's own com.bascal.stdlib.midAssign helper, same as
        // every other real-BASCOM incompatibility this transpiler works
        // around.
        let source = r#"s$ = "Hello World"
mid$(s$, 7, 5) = "BASIC"
mid$(s$, 1) = "Goodbye"
print s$
end
"#;
        let output = compile_source("mid.bcl", source).expect("should compile");
        assert!(
            !output.to_ascii_uppercase().contains("MID$(S$, 7, 5) ="),
            "MID$ assignment must not pass through as a raw statement -- real \
             MBASIC/BASCOM has no such statement:\n{output}"
        );
        assert!(
            output.contains("' function midassign$"),
            "expected the com.bascal.stdlib.midAssign helper to be auto-injected:\n{output}"
        );
        // Three-argument form.
        assert!(output.contains("midassignTarget0$ = s$"));
        assert!(output.contains("midassignStart0% = 7"));
        assert!(output.contains("midassignLen0% = 5"));
        assert!(output.contains(r#"midassignValue0$ = "BASIC""#));
        assert!(output.contains("s$ = midassignResult0$"));
        // Two-argument form: omitted length behaves as LEN(replacement$).
        assert!(output.contains(r#"midassignLen0% = LEN("Goodbye")"#));
        // Both call sites share the same GOSUB target -- one subroutine body.
        let gosub_targets: std::collections::HashSet<&str> = output
            .lines()
            .filter_map(|l| l.trim().strip_prefix("GOSUB "))
            .collect();
        assert_eq!(
            gosub_targets.len(),
            1,
            "expected one shared subroutine:\n{output}"
        );
        assert_eq!(
            output
                .matches(&format!("GOSUB {}", gosub_targets.iter().next().unwrap()))
                .count(),
            2,
            "expected both call sites to GOSUB the shared subroutine:\n{output}"
        );
    }

    #[test]
    fn mid_assign_target_index_is_evaluated_only_once() {
        // `target` may be an array element whose index has a side effect
        // (here, `nextIndex%()` advances a global counter each time it's
        // called). MID$ assignment transpilation must evaluate that index
        // exactly once -- reusing the already-rendered target text for
        // both the helper call and the write-back -- not once per use.
        let source = r#"dim names$(3)
names$(0) = "aaaaa"
i% = 0

function nextIndex%()
    global i%
    result% = i%
    i% = i% + 1
    return result%
end function

mid$(names$(nextIndex%()), 1, 3) = "XYZ"
print names$(0)
print i%
end
"#;
        let output = compile_source("mid_index_side_effect.bcl", source).expect("should compile");

        // Exactly one call into nextIndex%'s own subroutine label.
        let nextindex_label = output
            .lines()
            .find(|l| l.trim_end() == "' function nextindex%()")
            .and_then(|_| {
                output
                    .lines()
                    .skip_while(|l| l.trim_end() != "' function nextindex%()")
                    .nth(1)
                    .and_then(|l| l.trim().split_whitespace().next())
            })
            .expect("nextindex%'s label line should be the line right after its comment");
        assert_eq!(
            output.matches(&format!("GOSUB {nextindex_label}")).count(),
            1,
            "target's array index must be evaluated exactly once:\n{output}"
        );

        // The same rendered index expression is reused verbatim for the
        // write-back, not re-evaluated.
        let target_text = "names$(nextindexResult0%)";
        assert!(
            output.contains(&format!("midassignTarget0$ = {target_text}")),
            "expected the call argument to reuse the already-evaluated index:\n{output}"
        );
        assert!(
            output.contains(&format!("{target_text} = midassignResult0$")),
            "expected the write-back to reuse the already-evaluated index, not call \
             nextIndex%() again:\n{output}"
        );
    }

    // ── sequential file handle DSL ──────────────────────────────────────

    #[test]
    fn sequential_file_handle_opens_writes_reads_and_closes() {
        let source = r#"file scores = open("scores.csv") for output
scores.write("Ada", 98.5)
scores.close()

file scores2 = open("scores.csv") for input
while not scores2.eof()
    scores2.read(name$, score!)
    print name$; ": "; score!
end while
scores2.close()
end
"#;
        let output = compile_source("seq.bcl", source).expect("should compile");
        assert!(
            output.contains(r#"OPEN "scores.csv" FOR OUTPUT AS #1"#),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains(r#"WRITE #1, "Ada", 98.5"#),
            "unexpected output:\n{output}"
        );
        assert!(output.contains("CLOSE #1"), "unexpected output:\n{output}");
        assert!(
            output.contains(r#"OPEN "scores.csv" FOR INPUT AS #2"#),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("NOT (EOF(2))"),
            "`.eof()` should compile straight to the real EOF() builtin:\n{output}"
        );
        assert!(
            output.contains("INPUT #2, name$, score!"),
            "unexpected output:\n{output}"
        );
        assert!(output.contains("CLOSE #2"), "unexpected output:\n{output}");
    }

    #[test]
    fn sequential_file_channel_numbers_are_allocated_automatically() {
        // Two sequential handles and a record file in one program should
        // never collide on a channel number, same guarantee the record
        // DSL alone already gives.
        let source = r#"record Student
    id: int16
end record

file a = open("a.csv") for output
file db as Student = open("db.dat")
file b = open("b.csv") for input
end
"#;
        let output = compile_source("chan.bcl", source).expect("should compile");
        assert!(output.contains(r#"OPEN "a.csv" FOR OUTPUT AS #1"#));
        assert!(output.contains(r#"OPEN "db.dat" FOR RANDOM AS #2"#));
        assert!(output.contains(r#"OPEN "b.csv" FOR INPUT AS #3"#));
    }

    #[test]
    fn sequential_file_read_rejects_a_file_not_opened_for_input() {
        let source = r#"file scores = open("scores.csv") for output
scores.read(name$)
end
"#;
        let diagnostics =
            compile_source("bad.bcl", source).expect_err("read on an output file should fail");
        let msg = diagnostics
            .into_iter()
            .map(|d| d.to_string())
            .collect::<String>();
        assert!(
            msg.contains("needs `scores` opened `for input`") && msg.contains("for output"),
            "unexpected diagnostics: {msg}"
        );
    }

    #[test]
    fn sequential_file_eof_rejects_a_file_not_opened_for_input() {
        let source = r#"file scores = open("scores.csv") for output
while not scores.eof()
end while
end
"#;
        let diagnostics =
            compile_source("bad.bcl", source).expect_err("eof on an output file should fail");
        let msg = diagnostics
            .into_iter()
            .map(|d| d.to_string())
            .collect::<String>();
        assert!(
            msg.contains("needs `scores` opened `for input`"),
            "unexpected diagnostics: {msg}"
        );
    }

    #[test]
    fn sequential_methods_reject_a_record_file() {
        let source = r#"record Student
    id: int16
end record

file db as Student = open("db.dat")
db.read(x%)
end
"#;
        let diagnostics =
            compile_source("bad.bcl", source).expect_err("read on a record file should fail");
        let msg = diagnostics
            .into_iter()
            .map(|d| d.to_string())
            .collect::<String>();
        assert!(
            msg.contains("`db` is a record file"),
            "unexpected diagnostics: {msg}"
        );
    }

    #[test]
    fn record_dsl_methods_reject_a_sequential_file() {
        let source = r#"file scores = open("scores.csv") for output
scores.close()
end
"#;
        // `.close()` is valid on either kind -- confirm it still works here
        // rather than rejecting a sequential file, unlike `[i]`/`.field`.
        let output = compile_source("ok.bcl", source).expect("close should work on any file");
        assert!(output.contains("CLOSE #1"));

        let source = r#"file scores = open("scores.csv") for output
let s = scores[1]
end
"#;
        let diagnostics = compile_source("bad.bcl", source)
            .expect_err("record-only indexing on a sequential file should fail");
        let msg = diagnostics
            .into_iter()
            .map(|d| d.to_string())
            .collect::<String>();
        assert!(
            msg.contains("is a sequential file") && msg.contains("not a record file"),
            "unexpected diagnostics: {msg}"
        );
    }

    #[test]
    fn sequential_write_and_read_are_statement_only() {
        let source = r#"file scores = open("scores.csv") for output
dim x%
x% = scores.write("a")
end
"#;
        let diagnostics = compile_source("bad.bcl", source)
            .expect_err("`.write(...)` used as a value should fail");
        let msg = diagnostics
            .into_iter()
            .map(|d| d.to_string())
            .collect::<String>();
        assert!(
            msg.contains("may only be used as a standalone statement"),
            "unexpected diagnostics: {msg}"
        );
    }

    // ── stdlib functions ────────────────────────────────────────────────
    //
    // LTRIM$, RTRIM$, UCASE$, and LCASE$ aren't real MBASIC/BASCOM 2.00
    // builtins (verified against a real IBM BASIC Compiler 2.00 under
    // dosbox-x -- see com/bascal/stdlib/*.bcl's header comments), so BASCAL
    // ships its own implementations as an ordinary require-able library
    // under com.bascal.stdlib, resolved the same way as any other `require`
    // (see `stdlib_search_roots`) -- not auto-injected by call-site
    // detection, so a program must `require` a stdlib symbol to use it,
    // exactly like `com.bascal.sort.bubbleSort` in the tutorial.

    #[test]
    fn stdlib_functions_are_resolved_via_require() {
        let input = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/stdlib_usage.bcl");
        let output = compile_file(&input, &CompileOptions::new()).expect("should compile");
        for marker in [
            "function ltrim$",
            "function rtrim$",
            "function ucase$",
            "function lcase$",
            "function error$",
        ] {
            assert!(
                output.contains(marker),
                "expected {marker} to be required in:\n{output}"
            );
        }
        assert!(!output.contains("' require com.bascal.stdlib.ltrim"));
    }

    #[test]
    fn stdlib_functions_are_absent_when_not_required() {
        let source = r#"print "hello"
end
"#;
        let output = compile_source("stdlib_unused.bcl", source).expect("should compile");
        for marker in [
            "function ltrim$",
            "function rtrim$",
            "function ucase$",
            "function lcase$",
            "function error$",
        ] {
            assert!(
                !output.contains(marker),
                "unrequired stdlib function {marker} must not appear in output:\n{output}"
            );
        }
    }

    #[test]
    fn stdlib_native_builtins_still_pass_through_unchanged() {
        // STRING$, FIX, HEX$, and OCT$ *are* real BASCOM 2.00 builtins
        // (also verified against a real IBM BASIC Compiler 2.00), so
        // BASCAL must keep passing them straight through, not reimplement
        // them.
        let source = r#"print string$(3, "*")
print fix(-3.7)
print hex$(255)
print oct$(8)
end
"#;
        let output = compile_source("native_builtins.bcl", source).expect("should compile");
        assert!(output.contains("STRING$(3, \"*\")"));
        assert!(output.contains("FIX(-3.7)"));
        assert!(output.contains("HEX$(255)"));
        assert!(output.contains("OCT$(8)"));
    }

    #[test]
    fn multidimensional_arrays() {
        let source = r#"dim grid%(3, 4)
dim cube%(2, 3, 4)
grid%(1, 2) = 99
x% = grid%(1, 2)
end
"#;
        let output = compile_source("md.bcl", source).expect("should compile");
        assert!(output.contains("DIM grid%(3, 4)"));
        assert!(output.contains("DIM cube%(2, 3, 4)"));
        assert!(output.contains("grid%(1, 2) = 99"));
        assert!(output.contains("x% = grid%(1, 2)"));
    }

    #[test]
    fn peek_poke_and_new_builtins() {
        let source = r#"' POKE writes; PEEK reads (builtin function)
poke &H0400, 42
x% = peek(&H0400)
' TAB and SPC are recognised builtins for use in PRINT
print tab(10); "hi"
print spc(5); "hello"
' FRE, LPOS, VARPTR
f% = fre(0)
p% = lpos(0)
v% = varptr(x%)
end
"#;
        let output = compile_source("hw.bcl", source).expect("should compile");
        assert!(output.contains("POKE &H0400, 42"));
        assert!(output.contains("x% = PEEK(&H0400)"));
        assert!(output.contains("PRINT TAB(10); \"hi\""));
        assert!(output.contains("PRINT SPC(5); \"hello\""));
        assert!(output.contains("f% = FRE(0)"));
        assert!(output.contains("p% = LPOS(0)"));
        assert!(output.contains("v% = VARPTR(x%)"));
    }

    #[test]
    fn kill_and_name_as_statements() {
        let source = r#"kill "old.dat"
name "old.dat" as "new.dat"
end
"#;
        let output = compile_source("files.bcl", source).expect("should compile");
        assert!(output.contains(r#"KILL "old.dat""#));
        assert!(output.contains(r#"NAME "old.dat" AS "new.dat""#));
    }

    #[test]
    fn out_width_clear_and_date_time_builtins() {
        let source = r#"out 888, 3
width 80
width #1, 132
clear
print date$; " "; time$; timer
end
"#;
        let output = compile_source("sys.bcl", source).expect("should compile");
        assert!(output.contains("OUT 888, 3"));
        assert!(output.contains("WIDTH 80"));
        assert!(output.contains("WIDTH #1, 132"));
        assert!(output.contains("CLEAR"));
        assert!(output.contains("DATE$"));
        assert!(output.contains("TIME$"));
        assert!(output.contains("TIMER"));
    }

    #[test]
    fn supports_new_binary_operators() {
        let source = r#"' exponentiation, integer division, MOD, XOR
a% = 2 ^ 8
b% = 17 \ 5
c% = 17 mod 5
d% = 6 xor 3
' precedence: 2 ^ 3 ^ 2 = 2 ^ (3 ^ 2) = 512 (right-assoc)
e% = 2 ^ 3 ^ 2
' MOD < \ in precedence: (10 \ 3) mod 2 = 3 mod 2 = 1
f% = 10 \ 3 mod 2
print a%; b%; c%; d%; e%; f%
end
"#;
        let output = compile_source("ops.bcl", source).expect("should compile");
        assert!(output.contains("a% = 2 ^ 8"));
        assert!(output.contains("b% = 17 \\ 5"));
        assert!(output.contains("c% = 17 MOD 5"));
        assert!(output.contains("d% = 6 XOR 3"));
        assert!(output.contains("e% = 2 ^ (3 ^ 2)")); // right-associative ^
        assert!(output.contains("f% = (10 \\ 3) MOD 2")); // \ binds tighter than MOD
    }

    #[test]
    fn local_names_always_use_indexed_scheme() {
        // All params, results, and locals always get an indexed suffix (0, 1, …)
        // so they can never silently collide with a bare global name.
        // Global `fooX%` must be distinct from parameter `x%` in function `foo%`.
        let source = r#"
fooX% = 99
function foo%(x%)
  global fooX%
  return x% + fooX%
end function
print foo%(1)
end
"#;
        let output = compile_source("collision.bcl", source).expect("should compile");
        // Global is normalized to lowercase, as usual for top-level names.
        assert!(
            output.contains("foox%"),
            "global fooX% must be present (lowercased)"
        );
        // Parameter x% must be lowered to an indexed name, never the bare foox%.
        assert!(
            output.contains("fooX0%"),
            "param x% must use indexed name fooX0%"
        );
        // The two names must be distinct — no line should assign foox% from foox%.
        assert!(!output.contains("foox% = foox%"), "names must not collide");
    }

    // ── generated-name conflict detection ─────────────────────────────────

    #[test]
    fn global_matching_generated_param_name_is_an_error() {
        // fooX0% is exactly what the transpiler would generate for param x% in foo%.
        // Declaring it as a global must be rejected.
        let source = r#"
fooX0% = 99
function foo%(x%)
  return x% + 1
end function
print foo%(1)
end
"#;
        let err = compile_source("conflict_param.bcl", source)
            .expect_err("should reject global that conflicts with generated param name");
        assert!(
            err.iter().any(|d| d.message.contains("fooX0%")),
            "error must name the conflicting global: {:?}",
            err
        );
    }

    #[test]
    fn global_matching_generated_result_name_is_an_error() {
        // fooResult0% is what the transpiler generates for the result variable of foo%.
        let source = r#"
fooResult0% = 0
function foo%(n%)
  return n% * 2
end function
print foo%(3)
end
"#;
        let err = compile_source("conflict_result.bcl", source)
            .expect_err("should reject global that conflicts with generated result name");
        assert!(
            err.iter().any(|d| d.message.contains("fooResult0%")),
            "error must name the conflicting global: {:?}",
            err
        );
    }

    #[test]
    fn global_matching_generated_local_name_is_an_error() {
        // fooAcc0% is what the transpiler would generate for local acc% inside foo%.
        let source = r#"
fooAcc0% = 0
function foo%(n%)
  acc% = 0
  acc% = acc% + n%
  return acc%
end function
print foo%(5)
end
"#;
        let err = compile_source("conflict_local.bcl", source)
            .expect_err("should reject global that conflicts with generated local name");
        assert!(
            err.iter().any(|d| d.message.contains("fooAcc0%")),
            "error must name the conflicting global: {:?}",
            err
        );
    }

    // ── byref / byval parameters ────────────────────────────────────────

    #[test]
    fn byval_array_parameter_does_not_copy_result_back() {
        // Unmarked (byval, the default) array parameter: copy-in only.
        let source = r#"
function zeroOut%(arr%(?))
  for i% = 0 to sizeof(arr%) - 1
    arr%(i%) = 0
  end for
  return 0
end function

dim data%(3)
dummy% = zeroOut%(data%())
end
"#;
        let output = compile_source("byval_array.bcl", source).expect("should compile");
        assert!(
            output.contains("zerooutArr0%(") && output.contains(") = data%("),
            "byval should still copy the array in:\n{output}"
        );
        assert!(
            !output
                .lines()
                .any(|l| l.contains("data%(") && l.contains(") = zerooutArr0%(")),
            "byval must not copy the array back out:\n{output}"
        );
    }

    #[test]
    fn byref_array_parameter_copies_result_back() {
        let source = r#"
function zeroOut%(byref arr%(?))
  for i% = 0 to sizeof(arr%) - 1
    arr%(i%) = 0
  end for
  return 0
end function

dim data%(3)
dummy% = zeroOut%(data%())
end
"#;
        let output = compile_source("byref_array.bcl", source).expect("should compile");
        assert!(
            output.contains("zerooutArr0%(") && output.contains(") = data%("),
            "byref should still copy the array in:\n{output}"
        );
        assert!(
            output
                .lines()
                .any(|l| l.contains("data%(") && l.contains(") = zerooutArr0%(")),
            "byref must copy the array back out:\n{output}"
        );
    }

    #[test]
    fn array_parameter_copy_loops_start_at_index_0_not_1() {
        // Regression test for GitHub issue #39: the copy-in loop (and its
        // byref copy-back counterpart) used to run `FOR ... = 1 TO bound`,
        // silently dropping index 0 of both the source and destination
        // arrays -- see `array_copy_lines` in codegen_basic.rs. A real
        // top-level `dim`'d array is always indexed `0..=bound` (see
        // `docs/manual/arrays.html`), so the copy must cover the same
        // 0-based range, matching `--target c`'s own (always correct)
        // `for (int bcc_i = 0; ...)` copy-in loop.
        let source = r#"
function zeroOut%(byref arr%(?))
  for i% = 0 to sizeof(arr%) - 1
    arr%(i%) = 0
  end for
  return 0
end function

dim data%(3)
dummy% = zeroOut%(data%())
end
"#;
        let output = compile_source("array_copy_index0.bcl", source).expect("should compile");
        assert!(
            output.lines().any(|l| l.trim_start().starts_with("FOR BCCT") && l.contains(" = 0 TO ")),
            "array copy loops should start at index 0, not 1:\n{output}"
        );
        assert!(
            !output.lines().any(|l| l.trim_start().starts_with("FOR BCCT") && l.contains(" = 1 TO ")),
            "no array copy loop should start at index 1 (drops index 0):\n{output}"
        );
    }

    #[test]
    fn byref_scalar_parameter_writes_back_to_caller() {
        let source = r#"
procedure increment(byref n%)
  n% = n% + 1
end procedure

x% = 5
increment(x%)
print x%
end
"#;
        let output = compile_source("byref_scalar.bcl", source).expect("should compile");
        assert!(
            output.lines().any(|l| l.trim() == "x% = incrementN0%"),
            "byref scalar must write its result back to the caller's variable:\n{output}"
        );
    }

    #[test]
    fn byval_scalar_parameter_does_not_write_back() {
        // Unmarked (byval) scalar parameter: today's existing, unchanged behavior.
        let source = r#"
procedure increment(n%)
  n% = n% + 1
end procedure

x% = 5
increment(x%)
print x%
end
"#;
        let output = compile_source("byval_scalar.bcl", source).expect("should compile");
        assert!(
            !output.lines().any(|l| l.trim() == "x% = incrementN0%"),
            "byval scalar must not write back to the caller's variable:\n{output}"
        );
    }

    #[test]
    fn byref_scalar_argument_must_be_a_plain_variable() {
        let source = r#"
procedure increment(byref n%)
  n% = n% + 1
end procedure

x% = 5
increment(x% + 1)
end
"#;
        let err = compile_source("byref_non_lvalue.bcl", source)
            .expect_err("byref argument that isn't a plain variable should be rejected");
        assert!(
            err.iter()
                .any(|d| d.message.contains("byref") && d.message.contains("plain variable")),
            "error must explain the byref/lvalue requirement: {:?}",
            err
        );
    }

    #[test]
    fn global_shadowed_by_same_named_parameter_is_rejected() {
        let source = r#"
function f%(arr%)
  global arr%
  return arr%
end function

print f%(0)
end
"#;
        let err = compile_source("global_shadow.bcl", source)
            .expect_err("global declaration matching a parameter name should be rejected");
        assert!(
            err.iter()
                .any(|d| d.message.contains("global arr%") && d.message.contains("shadows")),
            "error must explain the parameter shadows the global: {:?}",
            err
        );
    }

    // ── multi-dimensional array parameters ──────────────────────────────

    #[test]
    fn two_dimensional_array_parameter_generates_nested_copy_loops() {
        let source = r#"
function sumGrid%(byref grid%(?, ?))
  total% = 0
  for r% = 0 to sizeof(grid%, 0) - 1
    for c% = 0 to sizeof(grid%, 1) - 1
      total% = total% + grid%(r%, c%)
    end for
  end for
  return total%
end function

dim g%(2, 2)
print sumGrid%(g%())
end
"#;
        let output = compile_source("multidim_2d.bcl", source).expect("should compile");
        assert!(
            output.contains("sumgridGridDim00% = 2") && output.contains("sumgridGridDim10% = 2"),
            "the caller should auto-inject g%'s real DIM bounds:\n{output}"
        );
        assert!(
            output.contains("DIM sumgridGrid0%(2, 2)"),
            "parameter storage should be DIMed once, at top-level, with both axes' resolved \
             capacity:\n{output}"
        );
        assert!(
            output
                .lines()
                .any(|l| { l.trim() == "sumgridGrid0%(BCCT1%, BCCT2%) = g%(BCCT1%, BCCT2%)" }),
            "copy-in should use two indices on both sides:\n{output}"
        );
        assert!(
            output
                .lines()
                .any(|l| { l.trim() == "g%(BCCT3%, BCCT4%) = sumgridGrid0%(BCCT3%, BCCT4%)" }),
            "byref copy-out should use two indices on both sides:\n{output}"
        );
        // Regression test: a 2+-index array access (`grid%(r%, c%)`) parses
        // as Expr::Call, not Expr::ArrayRef (see make_paren_ident_expr in
        // parser.rs) -- codegen's Call-fallback branch used to render the
        // raw source name instead of resolving it through the parameter's
        // mangled storage name, so reads/writes inside the function body
        // silently touched the wrong (nonexistent) variable.
        assert!(
            output.contains("sumgridGrid0%(sumgridR0%, sumgridC0%)"),
            "reading the array parameter inside the body must use its mangled storage name, \
             not the raw source name:\n{output}"
        );
    }

    #[test]
    fn two_dimensional_local_array_reads_and_writes_use_mangled_name() {
        // Same underlying bug as the parameter case above, but for a local
        // (non-parameter) multi-dimensional array declared with `dim`
        // inside a function -- both the write and the read need to resolve
        // through the function-local mangled name, not the raw source name.
        let source = r#"
function fillGrid%(n%)
  dim grid%(2, 2)
  grid%(1, 1) = n%
  return grid%(1, 1)
end function

print fillGrid%(10)
end
"#;
        let output = compile_source("multidim_local.bcl", source).expect("should compile");
        assert!(
            output.contains("fillgridGrid0%(1, 1) = fillgridN0%"),
            "writing a local 2-D array must use its mangled storage name:\n{output}"
        );
        assert!(
            output.contains("fillgridResult0% = fillgridGrid0%(1, 1)"),
            "reading a local 2-D array must use its mangled storage name:\n{output}"
        );
    }

    #[test]
    fn three_dimensional_array_parameter_generates_triple_nested_copy_loops() {
        let source = r#"
function sumCube%(byref cube%(?, ?, ?))
  total% = 0
  for i% = 0 to sizeof(cube%, 0) - 1
    for j% = 0 to sizeof(cube%, 1) - 1
      for k% = 0 to sizeof(cube%, 2) - 1
        total% = total% + cube%(i%, j%, k%)
      end for
    end for
  end for
  return total%
end function

dim cube%(1, 1, 1)
print sumCube%(cube%())
end
"#;
        let output = compile_source("multidim_3d.bcl", source).expect("should compile");
        assert!(
            output.contains("DIM sumcubeCube0%(1, 1, 1)"),
            "parameter storage should be DIMed once, at top-level, with all three axes' \
             resolved capacity:\n{output}"
        );
        assert!(
            output.lines().any(|l| {
                l.trim() == "sumcubeCube0%(BCCT1%, BCCT2%, BCCT3%) = cube%(BCCT1%, BCCT2%, BCCT3%)"
            }),
            "copy-in should use three indices on both sides:\n{output}"
        );
    }

    #[test]
    fn array_rank_mismatch_at_call_site_is_rejected() {
        // g% is DIMed 2-D, but sumRow%'s row% parameter is only ever indexed
        // with one subscript in its own body -- passing g%() there would
        // generate a `Wrong number of subscripts` BASIC program.
        let source = r#"
function sumRow%(byref row%(?))
  total% = 0
  for i% = 0 to sizeof(row%) - 1
    total% = total% + row%(i%)
  end for
  return total%
end function

dim g%(2, 2)
print sumRow%(g%())
end
"#;
        let err = compile_source("multidim_mismatch.bcl", source)
            .expect_err("rank mismatch between the array and the parameter should be rejected");
        assert!(
            err.iter().any(|d| {
                d.message.contains("2 dimensions")
                    && d.message.contains("indexed with 1")
                    && d.message.contains("row%")
            }),
            "error must name both ranks and the mismatched parameter: {:?}",
            err
        );
    }

    #[test]
    fn array_used_in_body_without_declared_rank_is_rejected() {
        // arr% is indexed as a 1-D array in the body but the declaration
        // never says so -- there's no other way to learn a parameter's
        // rank, so this must be a compile-time error, not an inference.
        let source = r#"
function sumArr%(arr%, count%)
  total% = 0
  for i% = 0 to count% - 1
    total% = total% + arr%(i%)
  end for
  return total%
end function
end
"#;
        let err = compile_source("missing_rank.bcl", source)
            .expect_err("an array parameter with no declared rank should be rejected");
        assert!(
            err.iter().any(|d| {
                d.message.contains("arr%")
                    && d.message.contains("sumArr%")
                    && d.message.contains("doesn't say so")
                    && d.message.contains("arr%(?)")
            }),
            "error must explain the missing declaration and suggest the fix: {:?}",
            err
        );
    }

    #[test]
    fn declared_rank_mismatched_with_body_usage_is_rejected() {
        let source = r#"
function bad%(arr%(?, ?))
  return arr%(0)
end function
end
"#;
        let err = compile_source("declared_rank_mismatch.bcl", source)
            .expect_err("a declared rank that disagrees with body usage should be rejected");
        assert!(
            err.iter().any(|d| {
                d.message.contains("declared with 2 dimensions")
                    && d.message.contains("indexed with 1 subscript")
            }),
            "error must name both the declared and used rank: {:?}",
            err
        );
    }

    #[test]
    fn bare_identifier_array_argument_is_accepted_without_parens() {
        // Once a parameter's rank is declared, the call site no longer
        // needs `()` to mark an argument as an array -- the transpiler
        // already knows from the callee's signature.
        let source = r#"
function sumGrid%(byref grid%(?, ?))
  total% = 0
  for r% = 0 to sizeof(grid%, 0) - 1
    for c% = 0 to sizeof(grid%, 1) - 1
      total% = total% + grid%(r%, c%)
    end for
  end for
  return total%
end function

dim g%(2, 2)
print sumGrid%(g%)
end
"#;
        let output = compile_source("bare_ident_call.bcl", source).expect("should compile");
        assert!(
            output.contains("DIM sumgridGrid0%(2, 2)"),
            "bare identifier array argument should still generate correct copy-in/copy-out:\n{output}"
        );
        assert!(
            output
                .lines()
                .any(|l| { l.trim() == "sumgridGrid0%(BCCT1%, BCCT2%) = g%(BCCT1%, BCCT2%)" }),
            "copy-in should read from g%, the bare identifier, not require g%():\n{output}"
        );
    }

    #[test]
    fn bare_identifier_and_parens_call_syntax_generate_identical_output() {
        let source_with_parens = r#"
function sumArr%(arr%(?))
  total% = 0
  for i% = 0 to sizeof(arr%) - 1
    total% = total% + arr%(i%)
  end for
  return total%
end function

dim data%(4)
print sumArr%(data%())
end
"#;
        let source_bare = r#"
function sumArr%(arr%(?))
  total% = 0
  for i% = 0 to sizeof(arr%) - 1
    total% = total% + arr%(i%)
  end for
  return total%
end function

dim data%(4)
print sumArr%(data%)
end
"#;
        let with_parens =
            compile_source("bare_vs_parens_a.bcl", source_with_parens).expect("should compile");
        let bare = compile_source("bare_vs_parens_b.bcl", source_bare).expect("should compile");
        assert_eq!(
            with_parens, bare,
            "arr%() and bare arr% at the call site must generate identical output"
        );
    }

    #[test]
    fn inconsistent_parameter_indexing_within_one_function_is_rejected() {
        let source = r#"
function bad%(arr%)
  x% = arr%(0)
  y% = arr%(0, 1)
  return x% + y%
end function

dim g%(2, 2)
print bad%(g%())
end
"#;
        let err = compile_source("inconsistent_indexing.bcl", source).expect_err(
            "indexing the same parameter with different subscript counts should be rejected",
        );
        assert!(
            err.iter()
                .any(|d| d.message.contains("different numbers of subscripts")),
            "error must explain the inconsistent usage: {:?}",
            err
        );
    }

    // ── sizeof() ─────────────────────────────────────────────────────────

    #[test]
    fn sizeof_one_dimensional_literal_bound_resolves_without_an_axis() {
        let source = r#"
dim data%(9)
print sizeof(data%)
end
"#;
        let output = compile_source("sizeof_1d.bcl", source).expect("should compile");
        assert!(
            output.contains("PRINT 9"),
            "sizeof should resolve to the literal bound:\n{output}"
        );
        assert!(
            !output.to_ascii_lowercase().contains("sizeof"),
            "sizeof must never appear literally in generated BASIC:\n{output}"
        );
    }

    #[test]
    fn sizeof_multi_dimensional_requires_and_uses_the_axis() {
        let source = r#"
dim grid%(2, 3)
print sizeof(grid%, 0)
print sizeof(grid%, 1)
end
"#;
        let output = compile_source("sizeof_2d.bcl", source).expect("should compile");
        assert!(
            output.contains("PRINT 2"),
            "axis 0 should resolve to the first DIM bound:\n{output}"
        );
        assert!(
            output.contains("PRINT 3"),
            "axis 1 should resolve to the second DIM bound:\n{output}"
        );
    }

    #[test]
    fn sizeof_freezes_a_non_literal_bound_at_dim_time() {
        // The bound isn't a literal, so sizeof must capture its value right
        // at the DIM site -- reading the variable again later must not
        // change what sizeof already resolved to.
        let source = r#"
n% = 5
dim data%(n%)
n% = 99
print sizeof(data%)
end
"#;
        let output = compile_source("sizeof_frozen.bcl", source).expect("should compile");
        assert!(
            output
                .lines()
                .any(|l| l.trim().starts_with("BCCT") && l.contains("= n%")),
            "a non-literal bound should be captured into a temp right at DIM time:\n{output}"
        );
        assert!(
            output.contains("PRINT BCCT1%"),
            "sizeof should read back the frozen temp, not the live variable:\n{output}"
        );
        // The frozen capture must appear before the later reassignment.
        let capture_pos = output.find("BCCT1% = n%").unwrap();
        let reassign_pos = output.find("n% = 99").unwrap();
        assert!(
            capture_pos < reassign_pos,
            "must freeze before n% is reassigned:\n{output}"
        );
    }

    #[test]
    fn sizeof_on_own_array_parameter_reads_the_auto_injected_bound() {
        // No DIM to freeze from inside the function, and no manually
        // written count parameter either -- sizeof on a parameter must
        // resolve to the hidden bound variable the *caller* sets, from the
        // real argument array's own resolved size, immediately before
        // GOSUB.
        let source = r#"
function sumGrid%(byref grid%(?, ?))
  total% = 0
  for r% = 0 to sizeof(grid%, 0) - 1
    for c% = 0 to sizeof(grid%, 1) - 1
      total% = total% + grid%(r%, c%)
    end for
  end for
  return total%
end function

dim g%(2, 2)
print sumGrid%(g%)
end
"#;
        let output = compile_source("sizeof_param.bcl", source).expect("should compile");
        assert!(
            output.contains("TO sumgridGridDim00% - 1"),
            "sizeof(grid%, 0) inside the body should read the auto-injected bound variable \
             directly:\n{output}"
        );
        assert!(
            output.contains("TO sumgridGridDim10% - 1"),
            "sizeof(grid%, 1) inside the body should read the auto-injected bound variable \
             directly:\n{output}"
        );
        assert!(
            output.contains("sumgridGridDim00% = 2") && output.contains("sumgridGridDim10% = 2"),
            "the call site should auto-inject g%'s real DIM bounds, with no manually written \
             count argument:\n{output}"
        );
    }

    #[test]
    fn sizeof_on_unknown_array_is_rejected() {
        let source = "print sizeof(nope%)\nend\n";
        let err = compile_source("sizeof_unknown.bcl", source)
            .expect_err("sizeof on an unrecognized array should be rejected");
        assert!(
            err.iter()
                .any(|d| d.message.contains("nope%") && d.message.contains("isn't a known array")),
            "error must name the unresolvable array: {:?}",
            err
        );
    }

    #[test]
    fn sizeof_axis_out_of_range_is_rejected() {
        let source = "dim g%(2, 2)\nprint sizeof(g%, 2)\nend\n";
        let err = compile_source("sizeof_bad_axis.bcl", source)
            .expect_err("an axis beyond the array's rank should be rejected");
        assert!(
            err.iter().any(
                |d| d.message.contains("only has 2 dimensions") && d.message.contains("axis 2")
            ),
            "error must explain the axis is out of range: {:?}",
            err
        );
    }

    #[test]
    fn sizeof_missing_axis_on_multi_dimensional_array_is_rejected() {
        let source = "dim g%(2, 2)\nprint sizeof(g%)\nend\n";
        let err = compile_source("sizeof_missing_axis.bcl", source)
            .expect_err("sizeof on a multi-D array with no axis argument should be rejected");
        assert!(
            err.iter()
                .any(|d| d.message.contains("needs an axis argument")),
            "error must explain an axis is required: {:?}",
            err
        );
    }

    #[test]
    fn sizeof_non_literal_axis_is_rejected() {
        let source = "dim g%(2, 2)\nn% = 1\nprint sizeof(g%, n%)\nend\n";
        let err = compile_source("sizeof_non_literal_axis.bcl", source)
            .expect_err("a non-literal axis argument should be rejected");
        assert!(
            err.iter()
                .any(|d| d.message.contains("must be a literal integer")),
            "error must explain the axis must be a literal: {:?}",
            err
        );
    }

    #[test]
    fn arrays_with_literal_bounds_generate_no_freeze_overhead() {
        // A literal DIM bound is already usable as-is -- freezing it into a
        // temp would just be a needless extra line, sizeof() or auto-
        // injection or not.
        let source = "dim data%(9)\nprint data%(0)\nend\n";
        let output = compile_source("literal_bound.bcl", source).expect("should compile");
        assert!(
            !output.contains("BCCT"),
            "a literal-bounded array must not generate a frozen temp:\n{output}"
        );
        assert!(output.contains("DIM data%(9)"));
    }

    #[test]
    fn arrays_with_non_literal_bounds_are_always_frozen_even_without_sizeof() {
        // Every array's bounds are frozen at DIM time unconditionally, not
        // just when sizeof() is actually called on it -- any array might
        // get passed to a function later, and the transpiler auto-injects
        // its bounds at that call site whether or not the source ever
        // calls sizeof() explicitly.
        let source = "n% = 5\ndim data%(n%)\nprint data%(0)\nend\n";
        let output = compile_source("non_literal_bound.bcl", source).expect("should compile");
        assert!(
            output.lines().any(|l| l.trim() == "BCCT1% = n%"),
            "a non-literal bound must be frozen into a temp at DIM time, unconditionally:\n{output}"
        );
        assert!(output.contains("DIM data%(n%)"));
    }

    // ── array parameter storage capacity ────────────────────────────────

    #[test]
    fn capacity_is_inferred_as_the_max_across_every_call_site() {
        let source = r#"
function sumArr%(arr%(?))
  total% = 0
  for i% = 0 to sizeof(arr%) - 1
    total% = total% + arr%(i%)
  end for
  return total%
end function

dim small%(2)
dim big%(9)
dummy% = sumArr%(small%)
dummy% = sumArr%(big%)
end
"#;
        let output = compile_source("capacity_max.bcl", source).expect("should compile");
        assert!(
            output.contains("DIM sumarrArr0%(9)"),
            "storage should be sized to the largest array ever passed, not the first call \
             site:\n{output}"
        );
    }

    #[test]
    fn capacity_is_inferred_through_a_const_reference() {
        let source = r#"
function sumArr%(arr%(?))
  total% = 0
  for i% = 0 to sizeof(arr%) - 1
    total% = total% + arr%(i%)
  end for
  return total%
end function

const n% = 6
dim data%(n%)
dummy% = sumArr%(data%)
end
"#;
        let output = compile_source("capacity_const.bcl", source).expect("should compile");
        assert!(
            output.contains("DIM sumarrArr0%(6)"),
            "a const-bounded DIM should still resolve to a concrete literal capacity, even \
             though its own DIM statement isn't a bare literal:\n{output}"
        );
    }

    #[test]
    fn capacity_is_inferred_through_a_forwarded_array_parameter() {
        // outer%'s own arr% parameter has its capacity inferred from its
        // one call site (dim data%(7)); inner%'s capacity must then be
        // inferred *through* outer%'s already-resolved capacity, not fail
        // just because outer%'s arr% isn't a literal DIM itself.
        let source = r#"
function inner%(arr%(?))
  return sizeof(arr%)
end function

function outer%(arr%(?))
  return inner%(arr%)
end function

dim data%(7)
print outer%(data%)
end
"#;
        let output = compile_source("capacity_forward.bcl", source).expect("should compile");
        assert!(
            output.contains("DIM innerArr0%(7)") && output.contains("DIM outerArr0%(7)"),
            "capacity should propagate through a forwarded array parameter:\n{output}"
        );
    }

    #[test]
    fn explicit_capacity_is_required_when_a_call_site_size_is_dynamic() {
        let source = r#"
function sumArr%(arr%(?))
  total% = 0
  for i% = 0 to sizeof(arr%) - 1
    total% = total% + arr%(i%)
  end for
  return total%
end function

input n%
dim data%(n%)
dummy% = sumArr%(data%)
end
"#;
        let err = compile_source("capacity_dynamic.bcl", source).expect_err(
            "a call site with a non-constant array size should be rejected without \
                          an explicit capacity",
        );
        assert!(
            err.iter().any(|d| {
                d.message.contains("can't automatically size")
                    && d.message.contains("arr%")
                    && d.message.contains("compile-time constant")
            }),
            "error must explain automatic sizing failed and why: {:?}",
            err
        );
    }

    #[test]
    fn explicit_capacity_is_accepted_for_a_dynamically_sized_call_site() {
        let source = r#"
function sumArr%(arr%(100))
  total% = 0
  for i% = 0 to sizeof(arr%) - 1
    total% = total% + arr%(i%)
  end for
  return total%
end function

input n%
dim data%(n%)
dummy% = sumArr%(data%)
end
"#;
        let output = compile_source("capacity_explicit.bcl", source)
            .expect("explicit capacity should compile");
        assert!(
            output.contains("DIM sumarrArr0%(100)"),
            "an explicit literal capacity should be used as-is, once, at top-level:\n{output}"
        );
    }

    #[test]
    fn explicit_capacity_too_small_for_a_literal_call_site_is_rejected() {
        let source = r#"
function sumArr%(arr%(4))
  total% = 0
  for i% = 0 to sizeof(arr%) - 1
    total% = total% + arr%(i%)
  end for
  return total%
end function

dim data%(9)
dummy% = sumArr%(data%)
end
"#;
        let err = compile_source("capacity_too_small.bcl", source).expect_err(
            "a literal call site provably bigger than the declared capacity should \
                          be a compile error",
        );
        assert!(
            err.iter().any(|d| {
                d.message.contains("9 elements") && d.message.contains("only sized for 4")
            }),
            "error must name both the offending size and the declared capacity: {:?}",
            err
        );
    }

    #[test]
    fn array_parameter_never_called_with_inferred_capacity_is_rejected() {
        let source = r#"
function sumArr%(arr%(?))
  return sizeof(arr%)
end function
end
"#;
        let err = compile_source("capacity_never_called.bcl", source)
            .expect_err("a `?` capacity with no call site to infer from should be rejected");
        assert!(
            err.iter().any(|d| d.message.contains("never called")),
            "error must explain there's no call site to infer from: {:?}",
            err
        );
    }

    #[test]
    fn call_site_emits_a_runtime_capacity_check_before_copy_in() {
        let source = r#"
function sumArr%(arr%(?))
  total% = 0
  for i% = 0 to sizeof(arr%) - 1
    total% = total% + arr%(i%)
  end for
  return total%
end function

dim data%(9)
dummy% = sumArr%(data%)
end
"#;
        let output = compile_source("capacity_runtime_check.bcl", source).expect("should compile");
        assert!(
            output
                .lines()
                .any(|l| { l.contains("IF sumarrArrDim00% > 9 THEN PRINT") && l.contains("STOP") }),
            "every call site should runtime-check the actual size against the resolved \
             capacity, as a backstop regardless of compile-time inference:\n{output}"
        );
        let check_pos = output.find("IF sumarrArrDim00% > 9 THEN").unwrap();
        let copy_pos = output
            .find("copy array argument into transpiled function storage")
            .unwrap();
        assert!(
            check_pos < copy_pos,
            "the runtime check must run before the copy-in loop:\n{output}"
        );
    }

    #[test]
    fn array_parameter_storage_is_dimed_exactly_once_even_when_called_repeatedly() {
        // Regression test: classic BASIC has no REDIM, so DIMing the same
        // shared storage array more than once at runtime is a fatal
        // "Duplicate Definition" error. Storage must be DIMed exactly once,
        // at top-level, no matter how many times the function is called.
        let source = r#"
function printArr%(arr%(?))
  return 0
end function

dim data%(9)
dummy% = printArr%(data%)
dummy% = printArr%(data%)
dummy% = printArr%(data%)
end
"#;
        let output = compile_source("capacity_dim_once.bcl", source).expect("should compile");
        let dim_count = output
            .lines()
            .filter(|l| l.trim().starts_with("DIM printarrArr0%"))
            .count();
        assert_eq!(
            dim_count, 1,
            "storage must be DIMed exactly once regardless of call count:\n{output}"
        );
    }

    // ── recursion (direct and indirect) ─────────────────────────────────

    #[test]
    fn direct_recursion_is_rejected() {
        let source = r#"
function fact%(n%)
  if n% <= 1 then
    return 1
  end if
  return n% * fact%(n% - 1)
end function
end
"#;
        let err = compile_source("direct_recursion.bcl", source)
            .expect_err("a function calling itself should be rejected");
        assert!(
            err.iter().any(|d| {
                d.message.contains("recursion is not supported")
                    && d.message.contains("`fact%` calls itself")
            }),
            "error must name the self-recursive function: {:?}",
            err
        );
    }

    #[test]
    fn two_hop_indirect_recursion_is_rejected() {
        let source = r#"
function isEven%(n%)
  if n% = 0 then
    return 1
  end if
  return isOdd%(n% - 1)
end function

function isOdd%(n%)
  if n% = 0 then
    return 0
  end if
  return isEven%(n% - 1)
end function

print isEven%(4)
end
"#;
        let err = compile_source("two_hop_recursion.bcl", source)
            .expect_err("mutual recursion between two functions should be rejected");
        assert!(
            err.iter().any(|d| {
                d.message.contains("recursion is not supported")
                    && d.message.contains("`isEven%`")
                    && d.message.contains("`isOdd%`")
                    && d.message.contains("->")
            }),
            "error must show the call cycle: {:?}",
            err
        );
    }

    #[test]
    fn three_hop_indirect_recursion_is_rejected() {
        let source = r#"
function a%(n%)
  return b%(n%)
end function

function b%(n%)
  return c%(n%)
end function

function c%(n%)
  return a%(n%)
end function

print a%(1)
end
"#;
        let err = compile_source("three_hop_recursion.bcl", source)
            .expect_err("a three-function call cycle should be rejected");
        assert!(
            err.iter().any(|d| {
                d.message.contains("`a%`")
                    && d.message.contains("`b%`")
                    && d.message.contains("`c%`")
            }),
            "error must show the full cycle: {:?}",
            err
        );
    }

    #[test]
    fn recursion_through_a_procedure_is_rejected() {
        // A function calling a procedure that calls back into the function
        // is just as broken as two functions calling each other -- both
        // still transpile to shared global storage with no call stack.
        let source = r#"
function f%(n%)
  p(n%)
  return n%
end function

procedure p(n%)
  dummy% = f%(n%)
end procedure

print f%(1)
end
"#;
        let err = compile_source("recursion_via_procedure.bcl", source)
            .expect_err("a cycle through a procedure should be rejected");
        assert!(
            err.iter()
                .any(|d| { d.message.contains("`f%`") && d.message.contains("`p`") }),
            "error must name both the function and the procedure in the cycle: {:?}",
            err
        );
    }

    #[test]
    fn non_cyclic_shared_helper_calls_are_not_rejected() {
        // A calls both B and C; B and C both call D. A diamond-shaped call
        // graph, not a cycle -- must compile cleanly.
        let source = r#"
function d%(n%)
  return n% + 1
end function

function b%(n%)
  return d%(n%)
end function

function c%(n%)
  return d%(n%) * 2
end function

function a%(n%)
  return b%(n%) + c%(n%)
end function

print a%(1)
end
"#;
        compile_source("diamond_calls.bcl", source)
            .expect("a non-cyclic call graph, even with a shared helper, should compile");
    }

    #[test]
    fn indirect_recursion_nested_inside_conditionals_is_still_rejected() {
        // The recursive call graph is built from every call reachable
        // anywhere in the AST, not just top-level statements -- a call
        // guarded by if/while/select case/for still counts as a real edge,
        // since the transpiler can't prove at compile time whether that
        // branch runs. This is also the realistic case: virtually all
        // correct recursion is conditional (an unconditional recursive
        // call would just infinite-loop at runtime with no base case).
        let source = r#"
function f%(n%)
  for i% = 1 to 3
    select case n%
        case 1
          while n% > 0
            if n% = 2 then
              return g%(n%)
            end if
            n% = n% - 1
          end while
    end select
  end for
  return 0
end function

function g%(n%)
  return f%(n% - 1)
end function

print f%(2)
end
"#;
        let err = compile_source("nested_conditional_recursion.bcl", source).expect_err(
            "a recursive call buried inside for/select/while/if should still be rejected",
        );
        assert!(
            err.iter()
                .any(|d| { d.message.contains("`f%`") && d.message.contains("`g%`") }),
            "error must still show the cycle even though the call is deeply nested: {:?}",
            err
        );
    }

    // ── short-circuit && / || ──────────────────────────────────────────

    /// Returns the label named by the first `THEN GOTO <label>` in `output`.
    fn first_goto_target(output: &str) -> &str {
        let idx = output
            .find("THEN GOTO ")
            .expect("expected a THEN GOTO line");
        output[idx + "THEN GOTO ".len()..]
            .split_whitespace()
            .next()
            .expect("label after THEN GOTO")
    }

    /// Returns the target of the first *unconditional* `GOTO <label>` line
    /// (no `THEN` on the same line) — the "chain exhausted" jump a
    /// not-simple `&&`/`||` chain emits after its per-operand guards.
    fn first_bare_goto_target(output: &str) -> &str {
        output
            .lines()
            .map(str::trim)
            .find(|l| l.starts_with("GOTO ") && !l.contains("THEN"))
            .expect("expected a bare GOTO line")
            .trim_start_matches("GOTO ")
    }

    #[test]
    fn double_amp_chain_is_one_guard_per_operand_no_extra_label() {
        let source = r#"if a% > 0 && b% > 0 then
    print "yes"
end if
end
"#;
        let output = compile_source("sc_and.bcl", source).expect("should compile");
        let target = first_goto_target(&output);
        let guard_a = format!("IF (a% > 0) = 0 THEN GOTO {target}");
        let guard_b = format!("IF (b% > 0) = 0 THEN GOTO {target}");
        assert!(output.contains(&guard_a), "missing: {guard_a}\n{output}");
        assert!(output.contains(&guard_b), "missing: {guard_b}\n{output}");
        // Simple AND-chain (not inverted) needs no extra "continue" label.
        assert!(
            !output.contains("SC_"),
            "unexpected short-circuit label:\n{output}"
        );
    }

    #[test]
    fn double_pipe_chain_needs_one_continue_label() {
        let source = r#"if a% > 0 || b% > 0 then
    print "yes"
end if
end
"#;
        let output = compile_source("sc_or.bcl", source).expect("should compile");
        let cont = first_goto_target(&output);
        let guard_a = format!("IF (a% > 0) <> 0 THEN GOTO {cont}");
        let guard_b = format!("IF (b% > 0) <> 0 THEN GOTO {cont}");
        assert!(output.contains(&guard_a), "missing: {guard_a}\n{output}");
        assert!(output.contains(&guard_b), "missing: {guard_b}\n{output}");
        // Exactly one continue label for the whole chain, not one per operand:
        // a single bare GOTO to a *different* target (the exit label) sits
        // between the two guards and the continue point.
        let exit = first_bare_goto_target(&output);
        assert_ne!(
            cont, exit,
            "continue and exit targets must differ:\n{output}"
        );
    }

    #[test]
    fn three_operand_amp_chain_emits_three_guards() {
        let source = r#"if a% > 0 && b% > 0 && c% > 0 then
    print "yes"
end if
end
"#;
        let output = compile_source("sc_and3.bcl", source).expect("should compile");
        let target = first_goto_target(&output);
        for var in ["a%", "b%", "c%"] {
            let guard = format!("IF ({var} > 0) = 0 THEN GOTO {target}");
            assert!(output.contains(&guard), "missing: {guard}\n{output}");
        }
    }

    #[test]
    fn do_until_amp_chain_is_inverted_and_needs_continue_label() {
        // `do until a && b` exits only when BOTH are true -- the De Morgan
        // mirror of a plain `if a && b`, so (unlike the simple if-case) it
        // needs a continue label, exactly like a plain OR chain does.
        let source = r#"k% = 0
do until a% > 0 && b% > 0
    k% = k% + 1
end do
end
"#;
        let output = compile_source("sc_do_until_and.bcl", source).expect("should compile");
        let cont = first_goto_target(&output);
        let guard_a = format!("IF (a% > 0) = 0 THEN GOTO {cont}");
        let guard_b = format!("IF (b% > 0) = 0 THEN GOTO {cont}");
        assert!(output.contains(&guard_a), "missing: {guard_a}\n{output}");
        assert!(output.contains(&guard_b), "missing: {guard_b}\n{output}");
        // A bare GOTO to a *different* target (the loop's exit label) sits
        // right after the two guards.
        let exit = first_bare_goto_target(&output);
        assert_ne!(
            cont, exit,
            "continue and exit targets must differ:\n{output}"
        );
    }

    #[test]
    fn do_until_pipe_chain_is_simple_no_extra_label() {
        // `do until a || b` exits as soon as EITHER is true -- the De Morgan
        // mirror of a plain `if a || b`, so (unlike the simple if-case for
        // OR) it needs no continue label, matching a plain AND-chain's shape.
        let source = r#"k% = 0
do until a% > 0 || b% > 0
    k% = k% + 1
end do
end
"#;
        let output = compile_source("sc_do_until_or.bcl", source).expect("should compile");
        assert!(
            !output.contains("SC_"),
            "unexpected continue label for simple inverted OR-chain:\n{output}"
        );
        let target = first_goto_target(&output);
        let guard_a = format!("IF (a% > 0) <> 0 THEN GOTO {target}");
        let guard_b = format!("IF (b% > 0) <> 0 THEN GOTO {target}");
        assert!(output.contains(&guard_a), "missing: {guard_a}\n{output}");
        assert!(output.contains(&guard_b), "missing: {guard_b}\n{output}");
    }

    #[test]
    fn do_while_pre_condition_loop_actually_loops() {
        // Regression test: a do while/do until loop with only a
        // pre-condition (no post-condition) previously never emitted a
        // GOTO back to re-check the condition, so it ran its body at most
        // once regardless of the condition -- discovered while verifying
        // the short-circuit `&&`/`||` condition lowering by hand.
        let source = r#"k% = 1
do while k% <= 3
    print k%
    k% = k% + 1
end do
end
"#;
        let output = compile_source("do_loops_back.bcl", source).expect("should compile");
        // The line number prefixing the condition check itself (`IF (k% <= 3) ...`)
        // is the loop's top -- confirm some later line jumps back to it.
        let top = output
            .lines()
            .find(|l| l.contains("IF (k% <= 3)"))
            .and_then(|l| l.split_whitespace().next())
            .expect("numbered condition-check line");
        let loop_back = format!("GOTO {top}");
        assert_eq!(
            output.matches(&loop_back).count(),
            1,
            "expected exactly one `{loop_back}` after the loop body, jumping back to re-check the condition:\n{output}"
        );
    }

    #[test]
    fn non_chain_condition_is_unchanged_by_short_circuit_support() {
        // Plain bitwise `and`/comparisons must render exactly as before --
        // condition_jump's fallback path for non-chain conditions.
        let source = r#"if a% > 0 and b% > 0 then
    print "yes"
end if
end
"#;
        let output = compile_source("sc_plain_and.bcl", source).expect("should compile");
        assert!(
            output.contains("IF ((a% > 0) AND (b% > 0)) = 0 THEN GOTO"),
            "unexpected rendering:\n{output}"
        );
        assert!(!output.contains("SC_"));
    }

    #[test]
    fn chain_operand_side_effect_only_evaluated_after_earlier_guard() {
        // A later operand's prelude (the GOSUB for a function call) must be
        // emitted after the earlier operand's own guard line, not hoisted
        // to the top -- proof that a false first operand really does skip
        // calling the second.
        let source = r#"function check%(n%)
    return n%
end function

if a% > 0 && check%(b%) > 0 then
    print "yes"
end if
end
"#;
        let output = compile_source("sc_side_effect.bcl", source).expect("should compile");
        let first_guard = output
            .find("IF (a% > 0) = 0 THEN GOTO")
            .expect("first guard");
        // Search for the actual GOSUB *statement* (line-initial), not the
        // word "GOSUB" that also appears in the generated header comment.
        let gosub = output
            .find("\nGOSUB ")
            .expect("GOSUB statement for check%(b%) call");
        assert!(
            gosub > first_guard,
            "check%(b%)'s GOSUB must come after a%'s guard line, not before:\n{output}"
        );
    }

    // ── DEF FN (deliberately unsupported) ───────────────────────────────

    fn assert_def_fn_rejected(filename: &str, source: &str, def_line: usize) {
        let err = compile_source(filename, source).expect_err("DEF FN should be rejected");
        assert_eq!(
            err.len(),
            1,
            "should report exactly one diagnostic: {:?}",
            err
        );
        let d = &err[0];
        assert!(
            d.message.contains("DEF FN is not supported by BASCAL"),
            "must be the specific DEF FN rejection, not a generic parse error: {:?}",
            err
        );
        assert!(
            d.message.contains("Rewrite this by hand as a `function`"),
            "must explain how to port it: {:?}",
            err
        );
        assert_eq!(
            d.pos.line, def_line,
            "diagnostic must point at the DEF FN statement itself, not a token inside its \
             expression: {:?}",
            err
        );
    }

    #[test]
    fn def_fn_clean_form_is_rejected() {
        let source = r#"program p
def fn A(X) = X * X + 1
print "after"
end
"#;
        assert_def_fn_rejected("deffn_clean.bcl", source, 2);
    }

    #[test]
    fn def_fn_comma_operator_form_is_rejected() {
        let source = r#"program p
def fn X(A) = (A = A + 1, A)
print "after"
end
"#;
        assert_def_fn_rejected("deffn_comma.bcl", source, 2);
    }

    #[test]
    fn def_fn_colon_chained_form_is_rejected() {
        let source = r#"program p
def fn Z(A) = A = A + 1 : A
print "after"
end
"#;
        assert_def_fn_rejected("deffn_colon.bcl", source, 2);
    }

    #[test]
    fn def_fn_no_params_form_is_rejected() {
        let source = r#"program p
def fn W = X + Y
print "after"
end
"#;
        assert_def_fn_rejected("deffn_noparams.bcl", source, 2);
    }

    #[test]
    fn def_fn_amid_other_statements_reports_only_itself_and_keeps_line_tracking() {
        // A DEF FN statement sandwiched between valid statements must
        // produce only the one DEF FN diagnostic (the parser bails out
        // on the first error, same as any other fatal parse error) --
        // and the token-skipping used to recognize DEF FN as a unit must
        // not desync line counting for anything that follows.
        let source = r#"program p
print "before"
def fn A(X) = X * X + 1
print "after"
end
"#;
        assert_def_fn_rejected("deffn_amid_others.bcl", source, 3);
    }

    #[test]
    fn def_used_as_a_plain_identifier_is_not_mistaken_for_def_fn() {
        // `def` (without a following `fn`) isn't a keyword BASCAL uses
        // for anything else -- confirmed by grepping the parser for other
        // `"def"` dispatches (none) -- so a variable literally named
        // `def%` must compile as an ordinary identifier, not trigger the
        // DEF FN rejection.
        let source = r#"program p
def% = 5
print def%
end
"#;
        let output = compile_source("def_as_ident.bcl", source).expect("should compile");
        assert!(
            output.contains("def% = 5"),
            "`def%` should compile as a plain variable:\n{output}"
        );
    }

    #[test]
    fn error_handler_procedure_that_always_resumes_compiles_with_no_trailing_return() {
        // A procedure named as an `on error goto` target is entered via a
        // raw GOTO, never a GOSUB -- so codegen's usual implicit trailing
        // RETURN (for a body that doesn't already end in `return`) would
        // have no call frame to pop if it were ever reached. Once
        // resolver::validate proves every path here ends in `resume`,
        // codegen must skip that RETURN entirely, same as a raw label.
        let source = r#"
on error goto errHandler
x% = 1 / 0
print "after"
end

procedure errHandler()
    print "caught err "; err
    resume next
end procedure
"#;
        let output = compile_source("error_handler_resumes.bcl", source).expect("should compile");
        let handler = output
            .lines()
            .skip_while(|l| !l.contains("procedure errhandler"))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            !handler.contains("RETURN"),
            "a proven-diverging error-handler procedure must not get an implicit RETURN:\n{output}"
        );
    }

    #[test]
    fn error_handler_procedure_containing_return_is_rejected() {
        let source = r#"
on error goto errHandler
x% = 1 / 0
end

procedure errHandler()
    if err = 11 then
        return
    end if
    resume next
end procedure
"#;
        let err = compile_source("error_handler_with_return.bcl", source)
            .expect_err("a `return` inside an on-error-goto-target procedure should be rejected");
        assert!(
            err.iter().any(|d| {
                d.message.contains("`errHandler` cannot contain `return`")
                    && d.message.contains("on error goto")
            }),
            "error must explain the RETURN-without-GOSUB risk: {:?}",
            err
        );
    }

    #[test]
    fn error_handler_procedure_that_can_fall_through_is_rejected() {
        let source = r#"
on error goto errHandler
x% = 1 / 0
end

procedure errHandler()
    if err = 11 then
        resume next
    end if
    print "unhandled"
end procedure
"#;
        let err = compile_source("error_handler_fallthrough.bcl", source)
            .expect_err("a procedure that can fall off the end should be rejected");
        assert!(
            err.iter().any(|d| {
                d.message.contains("`errHandler` doesn't end every path")
                    && d.message.contains("implicit RETURN")
            }),
            "error must explain the fallthrough risk: {:?}",
            err
        );
    }

    #[test]
    fn error_handler_procedure_also_called_normally_is_rejected() {
        let source = r#"
on error goto errHandler
errHandler()
end

procedure errHandler()
    resume next
end procedure
"#;
        let err = compile_source("error_handler_dual_use.bcl", source).expect_err(
            "a procedure that's both an error-goto target and normally called should be rejected",
        );
        assert!(
            err.iter().any(|d| {
                d.message
                    .contains("`errHandler` is both an `on error goto` target")
                    && d.message.contains("called like an ordinary procedure")
            }),
            "error must explain the dual-use conflict: {:?}",
            err
        );
    }

    #[test]
    fn error_handler_procedure_diverging_via_select_case_compiles_clean() {
        // Exercises the SelectCase arm of `diverges`: every case, including
        // a mandatory `case else`, must itself diverge.
        let source = r#"
on error goto errHandler
x% = 1 / 0
end

procedure errHandler()
    select case err
        case 11
            resume next
        case else
            resume next
    end select
end procedure
"#;
        let output =
            compile_source("error_handler_select_case.bcl", source).expect("should compile");
        let handler = output
            .lines()
            .skip_while(|l| !l.contains("procedure errhandler"))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            !handler.contains("RETURN"),
            "every select case arm diverges, so no implicit RETURN should be appended:\n{output}"
        );
    }

    #[test]
    fn error_handler_procedure_select_case_missing_case_else_is_rejected() {
        // Without `case else`, the select case can't be proven to diverge
        // on every input -- ERR could be a value none of the cases cover.
        let source = r#"
on error goto errHandler
x% = 1 / 0
end

procedure errHandler()
    select case err
        case 11
            resume next
    end select
end procedure
"#;
        let err = compile_source("error_handler_select_case_no_else.bcl", source)
            .expect_err("a select case with no case else can't be proven to diverge");
        assert!(
            err.iter()
                .any(|d| d.message.contains("`errHandler` doesn't end every path")),
            "error must flag the unproven fallthrough: {:?}",
            err
        );
    }

    #[test]
    fn on_error_goto_targeting_a_procedure_resolves_to_its_real_label() {
        // Regression test: a procedure's GOSUB entry point is emitted under
        // a synthesized `FN_<stem>` label (see FunctionInfo::from_def), not
        // the author's literal spelling -- an ordinary call site already
        // knows to emit that directly, but `on error goto`/`goto`/`gosub`/
        // `resume <label>` used to render the raw identifier text as-is
        // (label_target_text), which only ever matched a genuine `name:`
        // label statement. Targeting a procedure silently left the
        // identifier text unresolved in the output -- syntactically valid
        // BASCAL, but real BASIC would reject it as an undefined label.
        let source = r#"
on error goto errHandler
x% = 1 / 0
end

procedure errHandler()
    resume next
end procedure
"#;
        let output =
            compile_source("error_handler_label_resolution.bcl", source).expect("should compile");
        assert!(
            !output.contains("GOTO errHandler") && !output.contains("GOTO errhandler"),
            "on error goto's target must be resolved to a real line number, not left as \
             unresolved text:\n{output}"
        );
        assert!(
            output
                .lines()
                .any(|l| l.trim_start().starts_with("ON ERROR GOTO")
                    && l.trim_end()
                        .rsplit(' ')
                        .next()
                        .is_some_and(|w| w.chars().all(|c| c.is_ascii_digit()))),
            "expected `ON ERROR GOTO <number>`:\n{output}"
        );
    }

    #[test]
    fn plain_label_error_handler_is_unaffected() {
        // The raw-label form -- what inventory.bcl's own errorTrap used
        // before it became a procedure -- predates this check and must
        // keep compiling exactly as before: these new rules only ever
        // apply to a `procedure` target.
        let source = r#"
on error goto errorTrap
x% = 1 / 0
print "after"
end

errorTrap:
locate 25, 1
print "error " + str$(err)
resume next
"#;
        let output = compile_source("label_error_handler.bcl", source).expect("should compile");
        assert!(
            output.contains("RESUME NEXT"),
            "unexpected output:\n{output}"
        );
    }

    // ── C backend (Target::C) ──────────────────────────────────────────

    #[test]
    fn c_target_compiles_hello_world_tutorial() {
        // tutorial/01_hello.bcl uses only `print` of string literals and
        // `end` -- the minimal C backend's entire current surface -- so it
        // must compile cleanly under Target::C, unlike most tutorials.
        let input = Path::new(env!("CARGO_MANIFEST_DIR")).join("tutorial/01_hello.bcl");
        let options = CompileOptions {
            target: Target::C,
            ..CompileOptions::new()
        };
        let output = compile_file(&input, &options).expect("hello world should compile to C");
        assert!(output.contains("#include <stdio.h>"));
        assert!(output.contains("int main(void) {"));
        assert!(output.contains(r#"printf("Hello, World!\n");"#));
        assert!(output.contains(r#"printf("Welcome to BASCAL.\n");"#));
        assert_eq!(
            output.matches("return 0;").count(),
            1,
            "explicit `end` must not also get an implicit fallthrough return:\n{output}"
        );
    }

    #[test]
    fn c_target_hello_world_needs_no_runtime_file_at_all() {
        // Hello world's only surface (`print` of string literals, `end`)
        // needs none of the `bcc_*` runtime helpers -- so unlike a program
        // that does (see `c_target_splits_runtime_helpers_into_a_separate_file`
        // below), it should get no sibling runtime file, and no
        // `#include` line for one, at all (see GitHub issue #28 and
        // `codegen_c::GeneratedC`'s own doc comment).
        let input = Path::new(env!("CARGO_MANIFEST_DIR")).join("tutorial/01_hello.bcl");
        let options = CompileOptions {
            target: Target::C,
            ..CompileOptions::new()
        };
        let (app, runtime) = compile_file_with_runtime(&input, &options)
            .expect("hello world should compile to C");
        assert!(
            runtime.is_none(),
            "hello world needs no bcc_* helper, so it should get no runtime file"
        );
        assert!(
            !app.contains("bcc_runtime.h"),
            "no runtime file means no #include for it either:\n{app}"
        );
    }

    #[test]
    fn c_target_splits_runtime_helpers_into_a_separate_file() {
        // A program that needs a `bcc_*` runtime helper (RND, here) should
        // get that helper's *definition* only in the paired runtime file,
        // never inline in its own app file -- the whole point of GitHub
        // issue #28: reading a compiled program's own `.c` output should
        // show only that program's own logic. The app file still calls
        // into the helper by name (`bcc_rnd(...)`), and still `#include`s
        // the runtime file to see its declaration.
        let source = "print rnd(1)\nend\n";
        let (app, runtime) = compile_source_via_c_target_split(source);
        let runtime = runtime.expect("RND should need a runtime file");

        assert!(
            app.contains("#include \"bcc_runtime.h\""),
            "app file should include the runtime file:\n{app}"
        );
        assert!(
            app.contains("bcc_rnd("),
            "app file should still call the helper by name:\n{app}"
        );
        assert!(
            !app.contains("static double bcc_rnd_last"),
            "the helper's own definition must not leak into the app file:\n{app}"
        );
        assert!(
            runtime.contains("static double bcc_rnd_last"),
            "the helper's own definition should live in the runtime file:\n{runtime}"
        );
        assert!(
            runtime.contains("#ifndef BCC_RUNTIME_H"),
            "runtime file should be a safe-to-include header:\n{runtime}"
        );
    }

    #[test]
    fn c_target_compiles_arithmetic_and_conditions_tutorials() {
        // The first real, complete tutorials (not just custom test
        // snippets) the C backend can compile beyond 01_hello -- string
        // variables were the last piece both needed. Compiling is checked
        // here (in-process, fast); actual gcc-and-run output was verified
        // manually against each tutorial's own documented `// expect ...`
        // comments and matched exactly.
        let options = CompileOptions {
            target: Target::C,
            ..CompileOptions::new()
        };
        for tutorial in [
            "tutorial/03_arithmetic.bcl",
            "tutorial/04_conditions.bcl",
            "tutorial/05_loops.bcl",
            "tutorial/06_select_case.bcl",
        ] {
            let input = Path::new(env!("CARGO_MANIFEST_DIR")).join(tutorial);
            compile_file(&input, &options)
                .unwrap_or_else(|d| panic!("{tutorial} should compile to C: {d:?}"));
        }
    }

    #[test]
    fn c_target_rejects_unsupported_statements_with_a_diagnostic() {
        // PRINT USING isn't part of the minimal C backend's supported
        // surface yet -- this must fail with a clear diagnostic, not
        // panic or silently emit wrong C.
        let source = "print using \"###\"; 5\nend\n";
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("unsupported.bcl");
        std::fs::write(&path, format!("program p\n{source}")).unwrap();

        let options = CompileOptions {
            target: Target::C,
            ..CompileOptions::new()
        };
        let result = compile_file(&path, &options);
        assert!(result.is_err());
        let msg = result
            .unwrap_err()
            .into_iter()
            .map(|d| d.to_string())
            .collect::<String>();
        assert!(
            msg.contains("not supported by the minimal C backend yet"),
            "unexpected message: {msg}"
        );
    }

    #[test]
    fn c_target_print_supports_numeric_literals_mixed_with_strings() {
        let source = r#"print "Score: "; 100; " / "; 3.5; " (100%)"
print 42
print -1.25
end
"#;
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains(r#"printf("Score: %d / %g (100%%)\n", 100, 3.5);"#),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains(r#"printf("%d\n", 42);"#),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains(r#"printf("%g\n", -(1.25));"#),
            "unexpected output:\n{output}"
        );
        assert!(
            !output.contains("math.h"),
            "a program with no `\\` shouldn't pull in <math.h>:\n{output}"
        );
    }

    #[test]
    fn c_target_print_supports_len_builtin() {
        // LEN is one of the small set of BASIC intrinsics this backend
        // implements natively (see `render_numeric_call` in codegen_c.rs)
        // -- superseded `c_target_print_still_rejects_string_function_calls`,
        // which predates that support and asserted the opposite.
        let source = "name$ = \"Alice\"\nprint len(name$)\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("#include <string.h>"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("((int)strlen(bv_s_name))"),
            "unexpected output:\n{output}"
        );
    }

    #[test]
    fn c_target_supports_string_variables_const_and_assignment() {
        let source = r#"const appName$ = "Grade Checker"
playerName$ = "Alice"
print appName$
print "Player: "; playerName$
end
"#;
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("char bv_s_appname[256] = {0};"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains(
                r#"snprintf(bv_s_appname, sizeof(bv_s_appname), "%s", "Grade Checker");"#
            ),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains(r#"printf("%s\n", bv_s_appname);"#),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains(r#"printf("Player: %s\n", bv_s_playername);"#),
            "unexpected output:\n{output}"
        );
    }

    #[test]
    fn c_target_string_concatenation_uses_snprintf_never_strcpy_or_strcat() {
        // Every string buffer is fixed-size -- snprintf is used for every
        // write specifically so a string that doesn't fit is *safely
        // truncated*, never a buffer overflow. strcpy/strcat (unbounded,
        // a real overflow risk against a fixed buffer) must never appear.
        let source = r#"grade$ = "A"
print "Grade: " + grade$
print "Hello" + ", " + "World" + "!"
end
"#;
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains(r#"snprintf(bt_s_0, sizeof(bt_s_0), "%s%s", "Grade: ", bv_s_grade);"#),
            "unexpected output:\n{output}"
        );
        // Left-associative chain -- each + gets its own temp buffer.
        assert!(
            output.contains(r#"snprintf(bt_s_1, sizeof(bt_s_1), "%s%s", "Hello", ", ");"#)
                && output.contains(r#"snprintf(bt_s_2, sizeof(bt_s_2), "%s%s", bt_s_1, "World");"#)
                && output.contains(r#"snprintf(bt_s_3, sizeof(bt_s_3), "%s%s", bt_s_2, "!");"#),
            "unexpected output:\n{output}"
        );
        assert!(
            !output.contains("strcpy") && !output.contains("strcat"),
            "unexpected output:\n{output}"
        );
    }

    #[test]
    fn c_target_string_literal_percent_is_not_doubled_as_a_plain_value() {
        // A % in a string used as a printf format-string argument (this
        // module's escape_c_string_literal) must NOT be doubled to %% --
        // only text embedded directly into printf's own format string
        // (escape_c_format_text) needs that. Getting this backwards would
        // either corrupt a value string ("100%" -> "100%%") or, worse, let
        // an unescaped % inside a literal print reach printf's format
        // parser.
        let source = "grade$ = \"100%\"\nprint grade$\nprint \"100%\"\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains(r#"snprintf(bv_s_grade, sizeof(bv_s_grade), "%s", "100%");"#),
            "a literal used as a plain value must keep a single %:\n{output}"
        );
        assert!(
            output.contains(r#"printf("100%%\n");"#),
            "a literal embedded directly in printf's format string must double %:\n{output}"
        );
    }

    #[test]
    fn c_target_supports_scalar_variables_dim_assignment_and_read() {
        // dim + assignment + read, spring-into-existence zero-init (z% is
        // read before ever being assigned), and int/float mixed arithmetic
        // between two different variables.
        let source = r#"dim total%
x% = 5
y% = 10
total% = x% + y%
print "Total: "; total%

price! = 19.99
qty% = 3
print "Cost: "; price! * qty%

z% = z% + 1
print "Z: "; z%
end
"#;
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("int bv_i_total = 0;"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("int bv_i_x = 0;"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("float bv_f_price = 0;"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("bv_i_x = 5;"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains(r#"printf("Total: %d\n", bv_i_total);"#),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains(r#"printf("Cost: %g\n", (bv_f_price * bv_i_qty));"#),
            "unexpected output:\n{output}"
        );
        // z% is read (in `z% + 1`) before ever being assigned -- it must
        // still be declared/zero-initialized, not left as a use of an
        // undeclared C variable.
        assert!(
            output.contains("int bv_i_z = 0;"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("bv_i_z = (bv_i_z + 1);"),
            "unexpected output:\n{output}"
        );
    }

    #[test]
    fn c_target_scalar_variable_declarations_are_sorted_and_deduplicated() {
        // Declarations come from a BTreeMap (sorted by C name, so codegen
        // output is deterministic across runs) and are collected once per
        // variable no matter how many times it's referenced.
        let source = "x% = 1\nx% = x% + 1\nx% = x% + 1\ny% = 2\nprint x%; y%\nend\n";
        let output = compile_source_via_c_target(source);
        assert_eq!(
            output.matches("int bv_i_x = 0;").count(),
            1,
            "x% must be declared exactly once:\n{output}"
        );
        let x_pos = output.find("int bv_i_x = 0;").unwrap();
        let y_pos = output.find("int bv_i_y = 0;").unwrap();
        assert!(
            x_pos < y_pos,
            "declarations should be sorted (bv_i_x before bv_i_y):\n{output}"
        );
    }

    #[test]
    fn c_target_supports_const_same_as_assignment() {
        // Real MBASIC/BASCOM has no CONST statement at all -- `const`
        // codegens exactly like an ordinary assignment, same as the BASIC
        // backend's own treatment of it (BASCAL's resolver, not codegen,
        // is what enforces a const is never reassigned).
        let source = r#"const maxScore% = 100
const rate! = 0.15
score% = 85
print "Score: "; score%; " / "; maxScore%
print "Bonus: "; score% * rate!
end
"#;
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("int bv_i_maxscore = 0;"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("float bv_f_rate = 0;"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("bv_i_maxscore = 100;"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("bv_f_rate = 0.15;"),
            "unexpected output:\n{output}"
        );
        assert!(
            !output.contains("CONST"),
            "real MBASIC/BASCOM has no CONST:\n{output}"
        );
    }

    #[test]
    fn c_target_comparisons_produce_basic_minus_one_zero() {
        // Real MBASIC/BASCOM comparisons evaluate to -1 (true) or 0
        // (false), per the manual's own Comparison Operators section -- not
        // C's 1/0. -(a == b) gets there directly from C's native 0/1
        // comparison result.
        let source = r#"score% = 85
print "A: "; 5 = 5
print "B: "; 5 = 6
print "C: "; score% >= 60
end
"#;
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains(r#"printf("A: %d\n", (-(5 == 5)));"#),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains(r#"printf("B: %d\n", (-(5 == 6)));"#),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains(r#"printf("C: %d\n", (-(bv_i_score >= 60)));"#),
            "unexpected output:\n{output}"
        );
    }

    #[test]
    fn c_target_supports_if_elseif_else_and_nested_if() {
        // Unlike the BASIC backend, which transpiles `if`/`elseif`/`else`
        // into a GOTO/label chain (real MBASIC/BASCOM has no block IF), C
        // has native if/else, so this is a direct structural translation.
        // `elseif` needs no special handling: the parser already desugars
        // it into a single nested Statement::If inside else_body.
        let source = r#"score% = 72
if score% >= 60 then
    print "Pass"
else
    print "Fail"
end if

points% = 85
if points% >= 90 then
    grade% = 4
elseif points% >= 80 then
    grade% = 3
else
    grade% = 1
end if
print grade%

x% = 15
if x% > 0 then
    if x% > 10 then
        print "large"
    else
        print "small"
    end if
end if
end
"#;
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("if ((-(bv_i_score >= 60))) {"),
            "unexpected output:\n{output}"
        );
        assert!(output.contains("} else {"), "unexpected output:\n{output}");
        // elseif desugars to a nested if inside the else branch.
        assert!(
            output.contains("if ((-(bv_i_points >= 90))) {")
                && output.contains("if ((-(bv_i_points >= 80))) {"),
            "unexpected output:\n{output}"
        );
        // Nested if inside a then-branch.
        assert!(
            output.contains("if ((-(bv_i_x > 10))) {"),
            "unexpected output:\n{output}"
        );
    }

    #[test]
    fn c_target_if_with_no_else_omits_the_else_branch() {
        let source = "x% = 5\nif x% > 0 then\n    print \"positive\"\nend if\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("if ((-(bv_i_x > 0))) {"),
            "unexpected output:\n{output}"
        );
        assert!(
            !output.contains("} else {"),
            "no else branch should mean no else clause:\n{output}"
        );
    }

    #[test]
    fn c_target_supports_and_or_xor_not_matching_the_manual_examples() {
        // Real MBASIC/BASCOM's AND/OR/XOR/NOT are genuinely bitwise, not
        // short-circuit booleans -- verified against the GW-BASIC
        // Reference Manual's own worked examples (63 AND 16 = 16,
        // 6 XOR 3 = 5) and the manual's own NOT 1 = -2 example (the point
        // being it's NOT 0, which a naive C `!` translation would give).
        let source = r#"print "A: "; 63 and 16
print "B: "; 6 xor 3
print "C: "; not 1
end
"#;
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains(
                r#"printf("A: %d\n", ((int)((long)round((double)63) & (long)round((double)16))));"#
            ),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains(
                r#"printf("B: %d\n", ((int)((long)round((double)6) ^ (long)round((double)3))));"#
            ),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains(r#"printf("C: %d\n", ((int)(~(long)round((double)1))));"#),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("#include <math.h>"),
            "unexpected output:\n{output}"
        );
    }

    #[test]
    fn c_target_supports_and_in_if_condition() {
        // The "next simplest program" a bitwise AND actually gets used
        // for in practice: a compound if condition, same as
        // tutorial/03_arithmetic.bcl and tutorial/04_conditions.bcl (both
        // still blocked on string variables, but no longer on AND).
        let source = "age% = 25\nincome% = 45000\nif age% >= 18 and income% >= 30000 then\n    print \"Eligible\"\nelse\n    print \"Not eligible\"\nend if\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("if (((int)((long)round((double)(-(bv_i_age >= 18))) & (long)round((double)(-(bv_i_income >= 30000))))))"),
            "unexpected output:\n{output}"
        );
    }

    #[test]
    fn c_target_print_supports_add_sub_mul_of_literals() {
        let source = r#"print "Sum: "; 1 + 2 * 3
print "Mixed: "; 1 + 2.5
print "Neg: "; -(3 + 4)
end
"#;
        let output = compile_source_via_c_target(source);
        // (2 * 3) binds tighter than the outer +, same associativity BASCAL's
        // parser already resolved -- every Binary node is parenthesized, so
        // the C compiler doesn't need to re-derive precedence itself.
        assert!(
            output.contains(r#"printf("Sum: %d\n", (1 + (2 * 3)));"#),
            "unexpected output:\n{output}"
        );
        // int + float promotes the whole expression to %g, matching BASIC's
        // own mixed-type promotion rule.
        assert!(
            output.contains(r#"printf("Mixed: %g\n", (1 + 2.5));"#),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains(r#"printf("Neg: %d\n", -((3 + 4)));"#),
            "unexpected output:\n{output}"
        );
    }

    #[test]
    fn c_target_print_supports_division_as_true_division() {
        // `/` gets explicit `(double)` casts on both operands, so `10 / 3`
        // stays true (floating-point) division in the generated C too, the
        // same as BASIC -- not truncated to `3` the way plain C `int / int`
        // would be.
        let source = r#"print "Int/Int: "; 10 / 3
print "Exact: "; 10 / 2
end
"#;
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains(r#"printf("Int/Int: %g\n", ((double)10 / (double)3));"#),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains(r#"printf("Exact: %g\n", ((double)10 / (double)2));"#),
            "unexpected output:\n{output}"
        );
    }

    #[test]
    fn c_target_print_supports_intdiv_as_round_then_truncate() {
        // `\` rounds each operand to the nearest integer first (verified
        // against the GW-BASIC Reference Manual), then truncates the
        // integer quotient toward zero -- not plain C `/` truncation
        // between the original operands, and not a floor.
        let source = r#"print "Exact: "; 17 \ 5
print "Rounds: "; 7.5 \ 2
print "Neg: "; -17 \ 5
end
"#;
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains(
                r#"printf("Exact: %d\n", ((int)((long)round((double)17) / (long)round((double)5))));"#
            ),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains(
                r#"printf("Rounds: %d\n", ((int)((long)round((double)7.5) / (long)round((double)2))));"#
            ),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("#include <math.h>"),
            "unexpected output:\n{output}"
        );
    }

    #[test]
    fn c_target_print_supports_mod_matching_the_manual_examples() {
        // Same round-then-truncate rounding \ uses, then C's native % on
        // the rounded operands -- GW-BASIC's own MOD examples say the
        // remainder comes from the *same* integer division \ performs, and
        // C's % is defined the same way since C99 (sign follows the
        // dividend), so no separate sign logic is needed.
        let source = r#"print "A: "; 10.4 mod 4
print "B: "; 25.68 mod 6.99
end
"#;
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains(
                r#"printf("A: %d\n", ((int)((long)round((double)10.4) % (long)round((double)4))));"#
            ),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains(
                r#"printf("B: %d\n", ((int)((long)round((double)25.68) % (long)round((double)6.99))));"#
            ),
            "unexpected output:\n{output}"
        );
    }

    #[test]
    fn c_target_print_supports_pow_matching_the_manual_examples() {
        // Right-associativity (2 ^ 3 ^ 2 = 2 ^ (3^2) = 512, not (2^3)^2 =
        // 64) is already reflected in the AST's tree shape by the time
        // codegen sees it -- BASCAL's parser resolves that, same as +/-/*'s
        // precedence -- so a nested pow() call is all that's needed here.
        let source = r#"print "A: "; 2 ^ 8
print "B: "; 2 ^ 3 ^ 2
end
"#;
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains(r#"printf("A: %g\n", pow((double)2, (double)8));"#),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains(
                r#"printf("B: %g\n", pow((double)2, (double)pow((double)3, (double)2)));"#
            ),
            "unexpected output:\n{output}"
        );
    }

    #[test]
    fn c_target_supports_for_while_do_and_exit() {
        // C has native for/while/do-while and `break`, so all four map
        // directly -- exit becomes plain `break;` and relies on C's own
        // "innermost enclosing loop" rule, no manual tracking needed
        // (unlike the BASIC backend's loop_exit_stack, needed because real
        // MBASIC/BASCOM's loops are GOTO chains with no native break).
        let source = r#"for i% = 1 to 5
    print i%
end for

j% = 0
while j% < 3
    print j%
    j% = j% + 1
wend

k% = 0
do while k% < 3
    print k%
    k% = k% + 1
end do

do
    if k% = 5 then
        exit
    end if
    k% = k% + 1
end do
end
"#;
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains(
                "for (bv_i_i = 1; bt_step_0 >= 0 ? bv_i_i <= bt_lim_0 : bv_i_i >= bt_lim_0; \
                 bv_i_i += bt_step_0) {"
            ),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("while ((-(bv_i_j < 3))) {"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("while (1) {\n        if (!((-(bv_i_k < 3)))) break;"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("    break;\n"),
            "unexpected output:\n{output}"
        );
    }

    #[test]
    fn c_target_for_loop_evaluates_bounds_once_not_per_iteration() {
        // BASIC's FOR captures start/end/step at loop entry -- if the body
        // mutates the variable the bound expression reads, the bound must
        // NOT change mid-loop the way a naive re-evaluated C condition
        // would. limit%/step_var are captured into their own temps once,
        // used in every iteration's condition instead of re-reading limit%.
        let source =
            "limit% = 3\nfor i% = 1 to limit%\n    print i%\n    limit% = 100\nend for\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("int bt_lim_0 = bv_i_limit;"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("bv_i_i <= bt_lim_0"),
            "loop condition must use the captured temp, not bv_i_limit directly:\n{output}"
        );
    }

    #[test]
    fn c_target_assignment_rounds_narrowing_conversions_like_real_bascom() {
        // Confirmed directly against real IBM Personal Computer BASIC
        // Compiler 2.00 under dosbox-x: `N% = 27 / 2` gives `N% = 14`
        // (27/2 = 13.5, rounded), not 13 -- C's own implicit double-to-int
        // assignment conversion truncates toward zero instead, which would
        // silently produce a different, wrong value. Surfaced by a
        // Collatz-sequence loop (`n% = n% / 2`) actually exercising a
        // narrowing assignment for the first time.
        let source = "n% = 27\nn% = n% / 2\nprint n%\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("bv_i_n = ((int)round((double)(((double)bv_i_n / (double)2))));"),
            "unexpected output:\n{output}"
        );
    }

    #[test]
    fn c_target_assignment_does_not_round_widening_conversions() {
        // int -> float/double needs no rounding decision -- C converts an
        // in-range integer to float/double exactly. round() must not
        // appear here.
        let source = "x! = 5\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("bv_f_x = 5;"),
            "unexpected output:\n{output}"
        );
        assert!(
            !output.contains("round("),
            "widening assignment should not round:\n{output}"
        );
    }

    #[test]
    fn c_target_select_case_compiles_numeric_single_range_and_is_clauses() {
        // Numeric `select case` compiles to a native if/else-if/else
        // chain against a once-evaluated temp -- single-value, `to`
        // range, and `is <op>` clauses all in one selector.
        let source = "n% = 5\n\
                       select case n%\n\
                           case 1, 2\n\
                           print \"low\"\n\
                           case 3 to 6\n\
                           print \"mid\"\n\
                           case is >= 7\n\
                           print \"high\"\n\
                           case else\n\
                           print \"other\"\n\
                       end select\n\
                       end\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("int bt_sel_0 = bv_i_n;"),
            "selector should be evaluated once into its own temp:\n{output}"
        );
        assert!(
            output.contains("if ((bt_sel_0 == 1) || (bt_sel_0 == 2)) {"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("} else if ((bt_sel_0 >= 3 && bt_sel_0 <= 6)) {"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("} else if ((bt_sel_0 >= 7)) {"),
            "unexpected output:\n{output}"
        );
        assert!(output.contains("} else {"), "unexpected output:\n{output}");
    }

    #[test]
    fn c_target_select_case_compiles_string_exact_match_clauses() {
        // A string selector is copied into its own char[256] temp (same
        // buffer convention as every other string value in this backend)
        // and tested with strcmp(...) == 0, not C's `==` -- and pulls in
        // <string.h> for it.
        let source = "d$ = \"Saturday\"\n\
                       select case d$\n\
                           case \"Saturday\", \"Sunday\"\n\
                           print \"weekend\"\n\
                           case else\n\
                           print \"weekday\"\n\
                       end select\n\
                       end\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("#include <string.h>"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("char bt_sel_0[256];"),
            "string selector should get its own buffer temp:\n{output}"
        );
        assert!(
            output.contains(
                "if ((strcmp(bt_sel_0, \"Saturday\") == 0) || (strcmp(bt_sel_0, \"Sunday\") == 0)) {"
            ),
            "unexpected output:\n{output}"
        );
    }

    #[test]
    fn c_target_supports_functions_with_byval_numeric_params_and_return() {
        // `a%`/`b%`'s 2-argument call parses as `Expr::Call` directly; a
        // real C function/call is the natural translation, unlike the
        // BASIC backend's GOSUB-against-shared-globals approach -- no
        // result variable, no label, just `int bf_i_max(...) { ... }`.
        let source = "print max%(4, 9)\nend\n";
        let program = "function max%(a%, b%)\n    if a% > b% then\n        return a%\n    \
                        else\n        return b%\n    end if\nend function\n";
        let output = compile_source_via_c_target(&format!("{program}{source}"));
        assert!(
            output.contains("int bf_i_max(int bv_i_a, int bv_i_b) {"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("return bv_i_a;"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("printf(\"%d\\n\", bf_i_max(4, 9));"),
            "unexpected output:\n{output}"
        );
    }

    #[test]
    fn c_target_functions_with_one_or_zero_arguments_parse_as_arrayref_not_call() {
        // A single-argument (or zero-argument) call to a suffixed function
        // name is syntactically ambiguous with array indexing, so the
        // parser produces `Expr::ArrayRef`, not `Expr::Call`, for both
        // shapes (see `make_paren_ident_expr` in parser.rs) -- codegen
        // must disambiguate using the function table, not just match on
        // `Expr::Call`.
        let source = "print addOne%(10)\nprint one%()\nend\n";
        let program = "function addOne%(x%)\n    return x% + 1\nend function\n\
                        function one%()\n    return 1\nend function\n";
        let output = compile_source_via_c_target(&format!("{program}{source}"));
        assert!(
            output.contains("bf_i_addone(10)"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("bf_i_one()"),
            "unexpected output:\n{output}"
        );
    }

    #[test]
    fn c_target_supports_string_returning_functions_with_byval_copy_semantics() {
        // A string-returning function is `void` in C with a trailing
        // `char* bcc_out` parameter (see `function_signature`); a byval
        // string parameter gets its own local buffer copied in from a
        // `..._in` pointer parameter at the top of the body, so
        // reassigning it inside the function can't corrupt the caller's
        // buffer.
        let source = "print repeat$(\"ab\", 3)\nend\n";
        let program = "function repeat$(text$, n%)\n    acc$ = \"\"\n    for i% = 1 to n%\n    \
                        acc$ = acc$ + text$\n    end for\n    return acc$\nend function\n";
        let output = compile_source_via_c_target(&format!("{program}{source}"));
        assert!(
            output.contains(
                "void bf_s_repeat(const char* bv_s_text_in, int bv_i_n, char* bcc_out) {"
            ),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("snprintf(bv_s_text, sizeof(bv_s_text), \"%s\", bv_s_text_in);"),
            "byval string parameter must get its own local copy:\n{output}"
        );
        assert!(
            output.contains("snprintf(bcc_out, 256, \"%s\", bv_s_acc);"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("bf_s_repeat(\"ab\", 3, bt_s_"),
            "call site should pass a fresh temp buffer as the out-param:\n{output}"
        );
    }

    #[test]
    fn c_target_functions_share_globals_via_the_global_keyword() {
        // `global x%` inside a function body means: don't declare a local
        // named `x%` here (see `emit_function_def`'s exclusion set) -- the
        // plain identifier then resolves to the file-scope `static`
        // global of the same name via ordinary C lexical scoping, no
        // per-use rewriting needed.
        let source = "total% = 0\ndummy% = addToTotal%(10)\ndummy% = addToTotal%(5)\n\
                       print total%\nend\n";
        let program = "function addToTotal%(x%)\n    global total%\n    total% = total% + x%\n    return total%\nend function\n";
        let output = compile_source_via_c_target(&format!("{program}{source}"));
        assert!(
            output.contains("static int bv_i_total = 0;"),
            "unexpected output:\n{output}"
        );
        assert!(
            !output.contains("int bf_i_addtototal(int bv_i_x) {\n    int bv_i_total"),
            "a `global`-declared name must not also be declared as a local:\n{output}"
        );
    }

    #[test]
    fn c_target_supports_input_with_a_prompt_for_each_scalar_type() {
        let source = "dim n$\ndim age%\ndim price!\ninput \"What is your name\"; n$\ninput \"How old are you\"; age%\ninput \"Price\"; price!\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("static char bcc_input_buf[256];")
                && output.contains("static void bcc_read_line(void) {"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("printf(\"What is your name? \");")
                && output.contains("printf(\"How old are you? \");")
                && output.contains("printf(\"Price? \");"),
            "the prompt should always get a trailing `? `, matching real BASIC's own INPUT:\n{output}"
        );
        assert!(
            output.contains("snprintf(bv_s_n, sizeof(bv_s_n), \"%s\", bcc_input_buf);"),
            "a string target should be copied straight from the input line:\n{output}"
        );
        assert!(
            output.contains("bv_i_age = atoi(bcc_input_buf);"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("bv_f_price = atof(bcc_input_buf);"),
            "unexpected output:\n{output}"
        );
    }

    #[test]
    fn c_target_supports_input_with_no_prompt() {
        let source = "dim x%\ninput x%\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("printf(\"? \");"),
            "a bare `input` with no prompt should still show the plain `? `:\n{output}"
        );
    }

    #[test]
    fn c_target_omits_input_helper_when_input_is_never_used() {
        let source = "print \"hi\"\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            !output.contains("bcc_input_buf"),
            "the input helper shouldn't be emitted when `input` is never called:\n{output}"
        );
    }

    #[test]
    fn c_target_rejects_input_with_more_than_one_variable() {
        let source = "dim a%, b%\ninput a%, b%\nend\n";
        let diagnostics = compile_source_via_c_target_err(source);
        assert!(
            diagnostics
                .iter()
                .any(|d| d.message.contains("more than one variable")),
            "unexpected diagnostics: {diagnostics:?}"
        );
    }

    #[test]
    fn c_target_supports_right() {
        let source = "dim s$\ns$ = \"Hello, World!\"\nprint right$(s$, 6)\nprint right$(s$, 100)\nprint right$(s$, 0)\nprint right$(\"BASCAL\", 3)\nend\n";
        let output = compile_source_via_c_target(source);
        // Regression check: RIGHT$ needs the same MID_HELPER block (for
        // bcc_mid) and <string.h> (for strlen inside render_right_call)
        // that MID$/LEFT$ already pull in via scan_builtin_usage --
        // missing either produces C that fails to *compile*, not
        // something `compile_source_via_c_target`'s own success check
        // alone would catch (it only inspects generated source text).
        assert!(
            output.contains("#include <string.h>"),
            "RIGHT$ needs <string.h> for strlen:\n{output}"
        );
        assert!(
            output.contains("static const char* bcc_mid("),
            "RIGHT$ needs the MID_HELPER block for bcc_mid:\n{output}"
        );
        assert!(
            output.contains(
                "bcc_mid(bv_s_s, (int)strlen(bv_s_s) - (6) + 1, 6)"
            ),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains(
                "bcc_mid(\"BASCAL\", (int)strlen(\"BASCAL\") - (3) + 1, 3)"
            ),
            "a string literal argument should be passed straight through:\n{output}"
        );
    }

    #[test]
    fn c_target_supports_val() {
        let source = "dim s$, n%\ns$ = \"42abc\"\nn% = val(s$)\nprint n%\nprint val(\"3.14\")\nprint val(\"not a number\")\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("#include <stdlib.h>"),
            "VAL needs <stdlib.h> for atof:\n{output}"
        );
        assert!(
            output.contains("atof(bv_s_s)"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("atof(\"3.14\")") && output.contains("atof(\"not a number\")"),
            "a string literal argument should be passed straight through:\n{output}"
        );
        assert!(
            output.contains("((int)round((double)(atof(bv_s_s))))"),
            "VAL assigned into an integer-suffixed variable should still round-narrow the \
             same way any other float-returning expression does:\n{output}"
        );
    }

    #[test]
    fn c_target_supports_instr() {
        let source = "dim s$, pos%\ns$ = \"hello world\"\npos% = instr(s$, \"world\")\nprint pos%\nprint instr(s$, \"xyz\")\nprint instr(\"BASCAL\", \"CAL\")\nend\n";
        let output = compile_source_via_c_target(source);
        // Regression check, same category as the RIGHT$ one above: the
        // bcc_instr helper calls strstr, which needs <string.h> -- missing
        // it produces C that fails to *compile*, not something a
        // generated-source-text check alone would catch.
        assert!(
            output.contains("#include <string.h>"),
            "INSTR needs <string.h> for strstr:\n{output}"
        );
        assert!(
            output.contains("static int bcc_instr("),
            "INSTR needs the INSTR_HELPER block for bcc_instr:\n{output}"
        );
        assert!(
            output.contains("bcc_instr(bv_s_s, \"world\")"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("bcc_instr(\"BASCAL\", \"CAL\")"),
            "a string literal argument should be passed straight through:\n{output}"
        );
    }

    #[test]
    fn c_target_omits_instr_helper_when_instr_is_never_used() {
        let source = "print \"hi\"\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            !output.contains("bcc_instr"),
            "the INSTR helper shouldn't be emitted when `instr` is never called:\n{output}"
        );
    }

    #[test]
    fn c_target_supports_sqr_abs_int_fix() {
        let source = "dim x!\nx! = 9.0\nprint sqr(x!)\nprint abs(-5.5)\nprint abs(-5)\nprint int(3.7)\nprint int(-3.7)\nprint fix(3.7)\nprint fix(-3.7)\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("sqrt((double)(bv_f_x))"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("fabs((double)(-(5.5)))"),
            "a float ABS argument should stay float-typed:\n{output}"
        );
        assert!(
            output.contains("(int)(fabs((double)(-(5))))"),
            "an int ABS argument should stay int-typed:\n{output}"
        );
        assert!(
            output.contains("floor((double)(3.7))") && output.contains("floor((double)(-(3.7)))"),
            "INT should use floor (round toward negative infinity):\n{output}"
        );
        assert!(
            output.contains("trunc((double)(3.7))") && output.contains("trunc((double)(-(3.7)))"),
            "FIX should use trunc (round toward zero):\n{output}"
        );
    }

    #[test]
    fn c_target_supports_sgn() {
        let source = "print sgn(-9.0)\nprint sgn(0.0)\nprint sgn(9.0)\nend\n";
        let output = compile_source_via_c_target(source);
        // Regression check, same category as RIGHT$/INSTR above: bcc_sgn
        // is a genuine helper function, not an inline expression -- a
        // missing SGN_HELPER splice would fail to *compile*, not just
        // look wrong in generated source text.
        assert!(
            output.contains("static int bcc_sgn("),
            "SGN needs the SGN_HELPER block for bcc_sgn:\n{output}"
        );
        assert!(
            output.contains("bcc_sgn((double)(-(9.0)))")
                && output.contains("bcc_sgn((double)(0.0))")
                && output.contains("bcc_sgn((double)(9.0))"),
            "unexpected output:\n{output}"
        );
    }

    #[test]
    fn c_target_omits_sgn_helper_when_sgn_is_never_used() {
        let source = "print \"hi\"\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            !output.contains("bcc_sgn"),
            "the SGN helper shouldn't be emitted when `sgn` is never called:\n{output}"
        );
    }

    #[test]
    fn c_target_supports_cint_clng_csng_cdbl() {
        let source = "dim f!\nf! = 3.6\nprint cint(f!)\nprint clng(-3.6)\nprint csng(7)\nprint cdbl(7)\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("((int)round((double)(bv_f_f)))"),
            "CINT should round to the nearest integer, not truncate:\n{output}"
        );
        assert!(
            output.contains("((int)round((double)(-(3.6))))"),
            "CLNG should round the same way CINT does:\n{output}"
        );
        assert!(
            output.contains("((float)(7))"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("((double)(7))"),
            "unexpected output:\n{output}"
        );
    }

    #[test]
    fn c_target_supports_trig_and_log_exp() {
        let source = "print sin(0.0)\nprint cos(0.0)\nprint tan(0.0)\nprint atn(1.0)\nprint log(1.0)\nprint exp(0.0)\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("sin((double)(0.0))"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("cos((double)(0.0))"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("tan((double)(0.0))"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("atan((double)(1.0))"),
            "ATN should map to atan(), not atn():\n{output}"
        );
        assert!(
            output.contains("log((double)(1.0))"),
            "LOG should be the natural log, not log10:\n{output}"
        );
        assert!(
            output.contains("exp((double)(0.0))"),
            "unexpected output:\n{output}"
        );
    }

    #[test]
    fn c_target_supports_rnd() {
        let source = "print rnd(1)\nprint rnd(0)\nprint rnd(-5)\nprint rnd()\nend\n";
        let output = compile_source_via_c_target(source);
        // Regression check, same category as SGN/INSTR above: bcc_rnd is a
        // genuine helper function (plus its bcc_rnd_last state), not an
        // inline expression -- a missing RND_HELPER splice would fail to
        // *compile*, not just look wrong in generated source text.
        assert!(
            output.contains("static double bcc_rnd("),
            "RND needs the RND_HELPER block for bcc_rnd:\n{output}"
        );
        assert!(
            output.contains("#include <stdlib.h>"),
            "RND needs <stdlib.h> for rand()/srand():\n{output}"
        );
        assert!(
            output.contains("bcc_rnd((double)(1))"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("bcc_rnd((double)(0))"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("bcc_rnd((double)(-(5)))"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("bcc_rnd(1.0)"),
            "the no-argument RND() call should pass through as a literal 1.0, real BASIC's own \
             shorthand for \"draw the next value\":\n{output}"
        );
    }

    #[test]
    fn c_target_omits_rnd_helper_when_rnd_is_never_used() {
        let source = "print \"hi\"\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            !output.contains("bcc_rnd"),
            "the RND helper shouldn't be emitted when `rnd` is never called:\n{output}"
        );
    }

    #[test]
    fn c_target_supports_randomize_with_a_numeric_seed() {
        let source = "randomize 42\nprint rnd(1)\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("srand((unsigned int)(42));"),
            "unexpected output:\n{output}"
        );
        assert!(
            !output.contains("#include <time.h>"),
            "a numeric-seed RANDOMIZE needs no time.h -- only bare RANDOMIZE/RANDOMIZE TIMER \
             do:\n{output}"
        );
    }

    #[test]
    fn c_target_supports_bare_randomize_and_randomize_timer() {
        let source = "randomize\nrandomize timer\nprint rnd(1)\nend\n";
        let output = compile_source_via_c_target(source);
        assert_eq!(
            output.matches("srand((unsigned int)time(NULL));").count(),
            2,
            "both bare RANDOMIZE and RANDOMIZE TIMER should reseed from the current time, the \
             closest available stand-in for real BASIC's interactive seed prompt:\n{output}"
        );
        assert!(
            output.contains("#include <time.h>"),
            "RANDOMIZE/RANDOMIZE TIMER need <time.h> for time():\n{output}"
        );
    }

    #[test]
    fn c_target_supports_labels_and_goto() {
        let source = "dim i%\ni% = 0\ntop:\ni% = i% + 1\nif i% < 3 then goto top\ngoto skip\nprint \"unreachable\"\nskip:\nprint \"done\"\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("bcc_lbl_top:;"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("goto bcc_lbl_top;"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("bcc_lbl_skip:;") && output.contains("goto bcc_lbl_skip;"),
            "unexpected output:\n{output}"
        );
    }

    #[test]
    fn c_target_label_names_are_case_insensitive() {
        let source = "TOP:\ngoto top\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("bcc_lbl_top:;") && output.contains("goto bcc_lbl_top;"),
            "a label and a differently-cased GOTO to it should resolve to the same C label:\n{output}"
        );
    }

    #[test]
    fn c_target_supports_gosub_and_return() {
        let source = "print \"before\"\ngosub greet\nprint \"after\"\ngoto skip\ngreet:\nprint \"  hi\"\nreturn\nskip:\nprint \"done\"\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("#define BCC_MAX_GOSUB_DEPTH"),
            "GOSUB needs the GOSUB_HELPER block:\n{output}"
        );
        assert!(
            output.contains("bcc_gosub_stack[bcc_gosub_sp++] = 0;")
                && output.contains("goto bcc_lbl_greet;")
                && output.contains("bcc_ret_0:;"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("switch (bcc_gosub_stack[--bcc_gosub_sp]) {")
                && output.contains("case 0: goto bcc_ret_0;"),
            "RETURN should dispatch back through the ID stack:\n{output}"
        );
    }

    #[test]
    fn c_target_supports_a_gosub_shared_by_two_call_sites() {
        // The same RETURN is reached from two different GOSUB call sites
        // -- real GOSUB/RETURN's whole point (see `Statement::ReturnVoid`'s
        // own doc comment): the switch dispatch has to cover both IDs.
        let source = "gosub greet\ngosub greet\ngoto skip\ngreet:\nprint \"hi\"\nreturn\nskip:\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("bcc_gosub_stack[bcc_gosub_sp++] = 0;")
                && output.contains("bcc_ret_0:;")
                && output.contains("bcc_gosub_stack[bcc_gosub_sp++] = 1;")
                && output.contains("bcc_ret_1:;"),
            "each GOSUB call site should get its own resume ID:\n{output}"
        );
        assert!(
            output.contains("case 0: goto bcc_ret_0;") && output.contains("case 1: goto bcc_ret_1;"),
            "the shared RETURN's dispatch should cover both call sites' IDs:\n{output}"
        );
    }

    #[test]
    fn c_target_rejects_gosub_inside_a_procedure() {
        let source = "procedure p()\n    gosub top\nend procedure\np()\ntop:\nend\n";
        let diagnostics = compile_source_via_c_target_err(source);
        assert!(
            diagnostics
                .iter()
                .any(|d| d.message.contains("function/procedure")),
            "unexpected diagnostics: {diagnostics:?}"
        );
    }

    #[test]
    fn c_target_omits_gosub_helper_when_gosub_is_never_used() {
        let source = "print \"hi\"\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            !output.contains("BCC_MAX_GOSUB_DEPTH") && !output.contains("bcc_gosub_stack"),
            "the GOSUB helper shouldn't be emitted when GOSUB is never used:\n{output}"
        );
    }

    #[test]
    fn c_target_supports_on_error_goto_and_error_and_err() {
        let source = "on error goto h\nerror 53\ngoto after\nh:\nprint err\nresume next\nafter:\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("static int bcc_err = 0;")
                && output.contains("static int bcc_on_error_target = -1;")
                && output.contains("static int bcc_in_handler = 0;")
                && output.contains("static int bcc_resume_id = -1;"),
            "ON ERROR GOTO needs the ERROR_HANDLING_GLOBALS block:\n{output}"
        );
        assert!(
            output.contains("bcc_on_error_target = 0;"),
            "ON ERROR GOTO h should install h as handler 0:\n{output}"
        );
        assert!(
            output.contains("bcc_raise_retry_0: ;")
                && output.contains("bcc_err = 53;")
                && output.contains("bcc_resume_id = 0;")
                && output.contains("case 0: goto bcc_lbl_h;")
                && output.contains("bcc_raise_after_0: ;"),
            "ERROR 53 should be a raise site dispatching to handler 0:\n{output}"
        );
        assert!(
            output.contains("printf(\"%d\\n\", bcc_err)"),
            "bare ERR should read bcc_err, not an ordinary variable:\n{output}"
        );
    }

    #[test]
    fn c_target_on_error_goto_0_disables_the_trap() {
        let source = "on error goto 0\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("bcc_on_error_target = -1;"),
            "ON ERROR GOTO 0 should disable the trap, not target a handler:\n{output}"
        );
    }

    #[test]
    fn c_target_resume_same_dispatches_to_the_raise_sites_retry_label() {
        let source = "on error goto h\nerror 5\ngoto after\nh:\nresume\nafter:\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("switch (bcc_resume_id) {") && output.contains("case 0: goto bcc_raise_retry_0;"),
            "bare RESUME should dispatch back to the raise site's own retry label:\n{output}"
        );
        assert!(
            output.contains("bcc_in_handler = 0;"),
            "RESUME should clear bcc_in_handler so a later error can trap again:\n{output}"
        );
    }

    #[test]
    fn c_target_resume_next_dispatches_to_the_raise_sites_after_label() {
        let source = "on error goto h\nerror 5\ngoto after\nh:\nresume next\nafter:\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("switch (bcc_resume_id) {") && output.contains("case 0: goto bcc_raise_after_0;"),
            "RESUME NEXT should dispatch to the raise site's own after-label:\n{output}"
        );
    }

    #[test]
    fn c_target_resume_label_jumps_directly_with_no_dispatch() {
        let source = "on error goto h\nerror 5\ngoto after\nh:\nresume after\nafter:\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("bcc_in_handler = 0;\n    goto bcc_lbl_after;"),
            "RESUME <label> should jump directly to the label, no runtime dispatch needed:\n{output}"
        );
    }

    #[test]
    fn c_target_open_for_input_raises_error_53_on_a_missing_file() {
        let source =
            "on error goto h\nopen \"missing.dat\" for input as #1\ngoto after\nh:\nprint err\nresume next\nafter:\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("if (!bcc_files[0]) {") && output.contains("bcc_err = 53;"),
            "a failed sequential OPEN FOR INPUT should raise error 53:\n{output}"
        );
    }

    #[test]
    fn c_target_rejects_error_handling_statements_inside_a_procedure() {
        for stmt in ["on error goto h", "error 5", "resume next"] {
            let source = format!("procedure p()\n    {stmt}\nend procedure\np()\nh:\nend\n");
            let diagnostics = compile_source_via_c_target_err(&source);
            assert!(
                diagnostics
                    .iter()
                    .any(|d| d.message.contains("function/procedure")),
                "`{stmt}` inside a procedure should be rejected: {diagnostics:?}"
            );
        }
    }

    #[test]
    fn c_target_supports_data_read_and_restore() {
        let source = "read a$, b%\nprint a$\nprint b%\nrestore\nread c$\nprint c$\nend\ndata \"hello\", 42\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("#define BCC_DATA_COUNT 2")
                && output.contains("static const char* bcc_data[BCC_DATA_COUNT] = { \"hello\", \"42\" };"),
            "DATA items should be flattened into one static array:\n{output}"
        );
        assert!(
            output.contains("snprintf(bv_s_a, sizeof(bv_s_a), \"%s\", bcc_read_data());"),
            "READ into a string var should copy bcc_read_data()'s text:\n{output}"
        );
        assert!(
            output.contains("bv_i_b = atoi(bcc_read_data());"),
            "READ into a numeric var should parse bcc_read_data()'s text:\n{output}"
        );
        assert!(
            output.contains("bcc_data_ptr = 0;"),
            "bare RESTORE should rewind to the start:\n{output}"
        );
    }

    #[test]
    fn c_target_restore_to_a_label_rewinds_to_that_labels_data() {
        let source =
            "read first$\nrestore second\nread other$\nprint first$\nprint other$\nend\ndata \"a\"\nsecond:\ndata \"b\"\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("bcc_data_ptr = 1;"),
            "RESTORE second should resolve to the compile-time item count before that label \
             (1 item -- \"a\" -- precedes it):\n{output}"
        );
    }

    #[test]
    fn c_target_rejects_read_when_the_program_has_no_data() {
        let source = "dim x%\nread x%\nend\n";
        let diagnostics = compile_source_via_c_target_err(source);
        assert!(
            diagnostics.iter().any(|d| d.message.contains("no `data` items")),
            "unexpected diagnostics: {diagnostics:?}"
        );
    }

    #[test]
    fn c_target_supports_1d_numeric_array_declaration_indexing_and_assignment() {
        let source = "dim scores%(5)\nscores%(0) = 10\nscores%(1) = scores%(0) + 5\nprint scores%(1)\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("static int bv_i_scores[6] = {0};"),
            "dim scores%(5) should declare 6 elements (0..=5), zero-initialized:\n{output}"
        );
        assert!(
            output.contains("bv_i_scores[(0)] = 10;"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("bv_i_scores[(1)] = (bv_i_scores[(0)] + 5);"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("printf(\"%d\\n\", bv_i_scores[(1)])"),
            "unexpected output:\n{output}"
        );
    }

    #[test]
    fn c_target_supports_string_array_declaration_indexing_and_assignment() {
        let source = "dim country$(2)\ncountry$(0) = \"France\"\nprint country$(0)\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains(&format!(
                "static char bv_s_country[3][{}] = {{0}};",
                256
            )),
            "dim country$(2) should declare 3 string elements:\n{output}"
        );
        assert!(
            output.contains("snprintf(bv_s_country[(0)], sizeof(bv_s_country[(0)]), \"%s\", \"France\");"),
            "a string array element can't be assigned via plain =, needs snprintf:\n{output}"
        );
        assert!(
            output.contains("printf(\"%s\\n\", bv_s_country[(0)])"),
            "unexpected output:\n{output}"
        );
    }

    #[test]
    fn c_target_supports_2d_array_declaration_and_indexing() {
        let source = "dim grid%(2, 2)\ngrid%(1, 1) = 7\nprint grid%(1, 1)\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("static int bv_i_grid[3][3] = {0};"),
            "dim grid%(2, 2) should declare a native 3x3 C array, no manual flattening:\n{output}"
        );
        assert!(
            output.contains("bv_i_grid[(1)][(1)] = 7;")
                && output.contains("printf(\"%d\\n\", bv_i_grid[(1)][(1)])"),
            "unexpected output:\n{output}"
        );
    }

    #[test]
    fn c_target_array_bound_accepts_a_top_level_int_const() {
        // tutorial/09_data.bcl's own shape: `const numCapitals% = 5` then
        // `dim country$(numCapitals%)` -- a real C array needs a literal
        // size, so the const's own integer value must be recovered at
        // compile time, not treated as a runtime-only variable read.
        let source = "const n% = 5\ndim arr%(n%)\narr%(0) = 1\nprint arr%(0)\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("static int bv_i_arr[6] = {0};"),
            "the array bound should resolve the const's literal value (5), giving 6 elements:\n{output}"
        );
    }

    #[test]
    fn c_target_rejects_a_non_literal_array_bound() {
        let source = "dim n%\nn% = 5\ndim arr%(n%)\nend\n";
        let diagnostics = compile_source_via_c_target_err(source);
        assert!(
            diagnostics
                .iter()
                .any(|d| d.message.contains("compile-time-known size")),
            "unexpected diagnostics: {diagnostics:?}"
        );
    }

    #[test]
    fn c_target_supports_sizeof_1d_and_2d() {
        // `sizeof` returns the array's own declared bound -- the same
        // value `dim` used -- not the element count, matching
        // `docs/manual/arrays.html#sizeof`'s own `dim data%(9)` /
        // `sizeof(data%) = 9` example and `--target basic`'s
        // `resolve_axis_bound` exactly (see `render_numeric_call`'s
        // `sizeof` arm in codegen_c.rs).
        let source =
            "dim arr%(4)\ndim grid%(9, 4)\nprint sizeof(arr%)\nprint sizeof(grid%, 0)\nprint sizeof(grid%, 1)\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("printf(\"%d\\n\", 9)"),
            "sizeof(grid%(9, 4), 0) should resolve to the literal 9:\n{output}"
        );
        assert!(
            output.contains("printf(\"%d\\n\", 4)") && output.matches("printf(\"%d\\n\", 4)").count() >= 2,
            "sizeof(arr%(4)) and sizeof(grid%(9, 4), 1) should both resolve to the literal 4:\n{output}"
        );
    }

    #[test]
    fn c_target_sizeof_without_an_axis_on_a_multidim_array_is_an_error() {
        let source = "dim grid%(9, 9)\nprint sizeof(grid%)\nend\n";
        let diagnostics = compile_source_via_c_target_err(source);
        assert!(
            diagnostics
                .iter()
                .any(|d| d.message.contains("sizeof needs an axis argument")),
            "unexpected diagnostics: {diagnostics:?}"
        );
    }

    #[test]
    fn c_target_supports_swap_of_scalars_and_array_elements() {
        let source = "dim a%, b%\na% = 1\nb% = 2\nswap a%, b%\ndim arr$(2)\narr$(0) = \"x\"\narr$(1) = \"y\"\nswap arr$(0), arr$(1)\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("int bt_swap_0 = bv_i_a;")
                && output.contains("bv_i_a = bv_i_b;")
                && output.contains("bv_i_b = bt_swap_0;"),
            "scalar SWAP should use a real temp, no aliasing:\n{output}"
        );
        assert!(
            output.contains("char bt_swap_1[")
                && output.contains(
                    "snprintf(bv_s_arr[(0)], sizeof(bv_s_arr[(0)]), \"%s\", bv_s_arr[(1)]);"
                ),
            "a string array element SWAP should go through a temp buffer + snprintf, not plain \
             assignment (C arrays can't be assigned with =):\n{output}"
        );
    }

    #[test]
    fn c_target_supports_read_into_an_array_element() {
        let source = "dim country$(1)\nread country$(0), country$(1)\nprint country$(0)\nend\ndata \"France\", \"Japan\"\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains(
                "snprintf(bv_s_country[(0)], sizeof(bv_s_country[(0)]), \"%s\", bcc_read_data());"
            ),
            "READ into a string array element should snprintf bcc_read_data()'s text:\n{output}"
        );
    }

    #[test]
    fn c_target_supports_cls_and_beep() {
        let source = "cls\nbeep\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("printf(\"\\x1b[2J\\x1b[H\");"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("printf(\"\\a\");"),
            "unexpected output:\n{output}"
        );
    }

    #[test]
    fn c_target_supports_locate() {
        let source = "locate 5, 10\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("printf(\"\\x1b[%d;%dH\", 5, 10);"),
            "LOCATE row, col should map straight to ANSI's own row;col cursor-position \
             escape, no reordering:\n{output}"
        );
    }

    #[test]
    fn c_target_supports_color_with_and_without_background() {
        let source = "color 14, 1\ncolor 7\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("static const int bcc_ansi_fg[16] ="),
            "the color helper should only be emitted once color is actually used:\n{output}"
        );
        assert!(
            output.contains("bcc_color(14, 1);"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("bcc_color(7, -1);"),
            "a bare `color fg` (no background) should pass -1, leaving the background alone:\n{output}"
        );
    }

    #[test]
    fn c_target_omits_color_helper_when_color_is_never_used() {
        let source = "cls\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            !output.contains("bcc_ansi_fg"),
            "the color helper shouldn't be emitted when `color` is never called:\n{output}"
        );
    }

    #[test]
    fn c_target_supports_locate_and_color_with_variable_arguments() {
        // Regression check for collect_vars_in_statement: row/col/fg/bg
        // expressions must be scanned for variable declarations, the same
        // as any other statement's expressions, or referencing a variable
        // only inside locate/color would compile to an undeclared C name.
        let source = "row% = 5\ncol% = 10\nfg% = 14\nbg% = 1\nlocate row%, col%\ncolor fg%, bg%\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("static int bv_i_row = 0;")
                && output.contains("static int bv_i_col = 0;")
                && output.contains("static int bv_i_fg = 0;")
                && output.contains("static int bv_i_bg = 0;"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("printf(\"\\x1b[%d;%dH\", bv_i_row, bv_i_col);"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("bcc_color(bv_i_fg, bv_i_bg);"),
            "unexpected output:\n{output}"
        );
    }

    #[test]
    fn c_target_supports_a_zero_arg_procedure_call() {
        // A zero-argument call always parses as `Expr::ArrayRef`, not
        // `Expr::Call` (see `make_paren_ident_expr` in parser.rs) -- this
        // exercises that shape specifically, not just the general case
        // covered below.
        let source = "procedure sayHi()\n    print \"hi\"\nend procedure\nsayHi()\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("void bf_i_sayhi(void) {"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("    bf_i_sayhi();\n"),
            "a zero-arg procedure call should compile to a plain C call:\n{output}"
        );
    }

    #[test]
    fn c_target_supports_procedures_with_parameters_and_implicit_return() {
        let source = "procedure showTotal(amount!)\n    print \"Total: \"; amount!\nend procedure\nshowTotal(42.5)\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("void bf_i_showtotal(float bv_f_amount) {"),
            "a procedure should compile to a void C function:\n{output}"
        );
        assert!(
            output.contains("    bf_i_showtotal(42.5);\n"),
            "unexpected output:\n{output}"
        );
    }

    #[test]
    fn c_target_supports_an_early_bare_return_inside_a_procedure() {
        let source = "procedure greet(name$)\n    if name$ = \"\" then\n        print \"Hello, stranger.\"\n        return\n    end if\n    print \"Hello, \" + name$ + \".\"\nend procedure\ngreet(\"Ada\")\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("    return;\n"),
            "a bare `return` inside a procedure should compile to `return;`:\n{output}"
        );
    }

    #[test]
    fn c_target_supports_discarding_a_functions_return_value_as_a_bare_statement() {
        let source = "function larger%(a%, b%)\n    if a% > b% then\n        return a%\n    end if\n    return b%\nend function\nlarger%(3, 9)\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("    bf_i_larger(3, 9);\n"),
            "a value-returning function called as a bare statement should still just call it, \
             discarding the result:\n{output}"
        );
    }

    #[test]
    fn c_target_rejects_bare_return_outside_any_function_or_procedure() {
        // Sanity check: `return` outside any function is still rejected the
        // same way it always was -- procedure support shouldn't have loosened
        // this.
        let source = "return\nend\n";
        let diagnostics = compile_source_via_c_target_err(source);
        assert!(
            diagnostics
                .iter()
                .any(|d| d.message.contains("`return` outside of a function")),
            "unexpected diagnostics: {diagnostics:?}"
        );
    }

    #[test]
    fn c_target_supports_byref_scalar_parameters() {
        // A `byref` scalar parameter compiles to a real C pointer, with
        // copy-in/copy-out around it -- see `FnParam::by_ref`'s doc
        // comment in codegen_c.rs. The C function itself takes `int*`, and
        // the call site passes the caller's variable's address.
        let source = "function bump%(byref x%)\n    x% = x% + 1\n    return x%\nend function\n\
                       n% = 5\nprint bump%(n%)\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("bf_i_bump(int* bv_i_x_in)"),
            "byref scalar parameter should compile to a pointer:\n{output}"
        );
        assert!(
            output.contains("int bv_i_x = *bv_i_x_in;"),
            "byref scalar parameter should copy in from the pointer:\n{output}"
        );
        assert!(
            output.contains("*bv_i_x_in = bv_i_x;"),
            "byref scalar parameter should copy its result back out through the pointer:\n{output}"
        );
        assert!(
            output.contains("bf_i_bump(&bv_i_n)"),
            "the call site should pass the caller's variable's address:\n{output}"
        );
    }

    #[test]
    fn c_target_byref_scalar_argument_must_be_a_plain_variable() {
        let source = "function bump%(byref x%)\n    x% = x% + 1\n    return x%\nend function\n\
                       print bump%(5)\nend\n";
        let diagnostics = compile_source_via_c_target_err(source);
        assert!(
            diagnostics
                .iter()
                .any(|d| d.message.contains("byref") && d.message.contains("plain variable")),
            "unexpected diagnostics: {diagnostics:?}"
        );
    }

    #[test]
    fn c_target_supports_byval_array_parameters() {
        // A `byval` array parameter copies the caller's array into a
        // local buffer before the call -- writes inside the function
        // never reach the caller's own storage.
        let source = "function firstDoubled%(arr%(?))\n    arr%(0) = arr%(0) * 2\n    \
                       return arr%(0)\nend function\n\
                       dim data%(2)\n\
                       data%(0) = 5\n\
                       print firstDoubled%(data%)\n\
                       print data%(0)\n\
                       end\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("_len0"),
            "array parameter should carry a hidden element-count parameter:\n{output}"
        );
        assert!(
            output.contains("for (int bcc_i = 0;"),
            "byval array parameter should copy the caller's array into a local buffer:\n{output}"
        );
    }

    #[test]
    fn c_target_supports_byref_array_parameters() {
        // A `byref` array parameter passes the caller's array by real
        // pointer, natively -- no copy in, no copy back needed, and
        // mutations are visible to the caller immediately.
        let source = "function doubleFirst%(byref arr%(?))\n    arr%(0) = arr%(0) * 2\n    \
                       return 0\nend function\n\
                       dim data%(2)\n\
                       data%(0) = 5\n\
                       dummy% = doubleFirst%(data%)\n\
                       print data%(0)\n\
                       end\n";
        let output = compile_source_via_c_target(source);
        assert!(
            !output.contains("for (int bcc_i = 0;"),
            "byref array parameter shouldn't need a copy-in loop at all:\n{output}"
        );
        assert!(
            output.contains("bf_i_doublefirst(bv_i_data,"),
            "byref array argument should pass the real array pointer directly:\n{output}"
        );
    }

    #[test]
    fn c_target_supports_a_local_array_dimd_inside_a_function_body() {
        // A `dim` inside a function/procedure body (not top-level) needs
        // its own real local C array declared in that function's own
        // prologue -- see `collect_array_declarations`'s call site in
        // `generate` and `function_scoped_table`'s own doc comment. This
        // is exactly the shape `tutorial/com/bascal/sort/quickSort.bcl`
        // needs for its explicit partition-bounds stack (`sLow%`/
        // `sHigh%`), which `tutorial/sort_driver.bcl` depends on.
        let source = "function useLocal%(byref arr%(?))\n    \
                       dim scratch%(3)\n    \
                       scratch%(0) = 10\n    \
                       scratch%(3) = 20\n    \
                       arr%(0) = scratch%(0) + scratch%(3)\n    \
                       return 0\nend function\n\
                       dim data%(2)\n\
                       dummy% = useLocal%(data%)\n\
                       print data%(0)\n\
                       end\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("int bv_i_scratch[4]"),
            "a local `dim scratch%(3)` should declare a true 4-element C local array \
             inside the function body:\n{output}"
        );
        assert!(
            !output.contains("static int bv_i_scratch"),
            "a local array must not be hoisted to file scope like a top-level `dim`:\n{output}"
        );
    }

    #[test]
    fn c_target_rejects_a_function_body_that_might_not_return() {
        let source = "function maybe%(x%)\n    if x% > 0 then\n        return x%\n    end if\nend function\n\
                       print maybe%(5)\nend\n";
        let diagnostics = compile_source_via_c_target_err(source);
        assert!(
            diagnostics
                .iter()
                .any(|d| d.message.contains("must end with an explicit `return`")),
            "unexpected diagnostics: {diagnostics:?}"
        );
    }

    #[test]
    fn c_target_supports_asc_chr_mid_left_builtins() {
        // ASC/CHR$/MID$/LEFT$ round-trip through the `bcc_mid`/`bcc_chr`
        // ring-buffer helpers (see `MID_HELPER` in codegen_c.rs) so a
        // nested call (ASC of a MID$/CHR$ result) works even inside a
        // numeric context, which has no prelude mechanism of its own.
        let source = "s$ = \"hello\"\n\
                       print asc(s$)\n\
                       print chr$(65)\n\
                       print mid$(s$, 2)\n\
                       print mid$(s$, 2, 2)\n\
                       print left$(s$, 3)\n\
                       print asc(mid$(s$, 1, 1))\n\
                       end\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("#define BCC_STRBUF_COUNT"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("static const char* bcc_mid("),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("static const char* bcc_chr("),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("((int)(unsigned char)bv_s_s[0])"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("bcc_chr(65)"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("bcc_mid(bv_s_s, 2, 2147483647)"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("bcc_mid(bv_s_s, 2, 2)"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("bcc_mid(bv_s_s, 1, 3)"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("((int)(unsigned char)bcc_mid(bv_s_s, 1, 1)[0])"),
            "ASC of a nested MID$ call must not need a prelude:\n{output}"
        );
    }

    #[test]
    fn c_target_supports_short_circuit_and_or_in_conditions() {
        // BASCAL's `&&`/`||` are already real short-circuit operators
        // (unlike classic BASIC's bitwise-only AND/OR) -- C's own
        // `&&`/`||` are the direct, correct translation, not a bug the
        // way reusing them for bitwise AND/OR would be.
        let source = "x% = 5\nif x% > 0 && x% < 10 then\n    print \"in range\"\nend if\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("if (((-(bv_i_x > 0)) && (-(bv_i_x < 10)))) {"),
            "unexpected output:\n{output}"
        );
    }

    #[test]
    fn c_target_compiles_functions_tutorial() {
        // The tutorial that motivated adding function support in the
        // first place -- byval scalar functions, nested calls, `global`,
        // string functions, and (via its two `require`d library
        // functions) the LEN/ASC/CHR$/MID$/LEFT$ builtins all in one
        // real program.
        let input = Path::new(env!("CARGO_MANIFEST_DIR")).join("tutorial/07_functions.bcl");
        let options = CompileOptions {
            target: Target::C,
            ..CompileOptions::new()
        };
        compile_file(&input, &options)
            .unwrap_or_else(|d| panic!("tutorial/07_functions.bcl should compile to C: {d:?}"));
    }

    #[test]
    fn c_target_compiles_arrays_tutorial() {
        // Byval/byref array parameters, `sizeof`, and 2-D arrays --
        // verified correct end to end with gcc separately (matches
        // `--target basic`'s own real output once `--target basic`'s own
        // array-parameter-copy issue, filed separately, is accounted
        // for); this locks in that it keeps compiling.
        let input = Path::new(env!("CARGO_MANIFEST_DIR")).join("tutorial/08_arrays.bcl");
        let options = CompileOptions {
            target: Target::C,
            ..CompileOptions::new()
        };
        compile_file(&input, &options)
            .unwrap_or_else(|d| panic!("tutorial/08_arrays.bcl should compile to C: {d:?}"));
    }

    #[test]
    fn c_target_compiles_sort_driver_sample() {
        // Exercises `byref` array parameters, `byref` scalar parameters
        // (indirectly, via `bubbleSort%`/`shakerSort%`/`shellSort%`/
        // `quickSort%`'s own `byref data%(?)`), and a local (non-top-
        // level) `dim`'d array inside a function body --
        // `quickSort%`'s own explicit partition-bounds stack
        // (`sLow%(64)`/`sHigh%(64)`). Verified correct end to end with
        // gcc separately (all four sorts report OK on both a 50- and a
        // 5000-element reverse-sorted input); this locks in that it
        // keeps compiling.
        let input = Path::new(env!("CARGO_MANIFEST_DIR")).join("tutorial/sort_driver.bcl");
        let options = CompileOptions {
            target: Target::C,
            ..CompileOptions::new()
        };
        compile_file(&input, &options)
            .unwrap_or_else(|d| panic!("tutorial/sort_driver.bcl should compile to C: {d:?}"));
    }

    #[test]
    fn c_target_compiles_random_access_files_tutorial() {
        // Both Part 1 (hand-written FIELD/GET/PUT/LSET) and Part 2 (the
        // record/file DSL, which lowers to the same primitives) --
        // verified correct end to end with gcc separately; this just
        // locks in that it keeps compiling.
        let input =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tutorial/15_random_and_record_files.bcl");
        let options = CompileOptions {
            target: Target::C,
            ..CompileOptions::new()
        };
        compile_file(&input, &options).unwrap_or_else(|d| {
            panic!("tutorial/15_random_and_record_files.bcl should compile to C: {d:?}")
        });
    }

    #[test]
    fn c_target_supports_suffixless_numeric_variables() {
        // Real MBASIC/BASCOM's own default for a variable with no type
        // suffix is single-precision floating point -- BASCAL exposes no
        // DEFINT/DEFSNG/etc to override that default, so it's the one
        // correct fill-in here, not a guess (see `effective_suffix` in
        // codegen_c.rs).
        let source = "for i = 1 to 3\n    print i\nend for\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("float bv_f_i = 0;"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("for (bv_f_i = 1; "),
            "suffixless loop variable should use the Single-precision C variable:\n{output}"
        );
    }

    #[test]
    fn c_target_field_layout_tracks_program_order_not_last_field_wins() {
        // Regression test for a real bug: re-FIELDing the same channel
        // number later in the program (reopened under different buffer
        // variable names, exactly what tutorial 15's Part 1/Part 2 both
        // do on channel #1) used to make *every* GET/PUT on that channel
        // -- including ones textually *before* the second FIELD -- use
        // the last FIELD's layout, since it was computed once by an
        // up-front whole-program scan instead of tracked live in program
        // order (see FileIoLayout's doc comment in codegen_c.rs).
        let source = "open \"a.dat\" for random as #1 len = 2\n\
                       field #1, 2 as firstBuf$\n\
                       get #1, 1\n\
                       close #1\n\
                       open \"b.dat\" for random as #1 len = 3\n\
                       field #1, 3 as secondBuf$\n\
                       get #1, 1\n\
                       close #1\n\
                       end\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("bcc_get_record_fields_1_0(bcc_files[0], 1, bv_s_firstbuf);"),
            "the first GET must use the first FIELD layout helper:\n{output}"
        );
        assert!(
            output.contains("bcc_get_record_fields_1_1(bcc_files[0], 1, bv_s_secondbuf);"),
            "the second GET must use the second FIELD layout helper:\n{output}"
        );
    }

    #[test]
    fn c_target_record_field_accessed_from_a_procedure_body_sees_its_layout() {
        // Regression test for a real bug: a function/procedure body is
        // emitted *before* the top-level FIELD declarations that establish
        // a channel's layout, even though those declarations always run
        // first at actual program execution -- found via
        // tutorial/card_catalog.bcl while implementing procedure support.
        let source = r#"record Header
    size: int16
end record

file header as Header = open("catalog.dat")

procedure touch()
    header[1] = { size: 1 }
end procedure

touch()
end
"#;
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("static int bcc_put_record_header("),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("bcc_put_record_header(bcc_files[0], 1, &bcc_tmp_0);"),
            "the procedure body should be able to reach the typed record helper:\n{output}"
        );
        // The helper must be defined exactly once -- a naive fix that lets
        // the function-body pre-scan and the real top-level pass each emit
        // their own copy would produce a duplicate-symbol C file.
        assert_eq!(
            output.matches("static int bcc_put_record_header(").count(),
            1,
            "unexpected output:\n{output}"
        );
        assert_eq!(
            output.matches("static int bcc_get_record_header(").count(),
            1,
            "unexpected output:\n{output}"
        );
    }

    #[test]
    fn c_target_supports_random_access_field_get_put_lset() {
        // OPEN FOR RANDOM/FIELD/LSET/PUT/GET round-trip -- the core
        // random-access record I/O shape (see codegen_c.rs's
        // `FileIoLayout`/`emit_get_or_put`/`Statement::Lset` handling).
        let source = "open \"data.dat\" for random as #1 len = 22\n\
                       field #1, 2 as idBuf$, 20 as nameBuf$\n\
                       lset idBuf$ = mki$(7)\n\
                       lset nameBuf$ = \"Alice\"\n\
                       put #1, 1\n\
                       get #1, 1\n\
                       n% = cvi(idBuf$)\n\
                       close #1\n\
                       end\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("static FILE* bcc_files[BCC_MAX_CHANNELS];"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("bcc_files[0] = fopen(\"data.dat\", \"rb+\");"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("if (!bcc_files[0]) bcc_files[0] = fopen(\"data.dat\", \"wb+\");"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("bcc_mki(bv_s_idbuf, 7);"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains(
                "snprintf(bv_s_namebuf, sizeof(bv_s_namebuf), \"%-*.*s\", 20, 20, \"Alice\");"
            ),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("static int bcc_put_record_fields_1_0(")
                && output.contains("memcpy(buffer + 0, field_0, 2);")
                && output.contains("memcpy(buffer + 2, field_1, 20);"),
            "the raw FIELD layout should get one reusable pack helper:\n{output}"
        );
        assert!(
            output.contains("static int bcc_read_record(FILE* file, void* buffer, size_t reclen, long record)")
                && output.contains("static void bcc_write_record(FILE* file, const void* buffer, size_t reclen, long record)"),
            "record positioning and raw I/O should be centralised in shared helpers:\n{output}"
        );
        assert!(
            output
                .contains("bcc_put_record_fields_1_0(bcc_files[0], 1, bv_s_idbuf, bv_s_namebuf);")
                && output.contains(
                    "bcc_get_record_fields_1_0(bcc_files[0], 1, bv_s_idbuf, bv_s_namebuf);"
                ),
            "GET/PUT should delegate to the reusable field-layout helpers:\n{output}"
        );
        assert!(
            !output.contains("fseek(bcc_files[0]"),
            "individual GET/PUT sites should not repeat seek boilerplate:\n{output}"
        );
        assert!(
            output.contains("return fread(buffer, 1, reclen, file) == reclen;")
                && output.contains("fwrite(buffer, 1, reclen, file);"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("bv_i_n = bcc_cvi(bv_s_idbuf);"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("fclose(bcc_files[0]);"),
            "unexpected output:\n{output}"
        );
    }

    #[test]
    fn c_target_record_dsl_uses_one_get_put_helper_pair_per_record_type() {
        let source = r#"record Student
    id: int16
    name: string(20)
    score: float64
    faculty: string(20)
end record

file db as Student = open("students.dat")
db[1] = { id: 7, name: "Ada", score: 95.0, faculty: "Engineering" }
db[1] = ?{ score: 97.5 }
let s = db[1]
end
"#;
        let output = compile_source_via_c_target(source);
        assert_eq!(
            output.matches("static int bcc_put_record_student(").count(),
            1,
            "unexpected output:\n{output}"
        );
        assert_eq!(
            output.matches("static int bcc_get_record_student(").count(),
            1,
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains(
                "bcc_put_record_student(bcc_files[0], 1, &bcc_tmp_0, \"Ada\", &bcc_tmp_1, \"Engineering\");"
            ) && output.contains(
                "bcc_get_record_student(bcc_files[0], 1, bv_s_dbidbuf, bv_s_dbnamebuf, bv_s_dbscorebuf, bv_s_dbfacultybuf);"
            ),
            "PUT should pass typed values straight to the typed helper, with no manual packing \
             in main; GET should keep delegating to the type-specific helper:\n{output}"
        );
        assert!(
            !output.contains("bcc_mki(bv_s_db")
                && !output.contains("bcc_mkd(bv_s_db")
                && !output.contains("snprintf(bv_s_db"),
            "no manual MKx$/pad packing should remain in main for a record/file DSL write -- the \
             typed PUT helper now owns all of that:\n{output}"
        );
        assert!(
            output.contains("bcc_write_record(file, buffer, 50, record);")
                && output.contains("if (!bcc_read_record(file, buffer, 50, record)) return 0;")
                && output.contains(
                    "if (!bcc_put_record_student(bcc_files[0], 1, NULL, NULL, &bcc_tmp_2, NULL))"
                )
                && output.contains("bcc_read_string_field(field_1, buffer + 2, 20);")
                && output.contains("bcc_read_string_field(field_3, buffer + 30, 20);")
                && !output.contains("bcc_read_string_field(field_0")
                && !output.contains("bcc_read_string_field(field_2"),
            "record helpers should delegate positioning and I/O to the shared buffer helpers:\n{output}"
        );
        assert!(
            output.contains(
                "static int bcc_put_record_student(FILE* file, long record, const int16_t* field_0, const char* field_1, const double* field_2, const char* field_3) {"
            ),
            "the typed PUT helper should take native-typed parameters, not packed byte strings:\n{output}"
        );
    }

    #[test]
    fn c_target_partial_record_write_requires_an_existing_record() {
        let source = r#"record Student
    id: int16
    name: string(20)
    score: float64
    faculty: string(20)
end record

file db as Student = open("students.dat")
db[1] = { id: 7, name: "Ada", score: 95.0, faculty: "Engineering" }
db[2] = ?{ name: "Bob" }
end
"#;
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains(
                "bcc_put_record_student(bcc_files[0], 1, &bcc_tmp_0, \"Ada\", &bcc_tmp_1, \"Engineering\");"
            ) && !output.contains(
                "bcc_get_record_student(bcc_files[0], 1, bv_s_dbidbuf, bv_s_dbnamebuf, bv_s_dbscorebuf, bv_s_dbfacultybuf);"
            ),
            "a complete record literal must write directly without a read, passing typed values \
             with no manual packing in main:\n{output}"
        );
        assert!(
            output.contains("if (!bcc_put_record_student(bcc_files[0], 2, NULL, \"Bob\", NULL, NULL))")
                && output.contains("if ((!field_0 || !field_1 || !field_2 || !field_3) && !bcc_read_record(file, buffer, 50, record)) return 0;")
                && output.contains("BASCAL: record %ld does not exist"),
            "a partial record literal must let PUT preserve NULL fields and reject a missing record:\n{output}"
        );
    }

    #[test]
    fn c_target_random_access_helpers_use_ieee754_not_mbf() {
        // Documented divergence from real MBASIC/BASCOM: MKS$/MKD$/CVS/CVD
        // use plain IEEE 754 float/double via memcpy, not real BASIC's
        // Microsoft Binary Format -- see `FILE_IO_HELPER`'s doc comment.
        let source = "open \"d.dat\" for random as #1 len = 8\n\
                       field #1, 8 as scoreBuf$\n\
                       lset scoreBuf$ = mkd$(95.5)\n\
                       put #1, 1\n\
                       get #1, 1\n\
                       s# = cvd(scoreBuf$)\n\
                       close #1\n\
                       end\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains(
                "static void bcc_mkd(char* out, double value) {\n    memcpy(out, &value, 8);"
            ),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("bcc_mkd(bv_s_scorebuf, 95.5);"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("bv_d_s = bcc_cvd(bv_s_scorebuf);"),
            "unexpected output:\n{output}"
        );
    }

    #[test]
    fn c_target_supports_sequential_open_modes() {
        let source = "open \"f.txt\" for input as #1\nclose #1\nopen \"f.txt\" for output as #1\nclose #1\nopen \"f.txt\" for append as #1\nclose #1\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("fopen(\"f.txt\", \"r\")"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("fopen(\"f.txt\", \"w\")"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("fopen(\"f.txt\", \"a\")"),
            "unexpected output:\n{output}"
        );
    }

    #[test]
    fn c_target_round_trips_write_and_input_file() {
        let source = "dim name$, score%\nopen \"scores.csv\" for output as #1\nwrite #1, \"Alice\", 95\nclose #1\nopen \"scores.csv\" for input as #1\nwhile eof(1) = 0\n    input #1, name$, score%\n    print name$; score%\nwend\nclose #1\nend\n";
        let output = compile_source_via_c_target(source);
        // Regression check, same category as RIGHT$/INSTR above: bcc_eof
        // and bcc_read_file_field both need <string.h>/`FILE*` machinery
        // that a text-only check of the call sites wouldn't catch a
        // missing include/helper for.
        assert!(
            output.contains("static int bcc_eof("),
            "EOF needs the SEQ_FILE_HELPER block:\n{output}"
        );
        assert!(
            output.contains("static void bcc_read_file_field("),
            "INPUT # needs the SEQ_FILE_HELPER block:\n{output}"
        );
        assert!(
            output.contains("bcc_eof(bcc_files[0])"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("fprintf(bcc_files[0], \"\\\"%s\\\",%d\\n\", \"Alice\", 95)"),
            "unexpected output:\n{output}"
        );
    }

    #[test]
    fn c_target_supports_line_input_file_and_print_file() {
        let source = "dim line$\nopen \"f.txt\" for output as #1\nprint #1, \"hi\"\nclose #1\nopen \"f.txt\" for input as #1\nline input #1, line$\nclose #1\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("static void bcc_line_input_file("),
            "LINE INPUT # needs the SEQ_FILE_HELPER block:\n{output}"
        );
        assert!(
            output.contains("bcc_line_input_file(bcc_files[0], bv_s_line, sizeof(bv_s_line));"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("fprintf(bcc_files[0], \"hi\\n\")"),
            "unexpected output:\n{output}"
        );
    }

    #[test]
    fn c_target_omits_seq_file_helper_when_sequential_io_is_never_used() {
        let source = "print \"hi\"\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            !output.contains("bcc_eof(") && !output.contains("bcc_read_file_field("),
            "the sequential file I/O helper shouldn't be emitted when it's never used:\n{output}"
        );
    }

    #[test]
    fn c_target_rejects_lset_on_a_variable_that_was_never_fielded() {
        let source = "x$ = \"hi\"\nlset x$ = \"bye\"\nend\n";
        let diagnostics = compile_source_via_c_target_err(source);
        assert!(
            diagnostics
                .iter()
                .any(|d| d.message.contains("only a variable declared by a")),
            "unexpected diagnostics: {diagnostics:?}"
        );
    }

    #[test]
    fn c_target_rejects_get_put_with_no_record_number() {
        let source = "open \"f.dat\" for random as #1 len = 4\n\
                       field #1, 4 as b$\n\
                       get #1\n\
                       end\n";
        let diagnostics = compile_source_via_c_target_err(source);
        assert!(
            diagnostics
                .iter()
                .any(|d| d.message.contains("no record number")),
            "unexpected diagnostics: {diagnostics:?}"
        );
    }

    #[test]
    fn c_target_random_access_channel_1_indexes_bcc_files_safely_in_bounds() {
        // Regression test: `bcc_files` used to be declared
        // `[BCC_MAX_CHANNELS]` (32 elements, valid indices 0..=31) while
        // channel numbers -- 1-based, like real BASIC's own `#1`/`#2`/...
        // -- were used directly as the index, making channel
        // `BCC_MAX_CHANNELS` itself (32) index one past the end. Now every
        // `bcc_files[...]` reference is `channel - 1`, so the array stays
        // `[BCC_MAX_CHANNELS]` and channel `BCC_MAX_CHANNELS` lands on the
        // last valid index instead of one past the end.
        let source = "open \"f.dat\" for random as #32 len = 4\n\
                       field #32, 4 as b$\n\
                       close #32\n\
                       end\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("static FILE* bcc_files[BCC_MAX_CHANNELS];"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("bcc_files[31]"),
            "channel #32 should index bcc_files[31], the last valid slot:\n{output}"
        );
    }

    #[test]
    fn c_target_rejects_a_file_channel_number_out_of_range() {
        let source = "open \"f.dat\" for random as #99 len = 4\nfield #99, 4 as b$\nend\n";
        let diagnostics = compile_source_via_c_target_err(source);
        assert!(
            diagnostics
                .iter()
                .any(|d| d.message.contains("out of range")),
            "unexpected diagnostics: {diagnostics:?}"
        );

        let source = "open \"f.dat\" for random as #0 len = 4\nfield #0, 4 as b$\nend\n";
        let diagnostics = compile_source_via_c_target_err(source);
        assert!(
            diagnostics
                .iter()
                .any(|d| d.message.contains("out of range")),
            "channel 0 should be rejected too (BASIC channels are 1-based): {diagnostics:?}"
        );
    }

    #[test]
    fn c_target_supports_str_dollar_with_the_leading_space_convention() {
        // Real MBASIC/BASCOM's STR$ prefixes a space for non-negative
        // numbers (standing in for the sign) -- C's printf `%` (space)
        // flag gives this natively, no manual sign handling needed.
        let source = "n% = 5\nprint str$(n%)\nend\n";
        let output = compile_source_via_c_target(source);
        assert!(
            output.contains("bcc_stri(bv_i_n)"),
            "unexpected output:\n{output}"
        );
        assert!(
            output.contains("snprintf(out, 256, \"% d\", value);"),
            "unexpected output:\n{output}"
        );
    }

    /// Helper for the C-backend tests above: writes `source` (with a
    /// `program` header prepended) to a temp file and compiles it under
    /// `Target::C`, panicking with the diagnostics on failure. Returns the
    /// app file and its paired runtime file (see
    /// `compile_file_with_runtime`) concatenated back together (runtime
    /// first, matching where the helpers used to sit in the single
    /// pre-split file) -- most of the tests below only care whether some
    /// snippet of generated text is present/absent *somewhere*, and don't
    /// need to distinguish which of the two files it landed in. A few
    /// tests that specifically check the split itself use
    /// `compile_source_via_c_target_split` instead.
    fn compile_source_via_c_target(source: &str) -> String {
        let (app, runtime) = compile_source_via_c_target_split(source);
        format!("{}{app}", runtime.unwrap_or_default())
    }

    /// Same as `compile_source_via_c_target`, but keeps the app file and
    /// its (optional) paired runtime file separate -- for tests that
    /// check specifically which of the two a piece of generated text
    /// landed in.
    fn compile_source_via_c_target_split(source: &str) -> (String, Option<String>) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("numeric_print.bcl");
        std::fs::write(&path, format!("program p\n{source}")).unwrap();
        let options = CompileOptions {
            target: Target::C,
            ..CompileOptions::new()
        };
        compile_file_with_runtime(&path, &options).unwrap_or_else(|diagnostics| {
            panic!(
                "should compile: {}",
                diagnostics
                    .into_iter()
                    .map(|d| d.to_string())
                    .collect::<String>()
            )
        })
    }

    /// Same as `compile_source_via_c_target`, but for a negative test:
    /// returns the diagnostics from a compile that's expected to fail,
    /// panicking if it unexpectedly succeeds instead.
    fn compile_source_via_c_target_err(source: &str) -> Vec<Diagnostic> {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("numeric_print.bcl");
        std::fs::write(&path, format!("program p\n{source}")).unwrap();
        let options = CompileOptions {
            target: Target::C,
            ..CompileOptions::new()
        };
        compile_file(&path, &options).expect_err("should not compile")
    }
}
