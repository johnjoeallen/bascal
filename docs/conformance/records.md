# [Conformance tests](../)

## Files and records

| Test description | BASIC | C | JVM |
| --- | :---: | :---: | :---: |
| BASCOM creates file; target validates binary compatibility | NOT APPLICABLE | PASS | NOT APPLICABLE |
| Target creates file; BASCOM validates binary compatibility | NOT APPLICABLE | PASS | NOT APPLICABLE |
| BASCOM creates file; JVM validates binary compatibility | NOT APPLICABLE | NOT APPLICABLE | FAIL |
| JVM creates file; BASCOM validates binary compatibility | NOT APPLICABLE | NOT APPLICABLE | FAIL |
| Array of records is not yet supported | PASS | PASS | PASS |
| Variable-length string records are rejected as random-access file types | PASS | PASS | PASS |
| In-memory records support variable-length string fields | PASS | PASS | PASS |
| Existing random-file records compile on all targets | PASS | PASS | FAIL |
| JVM random-file records produce the expected diagnostic | NOT APPLICABLE | NOT APPLICABLE | FAIL |
| Nested record fields are not yet supported | PASS | PASS | PASS |
| Arrays of records are not yet supported | PASS | PASS | PASS |
| Record-valued parameters are not yet supported | PASS | PASS | PASS |
| Record-valued returns are not yet supported | PASS | PASS | PASS |
| Standalone record literals are not yet supported | PASS | PASS | PASS |

<nav class="conformance-nav" aria-label="Conformance results navigation">
  <a href="../jvm/">← Previous: JVM-specific</a>
  <a href="../">← Overview</a>
</nav>
