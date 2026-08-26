# [Conformance tests](../)

## Files and records conformance

| Test description | BASIC | C | JVM |
| --- | :---: | :---: | :---: |
| Constants and printing match BASCOM | PASS | PASS | UNSUPPORTED |
| `MID$` assignment matches BASCOM | PASS | PASS | UNSUPPORTED |
| Standard-library functions match BASCOM | PASS | PASS | UNSUPPORTED |
| Standard-library functions match C output | UNSUPPORTED | PASS | UNSUPPORTED |
| C random-file output is readable by BASCOM | UNSUPPORTED | PASS | UNSUPPORTED |
| BASCOM random-file output is readable by C | UNSUPPORTED | PASS | UNSUPPORTED |
| Standalone record literals are rejected consistently | PASS | PASS | PASS |
| Plain record declarations are rejected consistently | PASS | PASS | PASS |
| Record arrays are rejected consistently | PASS | PASS | PASS |
| Record-valued parameters are rejected consistently | PASS | PASS | PASS |
| Record-valued returns are rejected consistently | PASS | PASS | PASS |
| Nested record fields are rejected consistently | PASS | PASS | PASS |
| Existing random-file records compile on BASIC and C | PASS | PASS | UNSUPPORTED |
| JVM random-file records produce the expected diagnostic | UNSUPPORTED | UNSUPPORTED | PASS |

<nav class="conformance-nav" aria-label="Conformance results navigation">
  <a href="../jvm/">← Previous: JVM-specific</a>
  <a href="../">Back to overview</a>
</nav>
