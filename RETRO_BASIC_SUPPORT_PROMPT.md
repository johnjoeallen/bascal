# Task: Add Native 1980s/1990s Home-Computer Targets to BASCAL

## Direction (supersedes the original BASIC-dialect plan)

This plan previously proposed emitting native BASIC dialects (C64 BASIC V2,
CPC464 Locomotive BASIC, Atari 400/800 BASIC, Atari ST BASIC) directly, the
same way `--target basic` emits BASCOM-compatible BASIC today. That approach
is now superseded: BASIC *interpretation* on a 1-8MHz 8/16-bit CPU is slow,
and the actual reason to target this hardware -- games, demos, anything with
real timing -- means native machine code, not another interpreted BASIC.

The revised direction reuses the project's own `--target C` precedent
(`codegen_c.rs`): emit C, and route it through an **existing, mature
cross-compiler toolchain** for the target CPU/platform, the same way
`--target fbc` already routes BASIC through FreeBASIC instead of implementing
a second BASIC backend from scratch. No hand-written assembler, register
allocator, or linker is written by this project for any of these targets --
that would be building a full compiler backend per CPU family, wildly out of
proportion for a hobby project, and is explicitly a non-goal below.

The initial objective remains a vanity-project-quality set of targets that
can emit small, runnable native programs for real hardware or emulators. It
is not a complete implementation of every platform's OS/library surface or
every BASCAL feature.

## Planned targets, grouped by CPU family / toolchain

Grouping by toolchain matters: the first target in a family pays for that
family's C-dialect-quirk handling and toolchain-invocation plumbing: every
later target in the same family reuses it and is comparatively cheap.

**Family A -- 6502, via `cc65`** (a single mature, actively-maintained C
cross-compiler with official per-platform runtime/startup support for all of
these):

