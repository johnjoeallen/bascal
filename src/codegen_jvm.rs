//! Minimal native-JVM backend.
//!
//! Deliberately narrow, mirroring `codegen_c.rs`'s own bootstrap: this
//! understands a top-level `print` of string or numeric literals and their
//! arithmetic expressions, scalar variables/constants, plus `end`, wrapped
//! in a single class's `public static main([Ljava/lang/String;)V`. Everything
//! else (functions, control flow, and unsupported string operations) reports
//! a "not supported yet" diagnostic rather than panicking or emitting wrong
//! code -- a walking skeleton, not a real backend yet.
//!
//! Output is Krakatau assembly text (`.j`); the command-line driver also has a
//! small internal class-file writer for return-only programs, while general
//! output is assembled `.j` -> `.class` via the `krakatau2` crate (linked
//! directly into `bcc`, not shelled out to a separate binary -- see
//! `Cargo.toml`'s own comment and `main.rs`'s `invoke_krak2`), and run by any
//! JRE's `java`. `krakatau2` is `Storyyeller/Krakatau`'s `v2` branch -- itself
//! a Rust/Cargo project, pinned by commit since it has no versioned releases
//! (confirmed working end to end with a hand-written `.j` file: `krak2 asm`
//! + `java` before this codegen existed).
//!
//! The generated classes deliberately use class-file version 50.  That lets
//! the JVM's legacy verifier infer frames for the supported `if` branches,
//! instead of requiring StackMapTable emission before this backend has a full
//! control-flow frame analyser. Scalar local slots are allocated up front and
//! `.limit locals` is computed from them.
//!
//! This module doc's opening two paragraphs describe the original bootstrap
//! and are stale in the details (functions/procedures, `if`/`for`/`while`/
//! `do`, `goto`, arrays, structured `try`/`catch`/`finally`, and now random-
//! access record I/O all work -- see `tests/jvm_conformance.rs`) but
//! accurate in spirit: still narrower than `codegen_c.rs`/`codegen_basic.rs`,
//! and every genuinely unsupported construct still reports a clear "not
//! supported yet" diagnostic rather than panicking or emitting wrong code.
//!
//! Random-access record I/O (`OPEN ... FOR RANDOM`, `FIELD`, `GET`, `PUT`,
//! `LSET`/`RSET`, `MKI$`/`MKL$`/`MKS$`/`MKD$`/`CVI`/`CVL`/`CVS`/`CVD`) is
//! implemented via three parallel static
//! arrays (`bccFiles: RandomAccessFile[]`, `bccBufs: byte[][]`,
//! `bccRecLen: int[]`, each sized to `JVM_MAX_CHANNELS`, initialized once at
//! the top of `main` -- see `emit_file_io_initializers`), indexed at runtime
//! by `channel - 1`. `FIELD`'s channel number and every field width must be
//! literal (`JvmFieldVar`/`collect_field_vars`), matching every realistic
//! caller (including everything `records::lower` itself synthesizes from
//! the record/file DSL); `GET`/`PUT`'s own record number may be any numeric
//! expression. A `FIELD`-declared variable is never an ordinary local/
//! static string slot -- reading one decodes its byte range out of its
//! channel's shared buffer, and only `LSET`/`RSET` (not a plain assignment)
//! may write one, encoding back into that same buffer (see
//! `emit_field_var_load`/`emit_lset`/`emit_rset`). Every packed/decoded byte
//! goes through ISO-8859-1, which maps bytes 0-255 to chars 0-255 one-to-one
//! -- unlike real UTF-8/platform-default decoding, this can't corrupt a
//! packed numeric field's raw bytes. `MKI$`/`CVI` (16-bit int) and `MKL$`/
//! `CVL` (32-bit int) are little-endian, matching real MBASIC/BASCOM's own
//! on-disk layout exactly; `MKS$`/`CVS` (32-bit float) and `MKD$`/`CVD`
//! (64-bit double) use plain IEEE 754 instead of real BASIC's Microsoft
//! Binary Format -- the same real, documented divergence `codegen_c.rs`'s
//! own `bcc_mks`/`bcc_mkd`/`bcc_cvs`/`bcc_cvd` accept (see their doc
//! comment there): a `single`/`double` record field this backend writes
//! isn't binary-compatible with one real BASCOM wrote, unlike an `int16`/
//! `int32`/`string(N)` field, which is (see `tests/jvm_conformance.rs`'s
//! `jvm_random_and_record_files_tutorial_runs_when_available`, which
//! exercises the full family). Sequential file I/O (`OPEN ... FOR INPUT`/
//! `OUTPUT`/`APPEND`) and `MID$(...) = ...` statement-form assignment still
//! aren't implemented at all.
//!
//! Bare `INKEY$` is implemented via `stty` (see `emit_run_command_
//! inheriting_io`'s own doc comment for why a real `ProcessBuilder`,
//! `inheritIO()`'d, rather than JNI/a native helper): `emit_inkey_setup`
//! puts the terminal into no-canonical/no-echo/non-blocking mode once at
//! program start, left in effect until `emit_inkey_restore` undoes it at
//! every exit point; each `INKEY$` read is then just a
//! `System.in.available()` check plus an optional single-byte `read()`
//! (see `emit_string_expr`'s own `"inkey"` arm). Deliberately `stty
//! -icanon -echo`, not the `raw` canned mode (which also clears `opost`,
//! breaking every later `PRINT`'s `\n`->`\r\n` translation for the rest of
//! the run -- see `emit_inkey_setup`'s own doc comment for the real bug
//! this caused). Verified by hand against a real pseudo-terminal: a single
//! byte with no trailing newline was picked up immediately, with no local
//! echo.
//!
//! Interactive `INPUT ["prompt";] var` is implemented (one plain-identifier
//! target only -- no comma-separated multi-variable form): a shared
//! `bccStdin: BufferedReader` (see `emit_input_initializer`, same
//! initialize-in-`main` convention as the file-I/O arrays) backs
//! `emit_input`, which prints `prompt` + BASIC's own `"? "` suffix, reads
//! one line, and parses it for a numeric target or stores it verbatim for a
//! string one.
//!
//! `collect_scalar_declarations`/`collect_array_declarations`/
//! `collect_global_names`/`collect_labels` (plus this file's own
//! `collect_field_vars`/`program_uses_random_open`/`program_uses_input`) all
//! now recurse into `select case` clause bodies -- a real, pre-existing gap
//! (any variable/array/label/`global`/`FIELD`/`OPEN`/`INPUT` that only
//! appeared inside a `case` clause silently failed to register) that
//! `examples/card_catalog/card_catalog.bcl`'s own menu dispatch (every
//! branch is a `case`) surfaced immediately once file I/O and `INPUT`
//! stopped being the blocker.
//!
//! `examples/card_catalog/card_catalog.bcl` -- the flagship record/file DSL
//! and procedures example -- compiles and runs correctly end to end under
//! `--target jvm` as of this paragraph, verified interactively (add an
//! item, list it back, confirming the random-access write/read round-trip)
//! with real `java` and `krak2`.
//!
//! Scalar `byref` parameters are emulated (the JVM has no scalar reference
//! semantics the way C has pointers): each `byref` scalar argument is
//! wrapped in a fresh single-element array at the call site (see
//! `emit_call_arguments`), passed as that array (`FunctionSig::
//! byref_scalar_positions`/`descriptor_params` widen its descriptor slot to
//! `[<type>` accordingly), unwrapped into an ordinary "working" local at
//! function entry, written back into the array at every exit point
//! (`JvmContext::emit_byref_writebacks`, called from both the procedure/
//! function fallthrough and every explicit `return`), and unwrapped again
//! at the call site once the call returns (`emit_byref_call_writebacks`).
//! The synthetic "working" locals and per-function scratch-array slots are
//! allocated strictly *after* every real parameter slot (`for_function`'s
//! two-pass allocation) -- interleaving them with the true parameter slots
//! corrupts the JVM's own parameter numbering for every parameter after the
//! first `byref` one and fails verification (`VerifyError: Bad local
//! variable type`), the bug this two-pass split fixes.
//!
//! `tutorial/inventory.bcl` -- the most feature-complete tutorial, using
//! random-access record I/O, `INKEY$`, `INPUT`, `byref` scalars, and
//! `try`/`catch` together -- compiles and runs correctly end to end under
//! `--target jvm`, verified interactively against a real pseudo-terminal
//! (menu navigation via `INKEY$`, a part lookup via `INPUT`, and the full
//! record listing, all against a freshly-initialized data file). Getting
//! there surfaced two more real, narrow bugs beyond the ones above: `GET`
//! on a freshly-created (empty) random-access file used to throw
//! `EOFException` (`RandomAccessFile.readFully()` requires a full read;
//! `emit_get_or_put` now uses plain `read()` and discards the count,
//! matching C's `fread`-based leniency -- see
//! `jvm_get_on_a_fresh_empty_file_does_not_throw_when_available`), and
//! calling a non-`void` function as a bare statement (discarding its
//! result, the way `inventory.bcl`'s `readKey$()` consumes a keystroke)
//! used to be rejected outright -- the statement-call path now accepts any
//! function, popping the unwanted result (see
//! `jvm_function_call_as_bare_statement_discards_its_result_when_available`).
//! A third, more fundamental bug also surfaced here: once `INKEY$` has put
//! the terminal into raw, non-blocking (`min 0 time 0`) mode, a later
//! blocking `INPUT` needs an ordinary blocking terminal to read from --
//! `emit_input` now brackets its `readLine()` with `stty sane` /
//! `emit_inkey_setup`'s raw mode again (see `emit_input`'s own doc comment
//! and `jvm_input_after_inkey_reads_the_typed_value_when_available`).
//!
//! `STR$`/bare-numeric `PRINT` of a `single`/`double` value (every
//! `single`/`double` is a JVM `double` -- see `TypeSuffix::Single |
//! TypeSuffix::Double`) is routed through `bccStr` (`emit_double_str_helper`)
//! rather than Java's own `String.valueOf(double)`, which always prints
//! full round-trip precision: a real BASIC `single` unpacked from its
//! 32-bit on-disk form and widened to `double` made that widening's own
//! rounding noise visible verbatim (`0.03` printed as
//! `0.029999999329447746` -- `inventory.bcl`'s own `price!` field, found
//! interactively). `bccStr` rounds to 6 significant digits and drops
//! trailing zeros, matching `codegen_c.rs`'s own `bcc_strd` (`"% g"`
//! `snprintf` formatting). The helper is only emitted when something
//! actually calls it (checked by scanning the already-emitted method text
//! for its own call site), so a program with no `single`/`double` anywhere
//! (like `tutorial/hello.bcl`, whose checked-in `hello.j` fixture stays
//! byte-for-byte unchanged) carries no dead bytecode for it.

use std::cell::Cell;
use std::collections::{BTreeMap, HashMap, HashSet};

use crate::ast::{
    BasicIdent, BinaryOp, CaseValue, Expr, FunctionDef, OpenMode, ParamMode, PrintToken, Program,
    Statement, Stmt, TypeSuffix, UnaryOp,
};
use crate::diagnostics::{Diagnostic, SourcePos};

pub(crate) fn generate(program: &Program) -> Result<String, Vec<Diagnostic>> {
    let class_name = class_name_for(program);
    let functions = function_table(&program.functions);
    let context = JvmContext::build(program, functions.clone(), class_name.clone())?;
    let mut body = String::new();
    context.emit_initializers(&mut body);
    emit_array_initializers(&context, &mut body).map_err(|message| vec![unsupported(&message)])?;
    emit_file_io_initializers(&context, &mut body);
    emit_input_initializer(&context, &mut body);
    emit_inkey_setup(&context, &mut body);
    let mut emitter = JvmEmitter {
        context: &context,
        next_label: 0,
        loop_exits: Vec::new(),
        return_type: None,
        labels: collect_labels(&program.statements),
        exception_handlers: Vec::new(),
    };
    for statement in &program.statements {
        emitter
            .emit_statement(statement, &mut body)
            .map_err(|message| vec![unsupported(&message)])?;
    }
    // `Statement::End` already emits its own `return` -- only add the
    // implicit fallthrough one when the program didn't already end with an
    // explicit `end`, otherwise the method would end in two `return`
    // instructions back to back (harmless to the JVM, but not what a real
    // `end` vs. no `end` should look like in the generated text).
    if !ends_with_end(&program.statements) {
        emit_inkey_restore(&context, &mut body);
        body.push_str("    return\n");
    }
    let handlers = emitter
        .exception_handlers
        .iter()
        .map(|handler| {
            format!(
                "    .catch java/lang/RuntimeException from {} to {} using {}\n",
                handler.start, handler.end, handler.handler
            )
        })
        .collect::<String>();

    let mut methods = String::new();
    for function in &program.functions {
        methods.push_str(
            &emit_function(function, &context).map_err(|message| vec![unsupported(&message)])?,
        );
    }
    if program.functions.iter().any(|function| {
        function
            .params
            .iter()
            .any(|param| param.axes.is_some() && param.mode != ParamMode::ByRef)
    }) {
        methods.push_str(&emit_array_copy_helper(&class_name));
    }
    // Only pull in `bccStr` (see its own doc comment) when something in the
    // program actually calls it -- checking the already-emitted `body`/
    // `methods` text for the exact call site `STR$`/bare-numeric `PRINT`
    // emit is simpler and more reliably exhaustive than re-deriving "does
    // any expression anywhere evaluate to a `double`" from the AST a second
    // time, and keeps a program with no `single`/`double` value anywhere
    // (like `tutorial/hello.bcl`) free of dead helper bytecode.
    let bcc_str_call = format!("invokestatic {class_name}/bccStr (D)Ljava/lang/String;\n");
    if body.contains(&bcc_str_call) || methods.contains(&bcc_str_call) {
        methods.push_str(&emit_double_str_helper());
    }
    Ok(format!(
        ".version 50 0\n.class public {class_name}\n.super java/lang/Object\n\n{}{methods}\
         .method public static main : ([Ljava/lang/String;)V\n    \
         .limit stack 16\n    .limit locals {}\n\n\
         {body}{handlers}.end method\n",
        emit_fields(&context),
        context.local_count(),
    ))
}

/// Java class-name-cases BASCAL's `program <name>` declaration (BASCAL
/// identifiers are already alphanumeric-only, no underscores -- see
/// `reject_underscored_identifiers` in `lib.rs` -- so only the leading
/// letter needs adjusting to match Java's PascalCase convention; this is
/// cosmetic, not a correctness requirement, since the JVM itself accepts
/// any name here). Falls back to `Program` when the source has no `program`
/// declaration at all (it's optional in BASCAL). `main.rs`'s `invoke_krak2`
/// recovers this same name back out of the emitted `.class public <name>`
/// line rather than calling this directly -- `codegen_jvm` is a private
/// module, not part of `bcc`'s public API surface `main.rs` (a separate
/// crate) can reach.
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

fn emit_fields(context: &JvmContext) -> String {
    let mut fields = context
        .variables
        .values()
        .filter(|v| v.is_static)
        .map(|v| {
            format!(
                ".field public static {} {}\n",
                v.field_name(),
                v.descriptor()
            )
        })
        .collect::<String>();
    for (index, shape) in context.arrays.values().enumerate() {
        fields.push_str(&format!(
            ".field public static a{} {}\n",
            index,
            array_descriptor(shape)
        ));
    }
    if context.needs_file_io {
        fields.push_str(
            ".field public static bccFiles [Ljava/io/RandomAccessFile;\n\
             .field public static bccBufs [[B\n\
             .field public static bccRecLen [I\n",
        );
    }
    if context.needs_input {
        fields.push_str(".field public static bccStdin Ljava/io/BufferedReader;\n");
    }
    fields
}

/// Allocates `bccFiles`/`bccBufs`/`bccRecLen` (see `JVM_MAX_CHANNELS`'s own
/// doc comment) once, at the very top of `main` -- this class has no real
/// `<clinit>` (every static field here is instead initialized by code
/// emitted directly into `main`, see `JvmContext::emit_initializers`), so
/// this follows the same convention. Safe to run before any of the user's
/// own statements: nothing else in the class touches these three fields
/// until some `OPEN`/`FIELD`/`GET`/`PUT`/`CLOSE` actually runs, and every
/// entry point into user code (`main`, and every `function`/`procedure` it
/// transitively calls) is reachable only after this.
fn emit_file_io_initializers(context: &JvmContext, out: &mut String) {
    if !context.needs_file_io {
        return;
    }
    out.push_str(&format!(
        "    ldc {JVM_MAX_CHANNELS}\n    anewarray java/io/RandomAccessFile\n    putstatic {}/bccFiles [Ljava/io/RandomAccessFile;\n",
        context.class_name
    ));
    out.push_str(&format!(
        "    ldc {JVM_MAX_CHANNELS}\n    anewarray [B\n    putstatic {}/bccBufs [[B\n",
        context.class_name
    ));
    out.push_str(&format!(
        "    ldc {JVM_MAX_CHANNELS}\n    newarray int\n    putstatic {}/bccRecLen [I\n",
        context.class_name
    ));
}

/// Allocates the shared `bccStdin` reader once, at the very top of `main`
/// -- same convention/reasoning as `emit_file_io_initializers`.
fn emit_input_initializer(context: &JvmContext, out: &mut String) {
    if !context.needs_input {
        return;
    }
    out.push_str(&format!(
        "    new java/io/BufferedReader\n    dup\n    new java/io/InputStreamReader\n    dup\n    \
         getstatic java/lang/System/in Ljava/io/InputStream;\n    \
         invokespecial java/io/InputStreamReader/<init> (Ljava/io/InputStream;)V\n    \
         invokespecial java/io/BufferedReader/<init> (Ljava/io/Reader;)V\n    \
         putstatic {}/bccStdin Ljava/io/BufferedReader;\n",
        context.class_name
    ));
}

/// `INPUT ["prompt";] var` -- prints `prompt` followed by real BASIC's own
/// `"? "` suffix (suppressed only by `INPUT "prompt", var`'s comma form,
/// which isn't implemented -- nothing exercising this backend's INPUT yet
/// uses it), reads one line from the shared `bccStdin`, and stores it into
/// `var`: verbatim for a string target, or through `Integer.parseInt`/
/// `Long.parseLong`/`Double.parseDouble` (on the trimmed line, so leading/
/// trailing whitespace in the typed response doesn't fail the parse) for a
/// numeric one. Only a single plain-identifier target is supported --
/// real BASIC's comma-separated multi-variable `INPUT` isn't implemented,
/// since nothing exercising this backend's INPUT yet uses it.
///
/// When the program also uses `INKEY$`, the terminal is sitting in
/// `emit_inkey_setup`'s raw, non-blocking (`min 0 time 0`) mode for the
/// entire run. A blocking `BufferedReader.readLine()` under that mode
/// doesn't block for a line the way it would on a normal terminal --
/// empirically, Java's stream decoder treats the immediate zero-byte reads
/// `min 0 time 0` produces as EOF and `readLine()` returns `null` (crashing
/// the `.trim()`/`parseInt` call that follows) even though the user is
/// mid-keystroke. So `INPUT` brackets its read with `stty sane` /
/// `emit_inkey_setup`'s raw mode again, giving `readLine()` an ordinary
/// blocking, canonical-mode terminal to read from and leaving INKEY$'s
/// polling mode restored afterward.
fn emit_input(
    prompt: Option<&str>,
    vars: &[Expr],
    out: &mut String,
    context: &JvmContext,
) -> Result<(), String> {
    let [Expr::Ident(name)] = vars else {
        return Err(
            "INPUT with other than exactly one plain variable isn't supported under --target jvm"
                .to_string(),
        );
    };
    let variable = context.variable(name)?;
    if let Some(prompt) = prompt {
        // `.print()`, not `.println()` -- real BASIC's own trailing `"? "`
        // (appended below) stays on the same line as whatever's typed
        // next. `System.out`'s autoflush only triggers on a `\n` byte or a
        // `println` call, so without the explicit flush here the prompt
        // would sit invisible in the buffer until something else flushed
        // it -- see `emit_print_tokens`'s own doc comment on this same gap
        // (and its fix) for `print ...;`/`print ...,`.
        out.push_str(&format!(
            "    getstatic java/lang/System/out Ljava/io/PrintStream;\n    ldc \"{}? \"\n    \
             invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V\n    \
             getstatic java/lang/System/out Ljava/io/PrintStream;\n    \
             invokevirtual java/io/PrintStream/flush ()V\n",
            escape_jvm_string(prompt)
        ));
    }
    emit_inkey_restore(context, out);
    out.push_str(&format!(
        "    getstatic {}/bccStdin Ljava/io/BufferedReader;\n    \
         invokevirtual java/io/BufferedReader/readLine ()Ljava/lang/String;\n",
        context.class_name
    ));
    emit_inkey_setup(context, out);
    match variable.ty {
        JvmType::String => {}
        JvmType::Numeric(ty) => {
            out.push_str("    invokevirtual java/lang/String/trim ()Ljava/lang/String;\n");
            out.push_str(match ty {
                NumericType::Int => {
                    "    invokestatic java/lang/Integer/parseInt (Ljava/lang/String;)I\n"
                }
                NumericType::Long => {
                    "    invokestatic java/lang/Long/parseLong (Ljava/lang/String;)J\n"
                }
                NumericType::Double => {
                    "    invokestatic java/lang/Double/parseDouble (Ljava/lang/String;)D\n"
                }
            });
        }
    }
    emit_store(variable, out, context);
    Ok(())
}

