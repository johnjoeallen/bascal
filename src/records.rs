//! Lowers the high-level `record`/`file` DSL into the low-level random-access
//! file primitives (`OPEN ... FOR RANDOM`, `FIELD`, `GET`, `PUT`, `LSET`,
//! `MKx`/`CVx`) that `codegen.rs` already knows how to emit. This pass always
//! runs between parsing and `resolver::validate`; by the time codegen sees a
//! `Program`, no `RecordDef`, `Statement::FileDecl`, or DSL `Expr` variant
//! (`FileIndex`, `FieldAccess`, `MethodCall`, `RecordLit`) remains.

use std::collections::{HashMap, HashSet};

use crate::ast::*;
use crate::codegen::camel_join;
use crate::diagnostics::{Diagnostic, SourcePos};

/// Lowers the DSL and returns the rewritten program alongside the
/// lowercase BASIC names of every buffer variable this pass invented
/// (see `Lowerer::synthesized_buffer_names`) -- `codegen.rs` needs that
/// set to know which `FIELD` buffer names are transpiler-built camelCase
/// (case preserved) versus author-typed raw BASIC (still normalized to
/// lowercase, same as every other identifier).
pub fn lower(
    program: Program,
) -> Result<(Program, std::collections::HashSet<String>), Vec<Diagnostic>> {
    let mut lowerer = Lowerer::new();
    lowerer.build_record_table(&program.records);
    lowerer.build_user_method_table(&program.functions);

    let statements = lowerer.lower_statements(program.statements);
    let functions = program
        .functions
        .into_iter()
        .map(|mut f| {
            lowerer.enter_function(&f);
            f.body = lowerer.lower_statements(f.body);
            lowerer.leave_function();
            f
        })
        .collect();

    if !lowerer.diagnostics.is_empty() {
        return Err(lowerer.diagnostics);
    }

    Ok((
        Program {
            program_decl: program.program_decl,
            library_decl: program.library_decl,
            shared_decl: program.shared_decl,
            declarations: program.declarations,
            common: program.common,
            statements,
            functions,
            records: Vec::new(),
            typed_arrays: program.typed_arrays,
            typed_array_refs: program.typed_array_refs,
        },
        lowerer.synthesized_buffer_names,
    ))
}

#[derive(Clone)]
struct FieldSpec {
    name: String,
    ty: RecordFieldType,
}

#[derive(Clone)]
struct RecordType {
    fields: Vec<FieldSpec>,
    width: u32,
}

#[derive(Clone)]
struct FileInfo {
    channel: i64,
    kind: FileKind,
    /// `None` means a top-level file. A file declared inside a callable is
    /// visible only in that callable, even though lowering ultimately uses
    /// target-level channel and FIELD-buffer storage.
    owner: Option<String>,
}

/// What a `file` variable actually is: the random-access record/file DSL
/// (`file db as Student = open(...)`), or a plain sequential file handle
/// (`file scores = open(...) for output`). Tracking the sequential form's
/// `OpenMode` lets `.write(...)`/`.read(...)`/`.eof()` be rejected at
/// compile time against the wrong direction of file -- reading from a
/// file opened `for output`, say -- the same way the record DSL already
/// catches a misspelled or missing field before it ever reaches disk.
#[derive(Clone)]
enum FileKind {
    Record(String),
    Sequential(OpenMode),
}

#[derive(Clone, Copy, PartialEq)]
enum FieldKind {
    Numeric,
    Stringy,
}

struct Lowerer {
    /// Record type table, keyed by lowercase record name.
    records: HashMap<String, RecordType>,
    /// Declared `file` variables, keyed by lowercase variable name.
    files: HashMap<String, FileInfo>,
    /// `let <name> = <file>[<idx>]` bindings, keyed by lowercase variable
    /// name, mapping to the record type name they materialize.
    record_vars: HashMap<String, String>,
    next_channel: i64,
    diagnostics: Vec<Diagnostic>,
    /// Lowercase BASIC names of every buffer variable this pass invented
    /// via `buffer_ident` -- as opposed to a `FIELD` buffer name the
    /// author typed directly in raw-BASIC-passthrough source. Threaded
    /// back to `codegen.rs` so it can tell the two apart: a
    /// transpiler-synthesized buffer name is already deliberately
    /// camelCased and should keep that case, while a user-typed one still
    /// gets BASCAL's normal lowercase normalization.
    synthesized_buffer_names: std::collections::HashSet<String>,
    /// A user-declared scalar method's own declared
    /// result type, keyed by (receiver suffix, lowercase method name) --
    /// built once, up front, from `program.functions` (before it's moved
    /// into per-function lowering) since `rewrite_expr`'s own
    /// `Expr::ScalarMethodCall` arm needs a chain's inner receiver type
    /// resolved *before* it can decide whether the outer `.method()` is one
    /// of the built-in scalar methods (see `scalar_builtins.rs`) -- e.g. in
    /// `s$.trim().left(3)`, resolving `.left()` needs `.trim()`'s own
    /// declared result type, which only a user scalar method declaration (not
    /// this pass) provides.
    user_method_results: HashMap<(TypeSuffix, String), TypeSuffix>,
    /// Every `(lowercase name, suffix)` pair already claimed by an ordinary
    /// (non-method) function -- used by `rewrite_expr`'s `Expr::Call`/
    /// `Expr::ArrayRef` arms to know when an ordinary call is safe to fall
    /// back to a same-named method (see `try_ordinary_call_as_method`'s own
    /// doc comment): the fallback must never hijack a name a real function
    /// already legitimately owns. A real BASIC builtin (`BASIC_BUILTINS`)
    /// is checked separately, suffix-independently, in
    /// `try_ordinary_call_as_method` itself -- a builtin name is reserved
    /// regardless of which suffix a call site happens to use (`sqr%` and
    /// `sqr$` both collide with `SQR`, same rule
    /// `reject_functions_shadowing_builtins` already enforces for user
    /// declarations), unlike this set, which is genuinely suffix-specific.
    known_ordinary_functions: HashSet<(String, Option<TypeSuffix>)>,
    /// The callable whose body is currently being lowered, plus the file
    /// names it explicitly exposes with `global`. `file` DSL nodes disappear
    /// before resolver scoping runs, so this pass must enforce the same
    /// local-unless-global rule itself.
    current_function: Option<String>,
    current_file_globals: HashSet<String>,
    current_statement_pos: Option<SourcePos>,
}

impl Lowerer {
    fn new() -> Self {
        Self {
            records: HashMap::new(),
            files: HashMap::new(),
            record_vars: HashMap::new(),
            next_channel: 1,
            diagnostics: Vec::new(),
            synthesized_buffer_names: std::collections::HashSet::new(),
            user_method_results: HashMap::new(),
            known_ordinary_functions: HashSet::new(),
            current_function: None,
            current_file_globals: HashSet::new(),
            current_statement_pos: None,
        }
    }

    fn build_user_method_table(&mut self, functions: &[FunctionDef]) {
        for function in functions {
            if function.receiver.is_none() {
                self.known_ordinary_functions.insert((
                    function.name.name.to_ascii_lowercase(),
                    function.name.suffix,
                ));
            }
        }
        for function in functions {
            let Some(receiver) = function.receiver else {
                continue;
            };
            let Some(result) = function.name.suffix else {
                continue;
            };
            self.user_method_results
                .insert((receiver, function.name.name.to_ascii_lowercase()), result);
        }
    }

    fn build_record_table(&mut self, records: &[RecordDef]) {
        let mut seen_names = std::collections::HashSet::new();
        for rec in records {
            let key = rec.name.to_ascii_lowercase();
            if !seen_names.insert(key.clone()) {
                self.diagnostics.push(Diagnostic::error(
                    generated_pos(),
                    format!("duplicate record type `{}`", rec.name),
                ));
                continue;
            }
            let mut seen_fields = std::collections::HashSet::new();
            let mut fields = Vec::new();
            let mut width = 0u32;
            for f in &rec.fields {
                if !seen_fields.insert(f.name.to_ascii_lowercase()) {
                    self.diagnostics.push(Diagnostic::error(
                        generated_pos(),
                        format!("duplicate field `{}` in record `{}`", f.name, rec.name),
                    ));
                    continue;
                }
                width += field_width(&f.ty);
                fields.push(FieldSpec {
                    name: f.name.clone(),
                    ty: f.ty,
                });
            }
            self.records.insert(key, RecordType { fields, width });
        }
    }

    // ── statement lowering ──────────────────────────────────────────────