1. Commodore 64 (first deliverable overall)
2. Atari 400/800/XL/XE (`cc65`'s own `atari`/`atarixl` targets)
3. Apple II (`cc65`'s best-supported, most-used target historically)
4. Low-cost extras once the family is proven: Commodore 128, VIC-20, Plus/4,
   PET -- all official `cc65` targets sharing the C64 work almost entirely

**Family B -- Z80, via `z88dk` (or SDCC)**:

1. Amstrad CPC464 (first deliverable in this family)
2. ZX Spectrum -- comparable community size/significance to the CPC, and a
   first-class `z88dk` target
3. MSX -- also a first-class `z88dk` target; `openMSX` is a notably
   mature, scriptable emulator, a good fit for automated test fixtures

**Family C -- 68000, via `vbcc` (with `vasm`/`vlink`) or `bebbo`'s
`m68k-amigaos-gcc` fork**:

1. Atari ST (first deliverable in this family; TOS executable format,
   closer to bare metal)
2. Amiga -- same CPU family and largely the same C-dialect-quirk handling
   as the ST; the added cost is AmigaOS's real library-call API (`exec`,
   `dos.library`, `intuition.library`) versus TOS's more direct hardware
   access. Scope the initial Amiga target to console I/O via `dos.library`
   only; do not attempt Intuition/graphics in the first pass.

Do not attempt more than one target at a time, and do not attempt a second
*family* before the first target in family A (C64) is stable end to end.
Each target must have its own capability checks and its own toolchain
invocation; nothing here is shared automatically just because two targets
are in the same family.

Machines considered and deliberately excluded from this plan: BBC Micro /
Acorn Electron and TRS-80 (no C cross-compiler on the level of `cc65`/
`z88dk`; would mean writing a real backend from scratch, i.e. exactly the
cost this plan exists to avoid); Sinclair QL (68008, but modern cross-
compiler support is thin/unmaintained); NES (a `cc65` target exists, but a
game console has no BASIC/general-purpose-computer heritage and is a
thematic mismatch for this project).

## Repository constraints

Read and follow:

- `AGENTS.md`
- `src/driver.rs`
- `src/codegen.rs`
- `src/codegen_c.rs`
- `src/ast.rs`
- `src/resolver.rs`
- existing compiler tests

Use precise compiler terminology. When describing source transformation,
use `transpile`, not `lower`, unless referring specifically to compiler
implementation internals.

Preserve the typed-intermediate-representation requirement. Backends must
consume resolved type information and must not re-infer source types from
identifier spelling, syntax, or another backend's output.

Do not change the existing BASCOM, FreeBASIC, C, or JVM output while adding
these targets.

## Overall architecture

The intended pipeline, per target:

```text
.bcl source
  -> lexer/parser
  -> AST
  -> source-level lowering passes
  -> name and type resolution
  -> resolved typed IR
  -> target capability validation (does this program fit the target
     cross-compiler's supported C subset and the platform's memory/
     library model?)
  -> C emission, reusing codegen_c.rs, parameterized per target's
     C-dialect quirks
  -> invoke the target's cross-compiler/linker (cc65 / z88dk / vbcc /
     bebbo's gcc) the same way driver.rs already shells out to fbc/gcc
  -> a native loadable/executable image for the target platform
     (C64 PRG, CPC464 AMSDOS binary, Atari ST TOS .PRG, Amiga Hunk
     executable, etc.)
```

**Existing backends are out of scope for this rework.** `codegen_basic.rs`,
`codegen_c.rs`, and `codegen_jvm.rs` already have working, tested identifier
rendering and output. Do not retrofit any of them beyond the parameterization
`codegen_c.rs` itself needs to support multiple C dialects (see Phase 2) --
that parameterization must leave `--target c`'s existing host-`gcc` output
byte-for-byte unchanged. Do not build a second, independent C emitter per
retro target; every retro target is a *dialect variant* of `codegen_c.rs`,
not a new file that reimplements C emission from scratch.

One consequence worth calling out explicitly: because every retro target
here compiles through a real C compiler (`cc65`/`z88dk`/`vbcc`/`bebbo`'s
`gcc`), every one of them gets real, correct lexical scoping for free, the
same way `--target c`/`--target jvm` already do. The elaborate whole-program
name-mangling/truncation machinery a *native BASIC V2 emitter* would have
needed (C64 BASIC V2's flat, two-significant-character variable namespace)
simply does not apply here -- `cc65` handles C64 identifier scoping itself,
just as `gcc` does for `--target c` today. This is a major simplification
relative to the original BASIC-dialect draft of this plan, not a detail to
re-solve per target.

## Target roadmap

### Phase 0: Baseline

Before changing code:

1. Run the existing test suite.
2. Inspect target parsing and output-path selection, and how `driver.rs`
   already invokes `fbc`/`gcc` post-transpile -- this is the pattern the
   new cross-compiler invocations extend.
3. Install `cc65` and compile a small representative `--target c` example
   through it by hand (outside the compiler) to catalog real divergences
   from host `gcc`: `int` width (16-bit, not the host's 32/64-bit), the
   state of floating-point support (historically absent or slow software
   float; confirm current `cc65` behavior), stack depth and recursion
   limits, absence of VLAs, and header/library differences from a hosted
   libc.
4. Record which existing `codegen_c.rs` output constructs already compile
   unmodified under `cc65`, and which don't.
5. Identify all places where `codegen_c.rs` currently assumes a hosted
   (Linux/macOS/Win32 + host `gcc`) C environment, since those are exactly
   the seams the per-target dialect parameterization in Phase 2 needs.

Do not modify files in this phase.

### Phase 1: Add target identifiers

Add a target enum variant and CLI/configuration spelling for the C64 only
first:

```rust
Target::C64
```

Accept a case-insensitive form:

```text
--target c64
```

Update:

- target parsing
- command-line help
- configuration/environment parsing
- default output extension/artifact naming (a C64 target produces a
  loadable image, not a `.bas`/`.c` text file -- see Phase 4)
- target dispatch
- target parsing tests

At this stage the C64 target may emit a clear `not implemented` diagnostic.

Existing targets must produce byte-for-byte equivalent output wherever the
existing tests require it.

Add the remaining targets (`Target::Atari800`, `Target::AtariSt`,
`Target::Amiga`, `Target::Cpc464`, ...) the same way, one at a time, only
once the target before it in the roadmap is stable -- do not add every
variant up front.

### Phase 2: `codegen_c.rs` dialect parameterization

Introduce a small, explicit description of what varies between C dialects,
consumed by `codegen_c.rs` wherever it currently assumes a hosted-`gcc`
environment (per Phase 0's inventory). This is deliberately not a large
abstraction: start with only what C64/`cc65` actually needs, and extend it
per family as each new family's first target is built.

Suggested model:

```rust
struct CDialectProfile {
    target: Target,
    int_bits: u8,               // e.g. 16 for cc65, 32/64 for host gcc
    supports_float: bool,       // and, if true, at what cost
    supports_vla: bool,
    max_recursion_hint: Option<u32>,
    toolchain: ToolchainInvocation,
}
```

`codegen_c.rs` reads the active profile instead of hard-coding host-`gcc`
assumptions at those specific seams. `--target c`'s own profile must render
identically to today's unparameterized behavior -- this phase changes no
existing output.

Acceptance criteria:

- `--target c` output is byte-for-byte unchanged
- every seam identified in Phase 0 reads from `CDialectProfile` rather than
  assuming host `gcc`
- adding a new profile requires no change to target-neutral code paths

### Phase 3: Shared target capability validation

Add a validation pass after resolution and before C emission, checking a
BASCAL program against the *active dialect profile's* real constraints (not
a BASIC feature list, since these targets no longer emit BASIC):

```rust
enum CDialectFeature {
    ThirtyTwoBitInt,
    Float,
    Double,
    VariableLengthArrays,
    UnboundedRecursion,
    DynamicHeapAllocation,
}
```

Each dialect profile declares what it supports. Unsupported constructs must
produce source-positioned diagnostics instead of C the target toolchain will
reject or silently miscompile.

Example diagnostic:

```text
error: `double` variables are not supported by the C64 (cc65) target
```

Capability validation must occur before emission so a target never produces
partial output for an invalid program.

### Phase 4: Commodore 64, via `cc65`

Implement this target first.

Build process:

1. Transpile to C via `codegen_c.rs`, parameterized with the C64
   `CDialectProfile` from Phase 2.
2. Invoke `cc65`/`ca65`/`ld65` (or the `cl65` driver that wraps all three)
   to assemble and link a C64 program, the same way `driver.rs` already
   shells out to `fbc`/`gcc` for other targets.
3. Produce a loadable C64 `PRG` file.

Initial supported subset: whatever of BASCAL's existing `--target c`
feature set survives the C64 `CDialectProfile`'s capability validation --
scalar integer variables, string handling via `cc65`'s runtime, numeric and
string expressions, assignment, `print`/`input`, `if`/`for`/`while`/`do`,
`goto`/`gosub`/`return`, `data`/`read`/`restore`, one-dimensional arrays,
console I/O only.

Initially reject (via Phase 3's capability validation, not silently):

- `double` (unless/until Phase 0 confirms adequate `cc65` float support)
- records and random-access files
- `try`/`catch`, `on error goto`/`resume` (same permanent limitation
  `--target c` already documents for classic error handling on a real
  call stack)
- multidimensional arrays (until proven to fit `cc65`'s memory model)
- graphics/sound (out of scope for the first pass -- see Non-goals)

### Phase 5: Family A extras -- Atari 400/800/XL/XE, then Apple II

Once C64 is stable, add `cc65`'s own `atari`/`atarixl` target, then
`apple2`. Each is a new `CDialectProfile` plus toolchain invocation, not a
new emitter -- reuse Phase 2/3's machinery. Platform-specific I/O
differences (e.g. Atari's `GRAPHICS`/`SETCOLOR`/`SOUND` equivalents once
graphics support is in scope) are the only genuinely new work per target.

### Phase 6: Amstrad CPC464, via `z88dk`

Implement this after family A is stable, as the first Family B (Z80)
target.

Repeat Phase 2/3's pattern for the Z80/`z88dk` dialect: a new
`CDialectProfile`, its own capability constraints (`z88dk`'s C subset,
memory model, and CPC464-specific library bindings), and a toolchain
invocation producing a CPC464-loadable binary (AMSDOS format).

The CPC464 target must be distinct from later CPC BASIC/hardware
revisions (CPC664/CPC6128) -- do not silently emit 6128-only features.

### Phase 7: Family B extras -- ZX Spectrum, then MSX

Once CPC464 is stable, add the ZX Spectrum and MSX `z88dk` targets the same
way as Phase 5 did for family A.

### Phase 8: Atari ST, via `vbcc`/`vasm`/`vlink`

Implement this as the first Family C (68000) target, after family B is
stable.

New `CDialectProfile` for `vbcc`'s 68000 C dialect, capability constraints,
and a toolchain invocation producing a TOS-loadable `.PRG`/`.TOS`
executable.

### Phase 9: Amiga, via `vbcc` or `bebbo`'s `m68k-amigaos-gcc`

Implement after the Atari ST target is stable, reusing the same 68000
`CDialectProfile` machinery from Phase 8 -- the CPU-level C-dialect quirks
are shared with the ST.

The genuinely new work is AmigaOS's library-call API rather than the ST's
more direct hardware access. Scope the first pass to console I/O via
`dos.library` only (`print`/`input` equivalents); do not attempt
Intuition/graphics/windowing in this phase.

Produce an AmigaOS Hunk-format executable.

## Backend structure

```text
src/codegen_c.rs           existing native-C backend, now parameterized
                            by CDialectProfile for host gcc / cc65 / z88dk /
                            vbcc / bebbo's gcc
src/c_dialect.rs            CDialectProfile, CDialectFeature, and the
                            per-target profile definitions
```

Do not add a separate `codegen_c64.rs`/`codegen_cpc464.rs`/etc. per target.
The whole point of routing through existing C cross-compilers is that one
parameterized C emitter serves every target; a second full emitter per
platform would recreate the cost this plan exists to avoid.

Do not introduce a large abstraction beyond `CDialectProfile` until the C64
implementation demonstrates a second family genuinely needs something more.

## Toolchain invocation

Each target's post-transpile step (parallel to how `driver.rs` already
handles `--binary`/`--run` for `fbc`/`gcc`) must:

- locate the target cross-compiler/toolchain on `PATH` (or a configured
  path), and fail with a clear, actionable diagnostic if it is missing --
  never silently fall back to host `gcc`
- invoke it with the flags needed to select the specific platform variant
  (e.g. `cc65`'s `-t c64` vs `-t atari`)
- produce the platform's native loadable/executable image
- support an optional "run under emulator" flag per target, parallel to
  `--run`, invoking the platform's emulator (see Testing strategy) when
  available

## Runtime helpers

Some BASCAL operations are not native to every target's C dialect (e.g. a
`cc65` environment without adequate float support still needs
`left$`/`right$`/`mid$`/`str$`/`val` equivalents). Add generated helpers
only when required by a supported source construct, and only when the
target's own dialect doesn't already provide them via its C runtime.

Every helper must:

- be a real C function name resolved through the existing `codegen_c.rs`
  identifier machinery -- no separate name table is needed (see the
  scoping note under Overall architecture)
- avoid collisions with user symbols and with the target C runtime's own
  reserved names
- be emitted only when referenced
- be covered by output tests

Do not add runtime helpers for features that are rejected by the capability
validator.

## Testing strategy

Every target needs three levels of tests.

### Unit tests

Test:

- target parsing
- capability diagnostics
- dialect-profile-specific C emission (e.g. `int` width, float
  availability)
- toolchain-invocation argument construction (without requiring the
  toolchain to be installed)

### Golden output tests

Add fixtures containing expected C output (post-`CDialectProfile`, pre-
toolchain) for:

- hello world
- arithmetic
- strings
- loops
- arrays
- functions and procedures
- target-specific capability rejections

### Toolchain + emulator tests

These require the actual cross-compiler and emulator installed, and must be
optional and skipped cleanly when unavailable -- the same pattern the
project already uses for the `fbc`/`dosbox-x` conformance suites.

Suggested toolchains/emulators per target:

- C64: `cc65`; VICE, preferably `x64sc`
- Atari 400/800/XL: `cc65` (`atari`/`atarixl`); Atari800 or Altirra
- Apple II: `cc65` (`apple2`); AppleWin or LinApple
- CPC464: `z88dk`; a CPC emulator such as WinAPE or CPCEMU
- ZX Spectrum: `z88dk`; Fuse or ZEsarUX
- MSX: `z88dk`; openMSX
- Atari ST: `vbcc`/`vasm`/`vlink`; Hatari
- Amiga: `vbcc` or `bebbo`'s `m68k-amigaos-gcc`; FS-UAE or WinUAE

Do not make toolchain or emulator availability a requirement for ordinary
unit tests.

The first toolchain+emulator fixture per target should simply print a known
string and confirm the emulator's captured output/console matches. Add more
fixtures only after the basic build-and-run path is reliable.

## Documentation

Document, per target:

- the supported BASCAL subset (post-capability-validation)
- rejected constructs and why (dialect limitation vs. deliberately
  out of scope)
- required toolchain, its installation, and how to point `bcc` at it
- emulator setup
- how to produce, transfer, and run the resulting image on real hardware

State clearly that these targets produce real native machine code via an
existing third-party cross-compiler, not BASIC output, and that BASCAL does
not implement its own assembler, linker, or code generator for any of these
CPUs.

## Non-goals

Do not implement more than one target at a time, and do not start a second
CPU family before the first family's first target is stable.

Do not implement, for this project, ever:

- a hand-written assembler, register allocator, or linker for any of these
  CPUs -- always route through the target's existing cross-compiler
- BASIC-dialect emission for any of these platforms (superseded direction)

Do not implement initially:

- non-console graphics or sound (`cc65`/`z88dk`/`vbcc` all expose real
  platform libraries for this; defer until console I/O is solid per
  target)
- disk-image mastering beyond whatever the toolchain produces by default
- full OS/library-API coverage (e.g. AmigaOS Intuition, CPC AMSDOS beyond
  basic file I/O)
- complete error-handling parity with `--target c`'s classic-error-handling
  limitations already documented for real C call stacks
- automatic translation between targets/families
- full compatibility with every BASCAL feature on every target

## Final acceptance criteria

The staged work is complete only when, per target added:

1. `bcc --target <name> example.bcl` transpiles to C, invokes the correct
   cross-compiler, and produces a loadable native image.
2. Existing targets (`basic`/`fbc`/`c`/`jvm`) retain their current,
   byte-for-byte-unchanged behavior.
3. Target capability validation rejects unsupported constructs before
   emission, with a clear diagnostic naming the target.
4. `codegen_c.rs` remains the single C emitter for every target in this
   plan -- no per-target duplicate emitter exists.
5. Optional toolchain/emulator tests run when the relevant toolchain and
   emulator are installed, and are skipped cleanly otherwise.
6. A second target is only started once the target before it (per the
   family ordering above) is stable and tested.
7. All existing tests and new target tests pass.
8. The implementation remains small enough for a hobby project.
