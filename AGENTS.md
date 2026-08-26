# BASCAL — Agent Instructions

## Terminology

When describing how BASCAL source turns into generated BASIC or C code —
in docs, commit messages, or other prose — say **"transpile"**, not
"lower"/"lowering". "Lower/lowering" is compiler-internals jargon; this
project's docs consistently use "transpile" instead.

## Typed Intermediate Representation

After name and type resolution, every compiler stage must retain the full
compile-time type information needed by subsequent transformations and code
generators. Backends must consume this resolved typed IR; they must not
re-infer source types from syntax or backend-specific transpiled output.
