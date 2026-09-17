//! Grammar DSL parsing and validation producing [`rdgen_ir::Grammar`].

pub use rdgen_ir;

/// Compile the EBNF-plus grammar notation used by the rdgen starter fixtures.
pub fn compile(source: &str) -> Result<rdgen_ir::Grammar, String> {
    Parser::new(source).parse()
}

#[derive(Clone, Debug, PartialEq)]
enum TokenKind {
    Ident(String),
    Literal(String),
    Symbol(char),
    Arrow,
    Epsilon,
    Eof,
}

#[derive(Clone, Debug, PartialEq)]
struct Token {
    kind: TokenKind,
    span: rdgen_ir::Span,
}

struct Lexer<'a> {
    chars: std::str::CharIndices<'a>,
    source: &'a str,
    current: Option<(usize, char)>,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a str) -> Self {
        let mut chars = source.char_indices();
        let current = chars.next();
        Self {
            chars,
            source,
            current,
        }
    }

    fn next_char(&mut self) -> Option<(usize, char)> {
        let current = self.current;
        self.current = self.chars.next();
        current
    }

    fn lex(mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        while let Some((start, ch)) = self.current {
            if ch.is_whitespace() {
                self.next_char();
                continue;
            }
            if ch == '(' && self.peek_char() == Some('*') {
                self.skip_comment(start)?;
                continue;
            }
            if ch.is_ascii_alphabetic() || ch == '_' {
                let mut end = start;
                while let Some((index, part)) = self.current {
                    if !(part.is_ascii_alphanumeric() || part == '_') {
                        break;
                    }
                    end = index + part.len_utf8();
                    self.next_char();
                }
                let text = &self.source[start..end];
                tokens.push(Token {
                    kind: TokenKind::Ident(text.to_owned()),
                    span: rdgen_ir::Span::new(start, end),
                });
                continue;
            }
            if ch == '"' || ch == '\'' {
                tokens.push(self.literal()?);
                continue;
            }
            if ch == 'ε' {
                self.next_char();
                tokens.push(Token {
                    kind: TokenKind::Epsilon,
                    span: rdgen_ir::Span::new(start, start + ch.len_utf8()),
                });
                continue;
            }
            if ch == '=' && self.peek_char() == Some('>') {
                let end = start + 2;
                self.next_char();
                self.next_char();
                tokens.push(Token {
                    kind: TokenKind::Arrow,
                    span: rdgen_ir::Span::new(start, end),
                });
                continue;
            }
            if "=,;|(){}[]:".contains(ch) {
                self.next_char();
                tokens.push(Token {
                    kind: TokenKind::Symbol(ch),
                    span: rdgen_ir::Span::new(start, start + ch.len_utf8()),
                });
                continue;
            }
            return Err(format!("unexpected character {:?} at {}", ch, start));
        }
        tokens.push(Token {
            kind: TokenKind::Eof,
            span: rdgen_ir::Span::new(self.source.len(), self.source.len()),
        });
        Ok(tokens)
    }

    fn peek_char(&self) -> Option<char> {
        self.source[self.current.map_or(0, |(index, _)| index)..]
            .chars()
            .nth(1)
    }

    fn skip_comment(&mut self, start: usize) -> Result<(), String> {
        self.next_char();
        self.next_char();
        while self.current.is_some() {
            if self.current.map(|(_, ch)| ch) == Some('*') && self.peek_char() == Some(')') {
                self.next_char();
                self.next_char();
                return Ok(());
            }
            self.next_char();
        }
        Err(format!("unterminated comment at {}", start))
    }

    fn literal(&mut self) -> Result<Token, String> {
        let (start, quote) = self.current.expect("literal requires current character");
        self.next_char();
        let mut value = String::new();
        while let Some((index, ch)) = self.current {
            self.next_char();
            if ch == '\\' {
                if let Some((_, escaped)) = self.current {
                    value.push(match escaped {
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        other => other,
                    });
                    self.next_char();
                    continue;
                }
                return Err(format!("unterminated escape in literal at {}", index));
            }
            if ch == quote {
                let end = index + ch.len_utf8();
                return Ok(Token {
                    kind: TokenKind::Literal(value),
                    span: rdgen_ir::Span::new(start, end),
                });
            }
            value.push(ch);
        }
        Err(format!("unterminated literal at {}", start))
    }
}

struct Parser<'a> {
    tokens: Vec<Token>,
    position: usize,
    source: &'a str,
}

struct ParsedAlternative {
    elements: Vec<rdgen_ir::Element>,
    constructor: Option<rdgen_ir::Constructor>,
    recovery: Option<rdgen_ir::RecoveryPoint>,
}

