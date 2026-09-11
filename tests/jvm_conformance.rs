//! Opt-in end-to-end conformance tests for the bootstrap JVM target.
//!
//! `krak2` assembles the Krakatau text emitted by `--target jvm`, and a JRE
//! runs the resulting class.  Neither tool is a Rust dependency, so this
//! suite follows the other external-tool suites and skips rather than fails
//! when a prerequisite is unavailable.
// Conformance groups: tutorials, jvm

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn java_available() -> bool {
    Command::new("java").arg("-version").output().is_ok()
}

fn krak2_available() -> bool {
    Command::new("krak2").arg("--help").output().is_ok()
}

fn jvm_runtime_available() -> bool {
    java_available() && krak2_available()
}

fn assert_jvm_expected_failure(source: &str, expected: &str) {
    let file = tempfile::Builder::new()
        .suffix(".bcl")
        .tempfile()
        .expect("failed to create JVM expected-failure fixture");
    fs::write(file.path(), source).expect("failed to write JVM expected-failure fixture");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(file.path())
        .arg("--target")
        .arg("jvm")
        .arg("--clean")
        .output()
        .expect("failed to invoke bcc");
    assert!(
        !output.status.success(),
        "JVM fixture unexpectedly succeeded"
    );
    let diagnostics = String::from_utf8_lossy(&output.stderr);
    assert!(
        diagnostics.contains(expected),
        "expected `{expected}` in diagnostics:\n{diagnostics}"
    );
}

#[test]
fn jvm_expected_failure_sequential_file_io_is_non_blocking() {
    assert_jvm_expected_failure(
        "program jvmSequentialFile\nopen \"missing.dat\" for input as #1\nend\n",
        "not supported by the minimal JVM backend yet",
    );
}

#[test]
fn jvm_expected_failure_random_record_io_is_non_blocking() {
    // A dynamic FIELD channel number is the one shape `--target jvm`'s
    // random-access record I/O still rejects (`FIELD`'s channel and every
    // field width must be literals -- see `JvmFieldVar`'s own doc comment
    // in codegen_jvm.rs); random-access I/O itself is supported, see
    // `jvm_random_access_file_round_trips_when_available` below.
    assert_jvm_expected_failure(
        "program jvmRandomFile\nch% = 1\nopen \"records.dat\" for random as #ch% len = 12\nfield #ch%, 12 as buf$\nend\n",
        "FIELD's channel number must be a literal under --target jvm",
    );
}

/// `tests/fixtures/conformance/cross_write.bcl` / `cross_read.bcl` (a
/// 2-byte `MKI$`-packed int field plus a 10-byte right-justified string
/// field) are the same fixture pair `dosbox_conformance.rs`'s
/// `c_target_random_access_file_is_binary_compatible_with_real_bascom_*`
/// checks against real BASCOM. No dosbox/real-BASCOM dependency here --
/// this just checks the JVM backend's own random-access I/O round-trips
/// (write, then read back in a *separate* process/class, proving the
/// on-disk bytes -- not just in-memory state -- carry the record) --
/// against a JVM-specific expectation, not `cross_read.expected.txt`
/// itself: real BASCOM/the C backend's `STR$` includes a leading sign
/// placeholder space for a non-negative number (`" 42"`); this backend's
/// `STR$` is a bare `String.valueOf(int)` with no such space (`"42"`), an
/// existing, unrelated gap this test deliberately doesn't paper over.
#[test]
fn jvm_random_access_file_round_trips_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: java or krak2 is unavailable", module_path!());
        return;
    }
    let work_dir = std::env::temp_dir().join("bascal-jvm-conformance-cross");
    let _ = fs::remove_dir_all(&work_dir);
    fs::create_dir_all(&work_dir).expect("failed to create work directory");

    let run = |fixture: &str| {
        let source_path = repo_root()
            .join("tests/fixtures/conformance")
            .join(format!("{fixture}.bcl"));
        let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
            .arg(&source_path)
            .arg("--target")
            .arg("jvm")
            .arg("--clean")
            .arg("--run")
            .arg("-o")
            .arg(work_dir.join("out/"))
            .current_dir(&work_dir)
            .output()
            .expect("failed to invoke bcc");
        assert!(
            output.status.success(),
            "{fixture} failed under --target jvm:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n")
    };

    run("cross_write");
    let actual = run("cross_read");
    assert!(
        actual.ends_with("42\n CrossTest\n"),
        "the JVM target's own random-access record read/write round-trip doesn't match:\n{actual}"
    );
}

