//! Minimal native-JVM backend (bootstrap).
//!
//! Stage 1 of the JVM target: dispatch plumbing only. This backend doesn't
//! generate anything yet -- every program reports a "not supported yet"
//! diagnostic. Its purpose is to prove `Target::Jvm`/`--target jvm` wire up
//! end to end (CLI parsing, output-path defaulting, `main.rs`'s eventual
//! `krak2`/`java` invocation) before any real codegen exists.
//!
//! Output format is Krakatau assembly text (`.j`), assembled by the external
//! `krak2` tool (https://github.com/Storyyeller/Krakatau, `v2` branch --
//! itself a Rust/Cargo project, pinned at a specific commit since it has no
//! versioned releases) into a real `.class`, then run by any JRE's `java`.

use crate::ast::Program;
use crate::diagnostics::{Diagnostic, SourcePos};

pub(crate) fn generate(_program: &Program) -> Result<String, Vec<Diagnostic>> {
    Err(vec![unsupported(
        "--target jvm does not generate any code yet -- this is a walking-skeleton bootstrap, \
         see codegen_jvm.rs's module doc comment",
    )])
}

fn unsupported(message: &str) -> Diagnostic {
    Diagnostic::error(SourcePos::new("<target>", 1, 1), message.to_string())
}
