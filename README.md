# BASCAL

BASCAL is a Rust compiler for a structured extension of classic Microsoft BASIC.
The compiler command is `bcc`.

BASCAL keeps BASIC's global symbol model while adding enough structure to make
larger programs practical:

- multiline `if` / `else` / `end if`
- `function` declarations with explicit `return`
- path-style `require` dependencies
- BASIC type suffixes such as `%` and `$`
- retained BASIC-style comments
- generated `.bas` output using line-number `GOTO` / `GOSUB`

Everything is still global. Path-style names are linker selectors, not runtime
namespaces. For example:

```basic
require com.bascal.sort.bubbleSort%
```

loads a dependency, but the callable BASIC symbol is still:

```basic
bubbleSort%(data%(), 10)
```

## Build

```bash
env -u RUSTC_WRAPPER cargo build
```

The `env -u RUSTC_WRAPPER` prefix is only needed in environments where `sccache`
is blocked.

## Repository Layout

```text
src/        Rust compiler source
examples/   BASCAL source examples and dependency tree
output/     generated BASIC output from examples/tests
tmp/        temporary compiled binaries
```

`tmp/` is ignored by git. Generated `.bas` files in `output/` are useful for
inspection and are refreshed by the test suite.

## Compile BASCAL

```bash
target/debug/bcc examples/sort_driver.bcl -o output/sort_driver.bas
```

If `-o` is omitted, `bcc` writes a `.bas` file next to the input:

```bash
target/debug/bcc examples/add.bcl
```

Reserved search flags are accepted:

```bash
target/debug/bcc input.bcl -I ./src -L ./libs -l stdlib
```

`-I` and `-L` are used by the current dependency lookup. `-l` is accepted for
future library support.

## Dependencies

`require` and `import` recursively load `.bcl` files and merge their functions
into the generated BASIC program.

Current path mapping strips the BASIC suffix and maps dots to directories:

```basic
require com.bascal.sort.bubbleSort%
```

resolves as:

```text
com/bascal/sort/bubbleSort.bcl
```

The input file's directory is an implicit search root. Additional roots can be
provided with `-I` and `-L`.

## Generated BASIC

Functions are lowered to global parameter/result variables plus `GOSUB`.
Array arguments are copied into lowered function arrays before the call and
copied back afterward.

BASCAL source comments beginning with `'` are retained in generated BASIC. This
is useful because linked dependencies are merged into one output file, and the
comments preserve context around generated sections.

Example BASCAL:

```basic
function add%(left%, right%)
    return left% + right%
end function

total% = add%(10, 20)
PRINT total%
END
```

Generated shape:

```basic
add_left% = 10
add_right% = 20
GOSUB 10
total% = add_result%
PRINT total%
END
' ===== BEGIN FUNCTION add% =====
10     add_result% = add_left% + add_right%
    RETURN
' ===== END FUNCTION add% =====
```

Only `GOTO` / `GOSUB` target lines receive line numbers.

## Examples

Example BASCAL programs live in:

```text
examples/
```

The example dependency tree uses the `com.bascal` namespace selector:

```text
examples/com/bascal/sort/
```

Generated BASIC outputs are written to:

```text
output/
```

Temporary compiled binaries are written to:

```text
tmp/
```

The sort driver demonstrates recursive `require` declarations and four sort
routines:

```bash
target/debug/bcc examples/sort_driver.bcl -o output/sort_driver.bas
```

The generated `output/sort_driver.bas` is a complete BASIC program containing
the main driver plus the recursively loaded sort functions.

## Run With FreeBASIC

FreeBASIC can compile the generated BASIC in QB compatibility mode:

```bash
fbc -lang qb output/sort_driver.bas -x tmp/sort_driver_fbc
./tmp/sort_driver_fbc
```

Expected output includes sorted results for bubble, shaker, shell, and quick
sort sections:

```text
1
3
7
12
19
21
34
42
55
88
```

`quickSort%` is currently implemented as selection sort. For compatibility with
the way 1980s BASIC handled `GOSUB` and global state, BASCAL does not support
recursive functions either.

## Tests

```bash
env -u RUSTC_WRAPPER cargo test
```

The test suite:

- unit-tests lexer, parser, validation, and function lowering
- compiles every `examples/*.bcl`
- writes generated `.bas` files to `output/`
- if `fbc` is installed, compiles `output/sort_driver.bas` to `tmp/` and runs it

## Current Limits

- No library archive format yet.
- No transitive recursion analysis beyond simple validation.
- Function lowering is global and non-recursive for compatibility with the way
  1980s BASIC handled subroutines.
- Array argument lowering uses copy-in/copy-out based on the following count
  argument.