/// Runs an external command with its own stdin/stdout/stderr wired straight
/// to this process's (`ProcessBuilder.inheritIO()`) -- essential for `stty`,
/// which reads/writes *its own* terminal's settings, not a value passed as
/// an argument: only `inheritIO()` connects it to the actual controlling
/// terminal `System.in` is reading from, the same way a shell's `stty raw
/// -echo < /dev/tty` does. The exit code is discarded (`waitFor()`, then
/// `pop`) -- `stty` legitimately fails when stdin isn't a real terminal
/// (piped/redirected input under a test harness, say), and that's not a
/// program error: `INKEY$` still works via a plain, blocking-free
/// `available()` check either way, just without the "no Enter needed, no
/// echo" behavior a real terminal gives it.
fn emit_run_command_inheriting_io(args: &[&str], out: &mut String) {
    out.push_str("    new java/lang/ProcessBuilder\n    dup\n");
    out.push_str(&format!(
        "    ldc {}\n    anewarray java/lang/String\n",
        args.len()
    ));
    for (index, arg) in args.iter().enumerate() {
        out.push_str(&format!(
            "    dup\n    ldc {index}\n    ldc \"{}\"\n    aastore\n",
            escape_jvm_string(arg)
        ));
    }
    out.push_str(
        "    invokespecial java/lang/ProcessBuilder/<init> ([Ljava/lang/String;)V\n    \
         invokevirtual java/lang/ProcessBuilder/inheritIO ()Ljava/lang/ProcessBuilder;\n    \
         invokevirtual java/lang/ProcessBuilder/start ()Ljava/lang/Process;\n    \
         invokevirtual java/lang/Process/waitFor ()I\n    pop\n",
    );
}

/// Puts the terminal into no-canonical/no-echo/non-blocking mode once, at
/// the very top of `main` (see `emit_input_initializer`'s own doc comment
/// on the "initialize once in `main`, no real `<clinit>`" convention this
/// backend uses everywhere) -- `min 0 time 0` is what makes a read return
/// immediately with zero bytes when no key is waiting, instead of blocking
/// until one arrives, which is what makes `INKEY$`'s own `available()`
/// check (see `emit_string_expr`'s own `"inkey"` arm) meaningful at all.
/// Left in effect for the rest of the run (unlike `codegen_c.rs`'s
/// `bcc_inkey`, which toggles raw mode on *every single call* and restores
/// it immediately after -- fine for a `read()` syscall, but spawning a whole
/// `stty` process per keystroke poll here would be prohibitively slow) --
/// see `emit_inkey_restore` for where it gets undone.
///
/// Deliberately `-icanon -echo`, not the `raw` canned mode this used to
/// use: `stty raw` also clears `opost` (output post-processing), which is
/// what a POSIX tty driver uses to translate a bare `\n` into `\r\n` on the
/// wire. With `opost` off for the rest of the run, every later `PRINT`'s
/// `\n` stops returning the cursor to column 1, so each subsequent line
/// starts wherever the previous one left off -- a real, previously
/// misdiagnosed bug (`tutorial/inventory.bcl`'s `listAll` looked like it
/// was drifting rightward down the page; it was actually never returning
/// to column 1 at all). `-icanon -echo` disables line-buffering and echo,
/// the two properties `INKEY$` actually needs, without touching `opost`.
fn emit_inkey_setup(context: &JvmContext, out: &mut String) {
    if !context.needs_inkey {
        return;
    }
    emit_run_command_inheriting_io(&["stty", "-icanon", "-echo", "min", "0", "time", "0"], out);
}

/// Restores normal terminal behavior -- the counterpart to
/// `emit_inkey_setup`, run at every place the program can actually exit:
/// `Statement::End`'s own `return`, `Statement::Stop`/`Statement::System`'s
/// `System.exit(0)`, and `generate()`'s own implicit fallthrough `return`
/// when the program has no explicit `end`. Not run on an uncaught exception
/// -- a real gap (the terminal is left raw if the program crashes instead
/// of exiting normally), accepted for the same reason this backend accepts
/// `codegen_c.rs`'s narrower gaps elsewhere: nothing exercising `INKEY$` yet
/// needs it, and a real fix (a JVM shutdown hook) needs either a lambda/
/// `invokedynamic` or a whole second helper class overriding `Thread.run`,
/// both a materially bigger lift than hand-written straight-line bytecode.
fn emit_inkey_restore(context: &JvmContext, out: &mut String) {
    if !context.needs_inkey {
        return;
    }
    emit_run_command_inheriting_io(&["stty", "sane"], out);
}

/// Deep-clones a nested integer array.  BASCAL permits at most eight axes;
/// the caller checks that limit before invoking this helper.  Arrays of rank
/// two or greater are `Object[]` at each outer level, so a shallow clone plus
/// recursive replacement of every child preserves the concrete array class.
fn emit_array_copy_helper(class_name: &str) -> String {
    format!(".method private static bccCopyArray : (Ljava/lang/Object;II)Ljava/lang/Object;\n    .limit stack 5\n    .limit locals 5\n\n    iload 1\n    iconst_1\n    if_icmpne L_copy_nested\n    iload 2\n    tableswitch 0\n        L_copy_int\n        L_copy_long\n        L_copy_double\n        L_copy_object\nL_copy_int:\n    aload 0\n    checkcast [I\n    invokevirtual [I/clone ()Ljava/lang/Object;\n    areturn\nL_copy_long:\n    aload 0\n    checkcast [J\n    invokevirtual [J/clone ()Ljava/lang/Object;\n    areturn\nL_copy_double:\n    aload 0\n    checkcast [D\n    invokevirtual [D/clone ()Ljava/lang/Object;\n    areturn\nL_copy_object:\n    aload 0\n    checkcast [Ljava/lang/Object;\n    invokevirtual [Ljava/lang/Object;/clone ()Ljava/lang/Object;\n    areturn\nL_copy_nested:\n    aload 0\n    checkcast [Ljava/lang/Object;\n    invokevirtual [Ljava/lang/Object;/clone ()Ljava/lang/Object;\n    checkcast [Ljava/lang/Object;\n    astore 3\n    iconst_0\n    istore 4\nL_copy_loop:\n    iload 4\n    aload 3\n    arraylength\n    if_icmpge L_copy_done\n    aload 3\n    iload 4\n    aload 3\n    iload 4\n    aaload\n    iload 1\n    iconst_1\n    isub\n    iload 2\n    invokestatic {class_name}/bccCopyArray (Ljava/lang/Object;II)Ljava/lang/Object;\n    aastore\n    iinc 4 1\n    goto L_copy_loop\nL_copy_done:\n    aload 3\n    areturn\n.end method\n\n")
}

/// Formats a `double` the way `STR$`/bare-numeric `PRINT` need for a
/// BASCAL `single`/`double` value, instead of Java's own
/// `String.valueOf(double)`/`Double.toString`, which always prints full
/// round-trip precision (17 significant digits) -- for a real BASIC
/// `single` unpacked from its 32-bit on-disk representation and widened to
/// `double` (`CVS`, or any `!`-suffixed variable, which this backend always
/// represents as a JVM `double` -- see `TypeSuffix::Single |
/// TypeSuffix::Double => JvmType::Numeric(NumericType::Double)`), that
/// widening's own rounding noise becomes visible verbatim: `0.03` printed
/// as `0.029999999329447746`. Real BASIC/`codegen_c.rs`'s own `bcc_strd`
/// (`snprintf(..., "% g", value)`) round to 6 significant digits and drop
/// trailing zeros; `BigDecimal.round(new MathContext(6))` +
/// `stripTrailingZeros()` + `toPlainString()` is the JVM-side equivalent
/// (deliberately not chasing `%g`'s scientific-notation threshold for very
/// large/small magnitudes -- `toPlainString()` always expands to plain
/// digits, an accepted narrower gap for values realistic BASCAL programs
/// don't produce, the same spirit as this backend's other documented
/// simplifications).
fn emit_double_str_helper() -> String {
    "\
.method public static bccStr : (D)Ljava/lang/String;
    .limit stack 6
    .limit locals 2

    new java/math/BigDecimal
    dup
    dload 0
    invokespecial java/math/BigDecimal/<init> (D)V
    new java/math/MathContext
    dup
    bipush 6
    invokespecial java/math/MathContext/<init> (I)V
    invokevirtual java/math/BigDecimal/round (Ljava/math/MathContext;)Ljava/math/BigDecimal;
    invokevirtual java/math/BigDecimal/stripTrailingZeros ()Ljava/math/BigDecimal;
    invokevirtual java/math/BigDecimal/toPlainString ()Ljava/lang/String;
    areturn
.end method

"
    .to_string()
}

fn array_descriptor(shape: &ArrayShape) -> String {
    format!(
        "{}{}",
        "[".repeat(shape.dimensions.len()),
        descriptor(shape.element)
    )
}

fn emit_array_initializers(context: &JvmContext, out: &mut String) -> Result<(), String> {
    for (index, shape) in context.arrays.values().enumerate() {
        for dimension in &shape.dimensions {
            emit_numeric_expr_as(dimension, NumericType::Int, out, context)?;
            out.push_str("    iconst_1\n    iadd\n");
        }
        let desc = array_descriptor(shape);
        out.push_str(&format!(
            "    multianewarray {desc} {}\n    putstatic {}/a{} {desc}\n",
            shape.dimensions.len(),
            context.class_name,
            index
        ));
    }
    Ok(())
}

fn emit_load(variable: Variable, out: &mut String, context: &JvmContext) {
    if variable.is_static {
        out.push_str(&format!(
            "    getstatic {}/{} {}\n",
            context.class_name,
            variable.field_name(),
            variable.descriptor()
        ));
    } else {
        out.push_str(&format!(
            "    {} {}\n",
            variable.load_opcode(),
            variable.slot
        ));
    }
}

fn emit_store(variable: Variable, out: &mut String, context: &JvmContext) {
    if variable.is_static {
        out.push_str(&format!(
            "    putstatic {}/{} {}\n",
            context.class_name,
            variable.field_name(),
            variable.descriptor()
        ));
    } else {
        out.push_str(&format!(
            "    {} {}\n",
            variable.store_opcode(),
            variable.slot
        ));
    }
}

fn jvm_label(name: &str) -> String {
    format!("L_user_{}", name.to_ascii_lowercase())
}

fn collect_labels(statements: &[Stmt]) -> HashSet<String> {
    let mut labels = HashSet::new();
    fn visit(statements: &[Stmt], labels: &mut HashSet<String>) {
        for statement in statements {
            match &statement.kind {
                Statement::Label(name) => {
                    labels.insert(name.to_ascii_lowercase());
                }
                Statement::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    visit(then_body, labels);
                    visit(else_body, labels);
                }
                Statement::For { body, .. }
                | Statement::While { body, .. }
                | Statement::Do { body, .. } => visit(body, labels),
                Statement::SelectCase {
                    cases, else_body, ..
                } => {
                    for case in cases {
                        visit(&case.body, labels);
                    }
                    visit(else_body, labels);
                }
                Statement::TryCatch {
                    try_body,
                    catch,
                    finally_body,
                } => {
                    visit(try_body, labels);
                    if let Some(catch) = catch {
                        visit(&catch.body, labels);
                    }
                    visit(finally_body, labels);
                }
                _ => {}
            }
        }
    }
    visit(statements, &mut labels);
    labels
}

/// Fixed capacity for `OPEN ... FOR RANDOM AS #n` channels -- matches the
/// spirit of `codegen_c.rs`'s own `BCC_MAX_CHANNELS` (there, 32; here, a
/// smaller bootstrap-sized 16, since nothing exercising this yet needs
/// more). Backs three parallel static arrays (`bccFiles`/`bccBufs`/
/// `bccRecLen`), each sized to this, indexed by `channel - 1` at runtime --
/// see `program_uses_random_open`/`emit_random_open`.
const JVM_MAX_CHANNELS: i64 = 16;

/// Where one `FIELD`-declared variable's bytes live: a fixed byte range
/// inside its channel's shared record buffer (`bccBufs[channel - 1]`).
/// Captured once, program-wide, by [`collect_field_vars`] before any
/// codegen runs, so every method (`main` and every `function`/`procedure`)
/// agrees on the same layout -- mirrors `codegen_c.rs`'s own
/// `apply_field_statement`, minus the typed-record fast path (`field_types`/
/// `string_fields` on `Statement::Field` are C-backend-only; this backend
/// always uses the same raw byte-buffer semantics real `FIELD` does).
#[derive(Clone, Copy)]
struct JvmFieldVar {
    /// 1-based, matching the `FIELD #n, ...` / `OPEN ... AS #n` spelling --
    /// subtract 1 when indexing `bccFiles`/`bccBufs`/`bccRecLen`.
    channel: i64,
    offset: i64,
    width: i64,
}

/// Scans `statements` (recursing into every block form) for `Statement::
/// Field`, registering each named field variable's channel/offset/width.
/// Requires the channel number and every field width to be integer literals
/// -- true of every `FIELD` `records::lower` ever synthesizes from the
/// record/file DSL, and of realistic hand-written `FIELD` besides -- so a
/// dynamic channel/width is rejected here with a clear message rather than
/// silently mis-laying-out the buffer.
fn collect_field_vars(
    statements: &[Stmt],
    out: &mut BTreeMap<String, JvmFieldVar>,
) -> Result<(), String> {
    for statement in statements {
        match &statement.kind {
            Statement::Field {
                channel, fields, ..
            } => {
                let Expr::Integer(channel) = channel else {
                    return Err(
                        "FIELD's channel number must be a literal under --target jvm".to_string(),
                    );
                };
                let mut offset = 0i64;
                for (width, name) in fields {
                    let Expr::Integer(width) = width else {
                        return Err(
                            "FIELD's field widths must be literal under --target jvm".to_string()
                        );
                    };
                    out.insert(
                        variable_key(name),
                        JvmFieldVar {
                            channel: *channel,
                            offset,
                            width: *width,
                        },
                    );
                    offset += width;
                }
            }
            Statement::If {
                then_body,
                else_body,
                ..
            } => {
                collect_field_vars(then_body, out)?;
                collect_field_vars(else_body, out)?;
            }
            Statement::For { body, .. }
            | Statement::While { body, .. }
            | Statement::Do { body, .. } => collect_field_vars(body, out)?,
            Statement::SelectCase {
                cases, else_body, ..
            } => {
                for case in cases {
                    collect_field_vars(&case.body, out)?;
                }
                collect_field_vars(else_body, out)?;
            }
            Statement::TryCatch {
                try_body,
                catch,
                finally_body,
            } => {
                collect_field_vars(try_body, out)?;
                if let Some(catch) = catch {
                    collect_field_vars(&catch.body, out)?;
                }
                collect_field_vars(finally_body, out)?;
            }
            _ => {}
        }
    }
    Ok(())
}

/// Whether `statements` (recursing into every block form) contains any
/// `OPEN ... FOR RANDOM` -- decides whether `generate()` declares/
/// initializes `bccFiles`/`bccBufs`/`bccRecLen` at all, so a program that
/// never touches random-access files gets no extra fields or `java/io`/
/// `java/nio` references in its generated class.
fn program_uses_random_open(statements: &[Stmt]) -> bool {
    statements.iter().any(|stmt| match &stmt.kind {
        Statement::Open {
            mode: OpenMode::Random,
            ..
        } => true,
        Statement::If {
            then_body,
            else_body,
            ..
        } => program_uses_random_open(then_body) || program_uses_random_open(else_body),
        Statement::For { body, .. }
        | Statement::While { body, .. }
        | Statement::Do { body, .. } => program_uses_random_open(body),
        Statement::SelectCase {
            cases, else_body, ..
        } => {
            cases
                .iter()
                .any(|case| program_uses_random_open(&case.body))
                || program_uses_random_open(else_body)
        }
        Statement::TryCatch {
            try_body,
            catch,
            finally_body,
        } => {
            program_uses_random_open(try_body)
                || catch
                    .as_ref()
                    .is_some_and(|catch| program_uses_random_open(&catch.body))
                || program_uses_random_open(finally_body)
        }
        _ => false,
    })
}

/// Whether `statements` (recursing into every block form) contains any
/// interactive `INPUT` -- decides whether `generate()` declares/
/// initializes the shared `bccStdin` reader at all.
fn program_uses_input(statements: &[Stmt]) -> bool {
    statements.iter().any(|stmt| match &stmt.kind {
        Statement::Input { .. } => true,
        Statement::If {
            then_body,
            else_body,
            ..
        } => program_uses_input(then_body) || program_uses_input(else_body),
        Statement::For { body, .. }
        | Statement::While { body, .. }
        | Statement::Do { body, .. } => program_uses_input(body),
        Statement::SelectCase {
            cases, else_body, ..
        } => {
            cases.iter().any(|case| program_uses_input(&case.body)) || program_uses_input(else_body)
        }
        Statement::TryCatch {
            try_body,
            catch,
            finally_body,
        } => {
            program_uses_input(try_body)
                || catch
                    .as_ref()
                    .is_some_and(|catch| program_uses_input(&catch.body))
                || program_uses_input(finally_body)
        }
        _ => false,
    })
}

/// Whether `statements` (recursing into every expression, via
/// `codegen_basic::visit_body_exprs`) contains bare `INKEY$` anywhere --
/// decides whether `generate()` puts the terminal into raw mode at program
/// start and restores it at every exit point (see `JvmContext::needs_inkey`'s
/// own doc comment).
fn program_uses_inkey(statements: &[Stmt]) -> bool {
    let mut found = false;
    crate::codegen_basic::visit_body_exprs(statements, &mut |expr| {
        if let Expr::Ident(ident) = expr {
            if ident.suffix == Some(TypeSuffix::String) && ident.name.eq_ignore_ascii_case("inkey")
            {
                found = true;
            }
        }
    });
    found
}

fn function_key(name: &BasicIdent) -> String {
    format!(
        "{}{}",
        name.name.to_ascii_lowercase(),
        name.suffix
            .map_or_else(String::new, |suffix| suffix.to_string())
    )
}

fn descriptor(ty: JvmType) -> &'static str {
    match ty {
        JvmType::String => "Ljava/lang/String;",
        JvmType::Numeric(NumericType::Int) => "I",
        JvmType::Numeric(NumericType::Long) => "J",
        JvmType::Numeric(NumericType::Double) => "D",
    }
}

fn array_load_opcode(ty: JvmType) -> &'static str {
    match ty {
        JvmType::String => "aaload",
        JvmType::Numeric(NumericType::Int) => "iaload",
        JvmType::Numeric(NumericType::Long) => "laload",
        JvmType::Numeric(NumericType::Double) => "daload",
    }
}

fn array_store_opcode(ty: JvmType) -> &'static str {
    match ty {
        JvmType::String => "aastore",
        JvmType::Numeric(NumericType::Int) => "iastore",
        JvmType::Numeric(NumericType::Long) => "lastore",
        JvmType::Numeric(NumericType::Double) => "dastore",
    }
}

/// The `newarray`/`anewarray` instruction (length already on the stack) for
/// a fresh single-element array of `ty` -- used only to build a `byref`
/// scalar argument's wrapper array at its call site (see
/// `emit_call_arguments`).
fn new_scalar_array_instruction(ty: JvmType) -> &'static str {
    match ty {
        JvmType::String => "anewarray java/lang/String",
        JvmType::Numeric(NumericType::Int) => "newarray int",
        JvmType::Numeric(NumericType::Long) => "newarray long",
        JvmType::Numeric(NumericType::Double) => "newarray double",
    }
}

