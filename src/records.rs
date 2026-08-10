//! Lowers the high-level `record`/`file` DSL into the low-level random-access
//! file primitives (`OPEN ... FOR RANDOM`, `FIELD`, `GET`, `PUT`, `LSET`,
//! `MKx`/`CVx`) that `codegen.rs` already knows how to emit. This pass always
//! runs between parsing and `resolver::validate`; by the time codegen sees a
//! `Program`, no `RecordDef`, `Statement::FileDecl`, or DSL `Expr` variant
//! (`FileIndex`, `FieldAccess`, `MethodCall`, `RecordLit`) remains.

use std::collections::HashMap;

use crate::ast::*;
use crate::diagnostics::{Diagnostic, SourcePos};

pub fn lower(program: Program) -> Result<Program, Vec<Diagnostic>> {
    let mut lowerer = Lowerer::new();
    lowerer.build_record_table(&program.records);

    let statements = lowerer.lower_statements(program.statements);
    let functions = program
        .functions
        .into_iter()
        .map(|mut f| {
            f.body = lowerer.lower_statements(f.body);
            f
        })
        .collect();

    if !lowerer.diagnostics.is_empty() {
        return Err(lowerer.diagnostics);
    }

    Ok(Program {
        program_decl: program.program_decl,
        declarations: program.declarations,
        common: program.common,
        statements,
        functions,
        records: Vec::new(),
    })
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
    record_type: String,
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
}

