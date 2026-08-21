//! Minimal native-C backend.
//!
//! Deliberately narrow: this only understands a top-level `print` of
//! string/numeric literals -- including negation, every arithmetic
//! operator (`+`/`-`/`*`/`/`/`\`/MOD/`^`), every comparison operator
//! (`=`/`<>`/`<`/`<=`/`>`/`>=`), and every bitwise/logical operator
//! (`AND`/`OR`/`XOR`/`NOT` -- genuinely bitwise, not short-circuit
//! booleans) of them -- `end`, `dim`, `const`, assignment/reading of
//! *numeric scalar* variables (`%`/`&`/`!`/`#`), and `if`/`elseif`/`else`/
//! `end if` (including the single-line form, and nesting), wrapped in
//! `int main(void) { ... }`. Everything else (functions, other statement
//! kinds, string variables, arrays, loops) reports a "not supported yet"
//! diagnostic rather than panicking or emitting wrong code -- this is a
//! walking skeleton to prove the CLI/dispatch plumbing (`Target::C`,
//! `--target c`, `invoke_gcc`) end-to-end, not a real backend.
//!
//! Numeric `print` output is plain `%d`/`%g` `printf` formatting -- it does
//! not reproduce real MBASIC/BASCOM's own numeric `PRINT` convention (a
//! leading space standing in for a sign, a trailing space after the number).
//! Matching that exactly is future work, not attempted here.
//!
//! When this grows beyond `print`/`end`: record layout must NOT be expressed
//! as a plain C `struct` -- alignment padding would break binary
//! compatibility with the packed, no-padding layout `FIELD`/`GET`/`PUT` use
//! on the BASIC side. Every record field needs to be (de)serialized
//! explicitly at the byte offsets `records.rs` already computes for the
//! BASIC backend, the same offsets both backends should share.

use std::collections::{BTreeMap, BTreeSet};

use crate::ast::{BasicIdent, BinaryOp, Expr, PrintToken, Program, Statement, TypeSuffix, UnaryOp};
use crate::diagnostics::{Diagnostic, SourcePos};

pub(crate) fn generate(program: &Program) -> Result<String, Vec<Diagnostic>> {
    if !program.functions.is_empty() {
        return Err(vec![unsupported(
            "functions/procedures are not supported by the minimal C backend yet",
        )]);
    }

    // BASIC variables "spring into existence on first use" -- there's no
    // separate declaration step to hook into, so every scalar variable
    // touched anywhere in the program (by `dim`, an assignment target, or
    // a read) is collected up front and declared, zero-initialized (for a
    // string, that means an all-zero buffer -- a valid, empty C string),
    // at the very top of `main`, regardless of where it's first mentioned
    // in source order. This pass is deliberately infallible -- it only
    // ever *adds* a declaration for a variable shape it understands;
    // anything it doesn't (arrays, ...) is silently skipped here and
    // reported as a real error later, when `emit_statement`/
    // `render_numeric_expr`/`render_string_expr` actually tries to use it.
    let mut numeric_vars = BTreeMap::new();
    let mut string_vars = BTreeSet::new();
    for statement in &program.statements {
        collect_vars_in_statement(statement, &mut numeric_vars, &mut string_vars);
    }

    let mut body = String::new();
    for (c_name, c_type) in &numeric_vars {
        body.push_str(&format!("    {c_type} {c_name} = 0;\n"));
    }
    for c_name in &string_vars {
        body.push_str(&format!("    char {c_name}[{STRING_BUFFER_SIZE}] = {{0}};\n"));
    }
    if !numeric_vars.is_empty() || !string_vars.is_empty() {
        body.push('\n');
    }

    let mut needs_math = false;
    let mut temp_counter = 0;
    for statement in &program.statements {
        emit_statement(statement, &mut body, &mut needs_math, &mut temp_counter)
            .map_err(|message| vec![unsupported(&message)])?;
    }
    // `Statement::End` already emits its own `return 0;` -- only add the
    // implicit fallthrough one when the program didn't already end with an
    // explicit `end` (comments/blank lines don't count), otherwise `main`
    // would end in two `return 0;` statements back to back.
    if !ends_with_end(&program.statements) {
        body.push_str("    return 0;\n");
    }

    // <math.h> is only pulled in when something (currently just `\`) needs
    // round() from it -- most programs won't.
    let includes =
        if needs_math { "#include <stdio.h>\n#include <math.h>\n" } else { "#include <stdio.h>\n" };
    Ok(format!("{includes}\nint main(void) {{\n{body}}}\n"))
}