/// Renders one call's actual arguments in order: a byval scalar (plain
/// value), a byval/byref array (already a reference type in Java, so passed
/// straight through with no wrapping either way -- see `JvmArrayParam::
/// by_ref`'s own call-site handling), or a `byref` scalar -- wrapped in a
/// fresh single-element array seeded with the argument's current value,
/// built into one of this *caller* function's own reserved scratch slots
/// (see `JvmContext::byref_scratch_base`'s own doc comment; slots are
/// reused across separate calls in the same function, safe because one
/// call's own write-back always completes before the next call's argument-
/// building starts). Returns the `(scratch_slot, target_variable)` pairs to
/// write back into the caller's own variables once the call returns (see
/// `emit_byref_call_writebacks`) -- `target_variable` must be a plain
/// identifier, the only shape a `byref` argument can be resolved against.
fn emit_call_arguments(
    name: &BasicIdent,
    args: &[Expr],
    signature: &FunctionSig,
    out: &mut String,
    context: &JvmContext,
) -> Result<Vec<(usize, Variable)>, String> {
    let mut scalars = signature.params.iter();
    let mut writebacks = Vec::new();
    let mut next_scratch = context.byref_scratch_base;
    for (position, arg) in args.iter().enumerate() {
        if let Some(array) = signature
            .array_params
            .iter()
            .find(|array| array.position == position)
        {
            let Expr::Ident(array_name) = arg else {
                return Err(format!(
                    "array parameter {position} of `{name}` needs a plain array argument"
                ));
            };
            let shape = context
                .arrays
                .get(&variable_key(array_name))
                .ok_or_else(|| format!("unknown JVM array `{array_name}`"))?;
            if shape.dimensions.len() != array.rank || shape.element != array.element {
                return Err(format!(
                    "array argument `{array_name}` doesn't match `{name}`'s parameter rank/type"
                ));
            }
            context.emit_array_load(array_name, out);
        } else {
            let ty = *scalars.next().expect("scalar source parameter");
            if signature.byref_scalar_positions.contains(&position) {
                let Expr::Ident(target) = arg else {
                    return Err(format!(
                        "byref parameter {position} of `{name}` needs a plain variable argument"
                    ));
                };
                let variable = context.variable(target)?;
                let scratch = next_scratch;
                next_scratch += 1;
                out.push_str("    iconst_1\n");
                out.push_str(&format!("    {}\n", new_scalar_array_instruction(ty)));
                out.push_str("    dup\n    iconst_0\n");
                match ty {
                    JvmType::String => emit_string_expr(arg, out, context)?,
                    JvmType::Numeric(nt) => emit_numeric_expr_as(arg, nt, out, context)?,
                }
                out.push_str(&format!("    {}\n", array_store_opcode(ty)));
                out.push_str(&format!("    astore {scratch}\n"));
                out.push_str(&format!("    aload {scratch}\n"));
                writebacks.push((scratch, variable));
            } else {
                match ty {
                    JvmType::String => emit_string_expr(arg, out, context)?,
                    JvmType::Numeric(nt) => emit_numeric_expr_as(arg, nt, out, context)?,
                }
            }
        }
    }
    Ok(writebacks)
}

/// The counterpart to `emit_call_arguments`: once a call returns, unwraps
/// each `byref` scalar argument's scratch array and stores the (possibly
/// mutated) value back into the caller's own variable.
fn emit_byref_call_writebacks(
    writebacks: &[(usize, Variable)],
    out: &mut String,
    context: &JvmContext,
) {
    for (scratch, variable) in writebacks {
        out.push_str(&format!(
            "    aload {scratch}\n    iconst_0\n    {}\n",
            array_load_opcode(variable.ty)
        ));
        emit_store(*variable, out, context);
    }
}

fn function_table(functions: &[FunctionDef]) -> HashMap<String, FunctionSig> {
    functions
        .iter()
        .map(|function| {
            (
                function_key(&function.name),
                FunctionSig {
                    params: function
                        .receiver
                        .map(|suffix| {
                            type_for_ident(&BasicIdent {
                                name: "self".to_string(),
                                suffix: Some(suffix),
                            })
                        })
                        .into_iter()
                        .chain(
                            function
                                .params
                                .iter()
                                .filter(|param| param.axes.is_none())
                                .map(|param| type_for_ident(&param.name)),
                        )
                        .collect(),
                    array_params: function
                        .params
                        .iter()
                        .enumerate()
                        .filter_map(|(position, param)| {
                            param.axes.as_ref().map(|axes| JvmArrayParam {
                                position,
                                element: type_for_ident(&param.name),
                                rank: axes.len(),
                                by_ref: param.mode == ParamMode::ByRef,
                            })
                        })
                        .collect(),
                    byref_scalar_positions: function
                        .params
                        .iter()
                        .enumerate()
                        .filter(|(_, param)| param.axes.is_none() && param.mode == ParamMode::ByRef)
                        .map(|(position, _)| position)
                        .collect(),
                    source_param_count: function.params.len(),
                    has_receiver: function.receiver.is_some(),
                    result: type_for_ident(&function.name),
                    returns_void: function.is_procedure,
                },
            )
        })
        .collect()
}

fn emit_function(function: &FunctionDef, parent: &JvmContext) -> Result<String, String> {
    let context = JvmContext::for_function(function, parent);
    let mut body = String::new();
    context.emit_initializers(&mut body);
    let signature = parent
        .functions
        .get(&function_key(&function.name))
        .expect("registered function");
    for (position, param) in function.params.iter().enumerate() {
        let Some(array) = signature
            .array_params
            .iter()
            .find(|array| array.position == position)
        else {
            continue;
        };
        if array.by_ref {
            continue;
        }
        let rank = array.rank;
        if !(1..=8).contains(&rank) {
            return Err(format!(
                "JVM byval array parameter `{}` needs a rank between 1 and 8",
                param.name
            ));
        }
        let slot = context.array_slots[&variable_key(&param.name)];
        let desc = format!("{}I", "[".repeat(rank));
        let kind = match array.element {
            JvmType::Numeric(NumericType::Int) => 0,
            JvmType::Numeric(NumericType::Long) => 1,
            JvmType::Numeric(NumericType::Double) => 2,
            JvmType::String => 3,
        };
        body.push_str(&format!("    aload {slot}\n    bipush {rank}\n    bipush {kind}\n    invokestatic {}/bccCopyArray (Ljava/lang/Object;II)Ljava/lang/Object;\n    checkcast {desc}\n    astore {slot}\n", context.class_name));
    }
    // Unwrap every `byref` scalar parameter's incoming single-element array
    // into its own ordinary working local -- see `JvmContext::
    // byref_scalar_params`'s own doc comment. The reverse (write the
    // working local's final value back into the array) happens at every
    // exit point instead: `Statement::Return`/`ReturnVoid`'s own arms, and
    // this function's fallthrough default-return below.
    for (array_slot, working) in &context.byref_scalar_params {
        body.push_str(&format!(
            "    aload {array_slot}\n    iconst_0\n    {}\n    {} {}\n",
            array_load_opcode(working.ty),
            working.store_opcode(),
            working.slot
        ));
    }
    let mut emitter = JvmEmitter {
        context: &context,
        next_label: 0,
        loop_exits: Vec::new(),
        return_type: (!function.is_procedure).then(|| type_for_ident(&function.name)),
        labels: collect_labels(&function.body),
        exception_handlers: Vec::new(),
    };
    for statement in &function.body {
        emitter.emit_statement(statement, &mut body)?;
    }
    // Resolver guarantees a return on every reachable path. This fallback
    // keeps the JVM verifier satisfied if an unsupported analysis edge leaks through.
    if function.is_procedure {
        context.emit_byref_writebacks(&mut body);
        body.push_str("    return\n");
    } else {
        context.emit_byref_writebacks(&mut body);
        match type_for_ident(&function.name) {
            JvmType::String => body.push_str("    ldc \"\"\n    areturn\n"),
            JvmType::Numeric(NumericType::Int) => body.push_str("    iconst_0\n    ireturn\n"),
            JvmType::Numeric(NumericType::Long) => body.push_str("    lconst_0\n    lreturn\n"),
            JvmType::Numeric(NumericType::Double) => body.push_str("    dconst_0\n    dreturn\n"),
        }
    }
    let sig = parent
        .functions
        .get(&function_key(&function.name))
        .expect("registered function");
    let args = sig.descriptor_params().join("");
    let result = if sig.returns_void {
        "V"
    } else {
        descriptor(sig.result)
    };
    let handlers = emitter
        .exception_handlers
        .iter()
        .map(|handler| {
            format!(
                "    .catch java/lang/RuntimeException from {} to {} using {}\n",
                handler.start, handler.end, handler.handler
            )
        })
        .collect::<String>();
    Ok(format!(".method public static {} : ({args}){}\n    .limit stack 16\n    .limit locals {}\n\n{body}{handlers}.end method\n\n", function.name.name, result, context.local_count()))
}

struct JvmEmitter<'a> {
    context: &'a JvmContext,
    next_label: usize,
    loop_exits: Vec<String>,
    return_type: Option<JvmType>,
    labels: HashSet<String>,
    exception_handlers: Vec<JvmExceptionHandler>,
}

struct JvmExceptionHandler {
    start: String,
    end: String,
    handler: String,
}

impl JvmEmitter<'_> {
    fn emit_try_catch(
        &mut self,
        try_body: &[Stmt],
        catch: Option<&crate::ast::TryCatchHandler>,
        finally_body: &[Stmt],
        source_filename: &str,
        out: &mut String,
    ) -> Result<(), String> {
        let Some(catch) = catch else {
            return Err("JVM TRY currently requires a CATCH clause".to_string());
        };
        let id = self.next_label;
        self.next_label += 1;
        let start = format!("L_try_{id}_start");
        let end = format!("L_try_{id}_end");
        let handler = format!("L_try_{id}_catch");
        let finish = format!("L_try_{id}_finish");
        self.exception_handlers.push(JvmExceptionHandler {
            start: start.clone(),
            end: end.clone(),
            handler: handler.clone(),
        });

        out.push_str(&format!("{start}:\n"));
        for statement in try_body {
            self.emit_statement(statement, out)?;
        }
        out.push_str(&format!("    goto {finish}\n{end}:\n{handler}:\n"));
        out.push_str("    invokevirtual java/lang/Throwable/getMessage ()Ljava/lang/String;\n    invokestatic java/lang/Integer/parseInt (Ljava/lang/String;)I\n");
        let err = self.context.variable(&catch.err_var)?;
        emit_store(err, out, self.context);
        let erl = self.context.variable(&catch.erl_var)?;
        out.push_str("    iconst_0\n");
        emit_store(erl, out, self.context);
        if let Some(source_var) = &catch.source_var {
            let source = self.context.variable(source_var)?;
            out.push_str(&format!(
                "    ldc \"{}\"\n",
                escape_jvm_string(&crate::diagnostics::display_source_filename(
                    source_filename
                ))
            ));
            emit_store(source, out, self.context);
        }
        let matched = format!("L_try_{id}_matched");
        let rethrow = format!("L_try_{id}_rethrow");
        for filter in &catch.error_filter {
            emit_load(err, out, self.context);
            emit_numeric_expr_as(filter, NumericType::Int, out, self.context)?;
            out.push_str(&format!("    if_icmpeq {matched}\n"));
        }
        if !catch.error_filter.is_empty() {
            out.push_str(&format!("    goto {rethrow}\n{matched}:\n"));
        }
        for statement in &catch.body {
            self.emit_statement(statement, out)?;
        }
        out.push_str(&format!("    goto {finish}\n"));
        if !catch.error_filter.is_empty() {
            out.push_str(&format!("{rethrow}:\n"));
            for statement in finally_body {
                self.emit_statement(statement, out)?;
            }
            out.push_str("    new java/lang/RuntimeException\n    dup\n");
            emit_load(err, out, self.context);
            out.push_str("    invokestatic java/lang/Integer/toString (I)Ljava/lang/String;\n    invokespecial java/lang/RuntimeException/<init> (Ljava/lang/String;)V\n    athrow\n");
        }
        out.push_str(&format!("{finish}:\n"));
        for statement in finally_body {
            self.emit_statement(statement, out)?;
        }
        Ok(())
    }

    fn emit_statement(&mut self, statement: &Stmt, out: &mut String) -> Result<(), String> {
        match &statement.kind {
            Statement::Print { tokens } => emit_print_tokens(tokens, out, self.context),
            Statement::Lprint(tokens) => emit_print_tokens(tokens, out, self.context),
            Statement::Cls => emit_terminal_escape("\u{1b}[2J\u{1b}[H", out),
            Statement::Beep => emit_terminal_escape("\u{7}", out),
            // `STOP`/`SYSTEM` -- both halt the whole program outright,
            // same as `codegen_c.rs`'s own treatment (see its doc comment
            // there): `System.exit(0)`, not a plain `return`, since either
            // can appear inside a function/procedure body too, where
            // `return` would just unwind that one call frame -- wrong,
            // since both need to halt the entire JVM regardless of call
            // depth. Real BASIC's STOP is an interactive breakpoint-style
            // halt (resumable with CONT in an interpreter); meaningless for
            // a compiled program, so indistinguishable from SYSTEM here.
            Statement::Stop | Statement::System => {
                emit_inkey_restore(self.context, out);
                out.push_str("    iconst_0\n    invokestatic java/lang/System/exit (I)V\n");
                Ok(())
            }
            Statement::Color { fg, bg } => {
                let Expr::Integer(fg) = fg else {
                    return Err(
                        "JVM COLOR currently requires a literal foreground value".to_string()
                    );
                };
                let fg_code = ANSI_FG[(*fg as usize) & 15];
                let code = if let Some(Expr::Integer(bg)) = bg {
                    format!("\u{1b}[{};{}m", fg_code, ANSI_BG[(*bg as usize) & 7])
                } else if bg.is_none() {
                    format!("\u{1b}[{fg_code}m")
                } else {
                    return Err(
                        "JVM COLOR currently requires a literal background value".to_string()
                    );
                };
                emit_terminal_escape(&code, out)
            }
            // ANSI's own cursor-position escape is already `row;col`,
            // 1-based, exactly matching BASIC's own `LOCATE row, col` -- no
            // reordering or offset needed, unlike `COLOR`'s palette
            // remapping. `row`/`col` may be any numeric expression (not
            // just a literal), same as `codegen_c.rs`'s own `Statement::
            // Locate` -- built with `StringBuilder` since both interleave
            // with literal escape-sequence text, same as `emit_tab_escape`.
            Statement::Locate { row, col } => {
                out.push_str("    getstatic java/lang/System/out Ljava/io/PrintStream;\n");
                out.push_str("    new java/lang/StringBuilder\n    dup\n    invokespecial java/lang/StringBuilder/<init> ()V\n");
                out.push_str("    ldc \"\u{1b}[\"\n    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;\n");
                emit_numeric_expr_as(row, NumericType::Int, out, self.context)?;
                out.push_str("    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;\n");
                out.push_str("    ldc \";\"\n    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;\n");
                emit_numeric_expr_as(col, NumericType::Int, out, self.context)?;
                out.push_str("    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;\n");
                out.push_str("    ldc \"H\"\n    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;\n");
                out.push_str(
                    "    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;\n",
                );
                out.push_str("    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V\n");
                Ok(())
            }
            Statement::TryCatch {
                try_body,
                catch,
                finally_body,
            } => self.emit_try_catch(
                try_body,
                catch.as_ref(),
                finally_body,
                &statement.pos.filename,
                out,
            ),
            Statement::ThrowStmt { code: Some(code) } | Statement::ErrorStmt { code } => {
                out.push_str("    new java/lang/RuntimeException\n    dup\n");
                emit_numeric_expr_as(code, NumericType::Int, out, self.context)?;
                out.push_str("    invokestatic java/lang/Integer/toString (I)Ljava/lang/String;\n    invokespecial java/lang/RuntimeException/<init> (Ljava/lang/String;)V\n    athrow\n");
                Ok(())
            }
            Statement::ThrowStmt { code: None } => {
                return Err(
                    "JVM bare THROW is only supported inside a JVM catch handler".to_string(),
                );
            }
            Statement::Dim { .. } | Statement::Const { .. } => Ok(()),
            Statement::Open {
                mode: OpenMode::Random,
                file,
                channel,
                len: Some(len),
            } => emit_random_open(file, channel, len, out, self.context),
            Statement::Open {
                mode: OpenMode::Random,
                len: None,
                ..
            } => Err("OPEN ... FOR RANDOM needs an explicit LEN under --target jvm".to_string()),
            Statement::Close { channel } => emit_file_close(channel, out, self.context),
            // Pure compile-time bookkeeping -- every named field's channel/
            // offset/width was already captured program-wide by
            // `collect_field_vars` before codegen began (see `JvmContext::
            // field_vars`), so there is nothing left to emit here.
            Statement::Field { .. } => Ok(()),
            Statement::Get {
                channel,
                record: Some(record),
                ..
            } => emit_get_or_put(true, channel, record, out, self.context),
            Statement::Put {
                channel,
                record: Some(record),
                ..
            } => emit_get_or_put(false, channel, record, out, self.context),
            Statement::Get { record: None, .. } | Statement::Put { record: None, .. } => Err(
                "GET/PUT without an explicit record number aren't supported under --target jvm"
                    .to_string(),
            ),
            Statement::Lset { var, value } => {
                let field = *self
                    .context
                    .field_vars
                    .get(&variable_key(var))
                    .ok_or_else(|| {
                        format!("`{var}` isn't a FIELD-declared variable under --target jvm")
                    })?;
                emit_lset(field, value, out, self.context)
            }
            Statement::Rset { var, value } => {
                let field = *self
                    .context
                    .field_vars
                    .get(&variable_key(var))
                    .ok_or_else(|| {
                        format!("`{var}` isn't a FIELD-declared variable under --target jvm")
                    })?;
                emit_rset(field, value, out, self.context)
            }
            Statement::Input { prompt, vars } => {
                emit_input(prompt.as_deref(), vars, out, self.context)
            }
            Statement::Assignment {
                target: Expr::Ident(name),
                value,
            } => {
                let variable = self.context.variable(name)?;
                match variable.ty {
                    JvmType::String => emit_string_expr(value, out, self.context)?,
                    JvmType::Numeric(ty) => emit_numeric_expr_as(value, ty, out, self.context)?,
                }
                emit_store(variable, out, self.context);
                Ok(())
            }
            Statement::Assignment {
                target:
                    Expr::Call {
                        name,
                        args: indices,
                    }
                    | Expr::ArrayRef { name, indices },
                value,
            } => {
                let shape = self
                    .context
                    .arrays
                    .get(&variable_key(name))
                    .ok_or_else(|| format!("unknown JVM array `{name}`"))?;
                if indices.len() != shape.dimensions.len() {
                    return Err(
                        "JVM indexed assignment has the wrong number of dimensions".to_string()
                    );
                }
                self.context.emit_array_load(name, out);
                for index in &indices[..indices.len() - 1] {
                    emit_numeric_expr_as(index, NumericType::Int, out, self.context)?;
                    out.push_str("    aaload\n");
                }
                emit_numeric_expr_as(
                    indices.last().expect("validated index count"),
                    NumericType::Int,
                    out,
                    self.context,
                )?;
                match shape.element {
                    JvmType::String => emit_string_expr(value, out, self.context)?,
                    JvmType::Numeric(ty) => emit_numeric_expr_as(value, ty, out, self.context)?,
                }
                out.push_str(&format!("    {}\n", array_store_opcode(shape.element)));
                Ok(())
            }
            Statement::ExprStmt(Expr::Call { name, args })
            | Statement::ExprStmt(Expr::ArrayRef {
                name,
                indices: args,
            }) => {
                let signature = self
                    .context
                    .function(name)
                    .ok_or_else(|| format!("unknown JVM procedure `{name}`"))?;
                if signature.source_param_count != args.len() {
                    return Err(format!("invalid JVM procedure call `{name}`"));
                }
                let writebacks = emit_call_arguments(name, args, &signature, out, self.context)?;
                let args_descriptor = signature.descriptor_params().join("");
                let result_descriptor = if signature.returns_void {
                    "V".to_string()
                } else {
                    descriptor(signature.result).to_string()
                };
                out.push_str(&format!(
                    "    invokestatic {}/{} ({args_descriptor}){result_descriptor}\n",
                    self.context.class_name, name.name
                ));
                // A function called as a bare statement discards its
                // result -- real BASIC's own `readKey$()`/`waitAnyKey()`-
                // style "call for the side effect only" pattern (see
                // `tutorial/inventory.bcl`'s own `readKey$()` used this way
                // to pause for a keypress without caring what key it was).
                if !signature.returns_void {
                    out.push_str(match signature.result {
                        JvmType::String => "    pop\n",
                        JvmType::Numeric(NumericType::Long)
                        | JvmType::Numeric(NumericType::Double) => "    pop2\n",
                        JvmType::Numeric(NumericType::Int) => "    pop\n",
                    });
                }
                emit_byref_call_writebacks(&writebacks, out, self.context);
                Ok(())
            }
            Statement::If {
                condition,
                then_body,
                else_body,
            } => self.emit_if(condition, then_body, else_body, out),
            Statement::While { condition, body } => self.emit_while(condition, body, out),
            Statement::For {
                var,
                start,
                end,
                step,
                body,
            } => self.emit_for(var, start, end, step.as_ref(), body, out),
            Statement::Do {
                condition,
                body,
                post_condition,
            } => self.emit_do(condition.as_ref(), body, post_condition.as_ref(), out),
            Statement::SelectCase {
                expr,
                cases,
                else_body,
            } => self.emit_select_case(expr, cases, else_body, out),
            Statement::Exit => {
                let label = self.loop_exits.last().ok_or_else(|| {
                    "`exit` is only supported inside a JVM-transpiled loop".to_string()
                })?;
                out.push_str(&format!("    goto {label}\n"));
                Ok(())
            }
            Statement::Return { value } => match self.return_type {
                Some(JvmType::String) => {
                    emit_string_expr(value, out, self.context)?;
                    self.context.emit_byref_writebacks(out);
                    out.push_str("    areturn\n");
                    Ok(())
                }
                Some(JvmType::Numeric(ty)) => {
                    emit_numeric_expr_as(value, ty, out, self.context)?;
                    self.context.emit_byref_writebacks(out);
                    out.push_str(match ty {
                        NumericType::Int => "    ireturn\n",
                        NumericType::Long => "    lreturn\n",
                        NumericType::Double => "    dreturn\n",
                    });
                    Ok(())
                }
                None => Err("RETURN is only supported inside a JVM function".to_string()),
            },
            Statement::ReturnVoid => {
                if self.return_type.is_some() {
                    Err("bare RETURN is only supported inside a JVM procedure".to_string())
                } else {
                    self.context.emit_byref_writebacks(out);
                    out.push_str("    return\n");
                    Ok(())
                }
            }
            Statement::Label(name) => {
                out.push_str(&format!("{}:\n", jvm_label(name)));
                Ok(())
            }
            Statement::Goto(target) => {
                let Expr::Ident(target) = target else {
                    return Err("JVM GOTO targets must be labels".to_string());
                };
                let key = target.name.to_ascii_lowercase();
                if !self.labels.contains(&key) {
                    return Err(format!(
                        "JVM GOTO target `{target}` is not in this callable"
                    ));
                }
                out.push_str(&format!("    goto {}\n", jvm_label(&target.name)));
                Ok(())
            }
            Statement::Gosub(_) => Err(
                "GOSUB is not supported by the JVM target; use a function/procedure instead"
                    .to_string(),
            ),
            Statement::GlobalDecl(_) => Ok(()),
            Statement::End => {
                emit_inkey_restore(self.context, out);
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
                "{other:?} is not supported by the minimal JVM backend yet"
            )),
        }
    }

    fn emit_if(
        &mut self,
        condition: &Expr,
        then_body: &[Stmt],
        else_body: &[Stmt],
        out: &mut String,
    ) -> Result<(), String> {
        let id = self.next_label;
        self.next_label += 1;
        let else_label = format!("L_if_{id}_else");
        let end_label = format!("L_if_{id}_end");

        emit_jump_if_false(condition, &else_label, out, self.context)?;
        for statement in then_body {
            self.emit_statement(statement, out)?;
        }
        if else_body.is_empty() {
            out.push_str(&format!("{else_label}:\n"));
            return Ok(());
        }

        out.push_str(&format!("    goto {end_label}\n{else_label}:\n"));
        for statement in else_body {
            self.emit_statement(statement, out)?;
        }
        out.push_str(&format!("{end_label}:\n"));
        Ok(())
    }

    fn emit_while(
        &mut self,
        condition: &Expr,
        body: &[Stmt],
        out: &mut String,
    ) -> Result<(), String> {
        let id = self.next_label;
        self.next_label += 1;
        let top_label = format!("L_while_{id}_top");
        let end_label = format!("L_while_{id}_end");
        out.push_str(&format!("{top_label}:\n"));
        emit_jump_if_false(condition, &end_label, out, self.context)?;
        self.loop_exits.push(end_label.clone());
        for statement in body {
            self.emit_statement(statement, out)?;
        }
        self.loop_exits.pop();
        out.push_str(&format!("    goto {top_label}\n{end_label}:\n"));
        Ok(())
    }

    fn emit_for(
        &mut self,
        var: &BasicIdent,
        start: &Expr,
        end: &Expr,
        step: Option<&Expr>,
        body: &[Stmt],
        out: &mut String,
    ) -> Result<(), String> {
        let variable = self.context.variable(var)?;
        let JvmType::Numeric(NumericType::Int) = variable.ty else {
            return Err(
                "the JVM backend currently supports integer FOR variables only".to_string(),
            );
        };
        let default_step = Expr::Integer(1);
        let step = step.unwrap_or(&default_step);
        let step_value = match step {
            Expr::Integer(value) => *value,
            Expr::Unary {
                op: UnaryOp::Neg,
                expr,
            } => match &**expr {
                Expr::Integer(value) => -*value,
                _ => {
                    return Err("the JVM backend currently requires a literal FOR STEP".to_string())
                }
            },
            _ => return Err("the JVM backend currently requires a literal FOR STEP".to_string()),
        };
        if step_value == 0 {
            return Err("FOR STEP 0 is not supported by the JVM backend".to_string());
        }
        emit_numeric_expr_as(start, NumericType::Int, out, self.context)?;
        emit_store(variable, out, self.context);

        let id = self.next_label;
        self.next_label += 1;
        let top_label = format!("L_for_{id}_top");
        let end_label = format!("L_for_{id}_end");
        out.push_str(&format!("{top_label}:\n"));
        emit_load(variable, out, self.context);
        emit_numeric_expr_as(end, NumericType::Int, out, self.context)?;
        let branch = if step_value > 0 {
            "if_icmpgt"
        } else {
            "if_icmplt"
        };
        out.push_str(&format!("    {branch} {end_label}\n"));
        self.loop_exits.push(end_label.clone());
        for statement in body {
            self.emit_statement(statement, out)?;
        }
        self.loop_exits.pop();
        emit_load(variable, out, self.context);
        emit_numeric_expr_as(step, NumericType::Int, out, self.context)?;
        out.push_str(&format!("    iadd\n"));
        emit_store(variable, out, self.context);
        out.push_str(&format!("    goto {top_label}\n{end_label}:\n"));
        Ok(())
    }

    fn emit_do(
        &mut self,
        condition: Option<&crate::ast::DoCondition>,
        body: &[Stmt],
        post_condition: Option<&crate::ast::DoCondition>,
        out: &mut String,
    ) -> Result<(), String> {
        let id = self.next_label;
        self.next_label += 1;
        let top_label = format!("L_do_{id}_top");
        let end_label = format!("L_do_{id}_end");
        out.push_str(&format!("{top_label}:\n"));
        if let Some(condition) = condition {
            if condition.is_while {
                emit_jump_if_false(&condition.expr, &end_label, out, self.context)?;
            } else {
                emit_jump_if_true(&condition.expr, &end_label, out, self.context)?;
            }
        }
        self.loop_exits.push(end_label.clone());
        for statement in body {
            self.emit_statement(statement, out)?;
        }
        self.loop_exits.pop();
        if let Some(condition) = post_condition {
            if condition.is_while {
                emit_jump_if_false(&condition.expr, &end_label, out, self.context)?;
            } else {
                emit_jump_if_true(&condition.expr, &end_label, out, self.context)?;
            }
        }
        out.push_str(&format!("    goto {top_label}\n{end_label}:\n"));
        Ok(())
    }

    fn emit_select_case(
        &mut self,
        expr: &Expr,
        cases: &[crate::ast::CaseClause],
        else_body: &[Stmt],
        out: &mut String,
    ) -> Result<(), String> {
        let id = self.next_label;
        self.next_label += 1;
        let end_label = format!("L_select_{id}_end");
        let case_labels = (0..cases.len())
            .map(|index| format!("L_select_{id}_case_{index}"))
            .collect::<Vec<_>>();

        let string_selector = self.context.is_string_expr(expr);
        if string_selector {
            emit_string_expr(expr, out, self.context)?;
        } else if emit_numeric_expr(expr, out, self.context)? != NumericType::Int {
            return Err(
                "the JVM backend currently supports integer SELECT CASE selectors only".to_string(),
            );
        }

        for (index, case) in cases.iter().enumerate() {
            let next_label = format!("L_select_{id}_next_{index}");
            for (value_index, value) in case.values.iter().enumerate() {
                match value {
                    CaseValue::Single(value) if string_selector => {
                        if !self.context.is_string_expr(value) {
                            return Err("SELECT CASE string patterns must be strings".to_string());
                        }
                        out.push_str("    dup\n");
                        emit_string_expr(value, out, self.context)?;
                        out.push_str(&format!("    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z\n    ifne {}\n", case_labels[index]));
                    }
                    CaseValue::Single(value) => {
                        out.push_str("    dup\n");
                        emit_numeric_expr_as(value, NumericType::Int, out, self.context)?;
                        out.push_str(&format!("    isub\n    ifeq {}\n", case_labels[index]));
                    }
                    CaseValue::Range { from, to } if !string_selector => {
                        let next_value = format!("L_select_{id}_case_{index}_value_{value_index}");
                        out.push_str("    dup\n");
                        emit_numeric_expr_as(from, NumericType::Int, out, self.context)?;
                        out.push_str(&format!("    if_icmplt {next_value}\n    dup\n"));
                        emit_numeric_expr_as(to, NumericType::Int, out, self.context)?;
                        out.push_str(&format!(
                            "    if_icmple {}\n{next_value}:\n",
                            case_labels[index]
                        ));
                    }
                    CaseValue::Is { op, value } if !string_selector => {
                        out.push_str("    dup\n");
                        emit_numeric_expr_as(value, NumericType::Int, out, self.context)?;
                        let branch = match op {
                            BinaryOp::Eq => "if_icmpeq",
                            BinaryOp::Ne => "if_icmpne",
                            BinaryOp::Lt => "if_icmplt",
                            BinaryOp::Le => "if_icmple",
                            BinaryOp::Gt => "if_icmpgt",
                            BinaryOp::Ge => "if_icmpge",
                            other => {
                                return Err(format!(
                                    "CASE IS {other:?} is not supported by the JVM backend"
                                ))
                            }
                        };
                        out.push_str(&format!("    {branch} {}\n", case_labels[index]));
                    }
                    _ => {
                        return Err(
                            "this SELECT CASE pattern is not supported by the JVM backend"
                                .to_string(),
                        )
                    }
                }
            }
            out.push_str(&format!("    goto {next_label}\n{next_label}:\n"));
        }
        out.push_str("    pop\n");
        for statement in else_body {
            self.emit_statement(statement, out)?;
        }
        out.push_str(&format!("    goto {end_label}\n"));
        for (case, label) in cases.iter().zip(case_labels) {
            out.push_str(&format!("{label}:\n    pop\n"));
            for statement in &case.body {
                self.emit_statement(statement, out)?;
            }
            out.push_str(&format!("    goto {end_label}\n"));
        }
        out.push_str(&format!("{end_label}:\n"));
        Ok(())
    }
}

