//! Transpiles the record/file DSL over the typed IR.
//!
//! `records::lower` performs this rewrite on the legacy AST: a `file` declaration
//! becomes `OPEN` + `FIELD`, `let p = inv[i]` becomes `GET` plus unpacking
//! assignments, `inv[i] = { ... }` becomes `LSET`/`RSET` + `PUT`, and so on.
//! This pass performs the same expansion on `SemanticModule` statements, so a
//! backend can consume record programs as ordinary primitive typed-IR
//! statements instead of reading the AST's expanded siblings.
//!
//! Channel numbers, buffer names and field layouts are not recomputed here:
//! the AST pass already resolved them into `SemanticModule::lowered_record_files`
//! and this pass reads those facts. The AST pass also reports every
//! record-DSL diagnostic before this one runs, so the pass may assume its
//! input is valid.
//!
//! The pass is conservative. Anything it does not model yet (record methods,
//! sequential `file` handles, nested record fields, an unknown file) makes it
//! leave the module untouched and return `false`, so callers keep the existing
//! behavior for that program.

use std::collections::{HashMap, HashSet};

use crate::ast::TypeSuffix;
use crate::codegen::camel_join;
use crate::rdgen_frontend::SourceSpan;
use crate::semantic_ir::*;

/// Expands the record DSL in `module` in place. Returns `true` if the module
/// was rewritten, `false` (with `module` unchanged) if the program uses
/// nothing the pass handles or uses something it does not yet model.
pub fn transpile(module: &mut SemanticModule) -> bool {
    if std::env::var_os("BCC_NO_TRANSPILE").is_some() { return false; } // TEMP-DEBUG
    if module.records.is_empty() || module.lowered_record_files.is_empty() {
        return false;
    }
    let result = Transpiler::run(module);
    if std::env::var_os("BCC_TRANSPILE_LOG").is_some() { eprintln!("TRANSPILE {}", if result.is_some() { "applied" } else { "declined" }); } // TEMP-DEBUG
    match result {
        Some(rewritten) => {
            *module = rewritten;
            module.records_transpiled = true;
            true
        }
        None => false,
    }
}

#[derive(Clone)]
struct EffectiveField {
    name: String,
    suffix: TypeSuffix,
}

struct EffectiveRecord {
    name: String,
    fields: Vec<EffectiveField>,
}

#[derive(Clone)]
struct FileInfo {
    channel: i64,
    record_type: String,
}

/// Whether an expression is a string or numeric value for the purpose of the
/// `+` mixing rule (`"n: " + p.qty` gets `STR$` around the number).
#[derive(Clone, Copy, PartialEq, Eq)]
enum FieldKind {
    Numeric,
    Stringy,
}

struct Transpiler<'a> {
    source: &'a SemanticModule,
    records: HashMap<String, EffectiveRecord>,
    files: HashMap<String, FileInfo>,
    record_vars: HashMap<String, String>,
    current_function: Option<String>,
    current_globals: HashSet<String>,
    unsupported: bool,
}

impl<'a> Transpiler<'a> {
    fn run(source: &'a SemanticModule) -> Option<SemanticModule> {
        let mut transpiler = Transpiler {
            source,
            records: HashMap::new(),
            files: HashMap::new(),
            record_vars: HashMap::new(),
            current_function: None,
            current_globals: HashSet::new(),
            unsupported: false,
        };
        // Record methods are desugared into ordinary functions with per-field
        // `byref` parameters; that is not modeled yet.
        if source.callables.iter().any(|callable| {
            matches!(callable.kind, CallableKind::InlineMethod)
                || callable.receiver.as_deref().is_some_and(|receiver| {
                    source
                        .records
                        .iter()
                        .any(|record| record.name.eq_ignore_ascii_case(receiver))
                })
        }) {
            return None;
        }
        transpiler.build_records()?;

        let mut module = source.clone();
        module.statements = source
            .statements
            .iter()
            .cloned()
            .map(|root| {
                let span = root.span;
                let expanded = transpiler.statement(root);
                match <[SemanticStatement; 1]>::try_from(expanded) {
                    Ok([only]) => only,
                    Err(many) => SemanticStatement {
                        kind: SemanticStatementKind::Line(many),
                        span,
                    },
                }
            })
            .collect();
        for index in 0..module.callables.len() {
            let name = base_name(&module.callables[index].name);
            let body = std::mem::take(&mut module.callables[index].body);
            transpiler.current_globals = collect_globals(&body);
            transpiler.current_function = Some(name);
            module.callables[index].body = transpiler.list(body);
            transpiler.current_function = None;
            transpiler.current_globals.clear();
        }
        if transpiler.unsupported {
            None
        } else {
            Some(module)
        }
    }

    fn build_records(&mut self) -> Option<()> {
        fn flatten(
            records: &[Record],
            name: &str,
            visiting: &mut Vec<String>,
            out: &mut Vec<EffectiveField>,
        ) -> Option<()> {
            let record = records
                .iter()
                .find(|record| record.name.eq_ignore_ascii_case(name))?;
            if visiting.iter().any(|seen| seen.eq_ignore_ascii_case(name)) {
                return None;
            }
            visiting.push(name.to_string());
            for combined in &record.combines {
                flatten(records, combined, visiting, out)?;
            }
            for field in &record.fields {
                let suffix = match &field.field_type {
                    RecordFieldType::String { .. } => TypeSuffix::String,
                    RecordFieldType::Int16 { .. } | RecordFieldType::Int { .. } => {
                        TypeSuffix::Integer
                    }
                    RecordFieldType::Int32 { .. } => TypeSuffix::Long,
                    RecordFieldType::Float32 { .. } => TypeSuffix::Single,
                    RecordFieldType::Float64 { .. } => TypeSuffix::Double,
                    RecordFieldType::Record { .. } => return None,
                };
                out.push(EffectiveField {
                    name: field.name.clone(),
                    suffix,
                });
            }
            visiting.pop();
            Some(())
        }
        for record in &self.source.records {
            let mut fields = Vec::new();
            flatten(&self.source.records, &record.name, &mut Vec::new(), &mut fields)?;
            self.records.insert(
                record.name.to_ascii_lowercase(),
                EffectiveRecord {
                    name: record.name.clone(),
                    fields,
                },
            );
        }
        Some(())
    }