/// The C identifier a BASIC scalar variable maps to. Prefixed (`bv_...`,
/// "bascal variable") so a BASIC name that happens to collide with a C
/// keyword (a variable named `int`, say) can't produce invalid C, and
/// tagged with `suffix` so `x%`/`x&`/`x!`/`x#` -- distinct BASIC
/// variables, despite sharing a base name -- map to distinct C names
/// instead of colliding. Lowercases the name first since BASIC identifiers
/// are case-insensitive; BASCAL source can't contain an underscore in an
/// identifier (rejected at parse time, see `reject_underscored_identifiers`
/// in `lib.rs`), so the lowercased name alone is already collision-free as
/// a C identifier fragment.
fn c_var_name(ident: &BasicIdent, suffix: TypeSuffix) -> String {
    let tag = match suffix {
        TypeSuffix::Integer => 'i',
        TypeSuffix::Long => 'l',
        TypeSuffix::Single => 'f',
        TypeSuffix::Double => 'd',
        TypeSuffix::String => 's',
    };
    format!("bv_{tag}_{}", ident.name.to_ascii_lowercase())
}

/// The C type and `printf`/`render_numeric_expr` float-ness for a numeric
/// scalar variable's suffix. `%`/`&` (BASIC's 16-bit integer and 32-bit
/// long) are deliberately collapsed to the same plain C `int`/`%d` bucket
/// -- the same simplification this backend already makes for arithmetic
/// results (see `IntDiv`/`Mod` above), rather than introducing a `long`/
/// `%ld` type-tracking dimension for one BASIC type that's rarely
/// distinguished from `%` in practice. Returns `None` for `$`/no suffix --
/// string variables and suffixless (default-type) variables aren't
/// supported yet.
fn numeric_c_type(suffix: TypeSuffix) -> Option<(&'static str, bool)> {
    match suffix {
        TypeSuffix::Integer | TypeSuffix::Long => Some(("int", false)),
        TypeSuffix::Single => Some(("float", true)),
        TypeSuffix::Double => Some(("double", true)),
        TypeSuffix::String => None,
    }
}

fn collect_vars_in_statement(
    statement: &Statement,
    numeric_out: &mut BTreeMap<String, &'static str>,
    string_out: &mut BTreeSet<String>,
) {
    match statement {
        Statement::Dim { name, is_array: false, sizes } if sizes.is_empty() => {
            register_var(name, numeric_out, string_out);
        }
        Statement::Assignment { target: Expr::Ident(name), value }
        | Statement::Const { name, value } => {
            register_var(name, numeric_out, string_out);
            collect_vars_in_expr(value, numeric_out, string_out);
        }
        Statement::Print { tokens } => {
            for token in tokens {
                if let PrintToken::Expr(expr) = token {
                    collect_vars_in_expr(expr, numeric_out, string_out);
                }
            }
        }
        Statement::If { condition, then_body, else_body } => {
            collect_vars_in_expr(condition, numeric_out, string_out);
            for stmt in then_body {
                collect_vars_in_statement(stmt, numeric_out, string_out);
            }
            for stmt in else_body {
                collect_vars_in_statement(stmt, numeric_out, string_out);
            }
        }
        _ => {}
    }
}

fn collect_vars_in_expr(
    expr: &Expr,
    numeric_out: &mut BTreeMap<String, &'static str>,
    string_out: &mut BTreeSet<String>,
) {
    match expr {
        Expr::Ident(ident) => register_var(ident, numeric_out, string_out),
        Expr::Unary { expr, .. } => collect_vars_in_expr(expr, numeric_out, string_out),
        Expr::Binary { left, right, .. } => {
            collect_vars_in_expr(left, numeric_out, string_out);
            collect_vars_in_expr(right, numeric_out, string_out);
        }
        _ => {}
    }
}

