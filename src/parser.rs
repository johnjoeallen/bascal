use crate::ast::*;
use crate::diagnostics::{Diagnostic, SourcePos};
use crate::lexer::{Token, TokenKind};

type ParseResult<T> = Result<T, Diagnostic>;

pub struct Parser {
    filename: String,
    tokens: Vec<Token>,
    pos: usize,
    pending_blank: bool,
}

impl Parser {
    pub fn new(filename: String, tokens: Vec<Token>) -> Self {
        Self {
            filename,
            tokens,
            pos: 0,
            pending_blank: false,
        }
    }

    pub fn parse_program(&mut self) -> Result<Program, Vec<Diagnostic>> {
        match self.parse_program_inner() {
            Ok(program) => Ok(program),
            Err(diagnostic) => Err(vec![diagnostic]),
        }
    }

    fn parse_program_inner(&mut self) -> ParseResult<Program> {
        let mut program_decl = None;
        let mut declarations = Vec::new();
        let mut common = Vec::new();
        let mut statements = Vec::new();
        let mut functions = Vec::new();

        self.skip_newlines();
        while !self.is_eof() {
            if self.check_keyword("program") {
                let decl = self.parse_program_decl()?;
                if program_decl.is_some() {
                    return Err(self.error("only one `program` declaration is allowed per file"));
                }
                program_decl = Some(decl);
            } else if self.check_keyword("common") {
                common.push(self.parse_common_block()?);
            } else if self.check_keyword("require") {
                declarations.push(self.parse_path_decl(false)?);
            } else if self.check_keyword("import") {
                declarations.push(self.parse_path_decl(true)?);
            } else if self.check_keyword("function") {
                functions.push(self.parse_function()?);
            } else if self.check_keyword("procedure") {
                functions.push(self.parse_procedure()?);
            } else {
                statements.push(self.parse_statement()?);
            }
            if self.take_pending_blank() && !self.is_eof() {
                statements.push(Statement::BlankLine);
            }
        }

        Ok(Program {
            program_decl,
            declarations,
            common,
            statements,
            functions,
        })
    }

    fn parse_program_decl(&mut self) -> ParseResult<ProgramDecl> {
        self.expect_keyword("program")?;
        let name = self.expect_ident("expected program name")?;
        let suite = if self.check_keyword("suite") {
            self.advance();
            Some(self.expect_ident("expected suite name after `suite`")?)
        } else {
            None
        };
        self.consume_line_end()?;
        Ok(ProgramDecl { name, suite })
    }

    fn parse_common_block(&mut self) -> ParseResult<CommonBlock> {
        self.expect_keyword("common")?;
        let mut vars = Vec::new();
        loop {
            let name = BasicIdent::parse(&self.expect_ident("expected variable name in COMMON")?);
            let is_array = if self.eat(TokenKind::LParen) {
                self.expect(TokenKind::RParen, "expected `)` after `(` in COMMON")?;
                true
            } else {
                false
            };
            vars.push(CommonVar { name, is_array });
            if !self.eat(TokenKind::Comma) {
                break;
            }
        }
        self.consume_line_end()?;
        Ok(CommonBlock { vars })
    }

    fn parse_path_decl(&mut self, import: bool) -> ParseResult<DependencyDecl> {
        if import {
            self.expect_keyword("import")?;
        } else {
            self.expect_keyword("require")?;
        }
        let raw = self.expect_ident("expected path-style dependency symbol")?;
        self.consume_line_end()?;
        let symbol = PathSymbol { raw };
        Ok(if import {
            DependencyDecl::Import(symbol)
        } else {
            DependencyDecl::Require(symbol)
        })
    }

    fn parse_function(&mut self) -> ParseResult<FunctionDef> {
        self.expect_keyword("function")?;
        let name = BasicIdent::parse(&self.expect_ident("expected function name")?);
        self.expect(TokenKind::LParen, "expected `(` after function name")?;
        let params = self.parse_ident_list()?;
        self.expect(TokenKind::RParen, "expected `)` after function parameters")?;

        if self.check_keyword("returns") {
            return Err(self.error("`returns` clauses are not supported in BASCAL"));
        }
        self.consume_line_end()?;

        let body = self.parse_block(&[BlockEnd::EndFunction])?;
        self.expect_keyword("end")?;
        self.expect_keyword("function")?;
        self.consume_line_end()?;
        Ok(FunctionDef { name, params, body, is_procedure: false })
    }

    fn parse_procedure(&mut self) -> ParseResult<FunctionDef> {
        self.expect_keyword("procedure")?;
        let raw = self.expect_ident("expected procedure name")?;
        let name = BasicIdent::parse(&raw);
        if name.suffix.is_some() {
            return Err(self.error("procedure names must not carry a type suffix"));
        }
        self.expect(TokenKind::LParen, "expected `(` after procedure name")?;
        let params = self.parse_ident_list()?;
        self.expect(TokenKind::RParen, "expected `)` after procedure parameters")?;
        self.consume_line_end()?;

        let body = self.parse_block(&[BlockEnd::EndProcedure])?;
        self.expect_keyword("end")?;
        self.expect_keyword("procedure")?;
        self.consume_line_end()?;
        Ok(FunctionDef { name, params, body, is_procedure: true })
    }

    fn parse_ident_list(&mut self) -> ParseResult<Vec<BasicIdent>> {
        let mut items = Vec::new();
        if matches!(self.current().kind, TokenKind::RParen) {
            return Ok(items);
        }
        loop {
            items.push(BasicIdent::parse(
                &self.expect_ident("expected identifier")?,
            ));
            if !self.eat(TokenKind::Comma) {
                break;
            }
        }
        Ok(items)
    }