impl<'a> Parser<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            tokens: Vec::new(),
            position: 0,
            source,
        }
    }

    fn parse(mut self) -> Result<rdgen_ir::Grammar, String> {
        self.tokens = Lexer::new(self.source).lex()?;
        self.expect_ident("grammar")?;
        let name = self.ident()?;
        self.expect_symbol(';')?;
        let mut rules = Vec::new();
        let mut precedence = Vec::new();
        let mut start = None;
        while !self.at_eof() {
            if self.is_start_directive() {
                self.expect_ident("start")?;
                if start.is_some() {
                    return Err(self.error("duplicate start-rule declaration"));
                }
                start = Some(self.ident()?);
                self.expect_symbol(';')?;
                continue;
            }
            if self.at_ident("precedence") {
                precedence.push(self.parse_precedence()?);
                continue;
            }
            let lexical = if self.at_ident("lexical") {
                self.expect_ident("lexical")?;
                true
            } else {
                false
            };
            let start = self.current().span.start;
            let rule_name = self.ident()?;
            self.expect_symbol('=')?;
            let alternatives = self.alternatives()?;
            self.expect_symbol(';')?;
            if alternatives
                .iter()
                .any(|alternative| starts_with_rule(&alternative.elements, &rule_name))
            {
                return Err(format!(
                    "direct left recursion in rule '{}' at {}",
                    rule_name, start
                ));
            }
            rules.push(rdgen_ir::Rule {
                name: rule_name.clone(),
                lexical,
                output: rdgen_ir::TypeName(rule_name.clone()),
                alternatives: alternatives
                    .into_iter()
                    .map(|alternative| rdgen_ir::Alternative {
                        span: alternative
                            .elements
                            .first()
                            .map_or(rdgen_ir::Span::new(start, start), element_span),
                        elements: alternative.elements,
                        constructor: alternative.constructor.unwrap_or(rdgen_ir::Constructor {
                            type_name: rdgen_ir::TypeName(rule_name.clone()),
                            fields: Vec::new(),
                        }),
                        recovery: alternative.recovery,
                    })
                    .collect(),
                span: rdgen_ir::Span::new(start, self.previous().span.end),
            });
        }
        validate_rule_names(&rules)?;
        resolve_symbols(&mut rules)?;
        validate_constructors(&rules)?;
        validate_repetition_progress(&rules)?;
        reject_indirect_left_recursion(&rules)?;
        validate_precedence(&precedence, &rules)?;
        validate_precedence_constructors(&precedence)?;
        let start = start.unwrap_or_else(|| {
            rules
                .first()
                .map(|rule| rule.name.clone())
                .unwrap_or_default()
        });
        if !rules.iter().any(|rule| rule.name == start) {
            return Err(format!("start rule '{}' is not declared", start));
        }
        Ok(rdgen_ir::Grammar {
            name,
            start,
            rules,
            tokens: Vec::new(),
            precedence,
        })
    }

    fn parse_precedence(&mut self) -> Result<rdgen_ir::PrecedenceTable, String> {
        self.expect_ident("precedence")?;
        let rule = self.ident()?;
        self.expect_symbol('{')?;
        let mut levels = Vec::new();
        while !self.accept_symbol('}') {
            let associativity = match self.ident()?.as_str() {
                "left" => rdgen_ir::Associativity::Left,
                "right" => rdgen_ir::Associativity::Right,
                other => {
                    return Err(format!(
                        "unknown associativity '{}' at {}",
                        other,
                        self.previous().span.start
                    ))
                }
            };
            let mut operators = Vec::new();
            while self.starts_literal() {
                operators.push(
                    self.take_literal()
                        .expect("starts_literal guarantees a literal"),
                );
                self.accept_symbol(',');
            }
            if operators.is_empty() {
                return Err(self.error("precedence level requires an operator literal"));
            }
            let constructor = if self.accept_arrow() {
                Some(self.parse_constructor()?)
            } else {
                None
            };
            self.expect_symbol(';')?;
            levels.push(rdgen_ir::PrecedenceLevel {
                operators,
                associativity,
                constructor,
            });
        }
        Ok(rdgen_ir::PrecedenceTable { rule, levels })
    }

    fn alternatives(&mut self) -> Result<Vec<ParsedAlternative>, String> {
        let mut result = vec![self.alternative()?];
        while self.accept_symbol('|') {
            result.push(self.alternative()?);
        }
        Ok(result)
    }

    fn alternative(&mut self) -> Result<ParsedAlternative, String> {
        let elements = self.sequence()?;
        let constructor = if self.accept_arrow() {
            Some(self.parse_constructor()?)
        } else {
            None
        };
        let recovery = if self.at_ident("recover") {
            Some(self.parse_recovery()?)
        } else {
            None
        };
        Ok(ParsedAlternative {
            elements,
            constructor,
            recovery,
        })
    }

    fn sequence(&mut self) -> Result<Vec<rdgen_ir::Element>, String> {
        let mut result = Vec::new();
        while self.starts_element() {
            result.push(self.element()?);
            self.accept_symbol(',');
        }
        if result.is_empty() && !self.accept_epsilon() {
            return Err(self.error("expected a grammar element"));
        }
        Ok(result)
    }

    fn element(&mut self) -> Result<rdgen_ir::Element, String> {
        let label = if self.is_labeled_element() {
            let label = self.ident()?;
            self.expect_symbol(':')?;
            Some(label)
        } else {
            None
        };
        if self.at_ident("same_line") {
            let start = self.current().span.start;
            self.expect_ident("same_line")?;
            let element = self.element()?;
            let end = element_span(&element).end;
            return Ok(rdgen_ir::Element::SameLine {
                element: Box::new(element),
                span: rdgen_ir::Span::new(start, end),
            });
        }
        if self.at_ident("cut") {
            let span = self.current().span;
            self.expect_ident("cut")?;
            return Ok(rdgen_ir::Element::Cut { span });
        }
        if self.accept_symbol('{') {
            let start = self.previous().span.start;
            let alternatives = self
                .alternatives()?
                .into_iter()
                .map(|alternative| alternative.elements)
                .collect();
            let end = self.expect_symbol('}')?.span.end;
            let terminators = self.parse_repeat_terminators()?;
            return Ok(rdgen_ir::Element::Repeat {
                label,
                element: Box::new(rdgen_ir::Element::Group {
                    label: None,
                    alternatives,
                    span: rdgen_ir::Span::new(start, end),
                }),
                min: 0,
                max: None,
                terminators,
                span: rdgen_ir::Span::new(start, end),
            });
        }
        if self.accept_symbol('[') {
            let start = self.previous().span.start;
            let alternatives = self
                .alternatives()?
                .into_iter()
                .map(|alternative| alternative.elements)
                .collect();
            let end = self.expect_symbol(']')?.span.end;
            return Ok(rdgen_ir::Element::Repeat {
                label,
                element: Box::new(rdgen_ir::Element::Group {
                    label: None,
                    alternatives,
                    span: rdgen_ir::Span::new(start, end),
                }),
                min: 0,
                max: Some(1),
                terminators: Vec::new(),
                span: rdgen_ir::Span::new(start, end),
            });
        }
        let base = if let Some(text) = self.take_ident() {
            let span = self.previous().span;
            rdgen_ir::Element::Rule {
                label,
                rule: text,
                span,
            }
        } else if let Some(value) = self.take_literal() {
            let span = self.previous().span;
            rdgen_ir::Element::Literal { label, value, span }
        } else if self.accept_symbol('(') {
            let start = self.previous().span.start;
            let alternatives = self
                .alternatives()?
                .into_iter()
                .map(|alternative| alternative.elements)
                .collect();
            let end = self.expect_symbol(')')?.span.end;
            rdgen_ir::Element::Group {
                label,
                alternatives,
                span: rdgen_ir::Span::new(start, end),
            }
        } else {
            return Err(self.error("expected identifier, literal, or group"));
        };
        Ok(base)
    }

    fn parse_repeat_terminators(&mut self) -> Result<Vec<String>, String> {
        if !self.at_ident("until") {
            return Ok(Vec::new());
        }
        self.expect_ident("until")?;
        self.expect_symbol('{')?;
        let mut terminators = Vec::new();
        while !self.accept_symbol('}') {
            terminators.push(self.take_literal().ok_or_else(|| self.error("repeat terminator must be a literal"))?);
            self.accept_symbol(',');
        }
        if terminators.is_empty() {
            return Err(self.error("repeat terminator list cannot be empty"));
        }
        Ok(terminators)
    }

    fn is_labeled_element(&self) -> bool {
        matches!(self.current().kind, TokenKind::Ident(_))
            && self
                .tokens
                .get(self.position + 1)
                .map(|token| token.kind == TokenKind::Symbol(':'))
                == Some(true)
    }

    fn parse_constructor(&mut self) -> Result<rdgen_ir::Constructor, String> {
        let type_name = rdgen_ir::TypeName(self.ident()?);
        let mut fields = Vec::new();
        let start = self.previous().span.start;
        if self.accept_symbol('(') && !self.accept_symbol(')') {
            loop {
                let field = self.ident()?;
                self.expect_symbol(':')?;
                let source_label = self.ident()?;
                fields.push(rdgen_ir::FieldBinding {
                    field,
                    source_label,
                    span: rdgen_ir::Span::new(start, self.previous().span.end),
                });
                if self.accept_symbol(')') {
                    break;
                }
                self.expect_symbol(',')?;
            }
        }
        Ok(rdgen_ir::Constructor { type_name, fields })
    }

    fn parse_recovery(&mut self) -> Result<rdgen_ir::RecoveryPoint, String> {
        self.expect_ident("recover")?;
        let start = self.current().span.start;
        self.expect_symbol('{')?;
        self.expect_ident("sync")?;
        let mut sync_tokens = Vec::new();
        loop {
            let token = self
                .take_literal()
                .ok_or_else(|| self.error("recovery sync requires a literal"))?;
            if token.is_empty() {
                return Err(self.error("recovery sync literal must not be empty"));
            }
            if sync_tokens.iter().any(|existing| existing == &token) {
                return Err(self.error("recovery sync literals must be unique"));
            }
            sync_tokens.push(token);
            if !self.accept_symbol(',') {
                break;
            }
        }
        self.expect_symbol(';')?;
        let strategy_name = self.ident()?;
        let strategy = match strategy_name.as_str() {
            "skip_until_sync" => rdgen_ir::RecoveryStrategy::SkipUntilSync,
            "abort_rule" => rdgen_ir::RecoveryStrategy::AbortRule,
            "insert_token" => {
                let token = self
                    .take_literal()
                    .ok_or_else(|| self.error("insert_token requires a literal"))?;
                rdgen_ir::RecoveryStrategy::InsertToken(token)
            }
            other => {
                return Err(format!(
                    "unknown recovery strategy '{}' at {}",
                    other,
                    self.previous().span.start
                ))
            }
        };
        self.expect_symbol(';')?;
        let end = self.expect_symbol('}')?.span.end;
        Ok(rdgen_ir::RecoveryPoint {
            sync_tokens,
            strategy,
            span: rdgen_ir::Span::new(start, end),
        })
    }

    fn starts_element(&self) -> bool {
        matches!(
            self.current().kind,
            TokenKind::Ident(_)
                | TokenKind::Literal(_)
                | TokenKind::Symbol('(')
                | TokenKind::Symbol('{')
                | TokenKind::Symbol('[')
        )
    }
    fn starts_literal(&self) -> bool {
        matches!(self.current().kind, TokenKind::Literal(_))
    }

    fn is_start_directive(&self) -> bool {
        matches!(
            (
                self.tokens.get(self.position).map(|token| &token.kind),
                self.tokens.get(self.position + 1).map(|token| &token.kind),
                self.tokens.get(self.position + 2).map(|token| &token.kind),
            ),
            (
                Some(TokenKind::Ident(name)),
                Some(TokenKind::Ident(_)),
                Some(TokenKind::Symbol(';'))
            ) if name.eq_ignore_ascii_case("start")
        )
    }

    fn current(&self) -> &Token {
        &self.tokens[self.position]
    }
    fn previous(&self) -> &Token {
        &self.tokens[self.position - 1]
    }
    fn at_eof(&self) -> bool {
        matches!(self.current().kind, TokenKind::Eof)
    }
    fn take_ident(&mut self) -> Option<String> {
        if let TokenKind::Ident(value) = &self.current().kind {
            let value = value.clone();
            self.position += 1;
            Some(value)
        } else {
            None
        }
    }
    fn ident(&mut self) -> Result<String, String> {
        self.take_ident()
            .ok_or_else(|| self.error("expected identifier"))
    }
    fn expect_ident(&mut self, expected: &str) -> Result<(), String> {
        let value = self.ident()?;
        if value == expected {
            Ok(())
        } else {
            Err(format!(
                "expected '{}', got '{}' at {}",
                expected,
                value,
                self.previous().span.start
            ))
        }
    }
    fn at_ident(&self, expected: &str) -> bool {
        matches!(&self.current().kind, TokenKind::Ident(value) if value == expected)
    }
    fn take_literal(&mut self) -> Option<String> {
        if let TokenKind::Literal(value) = &self.current().kind {
            let value = value.clone();
            self.position += 1;
            Some(value)
        } else {
            None
        }
    }
    fn accept_epsilon(&mut self) -> bool {
        if matches!(self.current().kind, TokenKind::Epsilon) {
            self.position += 1;
            true
        } else {
            false
        }
    }
    fn accept_symbol(&mut self, symbol: char) -> bool {
        if self.current().kind == TokenKind::Symbol(symbol) {
            self.position += 1;
            true
        } else {
            false
        }
    }
    fn accept_arrow(&mut self) -> bool {
        if self.current().kind == TokenKind::Arrow {
            self.position += 1;
            true
        } else {
            false
        }
    }
    fn expect_symbol(&mut self, symbol: char) -> Result<&Token, String> {
        if self.accept_symbol(symbol) {
            Ok(self.previous())
        } else {
            Err(self.error(&format!(
                "expected '{}', got {:?}",
                symbol,
                self.current().kind
            )))
        }
    }
    fn error(&self, message: &str) -> String {
        format!("{} at {}", message, self.current().span.start)
    }
}