    fn lower_statements(&mut self, stmts: Vec<Stmt>) -> Vec<Stmt> {
        let mut out = Vec::new();
        for stmt in stmts {
            self.lower_statement(stmt, &mut out);
        }
        out
    }

    fn lower_statement(&mut self, stmt: Stmt, out: &mut Vec<Stmt>) {
        let pos = stmt.pos.clone();
        self.current_statement_pos = Some(pos.clone());
        match stmt.kind {
            // These three delegate to helpers that may synthesize several
            // low-level statements from one high-level DSL statement (e.g.
            // `file db as T = open(...)` -> a comment + `OPEN` + `FIELD`).
            // All of them share the DSL statement's own position -- good
            // enough for diagnostics, since none of resolver.rs's checks
            // run on the DSL form (records::lower always runs first).
            Statement::FileDecl {
                var,
                record_type,
                path,
                mode,
            } => {
                let mut raw = Vec::new();
                self.lower_file_decl(var, record_type, path, mode, &mut raw);
                out.extend(raw.into_iter().map(|s| Stmt::new(s, pos.clone())));
            }
            // A `file` object is erased into a fixed channel and (for a
            // record file) FIELD buffers. `global fileName` is therefore a
            // source-scope declaration only: preserve it as an explanatory
            // comment for BASIC output, but do not leave a fake suffixless
            // scalar for either backend to allocate.
            Statement::GlobalDecl(ident)
                if self.current_function.is_some()
                    && self.files.contains_key(&ident.name.to_ascii_lowercase()) =>
            {
                out.push(Stmt::new(
                    Statement::Raw(format!("' global {}", ident.name)),
                    pos,
                ));
            }
            Statement::Assignment { target, value } => {
                let mut raw = Vec::new();
                self.lower_assignment(target, value, &mut raw);
                out.extend(raw.into_iter().map(|s| Stmt::new(s, pos.clone())));
            }
            Statement::ExprStmt(expr) => {
                let mut raw = Vec::new();
                self.lower_expr_stmt(expr, &mut raw);
                out.extend(raw.into_iter().map(|s| Stmt::new(s, pos.clone())));
            }
            Statement::If {
                condition,
                then_body,
                else_body,
            } => {
                let condition = self.rewrite_expr(condition).0;
                let then_body = self.lower_statements(then_body);
                let else_body = self.lower_statements(else_body);
                out.push(Stmt::new(
                    Statement::If {
                        condition,
                        then_body,
                        else_body,
                    },
                    pos,
                ));
            }
            Statement::For {
                var,
                start,
                end,
                step,
                body,
            } => {
                let start = self.rewrite_expr(start).0;
                let end = self.rewrite_expr(end).0;
                let step = step.map(|s| self.rewrite_expr(s).0);
                let body = self.lower_statements(body);
                out.push(Stmt::new(
                    Statement::For {
                        var,
                        start,
                        end,
                        step,
                        body,
                    },
                    pos,
                ));
            }
            Statement::While { condition, body } => {
                let condition = self.rewrite_expr(condition).0;
                let body = self.lower_statements(body);
                out.push(Stmt::new(Statement::While { condition, body }, pos));
            }
            Statement::Do {
                condition,
                body,
                post_condition,
            } => {
                let condition = condition.map(|c| self.rewrite_do_condition(c));
                let body = self.lower_statements(body);
                let post_condition = post_condition.map(|c| self.rewrite_do_condition(c));
                out.push(Stmt::new(
                    Statement::Do {
                        condition,
                        body,
                        post_condition,
                    },
                    pos,
                ));
            }
            Statement::SelectCase {
                expr,
                cases,
                else_body,
            } => {
                let expr = self.rewrite_expr(expr).0;
                let cases = cases
                    .into_iter()
                    .map(|c| CaseClause {
                        values: c
                            .values
                            .into_iter()
                            .map(|v| self.rewrite_case_value(v))
                            .collect(),
                        body: self.lower_statements(c.body),
                    })
                    .collect();
                let else_body = self.lower_statements(else_body);
                out.push(Stmt::new(
                    Statement::SelectCase {
                        expr,
                        cases,
                        else_body,
                    },
                    pos,
                ));
            }
            Statement::TryCatch {
                try_body,
                catch,
                finally_body,
            } => {
                let try_body = self.lower_statements(try_body);
                let catch = catch.map(|catch| TryCatchHandler {
                    err_var: catch.err_var,
                    error_filter: catch
                        .error_filter
                        .into_iter()
                        .map(|e| self.rewrite_expr(e).0)
                        .collect(),
                    erl_var: catch.erl_var,
                    source_var: catch.source_var,
                    body: self.lower_statements(catch.body),
                });
                let finally_body = self.lower_statements(finally_body);
                out.push(Stmt::new(
                    Statement::TryCatch {
                        try_body,
                        catch,
                        finally_body,
                    },
                    pos,
                ));
            }
            other => out.push(Stmt::new(self.rewrite_statement_exprs(other), pos)),
        }
    }

    fn enter_function(&mut self, function: &FunctionDef) {
        self.current_function = Some(function.name.name.to_ascii_lowercase());
        self.current_file_globals = collect_global_names(&function.body);
    }

    fn leave_function(&mut self) {
        self.current_function = None;
        self.current_file_globals.clear();
    }

    fn rewrite_do_condition(&mut self, cond: DoCondition) -> DoCondition {
        DoCondition {
            is_while: cond.is_while,
            expr: self.rewrite_expr(cond.expr).0,
        }
    }

    fn rewrite_case_value(&mut self, value: CaseValue) -> CaseValue {
        match value {
            CaseValue::Single(e) => CaseValue::Single(self.rewrite_expr(e).0),
            CaseValue::Range { from, to } => CaseValue::Range {
                from: self.rewrite_expr(from).0,
                to: self.rewrite_expr(to).0,
            },
            CaseValue::Is { op, value } => CaseValue::Is {
                op,
                value: self.rewrite_expr(value).0,
            },
        }
    }

    fn rewrite_print_tokens(&mut self, tokens: Vec<PrintToken>) -> Vec<PrintToken> {
        tokens
            .into_iter()
            .map(|t| match t {
                PrintToken::Expr(e) => PrintToken::Expr(self.rewrite_expr(e).0),
                other => other,
            })
            .collect()
    }

