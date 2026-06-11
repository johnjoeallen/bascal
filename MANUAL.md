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
15. [Data Statements](#data-statements)
16. [Miscellaneous Statements](#miscellaneous-statements)
17. [Dependencies — REQUIRE and IMPORT](#dependencies--require-and-import)
18. [Suite COMMON](#suite-common)
19. [Generated BASIC Shape](#generated-basic-shape)
20. [Command-Line Reference](#command-line-reference)
21. [Statement Quick Reference](#statement-quick-reference)

---

## Introduction

BASCAL is a compiler that translates structured `.bcl` source files into
line-numbered Microsoft BASIC programs (`.bas`) compatible with BASCOM and
FreeBASIC's QB compatibility mode.

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
plain Microsoft BASIC. The structured constructs are lowered by the compiler:
functions become `GOSUB` subroutines, loops become `GOTO`-based constructs,
and `if` chains become `IF ... THEN GOTO` sequences.

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
| `%`    | Integer | 16-bit signed, −32768 to 32767           |
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
compiler automatically prefixes them with the function/procedure name so they
cannot collide with variables elsewhere.  To read or write a global variable
from inside a function or procedure, declare it at the top of the body with the
`global` keyword:

```
total% = 0

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

Comparison expressions evaluate to −1 (true) or 0 (false) at the BASIC
runtime, consistent with Microsoft BASIC semantics.

### Logical Operators

| Operator | Meaning |
|----------|---------|
| `AND`    | Bitwise AND (also serves as logical AND when operands are 0/−1) |
| `OR`     | Bitwise OR  |
| `NOT`    | Bitwise NOT |
| `XOR`    | Bitwise XOR |

**Important:** `NOT` is bitwise in Microsoft BASIC. `NOT 1` yields `−2`, not
`0`. BASCAL's compiler emits `(expr) = 0` instead of `NOT expr` in generated
control-flow conditions so that programmer-boolean values like `found% = 1`
behave as expected. Use explicit `= 0` or `<> 0` comparisons in your own code
when testing boolean flags.

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
| −1    | `XOR`            |

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
function insertionSort%(arr%, count%)
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

BASCAL only supports block-style `if` statements. The body must be on a
separate line; there is no single-line `IF … THEN stmt` form.

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

' EXIT FOR — stop at the first even number greater than 4
for i% = 1 to 20
    if i% > 4 and (i% / 2) * 2 = i% then
        PRINT "First even > 4: " + STR$(i%)
        exit for
    end if
end for
```

`exit for` exits the enclosing `for` loop immediately.

### WHILE / END WHILE

```
while condition
    ' body
end while
```

`end while` closes the loop. Bare `end` also works.

From `tutorial/05_loops.bcl`:

```
' Powers of 2 under 100
p% = 1
while p% < 100
    PRINT "  " + STR$(p%)
    p% = p% * 2
end while

' EXIT WHILE — stop after 8 Collatz steps
n% = 27
steps% = 0
while n% <> 1
    if steps% = 8 then
        PRINT "  ..."
        exit while
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

`exit while` exits the enclosing `while` loop immediately.

### DO / END DO

```
do [while/until condition]
    ' body
end do
```

`end do` closes the loop. Bare `end` also works. The optional `while` or
`until` clause tests the condition before each iteration.

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

' Run body at least once (post-check via EXIT DO)
k% = 99
do
    PRINT "  " + STR$(k%)    ' prints 99 even though k% > 3
    k% = k% + 1
    if k% > 3 then
        exit do
    end if
end do

' EXIT DO
k% = 1
do
    if k% = 3 then
        exit do
    end if
    PRINT "  " + STR$(k%)
    k% = k% + 1
end do
```

`exit do` exits the enclosing `do` loop immediately.

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
function max%(a%, b%)
    if a% > b% then
        return a%
    else
        return b%
    end if
end function

function min%(a%, b%)
    if a% < b% then
        return a%
    else
        return b%
    end if
end function

function clamp%(value%, lo%, hi%)
    ' Constrain value to [lo, hi].
    return max%(lo%, min%(value%, hi%))
end function

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

Each call writes the shared `fname_result` variable, so assignments must be
made before the next call overwrites it. BASCAL handles this automatically:

```
a$ = repeat$("x", 3)   ' repeat_result$ = "xxx"  →  a$ = "xxx"
b$ = repeat$("y", 2)   ' repeat_result$ = "yy"   →  b$ = "yy"
PRINT a$ + " " + b$    ' xxx yy
```

### Variable Scoping

Variables inside a function body are **local by default**: the compiler prefixes
them with the function stem.  Two functions can each have a variable named `i%`
with no conflict.  Use `global varname` to access a module-level variable:

```
function sumTo%(n%)
    acc% = 0                ' local to sumTo%
    for i% = 1 to n%       ' local to sumTo%
        acc% = acc% + i%
    end for
    return acc%
end function

runningTotal% = 0

function addToTotal%(x%)
    global runningTotal%    ' refers to the module-level variable
    runningTotal% = runningTotal% + x%
    return runningTotal%
end function
```

### Restrictions

- **No recursion.** Functions are lowered to `GOSUB` with global parameter
  variables. A recursive call would overwrite in-flight parameters. Use an
  explicit stack array to simulate recursion if needed.
- **No return value from a procedure.** Functions must `return` a value;
  for side-effect-only subroutines use `procedure` instead.

### How Functions Are Lowered

The compiler lowers each function call to:
1. Assign each argument to a global variable `fname_paramname`
2. `GOSUB` to the function's generated label
3. Assign the result from `fname_result`

Local variables in the function body are emitted as prefixed global BASIC
variables (e.g., `i%` in `sumTo%` becomes `sumto_i%`).

Array parameters use copy-in / copy-out: elements are copied into
`fname_paramname(i)` before the call and back into the caller's array after.

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
there is no return value.  Parameter names still carry their usual type suffixes.

From `tutorial/14_procedures.bcl`:

```
procedure printSeparator()
    PRINT "----------------------------"
end procedure

procedure printScore(label$, score%)
    PRINT label$ + ": " + STR$(score%)
end procedure

procedure printIfPass(name$, score%)
    if score% < 60 then
        return          // early exit — nothing printed for failing scores
    end if
    PRINT name$ + " passed with " + STR$(score%)
end procedure

procedure fillRange(arr%, count%, value%)
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
procedure printIfPass(name$, score%)
    if score% < 60 then
        return      ' exit early; nothing is printed
    end if
    PRINT name$ + " passed with " + STR$(score%)
end procedure
```

### Array Parameters

Array parameters use the same copy-in / copy-out convention as functions.
Declare the parameter without `()` in the procedure header; pass with `()` at
the call site:

```
procedure fillRange(arr%, count%, value%)   ' arr% — no () in header
    ...
end procedure

fillRange(data%(), N%, 99)                  ' data%() — () at call site
```

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

- **No recursion.**  Same GOSUB lowering as functions — a recursive call would
  overwrite in-flight parameters.
- **No return value.**  Do not use a procedure where an expression is expected.

### How Procedures Are Lowered

Procedures use the same GOSUB mechanism as functions:

1. Assign each argument to a global variable `pname_paramname`
2. `GOSUB` to the procedure's generated label
3. No result variable is read back

Local variables in the body are emitted as prefixed global BASIC variables.

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
being passed:

```
function insertionSort%(arr%, count%)
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

dummy% = insertionSort%(data%(), N%)   ' sorts in place

idx% = indexOf%(data%(), N%, 22)
if idx% >= 0 then
    PRINT "22 found at index " + STR$(idx%)
end if
```

Array arguments use copy-in / copy-out. The compiler generates loops that
copy elements into the function's parameter array before the `GOSUB` and
copy them back after the `RETURN`.

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
or a file.  The format string uses MS-BASIC format characters (`#` for digit
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

From `tutorial/15_random_files.bcl`:

Random-access files store fixed-length records that can be read or written in
any order, without scanning from the beginning.

### OPEN FOR RANDOM

```
open filename$ for random as #1 len = recLen%
```

`len` sets the record size in bytes.  Every record occupies exactly that many
bytes on disk.  Records are numbered from 1.

### FIELD

Binds string variables to regions of the shared file buffer:

```
field #1, 2 as idBuf$, 20 as nameBuf$, 8 as scoreBuf$
```

The widths must sum to the record length.  Only string variables may appear in
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

Resets the `DATA` pointer to the beginning (or to a specific line number).

```
RESTORE         ' rewind to the first DATA
RESTORE 1000    ' rewind to the DATA at line 1000
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
without modifying the original.  BASCAL handles the statement form as an
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

### GOTO

Transfers control to a line number. Prefer `if`, loops, and functions;
`GOTO` is primarily useful for error handlers.

```
GOTO 1000
```

### GOSUB / RETURN (BASIC-level)

Calls a BASIC subroutine at a line number. Note this is the raw BASIC `GOSUB`,
distinct from the function-call mechanism BASCAL generates internally.

```
GOSUB 2000
```

### ON ... GOTO / ON ... GOSUB

Computed branch: the integer expression selects the *n*th target (1-based).

```
ON choice% GOTO 100, 200, 300
ON mode%   GOSUB 500, 600, 700
```

If the expression evaluates to 0 or exceeds the number of targets, execution
continues with the next statement.

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
`common` declarations; programs that belong to the suite reference it with a
`suite` clause on their `program` declaration.

### Suite File

A suite file is a `.bcl` file containing only `common` declarations (and
comments). The file name, without extension, is the suite name.

From `tutorial/13_suite/shared.bcl`:

```
/*
 * Suite file for Tutorial 13 — COMMON / CHAIN.
 *
 * Every program that begins with "program name suite shared" receives
 * an identical COMMON block at the top of its generated BASIC, so the
 * listed variables survive a CHAIN to the next program.
 */
common count%, label$
```

Rules for suite files:
- Only `common` declarations, blank lines, and comments are allowed.
- `require`, `function`, executable statements, and `program` declarations
  are all rejected with a diagnostic error.
- The suite file must contain at least one `common` declaration.

### COMMON Declaration

```
common var1%, var2$, arr%()
```

Lists the variables that participate in the `COMMON` block. Array names are
written with empty parentheses `()`.

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
common count%, label$
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
- A `program` declaration is illegal in library modules (files loaded via
  `require`).
- If the named suite file does not exist, the program compiles without a
  `COMMON` block (no error). This allows incremental development.

---

## Generated BASIC Shape

Understanding how BASCAL lowers its constructs helps when reading generated
output or debugging.

### Header

Every generated file begins with:
```
' BASCAL generated BASIC
' Functions are lowered to global variables, labels, and GOSUB
```

### COMMON Block

If a suite is declared, `COMMON` lines appear before the header comment.

### Line Numbers

By default, only lines that are branch targets (destinations of `GOTO` or
`GOSUB`) receive line numbers. All other lines are unnumbered. Use
`--line-numbers` to number every line.

### If Lowering

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

### While Lowering

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

### Do Lowering

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

### For Lowering

BASCAL emits native `FOR` / `NEXT`, which BASIC runtimes handle efficiently.
The BASCAL `end for` (or bare `end`) is stripped; the BASIC `NEXT` is emitted
by the compiler:

```
FOR i% = 1 TO 5
    PRINT STR$(i%) + "^2 = " + STR$(i% * i%)
NEXT i%
```

### Function Lowering

```
function clamp%(value%, lo%, hi%)
    return max%(lo%, min%(value%, hi%))
end function

result% = clamp%(15, 1, 10)
```

The calls to `max%` and `min%` inside `clamp%` are also lowered to GOSUBs.
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
100 ' (lowered body — calls max% and min% via GOSUB)
    clamp_result% = ...
    RETURN
' end function clamp%
```

### Procedure Lowering

Procedures follow the same GOSUB pattern as functions but have no result
variable:

```
procedure printScore(label$, score%)
    PRINT label$ + ": " + STR$(score%)
end procedure

printScore("Alice", 91)
```

Lowers to:

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

There is no `printscore_result` variable.  A bare `return` inside a procedure
compiles to plain `RETURN`.

### Select Case Lowering

`SELECT CASE` is lowered to an `IF`/`GOTO` dispatch chain. The select
expression is stored in a temporary variable (e.g., `BCC_T1%`) to avoid
re-evaluation.

### Exit Statements

- `exit for` → `EXIT FOR` (native FreeBASIC / QB extension)
- `exit while` → `GOTO end_label`
- `exit do` → `GOTO end_label`

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
| `CLS` | `CLS` | Clear the screen |
| `CLOSE` | `CLOSE #n` | Close file channel *n* |
| `COLOR` | `COLOR fg[, bg]` | Set foreground/background colour |
| `COMMON` | `common var[, ...]` | Declare suite COMMON variables (suite files only) |
| `CONST` | `CONST name = expr` | Declare a named constant |
| `DATA` | `DATA val[, ...]` | Embed literal data values |
| `DIM` | `DIM name[(d1[, d2, ...])]` | Declare a variable or 1-D/multi-D array |
| `DO` | `DO [WHILE/UNTIL cond]` … `END DO` | Conditional loop |
| `END` | `END` | End of program |
| `EXIT DO` | `EXIT DO` | Exit enclosing DO loop |
| `EXIT FOR` | `EXIT FOR` | Exit enclosing FOR loop |
| `EXIT WHILE` | `EXIT WHILE` | Exit enclosing WHILE loop |
| `FOR` | `FOR v = start TO end [STEP s]` … `END FOR` | Counted loop |
| `FUNCTION` | `FUNCTION name%(params)` … `END FUNCTION` | Define a function with a return value |
| `GOSUB` | `GOSUB lineno` | Call BASIC subroutine |
| `GOTO` | `GOTO lineno` | Unconditional branch |
| `IF` | `IF cond THEN` … [`ELSEIF` …] [`ELSE` …] `END IF` | Conditional block |
| `INPUT` | `INPUT [prompt;] var[, ...]` | Read from keyboard |
| `KILL` | `KILL file$` | Delete a file |
| `INPUT #` | `INPUT #n, var[, ...]` | Read from file |
| `LET` | `LET var = expr` | Assignment (keyword optional) |
| `MID$` (stmt) | `MID$(str$, start[, len]) = repl$` | In-place substring replacement |
| `LINE INPUT` | `LINE INPUT #n, var$` | Read full line from file |
| `LOCATE` | `LOCATE row, col` | Position cursor |
| `LPRINT` | `LPRINT expr[, ...]` | Print to printer |
| `NAME` | `NAME old$ AS new$` | Rename a file |
| `ON...GOTO` | `ON expr GOTO n1, n2, ...` | Computed GOTO |
| `ON...GOSUB` | `ON expr GOSUB n1, n2, ...` | Computed GOSUB |
| `OPEN` | `OPEN file$ FOR INPUT/OUTPUT/APPEND AS #n` | Open file |
| `PRINT` | `PRINT expr[, ...]` | Print to screen |
| `PROCEDURE` | `PROCEDURE name(params)` … `END PROCEDURE` | Define a procedure (no return value) |
| `PRINT #` | `PRINT #n, expr[, ...]` | Print to file |
| `PRINT USING` | `PRINT USING fmt$; expr[; ...]` | Formatted print (also `LPRINT USING`, `PRINT #n, USING`) |
| `RANDOMIZE` | `RANDOMIZE [seed]` | Seed random number generator |
| `READ` | `READ var[, ...]` | Read from DATA stream |
| `REQUIRE` | `require path.symbol` | Load dependency module |
| `RESTORE` | `RESTORE [lineno]` | Reset DATA pointer |
| `RETURN` | `RETURN expr` / `RETURN` | Return value from function; bare form exits a procedure early |
| `SELECT CASE` | `SELECT CASE expr` … `END SELECT` | Multi-way branch |
| `STOP` | `STOP` | Stop program execution |
| `SWAP` | `SWAP a, b` | Exchange two variable values |
| `SYSTEM` | `SYSTEM` | Exit to operating system |
| `WHILE` | `WHILE cond` … `END WHILE` | Condition-at-top loop |
| `WRITE #` | `WRITE #n, expr[, ...]` | Write to file (quoted format) |
