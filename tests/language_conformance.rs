//! Cross-backend language conformance matrix.
//!
//! Each fixture is a small, deterministic language feature probe.  The
//! matrix records which backends are expected to accept it; backend-specific
//! runtime tests live beside this file (for example in `jvm_conformance.rs`
//! and `dosbox_conformance.rs`).
// Conformance groups: core, tutorials

use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn conformance_fixtures_transpile_on_their_supported_backends() {
    // Keep this list explicit: an unsupported backend is a documented
    // capability boundary, not an accidental omission from the suite.
    let cases: &[(&str, &[&str])] = &[
        ("const_and_print", &["basic", "c", "jvm"]),
        ("stdlib_functions", &["basic", "c", "jvm"]),
        ("tie_break_rounding", &["basic", "c", "jvm"]),
        ("string_self_concat", &["basic", "c"]),
        ("builtin_scalar_methods", &["basic", "c"]),
        ("mid_assign", &["basic"]),
        ("jvm_try", &["basic", "jvm"]),
        ("jvm_try_filter", &["basic", "jvm"]),
        ("jvm_noninteger_arrays", &["basic", "jvm"]),
    ];

    for (fixture, targets) in cases {
        let source = repo_root()
            .join("tests/fixtures/conformance")
            .join(format!("{fixture}.bcl"));
        let source = if source.exists() {
            source
        } else {
            repo_root()
                .join("tests/fixtures")
                .join(format!("{fixture}.bcl"))
        };
        assert!(
            source.exists(),
            "missing conformance fixture {}",
            source.display()
        );

        for target in *targets {
            let temp = tempfile::tempdir().expect("failed to create conformance output directory");
            let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
                .arg(&source)
                .arg("--target")
                .arg(target)
                .arg("--clean")
                .arg("-o")
                .arg(Path::new(temp.path()).join("out/"))
                .current_dir(repo_root())
                .output()
                .expect("failed to invoke bcc");
            assert!(
                output.status.success(),
                "{fixture} failed to transpile for {target}:\nstdout:\n{}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}