/// Evaluates a BASIC numeric condition and branches when it is zero. BASIC
/// accepts any numeric scalar as a condition, while the JVM's `ifeq` only
/// accepts an int; long and double values therefore need an explicit compare
/// with their respective zero constants first.
fn emit_jump_if_false(
    condition: &Expr,
    label: &str,
    out: &mut String,
    context: &JvmContext,
) -> Result<(), String> {
    if let Expr::Binary {
        left,
        op: BinaryOp::AndAnd,
        right,
    } = condition
    {
        emit_jump_if_false(left, label, out, context)?;
        return emit_jump_if_false(right, label, out, context);
    }
    if let Expr::Binary {
        left,
        op: BinaryOp::OrOr,
        right,
    } = condition
    {
        let success = next_condition_label(context);
        emit_jump_if_true(left, &success, out, context)?;
        emit_jump_if_false(right, label, out, context)?;
        out.push_str(&format!("{success}:\n"));
        return Ok(());
    }
    let ty = emit_numeric_expr(condition, out, context)?;
    match ty {
        NumericType::Int => out.push_str(&format!("    ifeq {label}\n")),
        NumericType::Long => out.push_str(&format!("    lconst_0\n    lcmp\n    ifeq {label}\n")),
        NumericType::Double => {
            out.push_str(&format!("    dconst_0\n    dcmpg\n    ifeq {label}\n"))
        }
    }
    Ok(())
}

fn emit_jump_if_true(
    condition: &Expr,
    label: &str,
    out: &mut String,
    context: &JvmContext,
) -> Result<(), String> {
    if let Expr::Binary {
        left,
        op: BinaryOp::OrOr,
        right,
    } = condition
    {
        emit_jump_if_true(left, label, out, context)?;
        return emit_jump_if_true(right, label, out, context);
    }
    if let Expr::Binary {
        left,
        op: BinaryOp::AndAnd,
        right,
    } = condition
    {
        let failure = next_condition_label(context);
        emit_jump_if_false(left, &failure, out, context)?;
        emit_jump_if_true(right, label, out, context)?;
        out.push_str(&format!("{failure}:\n"));
        return Ok(());
    }
    let ty = emit_numeric_expr(condition, out, context)?;
    match ty {
        NumericType::Int => out.push_str(&format!("    ifne {label}\n")),
        NumericType::Long => out.push_str(&format!("    lconst_0\n    lcmp\n    ifne {label}\n")),
        NumericType::Double => {
            out.push_str(&format!("    dconst_0\n    dcmpg\n    ifne {label}\n"))
        }
    }
    Ok(())
}

fn next_condition_label(context: &JvmContext) -> String {
    let id = context.condition_label.get();
    context.condition_label.set(id + 1);
    format!("L_condition_{id}")
}

/// Emits a bare numeric `PRINT` value and returns the descriptor its
/// `PrintStream` call should use. `Int`/`Long` print via the JVM's own
/// native formatting, unchanged; a `Double` (BASCAL `single`/`double`,
/// always represented as a JVM `double` -- see `TypeSuffix::Single |
/// TypeSuffix::Double`) is routed through `bccStr` instead of
/// `PrintStream.print(double)`'s own full-round-trip-precision formatting,
/// the same fix `STR$`'s `Double` branch needs and for the same reason --
/// see `emit_double_str_helper`'s own doc comment.
fn emit_numeric_print_value(
    expr: &Expr,
    out: &mut String,
    context: &JvmContext,
) -> Result<&'static str, String> {
    let ty = emit_numeric_expr(expr, out, context)?;
    if ty == NumericType::Double {
        out.push_str(&format!(
            "    invokestatic {}/bccStr (D)Ljava/lang/String;\n",
            context.class_name
        ));
        Ok("(Ljava/lang/String;)V")
    } else {
        Ok(ty.print_descriptor())
    }
}

/// Emits each PRINT value directly to `System.out`.  Separators currently
/// follow the bootstrap C backend's simple rule: they only suppress the
/// final newline; they do not implement BASCOM's tab-zone formatting.
///
/// When a trailing `;`/`,` suppresses that final newline (`print`, not
/// `println`), this also flushes `System.out` explicitly right after --
/// `System.out`'s own autoflush only triggers on a `\n` byte or a
/// `println` call, so a prompt like `waitAnyKey()`'s own `"Press the
/// AnyKey..."` (`tutorial/inventory.bcl`) or `INPUT`'s `"...? "`
/// (`emit_input`) would otherwise sit in the stream's internal buffer,
/// invisible on the real terminal, until something else happened to flush
/// it -- which, since neither `INKEY$`'s polling read nor `INPUT`'s
/// `readLine()` flushes either, could be arbitrarily later. A real run
/// showed the prompt appearing only *after* a keystroke was read blind,
/// with whatever printed next arriving all at once right alongside it --
/// a real bug, not print-ordering in the BCL source (`codegen_c.rs`'s
/// `printf`/`fflush(stdout)` has the identical gap and fix, for the same
/// reason: C's own stdio buffering).
fn emit_print_tokens(
    tokens: &[PrintToken],
    out: &mut String,
    context: &JvmContext,
) -> Result<(), String> {
    let last_expr = tokens
        .iter()
        .rposition(|token| matches!(token, PrintToken::Expr(_)));
    let trailing_separator = matches!(tokens.last(), Some(PrintToken::Semi | PrintToken::Comma));

    for (index, token) in tokens.iter().enumerate() {
        let PrintToken::Expr(expr) = token else {
            continue;
        };
        out.push_str("    getstatic java/lang/System/out Ljava/io/PrintStream;\n");
        // `TAB(n)`/`SPC(n)` -- print-position directives, not real values
        // (see codegen_c.rs's own `render_print_tokens` doc comment: only
        // legal as a bare, adjacent `PRINT` token, so they're intercepted
        // here before the general string/numeric rendering below, which has
        // no notion of either). `tab`/`spc` are suffixless, so a single-arg
        // call to either always parses as `Expr::Call`, never
        // `Expr::ArrayRef` (see `make_paren_ident_expr` in parser.rs).
        let descriptor = if let Expr::Call { name, args } = expr {
            if args.len() == 1 && name.name.eq_ignore_ascii_case("tab") {
                emit_tab_escape(&args[0], out, context)?;
                "(Ljava/lang/String;)V"
            } else if args.len() == 1 && name.name.eq_ignore_ascii_case("spc") {
                emit_numeric_expr_as(&args[0], NumericType::Int, out, context)?;
                emit_space_string_runtime(out);
                "(Ljava/lang/String;)V"
            } else if context.is_string_expr(expr) {
                emit_string_expr(expr, out, context)?;
                "(Ljava/lang/String;)V"
            } else {
                emit_numeric_print_value(expr, out, context)?
            }
        } else if context.is_string_expr(expr) {
            emit_string_expr(expr, out, context)?;
            "(Ljava/lang/String;)V"
        } else {
            emit_numeric_print_value(expr, out, context)?
        };
        let method = if Some(index) == last_expr && !trailing_separator {
            "println"
        } else {
            "print"
        };
        out.push_str(&format!(
            "    invokevirtual java/io/PrintStream/{method} {descriptor}\n"
        ));
    }
    if trailing_separator {
        out.push_str("    getstatic java/lang/System/out Ljava/io/PrintStream;\n    invokevirtual java/io/PrintStream/flush ()V\n");
    }
    Ok(())
}

/// `TAB(n)` -- moves the cursor to column `n` on the current line, via the
/// same ANSI cursor-column-absolute escape family `LOCATE`/`COLOR` already
/// use (`\x1b[<n>G`) -- consistent with `LOCATE`'s own row/col passing
/// straight through to ANSI's identical 1-based column numbering, no
/// reordering or offset needed. Mirrors `codegen_c.rs`'s own `TAB(n)`
/// handling in `render_print_tokens` (there, a `printf` format string; here,
/// built with `StringBuilder` since `n` is only known at runtime and
/// interleaves with literal escape-sequence text).
fn emit_tab_escape(n: &Expr, out: &mut String, context: &JvmContext) -> Result<(), String> {
    out.push_str("    new java/lang/StringBuilder\n    dup\n    invokespecial java/lang/StringBuilder/<init> ()V\n");
    out.push_str("    ldc \"\u{1b}[\"\n    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;\n");
    emit_numeric_expr_as(n, NumericType::Int, out, context)?;
    out.push_str("    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;\n");
    out.push_str("    ldc \"G\"\n    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;\n");
    out.push_str("    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;\n");
    Ok(())
}

/// Pushes a `String` of as many ASCII spaces as the already-computed `int`
/// on top of the stack -- the same `newarray`/`Arrays.fill`/`new String`
/// idiom `SPACE$`/`STRING$`/`SPC` all share (see `emit_string_expr`'s own
/// `"space"` arm), just factored out so `SPC` (a print-token directive, not
/// a general expression) can reuse it directly.
fn emit_space_string_runtime(out: &mut String) {
    out.push_str(
        "    newarray char\n    dup\n    bipush 32\n    invokestatic java/util/Arrays/fill ([CC)V\n    \
         new java/lang/String\n    dup_x1\n    swap\n    invokespecial java/lang/String/<init> ([C)V\n",
    );
}

/// `COLOR fg[, bg]`'s CGA-to-ANSI-SGR color index table -- CGA's 0-15
/// ordering (black, blue, green, cyan, red, magenta, brown, white, then the
/// same eight again "bright") does *not* match ANSI's 0-7 ordering (black,
/// red, green, yellow, blue, magenta, cyan, white), so a naive `30 + fg`
/// swaps blue and red (and cyan and yellow) outright -- exactly the bug this
/// table fixes. Mirrors `codegen_c.rs`'s own `bcc_ansi_fg`/`bcc_ansi_bg`
/// tables (and their real CGA palette doc comment) exactly, so a `COLOR`
/// value renders the same way under both targets.
const ANSI_FG: [i32; 16] = [
    30, 34, 32, 36, 31, 35, 33, 37, 90, 94, 92, 96, 91, 95, 93, 97,
];
const ANSI_BG: [i32; 8] = [40, 44, 42, 46, 41, 45, 43, 47];

fn emit_terminal_escape(value: &str, out: &mut String) -> Result<(), String> {
    out.push_str(&format!(
        "    getstatic java/lang/System/out Ljava/io/PrintStream;\n    ldc \"{}\"\n    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V\n",
        escape_jvm_string(value)
    ));
    Ok(())
}