    // ── statements ────────────────────────────────────────────────────────

    fn list(&mut self, statements: Vec<SemanticStatement>) -> Vec<SemanticStatement> {
        statements
            .into_iter()
            .flat_map(|statement| self.statement(statement))
            .collect()
    }

    /// Transpiles one statement into one or more statements at the same
    /// nesting level.
    fn statement(&mut self, statement: SemanticStatement) -> Vec<SemanticStatement> {
        let span = statement.span;
        let mut kind = statement.kind;
        match kind {
            SemanticStatementKind::Line(children) => {
                let children = self.list(children);
                return vec![SemanticStatement {
                    kind: SemanticStatementKind::Line(children),
                    span,
                }];
            }
            SemanticStatementKind::FileDeclaration {
                name,
                record_type,
                mut path,
                ..
            } => {
                let Some(record_type) = record_type else {
                    // Sequential file handles are not modeled yet.
                    self.unsupported = true;
                    return Vec::new();
                };
                self.expression(&mut path);
                return self.file_declaration(&name.name, &record_type.name, path, span);
            }
            SemanticStatementKind::Global { ref name, .. }
                if self.current_function.is_some()
                    && self.files.contains_key(&name.name.to_ascii_lowercase()) =>
            {
                return vec![comment(format!("' global {}", name.name), span)];
            }
            SemanticStatementKind::Assignment {
                target,
                operator,
                value,
            } => {
                return self.assignment(target, operator, value, span);
            }
            SemanticStatementKind::Expression(expression) => {
                if let Some(channel) = self.close_call(&expression) {
                    return vec![
                        comment(format!("' {}.close()", close_target(&expression)), span),
                        statement_of(SemanticStatementKind::Close(int(channel, span)), span),
                    ];
                }
                kind = SemanticStatementKind::Expression(expression);
            }
            other => kind = other,
        }

        // Ordinary statements: rewrite their own expressions, then recurse
        // into any nested bodies.
        self.statement_expressions(&mut kind);
        match &mut kind {
            SemanticStatementKind::If {
                then_body,
                else_body,
                ..
            } => {
                *then_body = self.list(std::mem::take(then_body));
                *else_body = self.list(std::mem::take(else_body));
            }
            SemanticStatementKind::While { body, .. }
            | SemanticStatementKind::For { body, .. }
            | SemanticStatementKind::Do { body, .. } => {
                *body = self.list(std::mem::take(body));
            }
            SemanticStatementKind::SelectCase {
                cases, else_body, ..
            } => {
                for case in cases.iter_mut() {
                    case.body = self.list(std::mem::take(&mut case.body));
                }
                *else_body = self.list(std::mem::take(else_body));
            }
            SemanticStatementKind::Try {
                body,
                catch,
                finally_body,
            } => {
                *body = self.list(std::mem::take(body));
                if let Some(catch) = catch {
                    catch.body = self.list(std::mem::take(&mut catch.body));
                }
                *finally_body = self.list(std::mem::take(finally_body));
            }
            _ => {}
        }
        vec![SemanticStatement { kind, span }]
    }

    // ── file declarations and close ──────────────────────────────────────

