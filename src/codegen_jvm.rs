//! Minimal native-JVM backend.
//!
//! Deliberately narrow, mirroring `codegen_c.rs`'s own bootstrap: this only
//! understands a top-level `print` of string literals and `end`, wrapped in
//! a single class's `public static main([Ljava/lang/String;)V`. Everything
//! else (functions, other statement kinds, non-string print arguments)
//! reports a "not supported yet" diagnostic rather than panicking or
//! emitting wrong code -- a walking skeleton, not a real backend yet.
//!
//! Output is Krakatau assembly text (`.j`), not a `.class` file directly --
//! same "emit text, let an external tool own the binary format" split
//! `codegen_c.rs` has with `gcc`, just with `krak2` assembling `.j` -> `.class`
//! and any JRE's `java` running it (see `main.rs`'s eventual
//! `invoke_krak2`/`invoke_java`). `krak2` is `Storyyeller/Krakatau`'s `v2`
//! branch -- itself a Rust/Cargo project, pinned by commit since it has no
//! versioned releases (confirmed working end to end with a hand-written
//! `.j` file: `krak2 asm` + `java` before this codegen existed).
//!
//! No `.stack`/`.limit locals` bookkeeping is needed yet: straight-line code
//! with no branches has exactly one stack-map-frame-free path through the
//! method, so `.limit stack`/`.limit locals` can stay fixed, conservative
//! constants. That stops being true the moment `if`/loops arrive (see
//! GitHub issue tracker's `jvm-target` label for the itemized plan) --
//! every branch target from there on needs real frame tracking, which
//! doesn't exist here.

use crate::ast::{Expr, PrintToken, Program, Statement, Stmt};
use crate::diagnostics::{Diagnostic, SourcePos};

/// The emitted `.j`'s text and the class name it declares (`.class public
/// <name>`) -- callers that go on to assemble it (`krak2 asm` then `java
/// <name>`) need that name verbatim: the JVM launcher requires the
/// `.class` file on disk to be named `<name>.class`, so it can't just reuse
/// the input `.bcl`'s own file stem the way `codegen_c`'s output can.
pub(crate) struct GeneratedJvm {
    pub(crate) class_name: String,
    pub(crate) source: String,
}

pub(crate) fn generate(program: &Program) -> Result<GeneratedJvm, Vec<Diagnostic>> {
    if !program.functions.is_empty() {
        return Err(vec![unsupported(
            "functions/procedures are not supported by the minimal JVM backend yet",
        )]);
    }

    let mut body = String::new();
    for statement in &program.statements {
        emit_statement(statement, &mut body).map_err(|message| vec![unsupported(&message)])?;
    }
    // `Statement::End` already emits its own `return` -- only add the
    // implicit fallthrough one when the program didn't already end with an
    // explicit `end`, otherwise the method would end in two `return`
    // instructions back to back (harmless to the JVM, but not what a real
    // `end` vs. no `end` should look like in the generated text).
    if !ends_with_end(&program.statements) {
        body.push_str("    return\n");
    }

    let class_name = class_name_for(program);
    let source = format!(
        ".class public {class_name}\n.super java/lang/Object\n\n\
         .method public static main : ([Ljava/lang/String;)V\n    \
         .limit stack 2\n    .limit locals 1\n\n\
         {body}.end method\n"
    );
    Ok(GeneratedJvm { class_name, source })
}

/// Java class-name-cases BASCAL's `program <name>` declaration (BASCAL
/// identifiers are already alphanumeric-only, no underscores -- see
/// `reject_underscored_identifiers` in `lib.rs` -- so only the leading
/// letter needs adjusting to match Java's PascalCase convention; this is
/// cosmetic, not a correctness requirement, since the JVM itself accepts
/// any name here). Falls back to `Program` when the source has no `program`
/// declaration at all (it's optional in BASCAL).
fn class_name_for(program: &Program) -> String {
    let Some(decl) = &program.program_decl else {
        return "Program".to_string();
    };
    let mut chars = decl.name.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => "Program".to_string(),
    }
}

