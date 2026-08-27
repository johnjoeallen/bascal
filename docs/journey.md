[Home](../) / Development Journey

<div class="prose" markdown="1">

# Development Journey

This is the project's technical history, reconstructed from its commits. It
follows the changes that altered what BASCAL can express, transpile, run, or
verify. Release and tagging commits are deliberately omitted.

## June: a structured language for classic BASIC

BASCAL began on 9 June 2026 as a compiler that turns `.bcl` files into
line-numbered BASIC. The first work established the basic project layout,
code generation, tutorial programs, and the convention that generated files
sit beside their source. It quickly gained block and line comments,
procedures, functions, `COMMON`-style shared source, and a manual built from
working examples (`e26bab5`–`c5eaaaf`).

The early language design was deliberately about making classic BASIC more
manageable rather than hiding it. Loop endings moved to the consistent
`end while`, `end for`, and `end do` form. Routine-local variables became
local by default, backed by a callable symbol table and compiler-generated
names rather than naming conventions (`c634a5f`–`33087e0`). Symbols and BASIC
builtins became case-insensitive, as BASIC programmers expect.

Classic-BASIC coverage expanded rapidly: random-access files, formatted
output, bitwise and arithmetic operators, hex and octal literals, error
handling, multidimensional arrays, `OPTION BASE`, and a range of statements
and builtins all became part of the BASIC backend (`3c99466`–`ba89fde`). The
result was already a practical structured source language that still emitted
the conservative BASIC shape required by old toolchains.

## Early August: records, source structure, and a public manual

After a pause, development resumed with a record/file syntax over
random-access BASIC files. A record declaration could describe a layout, and
field updates could be batched and explicitly written back. Partial record
updates, written `?{ ... }`, made it possible to express an update without
rewriting every field (`41c013a`–`8e6cadc`).

The documentation then became a site rather than only repository files. It
gained side-by-side BASCAL and generated-BASIC comparisons, the full manual,
and tutorials. This was also when the project's vocabulary was made
consistent: BASCAL *transpiles* source; it does not “lower” it
(`e4a7417`–`126cfe7`).

Language work continued alongside the documentation: short-circuit `&&` and
`||`, named labels for `goto`/`gosub` rather than raw line-number targets,
post-condition loops, single-line `if`, compound assignments, Boolean
literals, and multi-name `dim` all arrived. Shared source files were
formalized first as `suite` files, then as the current `shared` form
(`13c26c1`–`5491670`).

## Mid-August: routines and arrays became explicit

The next major task was making routines reliable on a BASIC backend without a
real call stack. Parameters gained explicit `byval` and `byref` semantics;
multi-dimensional array parameters were supported; and a `global` declaration
that accidentally shadowed a parameter became a transpile-time error
(`d96fe88`). Array rank moved into routine signatures using `arr%(?)` and
`grid%(?, ?)` rather than being inferred only from the routine body
(`fda32cd`).

That exposed the constraints of classic BASIC more clearly. Recursive calls
would overwrite shared generated parameter storage, so indirect as well as
direct recursion is rejected during transpilation (`e389d3e`). `sizeof()` was
added for array bounds, followed by automatic propagation of array bounds at
calls. Array parameter storage is now dimensioned once and given a capacity
resolved from its call sites, rather than being dimensioned repeatedly at
runtime (`8478b0d`, `afc6b46`, `763eecf`).

The same period tightened BASIC compatibility. The compiler gained a
dosbox-x conformance suite against IBM BASIC Compiler 2.00, and the backend
was corrected for real-BASCOM naming, constants, conversion and string
functions, line numbering, `MID$` assignment, and library helpers
(`9da3b49`–`248dbb7`). Top-level constants and several BASIC pseudo-variables
were fixed inside routines. Files also received an explicit role:
`program`, `library`, or `shared` (`0be65b0`–`5491670`).

## 21–24 August: BASCAL became multi-target

