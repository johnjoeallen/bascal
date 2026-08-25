<div id="why" class="section" markdown="1">

## Why BASCAL exists

BASCAL traces back to a preprocessor I wrote for a real BASIC shop in 1985, built to solve a real distribution problem: we had a shared set of library routines, and every change had to be merged by hand across a distributed dev team's copies, then merged again into the full application suite. That preprocessor supported directives like `@include`, `@if`, `@case`, `@function`, and `@procedure`, plus `{label}` in place of raw line numbers. I always wanted to build a proper modern equivalent — the tool I wish I'd had the skills and tools for back in 1985 — and building a real transpiler, rather than another preprocessor, was finally within reach with Claude and Codex doing the heavy lifting. BASCAL is that reconstruction in Rust, and the language itself is significantly more advanced than the original: more structured, easier to read, easier to write, and organized into reusable files. The idea hasn't changed, though — a real transpiler, not a text preprocessor, that still respects the classic BASIC it targets. The goal has stayed the same since 1985 — make BASIC more pleasant to write, without pretending it is a different runtime. Read the [full origin story](origin.md).

BASCAL keeps BASIC's global runtime model — functions transpile to `GOSUB` and generated storage is global — while giving source variables lexical scope. A variable used inside a function or procedure is local unless that body explicitly declares it with `global` to access a top-level variable.

BASCAL started as a strict superset of classic BASIC, and its `basic` target still is one — bitwise `AND`/`OR`/`NOT` and hand-written `OPEN`/`FIELD`/`GET`/`PUT` still pass through unchanged there. `GOTO`/`GOSUB` are raw BASIC too, but with one difference: BASCAL manages line numbering itself, so their targets are always a `name:` label declared in source, never a raw line number like `GOTO 140`. Beyond that, wherever BASCAL has its own construct for something, that construct is the canonical way to write it in `.bcl` source — treat the original BASIC syntax as what you're transpiling *away from*, not an equally-good alternative.

