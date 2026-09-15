//! Per-C-dialect variation `codegen_c.rs` needs once it targets more than
//! one downstream C compiler/platform -- see `RETRO_BASIC_SUPPORT_PROMPT.md`
//! for the full plan this exists to support. Deliberately small: only the
//! handful of fields Phase 0's hand-verification against a real `cc65` 2.19
//! install actually found `codegen_c.rs` hard-coding a hosted-`gcc`
//! assumption for. Extend this, not duplicate it, when a second family
//! (`z88dk`, `vbcc`, ...) needs something genuinely new -- do not add a
//! field "for completeness" that no seam in `codegen_c.rs` reads yet.

use crate::codegen::Target;

/// What varies about the target C compiler/platform `codegen_c.rs` emits
/// for. `Target::C`'s own profile (`CDialectProfile::host_gcc`) must keep
/// producing byte-for-byte identical output to today's unparameterized
/// behavior -- every other target's profile only changes what a *new*
/// target emits, never `--target c` itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CDialectProfile {
    #[allow(dead_code)] // not read anywhere yet -- carried for Phase 3/4.
    pub(crate) target: Target,
    /// Width of C's `int` under the target compiler -- 16 for `cc65`
    /// (confirmed via its own `limits.h`: `INT_MAX` is `32767`), 32 for a
    /// hosted `gcc`/`clang`. Not read anywhere in `codegen_c.rs` yet
    /// (nothing here currently depends on `int`'s width); carried on the
    /// profile now so Phase 3's capability validation has it available
    /// without a second Phase-0-style investigation.
    #[allow(dead_code)] // not read anywhere yet -- carried for Phase 3.
    pub(crate) int_bits: u8,
    /// Whether the target C compiler supports any floating-point type at
    /// all. `cc65` 2.19 does not: `#include <math.h>` fails outright (no
    /// such header ships with it), and any `double` fails to compile with
    /// "Fatal: Floating point type is currently unsupported" -- confirmed
    /// by hand, not assumed from documentation (see
    /// `RETRO_BASIC_SUPPORT_PROMPT.md`'s Phase 0 findings). Read by
    /// `codegen_c.rs`'s `\`/`MOD`/bitwise-operator emission to skip their
    /// `(double)`/`round()` round-trip when `false`: safe because every
    /// operand reaching those operators is already integer-typed once a
    /// `supports_float: false` target's capability validation (Phase 3)
    /// rejects float-typed expressions elsewhere first. `/` (true
    /// division) and `^` (exponentiation) are always float-*producing* in
    /// real BASIC semantics regardless of operand type (`5 / 2` is `2.5`)
    /// -- this flag does not, and cannot, change their emission; a target
    /// with `supports_float: false` simply can never support them at all,
    /// which is a Phase 3 capability-rejection concern, not something
    /// resolved here.
    pub(crate) supports_float: bool,
    /// Whether the target C compiler accepts a C99 variable-length array
    /// (a stack array sized by a runtime, not compile-time-constant,
    /// expression). `cc65` does not ("Error: Constant integer expression
    /// expected", confirmed by hand). **Not yet consumed anywhere in
    /// `codegen_c.rs`**: `emit_function_def`'s `byval` array-parameter
    /// copy still always emits a real VLA regardless of this flag. Giving
    /// a `supports_vla: false` target a working alternative (e.g. a
    /// `malloc`/`free`-based copy -- `cc65`'s own small heap does support
    /// both) is deferred to Phase 4, once Phase 3's capability validation
    /// exists to guard which programs can even reach that code path;
    /// implementing it earlier, before anything can distinguish "reached
    /// on a real C64 program" from "reached on a program Phase 3 will end
    /// up rejecting anyway," would be premature. Carried here now purely
    /// so that later work has a flag to read instead of a second Phase-0
    /// investigation.
    #[allow(dead_code)] // not read anywhere yet -- deferred to Phase 4.
    pub(crate) supports_vla: bool,
}

impl CDialectProfile {
    /// `Target::Basic`/`Fbc`/`Jvm` never reach `codegen_c.rs` at all, so
    /// this is really "the profile for every C-emitting target that isn't
    /// a retro cross-compiler" -- `Target::C`'s own hosted `gcc`/`clang`
    /// today. Every field here matches `codegen_c.rs`'s existing
    /// unparameterized behavior exactly; selecting this profile must never
    /// change `--target c`'s output.
    pub(crate) const fn host_gcc() -> Self {
        CDialectProfile {
            target: Target::C,
            int_bits: 32,
            supports_float: true,
            supports_vla: true,
        }
    }

    /// `cc65`'s C dialect, targeting a Commodore 64 (`cl65 -t c64`). Every
    /// field here was confirmed by hand against a real `cc65` 2.19 install
    /// (Debian package `cc65` 2.19-2), not assumed from documentation
    /// alone -- see `RETRO_BASIC_SUPPORT_PROMPT.md`'s Phase 0 findings for
    /// the specific test programs and compiler output that confirmed each
    /// one. Not reachable via `--target c64` yet: `driver.rs` still fails
    /// that target with a "not implemented" diagnostic before
    /// `codegen_c::generate` is ever called (Phase 4 wires this in).
    pub(crate) const fn c64_cc65() -> Self {
        CDialectProfile {
            target: Target::C64,
            int_bits: 16,
            supports_float: false,
            supports_vla: false,
        }
    }

