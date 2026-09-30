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
use crate::{ast, codegen_c, codegen_jvm, lower, record_transpile, resolver, semantic_ir};

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
    /// Runs `resolver::check_unused_declarations`/`check_shadowing`/
    /// `check_unreachable_code`/`check_magic_numbers`/
    /// `check_const_conventions` against the root program's own parse (same
    /// reasoning as `strict_vars`: never a `require`d library's own
    /// internals, never the DSL-lowered form) and prints every finding to
    /// stderr as a warning. Opt-in and off by default, unlike
    /// `check_legacy_forms` (always on): these five are more heuristic and
    /// more likely to flag something on existing, working code that isn't
    /// actually worth fixing right now, especially a program ported from
    /// real BASIC.
    pub lint: bool,
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
            lint: false,
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
    let program = parse_source(filename.clone(), source)?;
    let lower::Lowered {
        program,
        synthesized_buffer_names,
        lowered_record_files,
    } = lower::lower(program)?;
    let mut semantic_module = semantic_ir::parse_and_adapt_named(filename.clone(), source)
        .map_err(|error| vec![semantic_ir::parse_diagnostic(filename, &error)])?;
    semantic_module.lowered_record_files = lowered_record_files;
    // `compile_source` always generates BASIC.
    record_transpile::transpile(&mut semantic_module);
    let resolved = resolver::resolve_with_semantic(program, Some(semantic_module))?;
    let module = resolved
        .semantic_module
        .as_ref()
        .expect("compile_source requires generated semantic IR");
    print_semantic_legacy_form_warnings(module);
    let conflicts =
        codegen::check_generated_name_conflicts_semantic(module, &resolved.common_blocks);
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
    compile_file_impl(input, options, AstUse::TypedOnly)
}

/// How much of the resolved AST code generation may read.
#[derive(Clone, Copy, PartialEq, Eq)]
enum AstUse {
    /// The legacy AST-driven generation, kept as the baseline the
    /// AST-independence tests compare against.
    AstOnly,
    /// The typed module alone, with the resolved AST emptied. This is what
    /// `compile_file` uses: code generation reads only the typed IR.
    TypedOnly,
}

/// `compile_file` with an explicit choice of how much AST generation may
/// read; the tests use it to measure how independent generation is of the
/// legacy AST.
fn compile_file_impl(
    input: &Path,
    options: &CompileOptions,
    ast_use: AstUse,
) -> Result<String, Vec<Diagnostic>> {
    let mut options = options.clone();
    if let Some(parent) = input.parent() {
        let parent = parent.to_path_buf();
        if !options.library_dirs.contains(&parent) {
            options.library_dirs.insert(0, parent);
        }
    }
    let options = &options;

    if options.lint {
        // Same reasoning as --strict-vars above: checked against the root
        // file's own parse, not a required library's own internals, and
        // not the DSL-lowered form.
        let source = fs::read_to_string(input).map_err(|err| {
            vec![Diagnostic::error(
                diagnostics::SourcePos::new(input.display().to_string(), 1, 1),
                format!("failed to read source file: {err}"),
            )]
        })?;
        let root_only = parse_source(input.display().to_string(), &source)?;
        for finding in resolver::check_unused_declarations(&root_only) {
            eprintln!("{finding}");
        }
        for finding in resolver::check_shadowing(&root_only) {
            eprintln!("{finding}");
        }
        for finding in resolver::check_unreachable_code(&root_only) {
            eprintln!("{finding}");
        }
        for finding in resolver::check_magic_numbers(&root_only) {
            eprintln!("{finding}");
        }
        for finding in resolver::check_const_conventions(&root_only) {
            eprintln!("{finding}");
        }
    }

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

    // Load generated semantic metadata before resolving the root's shared
    // header so its typed declaration is authoritative for COMMON lookup.
    // The ordinary file path requires semantic parsing to succeed; codegen
    // backends receive the resolved typed IR with AST source locations and
    // compatibility metadata.
    let mut semantic_warnings = Vec::new();
    let semantic_module = load_semantic_module_recursive(
        input,
        true,
        options,
        &mut HashSet::new(),
        &mut semantic_warnings,
    )?;

    // Resolve the shared COMMON block if the program declares one.
    if let Some(shared_name) = program_shared_name(&program, Some(&semantic_module)) {
        if let Some(shared_path) = resolve_shared_path(&shared_name, input, options) {
            program.common = load_shared_file(&shared_path, &shared_name)?;
        }
        // Shared file not found → compile without COMMON (silent; it may not exist yet).
    }

    let lower::Lowered {
        program,
        synthesized_buffer_names,
        lowered_record_files,
    } = lower::lower(program)?;
    // Keep generated semantic IR attached throughout ordinary file
    // compilation so each codegen backend receives the resolved typed module
    // used for shared-header and dependency analysis.
    let semantic_module = Some({
        let mut module = semantic_module;
        module.lowered_record_files = lowered_record_files;
        // Every backend consumes the expanded primitives.
        if matches!(options.target, Target::Basic | Target::Fbc | Target::Jvm | Target::C) {
            record_transpile::transpile(&mut module);
        }
        if options.target == Target::C {
            // C storage needs a concrete type for every variable.
            module.bare_names_are_single = true;
            module.annotate_types();
        }
        module
    });
    let resolved = resolver::resolve_with_semantic(program, semantic_module)?;
    for finding in semantic_warnings {
        eprintln!("{finding}");
    }
    // The BASIC targets pass an unrecognized function name through to the
    // BASIC dialect; C and the JVM cannot emit it, so say so up front.
    if matches!(options.target, Target::C | Target::C64 | Target::Jvm) {
        if let Some(module) = resolved.semantic_module.as_ref() {
            let unknown = resolver::reject_unknown_calls(module);
            if !unknown.is_empty() {
                return Err(unknown);
            }
        }
    }
    if matches!(options.target, Target::Basic | Target::Fbc) {
        let conflicts = if let Some(module) = resolved.semantic_module.as_ref() {
            codegen::check_generated_name_conflicts_semantic(module, &resolved.common_blocks)
        } else {
            codegen::check_generated_name_conflicts(&resolved.program)
        };
        if !conflicts.is_empty() {
            return Err(conflicts);
        }
    }
    let generate = |resolved: &resolver::ResolvedProgram| {
        generate_for_target(resolved, options, &synthesized_buffer_names)
    };
    if ast_use == AstUse::AstOnly {
        return generate(&resolved);
    }
    // Code generation reads only the typed module: hand it a program whose
    // AST statement and function bodies are empty.
    let mut typed_only = resolved;
    typed_only.program.statements.clear();
    for function in &mut typed_only.program.functions {
        function.body.clear();
    }
    generate(&typed_only)
}

fn generate_for_target(
    resolved: &resolver::ResolvedProgram,
    options: &CompileOptions,
    synthesized_buffer_names: &HashSet<String>,
) -> Result<String, Vec<Diagnostic>> {
    match options.target {
        Target::Basic => {
            let basic = CodeGenerator::new()
                .with_line_numbers(options.line_numbers)
                .with_synthesized_buffer_names(synthesized_buffer_names.clone())
                .generate(resolved)?;
            Ok(basic)
        }
        Target::Fbc => {
            let module = resolved
                .semantic_module
                .as_ref()
                .expect("compile_file requires generated semantic IR");
            reject_semantic_fbc_incompatible_constructs(module)?;
            let basic = CodeGenerator::new()
                .with_line_numbers(options.line_numbers)
                .with_synthesized_buffer_names(synthesized_buffer_names.clone())
                .generate(&resolved)?;
            Ok(basic)
        }
        Target::C => {
            let generated = codegen_c::generate(&resolved, Target::C)?;
            Ok(generated.app)
        }
        Target::Jvm => codegen_jvm::generate(&resolved),
        // Phase 4 of RETRO_BASIC_SUPPORT_PROMPT.md: reuses `codegen_c.rs`
        // exactly like `Target::C` above, just under the C64/`cc65`
        // `CDialectProfile` -- Phase 3's `validate_capabilities` (run
        // first, inside `codegen_c::generate`) rejects whatever that
        // profile can't express (`/`, `^`, `single`/`double` variables,
        // `byval` array parameters) before any C is emitted, so what
        // reaches here is always C `cl65` can actually compile.
        // `main.rs`'s `--binary`/`-b` is what invokes `cl65` itself,
        // parallel to `Target::Fbc`/`Target::C`'s own `fbc`/`gcc`
        // invocations -- this function only ever produces the
        // intermediate C text, never a binary, for any target.
        Target::C64 => {
            let generated = codegen_c::generate(&resolved, Target::C64)?;
            Ok(generated.app)
        }
    }
}

fn semantic_source_position(
    source: &crate::semantic_ir::SemanticSource,
    span: crate::rdgen_frontend::SourceSpan,
) -> diagnostics::SourcePos {
    source.source_position(span)
}

