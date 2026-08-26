# Conformance results

The documentation build runs the complete suite with `cargo test --locked`
before publishing these results. `PASS` means the test passes; `N/A` means the
backend is not applicable (or requires an optional runtime).

The results are grouped into two pages, each showing more than 15 related
tests:

- [Core language and tutorial coverage](conformance/core-language.md)
- [Backend and runtime compatibility](conformance/backend-runtime.md)

Status cells are rendered as coloured buttons: green for PASS, red for FAIL,
and gray for N/A.
