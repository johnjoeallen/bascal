# rdgen implementation plan

`rdgen` is a recursive-descent parser generator for producing idiomatic,
source-span-aware semantic frontends. BASCAL is the reference grammar and
Rust is the current target backend. The generated frontend is intended to be
usable by BASCAL's interpreter and by transpilers targeting BASIC, C, and JVM
assembly.

## Scope

rdgen owns:

- grammar compilation into a language-agnostic typed IR;
- left-recursion diagnostics and precedence-climbing metadata;
- generated recursive-descent parser control flow;
- generated semantic AST construction;
- source spans and parser diagnostics;
- grammar-directed recovery and line-oriented statement termination.

rdgen does not own name resolution, type checking, method dispatch, runtime
evaluation, or target code generation. Those stages consume the resolved
typed IR or semantic AST produced after parsing.

## Design constraints

### Semantic AST, not a parse tree

The public generated AST must represent language concepts rather than grammar
productions. Precedence layers, repetition wrappers, alternatives, optional
fragments, and statement terminators are parser machinery unless they encode
a real semantic concept.

Expression nodes should converge on:

```text
Name
IntegerLiteral
FloatLiteral
StringLiteral
TextBlock
Boolean
Unary
Binary
Call
Member
Index
RecordLiteral
Closure
```

Declarations and statements should similarly expose semantic concepts such as
`Program`, `Record`, `Function`, `Procedure`, `Method`, `Parameter`,
`Assignment`, `If`, `For`, `While`, `Do`, `SelectCase`, and `Try`.

Every semantic node must retain a source span. Operator tokens may retain a
more precise operator span in addition to the enclosing expression span.

### Concrete syntax remains line-oriented

BASCAL uses both physical newline and `:` as statement terminators. The
parser must preserve these distinctions:

- labels consume their own colon;
- colon chains become ordinary statement lists;
- single-line `if` is selected by physical-line structure;
- compact `then` and `else` bodies may contain colon-separated statements;
- block headers, bodies, and terminators obey physical line boundaries;
- comments remain available where BASCAL semantics require preservation.

### Validation is separate from recognition

The grammar recognizes syntax. A later semantic validation pass handles
context-sensitive rules including:

- valid placement of `&&` and `||`;
- duplicate combined-record members;
- invalid `byref` defaults;
- method receiver and inline-record-method context;
- array-reference versus call interpretation when semantic knowledge is
  required.

## Current implementation

The current branch has these capabilities:

- shared `rdgen-ir` for rules, alternatives, constructors, recovery points,
  output types, and precedence tables;
- Rust recursive-descent parser emission;
- typed constructor fields and explicit semantic output declarations;
- merged output enums for multiple concrete rules;
- reserved `Identity(value: value)` constructors for parser-only forwarding;
- source-span runtime types and lexical source-text retention;
- line-end, newline, same-line, cut, and recovery primitives;
- explicit BASCAL `Expr` output shared by precedence-layer rules;
- `Binary(left, operator, right)` precedence metadata and generated AST
  helpers;
- a `climb <atom>` grammar element that wires a rule directly to its
  `precedence` table, replacing the hand-written layer ladder;
- `prefix <op> binds_below <op> => Constructor(...)` precedence-table
  entries, generating a Pratt-style prefix-aware atom so `-2^2` still
  parses as `-(2^2)` and `not a = b` as `not (a = b)`;
- a backtracking precedence-climbing primitive
  (`parse_precedence_climbing_tokens_backtracking`) used by `climb` so an
  operator literal that turns out to introduce something else (e.g. a
  comment marker sharing a leading character with an operator) is left
  unconsumed instead of hard-failing the parse;
- a `fold(base, step)` grammar element: parses `base` once, then repeatedly
  parses `step`, replacing `step`'s magic `base`-role constructor field
  with the value accumulated so far each time. This is climb's postfix
  counterpart: it is what makes `a.b.c()` nest as `Call(Member(Member(a,
  b)), c)` instead of staying a flat `Vec` of suffixes;
- BASCAL corpus recognition coverage, now generated entirely from the
  `climb`-driven `expr` rule and the `fold`-driven `postfix_expr` rule;
- automatic source-span propagation (Rust backend): every constructed
  non-lexical node gets a `span: SourceSpan` field for free (no grammar
  annotation needed), bracketing parser positions around each alternative's
  parse; every generated output-type enum also gets an inherent `span(&self)
  -> SourceSpan` method so precedence/prefix/fold combine logic can read a
  child node's span generically without matching on every variant by name;