/// Runtime index (0-based) for `bccFiles`/`bccBufs`/`bccRecLen`, given a
/// 1-based `#n` channel expression. Re-evaluates `channel` on every call --
/// harmless for the literal channel numbers every realistic `OPEN`/`FIELD`/
/// `GET`/`PUT`/`CLOSE` uses, and simpler than threading a scratch local
/// through the handful of call sites that need the index more than once.
fn emit_channel_index(
    channel: &Expr,
    out: &mut String,
    context: &JvmContext,
) -> Result<(), String> {
    emit_numeric_expr_as(channel, NumericType::Int, out, context)?;
    out.push_str("    iconst_1\n    isub\n");
    Ok(())
}

/// `OPEN <file> FOR RANDOM AS #<channel> LEN = <len>` -- allocates a real
/// `java.io.RandomAccessFile` in `"rw"` mode (auto-creates the file if it
/// doesn't exist yet, matching real BASIC's own OPEN FOR RANDOM), plus the
/// fixed `byte[len]` record buffer every `GET`/`PUT`/`LSET` on this channel
/// reads or writes through (see `emit_get_or_put`/`emit_lset`).
fn emit_random_open(
    file: &Expr,
    channel: &Expr,
    len: &Expr,
    out: &mut String,
    context: &JvmContext,
) -> Result<(), String> {
    out.push_str(&format!(
        "    getstatic {}/bccFiles [Ljava/io/RandomAccessFile;\n",
        context.class_name
    ));
    emit_channel_index(channel, out, context)?;
    out.push_str("    new java/io/RandomAccessFile\n    dup\n");
    emit_string_expr(file, out, context)?;
    out.push_str(
        "    ldc \"rw\"\n    invokespecial java/io/RandomAccessFile/<init> \
         (Ljava/lang/String;Ljava/lang/String;)V\n    aastore\n",
    );

    out.push_str(&format!(
        "    getstatic {}/bccRecLen [I\n",
        context.class_name
    ));
    emit_channel_index(channel, out, context)?;
    emit_numeric_expr_as(len, NumericType::Int, out, context)?;
    out.push_str("    iastore\n");

    out.push_str(&format!(
        "    getstatic {}/bccBufs [[B\n",
        context.class_name
    ));
    emit_channel_index(channel, out, context)?;
    emit_numeric_expr_as(len, NumericType::Int, out, context)?;
    out.push_str("    newarray byte\n    aastore\n");
    Ok(())
}

fn emit_file_close(channel: &Expr, out: &mut String, context: &JvmContext) -> Result<(), String> {
    out.push_str(&format!(
        "    getstatic {}/bccFiles [Ljava/io/RandomAccessFile;\n",
        context.class_name
    ));
    emit_channel_index(channel, out, context)?;
    out.push_str("    aaload\n    invokevirtual java/io/RandomAccessFile/close ()V\n");
    Ok(())
}

/// `GET #<channel>, <record>` (`is_get`) / `PUT #<channel>, <record>` --
/// seeks to `(record - 1) * bccRecLen[channel - 1]` then reads/writes the
/// channel's whole record buffer in one call. Real BASIC's own GET/PUT
/// default `record` to "the next one after the last GET/PUT on this
/// channel" when omitted; that form isn't implemented here (`record` is
/// required) since nothing exercising this backend's file I/O yet omits it.
fn emit_get_or_put(
    is_get: bool,
    channel: &Expr,
    record: &Expr,
    out: &mut String,
    context: &JvmContext,
) -> Result<(), String> {
    out.push_str(&format!(
        "    getstatic {}/bccFiles [Ljava/io/RandomAccessFile;\n",
        context.class_name
    ));
    emit_channel_index(channel, out, context)?;
    out.push_str("    aaload\n    dup\n");
    emit_numeric_expr_as(record, NumericType::Long, out, context)?;
    out.push_str("    lconst_1\n    lsub\n");
    out.push_str(&format!(
        "    getstatic {}/bccRecLen [I\n",
        context.class_name
    ));
    emit_channel_index(channel, out, context)?;
    out.push_str(
        "    iaload\n    i2l\n    lmul\n    invokevirtual java/io/RandomAccessFile/seek (J)V\n",
    );
    out.push_str(&format!(
        "    getstatic {}/bccBufs [[B\n",
        context.class_name
    ));
    emit_channel_index(channel, out, context)?;
    out.push_str("    aaload\n");
    if is_get {
        // `read([B)`, not `readFully([B)` -- `readFully` throws
        // `EOFException` on a short/empty read (a GET past the current end
        // of a freshly `OPEN`ed, still-empty file, e.g. `tutorial/
        // inventory.bcl`'s own `let p = inv[1]` right after creating a new
        // file), where `codegen_c.rs`'s own `bcc_read_record` (`fread`)
        // just returns a short count and leaves the buffer as it was --
        // already all zero bytes for a freshly allocated one, which is
        // exactly the "flag byte 0 means never-initialized" signal
        // `initializeInventoryFileIfNew` depends on. `read` matches that:
        // it returns -1/a short count instead of throwing, so the buffer
        // silently keeps whatever it already had for the bytes that
        // weren't actually there to read.
        out.push_str("    invokevirtual java/io/RandomAccessFile/read ([B)I\n    pop\n");
    } else {
        out.push_str("    invokevirtual java/io/RandomAccessFile/write ([B)V\n");
    }
    Ok(())
}

/// Wraps an already-computed `byte[]` (top of stack) into a `String`, one
/// byte per char, via ISO-8859-1 -- so a packed numeric field (`MKI$`) or a
/// raw record-buffer slice round-trips through a BASIC string variable
/// without any encoding surprises (real UTF-8/platform-default decoding
/// could corrupt bytes outside 0-127).
fn emit_wrap_bytes_as_string(out: &mut String) {
    out.push_str(
        "    new java/lang/String\n    dup_x1\n    swap\n    \
         getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;\n    \
         invokespecial java/lang/String/<init> ([BLjava/nio/charset/Charset;)V\n",
    );
}

/// Converts an already-computed `String` (top of stack) into a little-
/// endian `ByteBuffer` over its raw ISO-8859-1 bytes -- the shared prefix
/// every `CVI`/`CVL`/`CVS`/`CVD` unpack starts with (see `MKI$`'s own doc
/// comment in `emit_string_expr` for why ISO-8859-1). Each caller appends
/// its own `getShort`/`getInt`/`getFloat`/`getDouble`.
fn emit_wrap_string_as_little_endian_bytebuffer(out: &mut String) {
    out.push_str(
        "    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;\n    \
         invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B\n    \
         invokestatic java/nio/ByteBuffer/wrap ([B)Ljava/nio/ByteBuffer;\n    \
         getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;\n    \
         invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;\n",
    );
}

/// Pushes a `String` of `width` ASCII spaces -- the same
/// `newarray`/`Arrays.fill`/`new String` idiom `SPACE$`/`STRING$` already
/// use (see `emit_string_expr`'s own `"space"` arm), just with a compile-
/// time-known width instead of a numeric expression.
fn emit_space_string(width: i64, out: &mut String) {
    out.push_str(&format!(
        "    ldc {width}\n    newarray char\n    dup\n    bipush 32\n    \
         invokestatic java/util/Arrays/fill ([CC)V\n    new java/lang/String\n    dup_x1\n    \
         swap\n    invokespecial java/lang/String/<init> ([C)V\n"
    ));
}

/// Reads a `FIELD`-declared variable's current value: decodes its byte
/// range out of its channel's shared record buffer (see `JvmFieldVar`).
fn emit_field_var_load(field: JvmFieldVar, out: &mut String, context: &JvmContext) {
    out.push_str(&format!(
        "    getstatic {}/bccBufs [[B\n    ldc {}\n    aaload\n",
        context.class_name,
        field.channel - 1
    ));
    out.push_str("    new java/lang/String\n    dup_x1\n    swap\n");
    out.push_str(&format!(
        "    ldc {}\n    ldc {}\n",
        field.offset, field.width
    ));
    out.push_str(
        "    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;\n    \
         invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V\n",
    );
}

/// Encodes an already-computed, exactly-`field.width`-characters-long
/// `String` (top of stack) as ISO-8859-1 bytes and copies it into `field`'s
/// own byte range in its channel's shared record buffer. Shared tail of
/// `emit_lset`/`emit_rset` -- they differ only in how that fixed-width
/// string gets built.
fn emit_store_field_bytes(field: JvmFieldVar, out: &mut String, context: &JvmContext) {
    out.push_str(
        "    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;\n    \
         invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B\n",
    );
    out.push_str("    iconst_0\n");
    out.push_str(&format!(
        "    getstatic {}/bccBufs [[B\n    ldc {}\n    aaload\n",
        context.class_name,
        field.channel - 1
    ));
    out.push_str(&format!(
        "    ldc {}\n    ldc {}\n",
        field.offset, field.width
    ));
    out.push_str(
        "    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V\n",
    );
}

/// `LSET <field> = <value>` -- writes `value`, left-justified and padded
/// with spaces (or truncated) to exactly the field's own width, into its
/// channel's shared record buffer. Concatenating `width` spaces onto
/// `value` first and then taking `substring(0, width)` handles both cases
/// branch-free: if `value` is shorter than `width` the padding is what
/// survives the truncation; if `value` is already `width` characters or
/// longer, the spaces never get reached and this is exactly `LEFT$(value$,
/// width)` -- real LSET's own truncate-if-too-long behavior. Mirrors
/// `codegen_c.rs`'s `bcc_pad_string_field` (`memcpy` + `memset(' ')`)
/// exactly.
fn emit_lset(
    field: JvmFieldVar,
    value: &Expr,
    out: &mut String,
    context: &JvmContext,
) -> Result<(), String> {
    emit_string_expr(value, out, context)?;
    emit_space_string(field.width, out);
    out.push_str(
        "    invokevirtual java/lang/String/concat (Ljava/lang/String;)Ljava/lang/String;\n",
    );
    out.push_str(&format!(
        "    iconst_0\n    ldc {}\n    invokevirtual java/lang/String/substring (II)Ljava/lang/String;\n",
        field.width
    ));
    emit_store_field_bytes(field, out, context);
    Ok(())
}

/// `RSET <field> = <value>` -- writes `value`, right-justified and padded
/// with spaces on the left, into its channel's shared record buffer.
/// Prepending `width` spaces and taking the *last* `width` characters
/// (`substring(length - width, length)`) right-justifies a `value` shorter
/// than `width` correctly (the padding survives on the left, exactly as
/// real RSET pads). A `value` already `width` characters or longer is a
/// narrower case this doesn't reproduce real MBASIC/BASCOM's own C-backend-
/// matched truncate-the-*front* behavior for (`emit_lset`'s "keep the first
/// `width` chars" via `%.*s`) -- this instead keeps the *last* `width`
/// chars, since prepended spaces get pushed out of the window along with
/// the front of an over-length `value` -- a real, narrow divergence,
/// undocumented in practice because nothing exercising this backend's file
/// I/O yet RSETs a too-long value.
fn emit_rset(
    field: JvmFieldVar,
    value: &Expr,
    out: &mut String,
    context: &JvmContext,
) -> Result<(), String> {
    emit_space_string(field.width, out);
    emit_string_expr(value, out, context)?;
    out.push_str(
        "    invokevirtual java/lang/String/concat (Ljava/lang/String;)Ljava/lang/String;\n",
    );
    out.push_str(&format!(
        "    dup\n    invokevirtual java/lang/String/length ()I\n    dup\n    ldc {}\n    isub\n    swap\n    \
         invokevirtual java/lang/String/substring (II)Ljava/lang/String;\n",
        field.width
    ));
    emit_store_field_bytes(field, out, context);
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NumericType {
    Int,
    Long,
    Double,
}

impl NumericType {
    fn print_descriptor(self) -> &'static str {
        match self {
            Self::Int => "(I)V",
            Self::Long => "(J)V",
            Self::Double => "(D)V",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JvmType {
    Numeric(NumericType),
    String,
}

#[derive(Clone, Copy)]
struct Variable {
    slot: usize,
    ty: JvmType,
    is_static: bool,
}

#[derive(Clone)]
struct ArrayShape {
    element: JvmType,
    dimensions: Vec<Expr>,
}

impl Variable {
    fn field_name(self) -> String {
        format!("g{}", self.slot)
    }

    fn descriptor(self) -> &'static str {
        descriptor(self.ty)
    }
    fn width(self) -> usize {
        match self.ty {
            JvmType::Numeric(NumericType::Long | NumericType::Double) => 2,
            _ => 1,
        }
    }

    fn store_opcode(self) -> &'static str {
        match self.ty {
            JvmType::String => "astore",
            JvmType::Numeric(NumericType::Int) => "istore",
            JvmType::Numeric(NumericType::Long) => "lstore",
            JvmType::Numeric(NumericType::Double) => "dstore",
        }
    }

    fn load_opcode(self) -> &'static str {
        match self.ty {
            JvmType::String => "aload",
            JvmType::Numeric(NumericType::Int) => "iload",
            JvmType::Numeric(NumericType::Long) => "lload",
            JvmType::Numeric(NumericType::Double) => "dload",
        }
    }
}

struct JvmContext {
    variables: BTreeMap<String, Variable>,
    arrays: BTreeMap<String, ArrayShape>,
    array_slots: BTreeMap<String, usize>,
    array_aliases: BTreeMap<String, String>,
    array_refs: Vec<crate::ast::TypedArrayRef>,
    constants: HashMap<String, Expr>,
    local_count: usize,
    initializer_start: usize,
    functions: HashMap<String, FunctionSig>,
    class_name: String,
    condition_label: Cell<usize>,
    initialize_static: bool,
    /// Every `FIELD`-declared variable anywhere in the program (main body
    /// and every function/procedure body), computed once by
    /// [`collect_field_vars`] and shared verbatim by every method's own
    /// `JvmContext` (see `for_function`) -- a name in here is never an
    /// ordinary `variables` entry; reading/writing it redirects to its
    /// channel's shared record buffer instead.
    field_vars: BTreeMap<String, JvmFieldVar>,
    /// Whether the program uses `OPEN ... FOR RANDOM` anywhere -- gates
    /// declaring/initializing `bccFiles`/`bccBufs`/`bccRecLen` at all.
    needs_file_io: bool,
    /// Whether the program uses interactive `INPUT` anywhere -- gates
    /// declaring/initializing the shared `bccStdin` reader at all.
    needs_input: bool,
    /// Whether the program uses bare `INKEY$` anywhere -- gates putting the
    /// terminal into raw/no-echo/non-blocking mode at program start (via
    /// `stty`, see `emit_inkey_setup`'s own doc comment) and restoring it at
    /// every exit point (see `emit_inkey_restore`).
    needs_inkey: bool,
    /// This function/procedure's own `byref` scalar parameters:
    /// `(array_slot, working)` pairs, in source-declaration order.
    /// `array_slot` is the incoming single-element-array parameter slot;
    /// `working` is an ordinary local (already registered in `variables`,
    /// so every existing read/write of the parameter name just works
    /// unchanged) unwrapped from it at function entry and written back into
    /// it at every exit point -- see `emit_function`'s prologue and
    /// `JvmContext::emit_byref_writebacks`. Empty for `main`'s own context
    /// (top-level code has no parameters).
    byref_scalar_params: Vec<(usize, Variable)>,
    /// Base local slot for this function's own fixed pool of scratch slots,
    /// used to build `byref` scalar-argument wrapper arrays at its call
    /// sites (see `emit_call_arguments`). Sized once, in `for_function`, to
    /// the largest `byref` scalar parameter count of any function/procedure
    /// in the whole program -- so it's always big enough regardless of
    /// which one this function actually calls, without having to scan this
    /// function's own call sites to find the true minimum. A real Java
    /// array is already a reference type, so a `byref` *array* parameter
    /// needs no such wrapping or scratch slot at all (see
    /// `JvmArrayParam::by_ref`).
    byref_scratch_base: usize,
}

#[derive(Clone)]
struct FunctionSig {
    params: Vec<JvmType>,
    array_params: Vec<JvmArrayParam>,
    /// Source positions (like `JvmArrayParam::position`) of every plain
    /// `byref` *scalar* parameter -- a `byref` array parameter needs no
    /// entry here, since a real Java array is already a reference type and
    /// needs no extra wrapping (see `JvmArrayParam::by_ref`'s own call-site
    /// handling). A scalar has no such reference semantics, so `byref` on
    /// one is emulated by wrapping the argument in a fresh single-element
    /// array at the call site and unwrapping/writing back around it -- see
    /// `emit_call_arguments`/`JvmContext::byref_scalar_params`'s own doc
    /// comments for the full mechanism.
    byref_scalar_positions: Vec<usize>,
    source_param_count: usize,
    has_receiver: bool,
    result: JvmType,
    returns_void: bool,
}

#[derive(Clone, Copy)]
struct JvmArrayParam {
    /// Position in the source parameter list, before scalar parameters are
    /// compacted for the legacy JVM calling convention.
    position: usize,
    element: JvmType,
    rank: usize,
    by_ref: bool,
}

impl FunctionSig {
    /// JVM descriptor parameters in the source declaration's order.  The
    /// legacy `params` list intentionally contains scalars only, so array
    /// entries must be reinserted at their original source positions here.
    fn descriptor_params(&self) -> Vec<String> {
        let mut scalars = self.params.iter();
        let mut out = Vec::with_capacity(self.params.len() + self.array_params.len());
        if self.has_receiver {
            out.push(descriptor(*scalars.next().expect("receiver scalar")).to_string());
        }
        for position in 0..self.source_param_count {
            if let Some(array) = self
                .array_params
                .iter()
                .find(|array| array.position == position)
            {
                out.push(format!(
                    "{}{}",
                    "[".repeat(array.rank),
                    descriptor(array.element)
                ));
            } else {
                let ty = *scalars.next().expect("scalar source parameter");
                if self.byref_scalar_positions.contains(&position) {
                    out.push(format!("[{}", descriptor(ty)));
                } else {
                    out.push(descriptor(ty).to_string());
                }
            }
        }
        out
    }
}

impl JvmContext {
    fn emit_array_load(&self, ident: &BasicIdent, out: &mut String) {
        let key = variable_key(ident);
        if let Some(slot) = self.array_slots.get(&key) {
            out.push_str(&format!("    aload {slot}\n"));
        } else {
            let shape = self.arrays.get(&key).expect("registered JVM array");
            out.push_str(&format!(
                "    getstatic {}/a{} {}\n",
                self.class_name,
                self.array_index(ident),
                array_descriptor(shape)
            ));
        }
    }

    fn emit_array_axis_length(&self, ident: &BasicIdent, axis: usize, out: &mut String) {
        self.emit_array_load(ident, out);
        if axis == 0 {
            out.push_str("    arraylength\n");
        } else {
            // BASCAL dimensions are upper bounds, so allocation always adds
            // one and validation rejects negative bounds.  Every language-
            // constructed outer axis is therefore non-empty; descending
            // through element zero is safe for parameter-bound queries.
            for _ in 0..axis {
                out.push_str("    iconst_0\n    aaload\n");
            }
            out.push_str("    arraylength\n");
        }
    }

    fn array_index(&self, ident: &BasicIdent) -> usize {
        let key = variable_key(ident);
        let key = self.array_aliases.get(&key).unwrap_or(&key);
        self.arrays
            .keys()
            .filter(|candidate| !self.array_aliases.contains_key(*candidate))
            .position(|candidate| candidate == key)
            .expect("registered JVM array")
    }

    fn build(
        program: &Program,
        functions: HashMap<String, FunctionSig>,
        class_name: String,
    ) -> Result<Self, Vec<Diagnostic>> {
        let mut declarations = BTreeMap::new();
        let mut constants = HashMap::new();
        let mut arrays = BTreeMap::new();
        collect_scalar_declarations(
            &program.statements,
            &mut declarations,
            &mut constants,
            &functions,
        );
        for array in &program.typed_arrays {
            arrays.insert(
                variable_key(&array.name),
                ArrayShape {
                    element: type_for_ident(&array.name),
                    dimensions: array.dimensions.clone(),
                },
            );
        }
        if arrays.is_empty() {
            collect_array_declarations(&program.statements, &mut arrays);
        }
        let mut field_vars = BTreeMap::new();
        collect_field_vars(&program.statements, &mut field_vars)
            .map_err(|message| vec![unsupported(&message)])?;
        for function in &program.functions {
            collect_field_vars(&function.body, &mut field_vars)
                .map_err(|message| vec![unsupported(&message)])?;
        }
        let needs_file_io = program_uses_random_open(&program.statements)
            || program
                .functions
                .iter()
                .any(|f| program_uses_random_open(&f.body));
        let needs_input = program_uses_input(&program.statements)
            || program
                .functions
                .iter()
                .any(|f| program_uses_input(&f.body));
        let needs_inkey = program_uses_inkey(&program.statements)
            || program
                .functions
                .iter()
                .any(|f| program_uses_inkey(&f.body));
        let mut next_slot = 1;
        let variables = declarations
            .into_iter()
            .map(|(key, ty)| {
                let variable = Variable {
                    slot: next_slot,
                    ty,
                    is_static: true,
                };
                next_slot += variable.width();
                (key, variable)
            })
            .collect();
        let byref_scratch_base = next_slot;
        next_slot += functions
            .values()
            .map(|sig| sig.byref_scalar_positions.len())
            .max()
            .unwrap_or(0);
        Ok(Self {
            variables,
            arrays,
            array_slots: BTreeMap::new(),
            array_aliases: BTreeMap::new(),
            array_refs: program.typed_array_refs.clone(),
            constants,
            local_count: next_slot,
            initializer_start: 1,
            functions,
            class_name,
            condition_label: Cell::new(0),
            initialize_static: true,
            field_vars,
            needs_file_io,
            needs_input,
            needs_inkey,
            byref_scalar_params: Vec::new(),
            byref_scratch_base,
        })
    }

