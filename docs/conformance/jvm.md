# [Conformance tests](../)

## JVM-specific conformance

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
| Expected diagnostic for sequential file I/O | FAIL |
| Expected diagnostic for random/record file I/O | FAIL |
| Expected diagnostic for `MID$` assignment | FAIL |
| Expected failure for array `byval` clone assembly | FAIL |
| Random file binary compatibility | FAIL |
| Classic BASIC error handling | WILL NOT IMPLEMENT |

The expected-failure checks themselves pass when the JVM backend rejects an
unsupported feature with its documented diagnostic; the conformance result is
still FAIL because the required feature is not implemented. This keeps known
gaps visible without breaking the build.

<nav class="conformance-nav" aria-label="Conformance results navigation">
  <a href="../c/">← Previous: C-specific</a>
  <a href="../">Back to Core language</a>
</nav>
