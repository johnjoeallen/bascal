//! Opt-in end-to-end conformance tests for the bootstrap JVM target.
//!
//! `krakatau2::assemble` (linked directly into `bcc`, not a separate
//! `krak2` binary/subprocess -- see `Cargo.toml`'s own comment) turns the
//! Krakatau text emitted by `--target jvm` into a real `.class`, so
//! assembly itself is always available once `bcc` builds. A JRE is still
//! needed to actually *run* the resulting class, which is the one
//! remaining external, opt-in prerequisite this suite skips rather than
//! fails on.
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

fn jvm_runtime_available() -> bool {
    java_available()
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

#[test]
fn jvm_expected_failure_read_without_data_has_specific_diagnostic() {
    assert_jvm_expected_failure(
        "program ReadWithoutData\nread value%\nend\n",
        "no `data` items at all",
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
        eprintln!("skipping {}: java is unavailable", module_path!());
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
        eprintln!("skipping {}: java is unavailable", module_path!());
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
fn jvm_semantic_mid_assignment_runs_when_available() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create MID$ source directory");
    let source = source_dir.path().join("mid_semantic_regression.bcl");
    fs::write(
        &source,
        "program MidSemanticRegression\ntext$ = \"abcdef\"\nmid$(text$, 2, 2) = \"XY\"\nprint text$\nend\n",
    ).expect("failed to write MID$ fixture");
    let assembly_dir = tempfile::tempdir().expect("failed to create MID$ assembly directory");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("MidSemanticRegression")
        .output()
        .expect("failed to run assembled MID$ class");
    assert!(
        run.status.success(),
        "assembled MID$ program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr),
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"),
        "aXYdef\n"
    );
}

#[test]
fn jvm_callable_end_exits_the_process_when_available() {
    if !java_available() {
        eprintln!("skipping {}: java is unavailable", module_path!());
        return;
    }
    let source_dir = tempfile::tempdir().expect("failed to create callable END source directory");
    let source = source_dir.path().join("CallableEndProcess.bcl");
    fs::write(
        &source,
        "program CallableEndProcess\nfunction stop%()\nend\nreturn 1\nend function\nprint \"before\"\nstop%()\nprint \"after\"\nend\n",
    )
    .expect("failed to write callable END fixture");
    let assembly_dir =
        tempfile::tempdir().expect("failed to create callable END assembly directory");
    compile_and_assemble(&source, assembly_dir.path());
    let run = Command::new("java")
        .arg("-cp")
        .arg(repo_root().join("tmp"))
        .arg("CallableEndProcess")
        .output()
        .expect("failed to run callable END class");
    assert!(
        run.status.success(),
        "callable END class failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr),
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"),
        "before\n",
        "callable END must suppress statements after the call"
    );
}

#[test]
fn jvm_semantic_block_if_runs_when_available() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create semantic IF source directory");
    let source = source_dir.path().join("semantic_block_if_runtime.bcl");
    fs::write(
        &source,
        "program SemanticBlockIfRuntime\nresult% = 0\nif 2 = 2 then\nresult% = 17\nprint \"then\";\nelse\nresult% = 19\nprint \"else\";\nend if\nprint result%\nend\n",
    )
    .expect("failed to write semantic IF fixture");
    let assembly_dir =
        tempfile::tempdir().expect("failed to create semantic IF assembly directory");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("SemanticBlockIfRuntime")
        .output()
        .expect("failed to run assembled semantic IF class");
    assert!(
        run.status.success(),
        "assembled semantic IF program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"),
        "then17\n"
    );
}

#[test]
fn jvm_semantic_while_runs_when_available() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create semantic WHILE source directory");
    let source = source_dir.path().join("semantic_while_runtime.bcl");
    fs::write(
        &source,
        "program SemanticWhileRuntime\ncount% = 0\nwhile count% < 3\ncount% = count% + 1\nprint count%\nend while\nend\n",
    )
    .expect("failed to write semantic WHILE fixture");
    let assembly_dir =
        tempfile::tempdir().expect("failed to create semantic WHILE assembly directory");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("SemanticWhileRuntime")
        .output()
        .expect("failed to run assembled semantic WHILE class");
    assert!(
        run.status.success(),
        "assembled semantic WHILE program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"),
        "1\n2\n3\n"
    );
}

#[test]
fn jvm_semantic_while_transfers_run_when_available() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create semantic loop-transfer source");
    let source = source_dir.path().join("semantic_while_transfers.bcl");
    fs::write(
        &source,
        "program SemanticWhileTransfers\ncount% = 0\nwhile count% < 5\ncount% = count% + 1\nif count% = 2 then\ncontinue\nend if\nif count% = 4 then\nexit\nend if\nprint count%\nend while\nprint \"done\"\nend\n",
    )
    .expect("failed to write semantic loop-transfer fixture");
    let assembly_dir =
        tempfile::tempdir().expect("failed to create semantic loop-transfer assembly directory");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("SemanticWhileTransfers")
        .output()
        .expect("failed to run assembled semantic loop-transfer class");
    assert!(
        run.status.success(),
        "assembled semantic loop-transfer program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"),
        "1\n3\ndone\n"
    );
}

#[test]
fn jvm_semantic_for_transfers_run_when_available() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create semantic FOR source");
    let source = source_dir.path().join("semantic_for_transfers.bcl");
    fs::write(
        &source,
        "program SemanticForTransfers\nfor count%=1 to 5\nif count%=2 then\ncontinue\nend if\nif count%=4 then\nexit\nend if\nprint count%\nend for\nfor descending%=3 downto 1\nprint descending%\nend for\nprint \"done\"\nend\n",
    )
    .expect("failed to write semantic FOR transfer fixture");
    let assembly_dir =
        tempfile::tempdir().expect("failed to create semantic FOR assembly directory");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("SemanticForTransfers")
        .output()
        .expect("failed to run assembled semantic FOR class");
    assert!(
        run.status.success(),
        "assembled semantic FOR program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"),
        "1\n3\n3\n2\n1\ndone\n"
    );
}

#[test]
fn jvm_semantic_double_comparisons_follow_ieee_ordering_when_available() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create double-comparison source");
    let source = source_dir.path().join("semantic_double_comparisons.bcl");
    fs::write(
        &source,
        "program SemanticDoubleComparisons\ndim n as double\nn=0.0/0.0\nif n=n then\nprint \"nan-equal\"\nelse\nprint \"nan-not-equal\"\nend if\nif n<>n then\nprint \"nan-inequality\"\nend if\nif n<1.0 then\nprint \"nan-less\"\nend if\nif n>1.0 then\nprint \"nan-greater\"\nend if\nif 0.0=-0.0 then\nprint \"signed-zero-equal\"\nend if\nend\n",
    )
    .expect("failed to write double-comparison fixture");
    let assembly_dir =
        tempfile::tempdir().expect("failed to create double-comparison assembly directory");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("SemanticDoubleComparisons")
        .output()
        .expect("failed to run assembled double-comparison class");
    assert!(
        run.status.success(),
        "assembled double-comparison program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"),
        "nan-not-equal\nnan-inequality\nsigned-zero-equal\n"
    );
}

#[test]
fn jvm_semantic_single_arithmetic_preserves_float32_precision_when_available() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create SINGLE precision source");
    let source = source_dir.path().join("semantic_single_precision.bcl");
    fs::write(
        &source,
        "program SemanticSinglePrecision\ndim value as single\nvalue=16777216\nvalue=value+1\nif value=16777216 then\nprint \"single-assignment\"\nelse\nprint \"double-assignment\"\nend if\nif value+1=value then\nprint \"single-expression\"\nend if\nvalue=16777216\nvalue+=1\nif value=16777216 then\nprint \"single-compound\"\nend if\ndim values(1) as single\nvalues(0)=16777216\nvalues(0)+=1\nif values(0)=16777216 then\nprint \"single-array\"\nend if\ninput value\nif value=16777216 then\nprint \"single-input\"\nend if\ninput values(1)\nif values(1)=16777216 then\nprint \"single-array-input\"\nend if\nend\n",
    )
    .expect("failed to write SINGLE precision fixture");
    let assembly_dir =
        tempfile::tempdir().expect("failed to create SINGLE precision assembly directory");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let mut child = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("SemanticSinglePrecision")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to run assembled SINGLE precision class");
    child
        .stdin
        .take()
        .expect("SINGLE precision stdin")
        .write_all(b"16777217\n16777217\n")
        .expect("write SINGLE precision input");
    let run = child
        .wait_with_output()
        .expect("wait for SINGLE precision program");
    assert!(
        run.status.success(),
        "assembled SINGLE precision program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"),
        "single-assignment\nsingle-expression\nsingle-compound\nsingle-array\nsingle-input\nsingle-array-input\n"
    );
}

#[test]
fn jvm_semantic_for_method_step_is_evaluated_once_when_available() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create semantic method-step source");
    let source = source_dir.path().join("semantic_for_method_step.bcl");
    fs::write(
        &source,
        "program SemanticForMethodStep\nmethod tick%[integer](byref calls%)\ncalls%=calls%+1\nreturn self%\nend method\ncalls%=0\nfor index%=1 to 5.tick(calls%) step 2.tick(calls%)\nprint index%\nend for\ndownStep%=-1\nfor down%=3 to 1 step downStep%.tick(calls%)\nprint down%\nend for\nprint calls%\nend\n",
    )
    .expect("failed to write semantic method-step fixture");
    let assembly_dir =
        tempfile::tempdir().expect("failed to create semantic method-step assembly directory");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("SemanticForMethodStep")
        .output()
        .expect("failed to run assembled semantic method-step class");
    assert!(
        run.status.success(),
        "assembled semantic method-step program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"),
        "1\n3\n5\n3\n2\n1\n3\n"
    );
}

#[test]
fn jvm_semantic_method_loop_conditions_run_each_iteration_when_available() {
    let root = repo_root();
    let source_dir =
        tempfile::tempdir().expect("failed to create semantic method-condition source");
    let source = source_dir
        .path()
        .join("semantic_method_loop_conditions.bcl");
    fs::write(
        &source,
        "program SemanticMethodLoopConditions\nmethod remaining%[integer](byref calls%)\ncalls%=calls%+1\nreturn self%-calls%\nend method\ncalls%=0\ndo while 3.remaining(calls%)>0\nprint calls%\nend do\ndo\nprint calls%\nloop until 2.remaining(calls%)<=0\nprint calls%\nend\n",
    )
    .expect("failed to write semantic method-condition fixture");
    let assembly_dir =
        tempfile::tempdir().expect("failed to create method-condition assembly directory");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("SemanticMethodLoopConditions")
        .output()
        .expect("failed to run assembled semantic method-condition class");
    assert!(
        run.status.success(),
        "assembled semantic method-condition program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"),
        "1\n2\n3\n4\n"
    );
}

#[test]
fn jvm_for_start_is_captured_before_mutating_method_bound_when_available() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create mutating FOR-bound source");
    let source = source_dir.path().join("for_mutating_method_bound.bcl");
    fs::write(
        &source,
        "program ForMutatingMethodBound\nmethod bump%[integer](byref target%)\ntarget%=target%+1\nreturn self%\nend method\nindex%=5\nfor index%=index% to index%.bump(index%)\nprint index%\nend for\nprint index%\nend\n",
    )
    .expect("failed to write mutating FOR-bound fixture");
    let assembly_dir =
        tempfile::tempdir().expect("failed to create mutating FOR-bound assembly directory");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("ForMutatingMethodBound")
        .output()
        .expect("failed to run assembled mutating FOR-bound class");
    assert!(
        run.status.success(),
        "assembled mutating FOR-bound program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"),
        "5\n6\n"
    );
}

