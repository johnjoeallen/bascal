//! Rust backend for emitting hand-written-style recursive-descent parsers.

use rdgen_ir::Grammar;

/// Emit a Rust parser from the resolved grammar IR.
pub fn emit(_grammar: &Grammar) -> Result<String, String> {
    Err("Rust code emitter is not implemented yet".into())
}
