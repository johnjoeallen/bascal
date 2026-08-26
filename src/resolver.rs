use std::collections::{BTreeSet, HashSet};

use crate::ast::*;
use crate::diagnostics::{Diagnostic, SourcePos};

pub fn validate(program: &Program) -> Result<(), Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    reject_functions_shadowing_builtins(program, &mut diagnostics);
    reject_duplicate_functions(program, &mut diagnostics);
    reject_scalar_methods(program, &mut diagnostics);
    reject_call_cycles(program, &mut diagnostics);
    reject_missing_returns(program, &mut diagnostics);
    reject_invalid_parameter_defaults(program, &mut diagnostics);
    reject_global_shadows_param(program, &mut diagnostics);
    reject_unsafe_error_handler_procedures(program, &mut diagnostics);
    reject_option_base(program, &mut diagnostics);

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

fn reject_invalid_parameter_defaults(program: &Program, diagnostics: &mut Vec<Diagnostic>) {
    let fixed_consts: HashSet<String> = program
        .statements
        .iter()
        .filter_map(|stmt| match &stmt.kind {
            Statement::Const { name, value } if is_fixed_default_value(value) => {
                Some(name.as_basic().to_ascii_lowercase())
            }
            _ => None,
        })
        .collect();

    for function in &program.functions {
        let mut optional_seen = false;
        for param in &function.params {
            match &param.default {
                Some(default) => {
                    optional_seen = true;
                    let fixed = is_fixed_default_value(default)
                        || matches!(default, Expr::Ident(name) if fixed_consts.contains(&name.as_basic().to_ascii_lowercase()));
                    if !fixed {
                        diagnostics.push(Diagnostic::error(
                            function.pos.clone(),
                            format!(
                                "default value for parameter `{}` of `{}` must be a literal or a top-level `const`",
                                param.name, function.name
                            ),
                        ));
                    }
                }
                None if optional_seen => diagnostics.push(Diagnostic::error(
                    function.pos.clone(),
                    format!(
                        "parameter `{}` of `{}` is required but follows a parameter with a default value",
                        param.name, function.name
                    ),
                )),
                None => {}
            }
        }
    }
}