/// `tutorial/random_and_record_files.bcl` exercises the full `MKx$`/`CVx$`
/// packing family (`MKI$`/`CVI` 16-bit int, `MKL$`/`CVL` 32-bit int,
/// `MKS$`/`CVS` 32-bit float, `MKD$`/`CVD` 64-bit double), both via raw
/// hand-written `FIELD` and via the record/file DSL, split across two
/// record types read/written from separate procedures. Checks the numeric
/// values themselves round-trip correctly (a whole-number double prints
/// with a trailing `.0` under this backend -- a cosmetic PRINT-formatting
/// difference from the BASIC/C backends, not a data bug, so the assertion
/// is on substrings rather than an exact match).
#[test]
fn jvm_random_and_record_files_tutorial_runs_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: java or krak2 is unavailable", module_path!());
        return;
    }
    let work_dir = std::env::temp_dir().join("bascal-jvm-conformance-random-and-record-files");
    let _ = fs::remove_dir_all(&work_dir);
    fs::create_dir_all(&work_dir).expect("failed to create work directory");

    let source_path = repo_root().join("tutorial/random_and_record_files.bcl");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("jvm")
        .arg("--clean")
        .arg("--run")
        .arg("-o")
        .arg(work_dir.join("out/"))
        .current_dir(&work_dir)
        .output()
        .expect("failed to invoke bcc");
    assert!(
        output.status.success(),
        "random_and_record_files.bcl failed under --target jvm:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n");
    for expected in [
        "[3] Carol -- 78",
        "[2] Bob -- 54",
        "[1] Alice -- 95",
        "Alice Smith: 91",
        "Bob: 61.5",
        "Carol Jones: 88",
    ] {
        assert_eq!(
            stdout.matches(expected).count(),
            2,
            "expected `{expected}` twice (hand-written FIELD, then the record/file DSL):\n{stdout}"
        );
    }
}

#[test]
fn jvm_expected_failure_mid_assignment_is_non_blocking() {
    assert_jvm_expected_failure(
        "program jvmMid\ntext$ = \"abcdef\"\nmid$(text$, 1, 2) = \"x\"\nend\n",
        "not supported by the minimal JVM backend yet",
    );
}

/// Regression test for a real, pre-existing bug: `COLOR`'s ANSI translation
/// used a naive `30 + fg % 8`, which only happens to agree with the correct
/// CGA-to-ANSI reorder table (see `ANSI_FG`/`ANSI_BG` in codegen_jvm.rs) for
/// green/cyan/white/black -- `COLOR 1` (blue) and `COLOR 4` (red) rendered
/// as each other outright. Checks against the exact codes `codegen_c.rs`'s
/// already-correct implementation produces for the same statements.
#[test]
fn jvm_color_uses_the_correct_cga_to_ansi_mapping_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: java or krak2 is unavailable", module_path!());
        return;
    }
    let source_path = repo_root().join("tutorial/screen.bcl");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("jvm")
        .arg("--clean")
        .arg("--run")
        .current_dir(repo_root())
        .output()
        .expect("failed to invoke bcc");
    assert!(
        output.status.success(),
        "screen.bcl failed under --target jvm:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    // color 14, 1 (bright yellow on blue), color 10 (bright green),
    // color 12 (bright red), color 11 (bright cyan) -- codegen_c.rs's own
    // bcc_ansi_fg/bcc_ansi_bg tables give exactly these codes for the same
    // CGA numbers.
    assert!(
        stdout.contains("\u{1b}[93;44m"),
        "expected bright-yellow-on-blue:\n{stdout}"
    );
    assert!(
        stdout.contains("\u{1b}[92m"),
        "expected bright green:\n{stdout}"
    );
    assert!(
        stdout.contains("\u{1b}[91m"),
        "expected bright red:\n{stdout}"
    );
    assert!(
        stdout.contains("\u{1b}[96m"),
        "expected bright cyan:\n{stdout}"
    );
}