    fn parse_block(&mut self, ends: &[BlockEnd]) -> ParseResult<Vec<Statement>> {
        let mut body = Vec::new();
        self.pending_blank = false;
        self.skip_newlines();
        while !self.is_eof() && !self.at_any_block_end(ends) {
            body.push(self.parse_statement()?);
            if self.take_pending_blank() && !self.at_any_block_end(ends) && !self.is_eof() {
                body.push(Statement::BlankLine);
            }
        }
        Ok(body)
    }

    fn parse_statement(&mut self) -> ParseResult<Statement> {
        if self.check_keyword("dim") {
            self.parse_dim()
        } else if matches!(self.current().kind, TokenKind::Comment(_)) {
            self.parse_comment()
        } else if matches!(self.current().kind, TokenKind::BlockComment(_)) {
            self.parse_block_comment()
        } else if self.check_keyword("print") {
            self.parse_print()
        } else if self.check_keyword("lprint") {
            self.parse_lprint()
        } else if self.check_keyword("open") {
            self.parse_open()
        } else if self.check_keyword("line") && self.check_next_keyword("input") {
            self.parse_line_input()
        } else if self.check_keyword("input") {
            self.parse_input()
        } else if self.check_keyword("write") {
            self.parse_write()
        } else if self.check_keyword("field") {
            self.parse_field()
        } else if self.check_keyword("get") {
            self.parse_get()
        } else if self.check_keyword("put") {
            self.parse_put()
        } else if self.check_keyword("lset") {
            self.parse_lset()
        } else if self.check_keyword("rset") {
            self.parse_rset()
        } else if self.check_keyword("seek") {
            self.parse_seek()
        } else if self.check_keyword("kill") {
            self.parse_kill()
        } else if self.check_keyword("name") {
            self.parse_name()
        } else if self.check_keyword("close") {
            self.parse_close()
        } else if self.check_keyword("global") {
            self.parse_global_decl()
        } else if self.check_keyword("return") {
            self.parse_return()
        } else if self.check_keyword("if") {
            self.parse_if()
        } else if self.check_keyword("for") {
            self.parse_for()
        } else if self.check_keyword("while") {
            self.parse_while()
        } else if self.check_keyword("do") {
            self.parse_do()
        } else if self.check_keyword("select") && self.check_next_keyword("case") {
            self.parse_select_case()
        } else if self.check_keyword("end") {
            self.parse_end_statement()
        } else if self.check_keyword("exit") {
            self.parse_exit()
        } else if self.check_keyword("goto") {
            self.parse_goto()
        } else if self.check_keyword("gosub") {
            self.parse_gosub()
        } else if self.check_keyword("on") {
            self.parse_on()
        } else if self.check_keyword("resume") {
            self.parse_resume()
        } else if self.check_keyword("error") {
            self.parse_error_stmt()
        } else if self.check_keyword("option") {
            self.parse_option_base()
        } else if self.check_keyword("erase") {
            self.parse_erase()
        } else if self.check_keyword("stop") {
            self.advance(); self.consume_line_end()?; Ok(Statement::Stop)
        } else if self.check_keyword("cls") {
            self.advance(); self.consume_line_end()?; Ok(Statement::Cls)
        } else if self.check_keyword("beep") {
            self.advance(); self.consume_line_end()?; Ok(Statement::Beep)
        } else if self.check_keyword("system") {
            self.advance(); self.consume_line_end()?; Ok(Statement::System)
        } else if self.check_keyword("randomize") {
            self.parse_randomize()
        } else if self.check_keyword("poke") {
            self.parse_poke()
        } else if self.check_keyword("swap") {
            self.parse_swap()
        } else if self.check_keyword("data") {
            self.parse_data()
        } else if self.check_keyword("read") {
            self.parse_read()
        } else if self.check_keyword("restore") {
            self.parse_restore()
        } else if self.check_keyword("const") {
            self.parse_const()
        } else if self.check_keyword("locate") {
            self.parse_locate()
        } else if self.check_keyword("color") {
            self.parse_color()
        } else if self.check_keyword("let") {
            self.parse_let()
        } else if self.check_keyword("common") {
            Err(self.error("`common` is only valid at program level in a suite file"))
        } else if self.check_keyword("program") {
            Err(self.error("`program` declaration must appear before any statements"))
        } else {
            self.parse_assignment_or_expr()
        }
    }

    fn parse_dim(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("dim")?;
        let name = BasicIdent::parse(&self.expect_ident("expected DIM variable name")?);
        let sizes = if self.eat(TokenKind::LParen) {
            if self.eat(TokenKind::RParen) {
                Vec::new() // dim arr%() — declare without bounds
            } else {
                let mut sizes = vec![self.parse_expr(0)?];
                while self.eat(TokenKind::Comma) {
                    sizes.push(self.parse_expr(0)?);
                }
                self.expect(TokenKind::RParen, "expected `)` after DIM dimensions")?;
                sizes
            }
        } else {
            Vec::new()
        };
        self.consume_line_end()?;
        Ok(Statement::Dim { name, sizes })
    }

    fn parse_block_comment(&mut self) -> ParseResult<Statement> {
        let text = match &self.current().kind {
            TokenKind::BlockComment(text) => text.clone(),
            _ => return Err(self.error("expected block comment")),
        };
        self.advance();
        self.consume_line_end()?;
        let lines = text
            .lines()
            .map(|line| {
                let trimmed = line.trim();
                trimmed.strip_prefix('*').map(|s| s.trim()).unwrap_or(trimmed).to_string()
            })
            .collect::<Vec<_>>();
        let start = lines.iter().position(|l| !l.is_empty()).unwrap_or(0);
        let end = lines.iter().rposition(|l| !l.is_empty()).map(|i| i + 1).unwrap_or(start);
        let lines = lines[start..end].to_vec();
        Ok(Statement::BlockComment(lines))
    }