#[test]
fn jvm_for_limit_is_captured_before_mutating_method_step_when_available() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create mutating FOR-STEP source");
    let source = source_dir.path().join("for_mutating_method_step.bcl");
    fs::write(
        &source,
        "program ForMutatingMethodStep\nmethod advance%[integer](byref target%)\ntarget%=target%+3\nreturn 1\nend method\nlimit%=3\nfor index%=1 to limit% step limit%.advance(limit%)\nprint index%\nend for\nprint limit%\nend\n",
    )
    .expect("failed to write mutating FOR-STEP fixture");
    let assembly_dir =
        tempfile::tempdir().expect("failed to create mutating FOR-STEP assembly directory");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("ForMutatingMethodStep")
        .output()
        .expect("failed to run assembled mutating FOR-STEP class");
    assert!(
        run.status.success(),
        "assembled mutating FOR-STEP program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"),
        "1\n2\n3\n6\n"
    );
}

#[test]
fn jvm_for_limit_is_captured_before_body_mutation_when_available() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create mutable FOR-limit source");
    let source = source_dir.path().join("for_mutable_limit.bcl");
    fs::write(
        &source,
        "program ForMutableLimit\nlimit%=3\nfor index%=1 to limit%\nlimit%=0\nprint index%\nend for\nprint limit%\nend\n",
    )
    .expect("failed to write mutable FOR-limit fixture");
    let assembly_dir =
        tempfile::tempdir().expect("failed to create mutable FOR-limit assembly directory");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("ForMutableLimit")
        .output()
        .expect("failed to run assembled mutable FOR-limit class");
    assert!(
        run.status.success(),
        "assembled mutable FOR-limit program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"),
        "1\n2\n3\n0\n"
    );
}

#[test]
fn jvm_semantic_do_transfers_run_when_available() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create semantic DO source");
    let source = source_dir.path().join("semantic_do_transfers.bcl");
    fs::write(
        &source,
        "program SemanticDoTransfers\ncount%=0\ndo while count%<6\ncount%=count%+1\nif count%=2 then\ncontinue\nend if\nif count%=5 then\nexit\nend if\nprint count%\nend do\npost%=0\ndo\npost%=post%+1\nif post%=2 then\ncontinue\nend if\nif post%=5 then\nexit\nend if\nprint post%\nloop until post%>=8\nend\n",
    )
    .expect("failed to write semantic DO transfer fixture");
    let assembly_dir =
        tempfile::tempdir().expect("failed to create semantic DO assembly directory");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("SemanticDoTransfers")
        .output()
        .expect("failed to run assembled semantic DO class");
    assert!(
        run.status.success(),
        "assembled semantic DO program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"),
        "1\n3\n4\n1\n3\n4\n"
    );
}

#[test]
fn jvm_semantic_select_case_runs_when_available() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create semantic SELECT source");
    let source = source_dir.path().join("semantic_select.bcl");
    fs::write(
        &source,
        "program SemanticSelect\nchoice%=5\nselect case choice%\ncase 1\nprint \"one\"\ncase 2 to 4\nprint \"range\"\ncase is > 4\nprint \"large\"\ncase else\nprint \"other\"\nend select\nname$=\"BASCAL\"\nselect case name$\ncase \"BASCAL\"\nprint \"string\"\ncase else\nprint \"bad\"\nend select\nend\n",
    )
    .expect("failed to write semantic SELECT fixture");
    let assembly_dir =
        tempfile::tempdir().expect("failed to create semantic SELECT assembly directory");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("SemanticSelect")
        .output()
        .expect("failed to run assembled semantic SELECT class");
    assert!(
        run.status.success(),
        "assembled semantic SELECT program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"),
        "large\nstring\n"
    );
}

#[test]
fn jvm_semantic_nested_swap_runs_when_available() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create semantic nested SWAP source");
    let source = source_dir.path().join("semantic_nested_swap.bcl");
    fs::write(
        &source,
        "program SemanticNestedSwap\nleft%=3\nright%=7\nif left%=3 then\nswap left%,right%\nend if\nprint left%;\",\";right%\nend\n",
    )
    .expect("failed to write semantic nested SWAP fixture");
    let assembly_dir =
        tempfile::tempdir().expect("failed to create semantic nested SWAP assembly directory");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("SemanticNestedSwap")
        .output()
        .expect("failed to run assembled semantic nested SWAP class");
    assert!(
        run.status.success(),
        "assembled semantic nested SWAP program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"),
        "7,3\n"
    );
}

#[test]
fn jvm_semantic_nested_return_writes_back_byref_when_available() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create nested RETURN source");
    let source = source_dir.path().join("nested_return_byref.bcl");
    fs::write(
        &source,
        "program NestedReturnByRef\nfunction mutate%(byref value%)\nif value%=1 then\nvalue%=9\nreturn value%\nend if\nreturn 0\nend function\nvalue%=1\nresult%=mutate%(value%)\nprint result%;\",\";value%\nend\n",
    )
    .expect("failed to write nested RETURN fixture");
    let assembly_dir =
        tempfile::tempdir().expect("failed to create nested RETURN assembly directory");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("NestedReturnByRef")
        .output()
        .expect("failed to run assembled nested RETURN class");
    assert!(
        run.status.success(),
        "assembled nested RETURN program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"),
        "9,9\n"
    );
}

#[test]
fn jvm_semantic_nested_locate_color_run_when_available() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create nested terminal source");
    let source = source_dir.path().join("nested_terminal.bcl");
    fs::write(
        &source,
        "program NestedTerminal\nflag%=1\nif flag%=1 then\nlocate 5,6\ncolor 7,2\nend if\nprint \"done\"\nend\n",
    )
    .expect("failed to write nested terminal fixture");
    let assembly_dir =
        tempfile::tempdir().expect("failed to create nested terminal assembly directory");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("NestedTerminal")
        .output()
        .expect("failed to run assembled nested terminal class");
    assert!(
        run.status.success(),
        "assembled nested terminal program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "\u{1b}[5;6H\u{1b}[37;42mdone\n"
    );
}

#[test]
fn jvm_semantic_nested_input_reads_value_when_available() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create nested INPUT source");
    let source = source_dir.path().join("nested_input.bcl");
    fs::write(
        &source,
        "program NestedInput\nflag%=1\nif flag%=1 then\ninput \"value\"; value%\nend if\nprint \"read:\";value%\nend\n",
    )
    .expect("failed to write nested INPUT fixture");
    let assembly_dir =
        tempfile::tempdir().expect("failed to create nested INPUT assembly directory");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let mut child = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("NestedInput")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to launch assembled nested INPUT class");
    child
        .stdin
        .take()
        .expect("nested INPUT child has no stdin pipe")
        .write_all(b"42\n")
        .expect("failed to write nested INPUT value");
    let run = child
        .wait_with_output()
        .expect("failed to collect nested INPUT process output");
    assert!(
        run.status.success(),
        "assembled nested INPUT program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"),
        "value? read:42\n"
    );
}

#[test]
fn jvm_semantic_nested_random_file_lifecycle_runs_when_available() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create nested file-operation source");
    let source = source_dir.path().join("nested_file_lifecycle.bcl");
    fs::write(
        &source,
        "program NestedFileLifecycle\nflag%=1\nif flag%=1 then\nopen \"nested_random.dat\" for random as #1 len = 4\nclose #1\nkill \"nested_random.dat\"\nend if\nend\n",
    )
    .expect("failed to write nested file-operation fixture");
    let assembly_dir =
        tempfile::tempdir().expect("failed to create nested file-operation assembly directory");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("NestedFileLifecycle")
        .current_dir(source_dir.path())
        .output()
        .expect("failed to run assembled nested file-operation class");
    assert!(
        run.status.success(),
        "assembled nested file-operation program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert!(
        !source_dir.path().join("nested_random.dat").exists(),
        "nested typed OPEN/CLOSE/KILL left its random-access file behind"
    );
}

#[test]
fn jvm_semantic_nested_field_sets_round_trip_when_available() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create nested FIELD source");
    let source = source_dir.path().join("nested_field_sets.bcl");
    fs::write(
        &source,
        "program NestedFieldSets\nflag%=1\nopen \"nested_fields.dat\" for random as #1 len = 8\nfield #1, 4 as left$, 4 as right$\nif flag%=1 then\nlset left$ = \"AB\"\nrset right$ = \"Z\"\nend if\nput #1, 1\nleft$ = \"----\"\nright$ = \"----\"\nget #1, 1\nprint \"[\";left$;\"],[\";right$;\"]\"\nclose #1\nend\n",
    )
    .expect("failed to write nested FIELD fixture");
    let assembly_dir = tempfile::tempdir().expect("failed to create nested FIELD assembly dir");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("NestedFieldSets")
        .current_dir(source_dir.path())
        .output()
        .expect("failed to run assembled nested FIELD class");
    assert!(
        run.status.success(),
        "assembled nested FIELD program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"),
        "[AB  ],[   Z]\n"
    );
}

#[test]
fn jvm_semantic_nested_field_layout_round_trips_when_available() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create nested FIELD layout source");
    let source = source_dir.path().join("nested_field_layout.bcl");
    fs::write(
        &source,
        "program NestedFieldLayout\nflag%=1\nopen \"nested_field_layout.dat\" for random as #1 len = 8\nif flag%=1 then\nfield #1, 4 as left$, 4 as right$\nlset left$ = \"AB\"\nrset right$ = \"Z\"\nend if\nput #1, 1\nleft$ = \"----\"\nright$ = \"----\"\nget #1, 1\nprint \"[\";left$;\"],[\";right$;\"]\"\nclose #1\nend\n",
    )
    .expect("failed to write nested FIELD layout fixture");
    let assembly_dir =
        tempfile::tempdir().expect("failed to create nested FIELD layout assembly directory");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("NestedFieldLayout")
        .current_dir(source_dir.path())
        .output()
        .expect("failed to run assembled nested FIELD layout class");
    assert!(
        run.status.success(),
        "assembled nested FIELD layout program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"),
        "[AB  ],[   Z]\n"
    );
}

#[test]
fn jvm_semantic_nested_file_line_input_reads_a_typed_string_when_available() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create nested LINE INPUT source");
    fs::write(
        source_dir.path().join("line_input.dat"),
        b"typed line\nsecond line\n",
    )
    .expect("failed to seed LINE INPUT file");
    let source = source_dir.path().join("nested_line_input.bcl");
    fs::write(
        &source,
        "program NestedLineInput\nflag%=1\nchannel%=1\nopen \"line_input.dat\" for random as #1 len = 32\nif flag%=1 then\nline input #channel%, line$\nend if\nprint \"[\";line$;\"]\"\nclose #1\nend\n",
    )
    .expect("failed to write nested LINE INPUT fixture");
    let assembly_dir =
        tempfile::tempdir().expect("failed to create nested LINE INPUT assembly directory");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("NestedLineInput")
        .current_dir(source_dir.path())
        .output()
        .expect("failed to run assembled nested LINE INPUT class");
    assert!(
        run.status.success(),
        "assembled nested LINE INPUT program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"),
        "[typed line]\n"
    );
}

