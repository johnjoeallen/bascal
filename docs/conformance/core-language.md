# [Conformance tests](../)

## Core language and tutorial coverage

These tests exercise language constructs and complete tutorial programs across
the three transpilation backends.

| Test description | BASIC | C | JVM |
| --- | :---: | :---: | :---: |
| Every supported fixture transpiles successfully | PASS | PASS | PASS |
| All tutorial `.bcl` sources compile | PASS | N/A | N/A |
| Deterministic tutorials build and execute | N/A | PASS | N/A |
| C rejects the classic labels/error-handling tutorial | N/A | PASS | PASS |
| Nested procedure `TRY`/`CATCH` propagation | PASS | PASS | PASS |
| Interactive inventory case study | N/A | PASS | N/A |
| `remline` case study output | PASS | PASS | PASS |
| `MID$` assignment edge cases | PASS | PASS | PASS |
| Self-referential string concatenation | PASS | PASS | PASS |
| Built-in scalar method calls | PASS | N/A | N/A |
| Standard-library function execution | PASS | N/A | N/A |
| `remline` under FreeBASIC | PASS | N/A | N/A |
| Structured `TRY`/`CATCH`/`FINALLY` execution | PASS | PASS | PASS |
| Typed non-integer arrays and array parameters | N/A | N/A | PASS |
| Catch filters and source bindings | PASS | PASS | PASS |
| Portable error-handling tutorial | PASS | PASS | PASS |
| Hello-world assembly and execution | N/A | N/A | PASS |
| Numeric literals and arithmetic | PASS | PASS | PASS |
| Scalar variables and constants | PASS | PASS | PASS |
| Structured branches and `WHILE` loops | PASS | PASS | PASS |
| Scalar function calls and returns | PASS | PASS | PASS |
| Record binary compatibility across backends | PASS | PASS | FAIL |

<nav class="conformance-nav" aria-label="Conformance results navigation">
  <a href="../">← Previous: Overview</a>
  <a href="../backend-runtime/">Next: Backend and runtime →</a>
</nav>
