use std::collections::HashMap;

use crate::ast::*;

pub struct CodeGenerator {
    next_label: usize,
    indent: usize,
    output: String,
    functions: Vec<FunctionInfo>,
    line_numbers: bool,
}

#[derive(Debug, Clone)]
struct FunctionInfo {
    source_name: BasicIdent,
    label: String,
    result: BasicIdent,
    params: Vec<(BasicIdent, BasicIdent)>,
}

impl CodeGenerator {
    pub fn new() -> Self {
        Self {
            next_label: 1,
            indent: 0,
            output: String::new(),
            functions: Vec::new(),
            line_numbers: false,
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

        self.line("' BASCAL generated BASIC");
        self.line("' Functions are lowered to global variables, labels, and GOSUB");
        if !program.declarations.is_empty() {
            self.line("' TODO: resolve BASCAL dependency selectors during link");
            for declaration in &program.declarations {
                match declaration {
                    DependencyDecl::Include(path) => {
                        self.line(&format!("' include \"{}\"", escape_string(path)))
                    }
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
            .map(|p| p.as_basic())
            .collect::<Vec<_>>()
            .join(", ");
        self.blank();
        self.line(&format!("' function {}({})", function.name, params));
        self.line(&format!("{}:", info.label));
        self.indent += 1;
        self.statements(&function.body, Some(&info));
        if !ends_with_return(&function.body) {
            self.line("RETURN");
        }
        self.indent -= 1;
        self.line(&format!("' end function {}", function.name));
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
            Statement::Assignment { target, value } => {
                let (target_prelude, target) = self.expr(target, current_function);
                let (value_prelude, value) = self.expr(value, current_function);
                self.lines(target_prelude);
                self.lines(value_prelude);
                self.line(&format!("{target} = {value}"));
            }
            Statement::Print { exprs } => {
                let mut rendered = Vec::new();
                for item in exprs {
                    let (prelude, item) = self.expr(item, current_function);
                    self.lines(prelude);
                    rendered.push(item);
                }
                if rendered.is_empty() {
                    self.line("PRINT");
                } else {
                    self.line(&format!("PRINT {}", rendered.join(", ")));
                }
            }
            Statement::Return { value } => {
                let Some(info) = current_function else {
                    let (prelude, value) = self.expr(value, current_function);
                    self.lines(prelude);
                    self.line(&format!("RETURN {}", value));
                    return;
                };
                let (prelude, value) = self.expr(value, current_function);
                self.lines(prelude);
                self.line(&format!("{} = {}", info.result.as_basic(), value));
                self.line("RETURN");
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
                self.statements(body, current_function);
                self.line(&format!("GOTO {top_label}"));
                self.indent -= 1;
                self.line(&format!("{end_label}:"));
                self.line("REM END WHILE");
            }
            Statement::ExprStmt(expr_stmt) => self.expr_statement(expr_stmt, current_function),
            Statement::End => self.line("END"),
            Statement::Raw(raw) => self.line(raw),
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

    fn expr(
        &mut self,
        node: &Expr,
        current_function: Option<&FunctionInfo>,
    ) -> (Vec<String>, String) {
        match node {
            Expr::Integer(value) => (Vec::new(), value.to_string()),
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
                (
                    prelude,
                    format!(
                        "{}({})",
                        self.ident(name, current_function),
                        rendered_indices.join(", ")
                    ),
                )
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
                    (prelude, format!("{}({})", name, rendered_args.join(", ")))
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

    fn ident(&self, ident: &BasicIdent, current_function: Option<&FunctionInfo>) -> String {
        if let Some(info) = current_function {
            if let Some((_, lowered)) = info
                .params
                .iter()
                .find(|(source, _)| same_ident(source, ident))
            {
                return lowered.as_basic();
            }
        }
        ident.as_basic()
    }

    fn function_info(&self, name: &BasicIdent) -> Option<&FunctionInfo> {
        self.functions
            .iter()
            .find(|function| same_ident(&function.source_name, name))
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
        Self {
            source_name: function.name.clone(),
            label: format!("FN_{stem}"),
            result: BasicIdent {
                name: format!("{stem}_result"),
                suffix: function.name.suffix,
            },
            params,
        }
    }
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
        .is_some_and(|s| matches!(s, Statement::Return { .. }))
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

fn is_label_line(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    let label = trimmed.strip_suffix(':')?;
    if label.starts_with("IF_") || label.starts_with("FN_") || label.starts_with("WHILE_") {
        Some(label)
    } else {
        None
    }
}
