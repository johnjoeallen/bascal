[Home](../) / [Manual](../manual/) / Dependencies — REQUIRE and IMPORT

[← Miscellaneous Statements](miscellaneous-statements.md) [Shared COMMON →](shared-common.md)

<div class="prose" markdown="1">

BASCAL supports multi-file projects through `require` (and its alias `import`). Dependencies are declared at the top of the file, before any statements.

From `tutorial/12_require.bcl` — a program that uses a statistics library:

    require stats

    CONST N% = 8
    DIM scores%(N%)

    scores%(0) = 74 : scores%(1) = 91 : scores%(2) = 63 : scores%(3) = 88
    scores%(4) = 55 : scores%(5) = 97 : scores%(6) = 72 : scores%(7) = 84

    PRINT "Mean:   " + STR$(mean!(scores%()))
    PRINT "Max:    " + STR$(maximum%(scores%()))
    PRINT "Min:    " + STR$(minimum%(scores%()))
    PRINT "Range:  " + STR$(rangeOf%(scores%()))
    END

Transpile with `-L tutorial/lib` so that `require stats` resolves to `tutorial/lib/stats.bcl`:

    bcc tutorial/12_require.bcl -L tutorial/lib

`tutorial/lib/stats.bcl` defines `mean!`, `maximum%`, `minimum%`, and `rangeOf%` — all merged into the single generated `.bas` output.

### Path Resolution

The dot-separated path is converted to a file path by replacing each `.` with a directory separator and appending `.bcl`:

    require com.bascal.sort.bubbleSort  →  com/bascal/sort/bubbleSort.bcl
    require stats                       →  stats.bcl

The transpiler searches for the file in: 1. The directory containing the current source file 2. Additional directories supplied with `-L` flags (in order)

Dependencies are resolved recursively. A file is loaded at most once per compilation (circular dependencies are silently ignored after the first load).

### Function Merging

All functions from a required file (and its transitive dependencies) are merged into the generated output. Duplicate function names are rejected with a diagnostic error.

### Module Conventions

Every file loaded via `require`/`import` **must** start with a [`library <name>` declaration](program-structure.md#library-declaration) — a transpile-time error, not just a convention. Beyond that required header, by convention a library module should: - Contain only `function` definitions and supporting `DIM` / `DATA` statements - Not contain a `program` declaration - Not contain top-level executable statements other than `DIM` and `DATA`

</div>

[← Miscellaneous Statements](miscellaneous-statements.md) [Shared COMMON →](shared-common.md)
