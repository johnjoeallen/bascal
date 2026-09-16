# rdgen

`rdgen` is a recursive-descent parser generator that emits idiomatic parser
code and concrete typed AST nodes. It is designed as an alternative to ANTLR
for grammars where generated hand-written-style parsing logic and direct AST
construction are preferable to a generic parse-tree runtime.

The workspace is organized around a language-agnostic IR. The grammar DSL
compiler resolves rule references, constructor bindings, recovery points, and
precedence tables into `rdgen-ir`; target-specific emitters consume that IR.
The Rust emitter is the reference backend, followed by C and C++.

The first grammar fixture is [`grammars/bascal.bcl.rdg`](grammars/bascal.bcl.rdg),
derived from BASCAL grammar revision 5. It is currently a design-spec fixture;
the grammar frontend will compile it into the typed IR as the next milestone.

[`grammars/distill.matcher.rdg`](grammars/distill.matcher.rdg) is a second
fixture for Distill’s matcher DSL. It exercises interpolation holes, regex
and string literals, closures, chainable postfix expressions, ternaries, and
parse-time closed vocabularies.

The frontend can be exercised from the workspace CLI:

```text
cargo run -p rdgen -- rdgen/grammars/bascal.bcl.rdg
```

Use `--emit-rust` to write generated AST and parser Rust to stdout:

```text
cargo run -p rdgen -- path/to/grammar.rdg --emit-rust
```

The Rust backend emits concrete typed AST declarations and parser control flow
for alternatives composed of literals, lexical terminals, and rule references.
Each alternative is attempted with input-offset backtracking before the next
alternative is tried. Optional and zero-or-more repetition of a single
literal, terminal, or rule is supported, as are groups whose alternatives each
contain one such element. Multi-element grouped alternatives and complex
grouped sequences are supported as unit-valued parser control flow. Nested
groups/repetitions are diagnosed as unsupported until their control flow is
implemented. Alternatives without constructor annotations receive stable
`AltN` AST variants so grammars remain representable while annotations are
added incrementally.

Alternatives can carry grammar-directed recovery metadata:

```ebnf
statement = "statement" => Statement()
    recover { sync ";", "}"; skip_until_sync; };
```