/// `TAB(n)`/`SPC(n)` (print-position directives) and a dynamic (non-
/// literal) `LOCATE row%, col%` -- checked against the exact output
/// `codegen_c.rs`'s already-correct implementation produces for the same
/// statements.
#[test]
fn jvm_tab_spc_and_dynamic_locate_match_c_backend_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: java or krak2 is unavailable", module_path!());
        return;
    }
    let dir = tempfile::tempdir().expect("failed to create JVM TAB/SPC/LOCATE test directory");
    let source_path = dir.path().join("tab_spc_locate.bcl");
    fs::write(
        &source_path,
        "program tabSpcLocate\n\
         r% = 3\n\
         c% = 10\n\
         print \"Name\"; tab(20); \"Score\"\n\
         print spc(3); \"indented\"\n\
         locate r%, c%\n\
         print \"here\"\n\
         end\n",
    )
    .expect("failed to write TAB/SPC/LOCATE fixture");
    let mut output_dir = dir.path().join("out").into_os_string();
    output_dir.push("/");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("jvm")
        .arg("--clean")
        .arg("--run")
        .arg("-o")
        .arg(&output_dir)
        .current_dir(repo_root())
        .output()
        .expect("failed to invoke bcc");
    assert!(
        output.status.success(),
        "TAB/SPC/LOCATE fixture failed under --target jvm:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Name\u{1b}[20GScore"),
        "expected TAB:\n{stdout}"
    );
    assert!(stdout.contains("   indented"), "expected SPC:\n{stdout}");
    assert!(
        stdout.contains("\u{1b}[3;10Hhere"),
        "expected dynamic LOCATE:\n{stdout}"
    );
}

/// `DATE$` under `--target jvm`: `"MM-DD-YYYY"`, real MBASIC/BASCOM's own
/// fixed format, the exact same shape `codegen_c.rs`'s own `bcc_date`
/// produces. Can't check an exact value (today's actual date), so this just
/// checks the shape: two digits, a dash, two digits, a dash, four digits,
/// all numeric.
#[test]
fn jvm_date_dollar_matches_mm_dd_yyyy_format_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: java or krak2 is unavailable", module_path!());
        return;
    }
    let dir = tempfile::tempdir().expect("failed to create DATE$ fixture directory");
    let source_path = dir.path().join("date_dollar.bcl");
    fs::write(&source_path, "program dateDollar\nprint date$\nend\n")
        .expect("failed to write DATE$ fixture");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("jvm")
        .arg("--clean")
        .arg("--run")
        .current_dir(repo_root())
        .output()
        .expect("failed to invoke bcc");
    assert!(
        output.status.success(),
        "DATE$ fixture failed under --target jvm:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let value = stdout
        .lines()
        .next_back()
        .expect("expected at least one line of output");
    let parts: Vec<&str> = value.split('-').collect();
    assert_eq!(parts.len(), 3, "expected MM-DD-YYYY, got {value:?}");
    assert_eq!(parts[0].len(), 2, "expected 2-digit month, got {value:?}");
    assert_eq!(parts[1].len(), 2, "expected 2-digit day, got {value:?}");
    assert_eq!(parts[2].len(), 4, "expected 4-digit year, got {value:?}");
    assert!(
        parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit())),
        "expected only digits and dashes, got {value:?}"
    );
}

#[test]
fn jvm_try_catch_finally_runs_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: java or krak2 is unavailable", module_path!());
        return;
    }
    let source_path = repo_root().join("tests/fixtures/conformance/jvm_try.bcl");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(source_path)
        .arg("--target")
        .arg("jvm")
        .arg("--clean")
        .arg("--run")
        .current_dir(repo_root())
        .output()
        .expect("failed to invoke bcc");
    assert!(
        output.status.success(),
        "JVM try/catch fixture failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("caught 7"), "{stdout}");
    assert!(stdout.contains("finally"), "{stdout}");
}

