[Home](../index.md) / A built-in source formatter

<div id="formatter" class="section" markdown="1">

## `bcc --format`: one canonical style, enforced automatically

BASCAL ships its own source formatter, in the same spirit as `gofmt`: `bcc --format-check` reports every line that doesn't match `bcc`'s own canonical style without touching the file (and exits non-zero if it finds any — the shape a CI check wants), and `bcc --format` rewrites the file in place. Both are deterministic and idempotent — running `--format` twice never changes a file the second time — and both reach into a `require`d library's own declarations too, not just the file being formatted.

Every rule below is real: each comparison is an actual, verified `bcc --format` run — the left pane is what a file looked like beforehand (often ported from real BASIC, or just hand-edited over time), the right pane is `bcc --format`'s own output.

<div class="compare" markdown="1">

### 1. Indentation, spacing, and trailing whitespace

Recomputed from the real token stream, not guessed at from existing whitespace — so a comment or a string can never confuse it, and a run of extra spaces around an operator collapses to exactly one.

<div class="compare-grid" markdown="1">

<div class="pane old" markdown="1">

<span class="tag">Before</span>

```bascal
function f%()
if x% = 1 then
print   "one"
elseif x% = 2 then
print "two"   
else
print "other"
end if
return 0
end function
```

</div>

<div class="pane new" markdown="1">

<span class="tag">After `bcc --format`</span>

```bascal
function f%()
    if x% = 1 then
        print "one"
    elseif x% = 2 then
        print "two"
    else
        print "other"
    end if
    return 0
end function
```

</div>

</div>

</div>

<div class="compare" markdown="1">

### 2. Keyword and builtin casing, for a program ported from real BASIC

Every reserved word (`if`/`then`/`print`/`end function`/...) and every real BASIC intrinsic (`RND`, `LEN`, `STR$`, `MID$`, ...) is lowercased. Neither ever changes what the line parses as — BASIC keyword matching is already fully case-insensitive — only how it reads.

<div class="compare-grid" markdown="1">

<div class="pane old" markdown="1">

<span class="tag">Before (ported from BASIC)</span>

```bascal
PRINT "Adventure 3.2 for PyBASIC"
open "AMOVING" for INPUT as #4
IF RND(1) > 0.5 THEN goto L4360
```

</div>

<div class="pane new" markdown="1">

<span class="tag">After `bcc --format`</span>

```bascal
print "Adventure 3.2 for PyBASIC"
open "AMOVING" for input as #4
if rnd(1) > 0.5 then goto L4360
```

</div>

</div>

</div>

<div class="compare" markdown="1">

### 3. Declaration-matching casing, reaching into required libraries too

Every reference to a function, procedure, or variable is rewritten to match however that name's own declaration was written, file-wide — including a call site naming a function declared in a `require`d library, resolved the same way `bcc` itself resolves one.

<div class="compare-grid" markdown="1">

<div class="pane old" markdown="1">

<span class="tag">Before</span>

```bascal
program p
require com.bascal.sort.bubbleSort

BUBBLESORT%(bubbleData%)
```

</div>

<div class="pane new" markdown="1">

<span class="tag">After `bcc --format`</span>

```bascal
program p
require com.bascal.sort.bubbleSort

bubbleSort%(bubbleData%)
```

</div>

</div>

<div class="note" markdown="1">

`bubbleSort%` is declared as `function bubbleSort%(byref data%())` in the required library — the call site above is recased to match *that* declaration, not left alone just because it lives in a different file. A name that's also a record type name (`record Header` alongside a same-named `header` variable, legal since the two are different namespaces) is deliberately left alone rather than risk conflating the two.

</div>

</div>

<div class="compare" markdown="1">

### 4. Multi-statement line splitting

A physical line holding more than one top-level `:`-chained statement splits, one statement per line, at the same nesting level — no new block introduced.

<div class="compare-grid" markdown="1">

<div class="pane old" markdown="1">

<span class="tag">Before</span>

```bascal
fpos = 0 : fracnt = 0 : lastfra = -1
```

</div>

<div class="pane new" markdown="1">

<span class="tag">After `bcc --format`</span>

```bascal
fpos = 0
fracnt = 0
lastfra = -1
```

</div>

</div>

</div>

<div class="compare" markdown="1">

### 5. Single-line `if` explosion into block form

A single-line `if` (`if cond then stmt1 : stmt2`, real BASIC's own no-`end if` form) whose then- or else-body has more than one `:`-chained statement can't just split in place — it has no closing keyword for a multi-line body to end at — so it explodes into ordinary block form instead. A nested single-line `if` cascades too, and a trailing comment moves to the new `if ... then` header line, since it almost always describes the condition.

<div class="compare-grid" markdown="1">

<div class="pane old" markdown="1">

<span class="tag">Before</span>

```bascal
if l1 <> 50 then printMessage(2) : return 0
```

</div>

<div class="pane new" markdown="1">

<span class="tag">After `bcc --format`</span>

```bascal
if l1 <> 50 then
    printMessage(2)
    return 0
end if
```

</div>

</div>

</div>

<div class="compare" markdown="1">

### 6. `data` lines indented under their own label

A bare `label:` (nothing else on its own line) indents the `data` lines that immediately follow it one level deeper, matching its usual role as a `restore` target for a data table — grouping what belongs to it, for the same reason a `for`/`if` body is indented. That implicit block has no closing keyword of its own: it ends at the first line that isn't a `data` statement (or a comment about the table), or at a blank line.

<div class="compare-grid" markdown="1">

<div class="pane old" markdown="1">

<span class="tag">Before</span>

```bascal
function f%()
myTable:
data 1, 2, 3
data 4, 5, 6
print "after"
end function
```

</div>

<div class="pane new" markdown="1">

<span class="tag">After `bcc --format`</span>

```bascal
function f%()
    myTable:
        data 1, 2, 3
        data 4, 5, 6
    print "after"
end function
```

</div>

</div>

</div>

<div class="compare" markdown="1">

### 7. A `label: statement` line is never torn apart

Multi-statement splitting (rule 4) applies to the rest of a line, but a leading label always stays attached to whatever follows it — the common, deliberate `label: statement` idiom (a label and the one statement it guards, written on one line) is never split into a bare label line plus a separate statement line just because splitting logic is running at all.

<div class="compare-grid" markdown="1">

<div class="pane old" markdown="1">

<span class="tag">Before</span>

```bascal
function f%()
loopStart: x% = 1 : y% = 2
end function
```

</div>

<div class="pane new" markdown="1">

<span class="tag">After `bcc --format`</span>

```bascal
function f%()
    loopStart: x% = 1
    y% = 2
end function
```

</div>

</div>

</div>

This is deliberately as far as it goes: `bcc --format` never reflows an expression. See the [Command-Line Reference's Source Formatting section](../manual/command-line-reference.md#source-formatting) for the complete, exact rule set.

</div>

---

[← Previous: Record files](record-files.md) · [Next: Built-in linting →](linting.md)