#[test]
fn jvm_semantic_channel_print_writes_typed_file_bytes_when_available() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create channel PRINT source");
    let source = source_dir.path().join("channel_print.bcl");
    fs::write(
        &source,
        "program ChannelPrint\nfunction fileText$()\n return \"X\"\nend function\nchannel%=1\nopen \"channel_print.dat\" for random as #1 len = 4\nseek #1, 2\nprint #channel%, fileText$();\nprint #channel%, 42\nprint #channel%, \"Y\", 43\nclose #1\nend\n",
    )
    .expect("failed to write channel PRINT fixture");
    let assembly_dir = tempfile::tempdir().expect("failed to create channel PRINT assembly dir");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("ChannelPrint")
        .current_dir(source_dir.path())
        .output()
        .expect("failed to run assembled channel PRINT class");
    assert!(
        run.status.success(),
        "assembled channel PRINT program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    let bytes = fs::read(source_dir.path().join("channel_print.dat"))
        .expect("failed to read channel PRINT output file");
    assert_eq!(bytes, b"\0\0\0\0X42\nY43\n");
}

#[test]
fn jvm_nested_typed_write_emits_csv_file_bytes_when_available() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create nested WRITE source");
    let source = source_dir.path().join("nested_write.bcl");
    fs::write(
        &source,
        "program NestedWrite\nfunction writeValue%()\nglobal calls%\ncalls%=calls%+1\nreturn 42\nend function\ncalls%=0\nflag%=1\nopen \"nested_write.dat\" for random as #1 len = 32\nif flag%=1 then\nwrite #1, \"a,b\", writeValue%()\nend if\nclose #1\nprint calls%\nend\n",
    )
    .expect("failed to write nested WRITE fixture");
    let assembly_dir = tempfile::tempdir().expect("failed to create nested WRITE assembly dir");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("NestedWrite")
        .current_dir(source_dir.path())
        .output()
        .expect("failed to run assembled nested WRITE class");
    assert!(
        run.status.success(),
        "assembled nested WRITE program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"), "1\n");
    let bytes = fs::read(source_dir.path().join("nested_write.dat"))
        .expect("failed to read nested WRITE output file");
    assert_eq!(bytes, b"\"a,b\",42\n");
}

#[test]
fn jvm_nested_typed_read_consumes_data_items_when_available() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create nested READ source");
    let source = source_dir.path().join("nested_read.bcl");
    fs::write(
        &source,
        "program NestedRead\ndata 42, \"typed\"\n\nflag%=1\nif flag%=1 then\nread value%\nend if\nprocedure readText()\nglobal text$\nread text$\nend procedure\nreadText()\nprint value%; \":\"; text$\nend\n",
    )
    .expect("failed to write nested READ fixture");
    let assembly_dir = tempfile::tempdir().expect("failed to create nested READ assembly dir");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("NestedRead")
        .current_dir(source_dir.path())
        .output()
        .expect("failed to run assembled nested READ class");
    assert!(
        run.status.success(),
        "assembled nested READ program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"), "42:typed\n");
}

#[test]
fn jvm_conditional_typed_read_does_not_advance_when_branch_is_skipped() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create conditional READ source");
    let source = source_dir.path().join("conditional_read.bcl");
    fs::write(
        &source,
        "program ConditionalRead\ndata 41, 99\nif false then\nread skipped%\nend if\nread result%\nprint result%\nend\n",
    )
    .expect("failed to write conditional READ fixture");
    let assembly_dir = tempfile::tempdir().expect("failed to create conditional READ assembly dir");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("ConditionalRead")
        .current_dir(source_dir.path())
        .output()
        .expect("failed to run assembled conditional READ class");
    assert!(
        run.status.success(),
        "assembled conditional READ program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"), "41\n");
}

#[test]
fn jvm_typed_data_in_conditional_block_is_included_in_pool() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create nested DATA source");
    let source = source_dir.path().join("nested_data.bcl");
    fs::write(
        &source,
        "program NestedData\nif false then\ndata 58\nend if\nread value%\nprint value%\nend\n",
    )
    .expect("failed to write nested DATA fixture");
    let assembly_dir = tempfile::tempdir().expect("failed to create nested DATA assembly dir");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("NestedData")
        .current_dir(source_dir.path())
        .output()
        .expect("failed to run assembled nested DATA class");
    assert!(
        run.status.success(),
        "assembled nested DATA program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"), "58\n");
}

#[test]
fn jvm_typed_data_in_callable_body_is_added_to_shared_pool_when_available() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create callable DATA source");
    let source = source_dir.path().join("callable_data.bcl");
    fs::write(
        &source,
        "program CallableData\ndata 12\nprocedure provideData()\ndata 34\nend procedure\nread first%\nread second%\nprint first%; \":\"; second%\nend\n",
    )
    .expect("failed to write callable DATA fixture");
    let assembly_dir = tempfile::tempdir().expect("failed to create callable DATA assembly dir");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("CallableData")
        .current_dir(source_dir.path())
        .output()
        .expect("failed to run assembled callable DATA class");
    assert!(
        run.status.success(),
        "assembled callable DATA program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"), "12:34\n");
}

#[test]
fn jvm_nested_typed_read_stores_into_array_elements_when_available() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create array READ source");
    let source = source_dir.path().join("array_read.bcl");
    fs::write(
        &source,
        "program ArrayRead\ndata 41\ndim values%(4)\nflag%=1\nif flag%=1 then\nread values%(2)\nend if\nprint values%(2)\nend\n",
    )
    .expect("failed to write array READ fixture");
    let assembly_dir = tempfile::tempdir().expect("failed to create array READ assembly dir");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("ArrayRead")
        .current_dir(source_dir.path())
        .output()
        .expect("failed to run assembled array READ class");
    assert!(
        run.status.success(),
        "assembled array READ program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"), "41\n");
}

#[test]
fn jvm_typed_read_array_index_expression_is_evaluated_once_when_available() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create indexed READ source");
    let source = source_dir.path().join("indexed_read.bcl");
    fs::write(
        &source,
        "program IndexedRead\ndata 55\ndim values%(4)\nfunction nextIndex%()\nglobal calls%\ncalls%=calls%+1\nreturn 2\nend function\ncalls%=0\nread values%(nextIndex%())\nprint values%(2); \":\"; calls%\nend\n",
    )
    .expect("failed to write indexed READ fixture");
    let assembly_dir = tempfile::tempdir().expect("failed to create indexed READ assembly dir");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("IndexedRead")
        .current_dir(source_dir.path())
        .output()
        .expect("failed to run assembled indexed READ class");
    assert!(
        run.status.success(),
        "assembled indexed READ program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"), "55:1\n");
}

#[test]
fn jvm_typed_read_stores_string_data_into_array_elements_when_available() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create string array READ source");
    let source = source_dir.path().join("string_array_read.bcl");
    fs::write(
        &source,
        "program StringArrayRead\ndata \"semantic\"\ndim values$(2, 3)\nread values$(1, 2)\nprint values$(1, 2)\nend\n",
    )
    .expect("failed to write string array READ fixture");
    let assembly_dir = tempfile::tempdir().expect("failed to create string array READ assembly dir");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("StringArrayRead")
        .current_dir(source_dir.path())
        .output()
        .expect("failed to run assembled string array READ class");
    assert!(
        run.status.success(),
        "assembled string array READ program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"),
        "semantic\n"
    );
}

#[test]
fn jvm_typed_read_converts_long_double_and_string_data_when_available() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create typed DATA source");
    let source = source_dir.path().join("typed_data.bcl");
    fs::write(
        &source,
        "program TypedData\ndata -3.25, 9999999999, \"typed\"\nread amount#, total&, label$\nprint amount#; \":\"; total&; \":\"; label$\nend\n",
    )
    .expect("failed to write typed DATA fixture");
    let assembly_dir = tempfile::tempdir().expect("failed to create typed DATA assembly dir");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("TypedData")
        .current_dir(source_dir.path())
        .output()
        .expect("failed to run assembled typed DATA class");
    assert!(
        run.status.success(),
        "assembled typed DATA program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"), "-3.25:9999999999:typed\n");
}

#[test]
fn jvm_typed_single_read_preserves_binary32_rounding_when_available() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create SINGLE READ source");
    let source = source_dir.path().join("single_read.bcl");
    fs::write(
        &source,
        "program SingleRead\ndata 16777217\nread amount!\nif amount! = 16777216 then\nprint \"rounded\"\nelse\nprint \"unrounded\"\nend if\nend\n",
    )
    .expect("failed to write SINGLE READ fixture");
    let assembly_dir = tempfile::tempdir().expect("failed to create SINGLE READ assembly dir");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("SingleRead")
        .current_dir(source_dir.path())
        .output()
        .expect("failed to run assembled SINGLE READ class");
    assert!(
        run.status.success(),
        "assembled SINGLE READ program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"), "rounded\n");
}

#[test]
fn jvm_typed_restore_resets_data_cursor_to_start_and_label_when_available() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create RESTORE source");
    let source = source_dir.path().join("restore_data.bcl");
    fs::write(
        &source,
        "program RestoreData\ndata 5\nstart:\ndata 23\nread initial%\nrestore\nread first%\nrestore start\nread second%\nprint initial%; \":\"; first%; \":\"; second%\nend\n",
    )
    .expect("failed to write RESTORE fixture");
    let assembly_dir = tempfile::tempdir().expect("failed to create RESTORE assembly dir");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("RestoreData")
        .current_dir(source_dir.path())
        .output()
        .expect("failed to run assembled RESTORE class");
    assert!(
        run.status.success(),
        "assembled RESTORE program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"),
        "5:5:23\n"
    );
}

#[test]
fn jvm_callable_typed_restore_updates_shared_data_cursor_when_available() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create callable RESTORE source");
    let source = source_dir.path().join("callable_restore.bcl");
    fs::write(
        &source,
        "program CallableRestore\ndata 12, 34\nread first%\nprocedure rewindData()\nrestore\nend procedure\nrewindData()\nread second%\nprint first%; \":\"; second%\nend\n",
    )
    .expect("failed to write callable RESTORE fixture");
    let assembly_dir = tempfile::tempdir().expect("failed to create callable RESTORE assembly dir");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("CallableRestore")
        .current_dir(source_dir.path())
        .output()
        .expect("failed to run assembled callable RESTORE class");
    assert!(
        run.status.success(),
        "assembled callable RESTORE program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"), "12:12\n");
}

#[test]
fn jvm_typed_restore_resolves_labels_in_nested_blocks_when_available() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create nested RESTORE label source");
    let source = source_dir.path().join("nested_restore_label.bcl");
    fs::write(
        &source,
        "program NestedRestoreLabel\ndata 4\nif false then\nnestedData:\ndata 7\nend if\nread first%\nrestore nestedData\nread second%\nprint first%; \":\"; second%\nend\n",
    )
    .expect("failed to write nested RESTORE label fixture");
    let assembly_dir = tempfile::tempdir().expect("failed to create nested RESTORE label assembly dir");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("NestedRestoreLabel")
        .current_dir(source_dir.path())
        .output()
        .expect("failed to run assembled nested RESTORE label class");
    assert!(
        run.status.success(),
        "assembled nested RESTORE label program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"), "4:7\n");
}

#[test]
fn jvm_typed_on_goto_uses_one_based_label_dispatch_when_available() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create ON GOTO source");
    let source = source_dir.path().join("on_goto.bcl");
    fs::write(
        &source,
        "program OnGoto\nfunction nextSelector%()\nglobal calls%\ncalls%=calls%+1\nreturn 2\nend function\ncalls%=0\nif true then\non nextSelector%() goto first, second\nend if\nprint \"fallthrough\"\nend\nfirst:\nprint \"first\"\nend\nsecond:\nprint \"second\"\nprint calls%\nend\n",
    )
    .expect("failed to write ON GOTO fixture");
    let assembly_dir = tempfile::tempdir().expect("failed to create ON GOTO assembly dir");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("OnGoto")
        .current_dir(source_dir.path())
        .output()
        .expect("failed to run assembled ON GOTO class");
    assert!(
        run.status.success(),
        "assembled ON GOTO program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"),
        "second\n1\n"
    );
}