The C backend began as an opt-in proof of concept that could print literal
expressions (`5688f1f`). In the following days it grew into a native backend
covering scalar variables, strings, arithmetic, comparisons, structured
control flow, `select case`, functions, procedures, record files, sequential
files, interactive input, screen operations, standard-library math and string
functions, labels, and `gosub` (`d887498`–`a509736`).

Supporting C forced the front end to become clearer about what belongs to the
language and what belongs only to classic BASIC. The command line gained a
proper target selection and run/output options. Strict variable declarations,
`declare`, and builtin-shadow checks were added. Array parameters, local
arrays, runtime-computed bounds, scalar extension methods, `lbound()` and
`ubound()` were implemented across the relevant targets (`cccf7e1`–
`b119268`). Legacy BASIC forms with structured BASCAL equivalents began to
produce guidance rather than being treated as interchangeable source styles.

Structured error handling was the other large change in this period.
`try`, `catch`, `throw`, `finally`, catch filters, named error constants, and
the source filename made error handling portable across BASIC and C. The C
backend propagates raises through functions and procedures, and file failures
can be caught rather than terminating unconditionally (`11f7e36`–`eaaaf49`).
Classic `on error goto` remains supported where the target can represent it,
but portable code has a structured alternative.

At the same time, the documentation moved to MkDocs, regained the language
book presentation, and began embedding generated C beside generated BASIC.
That made the multi-target nature of the language visible in its examples
(`052e5fa`–`c850735`).

## 25 August: a JVM backend and typed IR

The JVM target began with target dispatch, Krakatau assembly for a minimal
program, and `--run`/binary integration (`e5ce649`–`74127ef`). It then gained
control flow, labels, scalar functions and procedures, scalar methods,
standard-library functions, terminal operations, and output. JVM limitations
were documented as they appeared rather than being hidden behind the BASIC
compatibility story.

Arrays made the need for a genuinely typed intermediate representation
unavoidable. The compiler now records typed array declarations and references
during parsing, preserves them while merging source files, and carries them
to JVM code generation. That enabled rank-aware allocation, multidimensional
access, array size expressions, and array parameters without asking the JVM
backend to infer types from source syntax or generated output
(`85ddd82`–`e0aa6f3`). The same approach was extended to non-integer arrays
and per-call bounds. This was an architectural step as much as a feature:
backends consume resolved type information.

JVM support also gained initial `try`/`catch`, catch filters and source
bindings, screen features, printing, and record-field alignment. The result
is a real third target, while the documentation and conformance results still
show plainly which BASIC-specific features cannot be portable (`e4fb248`–
`8c954a2`).

## 26 August to now: proving and presenting the language

The project then consolidated its test evidence into a cross-backend
conformance system. Tests were assigned stable IDs, descriptions, result
groups, expected states, and observed results. The documentation build
generates grouped pages for core language behavior, tutorials, and each
backend, distinguishing pass, failure, unsupported, unimplemented, deferred,
and not-applicable results (`1f38a82`, `c0a1b62`–`0e9dde8`). Random-file
compatibility was made directional and backend-scoped, and variable-length
record strings were covered in the in-memory record model.

The most recent work has focused on keeping the public material aligned with
the compiler: tutorial names and generated examples, syntax highlighting,
Windows and DOSBox guidance, a chaptered homepage, an accessible conformance
tab, and a concise account of Ramtech BASIC—the 1985 preprocessor that
inspired BASCAL (`a609efd`, `41dcf28`, `18d55be`, `aab2638`).

Today BASCAL is a Rust compiler for a structured BASIC-derived language with
classic BASIC, C, and JVM targets. Its development has moved from making BASIC
source more readable, through making its runtime translation trustworthy, to
making the same resolved language meaning available to three very different
backends—and publishing the resulting compatibility evidence alongside it.

</div>

[← Back to Why BASCAL](index.md)  ·  [Technical challenges →](challenges.md)
