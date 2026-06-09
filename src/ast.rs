use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub declarations: Vec<DependencyDecl>,
    pub statements: Vec<Statement>,
    pub functions: Vec<FunctionDef>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DependencyDecl {
    Include(String),
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
    pub params: Vec<BasicIdent>,
    pub body: Vec<Statement>,
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

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Dim {
        name: BasicIdent,
        size: Option<Expr>,
    },
    Assignment {
        target: Expr,
        value: Expr,
    },
    Print {
        exprs: Vec<Expr>,
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
    ExprStmt(Expr),
    End,
    Raw(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Integer(i64),
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
}