fn element_span(element: &rdgen_ir::Element) -> rdgen_ir::Span {
    match element {
        rdgen_ir::Element::Rule { span, .. }
        | rdgen_ir::Element::Token { span, .. }
        | rdgen_ir::Element::Literal { span, .. }
        | rdgen_ir::Element::Repeat { span, .. }
        | rdgen_ir::Element::Group { span, .. }
        | rdgen_ir::Element::SameLine { span, .. }
        | rdgen_ir::Element::Cut { span } => *span,
    }
}

fn starts_with_rule(elements: &[rdgen_ir::Element], rule: &str) -> bool {
    let Some(element) = elements.first() else {
        return false;
    };
    match element {
        rdgen_ir::Element::Rule { rule: name, .. } => name == rule,
        rdgen_ir::Element::Group { alternatives, .. } => alternatives
            .iter()
            .any(|alternative| starts_with_rule(alternative, rule)),
        rdgen_ir::Element::Repeat { element, min, .. } => {
            starts_with_rule(std::slice::from_ref(element.as_ref()), rule)
                || (*min == 0 && starts_with_rule(&elements[1..], rule))
        }
        rdgen_ir::Element::SameLine { element, .. } => {
            starts_with_rule(std::slice::from_ref(element.as_ref()), rule)
        }
        rdgen_ir::Element::Cut { .. } => false,
        rdgen_ir::Element::Literal { .. } | rdgen_ir::Element::Token { .. } => false,
    }
}

