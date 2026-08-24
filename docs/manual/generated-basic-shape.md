[Home](../) / [Manual](../manual/) / Generated BASIC Shape

[← Shared COMMON](shared-common.md) [Command-Line Reference →](command-line-reference.md)

<div class="prose" markdown="1">

Understanding how BASCAL transpiles its constructs helps when reading generated output or debugging.

### Header

Every generated file begins with:

    ' BASCAL generated BASIC
    ' Functions are transpiled to global variables, labels, and GOSUB

### COMMON Block

If a shared file is referenced, `COMMON` lines appear before the header comment.

### Line Numbers

By default, `bcc` numbers every emitted line, not just branch targets. Real MBASIC/BASCOM has no notion of an unnumbered statement line -- classic BASIC source is a sequence of numbered lines, full stop -- so this is what real compilers and interpreters expect. Numbered comment-only lines are harmless on real BASCOM, but an unnumbered *statement* line is a syntax error.

Pass `--sparse-line-numbers` to fall back to the old behavior, numbering only lines that are branch targets (destinations of `GOTO` or `GOSUB`) and leaving everything else unnumbered. This is more readable, but only safe with more lenient dialects (e.g. FreeBASIC's `-lang qb`) -- not real MBASIC/BASCOM.

### If Transpilation

    if x% > 0 then
        PRINT "positive"
    end if

Becomes:

    IF (x% > 0) = 0 THEN GOTO 10
        PRINT "positive"
    10 REM END IF

The condition is inverted with `= 0` rather than `NOT` to avoid bitwise semantics (see [Operators](operators-and-expressions.md#operators-and-expressions)).

### While Transpilation

    p% = 1
    while p% < 100
        PRINT STR$(p%)
        p% = p% * 2
    end while

Becomes:

    p% = 1
    10 IF (p% < 100) = 0 THEN GOTO 20
        PRINT STR$(p%)
        p% = p% * 2
        GOTO 10
    20 REM END WHILE

### Do Transpilation

    do while k% <= 3
        PRINT STR$(k%)
        k% = k% + 1
    end do

Becomes:

    10 IF (k% <= 3) = 0 THEN GOTO 20
        PRINT STR$(k%)
        k% = k% + 1
        GOTO 10
    20 REM END DO

The post-check form skips the leading guard entirely, since the body always runs at least once:

    do
        PRINT STR$(k%)
        k% = k% + 1
    loop until k% > 3

Becomes:

    10 PRINT STR$(k%)
        k% = k% + 1
        IF (k% > 3) = 0 THEN GOTO 10
    20 REM END DO

### For Transpilation

BASCAL emits native `FOR` / `NEXT`, which BASIC runtimes handle efficiently. The BASCAL `end for` (or bare `end`) is stripped; the BASIC `NEXT` is emitted by the transpiler:

    FOR i% = 1 TO 5
        PRINT STR$(i%) + "^2 = " + STR$(i% * i%)
    NEXT i%

### Function Transpilation

    ' value% -- number to constrain
    ' lo%    -- lower bound, inclusive
    ' hi%    -- upper bound, inclusive
    function clamp%(value%, lo%, hi%)
        return max%(lo%, min%(value%, hi%))
    end function

    result% = clamp%(15, 1, 10)

The calls to `max%` and `min%` inside `clamp%` are also transpiled to GOSUBs. The outermost call produces:

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

### Procedure Transpilation

Procedures follow the same GOSUB pattern as functions but have no result variable:

    ' label$ -- text shown before the score
    ' score% -- value to print
    procedure printScore(label$, score%)
        PRINT label$ + ": " + STR$(score%)
    end procedure

    printScore("Alice", 91)

Transpiles to:

    printscore_label$ = "Alice"
    printscore_score% = 91
    GOSUB 200
    ...
    END

    ' procedure printScore(label$, score%)
    200 PRINT (printscore_label$ + ": ") + STR$(printscore_score%)
        RETURN
    ' end procedure printScore

There is no `printscore_result` variable. A bare `return` inside a procedure transpiles to plain `RETURN`.

### Select Case Transpilation

`SELECT CASE` is transpiled to an `IF`/`GOTO` dispatch chain. The select expression is stored in a temporary variable (e.g., `BCCT1%`) to avoid re-evaluation.

### Exit Statements

`exit` is unqualified in BASCAL source; the transpiler picks the shape below based on which loop it's innermost inside:

- inside `for` → `EXIT FOR` (native FreeBASIC / QB extension)
- inside `while` → `GOTO end_label`
- inside `do` → `GOTO end_label`

</div>

[← Shared COMMON](shared-common.md) [Command-Line Reference →](command-line-reference.md)