    fn parse_global_decl(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("global")?;
        let name = BasicIdent::parse(&self.expect_ident("expected variable name after `global`")?);
        self.consume_line_end()?;
        Ok(Statement::GlobalDecl(name))
    }

    fn parse_comment(&mut self) -> ParseResult<Statement> {
        let comment = match &self.current().kind {
            TokenKind::Comment(comment) => comment.clone(),
            _ => return Err(self.error("expected comment")),
        };
        self.advance();
        self.consume_line_end()?;
        Ok(Statement::Raw(format!("' {comment}")))
    }

    fn parse_print_tokens(&mut self) -> ParseResult<Vec<PrintToken>> {
        let mut tokens = Vec::new();
        loop {
            if self.at_line_end() {
                break;
            }
            if self.eat(TokenKind::Semicolon) {
                tokens.push(PrintToken::Semi);
            } else if self.eat(TokenKind::Comma) {
                tokens.push(PrintToken::Comma);
            } else {
                tokens.push(PrintToken::Expr(self.parse_expr(0)?));
            }
        }
        Ok(tokens)
    }

    fn parse_print(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("print")?;
        if self.eat(TokenKind::Hash) {
            let channel = self.parse_expr(0)?;
            // structural comma separating channel from content
            self.expect(TokenKind::Comma, "expected `,` after file number")?;
            if self.check_keyword("using") {
                self.expect_keyword("using")?;
                let format = self.parse_expr(0)?;
                self.expect(TokenKind::Semicolon, "expected `;` after USING format string")?;
                let tokens = self.parse_print_tokens()?;
                self.consume_line_end()?;
                return Ok(Statement::PrintFileUsing { channel, format, tokens });
            }
            let tokens = self.parse_print_tokens()?;
            self.consume_line_end()?;
            return Ok(Statement::PrintFile { channel, tokens });
        }
        if self.check_keyword("using") {
            self.expect_keyword("using")?;
            let format = self.parse_expr(0)?;
            self.expect(TokenKind::Semicolon, "expected `;` after USING format string")?;
            let tokens = self.parse_print_tokens()?;
            self.consume_line_end()?;
            return Ok(Statement::PrintUsing { format, tokens });
        }
        let tokens = self.parse_print_tokens()?;
        self.consume_line_end()?;
        Ok(Statement::Print { tokens })
    }

    fn parse_open(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("open")?;
        let file = self.parse_expr(0)?;
        self.expect_keyword("for")?;
        let mode = if self.check_keyword("input") {
            self.expect_keyword("input")?;
            OpenMode::Input
        } else if self.check_keyword("output") {
            self.expect_keyword("output")?;
            OpenMode::Output
        } else if self.check_keyword("append") {
            self.expect_keyword("append")?;
            OpenMode::Append
        } else if self.check_keyword("random") {
            self.expect_keyword("random")?;
            OpenMode::Random
        } else if self.check_keyword("binary") {
            self.expect_keyword("binary")?;
            OpenMode::Binary
        } else {
            return Err(self.error("expected `input`, `output`, `append`, `random`, or `binary`"));
        };
        self.expect_keyword("as")?;
        self.expect(TokenKind::Hash, "expected `#` before file number")?;
        let channel = self.parse_expr(0)?;
        let len = if self.check_keyword("len") {
            self.expect_keyword("len")?;
            self.expect(TokenKind::Eq, "expected `=` after `len`")?;
            Some(self.parse_expr(0)?)
        } else {
            None
        };
        self.consume_line_end()?;
        Ok(Statement::Open { mode, file, channel, len })
    }

    fn parse_field(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("field")?;
        self.expect(TokenKind::Hash, "expected `#` before file number")?;
        let channel = self.parse_expr(0)?;
        self.expect(TokenKind::Comma, "expected `,` after file number")?;
        let mut fields = Vec::new();
        loop {
            let width = self.parse_expr(0)?;
            self.expect_keyword("as")?;
            let var = BasicIdent::parse(&self.expect_ident("expected variable name after `as`")?);
            fields.push((width, var));
            if !self.eat(TokenKind::Comma) {
                break;
            }
        }
        self.consume_line_end()?;
        Ok(Statement::Field { channel, fields })
    }

    fn parse_get(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("get")?;
        self.expect(TokenKind::Hash, "expected `#` before file number")?;
        let channel = self.parse_expr(0)?;
        let (record, var) = if self.eat(TokenKind::Comma) {
            let record = if self.current().kind == TokenKind::Comma || self.at_line_end() {
                None
            } else {
                Some(self.parse_expr(0)?)
            };
            let var = if self.eat(TokenKind::Comma) {
                Some(self.parse_expr(0)?)
            } else {
                None
            };
            (record, var)
        } else {
            (None, None)
        };
        self.consume_line_end()?;
        Ok(Statement::Get { channel, record, var })
    }

    fn parse_put(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("put")?;
        self.expect(TokenKind::Hash, "expected `#` before file number")?;
        let channel = self.parse_expr(0)?;
        let (record, var) = if self.eat(TokenKind::Comma) {
            let record = if self.current().kind == TokenKind::Comma || self.at_line_end() {
                None
            } else {
                Some(self.parse_expr(0)?)
            };
            let var = if self.eat(TokenKind::Comma) {
                Some(self.parse_expr(0)?)
            } else {
                None
            };
            (record, var)
        } else {
            (None, None)
        };
        self.consume_line_end()?;
        Ok(Statement::Put { channel, record, var })
    }

    fn parse_lset(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("lset")?;
        let var = BasicIdent::parse(&self.expect_ident("expected variable name after `lset`")?);
        self.expect(TokenKind::Eq, "expected `=`")?;
        let value = self.parse_expr(0)?;
        self.consume_line_end()?;
        Ok(Statement::Lset { var, value })
    }