Compiling to more than one runtime came at a cost, though: `--target c` and `--target jvm` each have to drop a small, permanent set of raw-BASIC forms that don't translate safely onto a real C call stack or the JVM's own method model — BASCAL as a whole is now a **partial** superset, not a strict one. `--target basic` remains the most complete, closest-to-strict superset; see [Portability across backends](manual/command-line-reference.md#portability-across-backends) for exactly what each other backend gives up and its structured, portable equivalent.

In practice that means: write the pane on the right below, not the pane on the left.

<div class="compare" markdown="1">

<div class="compare-grid" markdown="1">

<div class="pane old" markdown="1">

<span class="tag">Classic MBASIC — avoid</span>

```bascal
100 IF grade% >= 90 THEN GOTO 140
110 IF grade% >= 80 THEN GOTO 160
120 IF grade% >= 70 THEN GOTO 180
130 GOTO 200
140 PRINT "A" : GOTO 210
160 PRINT "B" : GOTO 210
180 PRINT "C" : GOTO 210
200 PRINT "F"
210 REM ...
```

</div>

<div class="pane new" markdown="1">

<span class="tag">BASCAL — write this instead</span>

```bascal
select case grade%
case is >= 90
    print "A"
case is >= 80
    print "B"
case is >= 70
    print "C"
case else
    print "F"
end select
```

</div>

</div>

Both panes run on the same classic BASIC. `bcc` transpiles the pane on the right straight down to the pane on the left's shape — you just never have to number the branches or wire the `GOTO`s by hand. See [more comparisons below](#control-flow) for loops and short-circuit conditions.

</div>

</div>

<div id="features" class="section" markdown="1">

## What BASCAL adds over raw BASIC

<div class="feature-examples" markdown="1">

<div class="snippet" markdown="1">

### Structured control flow

Multiline `if`/`elseif`/`else`/`end if`, `for`/`end for`, `while`/`wend`, and pre-/post-check `do`/`loop` — no hand-numbered `GOTO`.

```bascal
if score% >= 90 then
    grade$ = "A"
elseif score% >= 80 then
    grade$ = "B"
else
    grade$ = "C"
end if

for i% = 1 to 5
    print i%
end for
```

</div>

<div class="snippet" markdown="1">

### Functions and procedures

`function` returns a typed value with explicit `return`; `procedure` is the side-effect-only equivalent. Both transpile to global parameter/result variables plus `GOSUB`.

```bascal
function max%(a%, b%)
    if a% > b% then
        return a%
    end if
    return b%
end function

top% = max%(4, 9)          ' 9
```

</div>

<div class="snippet" markdown="1">

### Path-style dependencies

`require`/`import` pull in another `.bcl` file's functions by dotted path — dots become directory separators, resolved against `-L` search paths.

```bascal
require stats

print mean!(scores%())
print maximum%(scores%())
```

</div>

<div class="snippet" markdown="1">

### SELECT CASE

Single values, comma lists, `low to high` ranges, and `is <op> value` comparisons — one dispatch chain instead of a hand-wired `IF`/`GOTO` ladder.

```bascal
select case grade%
case 90 to 100
    print "A"
case is >= 80
    print "B"
case else
    print "C"
end select
```

</div>

<div class="snippet" markdown="1">

### Shared COMMON

A shared file's variables are emitted identically into every `program ... shared ...` that references it, so a `CHAIN`ed program can't drift out of sync with the variable slots it's sharing.

`state.bcl` (shared file) — `shared <name>` plus `dim`; every variable listed is COMMON by default, no separate keyword needed:

```bascal
shared state

dim count%
dim label$
```

`show.bcl`:

```bascal
program show shared state

print "Count: " + STR$(count%)
end
```

</div>

<div class="snippet" markdown="1">

### Typed record/file I/O

Declare a fixed-layout record once; `file ... = open(...)` and `db[i] = { ... }` generate the matching `OPEN`/`FIELD`/`LSET`/`PUT` calls — see the full comparison below.

```bascal
record Student
    id:    int16
    name:  string(20)
    score: float64
end record

file db as Student = open("students.dat")
db[1] = { id: 1, name: "Alice", score: 95.0 }
```

</div>

<div class="snippet" markdown="1">

### Compound assignment, TRUE/FALSE, multi-name DIM

`+=`/`-=`/`*=`/`/=` desugar to `var = var op expr`; `TRUE`/`FALSE` are sugar for BASIC's own `-1`/`0`; a single `dim` can declare several names at once.

```bascal
dim total%, found%

total% += 10
found% = TRUE

if found% = TRUE then
    print total%
end if
```

</div>

<div class="snippet" markdown="1">

### Readable generated output

Source comments pass straight through, and only the lines a `GOTO`/`GOSUB` actually targets get a line number — everything else stays plain, hand-readable BASIC.

```bascal
' BASCAL source
' Track the high score for the level
if score% > highScore% then highScore% = score%

' generated .bas
' Track the high score for the level
IF (score% > highScore%) = 0 THEN GOTO 10
    highScore% = score%
10 REM END IF
```

</div>

</div>

</div>

<div id="control-flow" class="section" markdown="1">

## Control flow: structured vs hand-rolled GOTO

The line-numbered dialects `bcc` targets have no `SELECT CASE`, no `DO`/`LOOP`, and no multiline `IF` — every branch and loop is just `IF ... THEN GOTO` threaded by hand. The panes on the left below are what that threading looks like written directly; the panes on the right are what `bcc` lets you write instead. `bcc` still generates the GOTO chain — it just generates it correctly, every time, so you never have to count line numbers or trace jumps yourself. Full sources: [tutorial/06_select_case.bcl](https://github.com/johnjoeallen/bascal/blob/main/tutorial/06_select_case.bcl) and [tutorial/05_loops.bcl](https://github.com/johnjoeallen/bascal/blob/main/tutorial/05_loops.bcl).

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

This is what `bcc` emits for the pane on the right — see the generated `IF (BCC_T2% = 100) <> 0 THEN GOTO 250` dispatch chain in [tutorial/06_select_case.bas](https://github.com/johnjoeallen/bascal/blob/main/tutorial/06_select_case.bas). The only difference is that a transpiler numbered and wired every jump, instead of you.

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

Multiline `if`/`else if`/`else` is not an exception — it still transpiles to the nested `IF (cond) = 0 THEN GOTO ...` chain seen inside the `FOR` body above. See [If Transpilation](manual/generated-basic-shape.md#if-transpilation) in the manual for why BASCAL inverts with `= 0` instead of BASIC's bitwise `NOT`. The record/file syntax below follows that same GOTO-chain trade-off — it's just the newest place BASCAL makes it.

</div>

</div>

<div id="tutorial" class="section" markdown="1">

## Record files: BASCAL's record/file syntax vs standard BASIC

BASCAL supports classic random-access file I/O directly (`OPEN ... FOR RANDOM`, `FIELD`, `GET`/`PUT`, `LSET`, `MKx`/`CVx` all still pass through as-is), but writing it by hand is exactly the kind of repetitive, error-prone bookkeeping a transpiler should do instead. The `record`/`file` syntax below is the canonical way to do this in BASCAL: every pane on the right is what you write, and it **generates** the pane on the left — this syntax doesn't change what runs, only how much of it you have to type and keep in sync yourself. Full source: [tutorial/15_random_and_record_files.bcl](https://github.com/johnjoeallen/bascal/blob/main/tutorial/15_random_and_record_files.bcl).

<div class="compare" markdown="1">

### 1. Declare the record shape and open the file

BASCAL sums the field widths for you and generates the matching `FIELD` binding — get one field's width wrong by hand and every record after it gets read or written off by that many bytes.

<div class="compare-grid" markdown="1">

<div class="pane old" markdown="1">

<span class="tag">Generated BASIC</span>

```bascal
const rec_len%  = 30   ' 2+20+8, by hand
const db_file$  = "students.dat"

open db_file$ for random as #1 len = rec_len%
field #1, 2 as idBuf$, 20 as nameBuf$, 8 as scoreBuf$
```

</div>

<div class="pane new" markdown="1">

<span class="tag">BASCAL</span>

```bascal
record Student
    id:    int16
    name:  string(20)
    score: float64
end record

file db as Student = open("students.dat")
```

</div>

</div>

</div>

<div class="compare" markdown="1">

### 2. Write a whole record

Every declared field is required in the record literal — a field forgotten by hand ships silently; a field forgotten in `{ ... }` is a transpile error.

<div class="compare-grid" markdown="1">

<div class="pane old" markdown="1">

<span class="tag">Generated BASIC</span>

```bascal
lset idBuf$    = mki%(1)
lset nameBuf$  = "Alice"
lset scoreBuf$ = mkd#(95.0)
put #1, 1
```

</div>

<div class="pane new" markdown="1">

<span class="tag">BASCAL</span>

```bascal
db[1] = { id: 1, name: "Alice", score: 95.0 }
```

</div>

</div>

</div>

<div class="compare" markdown="1">

### 3. Read records back in reverse order

`downto` is sugar for `step -1`; `s.id`/`s.name`/`s.score` resolve straight to the unpacked scalars — no `cvi%`/`cvd#`/`rtrim$` to remember or mismatch.

<div class="compare-grid" markdown="1">

<div class="pane old" markdown="1">

<span class="tag">Generated BASIC</span>

```bascal
for i% = num_recs% to 1 step -1
    get #1, i%
    id%    = cvi%(idBuf$)
    score# = cvd#(scoreBuf$)
    print "[" + str$(id%) + "] " _
        + rtrim$(nameBuf$) + " -- " + str$(score#)
end for
```

</div>

<div class="pane new" markdown="1">

<span class="tag">BASCAL</span>

```bascal
for i = 3 downto 1
    let s = db[i]
    print "[" + s.id + "] " + s.name + " -- " + s.score
end for
```

</div>

</div>

</div>

<div class="compare" markdown="1">

### 4. Update a single field

Bob just scraped a pass on re-mark — only the score changes, but `PUT` always writes the whole buffer, so a `GET` has to come first either way.

<div class="compare-grid" markdown="1">

<div class="pane old" markdown="1">

<span class="tag">Generated BASIC</span>

```bascal
get #1, 2
lset scoreBuf$ = mkd#(61.5)
put #1, 2
```

</div>

<div class="pane new" markdown="1">

<span class="tag">BASCAL</span>

```bascal
db[2].score = 61.5
```

</div>

</div>

</div>

<div class="compare" markdown="1">

### 5. Update two fields in one shot

Alice got married and re-sat the exam. Whether the fields you *didn't* list need a `GET` first is decided at **transpile time** — the transpiler compares the field names you gave against the record's declared fields. Give it every field and the `GET` disappears entirely, same as a plain `{ ... }`. Misspell a field name and it's still a transpile error, not silent data loss.

<div class="compare-grid" markdown="1">

<div class="pane old" markdown="1">

<span class="tag">Generated BASIC</span>

```bascal
get #1, 1
lset nameBuf$  = "Alice Smith"
lset scoreBuf$ = mkd#(91.0)
put #1, 1
```

</div>

<div class="pane new" markdown="1">

<span class="tag">BASCAL</span>

```bascal
db[1] = ?{ name: "Alice Smith", score: 91.0 }
```

</div>

</div>

<div class="tally" markdown="1">

**Generated BASIC:** 4 lines, 3 buffer names, 1 pack call, repeated per edit **BASCAL:** 1 line, still exactly one `GET` + one `PUT` generated

</div>

> **Why `?{ ... }`?** A few spellings for "partial record literal" were considered — `.{ ... }`, `<-{ ... }` — but `?` won because it reads as "this might be incomplete," the same sense it carries for optional values in most other languages, and it was free: nothing else in BASCAL's grammar used a bare `?`, so adding it couldn't collide with or reinterpret any existing program. It's also deliberately a *second* spelling rather than a relaxed `{ ... }` — keeping `{ ... }` strict means a record literal that's missing a field by accident is still always a transpile error. `?{ ... }` exists so that incompleteness has to be opted into explicitly, one call site at a time, instead of silently allowed everywhere.

</div>

<div class="compare" markdown="1">

### 6. Batched update via read → mutate → write back

Same one-`GET`-one-`PUT` shape as `?{ ... }` above, spelled as read/mutate/write-back instead — useful when the new values aren't just one-line literals. `s.field = value` alone is pure in-memory assignment; nothing touches disk until the final write-back.

<div class="compare-grid" markdown="1">

<div class="pane old" markdown="1">

<span class="tag">Generated BASIC</span>

```bascal
get #1, 3
lset nameBuf$  = "Carol Jones"
lset scoreBuf$ = mkd#(88.0)
put #1, 3
```

</div>

<div class="pane new" markdown="1">

<span class="tag">BASCAL</span>

```bascal
let carol = db[3]
carol.name  = "Carol Jones"
carol.score = 88.0
db[3] = carol
```

</div>

</div>

</div>

<div class="compare" markdown="1">

### 7. Close

<div class="compare-grid" markdown="1">

<div class="pane old" markdown="1">

<span class="tag">Generated BASIC</span>

```bascal
close #1
```

</div>

<div class="pane new" markdown="1">

<span class="tag">BASCAL</span>

```bascal
db.close()
```

</div>

</div>

</div>

Nothing above runs slower — every line here transpiles to exactly the same `OPEN`/`FIELD`/`LSET`/`PUT`/`GET`/`MKx`/`CVx` calls shown on the left, generated for you instead of typed and kept in sync by hand. See the [Record Files section of the manual](manual/record-files.md) for the full semantics, including the exact static rule for when a partial write needs a `GET`.

</div>

<div id="more" class="section" markdown="1">

## Full tutorial list

Every tutorial below has its own page with a short walkthrough and a few snippets, plus a link to the real, transpiling `.bcl` source (and its generated `.bas` output) in the repo.

- [Hello world](tutorials/01_hello.md)
- [Variables](tutorials/02_variables.md)
- [Arithmetic](tutorials/03_arithmetic.md)
- [Conditions](tutorials/04_conditions.md)
- [Loops](tutorials/05_loops.md)
- [Select case](tutorials/06_select_case.md)
- [Functions](tutorials/07_functions.md)
- [Arrays](tutorials/08_arrays.md)
- [Data statements](tutorials/09_data.md)
- [File I/O](tutorials/10_files.md)
- [Screen control](tutorials/11_screen.md)
- [Require / dependencies](tutorials/12_require.md)
- [Shared COMMON](tutorials/13_shared.md)
- [Procedures](tutorials/14_procedures.md)
- [Random-access & record files](tutorials/15_random_and_record_files.md)
- [Short-circuit && and \|\|](tutorials/16_short_circuit.md)
- [Labels and error handling](tutorials/17_labels_and_error_handling.md)
- [Standard library functions](tutorials/18_stdlib.md)
- [Scalar methods](tutorials/20_methods.md)

[Open the tutorials index →](tutorials/)

</div>