impl Lowerer {
    fn new() -> Self {
        Self {
            records: HashMap::new(),
            files: HashMap::new(),
            record_vars: HashMap::new(),
            next_channel: 1,
            diagnostics: Vec::new(),
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
                fields.push(FieldSpec { name: f.name.clone(), ty: f.ty });
            }
            self.records.insert(key, RecordType { fields, width });
        }
    }

    // ── statement lowering ──────────────────────────────────────────────

    fn lower_statements(&mut self, stmts: Vec<Statement>) -> Vec<Statement> {
        let mut out = Vec::new();
        for stmt in stmts {
            self.lower_statement(stmt, &mut out);
        }
        out
    }

    fn lower_statement(&mut self, stmt: Statement, out: &mut Vec<Statement>) {
        match stmt {
            Statement::FileDecl { var, record_type, path } => {
                self.lower_file_decl(var, record_type, path, out);
            }
            Statement::Assignment { target, value } => {
                self.lower_assignment(target, value, out);
            }
            Statement::ExprStmt(expr) => {
                self.lower_expr_stmt(expr, out);
            }
            Statement::If { condition, then_body, else_body } => {
                let condition = self.rewrite_expr(condition).0;
                let then_body = self.lower_statements(then_body);
                let else_body = self.lower_statements(else_body);
                out.push(Statement::If { condition, then_body, else_body });
            }
            Statement::For { var, start, end, step, body } => {
                let start = self.rewrite_expr(start).0;
                let end = self.rewrite_expr(end).0;
                let step = step.map(|s| self.rewrite_expr(s).0);
                let body = self.lower_statements(body);
                out.push(Statement::For { var, start, end, step, body });
            }
            Statement::While { condition, body } => {
                let condition = self.rewrite_expr(condition).0;
                let body = self.lower_statements(body);
                out.push(Statement::While { condition, body });
            }
            Statement::Do { condition, body, post_condition } => {
                let condition = condition.map(|c| self.rewrite_do_condition(c));
                let body = self.lower_statements(body);
                let post_condition = post_condition.map(|c| self.rewrite_do_condition(c));
                out.push(Statement::Do { condition, body, post_condition });
            }
            Statement::SelectCase { expr, cases, else_body } => {
                let expr = self.rewrite_expr(expr).0;
                let cases = cases
                    .into_iter()
                    .map(|c| CaseClause {
                        values: c.values.into_iter().map(|v| self.rewrite_case_value(v)).collect(),
                        body: self.lower_statements(c.body),
                    })
                    .collect();
                let else_body = self.lower_statements(else_body);
                out.push(Statement::SelectCase { expr, cases, else_body });
            }
            other => out.push(self.rewrite_statement_exprs(other)),
        }
    }

    fn rewrite_do_condition(&mut self, cond: DoCondition) -> DoCondition {
        DoCondition { is_while: cond.is_while, expr: self.rewrite_expr(cond.expr).0 }
    }

    fn rewrite_case_value(&mut self, value: CaseValue) -> CaseValue {
        match value {
            CaseValue::Single(e) => CaseValue::Single(self.rewrite_expr(e).0),
            CaseValue::Range { from, to } => {
                CaseValue::Range { from: self.rewrite_expr(from).0, to: self.rewrite_expr(to).0 }
            }
            CaseValue::Is { op, value } => CaseValue::Is { op, value: self.rewrite_expr(value).0 },
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
            Statement::Dim { name, sizes } => {
                let sizes = sizes.into_iter().map(|e| self.rewrite_expr(e).0).collect();
                Statement::Dim { name, sizes }
            }
            Statement::Open { mode, file, channel, len } => {
                let file = self.rewrite_expr(file).0;
                let channel = self.rewrite_expr(channel).0;
                let len = len.map(|e| self.rewrite_expr(e).0);
                Statement::Open { mode, file, channel, len }
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
            Statement::PrintFileUsing { channel, format, tokens } => {
                let channel = self.rewrite_expr(channel).0;
                let format = self.rewrite_expr(format).0;
                let tokens = self.rewrite_print_tokens(tokens);
                Statement::PrintFileUsing { channel, format, tokens }
            }
            Statement::Close { channel } => Statement::Close { channel: self.rewrite_expr(channel).0 },
            Statement::Kill { file } => Statement::Kill { file: self.rewrite_expr(file).0 },
            Statement::Name { from, to } => {
                let from = self.rewrite_expr(from).0;
                let to = self.rewrite_expr(to).0;
                Statement::Name { from, to }
            }
            Statement::Print { tokens } => Statement::Print { tokens: self.rewrite_print_tokens(tokens) },
            Statement::Return { value } => Statement::Return { value: self.rewrite_expr(value).0 },
            Statement::OptionBase(e) => Statement::OptionBase(self.rewrite_expr(e).0),
            Statement::Randomize(e) => Statement::Randomize(e.map(|e| self.rewrite_expr(e).0)),
            Statement::Swap(a, b) => Statement::Swap(self.rewrite_expr(a).0, self.rewrite_expr(b).0),
            Statement::Poke { address, value } => {
                let address = self.rewrite_expr(address).0;
                let value = self.rewrite_expr(value).0;
                Statement::Poke { address, value }
            }
            Statement::Goto(e) => Statement::Goto(self.rewrite_expr(e).0),
            Statement::Gosub(e) => Statement::Gosub(self.rewrite_expr(e).0),
            Statement::OnErrorGoto { target } => {
                Statement::OnErrorGoto { target: self.rewrite_expr(target).0 }
            }
            Statement::Resume(kind) => {
                let kind = match kind {
                    ResumeTarget::Line(e) => ResumeTarget::Line(self.rewrite_expr(e).0),
                    other => other,
                };
                Statement::Resume(kind)
            }
            Statement::ErrorStmt { code } => Statement::ErrorStmt { code: self.rewrite_expr(code).0 },
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
            Statement::Restore(target) => Statement::Restore(target.map(|e| self.rewrite_expr(e).0)),
            Statement::Const { name, value } => {
                Statement::Const { name, value: self.rewrite_expr(value).0 }
            }
            Statement::Write { channel, exprs } => {
                let channel = self.rewrite_expr(channel).0;
                let exprs = exprs.into_iter().map(|e| self.rewrite_expr(e).0).collect();
                Statement::Write { channel, exprs }
            }
            Statement::Field { channel, fields } => {
                let channel = self.rewrite_expr(channel).0;
                let fields = fields.into_iter().map(|(w, v)| (self.rewrite_expr(w).0, v)).collect();
                Statement::Field { channel, fields }
            }
            Statement::Get { channel, record, var } => {
                let channel = self.rewrite_expr(channel).0;
                let record = record.map(|e| self.rewrite_expr(e).0);
                let var = var.map(|e| self.rewrite_expr(e).0);
                Statement::Get { channel, record, var }
            }
            Statement::Put { channel, record, var } => {
                let channel = self.rewrite_expr(channel).0;
                let record = record.map(|e| self.rewrite_expr(e).0);
                let var = var.map(|e| self.rewrite_expr(e).0);
                Statement::Put { channel, record, var }
            }
            Statement::Lset { var, value } => Statement::Lset { var, value: self.rewrite_expr(value).0 },
            Statement::Rset { var, value } => Statement::Rset { var, value: self.rewrite_expr(value).0 },
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
            Statement::OnBranch { expr, targets, is_gosub } => {
                let expr = self.rewrite_expr(expr).0;
                let targets = targets.into_iter().map(|e| self.rewrite_expr(e).0).collect();
                Statement::OnBranch { expr, targets, is_gosub }
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
            | Statement::SelectCase { .. } => {
                unreachable!("handled directly in lower_statement")
            }
        }
    }

    // ── the five record/file DSL shapes ─────────────────────────────────

    fn lower_file_decl(&mut self, var: BasicIdent, record_type: String, path: Expr, out: &mut Vec<Statement>) {
        let path = self.rewrite_expr(path).0;
        let var_key = var.name.to_ascii_lowercase();
        if self.files.contains_key(&var_key) {
            self.diagnostics.push(Diagnostic::error(
                generated_pos(),
                format!("file `{}` is already declared", var.name),
            ));
            return;
        }
        let type_key = record_type.to_ascii_lowercase();
        let Some(rec) = self.records.get(&type_key).cloned() else {
            self.diagnostics.push(Diagnostic::error(
                generated_pos(),
                format!("unknown record type `{record_type}` in `file {} as {record_type}`", var.name),
            ));
            return;
        };

        let channel = self.next_channel;
        self.next_channel += 1;
        out.push(Statement::Raw(format!(
            "' file {} as {record_type} = open(...)  [{} bytes/record]",
            var.name, rec.width
        )));
        self.files.insert(var_key, FileInfo { channel, record_type });

        out.push(Statement::Open {
            mode: OpenMode::Random,
            file: path,
            channel: Expr::Integer(channel),
            len: Some(Expr::Integer(rec.width as i64)),
        });

        let fields = rec
            .fields
            .iter()
            .map(|f| (Expr::Integer(field_width(&f.ty) as i64), self.buffer_ident(&var.name, &f.name)))
            .collect();
        out.push(Statement::Field { channel: Expr::Integer(channel), fields });
    }

    fn lower_assignment(&mut self, target: Expr, value: Expr, out: &mut Vec<Statement>) {
        if let (Expr::FileIndex { var, index }, Expr::RecordLit { fields, partial }) = (&target, &value) {
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
            if self.record_vars.contains_key(&record_var.name.to_ascii_lowercase()) {
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
        let Some((channel, rec, type_name)) = self.lookup_file(&var) else { return };

        let literal_syntax = if partial { "?{ ... }" } else { "{ ... }" };
        let kind = if partial { "partial-record write" } else { "whole-record write" };
        out.push(Statement::Raw(format!("' {}[...] = {literal_syntax}  ({kind})", var.name)));

        let mut provided: HashMap<String, Expr> = HashMap::new();
        for (name, value) in pairs {
            if !rec.fields.iter().any(|f| f.name.eq_ignore_ascii_case(&name)) {
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

        let covers_every_field =
            rec.fields.iter().all(|f| provided.contains_key(&f.name.to_ascii_lowercase()));

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
            });
        }

        for f in &rec.fields {
            let Some(value_expr) = provided.remove(&f.name.to_ascii_lowercase()) else { continue };
            self.check_field_value_type(&value_expr, &f.ty, &f.name, &type_name);
            let value_expr = self.rewrite_expr(value_expr).0;
            let buf_ident = self.buffer_ident(&var.name, &f.name);
            let packed = pack_expr(&f.ty, value_expr);
            out.push(Statement::Lset { var: buf_ident, value: packed });
        }

        out.push(Statement::Put { channel: Expr::Integer(channel), record: Some(index), var: None });
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
        let Some((channel, rec, type_name)) = self.lookup_file(&var) else { return };

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
            let scalar_name =
                format!("{}_{}", record_var.name.to_ascii_lowercase(), f.name.to_ascii_lowercase());
            let scalar_ident = BasicIdent { name: scalar_name, suffix: Some(field_suffix(&f.ty)) };
            let buf_ident = self.buffer_ident(&var.name, &f.name);
            let packed = pack_expr(&f.ty, Expr::Ident(scalar_ident));
            out.push(Statement::Lset { var: buf_ident, value: packed });
        }

        out.push(Statement::Put { channel: Expr::Integer(channel), record: Some(index), var: None });
    }

    fn lower_whole_read(&mut self, target: BasicIdent, var: BasicIdent, index: Expr, out: &mut Vec<Statement>) {
        let index = self.rewrite_expr(index).0;
        let Some((channel, rec, type_name)) = self.lookup_file(&var) else { return };

        out.push(Statement::Raw(format!(
            "' let {} = {}[...]  (whole-record read)",
            target.name, var.name
        )));
        out.push(Statement::Get { channel: Expr::Integer(channel), record: Some(index), var: None });

        for f in &rec.fields {
            let buf_ident = self.buffer_ident(&var.name, &f.name);
            let unpacked = Expr::Call {
                name: BasicIdent::parse(unpack_fn_name(&f.ty)),
                args: vec![Expr::Ident(buf_ident)],
            };
            let scalar_name =
                format!("{}_{}", target.name.to_ascii_lowercase(), f.name.to_ascii_lowercase());
            let scalar_ident = BasicIdent { name: scalar_name, suffix: Some(field_suffix(&f.ty)) };
            out.push(Statement::Assignment { target: Expr::Ident(scalar_ident), value: unpacked });
        }

        self.record_vars.insert(target.name.to_ascii_lowercase(), type_name);
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
        let Some((channel, rec, type_name)) = self.lookup_file(&var) else { return };
        let Some(field_spec) = rec.fields.iter().find(|f| f.name.eq_ignore_ascii_case(&field)).cloned()
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
        out.push(Statement::Get { channel: Expr::Integer(channel), record: Some(index.clone()), var: None });

        self.check_field_value_type(&value, &field_spec.ty, &field_spec.name, &type_name);
        let value = self.rewrite_expr(value).0;
        let buf_ident = self.buffer_ident(&var.name, &field_spec.name);
        let packed = pack_expr(&field_spec.ty, value);
        out.push(Statement::Lset { var: buf_ident, value: packed });

        out.push(Statement::Put { channel: Expr::Integer(channel), record: Some(index), var: None });
    }

    fn lower_expr_stmt(&mut self, expr: Expr, out: &mut Vec<Statement>) {
        if let Expr::MethodCall { base, method, args } = &expr {
            if method.eq_ignore_ascii_case("close") && args.is_empty() {
                if let Expr::Ident(var) = base.as_ref() {
                    let var = var.clone();
                    if let Some((channel, _rec, _type_name)) = self.lookup_file(&var) {
                        out.push(Statement::Raw(format!("' {}.close()", var.name)));
                        out.push(Statement::Close { channel: Expr::Integer(channel) });
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
        let type_key = info.record_type.to_ascii_lowercase();
        let rec = self
            .records
            .get(&type_key)
            .cloned()
            .expect("file's record type was validated at declaration time");
        Some((info.channel, rec, info.record_type))
    }

    fn buffer_ident(&self, file_var: &str, field_name: &str) -> BasicIdent {
        BasicIdent {
            name: format!("{}_{}buf", file_var.to_ascii_lowercase(), field_name.to_ascii_lowercase()),
            suffix: Some(TypeSuffix::String),
        }
    }

    fn check_field_value_type(&mut self, value: &Expr, ty: &RecordFieldType, field_name: &str, type_name: &str) {
        match (value, ty) {
            (Expr::String(s), RecordFieldType::Str(width)) => {
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
            (Expr::Integer(_) | Expr::Float(_), RecordFieldType::Str(_)) => {
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
                let indices = indices.into_iter().map(|e| self.rewrite_expr(e).0).collect();
                (Expr::ArrayRef { name, indices }, None)
            }
            Expr::Call { name, args } => {
                let args = args.into_iter().map(|e| self.rewrite_expr(e).0).collect();
                (Expr::Call { name, args }, None)
            }
            Expr::Unary { op, expr } => {
                let (inner, kind) = self.rewrite_expr(*expr);
                (Expr::Unary { op, expr: Box::new(inner) }, kind)
            }
            Expr::Binary { left, op, right } => {
                let (left, left_kind) = self.rewrite_expr(*left);
                let (right, right_kind) = self.rewrite_expr(*right);
                if op == BinaryOp::Add {
                    match (left_kind, right_kind) {
                        (Some(FieldKind::Stringy), Some(FieldKind::Numeric)) => {
                            let right = wrap_str(right);
                            return (
                                Expr::Binary { left: Box::new(left), op, right: Box::new(right) },
                                Some(FieldKind::Stringy),
                            );
                        }
                        (Some(FieldKind::Numeric), Some(FieldKind::Stringy)) => {
                            let left = wrap_str(left);
                            return (
                                Expr::Binary { left: Box::new(left), op, right: Box::new(right) },
                                Some(FieldKind::Stringy),
                            );
                        }
                        (Some(FieldKind::Stringy), Some(FieldKind::Stringy)) => {
                            return (
                                Expr::Binary { left: Box::new(left), op, right: Box::new(right) },
                                Some(FieldKind::Stringy),
                            );
                        }
                        (Some(FieldKind::Numeric), Some(FieldKind::Numeric)) => {
                            return (
                                Expr::Binary { left: Box::new(left), op, right: Box::new(right) },
                                Some(FieldKind::Numeric),
                            );
                        }
                        _ => {}
                    }
                }
                (Expr::Binary { left: Box::new(left), op, right: Box::new(right) }, None)
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
            Expr::MethodCall { method, .. } => {
                if method.eq_ignore_ascii_case("close") {
                    self.diagnostics.push(Diagnostic::error(
                        generated_pos(),
                        "`.close()` may only be used as a standalone statement".to_string(),
                    ));
                } else {
                    self.diagnostics.push(Diagnostic::error(
                        generated_pos(),
                        format!("unknown method `.{method}()`"),
                    ));
                }
                (Expr::Integer(0), None)
            }
        }
    }

    fn resolve_field_access(&mut self, base: Expr, field: String) -> (Expr, Option<FieldKind>) {
        let Expr::Ident(base_ident) = &base else {
            self.diagnostics.push(Diagnostic::error(
                generated_pos(),
                format!("`.{field}` is not valid here; expected a record variable created with `let`"),
            ));
            return (Expr::Integer(0), None);
        };
        let base_key = base_ident.name.to_ascii_lowercase();
        let Some(record_type) = self.record_vars.get(&base_key).cloned() else {
            self.diagnostics.push(Diagnostic::error(
                generated_pos(),
                format!("`{}` is not a record variable bound with `let ... = file[i]`", base_ident.name),
            ));
            return (Expr::Integer(0), None);
        };
        let record_key = record_type.to_ascii_lowercase();
        let rec = self.records.get(&record_key).cloned().expect("record_vars only holds valid types");
        let Some(field_spec) = rec.fields.iter().find(|f| f.name.eq_ignore_ascii_case(&field)) else {
            self.diagnostics.push(Diagnostic::error(
                generated_pos(),
                format!("record `{record_type}` has no field `{field}`"),
            ));
            return (Expr::Integer(0), None);
        };
        let scalar_name =
            format!("{}_{}", base_ident.name.to_ascii_lowercase(), field_spec.name.to_ascii_lowercase());
        let ident = BasicIdent { name: scalar_name, suffix: Some(field_suffix(&field_spec.ty)) };
        let kind = if field_is_numeric(&field_spec.ty) { FieldKind::Numeric } else { FieldKind::Stringy };
        (Expr::Ident(ident), Some(kind))
    }
}

fn generated_pos() -> SourcePos {
    SourcePos::new("<records>", 1, 1)
}

fn wrap_str(expr: Expr) -> Expr {
    Expr::Call { name: BasicIdent { name: "str".to_string(), suffix: Some(TypeSuffix::String) }, args: vec![expr] }
}

fn pack_expr(ty: &RecordFieldType, value: Expr) -> Expr {
    if field_is_numeric(ty) {
        Expr::Call { name: BasicIdent::parse(pack_fn_name(ty)), args: vec![value] }
    } else {
        value
    }
}

fn field_width(ty: &RecordFieldType) -> u32 {
    match ty {
        RecordFieldType::Int16 => 2,
        RecordFieldType::Int32 => 4,
        RecordFieldType::Float32 => 4,
        RecordFieldType::Float64 => 8,
        RecordFieldType::Str(n) => *n,
    }
}

fn field_is_numeric(ty: &RecordFieldType) -> bool {
    !matches!(ty, RecordFieldType::Str(_))
}

fn field_suffix(ty: &RecordFieldType) -> TypeSuffix {
    match ty {
        RecordFieldType::Int16 => TypeSuffix::Integer,
        RecordFieldType::Int32 => TypeSuffix::Long,
        RecordFieldType::Float32 => TypeSuffix::Single,
        RecordFieldType::Float64 => TypeSuffix::Double,
        RecordFieldType::Str(_) => TypeSuffix::String,
    }
}

fn pack_fn_name(ty: &RecordFieldType) -> &'static str {
    match ty {
        RecordFieldType::Int16 => "mki%",
        RecordFieldType::Int32 => "mkl&",
        RecordFieldType::Float32 => "mks!",
        RecordFieldType::Float64 => "mkd#",
        RecordFieldType::Str(_) => unreachable!("string fields are not packed"),
    }
}

fn unpack_fn_name(ty: &RecordFieldType) -> &'static str {
    match ty {
        RecordFieldType::Int16 => "cvi%",
        RecordFieldType::Int32 => "cvl&",
        RecordFieldType::Float32 => "cvs!",
        RecordFieldType::Float64 => "cvd#",
        RecordFieldType::Str(_) => "rtrim$",
    }
}
