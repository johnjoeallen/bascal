# [Conformance tests](../)

## Files and records

| Test description | BASIC | C | JVM |
| --- | :---: | :---: | :---: |
| Random file binary compatibility | PASS | PASS | FAIL |
| BASCOM random-file output is readable by C | PASS | PASS | FAIL |
| C random-file output is readable by BASCOM | PASS | PASS | PASS |
| JVM random-file output is readable by BASCOM | FAIL | FAIL | DEFERRED |
| Array of records is not yet supported | PASS | PASS | PASS |
| In-memory records with variable-length string fields are not yet supported | PASS | PASS | PASS |
| Existing random-file records compile on BASIC and C | PASS | PASS | PASS |
| JVM random-file records produce the expected diagnostic | PASS | PASS | FAIL |
| Nested record fields are not yet supported | PASS | PASS | PASS |
| Arrays of records are not yet supported | PASS | PASS | PASS |
| Record-valued parameters are not yet supported | PASS | PASS | PASS |
| Record-valued returns are not yet supported | PASS | PASS | PASS |
| Standalone record literals are not yet supported | PASS | PASS | PASS |

<nav class="conformance-nav" aria-label="Conformance results navigation">
  <a href="../jvm/">← Previous: JVM-specific</a>
  <a href="../">← Overview</a>
</nav>
