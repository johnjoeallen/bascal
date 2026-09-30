//! The code in the docs is checked against the real compiler.
//!
//! `scripts/refresh_doc_generated.py --check --programs` fails when
//!
//! - a block the docs present as compiler output (tagged "Generated BASIC",
//!   marked `<!-- generated-basic -->`, or introduced by "Becomes:") differs
//!   from what `bcc` really generates for its BASCAL source, or
//! - a complete `program` example in the docs does not compile.
//!
//! Run `scripts/refresh_doc_generated.py` to rewrite stale generated blocks;
//! mark a deliberate non-compiling example with `<!-- no-compile -->` on the
//! line above its fence.

use std::path::Path;
use std::process::Command;

#[test]
fn docs_generated_blocks_are_current_and_example_programs_compile() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let script = root.join("scripts/refresh_doc_generated.py");
    let Ok(python) = ["python3", "python"]
        .into_iter()
        .find(|name| Command::new(name).arg("--version").output().is_ok())
        .ok_or(())
    else {
        eprintln!("skipping docs code check: python is unavailable");
        return;
    };
    let output = Command::new(python)
        .arg(&script)
        .args(["--check", "--programs"])
        .env("BCC", env!("CARGO_BIN_EXE_bcc"))
        .current_dir(root)
        .output()
        .expect("failed to run the docs code check");
    assert!(
        output.status.success(),
        "the docs are out of date with the compiler; run scripts/refresh_doc_generated.py\n{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