    fn parse_rset(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("rset")?;
        let var = BasicIdent::parse(&self.expect_ident("expected variable name after `rset`")?);
        self.expect(TokenKind::Eq, "expected `=`")?;
        let value = self.parse_expr(0)?;
        self.consume_line_end()?;
        Ok(Statement::Rset { var, value })
    }

    fn parse_seek(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("seek")?;
        self.expect(TokenKind::Hash, "expected `#` before file number")?;
        let channel = self.parse_expr(0)?;
        self.expect(TokenKind::Comma, "expected `,` after file number")?;
        let position = self.parse_expr(0)?;
        self.consume_line_end()?;
        Ok(Statement::Seek { channel, position })
    }

    fn parse_line_input(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("line")?;
        self.expect_keyword("input")?;
        self.expect(TokenKind::Hash, "expected `#` before file number")?;
        let channel = self.parse_expr(0)?;
        self.expect(TokenKind::Comma, "expected `,` after file number")?;
        let target = self.parse_expr(0)?;
        self.consume_line_end()?;
        Ok(Statement::LineInput { channel, target })
    }

    fn parse_kill(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("kill")?;
        let file = self.parse_expr(0)?;
        self.consume_line_end()?;
        Ok(Statement::Kill { file })
    }

    fn parse_name(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("name")?;
        let from = self.parse_expr(0)?;
        self.expect_keyword("as")?;
        let to = self.parse_expr(0)?;
        self.consume_line_end()?;
        Ok(Statement::Name { from, to })
    }

    fn parse_close(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("close")?;
        self.expect(TokenKind::Hash, "expected `#` before file number")?;
        let channel = self.parse_expr(0)?;
        self.consume_line_end()?;
        Ok(Statement::Close { channel })
    }

    fn parse_return(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("return")?;
        if self.at_line_end() {
            self.consume_line_end()?;
            return Ok(Statement::ReturnVoid);
        }
        let value = self.parse_expr(0)?;
        self.consume_line_end()?;
        Ok(Statement::Return { value })
    }

    fn parse_if(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("if")?;
        let condition = self.parse_expr(0)?;
        self.expect_keyword("then")?;
        self.consume_line_end()?;
        let then_body = self.parse_block(&[BlockEnd::Else, BlockEnd::ElseIf, BlockEnd::EndIf])?;
        let else_body = if self.check_keyword("elseif") {
            vec![self.parse_elseif()?]
        } else if self.check_keyword("else") {
            self.expect_keyword("else")?;
            self.consume_line_end()?;
            let body = self.parse_block(&[BlockEnd::EndIf])?;
            self.expect_keyword("end")?;
            self.expect_keyword("if")?;
            self.consume_line_end()?;
            body
        } else {
            self.expect_keyword("end")?;
            self.expect_keyword("if")?;
            self.consume_line_end()?;
            Vec::new()
        };
        Ok(Statement::If { condition, then_body, else_body })
    }

    fn parse_elseif(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("elseif")?;
        let condition = self.parse_expr(0)?;
        self.expect_keyword("then")?;
        self.consume_line_end()?;
        let then_body = self.parse_block(&[BlockEnd::Else, BlockEnd::ElseIf, BlockEnd::EndIf])?;
        let else_body = if self.check_keyword("elseif") {
            vec![self.parse_elseif()?]
        } else if self.check_keyword("else") {
            self.expect_keyword("else")?;
            self.consume_line_end()?;
            let body = self.parse_block(&[BlockEnd::EndIf])?;
            self.expect_keyword("end")?;
            self.expect_keyword("if")?;
            self.consume_line_end()?;
            body
        } else {
            self.expect_keyword("end")?;
            self.expect_keyword("if")?;
            self.consume_line_end()?;
            Vec::new()
        };
        Ok(Statement::If { condition, then_body, else_body })
    }

    fn parse_for(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("for")?;
        let var = BasicIdent::parse(&self.expect_ident("expected FOR variable")?);
        self.expect(TokenKind::Eq, "expected `=` in FOR statement")?;
        let start = self.parse_expr(0)?;
        self.expect_keyword("to")?;
        let end = self.parse_expr(0)?;
        let step = if self.check_keyword("step") {
            self.expect_keyword("step")?;
            Some(self.parse_expr(0)?)
        } else {
            None
        };
        self.consume_line_end()?;
        let body = self.parse_block(&[BlockEnd::ForEnd, BlockEnd::BareEnd])?;
        self.expect_keyword("end")?;
        if self.check_keyword("for") {
            self.expect_keyword("for")?;
        }
        self.consume_line_end()?;
        Ok(Statement::For {
            var,
            start,
            end,
            step,
            body,
        })
    }

    fn parse_while(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("while")?;
        let condition = self.parse_expr(0)?;
        self.consume_line_end()?;
        let body = self.parse_block(&[BlockEnd::WhileEnd, BlockEnd::BareEnd])?;
        self.expect_keyword("end")?;
        if self.check_keyword("while") {
            self.expect_keyword("while")?;
        }
        self.consume_line_end()?;
        Ok(Statement::While { condition, body })
    }

    fn parse_end_statement(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("end")?;
        if self.check_keyword("if") || self.check_keyword("function") || self.check_keyword("select")
            || self.check_keyword("procedure") || self.check_keyword("while")
            || self.check_keyword("for") || self.check_keyword("do")
        {
            return Err(self.error("unexpected block terminator"));
        }
        self.consume_line_end()?;
        Ok(Statement::End)
    }

    fn parse_do(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("do")?;
        let condition = if self.check_keyword("while") || self.check_keyword("until") {
            Some(self.parse_do_condition()?)
        } else {
            None
        };
        self.consume_line_end()?;
        let body = self.parse_block(&[BlockEnd::DoEnd, BlockEnd::BareEnd])?;
        self.expect_keyword("end")?;
        if self.check_keyword("do") {
            self.expect_keyword("do")?;
        }
        self.consume_line_end()?;
        Ok(Statement::Do { condition, body, post_condition: None })
    }

