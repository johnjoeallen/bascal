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
    pub header: Option<ModuleHeader>,
    pub dependencies: Vec<Dependency>,
    pub records: Vec<Record>,
    pub callables: Vec<CallableSignature>,
    pub statements: Vec<SemanticStatement>,
}

/// Name visibility facts derived from the generated semantic module.  This is
/// deliberately a fact table rather than a backend-specific symbol table:
/// code generators can use it for collision checks without re-walking parser
/// nodes or re-inferring names from emitted syntax.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SemanticNameScopes {
    pub global_names: BTreeSet<String>,
    pub callable_globals: HashMap<String, BTreeSet<String>>,
}

impl SemanticModule {
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
                    SemanticStatementKind::If { then_body, else_body, .. } => {
                        visit(then_body, names);
                        visit(else_body, names);
                    }
                    SemanticStatementKind::SelectCase { cases, else_body, .. } => {
                        for case in cases {
                            visit(&case.body, names);
                        }
                        visit(else_body, names);
                    }
                    SemanticStatementKind::Try { body, catch, finally_body } => {
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
        fn visit(
            statements: &[SemanticStatement],
            types: &mut BTreeMap<String, SemanticValueType>,
        ) {
            for statement in statements {
                match &statement.kind {
                    SemanticStatementKind::Const { name, value } => {
                        let suffix = name.name.chars().last().and_then(|character| {
                            SemanticValueType::from_suffix(Some(character)).suffix()
                        });
                        let value_type = suffix
                            .map(|character| SemanticValueType::from_suffix(Some(character)))
                            .unwrap_or(match value.value_type {
                                SemanticValueType::Unknown | SemanticValueType::Boolean => {
                                    SemanticValueType::Integer
                                }
                                value_type => value_type,
                            });
                        types.insert(name.name.clone(), value_type);
                    }
                    SemanticStatementKind::Line(body)
                    | SemanticStatementKind::While { body, .. }
                    | SemanticStatementKind::For { body, .. }
                    | SemanticStatementKind::Do { body, .. } => visit(body, types),
                    SemanticStatementKind::If { then_body, else_body, .. } => {
                        visit(then_body, types);
                        visit(else_body, types);
                    }
                    SemanticStatementKind::SelectCase { cases, else_body, .. } => {
                        for case in cases {
                            visit(&case.body, types);
                        }
                        visit(else_body, types);
                    }
                    SemanticStatementKind::Try { body, catch, finally_body } => {
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

        let mut types = BTreeMap::new();
        visit(&self.statements, &mut types);
        for callable in &self.callables {
            visit(&callable.body, &mut types);
        }
        types
    }

    /// Return named `ON ERROR GOTO` targets in source order. Numeric disable
    /// sentinels are intentionally omitted.
    pub fn error_handler_targets(&self) -> Vec<String> {
        fn visit(statements: &[SemanticStatement], targets: &mut Vec<String>) {
            for statement in statements {
                match &statement.kind {
                    SemanticStatementKind::OnErrorGoto(ErrorHandlerTarget::Label(target)) => {
                        if !targets.iter().any(|name| name.eq_ignore_ascii_case(&target.name)) {
                            targets.push(target.name.clone());
                        }
                    }
                    SemanticStatementKind::Line(body)
                    | SemanticStatementKind::While { body, .. }
                    | SemanticStatementKind::For { body, .. }
                    | SemanticStatementKind::Do { body, .. } => visit(body, targets),
                    SemanticStatementKind::If { then_body, else_body, .. } => {
                        visit(then_body, targets);
                        visit(else_body, targets);
                    }
                    SemanticStatementKind::SelectCase { cases, else_body, .. } => {
                        for case in cases {
                            visit(&case.body, targets);
                        }
                        visit(else_body, targets);
                    }
                    SemanticStatementKind::Try { body, catch, finally_body } => {
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
                        if !targets.iter().any(|name| name.eq_ignore_ascii_case(&target.name)) {
                            targets.push(target.name.clone());
                        }
                    }
                    SemanticStatementKind::Line(body)
                    | SemanticStatementKind::While { body, .. }
                    | SemanticStatementKind::For { body, .. }
                    | SemanticStatementKind::Do { body, .. } => visit(body, targets),
                    SemanticStatementKind::If { then_body, else_body, .. } => {
                        visit(then_body, targets);
                        visit(else_body, targets);
                    }
                    SemanticStatementKind::SelectCase { cases, else_body, .. } => {
                        for case in cases { visit(&case.body, targets); }
                        visit(else_body, targets);
                    }
                    SemanticStatementKind::Try { body, catch, finally_body } => {
                        visit(body, targets);
                        if let Some(catch) = catch { visit(&catch.body, targets); }
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
            statements.iter().map(|statement| {
                let self_count = usize::from(matches!(
                    &statement.kind,
                    SemanticStatementKind::Throw(_)
                        | SemanticStatementKind::Open {
                            mode: OpenMode { kind: OpenModeKind::Input | OpenModeKind::Random | OpenModeKind::Binary, .. },
                            ..
                        }
                ));
                let nested = match &statement.kind {
                    SemanticStatementKind::Line(body)
                    | SemanticStatementKind::While { body, .. }
                    | SemanticStatementKind::For { body, .. }
                    | SemanticStatementKind::Do { body, .. } => count(body),
                    SemanticStatementKind::If { then_body, else_body, .. } => count(then_body) + count(else_body),
                    SemanticStatementKind::SelectCase { cases, else_body, .. } => {
                        cases.iter().map(|case| count(&case.body)).sum::<usize>() + count(else_body)
                    }
                    SemanticStatementKind::Try { body, catch, finally_body } => {
                        count(body) + catch.as_ref().map_or(0, |catch| count(&catch.body)) + count(finally_body)
                    }
                    _ => 0,
                };
                self_count + nested
            }).sum()
        }
        count(&self.statements)
    }

    /// Count top-level TRY/CATCH blocks in source order.
    pub fn top_level_try_catch_count(&self) -> usize {
        fn count(statements: &[SemanticStatement]) -> usize {
            statements.iter().map(|statement| {
                let self_count = usize::from(matches!(&statement.kind, SemanticStatementKind::Try { .. }));
                let nested = match &statement.kind {
                    SemanticStatementKind::Line(body)
                    | SemanticStatementKind::While { body, .. }
                    | SemanticStatementKind::For { body, .. }
                    | SemanticStatementKind::Do { body, .. } => count(body),
                    SemanticStatementKind::If { then_body, else_body, .. } => count(then_body) + count(else_body),
                    SemanticStatementKind::SelectCase { cases, else_body, .. } => {
                        cases.iter().map(|case| count(&case.body)).sum::<usize>() + count(else_body)
                    }
                    SemanticStatementKind::Try { body, catch, finally_body } => {
                        count(body) + catch.as_ref().map_or(0, |catch| count(&catch.body)) + count(finally_body)
                    }
                    _ => 0,
                };
                self_count + nested
            }).sum()
        }
        count(&self.statements)
    }

    /// Count top-level GOSUB statements in source traversal order.
    pub fn top_level_gosub_count(&self) -> usize {
        fn count(statements: &[SemanticStatement]) -> usize {
            statements.iter().map(|statement| {
                let self_count = usize::from(matches!(&statement.kind, SemanticStatementKind::Gosub(_)));
                let nested = match &statement.kind {
                    SemanticStatementKind::Line(body)
                    | SemanticStatementKind::While { body, .. }
                    | SemanticStatementKind::For { body, .. }
                    | SemanticStatementKind::Do { body, .. } => count(body),
                    SemanticStatementKind::If { then_body, else_body, .. } => count(then_body) + count(else_body),
                    SemanticStatementKind::SelectCase { cases, else_body, .. } => {
                        cases.iter().map(|case| count(&case.body)).sum::<usize>() + count(else_body)
                    }
                    SemanticStatementKind::Try { body, catch, finally_body } => {
                        count(body) + catch.as_ref().map_or(0, |catch| count(&catch.body)) + count(finally_body)
                    }
                    _ => 0,
                };
                self_count + nested
            }).sum()
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
                    SemanticStatementKind::If { then_body, else_body, .. } => {
                        visit(then_body, names);
                        visit(else_body, names);
                    }
                    SemanticStatementKind::SelectCase { cases, else_body, .. } => {
                        for case in cases {
                            visit(&case.body, names);
                        }
                        visit(else_body, names);
                    }
                    SemanticStatementKind::Try { body, catch, finally_body } => {
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

    /// Return module-scope array names and ranks, including declarations
    /// nested in top-level control-flow bodies. Callable-local arrays are
    /// excluded because backends analyze each callable scope separately.
    pub fn top_level_array_ranks(&self) -> HashMap<String, usize> {
        fn visit(statements: &[SemanticStatement], ranks: &mut HashMap<String, usize>) {
            for statement in statements {
                match &statement.kind {
                    SemanticStatementKind::Dim(declarations) => {
                        for declaration in declarations.iter().filter(|declaration| declaration.array_axes > 0) {
                            ranks.insert(declaration.name.to_ascii_lowercase(), declaration.array_axes);
                        }
                    }
                    SemanticStatementKind::Line(body)
                    | SemanticStatementKind::While { body, .. }
                    | SemanticStatementKind::For { body, .. }
                    | SemanticStatementKind::Do { body, .. } => visit(body, ranks),
                    SemanticStatementKind::If { then_body, else_body, .. } => {
                        visit(then_body, ranks);
                        visit(else_body, ranks);
                    }
                    SemanticStatementKind::SelectCase { cases, else_body, .. } => {
                        for case in cases {
                            visit(&case.body, ranks);
                        }
                        visit(else_body, ranks);
                    }
                    SemanticStatementKind::Try { body, catch, finally_body } => {
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
                    SemanticStatementKind::If { then_body, else_body, .. } => {
                        visit(then_body, types);
                        visit(else_body, types);
                    }
                    SemanticStatementKind::SelectCase { cases, else_body, .. } => {
                        for case in cases {
                            visit(&case.body, types);
                        }
                        visit(else_body, types);
                    }
                    SemanticStatementKind::Try { body, catch, finally_body } => {
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
                        names.extend(bindings.iter().map(|binding| binding.name.to_ascii_lowercase()));
                    }
                    SemanticStatementKind::Line(body)
                    | SemanticStatementKind::While { body, .. }
                    | SemanticStatementKind::For { body, .. }
                    | SemanticStatementKind::Do { body, .. } => visit(body, names),
                    SemanticStatementKind::If { then_body, else_body, .. } => {
                        visit(then_body, names);
                        visit(else_body, names);
                    }
                    SemanticStatementKind::SelectCase { cases, else_body, .. } => {
                        for case in cases {
                            visit(&case.body, names);
                        }
                        visit(else_body, names);
                    }
                    SemanticStatementKind::Try { body, catch, finally_body } => {
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

    /// Whether any CATCH binding captures a source filename.
    pub fn uses_catch_source_var(&self) -> bool {
        fn visit(statements: &[SemanticStatement]) -> bool {
            statements.iter().any(|statement| match &statement.kind {
                SemanticStatementKind::Try { body, catch, finally_body } => {
                    catch.as_ref().is_some_and(|binding| binding.source.is_some())
                        || visit(body)
                        || catch.as_ref().is_some_and(|binding| visit(&binding.body))
                        || visit(finally_body)
                }
                SemanticStatementKind::Line(body)
                | SemanticStatementKind::While { body, .. }
                | SemanticStatementKind::For { body, .. }
                | SemanticStatementKind::Do { body, .. } => visit(body),
                SemanticStatementKind::If { then_body, else_body, .. } => {
                    visit(then_body) || visit(else_body)
                }
                SemanticStatementKind::SelectCase { cases, else_body, .. } => {
                    cases.iter().any(|case| visit(&case.body)) || visit(else_body)
                }
                _ => false,
            })
        }

        visit(&self.statements) || self.callables.iter().any(|callable| visit(&callable.body))
    }

    /// Whether a statement matching `predicate` occurs anywhere in the module,
    /// including callable bodies and nested control-flow blocks.
    pub fn has_statement(
        &self,
        predicate: impl Fn(&SemanticStatementKind) -> bool + Copy,
    ) -> bool {
        fn visit(
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
                    | SemanticStatementKind::Do { body, .. } => visit(body, predicate),
                    SemanticStatementKind::If { then_body, else_body, .. } => {
                        visit(then_body, predicate) || visit(else_body, predicate)
                    }
                    SemanticStatementKind::SelectCase { cases, else_body, .. } => {
                        cases.iter().any(|case| visit(&case.body, predicate))
                            || visit(else_body, predicate)
                    }
                    SemanticStatementKind::Try { body, catch, finally_body } => {
                        visit(body, predicate)
                            || catch.as_ref().is_some_and(|binding| visit(&binding.body, predicate))
                            || visit(finally_body, predicate)
                    }
                    _ => false,
                }
            })
        }

        visit(&self.statements, predicate)
            || self.callables.iter().any(|callable| visit(&callable.body, predicate))
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
                ExpressionKind::Literal(value) => value.parse().ok(),
                ExpressionKind::Unary { operator, operand } if operator == "-" => {
                    eval(operand, definitions, depth + 1).map(|value| -value)
                }
                ExpressionKind::Binary { left, operator, right } => {
                    let left = eval(left, definitions, depth + 1)?;
                    let right = eval(right, definitions, depth + 1)?;
                    match operator.as_str() {
                        "+" => left.checked_add(right),
                        "-" => left.checked_sub(right),
                        "*" => left.checked_mul(right),
                        "/" if right != 0 => Some(left / right),
                        _ => None,
                    }
                }
                ExpressionKind::Name(name) => {
                    let key = name.to_ascii_lowercase();
                    definitions.get(&key).and_then(|value| eval(value, definitions, depth + 1))
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
                    SemanticStatementKind::Const { name, value } => {
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
                    SemanticStatementKind::If { then_body, else_body, .. } => {
                        collect(then_body, definitions);
                        collect(else_body, definitions);
                    }
                    SemanticStatementKind::SelectCase { cases, else_body, .. } => {
                        for case in cases {
                            collect(&case.body, definitions);
                        }
                        collect(else_body, definitions);
                    }
                    SemanticStatementKind::Try { body, catch, finally_body } => {
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
        for callable in &self.callables {
            let key = callable.name.to_ascii_lowercase();
            let mut globals = BTreeSet::new();
            collect_semantic_global_declarations(&callable.body, &mut globals);
            scopes.callable_globals.insert(key, globals);
        }
        scopes
    }

    /// Merge a dependency module ahead of this module's executable content.
    /// This mirrors the driver's legacy dependency order while preserving the
    /// root module header and source span.
    pub fn prepend_dependency(&mut self, dependency: SemanticModule) {
        let mut dependencies = dependency.dependencies;
        dependencies.append(&mut self.dependencies);
        self.dependencies = dependencies;
        self.records.splice(0..0, dependency.records);
        self.callables.splice(0..0, dependency.callables);
        let mut statements = dependency.statements;
        statements.append(&mut self.statements);
        self.statements = statements;
    }

    /// Whether this module contains structured error handling.  This is a
    /// backend capability fact, so targets need not rediscover `try` nodes
    /// from parser-era statements.
    pub fn contains_try(&self) -> bool {
        fn statements_contain_try(statements: &[SemanticStatement]) -> bool {
            statements.iter().any(|statement| match &statement.kind {
                SemanticStatementKind::Try { .. } => true,
                SemanticStatementKind::If { then_body, else_body, .. } => {
                    statements_contain_try(then_body) || statements_contain_try(else_body)
                }
                SemanticStatementKind::While { body, .. }
                | SemanticStatementKind::Line(body)
                | SemanticStatementKind::For { body, .. }
                | SemanticStatementKind::Do { body, .. } => statements_contain_try(body),
                SemanticStatementKind::SelectCase { cases, else_body, .. } => {
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
        let Some(callable) = self.callables.iter().find(|callable| callable.name.eq_ignore_ascii_case(name)) else {
            return SemanticValueType::Unknown;
        };
        callable
            .result_type
            .as_deref()
            .and_then(|suffix| suffix.chars().next())
            .map_or(SemanticValueType::Unknown, |suffix| SemanticValueType::from_suffix(Some(suffix)))
    }

    pub fn annotate_expression_types(&self, expression: &mut Expression) {
        match &mut expression.kind {
            ExpressionKind::Call { name, arguments } => {
                for argument in arguments { self.annotate_expression_types(argument); }
                expression.value_type = self.callable_value_type(name);
            }
            ExpressionKind::Index { index, .. } => self.annotate_expression_types(index),
            ExpressionKind::Member { base, arguments, .. } => {
                if let Some(base) = base { self.annotate_expression_types(base); }
                if let Some(arguments) = arguments {
                    for argument in arguments { self.annotate_expression_types(argument); }
                }
            }
            ExpressionKind::Parenthesized(inner) | ExpressionKind::Unary { operand: inner, .. } => {
                self.annotate_expression_types(inner);
                expression.value_type = inner.value_type;
            }
            ExpressionKind::Binary { left, right, .. } => {
                self.annotate_expression_types(left);
                self.annotate_expression_types(right);
                expression.value_type = left.value_type;
            }
            ExpressionKind::RecordLiteral(fields) | ExpressionKind::PartialRecordLiteral(fields) => {
                for field in fields { self.annotate_expression_types(&mut field.value); }
            }
            ExpressionKind::Name(_) | ExpressionKind::Literal(_) | ExpressionKind::Boolean(_) => {}
        }
    }

    pub fn annotate_statement_types(&self, statements: &mut [SemanticStatement]) {
        for statement in statements {
            match &mut statement.kind {
                SemanticStatementKind::Assignment { target, value, .. } => {
                    self.annotate_expression_types(target);
                    self.annotate_expression_types(value);
                }
                SemanticStatementKind::Expression(expression)
                | SemanticStatementKind::Error(expression)
                | SemanticStatementKind::OptionBase(expression)
                | SemanticStatementKind::Kill(expression)
                | SemanticStatementKind::Close(expression) => self.annotate_expression_types(expression),
                SemanticStatementKind::Return(ReturnValue::Value(expression))
                | SemanticStatementKind::Throw(ThrowValue::Value(expression)) => self.annotate_expression_types(expression),
                SemanticStatementKind::If { condition, then_body, else_body, .. } => {
                    self.annotate_expression_types(condition);
                    self.annotate_statement_types(then_body);
                    self.annotate_statement_types(else_body);
                }
                SemanticStatementKind::While { condition, body } => {
                    self.annotate_expression_types(condition);
                    self.annotate_statement_types(body);
                }
                SemanticStatementKind::Line(body)
                | SemanticStatementKind::For { body, .. }
                | SemanticStatementKind::Do { body, .. } => self.annotate_statement_types(body),
                SemanticStatementKind::Try { body, catch, finally_body } => {
                    self.annotate_statement_types(body);
                    if let Some(catch) = catch { self.annotate_statement_types(&mut catch.body); }
                    self.annotate_statement_types(finally_body);
                }
                SemanticStatementKind::Print { destination, tokens } => {
                    match destination {
                        PrintDestination::Standard { .. } => {}
                        PrintDestination::Using { format, .. } => self.annotate_expression_types(format),
                        PrintDestination::Channel { channel, using, .. } => {
                            self.annotate_expression_types(channel);
                            if let Some(using) = using { self.annotate_expression_types(using); }
                        }
                    }
                    for token in tokens { if let PrintToken::Expression(expression) = token { self.annotate_expression_types(expression); } }
                }
                SemanticStatementKind::Lprint { using, tokens } => {
                    if let Some(using) = using { self.annotate_expression_types(using); }
                    for token in tokens { if let PrintToken::Expression(expression) = token { self.annotate_expression_types(expression); } }
                }
                SemanticStatementKind::Input { source, targets } => {
                    if let InputSource::Channel(channel) = source { self.annotate_expression_types(channel); }
                    for target in targets { self.annotate_expression_types(target); }
                }
                SemanticStatementKind::Open { path, channel, length, .. } => {
                    self.annotate_expression_types(path); self.annotate_expression_types(channel);
                    if let Some(length) = length { self.annotate_expression_types(length); }
                }
                SemanticStatementKind::Write { channel, values } => {
                    self.annotate_expression_types(channel);
                    if let WriteValues::Values(values) = values { for value in values { self.annotate_expression_types(value); } }
                }
                _ => {}
            }
        }
    }
}

fn collect_semantic_expression(expression: &Expression, names: &mut BTreeSet<String>) {
    match &expression.kind {
        ExpressionKind::Name(name) => {
            names.insert(name.to_ascii_lowercase());
        }
        ExpressionKind::Call { name, arguments } => {
            names.insert(name.to_ascii_lowercase());
            for argument in arguments { collect_semantic_expression(argument, names); }
        }
        ExpressionKind::Index { name, index } => {
            names.insert(name.to_ascii_lowercase());
            collect_semantic_expression(index, names);
        }
        ExpressionKind::Member { base, arguments, .. } => {
            if let Some(base) = base { collect_semantic_expression(base, names); }
            if let Some(arguments) = arguments {
                for argument in arguments { collect_semantic_expression(argument, names); }
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
            for field in fields { collect_semantic_expression(&field.value, names); }
        }
        ExpressionKind::Literal(_) | ExpressionKind::Boolean(_) => {}
    }
}

fn collect_semantic_statements(
    statements: &[SemanticStatement],
    names: &mut BTreeSet<String>,
) {
    for statement in statements {
        match &statement.kind {
            SemanticStatementKind::Line(body) => collect_semantic_statements(body, names),
            SemanticStatementKind::Assignment { target, value, .. } => {
                collect_semantic_expression(target, names);
                collect_semantic_expression(value, names);
            }
            SemanticStatementKind::MidAssign { target, start, length, value } => {
                collect_semantic_expression(target, names);
                collect_semantic_expression(start, names);
                if let Some(length) = length { collect_semantic_expression(length, names); }
                collect_semantic_expression(value, names);
            }
            SemanticStatementKind::Expression(expression)
            | SemanticStatementKind::Error(expression)
            | SemanticStatementKind::OptionBase(expression)
            | SemanticStatementKind::Kill(expression)
            | SemanticStatementKind::Close(expression) => collect_semantic_expression(expression, names),
            SemanticStatementKind::If { condition, then_body, else_body, .. } => {
                collect_semantic_expression(condition, names);
                collect_semantic_statements(then_body, names);
                collect_semantic_statements(else_body, names);
            }
            SemanticStatementKind::While { condition, body } => {
                collect_semantic_expression(condition, names);
                collect_semantic_statements(body, names);
            }
            SemanticStatementKind::For { variable, start, bounds, body } => {
                names.insert(variable.to_ascii_lowercase());
                collect_semantic_expression(start, names);
                match bounds {
                    ForBounds::To { limit, step } => {
                        collect_semantic_expression(limit, names);
                        if let Some(step) = step { collect_semantic_expression(step, names); }
                    }
                    ForBounds::Downto { limit, .. } => collect_semantic_expression(limit, names),
                }
                collect_semantic_statements(body, names);
            }
            SemanticStatementKind::Do { pre_condition, post_condition, body } => {
                for condition in pre_condition.iter().chain(post_condition.iter()) {
                    collect_semantic_expression(&condition.value, names);
                }
                collect_semantic_statements(body, names);
            }
            SemanticStatementKind::SelectCase { selector, cases, else_body } => {
                collect_semantic_expression(selector, names);
                for case in cases {
                    for value in &case.values {
                        match value {
                            CaseValue::Comparison { value, .. } => collect_semantic_expression(value, names),
                            CaseValue::Value { first, range_end, .. } => {
                                collect_semantic_expression(first, names);
                                if let Some(end) = range_end { collect_semantic_expression(end, names); }
                            }
                        }
                    }
                    collect_semantic_statements(&case.body, names);
                }
                collect_semantic_statements(else_body, names);
            }
            SemanticStatementKind::Try { body, catch, finally_body } => {
                collect_semantic_statements(body, names);
                if let Some(catch) = catch { collect_semantic_statements(&catch.body, names); }
                collect_semantic_statements(finally_body, names);
            }
            SemanticStatementKind::Return(ReturnValue::Value(value))
            | SemanticStatementKind::Throw(ThrowValue::Value(value)) => collect_semantic_expression(value, names),
            SemanticStatementKind::OnBranch { selector, .. } => collect_semantic_expression(selector, names),
            SemanticStatementKind::Erase(_) | SemanticStatementKind::Label(_) | SemanticStatementKind::Goto(_)
            | SemanticStatementKind::Gosub(_) | SemanticStatementKind::Resume(_)
            | SemanticStatementKind::OnErrorGoto(_) | SemanticStatementKind::Return(ReturnValue::Default)
            | SemanticStatementKind::Throw(ThrowValue::Bare) | SemanticStatementKind::Exit
            | SemanticStatementKind::Continue | SemanticStatementKind::Comment { .. }
            | SemanticStatementKind::Unsupported | SemanticStatementKind::Stop
            | SemanticStatementKind::Clear | SemanticStatementKind::Cls
            | SemanticStatementKind::Beep | SemanticStatementKind::System => {}
            SemanticStatementKind::Dim(declarations) => {
                for declaration in declarations { names.insert(declaration.name.to_ascii_lowercase()); }
            }
            SemanticStatementKind::Const { name, value } => {
                names.insert(name.name.to_ascii_lowercase());
                collect_semantic_expression(value, names);
            }
            SemanticStatementKind::Global(name) => {
                names.insert(name.name.to_ascii_lowercase());
            }
            _ => {}
        }
    }
}

fn collect_semantic_global_declarations(
    statements: &[SemanticStatement],
    globals: &mut BTreeSet<String>,
) {
    for statement in statements {
        match &statement.kind {
            SemanticStatementKind::Global(name) => {
                globals.insert(name.name.to_ascii_lowercase());
            }
            SemanticStatementKind::Line(body)
            | SemanticStatementKind::While { body, .. }
            | SemanticStatementKind::For { body, .. }
            | SemanticStatementKind::Do { body, .. } => {
                collect_semantic_global_declarations(body, globals);
            }
            SemanticStatementKind::If { then_body, else_body, .. } => {
                collect_semantic_global_declarations(then_body, globals);
                collect_semantic_global_declarations(else_body, globals);
            }
            SemanticStatementKind::SelectCase { cases, else_body, .. } => {
                for case in cases { collect_semantic_global_declarations(&case.body, globals); }
                collect_semantic_global_declarations(else_body, globals);
            }
            SemanticStatementKind::Try { body, catch, finally_body } => {
                collect_semantic_global_declarations(body, globals);
                if let Some(catch) = catch { collect_semantic_global_declarations(&catch.body, globals); }
                collect_semantic_global_declarations(finally_body, globals);
            }
            _ => {}
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModuleHeader {
    Program { name: String, shared: Option<String>, span: SourceSpan },
    Library { name: String, span: SourceSpan },
    Shared { name: String, span: SourceSpan },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dependency {
    pub kind: DependencyKind,
    pub path: String,
    pub span: SourceSpan,
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
    String { capacity: Option<String>, alignment: Option<StringAlignment>, span: SourceSpan },
    Int16 { span: SourceSpan },
    Int { span: SourceSpan },
    Int32 { span: SourceSpan },
    Float32 { span: SourceSpan },
    Float64 { span: SourceSpan },
    Record { name: String, span: SourceSpan },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CallableSignature { pub kind: CallableKind, pub name: String, pub name_span: SourceSpan, pub result_type: Option<String>, pub receiver: Option<String>, pub receiver_span: Option<SourceSpan>, pub parameters: Vec<Parameter>, pub body: Vec<SemanticStatement>, pub span: SourceSpan }
impl CallableSignature {
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
                    SemanticStatementKind::If { then_body, else_body, .. } => {
                        visit(then_body, ranks);
                        visit(else_body, ranks);
                    }
                    SemanticStatementKind::SelectCase { cases, else_body, .. } => {
                        for case in cases {
                            visit(&case.body, ranks);
                        }
                        visit(else_body, ranks);
                    }
                    SemanticStatementKind::Try { body, catch, finally_body } => {
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
                    SemanticStatementKind::If { then_body, else_body, .. } => {
                        visit(then_body, types);
                        visit(else_body, types);
                    }
                    SemanticStatementKind::SelectCase { cases, else_body, .. } => {
                        for case in cases {
                            visit(&case.body, types);
                        }
                        visit(else_body, types);
                    }
                    SemanticStatementKind::Try { body, catch, finally_body } => {
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
pub enum CallableKind { Function, Procedure, Method, FluentMethod, InlineMethod }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parameter { pub name: String, pub name_span: SourceSpan, pub type_suffix: Option<String>, pub passing: Option<Passing>, pub array_axes: usize, pub default: Option<Expression>, pub span: SourceSpan }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Passing { ByRef, ByVal }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SemanticValueType { Unknown, String, Integer, Long, Single, Double, Boolean }
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
pub struct Expression { pub kind: ExpressionKind, pub span: SourceSpan, pub value_type: SemanticValueType }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExpressionKind {
    Name(String), Literal(String), Boolean(bool), Parenthesized(Box<Expression>),
    Unary { operator: String, operand: Box<Expression> }, Binary { left: Box<Expression>, operator: String, right: Box<Expression> },
    Call { name: String, arguments: Vec<Expression> }, Index { name: String, index: Box<Expression> },
    Member { base: Option<Box<Expression>>, member: String, arguments: Option<Vec<Expression>> },
    RecordLiteral(Vec<FieldInitializer>), PartialRecordLiteral(Vec<FieldInitializer>),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldInitializer { pub name: String, pub value: Expression }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssignmentOperator { Assign, Add, Subtract, Multiply, Divide }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StringAlignment { LeftPad, Left, RightPad, Right }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SemanticStatement { pub kind: SemanticStatementKind, pub span: SourceSpan }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SemanticStatementKind {
    Line(Vec<SemanticStatement>), Assignment { target: Expression, operator: AssignmentOperator, value: Expression }, MidAssign { target: Expression, start: Expression, length: Option<Expression>, value: Expression }, Expression(Expression),
    If { condition: Expression, then_body: Vec<SemanticStatement>, else_body: Vec<SemanticStatement>, block: bool }, Unsupported,
    While { condition: Expression, body: Vec<SemanticStatement> },
    For { variable: String, start: Expression, bounds: ForBounds, body: Vec<SemanticStatement> },
    Do { pre_condition: Option<LoopCondition>, post_condition: Option<LoopCondition>, body: Vec<SemanticStatement> },
    SelectCase { selector: Expression, cases: Vec<CaseClause>, else_body: Vec<SemanticStatement> },
    Try { body: Vec<SemanticStatement>, catch: Option<CatchBinding>, finally_body: Vec<SemanticStatement> },
    Label(NamedReference), Comment { block: bool },
    Return(ReturnValue), Exit, Continue,
    Goto(NamedReference), Gosub(NamedReference), Resume(Option<ResumeTarget>),
    OnErrorGoto(ErrorHandlerTarget), OnBranch { selector: Expression, branch: BranchKind, targets: Vec<NamedReference> },
    Error(Expression), Throw(ThrowValue),
    OptionBase(Expression), Erase(Vec<NamedReference>),
    Print { destination: PrintDestination, tokens: Vec<PrintToken> },
    Input { source: InputSource, targets: Vec<Expression> },
    Write { channel: Expression, values: WriteValues },
    Open { path: Expression, mode: OpenMode, channel: Expression, length: Option<Expression> },
    Seek { channel: Expression, position: Expression },
    Kill(Expression),
    Rename { source: Expression, destination: Expression },
    Close(Expression),
    Data(Vec<Expression>),
    Read(Vec<Expression>),
    Restore(Option<NamedReference>),
    Dim(Vec<DimDeclaration>),
    Const { name: NamedReference, value: Expression },
    Global(NamedReference),
    Swap { left: Expression, right: Expression },
    Randomize(RandomizeSeed),
    Poke { address: Expression, value: Expression },
    Out { port: Expression, value: Expression },
    Width { channel: Option<Expression>, value: Expression },
    LineInput { channel: Expression, target: Expression },
    Get { channel: Expression, position: Option<FilePosition> },
    Put { channel: Expression, position: Option<FilePosition> },
    Lset { target: NamedReference, value: Expression },
    Rset { target: NamedReference, value: Expression },
    Locate { row: Expression, column: Expression },
    Color { foreground: Expression, background: Option<Expression> },
    Stop,
    Clear,
    Cls,
    Beep,
    System,
    Lprint { using: Option<Expression>, tokens: Vec<PrintToken> },
    FileDeclaration { name: NamedReference, record_type: Option<NamedReference>, path: Expression, mode: Option<FileModeKind> },
    Field { channel: Expression, bindings: Vec<FieldBinding> },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DimAxis {
    Inferred,
    Fixed(String),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DimDeclaration { pub name: String, pub array_axes: usize, pub dimensions: Vec<DimAxis>, pub type_annotation: Option<String>, pub span: SourceSpan }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FilePosition { pub position: Option<Expression>, pub record: Option<Expression> }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldBinding { pub length: Expression, pub name: String, pub name_span: SourceSpan, pub type_suffix: Option<String>, pub is_string: bool }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputPrompt { pub text: String, pub span: SourceSpan }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenMode { pub kind: OpenModeKind, pub span: SourceSpan }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpenModeKind { Input, Output, Append, Random, Binary }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileModeKind { Input, Output, Append }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputSource { Channel(Expression), Console(Option<InputPrompt>) }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WriteValues { Omitted, Values(Vec<Expression>) }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamedReference { pub name: String, pub span: SourceSpan }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PrintDestination {
    Standard { span: SourceSpan },
    Channel { channel: Expression, using: Option<Expression>, span: SourceSpan },
    Using { format: Expression, span: SourceSpan },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PrintToken { Comma { span: SourceSpan }, Semicolon { span: SourceSpan }, Expression(Expression) }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ForBounds { To { limit: Expression, step: Option<Expression> }, Downto { limit: Expression, step: i64 } }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoopCondition { pub kind: LoopConditionKind, pub value: Expression }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoopConditionKind { While, Until }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaseClause { pub values: Vec<CaseValue>, pub body: Vec<SemanticStatement>, pub span: SourceSpan }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CaseValue { Comparison { operator: ComparisonOperator, value: Expression, span: SourceSpan }, Value { first: Expression, range_end: Option<Expression>, span: SourceSpan } }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComparisonOperator { NotEqual, LessOrEqual, GreaterOrEqual, Equal, Less, Greater }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResumeTarget { Label(NamedReference), Next }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorHandlerTarget { Label(NamedReference), Disable }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BranchKind { Goto, Gosub }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ThrowValue { Bare, Value(Expression) }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReturnValue { Default, Value(Expression) }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RandomizeSeed { Default, Value(Expression) }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CatchBinding { pub error: String, pub error_span: SourceSpan, pub line: String, pub line_span: SourceSpan, pub filters: Vec<Expression>, pub source: Option<String>, pub source_span: Option<SourceSpan>, pub body: Vec<SemanticStatement> }

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
                let rdgen_frontend::ProgramDecl::ProgramDeclaration { name, shared, span } = declaration.as_ref();
                header = Some(ModuleHeader::Program {
                    name: identifier(name),
                    shared: shared.as_ref().map(|(_, name)| identifier(name)),
                    span: *span,
                });
            }
            rdgen_frontend::FileItem::LibraryDeclaration { declaration, .. } => {
                let rdgen_frontend::LibraryDecl::LibraryDeclaration { name, span } = declaration.as_ref();
                header = Some(ModuleHeader::Library { name: identifier(name), span: *span });
            }
            rdgen_frontend::FileItem::SharedDeclaration { declaration, .. } => {
                let rdgen_frontend::SharedDecl::SharedDeclaration { name, span } = declaration.as_ref();
                header = Some(ModuleHeader::Shared { name: identifier(name), span: *span });
            }
            rdgen_frontend::FileItem::RequireDeclaration { declaration, .. } => {
                let rdgen_frontend::RequireDecl::RequireDeclaration { path, span } = declaration.as_ref();
                dependencies.push(Dependency { kind: DependencyKind::Require, path: identifier(path), span: *span });
            }
            rdgen_frontend::FileItem::ImportDeclaration { declaration, .. } => {
                let rdgen_frontend::ImportDecl::ImportDeclaration { path, span } = declaration.as_ref();
                dependencies.push(Dependency { kind: DependencyKind::Import, path: identifier(path), span: *span });
            }
            rdgen_frontend::FileItem::Record { record, .. } => {
                let adapted = adapt_record(record);
                callables.extend(adapt_inline_methods(record, &adapted.name));
                records.push(adapted);
            }
            rdgen_frontend::FileItem::Subprogram { subprogram, .. } => callables.push(adapt_subprogram(subprogram)),
            rdgen_frontend::FileItem::Statement { statement, .. } => statements.push(adapt_top_level_statement(statement)),
        }
    }
    let mut module = SemanticModule { span: *span, header, dependencies, records, callables, statements };
    let facts = module.clone();
    facts.annotate_statement_types(&mut module.statements);
    for callable in &mut module.callables {
        facts.annotate_statement_types(&mut callable.body);
    }
    module
}

/// Parse source with the generated frontend and adapt it into semantic IR.
pub fn parse_and_adapt(source: &str) -> Result<SemanticModule, rdgen_frontend::ParseError> {
    rdgen_frontend::parse(source).map(|program| adapt_module(&program))
}

/// Convert a generated-parser failure into the compiler diagnostic shape.
pub fn parse_diagnostic(filename: impl Into<String>, error: &rdgen_frontend::ParseError) -> crate::diagnostics::Diagnostic {
    crate::diagnostics::Diagnostic::error(
        crate::diagnostics::SourcePos::new(filename, 1, error.position.saturating_add(1)),
        error.message.clone(),
    )
}

fn adapt_top_level_statement(statement: &rdgen_frontend::TopLevelStatement) -> SemanticStatement {
    match statement {
        rdgen_frontend::TopLevelStatement::Statement { statement, .. } => adapt_statement(statement),
        _ => SemanticStatement { kind: SemanticStatementKind::Unsupported, span: statement.span() },
    }
}

fn adapt_statement(statement: &rdgen_frontend::Statement) -> SemanticStatement {
    match statement {
        rdgen_frontend::Statement::Line { first, rest, span } => SemanticStatement { kind: SemanticStatementKind::Line(std::iter::once(first.as_ref()).chain(rest.iter().map(|(_, value)| value.as_ref())).map(adapt_statement_core).collect()), span: *span },
        _ => SemanticStatement { kind: SemanticStatementKind::Unsupported, span: statement.span() },
    }
}

fn adapt_statement_core(statement: &rdgen_frontend::StatementCore) -> SemanticStatement {
    match statement {
        rdgen_frontend::StatementCore::AssignmentOrExpression { statement, span } => match statement.as_ref() {
            rdgen_frontend::AssignmentOrExprStmt::MidAssignment { assignment, .. } => {
                let rdgen_frontend::MidAssign::MidAssign { target, start, length, value, .. } = assignment.as_ref();
                SemanticStatement { kind: SemanticStatementKind::MidAssign { target: adapt_expression(target), start: adapt_expression(start), length: length.as_ref().map(|(_, expression)| adapt_expression(expression)), value: adapt_expression(value) }, span: *span }
            }
            rdgen_frontend::AssignmentOrExprStmt::Assignment { target, operator, value, .. } => {
                let rdgen_frontend::AssignTarget::Target { value: target, .. } = target.as_ref();
                SemanticStatement { kind: SemanticStatementKind::Assignment { target: adapt_expression(target), operator: assignment_operator(operator), value: adapt_expression(value) }, span: *span }
            }
            rdgen_frontend::AssignmentOrExprStmt::Expression { value, .. } => SemanticStatement { kind: SemanticStatementKind::Expression(adapt_expression(value)), span: *span },
        },
        rdgen_frontend::StatementCore::If { if_statement, span } => adapt_if(if_statement, *span),
        rdgen_frontend::StatementCore::While { while_statement, span } => { let rdgen_frontend::WhileStmt::While { condition, body, .. } = while_statement.as_ref(); SemanticStatement { kind: SemanticStatementKind::While { condition: adapt_expression(condition), body: body.iter().map(|value| adapt_statement(value)).collect() }, span: *span } },
        rdgen_frontend::StatementCore::For { for_statement, span } => { let rdgen_frontend::ForStmt::For { variable, start, bounds, body, .. } = for_statement.as_ref(); SemanticStatement { kind: SemanticStatementKind::For { variable: typed_identifier(variable), start: adapt_expression(start), bounds: adapt_for_bounds(bounds), body: body.iter().map(|value| adapt_statement(value)).collect() }, span: *span } },
        rdgen_frontend::StatementCore::Do { do_statement, span } => { let rdgen_frontend::DoStmt::Do { pre_condition, body, terminator, .. } = do_statement.as_ref(); let post_condition = match terminator.as_ref() { rdgen_frontend::DoTerminator::Loop { post_condition, .. } => post_condition.as_deref().map(adapt_loop_condition), _ => None }; SemanticStatement { kind: SemanticStatementKind::Do { pre_condition: pre_condition.as_deref().map(adapt_loop_condition), post_condition, body: body.iter().map(|value| adapt_statement(value)).collect() }, span: *span } },
        rdgen_frontend::StatementCore::SelectCase { select_case, span } => { let rdgen_frontend::SelectCaseStmt::SelectCase { selector, cases, else_body, .. } = select_case.as_ref(); SemanticStatement { kind: SemanticStatementKind::SelectCase { selector: adapt_expression(selector), cases: cases.iter().map(|value| adapt_case_clause(value)).collect(), else_body: else_body.as_ref().map(|(_, _, body)| body.iter().map(|value| adapt_statement(value)).collect()).unwrap_or_default() }, span: *span } },
        rdgen_frontend::StatementCore::Try { try_statement, span } => { let rdgen_frontend::TryStmt::Try { body, catch_clause, finally_clause, .. } = try_statement.as_ref(); SemanticStatement { kind: SemanticStatementKind::Try { body: body.iter().map(|value| adapt_statement(value)).collect(), catch: catch_clause.as_deref().map(adapt_catch), finally_body: finally_clause.as_ref().map(|(_, body)| body.iter().map(|value| adapt_statement(value)).collect()).unwrap_or_default() }, span: *span } },
        rdgen_frontend::StatementCore::Label { label, span } => { let rdgen_frontend::LabelStmt::Label { name, .. } = label.as_ref(); SemanticStatement { kind: SemanticStatementKind::Label(NamedReference { name: identifier(name), span: name.span() }), span: *span } },
        rdgen_frontend::StatementCore::Comment { comment, span } => { let block = matches!(comment.as_ref(), rdgen_frontend::CommentStmt::BlockComment { .. }); SemanticStatement { kind: SemanticStatementKind::Comment { block }, span: *span } },
        rdgen_frontend::StatementCore::Return { return_statement, span } => { let rdgen_frontend::ReturnStmt::Return { value, .. } = return_statement.as_ref(); SemanticStatement { kind: SemanticStatementKind::Return(value.as_deref().map(adapt_expression).map(ReturnValue::Value).unwrap_or(ReturnValue::Default)), span: *span } },
        rdgen_frontend::StatementCore::Exit { span, .. } => SemanticStatement { kind: SemanticStatementKind::Exit, span: *span },
        rdgen_frontend::StatementCore::Continue { span, .. } => SemanticStatement { kind: SemanticStatementKind::Continue, span: *span },
        rdgen_frontend::StatementCore::Goto { goto, span } => { let rdgen_frontend::GotoStmt::Goto { target, .. } = goto.as_ref(); SemanticStatement { kind: SemanticStatementKind::Goto(NamedReference { name: identifier(target), span: target.span() }), span: *span } },
        rdgen_frontend::StatementCore::Gosub { gosub, span } => { let rdgen_frontend::GosubStmt::Gosub { target, .. } = gosub.as_ref(); SemanticStatement { kind: SemanticStatementKind::Gosub(NamedReference { name: identifier(target), span: target.span() }), span: *span } },
        rdgen_frontend::StatementCore::Resume { resume, span } => { let rdgen_frontend::ResumeStmt::Resume { target, .. } = resume.as_ref(); let target = target.as_deref().map(|target| match target { rdgen_frontend::ResumeTarget::Label { label, .. } => ResumeTarget::Label(NamedReference { name: identifier(label), span: label.span() }), rdgen_frontend::ResumeTarget::Next { .. } => ResumeTarget::Next }); SemanticStatement { kind: SemanticStatementKind::Resume(target), span: *span } },
        rdgen_frontend::StatementCore::OnErrorGoto { on_error, span } => { let rdgen_frontend::OnErrorGotoStmt::OnErrorGoto { target, .. } = on_error.as_ref(); let target = match target.as_ref() { rdgen_frontend::ErrorTarget::Label { label, .. } => ErrorHandlerTarget::Label(NamedReference { name: identifier(label), span: label.span() }), rdgen_frontend::ErrorTarget::Disable { .. } => ErrorHandlerTarget::Disable }; SemanticStatement { kind: SemanticStatementKind::OnErrorGoto(target), span: *span } },
        rdgen_frontend::StatementCore::OnBranch { on_branch, span } => { let rdgen_frontend::OnBranchStmt::OnBranch { selector, branch, first, rest, .. } = on_branch.as_ref(); let branch = match branch.as_ref() { rdgen_frontend::BranchKind::Goto { .. } => BranchKind::Goto, rdgen_frontend::BranchKind::Gosub { .. } => BranchKind::Gosub }; let targets = std::iter::once(first.as_ref()).chain(rest.iter().map(|(_, value)| value.as_ref())).map(|target| NamedReference { name: identifier(target), span: target.span() }).collect(); SemanticStatement { kind: SemanticStatementKind::OnBranch { selector: adapt_expression(selector), branch, targets }, span: *span } },
        rdgen_frontend::StatementCore::Error { error, span } => { let rdgen_frontend::ErrorStmt::Error { value, .. } = error.as_ref(); SemanticStatement { kind: SemanticStatementKind::Error(adapt_expression(value)), span: *span } },
        rdgen_frontend::StatementCore::Throw { throw, span } => { let rdgen_frontend::ThrowStmt::Throw { value, .. } = throw.as_ref(); SemanticStatement { kind: SemanticStatementKind::Throw(value.as_deref().map(adapt_expression).map(ThrowValue::Value).unwrap_or(ThrowValue::Bare)), span: *span } },
        rdgen_frontend::StatementCore::OptionBase { option_base, span } => { let rdgen_frontend::OptionBaseStmt::OptionBase { value, .. } = option_base.as_ref(); SemanticStatement { kind: SemanticStatementKind::OptionBase(adapt_expression(value)), span: *span } },
        rdgen_frontend::StatementCore::Erase { erase, span } => { let rdgen_frontend::EraseStmt::Erase { first, rest, .. } = erase.as_ref(); SemanticStatement { kind: SemanticStatementKind::Erase(std::iter::once(first.as_ref()).chain(rest.iter().map(|(_, value)| value.as_ref())).map(|value| NamedReference { name: identifier(value), span: value.span() }).collect()), span: *span } },
        rdgen_frontend::StatementCore::Print { print, span } => { let rdgen_frontend::PrintStmt::Print { destination, tokens, .. } = print.as_ref(); SemanticStatement { kind: SemanticStatementKind::Print { destination: adapt_print_destination(destination), tokens: tokens.iter().map(|value| adapt_print_token(value)).collect() }, span: *span } },
        rdgen_frontend::StatementCore::Input { input, span } => { let (source, first, rest) = match input.as_ref() { rdgen_frontend::InputStmt::ChannelInput { channel, first, rest, .. } => (InputSource::Channel(adapt_expression(channel)), first, rest), rdgen_frontend::InputStmt::ConsoleInput { prompt, first, rest, .. } => (InputSource::Console(prompt.as_ref().map(|(value, _)| InputPrompt { text: token_text_string(value), span: value.span() })), first, rest) }; SemanticStatement { kind: SemanticStatementKind::Input { source, targets: std::iter::once(adapt_expression(first)).chain(rest.iter().map(|(_, value)| adapt_expression(value))).collect() }, span: *span } },
        rdgen_frontend::StatementCore::Write { write, span } => { let rdgen_frontend::WriteStmt::Write { channel, values, .. } = write.as_ref(); let values = values.as_ref().map(|(_, first, rest)| WriteValues::Values(std::iter::once(adapt_expression(first)).chain(rest.iter().map(|(_, value)| adapt_expression(value))).collect())).unwrap_or(WriteValues::Omitted); SemanticStatement { kind: SemanticStatementKind::Write { channel: adapt_expression(channel), values }, span: *span } },
        rdgen_frontend::StatementCore::Open { open, span } => { let rdgen_frontend::OpenStmt::Open { path, mode, channel, length, .. } = open.as_ref(); SemanticStatement { kind: SemanticStatementKind::Open { path: adapt_expression(path), mode: adapt_open_mode(mode), channel: adapt_expression(channel), length: length.as_ref().map(|(_, _, value)| adapt_expression(value)) }, span: *span } },
        rdgen_frontend::StatementCore::Seek { seek, span } => { let rdgen_frontend::SeekStmt::Seek { channel, position, .. } = seek.as_ref(); SemanticStatement { kind: SemanticStatementKind::Seek { channel: adapt_expression(channel), position: adapt_expression(position) }, span: *span } },
        rdgen_frontend::StatementCore::Kill { kill, span } => { let rdgen_frontend::KillStmt::Kill { path, .. } = kill.as_ref(); SemanticStatement { kind: SemanticStatementKind::Kill(adapt_expression(path)), span: *span } },
        rdgen_frontend::StatementCore::Rename { rename, span } => { let rdgen_frontend::NameStmt::Rename { source, destination, .. } = rename.as_ref(); SemanticStatement { kind: SemanticStatementKind::Rename { source: adapt_expression(source), destination: adapt_expression(destination) }, span: *span } },
        rdgen_frontend::StatementCore::Close { close, span } => { let rdgen_frontend::CloseStmt::Close { channel, .. } = close.as_ref(); SemanticStatement { kind: SemanticStatementKind::Close(adapt_expression(channel)), span: *span } },
        rdgen_frontend::StatementCore::Data { data, span } => { let rdgen_frontend::DataStmt::Data { first, rest, .. } = data.as_ref(); SemanticStatement { kind: SemanticStatementKind::Data(std::iter::once(adapt_expression(first)).chain(rest.iter().map(|(_, value)| adapt_expression(value))).collect()), span: *span } },
        rdgen_frontend::StatementCore::Read { read, span } => { let rdgen_frontend::ReadStmt::Read { first, rest, .. } = read.as_ref(); SemanticStatement { kind: SemanticStatementKind::Read(std::iter::once(adapt_expression(first)).chain(rest.iter().map(|(_, value)| adapt_expression(value))).collect()), span: *span } },
        rdgen_frontend::StatementCore::Restore { restore, span } => { let rdgen_frontend::RestoreStmt::Restore { target, .. } = restore.as_ref(); SemanticStatement { kind: SemanticStatementKind::Restore(target.as_deref().map(|target| NamedReference { name: identifier(target), span: target.span() })), span: *span } },
        rdgen_frontend::StatementCore::Dim { dim, span } => { let rdgen_frontend::DimStmt::Dim { first, rest, .. } = dim.as_ref(); SemanticStatement { kind: SemanticStatementKind::Dim(std::iter::once(first.as_ref()).chain(rest.iter().map(|(_, value)| value.as_ref())).map(adapt_dim_item).collect()), span: *span } },
        rdgen_frontend::StatementCore::Const { constant, span } => { let rdgen_frontend::ConstStmt::Const { name, value, .. } = constant.as_ref(); SemanticStatement { kind: SemanticStatementKind::Const { name: NamedReference { name: identifier(name), span: name.span() }, value: adapt_expression(value) }, span: *span } },
        rdgen_frontend::StatementCore::Global { global, span } => { let rdgen_frontend::GlobalDecl::GlobalDeclaration { name, .. } = global.as_ref(); SemanticStatement { kind: SemanticStatementKind::Global(NamedReference { name: identifier(name), span: name.span() }), span: *span } },
        rdgen_frontend::StatementCore::Swap { swap, span } => { let rdgen_frontend::SwapStmt::Swap { left, right, .. } = swap.as_ref(); SemanticStatement { kind: SemanticStatementKind::Swap { left: adapt_expression(left), right: adapt_expression(right) }, span: *span } },
        rdgen_frontend::StatementCore::Randomize { randomize, span } => { let rdgen_frontend::RandomizeStmt::Randomize { seed, .. } = randomize.as_ref(); SemanticStatement { kind: SemanticStatementKind::Randomize(seed.as_deref().map(adapt_expression).map(RandomizeSeed::Value).unwrap_or(RandomizeSeed::Default)), span: *span } },
        rdgen_frontend::StatementCore::Poke { poke, span } => { let rdgen_frontend::PokeStmt::Poke { address, value, .. } = poke.as_ref(); SemanticStatement { kind: SemanticStatementKind::Poke { address: adapt_expression(address), value: adapt_expression(value) }, span: *span } },
        rdgen_frontend::StatementCore::Out { out, span } => { let rdgen_frontend::OutStmt::Out { port, value, .. } = out.as_ref(); SemanticStatement { kind: SemanticStatementKind::Out { port: adapt_expression(port), value: adapt_expression(value) }, span: *span } },
        rdgen_frontend::StatementCore::Width { width, span } => { let rdgen_frontend::WidthStmt::Width { channel, value, .. } = width.as_ref(); SemanticStatement { kind: SemanticStatementKind::Width { channel: channel.as_ref().map(|(_, value, _)| adapt_expression(value)), value: adapt_expression(value) }, span: *span } },
        rdgen_frontend::StatementCore::LineInput { line_input, span } => { let rdgen_frontend::LineInputStmt::LineInput { channel, target, .. } = line_input.as_ref(); SemanticStatement { kind: SemanticStatementKind::LineInput { channel: adapt_expression(channel), target: adapt_expression(target) }, span: *span } },
        rdgen_frontend::StatementCore::Get { get, span } => { let rdgen_frontend::GetStmt::Get { channel, position, .. } = get.as_ref(); SemanticStatement { kind: SemanticStatementKind::Get { channel: adapt_expression(channel), position: position.as_ref().map(adapt_file_position) }, span: *span } },
        rdgen_frontend::StatementCore::Put { put, span } => { let rdgen_frontend::PutStmt::Put { channel, position, .. } = put.as_ref(); SemanticStatement { kind: SemanticStatementKind::Put { channel: adapt_expression(channel), position: position.as_ref().map(adapt_file_position) }, span: *span } },
        rdgen_frontend::StatementCore::Lset { lset, span } => { let rdgen_frontend::LsetStmt::Lset { target, value, .. } = lset.as_ref(); SemanticStatement { kind: SemanticStatementKind::Lset { target: NamedReference { name: identifier(target), span: target.span() }, value: adapt_expression(value) }, span: *span } },
        rdgen_frontend::StatementCore::Rset { rset, span } => { let rdgen_frontend::RsetStmt::Rset { target, value, .. } = rset.as_ref(); SemanticStatement { kind: SemanticStatementKind::Rset { target: NamedReference { name: identifier(target), span: target.span() }, value: adapt_expression(value) }, span: *span } },
        rdgen_frontend::StatementCore::Locate { locate, span } => { let rdgen_frontend::LocateStmt::Locate { row, column, .. } = locate.as_ref(); SemanticStatement { kind: SemanticStatementKind::Locate { row: adapt_expression(row), column: adapt_expression(column) }, span: *span } },
        rdgen_frontend::StatementCore::Color { color, span } => { let rdgen_frontend::ColorStmt::Color { foreground, background, .. } = color.as_ref(); SemanticStatement { kind: SemanticStatementKind::Color { foreground: adapt_expression(foreground), background: background.as_ref().map(|(_, value)| adapt_expression(value)) }, span: *span } },
        rdgen_frontend::StatementCore::Stop { span, .. } => SemanticStatement { kind: SemanticStatementKind::Stop, span: *span },
        rdgen_frontend::StatementCore::Clear { span, .. } => SemanticStatement { kind: SemanticStatementKind::Clear, span: *span },
        rdgen_frontend::StatementCore::Cls { span, .. } => SemanticStatement { kind: SemanticStatementKind::Cls, span: *span },
        rdgen_frontend::StatementCore::Beep { span, .. } => SemanticStatement { kind: SemanticStatementKind::Beep, span: *span },
        rdgen_frontend::StatementCore::System { span, .. } => SemanticStatement { kind: SemanticStatementKind::System, span: *span },
        rdgen_frontend::StatementCore::Lprint { lprint, span } => { let rdgen_frontend::LprintStmt::Lprint { using, tokens, .. } = lprint.as_ref(); SemanticStatement { kind: SemanticStatementKind::Lprint { using: using.as_ref().map(|(_, value, _)| adapt_expression(value)), tokens: tokens.iter().map(|value| adapt_print_token(value)).collect() }, span: *span } },
        rdgen_frontend::StatementCore::FileDeclaration { file_declaration, span } => { let rdgen_frontend::FileDeclStmt::FileDeclaration { name, record_type, path, mode, .. } = file_declaration.as_ref(); SemanticStatement { kind: SemanticStatementKind::FileDeclaration { name: NamedReference { name: identifier(name), span: name.span() }, record_type: record_type.as_ref().map(|(_, value)| NamedReference { name: identifier(value), span: value.span() }), path: adapt_expression(path), mode: mode.as_ref().map(|(_, value)| adapt_file_mode(value)) }, span: *span } },
        rdgen_frontend::StatementCore::Field { field, span } => { let rdgen_frontend::FieldStmt::Field { channel, first, rest, .. } = field.as_ref(); SemanticStatement { kind: SemanticStatementKind::Field { channel: adapt_expression(channel), bindings: std::iter::once(first.as_ref()).chain(rest.iter().map(|(_, value)| value.as_ref())).map(adapt_field_binding).collect() }, span: *span } },
        _ => SemanticStatement { kind: SemanticStatementKind::Unsupported, span: statement.span() },
    }
}

fn adapt_print_destination(destination: &rdgen_frontend::PrintDestination) -> PrintDestination {
    match destination {
        rdgen_frontend::PrintDestination::Standard { span } => PrintDestination::Standard { span: *span },
        rdgen_frontend::PrintDestination::Using { format, span } => PrintDestination::Using { format: adapt_expression(format), span: *span },
        rdgen_frontend::PrintDestination::Channel { channel, using, span } => PrintDestination::Channel { channel: adapt_expression(channel), using: using.as_ref().map(|(_, value, _)| adapt_expression(value)), span: *span },
    }
}
fn adapt_print_token(token: &rdgen_frontend::PrintToken) -> PrintToken {
    match token {
        rdgen_frontend::PrintToken::Comma { span } => PrintToken::Comma { span: *span },
        rdgen_frontend::PrintToken::Semicolon { span } => PrintToken::Semicolon { span: *span },
        rdgen_frontend::PrintToken::Expression { value, .. } => PrintToken::Expression(adapt_expression(value)),
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
    OpenMode { kind, span: mode.span() }
}
fn adapt_file_mode(mode: &rdgen_frontend::FileMode) -> FileModeKind { match mode { rdgen_frontend::FileMode::Input { .. } => FileModeKind::Input, rdgen_frontend::FileMode::Output { .. } => FileModeKind::Output, rdgen_frontend::FileMode::Append { .. } => FileModeKind::Append } }
fn adapt_field_binding(binding: &rdgen_frontend::FieldBinding) -> FieldBinding { let rdgen_frontend::FieldBinding::Binding { length, name, .. } = binding; let name_span = name.span(); let name = identifier(name); let type_suffix = name.chars().last().filter(|suffix| matches!(suffix, '$' | '%' | '&' | '!' | '#')).map(|suffix| suffix.to_string()); let is_string = type_suffix.as_deref() == Some("$"); FieldBinding { length: adapt_expression(length), name, name_span, type_suffix, is_string } }
fn adapt_dim_item(item: &rdgen_frontend::DimItem) -> DimDeclaration {
    let rdgen_frontend::DimItem::DimItem { name, axes, type_annotation, .. } = item;
    let dimensions = axes.as_ref().map(|axes| {
        let rdgen_frontend::ArrayAxes::ArrayAxes { first, rest, .. } = axes.as_ref();
        std::iter::once(first.as_ref()).chain(rest.iter().map(|(_, axis)| axis.as_ref())).map(|axis| match axis {
            rdgen_frontend::Axis::InferredCapacity { .. } => DimAxis::Inferred,
            rdgen_frontend::Axis::FixedCapacity { capacity, .. } => DimAxis::Fixed(integer_literal(capacity)),
        }).collect::<Vec<_>>()
    }).unwrap_or_default();
    DimDeclaration { name: typed_identifier(name), array_axes: dimensions.len(), dimensions, type_annotation: type_annotation.as_ref().map(|(_, value)| identifier(value)), span: item.span() }
}
fn adapt_file_position(position: &(rdgen_frontend::Token, Option<Box<rdgen_frontend::Expr>>, Option<(rdgen_frontend::Token, Box<rdgen_frontend::Expr>)>)) -> FilePosition {
    FilePosition { position: position.1.as_deref().map(adapt_expression), record: position.2.as_ref().map(|(_, value)| adapt_expression(value)) }
}

fn adapt_for_bounds(bounds: &rdgen_frontend::ForBounds) -> ForBounds { match bounds { rdgen_frontend::ForBounds::To { limit, step, .. } => ForBounds::To { limit: adapt_expression(limit), step: step.as_ref().map(|(_, value)| adapt_expression(value)) }, rdgen_frontend::ForBounds::Downto { limit, step, .. } => ForBounds::Downto { limit: adapt_expression(limit), step: *step } } }
fn adapt_loop_condition(condition: &rdgen_frontend::DoCondition) -> LoopCondition { let rdgen_frontend::DoCondition::Condition { kind, value, .. } = condition; LoopCondition { kind: if matches!(kind.as_ref(), rdgen_frontend::DoConditionKind::Until { .. }) { LoopConditionKind::Until } else { LoopConditionKind::While }, value: adapt_expression(value) } }
fn adapt_case_clause(clause: &rdgen_frontend::CaseClause) -> CaseClause { let rdgen_frontend::CaseClause::Case { values, body, span } = clause; let rdgen_frontend::CaseValues::CaseValues { first, rest, .. } = values.as_ref(); CaseClause { values: std::iter::once(first.as_ref()).chain(rest.iter().map(|(_, value)| value.as_ref())).map(adapt_case_value).collect(), body: body.iter().map(|value| adapt_statement(value)).collect(), span: *span } }
fn adapt_case_value(value: &rdgen_frontend::CaseValue) -> CaseValue { match value { rdgen_frontend::CaseValue::Comparison { operator, value, span } => CaseValue::Comparison { operator: comparison_operator(operator), value: adapt_expression(value), span: *span }, rdgen_frontend::CaseValue::ValueOrRange { first, range_end, span } => CaseValue::Value { first: adapt_expression(first), range_end: range_end.as_ref().map(|(_, value)| adapt_expression(value)), span: *span } } }
fn comparison_operator(operator: &rdgen_frontend::CompareOp) -> ComparisonOperator { match operator { rdgen_frontend::CompareOp::NotEqual { .. } => ComparisonOperator::NotEqual, rdgen_frontend::CompareOp::LessOrEqual { .. } => ComparisonOperator::LessOrEqual, rdgen_frontend::CompareOp::GreaterOrEqual { .. } => ComparisonOperator::GreaterOrEqual, rdgen_frontend::CompareOp::Equal { .. } => ComparisonOperator::Equal, rdgen_frontend::CompareOp::Less { .. } => ComparisonOperator::Less, rdgen_frontend::CompareOp::Greater { .. } => ComparisonOperator::Greater } }
fn adapt_catch(value: &rdgen_frontend::CatchClause) -> CatchBinding { let rdgen_frontend::CatchClause::Catch { error, filters, line, source, body, .. } = value; CatchBinding { error: identifier(error), error_span: error.span(), line: identifier(line), line_span: line.span(), filters: filters.as_ref().map(|(_, first, rest, _)| std::iter::once(first.as_ref()).chain(rest.iter().map(|(_, value)| value.as_ref())).map(adapt_expression).collect()).unwrap_or_default(), source: source.as_ref().map(|(_, value)| identifier(value)), source_span: source.as_ref().map(|(_, value)| value.span()), body: body.iter().map(|value| adapt_statement(value)).collect() } }

fn adapt_if(value: &rdgen_frontend::IfStmt, span: SourceSpan) -> SemanticStatement {
    let rdgen_frontend::IfStmt::If { condition, tail, .. } = value;
    let (then_body, else_body, block) = match tail.as_ref() {
        rdgen_frontend::IfTail::SingleLine { tail, .. } => {
            let rdgen_frontend::SingleLineIfTail::SingleLineIf { first, rest, else_clause, .. } = tail.as_ref();
            let then_body = std::iter::once(first.as_ref()).chain(rest.iter().map(Box::as_ref)).map(adapt_statement).collect();
            let else_body = else_clause.as_ref().map(|(_, first, rest)| std::iter::once(first.as_ref()).chain(rest.iter().map(Box::as_ref)).map(adapt_statement).collect()).unwrap_or_default();
            (then_body, else_body, false)
        }
        rdgen_frontend::IfTail::Block { tail, .. } => adapt_block_if(tail),
    };
    SemanticStatement { kind: SemanticStatementKind::If { condition: adapt_expression(condition), then_body, else_body, block }, span }
}

fn adapt_block_if(tail: &rdgen_frontend::BlockIfTail) -> (Vec<SemanticStatement>, Vec<SemanticStatement>, bool) {
    let rdgen_frontend::BlockIfTail::BlockIf { then_body, continuation, .. } = tail;
    let then_body = then_body.iter().map(|value| adapt_statement(value)).collect();
    let else_body = match continuation.as_ref() {
        rdgen_frontend::IfBlockContinuation::End { else_body, .. } => else_body.as_ref().map(|(_, body)| body.iter().map(|value| adapt_statement(value)).collect()).unwrap_or_default(),
        rdgen_frontend::IfBlockContinuation::ElseIf { condition, tail, span } => vec![SemanticStatement { kind: SemanticStatementKind::If { condition: adapt_expression(condition), then_body: adapt_block_if(tail).0, else_body: adapt_block_if(tail).1, block: true }, span: *span }],
    };
    (then_body, else_body, true)
}

fn assignment_operator(operator: &rdgen_frontend::AssignmentOp) -> AssignmentOperator {
    let rdgen_frontend::AssignmentOp::Token { text, .. } = operator;
    match text.0.as_str() { "=" => AssignmentOperator::Assign, "+=" => AssignmentOperator::Add, "-=" => AssignmentOperator::Subtract, "*=" => AssignmentOperator::Multiply, "/=" => AssignmentOperator::Divide, _ => unreachable!("generated grammar only emits known assignment operators") }
}

fn adapt_subprogram(value: &rdgen_frontend::Subprogram) -> CallableSignature {
    use rdgen_frontend::Subprogram;
    match value {
        Subprogram::Function { function, .. } => { let rdgen_frontend::FunctionDecl::FunctionDeclaration { name, parameters, body, span } = function.as_ref(); let name_text = typed_identifier(name); CallableSignature { kind: CallableKind::Function, name: name_text.clone(), name_span: name.span(), result_type: suffix_from_name(&name_text), receiver: None, receiver_span: None, parameters: adapt_parameters(parameters), body: body.iter().map(|value| adapt_statement(value)).collect(), span: *span } }
        Subprogram::Procedure { procedure, .. } => { let rdgen_frontend::ProcedureDecl::ProcedureDeclaration { name, parameters, body, span } = procedure.as_ref(); CallableSignature { kind: CallableKind::Procedure, name: identifier(name), name_span: name.span(), result_type: None, receiver: None, receiver_span: None, parameters: adapt_parameters(parameters), body: body.iter().map(|value| adapt_statement(value)).collect(), span: *span } }
        Subprogram::Method { method, .. } => adapt_method(method, CallableKind::Method),
        Subprogram::FluentMethod { method, .. } => { let rdgen_frontend::FluentMethodDecl::FluentMethod { method, .. } = method.as_ref(); adapt_method(method, CallableKind::FluentMethod) }
    }
}

fn adapt_method(method: &rdgen_frontend::MethodDecl, kind: CallableKind) -> CallableSignature {
    let rdgen_frontend::MethodDecl::MethodDeclaration { name, receiver, parameters, result, body, span, .. } = method;
    CallableSignature { kind, name: identifier(name), name_span: name.span(), result_type: result.as_ref().map(|(_, value)| adapt_return_type(value)), receiver: Some(identifier(receiver)), receiver_span: Some(receiver.span()), parameters: adapt_parameters(parameters), body: body.iter().map(|value| adapt_statement(value)).collect(), span: *span }
}

fn adapt_inline_methods(record: &rdgen_frontend::RecordDecl, receiver: &str) -> Vec<CallableSignature> {
    let rdgen_frontend::RecordDecl::RecordDeclaration { members, .. } = record;
    members.iter().filter_map(|member| match member.as_ref() {
        rdgen_frontend::RecordMember::InlineMethod { method, .. } => { let rdgen_frontend::InlineMethod::InlineMethodDeclaration { name, parameters, result, body, span, .. } = method.as_ref(); Some(CallableSignature { kind: CallableKind::InlineMethod, name: identifier(name), name_span: name.span(), result_type: result.as_ref().map(|(_, value)| adapt_return_type(value)), receiver: Some(receiver.into()), receiver_span: None, parameters: adapt_parameters(parameters), body: body.iter().map(|value| adapt_statement(value)).collect(), span: *span }) }
        _ => None,
    }).collect()
}

fn adapt_parameters(parameters: &Option<Box<rdgen_frontend::ParamList>>) -> Vec<Parameter> {
    let Some(parameters) = parameters else { return Vec::new() };
    let rdgen_frontend::ParamList::ParameterList { first, rest, .. } = parameters.as_ref();
    std::iter::once(first.as_ref()).chain(rest.iter().map(|(_, parameter)| parameter.as_ref())).map(adapt_parameter).collect()
}

fn adapt_parameter(parameter: &rdgen_frontend::Param) -> Parameter {
    let rdgen_frontend::Param::Parameter { mode, name, axes, default, span, .. } = parameter;
    let passing = mode.as_ref().map(|mode| match mode.as_ref() { rdgen_frontend::PassingMode::ByRef { .. } => Passing::ByRef, rdgen_frontend::PassingMode::ByVal { .. } => Passing::ByVal });
    let array_axes = axes.as_ref().map(|axes| { let rdgen_frontend::ArrayAxes::ArrayAxes { rest, .. } = axes.as_ref(); rest.len() + 1 }).unwrap_or(0);
    Parameter { name: typed_identifier(name), name_span: name.span(), type_suffix: suffix_from_name(&typed_identifier(name)), passing, array_axes, default: default.as_ref().map(|(_, value)| adapt_expression(value)), span: *span }
}

fn suffix_from_name(name: &str) -> Option<String> { name.chars().last().filter(|value| matches!(value, '%' | '&' | '!' | '#' | '$' | '@')).map(|value| value.to_string()) }
fn adapt_return_type(value: &rdgen_frontend::ReturnType) -> String { match value { rdgen_frontend::ReturnType::Integer { .. } => "%", rdgen_frontend::ReturnType::Long { .. } => "&", rdgen_frontend::ReturnType::Single { .. } => "!", rdgen_frontend::ReturnType::Double { .. } => "#", rdgen_frontend::ReturnType::String { .. } => "$", rdgen_frontend::ReturnType::Suffix { value, .. } => { let rdgen_frontend::Suffix::Token { text, .. } = value.as_ref(); text.0.as_str() } }.into() }

fn adapt_record(record: &rdgen_frontend::RecordDecl) -> Record {
    let rdgen_frontend::RecordDecl::RecordDeclaration { name, combines, members, span } = record;
    let combines = combines.as_ref().map(|(_, list)| {
        let rdgen_frontend::CombinedRecordList::CombinedRecordList { first, rest, .. } = list.as_ref();
        std::iter::once(identifier(first)).chain(rest.iter().map(|(_, name)| identifier(name))).collect()
    }).unwrap_or_default();
    let fields = members.iter().filter_map(|member| match member.as_ref() {
        rdgen_frontend::RecordMember::Field { field, .. } => Some(adapt_record_field(field)),
        rdgen_frontend::RecordMember::InlineMethod { .. } => None,
    }).collect();
    Record { name: identifier(name), name_span: name.span(), combines, fields, span: *span }
}

fn adapt_record_field(field: &rdgen_frontend::FieldDecl) -> RecordField {
    let rdgen_frontend::FieldDecl::FieldDeclaration { name, field_type, span } = field;
    RecordField { name: identifier(name), name_span: name.span(), field_type: adapt_field_type(field_type), span: *span }
}

fn adapt_field_type(field_type: &rdgen_frontend::FieldType) -> RecordFieldType {
    use rdgen_frontend::FieldType;
    match field_type {
        FieldType::StringType { capacity, alignment, span } => RecordFieldType::String {
            capacity: capacity.as_ref().map(|(_, value, _)| integer_literal(value)),
            alignment: alignment.as_ref().map(|value| match value.as_ref() {
                rdgen_frontend::StringAlign::LeftPad { .. } => StringAlignment::LeftPad,
                rdgen_frontend::StringAlign::Left { .. } => StringAlignment::Left,
                rdgen_frontend::StringAlign::RightPad { .. } => StringAlignment::RightPad,
                rdgen_frontend::StringAlign::Right { .. } => StringAlignment::Right,
            }), span: *span },
        FieldType::Int16Type { span } => RecordFieldType::Int16 { span: *span },
        FieldType::IntType { span } => RecordFieldType::Int { span: *span },
        FieldType::Int32Type { span } => RecordFieldType::Int32 { span: *span },
        FieldType::Float32Type { span } => RecordFieldType::Float32 { span: *span },
        FieldType::Float64Type { span } => RecordFieldType::Float64 { span: *span },
        FieldType::RecordType { record, span } => RecordFieldType::Record { name: identifier(record), span: *span },
    }
}

fn identifier(identifier: &rdgen_frontend::Identifier) -> String {
    let rdgen_frontend::Identifier::Token { text, .. } = identifier;
    text.0.clone()
}

fn typed_identifier(typed: &rdgen_frontend::TypedIdent) -> String { let rdgen_frontend::TypedIdent::TypedIdentifier { value, .. } = typed; identifier(value) }

fn integer_literal(literal: &rdgen_frontend::IntegerLiteral) -> String {
    let rdgen_frontend::IntegerLiteral::Token { text, .. } = literal;
    text.0.clone()
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
        Expr::True { .. } => ExpressionKind::Boolean(true), Expr::False { .. } => ExpressionKind::Boolean(false),
        Expr::Parenthesized { value, .. } => ExpressionKind::Parenthesized(Box::new(adapt_expression(value))),
        Expr::Unary { operator, operand, .. } => ExpressionKind::Unary { operator: operator.0.clone(), operand: Box::new(adapt_expression(operand)) },
        Expr::Binary { left, operator, right, .. } => ExpressionKind::Binary { left: Box::new(adapt_expression(left)), operator: operator.0.clone(), right: Box::new(adapt_expression(right)) },
        Expr::Call { name, arguments, .. } => ExpressionKind::Call { name: identifier(name), arguments: adapt_arguments(arguments.as_deref()) },
        Expr::Index { name, index, .. } => ExpressionKind::Index { name: identifier(name), index: Box::new(adapt_expression(index)) },
        Expr::Member { base, member, arguments, .. } => ExpressionKind::Member { base: base.as_ref().map(|value| Box::new(adapt_expression(value))), member: identifier(member), arguments: arguments.as_ref().map(|(_, args, _)| adapt_arguments(args.as_deref())) },
        Expr::RecordLiteral { fields, .. } => ExpressionKind::RecordLiteral(adapt_record_literal_fields(fields)), Expr::PartialRecordLiteral { fields, .. } => ExpressionKind::PartialRecordLiteral(adapt_record_literal_fields(&fields.2)),
    };
    let value_type = match &kind {
        ExpressionKind::Name(name) => SemanticValueType::from_suffix(name.chars().last()),
        ExpressionKind::Literal(value) => {
            if value.starts_with('"') { SemanticValueType::String }
            else if value.contains('.') || value.contains('e') || value.contains('E') { SemanticValueType::Double }
            else { SemanticValueType::Integer }
        }
        ExpressionKind::Boolean(_) => SemanticValueType::Boolean,
        ExpressionKind::Parenthesized(inner) | ExpressionKind::Unary { operand: inner, .. } => inner.value_type,
        ExpressionKind::Binary { left, .. } => left.value_type,
        ExpressionKind::Call { .. }
        | ExpressionKind::Index { .. }
        | ExpressionKind::Member { .. }
        | ExpressionKind::RecordLiteral(_)
        | ExpressionKind::PartialRecordLiteral(_) => SemanticValueType::Unknown,
    };
    Expression { kind, span, value_type }
}
fn adapt_record_literal_fields(fields: &rdgen_frontend::RecordLitFields) -> Vec<FieldInitializer> {
    let rdgen_frontend::RecordLitFields::RecordFields { fields, .. } = fields;
    fields.as_ref().map(|(first, rest)| std::iter::once(first.as_ref()).chain(rest.iter().map(|(_, value)| value.as_ref())).map(|field| { let rdgen_frontend::FieldInit::Field { name, value, .. } = field; FieldInitializer { name: identifier(name), value: adapt_expression(value) } }).collect()).unwrap_or_default()
}

fn adapt_arguments(arguments: Option<&rdgen_frontend::ArgList>) -> Vec<Expression> {
    let Some(arguments) = arguments else { return Vec::new() };
    let rdgen_frontend::ArgList::Arguments { first, rest, .. } = arguments;
    std::iter::once(first.as_ref()).chain(rest.iter().map(|(_, value)| value.as_ref())).map(adapt_expression).collect()
}
fn float_literal(value: &rdgen_frontend::FloatLiteral) -> String { let rdgen_frontend::FloatLiteral::Token { text, .. } = value; text.0.clone() }
fn token_text_hex(value: &rdgen_frontend::HexLiteral) -> String { let rdgen_frontend::HexLiteral::Token { text, .. } = value; text.0.clone() }
fn token_text_octal(value: &rdgen_frontend::OctalLiteral) -> String { let rdgen_frontend::OctalLiteral::Token { text, .. } = value; text.0.clone() }
fn token_text_string(value: &rdgen_frontend::StringLiteral) -> String { let rdgen_frontend::StringLiteral::Token { text, .. } = value; text.0.clone() }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapts_header_and_dependencies_with_generated_spans() {
        let source = "program Demo shared globals\nrequire com.bascal.io\nimport com.bascal.math\n";
        let module = adapt_module(&rdgen_frontend::parse(source).unwrap());
        assert_eq!(module.span, SourceSpan { start: 0, end: source.len() - 1 });
        assert_eq!(module.header, Some(ModuleHeader::Program { name: "Demo".into(), shared: Some("globals".into()), span: SourceSpan { start: 0, end: 27 } }));
        assert_eq!(module.dependencies.len(), 2);
        assert_eq!(module.dependencies[0].kind, DependencyKind::Require);
        assert_eq!(module.dependencies[1].path, "com.bascal.math");
    }

    #[test]
    fn adapts_records_without_grammar_wrapper_nodes() {
        let module = adapt_module(&rdgen_frontend::parse(
            "record Parent combines Child\nname: string(24) right\nend record\n",
        ).unwrap());
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
            "function sum%(byref values%(?), byval count%)\nreturn count%\nend function\nprocedure log(message$)\nend procedure\nmethod scale[integer](byref factor%)\nend method\n",
        ).unwrap());
        assert_eq!(module.callables.len(), 3);
        assert_eq!(module.callables[0].kind, CallableKind::Function);
        assert_eq!(module.callables[0].name, "sum%");
        assert_eq!(module.callables[0].parameters[0].passing, Some(Passing::ByRef));
        assert_eq!(module.callables[0].parameters[0].array_axes, 1);
        assert_eq!(module.callables[0].parameters[0].type_suffix.as_deref(), Some("%"));
        assert_eq!(module.callables[0].parameters[1].type_suffix.as_deref(), Some("%"));
        assert_eq!(module.callables[1].kind, CallableKind::Procedure);
        assert_eq!(module.callables[1].parameters[0].type_suffix.as_deref(), Some("$"));
        assert_eq!(module.callables[2].receiver.as_deref(), Some("integer"));
        assert_eq!(module.callables[0].result_type.as_deref(), Some("%"));
        assert!(!module.callables[0].body.is_empty());
        assert_eq!(module.callable_value_type("sum%"), SemanticValueType::Integer);
        assert_eq!(module.callable_value_type("log"), SemanticValueType::Unknown);
        let mut call = Expression {
            kind: ExpressionKind::Call { name: "sum%".into(), arguments: Vec::new() },
            span: module.callables[0].span,
            value_type: SemanticValueType::Unknown,
        };
        module.annotate_expression_types(&mut call);
        assert_eq!(call.value_type, SemanticValueType::Integer);
        assert_eq!(SemanticValueType::from_suffix(Some('$')), SemanticValueType::String);
        assert_eq!(SemanticValueType::from_suffix(None), SemanticValueType::Unknown);
        assert_eq!(SemanticValueType::Integer.suffix(), Some('%'));
        assert_eq!(SemanticValueType::Boolean.suffix(), None);
    }

    #[test]
    fn preserves_nested_callable_declarations_for_backend_facts() {
        let module = adapt_module(&rdgen_frontend::parse(
            "function work%(value%)\nif value% then\ndim local%(10)\nglobal shared%\nend if\nreturn value%\nend function\n",
        ).unwrap());
        let body = &module.callables[0].body;
        let Some(SemanticStatement { kind: SemanticStatementKind::Line(items), .. }) = body.first() else { panic!() };
        assert!(matches!(items[0].kind, SemanticStatementKind::If { ref then_body, .. }
            if then_body.iter().any(|statement| matches!(statement.kind, SemanticStatementKind::Line(_)))));
    }

    #[test]
    fn preserves_bare_intrinsic_identifier_suffixes() {
        let module = adapt_module(&rdgen_frontend::parse("print inkey$\nvalue$ = date$\n").unwrap());
        let SemanticStatementKind::Line(print) = &module.statements[0].kind else { panic!() };
        assert!(matches!(print[0].kind, SemanticStatementKind::Print { ref tokens, .. }
            if matches!(tokens[0], PrintToken::Expression(ref expression)
                if matches!(expression.kind, ExpressionKind::Name(ref name) if name == "inkey$"))));
        let SemanticStatementKind::Line(assignment) = &module.statements[1].kind else { panic!() };
        assert!(matches!(assignment[0].kind, SemanticStatementKind::Assignment { ref value, .. }
            if matches!(value.kind, ExpressionKind::Name(ref name) if name == "date$")));
        let SemanticStatementKind::Assignment { value, .. } = &assignment[0].kind else { panic!() };
        assert_eq!(value.value_type, SemanticValueType::String);
    }

    #[test]
    fn annotates_call_types_across_statement_trees() {
        let module = parse_and_adapt("function f%()\nreturn 1\nend function\nresult% = f%()\n").unwrap();
        let SemanticStatementKind::Line(items) = &module.statements[0].kind else { panic!() };
        let SemanticStatementKind::Assignment { value, .. } = &items[0].kind else { panic!() };
        assert_eq!(value.value_type, SemanticValueType::Integer);
    }

    #[test]
    fn preserves_field_binding_string_classification() {
        let module = adapt_module(&rdgen_frontend::parse("field #1, 10 as name$, 4 as code%\n").unwrap());
        let SemanticStatementKind::Line(field) = &module.statements[0].kind else { panic!() };
        let SemanticStatementKind::Field { ref bindings, .. } = field[0].kind else { panic!() };
        assert_eq!(bindings.iter().map(|binding| binding.is_string).collect::<Vec<_>>(), [true, false]);
        assert_eq!(bindings[0].type_suffix.as_deref(), Some("$"));
        assert_eq!(bindings[1].type_suffix.as_deref(), Some("%"));
    }

    #[test]
    fn preserves_file_declaration_record_type() {
        let module = adapt_module(&rdgen_frontend::parse("file ledger as Entry = open(path$)\nfile scores = open(path$) for input\n").unwrap());
        let SemanticStatementKind::Line(statements) = &module.statements[0].kind else { panic!() };
        let SemanticStatementKind::FileDeclaration { name, record_type: Some(record), .. } = &statements[0].kind else { panic!() };
        assert_eq!(name.name, "ledger");
        assert_eq!(record.name, "Entry");
        let SemanticStatementKind::Line(sequential) = &module.statements[1].kind else { panic!() };
        let SemanticStatementKind::FileDeclaration { mode: Some(FileModeKind::Input), .. } = &sequential[0].kind else { panic!() };
    }

    #[test]
    fn adapts_precedence_expression_with_operator_spelling() {
        let program = rdgen_frontend::parse("result% = -2 ^ 2 + value%\n").unwrap();
        let rdgen_frontend::Program::File { items, .. } = program;
        let rdgen_frontend::FileItem::Statement { statement, .. } = items[0].as_ref() else { panic!() };
        let rdgen_frontend::TopLevelStatement::Statement { statement, .. } = statement.as_ref() else { panic!() };
        let rdgen_frontend::Statement::Line { first, .. } = statement.as_ref() else { panic!() };
        let rdgen_frontend::StatementCore::AssignmentOrExpression { statement, .. } = first.as_ref() else { panic!() };
        let rdgen_frontend::AssignmentOrExprStmt::Assignment { value, .. } = statement.as_ref() else { panic!() };
        assert!(matches!(adapt_expression(value).kind,
            ExpressionKind::Binary { ref operator, ref left, ref right }
            if operator == "+" && matches!(left.kind, ExpressionKind::Unary { .. }) && matches!(right.kind, ExpressionKind::Name(ref name) if name == "value%")));
    }

    #[test]
    fn adapts_colon_chained_assignments_as_a_statement_list() {
        let module = adapt_module(&rdgen_frontend::parse("a% = 1 : b% += a%\n").unwrap());
        let SemanticStatementKind::Line(statements) = &module.statements[0].kind else { panic!() };
        assert_eq!(statements.len(), 2);
        assert!(matches!(statements[0].kind, SemanticStatementKind::Assignment { operator: AssignmentOperator::Assign, .. }));
        assert!(matches!(statements[1].kind, SemanticStatementKind::Assignment { operator: AssignmentOperator::Add, .. }));
    }

    #[test]
    fn preserves_mid_assignment_as_a_typed_statement() {
        let module = adapt_module(&rdgen_frontend::parse("mid$(text$, 2, 3) = replacement$\n").unwrap());
        let SemanticStatementKind::Line(statements) = &module.statements[0].kind else { panic!() };
        assert!(matches!(statements[0].kind, SemanticStatementKind::MidAssign { ref target, ref start, ref length, .. }
            if matches!(target.kind, ExpressionKind::Name(ref name) if name == "text$")
                && matches!(start.kind, ExpressionKind::Literal(ref value) if value == "2")
                && matches!(length.as_ref().map(|value| &value.kind), Some(ExpressionKind::Literal(value)) if value == "3")));
    }

    #[test]
    fn adapts_print_input_and_write_payloads() {
        let module = adapt_module(&rdgen_frontend::parse(
            "print #1, value%;\ninput #1, target%\nwrite #1, value%, other%\n",
        ).unwrap());
        let SemanticStatementKind::Line(print) = &module.statements[0].kind else { panic!() };
        assert!(matches!(print[0].kind, SemanticStatementKind::Print { ref destination, ref tokens }
            if matches!(destination, PrintDestination::Channel { ref channel, using: None, .. } if matches!(channel.kind, ExpressionKind::Literal(ref value) if value == "1"))
                && tokens.len() == 2));
        let SemanticStatementKind::Line(input) = &module.statements[1].kind else { panic!() };
        assert!(matches!(input[0].kind, SemanticStatementKind::Input { source: InputSource::Channel(_), ref targets }
            if targets.len() == 1));
        let SemanticStatementKind::Line(write) = &module.statements[2].kind else { panic!() };
        assert!(matches!(write[0].kind, SemanticStatementKind::Write { values: WriteValues::Values(ref values), .. } if values.len() == 2));
    }

    #[test]
    fn adapts_file_operation_payloads() {
        let module = adapt_module(&rdgen_frontend::parse(
            "open path$ for output as #1 len = 128\nseek 1, 4\nkill path$\nname old$ as new$\nclose #1\n",
        ).unwrap());
        let SemanticStatementKind::Line(open) = &module.statements[0].kind else { panic!() };
        assert!(matches!(open[0].kind, SemanticStatementKind::Open { ref mode, ref length, .. } if mode.kind == OpenModeKind::Output && length.is_some()));
        let SemanticStatementKind::Line(seek) = &module.statements[1].kind else { panic!() };
        assert!(matches!(seek[0].kind, SemanticStatementKind::Seek { .. }));
        let SemanticStatementKind::Line(kill) = &module.statements[2].kind else { panic!() };
        assert!(matches!(kill[0].kind, SemanticStatementKind::Kill(_)));
        let SemanticStatementKind::Line(rename) = &module.statements[3].kind else { panic!() };
        assert!(matches!(rename[0].kind, SemanticStatementKind::Rename { .. }));
        let SemanticStatementKind::Line(close) = &module.statements[4].kind else { panic!() };
        assert!(matches!(close[0].kind, SemanticStatementKind::Close(_)));
    }

    #[test]
    fn adapts_data_read_restore_payloads() {
        let module = adapt_module(&rdgen_frontend::parse("data 1, 2\nread first%, second%\nrestore marker\n").unwrap());
        let SemanticStatementKind::Line(data) = &module.statements[0].kind else { panic!() };
        assert!(matches!(data[0].kind, SemanticStatementKind::Data(ref values) if values.len() == 2));
        let SemanticStatementKind::Line(read) = &module.statements[1].kind else { panic!() };
        assert!(matches!(read[0].kind, SemanticStatementKind::Read(ref values) if values.len() == 2));
        let SemanticStatementKind::Line(restore) = &module.statements[2].kind else { panic!() };
        assert!(matches!(restore[0].kind, SemanticStatementKind::Restore(Some(ref target)) if target.name == "marker"));
    }

    #[test]
    fn adapts_declaration_payloads() {
        let module = adapt_module(&rdgen_frontend::parse("dim values%(?) : name$\ndim total as integer\nconst limit% = 10\nglobal shared%\n").unwrap());
        let SemanticStatementKind::Line(dim) = &module.statements[0].kind else { panic!() };
        assert!(matches!(dim[0].kind, SemanticStatementKind::Dim(ref values) if values[0].name == "values%" && values[0].array_axes == 1 && values[0].dimensions == [DimAxis::Inferred]));
        let fixed = adapt_module(&rdgen_frontend::parse("dim table%(10, 20)\n").unwrap());
        let SemanticStatementKind::Line(dim) = &fixed.statements[0].kind else { panic!() };
        assert!(matches!(dim[0].kind, SemanticStatementKind::Dim(ref values) if values[0].dimensions == [DimAxis::Fixed("10".into()), DimAxis::Fixed("20".into())]));
        let SemanticStatementKind::Line(typed_dim) = &module.statements[1].kind else { panic!() };
        assert!(matches!(typed_dim[0].kind, SemanticStatementKind::Dim(ref values) if values[0].name == "total" && values[0].type_annotation.as_deref() == Some("integer")));
        let SemanticStatementKind::Line(constant) = &module.statements[2].kind else { panic!() };
        assert!(matches!(constant[0].kind, SemanticStatementKind::Const { ref name, .. } if name.name == "limit%"));
        let SemanticStatementKind::Line(global) = &module.statements[3].kind else { panic!() };
        assert!(matches!(global[0].kind, SemanticStatementKind::Global(ref name) if name.name == "shared%"));
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
        let module = parse_and_adapt("const count = 10\nconst label = \"ready\"\n").unwrap();
        assert_eq!(module.const_types().get("count"), Some(&SemanticValueType::Integer));
        assert_eq!(module.const_types().get("label"), Some(&SemanticValueType::String));
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
        assert_eq!(module.top_level_error_handler_targets(), ["first", "second"]);
        assert_eq!(module.error_handler_targets(), ["first", "second", "third"]);
    }

    #[test]
    fn dispatch_counts_cover_semantic_throw_and_try_nodes() {
        let module = parse_and_adapt("try\nthrow\nend try\n").unwrap();
        assert_eq!(module.top_level_raise_site_count(), 1);
        assert_eq!(module.top_level_try_catch_count(), 1);
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
            let SemanticStatementKind::Line(items) = &statement.kind else { continue };
            for item in items {
                let SemanticStatementKind::Assignment { target, .. } = &item.kind else { continue };
                let ExpressionKind::Name(name) = &target.kind else { continue };
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
        let paths: Vec<_> = root.dependencies.iter().map(|dependency| dependency.path.as_str()).collect();
        assert_eq!(paths, ["firstDep", "secondDep", "rootDep"]);
    }

    #[test]
    fn name_scopes_retain_global_and_callable_visibility() {
        let module = parse_and_adapt(
            "shared% = 1\nfunction work%()\nglobal shared%\nlocal% = shared%\nreturn local%\nend function\n",
        )
        .unwrap();
        let scopes = module.name_scopes();
        assert!(scopes.global_names.contains("shared%"));
        assert!(scopes
            .callable_globals
            .get("work%")
            .is_some_and(|names| names.contains("shared%") && !names.contains("local%")));
    }

    #[test]
    fn adapts_record_literal_fields() {
        let module = parse_and_adapt("value = { name: 1, count: other% }\npartial = ?{ name: 2 }\n").unwrap();
        let SemanticStatementKind::Line(first) = &module.statements[0].kind else { panic!() };
        let SemanticStatementKind::Assignment { value, .. } = &first[0].kind else { panic!() };
        assert!(matches!(value.kind, ExpressionKind::RecordLiteral(ref fields) if fields.len() == 2 && fields[0].name == "name"));
        let SemanticStatementKind::Line(second) = &module.statements[1].kind else { panic!() };
        let SemanticStatementKind::Assignment { value, .. } = &second[0].kind else { panic!() };
        assert!(matches!(value.kind, ExpressionKind::PartialRecordLiteral(ref fields) if fields.len() == 1 && fields[0].name == "name"));
    }

    #[test]
    fn adapts_representative_generated_statement_corpus() {
        let source = "dim value%\nconst limit% = 3\nglobal shared%\nprint value%\ninput value%\nwrite #1, value%\nopen path$ for input as #1\ndata 1, 2\nread value%\nlocate 1, 2\ncolor 3, 4\nswap value%, shared%\nrandomize 1\npoke 1, 2\nout 1, 2\nwidth 80\nline input #1, value$\nget #1\nput #1\nlset field$ = value$\nrset field$ = value$\n";
        let module = parse_and_adapt(source).unwrap();
        assert_eq!(module.statements.len(), 21);
        assert!(module.statements.iter().all(|statement| !matches!(statement.kind, SemanticStatementKind::Unsupported)));
    }

    #[test]
    fn preserves_typed_control_transfer_targets() {
        let module = parse_and_adapt("goto target\ngosub target\nresume next\non error goto target\n").unwrap();
        let SemanticStatementKind::Line(goto) = &module.statements[0].kind else { panic!() };
        assert!(matches!(goto[0].kind, SemanticStatementKind::Goto(ref target) if target.name == "target"));
        let SemanticStatementKind::Line(gosub) = &module.statements[1].kind else { panic!() };
        assert!(matches!(gosub[0].kind, SemanticStatementKind::Gosub(ref target) if target.name == "target"));
        let SemanticStatementKind::Line(resume) = &module.statements[2].kind else { panic!() };
        assert!(matches!(resume[0].kind, SemanticStatementKind::Resume(Some(ResumeTarget::Next))));
        let SemanticStatementKind::Line(error) = &module.statements[3].kind else { panic!() };
        assert!(matches!(error[0].kind, SemanticStatementKind::OnErrorGoto(ErrorHandlerTarget::Label(ref target)) if target.name == "target"));
    }

    #[test]
    fn preserves_declaration_name_spans() {
        let module = parse_and_adapt("record Item\nvalue: int\nend record\n").unwrap();
        assert_eq!(module.records[0].name_span, SourceSpan { start: 7, end: 11 });
        assert_eq!(module.records[0].fields[0].name_span, SourceSpan { start: 12, end: 17 });
        let declarations = parse_and_adapt("dim values%(?)\nconst limit% = 1\n").unwrap();
        let SemanticStatementKind::Line(dim) = &declarations.statements[0].kind else { panic!() };
        assert!(matches!(dim[0].kind, SemanticStatementKind::Dim(ref values) if values[0].span.start < values[0].span.end));
    }

    #[test]
    fn preserves_typed_optional_statement_payloads() {
        let module = parse_and_adapt("return\nthrow\nrandomize\nwrite #1\n").unwrap();
        let SemanticStatementKind::Line(return_stmt) = &module.statements[0].kind else { panic!() };
        assert!(matches!(return_stmt[0].kind, SemanticStatementKind::Return(ReturnValue::Default)));
        let SemanticStatementKind::Line(throw_stmt) = &module.statements[1].kind else { panic!() };
        assert!(matches!(throw_stmt[0].kind, SemanticStatementKind::Throw(ThrowValue::Bare)));
        let SemanticStatementKind::Line(randomize) = &module.statements[2].kind else { panic!() };
        assert!(matches!(randomize[0].kind, SemanticStatementKind::Randomize(RandomizeSeed::Default)));
        let SemanticStatementKind::Line(write) = &module.statements[3].kind else { panic!() };
        assert!(matches!(write[0].kind, SemanticStatementKind::Write { values: WriteValues::Omitted, .. }));
    }

    #[test]
    fn preserves_catch_binding_spans() {
        let module = parse_and_adapt("try\nvalue% = 1\ncatch err%, line%, source$\nvalue% = 2\nend try\n").unwrap();
        let SemanticStatementKind::Line(body) = &module.statements[0].kind else { panic!() };
        let SemanticStatementKind::Try { catch: Some(catch), .. } = &body[0].kind else { panic!() };
        assert!(catch.error_span.start < catch.error_span.end);
        assert!(catch.line_span.start < catch.line_span.end);
        assert!(catch.source_span.is_some());
    }

    #[test]
    fn adapts_single_line_and_block_if_bodies() {
        let single = adapt_module(&rdgen_frontend::parse("if ready% then a% = 1\n").unwrap());
        let SemanticStatementKind::Line(body) = &single.statements[0].kind else { panic!() };
        assert!(matches!(body[0].kind, SemanticStatementKind::If { block: false, ref then_body, ref else_body, .. } if then_body.len() == 1 && else_body.is_empty()), "{body:#?}");
        let block = adapt_module(&rdgen_frontend::parse("if ready% then\na% = 1\nelse\na% = 2\nend if\n").unwrap());
        let SemanticStatementKind::Line(body) = &block.statements[0].kind else { panic!() };
        assert!(matches!(body[0].kind, SemanticStatementKind::If { block: true, ref then_body, ref else_body, .. } if then_body.len() == 1 && else_body.len() == 1));
    }

    #[test]
    fn adapts_while_body_recursively() {
        let module = adapt_module(&rdgen_frontend::parse("while ready%\ncount% += 1\nend while\n").unwrap());
        let SemanticStatementKind::Line(body) = &module.statements[0].kind else { panic!() };
        assert!(matches!(body[0].kind, SemanticStatementKind::While { ref body, .. } if body.len() == 1));
    }

    #[test]
    fn adapts_to_and_downto_bounds_explicitly() {
        let to = adapt_module(&rdgen_frontend::parse("for index% = 1 to 10 step 2\nend for\n").unwrap());
        let SemanticStatementKind::Line(body) = &to.statements[0].kind else { panic!() };
        assert!(matches!(body[0].kind, SemanticStatementKind::For { bounds: ForBounds::To { step: Some(_), .. }, .. }), "{body:#?}");
        let down = adapt_module(&rdgen_frontend::parse("for index% = 10 downto 1\nend for\n").unwrap());
        let SemanticStatementKind::Line(body) = &down.statements[0].kind else { panic!() };
        assert!(matches!(body[0].kind, SemanticStatementKind::For { bounds: ForBounds::Downto { step: -1, .. }, .. }));
    }

    #[test]
    fn adapts_pre_and_post_condition_do_loops() {
        let pre = adapt_module(&rdgen_frontend::parse("do while ready%\ncount% += 1\nend do\n").unwrap());
        let SemanticStatementKind::Line(body) = &pre.statements[0].kind else { panic!() };
        assert!(matches!(body[0].kind, SemanticStatementKind::Do { pre_condition: Some(LoopCondition { kind: LoopConditionKind::While, .. }), post_condition: None, .. }));
        let post = adapt_module(&rdgen_frontend::parse("do\ncount% += 1\nloop until done%\n").unwrap());
        let SemanticStatementKind::Line(body) = &post.statements[0].kind else { panic!() };
        assert!(matches!(body[0].kind, SemanticStatementKind::Do { pre_condition: None, post_condition: Some(LoopCondition { kind: LoopConditionKind::Until, .. }), .. }));
    }

    #[test]
    fn adapts_select_case_values() {
        let module = adapt_module(&rdgen_frontend::parse("select case value%\ncase 1 to 3, is >= 9\nresult% = 1\nend select\n").unwrap());
        let SemanticStatementKind::Line(body) = &module.statements[0].kind else { panic!() };
        let SemanticStatementKind::SelectCase { cases, else_body, .. } = &body[0].kind else { panic!() };
        assert_eq!(cases.len(), 1);
        assert!(matches!(cases[0].values[0], CaseValue::Value { range_end: Some(_), .. }));
        assert!(matches!(cases[0].values[1], CaseValue::Comparison { operator: ComparisonOperator::GreaterOrEqual, .. }));
        assert!(else_body.is_empty());
    }

    #[test]
    fn adapts_try_catch_finally_bindings() {
        let module = adapt_module(&rdgen_frontend::parse("try\nwork% = 1\ncatch err%, line%, source$\nwork% = 2\nfinally\nwork% = 3\nend try\n").unwrap());
        let SemanticStatementKind::Line(body) = &module.statements[0].kind else { panic!() };
        let SemanticStatementKind::Try { catch: Some(catch), finally_body, .. } = &body[0].kind else { panic!() };
        assert_eq!(catch.error, "err%");
        assert_eq!(catch.line, "line%");
        assert_eq!(catch.source.as_deref(), Some("source$"));
        assert_eq!(finally_body.len(), 1);
    }
}
