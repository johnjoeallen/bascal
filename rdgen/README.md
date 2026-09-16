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
