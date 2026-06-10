use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn is_library_path(path: &Path) -> bool {
    path.components().any(|c| {
        matches!(c.as_os_str().to_str(), Some("com" | "lib"))
    })
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
            if diagnostics
                .iter()
                .all(|d| d.message.contains("COMMON is only valid in suite files")) =>
        {
            return; // suite definition file — not a standalone compilable program
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

fn compile_with_cli(source_path: &Path, output_path: &Path, extra_args: &[&str]) {
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .unwrap_or_else(|err| panic!("failed to create {}: {err}", parent.display()));
    }

    let mut command = Command::new(env!("CARGO_BIN_EXE_bcc"));
    command.arg(source_path).arg("-o").arg(output_path);
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