fn register_var(
    ident: &BasicIdent,
    numeric_out: &mut BTreeMap<String, &'static str>,
    string_out: &mut BTreeSet<String>,
) {
    match ident.suffix {
        Some(TypeSuffix::String) => {
            string_out.insert(c_var_name(ident, TypeSuffix::String));
        }
        Some(suffix) => {
            if let Some((c_type, _)) = numeric_c_type(suffix) {
                numeric_out.insert(c_var_name(ident, suffix), c_type);
            }
        }
        None => {}
    }
}

fn emit_statement(
    statement: &Statement,
    out: &mut String,
    needs_math: &mut bool,
    temp_counter: &mut usize,
) -> Result<(), String> {
    match statement {
        Statement::Print { tokens } => {
            let (prelude, mut format, args, needs_newline) =
                render_print_tokens(tokens, needs_math, temp_counter)?;
            for line in prelude {
                out.push_str(&line);
            }
            if needs_newline {
                format.push_str("\\n");
            }
            let mut call = format!("printf(\"{format}\"");
            for arg in &args {
                call.push_str(", ");
                call.push_str(arg);
            }
            call.push(')');
            out.push_str(&format!("    {call};\n"));
            Ok(())
        }
        Statement::End => {
            out.push_str("    return 0;\n");
            Ok(())
        }
        // Declarations are hoisted to the top of `main` up front (see
        // `collect_numeric_vars_in_statement` in `generate`), matching
        // BASIC's "springs into existence on first use" semantics -- `dim`
        // of a scalar is therefore a pure no-op here, already handled
        // wherever it happens to appear in source order.
        Statement::Dim { name, is_array: false, sizes } if sizes.is_empty() => match name.suffix {
            Some(suffix) if numeric_c_type(suffix).is_some() || suffix == TypeSuffix::String => Ok(()),
            _ => Err(format!(
                "`dim {name}` isn't supported by the minimal C backend yet -- only scalar \
                 variables (%, &, !, #, $) are"
            )),
        },
        Statement::Assignment { target: Expr::Ident(name), value } => {
            emit_assignment(name, value, out, needs_math, temp_counter)
        }
        // Real MBASIC/BASCOM has no CONST statement at all -- `const` in
        // `.bcl` source is purely a naming/intent signal to the reader
        // (BASCAL's resolver already enforces it's never reassigned before
        // codegen ever runs), so it codegens exactly like an ordinary
        // assignment, same as the BASIC backend's own treatment of it.
        Statement::Const { name, value } => {
            emit_assignment(name, value, out, needs_math, temp_counter)
        }
        // Unlike the BASIC backend, which has to transpile `if`/`elseif`/
        // `else` into a GOTO/label chain (real MBASIC/BASCOM has no block
        // `IF`), C has native `if`/`else`, so this is a direct structural
        // translation -- no labels needed. `elseif` doesn't need separate
        // handling either: the parser already desugars it into a single
        // nested `Statement::If` inside `else_body`, which the recursive
        // `emit_statement` call below just walks into naturally, producing
        // (harmless, if not maximally idiomatic) `} else {\n if (...) {`
        // nesting rather than a flat `else if` chain. Body statements are
        // NOT re-indented per nesting level (still flush against the same
        // base indent as everything else) -- purely cosmetic, not a
        // correctness gap.
        Statement::If { condition, then_body, else_body } => {
            let (cond_text, _) = render_numeric_expr(condition, needs_math)?;
            out.push_str(&format!("    if ({cond_text}) {{\n"));
            for stmt in then_body {
                emit_statement(stmt, out, needs_math, temp_counter)?;
            }
            if else_body.is_empty() {
                out.push_str("    }\n");
            } else {
                out.push_str("    } else {\n");
                for stmt in else_body {
                    emit_statement(stmt, out, needs_math, temp_counter)?;
                }
                out.push_str("    }\n");
            }
            Ok(())
        }
        Statement::BlankLine => {
            out.push('\n');
            Ok(())
        }
        Statement::BlockComment(lines) => {
            for line in lines {
                out.push_str(&format!("    // {line}\n"));
            }
            Ok(())
        }
        // A `'`- or `//`-style single-line comment always parses to
        // `Statement::Raw("' <text>")` (see `parser.rs`) -- genuine raw
        // BASIC passthrough (hand-written GOTO/OPEN/etc.) would land here
        // too, but with no leading `'`, so only the comment shape is safe
        // to translate; anything else falls through to the generic error.
        Statement::Raw(text) if text.trim_start().starts_with('\'') => {
            let comment = text.trim_start().trim_start_matches('\'').trim_start();
            out.push_str(&format!("    // {comment}\n"));
            Ok(())
        }
        other => Err(format!(
            "{other:?} is not supported by the minimal C backend yet -- only `print`, `end`, \
             `dim`, `if`, and assignment/`const` of scalar variables (%, &, !, #, $) are \
             implemented so far"
        )),
    }
}

