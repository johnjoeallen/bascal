pub mod ast;
pub mod codegen;
pub mod diagnostics;
pub mod lexer;
pub mod linker;
pub mod parser;
pub mod resolver;

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use codegen::CodeGenerator;
use diagnostics::Diagnostic;
use lexer::Lexer;
use parser::Parser;

#[derive(Debug, Clone)]
pub struct CompileOptions {
    pub library_dirs: Vec<PathBuf>,
    pub libraries: Vec<String>,
    /// Number every output line (BASCOM strict mode). When false, only lines
    /// that are branch targets receive a line number.
    pub line_numbers: bool,
}

impl CompileOptions {
    pub fn new() -> Self {
        Self {
            library_dirs: Vec::new(),
            libraries: Vec::new(),
            line_numbers: false,
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
    resolver::validate(&program)?;
    Ok(CodeGenerator::new().generate(&program))
}

pub fn compile_file(input: &Path, options: &CompileOptions) -> Result<String, Vec<Diagnostic>> {
    let mut options = options.clone();
    if let Some(parent) = input.parent() {
        let parent = parent.to_path_buf();
        if !options.library_dirs.contains(&parent) {
            options.library_dirs.insert(0, parent);
        }
    }
    let options = &options;
    let mut visited = HashSet::new();
    let mut program = load_program_recursive(input, true, options, &mut visited)?;

    // Resolve suite COMMON block if the program declares a suite.
    if let Some(suite_name) = program
        .program_decl
        .as_ref()
        .and_then(|d| d.suite.as_deref())
        .map(str::to_string)
    {
        if let Some(suite_path) = resolve_suite_path(&suite_name, input, options) {
            program.common = load_suite_file(&suite_path)?;
        }
        // Suite file not found → compile without COMMON (silent; suite may not exist yet).
    }

    resolver::validate(&program)?;
    Ok(CodeGenerator::new()
        .with_line_numbers(options.line_numbers)
        .generate(&program))
}

pub fn default_output_path(input: &Path) -> std::path::PathBuf {
    input.with_extension("bas")
}

fn parse_source(filename: String, source: &str) -> Result<ast::Program, Vec<Diagnostic>> {
    let tokens = Lexer::new(&filename, source).lex();
    let mut parser = Parser::new(filename, tokens);
    parser.parse_program()
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
            declarations: Vec::new(),
            common: Vec::new(),
            statements: Vec::new(),
            functions: Vec::new(),
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

    if !program.common.is_empty() {
        errors.push(Diagnostic::error(
            diagnostics::SourcePos::new(input.display().to_string(), 1, 1),
            format!(
                "COMMON is only valid in suite files, not in `{}`",
                input.display()
            ),
        ));
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    let mut merged = ast::Program {
        program_decl: program.program_decl,
        declarations: Vec::new(),
        common: Vec::new(),
        statements: Vec::new(),
        functions: Vec::new(),
    };

    for declaration in &program.declarations {
        match declaration {
            ast::DependencyDecl::Require(symbol) | ast::DependencyDecl::Import(symbol) => {
                let dependency_path = resolve_required_symbol(&symbol.raw, &input, options)?;
                let dependency =
                    load_program_recursive(&dependency_path, false, options, visited)?;
                merged.statements.extend(dependency.statements);
                merged.functions.extend(dependency.functions);
            }
        }
    }

    merged.statements.extend(program.statements);
    merged.functions.extend(program.functions);
    Ok(merged)
}

fn load_suite_file(path: &Path) -> Result<Vec<ast::CommonBlock>, Vec<Diagnostic>> {
    let source = fs::read_to_string(path).map_err(|err| {
        vec![Diagnostic::error(
            diagnostics::SourcePos::new(path.display().to_string(), 1, 1),
            format!("failed to read suite file: {err}"),
        )]
    })?;
    let program = parse_source(path.display().to_string(), &source)?;

    let pos = diagnostics::SourcePos::new(path.display().to_string(), 1, 1);
    let mut errors = Vec::new();

    if program.statements.iter().any(|s| match s {
        ast::Statement::BlankLine | ast::Statement::BlockComment(_) => false,
        ast::Statement::Raw(text) => !text.trim_start().starts_with('\''),
        _ => true,
    }) {
        errors.push(Diagnostic::error(
            pos.clone(),
            format!("suite file `{}` may only contain COMMON declarations (no statements)", path.display()),
        ));
    }
    if !program.functions.is_empty() {
        errors.push(Diagnostic::error(
            pos.clone(),
            format!("suite file `{}` may only contain COMMON declarations (no functions)", path.display()),
        ));
    }
    if !program.declarations.is_empty() {
        errors.push(Diagnostic::error(
            pos.clone(),
            format!("suite file `{}` may only contain COMMON declarations (no require/import)", path.display()),
        ));
    }
    if program.program_decl.is_some() {
        errors.push(Diagnostic::error(
            pos.clone(),
            format!("suite file `{}` may only contain COMMON declarations (no program declaration)", path.display()),
        ));
    }
    if program.common.is_empty() {
        errors.push(Diagnostic::error(
            pos,
            format!("suite file `{}` contains no COMMON declarations", path.display()),
        ));
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(program.common)
}

fn resolve_suite_path(suite_name: &str, source_file: &Path, options: &CompileOptions) -> Option<PathBuf> {
    let filename = format!("{suite_name}.bcl");
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
    roots
}

fn normalize_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiles_sort_driver_sample() {
        let source = include_str!("../examples/sort_driver.bcl");
        let output =
            compile_source("examples/sort_driver.bcl", source).expect("sample should compile");
        assert!(output.contains("' require com.bascal.sort.bubbleSort"));
        assert!(output.contains("bubbleSort%(bubbleData%(), 5000)"));
        assert!(output.contains("END"));
    }

    #[test]
    fn lowers_functions_to_labels_and_gosub() {
        let source = r#"function add%(left%, right%)
    return left% + right%
end function

total% = add%(10, 20)
PRINT total%
END
"#;

        let output = compile_source("add.bcl", source).expect("sample should compile");
        assert!(output.contains("' function add%"), "spec comment should be emitted");
        assert!(!output.lines().any(|l| {
            let p = l.trim_start()
                .trim_start_matches(|c: char| c.is_ascii_digit())
                .trim_start();
            !p.starts_with('\'') && p.to_ascii_lowercase().contains("function ")
        }), "should not emit BASCOM function declarations");
        assert!(output.contains("' end function add%"), "end function comment should be emitted");
        assert!(!output.lines().any(|l| {
            let p = l.trim_start()
                .trim_start_matches(|c: char| c.is_ascii_digit())
                .trim_start();
            !p.starts_with('\'') && p.to_ascii_lowercase().starts_with("end function")
        }), "should not emit BASCOM end function declarations");
        assert!(output.contains("add_left% = 10"));
        assert!(output.contains("add_right% = 20"));
        assert!(output.contains("GOSUB "));
        assert!(output.contains("total% = add_result%"));
        assert!(!output.contains("FN_add"));
        assert!(output.contains("add_result% = add_left% + add_right%"));
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
        assert!(output.contains("double_value% = 21"));
        assert!(output.contains("GOSUB "));
        assert!(!output.contains("FN_double"));
        assert!(output.contains("answer% = double_result%"));
        assert!(output.contains("double_result% = double_value% * 2"));
    }

    #[test]
    fn assigns_repeated_function_results_to_variables() {
        let source = include_str!("../examples/repeated_function_result.bcl");
        let output = compile_source("examples/repeated_function_result.bcl", source)
            .expect("sample should compile");

        assert!(output.contains("GOSUB "));
        assert!(output.contains("a$ = x_result$"));
        assert!(output.contains("b$ = x_result$"));
        assert!(output.contains("x_result$ = \"result\""));
    }

    #[test]
    fn compile_file_recursively_includes_required_bcl_files() {
        let input = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/sort_driver.bcl");
        let output =
            compile_file(&input, &CompileOptions::new()).expect("sort driver should compile");

        assert!(!output.contains("' require com.bascal.sort.bubbleSort%"));
        assert!(output.contains("' Sort driver for the BASCAL example sort library."));
        assert!(output.contains("' In-place bubble sort."));
        assert!(output.contains("' function bubbleSort%(data%, count%)"));
        assert!(output.contains("' function shellSort%(data%, count%)"));
        assert!(output.contains("' function touch%(value%)"));
        assert!(!output.contains("placeholder"));
        assert!(!output.contains("BCC_COPY%"), "hardcoded BCC_COPY% loop var should not appear");
        assert!(output.lines().any(|l| l.contains("bubblesort_data%(") && l.contains(") = bubbleData%(")));
        assert!(output.lines().any(|l| l.contains("bubbleData%(") && l.contains(") = bubblesort_data%(")));
        assert!(output.contains("bubblesort_data%(j%) = bubblesort_data%(j% + 1)"));
        assert!(output.contains("quicksort_data%(wall%) = quicksort_data%(qHigh%)"));
        assert!(output.contains("GOSUB "));
    }

    #[test]
    fn program_suite_loads_common_block() {
        let dir = tempfile::tempdir().unwrap();
        let suite_path = dir.path().join("myapp.bcl");
        let common_path = dir.path().join("mysuite.bcl");

        std::fs::write(&suite_path, "program myapp suite mysuite\nPRINT \"hello\"\nEND\n").unwrap();
        std::fs::write(&common_path, "common score%, level%\ncommon name$\n").unwrap();

        let output = compile_file(&suite_path, &CompileOptions::new())
            .expect("program with suite should compile");

        assert!(output.contains("COMMON score%, level%"));
        assert!(output.contains("COMMON name$"));
        assert!(output.contains("PRINT \"hello\""));
    }

    #[test]
    fn common_in_non_suite_file_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.bcl");
        std::fs::write(&path, "common score%\nPRINT 1\nEND\n").unwrap();

        let result = compile_file(&path, &CompileOptions::new());
        assert!(result.is_err());
        let msg = result.unwrap_err().into_iter().map(|d| d.to_string()).collect::<String>();
        assert!(msg.contains("COMMON is only valid in suite files"));
    }

    #[test]
    fn suite_file_with_statements_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let suite_path = dir.path().join("prog.bcl");
        let bad_suite = dir.path().join("badcommon.bcl");

        std::fs::write(&suite_path, "program prog suite badcommon\nEND\n").unwrap();
        std::fs::write(&bad_suite, "common score%\nPRINT 1\n").unwrap();

        let result = compile_file(&suite_path, &CompileOptions::new());
        assert!(result.is_err());
        let msg = result.unwrap_err().into_iter().map(|d| d.to_string()).collect::<String>();
        assert!(msg.contains("may only contain COMMON declarations"));
    }

    #[test]
    fn lowers_basic_file_io_statements() {
        let source = r#"open inputFile$ for input as #1
line input #1, line$
print #2, line$
close #1
END
"#;

        let output = compile_source("io.bcl", source).expect("sample should compile");
        assert!(output.contains("OPEN inputFile$ FOR INPUT AS #1"));
        assert!(output.contains("LINE INPUT #1, line$"));
        assert!(output.contains("PRINT #2, line$"));
        assert!(output.contains("CLOSE #1"));
    }
}