fn reject_semantic_fbc_incompatible_constructs(
    module: &crate::semantic_ir::SemanticModule,
) -> Result<(), Vec<Diagnostic>> {
    use crate::semantic_ir::{SemanticStatement, SemanticStatementKind as Kind};

    fn visit(
        statements: &[SemanticStatement],
        source: &crate::semantic_ir::SemanticSource,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        for statement in statements {
            match &statement.kind {
                Kind::Try {
                    body,
                    catch,
                    finally_body,
                } => {
                    diagnostics.push(Diagnostic::error(
                        semantic_source_position(source, statement.span),
                        "`try`/`catch` is permanently unsupported with --target fbc; fbc's \
                         `RESUME`/`RESUME NEXT` cannot resume at an arbitrary later line the \
                         way `RESUME <lineno>` does under real BASCOM (verified: fbc's parser \
                         rejects `RESUME <lineno>`/`RESUME <label>` outright under every `-lang` \
                         dialect, and the obvious GOSUB-plus-`RESUME NEXT` workaround crashes at \
                         runtime with fbc's own \"illegal resume\" error -- see GitHub issue #153 \
                         for the full investigation). Use `--target basic` (verified against real \
                         BASCOM) or `on error goto`/`resume` for a program that must build under fbc."
                            .to_string(),
                    ));
                    visit(body, source, diagnostics);
                    if let Some(catch) = catch {
                        visit(&catch.body, source, diagnostics);
                    }
                    visit(finally_body, source, diagnostics);
                }
                Kind::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    visit(then_body, source, diagnostics);
                    visit(else_body, source, diagnostics);
                }
                Kind::For { body, .. }
                | Kind::While { body, .. }
                | Kind::Do { body, .. }
                | Kind::Line(body) => visit(body, source, diagnostics),
                Kind::SelectCase {
                    cases, else_body, ..
                } => {
                    for case in cases {
                        visit(&case.body, source, diagnostics);
                    }
                    visit(else_body, source, diagnostics);
                }
                _ => {}
            }
        }
    }

    let mut diagnostics = Vec::new();
    for (index, statement) in module.statements.iter().enumerate() {
        let source = module
            .statement_sources
            .get(index)
            .and_then(|source_index| module.sources.get(*source_index))
            .or_else(|| module.sources.first());
        if let Some(source) = source {
            visit(std::slice::from_ref(statement), source, &mut diagnostics);
        }
    }
    for callable in &module.callables {
        let source = module
            .sources
            .get(callable.source_index)
            .or_else(|| module.sources.first());
        if let Some(source) = source {
            visit(&callable.body, source, &mut diagnostics);
        }
    }
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

/// Resolve the shared-module declaration from typed IR when available. This
/// keeps COMMON lookup aligned with the same semantic header used by the
/// driver and backends; compatibility callers may still provide AST alone.
fn program_shared_name(
    program: &ast::Program,
    semantic_module: Option<&semantic_ir::SemanticModule>,
) -> Option<String> {
    if let Some(module) = semantic_module {
        return match &module.header {
            Some(semantic_ir::ModuleHeader::Program { shared, .. }) => shared.clone(),
            _ => None,
        };
    }
    program
        .program_decl
        .as_ref()
        .and_then(|declaration| declaration.shared.clone())
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
    let semantic_module = fs::read_to_string(input).ok().and_then(|source| {
        semantic_ir::parse_and_adapt_named(input.display().to_string(), &source).ok()
    });
    if let Some(shared_name) = program_shared_name(&program, semantic_module.as_ref()) {
        if let Some(shared_path) = resolve_shared_path(&shared_name, input, &options) {
            load_shared_file(&shared_path, &shared_name)?;
        }
    }
    Ok(())
}

fn print_semantic_legacy_form_warnings(module: &semantic_ir::SemanticModule) {
    for finding in module.legacy_form_diagnostics() {
        eprintln!("{finding}");
    }
}

