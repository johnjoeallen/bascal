[Home](../../) / [Manual](../) / Generated BASIC Shape

[← Shared COMMON](shared-common.md) [Command-Line Reference →](command-line-reference.md)

<div class="prose" markdown="1">

Understanding how BASCAL transpiles its constructs helps when reading generated output or debugging.

### Header

Every generated file begins with:

```bascal
' BASCAL generated BASIC
' Functions are transpiled to global variables, labels, and GOSUB
```

### COMMON Block

If a shared file is referenced, `COMMON` lines appear before the header comment.

### Line Numbers

By default, `bcc` numbers every emitted line, not just branch targets, matching real MBASIC/BASCOM's own default: an unnumbered statement line is a syntax error unless the compiler is explicitly told to accept one (see below). Numbered comment-only lines are harmless either way.

Pass `--sparse-line-numbers` for the alternative: numbering only lines that are branch targets (destinations of `GOTO` or `GOSUB`) and leaving everything else unnumbered. This is more readable, and real MBASIC/BASCOM-family compilers can compile it too -- they just need a switch to accept unnumbered statement lines at all (Microsoft's BASCOM uses `/C`, IBM's BASIC Compiler uses `/N`); FreeBASIC's `-lang qb` accepts it with no switch at all. See the [Command-Line Reference](command-line-reference.md) for the exact flags.

The listings on this page are the compiler's own output, produced with `--sparse-line-numbers` so only branch targets carry line numbers. Comment lines that `bcc` copies through from the source are kept.

### If Transpilation

```bascal
if x% > 0 then
    PRINT "positive"
end if
```

Becomes:

```bascal
IF (x% > 0) = 0 THEN GOTO 10
    PRINT "positive"
10 REM END IF
```

The condition is inverted with `= 0` rather than `NOT` to avoid bitwise semantics (see [Operators](operators-and-expressions.md#operators-and-expressions)).

### While Transpilation

```bascal
p% = 1
while p% < 100
    PRINT STR$(p%)
    p% = p% * 2
end while
```

Becomes:

```basic
p% = 1
10 IF (p% < 100) = 0 THEN GOTO 20
    PRINT STR$(p%)
    p% = p% * 2
    GOTO 10
20 REM END WHILE
```

### Do Transpilation

```bascal
do while k% <= 3
    PRINT STR$(k%)
    k% = k% + 1
end do
```

Becomes:

```basic
10 IF (k% <= 3) = 0 THEN GOTO 30
    PRINT STR$(k%)
    k% = k% + 1
20 GOTO 10
30 REM END DO
```

The post-check form skips the leading guard entirely, since the body always runs at least once:

```bascal
do
    PRINT STR$(k%)
    k% = k% + 1
loop until k% > 3
```

Becomes:

```basic
10 PRINT STR$(k%)
    k% = k% + 1
20 IF (k% > 3) = 0 THEN GOTO 10
30 REM END DO
```

### For Transpilation

BASCAL emits native `FOR` / `NEXT`, which BASIC runtimes handle efficiently. The BASCAL `end for` (or bare `end`) is stripped; the BASIC `NEXT` takes its place.

```bascal
for i% = 1 to 5
    print str$(i%) + "^2 = " + str$(i% * i%)
end for
```

<!-- generated-basic -->
```basic
FOR i% = 1 TO 5
    PRINT (STR$(i%) + "^2 = ") + STR$(i% * i%)
10 NEXT i%
```

### Function Transpilation

A function becomes a `GOSUB` target. Its parameters and result are ordinary global variables named after the function, so a call assigns the arguments, runs `GOSUB`, and reads the result variable:

```bascal
' value% -- number to constrain
' lo%    -- lower bound, inclusive
' hi%    -- upper bound, inclusive
function clamp%(value%, lo%, hi%)
    if value% < lo% then
        return lo%
    end if
    if value% > hi% then
        return hi%
    end if
    return value%
end function

result% = clamp%(15, 1, 10)
print result%
```

The compiler produces:

<!-- generated-basic -->
```basic
' value% -- number to constrain
' lo%    -- lower bound, inclusive
' hi%    -- upper bound, inclusive
clampValue0% = 15
clampLo0% = 1
clampHi0% = 10
GOSUB 10
result% = clampResult0%
PRINT result%
END
' function clamp%(value%, lo%, hi%)
10 IF (clampValue0% < clampLo0%) = 0 THEN GOTO 20
        clampResult0% = clampLo0%
        RETURN
20 REM END IF
    IF (clampValue0% > clampHi0%) = 0 THEN GOTO 30
        clampResult0% = clampHi0%
        RETURN
30 REM END IF
    clampResult0% = clampValue0%
    RETURN
' end function clamp%
```

### Procedure Transpilation

Procedures follow the same GOSUB pattern as functions but have no result variable:

```bascal
' label$ -- text shown before the score
' score% -- value to print
procedure printScore(label$, score%)
    print label$ + ": " + str$(score%)
end procedure

printScore("Alice", 91)
```

Transpiles to:

<!-- generated-basic -->
```basic
' label$ -- text shown before the score
' score% -- value to print
printscoreLabel0$ = "Alice"
printscoreScore0% = 91
GOSUB 10
END
' procedure printscore(label$, score%)
10 PRINT (printscoreLabel0$ + ": ") + STR$(printscoreScore0%)
    RETURN
' end procedure printscore
```

A procedure has no result variable, and a bare `return` inside one transpiles to plain `RETURN`.
### Select Case Transpilation

`SELECT CASE` is transpiled to an `IF`/`GOTO` dispatch chain. The select expression is stored in a temporary variable (e.g., `BCCT1%`) to avoid re-evaluation.

### Exit Statements

`exit` is unqualified in BASCAL source; the transpiler picks the shape below based on which loop it's innermost inside:

- inside `for` → `EXIT FOR` (native FreeBASIC / QB extension)
- inside `while` → `GOTO end_label`
- inside `do` → `GOTO end_label`

### Continue Statements

`continue` is unqualified too, resolved the same way as `exit`, but always to a `GOTO` — never `EXIT FOR`, since that ends the loop rather than skipping to its next iteration:

- inside `for` → `GOTO continue_label`, a label placed right before `NEXT` (BASIC has no native "skip to the increment", the way it has `EXIT FOR`)
- inside `while` → `GOTO top_label` — the same label the loop's own back-edge already targets, since re-checking the condition *is* a `while` loop's own per-iteration bookkeeping
- inside `do` with no *post*-condition → `GOTO top_label`, same reasoning as `while`
- inside `do` with a post-condition (`loop while`/`loop until`) → `GOTO continue_label`, a label placed right after the body, before that post-condition check — targeting `top_label` instead would skip the post-condition check on every `continue`, turning the loop into an infinite one

</div>

[← Shared COMMON](shared-common.md) [Command-Line Reference →](command-line-reference.md)
