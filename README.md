# BASCAL

<img src="docs/hero-computer.svg" width="220" align="right" alt="A retro all-in-one computer showing green phosphor BASIC code, with a violet cursor blinking at the prompt">

**B**eginner's **A**ll-purpose **S**tructured **C**omputer **A**pplication
**L**anguage.

*A fun project, built mostly to see what's possible — not really meant for
building a real 2026 application. See [the origin story](https://johnjoeallen.github.io/bascal/origin.html).*

**[Browse the website](https://johnjoeallen.github.io/bascal/)** — tutorials,
side-by-side syntax comparisons, and the full [language manual](https://johnjoeallen.github.io/bascal/manual.html).

BASCAL translates structured `.bcl` source into plain 1980s Microsoft BASIC.
The compiler command is `bcc`.  See [MANUAL.md](MANUAL.md) for the full
language reference.

BASCAL keeps BASIC's global symbol model while adding enough structure to make
larger programs practical:

- multiline `if` / `else if` / `else` / `end if`
- `for` / `next`, `while` / `wend`, and `do` loops with early exit — `do`
  comes pre-check (`do [while/until cond] ... end do`) or post-check
  (`do ... loop [while/until cond]`, BASCAL's `repeat`/`until` equivalent)
- `function` declarations with explicit `return`
- path-style `require` dependencies
- `program name [suite suitename]` declaration with `COMMON` block coordination
- `select case` with single values, ranges, and `is` comparisons
- `&&` / `||` short-circuit operators for `if`/`elseif`/`while`/`do` conditions
  — unlike bitwise `AND`/`OR`, the second operand is only evaluated when it
  can still change the answer
- `+=` / `-=` / `*=` / `/=` compound assignment, `TRUE` / `FALSE` literals
  (`-1` / `0`), and comma-separated multi-name `dim a%, b%(3), c$`
- `/* */` block comments flattened to `'` lines in generated output
- `input`, `data` / `read` / `restore`, `const`, `locate`, `color`, `on ... goto`
- BASIC type suffixes (`%` integer, `$` string, `!` single, `#` double, `&` long)
- source comments preserved in generated output
- generated `.bas` output using line-number `GOTO` / `GOSUB`
- `record` / `file` DSL: typed fixed-layout records (`int16`, `int32`,
  `float32`, `float64`, `string(N)`) with `file db as T = open(path)`,
  `db[i] = { ... }` / `db[i] = ?{ ... }` (partial), `let s = db[i]`,
  `db[i].field = value`, `db[i] = s`, and `db.close()`, all transpiled to plain
  `FIELD`/`PUT`/`GET`/`LSET`/`MKx`/`CVx` — see
  [MANUAL.md](MANUAL.md#record-files) and
  [`tutorial/15_random_and_record_files.bcl`](tutorial/15_random_and_record_files.bcl)

Everything is still global. Path-style names are dependency selectors, not
runtime namespaces.

BASCAL is a strict superset of classic BASIC — bitwise `AND`/`OR`/`NOT` and
hand-written `OPEN`/`FIELD`/`GET`/`PUT` all still compile unchanged.
`GOTO`/`GOSUB`/`ON ERROR GOTO`/`RESUME`/`RESTORE` are raw BASIC too, but
BASCAL manages line numbering itself, so their targets are always a `name:`
label declared in source — never a raw line number. Beyond that, wherever
BASCAL has its own construct for something above, treat that construct as
canonical: it's the syntax the compiler exists to let you write instead of
the raw-BASIC equivalent, not just another option.

## Build

```bash
env -u RUSTC_WRAPPER cargo build
```

## Release Packages

GitHub Actions builds release packages from `.github/workflows/packages.yml`.
Run the **Packages** workflow manually to produce downloadable artifacts, or
push a `v*` tag such as `v0.1.0` to attach the Debian `.deb`, RPM, Linux
`.tar.gz`, and Windows `.zip` packages to a GitHub Release.

## Usage

```bash
bcc input.bcl [-o output.bas] [-L dir] [-l library]
              [--line-numbers] [--clean | -c] [--binary | -b]
```

| Flag | Meaning |
|------|---------|
| `-o output.bas` | Output path (default: input with `.bas` extension, same directory) |
| `-L dir` | Add a library search directory for `require` resolution (repeatable) |
| `-l name` | Name a library (reserved for future use) |
| `--line-numbers` | Number every output line, not just branch targets |
| `--clean`, `-c` | Recompile even if output is already up to date |
| `--binary`, `-b` | Invoke `fbc` to compile the generated `.bas` to a binary in `tmp/` |

The input file's directory is always the first implicit search root. `-L` adds
additional roots searched in order.

## Dependencies

`require` and `import` recursively load `.bcl` files and merge their functions
into the generated output. The two keywords are equivalent.

Dots become directory separators:

```
require com.bascal.sort.bubbleSort  →  com/bascal/sort/bubbleSort.bcl
```

The input file's directory is always searched first; additional roots are added
with `-L`:

```bash
bcc input.bcl -L ./libs -L ./vendor
```

## Suite COMMON

In 1980s BASIC, multi-program systems used `COMMON` to declare shared variable
slots that survive a `CHAIN` into the next program. Every chained program had to
declare an **identical** `COMMON` list or variables would land in the wrong
slots.

BASCAL coordinates this with suite files. A suite file contains only variable
declarations; any program that names it with `suite` receives those declarations
verbatim at the top of its generated `.bas` output.

**Suite file `arcade.bcl`:**

```
' Shared state for the ARCADE suite.
suite arcade

dim score%
dim level%
dim playerName$
dim hiScore%
```

The `suite <name>` header (mirroring a regular file's `program <name>`) plus
plain `dim` declarations is the recommended form. The older spelling — no
header, just `common score%, level%, playerName$` / `common hiScore%`, with
the suite name taken from the filename alone — still works and compiles to
identical output.

**Program files:**

```
program menu suite arcade

INPUT "Your name: "; playerName$
score% = 0
level% = 1
' CHAIN "game.bas"
END
```

```
program game suite arcade

score% = score% + 50 * level%
PRINT "Score: " + STR$(score%)
' CHAIN "menu.bas"
END
```

Both compile to `.bas` files that open with the same block:

```
COMMON score%, level%, playerName$
COMMON hiScore%
```

Rules:
- A suite file may contain only `dim`/`common` declarations (and comments).
  Functions, statements, and `require` are rejected.
- `common` and a `suite <name>` header are both illegal in any file that
  isn't a suite file.
- A file can't have both a `program` header and a `suite` header.
- A `program` declaration (with or without `suite`) is illegal in library
  modules loaded via `require`.

## Generated BASIC Shape

Functions are transpiled to global parameter/result variables plus `GOSUB`.
Array arguments use copy-in/copy-out around the call.

Only `GOTO` / `GOSUB` target lines receive line numbers (sparse mode). Use
`--line-numbers` for every line.

Source blank lines are preserved in the output. Multiple consecutive blank lines
are folded to one. Generated array-copy blocks are surrounded by blank lines.

Example BASCAL:

```
function add%(left%, right%)
    return left% + right%
end function

total% = add%(10, 20)
PRINT total%
END
```

Generated output:

```
' BASCAL generated BASIC
' Functions are transpiled to global variables, labels, and GOSUB

add_left% = 10
add_right% = 20
GOSUB 10
total% = add_result%
PRINT total%
END

' function add%(left%, right%)
10 add_result% = add_left% + add_right%
    RETURN
' end function add%
```

## Condition Transpilation

`if` and `while` conditions use `(cond) = 0` to invert, not `NOT`. This is
intentional: BASIC's `NOT` is bitwise, so `NOT 1 = -2` (still truthy), which
breaks programmer-boolean values like `swapped% = 1`. The `= 0` test treats
any non-zero as truthy, matching expected semantics.

## Recursive Functions

BASCAL does not support recursive functions. Functions are transpiled to `GOSUB`
with global parameter variables; a recursive call would overwrite its own
parameters. Use an explicit stack array to simulate recursion.

## Repository Layout

```
src/        Rust compiler source
examples/   BASCAL source examples (.bas generated alongside each .bcl)
tmp/        temporary compiled binaries (git-ignored)
```

## Examples

### Sort driver

`examples/sort_driver.bcl` exercises recursive `require`, array argument
passing, and timing:

```bash
cargo run -- examples/sort_driver.bcl
fbc -lang qb examples/sort_driver.bas -x tmp/sort_driver
./tmp/sort_driver
```

Expected output (5000 reverse-sorted elements):

```
Bubble sort time (ms):       ~200
Bubble: OK
Shaker sort time (ms):       ~180
Shaker: OK
Shell sort time (ms):        ~1
Shell: OK
Quick sort time (ms):        ~1
Quick: OK
```

### REMLINE

`examples/remline` is a real-world BASCAL example inspired by old BASIC
line-number utilities. It analyses a line-numbered BASIC program and removes
unnecessary line numbers while preserving referenced targets. The generated
program reads `examples/remline/sample/input.bas` and writes the cleaned
listing to `examples/remline/sample/output.bas`.

```bash
cargo run -- examples/remline/remline.bcl -L examples/remline
fbc -lang qb examples/remline/remline.bas -x tmp/remline
./tmp/remline
diff -u examples/remline/sample/expected.bas examples/remline/sample/output.bas
```

### Arcade suite

`examples/arcade` demonstrates suite `COMMON` coordination across two programs
that share score, level, and player state.

```bash
cargo run -- examples/arcade/menu.bcl
cargo run -- examples/arcade/game.bcl
```

Both generated `.bas` files open with the same `COMMON` block drawn from
`arcade.bcl`, ready to exchange state via `CHAIN`.

### Card catalog

`tutorial/card_catalog.bcl` is the flagship record/file DSL example: two
record types (`Header`, `Entry`) sharing one random-access file, and five
`procedure`s (`addItem`, `listAll`, `searchByAuthor`, `searchByAuthorTitle`,
`deleteItem`) that each read and write those records from inside their own
body — exercising record/file access from procedure scope, not just
top-level code. A `mainMenu` procedure drives them from an interactive,
`INPUT`-based menu loop. It's adapted from `CLERK.BAS`, a 1983 card-catalog
manager by Carlos A. Lujan S.; see the comment header in the source for the
full attribution and porting notes.

```bash
cargo run -- tutorial/card_catalog.bcl
fbc -lang qb tutorial/card_catalog.bas -x tmp/card_catalog
./tmp/card_catalog   # interactive -- follow the on-screen menu
```

## Run With FreeBASIC

```bash
fbc -lang qb examples/sort_driver.bas -x tmp/sort_driver
./tmp/sort_driver
```

## Tests

```bash
env -u RUSTC_WRAPPER cargo test
```

- Unit-tests for lexer, parser, validation, and function transpilation
- Compiles every driver-style `examples/**/*.bcl` file (excluding `com/`
  dependency trees) and writes `.bas` output alongside the source
- If `fbc` is installed, compiles and runs `sort_driver` and `remline`
  end-to-end

## Current Limits

- No library archive format.
- Array argument transpilation uses the parameter declared right after the
  array as its element count for copy-in/copy-out — an array carries no
  length of its own, so that ordering is required, not just conventional.
