# BASCAL — Agent Instructions

## Audience and documentation style

BASCAL is a technical project: a compiler, aimed at programmers who already
know a language like BASIC, Pascal, or C. Documentation, commit messages,
and code comments should use precise technical/compiler terminology rather
than plain-English paraphrases — say "lexer", "parser", "AST", "resolver",
"codegen backend", "diagnostic", "scope", "shadowing", "transpile" (per the
convention below), not informal substitutes like "the part that reads the
code" or "turns it into". Prefer the exact term a compiler engineer would
use, and don't soften or simplify jargon for a lay reader.

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
