use std::collections::{BTreeSet, HashSet};

use crate::ast::*;
use crate::diagnostics::{Diagnostic, SourcePos};

pub fn validate(program: &Program) -> Result<(), Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    reject_functions_shadowing_builtins(program, &mut diagnostics);
    reject_duplicate_functions(program, &mut diagnostics);
    reject_call_cycles(program, &mut diagnostics);
    reject_missing_returns(program, &mut diagnostics);
    reject_global_shadows_param(program, &mut diagnostics);
    reject_unsafe_error_handler_procedures(program, &mut diagnostics);

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

/// `global x` inside a function that also has a parameter named `x` is
/// silently inert: the transpiler always resolves a name to the parameter's
/// storage first, so the `global` declaration never takes effect. That's
/// never intentional, so reject it instead of leaving it a silent no-op.
fn reject_global_shadows_param(program: &Program, diagnostics: &mut Vec<Diagnostic>) {
    for function in &program.functions {
        let param_names: HashSet<String> = function
            .params
            .iter()
            .map(|p| p.name.as_basic().to_ascii_lowercase())
            .collect();

        let mut globals = Vec::new();
        collect_global_decls(&function.body, &mut globals);

        for global in globals {
            if param_names.contains(&global.as_basic().to_ascii_lowercase()) {
                diagnostics.push(Diagnostic::error(
                    generated_pos(),
                    format!(
                        "`global {}` in `{}` names a parameter of the same function -- \
                         the parameter always shadows it, so this declaration has no effect",
                        global, function.name
                    ),
                ));
            }
        }
    }
}

fn collect_global_decls(body: &[Statement], out: &mut Vec<BasicIdent>) {
    for stmt in body {
        match stmt {
            Statement::GlobalDecl(ident) => out.push(ident.clone()),
            Statement::If {
                then_body,
                else_body,
                ..
            } => {
                collect_global_decls(then_body, out);
                collect_global_decls(else_body, out);
            }
            Statement::For { body, .. }
            | Statement::While { body, .. }
            | Statement::Do { body, .. } => collect_global_decls(body, out),
            Statement::SelectCase {
                cases, else_body, ..
            } => {
                for case in cases {
                    collect_global_decls(&case.body, out);
                }
                collect_global_decls(else_body, out);
            }
            _ => {}
        }
    }
}

