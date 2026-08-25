[Home](../../) / [Manual](../) / Command-Line Reference

[← Generated BASIC Shape](generated-basic-shape.md) [Statement Quick Reference →](statement-quick-reference.md)

<div class="prose" markdown="1">

```bascal
bcc input.bcl [-o dir/] [-L dir] [-l library]
              [--line-numbers | --sparse-line-numbers] [--clean | -c]
              [--binary | -b] [--run | -r] [--target | -t basic|C|jvm]
              [--strict-vars | --strict-vars-warn]
```

| Flag                    | Short | Description                                                                                                                                                                                                                                                                                                                                                                                                                     |
|-------------------------|-------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `-o dir`                |       | Output directory -- already exists as one, or is written with a trailing `/` even if it doesn't exist yet. The output file is auto-named inside it (source's own name with `.bas`, `.c` under `--target c`, or `.j` under `--target jvm`) -- `-o` does not accept an exact output file path. Default (no `-o` at all): same directory as the source.                                                                                                       |
| `-L dir`                |       | Add a directory to the library search path. Repeatable.                                                                                                                                                                                                                                                                                                                                                                         |
| `-l name`               |       | Name a library (reserved).                                                                                                                                                                                                                                                                                                                                                                                                      |
| `--line-numbers`        |       | Number every output line, not just branch targets (the default — only needed to override an earlier `--sparse-line-numbers` on the same command line).                                                                                                                                                                                                                                                                          |
| `--sparse-line-numbers` |       | Number only branch targets, not every line. Real MBASIC/BASCOM-family compilers need a switch for this: Microsoft's BASCOM uses `/C`, IBM's BASIC Compiler uses `/N`; FreeBASIC's `-lang qb` accepts it with no switch at all.                                                                                                                                                                                                  |
| `--clean`               | `-c`  | Re-transpile even if the output is already up to date.                                                                                                                                                                                                                                                                                                                                                                          |
| `--binary`              | `-b`  | Compile the generated output to a binary in `tmp/`: `fbc` for `--target basic`, `gcc` for `--target c`, `krak2` (assembling to a `.class`, see below) for `--target jvm`.                                                                                                                                                                                                                                                                                                                         |
| `--run`                 | `-r`  | Also run the compiled binary (implies `--binary`), with stdin/stdout/stderr inherited. For `--target basic` this always means `fbc`'s binary, run directly — not real BASCOM, whose own `.EXE` needs a DOS environment/emulator like dosbox-x to run at all. For `--target jvm`, runs the assembled `.class` via `java`.                                                                                                                                                                    |
| `--target <t>`          | `-t`  | Backend to generate code for: `basic` (the original, complete backend), `c` (a mostly-complete native-C backend — see below), or `jvm` (an early-stage native-JVM backend — see below). Case-insensitive.                                                                                                                                                                                                                                                                 |
| `--strict-vars`         |       | Opt-in, Pascal-style mandatory variable declaration — see [DIM](variables-and-constants.md#dim). Rejects the compile if any scalar/array variable is used without a prior `DIM`/`DECLARE`, a `CONST`, a `FOR` loop's own counter, or a function/procedure parameter. Checked only against this program's own source, never a `require`d library's. Turning this on means the program is no longer a strict superset of BASIC. |
| `--strict-vars-warn`    |       | Same check as `--strict-vars`, but every finding is printed to stderr as a warning instead of failing the compile. Ignored if `--strict-vars` is also given.                                                                                                                                                                                                                                                                    |

### Default Target

Without `--target`, `bcc` picks a default from, first match wins: the `BASCAL_TARGET` environment variable; `~/.config/bascal/config` (a `target=c` line, simple `key=value` settings one per line, `#` comments allowed); `/etc/default/bascal` (same format, system-wide — the standard Debian `/etc/default/<pkgname>` convention, a plain file, not a directory); otherwise `basic`. An explicit `--target`/`-t` on the command line always overrides whatever this picks — handy for setting `c` as a working default without typing `--target c` on every call:

```bascal
mkdir -p ~/.config/bascal
echo "target=c" > ~/.config/bascal/config
```

### Portability across backends

BASCAL started as a strict superset of classic BASIC: every raw BASIC statement class the compiler recognizes at all was, by design, guaranteed to pass through and transpile correctly. That guarantee holds in full today only for `--target basic`. Answering to more than one runtime came at a cost: `--target c` and `--target jvm` each have to permanently drop a small set of raw-BASIC forms that don't translate safely onto a real C call stack or the JVM's own method/bytecode model — for readability and portability across backends, not because anyone stopped caring about compatibility. BASCAL as a whole is now a **partial** superset of BASIC, not a strict one. `--target basic` remains the most complete, closest-to-strict superset, and the right choice whenever maximum BASIC compatibility matters more than a native binary or JVM output.

The divergences below are permanent design decisions, not temporary implementation gaps — see each backend's own "Currently supported"/"Not yet supported" lists further down this page (and the GitHub issue tracker's `c-target`/`jvm-target` labels) for what's simply still unfinished, a separate and shrinking list.