#[test]
fn jvm_typed_on_goto_falls_through_outside_target_range_when_available() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create ON GOTO fallthrough source");
    let source = source_dir.path().join("on_goto_fallthrough.bcl");
    fs::write(
        &source,
        "program OnGotoFallthrough\nselector%=0\n\non selector% goto target\nprint \"fallthrough\"\nend\ntarget:\nprint \"branch\"\nend\n",
    )
    .expect("failed to write ON GOTO fallthrough fixture");
    let assembly_dir = tempfile::tempdir().expect("failed to create ON GOTO fallthrough assembly dir");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("OnGotoFallthrough")
        .current_dir(source_dir.path())
        .output()
        .expect("failed to run assembled ON GOTO fallthrough class");
    assert!(
        run.status.success(),
        "assembled ON GOTO fallthrough program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"),
        "fallthrough\n"
    );
}

#[test]
fn jvm_typed_on_goto_selector_one_selects_first_label_when_available() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create ON GOTO first-target source");
    let source = source_dir.path().join("on_goto_first.bcl");
    fs::write(
        &source,
        "program OnGotoFirst\non 1 goto first, second\nprint \"fallthrough\"\nend\nfirst:\nprint \"first\"\nend\nsecond:\nprint \"second\"\nend\n",
    )
    .expect("failed to write ON GOTO first-target fixture");
    let assembly_dir = tempfile::tempdir().expect("failed to create ON GOTO first-target assembly dir");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("OnGotoFirst")
        .current_dir(source_dir.path())
        .output()
        .expect("failed to run assembled ON GOTO first-target class");
    assert!(
        run.status.success(),
        "assembled ON GOTO first-target program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"), "first\n");
}

#[test]
fn jvm_typed_on_goto_converts_long_selector_for_later_target_when_available() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create long ON GOTO source");
    let source = source_dir.path().join("on_goto_long.bcl");
    fs::write(
        &source,
        "program OnGotoLong\nselector&=3\non selector& goto first, second, third\nprint \"fallthrough\"\nend\nfirst:\nprint \"first\"\nend\nsecond:\nprint \"second\"\nend\nthird:\nprint \"third\"\nend\n",
    )
    .expect("failed to write long ON GOTO fixture");
    let assembly_dir = tempfile::tempdir().expect("failed to create long ON GOTO assembly dir");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("OnGotoLong")
        .current_dir(source_dir.path())
        .output()
        .expect("failed to run assembled long ON GOTO class");
    assert!(
        run.status.success(),
        "assembled long ON GOTO program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"), "third\n");
}

#[test]
fn jvm_typed_on_goto_selector_above_target_count_falls_through_when_available() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create ON GOTO upper-bound source");
    let source = source_dir.path().join("on_goto_upper_bound.bcl");
    fs::write(
        &source,
        "program OnGotoUpperBound\non 3 goto first, second\nprint \"fallthrough\"\nend\nfirst:\nprint \"first\"\nend\nsecond:\nprint \"second\"\nend\n",
    )
    .expect("failed to write ON GOTO upper-bound fixture");
    let assembly_dir = tempfile::tempdir().expect("failed to create ON GOTO upper-bound assembly dir");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("OnGotoUpperBound")
        .current_dir(source_dir.path())
        .output()
        .expect("failed to run assembled ON GOTO upper-bound class");
    assert!(
        run.status.success(),
        "assembled ON GOTO upper-bound program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"), "fallthrough\n");
}

#[test]
fn jvm_typed_on_goto_assigns_unique_labels_for_multiple_statements_when_available() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create repeated ON GOTO source");
    let source = source_dir.path().join("repeated_on_goto.bcl");
    fs::write(
        &source,
        "program RepeatedOnGoto\non 0 goto first\non 2 goto first, second\nprint \"fallthrough\"\nend\nfirst:\nprint \"first\"\nend\nsecond:\nprint \"second\"\nend\n",
    )
    .expect("failed to write repeated ON GOTO fixture");
    let assembly_dir = tempfile::tempdir().expect("failed to create repeated ON GOTO assembly dir");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("RepeatedOnGoto")
        .current_dir(source_dir.path())
        .output()
        .expect("failed to run assembled repeated ON GOTO class");
    assert!(
        run.status.success(),
        "assembled repeated ON GOTO program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"), "second\n");
}

#[test]
fn jvm_typed_read_reports_out_of_data_when_available() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create exhausted DATA source");
    let source = source_dir.path().join("exhausted_data.bcl");
    fs::write(
        &source,
        "program ExhaustedData\ndata 5\nread first%\nread second%\nend\n",
    )
    .expect("failed to write exhausted DATA fixture");
    let assembly_dir = tempfile::tempdir().expect("failed to create exhausted DATA assembly dir");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("ExhaustedData")
        .current_dir(source_dir.path())
        .output()
        .expect("failed to run assembled exhausted DATA class");
    assert!(!run.status.success(), "exhausted READ unexpectedly succeeded");
    assert!(
        String::from_utf8_lossy(&run.stderr).contains("Out of DATA"),
        "missing Out of DATA runtime diagnostic:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );
}

#[test]
fn jvm_nested_typed_output_open_truncates_and_writes_when_available() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create nested OUTPUT source");
    let source = source_dir.path().join("nested_output.bcl");
    fs::write(
        &source,
        "program NestedOutput\nfunction outputPath$()\nreturn \"nested_output.dat\"\nend function\nfunction outputChannel%()\nglobal calls%\ncalls%=calls%+1\nreturn 2\nend function\ncalls%=0\nflag%=1\nprocedure writeOutput()\nglobal flag%\nif flag%=1 then\nopen outputPath$() for output as #outputChannel%()\nwrite #2, \"typed\", 73\nclose #2\nend if\nif flag%=1 then\nopen \"nested_output.dat\" for append as #3\nwrite #3, \"appended\"\nclose #3\nend if\nend procedure\nwriteOutput()\nprint calls%\nend\n",
    )
    .expect("failed to write nested OUTPUT fixture");
    fs::write(
        source_dir.path().join("nested_output.dat"),
        b"previous contents",
    )
    .expect("failed to seed nested OUTPUT file");
    let assembly_dir = tempfile::tempdir().expect("failed to create nested OUTPUT assembly dir");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("NestedOutput")
        .current_dir(source_dir.path())
        .output()
        .expect("failed to run assembled nested OUTPUT class");
    assert!(
        run.status.success(),
        "assembled nested OUTPUT program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"), "1\n");
    let bytes = fs::read(source_dir.path().join("nested_output.dat"))
        .expect("failed to read nested OUTPUT file");
    assert_eq!(bytes, b"\"typed\",73\n\"appended\"\n");
}

#[test]
fn jvm_semantic_random_open_evaluates_channel_once_when_available() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create random OPEN source");
    let source = source_dir.path().join("open_channel_once.bcl");
    fs::write(
        &source,
        "program OpenChannelOnce\nfunction selectChannel%()\n global calls%\n calls%=calls%+1\n return 1\nend function\ncalls%=0\nopen \"open_once.dat\" for random as #selectChannel%() len = 4\nclose #1\nprint calls%\nend\n",
    )
    .expect("failed to write random OPEN fixture");
    let assembly_dir = tempfile::tempdir().expect("failed to create random OPEN assembly dir");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("OpenChannelOnce")
        .current_dir(source_dir.path())
        .output()
        .expect("failed to run assembled random OPEN class");
    assert!(
        run.status.success(),
        "assembled random OPEN program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"),
        "1\n"
    );
}

#[test]
fn jvm_semantic_nested_get_put_round_trip_when_available() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create nested GET/PUT source");
    let source = source_dir.path().join("nested_get_put.bcl");
    fs::write(
        &source,
        "program NestedGetPut\nflag%=1\nopen \"nested_get_put.dat\" for random as #1 len = 4\nfield #1, 4 as value$\nlset value$ = \"ONE\"\nput #1, 1\nif flag%=1 then\nget #1, 1\nlset value$ = \"TWO\"\nput #1, 2\nget #1, 2\nend if\nprint \"[\";value$;\"]\"\nclose #1\nend\n",
    )
    .expect("failed to write nested GET/PUT fixture");
    let assembly_dir =
        tempfile::tempdir().expect("failed to create nested GET/PUT assembly directory");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("NestedGetPut")
        .current_dir(source_dir.path())
        .output()
        .expect("failed to run assembled nested GET/PUT class");
    assert!(
        run.status.success(),
        "assembled nested GET/PUT program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"),
        "[TWO ]\n"
    );
}

#[test]
fn jvm_semantic_seek_runs_at_module_and_nested_scope_when_available() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create semantic SEEK source");
    let source = source_dir.path().join("semantic_seek.bcl");
    fs::write(
        &source,
        "program SemanticSeek\nflag%=1\nopen \"semantic_seek.dat\" for random as #1 len = 4\nfield #1, 4 as value$\nseek #1, 2\nif flag%=1 then\nseek #1, 3\nend if\nclose #1\nend\n",
    )
    .expect("failed to write semantic SEEK fixture");
    let assembly_dir = tempfile::tempdir().expect("failed to create semantic SEEK assembly dir");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("SemanticSeek")
        .current_dir(source_dir.path())
        .output()
        .expect("failed to run assembled semantic SEEK class");
    assert!(
        run.status.success(),
        "assembled semantic SEEK program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
}

