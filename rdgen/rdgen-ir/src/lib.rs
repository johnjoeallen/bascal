//! The language-agnostic intermediate representation consumed by rdgen backends.
//!
//! The IR is deliberately independent of both the grammar surface syntax and
//! the generated parser language. In particular, constructor annotations and
//! resolved symbol identities are retained so a backend never has to recover
//! type or AST information from grammar syntax.

use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Grammar {
    pub name: String,
    pub start: String,
    pub rules: Vec<Rule>,
    pub tokens: Vec<Token>,
    pub precedence: Vec<PrecedenceTable>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Token {
    pub name: String,
    pub pattern: String,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Rule {
    pub name: String,
    pub lexical: bool,
    pub output: TypeName,
    pub alternatives: Vec<Alternative>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Alternative {
    pub elements: Vec<Element>,
    pub constructor: Constructor,
    pub recovery: Option<RecoveryPoint>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Element {
    Rule {
        label: Option<String>,
        rule: String,
        span: Span,
    },
    Token {
        label: Option<String>,
        token: String,
        span: Span,
    },
    Literal {
        label: Option<String>,
        value: String,
        span: Span,
    },
    Repeat {
        label: Option<String>,
        element: Box<Element>,
        min: usize,
        max: Option<usize>,
        terminators: Vec<String>,
        span: Span,
    },
    Group {
        label: Option<String>,
        alternatives: Vec<Vec<Element>>,
        span: Span,
    },
    /// Parse the wrapped element only when it remains on the physical line
    /// where parsing starts.  This is primarily useful for optional grammar
    /// elements whose absence is signalled by a newline.
    SameLine {
        element: Box<Element>,
        span: Span,
    },
    /// Parse `atom` and its trailing operators through the precedence table
    /// declared for the enclosing rule, producing that rule's own output
    /// type directly. Must be the sole element of its alternative.
    Climb {
        label: Option<String>,
        atom: String,
        span: Span,
    },
    /// Parse `base` once, then repeatedly parse `step`, each time replacing
    /// `step`'s magic `base`-role constructor field with the value
    /// accumulated so far. `base` and `step` must share an output type.
    Fold {
        label: Option<String>,
        base: String,
        step: String,
        span: Span,
    },
    /// Commit the current ordered-choice alternative after the preceding
    /// discriminating input has matched.
    Cut {
        span: Span,
    },
    /// Consume a physical statement terminator (`:`, newline, or EOF).
    LineEnd {
        span: Span,
    },
    /// Consume one or more physical newlines, but never a `:` separator.
    Newline {
        span: Span,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Constructor {
    pub type_name: TypeName,
    pub fields: Vec<FieldBinding>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldBinding {
    pub field: String,
    pub source_label: String,
    /// When set, this field is not sourced from any parsed element or
    /// magic role: its value is this fixed integer, spliced into the
    /// generated code verbatim. `source_label` is unused (empty) in this
    /// case. Used for values a grammar alternative implies but never
    /// consumes a token for (e.g. `downto`'s implicit step of -1).
    pub constant: Option<i64>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeName(pub String);

impl From<&str> for TypeName {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryPoint {
    pub sync_tokens: Vec<String>,
    pub strategy: RecoveryStrategy,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecoveryStrategy {
    SkipUntilSync,
    InsertToken(String),
    AbortRule,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrecedenceTable {
    pub rule: String,
    pub levels: Vec<PrecedenceLevel>,
    pub prefix_operators: Vec<PrefixOperator>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrecedenceLevel {
    pub operators: Vec<String>,
    pub associativity: Associativity,
    pub constructor: Option<Constructor>,
}

/// A prefix operator whose operand is parsed through the same table's
/// climbing loop, absorbing every operator at or above `binds_below`'s
/// level but leaving weaker operators for the enclosing climb.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrefixOperator {
    pub operators: Vec<String>,
    pub binds_below: String,
    pub constructor: Option<Constructor>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Associativity {
    Left,
    Right,
}

impl PrecedenceTable {
    /// Return the minimum binding powers used by the generated precedence loop.
    /// Levels are ordered from weakest to strongest precedence.
    pub fn binding_powers(&self, operator: &str) -> Option<(usize, usize)> {
        self.levels.iter().enumerate().find_map(|(level, spec)| {
            spec.operators
                .iter()
                .any(|candidate| candidate == operator)
                .then(|| {
                    let binding_power = level * 2 + 1;
                    match spec.associativity {
                        Associativity::Left => (binding_power, binding_power + 1),
                        Associativity::Right => (binding_power, binding_power),
                    }
                })
        })
    }

    /// The minimum binding power a prefix operator's operand-parse must use
    /// so that only operators at or above `reference_operator`'s level are
    /// absorbed into the operand.
    pub fn prefix_operand_threshold(&self, reference_operator: &str) -> Option<usize> {
        self.binding_powers(reference_operator)
            .map(|(left_binding_power, _)| left_binding_power)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IrError {
    pub message: String,
    pub span: Option<Span>,
}

impl IrError {
    pub fn new(message: impl Into<String>, span: Option<Span>) -> Self {
        Self {
            message: message.into(),
            span,
        }
    }
}

impl fmt::Display for IrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.span {
            Some(span) => write!(f, "{} ({}..{})", self.message, span.start, span.end),
            None => f.write_str(&self.message),
        }
    }
}

impl std::error::Error for IrError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_bindings_are_typed_ir_data() {
        let alternative = Alternative {
            elements: vec![Element::Token {
                label: Some("name".into()),
                token: "IDENT".into(),
                span: Span::new(0, 5),
            }],
            constructor: Constructor {
                type_name: TypeName::from("NameExpr"),
                fields: vec![FieldBinding {
                    field: "name".into(),
                    source_label: "name".into(),
                    constant: None,
                    span: Span::new(8, 18),
                }],
            },
            recovery: None,
            span: Span::new(0, 18),
        };

        assert_eq!(alternative.constructor.type_name.0, "NameExpr");
        assert_eq!(alternative.constructor.fields[0].source_label, "name");
    }

    #[test]
    fn recovery_and_precedence_are_representable_without_backend_types() {
        let recovery = RecoveryPoint {
            sync_tokens: vec!["SEMICOLON".into(), "RBRACE".into()],
            strategy: RecoveryStrategy::SkipUntilSync,
            span: Span::new(20, 42),
        };
        let precedence = PrecedenceTable {
            rule: "Expr".into(),
            levels: vec![PrecedenceLevel {
                operators: vec!["PLUS".into(), "MINUS".into()],
                associativity: Associativity::Left,
                constructor: None,
            }],
            prefix_operators: Vec::new(),
        };

        assert_eq!(recovery.sync_tokens.len(), 2);
        assert_eq!(precedence.levels[0].associativity, Associativity::Left);
        assert_eq!(precedence.binding_powers("PLUS"), Some((1, 2)));
        assert_eq!(precedence.binding_powers("MINUS"), Some((1, 2)));
        assert_eq!(precedence.binding_powers("STAR"), None);
    }

    #[test]
    fn right_associative_precedence_uses_equal_binding_powers() {
        let precedence = PrecedenceTable {
            rule: "Expr".into(),
            levels: vec![PrecedenceLevel {
                operators: vec!["POWER".into()],
                associativity: Associativity::Right,
                constructor: None,
            }],
            prefix_operators: Vec::new(),
        };
        assert_eq!(precedence.binding_powers("POWER"), Some((1, 1)));
    }

    #[test]
    fn prefix_operand_threshold_matches_referenced_level_binding_power() {
        let precedence = PrecedenceTable {
            rule: "Expr".into(),
            levels: vec![
                PrecedenceLevel {
                    operators: vec!["PLUS".into()],
                    associativity: Associativity::Left,
                    constructor: None,
                },
                PrecedenceLevel {
                    operators: vec!["POWER".into()],
                    associativity: Associativity::Right,
                    constructor: None,
                },
            ],
            prefix_operators: vec![PrefixOperator {
                operators: vec!["MINUS".into()],
                binds_below: "POWER".into(),
                constructor: None,
            }],
        };
        assert_eq!(
            precedence.prefix_operand_threshold("POWER"),
            Some(precedence.binding_powers("POWER").unwrap().0)
        );
        assert_eq!(precedence.prefix_operand_threshold("UNKNOWN"), None);
    }
}
