//! Grammar DSL parsing and validation will produce [`rdgen_ir::Grammar`].

pub use rdgen_ir;

/// Entry point reserved for the grammar DSL parser.
pub fn compile(_source: &str) -> Result<rdgen_ir::Grammar, String> {
    Err("rdgen grammar parser is not implemented yet".into())
}