#[test]
fn jvm_semantic_nested_rename_moves_file_when_available() {
    let root = repo_root();
    let source_dir = tempfile::tempdir().expect("failed to create nested rename source");
    let source = source_dir.path().join("nested_rename.bcl");
    fs::write(
        &source,
        "program NestedRename\nflag%=1\nopen \"before.dat\" for random as #1 len = 4\nclose #1\nif flag%=1 then\nname \"before.dat\" as \"after.dat\"\nend if\nend\n",
    )
    .expect("failed to write nested rename fixture");
    let assembly_dir = tempfile::tempdir().expect("failed to create nested rename assembly dir");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("NestedRename")
        .current_dir(source_dir.path())
        .output()
        .expect("failed to run assembled nested rename class");
    assert!(
        run.status.success(),
        "assembled nested rename program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert!(
        !source_dir.path().join("before.dat").exists(),
        "typed NAME left its source path in place"
    );
    assert!(
        source_dir.path().join("after.dat").is_file(),
        "typed NAME did not create the destination path"
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
        eprintln!("skipping {}: java is unavailable", module_path!());
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

#[test]
fn jvm_dynamic_dim_evaluates_typed_bound_at_statement_position_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: java is unavailable", module_path!());
        return;
    }
    let work_dir = tempfile::tempdir().expect("failed to create dynamic DIM work directory");
    let source_path = work_dir.path().join("dynamic_dim.bcl");
    fs::write(
        &source_path,
        "program DynamicDimOrder\ndim bound&\nbound& = 3\n\ndim values%(bound&)\nvalues%(3) = 42\nprint values%(3)\nend\n",
    )
    .expect("failed to write dynamic DIM fixture");

    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("jvm")
        .arg("--clean")
        .arg("--run")
        .current_dir(work_dir.path())
        .output()
        .expect("failed to invoke bcc");
    assert!(
        output.status.success(),
        "dynamic DIM failed under --target jvm:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line == "42"),
        "expected the dynamic array value in stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn jvm_dynamic_dim_evaluates_side_effecting_typed_bound_once_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: java is unavailable", module_path!());
        return;
    }
    let work_dir = tempfile::tempdir().expect("failed to create dynamic DIM work directory");
    let source_path = work_dir.path().join("dynamic_dim_call.bcl");
    fs::write(
        &source_path,
        "program DynamicDimCall\nfunction nextBound&()\nglobal calls%\ncalls% = calls% + 1\nreturn 3\nend function\ncalls% = 0\ndim values%(nextBound&())\nvalues%(3) = 42\nprint values%(3); \",\"; calls%\nend\n",
    )
    .expect("failed to write dynamic DIM fixture");

    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("jvm")
        .arg("--clean")
        .arg("--run")
        .current_dir(work_dir.path())
        .output()
        .expect("failed to invoke bcc");
    assert!(
        output.status.success(),
        "function-bound DIM failed under --target jvm:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("42,1"),
        "typed DIM bound function should run once: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn jvm_nested_dynamic_dim_allocates_only_in_typed_branch_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: java is unavailable", module_path!());
        return;
    }
    let work_dir = tempfile::tempdir().expect("failed to create nested DIM work directory");
    let source_path = work_dir.path().join("nested_dynamic_dim.bcl");
    fs::write(
        &source_path,
        "program NestedDynamicDim\ndim bound&\nbound& = 3\nflag% = 1\nif flag% then\ndim values%(bound&)\nvalues%(3) = 42\nend if\nprint values%(3)\nend\n",
    )
    .expect("failed to write nested dynamic DIM fixture");

    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("jvm")
        .arg("--clean")
        .arg("--run")
        .current_dir(work_dir.path())
        .output()
        .expect("failed to invoke bcc");
    assert!(
        output.status.success(),
        "nested dynamic DIM failed under --target jvm:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line == "42"),
        "nested typed DIM should allocate and retain its array: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn jvm_callable_dynamic_dim_uses_callable_typed_parameter_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: java is unavailable", module_path!());
        return;
    }
    let work_dir = tempfile::tempdir().expect("failed to create callable DIM work directory");
    let source_path = work_dir.path().join("callable_dynamic_dim.bcl");
    fs::write(
        &source_path,
        "program CallableDynamicDim\nfunction local%(bound&)\ndim values%(bound&)\nvalues%(bound&) = 42\nreturn values%(bound&)\nend function\nprint local%(3)\nend\n",
    )
    .expect("failed to write callable dynamic DIM fixture");

    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("jvm")
        .arg("--clean")
        .arg("--run")
        .current_dir(work_dir.path())
        .output()
        .expect("failed to invoke bcc");
    assert!(
        output.status.success(),
        "callable dynamic DIM failed under --target jvm:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line == "42"),
        "callable-local typed DIM should return its array value: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn jvm_callable_dynamic_dim_allocates_inside_typed_branch_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: java is unavailable", module_path!());
        return;
    }
    let work_dir = tempfile::tempdir().expect("failed to create callable branch DIM directory");
    let source_path = work_dir.path().join("callable_branch_dynamic_dim.bcl");
    fs::write(
        &source_path,
        "program CallableBranchDynamicDim\nfunction local%(bound%, flag%)\nif flag% then\ndim values%(bound%)\nvalues%(bound%) = 42\nend if\nreturn values%(bound%)\nend function\nprint local%(3, 1)\nend\n",
    )
    .expect("failed to write callable branch DIM fixture");

    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("jvm")
        .arg("--clean")
        .arg("--run")
        .current_dir(work_dir.path())
        .output()
        .expect("failed to invoke bcc");
    assert!(
        output.status.success(),
        "callable branch DIM failed under --target jvm:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line == "42"),
        "typed callable branch DIM should allocate before its array access: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn jvm_callable_dynamic_dim_evaluates_typed_bound_function_once_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: java is unavailable", module_path!());
        return;
    }
    let work_dir = tempfile::tempdir().expect("failed to create callable bound function directory");
    let source_path = work_dir.path().join("callable_bound_function_dim.bcl");
    fs::write(
        &source_path,
        "program CallableBoundFunctionDim\ncount%=0\nfunction nextBound%()\nglobal count%\ncount%=count%+1\nreturn 3\nend function\nfunction local%()\ndim values%(nextBound%())\nvalues%(3)=42\nreturn values%(3)\nend function\nprint local%();\",\";count%\nend\n",
    )
    .expect("failed to write callable bound function fixture");

    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("jvm")
        .arg("--clean")
        .arg("--run")
        .current_dir(work_dir.path())
        .output()
        .expect("failed to invoke bcc");
    assert!(
        output.status.success(),
        "callable bound function DIM failed under --target jvm:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line == "42,1"),
        "callable typed bound should be evaluated once before allocation: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn jvm_skipped_callable_dynamic_dim_does_not_evaluate_typed_bound_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: java is unavailable", module_path!());
        return;
    }
    let work_dir = tempfile::tempdir().expect("failed to create skipped callable DIM directory");
    let source_path = work_dir.path().join("skipped_callable_dynamic_dim.bcl");
    fs::write(
        &source_path,
        "program SkippedCallableDynamicDim\ncount%=0\nfunction nextBound%()\nglobal count%\ncount%=count%+1\nreturn 3\nend function\nfunction local%(flag%)\nglobal count%\nif flag% then\ndim values%(nextBound%())\nend if\nreturn count%\nend function\nprint local%(0)\nend\n",
    )
    .expect("failed to write skipped callable DIM fixture");

    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("jvm")
        .arg("--clean")
        .arg("--run")
        .current_dir(work_dir.path())
        .output()
        .expect("failed to invoke bcc");
    assert!(
        output.status.success(),
        "skipped callable DIM failed under --target jvm:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line == "0"),
        "a skipped typed DIM must not evaluate its bound: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn jvm_dynamic_string_array_uses_typed_capacity_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: java is unavailable", module_path!());
        return;
    }
    let work_dir = tempfile::tempdir().expect("failed to create dynamic string array directory");
    let source_path = work_dir.path().join("dynamic_string_array.bcl");
    fs::write(
        &source_path,
        "program DynamicStringArray\nbound%=2\ndim values$(bound%)\nvalues$(2)=\"typed\"\nprint values$(2)\nend\n",
    )
    .expect("failed to write dynamic string array fixture");

    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("jvm")
        .arg("--clean")
        .arg("--run")
        .current_dir(work_dir.path())
        .output()
        .expect("failed to invoke bcc");
    assert!(
        output.status.success(),
        "dynamic string array failed under --target jvm:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line == "typed"),
        "JVM dynamic string array should preserve the typed capacity and value: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn jvm_callable_dynamic_string_array_uses_local_typed_slot_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: java is unavailable", module_path!());
        return;
    }
    let work_dir =
        tempfile::tempdir().expect("failed to create callable dynamic string array directory");
    let source_path = work_dir.path().join("callable_dynamic_string_array.bcl");
    fs::write(
        &source_path,
        "program CallableDynamicStringArray\nfunction local$(bound&)\ndim values$(bound&)\nvalues$(bound&)=\"typed\"\nreturn values$(bound&)\nend function\nprint local$(2)\nend\n",
    )
    .expect("failed to write callable dynamic string array fixture");

    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("jvm")
        .arg("--clean")
        .arg("--run")
        .current_dir(work_dir.path())
        .output()
        .expect("failed to invoke bcc");
    assert!(
        output.status.success(),
        "callable dynamic string array failed under --target jvm:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line == "typed"),
        "callable JVM dynamic string array should preserve its typed slot and value: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn jvm_dynamic_long_array_uses_typed_capacity_and_element_type_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: java is unavailable", module_path!());
        return;
    }
    let work_dir = tempfile::tempdir().expect("failed to create dynamic LONG array directory");
    let source_path = work_dir.path().join("dynamic_long_array.bcl");
    fs::write(
        &source_path,
        "program DynamicLongArray\nbound&=3\ndim values&(bound&)\nvalues&(bound&)=123456789\nprint values&(bound&)\nend\n",
    )
    .expect("failed to write dynamic LONG array fixture");

    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("jvm")
        .arg("--clean")
        .arg("--run")
        .current_dir(work_dir.path())
        .output()
        .expect("failed to invoke bcc");
    assert!(
        output.status.success(),
        "dynamic LONG array failed under --target jvm:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line == "123456789"),
        "JVM dynamic LONG array should retain its typed capacity and element: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn jvm_dynamic_multidimensional_array_uses_typed_axes_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: java is unavailable", module_path!());
        return;
    }
    let work_dir =
        tempfile::tempdir().expect("failed to create dynamic multidimensional JVM array directory");
    let source_path = work_dir.path().join("dynamic_multidimensional_array.bcl");
    fs::write(
        &source_path,
        "program DynamicMultidimensionalArray\nrows&=2\ncols%=3\ndim grid%(rows&, cols%)\ngrid%(2,3)=42\nprint grid%(2,3)\nend\n",
    )
    .expect("failed to write dynamic multidimensional JVM array fixture");

    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("jvm")
        .arg("--clean")
        .arg("--run")
        .current_dir(work_dir.path())
        .output()
        .expect("failed to invoke bcc");
    assert!(
        output.status.success(),
        "dynamic multidimensional array failed under --target jvm:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line == "42"),
        "JVM dynamic array should retain each typed axis: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn jvm_dynamic_array_preserves_fixed_radix_axis_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: java is unavailable", module_path!());
        return;
    }
    let work_dir = tempfile::tempdir().expect("failed to create mixed-axis JVM array directory");
    let source_path = work_dir.path().join("mixed_axis_dynamic_array.bcl");
    fs::write(
        &source_path,
        "program MixedAxisDynamicArray\nlimit%=3\ndim grid%(&H10, limit%)\ngrid%(16,3)=42\nprint grid%(16,3)\nend\n",
    )
    .expect("failed to write mixed-axis JVM array fixture");

    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("jvm")
        .arg("--clean")
        .arg("--run")
        .current_dir(work_dir.path())
        .output()
        .expect("failed to invoke bcc");
    assert!(
        output.status.success(),
        "mixed-axis dynamic array failed under --target jvm:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line == "42"),
        "JVM dynamic allocation should combine fixed radix and runtime typed axes: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn jvm_dynamic_array_preserves_fixed_octal_axis_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: java is unavailable", module_path!());
        return;
    }
    let work_dir = tempfile::tempdir().expect("failed to create octal mixed-axis JVM directory");
    let source_path = work_dir.path().join("octal_mixed_axis_array.bcl");
    fs::write(
        &source_path,
        "program OctalMixedAxisArray\nlimit%=3\ndim grid%(&O20,limit%)\ngrid%(16,3)=42\nprint grid%(16,3)\nend\n",
    )
    .expect("failed to write octal mixed-axis JVM fixture");

    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("jvm")
        .arg("--clean")
        .arg("--run")
        .current_dir(work_dir.path())
        .output()
        .expect("failed to invoke bcc");
    assert!(
        output.status.success(),
        "octal mixed-axis array failed under --target jvm:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line == "42"),
        "JVM dynamic array should retain its fixed octal axis: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn jvm_callable_dynamic_array_preserves_fixed_radix_axis_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: java is unavailable", module_path!());
        return;
    }
    let work_dir =
        tempfile::tempdir().expect("failed to create callable mixed-axis JVM array directory");
    let source_path = work_dir.path().join("callable_mixed_axis_array.bcl");
    fs::write(
        &source_path,
        "program CallableMixedAxisArray\nfunction readValue%(limit%)\ndim grid%(&H10, limit%)\ngrid%(16,limit%)=42\nreturn grid%(16,limit%)\nend function\nprint readValue%(3)\nend\n",
    )
    .expect("failed to write callable mixed-axis JVM array fixture");

    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("jvm")
        .arg("--clean")
        .arg("--run")
        .current_dir(work_dir.path())
        .output()
        .expect("failed to invoke bcc");
    assert!(
        output.status.success(),
        "callable mixed-axis array failed under --target jvm:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line == "42"),
        "callable JVM array should preserve fixed radix and dynamic typed axes: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn jvm_dynamic_multidimensional_bounds_each_evaluate_once_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: java is unavailable", module_path!());
        return;
    }
    let work_dir =
        tempfile::tempdir().expect("failed to create dynamic axis evaluation directory");
    let source_path = work_dir.path().join("dynamic_axis_evaluation.bcl");
    fs::write(
        &source_path,
        "program DynamicAxisEvaluation\ncount%=0\nfunction nextBound%()\nglobal count%\ncount%=count%+1\nreturn 3\nend function\ndim grid%(nextBound%(),nextBound%())\ngrid%(3,3)=42\nresult%=grid%(3,3)\nprint result%;\",\";count%\nend\n",
    )
    .expect("failed to write dynamic axis evaluation fixture");

    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("jvm")
        .arg("--clean")
        .arg("--run")
        .current_dir(work_dir.path())
        .output()
        .expect("failed to invoke bcc");
    assert!(
        output.status.success(),
        "dynamic axis evaluation failed under --target jvm:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line == "42,2"),
        "each typed array axis bound should run exactly once: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn jvm_callable_dynamic_multidimensional_array_uses_typed_local_slot_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: java is unavailable", module_path!());
        return;
    }
    let work_dir = tempfile::tempdir()
        .expect("failed to create callable multidimensional JVM array directory");
    let source_path = work_dir.path().join("callable_multidimensional_array.bcl");
    fs::write(
        &source_path,
        "program CallableMultidimensionalArray\nfunction local%(rows&, cols%)\ndim grid%(rows&,cols%)\ngrid%(2,3)=42\nreturn grid%(2,3)\nend function\nprint local%(2,3)\nend\n",
    )
    .expect("failed to write callable multidimensional array fixture");

    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("jvm")
        .arg("--clean")
        .arg("--run")
        .current_dir(work_dir.path())
        .output()
        .expect("failed to invoke bcc");
    assert!(
        output.status.success(),
        "callable multidimensional array failed under --target jvm:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line == "42"),
        "callable JVM multidimensional array should use its typed local slot: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn jvm_dynamic_dim_transpiles_typed_arithmetic_bound_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: java is unavailable", module_path!());
        return;
    }
    let work_dir = tempfile::tempdir().expect("failed to create arithmetic DIM directory");
    let source_path = work_dir.path().join("arithmetic_dynamic_dim.bcl");
    fs::write(
        &source_path,
        "program ArithmeticDynamicDim\nbase%=2\ndim values%(base%+1)\nvalues%(3)=42\nprint values%(3)\nend\n",
    )
    .expect("failed to write arithmetic DIM fixture");

    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("jvm")
        .arg("--clean")
        .arg("--run")
        .current_dir(work_dir.path())
        .output()
        .expect("failed to invoke bcc");
    assert!(
        output.status.success(),
        "arithmetic dynamic DIM failed under --target jvm:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line == "42"),
        "JVM dynamic DIM should evaluate its typed arithmetic bound: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn jvm_dynamic_array_preserves_zero_upper_bound_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: java is unavailable", module_path!());
        return;
    }
    let work_dir = tempfile::tempdir().expect("failed to create zero-bound JVM directory");
    let source_path = work_dir.path().join("zero_bound_dynamic_array.bcl");
    fs::write(
        &source_path,
        "program ZeroBoundDynamicArray\nbound%=0\ndim values%(bound%)\nvalues%(0)=42\nprint values%(0)\nend\n",
    )
    .expect("failed to write zero-bound JVM fixture");

    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("jvm")
        .arg("--clean")
        .arg("--run")
        .current_dir(work_dir.path())
        .output()
        .expect("failed to invoke bcc");
    assert!(
        output.status.success(),
        "zero-bound JVM program failed to build or run:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line == "42"),
        "JVM zero upper bound must allocate one element: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn jvm_skipped_dynamic_dim_does_not_evaluate_typed_bound_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: java is unavailable", module_path!());
        return;
    }
    let work_dir = tempfile::tempdir().expect("failed to create skipped DIM JVM directory");
    let source_path = work_dir.path().join("skipped_dynamic_dim.bcl");
    fs::write(
        &source_path,
        "program SkippedDynamicDim\ncount%=0\nfunction nextBound%()\nglobal count%\ncount%=count%+1\nreturn 3\nend function\nif 0 then\ndim values%(nextBound%())\nend if\nprint count%\nend\n",
    )
    .expect("failed to write skipped DIM JVM fixture");

    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("jvm")
        .arg("--clean")
        .arg("--run")
        .current_dir(work_dir.path())
        .output()
        .expect("failed to invoke bcc");
    assert!(
        output.status.success(),
        "skipped DIM JVM program failed to build or run:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line == "0"),
        "a skipped typed JVM DIM must not evaluate its bound: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn jvm_dynamic_color_expressions_run_when_available() {
    if !jvm_runtime_available() {
        return;
    }
    let root = repo_root();
    let temp_dir = tempfile::tempdir().expect("failed to create dynamic COLOR fixture directory");
    let source = temp_dir.path().join("dynamic_color_semantic.bcl");
    fs::write(
        &source,
        "program DynamicSemanticColor\nforeground% = 4\nbackground% = 1\ncolor foreground%, background%\nprint \"x\"\nend\n",
    ).expect("failed to write dynamic COLOR fixture");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source)
        .arg("--target")
        .arg("jvm")
        .arg("--clean")
        .arg("--run")
        .current_dir(&root)
        .output()
        .expect("failed to invoke bcc for dynamic COLOR fixture");
    assert!(
        output.status.success(),
        "dynamic COLOR fixture failed under --target jvm:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    let stdout = String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n");
    assert!(
        stdout.contains("\u{1b}[31;44mx\n"),
        "typed dynamic palette mapping missing: {stdout:?}"
    );
}

