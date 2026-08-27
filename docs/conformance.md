# Conformance tests

## Core language

| Test description | BASIC | C | JVM |
| --- | :---: | :---: | :---: |
| All tutorial .bcl sources compile | PASS | PASS | DEFERRED |
| Built-in scalar method calls | PASS | PASS | PASS |
| C/JVM reject the classic labels/error-handling tutorial | NOT APPLICABLE | PASS | PASS |
| Deterministic tutorials build and execute | PASS | PASS | DEFERRED |
| Interactive inventory case study | PASS | PASS | DEFERRED |
| Standard-library function execution | PASS | PASS | PASS |
| Typed non-integer arrays and array parameters | DEFERRED | DEFERRED | PASS |
| BASCOM creates file; target validates binary compatibility | NOT APPLICABLE | PASS | NOT APPLICABLE |
| Target creates file; BASCOM validates binary compatibility | NOT APPLICABLE | PASS | NOT APPLICABLE |
| BASCOM creates file; JVM validates binary compatibility | NOT APPLICABLE | NOT APPLICABLE | FAIL |
| JVM creates file; BASCOM validates binary compatibility | NOT APPLICABLE | NOT APPLICABLE | FAIL |
| Every supported fixture transpiles successfully | PASS | PASS | PASS |
| Adventure game compiles through the front end without backend code generation | UNIMPLEMENTED | UNIMPLEMENTED | UNIMPLEMENTED |
| Array of records is not yet supported | PASS | PASS | PASS |
| Variable-length string records are rejected as random-access file types | PASS | PASS | PASS |
| In-memory records support variable-length string fields | PASS | PASS | PASS |
| Existing random-file records compile on all targets | PASS | PASS | FAIL |
| Nested record fields are not yet supported | PASS | PASS | PASS |
| Arrays of records are not yet supported | PASS | PASS | PASS |
| Record-valued parameters are not yet supported | PASS | PASS | PASS |
| Record-valued returns are not yet supported | PASS | PASS | PASS |
| Standalone record literals are not yet supported | PASS | PASS | PASS |

<nav class="conformance-nav" aria-label="Conformance results navigation">
  <a href="../">← Overview</a>
  <a href="tutorials/">Next: Tutorials →</a>
</nav>
