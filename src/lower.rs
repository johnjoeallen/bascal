//! One explicit post-parse lowering phase.
//!
//! Everything that transforms the AST between parsing and
//! `resolver::validate` lives here as an ordered list of sub-passes, so the
//! pipeline shape is readable in one place and no backend has to re-run (or
//! guess the order of) these transforms:
//!
//!   1. [`records::lower`] — desugar the record / file DSL into `FIELD` +
//!      `GET` / `PUT` + `MKx$` / `CVx$` and synthesized buffer variables.
//!
//! The one post-parse mutation that is *not* here is `Program.common`
//! population (in `compile_file`): it needs filesystem + `CompileOptions`
//! context to locate the `shared` file, so it stays in the IO-aware driver
//! and runs before `lower`.
//!
//! [`Lowered`] also gives `synthesized_buffer_names` a named home instead of
//! `records::lower`'s bare tuple return.

use crate::diagnostics::Diagnostic;
use crate::{ast, records};
use std::collections::HashSet;

/// The parsed program after every post-parse AST → AST pass has run, plus the
/// facts those passes established that a backend would otherwise have to
/// re-derive.
pub(crate) struct Lowered {
    pub program: ast::Program,
    /// Buffer variables synthesized by [`records::lower`] from the file DSL.
    /// Each must keep exactly one program-wide BASIC name (the `FIELD`
    /// binding is global), so the backend's per-procedure name allocator
    /// must never localize one.
    pub synthesized_buffer_names: HashSet<String>,
    /// Typed record/file layouts synthesized together with their allocated
    /// channels and FIELD buffer bindings.
    pub lowered_record_files: Vec<crate::semantic_ir::LoweredRecordFile>,
}

/// Run every post-parse lowering sub-pass, in order.
pub(crate) fn lower(program: ast::Program) -> Result<Lowered, Vec<Diagnostic>> {
    let (program, synthesized_buffer_names, lowered_record_files) = records::lower(program)?;
    Ok(Lowered {
        program,
        synthesized_buffer_names,
        lowered_record_files,
    })
}
