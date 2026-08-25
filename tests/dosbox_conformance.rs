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
        // The conformance suite neither displays nor plays anything.  More
        // importantly, letting SDL select the host audio backend can leave
        // DOSBox-X blocked indefinitely while trying to connect to
        // PulseAudio (common in containers and headless CI).
        .env("SDL_AUDIODRIVER", "dummy")
        .arg("-nogui")
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
        panic!(
            "failed to compile {}: {diagnostics:#?}",
            fixture_bcl.display()
        )
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
    let expected = fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", expected_path.display()));

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
    let expected = fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", expected_path.display()));

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

    let expected_path =
        repo_root().join("tests/fixtures/conformance/stdlib_functions.expected.txt");
    let expected = fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", expected_path.display()));

    assert_eq!(
        normalize(&actual),
        normalize(&expected),
        "real BASCOM's output for com.bascal.stdlib no longer matches the \
         golden expectation at {}",
        expected_path.display()
    );
}

/// GitHub issue #41: com.bascal.stdlib's ltrim$/rtrim$/ucase$/lcase$ are
/// now declared as scalar methods, with error$ staying an ordinary
/// function -- `stdlib_functions.bcl` exercises both the ordinary-call
/// form (which now resolves to the method declaration, filling self$ with
/// the first argument -- see `records::Lowerer::try_ordinary_call_as_method`)
/// and the chained method-call form, checked against the same golden file
/// `stdlib_functions_match_real_bascom` already uses.
#[test]
fn stdlib_functions_match_c_target() {
    let work_dir = std::env::temp_dir().join("bascal-c-target-stdlib-functions");
    let _ = fs::remove_dir_all(&work_dir);
    let actual = compile_run_c_target_in("stdlib_functions", &work_dir);
    let expected_path =
        repo_root().join("tests/fixtures/conformance/stdlib_functions.expected.txt");
    let expected = fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", expected_path.display()));

    assert_eq!(
        normalize(&actual),
        normalize(&expected),
        "C target output for com.bascal.stdlib no longer matches the golden \
         expectation at {}",
        expected_path.display()
    );
}

#[test]
fn self_referential_string_concatenation_matches_real_bascom() {
    require_fixture!();

    let actual = compile_link_run("string_self_concat");
    let expected_path =
        repo_root().join("tests/fixtures/conformance/string_self_concat.expected.txt");
    let expected = fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", expected_path.display()));

    assert_eq!(
        normalize(&actual),
        normalize(&expected),
        "real BASCOM's output for self-referential LEFT$/MID$/RIGHT$ concatenation no longer \
         matches the golden expectation at {}",
        expected_path.display()
    );
}

/// Compiles `fixture_name.bcl` to `--target c`, builds it with `gcc`, and
/// runs the resulting binary with `work_dir` as its working directory
/// (so a relative `OPEN "X.DAT"` reads/writes a file right there,
/// alongside anything `compile_link_run_in`/real BASCOM leaves behind in
/// the same directory) -- returns the program's captured stdout.
fn compile_run_c_target_in(fixture_name: &str, work_dir: &Path) -> String {
    let fixture_bcl = repo_root()
        .join("tests/fixtures/conformance")
        .join(format!("{fixture_name}.bcl"));
    let options = bcc::CompileOptions {
        target: bcc::Target::C,
        ..bcc::CompileOptions::new()
    };
    let c_source = bcc::compile_file(&fixture_bcl, &options).unwrap_or_else(|diagnostics| {
        panic!(
            "failed to compile {} to C: {diagnostics:#?}",
            fixture_bcl.display()
        )
    });

    fs::create_dir_all(work_dir).expect("failed to create C-target work directory");
    let c_path = work_dir.join(format!("{fixture_name}.c"));
    fs::write(&c_path, &c_source)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", c_path.display()));

    let bin_path = work_dir.join(fixture_name);
    let gcc_output = Command::new("gcc")
        .arg("-std=c99")
        .arg("-o")
        .arg(&bin_path)
        .arg(&c_path)
        .arg("-lm")
        .output()
        .expect("failed to invoke gcc");
    assert!(
        gcc_output.status.success(),
        "gcc failed to compile the C target's output for `{fixture_name}`:\n{}",
        String::from_utf8_lossy(&gcc_output.stderr)
    );

    let run_output = Command::new(&bin_path)
        .current_dir(work_dir)
        .output()
        .unwrap_or_else(|err| panic!("failed to run {}: {err}", bin_path.display()));
    assert!(
        run_output.status.success(),
        "`{fixture_name}` (C target) exited with {}:\nstdout:\n{}\nstderr:\n{}",
        run_output.status,
        String::from_utf8_lossy(&run_output.stdout),
        String::from_utf8_lossy(&run_output.stderr)
    );
    String::from_utf8_lossy(&run_output.stdout).into_owned()
}

