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
        let source = include_str!("../tutorial/sort_driver.bcl");
        let output =
            compile_source("tutorial/sort_driver.bcl", source).expect("sample should compile");
        assert!(output.contains("' require com.bascal.sort.bubbleSort"));
        // Without the sort library bubbleSort% is not in the symbol table;
        // it is emitted lowercase like any other user symbol, not uppercased.
        assert!(output.contains("bubblesort%(bubbledata%(), 5000)"));
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
        let source = include_str!("../tutorial/07_functions.bcl");
        let output = compile_source("tutorial/07_functions.bcl", source)
            .expect("sample should compile");

        // repeat$ is called twice; each result must be captured in a$ and b$ separately
        assert!(output.contains("GOSUB "));
        assert!(output.contains("a$ = repeat_result$"));
        assert!(output.contains("b$ = repeat_result$"));
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
        assert!(output.contains("GOSUB "), "should emit GOSUB for procedure call");
        assert!(!output.contains("greet_result"), "procedures must not emit a result variable");
        assert!(output.contains("' procedure greet("), "should annotate as procedure");
        assert!(output.contains("' end procedure greet"), "should close annotation as procedure");
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
        let output = compile_source("early.bcl", source).expect("procedure with return should compile");
        assert!(output.contains("RETURN"), "should emit RETURN");
        assert!(!output.contains("sayIfPositive_result"), "no result variable for procedure");
    }

    #[test]
    fn block_comment_preserves_internal_blank_lines() {
        let source = "/*\nFirst paragraph.\n\nSecond paragraph.\n*/\nEND\n";
        let output = compile_source("comment.bcl", source).expect("should compile");
        let lines: Vec<&str> = output.lines().collect();
        let first = lines.iter().position(|l| l.contains("First paragraph.")).unwrap();
        let second = lines.iter().position(|l| l.contains("Second paragraph.")).unwrap();
        assert!(second > first + 1, "blank line should separate the two comment paragraphs");
        assert!(lines[first + 1].trim().is_empty(), "line between paragraphs should be blank");
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
        assert!(output.contains("' function bubblesort%(data%, count%)"));
        assert!(output.contains("' function shellsort%(data%, count%)"));
        assert!(output.contains("' function touch%(value%)"));
        assert!(!output.contains("placeholder"));
        assert!(!output.contains("BCC_COPY%"), "hardcoded BCC_COPY% loop var should not appear");
        // sort_driver.bcl uses mixed-case `bubbleData%`; output normalises to lowercase.
        assert!(output.lines().any(|l| l.contains("bubblesort_data%(") && l.contains(") = bubbledata%(")));
        assert!(output.lines().any(|l| l.contains("bubbledata%(") && l.contains(") = bubblesort_data%(")));
        assert!(output.contains("bubblesort_data%(bubblesort_j%) = bubblesort_data%(bubblesort_j% + 1)"));
        assert!(output.contains("quicksort_data%(quicksort_wall%) = quicksort_data%(quicksort_qhigh%)"));
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
        // Mixed-case keywords and variable names: compiler normalises vars to lowercase.
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
        let output = compile_source("random.bcl", source).expect("random-access sample should compile");
        assert!(output.contains("OPEN datafile$ FOR RANDOM AS #1 LEN = 128"));
        assert!(output.contains("FIELD #1, 4 AS recnum$, 124 AS recdata$"));
        assert!(output.contains("LSET recnum$ = MKI%(1)"));
        assert!(output.contains("LSET recdata$ = \"hello\""));
        assert!(output.contains("PUT #1, 1"));
        assert!(output.contains("GET #1, 1"));
        assert!(output.contains("SEEK #1, 2"));
        assert!(output.contains("CLOSE #1"));
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
        let source = r#"' set and clear error trap
on error goto 9000
on error goto 0
' resume forms
resume
resume next
resume 9000
' trigger a synthetic error
error 53
end
"#;
        let output = compile_source("err.bcl", source).expect("should compile");
        assert!(output.contains("ON ERROR GOTO 9000"));
        assert!(output.contains("ON ERROR GOTO 0"));
        assert!(output.contains("RESUME\n") || output.ends_with("RESUME"));
        assert!(output.contains("RESUME NEXT"));
        assert!(output.contains("RESUME 9000"));
        assert!(output.contains("ERROR 53"));
    }

    #[test]
    fn mid_statement_form() {
        // MID$(str$, start[, length]) = replacement$ — in-place substring replace
        let source = r#"s$ = "Hello World"
mid$(s$, 7, 5) = "BASIC"
mid$(s$, 1) = "Goodbye"
print s$
end
"#;
        let output = compile_source("mid.bcl", source).expect("should compile");
        assert!(output.contains(r#"MID$(s$, 7, 5) = "BASIC""#));
        assert!(output.contains(r#"MID$(s$, 1) = "Goodbye""#));
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
        assert!(output.contains("e% = 2 ^ (3 ^ 2)"));    // right-associative ^
        assert!(output.contains("f% = (10 \\ 3) MOD 2")); // \ binds tighter than MOD
    }
}