    /// Rewrites every embedded `Expr` in a statement that isn't one of the
    /// five record/file DSL shapes handled directly in `lower_statement`.
    /// Structured exactly like `resolver.rs`'s `statement_calls_function` —
    /// exhaustive over every remaining `Statement` variant.
    fn rewrite_statement_exprs(&mut self, stmt: Statement) -> Statement {
        match stmt {
            Statement::Dim {
                name,
                is_array,
                sizes,
            } => {
                let sizes = sizes.into_iter().map(|e| self.rewrite_expr(e).0).collect();
                Statement::Dim {
                    name,
                    is_array,
                    sizes,
                }
            }
            Statement::Open {
                mode,
                file,
                channel,
                len,
            } => {
                let file = self.rewrite_expr(file).0;
                let channel = self.rewrite_expr(channel).0;
                let len = len.map(|e| self.rewrite_expr(e).0);
                Statement::Open {
                    mode,
                    file,
                    channel,
                    len,
                }
            }
            Statement::LineInput { channel, target } => {
                let channel = self.rewrite_expr(channel).0;
                let target = self.rewrite_expr(target).0;
                Statement::LineInput { channel, target }
            }
            Statement::PrintFile { channel, tokens } => {
                let channel = self.rewrite_expr(channel).0;
                let tokens = self.rewrite_print_tokens(tokens);
                Statement::PrintFile { channel, tokens }
            }
            Statement::PrintUsing { format, tokens } => {
                let format = self.rewrite_expr(format).0;
                let tokens = self.rewrite_print_tokens(tokens);
                Statement::PrintUsing { format, tokens }
            }
            Statement::PrintFileUsing {
                channel,
                format,
                tokens,
            } => {
                let channel = self.rewrite_expr(channel).0;
                let format = self.rewrite_expr(format).0;
                let tokens = self.rewrite_print_tokens(tokens);
                Statement::PrintFileUsing {
                    channel,
                    format,
                    tokens,
                }
            }
            Statement::Close { channel } => Statement::Close {
                channel: self.rewrite_expr(channel).0,
            },
            Statement::Kill { file } => Statement::Kill {
                file: self.rewrite_expr(file).0,
            },
            Statement::Name { from, to } => {
                let from = self.rewrite_expr(from).0;
                let to = self.rewrite_expr(to).0;
                Statement::Name { from, to }
            }
            Statement::Print { tokens } => Statement::Print {
                tokens: self.rewrite_print_tokens(tokens),
            },
            Statement::Return { value } => Statement::Return {
                value: self.rewrite_expr(value).0,
            },
            Statement::MidAssign {
                target,
                start,
                len,
                value,
            } => {
                let target = Box::new(self.rewrite_expr(*target).0);
                let start = Box::new(self.rewrite_expr(*start).0);
                let len = len.map(|e| Box::new(self.rewrite_expr(*e).0));
                let value = Box::new(self.rewrite_expr(*value).0);
                Statement::MidAssign {
                    target,
                    start,
                    len,
                    value,
                }
            }
            Statement::OptionBase(e) => Statement::OptionBase(self.rewrite_expr(e).0),
            Statement::Randomize(e) => Statement::Randomize(e.map(|e| self.rewrite_expr(e).0)),
            Statement::Swap(a, b) => {
                Statement::Swap(self.rewrite_expr(a).0, self.rewrite_expr(b).0)
            }
            Statement::Poke { address, value } => {
                let address = self.rewrite_expr(address).0;
                let value = self.rewrite_expr(value).0;
                Statement::Poke { address, value }
            }
            Statement::Goto(e) => Statement::Goto(self.rewrite_expr(e).0),
            Statement::Gosub(e) => Statement::Gosub(self.rewrite_expr(e).0),
            Statement::OnErrorGoto { target } => Statement::OnErrorGoto {
                target: self.rewrite_expr(target).0,
            },
            Statement::Resume(kind) => {
                let kind = match kind {
                    ResumeTarget::Line(e) => ResumeTarget::Line(self.rewrite_expr(e).0),
                    other => other,
                };
                Statement::Resume(kind)
            }
            Statement::ErrorStmt { code } => Statement::ErrorStmt {
                code: self.rewrite_expr(code).0,
            },
            Statement::ThrowStmt { code } => Statement::ThrowStmt {
                code: code.map(|code| self.rewrite_expr(code).0),
            },
            Statement::Input { prompt, vars } => {
                let vars = vars.into_iter().map(|e| self.rewrite_expr(e).0).collect();
                Statement::Input { prompt, vars }
            }
            Statement::InputFile { channel, vars } => {
                let channel = self.rewrite_expr(channel).0;
                let vars = vars.into_iter().map(|e| self.rewrite_expr(e).0).collect();
                Statement::InputFile { channel, vars }
            }
            Statement::Data(values) => {
                Statement::Data(values.into_iter().map(|e| self.rewrite_expr(e).0).collect())
            }
            Statement::Read(vars) => {
                Statement::Read(vars.into_iter().map(|e| self.rewrite_expr(e).0).collect())
            }
            Statement::Restore(target) => {
                Statement::Restore(target.map(|e| self.rewrite_expr(e).0))
            }
            Statement::Const { name, value } => Statement::Const {
                name,
                value: self.rewrite_expr(value).0,
            },
            Statement::Write { channel, exprs } => {
                let channel = self.rewrite_expr(channel).0;
                let exprs = exprs.into_iter().map(|e| self.rewrite_expr(e).0).collect();
                Statement::Write { channel, exprs }
            }
            Statement::Field {
                channel,
                fields,
                record_type,
                string_fields,
                field_types,
            } => {
                let channel = self.rewrite_expr(channel).0;
                let fields = fields
                    .into_iter()
                    .map(|(w, v)| (self.rewrite_expr(w).0, v))
                    .collect();
                Statement::Field {
                    channel,
                    fields,
                    record_type,
                    string_fields,
                    field_types,
                }
            }
            Statement::Get {
                channel,
                record,
                var,
                require_existing,
                record_length,
            } => {
                let channel = self.rewrite_expr(channel).0;
                let record = record.map(|e| self.rewrite_expr(e).0);
                let var = var.map(|e| self.rewrite_expr(e).0);
                Statement::Get {
                    channel,
                    record,
                    var,
                    require_existing,
                    record_length,
                }
            }
            Statement::Put {
                channel,
                record,
                var,
                provided_fields,
            } => {
                let channel = self.rewrite_expr(channel).0;
                let record = record.map(|e| self.rewrite_expr(e).0);
                let var = var.map(|e| self.rewrite_expr(e).0);
                Statement::Put {
                    channel,
                    record,
                    var,
                    provided_fields,
                }
            }
            Statement::Lset { var, value } => Statement::Lset {
                var,
                value: self.rewrite_expr(value).0,
            },
            Statement::Rset { var, value } => Statement::Rset {
                var,
                value: self.rewrite_expr(value).0,
            },
            Statement::Seek { channel, position } => {
                let channel = self.rewrite_expr(channel).0;
                let position = self.rewrite_expr(position).0;
                Statement::Seek { channel, position }
            }
            Statement::Lprint(tokens) => Statement::Lprint(self.rewrite_print_tokens(tokens)),
            Statement::LprintUsing { format, tokens } => {
                let format = self.rewrite_expr(format).0;
                let tokens = self.rewrite_print_tokens(tokens);
                Statement::LprintUsing { format, tokens }
            }
            Statement::Locate { row, col } => {
                let row = self.rewrite_expr(row).0;
                let col = self.rewrite_expr(col).0;
                Statement::Locate { row, col }
            }
            Statement::Color { fg, bg } => {
                let fg = self.rewrite_expr(fg).0;
                let bg = bg.map(|e| self.rewrite_expr(e).0);
                Statement::Color { fg, bg }
            }
            Statement::OnBranch {
                expr,
                targets,
                is_gosub,
            } => {
                let expr = self.rewrite_expr(expr).0;
                let targets = targets
                    .into_iter()
                    .map(|e| self.rewrite_expr(e).0)
                    .collect();
                Statement::OnBranch {
                    expr,
                    targets,
                    is_gosub,
                }
            }
            Statement::Out { port, value } => {
                let port = self.rewrite_expr(port).0;
                let value = self.rewrite_expr(value).0;
                Statement::Out { port, value }
            }
            Statement::Width { channel, cols } => {
                let channel = channel.map(|e| self.rewrite_expr(e).0);
                let cols = self.rewrite_expr(cols).0;
                Statement::Width { channel, cols }
            }
            Statement::Erase(_)
            | Statement::End
            | Statement::Stop
            | Statement::Cls
            | Statement::Beep
            | Statement::Clear
            | Statement::System
            | Statement::Exit
            | Statement::ReturnVoid
            | Statement::GlobalDecl(_)
            | Statement::Raw(_)
            | Statement::BlockComment(_)
            | Statement::Label(_)
            | Statement::BlankLine => stmt,
            Statement::FileDecl { .. }
            | Statement::Assignment { .. }
            | Statement::ExprStmt(_)
            | Statement::If { .. }
            | Statement::For { .. }
            | Statement::While { .. }
            | Statement::Do { .. }
            | Statement::SelectCase { .. }
            | Statement::TryCatch { .. } => {
                unreachable!("handled directly in lower_statement")
            }
        }
    }

    // ── the five record/file DSL shapes ─────────────────────────────────

