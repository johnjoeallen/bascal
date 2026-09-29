//! Resolved-IR building blocks derived directly from the generated frontend.
//!
//! This module is intentionally independent of the legacy `ast` module. It
//! is the first destination for semantic nodes produced by rdgen; later
//! adapters add declarations, statements, expressions, and resolver facts.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use crate::rdgen_frontend::{self, SourceSpan};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SemanticModule {
    pub span: SourceSpan,
    /// Original source retained so semantic diagnostics keep their filename and position.
    pub sources: Vec<SemanticSource>,
    pub header: Option<ModuleHeader>,
    pub dependencies: Vec<Dependency>,
    pub records: Vec<Record>,
    /// Record-file layouts resolved by the record/file desugaring pass.
    /// These are explicit typed-IR facts because their channel numbers and
    /// buffer bindings are synthesized after parsing.
    pub lowered_record_files: Vec<LoweredRecordFile>,
    pub callables: Vec<CallableSignature>,
    pub statements: Vec<SemanticStatement>,
    /// Source index for each top-level statement. `usize::MAX` means that the
    /// statement came from an adapter that did not retain source identity.
    /// Nested statements inherit the top-level statement's source.
    pub statement_sources: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoweredRecordFile {
    pub name: String,
    pub channel: i64,
    pub record_type: String,
    /// Resolved fixed-width record size used by random-access file I/O.
    pub record_length: u32,
    pub owner: Option<String>,
    pub fields: Vec<LoweredRecordField>,
    /// Typed scalar temporaries synthesized while unpacking record values.
    pub record_locals: Vec<LoweredRecordLocal>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoweredRecordLocal {
    pub name: String,
    pub value_type: SemanticValueType,
    pub owner: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoweredRecordField {
    pub buffer_name: String,
    pub width: u32,
    /// Byte offset from the start of the packed record.
    pub offset: u32,
    pub kind: LoweredRecordFieldKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoweredRecordFieldKind {
    Int16,
    Int32,
    Float32,
    Float64,
    String { right_aligned: bool },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SemanticSource {
    pub filename: String,
    pub text: String,
}

impl SemanticSource {
    /// Resolve a generated frontend span to a source location without using
    /// compatibility AST positions. Leading trivia within the span is
    /// skipped so diagnostics point at the first source token.
    pub fn source_position(&self, span: SourceSpan) -> crate::diagnostics::SourcePos {
        let mut start = span.start.min(self.text.len());
        while !self.text.is_char_boundary(start) {
            start -= 1;
        }
        let mut end = span.end.min(self.text.len());
        while !self.text.is_char_boundary(end) {
            end -= 1;
        }
        end = end.max(start);

        let mut offset = start;
        while offset < end {
            let character = self.text[offset..]
                .chars()
                .next()
                .expect("offset is before a valid UTF-8 boundary");
            if !character.is_whitespace() {
                break;
            }
            offset += character.len_utf8();
        }

        let prefix = &self.text[..offset];
        let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
        let column = prefix.rsplit('\n').next().unwrap_or("").chars().count() + 1;
        crate::diagnostics::SourcePos::new(self.filename.clone(), line, column)
    }

    /// Resolve a byte offset for source-to-AST alignment. Unlike diagnostic
    /// positioning, alignment must decline malformed offsets instead of
    /// clamping them to a potentially unrelated AST node.
    pub fn source_position_at(&self, offset: usize) -> Option<crate::diagnostics::SourcePos> {
        let tail = self.text.get(offset..)?;
        let leading_trivia = tail
            .char_indices()
            .find(|(_, character)| !character.is_whitespace())?
            .0;
        let prefix = self.text.get(..offset + leading_trivia)?;
        let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
        let column = prefix.rsplit('\n').next()?.chars().count() + 1;
        Some(crate::diagnostics::SourcePos::new(
            self.filename.clone(),
            line,
            column,
        ))
    }
}

/// Name visibility facts derived from the generated semantic module.  This is
/// deliberately a fact table rather than a backend-specific symbol table:
/// code generators can use it for collision checks without re-walking parser
/// nodes or re-inferring names from emitted syntax.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SemanticNameScopes {
    pub global_names: BTreeSet<String>,
    pub callable_globals: HashMap<String, BTreeSet<String>>,
    /// Resolved scalar types for names introduced by GLOBAL declarations.
    pub global_types: BTreeMap<String, SemanticValueType>,
}

fn collect_const_types(
    statements: &[SemanticStatement],
    types: &mut BTreeMap<String, SemanticValueType>,
) {
    fn initializer_type(
        expression: &Expression,
        types: &BTreeMap<String, SemanticValueType>,
    ) -> SemanticValueType {
        match &expression.kind {
            ExpressionKind::Name(name) => types
                .iter()
                .find(|(constant, _)| constant.eq_ignore_ascii_case(name))
                .map(|(_, value_type)| *value_type)
                .filter(|value_type| *value_type != SemanticValueType::Unknown)
                .unwrap_or_else(|| SemanticValueType::from_suffix(name.chars().last())),
            ExpressionKind::Parenthesized(inner) | ExpressionKind::Unary { operand: inner, .. } => {
                initializer_type(inner, types)
            }
            ExpressionKind::Binary {
                left,
                operator,
                right,
            } => {
                let left_type = initializer_type(left, types);
                let right_type = initializer_type(right, types);
                binary_expression_type(operator, left_type, right_type)
            }
            _ => expression.value_type,
        }
    }

    for statement in statements {
        match &statement.kind {
            SemanticStatementKind::Const { name, value, .. } => {
                let suffix =
                    name.name.chars().last().and_then(|character| {
                        SemanticValueType::from_suffix(Some(character)).suffix()
                    });
                let value_type = if let Some(character) = suffix {
                    SemanticValueType::from_suffix(Some(character))
                } else {
                    match initializer_type(value, types) {
                        SemanticValueType::Unknown => continue,
                        SemanticValueType::Boolean => SemanticValueType::Integer,
                        // Unsuffixed scalar bindings default to single precision
                        // in BASCAL, including double-valued initializers.
                        SemanticValueType::Double => SemanticValueType::Single,
                        value_type => value_type,
                    }
                };
                types.insert(name.name.clone(), value_type);
            }
            SemanticStatementKind::Line(body)
            | SemanticStatementKind::While { body, .. }
            | SemanticStatementKind::For { body, .. }
            | SemanticStatementKind::Do { body, .. } => collect_const_types(body, types),
            SemanticStatementKind::If {
                then_body,
                else_body,
                ..
            } => {
                collect_const_types(then_body, types);
                collect_const_types(else_body, types);
            }
            SemanticStatementKind::SelectCase {
                cases, else_body, ..
            } => {
                for case in cases {
                    collect_const_types(&case.body, types);
                }
                collect_const_types(else_body, types);
            }
            SemanticStatementKind::Try {
                body,
                catch,
                finally_body,
            } => {
                collect_const_types(body, types);
                if let Some(catch) = catch {
                    collect_const_types(&catch.body, types);
                }
                collect_const_types(finally_body, types);
            }
            _ => {}
        }
    }
}

fn binary_expression_type(
    operator: &str,
    left: SemanticValueType,
    right: SemanticValueType,
) -> SemanticValueType {
    let operator = operator.trim().to_ascii_uppercase();
    if matches!(operator.as_str(), "=" | "<>" | "<" | ">" | "<=" | ">=") {
        return SemanticValueType::Boolean;
    }
    if matches!(operator.as_str(), "AND" | "OR" | "XOR" | "EQV" | "IMP")
        && left == SemanticValueType::Boolean
        && right == SemanticValueType::Boolean
    {
        return SemanticValueType::Boolean;
    }
    if operator == "+" && (left == SemanticValueType::String || right == SemanticValueType::String)
    {
        return SemanticValueType::String;
    }
    if operator == "/" {
        return if left == SemanticValueType::Double || right == SemanticValueType::Double {
            SemanticValueType::Double
        } else {
            SemanticValueType::Single
        };
    }
    if matches!(operator.as_str(), "\\" | "MOD") {
        return if left == SemanticValueType::Long || right == SemanticValueType::Long {
            SemanticValueType::Long
        } else {
            SemanticValueType::Integer
        };
    }
    match (left, right) {
        (SemanticValueType::Double, _) | (_, SemanticValueType::Double) => {
            SemanticValueType::Double
        }
        (SemanticValueType::Single, _) | (_, SemanticValueType::Single) => {
            SemanticValueType::Single
        }
        (SemanticValueType::Long, _) | (_, SemanticValueType::Long) => SemanticValueType::Long,
        (SemanticValueType::Unknown, value_type) | (value_type, SemanticValueType::Unknown) => {
            value_type
        }
        (value_type, _) => value_type,
    }
}

impl SemanticModule {
    /// Whether the final top-level semantic statement explicitly terminates
    /// the program. Backends use this instead of inspecting their compatibility
    /// AST when semantic IR is available.
    pub fn ends_with_end(&self) -> bool {
        let Some(last) = self.statements.last() else {
            return false;
        };
        match &last.kind {
            SemanticStatementKind::Line(nodes) => nodes
                .last()
                .is_some_and(|node| matches!(&node.kind, SemanticStatementKind::End)),
            kind => matches!(kind, SemanticStatementKind::End),
        }
    }

    /// Names mentioned by a statement body, including declarations, targets,
    /// and expression operands, in BASIC's case-insensitive spelling.
    pub fn names_in_statements(statements: &[SemanticStatement]) -> BTreeSet<String> {
        let mut names = BTreeSet::new();
        collect_semantic_statements(statements, &mut names);
        names
    }

    /// Typed identifier occurrences in expression positions. Unlike
    /// [`names_in_statements`], this retains the resolver's value type for
    /// each scalar or array reference so codegen can declare storage without
    /// deriving a type from the identifier suffix.
    pub fn typed_names_in_statements(
        statements: &[SemanticStatement],
    ) -> Vec<(String, SemanticValueType)> {
        let mut names = TypedSemanticNames::default();
        collect_semantic_statements(statements, &mut names);
        names.typed
    }

    pub fn global_declarations_in(statements: &[SemanticStatement]) -> BTreeSet<String> {
        let mut names = BTreeSet::new();
        collect_semantic_global_declarations(statements, &mut names, &mut BTreeMap::new());
        names
    }

    /// Find advisory warnings for legacy constructs in generated semantic statements.
    pub fn legacy_form_diagnostics(&self) -> Vec<crate::diagnostics::Diagnostic> {
        fn walk(
            statements: &[SemanticStatement],
            source: &SemanticSource,
            labels: &mut BTreeSet<String>,
            out: &mut Vec<crate::diagnostics::Diagnostic>,
        ) {
            use SemanticStatementKind as Kind;
            fn depth(statement: &SemanticStatement) -> Option<usize> {
                let Kind::If {
                    then_body,
                    else_body,
                    ..
                } = &statement.kind
                else {
                    return None;
                };
                if !matches!(then_body.as_slice(), [item] if matches!(item.kind, Kind::Goto(_))) {
                    return None;
                }
                match else_body.as_slice() {
                    [] => Some(1),
                    [item] if matches!(item.kind, Kind::Goto(_)) => Some(1),
                    [item] if matches!(item.kind, Kind::If { .. }) => {
                        depth(item).map(|value| value + 1)
                    }
                    _ => None,
                }
            }
            for statement in statements {
                let chain = matches!(statement.kind, Kind::If { .. })
                    .then(|| depth(statement))
                    .flatten();
                let message = match &statement.kind {
                    Kind::Label(name) => { labels.insert(name.name.to_ascii_lowercase()); None }
                    Kind::OnBranch { branch, .. } => Some(if matches!(branch, BranchKind::Gosub) { "`ON ... GOSUB` is a legacy computed-branch dispatch with a direct BASCAL equivalent -- prefer `SELECT CASE`, which expresses the same dispatch without a positional branch-target list" } else { "`ON ... GOTO` is a legacy computed-branch dispatch with a direct BASCAL equivalent -- prefer `SELECT CASE`, which expresses the same dispatch without a positional branch-target list" }.to_string()),
                    Kind::If { .. } if chain.is_some_and(|value| value >= 2) => Some("this `IF ... THEN GOTO` / `ELSEIF ... THEN GOTO` chain dispatches to different labels by condition -- prefer `SELECT CASE`, BASCAL's structured equivalent for the same dispatch".to_string()),
                    Kind::Goto(target) if labels.contains(&target.name.to_ascii_lowercase()) => Some(format!("`GOTO {}` jumps back to a label already seen earlier in this block -- this is a hand-wired loop with a direct BASCAL equivalent -- prefer `DO ... LOOP` or `WHILE ... END WHILE`", target.name)),
                    Kind::Field { .. } => Some("hand-written `FIELD` bookkeeping has a direct BASCAL equivalent -- a `record ... end record` + `file ... as ... = open(...)` declaration expresses the same random-access layout without manually tracking FIELD/LSET/GET/PUT offsets".to_string()),
                    _ => None,
                };
                if let Some(message) = message {
                    let start = statement.span.start.min(source.text.len());
                    let end = statement.span.end.min(source.text.len()).max(start);
                    let text = &source.text[start..end];
                    let start = start + text.len() - text.trim_start().len();
                    let prefix = &source.text[..start];
                    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
                    let column = prefix.rsplit('\n').next().unwrap_or("").chars().count() + 1;
                    out.push(crate::diagnostics::Diagnostic::warning(
                        crate::diagnostics::SourcePos::new(source.filename.clone(), line, column),
                        message,
                    ));
                }
                if chain.is_some_and(|value| value >= 2) {
                    continue;
                }
                match &statement.kind {
                    Kind::Line(body)
                    | Kind::While { body, .. }
                    | Kind::For { body, .. }
                    | Kind::Do { body, .. } => walk(body, source, labels, out),
                    Kind::If {
                        then_body,
                        else_body,
                        ..
                    } => {
                        walk(then_body, source, labels, out);
                        walk(else_body, source, labels, out);
                    }
                    Kind::SelectCase {
                        cases, else_body, ..
                    } => {
                        for case in cases {
                            walk(&case.body, source, labels, out);
                        }
                        walk(else_body, source, labels, out);
                    }
                    _ => {}
                }
            }
        }
        let Some(source) = self.sources.first() else {
            return Vec::new();
        };
        let mut result = Vec::new();
        let mut labels = BTreeSet::new();
        walk(&self.statements, source, &mut labels, &mut result);
        for callable in &self.callables {
            labels.clear();
            walk(&callable.body, source, &mut labels, &mut result);
        }
        result
    }

    /// Return every immutable binding represented by the semantic module.
    /// Names retain their source type suffix; callers can therefore construct
    /// backend storage without consulting parser-era declaration nodes.
    pub fn const_names(&self) -> BTreeSet<String> {
        fn visit(statements: &[SemanticStatement], names: &mut BTreeSet<String>) {
            for statement in statements {
                match &statement.kind {
                    SemanticStatementKind::Const { name, .. } => {
                        names.insert(name.name.clone());
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

        let mut names = BTreeSet::new();
        visit(&self.statements, &mut names);
        for callable in &self.callables {
            visit(&callable.body, &mut names);
        }
        names
    }

    /// Return every immutable binding with its resolved value type.  CONST
    /// declarations normally omit a source suffix, so the type is taken from
    /// the typed initializer expression; an unresolved initializer retains
    /// BASCAL's integer default.
    pub fn const_types(&self) -> BTreeMap<String, SemanticValueType> {
        let mut types = BTreeMap::new();
        loop {
            let previous_count = types.len();
            collect_const_types(&self.statements, &mut types);
            for callable in &self.callables {
                collect_const_types(&callable.body, &mut types);
            }
            if types.len() == previous_count {
                break;
            }
        }
        for name in self.const_names() {
            types.entry(name).or_insert(SemanticValueType::Integer);
        }
        types
    }

    /// Return immutable bindings declared at module scope, including nested
    /// control-flow blocks but excluding callable-local constants.
    pub fn top_level_const_types(&self) -> BTreeMap<String, SemanticValueType> {
        let top_level = self.top_level_const_names();
        let mut types = self.const_types();
        types.retain(|name, _| top_level.contains(name));
        types
    }

    /// Return named `ON ERROR GOTO` targets in source order. Numeric disable
    /// sentinels are intentionally omitted.
    pub fn error_handler_targets(&self) -> Vec<String> {
        fn visit(statements: &[SemanticStatement], targets: &mut Vec<String>) {
            for statement in statements {
                match &statement.kind {
                    SemanticStatementKind::OnErrorGoto(ErrorHandlerTarget::Label(target)) => {
                        if !targets
                            .iter()
                            .any(|name| name.eq_ignore_ascii_case(&target.name))
                        {
                            targets.push(target.name.clone());
                        }
                    }
                    SemanticStatementKind::Line(body)
                    | SemanticStatementKind::While { body, .. }
                    | SemanticStatementKind::For { body, .. }
                    | SemanticStatementKind::Do { body, .. } => visit(body, targets),
                    SemanticStatementKind::If {
                        then_body,
                        else_body,
                        ..
                    } => {
                        visit(then_body, targets);
                        visit(else_body, targets);
                    }
                    SemanticStatementKind::SelectCase {
                        cases, else_body, ..
                    } => {
                        for case in cases {
                            visit(&case.body, targets);
                        }
                        visit(else_body, targets);
                    }
                    SemanticStatementKind::Try {
                        body,
                        catch,
                        finally_body,
                    } => {
                        visit(body, targets);
                        if let Some(catch) = catch {
                            visit(&catch.body, targets);
                        }
                        visit(finally_body, targets);
                    }
                    _ => {}
                }
            }
        }

        let mut targets = Vec::new();
        visit(&self.statements, &mut targets);
        for callable in &self.callables {
            visit(&callable.body, &mut targets);
        }
        targets
    }

    /// Return named error-handler targets in module scope only.
    pub fn top_level_error_handler_targets(&self) -> Vec<String> {
        let mut targets = Vec::new();
        fn visit(statements: &[SemanticStatement], targets: &mut Vec<String>) {
            for statement in statements {
                match &statement.kind {
                    SemanticStatementKind::OnErrorGoto(ErrorHandlerTarget::Label(target)) => {
                        if !targets
                            .iter()
                            .any(|name| name.eq_ignore_ascii_case(&target.name))
                        {
                            targets.push(target.name.clone());
                        }
                    }
                    SemanticStatementKind::Line(body)
                    | SemanticStatementKind::While { body, .. }
                    | SemanticStatementKind::For { body, .. }
                    | SemanticStatementKind::Do { body, .. } => visit(body, targets),
                    SemanticStatementKind::If {
                        then_body,
                        else_body,
                        ..
                    } => {
                        visit(then_body, targets);
                        visit(else_body, targets);
                    }
                    SemanticStatementKind::SelectCase {
                        cases, else_body, ..
                    } => {
                        for case in cases {
                            visit(&case.body, targets);
                        }
                        visit(else_body, targets);
                    }
                    SemanticStatementKind::Try {
                        body,
                        catch,
                        finally_body,
                    } => {
                        visit(body, targets);
                        if let Some(catch) = catch {
                            visit(&catch.body, targets);
                        }
                        visit(finally_body, targets);
                    }
                    _ => {}
                }
            }
        }
        visit(&self.statements, &mut targets);
        targets
    }

    /// Count top-level raise sites used by the C runtime dispatch table.
    pub fn top_level_raise_site_count(&self) -> usize {
        fn count(statements: &[SemanticStatement]) -> usize {
            statements
                .iter()
                .map(|statement| {
                    let self_count = usize::from(matches!(
                        &statement.kind,
                        SemanticStatementKind::Throw(_)
                            | SemanticStatementKind::Open {
                                mode: OpenMode {
                                    kind: OpenModeKind::Input
                                        | OpenModeKind::Random
                                        | OpenModeKind::Binary,
                                    ..
                                },
                                ..
                            }
                    ));
                    let nested = match &statement.kind {
                        SemanticStatementKind::Line(body)
                        | SemanticStatementKind::While { body, .. }
                        | SemanticStatementKind::For { body, .. }
                        | SemanticStatementKind::Do { body, .. } => count(body),
                        SemanticStatementKind::If {
                            then_body,
                            else_body,
                            ..
                        } => count(then_body) + count(else_body),
                        SemanticStatementKind::SelectCase {
                            cases, else_body, ..
                        } => {
                            cases.iter().map(|case| count(&case.body)).sum::<usize>()
                                + count(else_body)
                        }
                        SemanticStatementKind::Try {
                            body,
                            catch,
                            finally_body,
                        } => {
                            count(body)
                                + catch.as_ref().map_or(0, |catch| count(&catch.body))
                                + count(finally_body)
                        }
                        _ => 0,
                    };
                    self_count + nested
                })
                .sum()
        }
        count(&self.statements)
            + self
                .lowered_record_files
                .iter()
                .filter(|file| file.owner.is_none())
                .count()
    }

    /// Count top-level TRY/CATCH blocks in source order.
    pub fn top_level_try_catch_count(&self) -> usize {
        fn count(statements: &[SemanticStatement]) -> usize {
            statements
                .iter()
                .map(|statement| {
                    let self_count =
                        usize::from(matches!(&statement.kind, SemanticStatementKind::Try { .. }));
                    let nested = match &statement.kind {
                        SemanticStatementKind::Line(body)
                        | SemanticStatementKind::While { body, .. }
                        | SemanticStatementKind::For { body, .. }
                        | SemanticStatementKind::Do { body, .. } => count(body),
                        SemanticStatementKind::If {
                            then_body,
                            else_body,
                            ..
                        } => count(then_body) + count(else_body),
                        SemanticStatementKind::SelectCase {
                            cases, else_body, ..
                        } => {
                            cases.iter().map(|case| count(&case.body)).sum::<usize>()
                                + count(else_body)
                        }
                        SemanticStatementKind::Try {
                            body,
                            catch,
                            finally_body,
                        } => {
                            count(body)
                                + catch.as_ref().map_or(0, |catch| count(&catch.body))
                                + count(finally_body)
                        }
                        _ => 0,
                    };
                    self_count + nested
                })
                .sum()
        }
        count(&self.statements)
    }

    /// Count top-level GOSUB statements in source traversal order.
    pub fn top_level_gosub_count(&self) -> usize {
        fn count(statements: &[SemanticStatement]) -> usize {
            statements
                .iter()
                .map(|statement| {
                    let self_count = match &statement.kind {
                        SemanticStatementKind::Gosub(_) => 1,
                        SemanticStatementKind::OnBranch {
                            branch: BranchKind::Gosub,
                            targets,
                            ..
                        } => targets.len(),
                        _ => 0,
                    };
                    let nested = match &statement.kind {
                        SemanticStatementKind::Line(body)
                        | SemanticStatementKind::While { body, .. }
                        | SemanticStatementKind::For { body, .. }
                        | SemanticStatementKind::Do { body, .. } => count(body),
                        SemanticStatementKind::If {
                            then_body,
                            else_body,
                            ..
                        } => count(then_body) + count(else_body),
                        SemanticStatementKind::SelectCase {
                            cases, else_body, ..
                        } => {
                            cases.iter().map(|case| count(&case.body)).sum::<usize>()
                                + count(else_body)
                        }
                        SemanticStatementKind::Try {
                            body,
                            catch,
                            finally_body,
                        } => {
                            count(body)
                                + catch.as_ref().map_or(0, |catch| count(&catch.body))
                                + count(finally_body)
                        }
                        _ => 0,
                    };
                    self_count + nested
                })
                .sum()
        }
        count(&self.statements)
    }

    /// Return immutable bindings declared in module scope, excluding
    /// callable-local constants.
    pub fn top_level_const_names(&self) -> BTreeSet<String> {
        fn visit(statements: &[SemanticStatement], names: &mut BTreeSet<String>) {
            for statement in statements {
                match &statement.kind {
                    SemanticStatementKind::Const { name, .. } => {
                        names.insert(name.name.clone());
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

        let mut names = BTreeSet::new();
        visit(&self.statements, &mut names);
        names
    }

    /// Return module-scope CONST initializer expressions, including CONSTs
    /// nested in top-level control-flow bodies. Callable-local bindings are
    /// excluded so a backend can emit module initialization from typed IR.
    pub fn top_level_const_initializers(&self) -> HashMap<String, Expression> {
        fn visit(statements: &[SemanticStatement], values: &mut HashMap<String, Expression>) {
            for statement in statements {
                match &statement.kind {
                    SemanticStatementKind::Const { name, value, .. } => {
                        values.insert(name.name.to_ascii_lowercase(), value.clone());
                    }
                    SemanticStatementKind::Line(body)
                    | SemanticStatementKind::While { body, .. }
                    | SemanticStatementKind::For { body, .. }
                    | SemanticStatementKind::Do { body, .. } => visit(body, values),
                    SemanticStatementKind::If {
                        then_body,
                        else_body,
                        ..
                    } => {
                        visit(then_body, values);
                        visit(else_body, values);
                    }
                    SemanticStatementKind::SelectCase {
                        cases, else_body, ..
                    } => {
                        for case in cases {
                            visit(&case.body, values);
                        }
                        visit(else_body, values);
                    }
                    SemanticStatementKind::Try {
                        body,
                        catch,
                        finally_body,
                    } => {
                        visit(body, values);
                        if let Some(catch) = catch {
                            visit(&catch.body, values);
                        }
                        visit(finally_body, values);
                    }
                    _ => {}
                }
            }
        }
        let mut values = HashMap::new();
        visit(&self.statements, &mut values);
        values
    }

    /// Return module-scope DIM declarations, including declarations nested
    /// in top-level control-flow bodies. Callable-local declarations are
    /// analyzed from their callable signatures and are excluded here.
    pub fn top_level_dim_declarations(&self) -> HashMap<String, DimDeclaration> {
        fn visit(
            statements: &[SemanticStatement],
            declarations: &mut HashMap<String, DimDeclaration>,
        ) {
            for statement in statements {
                match &statement.kind {
                    SemanticStatementKind::Dim(items) => {
                        for item in items {
                            declarations.insert(item.name.to_ascii_lowercase(), item.clone());
                        }
                    }
                    SemanticStatementKind::Line(body)
                    | SemanticStatementKind::While { body, .. }
                    | SemanticStatementKind::For { body, .. }
                    | SemanticStatementKind::Do { body, .. } => visit(body, declarations),
                    SemanticStatementKind::If {
                        then_body,
                        else_body,
                        ..
                    } => {
                        visit(then_body, declarations);
                        visit(else_body, declarations);
                    }
                    SemanticStatementKind::SelectCase {
                        cases, else_body, ..
                    } => {
                        for case in cases {
                            visit(&case.body, declarations);
                        }
                        visit(else_body, declarations);
                    }
                    SemanticStatementKind::Try {
                        body,
                        catch,
                        finally_body,
                    } => {
                        visit(body, declarations);
                        if let Some(catch) = catch {
                            visit(&catch.body, declarations);
                        }
                        visit(finally_body, declarations);
                    }
                    _ => {}
                }
            }
        }
        let mut declarations = HashMap::new();
        visit(&self.statements, &mut declarations);
        declarations
    }

    /// Resolve implicit DIM types before codegen. BASCAL's unsuffixed numeric
    /// default is single precision; keeping that fact in typed IR prevents
    /// each backend from independently defaulting `Unknown`.
    pub fn resolve_dim_value_types(&mut self) {
        fn visit(statements: &mut [SemanticStatement]) {
            for statement in statements {
                match &mut statement.kind {
                    SemanticStatementKind::Dim(items) => {
                        for item in items {
                            if item.element_type == SemanticValueType::Unknown
                                && item.type_annotation.is_none()
                            {
                                item.element_type = SemanticValueType::Single;
                            }
                        }
                    }
                    SemanticStatementKind::Line(body)
                    | SemanticStatementKind::While { body, .. }
                    | SemanticStatementKind::For { body, .. }
                    | SemanticStatementKind::Do { body, .. } => visit(body),
                    SemanticStatementKind::If {
                        then_body,
                        else_body,
                        ..
                    } => {
                        visit(then_body);
                        visit(else_body);
                    }
                    SemanticStatementKind::SelectCase {
                        cases, else_body, ..
                    } => {
                        for case in cases {
                            visit(&mut case.body);
                        }
                        visit(else_body);
                    }
                    SemanticStatementKind::Try {
                        body,
                        catch,
                        finally_body,
                    } => {
                        visit(body);
                        if let Some(catch) = catch {
                            visit(&mut catch.body);
                        }
                        visit(finally_body);
                    }
                    _ => {}
                }
            }
        }

        visit(&mut self.statements);
        for callable in &mut self.callables {
            visit(&mut callable.body);
        }
    }

    /// Resolve unsuffixed FOR variables against their enclosing module or
    /// callable DIM declarations so codegen backends consume the declared
    /// type directly from typed IR.
    pub fn resolve_for_variable_types(&mut self) {
        fn collect_types(
            statements: &[SemanticStatement],
            types: &mut HashMap<String, SemanticValueType>,
        ) {
            for statement in statements {
                match &statement.kind {
                    SemanticStatementKind::Dim(items) => {
                        for item in items {
                            types.insert(item.name.to_ascii_lowercase(), item.element_type);
                        }
                    }
                    SemanticStatementKind::Line(body)
                    | SemanticStatementKind::While { body, .. }
                    | SemanticStatementKind::For { body, .. }
                    | SemanticStatementKind::Do { body, .. } => collect_types(body, types),
                    SemanticStatementKind::If {
                        then_body,
                        else_body,
                        ..
                    } => {
                        collect_types(then_body, types);
                        collect_types(else_body, types);
                    }
                    SemanticStatementKind::SelectCase {
                        cases, else_body, ..
                    } => {
                        for case in cases {
                            collect_types(&case.body, types);
                        }
                        collect_types(else_body, types);
                    }
                    SemanticStatementKind::Try {
                        body,
                        catch,
                        finally_body,
                    } => {
                        collect_types(body, types);
                        if let Some(catch) = catch {
                            collect_types(&catch.body, types);
                        }
                        collect_types(finally_body, types);
                    }
                    _ => {}
                }
            }
        }

        fn annotate(
            statements: &mut [SemanticStatement],
            types: &HashMap<String, SemanticValueType>,
        ) {
            for statement in statements {
                match &mut statement.kind {
                    SemanticStatementKind::For {
                        variable,
                        variable_type,
                        body,
                        ..
                    } => {
                        if *variable_type == SemanticValueType::Unknown {
                            if let Some(value_type) = types.get(&variable.to_ascii_lowercase()) {
                                *variable_type = *value_type;
                            }
                            if *variable_type == SemanticValueType::Unknown {
                                *variable_type = SemanticValueType::Single;
                            }
                        }
                        annotate(body, types);
                    }
                    SemanticStatementKind::Line(body)
                    | SemanticStatementKind::While { body, .. }
                    | SemanticStatementKind::Do { body, .. } => annotate(body, types),
                    SemanticStatementKind::If {
                        then_body,
                        else_body,
                        ..
                    } => {
                        annotate(then_body, types);
                        annotate(else_body, types);
                    }
                    SemanticStatementKind::SelectCase {
                        cases, else_body, ..
                    } => {
                        for case in cases {
                            annotate(&mut case.body, types);
                        }
                        annotate(else_body, types);
                    }
                    SemanticStatementKind::Try {
                        body,
                        catch,
                        finally_body,
                    } => {
                        annotate(body, types);
                        if let Some(catch) = catch {
                            annotate(&mut catch.body, types);
                        }
                        annotate(finally_body, types);
                    }
                    _ => {}
                }
            }
        }

        let top_level_types = self
            .top_level_dim_declarations()
            .into_iter()
            .map(|(name, declaration)| (name, declaration.element_type))
            .collect::<HashMap<_, _>>();
        annotate(&mut self.statements, &top_level_types);
        for callable in &mut self.callables {
            let mut callable_types = top_level_types.clone();
            for parameter in &callable.parameters {
                callable_types.insert(
                    parameter.name.to_ascii_lowercase(),
                    parameter.value_type,
                );
            }
            collect_types(&callable.body, &mut callable_types);
            annotate(&mut callable.body, &callable_types);
        }
    }

    /// Return module-scope array names and ranks, including declarations
    /// nested in top-level control-flow bodies. Callable-local arrays are
    /// excluded because backends analyze each callable scope separately.
    pub fn top_level_array_ranks(&self) -> HashMap<String, usize> {
        fn visit(statements: &[SemanticStatement], ranks: &mut HashMap<String, usize>) {
            for statement in statements {
                match &statement.kind {
                    SemanticStatementKind::Dim(declarations) => {
                        for declaration in declarations
                            .iter()
                            .filter(|declaration| declaration.array_axes > 0)
                        {
                            ranks.insert(
                                declaration.name.to_ascii_lowercase(),
                                declaration.array_axes,
                            );
                        }
                    }
                    SemanticStatementKind::Line(body)
                    | SemanticStatementKind::While { body, .. }
                    | SemanticStatementKind::For { body, .. }
                    | SemanticStatementKind::Do { body, .. } => visit(body, ranks),
                    SemanticStatementKind::If {
                        then_body,
                        else_body,
                        ..
                    } => {
                        visit(then_body, ranks);
                        visit(else_body, ranks);
                    }
                    SemanticStatementKind::SelectCase {
                        cases, else_body, ..
                    } => {
                        for case in cases {
                            visit(&case.body, ranks);
                        }
                        visit(else_body, ranks);
                    }
                    SemanticStatementKind::Try {
                        body,
                        catch,
                        finally_body,
                    } => {
                        visit(body, ranks);
                        if let Some(catch) = catch {
                            visit(&catch.body, ranks);
                        }
                        visit(finally_body, ranks);
                    }
                    _ => {}
                }
            }
        }

        let mut ranks = HashMap::new();
        visit(&self.statements, &mut ranks);
        ranks
    }

    /// Return scalar DIM type annotations in module scope, including
    /// declarations nested in top-level control-flow bodies.
    pub fn top_level_dim_types(&self) -> HashMap<String, String> {
        fn visit(statements: &[SemanticStatement], types: &mut HashMap<String, String>) {
            for statement in statements {
                match &statement.kind {
                    SemanticStatementKind::Dim(items) => {
                        for item in items {
                            if let Some(annotation) = &item.type_annotation {
                                types.insert(item.name.to_ascii_lowercase(), annotation.clone());
                            }
                        }
                    }
                    SemanticStatementKind::Line(body)
                    | SemanticStatementKind::While { body, .. }
                    | SemanticStatementKind::For { body, .. }
                    | SemanticStatementKind::Do { body, .. } => visit(body, types),
                    SemanticStatementKind::If {
                        then_body,
                        else_body,
                        ..
                    } => {
                        visit(then_body, types);
                        visit(else_body, types);
                    }
                    SemanticStatementKind::SelectCase {
                        cases, else_body, ..
                    } => {
                        for case in cases {
                            visit(&case.body, types);
                        }
                        visit(else_body, types);
                    }
                    SemanticStatementKind::Try {
                        body,
                        catch,
                        finally_body,
                    } => {
                        visit(body, types);
                        if let Some(catch) = catch {
                            visit(&catch.body, types);
                        }
                        visit(finally_body, types);
                    }
                    _ => {}
                }
            }
        }

        let mut types = HashMap::new();
        visit(&self.statements, &mut types);
        types
    }

    /// Return every BASIC FIELD buffer name in module and callable scopes.
    pub fn record_buffer_names(&self) -> HashSet<String> {
        fn visit(statements: &[SemanticStatement], names: &mut HashSet<String>) {
            for statement in statements {
                match &statement.kind {
                    SemanticStatementKind::Field { bindings, .. } => {
                        names.extend(
                            bindings
                                .iter()
                                .map(|binding| binding.name.to_ascii_lowercase()),
                        );
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
        visit(&self.statements, &mut names);
        for callable in &self.callables {
            visit(&callable.body, &mut names);
        }
        names
    }

    /// Return the DATA item count and label offsets in module and callable order.
    pub fn data_label_offsets(&self) -> (usize, HashMap<String, usize>) {
        fn visit(
            statements: &[SemanticStatement],
            items: &mut usize,
            labels: &mut HashMap<String, usize>,
        ) {
            for statement in statements {
                match &statement.kind {
                    SemanticStatementKind::Data(values) => *items += values.len(),
                    SemanticStatementKind::Label(name) => {
                        labels.insert(name.name.to_ascii_lowercase(), *items);
                    }
                    SemanticStatementKind::Line(body)
                    | SemanticStatementKind::While { body, .. }
                    | SemanticStatementKind::For { body, .. }
                    | SemanticStatementKind::Do { body, .. } => visit(body, items, labels),
                    SemanticStatementKind::If {
                        then_body,
                        else_body,
                        ..
                    } => {
                        visit(then_body, items, labels);
                        visit(else_body, items, labels);
                    }
                    SemanticStatementKind::SelectCase {
                        cases, else_body, ..
                    } => {
                        for case in cases {
                            visit(&case.body, items, labels);
                        }
                        visit(else_body, items, labels);
                    }
                    SemanticStatementKind::Try {
                        body,
                        catch,
                        finally_body,
                    } => {
                        visit(body, items, labels);
                        if let Some(catch) = catch {
                            visit(&catch.body, items, labels);
                        }
                        visit(finally_body, items, labels);
                    }
                    _ => {}
                }
            }
        }

        let mut items = 0;
        let mut labels = HashMap::new();
        visit(&self.statements, &mut items, &mut labels);
        for callable in &self.callables {
            visit(&callable.body, &mut items, &mut labels);
        }
        (items, labels)
    }

    /// Whether any CATCH binding captures a source filename.
    pub fn uses_catch_source_var(&self) -> bool {
        fn visit(statements: &[SemanticStatement]) -> bool {
            statements.iter().any(|statement| match &statement.kind {
                SemanticStatementKind::Try {
                    body,
                    catch,
                    finally_body,
                } => {
                    catch
                        .as_ref()
                        .is_some_and(|binding| binding.source.is_some())
                        || visit(body)
                        || catch.as_ref().is_some_and(|binding| visit(&binding.body))
                        || visit(finally_body)
                }
                SemanticStatementKind::Line(body)
                | SemanticStatementKind::While { body, .. }
                | SemanticStatementKind::For { body, .. }
                | SemanticStatementKind::Do { body, .. } => visit(body),
                SemanticStatementKind::If {
                    then_body,
                    else_body,
                    ..
                } => visit(then_body) || visit(else_body),
                SemanticStatementKind::SelectCase {
                    cases, else_body, ..
                } => cases.iter().any(|case| visit(&case.body)) || visit(else_body),
                _ => false,
            })
        }

        visit(&self.statements) || self.callables.iter().any(|callable| visit(&callable.body))
    }

    /// Whether a statement matching `predicate` occurs anywhere in the module,
    /// including callable bodies and nested control-flow blocks.
    pub fn has_statement(&self, predicate: impl Fn(&SemanticStatementKind) -> bool + Copy) -> bool {
        semantic_statements_have_statement(&self.statements, predicate)
            || self
                .callables
                .iter()
                .any(|callable| semantic_statements_have_statement(&callable.body, predicate))
    }

    /// Whether a statement matching `predicate` occurs in any callable body,
    /// including nested control-flow blocks.
    pub fn has_callable_statement(
        &self,
        predicate: impl Fn(&SemanticStatementKind) -> bool + Copy,
    ) -> bool {
        self.callables
            .iter()
            .any(|callable| semantic_statements_have_statement(&callable.body, predicate))
    }

    /// Evaluate integer-valued module-scope constants for compile-time
    /// consumers such as fixed array bounds. Runtime constant bindings remain
    /// variables; this fact table is only an optimization/validation aid.
    pub fn top_level_integer_constants(&self) -> HashMap<String, i64> {
        fn eval(
            expression: &Expression,
            definitions: &HashMap<String, Expression>,
            depth: u8,
        ) -> Option<i64> {
            if depth > 32 {
                return None;
            }
            match &expression.kind {
                ExpressionKind::Literal(value) => parse_integer_value(value),
                ExpressionKind::Boolean(value) => Some(if *value { -1 } else { 0 }),
                ExpressionKind::Unary { operator, operand } if operator == "-" => {
                    eval(operand, definitions, depth + 1).and_then(i64::checked_neg)
                }
                ExpressionKind::Parenthesized(inner) => eval(inner, definitions, depth + 1),
                ExpressionKind::Binary {
                    left,
                    operator,
                    right,
                } => {
                    let left = eval(left, definitions, depth + 1)?;
                    let right = eval(right, definitions, depth + 1)?;
                    match operator.as_str() {
                        "+" => left.checked_add(right),
                        "-" => left.checked_sub(right),
                        "*" => left.checked_mul(right),
                        "/" if right != 0 => left.checked_div(right),
                        "\\" if right != 0 => left.checked_div(right),
                        "MOD" | "mod" if right != 0 => left.checked_rem(right),
                        _ => None,
                    }
                }
                ExpressionKind::Name(name) => {
                    let key = name.to_ascii_lowercase();
                    definitions
                        .get(&key)
                        .and_then(|value| eval(value, definitions, depth + 1))
                }
                _ => None,
            }
        }

        fn collect(
            statements: &[SemanticStatement],
            definitions: &mut HashMap<String, Expression>,
        ) {
            for statement in statements {
                match &statement.kind {
                    SemanticStatementKind::Const { name, value, .. } => {
                        definitions.insert(name.name.to_ascii_lowercase(), value.clone());
                        let bare = name
                            .name
                            .trim_end_matches(['$', '%', '&', '!', '#'])
                            .to_ascii_lowercase();
                        definitions.entry(bare).or_insert_with(|| value.clone());
                    }
                    SemanticStatementKind::Line(body)
                    | SemanticStatementKind::While { body, .. }
                    | SemanticStatementKind::For { body, .. }
                    | SemanticStatementKind::Do { body, .. } => collect(body, definitions),
                    SemanticStatementKind::If {
                        then_body,
                        else_body,
                        ..
                    } => {
                        collect(then_body, definitions);
                        collect(else_body, definitions);
                    }
                    SemanticStatementKind::SelectCase {
                        cases, else_body, ..
                    } => {
                        for case in cases {
                            collect(&case.body, definitions);
                        }
                        collect(else_body, definitions);
                    }
                    SemanticStatementKind::Try {
                        body,
                        catch,
                        finally_body,
                    } => {
                        collect(body, definitions);
                        if let Some(catch) = catch {
                            collect(&catch.body, definitions);
                        }
                        collect(finally_body, definitions);
                    }
                    _ => {}
                }
            }
        }

        let mut definitions = HashMap::new();
        collect(&self.statements, &mut definitions);
        let mut values = HashMap::new();
        for name in definitions.keys() {
            if let Some(value) = eval(
                definitions.get(name).expect("definition key exists"),
                &definitions,
                0,
            ) {
                values.insert(name.clone(), value);
            }
        }
        values
    }

    /// Evaluate a compile-time integer expression using module-level CONST
    /// bindings. Backends use this fact for fixed array capacities.
    pub fn evaluate_integer_expression(&self, expression: &Expression) -> Option<i64> {
        fn evaluate(
            expression: &Expression,
            constants: &HashMap<String, i64>,
            depth: u8,
        ) -> Option<i64> {
            if depth > 32 {
                return None;
            }
            match &expression.kind {
                ExpressionKind::Literal(value) => parse_integer_value(value),
                ExpressionKind::Boolean(value) => Some(if *value { -1 } else { 0 }),
                ExpressionKind::Name(name) => {
                    let key = name.to_ascii_lowercase();
                    constants.get(&key).copied().or_else(|| {
                        constants
                            .get(key.trim_end_matches(['$', '%', '&', '!', '#']))
                            .copied()
                    })
                }
                ExpressionKind::Unary { operator, operand } if operator == "-" => {
                    evaluate(operand, constants, depth + 1)?.checked_neg()
                }
                ExpressionKind::Binary {
                    left,
                    operator,
                    right,
                } => {
                    let left = evaluate(left, constants, depth + 1)?;
                    let right = evaluate(right, constants, depth + 1)?;
                    match operator.as_str() {
                        "+" => left.checked_add(right),
                        "-" => left.checked_sub(right),
                        "*" => left.checked_mul(right),
                        "/" if right != 0 => left.checked_div(right),
                        "\\" if right != 0 => left.checked_div(right),
                        "MOD" | "mod" if right != 0 => left.checked_rem(right),
                        _ => None,
                    }
                }
                ExpressionKind::Parenthesized(inner) => evaluate(inner, constants, depth + 1),
                _ => None,
            }
        }
        evaluate(expression, &self.top_level_integer_constants(), 0)
    }

    /// Evaluate integer constants in module scope and in every callable body.
    /// Callable-local names are included for backend analyses that inspect a
    /// callable's fixed array capacities.
    pub fn integer_constants(&self) -> HashMap<String, i64> {
        let mut values = self.top_level_integer_constants();
        for callable in &self.callables {
            let mut body = self.clone();
            body.statements = callable.body.clone();
            values.extend(body.top_level_integer_constants());
        }
        values
    }

    pub fn name_scopes(&self) -> SemanticNameScopes {
        let mut scopes = SemanticNameScopes::default();
        collect_semantic_statements(&self.statements, &mut scopes.global_names);
        collect_semantic_global_declarations(
            &self.statements,
            &mut BTreeSet::new(),
            &mut scopes.global_types,
        );
        for callable in &self.callables {
            let key = callable.name.to_ascii_lowercase();
            let mut globals = BTreeSet::new();
            collect_semantic_global_declarations(
                &callable.body,
                &mut globals,
                &mut scopes.global_types,
            );
            scopes.global_names.extend(globals.iter().cloned());
            scopes.callable_globals.insert(key, globals);
        }
        scopes
    }

    /// Merge a dependency module ahead of this module's executable content.
    /// This mirrors the driver's legacy dependency order while preserving the
    /// root module header and source span.
    pub fn prepend_dependency(&mut self, dependency: SemanticModule) {
        let source_offset = dependency.sources.len();
        let mut sources = dependency.sources;
        sources.append(&mut self.sources);
        self.sources = sources;
        let mut dependencies = dependency.dependencies;
        dependencies.append(&mut self.dependencies);
        self.dependencies = dependencies;
        self.records.splice(0..0, dependency.records);
        let mut callables = dependency.callables;
        self.callables.iter_mut().for_each(|callable| {
            if callable.source_index != usize::MAX {
                callable.source_index += source_offset;
            }
        });
        callables.append(&mut self.callables);
        self.callables = callables;
        let mut statements = dependency.statements;
        statements.append(&mut self.statements);
        self.statements = statements;
        let mut statement_sources = dependency.statement_sources;
        statement_sources.extend(self.statement_sources.drain(..).map(|source| {
            if source == usize::MAX {
                source
            } else {
                source + source_offset
            }
        }));
        self.statement_sources = statement_sources;
    }

    /// Whether this module contains structured error handling.  This is a
    /// backend capability fact, so targets need not rediscover `try` nodes
    /// from parser-era statements.
    pub fn contains_try(&self) -> bool {
        fn statements_contain_try(statements: &[SemanticStatement]) -> bool {
            statements.iter().any(|statement| match &statement.kind {
                SemanticStatementKind::Try { .. } => true,
                SemanticStatementKind::If {
                    then_body,
                    else_body,
                    ..
                } => statements_contain_try(then_body) || statements_contain_try(else_body),
                SemanticStatementKind::While { body, .. }
                | SemanticStatementKind::Line(body)
                | SemanticStatementKind::For { body, .. }
                | SemanticStatementKind::Do { body, .. } => statements_contain_try(body),
                SemanticStatementKind::SelectCase {
                    cases, else_body, ..
                } => {
                    cases.iter().any(|case| statements_contain_try(&case.body))
                        || statements_contain_try(else_body)
                }
                _ => false,
            })
        }

        statements_contain_try(&self.statements)
            || self
                .callables
                .iter()
                .any(|callable| statements_contain_try(&callable.body))
    }

    pub fn callable_value_type(&self, name: &str) -> SemanticValueType {
        let Some(callable) = self
            .callables
            .iter()
            .find(|callable| callable.name.eq_ignore_ascii_case(name))
        else {
            return SemanticValueType::Unknown;
        };
        callable
            .result_type
            .as_deref()
            .and_then(|suffix| suffix.chars().next())
            .map_or(SemanticValueType::Unknown, |suffix| {
                SemanticValueType::from_suffix(Some(suffix))
            })
    }

    pub fn record_variable_types(&self) -> HashMap<String, String> {
        fn visit(
            statements: &[SemanticStatement],
            records: &[Record],
            types: &mut HashMap<String, String>,
        ) {
            for statement in statements {
                match &statement.kind {
                    SemanticStatementKind::Dim(items) => {
                        for item in items {
                            let key = item.name.to_ascii_lowercase();
                            if let Some(record) = item.type_annotation.as_deref().and_then(|name| {
                                records
                                    .iter()
                                    .find(|record| record.name.eq_ignore_ascii_case(name))
                            }) {
                                types.insert(key, record.name.clone());
                            } else {
                                types.remove(&key);
                            }
                        }
                    }
                    SemanticStatementKind::Assignment { target, value, .. } => {
                        if let ExpressionKind::Name(name) = &target.kind {
                            let key = name.to_ascii_lowercase();
                            let record_type =
                                infer_record_literal_type(value, records).or_else(|| {
                                    if let ExpressionKind::Name(source) = &value.kind {
                                        types.get(&source.to_ascii_lowercase()).cloned()
                                    } else {
                                        None
                                    }
                                });
                            if let Some(record_type) = record_type {
                                types.insert(key, record_type);
                            }
                        }
                    }
                    SemanticStatementKind::Line(body)
                    | SemanticStatementKind::While { body, .. }
                    | SemanticStatementKind::For { body, .. }
                    | SemanticStatementKind::Do { body, .. } => visit(body, records, types),
                    SemanticStatementKind::If {
                        then_body,
                        else_body,
                        ..
                    } => {
                        visit(then_body, records, types);
                        visit(else_body, records, types);
                    }
                    SemanticStatementKind::SelectCase {
                        cases, else_body, ..
                    } => {
                        for case in cases {
                            visit(&case.body, records, types);
                        }
                        visit(else_body, records, types);
                    }
                    SemanticStatementKind::Try {
                        body,
                        catch,
                        finally_body,
                    } => {
                        visit(body, records, types);
                        if let Some(catch) = catch {
                            visit(&catch.body, records, types);
                        }
                        visit(finally_body, records, types);
                    }
                    _ => {}
                }
            }
        }

        let mut types = HashMap::new();
        visit(&self.statements, &self.records, &mut types);
        types
    }

    pub fn annotate_expression_types(&self, expression: &mut Expression) {
        let mut declarations = self.top_level_dim_declarations();
        for (name, record_type) in self.record_variable_types() {
            declarations
                .entry(name.clone())
                .or_insert_with(|| DimDeclaration {
                    name,
                    array_axes: 0,
                    dimensions: Vec::new(),
                    element_type: semantic_type_from_annotation(&record_type),
                    type_annotation: Some(record_type),
                    span: SourceSpan { start: 0, end: 0 },
                });
        }
        let constants = self.const_types();
        let array_reference = match &expression.kind {
            ExpressionKind::Call { name, arguments }
                if declarations
                    .get(&name.to_ascii_lowercase())
                    .is_some_and(|declaration| declaration.array_axes == arguments.len())
                    && !arguments.is_empty() =>
            {
                Some((name.clone(), arguments.clone()))
            }
            _ => None,
        };
        if let Some((name, mut indices)) = array_reference {
            if indices.len() == 1 {
                expression.kind = ExpressionKind::Index {
                    name,
                    index: Box::new(indices.pop().expect("one array index")),
                };
            } else {
                expression.kind = ExpressionKind::MultiIndex { name, indices };
            }
        }
        let record_method_call = match &expression.kind {
            ExpressionKind::Call { name, arguments } => {
                name.rsplit_once('.').map(|(receiver, method)| {
                    (receiver.to_string(), method.to_string(), arguments.clone())
                })
            }
            _ => None,
        };
        if let Some((receiver_name, method, arguments)) = record_method_call {
            let mut base = Expression {
                kind: ExpressionKind::Name(receiver_name),
                span: expression.span,
                value_type: SemanticValueType::Unknown,
                record_type: None,
            };
            self.annotate_expression_types(&mut base);
            if base.record_type.as_deref().is_some_and(|record_type| {
                record_callable_result(&self.callables, &self.records, record_type, &method)
                    .is_some()
            }) {
                expression.kind = ExpressionKind::Member {
                    base: Some(Box::new(base)),
                    member: method,
                    arguments: Some(arguments),
                };
            }
        }
        // `base.left(3)` on a scalar receiver is the ordinary builtin call
        // `LEFT$(base, 3)`, as `records::Lowerer` decides for the AST; doing
        // it here gives every backend the resolved call.
        let builtin_call = match &mut expression.kind {
            ExpressionKind::Member {
                base: Some(base),
                member,
                arguments: Some(arguments),
            } => {
                self.annotate_expression_types(base);
                for argument in arguments.iter_mut() {
                    self.annotate_expression_types(argument);
                }
                scalar_builtin_receiver(base.value_type)
                    .and_then(|receiver| crate::scalar_builtins::find(receiver, member))
                    .filter(|builtin| {
                        (builtin.min_args..=builtin.max_args).contains(&arguments.len())
                    })
                    .map(|builtin| {
                        let name = format!(
                            "{}{}",
                            builtin.method,
                            builtin.call_suffix.map(|s| s.to_string()).unwrap_or_default()
                        );
                        let mut call_arguments = vec![(**base).clone()];
                        call_arguments.extend(arguments.iter().cloned());
                        (name, call_arguments)
                    })
            }
            _ => None,
        };
        if let Some((name, arguments)) = builtin_call {
            expression.kind = ExpressionKind::Call { name, arguments };
        }
        match &mut expression.kind {
            ExpressionKind::Call { name, arguments } => {
                for argument in arguments.iter_mut() {
                    self.annotate_expression_types(argument);
                }
                let callable_type = self.callable_value_type(name);
                if callable_type == SemanticValueType::Unknown {
                    let record_method =
                        name.rsplit_once('.').and_then(|(receiver_name, method)| {
                            let mut receiver = Expression {
                                kind: ExpressionKind::Name(receiver_name.to_string()),
                                span: expression.span,
                                value_type: SemanticValueType::Unknown,
                                record_type: None,
                            };
                            self.annotate_expression_types(&mut receiver);
                            receiver.record_type.as_deref().and_then(|record_type| {
                                record_callable_result(
                                    &self.callables,
                                    &self.records,
                                    record_type,
                                    method,
                                )
                            })
                        });
                    if let Some((value_type, record_type)) = record_method {
                        expression.value_type = value_type;
                        expression.record_type = record_type;
                    } else {
                        expression.value_type = builtin_call_value_type(name, arguments);
                    }
                } else {
                    expression.value_type = callable_type;
                }
            }
            ExpressionKind::Index { name, index } => {
                self.annotate_expression_types(index);
                expression.value_type = array_element_type(name, &declarations);
                expression.record_type = declared_record_type(name, &self.records, &declarations);
            }
            ExpressionKind::MultiIndex { name, indices } => {
                for index in indices.iter_mut() {
                    self.annotate_expression_types(index);
                }
                expression.value_type = array_element_type(name, &declarations);
                expression.record_type = declared_record_type(name, &self.records, &declarations);
            }
            ExpressionKind::Member {
                base,
                member,
                arguments,
            } => {
                if let Some(base) = base {
                    self.annotate_expression_types(base);
                }
                if let Some(arguments) = arguments {
                    for argument in arguments {
                        self.annotate_expression_types(argument);
                    }
                }
                let mut resolved_method = false;
                if let Some(receiver_type) = base
                    .as_ref()
                    .map(|value| value.value_type)
                    .filter(|value_type| *value_type != SemanticValueType::Unknown)
                {
                    let argument_count = arguments.as_ref().map_or(0, Vec::len);
                    let callable = self.callables.iter().find(|callable| {
                        semantic_callable_name_matches_member(&callable.name, member)
                            && callable.receiver.as_deref().is_some_and(|receiver| {
                                semantic_type_from_annotation(receiver) == receiver_type
                            })
                            && callable
                                .parameters
                                .iter()
                                .filter(|parameter| parameter.default.is_none())
                                .count()
                                <= argument_count
                            && argument_count <= callable.parameters.len()
                    });
                    if let Some(callable) = callable {
                        expression.value_type = callable
                            .result_type
                            .as_deref()
                            .and_then(|suffix| suffix.chars().next())
                            .map(|suffix| SemanticValueType::from_suffix(Some(suffix)))
                            .unwrap_or(receiver_type);
                        resolved_method = true;
                    }
                }
                if !resolved_method {
                    if let Some(base) = base.as_deref() {
                        if let Some(receiver_record) =
                            expression_record_type(base, &self.records, &declarations)
                        {
                            if let Some((value_type, record_type)) = record_callable_result(
                                &self.callables,
                                &self.records,
                                &receiver_record,
                                member,
                            ) {
                                expression.value_type = value_type;
                                expression.record_type = record_type;
                                resolved_method = true;
                            }
                        }
                    }
                }
                if !resolved_method && arguments.is_none() {
                    if let Some(base) = base.as_deref() {
                        expression.value_type =
                            record_field_value_type(base, member, &self.records, &declarations);
                        expression.record_type =
                            record_field_record_type(base, member, &self.records, &declarations);
                    }
                }
            }
            ExpressionKind::Parenthesized(inner) => {
                self.annotate_expression_types(inner);
                expression.value_type = inner.value_type;
                expression.record_type = inner.record_type.clone();
            }
            ExpressionKind::Unary { operand: inner, .. } => {
                self.annotate_expression_types(inner);
                expression.value_type = inner.value_type;
            }
            ExpressionKind::Binary {
                left,
                operator,
                right,
            } => {
                self.annotate_expression_types(left);
                self.annotate_expression_types(right);
                expression.value_type =
                    binary_expression_type(operator, left.value_type, right.value_type);
            }
            ExpressionKind::RecordLiteral(fields)
            | ExpressionKind::PartialRecordLiteral(fields) => {
                for field in fields {
                    self.annotate_expression_types(&mut field.value);
                }
            }
            ExpressionKind::Name(name) => {
                expression.record_type = name
                    .split_once('.')
                    .map(|(base_name, member_path)| {
                        let base = Expression {
                            kind: ExpressionKind::Name(base_name.to_string()),
                            span: expression.span,
                            value_type: SemanticValueType::Unknown,
                            record_type: declared_record_type(
                                base_name,
                                &self.records,
                                &declarations,
                            ),
                        };
                        record_field_record_type(&base, member_path, &self.records, &declarations)
                    })
                    .unwrap_or_else(|| declared_record_type(name, &self.records, &declarations));
                let record_field_type = name
                    .split_once('.')
                    .map(|(base_name, member_path)| {
                        let base = Expression {
                            kind: ExpressionKind::Name(base_name.to_string()),
                            span: expression.span,
                            value_type: SemanticValueType::Unknown,
                            record_type: declared_record_type(
                                base_name,
                                &self.records,
                                &declarations,
                            ),
                        };
                        record_field_value_type(&base, member_path, &self.records, &declarations)
                    })
                    .filter(|value_type| *value_type != SemanticValueType::Unknown);
                expression.value_type = record_field_type
                    .or_else(|| {
                        constants
                            .iter()
                            .find(|(constant, _)| constant.eq_ignore_ascii_case(name))
                            .map(|(_, value_type)| *value_type)
                            .or_else(|| {
                                declarations
                                    .get(&name.to_ascii_lowercase())
                                    .map(|declaration| declaration.element_type)
                                    .filter(|value_type| *value_type != SemanticValueType::Unknown)
                            })
                    })
                    .unwrap_or_else(|| SemanticValueType::from_suffix(name.chars().last()));
            }
            ExpressionKind::Literal(_) | ExpressionKind::Boolean(_) => {}
        }
    }

    pub fn annotate_statement_types(&self, statements: &mut [SemanticStatement]) {
        for statement in statements {
            match &mut statement.kind {
                SemanticStatementKind::Assignment { target, value, .. } => {
                    self.annotate_expression_types(target);
                    self.annotate_expression_types(value);
                    if let Some(record_type) = target.record_type.as_deref() {
                        annotate_record_literal_type(value, record_type, &self.records);
                    }
                }
                SemanticStatementKind::MidAssign {
                    target,
                    start,
                    length,
                    value,
                } => {
                    self.annotate_expression_types(target);
                    self.annotate_expression_types(start);
                    if let Some(length) = length {
                        self.annotate_expression_types(length);
                    }
                    self.annotate_expression_types(value);
                }
                SemanticStatementKind::Expression(expression)
                | SemanticStatementKind::Error(expression)
                | SemanticStatementKind::OptionBase(expression)
                | SemanticStatementKind::Kill(expression)
                | SemanticStatementKind::Close(expression) => {
                    self.annotate_expression_types(expression)
                }
                SemanticStatementKind::Return(ReturnValue::Value(expression))
                | SemanticStatementKind::Throw(ThrowValue::Value(expression)) => {
                    self.annotate_expression_types(expression)
                }
                SemanticStatementKind::If {
                    condition,
                    then_body,
                    else_body,
                    ..
                } => {
                    self.annotate_expression_types(condition);
                    self.annotate_statement_types(then_body);
                    self.annotate_statement_types(else_body);
                }
                SemanticStatementKind::While { condition, body } => {
                    self.annotate_expression_types(condition);
                    self.annotate_statement_types(body);
                }
                SemanticStatementKind::For {
                    start,
                    bounds,
                    body,
                    ..
                } => {
                    self.annotate_expression_types(start);
                    if let ForBounds::To { limit, step } = bounds {
                        self.annotate_expression_types(limit);
                        if let Some(step) = step {
                            self.annotate_expression_types(step);
                        }
                    }
                    self.annotate_statement_types(body);
                }
                SemanticStatementKind::Do {
                    pre_condition,
                    post_condition,
                    body,
                } => {
                    for condition in pre_condition.iter_mut().chain(post_condition.iter_mut()) {
                        self.annotate_expression_types(&mut condition.value);
                    }
                    self.annotate_statement_types(body);
                }
                SemanticStatementKind::Line(body) => self.annotate_statement_types(body),
                SemanticStatementKind::SelectCase {
                    selector,
                    cases,
                    else_body,
                } => {
                    self.annotate_expression_types(selector);
                    for case in cases {
                        for value in &mut case.values {
                            match value {
                                CaseValue::Comparison { value, .. } => {
                                    self.annotate_expression_types(value)
                                }
                                CaseValue::Value {
                                    first, range_end, ..
                                } => {
                                    self.annotate_expression_types(first);
                                    if let Some(end) = range_end {
                                        self.annotate_expression_types(end);
                                    }
                                }
                            }
                        }
                        self.annotate_statement_types(&mut case.body);
                    }
                    self.annotate_statement_types(else_body);
                }
                SemanticStatementKind::Try {
                    body,
                    catch,
                    finally_body,
                } => {
                    self.annotate_statement_types(body);
                    if let Some(catch) = catch {
                        for filter in &mut catch.filters {
                            self.annotate_expression_types(filter);
                        }
                        self.annotate_statement_types(&mut catch.body);
                    }
                    self.annotate_statement_types(finally_body);
                }
                SemanticStatementKind::Print {
                    destination,
                    tokens,
                } => {
                    match destination {
                        PrintDestination::Standard { .. } => {}
                        PrintDestination::Using { format, .. } => {
                            self.annotate_expression_types(format)
                        }
                        PrintDestination::Channel { channel, using, .. } => {
                            self.annotate_expression_types(channel);
                            if let Some(using) = using {
                                self.annotate_expression_types(using);
                            }
                        }
                    }
                    for token in tokens {
                        if let PrintToken::Expression(expression) = token {
                            self.annotate_expression_types(expression);
                        }
                    }
                }
                SemanticStatementKind::Lprint { using, tokens } => {
                    if let Some(using) = using {
                        self.annotate_expression_types(using);
                    }
                    for token in tokens {
                        if let PrintToken::Expression(expression) = token {
                            self.annotate_expression_types(expression);
                        }
                    }
                }
                SemanticStatementKind::Input { source, targets } => {
                    if let InputSource::Channel(channel) = source {
                        self.annotate_expression_types(channel);
                    }
                    for target in targets {
                        self.annotate_expression_types(target);
                    }
                }
                SemanticStatementKind::Read(targets) => {
                    for target in targets {
                        self.annotate_expression_types(target);
                    }
                }
                SemanticStatementKind::Open {
                    path,
                    channel,
                    length,
                    ..
                } => {
                    self.annotate_expression_types(path);
                    self.annotate_expression_types(channel);
                    if let Some(length) = length {
                        self.annotate_expression_types(length);
                    }
                }
                SemanticStatementKind::Write { channel, values } => {
                    self.annotate_expression_types(channel);
                    if let WriteValues::Values(values) = values {
                        for value in values {
                            self.annotate_expression_types(value);
                        }
                    }
                }
                SemanticStatementKind::OnBranch { selector, .. } => {
                    self.annotate_expression_types(selector)
                }
                SemanticStatementKind::Seek { channel, position } => {
                    self.annotate_expression_types(channel);
                    self.annotate_expression_types(position);
                }
                SemanticStatementKind::Rename {
                    source,
                    destination,
                } => {
                    self.annotate_expression_types(source);
                    self.annotate_expression_types(destination);
                }
                SemanticStatementKind::Data(values) => {
                    for value in values {
                        self.annotate_expression_types(value);
                    }
                }
                SemanticStatementKind::Dim(declarations) => {
                    for declaration in declarations {
                        for dimension in &mut declaration.dimensions {
                            if let DimAxis::Expression(expression) = dimension {
                                self.annotate_expression_types(expression);
                            }
                        }
                    }
                }
                SemanticStatementKind::Const { value, .. } => self.annotate_expression_types(value),
                SemanticStatementKind::Swap { left, right } => {
                    self.annotate_expression_types(left);
                    self.annotate_expression_types(right);
                }
                SemanticStatementKind::Randomize(RandomizeSeed::Value(seed)) => {
                    self.annotate_expression_types(seed)
                }
                SemanticStatementKind::Poke { address, value } => {
                    self.annotate_expression_types(address);
                    self.annotate_expression_types(value);
                }
                SemanticStatementKind::Out { port, value } => {
                    self.annotate_expression_types(port);
                    self.annotate_expression_types(value);
                }
                SemanticStatementKind::Width { channel, value } => {
                    if let Some(channel) = channel {
                        self.annotate_expression_types(channel);
                    }
                    self.annotate_expression_types(value);
                }
                SemanticStatementKind::LineInput { channel, target } => {
                    self.annotate_expression_types(channel);
                    self.annotate_expression_types(target);
                }
                SemanticStatementKind::Get { channel, position }
                | SemanticStatementKind::Put { channel, position } => {
                    self.annotate_expression_types(channel);
                    if let Some(position) = position {
                        if let Some(value) = &mut position.position {
                            self.annotate_expression_types(value);
                        }
                        if let Some(value) = &mut position.record {
                            self.annotate_expression_types(value);
                        }
                    }
                }
                SemanticStatementKind::Lset { value, .. }
                | SemanticStatementKind::Rset { value, .. } => {
                    self.annotate_expression_types(value)
                }
                SemanticStatementKind::Locate { row, column } => {
                    self.annotate_expression_types(row);
                    self.annotate_expression_types(column);
                }
                SemanticStatementKind::Color {
                    foreground,
                    background,
                } => {
                    self.annotate_expression_types(foreground);
                    if let Some(background) = background {
                        self.annotate_expression_types(background);
                    }
                }
                SemanticStatementKind::FileDeclaration { path, .. } => {
                    self.annotate_expression_types(path)
                }
                SemanticStatementKind::Field { channel, bindings } => {
                    self.annotate_expression_types(channel);
                    for binding in bindings {
                        self.annotate_expression_types(&mut binding.length);
                    }
                }
                _ => {}
            }
        }
    }
}

fn semantic_type_from_annotation(annotation: &str) -> SemanticValueType {
    match annotation.trim().to_ascii_lowercase().as_str() {
        "string" => SemanticValueType::String,
        "integer" | "int16" => SemanticValueType::Integer,
        "long" | "int32" => SemanticValueType::Long,
        "single" | "float32" => SemanticValueType::Single,
        "double" | "float64" => SemanticValueType::Double,
        _ => SemanticValueType::Unknown,
    }
}

fn semantic_callable_name_matches_member(callable: &str, member: &str) -> bool {
    let callable = callable.trim_end_matches(['$', '%', '!', '#', '&']);
    let member = member.trim_end_matches(['$', '%', '!', '#', '&']);
    callable.eq_ignore_ascii_case(member)
}

fn builtin_call_value_type(name: &str, arguments: &[Expression]) -> SemanticValueType {
    let has_string_suffix = name.ends_with('$');
    let name = name
        .trim_end_matches(['$', '%', '&', '!', '#'])
        .to_ascii_lowercase();
    match name.as_str() {
        "chr" | "str" | "mid" | "left" | "right" => SemanticValueType::String,
        "mki" | "mkl" | "mks" | "mkd" if has_string_suffix => SemanticValueType::String,
        "abs" | "int" | "fix" => arguments
            .first()
            .map(|argument| argument.value_type)
            .unwrap_or(SemanticValueType::Unknown),
        "sqr" | "sin" | "cos" | "tan" | "atn" | "log" | "exp" | "rnd" | "val" | "csng" | "cvs" => {
            SemanticValueType::Single
        }
        "cdbl" | "cvd" => SemanticValueType::Double,
        "clng" | "cvl" => SemanticValueType::Long,
        "sgn" | "cint" | "cvi" | "len" | "asc" | "instr" | "sizeof" | "lbound" | "ubound"
        | "eof" | "peek" | "inp" | "loc" | "lof" | "pos" => SemanticValueType::Integer,
        _ => SemanticValueType::Unknown,
    }
}

fn array_element_type(
    name: &str,
    declarations: &HashMap<String, DimDeclaration>,
) -> SemanticValueType {
    declarations
        .get(&name.to_ascii_lowercase())
        .map(|declaration| match declaration.element_type {
            SemanticValueType::Unknown => SemanticValueType::Single,
            value_type => value_type,
        })
        .unwrap_or(SemanticValueType::Unknown)
}

fn declared_record_type(
    name: &str,
    records: &[Record],
    declarations: &HashMap<String, DimDeclaration>,
) -> Option<String> {
    declarations
        .get(&name.to_ascii_lowercase())
        .and_then(|declaration| declaration.type_annotation.as_deref())
        .and_then(|annotation| {
            records
                .iter()
                .find(|record| record.name.eq_ignore_ascii_case(annotation))
                .map(|record| record.name.clone())
        })
}

fn infer_record_literal_type(expression: &Expression, records: &[Record]) -> Option<String> {
    fn field_names(
        records: &[Record],
        record_name: &str,
        visiting: &mut BTreeSet<String>,
        fields: &mut BTreeSet<String>,
    ) -> Option<()> {
        let key = record_name.to_ascii_lowercase();
        if !visiting.insert(key) {
            return None;
        }
        let record = records
            .iter()
            .find(|record| record.name.eq_ignore_ascii_case(record_name))?;
        fields.extend(
            record
                .fields
                .iter()
                .map(|field| field.name.to_ascii_lowercase()),
        );
        for combined in &record.combines {
            field_names(records, combined, visiting, fields)?;
        }
        visiting.remove(&record_name.to_ascii_lowercase());
        Some(())
    }

    let ExpressionKind::RecordLiteral(initializers) = &expression.kind else {
        return None;
    };
    let supplied: BTreeSet<_> = initializers
        .iter()
        .map(|initializer| initializer.name.to_ascii_lowercase())
        .collect();
    if supplied.len() != initializers.len() {
        return None;
    }
    let candidates: Vec<_> = records
        .iter()
        .filter_map(|record| {
            let mut fields = BTreeSet::new();
            field_names(records, &record.name, &mut BTreeSet::new(), &mut fields)?;
            (fields == supplied).then_some(record.name.clone())
        })
        .collect();
    match candidates.as_slice() {
        [record_type] => Some(record_type.clone()),
        _ => None,
    }
}

fn record_callable_result(
    callables: &[CallableSignature],
    records: &[Record],
    receiver_record: &str,
    method_name: &str,
) -> Option<(SemanticValueType, Option<String>)> {
    let callable = callables.iter().find(|callable| {
        callable.name.eq_ignore_ascii_case(method_name)
            && callable
                .receiver
                .as_deref()
                .is_some_and(|receiver| receiver.eq_ignore_ascii_case(receiver_record))
    })?;
    let result_type = callable
        .result_type
        .as_deref()
        .and_then(|result| result.chars().next())
        .map(|suffix| SemanticValueType::from_suffix(Some(suffix)))
        .unwrap_or(SemanticValueType::Unknown);
    let record_type = callable
        .result_type
        .as_deref()
        .and_then(|result| {
            records
                .iter()
                .find(|record| record.name.eq_ignore_ascii_case(result))
                .map(|record| record.name.clone())
        })
        .or_else(|| {
            callable
                .result_type
                .is_none()
                .then(|| receiver_record.to_string())
        });
    Some((result_type, record_type))
}

fn expression_record_type(
    expression: &Expression,
    records: &[Record],
    declarations: &HashMap<String, DimDeclaration>,
) -> Option<String> {
    if let Some(record_type) = &expression.record_type {
        return Some(record_type.clone());
    }
    match &expression.kind {
        ExpressionKind::Name(name) | ExpressionKind::Index { name, .. } => {
            let (base_name, member_path) = name
                .split_once('.')
                .map_or((name.as_str(), None), |(base, path)| (base, Some(path)));
            let owner = declared_record_type(base_name, records, declarations)?;
            if let Some(member_path) = member_path {
                match resolve_record_field_path(records, &owner, member_path)? {
                    RecordFieldType::Record { name, .. } => Some(name.clone()),
                    _ => None,
                }
            } else {
                Some(owner)
            }
        }
        ExpressionKind::Member {
            base: Some(base),
            member,
            arguments: None,
        } => {
            let owner = expression_record_type(base, records, declarations)?;
            match resolve_record_field_path(records, &owner, member)? {
                RecordFieldType::Record { name, .. } => Some(name.clone()),
                _ => None,
            }
        }
        _ => None,
    }
}

fn record_field_record_type(
    base: &Expression,
    member: &str,
    records: &[Record],
    declarations: &HashMap<String, DimDeclaration>,
) -> Option<String> {
    let owner = expression_record_type(base, records, declarations)?;
    match resolve_record_field_path(records, &owner, member)? {
        RecordFieldType::Record { name, .. } => records
            .iter()
            .find(|record| record.name.eq_ignore_ascii_case(name))
            .map(|record| record.name.clone()),
        _ => None,
    }
}

fn annotate_record_literal_type(
    expression: &mut Expression,
    record_type: &str,
    records: &[Record],
) {
    let fields = match &mut expression.kind {
        ExpressionKind::RecordLiteral(fields) | ExpressionKind::PartialRecordLiteral(fields) => {
            fields
        }
        _ => return,
    };
    let Some(record) = records
        .iter()
        .find(|record| record.name.eq_ignore_ascii_case(record_type))
    else {
        return;
    };
    expression.record_type = Some(record.name.clone());
    for initializer in fields {
        if let Some(RecordFieldType::Record { name, .. }) =
            resolve_record_field_path(records, &record.name, &initializer.name)
        {
            if records
                .iter()
                .any(|record| record.name.eq_ignore_ascii_case(name))
            {
                initializer.value.record_type = Some(name.clone());
                annotate_record_literal_type(&mut initializer.value, name, records);
            }
        }
    }
}

fn record_field_value_type(
    base: &Expression,
    member: &str,
    records: &[Record],
    declarations: &HashMap<String, DimDeclaration>,
) -> SemanticValueType {
    let Some(owner) = expression_record_type(base, records, declarations) else {
        return SemanticValueType::Unknown;
    };
    let Some(field_type) = resolve_record_field_path(records, &owner, member) else {
        return SemanticValueType::Unknown;
    };
    match field_type {
        RecordFieldType::String { .. } => SemanticValueType::String,
        RecordFieldType::Int16 { .. } => SemanticValueType::Integer,
        RecordFieldType::Int { .. } | RecordFieldType::Int32 { .. } => SemanticValueType::Long,
        RecordFieldType::Float32 { .. } => SemanticValueType::Single,
        RecordFieldType::Float64 { .. } => SemanticValueType::Double,
        RecordFieldType::Record { .. } => SemanticValueType::Unknown,
    }
}

fn resolve_record_field_path<'a>(
    records: &'a [Record],
    owner: &str,
    path: &str,
) -> Option<&'a RecordFieldType> {
    fn field_in_record<'a>(
        records: &'a [Record],
        owner: &str,
        field_name: &str,
        visited: &mut BTreeSet<String>,
    ) -> Option<&'a RecordFieldType> {
        if !visited.insert(owner.to_ascii_lowercase()) {
            return None;
        }
        let record = records
            .iter()
            .find(|record| record.name.eq_ignore_ascii_case(owner))?;
        if let Some(field) = record
            .fields
            .iter()
            .find(|field| field.name.eq_ignore_ascii_case(field_name))
        {
            return Some(&field.field_type);
        }
        record
            .combines
            .iter()
            .find_map(|combined| field_in_record(records, combined, field_name, visited))
    }

    let mut record_name = owner;
    let mut segments = path.split('.').peekable();
    while let Some(segment) = segments.next() {
        let field_type = field_in_record(records, record_name, segment, &mut BTreeSet::new())?;
        if segments.peek().is_none() {
            return Some(field_type);
        }
        let RecordFieldType::Record { name, .. } = field_type else {
            return None;
        };
        record_name = name;
    }
    None
}

trait SemanticNameSink {
    fn insert(&mut self, name: String) -> bool;

    fn insert_typed(&mut self, name: String, value_type: SemanticValueType) {
        let _ = value_type;
        self.insert(name.to_ascii_lowercase());
    }
}

impl SemanticNameSink for BTreeSet<String> {
    fn insert(&mut self, name: String) -> bool {
        BTreeSet::insert(self, name)
    }
}

#[derive(Default)]
struct TypedSemanticNames {
    names: BTreeSet<String>,
    typed: Vec<(String, SemanticValueType)>,
}

impl SemanticNameSink for TypedSemanticNames {
    fn insert(&mut self, name: String) -> bool {
        self.names.insert(name)
    }

    fn insert_typed(&mut self, name: String, value_type: SemanticValueType) {
        let name = name.to_ascii_lowercase();
        self.names.insert(name.clone());
        self.typed.push((name, value_type));
    }
}

fn collect_semantic_expression(
    expression: &Expression,
    names: &mut impl SemanticNameSink,
) {
    match &expression.kind {
        ExpressionKind::Name(name) => {
            names.insert_typed(name.clone(), expression.value_type);
        }
        ExpressionKind::Call { arguments, .. } => {
            for argument in arguments {
                collect_semantic_expression(argument, names);
            }
        }
        ExpressionKind::Index { name, index } => {
            names.insert_typed(name.clone(), expression.value_type);
            collect_semantic_expression(index, names);
        }
        ExpressionKind::MultiIndex { name, indices } => {
            names.insert_typed(name.clone(), expression.value_type);
            for index in indices {
                collect_semantic_expression(index, names);
            }
        }
        ExpressionKind::Member {
            base, arguments, ..
        } => {
            if let Some(base) = base {
                collect_semantic_expression(base, names);
            }
            if let Some(arguments) = arguments {
                for argument in arguments {
                    collect_semantic_expression(argument, names);
                }
            }
        }
        ExpressionKind::Parenthesized(inner) | ExpressionKind::Unary { operand: inner, .. } => {
            collect_semantic_expression(inner, names);
        }
        ExpressionKind::Binary { left, right, .. } => {
            collect_semantic_expression(left, names);
            collect_semantic_expression(right, names);
        }
        ExpressionKind::RecordLiteral(fields) | ExpressionKind::PartialRecordLiteral(fields) => {
            for field in fields {
                collect_semantic_expression(&field.value, names);
            }
        }
        ExpressionKind::Literal(_) | ExpressionKind::Boolean(_) => {}
    }
}

fn collect_semantic_statements(
    statements: &[SemanticStatement],
    names: &mut impl SemanticNameSink,
) {
    for statement in statements {
        match &statement.kind {
            SemanticStatementKind::Line(body) => collect_semantic_statements(body, names),
            SemanticStatementKind::Assignment { target, value, .. } => {
                collect_semantic_expression(target, names);
                collect_semantic_expression(value, names);
            }
            SemanticStatementKind::MidAssign {
                target,
                start,
                length,
                value,
            } => {
                collect_semantic_expression(target, names);
                collect_semantic_expression(start, names);
                if let Some(length) = length {
                    collect_semantic_expression(length, names);
                }
                collect_semantic_expression(value, names);
            }
            SemanticStatementKind::Expression(expression)
            | SemanticStatementKind::Error(expression)
            | SemanticStatementKind::OptionBase(expression)
            | SemanticStatementKind::Kill(expression)
            | SemanticStatementKind::Close(expression) => {
                collect_semantic_expression(expression, names)
            }
            SemanticStatementKind::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                collect_semantic_expression(condition, names);
                collect_semantic_statements(then_body, names);
                collect_semantic_statements(else_body, names);
            }
            SemanticStatementKind::While { condition, body } => {
                collect_semantic_expression(condition, names);
                collect_semantic_statements(body, names);
            }
            SemanticStatementKind::For {
                variable,
                start,
                bounds,
                body,
                ..
            } => {
                names.insert(variable.to_ascii_lowercase());
                collect_semantic_expression(start, names);
                match bounds {
                    ForBounds::To { limit, step } => {
                        collect_semantic_expression(limit, names);
                        if let Some(step) = step {
                            collect_semantic_expression(step, names);
                        }
                    }
                    ForBounds::Downto { limit, .. } => collect_semantic_expression(limit, names),
                }
                collect_semantic_statements(body, names);
            }
            SemanticStatementKind::Do {
                pre_condition,
                post_condition,
                body,
            } => {
                for condition in pre_condition.iter().chain(post_condition.iter()) {
                    collect_semantic_expression(&condition.value, names);
                }
                collect_semantic_statements(body, names);
            }
            SemanticStatementKind::SelectCase {
                selector,
                cases,
                else_body,
            } => {
                collect_semantic_expression(selector, names);
                for case in cases {
                    for value in &case.values {
                        match value {
                            CaseValue::Comparison { value, .. } => {
                                collect_semantic_expression(value, names)
                            }
                            CaseValue::Value {
                                first, range_end, ..
                            } => {
                                collect_semantic_expression(first, names);
                                if let Some(end) = range_end {
                                    collect_semantic_expression(end, names);
                                }
                            }
                        }
                    }
                    collect_semantic_statements(&case.body, names);
                }
                collect_semantic_statements(else_body, names);
            }
            SemanticStatementKind::Try {
                body,
                catch,
                finally_body,
            } => {
                collect_semantic_statements(body, names);
                if let Some(catch) = catch {
                    names.insert(catch.error.to_ascii_lowercase());
                    names.insert(catch.line.to_ascii_lowercase());
                    if let Some(source) = &catch.source {
                        names.insert(source.to_ascii_lowercase());
                    }
                    for filter in &catch.filters {
                        collect_semantic_expression(filter, names);
                    }
                    collect_semantic_statements(&catch.body, names);
                }
                collect_semantic_statements(finally_body, names);
            }
            SemanticStatementKind::Return(ReturnValue::Value(value))
            | SemanticStatementKind::Throw(ThrowValue::Value(value)) => {
                collect_semantic_expression(value, names)
            }
            SemanticStatementKind::OnBranch {
                selector, targets, ..
            } => {
                collect_semantic_expression(selector, names);
                for target in targets {
                    names.insert(target.name.to_ascii_lowercase());
                }
            }
            SemanticStatementKind::Goto(target) | SemanticStatementKind::Gosub(target) => {
                names.insert(target.name.to_ascii_lowercase());
            }
            SemanticStatementKind::Resume(Some(ResumeTarget::Label(target))) => {
                names.insert(target.name.to_ascii_lowercase());
            }
            SemanticStatementKind::Resume(Some(ResumeTarget::Next)) => {}
            SemanticStatementKind::OnErrorGoto(ErrorHandlerTarget::Label(target)) => {
                names.insert(target.name.to_ascii_lowercase());
            }
            SemanticStatementKind::OnErrorGoto(ErrorHandlerTarget::Disable) => {}
            SemanticStatementKind::Print {
                destination,
                tokens,
            } => {
                match destination {
                    PrintDestination::Channel { channel, using, .. } => {
                        collect_semantic_expression(channel, names);
                        if let Some(value) = using {
                            collect_semantic_expression(value, names);
                        }
                    }
                    PrintDestination::Using { format, .. } => {
                        collect_semantic_expression(format, names)
                    }
                    PrintDestination::Standard { .. } => {}
                }
                for token in tokens {
                    if let PrintToken::Expression(value) = token {
                        collect_semantic_expression(value, names);
                    }
                }
            }
            SemanticStatementKind::Lprint { using, tokens } => {
                if let Some(value) = using {
                    collect_semantic_expression(value, names);
                }
                for token in tokens {
                    if let PrintToken::Expression(value) = token {
                        collect_semantic_expression(value, names);
                    }
                }
            }
            SemanticStatementKind::Input { source, targets } => {
                match source {
                    InputSource::Channel(value) => collect_semantic_expression(value, names),
                    InputSource::Console(_) => {}
                }
                for value in targets {
                    collect_semantic_expression(value, names);
                }
            }
            SemanticStatementKind::Write { channel, values } => {
                collect_semantic_expression(channel, names);
                if let WriteValues::Values(values) = values {
                    for value in values {
                        collect_semantic_expression(value, names);
                    }
                }
            }
            SemanticStatementKind::Open {
                path,
                channel,
                length,
                ..
            } => {
                collect_semantic_expression(path, names);
                collect_semantic_expression(channel, names);
                if let Some(value) = length {
                    collect_semantic_expression(value, names);
                }
            }
            SemanticStatementKind::Seek { channel, position } => {
                collect_semantic_expression(channel, names);
                collect_semantic_expression(position, names);
            }
            SemanticStatementKind::Rename {
                source,
                destination,
            } => {
                collect_semantic_expression(source, names);
                collect_semantic_expression(destination, names);
            }
            SemanticStatementKind::Data(values) | SemanticStatementKind::Read(values) => {
                for value in values {
                    collect_semantic_expression(value, names);
                }
            }
            SemanticStatementKind::Restore(target) => {
                if let Some(target) = target {
                    names.insert(target.name.to_ascii_lowercase());
                }
            }
            SemanticStatementKind::Swap { left, right } => {
                collect_semantic_expression(left, names);
                collect_semantic_expression(right, names);
            }
            SemanticStatementKind::Randomize(RandomizeSeed::Value(value)) => {
                collect_semantic_expression(value, names)
            }
            SemanticStatementKind::Poke { address, value } => {
                collect_semantic_expression(address, names);
                collect_semantic_expression(value, names);
            }
            SemanticStatementKind::Out { port, value } => {
                collect_semantic_expression(port, names);
                collect_semantic_expression(value, names);
            }
            SemanticStatementKind::Width { channel, value } => {
                if let Some(value) = channel {
                    collect_semantic_expression(value, names);
                }
                collect_semantic_expression(value, names);
            }
            SemanticStatementKind::LineInput { channel, target } => {
                collect_semantic_expression(channel, names);
                collect_semantic_expression(target, names);
            }
            SemanticStatementKind::Get { channel, position }
            | SemanticStatementKind::Put { channel, position } => {
                collect_semantic_expression(channel, names);
                if let Some(position) = position {
                    if let Some(value) = &position.position {
                        collect_semantic_expression(value, names);
                    }
                    if let Some(value) = &position.record {
                        collect_semantic_expression(value, names);
                    }
                }
            }
            SemanticStatementKind::Lset { target, value }
            | SemanticStatementKind::Rset { target, value } => {
                names.insert(target.name.to_ascii_lowercase());
                collect_semantic_expression(value, names);
            }
            SemanticStatementKind::Locate { row, column } => {
                collect_semantic_expression(row, names);
                collect_semantic_expression(column, names);
            }
            SemanticStatementKind::Color {
                foreground,
                background,
            } => {
                collect_semantic_expression(foreground, names);
                if let Some(value) = background {
                    collect_semantic_expression(value, names);
                }
            }
            SemanticStatementKind::FileDeclaration { name, path, .. } => {
                names.insert(name.name.to_ascii_lowercase());
                collect_semantic_expression(path, names);
            }
            SemanticStatementKind::Field { channel, bindings } => {
                collect_semantic_expression(channel, names);
                for binding in bindings {
                    collect_semantic_expression(&binding.length, names);
                    names.insert(binding.name.to_ascii_lowercase());
                }
            }
            SemanticStatementKind::Erase(_)
            | SemanticStatementKind::Label(_)
            | SemanticStatementKind::Resume(None)
            | SemanticStatementKind::Return(ReturnValue::Default)
            | SemanticStatementKind::Throw(ThrowValue::Bare)
            | SemanticStatementKind::Exit
            | SemanticStatementKind::Continue
            | SemanticStatementKind::Comment { .. }
            | SemanticStatementKind::Unsupported
            | SemanticStatementKind::Stop
            | SemanticStatementKind::Clear
            | SemanticStatementKind::Cls
            | SemanticStatementKind::Beep
            | SemanticStatementKind::System => {}
            SemanticStatementKind::Dim(declarations) => {
                for declaration in declarations {
                    names.insert(declaration.name.to_ascii_lowercase());
                }
            }
            SemanticStatementKind::Const { name, value, .. } => {
                names.insert(name.name.to_ascii_lowercase());
                collect_semantic_expression(value, names);
            }
            SemanticStatementKind::Global { name, .. } => {
                names.insert(name.name.to_ascii_lowercase());
            }
            _ => {}
        }
    }
}

fn collect_semantic_global_declarations(
    statements: &[SemanticStatement],
    globals: &mut BTreeSet<String>,
    global_types: &mut BTreeMap<String, SemanticValueType>,
) {
    for statement in statements {
        match &statement.kind {
            SemanticStatementKind::Global { name, value_type } => {
                let key = name.name.to_ascii_lowercase();
                globals.insert(key.clone());
                global_types.insert(key, *value_type);
            }
            SemanticStatementKind::Line(body)
            | SemanticStatementKind::While { body, .. }
            | SemanticStatementKind::For { body, .. }
            | SemanticStatementKind::Do { body, .. } => {
                collect_semantic_global_declarations(body, globals, global_types);
            }
            SemanticStatementKind::If {
                then_body,
                else_body,
                ..
            } => {
                collect_semantic_global_declarations(then_body, globals, global_types);
                collect_semantic_global_declarations(else_body, globals, global_types);
            }
            SemanticStatementKind::SelectCase {
                cases, else_body, ..
            } => {
                for case in cases {
                    collect_semantic_global_declarations(&case.body, globals, global_types);
                }
                collect_semantic_global_declarations(else_body, globals, global_types);
            }
            SemanticStatementKind::Try {
                body,
                catch,
                finally_body,
            } => {
                collect_semantic_global_declarations(body, globals, global_types);
                if let Some(catch) = catch {
                    collect_semantic_global_declarations(&catch.body, globals, global_types);
                }
                collect_semantic_global_declarations(finally_body, globals, global_types);
            }
            _ => {}
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModuleHeader {
    Program {
        name: String,
        shared: Option<String>,
        span: SourceSpan,
    },
    Library {
        name: String,
        span: SourceSpan,
    },
    Shared {
        name: String,
        span: SourceSpan,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dependency {
    pub kind: DependencyKind,
    pub path: String,
    pub span: SourceSpan,
    /// Set by the driver after this dependency and its transitive graph load
    /// successfully; resolved dependencies remain metadata but are not
    /// emitted as unresolved dependency annotations by codegen backends.
    pub resolved: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DependencyKind {
    Require,
    Import,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    pub name: String,
    pub name_span: SourceSpan,
    pub combines: Vec<String>,
    pub fields: Vec<RecordField>,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordField {
    pub name: String,
    pub name_span: SourceSpan,
    pub field_type: RecordFieldType,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecordFieldType {
    String {
        capacity: Option<String>,
        alignment: Option<StringAlignment>,
        span: SourceSpan,
    },
    Int16 {
        span: SourceSpan,
    },
    Int {
        span: SourceSpan,
    },
    Int32 {
        span: SourceSpan,
    },
    Float32 {
        span: SourceSpan,
    },
    Float64 {
        span: SourceSpan,
    },
    Record {
        name: String,
        span: SourceSpan,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CallableSignature {
    pub kind: CallableKind,
    pub name: String,
    pub name_span: SourceSpan,
    pub result_type: Option<String>,
    pub receiver: Option<String>,
    pub receiver_span: Option<SourceSpan>,
    pub parameters: Vec<Parameter>,
    pub body: Vec<SemanticStatement>,
    pub span: SourceSpan,
    /// Index into `SemanticModule::sources` for offsets in this callable's
    /// spans. `usize::MAX` means the adapter did not retain source identity.
    pub source_index: usize,
}

impl CallableSignature {
    /// Return CONST initializers in this callable's scope, including
    /// declarations nested in structured statements.
    pub fn const_initializers(&self) -> HashMap<String, Expression> {
        fn visit(statements: &[SemanticStatement], values: &mut HashMap<String, Expression>) {
            for statement in statements {
                match &statement.kind {
                    SemanticStatementKind::Const { name, value, .. } => {
                        values.insert(name.name.to_ascii_lowercase(), value.clone());
                    }
                    SemanticStatementKind::Line(body)
                    | SemanticStatementKind::While { body, .. }
                    | SemanticStatementKind::For { body, .. }
                    | SemanticStatementKind::Do { body, .. } => visit(body, values),
                    SemanticStatementKind::If {
                        then_body,
                        else_body,
                        ..
                    } => {
                        visit(then_body, values);
                        visit(else_body, values);
                    }
                    SemanticStatementKind::SelectCase {
                        cases, else_body, ..
                    } => {
                        for case in cases {
                            visit(&case.body, values);
                        }
                        visit(else_body, values);
                    }
                    SemanticStatementKind::Try {
                        body,
                        catch,
                        finally_body,
                    } => {
                        visit(body, values);
                        if let Some(catch) = catch {
                            visit(&catch.body, values);
                        }
                        visit(finally_body, values);
                    }
                    _ => {}
                }
            }
        }
        let mut values = HashMap::new();
        visit(&self.body, &mut values);
        values
    }

    /// Return DIM declarations in this callable's scope, including declarations
    /// nested in structured statements.
    pub fn dim_declarations(&self) -> HashMap<String, DimDeclaration> {
        fn visit(
            statements: &[SemanticStatement],
            declarations: &mut HashMap<String, DimDeclaration>,
        ) {
            for statement in statements {
                match &statement.kind {
                    SemanticStatementKind::Dim(items) => {
                        for item in items {
                            declarations.insert(item.name.to_ascii_lowercase(), item.clone());
                        }
                    }
                    SemanticStatementKind::Line(body)
                    | SemanticStatementKind::While { body, .. }
                    | SemanticStatementKind::For { body, .. }
                    | SemanticStatementKind::Do { body, .. } => visit(body, declarations),
                    SemanticStatementKind::If {
                        then_body,
                        else_body,
                        ..
                    } => {
                        visit(then_body, declarations);
                        visit(else_body, declarations);
                    }
                    SemanticStatementKind::SelectCase {
                        cases, else_body, ..
                    } => {
                        for case in cases {
                            visit(&case.body, declarations);
                        }
                        visit(else_body, declarations);
                    }
                    SemanticStatementKind::Try {
                        body,
                        catch,
                        finally_body,
                    } => {
                        visit(body, declarations);
                        if let Some(catch) = catch {
                            visit(&catch.body, declarations);
                        }
                        visit(finally_body, declarations);
                    }
                    _ => {}
                }
            }
        }
        let mut declarations = HashMap::new();
        visit(&self.body, &mut declarations);
        declarations
    }
}
impl CallableSignature {
    /// Return declared parameter ranks and observed index counts in the
    /// semantic body. Observed uses diagnose omitted or inconsistent array
    /// declarations; they do not synthesize a missing declaration.
    pub fn parameter_array_ranks(&self) -> (Vec<Option<usize>>, Vec<Vec<usize>>) {
        fn expression_rank(expression: &Expression, name: &str, ranks: &mut BTreeSet<usize>) {
            match &expression.kind {
                ExpressionKind::Index {
                    name: indexed,
                    index,
                } => {
                    if indexed.eq_ignore_ascii_case(name) {
                        ranks.insert(1);
                    }
                    expression_rank(index, name, ranks);
                }
                ExpressionKind::MultiIndex {
                    name: indexed,
                    indices,
                } => {
                    if indexed.eq_ignore_ascii_case(name) {
                        ranks.insert(indices.len());
                    }
                    for index in indices {
                        expression_rank(index, name, ranks);
                    }
                }
                ExpressionKind::Call {
                    name: called,
                    arguments,
                } => {
                    if called.eq_ignore_ascii_case(name) {
                        ranks.insert(arguments.len());
                    }
                    for argument in arguments {
                        expression_rank(argument, name, ranks);
                    }
                }
                ExpressionKind::Parenthesized(value)
                | ExpressionKind::Unary { operand: value, .. } => {
                    expression_rank(value, name, ranks)
                }
                ExpressionKind::Binary { left, right, .. } => {
                    expression_rank(left, name, ranks);
                    expression_rank(right, name, ranks);
                }
                ExpressionKind::Member {
                    base, arguments, ..
                } => {
                    if let Some(base) = base {
                        expression_rank(base, name, ranks);
                    }
                    if let Some(arguments) = arguments {
                        for argument in arguments {
                            expression_rank(argument, name, ranks);
                        }
                    }
                }
                ExpressionKind::RecordLiteral(fields)
                | ExpressionKind::PartialRecordLiteral(fields) => {
                    for field in fields {
                        expression_rank(&field.value, name, ranks);
                    }
                }
                ExpressionKind::Name(_)
                | ExpressionKind::Literal(_)
                | ExpressionKind::Boolean(_) => {}
            }
        }
        fn statements_rank(
            statements: &[SemanticStatement],
            name: &str,
            ranks: &mut BTreeSet<usize>,
        ) {
            use SemanticStatementKind as Kind;
            for statement in statements {
                match &statement.kind {
                    Kind::Line(body)
                    | Kind::While { body, .. }
                    | Kind::For { body, .. }
                    | Kind::Do { body, .. } => {
                        if let Kind::While { condition, .. } = &statement.kind {
                            expression_rank(condition, name, ranks);
                        }
                        if let Kind::For { start, .. } = &statement.kind {
                            expression_rank(start, name, ranks);
                        }
                        statements_rank(body, name, ranks);
                    }
                    Kind::Assignment { target, value, .. } => {
                        expression_rank(target, name, ranks);
                        expression_rank(value, name, ranks);
                    }
                    Kind::MidAssign {
                        target,
                        start,
                        length,
                        value,
                    } => {
                        expression_rank(target, name, ranks);
                        expression_rank(start, name, ranks);
                        if let Some(length) = length {
                            expression_rank(length, name, ranks);
                        }
                        expression_rank(value, name, ranks);
                    }
                    Kind::Expression(value) | Kind::Error(value) => {
                        expression_rank(value, name, ranks)
                    }
                    Kind::If {
                        condition,
                        then_body,
                        else_body,
                        ..
                    } => {
                        expression_rank(condition, name, ranks);
                        statements_rank(then_body, name, ranks);
                        statements_rank(else_body, name, ranks);
                    }
                    Kind::SelectCase {
                        selector,
                        cases,
                        else_body,
                    } => {
                        expression_rank(selector, name, ranks);
                        for case in cases {
                            statements_rank(&case.body, name, ranks);
                        }
                        statements_rank(else_body, name, ranks);
                    }
                    Kind::Try {
                        body,
                        catch,
                        finally_body,
                    } => {
                        statements_rank(body, name, ranks);
                        if let Some(catch) = catch {
                            statements_rank(&catch.body, name, ranks);
                        }
                        statements_rank(finally_body, name, ranks);
                    }
                    Kind::Return(ReturnValue::Value(value))
                    | Kind::Throw(ThrowValue::Value(value)) => expression_rank(value, name, ranks),
                    Kind::Print {
                        destination,
                        tokens,
                    } => {
                        match destination {
                            PrintDestination::Channel { channel, using, .. } => {
                                expression_rank(channel, name, ranks);
                                if let Some(using) = using {
                                    expression_rank(using, name, ranks);
                                }
                            }
                            PrintDestination::Using { format, .. } => {
                                expression_rank(format, name, ranks)
                            }
                            PrintDestination::Standard { .. } => {}
                        }
                        for token in tokens {
                            if let PrintToken::Expression(value) = token {
                                expression_rank(value, name, ranks);
                            }
                        }
                    }
                    Kind::Input { source, targets } => {
                        if let InputSource::Channel(channel) = source {
                            expression_rank(channel, name, ranks);
                        }
                        for target in targets {
                            expression_rank(target, name, ranks);
                        }
                    }
                    Kind::Read(values) | Kind::Data(values) => {
                        for value in values {
                            expression_rank(value, name, ranks);
                        }
                    }
                    _ => {}
                }
            }
        }

        let mut ranks = Vec::with_capacity(self.parameters.len());
        let mut observed = Vec::with_capacity(self.parameters.len());
        for parameter in &self.parameters {
            let mut uses = BTreeSet::new();
            statements_rank(&self.body, &parameter.name, &mut uses);
            ranks.push((parameter.array_axes > 0).then_some(parameter.array_axes));
            observed.push(uses.into_iter().collect());
        }
        (ranks, observed)
    }

    /// Return array ranks declared in this callable, including nested blocks.
    pub fn array_ranks(&self) -> HashMap<String, usize> {
        fn visit(statements: &[SemanticStatement], ranks: &mut HashMap<String, usize>) {
            for statement in statements {
                match &statement.kind {
                    SemanticStatementKind::Dim(items) => {
                        for item in items.iter().filter(|item| item.array_axes > 0) {
                            ranks.insert(item.name.to_ascii_lowercase(), item.array_axes);
                        }
                    }
                    SemanticStatementKind::Line(body)
                    | SemanticStatementKind::While { body, .. }
                    | SemanticStatementKind::For { body, .. }
                    | SemanticStatementKind::Do { body, .. } => visit(body, ranks),
                    SemanticStatementKind::If {
                        then_body,
                        else_body,
                        ..
                    } => {
                        visit(then_body, ranks);
                        visit(else_body, ranks);
                    }
                    SemanticStatementKind::SelectCase {
                        cases, else_body, ..
                    } => {
                        for case in cases {
                            visit(&case.body, ranks);
                        }
                        visit(else_body, ranks);
                    }
                    SemanticStatementKind::Try {
                        body,
                        catch,
                        finally_body,
                    } => {
                        visit(body, ranks);
                        if let Some(catch) = catch {
                            visit(&catch.body, ranks);
                        }
                        visit(finally_body, ranks);
                    }
                    _ => {}
                }
            }
        }

        let mut ranks = HashMap::new();
        visit(&self.body, &mut ranks);
        ranks
    }

    /// Return scalar DIM type annotations declared in this callable.
    pub fn dim_types(&self) -> HashMap<String, String> {
        fn visit(statements: &[SemanticStatement], types: &mut HashMap<String, String>) {
            for statement in statements {
                match &statement.kind {
                    SemanticStatementKind::Dim(items) => {
                        for item in items {
                            if let Some(annotation) = &item.type_annotation {
                                types.insert(item.name.to_ascii_lowercase(), annotation.clone());
                            }
                        }
                    }
                    SemanticStatementKind::Line(body)
                    | SemanticStatementKind::While { body, .. }
                    | SemanticStatementKind::For { body, .. }
                    | SemanticStatementKind::Do { body, .. } => visit(body, types),
                    SemanticStatementKind::If {
                        then_body,
                        else_body,
                        ..
                    } => {
                        visit(then_body, types);
                        visit(else_body, types);
                    }
                    SemanticStatementKind::SelectCase {
                        cases, else_body, ..
                    } => {
                        for case in cases {
                            visit(&case.body, types);
                        }
                        visit(else_body, types);
                    }
                    SemanticStatementKind::Try {
                        body,
                        catch,
                        finally_body,
                    } => {
                        visit(body, types);
                        if let Some(catch) = catch {
                            visit(&catch.body, types);
                        }
                        visit(finally_body, types);
                    }
                    _ => {}
                }
            }
        }

        let mut types = HashMap::new();
        visit(&self.body, &mut types);
        types
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallableKind {
    Function,
    Procedure,
    Method,
    FluentMethod,
    InlineMethod,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parameter {
    pub name: String,
    pub name_span: SourceSpan,
    /// Resolved effective type, including the language's implicit `Single`
    /// type for parameters without an explicit suffix.
    pub value_type: SemanticValueType,
    /// Explicit source suffix, retained for syntax-sensitive target constraints.
    pub type_suffix: Option<String>,
    /// Explicit declaration type, retained after the frontend parse.
    pub type_annotation: Option<String>,
    pub passing: Option<Passing>,
    pub array_axes: usize,
    /// Per-axis capacity facts from the parameter declaration. `Inferred`
    /// marks `?`; fixed literals and typed expressions retain their source
    /// representation for downstream declaration and backend consumers.
    pub dimensions: Vec<DimAxis>,
    pub default: Option<Expression>,
    pub span: SourceSpan,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Passing {
    ByRef,
    ByVal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SemanticValueType {
    Unknown,
    String,
    Integer,
    Long,
    Single,
    Double,
    Boolean,
}
impl SemanticValueType {
    pub fn from_suffix(suffix: Option<char>) -> Self {
        match suffix {
            Some('$') => Self::String,
            Some('%') => Self::Integer,
            Some('&') => Self::Long,
            Some('!') => Self::Single,
            Some('#') => Self::Double,
            _ => Self::Unknown,
        }
    }

    pub fn suffix(self) -> Option<char> {
        match self {
            Self::String => Some('$'),
            Self::Integer => Some('%'),
            Self::Long => Some('&'),
            Self::Single => Some('!'),
            Self::Double => Some('#'),
            Self::Unknown | Self::Boolean => None,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub span: SourceSpan,
    pub value_type: SemanticValueType,
    /// Nominal record identity when the expression has a declared record type.
    pub record_type: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExpressionKind {
    Name(String),
    Literal(String),
    Boolean(bool),
    Parenthesized(Box<Expression>),
    Unary {
        operator: String,
        operand: Box<Expression>,
    },
    Binary {
        left: Box<Expression>,
        operator: String,
        right: Box<Expression>,
    },
    Call {
        name: String,
        arguments: Vec<Expression>,
    },
    Index {
        name: String,
        index: Box<Expression>,
    },
    MultiIndex {
        name: String,
        indices: Vec<Expression>,
    },
    Member {
        base: Option<Box<Expression>>,
        member: String,
        arguments: Option<Vec<Expression>>,
    },
    RecordLiteral(Vec<FieldInitializer>),
    PartialRecordLiteral(Vec<FieldInitializer>),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldInitializer {
    pub name: String,
    pub value: Expression,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssignmentOperator {
    Assign,
    Add,
    Subtract,
    Multiply,
    Divide,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StringAlignment {
    LeftPad,
    Left,
    RightPad,
    Right,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SemanticStatement {
    pub kind: SemanticStatementKind,
    pub span: SourceSpan,
}

/// Whether a typed-IR callable name denotes an AST function. A scalar method
/// is named by its bare spelling (`ucase`) in the typed IR, while the AST
/// function carries the synthesized result suffix (`ucase$`).
pub(crate) fn callable_name_matches_function(
    name: &str,
    function: &crate::ast::FunctionDef,
) -> bool {
    name.eq_ignore_ascii_case(&function.name.as_basic())
        || (function.receiver.is_some() && name.eq_ignore_ascii_case(&function.name.name))
}

/// The identifier an AST function is keyed by. A scalar method keeps the
/// result suffix the AST function carries, since the typed IR names it bare.
pub(crate) fn callable_ident_for_function(
    name: &str,
    function: &crate::ast::FunctionDef,
) -> crate::ast::BasicIdent {
    if function.receiver.is_some() && name.eq_ignore_ascii_case(&function.name.name) {
        function.name.clone()
    } else {
        crate::ast::BasicIdent::parse(name)
    }
}

impl SemanticStatement {
    /// A `'` or `//` comment. When one trails a statement on its line, the
    /// legacy parser discards it, so it has no AST counterpart to align to.
    pub fn is_line_comment(&self) -> bool {
        matches!(self.kind, SemanticStatementKind::Comment { block: false, .. })
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SemanticStatementKind {
    Line(Vec<SemanticStatement>),
    Assignment {
        target: Expression,
        operator: AssignmentOperator,
        value: Expression,
    },
    MidAssign {
        target: Expression,
        start: Expression,
        length: Option<Expression>,
        value: Expression,
    },
    Expression(Expression),
    If {
        condition: Expression,
        then_body: Vec<SemanticStatement>,
        else_body: Vec<SemanticStatement>,
        block: bool,
    },
    Unsupported,
    While {
        condition: Expression,
        body: Vec<SemanticStatement>,
    },
    For {
        variable: String,
        variable_type: SemanticValueType,
        start: Expression,
        bounds: ForBounds,
        body: Vec<SemanticStatement>,
    },
    Do {
        pre_condition: Option<LoopCondition>,
        post_condition: Option<LoopCondition>,
        body: Vec<SemanticStatement>,
    },
    SelectCase {
        selector: Expression,
        cases: Vec<CaseClause>,
        else_body: Vec<SemanticStatement>,
    },
    Try {
        body: Vec<SemanticStatement>,
        catch: Option<CatchBinding>,
        finally_body: Vec<SemanticStatement>,
    },
    Label(NamedReference),
    Comment {
        block: bool,
        text: String,
    },
    Return(ReturnValue),
    Exit,
    Continue,
    End,
    Goto(NamedReference),
    Gosub(NamedReference),
    Resume(Option<ResumeTarget>),
    OnErrorGoto(ErrorHandlerTarget),
    OnBranch {
        selector: Expression,
        branch: BranchKind,
        targets: Vec<NamedReference>,
    },
    Error(Expression),
    Throw(ThrowValue),
    OptionBase(Expression),
    Erase(Vec<NamedReference>),
    Print {
        destination: PrintDestination,
        tokens: Vec<PrintToken>,
    },
    Input {
        source: InputSource,
        targets: Vec<Expression>,
    },
    Write {
        channel: Expression,
        values: WriteValues,
    },
    Open {
        path: Expression,
        mode: OpenMode,
        channel: Expression,
        length: Option<Expression>,
    },
    Seek {
        channel: Expression,
        position: Expression,
    },
    Kill(Expression),
    Rename {
        source: Expression,
        destination: Expression,
    },
    Close(Expression),
    Data(Vec<Expression>),
    Read(Vec<Expression>),
    Restore(Option<NamedReference>),
    Dim(Vec<DimDeclaration>),
    Const {
        name: NamedReference,
        value: Expression,
        value_type: SemanticValueType,
    },
    Global {
        name: NamedReference,
        value_type: SemanticValueType,
    },
    Swap {
        left: Expression,
        right: Expression,
    },
    Randomize(RandomizeSeed),
    Poke {
        address: Expression,
        value: Expression,
    },
    Out {
        port: Expression,
        value: Expression,
    },
    Width {
        channel: Option<Expression>,
        value: Expression,
    },
    LineInput {
        channel: Expression,
        target: Expression,
    },
    Get {
        channel: Expression,
        position: Option<FilePosition>,
    },
    Put {
        channel: Expression,
        position: Option<FilePosition>,
    },
    Lset {
        target: NamedReference,
        value: Expression,
    },
    Rset {
        target: NamedReference,
        value: Expression,
    },
    Locate {
        row: Expression,
        column: Expression,
    },
    Color {
        foreground: Expression,
        background: Option<Expression>,
    },
    Stop,
    Clear,
    Cls,
    Beep,
    System,
    Lprint {
        using: Option<Expression>,
        tokens: Vec<PrintToken>,
    },
    FileDeclaration {
        name: NamedReference,
        record_type: Option<NamedReference>,
        path: Expression,
        mode: Option<FileModeKind>,
    },
    Field {
        channel: Expression,
        bindings: Vec<FieldBinding>,
    },
}

fn semantic_statements_have_statement(
    statements: &[SemanticStatement],
    predicate: impl Fn(&SemanticStatementKind) -> bool + Copy,
) -> bool {
    statements.iter().any(|statement| {
        if predicate(&statement.kind) {
            return true;
        }
        match &statement.kind {
            SemanticStatementKind::Line(body)
            | SemanticStatementKind::While { body, .. }
            | SemanticStatementKind::For { body, .. }
            | SemanticStatementKind::Do { body, .. } => {
                semantic_statements_have_statement(body, predicate)
            }
            SemanticStatementKind::If {
                then_body,
                else_body,
                ..
            } => {
                semantic_statements_have_statement(then_body, predicate)
                    || semantic_statements_have_statement(else_body, predicate)
            }
            SemanticStatementKind::SelectCase {
                cases, else_body, ..
            } => {
                cases
                    .iter()
                    .any(|case| semantic_statements_have_statement(&case.body, predicate))
                    || semantic_statements_have_statement(else_body, predicate)
            }
            SemanticStatementKind::Try {
                body,
                catch,
                finally_body,
            } => {
                semantic_statements_have_statement(body, predicate)
                    || catch.as_ref().is_some_and(|binding| {
                        semantic_statements_have_statement(&binding.body, predicate)
                    })
                    || semantic_statements_have_statement(finally_body, predicate)
            }
            _ => false,
        }
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DimAxis {
    Inferred,
    Fixed(String),
    Expression(Expression),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DimDeclaration {
    pub name: String,
    pub array_axes: usize,
    pub dimensions: Vec<DimAxis>,
    pub type_annotation: Option<String>,
    /// Resolved builtin element type for typed array declarations. This
    /// preserves suffix-based inference without requiring codegen to
    /// reinterpret the source identifier.
    pub element_type: SemanticValueType,
    pub span: SourceSpan,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FilePosition {
    pub position: Option<Expression>,
    pub record: Option<Expression>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldBinding {
    pub length: Expression,
    pub name: String,
    pub name_span: SourceSpan,
    pub type_suffix: Option<String>,
    pub is_string: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputPrompt {
    pub text: String,
    pub span: SourceSpan,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenMode {
    pub kind: OpenModeKind,
    pub span: SourceSpan,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpenModeKind {
    Input,
    Output,
    Append,
    Random,
    Binary,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileModeKind {
    Input,
    Output,
    Append,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputSource {
    Channel(Expression),
    Console(Option<InputPrompt>),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WriteValues {
    Omitted,
    Values(Vec<Expression>),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamedReference {
    pub name: String,
    pub span: SourceSpan,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PrintDestination {
    Standard {
        span: SourceSpan,
    },
    Channel {
        channel: Expression,
        using: Option<Expression>,
        span: SourceSpan,
    },
    Using {
        format: Expression,
        span: SourceSpan,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PrintToken {
    Comma { span: SourceSpan },
    Semicolon { span: SourceSpan },
    Expression(Expression),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ForBounds {
    To {
        limit: Expression,
        step: Option<Expression>,
    },
    Downto {
        limit: Expression,
        step: i64,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoopCondition {
    pub kind: LoopConditionKind,
    pub value: Expression,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoopConditionKind {
    While,
    Until,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaseClause {
    pub values: Vec<CaseValue>,
    pub body: Vec<SemanticStatement>,
    pub span: SourceSpan,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CaseValue {
    Comparison {
        operator: ComparisonOperator,
        value: Expression,
        span: SourceSpan,
    },
    Value {
        first: Expression,
        range_end: Option<Expression>,
        span: SourceSpan,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComparisonOperator {
    NotEqual,
    LessOrEqual,
    GreaterOrEqual,
    Equal,
    Less,
    Greater,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResumeTarget {
    Label(NamedReference),
    Next,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorHandlerTarget {
    Label(NamedReference),
    Disable,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BranchKind {
    Goto,
    Gosub,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ThrowValue {
    Bare,
    Value(Expression),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReturnValue {
    Default,
    Value(Expression),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RandomizeSeed {
    Default,
    Value(Expression),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CatchBinding {
    pub error: String,
    pub error_type: SemanticValueType,
    pub error_span: SourceSpan,
    pub line: String,
    pub line_type: SemanticValueType,
    pub line_span: SourceSpan,
    pub filters: Vec<Expression>,
    pub source: Option<String>,
    pub source_type: Option<SemanticValueType>,
    pub source_span: Option<SourceSpan>,
    pub body: Vec<SemanticStatement>,
}

/// Adapt generated semantic declarations without reconstructing the legacy
/// parser AST. Source spans are forwarded verbatim from rdgen nodes.
pub fn adapt_module(program: &rdgen_frontend::Program) -> SemanticModule {
    let rdgen_frontend::Program::File { items, span } = program;
    let mut header = None;
    let mut dependencies = Vec::new();
    let mut records = Vec::new();
    let mut callables = Vec::new();
    let mut statements = Vec::new();
    for item in items {
        match item.as_ref() {
            rdgen_frontend::FileItem::ProgramDeclaration { declaration, .. } => {
                let rdgen_frontend::ProgramDecl::ProgramDeclaration { name, shared, span } =
                    declaration.as_ref();
                header = Some(ModuleHeader::Program {
                    name: identifier(name),
                    shared: shared.as_ref().map(|(_, name)| identifier(name)),
                    span: *span,
                });
            }
            rdgen_frontend::FileItem::LibraryDeclaration { declaration, .. } => {
                let rdgen_frontend::LibraryDecl::LibraryDeclaration { name, span } =
                    declaration.as_ref();
                header = Some(ModuleHeader::Library {
                    name: identifier(name),
                    span: *span,
                });
            }
            rdgen_frontend::FileItem::SharedDeclaration { declaration, .. } => {
                let rdgen_frontend::SharedDecl::SharedDeclaration { name, span } =
                    declaration.as_ref();
                header = Some(ModuleHeader::Shared {
                    name: identifier(name),
                    span: *span,
                });
            }
            rdgen_frontend::FileItem::RequireDeclaration { declaration, .. } => {
                let rdgen_frontend::RequireDecl::RequireDeclaration { path, span } =
                    declaration.as_ref();
                dependencies.push(Dependency {
                    kind: DependencyKind::Require,
                    path: identifier(path),
                    span: *span,
                    resolved: false,
                });
            }
            rdgen_frontend::FileItem::ImportDeclaration { declaration, .. } => {
                let rdgen_frontend::ImportDecl::ImportDeclaration { path, span } =
                    declaration.as_ref();
                dependencies.push(Dependency {
                    kind: DependencyKind::Import,
                    path: identifier(path),
                    span: *span,
                    resolved: false,
                });
            }
            rdgen_frontend::FileItem::Record { record, .. } => {
                let adapted = adapt_record(record);
                callables.extend(adapt_inline_methods(record, &adapted.name));
                records.push(adapted);
            }
            rdgen_frontend::FileItem::Subprogram { subprogram, .. } => {
                callables.push(adapt_subprogram(subprogram))
            }
            rdgen_frontend::FileItem::Statement { statement, .. } => {
                statements.push(adapt_top_level_statement(statement))
            }
        }
    }
    let statement_sources = vec![usize::MAX; statements.len()];
    let mut module = SemanticModule {
        span: *span,
        sources: Vec::new(),
        header,
        dependencies,
        records,
        lowered_record_files: Vec::new(),
        callables,
        statements,
        statement_sources,
    };
    let facts = module.clone();
    facts.annotate_statement_types(&mut module.statements);
    for callable in &mut module.callables {
        let mut callable_facts = facts.clone();
        // Include module declarations first and callable declarations second:
        // the lexical callable scope shadows a same-named module array.
        callable_facts.statements.extend(callable.body.clone());
        let array_parameters = callable
            .parameters
            .iter()
            .filter(|parameter| parameter.array_axes > 0)
            .map(|parameter| DimDeclaration {
                name: parameter.name.clone(),
                array_axes: parameter.array_axes,
                dimensions: parameter.dimensions.clone(),
                element_type: SemanticValueType::from_suffix(parameter.name.chars().last()),
                type_annotation: None,
                span: parameter.span,
            })
            .collect::<Vec<_>>();
        if !array_parameters.is_empty() {
            callable_facts.statements.push(SemanticStatement {
                kind: SemanticStatementKind::Dim(array_parameters),
                span: callable.span,
            });
        }
        callable_facts.annotate_statement_types(&mut callable.body);
    }
    let const_types = module.const_types();
    fn apply_const_types(
        statements: &mut [SemanticStatement],
        const_types: &BTreeMap<String, SemanticValueType>,
    ) {
        for statement in statements {
            match &mut statement.kind {
                SemanticStatementKind::Const {
                    name, value_type, ..
                } => {
                    *value_type = const_types
                        .iter()
                        .find(|(constant, _)| constant.eq_ignore_ascii_case(&name.name))
                        .map(|(_, value_type)| *value_type)
                        .unwrap_or(SemanticValueType::Integer);
                }
                SemanticStatementKind::Line(body)
                | SemanticStatementKind::While { body, .. }
                | SemanticStatementKind::For { body, .. }
                | SemanticStatementKind::Do { body, .. } => apply_const_types(body, const_types),
                SemanticStatementKind::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    apply_const_types(then_body, const_types);
                    apply_const_types(else_body, const_types);
                }
                SemanticStatementKind::SelectCase {
                    cases, else_body, ..
                } => {
                    for case in cases {
                        apply_const_types(&mut case.body, const_types);
                    }
                    apply_const_types(else_body, const_types);
                }
                SemanticStatementKind::Try {
                    body,
                    catch,
                    finally_body,
                } => {
                    apply_const_types(body, const_types);
                    if let Some(catch) = catch {
                        apply_const_types(&mut catch.body, const_types);
                    }
                    apply_const_types(finally_body, const_types);
                }
                _ => {}
            }
        }
    }
    apply_const_types(&mut module.statements, &const_types);
    for callable in &mut module.callables {
        apply_const_types(&mut callable.body, &const_types);
    }
    module
}

/// Parse source with the generated frontend and adapt it into semantic IR.
pub fn parse_and_adapt(source: &str) -> Result<SemanticModule, rdgen_frontend::ParseError> {
    rdgen_frontend::parse(source).map(|program| adapt_module(&program))
}

/// Parse and adapt source while retaining its diagnostic origin.
pub fn parse_and_adapt_named(
    filename: impl Into<String>,
    source: &str,
) -> Result<SemanticModule, rdgen_frontend::ParseError> {
    parse_and_adapt(source).map(|mut module| {
        module.sources.push(SemanticSource {
            filename: filename.into(),
            text: source.to_owned(),
        });
        module.statement_sources.fill(0);
        for callable in &mut module.callables {
            callable.source_index = 0;
        }
        module
    })
}

/// Convert a generated-parser failure into the compiler diagnostic shape.
pub fn parse_diagnostic(
    filename: impl Into<String>,
    error: &rdgen_frontend::ParseError,
) -> crate::diagnostics::Diagnostic {
    crate::diagnostics::Diagnostic::error(
        crate::diagnostics::SourcePos::new(filename, 1, error.position.saturating_add(1)),
        error.message.clone(),
    )
}

fn adapt_top_level_statement(statement: &rdgen_frontend::TopLevelStatement) -> SemanticStatement {
    match statement {
        rdgen_frontend::TopLevelStatement::Statement { statement, .. } => {
            adapt_statement(statement)
        }
        rdgen_frontend::TopLevelStatement::End { span, .. } => SemanticStatement {
            kind: SemanticStatementKind::End,
            span: *span,
        },
    }
}

fn adapt_statement(statement: &rdgen_frontend::Statement) -> SemanticStatement {
    match statement {
        rdgen_frontend::Statement::Line { first, rest, span } => SemanticStatement {
            kind: SemanticStatementKind::Line(
                std::iter::once(first.as_ref())
                    .chain(rest.iter().map(|(_, value)| value.as_ref()))
                    .map(adapt_statement_core)
                    .collect(),
            ),
            span: *span,
        },
        rdgen_frontend::Statement::Label { label, span } => {
            let rdgen_frontend::LabelStmt::Label { name, .. } = label.as_ref();
            SemanticStatement {
                kind: SemanticStatementKind::Label(NamedReference {
                    name: identifier(name),
                    span: name.span(),
                }),
                span: *span,
            }
        }
    }
}

fn adapt_statement_core(statement: &rdgen_frontend::StatementCore) -> SemanticStatement {
    match statement {
        rdgen_frontend::StatementCore::AssignmentOrExpression { statement, span } => {
            match statement.as_ref() {
                rdgen_frontend::AssignmentOrExprStmt::MidAssignment { assignment, .. } => {
                    let rdgen_frontend::MidAssign::MidAssign {
                        target,
                        start,
                        length,
                        value,
                        ..
                    } = assignment.as_ref();
                    SemanticStatement {
                        kind: SemanticStatementKind::MidAssign {
                            target: adapt_expression(target),
                            start: adapt_expression(start),
                            length: length
                                .as_ref()
                                .map(|(_, expression)| adapt_expression(expression)),
                            value: adapt_expression(value),
                        },
                        span: *span,
                    }
                }
                rdgen_frontend::AssignmentOrExprStmt::Assignment {
                    target,
                    operator,
                    value,
                    ..
                } => {
                    let rdgen_frontend::AssignTarget::Target { value: target, .. } =
                        target.as_ref();
                    SemanticStatement {
                        kind: SemanticStatementKind::Assignment {
                            target: adapt_expression(target),
                            operator: assignment_operator(operator),
                            value: adapt_expression(value),
                        },
                        span: *span,
                    }
                }
                rdgen_frontend::AssignmentOrExprStmt::Expression { value, .. } => {
                    SemanticStatement {
                        kind: SemanticStatementKind::Expression(adapt_expression(value)),
                        span: *span,
                    }
                }
            }
        }
        rdgen_frontend::StatementCore::If { if_statement, span } => adapt_if(if_statement, *span),
        rdgen_frontend::StatementCore::While {
            while_statement,
            span,
        } => {
            let rdgen_frontend::WhileStmt::While {
                condition, body, ..
            } = while_statement.as_ref();
            SemanticStatement {
                kind: SemanticStatementKind::While {
                    condition: adapt_expression(condition),
                    body: body.iter().map(|value| adapt_statement(value)).collect(),
                },
                span: *span,
            }
        }
        rdgen_frontend::StatementCore::For {
            for_statement,
            span,
        } => {
            let rdgen_frontend::ForStmt::For {
                variable,
                start,
                bounds,
                body,
                ..
            } = for_statement.as_ref();
            SemanticStatement {
                kind: SemanticStatementKind::For {
                    variable: typed_identifier(variable),
                    variable_type: SemanticValueType::from_suffix(
                        typed_identifier(variable).chars().last(),
                    ),
                    start: adapt_expression(start),
                    bounds: adapt_for_bounds(bounds),
                    body: body.iter().map(|value| adapt_statement(value)).collect(),
                },
                span: *span,
            }
        }
        rdgen_frontend::StatementCore::Do { do_statement, span } => {
            let rdgen_frontend::DoStmt::Do {
                pre_condition,
                body,
                terminator,
                ..
            } = do_statement.as_ref();
            let post_condition = match terminator.as_ref() {
                rdgen_frontend::DoTerminator::Loop { post_condition, .. } => {
                    post_condition.as_deref().map(adapt_loop_condition)
                }
                _ => None,
            };
            SemanticStatement {
                kind: SemanticStatementKind::Do {
                    pre_condition: pre_condition.as_deref().map(adapt_loop_condition),
                    post_condition,
                    body: body.iter().map(|value| adapt_statement(value)).collect(),
                },
                span: *span,
            }
        }
        rdgen_frontend::StatementCore::SelectCase { select_case, span } => {
            let rdgen_frontend::SelectCaseStmt::SelectCase {
                selector, cases, ..
            } = select_case.as_ref();
            let mut semantic_cases = Vec::new();
            let mut else_body = Vec::new();
            for clause in cases {
                match clause.as_ref() {
                    rdgen_frontend::CaseClause::Case { .. } => {
                        semantic_cases.push(adapt_case_clause(clause))
                    }
                    rdgen_frontend::CaseClause::Else { body, .. } => {
                        else_body = body.iter().map(|value| adapt_statement(value)).collect();
                    }
                }
            }
            SemanticStatement {
                kind: SemanticStatementKind::SelectCase {
                    selector: adapt_expression(selector),
                    cases: semantic_cases,
                    else_body,
                },
                span: *span,
            }
        }
        rdgen_frontend::StatementCore::Try {
            try_statement,
            span,
        } => {
            let rdgen_frontend::TryStmt::Try {
                body,
                catch_clause,
                finally_clause,
                ..
            } = try_statement.as_ref();
            SemanticStatement {
                kind: SemanticStatementKind::Try {
                    body: body.iter().map(|value| adapt_statement(value)).collect(),
                    catch: catch_clause.as_deref().map(adapt_catch),
                    finally_body: finally_clause
                        .as_ref()
                        .map(|(_, body)| body.iter().map(|value| adapt_statement(value)).collect())
                        .unwrap_or_default(),
                },
                span: *span,
            }
        }
        rdgen_frontend::StatementCore::Label { label, span } => {
            let rdgen_frontend::LabelStmt::Label { name, .. } = label.as_ref();
            SemanticStatement {
                kind: SemanticStatementKind::Label(NamedReference {
                    name: identifier(name),
                    span: name.span(),
                }),
                span: *span,
            }
        }
        rdgen_frontend::StatementCore::Comment { comment, span } => {
            let (block, text) = match comment.as_ref() {
                rdgen_frontend::CommentStmt::Raw { comment, .. } => {
                    let (prefix, body) = match comment.as_ref() {
                        rdgen_frontend::LineComment::ApostropheComment { prefix, body, .. }
                        | rdgen_frontend::LineComment::SlashComment { prefix, body, .. } => {
                            (prefix, body)
                        }
                    };
                    (
                        false,
                        format!(
                            "{}{}",
                            prefix.0,
                            body.iter()
                                .map(|token| token.0.as_str())
                                .collect::<String>()
                        ),
                    )
                }
                rdgen_frontend::CommentStmt::BlockComment { comment, .. } => {
                    let rdgen_frontend::BlockComment::BlockComment {
                        opening,
                        body,
                        closing,
                        ..
                    } = comment.as_ref();
                    (
                        true,
                        format!(
                            "{}{}{}",
                            opening.0,
                            body.iter()
                                .map(|token| token.0.as_str())
                                .collect::<String>(),
                            closing.0
                        ),
                    )
                }
            };
            SemanticStatement {
                kind: SemanticStatementKind::Comment { block, text },
                span: *span,
            }
        }
        rdgen_frontend::StatementCore::Return {
            return_statement,
            span,
        } => {
            let rdgen_frontend::ReturnStmt::Return { value, .. } = return_statement.as_ref();
            SemanticStatement {
                kind: SemanticStatementKind::Return(
                    value
                        .as_deref()
                        .map(adapt_expression)
                        .map(ReturnValue::Value)
                        .unwrap_or(ReturnValue::Default),
                ),
                span: *span,
            }
        }
        rdgen_frontend::StatementCore::Exit { span, .. } => SemanticStatement {
            kind: SemanticStatementKind::Exit,
            span: *span,
        },
        rdgen_frontend::StatementCore::Continue { span, .. } => SemanticStatement {
            kind: SemanticStatementKind::Continue,
            span: *span,
        },
        rdgen_frontend::StatementCore::Goto { goto, span } => {
            let rdgen_frontend::GotoStmt::Goto { target, .. } = goto.as_ref();
            SemanticStatement {
                kind: SemanticStatementKind::Goto(NamedReference {
                    name: identifier(target),
                    span: target.span(),
                }),
                span: *span,
            }
        }
        rdgen_frontend::StatementCore::Gosub { gosub, span } => {
            let rdgen_frontend::GosubStmt::Gosub { target, .. } = gosub.as_ref();
            SemanticStatement {
                kind: SemanticStatementKind::Gosub(NamedReference {
                    name: identifier(target),
                    span: target.span(),
                }),
                span: *span,
            }
        }
        rdgen_frontend::StatementCore::Resume { resume, span } => {
            let rdgen_frontend::ResumeStmt::Resume { target, .. } = resume.as_ref();
            let target = target.as_deref().map(|target| match target {
                rdgen_frontend::ResumeTarget::Label { label, .. } => {
                    ResumeTarget::Label(NamedReference {
                        name: identifier(label),
                        span: label.span(),
                    })
                }
                rdgen_frontend::ResumeTarget::Next { .. } => ResumeTarget::Next,
            });
            SemanticStatement {
                kind: SemanticStatementKind::Resume(target),
                span: *span,
            }
        }
        rdgen_frontend::StatementCore::OnErrorGoto { on_error, span } => {
            let rdgen_frontend::OnErrorGotoStmt::OnErrorGoto { target, .. } = on_error.as_ref();
            let target = match target.as_ref() {
                rdgen_frontend::ErrorTarget::Label { label, .. } => {
                    ErrorHandlerTarget::Label(NamedReference {
                        name: identifier(label),
                        span: label.span(),
                    })
                }
                rdgen_frontend::ErrorTarget::Disable { .. } => ErrorHandlerTarget::Disable,
            };
            SemanticStatement {
                kind: SemanticStatementKind::OnErrorGoto(target),
                span: *span,
            }
        }
        rdgen_frontend::StatementCore::OnBranch { on_branch, span } => {
            let rdgen_frontend::OnBranchStmt::OnBranch {
                selector,
                branch,
                first,
                rest,
                ..
            } = on_branch.as_ref();
            let branch = match branch.as_ref() {
                rdgen_frontend::BranchKind::Goto { .. } => BranchKind::Goto,
                rdgen_frontend::BranchKind::Gosub { .. } => BranchKind::Gosub,
            };
            let targets = std::iter::once(first.as_ref())
                .chain(rest.iter().map(|(_, value)| value.as_ref()))
                .map(|target| NamedReference {
                    name: identifier(target),
                    span: target.span(),
                })
                .collect();
            SemanticStatement {
                kind: SemanticStatementKind::OnBranch {
                    selector: adapt_expression(selector),
                    branch,
                    targets,
                },
                span: *span,
            }
        }
        rdgen_frontend::StatementCore::Error { error, span } => {
            let rdgen_frontend::ErrorStmt::Error { value, .. } = error.as_ref();
            SemanticStatement {
                kind: SemanticStatementKind::Error(adapt_expression(value)),
                span: *span,
            }
        }
        rdgen_frontend::StatementCore::Throw { throw, span } => {
            let rdgen_frontend::ThrowStmt::Throw { value, .. } = throw.as_ref();
            SemanticStatement {
                kind: SemanticStatementKind::Throw(
                    value
                        .as_deref()
                        .map(adapt_expression)
                        .map(ThrowValue::Value)
                        .unwrap_or(ThrowValue::Bare),
                ),
                span: *span,
            }
        }
        rdgen_frontend::StatementCore::OptionBase { option_base, span } => {
            let rdgen_frontend::OptionBaseStmt::OptionBase { value, .. } = option_base.as_ref();
            SemanticStatement {
                kind: SemanticStatementKind::OptionBase(adapt_expression(value)),
                span: *span,
            }
        }
        rdgen_frontend::StatementCore::Erase { erase, span } => {
            let rdgen_frontend::EraseStmt::Erase { first, rest, .. } = erase.as_ref();
            SemanticStatement {
                kind: SemanticStatementKind::Erase(
                    std::iter::once(first.as_ref())
                        .chain(rest.iter().map(|(_, value)| value.as_ref()))
                        .map(|value| NamedReference {
                            name: identifier(value),
                            span: value.span(),
                        })
                        .collect(),
                ),
                span: *span,
            }
        }
        rdgen_frontend::StatementCore::Print { print, span } => {
            let rdgen_frontend::PrintStmt::Print {
                destination,
                tokens,
                ..
            } = print.as_ref();
            SemanticStatement {
                kind: SemanticStatementKind::Print {
                    destination: adapt_print_destination(destination),
                    tokens: tokens
                        .iter()
                        .map(|value| adapt_print_token(value))
                        .collect(),
                },
                span: *span,
            }
        }
        rdgen_frontend::StatementCore::Input { input, span } => {
            let (source, first, rest) = match input.as_ref() {
                rdgen_frontend::InputStmt::ChannelInput {
                    channel,
                    first,
                    rest,
                    ..
                } => (InputSource::Channel(adapt_expression(channel)), first, rest),
                rdgen_frontend::InputStmt::ConsoleInput {
                    prompt,
                    first,
                    rest,
                    ..
                } => (
                    InputSource::Console(prompt.as_ref().map(|(value, _)| InputPrompt {
                        text: token_text_string(value),
                        span: value.span(),
                    })),
                    first,
                    rest,
                ),
            };
            SemanticStatement {
                kind: SemanticStatementKind::Input {
                    source,
                    targets: std::iter::once(adapt_expression(first))
                        .chain(rest.iter().map(|(_, value)| adapt_expression(value)))
                        .collect(),
                },
                span: *span,
            }
        }
        rdgen_frontend::StatementCore::Write { write, span } => {
            let rdgen_frontend::WriteStmt::Write {
                channel, values, ..
            } = write.as_ref();
            let values = values
                .as_ref()
                .map(|(_, first, rest)| {
                    WriteValues::Values(
                        std::iter::once(adapt_expression(first))
                            .chain(rest.iter().map(|(_, value)| adapt_expression(value)))
                            .collect(),
                    )
                })
                .unwrap_or(WriteValues::Omitted);
            SemanticStatement {
                kind: SemanticStatementKind::Write {
                    channel: adapt_expression(channel),
                    values,
                },
                span: *span,
            }
        }
        rdgen_frontend::StatementCore::Open { open, span } => {
            let rdgen_frontend::OpenStmt::Open {
                path,
                mode,
                channel,
                length,
                ..
            } = open.as_ref();
            SemanticStatement {
                kind: SemanticStatementKind::Open {
                    path: adapt_expression(path),
                    mode: adapt_open_mode(mode),
                    channel: adapt_expression(channel),
                    length: length.as_ref().map(|(_, _, value)| adapt_expression(value)),
                },
                span: *span,
            }
        }
        rdgen_frontend::StatementCore::Seek { seek, span } => {
            let rdgen_frontend::SeekStmt::Seek {
                channel, position, ..
            } = seek.as_ref();
            SemanticStatement {
                kind: SemanticStatementKind::Seek {
                    channel: adapt_expression(channel),
                    position: adapt_expression(position),
                },
                span: *span,
            }
        }
        rdgen_frontend::StatementCore::Kill { kill, span } => {
            let rdgen_frontend::KillStmt::Kill { path, .. } = kill.as_ref();
            SemanticStatement {
                kind: SemanticStatementKind::Kill(adapt_expression(path)),
                span: *span,
            }
        }
        rdgen_frontend::StatementCore::Rename { rename, span } => {
            let rdgen_frontend::NameStmt::Rename {
                source,
                destination,
                ..
            } = rename.as_ref();
            SemanticStatement {
                kind: SemanticStatementKind::Rename {
                    source: adapt_expression(source),
                    destination: adapt_expression(destination),
                },
                span: *span,
            }
        }
        rdgen_frontend::StatementCore::Close { close, span } => {
            let rdgen_frontend::CloseStmt::Close { channel, .. } = close.as_ref();
            SemanticStatement {
                kind: SemanticStatementKind::Close(adapt_expression(channel)),
                span: *span,
            }
        }
        rdgen_frontend::StatementCore::Data { data, span } => {
            let rdgen_frontend::DataStmt::Data { first, rest, .. } = data.as_ref();
            SemanticStatement {
                kind: SemanticStatementKind::Data(
                    std::iter::once(adapt_expression(first))
                        .chain(rest.iter().map(|(_, value)| adapt_expression(value)))
                        .collect(),
                ),
                span: *span,
            }
        }
        rdgen_frontend::StatementCore::Read { read, span } => {
            let rdgen_frontend::ReadStmt::Read { first, rest, .. } = read.as_ref();
            SemanticStatement {
                kind: SemanticStatementKind::Read(
                    std::iter::once(adapt_expression(first))
                        .chain(rest.iter().map(|(_, value)| adapt_expression(value)))
                        .collect(),
                ),
                span: *span,
            }
        }
        rdgen_frontend::StatementCore::Restore { restore, span } => {
            let rdgen_frontend::RestoreStmt::Restore { target, .. } = restore.as_ref();
            SemanticStatement {
                kind: SemanticStatementKind::Restore(target.as_deref().map(|target| {
                    NamedReference {
                        name: identifier(target),
                        span: target.span(),
                    }
                })),
                span: *span,
            }
        }
        rdgen_frontend::StatementCore::Dim { dim, span } => {
            let rdgen_frontend::DimStmt::Dim { first, rest, .. } = dim.as_ref();
            SemanticStatement {
                kind: SemanticStatementKind::Dim(
                    std::iter::once(first.as_ref())
                        .chain(rest.iter().map(|(_, value)| value.as_ref()))
                        .map(adapt_dim_item)
                        .collect(),
                ),
                span: *span,
            }
        }
        rdgen_frontend::StatementCore::Const { constant, span } => {
            let rdgen_frontend::ConstStmt::Const { name, value, .. } = constant.as_ref();
            SemanticStatement {
                kind: SemanticStatementKind::Const {
                    name: NamedReference {
                        name: identifier(name),
                        span: name.span(),
                    },
                    value: adapt_expression(value),
                    value_type: SemanticValueType::Unknown,
                },
                span: *span,
            }
        }
        rdgen_frontend::StatementCore::Global { global, span } => {
            let rdgen_frontend::GlobalDecl::GlobalDeclaration { name, .. } = global.as_ref();
            let name_text = identifier(name);
            let value_type = SemanticValueType::from_suffix(name_text.chars().last());
            SemanticStatement {
                kind: SemanticStatementKind::Global {
                    name: NamedReference {
                        name: name_text.clone(),
                        span: name.span(),
                    },
                    value_type: if value_type == SemanticValueType::Unknown {
                        SemanticValueType::Single
                    } else {
                        value_type
                    },
                },
                span: *span,
            }
        }
        rdgen_frontend::StatementCore::Swap { swap, span } => {
            let rdgen_frontend::SwapStmt::Swap { left, right, .. } = swap.as_ref();
            SemanticStatement {
                kind: SemanticStatementKind::Swap {
                    left: adapt_expression(left),
                    right: adapt_expression(right),
                },
                span: *span,
            }
        }
        rdgen_frontend::StatementCore::Randomize { randomize, span } => {
            let rdgen_frontend::RandomizeStmt::Randomize { seed, .. } = randomize.as_ref();
            SemanticStatement {
                kind: SemanticStatementKind::Randomize(
                    seed.as_deref()
                        .map(adapt_expression)
                        .map(RandomizeSeed::Value)
                        .unwrap_or(RandomizeSeed::Default),
                ),
                span: *span,
            }
        }
        rdgen_frontend::StatementCore::Poke { poke, span } => {
            let rdgen_frontend::PokeStmt::Poke { address, value, .. } = poke.as_ref();
            SemanticStatement {
                kind: SemanticStatementKind::Poke {
                    address: adapt_expression(address),
                    value: adapt_expression(value),
                },
                span: *span,
            }
        }
        rdgen_frontend::StatementCore::Out { out, span } => {
            let rdgen_frontend::OutStmt::Out { port, value, .. } = out.as_ref();
            SemanticStatement {
                kind: SemanticStatementKind::Out {
                    port: adapt_expression(port),
                    value: adapt_expression(value),
                },
                span: *span,
            }
        }
        rdgen_frontend::StatementCore::Width { width, span } => {
            let rdgen_frontend::WidthStmt::Width { channel, value, .. } = width.as_ref();
            SemanticStatement {
                kind: SemanticStatementKind::Width {
                    channel: channel
                        .as_ref()
                        .map(|(_, value, _)| adapt_expression(value)),
                    value: adapt_expression(value),
                },
                span: *span,
            }
        }
        rdgen_frontend::StatementCore::LineInput { line_input, span } => {
            let rdgen_frontend::LineInputStmt::LineInput {
                channel, target, ..
            } = line_input.as_ref();
            SemanticStatement {
                kind: SemanticStatementKind::LineInput {
                    channel: adapt_expression(channel),
                    target: adapt_expression(target),
                },
                span: *span,
            }
        }
        rdgen_frontend::StatementCore::Get { get, span } => {
            let rdgen_frontend::GetStmt::Get {
                channel, position, ..
            } = get.as_ref();
            SemanticStatement {
                kind: SemanticStatementKind::Get {
                    channel: adapt_expression(channel),
                    position: position.as_ref().map(adapt_file_position),
                },
                span: *span,
            }
        }
        rdgen_frontend::StatementCore::Put { put, span } => {
            let rdgen_frontend::PutStmt::Put {
                channel, position, ..
            } = put.as_ref();
            SemanticStatement {
                kind: SemanticStatementKind::Put {
                    channel: adapt_expression(channel),
                    position: position.as_ref().map(adapt_file_position),
                },
                span: *span,
            }
        }
        rdgen_frontend::StatementCore::Lset { lset, span } => {
            let rdgen_frontend::LsetStmt::Lset { target, value, .. } = lset.as_ref();
            SemanticStatement {
                kind: SemanticStatementKind::Lset {
                    target: NamedReference {
                        name: identifier(target),
                        span: target.span(),
                    },
                    value: adapt_expression(value),
                },
                span: *span,
            }
        }
        rdgen_frontend::StatementCore::Rset { rset, span } => {
            let rdgen_frontend::RsetStmt::Rset { target, value, .. } = rset.as_ref();
            SemanticStatement {
                kind: SemanticStatementKind::Rset {
                    target: NamedReference {
                        name: identifier(target),
                        span: target.span(),
                    },
                    value: adapt_expression(value),
                },
                span: *span,
            }
        }
        rdgen_frontend::StatementCore::Locate { locate, span } => {
            let rdgen_frontend::LocateStmt::Locate { row, column, .. } = locate.as_ref();
            SemanticStatement {
                kind: SemanticStatementKind::Locate {
                    row: adapt_expression(row),
                    column: adapt_expression(column),
                },
                span: *span,
            }
        }
        rdgen_frontend::StatementCore::Color { color, span } => {
            let rdgen_frontend::ColorStmt::Color {
                foreground,
                background,
                ..
            } = color.as_ref();
            SemanticStatement {
                kind: SemanticStatementKind::Color {
                    foreground: adapt_expression(foreground),
                    background: background
                        .as_ref()
                        .map(|(_, value)| adapt_expression(value)),
                },
                span: *span,
            }
        }
        rdgen_frontend::StatementCore::Stop { span, .. } => SemanticStatement {
            kind: SemanticStatementKind::Stop,
            span: *span,
        },
        rdgen_frontend::StatementCore::Clear { span, .. } => SemanticStatement {
            kind: SemanticStatementKind::Clear,
            span: *span,
        },
        rdgen_frontend::StatementCore::Cls { span, .. } => SemanticStatement {
            kind: SemanticStatementKind::Cls,
            span: *span,
        },
        rdgen_frontend::StatementCore::Beep { span, .. } => SemanticStatement {
            kind: SemanticStatementKind::Beep,
            span: *span,
        },
        rdgen_frontend::StatementCore::System { span, .. } => SemanticStatement {
            kind: SemanticStatementKind::System,
            span: *span,
        },
        rdgen_frontend::StatementCore::End { span, .. } => SemanticStatement {
            kind: SemanticStatementKind::End,
            span: *span,
        },
        rdgen_frontend::StatementCore::Lprint { lprint, span } => {
            let rdgen_frontend::LprintStmt::Lprint { using, tokens, .. } = lprint.as_ref();
            SemanticStatement {
                kind: SemanticStatementKind::Lprint {
                    using: using.as_ref().map(|(_, value, _)| adapt_expression(value)),
                    tokens: tokens
                        .iter()
                        .map(|value| adapt_print_token(value))
                        .collect(),
                },
                span: *span,
            }
        }
        rdgen_frontend::StatementCore::FileDeclaration {
            file_declaration,
            span,
        } => {
            let rdgen_frontend::FileDeclStmt::FileDeclaration {
                name,
                record_type,
                path,
                mode,
                ..
            } = file_declaration.as_ref();
            SemanticStatement {
                kind: SemanticStatementKind::FileDeclaration {
                    name: NamedReference {
                        name: identifier(name),
                        span: name.span(),
                    },
                    record_type: record_type.as_ref().map(|(_, value)| NamedReference {
                        name: identifier(value),
                        span: value.span(),
                    }),
                    path: adapt_expression(path),
                    mode: mode.as_ref().map(|(_, value)| adapt_file_mode(value)),
                },
                span: *span,
            }
        }
        rdgen_frontend::StatementCore::Field { field, span } => {
            let rdgen_frontend::FieldStmt::Field {
                channel,
                first,
                rest,
                ..
            } = field.as_ref();
            SemanticStatement {
                kind: SemanticStatementKind::Field {
                    channel: adapt_expression(channel),
                    bindings: std::iter::once(first.as_ref())
                        .chain(rest.iter().map(|(_, value)| value.as_ref()))
                        .map(adapt_field_binding)
                        .collect(),
                },
                span: *span,
            }
        }
    }
}

fn adapt_print_destination(destination: &rdgen_frontend::PrintDestination) -> PrintDestination {
    match destination {
        rdgen_frontend::PrintDestination::Standard { span } => {
            PrintDestination::Standard { span: *span }
        }
        rdgen_frontend::PrintDestination::Using { format, span } => PrintDestination::Using {
            format: adapt_expression(format),
            span: *span,
        },
        rdgen_frontend::PrintDestination::Channel {
            channel,
            using,
            span,
        } => PrintDestination::Channel {
            channel: adapt_expression(channel),
            using: using.as_ref().map(|(_, value, _)| adapt_expression(value)),
            span: *span,
        },
    }
}
fn adapt_print_token(token: &rdgen_frontend::PrintToken) -> PrintToken {
    match token {
        rdgen_frontend::PrintToken::Comma { span } => PrintToken::Comma { span: *span },
        rdgen_frontend::PrintToken::Semicolon { span } => PrintToken::Semicolon { span: *span },
        rdgen_frontend::PrintToken::Expression { value, .. } => {
            PrintToken::Expression(adapt_expression(value))
        }
    }
}
fn adapt_open_mode(mode: &rdgen_frontend::OpenMode) -> OpenMode {
    let kind = match mode {
        rdgen_frontend::OpenMode::Input { .. } => OpenModeKind::Input,
        rdgen_frontend::OpenMode::Output { .. } => OpenModeKind::Output,
        rdgen_frontend::OpenMode::Append { .. } => OpenModeKind::Append,
        rdgen_frontend::OpenMode::Random { .. } => OpenModeKind::Random,
        rdgen_frontend::OpenMode::Binary { .. } => OpenModeKind::Binary,
    };
    OpenMode {
        kind,
        span: mode.span(),
    }
}
fn adapt_file_mode(mode: &rdgen_frontend::FileMode) -> FileModeKind {
    match mode {
        rdgen_frontend::FileMode::Input { .. } => FileModeKind::Input,
        rdgen_frontend::FileMode::Output { .. } => FileModeKind::Output,
        rdgen_frontend::FileMode::Append { .. } => FileModeKind::Append,
    }
}
fn adapt_field_binding(binding: &rdgen_frontend::FieldBinding) -> FieldBinding {
    let rdgen_frontend::FieldBinding::Binding { length, name, .. } = binding;
    let name_span = name.span();
    let name = identifier(name);
    let type_suffix = name
        .chars()
        .last()
        .filter(|suffix| matches!(suffix, '$' | '%' | '&' | '!' | '#'))
        .map(|suffix| suffix.to_string());
    let is_string = type_suffix.as_deref() == Some("$");
    FieldBinding {
        length: adapt_expression(length),
        name,
        name_span,
        type_suffix,
        is_string,
    }
}
fn adapt_array_axes(axes: &rdgen_frontend::ArrayAxes) -> Vec<DimAxis> {
    match axes {
        rdgen_frontend::ArrayAxes::ArrayAxes { first, rest, .. } => {
            std::iter::once(first.as_ref())
                .chain(rest.iter().map(|(_, axis)| axis.as_ref()))
                .map(|axis| match axis {
                    rdgen_frontend::Axis::InferredCapacity { .. } => DimAxis::Inferred,
                    rdgen_frontend::Axis::ExpressionCapacity { capacity, .. } => {
                        let expression = adapt_expression(capacity);
                        match &expression.kind {
                            ExpressionKind::Literal(value) => DimAxis::Fixed(value.clone()),
                            _ => DimAxis::Expression(expression),
                        }
                    }
                })
                .collect()
        }
        rdgen_frontend::ArrayAxes::EmptyArrayAxes { .. } => vec![DimAxis::Inferred],
    }
}

fn adapt_dim_item(item: &rdgen_frontend::DimItem) -> DimDeclaration {
    let rdgen_frontend::DimItem::DimItem {
        name,
        axes,
        type_annotation,
        ..
    } = item;
    let dimensions = axes.as_deref().map(adapt_array_axes).unwrap_or_default();
    let name = typed_identifier(name);
    let type_annotation = type_annotation.as_ref().map(|(_, value)| identifier(value));
    let suffix = name
        .chars()
        .last()
        .filter(|ch| matches!(ch, '$' | '%' | '&' | '!' | '#'));
    let suffix_type = SemanticValueType::from_suffix(suffix);
    let element_type = if suffix_type != SemanticValueType::Unknown {
        suffix_type
    } else {
        type_annotation
            .as_deref()
            .map(semantic_type_from_annotation)
            .unwrap_or(SemanticValueType::Unknown)
    };
    DimDeclaration {
        name,
        array_axes: dimensions.len(),
        dimensions,
        type_annotation,
        element_type,
        span: item.span(),
    }
}
fn adapt_file_position(
    position: &(
        rdgen_frontend::Token,
        Option<Box<rdgen_frontend::Expr>>,
        Option<(rdgen_frontend::Token, Box<rdgen_frontend::Expr>)>,
    ),
) -> FilePosition {
    FilePosition {
        position: position.1.as_deref().map(adapt_expression),
        record: position
            .2
            .as_ref()
            .map(|(_, value)| adapt_expression(value)),
    }
}

fn adapt_for_bounds(bounds: &rdgen_frontend::ForBounds) -> ForBounds {
    match bounds {
        rdgen_frontend::ForBounds::To { limit, step, .. } => ForBounds::To {
            limit: adapt_expression(limit),
            step: step.as_ref().map(|(_, value)| adapt_expression(value)),
        },
        rdgen_frontend::ForBounds::Downto { limit, step, .. } => ForBounds::Downto {
            limit: adapt_expression(limit),
            step: *step,
        },
    }
}
fn adapt_loop_condition(condition: &rdgen_frontend::DoCondition) -> LoopCondition {
    let rdgen_frontend::DoCondition::Condition { kind, value, .. } = condition;
    LoopCondition {
        kind: if matches!(kind.as_ref(), rdgen_frontend::DoConditionKind::Until { .. }) {
            LoopConditionKind::Until
        } else {
            LoopConditionKind::While
        },
        value: adapt_expression(value),
    }
}
fn adapt_case_clause(clause: &rdgen_frontend::CaseClause) -> CaseClause {
    let rdgen_frontend::CaseClause::Case { values, body, span } = clause else {
        unreachable!("CASE ELSE is adapted into SelectCase::else_body")
    };
    let rdgen_frontend::CaseValues::CaseValues { first, rest, .. } = values.as_ref();
    CaseClause {
        values: std::iter::once(first.as_ref())
            .chain(rest.iter().map(|(_, value)| value.as_ref()))
            .map(adapt_case_value)
            .collect(),
        body: body.iter().map(|value| adapt_statement(value)).collect(),
        span: *span,
    }
}
fn adapt_case_value(value: &rdgen_frontend::CaseValue) -> CaseValue {
    match value {
        rdgen_frontend::CaseValue::Comparison {
            operator,
            value,
            span,
        } => CaseValue::Comparison {
            operator: comparison_operator(operator),
            value: adapt_expression(value),
            span: *span,
        },
        rdgen_frontend::CaseValue::ValueOrRange {
            first,
            range_end,
            span,
        } => CaseValue::Value {
            first: adapt_expression(first),
            range_end: range_end.as_ref().map(|(_, value)| adapt_expression(value)),
            span: *span,
        },
    }
}
fn comparison_operator(operator: &rdgen_frontend::CompareOp) -> ComparisonOperator {
    match operator {
        rdgen_frontend::CompareOp::NotEqual { .. } => ComparisonOperator::NotEqual,
        rdgen_frontend::CompareOp::LessOrEqual { .. } => ComparisonOperator::LessOrEqual,
        rdgen_frontend::CompareOp::GreaterOrEqual { .. } => ComparisonOperator::GreaterOrEqual,
        rdgen_frontend::CompareOp::Equal { .. } => ComparisonOperator::Equal,
        rdgen_frontend::CompareOp::Less { .. } => ComparisonOperator::Less,
        rdgen_frontend::CompareOp::Greater { .. } => ComparisonOperator::Greater,
    }
}
fn adapt_catch(value: &rdgen_frontend::CatchClause) -> CatchBinding {
    let rdgen_frontend::CatchClause::Catch {
        error,
        filters,
        line,
        source,
        body,
        ..
    } = value;
    let error_span = error.span();
    let line_span = line.span();
    let error = identifier(error);
    let line = identifier(line);
    let error_type = SemanticValueType::from_suffix(error.chars().last());
    let line_type = SemanticValueType::from_suffix(line.chars().last());
    CatchBinding {
        error_type: if error_type == SemanticValueType::Unknown {
            SemanticValueType::Single
        } else {
            error_type
        },
        error,
        error_span,
        line_type: if line_type == SemanticValueType::Unknown {
            SemanticValueType::Single
        } else {
            line_type
        },
        line,
        line_span,
        filters: filters
            .as_ref()
            .map(|(_, first, rest, _)| {
                std::iter::once(first.as_ref())
                    .chain(rest.iter().map(|(_, value)| value.as_ref()))
                    .map(adapt_expression)
                    .collect()
            })
            .unwrap_or_default(),
        source: source.as_ref().map(|(_, value)| identifier(value)),
        source_type: source
            .as_ref()
            .map(|_| SemanticValueType::String),
        source_span: source.as_ref().map(|(_, value)| value.span()),
        body: body.iter().map(|value| adapt_statement(value)).collect(),
    }
}

fn adapt_if(value: &rdgen_frontend::IfStmt, span: SourceSpan) -> SemanticStatement {
    let rdgen_frontend::IfStmt::If {
        condition, tail, ..
    } = value;
    let (then_body, else_body, block) = match tail.as_ref() {
        rdgen_frontend::IfTail::SingleLine { tail, .. } => {
            let rdgen_frontend::SingleLineIfTail::SingleLineIf {
                first,
                rest,
                else_clause,
                ..
            } = tail.as_ref();
            let then_body = std::iter::once(first.as_ref())
                .chain(rest.iter().map(Box::as_ref))
                .map(adapt_statement)
                .collect();
            let else_body = else_clause
                .as_ref()
                .map(|(_, first, rest)| {
                    std::iter::once(first.as_ref())
                        .chain(rest.iter().map(Box::as_ref))
                        .map(adapt_statement)
                        .collect()
                })
                .unwrap_or_default();
            (then_body, else_body, false)
        }
        rdgen_frontend::IfTail::Block { tail, .. } => adapt_block_if(tail),
    };
    SemanticStatement {
        kind: SemanticStatementKind::If {
            condition: adapt_expression(condition),
            then_body,
            else_body,
            block,
        },
        span,
    }
}

fn adapt_block_if(
    tail: &rdgen_frontend::BlockIfTail,
) -> (Vec<SemanticStatement>, Vec<SemanticStatement>, bool) {
    let rdgen_frontend::BlockIfTail::BlockIf {
        then_body,
        continuation,
        ..
    } = tail;
    let then_body = then_body
        .iter()
        .map(|value| adapt_statement(value))
        .collect();
    let else_body = match continuation.as_ref() {
        rdgen_frontend::IfBlockContinuation::End { else_body, .. } => else_body
            .as_ref()
            .map(|(_, body)| body.iter().map(|value| adapt_statement(value)).collect())
            .unwrap_or_default(),
        rdgen_frontend::IfBlockContinuation::ElseIf {
            condition,
            tail,
            span,
        } => vec![SemanticStatement {
            kind: SemanticStatementKind::If {
                condition: adapt_expression(condition),
                then_body: adapt_block_if(tail).0,
                else_body: adapt_block_if(tail).1,
                block: true,
            },
            span: *span,
        }],
    };
    (then_body, else_body, true)
}

fn assignment_operator(operator: &rdgen_frontend::AssignmentOp) -> AssignmentOperator {
    let rdgen_frontend::AssignmentOp::Token { text, .. } = operator;
    match text.0.as_str() {
        "=" => AssignmentOperator::Assign,
        "+=" => AssignmentOperator::Add,
        "-=" => AssignmentOperator::Subtract,
        "*=" => AssignmentOperator::Multiply,
        "/=" => AssignmentOperator::Divide,
        _ => unreachable!("generated grammar only emits known assignment operators"),
    }
}

fn adapt_subprogram(value: &rdgen_frontend::Subprogram) -> CallableSignature {
    use rdgen_frontend::Subprogram;
    match value {
        Subprogram::Function { function, .. } => {
            let rdgen_frontend::FunctionDecl::FunctionDeclaration {
                name,
                parameters,
                body,
                span,
            } = function.as_ref();
            let name_text = typed_identifier(name);
            CallableSignature {
                kind: CallableKind::Function,
                name: name_text.clone(),
                name_span: name.span(),
                result_type: suffix_from_name(&name_text).or_else(|| Some("!".to_string())),
                receiver: None,
                receiver_span: None,
                parameters: adapt_parameters(parameters),
                body: body.iter().map(|value| adapt_statement(value)).collect(),
                span: *span,
                source_index: usize::MAX,
            }
        }
        Subprogram::Procedure { procedure, .. } => {
            let rdgen_frontend::ProcedureDecl::ProcedureDeclaration {
                name,
                parameters,
                body,
                span,
            } = procedure.as_ref();
            CallableSignature {
                kind: CallableKind::Procedure,
                name: identifier(name),
                name_span: name.span(),
                result_type: None,
                receiver: None,
                receiver_span: None,
                parameters: adapt_parameters(parameters),
                body: body.iter().map(|value| adapt_statement(value)).collect(),
                span: *span,
                source_index: usize::MAX,
            }
        }
        Subprogram::Method { method, .. } => adapt_method(method, CallableKind::Method),
        Subprogram::FluentMethod { method, .. } => {
            let rdgen_frontend::FluentMethodDecl::FluentMethod { method, .. } = method.as_ref();
            adapt_method(method, CallableKind::FluentMethod)
        }
    }
}

fn adapt_method(method: &rdgen_frontend::MethodDecl, kind: CallableKind) -> CallableSignature {
    let rdgen_frontend::MethodDecl::MethodDeclaration {
        name,
        receiver,
        parameters,
        result,
        body,
        span,
        ..
    } = method;
    let name_span = name.span();
    let name = identifier(name);
    let receiver_span = receiver.span();
    let receiver = identifier(receiver);
    // A scalar method with no explicit result has its receiver's type,
    // matching the legacy parser's `finish_scalar_method`.
    let result_type = result
        .as_ref()
        .map(|(_, value)| adapt_return_type(value))
        .or_else(|| suffix_from_name(&name))
        .or_else(|| scalar_receiver_suffix(&receiver));
    let mut body: Vec<SemanticStatement> = body.iter().map(|value| adapt_statement(value)).collect();
    // A scalar method with neither an explicit result nor a suffixed name
    // returns its receiver when it falls off the end, as the legacy parser
    // makes explicit with a trailing `return self`.
    let implicit_self_result = result.is_none() && suffix_from_name(&name).is_none();
    if let (true, Some(suffix)) = (implicit_self_result, scalar_receiver_suffix(&receiver)) {
        let ends_with_return = body.last().is_some_and(|last| match &last.kind {
            SemanticStatementKind::Line(children) => children
                .last()
                .is_some_and(|child| matches!(child.kind, SemanticStatementKind::Return(_))),
            kind => matches!(kind, SemanticStatementKind::Return(_)),
        });
        if !ends_with_return {
            let position = SourceSpan {
                start: span.start,
                end: span.start,
            };
            body.push(SemanticStatement {
                kind: SemanticStatementKind::Return(ReturnValue::Value(Expression {
                    kind: ExpressionKind::Name(format!("self{suffix}")),
                    span: position,
                    value_type: SemanticValueType::from_suffix(suffix.chars().next()),
                    record_type: None,
                })),
                span: position,
            });
        }
    }
    CallableSignature {
        kind,
        name,
        name_span,
        result_type,
        receiver: Some(receiver),
        receiver_span: Some(receiver_span),
        parameters: adapt_parameters(parameters),
        body,
        span: *span,
        source_index: usize::MAX,
    }
}

fn adapt_inline_methods(
    record: &rdgen_frontend::RecordDecl,
    receiver: &str,
) -> Vec<CallableSignature> {
    let rdgen_frontend::RecordDecl::RecordDeclaration { members, .. } = record;
    members
        .iter()
        .filter_map(|member| match member.as_ref() {
            rdgen_frontend::RecordMember::InlineMethod { method, .. } => {
                let rdgen_frontend::InlineMethod::InlineMethodDeclaration {
                    name,
                    parameters,
                    result,
                    body,
                    span,
                    ..
                } = method.as_ref();
                Some(CallableSignature {
                    kind: CallableKind::InlineMethod,
                    name: identifier(name),
                    name_span: name.span(),
                    result_type: result.as_ref().map(|(_, value)| adapt_return_type(value)),
                    receiver: Some(receiver.into()),
                    receiver_span: None,
                    parameters: adapt_parameters(parameters),
                    body: body.iter().map(|value| adapt_statement(value)).collect(),
                    span: *span,
                    source_index: usize::MAX,
                })
            }
            _ => None,
        })
        .collect()
}

fn adapt_parameters(parameters: &Option<Box<rdgen_frontend::ParamList>>) -> Vec<Parameter> {
    let Some(parameters) = parameters else {
        return Vec::new();
    };
    let rdgen_frontend::ParamList::ParameterList { first, rest, .. } = parameters.as_ref();
    std::iter::once(first.as_ref())
        .chain(rest.iter().map(|(_, parameter)| parameter.as_ref()))
        .map(adapt_parameter)
        .collect()
}

fn adapt_parameter(parameter: &rdgen_frontend::Param) -> Parameter {
    let rdgen_frontend::Param::Parameter {
        mode,
        name,
        axes,
        default,
        type_annotation,
        span,
        ..
    } = parameter;
    let passing = mode.as_ref().map(|mode| match mode.as_ref() {
        rdgen_frontend::PassingMode::ByRef { .. } => Passing::ByRef,
        rdgen_frontend::PassingMode::ByVal { .. } => Passing::ByVal,
    });
    let dimensions = axes
        .as_deref()
        .map(adapt_array_axes)
        .unwrap_or_default();
    let array_axes = dimensions.len();
    let typed_name = typed_identifier(name);
    let type_suffix = suffix_from_name(&typed_name);
    let type_annotation = type_annotation
        .as_ref()
        .map(|(_, value)| identifier(value));
    let suffix_type = type_suffix
        .as_deref()
        .and_then(|suffix| suffix.chars().next())
        .map(|suffix| SemanticValueType::from_suffix(Some(suffix)))
        .unwrap_or(SemanticValueType::Unknown);
    let value_type = if suffix_type != SemanticValueType::Unknown {
        suffix_type
    } else if let Some(annotation) = type_annotation.as_deref() {
        semantic_type_from_annotation(annotation)
    } else {
        SemanticValueType::Single
    };
    Parameter {
        name: typed_name,
        name_span: name.span(),
        value_type,
        type_suffix,
        type_annotation,
        passing,
        array_axes,
        dimensions,
        default: default.as_ref().map(|(_, value)| adapt_expression(value)),
        span: *span,
    }
}

fn suffix_from_name(name: &str) -> Option<String> {
    name.chars()
        .last()
        .filter(|value| matches!(value, '%' | '&' | '!' | '#' | '$' | '@'))
        .map(|value| value.to_string())
}
fn scalar_builtin_receiver(value_type: SemanticValueType) -> Option<crate::ast::TypeSuffix> {
    use crate::ast::TypeSuffix;
    match value_type {
        SemanticValueType::String => Some(TypeSuffix::String),
        SemanticValueType::Integer => Some(TypeSuffix::Integer),
        SemanticValueType::Long => Some(TypeSuffix::Long),
        SemanticValueType::Single => Some(TypeSuffix::Single),
        SemanticValueType::Double => Some(TypeSuffix::Double),
        SemanticValueType::Unknown | SemanticValueType::Boolean => None,
    }
}

fn scalar_receiver_suffix(receiver: &str) -> Option<String> {
    match receiver.to_ascii_lowercase().as_str() {
        "integer" => Some("%".to_string()),
        "long" => Some("&".to_string()),
        "single" => Some("!".to_string()),
        "double" => Some("#".to_string()),
        "string" => Some("$".to_string()),
        _ => None,
    }
}

fn adapt_return_type(value: &rdgen_frontend::ReturnType) -> String {
    match value {
        rdgen_frontend::ReturnType::Integer { .. } => "%",
        rdgen_frontend::ReturnType::Long { .. } => "&",
        rdgen_frontend::ReturnType::Single { .. } => "!",
        rdgen_frontend::ReturnType::Double { .. } => "#",
        rdgen_frontend::ReturnType::String { .. } => "$",
        rdgen_frontend::ReturnType::Suffix { value, .. } => {
            let rdgen_frontend::Suffix::Token { text, .. } = value.as_ref();
            text.0.as_str()
        }
    }
    .into()
}

fn adapt_record(record: &rdgen_frontend::RecordDecl) -> Record {
    let rdgen_frontend::RecordDecl::RecordDeclaration {
        name,
        combines,
        members,
        span,
    } = record;
    let combines = combines
        .as_ref()
        .map(|(_, list)| {
            let rdgen_frontend::CombinedRecordList::CombinedRecordList { first, rest, .. } =
                list.as_ref();
            std::iter::once(identifier(first))
                .chain(rest.iter().map(|(_, name)| identifier(name)))
                .collect()
        })
        .unwrap_or_default();
    let fields = members
        .iter()
        .filter_map(|member| match member.as_ref() {
            rdgen_frontend::RecordMember::Field { field, .. } => Some(adapt_record_field(field)),
            rdgen_frontend::RecordMember::InlineMethod { .. } => None,
        })
        .collect();
    Record {
        name: identifier(name),
        name_span: name.span(),
        combines,
        fields,
        span: *span,
    }
}

fn adapt_record_field(field: &rdgen_frontend::FieldDecl) -> RecordField {
    let rdgen_frontend::FieldDecl::FieldDeclaration {
        name,
        field_type,
        span,
    } = field;
    RecordField {
        name: identifier(name),
        name_span: name.span(),
        field_type: adapt_field_type(field_type),
        span: *span,
    }
}

fn adapt_field_type(field_type: &rdgen_frontend::FieldType) -> RecordFieldType {
    use rdgen_frontend::FieldType;
    match field_type {
        FieldType::StringType {
            capacity,
            alignment,
            span,
        } => RecordFieldType::String {
            capacity: capacity
                .as_ref()
                .map(|(_, value, _)| integer_literal(value)),
            alignment: alignment.as_ref().map(|value| match value.as_ref() {
                rdgen_frontend::StringAlign::LeftPad { .. } => StringAlignment::LeftPad,
                rdgen_frontend::StringAlign::Left { .. } => StringAlignment::Left,
                rdgen_frontend::StringAlign::RightPad { .. } => StringAlignment::RightPad,
                rdgen_frontend::StringAlign::Right { .. } => StringAlignment::Right,
            }),
            span: *span,
        },
        FieldType::Int16Type { span } => RecordFieldType::Int16 { span: *span },
        FieldType::IntType { span } => RecordFieldType::Int { span: *span },
        FieldType::Int32Type { span } => RecordFieldType::Int32 { span: *span },
        FieldType::Float32Type { span } => RecordFieldType::Float32 { span: *span },
        FieldType::Float64Type { span } => RecordFieldType::Float64 { span: *span },
        FieldType::RecordType { record, span } => RecordFieldType::Record {
            name: identifier(record),
            span: *span,
        },
    }
}

fn identifier(identifier: &rdgen_frontend::Identifier) -> String {
    let rdgen_frontend::Identifier::Token { text, .. } = identifier;
    text.0.clone()
}

fn typed_identifier(typed: &rdgen_frontend::TypedIdent) -> String {
    let rdgen_frontend::TypedIdent::TypedIdentifier { value, .. } = typed;
    identifier(value)
}

fn integer_literal(literal: &rdgen_frontend::IntegerLiteral) -> String {
    let rdgen_frontend::IntegerLiteral::Token { text, .. } = literal;
    text.0.clone()
}

pub fn parse_integer_value(value: &str) -> Option<i64> {
    let value = value.trim();
    if let Some(hex) = value
        .strip_prefix("&H")
        .or_else(|| value.strip_prefix("&h"))
    {
        i64::from_str_radix(hex, 16).ok()
    } else if let Some(octal) = value
        .strip_prefix("&O")
        .or_else(|| value.strip_prefix("&o"))
    {
        i64::from_str_radix(octal, 8).ok()
    } else {
        value.parse().ok()
    }
}

pub fn adapt_expression(expression: &rdgen_frontend::Expr) -> Expression {
    use rdgen_frontend::Expr;
    let span = expression.span();
    let kind = match expression {
        Expr::Name { name, .. } => ExpressionKind::Name(identifier(name)),
        Expr::FloatLiteral { value, .. } => ExpressionKind::Literal(float_literal(value)),
        Expr::IntegerLiteral { value, .. } => ExpressionKind::Literal(integer_literal(value)),
        Expr::HexLiteral { value, .. } => ExpressionKind::Literal(token_text_hex(value)),
        Expr::OctalLiteral { value, .. } => ExpressionKind::Literal(token_text_octal(value)),
        Expr::StringLiteral { value, .. } => ExpressionKind::Literal(token_text_string(value)),
        Expr::True { .. } => ExpressionKind::Boolean(true),
        Expr::False { .. } => ExpressionKind::Boolean(false),
        Expr::Parenthesized { value, .. } => {
            ExpressionKind::Parenthesized(Box::new(adapt_expression(value)))
        }
        Expr::Unary {
            operator, operand, ..
        } => ExpressionKind::Unary {
            operator: operator.0.clone(),
            operand: Box::new(adapt_expression(operand)),
        },
        Expr::Binary {
            left,
            operator,
            right,
            ..
        } => ExpressionKind::Binary {
            left: Box::new(adapt_expression(left)),
            operator: operator.0.clone(),
            right: Box::new(adapt_expression(right)),
        },
        Expr::Call {
            name, arguments, ..
        } => ExpressionKind::Call {
            name: identifier(name),
            arguments: adapt_arguments(arguments.as_deref()),
        },
        Expr::Index { name, index, .. } => ExpressionKind::Index {
            name: identifier(name),
            index: Box::new(adapt_expression(index)),
        },
        Expr::Member {
            base,
            member,
            arguments,
            ..
        } => ExpressionKind::Member {
            base: base.as_ref().map(|value| Box::new(adapt_expression(value))),
            member: identifier(member),
            arguments: arguments
                .as_ref()
                .map(|(_, args, _)| adapt_arguments(args.as_deref())),
        },
        Expr::RecordLiteral { fields, .. } => {
            ExpressionKind::RecordLiteral(adapt_record_literal_fields(fields))
        }
        Expr::PartialRecordLiteral { fields, .. } => {
            ExpressionKind::PartialRecordLiteral(adapt_record_literal_fields(&fields.2))
        }
    };
    let value_type = match &kind {
        ExpressionKind::Name(name) => SemanticValueType::from_suffix(name.chars().last()),
        ExpressionKind::Literal(value) => {
            if value.starts_with('"') {
                SemanticValueType::String
            } else if value.contains('.') || value.contains('e') || value.contains('E') {
                SemanticValueType::Double
            } else {
                SemanticValueType::Integer
            }
        }
        ExpressionKind::Boolean(_) => SemanticValueType::Boolean,
        ExpressionKind::Parenthesized(inner) | ExpressionKind::Unary { operand: inner, .. } => {
            inner.value_type
        }
        ExpressionKind::Binary { left, .. } => left.value_type,
        ExpressionKind::Call { .. }
        | ExpressionKind::Index { .. }
        | ExpressionKind::MultiIndex { .. }
        | ExpressionKind::Member { .. }
        | ExpressionKind::RecordLiteral(_)
        | ExpressionKind::PartialRecordLiteral(_) => SemanticValueType::Unknown,
    };
    Expression {
        kind,
        span,
        value_type,
        record_type: None,
    }
}
fn adapt_record_literal_fields(fields: &rdgen_frontend::RecordLitFields) -> Vec<FieldInitializer> {
    let rdgen_frontend::RecordLitFields::RecordFields { fields, .. } = fields;
    fields
        .as_ref()
        .map(|(first, rest)| {
            std::iter::once(first.as_ref())
                .chain(rest.iter().map(|(_, value)| value.as_ref()))
                .map(|field| {
                    let rdgen_frontend::FieldInit::Field { name, value, .. } = field;
                    FieldInitializer {
                        name: identifier(name),
                        value: adapt_expression(value),
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

fn adapt_arguments(arguments: Option<&rdgen_frontend::ArgList>) -> Vec<Expression> {
    let Some(arguments) = arguments else {
        return Vec::new();
    };
    let rdgen_frontend::ArgList::Arguments { first, rest, .. } = arguments;
    std::iter::once(first.as_ref())
        .chain(rest.iter().map(|(_, value)| value.as_ref()))
        .map(adapt_expression)
        .collect()
}
fn float_literal(value: &rdgen_frontend::FloatLiteral) -> String {
    let rdgen_frontend::FloatLiteral::Token { text, .. } = value;
    text.0.clone()
}
fn token_text_hex(value: &rdgen_frontend::HexLiteral) -> String {
    let rdgen_frontend::HexLiteral::Token { text, .. } = value;
    text.0.clone()
}
fn token_text_octal(value: &rdgen_frontend::OctalLiteral) -> String {
    let rdgen_frontend::OctalLiteral::Token { text, .. } = value;
    text.0.clone()
}
fn token_text_string(value: &rdgen_frontend::StringLiteral) -> String {
    let rdgen_frontend::StringLiteral::Token { text, .. } = value;
    text.0.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_source_position_uses_span_and_skips_leading_trivia() {
        let source = SemanticSource {
            filename: "typed_source.bcl".to_string(),
            text: "é\n  value%\n".to_string(),
        };
        let position = source.source_position(SourceSpan { start: 2, end: 10 });

        assert_eq!(position.filename, "typed_source.bcl");
        assert_eq!((position.line, position.column), (2, 3));
    }

    #[test]
    fn semantic_source_position_handles_non_boundary_offsets() {
        let source = SemanticSource {
            filename: "typed_source.bcl".to_string(),
            text: "évalue%".to_string(),
        };
        let position = source.source_position(SourceSpan { start: 1, end: 2 });

        assert_eq!((position.line, position.column), (1, 1));
    }

    #[test]
    fn semantic_source_alignment_rejects_non_boundary_offsets() {
        let source = SemanticSource {
            filename: "typed_source.bcl".to_string(),
            text: "évalue%".to_string(),
        };

        assert!(source.source_position_at(1).is_none());
    }

    #[test]
    fn annotates_array_element_types_from_dim_metadata() {
        let module = parse_and_adapt(
            "dim counts%(10)\ndim names(10) as string\ndim amounts(10)\nprint counts%(2), names(3), amounts(4)\n",
        )
        .unwrap();
        fn find_print<'a>(statements: &'a [SemanticStatement]) -> Option<&'a [PrintToken]> {
            for statement in statements {
                match &statement.kind {
                    SemanticStatementKind::Print { tokens, .. } => return Some(tokens),
                    SemanticStatementKind::Line(body) => {
                        if let Some(tokens) = find_print(body) {
                            return Some(tokens);
                        }
                    }
                    _ => {}
                }
            }
            None
        }
        let tokens = find_print(&module.statements).unwrap();
        let values = tokens
            .iter()
            .filter_map(|token| match token {
                PrintToken::Expression(expression) => Some(expression.value_type),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            values,
            [
                SemanticValueType::Integer,
                SemanticValueType::String,
                SemanticValueType::Single
            ],
            "{tokens:#?}"
        );
    }

    #[test]
    fn typed_dim_declarations_retain_suffix_and_annotation_element_types() {
        let module = parse_and_adapt(
            "dim counts%(10)\ndim names(10) as string\ndim totals&(10)\ndim ratios(10) as float64\n",
        )
        .unwrap();
        let declarations = module.top_level_dim_declarations();
        assert_eq!(
            declarations["counts%"].element_type,
            SemanticValueType::Integer
        );
        assert_eq!(
            declarations["names"].element_type,
            SemanticValueType::String
        );
        assert_eq!(
            declarations["totals&"].element_type,
            SemanticValueType::Long
        );
        assert_eq!(
            declarations["ratios"].element_type,
            SemanticValueType::Double
        );
    }

    #[test]
    fn typed_name_occurrences_retain_resolved_expression_types() {
        let module = parse_and_adapt(
            "dim amount as long\nprint amount\ndim labels(2) as string\nprint labels(1)\n",
        )
        .unwrap();
        let typed = SemanticModule::typed_names_in_statements(&module.statements);
        assert!(typed.iter().any(|(name, value_type)| {
            name == "amount" && *value_type == SemanticValueType::Long
        }));
        assert!(typed.iter().any(|(name, value_type)| {
            name == "labels" && *value_type == SemanticValueType::String
        }));
    }

    #[test]
    fn annotates_rank_two_array_references_as_typed_multi_indexes() {
        let module = parse_and_adapt(
            "dim grid%(2, 3)\ndim names(2, 3) as string\nprint grid%(1, 2), names(2, 3)\n",
        )
        .unwrap();
        fn find_print(statements: &[SemanticStatement]) -> Option<&[PrintToken]> {
            for statement in statements {
                match &statement.kind {
                    SemanticStatementKind::Print { tokens, .. } => return Some(tokens),
                    SemanticStatementKind::Line(body) => {
                        if let Some(tokens) = find_print(body) {
                            return Some(tokens);
                        }
                    }
                    _ => {}
                }
            }
            None
        }
        let tokens = find_print(&module.statements).expect("expected PRINT");
        let values = tokens
            .iter()
            .filter_map(|token| match token {
                PrintToken::Expression(expression) => Some(expression),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(
            matches!(&values[0].kind, ExpressionKind::MultiIndex { name, indices } if name == "grid%" && indices.len() == 2)
        );
        assert_eq!(values[0].value_type, SemanticValueType::Integer);
        assert!(
            matches!(&values[1].kind, ExpressionKind::MultiIndex { name, indices } if name == "names" && indices.len() == 2)
        );
        assert_eq!(values[1].value_type, SemanticValueType::String);
    }

    #[test]
    fn annotates_callable_local_array_references_in_lexical_scope() {
        let module = parse_and_adapt(
            "dim values%(4)\nprocedure work()\ndim values(8) as string\nprint values(2)\nend procedure\n",
        ).unwrap();
        fn find_print(statements: &[SemanticStatement]) -> Option<&[PrintToken]> {
            for statement in statements {
                match &statement.kind {
                    SemanticStatementKind::Print { tokens, .. } => return Some(tokens),
                    SemanticStatementKind::Line(body) => {
                        if let Some(tokens) = find_print(body) {
                            return Some(tokens);
                        }
                    }
                    _ => {}
                }
            }
            None
        }
        let tokens = find_print(&module.callables[0].body).unwrap();
        let Some(PrintToken::Expression(expression)) = tokens.first() else {
            panic!()
        };
        assert!(
            matches!(expression.kind, ExpressionKind::Index { ref name, .. } if name == "values")
        );
        assert_eq!(expression.value_type, SemanticValueType::String);
    }

    #[test]
    fn annotates_callable_array_parameter_references_from_signature() {
        let module =
            parse_and_adapt("function first%(items%(?))\nreturn items%(0)\nend function\n")
                .unwrap();
        let SemanticStatementKind::Line(body) = &module.callables[0].body[0].kind else {
            panic!("callable body should retain its source line");
        };
        let SemanticStatementKind::Return(ReturnValue::Value(expression)) = &body[0].kind else {
            panic!("expected a value return from the array parameter");
        };
        assert!(
            matches!(expression.kind, ExpressionKind::Index { ref name, .. } if name == "items%")
        );
        assert_eq!(expression.value_type, SemanticValueType::Integer);
    }

    #[test]
    fn adapts_header_and_dependencies_with_generated_spans() {
        let source = "program Demo shared globals\nrequire com.bascal.io\nimport com.bascal.math\n";
        let module = adapt_module(&rdgen_frontend::parse(source).unwrap());
        assert_eq!(
            module.span,
            SourceSpan {
                start: 0,
                end: source.len() - 1
            }
        );
        assert_eq!(
            module.header,
            Some(ModuleHeader::Program {
                name: "Demo".into(),
                shared: Some("globals".into()),
                span: SourceSpan { start: 0, end: 27 }
            })
        );
        assert_eq!(module.dependencies.len(), 2);
        assert_eq!(module.dependencies[0].kind, DependencyKind::Require);
        assert_eq!(module.dependencies[1].path, "com.bascal.math");
        assert!(
            module
                .dependencies
                .iter()
                .all(|dependency| !dependency.resolved)
        );
    }

    #[test]
    fn adapts_records_without_grammar_wrapper_nodes() {
        let module = adapt_module(
            &rdgen_frontend::parse(
                "record Parent combines Child\nname: string(24) right\nend record\n",
            )
            .unwrap(),
        );
        assert_eq!(module.records.len(), 1);
        assert_eq!(module.records[0].combines, ["Child"]);
        assert_eq!(module.records[0].fields[0].name, "name");
        assert!(matches!(module.records[0].fields[0].field_type,
            RecordFieldType::String { ref capacity, ref alignment, .. }
            if capacity.as_deref() == Some("24") && *alignment == Some(StringAlignment::Right)));
    }

    #[test]
    fn adapts_callable_signatures_and_parameter_axes() {
        let module = adapt_module(&rdgen_frontend::parse(
            "function sum%(byref values%(?), byval fixed%(3), byval count as long)\nreturn count\nend function\nprocedure log(message$ as integer, code as double, implicit)\nend procedure\nmethod scale[integer](byref factor%)\nend method\n",
        ).unwrap());
        assert_eq!(module.callables.len(), 3);
        assert_eq!(module.callables[0].kind, CallableKind::Function);
        assert_eq!(module.callables[0].name, "sum%");
        assert_eq!(
            module.callables[0].parameters[0].passing,
            Some(Passing::ByRef)
        );
        assert_eq!(module.callables[0].parameters[0].array_axes, 1);
        assert_eq!(
            module.callables[0].parameters[0].dimensions,
            [DimAxis::Inferred]
        );
        assert_eq!(
            module.callables[0].parameters[1].dimensions,
            [DimAxis::Fixed("3".into())]
        );
        assert_eq!(
            module.callables[0].parameters[0].type_suffix.as_deref(),
            Some("%")
        );
        assert_eq!(
            module.callables[0].parameters[1].type_suffix.as_deref(),
            Some("%")
        );
        assert_eq!(module.callables[1].kind, CallableKind::Procedure);
        assert_eq!(
            module.callables[1].parameters[0].type_suffix.as_deref(),
            Some("$")
        );
        assert_eq!(
            module.callables[1].parameters[0].value_type,
            SemanticValueType::String
        );
        assert_eq!(
            module.callables[1].parameters[0].type_annotation.as_deref(),
            Some("integer"),
            "parameter annotation is retained even when a suffix takes precedence"
        );
        assert_eq!(
            module.callables[1].parameters[1].value_type,
            SemanticValueType::Double
        );
        assert_eq!(
            module.callables[1].parameters[1].type_annotation.as_deref(),
            Some("double")
        );
        assert_eq!(
            module.callables[1].parameters[2].value_type,
            SemanticValueType::Single
        );
        assert_eq!(
            module.callables[0].parameters[2].value_type,
            SemanticValueType::Long
        );
        assert_eq!(
            module.callables[0].parameters[2].type_annotation.as_deref(),
            Some("long")
        );
        assert_eq!(module.callables[2].receiver.as_deref(), Some("integer"));
        assert_eq!(module.callables[0].result_type.as_deref(), Some("%"));
        assert!(!module.callables[0].body.is_empty());
        assert_eq!(
            module.callable_value_type("sum%"),
            SemanticValueType::Integer
        );
        assert_eq!(
            module.callable_value_type("log"),
            SemanticValueType::Unknown
        );
        let mut call = Expression {
            kind: ExpressionKind::Call {
                name: "sum%".into(),
                arguments: Vec::new(),
            },
            span: module.callables[0].span,
            value_type: SemanticValueType::Unknown,
            record_type: None,
        };
        module.annotate_expression_types(&mut call);
        assert_eq!(call.value_type, SemanticValueType::Integer);
        assert_eq!(
            SemanticValueType::from_suffix(Some('$')),
            SemanticValueType::String
        );
        assert_eq!(
            SemanticValueType::from_suffix(None),
            SemanticValueType::Unknown
        );
        assert_eq!(SemanticValueType::Integer.suffix(), Some('%'));
        assert_eq!(SemanticValueType::Boolean.suffix(), None);
    }

    #[test]
    fn unsuffixed_function_result_defaults_to_single_in_typed_ir() {
        let module = adapt_module(
            &rdgen_frontend::parse("function total()\nreturn 1\nend function\n").unwrap(),
        );

        assert_eq!(module.callables[0].result_type.as_deref(), Some("!"));
        assert_eq!(
            module.callable_value_type("total"),
            SemanticValueType::Single
        );
        let mut call = Expression {
            kind: ExpressionKind::Call {
                name: "total".into(),
                arguments: Vec::new(),
            },
            span: module.callables[0].span,
            value_type: SemanticValueType::Unknown,
            record_type: None,
        };
        module.annotate_expression_types(&mut call);
        assert_eq!(call.value_type, SemanticValueType::Single);
    }

    #[test]
    fn builtin_scalar_method_on_a_typed_receiver_becomes_an_ordinary_call() {
        let module = parse_and_adapt("s$ = \"hello\"\nprint s$.left(2)\nprint s$.len()\nn% = -3\nprint n%.abs()\n")
            .unwrap();
        let mut left = Expression {
            kind: ExpressionKind::Member {
                base: Some(Box::new(Expression {
                    kind: ExpressionKind::Name("s$".into()),
                    span: module.span,
                    value_type: SemanticValueType::Unknown,
                    record_type: None,
                })),
                member: "left".into(),
                arguments: Some(vec![Expression {
                    kind: ExpressionKind::Literal("2".into()),
                    span: module.span,
                    value_type: SemanticValueType::Integer,
                    record_type: None,
                }]),
            },
            span: module.span,
            value_type: SemanticValueType::Unknown,
            record_type: None,
        };
        module.annotate_expression_types(&mut left);
        let ExpressionKind::Call { name, arguments } = &left.kind else {
            panic!("builtin scalar method was not rewritten: {:?}", left.kind);
        };
        assert_eq!(name, "left$");
        assert_eq!(arguments.len(), 2);
        assert_eq!(left.value_type, SemanticValueType::String);
    }

    #[test]
    fn comment_after_then_still_selects_the_block_if_form() {
        for comment in ["// note", "' note", ""] {
            let source = format!(
                "x% = 1\nif x% = 1 then {comment}\n    x% = 2\nelse\n    x% = 3\nend if\nend\n"
            );
            let module = parse_and_adapt(&source).unwrap();
            let kinds = module
                .statements
                .iter()
                .flat_map(|root| match &root.kind {
                    SemanticStatementKind::Line(children) => children.iter().collect::<Vec<_>>(),
                    _ => vec![root],
                })
                .map(|statement| match &statement.kind {
                    SemanticStatementKind::Assignment { .. } => "assign",
                    SemanticStatementKind::If { block: true, .. } => "block-if",
                    SemanticStatementKind::End => "end",
                    _ => "other",
                })
                .collect::<Vec<_>>();
            assert_eq!(kinds, ["assign", "block-if", "end"], "comment {comment:?}");
        }
    }

    #[test]
    fn scalar_method_without_explicit_result_takes_its_receiver_type() {
        let module = parse_and_adapt(
            "method shout[string]()\nreturn self$\nend method\nmethod twice[integer]()\nreturn self%\nend method\nmethod widen%[string]()\nreturn 1\nend method\n",
        )
        .unwrap();
        let result = |name: &str| {
            module
                .callables
                .iter()
                .find(|callable| callable.name == name)
                .unwrap()
                .result_type
                .clone()
        };
        assert_eq!(result("shout").as_deref(), Some("$"));
        assert_eq!(result("twice").as_deref(), Some("%"));
        assert_eq!(result("widen%").as_deref(), Some("%"));
    }

    #[test]
    fn annotates_user_scalar_method_result_types() {
        let module = parse_and_adapt(
            "dim title as string\nconst label = \"hello\"\nmethod capitalize$[string]()\nreturn self$\nend method\nmethod size%[string]()\nreturn 1\nend method\nmethod echo[string]()\nend method\n",
        ).unwrap();
        let mut capitalize = Expression {
            kind: ExpressionKind::Member {
                base: Some(Box::new(Expression {
                    kind: ExpressionKind::Literal("\"hello\"".into()),
                    span: module.span,
                    value_type: SemanticValueType::String,
                    record_type: None,
                })),
                member: "capitalize".into(),
                arguments: Some(Vec::new()),
            },
            span: module.span,
            value_type: SemanticValueType::Unknown,
            record_type: None,
        };
        module.annotate_expression_types(&mut capitalize);
        assert_eq!(capitalize.value_type, SemanticValueType::String);

        let mut size = Expression {
            kind: ExpressionKind::Member {
                base: Some(Box::new(Expression {
                    kind: ExpressionKind::Literal("\"hello\"".into()),
                    span: module.span,
                    value_type: SemanticValueType::String,
                    record_type: None,
                })),
                member: "size".into(),
                arguments: Some(Vec::new()),
            },
            span: module.span,
            value_type: SemanticValueType::Unknown,
            record_type: None,
        };
        module.annotate_expression_types(&mut size);
        assert_eq!(size.value_type, SemanticValueType::Integer);

        let mut echo = Expression {
            kind: ExpressionKind::Member {
                base: Some(Box::new(Expression {
                    kind: ExpressionKind::Literal("\"hello\"".into()),
                    span: module.span,
                    value_type: SemanticValueType::String,
                    record_type: None,
                })),
                member: "echo".into(),
                arguments: Some(Vec::new()),
            },
            span: module.span,
            value_type: SemanticValueType::Unknown,
            record_type: None,
        };
        module.annotate_expression_types(&mut echo);
        assert_eq!(echo.value_type, SemanticValueType::String);

        let mut variable_method = Expression {
            kind: ExpressionKind::Member {
                base: Some(Box::new(Expression {
                    kind: ExpressionKind::Name("title".into()),
                    span: module.span,
                    value_type: SemanticValueType::Unknown,
                    record_type: None,
                })),
                member: "size".into(),
                arguments: Some(Vec::new()),
            },
            span: module.span,
            value_type: SemanticValueType::Unknown,
            record_type: None,
        };
        module.annotate_expression_types(&mut variable_method);
        assert_eq!(variable_method.value_type, SemanticValueType::Integer);

        let mut constant_method = Expression {
            kind: ExpressionKind::Member {
                base: Some(Box::new(Expression {
                    kind: ExpressionKind::Name("label".into()),
                    span: module.span,
                    value_type: SemanticValueType::Unknown,
                    record_type: None,
                })),
                member: "size".into(),
                arguments: Some(Vec::new()),
            },
            span: module.span,
            value_type: SemanticValueType::Unknown,
            record_type: None,
        };
        module.annotate_expression_types(&mut constant_method);
        assert_eq!(constant_method.value_type, SemanticValueType::Integer);
    }

    #[test]
    fn annotates_scalar_method_result_using_argument_arity() {
        let module = parse_and_adapt(
            "method choose$[string]()\nreturn self$\nend method\nmethod choose%[string](index%)\nreturn index%\nend method\n",
        )
        .unwrap();
        let mut call = Expression {
            kind: ExpressionKind::Member {
                base: Some(Box::new(Expression {
                    kind: ExpressionKind::Literal("\"item\"".into()),
                    span: module.span,
                    value_type: SemanticValueType::String,
                    record_type: None,
                })),
                member: "choose".into(),
                arguments: Some(vec![Expression {
                    kind: ExpressionKind::Literal("3".into()),
                    span: module.span,
                    value_type: SemanticValueType::Integer,
                    record_type: None,
                }]),
            },
            span: module.span,
            value_type: SemanticValueType::Unknown,
            record_type: None,
        };
        module.annotate_expression_types(&mut call);
        assert_eq!(call.value_type, SemanticValueType::Integer);
    }

    #[test]
    fn annotates_builtin_call_result_types() {
        let module = adapt_module(&rdgen_frontend::parse("end\n").unwrap());
        let cases = [
            (
                "chr$",
                SemanticValueType::Integer,
                SemanticValueType::String,
            ),
            ("str$", SemanticValueType::Double, SemanticValueType::String),
            ("mid$", SemanticValueType::String, SemanticValueType::String),
            (
                "mki$",
                SemanticValueType::Integer,
                SemanticValueType::String,
            ),
            ("mkl$", SemanticValueType::Long, SemanticValueType::String),
            ("mks$", SemanticValueType::Single, SemanticValueType::String),
            ("mkd$", SemanticValueType::Double, SemanticValueType::String),
            (
                "mki%",
                SemanticValueType::Integer,
                SemanticValueType::Unknown,
            ),
            (
                "mkd#",
                SemanticValueType::Double,
                SemanticValueType::Unknown,
            ),
            ("abs", SemanticValueType::Double, SemanticValueType::Double),
            ("int", SemanticValueType::Double, SemanticValueType::Double),
            (
                "fix",
                SemanticValueType::Integer,
                SemanticValueType::Integer,
            ),
            ("sqr", SemanticValueType::Integer, SemanticValueType::Single),
            ("sgn", SemanticValueType::Double, SemanticValueType::Integer),
            (
                "cint",
                SemanticValueType::Double,
                SemanticValueType::Integer,
            ),
            ("clng", SemanticValueType::Integer, SemanticValueType::Long),
            ("csng", SemanticValueType::Double, SemanticValueType::Single),
            (
                "cdbl",
                SemanticValueType::Integer,
                SemanticValueType::Double,
            ),
            ("sin", SemanticValueType::Integer, SemanticValueType::Single),
            ("val", SemanticValueType::String, SemanticValueType::Single),
            ("len", SemanticValueType::String, SemanticValueType::Integer),
            ("asc", SemanticValueType::String, SemanticValueType::Integer),
            (
                "instr",
                SemanticValueType::String,
                SemanticValueType::Integer,
            ),
        ];
        for (name, argument_type, expected_type) in cases {
            let mut call = Expression {
                kind: ExpressionKind::Call {
                    name: name.to_string(),
                    arguments: vec![Expression {
                        kind: ExpressionKind::Literal("1".to_string()),
                        span: SourceSpan { start: 0, end: 1 },
                        value_type: argument_type,
                        record_type: None,
                    }],
                },
                span: SourceSpan { start: 0, end: 1 },
                value_type: SemanticValueType::Unknown,
                record_type: None,
            };
            module.annotate_expression_types(&mut call);
            assert_eq!(call.value_type, expected_type, "{name}");
        }
    }

    #[test]
    fn preserves_nested_callable_declarations_for_backend_facts() {
        let module = adapt_module(&rdgen_frontend::parse(
            "function work%(value%)\nif value% then\ndim local%(10)\nglobal shared%\nend if\nreturn value%\nend function\n",
        ).unwrap());
        let body = &module.callables[0].body;
        let Some(SemanticStatement {
            kind: SemanticStatementKind::Line(items),
            ..
        }) = body.first()
        else {
            panic!()
        };
        assert!(
            matches!(items[0].kind, SemanticStatementKind::If { ref then_body, .. }
            if then_body.iter().any(|statement| matches!(statement.kind, SemanticStatementKind::Line(_))))
        );
    }

    #[test]
    fn preserves_bare_intrinsic_identifier_suffixes() {
        let module =
            adapt_module(&rdgen_frontend::parse("print inkey$\nvalue$ = date$\n").unwrap());
        let SemanticStatementKind::Line(print) = &module.statements[0].kind else {
            panic!()
        };
        assert!(
            matches!(print[0].kind, SemanticStatementKind::Print { ref tokens, .. }
            if matches!(tokens[0], PrintToken::Expression(ref expression)
                if matches!(expression.kind, ExpressionKind::Name(ref name) if name == "inkey$")))
        );
        let SemanticStatementKind::Line(assignment) = &module.statements[1].kind else {
            panic!()
        };
        assert!(
            matches!(assignment[0].kind, SemanticStatementKind::Assignment { ref value, .. }
            if matches!(value.kind, ExpressionKind::Name(ref name) if name == "date$"))
        );
        let SemanticStatementKind::Assignment { value, .. } = &assignment[0].kind else {
            panic!()
        };
        assert_eq!(value.value_type, SemanticValueType::String);
    }

    #[test]
    fn annotates_call_types_across_statement_trees() {
        let module =
            parse_and_adapt("function f%()\nreturn 1\nend function\nresult% = f%()\n").unwrap();
        let SemanticStatementKind::Line(items) = &module.statements[0].kind else {
            panic!()
        };
        let SemanticStatementKind::Assignment { value, .. } = &items[0].kind else {
            panic!()
        };
        assert_eq!(value.value_type, SemanticValueType::Integer);
    }

    #[test]
    fn preserves_field_binding_string_classification() {
        let module =
            adapt_module(&rdgen_frontend::parse("field #1, 10 as name$, 4 as code%\n").unwrap());
        let SemanticStatementKind::Line(field) = &module.statements[0].kind else {
            panic!()
        };
        let SemanticStatementKind::Field { ref bindings, .. } = field[0].kind else {
            panic!()
        };
        assert_eq!(
            bindings
                .iter()
                .map(|binding| binding.is_string)
                .collect::<Vec<_>>(),
            [true, false]
        );
        assert_eq!(bindings[0].type_suffix.as_deref(), Some("$"));
        assert_eq!(bindings[1].type_suffix.as_deref(), Some("%"));
    }

    #[test]
    fn preserves_file_declaration_record_type() {
        let module = adapt_module(
            &rdgen_frontend::parse(
                "file ledger as Entry = open(path$)\nfile scores = open(path$) for input\n",
            )
            .unwrap(),
        );
        let SemanticStatementKind::Line(statements) = &module.statements[0].kind else {
            panic!()
        };
        let SemanticStatementKind::FileDeclaration {
            name,
            record_type: Some(record),
            ..
        } = &statements[0].kind
        else {
            panic!()
        };
        assert_eq!(name.name, "ledger");
        assert_eq!(record.name, "Entry");
        let SemanticStatementKind::Line(sequential) = &module.statements[1].kind else {
            panic!()
        };
        let SemanticStatementKind::FileDeclaration {
            mode: Some(FileModeKind::Input),
            ..
        } = &sequential[0].kind
        else {
            panic!()
        };
    }

    #[test]
    fn preserves_line_and_block_comment_payloads() {
        let module = parse_and_adapt("' apostrophe\n// slash\n/*\n * block\n */\nend\n").unwrap();
        let comments = module
            .statements
            .iter()
            .flat_map(|statement| match &statement.kind {
                SemanticStatementKind::Line(children) => children.as_slice(),
                _ => std::slice::from_ref(statement),
            })
            .filter_map(|statement| match &statement.kind {
                SemanticStatementKind::Comment { block, text } => Some((*block, text.as_str())),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            comments,
            [
                (false, "' apostrophe"),
                (false, "// slash"),
                (true, "/*\n * block\n */"),
            ]
        );
    }

    #[test]
    fn adapts_precedence_expression_with_operator_spelling() {
        let program = rdgen_frontend::parse("result% = -2 ^ 2 + value%\n").unwrap();
        let rdgen_frontend::Program::File { items, .. } = program;
        let rdgen_frontend::FileItem::Statement { statement, .. } = items[0].as_ref() else {
            panic!()
        };
        let rdgen_frontend::TopLevelStatement::Statement { statement, .. } = statement.as_ref()
        else {
            panic!()
        };
        let rdgen_frontend::Statement::Line { first, .. } = statement.as_ref() else {
            panic!()
        };
        let rdgen_frontend::StatementCore::AssignmentOrExpression { statement, .. } =
            first.as_ref()
        else {
            panic!()
        };
        let rdgen_frontend::AssignmentOrExprStmt::Assignment { value, .. } = statement.as_ref()
        else {
            panic!()
        };
        assert!(matches!(adapt_expression(value).kind,
            ExpressionKind::Binary { ref operator, ref left, ref right }
            if operator == "+" && matches!(left.kind, ExpressionKind::Unary { .. }) && matches!(right.kind, ExpressionKind::Name(ref name) if name == "value%")));
    }

    #[test]
    fn adapts_colon_chained_assignments_as_a_statement_list() {
        let module = adapt_module(&rdgen_frontend::parse("a% = 1 : b% += a%\n").unwrap());
        let SemanticStatementKind::Line(statements) = &module.statements[0].kind else {
            panic!()
        };
        assert_eq!(statements.len(), 2);
        assert!(matches!(
            statements[0].kind,
            SemanticStatementKind::Assignment {
                operator: AssignmentOperator::Assign,
                ..
            }
        ));
        assert!(matches!(
            statements[1].kind,
            SemanticStatementKind::Assignment {
                operator: AssignmentOperator::Add,
                ..
            }
        ));
    }

    #[test]
    fn preserves_mid_assignment_as_a_typed_statement() {
        let module =
            adapt_module(&rdgen_frontend::parse("mid$(text$, 2, 3) = replacement$\n").unwrap());
        let SemanticStatementKind::Line(statements) = &module.statements[0].kind else {
            panic!()
        };
        assert!(
            matches!(statements[0].kind, SemanticStatementKind::MidAssign { ref target, ref start, ref length, .. }
            if matches!(target.kind, ExpressionKind::Name(ref name) if name == "text$")
                && matches!(start.kind, ExpressionKind::Literal(ref value) if value == "2")
                && matches!(length.as_ref().map(|value| &value.kind), Some(ExpressionKind::Literal(value)) if value == "3"))
        );
    }

    #[test]
    fn annotates_mid_assignment_operands_from_callable_signatures() {
        let module = parse_and_adapt(
            "function identity$(value$)\nreturn value$\nend function\nmid$(text$, 2, 3) = identity$(\"replacement\")\n",
        ).unwrap();
        let SemanticStatementKind::Line(statements) = &module.statements[0].kind else {
            panic!()
        };
        let SemanticStatementKind::MidAssign { value, .. } = &statements[0].kind else {
            panic!()
        };
        assert_eq!(value.value_type, SemanticValueType::String);
        assert!(matches!(value.kind, ExpressionKind::Call { ref name, .. } if name == "identity$"));
    }

    #[test]
    fn annotates_screen_and_seed_statement_call_results() {
        let module = parse_and_adapt(
            "function value%()\nreturn 1\nend function\nlocate value%(), value%()\ncolor value%(), value%()\nrandomize value%()\n",
        ).unwrap();
        let SemanticStatementKind::Line(locate) = &module.statements[0].kind else {
            panic!()
        };
        assert!(
            matches!(locate[0].kind, SemanticStatementKind::Locate { ref row, ref column }
            if row.value_type == SemanticValueType::Integer && column.value_type == SemanticValueType::Integer)
        );
        let SemanticStatementKind::Line(color) = &module.statements[1].kind else {
            panic!()
        };
        assert!(
            matches!(color[0].kind, SemanticStatementKind::Color { ref foreground, ref background }
            if foreground.value_type == SemanticValueType::Integer
                && background.as_ref().is_some_and(|value| value.value_type == SemanticValueType::Integer))
        );
        let SemanticStatementKind::Line(randomize) = &module.statements[2].kind else {
            panic!()
        };
        assert!(
            matches!(randomize[0].kind, SemanticStatementKind::Randomize(RandomizeSeed::Value(ref seed))
            if seed.value_type == SemanticValueType::Integer)
        );
    }

    #[test]
    fn adapts_print_input_and_write_payloads() {
        let module = adapt_module(
            &rdgen_frontend::parse(
                "print #1, value%;\ninput #1, target%\nwrite #1, value%, other%\n",
            )
            .unwrap(),
        );
        let SemanticStatementKind::Line(print) = &module.statements[0].kind else {
            panic!()
        };
        assert!(
            matches!(print[0].kind, SemanticStatementKind::Print { ref destination, ref tokens }
            if matches!(destination, PrintDestination::Channel { ref channel, using: None, .. } if matches!(channel.kind, ExpressionKind::Literal(ref value) if value == "1"))
                && tokens.len() == 2)
        );
        let SemanticStatementKind::Line(input) = &module.statements[1].kind else {
            panic!()
        };
        assert!(
            matches!(input[0].kind, SemanticStatementKind::Input { source: InputSource::Channel(_), ref targets }
            if targets.len() == 1)
        );
        let SemanticStatementKind::Line(write) = &module.statements[2].kind else {
            panic!()
        };
        assert!(
            matches!(write[0].kind, SemanticStatementKind::Write { values: WriteValues::Values(ref values), .. } if values.len() == 2)
        );
    }

    #[test]
    fn adapts_file_operation_payloads() {
        let module = adapt_module(&rdgen_frontend::parse(
            "open path$ for output as #1 len = 128\nseek #1, 4\nkill path$\nname old$ as new$\nclose #1\n",
        ).unwrap());
        let SemanticStatementKind::Line(open) = &module.statements[0].kind else {
            panic!()
        };
        assert!(
            matches!(open[0].kind, SemanticStatementKind::Open { ref mode, ref length, .. } if mode.kind == OpenModeKind::Output && length.is_some())
        );
        let SemanticStatementKind::Line(seek) = &module.statements[1].kind else {
            panic!()
        };
        assert!(matches!(seek[0].kind, SemanticStatementKind::Seek { .. }));
        let SemanticStatementKind::Line(kill) = &module.statements[2].kind else {
            panic!()
        };
        assert!(matches!(kill[0].kind, SemanticStatementKind::Kill(_)));
        let SemanticStatementKind::Line(rename) = &module.statements[3].kind else {
            panic!()
        };
        assert!(matches!(
            rename[0].kind,
            SemanticStatementKind::Rename { .. }
        ));
        let SemanticStatementKind::Line(close) = &module.statements[4].kind else {
            panic!()
        };
        assert!(matches!(close[0].kind, SemanticStatementKind::Close(_)));
    }

    #[test]
    fn adapts_data_read_restore_payloads() {
        let module = adapt_module(
            &rdgen_frontend::parse("data 1, 2\nread first%, second%\nrestore marker\n").unwrap(),
        );
        let SemanticStatementKind::Line(data) = &module.statements[0].kind else {
            panic!()
        };
        assert!(
            matches!(data[0].kind, SemanticStatementKind::Data(ref values) if values.len() == 2)
        );
        let SemanticStatementKind::Line(read) = &module.statements[1].kind else {
            panic!()
        };
        assert!(
            matches!(read[0].kind, SemanticStatementKind::Read(ref values) if values.len() == 2)
        );
        let SemanticStatementKind::Line(restore) = &module.statements[2].kind else {
            panic!()
        };
        assert!(
            matches!(restore[0].kind, SemanticStatementKind::Restore(Some(ref target)) if target.name == "marker")
        );
    }

    #[test]
    fn adapts_declaration_payloads() {
        let module = adapt_module(
            &rdgen_frontend::parse(
                "dim values%(?) : name$\ndim total as integer\nconst limit% = 10\nglobal shared%\nglobal defaultValue\n",
            )
            .unwrap(),
        );
        let SemanticStatementKind::Line(dim) = &module.statements[0].kind else {
            panic!()
        };
        assert!(
            matches!(dim[0].kind, SemanticStatementKind::Dim(ref values) if values[0].name == "values%" && values[0].array_axes == 1 && values[0].dimensions == [DimAxis::Inferred])
        );
        let fixed = adapt_module(&rdgen_frontend::parse("dim table%(10, 20)\n").unwrap());
        let SemanticStatementKind::Line(dim) = &fixed.statements[0].kind else {
            panic!()
        };
        assert!(
            matches!(dim[0].kind, SemanticStatementKind::Dim(ref values) if values[0].dimensions == [DimAxis::Fixed("10".into()), DimAxis::Fixed("20".into())])
        );
        let named = adapt_module(&rdgen_frontend::parse("dim table%(limit)\n").unwrap());
        let SemanticStatementKind::Line(dim) = &named.statements[0].kind else {
            panic!()
        };
        assert!(
            matches!(dim[0].kind, SemanticStatementKind::Dim(ref values) if matches!(values[0].dimensions.as_slice(), [DimAxis::Expression(Expression { kind: ExpressionKind::Name(name), .. })] if name == "limit"))
        );
        let SemanticStatementKind::Line(typed_dim) = &module.statements[1].kind else {
            panic!()
        };
        assert!(
            matches!(typed_dim[0].kind, SemanticStatementKind::Dim(ref values) if values[0].name == "total" && values[0].type_annotation.as_deref() == Some("integer"))
        );
        let SemanticStatementKind::Line(constant) = &module.statements[2].kind else {
            panic!()
        };
        assert!(
            matches!(constant[0].kind, SemanticStatementKind::Const { ref name, .. } if name.name == "limit%")
        );
        let SemanticStatementKind::Line(global) = &module.statements[3].kind else {
            panic!()
        };
        assert!(
            matches!(global[0].kind, SemanticStatementKind::Global { ref name, value_type: SemanticValueType::Integer } if name.name == "shared%")
        );
        assert_eq!(
            module.name_scopes().global_types.get("shared%"),
            Some(&SemanticValueType::Integer)
        );
        assert_eq!(
            module.name_scopes().global_types.get("defaultvalue"),
            Some(&SemanticValueType::Single)
        );
    }

    #[test]
    fn adapts_top_level_end_as_a_semantic_statement() {
        let module = parse_and_adapt("end\n").expect("END parses");
        assert!(matches!(module.statements.as_slice(), [SemanticStatement {
            kind: SemanticStatementKind::Line(inner), ..
        }] if matches!(inner.as_slice(), [SemanticStatement { kind: SemanticStatementKind::End, .. }])));
    }

    #[test]
    fn evaluates_decimal_hexadecimal_and_octal_integer_constants() {
        let module = parse_and_adapt(
            "const hexadecimal = &H10\nconst octal = &O10\ndim values%(hexadecimal + octal)\n",
        )
        .expect("integer constant fixture parses");
        let constants = module.top_level_integer_constants();
        assert_eq!(constants.get("hexadecimal"), Some(&16));
        assert_eq!(constants.get("octal"), Some(&8));
        let SemanticStatementKind::Line(dim) = &module.statements[2].kind else {
            panic!()
        };
        let SemanticStatementKind::Dim(values) = &dim[0].kind else {
            panic!()
        };
        let [DimAxis::Expression(capacity)] = values[0].dimensions.as_slice() else {
            panic!()
        };
        assert_eq!(module.evaluate_integer_expression(capacity), Some(24));
    }

    #[test]
    fn evaluates_parenthesized_module_integer_constants() {
        let module = parse_and_adapt("const capacity = (2 + 3) * 4\n").unwrap();
        assert_eq!(
            module.top_level_integer_constants().get("capacity"),
            Some(&20)
        );
    }

    #[test]
    fn evaluates_integer_division_and_modulo_for_array_bounds() {
        let module = parse_and_adapt("const capacity = 17 \\ 3 + 17 mod 3\n").unwrap();
        assert_eq!(
            module.top_level_integer_constants().get("capacity"),
            Some(&7)
        );
    }

    #[test]
    fn evaluates_boolean_constants_using_basic_integer_values() {
        let module = parse_and_adapt("const enabled = true\nconst disabled = false\n").unwrap();
        let constants = module.top_level_integer_constants();
        assert_eq!(constants.get("enabled"), Some(&-1));
        assert_eq!(constants.get("disabled"), Some(&0));
    }

    #[test]
    fn integer_constant_evaluator_rejects_cycles_and_division_by_zero() {
        let module = parse_and_adapt(
            "const first = second\nconst second = first\nconst invalid = 12 \\ 0\n",
        )
        .unwrap();
        let constants = module.top_level_integer_constants();
        assert!(!constants.contains_key("first"));
        assert!(!constants.contains_key("second"));
        assert!(!constants.contains_key("invalid"));
    }

    #[test]
    fn const_names_include_nested_and_callable_bindings() {
        let module = parse_and_adapt(
            "const limit% = 10\nif ready% then\nconst branch$ = \"ok\"\nend if\nprocedure work()\nconst local& = 1\nend procedure\n",
        )
        .unwrap();
        assert_eq!(
            module.const_names(),
            ["branch$", "limit%", "local&"]
                .into_iter()
                .map(String::from)
                .collect()
        );
        assert_eq!(
            module.top_level_const_names(),
            ["branch$", "limit%"]
                .into_iter()
                .map(String::from)
                .collect()
        );
    }

    #[test]
    fn const_types_infer_unsuffixed_initializer_types() {
        let module = parse_and_adapt(
            "const count = 10\nconst label = \"ready\"\nconst alias = label\nconst ratio = 0.25\nconst precise# = 0.25\nconst forwardLabel = laterLabel\nconst laterLabel = \"later\"\nconst forwardRatio = laterRatio\nconst laterRatio = 0.5\nconst forwardSum = 1 + laterRatio\nconst forwardCompare = laterLabel = \"later\"\n",
        ).unwrap();
        assert_eq!(
            module.const_types().get("count"),
            Some(&SemanticValueType::Integer)
        );
        assert_eq!(
            module.const_types().get("label"),
            Some(&SemanticValueType::String)
        );
        assert_eq!(
            module.const_types().get("alias"),
            Some(&SemanticValueType::String)
        );
        assert_eq!(
            module.const_types().get("ratio"),
            Some(&SemanticValueType::Single)
        );
        assert_eq!(
            module.const_types().get("precise#"),
            Some(&SemanticValueType::Double)
        );
        assert_eq!(
            module.const_types().get("forwardLabel"),
            Some(&SemanticValueType::String)
        );
        assert_eq!(
            module.const_types().get("forwardRatio"),
            Some(&SemanticValueType::Single)
        );
        assert_eq!(
            module.const_types().get("forwardSum"),
            Some(&SemanticValueType::Single)
        );
        assert_eq!(
            module.const_types().get("forwardCompare"),
            Some(&SemanticValueType::Integer)
        );
        for (statement, expected) in module.statements.iter().zip([
            SemanticValueType::Integer,
            SemanticValueType::String,
            SemanticValueType::String,
            SemanticValueType::Single,
            SemanticValueType::Double,
        ]) {
            let SemanticStatementKind::Line(items) = &statement.kind else {
                panic!("expected a line statement: {statement:#?}")
            };
            let SemanticStatementKind::Const { value_type, .. } = &items[0].kind else {
                panic!("expected a CONST statement: {:#?}", items[0])
            };
            assert_eq!(*value_type, expected);
        }
    }

    #[test]
    fn binary_expression_types_follow_operator_and_operand_types() {
        let module = parse_and_adapt("end\n").unwrap();
        let binary = |operator: &str, left_type, right_type| Expression {
            kind: ExpressionKind::Binary {
                left: Box::new(Expression {
                    kind: ExpressionKind::Literal("1".into()),
                    span: module.span,
                    value_type: left_type,
                    record_type: None,
                }),
                operator: operator.into(),
                right: Box::new(Expression {
                    kind: ExpressionKind::Literal("1".into()),
                    span: module.span,
                    value_type: right_type,
                    record_type: None,
                }),
            },
            span: module.span,
            value_type: SemanticValueType::Unknown,
            record_type: None,
        };
        let mut concatenation = binary("+", SemanticValueType::String, SemanticValueType::Integer);
        module.annotate_expression_types(&mut concatenation);
        assert_eq!(concatenation.value_type, SemanticValueType::String);

        let mut promotion = binary("+", SemanticValueType::Integer, SemanticValueType::Single);
        module.annotate_expression_types(&mut promotion);
        assert_eq!(promotion.value_type, SemanticValueType::Single);

        let mut comparison = binary("=", SemanticValueType::String, SemanticValueType::String);
        module.annotate_expression_types(&mut comparison);
        assert_eq!(comparison.value_type, SemanticValueType::Boolean);

        let mut division = binary("/", SemanticValueType::Integer, SemanticValueType::Integer);
        module.annotate_expression_types(&mut division);
        assert_eq!(division.value_type, SemanticValueType::Single);

        let mut logical = binary(
            "AND",
            SemanticValueType::Boolean,
            SemanticValueType::Boolean,
        );
        module.annotate_expression_types(&mut logical);
        assert_eq!(logical.value_type, SemanticValueType::Boolean);
    }

    #[test]
    fn constant_type_facts_respect_module_and_callable_scopes() {
        let module = parse_and_adapt(
            "const label = \"global\"\nprocedure work()\nconst localLabel = 7\nprint localLabel\nend procedure\n",
        ).unwrap();
        assert_eq!(
            module.top_level_const_types().get("label"),
            Some(&SemanticValueType::String)
        );
        assert_eq!(module.top_level_const_types().get("localLabel"), None);
        assert_eq!(
            module.const_types().get("localLabel"),
            Some(&SemanticValueType::Integer)
        );

        let mut global_name = Expression {
            kind: ExpressionKind::Name("label".into()),
            span: module.span,
            value_type: SemanticValueType::Unknown,
            record_type: None,
        };
        module.annotate_expression_types(&mut global_name);
        assert_eq!(global_name.value_type, SemanticValueType::String);

        let mut callable_facts = module.clone();
        callable_facts
            .statements
            .extend(module.callables[0].body.clone());
        let mut local_name = Expression {
            kind: ExpressionKind::Name("localLabel".into()),
            span: module.span,
            value_type: SemanticValueType::Unknown,
            record_type: None,
        };
        callable_facts.annotate_expression_types(&mut local_name);
        assert_eq!(local_name.value_type, SemanticValueType::Integer);
    }

    #[test]
    fn evaluates_integer_constants_for_compile_time_consumers() {
        let module = parse_and_adapt(
            "const base = 10\nconst doubled = base * 2\nconst adjusted = -doubled + 3\nfunction size%()\nconst local = 7\nreturn local\nend function\ndim values%(adjusted)\n",
        )
        .unwrap();
        let values = module.integer_constants();
        assert_eq!(values.get("base"), Some(&10));
        assert_eq!(values.get("doubled"), Some(&20));
        assert_eq!(values.get("adjusted"), Some(&-17));
        assert_eq!(values.get("local"), Some(&7));
    }

    #[test]
    fn generated_frontend_entry_point_returns_semantic_module() {
        let module = parse_and_adapt("value% = 1\n").unwrap();
        assert_eq!(module.statements.len(), 1);
        assert!(parse_and_adapt("value% =").is_err());
        let error = rdgen_frontend::parse("value% =").unwrap_err();
        let diagnostic = parse_diagnostic("demo.bcl", &error);
        assert_eq!(diagnostic.pos.filename, "demo.bcl");
        assert_eq!(diagnostic.pos.column, error.position + 1);
    }

    #[test]
    fn backend_capability_facts_traverse_callable_control_flow() {
        let module = parse_and_adapt(
            "function work%()\ntry\nwork% = 1\ncatch err%, line%\nwork% = 2\nend try\nreturn work%\nend function\n",
        )
        .unwrap();
        assert!(module.contains_try());
    }

    #[test]
    fn error_handler_targets_preserve_scope_and_source_order() {
        let module = parse_and_adapt(
            "on error goto first\non error goto second\nprocedure handler()\non error goto third\nend procedure\n",
        )
        .unwrap();
        assert_eq!(
            module.top_level_error_handler_targets(),
            ["first", "second"]
        );
        assert_eq!(module.error_handler_targets(), ["first", "second", "third"]);
    }

    #[test]
    fn dispatch_counts_cover_semantic_throw_and_try_nodes() {
        let module = parse_and_adapt("try\nthrow\nend try\n").unwrap();
        assert_eq!(module.top_level_raise_site_count(), 1);
        assert_eq!(module.top_level_try_catch_count(), 1);
    }

    #[test]
    fn dispatch_counts_include_synthesized_record_file_open_raise_sites() {
        let mut module = parse_and_adapt("end\n").unwrap();
        let file = |owner| LoweredRecordFile {
            name: "db".to_string(),
            channel: 1,
            record_type: "Entry".to_string(),
            record_length: 0,
            owner,
            fields: Vec::new(),
            record_locals: Vec::new(),
        };
        module.lowered_record_files = vec![file(None), file(Some("update".to_string()))];
        assert_eq!(module.top_level_raise_site_count(), 1);
    }

    #[test]
    fn gosub_count_traverses_top_level_control_flow() {
        let module = parse_and_adapt("if ready% then\ngosub target\nend if\n").unwrap();
        assert_eq!(module.top_level_gosub_count(), 1);
    }

    #[test]
    fn dependency_merge_preserves_left_to_right_order() {
        let mut root = parse_and_adapt("root% = 1\n").unwrap();
        let first = parse_and_adapt("first% = 1\n").unwrap();
        let second = parse_and_adapt("second% = 1\n").unwrap();
        root.prepend_dependency(second);
        root.prepend_dependency(first);
        let mut names = Vec::new();
        for statement in &root.statements {
            let SemanticStatementKind::Line(items) = &statement.kind else {
                continue;
            };
            for item in items {
                let SemanticStatementKind::Assignment { target, .. } = &item.kind else {
                    continue;
                };
                let ExpressionKind::Name(name) = &target.kind else {
                    continue;
                };
                names.push(name.as_str());
            }
        }
        assert_eq!(names, ["first%", "second%", "root%"]);
    }

    #[test]
    fn dependency_merge_preserves_dependency_metadata_order() {
        let mut root = parse_and_adapt("require rootDep\nvalue% = 1\n").unwrap();
        let first = parse_and_adapt("require firstDep\nfirst% = 1\n").unwrap();
        let second = parse_and_adapt("require secondDep\nsecond% = 1\n").unwrap();
        root.prepend_dependency(second);
        root.prepend_dependency(first);
        let paths: Vec<_> = root
            .dependencies
            .iter()
            .map(|dependency| dependency.path.as_str())
            .collect();
        assert_eq!(paths, ["firstDep", "secondDep", "rootDep"]);
    }

    #[test]
    fn dependency_merge_preserves_callable_source_identity() {
        let mut root =
            parse_and_adapt_named("root.bcl", "function root%()\nreturn 0\nend function\n")
                .unwrap();
        let first =
            parse_and_adapt_named("first.bcl", "function first%()\nreturn 0\nend function\n")
                .unwrap();
        let second =
            parse_and_adapt_named("second.bcl", "function second%()\nreturn 0\nend function\n")
                .unwrap();
        root.prepend_dependency(second);
        root.prepend_dependency(first);

        let callable_sources = root
            .callables
            .iter()
            .map(|callable| root.sources[callable.source_index].filename.as_str())
            .collect::<Vec<_>>();
        assert_eq!(callable_sources, ["first.bcl", "second.bcl", "root.bcl"]);
    }

    #[test]
    fn name_scopes_retain_global_and_callable_visibility() {
        let module = parse_and_adapt(
            "shared% = 1\nfunction work%()\nglobal shared%\nlocal% = shared%\nreturn local%\nend function\n",
        )
        .unwrap();
        let scopes = module.name_scopes();
        assert!(scopes.global_names.contains("shared%"));
        assert!(
            scopes
                .callable_globals
                .get("work%")
                .is_some_and(|names| names.contains("shared%") && !names.contains("local%"))
        );
    }

    #[test]
    fn adapts_record_literal_fields() {
        let module =
            parse_and_adapt("value = { name: 1, count: other% }\npartial = ?{ name: 2 }\n")
                .unwrap();
        let SemanticStatementKind::Line(first) = &module.statements[0].kind else {
            panic!()
        };
        let SemanticStatementKind::Assignment { value, .. } = &first[0].kind else {
            panic!()
        };
        assert!(
            matches!(value.kind, ExpressionKind::RecordLiteral(ref fields) if fields.len() == 2 && fields[0].name == "name")
        );
        let SemanticStatementKind::Line(second) = &module.statements[1].kind else {
            panic!()
        };
        let SemanticStatementKind::Assignment { value, .. } = &second[0].kind else {
            panic!()
        };
        assert!(
            matches!(value.kind, ExpressionKind::PartialRecordLiteral(ref fields) if fields.len() == 1 && fields[0].name == "name")
        );
    }

    #[test]
    fn annotates_record_member_fields_from_declared_record_types() {
        let module = parse_and_adapt(
            "record Address\nzip: string(12)\nend record\nrecord Person combines Address\nage: int\naddress: Address\nend record\ndim people(4) as Person\nprint people(1).age, people(1).address.zip, people(1).zip\n",
        )
        .unwrap();
        fn find_print(statements: &[SemanticStatement]) -> Option<&[PrintToken]> {
            for statement in statements {
                match &statement.kind {
                    SemanticStatementKind::Print { tokens, .. } => return Some(tokens),
                    SemanticStatementKind::Line(body) => {
                        if let Some(tokens) = find_print(body) {
                            return Some(tokens);
                        }
                    }
                    _ => {}
                }
            }
            None
        }
        let tokens = find_print(&module.statements).expect("PRINT statement");
        let [
            PrintToken::Expression(age),
            PrintToken::Comma { .. },
            PrintToken::Expression(zip),
            PrintToken::Comma { .. },
            PrintToken::Expression(combined_zip),
        ] = tokens
        else {
            panic!("{tokens:#?}")
        };
        assert_eq!(age.value_type, SemanticValueType::Long);
        assert_eq!(zip.value_type, SemanticValueType::String);
        assert_eq!(combined_zip.value_type, SemanticValueType::String);

        let local = parse_and_adapt(
            "record Address\nzip: string(12)\nend record\nrecord Person\nage: int\naddress: Address\nend record\nprocedure show()\ndim person as Person\nperson = {age: 1, address: {zip: \"12345\"}}\nprint person.address.zip\nend procedure\n",
        )
        .unwrap();
        fn find_record_assignment(statements: &[SemanticStatement]) -> Option<&Expression> {
            for statement in statements {
                match &statement.kind {
                    SemanticStatementKind::Assignment { value, .. } => return Some(value),
                    SemanticStatementKind::Line(body) => {
                        if let Some(value) = find_record_assignment(body) {
                            return Some(value);
                        }
                    }
                    _ => {}
                }
            }
            None
        }
        let record_value =
            find_record_assignment(&local.callables[0].body).expect("callable record assignment");
        assert_eq!(record_value.record_type.as_deref(), Some("Person"));
        let ExpressionKind::RecordLiteral(fields) = &record_value.kind else {
            panic!()
        };
        let address = fields.iter().find(|field| field.name == "address").unwrap();
        assert_eq!(address.value.record_type.as_deref(), Some("Address"));
        assert!(matches!(
            address.value.kind,
            ExpressionKind::RecordLiteral(_)
        ));
        let local_tokens = find_print(&local.callables[0].body).expect("callable PRINT statement");
        let [PrintToken::Expression(local_age)] = local_tokens else {
            panic!("{local_tokens:#?}")
        };
        assert_eq!(local_age.value_type, SemanticValueType::String);

        let fluent = parse_and_adapt(
            "record Person\nage: int\nmethod copy()\nend method\nend record\nlet person = {age: 1}\nprint person.copy()\n",
        )
        .unwrap();
        let fluent_tokens = find_print(&fluent.statements).expect("fluent method PRINT statement");
        let [PrintToken::Expression(fluent_call)] = fluent_tokens else {
            panic!("{fluent_tokens:#?}")
        };
        assert_eq!(fluent_call.record_type.as_deref(), Some("Person"));
        assert!(matches!(
            fluent_call.kind,
            ExpressionKind::Member { ref base, .. }
                if base.as_deref().is_some_and(|base| base.record_type.as_deref() == Some("Person"))
        ));

        let matrix = parse_and_adapt(
            "record Person\nage: int\nend record\ndim grid(2, 2) as Person\nprint grid(1, 1).age\n",
        )
        .unwrap();
        let matrix_tokens = find_print(&matrix.statements).expect("record array PRINT statement");
        let [PrintToken::Expression(field)] = matrix_tokens else {
            panic!("{matrix_tokens:#?}")
        };
        let ExpressionKind::Member {
            base: Some(base), ..
        } = &field.kind
        else {
            panic!("{field:#?}")
        };
        assert!(matches!(base.kind, ExpressionKind::MultiIndex { .. }));
        assert_eq!(base.record_type.as_deref(), Some("Person"));
        assert_eq!(field.value_type, SemanticValueType::Long);
    }

    #[test]
    fn adapts_representative_generated_statement_corpus() {
        let source = "dim value%\nconst limit% = 3\nglobal shared%\nprint value%\ninput value%\nwrite #1, value%\nopen path$ for input as #1\ndata 1, 2\nread value%\nlocate 1, 2\ncolor 3, 4\nswap value%, shared%\nrandomize 1\npoke 1, 2\nout 1, 2\nwidth 80\nline input #1, value$\nget #1\nput #1\nlset field$ = value$\nrset field$ = value$\n";
        let module = parse_and_adapt(source).unwrap();
        assert_eq!(module.statements.len(), 21);
        assert!(
            module
                .statements
                .iter()
                .all(|statement| !matches!(statement.kind, SemanticStatementKind::Unsupported))
        );
    }

    #[test]
    fn preserves_typed_control_transfer_targets() {
        let module =
            parse_and_adapt("goto target\ngosub target\nresume next\non error goto target\n")
                .unwrap();
        let SemanticStatementKind::Line(goto) = &module.statements[0].kind else {
            panic!()
        };
        assert!(
            matches!(goto[0].kind, SemanticStatementKind::Goto(ref target) if target.name == "target")
        );
        let SemanticStatementKind::Line(gosub) = &module.statements[1].kind else {
            panic!()
        };
        assert!(
            matches!(gosub[0].kind, SemanticStatementKind::Gosub(ref target) if target.name == "target")
        );
        let SemanticStatementKind::Line(resume) = &module.statements[2].kind else {
            panic!()
        };
        assert!(matches!(
            resume[0].kind,
            SemanticStatementKind::Resume(Some(ResumeTarget::Next))
        ));
        let SemanticStatementKind::Line(error) = &module.statements[3].kind else {
            panic!()
        };
        assert!(
            matches!(error[0].kind, SemanticStatementKind::OnErrorGoto(ErrorHandlerTarget::Label(ref target)) if target.name == "target")
        );
    }

    #[test]
    fn preserves_declaration_name_spans() {
        let module = parse_and_adapt("record Item\nvalue: int\nend record\n").unwrap();
        assert_eq!(
            module.records[0].name_span,
            SourceSpan { start: 7, end: 11 }
        );
        assert_eq!(
            module.records[0].fields[0].name_span,
            SourceSpan { start: 12, end: 17 }
        );
        let declarations = parse_and_adapt("dim values%(?)\nconst limit% = 1\n").unwrap();
        let SemanticStatementKind::Line(dim) = &declarations.statements[0].kind else {
            panic!()
        };
        assert!(
            matches!(dim[0].kind, SemanticStatementKind::Dim(ref values) if values[0].span.start < values[0].span.end)
        );
    }

    #[test]
    fn preserves_typed_optional_statement_payloads() {
        let module = parse_and_adapt("return\nthrow\nrandomize\nwrite #1\n").unwrap();
        let SemanticStatementKind::Line(return_stmt) = &module.statements[0].kind else {
            panic!()
        };
        assert!(matches!(
            return_stmt[0].kind,
            SemanticStatementKind::Return(ReturnValue::Default)
        ));
        let SemanticStatementKind::Line(throw_stmt) = &module.statements[1].kind else {
            panic!()
        };
        assert!(matches!(
            throw_stmt[0].kind,
            SemanticStatementKind::Throw(ThrowValue::Bare)
        ));
        let SemanticStatementKind::Line(randomize) = &module.statements[2].kind else {
            panic!()
        };
        assert!(matches!(
            randomize[0].kind,
            SemanticStatementKind::Randomize(RandomizeSeed::Default)
        ));
        let SemanticStatementKind::Line(write) = &module.statements[3].kind else {
            panic!()
        };
        assert!(matches!(
            write[0].kind,
            SemanticStatementKind::Write {
                values: WriteValues::Omitted,
                ..
            }
        ));
    }

    #[test]
    fn preserves_catch_binding_spans() {
        let module =
            parse_and_adapt("try\nvalue% = 1\ncatch err%, line%, source$\nvalue% = 2\nend try\n")
                .unwrap();
        let SemanticStatementKind::Line(body) = &module.statements[0].kind else {
            panic!()
        };
        let SemanticStatementKind::Try {
            catch: Some(catch), ..
        } = &body[0].kind
        else {
            panic!()
        };
        assert!(catch.error_span.start < catch.error_span.end);
        assert!(catch.line_span.start < catch.line_span.end);
        assert_eq!(catch.error_type, SemanticValueType::Integer);
        assert_eq!(catch.line_type, SemanticValueType::Integer);
        assert_eq!(catch.source_type, Some(SemanticValueType::String));
        assert!(catch.source_span.is_some());

        let suffixless = parse_and_adapt("try\nbeep\ncatch error, line\nbeep\nend try\n")
            .unwrap();
        let SemanticStatementKind::Line(body) = &suffixless.statements[0].kind else {
            panic!()
        };
        let SemanticStatementKind::Try {
            catch: Some(catch), ..
        } = &body[0].kind
        else {
            panic!()
        };
        assert_eq!(catch.error_type, SemanticValueType::Single);
        assert_eq!(catch.line_type, SemanticValueType::Single);
    }

    #[test]
    fn adapts_single_line_and_block_if_bodies() {
        let single = adapt_module(&rdgen_frontend::parse("if ready% then a% = 1\n").unwrap());
        let SemanticStatementKind::Line(body) = &single.statements[0].kind else {
            panic!()
        };
        assert!(
            matches!(body[0].kind, SemanticStatementKind::If { block: false, ref then_body, ref else_body, .. } if then_body.len() == 1 && else_body.is_empty()),
            "{body:#?}"
        );
        let single_else = adapt_module(
            &rdgen_frontend::parse("if ready% then print \"then\" else print \"else\"\n").unwrap(),
        );
        let SemanticStatementKind::Line(body) = &single_else.statements[0].kind else {
            panic!()
        };
        assert!(
            matches!(body[0].kind, SemanticStatementKind::If { block: false, ref then_body, ref else_body, .. } if then_body.len() == 1 && else_body.len() == 1),
            "single-line ELSE was not retained in typed IR: {body:#?}"
        );
        let block = adapt_module(
            &rdgen_frontend::parse("if ready% then\na% = 1\nelse\na% = 2\nend if\n").unwrap(),
        );
        let SemanticStatementKind::Line(body) = &block.statements[0].kind else {
            panic!()
        };
        assert!(
            matches!(body[0].kind, SemanticStatementKind::If { block: true, ref then_body, ref else_body, .. } if then_body.len() == 1 && else_body.len() == 1)
        );
    }

    #[test]
    fn adapts_while_body_recursively() {
        let module =
            adapt_module(&rdgen_frontend::parse("while ready%\ncount% += 1\nend while\n").unwrap());
        let SemanticStatementKind::Line(body) = &module.statements[0].kind else {
            panic!()
        };
        assert!(
            matches!(body[0].kind, SemanticStatementKind::While { ref body, .. } if body.len() == 1)
        );
    }

    #[test]
    fn adapts_to_and_downto_bounds_explicitly() {
        let to =
            adapt_module(&rdgen_frontend::parse("for index% = 1 to 10 step 2\nend for\n").unwrap());
        let SemanticStatementKind::Line(body) = &to.statements[0].kind else {
            panic!()
        };
        assert!(
            matches!(
                body[0].kind,
                SemanticStatementKind::For {
                    bounds: ForBounds::To { step: Some(_), .. },
                    variable_type: SemanticValueType::Integer,
                    ..
                }
            ),
            "{body:#?}"
        );
        let down =
            adapt_module(&rdgen_frontend::parse("for index% = 10 downto 1\nend for\n").unwrap());
        let SemanticStatementKind::Line(body) = &down.statements[0].kind else {
            panic!()
        };
        assert!(matches!(
            body[0].kind,
            SemanticStatementKind::For {
                bounds: ForBounds::Downto { step: -1, .. },
                ..
            }
        ));
    }

    #[test]
    fn adapts_pre_and_post_condition_do_loops() {
        let pre =
            adapt_module(&rdgen_frontend::parse("do while ready%\ncount% += 1\nend do\n").unwrap());
        let SemanticStatementKind::Line(body) = &pre.statements[0].kind else {
            panic!()
        };
        assert!(matches!(
            body[0].kind,
            SemanticStatementKind::Do {
                pre_condition: Some(LoopCondition {
                    kind: LoopConditionKind::While,
                    ..
                }),
                post_condition: None,
                ..
            }
        ));
        let post =
            adapt_module(&rdgen_frontend::parse("do\ncount% += 1\nloop until done%\n").unwrap());
        let SemanticStatementKind::Line(body) = &post.statements[0].kind else {
            panic!()
        };
        assert!(matches!(
            body[0].kind,
            SemanticStatementKind::Do {
                pre_condition: None,
                post_condition: Some(LoopCondition {
                    kind: LoopConditionKind::Until,
                    ..
                }),
                ..
            }
        ));
    }

    #[test]
    fn adapts_select_case_values() {
        let module = adapt_module(
            &rdgen_frontend::parse(
                "select case value%\ncase 1 to 3, is >= 9\nresult% = 1\nend select\n",
            )
            .unwrap(),
        );
        let SemanticStatementKind::Line(body) = &module.statements[0].kind else {
            panic!()
        };
        let SemanticStatementKind::SelectCase {
            cases, else_body, ..
        } = &body[0].kind
        else {
            panic!()
        };
        assert_eq!(cases.len(), 1);
        assert!(matches!(
            cases[0].values[0],
            CaseValue::Value {
                range_end: Some(_),
                ..
            }
        ));
        assert!(matches!(
            cases[0].values[1],
            CaseValue::Comparison {
                operator: ComparisonOperator::GreaterOrEqual,
                ..
            }
        ));
        assert!(else_body.is_empty());
    }

    #[test]
    fn adapts_select_case_else_body_separately_from_case_values() {
        let module = adapt_module(
            &rdgen_frontend::parse(
                "select case value%\ncase 1\nresult% = 1\ncase else\nresult% = 2\nend select\n",
            )
            .unwrap(),
        );
        let SemanticStatementKind::Line(body) = &module.statements[0].kind else {
            panic!()
        };
        let SemanticStatementKind::SelectCase {
            cases, else_body, ..
        } = &body[0].kind
        else {
            panic!("unexpected typed select node: {:#?}", body[0])
        };
        assert_eq!(cases.len(), 1);
        assert_eq!(cases[0].values.len(), 1);
        assert_eq!(cases[0].body.len(), 1);
        assert_eq!(else_body.len(), 1);
        let SemanticStatementKind::Line(else_statements) = &else_body[0].kind else {
            panic!()
        };
        assert!(matches!(
            else_statements[0].kind,
            SemanticStatementKind::Assignment { .. }
        ));
    }

    #[test]
    fn adapts_try_catch_finally_bindings() {
        let module = adapt_module(&rdgen_frontend::parse("try\nwork% = 1\ncatch err%, line%, source$\nwork% = 2\nfinally\nwork% = 3\nend try\n").unwrap());
        let SemanticStatementKind::Line(body) = &module.statements[0].kind else {
            panic!()
        };
        let SemanticStatementKind::Try {
            catch: Some(catch),
            finally_body,
            ..
        } = &body[0].kind
        else {
            panic!()
        };
        assert_eq!(catch.error, "err%");
        assert_eq!(catch.line, "line%");
        assert_eq!(catch.source.as_deref(), Some("source$"));
        assert_eq!(finally_body.len(), 1);
    }
}
