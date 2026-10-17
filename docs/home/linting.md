[Home](../index.md) / Built-in linting

<div id="linting" class="section" markdown="1">

## `bcc --lint`: five checks for things the compiler won't stop you on

Beyond `--strict-vars`'s mandatory-declaration check, `bcc --lint` runs five more warning-only passes against a program's own source (never a `require`d library's) and prints every finding to stderr. Unlike `check_legacy_forms` (always on, every compile), `--lint` is opt-in and never fails the build — these five are more heuristic, and more likely to flag something on existing, working code, especially a program ported from real BASIC.

Every comparison below is a real, verified `bcc --lint` run.

<div class="compare" markdown="1">

### 1. Unused declarations

A `dim`/`declare`/`const` never referenced again after its own declaration. A top-level declaration is checked against the whole program (every function can see it); a function-local one, only within that same function. A name only ever *assigned*, never read, still counts as used — this only catches a declaration nothing touches again at all.

<div class="compare-grid" markdown="1">

<div class="pane old" markdown="1">

<span class="tag">Input</span>

```bascal
function computeArea%(width%, height%)
    dim area%
    dim perimeter%
    area% = width% * height%
    return area%
end function
```

</div>

<div class="pane new" markdown="1">

<span class="tag">`bcc --lint` says</span>

```
warning: `perimeter%` is declared but never used
  --> unused.bcl:5:5
```

</div>

</div>

</div>

<div class="compare" markdown="1">

### 2. Shadowing

A local `dim`/`const` colliding with this function's own parameter, or with a name already meaningful at the whole-program level (a top-level `dim`/`const`, or a name any function promotes with `global`); a parameter colliding with such a global; or a `for` loop reusing a still-open enclosing `for` loop's own counter. All four are heuristic name-collision checks, not full scope analysis.

<div class="compare-grid" markdown="1">

<div class="pane old" markdown="1">

<span class="tag">Input</span>

```bascal
function setRoom%()
    global targetRoom%
    targetRoom% = 7
    return 1
end function

function performMove%(targetRoom%)
    global currentRoom%
    currentRoom% = targetRoom%
    return 1
end function
```

</div>

<div class="pane new" markdown="1">

<span class="tag">`bcc --lint` says</span>

```
warning: parameter `targetRoom%` of `performMove%`
shadows a global of the same name
  --> shadow.bcl:9:1
```

</div>

</div>

<div class="note" markdown="1">

`performMove%`'s own parameter never reads the global `targetRoom%` at all — it's a pure local value passed in by whoever calls it. The collision is a naming coincidence, but it's exactly the kind of thing that reads wrong to someone skimming the rest of the program, where `targetRoom%` already means something specific. Renaming the parameter (`destRoom%`, say) clears it.

</div>

</div>

<div class="compare" markdown="1">

### 3. A nested loop reusing its own outer counter

The fourth shadowing case, with its own example: a `for` loop variable reused while an identically-named outer `for` loop is still open silently overwrites the outer loop's own counter mid-iteration — a classic real bug, not just a style nit.

<div class="compare-grid" markdown="1">

<div class="pane old" markdown="1">

<span class="tag">Input</span>

```bascal
function sumGrid%()
    dim total%
    for row% = 1 to 5
        for row% = 1 to 5
            total% = total% + 1
        end for
    end for
    return total%
end function
```

</div>

<div class="pane new" markdown="1">

<span class="tag">`bcc --lint` says</span>

```
warning: `row%` shadows an already-open `for` loop
variable of the same name -- the inner loop silently
reuses the outer loop's own counter
  --> shadow2.bcl:6:9
```

</div>

</div>

</div>

<div class="compare" markdown="1">

### 4. Unreachable code

A statement after `return`/`goto`/`exit`/`throw`/`end`/`stop` in the same block, with no `label:` in between (a label might be jumped to from anywhere else in the program, so it always counts as reachable). Only the first unreachable statement in a run is reported; the rest follow from the same cause. This has no whole-program control-flow graph — it only ever looks at same-block, no-intervening-label fall-through.

<div class="compare-grid" markdown="1">

<div class="pane old" markdown="1">

<span class="tag">Input</span>

```bascal
function verbClose%()
    return 0
    print "never runs"
    return 1
end function
```

</div>

<div class="pane new" markdown="1">

<span class="tag">`bcc --lint` says</span>

```
warning: unreachable code: nothing before this
point in the same block can reach it
  --> unreach.bcl:5:5
```

</div>

</div>

<div class="note" markdown="1">

This one isn't always a bug to fix by deleting code — the [`adventure3000` case study](../examples/adventure3000.md)'s own `verbClose%` has exactly this shape on purpose, preserving a real, documented bug from the original 1979 game rather than "fixing" it. The check is still accurate (that code genuinely can't run); what to do about it is a judgment call `--lint` deliberately leaves to you.

</div>

</div>

<div class="compare" markdown="1">

### 5. Magic numbers

A bare numeric literal — other than `0`, `1`, or `-1`, exempted as near-universal sentinels (empty/none, first/last, true/false) — compared against something with `=`, `<>`, `<`, `<=`, `>`, or `>=`. The narrow definition: a comparison against a variable, not a loop bound or an array size, which are usually self-evidently structural rather than secretly meaningful.

<div class="compare-grid" markdown="1">

<div class="pane old" markdown="1">

<span class="tag">Input</span>

```bascal
function checkFlags%(blah%, zog%)
    if blah% = 34 and zog% = 65 then
        return 1
    end if
    return 0
end function
```

</div>

<div class="pane new" markdown="1">

<span class="tag">`bcc --lint` says</span>

```
warning: `34` is an unexplained numeric literal in
a comparison -- consider naming it with a `const`
  --> magic.bcl:4:5
warning: `65` is an unexplained numeric literal in
a comparison -- consider naming it with a `const`
  --> magic.bcl:4:5
```

</div>

</div>

<div class="note" markdown="1">

`--lint` only ever points out that a literal is unexplained — it has no idea `34` means "the room is locked." Naming it is still a judgment call for whoever reads the surrounding code (or, for the `adventure3000` case study, whoever can still read the original 1979 BASIC and its own `ADESCRIP`/`AITEMS`/`AMESSAGE` data files) — see the example below.

</div>

```bascal
const STATUS_ROOM_LOCKED = 34
const STATUS_ITEM_BURIED = 65

function checkFlags%(blah%, zog%)
    if blah% = STATUS_ROOM_LOCKED and zog% = STATUS_ITEM_BURIED then
        return 1
    end if
    return 0
end function
```

</div>

<div class="compare" markdown="1">

### 6. Constant naming convention

A top-level `const` whose name isn't uppercase snake case: it must start with an uppercase letter, contain only uppercase letters/digits/underscores, and have no doubled or trailing underscore. `MAX_COUNT` passes; `maxCount`, `Max_Count`, and `MAX__COUNT` don't.

<div class="compare-grid" markdown="1">

<div class="pane old" markdown="1">

<span class="tag">Input</span>

```bascal
const maxCount = 10
```

</div>

<div class="pane new" markdown="1">

<span class="tag">`bcc --lint` says</span>

```
warning: constant `maxCount` is not uppercase
snake case; use names such as `MAX_COUNT`
  --> constname.bcl:1:1
```

</div>

</div>

</div>

This is deliberately as far as it goes: none of the five checks ever fails the build, and `--lint` has no whole-program understanding of what a name is *supposed* to mean — only that a declaration went unused, a name collided, a line can't run, a literal has no explanation nearby, or a constant's name doesn't follow convention. The first four proved themselves on real code: applied to the `adventure3000` case study's own `stage17`, `--lint` found 6 genuine shadowing collisions and 156 unnamed room/item/message/direction codes across the whole file — including rediscovering a real, already-documented bug in the original 1979 game. See the [Command-Line Reference's Lints section](../manual/command-line-reference.md#lints) for the complete, exact rule set.

</div>

---

[← Previous: A built-in source formatter](formatter.md) · [Next: Tutorials and examples →](tutorials.md)
