// Conformance suite: compiles BASCAL fixtures and runs the generated BASIC
// through a *real* IBM Personal Computer BASIC Compiler 2.00 ("BASCOM"),
// under dosbox-x, then diffs its behavior against checked-in golden output.
//
// This is opt-in and local by default. It requires:
//   1. `dosbox-x` on PATH.
//   2. `test-fixtures/ibm-basic-compiler/c_drive/` populated by running
//      `scripts/fetch-ibm-basic-compiler.sh` (never committed -- see
//      test-fixtures/README.md).
//
// If either is missing, every test here prints a one-line explanation and
// returns early instead of failing -- see `skip_reason()`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture_c_drive() -> PathBuf {
    repo_root().join("test-fixtures/ibm-basic-compiler/c_drive")
}

fn dosbox_x_available() -> bool {
    // dosbox-x exits non-zero for `--version` (it treats the flag as an
    // early-exit trigger rather than a clean "print and succeed" query), so
    // just check that the binary runs at all rather than its exit status.
    Command::new("dosbox-x").arg("--version").output().is_ok()
}

fn ibm_basic_compiler_available() -> bool {
    fixture_c_drive().join("BASCOM.EXE").is_file()
}

/// Returns `Some(reason)` if the conformance suite should be skipped, or
/// `None` if both dosbox-x and the compiler fixture are ready to use.
fn skip_reason() -> Option<&'static str> {
    match (dosbox_x_available(), ibm_basic_compiler_available()) {
        (true, true) => None,
        (false, true) => Some(
            "dosbox-x not found on PATH -- install it to run the real-BASCOM \
             conformance suite (see CONTRIBUTING.md)",
        ),
        (true, false) => Some(
            "test-fixtures/ibm-basic-compiler/ not populated -- run \
             scripts/fetch-ibm-basic-compiler.sh to enable the real-BASCOM \
             conformance suite (see test-fixtures/README.md)",
        ),
        (false, false) => Some(
            "dosbox-x not found on PATH and test-fixtures/ibm-basic-compiler/ \
             not populated -- see CONTRIBUTING.md to enable the real-BASCOM \
             conformance suite",
        ),
    }
}

macro_rules! require_fixture {
    () => {
        if let Some(reason) = skip_reason() {
            eprintln!("skipping {}: {reason}", module_path!());
            return;
        }
    };
}

/// A DOS text file needs CRLF line endings and a trailing Ctrl-Z (0x1A) EOF
/// marker to be read correctly by real DOS-era tools.
fn to_dos_text(text: &str) -> Vec<u8> {
    let mut dos = text.replace('\n', "\r\n");
    if !dos.ends_with("\r\n") {
        dos.push_str("\r\n");
    }
    let mut bytes = dos.into_bytes();
    bytes.push(0x1A);
    bytes
}

fn normalize(text: &str) -> String {
    text.replace("\r\n", "\n")
}