#[test]
fn jvm_non_integer_arrays_run_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: java or krak2 is unavailable", module_path!());
        return;
    }
    let source_path = repo_root().join("tests/fixtures/jvm_noninteger_arrays.bcl");
    let temp_dir = tempfile::tempdir().expect("failed to create JVM array test directory");
    let mut output_arg = temp_dir.path().as_os_str().to_owned();
    output_arg.push("/");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("jvm")
        .arg("--clean")
        .arg("--run")
        .arg("-o")
        .arg(output_arg)
        .current_dir(repo_root())
        .output()
        .expect("failed to invoke bcc");
    assert!(
        output.status.success(),
        "JVM non-integer array fixture failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout)
        .replace("\r\n", "\n")
        .ends_with("2\n3\n1\n0\n2\n7\n4.0\nhello world\n9\n"));
}

#[test]
fn jvm_byval_arrays_expected_failure_is_non_blocking() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: java or krak2 is unavailable", module_path!());
        return;
    }
    let source_path = repo_root().join("tests/fixtures/jvm_byval_arrays.bcl");
    let temp_dir = tempfile::tempdir().expect("failed to create JVM byval array test directory");
    let mut output_arg = temp_dir.path().as_os_str().to_owned();
    output_arg.push("/");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("jvm")
        .arg("--clean")
        .arg("--run")
        .arg("-o")
        .arg(output_arg)
        .current_dir(repo_root())
        .output()
        .expect("failed to invoke bcc");
    assert!(
        !output.status.success(),
        "JVM byval array fixture unexpectedly succeeded"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("krak2 failed assembling"),
        "expected the known JVM byval clone assembly failure, got:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn jvm_catch_filters_and_source_bindings_run_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: java or krak2 is unavailable", module_path!());
        return;
    }
    let source_path = repo_root().join("tests/fixtures/conformance/jvm_try_filter.bcl");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(source_path)
        .arg("--target")
        .arg("jvm")
        .arg("--clean")
        .arg("--run")
        .current_dir(repo_root())
        .output()
        .expect("failed to invoke bcc");
    assert!(
        output.status.success(),
        "JVM catch filter fixture failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("caught 7"), "{stdout}");
    assert!(stdout.contains("jvm_try_filter.bcl"), "{stdout}");
    assert!(stdout.contains("finally"), "{stdout}");
}

#[test]
fn portable_error_handling_tutorial_runs_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: java or krak2 is unavailable", module_path!());
        return;
    }
    let source_path = repo_root().join("tutorial/portable_error_handling.bcl");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(source_path)
        .arg("--target")
        .arg("jvm")
        .arg("--clean")
        .arg("--run")
        .current_dir(repo_root())
        .output()
        .expect("failed to invoke bcc");
    assert!(
        output.status.success(),
        "portable error-handling tutorial failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("caught error 53"), "{stdout}");
    assert!(stdout.contains("portable_error_handling.bcl"), "{stdout}");
    assert!(stdout.contains("cleanup always runs"), "{stdout}");
}

/// Compile `source_path` to a temporary `.j` file and assemble it through
/// the CLI, so this exercises the same `krak2` configuration lookup users
/// get (`BASCAL_KRAK2`, config file, then PATH).  A missing assembler is a
/// skipped optional prerequisite; an assembler which rejects generated text
/// is a real test failure.
fn compile_and_assemble(source_path: &Path, output_dir: &Path) -> Option<PathBuf> {
    let mut output_arg = output_dir.as_os_str().to_owned();
    output_arg.push("/");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(source_path)
        .arg("--target")
        .arg("jvm")
        .arg("--clean")
        .arg("--binary")
        .arg("-o")
        .arg(output_arg)
        .current_dir(repo_root())
        .output()
        .expect("failed to invoke bcc");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("failed to invoke krak2") {
            eprintln!(
                "skipping {}: krak2 is unavailable -- install it with scripts/fetch-krak2.sh \
                 or configure BASCAL_KRAK2",
                module_path!()
            );
            return None;
        }
        panic!(
            "bcc failed to compile/assemble {} under --target jvm:\nstdout:\n{}\nstderr:\n{stderr}",
            source_path.display(),
            String::from_utf8_lossy(&output.stdout),
        );
    }

    Some(output_dir.join("hello.j"))
}

