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
        Ok(FunctionDef { name, params, body })
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
        } else if self.check_keyword("close") {
            self.parse_close()
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
            self.parse_on_branch()
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
        let size = if self.eat(TokenKind::LParen) {
            let size = if self.eat(TokenKind::RParen) {
                None
            } else {
                let expr = self.parse_expr(0)?;
                self.expect(TokenKind::RParen, "expected `)` after DIM size")?;
                Some(expr)
            };
            size
        } else {
            None
        };
        self.consume_line_end()?;
        Ok(Statement::Dim { name, size })
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
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>();
        Ok(Statement::BlockComment(lines))
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

    fn parse_print(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("print")?;
        if self.eat(TokenKind::Hash) {
            let channel = self.parse_expr(0)?;
            self.expect(TokenKind::Comma, "expected `,` after file number")?;
            let mut exprs = Vec::new();
            if !self.at_line_end() {
                loop {
                    exprs.push(self.parse_expr(0)?);
                    if !self.eat(TokenKind::Comma) {
                        break;
                    }
                }
            }
            self.consume_line_end()?;
            return Ok(Statement::PrintFile { channel, exprs });
        }
        let mut exprs = Vec::new();
        if !self.at_line_end() {
            loop {
                exprs.push(self.parse_expr(0)?);
                if !self.eat(TokenKind::Comma) {
                    break;
                }
            }
        }
        self.consume_line_end()?;
        Ok(Statement::Print { exprs })
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
        } else {
            return Err(self.error("expected `input`, `output`, or `append`"));
        };
        self.expect_keyword("as")?;
        self.expect(TokenKind::Hash, "expected `#` before file number")?;
        let channel = self.parse_expr(0)?;
        self.consume_line_end()?;
        Ok(Statement::Open {
            mode,
            file,
            channel,
        })
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

    fn parse_close(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("close")?;
        self.expect(TokenKind::Hash, "expected `#` before file number")?;
        let channel = self.parse_expr(0)?;
        self.consume_line_end()?;
        Ok(Statement::Close { channel })
    }

    fn parse_return(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("return")?;
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
        let body = self.parse_block(&[BlockEnd::Next])?;
        self.expect_keyword("next")?;
        if matches!(self.current().kind, TokenKind::Ident(_)) {
            let _ = self.advance();
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
        let body = self.parse_block(&[BlockEnd::Wend])?;
        self.expect_keyword("wend")?;
        self.consume_line_end()?;
        Ok(Statement::While { condition, body })
    }

    fn parse_end_statement(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("end")?;
        if self.check_keyword("if") || self.check_keyword("function") || self.check_keyword("select") {
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
        let body = self.parse_block(&[BlockEnd::Loop])?;
        self.expect_keyword("loop")?;
        let post_condition = if self.check_keyword("while") || self.check_keyword("until") {
            Some(self.parse_do_condition()?)
        } else {
            None
        };
        self.consume_line_end()?;
        Ok(Statement::Do { condition, body, post_condition })
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

    fn parse_on_branch(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("on")?;
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

    fn parse_lprint(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("lprint")?;
        let mut exprs = Vec::new();
        if !self.at_line_end() {
            loop {
                exprs.push(self.parse_expr(0)?);
                if !self.eat(TokenKind::Comma) { break; }
            }
        }
        self.consume_line_end()?;
        Ok(Statement::Lprint(exprs))
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
        let expr = self.parse_expr(6)?;
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
            TokenKind::String(value) => {
                let value = value.clone();
                self.advance();
                Expr::String(value)
            }
            TokenKind::Ident(value) if keyword_eq(value, "not") => {
                self.advance();
                let expr = self.parse_expr(7)?;
                Expr::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(expr),
                }
            }
            TokenKind::Minus => {
                self.advance();
                let expr = self.parse_expr(7)?;
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
            TokenKind::Ident(value) if keyword_eq(value, "or") => Some((1, 2, BinaryOp::Or)),
            TokenKind::Ident(value) if keyword_eq(value, "and") => Some((3, 4, BinaryOp::And)),
            TokenKind::Eq => Some((5, 6, BinaryOp::Eq)),
            TokenKind::Ne => Some((5, 6, BinaryOp::Ne)),
            TokenKind::Lt => Some((5, 6, BinaryOp::Lt)),
            TokenKind::Le => Some((5, 6, BinaryOp::Le)),
            TokenKind::Gt => Some((5, 6, BinaryOp::Gt)),
            TokenKind::Ge => Some((5, 6, BinaryOp::Ge)),
            TokenKind::Plus => Some((9, 10, BinaryOp::Add)),
            TokenKind::Minus => Some((9, 10, BinaryOp::Sub)),
            TokenKind::Star => Some((11, 12, BinaryOp::Mul)),
            TokenKind::Slash => Some((11, 12, BinaryOp::Div)),
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
            BlockEnd::Next => self.check_keyword("next"),
            BlockEnd::Wend => self.check_keyword("wend"),
            BlockEnd::Loop => self.check_keyword("loop"),
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
    Next,
    Wend,
    Loop,
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
