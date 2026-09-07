//! One explicit post-parse lowering phase.
//!
//! Everything that transforms the AST between parsing and
//! `resolver::validate` lives here as an ordered list of sub-passes, so the
//! pipeline shape is readable in one place and no backend has to re-run (or
//! guess the order of) these transforms:
//!
//!   1. [`records::lower`] — desugar the record / file DSL into `FIELD` +
//!      `GET` / `PUT` + `MKx$` / `CVx$` and synthesized buffer variables.
//!   2. [`inject_mid_assign_helper_if_used`] — splice in the
//!      `com.bascal.stdlib.midAssign` helper when the `MID$(...) = ...`
//!      statement form appears anywhere.
//!
//! The one post-parse mutation that is *not* here is `Program.common`
//! population (in `compile_file`): it needs filesystem + `CompileOptions`
//! context to locate the `shared` file, so it stays in the IO-aware driver
//! and runs before `lower`.
//!
//! [`Lowered`] also gives `synthesized_buffer_names` a named home instead of
//! `records::lower`'s bare tuple return.

use std::collections::HashSet;
use std::fs;

use crate::diagnostics::{self, Diagnostic};
use crate::{ast, codegen, parse_source, records, required_symbol_to_path, stdlib_search_roots};

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
}

/// Run every post-parse lowering sub-pass, in order.
pub(crate) fn lower(program: ast::Program) -> Result<Lowered, Vec<Diagnostic>> {
    let (mut program, synthesized_buffer_names) = records::lower(program)?;
    inject_mid_assign_helper_if_used(&mut program)?;
    Ok(Lowered {
        program,
        synthesized_buffer_names,
    })
}

/// If `program` uses `MID$` statement-form assignment anywhere (top-level
/// or inside any function body) and hasn't already defined or required its
/// own `midAssign$`, splices in `com.bascal.stdlib.midAssign` -- resolved
/// via `stdlib_search_roots()`, the same on-disk library `require
/// com.bascal.stdlib.*` resolves against, just triggered by the AST shape
/// instead of an explicit `require` line, since nothing in the user's own
/// source ever names this function -- the transpiler synthesizes the call
/// (see `codegen::MID_ASSIGN_HELPER_NAME`).
fn inject_mid_assign_helper_if_used(program: &mut ast::Program) -> Result<(), Vec<Diagnostic>> {
    let already_defined = program.functions.iter().any(|f| {
        f.name
            .name
            .eq_ignore_ascii_case(codegen::MID_ASSIGN_HELPER_NAME)
    });
    if already_defined || !program_uses_mid_assign(program) {
        return Ok(());
    }

    let symbol = format!("com.bascal.stdlib.{}", codegen::MID_ASSIGN_HELPER_NAME);
    let relative = required_symbol_to_path(&symbol);
    let path = stdlib_search_roots()
        .into_iter()
        .map(|root| root.join(&relative))
        .find(|candidate| candidate.exists())
        .ok_or_else(|| {
            vec![Diagnostic::error(
                diagnostics::SourcePos::new("<transpiler-internal>", 1, 1),
                format!(
                    "internal error: this program uses MID$ statement-form assignment, which \
                     needs BASCAL's own {symbol} helper, but {} could not be found -- this \
                     looks like a broken install; check that `com/` shipped alongside `bcc`",
                    relative.display()
                ),
            )]
        })?;

    let source = fs::read_to_string(&path).map_err(|err| {
        vec![Diagnostic::error(
            diagnostics::SourcePos::new("<transpiler-internal>", 1, 1),
            format!("internal error: failed to read {}: {err}", path.display()),
        )]
    })?;
    let filename = path.display().to_string();
    let helper_program = parse_source(filename.clone(), &source)?;
    let [function]: [ast::FunctionDef; 1] =
        helper_program
            .functions
            .try_into()
            .unwrap_or_else(|functions: Vec<ast::FunctionDef>| {
                panic!(
                    "BASCAL bug: {filename} must declare exactly one function, found {}",
                    functions.len()
                )
            });
    program.functions.push(function);
    Ok(())
}

fn program_uses_mid_assign(program: &ast::Program) -> bool {
    statements_use_mid_assign(&program.statements)
        || program
            .functions
            .iter()
            .any(|f| statements_use_mid_assign(&f.body))
}

fn statements_use_mid_assign(statements: &[ast::Stmt]) -> bool {
    statements.iter().any(statement_uses_mid_assign)
}

fn statement_uses_mid_assign(statement: &ast::Stmt) -> bool {
    use ast::Statement::*;
    match &statement.kind {
        MidAssign { .. } => true,
        If {
            then_body,
            else_body,
            ..
        } => statements_use_mid_assign(then_body) || statements_use_mid_assign(else_body),
        For { body, .. } | While { body, .. } | Do { body, .. } => statements_use_mid_assign(body),
        SelectCase {
            cases, else_body, ..
        } => {
            cases.iter().any(|c| statements_use_mid_assign(&c.body))
                || statements_use_mid_assign(else_body)
        }
        _ => false,
    }
}