/// `TAB(n)`/`SPC(n)` (print-position directives) and a dynamic (non-
/// literal) `LOCATE row%, col%` -- checked against the exact output
/// `codegen_c.rs`'s already-correct implementation produces for the same
/// statements.
#[test]
fn jvm_tab_spc_and_dynamic_locate_match_c_backend_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: java is unavailable", module_path!());
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
        eprintln!("skipping {}: java is unavailable", module_path!());
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
        eprintln!("skipping {}: java is unavailable", module_path!());
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
        eprintln!("skipping {}: java is unavailable", module_path!());
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
        .ends_with("2\n3\n1\n0\n2\n7\n4\nhello world\n9\n"));
}

#[test]
fn jvm_multi_index_arrays_run_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: java is unavailable", module_path!());
        return;
    }
    let temp_dir = tempfile::tempdir().expect("failed to create JVM multi-index array directory");
    let source_path = temp_dir.path().join("rank_two.bcl");
    fs::write(
        &source_path,
        "program RankTwo\ndim grid%(2, 3)\ndim labels(2, 3) as string\ngrid%(2, 3) = 41\nlabels(2, 3) = \"ready\"\nprint grid%(2, 3)\nprint labels(2, 3)\nend\n",
    ).expect("failed to write rank-two JVM fixture");
    let mut output_arg = temp_dir.path().join("out").into_os_string();
    output_arg.push("/");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("jvm")
        .arg("--clean")
        .arg("--run")
        .arg("-o")
        .arg(output_arg)
        .current_dir(temp_dir.path())
        .output()
        .expect("failed to invoke bcc");
    assert!(
        output.status.success(),
        "rank-two JVM array fixture failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .replace("\r\n", "\n")
            .ends_with("41\nready\n"),
        "JVM multi-index reads returned unexpected values: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn jvm_semantic_function_calls_pass_typed_byref_arrays_when_available() {
    let root = repo_root();
    let dir = tempfile::tempdir().expect("failed to create JVM byref array call directory");
    let source = dir.path().join("byref_array_call.bcl");
    fs::write(
        &source,
        "program ByRefArrayCall\nfunction increment%(byref values%(?))\nvalues%(0)=values%(0)+5\nreturn values%(0)\nend function\ndim values%(1)\nvalues%(0)=1\nresult%=increment%(values%)\nprint result%;\",\";values%(0)\nend\n",
    )
    .expect("failed to write byref array call fixture");
    let assembly_dir =
        tempfile::tempdir().expect("failed to create byref array assembly directory");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("ByRefArrayCall")
        .current_dir(dir.path())
        .output()
        .expect("failed to run assembled byref array call class");
    assert!(
        run.status.success(),
        "assembled byref array call failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"),
        "6,6\n"
    );
}

#[test]
fn jvm_byval_array_procedure_receives_a_copy() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: java is unavailable", module_path!());
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
        output.status.success(),
        "JVM byval array fixture failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let lines = String::from_utf8_lossy(&output.stdout)
        .replace("\r\n", "\n")
        .lines()
        .rev()
        .take(2)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    assert_eq!(lines, "1\n2\n");
}

#[test]
fn jvm_semantic_function_calls_copy_typed_byval_arrays_when_available() {
    let root = repo_root();
    let dir = tempfile::tempdir().expect("failed to create JVM byval array call directory");
    let source = dir.path().join("byval_array_call.bcl");
    fs::write(
        &source,
        "program ByValArrayCall\ndim grid%(1,1)\ngrid%(0,0)=1\ngrid%(1,1)=2\nfunction mutate%(values%(?,?))\nvalues%(0,0)=99\nreturn values%(0,0)\nend function\nresult%=mutate%(grid%)\ndim words$(1)\nwords$(0)=\"original\"\nfunction change$(values$(?))\nvalues$(0)=\"changed\"\nreturn values$(0)\nend function\ntext$=change$(words$)\nprint result%;\",\";grid%(0,0);\",\";grid%(1,1);\",\";text$;\",\";words$(0)\nend\n",
    )
    .expect("failed to write byval array call fixture");
    let assembly_dir =
        tempfile::tempdir().expect("failed to create byval array assembly directory");
    compile_and_assemble(&source, assembly_dir.path());
    if !java_available() {
        return;
    }
    let run = Command::new("java")
        .arg("-cp")
        .arg(root.join("tmp"))
        .arg("ByValArrayCall")
        .current_dir(dir.path())
        .output()
        .expect("failed to run assembled byval array call class");
    assert!(
        run.status.success(),
        "assembled byval array call failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"),
        "99,1,2,changed,original\n"
    );
}

#[test]
fn jvm_scalar_method_calls_copy_byval_array_arguments_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: Java runtime is unavailable", module_path!());
        return;
    }
    let dir = tempfile::tempdir().expect("failed to create JVM scalar method array directory");
    let source = dir.path().join("scalar_method_byval_array.bcl");
    fs::write(
        &source,
        "program ScalarMethodByValArray\nmethod mutate%[integer](byval values%(?))\nvalues%(0)=99\nreturn self%+values%(0)\nend method\ndim values%(1)\nvalues%(0)=1\nbase%=1\nresult%=base%.mutate(values%)\nprint result%;\",\";values%(0)\nend\n",
    )
    .expect("failed to write scalar method byval array fixture");
    let output_dir = tempfile::tempdir().expect("failed to create scalar method output directory");
    let mut output_arg = output_dir.path().as_os_str().to_owned();
    output_arg.push("/");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source)
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
        "JVM scalar method byval array fixture failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let lines = String::from_utf8_lossy(&output.stdout)
        .replace("\r\n", "\n")
        .lines()
        .rev()
        .take(1)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    assert_eq!(lines, "100,1\n");
}

