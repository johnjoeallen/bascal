# rdgen

`rdgen` is a recursive-descent parser generator that emits idiomatic parser
code and concrete typed AST nodes. It is designed as an alternative to ANTLR
for grammars where generated hand-written-style parsing logic and direct AST
construction are preferable to a generic parse-tree runtime.

The workspace is organized around a language-agnostic IR. The grammar DSL
compiler resolves rule references, constructor bindings, recovery points, and
precedence tables into `rdgen-ir`; target-specific emitters consume that IR.
Rust is currently the only completion target and the reference backend for
BASCAL. The C emitter is retained as an experimental pressure test, but C++
and GC-language backends are deferred until the Rust/BASCAL path is complete.

The first grammar fixture is [`grammars/bascal.bcl.rdg`](grammars/bascal.bcl.rdg),
derived from BASCAL grammar revision 5. The grammar frontend compiles it into
the typed IR and both current backends can emit declarations/parser source for
it.

Grammars may select their parser entry rule explicitly with `start <rule>;`.
If omitted, rdgen retains the compatibility default of using the first rule.
This is important for grammars that declare lexical rules before their
syntactic start rule, as BASCAL does.

Character-level lexical productions can be declared with `lexical`:

```text
lexical identifier = letter, { letter | digit };
```

The Rust backend disables trivia skipping while such a rule is consuming its
characters, then restores the surrounding parser mode when the rule returns.

The current BASCAL syntax smoke probe is reproducible with:

```text
bash rdgen/scripts/probe-bascal-files.sh
```

It generates the BASCAL parser, supplies a minimal case-insensitive scanner
and comment/whitespace skipper, and reports recognition for every `.bcl` file
under the tutorial, examples, and fixture trees. It intentionally reports
failure until the starter grammar covers the complete BASCAL statement set.

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

The C backend emits arena-compatible tagged-union declarations and a C11
recursive-descent parser for the currently supported control-flow slice. The
parser supports literals, named terminals, rule references, optional and
zero-or-more repetitions, nested single-element groups, multi-element grouped
alternatives, backtracking, trivia skipping, literal matching callbacks, and
arena-backed AST construction.

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
Single- and multi-element groups, including groups with alternatives, are
supported for consumption-only parsing. Groups containing repetitions remain
outside the current C parser slice and are rejected explicitly.

The Rust backend emits concrete typed AST declarations and parser control flow
for alternatives composed of literals, lexical terminals, and rule references.
Each alternative is attempted with input-offset backtracking before the next
alternative is tried. Optional and zero-or-more repetition, grouped
alternatives, and nested groups/repetitions are emitted as recursive parser
control flow. Labeled singleton groups preserve the child type, and labeled
single-alternative multi-element groups preserve their contents as Rust
tuples. Labeled groups with multiple alternatives are rejected until a
dedicated group-sum type is added. Alternatives without
constructor annotations receive stable `AltN` AST variants so grammars remain
representable while annotations are added incrementally.

The grammar compiler rejects repetitions whose child can derive epsilon; this
guarantees that generated repetition loops have a progress invariant.

Precedence tables are retained in the shared IR and validated for defined rule
targets, reachable operator literals, non-empty levels, non-empty operators,
and duplicate operators. The
Rust emitter exposes the validated levels as typed generated metadata and emits
per-rule operator lookup functions returning the level and associativity. It
also emits binding-power pairs for a conventional precedence-climbing loop:
left-associative operators use `(left_bp, right_bp) = (bp, bp + 1)` and
right-associative operators use `(bp, bp)`. The generated parser does not yet
consume those functions to select a grammar rule automatically, but it exposes
`Parser::parse_precedence_climbing` so an emitter can provide typed atom,
operator, and combine callbacks. An operator callback returns `Ok(None)` and
leaves the input position unchanged when no operator is present. Expression
tables also generate `parse_<rule>_precedence`, which supplies that rule's
binding-power lookup automatically.

Precedence levels may carry an AST-constructor contract:

```text
precedence expr {
    left "+" => Binary(left: left, operator: operator, right: right);
}
```

The constructor must bind exactly `left`, `operator`, and `right`. The Rust
backend emits the annotated constructor as a typed rule variant with boxed
recursive operands and a `Token` operator field, and preserves the same
contract in generated metadata.

When every level in a precedence table has a constructor annotation, the Rust
backend also emits `parse_<rule>_precedence_ast`. It accepts typed atom and
operator callbacks and constructs the annotated variants directly. Tables
with any unannotated level retain the generic `parse_<rule>_precedence` API;
this avoids silently selecting a constructor for an incomplete AST contract.

Repeated grammar bodies may declare their stopping literals explicitly:

```text
{ statement } until { "else", "end" }
```

The Rust emitter checks these literals after trivia skipping and before each
iteration, so nested block productions do not consume their own terminators.

Lexical policy is injected by the generated parser API rather than embedded in
the grammar backend. `TerminalScanner` recognizes named terminals and
`TriviaSkipper` advances over whitespace/comments. The C callback variants
receive the source buffer length, so scanners and trivia handlers can operate
on non-NUL-terminated input safely. Use
`Parser::with_scanner_and_trivia` to provide both; `Parser::new` retains a
deliberately non-scanning default for literal-only parser tests. The public
`parse` method also rejects unconsumed non-trivia input. Callback offsets are
UTF-8 byte offsets and are accepted only when they remain within the source,
advance the current position where advancement is required, and land on a
character boundary.

Alternatives can carry grammar-directed recovery metadata:

```ebnf
statement = "statement" => Statement()
    recover { sync ";", "}"; skip_until_sync; };
```

The Rust and C parser emitters currently implement `skip_until_sync` by
advancing to the next synchronization literal and returning a diagnostic at
that offset. Synchronization literals must be non-empty and unique within a
recovery point. `insert_token` and `abort_rule` remain represented in the IR
but are rejected by both parser backends until their target-language semantics
are defined.
