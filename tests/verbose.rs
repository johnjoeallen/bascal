//! `bcc --verbose` narrates each step on stderr and leaves stdout alone.

use std::process::Command;

fn bcc(args: &[&str], dir: &std::path::Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_bcc"))
        .args(args)
        .current_dir(dir)
        .output()
        .expect("run bcc")
}

#[test]
fn verbose_reports_each_step_to_stderr() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("hello.bcl"), "program hello\nprint \"hi\"\nend\n").unwrap();
    let output = bcc(&["hello.bcl", "-t", "C", "--clean", "--verbose"], dir.path());
    assert!(output.status.success(), "{output:?}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    for step in [
        "bcc: input hello.bcl, target C",
        "bcc: transpiling hello.bcl",
        "bcc: wrote ",
    ] {
        assert!(stderr.contains(step), "missing {step:?}:\n{stderr}");
    }
    assert!(dir.path().join("hello.c").is_file());
}

#[test]
fn without_verbose_bcc_stays_quiet() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("hello.bcl"), "program hello\nprint \"hi\"\nend\n").unwrap();
    let output = bcc(&["hello.bcl", "-t", "C", "--clean"], dir.path());
    assert!(output.status.success(), "{output:?}");
    assert!(!String::from_utf8_lossy(&output.stderr).contains("bcc:"));
}