#[test]
fn jvm_scalar_method_calls_pass_byref_array_arguments_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: Java runtime is unavailable", module_path!());
        return;
    }
    let dir = tempfile::tempdir().expect("failed to create JVM scalar method array directory");
    let source = dir.path().join("scalar_method_byref_array.bcl");
    fs::write(
        &source,
        "program ScalarMethodByRefArray\nmethod mutate%[integer](byref values%(?))\nvalues%(0)=99\nreturn self%+values%(0)\nend method\ndim values%(1)\nvalues%(0)=1\nbase%=1\nresult%=base%.mutate(values%)\nprint result%;\",\";values%(0)\nend\n",
    )
    .expect("failed to write scalar method byref array fixture");
    let output_dir = tempfile::tempdir().expect("failed to create scalar method output directory");
    let mut output_arg = output_dir.path().as_os_str().to_owned();
    output_arg.push("/");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source)
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
        "JVM scalar method byref array fixture failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let lines = String::from_utf8_lossy(&output.stdout)
        .replace("\r\n", "\n")
        .lines()
        .rev()
        .take(1)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    assert_eq!(lines, "100,99\n");
}

#[test]
fn jvm_scalar_method_calls_write_back_byref_string_arguments_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: Java runtime is unavailable", module_path!());
        return;
    }
    let dir = tempfile::tempdir().expect("failed to create JVM scalar string method directory");
    let source = dir.path().join("scalar_method_byref_string.bcl");
    fs::write(
        &source,
        "program ScalarMethodByRefString\nmethod append$[string](byref text$)\ntext$=text$+\"!\"\nreturn self$+text$\nend method\noriginal$=\"original\"\ntyped$=\"typed\"\nbase$=\"base:\"\nresult$=base$.append(original$)\nprint result$;\",\";original$;\",\";typed$\nend\n",
    )
    .expect("failed to write scalar string method fixture");
    let output_dir =
        tempfile::tempdir().expect("failed to create scalar string method output directory");
    let mut output_arg = output_dir.path().as_os_str().to_owned();
    output_arg.push("/");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source)
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
        "JVM scalar method string ByRef fixture failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let lines = String::from_utf8_lossy(&output.stdout)
        .replace("\r\n", "\n")
        .lines()
        .rev()
        .take(1)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    assert_eq!(lines, "base:original!,original!,typed\n");
}

#[test]
fn jvm_scalar_method_double_receiver_widens_integer_argument_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: Java runtime is unavailable", module_path!());
        return;
    }
    let dir = tempfile::tempdir().expect("failed to create JVM double method directory");
    let source = dir.path().join("scalar_method_double_coercion.bcl");
    fs::write(
        &source,
        "program ScalarMethodDoubleCoercion\nmethod adjust#[double](step#)\nreturn self#+step#\nend method\nvalue#=2.5\nresult#=value#.adjust(3)\nprint result#\nend\n",
    )
    .expect("failed to write scalar double method fixture");
    let output_dir =
        tempfile::tempdir().expect("failed to create scalar double method output directory");
    let mut output_arg = output_dir.path().as_os_str().to_owned();
    output_arg.push("/");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source)
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
        "JVM scalar double method fixture failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let lines = String::from_utf8_lossy(&output.stdout)
        .replace("\r\n", "\n")
        .lines()
        .rev()
        .take(1)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    assert_eq!(lines, "5.5\n");
}

#[test]
fn jvm_function_and_scalar_method_default_arguments_run_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: Java runtime is unavailable", module_path!());
        return;
    }
    let dir = tempfile::tempdir().expect("failed to create JVM default argument directory");
    let source = dir.path().join("typed_default_arguments.bcl");
    fs::write(
        &source,
        "program TypedDefaultArguments\nfunction add%(left%, right%=3)\nreturn left%+right%\nend function\nmethod adjust%[integer](step%=4)\nreturn self%+step%\nend method\nfunctionResult%=add%(5)\nbase%=10\nmethodResult%=base%.adjust()\nprint functionResult%;\",\";methodResult%\nend\n",
    )
    .expect("failed to write typed default argument fixture");
    let output_dir =
        tempfile::tempdir().expect("failed to create default argument output directory");
    let mut output_arg = output_dir.path().as_os_str().to_owned();
    output_arg.push("/");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source)
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
        "JVM default argument fixture failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let lines = String::from_utf8_lossy(&output.stdout)
        .replace("\r\n", "\n")
        .lines()
        .rev()
        .take(1)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    assert_eq!(lines, "8,14\n");
}

#[test]
fn jvm_string_and_constant_default_arguments_run_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: Java runtime is unavailable", module_path!());
        return;
    }
    let dir = tempfile::tempdir().expect("failed to create JVM const default directory");
    let source = dir.path().join("typed_const_and_string_defaults.bcl");
    fs::write(
        &source,
        "program TypedConstAndStringDefaults\nconst DEFAULT_INCREMENT = 6\nfunction add%(left%, right%=DEFAULT_INCREMENT)\nreturn left%+right%\nend function\nmethod append$[string](ending$=\"!\")\nreturn self$+ending$\nend method\nnumber%=add%(2)\ntext$=\"typed\".append()\nprint number%;\",\";text$\nend\n",
    )
    .expect("failed to write JVM const and string defaults fixture");
    let output_dir = tempfile::tempdir().expect("failed to create const default output directory");
    let mut output_arg = output_dir.path().as_os_str().to_owned();
    output_arg.push("/");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source)
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
        "JVM const/string default fixture failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let lines = String::from_utf8_lossy(&output.stdout)
        .replace("\r\n", "\n")
        .lines()
        .rev()
        .take(1)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    assert_eq!(lines, "8,typed!\n");
}

#[test]
fn jvm_catch_filters_and_source_bindings_run_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: java is unavailable", module_path!());
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
        eprintln!("skipping {}: java is unavailable", module_path!());
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

/// Compile `source_path` to a temporary `.j` file and assemble it. Assembly
/// (`krakatau2::assemble`) is linked directly into `bcc` -- see
/// `Cargo.toml`'s own comment -- so there's no external tool to be missing
/// here; a failure is always a real test failure.
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
        panic!(
            "bcc failed to compile/assemble {} under --target jvm:\nstdout:\n{}\nstderr:\n{}",
            source_path.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
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
/// Needs no `java` -- transpiling (not running) already exercises
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
/// file" error) was invisible to every one of them. Needs no `java`
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
/// would only unwind one call frame). Needs no `java` -- transpiling
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
        eprintln!("skipping {}: java is unavailable", module_path!());
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
/// in-memory state. Skipped (not failed) when `java` isn't
/// available, matching this file's other end-to-end tests.
#[test]
fn card_catalog_example_runs_under_jvm_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: java is unavailable", module_path!());
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

/// Regression test for JVM scalar `byref` parameters (no native scalar
/// reference semantics on the JVM, unlike C's pointers -- see the
/// `byref_scalar_params`/scratch-slot machinery in codegen_jvm.rs):
/// mutations a callee makes to a `byref` scalar must be visible to the
/// caller after the call returns. Exercises several things at once that a
/// narrower test wouldn't: four simultaneous `byref` scalars of mixed
/// types (String/int/int/double) in one call, an early `return` from
/// inside a conditional (which must still write back before returning,
/// not just at the procedure's final fallthrough), and a second call
/// reusing the same scratch slots to prove they're safe to reuse
/// sequentially rather than colliding.
#[test]
fn jvm_byref_scalar_parameters_write_back_to_the_caller_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: java is unavailable", module_path!());
        return;
    }
    let dir = tempfile::tempdir().expect("failed to create JVM byref test directory");
    let source_path = dir.path().join("byref_scalars.bcl");
    fs::write(
        &source_path,
        "program byrefScalars\n\
         procedure gather(which%, byref name$, byref count%, byref limit%, byref price!)\n\
         \x20   if which% = 1 then\n\
         \x20       name$ = \"first\"\n\
         \x20       count% = 1\n\
         \x20       limit% = 10\n\
         \x20       price! = 1.5\n\
         \x20       return\n\
         \x20   end if\n\
         \x20   name$ = \"second\"\n\
         \x20   count% = 2\n\
         \x20   limit% = 20\n\
         \x20   price! = 2.5\n\
         end procedure\n\
         n1$ = \"\"\n\
         c1% = 0\n\
         l1% = 0\n\
         p1! = 0\n\
         n2$ = \"\"\n\
         c2% = 0\n\
         l2% = 0\n\
         p2! = 0\n\
         gather(1, n1$, c1%, l1%, p1!)\n\
         gather(2, n2$, c2%, l2%, p2!)\n\
         print n1$ + \" \" + str$(c1%) + \" \" + str$(l1%) + \" \" + str$(p1!)\n\
         print n2$ + \" \" + str$(c2%) + \" \" + str$(l2%) + \" \" + str$(p2!)\n\
         end\n",
    )
    .expect("failed to write byref scalars fixture");
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
        "byref scalars fixture failed under --target jvm:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n");
    assert!(
        stdout.contains("first 1 10 1.5") && stdout.contains("second 2 20 2.5"),
        "expected both calls' byref writes to reach the caller:\n{stdout}"
    );
}

/// Regression test: calling a non-`void` function as a bare statement
/// (discarding its return value) must compile and run, not just when its
/// result is used in an expression. `readKey$()` in `tutorial/inventory.bcl`
/// is called exactly this way (`readKey$()` on its own line, just to
/// consume a keystroke) -- the statement-call codegen path previously
/// required `signature.returns_void` and rejected this with "invalid JVM
/// procedure call".
#[test]
fn jvm_function_call_as_bare_statement_discards_its_result_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: java is unavailable", module_path!());
        return;
    }
    let dir = tempfile::tempdir().expect("failed to create JVM bare-call test directory");
    let source_path = dir.path().join("bare_call.bcl");
    fs::write(
        &source_path,
        "program bareCall\n\
         function bump%()\n\
         \x20   global n%\n\
         \x20   n% = n% + 1\n\
         \x20   return n%\n\
         end function\n\
         n% = 0\n\
         bump%()\n\
         bump%()\n\
         print \"n=\" + str$(n%)\n\
         end\n",
    )
    .expect("failed to write bare-call fixture");
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
        "bare function-call-as-statement fixture failed under --target jvm:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("n=2"),
        "expected two discarded calls to still run their side effects:\n{stdout}"
    );
}

/// Regression test: `GET` on a freshly-created, empty random-access file
/// must not crash. Java's `RandomAccessFile.readFully()` throws
/// `EOFException` on a short/empty read; C's `fread`-based approach
/// silently returns a short count and leaves the (already zero-initialized)
/// buffer as-is -- `emit_get_or_put` must match that lenient behavior
/// (plain `read()`, discarding the count) rather than `readFully()`, or
/// `GET`-ing record 1 of a brand-new file crashes instead of reading zeros.
#[test]
fn jvm_get_on_a_fresh_empty_file_does_not_throw_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: java is unavailable", module_path!());
        return;
    }
    let work_dir = std::env::temp_dir().join("bascal-jvm-conformance-get-on-fresh-file");
    let _ = fs::remove_dir_all(&work_dir);
    fs::create_dir_all(&work_dir).expect("failed to create work directory");
    let source_path = work_dir.join("get_fresh.bcl");
    fs::write(
        &source_path,
        "program getFresh\n\
         open \"fresh.dat\" for random as #1 len = 4\n\
         field #1, 4 as buf$\n\
         get #1, 1\n\
         print \"len=\" + str$(buf$.len())\n\
         close #1\n\
         end\n",
    )
    .expect("failed to write GET-on-fresh-file fixture");
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
        "GET-on-fresh-file fixture failed under --target jvm:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("len=4"),
        "expected GET on an empty file to leave the zero-initialized buffer intact:\n{stdout}"
    );
}

