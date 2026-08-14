use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub program_decl: Option<ProgramDecl>,
    /// `suite <name>` — present when this file declares *itself* to be the
    /// suite definition named `<name>`, an alternative to the older
    /// filename-only convention (see load_suite_file in lib.rs). Distinct
    /// from `ProgramDecl.suite`, which is a *reference* to a suite by name
    /// from an ordinary program.
    pub suite_decl: Option<String>,
    pub declarations: Vec<DependencyDecl>,
    pub common: Vec<CommonBlock>,
    pub statements: Vec<Statement>,
    pub functions: Vec<FunctionDef>,
    pub records: Vec<RecordDef>,
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
    pub suite: Option<String>,
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
    pub body: Vec<Statement>,
    pub is_procedure: bool,
}

/// A declared parameter and its passing mode. Unmarked source (`arr%`)
/// parses as `ByVal`; `byref arr%` / `byval arr%` parse explicitly.
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: BasicIdent,
    pub mode: ParamMode,
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
    Same,        // RESUME — retry the statement that caused the error
    Next,        // RESUME NEXT — continue after the failing statement
    Line(Expr),  // RESUME lineno
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
    /// `file <var> as <RecordType> = open(<path>)` — high-level record/file
    /// DSL sugar. Always eliminated by `records::lower` before codegen runs;
    /// see src/records.rs.
    FileDecl {
        var: BasicIdent,
        record_type: String,
        path: Expr,
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
    Print {
        tokens: Vec<PrintToken>,
    },
    Return {
        value: Expr,
    },
    If {
        condition: Expr,
        then_body: Vec<Statement>,
        else_body: Vec<Statement>,
    },
    For {
        var: BasicIdent,
        start: Expr,
        end: Expr,
        step: Option<Expr>,
        body: Vec<Statement>,
    },
    While {
        condition: Expr,
        body: Vec<Statement>,
    },
    Do {
        condition: Option<DoCondition>,
        body: Vec<Statement>,
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
    /// label -> line-number resolution as compiler-internal labels; see
    /// `is_label_line` in codegen.rs.
    Label(String),
    Goto(Expr),
    Gosub(Expr),
    OnErrorGoto { target: Expr },
    Resume(ResumeTarget),
    ErrorStmt { code: Expr },
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
    },
    Get {
        channel: Expr,
        record: Option<Expr>,
        var: Option<Expr>,
    },
    Put {
        channel: Expr,
        record: Option<Expr>,
        var: Option<Expr>,
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
    /// Unqualified: the compiler resolves which loop kind it's inside.
    Exit,
    SelectCase {
        expr: Expr,
        cases: Vec<CaseClause>,
        else_body: Vec<Statement>,
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
pub struct DoCondition {
    pub is_while: bool,
    pub expr: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CaseClause {
    pub values: Vec<CaseValue>,
    pub body: Vec<Statement>,
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
