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
//! output still uses `krak2` to assemble `.j` -> `.class`.
//! and any JRE's `java` running it (see `main.rs`'s eventual
//! `invoke_krak2`/`invoke_java`). `krak2` is `Storyyeller/Krakatau`'s `v2`
//! branch -- itself a Rust/Cargo project, pinned by commit since it has no
//! versioned releases (confirmed working end to end with a hand-written
//! `.j` file: `krak2 asm` + `java` before this codegen existed).
//!
//! The generated classes deliberately use class-file version 50.  That lets
//! the JVM's legacy verifier infer frames for the supported `if` branches,
//! instead of requiring StackMapTable emission before this backend has a full
//! control-flow frame analyser. Scalar local slots are allocated up front and
//! `.limit locals` is computed from them.

use std::cell::Cell;
use std::collections::{BTreeMap, HashMap, HashSet};

use crate::ast::{
    BasicIdent, BinaryOp, CaseValue, Expr, FunctionDef, ParamMode, PrintToken, Program, Statement,
    Stmt, TypeSuffix, UnaryOp,
};
use crate::diagnostics::{Diagnostic, SourcePos};

pub(crate) fn generate(program: &Program) -> Result<String, Vec<Diagnostic>> {
    let class_name = class_name_for(program);
    let functions = function_table(&program.functions);
    let context = JvmContext::build(program, functions.clone(), class_name.clone())?;
    let mut body = String::new();
    context.emit_initializers(&mut body);
    emit_array_initializers(&context, &mut body).map_err(|message| vec![unsupported(&message)])?;
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
    fields
}

