[Home](../../) / [Manual](../) / Variables and Constants

[← Data Types and Type Suffixes](data-types-and-type-suffixes.md) [Operators and Expressions →](operators-and-expressions.md)

<div class="prose" markdown="1">

### Variables

Variables declared or assigned at the top level are **global** and visible throughout the entire program.

Variables inside a `function` or `procedure` body are **local by default**: the transpiler maps them to uniquely-generated BASIC names (e.g. `fnameVar0%`), indexed against every name already in use at transpile time so they're guaranteed never to collide with global variables or with locals in other functions. To read or write a global variable from inside a function or procedure, declare it at the top of the body with the `global` keyword:

```bascal
total% = 0

' x% -- amount to add to the running total
function addToTotal%(x%)
    global total%           ' access the global variable, not a local one
    total% = total% + x%
    return total%
end function
```

BASIC builtin functions (`UCASE$`, `STR$`, `LEN`, etc.) are always recognised as callables and are never treated as local variables.

Variables do not require pre-declaration; they come into existence on first assignment. Use `DIM` (or its synonym `DECLARE`) to declare arrays or to make intent clear. Compiling with [the `--strict-vars` flag](command-line-reference.md) turns that requirement on for real, rejecting any scalar/array variable used without a matching `DIM`/`DECLARE` (a `CONST`, a `FOR` loop's own counter, and a function/procedure parameter all still count) — useful as a safety net against a misspelled name silently becoming a new, separately-zeroed variable, but opt-in, since it's no longer a strict superset of BASIC once it's on.

### DIM

Declares an array or a simple variable. `declare` is an interchangeable synonym for `dim` — they parse to exactly the same statement and generate identical output. A reasonable convention: `declare` for a plain scalar (it reads as "declare this variable"), `dim` for an array (it keeps BASIC's own "dimension this array" sense) — but nothing enforces the split, and both spellings freely mix within one program.

```bascal
declare playerName$
dim scores%(100)       ' 1-D: 101 elements, scores%(0) .. scores%(100)
dim grid%(9, 9)        ' 2-D: 10×10 grid, grid%(row, col)
dim cube%(3, 4, 5)     ' 3-D: up to 8 dimensions supported
```

The bounds expression for each dimension may be any integer expression, including a constant. Elements are indexed from 0 to *bound* in each dimension (following `OPTION BASE 0`, the default):

```bascal
const rows% = 4
const cols% = 4
dim matrix%(rows% - 1, cols% - 1)

for r% = 0 to rows% - 1
    for c% = 0 to cols% - 1
        matrix%(r%, c%) = r% * cols% + c%
    end for
end for
```

`dim name%()` (empty parens) declares an array without specifying bounds — use this when the array will be passed in from outside or when BASIC's default sizing is sufficient.

A single `dim` may declare more than one name, comma-separated, mixing plain variables and arrays freely:

```bascal
dim a%, b%(3), c$
```

This is exactly equivalent to writing three separate `dim` statements — `dim a%`, then `dim b%(3)`, then `dim c$` — and generates one `DIM` line per name in the output.

### OPTION BASE

**`OPTION BASE` is rejected outright — it's a transpile-time error under both targets, not just an unsupported-but-accepted construct.** Every array is indexed from base 0; declare every array that way. If you're porting source that uses `OPTION BASE 1`, remove the statement and shift every index by one instead.

### ERASE

Do not use `ERASE` to release and re-declare an array in compiled classic BASIC. BASCOM-family compilers keep the array's declaration fixed, so a later `DIM` is rejected. Keep the original fixed-size array instead.

```bascal
dim bigTable%(1000, 200)
' ... use bigTable% ...
erase bigTable%          ' do not follow with another DIM in compiled BASIC

dim names$(50), codes%(50)
' ... use both ...
erase names$, codes%     ' erase multiple at once
```

### CONST

Declares a named constant. The value must be a literal. Constant types are
inferred from the value, so a constant name does not need a BASIC type suffix.
Use uppercase `SNAKE_CASE` names; non-compliant names (and legacy suffixes)
produce warnings only.

```bascal
CONST PASS_MARK  = 60
CONST APP_NAME   = "Grade Checker"
CONST PI         = 3.14159
CONST TAX_RATE   = 0.2
```

Constants follow the same type-suffix rules as variables. Once declared, a constant may not be reassigned.

From `tutorial/variables.bcl`:

```bascal
CONST PASS_MARK%  = 60
CONST APP_NAME$   = "Grade Checker"

score%       = 87
playerName$  = "Alice"

if score% >= PASS_MARK% then
    PRINT APP_NAME$ + ": " + playerName$ + " passed with " + STR$(score%)
end if
```

</div>

[← Data Types and Type Suffixes](data-types-and-type-suffixes.md) [Operators and Expressions →](operators-and-expressions.md)