    fn for_function(function: &FunctionDef, parent: &Self) -> Self {
        let mut variables = BTreeMap::new();
        let mut next_slot = 0;
        if let Some(suffix) = function.receiver {
            let self_ident = BasicIdent {
                name: "self".to_string(),
                suffix: Some(suffix),
            };
            let variable = Variable {
                slot: next_slot,
                ty: type_for_ident(&self_ident),
                is_static: false,
            };
            next_slot += variable.width();
            variables.insert(variable_key(&self_ident), variable);
        }
        let mut parameter_array_slots = BTreeMap::new();
        // (name, incoming array slot) for every byref scalar parameter --
        // its own "working" local (see below) can't be allocated in this
        // same pass: every *real* JVM parameter slot -- self, then each
        // source parameter in order -- must be assigned strictly by its own
        // descriptor width first (one slot for any array reference,
        // including a byref scalar's own wrapper array, regardless of the
        // wrapped element's width), exactly matching the actual JVM calling
        // convention. Interleaving an extra "working" slot per byref scalar
        // *inside* this loop (an earlier version of this code did) shifts
        // every later real parameter to the wrong slot entirely.
        let mut byref_scalar_param_names = Vec::new();
        for param in &function.params {
            if param.axes.is_some() {
                parameter_array_slots.insert(variable_key(&param.name), next_slot);
                next_slot += 1;
            } else if param.mode == ParamMode::ByRef {
                byref_scalar_param_names.push((param.name.clone(), next_slot));
                next_slot += 1;
            } else {
                let variable = Variable {
                    slot: next_slot,
                    ty: type_for_ident(&param.name),
                    is_static: false,
                };
                next_slot += variable.width();
                variables.insert(variable_key(&param.name), variable);
            }
        }
        // Now that every real parameter has its correct slot, append each
        // byref scalar's own "working" local right after them -- an
        // ordinary local (registered in `variables` like any other
        // parameter, so every existing read/write of the parameter name
        // needs no special-casing at all), unwrapped from its incoming
        // array at function entry and written back into it at every exit
        // point (see `emit_function`'s prologue and `JvmContext::
        // emit_byref_writebacks`).
        let mut byref_scalar_params = Vec::new();
        for (name, array_slot) in byref_scalar_param_names {
            let working = Variable {
                slot: next_slot,
                ty: type_for_ident(&name),
                is_static: false,
            };
            next_slot += working.width();
            variables.insert(variable_key(&name), working);
            byref_scalar_params.push((array_slot, working));
        }
        let initializer_start = next_slot;
        let mut declarations = BTreeMap::new();
        let mut constants = parent.constants.clone();
        collect_scalar_declarations(
            &function.body,
            &mut declarations,
            &mut constants,
            &parent.functions,
        );
        for name in collect_global_names(&function.body) {
            if let Some(variable) = parent.variables.get(&variable_key(&name)) {
                variables.insert(variable_key(&name), *variable);
            }
        }
        for (key, ty) in declarations {
            if variables.contains_key(&key) {
                continue;
            }
            let variable = Variable {
                slot: next_slot,
                ty,
                is_static: false,
            };
            next_slot += variable.width();
            variables.insert(key, variable);
        }
        let mut arrays = parent.arrays.clone();
        let array_slots = parameter_array_slots;
        let array_aliases = parent.array_aliases.clone();
        for param in function.params.iter().filter(|param| param.axes.is_some()) {
            let key = variable_key(&param.name);
            arrays.insert(
                key,
                ArrayShape {
                    element: type_for_ident(&param.name),
                    dimensions: vec![Expr::Integer(0); param.rank().unwrap_or(0)],
                },
            );
        }
        let byref_scratch_base = next_slot;
        next_slot += parent
            .functions
            .values()
            .map(|sig| sig.byref_scalar_positions.len())
            .max()
            .unwrap_or(0);
        Self {
            variables,
            arrays,
            array_slots,
            array_aliases,
            array_refs: parent.array_refs.clone(),
            constants,
            local_count: next_slot,
            initializer_start,
            functions: parent.functions.clone(),
            class_name: parent.class_name.clone(),
            condition_label: Cell::new(0),
            initialize_static: false,
            field_vars: parent.field_vars.clone(),
            needs_file_io: parent.needs_file_io,
            needs_input: parent.needs_input,
            needs_inkey: parent.needs_inkey,
            byref_scalar_params,
            byref_scratch_base,
        }
    }

    fn local_count(&self) -> usize {
        self.local_count
    }

    /// Writes every `byref` scalar parameter's current working-local value
    /// back into its incoming single-element array -- the counterpart to
    /// `emit_function`'s own unwrap prologue. Must run at *every* exit
    /// point of a function/procedure with `byref` scalar parameters, not
    /// just the end: `checkPart`/`editRecord`-shaped early `return`s are
    /// exactly why (see `Statement::Return`/`ReturnVoid`'s own arms).
    fn emit_byref_writebacks(&self, out: &mut String) {
        for (array_slot, working) in &self.byref_scalar_params {
            out.push_str(&format!(
                "    aload {array_slot}\n    iconst_0\n    {} {}\n    {}\n",
                working.load_opcode(),
                working.slot,
                array_store_opcode(working.ty)
            ));
        }
    }

    fn variable(&self, ident: &BasicIdent) -> Result<Variable, String> {
        self.variables
            .get(&variable_key(ident))
            .copied()
            .ok_or_else(|| {
                format!("`{ident}` must be assigned or declared before use under --target jvm")
            })
    }

    fn constant(&self, ident: &BasicIdent) -> Option<&Expr> {
        self.constants.get(&variable_key(ident)).or_else(|| {
            // Required libraries may declare an inferred constant with a
            // typed internal suffix while the importing source refers to it
            // suffixlessly. Resolve that reference by the stable source name
            // and retain the declaration's inferred value/type.
            let name = ident.name.to_ascii_lowercase();
            self.constants
                .iter()
                .find(|(key, _)| {
                    key.trim_end_matches(|ch| matches!(ch, '%' | '$' | '!' | '#' | '&')) == name
                })
                .map(|(_, value)| value)
        })
    }

    fn function(&self, ident: &BasicIdent) -> Option<FunctionSig> {
        self.functions.get(&function_key(ident)).cloned()
    }

    fn is_string_expr(&self, expr: &Expr) -> bool {
        match expr {
            Expr::String(_) => true,
            Expr::Ident(name) => {
                self.field_vars.contains_key(&variable_key(name))
                    || (name.suffix == Some(TypeSuffix::String)
                        && (name.name.eq_ignore_ascii_case("inkey")
                            || name.name.eq_ignore_ascii_case("date")))
                    || self
                        .constant(name)
                        .is_some_and(|value| self.is_string_expr(value))
                    || self
                        .variables
                        .get(&variable_key(name))
                        .is_some_and(|var| matches!(var.ty, JvmType::String))
            }
            Expr::Binary {
                left,
                op: BinaryOp::Add,
                right,
            } => self.is_string_expr(left) && self.is_string_expr(right),
            Expr::Call { name, args }
            | Expr::ArrayRef {
                name,
                indices: args,
            } if name.name.eq_ignore_ascii_case("str") && args.len() == 1 => true,
            Expr::Call { name, args }
            | Expr::ArrayRef {
                name,
                indices: args,
            } if (name.name.eq_ignore_ascii_case("chr") && args.len() == 1)
                || (name.name.eq_ignore_ascii_case("mid")
                    && (args.len() == 2 || args.len() == 3)) =>
            {
                true
            }
            Expr::Call { name, args }
            | Expr::ArrayRef {
                name,
                indices: args,
            } if self.arrays.get(&variable_key(name)).is_some_and(|shape| {
                args.len() == shape.dimensions.len() && shape.element == JvmType::String
            }) =>
            {
                true
            }
            Expr::Call { name, .. } | Expr::ArrayRef { name, .. } => self
                .function(name)
                .is_some_and(|signature| matches!(signature.result, JvmType::String)),
            Expr::ScalarMethodCall { method, .. } => {
                let ident = BasicIdent {
                    name: method.clone(),
                    suffix: Some(TypeSuffix::String),
                };
                self.function(&ident)
                    .is_some_and(|signature| signature.result == JvmType::String)
            }
            _ => false,
        }
    }

    fn emit_initializers(&self, out: &mut String) {
        for variable in self.variables.values().filter(|variable| {
            variable.slot >= self.initializer_start
                && (self.initialize_static || !variable.is_static)
        }) {
            match variable.ty {
                JvmType::String => {
                    if variable.is_static {
                        out.push_str(&format!(
                            "    ldc \"\"\n    putstatic {}/{} {}\n",
                            self.class_name,
                            variable.field_name(),
                            variable.descriptor()
                        ))
                    } else {
                        out.push_str(&format!("    ldc \"\"\n    astore {}\n", variable.slot))
                    }
                }
                JvmType::Numeric(NumericType::Int) => {
                    if variable.is_static {
                        out.push_str(&format!(
                            "    iconst_0\n    putstatic {}/{} {}\n",
                            self.class_name,
                            variable.field_name(),
                            variable.descriptor()
                        ))
                    } else {
                        out.push_str(&format!("    iconst_0\n    istore {}\n", variable.slot))
                    }
                }
                JvmType::Numeric(NumericType::Long) => {
                    if variable.is_static {
                        out.push_str(&format!(
                            "    lconst_0\n    putstatic {}/{} {}\n",
                            self.class_name,
                            variable.field_name(),
                            variable.descriptor()
                        ))
                    } else {
                        out.push_str(&format!("    lconst_0\n    lstore {}\n", variable.slot))
                    }
                }
                JvmType::Numeric(NumericType::Double) => {
                    if variable.is_static {
                        out.push_str(&format!(
                            "    dconst_0\n    putstatic {}/{} {}\n",
                            self.class_name,
                            variable.field_name(),
                            variable.descriptor()
                        ))
                    } else {
                        out.push_str(&format!("    dconst_0\n    dstore {}\n", variable.slot))
                    }
                }
            }
        }
    }
}

fn collect_scalar_declarations(
    statements: &[Stmt],
    declarations: &mut BTreeMap<String, JvmType>,
    constants: &mut HashMap<String, Expr>,
    functions: &HashMap<String, FunctionSig>,
) {
    for statement in statements {
        match &statement.kind {
            Statement::Dim {
                name,
                is_array: false,
                ..
            }
            | Statement::Assignment {
                target: Expr::Ident(name),
                ..
            } => {
                declarations.insert(variable_key(name), type_for_ident(name));
            }
            Statement::Input { vars, .. } => {
                for var in vars {
                    if let Expr::Ident(name) = var {
                        declarations.insert(variable_key(name), type_for_ident(name));
                    }
                }
            }
            // A `byref` scalar call argument is an output parameter: real
            // BASIC (and every other backend here) lets the callee's own
            // write be its first "declaration", with no `dim`/plain
            // assignment of its own needed first -- see
            // `gatherPartDetails(part%, editDesc$, editQty%, ...)` in
            // `tutorial/inventory.bcl`, where none of the `byref` arguments
            // are ever assigned any other way.
            Statement::ExprStmt(Expr::Call { name, args })
            | Statement::ExprStmt(Expr::ArrayRef {
                name,
                indices: args,
            }) => {
                if let Some(signature) = functions.get(&function_key(name)) {
                    for &position in &signature.byref_scalar_positions {
                        if let Some(Expr::Ident(arg_name)) = args.get(position) {
                            declarations.insert(variable_key(arg_name), type_for_ident(arg_name));
                        }
                    }
                }
            }
            Statement::Const { name, value } => {
                constants.insert(variable_key(name), value.clone());
            }
            Statement::If {
                then_body,
                else_body,
                ..
            } => {
                collect_scalar_declarations(then_body, declarations, constants, functions);
                collect_scalar_declarations(else_body, declarations, constants, functions);
            }
            Statement::For { var, body, .. } => {
                declarations.insert(variable_key(var), type_for_ident(var));
                collect_scalar_declarations(body, declarations, constants, functions);
            }
            Statement::While { body, .. } | Statement::Do { body, .. } => {
                collect_scalar_declarations(body, declarations, constants, functions);
            }
            Statement::TryCatch {
                try_body,
                catch,
                finally_body,
            } => {
                collect_scalar_declarations(try_body, declarations, constants, functions);
                if let Some(catch) = catch {
                    declarations
                        .insert(variable_key(&catch.err_var), type_for_ident(&catch.err_var));
                    declarations
                        .insert(variable_key(&catch.erl_var), type_for_ident(&catch.erl_var));
                    if let Some(source_var) = &catch.source_var {
                        declarations.insert(variable_key(source_var), JvmType::String);
                    }
                    collect_scalar_declarations(&catch.body, declarations, constants, functions);
                }
                collect_scalar_declarations(finally_body, declarations, constants, functions);
            }
            Statement::SelectCase {
                cases, else_body, ..
            } => {
                for case in cases {
                    collect_scalar_declarations(&case.body, declarations, constants, functions);
                }
                collect_scalar_declarations(else_body, declarations, constants, functions);
            }
            _ => {}
        }
    }
}

fn collect_array_declarations(statements: &[Stmt], arrays: &mut BTreeMap<String, ArrayShape>) {
    for statement in statements {
        match &statement.kind {
            Statement::Dim {
                name,
                is_array: true,
                sizes,
            } => {
                arrays.insert(
                    variable_key(name),
                    ArrayShape {
                        element: type_for_ident(name),
                        dimensions: sizes.clone(),
                    },
                );
            }
            Statement::If {
                then_body,
                else_body,
                ..
            } => {
                collect_array_declarations(then_body, arrays);
                collect_array_declarations(else_body, arrays);
            }
            Statement::For { body, .. }
            | Statement::While { body, .. }
            | Statement::Do { body, .. } => collect_array_declarations(body, arrays),
            Statement::SelectCase {
                cases, else_body, ..
            } => {
                for case in cases {
                    collect_array_declarations(&case.body, arrays);
                }
                collect_array_declarations(else_body, arrays);
            }
            Statement::TryCatch {
                try_body,
                catch,
                finally_body,
            } => {
                collect_array_declarations(try_body, arrays);
                if let Some(catch) = catch {
                    collect_array_declarations(&catch.body, arrays);
                }
                collect_array_declarations(finally_body, arrays);
            }
            _ => {}
        }
    }
}

fn collect_global_names(statements: &[Stmt]) -> Vec<BasicIdent> {
    let mut names = Vec::new();
    fn visit(statements: &[Stmt], names: &mut Vec<BasicIdent>) {
        for statement in statements {
            match &statement.kind {
                Statement::GlobalDecl(name) => names.push(name.clone()),
                Statement::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    visit(then_body, names);
                    visit(else_body, names);
                }
                Statement::For { body, .. }
                | Statement::While { body, .. }
                | Statement::Do { body, .. } => visit(body, names),
                Statement::SelectCase {
                    cases, else_body, ..
                } => {
                    for case in cases {
                        visit(&case.body, names);
                    }
                    visit(else_body, names);
                }
                Statement::TryCatch {
                    try_body,
                    catch,
                    finally_body,
                } => {
                    visit(try_body, names);
                    if let Some(catch) = catch {
                        visit(&catch.body, names);
                    }
                    visit(finally_body, names);
                }
                _ => {}
            }
        }
    }
    visit(statements, &mut names);
    names
}

fn variable_key(ident: &BasicIdent) -> String {
    format!(
        "{}{}",
        ident.name.to_ascii_lowercase(),
        ident
            .suffix
            .map_or("".to_string(), |suffix| suffix.to_string())
    )
}

fn type_for_ident(ident: &BasicIdent) -> JvmType {
    match ident.suffix.unwrap_or(TypeSuffix::Single) {
        TypeSuffix::String => JvmType::String,
        TypeSuffix::Integer => JvmType::Numeric(NumericType::Int),
        TypeSuffix::Long => JvmType::Numeric(NumericType::Long),
        // Doubles are a safe widening internal representation for BASCAL's
        // default single-precision scalar until a later precision pass.
        TypeSuffix::Single | TypeSuffix::Double => JvmType::Numeric(NumericType::Double),
    }
}