- semantic AST snapshot coverage: six compile-and-run tests in
  rdgen-codegen-rust, one per step-9 category, parsing representative
  BASCAL source through the real bascal.bcl.rdg grammar and asserting
  both node shape and span correctness (not parser-production names);
- constant constructor fields: `field: -1` binds a fixed integer directly
  in a constructor, for values a grammar alternative implies but never
  consumes a token for. Typed `i64`/`int64_t` in the two backends; cannot
  be forwarded through `Identity` (there is no element behind it to
  forward). Closes the `downto` gap from step 5: `ForBounds::Downto` now
  carries `step: -1` alongside `limit`, instead of leaving callers to
  infer the implied step on their own.

Recent milestones:

- resolver now retains each function's recursively declared `global` names,
  keyed by resolved function identity; C emission consumes that metadata when
  allocating file-scope storage and suppressing shadowing locals;
- moved C's fixed-array-bound constant analysis into `ResolvedProgram`:
  `resolver::resolve` now retains direct top-level integer `const` values
  under their case-insensitive name/suffix keys, and `codegen_c` consumes
  that fact for both top-level and function-local array declarations. This
  removes one backend AST rescan while keeping the existing rule intact:
  only literal integer top-level constants can establish C's fixed bounds;
- made `ResolvedProgram` the sole backend entry point: `codegen_basic`,
  `codegen_c`, and `codegen_jvm` now all accept it rather than a bare
  legacy `Program`, and the driver passes the resolver result unchanged to
  each target. C and JVM still have local AST scans to migrate into the
  resolver, but they can no longer be invoked on an unresolved AST;
- integrated the generated Rust frontend into `bcc`'s build: root `build.rs`
  compiles `rdgen/grammars/bascal.bcl.rdg` and emits the parser into Cargo's
  `OUT_DIR`; `src/rdgen_frontend.rs` includes that output, supplies the
  BASCAL terminal/trivia/case-insensitive-literal callbacks, and exposes a
  `parse` entry point with a direct smoke test. This is intentionally a
  parallel frontend, not an adapter to the legacy `ast::Program`: adapting
  it there would discard the generated source spans and semantic AST shape,
  then leave the existing backends re-inferring source types. The next
  pipeline migration must instead make the resolver and backends consume a
  resolved typed form derived from this AST;
- closed the step 5 `downto` gap: added constant constructor fields to
  the grammar DSL (`step: -1`, a bare signed integer literal in
  constructor position instead of a label) so `for_bounds`'s `downto`
  alternative can carry `step: -1` explicitly (`i64`), the same way `to`
  carries its own (real, parsed) `step` expression. The two `step` fields
  intentionally stay different types (`Option<Box<Expr>>` for `to`, plain
  `i64` for `downto`) rather than forcing one shape: fabricating a real
  `Expr::IntegerLiteral` subtree for the constant would require rdgen to
  know BASCAL's own literal-node shape, which breaks the backend-agnostic
  "IR data only, no inferred semantics" rule everything else here follows.
  A consumer already discriminates `To` vs `Downto`; now it just never has
  to know `-1` is implied by `Downto` rather than parsed. Extended the
  lexer (a new `Number` token kind), grammar validation (constant fields
  skip the "field references unknown label" check; rejected inside
  `Identity`, which has nothing to forward such a field from), and both
  backends' constructor-field type/value emission (Rust: `i64`; C: needed
  a new `<stdint.h>` include for `int64_t`) to support it;
- step 9 (semantic AST snapshots): added
  `snapshot_precedence_associativity_and_unary_vs_exponent`,
  `snapshot_calls_members_and_indexes`, `snapshot_records_and_methods`,
  `snapshot_compact_statement_chains_comments_and_labels`,
  `snapshot_downto_compound_assignment_and_let`, and
  `snapshot_error_handling_and_control_flow` to rdgen-codegen-rust,
  covering every category the plan lists. Implemented as inline
  compile-and-run tests (matching the existing `generated_bascal_parser_*`
  convention already used throughout this file) rather than a new
  external fixture-file-plus-golden-snapshot system, since introducing
  file-based snapshot infrastructure wasn't otherwise needed and every
  other rdgen test already lives this way.

  Writing these against the real grammar (not a toy one) surfaced two
  genuine pre-existing quirks worth knowing about, not new regressions:

  - `identifier`'s own lexical rule allows `.` as a continuation
    character (for dotted require/import paths), so a bare `obj.field`
    lexes as *one* identifier token and never reaches `postfix_suffix` -
    member access is only reachable when whitespace precedes the `.`
    (`obj .field`). Not fixed here: resolving it needs a lexing-priority
    decision (dotted identifiers vs. member access) beyond this step's
    scope.
  - non-lexical rules' spans can start slightly before their first real
    token when preceded by skippable trivia the rule itself never
    explicitly consumes before reaching it (documented in
    `emit_parser_rule`, where `rdgen_span_start` is captured). A tempting
    fix - skip trivia before capturing the span start - was tried and
    reverted: it silently broke `if_tail`'s block-vs-single-line
    detection, which depends on trivia *not* being eagerly skipped ahead
    of the `newline` builtin element. Corpus recognition caught this
    immediately (`cargo test -p rdgen-codegen-rust` regressed from 0 to
    83 corpus failures), which is exactly why that gate stays in the loop
    on every change, not just BASCAL-specific edits.

