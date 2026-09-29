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
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use crate::ast::{
    BasicIdent, BinaryOp, CaseValue, Expr, FunctionDef, OpenMode, ParamMode, PrintToken, Program,
    Statement, Stmt, TypeSuffix, UnaryOp,
};
use crate::diagnostics::{Diagnostic, SourcePos};

/// Transpile a resolver-owned typed program.  JVM emission must not bypass
/// the resolver boundary by accepting a raw AST directly.
pub(crate) fn generate(
    resolved: &crate::resolver::ResolvedProgram,
) -> Result<String, Vec<Diagnostic>> {
    let program = &resolved.program;
    let class_name = class_name_for(program, resolved.semantic_module.as_ref());
    let functions = function_table(&program.functions, resolved.semantic_module.as_ref());
    let context = JvmContext::build(
        program,
        functions.clone(),
        class_name.clone(),
        resolved.function_global_declarations.clone(),
        resolved.typed_array_declarations.clone(),
        resolved.semantic_module.as_ref(),
        resolved.semantic_name_scopes.clone(),
    )?;
    let mut body = String::new();
    context.emit_initializers(&mut body);
    emit_array_initializers(&context, &mut body).map_err(|message| vec![unsupported(&message)])?;
    emit_file_io_initializers(&context, &mut body);
    emit_data_initializers(&context, &mut body);
    emit_input_initializer(&context, &mut body);
    emit_inkey_setup(&context, &mut body);
    let mut emitter = JvmEmitter {
        context: &context,
        next_label: 0,
        loop_exits: Vec::new(),
        loop_continues: Vec::new(),
        return_type: None,
        is_callable: false,
        labels: HashSet::new(),
        exception_handlers: Vec::new(),
    };
    let mut emitted_typed_stream = false;
    if let Some(module) = resolved.semantic_module.as_ref() {
        let mut semantic_state = JvmSemanticState {
            source_filename: String::new(),
            next_label: emitter.next_label,
            exception_handlers: Vec::new(),
        };
        let mut loop_exits = emitter.loop_exits.clone();
        let mut loop_continues = emitter.loop_continues.clone();
        let mut typed_body = String::new();
        if emit_jvm_semantic_module(
            module,
            &context,
            &mut loop_exits,
            &mut loop_continues,
            &mut semantic_state,
            &mut typed_body,
        ) {
            body.push_str(&typed_body);
            emitter.next_label = semantic_state.next_label;
            emitter.loop_exits = loop_exits;
            emitter.loop_continues = loop_continues;
            emitter
                .exception_handlers
                .extend(semantic_state.exception_handlers);
            emitted_typed_stream = true;
        }
    }
    if !emitted_typed_stream {
        let semantic_statements = resolved
            .semantic_module
            .as_ref()
            .and_then(|module| jvm_semantic_statements_by_source(module, &program.statements));
        let semantic_labels = semantic_statements
            .as_ref()
            .map(|statements| {
                let mut labels = HashSet::new();
                for statement in statements.iter().flatten() {
                    collect_semantic_labels(std::slice::from_ref(statement), &mut labels);
                }
                labels
            })
            .unwrap_or_default();
        emitter.labels = program
            .statements
            .iter()
            .enumerate()
            .filter(|(index, _)| {
                semantic_statements
                    .as_ref()
                    .and_then(|statements| statements.get(*index))
                    .is_none_or(Option::is_none)
            })
            .flat_map(|(_, statement)| collect_labels(std::slice::from_ref(statement)))
            .chain(semantic_labels.iter().cloned())
            .collect();
        for (index, statement) in program.statements.iter().enumerate() {
            let semantic_statement = semantic_statements
                .as_ref()
                .and_then(|statements| statements[index]);
            let semantic_source_position = semantic_statement.and_then(|semantic| {
                resolved
                    .semantic_module
                    .as_ref()
                    .and_then(|module| jvm_semantic_top_level_source_position(module, semantic))
            });
            let mut semantic_state = JvmSemanticState {
                source_filename: semantic_source_position
                    .map(|position| position.filename)
                    .unwrap_or_else(|| statement.pos.filename.clone()),
                next_label: emitter.next_label,
                exception_handlers: Vec::new(),
            };
            let handled_semantically = semantic_statement.is_some_and(|semantic| {
                    use crate::semantic_ir::SemanticStatementKind as Kind;
                    match &semantic.kind {
                        Kind::End => {
                            emit_inkey_restore(&context, &mut body);
                            body.push_str("    return\n");
                        }
                        Kind::Stop | Kind::System => {
                            emit_inkey_restore(&context, &mut body);
                            body.push_str(
                                "    iconst_0\n    invokestatic java/lang/System/exit (I)V\n",
                            );
                        }
                        Kind::Cls => {
                            let _ = emit_terminal_escape("\u{1b}[2J\u{1b}[H", &mut body);
                        }
                        Kind::Beep => {
                            let _ = emit_terminal_escape("\u{7}", &mut body);
                        }
                        Kind::Assignment {
                            target,
                            operator,
                            value,
                        } => {
                            return emit_jvm_semantic_assignment(
                                target, *operator, value, &context, &mut body,
                            );
                        }
                        Kind::Expression(crate::semantic_ir::Expression {
                            kind: crate::semantic_ir::ExpressionKind::Call { name, arguments },
                            ..
                        }) => {
                            return emit_jvm_semantic_discarded_callable_call(
                                name, arguments, &context, &mut body,
                            )
                            .is_ok();
                        }
                        Kind::Expression(expression) => {
                            return emit_jvm_semantic_discarded_expression(
                                expression, &context, &mut body,
                            );
                        }
                        Kind::Global { .. } => {}
                        // CLEAR has no JVM runtime effect in this backend;
                        // resolver-owned storage remains method/class scoped.
                        Kind::Clear => {}
                        Kind::Erase(names) => {
                            return emit_jvm_semantic_erase(names, &context, &mut body);
                        }
                        // Scalar slots and arrays are allocated from resolver
                        // and typed-array facts before statement dispatch.
                        Kind::Dim(dimensions) => {
                            return emit_jvm_semantic_dim(dimensions, &context, &mut body);
                        }
                        // Typed DATA items are materialized in the class pool
                        // before statement dispatch.
                        Kind::Data(_) => {}
                        // Record file declarations are consumed by JVM record
                        // layout and file-I/O setup before statement dispatch.
                        Kind::FileDeclaration { .. } => {}
                        Kind::Const { name, value, .. } => {
                            return emit_jvm_semantic_const(name, value, &context, &mut body);
                        }
                        Kind::MidAssign {
                            target,
                            start,
                            length,
                            value,
                        } => {
                            return emit_jvm_semantic_mid_assign(
                                target,
                                start,
                                length.as_ref(),
                                value,
                                &context,
                                &mut body,
                            );
                        }
                        Kind::Swap { left, right } => {
                            return emit_jvm_semantic_swap(left, right, &context, &mut body);
                        }
                        Kind::Error(code) => {
                            return emit_jvm_semantic_error(code, &context, &mut body);
                        }
                        Kind::Throw(crate::semantic_ir::ThrowValue::Value(code)) => {
                            return emit_jvm_semantic_error(code, &context, &mut body);
                        }
                        Kind::Open {
                            path,
                            mode,
                            channel,
                            length: Some(length),
                        } if mode.kind == crate::semantic_ir::OpenModeKind::Random => {
                            return emit_jvm_semantic_random_open(
                                path, channel, length, &context, &mut body,
                            );
                        }
                        Kind::Open {
                            path,
                            mode,
                            channel,
                            length: None,
                        } if matches!(
                            mode.kind,
                            crate::semantic_ir::OpenModeKind::Output
                                | crate::semantic_ir::OpenModeKind::Append
                        ) => {
                            return emit_jvm_semantic_output_open(
                                path,
                                mode.kind,
                                channel,
                                &context,
                                &mut body,
                            );
                        }
                        Kind::Close(channel) => {
                            return emit_jvm_semantic_file_close(channel, &context, &mut body);
                        }
                        Kind::Read(targets) => {
                            return emit_jvm_semantic_read(targets, &context, &mut body);
                        }
                        Kind::Restore(target) => {
                            return emit_jvm_semantic_restore(
                                target.as_ref(),
                                &context,
                                &mut body,
                            );
                        }
                        Kind::OnBranch {
                            selector,
                            branch: crate::semantic_ir::BranchKind::Goto,
                            targets,
                        } => {
                            return emit_jvm_semantic_on_goto(
                                selector,
                                targets,
                                &context,
                                &mut semantic_state,
                                &mut body,
                            );
                        }
                        Kind::Kill(path) => {
                            return emit_jvm_semantic_kill(path, &context, &mut body);
                        }
                        Kind::Rename {
                            source,
                            destination,
                        } => {
                            return emit_jvm_semantic_rename(
                                source,
                                destination,
                                &context,
                                &mut body,
                            );
                        }
                        Kind::Seek { channel, position } => {
                            return emit_jvm_semantic_seek(channel, position, &context, &mut body);
                        }
                        Kind::Get { channel, position } => {
                            return emit_jvm_semantic_get_put(
                                true,
                                channel,
                                position.as_ref(),
                                &context,
                                &mut body,
                            );
                        }
                        Kind::Put { channel, position } => {
                            return emit_jvm_semantic_get_put(
                                false,
                                channel,
                                position.as_ref(),
                                &context,
                                &mut body,
                            );
                        }
                        Kind::Lset { target, value } => {
                            return emit_jvm_semantic_field_set(
                                true, target, value, &context, &mut body,
                            );
                        }
                        Kind::Rset { target, value } => {
                            return emit_jvm_semantic_field_set(
                                false, target, value, &context, &mut body,
                            );
                        }
                        Kind::Field { .. } => {}
                        Kind::Print {
                            destination,
                            tokens,
                        } => {
                            return emit_jvm_semantic_print(
                                destination,
                                tokens,
                                &context,
                                &mut body,
                            );
                        }
                        Kind::Write { channel, values } => {
                            return emit_jvm_semantic_write(channel, values, &context, &mut body);
                        }
                        Kind::Lprint {
                            using: None,
                            tokens,
                        } => {
                            return emit_jvm_semantic_print_tokens(tokens, &context, &mut body);
                        }
                        Kind::Input { source, targets } => {
                            return emit_jvm_semantic_input(source, targets, &context, &mut body);
                        }
                        Kind::LineInput { channel, target } => {
                            return emit_jvm_semantic_line_input(
                                channel, target, &context, &mut body,
                            );
                        }
                        Kind::If {
                            condition,
                            then_body,
                            else_body,
                            ..
                        } => {
                            return emit_jvm_semantic_if(
                                condition,
                                then_body,
                                else_body,
                                &context,
                                &mut semantic_state,
                                &mut body,
                            );
                        }
                        Kind::While {
                            condition,
                            body: loop_body,
                        } => {
                            return emit_jvm_semantic_while(
                                condition,
                                loop_body,
                                &context,
                                &mut semantic_state,
                                &mut body,
                            );
                        }
                        Kind::For {
                            variable,
                            start,
                            bounds,
                            body: loop_body,
                            ..
                        } => {
                            return emit_jvm_semantic_for(
                                variable,
                                start,
                                bounds,
                                loop_body,
                                &context,
                                &mut semantic_state,
                                &mut body,
                            );
                        }
                        Kind::Do {
                            pre_condition,
                            post_condition,
                            body: loop_body,
                        } => {
                            return emit_jvm_semantic_do(
                                pre_condition.as_ref(),
                                post_condition.as_ref(),
                                loop_body,
                                &context,
                                &mut semantic_state,
                                &mut body,
                            );
                        }
                        Kind::SelectCase {
                            selector,
                            cases,
                            else_body,
                        } => {
                            return emit_jvm_semantic_select_case(
                                selector,
                                cases,
                                else_body,
                                &context,
                                &mut semantic_state,
                                &mut body,
                            );
                        }
                        Kind::Try {
                            body: try_body,
                            catch,
                            finally_body,
                        } => {
                            return emit_jvm_semantic_try(
                                try_body,
                                catch.as_ref(),
                                finally_body,
                                &context,
                                &mut emitter.loop_exits,
                                &mut emitter.loop_continues,
                                &mut semantic_state,
                                &mut body,
                            );
                        }
                        Kind::Locate { row, column } => {
                            return emit_jvm_semantic_locate(row, column, &context, &mut body);
                        }
                        Kind::Color {
                            foreground,
                            background,
                        } => {
                            return emit_jvm_semantic_color(
                                foreground,
                                background.as_ref(),
                                &context,
                                &mut body,
                            );
                        }
                        Kind::Comment { block, text } => {
                            emit_jvm_semantic_comment(*block, text, &mut body);
                        }
                        Kind::Label(name) => {
                            body.push_str(&format!("{}:\n", jvm_label(&name.name)));
                        }
                        Kind::Goto(target)
                            if semantic_labels.contains(&target.name.to_ascii_lowercase()) =>
                        {
                            body.push_str(&format!("    goto {}\n", jvm_label(&target.name)));
                        }
                        _ => return false,
                    }
                    true
                });
            if handled_semantically {
                emitter.next_label = semantic_state.next_label;
                emitter
                    .exception_handlers
                    .extend(semantic_state.exception_handlers);
            }
            if !handled_semantically {
                emitter
                    .emit_statement(statement, &mut body)
                    .map_err(|message| vec![unsupported(&message)])?;
            }
        }
    }
    // `Statement::End` already emits its own `return` -- only add the
    // implicit fallthrough one when the program didn't already end with an
    // explicit `end`, otherwise the method would end in two `return`
    // instructions back to back (harmless to the JVM, but not what a real
    // `end` vs. no `end` should look like in the generated text).
    if !resolved
        .semantic_module
        .as_ref()
        .map(crate::semantic_ir::SemanticModule::ends_with_end)
        .unwrap_or_else(|| ends_with_end(&program.statements))
    {
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
    let has_semantic_mid_assign = resolved.semantic_module.as_ref().is_some_and(|module| {
        semantic_statements_contain_mid_assign(&module.statements)
            || module
                .callables
                .iter()
                .any(|callable| semantic_statements_contain_mid_assign(&callable.body))
    });
    for function in &program.functions {
        if has_semantic_mid_assign
            && function
                .name
                .name
                .eq_ignore_ascii_case(crate::codegen_basic::MID_ASSIGN_HELPER_NAME)
        {
            continue;
        }
        methods.push_str(
            &emit_function(function, &context).map_err(|message| vec![unsupported(&message)])?,
        );
    }
    if has_semantic_mid_assign {
        let semantic_call = format!("invokestatic {class_name}/bccMidAssign ");
        if !body.contains(&semantic_call) && !methods.contains(&semantic_call) {
            if let Some(function) = program.functions.iter().find(|function| {
                function
                    .name
                    .name
                    .eq_ignore_ascii_case(crate::codegen_basic::MID_ASSIGN_HELPER_NAME)
            }) {
                methods.push_str(
                    &emit_function(function, &context)
                        .map_err(|message| vec![unsupported(&message)])?,
                );
            }
        }
    }
    let bcc_array_copy_call = format!(
        "invokestatic {class_name}/bccCopyArray (Ljava/lang/Object;II)Ljava/lang/Object;\n"
    );
    if body.contains(&bcc_array_copy_call) || methods.contains(&bcc_array_copy_call) {
        methods.push_str(&emit_array_copy_helper(&class_name));
    }
    let bcc_read_data_call = format!(
        "invokestatic {class_name}/bccReadData ()Ljava/lang/String;\n"
    );
    if body.contains(&bcc_read_data_call) || methods.contains(&bcc_read_data_call) {
        methods.push_str(&emit_data_read_helper(&class_name));
    }
    let bcc_mid_assign_call = format!(
        "invokestatic {class_name}/bccMidAssign (Ljava/lang/String;IIZLjava/lang/String;)Ljava/lang/String;\n"
    );
    if body.contains(&bcc_mid_assign_call) || methods.contains(&bcc_mid_assign_call) {
        methods.push_str(&emit_mid_assign_helper());
    }
    for (helper_name, values, mask) in [
        ("bccAnsiFg", ANSI_FG.as_slice(), 15),
        ("bccAnsiBg", ANSI_BG.as_slice(), 7),
    ] {
        let call = format!("invokestatic {class_name}/{helper_name} (I)I\n");
        if body.contains(&call) || methods.contains(&call) {
            methods.push_str(&emit_ansi_palette_helper(helper_name, values, mask));
        }
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

fn emit_jvm_semantic_const(
    name: &crate::semantic_ir::NamedReference,
    value: &crate::semantic_ir::Expression,
    context: &JvmContext,
    out: &mut String,
) -> bool {
    let variable = match context.variable(&BasicIdent::parse(&name.name)) {
        Ok(variable) => variable,
        Err(_) => return false,
    };
    let mut rendered = String::new();
    match variable.ty {
        JvmType::String => {
            if value.value_type != crate::semantic_ir::SemanticValueType::String
                || emit_jvm_semantic_string_expression(value, &mut rendered, context).is_err()
            {
                return false;
            }
        }
        JvmType::Numeric(target_type) => {
            let value_type =
                match emit_jvm_semantic_numeric_expression(value, &mut rendered, context) {
                    Ok(value_type) => value_type,
                    Err(_) => return false,
                };
            coerce_jvm_semantic_numeric(value_type, target_type, &mut rendered);
        }
    }
    emit_store(variable, &mut rendered, context);
    out.push_str(&rendered);
    true
}

fn emit_jvm_semantic_assignment(
    target: &crate::semantic_ir::Expression,
    operator: crate::semantic_ir::AssignmentOperator,
    value: &crate::semantic_ir::Expression,
    context: &JvmContext,
    out: &mut String,
) -> bool {
    use crate::semantic_ir::{ExpressionKind, SemanticValueType};
    if target.value_type == SemanticValueType::Unknown {
        return false;
    }
    if matches!(
        target.kind,
        ExpressionKind::Index { .. } | ExpressionKind::MultiIndex { .. }
    ) {
        return emit_jvm_semantic_array_assignment(target, operator, value, context, out);
    }
    let ExpressionKind::Name(target_name) = &target.kind else {
        return false;
    };
    let target_variable = match context.variable(&BasicIdent::parse(target_name)) {
        Ok(variable) => variable,
        _ => return false,
    };
    if target_variable.ty == JvmType::String {
        if operator != crate::semantic_ir::AssignmentOperator::Assign
            || value.value_type != SemanticValueType::String
        {
            return false;
        }
        let mut rendered = String::new();
        if emit_jvm_semantic_string_expression(value, &mut rendered, context).is_err() {
            return false;
        }
        emit_store(target_variable, &mut rendered, context);
        out.push_str(&rendered);
        return true;
    }
    let JvmType::Numeric(target_type) = target_variable.ty else {
        return false;
    };
    let Some(semantic_target_type) = jvm_semantic_numeric_type(target.value_type) else {
        return false;
    };
    if semantic_target_type != target_type {
        return false;
    }
    if operator == crate::semantic_ir::AssignmentOperator::Assign {
        let mut rendered = String::new();
        let value_type = match emit_jvm_semantic_numeric_expression(value, &mut rendered, context) {
            Ok(value_type) => value_type,
            Err(_) => return false,
        };
        coerce_jvm_semantic_numeric(value_type, target_type, &mut rendered);
        emit_jvm_semantic_single_rounding(target.value_type, &mut rendered);
        emit_store(target_variable, &mut rendered, context);
        out.push_str(&rendered);
        return true;
    }
    emit_jvm_semantic_compound_assignment(
        target_variable,
        target_type,
        operator,
        value,
        target.value_type == SemanticValueType::Single,
        context,
        out,
    )
}

fn emit_jvm_semantic_mid_assign(
    target: &crate::semantic_ir::Expression,
    start: &crate::semantic_ir::Expression,
    length: Option<&crate::semantic_ir::Expression>,
    value: &crate::semantic_ir::Expression,
    context: &JvmContext,
    out: &mut String,
) -> bool {
    use crate::semantic_ir::{ExpressionKind, SemanticValueType};
    let ExpressionKind::Name(name) = &target.kind else {
        return false;
    };
    if target.value_type != SemanticValueType::String
        || value.value_type != SemanticValueType::String
    {
        return false;
    }
    let Ok(variable) = context.variable(&BasicIdent::parse(name)) else {
        return false;
    };
    if variable.ty != JvmType::String {
        return false;
    }
    let mut rendered = String::new();
    emit_load(variable, &mut rendered, context);
    let start_type = match emit_jvm_semantic_numeric_expression(start, &mut rendered, context) {
        Ok(ty) => ty,
        Err(_) => return false,
    };
    coerce_jvm_semantic_numeric(start_type, NumericType::Int, &mut rendered);
    if let Some(length) = length {
        let length_type = match emit_jvm_semantic_numeric_expression(length, &mut rendered, context)
        {
            Ok(ty) => ty,
            Err(_) => return false,
        };
        coerce_jvm_semantic_numeric(length_type, NumericType::Int, &mut rendered);
        rendered.push_str("    iconst_0\n");
    } else {
        rendered.push_str("    iconst_m1\n");
        rendered.push_str("    iconst_1\n");
    }
    if emit_jvm_semantic_string_expression(value, &mut rendered, context).is_err() {
        return false;
    }
    rendered.push_str(&format!(
        "    invokestatic {}/bccMidAssign (Ljava/lang/String;IIZLjava/lang/String;)Ljava/lang/String;\n",
        context.class_name
    ));
    emit_store(variable, &mut rendered, context);
    out.push_str(&rendered);
    true
}

fn emit_jvm_semantic_swap(
    left: &crate::semantic_ir::Expression,
    right: &crate::semantic_ir::Expression,
    context: &JvmContext,
    out: &mut String,
) -> bool {
    use crate::semantic_ir::{ExpressionKind, SemanticValueType};
    let (ExpressionKind::Name(left_name), ExpressionKind::Name(right_name)) =
        (&left.kind, &right.kind)
    else {
        return false;
    };
    let Ok(left_variable) = context.variable(&BasicIdent::parse(left_name)) else {
        return false;
    };
    let Ok(right_variable) = context.variable(&BasicIdent::parse(right_name)) else {
        return false;
    };
    if left_variable.ty != right_variable.ty {
        return false;
    }
    let type_matches = match left_variable.ty {
        JvmType::String => {
            left.value_type == SemanticValueType::String
                && right.value_type == SemanticValueType::String
        }
        JvmType::Numeric(ty) => {
            jvm_semantic_numeric_type(left.value_type) == Some(ty)
                && jvm_semantic_numeric_type(right.value_type) == Some(ty)
        }
    };
    if !type_matches {
        return false;
    }
    let mut rendered = String::new();
    emit_load(left_variable, &mut rendered, context);
    emit_load(right_variable, &mut rendered, context);
    emit_store(left_variable, &mut rendered, context);
    emit_store(right_variable, &mut rendered, context);
    out.push_str(&rendered);
    true
}

fn emit_jvm_semantic_error(
    expression: &crate::semantic_ir::Expression,
    context: &JvmContext,
    out: &mut String,
) -> bool {
    let mut rendered = String::new();
    let ty = match emit_jvm_semantic_numeric_expression(expression, &mut rendered, context) {
        Ok(ty) => ty,
        Err(_) => return false,
    };
    coerce_jvm_semantic_numeric(ty, NumericType::Int, &mut rendered);
    rendered.insert_str(0, "    new java/lang/RuntimeException\n    dup\n");
    rendered.push_str("    invokestatic java/lang/Integer/toString (I)Ljava/lang/String;\n    invokespecial java/lang/RuntimeException/<init> (Ljava/lang/String;)V\n    athrow\n");
    out.push_str(&rendered);
    true
}

fn emit_jvm_semantic_random_open(
    path: &crate::semantic_ir::Expression,
    channel: &crate::semantic_ir::Expression,
    length: &crate::semantic_ir::Expression,
    context: &JvmContext,
    out: &mut String,
) -> bool {
    let mut rendered = String::new();
    rendered.push_str(&format!(
        "    getstatic {}/bccFiles [Ljava/io/RandomAccessFile;\n",
        context.class_name
    ));
    let channel_type =
        match emit_jvm_semantic_numeric_expression(channel, &mut rendered, context) {
            Ok(ty) => ty,
            Err(_) => return false,
        };
    coerce_jvm_semantic_numeric(channel_type, NumericType::Int, &mut rendered);
    rendered.push_str(
        "    iconst_1\n    isub\n    dup_x1\n    new java/io/RandomAccessFile\n    dup\n",
    );
    if emit_jvm_semantic_string_expression(path, &mut rendered, context).is_err() {
        return false;
    }
    rendered.push_str("    ldc \"rw\"\n    invokespecial java/io/RandomAccessFile/<init> (Ljava/lang/String;Ljava/lang/String;)V\n    aastore\n");
    rendered.push_str(&format!(
        "    getstatic {}/bccRecLen [I\n    swap\n    dup_x1\n",
        context.class_name
    ));
    let length_type = match emit_jvm_semantic_numeric_expression(length, &mut rendered, context) {
        Ok(ty) => ty,
        Err(_) => return false,
    };
    coerce_jvm_semantic_numeric(length_type, NumericType::Int, &mut rendered);
    rendered.push_str("    iastore\n");
    rendered.push_str(&format!(
        "    getstatic {}/bccBufs [[B\n    swap\n    dup_x1\n",
        context.class_name
    ));
    let length_type = match emit_jvm_semantic_numeric_expression(length, &mut rendered, context) {
        Ok(ty) => ty,
        Err(_) => return false,
    };
    coerce_jvm_semantic_numeric(length_type, NumericType::Int, &mut rendered);
    rendered.push_str("    newarray byte\n    aastore\n    pop\n");
    out.push_str(&rendered);
    true
}

fn emit_jvm_semantic_output_open(
    path: &crate::semantic_ir::Expression,
    mode: crate::semantic_ir::OpenModeKind,
    channel: &crate::semantic_ir::Expression,
    context: &JvmContext,
    out: &mut String,
) -> bool {
    use crate::semantic_ir::OpenModeKind;
    if !matches!(mode, OpenModeKind::Output | OpenModeKind::Append)
        || path.value_type != crate::semantic_ir::SemanticValueType::String
    {
        return false;
    }
    let mut rendered = String::new();
    rendered.push_str(&format!(
        "    getstatic {}/bccFiles [Ljava/io/RandomAccessFile;\n",
        context.class_name
    ));
    let channel_type = match emit_jvm_semantic_numeric_expression(channel, &mut rendered, context) {
        Ok(ty) => ty,
        Err(_) => return false,
    };
    coerce_jvm_semantic_numeric(channel_type, NumericType::Int, &mut rendered);
    rendered.push_str("    iconst_1\n    isub\n    dup_x1\n    new java/io/RandomAccessFile\n    dup\n");
    if emit_jvm_semantic_string_expression(path, &mut rendered, context).is_err() {
        return false;
    }
    rendered.push_str("    ldc \"rw\"\n    invokespecial java/io/RandomAccessFile/<init> (Ljava/lang/String;Ljava/lang/String;)V\n    aastore\n");
    rendered.push_str(&format!(
        "    getstatic {}/bccFiles [Ljava/io/RandomAccessFile;\n    swap\n    dup_x1\n    aaload\n    swap\n    pop\n",
        context.class_name
    ));
    match mode {
        OpenModeKind::Output => rendered.push_str(
            "    lconst_0\n    invokevirtual java/io/RandomAccessFile/setLength (J)V\n",
        ),
        OpenModeKind::Append => rendered.push_str(
            "    dup\n    invokevirtual java/io/RandomAccessFile/length ()J\n    invokevirtual java/io/RandomAccessFile/seek (J)V\n",
        ),
        _ => return false,
    }
    out.push_str(&rendered);
    true
}

fn emit_jvm_semantic_file_close(
    channel: &crate::semantic_ir::Expression,
    context: &JvmContext,
    out: &mut String,
) -> bool {
    let mut rendered = format!(
        "    getstatic {}/bccFiles [Ljava/io/RandomAccessFile;\n",
        context.class_name
    );
    let ty = match emit_jvm_semantic_numeric_expression(channel, &mut rendered, context) {
        Ok(ty) => ty,
        Err(_) => return false,
    };
    coerce_jvm_semantic_numeric(ty, NumericType::Int, &mut rendered);
    rendered.push_str("    iconst_1\n    isub\n    aaload\n    invokevirtual java/io/RandomAccessFile/close ()V\n");
    out.push_str(&rendered);
    true
}

fn emit_jvm_semantic_read(
    targets: &[crate::semantic_ir::Expression],
    context: &JvmContext,
    out: &mut String,
) -> bool {
    use crate::semantic_ir::{ExpressionKind, SemanticValueType};
    if context.data_items.is_empty() {
        return false;
    }
    let mut rendered = String::new();
    for target in targets {
        let expected_type = match target.value_type {
            SemanticValueType::String => JvmType::String,
            SemanticValueType::Integer | SemanticValueType::Boolean => {
                JvmType::Numeric(NumericType::Int)
            }
            SemanticValueType::Long => JvmType::Numeric(NumericType::Long),
            SemanticValueType::Single | SemanticValueType::Double => {
                JvmType::Numeric(NumericType::Double)
            }
            SemanticValueType::Unknown => return false,
        };
        let array_element = match &target.kind {
            ExpressionKind::Name(name) => {
                let variable = match context.variable(&BasicIdent::parse(name)) {
                    Ok(variable) => variable,
                    Err(_) => return false,
                };
                if variable.ty != expected_type {
                    return false;
                }
                None
            }
            ExpressionKind::Index { name, index } => {
                match emit_jvm_semantic_array_address(
                    name,
                    std::slice::from_ref(index.as_ref()),
                    context,
                    &mut rendered,
                ) {
                    Ok(element) if element == expected_type => Some(element),
                    _ => return false,
                }
            }
            ExpressionKind::MultiIndex { name, indices } => {
                match emit_jvm_semantic_array_address(name, indices, context, &mut rendered) {
                    Ok(element) if element == expected_type => Some(element),
                    _ => return false,
                }
            }
            _ => return false,
        };
        rendered.push_str(&format!(
            "    invokestatic {}/bccReadData ()Ljava/lang/String;\n",
            context.class_name
        ));
        match expected_type {
            JvmType::String => {}
            JvmType::Numeric(NumericType::Int) => {
                rendered.push_str(
                    "    invokestatic java/lang/Integer/parseInt (Ljava/lang/String;)I\n",
                )
            }
            JvmType::Numeric(NumericType::Long) => rendered.push_str(
                "    invokestatic java/lang/Long/parseLong (Ljava/lang/String;)J\n",
            ),
            JvmType::Numeric(NumericType::Double) => rendered.push_str(
                "    invokestatic java/lang/Double/parseDouble (Ljava/lang/String;)D\n",
            ),
        }
        emit_jvm_semantic_single_rounding(target.value_type, &mut rendered);
        if let Some(element) = array_element {
            rendered.push_str(&format!("    {}\n", array_store_opcode(element)));
        } else {
            let ExpressionKind::Name(name) = &target.kind else {
                unreachable!()
            };
            let variable = context
                .variable(&BasicIdent::parse(name))
                .expect("scalar READ target was resolved above");
            emit_store(variable, &mut rendered, context);
        }
    }
    out.push_str(&rendered);
    true
}

fn emit_data_read_helper(class_name: &str) -> String {
    format!(
        ".method private static bccReadData : ()Ljava/lang/String;\n    .limit stack 3\n    .limit locals 0\n\n    getstatic {class_name}/bccDataPtr I\n    getstatic {class_name}/bccData [Ljava/lang/String;\n    arraylength\n    if_icmplt L_bcc_data_available\n    new java/lang/IllegalStateException\n    dup\n    ldc \"Out of DATA\"\n    invokespecial java/lang/IllegalStateException/<init> (Ljava/lang/String;)V\n    athrow\nL_bcc_data_available:\n    getstatic {class_name}/bccData [Ljava/lang/String;\n    getstatic {class_name}/bccDataPtr I\n    aaload\n    getstatic {class_name}/bccDataPtr I\n    iconst_1\n    iadd\n    putstatic {class_name}/bccDataPtr I\n    areturn\n.end method\n\n"
    )
}

fn emit_jvm_semantic_restore(
    target: Option<&crate::semantic_ir::NamedReference>,
    context: &JvmContext,
    out: &mut String,
) -> bool {
    if context.data_items.is_empty() {
        return true;
    }
    let index = match target {
        Some(target) => {
            let Some(index) = context
                .data_labels
                .get(&target.name.to_ascii_lowercase())
            else {
                return false;
            };
            *index
        }
        None => 0,
    };
    out.push_str(&format!(
        "    ldc {index}\n    putstatic {}/bccDataPtr I\n",
        context.class_name
    ));
    true
}

fn emit_jvm_semantic_on_goto(
    selector: &crate::semantic_ir::Expression,
    targets: &[crate::semantic_ir::NamedReference],
    context: &JvmContext,
    _state: &mut JvmSemanticState,
    out: &mut String,
) -> bool {
    if targets.is_empty() {
        return false;
    }
    let id = context.condition_label.get();
    context.condition_label.set(id + 1);
    let mut rendered = String::new();
    let ty = match emit_jvm_semantic_numeric_expression(selector, &mut rendered, context) {
        Ok(ty) => ty,
        Err(_) => return false,
    };
    coerce_jvm_semantic_numeric(ty, NumericType::Int, &mut rendered);
    for (index, _) in targets.iter().enumerate() {
        rendered.push_str(&format!(
            "    dup\n    ldc {}\n    if_icmpeq L_on_goto_{id}_case_{index}\n",
            index + 1
        ));
    }
    rendered.push_str(&format!(
        "    pop\n    goto L_on_goto_{id}_done\n"
    ));
    for (index, target) in targets.iter().enumerate() {
        rendered.push_str(&format!(
            "L_on_goto_{id}_case_{index}:\n    pop\n    goto {}\n",
            jvm_label(&target.name)
        ));
    }
    rendered.push_str(&format!("L_on_goto_{id}_done:\n"));
    out.push_str(&rendered);
    true
}

fn emit_jvm_semantic_kill(
    path: &crate::semantic_ir::Expression,
    context: &JvmContext,
    out: &mut String,
) -> bool {
    let mut rendered = String::from("    new java/io/File\n    dup\n");
    if emit_jvm_semantic_string_expression(path, &mut rendered, context).is_err() {
        return false;
    }
    rendered.push_str("    invokespecial java/io/File/<init> (Ljava/lang/String;)V\n    invokevirtual java/io/File/delete ()Z\n    pop\n");
    out.push_str(&rendered);
    true
}

fn emit_jvm_semantic_rename(
    source: &crate::semantic_ir::Expression,
    destination: &crate::semantic_ir::Expression,
    context: &JvmContext,
    out: &mut String,
) -> bool {
    let mut rendered = String::from("    new java/io/File\n    dup\n");
    if emit_jvm_semantic_string_expression(source, &mut rendered, context).is_err() {
        return false;
    }
    rendered.push_str("    invokespecial java/io/File/<init> (Ljava/lang/String;)V\n    new java/io/File\n    dup\n");
    if emit_jvm_semantic_string_expression(destination, &mut rendered, context).is_err() {
        return false;
    }
    rendered.push_str("    invokespecial java/io/File/<init> (Ljava/lang/String;)V\n    invokevirtual java/io/File/renameTo (Ljava/io/File;)Z\n    pop\n");
    out.push_str(&rendered);
    true
}

fn emit_jvm_semantic_field_set(
    left_justify: bool,
    target: &crate::semantic_ir::NamedReference,
    value: &crate::semantic_ir::Expression,
    context: &JvmContext,
    out: &mut String,
) -> bool {
    let ident = BasicIdent::parse(&target.name);
    let Some(field) = context.field_vars.get(&variable_key(&ident)).copied() else {
        return false;
    };
    let mut rendered = String::new();
    if left_justify {
        if emit_jvm_semantic_string_expression(value, &mut rendered, context).is_err() {
            return false;
        }
        emit_space_string(field.width, &mut rendered);
        rendered.push_str(
            "    invokevirtual java/lang/String/concat (Ljava/lang/String;)Ljava/lang/String;\n",
        );
        rendered.push_str(&format!(
            "    iconst_0\n    ldc {}\n    invokevirtual java/lang/String/substring (II)Ljava/lang/String;\n",
            field.width
        ));
    } else {
        emit_space_string(field.width, &mut rendered);
        if emit_jvm_semantic_string_expression(value, &mut rendered, context).is_err() {
            return false;
        }
        rendered.push_str(
            "    invokevirtual java/lang/String/concat (Ljava/lang/String;)Ljava/lang/String;\n",
        );
        rendered.push_str(&format!(
            "    dup\n    invokevirtual java/lang/String/length ()I\n    dup\n    ldc {}\n    isub\n    swap\n    invokevirtual java/lang/String/substring (II)Ljava/lang/String;\n",
            field.width
        ));
    }
    emit_store_field_bytes(field, &mut rendered, context);
    out.push_str(&rendered);
    true
}

fn emit_jvm_semantic_get_put(
    is_get: bool,
    channel: &crate::semantic_ir::Expression,
    position: Option<&crate::semantic_ir::FilePosition>,
    context: &JvmContext,
    out: &mut String,
) -> bool {
    let Some(record) = position
        .filter(|position| position.record.is_none())
        .and_then(|position| position.position.as_ref())
    else {
        return false;
    };
    let mut rendered = format!(
        "    getstatic {}/bccFiles [Ljava/io/RandomAccessFile;\n",
        context.class_name
    );
    let channel_type = match emit_jvm_semantic_numeric_expression(channel, &mut rendered, context) {
        Ok(ty) => ty,
        Err(_) => return false,
    };
    coerce_jvm_semantic_numeric(channel_type, NumericType::Int, &mut rendered);
    rendered.push_str("    iconst_1\n    isub\n    aaload\n    dup\n");
    let record_type = match emit_jvm_semantic_numeric_expression(record, &mut rendered, context) {
        Ok(ty) => ty,
        Err(_) => return false,
    };
    coerce_jvm_semantic_numeric(record_type, NumericType::Long, &mut rendered);
    rendered.push_str("    lconst_1\n    lsub\n");
    rendered.push_str(&format!(
        "    getstatic {}/bccRecLen [I\n",
        context.class_name
    ));
    let channel_type = match emit_jvm_semantic_numeric_expression(channel, &mut rendered, context) {
        Ok(ty) => ty,
        Err(_) => return false,
    };
    coerce_jvm_semantic_numeric(channel_type, NumericType::Int, &mut rendered);
    rendered.push_str(
        "    iconst_1\n    isub\n    iaload\n    i2l\n    lmul\n    invokevirtual java/io/RandomAccessFile/seek (J)V\n",
    );
    rendered.push_str(&format!(
        "    getstatic {}/bccBufs [[B\n",
        context.class_name
    ));
    let channel_type = match emit_jvm_semantic_numeric_expression(channel, &mut rendered, context) {
        Ok(ty) => ty,
        Err(_) => return false,
    };
    coerce_jvm_semantic_numeric(channel_type, NumericType::Int, &mut rendered);
    rendered.push_str("    iconst_1\n    isub\n    aaload\n");
    rendered.push_str(if is_get {
        "    invokevirtual java/io/RandomAccessFile/read ([B)I\n    pop\n"
    } else {
        "    invokevirtual java/io/RandomAccessFile/write ([B)V\n"
    });
    out.push_str(&rendered);
    true
}

fn emit_jvm_semantic_seek(
    channel: &crate::semantic_ir::Expression,
    position: &crate::semantic_ir::Expression,
    context: &JvmContext,
    out: &mut String,
) -> bool {
    let mut rendered = format!(
        "    getstatic {}/bccFiles [Ljava/io/RandomAccessFile;\n",
        context.class_name
    );
    let channel_type = match emit_jvm_semantic_numeric_expression(channel, &mut rendered, context) {
        Ok(ty) => ty,
        Err(_) => return false,
    };
    coerce_jvm_semantic_numeric(channel_type, NumericType::Int, &mut rendered);
    rendered.push_str("    iconst_1\n    isub\n    aaload\n");
    let position_type = match emit_jvm_semantic_numeric_expression(position, &mut rendered, context)
    {
        Ok(ty) => ty,
        Err(_) => return false,
    };
    coerce_jvm_semantic_numeric(position_type, NumericType::Long, &mut rendered);
    rendered.push_str("    lconst_1\n    lsub\n");
    rendered.push_str(&format!(
        "    getstatic {}/bccRecLen [I\n",
        context.class_name
    ));
    let channel_type = match emit_jvm_semantic_numeric_expression(channel, &mut rendered, context) {
        Ok(ty) => ty,
        Err(_) => return false,
    };
    coerce_jvm_semantic_numeric(channel_type, NumericType::Int, &mut rendered);
    rendered.push_str(
        "    iconst_1\n    isub\n    iaload\n    i2l\n    lmul\n    invokevirtual java/io/RandomAccessFile/seek (J)V\n",
    );
    out.push_str(&rendered);
    true
}

fn emit_mid_assign_helper() -> String {
    r#".method private static bccMidAssign : (Ljava/lang/String;IIZLjava/lang/String;)Ljava/lang/String;
    .limit stack 4
    .limit locals 9

    aload 4
    invokevirtual java/lang/String/length ()I
    istore 6
    aload 0
    invokevirtual java/lang/String/length ()I
    istore 5
    iload 3
    ifeq L_mid_len_set
    iload 6
    istore 2
L_mid_len_set:
    iload 2
    ifge L_mid_len_nonnegative
    iconst_0
    istore 2
L_mid_len_nonnegative:
    iload 6
    iload 2
    if_icmple L_mid_value_len_set
    iload 2
    istore 6
L_mid_value_len_set:
    iload 1
    iconst_1
    isub
    istore 7
    iload 7
    ifge L_mid_left_nonnegative
    iconst_0
    istore 7
L_mid_left_nonnegative:
    iload 7
    iload 5
    if_icmple L_mid_left_set
    iload 5
    istore 7
L_mid_left_set:
    iload 7
    iload 6
    iadd
    istore 8
    iload 8
    iload 5
    if_icmple L_mid_tail_set
    iload 5
    istore 8
L_mid_tail_set:
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    aload 0
    iconst_0
    iload 7
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    aload 4
    iconst_0
    iload 6
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    aload 0
    iload 8
    iload 5
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    areturn
.end method

"#
.to_string()
}

fn emit_jvm_semantic_array_address(
    name: &str,
    indices: &[crate::semantic_ir::Expression],
    context: &JvmContext,
    out: &mut String,
) -> Result<JvmType, String> {
    let ident = BasicIdent::parse(name);
    let shape = context
        .arrays
        .get(&variable_key(&ident))
        .ok_or_else(|| format!("`{name}` is not a resolved JVM array"))?;
    if indices.len() != shape.dimensions.len() {
        return Err(format!(
            "semantic JVM array rank {} does not match resolved rank {}",
            indices.len(),
            shape.dimensions.len()
        ));
    }
    context.emit_array_load(&ident, out);
    for (axis, index) in indices.iter().enumerate() {
        let index_type = emit_jvm_semantic_numeric_expression(index, out, context)?;
        emit_jvm_array_index_conversion(index_type, out);
        if axis + 1 < indices.len() {
            out.push_str("    aaload\n");
        }
    }
    Ok(shape.element)
}

fn emit_jvm_semantic_numeric_array_read(
    name: &str,
    indices: &[crate::semantic_ir::Expression],
    value_type: crate::semantic_ir::SemanticValueType,
    context: &JvmContext,
    out: &mut String,
) -> Result<NumericType, String> {
    let element = emit_jvm_semantic_array_address(name, indices, context, out)?;
    let JvmType::Numeric(ty) = element else {
        return Err("semantic numeric array read targets a string array".to_string());
    };
    if jvm_semantic_numeric_type(value_type) != Some(ty) {
        return Err(
            "semantic JVM array element type does not match its resolved declaration".to_string(),
        );
    }
    out.push_str(&format!("    {}\n", array_load_opcode(element)));
    Ok(ty)
}

fn emit_jvm_semantic_string_array_read(
    name: &str,
    indices: &[crate::semantic_ir::Expression],
    context: &JvmContext,
    out: &mut String,
) -> Result<(), String> {
    let element = emit_jvm_semantic_array_address(name, indices, context, out)?;
    if element != JvmType::String {
        return Err(
            "semantic JVM string array reference does not match its resolved declaration"
                .to_string(),
        );
    }
    out.push_str("    aaload\n");
    Ok(())
}

fn emit_jvm_semantic_array_assignment(
    target: &crate::semantic_ir::Expression,
    operator: crate::semantic_ir::AssignmentOperator,
    value: &crate::semantic_ir::Expression,
    context: &JvmContext,
    out: &mut String,
) -> bool {
    use crate::semantic_ir::{ExpressionKind, SemanticValueType};
    let (name, indices) = match &target.kind {
        ExpressionKind::Index { name, index } => {
            (name.as_str(), std::slice::from_ref(index.as_ref()))
        }
        ExpressionKind::MultiIndex { name, indices } => (name.as_str(), indices.as_slice()),
        _ => return false,
    };
    let mut rendered = String::new();
    let element_type = match emit_jvm_semantic_array_address(name, indices, context, &mut rendered)
    {
        Ok(ty) => ty,
        Err(_) => return false,
    };
    if operator != crate::semantic_ir::AssignmentOperator::Assign {
        let JvmType::Numeric(target_type) = element_type else {
            return false;
        };
        if jvm_semantic_numeric_type(target.value_type) != Some(target_type) {
            return false;
        }
        rendered.push_str("    dup2\n");
        rendered.push_str(&format!("    {}\n", array_load_opcode(element_type)));
        let right_type = match emit_jvm_semantic_numeric_expression(value, &mut rendered, context) {
            Ok(ty) => ty,
            Err(_) => return false,
        };
        let promoted = if operator == crate::semantic_ir::AssignmentOperator::Divide {
            NumericType::Double
        } else {
            promote_numeric(target_type, right_type)
        };
        coerce_top(target_type, promoted, &mut rendered);
        coerce_top(right_type, promoted, &mut rendered);
        if operator == crate::semantic_ir::AssignmentOperator::Divide {
            rendered.push_str("    ddiv\n");
        } else {
            let opcode = match (operator, promoted) {
                (crate::semantic_ir::AssignmentOperator::Add, NumericType::Int) => "iadd",
                (crate::semantic_ir::AssignmentOperator::Subtract, NumericType::Int) => "isub",
                (crate::semantic_ir::AssignmentOperator::Multiply, NumericType::Int) => "imul",
                (crate::semantic_ir::AssignmentOperator::Add, NumericType::Long) => "ladd",
                (crate::semantic_ir::AssignmentOperator::Subtract, NumericType::Long) => "lsub",
                (crate::semantic_ir::AssignmentOperator::Multiply, NumericType::Long) => "lmul",
                (crate::semantic_ir::AssignmentOperator::Add, NumericType::Double) => "dadd",
                (crate::semantic_ir::AssignmentOperator::Subtract, NumericType::Double) => "dsub",
                (crate::semantic_ir::AssignmentOperator::Multiply, NumericType::Double) => "dmul",
                _ => return false,
            };
            rendered.push_str(&format!("    {opcode}\n"));
        }
        coerce_jvm_semantic_numeric(promoted, target_type, &mut rendered);
        emit_jvm_semantic_single_rounding(target.value_type, &mut rendered);
    } else {
        match (element_type, target.value_type) {
            (JvmType::String, SemanticValueType::String)
                if value.value_type == SemanticValueType::String =>
            {
                if emit_jvm_semantic_string_expression(value, &mut rendered, context).is_err() {
                    return false;
                }
            }
            (JvmType::Numeric(target_type), value_type)
                if jvm_semantic_numeric_type(value_type) == Some(target_type) =>
            {
                let value_type =
                    match emit_jvm_semantic_numeric_expression(value, &mut rendered, context) {
                        Ok(ty) => ty,
                        Err(_) => return false,
                    };
                coerce_jvm_semantic_numeric(value_type, target_type, &mut rendered);
            }
            _ => return false,
        }
    }
    emit_jvm_semantic_single_rounding(target.value_type, &mut rendered);
    rendered.push_str(&format!("    {}\n", array_store_opcode(element_type)));
    out.push_str(&rendered);
    true
}

fn emit_jvm_semantic_return(
    value: &crate::semantic_ir::ReturnValue,
    result: JvmType,
    returns_void: bool,
    context: &JvmContext,
    out: &mut String,
) -> bool {
    use crate::semantic_ir::{ReturnValue, SemanticValueType};
    let mut rendered = String::new();
    let return_instruction = match value {
        ReturnValue::Default if returns_void => "    return\n",
        ReturnValue::Default => return false,
        ReturnValue::Value(_) if returns_void => return false,
        ReturnValue::Value(expression) => match result {
            JvmType::String if expression.value_type == SemanticValueType::String => {
                if emit_jvm_semantic_string_expression(expression, &mut rendered, context).is_err()
                {
                    return false;
                }
                "    areturn\n"
            }
            JvmType::Numeric(target_type)
                if jvm_semantic_numeric_type(expression.value_type).is_some() =>
            {
                let value_type = match emit_jvm_semantic_numeric_expression(
                    expression,
                    &mut rendered,
                    context,
                ) {
                    Ok(ty) => ty,
                    Err(_) => return false,
                };
                coerce_jvm_semantic_numeric(value_type, target_type, &mut rendered);
                match target_type {
                    NumericType::Int => "    ireturn\n",
                    NumericType::Long => "    lreturn\n",
                    NumericType::Double => "    dreturn\n",
                }
            }
            _ => return false,
        },
    };
    context.emit_byref_writebacks(&mut rendered);
    rendered.push_str(return_instruction);
    out.push_str(&rendered);
    true
}

fn emit_jvm_semantic_print(
    destination: &crate::semantic_ir::PrintDestination,
    tokens: &[crate::semantic_ir::PrintToken],
    context: &JvmContext,
    out: &mut String,
) -> bool {
    use crate::semantic_ir::PrintDestination;
    match destination {
        PrintDestination::Standard { .. } => emit_jvm_semantic_print_tokens(tokens, context, out),
        PrintDestination::Channel {
            channel,
            using: None,
            ..
        } => emit_jvm_semantic_file_print(channel, tokens, context, out),
        _ => false,
    }
}

fn emit_jvm_semantic_file_print(
    channel: &crate::semantic_ir::Expression,
    tokens: &[crate::semantic_ir::PrintToken],
    context: &JvmContext,
    out: &mut String,
) -> bool {
    use crate::semantic_ir::{PrintToken, SemanticValueType};
    if channel.value_type == SemanticValueType::String {
        return false;
    }
    let trailing_separator = matches!(
        tokens.last(),
        Some(PrintToken::Comma { .. } | PrintToken::Semicolon { .. })
    );
    let mut rendered = format!(
        "    getstatic {}/bccFiles [Ljava/io/RandomAccessFile;\n",
        context.class_name
    );
    let channel_type = match emit_jvm_semantic_numeric_expression(channel, &mut rendered, context) {
        Ok(ty) => ty,
        Err(_) => return false,
    };
    coerce_jvm_semantic_numeric(channel_type, NumericType::Int, &mut rendered);
    rendered.push_str("    iconst_1\n    isub\n    aaload\n");
    for token in tokens {
        let PrintToken::Expression(expression) = token else {
            continue;
        };
        rendered.push_str("    dup\n");
        if expression.value_type == SemanticValueType::String {
            if emit_jvm_semantic_string_expression(expression, &mut rendered, context).is_err() {
                return false;
            }
        } else {
            let numeric_type =
                match emit_jvm_semantic_numeric_expression(expression, &mut rendered, context) {
                    Ok(ty) => ty,
                    Err(_) => return false,
                };
            match numeric_type {
                NumericType::Int => rendered.push_str(
                    "    invokestatic java/lang/Integer/toString (I)Ljava/lang/String;\n",
                ),
                NumericType::Long => rendered
                    .push_str("    invokestatic java/lang/Long/toString (J)Ljava/lang/String;\n"),
                NumericType::Double => rendered.push_str(&format!(
                    "    invokestatic {}/bccStr (D)Ljava/lang/String;\n",
                    context.class_name
                )),
            }
        }
        rendered.push_str(
            "    invokevirtual java/io/RandomAccessFile/writeBytes (Ljava/lang/String;)V\n",
        );
    }
    if !trailing_separator {
        rendered.push_str(
            "    dup\n    ldc \"\\n\"\n    invokevirtual java/io/RandomAccessFile/writeBytes (Ljava/lang/String;)V\n",
        );
    }
    rendered.push_str("    pop\n");
    out.push_str(&rendered);
    true
}

fn emit_jvm_semantic_write(
    channel: &crate::semantic_ir::Expression,
    values: &crate::semantic_ir::WriteValues,
    context: &JvmContext,
    out: &mut String,
) -> bool {
    use crate::semantic_ir::{SemanticValueType, WriteValues};
    if channel.value_type == SemanticValueType::String {
        return false;
    }
    let mut rendered = format!(
        "    getstatic {}/bccFiles [Ljava/io/RandomAccessFile;\n",
        context.class_name
    );
    let channel_type = match emit_jvm_semantic_numeric_expression(channel, &mut rendered, context) {
        Ok(ty) => ty,
        Err(_) => return false,
    };
    coerce_jvm_semantic_numeric(channel_type, NumericType::Int, &mut rendered);
    rendered.push_str("    iconst_1\n    isub\n    aaload\n");
    let expressions = match values {
        WriteValues::Omitted => &[][..],
        WriteValues::Values(values) => values.as_slice(),
    };
    for (index, expression) in expressions.iter().enumerate() {
        if index > 0 {
            rendered.push_str(
                "    dup\n    ldc \",\"\n    invokevirtual java/io/RandomAccessFile/writeBytes (Ljava/lang/String;)V\n",
            );
        }
        rendered.push_str("    dup\n");
        if expression.value_type == SemanticValueType::String {
            if emit_jvm_semantic_string_expression(expression, &mut rendered, context).is_err() {
                return false;
            }
            // WRITE uses CSV string fields: double embedded quotes and quote
            // the complete field before writing it to the selected channel.
            rendered.push_str(
                r#"    ldc "\""
    ldc "\"\""
    invokevirtual java/lang/String/replace (Ljava/lang/CharSequence;Ljava/lang/CharSequence;)Ljava/lang/String;
"#,
            );
            rendered.push_str(
                r#"    ldc "\""
    invokevirtual java/lang/String/concat (Ljava/lang/String;)Ljava/lang/String;
    ldc "\""
    swap
    invokevirtual java/lang/String/concat (Ljava/lang/String;)Ljava/lang/String;
"#,
            );
        } else {
            let numeric_type =
                match emit_jvm_semantic_numeric_expression(expression, &mut rendered, context) {
                    Ok(ty) => ty,
                    Err(_) => return false,
                };
            match numeric_type {
                NumericType::Int => rendered.push_str(
                    "    invokestatic java/lang/Integer/toString (I)Ljava/lang/String;\n",
                ),
                NumericType::Long => rendered
                    .push_str("    invokestatic java/lang/Long/toString (J)Ljava/lang/String;\n"),
                NumericType::Double => rendered.push_str(&format!(
                    "    invokestatic {}/bccStr (D)Ljava/lang/String;\n",
                    context.class_name
                )),
            }
        }
        rendered.push_str(
            "    invokevirtual java/io/RandomAccessFile/writeBytes (Ljava/lang/String;)V\n",
        );
    }
    rendered.push_str(
        "    dup\n    ldc \"\\n\"\n    invokevirtual java/io/RandomAccessFile/writeBytes (Ljava/lang/String;)V\n    pop\n",
    );
    out.push_str(&rendered);
    true
}

fn emit_jvm_semantic_print_tokens(
    tokens: &[crate::semantic_ir::PrintToken],
    context: &JvmContext,
    out: &mut String,
) -> bool {
    use crate::semantic_ir::{PrintToken, SemanticValueType};
    let last_expression = tokens
        .iter()
        .rposition(|token| matches!(token, PrintToken::Expression(_)));
    let trailing_separator = matches!(
        tokens.last(),
        Some(PrintToken::Comma { .. } | PrintToken::Semicolon { .. })
    );
    let mut rendered = String::new();
    for (index, token) in tokens.iter().enumerate() {
        let PrintToken::Expression(expression) = token else {
            continue;
        };
        rendered.push_str("    getstatic java/lang/System/out Ljava/io/PrintStream;\n");
        let descriptor = if expression.value_type == SemanticValueType::String {
            if emit_jvm_semantic_string_expression(expression, &mut rendered, context).is_err() {
                return false;
            }
            "(Ljava/lang/String;)V"
        } else {
            let ty = match emit_jvm_semantic_numeric_expression(expression, &mut rendered, context)
            {
                Ok(ty) => ty,
                Err(_) => return false,
            };
            if ty == NumericType::Double {
                rendered.push_str(&format!(
                    "    invokestatic {}/bccStr (D)Ljava/lang/String;\n",
                    context.class_name
                ));
                "(Ljava/lang/String;)V"
            } else {
                ty.print_descriptor()
            }
        };
        let method = if Some(index) == last_expression && !trailing_separator {
            "println"
        } else {
            "print"
        };
        rendered.push_str(&format!(
            "    invokevirtual java/io/PrintStream/{method} {descriptor}\n"
        ));
    }
    if trailing_separator {
        rendered.push_str("    getstatic java/lang/System/out Ljava/io/PrintStream;\n    invokevirtual java/io/PrintStream/flush ()V\n");
    }
    out.push_str(&rendered);
    true
}

fn emit_jvm_semantic_if(
    condition: &crate::semantic_ir::Expression,
    then_body: &[crate::semantic_ir::SemanticStatement],
    else_body: &[crate::semantic_ir::SemanticStatement],
    context: &JvmContext,
    state: &mut JvmSemanticState,
    out: &mut String,
) -> bool {
    emit_jvm_semantic_if_in_loop(
        condition,
        then_body,
        else_body,
        context,
        &mut Vec::new(),
        &mut Vec::new(),
        state,
        out,
    )
}

fn emit_jvm_semantic_if_in_loop(
    condition: &crate::semantic_ir::Expression,
    then_body: &[crate::semantic_ir::SemanticStatement],
    else_body: &[crate::semantic_ir::SemanticStatement],
    context: &JvmContext,
    loop_exits: &mut Vec<String>,
    loop_continues: &mut Vec<String>,
    state: &mut JvmSemanticState,
    out: &mut String,
) -> bool {
    let mut rendered = String::new();
    let condition_type =
        match emit_jvm_semantic_numeric_expression(condition, &mut rendered, context) {
            Ok(ty) => ty,
            Err(_) => return false,
        };
    let alternative = next_condition_label(context);
    let done = next_condition_label(context);
    emit_jvm_semantic_truth_branch(&mut rendered, condition_type, &alternative, false);
    if !emit_jvm_semantic_block(
        then_body,
        context,
        loop_exits,
        loop_continues,
        state,
        &mut rendered,
    ) {
        return false;
    }
    if else_body.is_empty() {
        rendered.push_str(&format!("{alternative}:\n"));
    } else {
        rendered.push_str(&format!("    goto {done}\n{alternative}:\n"));
        if !emit_jvm_semantic_block(
            else_body,
            context,
            loop_exits,
            loop_continues,
            state,
            &mut rendered,
        ) {
            return false;
        }
        rendered.push_str(&format!("{done}:\n"));
    }
    out.push_str(&rendered);
    true
}

fn emit_jvm_semantic_while(
    condition: &crate::semantic_ir::Expression,
    body: &[crate::semantic_ir::SemanticStatement],
    context: &JvmContext,
    state: &mut JvmSemanticState,
    out: &mut String,
) -> bool {
    emit_jvm_semantic_while_in_loop(
        condition,
        body,
        context,
        &mut Vec::new(),
        &mut Vec::new(),
        state,
        out,
    )
}

fn emit_jvm_semantic_while_in_loop(
    condition: &crate::semantic_ir::Expression,
    body: &[crate::semantic_ir::SemanticStatement],
    context: &JvmContext,
    loop_exits: &mut Vec<String>,
    loop_continues: &mut Vec<String>,
    state: &mut JvmSemanticState,
    out: &mut String,
) -> bool {
    let header = next_condition_label(context);
    let done = next_condition_label(context);
    let mut rendered = format!("{header}:\n");
    let condition_type =
        match emit_jvm_semantic_numeric_expression(condition, &mut rendered, context) {
            Ok(ty) => ty,
            Err(_) => return false,
        };
    emit_jvm_semantic_truth_branch(&mut rendered, condition_type, &done, false);
    loop_exits.push(done.clone());
    loop_continues.push(header.clone());
    let body_handled = emit_jvm_semantic_block(
        body,
        context,
        loop_exits,
        loop_continues,
        state,
        &mut rendered,
    );
    loop_exits.pop();
    loop_continues.pop();
    if !body_handled {
        return false;
    }
    rendered.push_str(&format!("    goto {header}\n{done}:\n"));
    out.push_str(&rendered);
    true
}

fn emit_jvm_semantic_for(
    variable: &str,
    start: &crate::semantic_ir::Expression,
    bounds: &crate::semantic_ir::ForBounds,
    body: &[crate::semantic_ir::SemanticStatement],
    context: &JvmContext,
    state: &mut JvmSemanticState,
    out: &mut String,
) -> bool {
    emit_jvm_semantic_for_in_loop(
        variable,
        start,
        bounds,
        body,
        context,
        &mut Vec::new(),
        &mut Vec::new(),
        state,
        out,
    )
}

fn emit_jvm_semantic_for_in_loop(
    variable: &str,
    start: &crate::semantic_ir::Expression,
    bounds: &crate::semantic_ir::ForBounds,
    body: &[crate::semantic_ir::SemanticStatement],
    context: &JvmContext,
    loop_exits: &mut Vec<String>,
    loop_continues: &mut Vec<String>,
    state: &mut JvmSemanticState,
    out: &mut String,
) -> bool {
    use crate::semantic_ir::ForBounds;
    let ident = BasicIdent::parse(variable);
    let variable = match context.variable(&ident) {
        Ok(variable) if variable.ty == JvmType::Numeric(NumericType::Int) => variable,
        _ => return false,
    };
    let (limit, step, dynamic_step) = match bounds {
        ForBounds::To { limit, step } => {
            let step_value = step
                .as_ref()
                .map(semantic_integer_literal)
                .unwrap_or(Some(1));
            if step_value == Some(0) {
                return false;
            }
            (limit, step_value, step_value.is_none())
        }
        ForBounds::Downto { limit, step } => {
            let Ok(step) = i32::try_from(*step) else {
                return false;
            };
            if step == 0 {
                return false;
            }
            (limit, Some(step), false)
        }
    };
    if !matches!(
        start.value_type,
        crate::semantic_ir::SemanticValueType::Integer
            | crate::semantic_ir::SemanticValueType::Long
    ) || !matches!(
        limit.value_type,
        crate::semantic_ir::SemanticValueType::Integer
            | crate::semantic_ir::SemanticValueType::Long
    ) {
        return false;
    }
    let id = context.condition_label.get();
    context.condition_label.set(id + 1);
    let header = format!("L_for_{id}_top");
    let continuation = format!("L_for_{id}_continue");
    let done = format!("L_for_{id}_end");
    let mut rendered = String::new();
    let start_variable = context.reserve_semantic_int_local();
    let start_type = match emit_jvm_semantic_numeric_expression(start, &mut rendered, context) {
        Ok(ty) => ty,
        Err(_) => {
            context.condition_label.set(id);
            return false;
        }
    };
    coerce_jvm_semantic_numeric(start_type, NumericType::Int, &mut rendered);
    emit_store(start_variable, &mut rendered, context);
    let limit_variable = context.reserve_semantic_int_local();
    let limit_type = match emit_jvm_semantic_numeric_expression(limit, &mut rendered, context) {
        Ok(ty) => ty,
        Err(_) => {
            context.condition_label.set(id);
            return false;
        }
    };
    coerce_jvm_semantic_numeric(limit_type, NumericType::Int, &mut rendered);
    emit_store(limit_variable, &mut rendered, context);
    let step_variable = if dynamic_step {
        let ForBounds::To {
            step: Some(step_expression),
            ..
        } = bounds
        else {
            context.condition_label.set(id);
            return false;
        };
        if !matches!(
            step_expression.value_type,
            crate::semantic_ir::SemanticValueType::Integer
                | crate::semantic_ir::SemanticValueType::Long
        ) {
            context.condition_label.set(id);
            return false;
        }
        let temporary = context.reserve_semantic_int_local();
        let step_type =
            match emit_jvm_semantic_numeric_expression(step_expression, &mut rendered, context) {
                Ok(ty) => ty,
                Err(_) => {
                    context.condition_label.set(id);
                    return false;
                }
            };
        coerce_jvm_semantic_numeric(step_type, NumericType::Int, &mut rendered);
        emit_store(temporary, &mut rendered, context);
        Some(temporary)
    } else {
        None
    };
    emit_load(start_variable, &mut rendered, context);
    emit_store(variable, &mut rendered, context);
    rendered.push_str(&format!("{header}:\n"));
    if let Some(step_variable) = step_variable {
        let ascending = format!("L_for_{id}_ascending");
        let body_label = format!("L_for_{id}_body");
        emit_load(step_variable, &mut rendered, context);
        rendered.push_str(&format!("    ifge {ascending}\n"));
        emit_load(variable, &mut rendered, context);
        emit_load(limit_variable, &mut rendered, context);
        rendered.push_str(&format!(
            "    if_icmplt {done}\n    goto {body_label}\n{ascending}:\n"
        ));
        emit_load(variable, &mut rendered, context);
        emit_load(limit_variable, &mut rendered, context);
        rendered.push_str(&format!("    if_icmpgt {done}\n{body_label}:\n"));
    } else {
        emit_load(variable, &mut rendered, context);
        emit_load(limit_variable, &mut rendered, context);
        if step.unwrap_or(1) < 0 {
            rendered.push_str(&format!("    if_icmplt {done}\n"));
        } else {
            rendered.push_str(&format!("    if_icmpgt {done}\n"));
        }
    }
    loop_exits.push(done.clone());
    loop_continues.push(continuation.clone());
    let body_handled = emit_jvm_semantic_block(
        body,
        context,
        loop_exits,
        loop_continues,
        state,
        &mut rendered,
    );
    loop_exits.pop();
    loop_continues.pop();
    if !body_handled {
        context.condition_label.set(id);
        return false;
    }
    rendered.push_str(&format!("{continuation}:\n"));
    emit_load(variable, &mut rendered, context);
    if let Some(step_variable) = step_variable {
        emit_load(step_variable, &mut rendered, context);
    } else {
        rendered.push_str(&format!("    ldc {}\n", step.unwrap_or(1)));
    }
    rendered.push_str("    iadd\n");
    emit_store(variable, &mut rendered, context);
    rendered.push_str(&format!("    goto {header}\n{done}:\n"));
    out.push_str(&rendered);
    true
}

fn semantic_integer_literal(expression: &crate::semantic_ir::Expression) -> Option<i32> {
    use crate::semantic_ir::ExpressionKind;
    match &expression.kind {
        ExpressionKind::Literal(value) => value.parse().ok(),
        ExpressionKind::Parenthesized(inner) => semantic_integer_literal(inner),
        ExpressionKind::Unary { operator, operand } if operator == "-" => {
            semantic_integer_literal(operand)?.checked_neg()
        }
        _ => None,
    }
}

fn emit_jvm_semantic_do(
    pre_condition: Option<&crate::semantic_ir::LoopCondition>,
    post_condition: Option<&crate::semantic_ir::LoopCondition>,
    body: &[crate::semantic_ir::SemanticStatement],
    context: &JvmContext,
    state: &mut JvmSemanticState,
    out: &mut String,
) -> bool {
    emit_jvm_semantic_do_in_loop(
        pre_condition,
        post_condition,
        body,
        context,
        &mut Vec::new(),
        &mut Vec::new(),
        state,
        out,
    )
}

fn emit_jvm_semantic_do_in_loop(
    pre_condition: Option<&crate::semantic_ir::LoopCondition>,
    post_condition: Option<&crate::semantic_ir::LoopCondition>,
    body: &[crate::semantic_ir::SemanticStatement],
    context: &JvmContext,
    loop_exits: &mut Vec<String>,
    loop_continues: &mut Vec<String>,
    state: &mut JvmSemanticState,
    out: &mut String,
) -> bool {
    let id = context.condition_label.get();
    context.condition_label.set(id + 1);
    let top = format!("L_do_{id}_top");
    let continuation = format!("L_do_{id}_continue");
    let done = format!("L_do_{id}_end");
    let mut rendered = format!("{top}:\n");
    if let Some(condition) = pre_condition {
        if !emit_jvm_semantic_loop_condition(condition, &done, context, &mut rendered) {
            context.condition_label.set(id);
            return false;
        }
    }
    loop_exits.push(done.clone());
    loop_continues.push(continuation.clone());
    let body_handled = emit_jvm_semantic_block(
        body,
        context,
        loop_exits,
        loop_continues,
        state,
        &mut rendered,
    );
    loop_continues.pop();
    loop_exits.pop();
    if !body_handled {
        context.condition_label.set(id);
        return false;
    }
    rendered.push_str(&format!("{continuation}:\n"));
    if let Some(condition) = post_condition {
        if !emit_jvm_semantic_loop_condition(condition, &done, context, &mut rendered) {
            context.condition_label.set(id);
            return false;
        }
    }
    rendered.push_str(&format!("    goto {top}\n{done}:\n"));
    out.push_str(&rendered);
    true
}

fn emit_jvm_semantic_loop_condition(
    condition: &crate::semantic_ir::LoopCondition,
    done: &str,
    context: &JvmContext,
    out: &mut String,
) -> bool {
    let ty = match emit_jvm_semantic_numeric_expression(&condition.value, out, context) {
        Ok(ty) => ty,
        Err(_) => return false,
    };
    let until = condition.kind == crate::semantic_ir::LoopConditionKind::Until;
    emit_jvm_semantic_truth_branch(out, ty, done, until);
    true
}

fn emit_jvm_semantic_select_case(
    selector: &crate::semantic_ir::Expression,
    cases: &[crate::semantic_ir::CaseClause],
    else_body: &[crate::semantic_ir::SemanticStatement],
    context: &JvmContext,
    state: &mut JvmSemanticState,
    out: &mut String,
) -> bool {
    emit_jvm_semantic_select_case_in_loop(
        selector,
        cases,
        else_body,
        context,
        &mut Vec::new(),
        &mut Vec::new(),
        state,
        out,
    )
}

fn emit_jvm_semantic_select_case_in_loop(
    selector: &crate::semantic_ir::Expression,
    cases: &[crate::semantic_ir::CaseClause],
    else_body: &[crate::semantic_ir::SemanticStatement],
    context: &JvmContext,
    loop_exits: &mut Vec<String>,
    loop_continues: &mut Vec<String>,
    state: &mut JvmSemanticState,
    out: &mut String,
) -> bool {
    use crate::semantic_ir::{CaseValue, ComparisonOperator, SemanticValueType};
    let id = context.condition_label.get();
    context.condition_label.set(id + 1);
    let end = format!("L_select_{id}_end");
    let case_labels = (0..cases.len())
        .map(|index| format!("L_select_{id}_case_{index}"))
        .collect::<Vec<_>>();
    let string_selector = selector.value_type == SemanticValueType::String;
    let rendered = (|| {
        let mut rendered = String::new();
        if string_selector {
            emit_jvm_semantic_string_expression(selector, &mut rendered, context).ok()?;
        } else {
            if jvm_semantic_numeric_type(selector.value_type) != Some(NumericType::Int) {
                return None;
            }
            emit_jvm_semantic_numeric_expression(selector, &mut rendered, context).ok()?;
        }
        for (case_index, case) in cases.iter().enumerate() {
            let next = format!("L_select_{id}_next_{case_index}");
            for (value_index, value) in case.values.iter().enumerate() {
                match value {
                    CaseValue::Value {
                        first,
                        range_end: None,
                        ..
                    } if string_selector && first.value_type == SemanticValueType::String => {
                        rendered.push_str("    dup\n");
                        emit_jvm_semantic_string_expression(first, &mut rendered, context).ok()?;
                        rendered.push_str(&format!(
                            "    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z\n    ifne {}\n",
                            case_labels[case_index]
                        ));
                    }
                    CaseValue::Value {
                        first,
                        range_end: None,
                        ..
                    } if !string_selector => {
                        rendered.push_str("    dup\n");
                        let ty =
                            emit_jvm_semantic_numeric_expression(first, &mut rendered, context)
                                .ok()?;
                        coerce_jvm_semantic_numeric(ty, NumericType::Int, &mut rendered);
                        rendered
                            .push_str(&format!("    isub\n    ifeq {}\n", case_labels[case_index]));
                    }
                    CaseValue::Value {
                        first,
                        range_end: Some(last),
                        ..
                    } if !string_selector => {
                        let next_value =
                            format!("L_select_{id}_case_{case_index}_value_{value_index}");
                        rendered.push_str("    dup\n");
                        let first_type =
                            emit_jvm_semantic_numeric_expression(first, &mut rendered, context)
                                .ok()?;
                        coerce_jvm_semantic_numeric(first_type, NumericType::Int, &mut rendered);
                        rendered.push_str(&format!("    if_icmplt {next_value}\n    dup\n"));
                        let last_type =
                            emit_jvm_semantic_numeric_expression(last, &mut rendered, context)
                                .ok()?;
                        coerce_jvm_semantic_numeric(last_type, NumericType::Int, &mut rendered);
                        rendered.push_str(&format!(
                            "    if_icmple {}\n{next_value}:\n",
                            case_labels[case_index]
                        ));
                    }
                    CaseValue::Comparison {
                        operator, value, ..
                    } if !string_selector => {
                        rendered.push_str("    dup\n");
                        let ty =
                            emit_jvm_semantic_numeric_expression(value, &mut rendered, context)
                                .ok()?;
                        coerce_jvm_semantic_numeric(ty, NumericType::Int, &mut rendered);
                        let branch = match operator {
                            ComparisonOperator::NotEqual => "if_icmpne",
                            ComparisonOperator::LessOrEqual => "if_icmple",
                            ComparisonOperator::GreaterOrEqual => "if_icmpge",
                            ComparisonOperator::Equal => "if_icmpeq",
                            ComparisonOperator::Less => "if_icmplt",
                            ComparisonOperator::Greater => "if_icmpgt",
                        };
                        rendered.push_str(&format!("    {branch} {}\n", case_labels[case_index]));
                    }
                    _ => return None,
                }
            }
            rendered.push_str(&format!("    goto {next}\n{next}:\n"));
        }
        rendered.push_str("    pop\n");
        if !emit_jvm_semantic_block(
            else_body,
            context,
            loop_exits,
            loop_continues,
            state,
            &mut rendered,
        ) {
            return None;
        }
        rendered.push_str(&format!("    goto {end}\n"));
        for (case, label) in cases.iter().zip(case_labels.iter()) {
            rendered.push_str(&format!("{label}:\n    pop\n"));
            if !emit_jvm_semantic_block(
                &case.body,
                context,
                loop_exits,
                loop_continues,
                state,
                &mut rendered,
            ) {
                return None;
            }
            rendered.push_str(&format!("    goto {end}\n"));
        }
        rendered.push_str(&format!("{end}:\n"));
        Some(rendered)
    })();
    if let Some(rendered) = rendered {
        out.push_str(&rendered);
        true
    } else {
        context.condition_label.set(id);
        false
    }
}

#[derive(Clone)]
struct JvmSemanticState {
    source_filename: String,
    next_label: usize,
    exception_handlers: Vec<JvmExceptionHandler>,
}

fn emit_jvm_semantic_try(
    try_body: &[crate::semantic_ir::SemanticStatement],
    catch: Option<&crate::semantic_ir::CatchBinding>,
    finally_body: &[crate::semantic_ir::SemanticStatement],
    context: &JvmContext,
    loop_exits: &mut Vec<String>,
    loop_continues: &mut Vec<String>,
    state: &mut JvmSemanticState,
    out: &mut String,
) -> bool {
    let Some(catch) = catch else {
        return false;
    };
    let id = state.next_label;
    let start = format!("L_try_{id}_start");
    let end = format!("L_try_{id}_end");
    let handler = format!("L_try_{id}_catch");
    let finish = format!("L_try_{id}_finish");
    let mut rendered = String::new();
    if !emit_jvm_semantic_block(
        try_body,
        context,
        loop_exits,
        loop_continues,
        state,
        &mut rendered,
    ) {
        return false;
    }
    let mut error_ident = BasicIdent::parse(&catch.error);
    error_ident.suffix = catch
        .error_type
        .suffix()
        .and_then(TypeSuffix::from_char);
    let mut line_ident = BasicIdent::parse(&catch.line);
    line_ident.suffix = catch.line_type.suffix().and_then(TypeSuffix::from_char);
    let error = match context.variable(&error_ident) {
        Ok(variable) => variable,
        Err(_) => return false,
    };
    let line = match context.variable(&line_ident) {
        Ok(variable) => variable,
        Err(_) => return false,
    };
    let source = match catch.source.as_deref() {
        Some(name) => {
            let mut ident = BasicIdent::parse(name);
            ident.suffix = catch
                .source_type
                .and_then(crate::semantic_ir::SemanticValueType::suffix)
                .and_then(TypeSuffix::from_char);
            match context.variable(&ident) {
                Ok(variable) => Some(variable),
                Err(_) => return false,
            }
        }
        None => None,
    };
    rendered.insert_str(0, &format!("{start}:\n"));
    rendered.push_str(&format!("    goto {finish}\n{end}:\n{handler}:\n"));
    rendered.push_str("    invokevirtual java/lang/Throwable/getMessage ()Ljava/lang/String;\n    invokestatic java/lang/Integer/parseInt (Ljava/lang/String;)I\n");
    emit_store(error, &mut rendered, context);
    rendered.push_str("    iconst_0\n");
    emit_store(line, &mut rendered, context);
    if let Some(source) = source {
        rendered.push_str(&format!(
            "    ldc \"{}\"\n",
            escape_jvm_string(&crate::diagnostics::display_source_filename(
                &state.source_filename
            ))
        ));
        emit_store(source, &mut rendered, context);
    }
    let matched = format!("L_try_{id}_matched");
    let rethrow = format!("L_try_{id}_rethrow");
    for filter in &catch.filters {
        emit_load(error, &mut rendered, context);
        let ty = match emit_jvm_semantic_numeric_expression(filter, &mut rendered, context) {
            Ok(ty) => ty,
            Err(_) => return false,
        };
        coerce_jvm_semantic_numeric(ty, NumericType::Int, &mut rendered);
        rendered.push_str(&format!("    if_icmpeq {matched}\n"));
    }
    if !catch.filters.is_empty() {
        rendered.push_str(&format!("    goto {rethrow}\n{matched}:\n"));
    }
    if !emit_jvm_semantic_block(
        &catch.body,
        context,
        loop_exits,
        loop_continues,
        state,
        &mut rendered,
    ) {
        return false;
    }
    rendered.push_str(&format!("    goto {finish}\n"));
    if !catch.filters.is_empty() {
        rendered.push_str(&format!("{rethrow}:\n"));
        if !emit_jvm_semantic_block(
            finally_body,
            context,
            loop_exits,
            loop_continues,
            state,
            &mut rendered,
        ) {
            return false;
        }
        rendered.push_str("    new java/lang/RuntimeException\n    dup\n");
        emit_load(error, &mut rendered, context);
        rendered.push_str("    invokestatic java/lang/Integer/toString (I)Ljava/lang/String;\n    invokespecial java/lang/RuntimeException/<init> (Ljava/lang/String;)V\n    athrow\n");
    }
    rendered.push_str(&format!("{finish}:\n"));
    if !emit_jvm_semantic_block(
        finally_body,
        context,
        loop_exits,
        loop_continues,
        state,
        &mut rendered,
    ) {
        return false;
    }

    state.exception_handlers.push(JvmExceptionHandler {
        start,
        end,
        handler,
    });
    state.next_label += 1;
    out.push_str(&rendered);
    true
}

fn emit_jvm_semantic_comment(block: bool, text: &str, out: &mut String) {
    if !block {
        let body = text
            .strip_prefix('\'')
            .or_else(|| text.strip_prefix("//"))
            .unwrap_or(text);
        let body = body.trim_start();
        if body.is_empty() {
            out.push_str("    ;\n");
        } else {
            out.push_str(&format!("    ; {body}\n"));
        }
        return;
    }

    let body = text
        .strip_prefix("/*")
        .and_then(|body| body.strip_suffix("*/"))
        .unwrap_or(text);
    for line in body.lines() {
        let trimmed = line.trim();
        let comment = trimmed.strip_prefix('*').map(str::trim).unwrap_or(trimmed);
        if comment.is_empty() {
            out.push_str("    ;\n");
        } else {
            out.push_str(&format!("    ; {comment}\n"));
        }
    }
}

fn emit_jvm_semantic_block(
    statements: &[crate::semantic_ir::SemanticStatement],
    context: &JvmContext,
    loop_exits: &mut Vec<String>,
    loop_continues: &mut Vec<String>,
    state: &mut JvmSemanticState,
    out: &mut String,
) -> bool {
    use crate::semantic_ir::SemanticStatementKind as Kind;
    let initial_state = state.clone();
    let mut rendered = String::new();
    for statement in statements {
        let mut node = String::new();
        let handled = match &statement.kind {
            Kind::Label(name) => {
                node.push_str(&format!("{}:\n", jvm_label(&name.name)));
                true
            }
            Kind::Goto(target) => {
                node.push_str(&format!("    goto {}\n", jvm_label(&target.name)));
                true
            }
            Kind::Line(children) => emit_jvm_semantic_block(
                children,
                context,
                loop_exits,
                loop_continues,
                state,
                &mut node,
            ),
            Kind::Assignment {
                target,
                operator,
                value,
            } => emit_jvm_semantic_assignment(target, *operator, value, context, &mut node),
            // Scalar and array storage is allocated from resolver facts
            // before statement dispatch; DIM therefore has no instruction.
            Kind::Dim(dimensions) => emit_jvm_semantic_dim(dimensions, context, &mut node),
            Kind::Const { name, value, .. } => emit_jvm_semantic_const(name, value, context, &mut node),
            Kind::Expression(crate::semantic_ir::Expression {
                kind: crate::semantic_ir::ExpressionKind::Call { name, arguments },
                ..
            }) => emit_jvm_semantic_discarded_callable_call(name, arguments, context, &mut node)
                .is_ok(),
            Kind::Expression(expression) => {
                emit_jvm_semantic_discarded_expression(expression, context, &mut node)
            }
            Kind::MidAssign {
                target,
                start,
                length,
                value,
            } => emit_jvm_semantic_mid_assign(
                target,
                start,
                length.as_ref(),
                value,
                context,
                &mut node,
            ),
            Kind::Swap { left, right } => emit_jvm_semantic_swap(left, right, context, &mut node),
            Kind::Error(code) | Kind::Throw(crate::semantic_ir::ThrowValue::Value(code)) => {
                emit_jvm_semantic_error(code, context, &mut node)
            }
            Kind::Cls => emit_terminal_escape("\u{1b}[2J\u{1b}[H", &mut node).is_ok(),
            Kind::Beep => emit_terminal_escape("\u{7}", &mut node).is_ok(),
            Kind::Locate { row, column } => {
                emit_jvm_semantic_locate(row, column, context, &mut node)
            }
            Kind::Color {
                foreground,
                background,
            } => emit_jvm_semantic_color(foreground, background.as_ref(), context, &mut node),
            Kind::Input { source, targets } => {
                emit_jvm_semantic_input(source, targets, context, &mut node)
            }
            Kind::LineInput { channel, target } => {
                emit_jvm_semantic_line_input(channel, target, context, &mut node)
            }
            Kind::Open {
                path,
                mode,
                channel,
                length: Some(length),
            } if mode.kind == crate::semantic_ir::OpenModeKind::Random => {
                emit_jvm_semantic_random_open(path, channel, length, context, &mut node)
            }
            Kind::Open {
                path,
                mode,
                channel,
                length: None,
            } if matches!(
                mode.kind,
                crate::semantic_ir::OpenModeKind::Output
                    | crate::semantic_ir::OpenModeKind::Append
            ) => emit_jvm_semantic_output_open(path, mode.kind, channel, context, &mut node),
            Kind::Close(channel) => emit_jvm_semantic_file_close(channel, context, &mut node),
            Kind::Read(targets) => emit_jvm_semantic_read(targets, context, &mut node),
            Kind::Restore(target) => {
                emit_jvm_semantic_restore(target.as_ref(), context, &mut node)
            }
            Kind::OnBranch {
                selector,
                branch: crate::semantic_ir::BranchKind::Goto,
                targets,
            } => emit_jvm_semantic_on_goto(selector, targets, context, state, &mut node),
            Kind::Kill(path) => emit_jvm_semantic_kill(path, context, &mut node),
            Kind::Rename {
                source,
                destination,
            } => emit_jvm_semantic_rename(source, destination, context, &mut node),
            Kind::Lset { target, value } => {
                emit_jvm_semantic_field_set(true, target, value, context, &mut node)
            }
            Kind::Rset { target, value } => {
                emit_jvm_semantic_field_set(false, target, value, context, &mut node)
            }
            Kind::Seek { channel, position } => {
                emit_jvm_semantic_seek(channel, position, context, &mut node)
            }
            Kind::Get { channel, position } => {
                emit_jvm_semantic_get_put(true, channel, position.as_ref(), context, &mut node)
            }
            Kind::Put { channel, position } => {
                emit_jvm_semantic_get_put(false, channel, position.as_ref(), context, &mut node)
            }
            Kind::Field { .. } => true,
            // Callable GLOBAL affects slot ownership before block emission;
            // the declaration itself emits no JVM instruction.
            Kind::Global { .. } => true,
            // JVM storage has method/class lifetime, so CLEAR emits no
            // instruction while remaining owned by typed IR dispatch.
            Kind::Clear => true,
            Kind::Erase(names) => emit_jvm_semantic_erase(names, context, &mut node),
            // Preserve typed comments in the emitted assembly stream.
            Kind::Comment { block, text } => {
                emit_jvm_semantic_comment(*block, text, &mut node);
                true
            }
            // DATA values are materialized in the module's data table before
            // statement dispatch; the declaration itself emits no bytecode.
            Kind::Data(_) => true,
            // Record file declarations are consumed by JVM record layout and
            // file-I/O setup before statement dispatch.
            Kind::FileDeclaration { .. } => true,
            Kind::End if context.semantic_return.is_some() => {
                emit_inkey_restore(context, &mut node);
                node.push_str("    iconst_0\n    invokestatic java/lang/System/exit (I)V\n");
                true
            }
            Kind::End => {
                emit_inkey_restore(context, &mut node);
                node.push_str("    return\n");
                true
            }
            Kind::Stop | Kind::System => {
                node.push_str("    iconst_0\n    invokestatic java/lang/System/exit (I)V\n");
                true
            }
            Kind::Return(value) => context
                .semantic_return
                .is_some_and(|(result, returns_void)| {
                    emit_jvm_semantic_return(value, result, returns_void, context, &mut node)
                }),
            Kind::Print {
                destination,
                tokens,
            } => emit_jvm_semantic_print(destination, tokens, context, &mut node),
            Kind::Write { channel, values } => {
                emit_jvm_semantic_write(channel, values, context, &mut node)
            }
            Kind::Lprint {
                using: None,
                tokens,
            } => emit_jvm_semantic_print_tokens(tokens, context, &mut node),
            Kind::If {
                condition,
                then_body,
                else_body,
                ..
            } => emit_jvm_semantic_if_in_loop(
                condition,
                then_body,
                else_body,
                context,
                loop_exits,
                loop_continues,
                state,
                &mut node,
            ),
            Kind::While {
                condition,
                body: loop_body,
            } => emit_jvm_semantic_while_in_loop(
                condition,
                loop_body,
                context,
                loop_exits,
                loop_continues,
                state,
                &mut node,
            ),
            Kind::For {
                variable,
                start,
                bounds,
                body: loop_body,
                ..
            } => emit_jvm_semantic_for_in_loop(
                variable,
                start,
                bounds,
                loop_body,
                context,
                loop_exits,
                loop_continues,
                state,
                &mut node,
            ),
            Kind::Do {
                pre_condition,
                post_condition,
                body: loop_body,
            } => emit_jvm_semantic_do_in_loop(
                pre_condition.as_ref(),
                post_condition.as_ref(),
                loop_body,
                context,
                loop_exits,
                loop_continues,
                state,
                &mut node,
            ),
            Kind::SelectCase {
                selector,
                cases,
                else_body,
            } => emit_jvm_semantic_select_case_in_loop(
                selector,
                cases,
                else_body,
                context,
                loop_exits,
                loop_continues,
                state,
                &mut node,
            ),
            Kind::Try {
                body: try_body,
                catch,
                finally_body,
            } => emit_jvm_semantic_try(
                try_body,
                catch.as_ref(),
                finally_body,
                context,
                loop_exits,
                loop_continues,
                state,
                &mut node,
            ),
            Kind::Exit if !loop_exits.is_empty() => {
                node.push_str(&format!("    goto {}\n", loop_exits.last().unwrap()));
                true
            }
            Kind::Continue if !loop_continues.is_empty() => {
                node.push_str(&format!("    goto {}\n", loop_continues.last().unwrap()));
                true
            }
            _ => false,
        };
        if !handled {
            *state = initial_state;
            return false;
        }
        rendered.push_str(&node);
    }
    out.push_str(&rendered);
    true
}

fn emit_jvm_semantic_module(
    module: &crate::semantic_ir::SemanticModule,
    context: &JvmContext,
    loop_exits: &mut Vec<String>,
    loop_continues: &mut Vec<String>,
    state: &mut JvmSemanticState,
    out: &mut String,
) -> bool {
    if module.statement_sources.len() != module.statements.len() {
        return false;
    }
    let initial_state = state.clone();
    let initial_loop_exits = loop_exits.clone();
    let initial_loop_continues = loop_continues.clone();
    let mut rendered = String::new();
    for (statement, source_index) in module.statements.iter().zip(&module.statement_sources) {
        let Some(source) = module.sources.get(*source_index) else {
            *state = initial_state;
            *loop_exits = initial_loop_exits;
            *loop_continues = initial_loop_continues;
            return false;
        };
        state.source_filename.clone_from(&source.filename);
        let mut statement_output = String::new();
        if !emit_jvm_semantic_block(
            std::slice::from_ref(statement),
            context,
            loop_exits,
            loop_continues,
            state,
            &mut statement_output,
        ) {
            *state = initial_state;
            *loop_exits = initial_loop_exits;
            *loop_continues = initial_loop_continues;
            return false;
        }
        rendered.push_str(&statement_output);
    }
    out.push_str(&rendered);
    true
}

fn emit_jvm_semantic_input(
    source: &crate::semantic_ir::InputSource,
    targets: &[crate::semantic_ir::Expression],
    context: &JvmContext,
    out: &mut String,
) -> bool {
    use crate::semantic_ir::{ExpressionKind, InputSource, SemanticValueType};
    let InputSource::Console(prompt) = source else {
        return false;
    };
    let [target] = targets else {
        return false;
    };
    let (target_type, scalar, array) = match &target.kind {
        ExpressionKind::Name(name) => {
            let Ok(variable) = context.variable(&BasicIdent::parse(name)) else {
                return false;
            };
            (variable.ty, Some(variable), None)
        }
        ExpressionKind::Index { name, index } => {
            let mut address = String::new();
            let element = match emit_jvm_semantic_array_address(
                name,
                std::slice::from_ref(index.as_ref()),
                context,
                &mut address,
            ) {
                Ok(element) => element,
                Err(_) => return false,
            };
            (element, None, Some(address))
        }
        ExpressionKind::MultiIndex { name, indices } => {
            let mut address = String::new();
            let element =
                match emit_jvm_semantic_array_address(name, indices, context, &mut address) {
                    Ok(element) => element,
                    Err(_) => return false,
                };
            (element, None, Some(address))
        }
        _ => return false,
    };
    if match target_type {
        JvmType::String => target.value_type != SemanticValueType::String,
        JvmType::Numeric(ty) => jvm_semantic_numeric_type(target.value_type) != Some(ty),
    } {
        return false;
    }
    let prompt_text = prompt.as_ref().map(|prompt| {
        prompt
            .text
            .strip_prefix('"')
            .and_then(|text| text.strip_suffix('"'))
            .unwrap_or(prompt.text.as_str())
            .replace("\"\"", "\"")
    });
    let mut rendered = String::new();
    if let Some(prompt) = prompt_text {
        rendered.push_str(&format!(
            "    getstatic java/lang/System/out Ljava/io/PrintStream;\n    ldc \"{}? \"\n    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V\n    getstatic java/lang/System/out Ljava/io/PrintStream;\n    invokevirtual java/io/PrintStream/flush ()V\n",
            escape_jvm_string(&prompt)
        ));
    }
    if let Some(address) = array {
        rendered.push_str(&address);
    }
    emit_inkey_restore(context, &mut rendered);
    rendered.push_str(&format!(
        "    getstatic {}/bccStdin Ljava/io/BufferedReader;\n    invokevirtual java/io/BufferedReader/readLine ()Ljava/lang/String;\n",
        context.class_name
    ));
    emit_inkey_setup(context, &mut rendered);
    match target_type {
        JvmType::String if target.value_type == SemanticValueType::String => {}
        JvmType::Numeric(ty) if jvm_semantic_numeric_type(target.value_type) == Some(ty) => {
            rendered.push_str("    invokevirtual java/lang/String/trim ()Ljava/lang/String;\n");
            rendered.push_str(match ty {
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
        _ => return false,
    }
    if scalar.is_some() {
        emit_jvm_semantic_single_rounding(target.value_type, &mut rendered);
        emit_store(
            scalar.expect("scalar target was checked above"),
            &mut rendered,
            context,
        );
    } else {
        emit_jvm_semantic_single_rounding(target.value_type, &mut rendered);
        rendered.push_str(&format!("    {}\n", array_store_opcode(target_type)));
    }
    out.push_str(&rendered);
    true
}

fn emit_jvm_semantic_line_input(
    channel: &crate::semantic_ir::Expression,
    target: &crate::semantic_ir::Expression,
    context: &JvmContext,
    out: &mut String,
) -> bool {
    use crate::semantic_ir::{ExpressionKind, SemanticValueType};
    if channel.value_type == SemanticValueType::String
        || target.value_type != SemanticValueType::String
    {
        return false;
    }
    let mut rendered = String::new();
    let scalar = match &target.kind {
        ExpressionKind::Name(name) => {
            let Ok(variable) = context.variable(&BasicIdent::parse(name)) else {
                return false;
            };
            if variable.ty != JvmType::String {
                return false;
            }
            Some(variable)
        }
        ExpressionKind::Index { name, index } => {
            if emit_jvm_semantic_array_address(
                name,
                std::slice::from_ref(index.as_ref()),
                context,
                &mut rendered,
            ) != Ok(JvmType::String)
            {
                return false;
            }
            None
        }
        ExpressionKind::MultiIndex { name, indices } => {
            if emit_jvm_semantic_array_address(name, indices, context, &mut rendered)
                != Ok(JvmType::String)
            {
                return false;
            }
            None
        }
        _ => return false,
    };
    rendered.push_str(&format!(
        "    getstatic {}/bccFiles [Ljava/io/RandomAccessFile;\n",
        context.class_name
    ));
    let channel_type = match emit_jvm_semantic_numeric_expression(channel, &mut rendered, context) {
        Ok(ty) => ty,
        Err(_) => return false,
    };
    coerce_jvm_semantic_numeric(channel_type, NumericType::Int, &mut rendered);
    rendered.push_str(
        "    iconst_1\n    isub\n    aaload\n    invokevirtual java/io/RandomAccessFile/readLine ()Ljava/lang/String;\n",
    );
    let empty = next_condition_label(context);
    let loaded = next_condition_label(context);
    rendered.push_str(&format!(
        "    dup\n    ifnonnull {loaded}\n    pop\n    ldc \"\"\n    goto {empty}\n{loaded}:\n{empty}:\n"
    ));
    if let Some(variable) = scalar {
        emit_store(variable, &mut rendered, context);
    } else {
        rendered.push_str("    aastore\n");
    }
    out.push_str(&rendered);
    true
}

fn emit_jvm_semantic_locate(
    row: &crate::semantic_ir::Expression,
    column: &crate::semantic_ir::Expression,
    context: &JvmContext,
    out: &mut String,
) -> bool {
    let mut rendered = String::from(
        "    getstatic java/lang/System/out Ljava/io/PrintStream;\n    new java/lang/StringBuilder\n    dup\n    invokespecial java/lang/StringBuilder/<init> ()V\n    ldc \"\u{1b}[\"\n    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;\n",
    );
    for (expression, separator) in [(row, ";"), (column, "H")] {
        let value_type =
            match emit_jvm_semantic_numeric_expression(expression, &mut rendered, context) {
                Ok(value_type) => value_type,
                Err(_) => return false,
            };
        coerce_jvm_semantic_numeric(value_type, NumericType::Int, &mut rendered);
        rendered.push_str(
            "    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;\n",
        );
        rendered.push_str(&format!(
            "    ldc \"{separator}\"\n    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;\n"
        ));
    }
    rendered.push_str("    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;\n    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V\n");
    out.push_str(&rendered);
    true
}

fn emit_jvm_semantic_color(
    foreground: &crate::semantic_ir::Expression,
    background: Option<&crate::semantic_ir::Expression>,
    context: &JvmContext,
    out: &mut String,
) -> bool {
    let foreground_constant = jvm_semantic_integer_constant(foreground);
    let background_constant = background.and_then(jvm_semantic_integer_constant);
    if let Some(foreground) =
        foreground_constant.filter(|_| background.is_none() || background_constant.is_some())
    {
        let foreground_code = ANSI_FG[(foreground as usize) & 15];
        let escape = if background.is_some() {
            let Some(background) = background_constant else {
                return false;
            };
            format!(
                "\u{1b}[{foreground_code};{}m",
                ANSI_BG[(background as usize) & 7]
            )
        } else {
            format!("\u{1b}[{foreground_code}m")
        };
        let _ = emit_terminal_escape(&escape, out);
        return true;
    }
    let mut rendered = String::new();
    rendered.push_str("    getstatic java/lang/System/out Ljava/io/PrintStream;\n    new java/lang/StringBuilder\n    dup\n    invokespecial java/lang/StringBuilder/<init> ()V\n    ldc \"\\u001b[\"\n    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;\n");
    if !emit_jvm_semantic_palette_value(foreground, "bccAnsiFg", &mut rendered, context) {
        return false;
    }
    if let Some(background) = background {
        rendered.push_str("    ldc \";\"\n    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;\n");
        if !emit_jvm_semantic_palette_value(background, "bccAnsiBg", &mut rendered, context) {
            return false;
        }
    }
    rendered.push_str("    ldc \"m\"\n    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;\n    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;\n    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V\n");
    out.push_str(&rendered);
    true
}

fn emit_jvm_semantic_palette_value(
    expression: &crate::semantic_ir::Expression,
    helper_name: &str,
    out: &mut String,
    context: &JvmContext,
) -> bool {
    let ty = match emit_jvm_semantic_numeric_expression(expression, out, context) {
        Ok(ty) => ty,
        Err(_) => return false,
    };
    coerce_jvm_semantic_numeric(ty, NumericType::Int, out);
    out.push_str(&format!(
        "    invokestatic {}/{helper_name} (I)I\n    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;\n",
        context.class_name
    ));
    true
}

fn emit_ansi_palette_helper(name: &str, values: &[i32], mask: i32) -> String {
    let mut out = format!(
        ".method private static {name} : (I)I\n    .limit stack 3\n    .limit locals 1\n\n    iload 0\n    bipush {mask}\n    iand\n"
    );
    for index in 0..values.len() - 1 {
        out.push_str(&format!(
            "    dup\n    bipush {index}\n    if_icmpeq L_{name}_{index}\n"
        ));
    }
    out.push_str("    pop\n");
    out.push_str(&format!(
        "    bipush {}\n    ireturn\n",
        values[values.len() - 1]
    ));
    for (index, value) in values.iter().enumerate().take(values.len() - 1) {
        out.push_str(&format!(
            "L_{name}_{index}:\n    pop\n    bipush {value}\n    ireturn\n"
        ));
    }
    out.push_str(".end method\n\n");
    out
}

fn jvm_semantic_integer_constant(expression: &crate::semantic_ir::Expression) -> Option<i64> {
    use crate::semantic_ir::ExpressionKind as Kind;
    match &expression.kind {
        Kind::Parenthesized(inner) => jvm_semantic_integer_constant(inner),
        Kind::Unary { operator, operand } if operator == "-" => {
            jvm_semantic_integer_constant(operand)?.checked_neg()
        }
        Kind::Literal(value) => {
            if let Some(digits) = value
                .strip_prefix("&H")
                .or_else(|| value.strip_prefix("&h"))
            {
                i64::from_str_radix(digits, 16).ok()
            } else if let Some(digits) = value
                .strip_prefix("&O")
                .or_else(|| value.strip_prefix("&o"))
            {
                i64::from_str_radix(digits, 8).ok()
            } else {
                value.parse().ok()
            }
        }
        Kind::Boolean(value) => Some(if *value { -1 } else { 0 }),
        _ => None,
    }
}

fn emit_jvm_semantic_compound_assignment(
    target_variable: Variable,
    target_type: NumericType,
    operator: crate::semantic_ir::AssignmentOperator,
    value: &crate::semantic_ir::Expression,
    single_precision: bool,
    context: &JvmContext,
    out: &mut String,
) -> bool {
    let mut right_code = String::new();
    let right_type = match emit_jvm_semantic_numeric_expression(value, &mut right_code, context) {
        Ok(value_type) => value_type,
        Err(_) => return false,
    };
    let promoted = if operator == crate::semantic_ir::AssignmentOperator::Divide {
        NumericType::Double
    } else {
        promote_numeric(target_type, right_type)
    };
    let mut rendered = String::new();
    emit_load(target_variable, &mut rendered, context);
    coerce_top(target_type, promoted, &mut rendered);
    rendered.push_str(&right_code);
    coerce_top(right_type, promoted, &mut rendered);
    if operator == crate::semantic_ir::AssignmentOperator::Divide {
        rendered.push_str("    ddiv\n");
        coerce_jvm_semantic_numeric(NumericType::Double, target_type, &mut rendered);
        if single_precision {
            rendered.push_str("    d2f\n    f2d\n");
        }
        emit_store(target_variable, &mut rendered, context);
        out.push_str(&rendered);
        return true;
    }
    let opcode = match (operator, promoted) {
        (crate::semantic_ir::AssignmentOperator::Add, NumericType::Int) => "iadd",
        (crate::semantic_ir::AssignmentOperator::Subtract, NumericType::Int) => "isub",
        (crate::semantic_ir::AssignmentOperator::Multiply, NumericType::Int) => "imul",
        (crate::semantic_ir::AssignmentOperator::Add, NumericType::Long) => "ladd",
        (crate::semantic_ir::AssignmentOperator::Subtract, NumericType::Long) => "lsub",
        (crate::semantic_ir::AssignmentOperator::Multiply, NumericType::Long) => "lmul",
        (crate::semantic_ir::AssignmentOperator::Add, NumericType::Double) => "dadd",
        (crate::semantic_ir::AssignmentOperator::Subtract, NumericType::Double) => "dsub",
        (crate::semantic_ir::AssignmentOperator::Multiply, NumericType::Double) => "dmul",
        _ => return false,
    };
    rendered.push_str(&format!("    {opcode}\n"));
    coerce_jvm_semantic_numeric(promoted, target_type, &mut rendered);
    if single_precision {
        rendered.push_str("    d2f\n    f2d\n");
    }
    emit_store(target_variable, &mut rendered, context);
    out.push_str(&rendered);
    true
}

fn jvm_semantic_numeric_type(
    value_type: crate::semantic_ir::SemanticValueType,
) -> Option<NumericType> {
    use crate::semantic_ir::SemanticValueType as Type;
    match value_type {
        Type::Integer | Type::Boolean => Some(NumericType::Int),
        Type::Long => Some(NumericType::Long),
        Type::Single | Type::Double => Some(NumericType::Double),
        Type::Unknown | Type::String => None,
    }
}

fn emit_jvm_semantic_single_rounding(
    value_type: crate::semantic_ir::SemanticValueType,
    out: &mut String,
) {
    if value_type == crate::semantic_ir::SemanticValueType::Single {
        // JVM storage uses `double` for both BASCAL floating types; round at
        // typed IR boundaries so SINGLE retains IEEE-754 binary32 semantics.
        out.push_str("    d2f\n    f2d\n");
    }
}

fn coerce_jvm_semantic_numeric(from: NumericType, to: NumericType, out: &mut String) {
    match (from, to) {
        (NumericType::Double, NumericType::Int) => {
            emit_round_away_from_zero(out);
            out.push_str("    l2i\n");
        }
        (NumericType::Double, NumericType::Long) => emit_round_away_from_zero(out),
        (NumericType::Long, NumericType::Int) => out.push_str("    l2i\n"),
        _ => coerce_top(from, to, out),
    }
}

fn emit_jvm_semantic_numeric_expression(
    expression: &crate::semantic_ir::Expression,
    out: &mut String,
    context: &JvmContext,
) -> Result<NumericType, String> {
    let mut numeric_type = emit_jvm_semantic_numeric_expression_inner(expression, out, context)?;
    if expression.value_type == crate::semantic_ir::SemanticValueType::Single {
        coerce_top(numeric_type, NumericType::Double, out);
        numeric_type = NumericType::Double;
    }
    emit_jvm_semantic_single_rounding(expression.value_type, out);
    Ok(numeric_type)
}

fn emit_jvm_semantic_numeric_expression_inner(
    expression: &crate::semantic_ir::Expression,
    out: &mut String,
    context: &JvmContext,
) -> Result<NumericType, String> {
    use crate::semantic_ir::{ExpressionKind as Kind, SemanticValueType as ValueType};
    match &expression.kind {
        Kind::Boolean(value) => {
            out.push_str(if *value {
                "    iconst_m1\n"
            } else {
                "    iconst_0\n"
            });
            Ok(NumericType::Int)
        }
        Kind::Literal(value) => {
            if let Some(digits) = value
                .strip_prefix("&H")
                .or_else(|| value.strip_prefix("&h"))
            {
                let integer = i64::from_str_radix(digits, 16)
                    .map_err(|_| "invalid hexadecimal literal".to_string())?;
                if let Ok(integer) = i32::try_from(integer) {
                    out.push_str(&format!("    ldc {integer}\n"));
                    return Ok(NumericType::Int);
                }
                out.push_str(&format!("    ldc2_w {integer}L\n"));
                return Ok(NumericType::Long);
            }
            if let Some(digits) = value
                .strip_prefix("&O")
                .or_else(|| value.strip_prefix("&o"))
            {
                let integer = i64::from_str_radix(digits, 8)
                    .map_err(|_| "invalid octal literal".to_string())?;
                if let Ok(integer) = i32::try_from(integer) {
                    out.push_str(&format!("    ldc {integer}\n"));
                    return Ok(NumericType::Int);
                }
                out.push_str(&format!("    ldc2_w {integer}L\n"));
                return Ok(NumericType::Long);
            }
            if let Ok(integer) = value.parse::<i64>() {
                if let Ok(integer) = i32::try_from(integer) {
                    out.push_str(&format!("    ldc {integer}\n"));
                    return Ok(NumericType::Int);
                }
                out.push_str(&format!("    ldc2_w {integer}L\n"));
                return Ok(NumericType::Long);
            }
            let ty = jvm_semantic_numeric_type(expression.value_type)
                .ok_or_else(|| "literal isn't numeric".to_string())?;
            if ty != NumericType::Double || expression.value_type == ValueType::Integer {
                return Err("invalid typed JVM numeric literal".to_string());
            }
            let number = value
                .parse::<f64>()
                .map_err(|_| "invalid floating-point literal".to_string())?;
            if !number.is_finite() {
                return Err(
                    "non-finite numeric literals are not supported by the JVM backend".to_string(),
                );
            }
            out.push_str(&format!("    ldc2_w {number:?}\n"));
            Ok(NumericType::Double)
        }
        Kind::Name(name) => {
            let variable = context.variable(&BasicIdent::parse(name))?;
            let JvmType::Numeric(ty) = variable.ty else {
                return Err(format!("`{name}` is a string, not a numeric scalar"));
            };
            if jvm_semantic_numeric_type(expression.value_type) != Some(ty) {
                return Err("semantic numeric type does not match resolved JVM storage".to_string());
            }
            emit_load(variable, out, context);
            Ok(ty)
        }
        Kind::Index { name, index } => emit_jvm_semantic_numeric_array_read(
            name,
            std::slice::from_ref(index.as_ref()),
            expression.value_type,
            context,
            out,
        ),
        Kind::MultiIndex { name, indices } => {
            emit_jvm_semantic_numeric_array_read(name, indices, expression.value_type, context, out)
        }
        Kind::Parenthesized(inner) => emit_jvm_semantic_numeric_expression(inner, out, context),
        Kind::Unary { operator, operand } if operator == "-" => {
            let ty = emit_jvm_semantic_numeric_expression(operand, out, context)?;
            out.push_str(match ty {
                NumericType::Int => "    ineg\n",
                NumericType::Long => "    lneg\n",
                NumericType::Double => "    dneg\n",
            });
            Ok(ty)
        }
        Kind::Unary { operator, operand } if operator.eq_ignore_ascii_case("not") => {
            let mut operand_code = String::new();
            let operand_type =
                emit_jvm_semantic_numeric_expression(operand, &mut operand_code, context)?;
            out.push_str(&operand_code);
            coerce_top(operand_type, NumericType::Double, out);
            emit_round_away_from_zero(out);
            out.push_str("    lconst_1\n    lneg\n    lxor\n");
            Ok(NumericType::Long)
        }
        Kind::Binary {
            left,
            operator,
            right,
        } => {
            if matches!(operator.as_str(), "&&" | "||") {
                let left_type = emit_jvm_semantic_numeric_expression(left, out, context)?;
                let shortcut = next_condition_label(context);
                let done = next_condition_label(context);
                emit_jvm_semantic_truth_branch(out, left_type, &shortcut, operator == "||");
                let right_type = emit_jvm_semantic_numeric_expression(right, out, context)?;
                emit_jvm_semantic_normalize_truth(out, right_type, context);
                out.push_str(&format!("    goto {done}\n{shortcut}:\n"));
                out.push_str(if operator == "&&" {
                    "    iconst_0\n"
                } else {
                    "    iconst_m1\n"
                });
                out.push_str(&format!("{done}:\n"));
                return Ok(NumericType::Int);
            }
            if matches!(operator.as_str(), "=" | "<>")
                && left.value_type == ValueType::String
                && right.value_type == ValueType::String
            {
                emit_jvm_semantic_string_expression(left, out, context)?;
                emit_jvm_semantic_string_expression(right, out, context)?;
                out.push_str("    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z\n");
                if operator == "=" {
                    out.push_str("    ineg\n");
                } else {
                    out.push_str("    iconst_1\n    ixor\n    ineg\n");
                }
                return Ok(NumericType::Int);
            }
            let mut left_code = String::new();
            let left_type = emit_jvm_semantic_numeric_expression(left, &mut left_code, context)?;
            let mut right_code = String::new();
            let right_type = emit_jvm_semantic_numeric_expression(right, &mut right_code, context)?;
            match operator.as_str() {
                "+" | "-" | "*" => {
                    let ty = promote_numeric(left_type, right_type);
                    out.push_str(&left_code);
                    coerce_top(left_type, ty, out);
                    out.push_str(&right_code);
                    coerce_top(right_type, ty, out);
                    let opcode = match (operator.as_str(), ty) {
                        ("+", NumericType::Int) => "iadd",
                        ("-", NumericType::Int) => "isub",
                        ("*", NumericType::Int) => "imul",
                        ("+", NumericType::Long) => "ladd",
                        ("-", NumericType::Long) => "lsub",
                        ("*", NumericType::Long) => "lmul",
                        ("+", NumericType::Double) => "dadd",
                        ("-", NumericType::Double) => "dsub",
                        ("*", NumericType::Double) => "dmul",
                        _ => unreachable!(),
                    };
                    out.push_str(&format!("    {opcode}\n"));
                    Ok(ty)
                }
                "/" => {
                    out.push_str(&left_code);
                    coerce_top(left_type, NumericType::Double, out);
                    out.push_str(&right_code);
                    coerce_top(right_type, NumericType::Double, out);
                    out.push_str("    ddiv\n");
                    Ok(NumericType::Double)
                }
                "^" => {
                    out.push_str(&left_code);
                    coerce_top(left_type, NumericType::Double, out);
                    out.push_str(&right_code);
                    coerce_top(right_type, NumericType::Double, out);
                    out.push_str("    invokestatic java/lang/Math/pow (DD)D\n");
                    Ok(NumericType::Double)
                }
                "\\" | "mod" | "MOD" => {
                    out.push_str(&left_code);
                    coerce_top(left_type, NumericType::Double, out);
                    emit_round_away_from_zero(out);
                    out.push_str(&right_code);
                    coerce_top(right_type, NumericType::Double, out);
                    emit_round_away_from_zero(out);
                    out.push_str(if operator == "\\" {
                        "    ldiv\n"
                    } else {
                        "    lrem\n"
                    });
                    Ok(NumericType::Long)
                }
                "and" | "AND" | "or" | "OR" | "xor" | "XOR" => {
                    out.push_str(&left_code);
                    coerce_top(left_type, NumericType::Double, out);
                    emit_round_away_from_zero(out);
                    out.push_str(&right_code);
                    coerce_top(right_type, NumericType::Double, out);
                    emit_round_away_from_zero(out);
                    out.push_str(match operator.to_ascii_lowercase().as_str() {
                        "and" => "    land\n",
                        "or" => "    lor\n",
                        _ => "    lxor\n",
                    });
                    Ok(NumericType::Long)
                }
                "=" | "<>" | "<" | "<=" | ">" | ">=" => {
                    let ty = promote_numeric(left_type, right_type);
                    out.push_str(&left_code);
                    coerce_top(left_type, ty, out);
                    out.push_str(&right_code);
                    coerce_top(right_type, ty, out);
                    if ty == NumericType::Double {
                        let (compare, branch) = match operator.as_str() {
                            "=" => ("dcmpl", "ifeq"),
                            "<>" => ("dcmpl", "ifne"),
                            "<" => ("dcmpg", "iflt"),
                            "<=" => ("dcmpg", "ifle"),
                            ">" => ("dcmpl", "ifgt"),
                            ">=" => ("dcmpl", "ifge"),
                            _ => unreachable!(),
                        };
                        let truth = next_condition_label(context);
                        let done = next_condition_label(context);
                        out.push_str(&format!(
                            "    {compare}\n    {branch} {truth}\n    iconst_0\n    goto {done}\n{truth}:\n    iconst_m1\n{done}:\n"
                        ));
                        return Ok(NumericType::Int);
                    }
                    out.push_str(match ty {
                        NumericType::Int => "    invokestatic java/lang/Integer/compare (II)I\n",
                        NumericType::Long => "    invokestatic java/lang/Long/compare (JJ)I\n",
                        NumericType::Double => unreachable!(),
                    });
                    match operator.as_str() {
                        "=" => out.push_str("    dup\n    ineg\n    ior\n    bipush 31\n    iushr\n    iconst_1\n    ixor\n    ineg\n"),
                        "<>" => out.push_str("    dup\n    ineg\n    ior\n    bipush 31\n    iushr\n    ineg\n"),
                        "<" => out.push_str("    bipush 31\n    ishr\n"),
                        ">" => out.push_str("    ineg\n    bipush 31\n    ishr\n"),
                        "<=" => out.push_str("    iconst_1\n    isub\n    bipush 31\n    ishr\n"),
                        ">=" => out.push_str("    ineg\n    iconst_1\n    isub\n    bipush 31\n    ishr\n"),
                        _ => unreachable!(),
                    }
                    Ok(NumericType::Int)
                }
                _ => Err("unsupported typed JVM semantic numeric operator".to_string()),
            }
        }
        Kind::Call { name, arguments } => {
            let result = jvm_semantic_numeric_type(expression.value_type)
                .ok_or_else(|| "typed JVM function result isn't numeric".to_string())?;
            emit_jvm_semantic_function_call(
                name,
                arguments,
                JvmType::Numeric(result),
                context,
                out,
            )?;
            Ok(result)
        }
        Kind::Member {
            base: Some(receiver),
            member,
            arguments: Some(arguments),
        } => {
            let result = jvm_semantic_numeric_type(expression.value_type)
                .ok_or_else(|| "typed JVM method result isn't numeric".to_string())?;
            emit_jvm_semantic_method_call(
                receiver,
                member,
                arguments,
                JvmType::Numeric(result),
                context,
                out,
            )?;
            Ok(result)
        }
        _ => Err("unsupported typed JVM semantic numeric expression".to_string()),
    }
}

fn emit_jvm_semantic_function_call(
    name: &str,
    arguments: &[crate::semantic_ir::Expression],
    expected: JvmType,
    context: &JvmContext,
    out: &mut String,
) -> Result<(), String> {
    let ident = BasicIdent::parse(name);
    let Some(signature) = context.function(&ident) else {
        return Err(format!("unknown typed JVM function `{name}`"));
    };
    emit_jvm_semantic_callable_call(
        &ident,
        &signature,
        arguments,
        None,
        Some(expected),
        context,
        out,
    )
}

fn emit_jvm_semantic_discarded_callable_call(
    name: &str,
    arguments: &[crate::semantic_ir::Expression],
    context: &JvmContext,
    out: &mut String,
) -> Result<(), String> {
    let ident = BasicIdent::parse(name);
    let Some(signature) = context.function(&ident) else {
        return Err(format!("unknown typed JVM callable `{name}`"));
    };
    if signature.returns_void {
        return emit_jvm_semantic_callable_call(
            &ident, &signature, arguments, None, None, context, out,
        );
    }
    let result = signature.result;
    emit_jvm_semantic_callable_call(
        &ident,
        &signature,
        arguments,
        None,
        Some(result),
        context,
        out,
    )?;
    out.push_str(match result {
        JvmType::String => "    pop\n",
        JvmType::Numeric(NumericType::Int) => "    pop\n",
        JvmType::Numeric(NumericType::Long | NumericType::Double) => "    pop2\n",
    });
    Ok(())
}

fn emit_jvm_semantic_discarded_expression(
    expression: &crate::semantic_ir::Expression,
    context: &JvmContext,
    out: &mut String,
) -> bool {
    use crate::semantic_ir::SemanticValueType;
    let mut rendered = String::new();
    let pop = match expression.value_type {
        SemanticValueType::String => {
            if emit_jvm_semantic_string_expression(expression, &mut rendered, context).is_err() {
                return false;
            }
            "    pop\n"
        }
        SemanticValueType::Integer | SemanticValueType::Boolean => {
            if emit_jvm_semantic_numeric_expression(expression, &mut rendered, context).is_err() {
                return false;
            }
            "    pop\n"
        }
        SemanticValueType::Long | SemanticValueType::Single | SemanticValueType::Double => {
            if emit_jvm_semantic_numeric_expression(expression, &mut rendered, context).is_err() {
                return false;
            }
            "    pop2\n"
        }
        SemanticValueType::Unknown => return false,
    };
    rendered.push_str(pop);
    out.push_str(&rendered);
    true
}

fn emit_jvm_semantic_method_call(
    receiver: &crate::semantic_ir::Expression,
    member: &str,
    arguments: &[crate::semantic_ir::Expression],
    expected: JvmType,
    context: &JvmContext,
    out: &mut String,
) -> Result<(), String> {
    use crate::semantic_ir::{ExpressionKind, SemanticValueType};
    let receiver_type = match receiver.value_type {
        SemanticValueType::String => JvmType::String,
        value_type => {
            JvmType::Numeric(jvm_semantic_numeric_type(value_type).ok_or_else(|| {
                "typed JVM scalar method receiver has no resolved type".to_string()
            })?)
        }
    };
    let method_name = BasicIdent::parse(member);
    let mut candidates = context
        .functions
        .iter()
        .filter(|(key, signature)| {
            let candidate = BasicIdent::parse(key);
            candidate.name.eq_ignore_ascii_case(&method_name.name)
                && signature.has_receiver
                && signature.params.first() == Some(&receiver_type)
                && signature.result == expected
                && signature.accepts_source_argument_count(arguments.len())
                && signature.params.len() + signature.array_params.len()
                    == signature.source_param_count + 1
                && arguments.iter().enumerate().all(|(position, argument)| {
                    if let Some(array) = signature
                        .array_params
                        .iter()
                        .find(|array| array.position == position)
                    {
                        let ExpressionKind::Name(name) = &argument.kind else {
                            return false;
                        };
                        context
                            .arrays
                            .get(&variable_key(&BasicIdent::parse(name)))
                            .is_some_and(|shape| {
                                shape.dimensions.len() == array.rank
                                    && shape.element == array.element
                            })
                    } else {
                        let preceding_arrays = signature
                            .array_params
                            .iter()
                            .filter(|array| array.position < position)
                            .count();
                        let scalar_position = position + 1 - preceding_arrays;
                        let Some(parameter) = signature.params.get(scalar_position) else {
                            return false;
                        };
                        let type_matches = match parameter {
                            JvmType::String => argument.value_type == SemanticValueType::String,
                            JvmType::Numeric(_) => {
                                jvm_semantic_numeric_type(argument.value_type).is_some()
                            }
                        };
                        type_matches
                            && (!signature.byref_scalar_positions.contains(&position)
                                || matches!(argument.kind, ExpressionKind::Name(_)))
                    }
                })
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| left.0.cmp(right.0));
    let Some((key, signature)) = candidates.first() else {
        return Err(format!("unsupported typed JVM scalar method `{member}`"));
    };
    let ident = BasicIdent::parse(key);
    emit_jvm_semantic_callable_call(
        &ident,
        signature,
        arguments,
        Some(receiver),
        Some(expected),
        context,
        out,
    )
}

fn emit_jvm_semantic_callable_call(
    ident: &BasicIdent,
    signature: &FunctionSig,
    arguments: &[crate::semantic_ir::Expression],
    receiver: Option<&crate::semantic_ir::Expression>,
    expected: Option<JvmType>,
    context: &JvmContext,
    out: &mut String,
) -> Result<(), String> {
    use crate::semantic_ir::SemanticValueType;
    let name = ident.as_basic();
    if !signature.accepts_source_argument_count(arguments.len()) {
        return Err(format!(
            "typed JVM function `{name}` doesn't accept {} source argument(s)",
            arguments.len()
        ));
    }
    let mut complete_arguments = arguments.to_vec();
    for default in signature
        .parameter_defaults
        .iter()
        .skip(complete_arguments.len())
    {
        let Some(default) = default else {
            return Err(format!(
                "typed JVM function `{name}` has a missing required argument"
            ));
        };
        complete_arguments.push(default.clone());
    }
    if signature.returns_void != expected.is_none()
        || signature.has_receiver != receiver.is_some()
        || expected.is_some_and(|expected| signature.result != expected)
        || signature.params.len() + signature.array_params.len()
            != signature.source_param_count + usize::from(receiver.is_some())
    {
        return Err(format!("unsupported typed JVM function call `{name}`"));
    }
    let mut writebacks = Vec::new();
    let mut next_scratch = context.byref_scratch_base;
    let mut scalar_parameters = signature
        .params
        .iter()
        .skip(usize::from(receiver.is_some()));
    if let Some(receiver) = receiver {
        let parameter = signature
            .params
            .first()
            .ok_or_else(|| "typed JVM method receiver metadata is incomplete".to_string())?;
        match parameter {
            JvmType::String if receiver.value_type == SemanticValueType::String => {
                emit_jvm_semantic_string_expression(receiver, out, context)?;
            }
            JvmType::Numeric(expected) => {
                let actual = emit_jvm_semantic_numeric_expression(receiver, out, context)?;
                coerce_jvm_semantic_numeric(actual, *expected, out);
            }
            _ => return Err("typed JVM method receiver type mismatch".to_string()),
        }
    }
    for (position, argument) in complete_arguments.iter().enumerate() {
        if let Some(array_parameter) = signature
            .array_params
            .iter()
            .find(|array| array.position == position)
        {
            let crate::semantic_ir::ExpressionKind::Name(array_name) = &argument.kind else {
                return Err(format!(
                    "typed array parameter {position} of `{name}` needs a plain array argument"
                ));
            };
            let array_ident = BasicIdent::parse(array_name);
            let shape = context
                .arrays
                .get(&variable_key(&array_ident))
                .ok_or_else(|| format!("unknown typed JVM array `{array_name}`"))?;
            if shape.dimensions.len() != array_parameter.rank
                || shape.element != array_parameter.element
            {
                return Err(format!(
                    "typed array argument `{array_name}` doesn't match `{name}`'s parameter"
                ));
            }
            context.emit_array_load(&array_ident, out);
            continue;
        }
        let parameter = scalar_parameters
            .next()
            .ok_or_else(|| "typed JVM scalar parameter metadata is incomplete".to_string())?;
        if signature.byref_scalar_positions.contains(&position) {
            let crate::semantic_ir::ExpressionKind::Name(target_name) = &argument.kind else {
                return Err(format!(
                    "typed byref parameter {position} of `{name}` needs a plain variable argument"
                ));
            };
            let ident = BasicIdent::parse(target_name);
            if context.field_vars.contains_key(&variable_key(&ident)) {
                return Err("typed byref field arguments aren't supported".to_string());
            }
            let variable = context.variable(&ident)?;
            if variable.ty != *parameter {
                return Err("typed JVM byref argument type mismatch".to_string());
            }
            let scratch = next_scratch;
            next_scratch += 1;
            out.push_str("    iconst_1\n");
            out.push_str(&format!(
                "    {}\n",
                new_scalar_array_instruction(*parameter)
            ));
            out.push_str("    dup\n    iconst_0\n");
            emit_load(variable, out, context);
            out.push_str(&format!("    {}\n", array_store_opcode(*parameter)));
            out.push_str(&format!("    astore {scratch}\n    aload {scratch}\n"));
            writebacks.push((scratch, variable));
        } else {
            match parameter {
                JvmType::String if argument.value_type == SemanticValueType::String => {
                    emit_jvm_semantic_string_expression(argument, out, context)?;
                }
                JvmType::Numeric(expected) => {
                    let actual = emit_jvm_semantic_numeric_expression(argument, out, context)?;
                    coerce_jvm_semantic_numeric(actual, *expected, out);
                }
                _ => return Err("typed JVM function argument type mismatch".to_string()),
            }
        }
    }
    let descriptor_params = signature.descriptor_params().join("");
    out.push_str(&format!(
        "    invokestatic {}/{} ({descriptor_params}){}\n",
        context.class_name,
        signature.source_name,
        if signature.returns_void {
            "V"
        } else {
            descriptor(signature.result)
        }
    ));
    emit_byref_call_writebacks(&writebacks, out, context);
    Ok(())
}

fn emit_jvm_semantic_truth_branch(out: &mut String, ty: NumericType, label: &str, when_true: bool) {
    match ty {
        NumericType::Int => out.push_str(&format!(
            "    if{} {label}\n",
            if when_true { "ne" } else { "eq" }
        )),
        NumericType::Long => {
            out.push_str("    lconst_0\n    lcmp\n");
            out.push_str(&format!(
                "    if{} {label}\n",
                if when_true { "ne" } else { "eq" }
            ));
        }
        NumericType::Double => {
            out.push_str("    dconst_0\n    dcmpg\n");
            out.push_str(&format!(
                "    if{} {label}\n",
                if when_true { "ne" } else { "eq" }
            ));
        }
    }
}

fn emit_jvm_semantic_normalize_truth(out: &mut String, ty: NumericType, context: &JvmContext) {
    let truthy = next_condition_label(context);
    let done = next_condition_label(context);
    emit_jvm_semantic_truth_branch(out, ty, &truthy, true);
    out.push_str("    iconst_0\n");
    out.push_str(&format!(
        "    goto {done}\n{truthy}:\n    iconst_m1\n{done}:\n"
    ));
}

fn emit_jvm_semantic_string_expression(
    expression: &crate::semantic_ir::Expression,
    out: &mut String,
    context: &JvmContext,
) -> Result<(), String> {
    use crate::semantic_ir::{ExpressionKind as Kind, SemanticValueType as ValueType};
    match &expression.kind {
        Kind::Literal(raw) if expression.value_type == ValueType::String => {
            let contents = raw
                .strip_prefix('"')
                .and_then(|value| value.strip_suffix('"'))
                .ok_or_else(|| "invalid semantic string literal".to_string())?
                .replace("\"\"", "\"");
            out.push_str(&format!("    ldc \"{}\"\n", escape_jvm_string(&contents)));
            Ok(())
        }
        Kind::Name(name) if expression.value_type == ValueType::String => {
            if name.eq_ignore_ascii_case("inkey$") {
                let empty = next_condition_label(context);
                let done = next_condition_label(context);
                out.push_str(&format!(
                    "    getstatic java/lang/System/in Ljava/io/InputStream;\n    invokevirtual java/io/InputStream/available ()I\n    ifle {empty}\n    getstatic java/lang/System/in Ljava/io/InputStream;\n    invokevirtual java/io/InputStream/read ()I\n    i2c\n    invokestatic java/lang/String/valueOf (C)Ljava/lang/String;\n    goto {done}\n{empty}:\n    ldc \"\"\n{done}:\n"
                ));
                return Ok(());
            }
            if name.eq_ignore_ascii_case("date$") {
                out.push_str("    invokestatic java/time/LocalDate/now ()Ljava/time/LocalDate;\n    ldc \"MM-dd-yyyy\"\n    invokestatic java/time/format/DateTimeFormatter/ofPattern (Ljava/lang/String;)Ljava/time/format/DateTimeFormatter;\n    invokevirtual java/time/LocalDate/format (Ljava/time/format/DateTimeFormatter;)Ljava/lang/String;\n");
                return Ok(());
            }
            let ident = BasicIdent::parse(name);
            if let Some(field) = context.field_vars.get(&variable_key(&ident)).copied() {
                emit_field_var_load(field, out, context);
                return Ok(());
            }
            let variable = context.variable(&ident)?;
            if variable.ty != JvmType::String {
                return Err(format!("`{name}` is not a string expression"));
            }
            emit_load(variable, out, context);
            Ok(())
        }
        Kind::Call { name, arguments } if expression.value_type == ValueType::String => {
            emit_jvm_semantic_function_call(name, arguments, JvmType::String, context, out)
        }
        Kind::Member {
            base: Some(receiver),
            member,
            arguments: Some(arguments),
        } if expression.value_type == ValueType::String => emit_jvm_semantic_method_call(
            receiver,
            member,
            arguments,
            JvmType::String,
            context,
            out,
        ),
        Kind::Index { name, index } if expression.value_type == ValueType::String => {
            emit_jvm_semantic_string_array_read(
                name,
                std::slice::from_ref(index.as_ref()),
                context,
                out,
            )
        }
        Kind::MultiIndex { name, indices } if expression.value_type == ValueType::String => {
            emit_jvm_semantic_string_array_read(name, indices, context, out)
        }
        Kind::Parenthesized(inner) if expression.value_type == ValueType::String => {
            emit_jvm_semantic_string_expression(inner, out, context)
        }
        Kind::Binary {
            left,
            operator,
            right,
        } if operator == "+" && expression.value_type == ValueType::String => {
            out.push_str("    new java/lang/StringBuilder\n    dup\n    invokespecial java/lang/StringBuilder/<init> ()V\n");
            emit_jvm_semantic_string_append(left, out, context)?;
            emit_jvm_semantic_string_append(right, out, context)?;
            out.push_str(
                "    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;\n",
            );
            Ok(())
        }
        _ => Err("unsupported typed JVM semantic string expression".to_string()),
    }
}

fn emit_jvm_semantic_string_append(
    expression: &crate::semantic_ir::Expression,
    out: &mut String,
    context: &JvmContext,
) -> Result<(), String> {
    use crate::semantic_ir::SemanticValueType as ValueType;
    match expression.value_type {
        ValueType::String => {
            emit_jvm_semantic_string_expression(expression, out, context)?;
            out.push_str("    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;\n");
        }
        value_type => {
            let ty = jvm_semantic_numeric_type(value_type)
                .ok_or_else(|| "unsupported typed JVM string concatenation operand".to_string())?;
            let actual = emit_jvm_semantic_numeric_expression(expression, out, context)?;
            coerce_jvm_semantic_numeric(actual, ty, out);
            let (suffix, descriptor) = match ty {
                NumericType::Int => (
                    "(I)Ljava/lang/StringBuilder;",
                    "invokevirtual java/lang/StringBuilder/append",
                ),
                NumericType::Long => (
                    "(J)Ljava/lang/StringBuilder;",
                    "invokevirtual java/lang/StringBuilder/append",
                ),
                NumericType::Double => (
                    "(D)Ljava/lang/StringBuilder;",
                    "invokevirtual java/lang/StringBuilder/append",
                ),
            };
            out.push_str(&format!("    {descriptor} {suffix}\n"));
        }
    }
    Ok(())
}

fn semantic_statements_contain_mid_assign(
    statements: &[crate::semantic_ir::SemanticStatement],
) -> bool {
    use crate::semantic_ir::SemanticStatementKind as Kind;
    statements.iter().any(|statement| match &statement.kind {
        Kind::MidAssign { .. } => true,
        Kind::Line(items) => semantic_statements_contain_mid_assign(items),
        Kind::If {
            then_body,
            else_body,
            ..
        } => {
            semantic_statements_contain_mid_assign(then_body)
                || semantic_statements_contain_mid_assign(else_body)
        }
        Kind::While { body, .. } | Kind::For { body, .. } | Kind::Do { body, .. } => {
            semantic_statements_contain_mid_assign(body)
        }
        Kind::SelectCase {
            cases, else_body, ..
        } => {
            cases
                .iter()
                .any(|case| semantic_statements_contain_mid_assign(&case.body))
                || semantic_statements_contain_mid_assign(else_body)
        }
        Kind::Try {
            body,
            catch,
            finally_body,
        } => {
            semantic_statements_contain_mid_assign(body)
                || catch
                    .as_ref()
                    .is_some_and(|catch| semantic_statements_contain_mid_assign(&catch.body))
                || semantic_statements_contain_mid_assign(finally_body)
        }
        _ => false,
    })
}

fn jvm_semantic_statements_by_source<'a>(
    module: &'a crate::semantic_ir::SemanticModule,
    ast_statements: &[Stmt],
) -> Option<Vec<Option<&'a crate::semantic_ir::SemanticStatement>>> {
    use crate::semantic_ir::SemanticStatementKind as Kind;
    if module.statement_sources.len() != module.statements.len() {
        return None;
    }
    let mut aligned = vec![None; ast_statements.len()];
    let mut previous = None;
    for (root, source_index) in module.statements.iter().zip(&module.statement_sources) {
        let source = module.sources.get(*source_index)?;
        let children: Vec<_> = if let Kind::Line(children) = &root.kind {
            children.iter().collect()
        } else {
            vec![root]
        };
        for semantic in children {
            let position = source.source_position_at(semantic.span.start)?;
            let candidates = ast_statements
                .iter()
                .enumerate()
                .filter(|(_, statement)| {
                    statement.pos.filename == position.filename
                        && statement.pos.line == position.line
                        && statement.pos.column == position.column
                        && !matches!(statement.kind, Statement::BlankLine)
                })
                .map(|(index, _)| index)
                .collect::<Vec<_>>();
            let [index] = candidates.as_slice() else {
                return None;
            };
            if previous.is_some_and(|previous| previous >= *index) || aligned[*index].is_some() {
                return None;
            }
            aligned[*index] = Some(semantic);
            previous = Some(*index);
        }
    }
    Some(aligned)
}

fn jvm_semantic_source_position_in_roots(
    module: &crate::semantic_ir::SemanticModule,
    roots: &[crate::semantic_ir::SemanticStatement],
    source_index: usize,
    semantic: &crate::semantic_ir::SemanticStatement,
) -> Option<crate::diagnostics::SourcePos> {
    use crate::semantic_ir::SemanticStatementKind as Kind;
    let belongs_to_roots = roots.iter().any(|root| {
        std::ptr::eq(root, semantic)
            || matches!(
                &root.kind,
                Kind::Line(children)
                    if children.iter().any(|child| std::ptr::eq(child, semantic))
            )
    });
    if !belongs_to_roots {
        return None;
    }
    module
        .sources
        .get(source_index)?
        .source_position_at(semantic.span.start)
}

fn jvm_semantic_top_level_source_position(
    module: &crate::semantic_ir::SemanticModule,
    semantic: &crate::semantic_ir::SemanticStatement,
) -> Option<crate::diagnostics::SourcePos> {
    for (root, source_index) in module.statements.iter().zip(&module.statement_sources) {
        let belongs_to_root = std::ptr::eq(root, semantic)
            || matches!(
                &root.kind,
                crate::semantic_ir::SemanticStatementKind::Line(children)
                    if children.iter().any(|child| std::ptr::eq(child, semantic))
            );
        if belongs_to_root {
            return jvm_semantic_source_position_in_roots(
                module,
                std::slice::from_ref(root),
                *source_index,
                semantic,
            );
        }
    }
    None
}

fn jvm_semantic_callable_source_position(
    module: &crate::semantic_ir::SemanticModule,
    function: &FunctionDef,
    semantic: &crate::semantic_ir::SemanticStatement,
) -> Option<crate::diagnostics::SourcePos> {
    let callable = semantic_callable_for_function(module, function)?;
    jvm_semantic_source_position_in_roots(
        module,
        &callable.body,
        callable.source_index,
        semantic,
    )
}

fn jvm_semantic_callable_statements_by_source<'a>(
    module: &'a crate::semantic_ir::SemanticModule,
    function: &FunctionDef,
) -> Option<Vec<Option<&'a crate::semantic_ir::SemanticStatement>>> {
    use crate::semantic_ir::SemanticStatementKind as Kind;
    let callable = semantic_callable_for_function(module, function)?;
    let source = module.sources.get(callable.source_index)?;
    if callable.span.end > source.text.len() {
        return None;
    }
    let mut aligned = vec![None; function.body.len()];
    let mut previous = None;
    for root in &callable.body {
        let children: Vec<_> = if let Kind::Line(children) = &root.kind {
            children.iter().collect()
        } else {
            vec![root]
        };
        for semantic in children {
            let position = source.source_position_at(semantic.span.start)?;
            let candidates = function
                .body
                .iter()
                .enumerate()
                .filter(|(_, statement)| {
                    statement.pos.filename == position.filename
                        && statement.pos.line == position.line
                        && statement.pos.column == position.column
                        && !matches!(statement.kind, Statement::BlankLine)
                })
                .map(|(index, _)| index)
                .collect::<Vec<_>>();
            let [index] = candidates.as_slice() else {
                return None;
            };
            if previous.is_some_and(|previous| previous >= *index) || aligned[*index].is_some() {
                return None;
            }
            aligned[*index] = Some(semantic);
            previous = Some(*index);
        }
    }
    Some(aligned)
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
fn class_name_for(
    program: &Program,
    semantic_module: Option<&crate::semantic_ir::SemanticModule>,
) -> String {
    let name = if let Some(module) = semantic_module {
        match module.header.as_ref() {
            Some(crate::semantic_ir::ModuleHeader::Program { name, .. }) => Some(name.as_str()),
            _ => None,
        }
    } else {
        program.program_decl.as_ref().map(|decl| decl.name.as_str())
    };
    let Some(name) = name else {
        return "Program".to_string();
    };
    let mut chars = name.chars();
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
    if !context.data_items.is_empty() {
        fields.push_str(
            ".field public static bccData [Ljava/lang/String;\n\
             .field public static bccDataPtr I\n",
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

fn emit_data_initializers(context: &JvmContext, out: &mut String) {
    if context.data_items.is_empty() {
        return;
    }
    out.push_str(&format!(
        "    ldc {}\n    anewarray java/lang/String\n    putstatic {}/bccData [Ljava/lang/String;\n",
        context.data_items.len(), context.class_name
    ));
    for (index, item) in context.data_items.iter().enumerate() {
        out.push_str(&format!(
            "    getstatic {}/bccData [Ljava/lang/String;\n    ldc {index}\n    ldc \"{}\"\n    aastore\n",
            context.class_name,
            escape_jvm_string(item)
        ));
    }
    out.push_str(&format!(
        "    iconst_0\n    putstatic {}/bccDataPtr I\n",
        context.class_name
    ));
}

fn collect_jvm_semantic_data_items(
    module: &crate::semantic_ir::SemanticModule,
) -> Result<(Vec<String>, HashMap<String, usize>), String> {
    use crate::semantic_ir::{ExpressionKind, SemanticStatementKind as Kind, SemanticValueType};
    fn literal(expression: &crate::semantic_ir::Expression) -> Result<String, String> {
        match (&expression.kind, expression.value_type) {
            (ExpressionKind::Literal(raw), SemanticValueType::String) => raw
                .strip_prefix('"')
                .and_then(|value| value.strip_suffix('"'))
                .map(|value| value.replace("\"\"", "\""))
                .ok_or_else(|| "invalid typed JVM DATA string literal".to_string()),
            (ExpressionKind::Literal(raw), SemanticValueType::Integer | SemanticValueType::Long)
                if raw.parse::<i64>().is_ok() =>
            {
                Ok(raw.clone())
            }
            (ExpressionKind::Literal(raw), SemanticValueType::Single | SemanticValueType::Double)
                if raw.parse::<f64>().is_ok() =>
            {
                Ok(raw.clone())
            }
            (ExpressionKind::Unary { operator, operand }, ty)
                if operator == "-"
                    && matches!(
                        (&operand.kind, ty),
                        (ExpressionKind::Literal(value), SemanticValueType::Integer | SemanticValueType::Long)
                            if value.parse::<i64>().is_ok()
                    )
                    || operator == "-"
                        && matches!(
                            (&operand.kind, ty),
                            (ExpressionKind::Literal(value), SemanticValueType::Single | SemanticValueType::Double)
                                if value.parse::<f64>().is_ok()
                    ) =>
            {
                let ExpressionKind::Literal(value) = &operand.kind else {
                    unreachable!()
                };
                Ok(format!("-{value}"))
            }
            _ => Err(
                "DATA items aren't supported by the minimal JVM backend yet -- only typed literal numbers and strings are"
                    .to_string(),
            ),
        }
    }
    fn visit(
        statements: &[crate::semantic_ir::SemanticStatement],
        items: &mut Vec<String>,
        labels: &mut HashMap<String, usize>,
    ) -> Result<(), String> {
        for statement in statements {
            match &statement.kind {
                Kind::Data(values) => {
                    for value in values {
                        items.push(literal(value)?);
                    }
                }
                Kind::Label(name) => {
                    labels.insert(name.name.to_ascii_lowercase(), items.len());
                }
                Kind::Line(body)
                | Kind::While { body, .. }
                | Kind::For { body, .. }
                | Kind::Do { body, .. } => visit(body, items, labels)?,
                Kind::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    visit(then_body, items, labels)?;
                    visit(else_body, items, labels)?;
                }
                Kind::SelectCase {
                    cases, else_body, ..
                } => {
                    for case in cases {
                        visit(&case.body, items, labels)?;
                    }
                    visit(else_body, items, labels)?;
                }
                Kind::Try {
                    body,
                    catch,
                    finally_body,
                } => {
                    visit(body, items, labels)?;
                    if let Some(catch) = catch {
                        visit(&catch.body, items, labels)?;
                    }
                    visit(finally_body, items, labels)?;
                }
                _ => {}
            }
        }
        Ok(())
    }
    let mut items = Vec::new();
    let mut labels = HashMap::new();
    visit(&module.statements, &mut items, &mut labels)?;
    for callable in &module.callables {
        visit(&callable.body, &mut items, &mut labels)?;
    }
    Ok((items, labels))
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
    format!(
        ".method private static bccCopyArray : (Ljava/lang/Object;II)Ljava/lang/Object;\n    .limit stack 5\n    .limit locals 5\n\n    iload 1\n    iconst_1\n    if_icmpne L_copy_nested\n    iload 2\n    tableswitch 0\n        L_copy_int\n        L_copy_long\n        L_copy_double\n        L_copy_object\n    default : L_copy_invalid\nL_copy_int:\n    .stack full\n    locals Object java/lang/Object Integer Integer\n    stack\n    .end stack\n    aload 0\n    checkcast [I\n    invokevirtual [I/clone ()Ljava/lang/Object;\n    areturn\nL_copy_long:\n    .stack same\n    aload 0\n    checkcast [J\n    invokevirtual [J/clone ()Ljava/lang/Object;\n    areturn\nL_copy_double:\n    .stack same\n    aload 0\n    checkcast [D\n    invokevirtual [D/clone ()Ljava/lang/Object;\n    areturn\nL_copy_object:\n    .stack same\n    aload 0\n    checkcast [Ljava/lang/Object;\n    invokevirtual [Ljava/lang/Object;/clone ()Ljava/lang/Object;\n    areturn\nL_copy_nested:\n    .stack same\n    aload 0\n    checkcast [Ljava/lang/Object;\n    invokevirtual [Ljava/lang/Object;/clone ()Ljava/lang/Object;\n    checkcast [Ljava/lang/Object;\n    astore 3\n    iconst_0\n    istore 4\nL_copy_loop:\n    .stack full\n    locals Object java/lang/Object Integer Integer Object [Ljava/lang/Object; Integer\n    stack\n    .end stack\n    iload 4\n    aload 3\n    arraylength\n    if_icmpge L_copy_done\n    aload 3\n    iload 4\n    aload 3\n    iload 4\n    aaload\n    iload 1\n    iconst_1\n    isub\n    iload 2\n    invokestatic {class_name}/bccCopyArray (Ljava/lang/Object;II)Ljava/lang/Object;\n    aastore\n    iinc 4 1\n    goto L_copy_loop\nL_copy_done:\n    .stack same\n    aload 3\n    areturn\nL_copy_invalid:\n    .stack full\n    locals Object java/lang/Object Integer Integer\n    stack\n    .end stack\n    new java/lang/IllegalArgumentException\n    dup\n    invokespecial java/lang/IllegalArgumentException/<init> ()V\n    athrow\n.end method\n\n"
    )
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
    for (index, (key, shape)) in context.arrays.iter().enumerate() {
        if shape
            .dimensions
            .iter()
            .any(|dimension| matches!(dimension, ArrayDimension::TypedExpression(_)))
        {
            continue;
        }
        emit_jvm_array_allocation(key, shape, index, context, out)?;
    }
    Ok(())
}

fn emit_jvm_array_allocation(
    key: &str,
    shape: &ArrayShape,
    index: usize,
    context: &JvmContext,
    out: &mut String,
) -> Result<(), String> {
    for dimension in &shape.dimensions {
        emit_jvm_array_dimension(dimension, out, context)?;
        out.push_str("    iconst_1\n    iadd\n");
    }
    let desc = array_descriptor(shape);
    out.push_str(&format!("    multianewarray {desc} {}\n", shape.dimensions.len()));
    if let Some(slot) = context.array_slots.get(key) {
        out.push_str(&format!("    astore {slot}\n"));
    } else {
        out.push_str(&format!("    putstatic {}/a{} {desc}\n", context.class_name, index));
    }
    Ok(())
}

fn emit_jvm_semantic_dim(
    dimensions: &[crate::semantic_ir::DimDeclaration],
    context: &JvmContext,
    out: &mut String,
) -> bool {
    for declaration in dimensions.iter().filter(|item| item.array_axes > 0) {
        let parsed_name = BasicIdent::parse(&declaration.name);
        let requested_key = variable_key(&parsed_name);
        let key = if context.arrays.contains_key(&requested_key) {
            requested_key
        } else {
            let base_name = declaration
                .name
                .trim_end_matches(|character| matches!(character, '%' | '$' | '!' | '#' | '&'))
                .to_ascii_lowercase();
            let Some(key) = context.arrays.keys().find(|key| {
                key.trim_end_matches(|character| matches!(character, '%' | '$' | '!' | '#' | '&'))
                    == base_name
            }) else {
                return false;
            };
            key.clone()
        };
        let Some(shape) = context.arrays.get(&key) else {
            return false;
        };
        let local_slot = context.array_slots.get(&key).copied();
        if local_slot.is_some()
            && !shape
                .dimensions
                .iter()
                .any(|dimension| matches!(dimension, ArrayDimension::TypedExpression(_)))
        {
            continue;
        }
        if local_slot.is_none() && !context.initialize_static {
            continue;
        }
        if local_slot.is_none() && !shape
            .dimensions
            .iter()
            .any(|dimension| matches!(dimension, ArrayDimension::TypedExpression(_)))
        {
            continue;
        }
        let Some(index) = context.arrays.keys().position(|name| name == &key) else {
            return false;
        };
        if emit_jvm_array_allocation(&key, shape, index, context, out).is_err() {
            return false;
        }
    }
    true
}

fn emit_jvm_semantic_erase(
    _names: &[crate::semantic_ir::NamedReference],
    _context: &JvmContext,
    _out: &mut String,
) -> bool {
    // Compiled BASCAL arrays have fixed storage. ERASE does not release that
    // storage, so this declaration statement has no JVM runtime effect.
    true
}

fn emit_jvm_array_dimension(
    dimension: &ArrayDimension,
    out: &mut String,
    context: &JvmContext,
) -> Result<(), String> {
    match dimension {
        ArrayDimension::Typed(value) if i32::try_from(*value).is_ok() => {
            out.push_str(&format!("    ldc {value}\n"));
        }
        ArrayDimension::Typed(value) => {
            out.push_str(&format!("    ldc2_w {value}\n    l2i\n"));
        }
        ArrayDimension::TypedExpression(expression) => {
            let ty = emit_jvm_semantic_numeric_expression(expression, out, context)?;
            coerce_jvm_semantic_numeric(ty, NumericType::Int, out);
        }
        ArrayDimension::Legacy(expression) => {
            emit_numeric_expr_as(expression, NumericType::Int, out, context)?;
        }
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

fn collect_semantic_labels(
    statements: &[crate::semantic_ir::SemanticStatement],
    labels: &mut HashSet<String>,
) {
    use crate::semantic_ir::SemanticStatementKind as Kind;
    for statement in statements {
        let nested: Vec<&[crate::semantic_ir::SemanticStatement]> = match &statement.kind {
            Kind::Label(name) => {
                labels.insert(name.name.to_ascii_lowercase());
                Vec::new()
            }
            Kind::Line(body)
            | Kind::While { body, .. }
            | Kind::For { body, .. }
            | Kind::Do { body, .. } => vec![body],
            Kind::If {
                then_body,
                else_body,
                ..
            } => vec![then_body, else_body],
            Kind::SelectCase {
                cases, else_body, ..
            } => cases
                .iter()
                .map(|case| case.body.as_slice())
                .chain(std::iter::once(else_body.as_slice()))
                .collect(),
            Kind::Try {
                body,
                catch,
                finally_body,
            } => {
                let mut bodies = vec![body.as_slice(), finally_body.as_slice()];
                if let Some(catch) = catch {
                    bodies.push(catch.body.as_slice());
                }
                bodies
            }
            _ => Vec::new(),
        };
        for body in nested {
            collect_semantic_labels(body, labels);
        }
    }
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

fn collect_semantic_field_vars(
    module: &crate::semantic_ir::SemanticModule,
) -> Result<BTreeMap<String, JvmFieldVar>, String> {
    fn add_lowered_file(
        file: &crate::semantic_ir::LoweredRecordFile,
        out: &mut BTreeMap<String, JvmFieldVar>,
    ) {
        let mut offset = 0i64;
        for field in &file.fields {
            let width = i64::from(field.width);
            out.insert(
                field.buffer_name.to_ascii_lowercase(),
                JvmFieldVar {
                    channel: file.channel,
                    offset,
                    width,
                },
            );
            offset += width;
        }
    }

    fn visit(
        statements: &[crate::semantic_ir::SemanticStatement],
        owner: Option<&str>,
        lowered_files: &[crate::semantic_ir::LoweredRecordFile],
        out: &mut BTreeMap<String, JvmFieldVar>,
    ) -> Result<(), String> {
        use crate::semantic_ir::{ExpressionKind, SemanticStatementKind as Kind};
        for statement in statements {
            match &statement.kind {
                Kind::Field { channel, bindings } => {
                    let ExpressionKind::Literal(channel) = &channel.kind else {
                        return Err(
                            "FIELD's channel number must be a literal under --target jvm"
                                .to_string(),
                        );
                    };
                    let channel = channel.parse::<i64>().map_err(|_| {
                        "FIELD's channel number must be a literal under --target jvm".to_string()
                    })?;
                    let mut offset = 0i64;
                    for binding in bindings {
                        let ExpressionKind::Literal(width) = &binding.length.kind else {
                            return Err("FIELD's field widths must be literal under --target jvm"
                                .to_string());
                        };
                        let width = width.parse::<i64>().map_err(|_| {
                            "FIELD's field widths must be literal under --target jvm".to_string()
                        })?;
                        out.insert(
                            binding.name.to_ascii_lowercase(),
                            JvmFieldVar {
                                channel,
                                offset,
                                width,
                            },
                        );
                        offset += width;
                    }
                }
                Kind::FileDeclaration { name, .. } => {
                    for file in lowered_files.iter().filter(|file| {
                        file.name.eq_ignore_ascii_case(&name.name)
                            && match (file.owner.as_deref(), owner) {
                                (None, None) => true,
                                (Some(file_owner), Some(owner)) => {
                                    file_owner.eq_ignore_ascii_case(owner)
                                }
                                _ => false,
                            }
                    }) {
                        add_lowered_file(file, out);
                    }
                }
                Kind::Line(body)
                | Kind::While { body, .. }
                | Kind::For { body, .. }
                | Kind::Do { body, .. } => visit(body, owner, lowered_files, out)?,
                Kind::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    visit(then_body, owner, lowered_files, out)?;
                    visit(else_body, owner, lowered_files, out)?;
                }
                Kind::SelectCase {
                    cases, else_body, ..
                } => {
                    for case in cases {
                        visit(&case.body, owner, lowered_files, out)?;
                    }
                    visit(else_body, owner, lowered_files, out)?;
                }
                Kind::Try {
                    body,
                    catch,
                    finally_body,
                } => {
                    visit(body, owner, lowered_files, out)?;
                    if let Some(catch) = catch {
                        visit(&catch.body, owner, lowered_files, out)?;
                    }
                    visit(finally_body, owner, lowered_files, out)?;
                }
                _ => {}
            }
        }
        Ok(())
    }
    let mut fields = BTreeMap::new();
    visit(
        &module.statements,
        None,
        &module.lowered_record_files,
        &mut fields,
    )?;
    for callable in &module.callables {
        visit(
            &callable.body,
            Some(&callable.name),
            &module.lowered_record_files,
            &mut fields,
        )?;
    }
    Ok(fields)
}

fn collect_jvm_field_vars(
    program: &Program,
    semantic_module: Option<&crate::semantic_ir::SemanticModule>,
) -> Result<BTreeMap<String, JvmFieldVar>, String> {
    if let Some(module) = semantic_module {
        return collect_semantic_field_vars(module);
    }
    let mut fields = BTreeMap::new();
    collect_field_vars(&program.statements, &mut fields)?;
    for function in &program.functions {
        collect_field_vars(&function.body, &mut fields)?;
    }
    Ok(fields)
}

/// Add scalar declarations represented by the semantic IR, including output
/// variables passed to semantic `byref` scalar parameters. The legacy AST
/// collector remains available for callers without semantic IR.
fn collect_semantic_scalar_declarations(
    module: &crate::semantic_ir::SemanticModule,
    declarations: &mut BTreeMap<String, JvmType>,
) {
    fn declaration_type(value_type: crate::semantic_ir::SemanticValueType) -> Option<JvmType> {
        match value_type {
            crate::semantic_ir::SemanticValueType::String => Some(JvmType::String),
            crate::semantic_ir::SemanticValueType::Integer => {
                Some(JvmType::Numeric(NumericType::Int))
            }
            crate::semantic_ir::SemanticValueType::Long => {
                Some(JvmType::Numeric(NumericType::Long))
            }
            crate::semantic_ir::SemanticValueType::Single
            | crate::semantic_ir::SemanticValueType::Double => {
                Some(JvmType::Numeric(NumericType::Double))
            }
            crate::semantic_ir::SemanticValueType::Boolean => {
                Some(JvmType::Numeric(NumericType::Double))
            }
            crate::semantic_ir::SemanticValueType::Unknown => None,
        }
    }
    fn visit(
        statements: &[crate::semantic_ir::SemanticStatement],
        callables: &[crate::semantic_ir::CallableSignature],
        declarations: &mut BTreeMap<String, JvmType>,
    ) {
        use crate::semantic_ir::{ExpressionKind, SemanticStatementKind as Kind};
        fn register_target(
            expression: &crate::semantic_ir::Expression,
            declarations: &mut BTreeMap<String, JvmType>,
        ) {
            if expression.value_type != crate::semantic_ir::SemanticValueType::Unknown {
                if let ExpressionKind::Name(name) = &expression.kind {
                    let ident = BasicIdent::parse(name);
                    let ty = jvm_type_for_semantic_value(expression.value_type);
                    declarations.insert(variable_key(&ident), ty);
                }
            }
        }
        fn register_byref_call_targets(
            expression: &crate::semantic_ir::Expression,
            callables: &[crate::semantic_ir::CallableSignature],
            declarations: &mut BTreeMap<String, JvmType>,
        ) {
            use crate::semantic_ir::{ExpressionKind, Passing};
            match &expression.kind {
                ExpressionKind::Call { name, arguments } => {
                    if let Some(callable) = callables
                        .iter()
                        .find(|callable| callable.name.eq_ignore_ascii_case(name))
                    {
                        for (position, parameter) in callable.parameters.iter().enumerate() {
                            if parameter.array_axes != 0
                                || parameter.passing != Some(Passing::ByRef)
                            {
                                continue;
                            }
                            let Some(argument) = arguments.get(position) else {
                                continue;
                            };
                            let ExpressionKind::Name(argument_name) = &argument.kind else {
                                continue;
                            };
                            let ident = BasicIdent::parse(argument_name);
                            let ty = jvm_type_for_semantic_value(argument.value_type);
                            declarations.insert(variable_key(&ident), ty);
                        }
                    }
                    for argument in arguments {
                        register_byref_call_targets(argument, callables, declarations);
                    }
                }
                ExpressionKind::Parenthesized(inner)
                | ExpressionKind::Unary { operand: inner, .. } => {
                    register_byref_call_targets(inner, callables, declarations)
                }
                ExpressionKind::Binary { left, right, .. } => {
                    register_byref_call_targets(left, callables, declarations);
                    register_byref_call_targets(right, callables, declarations);
                }
                ExpressionKind::Index { index, .. } => {
                    register_byref_call_targets(index, callables, declarations)
                }
                ExpressionKind::MultiIndex { indices, .. } => {
                    for index in indices {
                        register_byref_call_targets(index, callables, declarations);
                    }
                }
                ExpressionKind::Member {
                    base,
                    member,
                    arguments,
                } => {
                    if let (Some(base), Some(arguments)) = (base, arguments) {
                        let receiver_suffix =
                            base.value_type.suffix().and_then(TypeSuffix::from_char);
                        if let Some(callable) = callables.iter().find(|callable| {
                            callable.name.eq_ignore_ascii_case(member)
                                && callable.receiver.as_deref().is_some_and(|receiver| {
                                    receiver_suffix == semantic_receiver_suffix(receiver)
                                })
                        }) {
                            for (position, parameter) in callable.parameters.iter().enumerate() {
                                if parameter.array_axes != 0
                                    || parameter.passing != Some(Passing::ByRef)
                                {
                                    continue;
                                }
                                let Some(argument) = arguments.get(position) else {
                                    continue;
                                };
                                let ExpressionKind::Name(argument_name) = &argument.kind else {
                                    continue;
                                };
                                let ident = BasicIdent::parse(argument_name);
                                let ty = jvm_type_for_semantic_value(argument.value_type);
                                declarations.insert(variable_key(&ident), ty);
                            }
                        }
                    }
                    if let Some(base) = base {
                        register_byref_call_targets(base, callables, declarations);
                    }
                    if let Some(arguments) = arguments {
                        for argument in arguments {
                            register_byref_call_targets(argument, callables, declarations);
                        }
                    }
                }
                ExpressionKind::RecordLiteral(fields)
                | ExpressionKind::PartialRecordLiteral(fields) => {
                    for field in fields {
                        register_byref_call_targets(&field.value, callables, declarations);
                    }
                }
                ExpressionKind::Name(_)
                | ExpressionKind::Literal(_)
                | ExpressionKind::Boolean(_) => {}
            }
        }
        for statement in statements {
            match &statement.kind {
                Kind::Dim(items) => {
                    for item in items {
                        if item.array_axes == 0 {
                            if let Some(ty) = declaration_type(item.element_type) {
                                declarations.insert(item.name.to_ascii_lowercase(), ty);
                            }
                        }
                    }
                }
                Kind::Const {
                    name,
                    value,
                    value_type,
                } => {
                    register_byref_call_targets(value, callables, declarations);
                    let ident = BasicIdent::parse(&name.name);
                    declarations.insert(
                        variable_key(&ident),
                        jvm_type_for_semantic_value(*value_type),
                    );
                }
                Kind::Assignment { target, value, .. } => {
                    register_target(target, declarations);
                    register_byref_call_targets(target, callables, declarations);
                    register_byref_call_targets(value, callables, declarations);
                }
                Kind::MidAssign {
                    target,
                    start,
                    length,
                    value,
                } => {
                    register_target(target, declarations);
                    register_byref_call_targets(target, callables, declarations);
                    register_byref_call_targets(start, callables, declarations);
                    if let Some(length) = length {
                        register_byref_call_targets(length, callables, declarations);
                    }
                    register_byref_call_targets(value, callables, declarations);
                }
                Kind::Input { source, targets } => {
                    if let crate::semantic_ir::InputSource::Channel(channel) = source {
                        register_byref_call_targets(channel, callables, declarations);
                    }
                    for target in targets {
                        register_target(target, declarations);
                        register_byref_call_targets(target, callables, declarations);
                    }
                }
                Kind::Read(targets) => {
                    for target in targets {
                        register_target(target, declarations);
                        register_byref_call_targets(target, callables, declarations);
                    }
                }
                Kind::LineInput { channel, target } => {
                    register_byref_call_targets(channel, callables, declarations);
                    register_target(target, declarations);
                    register_byref_call_targets(target, callables, declarations);
                }
                Kind::Expression(expression) => {
                    register_byref_call_targets(expression, callables, declarations);
                }
                Kind::Print {
                    destination,
                    tokens,
                } => {
                    use crate::semantic_ir::{PrintDestination, PrintToken};
                    match destination {
                        PrintDestination::Standard { .. } => {}
                        PrintDestination::Channel { channel, using, .. } => {
                            register_byref_call_targets(channel, callables, declarations);
                            if let Some(using) = using {
                                register_byref_call_targets(using, callables, declarations);
                            }
                        }
                        PrintDestination::Using { format, .. } => {
                            register_byref_call_targets(format, callables, declarations);
                        }
                    }
                    for token in tokens {
                        if let PrintToken::Expression(expression) = token {
                            register_byref_call_targets(expression, callables, declarations);
                        }
                    }
                }
                Kind::Lprint { using, tokens } => {
                    use crate::semantic_ir::PrintToken;
                    if let Some(using) = using {
                        register_byref_call_targets(using, callables, declarations);
                    }
                    for token in tokens {
                        if let PrintToken::Expression(expression) = token {
                            register_byref_call_targets(expression, callables, declarations);
                        }
                    }
                }
                Kind::For {
                    variable,
                    variable_type,
                    start,
                    bounds,
                    body,
                } => {
                    let ident = BasicIdent::parse(variable);
                    declarations
                        .entry(variable_key(&ident))
                        .or_insert_with(|| jvm_type_for_semantic_value(*variable_type));
                    register_byref_call_targets(start, callables, declarations);
                    match bounds {
                        crate::semantic_ir::ForBounds::To { limit, step } => {
                            register_byref_call_targets(limit, callables, declarations);
                            if let Some(step) = step {
                                register_byref_call_targets(step, callables, declarations);
                            }
                        }
                        crate::semantic_ir::ForBounds::Downto { limit, .. } => {
                            register_byref_call_targets(limit, callables, declarations);
                        }
                    }
                    visit(body, callables, declarations);
                }
                Kind::Line(body) => visit(body, callables, declarations),
                Kind::While { condition, body } => {
                    register_byref_call_targets(condition, callables, declarations);
                    visit(body, callables, declarations);
                }
                Kind::Do {
                    pre_condition,
                    post_condition,
                    body,
                } => {
                    for condition in pre_condition.iter().chain(post_condition.iter()) {
                        register_byref_call_targets(&condition.value, callables, declarations);
                    }
                    visit(body, callables, declarations);
                }
                Kind::If {
                    condition,
                    then_body,
                    else_body,
                    ..
                } => {
                    register_byref_call_targets(condition, callables, declarations);
                    visit(then_body, callables, declarations);
                    visit(else_body, callables, declarations);
                }
                Kind::SelectCase {
                    selector,
                    cases,
                    else_body,
                    ..
                } => {
                    register_byref_call_targets(selector, callables, declarations);
                    for case in cases {
                        for value in &case.values {
                            match value {
                                crate::semantic_ir::CaseValue::Comparison { value, .. } => {
                                    register_byref_call_targets(value, callables, declarations);
                                }
                                crate::semantic_ir::CaseValue::Value {
                                    first, range_end, ..
                                } => {
                                    register_byref_call_targets(first, callables, declarations);
                                    if let Some(end) = range_end {
                                        register_byref_call_targets(end, callables, declarations);
                                    }
                                }
                            }
                        }
                        visit(&case.body, callables, declarations);
                    }
                    visit(else_body, callables, declarations);
                }
                Kind::Try {
                    body,
                    catch,
                    finally_body,
                } => {
                    visit(body, callables, declarations);
                    if let Some(catch) = catch {
                        let mut error = BasicIdent::parse(&catch.error);
                        let mut line = BasicIdent::parse(&catch.line);
                        error.suffix = catch
                            .error_type
                            .suffix()
                            .and_then(TypeSuffix::from_char);
                        line.suffix = catch.line_type.suffix().and_then(TypeSuffix::from_char);
                        declarations.insert(
                            variable_key(&error),
                            jvm_type_for_semantic_value(catch.error_type),
                        );
                        declarations.insert(
                            variable_key(&line),
                            jvm_type_for_semantic_value(catch.line_type),
                        );
                        for filter in &catch.filters {
                            register_byref_call_targets(filter, callables, declarations);
                        }
                        if let Some(source) = &catch.source {
                            let mut source = BasicIdent::parse(source);
                            source.suffix = catch
                                .source_type
                                .and_then(crate::semantic_ir::SemanticValueType::suffix)
                                .and_then(TypeSuffix::from_char);
                            declarations.insert(
                                variable_key(&source),
                                catch
                                    .source_type
                                    .map(jvm_type_for_semantic_value)
                                    .unwrap_or(JvmType::String),
                            );
                        }
                        visit(&catch.body, callables, declarations);
                    }
                    visit(finally_body, callables, declarations);
                }
                Kind::Return(crate::semantic_ir::ReturnValue::Value(value))
                | Kind::Throw(crate::semantic_ir::ThrowValue::Value(value))
                | Kind::Error(value)
                | Kind::OptionBase(value)
                | Kind::Kill(value)
                | Kind::Close(value) => {
                    register_byref_call_targets(value, callables, declarations);
                }
                Kind::OnBranch { selector, .. } => {
                    register_byref_call_targets(selector, callables, declarations);
                }
                Kind::Write { channel, values } => {
                    register_byref_call_targets(channel, callables, declarations);
                    if let crate::semantic_ir::WriteValues::Values(values) = values {
                        for value in values {
                            register_byref_call_targets(value, callables, declarations);
                        }
                    }
                }
                Kind::Open {
                    path,
                    channel,
                    length,
                    ..
                } => {
                    register_byref_call_targets(path, callables, declarations);
                    register_byref_call_targets(channel, callables, declarations);
                    if let Some(length) = length {
                        register_byref_call_targets(length, callables, declarations);
                    }
                }
                Kind::Seek { channel, position } => {
                    register_byref_call_targets(channel, callables, declarations);
                    register_byref_call_targets(position, callables, declarations);
                }
                Kind::Rename {
                    source,
                    destination,
                } => {
                    register_byref_call_targets(source, callables, declarations);
                    register_byref_call_targets(destination, callables, declarations);
                }
                Kind::Data(values) => {
                    for value in values {
                        register_byref_call_targets(value, callables, declarations);
                    }
                }
                Kind::Swap { left, right } => {
                    register_byref_call_targets(left, callables, declarations);
                    register_byref_call_targets(right, callables, declarations);
                }
                Kind::Randomize(crate::semantic_ir::RandomizeSeed::Value(value)) => {
                    register_byref_call_targets(value, callables, declarations);
                }
                Kind::Poke { address, value } => {
                    register_byref_call_targets(address, callables, declarations);
                    register_byref_call_targets(value, callables, declarations);
                }
                Kind::Out { port, value } => {
                    register_byref_call_targets(port, callables, declarations);
                    register_byref_call_targets(value, callables, declarations);
                }
                Kind::Width { channel, value } => {
                    if let Some(channel) = channel {
                        register_byref_call_targets(channel, callables, declarations);
                    }
                    register_byref_call_targets(value, callables, declarations);
                }
                Kind::Get { channel, position } | Kind::Put { channel, position } => {
                    register_byref_call_targets(channel, callables, declarations);
                    if let Some(position) = position {
                        if let Some(value) = &position.position {
                            register_byref_call_targets(value, callables, declarations);
                        }
                        if let Some(value) = &position.record {
                            register_byref_call_targets(value, callables, declarations);
                        }
                    }
                }
                Kind::Lset { value, .. } | Kind::Rset { value, .. } => {
                    register_byref_call_targets(value, callables, declarations);
                }
                Kind::Locate { row, column } => {
                    register_byref_call_targets(row, callables, declarations);
                    register_byref_call_targets(column, callables, declarations);
                }
                Kind::Color {
                    foreground,
                    background,
                } => {
                    register_byref_call_targets(foreground, callables, declarations);
                    if let Some(background) = background {
                        register_byref_call_targets(background, callables, declarations);
                    }
                }
                _ => {}
            }
        }
    }
    visit(&module.statements, &module.callables, declarations);
    // General-purpose record members are represented by flattened scalar
    // bindings in the compatibility AST. Derive those bindings from the
    // semantic record declarations instead of scanning that AST.
    fn effective_fields<'a>(
        module: &'a crate::semantic_ir::SemanticModule,
        record: &'a crate::semantic_ir::Record,
        fields: &mut Vec<&'a crate::semantic_ir::RecordField>,
    ) {
        for combined in &record.combines {
            if let Some(source) = module
                .records
                .iter()
                .find(|candidate| candidate.name.eq_ignore_ascii_case(combined))
            {
                effective_fields(module, source, fields);
            }
        }
        fields.extend(record.fields.iter());
    }
    for (variable, record_type) in module.record_variable_types() {
        let Some(record) = module
            .records
            .iter()
            .find(|record| record.name.eq_ignore_ascii_case(&record_type))
        else {
            continue;
        };
        let mut fields = Vec::new();
        effective_fields(module, record, &mut fields);
        for field in fields {
            let (suffix, ty) = match &field.field_type {
                crate::semantic_ir::RecordFieldType::String { .. } => {
                    (TypeSuffix::String, JvmType::String)
                }
                crate::semantic_ir::RecordFieldType::Int16 { .. }
                | crate::semantic_ir::RecordFieldType::Int { .. } => {
                    (TypeSuffix::Integer, JvmType::Numeric(NumericType::Int))
                }
                crate::semantic_ir::RecordFieldType::Int32 { .. } => {
                    (TypeSuffix::Long, JvmType::Numeric(NumericType::Long))
                }
                crate::semantic_ir::RecordFieldType::Float32 { .. }
                | crate::semantic_ir::RecordFieldType::Float64 { .. } => {
                    (TypeSuffix::Single, JvmType::Numeric(NumericType::Double))
                }
                crate::semantic_ir::RecordFieldType::Record { .. } => continue,
            };
            let ident = BasicIdent {
                name: crate::codegen_basic::camel_join(&[&variable, &field.name]),
                suffix: Some(suffix),
            };
            declarations.insert(variable_key(&ident), ty);
        }
    }
    for file in &module.lowered_record_files {
        for local in &file.record_locals {
            let Some(suffix) = local.value_type.suffix() else {
                continue;
            };
            let ident = BasicIdent {
                name: local.name.clone(),
                suffix: TypeSuffix::from_char(suffix),
            };
            declarations
                .entry(variable_key(&ident))
                .or_insert_with(|| jvm_type_for_semantic_value(local.value_type));
        }
        for field in &file.fields {
            let ty = jvm_type_for_record_field(field.kind);
            declarations.insert(field.buffer_name.to_ascii_lowercase(), ty);
        }
    }
}

fn jvm_type_for_record_field(kind: crate::semantic_ir::LoweredRecordFieldKind) -> JvmType {
    match kind {
        crate::semantic_ir::LoweredRecordFieldKind::String { .. } => JvmType::String,
        crate::semantic_ir::LoweredRecordFieldKind::Int16 => JvmType::Numeric(NumericType::Int),
        crate::semantic_ir::LoweredRecordFieldKind::Int32 => JvmType::Numeric(NumericType::Long),
        // BASCAL's JVM backend represents both floating source types as
        // double values; Float32 file fields are widened when unpacked.
        crate::semantic_ir::LoweredRecordFieldKind::Float32
        | crate::semantic_ir::LoweredRecordFieldKind::Float64 => {
            JvmType::Numeric(NumericType::Double)
        }
    }
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

fn semantic_runtime_features(module: &crate::semantic_ir::SemanticModule) -> (bool, bool, bool) {
    fn expression_uses_name(expression: &crate::semantic_ir::Expression, name: &str) -> bool {
        use crate::semantic_ir::ExpressionKind;
        match &expression.kind {
            ExpressionKind::Name(value) => value
                .trim_end_matches(['$', '%', '&', '!', '#'])
                .eq_ignore_ascii_case(name),
            ExpressionKind::Call { arguments, .. } => {
                arguments.iter().any(|arg| expression_uses_name(arg, name))
            }
            ExpressionKind::Index { index, .. } => expression_uses_name(index, name),
            ExpressionKind::MultiIndex { indices, .. } => indices
                .iter()
                .any(|index| expression_uses_name(index, name)),
            ExpressionKind::Member {
                base, arguments, ..
            } => {
                base.as_deref()
                    .is_some_and(|base| expression_uses_name(base, name))
                    || arguments
                        .as_deref()
                        .is_some_and(|args| args.iter().any(|arg| expression_uses_name(arg, name)))
            }
            ExpressionKind::Parenthesized(inner) | ExpressionKind::Unary { operand: inner, .. } => {
                expression_uses_name(inner, name)
            }
            ExpressionKind::Binary { left, right, .. } => {
                expression_uses_name(left, name) || expression_uses_name(right, name)
            }
            ExpressionKind::RecordLiteral(fields)
            | ExpressionKind::PartialRecordLiteral(fields) => fields
                .iter()
                .any(|field| expression_uses_name(&field.value, name)),
            ExpressionKind::Literal(_) | ExpressionKind::Boolean(_) => false,
        }
    }
    fn visit(statements: &[crate::semantic_ir::SemanticStatement], flags: &mut (bool, bool, bool)) {
        use crate::semantic_ir::{OpenModeKind, SemanticStatementKind as Kind};
        for statement in statements {
            match &statement.kind {
                Kind::Input { source, targets } => {
                    if matches!(source, crate::semantic_ir::InputSource::Channel(_)) {
                        flags.0 = true;
                    }
                    flags.1 = true;
                    if matches!(source, crate::semantic_ir::InputSource::Channel(expression) if expression_uses_name(expression, "inkey"))
                        || targets
                            .iter()
                            .any(|target| expression_uses_name(target, "inkey"))
                    {
                        flags.2 = true;
                    }
                }
                Kind::Open { mode, .. } if mode.kind == OpenModeKind::Random => flags.0 = true,
                Kind::Field { bindings, .. } => {
                    flags.0 = true;
                    flags.2 |= bindings
                        .iter()
                        .any(|binding| expression_uses_name(&binding.length, "inkey"));
                }
                Kind::Get { channel, position } | Kind::Put { channel, position } => {
                    flags.0 = true;
                    flags.2 |= expression_uses_name(channel, "inkey")
                        || position.as_ref().is_some_and(|position| {
                            position
                                .position
                                .as_ref()
                                .is_some_and(|value| expression_uses_name(value, "inkey"))
                                || position
                                    .record
                                    .as_ref()
                                    .is_some_and(|value| expression_uses_name(value, "inkey"))
                        });
                }
                Kind::Assignment { target, value, .. } | Kind::MidAssign { target, value, .. } => {
                    flags.2 |= expression_uses_name(target, "inkey")
                        || expression_uses_name(value, "inkey");
                }
                Kind::Expression(expression) | Kind::Error(expression) => {
                    flags.2 |= expression_uses_name(expression, "inkey");
                }
                Kind::If {
                    condition,
                    then_body,
                    else_body,
                    ..
                } => {
                    flags.2 |= expression_uses_name(condition, "inkey");
                    visit(then_body, flags);
                    visit(else_body, flags);
                }
                Kind::While { condition, body } => {
                    flags.2 |= expression_uses_name(condition, "inkey");
                    visit(body, flags);
                }
                Kind::For { start, body, .. } => {
                    flags.2 |= expression_uses_name(start, "inkey");
                    visit(body, flags);
                }
                Kind::SelectCase {
                    selector,
                    cases,
                    else_body,
                } => {
                    flags.2 |= expression_uses_name(selector, "inkey");
                    for case in cases {
                        visit(&case.body, flags);
                    }
                    visit(else_body, flags);
                }
                Kind::Return(crate::semantic_ir::ReturnValue::Value(expression))
                | Kind::Throw(crate::semantic_ir::ThrowValue::Value(expression)) => {
                    flags.2 |= expression_uses_name(expression, "inkey");
                }
                Kind::Print { tokens, .. } | Kind::Lprint { tokens, .. } => {
                    flags.2 |= tokens.iter().any(|token| match token {
                        crate::semantic_ir::PrintToken::Expression(expression) => {
                            expression_uses_name(expression, "inkey")
                        }
                        _ => false,
                    });
                }
                Kind::Open {
                    path,
                    channel,
                    length,
                    ..
                } => {
                    flags.0 = true;
                    flags.2 |= expression_uses_name(path, "inkey")
                        || expression_uses_name(channel, "inkey")
                        || length
                            .as_ref()
                            .is_some_and(|length| expression_uses_name(length, "inkey"));
                }
                Kind::Write { channel, values } => {
                    flags.0 = true;
                    flags.2 |= expression_uses_name(channel, "inkey")
                        || matches!(values, crate::semantic_ir::WriteValues::Values(values) if values.iter().any(|value| expression_uses_name(value, "inkey")));
                }
                Kind::Seek { channel, position } => {
                    flags.0 = true;
                    flags.2 |= expression_uses_name(channel, "inkey")
                        || expression_uses_name(position, "inkey");
                }
                Kind::Locate { row, column } => {
                    flags.2 |=
                        expression_uses_name(row, "inkey") || expression_uses_name(column, "inkey");
                }
                Kind::Color {
                    foreground,
                    background,
                } => {
                    flags.2 |= expression_uses_name(foreground, "inkey")
                        || background
                            .as_ref()
                            .is_some_and(|value| expression_uses_name(value, "inkey"));
                }
                Kind::Width { channel, value } => {
                    flags.2 |= channel
                        .as_ref()
                        .is_some_and(|channel| expression_uses_name(channel, "inkey"))
                        || expression_uses_name(value, "inkey");
                }
                Kind::Rename {
                    source,
                    destination,
                } => {
                    flags.0 = true;
                    flags.2 |= expression_uses_name(source, "inkey")
                        || expression_uses_name(destination, "inkey");
                }
                Kind::Kill(expression) => {
                    flags.0 = true;
                    flags.2 |= expression_uses_name(expression, "inkey");
                }
                Kind::Close(expression) => {
                    flags.0 = true;
                    flags.2 |= expression_uses_name(expression, "inkey");
                }
                Kind::Data(values) | Kind::Read(values) => {
                    flags.2 |= values
                        .iter()
                        .any(|value| expression_uses_name(value, "inkey"));
                }
                Kind::Swap { left, right } => {
                    flags.2 |=
                        expression_uses_name(left, "inkey") || expression_uses_name(right, "inkey");
                }
                Kind::Poke { address, value }
                | Kind::Out {
                    port: address,
                    value,
                } => {
                    flags.2 |= expression_uses_name(address, "inkey")
                        || expression_uses_name(value, "inkey");
                }
                Kind::LineInput { channel, target } => {
                    flags.0 = true;
                    flags.2 |= expression_uses_name(channel, "inkey")
                        || expression_uses_name(target, "inkey");
                }
                Kind::FileDeclaration { path, .. } => {
                    flags.0 = true;
                    flags.2 |= expression_uses_name(path, "inkey");
                }
                Kind::Lset { value, .. } | Kind::Rset { value, .. } => {
                    flags.2 |= expression_uses_name(value, "inkey");
                }
                Kind::Do {
                    pre_condition,
                    post_condition,
                    body,
                } => {
                    flags.2 |= pre_condition
                        .iter()
                        .chain(post_condition.iter())
                        .any(|condition| expression_uses_name(&condition.value, "inkey"));
                    visit(body, flags);
                }
                Kind::Line(body) => visit(body, flags),
                Kind::Try {
                    body,
                    catch,
                    finally_body,
                } => {
                    visit(body, flags);
                    if let Some(catch) = catch {
                        visit(&catch.body, flags);
                    }
                    visit(finally_body, flags);
                }
                _ => {}
            }
        }
    }
    let mut flags = (false, false, false);
    visit(&module.statements, &mut flags);
    for callable in &module.callables {
        visit(&callable.body, &mut flags);
    }
    flags
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

fn collect_semantic_array_declarations(
    module: &crate::semantic_ir::SemanticModule,
    arrays: &mut BTreeMap<String, ArrayShape>,
) {
    fn element_type(value_type: crate::semantic_ir::SemanticValueType) -> Option<JvmType> {
        match value_type {
            crate::semantic_ir::SemanticValueType::String => Some(JvmType::String),
            crate::semantic_ir::SemanticValueType::Integer => {
                Some(JvmType::Numeric(NumericType::Int))
            }
            crate::semantic_ir::SemanticValueType::Long => {
                Some(JvmType::Numeric(NumericType::Long))
            }
            crate::semantic_ir::SemanticValueType::Single
            | crate::semantic_ir::SemanticValueType::Double
            | crate::semantic_ir::SemanticValueType::Boolean => {
                Some(JvmType::Numeric(NumericType::Double))
            }
            crate::semantic_ir::SemanticValueType::Unknown => None,
        }
    }
    fn visit(
        statements: &[crate::semantic_ir::SemanticStatement],
        module: &crate::semantic_ir::SemanticModule,
        arrays: &mut BTreeMap<String, ArrayShape>,
    ) {
        use crate::semantic_ir::{DimAxis, SemanticStatementKind as Kind};
        for statement in statements {
            match &statement.kind {
                Kind::Dim(items) => {
                    for item in items.iter().filter(|item| item.array_axes > 0) {
                        let Some(element) = element_type(item.element_type) else {
                            continue;
                        };
                        if item
                            .dimensions
                            .iter()
                            .any(|axis| matches!(axis, DimAxis::Inferred))
                        {
                            continue;
                        }
                        let dimensions = item
                            .dimensions
                            .iter()
                            .map(|axis| match axis {
                                DimAxis::Fixed(value) => ArrayDimension::Typed(
                                    crate::semantic_ir::parse_integer_value(value).unwrap_or(0),
                                ),
                                DimAxis::Inferred => {
                                    unreachable!("inferred dimensions are filtered above")
                                }
                                DimAxis::Expression(value) => module
                                    .evaluate_integer_expression(value)
                                    .map(ArrayDimension::Typed)
                                    .unwrap_or_else(|| {
                                        ArrayDimension::TypedExpression(Box::new(value.clone()))
                                    }),
                            })
                            .collect();
                        let ident = BasicIdent {
                            name: item.name.clone(),
                            suffix: None,
                        };
                        arrays.insert(
                            variable_key(&ident),
                            ArrayShape {
                                element,
                                dimensions,
                            },
                        );
                    }
                }
                Kind::Line(body)
                | Kind::While { body, .. }
                | Kind::For { body, .. }
                | Kind::Do { body, .. } => visit(body, module, arrays),
                Kind::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    visit(then_body, module, arrays);
                    visit(else_body, module, arrays);
                }
                Kind::SelectCase {
                    cases, else_body, ..
                } => {
                    for case in cases {
                        visit(&case.body, module, arrays);
                    }
                    visit(else_body, module, arrays);
                }
                Kind::Try {
                    body,
                    catch,
                    finally_body,
                } => {
                    visit(body, module, arrays);
                    if let Some(catch) = catch {
                        visit(&catch.body, module, arrays);
                    }
                    visit(finally_body, module, arrays);
                }
                _ => {}
            }
        }
    }
    visit(&module.statements, module, arrays);
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

fn emit_jvm_array_index_conversion(ty: NumericType, out: &mut String) {
    match ty {
        NumericType::Int => {}
        NumericType::Long => out.push_str("    l2i\n"),
        NumericType::Double => {
            emit_round_away_from_zero(out);
            out.push_str("    l2i\n");
        }
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

fn semantic_receiver_suffix(receiver: &str) -> Option<TypeSuffix> {
    match receiver.trim().to_ascii_lowercase().as_str() {
        "integer" => Some(TypeSuffix::Integer),
        "long" => Some(TypeSuffix::Long),
        "single" => Some(TypeSuffix::Single),
        "double" => Some(TypeSuffix::Double),
        "string" => Some(TypeSuffix::String),
        _ => None,
    }
}

fn semantic_callable_for_function<'a>(
    module: &'a crate::semantic_ir::SemanticModule,
    function: &FunctionDef,
) -> Option<&'a crate::semantic_ir::CallableSignature> {
    use crate::semantic_ir::CallableKind;
    let candidates = module
        .callables
        .iter()
        .filter(|callable| {
            callable
                .name
                .eq_ignore_ascii_case(&function.name.as_basic())
                && callable.receiver.is_some() == function.receiver.is_some()
                && match (function.receiver.is_some(), callable.kind) {
                    (
                        true,
                        CallableKind::Method
                        | CallableKind::FluentMethod
                        | CallableKind::InlineMethod,
                    )
                    | (false, CallableKind::Procedure | CallableKind::Function) => true,
                    _ => false,
                }
        })
        .collect::<Vec<_>>();
    candidates
        .iter()
        .copied()
        .find(|callable| {
            callable
                .receiver
                .as_deref()
                .and_then(semantic_receiver_suffix)
                == function.receiver
        })
        .or_else(|| (candidates.len() == 1).then(|| candidates[0]))
}

fn callable_key_for_function(
    function: &FunctionDef,
    semantic_module: Option<&crate::semantic_ir::SemanticModule>,
) -> String {
    let Some(callable) =
        semantic_module.and_then(|module| semantic_callable_for_function(module, function))
    else {
        return function_key(&function.name);
    };
    let mut ident = BasicIdent::parse(&callable.name);
    if let Some(suffix) = callable
        .result_type
        .as_deref()
        .and_then(|suffix| suffix.chars().next())
        .and_then(TypeSuffix::from_char)
    {
        ident.suffix = Some(suffix);
    }
    function_key(&ident)
}

fn function_table(
    functions: &[FunctionDef],
    semantic_module: Option<&crate::semantic_ir::SemanticModule>,
) -> HashMap<String, FunctionSig> {
    functions
        .iter()
        .map(|function| {
            let semantic_callable =
                semantic_module.and_then(|module| semantic_callable_for_function(module, function));
            let semantic_param = |position: usize| {
                semantic_callable.and_then(|callable| callable.parameters.get(position))
            };
            let semantic_param_is_array = |position: usize| match semantic_param(position) {
                Some(parameter) => parameter.array_axes > 0,
                None if semantic_callable.is_none() => function.params[position].axes.is_some(),
                None => unreachable!(
                    "parameter position comes from the selected typed callable signature"
                ),
            };
            let semantic_param_by_ref = |position: usize| match semantic_param(position) {
                Some(parameter) => {
                    matches!(parameter.passing, Some(crate::semantic_ir::Passing::ByRef))
                }
                None if semantic_callable.is_none() => {
                    function.params[position].mode == ParamMode::ByRef
                }
                None => unreachable!(
                    "parameter position comes from the selected typed callable signature"
                ),
            };
            let source_param_count = semantic_callable
                .map(|callable| callable.parameters.len())
                .unwrap_or(function.params.len());
            let semantic_param_ident = |position: usize| {
                let mut ident = match (semantic_param(position), semantic_callable) {
                    (Some(parameter), _) => BasicIdent::parse(&parameter.name),
                    (None, None) => function
                        .params
                        .get(position)
                        .map(|param| param.name.clone())
                        .expect("parameter position comes from the AST signature"),
                    (None, Some(_)) => unreachable!(
                        "parameter position comes from the selected typed callable signature"
                    ),
                };
                if let Some(suffix) = semantic_param(position)
                    .and_then(|parameter| parameter.value_type.suffix())
                    .and_then(TypeSuffix::from_char)
                {
                    ident.suffix = Some(suffix);
                }
                ident
            };
            let result_ident = semantic_callable
                .map(|callable| BasicIdent::parse(&callable.name))
                .unwrap_or_else(|| function.name.clone());
            let returns_void = semantic_callable
                .map(|callable| callable.kind == crate::semantic_ir::CallableKind::Procedure)
                .unwrap_or(function.is_procedure);
            let receiver_suffix = semantic_callable
                .map(|callable| {
                    callable
                        .receiver
                        .as_deref()
                        .and_then(semantic_receiver_suffix)
                })
                .unwrap_or(function.receiver);
            let receiver_ident = receiver_suffix.map(|suffix| BasicIdent {
                name: "self".to_string(),
                suffix: Some(suffix),
            });
            let receiver_parameter_type = if semantic_callable.is_some() {
                semantic_callable
                    .and_then(|callable| callable.receiver.as_deref())
                    .and_then(semantic_receiver_suffix)
                    .map(jvm_type_for_type_suffix)
            } else {
                receiver_ident.as_ref().map(type_for_ident)
            };
            let semantic_parameter_type = |position: usize| {
                semantic_param(position)
                    .map(|parameter| jvm_type_for_semantic_value(parameter.value_type))
                    .unwrap_or_else(|| type_for_ident(&semantic_param_ident(position)))
            };
            (
                function_key(&result_ident),
                FunctionSig {
                    source_name: result_ident.name.clone(),
                    parameter_names: (0..source_param_count).map(semantic_param_ident).collect(),
                    parameter_defaults: (0..source_param_count)
                        .map(|position| {
                            semantic_param(position)
                                .and_then(|parameter| parameter.default.clone())
                                .map(|mut default| {
                                    if let Some(module) = semantic_module {
                                        module.annotate_expression_types(&mut default);
                                    }
                                    default
                                })
                        })
                        .collect(),
                    receiver_ident: receiver_ident.clone(),
                    params: receiver_parameter_type
                        .into_iter()
                        .chain((0..source_param_count).filter_map(|position| {
                            let is_array = semantic_param_is_array(position);
                            (!is_array).then(|| semantic_parameter_type(position))
                        }))
                        .collect(),
                    array_params: (0..source_param_count)
                        .filter_map(|position| {
                            let rank = semantic_param(position)
                                .map(|parameter| {
                                    (parameter.array_axes > 0).then_some(parameter.array_axes)
                                })
                                .unwrap_or_else(|| {
                                    if semantic_callable.is_some() {
                                        None
                                    } else {
                                        function.params[position].axes.as_ref().map(Vec::len)
                                    }
                                });
                            rank.map(|rank| JvmArrayParam {
                                position,
                                element: semantic_parameter_type(position),
                                rank,
                                by_ref: semantic_param_by_ref(position),
                            })
                        })
                        .collect(),
                    byref_scalar_positions: (0..source_param_count)
                        .filter(|position| {
                            let is_array = semantic_param_is_array(*position);
                            let by_ref = semantic_param_by_ref(*position);
                            !is_array && by_ref
                        })
                        .collect(),
                    source_param_count,
                    has_receiver: receiver_suffix.is_some(),
                    result: semantic_callable
                        .map(|callable| {
                            jvm_type_for_semantic_suffix(callable.result_type.as_deref())
                        })
                        .unwrap_or_else(|| type_for_ident(&result_ident)),
                    returns_void,
                },
            )
        })
        .collect()
}

fn emit_function(function: &FunctionDef, parent: &JvmContext) -> Result<String, String> {
    let context = JvmContext::for_function(function, parent);
    let mut body = String::new();
    context.emit_initializers(&mut body);
    for (index, (key, shape)) in context.arrays.iter().enumerate() {
        if context
            .array_slots
            .get(key)
            .is_some_and(|slot| context.local_array_slots.contains(slot))
            && !shape
                .dimensions
                .iter()
                .any(|dimension| matches!(dimension, ArrayDimension::TypedExpression(_)))
        {
            emit_jvm_array_allocation(key, shape, index, &context, &mut body)?;
        }
    }
    let signature = parent
        .functions
        .get(&callable_key_for_function(
            function,
            parent.semantic_module.as_ref(),
        ))
        .expect("registered function");
    for array in &signature.array_params {
        if array.by_ref {
            continue;
        }
        let rank = array.rank;
        let parameter_name = &signature.parameter_names[array.position];
        if !(1..=8).contains(&rank) {
            return Err(format!(
                "JVM byval array parameter `{}` needs a rank between 1 and 8",
                parameter_name
            ));
        }
        let slot = context.array_slots[&variable_key(parameter_name)];
        let desc = format!("{}{}", "[".repeat(rank), descriptor(array.element));
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
        loop_continues: Vec::new(),
        return_type: (!signature.returns_void).then_some(signature.result),
        is_callable: true,
        labels: HashSet::new(),
        exception_handlers: Vec::new(),
    };
    let semantic_callable = parent
        .semantic_module
        .as_ref()
        .and_then(|module| semantic_callable_for_function(module, function));
    let mut emitted_typed_body = false;
    if let (Some(module), Some(callable)) = (parent.semantic_module.as_ref(), semantic_callable) {
        let source_filename = module
            .sources
            .get(callable.source_index)
            .map(|source| source.filename.clone())
            .unwrap_or_default();
        let mut semantic_state = JvmSemanticState {
            source_filename,
            next_label: emitter.next_label,
            exception_handlers: Vec::new(),
        };
        let mut typed_body = String::new();
        if emit_jvm_semantic_block(
            &callable.body,
            &context,
            &mut emitter.loop_exits,
            &mut emitter.loop_continues,
            &mut semantic_state,
            &mut typed_body,
        ) {
            body.push_str(&typed_body);
            emitter.next_label = semantic_state.next_label;
            emitter
                .exception_handlers
                .extend(semantic_state.exception_handlers);
            emitted_typed_body = true;
        }
    }
    if !emitted_typed_body {
        let semantic_statements = parent
            .semantic_module
            .as_ref()
            .and_then(|module| jvm_semantic_callable_statements_by_source(module, function));
        let semantic_labels = semantic_statements
            .as_ref()
            .map(|statements| {
                let mut labels = HashSet::new();
                for statement in statements.iter().flatten() {
                    collect_semantic_labels(std::slice::from_ref(statement), &mut labels);
                }
                labels
            })
            .unwrap_or_default();
        emitter.labels = function
            .body
            .iter()
            .enumerate()
            .filter(|(index, _)| {
                semantic_statements
                    .as_ref()
                    .and_then(|statements| statements.get(*index))
                    .is_none_or(Option::is_none)
            })
            .flat_map(|(_, statement)| collect_labels(std::slice::from_ref(statement)))
            .chain(semantic_labels.iter().cloned())
            .collect();
        for (index, statement) in function.body.iter().enumerate() {
            let semantic_statement = semantic_statements
                .as_ref()
                .and_then(|statements| statements[index]);
            let semantic_source_position = semantic_statement.and_then(|semantic| {
                parent.semantic_module.as_ref().and_then(|module| {
                    jvm_semantic_callable_source_position(module, function, semantic)
                })
            });
            let mut semantic_state = JvmSemanticState {
                source_filename: semantic_source_position
                    .map(|position| position.filename)
                    .unwrap_or_else(|| statement.pos.filename.clone()),
                next_label: emitter.next_label,
                exception_handlers: Vec::new(),
            };
            let handled_semantically = semantic_statement.is_some_and(|semantic| {
                    use crate::semantic_ir::SemanticStatementKind as Kind;
                    match &semantic.kind {
                        Kind::End => {
                            emit_inkey_restore(&context, &mut body);
                            body.push_str(
                                "    iconst_0\n    invokestatic java/lang/System/exit (I)V\n",
                            );
                        }
                        Kind::Stop | Kind::System => {
                            emit_inkey_restore(&context, &mut body);
                            body.push_str(
                                "    iconst_0\n    invokestatic java/lang/System/exit (I)V\n",
                            );
                        }
                        Kind::Cls => {
                            let _ = emit_terminal_escape("\u{1b}[2J\u{1b}[H", &mut body);
                        }
                        Kind::Beep => {
                            let _ = emit_terminal_escape("\u{7}", &mut body);
                        }
                        Kind::Assignment {
                            target,
                            operator,
                            value,
                        } => {
                            return emit_jvm_semantic_assignment(
                                target, *operator, value, &context, &mut body,
                            );
                        }
                        Kind::Expression(crate::semantic_ir::Expression {
                            kind: crate::semantic_ir::ExpressionKind::Call { name, arguments },
                            ..
                        }) => {
                            return emit_jvm_semantic_discarded_callable_call(
                                name, arguments, &context, &mut body,
                            )
                            .is_ok();
                        }
                        Kind::Expression(expression) => {
                            return emit_jvm_semantic_discarded_expression(
                                expression, &context, &mut body,
                            );
                        }
                        Kind::Global { .. } => {}
                        // DATA items are materialized in the class pool before
                        // callable statement dispatch; declarations emit no bytecode.
                        Kind::Data(_) => {}
                        Kind::Erase(names) => {
                            return emit_jvm_semantic_erase(names, &context, &mut body);
                        }
                        Kind::Dim(dimensions) => {
                            if !emit_jvm_semantic_dim(dimensions, &context, &mut body) {
                                return false;
                            }
                        }
                        Kind::MidAssign {
                            target,
                            start,
                            length,
                            value,
                        } => {
                            return emit_jvm_semantic_mid_assign(
                                target,
                                start,
                                length.as_ref(),
                                value,
                                &context,
                                &mut body,
                            );
                        }
                        Kind::Swap { left, right } => {
                            return emit_jvm_semantic_swap(left, right, &context, &mut body);
                        }
                        Kind::Error(code) => {
                            return emit_jvm_semantic_error(code, &context, &mut body);
                        }
                        Kind::Throw(crate::semantic_ir::ThrowValue::Value(code)) => {
                            return emit_jvm_semantic_error(code, &context, &mut body);
                        }
                        Kind::Open {
                            path,
                            mode,
                            channel,
                            length: Some(length),
                        } if mode.kind == crate::semantic_ir::OpenModeKind::Random => {
                            return emit_jvm_semantic_random_open(
                                path, channel, length, &context, &mut body,
                            );
                        }
                        Kind::Close(channel) => {
                            return emit_jvm_semantic_file_close(channel, &context, &mut body);
                        }
                        Kind::Kill(path) => {
                            return emit_jvm_semantic_kill(path, &context, &mut body);
                        }
                        Kind::Rename {
                            source,
                            destination,
                        } => {
                            return emit_jvm_semantic_rename(
                                source,
                                destination,
                                &context,
                                &mut body,
                            );
                        }
                        Kind::Seek { channel, position } => {
                            return emit_jvm_semantic_seek(channel, position, &context, &mut body);
                        }
                        Kind::Field { .. } => {}
                        Kind::Print {
                            destination,
                            tokens,
                        } => {
                            return emit_jvm_semantic_print(
                                destination,
                                tokens,
                                &context,
                                &mut body,
                            );
                        }
                        Kind::Lprint {
                            using: None,
                            tokens,
                        } => {
                            return emit_jvm_semantic_print_tokens(tokens, &context, &mut body);
                        }
                        Kind::Input { source, targets } => {
                            return emit_jvm_semantic_input(source, targets, &context, &mut body);
                        }
                        Kind::LineInput { channel, target } => {
                            return emit_jvm_semantic_line_input(
                                channel, target, &context, &mut body,
                            );
                        }
                        Kind::If {
                            condition,
                            then_body,
                            else_body,
                            ..
                        } => {
                            return emit_jvm_semantic_if(
                                condition,
                                then_body,
                                else_body,
                                &context,
                                &mut semantic_state,
                                &mut body,
                            );
                        }
                        Kind::While {
                            condition,
                            body: loop_body,
                        } => {
                            return emit_jvm_semantic_while(
                                condition,
                                loop_body,
                                &context,
                                &mut semantic_state,
                                &mut body,
                            );
                        }
                        Kind::For {
                            variable,
                            start,
                            bounds,
                            body: loop_body,
                            ..
                        } => {
                            return emit_jvm_semantic_for(
                                variable,
                                start,
                                bounds,
                                loop_body,
                                &context,
                                &mut semantic_state,
                                &mut body,
                            );
                        }
                        Kind::Do {
                            pre_condition,
                            post_condition,
                            body: loop_body,
                        } => {
                            return emit_jvm_semantic_do(
                                pre_condition.as_ref(),
                                post_condition.as_ref(),
                                loop_body,
                                &context,
                                &mut semantic_state,
                                &mut body,
                            );
                        }
                        Kind::SelectCase {
                            selector,
                            cases,
                            else_body,
                        } => {
                            return emit_jvm_semantic_select_case(
                                selector,
                                cases,
                                else_body,
                                &context,
                                &mut semantic_state,
                                &mut body,
                            );
                        }
                        Kind::Try {
                            body: try_body,
                            catch,
                            finally_body,
                        } => {
                            return emit_jvm_semantic_try(
                                try_body,
                                catch.as_ref(),
                                finally_body,
                                &context,
                                &mut emitter.loop_exits,
                                &mut emitter.loop_continues,
                                &mut semantic_state,
                                &mut body,
                            );
                        }
                        Kind::Return(value) => {
                            return emit_jvm_semantic_return(
                                value,
                                signature.result,
                                signature.returns_void,
                                &context,
                                &mut body,
                            );
                        }
                        Kind::Const { name, value, .. } => {
                            return emit_jvm_semantic_const(name, value, &context, &mut body);
                        }
                        Kind::Locate { row, column } => {
                            return emit_jvm_semantic_locate(row, column, &context, &mut body);
                        }
                        Kind::Color {
                            foreground,
                            background,
                        } => {
                            return emit_jvm_semantic_color(
                                foreground,
                                background.as_ref(),
                                &context,
                                &mut body,
                            );
                        }
                        Kind::Label(name) => {
                            body.push_str(&format!("{}:\n", jvm_label(&name.name)));
                        }
                        Kind::Goto(target)
                            if semantic_labels.contains(&target.name.to_ascii_lowercase()) =>
                        {
                            body.push_str(&format!("    goto {}\n", jvm_label(&target.name)));
                        }
                        _ => return false,
                    }
                    true
                });
            if handled_semantically {
                emitter.next_label = semantic_state.next_label;
                emitter
                    .exception_handlers
                    .extend(semantic_state.exception_handlers);
            }
            if !handled_semantically {
                emitter.emit_statement(statement, &mut body)?;
            }
        }
    }
    // Resolver guarantees a return on every reachable path. This fallback
    // keeps the JVM verifier satisfied if an unsupported analysis edge leaks through.
    if signature.returns_void {
        context.emit_byref_writebacks(&mut body);
        body.push_str("    return\n");
    } else {
        context.emit_byref_writebacks(&mut body);
        match signature.result {
            JvmType::String => body.push_str("    ldc \"\"\n    areturn\n"),
            JvmType::Numeric(NumericType::Int) => body.push_str("    iconst_0\n    ireturn\n"),
            JvmType::Numeric(NumericType::Long) => body.push_str("    lconst_0\n    lreturn\n"),
            JvmType::Numeric(NumericType::Double) => body.push_str("    dconst_0\n    dreturn\n"),
        }
    }
    let sig = parent
        .functions
        .get(&callable_key_for_function(
            function,
            parent.semantic_module.as_ref(),
        ))
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
    Ok(format!(
        ".method public static {} : ({args}){}\n    .limit stack 16\n    .limit locals {}\n\n{body}{handlers}.end method\n\n",
        signature.source_name,
        result,
        context.local_count()
    ))
}

struct JvmEmitter<'a> {
    context: &'a JvmContext,
    next_label: usize,
    loop_exits: Vec<String>,
    // The label a `continue` inside the current innermost loop should
    // `goto` -- for `while`, this is the same label as `loop_exits`'s own
    // top-of-loop condition check (re-checking the condition *is* a
    // while loop's own per-iteration bookkeeping); for `for`/`do`, it's a
    // dedicated label placed right after the body, before the increment/
    // post-condition-check bytecode that follows -- jumping straight to
    // the loop's own top label instead would skip that bytecode entirely.
    loop_continues: Vec<String>,
    return_type: Option<JvmType>,
    is_callable: bool,
    labels: HashSet<String>,
    exception_handlers: Vec<JvmExceptionHandler>,
}

#[derive(Clone)]
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
            Statement::Dim { .. } => Ok(()),
            Statement::Const { name, value } => {
                let variable = self.context.variable(name)?;
                match variable.ty {
                    JvmType::String => emit_string_expr(value, out, self.context)?,
                    JvmType::Numeric(ty) => emit_numeric_expr_as(value, ty, out, self.context)?,
                }
                emit_store(variable, out, self.context);
                Ok(())
            }
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
            Statement::Continue => {
                let label = self.loop_continues.last().ok_or_else(|| {
                    "`continue` is only supported inside a JVM-transpiled loop".to_string()
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
            Statement::End if self.is_callable => {
                emit_inkey_restore(self.context, out);
                out.push_str("    iconst_0\n    invokestatic java/lang/System/exit (I)V\n");
                Ok(())
            }
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
        // Re-checking the condition at `top_label` *is* this loop's own
        // per-iteration bookkeeping, so `continue` can reuse it directly --
        // no separate label needed, unlike `for`/`do` below.
        self.loop_continues.push(top_label.clone());
        for statement in body {
            self.emit_statement(statement, out)?;
        }
        self.loop_continues.pop();
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
                    return Err("the JVM backend currently requires a literal FOR STEP".to_string());
                }
            },
            _ => return Err("the JVM backend currently requires a literal FOR STEP".to_string()),
        };
        if step_value == 0 {
            return Err("FOR STEP 0 is not supported by the JVM backend".to_string());
        }
        let start_value = self.context.reserve_semantic_int_local();
        emit_numeric_expr_as(start, NumericType::Int, out, self.context)?;
        emit_store(start_value, out, self.context);
        let limit_value = self.context.reserve_semantic_int_local();
        emit_numeric_expr_as(end, NumericType::Int, out, self.context)?;
        emit_store(limit_value, out, self.context);
        emit_load(start_value, out, self.context);
        emit_store(variable, out, self.context);

        let id = self.next_label;
        self.next_label += 1;
        let top_label = format!("L_for_{id}_top");
        let end_label = format!("L_for_{id}_end");
        out.push_str(&format!("{top_label}:\n"));
        emit_load(variable, out, self.context);
        emit_load(limit_value, out, self.context);
        let branch = if step_value > 0 {
            "if_icmpgt"
        } else {
            "if_icmplt"
        };
        out.push_str(&format!("    {branch} {end_label}\n"));
        self.loop_exits.push(end_label.clone());
        // `top_label` re-checks the loop bound, not the increment -- a
        // `continue` jumping straight there would skip incrementing the
        // loop variable entirely (an infinite loop on the same value), so
        // it needs its own label placed right before the increment code
        // below instead.
        let continue_label = format!("L_for_{id}_continue");
        self.loop_continues.push(continue_label.clone());
        for statement in body {
            self.emit_statement(statement, out)?;
        }
        self.loop_continues.pop();
        self.loop_exits.pop();
        out.push_str(&format!("{continue_label}:\n"));
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
        // `top_label` re-checks the pre-condition (if any); a `continue`
        // jumping straight there would skip the post-condition check
        // below entirely, silently turning `do ... loop until done` into
        // an infinite loop on `continue` -- so it needs its own label,
        // placed right after the body instead.
        let continue_label = format!("L_do_{id}_continue");
        self.loop_continues.push(continue_label.clone());
        for statement in body {
            self.emit_statement(statement, out)?;
        }
        self.loop_continues.pop();
        self.loop_exits.pop();
        out.push_str(&format!("{continue_label}:\n"));
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
                                ));
                            }
                        };
                        out.push_str(&format!("    {branch} {}\n", case_labels[index]));
                    }
                    _ => {
                        return Err(
                            "this SELECT CASE pattern is not supported by the JVM backend"
                                .to_string(),
                        );
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
    dimensions: Vec<ArrayDimension>,
}

#[derive(Clone, Debug, PartialEq)]
enum ArrayDimension {
    Typed(i64),
    TypedExpression(Box<crate::semantic_ir::Expression>),
    Legacy(Expr),
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
    local_array_slots: Vec<usize>,
    array_aliases: BTreeMap<String, String>,
    constant_names: BTreeSet<String>,
    local_count: Cell<usize>,
    initializer_start: usize,
    functions: HashMap<String, FunctionSig>,
    semantic_module: Option<crate::semantic_ir::SemanticModule>,
    semantic_name_scopes: Option<crate::semantic_ir::SemanticNameScopes>,
    class_name: String,
    condition_label: Cell<usize>,
    semantic_return: Option<(JvmType, bool)>,
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
    /// Typed literal DATA pool shared by module and callable READ statements.
    data_items: Vec<String>,
    data_labels: HashMap<String, usize>,
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
    function_global_declarations: HashMap<(String, Option<TypeSuffix>), Vec<BasicIdent>>,
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
    /// Resolved callable identifier used by the JVM method declaration.
    source_name: String,
    /// Resolved parameter binding identifiers in source parameter order.
    parameter_names: Vec<BasicIdent>,
    /// Typed IR defaults in source parameter order. Defaults are emitted at
    /// typed call sites when callers omit trailing optional scalar arguments.
    parameter_defaults: Vec<Option<crate::semantic_ir::Expression>>,
    /// Resolved `self` binding identifier for scalar receiver methods.
    receiver_ident: Option<BasicIdent>,
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
    fn accepts_source_argument_count(&self, argument_count: usize) -> bool {
        if self.parameter_defaults.len() != self.source_param_count {
            return false;
        }
        let required = self
            .parameter_defaults
            .iter()
            .filter(|default| default.is_none())
            .count();
        (required..=self.source_param_count).contains(&argument_count)
    }

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
        function_global_declarations: HashMap<(String, Option<TypeSuffix>), Vec<BasicIdent>>,
        typed_array_declarations: Vec<crate::ast::TypedArrayDecl>,
        semantic_module: Option<&crate::semantic_ir::SemanticModule>,
        semantic_name_scopes: Option<crate::semantic_ir::SemanticNameScopes>,
    ) -> Result<Self, Vec<Diagnostic>> {
        let mut declarations = BTreeMap::new();
        let mut constant_names = BTreeSet::new();
        let mut arrays = BTreeMap::new();
        let (data_items, data_labels) = semantic_module
            .map(collect_jvm_semantic_data_items)
            .transpose()
            .map_err(|message| vec![unsupported(&message)])?
            .unwrap_or_default();
        if data_items.is_empty()
            && semantic_module.is_some_and(|module| {
                module.has_statement(|kind| {
                    matches!(kind, crate::semantic_ir::SemanticStatementKind::Read(_))
                })
            })
        {
            return Err(vec![unsupported(
                "`read` found but the program has no `data` items at all -- real BASIC's own \
                 \"Out of DATA\" error, caught here at compile time instead",
            )]);
        }
        let field_vars = collect_jvm_field_vars(program, semantic_module)
            .map_err(|message| vec![unsupported(&message)])?;
        if let Some(module) = semantic_module {
            let mut callable_names: BTreeSet<String> = crate::codegen_basic::BASIC_BUILTINS
                .iter()
                .map(|name| (*name).to_string())
                .collect();
            callable_names.extend(
                module
                    .callables
                    .iter()
                    .map(|callable| BasicIdent::parse(&callable.name).name.to_ascii_lowercase()),
            );
            let semantic_name_scopes = module.name_scopes();
            for name in semantic_name_scopes.global_names {
                let ident = BasicIdent::parse(&name);
                if callable_names.contains(&ident.name.to_ascii_lowercase())
                    || field_vars.contains_key(&variable_key(&ident))
                {
                    continue;
                }
                let ty = semantic_name_scopes
                    .global_types
                    .get(&name.to_ascii_lowercase())
                    .copied()
                    .map(jvm_type_for_semantic_value)
                    .unwrap_or_else(|| type_for_ident(&ident));
                declarations.insert(variable_key(&ident), ty);
            }
            collect_semantic_scalar_declarations(module, &mut declarations);
            let const_types = module.top_level_const_types();
            for name in module.top_level_const_names() {
                let mut ident = BasicIdent::parse(&name);
                let previous_key = variable_key(&ident);
                let value_type = const_types
                    .get(&name)
                    .copied()
                    .unwrap_or(crate::semantic_ir::SemanticValueType::Integer);
                if let Some(suffix) = value_type
                    .suffix()
                    .and_then(TypeSuffix::from_char)
                {
                    ident.suffix = Some(suffix);
                    let typed_key = variable_key(&ident);
                    if previous_key != typed_key {
                        declarations.remove(&previous_key);
                    }
                }
                let ty = jvm_type_for_semantic_value(value_type);
                declarations.insert(variable_key(&ident), ty);
                constant_names.insert(variable_key(&ident));
            }
        } else {
            collect_scalar_declarations(
                &program.statements,
                &mut declarations,
                &mut constant_names,
                &functions,
            );
            // Callable GLOBAL declarations refer to module storage even if
            // their first assignment appears only inside the callable body.
            // Collect the body's scalar declaration types, then promote the
            // declared globals into the module slot table; `for_function`
            // will bind each callable reference back to these static slots.
            for function in &program.functions {
                let Some(global_names) = function_global_declarations.get(&(
                    function.name.name.to_ascii_lowercase(),
                    function.name.suffix,
                )) else {
                    continue;
                };
                if global_names.is_empty() {
                    continue;
                }
                let mut callable_declarations = BTreeMap::new();
                let mut callable_constants = BTreeSet::new();
                collect_scalar_declarations(
                    &function.body,
                    &mut callable_declarations,
                    &mut callable_constants,
                    &functions,
                );
                for name in global_names {
                    let key = variable_key(name);
                    let ty = callable_declarations
                        .get(&key)
                        .copied()
                        .unwrap_or_else(|| type_for_ident(name));
                    declarations.entry(key).or_insert(ty);
                }
            }
        }
        if let Some(module) = semantic_module {
            collect_semantic_array_declarations(module, &mut arrays);
        } else {
            for array in &typed_array_declarations {
                arrays.insert(
                    variable_key(&array.name),
                    ArrayShape {
                        element: type_for_ident(&array.name),
                        dimensions: array
                            .dimensions
                            .iter()
                            .cloned()
                            .map(ArrayDimension::Legacy)
                            .collect(),
                    },
                );
            }
            if arrays.is_empty() {
                collect_array_declarations(&program.statements, &mut arrays);
            }
        }
        for key in arrays.keys() {
            declarations.remove(key);
            constant_names.remove(key);
        }
        let (needs_file_io, needs_input, semantic_needs_inkey) = semantic_module
            .map(semantic_runtime_features)
            .unwrap_or_else(|| {
                (
                    program_uses_random_open(&program.statements)
                        || program
                            .functions
                            .iter()
                            .any(|f| program_uses_random_open(&f.body)),
                    program_uses_input(&program.statements)
                        || program
                            .functions
                            .iter()
                            .any(|f| program_uses_input(&f.body)),
                    program_uses_inkey(&program.statements)
                        || program
                            .functions
                            .iter()
                            .any(|f| program_uses_inkey(&f.body)),
                )
            });
        let needs_inkey = if semantic_module.is_some() {
            semantic_needs_inkey
        } else {
            program_uses_inkey(&program.statements)
                || program
                    .functions
                    .iter()
                    .any(|f| program_uses_inkey(&f.body))
        };
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
        let has_semantic_name_scopes = semantic_name_scopes.is_some();
        Ok(Self {
            variables,
            arrays,
            array_slots: BTreeMap::new(),
            local_array_slots: Vec::new(),
            array_aliases: BTreeMap::new(),
            constant_names,
            local_count: Cell::new(next_slot),
            initializer_start: 1,
            functions,
            semantic_module: semantic_module.cloned(),
            semantic_name_scopes,
            class_name,
            condition_label: Cell::new(0),
            semantic_return: None,
            initialize_static: true,
            field_vars,
            needs_file_io,
            data_items,
            data_labels,
            needs_input,
            needs_inkey,
            byref_scalar_params: Vec::new(),
            function_global_declarations: if has_semantic_name_scopes {
                HashMap::new()
            } else {
                function_global_declarations
            },
            byref_scratch_base,
        })
    }

    fn for_function(function: &FunctionDef, parent: &Self) -> Self {
        let signature = parent
            .functions
            .get(&callable_key_for_function(
                function,
                parent.semantic_module.as_ref(),
            ))
            .expect("registered callable signature");
        let semantic_return = Some((signature.result, signature.returns_void));
        let mut variables = BTreeMap::new();
        let mut next_slot = 0;
        // Top-level CONST bindings are immutable source variables, but remain
        // file-scope storage so callable bodies can read them without
        // substituting their initializer expression at every use site.
        for key in &parent.constant_names {
            if let Some(variable) = parent.variables.get(key) {
                variables.insert(key.clone(), *variable);
            }
        }
        if let Some(self_ident) = signature.receiver_ident.as_ref() {
            let variable = Variable {
                slot: next_slot,
                ty: *signature
                    .params
                    .first()
                    .expect("receiver has a resolved JVM type"),
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
        let mut scalar_parameters = signature
            .params
            .iter()
            .skip(usize::from(signature.has_receiver));
        for position in 0..signature.source_param_count {
            let parameter_name = &signature.parameter_names[position];
            if signature
                .array_params
                .iter()
                .any(|array| array.position == position)
            {
                parameter_array_slots.insert(variable_key(parameter_name), next_slot);
                next_slot += 1;
            } else if signature.byref_scalar_positions.contains(&position) {
                let ty = *scalar_parameters
                    .next()
                    .expect("registered scalar parameter type");
                byref_scalar_param_names.push((parameter_name.clone(), next_slot, ty));
                next_slot += 1;
            } else {
                let ty = *scalar_parameters
                    .next()
                    .expect("registered scalar parameter type");
                let variable = Variable {
                    slot: next_slot,
                    ty,
                    is_static: false,
                };
                next_slot += variable.width();
                variables.insert(variable_key(parameter_name), variable);
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
        for (name, array_slot, ty) in byref_scalar_param_names {
            let working = Variable {
                slot: next_slot,
                ty,
                is_static: false,
            };
            next_slot += working.width();
            variables.insert(variable_key(&name), working);
            byref_scalar_params.push((array_slot, working));
        }
        let initializer_start = next_slot;
        let mut declarations = BTreeMap::new();
        let mut constant_names = parent.constant_names.clone();
        let mut semantic_callable_body = None;
        if let Some(module) = parent.semantic_module.as_ref() {
            let semantic_callable = semantic_callable_for_function(module, function);
            if let Some(callable) = semantic_callable {
                let mut body_module = module.clone();
                body_module.statements = callable.body.clone();
                // File-record temporaries and buffers are explicit in
                // lowered_record_files; semantic declarations own all other
                // callable slots when standalone records are absent.
                collect_semantic_scalar_declarations(&body_module, &mut declarations);
                constant_names.extend(
                    body_module
                        .top_level_const_types()
                        .keys()
                        .map(|name| variable_key(&BasicIdent::parse(name))),
                );
                semantic_callable_body = Some(body_module);
            } else {
                collect_scalar_declarations(
                    &function.body,
                    &mut declarations,
                    &mut constant_names,
                    &parent.functions,
                );
            }
        } else {
            collect_scalar_declarations(
                &function.body,
                &mut declarations,
                &mut constant_names,
                &parent.functions,
            );
        }
        let semantic_globals = parent
            .semantic_module
            .as_ref()
            .and_then(|module| semantic_callable_for_function(module, function))
            .map(|callable| {
                crate::semantic_ir::SemanticModule::global_declarations_in(&callable.body)
            });
        let global_names = semantic_globals
            .map(|names| {
                names
                    .into_iter()
                    .map(|name| BasicIdent::parse(&name))
                    .collect()
            })
            .unwrap_or_else(|| {
                parent
                    .function_global_declarations
                    .get(&(
                        function.name.name.to_ascii_lowercase(),
                        function.name.suffix,
                    ))
                    .cloned()
                    .unwrap_or_default()
            });
        for name in global_names {
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
        let mut local_arrays = BTreeMap::new();
        if let Some(body_module) = semantic_callable_body {
            collect_semantic_array_declarations(&body_module, &mut local_arrays);
            arrays.extend(local_arrays.clone());
        }
        let mut array_slots = parameter_array_slots;
        let array_aliases = parent.array_aliases.clone();
        for position in 0..signature.source_param_count {
            let parameter_name = &signature.parameter_names[position];
            if let Some(array) = signature
                .array_params
                .iter()
                .find(|array| array.position == position)
            {
                arrays.insert(
                    variable_key(parameter_name),
                    ArrayShape {
                        element: array.element,
                        dimensions: vec![
                            ArrayDimension::Typed(0);
                            array.rank
                        ],
                    },
                );
            }
        }
        let parameter_array_names = signature
            .array_params
            .iter()
            .map(|array| variable_key(&signature.parameter_names[array.position]))
            .collect::<BTreeSet<_>>();
        let local_array_names = local_arrays
            .keys()
            .filter(|key| !parameter_array_names.contains(*key))
            .cloned()
            .collect::<Vec<_>>();
        for key in &local_array_names {
            array_slots.insert(key.clone(), next_slot);
            next_slot += 1;
        }
        let local_array_slots = local_array_names
            .iter()
            .filter_map(|key| array_slots.get(key).copied())
            .collect();
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
            local_array_slots,
            array_aliases,
            constant_names,
            local_count: Cell::new(next_slot),
            initializer_start,
            functions: parent.functions.clone(),
            semantic_module: parent.semantic_module.clone(),
            semantic_name_scopes: parent.semantic_name_scopes.clone(),
            class_name: parent.class_name.clone(),
            condition_label: Cell::new(0),
            semantic_return,
            initialize_static: false,
            field_vars: parent.field_vars.clone(),
            needs_file_io: parent.needs_file_io,
            data_items: parent.data_items.clone(),
            data_labels: parent.data_labels.clone(),
            needs_input: parent.needs_input,
            needs_inkey: parent.needs_inkey,
            byref_scalar_params,
            function_global_declarations: parent.function_global_declarations.clone(),
            byref_scratch_base,
        }
    }

    fn local_count(&self) -> usize {
        self.local_count.get()
    }

    fn reserve_semantic_int_local(&self) -> Variable {
        let slot = self.local_count.get();
        self.local_count.set(slot + 1);
        Variable {
            slot,
            ty: JvmType::Numeric(NumericType::Int),
            is_static: false,
        }
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
        if let Some(variable) = self.variables.get(&variable_key(ident)).copied() {
            return Ok(variable);
        }
        // A CONST declaration may be referenced with or without the
        // inferred type suffix. Preserve the source binding's single storage
        // slot rather than treating the suffixed reference as a new scalar.
        if let Some(key) = self.constant_names.iter().find(|key| {
            key.trim_end_matches(|ch| matches!(ch, '%' | '$' | '!' | '#' | '&'))
                == ident.name.to_ascii_lowercase()
        }) {
            if let Some(variable) = self.variables.get(key).copied() {
                return Ok(variable);
            }
        }
        Err(format!(
            "`{ident}` must be assigned or declared before use under --target jvm"
        ))
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
                        .variable(name)
                        .ok()
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
        for slot in &self.local_array_slots {
            out.push_str(&format!("    aconst_null\n    astore {slot}\n"));
        }
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
    constant_names: &mut BTreeSet<String>,
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
                declarations.insert(variable_key(name), type_for_const_expr(value, name));
                constant_names.insert(variable_key(name));
            }
            Statement::If {
                then_body,
                else_body,
                ..
            } => {
                collect_scalar_declarations(then_body, declarations, constant_names, functions);
                collect_scalar_declarations(else_body, declarations, constant_names, functions);
            }
            Statement::For { var, body, .. } => {
                declarations.insert(variable_key(var), type_for_ident(var));
                collect_scalar_declarations(body, declarations, constant_names, functions);
            }
            Statement::While { body, .. } | Statement::Do { body, .. } => {
                collect_scalar_declarations(body, declarations, constant_names, functions);
            }
            Statement::TryCatch {
                try_body,
                catch,
                finally_body,
            } => {
                collect_scalar_declarations(try_body, declarations, constant_names, functions);
                if let Some(catch) = catch {
                    declarations
                        .insert(variable_key(&catch.err_var), type_for_ident(&catch.err_var));
                    declarations
                        .insert(variable_key(&catch.erl_var), type_for_ident(&catch.erl_var));
                    if let Some(source_var) = &catch.source_var {
                        declarations.insert(variable_key(source_var), JvmType::String);
                    }
                    collect_scalar_declarations(
                        &catch.body,
                        declarations,
                        constant_names,
                        functions,
                    );
                }
                collect_scalar_declarations(finally_body, declarations, constant_names, functions);
            }
            Statement::SelectCase {
                cases, else_body, ..
            } => {
                for case in cases {
                    collect_scalar_declarations(
                        &case.body,
                        declarations,
                        constant_names,
                        functions,
                    );
                }
                collect_scalar_declarations(else_body, declarations, constant_names, functions);
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
                        dimensions: sizes
                            .iter()
                            .cloned()
                            .map(ArrayDimension::Legacy)
                            .collect(),
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
    jvm_type_for_type_suffix(ident.suffix.unwrap_or(TypeSuffix::Single))
}

fn jvm_type_for_type_suffix(suffix: TypeSuffix) -> JvmType {
    match suffix {
        TypeSuffix::String => JvmType::String,
        TypeSuffix::Integer => JvmType::Numeric(NumericType::Int),
        TypeSuffix::Long => JvmType::Numeric(NumericType::Long),
        // Doubles are a safe widening internal representation for BASCAL's
        // default single-precision scalar until a later precision pass.
        TypeSuffix::Single | TypeSuffix::Double => JvmType::Numeric(NumericType::Double),
    }
}

fn jvm_type_for_semantic_value(value_type: crate::semantic_ir::SemanticValueType) -> JvmType {
    match value_type {
        crate::semantic_ir::SemanticValueType::String => JvmType::String,
        crate::semantic_ir::SemanticValueType::Integer => JvmType::Numeric(NumericType::Int),
        crate::semantic_ir::SemanticValueType::Long => JvmType::Numeric(NumericType::Long),
        crate::semantic_ir::SemanticValueType::Single
        | crate::semantic_ir::SemanticValueType::Double => JvmType::Numeric(NumericType::Double),
        crate::semantic_ir::SemanticValueType::Boolean
        | crate::semantic_ir::SemanticValueType::Unknown => JvmType::Numeric(NumericType::Int),
    }
}

fn jvm_type_for_semantic_suffix(suffix: Option<&str>) -> JvmType {
    let value_type = suffix
        .and_then(|suffix| suffix.chars().next())
        .map(|character| crate::semantic_ir::SemanticValueType::from_suffix(Some(character)))
        .unwrap_or(crate::semantic_ir::SemanticValueType::Single);
    jvm_type_for_semantic_value(value_type)
}

fn type_for_const_expr(expr: &Expr, name: &BasicIdent) -> JvmType {
    match expr {
        Expr::String(_) => JvmType::String,
        Expr::Integer(value) if i32::try_from(*value).is_ok() => JvmType::Numeric(NumericType::Int),
        Expr::Integer(_) => JvmType::Numeric(NumericType::Long),
        Expr::Float(_) => JvmType::Numeric(NumericType::Double),
        Expr::Unary { expr, .. } => type_for_const_expr(expr, name),
        Expr::Binary { left, op, right } => match op {
            BinaryOp::Div | BinaryOp::Pow => JvmType::Numeric(NumericType::Double),
            BinaryOp::IntDiv | BinaryOp::Mod | BinaryOp::And | BinaryOp::Or | BinaryOp::Xor => {
                JvmType::Numeric(NumericType::Long)
            }
            _ => match (
                type_for_const_expr(left, name),
                type_for_const_expr(right, name),
            ) {
                (JvmType::String, _) | (_, JvmType::String) => JvmType::String,
                (JvmType::Numeric(NumericType::Double), _)
                | (_, JvmType::Numeric(NumericType::Double)) => {
                    JvmType::Numeric(NumericType::Double)
                }
                (JvmType::Numeric(NumericType::Long), _)
                | (_, JvmType::Numeric(NumericType::Long)) => JvmType::Numeric(NumericType::Long),
                _ => JvmType::Numeric(NumericType::Int),
            },
        },
        _ => type_for_ident(name),
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
            let descriptor_args = signature
                .params
                .iter()
                .map(|ty| descriptor(*ty))
                .collect::<String>();
            out.push_str(&format!(
                "    invokestatic {}/{} ({descriptor_args}){}\n",
                context.class_name,
                name.name,
                descriptor(signature.result)
            ));
            Ok(())
        }
        Expr::Call { name, args }
        | Expr::ArrayRef {
            name,
            indices: args,
        } if name.name.eq_ignore_ascii_case("chr") && args.len() == 1 => {
            emit_numeric_expr_as(&args[0], NumericType::Int, out, context)?;
            out.push_str(
                "    i2c\n    invokestatic java/lang/String/valueOf (C)Ljava/lang/String;\n",
            );
            Ok(())
        }
        // `MKI$(n)` -- packs `n` as a raw little-endian 16-bit int, the same
        // two-byte layout `FIELD`/`GET`/`PUT` need for an `int16` record
        // field; see `codegen_c.rs`'s own `bcc_mki`/`CVI`'s doc comment for
        // why little-endian (real MBASIC/BASCOM's own on-disk layout, true
        // of every realistic deployment platform).
        Expr::Call { name, args }
        | Expr::ArrayRef {
            name,
            indices: args,
        } if name.name.eq_ignore_ascii_case("mki") && args.len() == 1 => {
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
        Expr::Call { name, args }
        | Expr::ArrayRef {
            name,
            indices: args,
        } if name.name.eq_ignore_ascii_case("mkl") && args.len() == 1 => {
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
        Expr::Call { name, args }
        | Expr::ArrayRef {
            name,
            indices: args,
        } if name.name.eq_ignore_ascii_case("mks") && args.len() == 1 => {
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
        Expr::Call { name, args }
        | Expr::ArrayRef {
            name,
            indices: args,
        } if name.name.eq_ignore_ascii_case("mkd") && args.len() == 1 => {
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
        Expr::Call { name, args }
        | Expr::ArrayRef {
            name,
            indices: args,
        } if name.name.eq_ignore_ascii_case("mid") && args.len() == 3 => {
            emit_string_expr(&args[0], out, context)?;
            emit_numeric_expr_as(&args[1], NumericType::Int, out, context)?;
            out.push_str("    iconst_1\n    isub\n    dup\n");
            emit_numeric_expr_as(&args[2], NumericType::Int, out, context)?;
            out.push_str(
                "    iadd\n    invokevirtual java/lang/String/substring (II)Ljava/lang/String;\n",
            );
            Ok(())
        }
        Expr::Call { name, args }
        | Expr::ArrayRef {
            name,
            indices: args,
        } if name.name.eq_ignore_ascii_case("mid") && args.len() == 2 => {
            emit_string_expr(&args[0], out, context)?;
            emit_numeric_expr_as(&args[1], NumericType::Int, out, context)?;
            out.push_str("    iconst_1\n    isub\n    invokevirtual java/lang/String/substring (I)Ljava/lang/String;\n");
            Ok(())
        }
        Expr::Call { name, args }
        | Expr::ArrayRef {
            name,
            indices: args,
        } if name.name.eq_ignore_ascii_case("left") && args.len() == 2 => {
            emit_string_expr(&args[0], out, context)?;
            out.push_str("    iconst_0\n");
            emit_numeric_expr_as(&args[1], NumericType::Int, out, context)?;
            out.push_str("    invokevirtual java/lang/String/substring (II)Ljava/lang/String;\n");
            Ok(())
        }
        Expr::Call { name, args }
        | Expr::ArrayRef {
            name,
            indices: args,
        } if name.name.eq_ignore_ascii_case("right") && args.len() == 2 => {
            emit_string_expr(&args[0], out, context)?;
            out.push_str("    dup\n    invokevirtual java/lang/String/length ()I\n");
            emit_numeric_expr_as(&args[1], NumericType::Int, out, context)?;
            out.push_str(
                "    isub\n    invokevirtual java/lang/String/substring (I)Ljava/lang/String;\n",
            );
            Ok(())
        }
        Expr::Call { name, args }
        | Expr::ArrayRef {
            name,
            indices: args,
        } if (name.name.eq_ignore_ascii_case("lcase")
            || name.name.eq_ignore_ascii_case("ucase"))
            && args.len() == 1 =>
        {
            emit_string_expr(&args[0], out, context)?;
            let method = if name.name.eq_ignore_ascii_case("lcase") {
                "toLowerCase"
            } else {
                "toUpperCase"
            };
            out.push_str(&format!(
                "    invokevirtual java/lang/String/{method} ()Ljava/lang/String;\n"
            ));
            Ok(())
        }
        Expr::Call { name, args }
        | Expr::ArrayRef {
            name,
            indices: args,
        } if name.name.eq_ignore_ascii_case("trim") && args.len() == 1 => {
            emit_string_expr(&args[0], out, context)?;
            out.push_str("    invokevirtual java/lang/String/trim ()Ljava/lang/String;\n");
            Ok(())
        }
        Expr::Call { name, args }
        | Expr::ArrayRef {
            name,
            indices: args,
        } if name.name.eq_ignore_ascii_case("space") && args.len() == 1 => {
            emit_numeric_expr_as(&args[0], NumericType::Int, out, context)?;
            emit_space_string_runtime(out);
            Ok(())
        }
        Expr::Call { name, args }
        | Expr::ArrayRef {
            name,
            indices: args,
        } if name.name.eq_ignore_ascii_case("string") && args.len() == 2 => {
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
        Expr::Call { name, args }
        | Expr::ArrayRef {
            name,
            indices: args,
        } if context.arrays.contains_key(&variable_key(name)) => {
            let shape = context
                .arrays
                .get(&variable_key(name))
                .expect("array exists");
            if args.len() != shape.dimensions.len() || shape.element != JvmType::String {
                return Err("JVM string-array access has the wrong type or dimensions".to_string());
            }
            context.emit_array_load(name, out);
            for index in &args[..args.len() - 1] {
                emit_numeric_expr_as(index, NumericType::Int, out, context)?;
                out.push_str("    aaload\n");
            }
            emit_numeric_expr_as(
                args.last().expect("validated index count"),
                NumericType::Int,
                out,
                context,
            )?;
            out.push_str("    aaload\n");
            Ok(())
        }
        Expr::Call { name, args }
        | Expr::ArrayRef {
            name,
            indices: args,
        } if context.function(name).is_some() => {
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
            if name.suffix == Some(TypeSuffix::String)
                && name.name.eq_ignore_ascii_case("inkey") =>
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
            if name.suffix == Some(TypeSuffix::String)
                && name.name.eq_ignore_ascii_case("date") =>
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
        Expr::Ident(name) => {
            let variable = context.variable(name)?;
            if !matches!(variable.ty, JvmType::String) {
                return Err(format!("`{name}` is numeric, not a string"));
            }
            emit_load(variable, out, context);
            Ok(())
        }
        Expr::Call { name, args }
        | Expr::ArrayRef {
            name,
            indices: args,
        } if name.name.eq_ignore_ascii_case("str") && args.len() == 1 => {
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
            out.push_str(
                "    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;\n",
            );
            Ok(())
        }
        other => Err(format!(
            "{other:?} is not supported by the JVM backend yet -- only string literals, scalar variables, and `+` concatenation are"
        )),
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
            let suffixes = [
                TypeSuffix::Integer,
                TypeSuffix::Long,
                TypeSuffix::Single,
                TypeSuffix::Double,
            ];
            let (name, signature) = suffixes
                .iter()
                .filter_map(|suffix| {
                    let ident = BasicIdent {
                        name: method.clone(),
                        suffix: Some(*suffix),
                    };
                    context.function(&ident).map(|sig| (ident, sig))
                })
                .find(|(_, sig)| {
                    sig.params.first() == Some(&JvmType::Numeric(base_ty))
                        && sig.params.len() == args.len() + 1
                        && !sig.returns_void
                })
                .ok_or_else(|| format!("unsupported JVM numeric method `{method}`"))?;
            for (arg, ty) in args.iter().zip(signature.params.iter().skip(1)) {
                match ty {
                    JvmType::String => emit_string_expr(arg, out, context)?,
                    JvmType::Numeric(ty) => emit_numeric_expr_as(arg, *ty, out, context)?,
                }
            }
            let descriptor_args = signature
                .params
                .iter()
                .map(|ty| descriptor(*ty))
                .collect::<String>();
            out.push_str(&format!(
                "    invokestatic {}/{} ({descriptor_args}){}\n",
                context.class_name,
                name.name,
                descriptor(signature.result)
            ));
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
            out.push_str(&format!(
                "    invokestatic java/lang/Math/abs {descriptor}\n"
            ));
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
            if (name.name.eq_ignore_ascii_case("csng")
                || name.name.eq_ignore_ascii_case("cdbl"))
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
            let method = if name.name.eq_ignore_ascii_case("min") {
                "min"
            } else {
                "max"
            };
            out.push_str(&format!(
                "    invokestatic java/lang/Math/{method} ({suffix}{suffix}){suffix}\n"
            ));
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
        Expr::Call { name, args }
        | Expr::ArrayRef {
            name,
            indices: args,
        } if ["sizeof", "lbound", "ubound"]
            .iter()
            .any(|builtin| name.name.eq_ignore_ascii_case(builtin))
            && (1..=2).contains(&args.len()) =>
        {
            let array = match &args[0] {
                Expr::Ident(array) => array,
                Expr::ArrayRef { name, indices } if indices.is_empty() => name,
                _ => {
                    return Err(format!(
                        "JVM {} requires an array identifier",
                        name.name.to_ascii_uppercase()
                    ));
                }
            };
            let shape = context
                .arrays
                .get(&variable_key(array))
                .ok_or_else(|| format!("unknown JVM array `{array}`"))?;
            let axis = match args.get(1) {
                Some(Expr::Integer(axis)) if *axis >= 0 => *axis as usize,
                Some(_) => return Err("JVM array bound axis must be a literal integer".to_string()),
                None if shape.dimensions.len() == 1 => 0,
                None => {
                    return Err(format!(
                        "JVM {} requires an axis for multidimensional arrays",
                        name.name.to_ascii_uppercase()
                    ));
                }
            };
            if axis >= shape.dimensions.len() {
                return Err(format!("JVM array axis {axis} is out of range"));
            }
            if name.name.eq_ignore_ascii_case("lbound") {
                out.push_str("    iconst_0\n");
            } else if context.array_slots.contains_key(&variable_key(array)) {
                context.emit_array_axis_length(array, axis, out);
                if name.name.eq_ignore_ascii_case("ubound") {
                    out.push_str("    iconst_1\n    isub\n");
                }
            } else {
                emit_jvm_array_dimension(&shape.dimensions[axis], out, context)?;
                if name.name.eq_ignore_ascii_case("sizeof") {
                    out.push_str("    iconst_1\n    iadd\n");
                }
            }
            Ok(NumericType::Int)
        }
        Expr::Call { name, args }
        | Expr::ArrayRef {
            name,
            indices: args,
        } if context.function(name).is_some() => {
            let signature = context.function(name).expect("checked above");
            let JvmType::Numeric(result) = signature.result else {
                return Err(format!("`{name}` returns a string, not a numeric value"));
            };
            emit_function_call(name, args, signature.result, out, context)?;
            Ok(result)
        }
        Expr::Call { name, args } if context.arrays.contains_key(&variable_key(name)) => {
            let shape = context
                .arrays
                .get(&variable_key(name))
                .expect("array exists");
            if args.len() != shape.dimensions.len() {
                return Err("JVM indexed access has the wrong number of dimensions".to_string());
            }
            context.emit_array_load(name, out);
            for index in &args[..args.len() - 1] {
                emit_numeric_expr_as(index, NumericType::Int, out, context)?;
                out.push_str("    aaload\n");
            }
            emit_numeric_expr_as(
                args.last().expect("validated index count"),
                NumericType::Int,
                out,
                context,
            )?;
            out.push_str(&format!("    {}\n", array_load_opcode(shape.element)));
            match shape.element {
                JvmType::Numeric(ty) => Ok(ty),
                JvmType::String => {
                    Err("string array used where a numeric value was expected".to_string())
                }
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
            emit_numeric_expr_as(
                indices.last().expect("validated index count"),
                NumericType::Int,
                out,
                context,
            )?;
            out.push_str(&format!("    {}\n", array_load_opcode(shape.element)));
            match shape.element {
                JvmType::Numeric(ty) => Ok(ty),
                JvmType::String => {
                    Err("string array used where a numeric value was expected".to_string())
                }
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
        Expr::Float(_) => {
            Err("non-finite numeric literals are not supported by the JVM backend".to_string())
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
            let ty = promote_numeric(
                infer_numeric_type(left, context)?,
                infer_numeric_type(right, context)?,
            );
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
        Expr::Binary {
            left,
            op: BinaryOp::Div,
            right,
        } => {
            emit_numeric_expr_as(left, NumericType::Double, out, context)?;
            emit_numeric_expr_as(right, NumericType::Double, out, context)?;
            out.push_str("    ddiv\n");
            Ok(NumericType::Double)
        }
        Expr::Binary {
            left,
            op: BinaryOp::Pow,
            right,
        } => {
            emit_numeric_expr_as(left, NumericType::Double, out, context)?;
            emit_numeric_expr_as(right, NumericType::Double, out, context)?;
            out.push_str("    invokestatic java/lang/Math/pow (DD)D\n");
            Ok(NumericType::Double)
        }
        Expr::Binary {
            left,
            op: BinaryOp::IntDiv | BinaryOp::Mod,
            right,
        } => {
            emit_numeric_expr_as(left, NumericType::Double, out, context)?;
            emit_round_away_from_zero(out);
            emit_numeric_expr_as(right, NumericType::Double, out, context)?;
            emit_round_away_from_zero(out);
            out.push_str(
                if matches!(
                    expr,
                    Expr::Binary {
                        op: BinaryOp::IntDiv,
                        ..
                    }
                ) {
                    "    ldiv\n"
                } else {
                    "    lrem\n"
                },
            );
            Ok(NumericType::Long)
        }
        Expr::Binary {
            left,
            op: op @ (BinaryOp::And | BinaryOp::Or | BinaryOp::Xor),
            right,
        } => {
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
        Expr::Binary {
            left,
            op:
                op @ (BinaryOp::Eq
                | BinaryOp::Ne
                | BinaryOp::Lt
                | BinaryOp::Le
                | BinaryOp::Gt
                | BinaryOp::Ge),
            right,
        } => {
            let ty = promote_numeric(
                infer_numeric_type(left, context)?,
                infer_numeric_type(right, context)?,
            );
            emit_numeric_expr_as(left, ty, out, context)?;
            emit_numeric_expr_as(right, ty, out, context)?;
            if ty == NumericType::Double {
                let (compare, branch) = match op {
                    BinaryOp::Eq => ("dcmpl", "ifeq"),
                    BinaryOp::Ne => ("dcmpl", "ifne"),
                    BinaryOp::Lt => ("dcmpg", "iflt"),
                    BinaryOp::Le => ("dcmpg", "ifle"),
                    BinaryOp::Gt => ("dcmpl", "ifgt"),
                    BinaryOp::Ge => ("dcmpl", "ifge"),
                    _ => unreachable!(),
                };
                let truth = next_condition_label(context);
                let done = next_condition_label(context);
                out.push_str(&format!(
                    "    {compare}\n    {branch} {truth}\n    iconst_0\n    goto {done}\n{truth}:\n    iconst_m1\n{done}:\n"
                ));
                return Ok(NumericType::Int);
            }
            out.push_str(match ty {
                NumericType::Int => "    invokestatic java/lang/Integer/compare (II)I\n",
                NumericType::Long => "    invokestatic java/lang/Long/compare (JJ)I\n",
                NumericType::Double => unreachable!(),
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
        Expr::Ident(name) if name.name.eq_ignore_ascii_case("pi") && name.suffix.is_none() => {
            Ok(NumericType::Double)
        }
        Expr::Call { name, args } if name.name.eq_ignore_ascii_case("asc") && args.len() == 1 => {
            Ok(NumericType::Int)
        }
        Expr::Call { name, args } if name.name.eq_ignore_ascii_case("len") && args.len() == 1 => {
            Ok(NumericType::Int)
        }
        Expr::Call { name, args } if name.name.eq_ignore_ascii_case("instr") && args.len() == 2 => {
            Ok(NumericType::Int)
        }
        Expr::Call { name, args } if name.name.eq_ignore_ascii_case("cvi") && args.len() == 1 => {
            Ok(NumericType::Int)
        }
        Expr::Call { name, args } if name.name.eq_ignore_ascii_case("cvl") && args.len() == 1 => {
            Ok(NumericType::Int)
        }
        Expr::Call { name, args } if name.name.eq_ignore_ascii_case("cvs") && args.len() == 1 => {
            Ok(NumericType::Double)
        }
        Expr::Call { name, args } if name.name.eq_ignore_ascii_case("cvd") && args.len() == 1 => {
            Ok(NumericType::Double)
        }
        Expr::Call { name, args } if name.name.eq_ignore_ascii_case("cint") && args.len() == 1 => {
            Ok(NumericType::Int)
        }
        Expr::Call { name, args } if name.name.eq_ignore_ascii_case("clng") && args.len() == 1 => {
            Ok(NumericType::Long)
        }
        Expr::Call { name, args }
            if (name.name.eq_ignore_ascii_case("csng")
                || name.name.eq_ignore_ascii_case("cdbl"))
                && args.len() == 1 =>
        {
            Ok(NumericType::Double)
        }
        Expr::Call { name, args }
            if (name.name.eq_ignore_ascii_case("min") || name.name.eq_ignore_ascii_case("max"))
                && args.len() == 2 =>
        {
            let left = infer_numeric_type(&args[0], context)?;
            let right = infer_numeric_type(&args[1], context)?;
            Ok(promote_numeric(left, right))
        }
        Expr::Call { name, args } if name.name.eq_ignore_ascii_case("val") && args.len() == 1 => {
            Ok(NumericType::Double)
        }
        Expr::Call { name, args }
            if name.name.eq_ignore_ascii_case("sizeof") && args.len() == 1 =>
        {
            Ok(NumericType::Int)
        }
        Expr::Call { name, .. } | Expr::ArrayRef { name, .. }
            if context.function(name).is_some() =>
        {
            match context.function(name).expect("checked above").result {
                JvmType::Numeric(ty) => Ok(ty),
                JvmType::String => Err(format!("`{name}` is a string function")),
            }
        }
        Expr::Integer(value) if i32::try_from(*value).is_ok() => Ok(NumericType::Int),
        Expr::Integer(_) => Ok(NumericType::Long),
        Expr::Float(value) if value.is_finite() => Ok(NumericType::Double),
        Expr::Float(_) => {
            Err("non-finite numeric literals are not supported by the JVM backend".to_string())
        }
        Expr::Ident(name) => match context.variable(name)?.ty {
            JvmType::Numeric(ty) => Ok(ty),
            JvmType::String => Err(format!("`{name}` is a string, not numeric")),
        },
        Expr::Unary {
            op: UnaryOp::Neg,
            expr,
        } => infer_numeric_type(expr, context),
        Expr::Binary {
            left,
            op: BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul,
            right,
        } => Ok(promote_numeric(
            infer_numeric_type(left, context)?,
            infer_numeric_type(right, context)?,
        )),
        Expr::Binary {
            op: BinaryOp::Div | BinaryOp::Pow,
            ..
        } => Ok(NumericType::Double),
        Expr::Unary {
            op: UnaryOp::Not, ..
        } => Ok(NumericType::Long),
        Expr::Binary {
            op: BinaryOp::IntDiv | BinaryOp::Mod | BinaryOp::And | BinaryOp::Or | BinaryOp::Xor,
            ..
        } => Ok(NumericType::Long),
        Expr::Binary {
            op:
                BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge,
            ..
        } => Ok(NumericType::Int),
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

#[cfg(test)]
mod tests {
    use super::semantic_runtime_features;

    #[test]
    fn jvm_generation_consumes_semantic_global_declarations() {
        let ast_source =
            "legacy% = 1\nfunction read%()\nlegacyLocal% = 1\nreturn 0\nend function\nend\n";
        let semantic_source = "global canonical%\nfunction read%()\nglobal callableCanonical%\nreturn 0\nend function\nend\n";
        let parsed = crate::parse_source("jvm_semantic_global.bcl".to_string(), ast_source)
            .expect("legacy source parses");
        let crate::lower::Lowered { program, .. } =
            crate::lower::lower(parsed).expect("legacy source lowers");
        let semantic =
            crate::semantic_ir::parse_and_adapt_named("jvm_semantic_global.bcl", semantic_source)
                .expect("typed source parses");
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic))
            .expect("typed global scopes resolve");

        let output = super::generate(&resolved).expect("typed global declarations emit");
        assert!(
            !output.contains("ldc 1"),
            "AST global placeholder leaked: {output}"
        );
        assert!(
            output.contains(".field public static"),
            "global slots missing: {output}"
        );

        let functions = super::function_table(
            &resolved.program.functions,
            resolved.semantic_module.as_ref(),
        );
        let context = super::JvmContext::build(
            &resolved.program,
            functions,
            super::class_name_for(&resolved.program, resolved.semantic_module.as_ref()),
            resolved.function_global_declarations.clone(),
            resolved.typed_array_declarations.clone(),
            resolved.semantic_module.as_ref(),
            resolved.semantic_name_scopes.clone(),
        )
        .expect("typed JVM context builds");
        assert!(
            !context
                .variables
                .contains_key(&super::variable_key(&crate::ast::BasicIdent::parse(
                    "legacy%"
                ))),
            "AST-only global placeholder allocated a JVM slot"
        );
        let callable_context =
            super::JvmContext::for_function(&resolved.program.functions[0], &context);
        assert!(
            !callable_context
                .variables
                .contains_key(&super::variable_key(&crate::ast::BasicIdent::parse(
                    "legacyLocal%"
                ))),
            "AST-only callable placeholder allocated a JVM local slot"
        );
    }

    #[test]
    fn jvm_generation_dispatches_source_aligned_terminal_nodes_from_semantic_ir() {
        let source = "stop\ncls\nend\n";
        let parsed = crate::parse_source("jvm_semantic_dispatch.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_dispatch.bcl",
                "end\nbeep\nend\n",
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("    return\n"),
            "semantic END was not emitted: {main}"
        );
        assert!(
            !main.contains("java/lang/System/exit"),
            "AST STOP replaced semantic END: {main}"
        );
        assert!(
            main.contains("ldc \"\u{7}\""),
            "semantic BEEP was not emitted: {main}"
        );
        assert!(
            !main.contains("\u{1b}[2J"),
            "AST CLS replaced semantic BEEP: {main}"
        );
    }

    #[test]
    fn jvm_source_aligned_comment_dispatch_uses_typed_ir() {
        let ast_source = "print \"ast\"\n' AST comment\n\nend\n";
        let semantic_source = "print \"semantic\"\n/* typed comment */\n\nend\n";
        let parsed = crate::parse_source("jvm_semantic_comment.bcl".to_string(), ast_source)
            .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let resolved = crate::resolver::resolve_with_semantic(
            program,
            Some(
                crate::semantic_ir::parse_and_adapt_named(
                    "jvm_semantic_comment.bcl",
                    semantic_source,
                )
                .unwrap(),
            ),
        )
        .unwrap();

        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(main.contains("; typed comment"), "typed comment missing: {main}");
        assert!(!main.contains("AST comment"), "AST comment replaced typed IR: {main}");
        assert!(main.contains("semantic"), "typed PRINT missing: {main}");
        assert!(!main.contains("ast"), "AST PRINT replaced typed IR: {main}");

        let ast_source = "procedure show()\nprint \"ast\"\n' AST callable comment\n\nend procedure\nshow()\nend\n";
        let semantic_source = "procedure show()\nprint \"semantic\"\n/* typed callable comment */\n\nend procedure\nshow()\nend\n";
        let parsed = crate::parse_source(
            "jvm_semantic_callable_comment.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let resolved = crate::resolver::resolve_with_semantic(
            program,
            Some(
                crate::semantic_ir::parse_and_adapt_named(
                    "jvm_semantic_callable_comment.bcl",
                    semantic_source,
                )
                .unwrap(),
            ),
        )
        .unwrap();
        let output = super::generate(&resolved).unwrap();
        let callable = output
            .split(".method public static show :")
            .nth(1)
            .unwrap()
            .split(".end method")
            .next()
            .unwrap();
        assert!(callable.contains("; typed callable comment"), "typed callable comment missing: {callable}");
        assert!(!callable.contains("AST callable comment"), "AST callable comment replaced typed IR: {callable}");
        assert!(callable.contains("semantic"), "typed callable PRINT missing: {callable}");
        assert!(!callable.contains("ast"), "AST callable PRINT replaced typed IR: {callable}");
    }

    #[test]
    fn jvm_generation_dispatches_top_level_try_from_semantic_ir() {
        let filename = "jvm_semantic_try.bcl";
        let ast_source = "program p\ntry\nthrow 7\ncatch err%, erl%, source$\nprint \"AST catch\"\nfinally\nprint \"AST finally\"\nend try\nend\n";
        let semantic_source = "program p\ntry\nthrow 9\ncatch err%, erl%, source$\nprint \"typed catch\"\nfinally\nprint \"typed finally\"\nend try\nend\n";
        let parsed = crate::parse_source(filename.to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut semantic = crate::semantic_ir::parse_and_adapt_named(filename, semantic_source)
            .expect("typed TRY parses");
        fn stale_catch_names(statements: &mut [crate::semantic_ir::SemanticStatement]) -> bool {
            for statement in statements {
                match &mut statement.kind {
                    crate::semantic_ir::SemanticStatementKind::Try {
                        catch: Some(catch), ..
                    } => {
                        catch.error = "err$".to_string();
                        catch.line = "erl&".to_string();
                        catch.source = Some("source%".to_string());
                        return true;
                    }
                    crate::semantic_ir::SemanticStatementKind::Line(body) => {
                        if stale_catch_names(body) {
                            return true;
                        }
                    }
                    _ => {}
                }
            }
            false
        }
        assert!(stale_catch_names(&mut semantic.statements));
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic))
            .expect("typed TRY resolves");

        let output = super::generate(&resolved).expect("typed JVM TRY emits");
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("ldc 9"),
            "semantic THROW was not emitted: {main}"
        );
        assert!(
            main.contains("typed catch"),
            "semantic CATCH was not emitted: {main}"
        );
        assert!(
            main.contains("typed finally"),
            "semantic FINALLY was not emitted: {main}"
        );
        assert!(
            main.contains("jvm_semantic_try.bcl"),
            "semantic source binding missing: {main}"
        );
        assert!(
            !main.contains("AST catch"),
            "AST CATCH replaced semantic IR: {main}"
        );
        assert!(
            !main.contains("AST finally"),
            "AST FINALLY replaced semantic IR: {main}"
        );
    }

    #[test]
    fn jvm_semantic_source_positions_use_module_and_callable_sources() {
        use crate::semantic_ir::{SemanticStatement, SemanticStatementKind as Kind};
        fn find_statement(
            statements: &[SemanticStatement],
            matches_kind: fn(&Kind) -> bool,
        ) -> Option<&SemanticStatement> {
            for statement in statements {
                if matches_kind(&statement.kind) {
                    return Some(statement);
                }
                if let Kind::Line(children) = &statement.kind {
                    if let Some(found) = find_statement(children, matches_kind) {
                        return Some(found);
                    }
                }
            }
            None
        }

        let filename = "jvm_semantic_source_position.bcl";
        let source = "print 1\nprocedure worker()\nthrow 7\nend procedure\nend\n";
        let parsed = crate::parse_source(filename.to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let module = crate::semantic_ir::parse_and_adapt_named(filename, source).unwrap();
        let top_level = find_statement(&module.statements, |kind| matches!(kind, Kind::Print { .. }))
            .unwrap();
        let callable_statement = find_statement(&module.callables[0].body, |kind| {
            matches!(kind, Kind::Throw(_))
        })
        .unwrap();

        let top_level_position =
            super::jvm_semantic_top_level_source_position(&module, top_level).unwrap();
        let callable_position = super::jvm_semantic_callable_source_position(
            &module,
            &program.functions[0],
            callable_statement,
        )
        .unwrap();
        assert_eq!(top_level_position.filename, filename);
        assert_eq!(top_level_position.line, 1);
        assert_eq!(callable_position.filename, filename);
        assert_eq!(callable_position.line, 3);
    }

    #[test]
    fn jvm_callable_dispatches_try_from_semantic_ir() {
        let filename = "jvm_semantic_callable_try.bcl";
        let ast_source = "program p\nprocedure worker()\ntry\nthrow 7\ncatch err%, erl%\nprint \"AST catch\"\nfinally\nprint \"AST finally\"\nend try\nend procedure\nworker()\nend\n";
        let semantic_source = "program p\nprocedure worker()\ntry\nthrow 9\ncatch err%, erl%\nprint \"typed catch\"\nfinally\nprint \"typed finally\"\nend try\nend procedure\nworker()\nend\n";
        let parsed = crate::parse_source(filename.to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut semantic = crate::semantic_ir::parse_and_adapt_named(filename, semantic_source)
            .expect("typed callable TRY parses");
        fn stale_callable_catch_names(
            statements: &mut [crate::semantic_ir::SemanticStatement],
        ) -> bool {
            for statement in statements {
                match &mut statement.kind {
                    crate::semantic_ir::SemanticStatementKind::Try {
                        catch: Some(catch), ..
                    } => {
                        catch.error = "err$".to_string();
                        catch.line = "erl&".to_string();
                        return true;
                    }
                    crate::semantic_ir::SemanticStatementKind::Line(body) => {
                        if stale_callable_catch_names(body) {
                            return true;
                        }
                    }
                    _ => {}
                }
            }
            false
        }
        assert!(stale_callable_catch_names(&mut semantic.callables[0].body));
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic))
            .expect("typed callable TRY resolves");

        let output = super::generate(&resolved).expect("typed callable JVM TRY emits");
        let worker = output
            .split(".method public static worker :")
            .nth(1)
            .unwrap();
        let worker = worker.split(".end method").next().unwrap();
        assert!(
            worker.contains("ldc 9"),
            "semantic THROW was not emitted: {worker}"
        );
        assert!(
            worker.contains("typed catch"),
            "semantic CATCH was not emitted: {worker}"
        );
        assert!(
            worker.contains("typed finally"),
            "semantic FINALLY was not emitted: {worker}"
        );
        assert!(
            !worker.contains("AST catch"),
            "AST CATCH replaced semantic IR: {worker}"
        );
        assert!(
            !worker.contains("AST finally"),
            "AST FINALLY replaced semantic IR: {worker}"
        );
    }

    #[test]
    fn jvm_nested_try_dispatches_from_semantic_ir_inside_loop() {
        let filename = "jvm_semantic_nested_try.bcl";
        let source = |throw: i32, label: &str| {
            format!(
                "program p\nfor i% = 1 to 1\nif i% = 1 then\ntry\nthrow {throw}\ncatch err%, erl%\nprint \"{label} catch\"\nfinally\nprint \"{label} finally\"\nend try\nend if\nend for\nend\n"
            )
        };
        let parsed = crate::parse_source(filename.to_string(), &source(7, "AST")).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(filename, &source(9, "typed"))
            .expect("typed nested TRY parses");
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic))
            .expect("typed nested TRY resolves");

        let output = super::generate(&resolved).expect("typed nested JVM TRY emits");
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("ldc 9"),
            "semantic THROW was not emitted: {main}"
        );
        assert!(
            main.contains("typed catch"),
            "semantic CATCH was not emitted: {main}"
        );
        assert!(
            main.contains("typed finally"),
            "semantic FINALLY was not emitted: {main}"
        );
        assert!(
            !main.contains("AST catch"),
            "AST CATCH replaced typed IR: {main}"
        );
        assert!(main.matches(".catch java/lang/RuntimeException").count() >= 1);
    }

    #[test]
    fn jvm_semantic_dispatch_declines_when_source_identity_does_not_match() {
        let parsed = crate::parse_source("jvm_ast_origin.bcl".to_string(), "stop\n").unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let module =
            crate::semantic_ir::parse_and_adapt_named("jvm_sem_origin.bcl", "end\n").unwrap();
        assert!(super::jvm_semantic_statements_by_source(&module, &program.statements).is_none());
    }

    #[test]
    fn jvm_generation_emits_typed_top_level_stream_without_source_alignment() {
        let parsed =
            crate::parse_source("jvm_top_level_ast_origin.bcl".to_string(), "print 1\nend\n")
                .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "jvm_top_level_typed_origin.bcl",
            "print 2\nend\n",
        )
        .unwrap();
        assert!(super::jvm_semantic_statements_by_source(&semantic, &program.statements).is_none());
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::generate(&resolved).expect("typed top-level stream should emit");
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(main.contains("ldc 2"), "typed statement missing: {main}");
        assert!(!main.contains("ldc 1"), "AST statement leaked: {main}");
    }

    #[test]
    fn jvm_generation_emits_typed_top_level_stream_with_blank_lines() {
        let parsed = crate::parse_source(
            "jvm_blank_line_ast_origin.bcl".to_string(),
            "print 1\nend\n",
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "jvm_blank_line_typed_origin.bcl",
            "print 2\n\nend\n",
        )
        .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::generate(&resolved).expect("typed top-level stream should emit");
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(main.contains("ldc 2"), "typed statement missing: {main}");
        assert!(!main.contains("ldc 1"), "AST statement leaked: {main}");
    }

    #[test]
    fn jvm_generation_emits_typed_top_level_stream_across_source_files() {
        let parsed = crate::parse_source(
            "jvm_multisource_ast_origin.bcl".to_string(),
            "print 0\nend\n",
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut semantic = crate::semantic_ir::parse_and_adapt_named(
            "jvm_multisource_root.bcl",
            "try\nerror 4\ncatch rootError%, rootLine%, rootSource$\nprint rootSource$\nend try\nend\n",
        )
        .unwrap();
        let dependency = crate::semantic_ir::parse_and_adapt_named(
            "jvm_multisource_dependency.bcl",
            "try\nerror 3\ncatch dependencyError%, dependencyLine%, dependencySource$\nprint dependencySource$\nend try\n",
        )
        .unwrap();
        semantic.prepend_dependency(dependency);
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::generate(&resolved).expect("multi-source typed stream should emit");
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("ldc \"jvm_multisource_dependency.bcl\""),
            "dependency source metadata missing: {main}"
        );
        assert!(
            main.contains("ldc \"jvm_multisource_root.bcl\""),
            "root source metadata missing: {main}"
        );
        assert!(!main.contains("ldc 0"), "AST statement leaked: {main}");
    }

    #[test]
    fn jvm_typed_module_declines_atomically_when_source_identity_is_missing() {
        let parsed = crate::parse_source(
            "jvm_missing_source_ast.bcl".to_string(),
            "print 1\nend\n",
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut semantic = crate::semantic_ir::parse_and_adapt_named(
            "jvm_missing_source_typed.bcl",
            "print 2\nend\n",
        )
        .unwrap();
        *semantic.statement_sources.last_mut().unwrap() = usize::MAX;
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::generate(&resolved).expect("AST compatibility path should emit");
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(main.contains("ldc 1"), "AST fallback was not used: {main}");
        assert!(!main.contains("ldc 2"), "partial typed output leaked: {main}");
    }

    #[test]
    fn jvm_callable_dispatch_uses_semantic_body_across_arity_mismatch() {
        let filename = "jvm_callable_arity.bcl";
        let parsed = crate::parse_source(
            filename.to_string(),
            "function work%(value%)\nreturn value%\nend function\n",
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let module = crate::semantic_ir::parse_and_adapt_named(
            filename,
            "function work%(left%, right%)\nreturn right%\nend function\n",
        )
        .unwrap();

        let aligned =
            super::jvm_semantic_callable_statements_by_source(&module, &program.functions[0]);
        assert!(
            aligned.is_some(),
            "matching callable identity must select the typed body despite stale AST arity"
        );
    }

    #[test]
    fn jvm_callable_emits_complete_typed_body_without_ast_alignment() {
        let parsed = crate::parse_source(
            "jvm_callable_ast_origin.bcl".to_string(),
            "function work%()\nreturn 1\nend function\nend\n",
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "jvm_callable_typed_origin.bcl",
            "function work%()\nreturn 2\nend function\nend\n",
        )
        .unwrap();
        assert!(super::jvm_semantic_callable_statements_by_source(
            &semantic,
            &program.functions[0]
        )
        .is_none());
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::generate(&resolved).expect("typed callable should emit");
        let function = output
            .split(".method public static work :")
            .nth(1)
            .expect("function method should be emitted")
            .split(".end method")
            .next()
            .unwrap();
        assert!(
            function.contains("ldc 2"),
            "typed callable body missing: {function}"
        );
        assert!(!function.contains("ldc 1"), "AST body leaked: {function}");
    }

    #[test]
    fn jvm_generation_dispatches_aligned_semantic_goto_and_label_names() {
        let ast_source = "goto astTarget\n\nastTarget:\nend\n";
        let parsed =
            crate::parse_source("jvm_semantic_labels.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_labels.bcl",
                "goto semanticTarget\n\nsemanticTarget:\nend\n",
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("goto L_user_semantictarget"),
            "semantic GOTO target missing: {main}"
        );
        assert!(
            main.contains("L_user_semantictarget:"),
            "semantic label missing: {main}"
        );
        assert!(
            !main.contains("L_user_asttarget"),
            "AST label replaced semantic IR: {main}"
        );
    }

    #[test]
    fn jvm_generation_dispatches_aligned_typed_if_when_stream_is_split() {
        let filename = "jvm_semantic_if_split.bcl";
        let ast_source = "if 1 then\nprint 1\nelse\nprint 2\nend if\n\nend\n";
        let parsed = crate::parse_source(filename.to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                filename,
                "if 0 then\nprint 3\nelse\nprint 4\nend if\n\nend\n",
            )
            .unwrap(),
        );

        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(main.contains("ldc 3") && main.contains("ldc 4"), "typed IF arms missing: {main}");
        assert!(!main.contains("ldc 1") && !main.contains("ldc 2"), "AST IF arms leaked: {main}");
    }

    #[test]
    fn jvm_semantic_block_accepts_record_file_declarations_as_metadata() {
        let source = "record Item\nvalue: int16\nend record\nfile items as Item = open(\"items.dat\")\nend\n";
        let parsed = crate::parse_source("jvm_file_metadata.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named("jvm_file_metadata.bcl", source)
            .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();
        let module = resolved.semantic_module.as_ref().unwrap();
        let context = super::JvmContext::build(
            &resolved.program,
            super::function_table(&resolved.program.functions, Some(module)),
            "Program".to_string(),
            resolved.function_global_declarations.clone(),
            resolved.typed_array_declarations.clone(),
            Some(module),
            resolved.semantic_name_scopes.clone(),
        )
        .unwrap();
        let mut output = String::new();
        let handled = super::emit_jvm_semantic_block(
            &module.statements,
            &context,
            &mut Vec::new(),
            &mut Vec::new(),
            &mut super::JvmSemanticState {
                source_filename: "jvm_file_metadata.bcl".to_string(),
                next_label: 0,
                exception_handlers: Vec::new(),
            },
            &mut output,
        );

        assert!(handled, "typed record-file metadata triggered AST fallback");
        assert!(output.contains("return"), "typed END was not emitted: {output}");
    }

    #[test]
    fn jvm_semantic_goto_discovers_labels_nested_in_typed_blocks() {
        let filename = "jvm_semantic_nested_label.bcl";
        let parsed = crate::parse_source(
            filename.to_string(),
            "goto old\nif 1 then\nold:\nend if\nend\n",
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                filename,
                "goto fresh\nif 1 then\nfresh:\nend if\nend\n",
            )
            .unwrap(),
        );

        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("goto L_user_fresh"),
            "semantic target missing: {main}"
        );
        assert!(
            main.contains("L_user_fresh:"),
            "nested semantic label missing: {main}"
        );
        assert!(!main.contains("L_user_old"), "AST label leaked: {main}");
    }

    #[test]
    fn jvm_callable_generation_dispatches_aligned_semantic_goto_and_labels() {
        let ast_source = "procedure jump()\ngoto old\nold:\nprint 1\nend procedure\njump()\nend\n";
        let parsed =
            crate::parse_source("jvm_callable_semantic_goto.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_callable_semantic_goto.bcl",
                "procedure jump()\ngoto fresh\nfresh:\nprint 9\nend procedure\njump()\nend\n",
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let procedure = output.split(".method public static jump :").nth(1).unwrap();
        let procedure = procedure.split(".end method").next().unwrap();
        assert!(
            procedure.contains("goto L_user_fresh"),
            "semantic callable GOTO target missing: {procedure}"
        );
        assert!(
            procedure.contains("L_user_fresh:"),
            "semantic callable label missing: {procedure}"
        );
        assert!(
            procedure.contains("ldc 9"),
            "semantic callable statement following label missing: {procedure}"
        );
        assert!(
            !procedure.contains("L_user_old") && !procedure.contains("ldc 1"),
            "AST callable branch/label replaced semantic IR: {procedure}"
        );
    }

    #[test]
    fn jvm_generation_dispatches_typed_semantic_error_and_throw_codes() {
        let ast_source = "error 5\nthrow 6\nend\n";
        let semantic_source = "error 9 + 1\nthrow 12 + 3\nend\n";
        let parsed = crate::parse_source("jvm_semantic_error.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named("jvm_semantic_error.bcl", semantic_source)
                .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains(
                "ldc 9\n    ldc 1\n    iadd\n    invokestatic java/lang/Integer/toString"
            ),
            "semantic ERROR code missing: {main}"
        );
        assert!(
            main.contains(
                "ldc 12\n    ldc 3\n    iadd\n    invokestatic java/lang/Integer/toString"
            ),
            "semantic THROW code missing: {main}"
        );
        assert!(
            !main.contains("ldc 5") && !main.contains("ldc 6"),
            "AST error codes replaced semantic IR: {main}"
        );
    }

    #[test]
    fn jvm_callable_generation_dispatches_typed_error_and_throw_codes() {
        let ast_source = "procedure fail()\nerror 5\nthrow 6\nend procedure\nfail()\nend\n";
        let parsed =
            crate::parse_source("jvm_callable_semantic_error.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_callable_semantic_error.bcl",
                "procedure fail()\nerror 9 + 1\nthrow 12 + 3\nend procedure\nfail()\nend\n",
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let procedure = output.split(".method public static fail :").nth(1).unwrap();
        let procedure = procedure.split(".end method").next().unwrap();
        assert!(
            procedure.contains("ldc 9\n    ldc 1\n    iadd"),
            "semantic callable ERROR code missing: {procedure}"
        );
        assert!(
            procedure.contains("ldc 12\n    ldc 3\n    iadd"),
            "semantic callable THROW code missing: {procedure}"
        );
        assert!(
            !procedure.contains("ldc 5") && !procedure.contains("ldc 6"),
            "AST callable error codes replaced semantic IR: {procedure}"
        );
    }

    #[test]
    fn jvm_generation_dispatches_random_open_from_semantic_ir() {
        let ast_source = "open \"ast.dat\" for random as #1 len = 4\nend\n";
        let semantic_source = "open \"semantic.dat\" for random as #2 len = 8\nend\n";
        let parsed = crate::parse_source("jvm_semantic_open.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named("jvm_semantic_open.bcl", semantic_source)
                .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("ldc \"semantic.dat\""),
            "semantic OPEN path missing: {main}"
        );
        assert!(
            main.contains("ldc 2\n    iconst_1\n    isub"),
            "semantic OPEN channel missing: {main}"
        );
        assert!(
            main.contains("ldc 8\n    iastore"),
            "semantic OPEN record length missing: {main}"
        );
        assert!(
            !main.contains("ast.dat") && !main.contains("ldc 4\n"),
            "AST OPEN operands replaced semantic IR: {main}"
        );
    }

    #[test]
    fn jvm_generation_dispatches_close_channel_from_semantic_ir() {
        let ast_source = "open \"ast.dat\" for random as #1 len = 4\nclose #1\nend\n";
        let semantic_source = "open \"semantic.dat\" for random as #2 len = 8\nclose #2\nend\n";
        let parsed = crate::parse_source("jvm_semantic_close.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named("jvm_semantic_close.bcl", semantic_source)
                .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(main.contains("ldc 2\n    iconst_1\n    isub\n    aaload\n    invokevirtual java/io/RandomAccessFile/close ()V"), "semantic CLOSE channel missing: {main}");
        assert!(!main.contains("ldc 1\n    iconst_1\n    isub\n    aaload\n    invokevirtual java/io/RandomAccessFile/close"), "AST CLOSE channel replaced semantic IR: {main}");
    }

    #[test]
    fn jvm_generation_dispatches_typed_kill_path_from_semantic_ir() {
        let parsed = crate::parse_source(
            "jvm_semantic_kill.bcl".to_string(),
            "kill \"ast.dat\"\nend\n",
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_kill.bcl",
                "kill \"semantic.dat\"\nend\n",
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(main.contains("ldc \"semantic.dat\"\n    invokespecial java/io/File/<init> (Ljava/lang/String;)V\n    invokevirtual java/io/File/delete ()Z"), "semantic KILL path missing: {main}");
        assert!(
            !main.contains("ast.dat"),
            "AST KILL path replaced semantic IR: {main}"
        );
    }

    #[test]
    fn jvm_callable_generation_dispatches_random_file_operations_from_semantic_ir() {
        let ast_source = "procedure manage()\nopen \"ast.dat\" for random as #1 len = 4\nclose #1\nkill \"old.dat\"\nend procedure\nmanage()\nend\n";
        let parsed =
            crate::parse_source("jvm_callable_semantic_files.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(crate::semantic_ir::parse_and_adapt_named(
            "jvm_callable_semantic_files.bcl",
            "procedure manage()\nopen \"semantic.dat\" for random as #2 len = 8\nclose #2\nkill \"new.dat\"\nend procedure\nmanage()\nend\n",
        ).unwrap());
        let output = super::generate(&resolved).unwrap();
        let procedure = output
            .split(".method public static manage :")
            .nth(1)
            .unwrap();
        let procedure = procedure.split(".end method").next().unwrap();
        assert!(
            procedure.contains("ldc \"semantic.dat\""),
            "semantic callable OPEN path missing: {procedure}"
        );
        assert!(
            procedure.contains("ldc 2\n    iconst_1\n    isub"),
            "semantic callable OPEN/CLOSE channel missing: {procedure}"
        );
        assert!(
            procedure.contains("ldc 8\n    iastore"),
            "semantic callable OPEN length missing: {procedure}"
        );
        assert!(
            procedure.contains("ldc \"new.dat\""),
            "semantic callable KILL path missing: {procedure}"
        );
        assert!(
            !procedure.contains("ast.dat") && !procedure.contains("old.dat"),
            "AST callable file operation replaced semantic IR: {procedure}"
        );
    }

    #[test]
    fn jvm_generation_dispatches_typed_string_assignment_from_semantic_ir() {
        let parsed = crate::parse_source(
            "jvm_semantic_string.bcl".to_string(),
            "text$ = \"ast\"\njoined$ = \"old\"\nend\n",
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_string.bcl",
                "text$ = \"semantic\"\njoined$ = \"A\" + \"B\"\nend\n",
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("    ldc \"semantic\"\n"),
            "semantic string value missing: {main}"
        );
        assert!(
            !main.contains("    ldc \"ast\"\n"),
            "AST string value replaced semantic IR: {main}"
        );
        assert!(
            main.contains(
                "java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;"
            ),
            "semantic concatenation missing: {main}"
        );
        assert!(
            main.contains("    ldc \"A\"\n") && main.contains("    ldc \"B\"\n"),
            "semantic concat operands missing: {main}"
        );
        assert!(
            !main.contains("    ldc \"old\"\n"),
            "AST string concat value replaced semantic IR: {main}"
        );
    }

    #[test]
    fn jvm_callable_generation_dispatches_semantic_string_concatenation() {
        let ast_source = "function build$()\ntext$ = \"old\"\ntext$ = text$ + \"?\"\nreturn text$\nend function\nprint build$()\nend\n";
        let parsed =
            crate::parse_source("jvm_callable_semantic_string.bcl".to_string(), ast_source)
                .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(crate::semantic_ir::parse_and_adapt_named(
            "jvm_callable_semantic_string.bcl",
            "function build$()\ntext$ = \"semantic\"\ntext$ = text$ + \"!\"\nreturn text$\nend function\nprint build$()\nend\n",
        ).unwrap());
        let output = super::generate(&resolved).unwrap();
        let function = output
            .split(".method public static build :")
            .nth(1)
            .unwrap();
        let function = function.split(".end method").next().unwrap();
        assert!(
            function.contains("ldc \"semantic\""),
            "semantic callable string assignment missing: {function}"
        );
        assert!(
            function.contains("ldc \"!\""),
            "semantic callable concat operand missing: {function}"
        );
        assert!(
            function.contains("StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;"),
            "semantic callable StringBuilder append missing: {function}"
        );
        assert!(
            function.contains("areturn"),
            "semantic callable string return missing: {function}"
        );
        assert!(
            !function.contains("ldc \"old\""),
            "AST callable string value replaced semantic IR: {function}"
        );
        assert!(
            !function.contains("ldc \"?\""),
            "AST callable concat replaced semantic IR: {function}"
        );
    }

    #[test]
    fn jvm_generation_dispatches_semantic_mid_assignment() {
        let ast_source = "text$ = \"AST\"\ntext$ = \"A\"\nprint text$\nend\n";
        let semantic_source = "text$ = \"HELLO\"\nmid$(text$, 2, 2) = \"XY\"\nprint text$\nend\n";
        let parsed =
            crate::parse_source("jvm_semantic_mid_assign.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_mid_assign.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(main.contains("invokestatic Program/bccMidAssign (Ljava/lang/String;IIZLjava/lang/String;)Ljava/lang/String;"), "semantic MID$ assignment call missing: {main}");
        assert!(
            main.contains("ldc 2\n    ldc 2\n    iconst_0\n    ldc \"XY\""),
            "semantic MID$ start/length/value missing: {main}"
        );
        assert!(
            !main.contains("ldc \"A\""),
            "AST assignment replaced semantic MID$ node: {main}"
        );
        assert!(output.contains(".method private static bccMidAssign : (Ljava/lang/String;IIZLjava/lang/String;)Ljava/lang/String;"), "MID$ helper missing: {output}");
    }

    #[test]
    fn jvm_generation_dispatches_semantic_block_if() {
        let ast_source = "if 1 = 0 then\nvalue% = 1\nelse\nvalue% = 2\nend if\nprint value%\nend\n";
        let semantic_source =
            "if 3 = 3 then\nvalue% = 17\nelse\nvalue% = 19\nend if\nprint value%\nend\n";
        let parsed =
            crate::parse_source("jvm_semantic_block_if.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named("jvm_semantic_block_if.bcl", semantic_source)
                .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(main.contains("ldc 17"), "semantic IF body missing: {main}");
        assert!(
            main.contains("ldc 19"),
            "semantic ELSE body missing: {main}"
        );
        assert!(
            main.contains("ldc 3\n    ldc 3"),
            "semantic IF condition missing: {main}"
        );
        assert!(
            !main.contains("ldc 1\n    ldc 0"),
            "AST IF condition replaced semantic IR: {main}"
        );
    }

    #[test]
    fn jvm_semantic_block_if_emits_typed_goto_without_ast_fallback() {
        let ast_source =
            "if 1 = 0 then\ngoto target\nend if\ntarget:\nvalue% = 5\nprint value%\nend\n";
        let semantic_source =
            "if 3 = 3 then\ngoto target\nend if\ntarget:\nvalue% = 17\nprint value%\nend\n";
        let parsed =
            crate::parse_source("jvm_semantic_block_if_fallback.bcl".to_string(), ast_source)
                .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_block_if_fallback.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("goto L_user_target"),
            "typed GOTO missing: {main}"
        );
        assert!(
            main.contains("ldc 3\n    ldc 3"),
            "typed IF condition missing: {main}"
        );
        assert!(main.contains("ldc 17"), "typed target body missing: {main}");
        assert!(
            !main.contains("ldc 1\n    ldc 0"),
            "AST IF condition replaced typed IR: {main}"
        );
    }

    #[test]
    fn jvm_module_clear_is_consumed_by_typed_ir_dispatch() {
        let ast_source = "print 1\nend\n";
        let semantic_source = "clear\nend\n";
        let parsed = crate::parse_source("jvm_semantic_clear.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named("jvm_semantic_clear.bcl", semantic_source)
                .unwrap(),
        );

        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            !main.contains("ldc 1"),
            "AST PRINT was emitted instead of typed CLEAR: {main}"
        );
    }

    #[test]
    fn jvm_module_typed_erase_consumes_fixed_array_declaration() {
        let ast_source = "dim values%(2)\nprint 99\nend\n";
        let semantic_source = "dim values%(2)\nerase values%\nend\n";
        let parsed = crate::parse_source("jvm_semantic_erase.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named("jvm_semantic_erase.bcl", semantic_source)
                .unwrap(),
        );

        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            !main.contains("aconst_null\n    putstatic Program/a0 [I"),
            "typed ERASE changed fixed module storage: {main}"
        );
        assert!(
            !main.contains("ldc 99"),
            "AST PRINT replaced typed ERASE: {main}"
        );
    }

    #[test]
    fn jvm_callable_nested_typed_erase_consumes_fixed_array_declaration() {
        let ast_source = "procedure worker()\ndim values%(2)\nif true then\nprint 98\nend if\nend procedure\nworker()\nend\n";
        let semantic_source = "procedure worker()\ndim values%(2)\nif true then\nerase values%\nend if\nend procedure\nworker()\nend\n";
        let parsed = crate::parse_source(
            "jvm_callable_semantic_erase.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_callable_semantic_erase.bcl",
                semantic_source,
            )
            .unwrap(),
        );

        let output = super::generate(&resolved).unwrap();
        let worker = output
            .split(".method public static worker : ()V")
            .nth(1)
            .unwrap()
            .split(".end method")
            .next()
            .unwrap();
        assert_eq!(
            worker.matches("aconst_null").count(),
            1,
            "typed ERASE changed fixed callable-local storage: {worker}"
        );
        assert!(
            !worker.contains("ldc 98"),
            "AST PRINT replaced typed ERASE: {worker}"
        );
    }

    #[test]
    fn jvm_nested_and_callable_clear_are_consumed_by_typed_ir_dispatch() {
        let ast_source = "if 1 then\nprint 1\nend if\nprocedure worker()\nprint 2\nend procedure\nworker()\nend\n";
        let semantic_source = "if 1 then\nclear\nend if\nprocedure worker()\nclear\nend procedure\nworker()\nend\n";
        let parsed = crate::parse_source(
            "jvm_semantic_nested_clear.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_nested_clear.bcl",
                semantic_source,
            )
            .unwrap(),
        );

        let output = super::generate(&resolved).unwrap();
        assert!(
            !output.contains("getstatic java/lang/System/out Ljava/io/PrintStream;"),
            "AST statements replaced typed CLEAR in a block or callable: {output}"
        );
    }

    #[test]
    fn jvm_module_discarded_typed_expression_evaluates_and_pops_result() {
        let ast_source = "function value%()\nreturn 1\nend function\nprocedure run()\nprint 98\nend procedure\nprint 99\nrun()\nend\n";
        let semantic_source = "function value%()\nreturn 2\nend function\nprocedure run()\nvalue%() + 3\nend procedure\nvalue%() + 4\nrun()\nend\n";
        let parsed = crate::parse_source(
            "jvm_discarded_typed_expression.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_discarded_typed_expression.bcl",
                semantic_source,
            )
            .unwrap(),
        );

        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(main.contains("ldc 4"), "typed expression operand missing: {main}");
        assert!(main.contains("invokestatic Program/"), "typed call missing: {main}");
        assert!(main.contains("    pop\n"), "discarded result was not popped: {main}");
        let run = output.split(".method public static run :").nth(1).unwrap();
        let run = run.split(".end method").next().unwrap();
        assert!(run.contains("ldc 3"), "typed callable expression operand missing: {run}");
        assert!(
            output.matches("    pop\n").count() >= 2,
            "module and callable expression results must both be discarded: {output}"
        );
        assert!(
            !output.contains("getstatic java/lang/System/out Ljava/io/PrintStream;"),
            "AST PRINT replaced a typed expression: {output}"
        );
    }

    #[test]
    fn jvm_discarded_typed_double_expression_uses_pop2() {
        let ast_source = "function value#()\nreturn 1.0\nend function\nprint 99\nend\n";
        let semantic_source =
            "function value#()\nreturn 2.0\nend function\nvalue#() + 3.0\nend\n";
        let parsed = crate::parse_source(
            "jvm_discarded_typed_double_expression.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_discarded_typed_double_expression.bcl",
                semantic_source,
            )
            .unwrap(),
        );

        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(main.contains("ldc2_w 3.0"), "typed DOUBLE operand missing: {main}");
        assert!(main.contains("    pop2\n"), "DOUBLE result needs pop2: {main}");
        assert!(
            !main.contains("getstatic java/lang/System/out Ljava/io/PrintStream;"),
            "AST PRINT replaced the typed DOUBLE expression: {main}"
        );
    }

    #[test]
    fn jvm_discarded_typed_string_expression_uses_pop() {
        let ast_source =
            "function value$()\nreturn \"ast\"\nend function\nprint 99\nend\n";
        let semantic_source =
            "function value$()\nreturn \"typed\"\nend function\nvalue$() + \" suffix\"\nend\n";
        let parsed = crate::parse_source(
            "jvm_discarded_typed_string_expression.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_discarded_typed_string_expression.bcl",
                semantic_source,
            )
            .unwrap(),
        );

        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(main.contains("ldc \" suffix\""), "typed string operand missing: {main}");
        assert!(main.contains("    pop\n"), "String result needs pop: {main}");
        assert!(
            !main.contains("getstatic java/lang/System/out Ljava/io/PrintStream;"),
            "AST PRINT replaced the typed String expression: {main}"
        );
    }

    #[test]
    fn jvm_callable_generation_dispatches_semantic_block_if() {
        let ast_source = "function choose%(flag%)\nif flag% then\nchoose% = 1\nelse\nchoose% = 2\nend if\nreturn choose%\nend function\nprint choose%(1)\nend\n";
        let semantic_source = "function choose%(flag%)\nif flag% then\nchoose% = 17\nelse\nchoose% = 19\nend if\nreturn choose%\nend function\nprint choose%(1)\nend\n";
        let parsed =
            crate::parse_source("jvm_callable_semantic_block_if.bcl".to_string(), ast_source)
                .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_callable_semantic_block_if.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let function = output
            .split(".method public static choose :")
            .nth(1)
            .unwrap();
        let function = function.split(".end method").next().unwrap();
        assert!(
            function.contains("ldc 17"),
            "semantic callable IF body missing: {function}"
        );
        assert!(
            function.contains("ldc 19"),
            "semantic callable ELSE body missing: {function}"
        );
        assert!(
            !function.contains("ldc 1\n    istore"),
            "AST callable IF body replaced semantic IR: {function}"
        );
    }

    #[test]
    fn jvm_generation_dispatches_semantic_while_loop() {
        let ast_source =
            "count% = 0\nwhile count% < 1\ncount% = count% + 1\nend while\nprint count%\nend\n";
        let semantic_source =
            "count% = 0\nwhile count% < 2\ncount% = count% + 3\nend while\nprint count%\nend\n";
        let parsed = crate::parse_source("jvm_semantic_while.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named("jvm_semantic_while.bcl", semantic_source)
                .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("ldc 2"),
            "semantic WHILE bound missing: {main}"
        );
        assert!(
            main.contains("ldc 3\n    iadd"),
            "semantic WHILE body missing: {main}"
        );
        assert!(
            !main.contains("ldc 1\n    if_icmp"),
            "AST WHILE bound replaced semantic IR: {main}"
        );
    }

    #[test]
    fn jvm_callable_generation_dispatches_semantic_while_loop() {
        let ast_source = "function count%(limit%)\ncount% = 0\nwhile count% < limit%\ncount% = count% + 1\nend while\nreturn count%\nend function\nprint count%(1)\nend\n";
        let semantic_source = "function count%(limit%)\ncount% = 0\nwhile count% < limit%\ncount% = count% + 3\nend while\nreturn count%\nend function\nprint count%(2)\nend\n";
        let parsed =
            crate::parse_source("jvm_callable_semantic_while.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_callable_semantic_while.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let function = output
            .split(".method public static count :")
            .nth(1)
            .unwrap();
        let function = function.split(".end method").next().unwrap();
        assert!(
            function.contains("ldc 3\n    iadd"),
            "semantic callable WHILE body missing: {function}"
        );
        assert!(
            !function.contains("ldc 1\n    iadd"),
            "AST callable WHILE body replaced semantic IR: {function}"
        );
    }

    #[test]
    fn jvm_semantic_while_dispatches_nested_exit_and_continue() {
        let ast_source = "count% = 0\nwhile count% < 5\ncount% = count% + 1\nif count% = 2 then\ncount% = count% + 0\nend if\nif count% = 4 then\ncount% = count% + 0\nend if\nprint count%\nend while\nend\n";
        let semantic_source = "count% = 0\nwhile count% < 5\ncount% = count% + 1\nif count% = 2 then\ncontinue\nend if\nif count% = 4 then\nexit\nend if\nprint count%\nend while\nend\n";
        let parsed =
            crate::parse_source("jvm_semantic_while_transfers.bcl".to_string(), ast_source)
                .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_while_transfers.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        assert!(
            super::jvm_semantic_statements_by_source(
                resolved.semantic_module.as_ref().unwrap(),
                &resolved.program.statements,
            )
            .is_some(),
            "semantic alignment failed for AST positions: {:#?}",
            resolved
                .program
                .statements
                .iter()
                .map(|statement| (&statement.kind, &statement.pos))
                .collect::<Vec<_>>()
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("goto L_condition_"),
            "semantic CONTINUE did not branch to the WHILE header: {main}"
        );
        assert!(
            main.matches("goto L_condition_").count() >= 3,
            "semantic loop header, continue, and exit branches missing: {main}"
        );
    }

    #[test]
    fn jvm_semantic_for_uses_typed_bounds_and_body() {
        let ast_source = "for i%=1 to 1\nprint i%\nend for\nend\n";
        let semantic_source = "for i%=2 to 3\nprint i%\nend for\nend\n";
        let parsed = crate::parse_source("jvm_semantic_for.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named("jvm_semantic_for.bcl", semantic_source)
                .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("ldc 2\n    istore"),
            "typed FOR start missing: {main}"
        );
        assert!(
            main.contains("ldc 3\n    istore") && main.contains("iload 3\n    if_icmpgt"),
            "typed FOR bound missing: {main}"
        );
        assert!(
            !main.contains("getstatic Program/g1 I\n    ldc 1\n    if_icmpgt"),
            "AST FOR bound replaced typed IR: {main}"
        );

        let ast_source = "function total%(limit%)\nsum%=0\nfor i%=1 to 1\nsum%=sum%+i%\nend for\nreturn sum%\nend function\nprint total%(1)\nend\n";
        let semantic_source = "function total%(limit%)\nsum%=0\nfor i%=2 to 3\nsum%=sum%+i%\nend for\nreturn sum%\nend function\nprint total%(1)\nend\n";
        let parsed =
            crate::parse_source("jvm_callable_semantic_for.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_callable_semantic_for.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let function = output
            .split(".method public static total :")
            .nth(1)
            .unwrap();
        let function = function.split(".end method").next().unwrap();
        assert!(
            function.contains("ldc 2\n    istore")
                && function.contains("ldc 3\n    istore")
                && function.contains("if_icmpgt"),
            "typed callable FOR bounds missing: {function}"
        );
        assert!(
            !function.contains("ldc 1\n    if_icmpgt"),
            "AST callable FOR bound replaced typed IR: {function}"
        );
    }

    #[test]
    fn jvm_semantic_do_uses_typed_pre_and_post_conditions() {
        let ast_source = "count%=0\ndo while count%<1\ncount%=count%+1\nend do\ndo\ncount%=count%+1\nloop until count%>=2\nend\n";
        let semantic_source = "count%=0\ndo while count%<3\ncount%=count%+2\nend do\ndo\ncount%=count%+4\nloop until count%>=9\nend\n";
        let parsed = crate::parse_source("jvm_semantic_do.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named("jvm_semantic_do.bcl", semantic_source)
                .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("ldc 2\n    iadd"),
            "typed DO body missing: {main}"
        );
        assert!(
            main.contains("ldc 4\n    iadd"),
            "typed post-condition DO body missing: {main}"
        );
        assert!(
            main.contains("L_do_0_top:") && main.contains("L_do_1_continue:"),
            "typed DO control-flow labels missing: {main}"
        );
        assert!(
            !main.contains("ldc 1\n    iadd"),
            "AST DO bodies replaced typed IR: {main}"
        );

        let ast_source = "function count%(limit%)\ncount%=0\ndo while count%<1\ncount%=count%+1\nend do\nreturn count%\nend function\nprint count%(1)\nend\n";
        let semantic_source = "function count%(limit%)\ncount%=0\ndo while count%<5\ncount%=count%+2\nend do\nreturn count%\nend function\nprint count%(1)\nend\n";
        let parsed =
            crate::parse_source("jvm_callable_semantic_do.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_callable_semantic_do.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let function = output
            .split(".method public static count :")
            .nth(1)
            .unwrap();
        let function = function.split(".end method").next().unwrap();
        assert!(
            function.contains("ldc 5") && function.contains("ldc 2\n    iadd"),
            "typed callable DO condition and body missing: {function}"
        );
        assert!(
            !function.contains("ldc 1\n    iadd"),
            "AST callable DO body replaced typed IR: {function}"
        );
    }

    #[test]
    fn jvm_semantic_select_case_uses_typed_selector_patterns_and_bodies() {
        let ast_source = "choice%=1\nselect case choice%\ncase 1\nprint \"ast-one\"\ncase 2 to 4\nprint \"ast-range\"\ncase is > 4\nprint \"ast-large\"\ncase else\nprint \"ast-other\"\nend select\nname$=\"old\"\nselect case name$\ncase \"old\"\nprint \"ast-string\"\ncase else\nprint \"ast-string-other\"\nend select\nend\n";
        let semantic_source = "choice%=5\nselect case choice%\ncase 9\nprint \"semantic-one\"\ncase 2 to 3\nprint \"semantic-range\"\ncase is > 4\nprint \"semantic-large\"\ncase else\nprint \"semantic-other\"\nend select\nname$=\"new\"\nselect case name$\ncase \"new\"\nprint \"semantic-string\"\ncase else\nprint \"semantic-string-other\"\nend select\nend\n";
        let parsed =
            crate::parse_source("jvm_semantic_select.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named("jvm_semantic_select.bcl", semantic_source)
                .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("ldc 5"),
            "typed SELECT selector missing: {main}"
        );
        assert!(
            main.contains("ldc 9"),
            "typed SELECT value pattern missing: {main}"
        );
        assert!(
            main.contains("ldc 2") && main.contains("ldc 3"),
            "typed SELECT range missing: {main}"
        );
        assert!(
            main.contains("semantic-large") && main.contains("semantic-string"),
            "typed SELECT case bodies missing: {main}"
        );
        assert!(
            !main.contains("ast-"),
            "AST SELECT CASE replaced typed IR: {main}"
        );

        let ast_source = "function choose%(value%)\nresult%=0\nselect case value%\ncase 1\nresult%=1\ncase else\nresult%=2\nend select\nreturn result%\nend function\nprint choose%(1)\nend\n";
        let semantic_source = "function choose%(value%)\nresult%=0\nselect case value%\ncase 1\nresult%=9\ncase else\nresult%=8\nend select\nreturn result%\nend function\nprint choose%(1)\nend\n";
        let parsed =
            crate::parse_source("jvm_callable_semantic_select.bcl".to_string(), ast_source)
                .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_callable_semantic_select.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let function = output
            .split(".method public static choose :")
            .nth(1)
            .unwrap();
        let function = function.split(".end method").next().unwrap();
        assert!(
            function.contains("ldc 9\n    istore"),
            "typed callable SELECT body missing: {function}"
        );
        assert!(
            !function.contains("ldc 1\n    istore"),
            "AST callable SELECT body replaced typed IR: {function}"
        );

        let ast_source = "choice%=1\nselect case choice%\ncase 1\nprint \"ast-case\"\n' compatibility comment\ncase else\nprint \"ast-else\"\nend select\nend\n";
        let semantic_source = "choice%=5\nselect case choice%\ncase 5\nprint \"semantic-case\"\n' comment is retained in typed IR\ncase else\nprint \"semantic-else\"\nend select\nend\n";
        let parsed =
            crate::parse_source("jvm_semantic_select_fallback.bcl".to_string(), ast_source)
                .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_select_fallback.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("semantic-case"),
            "typed SELECT body was not emitted: {main}"
        );
        assert!(
            !main.contains("ast-case"),
            "AST SELECT body replaced typed IR: {main}"
        );
        assert!(
            main.contains("ldc 5\n    isub"),
            "typed SELECT selector was not emitted: {main}"
        );
    }

    #[test]
    fn jvm_semantic_blocks_emit_typed_swap_terminal_and_error_statements() {
        let ast_source = "a%=1\nb%=2\nflag%=1\nif flag% then\nswap a%,b%\ncls\nbeep\nerror 3\nend if\nif flag% then\nthrow 4\nend if\nif flag% then\nstop\nend if\nif flag% then\nsystem\nend if\nend\n";
        let semantic_source = "a%=5\nb%=6\nflag%=1\nif flag% then\nswap a%,b%\ncls\nbeep\nerror 9\nend if\nif flag% then\nthrow 8\nend if\nif flag% then\nstop\nend if\nif flag% then\nsystem\nend if\nend\n";
        let parsed =
            crate::parse_source("jvm_semantic_block_effects.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_block_effects.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("ldc 5\n    putstatic"),
            "semantic swap operands missing: {main}"
        );
        assert!(main.contains("ldc 9"), "typed nested ERROR missing: {main}");
        assert!(main.contains("ldc 8"), "typed nested THROW missing: {main}");
        assert!(
            main.contains("ldc \"\u{1b}[2J\u{1b}[H\""),
            "typed nested CLS missing: {main}"
        );
        assert!(
            main.contains("ldc \"\u{7}\""),
            "typed nested BEEP missing: {main}"
        );
        assert_eq!(
            main.matches("invokestatic java/lang/System/exit (I)V")
                .count(),
            2
        );
        assert!(
            !main.contains("ldc 3"),
            "AST error code replaced typed IR: {main}"
        );
        assert!(
            !main.contains("ldc 4"),
            "AST throw code replaced typed IR: {main}"
        );

        let ast_source = "function exchange%(left%,right%)\nif left%=1 then\nswap left%,right%\nend if\nreturn left%\nend function\nprint exchange%(1,2)\nend\n";
        let semantic_source = "function exchange%(left%,right%)\nif left%=2 then\nswap left%,right%\nend if\nreturn left%\nend function\nprint exchange%(1,2)\nend\n";
        let parsed = crate::parse_source(
            "jvm_callable_semantic_block_swap.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_callable_semantic_block_swap.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let function = output
            .split(".method public static exchange :")
            .nth(1)
            .unwrap();
        let function = function.split(".end method").next().unwrap();
        assert!(
            function.contains("ldc 2"),
            "typed callable IF condition missing: {function}"
        );
        assert!(
            function.contains("iload 0\n    iload 1\n    istore 0\n    istore 1"),
            "typed callable SWAP missing: {function}"
        );
    }

    #[test]
    fn jvm_callable_nested_return_uses_typed_expression() {
        let ast_source = "function choose%(value%)\nif value%=1 then\nreturn 2\nelse\nreturn 3\nend if\nend function\nprint choose%(1)\nend\n";
        let semantic_source = "function choose%(value%)\nif value%=4 then\nreturn 9\nelse\nreturn 8\nend if\nend function\nprint choose%(1)\nend\n";
        let parsed =
            crate::parse_source("jvm_callable_nested_return.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_callable_nested_return.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let function = output
            .split(".method public static choose :")
            .nth(1)
            .unwrap();
        let function = function.split(".end method").next().unwrap();
        assert!(
            function.contains("ldc 4"),
            "typed nested IF condition missing: {function}"
        );
        assert!(
            function.contains("ldc 9\n    ireturn"),
            "typed nested THEN return missing: {function}"
        );
        assert!(
            function.contains("ldc 8\n    ireturn"),
            "typed nested ELSE return missing: {function}"
        );
        assert!(
            !function.contains("ldc 2\n    ireturn"),
            "AST nested return replaced typed IR: {function}"
        );

        let ast_source = "procedure stopIf(flag%)\nif flag%=1 then\nreturn\nend if\nend procedure\nstopIf(1)\nend\n";
        let semantic_source = "procedure stopIf(flag%)\nif flag%=4 then\nreturn\nend if\nend procedure\nstopIf(1)\nend\n";
        let parsed = crate::parse_source(
            "jvm_callable_nested_bare_return.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_callable_nested_bare_return.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let procedure = output
            .split(".method public static stopIf :")
            .nth(1)
            .unwrap();
        let procedure = procedure.split(".end method").next().unwrap();
        assert!(
            procedure.contains("ldc 4"),
            "typed procedure IF condition missing: {procedure}"
        );
        assert!(
            procedure.contains("ifeq") && procedure.contains("    return\n"),
            "typed bare RETURN missing: {procedure}"
        );
    }

    #[test]
    fn jvm_semantic_blocks_emit_typed_locate_and_color() {
        let ast_source = "flag%=1\nif flag%=1 then\nlocate 1,2\ncolor 3,4\nend if\nend\n";
        let semantic_source = "flag%=9\nif flag%=9 then\nlocate 5,6\ncolor 7,2\nend if\nend\n";
        let parsed =
            crate::parse_source("jvm_semantic_terminal_ops.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_terminal_ops.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("ldc 5"),
            "typed nested LOCATE row missing: {main}"
        );
        assert!(
            main.contains("ldc 6"),
            "typed nested LOCATE column missing: {main}"
        );
        assert!(
            main.contains("\u{1b}[37;42m"),
            "typed nested COLOR operands missing: {main}"
        );
        assert!(
            !main.contains("\u{1b}[31;44m"),
            "AST COLOR operands replaced typed IR: {main}"
        );

        let ast_source = "procedure show(flag%)\nif flag%=1 then\nlocate 1,2\ncolor 3,4\nend if\nend procedure\nshow(1)\nend\n";
        let semantic_source = "procedure show(flag%)\nif flag%=5 then\nlocate 8,9\ncolor 10,6\nend if\nend procedure\nshow(1)\nend\n";
        let parsed = crate::parse_source(
            "jvm_callable_semantic_terminal_ops.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_callable_semantic_terminal_ops.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let procedure = output.split(".method public static show :").nth(1).unwrap();
        let procedure = procedure.split(".end method").next().unwrap();
        assert!(
            procedure.contains("ldc 8"),
            "typed callable LOCATE row missing: {procedure}"
        );
        assert!(
            procedure.contains("ldc 9"),
            "typed callable LOCATE column missing: {procedure}"
        );
        assert!(
            procedure.contains("\u{1b}[92;43m"),
            "typed callable COLOR operands missing: {procedure}"
        );
    }

    #[test]
    fn jvm_semantic_blocks_emit_typed_console_input() {
        let ast_source = "flag%=1\nif flag%=1 then\ninput \"old\"; value%\nend if\nend\n";
        let semantic_source = "flag%=4\nif flag%=4 then\ninput \"new\"; value%\nend if\nend\n";
        let parsed =
            crate::parse_source("jvm_semantic_nested_input.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_nested_input.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("ldc \"new? \""),
            "typed nested INPUT prompt missing: {main}"
        );
        assert!(
            !main.contains("ldc \"old? \""),
            "AST INPUT prompt replaced typed IR: {main}"
        );

        let ast_source = "procedure ask(flag%)\nif flag%=1 then\ninput \"old\"; answer%\nend if\nend procedure\nask(1)\nend\n";
        let semantic_source = "procedure ask(flag%)\nif flag%=5 then\ninput \"new\"; answer%\nend if\nend procedure\nask(1)\nend\n";
        let parsed =
            crate::parse_source("jvm_callable_semantic_input.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_callable_semantic_input.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let procedure = output.split(".method public static ask :").nth(1).unwrap();
        let procedure = procedure.split(".end method").next().unwrap();
        assert!(
            procedure.contains("ldc \"new? \""),
            "typed callable INPUT prompt missing: {procedure}"
        );
        assert!(
            !procedure.contains("ldc \"old? \""),
            "AST callable INPUT prompt replaced typed IR: {procedure}"
        );
    }

    #[test]
    fn jvm_nested_semantic_input_transpiles_callable_rank_two_target() {
        let ast_source = "dim values%(10, 10)\nfunction index%()\nreturn 1\nend function\nif true then\ninput \"old\"; values%(index%(), index%())\nend if\nend\n";
        let semantic_source = "dim values%(10, 10)\nfunction index%()\nreturn 2\nend function\nif true then\ninput \"new\"; values%(index%() + 2, index%())\nend if\nend\n";
        let parsed = crate::parse_source(
            "jvm_nested_semantic_input_array.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let resolved = crate::resolver::resolve_with_semantic(
            program,
            Some(
                crate::semantic_ir::parse_and_adapt_named(
                    "jvm_nested_semantic_input_array.bcl",
                    semantic_source,
                )
                .unwrap(),
            ),
        )
        .unwrap();

        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert_eq!(main.matches("invokestatic Program/index ()I").count(), 2, "both typed axes missing: {main}");
        assert!(main.contains("ldc 2\n    iadd"), "typed index addition missing: {main}");
        assert!(main.contains("iastore"), "typed array INPUT store missing: {main}");
        assert!(main.contains("ldc \"new? \""), "typed prompt missing: {main}");
        assert!(!main.contains("ldc \"old? \""), "AST prompt replaced typed IR: {main}");
    }

    #[test]
    fn jvm_nested_semantic_line_input_uses_typed_callable_index() {
        let ast_source = "open \"input.dat\" for random as #1 len = 8\ndim lines$(10)\nfunction index%()\nreturn 1\nend function\nif true then\nline input #1, lines$(index%())\nend if\nend\n";
        let semantic_source = "open \"input.dat\" for random as #2 len = 8\ndim lines$(10)\nfunction index%()\nreturn 2\nend function\nif true then\nline input #2, lines$(index%() + 2)\nend if\nend\n";
        let parsed = crate::parse_source(
            "jvm_nested_semantic_line_input.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let resolved = crate::resolver::resolve_with_semantic(
            program,
            Some(
                crate::semantic_ir::parse_and_adapt_named(
                    "jvm_nested_semantic_line_input.bcl",
                    semantic_source,
                )
                .unwrap(),
            ),
        )
        .unwrap();

        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(main.contains("invokestatic Program/index ()I\n    ldc 2\n    iadd"), "typed callable index missing: {main}");
        assert!(main.contains("ldc 2\n    iconst_1\n    isub\n    aaload\n    invokevirtual java/io/RandomAccessFile/readLine"), "typed nested channel missing: {main}");
        assert!(!main.contains("ldc 0\n    aaload\n    invokevirtual java/io/RandomAccessFile/readLine"), "AST channel replaced typed IR: {main}");
    }

    #[test]
    fn jvm_semantic_blocks_emit_typed_random_file_operations() {
        let ast_source = "flag%=1\nif flag%=1 then\nopen \"old.dat\" for random as #1 len = 4\nclose #1\nkill \"old.dat\"\nend if\nend\n";
        let semantic_source = "flag%=1\nif flag%=1 then\nopen \"nested.dat\" for random as #2 len = 8\nclose #2\nkill \"removed.dat\"\nend if\nend\n";
        let parsed =
            crate::parse_source("jvm_semantic_nested_files.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_nested_files.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("ldc \"nested.dat\""),
            "typed OPEN path missing: {main}"
        );
        assert!(
            main.contains("ldc 2\n    iconst_1\n    isub"),
            "typed file channel missing: {main}"
        );
        assert!(
            main.contains("ldc 8\n    iastore"),
            "typed record length missing: {main}"
        );
        assert!(
            main.contains("ldc \"removed.dat\""),
            "typed KILL path missing: {main}"
        );
        assert!(
            !main.contains("old.dat"),
            "AST file operations replaced typed IR: {main}"
        );

        let ast_source = "procedure manage()\nflag%=1\nif flag%=1 then\nopen \"old.dat\" for random as #1 len = 4\nclose #1\nkill \"old.dat\"\nend if\nend procedure\nmanage()\nend\n";
        let semantic_source = "procedure manage()\nflag%=1\nif flag%=1 then\nopen \"callable.dat\" for random as #3 len = 12\nclose #3\nkill \"callable_removed.dat\"\nend if\nend procedure\nmanage()\nend\n";
        let parsed = crate::parse_source(
            "jvm_callable_semantic_nested_files.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_callable_semantic_nested_files.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let procedure = output
            .split(".method public static manage :")
            .nth(1)
            .unwrap()
            .split(".end method")
            .next()
            .unwrap();
        assert!(
            procedure.contains("ldc \"callable.dat\""),
            "typed callable OPEN path missing: {procedure}"
        );
        assert!(
            procedure.contains("ldc 3\n    iconst_1\n    isub"),
            "typed callable channel missing: {procedure}"
        );
        assert!(
            procedure.contains("ldc 12\n    iastore"),
            "typed callable record length missing: {procedure}"
        );
        assert!(
            procedure.contains("ldc \"callable_removed.dat\""),
            "typed callable KILL path missing: {procedure}"
        );
        assert!(
            !procedure.contains("old.dat"),
            "AST callable file operations replaced typed IR: {procedure}"
        );
    }

    #[test]
    fn jvm_semantic_blocks_emit_typed_field_string_assignments() {
        let ast_source = "flag%=1\nfield #1, 4 as left$, 4 as right$\nif flag%=1 then\nlset left$ = \"OLD\"\nrset right$ = \"OLD\"\nend if\nend\n";
        let semantic_source = "flag%=1\nfield #1, 4 as left$, 4 as right$\nif flag%=1 then\nlset left$ = \"new\"\nrset right$ = \"Q\"\nend if\nend\n";
        let parsed =
            crate::parse_source("jvm_semantic_nested_field_sets.bcl".to_string(), ast_source)
                .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_nested_field_sets.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("ldc \"new\""),
            "typed LSET value missing: {main}"
        );
        assert!(
            main.contains("ldc \"Q\""),
            "typed RSET value missing: {main}"
        );
        assert!(
            !main.contains("ldc \"OLD\""),
            "AST LSET/RSET values replaced typed IR: {main}"
        );

        let ast_source = "field #1, 4 as left$\nprocedure update()\nflag%=1\nif flag%=1 then\nlset left$ = \"OLD\"\nend if\nend procedure\nupdate()\nend\n";
        let semantic_source = "field #1, 4 as left$\nprocedure update()\nflag%=1\nif flag%=1 then\nlset left$ = \"typed-callable\"\nend if\nend procedure\nupdate()\nend\n";
        let parsed = crate::parse_source(
            "jvm_callable_semantic_nested_field_set.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_callable_semantic_nested_field_set.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let procedure = output
            .split(".method public static update :")
            .nth(1)
            .unwrap()
            .split(".end method")
            .next()
            .unwrap();
        assert!(
            procedure.contains("ldc \"typed-callable\""),
            "typed callable LSET value missing: {procedure}"
        );
        assert!(
            !procedure.contains("ldc \"OLD\""),
            "AST callable LSET value replaced typed IR: {procedure}"
        );
    }

    #[test]
    fn jvm_semantic_blocks_emit_typed_get_put_record_positions() {
        let ast_source = "flag%=1\nopen \"records.dat\" for random as #1 len = 4\nfield #1, 4 as value$\nif flag%=1 then\nput #1, 1\nget #1, 2\nend if\nend\n";
        let semantic_source = "flag%=1\nopen \"records.dat\" for random as #1 len = 4\nfield #1, 4 as value$\nif flag%=1 then\nput #1, 3\nget #1, 4\nend if\nend\n";
        let parsed =
            crate::parse_source("jvm_semantic_nested_get_put.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_nested_get_put.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("ldc 3\n    i2l\n    lconst_1\n    lsub"),
            "typed PUT record position missing: {main}"
        );
        assert!(
            main.contains("ldc 4\n    i2l\n    lconst_1\n    lsub"),
            "typed GET record position missing: {main}"
        );
        assert_eq!(
            main.matches("invokevirtual java/io/RandomAccessFile/seek (J)V")
                .count(),
            2,
            "expected one seek for each typed GET/PUT: {main}"
        );
        assert!(
            !main.contains("ldc 1\n    i2l\n    lconst_1\n    lsub")
                && !main.contains("ldc 2\n    i2l\n    lconst_1\n    lsub"),
            "AST record positions replaced typed IR: {main}"
        );

        let ast_source = "open \"records.dat\" for random as #1 len = 4\nfield #1, 4 as value$\nprocedure update()\nflag%=1\nif flag%=1 then\nput #1, 1\nget #1, 2\nend if\nend procedure\nupdate()\nend\n";
        let semantic_source = "open \"records.dat\" for random as #1 len = 4\nfield #1, 4 as value$\nprocedure update()\nflag%=1\nif flag%=1 then\nput #1, 5\nget #1, 6\nend if\nend procedure\nupdate()\nend\n";
        let parsed = crate::parse_source(
            "jvm_callable_semantic_nested_get_put.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_callable_semantic_nested_get_put.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let procedure = output
            .split(".method public static update :")
            .nth(1)
            .unwrap()
            .split(".end method")
            .next()
            .unwrap();
        assert!(
            procedure.contains("ldc 5\n    i2l\n    lconst_1\n    lsub"),
            "typed callable PUT position missing: {procedure}"
        );
        assert!(
            procedure.contains("ldc 6\n    i2l\n    lconst_1\n    lsub"),
            "typed callable GET position missing: {procedure}"
        );
        assert!(
            !procedure.contains("ldc 1\n    i2l\n    lconst_1\n    lsub")
                && !procedure.contains("ldc 2\n    i2l\n    lconst_1\n    lsub"),
            "AST callable record positions replaced typed IR: {procedure}"
        );
    }

    #[test]
    fn jvm_module_root_emits_typed_record_file_statements() {
        let ast_source = "open \"records.dat\" for random as #1 len = 4\nfield #1, 4 as value$\nlset value$ = \"AST-left\"\nrset value$ = \"AST-right\"\nput #1, 1\nget #1, 2\nend\n";
        let semantic_source = "open \"records.dat\" for random as #1 len = 4\nfield #1, 4 as value$\nlset value$ = \"typed-left\"\nrset value$ = \"typed-right\"\nput #1, 3\nget #1, 4\nend\n";
        let parsed = crate::parse_source(
            "jvm_module_root_semantic_record_io.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_module_root_semantic_record_io.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        assert!(
            !output.contains(".field public static g1 Ljava/lang/String;"),
            "FIELD buffer was also allocated as a scalar slot: {output}"
        );
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        for typed in ["typed-left", "typed-right"] {
            assert!(
                main.contains(&format!("ldc \"{typed}\"")),
                "{typed}: {main}"
            );
        }
        assert!(
            main.contains("ldc 3\n    i2l\n    lconst_1\n    lsub"),
            "{main}"
        );
        assert!(
            main.contains("ldc 4\n    i2l\n    lconst_1\n    lsub"),
            "{main}"
        );
        for stale in ["AST-left", "AST-right"] {
            assert!(!main.contains(stale), "stale AST value {stale}: {main}");
        }
    }

    #[test]
    fn jvm_generation_dispatches_typed_seek_in_module_callable_and_blocks() {
        let ast_source = "flag%=1\nopen \"records.dat\" for random as #1 len = 4\nfield #1, 4 as value$\nseek #1, 1\nif flag%=1 then\nseek #1, 2\nend if\nend\n";
        let semantic_source = "flag%=1\nopen \"records.dat\" for random as #1 len = 4\nfield #1, 4 as value$\nseek #1, 3\nif flag%=1 then\nseek #1, 4\nend if\nend\n";
        let parsed = crate::parse_source("jvm_semantic_seek.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named("jvm_semantic_seek.bcl", semantic_source)
                .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("ldc 3\n    i2l\n    lconst_1\n    lsub"),
            "typed module SEEK position missing: {main}"
        );
        assert!(
            main.contains("ldc 4\n    i2l\n    lconst_1\n    lsub"),
            "typed nested SEEK position missing: {main}"
        );
        assert_eq!(
            main.matches("invokevirtual java/io/RandomAccessFile/seek (J)V")
                .count(),
            2,
            "module and nested SEEK should both dispatch from typed IR: {main}"
        );
        assert!(
            !main.contains("ldc 1\n    i2l\n    lconst_1\n    lsub")
                && !main.contains("ldc 2\n    i2l\n    lconst_1\n    lsub"),
            "AST SEEK positions replaced typed IR: {main}"
        );

        let ast_source = "open \"records.dat\" for random as #1 len = 4\nfield #1, 4 as value$\nprocedure update()\nflag%=1\nif flag%=1 then\nseek #1, 1\nend if\nend procedure\nupdate()\nend\n";
        let semantic_source = "open \"records.dat\" for random as #1 len = 4\nfield #1, 4 as value$\nprocedure update()\nflag%=1\nif flag%=1 then\nseek #1, 5\nend if\nend procedure\nupdate()\nend\n";
        let parsed =
            crate::parse_source("jvm_callable_semantic_seek.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_callable_semantic_seek.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let procedure = output
            .split(".method public static update :")
            .nth(1)
            .unwrap()
            .split(".end method")
            .next()
            .unwrap();
        assert!(
            procedure.contains("ldc 5\n    i2l\n    lconst_1\n    lsub"),
            "typed callable SEEK position missing: {procedure}"
        );
        assert!(
            !procedure.contains("ldc 1\n    i2l\n    lconst_1\n    lsub"),
            "AST callable SEEK position replaced typed IR: {procedure}"
        );
    }

    #[test]
    fn jvm_generation_dispatches_typed_rename_in_module_callable_and_blocks() {
        let ast_source = "name \"old-top.dat\" as \"ast-top.dat\"\nflag%=1\nif flag%=1 then\nname \"old-nested.dat\" as \"ast-nested.dat\"\nend if\nend\n";
        let semantic_source = "name \"typed-top.dat\" as \"semantic-top.dat\"\nflag%=1\nif flag%=1 then\nname \"typed-nested.dat\" as \"semantic-nested.dat\"\nend if\nend\n";
        let parsed =
            crate::parse_source("jvm_semantic_rename.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named("jvm_semantic_rename.bcl", semantic_source)
                .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        for path in [
            "typed-top.dat",
            "semantic-top.dat",
            "typed-nested.dat",
            "semantic-nested.dat",
        ] {
            assert!(
                main.contains(path),
                "typed module rename path {path} missing: {main}"
            );
        }
        for path in [
            "old-top.dat",
            "ast-top.dat",
            "old-nested.dat",
            "ast-nested.dat",
        ] {
            assert!(
                !main.contains(path),
                "AST rename path {path} replaced typed IR: {main}"
            );
        }
        assert_eq!(
            main.matches("invokevirtual java/io/File/renameTo (Ljava/io/File;)Z")
                .count(),
            2,
            "expected typed top-level and nested renames: {main}"
        );

        let ast_source = "procedure renameFile()\nname \"old.dat\" as \"old-target.dat\"\nend procedure\nrenameFile()\nend\n";
        let semantic_source = "procedure renameFile()\nname \"typed.dat\" as \"typed-target.dat\"\nend procedure\nrenameFile()\nend\n";
        let parsed =
            crate::parse_source("jvm_callable_semantic_rename.bcl".to_string(), ast_source)
                .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_callable_semantic_rename.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let procedure = output
            .split(".method public static renameFile :")
            .nth(1)
            .unwrap()
            .split(".end method")
            .next()
            .unwrap();
        assert!(
            procedure.contains("typed.dat") && procedure.contains("typed-target.dat"),
            "typed callable rename paths missing: {procedure}"
        );
        assert!(
            !procedure.contains("old.dat"),
            "AST callable rename paths replaced typed IR: {procedure}"
        );
    }

    #[test]
    fn jvm_field_layout_scan_uses_semantic_ir_when_available() {
        let program = crate::parse_source(
            "jvm_field_layout_authority.bcl".to_string(),
            "open \"ast.dat\" for random as #1 len = 8\nfield #1, 3 as ast$\nend\n",
        )
        .unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "jvm_field_layout_authority.bcl",
            "open \"typed.dat\" for random as #2 len = 8\nfield #2, 5 as typed$\nend\n",
        )
        .unwrap();

        let ast_fields = super::collect_jvm_field_vars(&program, None).unwrap();
        assert_eq!(ast_fields.get("ast$").map(|field| field.width), Some(3));
        assert!(!ast_fields.contains_key("typed$"));

        let semantic_fields = super::collect_jvm_field_vars(&program, Some(&semantic)).unwrap();
        assert_eq!(
            semantic_fields
                .get("typed$")
                .map(|field| (field.channel, field.width)),
            Some((2, 5))
        );
        assert!(
            !semantic_fields.contains_key("ast$"),
            "typed JVM FIELD layout retained a compatibility AST binding"
        );
    }

    #[test]
    fn jvm_scalar_declarations_use_typed_record_buffer_metadata() {
        let mut semantic = crate::semantic_ir::parse_and_adapt_named(
            "jvm_record_buffer_declarations.bcl",
            "end\n",
        )
        .unwrap();
        semantic.lowered_record_files = vec![crate::semantic_ir::LoweredRecordFile {
            name: "inventory".to_string(),
            channel: 1,
            record_type: "Entry".to_string(),
            record_length: 9,
            owner: None,
            record_locals: vec![
                crate::semantic_ir::LoweredRecordLocal {
                    name: "rowid".to_string(),
                    value_type: crate::semantic_ir::SemanticValueType::Integer,
                    owner: None,
                },
                crate::semantic_ir::LoweredRecordLocal {
                    name: "rowname".to_string(),
                    value_type: crate::semantic_ir::SemanticValueType::String,
                    owner: None,
                },
            ],
            fields: vec![
                crate::semantic_ir::LoweredRecordField {
                    buffer_name: "stringBuffer".to_string(),
                    width: 4,
                    offset: 0,
                    kind: crate::semantic_ir::LoweredRecordFieldKind::String {
                        right_aligned: false,
                    },
                },
                crate::semantic_ir::LoweredRecordField {
                    buffer_name: "int16Buffer".to_string(),
                    width: 2,
                    offset: 4,
                    kind: crate::semantic_ir::LoweredRecordFieldKind::Int16,
                },
                crate::semantic_ir::LoweredRecordField {
                    buffer_name: "typedBuffer".to_string(),
                    width: 4,
                    offset: 6,
                    kind: crate::semantic_ir::LoweredRecordFieldKind::Int32,
                },
                crate::semantic_ir::LoweredRecordField {
                    buffer_name: "float32Buffer".to_string(),
                    width: 4,
                    offset: 10,
                    kind: crate::semantic_ir::LoweredRecordFieldKind::Float32,
                },
                crate::semantic_ir::LoweredRecordField {
                    buffer_name: "float64Buffer".to_string(),
                    width: 8,
                    offset: 14,
                    kind: crate::semantic_ir::LoweredRecordFieldKind::Float64,
                },
            ],
        }];
        let mut declarations = std::collections::BTreeMap::new();

        super::collect_semantic_scalar_declarations(&semantic, &mut declarations);

        assert_eq!(
            declarations.get("stringbuffer"),
            Some(&super::JvmType::String)
        );
        assert_eq!(
            declarations.get("int16buffer"),
            Some(&super::JvmType::Numeric(super::NumericType::Int))
        );
        assert_eq!(
            declarations.get("typedbuffer"),
            Some(&super::JvmType::Numeric(super::NumericType::Long))
        );
        assert_eq!(
            declarations.get("float32buffer"),
            Some(&super::JvmType::Numeric(super::NumericType::Double))
        );
        assert_eq!(
            declarations.get("float64buffer"),
            Some(&super::JvmType::Numeric(super::NumericType::Double))
        );
        assert_eq!(
            declarations.get("rowid%"),
            Some(&super::JvmType::Numeric(super::NumericType::Int))
        );
        assert_eq!(declarations.get("rowname$"), Some(&super::JvmType::String));
    }

    #[test]
    fn jvm_scalar_declarations_derive_standalone_record_storage_from_typed_ir() {
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "jvm_standalone_record_declarations.bcl",
            "record Core\nid: int16\nend record\nrecord Identity combines Core\nkey: int32\nend record\nrecord Room combines Identity\nname: string(20)\nnorth: int16\nend record\ndim room as Room\nend\n",
        )
        .unwrap();
        let mut declarations = std::collections::BTreeMap::new();

        super::collect_semantic_scalar_declarations(&semantic, &mut declarations);

        assert_eq!(
            declarations.get("roomid%"),
            Some(&super::JvmType::Numeric(super::NumericType::Int))
        );
        assert_eq!(declarations.get("roomname$"), Some(&super::JvmType::String));
        assert_eq!(
            declarations.get("roomnorth%"),
            Some(&super::JvmType::Numeric(super::NumericType::Int))
        );
        assert_eq!(
            declarations.get("roomkey&"),
            Some(&super::JvmType::Numeric(super::NumericType::Long))
        );
    }

    #[test]
    fn jvm_record_file_layout_uses_typed_file_metadata() {
        let program = crate::parse_source(
            "jvm_record_file_layout_authority.bcl".to_string(),
            "open \"ast.dat\" for random as #1 len = 8\nfield #1, 9 as stale$\nend\n",
        )
        .unwrap();
        let mut semantic = crate::semantic_ir::parse_and_adapt_named(
            "jvm_record_file_layout_authority.bcl",
            "record Entry\nname: string(4)\nscore: integer\nend record\nfile inventory as Entry = open(\"typed.dat\")\nend\n",
        )
        .unwrap();
        semantic.lowered_record_files = vec![crate::semantic_ir::LoweredRecordFile {
            name: "inventory".to_string(),
            channel: 2,
            record_type: "Entry".to_string(),
            record_length: 6,
            owner: None,
            record_locals: Vec::new(),
            fields: vec![
                crate::semantic_ir::LoweredRecordField {
                    buffer_name: "nameBuf$".to_string(),
                    width: 4,
                    offset: 0,
                    kind: crate::semantic_ir::LoweredRecordFieldKind::String {
                        right_aligned: false,
                    },
                },
                crate::semantic_ir::LoweredRecordField {
                    buffer_name: "scoreBuf%".to_string(),
                    width: 2,
                    offset: 4,
                    kind: crate::semantic_ir::LoweredRecordFieldKind::Int16,
                },
            ],
        }];

        let fields = super::collect_jvm_field_vars(&program, Some(&semantic)).unwrap();
        assert_eq!(
            fields
                .get("namebuf$")
                .map(|field| (field.channel, field.offset, field.width)),
            Some((2, 0, 4))
        );
        assert_eq!(
            fields
                .get("scorebuf%")
                .map(|field| (field.channel, field.offset, field.width)),
            Some((2, 4, 2))
        );
        assert!(
            !fields.contains_key("stale$"),
            "typed file layout retained a compatibility AST FIELD binding"
        );
    }

    #[test]
    fn jvm_semantic_blocks_consume_typed_field_layout_declarations() {
        let ast_source = "flag%=1\nopen \"record.dat\" for random as #1 len = 8\nif flag%=1 then\nfield #1, 3 as ast$\nlset ast$ = \"AST\"\nend if\nend\n";
        let semantic_source = "flag%=1\nopen \"record.dat\" for random as #1 len = 8\nif flag%=1 then\nfield #1, 5 as typed$\nlset typed$ = \"typed\"\nend if\nend\n";
        let parsed = crate::parse_source(
            "jvm_semantic_nested_field_layout.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_nested_field_layout.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("ldc \"typed\""),
            "nested semantic FIELD layout was not used by LSET: {main}"
        );
        assert!(
            main.contains(
                "ldc 5\n    invokevirtual java/lang/String/substring (II)Ljava/lang/String;"
            ),
            "typed FIELD width was not used by LSET: {main}"
        );
        assert!(
            !main.contains("ldc \"AST\""),
            "AST FIELD/LSET replaced typed nested statements: {main}"
        );

        let ast_source = "open \"record.dat\" for random as #1 len = 8\nprocedure writeField()\nflag%=1\nif flag%=1 then\nfield #1, 3 as ast$\nlset ast$ = \"AST\"\nend if\nend procedure\nwriteField()\nend\n";
        let semantic_source = "open \"record.dat\" for random as #1 len = 8\nprocedure writeField()\nflag%=1\nif flag%=1 then\nfield #1, 5 as typed$\nlset typed$ = \"typed-callable\"\nend if\nend procedure\nwriteField()\nend\n";
        let parsed = crate::parse_source(
            "jvm_callable_semantic_nested_field_layout.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_callable_semantic_nested_field_layout.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let procedure = output
            .split(".method public static writeField :")
            .nth(1)
            .unwrap()
            .split(".end method")
            .next()
            .unwrap();
        assert!(
            procedure.contains("ldc \"typed-callable\""),
            "nested callable semantic FIELD layout missing: {procedure}"
        );
        assert!(
            !procedure.contains("ldc \"AST\""),
            "AST callable FIELD/LSET replaced typed nested statements: {procedure}"
        );
    }

    #[test]
    fn jvm_semantic_blocks_emit_typed_file_line_input() {
        let ast_source = "flag%=1\nchannel%=1\nopen \"input.dat\" for random as #1 len = 8\ndim lines$(3)\nif flag%=1 then\nline input #1, lines$(0)\nend if\nend\n";
        let semantic_source = "flag%=1\nchannel%=2\nopen \"input.dat\" for random as #1 len = 8\ndim lines$(3)\nif flag%=1 then\nline input #channel%, lines$(2)\nend if\nend\n";
        let parsed =
            crate::parse_source("jvm_semantic_nested_line_input.bcl".to_string(), ast_source)
                .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_nested_line_input.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("getstatic Program/bccFiles [Ljava/io/RandomAccessFile;\n    getstatic Program/g1 I\n    iconst_1\n    isub\n    aaload\n    invokevirtual java/io/RandomAccessFile/readLine ()Ljava/lang/String;"),
            "typed channel LINE INPUT missing: {main}"
        );
        assert!(
            main.contains("getstatic Program/a0 [Ljava/lang/String;\n    ldc 2"),
            "typed indexed LINE INPUT target missing: {main}"
        );
        assert!(
            !main.contains("getstatic Program/bccFiles [Ljava/io/RandomAccessFile;\n    ldc 0\n    aaload\n    invokevirtual java/io/RandomAccessFile/readLine"),
            "AST LINE INPUT channel replaced typed IR: {main}"
        );

        let ast_source = "procedure readLineValue()\ndim lines$(3)\nline input #1, lines$(0)\nend procedure\nreadLineValue()\nend\n";
        let semantic_source = "procedure readLineValue()\ndim lines$(3)\nline input #2, lines$(2)\nend procedure\nreadLineValue()\nend\n";
        let parsed = crate::parse_source(
            "jvm_callable_semantic_line_input.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_callable_semantic_line_input.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let procedure = output
            .split(".method public static readLineValue :")
            .nth(1)
            .unwrap()
            .split(".end method")
            .next()
            .unwrap();
        assert!(
            procedure.contains("getstatic Program/bccFiles [Ljava/io/RandomAccessFile;\n    ldc 2\n    iconst_1\n    isub\n    aaload\n    invokevirtual java/io/RandomAccessFile/readLine ()Ljava/lang/String;"),
            "typed callable LINE INPUT missing: {procedure}"
        );
    }

    #[test]
    fn jvm_semantic_line_input_transpiles_typed_callable_array_index() {
        let ast_source = "dim lines$(10)\nfunction index%()\nreturn 1\nend function\nline input #1, lines$(index%())\nend\n";
        let semantic_source = "dim lines$(10)\nfunction index%()\nreturn 2\nend function\nline input #2, lines$(index%() + 2)\nend\n";
        let parsed = crate::parse_source(
            "jvm_semantic_callable_line_input_index.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_callable_line_input_index.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output
            .split(".method public static main :")
            .nth(1)
            .unwrap()
            .split(".end method")
            .next()
            .unwrap();
        assert!(
            main.contains("invokestatic Program/index ()I\n    ldc 2\n    iadd\n    getstatic Program/bccFiles [Ljava/io/RandomAccessFile;\n    ldc 2"),
            "typed callable array index or channel missing: {main}"
        );
        assert!(
            !main.contains("iconst_0\n    aaload\n    invokevirtual java/io/RandomAccessFile/readLine"),
            "AST channel replaced typed IR: {main}"
        );
        assert!(
            main.contains("invokestatic Program/index ()I\n    ldc 2\n    iadd"),
            "typed callable index expression replaced by the AST: {main}"
        );
    }

    #[test]
    fn jvm_semantic_input_transpiles_typed_callable_array_index() {
        let ast_source = "dim values%(10, 10)\nfunction index%()\nreturn 1\nend function\ninput \"old\"; values%(index%(), index%())\nend\n";
        let semantic_source = "dim values%(10, 10)\nfunction index%()\nreturn 2\nend function\ninput \"new\"; values%(index%(), index%())\nend\n";
        let parsed = crate::parse_source(
            "jvm_semantic_callable_input_index.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_callable_input_index.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output
            .split(".method public static main :")
            .nth(1)
            .unwrap()
            .split(".end method")
            .next()
            .unwrap();
        assert!(
            main.matches("invokestatic Program/index ()I").count() == 2 && main.contains("iastore"),
            "typed callable array INPUT target missing: {main}"
        );
        assert!(
            main.contains("ldc \"new? \"") && !main.contains("ldc \"old? \""),
            "AST prompt replaced typed INPUT prompt: {main}"
        );
    }

    #[test]
    fn jvm_generation_emits_typed_channel_print_in_module_callable_and_blocks() {
        let ast_source = "flag%=1\nvalue%=1\nchannel%=1\nopen \"print.dat\" for random as #1 len = 8\nprint #1, \"old\";value%\nif flag%=1 then\nprint #1, \"old-nested\"\nend if\nend\n";
        let semantic_source = "flag%=1\nvalue%=7\nchannel%=1\nopen \"print.dat\" for random as #1 len = 8\nprint #channel%, \"typed\";value%\nif flag%=1 then\nprint #channel%, \"typed-nested\"\nend if\nend\n";
        let parsed =
            crate::parse_source("jvm_semantic_file_print.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_file_print.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("getstatic Program/bccFiles [Ljava/io/RandomAccessFile;\n    getstatic Program/g1 I\n    iconst_1\n    isub\n    aaload\n    dup\n    ldc \"typed\"\n    invokevirtual java/io/RandomAccessFile/writeBytes"),
            "typed channel PRINT missing: {main}"
        );
        assert_eq!(
            main.matches("getstatic Program/bccFiles [Ljava/io/RandomAccessFile;")
                .count(),
            3,
            "each typed channel PRINT should load its selected file once: {main}"
        );
        assert!(
            main.contains("ldc \"typed-nested\""),
            "typed nested channel PRINT missing: {main}"
        );
        assert!(
            main.contains("invokestatic java/lang/Integer/toString (I)Ljava/lang/String;"),
            "typed numeric channel PRINT missing: {main}"
        );
        assert!(
            !main.contains("ldc \"old\"") && !main.contains("ldc \"old-nested\""),
            "AST channel PRINT tokens replaced typed IR: {main}"
        );

        let ast_source = "open \"print.dat\" for random as #1 len = 8\nprocedure printValue()\nprint #1, \"old\"\nend procedure\nprintValue()\nend\n";
        let semantic_source = "open \"print.dat\" for random as #1 len = 8\nprocedure printValue()\nprint #2, \"typed-callable\"\nend procedure\nprintValue()\nend\n";
        let parsed = crate::parse_source(
            "jvm_callable_semantic_file_print.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_callable_semantic_file_print.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let procedure = output
            .split(".method public static printValue :")
            .nth(1)
            .unwrap()
            .split(".end method")
            .next()
            .unwrap();
        assert!(
            procedure.contains("typed-callable"),
            "typed callable channel PRINT missing: {procedure}"
        );
        assert!(
            !procedure.contains("old\""),
            "AST callable channel PRINT replaced typed IR: {procedure}"
        );
    }

    #[test]
    fn jvm_generation_emits_typed_nested_write_as_csv_to_channel() {
        let ast_source = "value%=1\nopen \"write.dat\" for random as #1 len = 8\nif value%=1 then\nwrite #1, \"old\", value%\nend if\nend\n";
        let semantic_source = "value%=7\nopen \"write.dat\" for random as #1 len = 8\nif value%=7 then\nwrite #1, \"a,b\", value%\nend if\nend\n";
        let parsed = crate::parse_source("jvm_typed_write.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named("jvm_typed_write.bcl", semantic_source)
                .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("ldc \"a,b\""),
            "typed string field missing: {main}"
        );
        assert!(
            main.contains("invokevirtual java/lang/String/replace (Ljava/lang/CharSequence;Ljava/lang/CharSequence;)Ljava/lang/String;"),
            "CSV quote escaping missing: {main}"
        );
        assert!(
            main.contains("invokestatic java/lang/Integer/toString (I)Ljava/lang/String;"),
            "typed numeric field missing: {main}"
        );
        assert!(!main.contains("old"), "AST WRITE values replaced typed IR: {main}");
    }

    #[test]
    fn jvm_generation_emits_typed_callable_output_and_append_opens() {
        let ast_source = "procedure writeFiles()\nopen \"ast-output.dat\" for output as #1\nopen \"ast-append.dat\" for append as #2\nend procedure\nwriteFiles()\nend\n";
        let semantic_source = "procedure writeFiles()\nopen \"typed-output.dat\" for output as #3\nopen \"typed-append.dat\" for append as #4\nend procedure\nwriteFiles()\nend\n";
        let parsed = crate::parse_source("jvm_typed_output_open.bcl".to_string(), ast_source)
            .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_typed_output_open.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let body = output.split(".method public static writeFiles :").nth(1).unwrap();
        let body = body.split(".end method").next().unwrap();
        assert!(
            body.contains("ldc \"typed-output.dat\""),
            "typed OUTPUT path missing: {body}"
        );
        assert!(
            body.contains("ldc \"typed-append.dat\""),
            "typed APPEND path missing: {body}"
        );
        assert!(
            body.contains("RandomAccessFile/setLength (J)V"),
            "OUTPUT truncation missing: {body}"
        );
        assert!(
            body.contains("RandomAccessFile/seek (J)V"),
            "APPEND seek missing: {body}"
        );
        assert!(
            !body.contains("ast-output.dat") && !body.contains("ast-append.dat"),
            "AST paths leaked: {body}"
        );
    }

    #[test]
    fn jvm_generation_uses_typed_data_pool_and_read_target() {
        let parsed = crate::parse_source("jvm_typed_read.bcl".to_string(), "data 1\nread first%\nend\n").unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_typed_read.bcl",
                "data 73\nread resolved%\nend\n",
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(main.contains("ldc \"73\""), "typed DATA item missing: {main}");
        assert!(
            main.contains("invokestatic java/lang/Integer/parseInt (Ljava/lang/String;)I"),
            "typed integer READ conversion missing: {main}"
        );
        assert!(!main.contains("ldc \"1\""), "AST DATA item leaked: {main}");
        assert!(!output.contains("first"), "AST READ target leaked: {output}");
    }

    #[test]
    fn jvm_callable_data_dispatch_uses_typed_pool_without_ast_fallback() {
        let filename = "jvm_typed_callable_data.bcl";
        let ast_source = "procedure loadData()\ndata 1\nend procedure\nloadData()\nend\n";
        let semantic_source = "procedure loadData()\ndata 73\nend procedure\nloadData()\nend\n";
        let parsed = crate::parse_source(filename.to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(filename, semantic_source)
            .expect("typed callable DATA parses");
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic))
            .expect("typed callable DATA resolves");

        let output = super::generate(&resolved).expect("typed callable DATA emits");
        assert!(
            output.contains("ldc \"73\""),
            "typed DATA item missing: {output}"
        );
        assert!(
            !output.contains("ldc \"1\""),
            "AST DATA item leaked: {output}"
        );
    }

    #[test]
    fn jvm_generation_uses_typed_on_goto_selector_and_targets() {
        let ast_source = "on 1 goto first, second\nfirst:\nend\nsecond:\nend\n";
        let semantic_source = "on 2 goto alternate, selected\nalternate:\nend\nselected:\nend\n";
        let parsed = crate::parse_source("jvm_typed_on_goto.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_typed_on_goto.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(main.contains("    ldc 2\n"), "typed branch selector missing: {main}");
        assert!(main.contains("goto L_user_selected"), "typed branch target missing: {main}");
        assert!(!main.contains("L_user_first"), "AST branch target leaked: {main}");
    }

    #[test]
    fn jvm_generation_treats_restore_without_data_as_noop() {
        let parsed = crate::parse_source(
            "jvm_restore_without_data.bcl".to_string(),
            "restore\nend\n",
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_restore_without_data.bcl",
                "restore\nend\n",
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        assert!(!output.contains("bccDataPtr"), "no-op RESTORE allocated DATA state: {output}");
    }

    #[test]
    fn jvm_generation_assigns_unique_internal_labels_to_on_goto_statements() {
        let source = "on 0 goto first\non 2 goto second\nend\nfirst:\nend\nsecond:\nend\n";
        let parsed = crate::parse_source("jvm_repeated_on_goto.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named("jvm_repeated_on_goto.bcl", source).unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(main.contains("L_on_goto_0_done"), "first ON GOTO labels missing: {main}");
        assert!(main.contains("L_on_goto_1_done"), "second ON GOTO labels missing: {main}");
        assert!(main.contains("goto L_user_second"), "second ON GOTO branch missing: {main}");
    }

    #[test]
    fn jvm_generation_does_not_emit_read_helper_for_data_only_module() {
        let parsed = crate::parse_source("jvm_data_only.bcl".to_string(), "data 5\nend\n").unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named("jvm_data_only.bcl", "data 5\nend\n")
                .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        assert!(output.contains("bccData [Ljava/lang/String;"));
        assert!(!output.contains("bccReadData"), "unused DATA reader helper emitted: {output}");
    }

    #[test]
    fn jvm_generation_emits_typed_string_function_calls() {
        let ast_source = "function message$()\nreturn \"AST\"\nend function\nopen \"message.dat\" for random as #1 len = 8\nprint #1, \"old\"\nend\n";
        let semantic_source = "function message$()\nreturn \"typed\"\nend function\nopen \"message.dat\" for random as #1 len = 8\nprint #1, message$()\nend\n";
        let parsed = crate::parse_source(
            "jvm_semantic_string_function_call.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_string_function_call.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("invokestatic Program/message ()Ljava/lang/String;"),
            "typed string function call missing from channel PRINT: {main}"
        );
        let function = output
            .split(".method public static message :")
            .nth(1)
            .unwrap();
        let function = function.split(".end method").next().unwrap();
        assert!(
            function.contains("ldc \"typed\""),
            "typed callable return missing: {function}"
        );
        assert!(
            !function.contains("ldc \"AST\""),
            "AST callable return replaced typed IR: {function}"
        );
    }

    #[test]
    fn jvm_generation_coerces_typed_numeric_scalar_method_arguments() {
        let ast_source = "method adjust#[double](step#)\nreturn self#+step#\nend method\nvalue#=2.0\nresult#=value#.adjust(3)\nprint result#\nend\n";
        let semantic_source = "method adjust#[double](step#)\nreturn self#+step#\nend method\nvalue#=4.0\nresult#=value#.adjust(5)\nprint result#\nend\n";
        let parsed = crate::parse_source(
            "jvm_semantic_numeric_method_coercion.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_numeric_method_coercion.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("ldc 5\n    i2d\n    invokestatic Program/adjust (DD)D"),
            "typed integer actual must widen before calling the double method: {main}"
        );
        assert!(
            !main.contains("ldc 2.0"),
            "AST numeric receiver replaced typed IR: {main}"
        );
        let method = output
            .split(".method public static adjust :")
            .nth(1)
            .unwrap()
            .split(".end method")
            .next()
            .unwrap();
        assert!(
            method.contains("dload 0") && method.contains("dload 2") && method.contains("dadd"),
            "wide typed receiver and parameter slots are incorrect: {method}"
        );
    }

    #[test]
    fn jvm_generation_uses_semantic_module_header_for_class_identity() {
        let parsed = crate::parse_source(
            "jvm_semantic_module_header.bcl".to_string(),
            "program AstProgram\nend\n",
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_module_header.bcl",
                "program TypedProgram\nend\n",
            )
            .unwrap(),
        );

        let output = super::generate(&resolved).unwrap();
        assert!(
            output.contains(".class public TypedProgram\n"),
            "JVM class identity should come from the semantic module header: {output}"
        );
        assert!(
            !output.contains(".class public AstProgram\n"),
            "compatibility AST module name replaced typed IR"
        );
    }

    #[test]
    fn jvm_generation_emits_typed_scalar_method_calls() {
        let ast_source = "method decorate$[string]()\nreturn self$ + \"ast-method\"\nend method\nword$=\"ast-base\"\nresult$=word$.decorate()\nprint result$\nend\n";
        let semantic_source = "method decorate$[string]()\nreturn self$ + \"typed-method\"\nend method\nword$=\"typed-base\"\nresult$=word$.decorate()\nprint result$\nend\n";
        let parsed = crate::parse_source(
            "jvm_semantic_scalar_method_call.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_scalar_method_call.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("getstatic Program/g")
                && main.contains(
                    "invokestatic Program/decorate (Ljava/lang/String;)Ljava/lang/String;"
                ),
            "typed scalar method receiver must replace the AST receiver: {main}"
        );
        assert!(
            !main.contains("ldc \"ast-base\""),
            "AST scalar method receiver replaced typed IR: {main}"
        );
        let method = output
            .split(".method public static decorate :")
            .nth(1)
            .unwrap()
            .split(".end method")
            .next()
            .unwrap();
        assert!(
            method.contains("typed-method") && !method.contains("ast-method"),
            "typed scalar method body missing or AST body used: {method}"
        );
    }

    #[test]
    fn jvm_generation_emits_typed_numeric_scalar_method_calls() {
        let ast_source = "method adjust%[integer]()\nreturn self%+1\nend method\nvalue%=10\nresult%=value%.adjust()\nprint result%\nend\n";
        let semantic_source = "method adjust%[integer]()\nreturn self%+7\nend method\nvalue%=20\nresult%=value%.adjust()\nprint result%\nend\n";
        let parsed = crate::parse_source(
            "jvm_semantic_numeric_scalar_method_call.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_numeric_scalar_method_call.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("getstatic Program/g2 I\n    invokestatic Program/adjust (I)I"),
            "typed numeric scalar method receiver must replace the AST receiver: {main}"
        );
        assert!(
            !main.contains("ldc 10"),
            "AST numeric scalar method receiver replaced typed IR: {main}"
        );
        let method = output
            .split(".method public static adjust :")
            .nth(1)
            .unwrap()
            .split(".end method")
            .next()
            .unwrap();
        assert!(
            method.contains("ldc 7\n    iadd") && !method.contains("ldc 1\n    iadd"),
            "typed numeric scalar method body missing or AST body used: {method}"
        );
    }

    #[test]
    fn jvm_generation_emits_typed_scalar_method_arguments() {
        let ast_source = "method adjust%[integer](delta%)\nreturn self%+delta%\nend method\nvalue%=10\nresult%=value%.adjust(1)\nprint result%\nend\n";
        let semantic_source = "method adjust%[integer](delta%)\nreturn self%+delta%\nend method\nvalue%=20\nresult%=value%.adjust(7)\nprint result%\nend\n";
        let parsed = crate::parse_source(
            "jvm_semantic_scalar_method_argument.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_scalar_method_argument.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains(
                "getstatic Program/g2 I\n    ldc 7\n    invokestatic Program/adjust (II)I"
            ),
            "typed JVM scalar method receiver and argument must follow the receiver-first ABI: {main}"
        );
        assert!(
            !main.contains("ldc 1"),
            "AST scalar method argument replaced typed IR: {main}"
        );
        let method = output
            .split(".method public static adjust :")
            .nth(1)
            .unwrap()
            .split(".end method")
            .next()
            .unwrap();
        assert!(
            method.contains("iload 0") && method.contains("iload 1") && method.contains("iadd"),
            "typed receiver and parameter bindings missing from JVM method body: {method}"
        );
    }

    #[test]
    fn jvm_generation_emits_typed_string_scalar_method_arguments() {
        let ast_source = "method join$[string](ending$)\nreturn self$+\"ast-body:\"+ending$\nend method\nbase$=\"ast-base\"\nresult$=base$.join(\"ast-arg\")\nprint result$\nend\n";
        let semantic_source = "method join$[string](ending$)\nreturn self$+\"typed-body:\"+ending$\nend method\nbase$=\"typed-base\"\nresult$=base$.join(\"typed-arg\")\nprint result$\nend\n";
        let parsed = crate::parse_source(
            "jvm_semantic_string_scalar_method_argument.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_string_scalar_method_argument.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("getstatic Program/g1 Ljava/lang/String;\n    ldc \"typed-arg\"\n    invokestatic Program/join (Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;"),
            "typed string receiver and argument must follow the receiver-first JVM descriptor: {main}"
        );
        assert!(
            !main.contains("ast-base") && !main.contains("ast-arg"),
            "AST string method operands replaced typed IR: {main}"
        );
        let method = output
            .split(".method public static join :")
            .nth(1)
            .unwrap()
            .split(".end method")
            .next()
            .unwrap();
        assert!(
            method.contains("typed-body:") && !method.contains("ast-body:"),
            "typed string method body missing or AST body used: {method}"
        );
    }

    #[test]
    fn jvm_generation_emits_typed_scalar_method_byref_arguments() {
        let ast_source = "method adjust%[integer](byref delta%)\ndelta%=delta%+1\nreturn self%+delta%\nend method\noriginal%=1\ntyped%=2\nvalue%=10\nresult%=value%.adjust(original%)\nprint result%;\",\";original%;\",\";typed%\nend\n";
        let semantic_source = "method adjust%[integer](byref delta%)\ndelta%=delta%+9\nreturn self%+delta%\nend method\noriginal%=1\ntyped%=2\nvalue%=20\nresult%=value%.adjust(typed%)\nprint result%;\",\";original%;\",\";typed%\nend\n";
        let parsed = crate::parse_source(
            "jvm_semantic_scalar_method_byref_argument.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_scalar_method_byref_argument.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("invokestatic Program/adjust (I[I)I"),
            "typed scalar ByRef method call must use an array descriptor after its receiver: {main}"
        );
        assert!(
            !main.contains("ldc 10"),
            "AST method receiver replaced typed IR: {main}"
        );
        assert!(
            main.contains("getstatic Program/g2 I"),
            "typed ByRef actual wasn't loaded for the call/writeback: {main}"
        );
        let method = output
            .split(".method public static adjust :")
            .nth(1)
            .unwrap()
            .split(".end method")
            .next()
            .unwrap();
        assert!(
            method.contains("ldc 9\n    iadd") && !method.contains("ldc 1\n    iadd"),
            "typed scalar method body missing or AST body used: {method}"
        );
    }

    #[test]
    fn jvm_generation_emits_typed_scalar_method_array_arguments() {
        let ast_source = "method sum%[integer](byval values%(?))\nreturn self%+values%(0)\nend method\ndim original%(1)\ndim typed%(1)\noriginal%(0)=1\ntyped%(0)=9\nbase%=10\nresult%=base%.sum(original%)\nprint result%\nend\n";
        let semantic_source = "method sum%[integer](byval values%(?))\nreturn self%+values%(0)\nend method\ndim original%(1)\ndim typed%(1)\noriginal%(0)=1\ntyped%(0)=9\nbase%=20\nresult%=base%.sum(typed%)\nprint result%\nend\n";
        let parsed = crate::parse_source(
            "jvm_semantic_scalar_method_array_argument.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_scalar_method_array_argument.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("getstatic Program/a1 [I")
                && main.contains("invokestatic Program/sum (I[I)I"),
            "typed scalar method array argument and receiver-first descriptor missing: {main}"
        );
        let method = output
            .split(".method public static sum :")
            .nth(1)
            .unwrap()
            .split(".end method")
            .next()
            .unwrap();
        assert!(
            method.contains("iload 0")
                && method.contains("aload 1")
                && method.contains("invokestatic Program/bccCopyArray"),
            "typed method receiver, array parameter slot, or by-value copy prologue missing: {method}"
        );
    }

    #[test]
    fn jvm_generation_emits_typed_method_default_arguments() {
        let ast_source = "method sum%[integer](left%, right%=3)\nreturn self%+left%+right%\nend method\nbase%=10\nresult%=base%.sum(2)\nprint result%\nend\n";
        let semantic_source = "method sum%[integer](left%, right%=8)\nreturn self%+left%+right%\nend method\nbase%=20\nresult%=base%.sum(4)\nprint result%\nend\n";
        let parsed = crate::parse_source(
            "jvm_semantic_method_default_argument.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_method_default_argument.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains(
                "getstatic Program/g1 I\n    ldc 4\n    ldc 8\n    invokestatic Program/sum (III)I"
            ),
            "typed method call must append the typed IR default after explicit arguments: {main}"
        );
        assert!(
            !main.contains("ldc 2\n") && !main.contains("ldc 3\n"),
            "AST method argument or default replaced typed IR: {main}"
        );
    }

    #[test]
    fn jvm_generation_emits_typed_function_default_arguments() {
        let ast_source = "function sum%(left%, right%=3)\nreturn left%+right%\nend function\nresult%=sum%(2)\nprint result%\nend\n";
        let semantic_source = "function sum%(left%, right%=8)\nreturn left%+right%\nend function\nresult%=sum%(4)\nprint result%\nend\n";
        let parsed = crate::parse_source(
            "jvm_semantic_function_default_argument.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_function_default_argument.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("ldc 4\n    ldc 8\n    invokestatic Program/sum (II)I"),
            "typed function call must append the typed IR default after explicit arguments: {main}"
        );
        assert!(
            !main.contains("ldc 2\n") && !main.contains("ldc 3\n"),
            "AST function argument or default replaced typed IR: {main}"
        );
    }

    #[test]
    fn jvm_generation_emits_typed_function_calls_with_byref_scalars() {
        let ast_source = "function mutate%(byref item%)\nitem%=item%+1\nreturn item%\nend function\nvalue%=1\nother%=2\nresult%=mutate%(value%)\nend\n";
        let semantic_source = "function mutate%(byref item%)\nitem%=item%+9\nreturn item%\nend function\nvalue%=1\nother%=2\nresult%=mutate%(other%)\nend\n";
        let parsed = crate::parse_source(
            "jvm_semantic_byref_function_call.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_byref_function_call.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("getstatic Program/g1 I"),
            "typed byref actual must seed the wrapper from `other%`: {main}"
        );
        assert!(
            !main.contains("getstatic Program/g3 I"),
            "AST byref actual replaced typed IR: {main}"
        );
        let function = output
            .split(".method public static mutate :")
            .nth(1)
            .unwrap();
        let function = function.split(".end method").next().unwrap();
        assert!(
            function.contains("ldc 9\n    iadd"),
            "typed byref function body missing: {function}"
        );
        assert!(
            !function.contains("ldc 1\n    iadd"),
            "AST byref function body replaced typed IR: {function}"
        );
    }

    #[test]
    fn jvm_semantic_byref_calls_nested_in_print_declare_the_output_slot() {
        let ast_source = "function mutate%(byref item%)\nitem% = item% + 1\nreturn item%\nend function\nfunction caller%()\nprint 2\nreturn 0\nend function\nprint caller%()\nend\n";
        let semantic_source = "function mutate%(byref item%)\nitem% = item% + 9\nreturn item%\nend function\nfunction caller%()\nprint mutate%(implicit%)\nreturn 0\nend function\nprint caller%()\nend\n";
        let parsed = crate::parse_source(
            "jvm_semantic_byref_nested_print.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_byref_nested_print.bcl",
                semantic_source,
            )
            .unwrap(),
        );

        let output = super::generate(&resolved)
            .expect("typed nested BYREF call must allocate its output slot");
        let caller = output
            .split(".method public static caller :")
            .nth(1)
            .expect("generated caller method")
            .split(".end method")
            .next()
            .expect("caller method body");
        assert!(
            caller.contains("invokestatic Program/mutate ([I)I"),
            "typed nested BYREF call missing: {caller}"
        );
        assert!(
            caller.contains("iastore"),
            "typed BYREF output must write back to its registered local slot: {caller}"
        );
    }

    #[test]
    fn jvm_generation_emits_typed_function_calls_with_array_arguments() {
        let ast_source = "function mutate%(byref values%(?))\nvalues%(0)=values%(0)+1\nreturn values%(0)\nend function\ndim original%(1)\ndim typed%(1)\noriginal%(0)=1\ntyped%(0)=2\nresult%=mutate%(original%)\nend\n";
        let semantic_source = "function mutate%(byref values%(?))\nvalues%(0)=values%(0)+9\nreturn values%(0)\nend function\ndim original%(1)\ndim typed%(1)\noriginal%(0)=1\ntyped%(0)=2\nresult%=mutate%(typed%)\nend\n";
        let parsed = crate::parse_source(
            "jvm_semantic_array_function_call.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_array_function_call.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("getstatic Program/a1 [I\n    invokestatic Program/mutate ([I)I"),
            "typed array actual must replace AST array argument: {main}"
        );
        let function = output
            .split(".method public static mutate :")
            .nth(1)
            .unwrap();
        let function = function.split(".end method").next().unwrap();
        assert!(
            function.contains("ldc 9\n    iadd"),
            "typed array callable body missing: {function}"
        );
        assert!(
            !function.contains("iconst_1\n    iadd"),
            "AST array callable body replaced typed IR: {function}"
        );
    }

    #[test]
    fn jvm_generation_injects_array_copy_helper_for_semantic_byval_parameters() {
        let ast_source = "function copy%(byref values%(?))\nvalues%(0)=values%(0)+1\nreturn values%(0)\nend function\ndim values%(1)\nresult%=copy%(values%)\nend\n";
        let semantic_source = "function copy%(values%(?))\nvalues%(0)=values%(0)+1\nreturn values%(0)\nend function\ndim values%(1)\nresult%=copy%(values%)\nend\n";
        let parsed = crate::parse_source(
            "jvm_semantic_byval_array_helper.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_byval_array_helper.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        assert!(
            output.contains(".method private static bccCopyArray :"),
            "semantic byval parameter emits a call but no array-copy helper: {output}"
        );
    }

    #[test]
    fn jvm_callable_parameter_layout_uses_semantic_array_metadata() {
        let ast_source = "function copy%(values%)\nvalues%(0)=values%(0)+1\nreturn values%(0)\nend function\ndim values%(1)\nresult%=copy%(values%)\nend\n";
        let semantic_source = "function copy%(items%(?))\nitems%(0)=items%(0)+10\nreturn items%(0)\nend function\ndim values%(1)\nresult%=copy%(values%)\nend\n";
        let parsed = crate::parse_source(
            "jvm_semantic_array_parameter_layout.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_array_parameter_layout.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let function = output.split(".method public static copy :").nth(1).unwrap();
        let function = function.split(".end method").next().unwrap();
        assert!(
            function.contains("invokestatic Program/bccCopyArray"),
            "semantic byval array parameter must be copied in its callable prologue: {function}"
        );
        assert!(
            function.contains("aastore") || function.contains("iastore"),
            "typed array parameter must be writable in the callable body: {function}"
        );
        assert!(
            function.contains("ldc 10\n    iadd"),
            "semantic callable body must resolve its typed parameter binding: {function}"
        );
    }

    #[test]
    fn jvm_callable_arity_and_slots_come_from_semantic_signature() {
        let ast_source =
            "function total%(old%)\nreturn old%\nend function\nresult%=total%(1)\nend\n";
        let semantic_source = "function total%(left%, right%)\nreturn left%+right%\nend function\nresult%=total%(4,5)\nend\n";
        let parsed =
            crate::parse_source("jvm_semantic_callable_arity.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_callable_arity.bcl",
                semantic_source,
            )
            .unwrap(),
        );

        let output = super::generate(&resolved).unwrap();

        assert!(
            output.contains(".method public static total : (II)I"),
            "callable descriptor must use typed parameter arity: {output}"
        );
        assert!(
            output.contains("iload 0\n    iload 1\n    iadd"),
            "callable local slots must be allocated from the typed signature: {output}"
        );
        assert!(
            output.contains("ldc 4\n    ldc 5\n    invokestatic Program/total (II)I"),
            "typed call arity and arguments must match the callable descriptor: {output}"
        );
    }

    #[test]
    fn jvm_callable_result_slot_comes_from_typed_result_metadata() {
        let source = "function value%(old%)\nreturn old%\nend function\nend\n";
        let parsed = crate::parse_source("jvm_typed_result_slot.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut semantic = crate::semantic_ir::parse_and_adapt(source).unwrap();
        semantic.callables[0].result_type = Some("$".to_string());
        let functions = super::function_table(&program.functions, Some(&semantic));
        assert_eq!(
            functions["value%"].result,
            super::JvmType::String,
            "typed result metadata must override the AST suffix"
        );
    }

    #[test]
    fn jvm_callable_receiver_type_comes_from_semantic_signature() {
        let parsed = crate::parse_source(
            "jvm_semantic_receiver_type.bcl".to_string(),
            "method echo$[string]()\nreturn self$\nend method\nend\n",
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic_module = crate::semantic_ir::parse_and_adapt_named(
            "jvm_semantic_receiver_type.bcl",
            "method echo$[integer]()\nreturn self$\nend method\nend\n",
        )
        .unwrap();
        let functions = super::function_table(&program.functions, Some(&semantic_module));
        let signature = functions.values().next().unwrap();
        assert_eq!(
            signature.params.first(),
            Some(&super::JvmType::Numeric(super::NumericType::Int))
        );
        assert_eq!(
            signature
                .receiver_ident
                .as_ref()
                .and_then(|ident| ident.suffix),
            Some(crate::ast::TypeSuffix::Integer)
        );
        let context = super::JvmContext::build(
            &program,
            functions,
            "Program".to_string(),
            std::collections::HashMap::new(),
            Vec::new(),
            Some(&semantic_module),
            None,
        )
        .unwrap();
        let method_context = super::JvmContext::for_function(&program.functions[0], &context);
        let self_ident = crate::ast::BasicIdent {
            name: "self".to_string(),
            suffix: Some(crate::ast::TypeSuffix::Integer),
        };
        let receiver = method_context
            .variables
            .get(&super::variable_key(&self_ident))
            .expect("semantic receiver local");
        assert_eq!(
            receiver.ty,
            super::JvmType::Numeric(super::NumericType::Int)
        );
    }

    #[test]
    fn jvm_callable_generation_dispatches_aligned_semantic_assignments() {
        let ast_source = "function work%(value%)\nwork% = value% + 1\nreturn value% + 2\nend function\nprint work%(0)\nend\n";
        let parsed =
            crate::parse_source("jvm_callable_semantic.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(crate::semantic_ir::parse_and_adapt_named(
            "jvm_callable_semantic.bcl",
            "function work%(value%)\nwork% = value% + 9\nreturn value% + 8\nend function\nprint work%(0)\nend\n",
        ).unwrap());
        let output = super::generate(&resolved).unwrap();
        let function = output.split(".method public static work :").nth(1).unwrap();
        let function = function.split(".end method").next().unwrap();
        assert!(
            function.contains("    ldc 9\n    iadd\n"),
            "semantic callable assignment missing: {function}"
        );
        assert!(
            !function.contains("    ldc 1\n    iadd\n"),
            "AST callable assignment replaced semantic IR: {function}"
        );
        assert!(
            function.contains("    ldc 8\n    iadd\n    ireturn\n"),
            "semantic callable RETURN missing: {function}"
        );
        assert!(
            !function.contains("    ldc 2\n    iadd\n    ireturn\n"),
            "AST callable RETURN replaced semantic IR: {function}"
        );
    }

    #[test]
    fn jvm_callable_generation_dispatches_semantic_end_as_process_exit() {
        let ast_source = "function work%()\nprint 3\nreturn 2\nend function\nend\n";
        let parsed = crate::parse_source("jvm_callable_end.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_callable_end.bcl",
                "function work%()\nend\nreturn 8\nend function\nend\n",
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let function = output
            .split(".method public static work :")
            .nth(1)
            .expect("generated callable")
            .split(".end method")
            .next()
            .unwrap();
        assert!(
            function.contains("iconst_0\n    invokestatic java/lang/System/exit (I)V"),
            "typed callable END did not terminate the process: {function}"
        );
        assert!(
            !function.contains("getstatic java/lang/System/out") && !function.contains("ldc 2\n"),
            "AST callable body replaced semantic END: {function}"
        );
        assert!(
            function.contains("ldc 8\n    ireturn"),
            "semantic callable fallthrough body missing: {function}"
        );
    }

    #[test]
    fn jvm_generation_dispatches_typed_numeric_assignments_from_semantic_ir() {
        let parsed = crate::parse_source(
            "jvm_semantic_numeric.bcl".to_string(),
            "value% = 1 + 2\nratio% = 3 / 2\nend\n",
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_numeric.bcl",
                "value% = 7 + 8\nratio% = 9 / 2\nend\n",
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("ldc 7\n    ldc 8\n    iadd\n"),
            "semantic arithmetic assignment missing: {main}"
        );
        assert!(
            !main.contains("ldc 1\n    ldc 2\n    iadd\n"),
            "AST arithmetic replaced semantic IR: {main}"
        );
        assert!(
            main.contains("ldc 9\n    i2d\n    ldc 2\n    i2d\n    ddiv\n"),
            "semantic true division missing: {main}"
        );
        assert!(
            main.contains("d2l\n    l2i\n"),
            "narrowing conversion did not preserve BASIC rounding: {main}"
        );
    }

    #[test]
    fn jvm_generation_dispatches_compound_numeric_assignment_from_semantic_ir() {
        let parsed =
            crate::parse_source("jvm_semantic_compound.bcl".to_string(), "value% = 2\nend\n")
                .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_compound.bcl",
                "value% += 7\nend\n",
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("ldc 7\n    iadd\n"),
            "semantic compound assignment missing: {main}"
        );
        assert!(
            !main.contains("ldc 2\n"),
            "AST assignment replaced semantic IR: {main}"
        );
    }

    #[test]
    fn jvm_generation_dispatches_typed_semantic_scalar_swap() {
        let ast_source = "a% = 1\nb% = 2\nc% = 3\nswap a%, b%\nend\n";
        let semantic_source = "a% = 7\nb% = 8\nc% = 9\nswap a%, c%\nend\n";
        let parsed = crate::parse_source("jvm_semantic_swap.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named("jvm_semantic_swap.bcl", semantic_source)
                .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(main.contains("getstatic Program/g1 I\n    getstatic Program/g3 I\n    putstatic Program/g1 I\n    putstatic Program/g3 I"), "semantic scalar SWAP operands missing: {main}");
        assert!(!main.contains("getstatic Program/g1 I\n    getstatic Program/g2 I\n    putstatic Program/g1 I\n    putstatic Program/g2 I"), "AST SWAP operands replaced semantic IR: {main}");
    }

    #[test]
    fn jvm_callable_generation_dispatches_typed_semantic_scalar_swap() {
        let ast_source = "procedure exchange()\nleft% = 1\nright% = 2\nswap left%, right%\nend procedure\nexchange()\nend\n";
        let parsed =
            crate::parse_source("jvm_callable_semantic_swap.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(crate::semantic_ir::parse_and_adapt_named(
            "jvm_callable_semantic_swap.bcl",
            "procedure exchange()\nleft% = 9\nright% = 8\nswap left%, right%\nend procedure\nexchange()\nend\n",
        ).unwrap());
        let output = super::generate(&resolved).unwrap();
        let procedure = output
            .split(".method public static exchange :")
            .nth(1)
            .unwrap();
        let procedure = procedure.split(".end method").next().unwrap();
        assert!(
            procedure.contains("ldc 9\n    istore"),
            "semantic callable first operand setup missing: {procedure}"
        );
        assert!(
            procedure.contains("ldc 8\n    istore"),
            "semantic callable second operand setup missing: {procedure}"
        );
        assert!(
            procedure.contains("iload") && procedure.matches("istore").count() >= 4,
            "semantic callable scalar SWAP loads/stores missing: {procedure}"
        );
        assert!(
            !procedure.contains("ldc 1\n    istore") && !procedure.contains("ldc 2\n    istore"),
            "AST callable operands replaced semantic IR: {procedure}"
        );
    }

    #[test]
    fn jvm_generation_dispatches_semantic_print_tokens_and_separators() {
        let parsed = crate::parse_source(
            "jvm_semantic_print.bcl".to_string(),
            "print \"ast\"; 1\nprint 2;\nend\n",
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_print.bcl",
                "print \"semantic\"; 7\nprint 9;\nend\n",
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("    ldc \"semantic\"\n"),
            "semantic PRINT text missing: {main}"
        );
        assert!(
            !main.contains("    ldc \"ast\"\n"),
            "AST PRINT text replaced semantic IR: {main}"
        );
        assert!(
            main.contains("    ldc 7\n    invokevirtual java/io/PrintStream/println (I)V\n"),
            "PRINT expression or newline semantics missing: {main}"
        );
        assert!(
            main.contains("    ldc 9\n    invokevirtual java/io/PrintStream/print (I)V\n"),
            "trailing separator behavior missing: {main}"
        );
        assert!(
            main.contains("invokevirtual java/io/PrintStream/flush ()V"),
            "trailing separator did not flush: {main}"
        );
    }

    #[test]
    fn jvm_callable_generation_dispatches_semantic_print_and_lprint_tokens() {
        let ast_source = "procedure report()\nprint \"old\"; 1\nlprint \"printer\"; 2\nend procedure\nreport()\nend\n";
        let parsed =
            crate::parse_source("jvm_callable_semantic_print.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(crate::semantic_ir::parse_and_adapt_named(
            "jvm_callable_semantic_print.bcl",
            "procedure report()\nprint \"semantic\"; 7\nlprint \"plain\"; 9\nend procedure\nreport()\nend\n",
        ).unwrap());
        let output = super::generate(&resolved).unwrap();
        let procedure = output
            .split(".method public static report :")
            .nth(1)
            .unwrap();
        let procedure = procedure.split(".end method").next().unwrap();
        assert!(
            procedure.contains("ldc \"semantic\""),
            "semantic callable PRINT token missing: {procedure}"
        );
        assert!(
            procedure.contains("ldc 7"),
            "semantic callable PRINT numeric token missing: {procedure}"
        );
        assert!(
            procedure.contains("ldc \"plain\""),
            "semantic callable LPRINT token missing: {procedure}"
        );
        assert!(
            procedure.contains("ldc 9"),
            "semantic callable LPRINT numeric token missing: {procedure}"
        );
        assert!(
            !procedure.contains("old") && !procedure.contains("printer"),
            "AST callable print tokens replaced semantic IR: {procedure}"
        );
    }

    #[test]
    fn jvm_generation_dispatches_plain_lprint_tokens_from_semantic_ir() {
        let parsed = crate::parse_source(
            "jvm_semantic_lprint.bcl".to_string(),
            "lprint \"ast\"; 1\nend\n",
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_lprint.bcl",
                "lprint \"semantic\"; 7\nend\n",
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("ldc \"semantic\""),
            "semantic LPRINT string missing: {main}"
        );
        assert!(
            !main.contains("ldc \"ast\""),
            "AST LPRINT text replaced semantic IR: {main}"
        );
        assert!(
            main.contains("ldc 7\n    invokevirtual java/io/PrintStream/println (I)V"),
            "semantic LPRINT value missing: {main}"
        );
    }

    #[test]
    fn jvm_generation_dispatches_console_input_from_semantic_ir() {
        let parsed = crate::parse_source(
            "jvm_semantic_input.bcl".to_string(),
            "input \"ast\"; value%\nend\n",
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_input.bcl",
                "input \"semantic\"; value%\nend\n",
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("ldc \"semantic? \""),
            "semantic INPUT prompt missing: {main}"
        );
        assert!(
            !main.contains("ldc \"ast? \""),
            "AST INPUT prompt replaced semantic IR: {main}"
        );
        assert!(
            main.contains("java/lang/Integer/parseInt (Ljava/lang/String;)I"),
            "typed semantic INPUT conversion missing: {main}"
        );
    }

    #[test]
    fn jvm_generation_dispatches_console_input_to_semantic_array_elements() {
        let parsed = crate::parse_source(
            "jvm_semantic_input_array.bcl".to_string(),
            "dim values%(10)\ninput \"ast\"; values%(1)\nend\n",
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_input_array.bcl",
                "dim values%(10)\ninput \"semantic\"; values%(2)\nend\n",
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("ldc \"semantic? \""),
            "semantic INPUT prompt missing: {main}"
        );
        assert!(
            main.contains("getstatic Program/a0 [I\n    ldc 2\n    getstatic Program/bccStdin"),
            "semantic indexed INPUT target missing: {main}"
        );
        assert!(
            main.contains("java/lang/Integer/parseInt (Ljava/lang/String;)I\n    iastore"),
            "semantic indexed INPUT conversion/store missing: {main}"
        );
    }

    #[test]
    fn jvm_callable_generation_dispatches_console_input_to_semantic_array_elements() {
        let ast_source = "function read%()\ndim values%(10)\ninput \"old\"; values%(1)\nreturn values%(1)\nend function\nprint read%()\nend\n";
        let parsed = crate::parse_source(
            "jvm_callable_semantic_input_array.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(crate::semantic_ir::parse_and_adapt_named(
            "jvm_callable_semantic_input_array.bcl",
            "function read%()\ndim values%(10)\ninput \"semantic\"; values%(2)\nreturn values%(2)\nend function\nprint read%()\nend\n",
        ).unwrap());
        let output = super::generate(&resolved).unwrap();
        let function = output.split(".method public static read :").nth(1).unwrap();
        let function = function.split(".end method").next().unwrap();
        assert!(
            function.contains("ldc \"semantic? \""),
            "semantic callable prompt missing: {function}"
        );
        assert!(
            function.contains("ldc 2"),
            "semantic callable input index missing: {function}"
        );
        assert!(
            !function.contains("ldc \"old? \""),
            "AST callable prompt replaced semantic IR: {function}"
        );
        assert!(
            !function.contains("    ldc 1\n"),
            "AST callable input index replaced semantic IR: {function}"
        );
    }

    #[test]
    fn jvm_generation_prints_typed_semantic_array_reads() {
        let parsed = crate::parse_source(
            "jvm_semantic_array_read.bcl".to_string(),
            "dim values%(10)\nprint values%(1)\nend\n",
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_array_read.bcl",
                "dim values%(10)\nprint values%(2)\nend\n",
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("getstatic Program/a0 [I\n    ldc 2\n    iaload\n"),
            "semantic JVM array index or typed load missing: {main}"
        );
        assert!(
            !main.contains("    ldc 1\n"),
            "AST array index replaced semantic IR: {main}"
        );
    }

    #[test]
    fn jvm_generation_assigns_typed_semantic_array_elements() {
        let parsed = crate::parse_source(
            "jvm_semantic_array_assignment.bcl".to_string(),
            "dim values%(10)\ndim labels(10) as string\nvalues%(1) = 2\nlabels(1) = \"old\"\nend\n",
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(crate::semantic_ir::parse_and_adapt_named(
            "jvm_semantic_array_assignment.bcl",
            "dim values%(10)\ndim labels(10) as string\nvalues%(2) = 7\nlabels(2) = \"semantic\"\nend\n",
        ).unwrap());
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("getstatic Program/a1 [I\n    ldc 2\n    ldc 7\n    iastore\n"),
            "typed semantic array assignment missing: {main}"
        );
        assert!(
            !main.contains("    ldc 1\n    ldc 2\n    iastore\n"),
            "AST array assignment replaced semantic IR: {main}"
        );
        assert!(main.contains("getstatic Program/a0 [Ljava/lang/String;\n    ldc 2\n    ldc \"semantic\"\n    aastore\n"), "typed semantic string array assignment missing: {main}");
    }

    #[test]
    fn jvm_generation_dispatches_typed_rank_two_array_reads_and_writes() {
        let ast_source = "dim values%(2, 3)\ndim labels(2, 3) as string\nvalues%(1, 1) = 2\nvalues%(1, 1) += 3\nlabels(1, 1) = \"old\"\nprint values%(1, 1), labels(1, 1)\nend\n";
        let parsed =
            crate::parse_source("jvm_semantic_rank_two_arrays.bcl".to_string(), ast_source)
                .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(crate::semantic_ir::parse_and_adapt_named(
            "jvm_semantic_rank_two_arrays.bcl",
            "dim values%(2, 3)\ndim labels(2, 3) as string\nvalues%(2, 3) = 7\nvalues%(2, 3) += 11\nlabels(2, 3) = \"semantic\"\nprint values%(2, 3), labels(2, 3)\nend\n",
        ).unwrap());
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("[[I\n    ldc 2\n    aaload\n    ldc 3\n    ldc 7\n    iastore"),
            "semantic rank-two numeric assignment missing: {main}"
        );
        assert!(
            main.contains(
                "aaload\n    ldc 3\n    dup2\n    iaload\n    ldc 11\n    iadd\n    iastore"
            ),
            "semantic rank-two compound assignment missing: {main}"
        );
        assert!(main.contains("[[Ljava/lang/String;\n    ldc 2\n    aaload\n    ldc 3\n    ldc \"semantic\"\n    aastore"), "semantic rank-two string assignment missing: {main}");
        assert!(
            main.contains("    aaload\n    ldc 3\n    iaload"),
            "semantic rank-two numeric read missing: {main}"
        );
        assert!(
            main.contains("    aaload\n    ldc 3\n    aaload"),
            "semantic rank-two string read missing: {main}"
        );
        assert!(
            !main.contains("ldc \"old\""),
            "AST rank-two array value replaced semantic IR: {main}"
        );
        assert!(
            !main.contains("ldc 1\n"),
            "AST rank-two array indices replaced semantic IR: {main}"
        );
    }

    #[test]
    fn jvm_callable_generation_dispatches_semantic_local_array_access() {
        let parsed = crate::parse_source(
            "jvm_callable_semantic_array.bcl".to_string(),
            "function value%()\ndim values%(10)\ndim labels(10) as string\nvalues%(1) = 2\nlabels(1) = \"old\"\nprint labels(1)\nreturn values%(1)\nend function\nprint value%()\nend\n",
        ).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(crate::semantic_ir::parse_and_adapt_named(
            "jvm_callable_semantic_array.bcl",
            "function value%()\ndim values%(10)\ndim labels(10) as string\nvalues%(2) = 7\nlabels(2) = \"semantic\"\nprint labels(2)\nreturn values%(2)\nend function\nprint value%()\nend\n",
        ).unwrap());
        let output = super::generate(&resolved).unwrap();
        let method = output
            .split(".method public static value :")
            .nth(1)
            .unwrap();
        let method = method.split(".end method").next().unwrap();
        assert!(
            method.contains("ldc 2\n    ldc 7\n    iastore"),
            "callable semantic local array assignment missing: {method}"
        );
        assert!(
            method.contains("ldc 2\n    iaload"),
            "callable semantic local array read missing: {method}"
        );
        assert!(
            method.contains("ldc \"semantic\"\n    aastore"),
            "callable semantic string array assignment missing: {method}"
        );
        assert!(
            method.contains("ldc 2\n    aaload"),
            "callable semantic string array read missing: {method}"
        );
        assert!(
            !method.contains("ldc \"old\""),
            "AST callable string array value replaced semantic IR: {method}"
        );
        assert!(
            !method.contains("ldc 1\n"),
            "AST callable array index replaced semantic IR: {method}"
        );
    }

    #[test]
    fn jvm_callable_generation_dispatches_semantic_compound_local_array_assignment() {
        let ast_source = "function value%()\ndim values%(10)\nvalues%(1) = 2\nvalues%(1) += 3\nreturn values%(1)\nend function\nprint value%()\nend\n";
        let semantic_source = "function value%()\ndim values%(10)\nvalues%(2) = 7\nvalues%(2) += 11\nreturn values%(2)\nend function\nprint value%()\nend\n";
        let parsed =
            crate::parse_source("jvm_callable_array_compound.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_callable_array_compound.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let function = output
            .split(".method public static value :")
            .nth(1)
            .unwrap();
        let function = function.split(".end method").next().unwrap();
        assert!(
            function.contains("ldc 2\n    dup2\n    iaload\n    ldc 11\n    iadd\n    iastore"),
            "semantic compound array assignment missing: {function}"
        );
        assert!(
            !function.contains("ldc 1\n    dup2\n    iaload\n    ldc 3"),
            "AST array index/value replaced semantic IR: {function}"
        );
    }

    #[test]
    fn jvm_callable_generation_dispatches_typed_rank_two_array_reads_and_writes() {
        let ast_source = "function value%()\ndim grid%(2, 3)\ngrid%(1, 1) = 2\ninput \"old\"; grid%(1, 1)\nreturn grid%(1, 1)\nend function\nprint value%()\nend\n";
        let parsed =
            crate::parse_source("jvm_callable_semantic_rank_two.bcl".to_string(), ast_source)
                .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(crate::semantic_ir::parse_and_adapt_named(
            "jvm_callable_semantic_rank_two.bcl",
            "function value%()\ndim grid%(2, 3)\ngrid%(2, 3) = 7\ninput \"semantic\"; grid%(2, 3)\nreturn grid%(2, 3)\nend function\nprint value%()\nend\n",
        ).unwrap());
        let output = super::generate(&resolved).unwrap();
        let function = output
            .split(".method public static value :")
            .nth(1)
            .unwrap();
        let function = function.split(".end method").next().unwrap();
        assert!(
            function.contains("aaload\n    ldc 3\n    ldc 7\n    iastore"),
            "semantic callable rank-two assignment missing: {function}"
        );
        assert!(
            function.contains("aaload\n    ldc 3\n    iaload\n    ireturn"),
            "semantic callable rank-two read/return missing: {function}"
        );
        assert!(
            function.contains("ldc \"semantic? \""),
            "semantic callable rank-two INPUT prompt missing: {function}"
        );
        assert!(
            function.contains("ldc 2\n    aaload\n    ldc 3\n    getstatic Program/bccStdin"),
            "semantic callable rank-two INPUT address missing: {function}"
        );
        assert!(
            function.contains("Integer/parseInt (Ljava/lang/String;)I\n    iastore"),
            "semantic callable rank-two INPUT conversion/store missing: {function}"
        );
        assert!(
            !function.contains("ldc 1\n"),
            "AST callable rank-two indices replaced semantic IR: {function}"
        );
    }

    #[test]
    fn jvm_generation_dispatches_typed_array_lvalues_with_callable_indices() {
        let ast_source = "function index%()\nreturn 1\nend function\nfunction update%()\ndim values%(10)\nvalues%(1) = 2\ninput \"old\"; values%(1)\nreturn 0\nend function\nprint update%()\nend\n";
        let semantic_source = "function index%()\nreturn 2\nend function\nfunction update%()\ndim values%(10)\nvalues%(index%()) = 7\ninput \"typed\"; values%(index%())\nreturn 0\nend function\nprint update%()\nend\n";
        let parsed = crate::parse_source("jvm_callable_array_callable_index.bcl".to_string(), ast_source)
            .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_callable_array_callable_index.bcl",
                semantic_source,
            )
            .unwrap(),
        );

        let output = super::generate(&resolved).unwrap();
        let function = output
            .split(".method public static update :")
            .nth(1)
            .unwrap()
            .split(".end method")
            .next()
            .unwrap();

        assert!(
            function.matches("invokestatic Program/index ()I").count() >= 2,
            "typed assignment and INPUT indices must invoke the semantic callable: {function}"
        );
        assert!(
            function.contains("ldc 7") && function.contains("ldc \"typed? \""),
            "typed array assignment and INPUT prompt must replace stale AST values: {function}"
        );
        assert!(
            !function.lines().any(|line| line == "    ldc 1"),
            "AST array indices must not replace typed IR: {function}"
        );
    }

    #[test]
    fn jvm_generation_dispatches_typed_comparison_and_integer_operators() {
        let parsed = crate::parse_source(
            "jvm_semantic_operators.bcl".to_string(),
            "cmp% = 0\ntruth% = 1.5 < 2.5\nquotient& = 0\nremainder& = 0\nmask& = 0\npower# = 0\nend\n",
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        let ast_output = super::generate(&resolved).unwrap();
        let ast_main = ast_output
            .split(".method public static main :")
            .nth(1)
            .unwrap();
        let ast_main = ast_main.split(".end method").next().unwrap();
        assert!(
            ast_main.contains("dcmpg") && !ast_main.contains("java/lang/Double/compare"),
            "AST double comparison doesn't use IEEE JVM compare opcodes: {ast_main}"
        );
        resolved.semantic_module = Some(crate::semantic_ir::parse_and_adapt_named(
            "jvm_semantic_operators.bcl",
            "cmp% = 5 < 6\ntruth% = 1.5 < 2.5\nquotient& = 9 \\ 2\nremainder& = 9 mod 2\nmask& = 5 and 3\npower# = 2 ^ 3\nend\n",
        ).unwrap());
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("java/lang/Integer/compare (II)I"),
            "semantic comparison missing: {main}"
        );
        assert!(
            main.contains("dcmpg") && !main.contains("java/lang/Double/compare"),
            "semantic double comparison doesn't use IEEE JVM compare opcodes: {main}"
        );
        assert!(
            main.contains("ldiv\n"),
            "semantic integer division missing: {main}"
        );
        assert!(main.contains("lrem\n"), "semantic MOD missing: {main}");
        assert!(
            main.contains("land\n"),
            "semantic bitwise AND missing: {main}"
        );
        assert!(
            main.contains("java/lang/Math/pow (DD)D"),
            "semantic exponentiation missing: {main}"
        );
    }

    #[test]
    fn jvm_generation_preserves_semantic_short_circuit_boolean_operators() {
        let parsed = crate::parse_source(
            "jvm_semantic_short_circuit.bcl".to_string(),
            "truth% = 0\nend\n",
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_short_circuit.bcl",
                "truth% = 5 < 6 && 4 < 5\nend\n",
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("ifeq L_condition_"),
            "semantic AND does not skip its right operand when false: {main}"
        );
        assert!(
            main.contains("iconst_m1"),
            "semantic truth value is not BASIC -1: {main}"
        );
        assert!(
            main.contains("putstatic") || main.contains("istore"),
            "semantic boolean result was not stored: {main}"
        );
    }

    #[test]
    fn jvm_generation_dispatches_locate_from_typed_semantic_expressions() {
        let parsed =
            crate::parse_source("jvm_semantic_locate.bcl".to_string(), "locate 2, 3\nend\n")
                .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_locate.bcl",
                "locate 7, 9\nend\n",
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("ldc 7\n"),
            "semantic LOCATE row missing: {main}"
        );
        assert!(
            main.contains("ldc 9\n"),
            "semantic LOCATE column missing: {main}"
        );
        assert!(
            !main.contains("ldc 2\n") && !main.contains("ldc 3\n"),
            "AST LOCATE replaced semantic IR: {main}"
        );
        assert!(
            main.contains("append (I)Ljava/lang/StringBuilder;"),
            "LOCATE did not use integer cursor coordinates: {main}"
        );
    }

    #[test]
    fn jvm_generation_dispatches_literal_color_from_semantic_ir() {
        let parsed =
            crate::parse_source("jvm_semantic_color.bcl".to_string(), "color 1, 2\nend\n").unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_semantic_color.bcl",
                "color 4, 3\nend\n",
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("\u{1b}[31;46m"),
            "semantic COLOR did not preserve CGA-to-ANSI mapping: {main:?}"
        );
        assert!(
            !main.contains("\u{1b}[34;42m"),
            "AST COLOR replaced semantic IR: {main:?}"
        );
    }

    #[test]
    fn jvm_generation_dispatches_dynamic_color_from_typed_semantic_expressions() {
        let parsed = crate::parse_source(
            "jvm_dynamic_semantic_color.bcl".to_string(),
            "foreground% = 1\ncolor 1\nend\n",
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_dynamic_semantic_color.bcl",
                "foreground% = 4\ncolor foreground%\nend\n",
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        assert!(
            main.contains("invokestatic Program/bccAnsiFg (I)I"),
            "dynamic semantic foreground mapping missing: {main}"
        );
        assert!(
            !main.contains("ldc \"\u{1b}[34m\""),
            "AST literal COLOR replaced semantic IR: {main:?}"
        );
        assert!(
            output.contains(".method private static bccAnsiFg : (I)I"),
            "dynamic foreground palette helper missing: {output}"
        );
    }

    #[test]
    fn jvm_callable_generation_dispatches_locate_and_color_from_semantic_ir() {
        let ast_source =
            "procedure paint()\nlocate 1, 2\ncolor 1, 2\nend procedure\npaint()\nend\n";
        let parsed =
            crate::parse_source("jvm_callable_semantic_terminal.bcl".to_string(), ast_source)
                .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "jvm_callable_semantic_terminal.bcl",
                "procedure paint()\nlocate 4, 5\ncolor 4, 3\nend procedure\npaint()\nend\n",
            )
            .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        let procedure = output
            .split(".method public static paint :")
            .nth(1)
            .unwrap();
        let procedure = procedure.split(".end method").next().unwrap();
        assert!(procedure.contains("ldc 4\n    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;"), "semantic LOCATE row missing: {procedure}");
        assert!(procedure.contains("ldc 5\n    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;"), "semantic LOCATE column missing: {procedure}");
        assert!(!procedure.contains("ldc 1\n    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;"), "AST LOCATE replaced semantic IR: {procedure}");
        assert!(
            procedure.contains("\u{1b}[31;46m"),
            "semantic COLOR palette mapping missing: {procedure:?}"
        );
        assert!(
            !procedure.contains("\u{1b}[34;42m"),
            "AST COLOR replaced semantic IR: {procedure:?}"
        );
    }

    #[test]
    fn jvm_callable_dynamic_color_registers_semantic_palette_helpers() {
        let ast_source =
            "procedure paint()\nforeground% = 1\ncolor foreground%\nend procedure\npaint()\nend\n";
        let parsed =
            crate::parse_source("jvm_callable_dynamic_color.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(crate::semantic_ir::parse_and_adapt_named(
            "jvm_callable_dynamic_color.bcl",
            "procedure paint()\nforeground% = 4\ncolor foreground%\nend procedure\npaint()\nend\n",
        ).unwrap());
        let output = super::generate(&resolved).unwrap();
        let procedure = output
            .split(".method public static paint :")
            .nth(1)
            .unwrap();
        let procedure = procedure.split(".end method").next().unwrap();
        assert!(
            procedure.contains("invokestatic Program/bccAnsiFg (I)I"),
            "semantic dynamic COLOR call missing: {procedure}"
        );
        assert!(
            output.contains(".method private static bccAnsiFg : (I)I"),
            "callable semantic COLOR did not register its palette helper: {output}"
        );
    }

    #[test]
    fn semantic_sequential_file_operations_request_file_runtime() {
        let module = crate::semantic_ir::parse_and_adapt(
            "file scores = open(path$) for input\nwrite #1, value%\n",
        )
        .expect("semantic frontend accepts sequential file operations");
        assert!(semantic_runtime_features(&module).0);
    }

    #[test]
    fn semantic_file_mutation_operations_request_file_runtime() {
        let module =
            crate::semantic_ir::parse_and_adapt("seek #1, 4\nkill path$\nname old$ as new$\n")
                .expect("semantic file mutation statements parse");
        assert!(semantic_runtime_features(&module).0);
    }

    #[test]
    fn semantic_callable_file_operations_request_file_runtime() {
        let module = crate::semantic_ir::parse_and_adapt(
            "procedure close_file()\nclose #1\nend procedure\n",
        )
        .expect("semantic callable file statements parse");
        assert!(semantic_runtime_features(&module).0);
    }

    #[test]
    fn jvm_generation_uses_semantic_suffixless_string_dim() {
        let source = "dim text as string\nend\n";
        let parsed = crate::parse_source("semantic_dim.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(crate::semantic_ir::parse_and_adapt(source).unwrap());
        let output = super::generate(&resolved).unwrap();
        assert!(output.contains("Ljava/lang/String;"), "{output}");
    }

    #[test]
    fn jvm_generation_uses_semantic_callable_string_dim() {
        let source = "function f%()\ndim text as string\nreturn 0\nend function\nend\n";
        let parsed = crate::parse_source("semantic_callable_dim.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(crate::semantic_ir::parse_and_adapt(source).unwrap());
        let output = super::generate(&resolved).unwrap();
        assert!(output.contains("Ljava/lang/String;"), "{output}");
    }

    #[test]
    fn jvm_generation_prefers_semantic_array_capacity_over_legacy_shape() {
        let source = "dim values%(10)\nend\n";
        let semantic_source = "dim values%(20)\nend\n";
        let parsed =
            crate::parse_source("semantic_array_precedence.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module =
            Some(crate::semantic_ir::parse_and_adapt(semantic_source).unwrap());
        let output = super::generate(&resolved).unwrap();
        assert!(output.contains("ldc 20"), "{output}");
        assert!(!output.contains("ldc 10"), "{output}");
    }

    #[test]
    fn jvm_generation_emits_nonconstant_array_capacity_from_typed_expression() {
        let ast_source = "dim bound&\nbound& = 2\ndim values%(2)\nend\n";
        let semantic_source = "dim bound&\nbound& = 5\ndim values%(bound&)\nend\n";
        let parsed =
            crate::parse_source("semantic_dynamic_array_bound.bcl".to_string(), ast_source)
                .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let resolved = crate::resolver::resolve_with_semantic(
            program,
            Some(
                crate::semantic_ir::parse_and_adapt_named(
                    "semantic_dynamic_array_bound.bcl",
                    semantic_source,
                )
                .unwrap(),
            ),
        )
        .unwrap();

        let output = super::generate(&resolved).unwrap();
        let array_allocation = output
            .split("multianewarray")
            .next()
            .expect("typed array allocation should be emitted");
        assert!(
            array_allocation.contains("getstatic Program/g")
                && array_allocation.contains("    l2i\n"),
            "typed LONG capacity was not loaded and converted from resolved storage: {output}"
        );
        assert!(
            !output.contains("ldc 0\n    iconst_1\n    iadd\n    multianewarray"),
            "nonconstant typed capacity was silently replaced with zero: {output}"
        );
        let main = output.split(".method public static main :").nth(1).unwrap();
        let main = main.split(".end method").next().unwrap();
        let assigned_bound = main
            .lines()
            .position(|line| line.starts_with("    putstatic Program/g") && line.ends_with(" J"))
            .expect("typed LONG bound assignment should be emitted");
        let loaded_bound = main
            .lines()
            .position(|line| line.starts_with("    getstatic Program/g") && line.ends_with(" J"))
            .expect("dynamic DIM should load its typed bound");
        assert!(
            assigned_bound < loaded_bound,
            "dynamic DIM must evaluate its bound at its source position: {main}"
        );
    }

    #[test]
    fn jvm_generation_prefers_semantic_callable_array_capacity_over_legacy_shape() {
        let ast_source =
            "function capacity%()\ndim values%(2)\nreturn sizeof(values%)\nend function\nend\n";
        let semantic_source =
            "function capacity%()\ndim values%(5)\nreturn sizeof(values%)\nend function\nend\n";
        let parsed = crate::parse_source(
            "semantic_callable_array_precedence.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt(semantic_source).unwrap(),
        );

        let output = super::generate(&resolved).unwrap();
        assert!(output.contains("ldc 5"), "typed callable bound missing: {output}");
        assert!(!output.contains("ldc 2"), "AST callable bound leaked: {output}");
    }

    #[test]
    fn jvm_generation_evaluates_callable_const_array_capacity_from_typed_ir() {
        let module = crate::semantic_ir::parse_and_adapt(
            "function capacity%()\nconst limit = 6\ndim values%(limit)\nreturn 0\nend function\n",
        )
        .unwrap();
        let mut callable_module = module.clone();
        callable_module.statements = module.callables[0].body.clone();
        let mut arrays = std::collections::BTreeMap::new();

        super::collect_semantic_array_declarations(&callable_module, &mut arrays);

        assert_eq!(
            arrays["values%"].dimensions,
            [super::ArrayDimension::Typed(6)]
        );
    }

    #[test]
    fn jvm_generation_evaluates_callable_expression_array_capacity_from_typed_ir() {
        let module = crate::semantic_ir::parse_and_adapt(
            "function capacity%()\ndim values%(2 * 4)\nreturn 0\nend function\n",
        )
        .unwrap();
        let mut callable_module = module.clone();
        callable_module.statements = module.callables[0].body.clone();
        let mut arrays = std::collections::BTreeMap::new();

        super::collect_semantic_array_declarations(&callable_module, &mut arrays);

        assert_eq!(
            arrays["values%"].dimensions,
            [super::ArrayDimension::Typed(8)]
        );
    }

    #[test]
    fn jvm_legacy_array_collection_keeps_ast_dimensions_explicitly_tagged() {
        let parsed = crate::parse_source(
            "legacy_array_shape.bcl".to_string(),
            "dim values%(10)\n",
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut arrays = std::collections::BTreeMap::new();

        super::collect_array_declarations(&program.statements, &mut arrays);

        assert_eq!(
            arrays["values%"].dimensions,
            [super::ArrayDimension::Legacy(crate::ast::Expr::Integer(10))]
        );
    }

    #[test]
    fn jvm_generation_evaluates_named_semantic_const_array_capacity() {
        let source = "const limit = 4\ndim values%(limit)\nend\n";
        let parsed = crate::parse_source("semantic_const_bound.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let resolved = crate::resolver::resolve_with_semantic(
            program,
            Some(crate::semantic_ir::parse_and_adapt(source).unwrap()),
        )
        .unwrap();
        let output = super::generate(&resolved).unwrap();
        assert!(
            output.contains("ldc 4\n    iconst_1\n    iadd\n    multianewarray [I 1"),
            "{output}"
        );
    }

    #[test]
    fn jvm_generation_evaluates_radix_semantic_array_capacity() {
        let source = "dim values%(&H10 + &O10)\nend\n";
        let parsed = crate::parse_source("semantic_radix_bound.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let resolved = crate::resolver::resolve_with_semantic(
            program,
            Some(crate::semantic_ir::parse_and_adapt(source).unwrap()),
        )
        .unwrap();
        let output = super::generate(&resolved).unwrap();
        assert!(output.contains("ldc 24"), "{output}");
    }

    #[test]
    fn jvm_generation_materializes_const_storage_for_callable_reads() {
        let source = "const limit = 3\nfunction readLimit%()\nreturn limit\nend function\nprint readLimit%()\nend\n";
        let parsed = crate::parse_source("const_storage.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let resolved = crate::resolver::resolve(program).unwrap();
        let output = super::generate(&resolved).unwrap();
        assert!(output.contains(".field public static"), "{output}");
        assert!(output.contains("putstatic"), "{output}");
        assert!(output.contains("getstatic"), "{output}");
    }

    #[test]
    fn jvm_generation_replaces_legacy_unsuffixed_const_slot_with_semantic_slot() {
        let source = "const limit = 3\nconst label = \"ast\"\nfunction read%()\nconst local = 3\nreturn local\nend function\nprint limit; label; read%()\nend\n";
        let semantic_source = "const limit = 7\nconst label = \"typed\"\nfunction read%()\nconst local = 9\nreturn local\nend function\nprint limit; label; read%()\nend\n";
        let parsed = crate::parse_source("semantic_const_slot.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named("semantic_const_slot.bcl", semantic_source)
                .unwrap(),
        );
        let output = super::generate(&resolved).unwrap();
        assert_eq!(
            output.matches(".field public static g").count(),
            2,
            "{output}"
        );
        let main = output
            .split(".method public static main :")
            .nth(1)
            .expect("generated main");
        assert!(main.contains("ldc 7"), "typed CONST value missing: {main}");
        assert!(!main.contains("ldc 3"), "AST CONST value leaked: {main}");
        assert!(
            main.contains("ldc \"typed\""),
            "typed string CONST value missing: {main}"
        );
        assert!(
            !main.contains("ldc \"ast\""),
            "AST string CONST value leaked: {main}"
        );
        let function = output
            .split(".method public static read :")
            .nth(1)
            .expect("generated callable");
        assert!(
            function.contains("ldc 9"),
            "typed callable CONST value missing: {function}"
        );
        assert!(
            !function.contains("ldc 3"),
            "AST callable CONST value leaked: {function}"
        );
    }

    #[test]
    fn jvm_semantic_scalar_writes_preserve_resolved_target_type() {
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "semantic_scalar_target_type.bcl",
            "dim value as long\nvalue = 1\nend\n",
        )
        .unwrap();
        let mut declarations = std::collections::BTreeMap::new();

        super::collect_semantic_scalar_declarations(&semantic, &mut declarations);

        assert_eq!(
            declarations.get("value"),
            Some(&super::JvmType::Numeric(super::NumericType::Long))
        );
    }

    #[test]
    fn jvm_global_slot_uses_typed_global_binding_type() {
        let source = "procedure writer()\nglobal shared%\nend procedure\nwriter()\nend\n";
        let parsed = crate::parse_source("typed_global_slot.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut semantic =
            crate::semantic_ir::parse_and_adapt_named("typed_global_slot.bcl", source).unwrap();
        let crate::semantic_ir::SemanticStatementKind::Line(statements) =
            &mut semantic.callables[0].body[0].kind
        else {
            panic!()
        };
        let crate::semantic_ir::SemanticStatementKind::Global { value_type, .. } =
            &mut statements[0].kind
        else {
            panic!()
        };
        *value_type = crate::semantic_ir::SemanticValueType::Long;
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::generate(&resolved).unwrap();

        assert!(output.contains(".field public static g1 J"), "{output}");
        assert!(!output.contains(".field public static g1 I"), "{output}");
    }

    #[test]
    fn jvm_generation_uses_semantic_scalar_target_descriptor() {
        let legacy_source = "dim value%\nvalue% = 1\nend\n";
        let semantic_source = "dim value as long\nvalue = 1\nend\n";
        let parsed =
            crate::parse_source("semantic_scalar_slot.bcl".to_string(), legacy_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let resolved = crate::resolver::resolve_with_semantic(
            program,
            Some(
                crate::semantic_ir::parse_and_adapt_named(
                    "semantic_scalar_slot.bcl",
                    semantic_source,
                )
                .unwrap(),
            ),
        )
        .unwrap();

        let output = super::generate(&resolved).unwrap();

        let declaration_lines = output
            .lines()
            .filter(|line| line.starts_with(".field public static g"))
            .collect::<Vec<_>>();
        assert!(
            declaration_lines.iter().any(|line| line.ends_with(" J")),
            "{output}"
        );
        assert!(
            !declaration_lines.iter().any(|line| line.ends_with(" D")),
            "{output}"
        );
    }

    #[test]
    fn jvm_callable_local_slot_uses_semantic_dim_type() {
        let legacy_source =
            "procedure setValue()\ndim value%\nvalue% = 1\nend procedure\nsetValue()\nend\n";
        let semantic_source =
            "procedure setValue()\ndim value as long\nvalue = 2\nend procedure\nsetValue()\nend\n";
        let parsed = crate::parse_source(
            "semantic_callable_scalar_slot.bcl".to_string(),
            legacy_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let resolved = crate::resolver::resolve_with_semantic(
            program,
            Some(
                crate::semantic_ir::parse_and_adapt_named(
                    "semantic_callable_scalar_slot.bcl",
                    semantic_source,
                )
                .unwrap(),
            ),
        )
        .unwrap();

        let output = super::generate(&resolved).unwrap();
        let method = output
            .split(".method public static setValue")
            .nth(1)
            .unwrap();
        let method = method.split(".end method").next().unwrap();
        assert!(method.contains("lstore"), "{method}");
        assert!(!method.contains("dstore"), "{method}");
    }

    #[test]
    fn jvm_semantic_field_set_does_not_allocate_field_buffer_slots() {
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "semantic_fixed_string_type.bcl",
            "lset target = \"x\"\nrset other = \"y\"\nend\n",
        )
        .unwrap();
        let mut declarations = std::collections::BTreeMap::new();

        super::collect_semantic_scalar_declarations(&semantic, &mut declarations);

        assert!(!declarations.contains_key("target"));
        assert!(!declarations.contains_key("other"));
    }

    #[test]
    fn jvm_generation_emits_string_field_target_descriptor() {
        let source = "open \"record.dat\" for random as #1 len = 8\nfield #1, 4 as target$\nlset target$ = \"x\"\nend\n";
        let parsed =
            crate::parse_source("semantic_fixed_string_slot.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let resolved = crate::resolver::resolve_with_semantic(
            program,
            Some(crate::semantic_ir::parse_and_adapt(source).unwrap()),
        )
        .unwrap();

        let output = super::generate(&resolved).unwrap();

        assert!(output.contains("Ljava/lang/String;"), "{output}");
        assert!(!output.contains(".field public static g0 D"), "{output}");
    }

    #[test]
    fn jvm_semantic_for_variable_preserves_dim_type() {
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "semantic_for_variable_type.bcl",
            "dim index as long\nfor index = 1 to 2\nprint index\nnext index\nend\n",
        )
        .unwrap();
        let mut declarations = std::collections::BTreeMap::new();

        super::collect_semantic_scalar_declarations(&semantic, &mut declarations);

        assert_eq!(
            declarations.get("index"),
            Some(&super::JvmType::Numeric(super::NumericType::Long))
        );

        let suffixed = crate::semantic_ir::parse_and_adapt("for slot% = 1 to 2\nnext slot\n")
            .unwrap();
        let mut declarations = std::collections::BTreeMap::new();
        super::collect_semantic_scalar_declarations(&suffixed, &mut declarations);
        assert_eq!(
            declarations.get("slot%"),
            Some(&super::JvmType::Numeric(super::NumericType::Int))
        );
    }

    #[test]
    fn jvm_semantic_unannotated_dim_uses_resolver_default_type() {
        let source = "dim value\nvalue = 1\nend\n";
        let parsed = crate::parse_source("semantic_default_dim_type.bcl".to_string(), source)
            .expect("legacy fixture parses");
        let crate::lower::Lowered { program, .. } =
            crate::lower::lower(parsed).expect("fixture lowers");
        let resolved = crate::resolver::resolve_with_semantic(
            program,
            Some(
                crate::semantic_ir::parse_and_adapt_named("semantic_default_dim_type.bcl", source)
                    .expect("semantic fixture adapts"),
            ),
        )
        .expect("fixture resolves");
        let module = resolved.semantic_module.as_ref().unwrap();
        let mut declarations = std::collections::BTreeMap::new();

        super::collect_semantic_scalar_declarations(module, &mut declarations);

        assert_eq!(
            declarations.get("value"),
            Some(&super::JvmType::Numeric(super::NumericType::Double))
        );
    }

    #[test]
    fn jvm_semantic_dim_collectors_do_not_default_unknown_storage_types() {
        let mut module = crate::semantic_ir::parse_and_adapt(
            "dim scalar&\ndim values&(4)\nend\n",
        )
        .expect("typed source parses");
        for statement in &mut module.statements {
            let crate::semantic_ir::SemanticStatementKind::Line(body) = &mut statement.kind else {
                continue;
            };
            for item_statement in body {
                let crate::semantic_ir::SemanticStatementKind::Dim(items) =
                    &mut item_statement.kind
                else {
                    continue;
                };
                for item in items {
                    item.element_type = crate::semantic_ir::SemanticValueType::Unknown;
                }
            }
        }

        let mut scalars = std::collections::BTreeMap::new();
        super::collect_semantic_scalar_declarations(&module, &mut scalars);
        assert!(scalars.is_empty(), "{scalars:?}");

        let mut arrays = std::collections::BTreeMap::new();
        super::collect_semantic_array_declarations(&module, &mut arrays);
        assert!(arrays.is_empty(), "unexpected array declarations: {}", arrays.len());
    }

    #[test]
    fn jvm_semantic_catch_slots_use_typed_binding_types() {
        let mut semantic = crate::semantic_ir::parse_and_adapt(
            "try\nbeep\ncatch error%, line%, file$\nbeep\nend try\n",
        )
        .unwrap();
        let crate::semantic_ir::SemanticStatementKind::Line(statements) =
            &mut semantic.statements[0].kind
        else {
            panic!()
        };
        let crate::semantic_ir::SemanticStatementKind::Try {
            catch: Some(catch), ..
        } = &mut statements[0].kind
        else {
            panic!()
        };
        catch.error = "error$".to_string();
        catch.line = "line&".to_string();
        catch.source = Some("file%".to_string());
        let mut declarations = std::collections::BTreeMap::new();
        super::collect_semantic_scalar_declarations(&semantic, &mut declarations);
        assert_eq!(
            declarations.get("error%"),
            Some(&super::JvmType::Numeric(super::NumericType::Int))
        );
        assert_eq!(
            declarations.get("line%"),
            Some(&super::JvmType::Numeric(super::NumericType::Int))
        );
        assert_eq!(declarations.get("file$"), Some(&super::JvmType::String));

        let suffixless = crate::semantic_ir::parse_and_adapt(
            "try\nbeep\ncatch error, line\nbeep\nend try\n",
        )
        .unwrap();
        let mut declarations = std::collections::BTreeMap::new();
        super::collect_semantic_scalar_declarations(&suffixless, &mut declarations);
        assert_eq!(
            declarations.get("error!"),
            Some(&super::JvmType::Numeric(super::NumericType::Double))
        );
        assert_eq!(
            declarations.get("line!"),
            Some(&super::JvmType::Numeric(super::NumericType::Double))
        );
    }

    #[test]
    fn jvm_rejects_semantic_long_for_variable_with_capability_diagnostic() {
        let source = "dim index as long\nfor index = 1 to 2\nprint index\nnext\nend\n";
        let parsed = crate::parse_source("semantic_long_for.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let resolved = crate::resolver::resolve_with_semantic(
            program,
            Some(crate::semantic_ir::parse_and_adapt(source).unwrap()),
        )
        .unwrap();

        let diagnostics = super::generate(&resolved).expect_err("JVM long FOR is unsupported");

        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("integer FOR variables only")),
            "{diagnostics:?}"
        );
    }

    #[test]
    fn semantic_array_declarations_preserve_jvm_shapes() {
        let module = crate::semantic_ir::parse_and_adapt("dim values%(10, 20)\n").unwrap();
        let mut arrays = std::collections::BTreeMap::new();
        super::collect_semantic_array_declarations(&module, &mut arrays);
        assert_eq!(
            arrays["values%"].dimensions,
            [
                super::ArrayDimension::Typed(10),
                super::ArrayDimension::Typed(20)
            ]
        );
        let inferred = crate::semantic_ir::parse_and_adapt("dim dynamic%(?)\n").unwrap();
        let mut inferred_arrays = std::collections::BTreeMap::new();
        super::collect_semantic_array_declarations(&inferred, &mut inferred_arrays);
        assert!(inferred_arrays.is_empty());
    }

    #[test]
    fn semantic_callable_array_declarations_preserve_jvm_shapes() {
        let module = crate::semantic_ir::parse_and_adapt(
            "function f%()\ndim values%(10)\nreturn 0\nend function\n",
        )
        .unwrap();
        let mut arrays = std::collections::BTreeMap::new();
        super::collect_semantic_array_declarations(&module, &mut arrays);
        assert!(arrays.is_empty());
        let mut body_module = module.clone();
        body_module.statements = module.callables[0].body.clone();
        super::collect_semantic_array_declarations(&body_module, &mut arrays);
        assert_eq!(
            arrays["values%"].dimensions,
            [super::ArrayDimension::Typed(10)]
        );
    }
}