    /// Selects the right profile for `target` -- `codegen_c.rs`'s only
    /// entry point into this module, called once per `generate` and
    /// carried from there on `FunctionTable` (see its own doc comment for
    /// why a field on that pervasively-threaded struct, not a new
    /// parameter on every rendering function). Every `Target` variant
    /// that never reaches `codegen_c::generate` (`Basic`, `Fbc`, `Jvm`)
    /// falls back to `host_gcc` harmlessly rather than panicking, since a
    /// real call with one of those targets is driver-level dead code no
    /// test can exercise (`driver::transpile` never calls
    /// `codegen_c::generate` for them).
    pub(crate) fn for_target(target: Target) -> Self {
        match target {
            Target::C64 => Self::c64_cc65(),
            Target::Basic | Target::Fbc | Target::C | Target::Jvm => Self::host_gcc(),
        }
    }

    /// Whether this profile supports `feature` -- the single entry point
    /// Phase 3's capability validation (`codegen_c::validate_capabilities`)
    /// reads, rather than matching on individual `bool` fields by hand at
    /// each call site.
    pub(crate) fn supports(&self, feature: CDialectFeature) -> bool {
        match feature {
            CDialectFeature::Float | CDialectFeature::Double => self.supports_float,
            CDialectFeature::VariableLengthArrays => self.supports_vla,
        }
    }
}

/// A source-level capability a target's C dialect either does or doesn't
/// support -- what Phase 3's validation pass checks a program against, one
/// `CDialectProfile` field per feature (see `CDialectProfile::supports`).
/// Deliberately only the features `codegen_c.rs`'s Phase 0 investigation
/// actually found a real construct needing; extend this, per the same
/// "only what's needed" rule as `CDialectProfile` itself, when a new
/// target's own investigation finds another one (records/random-access
/// files and unbounded recursion are both plausible future additions --
/// see `RETRO_BASIC_SUPPORT_PROMPT.md`'s Phase 4 -- but neither has a
/// confirmed `cc65` divergence backing it yet, so neither is here).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CDialectFeature {
    /// Any floating-point type or floating-point-*producing* expression:
    /// a `single`-suffixed (`!`) variable, `/` (true division -- always
    /// float-producing per real BASIC semantics regardless of operand
    /// type, `5 / 2` is `2.5`), and `^` (exponentiation, same). `cc65`
    /// supports neither `float` nor `double` at all (confirmed by hand --
    /// see `CDialectProfile::supports_float`'s own doc comment), so both
    /// this and `Double` read the same underlying flag today; a future
    /// target that genuinely distinguishes them (supports one, not the
    /// other) earns `CDialectProfile` a second flag then, not before.
    Float,
    /// A `double`-suffixed (`#`) variable. See `Float`'s own doc comment
    /// for why this reads the same `supports_float` flag today.
    Double,
    /// A C99 variable-length array -- see `CDialectProfile::supports_vla`.
    VariableLengthArrays,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_gcc_profile_matches_todays_unparameterized_behavior() {
        let profile = CDialectProfile::host_gcc();
        assert_eq!(profile.target, Target::C);
        assert_eq!(profile.int_bits, 32);
        assert!(profile.supports_float);
        assert!(profile.supports_vla);
    }

    #[test]
    fn c64_profile_matches_hand_verified_cc65_behavior() {
        let profile = CDialectProfile::c64_cc65();
        assert_eq!(profile.target, Target::C64);
        assert_eq!(profile.int_bits, 16);
        assert!(!profile.supports_float);
        assert!(!profile.supports_vla);
    }

    #[test]
    fn for_target_selects_c64_profile_only_for_target_c64() {
        assert_eq!(
            CDialectProfile::for_target(Target::C64),
            CDialectProfile::c64_cc65()
        );
        assert_eq!(
            CDialectProfile::for_target(Target::C),
            CDialectProfile::host_gcc()
        );
        assert_eq!(
            CDialectProfile::for_target(Target::Basic),
            CDialectProfile::host_gcc()
        );
        assert_eq!(
            CDialectProfile::for_target(Target::Fbc),
            CDialectProfile::host_gcc()
        );
        assert_eq!(
            CDialectProfile::for_target(Target::Jvm),
            CDialectProfile::host_gcc()
        );
    }

    #[test]
    fn host_gcc_supports_every_feature() {
        let profile = CDialectProfile::host_gcc();
        assert!(profile.supports(CDialectFeature::Float));
        assert!(profile.supports(CDialectFeature::Double));
        assert!(profile.supports(CDialectFeature::VariableLengthArrays));
    }

    #[test]
    fn c64_supports_no_feature_checked_here() {
        let profile = CDialectProfile::c64_cc65();
        assert!(!profile.supports(CDialectFeature::Float));
        assert!(!profile.supports(CDialectFeature::Double));
        assert!(!profile.supports(CDialectFeature::VariableLengthArrays));
    }
}