/// Regression test for a real bug: after `INKEY$` puts the terminal into
/// raw, non-blocking (`min 0 time 0`) mode, a later blocking `INPUT` must
/// still work. On a real terminal this required `emit_input` to bracket its
/// `readLine()` with `stty sane` / `emit_inkey_setup`'s raw mode again (see
/// `emit_input`'s own doc comment) -- without that, `readLine()` returned
/// `null` instead of blocking, crashing the `.trim()`/`parseInt` that
/// follows. `stty` legitimately no-ops against a pipe (see
/// `jvm_inkey_polls_without_crashing_under_piped_input_when_available`'s own
/// doc comment), so this can't reproduce the raw-mode hang itself without a
/// pty this project doesn't take a dependency on -- but it does confirm the
/// combination compiles, runs, and reads the right value under piped input,
/// which is what actually regressed when this fix first landed (a
/// `NumberFormatException` from `--target jvm`'s TRY/CATCH treating the
/// resulting `NullPointerException`'s message as an error code -- see
/// `emit_try_catch`'s `getMessage`/`parseInt` handling in codegen_jvm.rs).
/// True raw-mode behavior was verified by hand against a real pseudo-tty.
#[test]
fn jvm_input_after_inkey_reads_the_typed_value_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: java is unavailable", module_path!());
        return;
    }
    let dir = tempfile::tempdir().expect("failed to create JVM input-after-inkey test directory");
    let source_path = dir.path().join("input_after_inkey.bcl");
    fs::write(
        &source_path,
        "program inputAfterInkey\n\
         k$ = \"\"\n\
         do while k$ = \"\"\n\
         \x20   k$ = inkey$\n\
         loop\n\
         print \"got: \" + k$\n\
         input \"number\"; n%\n\
         print \"you typed:\" + str$(n%)\n\
         end\n",
    )
    .expect("failed to write input-after-inkey fixture");
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
        "bcc failed to compile/assemble the input-after-inkey fixture"
    );

    let mut child = Command::new("java")
        .arg("-cp")
        .arg(repo_root().join("tmp"))
        .arg("InputAfterInkey")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn InputAfterInkey");
    child
        .stdin
        .take()
        .expect("child stdin should be piped")
        .write_all(b"X42\n")
        .expect("failed to write keystrokes to InputAfterInkey");

    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if child
            .try_wait()
            .expect("failed to poll InputAfterInkey")
            .is_some()
        {
            break;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("InputAfterInkey timed out after 30 seconds");
        }
        thread::sleep(Duration::from_millis(50));
    }
    let run = child
        .wait_with_output()
        .expect("failed to collect InputAfterInkey output");
    assert!(
        run.status.success(),
        "InputAfterInkey exited non-zero:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    let stdout = String::from_utf8_lossy(&run.stdout);
    assert!(
        stdout.contains("got: X") && stdout.contains("you typed:42"),
        "expected INKEY$ then INPUT to both read correctly:\n{stdout}"
    );
}

/// Regression test for a real bug: `STR$`/bare-numeric `PRINT` of a
/// `single`/`double` value used Java's own `String.valueOf(double)`, which
/// always prints full round-trip precision (17 significant digits). A real
/// BASIC `single` unpacked from its 32-bit on-disk form and widened to
/// `double` (this backend represents every `single`/`double` as a JVM
/// `double` -- see `TypeSuffix::Single | TypeSuffix::Double`) makes that
/// widening's own rounding noise visible verbatim: `0.03` printed as
/// `0.029999999329447746` (`tutorial/inventory.bcl`'s own `price!` field,
/// found interactively). `emit_double_str_helper`'s `bccStr` now rounds to
/// 6 significant digits and drops trailing zeros first, matching
/// `codegen_c.rs`'s own `bcc_strd` (`"% g"` `snprintf` formatting).
#[test]
fn jvm_double_to_string_rounds_to_six_significant_digits_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: java is unavailable", module_path!());
        return;
    }
    let dir = tempfile::tempdir().expect("failed to create JVM double-formatting test directory");
    let source_path = dir.path().join("double_format.bcl");
    fs::write(
        &source_path,
        "program doubleFormat\n\
         p! = 0.03\n\
         print str$(p!)\n\
         q! = 42\n\
         print str$(q!)\n\
         print 2 ^ 8\n\
         end\n",
    )
    .expect("failed to write double-formatting fixture");
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
        "double-formatting fixture failed under --target jvm:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n");
    assert!(
        stdout.ends_with("0.03\n42\n256\n"),
        "expected rounded, trailing-zero-free formatting:\n{stdout}"
    );
}

/// Regression test for a real bug: `emit_inkey_setup` used to run the `stty
/// raw` canned mode, which also clears `opost` (output post-processing) --
/// the tty driver flag that translates a bare `\n` into `\r\n` on the wire.
/// With `opost` off for the rest of the run (raw mode is left in effect
/// until program exit, not toggled per keystroke -- see `emit_inkey_setup`'s
/// own doc comment), every later `PRINT`'s `\n` stopped returning the
/// cursor to column 1: on a real terminal, `tutorial/inventory.bcl`'s
/// `listAll` looked like it was drifting rightward down the page, one line
/// at a time, when it was actually never returning to column 1 at all.
/// `-icanon -echo` (the fix) disables line-buffering and echo -- the two
/// properties `INKEY$` actually needs -- without touching `opost`. Needs no
/// `java`/pty: this pins the exact `stty` arguments in the
/// generated assembly text directly, which is both sufficient (the bug was
/// entirely in which flags get passed to `stty`) and the only way to catch
/// a regression back to the `raw` mode without a real pseudo-terminal (a
/// piped/non-tty test, like this file's other `INKEY$` coverage, can't
/// observe `opost`/`\r\n` translation at all -- there's no line discipline
/// on a pipe to misconfigure).
#[test]
fn jvm_inkey_setup_does_not_disable_output_postprocessing() {
    let file = tempfile::Builder::new()
        .suffix(".bcl")
        .tempfile()
        .expect("failed to create INKEY$ stty-flags fixture");
    fs::write(
        file.path(),
        "program inkeyStty\nk$ = inkey$\nprint k$\nend\n",
    )
    .expect("failed to write INKEY$ stty-flags fixture");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(file.path())
        .arg("--target")
        .arg("jvm")
        .arg("--clean")
        .output()
        .expect("failed to invoke bcc");
    assert!(
        output.status.success(),
        "INKEY$ stty-flags fixture failed to compile under --target jvm:\nstderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let j_path = file.path().with_extension("j");
    let generated = fs::read_to_string(&j_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", j_path.display()));
    assert!(
        generated.contains("\"-icanon\""),
        "expected the non-canonical-mode stty flag:\n{generated}"
    );
    assert!(
        !generated.contains("\"raw\""),
        "stty's `raw` canned mode also clears opost, breaking \\n -> \\r\\n \
         translation for the rest of the run:\n{generated}"
    );
}

/// Regression test for a real bug: `System.out`'s autoflush only triggers
/// on a `\n` byte or a `println` call, so a `print "...";`/`print "...",`
/// (no trailing newline -- `waitAnyKey()`'s own `"Press the AnyKey..."` in
/// `tutorial/inventory.bcl`) or `INPUT`'s own `"...? "` prompt sat
/// invisible in the buffer, on a real terminal, until something else
/// happened to flush it -- observed as the prompt only appearing *after* a
/// keystroke was read blind, with whatever printed next arriving all at
/// once right alongside it. `emit_print_tokens`/`emit_input` now flush
/// `System.out` explicitly right after a non-newline-terminated `print`/an
/// `INPUT` prompt. Needs no `java`: this pins the flush call in the
/// generated assembly text directly, which is sufficient (the bug was
/// entirely about whether the flush call is emitted at all).
#[test]
fn jvm_print_without_trailing_newline_flushes_stdout() {
    let file = tempfile::Builder::new()
        .suffix(".bcl")
        .tempfile()
        .expect("failed to create print-flush fixture");
    fs::write(
        file.path(),
        "program printFlush\nprint \"prompt\";\ninput \"n\"; x%\nprint \"done\"\nend\n",
    )
    .expect("failed to write print-flush fixture");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(file.path())
        .arg("--target")
        .arg("jvm")
        .arg("--clean")
        .output()
        .expect("failed to invoke bcc");
    assert!(
        output.status.success(),
        "print-flush fixture failed to compile under --target jvm:\nstderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let j_path = file.path().with_extension("j");
    let generated = fs::read_to_string(&j_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", j_path.display()));
    let flush_count = generated
        .matches("invokevirtual java/io/PrintStream/flush ()V")
        .count();
    assert_eq!(
        flush_count, 2,
        "expected a flush after both the bare `print \"prompt\";` and the \
         INPUT prompt (but not after `print \"done\"`, which already ends \
         in a newline):\n{generated}"
    );
}

/// Regression test for a real, source-level bug in `tutorial/inventory.bcl`
/// itself -- see `tests/examples.rs`'s own
/// `gcc_runs_inventory_list_all_without_a_garbled_press_any_key_prompt_when_available`
/// for the full mechanism (present identically under every target, since
/// it's a print/`LOCATE` sequencing bug in the `.bcl` source, not a
/// codegen one). Needs no pty: the bug is pure print/CLS ordering, not
/// terminal-size-dependent scroll timing, so piped stdin is sufficient to
/// catch a regression back to the glued/doubled
/// `"Press thePress the AnyKey..."` text.
#[test]
fn jvm_inventory_list_all_without_a_garbled_press_any_key_prompt_when_available() {
    if !jvm_runtime_available() {
        eprintln!("skipping {}: java is unavailable", module_path!());
        return;
    }
    let work_dir = std::env::temp_dir().join("bascal-jvm-conformance-inventory-list-all");
    let _ = fs::remove_dir_all(&work_dir);
    fs::create_dir_all(&work_dir).expect("failed to create work directory");

    let source_path = repo_root().join("tutorial/inventory.bcl");
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
        "bcc failed to compile/assemble inventory.bcl under --target jvm"
    );

    // "3" selects "List all" (100 parts, 20 per page -- 5 pages, 5
    // `waitAnyKey()` calls); one "x" per page to dismiss its prompt, then
    // "7" to exit back at the main menu.
    let mut child = Command::new("java")
        .arg("-cp")
        .arg(work_dir.join("tmp"))
        .arg("Inventory")
        .current_dir(&work_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn Inventory");
    child
        .stdin
        .take()
        .expect("child stdin should be piped")
        .write_all(b"3xxxxx7")
        .expect("failed to write keystrokes to Inventory");

    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if child
            .try_wait()
            .expect("failed to poll Inventory")
            .is_some()
        {
            break;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("Inventory timed out after 30 seconds");
        }
        thread::sleep(Duration::from_millis(50));
    }
    let run = child
        .wait_with_output()
        .expect("failed to collect Inventory output");
    assert!(
        run.status.success(),
        "Inventory exited non-zero:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let stdout = String::from_utf8_lossy(&run.stdout);
    assert!(
        !stdout.contains("Press thePress the"),
        "the initial header prompt and waitAnyKey()'s own prompt collided \
         on the same row:\n{stdout}"
    );
    assert_eq!(
        stdout.matches("I N V E N T O R Y   L I S T I N G").count(),
        5,
        "expected one freshly-redrawn header per page (5 pages of 20 for \
         100 parts):\n{stdout}"
    );
    assert_eq!(
        stdout.matches("Press the AnyKey to continue").count(),
        5,
        "expected exactly one wait prompt per page, each on its own \
         freshly-cleared row:\n{stdout}"
    );
}
