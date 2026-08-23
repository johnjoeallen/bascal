//! Registry of real-BASIC intrinsics BASCAL exposes as scalar methods
//! (`s$.left(3)`, `n%.abs()`) -- see GitHub issue #38. Each entry is
//! resolved by `records::Lowerer::rewrite_expr`'s own `Expr::ScalarMethodCall`
//! arm into the *ordinary* call form of the same intrinsic (`LEFT$(s$, 3)`),
//! which both `codegen_basic.rs` and `codegen_c.rs` already handle correctly
//! and unchanged -- this module is pure data plus one pure lookup function,
//! not an AST traversal of its own. `records.rs` already walks every
//! expression in the program and has everything else this rewrite needs
//! (receiver-type inference, diagnostics), so there's no reason to duplicate
//! that walk here.
//!
//! Deliberately narrow, matching the issue's own scope: `left`/`right`/`mid`/
//! `len`/`instr` (string) and `abs`/`sqr`/`sin`/`cos`/`tan`/`int`/`fix`/`sgn`
//! (numeric) -- no arrays, records, files, pipelines, overloads, or default
//! parameters (all explicitly out of scope for #38/#33).

use crate::ast::TypeSuffix;

/// Which scalar receiver types a built-in method accepts. The four numeric
/// suffixes (`%`/`&`/`!`/`#`) share identical behavior for every numeric
/// method here, so they're grouped as one family rather than enumerated --
/// same convention real BASIC's own `ABS`/`SQR`/... already follow (any
/// numeric argument, no separate overload per numeric type).
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum ReceiverFamily {
    String,
    AnyNumeric,
}

impl ReceiverFamily {
    fn matches(self, suffix: TypeSuffix) -> bool {
        match self {
            ReceiverFamily::String => suffix == TypeSuffix::String,
            ReceiverFamily::AnyNumeric => suffix != TypeSuffix::String,
        }
    }
}

pub(crate) struct BuiltinMethod {
    pub(crate) method: &'static str,
    family: ReceiverFamily,
    /// The rewritten ordinary call's own identifier suffix -- `Some(String)`
    /// for `left`/`right`/`mid` (real `LEFT$`/`RIGHT$`/`MID$`), `None` for
    /// every other entry here (bare, matching real BASIC's own `LEN`/`ABS`/
    /// `SQR`/`SIN`/`COS`/`TAN`/`INT`/`FIX`/`SGN`, none of which carry a type
    /// suffix).
    pub(crate) call_suffix: Option<TypeSuffix>,
    /// Explicit argument count range accepted, *not* counting the receiver
    /// itself (which always becomes the rewritten call's first argument).
    pub(crate) min_args: usize,
    pub(crate) max_args: usize,
}

pub(crate) const BUILTIN_METHODS: &[BuiltinMethod] = &[
    BuiltinMethod {
        method: "left",
        family: ReceiverFamily::String,
        call_suffix: Some(TypeSuffix::String),
        min_args: 1,
        max_args: 1,
    },
    BuiltinMethod {
        method: "right",
        family: ReceiverFamily::String,
        call_suffix: Some(TypeSuffix::String),
        min_args: 1,
        max_args: 1,
    },
    BuiltinMethod {
        method: "mid",
        family: ReceiverFamily::String,
        call_suffix: Some(TypeSuffix::String),
        min_args: 1,
        max_args: 2,
    },
    BuiltinMethod {
        method: "len",
        family: ReceiverFamily::String,
        call_suffix: None,
        min_args: 0,
        max_args: 0,
    },
    BuiltinMethod {
        method: "instr",
        family: ReceiverFamily::String,
        call_suffix: None,
        min_args: 1,
        max_args: 1,
    },
    BuiltinMethod {
        method: "abs",
        family: ReceiverFamily::AnyNumeric,
        call_suffix: None,
        min_args: 0,
        max_args: 0,
    },
    BuiltinMethod {
        method: "sqr",
        family: ReceiverFamily::AnyNumeric,
        call_suffix: None,
        min_args: 0,
        max_args: 0,
    },
    BuiltinMethod {
        method: "sin",
        family: ReceiverFamily::AnyNumeric,
        call_suffix: None,
        min_args: 0,
        max_args: 0,
    },
    BuiltinMethod {
        method: "cos",
        family: ReceiverFamily::AnyNumeric,
        call_suffix: None,
        min_args: 0,
        max_args: 0,
    },
    BuiltinMethod {
        method: "tan",
        family: ReceiverFamily::AnyNumeric,
        call_suffix: None,
        min_args: 0,
        max_args: 0,
    },
    BuiltinMethod {
        method: "int",
        family: ReceiverFamily::AnyNumeric,
        call_suffix: None,
        min_args: 0,
        max_args: 0,
    },
    BuiltinMethod {
        method: "fix",
        family: ReceiverFamily::AnyNumeric,
        call_suffix: None,
        min_args: 0,
        max_args: 0,
    },
    BuiltinMethod {
        method: "sgn",
        family: ReceiverFamily::AnyNumeric,
        call_suffix: None,
        min_args: 0,
        max_args: 0,
    },
];

/// Finds the built-in method entry named `method` (case-insensitive) whose
/// receiver family matches `receiver` -- `records::Lowerer` uses this to
/// decide whether a `ScalarMethodCall` should rewrite to an ordinary call at
/// all. Arg-count validation against `min_args`/`max_args` is the caller's
/// own job, so a name match with the wrong arity still gets a clear
/// "wrong number of arguments" diagnostic instead of silently falling
/// through to `ScalarMethodCall`'s generic "unknown method" check.
pub(crate) fn find(receiver: TypeSuffix, method: &str) -> Option<&'static BuiltinMethod> {
    BUILTIN_METHODS
        .iter()
        .find(|m| m.family.matches(receiver) && m.method.eq_ignore_ascii_case(method))
}
