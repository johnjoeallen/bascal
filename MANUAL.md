# BASCAL Language Reference Manual

**BASCAL Transpiler (bcc) — Version 0.1**

---

## Table of Contents

1. [Introduction](#introduction)
2. [Getting Started](#getting-started)
3. [Program Structure](#program-structure)
4. [Data Types and Type Suffixes](#data-types-and-type-suffixes)
5. [Variables and Constants](#variables-and-constants)
6. [Operators and Expressions](#operators-and-expressions)
7. [Comments](#comments)
8. [Control Flow](#control-flow)
9. [Functions](#functions)
10. [Procedures](#procedures)
11. [Arrays](#arrays)
12. [Input and Output](#input-and-output)
13. [File Input and Output](#file-input-and-output)
14. [Random-Access File I/O](#random-access-file-io)
15. [Record Files](#record-files)
16. [Data Statements](#data-statements)
17. [Miscellaneous Statements](#miscellaneous-statements)
18. [Dependencies — REQUIRE and IMPORT](#dependencies-require-and-import)
19. [Shared COMMON](#shared-common)
20. [Generated BASIC Shape](#generated-basic-shape)
21. [Command-Line Reference](#command-line-reference)
22. [Statement Quick Reference](#statement-quick-reference)
23. [Standard Library Functions](#standard-library-functions)

---

## Introduction

**BASCAL** — **B**eginner's **A**ll-purpose **S**tructured **C**omputer
**A**pplication **L**anguage — is a transpiler that translates structured
`.bcl` source files into line-numbered Microsoft BASIC programs (`.bas`)
compatible with BASCOM and FreeBASIC's QB compatibility mode.

BASCAL adds structured programming constructs on top of BASIC's run-time
semantics:

- Block `if` / `elseif` / `else` / `end if`
- `for` / `end for`, `while` / `end while`, and `do` / `end do` loops with early exit
- `function` declarations with typed return values and explicit `return`
- `procedure` declarations for action subroutines with no return value
- Path-style `require` for multi-file projects
- `program` / `library` / `shared` declarations, the last coordinating
  `COMMON` across chained programs
- Multi-line `/* */` block comments and `//` end-of-line comments in addition
  to the classic `'` comment
- `select case` with range and `is` comparisons
- All classic BASCOM 1980s statements: `DATA`/`READ`/`RESTORE`, `LOCATE`,
  `COLOR`, `ON ... GOTO`, `SWAP`, `RANDOMIZE`, `CONST`, and more

**BASCAL does not invent a new runtime.** Every BASCAL program transpiles to
plain Microsoft BASIC. The structured constructs are transpiled as follows:
functions become `GOSUB` subroutines, loops become `GOTO`-based constructs,
and `if` chains become `IF ... THEN GOTO` sequences.

That's the `basic` target -- the complete one, and what the rest of this
manual describes throughout. BASCAL also has a second, **experimental**
target: `--target c`, a native-C backend aiming to eventually produce
native Linux/macOS/Win32 binaries directly, without going through a BASIC
compiler at all. It's still narrow (no arrays, functions, or `select
case` yet) -- see the [Backends](#backends) section of the
[Command-Line Reference](#command-line-reference) for exactly what it
supports today.

**BASCAL is a strict superset of classic BASIC.** Raw statements from the
target dialect — `OPEN`/`FIELD`/`GET`/`PUT` for random-access files, bitwise
`AND`/`OR`/`NOT` — still pass through unchanged. `GOTO`/`GOSUB`/`ON ERROR GOTO`/
`RESUME`/`RESTORE` are raw BASIC too, but with one restriction: BASCAL
manages line numbering itself, so their targets must be a `name:` label
declared in source, never a raw line number — see [Labels](#labels). Beyond
that, wherever this manual documents a BASCAL construct for something
(`select case` instead of an `IF`/`GOTO` dispatch chain, `record`/`file`
instead of hand-written `FIELD`/`GET`/`PUT`, `&&`/`||` instead of bitwise
short-circuit workarounds), treat that construct as the canonical way to
write it in `.bcl` source — the original BASIC syntax is what the
transpiler exists to get you away from, not an equally good alternative.

---

## Getting Started

### Building bcc

```
env -u RUSTC_WRAPPER cargo build --release
```

The compiled binary is `target/release/bcc`.

### Your First Program

The file `tutorial/01_hello.bcl` demonstrates all three comment styles and
a basic PRINT/END structure:

```
// Tutorial 1 — Hello, World
' This is a classic single-quote comment (passes through to BASIC as-is).
// This is a double-slash end-of-line comment (same behaviour).

/*
 * Block comments span multiple lines.  Each line is emitted as a separate
 * ' comment in the generated output; blank lines are preserved as blank lines.
 */

PRINT "Hello, World!"
PRINT "Welcome to BASCAL."
END
```

Transpile it:

```
bcc tutorial/01_hello.bcl
```

This produces `tutorial/01_hello.bas`. To compile and run with FreeBASIC:

```
bcc tutorial/01_hello.bcl --binary
./tmp/01_hello
```

### A Simple Function

```
' name$ -- who to greet
function greet$(name$)
    return "Hello, " + name$ + "!"
end function

msg$ = greet$("BASCOM")
PRINT msg$
END
```

---

## Program Structure

Every `.bcl` file is exactly one of three things, declared by a mandatory
header on its first non-comment, non-blank line:

| Header | File is a... | May be `require`d? | May itself `require`? |
|--------|---------------|---------------------|------------------------|
| `program name` | runnable program (the file you hand to `bcc`) | no | yes |
| `library name` | library module | yes — only files with this header may | yes |
| `shared name` | shared-variables file (see [Shared COMMON](#shared-common)) | no (resolved via `program ... shared name`, not `require`) | no |

A file with no header, or with more than one of these, is a transpile-time
error. `require`/`import` targets a file that must declare `library`; a
`program name shared sharedname` clause resolves its shared file through a
separate lookup, not through `require`.

Beyond the header, a `.bcl` file consists of optional sections in the
following order:

1. Mandatory `program` / `library` / `shared` declaration
2. `require` / `import` dependency declarations (`program`/`library` files
   only)
3. Top-level statements (the main program body; a `shared` file's body is
   `dim` declarations only — every variable in it is COMMON by default, see
   [Shared COMMON](#shared-common) — and a `library` file should stick to
   `function`/`procedure` definitions and supporting `dim`/`data`, see
   [Module Conventions](#module-conventions))
4. `function` definitions (may appear in any order relative to statements)

### Program Declaration

```
program name
program name shared sharedname
```

Identifies the file as a runnable program, by name, and optionally links it
to a shared-variables file (see [Shared COMMON](#shared-common)). Required in
every file that isn't a `library` or `shared` file — in particular, the file
passed to `bcc` on the command line must have one.

A `program` declaration is **not allowed** in library modules loaded via
`require`.

### Library Declaration

```
library name
```

Identifies the file as a library module — the only kind of file `require`/
`import` may load. From `com/bascal/stdlib/ucase.bcl`:

```
// Upper-cases s$. Not a real MBASIC/BASCOM 2.00 builtin -- verified against
// a real IBM BASIC Compiler 2.00 under dosbox-x -- so BASCAL ships its own.
library ucase

function ucase$(s$)
    ...
```

The name isn't validated against anything (unlike `shared name`, which must
match the resolved shared file's filename) — it's documentation, not a lookup
key.
A `library` declaration is **not allowed** in the root file `bcc` was
invoked on, and a file `require`d/`import`ed without one is a transpile-time
error (see [Module Conventions](#module-conventions)).

### File Encoding

Source files are UTF-8 text. Line endings may be LF or CRLF. Statements are
separated by newlines; a colon `:` may also separate statements on one line.

There is no line-continuation syntax -- no trailing `_`, `\`, or similar. The
lexer turns every physical newline into a real token, so any single
expression (and the tokens making up a statement header, like a `case`
value list) must fit on one physical line; a newline there ends the
statement/expression rather than continuing it. This applies everywhere,
not just to obviously statement-like constructs -- a function call's
argument list, for example, can't have a newline before its closing `)`
either:

```
' Not allowed -- the newline after "1," ends the statement early:
result% = someFunction(1,
                        2)

' Allowed -- keep the whole call on one line:
result% = someFunction(1, 2)
```

The same rule governs statement headers like `if`/`then`: the condition and
the `then` that closes it must be on one physical line. A newline can only
appear *after* `then`, where it's meaningful -- it's what selects the block
form of `if` over the single-line form (see
[IF / ELSEIF / ELSE / END IF](#if-elseif-else-end-if)):

```
' Not allowed -- the newline before "then" ends the statement early:
if score% >= 60
    then PRINT "Pass"

' Not allowed -- same problem, condition split across lines:
if score% >= 60 and
    attendance% >= 80 then PRINT "Pass"

' Allowed -- condition and "then" on one line, body starts after the newline:
if score% >= 60 and attendance% >= 80 then
    PRINT "Pass"
end if
```

---

## Data Types and Type Suffixes

BASCAL uses Microsoft BASIC's type-suffix convention. Every variable or
function name carries its type in the final character:

| Suffix | Type    | Range / Notes                            |
|--------|---------|------------------------------------------|
| `%`    | Integer | 16-bit signed, -32768 to 32767           |
| `$`    | String  | Variable-length string                   |
| `!`    | Single  | 32-bit IEEE 754 single-precision float   |
| `#`    | Double  | 64-bit IEEE 754 double-precision float   |
| `&`    | Long    | 32-bit signed integer                    |

Variables without a suffix follow the DEFtype settings of the BASIC runtime
(default: single precision). In BASCAL source it is strongly recommended to
always use explicit suffixes.

All type checking is deferred to the BASIC runtime. The BASCAL transpiler does
not perform static type inference.

---

## Variables and Constants

### Variables

Variables declared or assigned at the top level are **global** and visible
throughout the entire program.

Variables inside a `function` or `procedure` body are **local by default**: the
transpiler maps them to uniquely-generated BASIC names (e.g. `fnameVar0%`),
indexed against every name already in use at transpile time so they're
guaranteed never to collide with global variables or with locals in other
functions. To read or write a global variable from inside a function or
procedure, declare it at the top of the body with the `global` keyword:

```
total% = 0

' x% -- amount to add to the running total
function addToTotal%(x%)
    global total%           ' access the global variable, not a local one
    total% = total% + x%
    return total%
end function
```

BASIC builtin functions (`UCASE$`, `STR$`, `LEN`, etc.) are always recognised as
callables and are never treated as local variables.

Variables do not require pre-declaration; they come into existence on first
assignment. Use `DIM` to declare arrays or to make intent clear.

### DIM

Declares an array or a simple variable.

```
dim playerName$
dim scores%(100)       ' 1-D: 101 elements, scores%(0) .. scores%(100)
dim grid%(9, 9)        ' 2-D: 10×10 grid, grid%(row, col)
dim cube%(3, 4, 5)     ' 3-D: up to 8 dimensions supported
```

The bounds expression for each dimension may be any integer expression,
including a constant. Elements are indexed from 0 to *bound* in each
dimension (following `OPTION BASE 0`, the default):

```
const rows% = 4
const cols% = 4
dim matrix%(rows% - 1, cols% - 1)

for r% = 0 to rows% - 1
    for c% = 0 to cols% - 1
        matrix%(r%, c%) = r% * cols% + c%
    end for
end for
```

`dim name%()` (empty parens) declares an array without specifying bounds — use
this when the array will be passed in from outside or when BASIC's default
sizing is sufficient.

A single `dim` may declare more than one name, comma-separated, mixing plain
variables and arrays freely:

```
dim a%, b%(3), c$
```

This is exactly equivalent to writing three separate `dim` statements —
`dim a%`, then `dim b%(3)`, then `dim c$` — and generates one `DIM` line per
name in the output.

### OPTION BASE

Sets the default lower bound for all subsequently declared arrays. Must be
placed before any `DIM` statements it is meant to affect.

```
option base 0   ' arrays start at index 0 (default)
option base 1   ' arrays start at index 1
```

With `OPTION BASE 1`, `dim scores%(10)` allocates 10 elements indexed 1..10
instead of 11 elements indexed 0..10.

### ERASE

Frees memory occupied by one or more arrays. After `ERASE`, the array
variables are undefined until re-declared.

```
dim bigTable%(1000, 200)
' ... use bigTable% ...
erase bigTable%          ' release memory

dim names$(50), codes%(50)
' ... use both ...
erase names$, codes%     ' erase multiple at once
```

### CONST

Declares a named constant. The value must be a literal.

```
CONST PASS_MARK%  = 60
CONST APP_NAME$   = "Grade Checker"
CONST PI!         = 3.14159
CONST TAX_RATE!   = 0.2
```

Constants follow the same type-suffix rules as variables. Once declared, a
constant may not be reassigned.

From `tutorial/02_variables.bcl`:

```
CONST PASS_MARK%  = 60
CONST APP_NAME$   = "Grade Checker"

score%       = 87
playerName$  = "Alice"

if score% >= PASS_MARK% then
    PRINT APP_NAME$ + ": " + playerName$ + " passed with " + STR$(score%)
end if
```

---

## Operators and Expressions

### Arithmetic Operators

| Operator | Operation      |
|----------|----------------|
| `+`      | Addition / string concatenation |
| `-`      | Subtraction / unary negation    |
| `*`      | Multiplication |
| `/`      | Division (always floating-point, even between two integers) |
| `\`      | Integer division (each operand is rounded to an integer first, then the quotient is truncated toward zero) |
| `MOD`    | Modulus (remainder after integer division) |
| `^`      | Exponentiation (right-associative) |

```
a% = 17
b% = 5
print a%; "+ "; b%; "="; a% + b%    // 22
print a%; "\ "; b%; "="; a% \ b%    // 3  (integer quotient)
print a%; "MOD "; b%; "="; a% mod b% // 2  (remainder)
print "2 ^ 8 ="; 2 ^ 8              // 256
print "2 ^ 3 ^ 2 ="; 2 ^ 3 ^ 2     // 512  (right-assoc: 2 ^ (3^2))
```

### Comparison Operators

| Operator | Meaning              |
|----------|----------------------|
| `=`      | Equal                |
| `<>`     | Not equal            |
| `<`      | Less than            |
| `<=`     | Less than or equal   |
| `>`      | Greater than         |
| `>=`     | Greater than or equal|

Comparison expressions evaluate to -1 (true) or 0 (false) at the BASIC
runtime, consistent with Microsoft BASIC semantics.

### TRUE and FALSE

`TRUE` and `FALSE` are transpile-time sugar for BASIC's own boolean
convention — `-1` and `0` — so a programmer-boolean flag can be compared
against a name instead of a magic number:

```
found% = TRUE
done%  = FALSE

if found% = TRUE then
    print "found it"
end if
```

They transpile straight through to the literals themselves — `found% = TRUE`
generates `found% = -1` — so they're valid anywhere an integer literal is,
including `CONST` and array bounds. No boolean type is introduced anywhere
else in the language; see the `NOT` caveat above for why explicit `= 0` /
`<> 0` comparisons are still how you test a flag.

### Compound Assignment

```
x% += n%    ' x% = x% + n%
x% -= n%    ' x% = x% - n%
x% *= n%    ' x% = x% * n%
x% /= n%    ' x% = x% / n%
```

Shorthand for reassigning a variable in terms of itself — the common case in
loop counters and accumulators. `total% += x%` is exactly equivalent to
`total% = total% + x%`; it works on array elements and record fields too:

```
scores%(i%) += 1
s.total# -= fee#
```

Only `+=`, `-=`, `*=`, `/=` are provided — there is no compound form of `\`,
`MOD`, `^`, or the bitwise/logical operators.

### Logical Operators

| Operator | Meaning |
|----------|---------|
| `AND`    | Bitwise AND (also serves as logical AND when operands are 0/-1) |
| `OR`     | Bitwise OR  |
| `NOT`    | Bitwise NOT |
| `XOR`    | Bitwise XOR |

**Important:** `NOT` is bitwise in Microsoft BASIC. `NOT 1` yields `-2`, not
`0`. BASCAL's transpiler emits `(expr) = 0` instead of `NOT expr` in generated
control-flow conditions so that programmer-boolean values like `found% = 1`
behave as expected. Use explicit `= 0` or `<> 0` comparisons in your own code
when testing boolean flags.

`AND`/`OR` always evaluate both sides — there's no short-circuit primitive in
generated BASIC at all. See [Short-Circuit `&&` and `||`](#short-circuit-and)
for BASCAL's condition-only short-circuit operators.

```
age%    = 25
income% = 45000
if age% >= 18 and income% >= 30000 then
    print "Eligible"
end if
print 6 xor 3   // 5  (110 XOR 011 = 101)
```

### Operator Precedence (highest first)

| Level | Operators        |
|-------|------------------|
| 9     | `^` (right-associative) |
| 8     | Unary `-`        |
| 7     | `*`, `/`         |
| 6     | `\`              |
| 5     | `MOD`            |
| 4     | `+`, `-`         |
| 3     | `=`, `<>`, `<`, `<=`, `>`, `>=` |
| 2     | `NOT`            |
| 1     | `AND`            |
| 0     | `OR`             |
| -1    | `XOR`            |

Use parentheses to override precedence.

---

## Comments

### Single-Line Comments

A single quote `'` or a double slash `//` begins a comment that extends to the
end of the line. Both forms are passed through to the generated BASIC output
as `'` comments.

```
' This is a single-line comment
// This is also a single-line comment
score% = 0  ' inline comment after a statement
score% = 0  // also valid inline
```

All three comment styles may appear inline after any statement.

### Block Comments

Block comments span multiple lines. The opening delimiter is `/*` and the
closing delimiter is `*/`. Block comments may appear anywhere a statement is
valid.

```
/*
 * Insertion sort — sorts arr%(0..sizeof(arr%)-1) in ascending order.
 * Time complexity: O(n^2) average and worst case.
 * Space complexity: O(1) — sorts in place.
 */
' arr% -- array to sort; byref because it's mutated in place
function insertionSort%(byref arr%(?))
    for i% = 1 to sizeof(arr%) - 1
        key% = arr%(i%)
        j%   = i% - 1
        while j% >= 0 and arr%(j%) > key%
            arr%(j% + 1) = arr%(j%)
            j% = j% - 1
        end while
        arr%(j% + 1) = key%
    end for
    return 0
end function
```

Each line of a block comment is emitted as a separate `'` comment in the
generated BASIC output. Leading `*` characters and surrounding whitespace are
stripped. Blank lines within the comment are preserved as blank lines in the
output.

One-line block comments are also valid:

```
/* Clear screen and draw title banner */
CLS
LOCATE 1, 30
PRINT "  BASCAL DEMO  "
```

---

## Control Flow

### IF / ELSEIF / ELSE / END IF

```
if condition then
    ' then body
end if

if condition then
    ' then body
else
    ' else body
end if
```

BASCAL also supports classic BASIC's single-line form: a statement
directly after `then`, on the same line, needs no `end if`.

```
if condition then statement
if condition then statement else statement
```

A newline right after `then` is what selects the block form above instead
— that's the only difference between the two (BASCAL is line-oriented with
no line-continuation syntax; see [File Encoding](#file-encoding)). The
single-line form may
chain multiple statements with `:`, same as anywhere else in BASCAL, and
its `else` (if any) must be on that same line too:

```
if x% > 0 then print "positive"
if x% > 100 then print "big" else print "small"
if x% > 0 then y% = 1: z% = 2
```

`elseif` isn't available in the single-line form — same as classic BASIC,
it needs the block form above.

From `tutorial/04_conditions.bcl` — a grade classification chain:

```
score% = 72
if score% >= 60 then
    PRINT "Pass (" + STR$(score%) + ")"
else
    PRINT "Fail (" + STR$(score%) + ")"
end if

points% = 85
if points% >= 90 then
    grade$ = "A"
elseif points% >= 80 then
    grade$ = "B"        ' points% = 85 lands here
elseif points% >= 70 then
    grade$ = "C"
elseif points% >= 60 then
    grade$ = "D"
else
    grade$ = "F"
end if
PRINT "Grade: " + grade$
```

`elseif` chains may be arbitrarily deep.

### FOR / END FOR

```
for var = start to end [step n]
    ' body
end for
```

`end for` closes the loop. Bare `end` also works. The `step` clause is
optional; the default step is 1.

From `tutorial/05_loops.bcl`:

```
' Squares 1..5
for i% = 1 to 5
    PRINT "  " + STR$(i%) + "^2 = " + STR$(i% * i%)
end for

' Countdown with negative step
for n% = 3 to 1 step -1
    PRINT "  " + STR$(n%)
end for
PRINT "  Go!"

' exit — stop at the first even number greater than 4
for i% = 1 to 20
    if i% > 4 and (i% / 2) * 2 = i% then
        PRINT "First even > 4: " + STR$(i%)
        exit
    end if
end for
```

`exit` exits the enclosing loop immediately. It's unqualified — not
`exit for`/`exit while`/`exit do` — the transpiler already knows which loop
it's inside; see [Exit](#exit) below.

### WHILE / END WHILE

```
while condition
    ' body
end while
```

`end while` closes the loop. Bare `end` also works, and so does classic
BASIC's own `wend`.

From `tutorial/05_loops.bcl`:

```
' Powers of 2 under 100
p% = 1
while p% < 100
    PRINT "  " + STR$(p%)
    p% = p% * 2
end while

' exit — stop after 8 Collatz steps
n% = 27
steps% = 0
while n% <> 1
    if steps% = 8 then
        PRINT "  ..."
        exit
    end if
    if (n% / 2) * 2 = n% then
        n% = n% / 2
    else
        n% = n% * 3 + 1
    end if
    steps% = steps% + 1
    PRINT "  " + STR$(n%)
end while
```

`exit` exits the enclosing `while` loop immediately; see [Exit](#exit) below.

### DO / END DO

```
do [while/until condition]
    ' body
end do
```

or the post-check form:

```
do
    ' body
loop [while/until condition]
```

`end do` (bare `end` also works) closes a **pre-check** loop: the optional
`while`/`until` clause tests the condition *before* each iteration, so the
body may run zero times. `loop [while/until condition]` closes a
**post-check** loop instead: the condition is tested *after* the body runs,
so the body always runs at least once — the direct BASCAL equivalent of
what other languages spell `repeat`/`until`. A bare `do ... loop` with no
condition on either end is a plain infinite loop, same as bare
`do ... end do`; both need `exit` to terminate.

From `tutorial/05_loops.bcl`:

```
' DO WHILE — condition tested before body
k% = 1
do while k% <= 3
    PRINT "  " + STR$(k%)
    k% = k% + 1
end do

' DO UNTIL — enters while condition is false
k% = 1
do until k% > 3
    PRINT "  " + STR$(k%)
    k% = k% + 1
end do

' DO ... LOOP UNTIL — post-check, body runs at least once
k% = 99
do
    PRINT "  " + STR$(k%)    ' prints 99 even though k% > 3
    k% = k% + 1
loop until k% > 3

' exit — leave from the middle of the body, either form
k% = 1
do
    if k% = 3 then
        exit
    end if
    PRINT "  " + STR$(k%)
    k% = k% + 1
end do
```

`exit` exits the enclosing `do` loop immediately, from either the
pre-check or post-check form; see [Exit](#exit) below.

### Exit

```
exit
```

`for`, `while`, and `do` share one early-exit statement: unqualified
`exit`, with no loop-type keyword after it. The transpiler resolves which
enclosing loop it leaves from context — the *innermost* one, if loops are
nested — so `exit` inside a `do` loop transpiles to a `GOTO` past the
loop's own end label, while `exit` inside a `for` loop transpiles to
BASIC's native `EXIT FOR` instead, since `for`/`next` transpiles to a real
`FOR ... NEXT` block rather than a `GOTO` chain (see
[For Transpilation](#for-transpilation)).

```
for i% = 1 to 5
    do
        if i% = 3 then
            exit          ' leaves the do, not the for
        end if
    end do
end for
```

`exit do`, `exit for`, and `exit while` are not valid — a loop-type
keyword after `exit` is a transpile-time error.

### SELECT CASE

```
select case expression
case value
    ' body
case value1, value2
    ' body for either value
case low to high
    ' body for values in range [low, high]
case is > threshold
    ' body when expression > threshold
case else
    ' default body
end select
```

The `select case` expression is evaluated once. Cases are tested in order.
`case else` is optional and must be the last clause.

From `tutorial/06_select_case.bcl`:

```
' Numeric score to letter grade
score% = 85
select case score%
case 100
    PRINT "Perfect!"
case 90 to 99
    PRINT "A  — Excellent"
case 80 to 89
    PRINT "B  — Good"      ' score% = 85 matches here
case 70 to 79
    PRINT "C  — Satisfactory"
case 60 to 69
    PRINT "D  — Passing"
case is >= 0
    PRINT "F  — Fail"
case else
    PRINT "Invalid score"
end select

' String select — weekend / weekday
day$ = "Saturday"
select case day$
case "Monday", "Tuesday", "Wednesday", "Thursday", "Friday"
    PRINT day$ + " is a weekday"
case "Saturday", "Sunday"
    PRINT day$ + " is a weekend"
case else
    PRINT "Unknown day: " + day$
end select

' IS comparisons
temp% = -3
select case temp%
case is < 0
    PRINT "Below freezing"
case is < 10
    PRINT "Cold"
case is < 20
    PRINT "Cool"
case else
    PRINT "Warm or hot"
end select
```

Supported `case` forms:

| Form | Matches when |
|------|-------------|
| `case value` | expression = value |
| `case v1, v2, v3` | expression = any listed value |
| `case low to high` | low ≤ expression ≤ high |
| `case is = value` | expression = value |
| `case is <> value` | expression ≠ value |
| `case is < value` | expression < value |
| `case is <= value` | expression ≤ value |
| `case is > value` | expression > value |
| `case is >= value` | expression ≥ value |

### Short-Circuit && and ||

```
if a% > 0 && b% > 0 then
    ' body only runs if BOTH are true -- b% > 0 is never evaluated
    ' unless a% > 0 already passed
end if

do until done% || attempts% >= max_attempts%
    ...
end do
```

Unlike `AND`/`OR` (bitwise, always evaluate both sides — see
[Logical Operators](#operators-and-expressions)), `&&` and `||` are true
short-circuit operators: `a% > 0 && f%()` never calls `f%()` when `a% > 0`
is already false, and `a% > 0 || f%()` never calls `f%()` when `a% > 0` is
already true.

`&&`/`||` are only legal directly in the condition of `if`/`elseif`/
`while`/`do [while/until]` — not as a general expression (can't be assigned
to a variable, passed as a function argument, etc.). A condition may chain
any number of the *same* operator (`a && b && c`); mixing `&&` and `||` in
one condition is a transpile-time error — split into nested `if` statements
instead.

From `tutorial/16_short_circuit.bcl`, an `&&` guard transpiles to one
guarded `IF` per operand — no bitwise `AND`, no wasted call:

```
if ptr% >= 0 && isPositive%(scores%(ptr%)) > 0 then
    print "safe to read"
end if
```
```
IF (ptr% >= 0) = 0 THEN GOTO 10
ispositiveN0% = scores%(ptr%)
GOSUB 20
IF (ispositiveResult0% > 0) = 0 THEN GOTO 10
    PRINT "safe to read"
10 REM END IF
```

`||` needs one extra label, since a *chain* has to keep checking until
either an operand proves it true or every operand has been tried:

```
if a% = 1 || a% = 2 then
    print "one or two"
end if
```
```
IF (a% = 1) <> 0 THEN GOTO 10
IF (a% = 2) <> 0 THEN GOTO 10
GOTO 20
10     PRINT "one or two"
20 REM END IF
```

`do until`/`do while`'s inverted polarity applies the same duality: a
`do until a% && b%` needs the extra label (mirroring a plain `||`), while
a `do until a% || b%` doesn't (mirroring a plain `&&`) — the transpiler
works this out per condition; it isn't something you need to reason about
yourself.

---

## Functions

### Declaration

```
function name%(param1%, param2%)
    ' body
    return expression
end function
```

The function name carries the return type suffix. Parameter names also carry
type suffixes. Functions may have zero or more parameters.

From `tutorial/07_functions.bcl`:

```
' a% -- first value to compare
' b% -- second value to compare
function max%(a%, b%)
    if a% > b% then
        return a%
    else
        return b%
    end if
end function

' a% -- first value to compare
' b% -- second value to compare
function min%(a%, b%)
    if a% < b% then
        return a%
    else
        return b%
    end if
end function

' value% -- number to constrain
' lo%    -- lower bound, inclusive
' hi%    -- upper bound, inclusive
function clamp%(value%, lo%, hi%)
    ' Constrain value to [lo, hi].
    return max%(lo%, min%(value%, hi%))
end function

' word$ -- string to title-case
function titleCase$(word$)
    ' Capitalise first letter, lowercase remainder.
    if LEN(word$) = 0 then
        return ""
    end if
    return UCASE$(LEFT$(word$, 1)) + LCASE$(MID$(word$, 2))
end function
```

### Calling Functions

```
PRINT "max(4, 9)      = " + STR$(max%(4, 9))         ' 9
PRINT "clamp(15,1,10) = " + STR$(clamp%(15, 1, 10))  ' 10
PRINT "clamp(-3,1,10) = " + STR$(clamp%(-3, 1, 10))  ' 1
PRINT titleCase$("bASCAL")                            ' Bascal
```

Functions called only for their side effects (discarding the return value)
are written as expression statements. The result variable is overwritten but
not read:

```
dummy% = sortArray%(data%)
```

### Return

Every function must contain at least one `return` statement. Implicit returns
at end-of-body are not supported.

### Calling the Same Function Twice

Each call writes the shared `fnameResult0` variable, so assignments must be
made before the next call overwrites it. BASCAL handles this automatically:

```
a$ = repeat$("x", 3)   ' repeatResult0$ = "xxx"  →  a$ = "xxx"
b$ = repeat$("y", 2)   ' repeatResult0$ = "yy"   →  b$ = "yy"
PRINT a$ + " " + b$    ' xxx yy
```

### Variable Scoping

Variables inside a function body are **local by default**: the transpiler maps
them to uniquely-generated BASIC names of the form `stemVar0%`, `stemVar1%`, etc.
Two functions can each have a variable named `i%` with no conflict, and a local
can never accidentally shadow a global that happens to share the naive prefix.
Use `global varname` to access a module-level variable:

```
' n% -- upper bound of the sum, inclusive
function sumTo%(n%)
    acc% = 0                ' local to sumTo%
    for i% = 1 to n%       ' local to sumTo%
        acc% = acc% + i%
    end for
    return acc%
end function

runningTotal% = 0

' x% -- amount to add to the running total
function addToTotal%(x%)
    global runningTotal%    ' refers to the module-level variable
    runningTotal% = runningTotal% + x%
    return runningTotal%
end function
```

`global` must name a real module-level variable, not one of the function's
own parameters — `function f%(x%) : global x% : ...` is a transpile-time
error, since the parameter always resolves first and the `global`
declaration could never take effect.

### Restrictions

- **No recursion, direct or indirect.** Functions and procedures are
  transpiled to `GOSUB` against shared global parameter storage, not a real
  call stack, so any call cycle — `f%` calling itself, or `f%` calling `g%`
  calling back into `f%`, however many hops apart — overwrites in-flight
  parameters. The transpiler checks the whole call graph and rejects **any**
  cycle at transpile time, not just direct self-calls. Use an explicit stack
  array to simulate recursion if needed.
- **No return value from a procedure.** Functions must `return` a value;
  for side-effect-only subroutines use `procedure` instead.
- **No `DEF FN`.** Classic MBASIC/GW-BASIC's single-line `DEF FN` (e.g.
  `DEF FN A(X) = X * X + 1`) is a deliberate scope decision to reject, not
  a missing feature — `function`/`procedure` blocks, with real parameters,
  `byref`/`byval`, and return values, fully supersede its clean form, and
  real-world `DEF FN` source sometimes abuses the comma operator or a
  colon-chained pseudo-statement list purely for evaluation-order side
  effects (`DEF FN X(A) = (A = A + 1, A)`), which has no clean general
  conversion into a `function` body. `bcc` recognizes the `DEF FN` grammar
  shape (including these abused forms) specifically so it can reject it
  with a clear, specific diagnostic rather than a generic parse error —
  but it is always rejected, never auto-converted. Rewrite each `DEF FN`
  by hand as a `function` before converting a file that uses it.

### How Functions Are Transpiled

Each function call transpiles to:
1. Assign each argument to a generated global variable (e.g. `fnameParam0%`)
2. `GOSUB` to the function's generated label
3. Assign the result from the generated result variable (e.g. `fnameResult0%`)

Local variables in the function body are emitted as uniquely-indexed BASIC
globals (`fnameVar0%`, `fnameVar1%`, …). The index is chosen so the name
does not clash with any global variable or with any name allocated by an
earlier function, making collisions impossible regardless of what names the
developer uses at global scope.

Every parameter is copied into its generated name before the call. Whether
anything is copied back afterward depends on its passing mode — see
[byref / byval](#byref-byval).

---

## Procedures

A procedure is a named subroutine that performs an action but returns no value.
It is declared with `procedure` … `end procedure`.

### Declaration

```
procedure name(param1%, param2$)
    ' body
end procedure
```

The procedure name has **no type suffix** — the absence of a suffix signals that
there is no return value. Parameter names still carry their usual type suffixes.

From `tutorial/14_procedures.bcl`:

```
procedure printSeparator()
    PRINT "----------------------------"
end procedure

' label$ -- text shown before the score
' score% -- value to print
procedure printScore(label$, score%)
    PRINT label$ + ": " + STR$(score%)
end procedure

' name$  -- person's name
' score% -- score to test against the passing threshold
procedure printIfPass(name$, score%)
    if score% < 60 then
        return          // early exit — nothing printed for failing scores
    end if
    PRINT name$ + " passed with " + STR$(score%)
end procedure

' arr%   -- array to fill; byref because it's mutated in place
' value% -- value written into every element
procedure fillRange(byref arr%(?), value%)
    for i% = 0 to sizeof(arr%) - 1
        arr%(i%) = value%
    end for
end procedure
```

### Calling Procedures

Procedures are called as statements (not inside expressions):

```
printSeparator()
printScore("Alice", 91)
printIfPass("Bob", 54)
fillRange(data%, 99)
```

### Early Exit

A bare `return` (no expression) exits a procedure immediately.
Falling through to `end procedure` is equally valid — the transpiler emits an
implicit `RETURN`.

```
' name$  -- person's name
' score% -- score to test against the passing threshold
procedure printIfPass(name$, score%)
    if score% < 60 then
        return      ' exit early; nothing is printed
    end if
    PRINT name$ + " passed with " + STR$(score%)
end procedure
```

### Array Parameters

Array parameters use the same [byref / byval](#byref-byval) rules and the
same `(?, ?, ...)` rank declaration as functions. Pass the plain array name
at the call site — no `()`:

```
' arr%   -- array to fill; byref because it's mutated in place
' value% -- value written into every element
procedure fillRange(byref arr%(?), value%)   ' arr%(?) -- 1-D array
    ...
end procedure

fillRange(data%, 99)                         ' plain data%, no ()
```

`fillRange` needs `byref` here because its entire job is to mutate the
caller's array — without it, `fillRange` would fill its own private copy
and the caller's array would be unchanged.

### Variable Scoping

Same rules as functions: variables in the body are local by default; use
`global varname` to access a module-level variable.

```
globalCount% = 0

procedure increment()
    global globalCount%
    globalCount% = globalCount% + 1
end procedure
```

### Restrictions

- **No recursion, direct or indirect.**  Same GOSUB transpilation as
  functions — the transpile-time cycle check covers procedures too, including
  a cycle that passes through both functions and procedures.
- **No return value.**  Do not use a procedure where an expression is expected.

### How Procedures Are Transpiled

Procedures use the same GOSUB mechanism as functions:

1. Assign each argument to a generated global variable (e.g. `pnameParam0%`)
2. `GOSUB` to the procedure's generated label
3. No result variable is read back

Local variables in the body are emitted as uniquely-indexed BASIC globals
(`pnameVar0%`, `pnameVar1%`, …) using the same collision-free scheme as
functions.

---

## Arrays

### Declaration

```
DIM values%(100)    ' 101 elements: values%(0) .. values%(100)
DIM names$(50)
```

Array indices run from 0 to *size* (i.e., *size*+1 elements in total, using
BASIC's default `OPTION BASE 0`).

### Access

```
values%(0) = 42
PRINT values%(i%)
```

### Passing Arrays to Functions

An array parameter must declare its rank right in the signature — one `?`
per dimension, in parens after the name: `arr%(?)` for 1-D, `grid%(?, ?)`
for 2-D, and so on. A scalar parameter stays a bare name. There's no way
to declare an array parameter without stating its rank this way — a
parameter that's indexed as an array in the function body but declared
without one is a **transpile-time error**, not a warning.

At the call site, just write the plain array name — **no `()` needed**.
The transpiler already knows that parameter is an array from its
declaration, so there's nothing left for the call site to mark.

And separately: `byref` does **not** give the function a real reference to
the caller's array — BASIC has no pointers or aliasing at this level.
`byref` copies the array's elements in before the call and copies them
back out after; `byval` (the default) only does the copy-in half. Either
way the function always works on its own private copy — `byref` just
*simulates* "the caller sees the result" by copying twice instead of once.
See [byref / byval](#byref-byval) for the full mechanism.

`insertionSort%` mutates the array in place, so its `arr%` parameter needs
`byref`; `indexOf%` only reads it, so the unmarked (`byval`) default is
correct as-is:

```
' arr% -- array to sort; byref because it's mutated in place
function insertionSort%(byref arr%(?))
    for i% = 1 to sizeof(arr%) - 1
        key% = arr%(i%)
        j%   = i% - 1
        while j% >= 0 and arr%(j%) > key%
            arr%(j% + 1) = arr%(j%)
            j% = j% - 1
        end while
        arr%(j% + 1) = key%
    end for
    return 0
end function

' arr%    -- array to search; byval, since indexOf% only reads it
' target% -- value to search for
function indexOf%(arr%(?), target%)
    for i% = 0 to sizeof(arr%) - 1
        if arr%(i%) = target% then
            return i%
        end if
    end for
    return -1
end function
```

From `tutorial/08_arrays.bcl`:

```
CONST N% = 6
DIM data%(N%)
data%(0) = 64 : data%(1) = 25 : data%(2) = 12
data%(3) = 22 : data%(4) =  3 : data%(5) = 11

dummy% = insertionSort%(data%)   ' sorts in place -- arr% is byref

idx% = indexOf%(data%, 22)
if idx% >= 0 then
    PRINT "22 found at index " + STR$(idx%)
end if
```

See [byref / byval](#byref-byval) for exactly what gets copied, and when.

### `byref` / `byval`

Every parameter — scalar or array — is copied into its generated storage
before the call. Whether that value is copied back to the caller afterward
depends on how the parameter is declared:

```
function insertionSort%(byref arr%(?))   ' byref: copied in, then back out
function indexOf%(arr%(?), target%)      ' unmarked = byval: copied in only
```

- **`byval`** (the default — an unmarked parameter is `byval`): the
  function gets its own private copy. Nothing is written back when the call
  returns, no matter what the function does to its copy internally.
- **`byref`**: copied in before the call, same as `byval` — but also copied
  back out to the caller after the call returns.

This applies uniformly to both parameter kinds:

- **Array parameters**: `byval` copies elements in; `byref` copies them in
  *and* back out. A function that only reads its array argument (like
  `indexOf%` above) should stay `byval` — a `byref` array with no writes is
  just a slower `byval`, since the transpiler still generates the copy-out
  loop.
- **Scalar parameters**: `byval` is the classic behavior scalar parameters
  have always had — a plain assignment in, nothing written back. `byref`
  turns a scalar parameter into a true output parameter:

  ```
  ' n% -- value to increment; byref so the caller sees the result
  procedure increment(byref n%)
      n% = n% + 1
  end procedure

  x% = 5
  increment(x%)   ' x% is now 6
  ```

  A `byref` argument must be a plain variable — `increment(x% + 1)` is a
  transpile-time error, because there's nowhere for the result to be written
  back to.

If you're coming from classic MBASIC/BASCOM: there's no local scope there
at all, so a `GOSUB`-based "subroutine" touching an array was always
touching the *one* array that exists — mutations were visible everywhere,
instantly, because there was never more than one copy. BASCAL's parameters
don't work that way by default. `byval` (the default) gives the function
its own copy, and `byref` is what asks for the old always-visible,
always-shared behavior back, deliberately, per parameter.

### Why Copy-In/Copy-Out, Not Just Globals?

MBASIC/BASCOM has exactly one subroutine primitive: `GOSUB`/`RETURN`. There
is no `SUB`, no `FUNCTION`, no parameter list of any kind — the most
disciplined code the raw dialect allows is a `GOSUB` target with a contract
enforced only by comments (*"expects `A$` and `B%` set, leaves the result in
`C%`"*). That convention-only discipline, and the maintenance burden of
getting it wrong across a team and a growing codebase, is exactly what
BASCAL exists to replace.

So the choice was never "copy-in/copy-out versus some simpler globals-based
way to pass parameters" — MBASIC/BASCOM has no other mechanism to build
parameters out of at all. `GOSUB` and global variables are the only two
primitives available. Giving `.bcl` functions real parameters means
simulating them using only those two primitives, and copy-in/copy-out is
what that simulation necessarily looks like: assign the argument into the
parameter's own storage before the `GOSUB`, and — for `byref` — copy the
result back out after.

There's also a structural reason it has to work this way, independent of
the target dialect. Each function body is transpiled exactly once, and every
call site `GOSUB`s to that same shared label. A shared body needs one
stable name for "its first parameter" — but different call sites pass
different things: different variable names, or a whole expression
(`f(a% + b%)`, `f(5)`) that isn't a variable at all. There's no single
caller-side location to just operate on directly in the general case, so
the value has to land somewhere fixed before the shared body runs.

`global` is the escape hatch for when you deliberately want the old
always-shared, no-copy behavior back for one specific variable — see
[Variable Scoping](#variable-scoping). It works precisely because it
commits to one hardcoded name forever, which is exactly what makes it not
a reusable, callable-with-different-data routine anymore.

### Multi-Dimensional Array Parameters

A 2-D (or higher) array parameter declares its rank the same way as 1-D —
one `?` per dimension — and passes the same way, too: just the plain array
name, no `()` and no count arguments of any kind. The transpiler already
knows the real array's bounds (from its `DIM`, or — if it's itself a
parameter being forwarded onward — from what *its* caller passed), and
carries them alongside the array automatically. Nothing about that is
visible in `.bcl` source; use [`sizeof()`](#sizeof) inside the function
body wherever the bound is needed.

```
' grid% -- 2-D array to sum
function sumGrid%(byref grid%(?, ?))
    total% = 0
    for r% = 0 to sizeof(grid%, 0) - 1
        for c% = 0 to sizeof(grid%, 1) - 1
            total% = total% + grid%(r%, c%)
        end for
    end for
    return total%
end function

dim g%(2, 2)
print sumGrid%(g%)
```

There's no way to pass the wrong bounds by hand here — unlike a manually
typed count argument, which could silently drift out of sync with the
array's real `DIM` (say, `3, 3` for an array actually `dim`ed `(2, 2)`,
reading one row and one column past the end of the real array at
runtime), the transpiler reads `grid%`'s bounds directly from `g%`'s own
`DIM` and there's no hand-typed number in the picture to get wrong.

`grid%(?, ?)` is cross-checked two ways, both at transpile time:

- Against the function's own body — `grid%(r%, c%)` above indexes with two
  subscripts, matching the declared two `?`s. A declaration that disagrees
  with how the body actually uses the parameter is an error.
- Against whatever array is actually passed at each call site — passing a
  1-D array where the parameter declares two dimensions (or vice versa) is
  also an error.

Either mismatch is caught before it ever reaches generated BASIC: the two
shapes genuinely can't share one copy loop, so BASCAL refuses rather than
emit a `DIM`/subscript mismatch that real BASIC would only catch at
runtime.

### Array Parameter Storage Capacity

An array parameter's storage is one shared, generated variable, reused by
every call to that function — the same reason a scalar parameter's
storage is shared. Arrays additionally need a fixed *size*, though, and
classic BASIC has no `REDIM`: once an array is `DIM`ed, it can never be
resized, and a second `DIM` on the same array is a fatal runtime error.
So a parameter's storage is `DIM`ed exactly once, at the very top of the
generated program — before any call happens — sized to the biggest array
anything anywhere ever passes it.

Normally this needs no attention at all. Write `?` for every axis, same
as always; the transpiler works out a safe capacity itself by scanning
every call site in the program and taking the largest resolved size.
Below, `sumArr%`'s storage ends up sized for 10 elements even though its
first call only ever passes it 3:

```
function sumArr%(arr%(?))
    ' ...
end function

dim small%(2)
dim big%(9)
dummy% = sumArr%(small%)
dummy% = sumArr%(big%)
```

This works whenever every call site's array size is knowable at transpile
time — a literal `DIM` bound, a `const`, or (when the array being passed
is itself another function's array parameter, forwarded onward) that
parameter's own already-resolved capacity. It genuinely can't work when a
call site's array size is a real runtime value:

```
input n%
dim data%(n%)
dummy% = sumArr%(data%)   ' error: arr%'s capacity can't be inferred
```

There's no way to know at transpile time how big `data%` will turn out to
be, so there's no safe number to give its shared storage automatically.
Write an explicit capacity instead of `?` for that axis — a literal
integer, chosen to comfortably cover every use:

```
function sumArr%(arr%(100))
    ' ...
end function
```

Whichever way a capacity is decided — inferred or explicit — every call
site still checks the array's *actual* size against it at runtime, right
before copying in, and halts with a clear error if it doesn't fit:

```
IF sumarrArrDim00% > 100 THEN PRINT "runtime error: ..." : STOP
```

This is a backstop, not the primary defense — a call site whose size
*is* a transpile-time constant and provably too big for its capacity is
already rejected at transpile time, before generated BASIC exists to run
at all. The runtime check exists for the one case that's genuinely
unprovable ahead of time: a capacity chosen to comfortably cover today's
inputs that a later, larger runtime value turns out to exceed.

### `sizeof()`

`sizeof(name)` returns a `dim`ed array's size, resolved entirely at
transpile time — it never appears in generated BASIC, only whatever value or
name it resolves to. For a 1-D array the axis is implicit:

```
dim data%(9)
print sizeof(data%)   ' 9 -- same value used in the dim
```

For 2-D or higher, the axis is required — `sizeof(name, axis)`, zero-based
in the same order as the array's own `DIM`:

```
dim grid%(2, 2)
print sizeof(grid%, 0)   ' 2 -- first DIM bound
print sizeof(grid%, 1)   ' 2 -- second DIM bound
```

The axis must be a literal integer — it selects which frozen value to
substitute at transpile time, not something computed at runtime.

**What "resolved at transpile time" means in practice:** if the bound is a
literal, `sizeof` just re-emits that literal. If it's an expression
(a variable, a `const`, anything not a bare number), the transpiler captures
its value into a hidden variable right at the `dim` site, and `sizeof`
always reads that captured value — never the live variable, which might
change afterward:

```
n% = 5
dim data%(n%)
n% = 99
print sizeof(data%)   ' 5 -- the value dim actually used, not the later 99
```

**Inside a function, `sizeof` on one of the function's own array
parameters works differently** — there's no local `dim` to freeze a value
from, since the array parameter's real size depends on whatever the
caller happens to pass. There's no manually declared count parameter to
read either: every array parameter's bounds are carried automatically,
one hidden transpiler-generated variable per axis, set by the caller (from
the real argument array's own resolved bounds) immediately before the
call. `sizeof(grid%, 0)` inside `sumGrid%`'s own body just reads that
hidden variable back:

```
function sumGrid%(byref grid%(?, ?))
    total% = 0
    for r% = 0 to sizeof(grid%, 0) - 1     ' reads the auto-passed row count
        for c% = 0 to sizeof(grid%, 1) - 1 ' reads the auto-passed column count
            total% = total% + grid%(r%, c%)
        end for
    end for
    return total%
end function
```

Nothing here is written by the `.bcl` author, at the call site or in the
signature — the bound simply isn't a value you pass, it's a value you ask
for with `sizeof()` wherever you need it.

`sizeof()` and [storage capacity](#array-parameter-storage-capacity) are
two different numbers that happen to often be equal. `sizeof(arr%)` is
always the *actual* array this particular call passed — it can be smaller
than capacity (e.g. `sumArr%` above sees `sizeof(arr%) = 3` on the call
that passes `small%`, even though its storage was sized for 10 to also
fit `big%`). Capacity is the fixed ceiling that storage was built for,
decided once, up front, for every call the program will ever make.

---

## Input and Output

### PRINT

Prints one or more expressions to the screen. Expressions are separated by
commas or concatenated with `+`.

```
PRINT "Hello, World!"
PRINT "Score: " + STR$(score%)
PRINT name$, score%
PRINT                              ' blank line
```

### LPRINT

Sends output to the printer (line printer). Same syntax as `PRINT`.

```
LPRINT "BASCAL screen demo printed at: " + DATE$
LPRINT "Score: " + STR$(score%)
```

### INPUT

Reads values from the keyboard.

```
INPUT name$
INPUT "Enter your name: "; name$
INPUT "Width, height: "; width%, height%
```

A prompt string followed by `;` suppresses the newline after the prompt (the
cursor remains on the same line). A prompt followed by `,` adds a `?` and
moves to the next print zone. The `;` form is recommended.

Multiple variables may be listed; the user enters values separated by commas.

### LOCATE

Positions the cursor before printing. From `tutorial/11_screen.bcl`:

```
CLS
COLOR 14, 1            ' bright yellow on blue
LOCATE 1, 30
PRINT "  BASCAL DEMO  "

COLOR 7, 0             ' restore white on black
LOCATE 3, 1
PRINT "Screen I/O tutorial"

LOCATE 5, 1 : COLOR 10 : PRINT "Green text"
LOCATE 6, 1 : COLOR 12 : PRINT "Red text"
LOCATE 7, 1 : COLOR  7 : PRINT "Normal text"
```

Rows and columns are 1-based on standard 80×25 displays.

### COLOR

Sets the foreground and optional background colour.

```
COLOR 14          ' bright yellow foreground, background unchanged
COLOR 15, 1       ' bright white on blue
COLOR 7, 0        ' grey on black (restore defaults)
```

Colour values follow CGA/EGA standard colour numbers (0–15 foreground,
0–7 background).

### BEEP

Sounds the system bell.

```
BEEP
```

### CLS

Clears the screen.

```
CLS
```

---

## File Input and Output

From `tutorial/10_files.bcl`:

### OPEN

Opens a file for reading, writing, or appending.

```
OPEN filename$ FOR INPUT  AS #1
OPEN filename$ FOR OUTPUT AS #2
OPEN filename$ FOR APPEND AS #3
```

The file number (`#1`, `#2`, etc.) is used in subsequent file I/O statements.

### CLOSE

Closes an open file.

```
CLOSE #1
```

### KILL

Deletes a file from disk.

```
kill "temp.dat"
kill tempFile$
```

Generates `KILL filename$`. The file must exist or a runtime error occurs.

### NAME ... AS

Renames (or moves) a file.

```
name "old.dat" as "new.dat"
name srcFile$ as destFile$
```

Generates `NAME old AS new`. Both arguments are expressions; string variables
or literals work equally well.

### WRITE # and INPUT #

`WRITE #` stores values in a quoted, comma-separated format that `INPUT #`
can read back reliably:

```
csvFile$ = "tutorial_scores.csv"

OPEN csvFile$ FOR OUTPUT AS #1
WRITE #1, "Alice", 95, "pass"
WRITE #1, "Bob",   54, "fail"
WRITE #1, "Carol", 78, "pass"
CLOSE #1

OPEN csvFile$ FOR APPEND AS #1
WRITE #1, "Dave", 88, "pass"
CLOSE #1

PRINT "Records in " + csvFile$ + ":"
OPEN csvFile$ FOR INPUT AS #1
while EOF(1) = 0
    INPUT #1, name$, score%, result$
    PRINT "  " + name$ + ": " + STR$(score%) + "  [" + result$ + "]"
end while
CLOSE #1
```

Output:
```
Records in tutorial_scores.csv:
  Alice: 95  [pass]
  Bob: 54  [fail]
  Carol: 78  [pass]
  Dave: 88  [pass]
```

### LINE INPUT #

Reads one complete line (including commas) from a file into a string variable:

```
OPEN csvFile$ FOR INPUT AS #1
while EOF(1) = 0
    LINE INPUT #1, line$
    PRINT "  " + line$
end while
CLOSE #1
```

### PRINT # (File Print)

Writes expressions to a file without the quoting that `WRITE #` adds:

```
PRINT #2, "Header line"
PRINT #2, count%, value!
```

### PRINT USING

Formats output with a template string before printing to the screen, printer,
or a file. The format string uses MS-BASIC format characters (`#` for digit
positions, `.` for the decimal point, `,` for thousands separator, `+`/`-` for
sign, etc.).

```
print using "####.##"; amount!          ' screen
lprint using "####.##"; amount!         ' printer
print #1, using "####.##"; amount!      ' file channel #1
```

Multiple values are separated by `;` or `,` exactly like a normal `PRINT`:

```
print using "Item ##: ####.##"; itemNo%, price!
```

The format string is any string expression; it does not have to be a literal:

```
fmt$ = "###.#"
print using fmt$; x!; y!; z!
```

---

## Random-Access File I/O

From Part 1 of `tutorial/15_random_and_record_files.bcl`:

Random-access files store fixed-length records that can be read or written in
any order, without scanning from the beginning.

BASCAL supports the classic statements below directly — `OPEN ... FOR
RANDOM`, `FIELD`, `LSET`/`RSET`, `PUT`/`GET`, `SEEK`, and the `MKx`/`CVx`
packing helpers all pass through as-is. But hand-summing field widths and
hand-matching pack/unpack calls is exactly the bookkeeping a transpiler should
do for you: see [Record Files](#record-files) below for BASCAL's typed
`record`/`file` syntax, the canonical way to do random-access I/O in BASCAL.
This section stays useful for reading the code that syntax generates, or for
files whose layout doesn't fit a fixed record type.

### OPEN FOR RANDOM

```
open filename$ for random as #1 len = recLen%
```

`len` sets the record size in bytes. Every record occupies exactly that many
bytes on disk. Records are numbered from 1.

### FIELD

Binds string variables to regions of the shared file buffer:

```
field #1, 2 as idBuf$, 20 as nameBuf$, 8 as scoreBuf$
```

The widths must sum to the record length. Only string variables may appear in
a `FIELD` statement.

### LSET and RSET

Copy data into a field-bound buffer variable, padded to the field width:

```
lset nameBuf$ = "Alice"    ' left-justified, padded with spaces on the right
rset idBuf$   = "42"       ' right-justified, padded with spaces on the left
```

### PUT and GET

Write or read a numbered record:

```
put #1, recordNum%    ' write current buffer as record recordNum%
get #1, recordNum%    ' load record recordNum% into buffer variables
```

Omitting the record number reads/writes at the current file position.

### SEEK

Move the file pointer to a given record position:

```
seek #1, recordNum%
```

### Packing Helpers

Numeric values must be packed into strings before storing in a `FIELD` buffer,
and unpacked after reading:

| Pack         | Unpack        | Type             |
|--------------|---------------|------------------|
| `mki%(n%)`   | `cvi%(s$)`    | 2-byte integer   |
| `mkl&(n&)`   | `cvl&(s$)`    | 4-byte long      |
| `mks!(n!)`   | `cvs!(s$)`    | 4-byte single    |
| `mkd#(n#)`   | `cvd#(s$)`    | 8-byte double    |

Example — writing and reading a numeric score:

```
const rec_len% = 30

open "students.dat" for random as #1 len = rec_len%
field #1, 2 as idBuf$, 20 as nameBuf$, 8 as scoreBuf$

lset idBuf$    = mki%(1)
lset nameBuf$  = "Alice"
lset scoreBuf$ = mkd#(95.0)
put #1, 1

get #1, 1
print rtrim$(nameBuf$) + ": " + str$(cvd#(scoreBuf$))
close #1
```

Output:
```
Alice: 95
```

---

## Record Files

From Part 2 of `tutorial/15_random_and_record_files.bcl`:

The `record` / `file` DSL is sugar over everything in
[Random-Access File I/O](#random-access-file-io) above. It computes the
record's byte width, allocates the file number, and generates the
`OPEN`/`FIELD`/`LSET`/`RSET`/`PUT`/`GET`/`MKx`/`CVx` calls for you — nothing
about the *generated* BASIC changes; only the BASCAL source you write does.

### record ... end record

Declares a fixed-layout record type:

```
record Student
    id:    int16
    name:  string(20)
    score: float64
end record
```

Supported field types and their packed width: `int16` (2 bytes), `int32`
(4 bytes), `float32` (4 bytes), `float64` (8 bytes), `string(N)` (N bytes).
The record's total width — used as the `OPEN ... LEN = ` value — is the sum
of its field widths, in declaration order.

### file ... as ... = open(...)

```
file db as Student = open("students.dat")
```

Transpiles to one `OPEN ... FOR RANDOM AS #n LEN = <width>` plus one matching
`FIELD #n, ...` statement, binding one string buffer variable per field.
File numbers are allocated automatically, starting at `#1`, in the order
`file` declarations appear in the source.

### Whole-record write

```
db[1] = { id: 1, name: "Alice", score: 95.0 }
```

Every declared field must be supplied exactly once. Transpiles to one `LSET`
per field — numeric fields are packed first (`MKI$`/`MKL$`/`MKS$`/`MKD$`),
string fields are assigned directly — followed by a single `PUT #n, 1`.
`LSET` is used for every field, numeric or string: once a numeric value is
packed, the result is exact-width binary, so left/right justification makes
no difference (this matches real BASCOM practice).

Note `MKx$` always carries a `$` suffix, never a type suffix matching the
value being packed (`MKI%`, `MKD#`, etc. are not real MBASIC/BASCOM
functions) — every `MKx$` variant returns a string, which is what `LSET`
requires.

A record literal missing a declared field is a **transpile-time error** — this
is a safety net that real BASIC's raw `FIELD`/`LSET`/`PUT` gives you no
equivalent of (see [Partial-record write](#partial-record-write) for the
deliberately-incomplete form).

### Partial-record write

```
db[2] = ?{ score: 61.5 }
```

`?{ ... }` is `{ ... }`'s deliberately-incomplete counterpart: any subset of
fields is allowed, and unlisted fields are left untouched on disk rather
than erroring. `?` doesn't collide with anything — it isn't tokenized at
all outside this position.

Whether the fields you *didn't* mention need preserving is fully decided at
transpile time, by comparing the field names you gave against the record's
declared fields — there's no runtime check:

- If the listed fields don't cover every declared field, an implicit
  `GET #n, i` is emitted first (so the unlisted fields keep their current
  on-disk values), then `LSET` for only the fields given, then `PUT #n, i`.
- If the listed fields happen to cover every declared field anyway, no
  `GET` is emitted — it transpiles exactly like a plain `{ ... }` literal.

Unlike `{ ... }`, an unknown field name inside `?{ ... }` is still a
transpile-time error — only *missing* fields are permitted, not *misspelled*
ones.

Note: `GET`ing a record number past the current end of a random-access file
doesn't error in real BASIC (records can be sparse), but the fields you
meant to "preserve" will simply read back as zero/blank the first time a
given record number is touched, since there was nothing on disk yet to
preserve.

### Whole-record read

```
let s = db[i]
```

Transpiles to `GET #n, i` followed by one unpacking assignment per field
(`CVI`/`CVL`/`CVS`/`CVD` for numeric fields, taking no suffix at all on real
MBASIC/BASCOM), each one written into a scalar named `<var><Field>` — e.g.
`sId%`, `sName$`, `sScore#`. Later references to `s.id`, `s.name`, `s.score`
in the source resolve directly to those scalars; no `Ident` named literally
`s.id` is ever emitted.

String fields aren't unpacked with `RTRIM$` — it isn't a real MBASIC/BASCOM
builtin. Instead, the transpiler builds an inline right-trim loop directly
from `LEN`/`MID$`/`LEFT$`, walking back from the end of the fixed-width
buffer past trailing spaces.

Because BASIC doesn't auto-convert numbers to strings for concatenation,
writing a numeric field next to a string with `+` (as in `print "[" + s.id + "]"`)
automatically wraps the numeric side in `STR$(...)` — but only where a record
field is actually involved; ordinary BASCAL `+` expressions are untouched.

Once `s` exists, `s.field = value` reassigns only the in-memory scalar
(`s_field`) — it does **not** touch the file. Assignment alone never causes
disk I/O; see [Writing a record variable back](#writing-a-record-variable-back)
for the explicit commit step.

### Partial-field update

```
db[i].field = value
```

For a single field, on its own, this is the terse form: it transpiles to an
implicit `GET #n, i`, a single `LSET` for just that field, then `PUT #n, i`.

This form does its own `GET`/`PUT` every time it appears, so chaining several
of them against the same record index costs one full round trip per field.
To change several fields on one record with a single `GET`/`PUT`, either use
[a partial-record write](#partial-record-write) (`db[i] = ?{ ... }`) with
several fields at once, or read it into a variable first and write it back
once — see below.

### Writing a record variable back

```
let s = db[i]
s.name  = "Alicia"
s.score = 99.0
db[i] = s
```

`db[i] = s` — where `s` was bound by an earlier `let s = db[...]` — packs
every field straight from `s`'s scalars and issues a single `PUT #n, i`,
regardless of how many of `s`'s fields were reassigned first. Combined with
the fact that `s.field = value` is pure in-memory assignment, this is the
one-`GET`-one-`PUT` way to change multiple fields: exactly one `GET` (from
the `let`) and one `PUT` (from the write-back) no matter how many fields
in between were changed. `s` must have been read from a `file` of the same
record type as the target; writing an `A` into a `file` of `B`s is a
transpile-time error.

### file.close()

```
db.close()
```

Transpiles to `CLOSE #n`.

### downto

```
for i = 3 downto 1
    ...
end for
```

Sugar for `for i = 3 to 1 step -1`; ascending `for i = A to B` is unchanged.

### Type checking

The transpilation pass rejects, at transpile time: field names not declared on the
record (in a record literal or a `.field` access), a record literal that is
missing a declared field or repeats one, a string literal that is wider
than its `string(N)` field, a string literal assigned to a numeric field (or
vice versa), an unknown record type named by `file ... as ...`, and any
reference to a `file` or `let`-bound record variable that was never
declared.

---

## Data Statements

`DATA`, `READ`, and `RESTORE` provide an embedded data table read at run time.
`DATA` statements may appear anywhere in the program body; the generated BASIC
places them after `END`.

From `tutorial/09_data.bcl`:

```
CONST NUM_CAPITALS% = 5

DIM country$(NUM_CAPITALS%)
DIM capital$(NUM_CAPITALS%)

for i% = 1 to NUM_CAPITALS%
    READ country$(i%), capital$(i%)
end for

PRINT "Country         Capital"
PRINT "--------------- ---------------"
for i% = 1 to NUM_CAPITALS%
    PRINT country$(i%) + "        " + capital$(i%)
end for

' RESTORE rewinds to the first DATA element
RESTORE
READ firstCountry$, firstCapital$
PRINT "First entry re-read: " + firstCountry$ + " -> " + firstCapital$

END

DATA "France",  "Paris"
DATA "Germany", "Berlin"
DATA "Japan",   "Tokyo"
DATA "Brazil",  "Brasilia"
DATA "Egypt",   "Cairo"
```

### RESTORE

Resets the `DATA` pointer to the beginning (or to a specific label).

```
RESTORE           ' rewind to the first DATA
RESTORE fromHere  ' rewind to the DATA right after the `fromHere:` label
```

---

## Miscellaneous Statements

### MID$ (statement form)

Overwrites a run of characters inside a string with a same-length
replacement — `target$` keeps its original length; see [MID$
assignment](#mid-assignment) for how this actually gets transpiled.

```
mid$(target$, start[, length]) = replacement$
```

`start` is 1-based. The optional `length` caps how many characters are
replaced; if omitted, replacement continues to the end of `target$` or until
`replacement$` runs out of characters, whichever comes first.

```
s$ = "Hello World"
mid$(s$, 7, 5) = "BASIC"   ' s$ → "Hello BASIC"
mid$(s$, 1)    = "Goodbye"  ' s$ → "GoodbyeBASIC" (no length cap)
```

This is distinct from the `mid$()` *function*, which extracts a substring
without modifying the original. BASCAL handles the statement form as an
ordinary assignment whose left-hand side is `mid$(...)`.

### SWAP

Exchanges the values of two variables — no explicit temporary needed.

From `tutorial/09_data.bcl`:

```
a% = 42
b% = 17
PRINT "Before SWAP: a=" + STR$(a%) + " b=" + STR$(b%)
SWAP a%, b%
PRINT "After SWAP:  a=" + STR$(a%) + " b=" + STR$(b%)
' Before SWAP: a=42 b=17
' After SWAP:  a=17 b=42
```

SWAP works on strings and array elements too:

```
SWAP first$, last$               ' exchange string variables
SWAP country$(i%), country$(j%)  ' exchange array elements (used in bubble sort)
```

### RANDOMIZE

Seeds the random number generator. With no argument, the runtime may prompt
for a seed or use a default.

```
RANDOMIZE           ' prompt or default
RANDOMIZE TIMER     ' time-based seed for different sequences each run
RANDOMIZE 99        ' fixed seed for reproducible output
```

### Labels

```
name:
```

Declares a branch target for `goto`/`gosub`/`on error goto`/`resume`/
`on ... goto`/`on ... gosub` to jump to. **BASCAL manages line numbers
itself — you cannot target a raw line number.** Every one of those
statements requires a label name; the transpiler assigns the actual BASIC
line number when it renders output, exactly the way it already numbers the
branch targets inside `if`/`while`/`do`/`select case`.

```
goto skip
print "not reached"
skip:
print "reached"
```

A label can share its line with the statement that follows it — the `:`
doubles as that statement's separator, same as anywhere else in BASCAL:

```
skip: print "reached"
```

### GOTO

Transfers control to a label. Prefer `if`, loops, and functions; `GOTO` is
primarily useful for error handlers.

```
GOTO doCleanup
```

### GOSUB / RETURN (BASIC-level)

Calls a BASIC subroutine at a label. Note this is the raw BASIC `GOSUB`,
distinct from the function-call mechanism BASCAL generates internally.

```
GOSUB writeLog
```

### ON ... GOTO / ON ... GOSUB

Computed branch: the integer expression selects the *n*th target (1-based).
Each target is a label, not a line number.

```
ON choice% GOTO firstCase, secondCase, thirdCase
ON mode%   GOSUB modeIdle, modeRun, modeError
```

If the expression evaluates to 0 or exceeds the number of targets, execution
continues with the next statement.

### ON ERROR GOTO / RESUME / ERROR

MS-BASIC structured error handling: trap runtime errors, handle them, and
resume execution.

#### ON ERROR GOTO

Installs an error handler at a given label. Any subsequent runtime error
causes execution to jump there. `ON ERROR GOTO 0` is the one place a numeric
argument is still legal — `0` isn't a line number, it's the sentinel that
disables the trap.

```
on error goto errHandler   ' jump to errHandler on any error
on error goto 0            ' disable the error trap
```

The target can be a raw `name:` label or a `procedure`. A `procedure` target
gets extra compile-time checking, because it's reached with a plain `GOTO`
(never a `GOSUB`), so there's no call frame for `RETURN` to pop: bcc rejects
any `return` inside such a procedure's body, and rejects the body unless
every path is proven to end in `resume`/`resume next`/`resume <label>` (an
`if`/`select case` only counts if every branch, including a mandatory
`else`/`case else`, diverges the same way). A procedure that passes both
checks also can't be called like an ordinary procedure anywhere else in the
program — it's proven to never return, so it could never come back to a
normal caller. See `errorTrap()` in `tutorial/inventory.bcl` for a worked
example.

#### RESUME

Resumes execution after an error handler has run.

```
resume             ' retry the statement that caused the error
resume next         ' continue at the statement after the failing one
resume afterError   ' jump to a specific label
```

`RESUME` without an argument retries the failing statement (useful for
recoverable errors like "disk full — retry after making space").
`RESUME NEXT` skips the failing statement.

#### ERROR

Triggers a runtime error with the given code, as if that error occurred
naturally. Useful for re-raising an error in a handler or testing.

```
error 53     ' simulate "file not found"
error code%  ' variable error code
```

#### ERR and ERL

Inside an error handler, the system pseudo-variables `err` and `erl`
hold the error code and the BASIC line number where the error occurred.
Write them without a type suffix; BASIC treats them as numeric:

```
on error goto handleErr
' ...
goto afterErr
handleErr:
' reached via ON ERROR GOTO
if err = 53 then
    print "File not found"
    resume next
end if
error err   ' re-raise unhandled errors
afterErr:
end
```

#### Typical pattern

```
on error goto errHandler
open fileName$ for input as #1
' ... file processing ...
close #1
on error goto 0
goto done

errHandler:
if err = 53 then
    print "Cannot open "; fileName$
    resume next
else
    error err
end if

done:
end
```

### POKE

Writes a byte value to a memory address. The address is an integer expression.

```
poke &H0400, 3      ' write to segment-zero address using hex literal
poke address%, val%
```

Generates `POKE address, value`. Use with care — writing to arbitrary addresses
is hardware-specific and may crash the runtime.

### OUT

Writes a byte to a hardware I/O port. Syntax mirrors `POKE`.

```
out 888, 3          ' send value 3 to port 888 (parallel port control)
out port%, val%
```

Generates `OUT port, value`. Port numbers and semantics are
hardware-specific.

### WIDTH

Sets the output line width for the screen or an open file channel.

```
width 80            ' set console line width to 80 characters
width 40            ' narrow console mode
width #1, 132       ' set line width for file channel #1
```

The optional `#n, ` prefix targets a file channel; without it, the console
width is set. Generates `WIDTH cols` or `WIDTH #n, cols`.

### CLEAR

Resets all program variables to their zero/empty defaults and closes all open
files. Useful at the start of a program or before reinitialising state.

```
clear
```

Generates `CLEAR`.

### DATE$, TIME$, TIMER

Read-only system pseudo-variables that return the current date, time, and
elapsed time. No parentheses — they are used as plain identifiers.

```
print "Today is "; date$         ' e.g.  06-11-2026
print "Time:    "; time$         ' e.g.  14:35:02
randomize timer                  ' time-based RNG seed
```

| Name     | Returns                                         |
|----------|-------------------------------------------------|
| `DATE$`  | Current date as `MM-DD-YYYY` string             |
| `TIME$`  | Current time as `HH:MM:SS` string               |
| `TIMER`  | Seconds since midnight (single-precision float) |

These are passed through verbatim as `DATE$`, `TIME$`, and `TIMER` in the
generated BASIC.

### STOP

Terminates the program immediately; may invoke the debugger in some
implementations.

```
STOP
```

### SYSTEM

Exits to the operating system immediately.

```
SYSTEM
```

### END

Signals the end of the main program body. Functions are emitted after `END`
in the generated output.

```
END
```

---

## Dependencies — REQUIRE and IMPORT

BASCAL supports multi-file projects through `require` (and its alias `import`).
Dependencies are declared at the top of the file, before any statements.

From `tutorial/12_require.bcl` — a program that uses a statistics library:

```
require stats

CONST N% = 8
DIM scores%(N%)

scores%(0) = 74 : scores%(1) = 91 : scores%(2) = 63 : scores%(3) = 88
scores%(4) = 55 : scores%(5) = 97 : scores%(6) = 72 : scores%(7) = 84

PRINT "Mean:   " + STR$(mean!(scores%()))
PRINT "Max:    " + STR$(maximum%(scores%()))
PRINT "Min:    " + STR$(minimum%(scores%()))
PRINT "Range:  " + STR$(rangeOf%(scores%()))
END
```

Transpile with `-L tutorial/lib` so that `require stats` resolves to
`tutorial/lib/stats.bcl`:

```
bcc tutorial/12_require.bcl -L tutorial/lib
```

`tutorial/lib/stats.bcl` defines `mean!`, `maximum%`, `minimum%`, and
`rangeOf%` — all merged into the single generated `.bas` output.

### Path Resolution

The dot-separated path is converted to a file path by replacing each `.` with
a directory separator and appending `.bcl`:

```
require com.bascal.sort.bubbleSort  →  com/bascal/sort/bubbleSort.bcl
require stats                       →  stats.bcl
```

The transpiler searches for the file in:
1. The directory containing the current source file
2. Additional directories supplied with `-L` flags (in order)

Dependencies are resolved recursively. A file is loaded at most once per
compilation (circular dependencies are silently ignored after the first load).

### Function Merging

All functions from a required file (and its transitive dependencies) are merged
into the generated output. Duplicate function names are rejected with a
diagnostic error.

### Module Conventions

Every file loaded via `require`/`import` **must** start with a
[`library <name>` declaration](#library-declaration) — a transpile-time
error, not just a convention. Beyond that required header, by convention a
library module should:
- Contain only `function` definitions and supporting `DIM` / `DATA` statements
- Not contain a `program` declaration
- Not contain top-level executable statements other than `DIM` and `DATA`

---

## Shared COMMON

In classic BASCOM programs, multiple programs chained together with `CHAIN`
share variables through `COMMON` declarations. For this to work correctly,
every program in the chain must declare **identical** `COMMON` lists — the
variable positions in the `COMMON` block must match exactly.

BASCAL coordinates `COMMON` through shared files. A shared file contains only
`dim` declarations (see below) — every variable in it is COMMON by default,
with no separate keyword needed to opt in — and programs that use it
reference it with a `shared` clause on their `program` declaration.

### Shared File

A shared file is a `.bcl` file containing only `dim` declarations (see
[DIM Declaration](#dim-declaration) below), plus blank lines and comments.

It starts with a mandatory `shared <name>` header, analogous to a regular
file's `program <name>` header, and declares its shared variables with
ordinary `dim`:

From `tutorial/13_shared/state.bcl`:

```
/*
 * Shared file for Tutorial 13 — COMMON / CHAIN.
 *
 * Every program that begins with "program name shared state" receives
 * an identical COMMON block at the top of its generated BASIC, so the
 * listed variables survive a CHAIN to the next program.
 */
shared state

dim count%
dim label$
```

Rules for shared files:
- The `shared <name>` header is mandatory, and its name must match the
  filename the transpiler resolved it as (`state.bcl` → `shared state`).
- Only `dim` declarations, blank lines, and comments are allowed.
- `require`, `function`, executable statements, and `program`/`library`
  declarations are all rejected with a diagnostic error.
- The shared file must contain at least one `dim` declaration.
- A file may declare at most one of `program`, `library`, or `shared` — a
  shared file can't also be an ordinary program or library module.

### DIM Declaration

```
shared state

dim count%
dim label$
dim scores%()
```

Inside a `shared <name>`-headed file, every top-level `dim` becomes one
shared (COMMON) variable, in declaration order — exactly the [DIM](#dim)
statement used anywhere else in BASCAL, including its multi-name comma form
(`dim count%, label$`) and array declarations (`dim scores%()`, empty-parens,
same as a `COMMON` array). No bounds are stored either way — a shared file's
`dim` only ever declares *that* a name is an array, not its size.

### Program Declaration with Shared File

```
program start shared state
```

When a shared-file name is present, the transpiler:
1. Searches for `state.bcl` in the source file's directory (then `-L` paths).
2. Validates that the shared file contains only `dim` declarations.
3. Emits the `COMMON` lines at the very top of the generated `.bas` file,
   before any other output.

### Using a Shared File

From `tutorial/13_shared/` — two programs that share `count%` and `label$`:

**`state.bcl`** (shared file):
```
shared state

dim count%
dim label$
```

**`start.bcl`** (program 1):
```
program start shared state

label$ = "Counter demo"
count% = 0
count% = count% + 1
count% = count% + 1
count% = count% + 1

PRINT "Initialised: " + label$
PRINT "Count after 3 increments: " + STR$(count%)

/* CHAIN "show.bas" */
END
```

**`show.bcl`** (program 2):
```
program show shared state

PRINT "Label:  " + label$
PRINT "Count:  " + STR$(count%)
END
```

Both `start.bas` and `show.bas` begin with:

```
COMMON count%, label$
```

ensuring that `CHAIN "show.bas"` from `start.bas` leaves the variables in the
correct slots.

### Restrictions

- A `shared <name>` header is illegal everywhere except in a shared file
  being loaded as one — a stray `shared` header in an ordinary program or
  library module is a transpile error. A shared file without one is also an
  error — the header is mandatory.
- A `program` declaration is illegal in library modules (files loaded via
  `require`), and mandatory in the root file `bcc` was invoked on.
- A `library` declaration is illegal in the root file `bcc` was invoked on,
  and mandatory in every file loaded via `require`/`import`.
- A file may declare at most one of `program`, `library`, or `shared`.
- If the named shared file does not exist, the program transpiles without a
  `COMMON` block (no error). This allows incremental development.

---

## Generated BASIC Shape

Understanding how BASCAL transpiles its constructs helps when reading generated
output or debugging.

### Header

Every generated file begins with:
```
' BASCAL generated BASIC
' Functions are transpiled to global variables, labels, and GOSUB
```

### COMMON Block

If a shared file is referenced, `COMMON` lines appear before the header comment.

### Line Numbers

By default, `bcc` numbers every emitted line, not just branch targets. Real
MBASIC/BASCOM has no notion of an unnumbered statement line -- classic BASIC
source is a sequence of numbered lines, full stop -- so this is what real
compilers and interpreters expect. Numbered comment-only lines are harmless
on real BASCOM, but an unnumbered *statement* line is a syntax error.

Pass `--sparse-line-numbers` to fall back to the old behavior, numbering
only lines that are branch targets (destinations of `GOTO` or `GOSUB`) and
leaving everything else unnumbered. This is more readable, but only safe
with more lenient dialects (e.g. FreeBASIC's `-lang qb`) -- not real
MBASIC/BASCOM.

### If Transpilation

```
if x% > 0 then
    PRINT "positive"
end if
```

Becomes:

```
IF (x% > 0) = 0 THEN GOTO 10
    PRINT "positive"
10 REM END IF
```

The condition is inverted with `= 0` rather than `NOT` to avoid bitwise
semantics (see [Operators](#operators-and-expressions)).

### While Transpilation

```
p% = 1
while p% < 100
    PRINT STR$(p%)
    p% = p% * 2
end while
```

Becomes:

```
p% = 1
10 IF (p% < 100) = 0 THEN GOTO 20
    PRINT STR$(p%)
    p% = p% * 2
    GOTO 10
20 REM END WHILE
```

### Do Transpilation

```
do while k% <= 3
    PRINT STR$(k%)
    k% = k% + 1
end do
```

Becomes:

```
10 IF (k% <= 3) = 0 THEN GOTO 20
    PRINT STR$(k%)
    k% = k% + 1
    GOTO 10
20 REM END DO
```

The post-check form skips the leading guard entirely, since the body always
runs at least once:

```
do
    PRINT STR$(k%)
    k% = k% + 1
loop until k% > 3
```

Becomes:

```
10 PRINT STR$(k%)
    k% = k% + 1
    IF (k% > 3) = 0 THEN GOTO 10
20 REM END DO
```

### For Transpilation

BASCAL emits native `FOR` / `NEXT`, which BASIC runtimes handle efficiently.
The BASCAL `end for` (or bare `end`) is stripped; the BASIC `NEXT` is emitted
by the transpiler:

```
FOR i% = 1 TO 5
    PRINT STR$(i%) + "^2 = " + STR$(i% * i%)
NEXT i%
```

### Function Transpilation

```
' value% -- number to constrain
' lo%    -- lower bound, inclusive
' hi%    -- upper bound, inclusive
function clamp%(value%, lo%, hi%)
    return max%(lo%, min%(value%, hi%))
end function

result% = clamp%(15, 1, 10)
```

The calls to `max%` and `min%` inside `clamp%` are also transpiled to GOSUBs.
The outermost call produces:

```
clamp_value% = 15
clamp_lo%    = 1
clamp_hi%    = 10
GOSUB 100
result% = clamp_result%
...
END

' function clamp%(value%, lo%, hi%)
100 ' (transpiled body — calls max% and min% via GOSUB)
    clamp_result% = ...
    RETURN
' end function clamp%
```

### Procedure Transpilation

Procedures follow the same GOSUB pattern as functions but have no result
variable:

```
' label$ -- text shown before the score
' score% -- value to print
procedure printScore(label$, score%)
    PRINT label$ + ": " + STR$(score%)
end procedure

printScore("Alice", 91)
```

Transpiles to:

```
printscore_label$ = "Alice"
printscore_score% = 91
GOSUB 200
...
END

' procedure printScore(label$, score%)
200 PRINT (printscore_label$ + ": ") + STR$(printscore_score%)
    RETURN
' end procedure printScore
```

There is no `printscore_result` variable. A bare `return` inside a procedure
transpiles to plain `RETURN`.

### Select Case Transpilation

`SELECT CASE` is transpiled to an `IF`/`GOTO` dispatch chain. The select
expression is stored in a temporary variable (e.g., `BCCT1%`) to avoid
re-evaluation.

### Exit Statements

`exit` is unqualified in BASCAL source; the transpiler picks the shape below
based on which loop it's innermost inside:

- inside `for` → `EXIT FOR` (native FreeBASIC / QB extension)
- inside `while` → `GOTO end_label`
- inside `do` → `GOTO end_label`

---

## Command-Line Reference

```
bcc input.bcl [-o output.bas] [-L dir] [-l library]
              [--line-numbers] [--clean | -c] [--binary | -b]
              [--target | -t basic|c]
```

| Flag | Short | Description |
|------|-------|-------------|
| `-o output.bas` | | Output file path. Default: source path with `.bas` (or `.c`, under `--target c`) extension in the same directory. |
| `-L dir` | | Add a directory to the library search path. Repeatable. |
| `-l name` | | Name a library (reserved). |
| `--line-numbers` | | Number every output line, not just branch targets. |
| `--clean` | `-c` | Re-transpile even if the output is already up to date. |
| `--binary` | `-b` | Compile the generated output to a binary in `tmp/`: `fbc` for `--target basic`, `gcc` for `--target c`. |
| `--target <t>` | `-t` | Backend to generate code for: `basic` (default, the only complete one) or `c` (just getting started — see below). |

### Backends

`--target basic` (the default) is everything this manual otherwise
describes: plain 1980s Microsoft BASIC/BASCOM output.

`--target c` is a new, deliberately minimal native-C backend, aiming to
produce native Linux/macOS/Win32 binaries directly (via `gcc`) without
going through a BASIC compiler at all — while the BASCOM-compatible
`basic` target keeps gating what language features BASCAL adds, so both
backends stay able to express the same language. Four tutorials compile
end to end today: [`tutorial/01_hello.bcl`](tutorial/01_hello.bcl),
[`tutorial/03_arithmetic.bcl`](tutorial/03_arithmetic.bcl),
[`tutorial/04_conditions.bcl`](tutorial/04_conditions.bcl), and
[`tutorial/05_loops.bcl`](tutorial/05_loops.bcl) — see each one's `.c`
counterpart for its output.

Currently supported: `print`, `end`, `dim`, `const`, `if`/`elseif`/
`else`/`end if` (block, single-line, and nested — C has native
`if`/`else`, so this is a direct structural translation, unlike the
`basic` target, which has to transpile to a GOTO/label chain since real
MBASIC/BASCOM has no block `IF`), `for`/`next`, `while`/`wend`, every
`do`/`loop` pre-/post-check variant, and `exit` (a plain C `break;` --
C's native loops already give it the right "innermost enclosing loop"
target for free, unlike the BASIC backend, which needs its own
loop_exit_stack since real MBASIC/BASCOM's loops are GOTO chains with no
native break), scalar variables — both numeric (`%`/`&`/`!`/`#`) and
string (`$`) — matching BASIC's spring-into-existence-zero-initialized
semantics (every variable touched anywhere is declared once at the top
of `main`), every arithmetic operator (`+ - * / \ MOD ^`), every
comparison operator (`= <> < <= > >=`, evaluating to BASIC's own
`-1`/`0`, not C's `1`/`0`), every bitwise/logical operator (`AND OR XOR
NOT` — genuinely bitwise, not short-circuit booleans: C's `&`/`|`/`^`/`~`
are correct here, not `&&`/`||`/`!`, since real MBASIC/BASCOM has no
short-circuit boolean primitive at all), and string concatenation (`+`).
Anything else (arrays, functions, calls...) reports a "not supported
yet" diagnostic instead of emitting incorrect code.

A narrowing numeric assignment (a float/double-valued expression
assigned into an integer-suffixed variable, e.g. `n% = n% / 2`) rounds,
matching real MBASIC/BASCOM's own `CINT()`-style conversion (confirmed
directly against real BASCOM: `N% = 27 / 2` gives `14`, not `13`) -- not
C's own implicit truncating conversion, which would silently give a
different, wrong answer. A `for` loop's start/end/step are each
captured into their own temp exactly once, at loop entry, matching
BASIC's own "evaluated once, not re-read every iteration" semantics -- a
naive C `for` whose condition directly re-reads a variable the body
mutates would behave differently.

Every operator needed its exact BASIC semantics tracked down first, not
assumed to be "the same as the C operator": `/` gets explicit `(double)`
casts so it stays true division even between two integers (plain C
`int / int` truncates); `\`/`MOD`/`AND`/`OR`/`XOR`/`NOT` round each
operand first via `round()` (verified against the GW-BASIC Reference
Manual, and `round()`'s ties-away-from-zero tie-break confirmed directly
against a genuine, period-accurate IBM Personal Computer BASIC Compiler
2.00 under dosbox-x -- `2.5 \ 1 = 3`, `2.5 AND 3 = 3`, matching `round()`
and disagreeing with e.g. round-half-to-even or plain truncation; see
`scripts/fetch-ibm-basic-compiler.sh` /
[test-fixtures/README.md](test-fixtures/README.md) if you want to check
this or other real-BASCOM claims yourself, the same fixture the `basic`
target's own dosbox-x conformance suite uses), then apply C's native
`/`, `%`, `&`, `|`, `^`, or `~`; `^` (exponent) maps to `pow()` from
`<math.h>`. String variables are
fixed-size buffers (`char[256]`) — real BASIC strings are dynamically
sized, which this backend doesn't attempt — written exclusively via
`snprintf` (safely truncates an over-long value, never overflows), never
`strcpy`/`strcat`. `%`/`&` (BASIC's 16-bit integer and 32-bit long) are
collapsed to the same plain C `int`.

### Up-to-Date Check

Without `--clean`, `bcc` skips re-transpiling if the output `.bas` file
is newer than all input `.bcl` files. With `--binary`, a second up-to-date
check covers the compiled binary.

### Library Search Order

1. The directory containing the primary source file (always first).
2. Paths supplied with `-L`, in the order given.

Multiple `-L` flags are supported:

```
bcc tutorial/12_require.bcl -L tutorial/lib
bcc main.bcl -L libs/sort -L libs/string
```

---

## Statement Quick Reference

| Statement | Syntax | Description |
|-----------|--------|-------------|
| `BEEP` | `BEEP` | Sound the system bell |
| `CLEAR` | `CLEAR` | Reset all variables and close all files |
| `CLS` | `CLS` | Clear the screen |
| `CLOSE` | `CLOSE #n` | Close file channel *n* |
| `COLOR` | `COLOR fg[, bg]` | Set foreground/background colour |
| `CONST` | `CONST name = expr` | Declare a named constant |
| `DATA` | `DATA val[, ...]` | Embed literal data values |
| `DIM` | `DIM name[(d1[, d2, ...])][, name2...]` | Declare one or more variables or 1-D/multi-D arrays |
| `ERASE` | `ERASE arr[, ...]` | Free memory used by arrays |
| `DO` | `DO [WHILE/UNTIL cond]` … `END DO`, or `DO` … `LOOP [WHILE/UNTIL cond]` | Pre-check or post-check conditional loop |
| `END` | `END` | End of program |
| `EXIT` | `exit` | Exit the innermost enclosing FOR/WHILE/DO loop |
| `FOR` | `FOR v = start TO end [STEP s]` … `END FOR` | Counted loop |
| `FUNCTION` | `FUNCTION name%(params)` … `END FUNCTION` | Define a function with a return value |
| Label | `name:` | Declare a branch target for GOTO/GOSUB/ON.../RESUME |
| `GOSUB` | `GOSUB label` | Call BASIC subroutine |
| `GOTO` | `GOTO label` | Unconditional branch |
| `IF` | `IF cond THEN` … [`ELSEIF` …] [`ELSE` …] `END IF` | Conditional block |
| `INPUT` | `INPUT [prompt;] var[, ...]` | Read from keyboard |
| `KILL` | `KILL file$` | Delete a file |
| `INPUT #` | `INPUT #n, var[, ...]` | Read from file |
| `LET` | `LET var = expr` | Assignment (keyword optional) |
| Compound assign | `var += / -= / *= / /= expr` | Assignment shorthand for `var = var op expr` |
| `MID$` (stmt) | `MID$(str$, start[, len]) = repl$` | In-place substring replacement |
| `LINE INPUT` | `LINE INPUT #n, var$` | Read full line from file |
| `LOCATE` | `LOCATE row, col` | Position cursor |
| `LPRINT` | `LPRINT expr[, ...]` | Print to printer |
| `NAME` | `NAME old$ AS new$` | Rename a file |
| `OPTION BASE` | `OPTION BASE 0\|1` | Set default array lower bound |
| `ON...GOTO` | `ON expr GOTO label1, label2, ...` | Computed GOTO |
| `ON...GOSUB` | `ON expr GOSUB label1, label2, ...` | Computed GOSUB |
| `ON ERROR GOTO` | `ON ERROR GOTO label` | Install error handler (`ON ERROR GOTO 0` disables) |
| `ERROR` | `ERROR n` | Trigger runtime error code *n* |
| `RESUME` | `RESUME` / `RESUME NEXT` / `RESUME label` | Resume after error handler |
| `OPEN` | `OPEN file$ FOR INPUT/OUTPUT/APPEND AS #n` | Open file |
| `OUT` | `OUT port, val` | Write byte to hardware I/O port |
| `POKE` | `POKE address, val` | Write byte to memory address |
| `PRINT` | `PRINT expr[, ...]` | Print to screen |
| `PROGRAM` | `program name` / `program name shared sharedname` | Declare this file as a runnable program (mandatory in the file passed to `bcc`) |
| `LIBRARY` | `library name` | Declare this file as a library module (mandatory in every `require`/`import` target) |
| `PROCEDURE` | `PROCEDURE name(params)` … `END PROCEDURE` | Define a procedure (no return value) |
| `PRINT #` | `PRINT #n, expr[, ...]` | Print to file |
| `PRINT USING` | `PRINT USING fmt$; expr[; ...]` | Formatted print (also `LPRINT USING`, `PRINT #n, USING`) |
| `RANDOMIZE` | `RANDOMIZE [seed]` | Seed random number generator |
| `READ` | `READ var[, ...]` | Read from DATA stream |
| `REQUIRE` | `require path.symbol` | Load dependency module |
| `RESTORE` | `RESTORE [label]` | Reset DATA pointer |
| `RETURN` | `RETURN expr` / `RETURN` | Return value from function; bare form exits a procedure early |
| `SELECT CASE` | `SELECT CASE expr` … `END SELECT` | Multi-way branch |
| `STOP` | `STOP` | Stop program execution |
| `SHARED` | `shared name` | Declare this file as the shared-variables file `name` (shared vars listed via `dim`) |
| `SWAP` | `SWAP a, b` | Exchange two variable values |
| `SYSTEM` | `SYSTEM` | Exit to operating system |
| `WHILE` | `WHILE cond` … `END WHILE` (or `WEND`) | Condition-at-top loop |
| `WIDTH` | `WIDTH [#n,] cols` | Set line width for console or file channel |
| `WRITE #` | `WRITE #n, expr[, ...]` | Write to file (quoted format) |

### System pseudo-variables (no parentheses)

| Name | Type | Returns |
|------|------|---------|
| `DATE$` | String | Current date as `MM-DD-YYYY` |
| `TIME$` | String | Current time as `HH:MM:SS` |
| `TIMER` | Single | Seconds elapsed since midnight |

### Boolean literals

| Name | Transpiles to |
|------|-------------|
| `TRUE` | `-1` |
| `FALSE` | `0` |

---

## Standard Library Functions

### MID$ assignment

```
MID$(target$, start[, len]) = replacement$
```

A same-length splice into `target$`, which keeps its original length —
not a value-producing expression, and not the same thing as `MID$(...)`
used to *read* a substring. `target$` must be a plain string variable or
string array element (not, for example, a record/file DSL field or a
nested call).

Despite compiling cleanly, this statement isn't reliable across every real
MBASIC/BASCOM dialect BASCAL targets, so it's transpiled into a call to
`com.bascal.stdlib.midAssign` — an ordinary BASCAL function, auto-added to
the program (like any other `com.bascal.stdlib` symbol; see [String and
error-message functions](#string-and-error-message-functions) below) the
moment `MID$` assignment syntax appears anywhere, with no `require` line
needed since nothing in your own source ever spells the function's name:

```
function midAssign$(target$, start%, len%, value$)
    t$ = value$
    if LEN(t$) > len% then
        t$ = LEFT$(t$, len%)
    end if
    return LEFT$(target$, start% - 1) + t$ + MID$(target$, start% + LEN(t$))
end function
```

Every call site becomes an ordinary function call (`GOSUB`, in the
generated BASIC) into that one shared body — the same call/return machinery
every other BASCAL function goes through, so there's no separate
inline-vs-shared-subroutine cutoff to reason about.

The two-argument form (`MID$(target$, start) = replacement$`) behaves as if
`len` were `LEN(replacement$)`. Total `LEN(target$)` never changes — this is
always a same-length overwrite, never a grow/shrink — and if `replacement$`
is shorter than `len`, only that many characters are overwritten; the rest
of `target$` past that point is left untouched, not padded.

### String and error-message functions

`LTRIM$`, `RTRIM$`, `UCASE$`, and `LCASE$` are not real MBASIC/BASCOM 2.00
builtins, and `ERROR$` compiles and links but silently returns an empty
string at runtime instead of a real message (all verified against a real
IBM Personal Computer BASIC Compiler 2.00 running under dosbox-x). BASCAL
ships its own implementations, built from genuinely portable primitives
(`LEFT$`/`MID$`/`LEN`/`ASC`/`CHR$`, loops — no `PEEK`/`POKE`, no `VARPTR`),
as an ordinary `require`-able library under `com.bascal.stdlib` — the same
mechanism as any other BASCAL library (see [Dependencies — REQUIRE and
IMPORT](#dependencies-require-and-import)), not something auto-injected
by call-site detection:

```
require com.bascal.stdlib.ltrim
require com.bascal.stdlib.rtrim
require com.bascal.stdlib.ucase
require com.bascal.stdlib.lcase
require com.bascal.stdlib.error
```

| Symbol | Signature | Behavior |
|----------|-----------|----------|
| `com.bascal.stdlib.ltrim` | `LTRIM$(s$)` | Strip leading spaces |
| `com.bascal.stdlib.rtrim` | `RTRIM$(s$)` | Strip trailing spaces |
| `com.bascal.stdlib.ucase` | `UCASE$(s$)` | Uppercase `a`-`z` only; other characters pass through unchanged |
| `com.bascal.stdlib.lcase` | `LCASE$(s$)` | Lowercase `A`-`Z` only; other characters pass through unchanged |
| `com.bascal.stdlib.error` | `ERROR$(code%)` | Human-readable message for a classic MBASIC/GW-BASIC/BASCOM error code (e.g. `ERROR$(53)` → `"File not found"`); falls back to `"Error " + STR$(code%)` for a code outside its lookup table |

Each `.bcl` source file lives under `com/bascal/stdlib/` in the BASCAL
distribution, and `bcc` always adds that directory to its library search
path automatically — a release package ships it next to the `bcc` binary
(or, for a `.deb`/`.rpm` install, under `.../share/bascal/`), so no `-L` is
needed to reach it. `-L` and a same-named file next to your own source both
still take priority, so you can shadow a stdlib module with your own if you
ever need to.

Requiring one of these and also defining a function under the same name is
a duplicate-function error, same as any other name collision between a
required library and your own code — pick one.

`STRING$`, `FIX`, `HEX$`, and `OCT$` were checked the same way against real
BASCOM 2.00 and *are* genuine builtins, so BASCAL passes calls to them
straight through rather than reimplementing them.