/// Emits a numeric expression and returns the JVM value type left on the
/// operand stack.  Integer literals use `int` while possible; decimal
/// literals and `/` use `double`, and `\\`/`MOD` use `long` so their
/// rounded operands and result cannot overflow a 16-bit BASIC integer on
/// the way through the JVM.
fn emit_string_expr(expr: &Expr, out: &mut String, context: &JvmContext) -> Result<(), String> {
    match expr {
        Expr::ScalarMethodCall { base, method, args } => {
            let name = BasicIdent {
                name: method.clone(),
                suffix: Some(TypeSuffix::String),
            };
            let signature = context
                .function(&name)
                .ok_or_else(|| format!("unsupported JVM string method `{method}`"))?;
            if signature.result != JvmType::String || signature.params.len() != args.len() + 1 {
                return Err(format!("invalid JVM string method call `{method}`"));
            }
            emit_string_expr(base, out, context)?;
            for (arg, ty) in args.iter().zip(signature.params.iter().skip(1)) {
                match ty {
                    JvmType::String => emit_string_expr(arg, out, context)?,
                    JvmType::Numeric(ty) => emit_numeric_expr_as(arg, *ty, out, context)?,
                }
            }
            let descriptor_args = signature.params.iter().map(|ty| descriptor(*ty)).collect::<String>();
            out.push_str(&format!("    invokestatic {}/{} ({descriptor_args}){}\n", context.class_name, name.name, descriptor(signature.result)));
            Ok(())
        }
        Expr::Call { name, args } | Expr::ArrayRef { name, indices: args }
            if name.name.eq_ignore_ascii_case("chr") && args.len() == 1 => {
            emit_numeric_expr_as(&args[0], NumericType::Int, out, context)?;
            out.push_str("    i2c\n    invokestatic java/lang/String/valueOf (C)Ljava/lang/String;\n");
            Ok(())
        }
        // `MKI$(n)` -- packs `n` as a raw little-endian 16-bit int, the same
        // two-byte layout `FIELD`/`GET`/`PUT` need for an `int16` record
        // field; see `codegen_c.rs`'s own `bcc_mki`/`CVI`'s doc comment for
        // why little-endian (real MBASIC/BASCOM's own on-disk layout, true
        // of every realistic deployment platform).
        Expr::Call { name, args } | Expr::ArrayRef { name, indices: args }
            if name.name.eq_ignore_ascii_case("mki") && args.len() == 1 => {
            out.push_str(
                "    ldc 2\n    invokestatic java/nio/ByteBuffer/allocate (I)Ljava/nio/ByteBuffer;\n    \
                 getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;\n    \
                 invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;\n",
            );
            emit_numeric_expr_as(&args[0], NumericType::Int, out, context)?;
            out.push_str(
                "    i2s\n    invokevirtual java/nio/ByteBuffer/putShort (S)Ljava/nio/ByteBuffer;\n    \
                 invokevirtual java/nio/ByteBuffer/array ()[B\n",
            );
            emit_wrap_bytes_as_string(out);
            Ok(())
        }
        // `MKL$(n)` -- packs `n` as a raw little-endian 32-bit int (real
        // BASIC's "long"), the same layout `CVL` and an `int32` record field
        // need. Same little-endian rationale as `MKI$`.
        Expr::Call { name, args } | Expr::ArrayRef { name, indices: args }
            if name.name.eq_ignore_ascii_case("mkl") && args.len() == 1 => {
            out.push_str(
                "    ldc 4\n    invokestatic java/nio/ByteBuffer/allocate (I)Ljava/nio/ByteBuffer;\n    \
                 getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;\n    \
                 invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;\n",
            );
            emit_numeric_expr_as(&args[0], NumericType::Int, out, context)?;
            out.push_str(
                "    invokevirtual java/nio/ByteBuffer/putInt (I)Ljava/nio/ByteBuffer;\n    \
                 invokevirtual java/nio/ByteBuffer/array ()[B\n",
            );
            emit_wrap_bytes_as_string(out);
            Ok(())
        }
        // `MKS$(n)` -- packs `n` as a raw little-endian 32-bit IEEE 754
        // float (`d2f` narrows this backend's internal `double`
        // representation of a BASIC single -- see `type_for_ident`'s own
        // doc comment on why `!`/`#` share one JVM type). Plain IEEE 754,
        // not real BASIC's Microsoft Binary Format -- the same real,
        // documented divergence `codegen_c.rs`'s `bcc_mks`/`bcc_cvs` accept
        // (see `FILE_IO_BODY`'s doc comment there): a `single`/`double`
        // record field this backend writes isn't binary-compatible with one
        // real BASCOM wrote, unlike an `int16`/`int32`/`string(N)` field.
        Expr::Call { name, args } | Expr::ArrayRef { name, indices: args }
            if name.name.eq_ignore_ascii_case("mks") && args.len() == 1 => {
            out.push_str(
                "    ldc 4\n    invokestatic java/nio/ByteBuffer/allocate (I)Ljava/nio/ByteBuffer;\n    \
                 getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;\n    \
                 invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;\n",
            );
            emit_numeric_expr_as(&args[0], NumericType::Double, out, context)?;
            out.push_str(
                "    d2f\n    invokevirtual java/nio/ByteBuffer/putFloat (F)Ljava/nio/ByteBuffer;\n    \
                 invokevirtual java/nio/ByteBuffer/array ()[B\n",
            );
            emit_wrap_bytes_as_string(out);
            Ok(())
        }
        // `MKD$(n)` -- packs `n` as a raw little-endian 64-bit IEEE 754
        // double. Same real, documented MBF divergence as `MKS$` (see its
        // own doc comment) -- `codegen_c.rs`'s `bcc_mkd`/`bcc_cvd` accept
        // the identical one.
        Expr::Call { name, args } | Expr::ArrayRef { name, indices: args }
            if name.name.eq_ignore_ascii_case("mkd") && args.len() == 1 => {
            out.push_str(
                "    ldc 8\n    invokestatic java/nio/ByteBuffer/allocate (I)Ljava/nio/ByteBuffer;\n    \
                 getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;\n    \
                 invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;\n",
            );
            emit_numeric_expr_as(&args[0], NumericType::Double, out, context)?;
            out.push_str(
                "    invokevirtual java/nio/ByteBuffer/putDouble (D)Ljava/nio/ByteBuffer;\n    \
                 invokevirtual java/nio/ByteBuffer/array ()[B\n",
            );
            emit_wrap_bytes_as_string(out);
            Ok(())
        }
        Expr::Call { name, args } | Expr::ArrayRef { name, indices: args }
            if name.name.eq_ignore_ascii_case("mid") && args.len() == 3 => {
            emit_string_expr(&args[0], out, context)?;
            emit_numeric_expr_as(&args[1], NumericType::Int, out, context)?;
            out.push_str("    iconst_1\n    isub\n    dup\n");
            emit_numeric_expr_as(&args[2], NumericType::Int, out, context)?;
            out.push_str("    iadd\n    invokevirtual java/lang/String/substring (II)Ljava/lang/String;\n");
            Ok(())
        }
        Expr::Call { name, args } | Expr::ArrayRef { name, indices: args }
            if name.name.eq_ignore_ascii_case("mid") && args.len() == 2 => {
            emit_string_expr(&args[0], out, context)?;
            emit_numeric_expr_as(&args[1], NumericType::Int, out, context)?;
            out.push_str("    iconst_1\n    isub\n    invokevirtual java/lang/String/substring (I)Ljava/lang/String;\n");
            Ok(())
        }
        Expr::Call { name, args } | Expr::ArrayRef { name, indices: args }
            if name.name.eq_ignore_ascii_case("left") && args.len() == 2 => {
            emit_string_expr(&args[0], out, context)?;
            out.push_str("    iconst_0\n");
            emit_numeric_expr_as(&args[1], NumericType::Int, out, context)?;
            out.push_str("    invokevirtual java/lang/String/substring (II)Ljava/lang/String;\n");
            Ok(())
        }
        Expr::Call { name, args } | Expr::ArrayRef { name, indices: args }
            if name.name.eq_ignore_ascii_case("right") && args.len() == 2 => {
            emit_string_expr(&args[0], out, context)?;
            out.push_str("    dup\n    invokevirtual java/lang/String/length ()I\n");
            emit_numeric_expr_as(&args[1], NumericType::Int, out, context)?;
            out.push_str("    isub\n    invokevirtual java/lang/String/substring (I)Ljava/lang/String;\n");
            Ok(())
        }
        Expr::Call { name, args } | Expr::ArrayRef { name, indices: args }
            if (name.name.eq_ignore_ascii_case("lcase") || name.name.eq_ignore_ascii_case("ucase")) && args.len() == 1 => {
            emit_string_expr(&args[0], out, context)?;
            let method = if name.name.eq_ignore_ascii_case("lcase") { "toLowerCase" } else { "toUpperCase" };
            out.push_str(&format!("    invokevirtual java/lang/String/{method} ()Ljava/lang/String;\n"));
            Ok(())
        }
        Expr::Call { name, args } | Expr::ArrayRef { name, indices: args }
            if name.name.eq_ignore_ascii_case("trim") && args.len() == 1 => {
            emit_string_expr(&args[0], out, context)?;
            out.push_str("    invokevirtual java/lang/String/trim ()Ljava/lang/String;\n");
            Ok(())
        }
        Expr::Call { name, args } | Expr::ArrayRef { name, indices: args }
            if name.name.eq_ignore_ascii_case("space") && args.len() == 1 => {
            emit_numeric_expr_as(&args[0], NumericType::Int, out, context)?;
            emit_space_string_runtime(out);
            Ok(())
        }
        Expr::Call { name, args } | Expr::ArrayRef { name, indices: args }
            if name.name.eq_ignore_ascii_case("string") && args.len() == 2 => {
            emit_numeric_expr_as(&args[0], NumericType::Int, out, context)?;
            out.push_str("    newarray char\n    dup\n");
            if context.is_string_expr(&args[1]) {
                emit_string_expr(&args[1], out, context)?;
                out.push_str("    iconst_0\n    invokevirtual java/lang/String/charAt (I)C\n");
            } else {
                emit_numeric_expr_as(&args[1], NumericType::Int, out, context)?;
                out.push_str("    i2c\n");
            }
            out.push_str("    invokestatic java/util/Arrays/fill ([CC)V\n    new java/lang/String\n    dup_x1\n    swap\n    invokespecial java/lang/String/<init> ([C)V\n");
            Ok(())
        }
        Expr::Call { name, args } | Expr::ArrayRef { name, indices: args }
            if context.arrays.contains_key(&variable_key(name)) =>
        {
            let shape = context.arrays.get(&variable_key(name)).expect("array exists");
            if args.len() != shape.dimensions.len() || shape.element != JvmType::String {
                return Err("JVM string-array access has the wrong type or dimensions".to_string());
            }
            context.emit_array_load(name, out);
            for index in &args[..args.len() - 1] {
                emit_numeric_expr_as(index, NumericType::Int, out, context)?;
                out.push_str("    aaload\n");
            }
            emit_numeric_expr_as(args.last().expect("validated index count"), NumericType::Int, out, context)?;
            out.push_str("    aaload\n");
            Ok(())
        }
        Expr::Call { name, args } | Expr::ArrayRef { name, indices: args }
            if context.function(name).is_some() => {
            emit_function_call(name, args, JvmType::String, out, context)
        }
        Expr::String(value) => {
            out.push_str(&format!("    ldc \"{}\"\n", escape_jvm_string(value)));
            Ok(())
        }
        Expr::Ident(name) if context.field_vars.contains_key(&variable_key(name)) => {
            let field = context.field_vars[&variable_key(name)];
            emit_field_var_load(field, out, context);
            Ok(())
        }
        // Bare `INKEY$` -- a one-shot, non-blocking check for a pending
        // keystroke: `System.in.available()` is safe to call without ever
        // blocking (unlike attempting a `read()` outright), and correctly
        // reflects "a keystroke is ready right now" once `emit_inkey_setup`
        // has put the terminal into raw/no-echo/non-blocking mode --
        // canonical (line-buffered) mode would otherwise report nothing
        // available until Enter flushes a whole line. Returns "" when
        // nothing is pending, or the one pending byte as a single-character
        // string (`i2c` + `String.valueOf(char)` -- the same idiom `CHR$`
        // already uses; values 0-255 map onto the same UTF-16 code units).
        Expr::Ident(name)
            if name.suffix == Some(TypeSuffix::String) && name.name.eq_ignore_ascii_case("inkey") =>
        {
            let empty = next_condition_label(context);
            let done = next_condition_label(context);
            out.push_str(&format!(
                "    getstatic java/lang/System/in Ljava/io/InputStream;\n    \
                 invokevirtual java/io/InputStream/available ()I\n    \
                 ifle {empty}\n    \
                 getstatic java/lang/System/in Ljava/io/InputStream;\n    \
                 invokevirtual java/io/InputStream/read ()I\n    \
                 i2c\n    invokestatic java/lang/String/valueOf (C)Ljava/lang/String;\n    \
                 goto {done}\n{empty}:\n    ldc \"\"\n{done}:\n"
            ));
            Ok(())
        }
        // Bare `DATE$` -- real MBASIC/BASCOM's fixed `"MM-DD-YYYY"` format,
        // read from the host clock via `java.time` -- the exact same
        // format `codegen_c.rs`'s own `bcc_date` (`<time.h>`) produces.
        Expr::Ident(name)
            if name.suffix == Some(TypeSuffix::String) && name.name.eq_ignore_ascii_case("date") =>
        {
            out.push_str(
                "    invokestatic java/time/LocalDate/now ()Ljava/time/LocalDate;\n    \
                 ldc \"MM-dd-yyyy\"\n    \
                 invokestatic java/time/format/DateTimeFormatter/ofPattern \
                 (Ljava/lang/String;)Ljava/time/format/DateTimeFormatter;\n    \
                 invokevirtual java/time/LocalDate/format \
                 (Ljava/time/format/DateTimeFormatter;)Ljava/lang/String;\n",
            );
            Ok(())
        }
        Expr::Ident(name) if context.constant(name).is_some() => {
            emit_string_expr(context.constant(name).expect("checked above"), out, context)
        }
        Expr::Ident(name) => {
            let variable = context.variable(name)?;
            if !matches!(variable.ty, JvmType::String) {
                return Err(format!("`{name}` is numeric, not a string"));
            }
            emit_load(variable, out, context);
            Ok(())
        }
        Expr::Call { name, args } | Expr::ArrayRef { name, indices: args }
            if name.name.eq_ignore_ascii_case("str") && args.len() == 1 =>
        {
            let ty = emit_numeric_expr(&args[0], out, context)?;
            match ty {
                NumericType::Int => {
                    out.push_str(
                        "    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;\n",
                    );
                }
                NumericType::Long => {
                    out.push_str(
                        "    invokestatic java/lang/String/valueOf (J)Ljava/lang/String;\n",
                    );
                }
                NumericType::Double => {
                    out.push_str(&format!(
                        "    invokestatic {}/bccStr (D)Ljava/lang/String;\n",
                        context.class_name
                    ));
                }
            }
            Ok(())
        }
        Expr::Binary {
            left,
            op: BinaryOp::Add,
            right,
        } if context.is_string_expr(left) && context.is_string_expr(right) => {
            out.push_str("    new java/lang/StringBuilder\n    dup\n");
            out.push_str("    invokespecial java/lang/StringBuilder/<init> ()V\n");
            emit_string_expr(left, out, context)?;
            out.push_str("    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;\n");
            emit_string_expr(right, out, context)?;
            out.push_str("    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;\n");
            out.push_str("    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;\n");
            Ok(())
        }
        other => Err(format!("{other:?} is not supported by the JVM backend yet -- only string literals, scalar variables, and `+` concatenation are")),
    }
}

fn emit_function_call(
    name: &BasicIdent,
    args: &[Expr],
    expected: JvmType,
    out: &mut String,
    context: &JvmContext,
) -> Result<(), String> {
    let signature = context
        .function(name)
        .ok_or_else(|| format!("unknown JVM function `{name}`"))?;
    if signature.returns_void
        || signature.result != expected
        || args.len() != signature.source_param_count
    {
        return Err(format!("invalid JVM function call `{name}`"));
    }
    let writebacks = emit_call_arguments(name, args, &signature, out, context)?;
    let args_descriptor = signature.descriptor_params().join("");
    out.push_str(&format!(
        "    invokestatic {}/{} ({args_descriptor}){}\n",
        context.class_name,
        name.name,
        descriptor(signature.result)
    ));
    // Safe to write back after the return value is already on top of stack:
    // each write-back's own net stack effect only ever touches what's above
    // the return value, never it (see `emit_byref_call_writebacks`'s own doc
    // comment).
    emit_byref_call_writebacks(&writebacks, out, context);
    Ok(())
}

