[Home](../index.md) / Structured control flow

<div id="control-flow" class="section" markdown="1">

## Control flow: structured vs hand-rolled GOTO

The line-numbered dialects `bcc` targets have no `SELECT CASE`, no `DO`/`LOOP`, and no multiline `IF` — every branch and loop is just `IF ... THEN GOTO` threaded by hand. The panes on the left below are what that threading looks like written directly; the panes on the right are what `bcc` lets you write instead. `bcc` still generates the GOTO chain — it just generates it correctly, every time, so you never have to count line numbers or trace jumps yourself. Full sources: [tutorial/select_case.bcl](https://github.com/johnjoeallen/bascal/blob/main/tutorial/select_case.bcl) and [tutorial/loops.bcl](https://github.com/johnjoeallen/bascal/blob/main/tutorial/loops.bcl).

<div class="compare" markdown="1">

### 1. SELECT CASE

Six clauses mean six branch targets and a fall-through target to keep numbered and pointed at the right place by hand — miswire one `GOTO` and a case silently falls into the wrong branch.

<div class="compare-grid" markdown="1">

<div class="pane old" markdown="1">

<span class="tag">Generated BASIC</span>

```bascal
score% = 85
IF score% = 100 THEN GOTO 250
IF score% >= 90 AND score% <= 99 THEN GOTO 270
IF score% >= 80 AND score% <= 89 THEN GOTO 290
IF score% >= 70 AND score% <= 79 THEN GOTO 310
IF score% >= 60 AND score% <= 69 THEN GOTO 330
IF score% >= 0 THEN GOTO 350
GOTO 370
250 PRINT "Perfect!" : GOTO 380
270 PRINT "A -- Excellent" : GOTO 380
290 PRINT "B -- Good" : GOTO 380
310 PRINT "C -- Satisfactory" : GOTO 380
330 PRINT "D -- Passing" : GOTO 380
350 PRINT "F -- Fail" : GOTO 380
370 PRINT "Invalid score"
380 REM ...
```

</div>

<div class="pane new" markdown="1">

<span class="tag">BASCAL</span>

```bascal
select case score%
    case 100
        print "Perfect!"
    case 90 to 99
        print "A -- Excellent"
    case 80 to 89
        print "B -- Good"
    case 70 to 79
        print "C -- Satisfactory"
    case 60 to 69
        print "D -- Passing"
    case is >= 0
        print "F -- Fail"
    case else
        print "Invalid score"
end select
```

</div>

</div>

This is what `bcc` emits for the pane on the right — see the generated `IF (BCC_T2% = 100) <> 0 THEN GOTO 250` dispatch chain in [tutorial/select_case.bas](https://github.com/johnjoeallen/bascal/blob/main/tutorial/select_case.bas). The only difference is that a transpiler numbered and wired every jump, instead of you.

</div>

<div class="compare" markdown="1">

### 2. WHILE / WEND

A pre-check loop is one `IF ... THEN GOTO` guarding the top and one `GOTO` looping back at the bottom — both have to name the same line number.

<div class="compare-grid" markdown="1">

<div class="pane old" markdown="1">

<span class="tag">Generated BASIC</span>

```bascal
p% = 1
400 IF p% >= 100 THEN GOTO 440
    PRINT "  "; p%
    p% = p% * 2
    GOTO 400
440 REM ...
```

</div>

<div class="pane new" markdown="1">

<span class="tag">BASCAL</span>

```bascal
p% = 1
while p% < 100
    print "  "; p%
    p% = p% * 2
wend
```

</div>

</div>

`end while` and bare `end` close a `while` loop too — `wend` is classic BASIC's own spelling, accepted alongside them.

</div>

<div class="compare" markdown="1">

### 3. DO WHILE / DO UNTIL

Same pre-check shape as `WHILE`, spelled two ways depending on whether the loop-continues condition reads more naturally as a "while" or an "until".

<div class="compare-grid" markdown="1">

<div class="pane old" markdown="1">

<span class="tag">Generated BASIC</span>

```bascal
k% = 1
670 IF k% > 3 THEN GOTO 700
    PRINT "  "; k%
    k% = k% + 1
    GOTO 670
700 REM ...

k% = 1
740 IF k% <= 3 THEN GOTO 770
    PRINT "  "; k%
    k% = k% + 1
    GOTO 740
770 REM ...
```

</div>

<div class="pane new" markdown="1">

<span class="tag">BASCAL</span>

```bascal
k% = 1
do while k% <= 3
    print "  "; k%
    k% = k% + 1
end do

k% = 1
do until k% > 3
    print "  "; k%
    k% = k% + 1
end do
```

</div>

</div>

</div>

<div class="compare" markdown="1">

### 4. Post-check loop ("repeat until") and a mid-loop exit

Classic BASIC has no `REPEAT`/`UNTIL` — a loop that must run its body at least once, or that needs to break out from the *middle* rather than the top, means the continuation test and the jump can't share one tidy spot the way `WHILE` can. BASCAL's `do ... loop until/while` is the direct `REPEAT`/`UNTIL` equivalent, in both polarities — same two spellings as `do while`/`do until` above, just tested at the bottom instead of the top, so the body always runs at least once. A true middle exit still needs `exit`, since no fixed top-or-bottom condition position can express "stop right here."

<div class="compare-grid" markdown="1">

<div class="pane old" markdown="1">

<span class="tag">Generated BASIC</span>

```bascal
' body always runs at least once
k% = 99
810 PRINT "  "; k%
    k% = k% + 1
    IF k% <= 3 THEN GOTO 810

j% = 1
910 PRINT "  "; j%
    j% = j% + 1
    IF j% <= 3 THEN GOTO 910

' exit from the middle, at k% = 3
k% = 1
1010 IF k% = 3 THEN GOTO 1070
    PRINT "  "; k%
    k% = k% + 1
    GOTO 1010
1070 REM ...
```

</div>

<div class="pane new" markdown="1">

<span class="tag">BASCAL</span>

```bascal
' body always runs at least once
k% = 99
do
    print "  "; k%
    k% = k% + 1
loop until k% > 3

j% = 1
do
    print "  "; j%
    j% = j% + 1
loop while j% <= 3

' exit from the middle, at k% = 3
k% = 1
do
    if k% = 3 then
        exit
    end if
    print "  "; k%
    k% = k% + 1
end do
```

</div>

</div>

</div>

<div class="compare" markdown="1">

### 5. Short-circuit `&&` / `||`

Classic BASIC's `AND`/`OR` are bitwise and always evaluate both sides — there's no short-circuit primitive in the target language at all. Getting the same safety by hand means nesting an `IF` per operand, so the guarded call is textually inside the guard. BASCAL's `&&`/`||` are real short-circuit operators, restricted to `if`/`elseif`/`while`/`do` conditions, that transpile straight to that same nested-`IF` shape — one guarded jump per operand, in order, each operand's own setup code emitted right before its guard so a later operand's side effects don't run once an earlier one already decided the answer.

<div class="compare-grid" markdown="1">

<div class="pane old" markdown="1">

<span class="tag">Generated BASIC</span>

```bascal
IF (ptr% >= 0) = 0 THEN GOTO 10
ispositive_n_0% = scores%(ptr%)
GOSUB 20
IF (ispositive_result_0% > 0) = 0 THEN GOTO 10
    PRINT "safe to read"
10 REM END IF
```

</div>

<div class="pane new" markdown="1">

<span class="tag">BASCAL</span>

```bascal
if ptr% >= 0 && isPositive%(scores%(ptr%)) > 0 then
    print "safe to read"
end if
```

</div>

</div>

A `||` chain (or an inverted `do until ... && ...`) needs exactly one extra local label — the point where the chain can stop early once *any* operand has already decided the answer — worked out by the transpiler per condition, not something to reason about by hand.

</div>

<div class="compare" markdown="1">

### 6. Labels instead of raw line numbers

Raw BASIC/BASCOM GOTO/GOSUB targets are line numbers you have to keep in sync by hand — renumber a block and every reference to it has to be found and fixed too. BASCAL doesn't let `.bcl` source target a line number at all: `goto`/`gosub`/`on error goto`/`resume` all require a `name:` label instead, and the transpiler is the one that assigns the real number when it renders output — the same job it already does for every `if`/`while`/`do`/`select case` branch target above.

<div class="compare-grid" markdown="1">

<div class="pane old" markdown="1">

<span class="tag">Generated BASIC</span>

```bascal
ON ERROR GOTO 10
OPEN filename$ FOR INPUT AS #1
CLOSE #1
ON ERROR GOTO 0
GOTO 30

10 IF (err = 53) = 0 THEN GOTO 20
    PRINT "file not found"
    RESUME NEXT
20 REM END IF

30 END
```

</div>

<div class="pane new" markdown="1">

<span class="tag">BASCAL</span>

```bascal
on error goto handleErr
open fileName$ for input as #1
close #1
on error goto 0
goto done

handleErr:
if err = 53 then
    print "file not found"
    resume next
end if

done:
end
```

</div>

</div>

`on error goto 0` is the one numeric exception — it isn't a line number, it's the sentinel that disables the error trap. Everywhere else, a raw number in a branch-target position is a transpile-time error.

</div>

<div class="compare" markdown="1">

### The one exception: `for`/`next`

Every loop above transpiles to a guarded `GOTO` chain with a local label. `for`/`next` doesn't need that trick: BASIC already has a native counted loop, so BASCAL transpiles straight to it. `exit` is spelled the same everywhere in BASCAL source — no `exit for`, just `exit` — but here it becomes BASIC's own `EXIT FOR` instead of a `GOTO`, because the transpiler knows this `exit` is inside a native `FOR`, not a GOTO-chain loop.

<div class="compare-grid" markdown="1">

<div class="pane new" markdown="1">

<span class="tag">BASCAL</span>

```bascal
for i% = 1 to 20
    if i% > 4 then
        print i%
        exit
    end if
end for
```

</div>

<div class="pane old" markdown="1">

<span class="tag">Generated BASIC</span>

```bascal
FOR i% = 1 TO 20
    IF (i% > 4) = 0 THEN GOTO 10
        PRINT i%
        EXIT FOR
10 REM END IF
NEXT i%
```

</div>

</div>

Multiline `if`/`else if`/`else` is not an exception — it still transpiles to the nested `IF (cond) = 0 THEN GOTO ...` chain seen inside the `FOR` body above. See [If Transpilation](../manual/generated-basic-shape.md#if-transpilation) in the manual for why BASCAL inverts with `= 0` instead of BASIC's bitwise `NOT`. The record/file syntax below follows that same GOTO-chain trade-off — it's just the newest place BASCAL makes it.

</div>

</div>
