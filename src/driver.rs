//! Compiler driver: source/file entry points, `CompileOptions`, and the
//! `require` / `shared`-file resolution that pulls a multi-file BASCAL
//! program (and the `com/` standard library) together before the
//! parse -> lower -> resolve -> codegen pipeline runs.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::codegen::{self, CodeGenerator};
use crate::diagnostics::{self, Diagnostic};
use crate::lexer::{self, Lexer, TokenKind};
use crate::parser::Parser;
use crate::{ast, codegen_c, codegen_jvm, lower, resolver};

pub use crate::codegen::Target;

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
    let lower::Lowered {
        program,
        synthesized_buffer_names,
    } = lower::lower(program)?;
    let resolved = resolver::resolve(program)?;
    print_legacy_form_warnings(&resolved.program);
    print_const_convention_warnings(&resolved.program);
    let conflicts = codegen::check_generated_name_conflicts(&resolved.program);
    if !conflicts.is_empty() {
        return Err(conflicts);
    }
    CodeGenerator::new()
        .with_synthesized_buffer_names(synthesized_buffer_names)
        .generate(&resolved)
}

/// The most common case: just the primary generated file (the whole
/// `.bas` for `Target::Basic`, or the single self-contained `.c` for
/// `Target::C` -- see `codegen_c::GeneratedC`'s own doc comment for why
/// that `.c` needs no paired file alongside it).
pub fn compile_file(input: &Path, options: &CompileOptions) -> Result<String, Vec<Diagnostic>> {
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

    let lower::Lowered {
        program,
        synthesized_buffer_names,
    } = lower::lower(program)?;
    let resolved = resolver::resolve(program)?;
    print_legacy_form_warnings(&resolved.program);
    print_const_convention_warnings(&resolved.program);
    match options.target {
        Target::Basic => {
            let basic = CodeGenerator::new()
                .with_line_numbers(options.line_numbers)
                .with_synthesized_buffer_names(synthesized_buffer_names)
                .generate(&resolved)?;
            Ok(basic)
        }
        Target::C => {
            let generated = codegen_c::generate(&resolved.program)?;
            Ok(generated.app)
        }
        Target::Jvm => codegen_jvm::generate(&resolved.program),
    }
}

/// Parses the root source file and every transitively required library, but
/// deliberately stops before record lowering, name/type resolution, and any
/// backend generation. This is useful while a program uses accepted planned
/// syntax whose typed IR or backend support is still under development.
pub fn check_file(input: &Path, options: &CompileOptions) -> Result<(), Vec<Diagnostic>> {
    let mut options = options.clone();
    if let Some(parent) = input.parent() {
        let parent = parent.to_path_buf();
        if !options.library_dirs.contains(&parent) {
            options.library_dirs.insert(0, parent);
        }
    }

    let mut visited = HashSet::new();
    let program = load_program_recursive(input, true, &options, &mut visited)?;
    if let Some(shared_name) = program
        .program_decl
        .as_ref()
        .and_then(|d| d.shared.as_deref())
    {
        if let Some(shared_path) = resolve_shared_path(shared_name, input, &options) {
            load_shared_file(&shared_path, shared_name)?;
        }
    }
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

fn print_const_convention_warnings(program: &ast::Program) {
    for finding in resolver::check_const_conventions(program) {
        eprintln!("{finding}");
    }
}

pub fn default_output_path(input: &Path, target: Target) -> std::path::PathBuf {
    let extension = match target {
        Target::Basic => "bas",
        Target::C => "c",
        Target::Jvm => "j",
    };
    input.with_extension(extension)
}

pub(crate) fn parse_source(
    filename: String,
    source: &str,
) -> Result<ast::Program, Vec<Diagnostic>> {
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
pub(crate) fn reject_underscored_identifiers(
    tokens: &[lexer::Token],
) -> Result<(), Vec<Diagnostic>> {
    let diagnostics: Vec<Diagnostic> = tokens
        .iter()
        .enumerate()
        .filter_map(|(_index, token)| match &token.kind {
            // Constants are compile-time names and may use the documented
            // uppercase-snake convention; generated BASIC never reads the
            // source spelling as an identifier.
            TokenKind::Ident(name) if name.contains('_') && !is_upper_snake_identifier(name) => {
                Some(Diagnostic::error(
                    token.pos.clone(),
                    format!(
                        "identifier `{name}` contains an underscore, which real MBASIC/BASCOM \
                     rejects as a syntax error wherever the name is read (not just assigned) -- \
                     use camelCase instead (e.g. `{}`)",
                        to_suggested_camel_case(name)
                    ),
                ))
            }
            _ => None,
        })
        .collect();
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

pub(crate) fn is_upper_snake_identifier(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|ch| ch == '_' || ch.is_ascii_uppercase() || ch.is_ascii_digit())
}

pub(crate) fn to_suggested_camel_case(name: &str) -> String {
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

pub(crate) fn load_program_recursive(
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
            typed_arrays: Vec::new(),
            typed_array_refs: Vec::new(),
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
        typed_arrays: program.typed_arrays.clone(),
        typed_array_refs: program.typed_array_refs.clone(),
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

pub(crate) fn load_shared_file(
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

pub(crate) fn resolve_shared_path(
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

pub(crate) fn resolve_required_symbol(
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

pub(crate) fn required_symbol_to_path(raw: &str) -> PathBuf {
    let mut path = raw.split('.').collect::<PathBuf>();
    path.set_extension("bcl");
    path
}

pub(crate) fn search_roots(source_file: &Path, options: &CompileOptions) -> Vec<PathBuf> {
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
pub(crate) fn stdlib_search_roots() -> Vec<PathBuf> {
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

pub(crate) fn normalize_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}
