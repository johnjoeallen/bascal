[Home](../) / [Manual](../manual/) / Standard Library Functions

[← Statement Quick Reference](statement-quick-reference.md)

<div class="prose" markdown="1">

### Classic BASIC functions

BASCAL recognises the classic BASIC functions below without a declaration or `require`. The `basic` target passes them through to the target compiler. The `c` target implements the subset listed in [Backends](command-line-reference.md#backends); use that section when a program must run on both targets.

| Group                               | Functions                                                                                                                                                                                      |
|-------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Text and conversion                 | `LEN`, `ASC`, `CHR$`, `LEFT$`, `RIGHT$`, `MID$`, `INSTR`, `STR$`, `VAL`, `STRING$`, `SPACE$`, `HEX$`, `OCT$`, `FORMAT$`, `TRIM$`, `INPUT$`, `CINT`, `CLNG`, `CSNG`, `CDBL`                     |
| Math and random numbers             | `SQR`, `ABS`, `INT`, `FIX`, `SGN`, `SIN`, `COS`, `TAN`, `ATN`, `LOG`, `EXP`, `RND`                                                                                                             |
| Files, system, arrays, and printing | `EOF`, `LOF`, `LOC`, `POS`, `CSRLIN`, `FREEFILE`, `FRE`, `LPOS`, `DATE$`, `TIME$`, `TIMER`, `INKEY$`, `ENVIRON$`, `COMMAND$`, `PEEK`, `INP`, `VARPTR`, `UBOUND`, `LBOUND`, `IIF`, `TAB`, `SPC` |
| Random-access records               | `MKI$`, `MKL$`, `MKS$`, `MKD$`, `CVI`, `CVL`, `CVS`, `CVD`                                                                                                                                     |

`SIN`, `COS`, `TAN`, and `ATN` use radians. `LOG` is the natural logarithm. `RANDOMIZE` is the companion statement that seeds `RND`.

`LEFT$`, `RIGHT$`, `MID$`, `LEN`, and `INSTR` (string receiver), and `ABS`, `SQR`, `SIN`, `COS`, `TAN`, `INT`, `FIX`, and `SGN` (any numeric receiver) are also callable as scalar methods with no declaration needed — `s$.left(3)` is exactly the same call as `LEFT$(s$, 3)`. See [Built-in methods](../language/functions-and-procedures.md#built-in-methods) in the language book.

### MID\$ assignment

```bascal
MID$(target$, start[, len]) = replacement$
```

A same-length splice into `target$`, which keeps its original length — not a value-producing expression, and not the same thing as `MID$(...)` used to *read* a substring. `target$` must be a plain string variable or string array element (not, for example, a record/file DSL field or a nested call).

Despite compiling cleanly, this statement isn't reliable across every real MBASIC/BASCOM dialect BASCAL targets, so it's transpiled into a call to `com.bascal.stdlib.midAssign` — an ordinary BASCAL function, auto-added to the program (like any other `com.bascal.stdlib` symbol; see [String and error-message functions](#string-and-error-message-functions) below) the moment `MID$` assignment syntax appears anywhere, with no `require` line needed since nothing in your own source ever spells the function's name:

```bascal
function midAssign$(target$, start%, len%, value$)
    t$ = value$
    if LEN(t$) > len% then
        t$ = LEFT$(t$, len%)
    end if
    return LEFT$(target$, start% - 1) + t$ + MID$(target$, start% + LEN(t$))
end function
```

Every call site becomes an ordinary function call (`GOSUB`, in the generated BASIC) into that one shared body — the same call/return machinery every other BASCAL function goes through, so there's no separate inline-vs-shared-subroutine cutoff to reason about.

The two-argument form (`MID$(target$, start) = replacement$`) behaves as if `len` were `LEN(replacement$)`. Total `LEN(target$)` never changes — this is always a same-length overwrite, never a grow/shrink — and if `replacement$` is shorter than `len`, only that many characters are overwritten; the rest of `target$` past that point is left untouched, not padded.

### String and error-message functions

`LTRIM$`, `RTRIM$`, `UCASE$`, and `LCASE$` are not real MBASIC/BASCOM 2.00 builtins, and `ERROR$` compiles and links but silently returns an empty string at runtime instead of a real message (all verified against a real IBM Personal Computer BASIC Compiler 2.00 running under dosbox-x). BASCAL ships its own implementations, built from genuinely portable primitives (`LEFT$`/`MID$`/`LEN`/`ASC`/`CHR$`, loops — no `PEEK`/`POKE`, no `VARPTR`), as an ordinary `require`-able library under `com.bascal.stdlib` — the same mechanism as any other BASCAL library (see [Dependencies — REQUIRE and IMPORT](dependencies-require-and-import.md#dependencies-require-and-import)), not something auto-injected by call-site detection. `ltrim$`/`rtrim$`/`ucase$`/`lcase$` are declared as scalar methods, so `s$.ltrim()` and `ltrim$(s$)` both call the identical declaration (see [Built-in methods](../language/functions-and-procedures.md#built-in-methods)); `error$` stays an ordinary function, since an error code is a lookup key rather than a value the call naturally operates on:

```bascal
require com.bascal.stdlib.ltrim
require com.bascal.stdlib.rtrim
require com.bascal.stdlib.ucase
require com.bascal.stdlib.lcase
require com.bascal.stdlib.error
```

| Symbol                    | Signature       | Behavior                                                                                                                                                                                    |
|---------------------------|-----------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `com.bascal.stdlib.ltrim` | `LTRIM$(s$)`    | Strip leading spaces                                                                                                                                                                        |
| `com.bascal.stdlib.rtrim` | `RTRIM$(s$)`    | Strip trailing spaces                                                                                                                                                                       |
| `com.bascal.stdlib.ucase` | `UCASE$(s$)`    | Uppercase `a`-`z` only; other characters pass through unchanged                                                                                                                             |
| `com.bascal.stdlib.lcase` | `LCASE$(s$)`    | Lowercase `A`-`Z` only; other characters pass through unchanged                                                                                                                             |
| `com.bascal.stdlib.error` | `ERROR$(code%)` | Human-readable message for a classic MBASIC/GW-BASIC/BASCOM error code (e.g. `ERROR$(53)` → `"File not found"`); falls back to `"Error " + STR$(code%)` for a code outside its lookup table |

Each `.bcl` source file lives under `com/bascal/stdlib/` in the BASCAL distribution, and `bcc` always adds that directory to its library search path automatically — a release package ships it next to the `bcc` binary (or, for a `.deb`/`.rpm` install, under `.../share/bascal/`), so no `-L` is needed to reach it. `-L` and a same-named file next to your own source both still take priority, so you can shadow a stdlib module with your own if you ever need to.

Requiring one of these and also defining a function under the same name is a duplicate-function error, same as any other name collision between a required library and your own code — pick one.

`STRING$`, `FIX`, `HEX$`, and `OCT$` were checked the same way against real BASCOM 2.00 and *are* genuine builtins, so BASCAL passes calls to them straight through rather than reimplementing them.

</div>

[← Statement Quick Reference](statement-quick-reference.md)
