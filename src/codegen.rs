//! Backend dispatch. `codegen_basic` holds the original, complete backend --
//! plain 1980s Microsoft BASIC/BASCOM, BASCAL's whole reason for existing.
//! `codegen_c` is a mostly-complete native-C backend, producing Linux/macOS/
//! Win32 binaries directly, without going through a BASIC compiler at all;
//! see `Target::C` below. `codegen_jvm` is a brand-new, bootstrap-stage
//! native-JVM backend -- just beginning, not yet ready for real programs;
//! see `Target::Jvm` below.
//!
//! Everything actually re-exported here is the BASIC backend's public
//! surface -- `records.rs` and `lib.rs` reach through this module rather
//! than `codegen_basic` directly, so callers don't need to know the split
//! exists.

pub use crate::codegen_basic::CodeGenerator;
pub(crate) use crate::codegen_basic::{
    camel_join, check_generated_name_conflicts, MID_ASSIGN_HELPER_NAME,
};

/// Which backend to generate code for. `Basic` (the default, also spelled
/// `bascom`) is BASCAL's original, complete target: plain BASIC verified
/// against real IBM/Microsoft BASCOM (see `tests/dosbox_conformance.rs`).
/// `Fbc` generates the same BASIC (`codegen_basic`, identically) but for
/// `fbc` (FreeBASIC) specifically, rejecting the handful of constructs real
/// BASCOM accepts that `fbc`'s own parser does not -- currently just
/// `try`/`catch`'s generated `RESUME <lineno>` (see GitHub issue #153;
/// #100 has the original report and the real-BASCOM-vs-`fbc` investigation
/// that led to this split, #152). `Basic` and `Fbc` are otherwise
/// interchangeable output -- BASCAL's actual complete backend is one thing,
/// this is only about which of its two downstream verifiers a given
/// program is guaranteed to satisfy. `C` is a native-C backend -- see
/// `codegen_c`'s own module doc comment for exactly what it supports
/// today. `Jvm` is a bootstrap-stage native-JVM backend -- see
/// `codegen_jvm`'s own module doc comment; it currently supports a small
/// straight-line subset. `main.rs`'s `--target` flag/`BASCAL_TARGET` env
/// var/config files all accept either spelling case-insensitively
/// (`basic`/`BASIC`/..., `bascom`/`BASCOM`/..., `fbc`/`FBC`/..., `c`/`C`,
/// `jvm`/`JVM`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Target {
    #[default]
    Basic,
    Fbc,
    C,
    Jvm,
}