fn emit_statement(statement: &Stmt, out: &mut String) -> Result<(), String> {
    match &statement.kind {
        Statement::Print { tokens } => {
            let (text, needs_newline) = render_print_tokens(tokens)?;
            let escaped = escape_jvm_string(&text);
            out.push_str("    getstatic java/lang/System/out Ljava/io/PrintStream;\n");
            out.push_str(&format!("    ldc \"{escaped}\"\n"));
            let method = if needs_newline { "println" } else { "print" };
            out.push_str(&format!(
                "    invokevirtual java/io/PrintStream/{method} (Ljava/lang/String;)V\n"
            ));
            Ok(())
        }
        Statement::End => {
            out.push_str("    return\n");
            Ok(())
        }
        Statement::BlankLine => {
            out.push('\n');
            Ok(())
        }
        Statement::BlockComment(lines) => {
            for line in lines {
                out.push_str(&format!("    ; {line}\n"));
            }
            Ok(())
        }
        // Same carve-out as `codegen_c.rs`'s bootstrap: a `'`/`//`-style
        // single-line comment always parses to `Statement::Raw("' <text>")`
        // (see `parser.rs`) -- genuine raw BASIC passthrough would land here
        // too, but with no leading `'`, so only the comment shape is safe to
        // translate.
        Statement::Raw(text) if text.trim_start().starts_with('\'') => {
            let comment = text.trim_start().trim_start_matches('\'').trim_start();
            out.push_str(&format!("    ; {comment}\n"));
            Ok(())
        }
        other => Err(format!(
            "{other:?} is not supported by the minimal JVM backend yet -- only `print` of \
             string literals and `end` are implemented so far"
        )),
    }
}

/// Concatenates every string-literal token's text and reports whether the
/// statement wants a trailing newline -- same rule `codegen_c.rs`'s own
/// `render_print_tokens` uses: a trailing `;`/`,` suppresses it, anything
/// else (including no separator at all) gets one.
fn render_print_tokens(tokens: &[PrintToken]) -> Result<(String, bool), String> {
    let mut text = String::new();
    let mut needs_newline = true;
    for (index, token) in tokens.iter().enumerate() {
        match token {
            PrintToken::Expr(Expr::String(s)) => {
                needs_newline = true;
                text.push_str(s);
            }
            PrintToken::Expr(_) => {
                return Err(
                    "the minimal JVM backend's `print` only supports string literals so far"
                        .to_string(),
                )
            }
            PrintToken::Semi | PrintToken::Comma => {
                needs_newline = index != tokens.len() - 1;
            }
        }
    }
    Ok((text, needs_newline))
}

/// Krakatau assembly string-literal escaping -- deliberately separate from
/// both `codegen_basic::escape_string` (BASIC has no backslash escapes) and
/// `codegen_c`'s `escape_c_string` (different escape set/target): a `.j`
/// string constant follows Java's own escaping rules for `"` and `\`, plus
/// `\n` for the one control byte BASCAL string literals can't otherwise
/// contain unescaped.
fn escape_jvm_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            _ => out.push(ch),
        }
    }
    out
}

/// Same rule as `codegen_c.rs`'s own `ends_with_end`: walks back past
/// trailing blank lines/block comments (but not a trailing `'`-comment
/// `Raw` line, which -- like those -- shouldn't suppress the synthesized
/// fallthrough `return`) to find whether the program's last real statement
/// was an explicit `end`.
fn ends_with_end(statements: &[Stmt]) -> bool {
    statements
        .iter()
        .rev()
        .find(|s| {
            !matches!(&s.kind, Statement::BlankLine | Statement::BlockComment(_))
                && !matches!(&s.kind, Statement::Raw(text) if text.trim_start().starts_with('\''))
        })
        .is_some_and(|s| matches!(&s.kind, Statement::End))
}

fn unsupported(message: &str) -> Diagnostic {
    Diagnostic::error(SourcePos::new("<target>", 1, 1), message.to_string())
}
