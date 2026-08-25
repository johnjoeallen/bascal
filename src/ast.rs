use std::fmt;
use std::ops::Deref;

use crate::diagnostics::SourcePos;

/// A parsed statement together with the source position of its first
/// token -- populated once, at `Parser::parse_statement`'s single dispatch
/// point (see parser.rs), so every diagnostic that complains about a
/// statement can point at a real `file:line:column` instead of a
/// placeholder. `Deref`s to the wrapped `Statement` so call sites that only
/// care about the statement's shape can keep matching on it directly
/// (`match &stmt.kind { .. }` or, via `Deref`, `match &*stmt { .. }`)
/// without threading position through every helper by hand.
#[derive(Debug, Clone, PartialEq)]
pub struct Stmt {
    pub kind: Statement,
    pub pos: SourcePos,
}

impl Stmt {
    pub fn new(kind: Statement, pos: SourcePos) -> Self {
        Self { kind, pos }
    }

    pub fn pos(&self) -> &SourcePos {
        &self.pos
    }
}

impl Deref for Stmt {
    type Target = Statement;

    fn deref(&self) -> &Statement {
        &self.kind
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub program_decl: Option<ProgramDecl>,
    /// `library <name>` — present when this file declares itself a library
    /// module (only such files may be `require`d/`import`ed; see
    /// load_program_recursive in lib.rs).
    pub library_decl: Option<String>,
    /// `shared <name>` — present when this file declares *itself* to be the
    /// shared-variables definition named `<name>` (see load_shared_file in
    /// lib.rs). Distinct from `ProgramDecl.shared`, which is a *reference*
    /// to one by name from an ordinary program.
    pub shared_decl: Option<String>,
    pub declarations: Vec<DependencyDecl>,
    /// COMMON blocks to emit, built by lib.rs from a `shared <name>` file's
    /// `dim` declarations -- BASCAL source itself has no `common` keyword;
    /// this is purely the internal representation codegen emits as `COMMON`
    /// lines.
    pub common: Vec<CommonBlock>,
    pub statements: Vec<Stmt>,
    pub functions: Vec<FunctionDef>,
    pub records: Vec<RecordDef>,
    /// Typed array declarations preserved for backends that need rank and
    /// element information after source-level lowering.
    pub typed_arrays: Vec<TypedArrayDecl>,
    pub typed_array_refs: Vec<TypedArrayRef>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypedArrayDecl {
    pub name: BasicIdent,
    pub element_suffix: Option<TypeSuffix>,
    pub dimensions: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypedArrayRef {
    pub name: BasicIdent,
    pub element_suffix: Option<TypeSuffix>,
    pub dimensions: usize,
    pub indices: Vec<Expr>,
    pub sizeof_axis: Option<usize>,
}

/// A `record ... end record` declaration: a fixed-layout struct that the
/// `records` lowering pass (src/records.rs) desugars into `FIELD`-bound
/// buffer variables and MKx$/CVx$ packing calls before codegen ever runs.
#[derive(Debug, Clone, PartialEq)]
pub struct RecordDef {
    pub name: String,
    pub fields: Vec<RecordFieldDef>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordFieldDef {
    pub name: String,
    pub ty: RecordFieldType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordFieldType {
    Int16,
    Int32,
    Float32,
    Float64,
    Str(u32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramDecl {
    pub name: String,
    pub shared: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommonBlock {
    pub vars: Vec<CommonVar>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommonVar {
    pub name: BasicIdent,
    pub is_array: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DependencyDecl {
    Require(PathSymbol),
    Import(PathSymbol),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathSymbol {
    pub raw: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDef {
    pub name: BasicIdent,
    pub params: Vec<Param>,
    pub body: Vec<Stmt>,
    pub is_procedure: bool,
    /// The scalar receiver type for a `method$`/`method%` declaration.
    /// Methods otherwise share function bodies and return syntax.
    pub receiver: Option<TypeSuffix>,
    /// Source position of the `function`/`procedure` keyword that starts
    /// this declaration -- lets resolver diagnostics about the function as
    /// a whole (duplicate name, shadowing a builtin, missing return,
    /// recursion, unsafe error handler) point at its declaration instead of
    /// a placeholder position.
    pub pos: SourcePos,
}

/// A declared parameter and its passing mode. Unmarked source (`arr%`)
/// parses as `ByVal`; `byref arr%` / `byval arr%` parse explicitly.
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: BasicIdent,
    pub mode: ParamMode,
    /// An optional fixed value supplied when a caller omits this trailing
    /// scalar parameter. Validation restricts defaults to literals or
    /// top-level constants.
    pub default: Option<Expr>,
    /// Declared array axes: `None` for a scalar parameter (bare name, no
    /// parens). `Some(axes)` for an array parameter, one entry per
    /// dimension -- `name(?)` is `Some(vec![None])`, `name(?, ?)` is
    /// `Some(vec![None, None])`, etc. An array parameter's rank must be
    /// declared this way; there is no way to declare an array parameter
    /// without it.
    ///
    /// Each axis entry is itself `None` (`?`, meaning: infer this
    /// parameter's storage capacity along this axis automatically, from
    /// the largest array ever passed here across every call site) or
    /// `Some(n)` (an explicit literal capacity, written because at least
    /// one call site's array size isn't a compile-time constant, so
    /// automatic inference can't produce a safe number).
    pub axes: Option<Vec<Option<i64>>>,
}

impl Param {
    /// Declared array rank -- `None` for a scalar, `Some(n)` for an
    /// n-dimensional array parameter, regardless of whether each axis is
    /// `?` (inferred) or an explicit literal.
    pub fn rank(&self) -> Option<usize> {
        self.axes.as_ref().map(|axes| axes.len())
    }
}

/// `ByVal` copies the argument in before the call and never writes it back.
/// `ByRef` copies in before the call *and* writes the result back to the
/// caller's variable/array after — for both scalar and array parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ParamMode {
    #[default]
    ByVal,
    ByRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BasicIdent {
    pub name: String,
    pub suffix: Option<TypeSuffix>,
}

impl BasicIdent {
    pub fn parse(raw: &str) -> Self {
        let mut chars = raw.chars();
        let suffix = chars.next_back().and_then(TypeSuffix::from_char);
        let name = if suffix.is_some() {
            raw[..raw.len() - 1].to_string()
        } else {
            raw.to_string()
        };
        Self { name, suffix }
    }

    pub fn as_basic(&self) -> String {
        match self.suffix {
            Some(suffix) => format!("{}{}", self.name, suffix),
            None => self.name.clone(),
        }
    }
}

impl fmt::Display for BasicIdent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.as_basic())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypeSuffix {
    Integer,
    String,
    Single,
    Double,
    Long,
}

impl TypeSuffix {
    pub fn from_char(ch: char) -> Option<Self> {
        match ch {
            '%' => Some(Self::Integer),
            '$' => Some(Self::String),
            '!' => Some(Self::Single),
            '#' => Some(Self::Double),
            '&' => Some(Self::Long),
            _ => None,
        }
    }
}

impl fmt::Display for TypeSuffix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let suffix = match self {
            Self::Integer => "%",
            Self::String => "$",
            Self::Single => "!",
            Self::Double => "#",
            Self::Long => "&",
        };
        f.write_str(suffix)
    }
}

/// One item in a PRINT / LPRINT / PRINT# argument list.
/// Semi and Comma are separators (`;` suppresses the trailing newline;
/// `,` advances to the next 14-char tab zone and suppresses the newline).
/// A trailing Semi or Comma as the last token suppresses the newline.
#[derive(Debug, Clone, PartialEq)]
pub enum PrintToken {
    Expr(Expr),
    Semi,
    Comma,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResumeTarget {
    Same,       // RESUME — retry the statement that caused the error
    Next,       // RESUME NEXT — continue after the failing statement
    Line(Expr), // RESUME lineno
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Dim {
        name: BasicIdent,
        /// True whenever `(...)` was written at all, even empty (`dim
        /// arr%()`) -- distinct from `sizes` being empty, which alone can't
        /// tell an unbounded array apart from a plain scalar `dim x%`.
        is_array: bool,
        sizes: Vec<Expr>,
    },
    Open {
        mode: OpenMode,
        file: Expr,
        channel: Expr,
        len: Option<Expr>,
    },
    /// `file <var> as <RecordType> = open(<path>)` (`record_type: Some`,
    /// `mode: None`) or `file <var> = open(<path>) for input/output/append`
    /// (`record_type: None`, `mode: Some`) — high-level record/file DSL
    /// sugar, either the random-access record form or the plain sequential
    /// file-handle form. Always eliminated by `records::lower` before
    /// codegen runs; see src/records.rs.
    FileDecl {
        var: BasicIdent,
        record_type: Option<String>,
        path: Expr,
        mode: Option<OpenMode>,
    },
    LineInput {
        channel: Expr,
        target: Expr,
    },
    PrintFile {
        channel: Expr,
        tokens: Vec<PrintToken>,
    },
    PrintUsing {
        format: Expr,
        tokens: Vec<PrintToken>,
    },
    PrintFileUsing {
        channel: Expr,
        format: Expr,
        tokens: Vec<PrintToken>,
    },
    Close {
        channel: Expr,
    },
    Kill {
        file: Expr,
    },
    Name {
        from: Expr,
        to: Expr,
    },
    Assignment {
        target: Expr,
        value: Expr,
    },
    /// `MID$(<target>, <start>[, <len>]) = <value>` — statement-form MID$
    /// assignment: overwrites a run of characters inside `target` with a
    /// same-length replacement (transpiled to a call that rebuilds and
    /// reassigns `target`, not a byte-level in-place mutation), and is not
    /// itself a value-producing expression. `target` is always `Expr::Ident` or
    /// `Expr::ArrayRef` (a plain string variable or string array element),
    /// enforced at parse time. `len` is `None` for the two-argument form
    /// (`MID$(a$, start) = value`), which real MBASIC/BASCOM treats as if
    /// `len` were `LEN(value)`.
    MidAssign {
        target: Box<Expr>,
        start: Box<Expr>,
        len: Option<Box<Expr>>,
        value: Box<Expr>,
    },
    Print {
        tokens: Vec<PrintToken>,
    },
    Return {
        value: Expr,
    },
    If {
        condition: Expr,
        then_body: Vec<Stmt>,
        else_body: Vec<Stmt>,
    },
    For {
        var: BasicIdent,
        start: Expr,
        end: Expr,
        step: Option<Expr>,
        body: Vec<Stmt>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    Do {
        condition: Option<DoCondition>,
        body: Vec<Stmt>,
        post_condition: Option<DoCondition>,
    },
    ExprStmt(Expr),
    End,
    Stop,
    Cls,
    Beep,
    System,
    OptionBase(Expr),
    Erase(Vec<BasicIdent>),
    Randomize(Option<Expr>),
    Swap(Expr, Expr),
    Poke {
        address: Expr,
        value: Expr,
    },
    /// `name:` — a user-declared branch target. Participates in the same
    /// label -> line-number resolution as transpiler-internal labels; see
    /// `is_label_line` in codegen.rs.
    Label(String),
    Goto(Expr),
    Gosub(Expr),
    OnErrorGoto {
        target: Expr,
    },
    Resume(ResumeTarget),
    /// `try ... [catch err%, erl% ...] [finally ...] end try` -- structured
    /// error recovery and cleanup
    /// (issue #60): a failed statement anywhere in `try_body` abandons the
    /// rest of it and runs the optional `catch` body once, then always runs
    /// `finally_body` before execution continues after `end try` -- never
    /// back inside `try_body`. A catch's `err_var`/`erl_var` are ordinary
    /// locals scoped to its body alone (not
    /// aliases for the ambient `err`/`erl` pseudo-variables `on error
    /// goto` uses), populated once at catch-entry with the error code and
    /// (for the C backend) the real `.bcl` source line the failure raised
    /// from. Unlike `on error goto`, `try`/`catch` offers no `resume`
    /// equivalent at all -- see `tutorial/inventory_try_catch.draft`'s own
    /// header comment for why arbitrary resume-at-the-failure-point
    /// behavior isn't reproduced here.
    TryCatch {
        try_body: Vec<Stmt>,
        catch: Option<TryCatchHandler>,
        finally_body: Vec<Stmt>,
    },
    ErrorStmt {
        code: Expr,
    },
    /// Structured portable raise. Without a code, rethrows the current
    /// error (`ERR` on BASIC, `bcc_err` on C).
    ThrowStmt {
        code: Option<Expr>,
    },
    Input {
        prompt: Option<String>,
        vars: Vec<Expr>,
    },
    InputFile {
        channel: Expr,
        vars: Vec<Expr>,
    },
    Data(Vec<Expr>),
    Read(Vec<Expr>),
    Restore(Option<Expr>),
    Const {
        name: BasicIdent,
        value: Expr,
    },
    Write {
        channel: Expr,
        exprs: Vec<Expr>,
    },
    Field {
        channel: Expr,
        fields: Vec<(Expr, BasicIdent)>,
        /// The record/file DSL's declared record type name (see
        /// `records::lower_file_decl`), when this `FIELD` was synthesized
        /// from `file db as T = open(...)` -- `None` for a raw, hand-typed
        /// `FIELD` statement, which BASCAL still passes through unchanged.
        /// Only the C backend (`codegen_c.rs`) reads this, to name its
        /// synthesized per-record-type `bcc_put<T>Record`/`bcc_get<T>Record`
        /// helpers after the actual record type instead of an anonymous
        /// shape index; the `basic` backend ignores it entirely.
        record_type: Option<String>,
        /// Logical string fields from the record/file DSL.  The C backend
        /// uses this to trim FIELD padding after a record read.
        string_fields: Option<Vec<bool>>,
        /// Exact DSL field types for the C typed-record path. Raw FIELD
        /// statements leave this unset and retain byte-buffer semantics.
        field_types: Option<Vec<RecordFieldType>>,
    },
    Get {
        channel: Expr,
        record: Option<Expr>,
        var: Option<Expr>,
        /// Set only by the record/file DSL for a partial update. Reading a
        /// missing record before changing selected fields would otherwise
        /// turn uninitialized buffer bytes into a newly written record.
        require_existing: bool,
        /// The fixed record length for a DSL partial update. This lets the
        /// BASIC backend reject a record beyond EOF before its GET; raw
        /// GET statements leave it unset.
        record_length: Option<u32>,
    },
    Put {
        channel: Expr,
        record: Option<Expr>,
        var: Option<Expr>,
        /// For a record/file DSL partial update, marks the fields supplied
        /// by the source literal.  The C backend passes `NULL` for omitted
        /// fields so its typed PUT helper can preserve them safely.
        provided_fields: Option<Vec<bool>>,
    },
    Lset {
        var: BasicIdent,
        value: Expr,
    },
    Rset {
        var: BasicIdent,
        value: Expr,
    },
    Seek {
        channel: Expr,
        position: Expr,
    },
    Lprint(Vec<PrintToken>),
    LprintUsing {
        format: Expr,
        tokens: Vec<PrintToken>,
    },
    /// `exit` — leaves the innermost enclosing `for`/`while`/`do` loop.
    /// Unqualified: the transpiler resolves which loop kind it's inside.
    Exit,
    SelectCase {
        expr: Expr,
        cases: Vec<CaseClause>,
        else_body: Vec<Stmt>,
    },
    Locate {
        row: Expr,
        col: Expr,
    },
    Color {
        fg: Expr,
        bg: Option<Expr>,
    },
    OnBranch {
        expr: Expr,
        targets: Vec<Expr>,
        is_gosub: bool,
    },
    Out {
        port: Expr,
        value: Expr,
    },
    Width {
        channel: Option<Expr>,
        cols: Expr,
    },
    Clear,
    ReturnVoid,
    GlobalDecl(BasicIdent),
    Raw(String),
    BlockComment(Vec<String>),
    BlankLine,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TryCatchHandler {
    pub err_var: BasicIdent,
    /// Optional error-code filter attached directly to `err_var`
    /// (`catch err%(53, 76), erl%`, GitHub issue #76): the `catch` body
    /// only runs when the raised error's code matches one of these
    /// (numeric, not necessarily literal) expressions. Empty means
    /// "match anything" -- bare `catch err%, erl%`, unchanged from before
    /// this existed. On no match, `catch`'s body is skipped and the error
    /// auto-rethrows: immediately if there's no `finally`, or after
    /// `finally` runs if there is one -- the same propagation path a bare
    /// `throw` already uses.
    pub error_filter: Vec<Expr>,
    pub erl_var: BasicIdent,
    /// Optional third `catch` binding (`catch err%, erl%, source$`):
    /// the original BASCAL source filename the raising statement came
    /// from (including code pulled in through `require`), never the
    /// generated BASIC/C output's own path. Parser-enforced to be a
    /// plain string variable (`TypeSuffix::String`).
    pub source_var: Option<BasicIdent>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DoCondition {
    pub is_while: bool,
    pub expr: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CaseClause {
    pub values: Vec<CaseValue>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CaseValue {
    Single(Expr),
    Range { from: Expr, to: Expr },
    Is { op: BinaryOp, value: Expr },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenMode {
    Input,
    Output,
    Append,
    Random,
    Binary,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Integer(i64),
    Float(f64),
    HexLit(String),
    String(String),
    Ident(BasicIdent),
    ArrayRef {
        name: BasicIdent,
        indices: Vec<Expr>,
    },
    Call {
        name: BasicIdent,
        args: Vec<Expr>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    /// `db[i]` — record/file DSL sugar. Always eliminated by `records::lower`
    /// before codegen runs; see src/records.rs.
    FileIndex {
        var: BasicIdent,
        index: Box<Expr>,
    },
    /// `s.field` / `db[i].field` — record/file DSL sugar.
    FieldAccess {
        base: Box<Expr>,
        field: String,
    },
    /// `db.close()` — record/file DSL sugar (currently the only recognized
    /// method is `close`; any other method is rejected during lowering).
    MethodCall {
        base: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },
    /// A scalar extension-method call. Kept separate from `MethodCall`,
    /// which is the existing record/file DSL syntax.
    ScalarMethodCall {
        base: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },
    /// `{ field: value, ... }` (every declared field required) or
    /// `?{ field: value, ... }` (a subset is allowed — `partial: true`) —
    /// record/file DSL sugar.
    RecordLit {
        fields: Vec<(String, Expr)>,
        partial: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Xor,
    Mod,
    IntDiv,
    Pow,
    /// `&&` — short-circuit AND. Only ever constructed by
    /// `Parser::parse_condition`, and only ever valid as the top-level
    /// shape (or a same-operator chain) of an `if`/`while`/`do` condition.
    AndAnd,
    /// `||` — short-circuit OR. See `AndAnd`.
    OrOr,
}