**`--target c`**:

- Classic `ON ERROR GOTO`/every `RESUME` variant/`ERROR` is rejected at compile time (see `reject_classic_error_handling` in `codegen_c.rs`). The C backend's `gosub`/`on error goto` support is built on a return-address-ID stack that only works because top-level code executes directly in `main` — a `RESUME` reaching into or across a `procedure`'s own C call frame has no safe equivalent without a much larger redesign of how procedures compile, so this is a permanent limitation of compiling to real C functions on the real C call stack, not a "not yet." **Portable equivalent:** `try`/`catch`/`finally`, BASCAL's own structured error recovery, supported on both `basic` and `C`.

**`--target jvm`:**

- `GOSUB`/bare `RETURN` (BASIC-level subroutine call, distinct from a `function`/`procedure`'s own `return`) is rejected entirely, not merely deferred. **Portable equivalent:** `function`/`procedure` — the structured replacement BASCAL already provides, and the one the language steers new/edited source toward anyway (see [Legacy-Form Warnings](miscellaneous-statements.md#legacy-form-warnings)).
- `GOTO`/`label:` may not cross into or out of a `function`/`procedure` — a `goto`'s target label must live in the same callable as the `goto` itself (or, for a top-level `goto`, another top-level label in the main program body). This isn't a BASCAL policy choice: the JVM's own `goto` instruction is a branch offset within one method's own bytecode, with no way to jump into another method's code at all, so this matches what the JVM can physically express. **Portable equivalent:** none direct — restructure the jump as a function call/return instead.
- Classic `ON ERROR GOTO`/every `RESUME` variant is rejected, for the same reason as `--target c` above. The JVM target supports the initial `try`/`catch`/`finally` path for explicit numeric `throw`/`error` statements; catch filters, source bindings, and runtime file errors remain unfinished (see issue #108). **Portable equivalent:** use structured `try`/`catch`/`finally` rather than legacy handlers.

### Backends

`--target basic` (the default) is everything else on this page: plain 1980s Microsoft BASIC/BASCOM output.

`--target c` is a new, deliberately minimal native-C backend, aiming to produce native Linux/macOS/Win32 binaries directly (via `gcc`) without going through a BASIC compiler at all — while the BASCOM-compatible `basic` target keeps gating what language features BASCAL adds, so both backends stay able to express the same language.

Tutorials that compile end to end today (see each one's `.c` counterpart for its output): [`01_hello`](https://github.com/johnjoeallen/bascal/blob/main/tutorial/01_hello.bcl), [`02_variables`](https://github.com/johnjoeallen/bascal/blob/main/tutorial/02_variables.bcl), [`03_arithmetic`](https://github.com/johnjoeallen/bascal/blob/main/tutorial/03_arithmetic.bcl), [`04_conditions`](https://github.com/johnjoeallen/bascal/blob/main/tutorial/04_conditions.bcl), [`05_loops`](https://github.com/johnjoeallen/bascal/blob/main/tutorial/05_loops.bcl), [`06_select_case`](https://github.com/johnjoeallen/bascal/blob/main/tutorial/06_select_case.bcl), [`07_functions`](https://github.com/johnjoeallen/bascal/blob/main/tutorial/07_functions.bcl), [`08_arrays`](https://github.com/johnjoeallen/bascal/blob/main/tutorial/08_arrays.bcl), [`09_data`](https://github.com/johnjoeallen/bascal/blob/main/tutorial/09_data.bcl), [`10_files`](https://github.com/johnjoeallen/bascal/blob/main/tutorial/10_files.bcl), [`11_screen`](https://github.com/johnjoeallen/bascal/blob/main/tutorial/11_screen.bcl), [`13_shared/start`](https://github.com/johnjoeallen/bascal/blob/main/tutorial/13_shared/start.bcl), [`13_shared/show`](https://github.com/johnjoeallen/bascal/blob/main/tutorial/13_shared/show.bcl), [`15_random_and_record_files`](https://github.com/johnjoeallen/bascal/blob/main/tutorial/15_random_and_record_files.bcl), [`16_short_circuit`](https://github.com/johnjoeallen/bascal/blob/main/tutorial/16_short_circuit.bcl), [`17_labels_and_error_handling`](https://github.com/johnjoeallen/bascal/blob/main/tutorial/17_labels_and_error_handling.bcl), [`18_stdlib`](https://github.com/johnjoeallen/bascal/blob/main/tutorial/18_stdlib.bcl), and [`inventory`](https://github.com/johnjoeallen/bascal/blob/main/tutorial/inventory.bcl). The GitHub issue tracker's `c-target` label lists what's still missing to get the rest of the tutorials compiling.

#### `--target c`: currently supported

- `print`, `end`.
- `dim` of a scalar or an array — a real, native C array, multi-dimensional too: e.g. `dim grid%(9, 9)` becomes a genuine `int grid[10][10]`, no manual flattening/stride arithmetic. Only a literal or top-level-`const`-literal bound is allowed in every dimension: a real C array needs a compile-time-known size, unlike real BASIC's own `dim arr%(n%)` for a runtime-computed `n%`, which isn't supported here.
- Indexed array reads/writes, and `sizeof(arr%)`/`sizeof(grid%, axis)` — resolved directly to a literal at compile time, since every tracked array's bounds already are.
- Array parameters, `byval` or `byref`, declared with one `?` per dimension (`arr%(?)`, `grid%(?, ?)`). `byval` copies the caller's array into a local buffer sized from the largest call site; `byref` reuses the caller's own storage directly, with no copy.
- `swap` of two scalars or array elements — a numeric swap uses a real C temp; a string swap can't use plain `=` at all (C arrays don't support whole-array assignment), so it goes through a temp buffer and `snprintf` instead.
- `const`.
- `if`/`elseif`/`else`/`end if` (block, single-line, and nested) — C has native `if`/`else`, so this is a direct structural translation, unlike the `basic` target, which has to transpile to a GOTO/label chain since real MBASIC/BASCOM has no block `IF`.
- `for`/`next`, `while`/`wend`, every `do`/`loop` pre-/post-check variant, and `exit` — a plain C `break;`, since C's native loops already give it the right "innermost enclosing loop" target for free, unlike the BASIC backend, which needs its own loop-exit stack because real MBASIC/BASCOM's loops are GOTO chains with no native break.
- `select case` (single-value, `to` range, and `is <op>` clauses on a numeric selector; exact-match-only on a string selector via `strcmp`) — compiled to a native `if`/`else if`/`else` chain against a once-evaluated temp, no dispatch labels needed, unlike the `basic` target.
- Scalar variables, both numeric (`%`/`&`/`!`/`#`) and string (`$`) — matching BASIC's spring-into-existence-zero-initialized semantics: every variable touched anywhere is declared once at the top of `main`.
- Every arithmetic operator (`+ - * / \ MOD ^`).
- Every comparison operator (`= <> < <= > >=`), evaluating to BASIC's own `-1`/`0`, not C's `1`/`0`.
- Every bitwise/logical operator (`AND OR XOR NOT`) — genuinely bitwise, not short-circuit booleans: C's `&`/`|`/`^`/`~` are correct here, not `&&`/`||`/`!`, since real MBASIC/BASCOM has no short-circuit boolean primitive at all.
- String concatenation (`+`).
- `&&`/`||` — BASCAL's own, already-short-circuit operators. Unlike bitwise `AND`/`OR`, C's native `&&`/`||` are the direct, correct translation here.
- `function` declarations with `byval` or `byref` scalar parameters (numeric and string), `return`, local variables, and `global`. A `byref` scalar compiles to a real C pointer, with copy-in/copy-out around it, matching the `basic` target's own `byref`/`byval` semantics. Local variables get real C function-local scope — unlike the `basic` target, no name-mangling is needed, since C's own lexical scoping already gives BASCAL's local-unless-declared-`global` semantics for free. `global` opts into reading/writing a top-level variable instead of a local one.
- `procedure` declarations — a real `void` C function, reusing the same machinery as `function`. Its body may fall through with no explicit `return`, matching real BASIC's implicit `RETURN` for `PROCEDURE`.
- A bare call statement (calling a `procedure`, or discarding a `function`'s return value).
- A suffixless (default-typed) numeric variable, filling in real MBASIC/BASCOM's own unoverridden default of single-precision floating point.
- `require`/`import` cross-file resolution — a required file's functions/procedures are merged in before either backend's codegen ever runs, so this needed no C-backend-specific work at all.
- `label:`/`goto label` — a direct 1:1 mapping onto C's own `goto`/label.
- BASIC-level `gosub label`/bare `return` (distinct from a `function`/`procedure`'s own `return`) — currently built on a small return-address-ID stack and scoped to top-level code only, not inside a `function`/`procedure` body. This legacy facility is scheduled for rejection on `--target c` (issue #110); keep it for `--target basic` and use functions/procedures for portable source.
- `on error goto label`/`on error goto 0`, `resume`/`resume next`/`resume label`, `error code`, and bare `err`/`erl` — a label handler target only, a `procedure` target isn't supported yet. Also top-level-code-only, same restriction as `gosub`/`return` and for the same reason: built on the identical return-address-ID-stack idea, just read in the opposite direction — a raise site's ID is written when it fires and read back later by whichever `resume` handles it, instead of a `gosub`'s ID being written at the call and read immediately by its own `return`. A failed sequential `open ... for input` now raises real BASIC's own error 53 this way too, instead of silently leaving a `NULL` file handle behind. `erl` reads the raising statement's own real `.bcl` source line, baked in as a compile-time literal at each raise site — not a generated `.bas` line number the way the `basic` target's own `ERL` is, since this backend never generates one, but the real line all the same.
- `try`/`catch` — portable, both-target structured error recovery. A raise reaches a `try`'s own `catch` when it happens in the `try` block itself or inside any procedure/function called, directly or transitively, from there. A caught function's result can be assigned directly but not yet embedded inside a larger expression; `try`/`catch` can't be nested.
- `data`/`read`/`restore` — a `read` target may be a bare scalar variable or a `dim`'d array element; a `data` item must be a literal number or string; `restore label` resolves to a fixed position at compile time, no runtime lookup needed.
- Twenty-five BASIC intrinsics implemented natively — `LEN`, `ASC`, `CHR$`, `MID$`, `LEFT$`, `RIGHT$`, `STR$`, `VAL`, `INSTR`, `SQR`, `ABS`, `INT`, `FIX`, `SGN`, `CINT`, `CLNG`, `CSNG`, `CDBL`, `SIN`, `COS`, `TAN`, `ATN`, `LOG`, `EXP`, and `RND` (several via a small runtime helper — see `MID_HELPER`/`INSTR_HELPER`/`SGN_HELPER`/`RND_HELPER` in `codegen_c.rs`) — plus the statement form `randomize`. A numeric seed maps straight to `srand()`; bare `randomize`/`randomize timer` both fall back to a `time()`-based seed, a real, documented divergence — real BASIC's own bare `RANDOMIZE` prompts interactively, which this backend has no model for.
- Random-access record I/O — `OPEN ... FOR RANDOM`/`BINARY` (a file that can't be opened for either — read-only, missing directory, and so on — exits cleanly with a message instead of leaving a `NULL` file handle for the next `GET`/`PUT` to crash on; not yet a trappable `try`/`catch` raise, since that failure can happen inside an arbitrary, possibly-`try`-unreachable procedure), `CLOSE`, `FIELD`, `GET`/`PUT` (whole-record form), `LSET`/`RSET`, and `MKI$`/`MKL$`/`MKS$`/`MKD$`/`CVI`/`CVL`/`CVS`/`CVD` — checked byte-for-byte compatible with real BASCOM for `int16`/`int32`/string fields (a file written by one target reads back correctly under the other; see `tests/dosbox_conformance.rs`), with two documented divergences: `MKS$`/`MKD$`/`CVS`/`CVD` use plain IEEE 754 instead of real BASIC's Microsoft Binary Format, and multi-byte values pack/unpack in the host's native (assumed little-endian) byte order.
- Sequential file I/O — `OPEN ... FOR INPUT`/`OUTPUT`/`APPEND`, `CLOSE`, `PRINT #`, `WRITE #`/`INPUT #` (a matched quoted, comma-separated format each can read back), `LINE INPUT #`, and `EOF(#ch)`.
- Interactive `INPUT` (one bare scalar variable per statement, with an optional prompt).
- `inkey$` — a real non-blocking single-keypress read, via a POSIX terminal raw-mode toggle around each individual call. Only works against an interactive terminal, and only on POSIX; scoped tightly to just that one call, not left permanently raw, since `INPUT`'s own buffered read needs real line-buffering on the same stdin.
- `stop`/`system` — both a clean process exit, via `exit(0)`, regardless of call depth.
- Screen I/O — `cls`, `locate row, col`, `color fg[, bg]`, `tab(n)`/`spc(n)` as bare `print` tokens (never as a general value — real BASCOM rejects that too), and `beep`, each mapped to a standard ANSI escape sequence rather than a platform-specific console API. `color`'s CGA-to-ANSI palette remapping lives in a small runtime helper, emitted only when `color` is actually used — see `COLOR_HELPER` in `codegen_c.rs`. The first `color` call also registers a terminal-reset-on-exit handler, so a compiled program never leaves the user's shell colored after it exits.

#### `--target c`: not yet supported

- A runtime-computed `dim` array bound — only a literal or a top-level `const` with an integer-literal value is.
- A `function` body that doesn't provably `return` on every path (a `procedure` has no such requirement).
- A `procedure` as an `on error goto` target (a label target is supported — see above).
- A `FIELD`/`OPEN`/`GET`/`PUT` channel or `FIELD` width that isn't a literal integer.
- `gosub`/`on error goto`/`resume` used inside a `function`/`procedure` body. `label`/`goto`/`read`/`restore` all work there fine — only the return-address-ID-stack techniques are scoped to top-level code, since a `return` inside a function/procedure body always means that callable's own return, leaving no unambiguous "this GOSUB's/raise site's own RETURN/RESUME" to dispatch to. `error` is the one exception: it's allowed inside a callable reachable, directly or transitively, from a top-level `try`'s own `try_body`, propagating a status back up to whichever `try` owns it instead of dispatching via a same-function `goto`.
- Everything else this page covers that isn't named above (`on ... goto`/`on ... gosub`, `mid$` statement-form assignment, `poke`/`out`, `print using`, ...).

Each unsupported form reports a "not supported yet" diagnostic rather than panicking or emitting wrong code — this is still a deliberately minimal backend, not a complete one; see the GitHub issue tracker's `c-target` label for the current, itemized list.

#### `--target c`: implementation notes

- A narrowing numeric assignment (a float/double-valued expression assigned into an integer-suffixed variable, e.g. `n% = n% / 2`) rounds, matching real MBASIC/BASCOM's own `CINT()`-style conversion (confirmed directly against real BASCOM: `N% = 27 / 2` gives `14`, not `13`) — not C's own implicit truncating conversion, which would silently give a different, wrong answer.
- A `for` loop's start/end/step are each captured into their own temp exactly once, at loop entry, matching BASIC's own "evaluated once, not re-read every iteration" semantics — a naive C `for` whose condition directly re-reads a variable the body mutates would behave differently.
- Every operator needed its exact BASIC semantics tracked down first, not assumed to be "the same as the C operator": `/` gets explicit `(double)` casts so it stays true division even between two integers (plain C `int / int` truncates); `\`/`MOD`/`AND`/`OR`/`XOR`/`NOT` round each operand first via `round()` (verified against the GW-BASIC Reference Manual), then apply C's native `/`, `%`, `&`, `|`, `^`, or `~`; `^` (exponent) maps to `pow()` from `<math.h>`.
- `round()`'s ties-away-from-zero tie-break is confirmed directly against a genuine, period-accurate IBM Personal Computer BASIC Compiler 2.00 under dosbox-x — `2.5 \ 1 = 3`, `2.5 AND 3 = 3`, matching `round()` and disagreeing with e.g. round-half-to-even or plain truncation. See `scripts/fetch-ibm-basic-compiler.sh` / [`test-fixtures/README.md`](https://github.com/johnjoeallen/bascal/blob/main/test-fixtures/README.md) if you want to check this or other real-BASCOM claims yourself — the same fixture the `basic` target's own dosbox-x conformance suite uses.
- String variables are fixed-size buffers (`char[256]`) — real BASIC strings are dynamically sized, which this backend doesn't attempt — written exclusively via `snprintf` (safely truncates an over-long value, never overflows), never `strcpy`/`strcat`.
- `%`/`&` (BASIC's 16-bit integer and 32-bit long) are collapsed to the same plain C `int`.

#### `--target jvm`

`--target jvm` is an early-stage native-JVM backend. It currently supports scalar variables and constants plus the `PI` constant, numeric arithmetic and comparisons, `ABS`/`SQR`/`INT`/`FIX`/`SGN`/`SIN`/`COS`/`TAN`/`ATN`/`LOG`/`EXP`/`RND`/`CINT`/`CLNG`/`CSNG`/`CDBL`/`VAL`/`INSTR`/`MIN`/`MAX`, string concatenation and selected string conversions including `TRIM$`/`SPACE$`/`STRING$`, `print`, `if`/`elseif`/`else`, `for`/`while`/`do` loops with `exit`, numeric or exact-string `select case`, by-value scalar functions and procedures with local variables, `global` access to top-level scalars, and initial one-/multi-dimensional integer array declarations and indexed access. It emits Krakatau assembly and can assemble/run it through `krak2` and a JRE. Complete array parameter semantics, non-integer arrays, genuine multidimensional semantics, file/screen I/O, by-reference parameters, and many standard-library operations remain unfinished; see the GitHub issue tracker's `jvm-target` label for progress.

Tutorials 01 through 08, 11 (screen I/O subset), 18 (standard-library functions), and 20 (scalar methods) currently transpile on the JVM; tutorials 01 through 08, 11, 18, and 20 also compile, assemble, and run there. Tutorial 08 validates the current integer-array declarations, indexed access, array calls, and 2-D example. Complete array parameter semantics (#112), non-integer arrays (#111), and genuine multidimensional semantics (#113) remain open. Tutorials using file/record I/O, shared COMMON state, or structured error handling remain marked Basic/C until their required features are implemented.

It emits [Krakatau](https://github.com/Storyyeller/Krakatau) assembly text (`.j`), assembled into a real `.class` by the external `krak2` tool (`v2`/Rust branch — no installed-by-default story, so `scripts/fetch-krak2.sh` builds and installs it into `~/.local`, matching how `scripts/install-bcc.sh` installs `bcc` itself), then run by any JRE's `java`.

`krak2` is looked for on `PATH` by default; point `bcc` at one living elsewhere with a `krak2=/path/to/krak2` line in `~/.config/bascal/config` or the `BASCAL_KRAK2` environment variable, the same precedence `--target` itself uses.

**Minimum JRE version: Java SE 6 (`java` 1.6) or later.** Generated `.class` files deliberately target class-file version 50 — Java SE 6's own class-file format — rather than a newer one. Class-file version 51 (Java SE 7) and up require every method with branches to carry an explicit `StackMapTable` attribute for the JVM's modern verifier to check; this backend doesn't compute those yet (see `codegen_jvm.rs`'s own module doc comment). Targeting version 50 instead means the JVM falls back to its older, slower but frame-inference-capable verifier, which works out `if`/loop branch types on its own — so `--target jvm`'s current `if`/`for`/`while`/`do`/`select case` support doesn't need real frame-tracking machinery to already exist. Any JRE from 6 onward runs a version-50 class file without issue (the JVM's class-file compatibility is backward, not forward-only), so there's no need to install anything newer than what's already on the system just to run BASCAL's JVM output.

### Up-to-Date Check

Without `--clean`, `bcc` skips re-transpiling if the output `.bas` file is newer than all input `.bcl` files. With `--binary` (or `--run`, which implies it), a second up-to-date check covers the compiled binary — and with `--run`, the already-built binary still runs even when nothing needed rebuilding.

### Library Search Order

1.  The directory containing the primary source file (always first).
2.  Paths supplied with `-L`, in the order given.

Multiple `-L` flags are supported:

```bascal
bcc tutorial/12_require.bcl -L tutorial/lib
bcc main.bcl -L libs/sort -L libs/string
```

</div>

[← Generated BASIC Shape](generated-basic-shape.md) [Statement Quick Reference →](statement-quick-reference.md)