- step 8 (source-span propagation), Rust backend only: every generated
  struct-like variant (ordinary alternatives, zero-field tags, precedence
  `Binary`, prefix `Unary`) now carries a `span: SourceSpan`; lexical
  default token variants already had one. `Identity`-forwarded values
  don't get a new span (they forward the child's). `climb`'s precedence
  combine function computes `Binary`/`Unary` spans from the left/right (or
  operator start/operand end) children's own spans via the new `.span()`
  method; `fold`'s `apply_fold_base` widens the step's own (too-narrow,
  e.g. just ".member") span to start where the accumulated base started.
  A real bug surfaced and got fixed along the way: the span-tracking local
  variable was originally named plain `start`, which silently shadowed
  grammar fields also named `start` (`for_stmt`'s loop-start expression,
  `mid_assign`'s start offset) inside the same closure scope, corrupting
  their values; renamed to `rdgen_span_start`/`rdgen_span_end` to avoid
  colliding with any user-chosen field name. `span` itself is now a
  reserved constructor field name, rejected at compile time if a grammar
  author tries to bind it explicitly. The C backend is untouched — it has
  no span support at all, matching its existing gap with `climb`/`fold`;
  its own test suite doesn't exercise spans so nothing broke.

  Span policy (the plan asks this be defined explicitly): a node's span
  covers everything its own alternative consumes, start to finish — the
  position before its first element is attempted through the position
  after its last element succeeds. Leading and trailing keywords/literals
  that are part of the alternative are included (e.g. `record_decl`'s span
  runs from `record` through the closing `record` keyword, not just the
  name), because they're ordinary elements like any other, parsed inside
  the same bracketed region. Nothing is trimmed or re-derived after the
  fact. Delimiters that are *not* part of the alternative at all (a
  `,` separating repeated items) naturally fall inside the enclosing
  repetition's own span the same way, not a per-item one.

- step 7 (statements and recovery) audited against the checklist: `If`/
  `For`/`While`/`Do`/`SelectCase`/`Try` and the ~48 other `statement_core`
  variants (I/O, transfer, error-handling, labels, comments, ...) already
  match the design's target statement list — each is its own named rule
  with every matched element bound to a field, no discarded groups found
  (the same audit method that caught `param`'s missing passing mode in
  step 6 found nothing comparable here). No grammar changes from this
  step; recovery points (`recover { sync ...; ...; }`) remain unused by
  BASCAL's own grammar, same as before — the IR/codegen support exists and
  is tested, but nothing in this grammar exercises it yet;
- step 6 (declarations and records) audited against the checklist: most of
  the section (`Program`/file items, `RecordDeclaration`, `FunctionDeclaration`/
  `ProcedureDeclaration`/`MethodDeclaration`, `Parameter`'s axes/default/
  type_annotation, `FileDeclaration`) was already in target shape — real,
  named semantic nodes, not grammar wrappers. Found and fixed the two
  actual gaps: `param`'s `[ "byref" | "byval" ]` was matched but never
  bound to a field, silently discarding the passing mode entirely, now
  captured via a `passing_mode` tagged rule (mirroring `string_align`/
  `file_mode`'s existing pattern for "which literal matched"); and
  `record_decl`'s inline `combines: [...]` list is now a named
  `combined_record_list` (`first`/`rest`) sub-rule instead of an ad hoc
  3-tuple, matching the `first`/`rest` convention used everywhere else in
  the grammar (`dim_stmt`, `param_list`, `array_axes`, ...);
- started step 5 (assignments/control-flow normalization): `let` is now
  matched and discarded rather than threaded through as an `Option<Token>`
  field on every `assignment_or_expr_stmt` alternative; `assignment_op`
  collapsed from five zero-field tags (`Assign`/`AddAssign`/...) into one
  `lexical` rule retaining the matched token directly, matching how
  `Binary`/`Unary` already carry their operator; and colon chains
  (`statement`) are now a flat `Line(first, rest: Vec<StatementCore>)`
  list instead of a right-nested `Core(core, continuation: Option<(Token,
  Box<Statement>)>)` cons chain. `downto`-to-`For`-with-step-`-1` was left
  undone at this point (rdgen had no way to fabricate a constructor field
  with no source token yet) - closed later via constant constructor
  fields, see the milestone above. Single-line/block `if` and label/
  comment preservation were already satisfied by the existing grammar,
  nothing to change there;
- wired BASCAL's `postfix_expr` rule to `fold(primary, postfix_suffix)`,
  deleting the flat `Postfix(base, suffixes: Vec<Member>)` list in favor of
  properly nested `Member`/`Call` chains;
- normalized `primary`/`ident_primary` leaves: `CallOrArray`/`FileOrRecordIndex`
  renamed to `Call`/`Index`, float/integer/hex/octal/string literals renamed
  to `FloatLiteral`/`IntegerLiteral`/`HexLiteral`/`OctalLiteral`/
  `StringLiteral`, and the redundant `Identifier(...)` wrapper around
  `ident_primary` dropped in favor of forwarding it with `Identity`;
- wired BASCAL's `expr` rule to `climb postfix_expr` against the
  `precedence expr` table, deleting `xor_expr` through `pow_expr` and their
  provisional `Xor`/`Or`/`And`/`Not`/`Compare`/`Add`/`Mod`/`IntegerDivide`/
  `Multiply`/`Negate`/`Power`/`Postfix` constructors in favor of `Binary`
  and `Unary`;
- `d003141` — share BASCAL expression AST output;
- `f44b2fe` — annotate BASCAL precedence binary nodes;
- `c00684d` — use semantic output in precedence helpers.

The generated BASCAL `Expr` now exposes semantic nodes at the binary/unary
layer (`Binary`, `Unary`), properly nested postfix chains (`Member`,
`Call`), and most leaves (`Name`, `Call`, `Index`, `*Literal`, `True`/
`False`, `RecordLiteral`/`PartialRecordLiteral`). Two small shapes remain
provisional by necessity rather than oversight, both because ordinary
alternatives (unlike precedence *levels*, which may deliberately share one
constructor across many operators) cannot merge under one constructor name
within a single rule in the C backend (which, unlike the Rust backend,
neither merges rules by output type nor special-cases `Identity`, so it
sees every "duplicate" within one rule, not just across rules):

- `primary`'s parenthesized-expression alternative keeps a `Parenthesized`
  wrapper instead of forwarding via `Identity`, because `primary` already
  forwards `ident_primary` with `Identity` and the C backend rejects a
  second `Identity` alternative in the same rule.
- `True`/`False` stayed as two variants rather than one shared `Boolean`,
  for the same reason applied to an ordinary (non-`Identity`) constructor
  name.

Declaration/statement shaping (steps 6-7 below) is still outstanding.

## Implementation sequence

### 1. Finish output-type-aware IR and emitter behavior

- Keep rule references typed through declared output types.
- Ensure parser entry points return the start rule's declared output type.
- Ensure precedence helpers return the declared output type.
- Keep explicit zero-field constructors distinct from default constructors.
- Reject ambiguous shared output declarations at grammar-compile time.
- Add focused tests for output-type propagation through fields, parser rules,
  start entry points, and precedence helpers.

### 2. Define semantic constructor annotations

Extend the grammar DSL and IR only where the parser needs semantic
normalization. Candidate contracts include:

- identity forwarding for parser-only rules;
- binary repetition folding;
- unary prefix construction;
- call/member/index construction;
- named statement-list and declaration constructors;
- explicit normalization markers for compound assignment and `downto`.

Constructor annotations must remain typed IR data. Backends must not infer
semantic meaning from rule names or from generated syntax.

### 3. Integrate precedence parsing without changing unary semantics

The immediate technical problem is wiring BASCAL's `parse_expr` to generated
precedence-climbing construction while preserving:

```text
-2^2 == -(2^2)
```

The solution must explicitly model:

- the precedence atom;
- prefix unary operators and their binding power;
- postfix operators;
- right-associative exponentiation;
- operator token retention;
- the shared semantic output type.

Do not select `unary_expr` as a generic atom if that causes unary minus to
consume exponentiation incorrectly. Add the required grammar/IR metadata for
prefix and postfix behavior instead.

The integration should produce `Expr::Binary` and `Expr::Unary` directly from
the generated parser. A compatibility normalization pass may be used as an
intermediate step, but it must not become the permanent consumer-facing API.

### 4. Normalize expression leaves and postfix operations

Replace provisional expression constructors with semantic nodes:

- literal rules construct typed literal nodes while retaining spelling and
  spans;
- `true` and `false` normalize to BASCAL's selected runtime representation;
- identifiers construct `Name`;
- calls construct `Call`;
- standalone member access constructs `Member`;
- indexing constructs `Index`;
- record literals construct `RecordLiteral`;
- postfix chains are ordinary semantic nesting, not `Postfix` lists.

Array-reference versus call ambiguity remains a semantic validation concern
where the grammar cannot decide from syntax alone.

### 5. Normalize assignments and control-flow syntax

Make syntax-directed normalization explicit in the AST construction layer:

- discard `let`;
- convert `+=`, `-=`, `*=`, and `/=` to ordinary assignment plus a binary
  expression, or record an explicit normalization marker where lvalue cloning
  is not yet safe;
- convert `downto` to `For` with an explicit step of `-1`;
- preserve single-line and block `if` distinctions;
- convert colon chains to ordinary statement lists with individual spans;
- preserve labels and comments as required by BASCAL re-emission semantics.

### 6. Shape declarations and records

Move declarations from grammar wrappers to semantic nodes:

- `Program` and file items;
- records, fields, combined-record lists, and inline methods;
- functions, procedures, fluent methods, and free-standing methods;
- parameters, passing mode, array axes, defaults, and type references;
- file declarations and other declaration-specific semantic forms.

Context-sensitive receiver rules and duplicate combined members remain in the
semantic validation pass.

### 7. Shape statements and recovery

Consolidate statement wrappers into semantic statement variants for:

- assignment and expression statements;
- I/O and runtime statements;
- transfer and error-handling statements;
- loops and conditional forms;
- `select case`;
- `try/catch/finally`;
- labels and comments.

Keep recovery points in the grammar and IR. Generated diagnostics should
retain the furthest failure and source position while recovery remains
explicit and target-backend-specific where necessary.

### 8. Add source-span propagation

Use parser byte offsets to construct spans for every semantic node. The span
policy must define whether delimiters, keywords, and terminators are included
for each node category. Add snapshots and direct AST assertions for:

- literals and names;
- nested expressions;
- assignments;
- colon-separated statements;
- declarations and block constructs;
- comments and preserved source text.

### 9. Add semantic AST snapshots and validation fixtures

Create representative BASCAL fixtures covering:

- precedence and associativity;
- unary minus versus exponentiation;
- calls, members, and indexes;
- records and methods;
- compact statement chains;
- comments and labels;
- `downto`, compound assignment, and `let`;
- error-handling and control-flow constructs.

Snapshots should assert semantic shape and spans, not parser-production names.

### 10. Keep recognition and regression gates green

After every coherent parser or grammar change:

1. add a focused regression test;
2. run the relevant crate tests;
3. run `bash rdgen/scripts/probe-bascal-files.sh` when grammar or parser
   behavior changes;
4. run `cargo test --workspace --offline`;
5. run `cargo clippy --workspace --all-targets --offline`;
6. run `git diff --check`;
7. commit only green changes.

The BASCAL corpus probe currently covers 83 `.bcl` files. A recognition
regression must be narrowed or reverted before proceeding.

## Backend order

Rust remains the reference backend. Once the semantic IR and Rust emitter are
stable:

1. use C as the pressure-test backend for arena allocation and tagged unions;
2. add C++ using the C strategy with optional RAII and `std::variant` choices;
3. add Java, Go, C#, Kotlin, and Swift backends as their AST/error idioms are
   specified;
4. defer target-specific optimization and runtime integration until the
   semantic frontend contract is stable.

The IR must remain language-agnostic. SGDL is intentionally out of scope for
the current implementation, but its future lowering should target the same
parser-generation IR rather than introducing an EBNF-only semantic model.

## Definition of completion

The Rust/BASCAL parser work is complete when:

- generated public AST nodes are semantic and source-span-aware;
- no precedence-layer or parser-wrapper types leak into the public AST;
- generated parsing constructs semantic expressions directly;
- syntax-directed normalization is explicit and tested;
- semantic validation has a defined diagnostic boundary;
- all repository BASCAL files remain recognized;
- semantic AST snapshots cover representative syntax;
- the workspace tests and Clippy gates are green apart from documented
  pre-existing warnings;
- the generated AST is suitable as the frontend input to interpretation and
  later transpilation stages.
