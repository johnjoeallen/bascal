# BASCAL

BASCAL translates structured `.bcl` source into plain 1980s Microsoft BASIC.
The compiler command is `bcc`.  See [MANUAL.md](MANUAL.md) for the full
language reference.

BASCAL keeps BASIC's global symbol model while adding enough structure to make
larger programs practical:

- multiline `if` / `else if` / `else` / `end if`
- `for` / `next`, `while` / `wend`, and `do` / `loop` loops with early exit
- `function` declarations with explicit `return`
- path-style `require` dependencies
- `program name [suite suitename]` declaration with `COMMON` block coordination
- `select case` with single values, ranges, and `is` comparisons
- `/* */` block comments flattened to `'` lines in generated output
- `input`, `data` / `read` / `restore`, `const`, `locate`, `color`, `on ... goto`
- BASIC type suffixes (`%` integer, `$` string, `!` single, `#` double, `&` long)
- source comments preserved in generated output
- generated `.bas` output using line-number `GOTO` / `GOSUB`

Everything is still global. Path-style names are linker selectors, not runtime
namespaces.

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

In 1980s BASIC, multi-program systems used `COMMON` to pass shared variables
across a `CHAIN` statement into the next program. Every chained program had to
declare an **identical** `COMMON` list or variables would land in the wrong
slots.

BASCAL coordinates this with suite files. A suite file contains only `common`
declarations; any program that names it with `suite` receives those declarations
verbatim at the top of its generated `.bas` output.

**Suite file `arcade.bcl`:**

```
' Shared state for the ARCADE suite.
common score%, level%, playerName$
common hiScore%
```

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
- A suite file may contain only `common` declarations (and comments). Functions,
  statements, and `require` are rejected.
- `common` is illegal in any file that is not a suite file.
- A `program` declaration (with or without `suite`) is illegal in library
  modules loaded via `require`.

## Generated BASIC Shape

Functions are lowered to global parameter/result variables plus `GOSUB`.
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
' Functions are lowered to global variables, labels, and GOSUB

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

## Condition Lowering

`if` and `while` conditions use `(cond) = 0` to invert, not `NOT`. This is
intentional: BASIC's `NOT` is bitwise, so `NOT 1 = -2` (still truthy), which
breaks programmer-boolean values like `swapped% = 1`. The `= 0` test treats
any non-zero as truthy, matching expected semantics.

## Recursive Functions

BASCAL does not support recursive functions. Functions are lowered to `GOSUB`
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

## Run With FreeBASIC

```bash
fbc -lang qb examples/sort_driver.bas -x tmp/sort_driver
./tmp/sort_driver
```

## Tests

```bash
env -u RUSTC_WRAPPER cargo test
```

- Unit-tests for lexer, parser, validation, and function lowering
- Compiles every driver-style `examples/**/*.bcl` file (excluding `com/`
  dependency trees) and writes `.bas` output alongside the source
- If `fbc` is installed, compiles and runs `sort_driver` and `remline`
  end-to-end

## Current Limits

- No library archive format.
- No local variable scoping; all variables inside functions are global.
- Array argument lowering uses the next argument as the element count.
