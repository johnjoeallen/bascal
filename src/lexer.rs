use crate::diagnostics::SourcePos;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub pos: SourcePos,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexes_identifiers_with_basic_suffixes() {
        let tokens = Lexer::new("test.bcl", "name$ count% amount! distance# id&").lex();
        let idents = tokens
            .into_iter()
            .filter_map(|token| match token.kind {
                TokenKind::Ident(value) => Some(value),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(
            idents,
            vec!["name$", "count%", "amount!", "distance#", "id&"]
        );
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Ident(String),
    Number(i64),
    String(String),
    Comment(String),
    Newline,
    LParen,
    RParen,
    Comma,
    Colon,
    Plus,
    Minus,
    Star,
    Slash,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Eof,
}

pub struct Lexer<'a> {
    filename: &'a str,
    chars: Vec<char>,
    index: usize,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(filename: &'a str, source: &'a str) -> Self {
        Self {
            filename,
            chars: source.chars().collect(),
            index: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn lex(mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while let Some(ch) = self.peek() {
            match ch {
                ' ' | '\t' | '\r' => {
                    self.advance();
                }
                '\n' => {
                    let pos = self.pos();
                    self.advance();
                    tokens.push(Token {
                        kind: TokenKind::Newline,
                        pos,
                    });
                }
                '\'' => tokens.push(self.comment()),
                '"' => tokens.push(self.string()),
                '0'..='9' => tokens.push(self.number()),
                'A'..='Z' | 'a'..='z' | '_' => tokens.push(self.ident()),
                '(' => tokens.push(self.single(TokenKind::LParen)),
                ')' => tokens.push(self.single(TokenKind::RParen)),
                ',' => tokens.push(self.single(TokenKind::Comma)),
                ':' => tokens.push(self.single(TokenKind::Colon)),
                '+' => tokens.push(self.single(TokenKind::Plus)),
                '-' => tokens.push(self.single(TokenKind::Minus)),
                '*' => tokens.push(self.single(TokenKind::Star)),
                '/' => tokens.push(self.single(TokenKind::Slash)),
                '=' => tokens.push(self.single(TokenKind::Eq)),
                '<' => {
                    let pos = self.pos();
                    self.advance();
                    let kind = match self.peek() {
                        Some('=') => {
                            self.advance();
                            TokenKind::Le
                        }
                        Some('>') => {
                            self.advance();
                            TokenKind::Ne
                        }
                        _ => TokenKind::Lt,
                    };
                    tokens.push(Token { kind, pos });
                }
                '>' => {
                    let pos = self.pos();
                    self.advance();
                    let kind = if self.peek() == Some('=') {
                        self.advance();
                        TokenKind::Ge
                    } else {
                        TokenKind::Gt
                    };
                    tokens.push(Token { kind, pos });
                }
                _ => {
                    self.advance();
                }
            }
        }
        tokens.push(Token {
            kind: TokenKind::Eof,
            pos: self.pos(),
        });
        tokens
    }

    fn ident(&mut self) -> Token {
        let pos = self.pos();
        let mut value = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '.' {
                value.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        if matches!(self.peek(), Some('%' | '$' | '!' | '#' | '&')) {
            value.push(self.peek().unwrap());
            self.advance();
        }
        Token {
            kind: TokenKind::Ident(value),
            pos,
        }
    }

    fn number(&mut self) -> Token {
        let pos = self.pos();
        let mut value = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                value.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        Token {
            kind: TokenKind::Number(value.parse().unwrap_or(0)),
            pos,
        }
    }

    fn string(&mut self) -> Token {
        let pos = self.pos();
        self.advance();
        let mut value = String::new();
        while let Some(ch) = self.peek() {
            self.advance();
            if ch == '"' {
                break;
            }
            value.push(ch);
        }
        Token {
            kind: TokenKind::String(value),
            pos,
        }
    }

    fn comment(&mut self) -> Token {
        let pos = self.pos();
        self.advance();
        let mut value = String::new();
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                break;
            }
            value.push(ch);
            self.advance();
        }
        Token {
            kind: TokenKind::Comment(value.trim_start().to_string()),
            pos,
        }
    }

    fn single(&mut self, kind: TokenKind) -> Token {
        let pos = self.pos();
        self.advance();
        Token { kind, pos }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.index).copied()
    }

    fn advance(&mut self) {
        if self.peek() == Some('\n') {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        self.index += 1;
    }

    fn pos(&self) -> SourcePos {
        SourcePos::new(self.filename, self.line, self.column)
    }
}