/// Shared by `Statement::Assignment` and `Statement::Const` -- both are
/// "evaluate `value`, store it in `name`'s variable," identical at the C
/// level (see the `Const` match arm's comment for why). Dispatches between
/// numeric and string variables; anything else (an array target, a
/// suffixless variable) is an error.
fn emit_assignment(
    name: &BasicIdent,
    value: &Expr,
    out: &mut String,
    needs_math: &mut bool,
    temp_counter: &mut usize,
) -> Result<(), String> {
    match name.suffix {
        Some(suffix) if numeric_c_type(suffix).is_some() => {
            let (value_text, _) = render_numeric_expr(value, needs_math)?;
            out.push_str(&format!("    {} = {value_text};\n", c_var_name(name, suffix)));
            Ok(())
        }
        Some(TypeSuffix::String) => {
            let (prelude, value_text) = render_string_expr(value, needs_math, temp_counter)?;
            for line in prelude {
                out.push_str(&line);
            }
            let c_name = c_var_name(name, TypeSuffix::String);
            out.push_str(&format!(
                "    snprintf({c_name}, sizeof({c_name}), \"%s\", {value_text});\n"
            ));
            Ok(())
        }
        _ => Err(format!(
            "assignment to `{name}` isn't supported by the minimal C backend yet -- give it an \
             explicit type suffix (%, &, !, #, $)"
        )),
    }
}

/// Whether `expr` is a string-typed expression -- a string literal, a
/// read of a `$`-suffixed variable, or `+` (concatenation) where either
/// side is. Used to route a `print`/assignment expression to
/// `render_string_expr` instead of `render_numeric_expr`; BASCAL's own
/// resolver has already rejected genuinely mixed-type `+` (a string plus a
/// number) before codegen ever runs, so this heuristic (check one side,
/// trust the program is well-typed) doesn't need to be a full type checker.
fn is_string_expr(expr: &Expr) -> bool {
    match expr {
        Expr::String(_) => true,
        Expr::Ident(ident) => ident.suffix == Some(TypeSuffix::String),
        Expr::Binary { op: BinaryOp::Add, left, right } => {
            is_string_expr(left) || is_string_expr(right)
        }
        _ => false,
    }
}

/// Every string variable/temporary is a fixed-size buffer -- real BASIC
/// strings are dynamically sized (heap-allocated, grow/shrink freely),
/// which this minimal backend doesn't attempt to replicate. `snprintf` is
/// used for every write into one of these buffers specifically so a string
/// longer than fits is *safely truncated*, never a buffer overflow --
/// unlike `strcpy`/`strcat`, which this backend deliberately never emits.
const STRING_BUFFER_SIZE: usize = 256;

