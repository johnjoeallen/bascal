//! Backend dispatch. `codegen_basic` holds the original, complete backend --
//! plain 1980s Microsoft BASIC/BASCOM, BASCAL's whole reason for existing.
//! `codegen_c` is an experimental, still-narrow native-C backend, aiming to
//! eventually produce Linux/macOS/Win32 binaries directly, without going
//! through a BASIC compiler at all; see `Target::C` below.
//!
//! Everything actually re-exported here is the BASIC backend's public
//! surface -- `records.rs` and `lib.rs` reach through this module rather
//! than `codegen_basic` directly, so callers don't need to know the split
//! exists.

pub use crate::codegen_basic::CodeGenerator;
pub(crate) use crate::codegen_basic::{camel_join, check_generated_name_conflicts, MID_ASSIGN_HELPER_NAME};

/// Which backend to generate code for. `Basic` (the default) is BASCAL's
/// original, complete target: plain BASCOM-compatible BASIC. `C` is an
/// experimental native-C backend -- see `codegen_c`'s own module doc
/// comment for exactly what it supports today. `main.rs`'s `--target`
/// flag/`BASCAL_TARGET` env var/config files all accept either spelling
/// case-insensitively (`basic`/`BASIC`/..., `c`/`C`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Target {
    #[default]
    Basic,
    C,
}
