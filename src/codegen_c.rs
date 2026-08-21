//! Minimal native-C backend.
//!
//! Deliberately narrow: this only understands a top-level `print` of string
//! and numeric literals, and `end`, wrapped in `int main(void) { ... }`.
//! Everything else (functions, other statement kinds, variables) reports a
//! "not supported yet" diagnostic rather than panicking or emitting wrong
//! code -- this is a walking skeleton to prove the CLI/dispatch plumbing
//! (`Target::C`, `--target c`, `invoke_gcc`) end-to-end, not a real backend.
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

use crate::ast::{Expr, PrintToken, Program, Statement, UnaryOp};
use crate::diagnostics::{Diagnostic, SourcePos};

pub(crate) fn generate(program: &Program) -> Result<String, Vec<Diagnostic>> {
    if !program.functions.is_empty() {
        return Err(vec![unsupported(
            "functions/procedures are not supported by the minimal C backend yet",
        )]);
    }

    let mut body = String::new();
    for statement in &program.statements {
        emit_statement(statement, &mut body).map_err(|message| vec![unsupported(&message)])?;
    }
    // `Statement::End` already emits its own `return 0;` -- only add the
    // implicit fallthrough one when the program didn't already end with an
    // explicit `end` (comments/blank lines don't count), otherwise `main`
    // would end in two `return 0;` statements back to back.
    if !ends_with_end(&program.statements) {
        body.push_str("    return 0;\n");
    }

    Ok(format!("#include <stdio.h>\n\nint main(void) {{\n{body}}}\n"))
}

fn emit_statement(statement: &Statement, out: &mut String) -> Result<(), String> {
    match statement {
        Statement::Print { tokens } => {
            let (mut format, args, needs_newline) = render_print_tokens(tokens)?;
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
            "{other:?} is not supported by the minimal C backend yet -- only `print` of string \
             literals and `end` are implemented so far"
        )),
    }
}

/// Builds a `printf` format string plus its positional argument
/// expressions, and reports whether the statement wants a trailing newline
/// -- same rule the BASIC backend's `render_print_tokens` uses: a trailing
/// `;`/`,` suppresses it, anything else (including no separator at all)
/// gets one.
///
/// String literals contribute their (escaped) text directly to the format
/// string; integer/float literals contribute a `%d`/`%g` placeholder plus a
/// C literal in `args`. Any other expression (a variable, a call, an
/// operator) isn't supported yet -- there's no C-side variable/expression
/// codegen at all so far -- and is reported as an error rather than
/// silently mishandled.
fn render_print_tokens(tokens: &[PrintToken]) -> Result<(String, Vec<String>, bool), String> {
    let mut format = String::new();
    let mut args = Vec::new();
    let mut needs_newline = true;
    for (index, token) in tokens.iter().enumerate() {
        match token {
            PrintToken::Expr(Expr::String(s)) => {
                needs_newline = true;
                format.push_str(&escape_c_string(s));
            }
            PrintToken::Expr(Expr::Integer(n)) => {
                needs_newline = true;
                format.push_str("%d");
                args.push(n.to_string());
            }
            PrintToken::Expr(Expr::Float(f)) => {
                needs_newline = true;
                format.push_str("%g");
                args.push(format!("{f:?}"));
            }
            // `-5`/`-1.25` parse as `Unary { op: Neg, expr: Integer/Float
            // }`, not a literal `Expr::Integer(-5)` -- constant-fold just
            // this one shape rather than reporting a negative literal, the
            // single most common case, as unsupported.
            PrintToken::Expr(Expr::Unary { op: UnaryOp::Neg, expr }) => match expr.as_ref() {
                Expr::Integer(n) => {
                    needs_newline = true;
                    format.push_str("%d");
                    args.push((-n).to_string());
                }
                Expr::Float(f) => {
                    needs_newline = true;
                    format.push_str("%g");
                    args.push(format!("{:?}", -f));
                }
                _ => {
                    return Err(
                        "the minimal C backend's `print` only supports string and numeric \
                         literals so far -- not variables, calls, or operators"
                            .to_string(),
                    )
                }
            },
            PrintToken::Expr(_) => {
                return Err(
                    "the minimal C backend's `print` only supports string and numeric literals \
                     so far -- not variables, calls, or operators"
                        .to_string(),
                )
            }
            PrintToken::Semi | PrintToken::Comma => {
                needs_newline = index != tokens.len() - 1;
            }
        }
    }
    Ok((format, args, needs_newline))
}

/// C string-literal escaping -- deliberately a separate function from
/// `codegen_basic::escape_string`, not a shared one: BASIC string literals
/// have no backslash escapes at all (a literal `"` is doubled, that's the
/// entire rule), while C needs `\"`, `\\`, and control bytes escaped.
/// Reusing the BASIC escaper here would silently emit invalid/wrong C the
/// moment a string contained a backslash or an unescaped control byte.
///
/// Every use of this ends up inside a `printf` format-string argument (see
/// `render_print_tokens`), so a literal `%` is escaped to `%%` too --
/// without that, a BASIC string containing `%` would be read by `printf` as
/// a format specifier instead of literal text, which is a correctness bug
/// (wrong output at best, mismatched varargs / crash at worst).
fn escape_c_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '%' => out.push_str("%%"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\x{:02x}", c as u32)),
            c => out.push(c),
        }
    }
    out
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
