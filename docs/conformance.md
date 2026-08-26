# Conformance results

This page records the backend conformance tests shipped with BASCAL. `PASS`
means the test currently passes in the normal test environment; `N/A` means
that the test does not apply to that backend (or requires an optional runtime
not present in the ordinary build). The matrix is kept alongside the tests in
`tests/` and should be updated whenever a backend capability changes.

| Test description | BASIC | C | JVM |
| --- | :---: | :---: | :---: |
| Every supported fixture transpiles successfully | PASS | PASS | PASS |
| All tutorial `.bcl` sources compile | PASS | N/A | N/A |
| Deterministic tutorials build and execute | N/A | PASS | N/A |
| C rejects the classic labels/error-handling tutorial | N/A | PASS | N/A |
| Nested procedure `TRY`/`CATCH` propagation | N/A | PASS | N/A |
| Interactive inventory case study | N/A | PASS | N/A |
| `remline` case study output | N/A | PASS | N/A |
| `MID$` assignment edge cases | PASS | N/A | N/A |
| Self-referential string concatenation | PASS | N/A | N/A |
| Built-in scalar method calls | PASS | N/A | N/A |
| Standard-library function execution | PASS | N/A | N/A |
| `remline` under FreeBASIC | PASS | N/A | N/A |
| Structured `TRY`/`CATCH`/`FINALLY` execution | N/A | N/A | PASS |
| Typed non-integer arrays and array parameters | N/A | N/A | PASS |
| Catch filters and source bindings | N/A | N/A | PASS |
| Portable error-handling tutorial | N/A | N/A | PASS |
| Hello-world assembly and execution | N/A | N/A | PASS |
| Numeric literals and arithmetic | N/A | N/A | PASS |
| Scalar variables and constants | N/A | N/A | PASS |
| Structured branches and `WHILE` loops | N/A | N/A | PASS |
| Scalar function calls and returns | N/A | N/A | PASS |
| Scoped `GOTO` labels | N/A | N/A | PASS |
| Pending JVM record binary compatibility check | N/A | N/A | N/A |
| Constants and printing match BASCOM | PASS | PASS | N/A |
| `MID$` assignment matches BASCOM | PASS | PASS | N/A |
| Standard-library functions match BASCOM | PASS | PASS | N/A |
| Standard-library functions match C output | N/A | PASS | N/A |
| String self-concatenation matches BASCOM | PASS | PASS | N/A |
| String self-concatenation matches C output | N/A | PASS | N/A |
| Scalar methods match BASCOM | PASS | PASS | N/A |
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

The optional BASCOM/DOSBox and Krakatau/JVM tests are skipped when their
external tools are unavailable; those environments are exercised by the
dedicated conformance workflow. The one ignored JVM record test is an
intentional pending check for issue #105.
