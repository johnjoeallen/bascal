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

Use `--emit-c` to write the current C AST ABI to stdout:

```text
cargo run -p rdgen -- path/to/grammar.rdg --emit-c
```

The C backend currently emits arena-compatible tagged-union declarations and
is validated as C11. Its recursive-descent parser emitter will consume the
same IR in a subsequent backend increment.

The initial C parser slice can be exercised with the smoke grammar:

```text
cargo run -p rdgen -- rdgen/grammars/c-parser-smoke.rdg --emit-c-parser
```

It supports alternatives composed of literals, named terminals, and rule
references, plus optional and zero-or-more repetition of a single literal,
terminal, or rule, with arena-backed AST construction and input-offset
backtracking.
The generated `rdgen_parser_init` and `rdgen_parse` functions provide the C
entry point and enforce complete input consumption after trivia skipping.
Groups and repetitions are rejected until their C control flow is implemented.

The Rust backend emits concrete typed AST declarations and parser control flow
for alternatives composed of literals, lexical terminals, and rule references.
Each alternative is attempted with input-offset backtracking before the next
alternative is tried. Optional and zero-or-more repetition, grouped
alternatives, and nested groups/repetitions are emitted as recursive parser
control flow. Grouped elements are currently unit-valued unless their
contents are bound directly as constructor fields. Alternatives without
constructor annotations receive stable `AltN` AST variants so grammars remain
representable while annotations are added incrementally.

Lexical policy is injected by the generated parser API rather than embedded in
the grammar backend. `TerminalScanner` recognizes named terminals and
`TriviaSkipper` advances over whitespace/comments. Use
`Parser::with_scanner_and_trivia` to provide both; `Parser::new` retains a
deliberately non-scanning default for literal-only parser tests. The public
`parse` method also rejects unconsumed non-trivia input.

Alternatives can carry grammar-directed recovery metadata:

```ebnf
statement = "statement" => Statement()
    recover { sync ";", "}"; skip_until_sync; };
```
