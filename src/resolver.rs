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
        if function.is_procedure {
            continue;
        }
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
        Statement::Open { file, channel, .. } => {
            expr_calls_function(file, target) || expr_calls_function(channel, target)
        }
        Statement::LineInput {
            channel,
            target: line_target,
        } => expr_calls_function(channel, target) || expr_calls_function(line_target, target),
        Statement::PrintFile { channel, exprs } => {
            expr_calls_function(channel, target)
                || exprs.iter().any(|expr| expr_calls_function(expr, target))
        }
        Statement::Close { channel } => expr_calls_function(channel, target),
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
        Statement::Do { condition, body, post_condition } => {
            condition.as_ref().is_some_and(|c| expr_calls_function(&c.expr, target))
                || statements_call_function(body, target)
                || post_condition.as_ref().is_some_and(|c| expr_calls_function(&c.expr, target))
        }
        Statement::Randomize(expr) => {
            expr.as_ref().is_some_and(|e| expr_calls_function(e, target))
        }
        Statement::Swap(a, b) => {
            expr_calls_function(a, target) || expr_calls_function(b, target)
        }
        Statement::Goto(e) | Statement::Gosub(e) | Statement::Restore(Some(e)) => {
            expr_calls_function(e, target)
        }
        Statement::Input { vars, .. } | Statement::Read(vars) => {
            vars.iter().any(|e| expr_calls_function(e, target))
        }
        Statement::InputFile { channel, vars } => {
            expr_calls_function(channel, target)
                || vars.iter().any(|e| expr_calls_function(e, target))
        }
        Statement::Data(values) => values.iter().any(|e| expr_calls_function(e, target)),
        Statement::Const { value, .. } => expr_calls_function(value, target),
        Statement::Write { channel, exprs } => {
            expr_calls_function(channel, target)
                || exprs.iter().any(|e| expr_calls_function(e, target))
        }
        Statement::Lprint(exprs) => exprs.iter().any(|e| expr_calls_function(e, target)),
        Statement::SelectCase { expr, cases, else_body } => {
            expr_calls_function(expr, target)
                || cases.iter().any(|c| {
                    c.values.iter().any(|v| match v {
                        CaseValue::Single(e) | CaseValue::Is { value: e, .. } => {
                            expr_calls_function(e, target)
                        }
                        CaseValue::Range { from, to } => {
                            expr_calls_function(from, target) || expr_calls_function(to, target)
                        }
                    }) || statements_call_function(&c.body, target)
                })
                || statements_call_function(else_body, target)
        }
        Statement::Locate { row, col } => {
            expr_calls_function(row, target) || expr_calls_function(col, target)
        }
        Statement::Color { fg, bg } => {
            expr_calls_function(fg, target)
                || bg.as_ref().is_some_and(|e| expr_calls_function(e, target))
        }
        Statement::OnBranch { expr, targets, .. } => {
            expr_calls_function(expr, target)
                || targets.iter().any(|e| expr_calls_function(e, target))
        }
        Statement::End
        | Statement::Stop
        | Statement::Cls
        | Statement::Beep
        | Statement::System
        | Statement::ExitFor
        | Statement::ExitWhile
        | Statement::ExitDo
        | Statement::Restore(None)
        | Statement::ReturnVoid
        | Statement::Raw(_)
        | Statement::BlockComment(_)
        | Statement::BlankLine => false,
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
        Expr::Integer(_) | Expr::Float(_) | Expr::String(_) | Expr::Ident(_) => false,
    }
}

fn contains_return(statements: &[Statement]) -> bool {
    statements.iter().any(|statement| match statement {
        Statement::Return { .. } | Statement::ReturnVoid => true,
        Statement::If {
            then_body,
            else_body,
            ..
        } => contains_return(then_body) || contains_return(else_body),
        Statement::For { body, .. }
        | Statement::While { body, .. }
        | Statement::Do { body, .. } => contains_return(body),
        Statement::SelectCase { cases, else_body, .. } => {
            cases.iter().any(|c| contains_return(&c.body))
                || contains_return(else_body)
        }
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
