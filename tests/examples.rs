use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[test]
fn compiles_every_example_bcl_file() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let examples_dir = repo_root.join("examples");
    let output_dir = repo_root.join("output");
    fs::create_dir_all(&output_dir).expect("output directory should be creatable");

    let mut examples = fs::read_dir(&examples_dir)
        .expect("examples directory should exist")
        .map(|entry| {
            entry
                .expect("example directory entry should be readable")
                .path()
        })
        .filter(|path| path.extension().is_some_and(|extension| extension == "bcl"))
        .collect::<Vec<_>>();
    examples.sort();

    assert!(
        !examples.is_empty(),
        "expected at least one .bcl example in {}",
        examples_dir.display()
    );

    for example in examples {
        compile_example(&example, &output_dir);
    }
}

fn compile_example(path: &PathBuf, output_dir: &Path) {
    let options = bcc::CompileOptions::new();
    let output = bcc::compile_file(path, &options).unwrap_or_else(|diagnostics| {
        panic!("failed to compile {}:\n{diagnostics:#?}", path.display())
    });

    let output_path = output_dir
        .join(path.file_name().expect("example should have a file name"))
        .with_extension("bas");
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

fn assert_branch_targets_are_numeric(output: &str, path: &Path) {
    for line in output.lines() {
        if line_payload_is_comment(line) {
            continue;
        }
        for keyword in ["GOTO", "GOSUB"] {
            if let Some((_, target)) = line.split_once(keyword) {
                let target = target.trim();
                assert!(
                    target
                        .chars()
                        .next()
                        .is_some_and(|first| first.is_ascii_digit()),
                    "{} should use numeric {keyword} targets, got `{line}`",
                    path.display()
                );
            }
        }
    }
}

fn line_payload_is_comment(line: &str) -> bool {
    let payload = line
        .trim_start()
        .trim_start_matches(|ch: char| ch.is_ascii_digit())
        .trim_start();
    payload.starts_with('\'')
}

#[test]
fn freebasic_runs_sort_driver_when_available() {
    if Command::new("fbc").arg("-version").output().is_err() {
        return;
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_path = manifest_dir.join("examples/sort_driver.bcl");
    let output_dir = manifest_dir.join("output");
    let tmp_dir = manifest_dir.join("tmp");
    fs::create_dir_all(&output_dir).expect("output directory should be creatable");
    fs::create_dir_all(&tmp_dir).expect("tmp directory should be creatable");

    let output = bcc::compile_file(&source_path, &bcc::CompileOptions::new())
        .unwrap_or_else(|diagnostics| panic!("failed to compile sort driver:\n{diagnostics:#?}"));
    let basic_path = output_dir.join("sort_driver.bas");
    fs::write(&basic_path, output)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", basic_path.display()));

    let executable_path = tmp_dir.join("sort_driver_fbc");
    let compile = Command::new("fbc")
        .arg("-lang")
        .arg("qb")
        .arg(&basic_path)
        .arg("-x")
        .arg(&executable_path)
        .output()
        .expect("failed to run fbc");
    assert!(
        compile.status.success(),
        "fbc failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );

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