/// A `function`/`procedure` named the same (case-insensitively, ignoring
/// its type suffix -- `sqr%` and `sqr$` both collide with `SQR`, the same
/// way real BASIC's own suffix-independent intrinsic dispatch works) as one
/// of the real BASIC builtins that pass straight through with no `require`
/// (see `codegen_basic::BASIC_BUILTINS`) would either silently shadow the
/// builtin everywhere it's called, or the reverse -- neither is ever what
/// the program meant, since a genuinely intended override already has the
/// right tool (`com.bascal.stdlib`'s own require-based library functions,
/// see `standard-library.html`), so this is rejected outright rather than
/// left to whichever resolution order the codegen happens to pick.
fn reject_functions_shadowing_builtins(program: &Program, diagnostics: &mut Vec<Diagnostic>) {
    let builtins: HashSet<&str> = crate::codegen_basic::BASIC_BUILTINS.iter().copied().collect();
    for function in &program.functions {
        let base_name = function.name.name.to_ascii_lowercase();
        if builtins.contains(base_name.as_str()) {
            let kind = if function.is_procedure {
                "procedure"
            } else {
                "function"
            };
            diagnostics.push(Diagnostic::error(
                generated_pos(),
                format!(
                    "{kind} `{}` has the same name as the built-in `{}` -- a program can't \
                     override a BASIC builtin, name it something else",
                    function.name,
                    base_name.to_ascii_uppercase()
                ),
            ));
        }
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

/// Rejects any recursive call cycle -- direct (a function calling itself)
/// or indirect (`f%` calls `g%` calls `f%`, or a longer chain). Functions
/// and procedures transpile to `GOSUB` against shared global parameter
/// storage, not a real call stack, so *any* cycle in the call graph means
/// a second entry overwrites the first call's still-in-flight parameters
/// -- there's no depth at which that becomes safe.
fn reject_call_cycles(program: &Program, diagnostics: &mut Vec<Diagnostic>) {
    let n = program.functions.len();
    // adjacency[i] = indices of functions/procedures directly called from
    // function i's body.
    let adjacency: Vec<Vec<usize>> = program
        .functions
        .iter()
        .map(|caller| {
            program
                .functions
                .iter()
                .enumerate()
                .filter(|(_, callee)| statements_call_function(&caller.body, &callee.name))
                .map(|(j, _)| j)
                .collect()
        })
        .collect();

    #[derive(Clone, Copy, PartialEq)]
    enum Color {
        White,
        Gray,
        Black,
    }

    fn visit(
        node: usize,
        adjacency: &[Vec<usize>],
        color: &mut [Color],
        path: &mut Vec<usize>,
        program: &Program,
        reported: &mut HashSet<BTreeSet<usize>>,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        color[node] = Color::Gray;
        path.push(node);
        for &next in &adjacency[node] {
            match color[next] {
                Color::White => visit(next, adjacency, color, path, program, reported, diagnostics),
                Color::Gray => {
                    let start = path.iter().position(|&x| x == next).unwrap();
                    let cycle_indices: BTreeSet<usize> = path[start..].iter().copied().collect();
                    if reported.insert(cycle_indices) {
                        let names: Vec<String> = path[start..]
                            .iter()
                            .map(|&idx| program.functions[idx].name.as_basic())
                            .collect();
                        let chain = if names.len() == 1 {
                            format!("`{}` calls itself", names[0])
                        } else {
                            let mut labeled: Vec<String> =
                                names.iter().map(|n| format!("`{n}`")).collect();
                            labeled.push(format!("`{}`", names[0]));
                            labeled.join(" -> ")
                        };
                        diagnostics.push(Diagnostic::error(
                            generated_pos(),
                            format!(
                                "recursion is not supported: {chain} -- functions and \
                                 procedures transpile to shared global parameter storage, not \
                                 a real call stack, so a recursive call overwrites its own \
                                 in-flight parameters"
                            ),
                        ));
                    }
                }
                Color::Black => {}
            }
        }
        path.pop();
        color[node] = Color::Black;
    }

    let mut color = vec![Color::White; n];
    let mut path = Vec::new();
    let mut reported = HashSet::new();
    for i in 0..n {
        if color[i] == Color::White {
            visit(
                i,
                &adjacency,
                &mut color,
                &mut path,
                program,
                &mut reported,
                diagnostics,
            );
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

/// A procedure named as an `on error goto` target is entered via a raw
/// `GOTO`, never a `GOSUB` -- so there is no call frame for a `RETURN` to
/// pop. That makes two things unsafe for such a procedure, both otherwise
/// perfectly normal:
///
/// - Any `return`/bare `return` (== `RETURN`) inside its body, reached or
///   not: real BASIC has no way to distinguish "this RETURN was reached via
///   GOSUB" from "it wasn't" until the crash.
/// - Falling off the end of the body: codegen appends an implicit `RETURN`
///   there for every procedure whose body doesn't already end in `return`
///   (see `ends_with_return` in codegen.rs) -- fine for an ordinarily
///   GOSUB-called procedure, fatal here if that fallthrough is ever
///   actually reached.
///
/// So a procedure used this way must (a) contain no `return` anywhere, and
/// (b) provably never fall through -- every path must end in `resume`,
/// `resume next`, `resume <label>`, `goto`, or `end`. Given both hold, it
/// also can't be usable as an ordinary procedure anywhere else in the
/// program: something proven to never return can never come back to a
/// normal caller either.
fn reject_unsafe_error_handler_procedures(program: &Program, diagnostics: &mut Vec<Diagnostic>) {
    let handler_targets = error_handler_targets(program);
    if handler_targets.is_empty() {
        return;
    }

    for function in &program.functions {
        if !function.is_procedure {
            continue;
        }
        if !handler_targets
            .iter()
            .any(|t| same_ident(t, &function.name))
        {
            continue;
        }

        if contains_return(&function.body) {
            diagnostics.push(Diagnostic::error(
                generated_pos(),
                format!(
                    "`{}` cannot contain `return` -- it's the target of `on error goto`, \
                     which jumps to it with a raw GOTO, never a GOSUB, so a RETURN there has \
                     no call frame to pop and crashes at runtime; end every path with \
                     `resume`, `resume next`, or `resume <label>` instead",
                    function.name
                ),
            ));
        }

        if !diverges(&function.body) {
            diagnostics.push(Diagnostic::error(
                generated_pos(),
                format!(
                    "`{}` doesn't end every path with `resume`/`resume next`/`resume <label>` \
                     (or `goto`/`end`) -- it's the target of `on error goto`, so falling off \
                     the end would run into bcc's implicit RETURN with no GOSUB frame to pop, \
                     crashing at runtime",
                    function.name
                ),
            ));
        }

        if statements_call_function(&program.statements, &function.name)
            || program
                .functions
                .iter()
                .any(|f| statements_call_function(&f.body, &function.name))
        {
            diagnostics.push(Diagnostic::error(
                generated_pos(),
                format!(
                    "`{}` is both an `on error goto` target and called like an ordinary \
                     procedure -- a procedure that safely handles the first can never return \
                     to a normal caller, so it can't do both; give the error handler its own \
                     procedure",
                    function.name
                ),
            ));
        }
    }
}

/// Every identifier named as an `on error goto` target anywhere in the
/// program (main body or any function/procedure body) -- `on error goto 0`
/// (the disable sentinel) is a numeric literal, not an identifier, so it's
/// naturally excluded. Exposed for codegen: a procedure in this set has
/// been proven by `validate` to never fall through, so codegen must not
/// append its usual implicit trailing RETURN for one.
pub fn error_handler_targets(program: &Program) -> Vec<BasicIdent> {
    let mut targets = Vec::new();
    collect_on_error_goto_targets(&program.statements, &mut targets);
    for function in &program.functions {
        collect_on_error_goto_targets(&function.body, &mut targets);
    }
    targets
}

fn collect_on_error_goto_targets(statements: &[Statement], out: &mut Vec<BasicIdent>) {
    for stmt in statements {
        match stmt {
            Statement::OnErrorGoto {
                target: Expr::Ident(ident),
            } => out.push(ident.clone()),
            Statement::If {
                then_body,
                else_body,
                ..
            } => {
                collect_on_error_goto_targets(then_body, out);
                collect_on_error_goto_targets(else_body, out);
            }
            Statement::For { body, .. }
            | Statement::While { body, .. }
            | Statement::Do { body, .. } => collect_on_error_goto_targets(body, out),
            Statement::SelectCase {
                cases, else_body, ..
            } => {
                for case in cases {
                    collect_on_error_goto_targets(&case.body, out);
                }
                collect_on_error_goto_targets(else_body, out);
            }
            _ => {}
        }
    }
}

/// True if every path through `statements` is provably non-fallthrough --
/// ends in `resume`/`goto`/`end`, or (recursively) an `if` whose `then` and
/// `else` both diverge, or a `select case` whose every `case` and a
/// mandatory `case else` all diverge. Loops are never treated as diverging
/// (they may not run, or may complete normally), so a handler ending in a
/// loop still needs an explicit `resume`/`goto`/`end` after it.
fn diverges(statements: &[Statement]) -> bool {
    let last = statements
        .iter()
        .rev()
        .find(|s| !matches!(s, Statement::BlankLine));
    match last {
        Some(Statement::Resume(_)) | Some(Statement::Goto(_)) | Some(Statement::End) => true,
        Some(Statement::If {
            then_body,
            else_body,
            ..
        }) => diverges(then_body) && diverges(else_body),
        Some(Statement::SelectCase {
            cases, else_body, ..
        }) => cases.iter().all(|c| diverges(&c.body)) && diverges(else_body),
        _ => false,
    }
}

fn statements_call_function(statements: &[Statement], target: &BasicIdent) -> bool {
    statements
        .iter()
        .any(|statement| statement_calls_function(statement, target))
}

fn statement_calls_function(statement: &Statement, target: &BasicIdent) -> bool {
    match statement {
        Statement::Dim { sizes, .. } => sizes.iter().any(|e| expr_calls_function(e, target)),
        Statement::Assignment { target: lhs, value } => {
            expr_calls_function(lhs, target) || expr_calls_function(value, target)
        }
        Statement::MidAssign {
            target: lhs,
            start,
            len,
            value,
        } => {
            expr_calls_function(lhs, target)
                || expr_calls_function(start, target)
                || len.as_ref().is_some_and(|e| expr_calls_function(e, target))
                || expr_calls_function(value, target)
        }
        Statement::Print { tokens } => tokens.iter().any(|t| match t {
            PrintToken::Expr(e) => expr_calls_function(e, target),
            _ => false,
        }),
        Statement::Open { file, channel, .. } => {
            expr_calls_function(file, target) || expr_calls_function(channel, target)
        }
        Statement::FileDecl { path, .. } => expr_calls_function(path, target),
        Statement::LineInput {
            channel,
            target: line_target,
        } => expr_calls_function(channel, target) || expr_calls_function(line_target, target),
        Statement::PrintFile { channel, tokens } => {
            expr_calls_function(channel, target)
                || tokens.iter().any(|t| match t {
                    PrintToken::Expr(e) => expr_calls_function(e, target),
                    _ => false,
                })
        }
        Statement::PrintUsing { format, tokens } | Statement::LprintUsing { format, tokens } => {
            expr_calls_function(format, target)
                || tokens.iter().any(|t| match t {
                    PrintToken::Expr(e) => expr_calls_function(e, target),
                    _ => false,
                })
        }
        Statement::PrintFileUsing {
            channel,
            format,
            tokens,
        } => {
            expr_calls_function(channel, target)
                || expr_calls_function(format, target)
                || tokens.iter().any(|t| match t {
                    PrintToken::Expr(e) => expr_calls_function(e, target),
                    _ => false,
                })
        }
        Statement::Close { channel } => expr_calls_function(channel, target),
        Statement::Kill { file } => expr_calls_function(file, target),
        Statement::Name { from, to } => {
            expr_calls_function(from, target) || expr_calls_function(to, target)
        }
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
        Statement::Do {
            condition,
            body,
            post_condition,
        } => {
            condition
                .as_ref()
                .is_some_and(|c| expr_calls_function(&c.expr, target))
                || statements_call_function(body, target)
                || post_condition
                    .as_ref()
                    .is_some_and(|c| expr_calls_function(&c.expr, target))
        }
        Statement::Randomize(expr) => expr
            .as_ref()
            .is_some_and(|e| expr_calls_function(e, target)),
        Statement::Swap(a, b) => expr_calls_function(a, target) || expr_calls_function(b, target),
        Statement::Poke { address, value } => {
            expr_calls_function(address, target) || expr_calls_function(value, target)
        }
        Statement::Goto(e) | Statement::Gosub(e) | Statement::Restore(Some(e)) => {
            expr_calls_function(e, target)
        }
        Statement::OnErrorGoto { target: t } | Statement::ErrorStmt { code: t } => {
            expr_calls_function(t, target)
        }
        Statement::Resume(kind) => match kind {
            ResumeTarget::Line(e) => expr_calls_function(e, target),
            _ => false,
        },
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
        Statement::Lprint(tokens) => tokens.iter().any(|t| match t {
            PrintToken::Expr(e) => expr_calls_function(e, target),
            _ => false,
        }),
        Statement::SelectCase {
            expr,
            cases,
            else_body,
        } => {
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
        Statement::Field {
            channel, fields, ..
        } => {
            expr_calls_function(channel, target)
                || fields.iter().any(|(w, _)| expr_calls_function(w, target))
        }
        Statement::Get {
            channel,
            record,
            var,
            ..
        }
        | Statement::Put {
            channel,
            record,
            var,
            ..
        } => {
            expr_calls_function(channel, target)
                || record
                    .as_ref()
                    .is_some_and(|e| expr_calls_function(e, target))
                || var.as_ref().is_some_and(|e| expr_calls_function(e, target))
        }
        Statement::Lset { value, .. } | Statement::Rset { value, .. } => {
            expr_calls_function(value, target)
        }
        Statement::Seek { channel, position } => {
            expr_calls_function(channel, target) || expr_calls_function(position, target)
        }
        Statement::OptionBase(e) => expr_calls_function(e, target),
        Statement::Erase(_) => false,
        Statement::Out { port, value } => {
            expr_calls_function(port, target) || expr_calls_function(value, target)
        }
        Statement::Width { channel, cols } => {
            channel
                .as_ref()
                .is_some_and(|c| expr_calls_function(c, target))
                || expr_calls_function(cols, target)
        }
        Statement::End
        | Statement::Stop
        | Statement::Cls
        | Statement::Beep
        | Statement::Clear
        | Statement::System
        | Statement::Exit
        | Statement::Restore(None)
        | Statement::ReturnVoid
        | Statement::GlobalDecl(_)
        | Statement::Raw(_)
        | Statement::BlockComment(_)
        | Statement::Label(_)
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
        Expr::Integer(_) | Expr::Float(_) | Expr::HexLit(_) | Expr::String(_) | Expr::Ident(_) => {
            false
        }
        Expr::FileIndex { index, .. } => expr_calls_function(index, target),
        Expr::FieldAccess { base, .. } => expr_calls_function(base, target),
        Expr::MethodCall { base, args, .. } => {
            expr_calls_function(base, target) || args.iter().any(|a| expr_calls_function(a, target))
        }
        Expr::RecordLit { fields, .. } => {
            fields.iter().any(|(_, e)| expr_calls_function(e, target))
        }
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
        Statement::SelectCase {
            cases, else_body, ..
        } => cases.iter().any(|c| contains_return(&c.body)) || contains_return(else_body),
        _ => false,
    })
}

fn same_ident(left: &BasicIdent, right: &BasicIdent) -> bool {
    left.suffix == right.suffix && left.name.eq_ignore_ascii_case(&right.name)
}

fn generated_pos() -> SourcePos {
    SourcePos::new("<validation>", 1, 1)
}

// ── --strict-vars / --strict-vars-warn ──────────────────────────────────
//
// Pascal-style mandatory declaration, opt-in only (see `CompileOptions::
// strict_vars`/`strict_vars_warn`). Run on the *root program's own*,
// pre-`records::lower` AST -- the caller (`compile_file`) is responsible
// for passing in only the root file's own statements/functions, never a
// `require`d library's (BASCAL's own `com.bascal.stdlib` isn't written
// this way, and shouldn't have to be just because the importing program
// turned this on) and never the DSL-lowered form (which invents its own
// buffer/scalar variables that were never meant to be `dim`'d by hand).
//
// A name is valid without a matching `dim`/`declare` if it's a `const`, a
// `for` loop's own counter, a function/procedure parameter, a raw `FIELD`
// buffer variable, a known callable (a program function or a real BASIC
// builtin -- `Expr::Call`/`Expr::ArrayRef` used in call position, or a
// bare zero-arg builtin like `ERR`/`DATE$`), or the target of a `let`/
// `file`/`record` DSL binding (those are their own declaration forms,
// validated separately by `records::lower`, and out of scope here since
// this check runs *before* lowering).

type VarKey = (String, Option<TypeSuffix>);

fn var_key(ident: &BasicIdent) -> VarKey {
    (ident.name.to_ascii_lowercase(), ident.suffix)
}

/// Checks `program` (see this section's own doc comment for exactly which
/// AST this must be) and returns every undeclared-variable finding, as
/// errors if `warn_only` is false or warnings if `warn_only` is true --
/// the caller decides whether that means failing the compile or just
/// printing them (see `compile_file`).
pub fn check_strict_vars(program: &Program, warn_only: bool) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // A `require`d function is never actually in `program.functions` here --
    // this runs on the root file's own parse, before `require`/`import` get
    // resolved and merged in (see this section's own doc comment) -- so its
    // *name* is approximated from its dotted path's last segment instead
    // (`com.bascal.stdlib.ucase` -> `ucase`), matching the filename
    // convention `required_symbol_to_path` itself relies on. A library that
    // broke that convention (defining a differently-named function than its
    // own path's last segment) would be misjudged here, but every real
    // library in this repo -- `com.bascal.stdlib.*` included -- follows it.
    let known_callables: HashSet<String> = program
        .functions
        .iter()
        .map(|f| f.name.name.to_ascii_lowercase())
        .chain(
            crate::codegen_basic::BASIC_BUILTINS
                .iter()
                .map(|s| s.to_string()),
        )
        .chain(program.declarations.iter().map(|decl| {
            let symbol = match decl {
                DependencyDecl::Require(s) | DependencyDecl::Import(s) => s,
            };
            symbol
                .raw
                .rsplit('.')
                .next()
                .unwrap_or(&symbol.raw)
                .to_ascii_lowercase()
        }))
        .collect();

    let mut top_level_declared = HashSet::new();
    collect_declarations(&program.statements, &mut top_level_declared);
    check_var_uses(
        &program.statements,
        &top_level_declared,
        &known_callables,
        warn_only,
        &mut diagnostics,
    );

    for function in &program.functions {
        let mut declared = HashSet::new();
        for param in &function.params {
            declared.insert(var_key(&param.name));
        }
        collect_declarations(&function.body, &mut declared);
        let mut globals = Vec::new();
        collect_global_decls(&function.body, &mut globals);
        for global in &globals {
            declared.insert(var_key(global));
        }
        check_var_uses(
            &function.body,
            &declared,
            &known_callables,
            warn_only,
            &mut diagnostics,
        );
    }

    diagnostics
}

/// Collects every name `dim`/`declare`, `const`, a `for` loop's own
/// counter, or a raw `FIELD` statement introduces, recursing into every
/// nested statement body -- but *not* into `program.functions`, which the
/// caller (`check_strict_vars`) walks separately with its own declared-name
/// set, matching BASCAL's real function-local-by-default scoping (see
/// tutorial 14: a name inside a function/procedure is local unless
/// promoted with `global`).
fn collect_declarations(statements: &[Statement], declared: &mut HashSet<VarKey>) {
    for stmt in statements {
        match stmt {
            Statement::Dim { name, .. } | Statement::Const { name, .. } => {
                declared.insert(var_key(name));
            }
            Statement::For { var, body, .. } => {
                declared.insert(var_key(var));
                collect_declarations(body, declared);
            }
            Statement::Field { fields, .. } => {
                for (_, name) in fields {
                    declared.insert(var_key(name));
                }
            }
            Statement::If {
                then_body,
                else_body,
                ..
            } => {
                collect_declarations(then_body, declared);
                collect_declarations(else_body, declared);
            }
            Statement::While { body, .. } | Statement::Do { body, .. } => {
                collect_declarations(body, declared);
            }
            Statement::SelectCase {
                cases, else_body, ..
            } => {
                for case in cases {
                    collect_declarations(&case.body, declared);
                }
                collect_declarations(else_body, declared);
            }
            _ => {}
        }
    }
}

fn check_var_uses(
    statements: &[Statement],
    declared: &HashSet<VarKey>,
    known_callables: &HashSet<String>,
    warn_only: bool,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut check = |expr: &Expr| {
        let ident = match expr {
            Expr::Ident(ident) => ident,
            Expr::ArrayRef { name, .. } => name,
            _ => return,
        };
        if known_callables.contains(&ident.name.to_ascii_lowercase()) {
            return;
        }
        if declared.contains(&var_key(ident)) {
            return;
        }
        let make = if warn_only {
            Diagnostic::warning
        } else {
            Diagnostic::error
        };
        diagnostics.push(make(
            generated_pos(),
            format!(
                "`{ident}` is used without a `dim`/`declare` -- --strict-vars requires every \
                 variable to be declared before use (a misspelled name is the usual cause)"
            ),
        ));
    };
    walk_statements_exprs(statements, &mut check);
}

fn walk_statements_exprs(statements: &[Statement], f: &mut dyn FnMut(&Expr)) {
    for stmt in statements {
        walk_statement_exprs(stmt, f);
    }
}

fn walk_statement_exprs(statement: &Statement, f: &mut dyn FnMut(&Expr)) {
    match statement {
        Statement::Dim { sizes, .. } => sizes.iter().for_each(|e| walk_expr(e, f)),
        Statement::Assignment { target, value } => {
            walk_expr(target, f);
            walk_expr(value, f);
        }
        Statement::MidAssign {
            target,
            start,
            len,
            value,
        } => {
            walk_expr(target, f);
            walk_expr(start, f);
            if let Some(len) = len {
                walk_expr(len, f);
            }
            walk_expr(value, f);
        }
        Statement::Print { tokens }
        | Statement::Lprint(tokens)
        | Statement::PrintFile { tokens, .. } => {
            for t in tokens {
                if let PrintToken::Expr(e) = t {
                    walk_expr(e, f);
                }
            }
        }
        Statement::PrintUsing { format, tokens } | Statement::LprintUsing { format, tokens } => {
            walk_expr(format, f);
            for t in tokens {
                if let PrintToken::Expr(e) = t {
                    walk_expr(e, f);
                }
            }
        }
        Statement::PrintFileUsing {
            channel,
            format,
            tokens,
        } => {
            walk_expr(channel, f);
            walk_expr(format, f);
            for t in tokens {
                if let PrintToken::Expr(e) = t {
                    walk_expr(e, f);
                }
            }
        }
        Statement::Open { file, channel, .. } => {
            walk_expr(file, f);
            walk_expr(channel, f);
        }
        Statement::FileDecl { path, .. } => walk_expr(path, f),
        Statement::LineInput { channel, target } => {
            walk_expr(channel, f);
            walk_expr(target, f);
        }
        Statement::Close { channel } | Statement::Restore(Some(channel)) => walk_expr(channel, f),
        Statement::Kill { file } => walk_expr(file, f),
        Statement::Name { from, to } => {
            walk_expr(from, f);
            walk_expr(to, f);
        }
        Statement::Return { value } => walk_expr(value, f),
        Statement::If {
            condition,
            then_body,
            else_body,
        } => {
            walk_expr(condition, f);
            walk_statements_exprs(then_body, f);
            walk_statements_exprs(else_body, f);
        }
        Statement::For {
            start,
            end,
            step,
            body,
            ..
        } => {
            walk_expr(start, f);
            walk_expr(end, f);
            if let Some(step) = step {
                walk_expr(step, f);
            }
            walk_statements_exprs(body, f);
        }
        Statement::While { condition, body } => {
            walk_expr(condition, f);
            walk_statements_exprs(body, f);
        }
        Statement::ExprStmt(expr) => walk_expr(expr, f),
        Statement::Do {
            condition,
            body,
            post_condition,
        } => {
            if let Some(c) = condition {
                walk_expr(&c.expr, f);
            }
            walk_statements_exprs(body, f);
            if let Some(c) = post_condition {
                walk_expr(&c.expr, f);
            }
        }
        Statement::Randomize(Some(e)) => walk_expr(e, f),
        Statement::Swap(a, b) => {
            walk_expr(a, f);
            walk_expr(b, f);
        }
        Statement::Poke { address, value } | Statement::Out { port: address, value } => {
            walk_expr(address, f);
            walk_expr(value, f);
        }
        Statement::Goto(e) | Statement::Gosub(e) => walk_expr(e, f),
        Statement::OnErrorGoto { target } => walk_expr(target, f),
        Statement::ErrorStmt { code } => walk_expr(code, f),
        Statement::Resume(ResumeTarget::Line(e)) => walk_expr(e, f),
        Statement::Input { vars, .. } | Statement::Read(vars) => {
            vars.iter().for_each(|e| walk_expr(e, f))
        }
        Statement::InputFile { channel, vars } => {
            walk_expr(channel, f);
            vars.iter().for_each(|e| walk_expr(e, f));
        }
        Statement::Data(values) => values.iter().for_each(|e| walk_expr(e, f)),
        Statement::Const { value, .. } => walk_expr(value, f),
        Statement::Write { channel, exprs } => {
            walk_expr(channel, f);
            exprs.iter().for_each(|e| walk_expr(e, f));
        }
        Statement::SelectCase {
            expr,
            cases,
            else_body,
        } => {
            walk_expr(expr, f);
            for case in cases {
                for value in &case.values {
                    match value {
                        CaseValue::Single(e) | CaseValue::Is { value: e, .. } => walk_expr(e, f),
                        CaseValue::Range { from, to } => {
                            walk_expr(from, f);
                            walk_expr(to, f);
                        }
                    }
                }
                walk_statements_exprs(&case.body, f);
            }
            walk_statements_exprs(else_body, f);
        }
        Statement::Locate { row, col } => {
            walk_expr(row, f);
            walk_expr(col, f);
        }
        Statement::Color { fg, bg } => {
            walk_expr(fg, f);
            if let Some(bg) = bg {
                walk_expr(bg, f);
            }
        }
        Statement::OnBranch { expr, targets, .. } => {
            walk_expr(expr, f);
            targets.iter().for_each(|e| walk_expr(e, f));
        }
        Statement::Field { channel, fields, .. } => {
            walk_expr(channel, f);
            fields.iter().for_each(|(w, _)| walk_expr(w, f));
        }
        Statement::Get {
            channel,
            record,
            var,
            ..
        }
        | Statement::Put {
            channel,
            record,
            var,
            ..
        } => {
            walk_expr(channel, f);
            if let Some(record) = record {
                walk_expr(record, f);
            }
            if let Some(var) = var {
                walk_expr(var, f);
            }
        }
        Statement::Lset { value, .. } | Statement::Rset { value, .. } => walk_expr(value, f),
        Statement::Seek { channel, position } => {
            walk_expr(channel, f);
            walk_expr(position, f);
        }
        Statement::OptionBase(e) => walk_expr(e, f),
        Statement::Width { channel, cols } => {
            if let Some(channel) = channel {
                walk_expr(channel, f);
            }
            walk_expr(cols, f);
        }
        Statement::Erase(_)
        | Statement::End
        | Statement::Stop
        | Statement::Cls
        | Statement::Beep
        | Statement::Clear
        | Statement::System
        | Statement::Exit
        | Statement::Restore(None)
        | Statement::ReturnVoid
        | Statement::GlobalDecl(_)
        | Statement::Raw(_)
        | Statement::BlockComment(_)
        | Statement::Label(_)
        | Statement::Randomize(None)
        | Statement::Resume(_)
        | Statement::BlankLine => {}
    }
}

fn walk_expr(expr: &Expr, f: &mut dyn FnMut(&Expr)) {
    f(expr);
    match expr {
        Expr::Integer(_) | Expr::Float(_) | Expr::HexLit(_) | Expr::String(_) | Expr::Ident(_) => {
        }
        Expr::ArrayRef { indices, .. } => indices.iter().for_each(|e| walk_expr(e, f)),
        Expr::Call { args, .. } => args.iter().for_each(|e| walk_expr(e, f)),
        Expr::Unary { expr, .. } => walk_expr(expr, f),
        Expr::Binary { left, right, .. } => {
            walk_expr(left, f);
            walk_expr(right, f);
        }
        Expr::FileIndex { index, .. } => walk_expr(index, f),
        Expr::FieldAccess { base, .. } => walk_expr(base, f),
        Expr::MethodCall { base, args, .. } => {
            walk_expr(base, f);
            args.iter().for_each(|e| walk_expr(e, f));
        }
        Expr::RecordLit { fields, .. } => fields.iter().for_each(|(_, e)| walk_expr(e, f)),
    }
}

// TODO: Add source-location carrying AST nodes so validation diagnostics can
// point at the exact function declaration or call expression.