    fn parse_do_condition(&mut self) -> ParseResult<DoCondition> {
        let is_while = if self.check_keyword("while") {
            self.advance();
            true
        } else {
            self.expect_keyword("until")?;
            false
        };
        let expr = self.parse_expr(0)?;
        Ok(DoCondition { is_while, expr })
    }

    fn parse_select_case(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("select")?;
        self.expect_keyword("case")?;
        let expr = self.parse_expr(0)?;
        self.consume_line_end()?;
        self.skip_newlines();
        let mut cases = Vec::new();
        let mut else_body = Vec::new();
        while !self.is_eof() && !(self.check_keyword("end") && self.check_next_keyword("select")) {
            self.expect_keyword("case")?;
            if self.check_keyword("else") {
                self.advance();
                self.consume_line_end()?;
                else_body = self.parse_block(&[BlockEnd::EndSelect])?;
                break;
            }
            let values = self.parse_case_values()?;
            self.consume_line_end()?;
            let body = self.parse_block(&[BlockEnd::Case, BlockEnd::EndSelect])?;
            cases.push(CaseClause { values, body });
        }
        self.expect_keyword("end")?;
        self.expect_keyword("select")?;
        self.consume_line_end()?;
        Ok(Statement::SelectCase { expr, cases, else_body })
    }

    fn parse_case_values(&mut self) -> ParseResult<Vec<CaseValue>> {
        let mut values = Vec::new();
        loop {
            let value = if self.check_keyword("is") {
                self.advance();
                let op = self.parse_comparison_op()?;
                let expr = self.parse_expr(0)?;
                CaseValue::Is { op, value: expr }
            } else {
                let from = self.parse_expr(0)?;
                if self.check_keyword("to") {
                    self.advance();
                    let to = self.parse_expr(0)?;
                    CaseValue::Range { from, to }
                } else {
                    CaseValue::Single(from)
                }
            };
            values.push(value);
            if !self.eat(TokenKind::Comma) {
                break;
            }
        }
        Ok(values)
    }

    fn parse_comparison_op(&mut self) -> ParseResult<BinaryOp> {
        let op = match &self.current().kind {
            TokenKind::Eq => BinaryOp::Eq,
            TokenKind::Ne => BinaryOp::Ne,
            TokenKind::Lt => BinaryOp::Lt,
            TokenKind::Le => BinaryOp::Le,
            TokenKind::Gt => BinaryOp::Gt,
            TokenKind::Ge => BinaryOp::Ge,
            _ => return Err(self.error("expected comparison operator after IS")),
        };
        self.advance();
        Ok(op)
    }

    fn parse_exit(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("exit")?;
        if self.check_keyword("for") {
            self.advance(); self.consume_line_end()?; Ok(Statement::ExitFor)
        } else if self.check_keyword("while") {
            self.advance(); self.consume_line_end()?; Ok(Statement::ExitWhile)
        } else if self.check_keyword("do") {
            self.advance(); self.consume_line_end()?; Ok(Statement::ExitDo)
        } else {
            Err(self.error("expected `for`, `while`, or `do` after `exit`"))
        }
    }

    fn parse_goto(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("goto")?;
        let target = self.parse_expr(0)?;
        self.consume_line_end()?;
        Ok(Statement::Goto(target))
    }

    fn parse_gosub(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("gosub")?;
        let target = self.parse_expr(0)?;
        self.consume_line_end()?;
        Ok(Statement::Gosub(target))
    }

    fn parse_on(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("on")?;
        if self.check_keyword("error") {
            self.expect_keyword("error")?;
            self.expect_keyword("goto")?;
            let target = self.parse_expr(0)?;
            self.consume_line_end()?;
            return Ok(Statement::OnErrorGoto { target });
        }
        let expr = self.parse_expr(0)?;
        let is_gosub = if self.check_keyword("goto") {
            self.advance(); false
        } else {
            self.expect_keyword("gosub")?; true
        };
        let mut targets = Vec::new();
        loop {
            targets.push(self.parse_expr(0)?);
            if !self.eat(TokenKind::Comma) { break; }
        }
        self.consume_line_end()?;
        Ok(Statement::OnBranch { expr, targets, is_gosub })
    }

    fn parse_resume(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("resume")?;
        let target = if self.at_line_end() {
            ResumeTarget::Same
        } else if self.check_keyword("next") {
            self.advance();
            ResumeTarget::Next
        } else {
            ResumeTarget::Line(self.parse_expr(0)?)
        };
        self.consume_line_end()?;
        Ok(Statement::Resume(target))
    }

    fn parse_error_stmt(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("error")?;
        let code = self.parse_expr(0)?;
        self.consume_line_end()?;
        Ok(Statement::ErrorStmt { code })
    }

    fn parse_lprint(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("lprint")?;
        if self.check_keyword("using") {
            self.expect_keyword("using")?;
            let format = self.parse_expr(0)?;
            self.expect(TokenKind::Semicolon, "expected `;` after USING format string")?;
            let tokens = self.parse_print_tokens()?;
            self.consume_line_end()?;
            return Ok(Statement::LprintUsing { format, tokens });
        }
        let tokens = self.parse_print_tokens()?;
        self.consume_line_end()?;
        Ok(Statement::Lprint(tokens))
    }

