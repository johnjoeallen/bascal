# JVM-specific conformance

| Test description | Result |
| --- | :---: |
| Structured `TRY`/`CATCH`/`FINALLY` execution | PASS |
| Typed non-integer arrays and array parameters | PASS |
| Catch filters and source bindings | PASS |
| Portable error-handling tutorial | PASS |
| Hello-world assembly and execution | PASS |
| Numeric literals and arithmetic | PASS |
| Scalar variables and constants | PASS |
| Structured branches and `WHILE` loops | PASS |
| Scalar function calls and returns | PASS |
| Scoped `GOTO` labels | PASS |
| Expected diagnostic for sequential file I/O | PASS |
| Expected diagnostic for random/record file I/O | PASS |
| Expected diagnostic for `MID$` assignment | PASS |
| Expected failure for array `byval` clone assembly | PASS |
| Pending record binary compatibility check | N/A |

Expected-failure checks are passing when the JVM backend rejects an
unsupported feature with its documented diagnostic. They keep known gaps
visible without treating an unimplemented feature as a broken build.

<nav class="conformance-nav" aria-label="Conformance results navigation">
  <a href="../c/">← Previous: C-specific</a>
  <a href="../">Back to Core language</a>
</nav>