fn is_fixed_default_value(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Integer(_) | Expr::Float(_) | Expr::HexLit(_) | Expr::String(_)
    ) || matches!(
        expr,
        Expr::Unary {
            op: UnaryOp::Neg,
            expr,
        } if matches!(expr.as_ref(), Expr::Integer(_) | Expr::Float(_) | Expr::HexLit(_))
    )
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

        for (pos, global) in globals {
            if param_names.contains(&global.as_basic().to_ascii_lowercase()) {
                diagnostics.push(Diagnostic::error(
                    pos,
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

/// Collects every `global <name>` declaration in `body` (recursing into
/// every nested statement body), each paired with the source position of
/// its own `Statement::GlobalDecl` -- callers that only care about the
/// declared names (like `check_strict_vars`) can just ignore the position.
fn collect_global_decls(body: &[Stmt], out: &mut Vec<(SourcePos, BasicIdent)>) {
    for stmt in body {
        match &stmt.kind {
            Statement::GlobalDecl(ident) => out.push((stmt.pos.clone(), ident.clone())),
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
    let builtins: HashSet<&str> = crate::codegen_basic::BASIC_BUILTINS
        .iter()
        .copied()
        .collect();
    for function in &program.functions {
        if function.receiver.is_some() {
            continue;
        }
        let base_name = function.name.name.to_ascii_lowercase();
        if builtins.contains(base_name.as_str()) {
            let kind = if function.is_procedure {
                "procedure"
            } else {
                "function"
            };
            diagnostics.push(Diagnostic::error(
                function.pos.clone(),
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

/// `OPTION BASE` is accepted by the parser but was never properly
/// supported: `--target basic` passes it straight through as text with no
/// internal tracking of which base is active for which array, so array
/// bounds/`sizeof()`/an array-parameter's copy loop all silently assume
/// base 0 regardless -- concretely, a `dim`'d array under `OPTION BASE 1`
/// passed as a function parameter reads/writes index 0 of it, which
/// doesn't exist under that array's real base (confirmed: `fbc -exx`
/// aborts with "out of bounds array access" on exactly this). `--target c`
/// already rejects it as an unsupported statement. Rather than properly
/// supporting a non-zero base -- which would mean threading it through
/// every one of those call sites -- this rejects it outright, for both
/// targets, with a clear diagnostic instead of a silent wrong answer or an
/// opaque generated-code error.
fn reject_option_base(program: &Program, diagnostics: &mut Vec<Diagnostic>) {
    fn walk(statements: &[Stmt], diagnostics: &mut Vec<Diagnostic>) {
        for stmt in statements {
            match &stmt.kind {
                Statement::OptionBase(_) => {
                    diagnostics.push(Diagnostic::error(
                        stmt.pos.clone(),
                        "`OPTION BASE` isn't supported -- BASCAL only supports the default \
                         base-0 array indexing (see docs/manual/arrays.html); remove this \
                         statement and declare every array under base 0"
                            .to_string(),
                    ));
                }
                Statement::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    walk(then_body, diagnostics);
                    walk(else_body, diagnostics);
                }
                Statement::For { body, .. }
                | Statement::While { body, .. }
                | Statement::Do { body, .. } => {
                    walk(body, diagnostics);
                }
                Statement::SelectCase {
                    cases, else_body, ..
                } => {
                    for case in cases {
                        walk(&case.body, diagnostics);
                    }
                    walk(else_body, diagnostics);
                }
                _ => {}
            }
        }
    }

    walk(&program.statements, diagnostics);
    for function in &program.functions {
        walk(&function.body, diagnostics);
    }
}

fn reject_duplicate_functions(program: &Program, diagnostics: &mut Vec<Diagnostic>) {
    let mut seen = HashSet::new();
    for function in &program.functions {
        if function.receiver.is_some() {
            continue;
        }
        let name = function.name.as_basic().to_ascii_lowercase();
        if !seen.insert(name) {
            diagnostics.push(Diagnostic::error(
                function.pos.clone(),
                format!("duplicate function `{}`", function.name),
            ));
        }
    }
}

fn reject_scalar_methods(program: &Program, diagnostics: &mut Vec<Diagnostic>) {
    // A method is conceptually a function with its receiver as an implicit
    // first parameter -- see `records::Lowerer::rewrite_scalar_method_call`'s
    // own mirror-image rewrite, which lets ordinary-call syntax
    // (`ltrim$(s$)`) resolve straight to a method declaration
    // (`method ltrim[string]()` ) when no ordinary function of that name already
    // exists. That means a program can never legitimately declare *both* an
    // ordinary function and a method sharing the same (name, result suffix)
    // -- they'd be two different, ambiguous implementations of what's
    // supposed to be one callable identity, and (before this check existed)
    // silently collided on the same generated GOSUB label under
    // `--target basic`, with whichever declaration happened to be lowered
    // last winning every call site -- confirmed via real `fbc` execution,
    // no diagnostic at all.
    let ordinary_functions: HashSet<(String, Option<TypeSuffix>)> = program
        .functions
        .iter()
        .filter(|f| f.receiver.is_none())
        .map(|f| (f.name.name.to_ascii_lowercase(), f.name.suffix))
        .collect();

    let mut seen = HashSet::new();
    for function in &program.functions {
        let Some(receiver) = function.receiver else {
            continue;
        };
        let key = (receiver, function.name.name.to_ascii_lowercase());
        if !seen.insert(key) {
            diagnostics.push(Diagnostic::error(
                function.pos.clone(),
                format!("duplicate method `{}.{}`", receiver, function.name),
            ));
        }
        if ordinary_functions.contains(&(
            function.name.name.to_ascii_lowercase(),
            function.name.suffix,
        )) {
            diagnostics.push(Diagnostic::error(
                function.pos.clone(),
                format!(
                    "`{}` is declared as both a function and a method -- a method's receiver \
                     already fills the role of an ordinary function's first argument (calling \
                     `{}(x)` resolves to the method with `x` as the receiver when no ordinary \
                     function claims the name), so a program can't have both",
                    function.name, function.name
                ),
            ));
        }
        if function
            .params
            .iter()
            .any(|p| p.name.name.eq_ignore_ascii_case("self") && p.name.suffix == Some(receiver))
        {
            diagnostics.push(Diagnostic::error(
                function.pos.clone(),
                format!(
                    "method `{}` reserves implicit receiver `self{receiver}`",
                    function.name
                ),
            ));
        }
    }

    for function in &program.functions {
        let Some(receiver) = function.receiver else {
            continue;
        };
        let key = function.name.name.to_ascii_lowercase();
        if crate::codegen_basic::BASIC_BUILTINS
            .iter()
            .any(|b| *b == key)
        {
            diagnostics.push(Diagnostic::error(
                function.pos.clone(),
                format!(
                    "method name `{}` is reserved for built-in scalar methods",
                    function.name
                ),
            ));
        }
        validate_scalar_method_calls(&function.body, program, diagnostics, receiver);
    }
    walk_statements_exprs(&program.statements, &mut |expr, pos| {
        validate_one_scalar_method(expr, program, diagnostics, pos)
    });
}

fn validate_scalar_method_calls(
    body: &[Stmt],
    program: &Program,
    diagnostics: &mut Vec<Diagnostic>,
    _receiver: TypeSuffix,
) {
    for stmt in body {
        walk_statements_exprs(std::slice::from_ref(stmt), &mut |expr, pos| {
            validate_one_scalar_method(expr, program, diagnostics, pos)
        });
    }
}

fn validate_one_scalar_method(
    expr: &Expr,
    program: &Program,
    diagnostics: &mut Vec<Diagnostic>,
    pos: &SourcePos,
) {
    let Expr::ScalarMethodCall { base, method, .. } = expr else {
        return;
    };
    let Some(receiver) = scalar_expr_type(base, program) else {
        diagnostics.push(Diagnostic::error(
            pos.clone(),
            format!("method receiver for `.{method}()` must be a scalar expression"),
        ));
        return;
    };
    if !program
        .functions
        .iter()
        .any(|f| f.receiver == Some(receiver) && f.name.name.eq_ignore_ascii_case(method))
    {
        diagnostics.push(Diagnostic::error(
            pos.clone(),
            format!("unknown method `.{method}()` for receiver type `{receiver}`"),
        ));
    }
}

fn scalar_expr_type(expr: &Expr, program: &Program) -> Option<TypeSuffix> {
    match expr {
        Expr::String(_) => Some(TypeSuffix::String),
        Expr::Integer(_) | Expr::HexLit(_) => Some(TypeSuffix::Integer),
        Expr::Float(_) => Some(TypeSuffix::Single),
        Expr::Ident(id) | Expr::Call { name: id, .. } | Expr::ArrayRef { name: id, .. } => {
            Some(id.suffix.unwrap_or(TypeSuffix::Single))
        }
        Expr::Unary { expr, .. } => scalar_expr_type(expr, program),
        Expr::Binary { left, .. } => scalar_expr_type(left, program),
        Expr::ScalarMethodCall { base, method, .. } => {
            let recv = scalar_expr_type(base, program)?;
            program.functions.iter().find_map(|f| {
                (f.receiver == Some(recv) && f.name.name.eq_ignore_ascii_case(method))
                    .then_some(f.name.suffix)
                    .flatten()
            })
        }
        _ => None,
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
                            program.functions[path[start]].pos.clone(),
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
                function.pos.clone(),
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
                function.pos.clone(),
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
                function.pos.clone(),
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
                function.pos.clone(),
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

fn collect_on_error_goto_targets(statements: &[Stmt], out: &mut Vec<BasicIdent>) {
    for stmt in statements {
        match &stmt.kind {
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
fn diverges(statements: &[Stmt]) -> bool {
    let last = statements
        .iter()
        .rev()
        .find(|s| !matches!(s.kind, Statement::BlankLine));
    match last.map(|s| &s.kind) {
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

fn statements_call_function(statements: &[Stmt], target: &BasicIdent) -> bool {
    statements
        .iter()
        .any(|statement| statement_calls_function(statement, target))
}

fn statement_calls_function(statement: &Stmt, target: &BasicIdent) -> bool {
    match &statement.kind {
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
        Statement::ThrowStmt { code } => code
            .as_ref()
            .is_some_and(|code| expr_calls_function(code, target)),
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
        Statement::TryCatch {
            try_body,
            catch,
            finally_body,
            ..
        } => {
            statements_call_function(try_body, target)
                || catch.as_ref().is_some_and(|catch| {
                    catch
                        .error_filter
                        .iter()
                        .any(|e| expr_calls_function(e, target))
                        || statements_call_function(&catch.body, target)
                })
                || statements_call_function(finally_body, target)
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
        Expr::ScalarMethodCall { base, args, .. } => {
            expr_calls_function(base, target) || args.iter().any(|a| expr_calls_function(a, target))
        }
        Expr::RecordLit { fields, .. } => {
            fields.iter().any(|(_, e)| expr_calls_function(e, target))
        }
    }
}

fn contains_return(statements: &[Stmt]) -> bool {
    statements.iter().any(|statement| match &statement.kind {
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
        Statement::TryCatch {
            try_body,
            catch,
            finally_body,
            ..
        } => {
            contains_return(try_body)
                || catch
                    .as_ref()
                    .is_some_and(|catch| contains_return(&catch.body))
                || contains_return(finally_body)
        }
        _ => false,
    })
}

fn same_ident(left: &BasicIdent, right: &BasicIdent) -> bool {
    left.suffix == right.suffix && left.name.eq_ignore_ascii_case(&right.name)
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
/// printing them (see `compile_file`). Each finding points at the real
/// position of the statement that uses the undeclared name.
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
        for (_, global) in &globals {
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
fn collect_declarations(statements: &[Stmt], declared: &mut HashSet<VarKey>) {
    for stmt in statements {
        match &stmt.kind {
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
            Statement::TryCatch {
                try_body,
                catch,
                finally_body,
            } => {
                collect_declarations(try_body, declared);
                if let Some(catch) = catch {
                    declared.insert(var_key(&catch.err_var));
                    declared.insert(var_key(&catch.erl_var));
                    if let Some(source_var) = &catch.source_var {
                        declared.insert(var_key(source_var));
                    }
                    collect_declarations(&catch.body, declared);
                }
                collect_declarations(finally_body, declared);
            }
            _ => {}
        }
    }
}

fn check_var_uses(
    statements: &[Stmt],
    declared: &HashSet<VarKey>,
    known_callables: &HashSet<String>,
    warn_only: bool,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut check = |expr: &Expr, pos: &SourcePos| {
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
            pos.clone(),
            format!(
                "`{ident}` is used without a `dim`/`declare` -- --strict-vars requires every \
                 variable to be declared before use (a misspelled name is the usual cause)"
            ),
        ));
    };
    walk_statements_exprs(statements, &mut check);
}

fn walk_statements_exprs(statements: &[Stmt], f: &mut dyn FnMut(&Expr, &SourcePos)) {
    for stmt in statements {
        walk_statement_exprs(stmt, f);
    }
}

fn walk_statement_exprs(statement: &Stmt, f: &mut dyn FnMut(&Expr, &SourcePos)) {
    let pos = &statement.pos;
    match &statement.kind {
        Statement::Dim { sizes, .. } => sizes.iter().for_each(|e| walk_expr(e, pos, f)),
        Statement::Assignment { target, value } => {
            walk_expr(target, pos, f);
            walk_expr(value, pos, f);
        }
        Statement::MidAssign {
            target,
            start,
            len,
            value,
        } => {
            walk_expr(target, pos, f);
            walk_expr(start, pos, f);
            if let Some(len) = len {
                walk_expr(len, pos, f);
            }
            walk_expr(value, pos, f);
        }
        Statement::Print { tokens }
        | Statement::Lprint(tokens)
        | Statement::PrintFile { tokens, .. } => {
            for t in tokens {
                if let PrintToken::Expr(e) = t {
                    walk_expr(e, pos, f);
                }
            }
        }
        Statement::PrintUsing { format, tokens } | Statement::LprintUsing { format, tokens } => {
            walk_expr(format, pos, f);
            for t in tokens {
                if let PrintToken::Expr(e) = t {
                    walk_expr(e, pos, f);
                }
            }
        }
        Statement::PrintFileUsing {
            channel,
            format,
            tokens,
        } => {
            walk_expr(channel, pos, f);
            walk_expr(format, pos, f);
            for t in tokens {
                if let PrintToken::Expr(e) = t {
                    walk_expr(e, pos, f);
                }
            }
        }
        Statement::Open { file, channel, .. } => {
            walk_expr(file, pos, f);
            walk_expr(channel, pos, f);
        }
        Statement::FileDecl { path, .. } => walk_expr(path, pos, f),
        Statement::LineInput { channel, target } => {
            walk_expr(channel, pos, f);
            walk_expr(target, pos, f);
        }
        Statement::Close { channel } | Statement::Restore(Some(channel)) => {
            walk_expr(channel, pos, f)
        }
        Statement::Kill { file } => walk_expr(file, pos, f),
        Statement::Name { from, to } => {
            walk_expr(from, pos, f);
            walk_expr(to, pos, f);
        }
        Statement::Return { value } => walk_expr(value, pos, f),
        Statement::If {
            condition,
            then_body,
            else_body,
        } => {
            walk_expr(condition, pos, f);
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
            walk_expr(start, pos, f);
            walk_expr(end, pos, f);
            if let Some(step) = step {
                walk_expr(step, pos, f);
            }
            walk_statements_exprs(body, f);
        }
        Statement::While { condition, body } => {
            walk_expr(condition, pos, f);
            walk_statements_exprs(body, f);
        }
        Statement::ExprStmt(expr) => walk_expr(expr, pos, f),
        Statement::Do {
            condition,
            body,
            post_condition,
        } => {
            if let Some(c) = condition {
                walk_expr(&c.expr, pos, f);
            }
            walk_statements_exprs(body, f);
            if let Some(c) = post_condition {
                walk_expr(&c.expr, pos, f);
            }
        }
        Statement::Randomize(Some(e)) => walk_expr(e, pos, f),
        Statement::Swap(a, b) => {
            walk_expr(a, pos, f);
            walk_expr(b, pos, f);
        }
        Statement::Poke { address, value }
        | Statement::Out {
            port: address,
            value,
        } => {
            walk_expr(address, pos, f);
            walk_expr(value, pos, f);
        }
        Statement::Goto(e) | Statement::Gosub(e) => walk_expr(e, pos, f),
        Statement::OnErrorGoto { target } => walk_expr(target, pos, f),
        Statement::ErrorStmt { code } => walk_expr(code, pos, f),
        Statement::ThrowStmt { code } => {
            if let Some(code) = code {
                walk_expr(code, pos, f);
            }
        }
        Statement::Resume(ResumeTarget::Line(e)) => walk_expr(e, pos, f),
        Statement::Input { vars, .. } | Statement::Read(vars) => {
            vars.iter().for_each(|e| walk_expr(e, pos, f))
        }
        Statement::InputFile { channel, vars } => {
            walk_expr(channel, pos, f);
            vars.iter().for_each(|e| walk_expr(e, pos, f));
        }
        Statement::Data(values) => values.iter().for_each(|e| walk_expr(e, pos, f)),
        Statement::Const { value, .. } => walk_expr(value, pos, f),
        Statement::Write { channel, exprs } => {
            walk_expr(channel, pos, f);
            exprs.iter().for_each(|e| walk_expr(e, pos, f));
        }
        Statement::SelectCase {
            expr,
            cases,
            else_body,
        } => {
            walk_expr(expr, pos, f);
            for case in cases {
                for value in &case.values {
                    match value {
                        CaseValue::Single(e) | CaseValue::Is { value: e, .. } => {
                            walk_expr(e, pos, f)
                        }
                        CaseValue::Range { from, to } => {
                            walk_expr(from, pos, f);
                            walk_expr(to, pos, f);
                        }
                    }
                }
                walk_statements_exprs(&case.body, f);
            }
            walk_statements_exprs(else_body, f);
        }
        Statement::TryCatch {
            try_body,
            catch,
            finally_body,
            ..
        } => {
            walk_statements_exprs(try_body, f);
            if let Some(catch) = catch {
                for filter_expr in &catch.error_filter {
                    walk_expr(filter_expr, pos, f);
                }
                walk_statements_exprs(&catch.body, f);
            }
            walk_statements_exprs(finally_body, f);
        }
        Statement::Locate { row, col } => {
            walk_expr(row, pos, f);
            walk_expr(col, pos, f);
        }
        Statement::Color { fg, bg } => {
            walk_expr(fg, pos, f);
            if let Some(bg) = bg {
                walk_expr(bg, pos, f);
            }
        }
        Statement::OnBranch { expr, targets, .. } => {
            walk_expr(expr, pos, f);
            targets.iter().for_each(|e| walk_expr(e, pos, f));
        }
        Statement::Field {
            channel, fields, ..
        } => {
            walk_expr(channel, pos, f);
            fields.iter().for_each(|(w, _)| walk_expr(w, pos, f));
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
            walk_expr(channel, pos, f);
            if let Some(record) = record {
                walk_expr(record, pos, f);
            }
            if let Some(var) = var {
                walk_expr(var, pos, f);
            }
        }
        Statement::Lset { value, .. } | Statement::Rset { value, .. } => walk_expr(value, pos, f),
        Statement::Seek { channel, position } => {
            walk_expr(channel, pos, f);
            walk_expr(position, pos, f);
        }
        Statement::OptionBase(e) => walk_expr(e, pos, f),
        Statement::Width { channel, cols } => {
            if let Some(channel) = channel {
                walk_expr(channel, pos, f);
            }
            walk_expr(cols, pos, f);
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

fn walk_expr(expr: &Expr, pos: &SourcePos, f: &mut dyn FnMut(&Expr, &SourcePos)) {
    f(expr, pos);
    match expr {
        Expr::Integer(_) | Expr::Float(_) | Expr::HexLit(_) | Expr::String(_) | Expr::Ident(_) => {}
        Expr::ArrayRef { indices, .. } => indices.iter().for_each(|e| walk_expr(e, pos, f)),
        Expr::Call { args, .. } => args.iter().for_each(|e| walk_expr(e, pos, f)),
        Expr::Unary { expr, .. } => walk_expr(expr, pos, f),
        Expr::Binary { left, right, .. } => {
            walk_expr(left, pos, f);
            walk_expr(right, pos, f);
        }
        Expr::FileIndex { index, .. } => walk_expr(index, pos, f),
        Expr::FieldAccess { base, .. } => walk_expr(base, pos, f),
        Expr::MethodCall { base, args, .. } => {
            walk_expr(base, pos, f);
            args.iter().for_each(|e| walk_expr(e, pos, f));
        }
        Expr::ScalarMethodCall { base, args, .. } => {
            walk_expr(base, pos, f);
            args.iter().for_each(|e| walk_expr(e, pos, f));
        }
        Expr::RecordLit { fields, .. } => fields.iter().for_each(|(_, e)| walk_expr(e, pos, f)),
    }
}

// ── legacy-form deprecation warnings ─────────────────────────────────────
//
// BASCAL stays a superset of classic BASIC: every legacy form below keeps
// compiling exactly as before. But where a legacy form has a direct BASCAL
// structured equivalent, `check_legacy_forms` names it as an advisory
// warning, steering new/edited source toward the structured spelling
// without breaking existing programs (see issue #2 / CLAUDE.md's own
// framing of "guiding new source toward BASCAL structured syntax"). Always
// on -- these are warnings, never errors, so the caller (`compile_source`/
// `compile_file`) just prints them and keeps going.
//
// Run on the fully merged, lowered `Program` (after `records::lower`), not
// the root file's raw parse -- unlike `check_strict_vars` above, the
// `FIELD` check here depends on `records::lower` having already turned
// `file ... as ... = open(...)` into a `FIELD` with `record_type: Some`,
// which is exactly what distinguishes it from a hand-written `FIELD`.

/// Three legacy forms are recognised, each with an unambiguous BASCAL
/// equivalent:
///   - `ON ... GOTO`/`ON ... GOSUB` computed branching -- `SELECT CASE`.
///   - a chain of at least two `IF ... THEN GOTO` / `ELSEIF ... THEN GOTO`
///     links used purely to dispatch to different labels -- `SELECT CASE`.
///     A single, unchained `IF cond THEN GOTO label` is left alone: that's
///     an ordinary early-exit/error-check branch, not a dispatch table.
///   - a `GOTO` back to a label already seen earlier in the same body --
///     a hand-wired loop -- `DO ... LOOP` / `WHILE ... END WHILE`.
///   - a hand-typed `FIELD` statement (`record_type: None` -- see
///     `Statement::Field`'s own doc comment) -- `record`/`file`.
pub fn check_legacy_forms(program: &Program) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    check_legacy_forms_in_body(&program.statements, &mut diagnostics);
    for function in &program.functions {
        check_legacy_forms_in_body(&function.body, &mut diagnostics);
    }
    diagnostics
}

fn check_legacy_forms_in_body(body: &[Stmt], diagnostics: &mut Vec<Diagnostic>) {
    let mut seen_labels: HashSet<String> = HashSet::new();
    walk_legacy_forms(body, &mut seen_labels, diagnostics);
}

/// Depth of a pure `IF/ELSEIF ... THEN GOTO` dispatch chain rooted at
/// `stmt` -- `Some(n)` for an `If` whose `then_body` is exactly one `GOTO`
/// and whose `else_body` is either empty, exactly one `GOTO`, or another
/// such chain (`n` counts the links); `None` if `stmt` isn't an `If`, or
/// its body holds anything but bare `GOTO`s.
fn goto_if_chain_depth(stmt: &Statement) -> Option<usize> {
    let Statement::If {
        then_body,
        else_body,
        ..
    } = stmt
    else {
        return None;
    };
    if !matches!(then_body.as_slice(), [s] if matches!(s.kind, Statement::Goto(_))) {
        return None;
    }
    match else_body.as_slice() {
        [] => Some(1),
        [s] if matches!(s.kind, Statement::Goto(_)) => Some(1),
        [nested] if matches!(nested.kind, Statement::If { .. }) => {
            goto_if_chain_depth(&nested.kind).map(|d| d + 1)
        }
        _ => None,
    }
}

fn walk_legacy_forms(
    statements: &[Stmt],
    seen_labels: &mut HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for stmt in statements {
        // Computed once per `If` and reused below: a chain reported as a
        // dispatch warning (depth >= 2) has nothing further worth
        // recursing into (then_body/else_body only hold bare GOTOs and the
        // chained `If`s already covered by the depth count) -- but a lone
        // `IF cond THEN GOTO label` (depth == 1, not itself warned about as
        // a "chain") still needs its `GOTO` visited normally, since that's
        // exactly the shape a hand-wired backward-jump loop takes.
        let chain_depth = matches!(stmt.kind, Statement::If { .. })
            .then(|| goto_if_chain_depth(&stmt.kind))
            .flatten();

        match &stmt.kind {
            Statement::Label(name) => {
                seen_labels.insert(name.to_ascii_lowercase());
            }
            Statement::OnBranch { is_gosub, .. } => {
                let legacy = if *is_gosub {
                    "ON ... GOSUB"
                } else {
                    "ON ... GOTO"
                };
                diagnostics.push(Diagnostic::warning(
                    stmt.pos.clone(),
                    format!(
                        "`{legacy}` is a legacy computed-branch dispatch with a direct BASCAL \
                         equivalent -- prefer `SELECT CASE`, which expresses the same dispatch \
                         without a positional branch-target list"
                    ),
                ));
            }
            Statement::If { .. } if chain_depth.is_some_and(|d| d >= 2) => {
                diagnostics.push(Diagnostic::warning(
                    stmt.pos.clone(),
                    "this `IF ... THEN GOTO` / `ELSEIF ... THEN GOTO` chain dispatches to \
                     different labels by condition -- prefer `SELECT CASE`, BASCAL's structured \
                     equivalent for the same dispatch"
                        .to_string(),
                ));
            }
            Statement::Goto(Expr::Ident(ident))
                if seen_labels.contains(&ident.name.to_ascii_lowercase()) =>
            {
                diagnostics.push(Diagnostic::warning(
                    stmt.pos.clone(),
                    format!(
                        "`GOTO {ident}` jumps back to a label already seen earlier in this \
                         block -- this is a hand-wired loop with a direct BASCAL equivalent -- \
                         prefer `DO ... LOOP` or `WHILE ... END WHILE`"
                    ),
                ));
            }
            Statement::Field {
                record_type: None, ..
            } => {
                diagnostics.push(Diagnostic::warning(
                    stmt.pos.clone(),
                    "hand-written `FIELD` bookkeeping has a direct BASCAL equivalent -- a \
                     `record ... end record` + `file ... as ... = open(...)` declaration \
                     expresses the same random-access layout without manually tracking \
                     FIELD/LSET/GET/PUT offsets"
                        .to_string(),
                ));
            }
            _ => {}
        }

        match &stmt.kind {
            // A reported dispatch chain (depth >= 2) has nothing left to
            // recurse into -- see the comment on `chain_depth` above.
            Statement::If { .. } if chain_depth.is_some_and(|d| d >= 2) => {}
            Statement::If {
                then_body,
                else_body,
                ..
            } => {
                walk_legacy_forms(then_body, seen_labels, diagnostics);
                walk_legacy_forms(else_body, seen_labels, diagnostics);
            }
            Statement::For { body, .. }
            | Statement::While { body, .. }
            | Statement::Do { body, .. } => {
                walk_legacy_forms(body, seen_labels, diagnostics);
            }
            Statement::SelectCase {
                cases, else_body, ..
            } => {
                for case in cases {
                    walk_legacy_forms(&case.body, seen_labels, diagnostics);
                }
                walk_legacy_forms(else_body, seen_labels, diagnostics);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod legacy_form_tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn parse(source: &str) -> Program {
        let tokens = Lexer::new("test.bcl", source).lex();
        Parser::new("test.bcl".to_string(), tokens)
            .parse_program()
            .expect("source should parse")
    }

    fn messages(source: &str) -> Vec<String> {
        check_legacy_forms(&parse(source))
            .into_iter()
            .map(|d| d.message)
            .collect()
    }

    #[test]
    fn on_goto_is_flagged() {
        let msgs = messages("on x goto first, second\nfirst:\nsecond:\nend\n");
        assert_eq!(msgs.len(), 1, "unexpected findings: {msgs:?}");
        assert!(msgs[0].contains("SELECT CASE"), "{}", msgs[0]);
        assert!(msgs[0].contains("ON ... GOTO"), "{}", msgs[0]);
    }

    #[test]
    fn on_gosub_is_flagged() {
        let msgs = messages("on x gosub first, second\nfirst:\nreturn\nsecond:\nreturn\nend\n");
        assert_eq!(msgs.len(), 1, "unexpected findings: {msgs:?}");
        assert!(msgs[0].contains("ON ... GOSUB"), "{}", msgs[0]);
    }

    #[test]
    fn scalar_methods_resolve_by_receiver_and_result_type() {
        let program = parse(
            "method capitalize$[string]()\nreturn self$\nend method\nmethod pad$[string](n%)\nreturn self$\nend method\ns$ = name$.capitalize().pad(2)\nend\n",
        );
        assert!(validate(&program).is_ok(), "{:?}", validate(&program));
    }

    #[test]
    fn unknown_scalar_methods_are_rejected() {
        let program = parse("s$ = name$.missing()\nend\n");
        let diagnostics = validate(&program).expect_err("unknown method should fail");
        assert!(diagnostics
            .iter()
            .any(|d| d.message.contains("unknown method")));
    }

    #[test]
    fn single_if_then_goto_is_not_flagged_as_a_dispatch_chain() {
        // A lone early-exit/error-check branch, not a dispatch table.
        let msgs = messages("if eof(1) then goto donereading\ndonereading:\nend\n");
        assert!(
            msgs.is_empty(),
            "a single unchained IF...THEN GOTO shouldn't be flagged: {msgs:?}"
        );
    }

    #[test]
    fn chained_if_elseif_then_goto_dispatch_is_flagged() {
        let source = r#"if x = 1 then
goto one
elseif x = 2 then
goto two
else
goto other
end if
one:
print "one"
goto done
two:
print "two"
goto done
other:
print "other"
done:
end
"#;
        let msgs = messages(source);
        assert_eq!(msgs.len(), 1, "unexpected findings: {msgs:?}");
        assert!(msgs[0].contains("SELECT CASE"), "{}", msgs[0]);
    }

    #[test]
    fn backward_goto_to_an_earlier_label_is_flagged_as_a_hand_wired_loop() {
        let source = "mainloop:\nx = x + 1\nif x < 10 then goto mainloop\nend\n";
        let msgs = messages(source);
        assert_eq!(msgs.len(), 1, "unexpected findings: {msgs:?}");
        assert!(msgs[0].contains("GOTO mainloop"), "{}", msgs[0]);
        assert!(msgs[0].contains("DO ... LOOP"), "{}", msgs[0]);
    }

    #[test]
    fn forward_goto_to_a_later_label_is_not_flagged() {
        let source = "if x = 1 then goto skip\nprint \"not skipped\"\nskip:\nend\n";
        let msgs = messages(source);
        assert!(msgs.is_empty(), "a forward jump isn't a loop: {msgs:?}");
    }

    #[test]
    fn structured_loops_and_select_case_are_never_flagged() {
        let source = r#"for i = 1 to 5
print i
end for

x = 1
do while x < 5
x = x + 1
end do

select case x
case 1
print "one"
case else
print "other"
end select
end
"#;
        assert!(messages(source).is_empty());
    }

    #[test]
    fn record_file_dsl_is_not_flagged_after_lowering() {
        // Mirrors the check's real call site in lib.rs: `check_legacy_forms`
        // runs on the post-`records::lower` program, where DSL-synthesized
        // `FIELD` statements carry `record_type: Some(..)` -- unlike a
        // hand-typed `FIELD`, which leaves it `None` (see `Statement::
        // Field`'s own doc comment).
        let source = r#"record Item
    name: string(10)
end record

file items as Item = open("probe.dat")

items[1] = { name: "widget" }
items.close()
end
"#;
        let (lowered, _) = crate::records::lower(parse(source)).expect("should lower");
        let msgs: Vec<String> = check_legacy_forms(&lowered)
            .into_iter()
            .map(|d| d.message)
            .collect();
        assert!(
            msgs.is_empty(),
            "record/file DSL usage shouldn't be flagged as hand-written FIELD bookkeeping: {msgs:?}"
        );
    }

    #[test]
    fn hand_written_field_is_flagged() {
        let source = r#"open "data.dat" for random as #1 len = 10
field #1, 10 as buf$
close #1
end
"#;
        let msgs = messages(source);
        assert_eq!(msgs.len(), 1, "unexpected findings: {msgs:?}");
        assert!(msgs[0].contains("FIELD"), "{}", msgs[0]);
        assert!(msgs[0].contains("record"), "{}", msgs[0]);
    }
}

#[cfg(test)]
mod position_tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn parse(source: &str) -> Program {
        let tokens = Lexer::new("test.bcl", source).lex();
        Parser::new("test.bcl".to_string(), tokens)
            .parse_program()
            .expect("source should parse")
    }

    /// Every diagnostic in this module must point at a real line, never the
    /// old `<validation>:1:1` placeholder -- see issue #27.
    fn assert_real_pos(diags: &[Diagnostic]) {
        for d in diags {
            assert_ne!(
                d.pos.filename, "<validation>",
                "diagnostic still uses the placeholder position: {}",
                d.message
            );
        }
    }

    #[test]
    fn duplicate_function_points_at_the_second_declaration() {
        let source =
            "function f(x)\nreturn x\nend function\nfunction f(y)\nreturn y\nend function\nend\n";
        let program = parse(source);
        let diags = validate(&program).unwrap_err();
        assert_real_pos(&diags);
        let dup = diags
            .iter()
            .find(|d| d.message.contains("duplicate function"))
            .expect("expected a duplicate-function diagnostic");
        assert_eq!(dup.pos.line, 4, "diagnostics: {diags:?}");
    }

    #[test]
    fn function_shadowing_builtin_points_at_its_declaration() {
        let source = "\n\nfunction len(x)\nreturn x\nend function\nend\n";
        let program = parse(source);
        let diags = validate(&program).unwrap_err();
        assert_real_pos(&diags);
        let shadow = diags
            .iter()
            .find(|d| d.message.contains("built-in"))
            .expect("expected a shadowing diagnostic");
        assert_eq!(shadow.pos.line, 3, "diagnostics: {diags:?}");
    }

    #[test]
    fn recursive_call_points_at_the_function_declaration() {
        let source = "\nfunction f(x)\nreturn f(x)\nend function\nend\n";
        let program = parse(source);
        let diags = validate(&program).unwrap_err();
        assert_real_pos(&diags);
        let rec = diags
            .iter()
            .find(|d| d.message.contains("recursion"))
            .expect("expected a recursion diagnostic");
        assert_eq!(rec.pos.line, 2, "diagnostics: {diags:?}");
    }

    #[test]
    fn missing_return_points_at_the_function_declaration() {
        let source = "\n\n\nfunction f(x)\nprint x\nend function\nend\n";
        let program = parse(source);
        let diags = validate(&program).unwrap_err();
        assert_real_pos(&diags);
        let missing = diags
            .iter()
            .find(|d| d.message.contains("implicit function return"))
            .expect("expected a missing-return diagnostic");
        assert_eq!(missing.pos.line, 4, "diagnostics: {diags:?}");
    }

    #[test]
    fn global_shadowing_param_points_at_the_global_statement() {
        let source = "procedure p(x)\n\n\nglobal x\nend procedure\nend\n";
        let program = parse(source);
        let diags = validate(&program).unwrap_err();
        assert_real_pos(&diags);
        let shadow = diags
            .iter()
            .find(|d| d.message.contains("names a parameter"))
            .expect("expected a global-shadows-param diagnostic");
        assert_eq!(shadow.pos.line, 4, "diagnostics: {diags:?}");
    }

    #[test]
    fn unsafe_error_handler_points_at_the_procedure_declaration() {
        let source = "on error goto handler\n\nprocedure handler()\nreturn\nend procedure\nend\n";
        let program = parse(source);
        let diags = validate(&program).unwrap_err();
        assert_real_pos(&diags);
        let unsafe_handler = diags
            .iter()
            .find(|d| d.message.contains("cannot contain `return`"))
            .expect("expected an unsafe-error-handler diagnostic");
        assert_eq!(unsafe_handler.pos.line, 3, "diagnostics: {diags:?}");
    }

    #[test]
    fn strict_vars_points_at_the_offending_statement() {
        let source = "\n\nprint undeclaredvar\nend\n";
        let program = parse(source);
        let diags = check_strict_vars(&program, false);
        assert_real_pos(&diags);
        assert_eq!(diags.len(), 1, "diagnostics: {diags:?}");
        assert_eq!(diags[0].pos.line, 3, "diagnostics: {diags:?}");
    }

    #[test]
    fn legacy_form_points_at_the_offending_statement() {
        let source = "\n\n\non x goto first, second\nfirst:\nsecond:\nend\n";
        let program = parse(source);
        let diags = check_legacy_forms(&program);
        assert_real_pos(&diags);
        assert_eq!(diags.len(), 1, "diagnostics: {diags:?}");
        assert_eq!(diags[0].pos.line, 4, "diagnostics: {diags:?}");
    }
}