    fn file_fact(&self, name: &str) -> Option<&'a LoweredRecordFile> {
        let owner = self.current_function.as_deref();
        let source = self.source;
        source
            .lowered_record_files
            .iter()
            .find(|file| {
                file.name.eq_ignore_ascii_case(name)
                    && file.owner.as_deref().map(str::to_ascii_lowercase).as_deref() == owner
            })
            .or_else(|| {
                // A top-level file used from a function that declares `global`.
                source
                    .lowered_record_files
                    .iter()
                    .find(|file| file.name.eq_ignore_ascii_case(name) && file.owner.is_none())
            })
    }

    fn file_declaration(
        &mut self,
        name: &str,
        record_type: &str,
        path: Expression,
        span: SourceSpan,
    ) -> Vec<SemanticStatement> {
        let Some(fact) = self.file_fact(name) else {
            self.unsupported = true;
            return Vec::new();
        };
        let Some(record) = self.records.get(&record_type.to_ascii_lowercase()) else {
            self.unsupported = true;
            return Vec::new();
        };
        if record.fields.len() != fact.fields.len() {
            self.unsupported = true;
            return Vec::new();
        }
        self.files.insert(
            name.to_ascii_lowercase(),
            FileInfo {
                channel: fact.channel,
                record_type: record_type.to_string(),
            },
        );
        let bindings = record
            .fields
            .iter()
            .zip(&fact.fields)
            .map(|(_, layout)| {
                let buffer = BufferName::parse(&layout.buffer_name);
                FieldBinding {
                    length: int(layout.width as i64, span),
                    name: buffer.name,
                    name_span: span,
                    type_suffix: Some("$".to_string()),
                    is_string: true,
                }
            })
            .collect();
        vec![
            comment(
                format!(
                    "' file {name} as {record_type} = open(...)  [{} bytes/record]",
                    fact.record_length
                ),
                span,
            ),
            statement_of(
                SemanticStatementKind::Open {
                    path,
                    mode: OpenMode {
                        kind: OpenModeKind::Random,
                        span,
                    },
                    channel: int(fact.channel, span),
                    length: Some(int(fact.record_length as i64, span)),
                },
                span,
            ),
            statement_of(
                SemanticStatementKind::Field {
                    channel: int(fact.channel, span),
                    bindings,
                },
                span,
            ),
        ]
    }

    /// `x.close()` on a declared file: its channel.
    fn close_call(&mut self, expression: &Expression) -> Option<i64> {
        let target = close_target_name(expression)?;
        self.files
            .get(&target.to_ascii_lowercase())
            .map(|info| info.channel)
    }

    // ── assignments ───────────────────────────────────────────────────────

    fn assignment(
        &mut self,
        mut target: Expression,
        operator: AssignmentOperator,
        mut value: Expression,
        span: SourceSpan,
    ) -> Vec<SemanticStatement> {
        let plain = |target: Expression, value: Expression, this: &mut Self| {
            let mut target = target;
            let mut value = value;
            this.expression(&mut target);
            this.expression(&mut value);
            vec![statement_of(
                SemanticStatementKind::Assignment {
                    target,
                    operator,
                    value,
                },
                span,
            )]
        };
        if operator != AssignmentOperator::Assign {
            return plain(target, value, self);
        }

        // `x = { ... }`: initialize an in-memory record from a full literal.
        if let (ExpressionKind::Name(name), ExpressionKind::RecordLiteral(fields)) =
            (&target.kind, &value.kind)
        {
            let (name, fields) = (name.clone(), fields.clone());
            return self.record_literal_init(&name, fields, span);
        }
        // `a = b` where `b` is a record variable: member-wise copy.
        if let (ExpressionKind::Name(destination), ExpressionKind::Name(source)) =
            (&target.kind, &value.kind)
        {
            if let Some(record_type) = self.record_vars.get(&source.to_ascii_lowercase()).cloned()
            {
                let (destination, source) = (destination.clone(), source.clone());
                return self.record_copy(&destination, &source, &record_type, span);
            }
        }
        // `file[i] = { ... }` / `?{ ... }`: whole or partial record write.
        if let (ExpressionKind::Index { name, index }, literal) = (&target.kind, &value.kind) {
            let (fields, partial) = match literal {
                ExpressionKind::RecordLiteral(fields) => (Some(fields.clone()), false),
                ExpressionKind::PartialRecordLiteral(fields) => (Some(fields.clone()), true),
                _ => (None, false),
            };
            if let Some(fields) = fields {
                let (name, index) = (name.clone(), (**index).clone());
                return self.whole_write(&name, index, fields, partial, span);
            }
        }
        // `x = file[i]`: whole-record read.
        if let (ExpressionKind::Name(destination), ExpressionKind::Index { name, index }) =
            (&target.kind, &value.kind)
        {
            if self.files.contains_key(&name.to_ascii_lowercase()) {
                let (destination, name, index) =
                    (destination.clone(), name.clone(), (**index).clone());
                return self.whole_read(&destination, &name, index, span);
            }
        }
        // `file[i] = x` where `x` is a record variable: write it back.
        if let (ExpressionKind::Index { name, index }, ExpressionKind::Name(variable)) =
            (&target.kind, &value.kind)
        {
            if self.record_vars.contains_key(&variable.to_ascii_lowercase())
                && self.files.contains_key(&name.to_ascii_lowercase())
            {
                let (name, index, variable) = (name.clone(), (**index).clone(), variable.clone());
                return self.write_back(&name, index, &variable, span);
            }
        }
        // `file[i].field = value`: partial-field update.
        if let ExpressionKind::Member {
            base: Some(base),
            member,
            arguments: None,
        } = &target.kind
        {
            if let ExpressionKind::Index { name, index } = &base.kind {
                if self.files.contains_key(&name.to_ascii_lowercase()) {
                    let (name, index, field) = (name.clone(), (**index).clone(), member.clone());
                    return self.partial_update(&name, index, &field, value, span);
                }
            }
        }
        self.expression(&mut target);
        self.expression(&mut value);
        vec![statement_of(
            SemanticStatementKind::Assignment {
                target,
                operator,
                value,
            },
            span,
        )]
    }

    fn scalar_for(&self, variable: &str, field: &EffectiveField, span: SourceSpan) -> Expression {
        name_expr(
            &format!(
                "{}{}",
                camel_join(&[variable, &field.name]),
                field.suffix.to_string()
            ),
            value_type_of(field.suffix),
            span,
        )
    }

    fn record_literal_init(
        &mut self,
        variable: &str,
        pairs: Vec<FieldInitializer>,
        span: SourceSpan,
    ) -> Vec<SemanticStatement> {
        let supplied: HashSet<String> = pairs
            .iter()
            .map(|pair| pair.name.to_ascii_lowercase())
            .collect();
        // The literal's field set names its record type unambiguously.
        let candidates: Vec<&EffectiveRecord> = self
            .records
            .values()
            .filter(|record| {
                record.fields.len() == supplied.len()
                    && record
                        .fields
                        .iter()
                        .all(|field| supplied.contains(&field.name.to_ascii_lowercase()))
            })
            .collect();
        let [record] = candidates.as_slice() else {
            self.unsupported = true;
            return Vec::new();
        };
        let (record_name, fields) = (record.name.clone(), record.fields.clone());
        let mut values: HashMap<String, Expression> = pairs
            .into_iter()
            .map(|pair| (pair.name.to_ascii_lowercase(), pair.value))
            .collect();
        let mut out = Vec::new();
        for field in &fields {
            let Some(mut value) = values.remove(&field.name.to_ascii_lowercase()) else {
                self.unsupported = true;
                return Vec::new();
            };
            self.expression(&mut value);
            out.push(assign(self.scalar_for(variable, field, span), value, span));
        }
        self.record_vars
            .insert(variable.to_ascii_lowercase(), record_name);
        out
    }

    fn record_copy(
        &mut self,
        destination: &str,
        source: &str,
        record_type: &str,
        span: SourceSpan,
    ) -> Vec<SemanticStatement> {
        let Some(record) = self.records.get(&record_type.to_ascii_lowercase()) else {
            self.unsupported = true;
            return Vec::new();
        };
        let fields = record.fields.clone();
        let out = fields
            .iter()
            .map(|field| {
                assign(
                    self.scalar_for(destination, field, span),
                    self.scalar_for(source, field, span),
                    span,
                )
            })
            .collect();
        self.record_vars
            .insert(destination.to_ascii_lowercase(), record_type.to_string());
        out
    }

    fn file_and_record(&mut self, file: &str) -> Option<(FileInfo, Vec<EffectiveField>)> {
        let info = self.files.get(&file.to_ascii_lowercase())?.clone();
        let record = self.records.get(&info.record_type.to_ascii_lowercase())?;
        Some((info, record.fields.clone()))
    }

    fn buffer(&self, file: &str, field: &str, span: SourceSpan) -> Expression {
        name_expr(
            &format!("{}$", camel_join(&[file, field, "buf"])),
            SemanticValueType::String,
            span,
        )
    }

    /// `LSET`/`RSET buffer = pack(value)` for one field.
    fn store_field(
        &self,
        file: &str,
        field_index: usize,
        field: &EffectiveField,
        value: Expression,
        span: SourceSpan,
    ) -> Option<SemanticStatement> {
        let layout = self.layout(file, field_index)?;
        let packed = match layout.kind {
            LoweredRecordFieldKind::String { .. } => value,
            LoweredRecordFieldKind::Int16 => call("mki$", vec![value], SemanticValueType::String, span),
            LoweredRecordFieldKind::Int32 => call("mkl$", vec![value], SemanticValueType::String, span),
            LoweredRecordFieldKind::Float32 => call("mks$", vec![value], SemanticValueType::String, span),
            LoweredRecordFieldKind::Float64 => call("mkd$", vec![value], SemanticValueType::String, span),
        };
        let target = NamedReference {
            name: camel_join(&[file, &field.name, "buf"]) + "$",
            span,
        };
        let right = matches!(
            layout.kind,
            LoweredRecordFieldKind::String {
                right_aligned: true
            }
        );
        Some(statement_of(
            if right {
                SemanticStatementKind::Rset {
                    target,
                    value: packed,
                }
            } else {
                SemanticStatementKind::Lset {
                    target,
                    value: packed,
                }
            },
            span,
        ))
    }

    fn layout(&self, file: &str, field_index: usize) -> Option<&'a LoweredRecordField> {
        let fact = self.file_fact(file)?;
        fact.fields.get(field_index)
    }

    fn get_statement(&self, channel: i64, index: Expression, span: SourceSpan) -> SemanticStatement {
        statement_of(
            SemanticStatementKind::Get {
                channel: int(channel, span),
                position: Some(FilePosition {
                    position: Some(index),
                    record: None,
                }),
            },
            span,
        )
    }

    fn put_statement(&self, channel: i64, index: Expression, span: SourceSpan) -> SemanticStatement {
        statement_of(
            SemanticStatementKind::Put {
                channel: int(channel, span),
                position: Some(FilePosition {
                    position: Some(index),
                    record: None,
                }),
            },
            span,
        )
    }

    /// The `GET` that precedes a partial update, refusing a record beyond the
    /// end of the file exactly like the AST form.
    fn get_existing(
        &self,
        channel: i64,
        record_length: u32,
        index: &Expression,
        span: SourceSpan,
    ) -> Vec<SemanticStatement> {
        let bound = Expression {
            kind: ExpressionKind::Binary {
                left: Box::new(Expression {
                    kind: ExpressionKind::Parenthesized(Box::new(index.clone())),
                    span,
                    value_type: index.value_type,
                    record_type: None,
                }),
                operator: "*".to_string(),
                right: Box::new(int(record_length as i64, span)),
            },
            span,
            value_type: SemanticValueType::Integer,
            record_type: None,
        };
        let too_short = Expression {
            kind: ExpressionKind::Binary {
                left: Box::new(call(
                    "lof",
                    vec![int(channel, span)],
                    SemanticValueType::Long,
                    span,
                )),
                operator: "<".to_string(),
                right: Box::new(bound),
            },
            span,
            value_type: SemanticValueType::Integer,
            record_type: None,
        };
        vec![
            statement_of(
                SemanticStatementKind::If {
                    condition: too_short,
                    then_body: vec![statement_of(
                        SemanticStatementKind::Error(int(63, span)),
                        span,
                    )],
                    else_body: Vec::new(),
                    block: false,
                },
                span,
            ),
            self.get_statement(channel, index.clone(), span),
        ]
    }

    fn whole_write(
        &mut self,
        file: &str,
        mut index: Expression,
        pairs: Vec<FieldInitializer>,
        partial: bool,
        span: SourceSpan,
    ) -> Vec<SemanticStatement> {
        let Some((info, fields)) = self.file_and_record(file) else {
            self.unsupported = true;
            return Vec::new();
        };
        let Some(fact) = self.file_fact(file) else {
            self.unsupported = true;
            return Vec::new();
        };
        self.expression(&mut index);
        let literal = if partial { "?{ ... }" } else { "{ ... }" };
        let kind = if partial {
            "partial-record write"
        } else {
            "whole-record write"
        };
        let mut out = vec![comment(format!("' {file}[...] = {literal}  ({kind})"), span)];
        let mut provided: HashMap<String, Expression> = pairs
            .into_iter()
            .map(|pair| (pair.name.to_ascii_lowercase(), pair.value))
            .collect();
        let covers_every_field = fields
            .iter()
            .all(|field| provided.contains_key(&field.name.to_ascii_lowercase()));
        if partial && !covers_every_field {
            out.extend(self.get_existing(info.channel, fact.record_length, &index, span));
        }
        for (position, field) in fields.iter().enumerate() {
            let Some(mut value) = provided.remove(&field.name.to_ascii_lowercase()) else {
                continue;
            };
            self.expression(&mut value);
            let Some(store) = self.store_field(file, position, field, value, span) else {
                self.unsupported = true;
                return Vec::new();
            };
            out.push(store);
        }
        out.push(self.put_statement(info.channel, index, span));
        out
    }

    fn write_back(
        &mut self,
        file: &str,
        mut index: Expression,
        variable: &str,
        span: SourceSpan,
    ) -> Vec<SemanticStatement> {
        let Some((info, fields)) = self.file_and_record(file) else {
            self.unsupported = true;
            return Vec::new();
        };
        self.expression(&mut index);
        let mut out = vec![comment(
            format!("' {file}[...] = {variable}  (write back a let-bound record)"),
            span,
        )];
        for (position, field) in fields.iter().enumerate() {
            let value = self.scalar_for(variable, field, span);
            let Some(store) = self.store_field(file, position, field, value, span) else {
                self.unsupported = true;
                return Vec::new();
            };
            out.push(store);
        }
        out.push(self.put_statement(info.channel, index, span));
        out
    }

    fn whole_read(
        &mut self,
        variable: &str,
        file: &str,
        mut index: Expression,
        span: SourceSpan,
    ) -> Vec<SemanticStatement> {
        let Some((info, fields)) = self.file_and_record(file) else {
            self.unsupported = true;
            return Vec::new();
        };
        self.expression(&mut index);
        let mut out = vec![
            comment(
                format!("' let {variable} = {file}[...]  (whole-record read)"),
                span,
            ),
            self.get_statement(info.channel, index, span),
        ];
        for (position, field) in fields.iter().enumerate() {
            let Some(layout) = self.layout(file, position) else {
                self.unsupported = true;
                return Vec::new();
            };
            let buffer = self.buffer(file, &field.name, span);
            let target = self.scalar_for(variable, field, span);
            let unpack = match layout.kind {
                LoweredRecordFieldKind::Int16 => Some("cvi"),
                LoweredRecordFieldKind::Int32 => Some("cvl"),
                LoweredRecordFieldKind::Float32 => Some("cvs"),
                LoweredRecordFieldKind::Float64 => Some("cvd"),
                LoweredRecordFieldKind::String { .. } => None,
            };
            match unpack {
                Some(function) => out.push(assign(
                    target.clone(),
                    call(function, vec![buffer], target.value_type, span),
                    span,
                )),
                None => out.extend(trim_statements(
                    &buffer,
                    &name_expr(
                        &format!("{}%", camel_join(&[variable, &field.name, "trimI"])),
                        SemanticValueType::Integer,
                        span,
                    ),
                    &target,
                    span,
                )),
            }
        }
        self.record_vars
            .insert(variable.to_ascii_lowercase(), info.record_type);
        out
    }

    fn partial_update(
        &mut self,
        file: &str,
        mut index: Expression,
        field_name: &str,
        mut value: Expression,
        span: SourceSpan,
    ) -> Vec<SemanticStatement> {
        let Some((info, fields)) = self.file_and_record(file) else {
            self.unsupported = true;
            return Vec::new();
        };
        let Some(fact) = self.file_fact(file) else {
            self.unsupported = true;
            return Vec::new();
        };
        let Some(position) = fields
            .iter()
            .position(|field| field.name.eq_ignore_ascii_case(field_name))
        else {
            self.unsupported = true;
            return Vec::new();
        };
        self.expression(&mut index);
        self.expression(&mut value);
        let mut out = vec![comment(
            format!("' {file}[...].{field_name} = ...  (partial-field update)"),
            span,
        )];
        out.extend(self.get_existing(info.channel, fact.record_length, &index, span));
        let Some(store) = self.store_field(file, position, &fields[position], value, span) else {
            self.unsupported = true;
            return Vec::new();
        };
        out.push(store);
        out.push(self.put_statement(info.channel, index, span));
        out
    }

    // ── expressions ───────────────────────────────────────────────────────

    /// Rewrites `recordVar.field` names to their unpacked scalars and wraps
    /// the numeric side of a string/number `+` in `STR$`.
    fn expression(&mut self, expression: &mut Expression) -> Option<FieldKind> {
        match &mut expression.kind {
            ExpressionKind::Literal(text) => {
                if expression.value_type == SemanticValueType::String || text.starts_with('"') {
                    Some(FieldKind::Stringy)
                } else {
                    Some(FieldKind::Numeric)
                }
            }
            ExpressionKind::Name(name) => {
                let (variable, field) = name.split_once('.')?;
                let record_type = self.record_vars.get(&variable.to_ascii_lowercase())?.clone();
                let record = self.records.get(&record_type.to_ascii_lowercase())?;
                let field = record
                    .fields
                    .iter()
                    .find(|candidate| candidate.name.eq_ignore_ascii_case(field))?
                    .clone();
                let scalar = self.scalar_for(variable, &field, expression.span);
                let kind = if field.suffix == TypeSuffix::String {
                    FieldKind::Stringy
                } else {
                    FieldKind::Numeric
                };
                *expression = scalar;
                Some(kind)
            }
            ExpressionKind::Parenthesized(inner) => {
                let kind = self.expression(inner);
                expression.value_type = inner.value_type;
                kind
            }
            ExpressionKind::Unary { operand, .. } => {
                let kind = self.expression(operand);
                kind
            }
            ExpressionKind::Binary {
                left,
                operator,
                right,
            } => {
                let left_kind = self.expression(left);
                let right_kind = self.expression(right);
                if operator == "+" {
                    let wrap = |operand: &mut Box<Expression>| {
                        let inner = std::mem::replace(
                            &mut **operand,
                            int(0, expression.span),
                        );
                        let span = inner.span;
                        **operand = call("str$", vec![inner], SemanticValueType::String, span);
                    };
                    match (left_kind, right_kind) {
                        (Some(FieldKind::Stringy), Some(FieldKind::Numeric)) => {
                            wrap(right);
                            expression.value_type = SemanticValueType::String;
                            return Some(FieldKind::Stringy);
                        }
                        (Some(FieldKind::Numeric), Some(FieldKind::Stringy)) => {
                            wrap(left);
                            expression.value_type = SemanticValueType::String;
                            return Some(FieldKind::Stringy);
                        }
                        (Some(FieldKind::Stringy), Some(FieldKind::Stringy)) => {
                            return Some(FieldKind::Stringy);
                        }
                        (Some(FieldKind::Numeric), Some(FieldKind::Numeric)) => {
                            return Some(FieldKind::Numeric);
                        }
                        _ => {}
                    }
                }
                None
            }
            ExpressionKind::Call { arguments, .. } => {
                for argument in arguments.iter_mut() {
                    self.expression(argument);
                }
                None
            }
            ExpressionKind::Index { index, .. } => {
                self.expression(index);
                None
            }
            ExpressionKind::MultiIndex { indices, .. } => {
                for index in indices.iter_mut() {
                    self.expression(index);
                }
                None
            }
            ExpressionKind::Member {
                base, arguments, ..
            } => {
                if let Some(base) = base {
                    self.expression(base);
                }
                if let Some(arguments) = arguments {
                    for argument in arguments.iter_mut() {
                        self.expression(argument);
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// Rewrites every expression a statement owns (not its nested bodies).
    fn statement_expressions(&mut self, kind: &mut SemanticStatementKind) {
        use SemanticStatementKind as K;
        match kind {
            K::Assignment { target, value, .. } => {
                self.expression(target);
                self.expression(value);
            }
            K::MidAssign {
                target,
                start,
                length,
                value,
            } => {
                self.expression(target);
                self.expression(start);
                if let Some(length) = length {
                    self.expression(length);
                }
                self.expression(value);
            }
            K::Expression(expression)
            | K::Error(expression)
            | K::OptionBase(expression)
            | K::Kill(expression)
            | K::Close(expression) => {
                self.expression(expression);
            }
            K::Return(ReturnValue::Value(expression))
            | K::Throw(ThrowValue::Value(expression)) => {
                self.expression(expression);
            }
            K::If { condition, .. } | K::While { condition, .. } => {
                self.expression(condition);
            }
            K::For { start, bounds, .. } => {
                self.expression(start);
                if let ForBounds::To { limit, step } = bounds {
                    self.expression(limit);
                    if let Some(step) = step {
                        self.expression(step);
                    }
                }
                if let ForBounds::Downto { limit, .. } = bounds {
                    self.expression(limit);
                }
            }
            K::Do {
                pre_condition,
                post_condition,
                ..
            } => {
                for condition in pre_condition.iter_mut().chain(post_condition.iter_mut()) {
                    self.expression(&mut condition.value);
                }
            }
            K::SelectCase {
                selector, cases, ..
            } => {
                self.expression(selector);
                for case in cases {
                    for value in &mut case.values {
                        match value {
                            CaseValue::Comparison { value, .. } => {
                                self.expression(value);
                            }
                            CaseValue::Value {
                                first, range_end, ..
                            } => {
                                self.expression(first);
                                if let Some(end) = range_end {
                                    self.expression(end);
                                }
                            }
                        }
                    }
                }
            }
            K::Try { catch, .. } => {
                if let Some(catch) = catch {
                    for filter in &mut catch.filters {
                        self.expression(filter);
                    }
                }
            }
            K::Print {
                destination,
                tokens,
            } => {
                match destination {
                    PrintDestination::Standard { .. } => {}
                    PrintDestination::Using { format, .. } => {
                        self.expression(format);
                    }
                    PrintDestination::Channel { channel, using, .. } => {
                        self.expression(channel);
                        if let Some(using) = using {
                            self.expression(using);
                        }
                    }
                }
                for token in tokens {
                    if let PrintToken::Expression(expression) = token {
                        self.expression(expression);
                    }
                }
            }
            K::Input { source, targets } => {
                if let InputSource::Channel(channel) = source {
                    self.expression(channel);
                }
                for target in targets {
                    self.expression(target);
                }
            }
            K::Write { channel, values } => {
                self.expression(channel);
                if let WriteValues::Values(values) = values {
                    for value in values {
                        self.expression(value);
                    }
                }
            }
            K::OnBranch { selector, .. } => {
                self.expression(selector);
            }
            K::Const { value, .. } => {
                self.expression(value);
            }
            K::Read(targets) | K::Data(targets) => {
                for target in targets {
                    self.expression(target);
                }
            }
            K::Swap { left, right } => {
                self.expression(left);
                self.expression(right);
            }
            K::Lset { value, .. } | K::Rset { value, .. } => {
                self.expression(value);
            }
            K::LineInput { channel, target } => {
                self.expression(channel);
                self.expression(target);
            }
            K::Locate { row, column } => {
                self.expression(row);
                self.expression(column);
            }
            K::Color {
                foreground,
                background,
            } => {
                self.expression(foreground);
                if let Some(background) = background {
                    self.expression(background);
                }
            }
            _ => {}
        }
    }
}

// ── builders ────────────────────────────────────────────────────────────

/// A synthesized buffer variable name split into `name` + suffix.
struct BufferName {
    name: String,
}

impl BufferName {
    fn parse(basic: &str) -> Self {
        Self {
            name: basic.trim_end_matches('$').to_string(),
        }
    }
}

fn statement_of(kind: SemanticStatementKind, span: SourceSpan) -> SemanticStatement {
    SemanticStatement { kind, span }
}

fn comment(text: String, span: SourceSpan) -> SemanticStatement {
    statement_of(
        SemanticStatementKind::Comment { block: false, text },
        span,
    )
}

fn assign(target: Expression, value: Expression, span: SourceSpan) -> SemanticStatement {
    statement_of(
        SemanticStatementKind::Assignment {
            target,
            operator: AssignmentOperator::Assign,
            value,
        },
        span,
    )
}

fn int(value: i64, span: SourceSpan) -> Expression {
    Expression {
        kind: ExpressionKind::Literal(value.to_string()),
        span,
        value_type: SemanticValueType::Integer,
        record_type: None,
    }
}

fn name_expr(name: &str, value_type: SemanticValueType, span: SourceSpan) -> Expression {
    Expression {
        kind: ExpressionKind::Name(name.to_string()),
        span,
        value_type,
        record_type: None,
    }
}

fn call(
    name: &str,
    arguments: Vec<Expression>,
    value_type: SemanticValueType,
    span: SourceSpan,
) -> Expression {
    Expression {
        kind: ExpressionKind::Call {
            name: name.to_string(),
            arguments,
        },
        span,
        value_type,
        record_type: None,
    }
}

fn value_type_of(suffix: TypeSuffix) -> SemanticValueType {
    match suffix {
        TypeSuffix::Integer => SemanticValueType::Integer,
        TypeSuffix::Long => SemanticValueType::Long,
        TypeSuffix::Single => SemanticValueType::Single,
        TypeSuffix::Double => SemanticValueType::Double,
        TypeSuffix::String => SemanticValueType::String,
    }
}

fn binary(
    left: Expression,
    operator: &str,
    right: Expression,
    value_type: SemanticValueType,
    span: SourceSpan,
) -> Expression {
    Expression {
        kind: ExpressionKind::Binary {
            left: Box::new(left),
            operator: operator.to_string(),
            right: Box::new(right),
        },
        span,
        value_type,
        record_type: None,
    }
}

/// `counter = LEN(buf$) : WHILE counter > 0 && MID$(buf$, counter, 1) = " " :
/// counter = counter - 1 : WEND : target$ = LEFT$(buf$, counter)` -- an inline
/// right-trim built from `LEN`/`MID$`/`LEFT$`, which every target has.
fn trim_statements(
    buffer: &Expression,
    counter: &Expression,
    target: &Expression,
    span: SourceSpan,
) -> Vec<SemanticStatement> {
    let integer = SemanticValueType::Integer;
    let init = assign(
        counter.clone(),
        call("len", vec![buffer.clone()], integer, span),
        span,
    );
    let condition = binary(
        binary(counter.clone(), ">", int(0, span), integer, span),
        "&&",
        binary(
            call(
                "mid$",
                vec![buffer.clone(), counter.clone(), int(1, span)],
                SemanticValueType::String,
                span,
            ),
            "=",
            Expression {
                kind: ExpressionKind::Literal("\" \"".to_string()),
                span,
                value_type: SemanticValueType::String,
                record_type: None,
            },
            integer,
            span,
        ),
        integer,
        span,
    );
    let decrement = assign(
        counter.clone(),
        binary(counter.clone(), "-", int(1, span), integer, span),
        span,
    );
    let while_loop = statement_of(
        SemanticStatementKind::While {
            condition,
            body: vec![decrement],
        },
        span,
    );
    let finalize = assign(
        target.clone(),
        call(
            "left$",
            vec![buffer.clone(), counter.clone()],
            SemanticValueType::String,
            span,
        ),
        span,
    );
    vec![init, while_loop, finalize]
}

/// A callable's name without its type suffix, lowercased -- how the AST pass
/// records a function as a file's owner.
fn base_name(name: &str) -> String {
    name.trim_end_matches(['$', '%', '!', '#', '&'])
        .to_ascii_lowercase()
}

fn close_target_name(expression: &Expression) -> Option<String> {
    match &expression.kind {
        ExpressionKind::Call { name, arguments } if arguments.is_empty() => name
            .strip_suffix(".close")
            .or_else(|| name.strip_suffix(".CLOSE"))
            .map(str::to_string),
        _ => None,
    }
}

fn close_target(expression: &Expression) -> String {
    close_target_name(expression).unwrap_or_default()
}

/// Names declared with `global` anywhere in a callable body.
fn collect_globals(body: &[SemanticStatement]) -> HashSet<String> {
    fn visit(statements: &[SemanticStatement], names: &mut HashSet<String>) {
        for statement in statements {
            match &statement.kind {
                SemanticStatementKind::Global { name, .. } => {
                    names.insert(name.name.to_ascii_lowercase());
                }
                SemanticStatementKind::Line(body)
                | SemanticStatementKind::While { body, .. }
                | SemanticStatementKind::For { body, .. }
                | SemanticStatementKind::Do { body, .. } => visit(body, names),
                SemanticStatementKind::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    visit(then_body, names);
                    visit(else_body, names);
                }
                SemanticStatementKind::SelectCase {
                    cases, else_body, ..
                } => {
                    for case in cases {
                        visit(&case.body, names);
                    }
                    visit(else_body, names);
                }
                SemanticStatementKind::Try {
                    body,
                    catch,
                    finally_body,
                } => {
                    visit(body, names);
                    if let Some(catch) = catch {
                        visit(&catch.body, names);
                    }
                    visit(finally_body, names);
                }
                _ => {}
            }
        }
    }
    let mut names = HashSet::new();
    visit(body, &mut names);
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The typed module for `source` as the driver builds it: AST lowering
    /// supplies the file facts, then the pass runs.
    pub(crate) fn transpiled(source: &str) -> Option<SemanticModule> {
        let parsed = crate::driver::parse_source("rt.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered {
            lowered_record_files,
            ..
        } = crate::lower::lower(parsed).unwrap();
        let mut module = parse_and_adapt_named("rt.bcl", source).unwrap();
        module.lowered_record_files = lowered_record_files;
        transpile(&mut module).then_some(module)
    }

    /// The statement kinds of a module, flattened through `Line` wrappers, as
    /// short names.
    fn shape(module: &SemanticModule) -> Vec<&'static str> {
        fn visit(statements: &[SemanticStatement], out: &mut Vec<&'static str>) {
            for statement in statements {
                out.push(match &statement.kind {
                    SemanticStatementKind::Line(children) => {
                        visit(children, out);
                        continue;
                    }
                    SemanticStatementKind::Comment { .. } => "comment",
                    SemanticStatementKind::Open { .. } => "open",
                    SemanticStatementKind::Field { .. } => "field",
                    SemanticStatementKind::Get { .. } => "get",
                    SemanticStatementKind::Put { .. } => "put",
                    SemanticStatementKind::Lset { .. } => "lset",
                    SemanticStatementKind::Rset { .. } => "rset",
                    SemanticStatementKind::Close(_) => "close",
                    SemanticStatementKind::Assignment { .. } => "assign",
                    SemanticStatementKind::While { .. } => "while",
                    SemanticStatementKind::If { .. } => "if",
                    SemanticStatementKind::Print { .. } => "print",
                    SemanticStatementKind::End => "end",
                    _ => "other",
                });
            }
        }
        let mut out = Vec::new();
        visit(&module.statements, &mut out);
        out
    }

    const RECORD_PREFIX: &str = "record Part\n    flag: string(1) lpad\n    desc: string(20) lpad\n    qty: int16\n    price: float32\nend record\nfile inv as Part = open(\"inven.dat\")\n";

    fn program(body: &str) -> String {
        format!("program rt\n{RECORD_PREFIX}{body}end\n")
    }

    #[test]
    fn file_declaration_expands_to_open_and_field() {
        let module = transpiled(&program("")).expect("pass applies");
        assert!(module.records_transpiled);
        assert_eq!(shape(&module), ["comment", "open", "field", "end"]);
    }

    #[test]
    fn whole_write_is_lset_per_field_then_put() {
        let module = transpiled(&program(
            "inv[1] = { flag: \"A\", desc: \"x\", qty: 3, price: 1.5 }\n",
        ))
        .unwrap();
        assert_eq!(
            shape(&module)[3..],
            ["comment", "lset", "lset", "lset", "lset", "put", "end"]
        );
    }

    #[test]
    fn partial_write_of_a_subset_reads_the_record_first() {
        let module = transpiled(&program("inv[2] = ?{ qty: 7 }\n")).unwrap();
        assert_eq!(
            shape(&module)[3..],
            ["comment", "if", "get", "lset", "put", "end"]
        );
        // Naming every field needs no read, exactly like a full literal.
        let module = transpiled(&program(
            "inv[2] = ?{ flag: \"A\", desc: \"x\", qty: 7, price: 1.0 }\n",
        ))
        .unwrap();
        assert!(!shape(&module).contains(&"get"));
    }

    #[test]
    fn whole_read_unpacks_every_field_and_trims_strings() {
        let module = transpiled(&program("let p = inv[1]\nprint p.desc + p.qty\n")).unwrap();
        assert_eq!(
            shape(&module)[3..],
            [
                "comment", "get", // read
                "assign", "while", "assign", // flag: trim
                "assign", "while", "assign", // desc: trim
                "assign", "assign", // qty, price: unpack
                "print", "end"
            ]
        );
        // The string/number mix gets STR$ around the number.
        let print = module
            .statements
            .iter()
            .flat_map(|root| match &root.kind {
                SemanticStatementKind::Line(children) => children.iter().collect::<Vec<_>>(),
                _ => vec![root],
            })
            .find_map(|statement| match &statement.kind {
                SemanticStatementKind::Print { tokens, .. } => Some(format!("{tokens:?}")),
                _ => None,
            })
            .unwrap();
        assert!(print.contains("\"str$\""), "{print}");
        assert!(print.contains("pDesc$") && print.contains("pQty%"), "{print}");
    }

    #[test]
    fn write_back_close_and_global_comment() {
        let module = transpiled(&program(
            "let p = inv[1]\ninv[2] = p\ninv.close()\n",
        ))
        .unwrap();
        let shape = shape(&module);
        let expected = [
            "comment", "lset", "lset", "lset", "lset", "put", "comment", "close", "end",
        ];
        assert_eq!(shape[shape.len() - expected.len()..], expected, "{shape:?}");
    }

    #[test]
    fn record_literal_init_and_copy_are_scalar_assignments() {
        let module = transpiled(&program(
            "a = { flag: \"A\", desc: \"x\", qty: 3, price: 1.5 }\nb = a\n",
        ))
        .unwrap();
        // four field assignments for the literal, four for the copy
        assert_eq!(
            shape(&module)[3..],
            ["assign", "assign", "assign", "assign", "assign", "assign", "assign", "assign", "end"]
        );
    }

    #[test]
    fn programs_without_record_files_are_left_alone() {
        let parsed = crate::driver::parse_source("rt.bcl".to_string(), "print 1\nend\n").unwrap();
        let crate::lower::Lowered {
            lowered_record_files,
            ..
        } = crate::lower::lower(parsed).unwrap();
        let mut module = parse_and_adapt_named("rt.bcl", "print 1\nend\n").unwrap();
        module.lowered_record_files = lowered_record_files;
        let before = module.clone();
        assert!(!transpile(&mut module));
        assert_eq!(module, before);
    }
}
