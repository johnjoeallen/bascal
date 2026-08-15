# Agent Instructions

- Basic code generation should only number lines that need numbers.
- Add a transpiler option to number every line when that behavior is explicitly requested.
- Format generated BASIC cleanly with retained comments, readable sectioning, and consistent indentation.
- Example BASCAL sources live in `examples/` and are the source of truth for transpiler tests.
- Generated `.bas` output belongs in `output/`.
- Temporary compiled binaries belong in `tmp/`.
