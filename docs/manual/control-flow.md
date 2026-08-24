[Home](../) / [Manual](../manual/) / Control Flow

[← Comments](comments.md) [Functions →](functions.md)

<div class="prose" markdown="1">

### IF / ELSEIF / ELSE / END IF

```bascal
if condition then
    ' then body
end if

if condition then
    ' then body
else
    ' else body
end if
```

BASCAL also supports classic BASIC's single-line form: a statement directly after `then`, on the same line, needs no `end if`.

```bascal
if condition then statement
if condition then statement else statement
```

A newline right after `then` is what selects the block form above instead — that's the only difference between the two (BASCAL is line-oriented with no line-continuation syntax; see [File Encoding](program-structure.md#file-encoding)). The single-line form may chain multiple statements with `:`, same as anywhere else in BASCAL, and its `else` (if any) must be on that same line too:

```bascal
if x% > 0 then print "positive"
if x% > 100 then print "big" else print "small"
if x% > 0 then y% = 1: z% = 2
```

`elseif` isn't available in the single-line form — same as classic BASIC, it needs the block form above.

From `tutorial/04_conditions.bcl` — a grade classification chain:

```bascal
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

```bascal
for var = start to end [step n]
    ' body
end for
```

`end for` closes the loop. Bare `end` also works. The `step` clause is optional; the default step is 1.

From `tutorial/05_loops.bcl`:

```bascal
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

`exit` exits the enclosing loop immediately. It's unqualified — not `exit for`/`exit while`/`exit do` — the transpiler already knows which loop it's inside; see [Exit](#exit) below.

### WHILE / END WHILE

```bascal
while condition
    ' body
end while
```

`end while` closes the loop. Bare `end` also works, and so does classic BASIC's own `wend`.

From `tutorial/05_loops.bcl`:

```bascal
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

```bascal
do [while/until condition]
    ' body
end do
```

or the post-check form:

```bascal
do
    ' body
loop [while/until condition]
```

`end do` (bare `end` also works) closes a **pre-check** loop: the optional `while`/`until` clause tests the condition *before* each iteration, so the body may run zero times. `loop [while/until condition]` closes a **post-check** loop instead: the condition is tested *after* the body runs, so the body always runs at least once — the direct BASCAL equivalent of what other languages spell `repeat`/`until`. A bare `do ... loop` with no condition on either end is a plain infinite loop, same as bare `do ... end do`; both need `exit` to terminate.

From `tutorial/05_loops.bcl`:

```bascal
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

`exit` exits the enclosing `do` loop immediately, from either the pre-check or post-check form; see [Exit](#exit) below.

### Exit

```bascal
exit
```

`for`, `while`, and `do` share one early-exit statement: unqualified `exit`, with no loop-type keyword after it. The transpiler resolves which enclosing loop it leaves from context — the *innermost* one, if loops are nested — so `exit` inside a `do` loop transpiles to a `GOTO` past the loop's own end label, while `exit` inside a `for` loop transpiles to BASIC's native `EXIT FOR` instead, since `for`/`next` transpiles to a real `FOR ... NEXT` block rather than a `GOTO` chain (see [For Transpilation](generated-basic-shape.md#for-transpilation)).

```bascal
for i% = 1 to 5
    do
        if i% = 3 then
            exit          ' leaves the do, not the for
        end if
    end do
end for
```

`exit do`, `exit for`, and `exit while` are not valid — a loop-type keyword after `exit` is a transpile-time error.

### SELECT CASE

```bascal
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

The `select case` expression is evaluated once. Cases are tested in order. `case else` is optional and must be the last clause.

From `tutorial/06_select_case.bcl`:

```bascal
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

| Form               | Matches when                  |
|--------------------|-------------------------------|
| `case value`       | expression = value            |
| `case v1, v2, v3`  | expression = any listed value |
| `case low to high` | low ≤ expression ≤ high       |
| `case is = value`  | expression = value            |
| `case is <> value` | expression ≠ value            |
| `case is < value`  | expression \< value           |
| `case is <= value` | expression ≤ value            |
| `case is > value`  | expression \> value           |
| `case is >= value` | expression ≥ value            |

### Short-Circuit && and \|\|

```bascal
if a% > 0 && b% > 0 then
    ' body only runs if BOTH are true -- b% > 0 is never evaluated
    ' unless a% > 0 already passed
end if

do until done% || attempts% >= max_attempts%
    ...
end do
```

Unlike `AND`/`OR` (bitwise, always evaluate both sides — see [Logical Operators](operators-and-expressions.md#operators-and-expressions)), `&&` and `||` are true short-circuit operators: `a% > 0 && f%()` never calls `f%()` when `a% > 0` is already false, and `a% > 0 || f%()` never calls `f%()` when `a% > 0` is already true.

`&&`/`||` are only legal directly in the condition of `if`/`elseif`/ `while`/`do [while/until]` — not as a general expression (can't be assigned to a variable, passed as a function argument, etc.). A condition may chain any number of the *same* operator (`a && b && c`); mixing `&&` and `||` in one condition is a transpile-time error — split into nested `if` statements instead.

From `tutorial/16_short_circuit.bcl`, an `&&` guard transpiles to one guarded `IF` per operand — no bitwise `AND`, no wasted call:

```bascal
if ptr% >= 0 && isPositive%(scores%(ptr%)) > 0 then
    print "safe to read"
end if

IF (ptr% >= 0) = 0 THEN GOTO 10
ispositiveN0% = scores%(ptr%)
GOSUB 20
IF (ispositiveResult0% > 0) = 0 THEN GOTO 10
    PRINT "safe to read"
10 REM END IF
```

`||` needs one extra label, since a *chain* has to keep checking until either an operand proves it true or every operand has been tried:

```bascal
if a% = 1 || a% = 2 then
    print "one or two"
end if

IF (a% = 1) <> 0 THEN GOTO 10
IF (a% = 2) <> 0 THEN GOTO 10
GOTO 20
10     PRINT "one or two"
20 REM END IF
```

`do until`/`do while`'s inverted polarity applies the same duality: a `do until a% && b%` needs the extra label (mirroring a plain `||`), while a `do until a% || b%` doesn't (mirroring a plain `&&`) — the transpiler works this out per condition; it isn't something you need to reason about yourself.

</div>

[← Comments](comments.md) [Functions →](functions.md)