/// Renders a string expression tree as C expression text usable as a
/// `char*` argument (an `snprintf`/`printf` `%s` argument, an assignment
/// source), plus any statements that must run first to materialize it.
///
/// String literals and `$`-suffixed variable reads need no prelude -- a
/// quoted C string literal and a buffer name (which decays to `char*`)
/// are both usable directly as a `%s` argument. `+` (concatenation)
/// does: C has no string-concatenation operator, so each concatenation
/// gets its own freshly declared temp buffer (`bt_s_<n>`, `temp_counter`
/// keeps every one unique within the program) and an `snprintf(dest,
/// sizeof(dest), "%s%s", left, right)` call -- safe against overflow the
/// same way `emit_assignment`'s string case is, and it's why plain C99
/// mid-block declarations (the temp buffer's `char bt_s_n[256];` line)
/// are emitted right where first needed rather than hoisted, unlike named
/// variables. A chain like `a$ + b$ + c$` (left-associative, same as
/// BASIC) therefore costs one temp buffer per `+`, not just one for the
/// whole chain -- more buffers than strictly necessary, but simple and
/// correct, consistent with this backend's other "correct over clever"
/// choices.
fn render_string_expr(
    expr: &Expr,
    needs_math: &mut bool,
    temp_counter: &mut usize,
) -> Result<(Vec<String>, String), String> {
    match expr {
        Expr::String(s) => Ok((Vec::new(), format!("\"{}\"", escape_c_string_literal(s)))),
        Expr::Ident(ident) if ident.suffix == Some(TypeSuffix::String) => {
            Ok((Vec::new(), c_var_name(ident, TypeSuffix::String)))
        }
        Expr::Binary { op: BinaryOp::Add, left, right } if is_string_expr(left) || is_string_expr(right) => {
            let (mut prelude, left_text) = render_string_expr(left, needs_math, temp_counter)?;
            let (right_prelude, right_text) = render_string_expr(right, needs_math, temp_counter)?;
            prelude.extend(right_prelude);
            let temp = format!("bt_s_{temp_counter}");
            *temp_counter += 1;
            prelude.push(format!("    char {temp}[{STRING_BUFFER_SIZE}];\n"));
            prelude.push(format!(
                "    snprintf({temp}, sizeof({temp}), \"%s%s\", {left_text}, {right_text});\n"
            ));
            Ok((prelude, temp))
        }
        _ => Err(
            "the minimal C backend's string expressions only support string literals, string \
             scalar variables ($), and + (concatenation) of them so far"
                .to_string(),
        ),
    }
}

/// Builds any statements that must run before the `printf` call (e.g. a
/// string concatenation's temp-buffer setup), the `printf` format string
/// itself, its positional argument expressions, and whether the statement
/// wants a trailing newline -- same rule the BASIC backend's
/// `render_print_tokens` uses: a trailing `;`/`,` suppresses it, anything
/// else (including no separator at all) gets one.
///
/// A bare string literal contributes its (escaped) text directly to the
/// format string, with no `%s`/prelude needed. Any other string-typed
/// expression (`is_string_expr`: a string variable read, or `+`
/// concatenation) goes through `render_string_expr` instead, contributing
/// a `%s` placeholder, its value in `args`, and its prelude lines (if any)
/// to the output. Everything else goes through `render_numeric_expr`,
/// contributing a `%d`/`%g` placeholder plus C expression text. Anything
/// neither function understands (a call, an array) isn't supported yet and
/// is reported as an error rather than silently mishandled.
fn render_print_tokens(
    tokens: &[PrintToken],
    needs_math: &mut bool,
    temp_counter: &mut usize,
) -> Result<(Vec<String>, String, Vec<String>, bool), String> {
    let mut prelude = Vec::new();
    let mut format = String::new();
    let mut args = Vec::new();
    let mut needs_newline = true;
    for (index, token) in tokens.iter().enumerate() {
        match token {
            PrintToken::Expr(Expr::String(s)) => {
                needs_newline = true;
                format.push_str(&escape_c_format_text(s));
            }
            PrintToken::Expr(expr) if is_string_expr(expr) => {
                let (expr_prelude, text) = render_string_expr(expr, needs_math, temp_counter)?;
                prelude.extend(expr_prelude);
                needs_newline = true;
                format.push_str("%s");
                args.push(text);
            }
            PrintToken::Expr(expr) => {
                let (text, is_float) = render_numeric_expr(expr, needs_math)?;
                needs_newline = true;
                format.push_str(if is_float { "%g" } else { "%d" });
                args.push(text);
            }
            PrintToken::Semi | PrintToken::Comma => {
                needs_newline = index != tokens.len() - 1;
            }
        }
    }
    Ok((prelude, format, args, needs_newline))
}

