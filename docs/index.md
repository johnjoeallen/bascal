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

Both panes run on the same classic BASIC. `bcc` transpiles the pane on the right straight down to the pane on the left's shape — you just never have to number the branches or wire the `GOTO`s by hand. See the [structured control-flow chapter](home/control-flow.md) for loops and short-circuit conditions.

</div>

</div>