#[test]
fn self_referential_string_concatenation_matches_c_target() {
    let work_dir = std::env::temp_dir().join("bascal-c-target-string-self-concat");
    let _ = fs::remove_dir_all(&work_dir);
    let actual = compile_run_c_target_in("string_self_concat", &work_dir);
    let expected_path =
        repo_root().join("tests/fixtures/conformance/string_self_concat.expected.txt");
    let expected = fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", expected_path.display()));

    assert_eq!(
        normalize(&actual),
        normalize(&expected),
        "C target output for self-referential LEFT$/MID$/RIGHT$ concatenation no longer \
         matches the golden expectation at {}",
        expected_path.display()
    );
}

/// GitHub issue #38: real BASIC intrinsics (`left`/`right`/`mid`/`len`/
/// `instr`, `abs`/`sqr`/`sin`/`cos`/`tan`/`int`/`fix`/`sgn`) callable as
/// scalar methods (`s$.left(3)`), resolved by rewriting to the identical
/// ordinary call rather than teaching either backend a second lookup path
/// (see `scalar_builtins.rs`) -- so this is really testing that the
/// rewrite produces exactly the same real-BASIC-recognized syntax a
/// hand-written ordinary call would.
#[test]
fn builtin_scalar_methods_match_real_bascom() {
    require_fixture!();

    let actual = compile_link_run("builtin_scalar_methods");
    let expected_path =
        repo_root().join("tests/fixtures/conformance/builtin_scalar_methods.expected.txt");
    let expected = fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", expected_path.display()));

    assert_eq!(
        normalize(&actual),
        normalize(&expected),
        "real BASCOM's output for built-in scalar methods no longer matches the \
         golden expectation at {}",
        expected_path.display()
    );
}

#[test]
fn builtin_scalar_methods_match_c_target() {
    let work_dir = std::env::temp_dir().join("bascal-c-target-builtin-scalar-methods");
    let _ = fs::remove_dir_all(&work_dir);
    let actual = compile_run_c_target_in("builtin_scalar_methods", &work_dir);
    let expected_path =
        repo_root().join("tests/fixtures/conformance/builtin_scalar_methods.expected.txt");
    let expected = fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", expected_path.display()));

    assert_eq!(
        normalize(&actual),
        normalize(&expected),
        "C target output for built-in scalar methods no longer matches the \
         golden expectation at {}",
        expected_path.display()
    );
}

