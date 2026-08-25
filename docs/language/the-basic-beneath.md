## One source language, three targets

BASCAL has a complete `basic` backend, a mostly-complete `c` backend, and an early-stage `jvm` backend. The BASIC backend produces classic line-numbered Microsoft BASIC. The C backend produces C source for a native compiler; use `--target c` to select it. The JVM backend, `--target jvm`, produces [Krakatau](https://github.com/Storyyeller/Krakatau) assembly text, assembled into a `.class` and run on any JRE. It supports scalar values, arithmetic, strings and concatenation, comparisons, structured branches and loops, and numeric/string `SELECT CASE`; functions and most I/O remain in progress.

`basic` is the fallback target when no target is selected, but `BASCAL_TARGET` or the BASCAL configuration file can choose a different default. The C backend covers the tutorials and most of the core language, but it deliberately has limits; the JVM backend covers far less so far, being much newer. Consult the [backend reference](../manual/command-line-reference.md#backends) before relying on less common features or portability-sensitive behaviour on either.

Generated output is not a disguise. You can inspect it and compile it with the toolchain for its target. Even so, write structured BASCAL where possible. A block `if` is clearer than a hand-wired `GOTO` ladder because it keeps the program’s intent where you maintain it.

## See it happen

A block `if` and a `while` loop become the same branches and jumps you would write by hand in classic BASIC.

```bascal
if n% > 3 then
    print "big"
else
    print "small"
end if

while n% > 0
    print n%
    n% = n% - 1
end while
```

transpiles to:

```bascal
50 IF (n% > 3) = 0 THEN GOTO 80
60     PRINT "big"
70     GOTO 90
80     PRINT "small"
90 REM END IF

100 IF (n% > 0) = 0 THEN GOTO 140
110     PRINT n%
120     n% = n% - 1
130     GOTO 100
140 REM END WHILE
```

The `if` test jumps past the `then` block when false, and the `while` test is repeated at the top of the loop. The closing `REM` lines are comments that make the generated BASIC easier to read.

## When raw BASIC matters

Some classic statements remain available for programs that need BASIC-specific behaviour. Labels let `GOTO`, `GOSUB`, `RESTORE`, and BASIC-only `ON ERROR GOTO` recovery name a destination without exposing a numeric line number. The compiler assigns the number.

For error handling that must work with both backends, use structured `try`/`catch`/`finally` with `throw` or `throw n`. The C target rejects classic `on error goto`, `resume`, and `error`: their resumable BASIC control flow cannot cross ordinary C function calls safely. Use the structured form instead.

<div class="aside" markdown="1">

Think in BASCAL; inspect generated BASIC or C when target behaviour matters. Design in the source and use the target to check compatibility or reach platform-specific facilities.

</div>

## Licensing generated programs

BASCAL itself is licensed under the [GNU GPLv3](https://www.gnu.org/licenses/gpl-3.0.md). That licence applies to BASCAL, not automatically to programs it generates. You may distribute a BASCAL source program, its generated BASIC or C, and its compiled binary under any licence you choose.

When the C backend needs support functions, it emits them into the generated C source. The output exception also covers those functions: they do not place the final binary under the GPLv3 or limit the licence you choose for it. See the repository’s [output exception](https://github.com/johnjoeallen/bascal/blob/main/LICENSE-OUTPUT-EXCEPTION.md) for the exact permission.

## One complete book

This book is both an introduction and the authoritative BASCAL reference. Read it in order to learn the language, then return to the relevant chapter for its complete syntax, behaviour, and examples.