pub fn default_output_path(input: &Path, target: Target) -> std::path::PathBuf {
    let extension = match target {
        Target::Basic | Target::Fbc => "bas",
        // `Target::C64`'s primary `transpile` output is C text too --
        // `codegen_c::generate` under the C64/`cc65` `CDialectProfile`,
        // exactly like `Target::C` (see `transpile`'s own `Target::C64`
        // arm). The loadable `PRG` image is a separate, later artifact
        // `--binary`'s `cl65` invocation produces in `tmp/`, the same way
        // `Target::Basic`'s DOS `.EXE`/`Target::Fbc`'s native binary
        // aren't this function's concern either -- an earlier version of
        // this comment reserved `prg` here on the (wrong) assumption that
        // C64's primary output would be the binary itself.
        Target::C | Target::C64 => "c",
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

/// Load generated semantic modules in the same dependency order as the
/// legacy AST loader.  Semantic backend facts must include required library
/// declarations; adapting only the root file would make callable and record
/// lookup depend on whether the caller used `compile_source` or `compile_file`.
fn load_semantic_module_recursive(
    input: &Path,
    _is_root: bool,
    options: &CompileOptions,
    visited: &mut HashSet<PathBuf>,
    warnings: &mut Vec<Diagnostic>,
) -> Result<semantic_ir::SemanticModule, Vec<Diagnostic>> {
    let input = normalize_path(input);
    if !visited.insert(input.clone()) {
        return Ok(semantic_ir::SemanticModule {
            span: crate::rdgen_frontend::SourceSpan { start: 0, end: 0 },
            sources: Vec::new(),
            header: None,
            dependencies: Vec::new(),
            records: Vec::new(),
            lowered_record_files: Vec::new(),
            callables: Vec::new(),
            statements: Vec::new(),
            statement_sources: Vec::new(),
            records_transpiled: false,
            bare_names_are_single: false,
        });
    }
    let source = fs::read_to_string(&input).map_err(|error| {
        vec![Diagnostic::error(
            diagnostics::SourcePos::new(input.display().to_string(), 1, 1),
            format!("failed to read source file: {error}"),
        )]
    })?;
    let mut module = semantic_ir::parse_and_adapt_named(input.display().to_string(), &source)
        .map_err(|error| vec![semantic_ir::parse_diagnostic(input.display().to_string(), &error)])?;
    warnings.extend(module.legacy_form_diagnostics());
    let dependencies = module.dependencies.clone();
    // Load in declaration order so a library shared by two siblings lands
    // where the first of them reaches it, as the legacy loader has it, then
    // prepend backwards to retain the left-to-right sibling order.
    let mut loaded = Vec::with_capacity(dependencies.len());
    for dependency in &dependencies {
        let path = resolve_required_symbol(&dependency.path, &input, options)?;
        loaded.push(load_semantic_module_recursive(
            &path, false, options, visited, warnings,
        )?);
    }
    for dependency in loaded.into_iter().rev() {
        module.prepend_dependency(dependency);
    }
    for dependency in &mut module.dependencies {
        dependency.resolved = true;
    }
    Ok(module)
}

#[cfg(test)]
mod semantic_driver_differential_tests {
    use super::*;

    #[test]
    fn shared_header_lookup_prefers_semantic_program_header() {
        let ast = parse_source(
            "shared_header_ast.bcl".to_string(),
            "program astProgram shared astCommon\nend\n",
        )
        .expect("AST header parses");
        let semantic = semantic_ir::parse_and_adapt_named(
            "shared_header_typed.bcl",
            "program typedProgram shared typedCommon\nend\n",
        )
        .expect("semantic header parses");

        assert_eq!(
            program_shared_name(&ast, Some(&semantic)),
            Some("typedCommon".to_string())
        );
        assert_eq!(
            program_shared_name(&ast, None),
            Some("astCommon".to_string())
        );

        let semantic_without_shared = semantic_ir::parse_and_adapt_named(
            "shared_header_typed.bcl",
            "program typedProgram\nend\n",
        )
        .expect("semantic header without COMMON parses");
        assert_eq!(
            program_shared_name(&ast, Some(&semantic_without_shared)),
            None
        );
    }

    #[test]
    fn required_library_callable_uses_typed_return_call_across_backends() {
        let directory = tempfile::tempdir().expect("temporary project directory");
        let library_dir = directory.path().join("com/example");
        fs::create_dir_all(&library_dir).expect("library directory");
        fs::write(
            library_dir.join("base.bcl"),
            "library com.example.base\nfunction identity%(value%)\nreturn value%\nend function\n",
        )
        .expect("base library source");
        fs::write(
            library_dir.join("math.bcl"),
            "library com.example.math\nrequire com.example.base\nfunction twice%(value%)\nreturn identity%(value%) * 2\nend function\n",
        )
        .expect("library source");
        let root = directory.path().join("main.bcl");
        fs::write(
            &root,
            "program main\nrequire com.example.math\nresult% = twice%(3)\nprint result%\nend\n",
        )
        .expect("root source");
        let options = CompileOptions {
            library_dirs: vec![directory.path().to_path_buf()],
            ..CompileOptions::new()
        };

        let mut ast_visited = HashSet::new();
        let ast = load_program_recursive(&root, true, &options, &mut ast_visited)
            .expect("load legacy dependency graph");
        let lower::Lowered {
            program,
            synthesized_buffer_names,
            lowered_record_files,
        } = lower::lower(ast).expect("lower dependency graph");
        let mut semantic_warnings = Vec::new();
        let semantic = load_semantic_module_recursive(
            &root,
            true,
            &options,
            &mut HashSet::new(),
            &mut semantic_warnings,
        )
        .expect("load generated semantic dependency graph");
        let mut semantic = semantic;
        assert!(semantic
            .dependencies
            .iter()
            .all(|dependency| dependency.resolved));
        semantic.lowered_record_files = lowered_record_files;
        assert!(semantic_warnings.is_empty());

        fn output(
            program: ast::Program,
            semantic: Option<semantic_ir::SemanticModule>,
            buffers: &std::collections::HashSet<String>,
            target: Target,
        ) -> String {
            let resolved = resolver::resolve_with_semantic(program, semantic)
                .expect("resolve differential program");
            match target {
                Target::Basic => crate::codegen::CodeGenerator::new()
                    .with_synthesized_buffer_names(buffers.clone())
                    .generate(&resolved)
                    .expect("transpile BASIC"),
                Target::C => {
                    crate::codegen_c::generate(&resolved, Target::C)
                        .expect("transpile C")
                        .app
                }
                Target::Jvm => crate::codegen_jvm::generate(&resolved).expect("transpile JVM"),
                Target::Fbc | Target::C64 => unreachable!(),
            }
        }

        for target in [Target::Basic, Target::C, Target::Jvm] {
            let semantic_output = output(
                program.clone(),
                Some(semantic.clone()),
                &synthesized_buffer_names,
                target,
            );
            let legacy_output = output(program.clone(), None, &synthesized_buffer_names, target);
            if target == Target::Basic {
                assert!(
                    semantic_output.contains("GOSUB 10\n    BCCT1% = identityResult0%\n    twiceResult0% = BCCT1% * 2"),
                    "typed BASIC return call must execute and capture the required-library result: {semantic_output}"
                );
                assert_eq!(
                    semantic_output.contains("BCCT1% = identityResult0%"),
                    !legacy_output.contains("BCCT1% = identityResult0%"),
                    "typed BASIC return call must capture its call result before arithmetic"
                );
            } else {
                assert_eq!(semantic_output, legacy_output, "target: {target:?}");
            }
        }
    }

    #[test]
    fn callable_global_name_migration_matches_typed_source_across_backends() {
        let filename = "driver_callable_global.bcl";
        let ast_source = "function choose%(flag%)\nif flag% = 1 then\nglobal legacy%\nlegacy% = 3\nreturn legacy%\nend if\nreturn 0\nend function\nprint choose%(1)\nend\n";
        let semantic_source = "function choose%(flag%)\nif flag% = 1 then\nglobal canonical%\ncanonical% = 42\nreturn canonical%\nend if\nreturn 0\nend function\nprint choose%(1)\nend\n";
        let parsed = parse_source(filename.to_string(), ast_source).unwrap();
        let lower::Lowered {
            program,
            synthesized_buffer_names,
            ..
        } = lower::lower(parsed).unwrap();
        let module = semantic_ir::parse_and_adapt_named(filename, semantic_source).unwrap();
        let resolved = resolver::resolve_with_semantic(program, Some(module)).unwrap();

        let basic = crate::codegen::CodeGenerator::new()
            .with_synthesized_buffer_names(synthesized_buffer_names.clone())
            .generate(&resolved)
            .expect("transpile typed BASIC");
        let c = crate::codegen_c::generate(&resolved, Target::C)
            .expect("transpile typed C")
            .app;
        let jvm = crate::codegen_jvm::generate(&resolved).expect("transpile typed JVM");
        let semantic_ast = parse_source(filename.to_string(), semantic_source)
            .expect("typed source also parses through compatibility AST");
        let lower::Lowered {
            program: semantic_ast,
            synthesized_buffer_names: semantic_ast_buffers,
            ..
        } = lower::lower(semantic_ast).expect("lower typed-source AST");
        let compatibility = resolver::resolve(semantic_ast).expect("resolve typed-source AST");
        assert!(compatibility
            .function_global_declarations
            .values()
            .flatten()
            .any(|ident| ident.name.eq_ignore_ascii_case("canonical")));
        let legacy_basic = crate::codegen::CodeGenerator::new()
            .with_synthesized_buffer_names(semantic_ast_buffers)
            .generate(&compatibility)
            .expect("transpile compatibility BASIC");
        let legacy_c = crate::codegen_c::generate(&compatibility, Target::C)
            .expect("transpile compatibility C")
            .app;
        let legacy_jvm =
            crate::codegen_jvm::generate(&compatibility).expect("transpile compatibility JVM");
        assert_eq!(basic, legacy_basic, "BASIC semantic/compatibility output");
        assert_eq!(c, legacy_c, "C semantic/compatibility output");
        assert_ne!(
            jvm, legacy_jvm,
            "semantic and compatibility label layouts differ"
        );
        assert!(
            jvm.contains(".field public static g1 I") && jvm.contains("putstatic Program/g1 I")
        );
        assert!(
            legacy_jvm.contains(".field public static g1 I")
                && legacy_jvm.contains("putstatic Program/g1 I")
                && !legacy_jvm.contains("istore 1")
        );
        for output in [&basic, &c] {
            let normalized = output.to_ascii_lowercase();
            assert!(
                normalized.contains("canonical") && !normalized.contains("legacy"),
                "backend output retained the AST global name:\n{output}"
            );
        }
        assert!(
            jvm.contains("ldc 42") && !jvm.contains("ldc 3"),
            "JVM bytecode did not retain semantic assignment value:\n{jvm}"
        );
    }

    #[test]
    fn semantic_array_bounds_match_driver_output_across_backends() {
        let filename = "driver_semantic_array_bounds.bcl";
        let ast_source = "dim values%(1)\nvalues%(0) = 2\nprint values%(0)\nend\n";
        let semantic_source = "dim values%(4)\nvalues%(4) = 7\nprint values%(4)\nend\n";
        let parsed = parse_source(filename.to_string(), ast_source).unwrap();
        let lower::Lowered {
            program,
            synthesized_buffer_names,
            ..
        } = lower::lower(parsed).unwrap();
        let module = semantic_ir::parse_and_adapt_named(filename, semantic_source).unwrap();
        let typed = resolver::resolve_with_semantic(program, Some(module)).unwrap();

        let semantic_ast = parse_source(filename.to_string(), semantic_source).unwrap();
        let lower::Lowered {
            program: semantic_ast,
            synthesized_buffer_names: semantic_ast_buffers,
            ..
        } = lower::lower(semantic_ast).unwrap();
        let compatibility = resolver::resolve(semantic_ast).unwrap();

        fn outputs(resolved: &resolver::ResolvedProgram, buffers: &HashSet<String>) -> [String; 3] {
            [
                crate::codegen::CodeGenerator::new()
                    .with_synthesized_buffer_names(buffers.clone())
                    .generate(resolved)
                    .expect("transpile BASIC"),
                crate::codegen_c::generate(resolved, Target::C)
                    .expect("transpile C")
                    .app,
                crate::codegen_jvm::generate(resolved).expect("transpile JVM"),
            ]
        }
        let typed_output = outputs(&typed, &synthesized_buffer_names);
        let compatibility_output = outputs(&compatibility, &semantic_ast_buffers);
        for (backend, output) in ["BASIC", "C", "JVM"].into_iter().zip(&typed_output) {
            let normalized = output.to_ascii_lowercase();
            assert!(
                normalized.contains("7") && !normalized.contains("= 2"),
                "{backend} output did not use the typed assignment: {output}"
            );
        }
        assert!(typed_output[0].contains("values%(4)"));
        assert!(
            typed_output[1].contains("bv_i_values[5]") && typed_output[1].contains("[(4)] = 7")
        );
        assert!(typed_output[2].contains("ldc 7") && !typed_output[2].contains("ldc 2"));
        assert_eq!(
            typed_output, compatibility_output,
            "typed and compatibility backends"
        );
    }

    #[test]
    fn compile_source_transpiles_typed_module_statement_operands() {
        for (filename, statement, expected) in [
            (
                "driver_typed_locate_calls.bcl",
                "locate tick%(), tick%()",
                "LOCATE BCCT1%, BCCT2%",
            ),
            (
                "driver_typed_color_calls.bcl",
                "color tick%(), tick%()",
                "COLOR BCCT1%, BCCT2%",
            ),
            (
                "driver_typed_width_calls.bcl",
                "width #tick%(), tick%()",
                "WIDTH #BCCT1%, BCCT2%",
            ),
            (
                "driver_typed_poke_out_calls.bcl",
                "poke tick%(), tick%()\nout tick%(), tick%()",
                "OUT BCCT3%, BCCT4%",
            ),
        ] {
            let source = format!("function tick%()\nreturn 9\nend function\n{statement}\nend\n");
            let output = compile_source(filename, &source)
                .expect("compile source through the generated frontend and BASIC backend");

            assert!(
                output.contains(expected),
                "driver output must evaluate typed operands for {statement}: {output}"
            );
        }
    }

    #[test]
    fn compile_file_preserves_typed_module_statement_operands() {
        let directory = tempfile::tempdir().expect("temporary typed BASIC project");
        let input = directory.path().join("typed_statements.bcl");
        let source = "program typed\nfunction tick%()\nreturn 9\nend function\nlocate tick%(), tick%()\ncolor tick%(), tick%()\nwidth #tick%(), tick%()\npoke tick%(), tick%()\nout tick%(), tick%()\nend\n";
        fs::write(&input, source).expect("write typed BASIC source");

        let direct_basic = compile_source(input.display().to_string(), source)
            .expect("compile typed module statements through generated frontend");
        for target in [Target::Basic, Target::Fbc] {
            let from_file = compile_file(
                &input,
                &CompileOptions {
                    target,
                    ..CompileOptions::new()
                },
            )
            .unwrap_or_else(|diagnostics| panic!("compile file for {target:?}: {diagnostics:?}"));
            assert_eq!(
                from_file, direct_basic,
                "file driver output differs from direct typed {target:?} codegen"
            );
        }
        assert!(
            direct_basic.contains("LOCATE BCCT1%, BCCT2%")
                && direct_basic.contains("COLOR BCCT3%, BCCT4%")
                && direct_basic.contains("WIDTH #BCCT5%, BCCT6%")
                && direct_basic.contains("POKE BCCT7%, BCCT8%")
                && direct_basic.contains("OUT BCCT9%, BCCT10%"),
            "BASIC driver output lost typed operand evaluation: {}",
            direct_basic
        );

        // Use a statement subset accepted by C and JVM to verify their
        // compile_file target dispatch independently of BASIC-only I/O.
        let cross_target_input = directory.path().join("typed_call_assignment.bcl");
        let cross_target_source =
            "program typed\nfunction tick%()\nreturn 9\nend function\nresult% = tick%()\nend\n";
        fs::write(&cross_target_input, cross_target_source)
            .expect("write cross-target typed source");
        let parsed = parse_source(
            cross_target_input.display().to_string(),
            cross_target_source,
        )
        .unwrap();
        let lower::Lowered {
            program,
            lowered_record_files,
            ..
        } = lower::lower(parsed).unwrap();
        let mut module = semantic_ir::parse_and_adapt_named(
            cross_target_input.display().to_string(),
            cross_target_source,
        )
        .expect("adapt cross-target typed IR");
        module.lowered_record_files = lowered_record_files;
        let typed = resolver::resolve_with_semantic(program, Some(module)).unwrap();
        for (target, expected) in [
            (
                Target::C,
                crate::codegen_c::generate(&typed, Target::C)
                    .expect("transpile typed C")
                    .app,
            ),
            (
                Target::Jvm,
                crate::codegen_jvm::generate(&typed).expect("transpile typed JVM"),
            ),
        ] {
            let from_file = compile_file(
                &cross_target_input,
                &CompileOptions {
                    target,
                    ..CompileOptions::new()
                },
            )
            .unwrap_or_else(|diagnostics| panic!("compile file for {target:?}: {diagnostics:?}"));
            assert_eq!(
                from_file, expected,
                "file driver output differs from direct typed {target:?} codegen"
            );
        }
    }

    /// Every corpus program (tutorials, examples, fixtures), skipping the
    /// early `adventure3000` stages that are deliberately raw classic BASIC.
    fn corpus_programs() -> Vec<PathBuf> {
        fn visit(directory: &Path, out: &mut Vec<PathBuf>) {
            let Ok(entries) = fs::read_dir(directory) else {
                return;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // `tests/fixtures/invalid` holds programs meant to be
                    // rejected; they have their own test.
                    if path.file_name().is_none_or(|name| name != "invalid") {
                        visit(&path, out);
                    }
                } else if path.extension().is_some_and(|extension| extension == "bcl") {
                    out.push(path);
                }
            }
        }
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut programs = Vec::new();
        for directory in ["tutorial", "examples", "tests/fixtures"] {
            visit(&root.join(directory), &mut programs);
        }
        programs.retain(|path| {
            let text = path.display().to_string();
            !(text.contains("adventure3000")
                && ["stage2-", "stage3-", "stage4-", "stage5-", "stage6-", "stage7-", "stage8-", "stage9-", "stage10-", "stage11-"]
                    .iter()
                    .any(|stage| text.contains(stage)))
        });
        programs.sort();
        programs
    }

    /// Programs whose output changes when the AST is emptied, per target.
    fn ast_dependent_programs(target: Target) -> Vec<String> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let options = CompileOptions {
            target,
            library_dirs: vec![root.to_path_buf()],
            ..CompileOptions::new()
        };
        corpus_programs()
            .into_iter()
            .filter(|program| {
                let with_ast = compile_file_impl(program, &options, AstUse::AstOnly);
                let without_ast = compile_file_impl(program, &options, AstUse::TypedOnly);
                match (with_ast, without_ast) {
                    (Ok(with_ast), Ok(without_ast)) => with_ast != without_ast,
                    // A program the target rejects has no output to depend on
                    // the AST; emptying the AST only hides the rejection.
                    (Err(_), Err(_)) => false,
                    _ => true,
                }
            })
            .map(|program| {
                program
                    .strip_prefix(root)
                    .unwrap_or(&program)
                    .display()
                    .to_string()
            })
            .collect()
    }

    /// BASIC output must not depend on `ResolvedProgram::program`: emptying
    /// the AST leaves every corpus program's output unchanged.
    /// A construct the JVM backend cannot emit is reported from the typed IR
    /// as well, not silently dropped when there is no AST to fall back to.
    #[test]
    fn jvm_reports_unsupported_typed_statements_without_the_ast() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let options = CompileOptions {
            target: Target::Jvm,
            ..CompileOptions::new()
        };
        let program = root.join("tests/fixtures/conformance/nested_on_gosub.bcl");
        let diagnostics = compile_file_impl(&program, &options, AstUse::TypedOnly)
            .expect_err("gosub is rejected without the AST too");
        assert!(
            diagnostics.iter().any(|d| d.message.contains("GOSUB is not supported")),
            "{diagnostics:?}"
        );
    }

    /// C output must not depend on `ResolvedProgram::program`: emptying the
    /// AST leaves every corpus program's C unchanged.
    #[test]
    fn c_output_is_independent_of_the_ast_for_the_corpus() {
        let dependent = ast_dependent_programs(Target::C);
        assert!(
            dependent.is_empty(),
            "C output still depends on the AST for: {dependent:#?}"
        );
    }

    /// The growing corpus of invalid programs: every `tests/fixtures/invalid/
    /// <name>.bcl` has a `<name>.expected` file with one line per expectation,
    /// `<target>: ok` when the target must accept the program or
    /// `<target>: <text>` when compiling it must fail with a diagnostic that
    /// contains `<text>`. A target is `basic`, `c`, `jvm` or `all` (a specific
    /// target line overrides `all`); blank lines and `#` comments are ignored.
    /// Add a pair of files to extend it. See `tests/fixtures/invalid/README.md`.
    #[test]
    fn invalid_program_corpus_reports_the_expected_diagnostics() {
        let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/invalid");
        let mut programs: Vec<PathBuf> = fs::read_dir(&directory)
            .expect("read tests/fixtures/invalid")
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "bcl"))
            .collect();
        programs.sort();
        assert!(programs.len() >= 50, "the invalid-program corpus shrank");
        let mut failures = Vec::new();
        for program in &programs {
            let name = program.file_name().unwrap().to_string_lossy().to_string();
            let expected_path = program.with_extension("expected");
            let Ok(expected_text) = fs::read_to_string(&expected_path) else {
                failures.push(format!("{name}: missing {}", expected_path.display()));
                continue;
            };
            let mut expectations: std::collections::HashMap<String, String> =
                std::collections::HashMap::new();
            for line in expected_text.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                let Some((target, text)) = line.split_once(':') else {
                    failures.push(format!("{name}: malformed expectation `{line}`"));
                    continue;
                };
                expectations.insert(target.trim().to_string(), text.trim().to_string());
            }
            for (label, target) in [
                ("basic", Target::Basic),
                ("c", Target::C),
                ("jvm", Target::Jvm),
            ] {
                let Some(expectation) = expectations.get(label).or_else(|| expectations.get("all"))
                else {
                    failures.push(format!("{name}: no expectation for {label}"));
                    continue;
                };
                let options = CompileOptions {
                    target,
                    ..CompileOptions::new()
                };
                match (compile_file(program, &options), expectation.as_str()) {
                    (Ok(_), "ok") => {}
                    (Ok(_), text) => failures.push(format!(
                        "{name} [{label}]: expected an error containing `{text}` but it compiled"
                    )),
                    (Err(diagnostics), "ok") => failures.push(format!(
                        "{name} [{label}]: expected it to compile but got: {}",
                        diagnostics
                            .iter()
                            .map(|d| d.message.clone())
                            .collect::<Vec<_>>()
                            .join(" | ")
                    )),
                    (Err(diagnostics), text) => {
                        if !diagnostics.iter().any(|d| d.message.contains(text)) {
                            failures.push(format!(
                                "{name} [{label}]: expected a diagnostic containing `{text}` but got: {}",
                                diagnostics
                                    .iter()
                                    .map(|d| d.message.clone())
                                    .collect::<Vec<_>>()
                                    .join(" | ")
                            ));
                        }
                    }
                }
            }
        }
        // Every expectation file must belong to a program.
        for entry in fs::read_dir(&directory).unwrap().flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|extension| extension == "expected")
                && !path.with_extension("bcl").exists()
            {
                failures.push(format!("{}: no matching .bcl", path.display()));
            }
        }
        assert!(failures.is_empty(), "invalid-program corpus:\n{}", failures.join("\n"));
    }

    /// Debug aid: `BAD_DIR=<dir> cargo test --lib show_invalid_program_diagnostics
    /// -- --ignored --nocapture` compiles every `.bcl` in the directory for each
    /// target both AST-driven and typed-only and prints the diagnostics.
    #[test]
    #[ignore]
    fn show_invalid_program_diagnostics() {
        let directory = PathBuf::from(std::env::var("BAD_DIR").expect("BAD_DIR"));
        let mut files: Vec<PathBuf> = fs::read_dir(&directory)
            .expect("read BAD_DIR")
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "bcl"))
            .collect();
        files.sort();
        let summary = |result: Result<String, Vec<Diagnostic>>| match result {
            Ok(_) => "OK".to_string(),
            Err(diagnostics) => diagnostics
                .iter()
                .map(|d| d.message.replace('\n', " "))
                .collect::<Vec<_>>()
                .join(" | "),
        };
        for file in files {
            for (name, target) in [
                ("basic", Target::Basic),
                ("c", Target::C),
                ("jvm", Target::Jvm),
            ] {
                let options = CompileOptions {
                    target,
                    ..CompileOptions::new()
                };
                let ast = summary(compile_file_impl(&file, &options, AstUse::AstOnly));
                let typed = summary(compile_file_impl(&file, &options, AstUse::TypedOnly));
                let flag = if ast == typed { "same" } else { "DIFF" };
                println!(
                    "{flag} {} [{name}]\n    ast:   {ast}\n    typed: {typed}",
                    file.file_name().unwrap().to_string_lossy()
                );
            }
        }
    }

    /// `try`/`catch`/`finally` with an error filter and a source binding is
    /// emitted for C from the typed IR alone.
    #[test]
    fn c_try_catch_is_emitted_from_typed_ir_without_the_ast() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let options = CompileOptions {
            target: Target::C,
            ..CompileOptions::new()
        };
        let program = root.join("tests/fixtures/conformance/jvm_try_filter.bcl");
        let output = compile_file_impl(&program, &options, AstUse::TypedOnly)
            .expect("try/catch compiles for C without the AST");
        assert!(output.contains("bcc_on_error_target = 0;"), "{output}");
        assert!(output.contains("if (!((bcc_err == 6) || (bcc_err == 7)))"), "{output}");
        assert!(output.contains("bcc_try_0_finally: ;"), "{output}");
        assert!(output.contains("bcc_err_file"), "{output}");
    }

    /// The JVM equivalent: every corpus program the JVM target accepts emits
    /// the same class with the AST emptied.
    #[test]
    fn jvm_output_is_independent_of_the_ast_for_the_corpus() {
        let dependent = ast_dependent_programs(Target::Jvm);
        assert!(
            dependent.is_empty(),
            "JVM output still depends on the AST for: {dependent:#?}"
        );
    }

    #[test]
    fn basic_output_is_independent_of_the_ast_for_the_corpus() {
        let dependent = ast_dependent_programs(Target::Basic);
        assert!(
            dependent.is_empty(),
            "BASIC output still depends on the AST for: {dependent:#?}"
        );
    }

    /// Debug aid: `DIFF_FILE=<path> DIFF_TARGET=basic|jvm|c cargo test --lib
    /// show_ast_diff -- --ignored`, then diff `tmp/diff_a.txt` (with the AST)
    /// against `tmp/diff_b.txt` (AST emptied).
    #[test]
    #[ignore]
    fn show_ast_diff() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let target = match std::env::var("DIFF_TARGET").as_deref() {
            Ok("basic") => Target::Basic,
            Ok("jvm") => Target::Jvm,
            _ => Target::C,
        };
        let options = CompileOptions {
            target,
            library_dirs: vec![root.to_path_buf()],
            ..CompileOptions::new()
        };
        let program = root.join(std::env::var("DIFF_FILE").expect("DIFF_FILE"));
        let render = |clear_ast| {
            compile_file_impl(
                &program,
                &options,
                if clear_ast { AstUse::TypedOnly } else { AstUse::AstOnly },
            )
                .unwrap_or_else(|error| format!("ERR {error:?}"))
        };
        fs::create_dir_all(root.join("tmp")).unwrap();
        fs::write(root.join("tmp/diff_a.txt"), render(false)).unwrap();
        fs::write(root.join("tmp/diff_b.txt"), render(true)).unwrap();
    }

    #[test]
    fn c_sequential_file_program_is_transpiled_over_the_typed_ir() {
        let directory = tempfile::tempdir().expect("temporary sequential-file project");
        let input = directory.path().join("sequential.bcl");
        fs::write(
            &input,
            "program s\nfile log = open(\"log.txt\") for output\nlog.write(\"a\", 1)\nlog.close()\nend\n",
        )
        .expect("write sequential-file source");
        let output = compile_file(
            &input,
            &CompileOptions {
                target: Target::C,
                ..CompileOptions::new()
            },
        )
        .expect("sequential-file program compiles for C");
        // The expansion's comments and file operations come from the typed
        // module, in source order.
        let open = output.find("bcc_files[0] = fopen(\"log.txt\", \"w\")").expect("open");
        let write = output.find("fprintf(bcc_files[0]").expect("write");
        let close = output.find("fclose(bcc_files[0])").expect("close");
        assert!(open < write && write < close, "{output}");
        assert!(output.contains("// log.write(...)"), "{output}");
    }

    #[test]
    fn compile_file_preserves_semantic_record_field_types_across_backends() {
        let directory = tempfile::tempdir().expect("temporary typed record project");
        let filename = "typed_record_fields.bcl";
        let input = directory.path().join(filename);
        let source = "program typed\nrecord Entry\nid: int16\nname: string(8)\nend record\nlet row = { id: 1, name: \"ast\" }\nrow.id = 7\nrow.name = \"typed\"\nprint row.id, row.name\nend\n";
        fs::write(&input, source).expect("write typed record source");

        let parsed = parse_source(filename.to_string(), source).expect("parse typed record source");
        let lower::Lowered {
            program,
            synthesized_buffer_names,
            ..
        } = lower::lower(parsed).expect("lower typed record source");
        let module = semantic_ir::parse_and_adapt_named(filename, source)
            .expect("adapt typed record source");
        // Every backend consumes the transpiled module.
        let mut transpiled = module.clone();
        record_transpile::transpile(&mut transpiled);
        let resolved_transpiled = resolver::resolve_with_semantic(program.clone(), Some(transpiled))
            .expect("resolve transpiled typed record source");
        let expected = [
            crate::codegen::CodeGenerator::new()
                .with_synthesized_buffer_names(synthesized_buffer_names)
                .generate(&resolved_transpiled)
                .expect("transpile typed record to BASIC"),
            crate::codegen_c::generate(&resolved_transpiled, Target::C)
                .expect("transpile typed record to C")
                .app,
            crate::codegen_jvm::generate(&resolved_transpiled)
                .expect("transpile typed record to JVM"),
        ];

        for (target, expected) in [Target::Basic, Target::C, Target::Jvm]
            .into_iter()
            .zip(expected)
        {
            let actual = compile_file(
                &input,
                &CompileOptions {
                    target,
                    ..CompileOptions::new()
                },
            )
            .unwrap_or_else(|diagnostics| {
                panic!("compile typed record through {target:?}: {diagnostics:?}")
            });
            assert_eq!(actual, expected, "{target:?} driver output");
        }
    }

    #[test]
    fn compile_file_preserves_lowered_record_file_layouts_across_backends() {
        let directory = tempfile::tempdir().expect("temporary typed record-file project");
        let filename = "typed_record_file.bcl";
        let input = directory.path().join(filename);
        let source = "program typed\nrecord Entry\nid: int16\nname: string(8)\nend record\nfile db as Entry = open(\"entries.dat\")\ndb[1] = { id: 7, name: \"typed\" }\nend\n";
        fs::write(&input, source).expect("write typed record-file source");

        let source_name = input.display().to_string();
        let parsed =
            parse_source(source_name.clone(), source).expect("parse typed record-file source");
        let lower::Lowered {
            program,
            synthesized_buffer_names,
            lowered_record_files,
        } = lower::lower(parsed).expect("lower typed record-file source");
        let mut module = semantic_ir::parse_and_adapt_named(source_name, source)
            .expect("adapt typed record-file source");
        module.lowered_record_files = lowered_record_files;
        // Every backend consumes the transpiled module.
        let mut transpiled = module.clone();
        record_transpile::transpile(&mut transpiled);
        let resolved_transpiled = resolver::resolve_with_semantic(program.clone(), Some(transpiled))
            .expect("resolve transpiled typed record-file source");
        let expected = [
            crate::codegen::CodeGenerator::new()
                .with_synthesized_buffer_names(synthesized_buffer_names)
                .generate(&resolved_transpiled)
                .expect("transpile typed record file to BASIC"),
            crate::codegen_c::generate(&resolved_transpiled, Target::C)
                .expect("transpile typed record file to C")
                .app,
            crate::codegen_jvm::generate(&resolved_transpiled)
                .expect("transpile typed record file to JVM"),
        ];

        for (target, expected) in [Target::Basic, Target::C, Target::Jvm]
            .into_iter()
            .zip(expected)
        {
            let actual = compile_file(
                &input,
                &CompileOptions {
                    target,
                    ..CompileOptions::new()
                },
            )
            .unwrap_or_else(|diagnostics| {
                panic!("compile typed record file through {target:?}: {diagnostics:?}")
            });
            assert_eq!(actual, expected, "{target:?} driver output");
        }
    }

    #[test]
    fn compile_file_transpiles_scalar_method_byref_prints_across_backends() {
        let directory = tempfile::tempdir().expect("temporary scalar method project");
        let filename = "typed_scalar_method_byref_print.bcl";
        let input = directory.path().join(filename);
        let source = "program typed\nmethod adjust%[integer](byref delta%)\ndelta%=delta%+9\nreturn self%+delta%\nend method\nvalue%=20\nimplicit%=3\nprint value%.adjust(implicit%)\nend\n";
        fs::write(&input, source).expect("write typed scalar method source");

        let parsed = parse_source(filename.to_string(), source).expect("parse typed method");
        let lower::Lowered {
            program,
            synthesized_buffer_names,
            ..
        } = lower::lower(parsed).expect("lower typed method");
        let module =
            semantic_ir::parse_and_adapt_named(filename, source).expect("adapt typed method");
        let resolved =
            resolver::resolve_with_semantic(program, Some(module)).expect("resolve typed method");
        let expected = [
            crate::codegen::CodeGenerator::new()
                .with_synthesized_buffer_names(synthesized_buffer_names)
                .generate(&resolved)
                .expect("transpile typed method to BASIC"),
            crate::codegen_c::generate(&resolved, Target::C)
                .expect("transpile typed method to C")
                .app,
            crate::codegen_jvm::generate(&resolved).expect("transpile typed method to JVM"),
        ];

        for (target, expected) in [Target::Basic, Target::C, Target::Jvm]
            .into_iter()
            .zip(expected)
        {
            let actual = compile_file(
                &input,
                &CompileOptions {
                    target,
                    ..CompileOptions::new()
                },
            )
            .unwrap_or_else(|diagnostics| {
                panic!("compile scalar method through {target:?}: {diagnostics:?}")
            });
            assert_eq!(actual, expected, "{target:?} driver output");
        }
    }

    #[test]
    fn compile_file_transpiles_scalar_method_assignment_and_condition_across_backends() {
        let directory = tempfile::tempdir().expect("temporary scalar method expression project");
        let filename = "typed_scalar_method_expressions.bcl";
        let input = directory.path().join(filename);
        let source = "program typed\nmethod adjust%[integer](delta%)\nreturn self%+delta%\nend method\nmethod bump%[integer](byref target%)\ntarget%=target%+1\nreturn self%\nend method\nfunction transformed%(input%)\nreturn input%.adjust(3)\nend function\nvalue%=20\nbound%=8\nresult%=value%.adjust(7)\nif value%.adjust(0) then\nprint result%\nend if\nwhile value%.adjust(-20)\nprint result%\nend while\ndo while value%.adjust(-20)\nprint result%\nend do\ndo\nprint result%\nloop until value%.adjust(0)\nfor snapshot%=value% to value%.bump(value%)\nprint snapshot%\nend for\nalias%=5\nfor alias%=alias% to alias%.bump(alias%)\nprint alias%\nend for\nfor stepSnapshot%=value% to bound% step bound%.bump(bound%)\nprint stepSnapshot%\nend for\nbodyLimit%=3\nfor bodyIndex%=1 to bodyLimit%\nbodyLimit%=0\nprint bodyIndex%\nend for\ndim index%\nfor index%=value%.adjust(1) to value%.adjust(2) step value%.adjust(0)\nprint index%\nend for\nselect case value%.adjust(7)\ncase 27\nprint result%\ncase else\nprint 0\nend select\nprint value%.adjust(1)+value%.adjust(2)\nprint transformed%(value%)\nend\n";
        fs::write(&input, source).expect("write typed scalar method expressions");

        let parsed = parse_source(filename.to_string(), source).expect("parse typed methods");
        let lower::Lowered {
            program,
            synthesized_buffer_names,
            ..
        } = lower::lower(parsed).expect("lower typed methods");
        let module =
            semantic_ir::parse_and_adapt_named(filename, source).expect("adapt typed methods");
        let resolved =
            resolver::resolve_with_semantic(program, Some(module)).expect("resolve typed methods");
        let expected = [
            crate::codegen::CodeGenerator::new()
                .with_synthesized_buffer_names(synthesized_buffer_names)
                .generate(&resolved)
                .expect("transpile method expressions to BASIC"),
            crate::codegen_c::generate(&resolved, Target::C)
                .expect("transpile method expressions to C")
                .app,
            crate::codegen_jvm::generate(&resolved).expect("transpile method expressions to JVM"),
        ];

        for (target, expected) in [Target::Basic, Target::C, Target::Jvm]
            .into_iter()
            .zip(expected)
        {
            let actual = compile_file(
                &input,
                &CompileOptions {
                    target,
                    ..CompileOptions::new()
                },
            )
            .unwrap_or_else(|diagnostics| {
                panic!("compile method expressions through {target:?}: {diagnostics:?}")
            });
            assert_eq!(actual, expected, "{target:?} driver output");
        }
    }

    #[test]
    fn semantic_array_element_types_reach_driver_backends() {
        let filename = "driver_semantic_array_element_types.bcl";
        let directory = tempfile::tempdir().expect("temporary typed-source directory");
        let input = directory.path().join(filename);
        let ast_source = "program typed\ndim values(2)\nvalues(1) = 2\nprint values(1)\nend\n";
        let parsed = parse_source(filename.to_string(), ast_source).unwrap();
        let lower::Lowered {
            program,
            synthesized_buffer_names,
            ..
        } = lower::lower(parsed).unwrap();
        for (annotation, c_storage, jvm_array, c_name, store, load) in [
            (
                "integer",
                "int bv_i_values[3]",
                "[I",
                "bv_i_values",
                "iastore",
                "iaload",
            ),
            (
                "long",
                "int bv_l_values[3]",
                "[J",
                "bv_l_values",
                "lastore",
                "laload",
            ),
            (
                "single",
                "float bv_f_values[3]",
                "[D",
                "bv_f_values",
                "dastore",
                "daload",
            ),
            (
                "double",
                "double bv_d_values[3]",
                "[D",
                "bv_d_values",
                "dastore",
                "daload",
            ),
            (
                "string",
                "bv_s_values[3]",
                "[Ljava/lang/String;",
                "bv_s_values",
                "aastore",
                "aaload",
            ),
        ] {
            let assigned = if annotation == "string" {
                "\"typed\""
            } else {
                "7"
            };
            let semantic_source = format!(
                "program typed\ndim values(2) as {annotation}\nvalues(1) = {assigned}\nprint values(1)\nend\n"
            );
            fs::write(&input, &semantic_source).expect("write typed source for driver");
            let module = semantic_ir::parse_and_adapt_named(filename, &semantic_source).unwrap();
            let typed = resolver::resolve_with_semantic(program.clone(), Some(module)).unwrap();
            let typed_output = [
                crate::codegen::CodeGenerator::new()
                    .with_synthesized_buffer_names(synthesized_buffer_names.clone())
                    .generate(&typed)
                    .expect("transpile typed BASIC"),
                crate::codegen_c::generate(&typed, Target::C)
                    .expect("transpile typed C")
                    .app,
                crate::codegen_jvm::generate(&typed).expect("transpile typed JVM"),
            ];
            for (target, expected) in [
                (Target::Basic, &typed_output[0]),
                (Target::Fbc, &typed_output[0]),
                (Target::C, &typed_output[1]),
                (Target::Jvm, &typed_output[2]),
            ] {
                let driver_output = compile_file(
                    &input,
                    &CompileOptions {
                        target,
                        ..CompileOptions::new()
                    },
                )
                .unwrap_or_else(|diagnostics| {
                    panic!("compile {annotation} source through {target:?} driver: {diagnostics:?}")
                });
                assert_eq!(
                    &driver_output, expected,
                    "{target:?} driver changed typed {annotation} array output"
                );
            }
            let compatibility_parsed = parse_source(filename.to_string(), &semantic_source)
                .expect("parse typed source through compatibility AST");
            let lower::Lowered {
                program: compatibility_program,
                synthesized_buffer_names: compatibility_buffers,
                ..
            } = lower::lower(compatibility_parsed).expect("lower compatibility AST");
            let compatibility =
                resolver::resolve(compatibility_program).expect("resolve compatibility AST");
            let compatibility_basic = crate::codegen::CodeGenerator::new()
                .with_synthesized_buffer_names(compatibility_buffers)
                .generate(&compatibility)
                .expect("transpile compatibility BASIC");
            let compatibility_c = crate::codegen_c::generate(&compatibility, Target::C);
            let compatibility_jvm = crate::codegen_jvm::generate(&compatibility);
            assert!(
                typed_output[0].contains(&format!(
                    "DIM values(2) AS {}",
                    annotation.to_ascii_uppercase()
                )),
                "BASIC lost {annotation} annotation:\n{}",
                typed_output[0]
            );
            assert_ne!(
                typed_output[0], compatibility_basic,
                "BASIC compatibility output unexpectedly retained the {annotation} DIM fact"
            );
            if annotation == "string" {
                assert!(
                    compatibility_c.is_err(),
                    "C compatibility path unexpectedly accepted untyped STRING array assignment"
                );
            } else if annotation != "single" {
                let compatibility_c = compatibility_c.expect("transpile compatibility C").app;
                assert_ne!(
                    typed_output[1], compatibility_c,
                    "C compatibility output unexpectedly retained the {annotation} DIM fact"
                );
            }
            if matches!(annotation, "integer" | "long" | "string") {
                match compatibility_jvm {
                    Ok(compatibility_jvm) => assert_ne!(
                        typed_output[2], compatibility_jvm,
                        "JVM compatibility output unexpectedly retained the {annotation} DIM fact"
                    ),
                    Err(_) => assert_eq!(
                        annotation, "string",
                        "JVM compatibility path unexpectedly rejected {annotation} array"
                    ),
                }
            }
            assert!(
                typed_output[1].contains(c_storage),
                "C lost {annotation} element type:\n{}",
                typed_output[1]
            );
            assert!(
                typed_output[1].contains(c_name) && typed_output[1].contains(assigned),
                "C lost {annotation} array access:\n{}",
                typed_output[1]
            );
            assert!(
                typed_output[2].contains(&format!(".field public static a0 {jvm_array}")),
                "JVM lost {annotation} element type:\n{}",
                typed_output[2]
            );
            assert!(
                typed_output[2].contains(store) && typed_output[2].contains(load),
                "JVM lost {annotation} array access:\n{}",
                typed_output[2]
            );
        }
    }

    #[test]
    fn semantic_long_array_assignment_reaches_c_driver_output() {
        let filename = "driver_semantic_long_array_assignment.bcl";
        let ast_source = "dim values(2)\nvalues(1) = 2\nprint values(1)\nend\n";
        let semantic_source = "dim values(2) as long\nvalues(1) = 7\nprint values(1)\nend\n";
        let parsed = parse_source(filename.to_string(), ast_source).unwrap();
        let lower::Lowered {
            program,
            synthesized_buffer_names,
            ..
        } = lower::lower(parsed).unwrap();
        let module = semantic_ir::parse_and_adapt_named(filename, semantic_source).unwrap();
        let resolved = resolver::resolve_with_semantic(program, Some(module)).unwrap();
        let basic = crate::codegen::CodeGenerator::new()
            .with_synthesized_buffer_names(synthesized_buffer_names)
            .generate(&resolved)
            .expect("transpile typed LONG array to BASIC");
        let c = crate::codegen_c::generate(&resolved, Target::C)
            .expect("transpile typed LONG array assignment to C")
            .app;
        let jvm =
            crate::codegen_jvm::generate(&resolved).expect("transpile typed LONG array to JVM");
        assert!(
            basic.contains("DIM values(2) AS LONG") && basic.contains("values(1)"),
            "{basic}"
        );
        assert!(c.contains("bv_l_values[(1)] = 7;"), "{c}");
        assert!(!c.contains("bv_l_values[(1)] = 2;"), "{c}");
        assert!(c.contains("bv_l_values[(1)]"), "{c}");
        assert!(jvm.contains(".field public static a0 [J"), "{jvm}");
        assert!(jvm.contains("lastore") && jvm.contains("laload"), "{jvm}");
    }

    #[test]
    fn semantic_long_array_assignment_reaches_c64_driver_output() {
        let filename = "driver_semantic_long_array_assignment_c64.bcl";
        let directory = tempfile::tempdir().expect("temporary C64 source directory");
        let input = directory.path().join(filename);
        let ast_source = "program typed\ndim values(2)\nvalues(1) = 2\nprint values(1)\nend\n";
        let semantic_source =
            "program typed\ndim values(2) as long\nvalues(1) = 7\nprint values(1)\nend\n";
        fs::write(&input, semantic_source).expect("write typed C64 source for driver");
        let parsed = parse_source(filename.to_string(), ast_source).unwrap();
        let lower::Lowered { program, .. } = lower::lower(parsed).unwrap();
        let module = semantic_ir::parse_and_adapt_named(filename, semantic_source).unwrap();
        let resolved = resolver::resolve_with_semantic(program, Some(module)).unwrap();
        let c = crate::codegen_c::generate(&resolved, Target::C64)
            .expect("transpile typed LONG array assignment to C64")
            .app;
        assert!(c.contains("bv_l_values[(1)] = 7;"), "{c}");
        assert!(!c.contains("bv_l_values[(1)] = 2;"), "{c}");
        assert!(c.contains("bv_l_values[(1)]"), "{c}");
        let driver_output = compile_file(
            &input,
            &CompileOptions {
                target: Target::C64,
                ..CompileOptions::new()
            },
        )
        .expect("compile typed LONG array through C64 driver");
        assert_eq!(driver_output, c);
    }

    #[test]
    fn semantic_callable_local_array_type_reaches_driver_backends() {
        let filename = "driver_semantic_callable_local_array_type.bcl";
        let directory = tempfile::tempdir().expect("temporary typed-source directory");
        let input = directory.path().join(filename);
        let ast_source = "program typed\nfunction read%()\ndim values(2)\nvalues(1) = 2\nprint values(1)\nreturn 0\nend function\nprint read%()\nend\n";
        let parsed = parse_source(filename.to_string(), ast_source).unwrap();
        let lower::Lowered {
            program,
            synthesized_buffer_names,
            ..
        } = lower::lower(parsed).unwrap();
        for (annotation, c_storage, c_name, store, load) in [
            (
                "integer",
                "bv_i_values[3]",
                "bv_i_values",
                "iastore",
                "iaload",
            ),
            ("long", "bv_l_values[3]", "bv_l_values", "lastore", "laload"),
            (
                "single",
                "bv_f_values[3]",
                "bv_f_values",
                "dastore",
                "daload",
            ),
            (
                "double",
                "bv_d_values[3]",
                "bv_d_values",
                "dastore",
                "daload",
            ),
            (
                "string",
                "bv_s_values[3]",
                "bv_s_values",
                "aastore",
                "aaload",
            ),
        ] {
            let assigned = if annotation == "string" {
                "\"typed\""
            } else {
                "7"
            };
            let semantic_source = format!(
                "program typed\nfunction read%()\ndim values(2) as {annotation}\nvalues(1) = {assigned}\nprint values(1)\nreturn 0\nend function\nprint read%()\nend\n"
            );
            fs::write(&input, &semantic_source).expect("write typed callable source for driver");
            let module = semantic_ir::parse_and_adapt_named(filename, &semantic_source).unwrap();
            let resolved = resolver::resolve_with_semantic(program.clone(), Some(module)).unwrap();
            let basic = crate::codegen::CodeGenerator::new()
                .with_synthesized_buffer_names(synthesized_buffer_names.clone())
                .generate(&resolved)
                .expect("transpile typed callable array to BASIC");
            let compatibility_parsed = parse_source(filename.to_string(), &semantic_source)
                .expect("parse typed callable source through compatibility AST");
            let lower::Lowered {
                program: compatibility_program,
                synthesized_buffer_names: compatibility_buffers,
                ..
            } = lower::lower(compatibility_parsed).expect("lower compatibility callable AST");
            let compatibility =
                resolver::resolve(compatibility_program).expect("resolve compatibility AST");
            let compatibility_basic = crate::codegen::CodeGenerator::new()
                .with_synthesized_buffer_names(compatibility_buffers)
                .generate(&compatibility)
                .expect("transpile compatibility callable BASIC");
            let compatibility_c = crate::codegen_c::generate(&compatibility, Target::C);
            let compatibility_jvm = crate::codegen_jvm::generate(&compatibility);
            let c = crate::codegen_c::generate(&resolved, Target::C)
                .expect("transpile typed callable array to C")
                .app;
            let jvm = crate::codegen_jvm::generate(&resolved)
                .expect("transpile typed callable array to JVM");
            for (target, expected) in [
                (Target::Basic, &basic),
                (Target::Fbc, &basic),
                (Target::C, &c),
                (Target::Jvm, &jvm),
            ] {
                let driver_output = compile_file(
                    &input,
                    &CompileOptions {
                        target,
                        ..CompileOptions::new()
                    },
                )
                .unwrap_or_else(|diagnostics| {
                    panic!(
                        "compile callable {annotation} source through {target:?}: {diagnostics:?}"
                    )
                });
                assert_eq!(
                    &driver_output, expected,
                    "{target:?} driver changed callable-local {annotation} array output"
                );
            }
            assert!(
                basic.contains(&format!("AS {}", annotation.to_ascii_uppercase()))
                    && basic.contains(assigned),
                "BASIC lost callable-local {annotation} array type:\n{basic}"
            );
            assert_ne!(
                basic, compatibility_basic,
                "BASIC compatibility path unexpectedly retained callable-local {annotation} DIM"
            );
            if annotation == "string" {
                assert!(
                    compatibility_c.is_err(),
                    "C compatibility path unexpectedly accepted untyped callable STRING array"
                );
            } else if annotation != "single" {
                let compatibility_c = compatibility_c
                    .expect("transpile compatibility callable C")
                    .app;
                assert_ne!(
                    c, compatibility_c,
                    "C compatibility output unexpectedly retained callable-local {annotation} DIM"
                );
            }
            if matches!(annotation, "integer" | "long" | "string") {
                match compatibility_jvm {
                    Ok(compatibility_jvm) => assert_ne!(
                        jvm, compatibility_jvm,
                        "JVM compatibility output unexpectedly retained callable-local {annotation} DIM"
                    ),
                    Err(diagnostics) => assert!(
                        !diagnostics.is_empty(),
                        "JVM compatibility path returned an empty callable {annotation} diagnostic"
                    ),
                }
            }
            assert!(
                c.contains(c_storage) && c.contains(c_name),
                "C lost callable-local {annotation} array type:\n{c}"
            );
            assert!(
                jvm.contains(store) && jvm.contains(load),
                "JVM lost callable-local {annotation} array type:\n{jvm}"
            );
        }
    }

    #[test]
    fn semantic_callable_long_array_reaches_c64_driver_output() {
        let filename = "driver_semantic_callable_long_array_c64.bcl";
        let directory = tempfile::tempdir().expect("temporary callable C64 source directory");
        let input = directory.path().join(filename);
        let ast_source = "program typed\nfunction read%()\ndim values(2)\nvalues(1) = 2\nprint values(1)\nreturn 0\nend function\nprint read%()\nend\n";
        let semantic_source = "program typed\nfunction read%()\ndim values(2) as long\nvalues(1) = 7\nprint values(1)\nreturn 0\nend function\nprint read%()\nend\n";
        fs::write(&input, semantic_source).expect("write typed callable C64 source");
        let parsed = parse_source(filename.to_string(), ast_source).unwrap();
        let lower::Lowered { program, .. } = lower::lower(parsed).unwrap();
        let module = semantic_ir::parse_and_adapt_named(filename, semantic_source).unwrap();
        let resolved = resolver::resolve_with_semantic(program, Some(module)).unwrap();
        let c = crate::codegen_c::generate(&resolved, Target::C64)
            .expect("transpile typed callable LONG array to C64")
            .app;
        assert!(c.contains("bv_l_values[3]"), "{c}");
        assert!(c.contains("bv_l_values[(1)] = 7;"), "{c}");
        assert!(!c.contains("bv_l_values[(1)] = 2;"), "{c}");
        let driver_output = compile_file(
            &input,
            &CompileOptions {
                target: Target::C64,
                ..CompileOptions::new()
            },
        )
        .expect("compile typed callable LONG array through C64 driver");
        assert_eq!(driver_output, c);
    }

    #[test]
    fn semantic_scalar_dim_type_reaches_all_driver_targets() {
        let filename = "driver_semantic_scalar_dim_type.bcl";
        let directory = tempfile::tempdir().expect("temporary typed scalar source directory");
        let input = directory.path().join(filename);
        let ast_source = "program typed\ndim value\nvalue = 2\nprint value\nend\n";
        let parsed = parse_source(filename.to_string(), ast_source).unwrap();
        let lower::Lowered { program, .. } = lower::lower(parsed).unwrap();
        for (annotation, c_name, jvm_type, assigned) in [
            ("integer", "bv_i_value", "I", "7"),
            ("long", "bv_l_value", "J", "7"),
            ("single", "bv_f_value", "D", "7"),
            ("double", "bv_d_value", "D", "7"),
            ("string", "bv_s_value", "Ljava/lang/String;", "\"typed\""),
        ] {
            let semantic_source = format!(
                "program typed\ndim value as {annotation}\nvalue = {assigned}\nprint value\nend\n"
            );
            fs::write(&input, &semantic_source).expect("write typed scalar source for driver");
            let module = semantic_ir::parse_and_adapt_named(filename, &semantic_source).unwrap();
            let resolved = resolver::resolve_with_semantic(program.clone(), Some(module)).unwrap();
            let basic = crate::codegen::CodeGenerator::new()
                .generate(&resolved)
                .expect("transpile typed scalar to BASIC");
            let c = crate::codegen_c::generate(&resolved, Target::C)
                .expect("transpile typed scalar to C")
                .app;
            let jvm =
                crate::codegen_jvm::generate(&resolved).expect("transpile typed scalar to JVM");
            assert!(
                basic.contains(&format!("DIM value AS {}", annotation.to_ascii_uppercase()))
                    && basic.contains(assigned),
                "BASIC lost {annotation} scalar DIM:\n{basic}"
            );
            assert!(
                c.contains(c_name) && c.contains(assigned),
                "C lost {annotation} scalar DIM:\n{c}"
            );
            assert!(
                jvm.contains(&format!(".field public static g1 {jvm_type}"))
                    && jvm.contains(assigned),
                "JVM lost {annotation} scalar DIM:\n{jvm}"
            );
            let mut targets = vec![
                (Target::Basic, basic.clone()),
                (Target::Fbc, basic.clone()),
                (Target::C, c.clone()),
                (Target::Jvm, jvm.clone()),
            ];
            if matches!(annotation, "integer" | "long") {
                let c64 = crate::codegen_c::generate(&resolved, Target::C64)
                    .expect("transpile supported typed scalar to C64")
                    .app;
                assert!(c64.contains(c_name) && c64.contains(assigned), "{c64}");
                targets.push((Target::C64, c64));
            }
            for (target, expected) in targets {
                let driver_output = compile_file(
                    &input,
                    &CompileOptions {
                        target,
                        ..CompileOptions::new()
                    },
                )
                .unwrap_or_else(|diagnostics| {
                    panic!("compile {annotation} scalar through {target:?}: {diagnostics:?}")
                });
                assert_eq!(
                    driver_output, expected,
                    "{target:?} {annotation} scalar driver output"
                );
            }
        }
    }

    #[test]
    fn semantic_callable_scalar_dim_type_reaches_all_driver_targets() {
        let filename = "driver_semantic_callable_scalar_dim_type.bcl";
        let directory = tempfile::tempdir().expect("temporary callable scalar source directory");
        let input = directory.path().join(filename);
        let ast_source = "program typed\nfunction read%()\ndim value\nvalue = 2\nprint value\nreturn 0\nend function\nprint read%()\nend\n";
        let parsed = parse_source(filename.to_string(), ast_source).unwrap();
        let lower::Lowered { program, .. } = lower::lower(parsed).unwrap();
        for (annotation, c_name, jvm_op, assigned) in [
            ("integer", "bv_i_value", "istore", "7"),
            ("long", "bv_l_value", "lstore", "7"),
            ("single", "bv_f_value", "dstore", "7"),
            ("double", "bv_d_value", "dstore", "7"),
            ("string", "bv_s_value", "astore", "\"typed\""),
        ] {
            let semantic_source = format!(
                "program typed\nfunction read%()\ndim value as {annotation}\nvalue = {assigned}\nprint value\nreturn 0\nend function\nprint read%()\nend\n"
            );
            fs::write(&input, &semantic_source).expect("write typed callable scalar source");
            let module = semantic_ir::parse_and_adapt_named(filename, &semantic_source).unwrap();
            let resolved = resolver::resolve_with_semantic(program.clone(), Some(module)).unwrap();
            let basic = crate::codegen::CodeGenerator::new()
                .generate(&resolved)
                .expect("transpile typed callable scalar to BASIC");
            let c = crate::codegen_c::generate(&resolved, Target::C)
                .expect("transpile typed callable scalar to C")
                .app;
            let jvm = crate::codegen_jvm::generate(&resolved)
                .expect("transpile typed callable scalar to JVM");
            assert!(
                basic.contains(&format!("AS {}", annotation.to_ascii_uppercase()))
                    && basic.contains(assigned),
                "BASIC lost callable {annotation} scalar DIM:\n{basic}"
            );
            assert!(
                c.contains(c_name) && c.contains(assigned),
                "C lost callable {annotation} scalar DIM:\n{c}"
            );
            assert!(
                jvm.contains(jvm_op),
                "JVM lost callable {annotation} scalar storage:\n{jvm}"
            );
            let mut targets = vec![
                (Target::Basic, basic.clone()),
                (Target::Fbc, basic),
                (Target::C, c),
                (Target::Jvm, jvm),
            ];
            if matches!(annotation, "integer" | "long") {
                let c64 = crate::codegen_c::generate(&resolved, Target::C64)
                    .expect("transpile supported callable scalar to C64")
                    .app;
                assert!(c64.contains(c_name) && c64.contains(assigned), "{c64}");
                targets.push((Target::C64, c64));
            }
            for (target, expected) in targets {
                let driver_output = compile_file(
                    &input,
                    &CompileOptions {
                        target,
                        ..CompileOptions::new()
                    },
                )
                .unwrap_or_else(|diagnostics| {
                    panic!(
                        "compile callable {annotation} scalar through {target:?}: {diagnostics:?}"
                    )
                });
                assert_eq!(
                    driver_output, expected,
                    "{target:?} callable {annotation} scalar driver output"
                );
            }
        }
    }
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
    let semantic_facts = semantic_ir::parse_and_adapt_named(path.display().to_string(), &source)
        .ok()
        .map(|module| {
            fn collect_dims(
                statements: &[semantic_ir::SemanticStatement],
                dim_vars: &mut Vec<ast::CommonVar>,
            ) -> bool {
                use semantic_ir::SemanticStatementKind as Kind;
                let mut has_other = false;
                for statement in statements {
                    match &statement.kind {
                        Kind::Line(body) => {
                            has_other |= collect_dims(body, dim_vars);
                        }
                        Kind::Comment { .. } => {}
                        Kind::Dim(items) => {
                            dim_vars.extend(items.iter().map(|item| ast::CommonVar {
                                name: ast::BasicIdent::parse(&item.name),
                                is_array: item.array_axes > 0,
                            }));
                        }
                        _ => has_other = true,
                    }
                }
                has_other
            }

            let mut dim_vars = Vec::new();
            let has_other_statements = collect_dims(&module.statements, &mut dim_vars);
            let (has_program_header, has_library_header, shared_decl) = match module.header {
                Some(semantic_ir::ModuleHeader::Program { .. }) => (true, false, None),
                Some(semantic_ir::ModuleHeader::Library { .. }) => (false, true, None),
                Some(semantic_ir::ModuleHeader::Shared { name, .. }) => (false, false, Some(name)),
                None => (false, false, None),
            };
            (
                has_other_statements,
                !module.callables.is_empty(),
                !module.dependencies.is_empty(),
                has_program_header,
                has_library_header,
                shared_decl,
                dim_vars,
            )
        });
    // Avoid requiring legacy-parser acceptance once the generated frontend
    // has supplied the shared-file header, statement classification, and
    // DIM declarations. Parse the compatibility AST only for that explicit
    // fallback.
    let program = if semantic_facts.is_none() {
        Some(parse_source(path.display().to_string(), &source)?)
    } else {
        None
    };
    let (
        has_other_statements,
        has_functions,
        has_dependencies,
        has_program_header,
        has_library_header,
        shared_decl,
        dim_vars,
    ) =
        semantic_facts.unwrap_or_else(|| {
            let program = program
                .as_ref()
                .expect("legacy shared-file fallback parsed its AST");
            let has_other_statements = program.statements.iter().any(|statement| match &statement
                .kind
            {
                ast::Statement::BlankLine
                | ast::Statement::BlockComment(_)
                | ast::Statement::Dim { .. } => false,
                ast::Statement::Raw(text) => !text.trim_start().starts_with('\''),
                _ => true,
            });
            let dim_vars = program
                .statements
                .iter()
                .filter_map(|statement| match &statement.kind {
                    ast::Statement::Dim { name, is_array, .. } => Some(ast::CommonVar {
                        name: name.clone(),
                        is_array: *is_array,
                    }),
                    _ => None,
                })
                .collect();
            (
                has_other_statements,
                !program.functions.is_empty(),
                !program.declarations.is_empty(),
                program.program_decl.is_some(),
                program.library_decl.is_some(),
                program.shared_decl.clone(),
                dim_vars,
            )
        });

    let pos = diagnostics::SourcePos::new(path.display().to_string(), 1, 1);
    let mut errors = Vec::new();

    // Every top-level `dim` becomes a CommonVar below -- a `shared <name>`
    // file's variables are COMMON by default, with no separate keyword to
    // opt in.
    if has_other_statements {
        errors.push(Diagnostic::error(
            pos.clone(),
            format!(
                "shared file `{}` may only contain DIM declarations (no other statements)",
                path.display()
            ),
        ));
    }
    if has_functions {
        errors.push(Diagnostic::error(
            pos.clone(),
            format!(
                "shared file `{}` may only contain DIM declarations (no functions)",
                path.display()
            ),
        ));
    }
    if has_dependencies {
        errors.push(Diagnostic::error(
            pos.clone(),
            format!(
                "shared file `{}` may only contain DIM declarations (no require/import)",
                path.display()
            ),
        ));
    }
    if has_program_header {
        errors.push(Diagnostic::error(
            pos.clone(),
            format!(
                "shared file `{}` may only contain DIM declarations (no program declaration)",
                path.display()
            ),
        ));
    }
    if has_library_header {
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
    match &shared_decl {
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