    fn lower_file_decl(
        &mut self,
        var: BasicIdent,
        record_type: Option<String>,
        path: Expr,
        mode: Option<OpenMode>,
        out: &mut Vec<Statement>,
    ) {
        let path = self.rewrite_expr(path).0;
        let var_key = var.name.to_ascii_lowercase();
        if self.files.contains_key(&var_key) {
            self.diagnostics.push(Diagnostic::error(
                generated_pos(),
                format!("file `{}` is already declared", var.name),
            ));
            return;
        }

        let Some(record_type) = record_type else {
            // `file scores = open(...) for output/input/append` -- the
            // plain sequential-handle form. No FIELD layout involved at
            // all; `.write(...)`/`.read(...)`/`.eof()`/`.close()` lower
            // straight to the channel number this allocates.
            let mode = mode.expect("parser only omits `record_type` alongside a `mode`");
            let channel = self.next_channel;
            self.next_channel += 1;
            let mode_word = match mode {
                OpenMode::Input => "input",
                OpenMode::Output => "output",
                OpenMode::Append => "append",
                OpenMode::Random | OpenMode::Binary => {
                    unreachable!("the file/open DSL sugar never parses a random/binary mode")
                }
            };
            out.push(Statement::Raw(format!(
                "' file {} = open(...) for {mode_word}",
                var.name
            )));
            self.files.insert(
                var_key,
                FileInfo {
                    channel,
                    kind: FileKind::Sequential(mode),
                    owner: self.current_function.clone(),
                },
            );
            out.push(Statement::Open {
                mode,
                file: path,
                channel: Expr::Integer(channel),
                len: None,
            });
            return;
        };

        let type_key = record_type.to_ascii_lowercase();
        let Some(rec) = self.records.get(&type_key).cloned() else {
            self.diagnostics.push(Diagnostic::error(
                generated_pos(),
                format!(
                    "unknown record type `{record_type}` in `file {} as {record_type}`",
                    var.name
                ),
            ));
            return;
        };

        let channel = self.next_channel;
        self.next_channel += 1;
        out.push(Statement::Raw(format!(
            "' file {} as {record_type} = open(...)  [{} bytes/record]",
            var.name, rec.width
        )));
        self.files.insert(
            var_key,
            FileInfo {
                channel,
                kind: FileKind::Record(record_type.clone()),
                owner: self.current_function.clone(),
            },
        );

        out.push(Statement::Open {
            mode: OpenMode::Random,
            file: path,
            channel: Expr::Integer(channel),
            len: Some(Expr::Integer(rec.width as i64)),
        });

        let fields = rec
            .fields
            .iter()
            .map(|f| {
                (
                    Expr::Integer(field_width(&f.ty) as i64),
                    self.buffer_ident(&var.name, &f.name),
                )
            })
            .collect();
        out.push(Statement::Field {
            channel: Expr::Integer(channel),
            fields,
            record_type: Some(record_type),
            string_fields: Some(
                rec.fields
                    .iter()
                    .map(|field| matches!(field.ty, RecordFieldType::Str(..)))
                    .collect(),
            ),
            field_types: Some(rec.fields.iter().map(|field| field.ty).collect()),
        });
    }

    fn lower_assignment(&mut self, target: Expr, value: Expr, out: &mut Vec<Statement>) {
        if let (Expr::FileIndex { var, index }, Expr::RecordLit { fields, partial }) =
            (&target, &value)
        {
            let var = var.clone();
            let index = (**index).clone();
            let fields = fields.clone();
            let partial = *partial;
            self.lower_whole_write(var, index, fields, partial, out);
            return;
        }
        if let (Expr::Ident(name), Expr::FileIndex { var, index }) = (&target, &value) {
            let name = name.clone();
            let var = var.clone();
            let index = (**index).clone();
            self.lower_whole_read(name, var, index, out);
            return;
        }
        // `db[i] = s` — write a `let`-bound record variable back in one shot
        // (one PUT), as opposed to `db[i] = { ... }` which writes a fresh
        // literal. Only takes this path when `s` is actually a known
        // record variable; otherwise falls through to the generic case
        // below, where a bare `Expr::FileIndex` target is rejected.
        if let (Expr::FileIndex { var, index }, Expr::Ident(record_var)) = (&target, &value) {
            if self
                .record_vars
                .contains_key(&record_var.name.to_ascii_lowercase())
            {
                let var = var.clone();
                let index = (**index).clone();
                let record_var = record_var.clone();
                self.lower_whole_write_from_var(var, index, record_var, out);
                return;
            }
        }
        if let Expr::FieldAccess { base, field } = &target {
            if let Expr::FileIndex { var, index } = base.as_ref() {
                let var = var.clone();
                let index = (**index).clone();
                let field = field.clone();
                self.lower_partial_update(var, index, field, value, out);
                return;
            }
        }
        let target = self.rewrite_expr(target).0;
        let value = self.rewrite_expr(value).0;
        out.push(Statement::Assignment { target, value });
    }

    /// `db[i] = { ... }` (`partial: false`, every declared field required —
    /// no `GET`, since every byte of the buffer is about to be overwritten)
    /// or `db[i] = ?{ ... }` (`partial: true`, a subset is allowed). Whether
    /// a `GET` is needed is decided purely by comparing the given field
    /// names against the record's declared fields — fully static, no
    /// runtime check — so a partial literal that happens to name every
    /// field gets the same no-`GET` treatment as a full one.
    fn lower_whole_write(
        &mut self,
        var: BasicIdent,
        index: Expr,
        pairs: Vec<(String, Expr)>,
        partial: bool,
        out: &mut Vec<Statement>,
    ) {
        let index = self.rewrite_expr(index).0;
        let Some((channel, rec, type_name)) = self.lookup_file(&var) else {
            return;
        };

        let literal_syntax = if partial { "?{ ... }" } else { "{ ... }" };
        let kind = if partial {
            "partial-record write"
        } else {
            "whole-record write"
        };
        out.push(Statement::Raw(format!(
            "' {}[...] = {literal_syntax}  ({kind})",
            var.name
        )));

        let mut provided: HashMap<String, Expr> = HashMap::new();
        for (name, value) in pairs {
            if !rec
                .fields
                .iter()
                .any(|f| f.name.eq_ignore_ascii_case(&name))
            {
                self.diagnostics.push(Diagnostic::error(
                    generated_pos(),
                    format!("record `{type_name}` has no field `{name}`"),
                ));
                continue;
            }
            if provided.insert(name.to_ascii_lowercase(), value).is_some() {
                self.diagnostics.push(Diagnostic::error(
                    generated_pos(),
                    format!("field `{name}` specified more than once in record literal"),
                ));
            }
        }

        let covers_every_field = rec
            .fields
            .iter()
            .all(|f| provided.contains_key(&f.name.to_ascii_lowercase()));

        if !partial && !covers_every_field {
            for f in &rec.fields {
                if !provided.contains_key(&f.name.to_ascii_lowercase()) {
                    self.diagnostics.push(Diagnostic::error(
                        generated_pos(),
                        format!(
                            "record literal for `{type_name}` is missing field `{}` (use `?{{ ... }}` for a partial update)",
                            f.name
                        ),
                    ));
                }
            }
        }

        if partial && !covers_every_field {
            out.push(Statement::Get {
                channel: Expr::Integer(channel),
                record: Some(index.clone()),
                var: None,
                require_existing: true,
                record_length: Some(rec.width),
            });
        }

        let provided_fields = rec
            .fields
            .iter()
            .map(|f| provided.contains_key(&f.name.to_ascii_lowercase()))
            .collect();
        for f in &rec.fields {
            let Some(value_expr) = provided.remove(&f.name.to_ascii_lowercase()) else {
                continue;
            };
            self.check_field_value_type(&value_expr, &f.ty, &f.name, &type_name);
            let value_expr = self.rewrite_expr(value_expr).0;
            let buf_ident = self.buffer_ident(&var.name, &f.name);
            let packed = pack_expr(&f.ty, value_expr);
            out.push(string_assignment_statement(&f.ty, buf_ident, packed));
        }

        out.push(Statement::Put {
            channel: Expr::Integer(channel),
            record: Some(index),
            var: None,
            provided_fields: Some(provided_fields),
        });
    }

    /// `db[i] = s` where `s` was bound by an earlier `let s = db[j]`. Packs
    /// every field straight from `s`'s already-unpacked scalars (`s_field`)
    /// and issues a single `PUT` — this is the batched write-back path: any
    /// number of prior `s.field = value` assignments (which only touch the
    /// in-memory scalar, see `resolve_field_access`) are committed in one
    /// round trip here, instead of one `GET`+`PUT` per field.
    fn lower_whole_write_from_var(
        &mut self,
        var: BasicIdent,
        index: Expr,
        record_var: BasicIdent,
        out: &mut Vec<Statement>,
    ) {
        let index = self.rewrite_expr(index).0;
        let Some((channel, rec, type_name)) = self.lookup_file(&var) else {
            return;
        };

        let record_var_key = record_var.name.to_ascii_lowercase();
        let Some(record_var_type) = self.record_vars.get(&record_var_key).cloned() else {
            // lower_assignment only takes this path when the key exists.
            return;
        };
        if !record_var_type.eq_ignore_ascii_case(&type_name) {
            self.diagnostics.push(Diagnostic::error(
                generated_pos(),
                format!(
                    "cannot write `{}` (a `{record_var_type}`) into `{}`, which holds `{type_name}` records",
                    record_var.name, var.name
                ),
            ));
            return;
        }

        out.push(Statement::Raw(format!(
            "' {}[...] = {}  (write back a let-bound record)",
            var.name, record_var.name
        )));

        for f in &rec.fields {
            let scalar_name = camel_join(&[&record_var.name, &f.name]);
            let scalar_ident = BasicIdent {
                name: scalar_name,
                suffix: Some(field_suffix(&f.ty)),
            };
            let buf_ident = self.buffer_ident(&var.name, &f.name);
            let packed = pack_expr(&f.ty, Expr::Ident(scalar_ident));
            out.push(string_assignment_statement(&f.ty, buf_ident, packed));
        }

        out.push(Statement::Put {
            channel: Expr::Integer(channel),
            record: Some(index),
            var: None,
            provided_fields: None,
        });
    }