#[test]
fn hello_world_transpiles_assembles_and_runs_when_available() {
    if !java_available() {
        eprintln!(
            "skipping {}: java is not found on PATH -- install a JRE to run the JVM conformance suite",
            module_path!()
        );
        return;
    }

    let repo_root = repo_root();
    let source_path = repo_root.join("tutorial/hello.bcl");
    let expected_j_path = repo_root.join("tutorial/hello.j");
    let temp_dir = tempfile::tempdir().expect("failed to create JVM conformance temp directory");
    let output_dir = temp_dir.path().join("out");
    fs::create_dir(&output_dir)
        .unwrap_or_else(|err| panic!("failed to create {}: {err}", output_dir.display()));

    let Some(generated_j_path) = compile_and_assemble(&source_path, &output_dir) else {
        return;
    };

    let generated = fs::read_to_string(&generated_j_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", generated_j_path.display()));
    let expected = fs::read_to_string(&expected_j_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", expected_j_path.display()));
    assert_eq!(
        generated, expected,
        "checked-in JVM assembly fixture is stale"
    );

    let run = Command::new("java")
        .arg("-cp")
        .arg(repo_root.join("tmp"))
        .arg("Hello")
        .output()
        .expect("failed to run assembled Hello class");
    assert!(
        run.status.success(),
        "assembled Hello class failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"),
        "Hello, World!\nWelcome to BASCAL.\n"
    );
}

#[test]
fn numeric_literals_and_arithmetic_run_when_available() {
    if !java_available() {
        eprintln!(
            "skipping {}: java is not found on PATH -- install a JRE to run the JVM conformance suite",
            module_path!()
        );
        return;
    }

    let repo_root = repo_root();
    let source_path = repo_root.join("tests/fixtures/jvm_numeric.bcl");
    let expected_path = repo_root.join("tests/fixtures/jvm_numeric.expected.txt");
    let temp_dir = tempfile::tempdir().expect("failed to create JVM conformance temp directory");
    let output_dir = temp_dir.path().join("out");
    fs::create_dir(&output_dir)
        .unwrap_or_else(|err| panic!("failed to create {}: {err}", output_dir.display()));

    let Some(_) = compile_and_assemble(&source_path, &output_dir) else {
        return;
    };
    let expected = fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", expected_path.display()));
    let run = Command::new("java")
        .arg("-cp")
        .arg(repo_root.join("tmp"))
        .arg("Numeric")
        .output()
        .expect("failed to run assembled Numeric class");
    assert!(
        run.status.success(),
        "assembled Numeric class failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"),
        expected
    );
}

#[test]
fn scalar_variables_and_constants_run_when_available() {
    if !java_available() {
        eprintln!(
            "skipping {}: java is not found on PATH -- install a JRE to run the JVM conformance suite",
            module_path!()
        );
        return;
    }

    let repo_root = repo_root();
    let source_path = repo_root.join("tests/fixtures/jvm_variables.bcl");
    let expected_path = repo_root.join("tests/fixtures/jvm_variables.expected.txt");
    let temp_dir = tempfile::tempdir().expect("failed to create JVM conformance temp directory");
    let output_dir = temp_dir.path().join("out");
    fs::create_dir(&output_dir)
        .unwrap_or_else(|err| panic!("failed to create {}: {err}", output_dir.display()));

    let Some(_) = compile_and_assemble(&source_path, &output_dir) else {
        return;
    };
    let expected = fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", expected_path.display()));
    let run = Command::new("java")
        .arg("-cp")
        .arg(repo_root.join("tmp"))
        .arg("Variables")
        .output()
        .expect("failed to run assembled Variables class");
    assert!(
        run.status.success(),
        "assembled Variables class failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"),
        expected
    );
}

