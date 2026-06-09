use std::collections::HashSet;

use crate::ast::*;
use crate::diagnostics::{Diagnostic, SourcePos};

pub fn validate(program: &Program) -> Result<(), Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    reject_duplicate_functions(program, &mut diagnostics);
    reject_direct_recursion(program, &mut diagnostics);
    reject_missing_returns(program, &mut diagnostics);

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

fn reject_duplicate_functions(program: &Program, diagnostics: &mut Vec<Diagnostic>) {
    let mut seen = HashSet::new();
    for function in &program.functions {
        let name = function.name.as_basic().to_ascii_lowercase();
        if !seen.insert(name) {
            diagnostics.push(Diagnostic::error(
                generated_pos(),
                format!("duplicate function `{}`", function.name),
            ));
        }
    }
}

fn reject_direct_recursion(program: &Program, diagnostics: &mut Vec<Diagnostic>) {
    for function in &program.functions {
        if statements_call_function(&function.body, &function.name) {
            diagnostics.push(Diagnostic::error(
                generated_pos(),
                format!("direct recursion is not supported for `{}`", function.name),
            ));
        }
    }
}

fn reject_missing_returns(program: &Program, diagnostics: &mut Vec<Diagnostic>) {
    for function in &program.functions {
        if !contains_return(&function.body) {
            diagnostics.push(Diagnostic::error(
                generated_pos(),
                format!(
                    "implicit function return is not supported for `{}`; use `return`",
                    function.name
                ),
            ));
        }
    }
}

fn statements_call_function(statements: &[Statement], target: &BasicIdent) -> bool {
    statements
        .iter()
        .any(|statement| statement_calls_function(statement, target))
}

fn statement_calls_function(statement: &Statement, target: &BasicIdent) -> bool {
    match statement {
        Statement::Dim { size, .. } => size
            .as_ref()
            .is_some_and(|expr| expr_calls_function(expr, target)),
        Statement::Assignment { target: lhs, value } => {
            expr_calls_function(lhs, target) || expr_calls_function(value, target)
        }
        Statement::Print { exprs } => exprs.iter().any(|expr| expr_calls_function(expr, target)),
        Statement::Return { value } => expr_calls_function(value, target),
        Statement::If {
            condition,
            then_body,
            else_body,
        } => {
            expr_calls_function(condition, target)
                || statements_call_function(then_body, target)
                || statements_call_function(else_body, target)
        }
        Statement::For {
            start,
            end,
            step,
            body,
            ..
        } => {
            expr_calls_function(start, target)
                || expr_calls_function(end, target)
                || step
                    .as_ref()
                    .is_some_and(|expr| expr_calls_function(expr, target))
                || statements_call_function(body, target)
        }
        Statement::While { condition, body } => {
            expr_calls_function(condition, target) || statements_call_function(body, target)
        }
        Statement::ExprStmt(expr) => expr_calls_function(expr, target),
        Statement::End | Statement::Raw(_) | Statement::BlankLine => false,
    }
}

fn expr_calls_function(expr: &Expr, target: &BasicIdent) -> bool {
    match expr {
        Expr::Call { name, args } => {
            same_ident(name, target) || args.iter().any(|arg| expr_calls_function(arg, target))
        }
        Expr::ArrayRef { name, indices } => {
            same_ident(name, target) || indices.iter().any(|arg| expr_calls_function(arg, target))
        }
        Expr::Unary { expr, .. } => expr_calls_function(expr, target),
        Expr::Binary { left, right, .. } => {
            expr_calls_function(left, target) || expr_calls_function(right, target)
        }
        Expr::Integer(_) | Expr::String(_) | Expr::Ident(_) => false,
    }
}

fn contains_return(statements: &[Statement]) -> bool {
    statements.iter().any(|statement| match statement {
        Statement::Return { .. } => true,
        Statement::If {
            then_body,
            else_body,
            ..
        } => contains_return(then_body) || contains_return(else_body),
        Statement::For { body, .. } | Statement::While { body, .. } => contains_return(body),
        _ => false,
    })
}

fn same_ident(left: &BasicIdent, right: &BasicIdent) -> bool {
    left.suffix == right.suffix && left.name.eq_ignore_ascii_case(&right.name)
}

fn generated_pos() -> SourcePos {
    SourcePos::new("<validation>", 1, 1)
}

// TODO: Add source-location carrying AST nodes so validation diagnostics can
// point at the exact function declaration or call expression.
// TODO: Resolve include/require/import declarations into linker inputs.
// BASCAL paths are linker selectors, not namespaces; final BASIC symbols stay
// global and parameters will be lowered to function-prefixed globals.