    fn lower_whole_read(
        &mut self,
        target: BasicIdent,
        var: BasicIdent,
        index: Expr,
        out: &mut Vec<Statement>,
    ) {
        let index = self.rewrite_expr(index).0;
        let Some((channel, rec, type_name)) = self.lookup_file(&var) else {
            return;
        };

        out.push(Statement::Raw(format!(
            "' let {} = {}[...]  (whole-record read)",
            target.name, var.name
        )));
        out.push(Statement::Get {
            channel: Expr::Integer(channel),
            record: Some(index),
            var: None,
            require_existing: false,
            record_length: None,
        });

        for f in &rec.fields {
            let buf_ident = self.buffer_ident(&var.name, &f.name);
            let scalar_name = camel_join(&[&target.name, &f.name]);
            let scalar_ident = BasicIdent {
                name: scalar_name,
                suffix: Some(field_suffix(&f.ty)),
            };
            if field_is_numeric(&f.ty) {
                let unpacked = Expr::Call {
                    name: BasicIdent::parse(unpack_fn_name(&f.ty)),
                    args: vec![Expr::Ident(buf_ident)],
                };
                out.push(Statement::Assignment {
                    target: Expr::Ident(scalar_ident),
                    value: unpacked,
                });
            } else {
                // Real MBASIC/BASCOM has no RTRIM$ builtin -- strip the
                // trailing space padding LSET left in the fixed-width
                // buffer with an inline scan instead, built directly out
                // of LEN/MID$/LEFT$, which every target actually has.
                let counter_ident = BasicIdent {
                    name: camel_join(&[&target.name, &f.name, "trimI"]),
                    suffix: Some(TypeSuffix::Integer),
                };
                out.extend(trim_statements(&buf_ident, &counter_ident, &scalar_ident));
            }
        }

        self.record_vars
            .insert(target.name.to_ascii_lowercase(), type_name);
    }

    fn lower_partial_update(
        &mut self,
        var: BasicIdent,
        index: Expr,
        field: String,
        value: Expr,
        out: &mut Vec<Statement>,
    ) {
        let index = self.rewrite_expr(index).0;
        let Some((channel, rec, type_name)) = self.lookup_file(&var) else {
            return;
        };
        let Some(field_spec) = rec
            .fields
            .iter()
            .find(|f| f.name.eq_ignore_ascii_case(&field))
            .cloned()
        else {
            self.diagnostics.push(Diagnostic::error(
                generated_pos(),
                format!("record `{type_name}` has no field `{field}`"),
            ));
            return;
        };

        out.push(Statement::Raw(format!(
            "' {}[...].{field} = ...  (partial-field update)",
            var.name
        )));
        out.push(Statement::Get {
            channel: Expr::Integer(channel),
            record: Some(index.clone()),
            var: None,
            require_existing: true,
            record_length: Some(rec.width),
        });

        self.check_field_value_type(&value, &field_spec.ty, &field_spec.name, &type_name);
        let value = self.rewrite_expr(value).0;
        let buf_ident = self.buffer_ident(&var.name, &field_spec.name);
        let packed = pack_expr(&field_spec.ty, value);
        out.push(string_assignment_statement(
            &field_spec.ty,
            buf_ident,
            packed,
        ));

        let provided_fields = Some(
            rec.fields
                .iter()
                .map(|f| f.name.eq_ignore_ascii_case(&field))
                .collect(),
        );
        out.push(Statement::Put {
            channel: Expr::Integer(channel),
            record: Some(index),
            var: None,
            provided_fields,
        });
    }

    fn lower_expr_stmt(&mut self, expr: Expr, out: &mut Vec<Statement>) {
        if let Expr::MethodCall { base, method, args } = &expr {
            if let Expr::Ident(var) = base.as_ref() {
                let var = var.clone();
                if method.eq_ignore_ascii_case("close") && args.is_empty() {
                    if let Some(channel) = self.lookup_any_file_channel(&var) {
                        out.push(Statement::Raw(format!("' {}.close()", var.name)));
                        out.push(Statement::Close {
                            channel: Expr::Integer(channel),
                        });
                    }
                    return;
                }
                if method.eq_ignore_ascii_case("write") {
                    if let Some(channel) = self.lookup_sequential_file(
                        &var,
                        "write",
                        &[OpenMode::Output, OpenMode::Append],
                    ) {
                        let exprs = args
                            .iter()
                            .cloned()
                            .map(|e| self.rewrite_expr(e).0)
                            .collect();
                        out.push(Statement::Raw(format!("' {}.write(...)", var.name)));
                        out.push(Statement::Write {
                            channel: Expr::Integer(channel),
                            exprs,
                        });
                    }
                    return;
                }
                if method.eq_ignore_ascii_case("read") {
                    if let Some(channel) =
                        self.lookup_sequential_file(&var, "read", &[OpenMode::Input])
                    {
                        let vars = args
                            .iter()
                            .cloned()
                            .map(|e| self.rewrite_expr(e).0)
                            .collect();
                        out.push(Statement::Raw(format!("' {}.read(...)", var.name)));
                        out.push(Statement::InputFile {
                            channel: Expr::Integer(channel),
                            vars,
                        });
                    }
                    return;
                }
            }
        }
        let expr = self.rewrite_expr(expr).0;
        out.push(Statement::ExprStmt(expr));
    }

    // ── shared helpers ───────────────────────────────────────────────────

    fn lookup_file(&mut self, var: &BasicIdent) -> Option<(i64, RecordType, String)> {
        let key = var.name.to_ascii_lowercase();
        let Some(info) = self.files.get(&key).cloned() else {
            self.diagnostics.push(Diagnostic::error(
                generated_pos(),
                format!("`{}` is not a declared `file`", var.name),
            ));
            return None;
        };
        if !self.file_is_visible(var, &info) {
            return None;
        }
        let FileKind::Record(record_type) = info.kind else {
            self.diagnostics.push(Diagnostic::error(
                generated_pos(),
                format!(
                    "`{}` is a sequential file (`open(...) for input/output/append`), not a \
                     record file -- `file[i]`/`.field` operations need `file {} as <RecordType> \
                     = open(...)` instead",
                    var.name, var.name
                ),
            ));
            return None;
        };
        let type_key = record_type.to_ascii_lowercase();
        let rec = self
            .records
            .get(&type_key)
            .cloned()
            .expect("file's record type was validated at declaration time");
        Some((info.channel, rec, record_type))
    }

    /// The channel any declared `file` variable was allocated -- record or
    /// sequential alike. Used by `.close()`, which is valid on either kind.
    fn lookup_any_file_channel(&mut self, var: &BasicIdent) -> Option<i64> {
        let key = var.name.to_ascii_lowercase();
        let Some(info) = self.files.get(&key).cloned() else {
            self.diagnostics.push(Diagnostic::error(
                generated_pos(),
                format!("`{}` is not a declared `file`", var.name),
            ));
            return None;
        };
        if !self.file_is_visible(var, &info) {
            return None;
        }
        Some(info.channel)
    }

    /// The channel and mode of a *sequential* `file` variable, for
    /// `.write(...)`/`.read(...)`/`.eof()`. Rejects a record file (those
    /// use `file[i]`/`.field`/`let`, never these methods) and a mode
    /// mismatch (`.read(...)` on a file opened `for output`, etc.) at
    /// compile time, the same way the record DSL already rejects a
    /// misspelled field before it ever reaches disk.
    fn lookup_sequential_file(
        &mut self,
        var: &BasicIdent,
        method: &str,
        allowed_modes: &[OpenMode],
    ) -> Option<i64> {
        let key = var.name.to_ascii_lowercase();
        let Some(info) = self.files.get(&key).cloned() else {
            self.diagnostics.push(Diagnostic::error(
                generated_pos(),
                format!("`{}` is not a declared `file`", var.name),
            ));
            return None;
        };
        if !self.file_is_visible(var, &info) {
            return None;
        }
        let FileKind::Sequential(mode) = info.kind else {
            self.diagnostics.push(Diagnostic::error(
                generated_pos(),
                format!(
                    "`.{method}()` needs a sequential file -- `{}` is a record file (`file {} \
                     as <RecordType> = open(...)`)",
                    var.name, var.name
                ),
            ));
            return None;
        };
        if !allowed_modes.contains(&mode) {
            let mode_word = |m: OpenMode| match m {
                OpenMode::Input => "input",
                OpenMode::Output => "output",
                OpenMode::Append => "append",
                OpenMode::Random | OpenMode::Binary => {
                    unreachable!("the file/open DSL sugar never parses a random/binary mode")
                }
            };
            let expected = allowed_modes
                .iter()
                .map(|m| mode_word(*m))
                .collect::<Vec<_>>()
                .join(" or ");
            self.diagnostics.push(Diagnostic::error(
                generated_pos(),
                format!(
                    "`.{method}()` needs `{}` opened `for {expected}`, but it was opened `for {}`",
                    var.name,
                    mode_word(mode)
                ),
            ));
            return None;
        }
        Some(info.channel)
    }