#[test]
fn structured_branches_and_while_loops_run_when_available() {
    if !java_available() {
        eprintln!(
            "skipping {}: java is not found on PATH -- install a JRE to run the JVM conformance suite",
            module_path!()
        );
        return;
    }

    let repo_root = repo_root();
    let source_path = repo_root.join("tests/fixtures/jvm_if.bcl");
    let expected_path = repo_root.join("tests/fixtures/jvm_if.expected.txt");
    let temp_dir = tempfile::tempdir().expect("failed to create JVM conformance temp directory");
    let output_dir = temp_dir.path().join("out");
    fs::create_dir(&output_dir)
        .unwrap_or_else(|err| panic!("failed to create {}: {err}", output_dir.display()));

    let Some(_) = compile_and_assemble(&source_path, &output_dir) else {
        return;
    };
    let expected = fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", expected_path.display()));
    let run = Command::new("java")
        .arg("-cp")
        .arg(repo_root.join("tmp"))
        .arg("Branching")
        .output()
        .expect("failed to run assembled Branching class");
    assert!(
        run.status.success(),
        "assembled Branching class failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"),
        expected
    );
}

#[test]
fn scalar_functions_run_when_available() {
    if !java_available() {
        return;
    }
    let repo_root = repo_root();
    let source_path = repo_root.join("tests/fixtures/jvm_functions.bcl");
    let expected_path = repo_root.join("tests/fixtures/jvm_functions.expected.txt");
    let temp_dir = tempfile::tempdir().expect("failed to create JVM conformance temp directory");
    let output_dir = temp_dir.path().join("out");
    fs::create_dir(&output_dir).expect("failed to create JVM output directory");
    if compile_and_assemble(&source_path, &output_dir).is_none() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(repo_root.join("tmp"))
        .arg("Functions")
        .output()
        .expect("failed to run assembled Functions class");
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"),
        fs::read_to_string(expected_path).expect("failed to read expected output")
    );
}

#[test]
fn scoped_goto_runs_when_available() {
    if !java_available() {
        return;
    }
    let root = repo_root();
    let temp_dir = tempfile::tempdir().expect("failed to create JVM conformance temp directory");
    let output_dir = temp_dir.path().join("out");
    fs::create_dir(&output_dir).expect("failed to create JVM output directory");
    let source = root.join("tests/fixtures/jvm_goto.bcl");
    if compile_and_assemble(&source, &output_dir).is_none() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("Labels")
        .output()
        .expect("failed to run assembled Labels class");
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "start\nfinish\n");
}