/// Same as `compile_link_run`, but runs inside an already-prepared
/// `work_dir` instead of creating (and wiping) its own -- needed for the
/// cross-target file-compatibility tests below, where a data file one
/// target's run wrote into `work_dir` has to survive into the other
/// target's run right after it.
fn compile_link_run_in(fixture_name: &str, work_dir: &Path) -> String {
    let fixture_bcl = repo_root()
        .join("tests/fixtures/conformance")
        .join(format!("{fixture_name}.bcl"));
    let options = bcc::CompileOptions {
        line_numbers: true,
        ..bcc::CompileOptions::new()
    };
    let basic = bcc::compile_file(&fixture_bcl, &options).unwrap_or_else(|diagnostics| {
        panic!(
            "failed to compile {}: {diagnostics:#?}",
            fixture_bcl.display()
        )
    });

    stage_compiler_fixture(work_dir);

    let dos_stem = "TEST";
    write_dos_file(&work_dir.join(format!("{dos_stem}.BAS")), &basic);
    write_dos_file(
        &work_dir.join("RUNIT.BAT"),
        &format!(
            "BASCOM {dos_stem}.BAS,,;\r\nLINK {dos_stem}.OBJ;\r\n{dos_stem}.EXE > OUT.TXT\r\nEXIT\r\n"
        ),
    );

    run_dosbox_batch(work_dir, "RUNIT.BAT");

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
        "expected BASCOM+LINK to produce {} -- compile succeeded but link must have failed",
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

/// Random-access record files are BASCAL's one place where the two
/// backends' generated output has to agree on more than just program
/// *behavior* -- it has to agree on the actual *on-disk byte layout*, or
/// a file one target writes won't read back correctly under the other.
/// cross_write.bcl/cross_read.bcl (a 2-byte MKI$-packed int field plus a
/// 10-byte right-justified string field -- deliberately no float field, since
/// MKS$/MKD$/CVS/CVD are a real, documented divergence from real BASCOM's
/// Microsoft Binary Format, see `FILE_IO_HELPER`'s doc comment in
/// codegen_c.rs -- this test isn't the place to re-demonstrate that)
/// write, then read back, the same record. Checked in both directions:
/// C writes it and real BASCOM reads it back, and real BASCOM writes it
/// and C reads it back.
#[test]
fn c_target_random_access_file_is_binary_compatible_with_real_bascom_c_writes() {
    require_fixture!();

    let work_dir = std::env::temp_dir().join("bascal-conformance-cross-c-writes");
    let _ = fs::remove_dir_all(&work_dir);
    fs::create_dir_all(&work_dir).expect("failed to create work directory");

    compile_run_c_target_in("cross_write", &work_dir);
    let actual = compile_link_run_in("cross_read", &work_dir);

    let expected_path = repo_root().join("tests/fixtures/conformance/cross_read.expected.txt");
    let expected = fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", expected_path.display()));

    assert_eq!(
        normalize(&actual),
        normalize(&expected),
        "real BASCOM's read of a random-access file the C target wrote doesn't match the \
         golden expectation at {} -- the two backends' on-disk record layout has diverged",
        expected_path.display()
    );
}

#[test]
fn c_target_random_access_file_is_binary_compatible_with_real_bascom_bascom_writes() {
    require_fixture!();

    let work_dir = std::env::temp_dir().join("bascal-conformance-cross-bascom-writes");
    let _ = fs::remove_dir_all(&work_dir);
    fs::create_dir_all(&work_dir).expect("failed to create work directory");

    compile_link_run_in("cross_write", &work_dir);
    let actual = compile_run_c_target_in("cross_read", &work_dir);

    let expected_path = repo_root().join("tests/fixtures/conformance/cross_read.expected.txt");
    let expected = fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", expected_path.display()));

    assert_eq!(
        normalize(&actual),
        normalize(&expected),
        "the C target's read of a random-access file real BASCOM wrote doesn't match the \
         golden expectation at {} -- the two backends' on-disk record layout has diverged",
        expected_path.display()
    );
}

/// Locks in the exact tie-break rounding facts the C backend's \/MOD/AND
/// translation (src/codegen_c.rs) depends on -- round() ties away from
/// zero, matching real BASCOM's own float-to-integer conversion, verified
/// once by hand under dosbox-x and captured here as a permanent regression
/// check rather than staying a one-off manual run.
#[test]
fn tie_break_rounding_matches_real_bascom() {
    require_fixture!();

    let actual = compile_link_run("tie_break_rounding");

    let expected_path =
        repo_root().join("tests/fixtures/conformance/tie_break_rounding.expected.txt");
    let expected = fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", expected_path.display()));

    assert_eq!(
        normalize(&actual),
        normalize(&expected),
        "real BASCOM's \\/MOD/AND tie-break rounding no longer matches the \
         golden expectation at {} -- if this is a deliberate/expected \
         change, double-check codegen_c.rs's round()-based translation \
         still matches",
        expected_path.display()
    );
}