    fn parse_write(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("write")?;
        self.expect(TokenKind::Hash, "expected `#` after WRITE")?;
        let channel = self.parse_expr(0)?;
        self.expect(TokenKind::Comma, "expected `,` after file number")?;
        let mut exprs = Vec::new();
        if !self.at_line_end() {
            loop {
                exprs.push(self.parse_expr(0)?);
                if !self.eat(TokenKind::Comma) { break; }
            }
        }
        self.consume_line_end()?;
        Ok(Statement::Write { channel, exprs })
    }

    fn parse_input(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("input")?;
        if self.eat(TokenKind::Hash) {
            let channel = self.parse_expr(0)?;
            self.expect(TokenKind::Comma, "expected `,` after file number")?;
            let mut vars = Vec::new();
            loop {
                vars.push(self.parse_expr(0)?);
                if !self.eat(TokenKind::Comma) { break; }
            }
            self.consume_line_end()?;
            return Ok(Statement::InputFile { channel, vars });
        }
        let prompt = if matches!(self.current().kind, TokenKind::String(_)) {
            let text = match &self.current().kind {
                TokenKind::String(s) => s.clone(),
                _ => unreachable!(),
            };
            self.advance();
            // accept either ; or , after the prompt string
            if !self.eat(TokenKind::Semicolon) {
                self.eat(TokenKind::Comma);
            }
            Some(text)
        } else {
            None
        };
        let mut vars = Vec::new();
        loop {
            vars.push(self.parse_expr(0)?);
            if !self.eat(TokenKind::Comma) { break; }
        }
        self.consume_line_end()?;
        Ok(Statement::Input { prompt, vars })
    }

    fn parse_option_base(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("option")?;
        self.expect_keyword("base")?;
        let base = self.parse_expr(0)?;
        self.consume_line_end()?;
        Ok(Statement::OptionBase(base))
    }

    fn parse_erase(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("erase")?;
        let mut vars = Vec::new();
        loop {
            let name = self.expect_ident("expected array name in ERASE")?;
            vars.push(BasicIdent::parse(&name));
            if !self.eat(TokenKind::Comma) {
                break;
            }
        }
        self.consume_line_end()?;
        Ok(Statement::Erase(vars))
    }

    fn parse_randomize(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("randomize")?;
        let expr = if !self.at_line_end() {
            Some(self.parse_expr(0)?)
        } else {
            None
        };
        self.consume_line_end()?;
        Ok(Statement::Randomize(expr))
    }

    fn parse_poke(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("poke")?;
        let address = self.parse_expr(0)?;
        self.expect(TokenKind::Comma, "expected `,` in POKE")?;
        let value = self.parse_expr(0)?;
        self.consume_line_end()?;
        Ok(Statement::Poke { address, value })
    }

    fn parse_swap(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("swap")?;
        let a = self.parse_expr(0)?;
        self.expect(TokenKind::Comma, "expected `,` in SWAP")?;
        let b = self.parse_expr(0)?;
        self.consume_line_end()?;
        Ok(Statement::Swap(a, b))
    }

    fn parse_data(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("data")?;
        let mut values = Vec::new();
        loop {
            values.push(self.parse_expr(0)?);
            if !self.eat(TokenKind::Comma) { break; }
        }
        self.consume_line_end()?;
        Ok(Statement::Data(values))
    }

    fn parse_read(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("read")?;
        let mut vars = Vec::new();
        loop {
            vars.push(self.parse_expr(0)?);
            if !self.eat(TokenKind::Comma) { break; }
        }
        self.consume_line_end()?;
        Ok(Statement::Read(vars))
    }

    fn parse_restore(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("restore")?;
        let target = if !self.at_line_end() {
            Some(self.parse_expr(0)?)
        } else {
            None
        };
        self.consume_line_end()?;
        Ok(Statement::Restore(target))
    }

    fn parse_const(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("const")?;
        let name = BasicIdent::parse(&self.expect_ident("expected CONST name")?);
        self.expect(TokenKind::Eq, "expected `=` in CONST")?;
        let value = self.parse_expr(0)?;
        self.consume_line_end()?;
        Ok(Statement::Const { name, value })
    }

    fn parse_locate(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("locate")?;
        let row = self.parse_expr(0)?;
        self.expect(TokenKind::Comma, "expected `,` in LOCATE")?;
        let col = self.parse_expr(0)?;
        self.consume_line_end()?;
        Ok(Statement::Locate { row, col })
    }

    fn parse_color(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("color")?;
        let fg = self.parse_expr(0)?;
        let bg = if self.eat(TokenKind::Comma) {
            Some(self.parse_expr(0)?)
        } else {
            None
        };
        self.consume_line_end()?;
        Ok(Statement::Color { fg, bg })
    }

    fn parse_let(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("let")?;
        self.parse_assignment_or_expr()
    }

    fn parse_assignment_or_expr(&mut self) -> ParseResult<Statement> {
        let expr = self.parse_expr(8)?;
        if self.eat(TokenKind::Eq) {
            let value = self.parse_expr(0)?;
            self.consume_line_end()?;
            Ok(Statement::Assignment {
                target: normalize_assignment_target(expr),
                value,
            })
        } else {
            self.consume_line_end()?;
            Ok(Statement::ExprStmt(expr))
        }
    }

