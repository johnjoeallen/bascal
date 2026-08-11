pub mod ast;
pub mod codegen;
pub mod diagnostics;
pub mod lexer;
pub mod linker;
pub mod parser;
pub mod records;
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
    let program = records::lower(program)?;
    resolver::validate(&program)?;
    let conflicts = codegen::check_generated_name_conflicts(&program);
    if !conflicts.is_empty() {
        return Err(conflicts);
    }
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
            program.common = load_suite_file(&suite_path, &suite_name)?;
        }
        // Suite file not found → compile without COMMON (silent; suite may not exist yet).
    }

    let program = records::lower(program)?;
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
            suite_decl: None,
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

    if !program.common.is_empty() {
        errors.push(Diagnostic::error(
            diagnostics::SourcePos::new(input.display().to_string(), 1, 1),
            format!(
                "COMMON is only valid in suite files, not in `{}`",
                input.display()
            ),
        ));
    }

    if program.suite_decl.is_some() {
        errors.push(Diagnostic::error(
            diagnostics::SourcePos::new(input.display().to_string(), 1, 1),
            format!(
                "`suite` declaration is only valid in suite files, not in `{}`",
                input.display()
            ),
        ));
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    let mut merged = ast::Program {
        program_decl: program.program_decl,
        suite_decl: None,
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
                let dependency =
                    load_program_recursive(&dependency_path, false, options, visited)?;
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

fn load_suite_file(path: &Path, suite_name: &str) -> Result<Vec<ast::CommonBlock>, Vec<Diagnostic>> {
    let source = fs::read_to_string(path).map_err(|err| {
        vec![Diagnostic::error(
            diagnostics::SourcePos::new(path.display().to_string(), 1, 1),
            format!("failed to read suite file: {err}"),
        )]
    })?;
    let program = parse_source(path.display().to_string(), &source)?;

    let pos = diagnostics::SourcePos::new(path.display().to_string(), 1, 1);
    let mut errors = Vec::new();

    // `dim` is allowed alongside (or instead of) `common` -- each DIM'd name
    // becomes a CommonVar below, so a `suite <name>` header file can list its
    // shared variables the same way any other BASCAL file declares them.
    if program.statements.iter().any(|s| match s {
        ast::Statement::BlankLine | ast::Statement::BlockComment(_) | ast::Statement::Dim { .. } => false,
        ast::Statement::Raw(text) => !text.trim_start().starts_with('\''),
        _ => true,
    }) {
        errors.push(Diagnostic::error(
            pos.clone(),
            format!("suite file `{}` may only contain COMMON declarations and DIM statements (no other statements)", path.display()),
        ));
    }
    if !program.functions.is_empty() {
        errors.push(Diagnostic::error(
            pos.clone(),
            format!("suite file `{}` may only contain COMMON declarations and DIM statements (no functions)", path.display()),
        ));
    }
    if !program.declarations.is_empty() {
        errors.push(Diagnostic::error(
            pos.clone(),
            format!("suite file `{}` may only contain COMMON declarations and DIM statements (no require/import)", path.display()),
        ));
    }
    if program.program_decl.is_some() {
        errors.push(Diagnostic::error(
            pos.clone(),
            format!("suite file `{}` may only contain COMMON declarations and DIM statements (no program declaration)", path.display()),
        ));
    }
    // A `suite <name>` header is optional (the older filename-only
    // convention still works), but if present it must name the same suite
    // this file was actually resolved as -- catches a copy-pasted header
    // pointing at the wrong filename.
    if let Some(declared) = &program.suite_decl {
        if declared != suite_name {
            errors.push(Diagnostic::error(
                pos.clone(),
                format!(
                    "suite file `{}` declares `suite {declared}`, but was loaded as suite `{suite_name}` -- its filename must be `{declared}.bcl`",
                    path.display()
                ),
            ));
        }
    }

    // Every `dim name[()]` in the file becomes one more shared variable,
    // collected into a single trailing COMMON block (order matters for
    // CHAIN, not which statement -- common vs. dim -- declared it).
    let dim_vars: Vec<ast::CommonVar> = program
        .statements
        .iter()
        .filter_map(|s| match s {
            ast::Statement::Dim { name, is_array, .. } => Some(ast::CommonVar {
                name: name.clone(),
                is_array: *is_array,
            }),
            _ => None,
        })
        .collect();

    let mut common = program.common;
    if !dim_vars.is_empty() {
        common.push(ast::CommonBlock { vars: dim_vars });
    }

    if common.is_empty() {
        errors.push(Diagnostic::error(
            pos,
            format!("suite file `{}` contains no COMMON declarations or DIM statements", path.display()),
        ));
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(common)
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
        assert!(output.contains("add_left_0% = 10"));
        assert!(output.contains("add_right_0% = 20"));
        assert!(output.contains("GOSUB "));
        assert!(output.contains("total% = add_result_0%"));
        assert!(!output.contains("FN_add"));
        assert!(output.contains("add_result_0% = add_left_0% + add_right_0%"));
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
        assert!(output.contains("double_value_0% = 21"));
        assert!(output.contains("GOSUB "));
        assert!(!output.contains("FN_double"));
        assert!(output.contains("answer% = double_result_0%"));
        assert!(output.contains("double_result_0% = double_value_0% * 2"));
    }

    #[test]
    fn assigns_repeated_function_results_to_variables() {
        let source = include_str!("../tutorial/07_functions.bcl");
        let output = compile_source("tutorial/07_functions.bcl", source)
            .expect("sample should compile");

        // repeat$ is called twice; each result must be captured in a$ and b$ separately
        assert!(output.contains("GOSUB "));
        assert!(output.contains("a$ = repeat_result_0$"));
        assert!(output.contains("b$ = repeat_result_0$"));
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
        assert!(output.contains("GOTO 10"), "goto should still resolve to the label's line number");
    }

    #[test]
    fn function_local_const_declares_and_reads_the_same_lowered_name() {
        // A `const` declared inside a function/procedure body must be
        // renamed to its local BASIC name (like `dim`/assignment already
        // are) so the CONST statement and every later read of it agree --
        // otherwise the declaration binds one name while reads resolve to a
        // fresh, never-assigned local of a different name.
        let source = r#"procedure show()
    const n% = 5
    print n%
end procedure

show()
end
"#;
        let output = compile_source("const_local.bcl", source).expect("should compile");
        assert!(output.contains("CONST show_n_0% = 5"), "unexpected CONST line:\n{output}");
        assert!(
            output.contains("PRINT show_n_0%"),
            "PRINT should read back the same local name the CONST line declared:\n{output}"
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
        assert!(output.lines().any(|l| l.contains("bubblesort_data_0%(") && l.contains(") = bubbledata%(")));
        assert!(output.lines().any(|l| l.contains("bubbledata%(") && l.contains(") = bubblesort_data_0%(")));
        assert!(output.contains("bubblesort_data_0%(bubblesort_j_0%) = bubblesort_data_0%(bubblesort_j_0% + 1)"));
        assert!(output.contains("quicksort_data_0%(quicksort_wall_0%) = quicksort_data_0%(quicksort_qhigh_0%)"));
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
    fn suite_header_plus_dim_declares_common_vars() {
        let dir = tempfile::tempdir().unwrap();
        let prog_path = dir.path().join("show.bcl");
        let suite_path = dir.path().join("shared.bcl");

        std::fs::write(&prog_path, "program show suite shared\nprint count%\nend\n").unwrap();
        std::fs::write(
            &suite_path,
            "suite shared\n\ndim count%\ndim label$\ndim scores%()\n",
        )
        .unwrap();

        let output = compile_file(&prog_path, &CompileOptions::new())
            .expect("suite header + dim should compile");
        assert!(output.contains("COMMON count%, label$, scores%()"));
    }

    #[test]
    fn suite_header_name_must_match_the_filename_it_was_loaded_as() {
        let dir = tempfile::tempdir().unwrap();
        let prog_path = dir.path().join("show.bcl");
        let suite_path = dir.path().join("shared.bcl");

        std::fs::write(&prog_path, "program show suite shared\nend\n").unwrap();
        std::fs::write(&suite_path, "suite wrongname\n\ndim count%\n").unwrap();

        let result = compile_file(&prog_path, &CompileOptions::new());
        assert!(result.is_err());
        let msg = result.unwrap_err().into_iter().map(|d| d.to_string()).collect::<String>();
        assert!(msg.contains("declares `suite wrongname`"));
    }

    #[test]
    fn a_file_cannot_have_both_program_and_suite_declarations() {
        let source = "program foo\nsuite bar\nend\n";
        let result = compile_source("both.bcl", source);
        assert!(result.is_err());
        let msg = result.unwrap_err().into_iter().map(|d| d.to_string()).collect::<String>();
        assert!(msg.contains("cannot have both a `program` declaration and a `suite` declaration"));
    }

    #[test]
    fn suite_declaration_is_rejected_outside_a_suite_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("stray.bcl");
        std::fs::write(&path, "suite lonely\n\ndim x%\n").unwrap();

        let result = compile_file(&path, &CompileOptions::new());
        assert!(result.is_err());
        let msg = result.unwrap_err().into_iter().map(|d| d.to_string()).collect::<String>();
        assert!(msg.contains("`suite` declaration is only valid in suite files"));
    }

    #[test]
    fn dim_with_empty_parens_stays_an_array_in_generated_output() {
        // Regression test: `dim arr%()` used to collapse to the same
        // `DIM arr%` as a plain scalar `dim arr%`, because Statement::Dim
        // only tracked `sizes` (empty either way) with no separate signal
        // for "parens were written at all". Surfaced by the suite/dim work
        // above, since a suite file needs to tell scalar and unbounded-array
        // dims apart to emit `arr%()` vs `arr%` in the COMMON line -- fixed
        // by giving Statement::Dim its own `is_array` field.
        let source = "dim x%\ndim arr%()\nend\n";
        let output = compile_source("dimarr.bcl", source).expect("should compile");
        assert!(output.contains("DIM x%\n"));
        assert!(output.contains("DIM arr%()"));
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
        assert!(output.contains("FIELD #1, 2 AS db_idbuf$, 20 AS db_namebuf$, 8 AS db_scorebuf$"));
    }

    #[test]
    fn record_whole_write_lowers_to_lset_and_put() {
        let output = compile_source("rec.bcl", record_dsl_source()).expect("should compile");
        assert!(output.contains("LSET db_idbuf$ = MKI%(1)"));
        assert!(output.contains(r#"LSET db_namebuf$ = "Alice""#));
        assert!(output.contains("LSET db_scorebuf$ = MKD#(95)"));
        assert!(output.contains("PUT #1, 1"));
    }

    #[test]
    fn record_whole_read_lowers_to_get_and_unpack() {
        let output = compile_source("rec.bcl", record_dsl_source()).expect("should compile");
        assert!(output.contains("GET #1, i"));
        assert!(output.contains("s_id% = CVI%(db_idbuf$)"));
        assert!(output.contains("s_name$ = RTRIM$(db_namebuf$)"));
        assert!(output.contains("s_score# = CVD#(db_scorebuf$)"));
    }

    #[test]
    fn record_dotted_field_access_resolves_to_unpacked_scalar() {
        let output = compile_source("rec.bcl", record_dsl_source()).expect("should compile");
        // s.name is already a string — must not be STR$()-wrapped.
        assert!(output.contains("+ s_name$"));
        assert!(!output.contains("STR$(s_name$)"));
        // s.id / s.score are numeric and combined with strings via `+` — must be wrapped.
        assert!(output.contains("STR$(s_id%)"));
        assert!(output.contains("STR$(s_score#)"));
    }

    #[test]
    fn record_partial_update_lowers_to_get_lset_put() {
        let output = compile_source("rec.bcl", record_dsl_source()).expect("should compile");
        assert!(output.contains("GET #1, 2"));
        assert!(output.contains("LSET db_scorebuf$ = MKD#(61.5)"));
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
            output.contains("FIELD #1, 10 AS items_namebuf$, 2 AS items_qtybuf$"),
            "unexpected FIELD line:\n{output}"
        );
        assert!(
            output.contains("LSET items_namebuf$ = "),
            "addItem should LSET the top-level FIELD buffer, not a per-procedure local:\n{output}"
        );
        assert!(
            output.contains("LSET items_qtybuf$ = "),
            "addItem should LSET the top-level FIELD buffer, not a per-procedure local:\n{output}"
        );
        assert!(
            output.contains("RTRIM$(items_namebuf$)") && output.contains("CVI%(items_qtybuf$)"),
            "showItem should read back from the same top-level FIELD buffers:\n{output}"
        );
        assert!(
            !output.contains("additem_items_namebuf")
                && !output.contains("showitem_items_namebuf"),
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
        assert_eq!(output.matches("GET #1").count(), 1, "exactly one GET for the whole batch");
        assert_eq!(output.matches("PUT #1").count(), 1, "exactly one PUT for the whole batch");
        assert!(output.contains(r#"s_name$ = "Alicia""#), "field mutation is a plain in-memory assignment");
        assert!(output.contains("s_score# = 99"), "field mutation is a plain in-memory assignment");
        assert!(output.contains("LSET db_idbuf$ = MKI%(s_id%)"));
        assert!(output.contains("LSET db_namebuf$ = s_name$"));
        assert!(output.contains("LSET db_scorebuf$ = MKD#(s_score#)"));
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
        assert!(output.contains("GET #1, 2"), "partial write covering only some fields needs a GET");
        assert!(output.contains("LSET db_scorebuf$ = MKD#(61.5)"));
        assert!(!output.contains("LSET db_idbuf$"), "unmentioned fields must not be LSET");
        assert!(!output.contains("LSET db_namebuf$"), "unmentioned fields must not be LSET");
        assert!(output.contains("PUT #1, 2"));
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
        assert!(!output.contains("GET #1"), "covering every field needs no GET");
        assert!(output.contains("LSET db_idbuf$ = MKI%(3)"));
        assert!(output.contains("LSET db_namebuf$ = \"Carol\""));
        assert!(output.contains("LSET db_scorebuf$ = MKD#(78)"));
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
        let err = compile_source("full.bcl", source).expect_err("should reject incomplete full literal");
        assert!(err.iter().any(|d| d.message.contains("missing field `m`")));
        assert!(err.iter().any(|d| d.message.contains("?{")), "error should point at the partial alternative");
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
        let err = compile_source("bogus.bcl", source).expect_err("should reject unknown field even in a partial literal");
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
        let err = compile_source("bad.bcl", source).expect_err("should reject oversized string literal");
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
        let err = compile_source("bad.bcl", source).expect_err("should reject string for numeric field");
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
        let err = compile_source("bad.bcl", source).expect_err("should reject numeric for string field");
        assert!(err.iter().any(|d| d.message.contains("string(N)")));
    }

    #[test]
    fn record_rejects_unknown_record_type() {
        let source = r#"file db as Nope = open("a.dat")
end
"#;
        let err = compile_source("bad.bcl", source).expect_err("should reject unknown record type");
        assert!(err.iter().any(|d| d.message.contains("unknown record type")));
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
        assert!(err.iter().any(|d| d.message.contains("not a declared `file`")));
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
        assert!(output.contains("<> 0 THEN GOTO"), "loop while repeats when the condition is still true");
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
        assert!(output.contains("EXIT FOR"), "exit directly inside for must become EXIT FOR");
        // The exit inside the inner `do` must NOT have produced a second
        // EXIT FOR -- only exactly one EXIT FOR should appear anywhere.
        assert_eq!(output.matches("EXIT FOR").count(), 1);
        assert!(output.contains("GOTO"), "exit inside the inner do must become a GOTO, not EXIT FOR");
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
        assert!(!output.contains("else"), "else must not leak into the generated output");
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
        let source = "found% = TRUE\ndone% = FALSE\nif found% = TRUE then\n    print \"yes\"\nend if\nend\n";
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
        let if_line = output.lines().position(|l| l.contains("IF (")).expect("if line");
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
        assert!(!output.contains("skip"), "label text must not leak into generated output");
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
        // because compiler-internal labels use distinctive prefixed names
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
        assert!(output.contains("e% = 2 ^ (3 ^ 2)"));    // right-associative ^
        assert!(output.contains("f% = (10 \\ 3) MOD 2")); // \ binds tighter than MOD
    }

    #[test]
    fn local_names_always_use_indexed_scheme() {
        // All params, results, and locals always get an indexed suffix (_0, _1, …)
        // so they can never silently collide with a bare global name.
        // Global `foo_x%` must be distinct from parameter `x%` in function `foo%`.
        let source = r#"
foo_x% = 99
function foo%(x%)
  global foo_x%
  return x% + foo_x%
end function
print foo%(1)
end
"#;
        let output = compile_source("collision.bcl", source).expect("should compile");
        // Global must appear as-is.
        assert!(output.contains("foo_x%"), "global foo_x% must be present");
        // Parameter x% must be lowered to an indexed name, never the bare foo_x%.
        assert!(output.contains("foo_x_0%"), "param x% must use indexed name foo_x_0%");
        // The two names must be distinct — no line should assign foo_x% from foo_x%.
        assert!(!output.contains("foo_x% = foo_x%"), "names must not collide");
    }

    // ── generated-name conflict detection ─────────────────────────────────

    #[test]
    fn global_matching_generated_param_name_is_an_error() {
        // foo_x_0% is exactly what the compiler would generate for param x% in foo%.
        // Declaring it as a global must be rejected.
        let source = r#"
foo_x_0% = 99
function foo%(x%)
  return x% + 1
end function
print foo%(1)
end
"#;
        let err = compile_source("conflict_param.bcl", source)
            .expect_err("should reject global that conflicts with generated param name");
        assert!(
            err.iter().any(|d| d.message.contains("foo_x_0%")),
            "error must name the conflicting global: {:?}", err
        );
    }

    #[test]
    fn global_matching_generated_result_name_is_an_error() {
        // foo_result_0% is what the compiler generates for the result variable of foo%.
        let source = r#"
foo_result_0% = 0
function foo%(n%)
  return n% * 2
end function
print foo%(3)
end
"#;
        let err = compile_source("conflict_result.bcl", source)
            .expect_err("should reject global that conflicts with generated result name");
        assert!(
            err.iter().any(|d| d.message.contains("foo_result_0%")),
            "error must name the conflicting global: {:?}", err
        );
    }

    #[test]
    fn global_matching_generated_local_name_is_an_error() {
        // foo_acc_0% is what the compiler would generate for local acc% inside foo%.
        let source = r#"
foo_acc_0% = 0
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
            err.iter().any(|d| d.message.contains("foo_acc_0%")),
            "error must name the conflicting global: {:?}", err
        );
    }

    // ── short-circuit && / || ──────────────────────────────────────────

    /// Returns the label named by the first `THEN GOTO <label>` in `output`.
    fn first_goto_target(output: &str) -> &str {
        let idx = output.find("THEN GOTO ").expect("expected a THEN GOTO line");
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
        assert!(!output.contains("SC_"), "unexpected short-circuit label:\n{output}");
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
        assert_ne!(cont, exit, "continue and exit targets must differ:\n{output}");
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
        assert_ne!(cont, exit, "continue and exit targets must differ:\n{output}");
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
        assert!(!output.contains("SC_"), "unexpected continue label for simple inverted OR-chain:\n{output}");
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
        let first_guard = output.find("IF (a% > 0) = 0 THEN GOTO").expect("first guard");
        // Search for the actual GOSUB *statement* (line-initial), not the
        // word "GOSUB" that also appears in the generated header comment.
        let gosub = output.find("\nGOSUB ").expect("GOSUB statement for check%(b%) call");
        assert!(
            gosub > first_guard,
            "check%(b%)'s GOSUB must come after a%'s guard line, not before:\n{output}"
        );
    }
}
