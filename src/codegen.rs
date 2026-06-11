use std::collections::{HashMap, HashSet};

use crate::ast::*;

const BASIC_BUILTINS: &[&str] = &[
    // Type-suffixed single-arg — parser creates Expr::ArrayRef for these
    "ucase", "lcase", "str", "chr", "hex", "oct", "space", "environ",
    "command", "ltrim", "rtrim", "trim",
    // Multi-arg string (Expr::Call, but include for completeness)
    "left", "right", "mid", "instr", "format", "string",
    // Single-arg numeric (no suffix → Expr::Call already, but included for safety)
    "len", "val", "asc", "sqr", "abs", "int", "fix", "sgn", "rnd", "eof",
    "sin", "cos", "tan", "atn", "log", "exp", "cint", "clng", "csng", "cdbl",
    "peek", "inp", "lof", "loc", "pos", "csrlin", "freefile",
    // Multi-arg numeric
    "ubound", "lbound", "iif",
    // Random-access record packing/unpacking
    "mki", "mkl", "mks", "mkd",
    "cvi", "cvl", "cvs", "cvd",
];

pub struct CodeGenerator {
    next_label: usize,
    indent: usize,
    output: String,
    functions: Vec<FunctionInfo>,
    known_callables: HashSet<String>,
    line_numbers: bool,
    loop_exit_stack: Vec<String>,
}

#[derive(Debug, Clone)]
struct FunctionInfo {
    source_name: BasicIdent,
    stem: String,
    label: String,
    result: BasicIdent,
    params: Vec<(BasicIdent, BasicIdent)>,
    is_procedure: bool,
    globals: HashSet<String>,
}

impl CodeGenerator {
    pub fn new() -> Self {
        Self {
            next_label: 1,
            indent: 0,
            output: String::new(),
            functions: Vec::new(),
            known_callables: HashSet::new(),
            line_numbers: false,
            loop_exit_stack: Vec::new(),
        }
    }

    pub fn with_line_numbers(mut self, value: bool) -> Self {
        self.line_numbers = value;
        self
    }