/// Renders a numeric expression tree as C expression text, plus whether the
/// result is floating-point (picks `%g` vs `%d` in the caller). Covers
/// literals, numeric scalar variable reads (`%`/`&`/`!`/`#` -- the C
/// identifier and float-ness come from `c_var_name`/`numeric_c_type`;
/// string variables and no-suffix variables aren't supported yet), negation,
/// and every arithmetic operator: `+`/`-`/`*`
/// (direct translations, no semantic gap between BASIC and C), `/`
/// (explicit `(double)` casts on both operands, since BASIC's `/` always
/// performs floating-point division, even between two integers, unlike
/// plain C `/` between two `int`s), `\`/`MOD` (round each operand first,
/// per real MBASIC/BASCOM, then respectively truncate or take the
/// remainder of the integer quotient), and `^` (`pow()` from `<math.h>`,
/// right-associative -- already reflected in the tree shape by the time it
/// reaches here, same as the other operators' precedence). Every one of
/// these needed its exact BASIC semantics tracked down first (see the
/// per-arm comments below) rather than assuming "it's just the C operator"
/// -- several aren't.
///
/// `needs_math` is set whenever generated code calls into `<math.h>` (so
/// far: `\`/`MOD` via `round()`, `^` via `pow()`), so the caller knows to
/// add that `#include` -- most programs won't need it.
fn render_numeric_expr(expr: &Expr, needs_math: &mut bool) -> Result<(String, bool), String> {
    match expr {
        Expr::Integer(n) => Ok((n.to_string(), false)),
        Expr::Float(f) => Ok((format!("{f:?}"), true)),
        Expr::Ident(ident) => match ident.suffix.and_then(numeric_c_type) {
            Some((_, is_float)) => Ok((c_var_name(ident, ident.suffix.unwrap()), is_float)),
            None => Err(format!(
                "`{ident}` isn't supported by the minimal C backend yet -- only numeric scalar \
                 variables (%, &, !, #) are"
            )),
        },
        Expr::Unary { op: UnaryOp::Neg, expr } => {
            let (inner, is_float) = render_numeric_expr(expr, needs_math)?;
            Ok((format!("-({inner})"), is_float))
        }
        Expr::Binary { left, op: op @ (BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul), right } => {
            let (left_text, left_float) = render_numeric_expr(left, needs_math)?;
            let (right_text, right_float) = render_numeric_expr(right, needs_math)?;
            let c_op = match op {
                BinaryOp::Add => "+",
                BinaryOp::Sub => "-",
                BinaryOp::Mul => "*",
                _ => unreachable!(),
            };
            Ok((format!("({left_text} {c_op} {right_text})"), left_float || right_float))
        }
        // Explicit `(double)` casts on both operands make this always a
        // floating-point division in C too, matching BASIC's `/` even when
        // both operands are integers (`5 / 2` is `2.5` in BASIC, but plain
        // C `/` between two `int`s would truncate to `2`).
        //
        // Division by a literal zero isn't specially detected: BASIC's `/`
        // raises a runtime "Division by zero" error, while C's `(double)x /
        // (double)0` silently produces `inf`/`nan` instead of crashing --
        // a real behavioral gap, just not a memory-safety one, and not
        // addressed here.
        Expr::Binary { left, op: BinaryOp::Div, right } => {
            let (left_text, _) = render_numeric_expr(left, needs_math)?;
            let (right_text, _) = render_numeric_expr(right, needs_math)?;
            Ok((format!("((double){left_text} / (double){right_text})"), true))
        }
        // Real MBASIC/BASCOM's `\`: each operand is rounded to the nearest
        // integer first (verified against the GW-BASIC Reference Manual --
        // see MANUAL.md's Arithmetic Operators table), *then* the quotient
        // is truncated toward zero. `round()` rounds ties away from zero,
        // which is the assumed tie-break rule here -- not independently
        // verified against real BASCOM output (no dosbox-x in this
        // environment), unlike most of this codebase's BASIC-compatibility
        // claims. C's `/` between two (rounded, cast-to-`long`) integers
        // already truncates toward zero as of C99, so no extra truncation
        // step is needed once both operands are rounded. The final
        // `(int)` cast keeps the result a plain `int` so `%d` (not `%ld`)
        // is a correct printf format for it -- passing a `long` through a
        // `%d` vararg would be a real (if often silently-tolerated) type
        // mismatch. Overflow (a rounded operand or the quotient not
        // fitting in `long`/`int`) isn't specially detected, same as `/`'s
        // division-by-zero gap above.
        Expr::Binary { left, op: BinaryOp::IntDiv, right } => {
            let (left_text, _) = render_numeric_expr(left, needs_math)?;
            let (right_text, _) = render_numeric_expr(right, needs_math)?;
            *needs_math = true;
            Ok((
                format!(
                    "((int)((long)round((double){left_text}) / (long)round((double){right_text})))"
                ),
                false,
            ))
        }
        // Real MBASIC/BASCOM's `MOD`: "the integer value that is the
        // remainder of an integer division" -- the same rounded, truncating
        // division `\` performs above (GW-BASIC Reference Manual examples:
        // `10.4 MOD 4 = 2` from `10 \ 4 = 2`; `25.68 MOD 6.99 = 5` from
        // `26 \ 7 = 3`, remainder 5). That's exactly C's `%` operator's own
        // definition since C99 (`a % b` has the same sign as `a`, matching
        // truncating division) -- no separate sign-handling logic needed,
        // just apply `%` to the same rounded, `long`-cast operands `\` uses.
        // MOD by a literal zero isn't specially detected: it's undefined
        // behavior in C (typically SIGFPE), where BASIC raises a runtime
        // "Division by zero" error instead -- not addressed here, same
        // category of gap as `/`'s and `\`'s.
        Expr::Binary { left, op: BinaryOp::Mod, right } => {
            let (left_text, _) = render_numeric_expr(left, needs_math)?;
            let (right_text, _) = render_numeric_expr(right, needs_math)?;
            *needs_math = true;
            Ok((
                format!(
                    "((int)((long)round((double){left_text}) % (long)round((double){right_text})))"
                ),
                false,
            ))
        }
        // `^` (right-associative -- already reflected in the tree shape by
        // the time it reaches here, same as `+`/`-`/`*`'s precedence, so no
        // extra handling needed for that part) maps directly to pow() from
        // <math.h>, which always returns a `double`. A whole-number result
        // (`2 ^ 8` -> `256.0`) still prints as `256` under `%g`, not
        // `256.000000`, so this doesn't look any different from an integer
        // result in the common case. Negative bases with non-integer
        // exponents (e.g. `(-8) ^ (1/3)`) produce `nan` via real-valued
        // pow(), same as they'd error in BASIC -- a behavioral gap, not
        // addressed here, same category as the other operators' noted
        // gaps above.
        Expr::Binary { left, op: BinaryOp::Pow, right } => {
            let (left_text, _) = render_numeric_expr(left, needs_math)?;
            let (right_text, _) = render_numeric_expr(right, needs_math)?;
            *needs_math = true;
            Ok((format!("pow((double){left_text}, (double){right_text})"), true))
        }
        // Real MBASIC/BASCOM's comparison operators evaluate to -1 (true)
        // or 0 (false) -- confirmed in MANUAL.md's own Comparison Operators
        // section -- not 1/0 like C's `==`/`<`/etc. `-(a == b)` gets there
        // directly: C's comparison already produces 0 or 1, and negating
        // that gives exactly 0 or -1. The result is always a plain `int`
        // (is_float = false), matching how a BASIC boolean gets used
        // (printed as an integer, fed into arithmetic or, eventually,
        // AND/OR -- see the bitwise-AND/OR project memory for why those
        // must NOT reuse C's `&&`/`||` the way this reuses `==`/`<`/etc.).
        Expr::Binary {
            left,
            op: op @ (BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge),
            right,
        } => {
            let (left_text, _) = render_numeric_expr(left, needs_math)?;
            let (right_text, _) = render_numeric_expr(right, needs_math)?;
            let c_op = match op {
                BinaryOp::Eq => "==",
                BinaryOp::Ne => "!=",
                BinaryOp::Lt => "<",
                BinaryOp::Le => "<=",
                BinaryOp::Gt => ">",
                BinaryOp::Ge => ">=",
                _ => unreachable!(),
            };
            Ok((format!("(-({left_text} {c_op} {right_text}))"), false))
        }
        // Real MBASIC/BASCOM's AND/OR/XOR are genuinely bitwise, not
        // short-circuit booleans -- see the project memory saved
        // specifically for this. Verified against the GW-BASIC Reference
        // Manual: "Logical operators work by converting their operands to
        // 16-bit, signed, two's complement integers... the given operation
        // is performed on these integers bit by bit." Same round-to-integer
        // step `\`/MOD use (the manual doesn't say "rounded" here as
        // explicitly as it does for `\`/MOD, but "converting... to
        // integers" for a float operand is assumed to mean the same
        // rounding, for consistency with the rest of this codebase's
        // float-to-int conversions -- not independently verified against
        // real BASCOM output). This is exactly why C's `&`/`|`/`^` are the
        // right translation and C's `&&`/`||` would NOT be: this operates
        // on arbitrary integer *values* (`6 XOR 3 = 5`), not just BASIC's
        // -1/0 booleans -- though on -1/0 inputs specifically, plain
        // bitwise AND/OR/XOR already reproduces BASIC's truth table exactly
        // (two's complement -1 is all-ones), so no separate boolean-vs-
        // integer branch is needed.
        Expr::Binary {
            left,
            op: op @ (BinaryOp::And | BinaryOp::Or | BinaryOp::Xor),
            right,
        } => {
            let (left_text, _) = render_numeric_expr(left, needs_math)?;
            let (right_text, _) = render_numeric_expr(right, needs_math)?;
            *needs_math = true;
            let c_op = match op {
                BinaryOp::And => "&",
                BinaryOp::Or => "|",
                BinaryOp::Xor => "^",
                _ => unreachable!(),
            };
            Ok((
                format!(
                    "((int)((long)round((double){left_text}) {c_op} (long)round((double){right_text})))"
                ),
                false,
            ))
        }
        // `NOT` is bitwise complement, not boolean negation -- `NOT 1` is
        // `-2`, not `0` (MANUAL.md's own Logical Operators section makes a
        // point of this exact example, since it surprises anyone expecting
        // C-style `!`). Same round-to-integer step as AND/OR/XOR above.
        Expr::Unary { op: UnaryOp::Not, expr } => {
            let (inner, _) = render_numeric_expr(expr, needs_math)?;
            *needs_math = true;
            Ok((format!("((int)(~(long)round((double){inner})))"), false))
        }
        _ => Err(
            "the minimal C backend only supports string/numeric literals, numeric scalar \
             variables (%, &, !, #), arithmetic, comparisons, and AND/OR/XOR/NOT on them so far \
             -- not calls, string variables, or arrays"
                .to_string(),
        ),
    }
}