    fn file_is_visible(&mut self, var: &BasicIdent, info: &FileInfo) -> bool {
        let Some(current_function) = &self.current_function else {
            return true;
        };
        let visible = match &info.owner {
            Some(owner) => owner == current_function,
            None => self
                .current_file_globals
                .contains(&var.name.to_ascii_lowercase()),
        };
        if !visible {
            self.diagnostics.push(Diagnostic::error(
                self.current_statement_pos
                    .clone()
                    .unwrap_or_else(generated_pos),
                format!(
                    "`{}` is a top-level file; declare `global {}` in `{}` before using it",
                    var.name, var.name, current_function
                ),
            ));
        }
        visible
    }

    fn buffer_ident(&mut self, file_var: &str, field_name: &str) -> BasicIdent {
        let name = camel_join(&[file_var, field_name, "buf"]);
        let ident = BasicIdent {
            name,
            suffix: Some(TypeSuffix::String),
        };
        // Keyed on the full `as_basic()` form (including the `$` suffix) to
        // match how `codegen.rs`'s `ident()` builds its lookup key.
        self.synthesized_buffer_names
            .insert(ident.as_basic().to_ascii_lowercase());
        ident
    }

    fn check_field_value_type(
        &mut self,
        value: &Expr,
        ty: &RecordFieldType,
        field_name: &str,
        type_name: &str,
    ) {
        match (value, ty) {
            (Expr::String(s), RecordFieldType::Str(width, _)) => {
                if s.len() as u32 > *width {
                    self.diagnostics.push(Diagnostic::error(
                        generated_pos(),
                        format!(
                            "value for field `{field_name}` of record `{type_name}` is {} characters, exceeds string({width})",
                            s.len()
                        ),
                    ));
                }
            }
            (Expr::String(_), _) => {
                self.diagnostics.push(Diagnostic::error(
                    generated_pos(),
                    format!("field `{field_name}` of record `{type_name}` is numeric; cannot assign a string literal"),
                ));
            }
            (Expr::Integer(_) | Expr::Float(_), RecordFieldType::Str(..)) => {
                self.diagnostics.push(Diagnostic::error(
                    generated_pos(),
                    format!("field `{field_name}` of record `{type_name}` is string(N); cannot assign a numeric literal"),
                ));
            }
            _ => {}
        }
    }

    // ── expression rewriting ────────────────────────────────────────────

    /// Rewrites `Expr::FieldAccess` for `let`-bound record variables into
    /// their unpacked scalar `Ident`, and (scoped to expressions that touch
    /// a record field) auto-wraps the numeric side of a `+` string/number
    /// mix in `STR$(...)`. Returns the rewritten expression plus, when it is
    /// (or resolves to) a freshly-rewritten record field, its `FieldKind` —
    /// this is deliberately not a general BASIC type inferencer; `FieldKind`
    /// is only ever produced here and only propagated through `+`.
    fn rewrite_expr(&mut self, expr: Expr) -> (Expr, Option<FieldKind>) {
        match expr {
            Expr::Integer(_) | Expr::Float(_) | Expr::HexLit(_) => (expr, Some(FieldKind::Numeric)),
            Expr::String(_) => (expr, Some(FieldKind::Stringy)),
            Expr::Ident(_) => (expr, None),
            Expr::ArrayRef { name, indices } => {
                let indices: Vec<Expr> = indices
                    .into_iter()
                    .map(|e| self.rewrite_expr(e).0)
                    .collect();
                match self.try_ordinary_call_as_method(&name, indices) {
                    Ok(rewritten) => (rewritten, None),
                    Err(indices) => (Expr::ArrayRef { name, indices }, None),
                }
            }
            Expr::Call { name, args } => {
                let args: Vec<Expr> = args.into_iter().map(|e| self.rewrite_expr(e).0).collect();
                match self.try_ordinary_call_as_method(&name, args) {
                    Ok(rewritten) => (rewritten, None),
                    Err(args) => (Expr::Call { name, args }, None),
                }
            }
            Expr::Unary { op, expr } => {
                let (inner, kind) = self.rewrite_expr(*expr);
                (
                    Expr::Unary {
                        op,
                        expr: Box::new(inner),
                    },
                    kind,
                )
            }
            Expr::Binary { left, op, right } => {
                let (left, left_kind) = self.rewrite_expr(*left);
                let (right, right_kind) = self.rewrite_expr(*right);
                if op == BinaryOp::Add {
                    match (left_kind, right_kind) {
                        (Some(FieldKind::Stringy), Some(FieldKind::Numeric)) => {
                            let right = wrap_str(right);
                            return (
                                Expr::Binary {
                                    left: Box::new(left),
                                    op,
                                    right: Box::new(right),
                                },
                                Some(FieldKind::Stringy),
                            );
                        }
                        (Some(FieldKind::Numeric), Some(FieldKind::Stringy)) => {
                            let left = wrap_str(left);
                            return (
                                Expr::Binary {
                                    left: Box::new(left),
                                    op,
                                    right: Box::new(right),
                                },
                                Some(FieldKind::Stringy),
                            );
                        }
                        (Some(FieldKind::Stringy), Some(FieldKind::Stringy)) => {
                            return (
                                Expr::Binary {
                                    left: Box::new(left),
                                    op,
                                    right: Box::new(right),
                                },
                                Some(FieldKind::Stringy),
                            );
                        }
                        (Some(FieldKind::Numeric), Some(FieldKind::Numeric)) => {
                            return (
                                Expr::Binary {
                                    left: Box::new(left),
                                    op,
                                    right: Box::new(right),
                                },
                                Some(FieldKind::Numeric),
                            );
                        }
                        _ => {}
                    }
                }
                (
                    Expr::Binary {
                        left: Box::new(left),
                        op,
                        right: Box::new(right),
                    },
                    None,
                )
            }
            Expr::FieldAccess { base, field } => self.resolve_field_access(*base, field),
            Expr::FileIndex { .. } => {
                self.diagnostics.push(Diagnostic::error(
                    generated_pos(),
                    "`file[i]` may only be used as `let x = file[i]`, `file[i] = { ... }`, `file[i].field = value`, or as the target of `.close()`".to_string(),
                ));
                (Expr::Integer(0), None)
            }
            Expr::RecordLit { .. } => {
                self.diagnostics.push(Diagnostic::error(
                    generated_pos(),
                    "record literal `{ ... }` / `?{ ... }` may only be used as the right-hand side of `file[i] = { ... }` or `file[i] = ?{ ... }`".to_string(),
                ));
                (Expr::Integer(0), None)
            }
            Expr::MethodCall { base, method, args } => {
                if method.eq_ignore_ascii_case("eof") && args.is_empty() {
                    if let Expr::Ident(var) = base.as_ref() {
                        let var = var.clone();
                        if let Some(channel) =
                            self.lookup_sequential_file(&var, "eof", &[OpenMode::Input])
                        {
                            return (
                                Expr::Call {
                                    name: BasicIdent::parse("eof"),
                                    args: vec![Expr::Integer(channel)],
                                },
                                None,
                            );
                        }
                        return (Expr::Integer(0), None);
                    }
                }
                if method.eq_ignore_ascii_case("close") {
                    self.diagnostics.push(Diagnostic::error(
                        generated_pos(),
                        "`.close()` may only be used as a standalone statement".to_string(),
                    ));
                } else if method.eq_ignore_ascii_case("write")
                    || method.eq_ignore_ascii_case("read")
                {
                    self.diagnostics.push(Diagnostic::error(
                        generated_pos(),
                        format!("`.{method}(...)` may only be used as a standalone statement"),
                    ));
                } else {
                    self.diagnostics.push(Diagnostic::error(
                        generated_pos(),
                        format!("unknown method `.{method}()`"),
                    ));
                }
                (Expr::Integer(0), None)
            }
            Expr::ScalarMethodCall { base, method, args } => {
                let (base, _) = self.rewrite_expr(*base);
                let args: Vec<Expr> = args
                    .into_iter()
                    .map(|arg| self.rewrite_expr(arg).0)
                    .collect();
                (self.rewrite_scalar_method_call(base, method, args), None)
            }
        }
    }

