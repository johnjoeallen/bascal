# Backend and runtime compatibility

These tests compare generated programs with native runtimes and record the
backend-specific compatibility boundaries.

| Test description | BASIC | C | JVM |
| --- | :---: | :---: | :---: |
| JVM numeric literals and arithmetic | N/A | N/A | PASS |
| JVM scalar variables and constants | N/A | N/A | PASS |
| JVM scoped `GOTO` labels | N/A | N/A | PASS |
| JVM structured branches and loops | N/A | N/A | PASS |
| JVM scalar functions | N/A | N/A | PASS |
| JVM catch filters and source bindings | N/A | N/A | PASS |
| JVM portable error-handling tutorial | N/A | N/A | PASS |
| JVM pending record binary compatibility check | N/A | N/A | N/A |
| Constants and printing match BASCOM | PASS | PASS | N/A |
| `MID$` assignment matches BASCOM | PASS | PASS | N/A |
| Standard-library functions match BASCOM | PASS | PASS | N/A |
| Standard-library functions match C output | N/A | PASS | N/A |
| String self-concatenation matches BASCOM | PASS | PASS | PASS |
| String self-concatenation matches C output | N/A | PASS | N/A |
| Scalar methods match BASCOM | PASS | PASS | PASS |
| Scalar methods match C output | N/A | PASS | N/A |
| C random-file output is readable by BASCOM | N/A | PASS | N/A |
| BASCOM random-file output is readable by C | N/A | PASS | N/A |
| Tie-break rounding matches BASCOM | PASS | PASS | N/A |
| Standalone record literals are rejected consistently | PASS | PASS | PASS |
| Plain record declarations are rejected consistently | PASS | PASS | PASS |
| Record arrays are rejected consistently | PASS | PASS | PASS |
| Record-valued parameters are rejected consistently | PASS | PASS | PASS |
| Record-valued returns are rejected consistently | PASS | PASS | PASS |
| Nested record fields are rejected consistently | PASS | PASS | PASS |
| Bare dynamic record strings are rejected at parse time | PASS | PASS | PASS |
| Existing random-file records compile on BASIC and C | PASS | PASS | N/A |
| JVM random-file records produce the expected diagnostic | N/A | N/A | PASS |
