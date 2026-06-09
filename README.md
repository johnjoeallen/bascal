# BASCAL

BASCAL is a Rust compiler for a structured extension of classic Microsoft BASIC.
The compiler command is `bcc`.

BASCAL keeps BASIC's global symbol model while adding enough structure to make
larger programs practical:

- multiline `if` / `else` / `end if`
- `for` / `next` and `while` / `wend` loops
- `function` declarations with explicit `return`
- path-style `require` dependencies
- BASIC type suffixes (`%` integer, `$` string, `!` single, `#` double, `&` long)
- source comments preserved in generated output
- generated `.bas` output using line-number `GOTO` / `GOSUB`

Everything is still global. Path-style names are linker selectors, not runtime
namespaces.

## Build

```bash
env -u RUSTC_WRAPPER cargo build
```

## Usage

```bash
bcc input.bcl [-o output.bas] [-L dir] [-l library]
              [--line-numbers] [--rebuild | -r] [--binary | -b]
```

| Flag | Meaning |
|------|---------|
| `-o output.bas` | Output path (default: input with `.bas` extension) |
| `-L dir` | Add a library search directory for `require` resolution (repeatable) |
| `-l name` | Name a library (reserved for future use) |
| `--line-numbers` | Number every output line, not just branch targets |
| `--rebuild`, `-r` | Recompile even if output is already up to date |
| `--binary`, `-b` | Invoke `fbc` to compile the generated `.bas` to a binary |

The input file's directory is always the first implicit search root. `-L` adds
additional roots searched in order.

## Dependencies

`require` and `import` recursively load `.bcl` files and merge their functions
into the generated output. The two keywords are equivalent.

Path mapping: the BASIC suffix is stripped and dots become directory separators.

```
require com.bascal.sort.bubbleSort%  →  com/bascal/sort/bubbleSort.bcl
```

The input file's directory is always searched first; additional roots are added
with `-L`. Multiple `-L` flags are supported:

```bash
bcc input.bcl -L ./libs -L ./vendor
```

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
examples/   BASCAL source examples and dependency tree
output/     generated BASIC output (refreshed by tests)
tmp/        temporary compiled binaries (git-ignored)
```

## Examples

The sort driver (`examples/sort_driver.bcl`) exercises recursive `require`,
array argument passing, and timing:

```bash
bcc examples/sort_driver.bcl -o output/sort_driver.bas
```

## Run With FreeBASIC

```bash
fbc -lang qb output/sort_driver.bas -x tmp/sort_driver
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

## Tests

```bash
env -u RUSTC_WRAPPER cargo test
```

- Unit-tests for lexer, parser, validation, and function lowering
- Compiles every `examples/*.bcl` and writes output to `output/`
- If `fbc` is installed, compiles and runs `sort_driver` end-to-end

## Current Limits

- No library archive format.
- No local variable scoping; all variables inside functions are global.
- Array argument lowering uses the next argument as the element count.
