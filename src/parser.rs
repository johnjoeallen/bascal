use crate::ast::*;
use crate::diagnostics::{Diagnostic, SourcePos};
use crate::lexer::{Token, TokenKind};

type ParseResult<T> = Result<T, Diagnostic>;

pub struct Parser {
    filename: String,
    tokens: Vec<Token>,
    pos: usize,
    pending_blank: bool,
    // >0 while parsing a single-line `if ... then stmt [else stmt]`'s
    // then/else clause. Lets the shared end-of-line checks (`at_line_end`,
    // `consume_line_end`) treat a same-line `else` as a stopping point too
    // -- without this, a greedy multi-arg statement like PRINT's own
    // token-list loop has no way to know "else" isn't just another
    // expression to print. A counter, not a bool, so a nested single-line
    // `if` inside this one's body doesn't clear the flag while unwinding.
    single_line_if_depth: usize,
    // Extra statements produced by a single physical-line construct that
    // desugars to more than one `Statement` -- currently just multi-name
    // `dim a%, b%, c%`. `parse_statement` drains this before dispatching on
    // the next token, so callers that loop on `parse_statement()` (parse_block,
    // the top-level program loop, and a single-line `if`'s body) see the
    // extra statements as if they'd been parsed individually.
    pending_statements: std::collections::VecDeque<Stmt>,
}