/// Escapes `value` as the body of a plain C string literal (no surrounding
/// quotes) -- correct for a string used as an ordinary argument (an
/// `snprintf` source, an assignment value, a concatenation operand: see
/// `render_string_expr`). Deliberately a separate function from
/// `codegen_basic::escape_string`, not a shared one: BASIC string literals
/// have no backslash escapes at all (a literal `"` is doubled, that's the
/// entire rule), while C needs `\"`, `\\`, and control bytes escaped.
/// Reusing the BASIC escaper here would silently emit invalid/wrong C the
/// moment a string contained a backslash or an unescaped control byte.
///
/// NOT correct for embedding text directly into a `printf`-style format
/// string itself (as opposed to one of its arguments) -- that additionally
/// needs a literal `%` doubled to `%%`, which this deliberately does NOT
/// do (a string used as a plain value, e.g. `grade$ = "100%"`, must keep
/// its single `%` -- doubling it here would be a correctness bug in the
/// other direction). See `escape_c_format_text` for the format-string case.
fn escape_c_string_literal(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\x{:02x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// Everything `escape_c_string_literal` does, plus doubling a literal `%`
/// to `%%` -- for text embedded directly into a `printf`-style format
/// string (see `render_print_tokens`'s `PrintToken::Expr(Expr::String(s))`
/// case), where an unescaped `%` would otherwise be read as a format
/// specifier instead of literal text -- a correctness bug (wrong output at
/// best, mismatched varargs / crash at worst).
fn escape_c_format_text(value: &str) -> String {
    escape_c_string_literal(value).replace('%', "%%")
}

/// Same rule as `codegen_basic::ends_with_end`: the last statement that
/// isn't a comment or blank line must be `end` for the program to already
/// have emitted its own `return 0;`.
fn ends_with_end(statements: &[Statement]) -> bool {
    statements
        .iter()
        .rev()
        .find(|s| {
            !matches!(s, Statement::BlankLine | Statement::BlockComment(_))
                && !matches!(s, Statement::Raw(text) if text.trim_start().starts_with('\''))
        })
        .is_some_and(|s| matches!(s, Statement::End))
}

fn unsupported(message: &str) -> Diagnostic {
    Diagnostic::error(SourcePos::new("<target>", 1, 1), message.to_string())
}