fn resolve_symbols(rules: &mut [rdgen_ir::Rule]) -> Result<(), String> {
    let rule_names: std::collections::HashSet<String> =
        rules.iter().map(|rule| rule.name.clone()).collect();
    for rule in rules {
        for alternative in &mut rule.alternatives {
            for element in &mut alternative.elements {
                resolve_element(element, &rule_names)?;
            }
        }
    }
    Ok(())
}

fn validate_constructors(rules: &[rdgen_ir::Rule]) -> Result<(), String> {
    for rule in rules {
        for alternative in &rule.alternatives {
            let mut labels = Vec::new();
            collect_labels(&alternative.elements, &mut labels);
            let mut unique_labels = std::collections::HashSet::new();
            for label in &labels {
                if !unique_labels.insert(label.as_str()) {
                    return Err(format!(
                        "duplicate element label '{}' in rule '{}'",
                        label, rule.name
                    ));
                }
            }
            let mut fields = std::collections::HashSet::new();
            for binding in &alternative.constructor.fields {
                if !fields.insert(binding.field.as_str()) {
                    return Err(format!(
                        "duplicate constructor field '{}' in rule '{}'",
                        binding.field, rule.name
                    ));
                }
                if !unique_labels.contains(binding.source_label.as_str()) {
                    return Err(format!(
                        "constructor field '{}' references unknown label '{}' in rule '{}'",
                        binding.field, binding.source_label, rule.name
                    ));
                }
            }
        }
    }
    Ok(())
}

fn collect_labels(elements: &[rdgen_ir::Element], labels: &mut Vec<String>) {
    for element in elements {
        match element {
            rdgen_ir::Element::Rule { label, .. }
            | rdgen_ir::Element::Token { label, .. }
            | rdgen_ir::Element::Literal { label, .. } => {
                if let Some(label) = label {
                    labels.push(label.clone());
                }
            }
            rdgen_ir::Element::Group {
                label,
                alternatives,
                ..
            } => {
                if let Some(label) = label {
                    labels.push(label.clone());
                }
                for alternative in alternatives {
                    collect_labels(alternative, labels);
                }
            }
            rdgen_ir::Element::Repeat { label, element, .. } => {
                if let Some(label) = label {
                    labels.push(label.clone());
                }
                collect_labels(std::slice::from_ref(element.as_ref()), labels)
            }
            rdgen_ir::Element::SameLine { element, .. } => {
                collect_labels(std::slice::from_ref(element.as_ref()), labels)
            }
            rdgen_ir::Element::Cut { .. } => {}
        }
    }
}

fn resolve_element(
    element: &mut rdgen_ir::Element,
    rule_names: &std::collections::HashSet<String>,
) -> Result<(), String> {
    match element {
        rdgen_ir::Element::Rule { rule, label, span } => {
            let name = rule.clone();
            let label = label.clone();
            let span = *span;
            if rule_names.contains(&name) {
                return Ok(());
            }
            if is_builtin_terminal(&name) {
                *element = rdgen_ir::Element::Token {
                    label,
                    token: name,
                    span,
                };
                return Ok(());
            }
            Err(format!(
                "undefined rule or terminal '{}' at {}",
                name, span.start
            ))
        }
        rdgen_ir::Element::Group { alternatives, .. } => {
            for alternative in alternatives {
                for child in alternative {
                    resolve_element(child, rule_names)?;
                }
            }
            Ok(())
        }
        rdgen_ir::Element::Repeat { element, .. } => resolve_element(element, rule_names),
        rdgen_ir::Element::SameLine { element, .. } => resolve_element(element, rule_names),
        rdgen_ir::Element::Cut { .. } => Ok(()),
        rdgen_ir::Element::Literal { .. } | rdgen_ir::Element::Token { .. } => Ok(()),
    }
}

fn is_builtin_terminal(name: &str) -> bool {
    matches!(
        name,
        "letter"
            | "digit"
            | "hex_digit"
            | "java_ident_start"
            | "java_ident_part"
            | "any_char"
            | "any_char_except_quote"
            | "any_char_except_newline"
            | "any_char_except_slash"
            | "any_char_except_quote_or_open_brace_or_backslash"
            | "any_char_except_slash_or_open_brace_or_backslash"
            | "text_block_char"
    )
}

