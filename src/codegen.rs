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

/// Which backend to generate code for. `Basic` (the default) is BASCAL's
/// original, complete target: plain BASCOM-compatible BASIC. `C` is a
/// native-C backend -- see `codegen_c`'s own module doc comment for exactly
/// what it supports today. `Jvm` is a bootstrap-stage native-JVM backend --
/// see `codegen_jvm`'s own module doc comment; it doesn't generate any code
/// yet. `main.rs`'s `--target` flag/`BASCAL_TARGET` env var/config files all
/// accept either spelling case-insensitively (`basic`/`BASIC`/..., `c`/`C`,
/// `jvm`/`JVM`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Target {
    #[default]
    Basic,
    C,
    Jvm,
}
