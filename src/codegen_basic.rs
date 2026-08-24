use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use crate::ast::*;
use crate::diagnostics::{Diagnostic, SourcePos};

// "ucase", "lcase", "ltrim", and "rtrim" are deliberately absent here: real
// MBASIC/BASCOM 2.00 has none of them (verified against a real IBM BASIC
// Compiler 2.00 under dosbox-x -- see com/bascal/stdlib/), so treating them
// as safe passthrough builtins the target dialect provides would be wrong.
// BASCAL provides its own implementations instead, as an ordinary
// require-able library; see `lib::stdlib_search_roots`.
pub(crate) const BASIC_BUILTINS: &[&str] = &[
    // Type-suffixed single-arg — parser creates Expr::ArrayRef for these
    "str", "chr", "hex", "oct", "space", "environ", "command", "trim",
    // Multi-arg string (Expr::Call, but include for completeness)
    "left", "right", "mid", "instr", "format", "string", "input",
    // Single-arg numeric (no suffix → Expr::Call already, but included for safety)
    "len", "val", "asc", "sqr", "abs", "int", "fix", "sgn", "rnd", "eof", "sin", "cos", "tan",
    "atn", "log", "exp", "cint", "clng", "csng", "cdbl", "peek", "inp", "lof", "loc", "pos",
    "csrlin", "freefile", "fre", "lpos", "varptr", "date", "time", "timer", "inkey", "err", "erl",
    // Print-position helpers (used inside PRINT)
    "tab", "spc", // Multi-arg numeric
    "ubound", "lbound", "iif", // Random-access record packing/unpacking
    "mki", "mkl", "mks", "mkd", "cvi", "cvl", "cvs", "cvd",
];

/// What a bare `exit` resolves to, tracked per enclosing loop. `for`/`next`
/// compiles to a native BASIC `FOR ... NEXT` block, so leaving it is just
/// BASIC's own `EXIT FOR` -- no label involved, unlike `while`/`do`, which
/// transpile to a GOTO chain and so need a real jump target.
#[derive(Debug, Clone)]
enum LoopExit {
    NativeFor,
    Goto(String),
}

pub struct CodeGenerator {
    next_label: usize,
    indent: usize,
    output: String,
    functions: Vec<FunctionInfo>,
    known_callables: HashSet<String>,
    line_numbers: bool,
    loop_exit_stack: Vec<LoopExit>,
    // All BASIC names already claimed: global vars + every allocated param/result/local name.
    // RefCell because ident() must read and extend this set through a shared &self reference.
    taken_names: RefCell<HashSet<String>>,
    // Lowercase BASIC names of every record/file FIELD buffer variable
    // (from `records::lower`'s `Statement::Field`). These are structurally
    // global -- there is exactly one FIELD-bound buffer per record field,
    // shared by every function/procedure that touches that file -- so
    // ident() must never allocate a per-function local for one, regardless
    // of which scope the LSET/GET/PUT referencing it appears in.
    record_buffer_names: HashSet<String>,
    // Lowercase BASIC names of every top-level `const` declaration. `const`
    // values are compile-time literals, never reassignable, so unlike an
    // ordinary global variable there's no scoping/shadowing concern that
    // would justify requiring an explicit `global` declaration to see one
    // from inside a function/procedure body -- they should always resolve
    // to the real top-level name, the same way record_buffer_names does.
    const_names: HashSet<String>,
    // Lowercase BASIC names of the *subset* of `record_buffer_names` that
    // `records::lower` invented itself (via `buffer_ident`), as opposed to
    // a `FIELD` buffer name the author typed directly in raw-BASIC-
    // passthrough source. A synthesized name is already deliberately
    // camelCased and keeps that case in `ident()`; an author-typed one
    // still gets BASCAL's normal lowercase normalization, same as any
    // other identifier.
    synthesized_buffer_names: HashSet<String>,
    // Errors found while generating (e.g. an invalid `byref` call argument).
    // Collected rather than returned immediately since codegen methods are
    // called deep inside statement/expression recursion.
    diagnostics: Vec<Diagnostic>,
    // Declared rank (number of DIM dimensions) of every top-level array,
    // lowercase name -> rank. Used to check a call site's array argument
    // against the callee's own inferred parameter rank.
    top_level_array_ranks: HashMap<String, usize>,
    // Frozen per-axis bound text for every top-level array -- a literal
    // directly, or a generated temp's name if the bound wasn't already a
    // compile-time constant. Populated as each DIM is actually generated.
    // Every array needs this available, not just ones `sizeof()` is called
    // on directly: any array can be passed to a function, and the transpiler
    // auto-injects its bounds at the call site so the callee's own
    // `sizeof()` on that parameter has something to read.
    top_level_array_bounds: HashMap<String, Vec<String>>,
    // Lowercase names of every procedure named as an `on error goto` target
    // somewhere in the program. resolver::validate has already proven each
    // one contains no `return` and never falls off the end (every path
    // ends in resume/goto/end) -- so unlike an ordinary procedure, codegen
    // must NOT append an implicit trailing RETURN for one of these: it's
    // been proven unreachable, and appending it anyway would reintroduce
    // exactly the "RETURN with no GOSUB frame" crash risk the proof exists
    // to rule out.
    error_handler_procedures: HashSet<String>,
}

