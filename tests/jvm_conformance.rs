//! Opt-in end-to-end conformance tests for the bootstrap JVM target.
//!
//! `krak2` assembles the Krakatau text emitted by `--target jvm`, and a JRE
//! runs the resulting class.  Neither tool is a Rust dependency, so this
//! suite follows the other external-tool suites and skips rather than fails
//! when a prerequisite is unavailable.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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

/// Pending cross-backend record-file compatibility check. This is deliberately
/// ignored until JVM random-access file I/O exists; enabling it today should
/// fail because the JVM backend rejects `OPEN`/`FIELD`/`GET`/`PUT`.
#[test]
#[ignore = "expected failure until JVM random-access record I/O is implemented (#105)"]
fn jvm_record_binary_compatibility_with_basic_and_c_is_pending() {
    let source_path = repo_root().join("tests/fixtures/conformance/cross_write.bcl");
    let temp_dir = tempfile::tempdir().expect("failed to create JVM record test directory");
    let mut output_arg = temp_dir.path().as_os_str().to_owned();
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
    assert!(
        output.status.success(),
        "JVM record binary compatibility is not implemented yet:\n{}",
        String::from_utf8_lossy(&output.stderr)
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
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .replace("\r\n", "\n")
            .ends_with("2\n3\n1\n0\n2\n7\n4.0\nhello world\n9\n")
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
    let source_path = repo_root().join("tutorial/21_portable_error_handling.bcl");
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

    Some(output_dir.join("01_hello.j"))
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
    let source_path = repo_root.join("tutorial/01_hello.bcl");
    let expected_j_path = repo_root.join("tutorial/01_hello.j");
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