fn reject_indirect_left_recursion(rules: &[rdgen_ir::Rule]) -> Result<(), String> {
    let names: std::collections::HashSet<&str> =
        rules.iter().map(|rule| rule.name.as_str()).collect();
    let nullable = nullable_rules(rules);
    let mut graph = std::collections::HashMap::<&str, Vec<&str>>::new();
    for rule in rules {
        let mut first = Vec::new();
        for alternative in &rule.alternatives {
            first_rule_names(&alternative.elements, &names, &nullable, &mut first);
        }
        graph.insert(rule.name.as_str(), first);
    }

    for rule in rules {
        let mut path = vec![rule.name.as_str()];
        if let Some(cycle) = find_cycle(rule.name.as_str(), rule.name.as_str(), &graph, &mut path) {
            return Err(format!("left recursion through {}", cycle.join(" -> ")));
        }
    }
    Ok(())
}

fn validate_precedence(
    precedence: &[rdgen_ir::PrecedenceTable],
    rules: &[rdgen_ir::Rule],
) -> Result<(), String> {
    let names: std::collections::HashSet<&str> =
        rules.iter().map(|rule| rule.name.as_str()).collect();
    let mut seen = std::collections::HashSet::new();
    let rule_map: std::collections::HashMap<&str, &rdgen_ir::Rule> = rules
        .iter()
        .map(|rule| (rule.name.as_str(), rule))
        .collect();
    for table in precedence {
        if !names.contains(table.rule.as_str()) {
            return Err(format!(
                "precedence table references undefined rule '{}'",
                table.rule
            ));
        }
        if table.levels.is_empty() {
            return Err(format!(
                "precedence table for rule '{}' must contain at least one level",
                table.rule
            ));
        }
        if !seen.insert(table.rule.as_str()) {
            return Err(format!(
                "duplicate precedence table for rule '{}'",
                table.rule
            ));
        }
        let mut literals = std::collections::HashSet::new();
        let mut visited = std::collections::HashSet::new();
        collect_reachable_literals(
            &table.rule,
            &rule_map,
            &mut visited,
            &mut literals,
        );
        let mut operators = std::collections::HashSet::new();
        for level in &table.levels {
            for operator in &level.operators {
                if operator.is_empty() {
                    return Err(format!(
                        "precedence operator in rule '{}' must not be empty",
                        table.rule
                    ));
                }
                if !operators.insert(operator.as_str()) {
                    return Err(format!(
                        "duplicate precedence operator {:?} in rule '{}'",
                        operator, table.rule
                    ));
                }
                if !literals.contains(operator.as_str()) {
                    return Err(format!(
                        "precedence operator {:?} in rule '{}' does not occur as a grammar literal",
                        operator, table.rule
                    ));
                }
            }
        }
    }
    Ok(())
}

fn validate_precedence_constructors(
    precedence: &[rdgen_ir::PrecedenceTable],
) -> Result<(), String> {
    for table in precedence {
        for level in &table.levels {
            let Some(constructor) = &level.constructor else {
                continue;
            };
            let required = ["left", "operator", "right"];
            if constructor.fields.len() != required.len()
                || required.iter().any(|field| {
                    constructor
                        .fields
                        .iter()
                        .filter(|binding| binding.source_label == *field)
                        .count()
                        != 1
                })
            {
                return Err(format!(
                    "precedence constructor '{}' for rule '{}' must bind exactly left, operator, and right",
                    constructor.type_name.0, table.rule
                ));
            }
            let mut field_names = std::collections::HashSet::new();
            for field in &constructor.fields {
                if !field_names.insert(field.field.as_str()) {
                    return Err(format!(
                        "precedence constructor '{}' for rule '{}' has duplicate output field '{}'",
                        constructor.type_name.0, table.rule, field.field
                    ));
                }
            }
        }
    }
    Ok(())
}

fn collect_reachable_literals(
    rule_name: &str,
    rules: &std::collections::HashMap<&str, &rdgen_ir::Rule>,
    visited: &mut std::collections::HashSet<String>,
    literals: &mut std::collections::HashSet<String>,
) {
    if !visited.insert(rule_name.to_owned()) {
        return;
    }
    if let Some(rule) = rules.get(rule_name) {
        for alternative in &rule.alternatives {
            collect_literal_values(&alternative.elements, rules, visited, literals);
        }
    }
}

fn collect_literal_values(
    elements: &[rdgen_ir::Element],
    rules: &std::collections::HashMap<&str, &rdgen_ir::Rule>,
    visited: &mut std::collections::HashSet<String>,
    literals: &mut std::collections::HashSet<String>,
) {
    for element in elements {
        match element {
            rdgen_ir::Element::Literal { value, .. } => {
                literals.insert(value.clone());
            }
            rdgen_ir::Element::Group { alternatives, .. } => {
                for alternative in alternatives {
                    collect_literal_values(alternative, rules, visited, literals);
                }
            }
            rdgen_ir::Element::Repeat { element, .. } => {
                collect_literal_values(
                    std::slice::from_ref(element.as_ref()),
                    rules,
                    visited,
                    literals,
                );
            }
            rdgen_ir::Element::SameLine { element, .. } => {
                collect_literal_values(std::slice::from_ref(element.as_ref()), rules, visited, literals);
            }
            rdgen_ir::Element::Cut { .. } => {}
            rdgen_ir::Element::Rule { rule, .. } => {
                collect_reachable_literals(rule, rules, visited, literals);
            }
            rdgen_ir::Element::Token { .. } => {}
        }
    }
}

fn validate_rule_names(rules: &[rdgen_ir::Rule]) -> Result<(), String> {
    let mut names = std::collections::HashSet::new();
    for rule in rules {
        if !names.insert(rule.name.as_str()) {
            return Err(format!("duplicate rule '{}'", rule.name));
        }
    }
    Ok(())
}

fn validate_repetition_progress(rules: &[rdgen_ir::Rule]) -> Result<(), String> {
    let nullable = nullable_rules(rules);
    for rule in rules {
        for alternative in &rule.alternatives {
            validate_repetition_elements(&alternative.elements, &nullable, &rule.name)?;
        }
    }
    Ok(())
}