    /// Rewrites `base.method(args)` into the equivalent ordinary call
    /// (`method$(base, args...)`) when `method` names one of the built-in
    /// scalar methods `scalar_builtins.rs` registers for `base`'s own
    /// receiver type -- see that module's own doc comment for why this
    /// reuses the ordinary-call codegen both backends already have,
    /// instead of teaching `codegen_basic.rs`/`codegen_c.rs` a second,
    /// built-in source of truth alongside their existing user-method
    /// lookups. `base`/`args` are already fully rewritten by the caller, so
    /// a chain like `s$.trim().left(3)` resolves correctly regardless of
    /// which half is user-defined (`.trim()`, left as `ScalarMethodCall`
    /// here, resolved via `user_method_results`) vs. built-in (`.left()`,
    /// rewritten here).
    ///
    /// Matches the parser's own `make_paren_ident_expr` convention
    /// (parser.rs) for which shape a call becomes: `Expr::ArrayRef` only
    /// when there are zero arguments, or the callee carries a type suffix
    /// and there's exactly one. None of this registry's entries hit that
    /// second case (every one whose rewritten call could have exactly one
    /// total argument -- `len()`, `abs()`, `sqr()`, ... -- uses a
    /// suffix-less `call_suffix: None`, matching real BASIC's own bare
    /// `LEN`/`ABS`/`SQR`/...), so every rewrite here is `Expr::Call` in
    /// practice; the check is still spelled out in full below rather than
    /// hardcoded, so it stays correct if a future entry ever pairs a
    /// suffix with a single argument.
    ///
    /// `method` not matching any built-in for this receiver leaves the node
    /// as `Expr::ScalarMethodCall` unchanged -- resolver's own
    /// `validate_one_scalar_method` still handles the "unknown method" and
    /// "user-declared method" cases exactly as it does today.
    fn rewrite_scalar_method_call(&mut self, base: Expr, method: String, args: Vec<Expr>) -> Expr {
        let Some(receiver) = self.infer_scalar_suffix(&base) else {
            self.diagnostics.push(Diagnostic::error(
                generated_pos(),
                format!("method receiver for `.{method}()` must be a scalar expression"),
            ));
            return Expr::Integer(0);
        };
        let Some(builtin) = crate::scalar_builtins::find(receiver, &method) else {
            return Expr::ScalarMethodCall {
                base: Box::new(base),
                method,
                args,
            };
        };
        if args.len() < builtin.min_args || args.len() > builtin.max_args {
            let expected = if builtin.min_args == builtin.max_args {
                format!("{} argument(s)", builtin.min_args)
            } else {
                format!("{} to {} argument(s)", builtin.min_args, builtin.max_args)
            };
            self.diagnostics.push(Diagnostic::error(
                generated_pos(),
                format!("`.{method}()` expects {expected}, got {}", args.len()),
            ));
            return Expr::Integer(0);
        }
        let name = BasicIdent {
            name: builtin.method.to_string(),
            suffix: builtin.call_suffix,
        };
        let mut call_args = Vec::with_capacity(args.len() + 1);
        call_args.push(base);
        call_args.extend(args);
        // Exactly `make_paren_ident_expr`'s own disambiguation rule
        // (parser.rs) -- a call with a type suffix and exactly one
        // argument parses as `ArrayRef` in real source, so a synthesized
        // one must match; every entry here that could ever reach exactly
        // one total argument (`len`/`abs`/`sqr`/`sin`/`cos`/`tan`/`int`/
        // `fix`/`sgn`, receiver alone) has `call_suffix: None`, so this
        // always resolves to `Call` in practice -- still spelled out in
        // full rather than assuming that, so it stays correct if a future
        // registry entry ever pairs a suffix with a single argument.
        if call_args.is_empty() || (name.suffix.is_some() && call_args.len() == 1) {
            Expr::ArrayRef {
                name,
                indices: call_args,
            }
        } else {
            Expr::Call {
                name,
                args: call_args,
            }
        }
    }

    /// A minimal scalar-type inference used only to pick the right
    /// receiver family for `rewrite_scalar_method_call` -- not a full
    /// expression-type checker. Mirrors `resolver::scalar_expr_type`'s own
    /// shape (same literal/`Ident`/`Unary`/`Binary`/`Call`/`ArrayRef` arms,
    /// same "bare/suffixless defaults to `Single`" convention already
    /// established there), but reads a chained user-declared method's
    /// result type from `user_method_results` (built once, up front --
    /// see `build_user_method_table`) instead of scanning `program.functions`
    /// live, since this pass no longer holds onto the whole `Program`.
    fn infer_scalar_suffix(&self, expr: &Expr) -> Option<TypeSuffix> {
        match expr {
            Expr::String(_) => Some(TypeSuffix::String),
            Expr::Integer(_) | Expr::HexLit(_) => Some(TypeSuffix::Integer),
            Expr::Float(_) => Some(TypeSuffix::Single),
            Expr::Ident(id) | Expr::Call { name: id, .. } | Expr::ArrayRef { name: id, .. } => {
                Some(id.suffix.unwrap_or(TypeSuffix::Single))
            }
            Expr::Unary { expr, .. } => self.infer_scalar_suffix(expr),
            Expr::Binary { left, .. } => self.infer_scalar_suffix(left),
            Expr::ScalarMethodCall { base, method, .. } => {
                let receiver = self.infer_scalar_suffix(base)?;
                self.user_method_results
                    .get(&(receiver, method.to_ascii_lowercase()))
                    .copied()
            }
            _ => None,
        }
    }

    /// The mirror image of `rewrite_scalar_method_call`: an *ordinary* call
    /// `name(arg0, arg1, ...)` resolves straight to a user-declared method
    /// `method<T> name(...)` when no ordinary function or real BASIC
    /// builtin already claims `name` (at this exact suffix, for a
    /// function -- suffix-independently, for a builtin, matching
    /// `reject_functions_shadowing_builtins`'s own rule) and `arg0`'s own
    /// scalar type matches `T`. A method is conceptually a function with
    /// its receiver as an implicit first parameter (per the resolver's own
    /// `reject_scalar_methods`, which now rejects a program that declares
    /// both), so `ltrim$(s$)` resolving to `method ltrim[string]()` with `s$` as
    /// the receiver keeps the ordinary call syntax working with no
    /// duplicate declaration needed.
    ///
    /// Returns `Ok(rewritten)` (an `Expr::ScalarMethodCall`, left for the
    /// resolver's/codegen's existing user-method handling to process
    /// exactly like a hand-written `s$.ltrim()` -- including its existing
    /// argument-count validation, so a wrong number of *extra* arguments
    /// here still gets a sensible error with no special-casing needed) when
    /// the fallback applies, or `Err(args)` (the original arguments, handed
    /// back unchanged so the caller can reassemble its own node) when it
    /// doesn't -- `name` then stays whatever it already was
    /// (`Expr::Call`/`Expr::ArrayRef`), and resolver's existing "unknown
    /// function" diagnostics still fire correctly downstream.
    fn try_ordinary_call_as_method(
        &self,
        name: &BasicIdent,
        args: Vec<Expr>,
    ) -> Result<Expr, Vec<Expr>> {
        if args.is_empty() {
            return Err(args);
        }
        let mut args = args;
        let key = (name.name.to_ascii_lowercase(), name.suffix);
        if self.known_ordinary_functions.contains(&key)
            || crate::codegen_basic::BASIC_BUILTINS.contains(&key.0.as_str())
        {
            return Err(args);
        }
        let base = args.remove(0);
        let Some(receiver) = self.infer_scalar_suffix(&base) else {
            args.insert(0, base);
            return Err(args);
        };
        let Some(&result) = self.user_method_results.get(&(receiver, key.0.clone())) else {
            args.insert(0, base);
            return Err(args);
        };
        if name.suffix != Some(result) {
            args.insert(0, base);
            return Err(args);
        }
        Ok(Expr::ScalarMethodCall {
            base: Box::new(base),
            method: name.name.clone(),
            args,
        })
    }

