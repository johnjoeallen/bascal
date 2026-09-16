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
                    value.push(escaped);
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
        while !self.at_eof() {
            let start = self.current().span.start;
            let rule_name = self.ident()?;
            self.expect_symbol('=')?;
            let alternatives = self.alternatives()?;
            self.expect_symbol(';')?;
            if alternatives
                .iter()
                .any(|alternative| starts_with_rule(alternative, &rule_name))
            {
                return Err(format!(
                    "direct left recursion in rule '{}' at {}",
                    rule_name, start
                ));
            }
            rules.push(rdgen_ir::Rule {
                name: rule_name.clone(),
                output: rdgen_ir::TypeName(rule_name),
                alternatives: alternatives
                    .into_iter()
                    .map(|elements| rdgen_ir::Alternative {
                        span: elements
                            .first()
                            .map_or(rdgen_ir::Span::new(start, start), element_span),
                        elements,
                        constructor: rdgen_ir::Constructor {
                            type_name: rdgen_ir::TypeName("".into()),
                            fields: Vec::new(),
                        },
                        recovery: None,
                    })
                    .collect(),
                span: rdgen_ir::Span::new(start, self.previous().span.end),
            });
        }
        Ok(rdgen_ir::Grammar {
            name,
            rules,
            tokens: Vec::new(),
        })
    }

    fn alternatives(&mut self) -> Result<Vec<Vec<rdgen_ir::Element>>, String> {
        let mut result = vec![self.sequence()?];
        while self.accept_symbol('|') {
            result.push(self.sequence()?);
        }
        Ok(result)
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
        if self.accept_symbol('{') {
            let start = self.previous().span.start;
            let alternatives = self.alternatives()?;
            let end = self.expect_symbol('}')?.span.end;
            return Ok(rdgen_ir::Element::Repeat {
                element: Box::new(rdgen_ir::Element::Group {
                    alternatives,
                    span: rdgen_ir::Span::new(start, end),
                }),
                min: 0,
                max: None,
                span: rdgen_ir::Span::new(start, end),
            });
        }
        if self.accept_symbol('[') {
            let start = self.previous().span.start;
            let alternatives = self.alternatives()?;
            let end = self.expect_symbol(']')?.span.end;
            return Ok(rdgen_ir::Element::Repeat {
                element: Box::new(rdgen_ir::Element::Group {
                    alternatives,
                    span: rdgen_ir::Span::new(start, end),
                }),
                min: 0,
                max: Some(1),
                span: rdgen_ir::Span::new(start, end),
            });
        }
        let base = if let Some(text) = self.take_ident() {
            let span = self.previous().span;
            rdgen_ir::Element::Rule {
                label: None,
                rule: text,
                span,
            }
        } else if let Some(value) = self.take_literal() {
            let span = self.previous().span;
            rdgen_ir::Element::Literal {
                label: None,
                value,
                span,
            }
        } else if self.accept_symbol('(') {
            let start = self.previous().span.start;
            let alternatives = self.alternatives()?;
            let end = self.expect_symbol(')')?.span.end;
            rdgen_ir::Element::Group {
                alternatives,
                span: rdgen_ir::Span::new(start, end),
            }
        } else {
            return Err(self.error("expected identifier, literal, or group"));
        };
        Ok(base)
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
        | rdgen_ir::Element::Group { span, .. } => *span,
    }
}

fn starts_with_rule(elements: &[rdgen_ir::Element], rule: &str) -> bool {
    matches!(elements.first(), Some(rdgen_ir::Element::Rule { rule: name, .. }) if name == rule)
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
    fn rejects_direct_left_recursion_with_source_offset() {
        let error = compile("grammar Bad; expr = expr , \"+\" , atom | atom;").unwrap_err();
        assert!(error.contains("direct left recursion in rule 'expr'"));
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
    fn parses_the_bascal_starter_fixture() {
        let grammar = compile(include_str!("../../grammars/bascal.bcl.rdg")).unwrap();
        assert_eq!(grammar.name, "Bascal");
        assert!(grammar.rules.iter().any(|rule| rule.name == "expr"));
    }
}