fn emit_numeric_expr(
    expr: &Expr,
    out: &mut String,
    context: &JvmContext,
) -> Result<NumericType, String> {
    match expr {
        Expr::ScalarMethodCall { base, method, args } => {
            let base_ty = emit_numeric_expr(base, out, context)?;
            let suffixes = [TypeSuffix::Integer, TypeSuffix::Long, TypeSuffix::Single, TypeSuffix::Double];
            let (name, signature) = suffixes
                .iter()
                .filter_map(|suffix| {
                    let ident = BasicIdent { name: method.clone(), suffix: Some(*suffix) };
                    context.function(&ident).map(|sig| (ident, sig))
                })
                .find(|(_, sig)| sig.params.first() == Some(&JvmType::Numeric(base_ty)) && sig.params.len() == args.len() + 1 && !sig.returns_void)
                .ok_or_else(|| format!("unsupported JVM numeric method `{method}`"))?;
            for (arg, ty) in args.iter().zip(signature.params.iter().skip(1)) {
                match ty {
                    JvmType::String => emit_string_expr(arg, out, context)?,
                    JvmType::Numeric(ty) => emit_numeric_expr_as(arg, *ty, out, context)?,
                }
            }
            let descriptor_args = signature.params.iter().map(|ty| descriptor(*ty)).collect::<String>();
            out.push_str(&format!("    invokestatic {}/{} ({descriptor_args}){}\n", context.class_name, name.name, descriptor(signature.result)));
            match signature.result {
                JvmType::Numeric(result) => Ok(result),
                JvmType::String => Err(format!("numeric method `{method}` returns a string")),
            }
        }
        Expr::Call { name, args } if name.name.eq_ignore_ascii_case("asc") && args.len() == 1 => {
            emit_string_expr(&args[0], out, context)?;
            out.push_str("    iconst_0\n    invokevirtual java/lang/String/charAt (I)C\n");
            Ok(NumericType::Int)
        }
        Expr::Call { name, args } if name.name.eq_ignore_ascii_case("len") && args.len() == 1 => {
            emit_string_expr(&args[0], out, context)?;
            out.push_str("    invokevirtual java/lang/String/length ()I\n");
            Ok(NumericType::Int)
        }
        // `INSTR(s$, needle$)` -- the 1-based position of the first match,
        // or 0. Scoped to this 2-argument form only, matching what
        // `docs/language/arrays-and-strings.html` documents and what
        // `codegen_c.rs`'s own `bcc_instr` implements -- real BASCOM's
        // optional leading `start%` argument (`INSTR(start%, s$, needle$)`)
        // isn't implemented. `String.indexOf` is 0-based-or--1; `+ 1` maps
        // that straight onto BASIC's 1-based-or-0 convention in one step
        // (`-1 + 1 == 0`).
        Expr::Call { name, args } if name.name.eq_ignore_ascii_case("instr") && args.len() == 2 => {
            emit_string_expr(&args[0], out, context)?;
            emit_string_expr(&args[1], out, context)?;
            out.push_str(
                "    invokevirtual java/lang/String/indexOf (Ljava/lang/String;)I\n    \
                 iconst_1\n    iadd\n",
            );
            Ok(NumericType::Int)
        }
        // `CVI(s$)` -- unpacks a raw little-endian 16-bit int from `s$`'s
        // first two bytes (see `MKI$`'s own doc comment in
        // `emit_string_expr`). Works on any string, not just a `FIELD`
        // variable's own decoded value -- ISO-8859-1 round-trips every byte
        // 0-255 exactly, so re-encoding a `FIELD` read back to bytes here
        // recovers the original packed bytes bit-for-bit.
        Expr::Call { name, args } if name.name.eq_ignore_ascii_case("cvi") && args.len() == 1 => {
            emit_string_expr(&args[0], out, context)?;
            emit_wrap_string_as_little_endian_bytebuffer(out);
            out.push_str("    invokevirtual java/nio/ByteBuffer/getShort ()S\n");
            Ok(NumericType::Int)
        }
        // `CVL(s$)` -- unpacks a raw little-endian 32-bit int (see `MKL$`'s
        // own doc comment).
        Expr::Call { name, args } if name.name.eq_ignore_ascii_case("cvl") && args.len() == 1 => {
            emit_string_expr(&args[0], out, context)?;
            emit_wrap_string_as_little_endian_bytebuffer(out);
            out.push_str("    invokevirtual java/nio/ByteBuffer/getInt ()I\n");
            Ok(NumericType::Int)
        }
        // `CVS(s$)` -- unpacks a raw little-endian 32-bit IEEE 754 float
        // (see `MKS$`'s own doc comment on the real, documented MBF
        // divergence this shares with `codegen_c.rs`), widened to this
        // backend's internal `double` representation of a BASIC single.
        Expr::Call { name, args } if name.name.eq_ignore_ascii_case("cvs") && args.len() == 1 => {
            emit_string_expr(&args[0], out, context)?;
            emit_wrap_string_as_little_endian_bytebuffer(out);
            out.push_str("    invokevirtual java/nio/ByteBuffer/getFloat ()F\n    f2d\n");
            Ok(NumericType::Double)
        }
        // `CVD(s$)` -- unpacks a raw little-endian 64-bit IEEE 754 double
        // (see `MKD$`'s own doc comment).
        Expr::Call { name, args } if name.name.eq_ignore_ascii_case("cvd") && args.len() == 1 => {
            emit_string_expr(&args[0], out, context)?;
            emit_wrap_string_as_little_endian_bytebuffer(out);
            out.push_str("    invokevirtual java/nio/ByteBuffer/getDouble ()D\n");
            Ok(NumericType::Double)
        }
        Expr::Call { name, args } if name.name.eq_ignore_ascii_case("abs") && args.len() == 1 => {
            let ty = emit_numeric_expr(&args[0], out, context)?;
            let descriptor = match ty {
                NumericType::Int => "(I)I",
                NumericType::Long => "(J)J",
                NumericType::Double => "(D)D",
            };
            out.push_str(&format!("    invokestatic java/lang/Math/abs {descriptor}\n"));
            Ok(ty)
        }
        Expr::Call { name, args } if name.name.eq_ignore_ascii_case("sqr") && args.len() == 1 => {
            emit_numeric_expr_as(&args[0], NumericType::Double, out, context)?;
            out.push_str("    invokestatic java/lang/Math/sqrt (D)D\n");
            Ok(NumericType::Double)
        }
        Expr::Call { name, args } if name.name.eq_ignore_ascii_case("int") && args.len() == 1 => {
            emit_numeric_expr_as(&args[0], NumericType::Double, out, context)?;
            out.push_str("    invokestatic java/lang/Math/floor (D)D\n");
            Ok(NumericType::Double)
        }
        Expr::Call { name, args } if name.name.eq_ignore_ascii_case("fix") && args.len() == 1 => {
            emit_numeric_expr_as(&args[0], NumericType::Double, out, context)?;
            out.push_str("    d2l\n    l2d\n");
            Ok(NumericType::Double)
        }
        Expr::Call { name, args } if name.name.eq_ignore_ascii_case("sgn") && args.len() == 1 => {
            emit_numeric_expr_as(&args[0], NumericType::Double, out, context)?;
            out.push_str("    invokestatic java/lang/Math/signum (D)D\n");
            Ok(NumericType::Double)
        }
        Expr::Call { name, args } if name.name.eq_ignore_ascii_case("sin") && args.len() == 1 => {
            emit_numeric_expr_as(&args[0], NumericType::Double, out, context)?;
            out.push_str("    invokestatic java/lang/Math/sin (D)D\n");
            Ok(NumericType::Double)
        }
        Expr::Call { name, args } if name.name.eq_ignore_ascii_case("cos") && args.len() == 1 => {
            emit_numeric_expr_as(&args[0], NumericType::Double, out, context)?;
            out.push_str("    invokestatic java/lang/Math/cos (D)D\n");
            Ok(NumericType::Double)
        }
        Expr::Call { name, args } if name.name.eq_ignore_ascii_case("tan") && args.len() == 1 => {
            emit_numeric_expr_as(&args[0], NumericType::Double, out, context)?;
            out.push_str("    invokestatic java/lang/Math/tan (D)D\n");
            Ok(NumericType::Double)
        }
        Expr::Call { name, args } if name.name.eq_ignore_ascii_case("atn") && args.len() == 1 => {
            emit_numeric_expr_as(&args[0], NumericType::Double, out, context)?;
            out.push_str("    invokestatic java/lang/Math/atan (D)D\n");
            Ok(NumericType::Double)
        }
        Expr::Call { name, args } if name.name.eq_ignore_ascii_case("log") && args.len() == 1 => {
            emit_numeric_expr_as(&args[0], NumericType::Double, out, context)?;
            out.push_str("    invokestatic java/lang/Math/log (D)D\n");
            Ok(NumericType::Double)
        }
        Expr::Call { name, args } if name.name.eq_ignore_ascii_case("exp") && args.len() == 1 => {
            emit_numeric_expr_as(&args[0], NumericType::Double, out, context)?;
            out.push_str("    invokestatic java/lang/Math/exp (D)D\n");
            Ok(NumericType::Double)
        }
        Expr::Call { name, args } if name.name.eq_ignore_ascii_case("rnd") && args.is_empty() => {
            out.push_str("    invokestatic java/lang/Math/random ()D\n");
            Ok(NumericType::Double)
        }
        Expr::Call { name, args } if name.name.eq_ignore_ascii_case("cint") && args.len() == 1 => {
            emit_numeric_expr_as(&args[0], NumericType::Double, out, context)?;
            emit_round_away_from_zero(out);
            out.push_str("    l2i\n");
            Ok(NumericType::Int)
        }
        Expr::Call { name, args } if name.name.eq_ignore_ascii_case("clng") && args.len() == 1 => {
            emit_numeric_expr_as(&args[0], NumericType::Double, out, context)?;
            emit_round_away_from_zero(out);
            Ok(NumericType::Long)
        }
        Expr::Call { name, args }
            if (name.name.eq_ignore_ascii_case("csng") || name.name.eq_ignore_ascii_case("cdbl"))
                && args.len() == 1 =>
        {
            emit_numeric_expr_as(&args[0], NumericType::Double, out, context)?;
            Ok(NumericType::Double)
        }
        Expr::Call { name, args }
            if (name.name.eq_ignore_ascii_case("min") || name.name.eq_ignore_ascii_case("max"))
                && args.len() == 2 =>
        {
            let left = infer_numeric_type(&args[0], context)?;
            let right = infer_numeric_type(&args[1], context)?;
            let ty = promote_numeric(left, right);
            emit_numeric_expr_as(&args[0], ty, out, context)?;
            emit_numeric_expr_as(&args[1], ty, out, context)?;
            let suffix = match ty {
                NumericType::Int => "I",
                NumericType::Long => "J",
                NumericType::Double => "D",
            };
            let method = if name.name.eq_ignore_ascii_case("min") { "min" } else { "max" };
            out.push_str(&format!("    invokestatic java/lang/Math/{method} ({suffix}{suffix}){suffix}\n"));
            Ok(ty)
        }
        Expr::Call { name, args } if name.name.eq_ignore_ascii_case("val") && args.len() == 1 => {
            emit_string_expr(&args[0], out, context)?;
            out.push_str("    invokestatic java/lang/Double/parseDouble (Ljava/lang/String;)D\n");
            Ok(NumericType::Double)
        }
        Expr::Call { name, args } if name.name.eq_ignore_ascii_case("instr") && args.len() == 2 => {
            emit_string_expr(&args[0], out, context)?;
            emit_string_expr(&args[1], out, context)?;
            out.push_str("    invokevirtual java/lang/String/indexOf (Ljava/lang/String;)I\n    iconst_1\n    iadd\n");
            Ok(NumericType::Int)
        }
        Expr::Call { name, args } | Expr::ArrayRef { name, indices: args }
            if ["sizeof", "lbound", "ubound"].iter().any(|builtin| name.name.eq_ignore_ascii_case(builtin)) && (1..=2).contains(&args.len()) => {
            let array = match &args[0] {
                Expr::Ident(array) => array,
                Expr::ArrayRef { name, indices } if indices.is_empty() => name,
                _ => return Err(format!("JVM {} requires an array identifier", name.name.to_ascii_uppercase())),
            };
            let shape = context.arrays.get(&variable_key(array)).ok_or_else(|| format!("unknown JVM array `{array}`"))?;
            let axis = match args.get(1) {
                Some(Expr::Integer(axis)) if *axis >= 0 => *axis as usize,
                Some(_) => return Err("JVM array bound axis must be a literal integer".to_string()),
                None if shape.dimensions.len() == 1 => 0,
                None => return Err(format!("JVM {} requires an axis for multidimensional arrays", name.name.to_ascii_uppercase())),
            };
            if axis >= shape.dimensions.len() { return Err(format!("JVM array axis {axis} is out of range")); }
            if name.name.eq_ignore_ascii_case("lbound") {
                out.push_str("    iconst_0\n");
            } else if context.array_slots.contains_key(&variable_key(array)) {
                context.emit_array_axis_length(array, axis, out);
                if name.name.eq_ignore_ascii_case("ubound") { out.push_str("    iconst_1\n    isub\n"); }
            } else {
                emit_numeric_expr_as(&shape.dimensions[axis], NumericType::Int, out, context)?;
                if name.name.eq_ignore_ascii_case("sizeof") { out.push_str("    iconst_1\n    iadd\n"); }
            }
            Ok(NumericType::Int)
        }
        Expr::Call { name, args } | Expr::ArrayRef { name, indices: args }
            if context.function(name).is_some() => {
            let signature = context.function(name).expect("checked above");
            let JvmType::Numeric(result) = signature.result else {
                return Err(format!("`{name}` returns a string, not a numeric value"));
            };
            emit_function_call(name, args, signature.result, out, context)?;
            Ok(result)
        }
        Expr::Call { name, args } if context.arrays.contains_key(&variable_key(name)) => {
            let shape = context.arrays.get(&variable_key(name)).expect("array exists");
            if args.len() != shape.dimensions.len() {
                return Err("JVM indexed access has the wrong number of dimensions".to_string());
            }
            context.emit_array_load(name, out);
            for index in &args[..args.len() - 1] { emit_numeric_expr_as(index, NumericType::Int, out, context)?; out.push_str("    aaload\n"); }
            emit_numeric_expr_as(args.last().expect("validated index count"), NumericType::Int, out, context)?;
            out.push_str(&format!("    {}\n", array_load_opcode(shape.element)));
            match shape.element {
                JvmType::Numeric(ty) => Ok(ty),
                JvmType::String => Err("string array used where a numeric value was expected".to_string()),
            }
        }
        Expr::ArrayRef { name, indices } => {
            let shape = context
                .arrays
                .get(&variable_key(name))
                .ok_or_else(|| format!("unknown JVM array `{name}`"))?;
            if indices.len() != shape.dimensions.len() {
                return Err("JVM indexed access has the wrong number of dimensions".to_string());
            }
            context.emit_array_load(name, out);
            for index in &indices[..indices.len() - 1] {
                emit_numeric_expr_as(index, NumericType::Int, out, context)?;
                out.push_str("    aaload\n");
            }
            emit_numeric_expr_as(indices.last().expect("validated index count"), NumericType::Int, out, context)?;
            out.push_str(&format!("    {}\n", array_load_opcode(shape.element)));
            match shape.element {
                JvmType::Numeric(ty) => Ok(ty),
                JvmType::String => Err("string array used where a numeric value was expected".to_string()),
            }
        }
        Expr::Binary {
            left,
            op: op @ (BinaryOp::Eq | BinaryOp::Ne),
            right,
        } if context.is_string_expr(left) && context.is_string_expr(right) => {
            emit_string_expr(left, out, context)?;
            emit_string_expr(right, out, context)?;
            out.push_str("    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z\n");
            if matches!(op, BinaryOp::Eq) {
                // Convert Java's 1/0 boolean to BASCOM's -1/0 truth value.
                out.push_str("    ineg\n");
            } else {
                // `1 xor 1` is 0 and `0 xor 1` is 1; negate for -1/0.
                out.push_str("    iconst_1\n    ixor\n    ineg\n");
            }
            Ok(NumericType::Int)
        }
        Expr::Integer(value) if i32::try_from(*value).is_ok() => {
            out.push_str(&format!("    ldc {value}\n"));
            Ok(NumericType::Int)
        }
        Expr::Integer(value) => {
            out.push_str(&format!("    ldc2_w {value}L\n"));
            Ok(NumericType::Long)
        }
        Expr::Float(value) if value.is_finite() => {
            out.push_str(&format!("    ldc2_w {value:?}\n"));
            Ok(NumericType::Double)
        }
        Expr::Ident(name) if name.name.eq_ignore_ascii_case("pi") && name.suffix.is_none() => {
            out.push_str("    ldc2_w 3.141592653589793\n");
            Ok(NumericType::Double)
        }
        Expr::Float(_) => Err("non-finite numeric literals are not supported by the JVM backend".to_string()),
        Expr::Ident(name) if context.constant(name).is_some() => {
            emit_numeric_expr(context.constant(name).expect("checked above"), out, context)
        }
        Expr::Ident(name) => {
            let variable = context.variable(name)?;
            let JvmType::Numeric(ty) = variable.ty else {
                return Err(format!("`{name}` is a string, not a numeric scalar"));
            };
            emit_load(variable, out, context);
            Ok(ty)
        }
        Expr::Unary {
            op: UnaryOp::Neg,
            expr,
        } => {
            let ty = emit_numeric_expr(expr, out, context)?;
            out.push_str(match ty {
                NumericType::Int => "    ineg\n",
                NumericType::Long => "    lneg\n",
                NumericType::Double => "    dneg\n",
            });
            Ok(ty)
        }
        Expr::Unary {
            op: UnaryOp::Not,
            expr,
        } => {
            emit_numeric_expr_as(expr, NumericType::Double, out, context)?;
            emit_round_away_from_zero(out);
            out.push_str("    lconst_1\n    lneg\n    lxor\n");
            Ok(NumericType::Long)
        }
        Expr::Binary {
            left,
            op: op @ (BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul),
            right,
        } => {
            let ty = promote_numeric(infer_numeric_type(left, context)?, infer_numeric_type(right, context)?);
            emit_numeric_expr_as(left, ty, out, context)?;
            emit_numeric_expr_as(right, ty, out, context)?;
            let opcode = match (op, ty) {
                (BinaryOp::Add, NumericType::Int) => "iadd",
                (BinaryOp::Sub, NumericType::Int) => "isub",
                (BinaryOp::Mul, NumericType::Int) => "imul",
                (BinaryOp::Add, NumericType::Long) => "ladd",
                (BinaryOp::Sub, NumericType::Long) => "lsub",
                (BinaryOp::Mul, NumericType::Long) => "lmul",
                (BinaryOp::Add, NumericType::Double) => "dadd",
                (BinaryOp::Sub, NumericType::Double) => "dsub",
                (BinaryOp::Mul, NumericType::Double) => "dmul",
                _ => unreachable!(),
            };
            out.push_str(&format!("    {opcode}\n"));
            Ok(ty)
        }
        Expr::Binary { left, op: BinaryOp::Div, right } => {
            emit_numeric_expr_as(left, NumericType::Double, out, context)?;
            emit_numeric_expr_as(right, NumericType::Double, out, context)?;
            out.push_str("    ddiv\n");
            Ok(NumericType::Double)
        }
        Expr::Binary { left, op: BinaryOp::Pow, right } => {
            emit_numeric_expr_as(left, NumericType::Double, out, context)?;
            emit_numeric_expr_as(right, NumericType::Double, out, context)?;
            out.push_str("    invokestatic java/lang/Math/pow (DD)D\n");
            Ok(NumericType::Double)
        }
        Expr::Binary { left, op: BinaryOp::IntDiv | BinaryOp::Mod, right } => {
            emit_numeric_expr_as(left, NumericType::Double, out, context)?;
            emit_round_away_from_zero(out);
            emit_numeric_expr_as(right, NumericType::Double, out, context)?;
            emit_round_away_from_zero(out);
            out.push_str(if matches!(expr, Expr::Binary { op: BinaryOp::IntDiv, .. }) {
                "    ldiv\n"
            } else {
                "    lrem\n"
            });
            Ok(NumericType::Long)
        }
        Expr::Binary { left, op: op @ (BinaryOp::And | BinaryOp::Or | BinaryOp::Xor), right } => {
            emit_numeric_expr_as(left, NumericType::Double, out, context)?;
            emit_round_away_from_zero(out);
            emit_numeric_expr_as(right, NumericType::Double, out, context)?;
            emit_round_away_from_zero(out);
            out.push_str(match op {
                BinaryOp::And => "    land\n",
                BinaryOp::Or => "    lor\n",
                BinaryOp::Xor => "    lxor\n",
                _ => unreachable!(),
            });
            Ok(NumericType::Long)
        }
        Expr::Binary { left, op: op @ (BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge), right } => {
            let ty = promote_numeric(infer_numeric_type(left, context)?, infer_numeric_type(right, context)?);
            emit_numeric_expr_as(left, ty, out, context)?;
            emit_numeric_expr_as(right, ty, out, context)?;
            out.push_str(match ty {
                NumericType::Int => "    invokestatic java/lang/Integer/compare (II)I\n",
                NumericType::Long => "    invokestatic java/lang/Long/compare (JJ)I\n",
                NumericType::Double => "    invokestatic java/lang/Double/compare (DD)I\n",
            });
            match op {
                BinaryOp::Eq => out.push_str("    dup\n    ineg\n    ior\n    bipush 31\n    iushr\n    iconst_1\n    ixor\n    ineg\n"),
                BinaryOp::Ne => out.push_str("    dup\n    ineg\n    ior\n    bipush 31\n    iushr\n    ineg\n"),
                BinaryOp::Lt => out.push_str("    bipush 31\n    ishr\n"),
                BinaryOp::Gt => out.push_str("    ineg\n    bipush 31\n    ishr\n"),
                BinaryOp::Le => out.push_str("    iconst_1\n    isub\n    bipush 31\n    ishr\n"),
                BinaryOp::Ge => out.push_str("    ineg\n    iconst_1\n    isub\n    bipush 31\n    ishr\n"),
                _ => unreachable!(),
            }
            Ok(NumericType::Int)
        }
        other => Err(format!(
            "{other:?} is not supported by the JVM backend yet -- numeric literals and arithmetic are supported"
        )),
    }
}

fn infer_numeric_type(expr: &Expr, context: &JvmContext) -> Result<NumericType, String> {
    match expr {
        Expr::ArrayRef { name, .. } if context.arrays.contains_key(&variable_key(name)) => {
            match context.arrays[&variable_key(name)].element {
                JvmType::Numeric(ty) => Ok(ty),
                JvmType::String => Err(format!("`{name}` is a string array")),
            }
        }
        Expr::Ident(name) if name.name.eq_ignore_ascii_case("pi") && name.suffix.is_none() => Ok(NumericType::Double),
        Expr::Call { name, args } if name.name.eq_ignore_ascii_case("asc") && args.len() == 1 => Ok(NumericType::Int),
        Expr::Call { name, args } if name.name.eq_ignore_ascii_case("len") && args.len() == 1 => Ok(NumericType::Int),
        Expr::Call { name, args } if name.name.eq_ignore_ascii_case("instr") && args.len() == 2 => Ok(NumericType::Int),
        Expr::Call { name, args } if name.name.eq_ignore_ascii_case("cvi") && args.len() == 1 => Ok(NumericType::Int),
        Expr::Call { name, args } if name.name.eq_ignore_ascii_case("cvl") && args.len() == 1 => Ok(NumericType::Int),
        Expr::Call { name, args } if name.name.eq_ignore_ascii_case("cvs") && args.len() == 1 => Ok(NumericType::Double),
        Expr::Call { name, args } if name.name.eq_ignore_ascii_case("cvd") && args.len() == 1 => Ok(NumericType::Double),
        Expr::Call { name, args } if name.name.eq_ignore_ascii_case("cint") && args.len() == 1 => Ok(NumericType::Int),
        Expr::Call { name, args } if name.name.eq_ignore_ascii_case("clng") && args.len() == 1 => Ok(NumericType::Long),
        Expr::Call { name, args }
            if (name.name.eq_ignore_ascii_case("csng") || name.name.eq_ignore_ascii_case("cdbl"))
                && args.len() == 1 => Ok(NumericType::Double),
        Expr::Call { name, args }
            if (name.name.eq_ignore_ascii_case("min") || name.name.eq_ignore_ascii_case("max"))
                && args.len() == 2 => {
            let left = infer_numeric_type(&args[0], context)?;
            let right = infer_numeric_type(&args[1], context)?;
            Ok(promote_numeric(left, right))
        }
        Expr::Call { name, args } if name.name.eq_ignore_ascii_case("val") && args.len() == 1 => Ok(NumericType::Double),
        Expr::Call { name, args } if name.name.eq_ignore_ascii_case("sizeof") && args.len() == 1 => Ok(NumericType::Int),
        Expr::Call { name, .. } | Expr::ArrayRef { name, .. }
            if context.function(name).is_some() => match context.function(name).expect("checked above").result {
            JvmType::Numeric(ty) => Ok(ty),
            JvmType::String => Err(format!("`{name}` is a string function")),
        },
        Expr::Integer(value) if i32::try_from(*value).is_ok() => Ok(NumericType::Int),
        Expr::Integer(_) => Ok(NumericType::Long),
        Expr::Float(value) if value.is_finite() => Ok(NumericType::Double),
        Expr::Float(_) => Err("non-finite numeric literals are not supported by the JVM backend".to_string()),
        Expr::Ident(name) if context.constant(name).is_some() => infer_numeric_type(context.constant(name).expect("checked above"), context),
        Expr::Ident(name) => match context.variable(name)?.ty {
            JvmType::Numeric(ty) => Ok(ty),
            JvmType::String => Err(format!("`{name}` is a string, not numeric")),
        },
        Expr::Unary { op: UnaryOp::Neg, expr } => infer_numeric_type(expr, context),
        Expr::Binary { left, op: BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul, right } => {
            Ok(promote_numeric(infer_numeric_type(left, context)?, infer_numeric_type(right, context)?))
        }
        Expr::Binary { op: BinaryOp::Div | BinaryOp::Pow, .. } => Ok(NumericType::Double),
        Expr::Unary { op: UnaryOp::Not, .. } => Ok(NumericType::Long),
        Expr::Binary { op: BinaryOp::IntDiv | BinaryOp::Mod | BinaryOp::And | BinaryOp::Or | BinaryOp::Xor, .. } => Ok(NumericType::Long),
        Expr::Binary { op: BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge, .. } => Ok(NumericType::Int),
        other => Err(format!(
            "{other:?} is not supported by the JVM backend yet -- numeric literals and arithmetic are supported"
        )),
    }
}

fn emit_numeric_expr_as(
    expr: &Expr,
    target: NumericType,
    out: &mut String,
    context: &JvmContext,
) -> Result<(), String> {
    let inferred = infer_numeric_type(expr, context)?;
    if !can_widen_numeric(inferred, target) {
        emit_numeric_expr(expr, out, context)?;
        match (inferred, target) {
            (NumericType::Long, NumericType::Int) => out.push_str("    l2i\n"),
            (NumericType::Double, NumericType::Long) => emit_round_away_from_zero(out),
            (NumericType::Double, NumericType::Int) => {
                emit_round_away_from_zero(out);
                out.push_str("    l2i\n");
            }
            _ => {
                return Err(format!(
                    "assigning a {inferred:?} expression to a {target:?} scalar is not supported by \
                     the JVM backend yet"
                ));
            }
        }
        return Ok(());
    }
    let actual = emit_numeric_expr(expr, out, context)?;
    coerce_top(actual, target, out);
    Ok(())
}

fn can_widen_numeric(from: NumericType, to: NumericType) -> bool {
    matches!(
        (from, to),
        (
            NumericType::Int,
            NumericType::Int | NumericType::Long | NumericType::Double
        ) | (NumericType::Long, NumericType::Long | NumericType::Double)
            | (NumericType::Double, NumericType::Double)
    )
}

fn promote_numeric(left: NumericType, right: NumericType) -> NumericType {
    if left == NumericType::Double || right == NumericType::Double {
        NumericType::Double
    } else if left == NumericType::Long || right == NumericType::Long {
        NumericType::Long
    } else {
        NumericType::Int
    }
}

fn coerce_top(from: NumericType, to: NumericType, out: &mut String) {
    match (from, to) {
        (NumericType::Int, NumericType::Long) => out.push_str("    i2l\n"),
        (NumericType::Int, NumericType::Double) => out.push_str("    i2d\n"),
        (NumericType::Long, NumericType::Double) => out.push_str("    l2d\n"),
        _ => {}
    }
}

/// Implements BASCOM's round-to-nearest, ties-away-from-zero conversion
/// without a branch: truncate `value + copySign(0.5, value)` toward zero.
fn emit_round_away_from_zero(out: &mut String) {
    out.push_str(
        "    dup2\n    ldc2_w 0.5\n    dup2_x2\n    pop2\n    \
         invokestatic java/lang/Math/copySign (DD)D\n    dadd\n    d2l\n",
    );
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
