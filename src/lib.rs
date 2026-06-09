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
    pub include_dirs: Vec<PathBuf>,
    pub library_dirs: Vec<PathBuf>,
    pub libraries: Vec<String>,
}

impl CompileOptions {
    pub fn new() -> Self {
        Self {
            include_dirs: Vec::new(),
            library_dirs: Vec::new(),
            libraries: Vec::new(),
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
        options.include_dirs.insert(0, parent.to_path_buf());
    }
    let mut visited = HashSet::new();
    let program = load_program_recursive(input, &options, &mut visited)?;
    resolver::validate(&program)?;
    Ok(CodeGenerator::new().generate(&program))
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
    options: &CompileOptions,
    visited: &mut HashSet<PathBuf>,
) -> Result<ast::Program, Vec<Diagnostic>> {
    let input = normalize_path(input);
    if !visited.insert(input.clone()) {
        return Ok(ast::Program {
            declarations: Vec::new(),
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
    let mut merged = ast::Program {
        declarations: Vec::new(),
        statements: Vec::new(),
        functions: Vec::new(),
    };

    for declaration in &program.declarations {
        match declaration {
            ast::DependencyDecl::Require(symbol) | ast::DependencyDecl::Import(symbol) => {
                let dependency_path = resolve_required_symbol(&symbol.raw, &input, options)?;
                let dependency = load_program_recursive(&dependency_path, options, visited)?;
                merged.statements.extend(dependency.statements);
                merged.functions.extend(dependency.functions);
            }
            ast::DependencyDecl::Include(path) => {
                let dependency_path = resolve_include_path(path, &input, options)?;
                let dependency = load_program_recursive(&dependency_path, options, visited)?;
                merged.statements.extend(dependency.statements);
                merged.functions.extend(dependency.functions);
            }
        }
    }

    merged.statements.extend(program.statements);
    merged.functions.extend(program.functions);
    Ok(merged)
}

fn resolve_required_symbol(
    raw: &str,
    source_file: &Path,
    options: &CompileOptions,
) -> Result<PathBuf, Vec<Diagnostic>> {
    let relative = required_symbol_to_path(raw);
    let mut roots = search_roots(source_file, options);
    roots.extend(options.library_dirs.iter().cloned());

    for root in roots {
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

fn resolve_include_path(
    raw: &str,
    source_file: &Path,
    options: &CompileOptions,
) -> Result<PathBuf, Vec<Diagnostic>> {
    let include = PathBuf::from(raw);
    let roots = search_roots(source_file, options);
    for root in roots {
        let candidate = root.join(&include);
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    Err(vec![Diagnostic::error(
        diagnostics::SourcePos::new(source_file.display().to_string(), 1, 1),
        format!("failed to resolve include `{raw}`"),
    )])
}

fn required_symbol_to_path(raw: &str) -> PathBuf {
    let mut symbol = raw.to_string();
    if symbol
        .chars()
        .last()
        .is_some_and(|suffix| ast::TypeSuffix::from_char(suffix).is_some())
    {
        symbol.pop();
    }

    let mut path = symbol.split('.').collect::<PathBuf>();
    path.set_extension("bcl");
    path
}

fn search_roots(source_file: &Path, options: &CompileOptions) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(parent) = source_file.parent() {
        roots.push(parent.to_path_buf());
    }
    roots.extend(options.include_dirs.iter().cloned());
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
        assert!(output.contains("' require com.bascal.sort.bubbleSort%"));
        assert!(output.contains("bubbleSort%(bubbleData%(), 10)"));
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
        assert!(!output.contains("function add%"));
        assert!(!output.contains("end function"));
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
        assert!(output.contains("' ===== BEGIN FUNCTION bubbleSort% ====="));
        assert!(output.contains("' ===== BEGIN FUNCTION shellSort% ====="));
        assert!(output.contains("' ===== BEGIN FUNCTION touch% ====="));
        assert!(!output.contains("placeholder"));
        assert!(output.contains("bubblesort_data%(BCC_COPY%) = bubbleData%(BCC_COPY%)"));
        assert!(output.contains("bubbleData%(BCC_COPY%) = bubblesort_data%(BCC_COPY%)"));
        assert!(output.contains("bubblesort_data%(j%) = bubblesort_data%(j% + 1)"));
        assert!(output.contains("quicksort_data%(min%) = temp%"));
        assert!(output.contains("GOSUB "));
    }
}
