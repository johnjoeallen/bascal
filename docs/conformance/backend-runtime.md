# [Conformance tests](../)

## Backend and runtime compatibility

These tests compare generated programs with native runtimes and record the
backend-specific compatibility boundaries.

| Test description | BASIC | C | JVM |
| --- | :---: | :---: | :---: |
| JVM numeric literals and arithmetic | UNSUPPORTED | UNSUPPORTED | PASS |
| JVM scalar variables and constants | UNSUPPORTED | UNSUPPORTED | PASS |
| JVM scoped `GOTO` labels | UNSUPPORTED | UNSUPPORTED | PASS |
| JVM structured branches and loops | UNSUPPORTED | UNSUPPORTED | PASS |
| JVM scalar functions | UNSUPPORTED | UNSUPPORTED | PASS |
| JVM catch filters and source bindings | UNSUPPORTED | UNSUPPORTED | PASS |
| JVM portable error-handling tutorial | UNSUPPORTED | UNSUPPORTED | PASS |
| JVM pending record binary compatibility check | UNSUPPORTED | UNSUPPORTED | UNSUPPORTED |
| Constants and printing match BASCOM | PASS | PASS | UNSUPPORTED |
| `MID$` assignment matches BASCOM | PASS | PASS | UNSUPPORTED |
| Standard-library functions match BASCOM | PASS | PASS | UNSUPPORTED |
| Standard-library functions match C output | UNSUPPORTED | PASS | UNSUPPORTED |
| String self-concatenation matches BASCOM | PASS | PASS | PASS |
| String self-concatenation matches C output | UNSUPPORTED | PASS | UNSUPPORTED |
| Scalar methods match BASCOM | PASS | PASS | PASS |
| Scalar methods match C output | UNSUPPORTED | PASS | UNSUPPORTED |
| Tie-break rounding matches BASCOM | PASS | PASS | UNSUPPORTED |
| Standalone record literals are rejected consistently | PASS | PASS | PASS |
| Plain record declarations are rejected consistently | PASS | PASS | PASS |
| Record arrays are rejected consistently | PASS | PASS | PASS |
| Record-valued parameters are rejected consistently | PASS | PASS | PASS |
| Record-valued returns are rejected consistently | PASS | PASS | PASS |
| Nested record fields are rejected consistently | PASS | PASS | PASS |
| Bare dynamic record strings are rejected at parse time | PASS | PASS | PASS |
| Existing random-file records compile on all targets | PASS | PASS | FAIL |
| JVM random-file records produce the expected diagnostic | UNSUPPORTED | UNSUPPORTED | PASS |

<nav class="conformance-nav" aria-label="Conformance results navigation">
  <a href="../">← Previous: Core language and tutorials</a>
  <a href="../jvm/">Next: JVM-specific →</a>
</nav>
