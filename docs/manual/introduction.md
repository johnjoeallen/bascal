[Home](../) / [Manual](../manual/) / Introduction

[Getting Started →](getting-started.md)

<div class="prose" markdown="1">

**BASCAL** — **B**eginner's **A**ll-purpose **S**tructured **C**omputer **A**pplication **L**anguage — is a transpiler with two backends: `--target basic` (the default, and the complete one), which translates structured `.bcl` source files into line-numbered Microsoft BASIC programs (`.bas`) compatible with BASCOM and FreeBASIC's QB compatibility mode; and `--target c`, an **experimental**, still-narrow native-C backend aiming to eventually produce native Linux/macOS/Win32 binaries directly, with no BASIC compiler involved at all — see the [Backends](command-line-reference.md#backends) section of the [Command-Line Reference](command-line-reference.md) for exactly what it supports today. Everything else in this manual describes the `basic` target, unless a section says otherwise.

BASCAL adds structured programming constructs on top of BASIC's run-time semantics:

- Block `if` / `elseif` / `else` / `end if`
- `for` / `end for`, `while` / `end while`, and `do` / `end do` loops with early exit
- `function` declarations with typed return values and explicit `return`
- `procedure` declarations for action subroutines with no return value
- Path-style `require` for multi-file projects
- `program` / `library` / `shared` declarations, the last coordinating `COMMON` across chained programs
- Multi-line `/* */` block comments and `//` end-of-line comments in addition to the classic `'` comment
- `select case` with range and `is` comparisons
- All classic BASCOM 1980s statements: `DATA`/`READ`/`RESTORE`, `LOCATE`, `COLOR`, `ON ... GOTO`, `SWAP`, `RANDOMIZE`, `CONST`, and more

For the `basic` target, BASCAL does not invent a new runtime: every BASCAL program transpiles to plain Microsoft BASIC and runs under whatever BASIC you already have (BASCOM, FreeBASIC's QB compatibility mode). The structured constructs are transpiled directly onto BASIC's own control flow — functions become `GOSUB` subroutines, loops become `GOTO`-based constructs, and `if` chains become `IF ... THEN GOTO` sequences.

For the `c` target, the opposite is true: there's no BASIC runtime underneath at all. `if`/`for`/`while` compile to C's own native control flow, and BASCAL's C backend has its own, different runtime rather than reusing BASIC's — this backend is experimental and still narrow, so not everything carries over. Statements whose behavior depends on BASIC's own runtime — `CHAIN` being the clearest example, since it depends on BASIC's program-swapping semantics — aren't supported under `--target c`. See the [Backends](command-line-reference.md#backends) section of the [Command-Line Reference](command-line-reference.md) for exactly what is and isn't supported today.

**BASCAL is a strict superset of classic BASIC.** Raw statements from the target dialect — `OPEN`/`FIELD`/`GET`/`PUT` for random-access files, bitwise `AND`/`OR`/`NOT` — still pass through unchanged. `GOTO`/`GOSUB`/`ON ERROR GOTO`/ `RESUME`/`RESTORE` are raw BASIC too, but with one restriction: BASCAL manages line numbering itself, so their targets must be a `name:` label declared in source, never a raw line number — see [Labels](miscellaneous-statements.md#labels). Beyond that, wherever this manual documents a BASCAL construct for something (`select case` instead of an `IF`/`GOTO` dispatch chain, `record`/`file` instead of hand-written `FIELD`/`GET`/`PUT`, `&&`/`||` instead of bitwise short-circuit workarounds), treat that construct as the canonical way to write it in `.bcl` source — the original BASIC syntax is what the transpiler exists to get you away from, not an equally good alternative.

</div>

[Getting Started →](getting-started.md)
