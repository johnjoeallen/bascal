# Conformance metadata

Conformance tests expose stable IDs in the test sources. The metadata file
for the suite will define the description, groups, backend applicability, and
expected state for each ID. Valid states are `PASS`, `FAIL`, `UNSUPPORTED`,
`DEFERRED`, and `UNKNOWN`.

The documentation build must validate that every conformance test and tutorial
has exactly one metadata entry before generating any result page.

The metadata state is the expected feature state; generated pages compare it
with the observed test result. A required PASS that does not pass is shown as
FAIL. An expected DEFERRED or UNSUPPORTED check remains in that state until it
actually passes, at which point it is shown as PASS.