fn validate_repetition_elements(
    elements: &[rdgen_ir::Element],
    nullable: &std::collections::HashSet<&str>,
    rule_name: &str,
) -> Result<(), String> {
    for element in elements {
        match element {
            rdgen_ir::Element::Repeat { element, .. } => {
                if nullable_element(element, nullable) {
                    return Err(format!(
                        "repetition in rule '{}' can match empty input",
                        rule_name
                    ));
                }
                validate_repetition_elements(std::slice::from_ref(element.as_ref()), nullable, rule_name)?;
            }
            rdgen_ir::Element::Group { alternatives, .. } => {
                for alternative in alternatives {
                    validate_repetition_elements(alternative, nullable, rule_name)?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn first_rule_names<'a>(
    elements: &'a [rdgen_ir::Element],
    names: &std::collections::HashSet<&'a str>,
    nullable: &std::collections::HashSet<&'a str>,
    output: &mut Vec<&'a str>,
) {
    for (index, element) in elements.iter().enumerate() {
        match element {
            rdgen_ir::Element::Rule { rule, .. } => {
                if names.contains(rule.as_str()) {
                    output.push(rule.as_str());
                }
            }
            rdgen_ir::Element::Group { alternatives, .. } => {
                for alternative in alternatives {
                    first_rule_names(alternative, names, nullable, output);
                }
            }
            rdgen_ir::Element::Repeat { element, .. } => {
                first_rule_names(std::slice::from_ref(element.as_ref()), names, nullable, output);
            }
            rdgen_ir::Element::SameLine { element, .. } => {
                first_rule_names(std::slice::from_ref(element.as_ref()), names, nullable, output);
            }
            rdgen_ir::Element::Cut { .. } => {}
            rdgen_ir::Element::Literal { .. } | rdgen_ir::Element::Token { .. } => {}
        }
        if !nullable_element(element, nullable) || index + 1 == elements.len() {
            break;
        }
    }
}

fn nullable_rules<'a>(rules: &'a [rdgen_ir::Rule]) -> std::collections::HashSet<&'a str> {
    let mut nullable = std::collections::HashSet::new();
    loop {
        let before = nullable.len();
        for rule in rules {
            if rule
                .alternatives
                .iter()
                .any(|alternative| nullable_sequence(&alternative.elements, &nullable))
            {
                nullable.insert(rule.name.as_str());
            }
        }
        if nullable.len() == before {
            return nullable;
        }
    }
}

fn nullable_element(
    element: &rdgen_ir::Element,
    nullable: &std::collections::HashSet<&str>,
) -> bool {
    match element {
        rdgen_ir::Element::Repeat { min, .. } => *min == 0,
        rdgen_ir::Element::Group { alternatives, .. } => alternatives
            .iter()
            .any(|alternative| nullable_sequence(alternative, nullable)),
        rdgen_ir::Element::SameLine { element, .. } => nullable_element(element, nullable),
        rdgen_ir::Element::Cut { .. } => true,
        rdgen_ir::Element::Rule { rule, .. } => nullable.contains(rule.as_str()),
        | rdgen_ir::Element::Token { .. }
        | rdgen_ir::Element::Literal { .. } => false,
    }
}

fn nullable_sequence(
    elements: &[rdgen_ir::Element],
    nullable: &std::collections::HashSet<&str>,
) -> bool {
    elements
        .iter()
        .all(|element| nullable_element(element, nullable))
}

fn find_cycle<'a>(
    origin: &'a str,
    current: &'a str,
    graph: &std::collections::HashMap<&'a str, Vec<&'a str>>,
    path: &mut Vec<&'a str>,
) -> Option<Vec<String>> {
    for next in graph.get(current).into_iter().flatten() {
        if *next == origin {
            return Some(
                path.iter()
                    .chain(std::iter::once(next))
                    .map(|name| (*name).to_owned())
                    .collect(),
            );
        }
        if !path.contains(next) {
            path.push(next);
            if let Some(cycle) = find_cycle(origin, next, graph, path) {
                return Some(cycle);
            }
            path.pop();
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiles_sequences_alternatives_and_ebnf_quantifiers() {
        let grammar = compile(
            r#"
            (* comment *) grammar Demo;
            expr = number , { "+" , number } ;
            number = "0" | "1" ;
        "#,
        )
        .unwrap();
        assert_eq!(grammar.name, "Demo");
        assert_eq!(grammar.rules.len(), 2);
        assert_eq!(grammar.rules[0].alternatives.len(), 1);
        assert!(matches!(
            grammar.rules[0].alternatives[0].elements[1],
            rdgen_ir::Element::Repeat {
                min: 0,
                max: None,
                ..
            }
        ));
        assert_eq!(grammar.rules[1].alternatives.len(), 2);
    }

    #[test]
    fn parses_explicit_precedence_levels() {
        let grammar = compile("grammar Expr; precedence expr { left \"+\", \"-\"; right \"^\"; } expr = atom, ( \"+\" | \"-\" | \"^\" ); atom = \"x\";").unwrap();
        assert_eq!(grammar.precedence.len(), 1);
        assert_eq!(grammar.precedence[0].rule, "expr");
        assert_eq!(grammar.precedence[0].levels[0].operators, vec!["+", "-"]);
        assert_eq!(
            grammar.precedence[0].levels[1].associativity,
            rdgen_ir::Associativity::Right
        );
        assert!(grammar.precedence[0].levels[0].constructor.is_none());
    }

    #[test]
    fn parses_precedence_constructor_annotations() {
        let grammar = compile(
            "grammar Expr; precedence expr { left \"+\" => Binary(left: left, operator: operator, right: right); } expr = atom, \"+\", atom; atom = \"x\";",
        )
        .unwrap();
        let constructor = grammar.precedence[0].levels[0]
            .constructor
            .as_ref()
            .unwrap();
        assert_eq!(constructor.type_name.0, "Binary");
        assert_eq!(constructor.fields.len(), 3);
    }

    #[test]
    fn rejects_precedence_for_unknown_rule() {
        let error = compile("grammar Expr; precedence missing { left \"+\"; } expr = \"x\";").unwrap_err();
        assert!(error.contains("precedence table references undefined rule 'missing'"));
    }

    #[test]
    fn rejects_duplicate_rule_names() {
        let error = compile("grammar Bad; item = \"a\"; item = \"b\";").unwrap_err();
        assert!(error.contains("duplicate rule 'item'"));
    }

    #[test]
    fn rejects_duplicate_precedence_tables() {
        let error = compile("grammar Expr; precedence expr { left \"+\"; } precedence expr { left \"-\"; } expr = \"x\", ( \"+\" | \"-\" );").unwrap_err();
        assert!(error.contains("duplicate precedence table for rule 'expr'"));
    }

    #[test]
    fn rejects_empty_precedence_tables() {
        let error = compile("grammar Expr; precedence expr { } expr = \"x\";").unwrap_err();
        assert!(error.contains("precedence table for rule 'expr' must contain at least one level"));
    }

    #[test]
    fn rejects_duplicate_precedence_operators() {
        let error = compile(
            "grammar Expr; precedence expr { left \"+\"; left \"+\"; } expr = \"x\", \"+\";",
        )
        .unwrap_err();
        assert!(error.contains("duplicate precedence operator \"+\" in rule 'expr'"));
    }

    #[test]
    fn rejects_empty_precedence_operators() {
        let error = compile(
            "grammar Expr; precedence expr { left \"\"; } expr = \"x\";",
        )
        .unwrap_err();
        assert!(error.contains("precedence operator in rule 'expr' must not be empty"));
    }

    #[test]
    fn rejects_precedence_operators_missing_from_grammar() {
        let error = compile(
            "grammar Expr; precedence expr { left \"+\"; } expr = \"x\";",
        )
        .unwrap_err();
        assert!(error.contains(
            "precedence operator \"+\" in rule 'expr' does not occur as a grammar literal"
        ));
    }

    #[test]
    fn rejects_incomplete_precedence_constructor_annotations() {
        let error = compile(
            "grammar Expr; precedence expr { left \"+\" => Binary(left: left); } expr = \"x\", \"+\";",
        )
        .unwrap_err();
        assert!(error.contains(
            "precedence constructor 'Binary' for rule 'expr' must bind exactly left, operator, and right"
        ));
    }

    #[test]
    fn rejects_duplicate_precedence_constructor_fields() {
        let error = compile(
            "grammar Expr; precedence expr { left \"+\" => Binary(left: left, left: operator, right: right); } expr = \"x\", \"+\";",
        )
        .unwrap_err();
        assert!(error.contains(
            "precedence constructor 'Binary' for rule 'expr' has duplicate output field 'left'"
        ));
    }

    #[test]
    fn precedence_coverage_ignores_unreachable_literals() {
        let error = compile(
            "grammar Expr; precedence expr { left \"+\"; } expr = atom; atom = \"x\"; unrelated = \"+\";",
        )
        .unwrap_err();
        assert!(error.contains(
            "precedence operator \"+\" in rule 'expr' does not occur as a grammar literal"
        ));
    }

    #[test]
    fn preserves_recovery_annotations_in_the_ir() {
        let grammar = compile("grammar Demo; statement = \"x\" => Statement() recover { sync \";\", \"}\"; skip_until_sync; }; ").unwrap();
        let recovery = grammar.rules[0].alternatives[0].recovery.as_ref().unwrap();
        assert_eq!(recovery.sync_tokens, vec![";", "}"]);
        assert_eq!(recovery.strategy, rdgen_ir::RecoveryStrategy::SkipUntilSync);
    }

    #[test]
    fn rejects_empty_recovery_sync_literals() {
        let error = compile(
            "grammar Bad; statement = \"x\" => Statement() recover { sync \"\"; skip_until_sync; };",
        )
        .unwrap_err();
        assert!(error.contains("recovery sync literal must not be empty"));
    }

    #[test]
    fn rejects_duplicate_recovery_sync_literals() {
        let error = compile(
            "grammar Bad; statement = \"x\" => Statement() recover { sync \";\", \";\"; skip_until_sync; };",
        )
        .unwrap_err();
        assert!(error.contains("recovery sync literals must be unique"));
    }

    #[test]
    fn rejects_direct_left_recursion_with_source_offset() {
        let error = compile("grammar Bad; expr = expr , \"+\" , atom | atom;").unwrap_err();
        assert!(error.contains("direct left recursion in rule 'expr'"));
    }

    #[test]
    fn rejects_indirect_left_recursion() {
        let error = compile("grammar Bad; a = b ; b = a | \"x\" ;").unwrap_err();
        assert!(error.contains("left recursion through a -> b -> a"));
    }

    #[test]
    fn rejects_left_recursion_hidden_behind_an_optional_prefix() {
        let error =
            compile("grammar Bad; expr = [prefix] , expr | atom; prefix = \"p\"; atom = \"x\";")
                .unwrap_err();
        assert!(error.contains("direct left recursion in rule 'expr'"));
    }

    #[test]
    fn rejects_left_recursion_through_nullable_rule_reference() {
        let error = compile("grammar Bad; expr = prefix, expr | atom; prefix = ε; atom = \"x\";").unwrap_err();
        assert!(error.contains("left recursion through expr -> expr"));
    }

    #[test]
    fn rejects_left_recursion_through_nullable_group_and_repetition() {
        let error = compile(
            "grammar Bad; a = ( [ b ] ) , \"x\" | \"a\" ; b = { c } , a | \"b\" ; c = \"c\" ;",
        )
        .unwrap_err();
        assert!(error.contains("left recursion through a -> b -> a"));
    }

    #[test]
    fn accepts_epsilon_and_nested_groups() {
        let grammar = compile("grammar Demo; start = ( \"a\" | ε ) , [ \"b\" ] ;").unwrap();
        assert!(matches!(
            grammar.rules[0].alternatives[0].elements[0],
            rdgen_ir::Element::Group { .. }
        ));
        assert!(matches!(
            grammar.rules[0].alternatives[0].elements[1],
            rdgen_ir::Element::Repeat { max: Some(1), .. }
        ));
    }

    #[test]
    fn preserves_same_line_wrappers_in_the_ir() {
        let grammar = compile("grammar Demo; start = \"return\", [ same_line expr ]; expr = \"x\";").unwrap();
        let optional = &grammar.rules[0].alternatives[0].elements[1];
        assert!(matches!(
            optional,
            rdgen_ir::Element::Repeat { element, max: Some(1), .. }
                if matches!(element.as_ref(), rdgen_ir::Element::Group { alternatives, .. }
                    if matches!(alternatives.first().and_then(|alternative| alternative.first()), Some(rdgen_ir::Element::SameLine { .. })))
        ));
    }

    #[test]
    fn rejects_repetition_of_nullable_element() {
        let error = compile("grammar Bad; start = { ε };").unwrap_err();
        assert!(error.contains("repetition in rule 'start' can match empty input"));
    }

    #[test]
    fn parses_the_bascal_starter_fixture() {
        let grammar = compile(include_str!("../../grammars/bascal.bcl.rdg")).unwrap();
        assert_eq!(grammar.name, "Bascal");
        assert_eq!(grammar.start, "program");
        assert!(grammar.rules.iter().any(|rule| rule.name == "expr"));
        assert_eq!(grammar.precedence[0].rule, "expr");
        assert_eq!(grammar.precedence[0].levels.len(), 9);
    }

    #[test]
    fn accepts_explicit_start_rule_before_lexical_rules() {
        let grammar = compile(
            "grammar Demo; start program; lexical identifier = \"x\"; program = identifier;",
        )
        .unwrap();
        assert_eq!(grammar.start, "program");
        assert_eq!(grammar.rules[0].name, "identifier");
        assert!(grammar.rules[0].lexical);
    }

    #[test]
    fn rejects_unknown_explicit_start_rule() {
        let error = compile("grammar Demo; start missing; item = \"x\";").unwrap_err();
        assert!(error.contains("start rule 'missing' is not declared"));
    }

    #[test]
    fn preserves_all_bascal_top_level_prefix_forms() {
        let grammar = compile(include_str!("../../grammars/bascal.bcl.rdg")).unwrap();
        let file_item = grammar
            .rules
            .iter()
            .find(|rule| rule.name == "file_item")
            .unwrap();
        let referenced_rules = file_item
            .alternatives
            .iter()
            .filter_map(|alternative| match alternative.elements.first() {
                Some(rdgen_ir::Element::Rule { rule, .. }) => Some(rule.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(referenced_rules.contains(&"program_decl"));
        assert!(referenced_rules.contains(&"library_decl"));
        assert!(referenced_rules.contains(&"shared_decl"));
        assert!(referenced_rules.contains(&"top_level_statement"));
        let statement = grammar
            .rules
            .iter()
            .find(|rule| rule.name == "statement_core")
            .unwrap();
        assert!(statement.alternatives.iter().any(|alternative| {
            matches!(
                alternative.elements.first(),
                Some(rdgen_ir::Element::Rule { rule, .. }) if rule == "comment_stmt"
            )
        }));
    }

    #[test]
    fn parses_expression_assignments_and_record_literals() {
        let grammar = compile(
            "grammar Demo; start statement; statement = expr; expr = primary; primary = identifier, \"(\", [ expr ], \")\" | \"{\", field_init, \"}\" | identifier; field_init = identifier, \":\", expr; identifier = letter, { letter };",
        );
        assert!(grammar.is_ok(), "expression assignment fixture: {grammar:?}");
    }

    #[test]
    fn preserves_repeat_terminators_in_the_ir() {
        let grammar = compile(
            "grammar Blocks; start block; block = \"begin\", { item } until { \"end\" }, \"end\"; item = \"item\";",
        )
        .unwrap();
        let repeat = &grammar.rules[0].alternatives[0].elements[1];
        assert!(matches!(
            repeat,
            rdgen_ir::Element::Repeat { terminators, .. }
                if terminators == &["end".to_owned()]
        ));
    }

    #[test]
    fn resolves_builtin_terminals_and_rejects_unknown_references() {
        let grammar = compile("grammar Demo; start = letter , missing; ").unwrap_err();
        assert!(grammar.contains("undefined rule or terminal 'missing'"));

        let grammar = compile("grammar Demo; start = letter , \"x\"; ").unwrap();
        assert!(
            matches!(&grammar.rules[0].alternatives[0].elements[0], rdgen_ir::Element::Token { token, .. } if token == "letter")
        );
    }

    #[test]
    fn rejects_constructor_bindings_without_matching_element_labels() {
        let error =
            compile("grammar Demo; start = value: \"x\" => Node(other: value2); ").unwrap_err();
        assert!(error.contains("constructor field 'other' references unknown label 'value2'"));
    }

    #[test]
    fn rejects_duplicate_element_and_constructor_labels() {
        let duplicate_element =
            compile("grammar Demo; start = value: \"x\", value: \"y\"; ").unwrap_err();
        assert!(duplicate_element.contains("duplicate element label 'value'"));
        let duplicate_field =
            compile("grammar Demo; start = value: \"x\" => Node(a: value, a: value); ")
                .unwrap_err();
        assert!(duplicate_field.contains("duplicate constructor field 'a'"));
    }

    #[test]
    fn parses_the_distill_starter_fixture() {
        let grammar = compile(include_str!("../../grammars/distill.matcher.rdg")).unwrap();
        assert_eq!(grammar.name, "DistillMatcher");
        assert!(grammar.rules.iter().any(|rule| rule.name == "postfix_expr"));
    }

    #[test]
    fn preserves_labeled_constructor_annotations_in_the_typed_ir() {
        let grammar = compile(
            "grammar Expr; expr = value: number => Number(value: value) | left: atom, \"+\", right: atom => Add(left: left, right: right); number = \"n\"; atom = \"x\";",
        )
        .unwrap();
        let alternative = &grammar.rules[0].alternatives[0];
        assert_eq!(alternative.constructor.type_name.0, "Number");
        assert_eq!(alternative.constructor.fields[0].source_label, "value");
        assert!(matches!(
            &alternative.elements[0],
            rdgen_ir::Element::Rule { label: Some(label), rule, .. }
                if label == "value" && rule == "number"
        ));
    }

    #[test]
    fn preserves_labels_on_group_and_repeat_elements() {
        let grammar = compile(
            "grammar Expr; expr = values: { value: atom } => Values(values: values); atom = \"x\";",
        )
        .unwrap();
        let element = &grammar.rules[0].alternatives[0].elements[0];
        assert!(
            matches!(element, rdgen_ir::Element::Repeat { label: Some(label), .. } if label == "values")
        );
        assert!(grammar.rules[0].alternatives[0]
            .constructor
            .fields
            .iter()
            .any(|field| field.source_label == "values"));
    }
}