/// Runs dosbox-x headlessly against `work_dir`, mounted as `C:`, executing
/// `batch_file` (a filename inside `work_dir`) and exiting immediately
/// after. Real BASCOM/LINK invocations here take a few seconds each under
/// emulation, so this generous timeout is for the whole batch, not per
/// command.
fn run_dosbox_batch(work_dir: &Path, batch_file: &str) {
    let mount_arg = format!("MOUNT C: {}", work_dir.display());
    let output = Command::new("dosbox-x")
        .arg("-c")
        .arg(&mount_arg)
        .arg("-c")
        .arg("C:")
        .arg("-c")
        .arg(batch_file)
        .arg("-fastlaunch")
        .arg("-exit")
        .output()
        .expect("failed to invoke dosbox-x");
    assert!(
        output.status.success(),
        "dosbox-x exited with {}:\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn stage_compiler_fixture(work_dir: &Path) {
    fs::create_dir_all(work_dir).expect("failed to create dosbox-x work directory");
    for entry in fs::read_dir(fixture_c_drive()).expect("failed to read compiler fixture") {
        let entry = entry.expect("failed to read compiler fixture entry");
        let dest = work_dir.join(entry.file_name());
        fs::copy(entry.path(), dest).expect("failed to stage compiler fixture file");
    }
}

fn write_dos_file(path: &Path, contents: &str) {
    fs::write(path, to_dos_text(contents)).unwrap_or_else(|err| {
        panic!("failed to write {}: {err}", path.display());
    });
}

/// Compiles `fixture_name.bcl` with the current `bcc`, runs it through real
/// BASCOM, and asserts a clean compile (0 severe errors) plus a link. Runs
/// the resulting .EXE and returns its captured stdout.
fn compile_link_run(fixture_name: &str) -> String {
    let fixture_bcl = repo_root()
        .join("tests/fixtures/conformance")
        .join(format!("{fixture_name}.bcl"));
    // Real BASCOM has no notion of an unnumbered statement line -- full
    // line numbering (bcc's own CLI default) is required here, not the
    // library's sparse-by-default CodeGenerator.
    let options = bcc::CompileOptions {
        line_numbers: true,
        ..bcc::CompileOptions::new()
    };
    let basic = bcc::compile_file(&fixture_bcl, &options).unwrap_or_else(|diagnostics| {
        panic!("failed to compile {}: {diagnostics:#?}", fixture_bcl.display())
    });

    let work_dir = std::env::temp_dir().join(format!("bascal-conformance-{fixture_name}"));
    let _ = fs::remove_dir_all(&work_dir);
    stage_compiler_fixture(&work_dir);

    // 8.3-safe DOS name, independent of the fixture's own (possibly longer)
    // file stem.
    let dos_stem = "TEST";
    write_dos_file(&work_dir.join(format!("{dos_stem}.BAS")), &basic);
    write_dos_file(
        &work_dir.join("RUNIT.BAT"),
        &format!(
            "BASCOM {dos_stem}.BAS,,;\r\nLINK {dos_stem}.OBJ;\r\n{dos_stem}.EXE > OUT.TXT\r\nEXIT\r\n"
        ),
    );

    run_dosbox_batch(&work_dir, "RUNIT.BAT");

    let lst_path = work_dir.join(format!("{dos_stem}.LST"));
    let lst = fs::read_to_string(&lst_path).unwrap_or_else(|err| {
        panic!(
            "expected BASCOM to produce {} (BASCOM compile failed to run at all?): {err}",
            lst_path.display()
        )
    });
    assert!(
        lst.contains("0 Severe  Error(s)"),
        "real BASCOM rejected the generated BASIC for `{fixture_name}`:\n{lst}"
    );

    let exe_path = work_dir.join(format!("{dos_stem}.EXE"));
    assert!(
        exe_path.is_file(),
        "expected BASCOM+LINK to produce {} -- compile succeeded but link \
         must have failed",
        exe_path.display()
    );

    let out_path = work_dir.join("OUT.TXT");
    fs::read_to_string(&out_path).unwrap_or_else(|err| {
        panic!(
            "expected the compiled program to write {}: {err}",
            out_path.display()
        )
    })
}

#[test]
fn const_and_print_matches_real_bascom() {
    require_fixture!();

    let actual = compile_link_run("const_and_print");

    let expected_path = repo_root().join("tests/fixtures/conformance/const_and_print.expected.txt");
    let expected = fs::read_to_string(&expected_path).unwrap_or_else(|err| {
        panic!("failed to read {}: {err}", expected_path.display())
    });

    assert_eq!(
        normalize(&actual),
        normalize(&expected),
        "real BASCOM's output for const_and_print.bcl no longer matches the \
         golden expectation at {}",
        expected_path.display()
    );
}

#[test]
fn mid_assign_matches_real_bascom() {
    require_fixture!();

    let actual = compile_link_run("mid_assign");

    let expected_path = repo_root().join("tests/fixtures/conformance/mid_assign.expected.txt");
    let expected = fs::read_to_string(&expected_path).unwrap_or_else(|err| {
        panic!("failed to read {}: {err}", expected_path.display())
    });

    assert_eq!(
        normalize(&actual),
        normalize(&expected),
        "real BASCOM's output for MID$ statement-form assignment no longer \
         matches the golden expectation at {}",
        expected_path.display()
    );
}

#[test]
fn stdlib_functions_match_real_bascom() {
    require_fixture!();

    let actual = compile_link_run("stdlib_functions");

    let expected_path = repo_root().join("tests/fixtures/conformance/stdlib_functions.expected.txt");
    let expected = fs::read_to_string(&expected_path).unwrap_or_else(|err| {
        panic!("failed to read {}: {err}", expected_path.display())
    });

    assert_eq!(
        normalize(&actual),
        normalize(&expected),
        "real BASCOM's output for com.bascal.stdlib no longer matches the \
         golden expectation at {}",
        expected_path.display()
    );
}
