[Home](../index.md) / What BASCAL adds

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

`function` returns a typed value with explicit `return`; `procedure` is the side-effect-only equivalent. On the `basic` target they transpile to global parameter/result variables plus `GOSUB`; the C and JVM targets use real callable methods, so `GOSUB` remains a BASIC-only compatibility form for portable source.

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

Declare a fixed-layout record once; `file ... = open(...)` and `db[i] = { ... }` generate the matching `OPEN`/`FIELD`/`LSET`/`PUT` calls — see the full comparison below. Records used only in memory may also contain plain variable-length `string` members, but those records cannot be used as random-access file types.

```bascal
record Student
    id:    int16
    name:  string(20) lpad
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

---

[← Previous: Why BASCAL exists](../index.md) · [Next: Structured control flow →](control-flow.md)