    fn parse_expr(&mut self, min_bp: u8) -> ParseResult<Expr> {
        let mut left = match &self.current().kind {
            TokenKind::Number(value) => {
                let value = *value;
                self.advance();
                Expr::Integer(value)
            }
            TokenKind::HexLit(s) => {
                let s = s.clone();
                self.advance();
                Expr::HexLit(s)
            }
            TokenKind::Float(value) => {
                let value = *value;
                self.advance();
                Expr::Float(value)
            }
            TokenKind::String(value) => {
                let value = value.clone();
                self.advance();
                Expr::String(value)
            }
            TokenKind::Ident(value) if keyword_eq(value, "not") => {
                self.advance();
                let expr = self.parse_expr(6)?;
                Expr::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(expr),
                }
            }
            TokenKind::Minus => {
                self.advance();
                let expr = self.parse_expr(17)?;
                Expr::Unary {
                    op: UnaryOp::Neg,
                    expr: Box::new(expr),
                }
            }
            TokenKind::Ident(value) => {
                let ident = BasicIdent::parse(value);
                self.advance();
                if self.eat(TokenKind::LParen) {
                    let args = self.parse_expr_list_until_rparen()?;
                    make_paren_ident_expr(ident, args)
                } else {
                    Expr::Ident(ident)
                }
            }
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expr(0)?;
                self.expect(TokenKind::RParen, "expected `)`")?;
                expr
            }
            _ => return Err(self.error("expected expression")),
        };

        loop {
            let Some((left_bp, right_bp, op)) = self.infix_binding_power() else {
                break;
            };
            if left_bp < min_bp {
                break;
            }
            self.advance();
            let right = self.parse_expr(right_bp)?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_expr_list_until_rparen(&mut self) -> ParseResult<Vec<Expr>> {
        let mut args = Vec::new();
        if self.eat(TokenKind::RParen) {
            return Ok(args);
        }
        loop {
            args.push(self.parse_expr(0)?);
            if !self.eat(TokenKind::Comma) {
                break;
            }
        }
        self.expect(TokenKind::RParen, "expected `)` after argument list")?;
        Ok(args)
    }

    fn infix_binding_power(&self) -> Option<(u8, u8, BinaryOp)> {
        match &self.current().kind {
            // Logical — lowest precedence (MS-BASIC order: XOR < OR < AND < NOT)
            TokenKind::Ident(value) if keyword_eq(value, "xor") => Some((1, 2, BinaryOp::Xor)),
            TokenKind::Ident(value) if keyword_eq(value, "or") => Some((3, 4, BinaryOp::Or)),
            TokenKind::Ident(value) if keyword_eq(value, "and") => Some((5, 6, BinaryOp::And)),
            // Comparison
            TokenKind::Eq => Some((7, 8, BinaryOp::Eq)),
            TokenKind::Ne => Some((7, 8, BinaryOp::Ne)),
            TokenKind::Lt => Some((7, 8, BinaryOp::Lt)),
            TokenKind::Le => Some((7, 8, BinaryOp::Le)),
            TokenKind::Gt => Some((7, 8, BinaryOp::Gt)),
            TokenKind::Ge => Some((7, 8, BinaryOp::Ge)),
            // Additive
            TokenKind::Plus => Some((9, 10, BinaryOp::Add)),
            TokenKind::Minus => Some((9, 10, BinaryOp::Sub)),
            // Integer MOD and \ (between additive and multiplicative)
            TokenKind::Ident(value) if keyword_eq(value, "mod") => Some((11, 12, BinaryOp::Mod)),
            TokenKind::Backslash => Some((13, 14, BinaryOp::IntDiv)),
            // Multiplicative
            TokenKind::Star => Some((15, 16, BinaryOp::Mul)),
            TokenKind::Slash => Some((15, 16, BinaryOp::Div)),
            // Exponentiation — right-associative, highest arithmetic precedence
            TokenKind::Caret => Some((18, 17, BinaryOp::Pow)),
            _ => None,
        }
    }

    fn consume_line_end(&mut self) -> ParseResult<()> {
        // Discard any trailing inline comment(s) before the actual line ending.
        while matches!(
            self.current().kind,
            TokenKind::Comment(_) | TokenKind::BlockComment(_)
        ) {
            self.advance();
        }
        if self.is_eof() {
            return Ok(());
        }
        if self.eat(TokenKind::Colon) || self.eat(TokenKind::Newline) {
            let extra = self.count_and_skip_newlines();
            if extra >= 1 {
                self.pending_blank = true;
            }
            return Ok(());
        }
        Err(self.error("expected end of line"))
    }

    fn skip_newlines(&mut self) {
        self.count_and_skip_newlines();
    }

    fn count_and_skip_newlines(&mut self) -> usize {
        let mut count = 0;
        while self.eat(TokenKind::Newline) {
            count += 1;
        }
        count
    }

    fn take_pending_blank(&mut self) -> bool {
        let val = self.pending_blank;
        self.pending_blank = false;
        val
    }

    fn at_line_end(&self) -> bool {
        matches!(
            self.current().kind,
            TokenKind::Newline
                | TokenKind::Colon
                | TokenKind::Eof
                | TokenKind::Comment(_)
                | TokenKind::BlockComment(_)
        )
    }

    fn at_any_block_end(&self, ends: &[BlockEnd]) -> bool {
        ends.iter().any(|end| self.at_block_end(*end))
    }

    fn at_block_end(&self, end: BlockEnd) -> bool {
        match end {
            BlockEnd::Else => self.check_keyword("else"),
            BlockEnd::ElseIf => self.check_keyword("elseif"),
            BlockEnd::EndIf => self.check_keyword("end") && self.check_next_keyword("if"),
            BlockEnd::EndFunction => {
                self.check_keyword("end") && self.check_next_keyword("function")
            }
            BlockEnd::EndProcedure => {
                self.check_keyword("end") && self.check_next_keyword("procedure")
            }
            BlockEnd::ForEnd => self.check_keyword("end") && self.check_next_keyword("for"),
            BlockEnd::WhileEnd => self.check_keyword("end") && self.check_next_keyword("while"),
            BlockEnd::DoEnd => self.check_keyword("end") && self.check_next_keyword("do"),
            BlockEnd::BareEnd => self.check_keyword("end") && self.check_next_is_line_end(),
            BlockEnd::Case => self.check_keyword("case"),
            BlockEnd::EndSelect => self.check_keyword("end") && self.check_next_keyword("select"),
        }
    }

    fn expect_keyword(&mut self, keyword: &str) -> ParseResult<()> {
        if self.check_keyword(keyword) {
            self.advance();
            Ok(())
        } else {
            Err(self.error(format!("expected `{keyword}`")))
        }
    }

    fn check_keyword(&self, keyword: &str) -> bool {
        matches!(&self.current().kind, TokenKind::Ident(value) if keyword_eq(value, keyword))
    }

    fn check_next_keyword(&self, keyword: &str) -> bool {
        matches!(
            self.tokens.get(self.pos + 1).map(|token| &token.kind),
            Some(TokenKind::Ident(value)) if keyword_eq(value, keyword)
        )
    }

    fn check_next_is_line_end(&self) -> bool {
        matches!(
            self.tokens.get(self.pos + 1).map(|t| &t.kind),
            Some(
                TokenKind::Newline
                    | TokenKind::Colon
                    | TokenKind::Eof
                    | TokenKind::Comment(_)
                    | TokenKind::BlockComment(_)
            ) | None
        )
    }

    fn expect_ident(&mut self, message: &str) -> ParseResult<String> {
        match &self.current().kind {
            TokenKind::Ident(value) => {
                let value = value.clone();
                self.advance();
                Ok(value)
            }
            _ => Err(self.error(message)),
        }
    }

    fn expect(&mut self, kind: TokenKind, message: &str) -> ParseResult<()> {
        if self.eat(kind) {
            Ok(())
        } else {
            Err(self.error(message))
        }
    }

    fn eat(&mut self, kind: TokenKind) -> bool {
        if self.current().kind == kind {
            self.advance();
            true
        } else {
            false
        }
    }

    fn advance(&mut self) -> &Token {
        let old = self.pos;
        if !self.is_eof() {
            self.pos += 1;
        }
        &self.tokens[old]
    }

    fn current(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn is_eof(&self) -> bool {
        matches!(self.current().kind, TokenKind::Eof)
    }

    fn error(&self, message: impl Into<String>) -> Diagnostic {
        Diagnostic::error(self.current_pos(), message)
    }

    fn current_pos(&self) -> SourcePos {
        self.tokens
            .get(self.pos)
            .map(|token| token.pos.clone())
            .unwrap_or_else(|| SourcePos::new(self.filename.clone(), 1, 1))
    }
}

fn make_paren_ident_expr(ident: BasicIdent, args: Vec<Expr>) -> Expr {
    if args.is_empty() || (ident.suffix.is_some() && args.len() == 1) {
        Expr::ArrayRef {
            name: ident,
            indices: args,
        }
    } else {
        Expr::Call { name: ident, args }
    }
}

fn normalize_assignment_target(expr: Expr) -> Expr {
    match expr {
        Expr::Call { name, args } => Expr::ArrayRef {
            name,
            indices: args,
        },
        other => other,
    }
}

fn keyword_eq(value: &str, keyword: &str) -> bool {
    value.eq_ignore_ascii_case(keyword)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockEnd {
    Else,
    ElseIf,
    EndIf,
    EndFunction,
    EndProcedure,
    ForEnd,
    WhileEnd,
    DoEnd,
    BareEnd,
    Case,
    EndSelect,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse(source: &str) -> Program {
        let tokens = Lexer::new("test.bcl", source).lex();
        Parser::new("test.bcl".to_string(), tokens)
            .parse_program()
            .expect("parse failed")
    }

    #[test]
    fn parses_function_with_return() {
        let program = parse("function add%(left%, right%)\n return left% + right%\nend function\n");
        assert_eq!(program.functions.len(), 1);
        assert_eq!(program.functions[0].name.as_basic(), "add%");
        assert!(matches!(
            program.functions[0].body[0],
            Statement::Return { .. }
        ));
    }

    #[test]
    fn parses_multiline_and_nested_if() {
        let program = parse(
            "if score% >= 90 then\n if score% > 95 then\n  PRINT \"A+\"\n end if\nelse\n PRINT \"Not A\"\nend if\n",
        );
        assert!(matches!(program.statements[0], Statement::If { .. }));
    }

    #[test]
    fn parses_dependency_declarations() {
        let program =
            parse("require com.bascal.sort.bubbleSort\nimport com.bascal.sort.shakerSort\n");
        assert!(matches!(
            program.declarations[0],
            DependencyDecl::Require(_)
        ));
        assert!(matches!(program.declarations[1], DependencyDecl::Import(_)));
    }

    #[test]
    fn parses_standalone_call_and_array_ref() {
        let program = parse("bubbleSort%(data%(), 10)\nvalue% = data%(i%)\n");
        assert!(matches!(
            program.statements[0],
            Statement::ExprStmt(Expr::Call { .. })
        ));
        match &program.statements[1] {
            Statement::Assignment { value, .. } => {
                assert!(matches!(value, Expr::ArrayRef { .. }));
            }
            _ => panic!("expected assignment"),
        }
    }

    #[test]
    fn parses_basic_file_io_statements() {
        let program = parse(
            "open inputFile$ for input as #1\nline input #1, line$\nprint #2, line$\nclose #1\n",
        );
        assert!(matches!(program.statements[0], Statement::Open { .. }));
        assert!(matches!(program.statements[1], Statement::LineInput { .. }));
        assert!(matches!(program.statements[2], Statement::PrintFile { .. }));
        assert!(matches!(program.statements[3], Statement::Close { .. }));
    }

    #[test]
    fn rejects_returns_clause() {
        let tokens = Lexer::new(
            "test.bcl",
            "function add%() returns integer\nend function\n",
        )
        .lex();
        let result = Parser::new("test.bcl".to_string(), tokens).parse_program();
        assert!(result
            .unwrap_err()
            .iter()
            .any(|diagnostic| diagnostic.message.contains("returns")));
    }
}
