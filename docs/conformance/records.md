# [Conformance tests](../)

## Files and records conformance

| Test description | BASIC | C | JVM |
| --- | :---: | :---: | :---: |
| Constants and printing match BASCOM | PASS | PASS | N/A |
| `MID$` assignment matches BASCOM | PASS | PASS | N/A |
| Standard-library functions match BASCOM | PASS | PASS | N/A |
| Standard-library functions match C output | N/A | PASS | N/A |
| C random-file output is readable by BASCOM | N/A | PASS | N/A |
| BASCOM random-file output is readable by C | N/A | PASS | N/A |
| Standalone record literals are rejected consistently | PASS | PASS | PASS |
| Plain record declarations are rejected consistently | PASS | PASS | PASS |
| Record arrays are rejected consistently | PASS | PASS | PASS |
| Record-valued parameters are rejected consistently | PASS | PASS | PASS |
| Record-valued returns are rejected consistently | PASS | PASS | PASS |
| Nested record fields are rejected consistently | PASS | PASS | PASS |
| Existing random-file records compile on BASIC and C | PASS | PASS | N/A |
| JVM random-file records produce the expected diagnostic | N/A | N/A | PASS |

<nav class="conformance-nav" aria-label="Conformance results navigation">
  <a href="../jvm/">← Previous: JVM-specific</a>
  <a href="../">Back to overview</a>
</nav>
