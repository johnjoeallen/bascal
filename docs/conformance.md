# Conformance tests

## Core language

| Test description | BASIC | C | JVM |
| --- | :---: | :---: | :---: |
| All tutorial .bcl sources compile | PASS | PASS | DEFERRED |
| Built-in scalar method calls | PASS | PASS | PASS |
| C/JVM reject the classic labels/error-handling tutorial | NOT APPLICABLE | UNSUPPORTED | UNSUPPORTED |
| Deterministic tutorials build and execute | PASS | PASS | DEFERRED |
| Interactive inventory case study | PASS | PASS | DEFERRED |
| Random file binary compatibility | PASS | PASS | FAIL |
| Standard-library function execution | PASS | PASS | PASS |
| Typed non-integer arrays and array parameters | DEFERRED | DEFERRED | PASS |
| BASCOM random-file output is readable by C | PASS | PASS | FAIL |
| C random-file output is readable by BASCOM | NOT APPLICABLE | PASS | FAIL |
| Every supported fixture transpiles successfully | PASS | PASS | PASS |
| Array of records is not yet supported | PASS | PASS | PASS |
| Bare dynamic record string fields are not yet supported | PASS | PASS | PASS |
| Nested record fields are not yet supported | PASS | PASS | PASS |
| Arrays of records are not yet supported | PASS | PASS | PASS |
| Record-valued parameters are not yet supported | PASS | PASS | PASS |
| Record-valued returns are not yet supported | PASS | PASS | PASS |
| Standalone record literals are not yet supported | PASS | PASS | PASS |

<nav class="conformance-nav" aria-label="Conformance results navigation">
  <a href="../">← Overview</a>
  <a href="tutorials/">Next: Tutorials →</a>
</nav>