/// Deep-clones a nested integer array.  BASCAL permits at most eight axes;
/// the caller checks that limit before invoking this helper.  Arrays of rank
/// two or greater are `Object[]` at each outer level, so a shallow clone plus
/// recursive replacement of every child preserves the concrete array class.
fn emit_array_copy_helper(class_name: &str) -> String {
    format!(".method private static bccCopyArray : (Ljava/lang/Object;II)Ljava/lang/Object;\n    .limit stack 5\n    .limit locals 5\n\n    iload 1\n    iconst_1\n    if_icmpne L_copy_nested\n    iload 2\n    tableswitch 0\n        L_copy_int\n        L_copy_long\n        L_copy_double\n        L_copy_object\nL_copy_int:\n    aload 0\n    checkcast [I\n    invokevirtual [I/clone ()Ljava/lang/Object;\n    areturn\nL_copy_long:\n    aload 0\n    checkcast [J\n    invokevirtual [J/clone ()Ljava/lang/Object;\n    areturn\nL_copy_double:\n    aload 0\n    checkcast [D\n    invokevirtual [D/clone ()Ljava/lang/Object;\n    areturn\nL_copy_object:\n    aload 0\n    checkcast [Ljava/lang/Object;\n    invokevirtual [Ljava/lang/Object;/clone ()Ljava/lang/Object;\n    areturn\nL_copy_nested:\n    aload 0\n    checkcast [Ljava/lang/Object;\n    invokevirtual [Ljava/lang/Object;/clone ()Ljava/lang/Object;\n    checkcast [Ljava/lang/Object;\n    astore 3\n    iconst_0\n    istore 4\nL_copy_loop:\n    iload 4\n    aload 3\n    arraylength\n    if_icmpge L_copy_done\n    aload 3\n    iload 4\n    aload 3\n    iload 4\n    aaload\n    iload 1\n    iconst_1\n    isub\n    iload 2\n    invokestatic {class_name}/bccCopyArray (Ljava/lang/Object;II)Ljava/lang/Object;\n    aastore\n    iinc 4 1\n    goto L_copy_loop\nL_copy_done:\n    aload 3\n    areturn\n.end method\n\n")
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
                _ => {}
            }
        }
    }
    visit(statements, &mut labels);
    labels
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
        body.push_str("    return\n");
    } else {
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
            Statement::Color { fg, bg } => {
                let Expr::Integer(fg) = fg else {
                    return Err(
                        "JVM COLOR currently requires a literal foreground value".to_string()
                    );
                };
                let code = if let Some(Expr::Integer(bg)) = bg {
                    format!(
                        "\u{1b}[{};{}m",
                        30 + (*fg as i32 % 8),
                        40 + (*bg as i32 % 8)
                    )
                } else if bg.is_none() {
                    format!("\u{1b}[{}m", 30 + (*fg as i32 % 8))
                } else {
                    return Err(
                        "JVM COLOR currently requires a literal background value".to_string()
                    );
                };
                emit_terminal_escape(&code, out)
            }
            Statement::Locate { row, col } => {
                let (Expr::Integer(row), Expr::Integer(col)) = (row, col) else {
                    return Err(
                        "JVM LOCATE currently requires literal row and column values".to_string(),
                    );
                };
                emit_terminal_escape(&format!("\u{1b}[{};{}H", row, col), out)
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
                if !signature.returns_void || signature.source_param_count != args.len() {
                    return Err(format!("invalid JVM procedure call `{name}`"));
                }
                let mut scalars = signature.params.iter();
                for (position, arg) in args.iter().enumerate() {
                    if let Some(array) = signature
                        .array_params
                        .iter()
                        .find(|array| array.position == position)
                    {
                        let Expr::Ident(array_name) = arg else {
                            return Err(format!("array parameter {position} of `{name}` needs a plain array argument"));
                        };
                        let shape = self
                            .context
                            .arrays
                            .get(&variable_key(array_name))
                            .ok_or_else(|| format!("unknown JVM array `{array_name}`"))?;
                        if shape.dimensions.len() != array.rank || shape.element != array.element {
                            return Err(format!("array argument `{array_name}` doesn't match `{name}`'s parameter rank/type"));
                        }
                        self.context.emit_array_load(array_name, out);
                    } else {
                        let ty = scalars.next().expect("scalar source parameter");
                        match ty {
                            JvmType::String => emit_string_expr(arg, out, self.context)?,
                            JvmType::Numeric(ty) => {
                                emit_numeric_expr_as(arg, *ty, out, self.context)?
                            }
                        }
                    }
                }
                let args_descriptor = signature.descriptor_params().join("");
                out.push_str(&format!(
                    "    invokestatic {}/{} ({args_descriptor})V\n",
                    self.context.class_name, name.name
                ));
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
                    out.push_str("    areturn\n");
                    Ok(())
                }
                Some(JvmType::Numeric(ty)) => {
                    emit_numeric_expr_as(value, ty, out, self.context)?;
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

/// Emits each PRINT value directly to `System.out`.  Separators currently
/// follow the bootstrap C backend's simple rule: they only suppress the
/// final newline; they do not implement BASCOM's tab-zone formatting.
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
        let descriptor = if context.is_string_expr(expr) {
            emit_string_expr(expr, out, context)?;
            "(Ljava/lang/String;)V"
        } else {
            emit_numeric_expr(expr, out, context)?.print_descriptor()
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
    Ok(())
}

fn emit_terminal_escape(value: &str, out: &mut String) -> Result<(), String> {
    out.push_str(&format!(
        "    getstatic java/lang/System/out Ljava/io/PrintStream;\n    ldc \"{}\"\n    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V\n",
        escape_jvm_string(value)
    ));
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
}

#[derive(Clone)]
struct FunctionSig {
    params: Vec<JvmType>,
    array_params: Vec<JvmArrayParam>,
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
                out.push(descriptor(*scalars.next().expect("scalar source parameter")).to_string());
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
        collect_scalar_declarations(&program.statements, &mut declarations, &mut constants);
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
        for param in &function.params {
            if param.axes.is_some() {
                parameter_array_slots.insert(variable_key(&param.name), next_slot);
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
        let initializer_start = next_slot;
        let mut declarations = BTreeMap::new();
        let mut constants = parent.constants.clone();
        collect_scalar_declarations(&function.body, &mut declarations, &mut constants);
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
        }
    }

    fn local_count(&self) -> usize {
        self.local_count
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
        self.constants.get(&variable_key(ident))
    }

    fn function(&self, ident: &BasicIdent) -> Option<FunctionSig> {
        self.functions.get(&function_key(ident)).cloned()
    }

    fn is_string_expr(&self, expr: &Expr) -> bool {
        match expr {
            Expr::String(_) => true,
            Expr::Ident(name) => {
                self.constant(name)
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
            Statement::Const { name, value } => {
                constants.insert(variable_key(name), value.clone());
            }
            Statement::If {
                then_body,
                else_body,
                ..
            } => {
                collect_scalar_declarations(then_body, declarations, constants);
                collect_scalar_declarations(else_body, declarations, constants);
            }
            Statement::For { var, body, .. } => {
                declarations.insert(variable_key(var), type_for_ident(var));
                collect_scalar_declarations(body, declarations, constants);
            }
            Statement::While { body, .. } | Statement::Do { body, .. } => {
                collect_scalar_declarations(body, declarations, constants);
            }
            Statement::TryCatch {
                try_body,
                catch,
                finally_body,
            } => {
                collect_scalar_declarations(try_body, declarations, constants);
                if let Some(catch) = catch {
                    declarations
                        .insert(variable_key(&catch.err_var), type_for_ident(&catch.err_var));
                    declarations
                        .insert(variable_key(&catch.erl_var), type_for_ident(&catch.erl_var));
                    if let Some(source_var) = &catch.source_var {
                        declarations.insert(variable_key(source_var), JvmType::String);
                    }
                    collect_scalar_declarations(&catch.body, declarations, constants);
                }
                collect_scalar_declarations(finally_body, declarations, constants);
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
            out.push_str("    newarray char\n    dup\n    bipush 32\n    invokestatic java/util/Arrays/fill ([CC)V\n    new java/lang/String\n    dup_x1\n    swap\n    invokespecial java/lang/String/<init> ([C)V\n");
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
            let descriptor = match ty {
                NumericType::Int => "(I)Ljava/lang/String;",
                NumericType::Long => "(J)Ljava/lang/String;",
                NumericType::Double => "(D)Ljava/lang/String;",
            };
            out.push_str(&format!("    invokestatic java/lang/String/valueOf {descriptor}\n"));
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
    let mut scalars = signature.params.iter();
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
            let ty = scalars.next().expect("scalar source parameter");
            match ty {
                JvmType::String => emit_string_expr(arg, out, context)?,
                JvmType::Numeric(ty) => emit_numeric_expr_as(arg, *ty, out, context)?,
            }
        }
    }
    let args = signature.descriptor_params().join("");
    out.push_str(&format!(
        "    invokestatic {}/{} ({args}){}\n",
        context.class_name,
        name.name,
        descriptor(signature.result)
    ));
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
