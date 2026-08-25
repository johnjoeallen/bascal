# JVM array implementation record

This record tracks the staged JVM array work on the `jvm-arrays` branch.

## Why the work was split into stages

The JVM backend originally inferred types from `BasicIdent` names and scalar
declarations. That was sufficient for scalar expressions, but not for arrays:
array rank, element type, frozen bounds, and lowered `SIZEOF` references must
survive into code generation. The implementation was therefore isolated on a
branch and extended incrementally.

## Stages completed

1. Added rank-aware array metadata (`element_suffix` plus a full dimension list)
   to the shared AST/IR.
2. Populated that metadata while parsing nested declarations.
3. Preserved it through program merging and record lowering.
4. Switched JVM codegen to consume the typed array metadata.
5. Added JVM array descriptors and static fields.
6. Added nested allocation with `multianewarray`, using BASIC's
   upper-bound-plus-one sizing rule.
7. Added multidimensional integer reads and writes using `aaload`, `iaload`,
   and `iastore`.
8. Added initial `SIZEOF` handling for one-dimensional arrays.
9. Expanded the shared IR with typed array-reference records carrying array
   identity, rank, indices, element suffix, and optional `SIZEOF` axis.
10. Began populating typed array-reference records during parsing for array
    expressions, assignments, control-flow conditions, and `SIZEOF`.
11. Connected parser-shaped `Expr::Call` array references to JVM reads and
    writes, and represented array parameters as rank-aware aliases to the
    program's static array storage so tutorial 08 now transpiles.

Each stage was committed separately and the existing JVM conformance suite
continued to pass.

## Prompting/clarifications that enabled progress

- “Proceed in stages on a separate branch till complete” established the
  isolated `jvm-arrays` branch and incremental commits.
- “Will need multi-dimensional support” changed the design from a one-
  dimensional special case to rank-aware metadata and nested JVM arrays.
- “We need to update the IR so the JVM knows what the types are” identified
  the central issue: codegen must consume typed declaration metadata instead
  of reconstructing types from names after lowering.
- Repeated “proceed with implementation” prompts kept the work moving through
  metadata, allocation, access, and propagation stages rather than stopping at
  the initial design discussion.

## Current status

Tutorial 08 now transpiles for the JVM target, including one- and two-
dimensional integer reads, writes, `SIZEOF`, and array-parameter calls. Runtime
array-parameter copying and non-integer element arrays remain future work.

## Latest continuation prompt

The follow-up request was to continue array support while keeping this record
up to date. This is now part of the implementation workflow: every array
stage and every prompting clarification is recorded here before the branch is
considered complete.

## Relevant commits

- `eeb15ff` — rank-aware JVM array metadata
- `1646b0f` — rank-aware JVM array allocations
- `1319678` — one-dimensional integer array reads
- `3e0bfe5` — one-dimensional integer array writes
- `163390a` — multidimensional integer array access
- `f5af594` — array size expressions
- `85ddd82`, `f80f39c`, `34971ef`, `ed4c097`, `1d537c9` — typed IR definition,
  propagation, parsing, and JVM integration
- `e0aa6f3` — call-shaped array references and rank-aware array parameters