    pub fn generate(mut self, program: &Program) -> String {
        self.functions = program
            .functions
            .iter()
            .map(FunctionInfo::from_def)
            .collect();

        self.known_callables = self
            .functions
            .iter()
            .map(|f| f.source_name.name.to_ascii_lowercase())
            .chain(BASIC_BUILTINS.iter().map(|s| s.to_string()))
            .collect();

        self.line("' BASCAL generated BASIC");
        self.line("' Functions are lowered to global variables, labels, and GOSUB");

        for block in &program.common {
            let vars = block
                .vars
                .iter()
                .map(|v| {
                    if v.is_array {
                        format!("{}()", v.name.as_basic())
                    } else {
                        v.name.as_basic()
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            self.line(&format!("COMMON {vars}"));
        }

        if !program.declarations.is_empty() {
            self.line("' TODO: resolve BASCAL dependency selectors during link");
            for declaration in &program.declarations {
                match declaration {
                    DependencyDecl::Require(symbol) => {
                        self.line(&format!("' require {}", symbol.raw))
                    }
                    DependencyDecl::Import(symbol) => {
                        self.line(&format!("' import {} (alias for require)", symbol.raw))
                    }
                }
            }
        }

        if !program.statements.is_empty() {
            self.blank();
            self.statements(&program.statements, None);
        }

        if !program.functions.is_empty() {
            if !ends_with_end(&program.statements) {
                self.line("END");
            }
            for function in &program.functions {
                self.function(function);
            }
        }

        number_basic_lines(&self.output, self.line_numbers)
    }

    fn function(&mut self, function: &FunctionDef) {
        let info = self
            .function_info(&function.name)
            .expect("function table should contain every function")
            .clone();
        let params = function
            .params
            .iter()
            .map(|p| BasicIdent { name: p.name.to_ascii_lowercase(), suffix: p.suffix }.as_basic())
            .collect::<Vec<_>>()
            .join(", ");
        let lowered_name =
            BasicIdent { name: function.name.name.to_ascii_lowercase(), suffix: function.name.suffix }
                .as_basic();
        let kind = if function.is_procedure { "procedure" } else { "function" };
        self.blank();
        self.line(&format!("' {kind} {}({})", lowered_name, params));
        self.line(&format!("{}:", info.label));
        self.indent += 1;
        self.statements(&function.body, Some(&info));
        if !ends_with_return(&function.body) {
            self.line("RETURN");
        }
        self.indent -= 1;
        self.line(&format!("' end {kind} {}", lowered_name));
    }

    fn statements(&mut self, statements: &[Statement], current_function: Option<&FunctionInfo>) {
        for statement in statements {
            self.statement(statement, current_function);
        }
    }

    fn statement(&mut self, statement: &Statement, current_function: Option<&FunctionInfo>) {
        match statement {
            Statement::Dim { name, size } => match size {
                Some(size) => {
                    let (prelude, size) = self.expr(size, current_function);
                    self.lines(prelude);
                    self.line(&format!(
                        "DIM {}({})",
                        self.ident(name, current_function),
                        size
                    ));
                }
                None => self.line(&format!("DIM {}", self.ident(name, current_function))),
            },
            Statement::Open { mode, file, channel, len } => {
                let (file_prelude, file) = self.expr(file, current_function);
                let (channel_prelude, channel) = self.expr(channel, current_function);
                self.lines(file_prelude);
                self.lines(channel_prelude);
                let mode_str = match mode {
                    OpenMode::Input => "INPUT",
                    OpenMode::Output => "OUTPUT",
                    OpenMode::Append => "APPEND",
                    OpenMode::Random => "RANDOM",
                    OpenMode::Binary => "BINARY",
                };
                let len_clause = if let Some(len_expr) = len {
                    let (len_pre, len_val) = self.expr(len_expr, current_function);
                    self.lines(len_pre);
                    format!(" LEN = {len_val}")
                } else {
                    String::new()
                };
                self.line(&format!("OPEN {file} FOR {mode_str} AS #{channel}{len_clause}"));
            }
            Statement::LineInput { channel, target } => {
                let (channel_prelude, channel) = self.expr(channel, current_function);
                let (target_prelude, target) = self.expr(target, current_function);
                self.lines(channel_prelude);
                self.lines(target_prelude);
                self.line(&format!("LINE INPUT #{channel}, {target}"));
            }
            Statement::PrintFile { channel, tokens } => {
                let (channel_prelude, channel) = self.expr(channel, current_function);
                self.lines(channel_prelude);
                let body = self.render_print_tokens(tokens, current_function);
                if body.is_empty() {
                    self.line(&format!("PRINT #{channel}"));
                } else {
                    self.line(&format!("PRINT #{channel}, {body}"));
                }
            }
            Statement::Close { channel } => {
                let (channel_prelude, channel) = self.expr(channel, current_function);
                self.lines(channel_prelude);
                self.line(&format!("CLOSE #{channel}"));
            }
            Statement::Kill { file } => {
                let (prelude, file) = self.expr(file, current_function);
                self.lines(prelude);
                self.line(&format!("KILL {file}"));
            }
            Statement::Name { from, to } => {
                let (from_prelude, from) = self.expr(from, current_function);
                let (to_prelude, to) = self.expr(to, current_function);
                self.lines(from_prelude);
                self.lines(to_prelude);
                self.line(&format!("NAME {from} AS {to}"));
            }
            Statement::Assignment { target, value } => {
                let (target_prelude, target) = self.expr(target, current_function);
                let (value_prelude, value) = self.expr(value, current_function);
                self.lines(target_prelude);
                self.lines(value_prelude);
                self.line(&format!("{target} = {value}"));
            }
            Statement::Print { tokens } => {
                let body = self.render_print_tokens(tokens, current_function);
                if body.is_empty() {
                    self.line("PRINT");
                } else {
                    self.line(&format!("PRINT {body}"));
                }
            }
            Statement::ReturnVoid => {
                self.line("RETURN");
            }
            Statement::Return { value } => {
                let Some(info) = current_function else {
                    let (prelude, value) = self.expr(value, current_function);
                    self.lines(prelude);
                    self.line(&format!("RETURN {}", value));
                    return;
                };
                if info.is_procedure {
                    self.line("RETURN");
                } else {
                    let (prelude, value) = self.expr(value, current_function);
                    self.lines(prelude);
                    self.line(&format!("{} = {}", info.result.as_basic(), value));
                    self.line("RETURN");
                }
            }
            Statement::If {
                condition,
                then_body,
                else_body,
            } => self.if_statement(condition, then_body, else_body, current_function),
            Statement::For {
                var,
                start,
                end,
                step,
                body,
            } => {
                let (start_prelude, start) = self.expr(start, current_function);
                let (end_prelude, end) = self.expr(end, current_function);
                let step = step.as_ref().map(|step| self.expr(step, current_function));
                self.lines(start_prelude);
                self.lines(end_prelude);
                let step = if let Some((step_prelude, step)) = step {
                    self.lines(step_prelude);
                    format!(" STEP {step}")
                } else {
                    String::new()
                };
                self.line(&format!(
                    "FOR {} = {start} TO {end}{step}",
                    self.ident(var, current_function)
                ));
                self.indent += 1;
                self.statements(body, current_function);
                self.indent -= 1;
                self.line(&format!("NEXT {}", self.ident(var, current_function)));
            }
            Statement::While { condition, body } => {
                let id = self.next_label;
                self.next_label += 1;
                let top_label = format!("WHILE_{id:04}_TOP");
                let end_label = format!("WHILE_{id:04}_END");
                self.line(&format!("{top_label}:"));
                let (prelude, condition) = self.expr(condition, current_function);
                self.lines(prelude);
                self.line(&format!("IF ({condition}) = 0 THEN GOTO {end_label}"));
                self.indent += 1;
                self.loop_exit_stack.push(end_label.clone());
                self.statements(body, current_function);
                self.loop_exit_stack.pop();
                self.line(&format!("GOTO {top_label}"));
                self.indent -= 1;
                self.line(&format!("{end_label}:"));
                self.line("REM END WHILE");
            }
            Statement::Do { condition, body, post_condition } => {
                let id = self.next_label;
                self.next_label += 1;
                let top_label = format!("DO_{id:04}_TOP");
                let end_label = format!("DO_{id:04}_END");
                self.line(&format!("{top_label}:"));
                if let Some(cond) = condition {
                    let (prelude, expr) = self.expr(&cond.expr, current_function);
                    self.lines(prelude);
                    if cond.is_while {
                        self.line(&format!("IF ({expr}) = 0 THEN GOTO {end_label}"));
                    } else {
                        self.line(&format!("IF ({expr}) <> 0 THEN GOTO {end_label}"));
                    }
                }
                self.indent += 1;
                self.loop_exit_stack.push(end_label.clone());
                self.statements(body, current_function);
                self.loop_exit_stack.pop();
                if let Some(cond) = post_condition {
                    let (prelude, expr) = self.expr(&cond.expr, current_function);
                    self.lines(prelude);
                    if cond.is_while {
                        self.line(&format!("IF ({expr}) <> 0 THEN GOTO {top_label}"));
                    } else {
                        self.line(&format!("IF ({expr}) = 0 THEN GOTO {top_label}"));
                    }
                } else if condition.is_none() {
                    self.line(&format!("GOTO {top_label}"));
                }
                self.indent -= 1;
                self.line(&format!("{end_label}:"));
                self.line("REM END DO");
            }
            Statement::ExprStmt(expr_stmt) => self.expr_statement(expr_stmt, current_function),
            Statement::End => self.line("END"),
            Statement::Stop => self.line("STOP"),
            Statement::Cls => self.line("CLS"),
            Statement::Beep => self.line("BEEP"),
            Statement::System => self.line("SYSTEM"),
            Statement::Randomize(expr) => {
                if let Some(expr) = expr {
                    let (prelude, expr) = self.expr(expr, current_function);
                    self.lines(prelude);
                    self.line(&format!("RANDOMIZE {expr}"));
                } else {
                    self.line("RANDOMIZE");
                }
            }
            Statement::Swap(a, b) => {
                let (a_prelude, a) = self.expr(a, current_function);
                let (b_prelude, b) = self.expr(b, current_function);
                self.lines(a_prelude);
                self.lines(b_prelude);
                self.line(&format!("SWAP {a}, {b}"));
            }
            Statement::Goto(target) => {
                let (prelude, target) = self.expr(target, current_function);
                self.lines(prelude);
                self.line(&format!("GOTO {target}"));
            }
            Statement::Gosub(target) => {
                let (prelude, target) = self.expr(target, current_function);
                self.lines(prelude);
                self.line(&format!("GOSUB {target}"));
            }
            Statement::Input { prompt, vars } => {
                let mut rendered = Vec::new();
                for var in vars {
                    let (prelude, var) = self.expr(var, current_function);
                    self.lines(prelude);
                    rendered.push(var);
                }
                let prompt_part = match prompt {
                    Some(p) => format!("\"{p}\"; "),
                    None => String::new(),
                };
                self.line(&format!("INPUT {}{}", prompt_part, rendered.join(", ")));
            }
            Statement::InputFile { channel, vars } => {
                let (channel_prelude, channel) = self.expr(channel, current_function);
                self.lines(channel_prelude);
                let mut rendered = Vec::new();
                for var in vars {
                    let (prelude, var) = self.expr(var, current_function);
                    self.lines(prelude);
                    rendered.push(var);
                }
                self.line(&format!("INPUT #{channel}, {}", rendered.join(", ")));
            }
            Statement::Data(values) => {
                let mut rendered = Vec::new();
                for val in values {
                    let (prelude, val) = self.expr(val, current_function);
                    self.lines(prelude);
                    rendered.push(val);
                }
                self.line(&format!("DATA {}", rendered.join(", ")));
            }
            Statement::Read(vars) => {
                let mut rendered = Vec::new();
                for var in vars {
                    let (prelude, var) = self.expr(var, current_function);
                    self.lines(prelude);
                    rendered.push(var);
                }
                self.line(&format!("READ {}", rendered.join(", ")));
            }
            Statement::Restore(target) => {
                if let Some(target) = target {
                    let (prelude, target) = self.expr(target, current_function);
                    self.lines(prelude);
                    self.line(&format!("RESTORE {target}"));
                } else {
                    self.line("RESTORE");
                }
            }
            Statement::Const { name, value } => {
                let (prelude, value) = self.expr(value, current_function);
                self.lines(prelude);
                self.line(&format!(
                    "CONST {} = {value}",
                    BasicIdent { name: name.name.to_ascii_lowercase(), suffix: name.suffix }
                        .as_basic()
                ));
            }
            Statement::Write { channel, exprs } => {
                let (channel_prelude, channel) = self.expr(channel, current_function);
                self.lines(channel_prelude);
                let mut rendered = Vec::new();
                for item in exprs {
                    let (prelude, item) = self.expr(item, current_function);
                    self.lines(prelude);
                    rendered.push(item);
                }
                self.line(&format!("WRITE #{channel}, {}", rendered.join(", ")));
            }
            Statement::Field { channel, fields } => {
                let (ch_pre, ch) = self.expr(channel, current_function);
                self.lines(ch_pre);
                let mut parts = Vec::new();
                for (width, var) in fields {
                    let (w_pre, w) = self.expr(width, current_function);
                    self.lines(w_pre);
                    parts.push(format!("{w} AS {}", self.ident(var, current_function)));
                }
                self.line(&format!("FIELD #{ch}, {}", parts.join(", ")));
            }
            Statement::Get { channel, record, var } => {
                let (ch_pre, ch) = self.expr(channel, current_function);
                self.lines(ch_pre);
                match (record, var) {
                    (None, None) => self.line(&format!("GET #{ch}")),
                    (Some(rec), None) => {
                        let (r_pre, r) = self.expr(rec, current_function);
                        self.lines(r_pre);
                        self.line(&format!("GET #{ch}, {r}"));
                    }
                    (None, Some(v)) => {
                        let (v_pre, v) = self.expr(v, current_function);
                        self.lines(v_pre);
                        self.line(&format!("GET #{ch}, , {v}"));
                    }
                    (Some(rec), Some(v)) => {
                        let (r_pre, r) = self.expr(rec, current_function);
                        let (v_pre, v) = self.expr(v, current_function);
                        self.lines(r_pre);
                        self.lines(v_pre);
                        self.line(&format!("GET #{ch}, {r}, {v}"));
                    }
                }
            }
            Statement::Put { channel, record, var } => {
                let (ch_pre, ch) = self.expr(channel, current_function);
                self.lines(ch_pre);
                match (record, var) {
                    (None, None) => self.line(&format!("PUT #{ch}")),
                    (Some(rec), None) => {
                        let (r_pre, r) = self.expr(rec, current_function);
                        self.lines(r_pre);
                        self.line(&format!("PUT #{ch}, {r}"));
                    }
                    (None, Some(v)) => {
                        let (v_pre, v) = self.expr(v, current_function);
                        self.lines(v_pre);
                        self.line(&format!("PUT #{ch}, , {v}"));
                    }
                    (Some(rec), Some(v)) => {
                        let (r_pre, r) = self.expr(rec, current_function);
                        let (v_pre, v) = self.expr(v, current_function);
                        self.lines(r_pre);
                        self.lines(v_pre);
                        self.line(&format!("PUT #{ch}, {r}, {v}"));
                    }
                }
            }
            Statement::Lset { var, value } => {
                let (v_pre, v) = self.expr(value, current_function);
                self.lines(v_pre);
                self.line(&format!("LSET {} = {v}", self.ident(var, current_function)));
            }
            Statement::Rset { var, value } => {
                let (v_pre, v) = self.expr(value, current_function);
                self.lines(v_pre);
                self.line(&format!("RSET {} = {v}", self.ident(var, current_function)));
            }
            Statement::Seek { channel, position } => {
                let (ch_pre, ch) = self.expr(channel, current_function);
                let (pos_pre, pos) = self.expr(position, current_function);
                self.lines(ch_pre);
                self.lines(pos_pre);
                self.line(&format!("SEEK #{ch}, {pos}"));
            }
            Statement::Lprint(tokens) => {
                let body = self.render_print_tokens(tokens, current_function);
                if body.is_empty() {
                    self.line("LPRINT");
                } else {
                    self.line(&format!("LPRINT {body}"));
                }
            }
            Statement::ExitFor => self.line("EXIT FOR"),
            Statement::ExitWhile => {
                if let Some(label) = self.loop_exit_stack.last().cloned() {
                    self.line(&format!("GOTO {label}"));
                } else {
                    self.line("' warning: EXIT WHILE outside of WHILE loop");
                }
            }
            Statement::ExitDo => {
                if let Some(label) = self.loop_exit_stack.last().cloned() {
                    self.line(&format!("GOTO {label}"));
                } else {
                    self.line("' warning: EXIT DO outside of DO loop");
                }
            }
            Statement::SelectCase { expr, cases, else_body } => {
                self.select_case(expr, cases, else_body, current_function);
            }
            Statement::Locate { row, col } => {
                let (row_prelude, row) = self.expr(row, current_function);
                let (col_prelude, col) = self.expr(col, current_function);
                self.lines(row_prelude);
                self.lines(col_prelude);
                self.line(&format!("LOCATE {row}, {col}"));
            }
            Statement::Color { fg, bg } => {
                let (fg_prelude, fg) = self.expr(fg, current_function);
                self.lines(fg_prelude);
                if let Some(bg) = bg {
                    let (bg_prelude, bg) = self.expr(bg, current_function);
                    self.lines(bg_prelude);
                    self.line(&format!("COLOR {fg}, {bg}"));
                } else {
                    self.line(&format!("COLOR {fg}"));
                }
            }
            Statement::OnBranch { expr, targets, is_gosub } => {
                let (prelude, expr) = self.expr(expr, current_function);
                self.lines(prelude);
                let mut rendered = Vec::new();
                for target in targets {
                    let (t_prelude, t) = self.expr(target, current_function);
                    self.lines(t_prelude);
                    rendered.push(t);
                }
                let keyword = if *is_gosub { "GOSUB" } else { "GOTO" };
                self.line(&format!("ON {expr} {keyword} {}", rendered.join(", ")));
            }
            Statement::GlobalDecl(_) => {}
            Statement::Raw(raw) => self.line(raw),
            Statement::BlockComment(lines) => {
                for line in lines {
                    if line.is_empty() {
                        self.blank();
                    } else {
                        self.line(&format!("' {line}"));
                    }
                }
            }
            Statement::BlankLine => self.blank(),
        }
    }

    fn expr_statement(&mut self, expr_stmt: &Expr, current_function: Option<&FunctionInfo>) {
        if let Some((name, args)) = callable_expr(expr_stmt) {
            if let Some(info) = self.function_info(name).cloned() {
                self.emit_call_statement(&info, args, current_function);
                return;
            }
        }

        let (prelude, expr_stmt) = self.expr(expr_stmt, current_function);
        self.lines(prelude);
        self.line(&expr_stmt);
    }

    fn if_statement(
        &mut self,
        condition: &Expr,
        then_body: &[Statement],
        else_body: &[Statement],
        current_function: Option<&FunctionInfo>,
    ) {
        let (prelude, condition) = self.expr(condition, current_function);
        self.lines(prelude);

        let id = self.next_label;
        self.next_label += 1;
        let else_label = format!("IF_{id:04}_ELSE");
        let end_label = format!("IF_{id:04}_END");

        if else_body.is_empty() {
            self.line(&format!("IF ({condition}) = 0 THEN GOTO {end_label}"));
            self.indent += 1;
            self.statements(then_body, current_function);
            self.indent -= 1;
            self.line(&format!("{end_label}:"));
            self.line("REM END IF");
        } else {
            self.line(&format!("IF ({condition}) = 0 THEN GOTO {else_label}"));
            self.indent += 1;
            self.statements(then_body, current_function);
            self.line(&format!("GOTO {end_label}"));
            self.indent -= 1;
            self.line(&format!("{else_label}:"));
            self.indent += 1;
            self.statements(else_body, current_function);
            self.indent -= 1;
            self.line(&format!("{end_label}:"));
            self.line("REM END IF");
        }
    }

    fn select_case(
        &mut self,
        expr: &Expr,
        cases: &[CaseClause],
        else_body: &[Statement],
        current_function: Option<&FunctionInfo>,
    ) {
        let id = self.next_label;
        self.next_label += 1;
        let end_label = format!("SEL_{id:04}_END");

        // Store the select expression in a temp variable to avoid re-evaluation.
        // The temp variable must carry the same type suffix as the expression.
        let (prelude, expr_str) = self.expr(expr, current_function);
        self.lines(prelude);
        let suffix = expr_type_suffix(expr);
        let temp = {
            let id = self.next_label;
            self.next_label += 1;
            format!("BCC_T{id}{suffix}")
        };
        self.line(&format!("{temp} = {expr_str}"));

        // Emit dispatch: one IF/GOTO per case clause.
        let case_labels: Vec<String> = (0..cases.len())
            .map(|i| format!("SEL_{id:04}_C{i}"))
            .collect();
        let else_label = format!("SEL_{id:04}_ELSE");

        for (i, clause) in cases.iter().enumerate() {
            let cond = clause
                .values
                .iter()
                .map(|v| self.case_value_cond(v, &temp, current_function))
                .collect::<Vec<_>>()
                .join(" OR ");
            self.line(&format!("IF ({cond}) <> 0 THEN GOTO {}", case_labels[i]));
        }
        self.line(&format!("GOTO {}", if else_body.is_empty() { &end_label } else { &else_label }));

        // Emit each case body.
        for (i, clause) in cases.iter().enumerate() {
            self.line(&format!("{}:", case_labels[i]));
            self.indent += 1;
            self.statements(&clause.body, current_function);
            self.line(&format!("GOTO {end_label}"));
            self.indent -= 1;
        }

        // Emit else body.
        if !else_body.is_empty() {
            self.line(&format!("{else_label}:"));
            self.indent += 1;
            self.statements(else_body, current_function);
            self.indent -= 1;
        }

        self.line(&format!("{end_label}:"));
        self.line("REM END SELECT");
    }

    fn case_value_cond(
        &mut self,
        value: &CaseValue,
        temp: &str,
        current_function: Option<&FunctionInfo>,
    ) -> String {
        match value {
            CaseValue::Single(expr) => {
                let (_, s) = self.expr(expr, current_function);
                format!("{temp} = {s}")
            }
            CaseValue::Range { from, to } => {
                let (_, from) = self.expr(from, current_function);
                let (_, to) = self.expr(to, current_function);
                format!("{temp} >= {from} AND {temp} <= {to}")
            }
            CaseValue::Is { op, value } => {
                let (_, val) = self.expr(value, current_function);
                format!("{temp} {} {val}", binary_op(*op))
            }
        }
    }

    fn expr(
        &mut self,
        node: &Expr,
        current_function: Option<&FunctionInfo>,
    ) -> (Vec<String>, String) {
        match node {
            Expr::Integer(value) => (Vec::new(), value.to_string()),
            Expr::Float(value) => (Vec::new(), value.to_string()),
            Expr::String(value) => (Vec::new(), format!("\"{}\"", escape_string(value))),
            Expr::Ident(ident) => (Vec::new(), self.ident(ident, current_function)),
            Expr::ArrayRef { name, indices } => {
                if let Some(info) = self.function_info(name).cloned() {
                    let call = self.call_lines(&info, indices, current_function);
                    return (call, info.result.as_basic());
                }

                let mut prelude = Vec::new();
                let mut rendered_indices = Vec::new();
                for index in indices {
                    let (index_prelude, index) = self.expr(index, current_function);
                    prelude.extend(index_prelude);
                    rendered_indices.push(index);
                }
                let base = if self.known_callables.contains(&name.name.to_ascii_lowercase()) {
                    self.canonical_callable(name)
                } else {
                    self.ident(name, current_function)
                };
                (prelude, format!("{}({})", base, rendered_indices.join(", ")))
            }
            Expr::Call { name, args } => {
                if let Some(info) = self.function_info(name).cloned() {
                    (
                        self.call_lines(&info, args, current_function),
                        info.result.as_basic(),
                    )
                } else {
                    let mut prelude = Vec::new();
                    let mut rendered_args = Vec::new();
                    for arg in args {
                        let (arg_prelude, arg) = self.expr(arg, current_function);
                        prelude.extend(arg_prelude);
                        rendered_args.push(arg);
                    }
                    let key = name.name.to_ascii_lowercase();
                    let emit_name = if self.known_callables.contains(&key) {
                        self.canonical_callable(name)
                    } else {
                        BasicIdent { name: key, suffix: name.suffix }.as_basic()
                    };
                    (prelude, format!("{}({})", emit_name, rendered_args.join(", ")))
                }
            }
            Expr::Unary { op, expr } => {
                let (prelude, inner) = self.expr(expr, current_function);
                let rendered = match op {
                    UnaryOp::Neg => format!("-{inner}"),
                    UnaryOp::Not => format!("NOT ({inner})"),
                };
                (prelude, rendered)
            }
            Expr::Binary { left, op, right } => {
                let (mut prelude, left_str) = self.expr(left, current_function);
                let (right_prelude, right_str) = self.expr(right, current_function);
                prelude.extend(right_prelude);
                let left_r = if matches!(left.as_ref(), Expr::Binary { .. }) {
                    format!("({left_str})")
                } else {
                    left_str
                };
                let right_r = if matches!(right.as_ref(), Expr::Binary { .. }) {
                    format!("({right_str})")
                } else {
                    right_str
                };
                (prelude, format!("{left_r} {} {right_r}", binary_op(*op)))
            }
        }
    }

    fn call_lines(
        &mut self,
        info: &FunctionInfo,
        args: &[Expr],
        current_function: Option<&FunctionInfo>,
    ) -> Vec<String> {
        let mut lines = Vec::new();
        let mut rendered_args = Vec::new();
        for arg in args {
            let (arg_prelude, rendered_arg) = self.expr(arg, current_function);
            lines.extend(arg_prelude);
            rendered_args.push(rendered_arg);
        }

        for (index, rendered_arg) in rendered_args.iter().enumerate() {
            if let Some((_, lowered)) = info.params.get(index) {
                if !is_empty_array_arg(args.get(index)) {
                    lines.push(format!("{} = {rendered_arg}", lowered.as_basic()));
                }
            } else {
                lines.push(format!(
                    "' warning: extra argument {} for {} ignored by current lowering",
                    index + 1,
                    info.source_name
                ));
            }
        }

        for (index, arg) in args.iter().enumerate() {
            if let Some((_, lowered)) = info.params.get(index) {
                if let Some(actual_array) = empty_array_name(arg, current_function, self) {
                    let bound = copy_bound(info, &rendered_args, index);
                    let loop_var = self.next_temp_var();
                    lines.push(format!("DIM {}({bound})", lowered.as_basic()));
                    lines.extend(array_copy_lines(
                        &lowered.as_basic(),
                        &actual_array,
                        &bound,
                        "copy array argument into lowered function storage",
                        &loop_var,
                    ));
                }
            }
        }

        lines.push(format!("GOSUB {}", info.label));

        for (index, arg) in args.iter().enumerate() {
            if let Some((_, lowered)) = info.params.get(index) {
                if let Some(actual_array) = empty_array_name(arg, current_function, self) {
                    let bound = copy_bound(info, &rendered_args, index);
                    let loop_var = self.next_temp_var();
                    lines.extend(array_copy_lines(
                        &actual_array,
                        &lowered.as_basic(),
                        &bound,
                        "copy mutated array argument back to caller storage",
                        &loop_var,
                    ));
                }
            }
        }

        lines
    }

    fn emit_call_statement(
        &mut self,
        info: &FunctionInfo,
        args: &[Expr],
        current_function: Option<&FunctionInfo>,
    ) {
        let lines = self.call_lines(info, args, current_function);
        self.lines(lines);
    }

    fn next_temp_var(&mut self) -> String {
        let id = self.next_label;
        self.next_label += 1;
        format!("BCC_T{id}%")
    }

    fn canonical_callable(&self, name: &BasicIdent) -> String {
        BasicIdent {
            name: name.name.to_ascii_uppercase(),
            suffix: name.suffix,
        }
        .as_basic()
    }

    fn ident(&self, ident: &BasicIdent, current_function: Option<&FunctionInfo>) -> String {
        if let Some(info) = current_function {
            if let Some((_, lowered)) = info
                .params
                .iter()
                .find(|(source, _)| same_ident(source, ident))
            {
                return lowered.as_basic();
            }
            let key = ident.as_basic().to_ascii_lowercase();
            if !info.globals.contains(&key) {
                return BasicIdent {
                    name: format!("{}_{}", info.stem, sanitize_symbol(&ident.name)),
                    suffix: ident.suffix,
                }
                .as_basic();
            }
        }
        BasicIdent { name: ident.name.to_ascii_lowercase(), suffix: ident.suffix }.as_basic()
    }

    fn function_info(&self, name: &BasicIdent) -> Option<&FunctionInfo> {
        self.functions
            .iter()
            .find(|function| same_ident(&function.source_name, name))
    }

    fn render_print_tokens(
        &mut self,
        tokens: &[PrintToken],
        current_function: Option<&FunctionInfo>,
    ) -> String {
        let mut out = String::new();
        // after_sep: push a space BEFORE the next Expr (readable: `; x%` not `;x%`)
        // Starts false so the very first Expr gets no leading space.
        let mut after_sep = false;
        for token in tokens {
            match token {
                PrintToken::Expr(e) => {
                    let (prelude, rendered) = self.expr(e, current_function);
                    self.lines(prelude);
                    if after_sep {
                        out.push(' ');
                    }
                    out.push_str(&rendered);
                    after_sep = false;
                }
                PrintToken::Semi => {
                    out.push(';');
                    after_sep = true;
                }
                PrintToken::Comma => {
                    out.push(',');
                    after_sep = true;
                }
            }
        }
        out
    }

    fn lines(&mut self, lines: Vec<String>) {
        for line in lines {
            self.line(&line);
        }
    }

    fn line(&mut self, line: &str) {
        for _ in 0..self.indent {
            self.output.push_str("    ");
        }
        self.output.push_str(line);
        self.output.push('\n');
    }

    fn blank(&mut self) {
        self.output.push('\n');
    }
}

impl FunctionInfo {
    fn from_def(function: &FunctionDef) -> Self {
        let stem = sanitize_symbol(&function.name.name);
        let params = function
            .params
            .iter()
            .map(|param| {
                (
                    param.clone(),
                    BasicIdent {
                        name: format!("{}_{}", stem, sanitize_symbol(&param.name)),
                        suffix: param.suffix,
                    },
                )
            })
            .collect();
        let globals = collect_globals(&function.body);
        Self {
            source_name: function.name.clone(),
            stem: stem.clone(),
            label: format!("FN_{stem}"),
            result: BasicIdent {
                name: format!("{stem}_result"),
                suffix: function.name.suffix,
            },
            params,
            is_procedure: function.is_procedure,
            globals,
        }
    }
}

fn collect_globals(body: &[Statement]) -> HashSet<String> {
    let mut globals = HashSet::new();
    for stmt in body {
        match stmt {
            Statement::GlobalDecl(ident) => {
                globals.insert(ident.as_basic().to_ascii_lowercase());
            }
            Statement::If { then_body, else_body, .. } => {
                globals.extend(collect_globals(then_body));
                globals.extend(collect_globals(else_body));
            }
            Statement::For { body, .. }
            | Statement::While { body, .. }
            | Statement::Do { body, .. } => {
                globals.extend(collect_globals(body));
            }
            Statement::SelectCase { cases, else_body, .. } => {
                for case in cases {
                    globals.extend(collect_globals(&case.body));
                }
                globals.extend(collect_globals(else_body));
            }
            _ => {}
        }
    }
    globals
}

fn same_ident(left: &BasicIdent, right: &BasicIdent) -> bool {
    left.suffix == right.suffix && left.name.eq_ignore_ascii_case(&right.name)
}

fn callable_expr(expr: &Expr) -> Option<(&BasicIdent, &[Expr])> {
    match expr {
        Expr::Call { name, args } => Some((name, args)),
        Expr::ArrayRef { name, indices } => Some((name, indices)),
        _ => None,
    }
}

fn is_empty_array_arg(expr: Option<&Expr>) -> bool {
    matches!(expr, Some(Expr::ArrayRef { indices, .. }) if indices.is_empty())
}

fn empty_array_name(
    expr: &Expr,
    current_function: Option<&FunctionInfo>,
    generator: &CodeGenerator,
) -> Option<String> {
    match expr {
        Expr::ArrayRef { name, indices } if indices.is_empty() => {
            Some(generator.ident(name, current_function))
        }
        _ => None,
    }
}

fn copy_bound(info: &FunctionInfo, rendered_args: &[String], array_arg_index: usize) -> String {
    rendered_args
        .get(array_arg_index + 1)
        .cloned()
        .or_else(|| {
            info.params
                .get(array_arg_index + 1)
                .map(|(_, lowered)| lowered.as_basic())
        })
        .unwrap_or_else(|| "10".to_string())
}

fn array_copy_lines(
    destination: &str,
    source: &str,
    bound: &str,
    comment: &str,
    loop_var: &str,
) -> Vec<String> {
    vec![
        String::new(),
        format!("' {comment}: {source}() -> {destination}()"),
        format!("FOR {loop_var} = 1 TO {bound}"),
        format!("    {destination}({loop_var}) = {source}({loop_var})"),
        format!("NEXT {loop_var}"),
        String::new(),
    ]
}

fn sanitize_symbol(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

fn binary_op(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "+",
        BinaryOp::Sub => "-",
        BinaryOp::Mul => "*",
        BinaryOp::Div => "/",
        BinaryOp::Eq => "=",
        BinaryOp::Ne => "<>",
        BinaryOp::Lt => "<",
        BinaryOp::Le => "<=",
        BinaryOp::Gt => ">",
        BinaryOp::Ge => ">=",
        BinaryOp::And => "AND",
        BinaryOp::Or => "OR",
        BinaryOp::Xor => "XOR",
        BinaryOp::Mod => "MOD",
        BinaryOp::IntDiv => "\\",
        BinaryOp::Pow => "^",
    }
}

fn escape_string(value: &str) -> String {
    value.replace('"', "\"\"")
}

fn ends_with_end(statements: &[Statement]) -> bool {
    statements
        .iter()
        .rev()
        .find(|s| !matches!(s, Statement::BlankLine))
        .is_some_and(|s| matches!(s, Statement::End))
}

fn ends_with_return(statements: &[Statement]) -> bool {
    statements
        .iter()
        .rev()
        .find(|s| !matches!(s, Statement::BlankLine))
        .is_some_and(|s| matches!(s, Statement::Return { .. } | Statement::ReturnVoid))
}

fn number_basic_lines(source: &str, full: bool) -> String {
    let lines = source.lines().collect::<Vec<_>>();

    // Lines that survive into the output (non-blank, non-label-only)
    let emitted: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty() && is_label_line(line).is_none())
        .map(|(i, _)| i)
        .collect();

    // In full mode every emitted line is a target; in sparse mode only lines
    // that are actually jumped to receive a number.
    let target_indices: std::collections::HashSet<usize> = if full {
        emitted.iter().copied().collect()
    } else {
        lines
            .iter()
            .enumerate()
            .filter_map(|(index, line)| {
                is_label_line(line)?;
                next_emitted_line_index(&lines, index + 1)
            })
            .collect()
    };

    // Assign sequential line numbers (step 10) to target lines in source order
    let mut index_to_number: HashMap<usize, usize> = HashMap::new();
    let mut current_number = 10usize;
    for &index in &emitted {
        if target_indices.contains(&index) {
            index_to_number.insert(index, current_number);
            current_number += 10;
        }
    }

    // Map each label name to the line number of the first emitted line after it
    let label_numbers: HashMap<String, usize> = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| {
            let label = is_label_line(line)?;
            let target = next_emitted_line_index(&lines, index + 1)?;
            let number = *index_to_number.get(&target)?;
            Some((label.to_string(), number))
        })
        .collect();

    // Walk every intermediate line in order so blank lines pass through.
    // Label-only lines are dropped; everything else is emitted.
    // Consecutive blank lines are folded into a single blank.
    let mut output = String::new();
    let mut last_was_blank = false;
    for (index, &raw) in lines.iter().enumerate() {
        if is_label_line(raw).is_some() {
            continue;
        }
        if raw.trim().is_empty() {
            if !last_was_blank {
                output.push('\n');
                last_was_blank = true;
            }
            continue;
        }
        last_was_blank = false;
        // Sparse-mode target lines are GOTO entry points: trim to column-0 after the
        // number.  Every other line keeps the structural indentation that codegen built
        // up via self.indent, so IF/WHILE/FOR bodies stay visually nested.
        let mut text = if index_to_number.contains_key(&index) && !full {
            raw.trim().to_string()
        } else {
            raw.to_string()
        };
        for (label, number) in &label_numbers {
            text = text.replace(label.as_str(), &number.to_string());
        }
        if let Some(&number) = index_to_number.get(&index) {
            output.push_str(&format!("{number} {text}\n"));
        } else {
            output.push_str(&format!("{text}\n"));
        }
    }
    output
}

fn next_emitted_line_index(lines: &[&str], start: usize) -> Option<usize> {
    for (index, line) in lines.iter().enumerate().skip(start) {
        if is_label_line(line).is_some() || line.trim().is_empty() {
            continue;
        }
        return Some(index);
    }
    None
}

fn expr_type_suffix(expr: &Expr) -> &'static str {
    match expr {
        Expr::String(_) => "$",
        Expr::Integer(_) => "%",
        Expr::Float(_) => "!",
        Expr::Ident(ident) | Expr::Call { name: ident, .. } | Expr::ArrayRef { name: ident, .. } => {
            match ident.suffix {
                Some(TypeSuffix::String) => "$",
                Some(TypeSuffix::Single) => "!",
                Some(TypeSuffix::Double) => "#",
                Some(TypeSuffix::Long) => "&",
                _ => "%",
            }
        }
        Expr::Unary { expr, .. } => expr_type_suffix(expr),
        Expr::Binary { left, .. } => expr_type_suffix(left),
    }
}

fn is_label_line(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    let label = trimmed.strip_suffix(':')?;
    if label.starts_with("IF_")
        || label.starts_with("FN_")
        || label.starts_with("WHILE_")
        || label.starts_with("DO_")
        || label.starts_with("SEL_")
    {
        Some(label)
    } else {
        None
    }
}