    fn resolve_field_access(&mut self, base: Expr, field: String) -> (Expr, Option<FieldKind>) {
        let Expr::Ident(base_ident) = &base else {
            self.diagnostics.push(Diagnostic::error(
                generated_pos(),
                format!(
                    "`.{field}` is not valid here; expected a record variable created with `let`"
                ),
            ));
            return (Expr::Integer(0), None);
        };
        let base_key = base_ident.name.to_ascii_lowercase();
        let Some(record_type) = self.record_vars.get(&base_key).cloned() else {
            self.diagnostics.push(Diagnostic::error(
                generated_pos(),
                format!(
                    "`{}` is not a record variable bound with `let ... = file[i]`",
                    base_ident.name
                ),
            ));
            return (Expr::Integer(0), None);
        };
        let record_key = record_type.to_ascii_lowercase();
        let rec = self
            .records
            .get(&record_key)
            .cloned()
            .expect("record_vars only holds valid types");
        let Some(field_spec) = rec
            .fields
            .iter()
            .find(|f| f.name.eq_ignore_ascii_case(&field))
        else {
            self.diagnostics.push(Diagnostic::error(
                generated_pos(),
                format!("record `{record_type}` has no field `{field}`"),
            ));
            return (Expr::Integer(0), None);
        };
        let scalar_name = camel_join(&[&base_ident.name, &field_spec.name]);
        let ident = BasicIdent {
            name: scalar_name,
            suffix: Some(field_suffix(&field_spec.ty)),
        };
        let kind = if field_is_numeric(&field_spec.ty) {
            FieldKind::Numeric
        } else {
            FieldKind::Stringy
        };
        (Expr::Ident(ident), Some(kind))
    }
}

fn generated_pos() -> SourcePos {
    SourcePos::new("<records>", 1, 1)
}

fn wrap_str(expr: Expr) -> Expr {
    Expr::Call {
        name: BasicIdent {
            name: "str".to_string(),
            suffix: Some(TypeSuffix::String),
        },
        args: vec![expr],
    }
}

fn pack_expr(ty: &RecordFieldType, value: Expr) -> Expr {
    if field_is_numeric(ty) {
        Expr::Call {
            name: BasicIdent::parse(pack_fn_name(ty)),
            args: vec![value],
        }
    } else {
        value
    }
}

/// Builds `i% = LEN(buf$) : WHILE i% > 0 AND MID$(buf$,i%,1) = " " : i% = i%
/// - 1 : WEND : target$ = LEFT$(buf$, i%)` -- a right-trim, done inline
/// with LEN/MID$/LEFT$, since real MBASIC/BASCOM has no RTRIM$ builtin.
/// The `AND` here has to be the short-circuit `AndAnd` form: once `i% =
/// 0`, evaluating `MID$(buf$, 0, 1)` is itself a runtime error, so the
/// second operand must never be reached once the first is false.
fn trim_statements(
    buf_ident: &BasicIdent,
    counter_ident: &BasicIdent,
    target_ident: &BasicIdent,
) -> Vec<Statement> {
    let buf = || Expr::Ident(buf_ident.clone());
    let counter = || Expr::Ident(counter_ident.clone());

    let init = Statement::Assignment {
        target: counter(),
        value: Expr::Call {
            name: BasicIdent::parse("len"),
            args: vec![buf()],
        },
    };

    let condition = Expr::Binary {
        left: Box::new(Expr::Binary {
            left: Box::new(counter()),
            op: BinaryOp::Gt,
            right: Box::new(Expr::Integer(0)),
        }),
        op: BinaryOp::AndAnd,
        right: Box::new(Expr::Binary {
            left: Box::new(Expr::Call {
                name: BasicIdent::parse("mid$"),
                args: vec![buf(), counter(), Expr::Integer(1)],
            }),
            op: BinaryOp::Eq,
            right: Box::new(Expr::String(" ".to_string())),
        }),
    };

    let decrement = Statement::Assignment {
        target: counter(),
        value: Expr::Binary {
            left: Box::new(counter()),
            op: BinaryOp::Sub,
            right: Box::new(Expr::Integer(1)),
        },
    };

    let while_loop = Statement::While {
        condition,
        // Synthesized bookkeeping, never user-written source -- no real
        // position to report, so this inner statement gets the same
        // placeholder `generated_pos()` records.rs's own diagnostics use.
        body: vec![Stmt::new(decrement, generated_pos())],
    };

    let finalize = Statement::Assignment {
        target: Expr::Ident(target_ident.clone()),
        value: Expr::Call {
            name: BasicIdent::parse("left$"),
            args: vec![buf(), counter()],
        },
    };

    vec![init, while_loop, finalize]
}

/// Collects `global` declarations at any nesting depth in a callable body.
/// The ordinary resolver does the same for normal variables; record/file
/// lowering runs first, so it needs this small parallel view before it erases
/// the file-object syntax.
fn collect_global_names(body: &[Stmt]) -> HashSet<String> {
    let mut names = HashSet::new();
    collect_global_names_into(body, &mut names);
    names
}

fn collect_global_names_into(body: &[Stmt], names: &mut HashSet<String>) {
    for stmt in body {
        match &stmt.kind {
            Statement::GlobalDecl(ident) => {
                names.insert(ident.name.to_ascii_lowercase());
            }
            Statement::If {
                then_body,
                else_body,
                ..
            } => {
                collect_global_names_into(then_body, names);
                collect_global_names_into(else_body, names);
            }
            Statement::For { body, .. }
            | Statement::While { body, .. }
            | Statement::Do { body, .. } => collect_global_names_into(body, names),
            Statement::SelectCase {
                cases, else_body, ..
            } => {
                for case in cases {
                    collect_global_names_into(&case.body, names);
                }
                collect_global_names_into(else_body, names);
            }
            Statement::TryCatch {
                try_body,
                catch,
                finally_body,
            } => {
                collect_global_names_into(try_body, names);
                if let Some(catch) = catch {
                    collect_global_names_into(&catch.body, names);
                }
                collect_global_names_into(finally_body, names);
            }
            _ => {}
        }
    }
}

fn field_width(ty: &RecordFieldType) -> u32 {
    match ty {
        RecordFieldType::Int16 => 2,
        RecordFieldType::Int32 => 4,
        RecordFieldType::Float32 => 4,
        RecordFieldType::Float64 => 8,
        RecordFieldType::Str(n, _) => *n,
    }
}

fn string_assignment_statement(ty: &RecordFieldType, var: BasicIdent, value: Expr) -> Statement {
    match ty {
        RecordFieldType::Str(_, RecordStringAlignment::Right) => Statement::Rset { var, value },
        _ => Statement::Lset { var, value },
    }
}

fn field_is_numeric(ty: &RecordFieldType) -> bool {
    !matches!(ty, RecordFieldType::Str(..))
}

fn field_suffix(ty: &RecordFieldType) -> TypeSuffix {
    match ty {
        RecordFieldType::Int16 => TypeSuffix::Integer,
        RecordFieldType::Int32 => TypeSuffix::Long,
        RecordFieldType::Float32 => TypeSuffix::Single,
        RecordFieldType::Float64 => TypeSuffix::Double,
        RecordFieldType::Str(..) => TypeSuffix::String,
    }
}

/// `MKI$`/`MKL$`/`MKS$`/`MKD$` always return a string -- packing an integer
/// field with a `%` suffix (etc.) generates code real MBASIC/BASCOM rejects
/// outright, since those functions don't exist under that name.
fn pack_fn_name(ty: &RecordFieldType) -> &'static str {
    match ty {
        RecordFieldType::Int16 => "mki$",
        RecordFieldType::Int32 => "mkl$",
        RecordFieldType::Float32 => "mks$",
        RecordFieldType::Float64 => "mkd$",
        RecordFieldType::Str(..) => unreachable!("string fields are not packed"),
    }
}

/// `CVI`/`CVL`/`CVS`/`CVD` take no suffix at all -- each already returns its
/// one unambiguous numeric type. `RTRIM$` isn't a real MBASIC/BASCOM builtin
/// either; string fields are unpacked through the transpiler-generated trim
/// routine instead (see `emit_trim_helper` in `codegen.rs`), not this name.
fn unpack_fn_name(ty: &RecordFieldType) -> &'static str {
    match ty {
        RecordFieldType::Int16 => "cvi",
        RecordFieldType::Int32 => "cvl",
        RecordFieldType::Float32 => "cvs",
        RecordFieldType::Float64 => "cvd",
        RecordFieldType::Str(..) => {
            unreachable!("string fields are trimmed, not unpacked via a function call")
        }
    }
}
