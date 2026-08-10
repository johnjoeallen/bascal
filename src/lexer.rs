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
    fn lexes_eol_comments_from_double_slash() {
        let tokens = Lexer::new("test.bcl", "x% = 1 // set x\ny% = 2 ' set y\n").lex();
        let comments: Vec<_> = tokens
            .iter()
            .filter_map(|t| match &t.kind {
                TokenKind::Comment(c) => Some(c.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(comments, vec!["set x", "set y"]);
    }

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

    #[test]
    fn lexes_bracket_brace_and_standalone_dot() {
        // Regression test: a `.` that does not immediately follow an
        // identifier-start character used to be silently dropped by the
        // lexer's catch-all case instead of producing a token.
        let tokens = Lexer::new("test.bcl", "db[i].field {a: 1}").lex();
        let kinds: Vec<_> = tokens
            .into_iter()
            .filter(|t| !matches!(t.kind, TokenKind::Eof))
            .map(|t| t.kind)
            .collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Ident("db".to_string()),
                TokenKind::LBracket,
                TokenKind::Ident("i".to_string()),
                TokenKind::RBracket,
                TokenKind::Dot,
                TokenKind::Ident("field".to_string()),
                TokenKind::LBrace,
                TokenKind::Ident("a".to_string()),
                TokenKind::Colon,
                TokenKind::Number(1),
                TokenKind::RBrace,
            ]
        );
    }

    #[test]
    fn dotted_identifier_still_lexes_as_one_token() {
        // `s.id` and dotted require/import paths must keep lexing as a
        // single Ident token — only a *standalone* `.` gets its own token.
        let tokens = Lexer::new("test.bcl", "s.id com.bascal.sort.bubbleSort").lex();
        let idents: Vec<_> = tokens
            .into_iter()
            .filter_map(|t| match t.kind {
                TokenKind::Ident(v) => Some(v),
                _ => None,
            })
            .collect();
        assert_eq!(idents, vec!["s.id", "com.bascal.sort.bubbleSort"]);
    }

    #[test]
    fn lexes_double_amp_and_pipe_as_short_circuit_operators() {
        let tokens = Lexer::new("test.bcl", "a && b\nc || d\nx&&y").lex();
        let kinds: Vec<_> = tokens
            .into_iter()
            .filter(|t| !matches!(t.kind, TokenKind::Eof | TokenKind::Newline))
            .map(|t| t.kind)
            .collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Ident("a".to_string()),
                TokenKind::AndAnd,
                TokenKind::Ident("b".to_string()),
                TokenKind::Ident("c".to_string()),
                TokenKind::OrOr,
                TokenKind::Ident("d".to_string()),
                TokenKind::Ident("x".to_string()),
                TokenKind::AndAnd,
                TokenKind::Ident("y".to_string()),
            ]
        );
    }

    #[test]
    fn long_suffix_and_hex_octal_literals_unaffected_by_double_amp() {
        // Regression test: adding `&&` must not break the existing single-`&`
        // uses — the `Long` type suffix on an identifier, and `&H`/`&O`
        // hex/octal literal prefixes.
        let tokens = Lexer::new("test.bcl", "count& &H1A &O17").lex();
        let kinds: Vec<_> = tokens
            .into_iter()
            .filter(|t| !matches!(t.kind, TokenKind::Eof))
            .map(|t| t.kind)
            .collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Ident("count&".to_string()),
                TokenKind::HexLit("&H1A".to_string()),
                TokenKind::HexLit("&O17".to_string()),
            ]
        );
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Ident(String),
    Number(i64),
    String(String),
    Comment(String),
    BlockComment(String),
    Float(f64),
    Newline,
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Comma,
    Colon,
    Semicolon,
    Hash,
    Dot,
    Question,
    HexLit(String),
    Plus,
    Minus,
    Star,
    Slash,
    Backslash,
    Caret,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    AndAnd,
    OrOr,
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
                '&' => {
                    if self.peek_at(1) == Some('&') {
                        let pos = self.pos();
                        self.advance();
                        self.advance();
                        tokens.push(Token { kind: TokenKind::AndAnd, pos });
                    } else {
                        tokens.push(self.hex_or_octal_lit());
                    }
                }
                '|' if self.peek_at(1) == Some('|') => {
                    let pos = self.pos();
                    self.advance();
                    self.advance();
                    tokens.push(Token { kind: TokenKind::OrOr, pos });
                }
                'A'..='Z' | 'a'..='z' | '_' => tokens.push(self.ident()),
                '(' => tokens.push(self.single(TokenKind::LParen)),
                ')' => tokens.push(self.single(TokenKind::RParen)),
                '[' => tokens.push(self.single(TokenKind::LBracket)),
                ']' => tokens.push(self.single(TokenKind::RBracket)),
                '{' => tokens.push(self.single(TokenKind::LBrace)),
                '}' => tokens.push(self.single(TokenKind::RBrace)),
                ',' => tokens.push(self.single(TokenKind::Comma)),
                ';' => tokens.push(self.single(TokenKind::Semicolon)),
                ':' => tokens.push(self.single(TokenKind::Colon)),
                '#' => tokens.push(self.single(TokenKind::Hash)),
                '.' => tokens.push(self.single(TokenKind::Dot)),
                '?' => tokens.push(self.single(TokenKind::Question)),
                '+' => tokens.push(self.single(TokenKind::Plus)),
                '-' => tokens.push(self.single(TokenKind::Minus)),
                '*' => tokens.push(self.single(TokenKind::Star)),
                '^' => tokens.push(self.single(TokenKind::Caret)),
                '\\' => tokens.push(self.single(TokenKind::Backslash)),
                '/' => {
                    if self.peek_at(1) == Some('*') {
                        tokens.push(self.block_comment());
                    } else if self.peek_at(1) == Some('/') {
                        tokens.push(self.eol_comment());
                    } else {
                        tokens.push(self.single(TokenKind::Slash));
                    }
                }
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
        if matches!(self.peek(), Some('%' | '$' | '!' | '#'))
            || (self.peek() == Some('&') && self.peek_at(1) != Some('&'))
        {
            value.push(self.peek().unwrap());
            self.advance();
        }
        Token {
            kind: TokenKind::Ident(value),
            pos,
        }
    }

    fn hex_or_octal_lit(&mut self) -> Token {
        let pos = self.pos();
        self.advance(); // consume '&'
        let prefix = self.peek().map(|c| c.to_ascii_uppercase()).unwrap_or(' ');
        if prefix == 'H' || prefix == 'O' {
            self.advance(); // consume 'H' or 'O'
            let mut digits = String::new();
            while let Some(ch) = self.peek() {
                if ch.is_ascii_hexdigit() {
                    digits.push(ch.to_ascii_uppercase());
                    self.advance();
                } else {
                    break;
                }
            }
            let lit = format!("&{prefix}{digits}");
            Token { kind: TokenKind::HexLit(lit), pos }
        } else {
            // bare & — emit as integer 0 (shouldn't occur in valid BASCAL)
            Token { kind: TokenKind::Number(0), pos }
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
        if self.peek() == Some('.') && self.peek_at(1).is_some_and(|c| c.is_ascii_digit()) {
            value.push('.');
            self.advance();
            while let Some(ch) = self.peek() {
                if ch.is_ascii_digit() {
                    value.push(ch);
                    self.advance();
                } else {
                    break;
                }
            }
            Token {
                kind: TokenKind::Float(value.parse().unwrap_or(0.0)),
                pos,
            }
        } else {
            Token {
                kind: TokenKind::Number(value.parse().unwrap_or(0)),
                pos,
            }
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

    fn eol_comment(&mut self) -> Token {
        let pos = self.pos();
        self.advance(); // first '/'
        self.advance(); // second '/'
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

    fn block_comment(&mut self) -> Token {
        let pos = self.pos();
        self.advance(); // '/'
        self.advance(); // '*'
        let mut value = String::new();
        loop {
            match self.peek() {
                None => break,
                Some('*') if self.peek_at(1) == Some('/') => {
                    self.advance(); // '*'
                    self.advance(); // '/'
                    break;
                }
                Some(ch) => {
                    value.push(ch);
                    self.advance();
                }
            }
        }
        Token {
            kind: TokenKind::BlockComment(value),
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
        self.peek_at(0)
    }

    fn peek_at(&self, offset: usize) -> Option<char> {
        self.chars.get(self.index + offset).copied()
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
