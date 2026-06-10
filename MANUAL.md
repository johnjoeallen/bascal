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
10. [Arrays](#arrays)
11. [Input and Output](#input-and-output)
12. [File Input and Output](#file-input-and-output)
13. [Data Statements](#data-statements)
14. [Miscellaneous Statements](#miscellaneous-statements)
15. [Dependencies — REQUIRE and IMPORT](#dependencies--require-and-import)
16. [Suite COMMON](#suite-common)
17. [Generated BASIC Shape](#generated-basic-shape)
18. [Command-Line Reference](#command-line-reference)
19. [Statement Quick Reference](#statement-quick-reference)

---

## Introduction

BASCAL is a compiler that translates structured `.bcl` source files into
line-numbered Microsoft BASIC programs (`.bas`) compatible with BASCOM and
FreeBASIC's QB compatibility mode.

BASCAL keeps BASIC's global symbol model and run-time semantics while adding the
structural constructs needed to write and maintain larger programs:

- Block `if` / `else if` / `else` / `end if`
- `for` / `next`, `while` / `wend`, and `do` / `loop` loops with early exit
- `function` declarations with typed return values and explicit `return`
- Path-style `require` for multi-file projects
- `program` / `suite` declarations for coordinating `COMMON` across chained
  programs
- Multi-line `/* */` block comments in addition to single-line `'` comments
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

Create `hello.bcl`:

```
PRINT "Hello, World!"
END
```

Compile it:

```
bcc hello.bcl
```

This produces `hello.bas` in the same directory. To compile and run with
FreeBASIC:

```
bcc hello.bcl --binary
./tmp/hello
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

All variables are **global**. There is no local variable scoping. Variables
declared or assigned inside a function body are visible throughout the entire
program.

Variables do not require pre-declaration; they come into existence on first
assignment. Use `DIM` to declare arrays or to make intent clear.

### DIM

Declares an array or a simple variable.

```
DIM name%
DIM scores%(100)
DIM names$(50)
```

When a size expression is provided, the variable is treated as a fixed-size
array with indices 0 through *size*. The size expression may be any integer
expression.

### CONST

Declares a named constant. The value must be a compile-time literal or
expression.

```
CONST MAX_LINES% = 1000
CONST GREETING$ = "Hello"
CONST PI! = 3.14159
```

Constants follow the same type-suffix rules as variables. Once declared, a
constant may not be reassigned.

---

## Operators and Expressions

### Arithmetic Operators

| Operator | Operation      |
|----------|----------------|
| `+`      | Addition / string concatenation |
| `-`      | Subtraction / unary negation    |
| `*`      | Multiplication |
| `/`      | Division       |

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

**Important:** `NOT` is bitwise in Microsoft BASIC. `NOT 1` yields `−2`, not
`0`. BASCAL's compiler emits `(expr) = 0` instead of `NOT expr` in generated
control-flow conditions so that programmer-boolean values like `found% = 1`
behave as expected. Use explicit `= 0` or `<> 0` comparisons in your own code
when testing boolean flags.

### Operator Precedence (highest first)

| Level | Operators        |
|-------|------------------|
| 7     | Unary `-`, `NOT` |
| 6     | `*`, `/`         |
| 5     | `+`, `-`         |
| 4     | `=`, `<>`, `<`, `<=`, `>`, `>=` |
| 3     | `AND`            |
| 2     | `OR`             |

Use parentheses to override precedence.

---

## Comments

### Single-Line Comments

A single quote `'` begins a comment that extends to the end of the line.
Comments are passed through to the generated BASIC output unchanged.

```
' This is a single-line comment
score% = 0  ' inline comment
```

### Block Comments

Block comments span multiple lines. The opening delimiter is `/*` and the
closing delimiter is `*/`. Block comments may appear anywhere a statement is
valid.

```
/*
 * ARCADE SUITE — menu program
 * Initialises shared state and welcomes the player.
 *
 * Variables shared with game.bas via the arcade.bcl suite:
 *   score%, level%, playerName$, hiScore%
 */
program menu suite arcade
```

Each non-empty line of a block comment is emitted as a separate `'` comment in
the generated BASIC output. Leading `*` characters and surrounding whitespace
are stripped.

One-line block comments are also valid:

```
/* Set initial values. */
score% = 0
```

---

## Control Flow

### IF / ELSE IF / ELSE / END IF

```
if condition then
    ' then body
end if

if condition then
    ' then body
else
    ' else body
end if

if score% > 100 then
    PRINT "High score!"
elseif score% > 50 then
    PRINT "Good score."
else
    PRINT "Keep trying."
end if
```

`elseif` chains may be arbitrarily deep. BASCAL lowers `elseif` to nested
`if`/`else` structures; no new AST node is introduced.

### FOR / NEXT

```
for i% = 1 to 10
    PRINT i%
next i%

for i% = 10 to 1 step -1
    PRINT i%
next

for i% = 0 to count% - 1 step 2
    process%(data%(i%), data%(i% + 1))
next i%
```

The variable name after `next` is optional. The `step` clause is optional;
the default step is 1.

`exit for` exits the enclosing `for` loop immediately.

### WHILE / WEND

```
while condition
    ' body
wend

i% = 0
while i% < 10
    PRINT i%
    i% = i% + 1
wend
```

`exit while` exits the enclosing `while` loop immediately, jumping to the
statement after `wend`.

### DO / LOOP

The `do` statement supports four forms:

```
' Unconditional loop (exit with EXIT DO or GOTO)
do
    ' body
loop

' Test at top — loop while condition is true
do while condition
    ' body
loop

' Test at top — loop until condition is true
do until condition
    ' body
loop

' Test at bottom — always executes body at least once
do
    ' body
loop while condition

do
    ' body
loop until condition
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

```
function add%(left%, right%)
    return left% + right%
end function

function fullName$(first$, last$)
    return first$ + " " + last$
end function

function noReturn%()
    PRINT "side effect"
    return 0
end function
```

### Calling Functions

```
total% = add%(10, 20)
PRINT fullName$("John", "Smith")
dummy% = noReturn%()
```

Functions called as standalone statements (not used in an expression) are
written as expression statements:

```
processData%(buffer%(), count%)
```

### Return

Every function must contain at least one `return` statement. Implicit returns
at end-of-body are not supported.

```
function max%(a%, b%)
    if a% > b% then
        return a%
    else
        return b%
    end if
end function
```

### Restrictions

- **No recursion.** Functions are lowered to `GOSUB` with global parameter
  variables. A recursive call would overwrite in-flight parameters. Use an
  explicit stack array to simulate recursion if needed.
- **No local scope.** All variables inside a function body are global. Use
  function-name-prefixed variable names (e.g., `myFunc_temp%`) to avoid
  collisions.

### How Functions Are Lowered

The compiler lowers each function call to:
1. Assign each argument to a global variable `fname_paramname`
2. `GOSUB` to the function's generated label
3. Assign the result from `fname_result`

Array parameters are copy-in / copy-out: the array elements are copied into
`fname_paramname(i)` before the call and back into the caller's array after.

---

## Arrays

### Declaration

```
DIM values%(100)
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

Declare the parameter with an empty `()`:

```
function sum%(data%(), count%)
    total% = 0
    for i% = 0 to count% - 1
        total% = total% + data%(i%)
    next i%
    return total%
end function

DIM nums%(5)
nums%(0) = 10 : nums%(1) = 20 : nums%(2) = 30
result% = sum%(nums%(), 3)
```

Array arguments use copy-in / copy-out. The next non-array argument after an
array parameter is used as the element count for the copy loop.

---

## Input and Output

### PRINT

Prints one or more expressions to the screen. Expressions are separated by
commas.

```
PRINT "Hello, World!"
PRINT name$, score%
PRINT "Score: " + STR$(score%)
PRINT                        ' blank line
```

### LPRINT

Sends output to the printer (line printer). Same syntax as `PRINT`.

```
LPRINT "Report for: " + date$
LPRINT total%
```

### INPUT

Reads values from the keyboard.

```
INPUT name$
INPUT "Enter your name: "; name$
INPUT "Width, height: "; width%, height%
```

A prompt string followed by `;` suppresses the newline after the prompt (the
cursor remains on the same line). A prompt followed by `,` adds a `?` after the
prompt and moves to the next line before input. The `;` form is recommended.

Multiple variables may be listed; the user enters values separated by commas.

### LOCATE

Positions the cursor at a specific row and column before printing.

```
LOCATE 12, 1
PRINT "Centred text"
LOCATE row%, col%
```

Rows and columns are 1-based on standard 80×25 displays.

### COLOR

Sets the foreground and optional background colour.

```
COLOR 14          ' bright yellow foreground
COLOR 15, 1       ' bright white on blue
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

### OPEN

Opens a file for reading, writing, or appending.

```
OPEN filename$ FOR INPUT AS #1
OPEN filename$ FOR OUTPUT AS #2
OPEN filename$ FOR APPEND AS #3
```

The file number (`#1`, `#2`, etc.) is used in subsequent file I/O statements.
Up to a BASIC runtime limit of concurrent files may be open simultaneously.

### CLOSE

Closes an open file.

```
CLOSE #1
```

### LINE INPUT

Reads one complete line from a file into a string variable.

```
LINE INPUT #1, line$
```

### PRINT # (File Print)

Writes expressions to a file.

```
PRINT #2, name$
PRINT #2, count%, value!
```

### WRITE #

Writes expressions to a file in a format readable by `INPUT #`. Strings are
enclosed in double quotes; values are separated by commas.

```
WRITE #2, name$, score%, level%
```

### INPUT # (File Input)

Reads comma-separated values from a file into variables.

```
INPUT #1, name$, score%, level%
```

### Typical File Loop Pattern

```
OPEN "data.txt" FOR INPUT AS #1
while EOF(1) = 0
    LINE INPUT #1, line$
    PRINT line$
wend
CLOSE #1
```

---

## Data Statements

`DATA`, `READ`, and `RESTORE` provide an embedded data table that the program
reads at run time.

### DATA

Embeds literal values into the program. `DATA` statements may appear anywhere
in the program body.

```
DATA 10, 20, 30, 40, 50
DATA "Alice", "Bob", "Carol"
```

### READ

Reads the next value(s) from the `DATA` stream into variables.

```
READ value%
READ name$, score%
```

Values are read in the order `DATA` statements appear in the generated output.

### RESTORE

Resets the `DATA` pointer to the beginning (or to a specific line number).

```
RESTORE
RESTORE 1000
```

### Example

```
RESTORE
for i% = 1 to 3
    READ name$, score%
    PRINT name$ + ": " + STR$(score%)
next i%
END

DATA "Alice", 95
DATA "Bob", 87
DATA "Carol", 91
```

---

## Miscellaneous Statements

### SWAP

Exchanges the values of two variables.

```
SWAP a%, b%
SWAP names$(i%), names$(j%)
```

### RANDOMIZE

Seeds the random number generator. With no argument, the runtime may prompt
for a seed or use a default.

```
RANDOMIZE
RANDOMIZE TIMER
RANDOMIZE seed%
```

### GOTO

Transfers control to a line number.

```
GOTO 1000
```

Using `GOTO` with literal line numbers is valid but bypasses BASCAL's
structured constructs. Prefer `if`, loops, and functions. `GOTO` is primarily
useful for error handlers.

### GOSUB / RETURN (BASIC-level)

Calls a BASIC subroutine at a line number. Note this is the raw BASIC `GOSUB`,
distinct from the function-call mechanism BASCAL generates internally.

```
GOSUB 2000
```

### ON ... GOTO / ON ... GOSUB

Computed branch: transfers to one of several targets based on an integer
expression. The expression selects the *n*th target (1-based).

```
ON choice% GOTO 100, 200, 300
ON mode% GOSUB 500, 600, 700
```

If the expression evaluates to 0 or exceeds the number of targets, execution
continues with the next statement.

### STOP

Terminates the program immediately (equivalent to `END` in most BASIC
dialects; may invoke the debugger in some implementations).

```
STOP
```

### SYSTEM

Exits to the operating system immediately.

```
SYSTEM
```

### END

Signals the end of the main program body. `END` must appear at the logical end
of the main program. Functions are emitted after `END` in the generated output.

```
END
```

---

## Dependencies — REQUIRE and IMPORT

BASCAL supports multi-file projects through `require` (and its alias `import`).
Dependencies are declared at the top of the file, before any statements.

```
require com.bascal.sort.bubbleSort
require com.bascal.io.readline
import  com.vendor.utils.strings
```

`require` and `import` are identical in behaviour.

### Path Resolution

The dot-separated path is converted to a file path by replacing each `.` with
a directory separator and appending `.bcl`:

```
com.bascal.sort.bubbleSort  →  com/bascal/sort/bubbleSort.bcl
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
variable positions in the COMMON block must match exactly.

BASCAL coordinates `COMMON` through suite files. A suite file contains only
`common` declarations; programs that belong to the suite reference it with a
`suite` clause on their `program` declaration.

### Suite File

A suite file is a `.bcl` file containing only `common` declarations (and
comments). The file name, without extension, is the suite name.

```
' arcade.bcl — suite definition for the ARCADE programs
common score%, level%, playerName$
common hiScore%
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
program menu suite arcade
```

When a suite name is present, the compiler:
1. Searches for `arcade.bcl` in the source file's directory (then `-L` paths).
2. Validates that the suite file contains only `common` declarations.
3. Emits the `COMMON` lines at the very top of the generated `.bas` file,
   before any other output.

### Using the Suite

Create one suite file and reference it from every program in the set:

**`arcade.bcl`** (suite file):
```
/* Shared variables for all ARCADE suite programs. */
common score%, level%, playerName$
common hiScore%
```

**`menu.bcl`**:
```
program menu suite arcade

INPUT "Your name: "; playerName$
score% = 0
level% = 1
PRINT "Welcome, " + playerName$
END
```

**`game.bcl`**:
```
program game suite arcade

score% = score% + 50 * level%
level% = level% + 1
PRINT "Score: " + STR$(score%)
END
```

Both `menu.bas` and `game.bas` will begin with:
```
COMMON score%, level%, playerName$
COMMON hiScore%
```

ensuring that `CHAIN "game.bas"` from `menu.bas` leaves the variables in the
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
while i% < 10
    i% = i% + 1
wend
```

Becomes:

```
10 IF (i% < 10) = 0 THEN GOTO 20
    i% = i% + 1
    GOTO 10
20 REM END WHILE
```

### Do Lowering

```
do while i% < 10
    i% = i% + 1
loop
```

Becomes:

```
10 IF (i% < 10) = 0 THEN GOTO 20
    i% = i% + 1
    GOTO 10
20 REM END DO
```

### For Lowering

BASCAL emits native `FOR` / `NEXT`, which BASIC runtimes handle efficiently:

```
FOR i% = 1 TO 10
    PRINT i%
NEXT i%
```

### Function Lowering

```
function add%(left%, right%)
    return left% + right%
end function

total% = add%(10, 20)
```

Becomes:

```
add_left% = 10
add_right% = 20
GOSUB 10
total% = add_result%
...
END

' function add%(left%, right%)
10 add_result% = add_left% + add_right%
    RETURN
' end function add%
```

### Select Case Lowering

`SELECT CASE` is lowered to a `IF`/`GOTO` dispatch chain. The select
expression is stored in a temporary variable (`SEL_nnnn`) to avoid
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
| `DIM` | `DIM name[(size)]` | Declare a variable or array |
| `DO` | `DO [WHILE/UNTIL cond]` … `LOOP [WHILE/UNTIL cond]` | Conditional loop |
| `END` | `END` | End of program |
| `EXIT DO` | `EXIT DO` | Exit enclosing DO loop |
| `EXIT FOR` | `EXIT FOR` | Exit enclosing FOR loop |
| `EXIT WHILE` | `EXIT WHILE` | Exit enclosing WHILE loop |
| `FOR` | `FOR v = start TO end [STEP s]` … `NEXT [v]` | Counted loop |
| `FUNCTION` | `FUNCTION name(params)` … `END FUNCTION` | Define a function |
| `GOSUB` | `GOSUB lineno` | Call BASIC subroutine |
| `GOTO` | `GOTO lineno` | Unconditional branch |
| `IF` | `IF cond THEN` … [`ELSEIF` …] [`ELSE` …] `END IF` | Conditional block |
| `INPUT` | `INPUT [prompt;] var[, ...]` | Read from keyboard |
| `INPUT #` | `INPUT #n, var[, ...]` | Read from file |
| `LET` | `LET var = expr` | Assignment (keyword optional) |
| `LINE INPUT` | `LINE INPUT #n, var$` | Read full line from file |
| `LOCATE` | `LOCATE row, col` | Position cursor |
| `LPRINT` | `LPRINT expr[, ...]` | Print to printer |
| `ON...GOTO` | `ON expr GOTO n1, n2, ...` | Computed GOTO |
| `ON...GOSUB` | `ON expr GOSUB n1, n2, ...` | Computed GOSUB |
| `OPEN` | `OPEN file$ FOR INPUT/OUTPUT/APPEND AS #n` | Open file |
| `PRINT` | `PRINT expr[, ...]` | Print to screen |
| `PRINT #` | `PRINT #n, expr[, ...]` | Print to file |
| `RANDOMIZE` | `RANDOMIZE [seed]` | Seed random number generator |
| `READ` | `READ var[, ...]` | Read from DATA stream |
| `REQUIRE` | `require path.symbol` | Load dependency module |
| `RESTORE` | `RESTORE [lineno]` | Reset DATA pointer |
| `RETURN` | `RETURN expr` | Return value from function |
| `SELECT CASE` | `SELECT CASE expr` … `END SELECT` | Multi-way branch |
| `STOP` | `STOP` | Stop program execution |
| `SWAP` | `SWAP a, b` | Exchange two variable values |
| `SYSTEM` | `SYSTEM` | Exit to operating system |
| `WHILE` | `WHILE cond` … `WEND` | Condition-at-top loop |
| `WRITE #` | `WRITE #n, expr[, ...]` | Write to file (quoted format) |