#[derive(Debug, Clone)]
struct FunctionInfo {
    source_name: BasicIdent,
    stem: String,
    label: String,
    result: BasicIdent,
    /// (source parameter, allocated lowered BASIC name) pairs, in declared order.
    params: Vec<(Param, BasicIdent)>,
    /// Array rank inferred per parameter from how it's indexed inside this
    /// function's own body (`None` if never directly indexed, or indexed
    /// inconsistently). Parallel to `params`.
    param_ranks: Vec<Option<usize>>,
    /// For each array parameter (rank `Some(n)` in `param_ranks`), the `n`
    /// transpiler-synthesized BASIC variable names that carry its per-axis
    /// bounds -- never written by the `.bcl` author, never appearing at a
    /// call site. The caller sets them (from the actual argument array's
    /// own resolved bounds) immediately before `GOSUB`, alongside the
    /// ordinary copy-in; the callee's body reads them back through
    /// `sizeof()`. Empty `Vec` for a scalar parameter. Parallel to `params`.
    param_bound_vars: Vec<Vec<String>>,
    /// Fixed storage capacity per axis for each array parameter -- how big
    /// the shared storage array named by `param_bound_vars`'s sibling
    /// lowered name actually gets `DIM`ed, once, at top-level. Resolved by
    /// `infer_array_param_capacities` before any `FunctionInfo` is built:
    /// either the largest array ever passed to this parameter across every
    /// call site in the program (when every axis is declared `?`), or an
    /// explicit literal the author wrote instead. Distinct from
    /// `param_bound_vars`, which tracks each individual call's *actual*
    /// size -- capacity is the fixed ceiling that actual size is checked
    /// against at runtime before every call. Empty `Vec` for a scalar
    /// parameter. Parallel to `params`.
    param_capacities: Vec<Vec<i64>>,
    /// Declared rank of every array this function DIMs locally, lowercase
    /// name -> rank.
    local_array_ranks: HashMap<String, usize>,
    /// Frozen per-axis bound text for every array DIMed locally within this
    /// function. RefCell because it's populated lazily as DIM statements
    /// are generated, through a shared `&FunctionInfo` reference -- same
    /// pattern as `local_var_map`.
    local_array_bounds: RefCell<HashMap<String, Vec<String>>>,
    is_procedure: bool,
    receiver: Option<TypeSuffix>,
    globals: HashSet<String>,
    // Cache of source-variable-key → allocated lowered BASIC name for locals in this function.
    // RefCell because ident() populates this lazily through a shared &FunctionInfo reference.
    local_var_map: RefCell<HashMap<String, String>>,
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
            taken_names: RefCell::new(HashSet::new()),
            record_buffer_names: HashSet::new(),
            const_names: HashSet::new(),
            synthesized_buffer_names: HashSet::new(),
            diagnostics: Vec::new(),
            top_level_array_ranks: HashMap::new(),
            top_level_array_bounds: HashMap::new(),
            error_handler_procedures: HashSet::new(),
        }
    }

    pub fn with_synthesized_buffer_names(mut self, value: HashSet<String>) -> Self {
        self.synthesized_buffer_names = value;
        self
    }

    pub fn with_line_numbers(mut self, value: bool) -> Self {
        self.line_numbers = value;
        self
    }

    pub fn generate(mut self, program: &Program) -> Result<String, Vec<Diagnostic>> {
        // Seed the name registry with every variable visible at global scope.
        // Function params/results are registered as each FunctionInfo is built so
        // later functions cannot collide with earlier ones either.
        let mut taken = collect_program_names(program);
        let known_callables: HashSet<String> = program
            .functions
            .iter()
            .map(|f| f.name.name.to_ascii_lowercase())
            .chain(BASIC_BUILTINS.iter().map(|s| s.to_string()))
            .collect();
        let param_capacities = infer_array_param_capacities(program, &mut self.diagnostics);
        let mut functions = Vec::new();
        for f in &program.functions {
            let capacities = param_capacities
                .get(&f.name.name.to_ascii_lowercase())
                .cloned()
                .unwrap_or_default();
            functions.push(FunctionInfo::from_def(
                f,
                &mut taken,
                &known_callables,
                &mut self.diagnostics,
                capacities,
            ));
        }
        self.functions = functions;
        self.error_handler_procedures = crate::resolver::error_handler_targets(program)
            .iter()
            .map(|ident| ident.name.to_ascii_lowercase())
            .collect();
        self.top_level_array_ranks = dim_ranks_in_body(&program.statements);
        self.record_buffer_names = collect_record_buffer_names(program);
        {
            let mut consts = HashMap::new();
            collect_consts(&program.statements, &mut consts);
            self.const_names = consts.keys().map(|k| k.to_ascii_lowercase()).collect();
        }
        *self.taken_names.borrow_mut() = taken;

        self.known_callables = self
            .functions
            .iter()
            .map(|f| f.source_name.name.to_ascii_lowercase())
            .chain(BASIC_BUILTINS.iter().map(|s| s.to_string()))
            .collect();

        self.line("' BASCAL generated BASIC");
        self.line("' Functions are transpiled to global variables, labels, and GOSUB");

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

        self.emit_array_param_storage_dims();

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
        if self.diagnostics.is_empty() {
            Ok(number_basic_lines(&self.output, self.line_numbers))
        } else {
            Err(self.diagnostics)
        }
    }

    /// Emits the one-time, top-level `DIM` for every array parameter's
    /// shared storage, sized to its resolved capacity (see
    /// `infer_array_param_capacities`). Must run exactly once per program,
    /// before any call -- classic BASIC has no `REDIM`, so this is the
    /// only `DIM` these storage arrays ever get; `call_lines` no longer
    /// DIMs them per call site.
    fn emit_array_param_storage_dims(&mut self) {
        let lines: Vec<String> = self
            .functions
            .iter()
            .flat_map(|info| {
                info.params
                    .iter()
                    .enumerate()
                    .filter_map(move |(index, (param, lowered))| {
                        param.axes.as_ref()?;
                        let capacities = info.param_capacities.get(index)?;
                        if capacities.is_empty() {
                            return None;
                        }
                        let bounds = capacities
                            .iter()
                            .map(|c| c.to_string())
                            .collect::<Vec<_>>()
                            .join(", ");
                        Some(format!("DIM {}({bounds})", lowered.as_basic()))
                    })
            })
            .collect();
        if lines.is_empty() {
            return;
        }
        self.blank();
        self.line("' Storage for array parameters, sized to fit every call site");
        self.lines(lines);
    }

    fn function(&mut self, function: &FunctionDef) {
        let info = self
            .function_info(&function.name)
            .expect("function table should contain every function")
            .clone();
        let params = function
            .params
            .iter()
            .map(|p| {
                BasicIdent {
                    name: p.name.name.to_ascii_lowercase(),
                    suffix: p.name.suffix,
                }
                .as_basic()
            })
            .collect::<Vec<_>>()
            .join(", ");
        let lowered_name = BasicIdent {
            name: function.name.name.to_ascii_lowercase(),
            suffix: function.name.suffix,
        }
        .as_basic();
        let kind = if function.is_procedure {
            "procedure"
        } else {
            "function"
        };
        self.blank();
        self.line(&format!("' {kind} {}({})", lowered_name, params));
        self.line(&format!("{}:", info.label));
        self.indent += 1;
        self.statements(&function.body, Some(&info));
        // A procedure named as an `on error goto` target is entered via a
        // raw GOTO, never a GOSUB -- resolver::validate has already proven
        // it contains no `return` and never falls off the end, so the
        // usual implicit trailing RETURN below would be both unreachable
        // and, if that proof were ever wrong, a "RETURN without GOSUB"
        // crash. Skip it entirely for these, same as a raw label.
        let is_unreturnable_error_handler = function.is_procedure
            && self
                .error_handler_procedures
                .contains(&function.name.name.to_ascii_lowercase());
        if !ends_with_return(&function.body) && !is_unreturnable_error_handler {
            self.line("RETURN");
        }
        self.indent -= 1;
        self.line(&format!("' end {kind} {}", lowered_name));
    }

    fn statements(&mut self, statements: &[Stmt], current_function: Option<&FunctionInfo>) {
        for statement in statements {
            self.statement(statement, current_function);
        }
    }

    fn statement(&mut self, statement: &Stmt, current_function: Option<&FunctionInfo>) {
        match &statement.kind {
            Statement::Dim {
                name,
                is_array,
                sizes,
            } => {
                let base = self.ident(name, current_function);
                if sizes.is_empty() {
                    if *is_array {
                        self.line(&format!("DIM {base}()"));
                    } else {
                        self.line(&format!("DIM {base}"));
                    }
                } else {
                    let mut rendered = Vec::new();
                    for s in sizes {
                        let (prelude, val) = self.expr(s, current_function);
                        self.lines(prelude);
                        rendered.push(val);
                    }
                    self.line(&format!("DIM {base}({})", rendered.join(", ")));

                    // Every DIMed array's bounds get frozen here -- a
                    // literal bound needs no extra line, but a non-literal
                    // one is captured into a temp so it's still readable
                    // later (by `sizeof()`, or by the auto-injected bound
                    // passed to a function this array gets passed into).
                    let key = name.as_basic().to_ascii_lowercase();
                    let frozen: Vec<String> = sizes
                        .iter()
                        .zip(rendered.iter())
                        .map(|(s, val)| {
                            if matches!(s, Expr::Integer(_)) {
                                val.clone()
                            } else {
                                let temp = self.next_temp_var();
                                self.line(&format!("{temp} = {val}"));
                                temp
                            }
                        })
                        .collect();
                    if let Some(info) = current_function {
                        info.local_array_bounds.borrow_mut().insert(key, frozen);
                    } else {
                        self.top_level_array_bounds.insert(key, frozen);
                    }
                }
            }
            Statement::Open {
                mode,
                file,
                channel,
                len,
            } => {
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
                self.line(&format!(
                    "OPEN {file} FOR {mode_str} AS #{channel}{len_clause}"
                ));
            }
            Statement::FileDecl { .. } => {
                unreachable!("record/file DSL must be lowered before codegen")
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
            Statement::MidAssign {
                target,
                start,
                len,
                value,
            } => {
                // Left-to-right evaluation order, matching how the
                // statement reads: target, then start, then len (if
                // written), then value last -- same order `Statement::
                // Assignment` flushes target before value. Each operand is
                // evaluated exactly once and its *rendered text* reused
                // below (for the call, and again for the final assignment)
                // rather than re-running `self.expr` on the original
                // `target` a second time -- `target` may be an array
                // element whose index has side effects.
                let (target_prelude, target_text) = self.expr(target, current_function);
                let (start_prelude, start_text) = self.expr(start, current_function);
                let len_rendered = len.as_ref().map(|e| self.expr(e, current_function));
                let (value_prelude, value_text) = self.expr(value, current_function);
                self.lines(target_prelude);
                self.lines(start_prelude);
                let len_text = match len_rendered {
                    Some((len_prelude, len_text)) => {
                        self.lines(len_prelude);
                        len_text
                    }
                    // Two-argument form (`MID$(a$, start) = value`): real
                    // MBASIC/BASCOM behaves as if `len` were `LEN(value)`.
                    None => format!("LEN({value_text})"),
                };
                self.lines(value_prelude);

                // Transpiled into an ordinary call to com.bascal.stdlib.
                // midAssign, auto-injected into the program (see
                // `lib::inject_mid_assign_helper_if_used`) whenever this
                // statement is used anywhere -- the same GOSUB-based
                // call/return machinery every other function call already
                // goes through, just fed pre-rendered argument text instead
                // of re-evaluating `target`/`start`/`len`/`value` a second
                // time (see `call_lines_from_rendered_scalars`).
                let info = self
                    .function_info(&mid_assign_helper_ident())
                    .cloned()
                    .unwrap_or_else(|| {
                        panic!(
                            "BASCAL bug: com.bascal.stdlib.midAssign should always be \
                         auto-injected into the program whenever MID$ assignment \
                         syntax is used"
                        )
                    });
                let call = self.call_lines_from_rendered_scalars(
                    &info,
                    &[target_text.clone(), start_text, len_text, value_text],
                );
                self.lines(call);
                self.line(&format!("{target_text} = {}", info.result.as_basic()));
            }
            Statement::Print { tokens } => {
                let body = self.render_print_tokens(tokens, current_function);
                if body.is_empty() {
                    self.line("PRINT");
                } else {
                    self.line(&format!("PRINT {body}"));
                }
            }
            Statement::PrintUsing { format, tokens } => {
                let (fmt_pre, fmt_str) = self.expr(format, current_function);
                self.lines(fmt_pre);
                let body = self.render_print_tokens(tokens, current_function);
                if body.is_empty() {
                    self.line(&format!("PRINT USING {fmt_str}"));
                } else {
                    self.line(&format!("PRINT USING {fmt_str}; {body}"));
                }
            }
            Statement::PrintFileUsing {
                channel,
                format,
                tokens,
            } => {
                let (ch_pre, ch) = self.expr(channel, current_function);
                let (fmt_pre, fmt_str) = self.expr(format, current_function);
                self.lines(ch_pre);
                self.lines(fmt_pre);
                let body = self.render_print_tokens(tokens, current_function);
                if body.is_empty() {
                    self.line(&format!("PRINT #{ch}, USING {fmt_str}"));
                } else {
                    self.line(&format!("PRINT #{ch}, USING {fmt_str}; {body}"));
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
                self.loop_exit_stack.push(LoopExit::NativeFor);
                self.statements(body, current_function);
                self.loop_exit_stack.pop();
                self.indent -= 1;
                self.line(&format!("NEXT {}", self.ident(var, current_function)));
            }
            Statement::While { condition, body } => {
                let id = self.next_label;
                self.next_label += 1;
                let top_label = format!("WHILE_{id:04}_TOP");
                let end_label = format!("WHILE_{id:04}_END");
                self.line(&format!("{top_label}:"));
                self.condition_jump(condition, &end_label, false, current_function);
                self.indent += 1;
                self.loop_exit_stack.push(LoopExit::Goto(end_label.clone()));
                self.statements(body, current_function);
                self.loop_exit_stack.pop();
                self.line(&format!("GOTO {top_label}"));
                self.indent -= 1;
                self.line(&format!("{end_label}:"));
                self.line("REM END WHILE");
            }
            Statement::Do {
                condition,
                body,
                post_condition,
            } => {
                let id = self.next_label;
                self.next_label += 1;
                let top_label = format!("DO_{id:04}_TOP");
                let end_label = format!("DO_{id:04}_END");
                self.line(&format!("{top_label}:"));
                if let Some(cond) = condition {
                    // is_while -> exit when false (invert=false); is_until -> exit when true (invert=true).
                    self.condition_jump(&cond.expr, &end_label, !cond.is_while, current_function);
                }
                self.indent += 1;
                self.loop_exit_stack.push(LoopExit::Goto(end_label.clone()));
                self.statements(body, current_function);
                self.loop_exit_stack.pop();
                if let Some(cond) = post_condition {
                    // is_while -> repeat when true (invert=true); is_until -> repeat when false (invert=false).
                    self.condition_jump(&cond.expr, &top_label, cond.is_while, current_function);
                } else {
                    // No post-condition: loop back to re-check the pre-condition (if
                    // any) or repeat unconditionally (bare `do ... end do`, relying on
                    // `exit do` to leave). Previously this only fired for a bare
                    // `do`, so a pre-condition-only `do while`/`do until` loop never
                    // actually looped — it ran the body at most once and fell
                    // through, regardless of the condition.
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
            Statement::OptionBase(expr) => {
                let (prelude, base) = self.expr(expr, current_function);
                self.lines(prelude);
                self.line(&format!("OPTION BASE {base}"));
            }
            Statement::Erase(vars) => {
                let names: Vec<String> = vars
                    .iter()
                    .map(|v| self.ident(v, current_function))
                    .collect();
                self.line(&format!("ERASE {}", names.join(", ")));
            }
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
            Statement::Poke { address, value } => {
                let (addr_prelude, addr) = self.expr(address, current_function);
                let (val_prelude, val) = self.expr(value, current_function);
                self.lines(addr_prelude);
                self.lines(val_prelude);
                self.line(&format!("POKE {addr}, {val}"));
            }
            Statement::Out { port, value } => {
                let (port_prelude, port) = self.expr(port, current_function);
                let (val_prelude, val) = self.expr(value, current_function);
                self.lines(port_prelude);
                self.lines(val_prelude);
                self.line(&format!("OUT {port}, {val}"));
            }
            Statement::Width { channel, cols } => {
                let (cols_prelude, cols_s) = self.expr(cols, current_function);
                self.lines(cols_prelude);
                match channel {
                    Some(ch) => {
                        let (ch_prelude, ch_s) = self.expr(ch, current_function);
                        self.lines(ch_prelude);
                        self.line(&format!("WIDTH #{ch_s}, {cols_s}"));
                    }
                    None => self.line(&format!("WIDTH {cols_s}")),
                }
            }
            Statement::Clear => {
                self.line("CLEAR");
            }
            Statement::Label(name) => {
                self.line(&format!("{name}:"));
            }
            Statement::Goto(target) => {
                self.line(&format!("GOTO {}", self.label_target_text(target)));
            }
            Statement::Gosub(target) => {
                self.line(&format!("GOSUB {}", self.label_target_text(target)));
            }
            Statement::OnErrorGoto { target } => {
                self.line(&format!("ON ERROR GOTO {}", self.label_target_text(target)));
            }
            Statement::Resume(kind) => match kind {
                ResumeTarget::Same => self.line("RESUME"),
                ResumeTarget::Next => self.line("RESUME NEXT"),
                ResumeTarget::Line(expr) => {
                    self.line(&format!("RESUME {}", self.label_target_text(expr)));
                }
            },
            Statement::ErrorStmt { code } => {
                let (prelude, code) = self.expr(code, current_function);
                self.lines(prelude);
                self.line(&format!("ERROR {code}"));
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
                    self.line(&format!("RESTORE {}", self.label_target_text(target)));
                } else {
                    self.line("RESTORE");
                }
            }
            Statement::Const { name, value } => {
                // Real MBASIC/BASCOM has no CONST statement at all (a
                // QuickBASIC-era addition) -- a plain assignment is the
                // only thing that compiles there. `const` in .bcl source
                // is purely a naming/intent signal to the reader; nothing
                // in generated BASIC needs to express "this shouldn't be
                // reassigned" for it to behave correctly.
                let (prelude, value) = self.expr(value, current_function);
                self.lines(prelude);
                self.line(&format!("{} = {value}", self.ident(name, current_function)));
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
            Statement::Field {
                channel, fields, ..
            } => {
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
            Statement::Get {
                channel,
                record,
                var,
                require_existing,
                record_length,
            } => {
                let (ch_pre, ch) = self.expr(channel, current_function);
                self.lines(ch_pre);
                match (record, var) {
                    (None, None) => self.line(&format!("GET #{ch}")),
                    (Some(rec), None) => {
                        let (r_pre, r) = self.expr(rec, current_function);
                        self.lines(r_pre);
                        if *require_existing {
                            let length = record_length
                                .expect("DSL partial update must carry its record length");
                            self.line(&format!("IF LOF(#{ch}) < ({r}) * {length} THEN ERROR 63"));
                        }
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
            Statement::Put {
                channel,
                record,
                var,
                ..
            } => {
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
            Statement::LprintUsing { format, tokens } => {
                let (fmt_pre, fmt_str) = self.expr(format, current_function);
                self.lines(fmt_pre);
                let body = self.render_print_tokens(tokens, current_function);
                if body.is_empty() {
                    self.line(&format!("LPRINT USING {fmt_str}"));
                } else {
                    self.line(&format!("LPRINT USING {fmt_str}; {body}"));
                }
            }
            Statement::Exit => match self.loop_exit_stack.last() {
                Some(LoopExit::NativeFor) => self.line("EXIT FOR"),
                Some(LoopExit::Goto(label)) => {
                    let label = label.clone();
                    self.line(&format!("GOTO {label}"));
                }
                None => self.line("' warning: EXIT outside of a loop"),
            },
            Statement::SelectCase {
                expr,
                cases,
                else_body,
            } => {
                self.select_case(expr, cases, else_body, current_function);
            }
            Statement::TryCatch {
                try_body,
                catch,
                finally_body,
            } => {
                self.try_catch(try_body, catch.as_ref(), finally_body, current_function);
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
            Statement::OnBranch {
                expr,
                targets,
                is_gosub,
            } => {
                let (prelude, expr) = self.expr(expr, current_function);
                self.lines(prelude);
                let rendered: Vec<String> =
                    targets.iter().map(|t| self.label_target_text(t)).collect();
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
        if let Expr::ScalarMethodCall { base, method, args } = expr_stmt {
            if let Some(receiver) = self.expr_receiver_type(base) {
                if let Some(info) = self.method_info(receiver, method).cloned() {
                    let mut call_args = Vec::with_capacity(args.len() + 1);
                    call_args.push((**base).clone());
                    call_args.extend(args.iter().cloned());
                    let lines = self.call_lines(&info, &call_args, current_function);
                    self.lines(lines);
                    return;
                }
            }
        }
        if let Some((name, args)) = callable_expr(expr_stmt) {
            if let Some(info) = self.ordinary_function_info(name).cloned() {
                self.emit_call_statement(&info, args, current_function);
                return;
            }
        }

        let (prelude, expr_stmt) = self.expr(expr_stmt, current_function);
        self.lines(prelude);
        self.line(&expr_stmt);
    }

    /// Emits guarded-jump code for `condition`: jumps to `target` when the
    /// condition is false (`invert = false`) or true (`invert = true`),
    /// otherwise falls through to whatever is emitted next.
    ///
    /// Detects a top-level `&&`/`||` chain (only ever produced by
    /// `Parser::parse_condition`) and emits one short-circuit guard line per
    /// operand instead of rendering the whole condition as a single BASIC
    /// expression. Each operand's own prelude (e.g. a `GOSUB` for a function
    /// call) is emitted immediately before that operand's guard line, not
    /// hoisted to the top — this is what makes a later operand's side
    /// effects genuinely not run once an earlier operand already decided
    /// the outcome, the actual point of the feature.
    ///
    /// Any condition that isn't a chain falls back to exactly the
    /// single-`IF` behavior this replaced, so every existing condition is
    /// byte-for-byte unchanged.
    fn condition_jump(
        &mut self,
        condition: &Expr,
        target: &str,
        invert: bool,
        current_function: Option<&FunctionInfo>,
    ) {
        let chain_op = match condition {
            Expr::Binary {
                op: op @ (BinaryOp::AndAnd | BinaryOp::OrOr),
                ..
            } => Some(*op),
            _ => None,
        };

        let Some(chain_op) = chain_op else {
            let (prelude, text) = self.expr(condition, current_function);
            self.lines(prelude);
            let polarity = if invert { "<> 0" } else { "= 0" };
            self.line(&format!("IF ({text}) {polarity} THEN GOTO {target}"));
            return;
        };

        let mut operands = Vec::new();
        flatten_chain(condition, chain_op, &mut operands);

        let is_and = matches!(chain_op, BinaryOp::AndAnd);
        let polarity = if is_and { "= 0" } else { "<> 0" };
        // De Morgan duality: an AND-chain under `invert` behaves like a
        // plain OR-chain (needs a "some operand already decided true" skip
        // label) and vice versa — captured by this single XOR flag.
        let simple = is_and != invert;

        let jump_dest = if simple {
            target.to_string()
        } else {
            let id = self.next_label;
            self.next_label += 1;
            format!("SC_{id:04}_CONT")
        };

        for operand in &operands {
            let (prelude, text) = self.expr(operand, current_function);
            self.lines(prelude);
            self.line(&format!("IF ({text}) {polarity} THEN GOTO {jump_dest}"));
        }

        if !simple {
            self.line(&format!("GOTO {target}"));
            self.line(&format!("{jump_dest}:"));
        }
    }

    fn if_statement(
        &mut self,
        condition: &Expr,
        then_body: &[Stmt],
        else_body: &[Stmt],
        current_function: Option<&FunctionInfo>,
    ) {
        let id = self.next_label;
        self.next_label += 1;
        let else_label = format!("IF_{id:04}_ELSE");
        let end_label = format!("IF_{id:04}_END");

        if else_body.is_empty() {
            self.condition_jump(condition, &end_label, false, current_function);
            self.indent += 1;
            self.statements(then_body, current_function);
            self.indent -= 1;
            self.line(&format!("{end_label}:"));
            self.line("REM END IF");
        } else {
            self.condition_jump(condition, &else_label, false, current_function);
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
        else_body: &[Stmt],
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
            format!("BCCT{id}{suffix}")
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
        self.line(&format!(
            "GOTO {}",
            if else_body.is_empty() {
                &end_label
            } else {
                &else_label
            }
        ));

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

    /// `try ... catch err%, erl% ... end try` -- see `Statement::TryCatch`'s
    /// own doc comment for the semantics. Transpiles straight onto real
    /// BASIC's own `ON ERROR GOTO`/`RESUME <label>`: the catch label is a
    /// synthetic one like `if_statement`/`select_case`'s own, not a named
    /// procedure, so none of `resolver::validate`'s named-error-handler-
    /// target rules apply here. `RESUME <label>` (not a plain `GOTO`) at
    /// the end of the catch body is required, not stylistic -- a bare
    /// `GOTO` out of a handler leaves BASIC's own "currently trapping"
    /// state set, so a *later*, unrelated error elsewhere in the program
    /// would silently fail to trap at all (verified under dosbox-x; see
    /// tutorial/17_labels_and_error_handling.bcl's matching comment).
    fn try_catch(
        &mut self,
        try_body: &[Stmt],
        catch: Option<&TryCatchHandler>,
        finally_body: &[Stmt],
        current_function: Option<&FunctionInfo>,
    ) {
        let id = self.next_label;
        self.next_label += 1;
        let catch_label = format!("TRY_{id:04}_CATCH");
        let finally_label = format!("TRY_{id:04}_FINALLY");
        let end_label = format!("TRY_{id:04}_END");

        self.line(&format!("ON ERROR GOTO {catch_label}"));
        self.indent += 1;
        self.statements(try_body, current_function);
        self.indent -= 1;
        self.line("ON ERROR GOTO 0");
        self.line(&format!("GOTO {finally_label}"));

        self.line(&format!("{catch_label}:"));
        self.indent += 1;
        if let Some(catch) = catch {
            let err_name = self.ident(&catch.err_var, current_function);
            let erl_name = self.ident(&catch.erl_var, current_function);
            self.line(&format!("{err_name} = ERR"));
            self.line(&format!("{erl_name} = ERL"));
            self.statements(&catch.body, current_function);
            self.line(&format!("RESUME {finally_label}"));
        } else {
            let err_name = format!("BCC_TRY_{id:04}_ERR%");
            self.line(&format!("{err_name} = ERR"));
            self.line(&format!("RESUME {finally_label}"));
        }
        self.indent -= 1;

        self.line(&format!("{finally_label}:"));
        self.indent += 1;
        self.statements(finally_body, current_function);
        if catch.is_none() {
            self.line(&format!("IF BCC_TRY_{id:04}_ERR% <> 0 THEN ERROR BCC_TRY_{id:04}_ERR%"));
        }
        self.indent -= 1;

        self.line(&format!("{end_label}:"));
        self.line("REM END TRY");
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
            Expr::HexLit(s) => (Vec::new(), s.clone()),
            Expr::String(value) => (Vec::new(), format!("\"{}\"", escape_string(value))),
            Expr::Ident(ident) => {
                let is_param = current_function
                    .is_some_and(|f| f.params.iter().any(|(src, _)| same_ident(&src.name, ident)));
                // `ERR`/`ERL` are the one pair of zero-arg builtins in
                // `known_callables` that are *always* suffixless in real
                // BASIC -- unlike `DATE$`/`TIME$`/`INKEY$`, whose own
                // legitimate spelling already carries a `$` suffix, so a
                // suffixed `err%`/`erl%` here can only be a genuine user
                // variable (e.g. `try`/`catch`'s own `catch err%, erl%`
                // locals -- see `Statement::TryCatch`), never the real
                // pseudo-variable, which this branch would otherwise
                // silently misrender as the literal text "ERR"/"ERL",
                // discarding whatever value the real local actually holds.
                let is_suffixed_err_or_erl = ident.suffix.is_some()
                    && (ident.name.eq_ignore_ascii_case("err")
                        || ident.name.eq_ignore_ascii_case("erl"));
                let emitted = if !is_param
                    && !is_suffixed_err_or_erl
                    && self
                        .known_callables
                        .contains(&ident.name.to_ascii_lowercase())
                {
                    self.canonical_callable(ident)
                } else {
                    self.ident(ident, current_function)
                };
                (Vec::new(), emitted)
            }
            Expr::ArrayRef { name, indices } => {
                if let Some(info) = self.ordinary_function_info(name).cloned() {
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
                let base = if self
                    .known_callables
                    .contains(&name.name.to_ascii_lowercase())
                {
                    self.canonical_callable(name)
                } else {
                    self.ident(name, current_function)
                };
                (
                    prelude,
                    format!("{}({})", base, rendered_indices.join(", ")),
                )
            }
            Expr::Call { name, args } => {
                let array_bound_builtin = ["sizeof", "lbound", "ubound"]
                    .iter()
                    .find(|builtin| name.name.eq_ignore_ascii_case(builtin));
                if let Some(&builtin_name) = array_bound_builtin {
                    let resolved = match args.first() {
                        Some(Expr::Ident(array_name)) if args.len() <= 2 => {
                            match builtin_name {
                                "lbound" => {
                                    self.resolve_lbound(array_name, args.get(1), current_function)
                                }
                                "ubound" => {
                                    self.resolve_ubound(array_name, args.get(1), current_function)
                                }
                                _ => {
                                    self.resolve_sizeof(array_name, args.get(1), current_function)
                                }
                            }
                        }
                        _ => Err(format!(
                            "{builtin_name} expects an array name, e.g. `{builtin_name}(arr%)` \
                             or `{builtin_name}(grid%, 1)`"
                        )),
                    };
                    return match resolved {
                        Ok(text) => (Vec::new(), text),
                        Err(message) => {
                            self.diagnostics.push(Diagnostic::error(
                                SourcePos::new("<validation>", 1, 1),
                                message,
                            ));
                            (Vec::new(), "1".to_string())
                        }
                    };
                }
                if let Some(info) = self.ordinary_function_info(name).cloned() {
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
                        // Not a recognized callable, so this is really a
                        // multi-index array element access parsed as a Call
                        // (see make_paren_ident_expr in parser.rs) -- needs
                        // the same param/local scope resolution as the
                        // single-index ArrayRef case above, or a function
                        // parameter's or local array's mangled storage name
                        // would be silently skipped.
                        self.ident(name, current_function)
                    };
                    (
                        prelude,
                        format!("{}({})", emit_name, rendered_args.join(", ")),
                    )
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
            Expr::ScalarMethodCall { base, method, args } => {
                let Some(receiver) = self.expr_receiver_type(base) else {
                    self.diagnostics.push(Diagnostic::error(
                        SourcePos::new("<validation>", 1, 1),
                        format!("method receiver for `.{method}()` must be scalar"),
                    ));
                    return (Vec::new(), "0".to_string());
                };
                let Some(info) = self.method_info(receiver, method).cloned() else {
                    self.diagnostics.push(Diagnostic::error(
                        SourcePos::new("<validation>", 1, 1),
                        format!("unknown method `.{method}()`"),
                    ));
                    return (Vec::new(), "0".to_string());
                };
                let mut call_args = Vec::with_capacity(args.len() + 1);
                call_args.push((**base).clone());
                call_args.extend(args.iter().cloned());
                (self.call_lines(&info, &call_args, current_function), info.result.as_basic())
            }
            Expr::FileIndex { .. } | Expr::FieldAccess { .. } | Expr::MethodCall { .. }
            | Expr::RecordLit { .. } => {
                unreachable!("record/file DSL must be lowered before codegen")
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

        // Resolve, once up front, which arguments are being passed as
        // whole arrays -- either `arr%()` (empty parens; array-ness is
        // explicit in the syntax) or a bare identifier that resolves to a
        // declared array *and* whose corresponding parameter is itself
        // declared as an array (array-ness is inferred purely from the
        // callee's signature in that case, so a bare name only counts when
        // the callee already expects an array there). `Some((resolved
        // caller-side mangled name, original source identifier, reconciled
        // rank))` when so -- the original identifier is kept alongside the
        // mangled one because resolving its bounds (below) has to look it
        // up by its *source* name, not the per-function generated one.
        // `None` for a plain scalar argument, or an array-shaped argument
        // whose rank didn't match (reported once, here).
        let array_args: Vec<Option<(String, BasicIdent, usize)>> = args
            .iter()
            .enumerate()
            .map(|(index, arg)| {
                let target_rank = info.param_ranks.get(index).copied().flatten();
                let source_name: &BasicIdent = match arg {
                    Expr::ArrayRef { name, indices } if indices.is_empty() => name,
                    Expr::Ident(name)
                        if target_rank.is_some()
                            && self.resolve_array_rank(name, current_function).is_some() =>
                    {
                        name
                    }
                    _ => return None,
                };
                let source_rank = self.resolve_array_rank(source_name, current_function);
                match (target_rank, source_rank) {
                    (Some(t), Some(s)) if t != s => {
                        let param_name = info
                            .params
                            .get(index)
                            .map(|(p, _)| p.name.as_basic())
                            .unwrap_or_default();
                        self.diagnostics.push(Diagnostic::error(
                            SourcePos::new("<validation>", 1, 1),
                            format!(
                                "`{}` has {} dimension{} here, but parameter `{}` of `{}` is \
                                 indexed with {} -- passing it would generate incorrect BASIC",
                                source_name,
                                s,
                                if s == 1 { "" } else { "s" },
                                param_name,
                                info.source_name,
                                t,
                            ),
                        ));
                        None
                    }
                    _ => {
                        let rank = target_rank.or(source_rank).unwrap_or(1);
                        Some((
                            self.ident(source_name, current_function),
                            source_name.clone(),
                            rank,
                        ))
                    }
                }
            })
            .collect();

        for (index, rendered_arg) in rendered_args.iter().enumerate() {
            if let Some((_, lowered)) = info.params.get(index) {
                if array_args[index].is_none() {
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

        for (param, lowered) in info.params.iter().skip(args.len()) {
            if let Some(default) = &param.default {
                let (default_prelude, rendered_default) = self.expr(default, current_function);
                lines.extend(default_prelude);
                lines.push(format!("{} = {rendered_default}", lowered.as_basic()));
            } else {
                self.diagnostics.push(Diagnostic::error(
                    SourcePos::new("<validation>", 1, 1),
                    format!(
                        "`{}` expects {} argument(s), got {}",
                        info.source_name,
                        info.params.len(),
                        args.len()
                    ),
                ));
            }
        }

        for (index, _arg) in args.iter().enumerate() {
            if let Some((_, lowered)) = info.params.get(index) {
                if let Some((actual_array, source_name, rank)) = &array_args[index] {
                    let bound_vars = info
                        .param_bound_vars
                        .get(index)
                        .cloned()
                        .unwrap_or_default();
                    let capacities = info
                        .param_capacities
                        .get(index)
                        .cloned()
                        .unwrap_or_default();
                    for (axis, bound_var) in bound_vars.iter().enumerate() {
                        let bound = self
                            .resolve_axis_bound(source_name, axis, current_function)
                            .unwrap_or_else(|| {
                                self.diagnostics.push(Diagnostic::error(
                                    SourcePos::new("<validation>", 1, 1),
                                    format!(
                                        "could not determine the size of `{}` along axis {} \
                                         to pass to `{}`",
                                        source_name, axis, info.source_name,
                                    ),
                                ));
                                "1".to_string()
                            });
                        lines.push(format!("{bound_var} = {bound}"));
                        // Parameter storage is DIMed once, at top-level, sized to fit
                        // every call site the transpiler could resolve at compile time
                        // (see `infer_array_param_capacities`). This is the runtime
                        // backstop for whatever that inference couldn't prove safe --
                        // a call passing more elements than the storage was built for
                        // would otherwise silently corrupt whatever memory follows it.
                        if let Some(capacity) = capacities.get(axis) {
                            let param_name = info
                                .params
                                .get(index)
                                .map(|(p, _)| p.name.as_basic())
                                .unwrap_or_default();
                            lines.push(format!(
                                "IF {bound_var} > {capacity} THEN PRINT \"runtime error: `{param_name}` of `{}` needs \"; {bound_var}; \" elements along axis {axis}, but its storage only holds {capacity}\" : STOP",
                                info.source_name,
                            ));
                        }
                    }
                    let loop_vars: Vec<String> = (0..*rank).map(|_| self.next_temp_var()).collect();
                    lines.extend(array_copy_lines(
                        &lowered.as_basic(),
                        actual_array,
                        &bound_vars,
                        "copy array argument into transpiled function storage",
                        &loop_vars,
                    ));
                }
            }
        }

        lines.push(format!("GOSUB {}", info.label));

        for (index, arg) in args.iter().enumerate() {
            if let Some((param, lowered)) = info.params.get(index) {
                if param.mode != ParamMode::ByRef {
                    continue;
                }
                if let Some((actual_array, _source_name, rank)) = &array_args[index] {
                    let bound_vars = info
                        .param_bound_vars
                        .get(index)
                        .cloned()
                        .unwrap_or_default();
                    let loop_vars: Vec<String> = (0..*rank).map(|_| self.next_temp_var()).collect();
                    lines.extend(array_copy_lines(
                        actual_array,
                        &lowered.as_basic(),
                        &bound_vars,
                        "copy mutated array argument back to caller storage",
                        &loop_vars,
                    ));
                } else if let Expr::Ident(ident) = arg {
                    let caller_name = self.ident(ident, current_function);
                    lines.push(format!("{} = {}", caller_name, lowered.as_basic()));
                } else {
                    self.diagnostics.push(Diagnostic::error(
                        SourcePos::new("<validation>", 1, 1),
                        format!(
                            "`byref` parameter `{}` of `{}` was called with an argument that \
                             isn't a plain variable -- byref requires somewhere to write the \
                             result back to",
                            param.name, info.source_name
                        ),
                    ));
                }
            }
        }

        lines
    }

    /// Like `call_lines`, but for a callee (`com.bascal.stdlib.midAssign`,
    /// the only current caller) whose every parameter is a plain byval
    /// scalar, and whose arguments the caller has *already* rendered to
    /// text via its own `self.expr` calls -- so this skips `call_lines`'s
    /// own per-argument evaluation instead of re-running it (which would
    /// re-execute any side effect in an argument expression a second time,
    /// e.g. an array index).
    fn call_lines_from_rendered_scalars(
        &self,
        info: &FunctionInfo,
        rendered_args: &[String],
    ) -> Vec<String> {
        let mut lines: Vec<String> = rendered_args
            .iter()
            .enumerate()
            .filter_map(|(index, rendered)| {
                info.params
                    .get(index)
                    .map(|(_, lowered)| format!("{} = {rendered}", lowered.as_basic()))
            })
            .collect();
        lines.push(format!("GOSUB {}", info.label));
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
        self.next_temp_var_suffixed("%")
    }

    fn next_temp_var_suffixed(&mut self, suffix: &str) -> String {
        let id = self.next_label;
        self.next_label += 1;
        format!("BCCT{id}{suffix}")
    }

    fn canonical_callable(&self, name: &BasicIdent) -> String {
        BasicIdent {
            name: name.name.to_ascii_uppercase(),
            suffix: name.suffix,
        }
        .as_basic()
    }

    fn ident(&self, ident: &BasicIdent, current_function: Option<&FunctionInfo>) -> String {
        let source_key = ident.as_basic().to_ascii_lowercase();
        if self.record_buffer_names.contains(&source_key) {
            // FIELD buffers are structurally global -- there is exactly
            // one FIELD-bound buffer per record field, shared by every
            // function/procedure that touches that file -- so this must
            // be checked before, and instead of, per-function allocation,
            // regardless of scope.
            return if self.synthesized_buffer_names.contains(&source_key) {
                // Transpiler-built (via `buffer_ident`): already
                // deliberately camelCased, so its case is preserved
                // rather than flattened by the normalization below.
                ident.as_basic()
            } else {
                // Author-typed in raw-BASIC-passthrough source: still
                // gets BASCAL's normal lowercase normalization, same as
                // any other identifier.
                BasicIdent {
                    name: ident.name.to_ascii_lowercase(),
                    suffix: ident.suffix,
                }
                .as_basic()
            };
        }
        if self.const_names.contains(&source_key) {
            // `const` values always resolve globally, with or without an
            // explicit `global` declaration -- see the field comment on
            // const_names for why. Checked before the current_function
            // branch below, same as record_buffer_names above.
            return BasicIdent {
                name: ident.name.to_ascii_lowercase(),
                suffix: ident.suffix,
            }
            .as_basic();
        }
        if let Some(info) = current_function {
            // Params have already-allocated lowered names.
            if let Some((_, lowered)) = info
                .params
                .iter()
                .find(|(source, _)| same_ident(&source.name, ident))
            {
                return lowered.as_basic();
            }
            if !info.globals.contains(&source_key) {
                // Check per-function cache first.
                {
                    let cache = info.local_var_map.borrow();
                    if let Some(cached) = cache.get(&source_key) {
                        return cached.clone();
                    }
                }
                // Allocate a name that doesn't clash with any already-claimed BASIC name.
                let preferred_stem = camel_join(&[&info.stem, &ident.name]);
                let lowered = {
                    let taken = self.taken_names.borrow();
                    allocate_unique(&preferred_stem, ident.suffix, &taken)
                };
                let lowered_basic = lowered.as_basic();
                self.taken_names
                    .borrow_mut()
                    .insert(lowered_basic.to_ascii_lowercase());
                info.local_var_map
                    .borrow_mut()
                    .insert(source_key, lowered_basic.clone());
                return lowered_basic;
            }
        }
        BasicIdent {
            name: ident.name.to_ascii_lowercase(),
            suffix: ident.suffix,
        }
        .as_basic()
    }

    fn function_info(&self, name: &BasicIdent) -> Option<&FunctionInfo> {
        self.functions
            .iter()
            .find(|function| same_ident(&function.source_name, name))
    }

    /// Same lookup as `function_info`, but excluding methods
    /// (`receiver.is_some()`) -- used wherever an *ordinary* call site
    /// (`Expr::Call`/`Expr::ArrayRef`, or a bare call statement) is being
    /// resolved, so a method never gets matched here with zero type
    /// checking on its receiver (`function_info` alone would happily match
    /// a method by name+suffix and hand its whole indices/args list
    /// straight to `call_lines`, silently binding the first argument to
    /// `self` regardless of its actual type -- confirmed: `ltrim$(n%)`,
    /// `n%` an Integer, compiled and ran with no diagnostic at all before
    /// this fix, assigning `n%` into `ltrim`'s string `self` param).
    /// `records::Lowerer::try_ordinary_call_as_method` is the one, real,
    /// type-checked path from ordinary-call syntax to a method now -- it
    /// runs before this codegen pass ever sees the program, rewriting an
    /// eligible call into a genuine `Expr::ScalarMethodCall`, which this
    /// function never needs to see at all.
    fn ordinary_function_info(&self, name: &BasicIdent) -> Option<&FunctionInfo> {
        self.functions
            .iter()
            .find(|function| function.receiver.is_none() && same_ident(&function.source_name, name))
    }

    fn method_info(&self, receiver: TypeSuffix, name: &str) -> Option<&FunctionInfo> {
        self.functions.iter().find(|function| {
            function.receiver == Some(receiver) && function.source_name.name.eq_ignore_ascii_case(name)
        })
    }

    fn expr_receiver_type(&self, expr: &Expr) -> Option<TypeSuffix> {
        match expr {
            Expr::String(_) => Some(TypeSuffix::String),
            Expr::Integer(_) | Expr::HexLit(_) => Some(TypeSuffix::Integer),
            Expr::Float(_) => Some(TypeSuffix::Single),
            Expr::Ident(id) | Expr::Call { name: id, .. } | Expr::ArrayRef { name: id, .. } =>
                Some(id.suffix.unwrap_or(TypeSuffix::Single)),
            Expr::Unary { expr, .. } => self.expr_receiver_type(expr),
            Expr::Binary { left, .. } => self.expr_receiver_type(left),
            Expr::ScalarMethodCall { base, method, .. } => {
                let receiver = self.expr_receiver_type(base)?;
                self.method_info(receiver, method)?.source_name.suffix
            }
            _ => None,
        }
    }

    /// Renders a `goto`/`gosub`/`on error goto`/`resume`/`on ... goto`/
    /// `on ... gosub` target. The parser only ever produces a bare label
    /// identifier here, or (for `on error goto` only) the integer `0`
    /// sentinel that disables the error trap.
    ///
    /// A target naming a declared `function`/`procedure` needs special
    /// handling: unlike an ordinary `name:` label (emitted, and later
    /// number-resolved, using the exact text the author wrote), a
    /// function/procedure entry point is emitted under its own synthesized
    /// `FN_<stem>` label (see `FunctionInfo::from_def`) -- an ordinary call
    /// site already knows to emit that directly, but a raw label reference
    /// like this one has no reason to guess it, so look it up through the
    /// function table instead of rendering the identifier text as-is.
    fn label_target_text(&self, target: &Expr) -> String {
        match target {
            Expr::Ident(ident) => match self.function_info(ident) {
                Some(info) => info.label.clone(),
                None => ident.as_basic(),
            },
            Expr::Integer(0) => "0".to_string(),
            _ => unreachable!(
                "goto/gosub/on/resume targets are label identifiers (or the `on error goto 0` sentinel), enforced at parse time"
            ),
        }
    }

    /// Declared rank of the array named `name`, resolved in whatever scope
    /// it's actually visible in: a local `dim` inside `current_function`, a
    /// parameter of `current_function` being forwarded onward (using that
    /// parameter's own inferred rank), or a top-level `dim`. `None` means
    /// unknown -- nothing to check a call site's argument against.
    fn resolve_array_rank(
        &self,
        name: &BasicIdent,
        current_function: Option<&FunctionInfo>,
    ) -> Option<usize> {
        let key = name.as_basic().to_ascii_lowercase();
        if let Some(info) = current_function {
            if let Some(rank) = info.local_array_ranks.get(&key) {
                return Some(*rank);
            }
            if let Some(index) = info
                .params
                .iter()
                .position(|(p, _)| same_ident(&p.name, name))
            {
                return info.param_ranks.get(index).copied().flatten();
            }
        }
        self.top_level_array_ranks.get(&key).copied()
    }

    /// Bound text for one axis of a known array: a frozen DIM-time bound
    /// for a directly-`dim`ed array (local or top-level), or -- for an
    /// array *parameter* -- the transpiler-synthesized hidden variable that
    /// the caller sets (from the actual argument's own resolved bound)
    /// immediately before `GOSUB`. `None` means the axis genuinely can't
    /// be resolved (unknown array, or an unsized `dim arr%()` with no
    /// bounds to freeze).
    fn resolve_axis_bound(
        &self,
        name: &BasicIdent,
        axis: usize,
        current_function: Option<&FunctionInfo>,
    ) -> Option<String> {
        if let Some(info) = current_function {
            if let Some(index) = info
                .params
                .iter()
                .position(|(p, _)| same_ident(&p.name, name))
            {
                return info
                    .param_bound_vars
                    .get(index)
                    .and_then(|bounds| bounds.get(axis))
                    .cloned();
            }
            let key = name.as_basic().to_ascii_lowercase();
            if let Some(bound) = info
                .local_array_bounds
                .borrow()
                .get(&key)
                .and_then(|b| b.get(axis))
            {
                return Some(bound.clone());
            }
        }
        let key = name.as_basic().to_ascii_lowercase();
        self.top_level_array_bounds
            .get(&key)
            .and_then(|b| b.get(axis))
            .cloned()
    }

    /// Shared by `sizeof`/`LBOUND`/`UBOUND`: validates `name` is a known
    /// array and `axis_expr` names one of its real axes (a literal
    /// integer, defaulting to `0` for a 1-D array; required and checked
    /// in range for 2-D+), then resolves that axis's raw `DIM` bound the
    /// same way `resolve_axis_bound` always has -- a real top-level
    /// array's literal-or-captured-at-DIM-time bound, or (inside a
    /// function, for one of its own array parameters) the hidden
    /// auto-injected bound variable the caller sets immediately before
    /// `GOSUB`. `UBOUND` exposes this value directly; `sizeof` adds 1 to
    /// it (see `resolve_sizeof`'s own doc comment); `LBOUND` ignores it
    /// entirely and always resolves to the literal `0` (see
    /// `resolve_lbound`'s own doc comment) -- but still needs the same
    /// validation, so an unknown array or a bad axis is still a clear
    /// error rather than a silently-wrong `0`.
    fn resolve_array_bound_for_builtin(
        &self,
        builtin_name: &str,
        name: &BasicIdent,
        axis_expr: Option<&Expr>,
        current_function: Option<&FunctionInfo>,
    ) -> Result<String, String> {
        let rank = self
            .resolve_array_rank(name, current_function)
            .ok_or_else(|| {
                format!("`{name}` isn't a known array, so `{builtin_name}` can't determine its size")
            })?;

        let axis = match axis_expr {
            Some(Expr::Integer(n)) => *n as usize,
            Some(_) => {
                return Err(format!(
                    "the axis argument to `{builtin_name}` must be a literal integer"
                ))
            }
            None if rank == 1 => 0,
            None => {
                return Err(format!(
                    "`{name}` has {rank} dimensions -- {builtin_name} needs an axis argument, \
                     e.g. `{builtin_name}({name}, 0)`"
                ))
            }
        };
        if axis >= rank {
            return Err(format!(
                "`{name}` only has {rank} dimension{} -- axis {axis} doesn't exist",
                if rank == 1 { "" } else { "s" }
            ));
        }

        self.resolve_axis_bound(name, axis, current_function)
            .ok_or_else(|| format!("could not determine the size of `{name}`"))
    }

    /// Resolves `UBOUND(name)` / `UBOUND(name, axis)` -- the array's real
    /// declared bound along an axis (its highest valid index), exactly
    /// the value `resolve_array_bound_for_builtin` gives back.
    fn resolve_ubound(
        &self,
        name: &BasicIdent,
        axis_expr: Option<&Expr>,
        current_function: Option<&FunctionInfo>,
    ) -> Result<String, String> {
        self.resolve_array_bound_for_builtin("UBOUND", name, axis_expr, current_function)
    }

    /// Resolves `LBOUND(name)` / `LBOUND(name, axis)` -- always the
    /// literal `0`, since BASCAL only supports base-0 array indexing
    /// (`OPTION BASE` is rejected outright -- see GitHub issue #50). Still
    /// runs the same array-known/axis-valid validation
    /// `resolve_array_bound_for_builtin` does for `UBOUND`/`sizeof`, so
    /// `LBOUND(nope%)` is a clear error, not a silently-wrong `0`.
    fn resolve_lbound(
        &self,
        name: &BasicIdent,
        axis_expr: Option<&Expr>,
        current_function: Option<&FunctionInfo>,
    ) -> Result<String, String> {
        self.resolve_array_bound_for_builtin("LBOUND", name, axis_expr, current_function)?;
        Ok("0".to_string())
    }

    /// Resolves `sizeof(name)` / `sizeof(name, axis)` to the text that
    /// should replace the call in generated code -- the array's real
    /// element *count* along this axis, i.e. `UBOUND(name, axis) + 1`
    /// (real BASIC's own inclusive-bound convention: a `DIM arr%(N)` axis
    /// holds `N + 1` elements, indices `0..=N`, so the bound alone is one
    /// short of the count). Adding 1 to an already-resolved integer
    /// literal keeps the common case's generated code a plain literal
    /// rather than a `(9 + 1)` expression; a captured runtime bound (a
    /// variable name, from a non-literal `DIM` size, or an array
    /// parameter's own hidden bound variable) still needs the arithmetic
    /// spelled out.
    fn resolve_sizeof(
        &self,
        name: &BasicIdent,
        axis_expr: Option<&Expr>,
        current_function: Option<&FunctionInfo>,
    ) -> Result<String, String> {
        let bound = self.resolve_ubound(name, axis_expr, current_function)?;
        Ok(match bound.parse::<i64>() {
            Ok(n) => (n + 1).to_string(),
            Err(_) => format!("({bound} + 1)"),
        })
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
    fn from_def(
        function: &FunctionDef,
        taken: &mut HashSet<String>,
        known_callables: &HashSet<String>,
        diagnostics: &mut Vec<Diagnostic>,
        mut param_capacities: Vec<Vec<i64>>,
    ) -> Self {
        let stem = sanitize_symbol(&function.name.name);
        let mut params: Vec<(Param, BasicIdent)> = function
            .params
            .iter()
            .map(|param| {
                let preferred = camel_join(&[&stem, &param.name.name]);
                let lowered = allocate_unique(&preferred, param.name.suffix, taken);
                taken.insert(lowered.as_basic().to_ascii_lowercase());
                (param.clone(), lowered)
            })
            .collect();
        let mut param_ranks = infer_param_ranks(function, known_callables, diagnostics);
        let mut param_bound_vars: Vec<Vec<String>> = function
            .params
            .iter()
            .zip(param_ranks.iter())
            .map(|(param, rank)| match rank {
                Some(rank) => (0..*rank)
                    .map(|axis| {
                        let preferred =
                            camel_join(&[&stem, &param.name.name, &format!("dim{axis}")]);
                        let lowered = allocate_unique(&preferred, Some(TypeSuffix::Integer), taken);
                        taken.insert(lowered.as_basic().to_ascii_lowercase());
                        lowered.as_basic()
                    })
                    .collect(),
                None => Vec::new(),
            })
            .collect();
        if let Some(receiver) = function.receiver {
            let self_param = Param {
                name: BasicIdent { name: "self".to_string(), suffix: Some(receiver) },
                mode: ParamMode::ByVal,
                default: None,
                axes: None,
            };
            let preferred = camel_join(&[&stem, "self"]);
            let lowered = allocate_unique(&preferred, Some(receiver), taken);
            taken.insert(lowered.as_basic().to_ascii_lowercase());
            params.insert(0, (self_param, lowered));
            param_ranks.insert(0, None);
            param_bound_vars.insert(0, Vec::new());
            param_capacities.insert(0, Vec::new());
        }
        let local_array_ranks = dim_ranks_in_body(&function.body);
        let result = allocate_unique(&camel_join(&[&stem, "result"]), function.name.suffix, taken);
        taken.insert(result.as_basic().to_ascii_lowercase());
        let globals = collect_globals(&function.body);
        Self {
            source_name: function.name.clone(),
            stem: stem.clone(),
            label: function_label(&stem, function.name.suffix, function.receiver),
            result,
            params,
            param_ranks,
            param_bound_vars,
            param_capacities,
            local_array_ranks,
            local_array_bounds: RefCell::new(HashMap::new()),
            is_procedure: function.is_procedure,
            receiver: function.receiver,
            globals,
            local_var_map: RefCell::new(HashMap::new()),
        }
    }
}

/// Visits every `Expr` node (recursively, including sub-expressions) that
/// appears anywhere in `body` -- every statement kind, every clause. Used to
/// find every array-element access to a given parameter, wherever it
/// appears in a function body, so its declared rank can be inferred from
/// how many indices it's actually used with.
pub(crate) fn visit_body_exprs<'a>(body: &'a [Stmt], f: &mut impl FnMut(&'a Expr)) {
    for stmt in body {
        visit_statement_exprs(stmt, f);
    }
}

fn visit_statement_exprs<'a>(stmt: &'a Stmt, f: &mut impl FnMut(&'a Expr)) {
    match &stmt.kind {
        Statement::Dim { sizes, .. } => {
            for e in sizes {
                visit_expr(e, f);
            }
        }
        Statement::Open {
            file, channel, len, ..
        } => {
            visit_expr(file, f);
            visit_expr(channel, f);
            if let Some(e) = len {
                visit_expr(e, f);
            }
        }
        Statement::FileDecl { path, .. } => visit_expr(path, f),
        Statement::LineInput { channel, target } => {
            visit_expr(channel, f);
            visit_expr(target, f);
        }
        Statement::PrintFile { channel, tokens } => {
            visit_expr(channel, f);
            visit_print_tokens(tokens, f);
        }
        Statement::PrintUsing { format, tokens } => {
            visit_expr(format, f);
            visit_print_tokens(tokens, f);
        }
        Statement::PrintFileUsing {
            channel,
            format,
            tokens,
        } => {
            visit_expr(channel, f);
            visit_expr(format, f);
            visit_print_tokens(tokens, f);
        }
        Statement::Close { channel } => visit_expr(channel, f),
        Statement::Kill { file } => visit_expr(file, f),
        Statement::Name { from, to } => {
            visit_expr(from, f);
            visit_expr(to, f);
        }
        Statement::Assignment { target, value } => {
            visit_expr(target, f);
            visit_expr(value, f);
        }
        Statement::MidAssign {
            target,
            start,
            len,
            value,
        } => {
            visit_expr(target, f);
            visit_expr(start, f);
            if let Some(e) = len {
                visit_expr(e, f);
            }
            visit_expr(value, f);
        }
        Statement::Print { tokens } => visit_print_tokens(tokens, f),
        Statement::Return { value } => visit_expr(value, f),
        Statement::If {
            condition,
            then_body,
            else_body,
        } => {
            visit_expr(condition, f);
            visit_body_exprs(then_body, f);
            visit_body_exprs(else_body, f);
        }
        Statement::For {
            start,
            end,
            step,
            body,
            ..
        } => {
            visit_expr(start, f);
            visit_expr(end, f);
            if let Some(e) = step {
                visit_expr(e, f);
            }
            visit_body_exprs(body, f);
        }
        Statement::While { condition, body } => {
            visit_expr(condition, f);
            visit_body_exprs(body, f);
        }
        Statement::Do {
            condition,
            body,
            post_condition,
        } => {
            if let Some(c) = condition {
                visit_expr(&c.expr, f);
            }
            visit_body_exprs(body, f);
            if let Some(c) = post_condition {
                visit_expr(&c.expr, f);
            }
        }
        Statement::ExprStmt(e) => visit_expr(e, f),
        Statement::OptionBase(e) => visit_expr(e, f),
        Statement::Erase(_) => {}
        Statement::Randomize(e) => {
            if let Some(e) = e {
                visit_expr(e, f);
            }
        }
        Statement::Swap(a, b) => {
            visit_expr(a, f);
            visit_expr(b, f);
        }
        Statement::Poke { address, value } => {
            visit_expr(address, f);
            visit_expr(value, f);
        }
        Statement::Goto(e) | Statement::Gosub(e) => visit_expr(e, f),
        Statement::OnErrorGoto { target } => visit_expr(target, f),
        Statement::Resume(kind) => {
            if let ResumeTarget::Line(e) = kind {
                visit_expr(e, f);
            }
        }
        Statement::ErrorStmt { code } => visit_expr(code, f),
        Statement::Input { vars, .. } => {
            for e in vars {
                visit_expr(e, f);
            }
        }
        Statement::InputFile { channel, vars } => {
            visit_expr(channel, f);
            for e in vars {
                visit_expr(e, f);
            }
        }
        Statement::Data(values) | Statement::Read(values) => {
            for e in values {
                visit_expr(e, f);
            }
        }
        Statement::Restore(e) => {
            if let Some(e) = e {
                visit_expr(e, f);
            }
        }
        Statement::Const { value, .. } => visit_expr(value, f),
        Statement::Write { channel, exprs } => {
            visit_expr(channel, f);
            for e in exprs {
                visit_expr(e, f);
            }
        }
        Statement::Field {
            channel, fields, ..
        } => {
            visit_expr(channel, f);
            for (w, _) in fields {
                visit_expr(w, f);
            }
        }
        Statement::Get {
            channel,
            record,
            var,
            ..
        }
        | Statement::Put {
            channel,
            record,
            var,
            ..
        } => {
            visit_expr(channel, f);
            if let Some(e) = record {
                visit_expr(e, f);
            }
            if let Some(e) = var {
                visit_expr(e, f);
            }
        }
        Statement::Lset { value, .. } | Statement::Rset { value, .. } => visit_expr(value, f),
        Statement::Seek { channel, position } => {
            visit_expr(channel, f);
            visit_expr(position, f);
        }
        Statement::Lprint(tokens) => visit_print_tokens(tokens, f),
        Statement::LprintUsing { format, tokens } => {
            visit_expr(format, f);
            visit_print_tokens(tokens, f);
        }
        Statement::SelectCase {
            expr,
            cases,
            else_body,
        } => {
            visit_expr(expr, f);
            for case in cases {
                for v in &case.values {
                    match v {
                        CaseValue::Single(e) | CaseValue::Is { value: e, .. } => visit_expr(e, f),
                        CaseValue::Range { from, to } => {
                            visit_expr(from, f);
                            visit_expr(to, f);
                        }
                    }
                }
                visit_body_exprs(&case.body, f);
            }
            visit_body_exprs(else_body, f);
        }
        Statement::TryCatch {
            try_body,
            catch,
            finally_body,
            ..
        } => {
            visit_body_exprs(try_body, f);
            if let Some(catch) = catch {
                visit_body_exprs(&catch.body, f);
            }
            visit_body_exprs(finally_body, f);
        }
        Statement::Locate { row, col } => {
            visit_expr(row, f);
            visit_expr(col, f);
        }
        Statement::Color { fg, bg } => {
            visit_expr(fg, f);
            if let Some(e) = bg {
                visit_expr(e, f);
            }
        }
        Statement::OnBranch { expr, targets, .. } => {
            visit_expr(expr, f);
            for e in targets {
                visit_expr(e, f);
            }
        }
        Statement::Out { port, value } => {
            visit_expr(port, f);
            visit_expr(value, f);
        }
        Statement::Width { channel, cols } => {
            if let Some(e) = channel {
                visit_expr(e, f);
            }
            visit_expr(cols, f);
        }
        Statement::End
        | Statement::Stop
        | Statement::Cls
        | Statement::Beep
        | Statement::System
        | Statement::Clear
        | Statement::ReturnVoid
        | Statement::GlobalDecl(_)
        | Statement::Raw(_)
        | Statement::BlockComment(_)
        | Statement::Label(_)
        | Statement::BlankLine
        | Statement::Exit => {}
    }
}

fn visit_print_tokens<'a>(tokens: &'a [PrintToken], f: &mut impl FnMut(&'a Expr)) {
    for t in tokens {
        if let PrintToken::Expr(e) = t {
            visit_expr(e, f);
        }
    }
}

fn visit_expr<'a>(expr: &'a Expr, f: &mut impl FnMut(&'a Expr)) {
    f(expr);
    match expr {
        Expr::Integer(_) | Expr::Float(_) | Expr::HexLit(_) | Expr::String(_) | Expr::Ident(_) => {}
        Expr::ArrayRef { indices, .. } => {
            for e in indices {
                visit_expr(e, f);
            }
        }
        Expr::Call { args, .. } => {
            for e in args {
                visit_expr(e, f);
            }
        }
        Expr::Unary { expr, .. } => visit_expr(expr, f),
        Expr::Binary { left, right, .. } => {
            visit_expr(left, f);
            visit_expr(right, f);
        }
        Expr::FileIndex { index, .. } => visit_expr(index, f),
        Expr::FieldAccess { base, .. } => visit_expr(base, f),
        Expr::MethodCall { base, args, .. } => {
            visit_expr(base, f);
            for e in args {
                visit_expr(e, f);
            }
        }
        Expr::ScalarMethodCall { base, args, .. } => {
            visit_expr(base, f);
            for e in args { visit_expr(e, f); }
        }
        Expr::RecordLit { fields, .. } => {
            for (_, e) in fields {
                visit_expr(e, f);
            }
        }
    }
}

/// Infers each parameter's array rank (number of subscripts) from how it's
/// actually indexed inside the function's own body -- there's no type
/// annotation to read it from directly. `None` means either the parameter
/// is never directly indexed in this body (e.g. it's only ever forwarded
/// on as a whole array to another call), or it's indexed inconsistently,
/// which is reported as its own diagnostic.
fn infer_param_ranks(
    function: &FunctionDef,
    known_callables: &HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<Option<usize>> {
    function
        .params
        .iter()
        .map(|param| {
            let mut ranks: HashSet<usize> = HashSet::new();
            visit_body_exprs(&function.body, &mut |e| {
                // `make_paren_ident_expr` in parser.rs only produces
                // ArrayRef for empty parens or exactly one index; anything
                // else -- including every 2+ dimensional array access --
                // parses as a Call, disambiguated from a real function call
                // later by name. Both shapes need checking here.
                let (name, count) = match e {
                    Expr::ArrayRef { name, indices } if !indices.is_empty() => {
                        (name, indices.len())
                    }
                    Expr::Call { name, args }
                        if !known_callables.contains(&name.name.to_ascii_lowercase()) =>
                    {
                        (name, args.len())
                    }
                    _ => return,
                };
                if same_ident(name, &param.name) {
                    ranks.insert(count);
                }
            });
            let usage_rank = match ranks.len() {
                0 => None,
                1 => ranks.into_iter().next(),
                _ => {
                    diagnostics.push(Diagnostic::error(
                        SourcePos::new("<validation>", 1, 1),
                        format!(
                            "parameter `{}` of `{}` is indexed with different numbers of \
                             subscripts in different places -- BASCAL can't tell how many \
                             dimensions it has",
                            param.name, function.name
                        ),
                    ));
                    None
                }
            };

            // The declaration (`arr%(?)`, `arr%(?, ?)`, ...) is the
            // authoritative rank when present. Body usage is still checked
            // against it -- an array parameter with no declared rank at
            // all is rejected outright, since there's no other way to
            // learn a parameter's rank from its declaration.
            match (param.rank(), usage_rank) {
                (Some(declared), Some(used)) if declared != used => {
                    diagnostics.push(Diagnostic::error(
                        SourcePos::new("<validation>", 1, 1),
                        format!(
                            "parameter `{}` of `{}` is declared with {} dimension{} but indexed \
                             with {} subscript{} in the body",
                            param.name,
                            function.name,
                            declared,
                            if declared == 1 { "" } else { "s" },
                            used,
                            if used == 1 { "" } else { "s" },
                        ),
                    ));
                    None
                }
                (Some(declared), _) => Some(declared),
                (None, Some(used)) => {
                    let placeholders = vec!["?"; used].join(", ");
                    diagnostics.push(Diagnostic::error(
                        SourcePos::new("<validation>", 1, 1),
                        format!(
                            "parameter `{}` of `{}` is indexed as a {}-D array in the body, but \
                             its declaration doesn't say so -- write `{}({})`",
                            param.name,
                            function.name,
                            used,
                            param.name.as_basic(),
                            placeholders,
                        ),
                    ));
                    None
                }
                (None, None) => None,
            }
        })
        .collect()
}

/// Declared rank (number of DIM dimensions) of every array DIMed anywhere
/// in `body`, lowercase name -> rank. `dim arr%()` (no bounds written) has
/// no rank recorded here -- there's nothing to check it against.
fn dim_ranks_in_body(body: &[Stmt]) -> HashMap<String, usize> {
    let mut ranks = HashMap::new();
    collect_dim_ranks(body, &mut ranks);
    ranks
}

fn collect_dim_ranks(body: &[Stmt], out: &mut HashMap<String, usize>) {
    for stmt in body {
        match &stmt.kind {
            Statement::Dim {
                name,
                is_array,
                sizes,
            } => {
                if *is_array && !sizes.is_empty() {
                    out.insert(name.as_basic().to_ascii_lowercase(), sizes.len());
                }
            }
            Statement::If {
                then_body,
                else_body,
                ..
            } => {
                collect_dim_ranks(then_body, out);
                collect_dim_ranks(else_body, out);
            }
            Statement::For { body, .. }
            | Statement::While { body, .. }
            | Statement::Do { body, .. } => collect_dim_ranks(body, out),
            Statement::SelectCase {
                cases, else_body, ..
            } => {
                for case in cases {
                    collect_dim_ranks(&case.body, out);
                }
                collect_dim_ranks(else_body, out);
            }
            _ => {}
        }
    }
}

// ── array parameter storage capacity inference ──────────────────────────
//
// Every array parameter's shared storage array is DIMed exactly once, at
// top-level, before any call happens (classic BASIC has no REDIM, so a
// shared storage slot can never be resized once DIMed). Its size has to be
// a fixed capacity, decided once, big enough for the largest thing any
// call site ever passes it. This section computes that capacity: for a
// `?` axis, the max of every call site's resolved bound, but only when
// every one of those bounds is itself resolvable at compile time (a
// literal, a `const`, or -- when the array being passed is itself another
// function's array parameter being forwarded onward -- that parameter's
// own already-resolved capacity). An axis that can't be resolved this way
// needs an explicit literal capacity written in the declaration instead.

/// One axis's resolved bound for a single call site's array argument.
enum ArgBound {
    /// This argument isn't array-shaped at all (a scalar, or a rank
    /// mismatch that a later, more specific diagnostic will catch) --
    /// contributes nothing, doesn't count as a data point either way.
    NotAnArray,
    /// It's an array, but this axis's bound can't be pinned to a concrete
    /// integer -- a genuinely dynamic (runtime) value, or a forwarded
    /// parameter whose own capacity hasn't resolved yet this round.
    Unresolvable,
    Resolved(i64),
}

/// Evaluates `expr` to a concrete integer if it's a compile-time constant:
/// a literal, a reference to an unambiguous `const` (recursively), or
/// +/-/*// on two such values. Anything else (a plain variable, a function
/// call, an ambiguous multiply-defined `const` name) is `None` -- a
/// genuine runtime value, not something this pass can reason about.
fn const_eval(expr: &Expr, consts: &HashMap<String, Vec<Expr>>, depth: u32) -> Option<i64> {
    if depth > 32 {
        return None; // guards against a self-referential `const`
    }
    match expr {
        Expr::Integer(n) => Some(*n),
        Expr::Ident(name) => {
            let key = name.as_basic().to_ascii_lowercase();
            match consts.get(&key) {
                Some(defs) if defs.len() == 1 => const_eval(&defs[0], consts, depth + 1),
                _ => None,
            }
        }
        Expr::Unary {
            op: UnaryOp::Neg,
            expr,
        } => const_eval(expr, consts, depth + 1).map(|v| -v),
        Expr::Binary { left, op, right } => {
            let l = const_eval(left, consts, depth + 1)?;
            let r = const_eval(right, consts, depth + 1)?;
            match op {
                BinaryOp::Add => Some(l.wrapping_add(r)),
                BinaryOp::Sub => Some(l.wrapping_sub(r)),
                BinaryOp::Mul => Some(l.wrapping_mul(r)),
                BinaryOp::Div if r != 0 => Some(l / r),
                _ => None,
            }
        }
        _ => None,
    }
}

/// Every `const` declaration anywhere in `body` (recursing into nested
/// blocks), keyed by lowercase name. More than one definition under the
/// same name is tracked (not merged) so `const_eval` can refuse to guess
/// which one a reference means.
fn collect_consts(body: &[Stmt], out: &mut HashMap<String, Vec<Expr>>) {
    for stmt in body {
        match &stmt.kind {
            Statement::Const { name, value } => {
                out.entry(name.as_basic().to_ascii_lowercase())
                    .or_default()
                    .push(value.clone());
            }
            Statement::If {
                then_body,
                else_body,
                ..
            } => {
                collect_consts(then_body, out);
                collect_consts(else_body, out);
            }
            Statement::For { body, .. }
            | Statement::While { body, .. }
            | Statement::Do { body, .. } => collect_consts(body, out),
            Statement::SelectCase {
                cases, else_body, ..
            } => {
                for case in cases {
                    collect_consts(&case.body, out);
                }
                collect_consts(else_body, out);
            }
            _ => {}
        }
    }
}

/// Every `dim`ed array's full size-expression list anywhere in `body`
/// (recursing into nested blocks), keyed by lowercase name -- the same
/// traversal as `collect_dim_ranks`, but keeping the bound expressions
/// themselves instead of just their count.
fn collect_dim_sizes(body: &[Stmt], out: &mut HashMap<String, Vec<Expr>>) {
    for stmt in body {
        match &stmt.kind {
            Statement::Dim {
                name,
                is_array,
                sizes,
            } => {
                if *is_array && !sizes.is_empty() {
                    out.insert(name.as_basic().to_ascii_lowercase(), sizes.clone());
                }
            }
            Statement::If {
                then_body,
                else_body,
                ..
            } => {
                collect_dim_sizes(then_body, out);
                collect_dim_sizes(else_body, out);
            }
            Statement::For { body, .. }
            | Statement::While { body, .. }
            | Statement::Do { body, .. } => collect_dim_sizes(body, out),
            Statement::SelectCase {
                cases, else_body, ..
            } => {
                for case in cases {
                    collect_dim_sizes(&case.body, out);
                }
                collect_dim_sizes(else_body, out);
            }
            _ => {}
        }
    }
}

/// Every call site anywhere in the program that calls one of the
/// program's own functions: `(enclosing function name, lowercase -- None
/// for top-level, callee name, argument list)`. A single-argument call
/// (`f%(x%)`) parses as `Expr::ArrayRef`, not `Expr::Call` (see
/// `make_paren_ident_expr` in `parser.rs`), so both shapes are checked.
fn collect_call_sites(
    program: &Program,
    function_names: &HashSet<String>,
) -> Vec<(Option<String>, BasicIdent, Vec<Expr>)> {
    let mut sites = Vec::new();
    let mut scan = |scope: Option<String>, body: &[Stmt]| {
        let mut visit = |e: &Expr| match e {
            Expr::Call { name, args }
                if function_names.contains(&name.name.to_ascii_lowercase()) =>
            {
                sites.push((scope.clone(), name.clone(), args.clone()));
            }
            Expr::ArrayRef { name, indices }
                if function_names.contains(&name.name.to_ascii_lowercase()) =>
            {
                sites.push((scope.clone(), name.clone(), indices.clone()));
            }
            _ => {}
        };
        visit_body_exprs(body, &mut visit);
    };
    scan(None, &program.statements);
    for f in &program.functions {
        scan(Some(f.name.name.to_ascii_lowercase()), &f.body);
    }
    sites
}

/// Resolves one call site's argument to a concrete per-axis bound, if
/// possible -- see `ArgBound`.
fn resolve_call_arg_bound(
    scope: &Option<String>,
    arg: &Expr,
    axis: usize,
    resolved: &HashMap<String, Vec<Vec<Option<i64>>>>,
    local_dim_sizes: &HashMap<String, HashMap<String, Vec<Expr>>>,
    top_level_dim_sizes: &HashMap<String, Vec<Expr>>,
    functions_by_name: &HashMap<String, &FunctionDef>,
    consts: &HashMap<String, Vec<Expr>>,
) -> ArgBound {
    let source_name: &BasicIdent = match arg {
        Expr::ArrayRef { name, indices } if indices.is_empty() => name,
        Expr::Ident(name) => name,
        _ => return ArgBound::NotAnArray,
    };
    let key = source_name.as_basic().to_ascii_lowercase();

    if let Some(func) = scope {
        if let Some(def) = functions_by_name.get(func) {
            if let Some(idx) = def
                .params
                .iter()
                .position(|p| p.name.as_basic().to_ascii_lowercase() == key)
            {
                return if def.params[idx].axes.is_some() {
                    match resolved
                        .get(func)
                        .and_then(|v| v.get(idx))
                        .and_then(|a| a.get(axis))
                    {
                        Some(Some(v)) => ArgBound::Resolved(*v),
                        _ => ArgBound::Unresolvable,
                    }
                } else {
                    ArgBound::NotAnArray
                };
            }
        }
        if let Some(sizes) = local_dim_sizes.get(func).and_then(|m| m.get(&key)) {
            return match sizes.get(axis).and_then(|e| const_eval(e, consts, 0)) {
                Some(v) => ArgBound::Resolved(v),
                None => ArgBound::Unresolvable,
            };
        }
    }
    if let Some(sizes) = top_level_dim_sizes.get(&key) {
        return match sizes.get(axis).and_then(|e| const_eval(e, consts, 0)) {
            Some(v) => ArgBound::Resolved(v),
            None => ArgBound::Unresolvable,
        };
    }
    ArgBound::NotAnArray
}

/// Resolves every array parameter's per-axis storage capacity across the
/// whole program: lowercase function name -> per-parameter (empty for a
/// scalar) -> per-axis capacity. An axis whose capacity couldn't be
/// resolved (an unresolvable `?`, reported as a diagnostic) comes back as
/// `0` -- codegen still runs to completion so every diagnostic in the
/// program gets collected, but the result is discarded once any
/// diagnostic exists.
fn infer_array_param_capacities(
    program: &Program,
    diagnostics: &mut Vec<Diagnostic>,
) -> HashMap<String, Vec<Vec<i64>>> {
    let function_names: HashSet<String> = program
        .functions
        .iter()
        .map(|f| f.name.name.to_ascii_lowercase())
        .collect();
    let functions_by_name: HashMap<String, &FunctionDef> = program
        .functions
        .iter()
        .map(|f| (f.name.name.to_ascii_lowercase(), f))
        .collect();

    let mut consts = HashMap::new();
    collect_consts(&program.statements, &mut consts);
    for f in &program.functions {
        collect_consts(&f.body, &mut consts);
    }

    let mut top_level_dim_sizes = HashMap::new();
    collect_dim_sizes(&program.statements, &mut top_level_dim_sizes);

    let mut local_dim_sizes: HashMap<String, HashMap<String, Vec<Expr>>> = HashMap::new();
    for f in &program.functions {
        let mut sizes = HashMap::new();
        collect_dim_sizes(&f.body, &mut sizes);
        local_dim_sizes.insert(f.name.name.to_ascii_lowercase(), sizes);
    }

    let call_sites = collect_call_sites(program, &function_names);

    let mut resolved: HashMap<String, Vec<Vec<Option<i64>>>> = program
        .functions
        .iter()
        .map(|f| {
            let per_param = f
                .params
                .iter()
                .map(|p| p.axes.clone().unwrap_or_default())
                .collect();
            (f.name.name.to_ascii_lowercase(), per_param)
        })
        .collect();

    // Fixed-point: each round resolves whatever `?` axes it can from
    // already-known bounds (literals, consts, or another parameter's
    // already-resolved capacity). Since BASCAL rejects every call cycle,
    // direct or indirect, the dependency chain through forwarded array
    // parameters is finite, so this always terminates.
    loop {
        let mut changed = false;
        for f in &program.functions {
            let fname = f.name.name.to_ascii_lowercase();
            for (param_index, param) in f.params.iter().enumerate() {
                let Some(declared_axes) = &param.axes else {
                    continue;
                };
                for axis in 0..declared_axes.len() {
                    if resolved[&fname][param_index][axis].is_some() {
                        continue;
                    }
                    let mut max_value: Option<i64> = None;
                    let mut all_resolved = true;
                    let mut any_call_site = false;
                    for (scope, callee, call_args) in &call_sites {
                        if callee.name.to_ascii_lowercase() != fname {
                            continue;
                        }
                        let Some(arg) = call_args.get(param_index) else {
                            continue;
                        };
                        match resolve_call_arg_bound(
                            scope,
                            arg,
                            axis,
                            &resolved,
                            &local_dim_sizes,
                            &top_level_dim_sizes,
                            &functions_by_name,
                            &consts,
                        ) {
                            ArgBound::NotAnArray => {}
                            ArgBound::Unresolvable => {
                                any_call_site = true;
                                all_resolved = false;
                            }
                            ArgBound::Resolved(v) => {
                                any_call_site = true;
                                max_value = Some(max_value.map_or(v, |m: i64| m.max(v)));
                            }
                        }
                    }
                    if any_call_site && all_resolved {
                        if let Some(v) = max_value {
                            resolved.get_mut(&fname).unwrap()[param_index][axis] = Some(v);
                            changed = true;
                        }
                    }
                }
            }
        }
        if !changed {
            break;
        }
    }

    // Compile-time-provable overflow check: for every axis (inferred or
    // explicit), any call site whose bound *does* resolve gets compared
    // against the final capacity right now, instead of waiting to catch
    // it with the runtime check `call_lines` emits for the general case.
    for f in &program.functions {
        let fname = f.name.name.to_ascii_lowercase();
        for (param_index, param) in f.params.iter().enumerate() {
            let Some(declared_axes) = &param.axes else {
                continue;
            };
            for axis in 0..declared_axes.len() {
                let Some(capacity) = resolved[&fname][param_index][axis] else {
                    continue;
                };
                for (scope, callee, call_args) in &call_sites {
                    if callee.name.to_ascii_lowercase() != fname {
                        continue;
                    }
                    let Some(arg) = call_args.get(param_index) else {
                        continue;
                    };
                    if let ArgBound::Resolved(actual) = resolve_call_arg_bound(
                        scope,
                        arg,
                        axis,
                        &resolved,
                        &local_dim_sizes,
                        &top_level_dim_sizes,
                        &functions_by_name,
                        &consts,
                    ) {
                        if actual > capacity {
                            diagnostics.push(Diagnostic::error(
                                SourcePos::new("<validation>", 1, 1),
                                format!(
                                    "a call to `{}` passes {} elements along axis {} of `{}`, \
                                     but its storage is only sized for {} -- give `{}` a \
                                     bigger explicit capacity",
                                    f.name, actual, axis, param.name, capacity, param.name,
                                ),
                            ));
                        }
                    }
                }
            }
        }
    }

    // Any `?` axis still unresolved at this point genuinely can't be
    // inferred -- either no call site could be resolved, or the parameter
    // is never called at all.
    for f in &program.functions {
        let fname = f.name.name.to_ascii_lowercase();
        for (param_index, param) in f.params.iter().enumerate() {
            let Some(declared_axes) = &param.axes else {
                continue;
            };
            for axis in 0..declared_axes.len() {
                if resolved[&fname][param_index][axis].is_some() {
                    continue;
                }
                let any_call_site = call_sites.iter().any(|(_, callee, call_args)| {
                    callee.name.to_ascii_lowercase() == fname
                        && call_args.get(param_index).is_some()
                });
                let message = if any_call_site {
                    format!(
                        "can't automatically size `{}`'s storage along axis {} of `{}` -- at \
                         least one call site passes an array whose size isn't a compile-time \
                         constant. Give it an explicit capacity instead of `?`, e.g. `{}(100)`",
                        param.name, axis, f.name, param.name,
                    )
                } else {
                    format!(
                        "can't automatically size `{}`'s storage along axis {} of `{}` -- `{}` \
                         is never called, so there's no call site to infer a capacity from. \
                         Give it an explicit capacity instead of `?`, e.g. `{}(100)`",
                        param.name, axis, f.name, f.name, param.name,
                    )
                };
                diagnostics.push(Diagnostic::error(
                    SourcePos::new("<validation>", 1, 1),
                    message,
                ));
            }
        }
    }

    resolved
        .into_iter()
        .map(|(name, per_param)| {
            let capacities = per_param
                .into_iter()
                .map(|axes| axes.into_iter().map(|axis| axis.unwrap_or(0)).collect())
                .collect();
            (name, capacities)
        })
        .collect()
}

fn collect_globals(body: &[Stmt]) -> HashSet<String> {
    let mut globals = HashSet::new();
    for stmt in body {
        match &stmt.kind {
            Statement::GlobalDecl(ident) => {
                globals.insert(ident.as_basic().to_ascii_lowercase());
            }
            Statement::If {
                then_body,
                else_body,
                ..
            } => {
                globals.extend(collect_globals(then_body));
                globals.extend(collect_globals(else_body));
            }
            Statement::For { body, .. }
            | Statement::While { body, .. }
            | Statement::Do { body, .. } => {
                globals.extend(collect_globals(body));
            }
            Statement::SelectCase {
                cases, else_body, ..
            } => {
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

/// Collect the lowercase BASIC name of every record/file FIELD buffer
/// variable in the whole program (top-level statements and every function
/// body). `records::lower` always names a given file/field pair's buffer
/// identically everywhere it's referenced, so this set is exactly the set
/// of names `ident()` must resolve to their bare global form, no matter
/// which function/procedure body an LSET/GET/PUT referencing one appears in.
fn collect_record_buffer_names(program: &Program) -> HashSet<String> {
    let mut names = HashSet::new();
    collect_record_buffer_names_in(&program.statements, &mut names);
    for func in &program.functions {
        collect_record_buffer_names_in(&func.body, &mut names);
    }
    names
}

fn collect_record_buffer_names_in(stmts: &[Stmt], names: &mut HashSet<String>) {
    for stmt in stmts {
        match &stmt.kind {
            Statement::Field { fields, .. } => {
                for (_, var) in fields {
                    names.insert(var.as_basic().to_ascii_lowercase());
                }
            }
            Statement::If {
                then_body,
                else_body,
                ..
            } => {
                collect_record_buffer_names_in(then_body, names);
                collect_record_buffer_names_in(else_body, names);
            }
            Statement::For { body, .. }
            | Statement::While { body, .. }
            | Statement::Do { body, .. } => {
                collect_record_buffer_names_in(body, names);
            }
            Statement::SelectCase {
                cases, else_body, ..
            } => {
                for case in cases {
                    collect_record_buffer_names_in(&case.body, names);
                }
                collect_record_buffer_names_in(else_body, names);
            }
            _ => {}
        }
    }
}

/// Name (sans type suffix) of the require-able `com.bascal.stdlib.
/// midAssign` helper function `Statement::MidAssign` transpiles into a call
/// to -- shared with `lib::inject_mid_assign_helper_if_used`, which
/// resolves and auto-injects it, so the two can't drift out of sync.
pub(crate) const MID_ASSIGN_HELPER_NAME: &str = "midAssign";

/// `BasicIdent` for `com.bascal.stdlib.midAssign`. Case-insensitive lookup
/// (`same_ident`/`function_info`) means this matches the function
/// regardless of how its own source spells the name.
fn mid_assign_helper_ident() -> BasicIdent {
    BasicIdent::parse(&format!("{MID_ASSIGN_HELPER_NAME}$"))
}

/// Returns a `BasicIdent` whose BASIC form is not present in `taken`.
/// Always uses the indexed form `preferredStem0`, `1`, … so that allocated
/// names are visually distinct from bare global names and can never coincide
/// with an unindexed global even if no collision exists today.
/// A short, alphanumeric-only tag distinguishing one type suffix from
/// another in a generated GOSUB label (see `function_label`) -- `allocate_unique`
/// already keeps ordinary variable names apart this way via the real BASIC
/// suffix character embedded in `BasicIdent::as_basic()`'s own rendering,
/// but a label is plain emitted text (`self.line(&format!("{}:", info.label))`
/// in `emit_function_def`), not a `BasicIdent`, so it needs its own safe
/// (non-`%`/`$`/`!`/`#`/`&`) tag instead.
fn label_suffix_tag(suffix: TypeSuffix) -> &'static str {
    match suffix {
        TypeSuffix::Integer => "i",
        TypeSuffix::Long => "l",
        TypeSuffix::Single => "f",
        TypeSuffix::Double => "d",
        TypeSuffix::String => "s",
    }
}

/// The GOSUB label a function/procedure/method's own body starts at --
/// keyed by `stem` (the base name alone) *plus* the result suffix and (for
/// a method) the receiver, so two functions/methods that only differ by
/// suffix and/or receiver never collide on the same label. Two ordinary
/// functions differing only by suffix (`function foo%(x%)` / `function
/// foo$(x%)`) used to collide this way -- both got `FN_foo`, and whichever
/// claimed it last silently won every call site, with no diagnostic at
/// all (confirmed via real `fbc` execution: `print foo%(5)` actually ran
/// `foo$`'s body). A procedure has no result suffix at all
/// (`function.name.suffix` is `None`), so its label is keyed on the bare
/// stem alone -- safe, since `reject_duplicate_functions` already rejects
/// two procedures (or a procedure and a function, which real BASIC's own
/// name resolution can't tell apart either) sharing one name.
fn function_label(stem: &str, suffix: Option<TypeSuffix>, receiver: Option<TypeSuffix>) -> String {
    let mut label = format!("FN_{stem}");
    if let Some(suffix) = suffix {
        label.push('_');
        label.push_str(label_suffix_tag(suffix));
    }
    if let Some(receiver) = receiver {
        label.push_str("_of_");
        label.push_str(label_suffix_tag(receiver));
    }
    label
}

fn allocate_unique(
    preferred_stem: &str,
    suffix: Option<TypeSuffix>,
    taken: &HashSet<String>,
) -> BasicIdent {
    for i in 0u32.. {
        let candidate = BasicIdent {
            name: format!("{preferred_stem}{i}"),
            suffix,
        };
        if !taken.contains(&candidate.as_basic().to_ascii_lowercase()) {
            return candidate;
        }
    }
    unreachable!("allocate_unique exhausted u32 candidates")
}

/// Collect the lowercase BASIC form of every variable name used at global
/// (program-level) scope, plus any names declared as `global` inside
/// functions.  This forms the initial "taken" set before function params and
/// results are allocated.
fn collect_program_names(program: &Program) -> HashSet<String> {
    let mut names = HashSet::new();
    collect_names_from_stmts(&program.statements, &mut names);
    for block in &program.common {
        for var in &block.vars {
            names.insert(var.name.as_basic().to_ascii_lowercase());
        }
    }
    for func in &program.functions {
        collect_global_decl_names(&func.body, &mut names);
    }
    names
}

fn collect_names_from_stmts(stmts: &[Stmt], names: &mut HashSet<String>) {
    for stmt in stmts {
        collect_names_from_stmt(stmt, names);
    }
}

fn collect_names_from_stmt(stmt: &Stmt, names: &mut HashSet<String>) {
    match &stmt.kind {
        Statement::Assignment { target, value } => {
            collect_names_from_expr(target, names);
            collect_names_from_expr(value, names);
        }
        Statement::MidAssign {
            target,
            start,
            len,
            value,
        } => {
            collect_names_from_expr(target, names);
            collect_names_from_expr(start, names);
            if let Some(e) = len {
                collect_names_from_expr(e, names);
            }
            collect_names_from_expr(value, names);
        }
        Statement::Dim { name, .. } => {
            names.insert(name.as_basic().to_ascii_lowercase());
        }
        Statement::Const { name, value } => {
            names.insert(name.as_basic().to_ascii_lowercase());
            collect_names_from_expr(value, names);
        }
        Statement::Print { tokens } | Statement::Lprint(tokens) => {
            for t in tokens {
                if let PrintToken::Expr(e) = t {
                    collect_names_from_expr(e, names);
                }
            }
        }
        Statement::PrintUsing { format, tokens } | Statement::LprintUsing { format, tokens } => {
            collect_names_from_expr(format, names);
            for t in tokens {
                if let PrintToken::Expr(e) = t {
                    collect_names_from_expr(e, names);
                }
            }
        }
        Statement::PrintFile { channel, tokens } => {
            collect_names_from_expr(channel, names);
            for t in tokens {
                if let PrintToken::Expr(e) = t {
                    collect_names_from_expr(e, names);
                }
            }
        }
        Statement::PrintFileUsing {
            channel,
            format,
            tokens,
        } => {
            collect_names_from_expr(channel, names);
            collect_names_from_expr(format, names);
            for t in tokens {
                if let PrintToken::Expr(e) = t {
                    collect_names_from_expr(e, names);
                }
            }
        }
        Statement::If {
            condition,
            then_body,
            else_body,
        } => {
            collect_names_from_expr(condition, names);
            collect_names_from_stmts(then_body, names);
            collect_names_from_stmts(else_body, names);
        }
        Statement::For {
            var,
            start,
            end,
            step,
            body,
        } => {
            names.insert(var.as_basic().to_ascii_lowercase());
            collect_names_from_expr(start, names);
            collect_names_from_expr(end, names);
            if let Some(s) = step {
                collect_names_from_expr(s, names);
            }
            collect_names_from_stmts(body, names);
        }
        Statement::While { condition, body } => {
            collect_names_from_expr(condition, names);
            collect_names_from_stmts(body, names);
        }
        Statement::Do {
            condition,
            body,
            post_condition,
        } => {
            if let Some(c) = condition {
                collect_names_from_expr(&c.expr, names);
            }
            collect_names_from_stmts(body, names);
            if let Some(c) = post_condition {
                collect_names_from_expr(&c.expr, names);
            }
        }
        Statement::SelectCase {
            expr,
            cases,
            else_body,
        } => {
            collect_names_from_expr(expr, names);
            for case in cases {
                for v in &case.values {
                    match v {
                        CaseValue::Single(e) | CaseValue::Is { value: e, .. } => {
                            collect_names_from_expr(e, names);
                        }
                        CaseValue::Range { from, to } => {
                            collect_names_from_expr(from, names);
                            collect_names_from_expr(to, names);
                        }
                    }
                }
                collect_names_from_stmts(&case.body, names);
            }
            collect_names_from_stmts(else_body, names);
        }
        Statement::TryCatch {
            try_body,
            catch,
            finally_body,
        } => {
            collect_names_from_stmts(try_body, names);
            if let Some(catch) = catch {
                names.insert(catch.err_var.as_basic().to_ascii_lowercase());
                names.insert(catch.erl_var.as_basic().to_ascii_lowercase());
                collect_names_from_stmts(&catch.body, names);
            }
            collect_names_from_stmts(finally_body, names);
        }
        Statement::ExprStmt(e) => collect_names_from_expr(e, names),
        Statement::Return { value } => collect_names_from_expr(value, names),
        Statement::Input { vars, .. } | Statement::Read(vars) => {
            for e in vars {
                collect_names_from_expr(e, names);
            }
        }
        Statement::InputFile { channel, vars } => {
            collect_names_from_expr(channel, names);
            for e in vars {
                collect_names_from_expr(e, names);
            }
        }
        Statement::Data(values) => {
            for e in values {
                collect_names_from_expr(e, names);
            }
        }
        Statement::Open { file, channel, .. } => {
            collect_names_from_expr(file, names);
            collect_names_from_expr(channel, names);
        }
        Statement::FileDecl { .. } => {
            unreachable!("record/file DSL must be lowered before codegen")
        }
        Statement::Close { channel } => collect_names_from_expr(channel, names),
        Statement::LineInput { channel, target } => {
            collect_names_from_expr(channel, names);
            collect_names_from_expr(target, names);
        }
        Statement::Write { channel, exprs } => {
            collect_names_from_expr(channel, names);
            for e in exprs {
                collect_names_from_expr(e, names);
            }
        }
        Statement::Field {
            channel, fields, ..
        } => {
            collect_names_from_expr(channel, names);
            for (w, v) in fields {
                collect_names_from_expr(w, names);
                names.insert(v.as_basic().to_ascii_lowercase());
            }
        }
        Statement::Get {
            channel,
            record,
            var,
            ..
        }
        | Statement::Put {
            channel,
            record,
            var,
            ..
        } => {
            collect_names_from_expr(channel, names);
            if let Some(e) = record {
                collect_names_from_expr(e, names);
            }
            if let Some(e) = var {
                collect_names_from_expr(e, names);
            }
        }
        Statement::Lset { var, value } | Statement::Rset { var, value } => {
            names.insert(var.as_basic().to_ascii_lowercase());
            collect_names_from_expr(value, names);
        }
        Statement::Seek { channel, position } => {
            collect_names_from_expr(channel, names);
            collect_names_from_expr(position, names);
        }
        Statement::Locate { row, col } => {
            collect_names_from_expr(row, names);
            collect_names_from_expr(col, names);
        }
        Statement::Color { fg, bg } => {
            collect_names_from_expr(fg, names);
            if let Some(e) = bg {
                collect_names_from_expr(e, names);
            }
        }
        Statement::Poke { address, value } => {
            collect_names_from_expr(address, names);
            collect_names_from_expr(value, names);
        }
        Statement::Out { port, value } => {
            collect_names_from_expr(port, names);
            collect_names_from_expr(value, names);
        }
        Statement::Width { channel, cols } => {
            if let Some(c) = channel {
                collect_names_from_expr(c, names);
            }
            collect_names_from_expr(cols, names);
        }
        Statement::Swap(a, b) => {
            collect_names_from_expr(a, names);
            collect_names_from_expr(b, names);
        }
        Statement::Randomize(e) => {
            if let Some(e) = e {
                collect_names_from_expr(e, names);
            }
        }
        Statement::OnBranch { expr, targets, .. } => {
            collect_names_from_expr(expr, names);
            for t in targets {
                collect_names_from_expr(t, names);
            }
        }
        Statement::OnErrorGoto { target } | Statement::ErrorStmt { code: target } => {
            collect_names_from_expr(target, names);
        }
        Statement::Goto(e) | Statement::Gosub(e) | Statement::Restore(Some(e)) => {
            collect_names_from_expr(e, names);
        }
        Statement::Resume(kind) => {
            if let ResumeTarget::Line(e) = kind {
                collect_names_from_expr(e, names);
            }
        }
        Statement::OptionBase(e) => collect_names_from_expr(e, names),
        Statement::Kill { file } => collect_names_from_expr(file, names),
        Statement::Name { from, to } => {
            collect_names_from_expr(from, names);
            collect_names_from_expr(to, names);
        }
        Statement::GlobalDecl(ident) => {
            names.insert(ident.as_basic().to_ascii_lowercase());
        }
        Statement::Erase(_)
        | Statement::End
        | Statement::Stop
        | Statement::Cls
        | Statement::Beep
        | Statement::Clear
        | Statement::System
        | Statement::Exit
        | Statement::Restore(None)
        | Statement::ReturnVoid
        | Statement::Raw(_)
        | Statement::BlockComment(_)
        | Statement::Label(_)
        | Statement::BlankLine => {}
    }
}

fn collect_names_from_expr(expr: &Expr, names: &mut HashSet<String>) {
    match expr {
        Expr::Ident(ident) => {
            names.insert(ident.as_basic().to_ascii_lowercase());
        }
        Expr::ArrayRef { name, indices } => {
            names.insert(name.as_basic().to_ascii_lowercase());
            for i in indices {
                collect_names_from_expr(i, names);
            }
        }
        Expr::Call { args, .. } => {
            for a in args {
                collect_names_from_expr(a, names);
            }
        }
        Expr::Unary { expr, .. } => collect_names_from_expr(expr, names),
        Expr::Binary { left, right, .. } => {
            collect_names_from_expr(left, names);
            collect_names_from_expr(right, names);
        }
        Expr::Integer(_) | Expr::Float(_) | Expr::HexLit(_) | Expr::String(_) => {}
        Expr::FileIndex { .. } | Expr::FieldAccess { .. } | Expr::MethodCall { .. }
        | Expr::RecordLit { .. } => {
            unreachable!("record/file DSL must be lowered before codegen")
        }
        Expr::ScalarMethodCall { base, args, .. } => {
            collect_names_from_expr(base, names);
            for arg in args { collect_names_from_expr(arg, names); }
        }
    }
}

/// Collect names from `GlobalDecl` statements anywhere in a function body.
/// These become global-scope names in the emitted BASIC, so they must be
/// excluded from the "taken" set to avoid double-counting but we still need
/// the bare name reserved.
fn collect_global_decl_names(body: &[Stmt], names: &mut HashSet<String>) {
    for stmt in body {
        match &stmt.kind {
            Statement::GlobalDecl(ident) => {
                names.insert(ident.as_basic().to_ascii_lowercase());
            }
            Statement::If {
                then_body,
                else_body,
                ..
            } => {
                collect_global_decl_names(then_body, names);
                collect_global_decl_names(else_body, names);
            }
            Statement::For { body, .. }
            | Statement::While { body, .. }
            | Statement::Do { body, .. } => {
                collect_global_decl_names(body, names);
            }
            Statement::SelectCase {
                cases, else_body, ..
            } => {
                for case in cases {
                    collect_global_decl_names(&case.body, names);
                }
                collect_global_decl_names(else_body, names);
            }
            _ => {}
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

/// Bound for one axis of an array argument: the arguments immediately
/// following the array argument are its per-axis element counts, in the
/// same order as `DIM`'s own bounds -- `axis` 0 is the first of these.
/// Nested copy loop, one FOR per axis, innermost body doing the actual
/// element assignment. `bounds` and `loop_vars` are parallel, one entry per
/// dimension -- rank 1 (the common case) produces exactly the same output
/// as the original single-loop version.
fn array_copy_lines(
    destination: &str,
    source: &str,
    bounds: &[String],
    comment: &str,
    loop_vars: &[String],
) -> Vec<String> {
    let rank = loop_vars.len();
    let mut lines = vec![
        String::new(),
        format!("' {comment}: {source}() -> {destination}()"),
    ];
    for (level, (var, bound)) in loop_vars.iter().zip(bounds.iter()).enumerate() {
        lines.push(format!("{}FOR {var} = 0 TO {bound}", "    ".repeat(level)));
    }
    let index_list = loop_vars.join(", ");
    lines.push(format!(
        "{}{destination}({index_list}) = {source}({index_list})",
        "    ".repeat(rank)
    ));
    for (level, var) in loop_vars.iter().enumerate().rev() {
        lines.push(format!("{}NEXT {var}", "    ".repeat(level)));
    }
    lines.push(String::new());
    lines
}

pub(crate) fn sanitize_symbol(value: &str) -> String {
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

/// Joins identifier fragments into one camelCase symbol, with no separator
/// -- BASIC is case-insensitive, but camelCase reads far better in generated
/// output than an underscore chain, and (for real MBASIC/BASCOM targets)
/// underscores in an identifier that's read as an expression operand are a
/// hard compile error, not just a style choice. The first non-empty
/// fragment is lowercased in full; every fragment after that only has its
/// own first character forced to uppercase, with the rest left exactly as
/// given -- never force-lowercased. That matters because a later fragment
/// is sometimes itself an already-camelCased compound built by an earlier
/// `camel_join` call (e.g. records.rs building `sName` before handing it
/// to codegen's own `ident()`, which joins it onto a function stem); force-
/// lowercasing the remainder would flatten `sName` into `Sname`, silently
/// erasing the word boundary it already carried. Non-alphanumeric
/// characters are dropped, since BASIC identifiers are letters and digits
/// only. Collision-freedom is still guaranteed the same way it always was
/// -- by `allocate_unique` checking the result against `taken`, not by
/// this function.
pub(crate) fn camel_join(parts: &[&str]) -> String {
    let mut out = String::new();
    for part in parts {
        let clean: String = part.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
        if clean.is_empty() {
            continue;
        }
        if out.is_empty() {
            out.push_str(&clean.to_ascii_lowercase());
        } else {
            let mut chars = clean.chars();
            if let Some(first) = chars.next() {
                out.push(first.to_ascii_uppercase());
                out.push_str(chars.as_str());
            }
        }
    }
    out
}

/// Flattens a left-associated `&&`/`||` chain (as built by
/// `Parser::parse_condition`) into its operands, left to right.
fn flatten_chain<'a>(expr: &'a Expr, op: BinaryOp, out: &mut Vec<&'a Expr>) {
    match expr {
        Expr::Binary { left, op: o, right } if *o == op => {
            flatten_chain(left, op, out);
            out.push(right);
        }
        _ => out.push(expr),
    }
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
        BinaryOp::AndAnd | BinaryOp::OrOr => {
            unreachable!(
                "&&/|| only valid as an if/while/do condition chain — codegen bug if reached here"
            )
        }
    }
}

fn escape_string(value: &str) -> String {
    value.replace('"', "\"\"")
}

fn ends_with_end(statements: &[Stmt]) -> bool {
    statements
        .iter()
        .rev()
        .find(|s| !matches!(&***s, Statement::BlankLine))
        .is_some_and(|s| matches!(&**s, Statement::End))
}

fn ends_with_return(statements: &[Stmt]) -> bool {
    statements
        .iter()
        .rev()
        .find(|s| !matches!(&***s, Statement::BlankLine))
        .is_some_and(|s| matches!(&**s, Statement::Return { .. } | Statement::ReturnVoid))
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
        // Comment lines are user text, not code — never rewrite label words
        // inside them, even if a label name happens to appear as an ordinary
        // word in the comment.
        if !text.trim_start().starts_with('\'') {
            for (label, number) in &label_numbers {
                text = replace_label_word(&text, label, &number.to_string());
            }
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
        Expr::Ident(ident)
        | Expr::Call { name: ident, .. }
        | Expr::ArrayRef { name: ident, .. } => match ident.suffix {
            Some(TypeSuffix::String) => "$",
            Some(TypeSuffix::Single) => "!",
            Some(TypeSuffix::Double) => "#",
            Some(TypeSuffix::Long) => "&",
            _ => "%",
        },
        Expr::HexLit(_) => "%",
        Expr::Unary { expr, .. } => expr_type_suffix(expr),
        Expr::Binary { left, .. } => expr_type_suffix(left),
        Expr::FileIndex { .. }
        | Expr::FieldAccess { .. }
        | Expr::MethodCall { .. }
        | Expr::ScalarMethodCall { .. }
        | Expr::RecordLit { .. } => {
            unreachable!("record/file DSL must be lowered before codegen")
        }
    }
}

/// True for any generated line that is *only* a label declaration: either a
/// transpiler-internal control-flow label (`IF_0004_END:`, `WHILE_0002_TOP:`,
/// ...) or a user-written `name:` label from BASCAL source (`Statement::Label`).
/// Both kinds are resolved to real BASIC line numbers by `number_basic_lines`
/// and then dropped from the output — codegen never emits any other line
/// that is nothing but an identifier followed by a colon.
fn is_label_line(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    let label = trimmed.strip_suffix(':')?;
    if !label.is_empty() && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        Some(label)
    } else {
        None
    }
}

/// Replace whole-word occurrences of `label` in `text` with `replacement`,
/// skipping anything inside a `"..."` string literal. A plain `str::replace`
/// would also match `label` as a substring of an unrelated longer identifier,
/// or inside program output text (e.g. a label named `done` corrupting
/// `PRINT "done"`) — user-chosen label names are short, ordinary words, so
/// that collision is a real risk in a way it never was for the transpiler's
/// own distinctively-prefixed internal labels.
fn replace_label_word(text: &str, label: &str, replacement: &str) -> String {
    if label.is_empty() {
        return text.to_string();
    }
    let is_ident_char = |c: char| c.is_ascii_alphanumeric() || c == '_';
    let mut out = String::with_capacity(text.len());
    let mut in_string = false;
    let mut i = 0;
    while i < text.len() {
        let ch = text[i..].chars().next().unwrap();
        if ch == '"' {
            in_string = !in_string;
            out.push(ch);
            i += 1;
            continue;
        }
        if !in_string && text[i..].starts_with(label) {
            let before_ok = i == 0 || !is_ident_char(text[..i].chars().next_back().unwrap());
            let after_idx = i + label.len();
            let after_ok = after_idx >= text.len()
                || !is_ident_char(text[after_idx..].chars().next().unwrap());
            if before_ok && after_ok {
                out.push_str(replacement);
                i = after_idx;
                continue;
            }
        }
        let ch_len = ch.len_utf8();
        out.push_str(&text[i..i + ch_len]);
        i += ch_len;
    }
    out
}

/// Pre-generation validation: report every global variable whose name matches
/// a transpiler-generated local name (`stem_var_0suffix`), which would silently
/// produce a BASIC program with two distinct roles sharing the same identifier.
///
/// Checks the result variable, every parameter, and every local variable
/// reference in each function body.
pub(crate) fn check_generated_name_conflicts(program: &Program) -> Vec<Diagnostic> {
    let globals = collect_program_names(program);

    // Names the transpiler will never treat as locals.
    let builtin_stems: HashSet<&str> = BASIC_BUILTINS.iter().copied().collect();
    let function_stems: HashSet<String> = program
        .functions
        .iter()
        .map(|f| sanitize_symbol(&f.name.name))
        .collect();

    let mut diagnostics = Vec::new();

    for func in &program.functions {
        let stem = sanitize_symbol(&func.name.name);
        let param_keys: HashSet<String> = func
            .params
            .iter()
            .map(|p| p.name.as_basic().to_ascii_lowercase())
            .collect();
        let global_decls = collect_globals(&func.body);

        // ── result variable ────────────────────────────────────────────────
        check_one_conflict(
            &globals,
            &stem,
            "result",
            func.name.suffix,
            &func.name,
            &format!("result variable for `{}`", func.name.as_basic()),
            &mut diagnostics,
        );

        // ── parameters ────────────────────────────────────────────────────
        for param in &func.params {
            check_one_conflict(
                &globals,
                &stem,
                &sanitize_symbol(&param.name.name),
                param.name.suffix,
                &func.name,
                &format!(
                    "parameter `{}` of `{}`",
                    param.name.as_basic(),
                    func.name.as_basic()
                ),
                &mut diagnostics,
            );
        }

        // ── locals referenced in the body ──────────────────────────────────
        let mut body_names: HashSet<String> = HashSet::new();
        collect_names_from_stmts(&func.body, &mut body_names);

        for key in &body_names {
            if param_keys.contains(key) || global_decls.contains(key) {
                continue;
            }
            let local = BasicIdent::parse(key);
            let bare = local.name.to_ascii_lowercase();
            if builtin_stems.contains(bare.as_str()) || function_stems.contains(&bare) {
                continue;
            }
            check_one_conflict(
                &globals,
                &stem,
                &sanitize_symbol(&local.name),
                local.suffix,
                &func.name,
                &format!(
                    "local variable `{}` in `{}`",
                    local.as_basic(),
                    func.name.as_basic()
                ),
                &mut diagnostics,
            );
        }
    }

    diagnostics
}

fn check_one_conflict(
    globals: &HashSet<String>,
    stem: &str,
    var_stem: &str,
    suffix: Option<TypeSuffix>,
    func_name: &BasicIdent,
    description: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let candidate = BasicIdent {
        name: format!("{}0", camel_join(&[stem, var_stem])),
        suffix,
    };
    if globals.contains(&candidate.as_basic().to_ascii_lowercase()) {
        diagnostics.push(Diagnostic::error(
            SourcePos::new("<validation>", 1, 1),
            format!(
                "global `{}` conflicts with the transpiler-generated name for {}; \
                 rename the global or the function `{}`",
                candidate.as_basic(),
                description,
                func_name.as_basic(),
            ),
        ));
    }
}
