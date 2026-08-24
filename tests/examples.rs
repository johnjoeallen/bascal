use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn is_library_path(path: &Path) -> bool {
    path.components()
        .any(|c| matches!(c.as_os_str().to_str(), Some("com" | "lib")))
}

#[test]
fn compiles_every_example_bcl_file() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let tutorial_dir = repo_root.join("tutorial");
    let output_dir = repo_root.join("output");

    let mut examples: Vec<PathBuf> = collect_example_sources(&tutorial_dir)
        .into_iter()
        .filter(|path| !is_library_path(path))
        .collect();
    examples.sort();

    assert!(
        !examples.is_empty(),
        "expected at least one .bcl file in {}",
        tutorial_dir.display()
    );

    for example in examples {
        compile_example(&example, &tutorial_dir, &output_dir);
    }
}

#[test]
fn freebasic_runs_sort_driver_when_available() {
    if Command::new("fbc").arg("-version").output().is_err() {
        return;
    }

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_path = repo_root.join("tutorial/sort_driver.bcl");
    let output_path = repo_root.join("output/sort_driver.bas");

    compile_with_cli(&source_path, &output_path, &["--clean", "--binary"]);

    let executable_path = repo_root.join("tmp/sort_driver");
    let run = Command::new(&executable_path)
        .output()
        .expect("failed to run compiled sort driver");
    assert!(
        run.status.success(),
        "compiled sort driver failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let stdout = String::from_utf8_lossy(&run.stdout);
    for label in ["Bubble: OK", "Shaker: OK", "Shell: OK", "Quick: OK"] {
        assert_eq!(stdout.matches(label).count(), 1, "missing {label}");
    }
}

/// End-to-end confirmation for issue #10's Phase 2 acceptance criterion:
/// `tutorial/17_labels_and_error_handling.bcl` (GOTO/labels, GOSUB/RETURN,
/// `ON ERROR GOTO`/`RESUME`/`ERR`, and label-targeted `RESTORE`) compiles
/// *and runs correctly* under `--target c`, not just transpiles -- gcc-
/// compiled and executed here, the same bar `freebasic_runs_sort_driver_
/// when_available` already holds the `basic` target to via `fbc`. Skipped
/// (not failed) when `gcc` isn't available, matching that test's own
/// skip-if-unavailable convention.
#[test]
fn gcc_runs_labels_and_error_handling_tutorial_under_c_target_when_available() {
    if Command::new("gcc").arg("--version").output().is_err() {
        return;
    }

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_path = repo_root.join("tutorial/17_labels_and_error_handling.bcl");
    let output_dir = repo_root.join("output/c_target_error_handling");
    fs::create_dir_all(&output_dir)
        .unwrap_or_else(|err| panic!("failed to create {}: {err}", output_dir.display()));
    let mut dir_arg = output_dir.as_os_str().to_owned();
    dir_arg.push("/");

    let status = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("-o")
        .arg(&dir_arg)
        .arg("--target")
        .arg("C")
        .arg("--clean")
        .arg("--binary")
        .status()
        .expect("failed to invoke bcc");
    assert!(status.success(), "bcc failed to compile/build {source_path:?} under --target C");

    let executable_path = repo_root.join("tmp/17_labels_and_error_handling");
    let run = Command::new(&executable_path)
        .output()
        .expect("failed to run compiled labels_and_error_handling binary");
    assert!(
        run.status.success(),
        "compiled labels_and_error_handling binary failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let stdout = String::from_utf8_lossy(&run.stdout);
    for expected in [
        "reached via goto",
        "inside the gosub'd subroutine",
        "back after gosub",
        "caught error 53: does_not_exist.dat not found",
        "first read: France",
        "after restore secondBatch: Japan",
    ] {
        assert!(
            stdout.contains(expected),
            "missing {expected:?} in stdout:\n{stdout}"
        );
    }
}

/// End-to-end confirmation for GitHub issue #60's procedures-only C-target
/// propagation: a raise two call frames deep (inside `checkPart()`,
/// called by `editRecord()`, called from inside a `try`) reaches that
/// `try`'s own `catch`, correctly skipping the rest of both `checkPart()`
/// and `editRecord()` -- the scenario `--target basic` already handles for
/// free via real `ON ERROR GOTO`'s own global trap, and that `--target C`
/// needed `collect_try_reachable_procedures`'s `bcc_result_void`
/// propagation for (see codegen_c.rs). Skipped (not failed) when `gcc`
/// isn't available, matching this file's other C-target tests.
#[test]
fn gcc_runs_try_catch_through_nested_procedure_calls_under_c_target_when_available() {
    if Command::new("gcc").arg("--version").output().is_err() {
        return;
    }

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("nested_try.bcl");
    fs::write(
        &source_path,
        r#"program nestedTry

procedure checkPart()
    print "checking"
    error 11
    print "unreachable in checkPart"
end procedure

procedure editRecord()
    print "editing"
    checkPart()
    print "unreachable in editRecord"
end procedure

dim e%
dim l%

try
    editRecord()
catch e%, l%
    print "caught " + str$(e%) + " at " + str$(l%)
end try
print "after"
end
"#,
    )
    .unwrap();

    let output_dir = dir.path().join("out");
    fs::create_dir_all(&output_dir)
        .unwrap_or_else(|err| panic!("failed to create {}: {err}", output_dir.display()));
    let mut dir_arg = output_dir.as_os_str().to_owned();
    dir_arg.push("/");

    let status = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("-o")
        .arg(&dir_arg)
        .arg("--target")
        .arg("C")
        .arg("--clean")
        .arg("--binary")
        .current_dir(repo_root)
        .status()
        .expect("failed to invoke bcc");
    assert!(
        status.success(),
        "bcc failed to compile/build {source_path:?} under --target C"
    );

    let executable_path = repo_root.join("tmp/nested_try");
    let run = Command::new(&executable_path)
        .output()
        .expect("failed to run compiled nested_try binary");
    assert!(
        run.status.success(),
        "compiled nested_try binary failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let stdout = String::from_utf8_lossy(&run.stdout);
    assert!(stdout.contains("checking"), "{stdout}");
    assert!(stdout.contains("editing"), "{stdout}");
    assert!(!stdout.contains("unreachable"), "{stdout}");
    // `str$` matches real BASIC's own leading-space-for-non-negative
    // convention, so this is "caught " + " 11" + " at " + " 5".
    assert!(stdout.contains("caught  11 at  5"), "{stdout}");
    assert!(stdout.contains("after"), "{stdout}");
}

/// End-to-end confirmation that the real case-study program (issue #66's
/// try/catch migration) actually runs correctly under `--target C`, not
/// just compiles -- exercising the full chain landed to get it there:
/// try/catch propagation through checkPart()/editRecord()/etc. (#60),
/// `TAB`/`SPC`/`STOP`/`SYSTEM`, a real non-blocking `INKEY$` that doesn't
/// break `INPUT`'s own buffered reads on the same stdin (`bcc_inkey`'s
/// own doc comment), a top-level `const` correctly readable from inside
/// a procedure (`collect_top_level_const_c_names` -- `partCount%` was
/// reading as `0` inside `showMainMenu()`/`initializeInventoryFileIfNew()`
/// before that fix), and `initializeInventoryFileIfNew()` itself
/// (pre-populating a brand-new `inven.dat` so it doesn't have to be
/// supplied by hand). Skipped (not failed) when `gcc` isn't available,
/// matching this file's other C-target tests.
#[test]
fn gcc_runs_inventory_tutorial_under_c_target_when_available() {
    if Command::new("gcc").arg("--version").output().is_err() {
        return;
    }

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_path = repo_root.join("tutorial/inventory.bcl");
    let dir = tempfile::tempdir().unwrap();
    let output_dir = dir.path().join("out");
    fs::create_dir_all(&output_dir)
        .unwrap_or_else(|err| panic!("failed to create {}: {err}", output_dir.display()));
    let mut dir_arg = output_dir.as_os_str().to_owned();
    dir_arg.push("/");

    let status = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("-o")
        .arg(&dir_arg)
        .arg("--target")
        .arg("C")
        .arg("--clean")
        .arg("--binary")
        .current_dir(repo_root)
        .status()
        .expect("failed to invoke bcc");
    assert!(
        status.success(),
        "bcc failed to compile/build {source_path:?} under --target C"
    );

    let executable_path = repo_root.join("tmp/inventory");
    // Run from a fresh temp dir (not the repo root) so the freshly
    // written inven.dat lands there instead of polluting the repo.
    // Keystrokes: "1" selects "check a part" (INKEY$, one raw byte);
    // "1\n" is the part-number INPUT line (1 is still an empty slot,
    // since initializeInventoryFileIfNew() just populated it); "x"
    // dismisses the "press any key" prompt; "q" quits back out of the
    // main menu loop.
    let mut child = Command::new(&executable_path)
        .current_dir(dir.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn compiled inventory binary");
    child
        .stdin
        .take()
        .expect("child stdin should be piped")
        .write_all(b"11\nxq")
        .expect("failed to write keystrokes to inventory binary");
    let run = child
        .wait_with_output()
        .expect("failed to run compiled inventory binary");
    assert!(
        run.status.success(),
        "compiled inventory binary failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let stdout = String::from_utf8_lossy(&run.stdout);
    assert!(stdout.contains("Inventory Program"), "{stdout}");
    // `str$` matches real BASIC's own leading-space-for-non-negative
    // convention: "L)ist all" + " 100" + "parts".
    assert!(stdout.contains("L)ist all 100parts"), "{stdout}");
    assert!(stdout.contains("Input part number"), "{stdout}");
    assert!(
        stdout.contains("Part number 1is still a null entry at this time"),
        "{stdout}"
    );

    let inven_dat = dir.path().join("inven.dat");
    let metadata = fs::metadata(&inven_dat)
        .unwrap_or_else(|err| panic!("expected {} to exist: {err}", inven_dat.display()));
    // 100 records * 39 bytes each (1 flag + 30 desc + 2 qty + 2 reorder +
    // 4 price) -- confirms initializeInventoryFileIfNew() actually wrote
    // all 100 blank records, not zero (the very bug this test exists to
    // catch: partCount% reading as 0 inside the procedure made `for i% =
    // 1 to partCount%` a no-op, leaving inven.dat empty).
    assert_eq!(
        metadata.len(),
        3900,
        "inven.dat should hold 100 blank records"
    );
}

/// GitHub issue #29's own acceptance criterion: `LINE INPUT #` into a
/// `dim`'d string array element (`rawLine$(lineCount%)` in
/// `tutorial/remline/com/bascal/examples/remline/transform.bcl`) now
/// compiles and runs correctly under `--target c`, producing output
/// identical to `--target basic`'s own (see
/// `freebasic_runs_remline_when_available`'s matching assertion against
/// the same `tutorial/remline/sample/expected.bas` fixture).
#[test]
fn gcc_runs_remline_under_c_target_when_available() {
    if Command::new("gcc").arg("--version").output().is_err() {
        return;
    }

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_path = repo_root.join("tutorial/remline/remline.bcl");
    let sample_output_path = repo_root.join("tutorial/remline/sample/output.bas");
    let output_dir = repo_root.join("output/c_target_remline");
    fs::create_dir_all(&output_dir)
        .unwrap_or_else(|err| panic!("failed to create {}: {err}", output_dir.display()));
    let mut dir_arg = output_dir.as_os_str().to_owned();
    dir_arg.push("/");

    let _ = fs::remove_file(&sample_output_path);

    let status = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("-o")
        .arg(&dir_arg)
        .arg("--target")
        .arg("C")
        .arg("-L")
        .arg("tutorial/remline")
        .arg("--clean")
        .arg("--binary")
        .status()
        .expect("failed to invoke bcc");
    assert!(status.success(), "bcc failed to compile/build {source_path:?} under --target C");

    let executable_path = repo_root.join("tmp/remline");
    let run = Command::new(&executable_path)
        .output()
        .expect("failed to run compiled remline binary");
    assert!(
        run.status.success(),
        "compiled remline binary failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let expected = fs::read_to_string(repo_root.join("tutorial/remline/sample/expected.bas"))
        .expect("expected output should be readable");
    let actual = fs::read_to_string(&sample_output_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", sample_output_path.display()));
    assert_eq!(
        normalize_newlines(&actual),
        normalize_newlines(&expected),
        "remline output under --target c should match the sample expectation"
    );
}

#[test]
fn freebasic_runs_mid_assign_edge_cases_when_available() {
    if Command::new("fbc").arg("-version").output().is_err() {
        return;
    }

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_path = repo_root.join("tests/fixtures/mid_assign_edge_cases.bcl");
    let output_path = repo_root.join("output/mid_assign_edge_cases.bas");

    compile_with_cli(&source_path, &output_path, &["--clean", "--binary"]);

    let executable_path = repo_root.join("tmp/mid_assign_edge_cases");
    let run = Command::new(&executable_path)
        .output()
        .expect("failed to run compiled mid_assign_edge_cases");
    assert!(
        run.status.success(),
        "compiled mid_assign_edge_cases failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let stdout = String::from_utf8_lossy(&run.stdout);
    let lines: Vec<&str> = stdout.lines().map(str::trim).collect();
    assert_eq!(
        lines,
        vec![
            "01XY456789", // repl$ shorter than len: only LEN(repl$) chars overwritten
            "ABC3456789", // repl$ longer than len: truncated to len
            "0123456789", // repl$ empty: no-op, length preserved
            "012345678Z", // 2-arg form, pos at the very end of the string
        ],
        "MID$ assignment edge cases produced unexpected output:\n{stdout}"
    );
}

#[test]
fn freebasic_runs_self_referential_string_concatenation_when_available() {
    if Command::new("fbc").arg("-version").output().is_err() {
        return;
    }

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_path = repo_root.join("tests/fixtures/string_self_concat.bcl");
    let output_path = repo_root.join("output/string_self_concat.bas");

    compile_with_cli(&source_path, &output_path, &["--clean", "--binary"]);

    let executable_path = repo_root.join("tmp/string_self_concat");
    let run = Command::new(&executable_path)
        .output()
        .expect("failed to run compiled string_self_concat");
    assert!(
        run.status.success(),
        "compiled string_self_concat failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let stdout = String::from_utf8_lossy(&run.stdout);
    let lines: Vec<&str> = stdout.lines().map(str::trim).collect();
    assert_eq!(
        lines,
        vec!["abcdefghij", "abcdefgghij", "[ij]abcdefg", "fg-a-fghij"],
        "self-referential LEFT$/MID$/RIGHT$ concatenation produced unexpected output:\n{stdout}"
    );
}

/// Third leg of GitHub issue #38's own explicit ask (FreeBASIC, real
/// BASCOM 2.00, and C) -- the other two live in `tests/dosbox_conformance.rs`
/// (`builtin_scalar_methods_match_real_bascom`/`_match_c_target`), all three
/// checked against the same golden file.
#[test]
fn freebasic_runs_builtin_scalar_methods_when_available() {
    if Command::new("fbc").arg("-version").output().is_err() {
        return;
    }

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_path = repo_root.join("tests/fixtures/builtin_scalar_methods.bcl");
    let output_path = repo_root.join("output/builtin_scalar_methods.bas");

    compile_with_cli(&source_path, &output_path, &["--clean", "--binary"]);

    let executable_path = repo_root.join("tmp/builtin_scalar_methods");
    let run = Command::new(&executable_path)
        .output()
        .expect("failed to run compiled builtin_scalar_methods");
    assert!(
        run.status.success(),
        "compiled builtin_scalar_methods failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let expected = fs::read_to_string(
        repo_root.join("tests/fixtures/conformance/builtin_scalar_methods.expected.txt"),
    )
    .expect("expected output should be readable");
    let actual = String::from_utf8_lossy(&run.stdout);
    assert_eq!(
        normalize_newlines(&actual),
        normalize_newlines(&expected),
        "FreeBASIC output for built-in scalar methods should match the golden expectation"
    );
}

/// GitHub issue #41: third leg (FreeBASIC) for com.bascal.stdlib's
/// ltrim$/rtrim$/ucase$/lcase$, now scalar methods -- the other two live
/// in tests/dosbox_conformance.rs
/// (`stdlib_functions_match_real_bascom`/`_match_c_target`), all three
/// checked against the same golden file.
#[test]
fn freebasic_runs_stdlib_functions_when_available() {
    if Command::new("fbc").arg("-version").output().is_err() {
        return;
    }

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_path = repo_root.join("tests/fixtures/stdlib_functions.bcl");
    let output_path = repo_root.join("output/stdlib_functions.bas");

    compile_with_cli(&source_path, &output_path, &["--clean", "--binary"]);

    let executable_path = repo_root.join("tmp/stdlib_functions");
    let run = Command::new(&executable_path)
        .output()
        .expect("failed to run compiled stdlib_functions");
    assert!(
        run.status.success(),
        "compiled stdlib_functions failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let expected = fs::read_to_string(
        repo_root.join("tests/fixtures/conformance/stdlib_functions.expected.txt"),
    )
    .expect("expected output should be readable");
    let actual = String::from_utf8_lossy(&run.stdout);
    assert_eq!(
        normalize_newlines(&actual),
        normalize_newlines(&expected),
        "FreeBASIC output for com.bascal.stdlib should match the golden expectation"
    );
}

#[test]
fn freebasic_runs_remline_when_available() {
    if Command::new("fbc").arg("-version").output().is_err() {
        return;
    }

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_path = repo_root.join("tutorial/remline/remline.bcl");
    let output_path = repo_root.join("output/remline/remline.bas");
    let sample_output_path = repo_root.join("tutorial/remline/sample/output.bas");

    let _ = fs::remove_file(&sample_output_path);

    compile_with_cli(
        &source_path,
        &output_path,
        &["-L", "tutorial/remline", "--clean", "--binary"],
    );

    let executable_path = repo_root.join("tmp/remline");
    let run = Command::new(&executable_path)
        .output()
        .expect("failed to run compiled remline example");
    assert!(
        run.status.success(),
        "compiled remline example failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let expected = fs::read_to_string(repo_root.join("tutorial/remline/sample/expected.bas"))
        .expect("expected output should be readable");
    let actual = fs::read_to_string(&sample_output_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", sample_output_path.display()));
    assert_eq!(
        normalize_newlines(&actual),
        normalize_newlines(&expected),
        "remline output should match the sample expectation"
    );
}

fn collect_example_sources(dir: &Path) -> Vec<PathBuf> {
    let mut sources = Vec::new();
    collect_example_sources_recursive(dir, &mut sources);
    sources
}

fn collect_example_sources_recursive(dir: &Path, sources: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|err| {
        panic!("failed to read {}: {err}", dir.display());
    });

    for entry in entries {
        let path = entry
            .unwrap_or_else(|err| panic!("failed to read entry in {}: {err}", dir.display()))
            .path();

        if path.is_dir() {
            collect_example_sources_recursive(&path, sources);
            continue;
        }

        if path.extension().is_some_and(|extension| extension == "bcl") {
            sources.push(path);
        }
    }
}

fn compile_example(path: &Path, tutorial_dir: &Path, output_dir: &Path) {
    let mut options = bcc::CompileOptions::new();
    // Make any sibling `lib/` directory available as a search root.
    if let Some(parent) = path.parent() {
        let lib_dir = parent.join("lib");
        if lib_dir.is_dir() {
            options.library_dirs.push(lib_dir);
        }
    }
    let output = match bcc::compile_file(path, &options) {
        Ok(o) => o,
        Err(ref diagnostics)
            if diagnostics.iter().all(|d| {
                d.message
                    .contains("`shared` declaration is only valid in shared-variable files")
            }) =>
        {
            return; // shared-variables file — not a standalone compilable program
        }
        Err(diagnostics) => {
            panic!("failed to compile {}:\n{diagnostics:#?}", path.display())
        }
    };

    let output_path = output_path_for_source(path, tutorial_dir, output_dir);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .unwrap_or_else(|err| panic!("failed to create {}: {err}", parent.display()));
    }
    fs::write(&output_path, &output)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", output_path.display()));

    assert!(
        output.contains("' BASCAL generated BASIC\n"),
        "{} should produce generated BASIC",
        path.display()
    );
    assert!(
        output.lines().all(|line| {
            let trimmed = line.trim_start();
            trimmed.starts_with('\'')
                || (!trimmed.to_ascii_lowercase().starts_with("function ")
                    && !trimmed.to_ascii_lowercase().starts_with("end function"))
        }),
        "{} should not emit structured BASIC functions",
        path.display()
    );
    assert!(
        !output.contains("FN_") && !output.contains("IF_"),
        "{} should not expose symbolic labels",
        path.display()
    );
    assert_branch_targets_are_numeric(&output, path);
}

/// `output_path` names the exact file `bcc` is expected to produce --
/// `-o` itself only ever accepts a directory (auto-naming the file inside
/// it from the input's own stem), so this passes `output_path`'s parent
/// directory instead and relies on that auto-naming landing on exactly
/// `output_path`, which it always does here since every caller already
/// names it `<input stem>.bas`.
fn compile_with_cli(source_path: &Path, output_path: &Path, extra_args: &[&str]) {
    let parent = output_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)
        .unwrap_or_else(|err| panic!("failed to create {}: {err}", parent.display()));
    let mut dir_arg = parent.as_os_str().to_owned();
    dir_arg.push("/");

    let mut command = Command::new(env!("CARGO_BIN_EXE_bcc"));
    // Every current caller of this helper runs the result through `fbc`,
    // so it always needs BASIC output -- explicit rather than relying on
    // `bcc`'s own ambient default-target resolution (BASCAL_TARGET / a
    // dev's own ~/.config/bascal/config), which a machine set to `C` by
    // default would otherwise silently break this against.
    command
        .arg(source_path)
        .arg("-o")
        .arg(&dir_arg)
        .arg("--target")
        .arg("basic");
    for arg in extra_args {
        command.arg(arg);
    }

    let compile = command.output().expect("failed to run bcc");
    assert!(
        compile.status.success(),
        "bcc failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
}

fn output_path_for_source(source: &Path, tutorial_dir: &Path, output_dir: &Path) -> PathBuf {
    let relative = source
        .strip_prefix(tutorial_dir)
        .unwrap_or_else(|_| source.file_name().map(Path::new).unwrap_or(source));
    output_dir.join(relative).with_extension("bas")
}

fn assert_branch_targets_are_numeric(output: &str, path: &Path) {
    for line in output.lines() {
        if line_payload_is_comment(line) {
            continue;
        }
        let trimmed = line.trim_start();
        if let Some(target) = branch_target_after_keyword(trimmed, "GOTO") {
            assert!(
                target
                    .chars()
                    .next()
                    .is_some_and(|first| first.is_ascii_digit()),
                "{} should use numeric GOTO targets, got `{line}`",
                path.display()
            );
        }
        if let Some(target) = branch_target_after_keyword(trimmed, "GOSUB") {
            assert!(
                target
                    .chars()
                    .next()
                    .is_some_and(|first| first.is_ascii_digit()),
                "{} should use numeric GOSUB targets, got `{line}`",
                path.display()
            );
        }
    }
}

fn branch_target_after_keyword<'a>(line: &'a str, keyword: &str) -> Option<&'a str> {
    if line.starts_with(keyword) {
        return line.strip_prefix(keyword).map(str::trim_start);
    }

    if line.starts_with("IF ") {
        for marker in [format!(" THEN {keyword} "), format!(" THEN {keyword}\t")] {
            if let Some(index) = line.find(&marker) {
                return Some(line[index + marker.len()..].trim_start());
            }
        }
    }

    None
}

fn line_payload_is_comment(line: &str) -> bool {
    let payload = line
        .trim_start()
        .trim_start_matches(|ch: char| ch.is_ascii_digit())
        .trim_start();
    payload.starts_with('\'')
}

fn normalize_newlines(text: &str) -> String {
    text.replace("\r\n", "\n")
}