/// Regression test for a real, pre-existing gap: `collect_scalar_
/// declarations`/`collect_array_declarations`/`collect_global_names`/
/// `collect_labels` used to skip `select case` clause bodies entirely, so a
/// variable/array/label/`global` only ever appearing inside a `case` clause
/// silently failed to register -- surfaced by `examples/card_catalog/
/// card_catalog.bcl`'s own menu dispatch, where every branch is a `case`.
/// Needs no `java`/`krak2` -- transpiling (not running) already exercises
/// the fix.
#[test]
fn jvm_select_case_registers_variables_declared_only_inside_a_case_clause() {
    let file = tempfile::Builder::new()
        .suffix(".bcl")
        .tempfile()
        .expect("failed to create select-case fixture");
    fs::write(
        file.path(),
        "program selectVarTest\n\
         x% = 1\n\
         select case x%\n\
         \x20   case 1\n\
         \x20       y% = 5\n\
         \x20       print y%\n\
         \x20   case else\n\
         \x20       print \"no\"\n\
         end select\n\
         end\n",
    )
    .expect("failed to write select-case fixture");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(file.path())
        .arg("--target")
        .arg("jvm")
        .arg("--clean")
        .output()
        .expect("failed to invoke bcc");
    assert!(
        output.status.success(),
        "a variable declared only inside a `select case` clause should compile under \
         --target jvm:\nstderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Regression test for a real, pre-existing bug shared with `codegen_basic.
/// rs` (see `record_field_buffers_stay_global_when_the_file_open_is_wrapped_
/// in_try_catch` in lib.rs): `collect_field_vars`/`program_uses_random_open`/
/// `program_uses_input`/`collect_labels`/`collect_array_declarations`/
/// `collect_global_names` all recursed into `If`/`For`/`While`/`Do`/
/// `SelectCase` but not `TryCatch`, so a `FIELD`/`OPEN`/`INPUT`/label/array/
/// `global` inside a `try`/`catch`-wrapped `file ... = open(...)` (the
/// pattern `tutorial/inventory.bcl` uses to trap a real "can't open this
/// file" error) was invisible to every one of them. Needs no `java`/`krak2`
/// -- transpiling already exercises the fix.
#[test]
fn jvm_field_buffer_registers_when_the_file_open_is_wrapped_in_try_catch() {
    let file = tempfile::Builder::new()
        .suffix(".bcl")
        .tempfile()
        .expect("failed to create try/catch FIELD fixture");
    fs::write(
        file.path(),
        "program tryCatchField\n\
         record Item\n\
         \x20   name: string(10)\n\
         \x20   qty:  int16\n\
         end record\n\
         try\n\
         \x20   file items as Item = open(\"probe.dat\")\n\
         catch err%, erl%\n\
         \x20   print \"could not open\"\n\
         end try\n\
         items[1] = { name: \"widget\", qty: 5 }\n\
         let s = items[1]\n\
         print s.name + \" \" + str$(s.qty)\n\
         items.close()\n\
         end\n",
    )
    .expect("failed to write try/catch FIELD fixture");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(file.path())
        .arg("--target")
        .arg("jvm")
        .arg("--clean")
        .output()
        .expect("failed to invoke bcc");
    assert!(
        output.status.success(),
        "a FIELD declared via a try/catch-wrapped file open should compile under --target \
         jvm:\nstderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// `INSTR(s$, needle$)` (2-argument form) and `STOP`/`SYSTEM` (both compile
/// to `System.exit(0)`, usable from anywhere, unlike a plain `return` which
/// would only unwind one call frame). Needs no `java`/`krak2` -- transpiling
/// already exercises both.
#[test]
fn jvm_instr_and_stop_and_system_compile() {
    let file = tempfile::Builder::new()
        .suffix(".bcl")
        .tempfile()
        .expect("failed to create instr/stop/system fixture");
    fs::write(
        file.path(),
        "program instrStopSystemTest\n\
         kp$ = \"c\"\n\
         if instr(\"1234567cCeElLaAsSrRxX\", kp$) <> 0 then\n\
         \x20   print \"matched\"\n\
         \x20   stop\n\
         end if\n\
         system\n\
         end\n",
    )
    .expect("failed to write instr/stop/system fixture");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(file.path())
        .arg("--target")
        .arg("jvm")
        .arg("--clean")
        .output()
        .expect("failed to invoke bcc");
    assert!(
        output.status.success(),
        "INSTR/STOP/SYSTEM should compile under --target jvm:\nstderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// `INKEY$` under piped (non-tty) input -- the best this suite can automate
/// without a pty (the project takes no crate dependencies beyond `clap`, so
/// no pty crate either): `stty` legitimately fails against a pipe (see
/// `emit_run_command_inheriting_io`'s own doc comment in codegen_jvm.rs),
/// but the program must still not crash, and a byte written to the pipe
/// must still eventually become visible to `INKEY$`'s own `available()`
/// check. True raw-mode behavior (no Enter needed, no local echo) was
/// verified by hand against a real pseudo-terminal: a single byte with no
/// trailing newline was picked up immediately, with no echo -- not
/// reproducible here without a pty dependency this project doesn't take.
#[test]
fn jvm_inkey_polls_without_crashing_under_piped_input_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: java or krak2 is unavailable", module_path!());
        return;
    }
    let dir = tempfile::tempdir().expect("failed to create JVM INKEY$ test directory");
    let source_path = dir.path().join("inkey_poll.bcl");
    fs::write(
        &source_path,
        "program inkeyPoll\n\
         k$ = \"\"\n\
         do while k$ = \"\"\n\
         \x20   k$ = inkey$\n\
         loop\n\
         print \"got: \" + k$\n\
         end\n",
    )
    .expect("failed to write INKEY$ fixture");
    let mut output_dir = dir.path().join("out").into_os_string();
    output_dir.push("/");
    let status = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("jvm")
        .arg("--clean")
        .arg("--binary")
        .arg("-o")
        .arg(&output_dir)
        .current_dir(repo_root())
        .status()
        .expect("failed to invoke bcc");
    assert!(
        status.success(),
        "bcc failed to compile/assemble the INKEY$ fixture"
    );

    let mut child = Command::new("java")
        .arg("-cp")
        .arg(repo_root().join("tmp"))
        .arg("InkeyPoll")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn InkeyPoll");
    child
        .stdin
        .take()
        .expect("child stdin should be piped")
        .write_all(b"Q")
        .expect("failed to write keystroke to InkeyPoll");

    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if child
            .try_wait()
            .expect("failed to poll InkeyPoll")
            .is_some()
        {
            break;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("InkeyPoll timed out after 30 seconds");
        }
        thread::sleep(Duration::from_millis(50));
    }
    let run = child
        .wait_with_output()
        .expect("failed to collect InkeyPoll output");
    assert!(
        run.status.success(),
        "InkeyPoll exited non-zero:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "got: Q\n");
}

/// End-to-end confirmation that `examples/card_catalog/card_catalog.bcl` --
/// the flagship record/file DSL + procedures example -- runs correctly
/// under `--target jvm`: adds one entry through the interactive menu, then
/// lists it back, proving the random-access write (`addItem`/`PUT`) and
/// read (`listAll`/`GET`) round-trip through the real file, not just
/// in-memory state. Skipped (not failed) when `java`/`krak2` aren't
/// available, matching this file's other end-to-end tests.
#[test]
fn card_catalog_example_runs_under_jvm_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: java or krak2 is unavailable", module_path!());
        return;
    }
    let work_dir = std::env::temp_dir().join("bascal-jvm-conformance-card-catalog");
    let _ = fs::remove_dir_all(&work_dir);
    fs::create_dir_all(&work_dir).expect("failed to create work directory");

    let source_path = repo_root().join("examples/card_catalog/card_catalog.bcl");
    let status = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("jvm")
        .arg("--clean")
        .arg("--binary")
        .arg("-o")
        .arg(work_dir.join("out/"))
        .current_dir(&work_dir)
        .status()
        .expect("failed to invoke bcc");
    assert!(
        status.success(),
        "bcc failed to compile/assemble card_catalog.bcl under --target jvm"
    );

    let mut child = Command::new("java")
        .arg("-cp")
        .arg(work_dir.join("tmp"))
        .arg("CardCatalog")
        .current_dir(&work_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn CardCatalog");
    // 2 (new item), the three field prompts, 1 (list all), 6 (stop).
    child
        .stdin
        .take()
        .expect("child stdin should be piped")
        .write_all(b"2\nTwain\nHuck Finn\nFiction\n1\n6\n")
        .expect("failed to write keystrokes to CardCatalog");

    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if child
            .try_wait()
            .expect("failed to poll CardCatalog")
            .is_some()
        {
            break;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("CardCatalog timed out after 30 seconds");
        }
        thread::sleep(Duration::from_millis(50));
    }
    let run = child
        .wait_with_output()
        .expect("failed to collect CardCatalog output");
    assert!(
        run.status.success(),
        "CardCatalog exited non-zero:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    let stdout = String::from_utf8_lossy(&run.stdout);
    assert!(
        stdout.contains("Twain") && stdout.contains("Huck Finn") && stdout.contains("Fiction"),
        "expected the newly added entry to be listed back:\n{stdout}"
    );
}
