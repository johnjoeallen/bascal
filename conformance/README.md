# Conformance metadata

Conformance tests expose stable IDs in the test sources. The metadata file
for the suite will define the description, groups, backend applicability, and
expected state for each ID. Valid states are `PASS`, `FAIL`, `UNSUPPORTED`,
`DEFERRED`, and `UNKNOWN`.

The documentation build must validate that every conformance test and tutorial
has exactly one metadata entry before generating any result page.
