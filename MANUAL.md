# BASCAL Language Reference Manual

**BASCAL Compiler (bcc) — Version 0.1**

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
18. [Dependencies — REQUIRE and IMPORT](#dependencies--require-and-import)
19. [Suite COMMON](#suite-common)
20. [Generated BASIC Shape](#generated-basic-shape)
21. [Command-Line Reference](#command-line-reference)
22. [Statement Quick Reference](#statement-quick-reference)

---

## Introduction

**BASCAL** — **B**eginner's **A**ll-purpose **S**tructured **C**omputer
**A**pplication **L**anguage — is a compiler that translates structured
`.bcl` source files into line-numbered Microsoft BASIC programs (`.bas`)
compatible with BASCOM and FreeBASIC's QB compatibility mode.

BASCAL adds structured programming constructs on top of BASIC's run-time
semantics:

- Block `if` / `elseif` / `else` / `end if`
- `for` / `end for`, `while` / `end while`, and `do` / `end do` loops with early exit
- `function` declarations with typed return values and explicit `return`
- `procedure` declarations for action subroutines with no return value
- Path-style `require` for multi-file projects
- `program` / `suite` declarations for coordinating `COMMON` across chained
  programs
- Multi-line `/* */` block comments and `//` end-of-line comments in addition
  to the classic `'` comment
- `select case` with range and `is` comparisons
- All classic BASCOM 1980s statements: `DATA`/`READ`/`RESTORE`, `LOCATE`,
  `COLOR`, `ON ... GOTO`, `SWAP`, `RANDOMIZE`, `CONST`, and more

**BASCAL does not invent a new runtime.** Every BASCAL program compiles to
plain Microsoft BASIC. The structured constructs are transpiled by the compiler:
functions become `GOSUB` subroutines, loops become `GOTO`-based constructs,
and `if` chains become `IF ... THEN GOTO` sequences.

**BASCAL is a strict superset of classic BASIC.** Raw statements from the
target dialect — `OPEN`/`FIELD`/`GET`/`PUT` for random-access files, bitwise
`AND`/`OR`/`NOT` — still compile unchanged. `GOTO`/`GOSUB`/`ON ERROR GOTO`/
`RESUME`/`RESTORE` are raw BASIC too, but with one restriction: BASCAL
manages line numbering itself, so their targets must be a `name:` label
declared in source, never a raw line number — see [Labels](#labels). Beyond
that, wherever this manual documents a BASCAL construct for something
(`select case` instead of an `IF`/`GOTO` dispatch chain, `record`/`file`
instead of hand-written `FIELD`/`GET`/`PUT`, `&&`/`||` instead of bitwise
short-circuit workarounds), treat that construct as the canonical way to
write it in `.bcl` source — the original BASIC syntax is what the
compiler exists to get you away from, not an equally good alternative.

---

## Getting Started

### Building the Compiler

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

Compile it:

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

A `.bcl` file consists of optional sections in the following order:

1. Optional `program` declaration
2. `require` / `import` dependency declarations
3. `common` declarations (suite files only)
4. Top-level statements (the main program body)
5. `function` definitions (may appear in any order relative to statements)

### Program Declaration

```
program name
program name suite suitename
```

The `program` declaration is optional. When present it must be the first
non-comment, non-blank line in the file. It identifies the program by name and
optionally links it to a suite (see [Suite COMMON](#suite-common)).

A `program` declaration is **not allowed** in library modules loaded via
`require`.

### File Encoding

Source files are UTF-8 text. Line endings may be LF or CRLF. Statements are
separated by newlines; a colon `:` may also separate statements on one line.

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

All type checking is deferred to the BASIC runtime. The BASCAL compiler does
not perform static type inference.

---

## Variables and Constants

### Variables

Variables declared or assigned at the top level are **global** and visible
throughout the entire program.

Variables inside a `function` or `procedure` body are **local by default**: the
compiler maps them to uniquely-generated BASIC names (e.g. `fname_var_0%`),
indexed against every name already in use at compile time so they're
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
| `/`      | Division (truncates toward zero) |
| `\`      | Integer division (floor quotient) |
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

`TRUE` and `FALSE` are compile-time sugar for BASIC's own boolean
convention — `-1` and `0` — so a programmer-boolean flag can be compared
against a name instead of a magic number:

```
found% = TRUE
done%  = FALSE

if found% = TRUE then
    print "found it"
end if
```

They compile straight through to the literals themselves — `found% = TRUE`
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
`0`. BASCAL's compiler emits `(expr) = 0` instead of `NOT expr` in generated
control-flow conditions so that programmer-boolean values like `found% = 1`
behave as expected. Use explicit `= 0` or `<> 0` comparisons in your own code
when testing boolean flags.

`AND`/`OR` always evaluate both sides — there's no short-circuit primitive in
generated BASIC at all. See [Short-Circuit `&&` and `||`](#short-circuit--and-)
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
 * Insertion sort — sorts arr%(0..count%-1) in ascending order.
 * Time complexity: O(n^2) average and worst case.
 * Space complexity: O(1) — sorts in place.
 */
' arr%   -- array to sort; byref because it's mutated in place
' count% -- number of elements in arr%
function insertionSort%(byref arr%, count%)
    for i% = 1 to count% - 1
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
— that's the only difference between the two. The single-line form may
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
`exit for`/`exit while`/`exit do` — the compiler already knows which loop
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
`exit`, with no loop-type keyword after it. The compiler resolves which
enclosing loop it leaves from context — the *innermost* one, if loops are
nested — so `exit` inside a `do` loop transpiles to a `GOTO` past the
loop's own end label, while `exit` inside a `for` loop transpiles to
BASIC's native `EXIT FOR` instead, since `for`/`next` compiles to a real
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
keyword after `exit` is a compile-time error.

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
one condition is a compile-time error — split into nested `if` statements
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
ispositive_n_0% = scores%(ptr%)
GOSUB 20
IF (ispositive_result_0% > 0) = 0 THEN GOTO 10
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
a `do until a% || b%` doesn't (mirroring a plain `&&`) — the compiler
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
dummy% = sortArray%(data%(), N%)
```

### Return

Every function must contain at least one `return` statement. Implicit returns
at end-of-body are not supported.

### Calling the Same Function Twice

Each call writes the shared `fname_result_0` variable, so assignments must be
made before the next call overwrites it. BASCAL handles this automatically:

```
a$ = repeat$("x", 3)   ' repeat_result$ = "xxx"  →  a$ = "xxx"
b$ = repeat$("y", 2)   ' repeat_result$ = "yy"   →  b$ = "yy"
PRINT a$ + " " + b$    ' xxx yy
```

### Variable Scoping

Variables inside a function body are **local by default**: the compiler maps
them to uniquely-generated BASIC names of the form `stem_var_0%`, `_1%`, etc.
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
own parameters — `function f%(x%) : global x% : ...` is a compile-time
error, since the parameter always resolves first and the `global`
declaration could never take effect.

### Restrictions

- **No recursion.** Functions are transpiled to `GOSUB` with global parameter
  variables. A recursive call would overwrite in-flight parameters. Use an
  explicit stack array to simulate recursion if needed.
- **No return value from a procedure.** Functions must `return` a value;
  for side-effect-only subroutines use `procedure` instead.

### How Functions Are Transpiled

The compiler transpiles each function call to:
1. Assign each argument to a generated global variable (e.g. `fname_param_0%`)
2. `GOSUB` to the function's generated label
3. Assign the result from the generated result variable (e.g. `fname_result_0%`)

Local variables in the function body are emitted as uniquely-indexed BASIC
globals (`fname_var_0%`, `fname_var_1%`, …). The index is chosen so the name
does not clash with any global variable or with any name allocated by an
earlier function, making collisions impossible regardless of what names the
developer uses at global scope.

Every parameter is copied into its generated name before the call. Whether
anything is copied back afterward depends on its passing mode — see
[byref / byval](#byref--byval).

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
' count% -- number of elements in arr%
' value% -- value written into every element
procedure fillRange(byref arr%, count%, value%)
    for i% = 0 to count% - 1
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
fillRange(data%(), N%, 99)
```

### Early Exit

A bare `return` (no expression) exits a procedure immediately.
Falling through to `end procedure` is equally valid — the compiler emits an
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

Array parameters use the same [byref / byval](#byref--byval) rules as
functions. Declare the parameter without `()` in the procedure header; pass
with `()` at the call site:

```
' arr%   -- array to fill; byref because it's mutated in place
' count% -- number of elements in arr%
' value% -- value written into every element
procedure fillRange(byref arr%, count%, value%)   ' arr% — no () in header
    ...
end procedure

fillRange(data%(), N%, 99)                        ' data%() — () at call site
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

- **No recursion.**  Same GOSUB transpilation as functions — a recursive call would
  overwrite in-flight parameters.
- **No return value.**  Do not use a procedure where an expression is expected.

### How Procedures Are Transpiled

Procedures use the same GOSUB mechanism as functions:

1. Assign each argument to a generated global variable (e.g. `pname_param_0%`)
2. `GOSUB` to the procedure's generated label
3. No result variable is read back

Local variables in the body are emitted as uniquely-indexed BASIC globals
(`pname_var_0%`, `pname_var_1%`, …) using the same collision-free scheme as
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

Declare the parameter with the plain variable name — **no `()` in the
declaration**. At the call site, write `arr%()` to signal that an array is
being passed. `insertionSort%` mutates the array in place, so its `arr%`
parameter needs `byref`; `indexOf%` only reads it, so the unmarked (`byval`)
default is correct as-is:

```
' arr%   -- array to sort; byref because it's mutated in place
' count% -- number of elements in arr%
function insertionSort%(byref arr%, count%)
    for i% = 1 to count% - 1
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
' count%  -- number of elements in arr%
' target% -- value to search for
function indexOf%(arr%, count%, target%)
    for i% = 0 to count% - 1
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

dummy% = insertionSort%(data%(), N%)   ' sorts in place -- arr% is byref

idx% = indexOf%(data%(), N%, 22)
if idx% >= 0 then
    PRINT "22 found at index " + STR$(idx%)
end if
```

See [byref / byval](#byref--byval) for exactly what gets copied, and when.

### `byref` / `byval`

Every parameter — scalar or array — is copied into its generated storage
before the call. Whether that value is copied back to the caller afterward
depends on how the parameter is declared:

```
function insertionSort%(byref arr%, count%)   ' byref: copied in, then back out
function indexOf%(arr%, count%, target%)      ' unmarked = byval: copied in only
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
  just a slower `byval`, since the compiler still generates the copy-out
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
  compile-time error, because there's nowhere for the result to be written
  back to.

If you're coming from classic MBASIC/BASCOM: there's no local scope there
at all, so a `GOSUB`-based "subroutine" touching an array was always
touching the *one* array that exists — mutations were visible everywhere,
instantly, because there was never more than one copy. BASCAL's parameters
don't work that way by default. `byval` (the default) gives the function
its own copy, and `byref` is what asks for the old always-visible,
always-shared behavior back, deliberately, per parameter.

### Multi-Dimensional Array Parameters

A 2-D (or higher) array passes the same way as 1-D — empty parens at the
call site — but needs one count argument per axis, not just one, in the
same order as the array's own `DIM`:

```
' grid% -- 2-D array to sum
' rows% -- number of rows in grid%
' cols% -- number of columns in grid%
function sumGrid%(byref grid%, rows%, cols%)
    total% = 0
    for r% = 0 to rows% - 1
        for c% = 0 to cols% - 1
            total% = total% + grid%(r%, c%)
        end for
    end for
    return total%
end function

dim g%(2, 2)
print sumGrid%(g%(), 3, 3)
```

The compiler infers how many dimensions a parameter has from how the
function's own body indexes it (`grid%(r%, c%)` above means two), then
checks that against the array actually being passed at each call site. A
mismatch — passing a 2-D array where a function indexes its parameter with
one subscript, or vice versa — is a compile-time error, not a
miscompile: the two shapes genuinely can't share one copy loop, so BASCAL
refuses rather than generate a `DIM`/subscript mismatch that real BASIC
would only catch at runtime.

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
packing helpers all compile as-is. But hand-summing field widths and
hand-matching pack/unpack calls is exactly the bookkeeping a compiler should
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
per field — numeric fields are packed first (`MKI%`/`MKL&`/`MKS!`/`MKD#`),
string fields are assigned directly — followed by a single `PUT #n, 1`.
`LSET` is used for every field, numeric or string: once a numeric value is
packed, the result is exact-width binary, so left/right justification makes
no difference (this matches real BASCOM practice).

A record literal missing a declared field is a **compile-time error** — this
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
compile time, by comparing the field names you gave against the record's
declared fields — there's no runtime check:

- If the listed fields don't cover every declared field, an implicit
  `GET #n, i` is emitted first (so the unlisted fields keep their current
  on-disk values), then `LSET` for only the fields given, then `PUT #n, i`.
- If the listed fields happen to cover every declared field anyway, no
  `GET` is emitted — it transpiles exactly like a plain `{ ... }` literal.

Unlike `{ ... }`, an unknown field name inside `?{ ... }` is still a
compile-time error — only *missing* fields are permitted, not *misspelled*
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
(`CVI%`/`CVL&`/`CVS!`/`CVD#` for numeric fields, `RTRIM$` for strings), each
one written into a scalar named `<var>_<field>` — e.g. `s_id%`, `s_name$`,
`s_score#`. Later references to `s.id`, `s.name`, `s.score` in the source
resolve directly to those scalars; no `Ident` named literally `s.id` is ever
emitted.

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
compile-time error.

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

The transpilation pass rejects, at compile time: field names not declared on the
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

Replaces characters inside a string in place, without allocating a new string.

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
statements requires a label name; the compiler assigns the actual BASIC
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

PRINT "Mean:   " + STR$(mean!(scores%(), N%))
PRINT "Max:    " + STR$(maximum%(scores%(), N%))
PRINT "Min:    " + STR$(minimum%(scores%(), N%))
PRINT "Range:  " + STR$(rangeOf%(scores%(), N%))
END
```

Compile with `-L tutorial/lib` so that `require stats` resolves to
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

The compiler searches for the file in:
1. The directory containing the current source file
2. Additional directories supplied with `-L` flags (in order)

Dependencies are resolved recursively. A file is loaded at most once per
compilation (circular dependencies are silently ignored after the first load).

### Function Merging

All functions from a required file (and its transitive dependencies) are merged
into the generated output. Duplicate function names are rejected with a
diagnostic error.

### Module Conventions

By convention, library modules (files loaded via `require`) should:
- Contain only `function` definitions and supporting `DIM` / `DATA` statements
- Not contain a `program` declaration
- Not contain top-level executable statements other than `DIM` and `DATA`

---

## Suite COMMON

In classic BASCOM programs, multiple programs chained together with `CHAIN`
share variables through `COMMON` declarations. For this to work correctly,
every program in the chain must declare **identical** `COMMON` lists — the
variable positions in the `COMMON` block must match exactly.

BASCAL coordinates `COMMON` through suite files. A suite file contains only
variable declarations (see below); programs that belong to the suite
reference it with a `suite` clause on their `program` declaration.

### Suite File

A suite file is a `.bcl` file containing only variable declarations — `dim`
and/or `common` (see [DIM Declaration](#dim-declaration-recommended) and
[COMMON Declaration](#common-declaration) below) — plus blank lines and
comments.

The canonical form starts with a `suite <name>` header, analogous to a
regular file's `program <name>` header, and declares its shared variables
with ordinary `dim`:

From `tutorial/13_suite/shared.bcl`:

```
/*
 * Suite file for Tutorial 13 — COMMON / CHAIN.
 *
 * Every program that begins with "program name suite shared" receives
 * an identical COMMON block at the top of its generated BASIC, so the
 * listed variables survive a CHAIN to the next program.
 */
suite shared

dim count%
dim label$
```

The older filename-only form — no `suite` header, just one or more `common`
declarations, with the suite name taken purely from the filename — still
works and compiles to identical output; see
[COMMON Declaration](#common-declaration) below.

Rules for suite files:
- Only `dim`/`common` declarations, blank lines, and comments are allowed.
- `require`, `function`, executable statements, and `program` declarations
  are all rejected with a diagnostic error.
- The suite file must contain at least one `dim` or `common` declaration.
- A file may not have both a `program` header and a `suite` header — a file
  is either an ordinary program (optionally referencing a suite) or a suite
  definition, never both.

### DIM Declaration (recommended)

```
suite shared

dim count%
dim label$
dim scores%()
```

Inside a `suite <name>`-headed file, every top-level `dim` becomes one
shared variable, in declaration order — exactly the [DIM](#dim) statement
used anywhere else in BASCAL, including its multi-name comma form
(`dim count%, label$`) and array declarations (`dim scores%()`,
empty-parens, same as a `COMMON` array). No bounds are stored either way —
`common`/suite `dim` only ever declares *that* a name is an array, not its
size.

If present, the `suite <name>` header's name must match the filename the
compiler resolved it as (`shared.bcl` → `suite shared`) — a mismatch is a
compile-time error, catching a suite file copied to a new filename without
updating its own header.

### COMMON Declaration

```
common var1%, var2$, arr%()
```

The older, pre-`suite`-header spelling: lists the variables that participate
in the `COMMON` block directly, without a `dim`. Array names are written
with empty parentheses `()`. Still fully supported — a suite file needs
*either* `dim` or `common` declarations (or both), not specifically one.

Multiple `common` declarations are allowed; each generates a separate `COMMON`
line in the output:

```
common score%, level%, playerName$
common hiScore%
```

Generates:

```
COMMON score%, level%, playerName$
COMMON hiScore%
```

### Program Declaration with Suite

```
program start suite shared
```

When a suite name is present, the compiler:
1. Searches for `shared.bcl` in the source file's directory (then `-L` paths).
2. Validates that the suite file contains only `common` declarations.
3. Emits the `COMMON` lines at the very top of the generated `.bas` file,
   before any other output.

### Using the Suite

From `tutorial/13_suite/` — two programs that share `count%` and `label$`:

**`shared.bcl`** (suite file):
```
suite shared

dim count%
dim label$
```

**`start.bcl`** (program 1):
```
program start suite shared

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
program show suite shared

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

- `common` is illegal everywhere except in suite files. Using `common` in a
  regular program or library module is a compile error.
- A `suite <name>` header is illegal everywhere except in a suite file being
  loaded as a suite — a stray `suite` header in an ordinary program or
  library module is a compile error, same as `common`.
- A `program` declaration is illegal in library modules (files loaded via
  `require`).
- A file cannot have both a `program` header and a `suite` header.
- If the named suite file does not exist, the program compiles without a
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

If a suite is declared, `COMMON` lines appear before the header comment.

### Line Numbers

By default, only lines that are branch targets (destinations of `GOTO` or
`GOSUB`) receive line numbers. All other lines are unnumbered. Use
`--line-numbers` to number every line.

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
by the compiler:

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
compiles to plain `RETURN`.

### Select Case Transpilation

`SELECT CASE` is transpiled to an `IF`/`GOTO` dispatch chain. The select
expression is stored in a temporary variable (e.g., `BCC_T1%`) to avoid
re-evaluation.

### Exit Statements

`exit` is unqualified in BASCAL source; the compiler picks the shape below
based on which loop it's innermost inside:

- inside `for` → `EXIT FOR` (native FreeBASIC / QB extension)
- inside `while` → `GOTO end_label`
- inside `do` → `GOTO end_label`

---

## Command-Line Reference

```
bcc input.bcl [-o output.bas] [-L dir] [-l library]
              [--line-numbers] [--clean | -c] [--binary | -b]
```

| Flag | Short | Description |
|------|-------|-------------|
| `-o output.bas` | | Output file path. Default: source path with `.bas` extension in the same directory. |
| `-L dir` | | Add a directory to the library search path. Repeatable. |
| `-l name` | | Name a library (reserved). |
| `--line-numbers` | | Number every output line, not just branch targets. |
| `--clean` | `-c` | Recompile even if the output is already up to date. |
| `--binary` | `-b` | Invoke `fbc` after compilation to produce a binary. The binary is placed in `tmp/`. |

### Up-to-Date Check

Without `--clean`, the compiler skips recompilation if the output `.bas` file
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
| `COMMON` | `common var[, ...]` | Declare suite COMMON variables (suite files only) |
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
| `SUITE` | `suite name` | Declare this file as suite `name` (suite files only; shared vars listed via `dim`) |
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

| Name | Compiles to |
|------|-------------|
| `TRUE` | `-1` |
| `FALSE` | `0` |