impl Parser {
    pub fn new(filename: String, tokens: Vec<Token>) -> Self {
        Self {
            filename,
            tokens,
            pos: 0,
            pending_blank: false,
            single_line_if_depth: 0,
            pending_statements: std::collections::VecDeque::new(),
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
        let mut library_decl = None;
        let mut shared_decl = None;
        let mut declarations = Vec::new();
        let mut statements = Vec::new();
        let mut functions = Vec::new();
        let mut records = Vec::new();

        self.skip_newlines();
        while !self.is_eof() {
            if self.check_keyword("program") {
                let decl = self.parse_program_decl()?;
                if program_decl.is_some() {
                    return Err(self.error("only one `program` declaration is allowed per file"));
                }
                if library_decl.is_some() {
                    return Err(self.error("a file cannot have both a `program` declaration and a `library` declaration"));
                }
                if shared_decl.is_some() {
                    return Err(self.error("a file cannot have both a `program` declaration and a `shared` declaration"));
                }
                program_decl = Some(decl);
            } else if self.check_keyword("library") {
                if library_decl.is_some() {
                    return Err(self.error("only one `library` declaration is allowed per file"));
                }
                if program_decl.is_some() {
                    return Err(self.error("a file cannot have both a `program` declaration and a `library` declaration"));
                }
                if shared_decl.is_some() {
                    return Err(self.error("a file cannot have both a `library` declaration and a `shared` declaration"));
                }
                self.expect_keyword("library")?;
                let name = self.expect_ident("expected library name after `library`")?;
                self.consume_line_end()?;
                library_decl = Some(name);
            } else if self.check_keyword("shared") {
                if shared_decl.is_some() {
                    return Err(self.error("only one `shared` declaration is allowed per file"));
                }
                if program_decl.is_some() {
                    return Err(self.error("a file cannot have both a `program` declaration and a `shared` declaration"));
                }
                if library_decl.is_some() {
                    return Err(self.error("a file cannot have both a `library` declaration and a `shared` declaration"));
                }
                self.expect_keyword("shared")?;
                let name = self.expect_ident("expected shared-file name after `shared`")?;
                self.consume_line_end()?;
                shared_decl = Some(name);
            } else if self.check_keyword("common") {
                return Err(self.error(
                    "the `common` keyword has been removed -- declare shared variables with \
                     `dim` inside a `shared <name>` file instead",
                ));
            } else if self.check_keyword("require") {
                declarations.push(self.parse_path_decl(false)?);
            } else if self.check_keyword("import") {
                declarations.push(self.parse_path_decl(true)?);
            } else if self.check_keyword("function") {
                functions.push(self.parse_function()?);
            } else if self.check_keyword("procedure") {
                functions.push(self.parse_procedure()?);
            } else if self.check_method_keyword() {
                functions.push(self.parse_method()?);
            } else if self.check_keyword("record") {
                records.push(self.parse_record_def()?);
            } else {
                statements.push(self.parse_statement()?);
                while !self.pending_statements.is_empty() {
                    statements.push(self.parse_statement()?);
                }
            }
            if self.take_pending_blank() && !self.is_eof() {
                let blank_pos = self.current_pos();
                statements.push(Stmt::new(Statement::BlankLine, blank_pos));
            }
        }

        Ok(Program {
            program_decl,
            library_decl,
            shared_decl,
            declarations,
            common: Vec::new(),
            statements,
            functions,
            records,
        })
    }

    fn parse_program_decl(&mut self) -> ParseResult<ProgramDecl> {
        self.expect_keyword("program")?;
        let name = self.expect_ident("expected program name")?;
        let shared = if self.check_keyword("shared") {
            self.advance();
            Some(self.expect_ident("expected shared-file name after `shared`")?)
        } else {
            None
        };
        self.consume_line_end()?;
        Ok(ProgramDecl { name, shared })
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
        let fn_pos = self.current_pos();
        self.expect_keyword("function")?;
        let name = BasicIdent::parse(&self.expect_ident("expected function name")?);
        self.expect(TokenKind::LParen, "expected `(` after function name")?;
        let params = self.parse_param_list()?;
        self.expect(TokenKind::RParen, "expected `)` after function parameters")?;

        if self.check_keyword("returns") {
            return Err(self.error("`returns` clauses are not supported in BASCAL"));
        }
        self.consume_line_end()?;

        let body = self.parse_block(&[BlockEnd::EndFunction])?;
        self.expect_keyword("end")?;
        self.expect_keyword("function")?;
        self.consume_line_end()?;
        Ok(FunctionDef {
            name,
            params,
            body,
            is_procedure: false,
            receiver: None,
            pos: fn_pos,
        })
    }

    fn parse_procedure(&mut self) -> ParseResult<FunctionDef> {
        let fn_pos = self.current_pos();
        self.expect_keyword("procedure")?;
        let raw = self.expect_ident("expected procedure name")?;
        let name = BasicIdent::parse(&raw);
        if name.suffix.is_some() {
            return Err(self.error("procedure names must not carry a type suffix"));
        }
        self.expect(TokenKind::LParen, "expected `(` after procedure name")?;
        let params = self.parse_param_list()?;
        self.expect(TokenKind::RParen, "expected `)` after procedure parameters")?;
        self.consume_line_end()?;

        let body = self.parse_block(&[BlockEnd::EndProcedure])?;
        self.expect_keyword("end")?;
        self.expect_keyword("procedure")?;
        self.consume_line_end()?;
        Ok(FunctionDef {
            name,
            params,
            body,
            is_procedure: true,
            receiver: None,
            pos: fn_pos,
        })
    }

    fn check_method_keyword(&self) -> bool {
        matches!(&self.current().kind, TokenKind::Ident(raw) if BasicIdent::parse(raw).name.eq_ignore_ascii_case("method"))
    }

    fn parse_method(&mut self) -> ParseResult<FunctionDef> {
        let fn_pos = self.current_pos();
        let receiver = BasicIdent::parse(&self.expect_ident("expected method receiver type")?)
            .suffix
            .ok_or_else(|| self.error("method declarations need a receiver suffix, e.g. `method$`"))?;
        let name = BasicIdent::parse(&self.expect_ident("expected method name")?);
        if name.suffix.is_none() {
            return Err(self.error("method names need a result type suffix"));
        }
        self.expect(TokenKind::LParen, "expected `(` after method name")?;
        let params = self.parse_param_list()?;
        self.expect(TokenKind::RParen, "expected `)` after method parameters")?;
        self.consume_line_end()?;
        let body = self.parse_block(&[BlockEnd::EndMethod])?;
        self.expect_keyword("end")?;
        self.expect_keyword("method")?;
        self.consume_line_end()?;
        Ok(FunctionDef { name, params, body, is_procedure: false, receiver: Some(receiver), pos: fn_pos })
    }

    fn parse_record_def(&mut self) -> ParseResult<RecordDef> {
        self.expect_keyword("record")?;
        let name = self.expect_ident("expected record name")?;
        self.consume_line_end()?;
        self.skip_newlines();
        let mut fields = Vec::new();
        while !self.is_eof() && !(self.check_keyword("end") && self.check_next_keyword("record")) {
            let field_name = self.expect_ident("expected record field name")?;
            self.expect(TokenKind::Colon, "expected `:` after record field name")?;
            let ty = self.parse_record_field_type()?;
            self.consume_line_end()?;
            fields.push(RecordFieldDef {
                name: field_name,
                ty,
            });
            self.skip_newlines();
        }
        self.expect_keyword("end")?;
        self.expect_keyword("record")?;
        self.consume_line_end()?;
        Ok(RecordDef { name, fields })
    }

    fn parse_record_field_type(&mut self) -> ParseResult<RecordFieldType> {
        let raw = self.expect_ident("expected record field type")?;
        match raw.to_ascii_lowercase().as_str() {
            "int16" => Ok(RecordFieldType::Int16),
            "int32" => Ok(RecordFieldType::Int32),
            "float32" => Ok(RecordFieldType::Float32),
            "float64" => Ok(RecordFieldType::Float64),
            "string" => {
                self.expect(TokenKind::LParen, "expected `(` after `string`")?;
                let width = self.expect_number_literal("expected string field width")?;
                self.expect(TokenKind::RParen, "expected `)` after string field width")?;
                if width <= 0 {
                    return Err(self.error("string field width must be a positive integer"));
                }
                Ok(RecordFieldType::Str(width as u32))
            }
            other => Err(self.error(format!(
                "unknown record field type `{other}`; expected int16, int32, float32, float64, or string(N)"
            ))),
        }
    }

    fn expect_number_literal(&mut self, message: &str) -> ParseResult<i64> {
        match &self.current().kind {
            TokenKind::Number(value) => {
                let value = *value;
                self.advance();
                Ok(value)
            }
            _ => Err(self.error(message)),
        }
    }

    fn parse_file_decl(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("file")?;
        let var = BasicIdent::parse(&self.expect_ident("expected file variable name")?);
        let record_type = if self.check_keyword("as") {
            self.advance();
            Some(self.expect_ident("expected record type name after `as`")?)
        } else {
            None
        };
        self.expect(TokenKind::Eq, "expected `=` in file declaration")?;
        if !self.check_keyword("open") {
            return Err(self.error("expected `open(...)` in file declaration"));
        }
        self.advance(); // consume `open`
        self.expect(TokenKind::LParen, "expected `(` after `open`")?;
        let path = self.parse_expr(0)?;
        self.expect(TokenKind::RParen, "expected `)` after file path")?;
        // The record form (`file db as Student = open(...)`) always means
        // random access, with no `for ...` to write -- its width comes from
        // the record type instead. The plain sequential-handle form has no
        // record type to infer a mode from, so it's the one place this
        // sugar needs `for input/output/append` spelled out, same as raw
        // `open ... for ...` would.
        let mode = if record_type.is_none() {
            self.expect_keyword("for")?;
            let mode = if self.check_keyword("input") {
                self.advance();
                OpenMode::Input
            } else if self.check_keyword("output") {
                self.advance();
                OpenMode::Output
            } else if self.check_keyword("append") {
                self.advance();
                OpenMode::Append
            } else {
                return Err(self.error(
                    "expected `input`, `output`, or `append` after `for` in file declaration",
                ));
            };
            Some(mode)
        } else {
            None
        };
        self.consume_line_end()?;
        Ok(Statement::FileDecl {
            var,
            record_type,
            path,
            mode,
        })
    }

    fn parse_param_list(&mut self) -> ParseResult<Vec<Param>> {
        let mut items = Vec::new();
        if matches!(self.current().kind, TokenKind::RParen) {
            return Ok(items);
        }
        loop {
            let mode = if self.check_keyword("byref") {
                self.advance();
                ParamMode::ByRef
            } else if self.check_keyword("byval") {
                self.advance();
                ParamMode::ByVal
            } else {
                ParamMode::ByVal
            };
            let name = BasicIdent::parse(&self.expect_ident("expected identifier")?);
            let axes = if self.eat(TokenKind::LParen) {
                let mut axes = Vec::new();
                loop {
                    if self.eat(TokenKind::Question) {
                        axes.push(None);
                    } else if let TokenKind::Number(value) = self.current().kind {
                        self.advance();
                        if value <= 0 {
                            return Err(self.error(
                                "array parameter capacity must be a positive integer -- e.g. \
                                 `arr%(100)`"
                                    .to_string(),
                            ));
                        }
                        axes.push(Some(value));
                    } else {
                        return Err(self.error(
                            "expected `?` or an integer capacity in array parameter \
                             declaration -- e.g. `arr%(?)` for an inferred 1-D capacity, \
                             `arr%(100)` for an explicit one"
                                .to_string(),
                        ));
                    }
                    if !self.eat(TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(TokenKind::RParen, "expected `)` after array parameter rank")?;
                Some(axes)
            } else {
                None
            };
            let default = if self.eat(TokenKind::Eq) {
                if axes.is_some() {
                    return Err(self.error("array parameters cannot have default values"));
                }
                if mode == ParamMode::ByRef {
                    return Err(self.error("`byref` parameters cannot have default values"));
                }
                Some(self.parse_expr(0)?)
            } else {
                None
            };
            items.push(Param { name, mode, default, axes });
            if !self.eat(TokenKind::Comma) {
                break;
            }
        }
        Ok(items)
    }

    fn parse_block(&mut self, ends: &[BlockEnd]) -> ParseResult<Vec<Stmt>> {
        let mut body = Vec::new();
        self.pending_blank = false;
        self.skip_newlines();
        while !self.is_eof() && !self.at_any_block_end(ends) {
            body.push(self.parse_statement()?);
            while !self.pending_statements.is_empty() {
                body.push(self.parse_statement()?);
            }
            if self.take_pending_blank() && !self.at_any_block_end(ends) && !self.is_eof() {
                let blank_pos = self.current_pos();
                body.push(Stmt::new(Statement::BlankLine, blank_pos));
            }
        }
        Ok(body)
    }

    /// Single dispatch point that turns a bare `Statement` into a
    /// position-carrying `Stmt` -- every caller that builds up a `Vec<Stmt>`
    /// (parse_block, the top-level program loop, a single-line `if`'s body)
    /// goes through this, so every statement in the AST carries a real
    /// source position without each of the ~60 `parse_xxx` functions having
    /// to thread it through by hand.
    fn parse_statement(&mut self) -> ParseResult<Stmt> {
        if let Some(stmt) = self.pending_statements.pop_front() {
            return Ok(stmt);
        }
        let start_pos = self.current_pos();
        let kind = self.parse_statement_kind()?;
        Ok(Stmt::new(kind, start_pos))
    }

    fn parse_statement_kind(&mut self) -> ParseResult<Statement> {
        if self.check_keyword("def") && self.check_next_keyword("fn") {
            self.parse_def_fn()
        } else if self.check_dim_keyword() {
            self.parse_dim()
        } else if self.check_keyword("file") {
            self.parse_file_decl()
        } else if self.check_keyword("record") {
            Err(self.error("`record` declarations are only valid at program level"))
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
            self.advance();
            self.consume_line_end()?;
            Ok(Statement::Stop)
        } else if self.check_keyword("cls") {
            self.advance();
            self.consume_line_end()?;
            Ok(Statement::Cls)
        } else if self.check_keyword("beep") {
            self.advance();
            self.consume_line_end()?;
            Ok(Statement::Beep)
        } else if self.check_keyword("system") {
            self.advance();
            self.consume_line_end()?;
            Ok(Statement::System)
        } else if self.check_keyword("randomize") {
            self.parse_randomize()
        } else if self.check_keyword("poke") {
            self.parse_poke()
        } else if self.check_keyword("out") {
            self.parse_out()
        } else if self.check_keyword("width") {
            self.parse_width()
        } else if self.check_keyword("clear") {
            self.advance();
            self.consume_line_end()?;
            Ok(Statement::Clear)
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
            Err(self.error(
                "the `common` keyword has been removed -- declare shared variables with `dim` \
                 inside a `shared <name>` file instead",
            ))
        } else if self.check_keyword("program") {
            Err(self.error("`program` declaration must appear before any statements"))
        } else if matches!(self.current().kind, TokenKind::Ident(_)) && self.check_next_is_colon() {
            self.parse_label()
        } else {
            self.parse_assignment_or_expr()
        }
    }

    fn parse_label(&mut self) -> ParseResult<Statement> {
        let name = self.expect_ident("expected label name")?;
        self.eat(TokenKind::Colon); // guaranteed present by check_next_is_colon
                                    // Unlike a genuine same-line separator (`x = 1: y = 2`), the label's
                                    // own colon is part of `name:` itself — a newline immediately after
                                    // it is this statement's normal end of line, not "blank line"
                                    // padding, so it doesn't count toward `count_and_skip_newlines`'s
                                    // extra-newline (pending_blank) detection below.
        if matches!(self.current().kind, TokenKind::Newline) {
            self.advance();
        }
        if self.count_and_skip_newlines() >= 1 {
            self.pending_blank = true;
        }
        Ok(Statement::Label(name))
    }

    /// Classic single-line `DEF FN` (e.g. `DEF FN A(X) = X * X + 1`) is a
    /// deliberate scope decision to reject, not a missing feature --
    /// BASCAL's `function`/`procedure` blocks fully supersede it, and
    /// real-world `DEF FN` abuse (comma-operator side effects, colon-
    /// chained pseudo-statements) has no clean general conversion
    /// semantics. So this parses the grammar shape just far enough to
    /// recognize the statement as a unit -- including the "weird" forms
    /// -- purely so the rejection diagnostic below can point at the `DEF`
    /// token and name the construct specifically, instead of failing with
    /// a generic error deep inside whatever expression follows `=`. No
    /// AST node is built and nothing is lowered; this always errors.
    fn parse_def_fn(&mut self) -> ParseResult<Statement> {
        let def_pos = self.current_pos();
        self.expect_keyword("def")?;
        self.expect_keyword("fn")?;
        self.expect_ident("expected DEF FN name")?;
        if self.eat(TokenKind::LParen) {
            if !matches!(self.current().kind, TokenKind::RParen) {
                loop {
                    self.expect_ident("expected DEF FN parameter name")?;
                    if !self.eat(TokenKind::Comma) {
                        break;
                    }
                }
            }
            self.expect(TokenKind::RParen, "expected `)` after DEF FN parameters")?;
        }
        self.expect(TokenKind::Eq, "expected `=` in DEF FN")?;
        // The expression after `=` is intentionally not structurally
        // parsed -- real-world DEF FN abuse includes a parenthesized
        // comma-operator list (`(A = A + 1, A)`) and colon-chained
        // pseudo-statements (`A = A + 1 : A`), neither of which is a
        // single expression tree BASCAL's expression parser understands.
        // Since this always ends in rejection regardless of shape, just
        // skip every token through end of line/statement -- DEF FN is a
        // single-line, non-block statement, so nothing after the trailing
        // newline belongs to it, and skipping is what keeps the error
        // pointed at `DEF` instead of choking partway through the
        // expression on some unexpected comma or colon.
        while !matches!(
            self.current().kind,
            TokenKind::Newline
                | TokenKind::Eof
                | TokenKind::Comment(_)
                | TokenKind::BlockComment(_)
        ) {
            self.advance();
        }
        self.consume_line_end()?;
        Err(Diagnostic::error(
            def_pos,
            "DEF FN is not supported by BASCAL -- `function` and `procedure` blocks (with \
             parameters, byref/byval, and return values) fully replace it, and real-world DEF \
             FN abuse has no clean general conversion semantics. Rewrite this by hand as a \
             `function` before converting this file -- DEF FN is not automatically converted.",
        ))
    }

    fn parse_dim(&mut self) -> ParseResult<Statement> {
        self.expect_dim_keyword()?;
        let (name, is_array, sizes) = self.parse_dim_one()?;
        // `dim a%, b%(10), c%` -- each comma-separated name is its own
        // declaration; queue the rest and return the first so every caller
        // that loops on `parse_statement()` sees them as separate statements.
        while self.eat(TokenKind::Comma) {
            let dim_pos = self.current_pos();
            let (name, is_array, sizes) = self.parse_dim_one()?;
            self.pending_statements.push_back(Stmt::new(
                Statement::Dim {
                    name,
                    is_array,
                    sizes,
                },
                dim_pos,
            ));
        }
        self.consume_line_end()?;
        Ok(Statement::Dim {
            name,
            is_array,
            sizes,
        })
    }

    fn parse_dim_one(&mut self) -> ParseResult<(BasicIdent, bool, Vec<Expr>)> {
        let name = BasicIdent::parse(&self.expect_ident("expected DIM variable name")?);
        let (is_array, sizes) = if self.eat(TokenKind::LParen) {
            if self.eat(TokenKind::RParen) {
                (true, Vec::new()) // dim arr%() — declare without bounds
            } else {
                let mut sizes = vec![self.parse_expr(0)?];
                while self.eat(TokenKind::Comma) {
                    sizes.push(self.parse_expr(0)?);
                }
                self.expect(TokenKind::RParen, "expected `)` after DIM dimensions")?;
                (true, sizes)
            }
        } else {
            (false, Vec::new())
        };
        Ok((name, is_array, sizes))
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
                trimmed
                    .strip_prefix('*')
                    .map(|s| s.trim())
                    .unwrap_or(trimmed)
                    .to_string()
            })
            .collect::<Vec<_>>();
        let start = lines.iter().position(|l| !l.is_empty()).unwrap_or(0);
        let end = lines
            .iter()
            .rposition(|l| !l.is_empty())
            .map(|i| i + 1)
            .unwrap_or(start);
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
                self.expect(
                    TokenKind::Semicolon,
                    "expected `;` after USING format string",
                )?;
                let tokens = self.parse_print_tokens()?;
                self.consume_line_end()?;
                return Ok(Statement::PrintFileUsing {
                    channel,
                    format,
                    tokens,
                });
            }
            let tokens = self.parse_print_tokens()?;
            self.consume_line_end()?;
            return Ok(Statement::PrintFile { channel, tokens });
        }
        if self.check_keyword("using") {
            self.expect_keyword("using")?;
            let format = self.parse_expr(0)?;
            self.expect(
                TokenKind::Semicolon,
                "expected `;` after USING format string",
            )?;
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
        Ok(Statement::Open {
            mode,
            file,
            channel,
            len,
        })
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
        Ok(Statement::Field {
            channel,
            fields,
            record_type: None,
            string_fields: None,
            field_types: None,
        })
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
        Ok(Statement::Get {
            channel,
            record,
            var,
            require_existing: false,
            record_length: None,
        })
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
        Ok(Statement::Put {
            channel,
            record,
            var,
            provided_fields: None,
        })
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

    /// Parses the condition of an `if`/`elseif`/`while`/`do` — a single
    /// expression, or a chain of `&&`-only or `||`-only operands. `&&`/`||`
    /// are deliberately kept out of `infix_binding_power` so `parse_expr(0)`
    /// naturally stops right before them; mixing the two operators in one
    /// condition is a parse error for now (split into nested `if`s instead).
    fn parse_condition(&mut self) -> ParseResult<Expr> {
        let mut expr = self.parse_expr(0)?;
        if matches!(self.current().kind, TokenKind::AndAnd) {
            while self.eat(TokenKind::AndAnd) {
                let rhs = self.parse_expr(0)?;
                expr = Expr::Binary {
                    left: Box::new(expr),
                    op: BinaryOp::AndAnd,
                    right: Box::new(rhs),
                };
            }
        } else if matches!(self.current().kind, TokenKind::OrOr) {
            while self.eat(TokenKind::OrOr) {
                let rhs = self.parse_expr(0)?;
                expr = Expr::Binary {
                    left: Box::new(expr),
                    op: BinaryOp::OrOr,
                    right: Box::new(rhs),
                };
            }
        }
        if matches!(self.current().kind, TokenKind::AndAnd | TokenKind::OrOr) {
            return Err(self.error(
                "mixing `&&` and `||` in one condition isn't supported yet — split into nested `if` statements",
            ));
        }
        Ok(expr)
    }

    fn parse_if(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("if")?;
        let condition = self.parse_condition()?;
        self.expect_keyword("then")?;
        // Classic single-line form: a statement follows `then` directly on
        // the same line, no `end if` needed. Block form needs a newline
        // right after `then`; that's the only thing distinguishing the two.
        if !self.at_line_end() {
            return self.parse_single_line_if(condition);
        }
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
        Ok(Statement::If {
            condition,
            then_body,
            else_body,
        })
    }

    /// `if cond then stmt [: stmt ...] [else stmt [: stmt ...]]` -- no
    /// `end if`, terminated by the end of the physical line. `elseif` isn't
    /// supported here, same as classic BASIC: it needs the block form.
    fn parse_single_line_if(&mut self, condition: Expr) -> ParseResult<Statement> {
        let (then_body, saw_else) = self.parse_single_line_if_body()?;
        let else_body = if saw_else {
            self.expect_keyword("else")?;
            self.parse_single_line_if_body()?.0
        } else {
            Vec::new()
        };
        Ok(Statement::If {
            condition,
            then_body,
            else_body,
        })
    }

    /// Parses statements up to the end of the physical line (each one's own
    /// `consume_line_end` already accepts `:` as a same-line separator).
    /// Returns whether the stop was caused by an `else` still on this same
    /// line (vs. a real newline/EOF, where an `else` on the *next* line
    /// must not be mistaken for this if's else-clause).
    fn parse_single_line_if_body(&mut self) -> ParseResult<(Vec<Stmt>, bool)> {
        self.single_line_if_depth += 1;
        let result = (|| {
            let mut body = Vec::new();
            loop {
                body.push(self.parse_statement()?);
                while !self.pending_statements.is_empty() {
                    body.push(self.parse_statement()?);
                }
                if self.previous_was_newline() || self.is_eof() {
                    return Ok((body, false));
                }
                if self.check_keyword("else") {
                    return Ok((body, true));
                }
            }
        })();
        self.single_line_if_depth -= 1;
        result
    }

    fn previous_was_newline(&self) -> bool {
        self.pos > 0 && matches!(self.tokens[self.pos - 1].kind, TokenKind::Newline)
    }

    /// Constructs a nested `Statement::If` directly (an `elseif` chain
    /// desugars into nested `If`s in `else_body`), bypassing
    /// `parse_statement`'s single dispatch point -- so it captures its own
    /// start position and wraps into a `Stmt` itself.
    fn parse_elseif(&mut self) -> ParseResult<Stmt> {
        let start_pos = self.current_pos();
        self.expect_keyword("elseif")?;
        let condition = self.parse_condition()?;
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
        Ok(Stmt::new(
            Statement::If {
                condition,
                then_body,
                else_body,
            },
            start_pos,
        ))
    }

    fn parse_for(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("for")?;
        let var = BasicIdent::parse(&self.expect_ident("expected FOR variable")?);
        self.expect(TokenKind::Eq, "expected `=` in FOR statement")?;
        let start = self.parse_expr(0)?;
        let (end, step) = if self.check_keyword("downto") {
            self.expect_keyword("downto")?;
            let end = self.parse_expr(0)?;
            // `for i = A downto B` is sugar for `for i = A to B step -1`.
            (end, Some(Expr::Integer(-1)))
        } else {
            self.expect_keyword("to")?;
            let end = self.parse_expr(0)?;
            let step = if self.check_keyword("step") {
                self.expect_keyword("step")?;
                Some(self.parse_expr(0)?)
            } else {
                None
            };
            (end, step)
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
        let condition = self.parse_condition()?;
        self.consume_line_end()?;
        // `wend` is classic BASIC's own WHILE terminator, accepted alongside
        // `end while`/bare `end` -- without this, `wend` isn't recognized at
        // all, so it silently parses as a no-op statement and the block
        // keeps consuming everything after it (including the program's own
        // `end`) looking for a real terminator.
        let body = self.parse_block(&[BlockEnd::WhileEnd, BlockEnd::BareEnd, BlockEnd::Wend])?;
        if self.check_keyword("wend") {
            self.expect_keyword("wend")?;
        } else {
            self.expect_keyword("end")?;
            if self.check_keyword("while") {
                self.expect_keyword("while")?;
            }
        }
        self.consume_line_end()?;
        Ok(Statement::While { condition, body })
    }

    fn parse_end_statement(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("end")?;
        if self.check_keyword("if")
            || self.check_keyword("function")
            || self.check_keyword("select")
            || self.check_keyword("procedure")
            || self.check_keyword("while")
            || self.check_keyword("for")
            || self.check_keyword("do")
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
        let body = self.parse_block(&[BlockEnd::DoEnd, BlockEnd::BareEnd, BlockEnd::Loop])?;
        // `loop [while/until cond]` is the post-check form -- the condition
        // is tested after the body runs, so it always executes at least
        // once. `end [do]` is the existing pre-check-or-bare form.
        let post_condition = if self.check_keyword("loop") {
            self.expect_keyword("loop")?;
            let post = if self.check_keyword("while") || self.check_keyword("until") {
                Some(self.parse_do_condition()?)
            } else {
                None
            };
            self.consume_line_end()?;
            post
        } else {
            self.expect_keyword("end")?;
            if self.check_keyword("do") {
                self.expect_keyword("do")?;
            }
            self.consume_line_end()?;
            None
        };
        Ok(Statement::Do {
            condition,
            body,
            post_condition,
        })
    }

    fn parse_do_condition(&mut self) -> ParseResult<DoCondition> {
        let is_while = if self.check_keyword("while") {
            self.advance();
            true
        } else {
            self.expect_keyword("until")?;
            false
        };
        let expr = self.parse_condition()?;
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
        Ok(Statement::SelectCase {
            expr,
            cases,
            else_body,
        })
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
        if self.check_keyword("for") || self.check_keyword("while") || self.check_keyword("do") {
            return Err(self.error(
                "`exit` no longer takes a loop-type keyword -- just write `exit`; \
                 the transpiler resolves which enclosing loop it leaves",
            ));
        }
        self.consume_line_end()?;
        Ok(Statement::Exit)
    }

    /// `goto`/`gosub`/`on ... goto`/`on ... gosub`/`resume` targets must
    /// name a label — BASCAL owns line numbering, so raw line-number
    /// literals aren't legal targets. `on error goto` has its own variant
    /// below because `on error goto 0` (disable the trap) is a legal
    /// numeric sentinel.
    fn parse_label_target(&mut self, keyword: &str) -> ParseResult<Expr> {
        let expr = self.parse_expr(0)?;
        match &expr {
            Expr::Ident(_) => Ok(expr),
            Expr::Integer(_) => Err(self.error(format!(
                "`{keyword}` target must be a label, not a line number — \
                 BASCAL manages line numbers itself; declare a label with \
                 `name:` and use `{keyword} name` instead"
            ))),
            _ => Err(self.error(format!("`{keyword}` target must be a label name"))),
        }
    }

    fn parse_on_error_goto_target(&mut self) -> ParseResult<Expr> {
        let expr = self.parse_expr(0)?;
        match &expr {
            Expr::Ident(_) => Ok(expr),
            Expr::Integer(0) => Ok(expr),
            Expr::Integer(_) => Err(self.error(
                "`on error goto` target must be a label, not a line number \
                 (except `on error goto 0` to disable the trap) — declare a \
                 label with `name:` instead",
            )),
            _ => Err(self.error("`on error goto` target must be a label name or `0`")),
        }
    }

    fn parse_goto(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("goto")?;
        let target = self.parse_label_target("goto")?;
        self.consume_line_end()?;
        Ok(Statement::Goto(target))
    }

    fn parse_gosub(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("gosub")?;
        let target = self.parse_label_target("gosub")?;
        self.consume_line_end()?;
        Ok(Statement::Gosub(target))
    }

    fn parse_on(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("on")?;
        if self.check_keyword("error") {
            self.expect_keyword("error")?;
            self.expect_keyword("goto")?;
            let target = self.parse_on_error_goto_target()?;
            self.consume_line_end()?;
            return Ok(Statement::OnErrorGoto { target });
        }
        let expr = self.parse_expr(0)?;
        let is_gosub = if self.check_keyword("goto") {
            self.advance();
            false
        } else {
            self.expect_keyword("gosub")?;
            true
        };
        let keyword = if is_gosub {
            "on ... gosub"
        } else {
            "on ... goto"
        };
        let mut targets = Vec::new();
        loop {
            targets.push(self.parse_label_target(keyword)?);
            if !self.eat(TokenKind::Comma) {
                break;
            }
        }
        self.consume_line_end()?;
        Ok(Statement::OnBranch {
            expr,
            targets,
            is_gosub,
        })
    }

    fn parse_resume(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("resume")?;
        let target = if self.at_line_end() {
            ResumeTarget::Same
        } else if self.check_keyword("next") {
            self.advance();
            ResumeTarget::Next
        } else {
            ResumeTarget::Line(self.parse_label_target("resume")?)
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
            self.expect(
                TokenKind::Semicolon,
                "expected `;` after USING format string",
            )?;
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
                if !self.eat(TokenKind::Comma) {
                    break;
                }
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
                if !self.eat(TokenKind::Comma) {
                    break;
                }
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
            if !self.eat(TokenKind::Comma) {
                break;
            }
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

    fn parse_out(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("out")?;
        let port = self.parse_expr(0)?;
        self.expect(TokenKind::Comma, "expected `,` in OUT")?;
        let value = self.parse_expr(0)?;
        self.consume_line_end()?;
        Ok(Statement::Out { port, value })
    }

    fn parse_width(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("width")?;
        let channel = if self.eat(TokenKind::Hash) {
            let ch = self.parse_expr(0)?;
            self.expect(TokenKind::Comma, "expected `,` after channel in WIDTH")?;
            Some(ch)
        } else {
            None
        };
        let cols = self.parse_expr(0)?;
        self.consume_line_end()?;
        Ok(Statement::Width { channel, cols })
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
            if !self.eat(TokenKind::Comma) {
                break;
            }
        }
        self.consume_line_end()?;
        Ok(Statement::Data(values))
    }

    fn parse_read(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("read")?;
        let mut vars = Vec::new();
        loop {
            vars.push(self.parse_expr(0)?);
            if !self.eat(TokenKind::Comma) {
                break;
            }
        }
        self.consume_line_end()?;
        Ok(Statement::Read(vars))
    }

    fn parse_restore(&mut self) -> ParseResult<Statement> {
        self.expect_keyword("restore")?;
        let target = if !self.at_line_end() {
            Some(self.parse_label_target("restore")?)
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
            if let Some((target, start, len)) = self.try_mid_assign_call(&expr)? {
                let value = self.parse_expr(0)?;
                self.consume_line_end()?;
                return Ok(Statement::MidAssign {
                    target: Box::new(target),
                    start: Box::new(start),
                    len: len.map(Box::new),
                    value: Box::new(value),
                });
            }
            let value = self.parse_expr(0)?;
            self.consume_line_end()?;
            Ok(Statement::Assignment {
                target: normalize_assignment_target(expr),
                value,
            })
        } else if let Some(op) = self.eat_compound_assign_op() {
            // `x% += e` desugars to `x% = x% + e` right here in the parser --
            // codegen and every later pass see an ordinary Assignment, same
            // as if the programmer had spelled the target out twice.
            let target = normalize_assignment_target(expr);
            let rhs = self.parse_expr(0)?;
            self.consume_line_end()?;
            let value = Expr::Binary {
                left: Box::new(target.clone()),
                op,
                right: Box::new(rhs),
            };
            Ok(Statement::Assignment { target, value })
        } else {
            self.consume_line_end()?;
            Ok(Statement::ExprStmt(expr))
        }
    }

    /// If `expr` is a `MID$(<target>, <start>[, <len>])` call shape (as an
    /// assignment target, not a value read), returns its decomposed parts.
    /// Returns `Ok(None)` for anything else -- including a *value-position*
    /// `MID$(...)` read, which never reaches this method since it's only
    /// called on the parsed left-hand side of `=`. Rejects a target that
    /// isn't a plain string variable or string array element (record/file
    /// DSL sugar, a nested call, etc. can't be spliced into in place).
    fn try_mid_assign_call(&self, expr: &Expr) -> ParseResult<Option<(Expr, Expr, Option<Expr>)>> {
        let Expr::Call { name, args } = expr else {
            return Ok(None);
        };
        if !name.name.eq_ignore_ascii_case("mid") || name.suffix != Some(TypeSuffix::String) {
            return Ok(None);
        }
        if args.len() != 2 && args.len() != 3 {
            return Ok(None);
        }
        let target = &args[0];
        if !matches!(target, Expr::Ident(_) | Expr::ArrayRef { .. }) {
            return Err(self.error(
                "MID$ assignment target must be a plain string variable or string array element",
            ));
        }
        let start = args[1].clone();
        let len = args.get(2).cloned();
        Ok(Some((target.clone(), start, len)))
    }

    fn eat_compound_assign_op(&mut self) -> Option<BinaryOp> {
        let op = match self.current().kind {
            TokenKind::PlusEq => BinaryOp::Add,
            TokenKind::MinusEq => BinaryOp::Sub,
            TokenKind::StarEq => BinaryOp::Mul,
            TokenKind::SlashEq => BinaryOp::Div,
            _ => return None,
        };
        self.advance();
        Some(op)
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
            // MS-BASIC's own boolean convention (see Operators and
            // Expressions in the manual): -1 is true, 0 is false. `TRUE`/
            // `FALSE` are transpile-time sugar for those literals, nothing more
            // -- no boolean type is introduced anywhere else in the transpiler.
            TokenKind::Ident(value) if keyword_eq(value, "true") => {
                self.advance();
                Expr::Integer(-1)
            }
            TokenKind::Ident(value) if keyword_eq(value, "false") => {
                self.advance();
                Expr::Integer(0)
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
                // A dotted identifier (`s.id`, `db.close`) already lexes as one
                // token (see lexer.rs `ident()`) — split on the last `.` to
                // build record-file field-access/method-call sugar. This is
                // safe because no other BASCAL construct produces a dotted
                // identifier through expression parsing (dotted `require`/
                // `import` paths are parsed via a separate grammar path).
                if let Some(dot_pos) = ident.name.rfind('.') {
                    let base = BasicIdent {
                        name: ident.name[..dot_pos].to_string(),
                        suffix: None,
                    };
                    let member = ident.name[dot_pos + 1..].to_string();
                    if self.eat(TokenKind::LParen) {
                        let args = self.parse_expr_list_until_rparen()?;
                        Expr::MethodCall {
                            base: Box::new(Expr::Ident(base)),
                            method: member,
                            args,
                        }
                    } else {
                        Expr::FieldAccess {
                            base: Box::new(Expr::Ident(base)),
                            field: member,
                        }
                    }
                } else if self.eat(TokenKind::LParen) {
                    let args = self.parse_expr_list_until_rparen()?;
                    make_paren_ident_expr(ident, args)
                } else if self.eat(TokenKind::LBracket) {
                    let index = self.parse_expr(0)?;
                    self.expect(TokenKind::RBracket, "expected `]` after file index")?;
                    Expr::FileIndex {
                        var: ident,
                        index: Box::new(index),
                    }
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
            TokenKind::LBrace => {
                self.advance();
                let fields = self.parse_record_lit_fields()?;
                Expr::RecordLit {
                    fields,
                    partial: false,
                }
            }
            TokenKind::Question => {
                self.advance();
                self.expect(TokenKind::LBrace, "expected `{` after `?`")?;
                let fields = self.parse_record_lit_fields()?;
                Expr::RecordLit {
                    fields,
                    partial: true,
                }
            }
            _ => return Err(self.error("expected expression")),
        };

        // Postfix `.field` / `.method(...)` — handles `db[i].field`, where the
        // preceding `]` prevents the lexer from gluing the dot into an
        // identifier token (unlike `s.id`, handled above).
        while self.eat(TokenKind::Dot) {
            let member = self.expect_ident("expected field or method name after `.`")?;
            if self.eat(TokenKind::LParen) {
                let args = self.parse_expr_list_until_rparen()?;
                left = if is_scalar_receiver(&left) {
                    Expr::ScalarMethodCall { base: Box::new(left), method: member, args }
                } else {
                    Expr::MethodCall { base: Box::new(left), method: member, args }
                };
            } else {
                left = Expr::FieldAccess {
                    base: Box::new(left),
                    field: member,
                };
            }
        }

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

    /// Parses `ident: expr, ident: expr, ... }` — the caller has already
    /// consumed the opening `{` (whether it arrived bare or after `?`).
    fn parse_record_lit_fields(&mut self) -> ParseResult<Vec<(String, Expr)>> {
        let mut fields = Vec::new();
        if !matches!(self.current().kind, TokenKind::RBrace) {
            loop {
                let field_name = self.expect_ident("expected field name in record literal")?;
                self.expect(
                    TokenKind::Colon,
                    "expected `:` after field name in record literal",
                )?;
                let value = self.parse_expr(0)?;
                fields.push((field_name, value));
                if !self.eat(TokenKind::Comma) {
                    break;
                }
            }
        }
        self.expect(TokenKind::RBrace, "expected `}` after record literal")?;
        Ok(fields)
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
        // Inside a single-line `if`'s then/else clause, a same-line `else`
        // ends the current statement too -- don't consume it, the caller
        // still needs to see it.
        if self.single_line_if_depth > 0 && self.check_keyword("else") {
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
        ) || (self.single_line_if_depth > 0 && self.check_keyword("else"))
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
            BlockEnd::EndMethod => self.check_keyword("end") && self.check_next_keyword("method"),
            BlockEnd::ForEnd => self.check_keyword("end") && self.check_next_keyword("for"),
            BlockEnd::WhileEnd => self.check_keyword("end") && self.check_next_keyword("while"),
            BlockEnd::Wend => self.check_keyword("wend"),
            BlockEnd::DoEnd => self.check_keyword("end") && self.check_next_keyword("do"),
            BlockEnd::BareEnd => self.check_keyword("end") && self.check_next_is_line_end(),
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

    /// `dim`/`declare` are interchangeable spellings of the same
    /// declaration statement -- `declare` reads better for the Pascal-
    /// leaning `--strict-vars` mode (see `resolver::reject_undeclared_
    /// variables`), but neither one implies or requires the other; either
    /// spelling works regardless of `--strict-vars`.
    fn check_dim_keyword(&self) -> bool {
        self.check_keyword("dim") || self.check_keyword("declare")
    }

    fn expect_dim_keyword(&mut self) -> ParseResult<()> {
        if self.check_dim_keyword() {
            self.advance();
            Ok(())
        } else {
            Err(self.error("expected `dim` or `declare`"))
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

    fn check_next_is_colon(&self) -> bool {
        matches!(
            self.tokens.get(self.pos + 1).map(|token| &token.kind),
            Some(TokenKind::Colon)
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

fn is_scalar_receiver(expr: &Expr) -> bool {
    match expr {
        Expr::String(_) | Expr::Integer(_) | Expr::Float(_) | Expr::HexLit(_) => true,
        Expr::Ident(ident) | Expr::Call { name: ident, .. } => ident.suffix.is_some(),
        Expr::ScalarMethodCall { .. } => true,
        _ => false,
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
    EndMethod,
    ForEnd,
    WhileEnd,
    Wend,
    DoEnd,
    BareEnd,
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
            program.functions[0].body[0].kind,
            Statement::Return { .. }
        ));
    }

    #[test]
    fn parses_multiline_and_nested_if() {
        let program = parse(
            "if score% >= 90 then\n if score% > 95 then\n  PRINT \"A+\"\n end if\nelse\n PRINT \"Not A\"\nend if\n",
        );
        assert!(matches!(program.statements[0].kind, Statement::If { .. }));
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
            program.statements[0].kind,
            Statement::ExprStmt(Expr::Call { .. })
        ));
        match &*program.statements[1] {
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
        assert!(matches!(program.statements[0].kind, Statement::Open { .. }));
        assert!(matches!(program.statements[1].kind, Statement::LineInput { .. }));
        assert!(matches!(program.statements[2].kind, Statement::PrintFile { .. }));
        assert!(matches!(program.statements[3].kind, Statement::Close { .. }));
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

    #[test]
    fn parses_record_def() {
        let program = parse(
            "record Student\n    id: int16\n    name: string(20)\n    score: float64\nend record\nend\n",
        );
        assert_eq!(program.records.len(), 1);
        let rec = &program.records[0];
        assert_eq!(rec.name, "Student");
        assert_eq!(rec.fields.len(), 3);
        assert_eq!(rec.fields[0].name, "id");
        assert_eq!(rec.fields[0].ty, RecordFieldType::Int16);
        assert_eq!(rec.fields[1].name, "name");
        assert_eq!(rec.fields[1].ty, RecordFieldType::Str(20));
        assert_eq!(rec.fields[2].name, "score");
        assert_eq!(rec.fields[2].ty, RecordFieldType::Float64);
    }

    #[test]
    fn declare_is_an_interchangeable_synonym_for_dim() {
        let dim_program = parse("dim x%, y%(20)\nend\n");
        let declare_program = parse("declare x%, y%(20)\nend\n");
        // Compare statement *shape* only -- `declare` is a longer keyword
        // than `dim`, so the two programs' statements don't share column
        // positions even though they parse to the same `Statement`s.
        let kinds = |p: &Program| p.statements.iter().map(|s| s.kind.clone()).collect::<Vec<_>>();
        assert_eq!(kinds(&dim_program), kinds(&declare_program));
        match &*declare_program.statements[0] {
            Statement::Dim { name, is_array, .. } => {
                assert_eq!(name.name, "x");
                assert!(!is_array);
            }
            other => panic!("expected Dim, got {other:?}"),
        }
    }

    #[test]
    fn parses_file_decl() {
        let program = parse("file db as Student = open(\"students.dat\")\nend\n");
        match &*program.statements[0] {
            Statement::FileDecl {
                var,
                record_type,
                path,
                mode,
            } => {
                assert_eq!(var.name, "db");
                assert_eq!(record_type.as_deref(), Some("Student"));
                assert!(matches!(path, Expr::String(s) if s == "students.dat"));
                assert_eq!(*mode, None);
            }
            other => panic!("expected FileDecl, got {other:?}"),
        }
    }

    #[test]
    fn parses_sequential_file_decl() {
        let program = parse("file scores = open(\"scores.csv\") for output\nend\n");
        match &*program.statements[0] {
            Statement::FileDecl {
                var,
                record_type,
                path,
                mode,
            } => {
                assert_eq!(var.name, "scores");
                assert_eq!(*record_type, None);
                assert!(matches!(path, Expr::String(s) if s == "scores.csv"));
                assert_eq!(*mode, Some(OpenMode::Output));
            }
            other => panic!("expected FileDecl, got {other:?}"),
        }
    }

    #[test]
    fn parses_file_index_expr() {
        let program = parse("let s = db[i]\nend\n");
        match &*program.statements[0] {
            Statement::Assignment { value, .. } => {
                assert!(matches!(value, Expr::FileIndex { .. }));
            }
            other => panic!("expected assignment, got {other:?}"),
        }
    }

    #[test]
    fn parses_bracketed_field_access() {
        // db[i].field — the `.` follows `]`, so it must come through the
        // lexer's new standalone Dot token, not the glued-identifier path.
        let program = parse("db[i].score = 61.5\nend\n");
        match &*program.statements[0] {
            Statement::Assignment { target, .. } => match target {
                Expr::FieldAccess { base, field } => {
                    assert_eq!(field, "score");
                    assert!(matches!(base.as_ref(), Expr::FileIndex { .. }));
                }
                other => panic!("expected FieldAccess, got {other:?}"),
            },
            other => panic!("expected assignment, got {other:?}"),
        }
    }

    #[test]
    fn parses_dotted_field_access_and_method_call() {
        let program = parse("print s.id\ndb.close()\nend\n");
        match &*program.statements[0] {
            Statement::Print { tokens } => match &tokens[0] {
                PrintToken::Expr(Expr::FieldAccess { base, field }) => {
                    assert_eq!(field, "id");
                    assert!(matches!(base.as_ref(), Expr::Ident(i) if i.name == "s"));
                }
                other => panic!("expected FieldAccess print token, got {other:?}"),
            },
            other => panic!("expected print, got {other:?}"),
        }
        match &*program.statements[1] {
            Statement::ExprStmt(Expr::MethodCall { base, method, args }) => {
                assert!(matches!(base.as_ref(), Expr::Ident(i) if i.name == "db"));
                assert_eq!(method, "close");
                assert!(args.is_empty());
            }
            other => panic!("expected MethodCall exprstmt, got {other:?}"),
        }
    }

    #[test]
    fn parses_record_literal() {
        let program = parse("db[1] = { id: 1, name: \"Alice\", score: 95.0 }\nend\n");
        match &*program.statements[0] {
            Statement::Assignment { target, value } => {
                assert!(matches!(target, Expr::FileIndex { .. }));
                match value {
                    Expr::RecordLit { fields, partial } => {
                        assert!(!*partial);
                        assert_eq!(fields.len(), 3);
                        assert_eq!(fields[0].0, "id");
                        assert_eq!(fields[1].0, "name");
                        assert_eq!(fields[2].0, "score");
                    }
                    other => panic!("expected RecordLit, got {other:?}"),
                }
            }
            other => panic!("expected assignment, got {other:?}"),
        }
    }

    #[test]
    fn parses_partial_record_literal() {
        let program = parse("db[1] = ?{ score: 88.0 }\nend\n");
        match &*program.statements[0] {
            Statement::Assignment { value, .. } => match value {
                Expr::RecordLit { fields, partial } => {
                    assert!(*partial);
                    assert_eq!(fields.len(), 1);
                    assert_eq!(fields[0].0, "score");
                }
                other => panic!("expected RecordLit, got {other:?}"),
            },
            other => panic!("expected assignment, got {other:?}"),
        }
    }

    #[test]
    fn parses_for_downto_as_step_negative_one() {
        let program = parse("for i = 3 downto 1\nprint i\nend for\nend\n");
        match &*program.statements[0] {
            Statement::For {
                start, end, step, ..
            } => {
                assert!(matches!(start, Expr::Integer(3)));
                assert!(matches!(end, Expr::Integer(1)));
                assert!(matches!(step, Some(Expr::Integer(-1))));
            }
            other => panic!("expected for statement, got {other:?}"),
        }
    }

    #[test]
    fn parses_double_amp_chain_as_left_folded_and_and() {
        let program = parse("if a > 0 && b > 0 && c > 0 then\nprint a\nend if\nend\n");
        match &*program.statements[0] {
            Statement::If { condition, .. } => match condition {
                Expr::Binary {
                    op: BinaryOp::AndAnd,
                    left,
                    right,
                } => {
                    assert!(matches!(
                        right.as_ref(),
                        Expr::Binary {
                            op: BinaryOp::Gt,
                            ..
                        }
                    ));
                    match left.as_ref() {
                        Expr::Binary {
                            op: BinaryOp::AndAnd,
                            ..
                        } => {}
                        other => panic!("expected nested && on the left, got {other:?}"),
                    }
                }
                other => panic!("expected && chain, got {other:?}"),
            },
            other => panic!("expected if statement, got {other:?}"),
        }
    }

    #[test]
    fn parses_double_pipe_chain_as_or_or() {
        let program = parse("if a = 1 || a = 2 then\nprint a\nend if\nend\n");
        match &*program.statements[0] {
            Statement::If { condition, .. } => {
                assert!(matches!(
                    condition,
                    Expr::Binary {
                        op: BinaryOp::OrOr,
                        ..
                    }
                ));
            }
            other => panic!("expected if statement, got {other:?}"),
        }
    }

    #[test]
    fn rejects_mixed_double_amp_and_double_pipe_in_one_condition() {
        let tokens = Lexer::new("test.bcl", "if a && b || c then\nprint a\nend if\nend\n").lex();
        let result = Parser::new("test.bcl".to_string(), tokens).parse_program();
        let errs = result.expect_err("expected a parse error for mixed && and ||");
        assert!(errs
            .iter()
            .any(|d| d.message.contains("mixing `&&` and `||`")));
    }

    #[test]
    fn not_still_applies_to_a_single_operand_of_a_double_amp_chain() {
        let program = parse("if not flag && b > 0 then\nprint b\nend if\nend\n");
        match &*program.statements[0] {
            Statement::If { condition, .. } => match condition {
                Expr::Binary {
                    op: BinaryOp::AndAnd,
                    left,
                    ..
                } => {
                    assert!(matches!(
                        left.as_ref(),
                        Expr::Unary {
                            op: UnaryOp::Not,
                            ..
                        }
                    ));
                }
                other => panic!("expected && chain, got {other:?}"),
            },
            other => panic!("expected if statement, got {other:?}"),
        }
    }

    #[test]
    fn do_until_condition_supports_double_pipe_chain() {
        let program = parse("do until done || attempts >= 3\nprint 1\nend do\nend\n");
        match &*program.statements[0] {
            Statement::Do {
                condition: Some(cond),
                ..
            } => {
                assert!(!cond.is_while);
                assert!(matches!(
                    cond.expr,
                    Expr::Binary {
                        op: BinaryOp::OrOr,
                        ..
                    }
                ));
            }
            other => panic!("expected do statement with condition, got {other:?}"),
        }
    }

    #[test]
    fn parses_label_declaration_and_following_statement() {
        let program = parse("skip:\nprint \"after\"\nend\n");
        assert!(matches!(&*program.statements[0], Statement::Label(name) if name == "skip"));
        assert!(matches!(program.statements[1].kind, Statement::Print { .. }));
    }

    #[test]
    fn parses_label_and_statement_sharing_one_colon() {
        // The label's own `:` doubles as the statement separator, so
        // `skip: print "hi"` puts both statements on one physical line.
        let program = parse("skip: print \"hi\"\nend\n");
        assert!(matches!(&*program.statements[0], Statement::Label(name) if name == "skip"));
        assert!(matches!(program.statements[1].kind, Statement::Print { .. }));
    }

    #[test]
    fn goto_and_gosub_accept_label_targets() {
        let program = parse("goto there\ngosub there\nend\nthere:\nprint 1\n");
        assert!(
            matches!(&*program.statements[0], Statement::Goto(Expr::Ident(ident)) if ident.name == "there")
        );
        assert!(
            matches!(&*program.statements[1], Statement::Gosub(Expr::Ident(ident)) if ident.name == "there")
        );
    }

    #[test]
    fn goto_rejects_numeric_line_number_target() {
        let tokens = Lexer::new("test.bcl", "goto 100\nend\n").lex();
        let result = Parser::new("test.bcl".to_string(), tokens).parse_program();
        let errs = result.expect_err("expected a parse error for a numeric goto target");
        assert!(errs
            .iter()
            .any(|d| d.message.contains("must be a label, not a line number")));
    }

    #[test]
    fn resume_rejects_numeric_line_number_target() {
        let tokens = Lexer::new("test.bcl", "resume 100\nend\n").lex();
        let result = Parser::new("test.bcl".to_string(), tokens).parse_program();
        let errs = result.expect_err("expected a parse error for a numeric resume target");
        assert!(errs
            .iter()
            .any(|d| d.message.contains("must be a label, not a line number")));
    }

    #[test]
    fn on_error_goto_zero_sentinel_is_still_allowed() {
        let program = parse("on error goto 0\nend\n");
        assert!(matches!(
            &*program.statements[0],
            Statement::OnErrorGoto {
                target: Expr::Integer(0)
            }
        ));
    }

    #[test]
    fn on_error_goto_rejects_nonzero_numeric_target() {
        let tokens = Lexer::new("test.bcl", "on error goto 9000\nend\n").lex();
        let result = Parser::new("test.bcl".to_string(), tokens).parse_program();
        let errs = result.expect_err("expected a parse error for a numeric on-error-goto target");
        assert!(errs
            .iter()
            .any(|d| d.message.contains("except `on error goto 0`")));
    }

    #[test]
    fn on_goto_rejects_numeric_targets_in_the_list() {
        let tokens = Lexer::new("test.bcl", "on choice% goto 10, 20\nend\n").lex();
        let result = Parser::new("test.bcl".to_string(), tokens).parse_program();
        let errs = result.expect_err("expected a parse error for numeric on...goto targets");
        assert!(errs
            .iter()
            .any(|d| d.message.contains("must be a label, not a line number")));
    }

    #[test]
    fn parses_do_loop_until_as_post_condition() {
        let program = parse("do\nprint 1\nloop until k% > 3\nend\n");
        match &*program.statements[0] {
            Statement::Do {
                condition,
                post_condition: Some(cond),
                ..
            } => {
                assert!(condition.is_none());
                assert!(!cond.is_while);
            }
            other => panic!("expected do statement with a post-condition, got {other:?}"),
        }
    }

    #[test]
    fn parses_do_loop_while_as_post_condition() {
        let program = parse("do\nprint 1\nloop while j% <= 3\nend\n");
        match &*program.statements[0] {
            Statement::Do {
                condition,
                post_condition: Some(cond),
                ..
            } => {
                assert!(condition.is_none());
                assert!(cond.is_while);
            }
            other => panic!("expected do statement with a post-condition, got {other:?}"),
        }
    }

    #[test]
    fn parses_bare_do_loop_with_no_condition_at_all() {
        let program = parse("do\nn% = n% + 1\nloop\nend\n");
        match &*program.statements[0] {
            Statement::Do {
                condition,
                post_condition,
                ..
            } => {
                assert!(condition.is_none());
                assert!(post_condition.is_none());
            }
            other => panic!("expected bare do statement, got {other:?}"),
        }
    }

    #[test]
    fn do_end_do_still_works_alongside_loop_until() {
        // The pre-existing `end`/`end do` terminator must keep working now
        // that `loop [while/until]` is a second valid terminator.
        let program = parse("do while k% <= 3\nprint k%\nend do\nend\n");
        match &*program.statements[0] {
            Statement::Do {
                condition: Some(cond),
                post_condition,
                ..
            } => {
                assert!(cond.is_while);
                assert!(post_condition.is_none());
            }
            other => panic!("expected do-while statement, got {other:?}"),
        }
    }

    #[test]
    fn parses_bare_exit_inside_for_while_and_do() {
        let program = parse(
            "for i% = 1 to 5\nexit\nend for\nwhile 1\nexit\nend while\ndo\nexit\nend do\nend\n",
        );
        assert!(
            matches!(&*program.statements[0], Statement::For { body, .. } if matches!(body[0].kind, Statement::Exit))
        );
        assert!(
            matches!(&*program.statements[1], Statement::While { body, .. } if matches!(body[0].kind, Statement::Exit))
        );
        assert!(
            matches!(&*program.statements[2], Statement::Do { body, .. } if matches!(body[0].kind, Statement::Exit))
        );
    }

    #[test]
    fn exit_rejects_the_old_qualified_forms() {
        for keyword in ["for", "while", "do"] {
            let source = format!("do\nexit {keyword}\nend do\nend\n");
            let tokens = Lexer::new("test.bcl", &source).lex();
            let result = Parser::new("test.bcl".to_string(), tokens).parse_program();
            let errs = result.expect_err(&format!("expected a parse error for `exit {keyword}`"));
            assert!(errs
                .iter()
                .any(|d| d.message.contains("no longer takes a loop-type keyword")));
        }
    }

    #[test]
    fn wend_closes_a_while_loop() {
        let program = parse("while p% < 10\nprint p%\nwend\nprint \"after\"\nend\n");
        assert!(matches!(program.statements[0].kind, Statement::While { .. }));
        // The statement after `wend` must be a sibling of the while loop,
        // not part of its body -- this is exactly the case that silently
        // broke before `wend` was a recognized terminator.
        assert!(matches!(program.statements[1].kind, Statement::Print { .. }));
        if let Statement::While { body, .. } = &*program.statements[0] {
            assert_eq!(
                body.len(),
                1,
                "wend must not be absorbed into the loop body"
            );
        }
    }

    #[test]
    fn end_while_and_bare_end_still_work_alongside_wend() {
        let program = parse("while p% < 10\nprint p%\nend while\nend\n");
        assert!(matches!(program.statements[0].kind, Statement::While { .. }));
        let program = parse("while p% < 10\nprint p%\nend\nend\n");
        assert!(matches!(program.statements[0].kind, Statement::While { .. }));
    }

    #[test]
    fn single_line_if_needs_no_end_if() {
        let program = parse("if x% > 0 then print \"positive\"\nprint \"after\"\nend\n");
        match &*program.statements[0] {
            Statement::If {
                then_body,
                else_body,
                ..
            } => {
                assert_eq!(then_body.len(), 1);
                assert!(else_body.is_empty());
            }
            other => panic!("expected if statement, got {other:?}"),
        }
        // The statement after the single-line if must be its sibling, not
        // absorbed into the then-body.
        assert!(matches!(program.statements[1].kind, Statement::Print { .. }));
    }

    #[test]
    fn single_line_if_supports_else_on_the_same_line() {
        let program = parse("if x% > 0 then print \"a\" else print \"b\"\nend\n");
        match &*program.statements[0] {
            Statement::If {
                then_body,
                else_body,
                ..
            } => {
                assert_eq!(then_body.len(), 1);
                assert_eq!(else_body.len(), 1);
            }
            other => panic!("expected if statement, got {other:?}"),
        }
    }

    #[test]
    fn single_line_if_else_does_not_bleed_into_the_next_physical_line() {
        // A bare `else` starting the *next* line, unattached to any `if`,
        // is invalid -- proves the first if's then-clause really stopped
        // at the newline instead of somehow reaching across it.
        let tokens = Lexer::new(
            "test.bcl",
            "if x% > 0 then print \"a\"\nelse print \"stray\"\nend\n",
        )
        .lex();
        let result = Parser::new("test.bcl".to_string(), tokens).parse_program();
        assert!(
            result.is_err(),
            "a dangling `else` on its own line must not parse"
        );

        // A second line with its *own* legitimate if/else must not have
        // that else misattributed to the first line's if.
        let program =
            parse("if x% > 0 then print \"a\"\nif y% > 0 then print \"b\" else print \"c\"\nend\n");
        match &*program.statements[0] {
            Statement::If { else_body, .. } => assert!(else_body.is_empty()),
            other => panic!("expected if statement, got {other:?}"),
        }
        match &*program.statements[1] {
            Statement::If { else_body, .. } => assert_eq!(else_body.len(), 1),
            other => panic!("expected second if statement, got {other:?}"),
        }
    }

    #[test]
    fn single_line_if_supports_colon_chained_statements() {
        let program = parse("if x% > 0 then y% = 1: z% = 2\nend\n");
        match &*program.statements[0] {
            Statement::If { then_body, .. } => assert_eq!(then_body.len(), 2),
            other => panic!("expected if statement, got {other:?}"),
        }
    }

    #[test]
    fn nested_single_line_if_resolves_dangling_else_to_the_innermost_if() {
        let program =
            parse("if a% = 1 then if b% = 2 then print \"both\" else print \"only a\"\nend\n");
        match &*program.statements[0] {
            Statement::If {
                then_body,
                else_body: outer_else,
                ..
            } => {
                assert!(outer_else.is_empty());
                match &*then_body[0] {
                    Statement::If {
                        else_body: inner_else,
                        ..
                    } => assert_eq!(inner_else.len(), 1),
                    other => panic!("expected nested if, got {other:?}"),
                }
            }
            other => panic!("expected if statement, got {other:?}"),
        }
    }

    #[test]
    fn multiline_if_still_requires_end_if() {
        // A newline directly after `then` must still select the block form.
        let program = parse("if x% > 0 then\nprint \"positive\"\nend if\nend\n");
        assert!(matches!(program.statements[0].kind, Statement::If { .. }));
    }

    #[test]
    fn parses_scalar_method_declarations_and_chained_calls() {
        let program = parse("method$ capitalize$()\nreturn self$\nend method\nmethod$ pad$(n%)\nreturn self$\nend method\ns$ = name$.capitalize().pad(2)\nend\n");
        assert_eq!(program.functions.len(), 2);
        assert_eq!(program.functions[0].receiver, Some(TypeSuffix::String));
        assert_eq!(program.functions[0].name.as_basic(), "capitalize$");
        match &*program.statements[0] {
            Statement::Assignment { value, .. } => match value {
                Expr::ScalarMethodCall { base, method, .. } => {
                    assert_eq!(method, "pad");
                    assert!(matches!(base.as_ref(), Expr::ScalarMethodCall { method, .. } if method == "capitalize"));
                }
                other => panic!("expected scalar method chain, got {other:?}"),
            },
            other => panic!("expected assignment, got {other:?}"),
        }
    }
}
