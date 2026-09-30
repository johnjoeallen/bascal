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

- made the legacy AST backing `ResolvedProgram` private. Backends and driver
  code use a crate-internal compatibility view while their remaining AST
  consumers are migrated, giving the generated semantic frontend one
  controlled replacement boundary rather than a public escape hatch;
- began the semantic-to-resolved adapter with span-preserving module headers
  and `require`/`import` declarations in `semantic_ir`, directly from
  `rdgen_frontend::Program` and without a legacy-AST round trip;
- promoted parsed typed-array declarations onto `ResolvedProgram`; JVM now
  consumes that resolver-owned semantic metadata rather than reading the
  legacy program's parser-populated side table directly;
- promoted typed array references (rank, indices, and `sizeof` axes) onto
  `ResolvedProgram` as well, keeping JVM array-use code on the same resolved
  metadata boundary;
- promoted driver-loaded COMMON blocks onto `ResolvedProgram`; BASIC emission
  now consumes the resolved metadata instead of a mutable program side table;
- resolver now retains each function's recursively declared `global` names,
  keyed by resolved function identity; C and JVM emission consume that
  metadata when allocating/sharing program-wide storage, eliminating their
  duplicate recursive declaration scans;
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

### 0. Migrate the compiler pipeline onto the generated semantic frontend

The generated `rdgen_frontend::Program` must become the parser output for the
new pipeline; it must not be converted back into `ast::Program`, because that
would discard source spans and reintroduce parser-era type inference. The
migration proceeds through a new resolved typed IR:

1. add a semantic-to-resolved declaration adapter for program headers,
   dependencies, records, and callable signatures, retaining generated spans;
2. add typed expression and statement adapters, with resolution facts attached
   as fields rather than recomputed by backends;
3. migrate each backend from `ResolvedProgram::legacy_program()` to those
   resolved fields, deleting the compatibility accessor only after no caller
   remains;
4. switch the driver to the generated frontend and retain legacy-parser
   differential fixtures until corpus and diagnostic equivalence are proven.

The legacy compatibility view is deliberately private as of the initial
migration commit, so no public API can grow around it during this transition.

The adapter now also retains typed control-transfer targets, assignment and
comparison operators, input provenance, optional throw/return/randomize/write
payloads, and source spans for generated names and punctuation. This keeps
backend consumers on semantic IR data rather than requiring syntax recovery.

The current adapter tests also cover declaration spans, record literals,
control-transfer targets, and omitted-versus-present optional statement
payloads. These are compatibility guards for the eventual backend migration;
they are not invitations for backends to reconstruct syntax from tokens.

Declaration and control-reference nodes now retain generated source spans,
including record fields, callable parameters, file declarations, labels,
`restore`, `goto`, `gosub`, and `catch` bindings. File modes and loop/input
choices are represented as typed enums at the semantic boundary.

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

### Adapter coverage milestone

The generated BASCAL frontend now has an adapter boundary for declarations,
expressions, control flow, error handling, file and console I/O, data/read/
restore, and terminal statements. `semantic_ir::parse_and_adapt` is the
canonical source-to-typed-IR entry point; downstream stages consume its
resolved semantic payloads rather than re-inferencing syntax from generated
nodes.

### Next integration stages (59–68)

The next tranche is intentionally backend-facing:

59. attach the generated semantic module to the resolver pipeline;
60. define generated-to-resolved diagnostic conversion;
61. migrate driver legacy-form diagnostics to semantic IR;
62. migrate generated-name conflict checks;
63. migrate the BASIC backend's declaration facts;
64. migrate BASIC statement emission incrementally;
65. migrate C backend declaration and type facts;
66. migrate JVM backend declaration and type facts;
67. add generated-versus-legacy differential fixtures at driver boundaries;
68. remove the private `ResolvedProgram::program` compatibility view after all
    consumers are gone.

Stages 59–68 cannot be completed by adapter-only edits: the current driver,
BASIC, C, and JVM backends still call the private legacy compatibility view.
That is the remaining implementation blocker; the next work should proceed
backend-by-backend with differential tests after each migration slice.

Stages 59 and 60 are complete: `ResolvedProgram` retains an optional
generated semantic module, and generated parser failures have a compiler
`Diagnostic` conversion helper. Stage 61 now runs the always-on legacy-form
advisories over generated semantic statements and derives warning locations
from retained source text and filenames. File compilation reports each
module's findings while that module's source buffer is still available, so
dependency merging cannot attach a dependency span to the root filename.
Stage 62 now validates generated BASIC function-name conflicts from semantic
callable signatures and semantic name collection, including resolved COMMON
names, on both source and file compilation paths. The legacy AST checker
remains only as a compatibility entry point when generated parsing is
unavailable. Stages 63–68 remain backend-facing work.

Stage 63 has started by making a complete semantic callable parameter-rank
vector authoritative in BASIC allocation. A semantic scalar parameter's
explicit zero-rank fact no longer falls through to a rank inferred from call
syntax. Semantic callable `global` declarations now also participate in the
resolved global-name scope used by BASIC's allocator. BASIC now selects
callable metadata by semantic identity (name, kind, receiver, and arity)
without revalidating result and parameter types against legacy AST
annotations, so semantic signature facts remain authoritative during backend
callable allocation. The rest of BASIC
declaration and emission migration remains open.

Stages 65 and 66 have begun: when semantic IR is present, an empty semantic
array-declaration set is now authoritative in both the C and JVM backends;
neither backend scans the compatibility AST to repopulate it. The remaining
C/JVM declaration and type facts still need migration. C's global scalar
declaration pass also now starts from semantic name scopes (filtering
callable names) and applies semantic declaration types; its AST scan is kept
for compatibility callers only. JVM's top-level scalar declarations now use
the same semantic name-scope and declaration-type sources, filter callable
names, and remove names owned by the resolved array table before slot
allocation. Its AST scan remains in compatibility mode. C record-layout
classification now comes only from semantic record declarations and file
references when semantic IR is available; lowered `FIELD` statements are
compatibility artifacts and no longer supplement that table. C and JVM
callable ABI selection now treats a present semantic parameter fact as
authoritative for scalar/array classification and `byref`; callable lookup no
longer rejects semantic signatures by comparing result and parameter types
back to legacy AST annotations. Legacy parameter syntax is consulted only
when semantic metadata is absent. C now takes parameter suffixes from
semantic parameter metadata for its scalar and array ABI types, and JVM uses
those suffixes for scalar slots and array element types. C now derives
callable return classification and C result suffixes from the semantic
callable result type, while JVM uses the same fact for its method descriptor,
return emission, and verifier fallback. C and JVM array
declaration setup now selects the semantic table wholesale whenever semantic
IR is present, so stale resolver-era typed-array declarations cannot leak
through when that table is empty. JVM's callable-local declaration collector
also records `const` bindings from the semantic initializer type and name,
preserving the integer default for unresolved or boolean initializer types. It
also discovers scalar output variables passed to semantic `byref` parameters,
using semantic call signatures rather than a second AST parameter-mode scan.
When a matching semantic callable exists, JVM local-slot declaration setup
now uses that semantic walk authoritatively; AST declaration scanning is kept
only for compatibility callers without matching semantic callable metadata.
The semantic walk also recognizes implicit scalar targets in `READ`,
`LINE INPUT`, `LSET`, and `RSET` statements.
C's semantic scalar collector now recognizes those same writable targets for
module and callable storage declarations. It also classifies an unsuffixed
scalar `DIM` with no type annotation as the default single-precision type
rather than omitting that declaration from semantic storage facts. C's
semantic array collector applies that same default to unsuffixed array `DIM`
declarations without a type annotation.
Semantic expression annotation now resolves one-dimensional array-shaped call
syntax against DIM declarations in the active lexical scope. Callable-local
declarations are merged after module declarations, so a same-named local array
shadows the module array and its declared element type is retained on the
semantic index expression. A regression verifies that a suffixless local
`AS STRING` array remains string-typed despite a same-named module integer
array.

Stage 67 now has driver-boundary differential regressions covering scalar
declarations, callable signatures, and fixed-size arrays. It compares BASIC, C,
and JVM output from the ordinary resolved semantic path against the AST-only
compatibility path. A required-library callable is also compared across BASIC,
C, and JVM output. Resolver diagnostic parity checks an undefined label on
both paths; BASIC backend diagnostic parity covers undeclared array rank and
declared/body rank mismatch. Stage 68 remains blocked until migrated backend
consumers no longer depend on the legacy `Program` view.

The offline workspace test run is green after the Stage 67 fixes: all unit,
integration, and fixture suites pass. The fixes covered expression-based array
capacities, record `FIELD` buffers, callable and method metadata, array
parameter rank diagnostics, `RESTORE` data labels, and JVM generated-name and
slot handling. `git diff --check` also passes. Stage 68 is not ready: the
remaining dependency is direct backend and driver use of
`ResolvedProgram::program`, so migration must remove those consumers before
deleting that compatibility field.

Stage 64 has begun with BASIC module-scope `CONST` and `DIM` emission. The
backend reads literal, name, parenthesized, unary, and binary expressions
from semantic IR and uses them to produce assignments and array capacities.
For top-level `DIM`, it consumes semantic axes and type annotations when every
axis is renderable; inferred axes and unsupported expression forms remain on
the compatibility path. The adapter retains named capacity expressions as
typed semantic expressions instead of reducing them to strings. Differential
coverage checks numeric and string constants, constant references, literal
capacities, arithmetic capacities, ordinary builtin calls, Boolean literals,
array reads, and callable-local or nested declarations. Callable-local
`CONST` and `DIM` collection now follows semantic callable identity and uses
the allocated local scope when rendering names. A JVM regression verifies
that a named semantic `CONST` supplies the fixed array capacity. User-callable
arguments, array-bound builtins, record expressions, and inferred axes still
use compatibility emission. `CodeGenerator::generate` still passes most
`program.statements` to the AST emitter and iterates `program.functions`;
recursive `self.statements` calls are AST-typed. Continue statement-family
by statement-family with differential coverage. Do not adapt semantic nodes
back into AST nodes, since that would discard the typed IR boundary this
migration establishes.

The next ten migration iterations tightened compile-time integer facts used
by C and JVM array backends. Semantic constant and bound evaluation now
accepts decimal, hexadecimal, and octal literals, parenthesized expressions,
integer division, `MOD`, and Boolean literals (BASIC integer values `-1` and
`0`); checked arithmetic rejects overflow, division by zero, and cyclic
constant references. C numeric expression emission transpiles hexadecimal and
octal tokens to numeric values. Regression coverage checks semantic dimensions
and emitted capacities in C and JVM. These changes preserve resolved
expression types and keep array-capacity decisions in semantic IR. The next
stage remains Stage 64's statement-family migration, especially inferred
dimensions and unsupported initializer expressions; Stage 68 still depends
on removing remaining backend and driver reads of `ResolvedProgram::program`.

The next Stage 64 slice adds BASIC differential coverage for parenthesized
expressions, radix literals, multidimensional capacities, integer division,
`MOD`, unary Boolean expressions, comparisons, builtin calls, and callable-local
array declarations. A semantic DIM precedence regression confirms that BASIC
uses the typed IR's array capacity when it differs from the compatibility AST.
The typed IR also correctly classifies a concatenated string `CONST` as string
storage; the legacy AST-only path currently assigns that case integer storage.
Keep semantic string typing as the expected result and retire the discrepancy
when the remaining BASIC declaration consumers migrate. Next, continue with
semantic statement emission beyond `CONST` and `DIM`, then remove the current
AST statement gate instead of extending compatibility fallbacks.

The following Stage 64 slice extends BASIC semantic constant rendering for
`SIZEOF`, `LBOUND`, and `UBOUND`. Calls with a known semantic array and a
literal axis now resolve from the typed array rank and captured DIM bounds;
calls without an explicit axis also work for one-dimensional arrays. The same
resolution works in callable-local scopes, and builtin spelling remains
case-insensitive. Differential coverage checks one- and multidimensional
arrays, callable-local declarations, and semantic-vs-AST capacity divergence.
When semantic resolution cannot handle a call, AST compatibility rendering
still supplies the established diagnostic; regressions cover unknown arrays
and out-of-range axes. The semantic statement gate and callable iteration
remain the next Stage 64 work. Stage 68 still requires removing every direct
backend and driver dependency on `ResolvedProgram::program`.

The next twenty Stage 64 iterations extended semantic constant handling for
array-bound builtins. `SIZEOF`, `LBOUND`, and `UBOUND` now resolve from typed
array ranks and the bounds captured at `DIM`; literal axes work for arrays of
multiple ranks, callable-local arrays, and array parameters. Differential
fixtures cover case-insensitive builtin names, nested declarations, runtime
captured capacities, and 1D/2D/3D arrays. Invalid, omitted, and nonliteral axes
and unknown arrays retain diagnostics through the compatibility renderer.
`SIZEOF` now checks its count increment and reports overflow rather than
panicking. The semantic path preserves `AS INTEGER` on an unsuffixed typed
array; the AST-only compatibility path drops that annotation, like its earlier
string-constant type loss. Keep typed semantic output authoritative and retire
both parity gaps as AST declaration emission is removed. Next, continue with
semantic emission for further BASIC statement families and reduce the top
level AST statement dependency; Stage 68 still depends on eliminating all
backend and driver reads of `ResolvedProgram::program`.

The next twenty Stage 64 iterations completed the BASIC array declaration
and bound coverage matrix. Semantic `DIM` type annotations are checked for
LONG, SINGLE, DOUBLE, and STRING at module and callable scope; array-bound
builtins are checked for every scalar suffix and declared element type. The
matrix also covers callable/global arrays, 3D axes, multi-item declarations,
captured and arithmetic capacities, and the largest valid `UBOUND` and
`SIZEOF` results. `SIZEOF` overflow now reports a diagnostic, and its missing
axis diagnostic names `SIZEOF` correctly. Invalid, negative, nonliteral, and
missing axes all retain precise diagnostics. One remaining compatibility
discrepancy is explicit typed array annotation loss in the AST-only BASIC
path; semantic output preserves the type. The next slice must begin consuming
additional `SemanticStatementKind` families directly, then shrink the legacy
statement emitter and ultimately remove `ResolvedProgram::program` from
backend consumers.

The next twenty slices broadened typed BASIC declaration coverage across
numeric and string annotations, scalar suffixes, callable-local arrays,
multidimensional parameters, and module/callable capacity sources. Boundary
tests cover the maximum representable `UBOUND` and `SIZEOF` results, `SIZEOF`
overflow, and diagnostics for missing, negative, nonliteral, or invalid axes.
The `SIZEOF` missing-axis diagnostic now names the correct builtin. A first
direct semantic statement path now emits complete top-level streams composed
only of `STOP`, `CLS`, `BEEP`, `SYSTEM`, and `CLEAR` from
`SemanticStatementKind`; the path declines mixed or unsupported streams as a
whole, preserving AST compatibility emission. The next slice should expand
this closed subset to other statement families whose semantic representation
retains all required output data, then handle mixed streams by semantic-node
dispatch rather than converting nodes back into AST.

The next twenty statement-family slices promoted `END` into
`SemanticStatementKind` and adapted both top-level and line-form terminators.
BASIC's semantic-only top-level emitter now handles `END`, `GLOBAL`, terminal
intrinsics, `DATA`/`READ`, user labels, `GOTO`/`GOSUB`, `RESTORE`, `RESUME`, and
`ON ERROR GOTO`. Transfer targets use the generator's callable-entry and
user-label mapping, so BASIC line numbering continues to resolve them. The
emitter accepts only a wholly supported top-level stream; mixed or unsupported
streams retain the AST path. Regression coverage verifies semantic-source
precedence, line-numbered labels, procedure error-handler entry, data/read
expressions, and whole-stream fallback. Next, add semantic dispatch for
expression-bearing statements and then nested control-flow bodies, while
keeping statements whose IR lacks source payload (such as comment text) on
the compatibility path until the typed IR retains that information.

The following Stage 64 slice extended direct BASIC emission from terminal and
transfer statements into scalar assignments, printing, scalar and machine
statements, and file I/O. Assignments preserve all five semantic assignment
operators. `PRINT`/`LPRINT` cover standard, `USING`, channel, and printer
destinations with separators retained from semantic tokens. File operations
include `OPEN`, `WRITE`, `CLOSE`, `SEEK`, `INPUT`, `LINE INPUT`, `FIELD`,
`GET`/`PUT`, `LSET`/`RSET`, `KILL`, and `NAME`. Scalar output includes
`OPTION BASE`, `ERASE`, `RANDOMIZE`, `SWAP`, `POKE`, `OUT`, `WIDTH`, `LOCATE`,
`COLOR`, `ERROR`/`THROW`, and computed `ON ... GOTO`/`GOSUB`. Operands are
rendered from semantic expressions, and the whole-stream gate retains the AST
compatibility path if any statement family is unsupported or an operand cannot
be rendered. Regression tests cover semantic-source precedence, compound
assignment, print modes, file and scalar operations, and mixed-stream fallback.
Full `cargo test --offline` passes (627 library tests, 19 binary tests, and all
integration suites). Next, migrate the remaining side-effecting statement
families and replace the whole-stream gate with per-node semantic dispatch
without reconstructing AST nodes. Nested control flow and Stage 68 removal of
`ResolvedProgram::program` from backend consumers remain open.

The next semantic BASIC slice added nested block `IF`, counted `FOR` (including
`DOWNTO`), `WHILE`, and typed `SELECT CASE` dispatch. Control-flow statements
reuse the existing guarded-jump layout and generator label counter; declined
streams restore the counter before AST fallback. `SELECT CASE` derives its
temporary suffix from the selector's retained semantic type and covers value,
range, and comparison case clauses. Semantic `TRUE`/`FALSE` names transpile as
`-1`/`0`. Keyword-prefix matching was then fixed in the RDGEN Rust and C parser
emitters for multi-character alphabetic literals while preserving prefix
matching for one-character literals. This prevents names such as `done%` from
being adapted as a leading `DO` statement, so direct semantic `DO` emission is
enabled again. The `MID$` assignment grammar now matches `mid$` as a complete
literal before `(`; matching `mid` separately failed the new keyword boundary
because `$` is an identifier continuation. Its adapter again retains
`SemanticStatementKind::MidAssign`. Semantic `EXIT` and `CONTINUE` use the
active nested loop context: `FOR` exits use native `EXIT FOR`, while/`DO` exits
branch to the loop end, and continues branch to the existing loop continuation
label. Loop context and label state are restored if semantic emission declines.
Rust and C parser output was generated from a keyword smoke grammar. C parser
emission for the full BASCAL grammar remains unavailable because the C backend
does not yet support grouped or repeated elements.
Structured error handling and callable bodies remain open.

Top-level per-node dispatch now has source provenance. `SemanticModule` keeps a
source index parallel to each top-level semantic statement; named adaptation
sets it, and dependency prepending shifts indexes as source lists are merged.
The BASIC backend preflights a unique, ordered match against legacy `Stmt`
positions before emitting anything. A `Line` span supplies the source line for
its colon-separated children; its byte span may begin before leading whitespace
or blank lines, so the mapper advances to the first non-whitespace character.
Legacy blank-line markers are excluded from candidate matches and retained in
the compatibility emission pass. When the mapping is proven, supported nodes
transpile from typed semantic IR and declined nodes use only their matched AST
statement. If source identity or a unique match is unavailable, the existing
whole-stream gate remains active.

Regression coverage now gives AST and semantic sources different literal
values at identical positions: colon-separated and later `PRINT` nodes must use
semantic values, while an unsupported top-level `TRY` must use its matched AST
body. It includes a blank line. The multi-file `source$` boundary test now also
has a top-level library `PRINT`, exercising shifted dependency source indexes
and source lookup across semantic library emission and root AST fallback. Both
per-node dispatch tests pass. A separate failed-alignment regression gives the
semantic module a different filename and verifies dispatch declines before
emitting; an unsupported semantic node then preserves the whole-stream AST
fallback with no partial semantic output. `cargo check --offline` and
`git diff --check` pass. An additional same-file ambiguity regression inserts
a second AST statement at the same source line and verifies alignment declines
without writing output. Semantic expression dispatch identifies indexed arrays
through resolved ranks, preserving source identifier casing and multiple axes,
and declines calls to declared callables so method resolution uses AST codegen.
The semantic/legacy output differential covers these cases. All focused cases
pass, and full `cargo test --offline` passes across the workspace (636 library
tests, 19 binary tests, and every integration suite). `cargo check --offline`
and `git diff --check` pass. Next, extend per-node dispatch to other codegen
backends.

BASIC callable bodies now also preflight source-position alignment using the
retained semantic source and the matching callable signature. When aligned,
typed callable scalar assignments, standard `PRINT`, plain `LPRINT`, console
or channel `INPUT`, `READ` targets, one-dimensional local array writes and
reads, and function `RETURN` expressions emit
with the function's BASIC name allocator;
unsupported nodes retain their matching AST statements. Differential
regressions confirm semantic assignment, print, input prompts/targets, array
indices and values, and return values replace AST values inside a function. Semantic INPUT prompt
tokens are unquoted and re-escaped once for BASIC output, shared with the
top-level semantic path. This starts callable-level
per-node migration while preserving the legacy emitter for unsupported forms.

The C backend now preflights top-level semantic-to-AST statement alignment
using retained source indexes and exact source positions. Aligned `END`,
`STOP`, and `SYSTEM` nodes emit their C termination forms directly from
semantic IR; other statements continue through the existing C AST emitter.
The same direct dispatch slice includes `CLS` and `BEEP`, whose C output is
self-contained and does not require expression rendering or helper state.
Missing source identity, ambiguous positions, or malformed source spans
decline the mapping before emission, so this stage cannot produce partial
semantic output. A differential regression supplies `STOP` in the AST and
`END` in the semantic source at the same location and confirms the generated
`main` follows semantic IR; a second differential regression verifies `CLS`
over an AST `BEEP`, and a source-filename mismatch test verifies the mapper
declines. These focused regressions pass. Remaining C statement families
still use AST emission except for scalar numeric assignment. That path now
renders integer, radix, floating-point, boolean, unary-minus, arithmetic,
division, comparison, short-circuit boolean, integer-division, `MOD`,
bitwise, unary `NOT`, and numeric `ABS`, `INT`, `FIX`, `SQR`, `SGN`, `CINT`,
`CLNG`, `CSNG`, and `CDBL` calls with typed arguments from semantic IR. `SQR`
uses the floating math path; `SGN` follows the target dialect's helper ABI;
the conversion calls preserve their existing rounding and result types; and
`SIN`, `COS`, `TAN`, `ATN`, `LOG`, and `EXP` use their existing `<math.h>`
mappings.
Logical `&&`/`||`
preserve short-circuiting and normalize to BASCAL's `-1`/`0`; float-to-integer
bitwise operands retain round-away-from-zero conversion and request `<math.h>`
only when needed. The backend takes each
identifier and target type from retained expression type metadata. Unsupported
expressions decline the single node and use its aligned AST statement. The
C64 capability gate is preserved: semantic float assignments cannot bypass
the no-float target diagnostic. Differential tests cover semantic value
precedence, per-node fallback on a call expression, and the C64 float gate.
Numeric compound assignment (`+=`, `-=`, `*=`, `/=`) now renders a typed
read/compute/store sequence, applying the same integer-result rounding as
ordinary assignment. Standard-output `PRINT` also dispatches per node for
semantic numeric and string expressions and preserves comma/semicolon
newline behavior. Semantic string literals remain embedded in the C format
string with percent escaping, while string expressions use `%s` arguments;
`DATE$` and `INKEY$` retain their runtime-helper calls. Unsupported
destinations and expressions fall back to the matched AST statement.
String-valued `CHR$`, `STR$`, `MID$`, `LEFT$`, and `RIGHT$` expressions now
also dispatch from typed semantic calls. Numeric `LEN`, `ASC`, `VAL`, and
`INSTR` calls consume prelude-free typed string expressions directly. The
semantic IR annotates intrinsic result types, and C preserves its existing
ring-buffer, formatting, and search helpers.
Numeric user-defined function calls now render through resolved C signatures
in typed scalar assignments and standard `PRINT`, including nested calls,
typed trailing defaults, parameter conversions, and type-compatible scalar
byref names passed by address. String-returning user-defined calls now render
through the caller-output-buffer ABI in typed scalar assignments and standard
`PRINT`, including nested string arguments and trailing defaults. Calls with
try-result signatures, unsupported parameter shapes, or a byref expression
that is not a matching scalar name still decline to the aligned AST call path.
The semantic type annotator now recurses through every expression-bearing
statement form, including `MID$`, loop bounds and conditions, `SELECT CASE`
values, I/O operands, `LOCATE`, `COLOR`, `RANDOMIZE`, and callable catch
filters. C screen and random-seed emitters now receive resolved function and
array tables, allowing supported numeric user calls in these operands at both
top level and inside callable bodies. Differential regressions verify semantic
call arguments replace their AST counterparts.
Top-level `LOCATE` now renders typed semantic coordinates with BASCAL's
integer conversion, and `COLOR` passes typed numeric expressions to the
existing CGA-to-ANSI runtime helper. A differential regression verifies both
statements use semantic values over their aligned AST counterparts.
`RANDOMIZE` now dispatches supported typed numeric seeds from semantic IR and
retains the existing `TIMER`/bare time-seed behavior; unsupported seed
expressions decline only that node. Its differential regression verifies the
semantic seed wins over the aligned AST. `DATA` is consumed as an aligned
no-op because its values already populate the runtime table from semantic IR;
`READ` targets now use typed semantic scalar or one-dimensional array
references for conversion and storage. Differential regressions verify
semantic DATA values and scalar and indexed READ target selection.
`SWAP` now emits typed, type-matched scalar and declared-array lvalues from
semantic IR. Numeric values use a temporary of the resolved C type; string
values use `sizeof` for scalar and static-array destinations and the fixed
element capacity for runtime-backed arrays. Top-level and callable
differential tests cover semantic lvalue selection for numeric scalars,
strings, and array elements; unsupported or mismatched lvalues still fall back
per statement.
Top-level and callable `CLOSE` now emit their typed literal channel directly
and request the file runtime table even when no `OPEN` statement precedes
them. Differential regressions verify the semantic channel replaces the AST
channel and that runtime storage is declared.
Top-level and callable `LINE INPUT #` now emit typed literal channels and
string scalar or array lvalues, preserving each target's buffer capacity while
requesting the sequential-file helper and channel table. Differential tests
verify semantic channel and array-index selection over aligned AST statements.
Top-level and callable `WRITE #` now render typed string and numeric values
from semantic IR, including string-returning user calls, while preserving the
quoted-string/comma-separated record format and requesting file runtime state.
Differential tests verify semantic channel and value selection over the AST.
Top-level and callable plain `PRINT #` now reuse the typed semantic token
renderer with a literal channel and `fprintf`, registering the file runtime
table when emitted. `PRINT # USING` retains the aligned AST path. An end-to-end
C regression verifies string and numeric output reaches the selected file.
Top-level and callable `INPUT #` now consume typed literal channels and scalar
targets in semantic order, selecting string, integer, or floating-point input
conversion from each resolved target type. The dispatch requests channel
storage and the sequential-file helpers. Differential tests verify semantic
channel and target order over the AST.
Top-level `RESTORE` now resolves its semantic label against semantic DATA
offsets and updates the shared DATA cursor directly; a differential regression
confirms the semantic label and DATA sequence take precedence over the AST.
Top-level `GOSUB` now uses its resolved semantic label while preserving the
existing return-stack ID and resume-label protocol; a differential regression
checks the semantic target and generated return site.
Top-level bare `RETURN` now emits the matching dynamic return-stack switch
from semantic IR when GOSUB sites exist; otherwise it declines to the existing
diagnostic path. A differential test verifies semantic RETURN dispatch.
Computed `ON ... GOTO` and `ON ... GOSUB` now dispatch their typed selector and
ordered semantic label list through C `switch` statements. Each computed
GOSUB arm reserves a distinct return-stack ID, included in semantic GOSUB
counting, so bare `RETURN` resumes at the correct continuation. Differential
tests verify semantic selectors and targets and both return IDs.
Top-level structured `THROW` now renders its typed semantic error code and
registers the matching retry/continuation raise-site labels through the C
error runtime. A differential regression verifies semantic error-code
selection and raise-site emission. A try-reachable callable now also emits a
typed semantic `THROW` through the existing result-wrapper ABI, with a
differential regression confirming the semantic code replaces the AST code and
an executable GCC test confirming propagation into the enclosing `catch`.
Callable `RETURN` without a value now emits directly for typed procedures,
including byref copyback and the successful `bcc_result_void` wrapper for a
try-reachable procedure. A differential regression verifies semantic early
return replaces the aligned AST statement.
Top-level and callable `OPEN` now consumes a typed string path and literal
channel from semantic IR for every mode. Sequential `OUTPUT`/`APPEND` preserve
their `fopen` modes; `INPUT` raises error 53 and `RANDOM`/`BINARY` preserve
the `rb+`/`wb+` fallback and error 75 behavior through top-level raise-site or
try-reachable callable propagation. All modes register the C file-channel
table. Differential coverage verifies semantic paths in top-level and
callable scopes, including input raise state and random-mode fallback.
The generated semantic grammar now accepts the documented `SEEK #channel`
syntax. C semantic `SEEK` consumes a typed numeric record position and the
active `FIELD` width to seek to the corresponding 1-based record offset in
both top-level and callable bodies. Differential output checks both scopes,
and a GCC runtime test verifies writing after seeking record 2 starts at byte
offset 4 for a four-byte record.
Raw `FIELD` `GET`/`PUT` now consume typed literal channels and record-position
expressions in C top-level and callable bodies, using the active byte-buffer
helpers. Record/file DSL channels retain the legacy path because partial
updates carry `provided_fields` metadata absent from the current semantic IR.
A differential regression verifies semantic positions in both scopes. Raw
`FIELD` itself now registers its literal channel, typed string bindings, and
literal widths from semantic IR; those bindings are declared and feed the
record helpers. Differential coverage verifies semantic FIELD names and widths
replace the aligned AST layout.
String scalar assignment now also uses typed semantic nodes for string
literals, scalar names, parenthesized forms, and concatenation; concatenation
materializes into a C temporary buffer. Other string forms still decline per
node. Differential tests verify semantic literal precedence and concat
buffer emission, compound arithmetic, mixed string/numeric PRINT tokens, and
compatibility output shapes including string percents and numeric expression
parentheses. The semantic IR annotator now recursively types every `MID$`
assignment operand, including user-defined call results. Top-level `MID$`
assignment dispatches its typed target, start, optional length, and value from
semantic IR, sharing the callable path and its typed numeric/string expression
renderers. Differential regressions verify semantic indexes and string calls
replace AST values at top level and inside callable bodies. The full suite also caught and verified preservation of the
`DATE$` helper call. Typed semantic numeric expression emission now handles
float exponentiation with C `pow` and registers the math dependency; targets
without float support continue to decline or fail capability validation.
Differential coverage verifies semantic exponent operands replace the AST
assignment. One-dimensional array references are now distinguished
from callable invocations after DIM declarations are available and retain
their declared element type in semantic IR (explicit AS types, suffix types,
and the default Single type). C semantic numeric and string expression
renderers consume typed array reads for standard PRINT; scalar assignment and
`READ` now read or write one-dimensional array elements from semantic IR.
Plain `LPRINT` token lists also share the typed semantic print renderer;
formatted `USING` forms decline per node. Console `INPUT` for one typed
scalar or one-dimensional statically allocated array target now emits its
prompt and target-specific C conversion from semantic IR; channel input,
multiple targets, dynamic arrays, and unsupported target types fall back per
node. Differential regressions verify that semantic prompts, indexes, and
target conversions replace their AST counterparts.
Fractional numeric indexes retain the established round-away-from-zero conversion.
The typed IR now represents declared rank-matched array calls with multiple
indices as `MultiIndex`, preserving each typed index and the declared element
type. C semantic emission handles statically sized multi-rank numeric and
string reads/writes, `PRINT`, `INPUT`, and `READ`; aligned static `DIM` nodes
consume resolver-owned declarations rather than re-looking up names through
the legacy AST. JVM emission walks resolved nested-array shapes for reads,
writes, compound assignment, `INPUT`, and callable-local arrays. BASIC callable
assignment and reads retain the original multi-index argument list. Differential
coverage checks rank-two accesses in each backend. C multi-rank accesses now
also linearize runtime-sized arrays using their resolved per-axis lengths,
including fixed-width string element strides; an executable GCC regression
covers numeric and string rank-two reads and writes. Other C call expressions,
broader statement families, and nested control flow remain subsequent migration
work. A focused differential
regression now also covers compound assignment to a one-dimensional C array
element, checking that the semantic index and right-hand expression replace
the aligned AST target and value. C callable bodies now map
semantic statements against retained source coordinates and dispatch typed
assignments, scalar string `MID$` assignment, standard `PRINT`, plain `LPRINT`,
console `INPUT`, `READ`, `RESTORE`, `GOTO`, labels, `LOCATE`, `COLOR`,
`RANDOMIZE`, `CLS`, `BEEP`, and termination nodes independently; unsupported
nodes and formatted `LPRINT USING` continue through the AST emitter. Typed
one-dimensional local-array assignments, including compound arithmetic, and
`PRINT` reads use the callable's
resolved array shape, and console `INPUT` uses its local array table for target
validation. Numeric and string function `RETURN` values now emit from typed
semantic expressions, retaining result coercion, string result storage, and
by-reference scalar copyback. The callable `RETURN` renderer also receives the
resolved function and array tables, so supported nested numeric and
string-returning user calls use typed arguments and the caller-output-buffer
ABI instead of falling back to AST expressions. Callable `READ` uses semantic DATA items and
typed targets. A kind-changing regression confirms semantic-only callable
`INPUT` dispatch also registers its shared input helpers even when the AST has
an assignment at that position. Differential coverage verifies that semantic callable values,
indexes, prompts, `PRINT`/`LPRINT` tokens, screen/randomize operands,
DATA/READ operands, GOTO labels, label-derived RESTORE offsets, numeric/string
return values, terminal-control kinds, and `MID$` operands replace AST values.

The JVM backend now has its own all-or-nothing source-position mapper for
top-level statements, using the semantic module's per-statement source indexes.
Callable bodies now receive the same per-node alignment: the matching
semantic callable and its retained source identity are mapped against the
legacy function body before supported nodes dispatch, with unsupported or
ambiguous nodes retaining their AST emitter. A differential regression
confirms callable-local scalar assignment and typed `RETURN` expressions use
semantic values; return emission retains the JVM result descriptor and
by-reference writebacks.
Aligned termination and screen statements (`END`, `STOP`, `SYSTEM`, `CLS`,
`BEEP`) emit from semantic IR. Scalar string assignment also dispatches for
typed literal/name values; other assignment forms fall back individually to
the matched AST statement. Tests verify semantic-vs-AST precedence and that a
filename mismatch declines mapping. Typed numeric scalar assignment now
handles integer/radix/floating literals, numeric scalar names, unary minus,
arithmetic, true division, widening/narrowing conversions, and compound
`+=`, `-=`, `*=`, `/=` operators directly from semantic expressions. Integer
narrowing retains BASCAL's round-away-from-zero rule. Differential tests cover
semantic precedence, true division plus narrowing, and compound assignment.
Aligned top-level and callable `GOTO` and label nodes now emit JVM branches and
label symbols from semantic names, guarded by the set of labels in the aligned
semantic body. Missing semantic targets decline to the matching AST statement.
A differential regression changes both the target and definition names in the
semantic source and confirms the emitted branch resolves to the semantic label;
a callable regression verifies the same target validation and label naming in a
procedure body.
`ERROR` and valued `THROW` now dispatch typed numeric exception codes in both
top-level and callable JVM bodies, retaining the current `RuntimeException`
encoding and integer coercion; bare `THROW` still follows the existing
compatibility diagnostic path. A differential regression confirms semantic
arithmetic codes replace their AST counterparts; callable coverage separately
checks typed arithmetic codes for both exception statements. Random-access `OPEN` also
dispatches semantic path, channel, and record-length expressions through the
existing `RandomAccessFile`/record-buffer initialization sequence; other file
modes and missing lengths retain the AST path. Its differential regression
confirms semantic path, channel, and length values replace AST values.
`CLOSE` now dispatches its typed semantic channel expression to the existing
random-access file table and `RandomAccessFile.close()` operation in both
top-level and callable bodies. Its differential regression confirms the
semantic channel replaces the AST channel.
`KILL` dispatches typed semantic string paths through `java.io.File.delete()`
in top-level and callable bodies, declining non-string path forms back to AST.
Its differential regression confirms the semantic path takes precedence over
the aligned AST path. A callable regression now covers the combined random
`OPEN`, `CLOSE`, and `KILL` sequence, checking that semantic path, channel, and
record-length operands replace their AST counterparts.
One-dimensional numeric and string array reads in JVM semantic numeric/string
expressions now use the resolved array shape and element type; numeric indexes
retain the JVM backend's round-away-from-zero conversion. Simple assignment to
one-dimensional numeric or string array elements also emits from typed semantic
targets. Compound `+=`, `-=`, `*=`, and `/=` assignments to one-dimensional
numeric array elements now retain the semantic array reference and index while
loading the current element, applying typed numeric promotion and BASCAL's
rounding rules, and storing the result. Differential tests confirm semantic
indexes and values win over the aligned AST, including simple and compound
numeric local-array writes inside callable bodies. Callable local string-array
reads and assignments now also have differential coverage for semantic index
and value selection. Console `INPUT` for one
typed scalar or one-dimensional array
target also emits its prompt, terminal state transitions, typed conversion,
and storage from semantic IR; channel input and multiple targets decline per
node. Differential regressions confirm the semantic prompt, target index, and
target type take precedence. Plain `LPRINT` token lists share the typed
semantic stream renderer used by standard-output `PRINT`; formatted `USING`
forms decline per node. Callable bodies now have differential coverage for
both standard `PRINT` and plain `LPRINT` token streams. Its top-level
differential regression verifies semantic string,
numeric, and separator handling. Statement-form `MID$` assignment now dispatches
for a typed string scalar target, typed numeric start/optional length
expressions, and a typed string replacement. A shared JVM helper preserves
clamping, truncation, and splice behavior; the synthesized BASIC helper method
is omitted only when semantic dispatch emitted its replacement call. The JVM
conformance test assembles and runs a full source program, verifying that
`MID$(text$, 2, 2) = "XY"` changes `abcdef` to `aXYdef`. Unsupported semantic
forms retain the compatibility path. Callable console `INPUT` now also has
differential coverage for a one-dimensional local array target: its prompt,
index, and typed store are sourced from semantic IR rather than the aligned
AST. Callable-local array coverage now includes rank-two numeric writes,
reads, returns, and `INPUT`. Static multidimensional references dispatch from
typed `MultiIndex` nodes in top-level and callable JVM bodies. Calls, dynamic
arrays, and broader JVM statement families remain subsequent migration work.
Scalar `SWAP` now dispatches in top-level and callable JVM
bodies when both semantic operands are names with matching resolved storage
and retained semantic types; it emits load/store sequences directly, including
category-two numeric values, without scratch-local allocation. Array, field,
or mismatched operands retain the AST path. A differential regression confirms
semantic scalar names replace the AST pair; callable coverage now verifies
semantic local scalar values and matching typed operands feed the callable
swap sequence. Top-level `LOCATE` now emits its row and column from typed
semantic expressions, retaining JVM integer conversion and ANSI cursor
format. `COLOR` dispatches both literal and typed numeric semantic operands;
dynamic foreground/background values use generated palette lookup methods
that preserve the JVM backend's CGA-to-ANSI maps and modulo behavior. The
JVM conformance test assembles and runs dynamic foreground/background values
and confirms `COLOR 4, 1` emits the same red-on-blue sequence as the literal
path. Differential regressions confirm semantic coordinates and colors take
precedence over the aligned AST. Callable JVM bodies now use the same typed
`LOCATE` and `COLOR` dispatch, including row/column integer conversion and
the literal CGA-to-ANSI mapping; a differential regression verifies both
operations use callable semantic operands in place of their aligned AST
values. A second regression covers callable dynamic foreground lookup and
confirms semantic dispatch registers the required palette helper methods.
Callable string assignments, concatenation, and string return also have a
differential regression verifying semantic literals and `StringBuilder`
operations replace the aligned AST values.
Numeric comparisons now dispatch
from typed semantic expressions, including string equality/inequality;
`MOD`, integer division, bitwise `AND`/`OR`/`XOR`, and exponentiation also
use JVM-native operations with BASCAL's rounding and truth-value rules;
`&&` and `||` preserve short-circuit evaluation and normalize results to
BASIC `-1`/`0`. Double-valued semantic `PRINT` expressions use the same
six-significant-digit formatter as the AST path. Differential tests cover
comparison, quotient/remainder, bitwise AND, power, and short-circuiting.
Scalar string assignment now handles semantic
concatenation with typed `StringBuilder.append` overloads for string and
numeric operands. Unsupported call/array operands still decline per node. A differential test covers
semantic literal assignment and concatenation. Standard-output `PRINT` now
dispatches typed semantic string/numeric tokens with trailing separator
flushes and newline selection preserved; file/printer destinations, `USING`,
and `TAB`/`SPC` expressions still use the aligned AST statement when present.
A differential regression covers semantic values and both newline and
trailing-separator behavior. Semantic string names declared by `FIELD` use
the JVM record-buffer decoder so fixed-width padding remains observable; the
cross-process random-access record round-trip test verifies this path.

The BASIC, C, and JVM backends now derive whether to append an implicit
top-level terminator from `SemanticModule::ends_with_end` whenever typed IR
is present. BASIC uses this fact in both its ordinary output path and the
source-file lookup path; legacy AST termination remains the compatibility
fallback in all three backends. A focused regression checks semantic modules
with and without explicit `END`.

C's compile-time `READ`-without-`DATA` guard now queries the semantic module's
nested statement tree whenever typed IR is present. AST scanning remains the
fallback for compatibility callers without a semantic module, keeping the
diagnostic predicate aligned with the same resolved statement source used to
collect DATA items.

C's callable-body `FIELD` prepass now consumes explicit typed layout facts
from `SemanticModule::lowered_record_files` for record/file DSL declarations.
`records::lower` produces each file's allocated channel, owner, synthesized
buffer bindings, fixed widths, field kinds, and string alignment; the driver
attaches those facts to the semantic module. Callable `GET`/`PUT` generation
therefore has the needed layout before top-level statements are emitted. Raw
semantic `FIELD` statements and desugared record-file layouts now share one
prepass ordered by semantic source index and span offset, preserving
re-`FIELD` generations when both forms target the same channel. The AST
prepass remains for AST-only callers and manually constructed semantic modules
that have not been paired with record-lowering metadata. Tests verify producer
facts, mixed raw/DSL field ordering, raw field dispatch, and record-file
procedure access.
C's global scalar declaration pass also removes FIELD names gathered from the
compatibility AST, then declares raw semantic FIELD bindings and synthesized
record-file buffers from typed IR. A differential regression verifies that a
stale AST buffer is absent when the semantic FIELD binding has a different
name. Callable-local C declarations now apply the same rule when an aligned
semantic FIELD or record-lowering layout is available; a differential test
checks the local buffer name and width used by a procedure.
C's callable try-result ABI prepass now decides whether a callable owns a
`TRY` block or contains a direct semantic raise (`THROW` or a raising `OPEN`)
from its matched typed callable body. AST scans remain the fallback when a
callable cannot be matched to semantic IR. A differential regression verifies
that semantic `TRY`/`THROW` selects the result-wrapper ABI when the paired AST
body has neither construct. TRY-body seeds and transitive callable edges now
come from semantic statement trees and typed call expressions whenever the
matched semantic body is available; AST discovery remains the compatibility
fallback only when a callable cannot be matched to typed IR or no semantic
module is available. Incomplete/unsupported semantic statements no longer trigger AST
edge supplementation, which could otherwise add stale calls and change ABI
selection. Differential regressions verify both semantic transitive
propagation and rejection of an AST-only stale call edge. Semantic record
method calls now contribute typed receiver/method edges. The typed receiver
comes from declared record variables or an unambiguous complete record literal;
the C prepass maps that source identity to the callable produced by
`records::lower`. A regression verifies transitive result-ABI propagation
through a record method invoked on a `LET`-initialized record value.

Semantic expression annotation now assigns a user-defined scalar method call
the declared result type from its callable signature, using the typed receiver
type to select among same-named methods. Methods with an omitted return type
inherit the receiver type. A focused typed-IR regression covers string and
integer method results. Name expressions now also recover explicit scalar
`DIM ... AS <type>` annotations from module-scope declarations, allowing the
same method lookup on typed variables as on literals. Unsuffixed `CONST`
names now recover their initializer-derived type from
`SemanticModule::const_types` as well, so they can serve as typed scalar
receivers. Record-valued receivers still need an explicit type in
`SemanticValueType` before this lookup can cover record methods.
Initializer-derived constant types now follow references to earlier
constants, so string aliases stay strings in typed IR. Unsuffixed numeric
constants use BASCAL's default single-precision type, while an explicit `#`
retains double precision. The BASIC differential compatibility fixture now
asserts the semantic string alias output directly: the AST-only path inferred
that unsuffixed alias as integer, so equality with that stale type would
encode the wrong contract.
Type annotation consults the aggregate constant type facts because resolver
semantics require CONST names to be unique program-wide even when a declaration
appears inside a callable. The separate `top_level_const_types()` query lets
backend module-declaration passes select only module-scope declarations; the
aggregate map remains available to callable codegen for constants declared in
callable bodies. A regression verifies the distinct aggregate and module-scope
fact sets.
Constant type inference now iterates through program-wide `CONST` declarations
before applying the integer default, so aliases can refer forward to later
string or numeric constants without losing their initializer-derived type.
The same inference now respects binary operator result categories while
resolving forward references: comparisons produce Boolean, Boolean logical
operands retain Boolean, string `+` remains string concatenation, and numeric
operators preserve/promote operand types. Regressions cover forward string,
numeric, arithmetic, and comparison initializers; unresolved cycles still
reach the existing integer fallback.
The operator result rule is shared with ordinary semantic expression
annotation, replacing the previous unconditional propagation of the left
operand's type. Focused coverage verifies concatenation, mixed numeric
promotion, comparison results, and integer division's floating-point type.
Typed-IR expressions now carry nominal record identity separately from their
scalar `value_type`. Names and array elements recover the identity from
`DIM ... AS Record`; a uniquely shaped complete record literal also types its
assigned variable. Record fields propagate nested record identities, and
record literals receive their nominal type from an assignment target, including
nested record-valued fields. Scalar member types resolve through nested fields
and retained `combines` relationships. Both generated access shapes are
covered: array-rooted member nodes and dotted names such as `person.age`, with
callable-local `DIM ... AS Record` declarations and multidimensional record
arrays included in annotation.
Fluent record method calls with an omitted result annotation inherit the
receiver's nominal record type. Record-typed callable parameters and explicit
record return annotations remain unrepresented in callable signatures, which
currently retain scalar suffix types only.

The C semantic numeric renderer now declines dotted `Name` expressions.
These nodes can denote record fields, which the semantic scalar renderer does
not yet transpile; declining keeps them on the record-aware compatibility
path instead of emitting invalid C member access. A record-method mutation
regression caught this dispatch gap. The focused test and full
`cargo test --locked --offline` suite pass with the guard in place. C's typed
record-method TRY-edge work also passes positive transitive propagation and
stale-AST-edge regressions.

BASIC callable emission now directly handles typed `LPRINT USING`, `LINE
INPUT`, `WRITE`, and `CLOSE` nodes. Standard, formatted, and channel
`PRINT` destinations now all dispatch from typed callable statements. Random
access `GET`/`PUT` with typed channel, position, and record operands also
dispatch directly. These cases render expressions with the
callable's allocated local and parameter names, and decline to AST emission
only when typed expression rendering is unsupported. Differential callable
fixtures use different AST and semantic statements at aligned source
positions, verifying that semantic file and printer operations determine the
output. Callable emission also directly handles typed `CLS`, `BEEP`, `CLEAR`,
`STOP`, and `SYSTEM`; a differential callable fixture verifies semantic
precedence for this terminal family. `ERROR` and bare or valued `THROW` now
also emit directly from callable typed IR, with the same semantic-over-AST
fixture checking valued and bare error forms. File lifecycle and console
operations (`OPEN`, `SEEK`, `KILL`, `NAME`, `WIDTH`, `LOCATE`, and `COLOR`) and
machine operations (`RANDOMIZE`, `SWAP`, `POKE`, `OUT`, `LSET`, `RSET`, and
`ERASE`) also dispatch from typed callable bodies, with distinct aligned-AST
fixtures. Callable `GOTO`, `GOSUB`, `RESUME`, `RESTORE`, labels, and
`ON ERROR GOTO` now use the existing resolved label and callable-entry map;
the regression checks numeric target resolution after BASIC line numbering.
Aligned callable semantic statements now recurse through block `IF`, `FOR`,
`WHILE`, `DO`, and `SELECT CASE` bodies. Loop exit and continue stacks are
scoped while rendering nested bodies; a declined node restores label and loop
state before AST fallback. Differential callable tests verify semantic branch
and loop bodies, typed case conditions, and transfer targets. Focused tests
pass. A serial offline workspace run passes all 774
library tests, 19 CLI tests, and the integration suites (25 examples,
34 JVM conformance, 1 language conformance, 11 general record, and 38 record
method tests). A parallel examples run had a transient `Text file busy`
failure for the generated `remline` binary; that test passed alone and the
serial full run passed.

The generated grammar previously let repeated `case_clause` parsing consume
`CASE ELSE` as an identifier-valued case, leaving typed `else_body` empty.
The grammar now parses `CASE ELSE` as a separate `CaseClause::Else` variant,
and the semantic adapter places that body in
`SemanticStatementKind::SelectCase::else_body`. A parser/adapter regression
checks that only ordinary clauses populate `cases`. This restored the
`ERROR$` fallback branch; FreeBASIC and real BASCOM stdlib conformance checks
pass, followed by the full serial offline workspace suite.

Source-file markers are now also emitted when callable statements dispatch
from typed IR. Multi-file integration coverage found that the callable
dispatcher otherwise left the root module's marker active while emitting a
required library procedure, causing the `ERL` to source-file boundary chain
to collapse to the root filename. The helper callable now switches markers at
its aligned source location; the focused boundary-chain regression and the
serial full workspace run pass.

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

Typed BASIC `MID$` assignment now dispatches through semantic IR at both
module and callable scope. Both the typed path and legacy AST fallback emit
the splice inline with `LEFT$` and `MID$`; post-parse AST lowering no longer
injects the `midAssign$` library function. A dedicated regression verifies
that the AST path emits inline without helper injection. The typed path emits
the splice directly with `LEFT$` and `MID$`,
snapshots the target value and operands in source order, and computes the
omitted length from the snapshotted replacement. The semantic expression
renderer emits declared scalar calls with typed result temporaries. Calls now
support by-value arguments and plain-name by-reference arguments, including
recursive call expressions; argument values are snapshotted before the outer
callee's parameter slots are set, and by-reference values copy back after the
call. Inline builtin calls also execute at their source position. Array-element targets
capture every subscript before reading the target value and reuse the captured
lvalue for the write-back, including a subscript function call. The semantic
emitter now supports whole-array call arguments for typed array parameters,
including rank checks, bound propagation, capacity guards, array copy-in, and
ByRef array copy-back. BASIC typed expressions now emit
scalar record-field reads from semantic member nodes and dotted typed names,
using the annotated value type for the storage suffix. Typed top-level record
backing fields now enter each callable's global-name set, while callable-local
fields retain their allocated local names. A source-aligned module/callable
differential covers field writes, top-level reads, and local reads. AST/semantic
differentials cover module
and callable dispatch, and the callable ordering regression checks nested
by-reference `offset%()` calls, `position%()`, `ABS()`, `RND()`,
`replacement$()`, and the inline splice. The 62-test
BASIC codegen group passes, as do the five-test
`cargo test --offline semantic_mid_assign` filter and two dedicated scalar-call
ordering tests, plus the dynamic-index single-evaluation regression. The
legacy AST fallback has now also been verified to emit inline without helper
injection.
The semantic inline path passes the FreeBASIC edge-case runtime test plus
dedicated FreeBASIC tests for a ByRef operand and a dynamic array target index.
The latter confirms the index call executes exactly once. The shared BASCOM/C
conformance fixture remains unchanged and continues to pass on both backends.
Full serialized `cargo test --locked --offline -- --test-threads=1` passes
(782 library tests, 19 CLI tests, 11 DOSBox conformance, 28 examples, 34 JVM
conformance, 1 language conformance, 11 general record, and 38 record method
tests; doc-tests also pass).
Graphify code-only extraction completed at 3,660 nodes and 10,992 edges; full
`graphify .` document extraction needs an LLM API key, which is not configured.
Typed array-bound constant initializers now accept parenthesized array
designators and literal axes.
Their source-aligned module and callable regressions also exposed and fixed a
CONST initializer lookup mismatch: the BASIC compatibility name can carry a
type suffix while typed IR records the source name without one. The semantic
initializer lookup now checks both normalized forms, preventing AST axis
selection from silently replacing the typed expression.
At that checkpoint, the remaining MID$ follow-up was to verify composition
with array-bound intrinsics and callable expressions. Record-valued expressions
remain outside the scalar string operand contract. The serialized workspace run
at that checkpoint passed with 782 library tests,
19 CLI tests, 11 DOSBox conformance, 28 examples, 34 JVM conformance, 1 language
conformance, 11 general record, and 38 record method tests; doc-tests also pass.

The AST `MID$` compatibility path now also emits the splice inline. Its target,
start, optional length, and replacement values are snapshotted in evaluation
order; generated labels share the typed emitter's unique label format. The
post-parse lowering pass no longer scans for `MID$` statements or injects
`com.bascal.stdlib.midAssign`; explicit `require` of the library function
remains available. A direct AST codegen regression verifies inline output and
absence of helper injection. The BASIC codegen group passes (63 tests), and
the serialized full offline workspace suite passes with 784 library tests,
19 CLI tests, 11 DOSBox conformance, 28 examples, 34 JVM conformance, 1
language conformance, 11 general record, 38 record method tests, and doc-tests.
Graphify was refreshed after the stage. Record-valued expressions remain
outside the scalar string contract of MID$ assignment; scalar string fields
are covered directly.
Typed scalar string record members are no longer rejected as a blanket class:
the BASIC semantic MID$ emitter now accepts a simple typed member as its
lvalue and scalar operand, using the existing typed member-to-storage mapping.
A module differential verifies write-back to the typed backing-field name.
A callable differential also verifies write-back through the allocated local
record-field name after source-aligned dispatch.
A follow-up regression also covers SIZEOF, LBOUND, and UBOUND results combined
with a nested callable in the MID$ start expression, verifying typed bounds,
call ordering, and splice emission without relying on AST fallback. Intrinsic
array designators and axes continue to follow the existing typed array-name/
literal-axis contract.
The AST fallback ordering regression also checks a side-effecting array
target index, start function, replacement function, and the omitted-length
form. These calls are emitted once in source order, and the default length
uses LEN of the captured replacement. The serialized workspace suite now
passes with 785 library tests and the same integration and doc-test counts.

The next C backend slice extends typed `INPUT #` targets from scalar names to
semantic lvalues, including indexed array elements. The file-input emitter
now uses the resolved target type, rank, indices, and array table for numeric
and string conversion/storage in both module and callable scopes. Differential
regressions verify that a semantic channel and array index take precedence
over the aligned AST target. Focused `semantic_file_input` tests pass. The
serialized offline workspace suite passes after the slice (787 library tests,
19 CLI tests, 11 DOSBox conformance, 28 examples, 34 JVM conformance, 1
language conformance, 11 general record, 38 record method tests, and
doc-tests).

The next JVM backend slice adds typed block `IF` dispatch at module and
callable scope. Conditions use semantic numeric types and branch operands;
supported nested bodies emit typed assignments, `MID$`, standard `PRINT`,
and plain `LPRINT`, including recursively nested `IF` nodes. Each block is
rendered transactionally, so an unsupported child declines the complete block
before bytecode is appended and the aligned AST statement remains available
for compatibility. Differential tests cover semantic condition and both
branches in module and callable bodies; a dedicated regression verifies
atomic fallback on a nested unsupported transfer. The assembled JVM runtime
regression passes, as does the focused `block_if` test group.
The serialized full offline workspace run passes with 790 library tests, 19
CLI tests, 11 DOSBox conformance tests, 29 examples, 35 JVM conformance tests,
1 language conformance test, 11 general record tests, 38 record method tests,
and doc-tests. The C `INPUT #` array path also has an executable GCC regression
that reads numeric and string fields into indexed array elements and verifies
their generated runtime values.

The next JVM control-flow slice adds typed `WHILE` dispatch at module and
callable scope. The typed condition is reevaluated at the loop header, and the
loop body uses the same nested semantic-statement walker as block `IF`.
Supported bodies are staged transactionally; unsupported constructs decline
before any loop bytecode is committed. `EXIT` and `CONTINUE` use the innermost
typed loop context, including through nested semantic `IF` nodes. Differential
tests verify semantic bounds and body operands in module and callable scopes,
and a source-aligned regression verifies that semantic loop transfers replace
AST assignments. JVM runtime regressions execute a three-iteration loop and a
loop that continues at 2 and exits at 4. Focused `semantic_while` tests pass.
The serialized offline workspace suite passes with 793 library tests, 19 CLI
tests, 11 DOSBox conformance tests, 29 examples, 37 JVM conformance tests, 1
language conformance test, 11 general record tests, 38 record method tests,
and doc-tests.

The next JVM control-flow slice dispatches typed counted `FOR` loops from
module and callable bodies. The emitter consumes the semantic loop variable,
start expression, bound, and direction; supported nested statements use the
same transactional block renderer as typed `IF` and `WHILE`. `EXIT` branches
to the loop end and `CONTINUE` branches to the increment, including when nested
under semantic `IF`. Integer loop variables and literal nonzero steps preserve
the JVM backend's existing capability boundary. Differential tests verify
semantic bounds override AST bounds in module and callable scopes. The JVM
runtime regression executes ascending and descending loops with nested
`CONTINUE` and `EXIT`. Focused `jvm_semantic_for` tests and the serialized
offline workspace suite pass (794 library tests, 19 CLI tests, 11 DOSBox
conformance tests, 29 examples, 38 JVM conformance tests, 1 language
conformance test, 11 general record tests, 38 record method tests, and
doc-tests). Graphify was refreshed after the backend dispatch change.

The next JVM control-flow slice dispatches typed `DO` loops in module and
callable bodies. Precondition expressions are evaluated at the loop header;
postcondition expressions are evaluated after the body and the continue label.
`WHILE` branches to the loop end on false and `UNTIL` branches on true. Nested
`EXIT` targets the innermost loop end, while `CONTINUE` reaches the postcondition
check (or the next header when there is no postcondition). Unsupported bodies
decline transactionally and restore label state before compatibility fallback.
Differential tests cover typed pre- and postconditions in module scope and a
typed precondition in callable scope. The JVM runtime regression executes both
condition positions with nested `EXIT` and `CONTINUE`. Focused
`jvm_semantic_do` tests and the serialized offline workspace suite pass (795
library tests, 19 CLI tests, 11 DOSBox conformance tests, 29 examples, 39 JVM
conformance tests, 1 language conformance test, 11 general record tests, 38
record method tests, and doc-tests). Graphify was refreshed after the backend
dispatch change.

The next JVM statement slice dispatches typed `SELECT CASE` in module and
callable bodies. It consumes the semantic selector, equality values, numeric
ranges, comparison patterns, `CASE ELSE` body, and typed clause bodies. Integer
selectors and string equality retain the JVM AST backend's supported pattern
boundary. Clause bodies reuse the nested semantic block renderer, including
active loop-transfer contexts. Rendering is transactional: an unsupported
pattern or child declines the whole SELECT before appending bytecode or
retaining allocated labels. Differential tests verify semantic pattern and
body precedence in module and callable scopes, and verify AST fallback after a
case child declines. The JVM runtime regression executes numeric equality,
range, comparison, and string equality clauses. Focused `jvm_semantic_select`
tests and the serialized offline workspace suite pass (796 library tests, 19
CLI tests, 11 DOSBox conformance tests, 29 examples, 40 JVM conformance tests,
1 language conformance test, 11 general record tests, 38 record method tests,
and doc-tests). Graphify was refreshed after the backend dispatch change.

The next JVM semantic-block slice dispatches `SWAP`, `CLS`, `BEEP`, `STOP`,
`SYSTEM`, `ERROR`, and valued `THROW` inside nested control-flow bodies. These
nodes use the existing typed swap, numeric error, and terminal emitters, so
their operands and effects remain sourced from semantic IR. Differential
coverage checks semantic values in module blocks and a callable SWAP inside
typed `IF`; bytecode assertions cover all added statement families. An
assembled JVM runtime regression confirms that nested typed SWAP mutates both
values. The serialized offline workspace suite passes with 797 library tests,
19 CLI tests, 11 DOSBox conformance tests, 29 examples, 41 JVM conformance
tests, 1 language conformance test, 11 general record tests, 38 record method
tests, and doc-tests. Graphify was refreshed after extending nested semantic
statement dispatch.

The next JVM callable slice dispatches typed `RETURN` nodes inside semantic
control-flow blocks. `JvmContext` now carries the callable result type and
void-return contract into nested `IF`, loop, and `SELECT CASE` bodies; return
expressions use semantic type information, and ByRef scalar write-backs run
before every nested return. Differential coverage verifies semantic THEN and
ELSE return values and bare procedure return behavior. An assembled JVM
runtime regression verifies a nested typed return writes its updated ByRef
scalar back to the caller. The serialized offline workspace suite passes
with 798 library tests, 19 CLI tests, 11 DOSBox conformance tests, 29 examples,
43 JVM conformance tests, 1 language conformance test, 11 general record tests,
38 record method tests, and doc-tests. Graphify was refreshed after extending
the JVM callable semantic context.

The next JVM semantic-block slice dispatches typed `LOCATE` and `COLOR` inside
nested control-flow bodies. Both operations use the existing semantic numeric
expression and target-escape emitters, preserving typed coordinates and the
CGA-to-ANSI palette mapping in module and callable scopes. Differential tests
verify semantic row, column, foreground, and background values override their
aligned AST values in both scopes. An assembled JVM runtime regression checks
the emitted cursor-position and color escape sequences inside typed `IF`. The
serialized offline workspace suite passes with 799 library tests, 19 CLI
tests, 11 DOSBox conformance tests, 29 examples, 44 JVM conformance tests, 1
language conformance test, 11 general record tests, 38 record method tests,
and doc-tests. Graphify was refreshed after extending nested semantic
statement dispatch.

The next JVM semantic-block slice dispatches typed console `INPUT` inside
nested control-flow bodies. It consumes the typed prompt and single scalar or
indexed-array target using the existing semantic INPUT emitter; unsupported
channel and multi-target forms still decline to the aligned AST path. Module
and callable differential tests verify semantic prompts override AST prompts.
An assembled JVM runtime regression supplies piped console input inside typed
`IF` and verifies the typed target receives the parsed value. The serialized
offline workspace suite passes with 800 library tests, 19 CLI tests, 11 DOSBox
conformance tests, 29 examples, 45 JVM conformance tests, 1 language
conformance test, 11 general record tests, 38 record method tests, and
doc-tests. Graphify was refreshed after extending nested semantic statement
dispatch.

The next JVM semantic-block slice dispatches typed random-file `OPEN`, `CLOSE`,
and `KILL` operations inside nested control-flow bodies. Module and callable
differential coverage verifies the typed path, channel, and record length
override their aligned AST values. An assembled JVM runtime regression opens a
random file inside typed `IF`, closes and kills it, then verifies the file was
removed. The serialized offline workspace suite passes with 801 library tests,
19 CLI tests, 11 DOSBox conformance tests, 29 examples, 45 JVM conformance
tests, 1 language conformance test, 11 general record tests, 38 record method
tests, and doc-tests. Graphify was refreshed after extending nested semantic
statement dispatch.

The next JVM semantic-block slice dispatches typed `LSET` and `RSET` inside
nested control-flow bodies. It resolves each target through the JVM backend's
semantic `FIELD` layout and emits padding and byte-buffer writes from typed
string expressions. Differential coverage checks module and callable scopes;
an assembled runtime regression verifies left/right padding survives `PUT` and
`GET`. The serialized offline workspace suite passes with 802 library tests,
19 CLI tests, 11 DOSBox conformance tests, 29 examples, 46 JVM conformance
tests, 1 language conformance test, 11 general record tests, 38 record method
tests, and doc-tests. Graphify was refreshed after extending nested semantic
statement dispatch.

The next JVM semantic-block slice dispatches typed random-record `GET` and
`PUT` with explicit record positions inside nested control-flow bodies. The
emitter evaluates the typed channel and position, seeks using the resolved
record length, and transfers bytes through the JVM record buffer. Module and
callable differential tests verify typed positions override AST positions; an
assembled JVM regression reads, updates, writes, and rereads a nested record.
The serialized offline workspace suite passes with 803 library tests, 19 CLI
tests, 11 DOSBox conformance tests, 29 examples, 47 JVM conformance tests, 1
language conformance test, 11 general record tests, 38 record method tests,
and doc-tests. Graphify was refreshed after extending nested semantic
statement dispatch.

The next JVM semantic-file slice dispatches typed `SEEK` at module, callable,
and nested-block scope. It computes the byte offset from the typed record
position and channel record length and invokes `RandomAccessFile.seek`. Module,
callable, and nested-block differential coverage verifies semantic positions
override AST positions; an assembled JVM regression executes both top-level
and nested SEEK statements. The serialized offline workspace suite passes
with 804 library tests, 19 CLI tests, 11 DOSBox conformance tests, 29 examples,
48 JVM conformance tests, 1 language conformance test, 11 general record
tests, 38 record method tests, and doc-tests. Graphify was refreshed after
extending semantic JVM file-operation dispatch.

The next JVM semantic-file slice dispatches typed `NAME`/rename at module,
callable, and nested-block scope. The emitter constructs source and destination
`File` values from typed string expressions and invokes `File.renameTo`.
Differential tests verify typed paths override AST paths; an assembled JVM
regression confirms a file created by random `OPEN` moves to its typed
destination inside `IF`. The serialized offline workspace suite passes with
805 library tests, 19 CLI tests, 11 DOSBox conformance tests, 29 examples, 49
JVM conformance tests, 1 language conformance test, 11 general record tests,
38 record method tests, and doc-tests. Graphify was refreshed after extending
semantic JVM file-operation dispatch.

The next JVM semantic-block slice consumes typed `FIELD` declarations in
module and callable control-flow bodies. `FIELD` emits no runtime bytecode;
its channel, field names, widths, and offsets are collected from semantic IR
before codegen. Differential tests verify typed names and widths override AST
declarations, and an assembled JVM regression round-trips values through a
nested `FIELD`/`LSET`/`RSET` block. The serialized offline workspace suite
passes with 806 library tests, 19 CLI tests, 11 DOSBox conformance tests, 29
examples, 50 JVM conformance tests, 1 language conformance test, 11 general
record tests, 38 record method tests, and doc-tests. Graphify was refreshed
after extending nested semantic statement dispatch.

The next JVM semantic-file slice dispatches typed channel `LINE INPUT` at
module, callable, and nested-block scope. Literal channel numbers address the
existing random-file table; typed scalar and indexed string targets receive
`RandomAccessFile.readLine()` results, with EOF normalized to an empty string.
Differential tests verify typed channels and array indices override AST
operands, and an assembled JVM regression reads a prepopulated line inside
typed `IF`. The serialized offline workspace suite passes with 807 library
tests, 19 CLI tests, 11 DOSBox conformance tests, 29 examples, 51 JVM
conformance tests, 1 language conformance test, 11 general record tests, 38
record method tests, and doc-tests. Graphify was refreshed after extending
nested semantic file-operation dispatch.

The next JVM semantic-file slice dispatches typed channel `PRINT` at module,
callable, and nested-block scope. Literal random-file channels select the file
table entry; typed string and numeric expressions emit with the existing JVM
string conversion conventions. A trailing semicolon suppresses the line feed;
channel `USING` and comma-zone formatting remain declined for fallback.
Differential tests verify typed channels and values override AST operands, and
an assembled JVM regression verifies exact file bytes after a typed `SEEK`.
The serialized offline workspace suite passes. Graphify was refreshed after
extending semantic JVM file-operation dispatch.

The next JVM semantic-file slice accepts comma separators in typed channel
`PRINT`. Commas and semicolons separate adjacent typed values without adding
formatting bytes; either trailing separator suppresses the newline, matching
the existing typed console and C channel renderers. An assembled JVM regression
checks concatenated comma-separated values and a semicolon-terminated value in
the same random file. The serialized offline workspace suite passes with 808
library tests, 19 CLI tests, 11 DOSBox conformance tests, 29 examples, 52 JVM
conformance tests, 1 language conformance test, 11 general record tests, 38
record method tests, and doc-tests. Graphify was refreshed after extending
typed JVM channel-print dispatch.

The next JVM semantic-file slice accepts typed numeric channel expressions in
`PRINT #`. The channel expression is evaluated once, converted to a JVM array
index, and the selected `RandomAccessFile` receiver is retained on the operand
stack while every typed token and optional newline is written. Differential
coverage verifies a typed channel variable overrides the AST literal and is
loaded once per statement; the assembled JVM regression writes exact bytes
through that variable channel. The serialized offline workspace suite passes
with 808 library tests, 19 CLI tests, 11 DOSBox conformance tests, 29 examples,
52 JVM conformance tests, 1 language conformance test, 11 general record
tests, 38 record method tests, and doc-tests. Graphify refreshed to 3614 nodes,
11302 edges, and 136 communities after extending typed JVM channel printing.

The next JVM semantic-file slice accepts typed numeric channel expressions in
`LINE INPUT #`, in addition to literal channels. The emitter evaluates the
channel expression, converts it to the random-file table index, reads one line,
normalizes EOF to the empty string, and stores through the typed scalar or
array target. Differential coverage verifies a semantic channel variable and
typed array index override AST operands; an assembled nested-block regression
reads through a channel variable at runtime. The serialized offline workspace
suite passes with 808 library tests, 19 CLI tests, 11 DOSBox conformance tests,
29 examples, 52 JVM conformance tests, 1 language conformance test, 11 general
record tests, 38 record method tests, and doc-tests. Graphify refreshed to 3614
nodes, 11304 edges, and 141 communities after extending semantic JVM file
input.

The next JVM typed-file slice evaluates a semantic random-`OPEN` channel once
and reuses its computed index for `bccFiles`, `bccRecLen`, and `bccBufs`. The
index remains on the operand stack across the three stores, avoiding a
backend-local semantic re-evaluation. Typed numeric expressions now also emit
calls to scalar numeric-returning functions when signatures and argument
types match; unsupported receiver, array, and by-reference signatures decline
to the compatibility path. An assembled JVM regression uses a side-effecting
channel selector and confirms it executes exactly once. The serialized offline
workspace suite passes with 808 library tests, 19 CLI tests, 11 DOSBox
conformance tests, 29 examples, 53 JVM conformance tests, 1 language
conformance test, 11 general record tests, 38 record method tests, and
doc-tests. Graphify refreshed to 3615 nodes, 11311 edges, and 135 communities.

The next JVM typed-expression slice emits calls to scalar string-returning
functions from semantic IR, sharing signature validation and argument
transpilation with numeric-returning function calls. Calls with receiver,
array, or by-reference signatures retain compatibility fallback. Differential
coverage verifies that a typed string function call and its typed callable
return replace AST literals; an assembled channel `PRINT` regression writes
the returned string to a random file. The serialized offline workspace suite
passes with 809 library tests, 19 CLI tests, 11 DOSBox conformance tests, 29
examples, 53 JVM conformance tests, 1 language conformance test, 11 general
record tests, 38 record method tests, and doc-tests. Graphify refreshed to 3617
nodes, 11321 edges, and 138 communities.

The next JVM typed-call slice supports scalar `byref` arguments in semantic
function calls when each actual is a plain resolved variable. The codegen
backend seeds caller scratch arrays from typed lvalues, passes those arrays
using the resolved callable descriptor, and writes mutated values back after
the invocation. Receiver, array, and non-name `byref` actuals retain
compatibility fallback. Differential coverage verifies that semantic lvalue
selection and callable-body values override AST operands; assembled JVM
by-reference regressions verify caller-visible mutation. The serialized offline
workspace suite passes with 810 library tests, 19 CLI tests, 11 DOSBox
conformance tests, 29 examples, 53 JVM conformance tests, 1 language
conformance test, 11 general record tests, 38 record method tests, and
doc-tests. Graphify refreshed to 3618 nodes, 11327 edges, and 138 communities.

The next JVM typed-call slice passes resolved array arguments to semantic
function calls, preserving array rank and element type from the JVM callable
signature and caller array table. Callable array parameters now contribute
their rank and suffix-derived element type during semantic-IR annotation, so
subscript syntax becomes a typed `Index` node rather than an unknown `Call`.
Differential tests verify a typed array actual and callable body override AST
operands; a JVM runtime regression verifies a `byref` array mutation reaches
the caller. The existing by-value array assembly expected-failure test remains
unchanged. The serialized offline workspace suite passes with 812 library
tests, 19 CLI tests, 11 DOSBox conformance tests, 29 examples, 54 JVM
conformance tests, 1 language conformance test, 11 general record tests, 38
record method tests, and doc-tests. Graphify refreshed to 3621 nodes, 11336
edges, and 143 communities.

Iteration 27 adds JVM by-value array copying to semantic function calls and
repairs the JVM array-copy helper's `tableswitch` default-label syntax so the
helper assembles correctly. The by-value procedure fixture now runs and
confirms mutations stay within the copied array; an assembled semantic
function-call regression confirms the typed array actual is copied and the
caller retains its original value. Focused JVM regressions pass. The serialized
offline workspace suite passes with 812 library tests, 19 CLI tests, 11 DOSBox
conformance tests, 29 examples, 55 JVM conformance tests, 1 language
conformance test, 11 general record tests, 38 record method tests, and
doc-tests. Graphify refreshed to 3622 nodes, 11340 edges, and 142 communities.

Iteration 28 completes the JVM by-value array copy path across primitive and
reference element types and multidimensional arrays. The JVM copy helper now
emits verifier frames for its branch targets, and callable prologues cast
copied array parameters using the resolved element descriptor instead of
assuming integer arrays. An assembled semantic-IR regression exercises a
rank-two integer array and a string array, confirming that both calls mutate
their private copies while the caller arrays retain their original values.
Focused JVM regressions and the serialized offline workspace suite pass with
812 library tests, 19 CLI tests, 11 DOSBox conformance tests, 29 examples,
55 JVM conformance tests, 1 language conformance test, 11 general record
tests, 38 record method tests, and doc-tests. Graphify reports no topology
changes and remains at 3622 nodes, 11340 edges, and 142 communities.

Iteration 29 aligns JVM array-copy helper injection with emitted typed-callable
prologues. The backend now includes `bccCopyArray` whenever generated module
or callable code invokes it, so semantic `ByVal` array metadata remains
authoritative even when the compatibility AST marks the same parameter
`ByRef`. A differential JVM codegen regression reproduces the AST/typed-IR
mode mismatch and verifies the required helper is present. Focused validation
and the serialized offline workspace suite pass with 813 library tests, 19
CLI tests, 11 DOSBox conformance tests, 29 examples, 55 JVM conformance tests,
1 language conformance test, 11 general record tests, 38 record method tests,
and doc-tests. Graphify refreshed to 3623 nodes, 11342 edges, and 136
communities.

Iteration 30 makes JVM callable parameter slots and array shapes consume the
resolved `FunctionSig` metadata. Array rank and element type, scalar parameter
types, and scalar `ByRef` positions now determine the JVM local layout rather
than being re-inferred from AST axes, suffixes, or passing modes. A
differential codegen regression changes an AST scalar parameter into a typed
array parameter and verifies that the typed callable receives an array slot,
emits the by-value copy prologue, and can access the array element. Focused
validation and the serialized offline workspace suite pass with 814 library
tests, 19 CLI tests, 11 DOSBox conformance tests, 29 examples, 55 JVM
conformance tests, 1 language conformance test, 11 general record tests, 38
record method tests, and doc-tests. Graphify refreshed to 3624 nodes, 11344
edges, and 135 communities.

Iteration 31 carries resolved callable parameter identifiers into the JVM
function signature and uses them for local scalar bindings, array slots, and
array shapes. This keeps callable body name resolution aligned with typed IR
when compatibility AST parameter names differ. A differential codegen
regression renames an array parameter in the semantic callable and verifies
that the typed array mutation and return use the resolved binding. Focused
validation and the serialized offline workspace suite pass with 814 library
tests, 19 CLI tests, 11 DOSBox conformance tests, 29 examples, 55 JVM
conformance tests, 1 language conformance test, 11 general record tests, 38
record method tests, and doc-tests. Graphify refreshed to 3624 nodes, 11345
edges, and 135 communities.

Iteration 32 makes scalar JVM receiver types and the `self` binding consume
the semantic callable's receiver annotation. Exact receiver matches continue
to select among overloads; a unique same-name callable supplies the resolved
receiver type when the compatibility AST differs. A differential signature
and context regression verifies that an AST string receiver becomes the
semantic integer receiver in both the JVM descriptor and callable local.
Focused validation and the serialized offline workspace suite pass with 815
library tests, 19 CLI tests, 11 DOSBox conformance tests, 29 examples, 55 JVM
conformance tests, 1 language conformance test, 11 general record tests, 38
record method tests, and doc-tests. Graphify refreshed to 3626 nodes, 11353
edges, and 141 communities.

Iteration 33 makes JVM scalar method-call emission consume the typed member
expression, including its resolved receiver and callable signature. Semantic
IR now matches a member token against the declared callable after removing
the BASIC scalar suffix, so a method declared as `decorate$` is correctly
annotated when invoked as `decorate`. A differential JVM codegen regression
verifies that the typed receiver and method body replace their AST counterparts.
Focused validation and the serialized offline workspace suite pass with 816
library tests, 19 CLI tests, 11 DOSBox conformance tests, 29 examples, 55 JVM
conformance tests, 1 language conformance test, 11 general record tests, 38
record method tests, and doc-tests. Graphify refreshed to 3630 nodes, 11380
edges, and 142 communities.

Iteration 34 extends typed JVM scalar method-call coverage to numeric
receivers and results. The differential codegen regression verifies that the
typed receiver global and integer method descriptor replace the AST operand,
and that the callable body emits the typed numeric expression. Focused JVM
emission and semantic-IR annotation tests pass. The serialized offline
workspace suite passes with 817 library tests, 19 CLI tests, 11 DOSBox
conformance tests, 29 examples, 55 JVM conformance tests, 1 language
conformance test, 11 general record tests, 38 record method tests, and
doc-tests. Graphify topology is unchanged at 3630 nodes, 11380 edges, and 142
communities.

Iteration 35 makes semantic-IR scalar method result annotation select a
callable whose required and defaulted parameter counts accept the member-call
arity. This prevents an earlier same-name, same-receiver zero-argument method
from supplying the result type for a later overload that takes an argument.
A regression covers zero- and one-argument `choose` methods with different
result types. Focused validation and the serialized offline workspace suite
pass with 818 library tests, 19 CLI tests, 11 DOSBox conformance tests, 29
examples, 55 JVM conformance tests, 1 language conformance test, 11 general
record tests, 38 record method tests, and doc-tests. Graphify topology is
unchanged at 3630 nodes, 11380 edges, and 142 communities.

Iteration 36 extends typed JVM scalar method-call coverage to explicit scalar
arguments. A differential codegen regression verifies that typed receiver and
argument expressions are emitted in receiver-first descriptor order and that
the method body binds both the resolved `self` and parameter locals. Focused
validation and the serialized offline workspace suite pass with 819 library
tests, 19 CLI tests, 11 DOSBox conformance tests, 29 examples, 55 JVM
conformance tests, 1 language conformance test, 11 general record tests, 38
record method tests, and doc-tests. Graphify topology remains at 3630 nodes,
11380 edges, and 142 communities.

Iteration 37 extends typed JVM scalar method-call coverage to scalar `ByRef`
arguments. A differential codegen regression verifies that the typed actual is
loaded for the reference wrapper and writeback, that the JVM descriptor places
the receiver before the array-backed `ByRef` parameter, and that the method
body uses the typed implementation. Focused validation and the serialized
offline workspace suite pass with 820 library tests, 19 CLI tests, 11 DOSBox
conformance tests, 29 examples, 55 JVM conformance tests, 1 language
conformance test, 11 general record tests, 38 record method tests, and
doc-tests. Graphify topology remains at 3630 nodes, 11380 edges, and 142
communities.

Iteration 38 extends typed JVM scalar method-call coverage to by-value array
arguments. A differential codegen regression verifies that the typed array
actual is passed with the receiver-first descriptor and that the typed method
prologue copies the array into its parameter slot. Focused validation and the
serialized offline workspace suite pass with 821 library tests, 19 CLI tests,
11 DOSBox conformance tests, 29 examples, 55 JVM conformance tests, 1 language
conformance test, 11 general record tests, 38 record method tests, and
doc-tests. Graphify topology remains at 3630 nodes, 11380 edges, and 142
communities.

Iteration 39 adds an assembled JVM runtime regression for a scalar method with
a typed by-value array parameter. The method mutates its local array copy and
returns a value derived from that mutation; the caller observes the return
value while its original array element remains unchanged. The focused runtime
test and serialized offline workspace suite pass with 821 library tests, 19
CLI tests, 11 DOSBox conformance tests, 29 examples, 56 JVM conformance tests,
1 language conformance test, 11 general record tests, 38 record method tests,
and doc-tests. Graphify topology is unchanged at 3630 nodes, 11380 edges, and
142 communities.

Iteration 40 adds an assembled JVM runtime regression for a scalar method with
a typed `ByRef` array parameter. The method mutates the referenced array and
returns a value derived from it; runtime output confirms both the returned
value and the caller-visible array mutation. The focused runtime test and
serialized offline workspace suite pass with 821 library tests, 19 CLI tests,
11 DOSBox conformance tests, 29 examples, 57 JVM conformance tests, 1 language
conformance test, 11 general record tests, 38 record method tests, and
doc-tests. Graphify topology remains at 3630 nodes, 11380 edges, and 142
communities.

Iteration 41 extends typed JVM scalar method-call coverage to explicit string
arguments. A differential codegen regression verifies that the typed receiver
and string argument use the receiver-first JVM descriptor and that the method
body consumes the typed string implementation. Focused validation and the
serialized offline workspace suite pass with 822 library tests, 19 CLI tests,
11 DOSBox conformance tests, 29 examples, 57 JVM conformance tests, 1 language
conformance test, 11 general record tests, 38 record method tests, and
doc-tests. Graphify topology remains at 3630 nodes, 11380 edges, and 142
communities.

Iteration 42 verifies numeric widening and JVM wide-slot allocation in typed
scalar method calls. The differential regression passes an integer argument to
a double method, verifies `i2d` before invocation, and checks that the double
receiver and parameter occupy the correct local slots in the typed method
body. Focused validation and the serialized offline workspace suite pass with
823 library tests, 19 CLI tests, 11 DOSBox conformance tests, 29 examples, 57
JVM conformance tests, 1 language conformance test, 11 general record tests,
38 record method tests, and doc-tests. Graphify topology remains at 3630 nodes,
11380 edges, and 142 communities.

Iteration 43 adds an assembled JVM runtime regression for a scalar method with
a string `ByRef` parameter. The method appends to its string argument and
returns a value that consumes the updated parameter; runtime output confirms
the returned string and caller-visible writeback while an unrelated string
remains unchanged. Focused runtime validation and the serialized offline
workspace suite pass with 823 library tests, 19 CLI tests, 11 DOSBox
conformance tests, 29 examples, 58 JVM conformance tests, 1 language
conformance test, 11 general record tests, 38 record method tests, and
doc-tests. Graphify topology remains at 3630 nodes, 11380 edges, and 142
communities.

Iteration 44 adds an assembled JVM runtime regression for a double scalar
method receiver with an integer argument. The emitted typed call widens the
integer actual to double, preserves the two-slot receiver and parameter ABI,
and runs to the expected result. Focused runtime validation and the serialized
offline workspace suite pass with 823 library tests, 19 CLI tests, 11 DOSBox
conformance tests, 29 examples, 59 JVM conformance tests, 1 language
conformance test, 11 general record tests, 38 record method tests, and
doc-tests. Graphify topology remains at 3630 nodes, 11380 edges, and 142
communities.

Iteration 45 adds typed default argument metadata to JVM callable signatures
and emits omitted trailing arguments from semantic IR for both function and
scalar method calls. Differential codegen regressions give AST and semantic
IR different explicit arguments and defaults, verifying that both the call
operands and appended default come from the semantic callable. Focused
validation and the serialized offline workspace suite pass with 825 library
tests, 19 CLI tests, 11 DOSBox conformance tests, 29 examples, 59 JVM
conformance tests, 1 language conformance test, 11 general record tests, 38
record method tests, and doc-tests. Graphify refreshed to 3644 nodes, 11415
edges, and 136 communities.

Iteration 46 adds an assembled JVM runtime regression that calls both a
function and a scalar method with omitted trailing defaults. Runtime output
verifies that the JVM supplies both declared defaults correctly. Focused
runtime validation and the serialized offline workspace suite pass with 825
library tests, 19 CLI tests, 11 DOSBox conformance tests, 29 examples, 60 JVM
conformance tests, 1 language conformance test, 11 general record tests, 38
record method tests, and doc-tests. Graphify topology remains at 3644 nodes,
11415 edges, and 136 communities.

Iteration 47 adds JVM runtime coverage for a string literal method default and
a numeric function default supplied by a top-level `const`. The assembled
program verifies both the string concatenation result and constant-backed
numeric result. Focused runtime validation and the serialized offline
workspace suite pass with 825 library tests, 19 CLI tests, 11 DOSBox
conformance tests, 29 examples, 61 JVM conformance tests, 1 language
conformance test, 11 general record tests, 38 record method tests, and
doc-tests. Graphify topology remains at 3644 nodes, 11415 edges, and 136
communities.

Iteration 48 makes typed JVM `FIELD` layout collection consume semantic IR as
the authoritative source whenever it is available, retaining compatibility
AST scanning only for AST-only callers. Generated record/file buffer layouts
are reconstructed from `LoweredRecordFile` metadata at semantic file
declarations, so the backend no longer needs synthesized AST `FIELD` nodes.
A differential collector regression verifies typed-versus-AST layout
selection. Focused regressions and the serialized offline workspace suite
pass with 826 library tests, 19 CLI tests, 11 DOSBox conformance tests, 29
examples, 61 JVM conformance tests, 1 language conformance test, 11 general
record tests, 38 record method tests, and doc-tests. Graphify refreshed to
3648 nodes, 11432 edges, and 140 communities.

Iteration 49 makes JVM class identity consume the semantic module header when
typed IR is available, retaining AST `program_decl` lookup only for
compatibility callers. A differential codegen regression assigns different
program names to the AST and semantic module and verifies that the emitted
class uses the typed name. Focused validation and the serialized offline
workspace suite pass with 827 library tests, 19 CLI tests, 11 DOSBox
conformance tests, 29 examples, 61 JVM conformance tests, 1 language
conformance test, 11 general record tests, 38 record method tests, and
doc-tests. Graphify refreshed to 3649 nodes, 11436 edges, and 140 communities.

Iteration 50 adds a focused regression for JVM record-file buffer layouts
reconstructed from typed `LoweredRecordFile` metadata. It verifies channel,
field widths, cumulative offsets, and exclusion of an AST-only `FIELD`
binding. The focused regression and serialized offline workspace suite pass
with 828 library tests, 19 CLI tests, 11 DOSBox conformance tests, 29 examples,
61 JVM conformance tests, 1 language conformance test, 11 general record
tests, 38 record method tests, and doc-tests. Graphify topology remains at
3649 nodes, 11436 edges, and 140 communities.

Iteration 51 extends the JVM semantic scalar-declaration collector with
typed scalar slots for generated record-file buffers, deriving JVM storage
types from `LoweredRecordFieldKind`. A focused collector regression verifies
that an `Int32` buffer receives a JVM `long` slot, and the existing
try/catch-wrapped file transpilation regression confirms compatibility
record temporaries remain available while their identities are still carried
by the compatibility AST. The serialized offline workspace suite passes
with 829 library tests, 19 CLI tests, 11 DOSBox conformance tests, 29 examples,
61 JVM conformance tests, 1 language conformance test, 11 general record
tests, 38 record method tests, and doc-tests. Graphify topology remains at
3649 nodes, 11436 edges, and 140 communities.

Iteration 52 factors JVM storage classification for `LoweredRecordFieldKind`
into one typed conversion and extends the semantic declaration regression
across string, 16-bit integer, 32-bit integer, 32-bit float, and 64-bit float
fields. The test confirms storage classification follows typed record
metadata, including the JVM's widened `double` representation for both float
field kinds. Focused validation and the serialized offline workspace suite
pass with 829 library tests, 19 CLI tests, 11 DOSBox conformance tests, 29
examples, 61 JVM conformance tests, 1 language conformance test, 11 general
record tests, 38 record method tests, and doc-tests. Graphify topology remains
at 3649 nodes, 11436 edges, and 140 communities.

Iteration 53 centralizes C suffix selection for semantic record field types
and applies the typed mapping when expanding record fields into global scalar
declarations. Coverage verifies strings, 16-bit and 32-bit integers, both
floating-point widths, and that nested-record fields do not become scalar
declarations. Focused validation and the serialized offline workspace suite
pass with 830 library tests, 19 CLI tests, 11 DOSBox conformance tests, 29
examples, 61 JVM conformance tests, 1 language conformance test, 11 general
record tests, 38 record method tests, and doc-tests. Graphify topology remains
at 3649 nodes, 11436 edges, and 140 communities.

Iteration 54 teaches the C semantic declaration collector to classify typed
dotted record-field targets from their resolved `value_type`, flattening the
field path with the same identifier convention used by record lowering. The
semantic C lvalue renderer now accepts typed dotted names as well. A
differential regression confirms the semantic field type contributes its C
storage declaration independently of the compatibility AST type. Record
statement source alignment is still lost across record transpilation, so
that assignment remains on the compatibility emitter until the lowered
record operation retains source identity. Focused checks and the serialized
offline workspace suite pass with 831 library tests, 19 CLI tests, 11 DOSBox
conformance tests, 29 examples, 61 JVM conformance tests, 1 language
conformance test, 11 general record tests, 38 record method tests, and
doc-tests. Graphify topology remains at 3649 nodes, 11436 edges, and 140
communities.

Iteration 55 makes record-field type annotation consistent with resolver and
lowerer contracts: `int`/`int32` fields retain `Long`, while `int16` retains
`Integer`. C semantic name collection, lvalue emission, and numeric/string
expression rendering flatten typed dotted record names to the same generated
storage identity as record transpilation. The source mapper now assigns a
semantic node to the first compatible AST statement when one source statement
expanded into multiple lowered siblings, preserving semantic dispatch for
subsequent source statements. Differential tests verify typed member writes;
the methods tutorial and combined-record C/JVM runtime regressions pass. The
serialized offline workspace suite passes with 831 library tests, 19 CLI
tests, 11 DOSBox conformance tests, 29 examples, 61 JVM conformance tests, 1
language conformance test, 11 general record tests, 38 record method tests,
and doc-tests. Graphify refreshed to 3657 nodes, 11465 edges, and 140
communities.

Iteration 56 adds C runtime coverage for a typed record member passed as a
scalar `byref` argument. The regression verifies that the callee's writeback
updates the flattened record-field storage observed by the caller, exercising
the semantic argument identifier path introduced alongside record-field
expression transpilation. The record-method integration suite passes all 39
tests, and the serialized offline workspace suite passes with 831 library
tests, 19 CLI tests, 11 DOSBox conformance tests, 29 examples, 61 JVM
conformance tests, 1 language conformance test, 11 general record tests, 39
record method tests, and doc-tests. The test fixture explicitly declares the
parameter `byref`, so it validates caller-visible mutation rather than
by-value argument behavior.

Iteration 57 moves C top-level scalar declaration collection off the
compatibility AST whenever typed IR is present. Semantic global names,
semantic scalar statements, and typed record-instance layouts now provide the
storage facts; combined record fields are traversed recursively from their
typed declarations. This prevents AST-only names from creating unused or
stale C globals while retaining flattened storage required by record literal
transpilation. Differential coverage verifies semantic assignment names
replace AST-only names, and the typed record field test covers semantic field
types. C backend unit coverage passes (112 tests), the methods tutorial builds
and runs, and the serialized offline workspace suite passes with 832 library
tests, 19 CLI tests, 11 DOSBox conformance tests, 29 examples, 61 JVM
conformance tests, 1 language conformance test, 11 general record tests, 39
record method tests, and doc-tests. Graphify refreshed to 3660 nodes, 11475
edges, and 142 communities.

Iteration 58 adds C runtime coverage for a typed record string member passed
as a scalar `byref` argument. The test verifies that semantic string call
rendering passes the flattened member buffer through the C pointer ABI and
that callee copyback is visible in the caller's record field. The focused
case passes, and the full record-method integration suite passes all 40 tests
including the numeric and string record-member `byref` regressions. No
compiler change was required because the typed semantic string-call path
already preserves the buffer's field identity and capacity.

Iteration 59 adds per-node BASIC `DIM` emission from semantic declarations.
The emitter consumes the typed name, array rank, dimension expressions, and
`AS` annotation directly, captures nonliteral axis capacities for later
`SIZEOF`/`LBOUND`/`UBOUND` resolution, and declines unsupported inferred axes
before mutating output state. A differential regression gives the aligned
AST and semantic IR different array names and bounds, verifying that emitted
BASIC uses the typed declaration. The existing nested-DIM differential now
asserts declaration and capacity behavior directly because AST and semantic
control-flow paths allocate alpha-equivalent temporary names differently.
All 66 BASIC backend tests and the serialized offline workspace suite pass
(833 library tests, 19 CLI tests, 11 DOSBox tests, 29 examples, 61 JVM tests,
1 language test, 11 general record tests, 40 record-method tests, and
doc-tests). Graphify refreshed to 3663 nodes, 11490 edges, and 139
communities.

Iteration 60 extends semantic BASIC `DIM` dispatch into callable bodies. The
callable emitter now consumes its `FunctionInfo` typed dimension declarations,
uses the callable-local name allocator for the array identifier and bound
expressions, and stores captured capacities in the callable's local bound
state. A differential function fixture gives the AST and semantic IR distinct
local array names and bounds; nested module and callable DIM parity checks now
assert emitted declarations and captures rather than alpha-equivalence of
backend-generated temporary identifiers. All 67 BASIC backend tests pass,
and the serialized offline workspace suite passes (834 library tests, 19 CLI
tests, 11 DOSBox tests, 29 examples, 61 JVM tests, 1 language test, 11
general record tests, 40 record-method tests, and doc-tests). Graphify
refreshed to 3664 nodes, 11496 edges, and 139 communities.

Iteration 61 adds aligned per-node BASIC `TRY` emission from typed IR. The
emitter consumes semantic try/catch/finally bodies, catch bindings and error
filters, preserving the existing `ON ERROR GOTO`/`RESUME` unwind protocol and
nested handler stack. Structured `TRY` is enabled only after source alignment;
the unaligned whole-module fallback declines it so an unrelated semantic
source cannot replace compatibility AST output. Differential regressions
verify semantic throw/catch values, filter codes, and finally bodies take
precedence over AST values, while the existing alignment-failure tests retain
their AST fallback behavior. All 68 BASIC backend tests pass, and the
serialized offline workspace suite passes (835 library tests, 19 CLI tests,
11 DOSBox tests, 29 examples, 61 JVM tests, 1 language test, 11 general
record tests, 40 record-method tests, and doc-tests). Graphify refreshed to
3665 nodes, 11499 edges, and 142 communities.

Iteration 62 extends aligned BASIC semantic `TRY` emission to callable bodies.
Callable catch bindings, filter expressions, try/catch/finally statement trees,
and `source$` bindings now use the function's semantic expression renderer
and local name allocator. The existing error-handler stack is nested across
callable try regions, and a declined semantic subtree restores label, loop,
handler, local-name, and array-bound state before the AST fallback runs. A
differential procedure regression verifies semantic throw values, catch
filters, catch output, and finally output replace their AST counterparts. All
69 BASIC backend tests pass, and the serialized offline workspace suite
passes (836 library tests, 19 CLI tests, 11 DOSBox tests, 29 examples, 61 JVM
tests, 1 language test, 11 general record tests, 40 record-method tests, and
doc-tests). Graphify refreshed to 3667 nodes, 11510 edges, and 140
communities.

Iteration 63 completes aligned semantic BASIC `TRY` dispatch in callable
bodies, including optional `source$` resolution through the generated source
lookup subroutine. A differential function fixture checks semantic throw,
catch filter, catch/finally statements, and source binding against AST
counterparts; a second exact-output comparison verifies the typed emitter
matches compatibility output for identical source. That comparison caught
and corrected an indentation difference on the normal catch exit. Failed
semantic subtrees restore handler, label, loop, local-name, and array-bound
state before AST fallback. All 69 BASIC backend tests pass, and the serialized
offline workspace suite passes (836 library tests, 19 CLI tests, 11 DOSBox
tests, 29 examples, 61 JVM tests, 1 language test, 11 general record tests,
40 record-method tests, and doc-tests). Graphify refreshed; topology remains
3667 nodes, 11510 edges, and 140 communities.

Iteration 64 adds direct typed-IR dispatch for callable expression statements
in the BASIC codegen backend. The emitter now uses the semantic expression
renderer so function calls retain their GOSUB and argument-evaluation
preludes while discarding the expression result, matching statement context.
A differential regression verifies a semantic call replaces the AST statement
and preserves the call side effect. The focused regression passes; the
serialized offline workspace suite passes (837 library tests, 19 CLI tests,
11 DOSBox tests, 29 examples, 61 JVM tests, 1 language test, 11 general
record tests, 40 record-method tests, and doc-tests). `git diff --check`
passes. Workspace-wide `cargo fmt --check` remains blocked by formatting drift
in existing unrelated files.

Iteration 65 extends typed BASIC expression-statement dispatch to aligned
top-level statements. The semantic expression renderer emits function-call
preludes while discarding the call result; procedure-call expressions remain
on the compatibility path. A source-aligned regression verifies semantic
top-level call precedence. Declined semantic streams now restore generated
name allocation, top-level array-bound state, and diagnostics in addition to
labels and loop stacks. Focused regressions and the serialized offline
workspace suite pass (838 library tests, 19 CLI tests, 11 DOSBox tests, 29
examples, 61 JVM tests, 1 language test, 11 general record tests, 40
record-method tests, and doc-tests). `git diff --check` passes.

Iteration 66 refines BASIC semantic expression statements for function calls.
The shared statement-context renderer preserves argument and nested-call
preludes but removes the root result capture and returns that generated label
slot to the allocator, so discarded values do not produce unused BASIC
temporaries or shift subsequent labels. Procedure-call expressions continue
through the compatibility emitter. Callable and top-level regressions verify
semantic call dispatch without result capture; callable TRY/error-handler
regressions confirm fallback label and local behavior. The serialized offline
workspace suite passes (838 library tests, 19 CLI tests, 11 DOSBox tests, 29
examples, 61 JVM tests, 1 language test, 11 general record tests, 40
record-method tests, and doc-tests). `git diff --check` passes.

Iteration 67 enables semantic procedure-call statements through the shared
BASIC expression-statement renderer. Procedure calls retain typed argument
evaluation and ByRef copy-back preludes while discarding the synthetic result
capture. This exposed and fixed a typed-name collision: suffixed `err%` and
`erl%` locals now bypass suffixless `ERR`/`ERL` callable canonicalization in
semantic expression rendering, matching the AST renderer. The existing
catch-local argument regression, callable TRY differential, and serialized
offline workspace suite pass (838 library tests, 19 CLI tests, 11 DOSBox
tests, 29 examples, 61 JVM tests, 1 language test, 11 general record tests,
40 record-method tests, and doc-tests). `git diff --check` passes.

Iteration 68 adds direct typed-IR dispatch for zero-argument scalar method
expression statements in the BASIC backend. The receiver must be a typed
scalar name and its `SemanticValueType` selects the method signature in the
backend callable table; unsupported receiver or argument forms retain the
existing compatibility path. A differential regression verifies the typed
receiver assignment and scalar method GOSUB replace the AST print statement.
The focused regression and serialized offline workspace suite pass (839
library tests, 19 CLI tests, 11 DOSBox tests, 29 examples, 61 JVM tests, 1
language test, 11 general record tests, 40 record-method tests, and
doc-tests). `git diff --check` passes.

Iteration 69 extends direct BASIC typed-IR scalar method statement dispatch
to methods with scalar arguments. The backend resolves the method from the
receiver's semantic value type, evaluates each typed argument with its
prelude, assigns the callable's allocated parameter storage, and emits the
method GOSUB. Array parameters and unsupported receiver forms retain the
compatibility path. A differential regression verifies semantic receiver
and argument values replace the AST statement. Both focused method statement
tests and the serialized offline workspace suite pass (840 library tests,
19 CLI tests, 11 DOSBox tests, 29 examples, 61 JVM tests, 1 language test,
11 general record tests, 40 record-method tests, and doc-tests).
`git diff --check` passes. Clippy remains blocked by existing lint errors in
workspace dependency crates under `rdgen/`.

Iteration 70 extends scalar method expression-statement dispatch to typed
receiver expressions beyond bare names. The method signature is still selected
from the receiver's `SemanticValueType`; the receiver's own semantic expression
prelude now runs before typed method arguments and the GOSUB. A differential
regression uses a function-call receiver and confirms both the receiver
function and method execute while the AST statement is replaced. Focused
coverage and the serialized offline workspace suite pass (841 library tests,
19 CLI tests, 11 DOSBox tests, 29 examples, 61 JVM tests, 1 language test,
11 general record tests, 40 record-method tests, and doc-tests).
`git diff --check` passes.

Iteration 71 adds ByRef copy-back to direct typed BASIC scalar method
statement emission. ByRef arguments must be typed names; unsupported lvalues
decline semantic emission and retain the compatibility statement. A
differential method regression mutates a ByRef argument and verifies the
caller-side target is written back after the method GOSUB. The focused test
and serialized offline workspace suite pass (842 library tests, 19 CLI tests,
11 DOSBox tests, 29 examples, 61 JVM tests, 1 language test, 11 general
record tests, 40 record-method tests, and doc-tests). `git diff --check`
passes.

Iteration 72 adds an evaluation-order regression for typed BASIC scalar method
statement calls. It uses a function-call receiver and two function-call
arguments, then asserts the generated GOSUB sequence follows source order and
places the method invocation after all operand calls. All 76 BASIC backend
tests pass. This iteration changes test coverage only; the immediately
preceding Iteration 71 full workspace suite remains the applicable full-suite
result (842 library tests, 19 CLI tests, 11 DOSBox tests, 29 examples, 61 JVM
tests, 1 language test, 11 general record tests, 40 record-method tests, and
doc-tests). `git diff --check` passes.

Iteration 73 adds a callable-level regression for a typed function-call
statement with a ByRef argument. It verifies the call result is discarded and
the callee's mutated parameter is copied back to caller storage. All 77 BASIC
backend tests pass. This is a test-coverage iteration; the Iteration 71 full
workspace suite remains the latest full-suite result (842 library tests, 19
CLI tests, 11 DOSBox tests, 29 examples, 61 JVM tests, 1 language test, 11
general record tests, 40 record-method tests, and doc-tests).
`git diff --check` passes.

Iteration 74 adds direct typed-IR BASIC emission for top-level and callable
`CONST` statements. Initializers render through the semantic expression path,
and the existing allocated constant binding is used for the assignment
target. Differential regressions verify semantic initializer precedence in
both scopes. The serialized offline workspace suite passes (846 library
tests, 19 CLI tests, 11 DOSBox tests, 29 examples, 61 JVM tests, 1 language
test, 11 general record tests, 40 record-method tests, and doc-tests).
`git diff --check` passes.

Iteration 75 retains raw apostrophe and slash line-comment payloads in
`SemanticStatementKind` and directly transpiles typed BASIC module and
callable comments from semantic IR. The emitter normalizes leading whitespace
to preserve existing BASIC comment text. Regressions verify both payload
preservation and semantic comment precedence over AST comments. The serialized
offline workspace suite passes (848 library tests, 19 CLI tests, 11 DOSBox
tests, 29 examples, 61 JVM tests, 1 language test, 11 general record tests,
40 record-method tests, and doc-tests). `graphify update .` refreshed the
semantic-IR graph; `git diff --check` passes. The current rdgen frontend
rejects block-comment forms with `expected "*/"`, so block-comment semantic
dispatch remains pending a grammar fix.

Iteration 76 extends the direct BASIC comment regression to use `//` line
comments for both callable and module scope. The semantic payload is converted
to BASIC apostrophe comments while still replacing the aligned AST comment.
The focused regression passes, as does `git diff --check`.

Iteration 77 completes line and block comment payload adaptation by stopping
BASCAL's `any_char` scanner before the block-comment terminator. Enabling
block comments exposed two callable BASIC semantic-emission defects: WHILE
labels lacked their declaration colon, and unresolved calls with array
arguments incorrectly entered scalar expression emission. Callable WHILE now
emits a numeric-label-compatible declaration, and the call path declines
semantic emission when unresolved signature metadata cannot distinguish a
whole-array argument, preserving AST compatibility output. Regressions cover
line/block comment payloads, BASIC comment precedence, callable loop control,
and the sort-driver external array call. The serialized offline workspace
suite passes (848 library tests, 19 CLI tests, 11 DOSBox tests, 29 examples,
61 JVM tests, 1 language test, 11 general record tests, 40 record-method
tests, and doc-tests). `graphify update .` refreshed the graph, and
`git diff --check` passes.

Iteration 78 strengthens block-comment coverage with a multiline comment body.
The semantic IR regression verifies delimiters, indentation, and line breaks
survive adaptation; the BASIC backend regression verifies multiline callable
comments transpile to the same normalized apostrophe comment as the legacy
path. Both focused tests pass, as does `git diff --check`.

Iteration 79 extends the BASIC comment precedence regression to block comments
at both module and callable scope. The multiline block-comment payloads from the
semantic source replace the aligned AST comments in both scopes. The focused
BASIC backend test passes, as does `git diff --check`.

Iteration 80 gives block-comment scanning a dedicated
`any_char_except_block_comment_close` terminal, preventing `*/` handling from
changing the semantics of generic `any_char`. The rdgen grammar validator,
BASCAL grammar, generated-parser test scanner, build-time frontend scanner,
and probe scanner now use the scoped terminal. The generated BASCAL parser
fixture and rdgen grammar suite pass; `graphify update .` refreshed the graph,
and the serialized offline workspace suite passes (849 library tests, 19 CLI
tests, 11 DOSBox tests, 29 examples, 61 JVM tests, 1 language test, 11
general record tests, 40 record-method tests, and doc-tests).

Iteration 81 adds a scanner contract regression asserting `any_char` still
accepts `*/` while the block-comment-specific terminal stops before it. The
focused scanner test and the Iteration 80 full suite pass; `git diff --check`
passes.

Iteration 82 keeps slash-comment coverage in the BASIC semantic precedence
regression alongside module and callable multiline block comments. The focused
BASIC backend test passes, as does `git diff --check`.

Iteration 83 removes one callable codegen decision from the legacy AST when
semantic statement alignment succeeds: the implicit BASIC `RETURN` check now
uses the last aligned semantic statement. A differential regression places an
AST-only comment after its `RETURN` and verifies a semantic `RETURN` does not
receive a duplicate implicit return. The focused regression and serialized
offline workspace suite pass (850 library tests, 19 CLI tests, 11 DOSBox
tests, 29 examples, 61 JVM tests, 1 language test, 11 general record tests,
40 record-method tests, and doc-tests). `git diff --check` passes.

Iteration 84 adds a compatibility-boundary regression for callable return
handling. When semantic callable identity does not align, BASIC codegen still
uses the legacy AST return terminator and emits exactly one `RETURN`. The
focused fallback test passes, as does `git diff --check`.

Iteration 85 makes callable implicit-return detection follow the last effective
statement emitted from aligned semantic IR or per-node AST fallback. A
regression with a semantic assignment and unmatched AST `RETURN` verifies the
fallback return prevents duplicate implicit output. Focused callable return
coverage and the serialized offline workspace suite pass (852 library tests,
19 CLI tests, 11 DOSBox tests, 29 examples, 61 JVM tests, 1 language test,
11 general record tests, 40 record-method tests, and doc-tests). The full suite
passed on rerun after one transient generated-executable `PermissionDenied`
in the example harness; the isolated example and complete rerun both pass.
`git diff --check` passes.

Iteration 86 makes BASIC dependency annotations consume semantic `require`
and `import` declarations when typed IR is available, with a regression proving
semantic paths replace AST paths. The first full-suite run exposed linked
library dependencies being re-emitted as unresolved annotations. Iteration 87
adds an explicit resolved-state fact to semantic dependencies; the driver sets
it after recursively loading the dependency graph, and BASIC codegen omits
resolved annotations while retaining unresolved source declarations. Focused
driver and BASIC dependency tests pass. `graphify update .` refreshed the
cross-stage dependency graph, and the serialized offline workspace suite
passes (853 library tests, 19 CLI tests, 11 DOSBox tests, 29 examples, 61 JVM
tests, 1 language test, 11 general record tests, 40 record-method tests, and
doc-tests). `git diff --check` passes.

Iteration 88 strengthens the driver dependency-linking regression to assert
that every dependency retained in the merged semantic module is marked
resolved after recursive loading. The focused differential test passes, as
does `git diff --check`.

Iteration 89 verifies parsed `require` and `import` declarations start as
unresolved semantic dependencies; Iterations 87–88 cover the driver transition
to resolved state. The focused semantic IR test passes, as does
`git diff --check`.

Iteration 90 extends the driver dependency-state regression to a transitive
library graph: the root requires `math`, which itself requires `base`. The
focused differential test verifies all merged semantic dependencies are
marked resolved and linked callable output remains compatible. The test passes;
`git diff --check` passes.

Iteration 91 extends typed callable parameter facts to retain every declared
array-axis capacity (`?`, fixed literal, or typed expression) instead of only
rank. Callable semantic annotation now seeds parameter array declarations
from those retained dimensions. Regression coverage verifies inferred and
fixed parameter axes survive adaptation. `graphify update .` refreshed the
IR relationships, and the serialized offline workspace suite passes (853
library tests, 19 CLI tests, 11 DOSBox tests, 29 examples, 61 JVM tests, 1
language test, 11 general record tests, 40 record-method tests, and doc-tests).
`git diff --check` passes.

Iteration 92 migrates BASIC fixed callable-parameter capacities to typed IR.
The backend now uses retained semantic fixed-axis capacities when constructing
`FunctionInfo` parameter storage, so BASIC `DIM` output follows resolved
semantic capacity even when the legacy AST differs. Inferred and expression
axes retain the existing call-site analysis pending a typed-IR migration.
Regression coverage verifies the semantic capacity takes precedence. The
serialized offline workspace suite passes (854 library tests, 19 CLI tests,
11 DOSBox tests, 29 examples, 61 JVM tests, 1 language test, 11 general
record tests, 40 record-method tests, and doc-tests). `git diff --check`
passes.

Iteration 93 moves fixed typed parameter-capacity seeding into
`infer_array_param_capacities`, where semantic dimensions initialize the
same resolved capacity map used by inferred axes. The backend setup no longer
patches a second capacity map after inference. The semantic-capacity
regression and serialized offline workspace suite pass (854 library tests,
19 CLI tests, 11 DOSBox tests, 29 examples, 61 JVM tests, 1 language test,
11 general record tests, 40 record-method tests, and doc-tests).
`git diff --check` passes.

Iteration 94 makes typed callable parameter dimensions authoritative in BASIC
capacity inference for all axis variants: fixed literals parse from semantic
IR, constant expressions evaluate through `SemanticModule`, and inferred axes
remain open for call-site sizing. A differential regression proves a semantic
`CONST` expression capacity overrides the AST's fixed value. Both focused
capacity regressions pass. Parallel workspace runs encountered races in the
example harness while launching shared `remline` binaries; the serialized
offline workspace suite passes (855 library tests, 19 CLI tests, 11 DOSBox
tests, 29 examples, 61 JVM tests, 1 language test, 11 general record tests,
40 record-method tests, and doc-tests). `git diff --check` passes.

Iteration 95 adds a regression for semantic inferred parameter capacities.
When the semantic signature declares `?` and has no call sites, BASIC reports
that inference is impossible even if the compatibility AST contains a fixed
capacity. The focused regression passes and `git diff --check` passes.

Iteration 96 migrates top-level actual-array bound lookup for BASIC `?`
parameter inference to typed `DIM` declarations when semantic IR is available.
Semantic literal and compile-time expression capacities now replace the AST
size during call-site resolution; unknown typed axes remain unresolved instead
of falling back to stale AST bounds. A differential regression gives the AST
and semantic array different capacities and verifies the parameter storage
uses the semantic bound. `graphify update .` refreshed the data-flow graph.
The serialized offline workspace suite passes (857 library tests, 19 CLI
tests, 11 DOSBox tests, 29 examples, 61 JVM tests, 1 language test, 11
general record tests, 40 record-method tests, and doc-tests).
`git diff --check` passes.

Iteration 97 extends semantic array-bound lookup into ordinary callable
scopes. BASIC capacity inference now reads callable-local typed `DIM` axes and
evaluates their expressions with the callable's own `CONST` scope; unresolved
semantic axes remain unresolved. A differential regression verifies a
callable-local semantic bound overrides the AST bound. `graphify update .`
refreshed the data-flow graph. The serialized offline workspace suite passes
(858 library tests, 19 CLI tests, 11 DOSBox tests, 29 examples, 61 JVM tests,
1 language test, 11 general record tests, 40 record-method tests, and
doc-tests). `git diff --check` passes.

Iteration 98 adds differential coverage for callable-local semantic `CONST`
expressions used by array `DIM` bounds. The test verifies the scoped typed
constant evaluator resolves `2 + 3` from callable IR and sizes an inferred
parameter from that bound, replacing the AST's value of `2`. Focused and
serialized offline workspace suites pass (859 library tests, 19 CLI tests,
11 DOSBox tests, 29 examples, 61 JVM tests, 1 language test, 11 general
record tests, 40 record-method tests, and doc-tests). `git diff --check`
passes.

Iteration 99 adds differential coverage for module-scope semantic `CONST`
expressions used by top-level array `DIM` bounds. BASIC call-site inference
resolves the typed `2 + 3` capacity rather than the AST's fixed value of `2`.
The focused regression and serialized offline workspace suite pass (860
library tests, 19 CLI tests, 11 DOSBox tests, 29 examples, 61 JVM tests, 1
language test, 11 general record tests, 40 record-method tests, and
doc-tests). `git diff --check` passes.

Iteration 100 verifies typed actual-array bounds also drive BASIC's
compile-time parameter-capacity overflow diagnostic. The regression gives the
compatibility AST an actual capacity of 2 and semantic IR a capacity of 5;
with parameter storage fixed at 3, codegen reports that 5 elements exceed the
capacity. Focused and serialized offline workspace suites pass (861 library
tests, 19 CLI tests, 11 DOSBox tests, 29 examples, 61 JVM tests, 1 language
test, 11 general record tests, 40 record-method tests, and doc-tests).
`git diff --check` passes.

Iteration 101 verifies BASIC's inferred-capacity fixed point uses the typed
capacity of a forwarded array parameter. The differential case changes the
forwarding callable's semantic axis from the AST capacity 2 to capacity 6 and
confirms the downstream inferred parameter storage is sized to 6. Focused and
serialized offline workspace suites pass (862 library tests, 19 CLI tests,
11 DOSBox tests, 29 examples, 61 JVM tests, 1 language test, 11 general
record tests, 40 record-method tests, and doc-tests). `git diff --check`
passes.

Iteration 102 verifies an unresolved semantic actual-array dimension blocks
`?` inference even when the compatibility AST gives that array a fixed bound.
The regression expects the compile-time inference diagnostic instead of
accepting the stale AST capacity. Focused and serialized offline workspace
suites pass (863 library tests, 19 CLI tests, 11 DOSBox tests, 29 examples,
61 JVM tests, 1 language test, 11 general record tests, 40 record-method
tests, and doc-tests). `git diff --check` passes.

Iteration 103 verifies callable-local semantic array expressions with dynamic
bounds also block `?` inference rather than reusing a fixed AST size. The
focused regression and serialized offline workspace suite pass (864 library
tests, 19 CLI tests, 11 DOSBox tests, 29 examples, 61 JVM tests, 1 language
test, 11 general record tests, 40 record-method tests, and doc-tests).
`git diff --check` passes.

Iteration 104 adds differential coverage for multidimensional typed actual
array capacities. Both semantic axes (5 and 7) now determine inferred
parameter storage instead of the AST axes (2 and 3). The focused regression
and serialized offline workspace suite pass (865 library tests, 19 CLI tests,
11 DOSBox tests, 29 examples, 61 JVM tests, 1 language test, 11 general
record tests, 40 record-method tests, and doc-tests). `git diff --check`
passes.

Iteration 105 verifies maximum-capacity aggregation across multiple semantic
call sites. Actual arrays with typed capacities 5 and 8 produce inferred
parameter storage of 8 even when the AST capacities are 1 and 2. Focused and
serialized offline workspace suites pass (866 library tests, 19 CLI tests,
11 DOSBox tests, 29 examples, 61 JVM tests, 1 language test, 11 general
record tests, 40 record-method tests, and doc-tests). `git diff --check`
passes.

Iteration 106 verifies fixed-point inference rejects mixed semantic call-site
bounds when one actual array has a compile-time capacity and another has a
dynamic capacity. The dynamic typed axis prevents inferring storage from the
known call site alone. Focused and serialized offline workspace suites pass
(867 library tests, 19 CLI tests, 11 DOSBox tests, 29 examples, 61 JVM tests,
1 language test, 11 general record tests, 40 record-method tests, and
doc-tests). `git diff --check` passes.

Iteration 107 adds regression coverage for radix-form fixed capacities in
semantic callable parameters. The typed hexadecimal axis `&H5` is interpreted
as capacity 5 by BASIC inference and overrides the AST capacity 2. Focused and
serialized offline workspace suites pass (868 library tests, 19 CLI tests,
11 DOSBox tests, 29 examples, 61 JVM tests, 1 language test, 11 general
record tests, 40 record-method tests, and doc-tests). `git diff --check`
passes.

Iteration 108 verifies radix-form capacities in semantic actual-array
`DIM` declarations. A typed `&H5` array dimension yields inferred parameter
storage of 5 despite the AST's decimal capacity of 2. Focused and serialized
offline workspace suites pass (869 library tests, 19 CLI tests, 11 DOSBox
tests, 29 examples, 61 JVM tests, 1 language test, 11 general record tests,
40 record-method tests, and doc-tests). `git diff --check` passes.

Iteration 109 adds coverage for division and parenthesized arithmetic in typed
callable parameter capacities. The semantic dimension `(12 / 2) - 1` evaluates
to 5 in BASIC capacity inference and overrides the AST capacity 2. Focused and
serialized offline workspace suites pass (870 library tests, 19 CLI tests,
11 DOSBox tests, 29 examples, 61 JVM tests, 1 language test, 11 general
record tests, 40 record-method tests, and doc-tests). `git diff --check`
passes.

Iteration 110 verifies radix-form fixed bounds in callable-local semantic
`DIM` declarations. The local dimension map parses `&H5` into inferred
parameter capacity 5, replacing the AST capacity of 2. Focused and serialized
offline workspace suites pass (871 library tests, 19 CLI tests, 11 DOSBox
tests, 29 examples, 61 JVM tests, 1 language test, 11 general record tests,
40 record-method tests, and doc-tests). `git diff --check` passes.

Iteration 111 verifies per-axis resolution for multidimensional semantic
bounds. A dynamic second axis blocks inference only for axis 1; the diagnostic
identifies that axis despite a known first-axis capacity and a stale fixed
AST bound on the second. Focused and serialized offline workspace suites pass
(872 library tests, 19 CLI tests, 11 DOSBox tests, 29 examples, 61 JVM tests,
1 language test, 11 general record tests, 40 record-method tests, and
doc-tests). `git diff --check` passes.

Iteration 112 verifies typed parameter capacity expressions participate in the
forwarding fixed point. A caller parameter sized by semantic module `CONST`
expression `2 + 3` propagates capacity 5 to a downstream inferred parameter,
while its AST capacity of 2 does not leak into storage sizing. Focused and
serialized offline workspace suites pass (873 library tests, 19 CLI tests,
11 DOSBox tests, 29 examples, 61 JVM tests, 1 language test, 11 general
record tests, 40 record-method tests, and doc-tests). `git diff --check`
passes.

Iteration 113 makes semantic callable parameter rank authoritative throughout
BASIC array-capacity inference. The resolved per-parameter axis vectors now
come from typed callable dimensions when callable identity and arity align;
fixed-point inference, overflow checks, and forwarded-parameter bounds consume
those vectors instead of gating on AST `axes`. A differential regression
confirms a semantic inferred array parameter is sized even when the AST
parameter is scalar. `graphify update .` refreshed the data-flow graph. The
serialized offline workspace suite passes (874 library tests, 19 CLI tests,
11 DOSBox tests, 29 examples, 61 JVM tests, 1 language test, 11 general
record tests, 40 record-method tests, and doc-tests). `git diff --check`
passes.

Iteration 114 adds the inverse rank-authority regression: a semantic scalar
parameter drops array capacity declared only in the compatibility AST. BASIC
no longer emits array storage or dimension scratch for that stale AST axis.
The focused test and serialized offline workspace suite pass (875 library
tests, 19 CLI tests, 11 DOSBox tests, 29 examples, 61 JVM tests, 1 language
test, 11 general record tests, 40 record-method tests, and doc-tests).
`git diff --check` passes.

Iteration 115 adds differential coverage for semantic parameter rank adding an
axis absent from the AST. A typed two-axis inferred parameter now receives
both capacities from the actual array and emits two-axis storage even though
the compatibility AST signature is one-dimensional. Focused and serialized
offline workspace suites pass (876 library tests, 19 CLI tests, 11 DOSBox
tests, 29 examples, 61 JVM tests, 1 language test, 11 general record tests,
40 record-method tests, and doc-tests). `git diff --check` passes.

Iteration 116 completes the rank-driven storage declaration slice. BASIC's
one-time parameter `DIM` emitter now filters by resolved parameter rank rather
than AST `Param.axes`, so fixed semantic array parameters allocate storage
when the AST says scalar, while semantic scalar parameters suppress stale AST
array storage. The differential rank regressions pass. `graphify update .`
refreshed the typed-rank-to-BASIC-storage relationship; the serialized offline
workspace suite passes (877 library tests, 19 CLI tests, 11 DOSBox tests,
29 examples, 61 JVM tests, 1 language test, 11 general record tests,
40 record-method tests, and doc-tests). `git diff --check` passes.

Iteration 117 centralizes typed dimension-capacity evaluation for semantic
parameter axes, module `DIM` axes, and callable-local `DIM` axes. This keeps
fixed-literal parsing and compile-time expression evaluation consistent across
the BASIC capacity inference paths. Focused module/local constant cases and
the serialized offline workspace suite pass (877 library tests, 19 CLI tests,
11 DOSBox tests, 29 examples, 61 JVM tests, 1 language test, 11 general
record tests, 40 record-method tests, and doc-tests). `git diff --check`
passes.

Iteration 118 makes semantic DIM presence authoritative during capacity
resolution. When a semantic callable matches, its local array table and the
semantic module's top-level array table are consulted; absent semantic arrays
no longer reuse compatibility-AST size expressions. Callables without a
matching semantic signature retain their existing AST fallback. Regression
coverage verifies an AST-only top-level bound cannot size a semantic `?`
parameter. `graphify update .` refreshed the relationship graph, and the
serialized offline workspace suite passes (878 library tests, 19 CLI tests,
11 DOSBox tests, 29 examples, 61 JVM tests, 1 language test, 11 general
record tests, 40 record-method tests, and doc-tests). `git diff --check`
passes.

Iteration 119 verifies the compatibility boundary introduced by semantic DIM
authority: when the enclosing callable has no matching semantic signature,
BASIC still resolves local array bounds from its legacy AST body. The
differential regression preserves inferred capacity 5 for that unmatched
scope. Focused and serialized offline workspace suites pass (879 library
tests, 19 CLI tests, 11 DOSBox tests, 29 examples, 61 JVM tests, 1 language
test, 11 general record tests, 40 record-method tests, and doc-tests).
`git diff --check` passes.

Iteration 120 verifies semantic DIM authority in a matched callable scope. A
local array bound present only in the AST no longer sizes a semantic inferred
parameter when the semantic callable omits that declaration. The focused test
and serialized offline workspace suite pass (880 library tests, 19 CLI tests,
11 DOSBox tests, 29 examples, 61 JVM tests, 1 language test, 11 general
record tests, 40 record-method tests, and doc-tests). `git diff --check`
passes.

Iteration 121 verifies compile-time overflow checks use evaluated semantic
parameter expressions. A typed capacity expression `1 + 2` constrains storage
to 3 and produces a diagnostic when a semantic actual array has capacity 5,
regardless of the AST's capacity 10. Focused and serialized offline workspace
suites pass (881 library tests, 19 CLI tests, 11 DOSBox tests, 29 examples,
61 JVM tests, 1 language test, 11 general record tests, 40 record-method
tests, and doc-tests). `git diff --check` passes.

Iteration 122 verifies compile-time overflow diagnostics identify the semantic
axis that exceeds its fixed parameter capacity. With AST axis 1 at 3 and
typed semantic axis 1 at 6 against capacity 4, BASIC reports axis 1 and the
actual semantic count. Focused and serialized offline workspace suites pass
(882 library tests, 19 CLI tests, 11 DOSBox tests, 29 examples, 61 JVM tests,
1 language test, 11 general record tests, 40 record-method tests, and
doc-tests). `git diff --check` passes.

Iteration 123 adds a semantic procedure parameter regression. A typed fixed
array parameter allocates BASIC procedure storage with its resolved capacity
even when the AST procedure parameter is scalar. Focused and serialized
offline workspace suites pass (883 library tests, 19 CLI tests, 11 DOSBox
tests, 29 examples, 61 JVM tests, 1 language test, 11 general record tests,
40 record-method tests, and doc-tests). `git diff --check` passes.

Iteration 124 advances Stage 65 by making matched semantic callable bodies
authoritative for C local array declarations. The backend no longer unions
AST-local arrays into the semantic array table; unmatched callable scopes
retain the AST compatibility path. A differential regression confirms an
omitted semantic local `DIM` stays absent and a semantic bound of 5 replaces
an AST bound of 10. `graphify update .` refreshed the C declaration data flow.
The first full-suite run exhausted `/tmp` inodes; the serialized rerun with
`TMPDIR=target/tmp` passes (884 library tests, 19 CLI tests, 11 DOSBox tests,
29 examples, 61 JVM tests, 1 language test, 11 general record tests, 40
record-method tests, and doc-tests). `git diff --check` passes.

Iteration 125 adds end-to-end C output coverage for matched semantic callable
array bounds. A semantic local bound of 5 emits C storage with six elements
(BASIC inclusive bound) and replaces the AST bound of 10; the semantic indexed
write also appears in the generated function body. The focused regression and
serialized offline workspace suite with `TMPDIR=target/tmp` pass (885 library
tests, 19 CLI tests, 11 DOSBox tests, 29 examples, 61 JVM tests, 1 language
test, 11 general record tests, 40 record-method tests, and doc-tests).
`git diff --check` passes.

Iteration 126 verifies callable-local semantic array dimensions evaluate
callable-local `CONST` initializers in the C declaration path. A semantic
capacity of 5 allocates six C elements and replaces the AST's local constant
capacity of 10. The focused regression and serialized offline workspace suite
with `TMPDIR=target/tmp` pass (886 library tests, 19 CLI tests, 11 DOSBox
tests, 29 examples, 61 JVM tests, 1 language test, 11 general record tests,
40 record-method tests, and doc-tests). `git diff --check` passes.

Iteration 127 strengthens the Stage 65 compatibility-boundary regression. A
callable without a matching semantic signature still sources its local array
capacity from the AST, while a matching semantic callable continues to use its
typed DIM table. The focused declaration-table regression and
`git diff --check` pass.

Iteration 128 fixes a C callable-local scalar type leak. The semantic scalar
collector now registers unsuffixed writable names using the typed expression's
resolved value type, so a `LONG` target no longer reintroduces a default
single-precision AST candidate after semantic `DIM` has replaced it. A focused
regression verifies semantic `DIM AS LONG` removes a prior stale single local
and retains the resolved `LONG` local. Graphify was refreshed for the
semantic-to-C declaration flow; the serialized offline workspace suite passes
(887 library tests, 19 CLI tests, 11 DOSBox tests, 29 examples, 61 JVM tests,
1 language test, 11 general record tests, 40 record-method tests, and
doc-tests). `git diff --check` passes.

Iteration 129 verifies the C module-scope consumer of the corrected semantic
scalar target typing. A semantic global `DIM AS LONG` plus an unsuffixed
semantic assignment emits only the resolved integer C storage and does not
retain the legacy AST's single-precision suffix. The focused global-storage
regression and `git diff --check` pass.

Iteration 130 fixes the analogous JVM callable/module scalar declaration leak.
The semantic declaration collector now assigns a writable unsuffixed name from
its resolved expression type, preserving `LONG` instead of overwriting the
`DIM AS LONG` declaration with the backend's default numeric type. A focused
collector regression passes; Graphify was refreshed for the typed semantic to
JVM slot flow. The serialized offline workspace suite passes (889 library
tests, 19 CLI tests, 11 DOSBox tests, 29 examples, 61 JVM tests, 1 language
test, 11 general record tests, 40 record-method tests, and doc-tests).
`git diff --check` passes.

Iteration 131 adds JVM emitted-representation coverage for typed scalar
storage. With semantic `DIM AS LONG` and an unsuffixed write, generated JVM
assembly retains a long field descriptor and emits no double field for the
same name. The focused generation regression and `git diff --check` pass.

Iteration 132 preserves JVM semantic scalar `DIM` types while collecting
`LSET`/`RSET` write targets. Those write statements now add a fallback type
only when no stronger semantic declaration exists, preventing an unsuffixed
`AS STRING` binding from being overwritten with the default numeric type.
Focused `LSET`/`RSET` declaration coverage passes; Graphify was refreshed for
the semantic write-target-to-slot flow. The serialized offline workspace
suite passes (890 library tests, 19 CLI tests, 11 DOSBox tests, 29 examples,
61 JVM tests, 1 language test, 11 general record tests, 40 record-method
tests, and doc-tests). `git diff --check` passes.

Iteration 133 adds generated JVM descriptor coverage for a semantic string
`FIELD` binding consumed by `LSET`. A valid random-file fixture verifies the
string buffer is represented by the JVM string descriptor through generation.
The focused generation test and `git diff --check` pass.

Iteration 134 preserves resolved declaration types for JVM semantic `FOR`
variables. Loop registration now supplies an identifier-derived fallback only
when no typed declaration exists, retaining `DIM AS LONG` across loop slot
allocation. A focused declaration regression passes; Graphify was refreshed
for the semantic loop-variable-to-slot flow. The serialized offline workspace
suite passes (892 library tests, 19 CLI tests, 11 DOSBox tests, 29 examples,
61 JVM tests, 1 language test, 11 general record tests, 40 record-method
tests, and doc-tests). `git diff --check` passes.

Iteration 135 records the JVM backend's current integer-only `FOR` capability
boundary with a semantic `DIM AS LONG` regression. Generation returns the
specific capability diagnostic rather than emitting an invalid long-loop
sequence; the focused diagnostic regression and `git diff --check` pass.

Iteration 136 adds BASIC emitted-code coverage for semantic scalar type
authority. A semantic `DIM value AS LONG` replaces legacy AST `value%` and the
assignment emits the semantic value; the generated BASIC contains no stale
`DIM value%`. The focused generation regression and `git diff --check` pass.

Iteration 137 verifies the JVM callable-local slot consumer of semantic scalar
DIM types. A semantic `DIM value AS LONG` replaces legacy `%` in a procedure
body, and generated method assembly uses `lstore` without the double-store
path. The focused callable generation regression and `git diff --check` pass.

Iteration 138 corrects C scalar storage types for semantic string `FIELD`
bindings and `LSET`/`RSET` targets. The declaration collector now uses each
semantic construct's string-storage fact instead of inferring numeric storage
from an unsuffixed identifier. A focused regression verifies the emitted
storage table contains the string binding without a spurious numeric local;
Graphify was refreshed, the serialized offline workspace suite passes (897
library tests, 19 CLI tests, 11 DOSBox tests, 29 examples, 61 JVM tests,
1 language test, 11 general record tests, 40 record-method tests, and
doc-tests), and `git diff --check` passes.

Iteration 139 adds C backend diagnostic parity coverage for semantic `FIELD`
bindings whose identifiers omit the required string suffix. Although typed IR
marks the binding as string storage, C generation retains BASCAL's explicit
`$` requirement and returns the specific backend diagnostic. The focused
regression and `git diff --check` pass.

Iteration 140 preserves resolved C storage for semantic `FOR` variables. An
unsuffixed loop variable now reuses an existing typed declaration (such as
`DIM AS LONG`) instead of adding default single-precision storage. The focused
collector regression passes; Graphify was refreshed for the C loop-variable
storage path. The serialized offline workspace suite passes (899 library
tests, 19 CLI tests, 11 DOSBox tests, 29 examples, 61 JVM tests, 1 language
test, 11 general record tests, 40 record-method tests, and doc-tests).
`git diff --check` passes.

Iteration 141 adds direct typed C backend dispatch for semantic `LSET`/`RSET`
on supported string field expressions. The emitter uses the semantic field
width and preserves left/right alignment, while retaining compatibility
fallback for packed numeric conversion calls and generated record buffers. An
end-to-end differential regression confirms semantic `LSET` content replaces
the AST value. Graphify was refreshed; the serialized offline workspace suite
passes (900 library tests, 19 CLI tests, 11 DOSBox tests, 29 examples,
61 JVM tests, 1 language test, 11 general record tests, 40 record-method
tests, and doc-tests). `git diff --check` passes.

Iteration 142 verifies the right-aligned C semantic `RSET` path separately
from `LSET`. The end-to-end regression confirms semantic string content is
emitted with the field-width right-alignment format and replaces the AST value.
The focused generation test and `git diff --check` pass.

Iteration 143 extends typed C `LSET` coverage to semantic string callable
expressions. The generated `LSET` now consumes the semantic string-call result,
and the changed semantic callable return replaces the AST literal. The focused
end-to-end regression and `git diff --check` pass.

Iteration 144 extends typed C `LSET` dispatch to packed numeric string
conversions. Semantic `MKI$`, `MKL$`, `MKS$`, and `MKD$` calls now render their
resolved numeric argument directly into the corresponding C pack helper;
`RSET` packing and generated record buffers retain their compatibility path.
Focused conversion coverage passes. Graphify was refreshed, the serialized
offline workspace suite passes (904 library tests, 19 CLI tests, 11 DOSBox
tests, 29 examples, 61 JVM tests, 1 language test, 11 general record tests,
40 record-method tests, and doc-tests), and `git diff --check` passes.

Iteration 145 verifies the packed-conversion boundary for semantic C `RSET`.
Packed `MKI$` remains rejected with the existing specific diagnostic while
semantic `LSET` packing is enabled. The focused diagnostic regression and
`git diff --check` pass.

Iteration 146 verifies the callable C codegen dispatcher for semantic `LSET`.
A procedure-level differential fixture confirms the field write uses the
semantic string value rather than the compatibility AST value. The focused
callable generation regression and `git diff --check` pass.

Iteration 147 verifies callable-scope semantic `RSET` dispatch and right
alignment. The generated procedure uses the semantic value with the semantic
FIELD width, replacing the AST value. The focused callable regression and
`git diff --check` pass.

Iteration 148 aligns the semantic C pack-call recognizer with the legacy
`mk_pack_call` contract: only `$`-suffixed `MKI$`/`MKL$`/`MKS$`/`MKD$` calls
enter the numeric packing path. The complete supported conversion matrix,
focused `MKI$` dispatch regression, and `git diff --check` pass.

Iteration 149 validates the typed pack-call suffix guard across the serialized
offline workspace. The complete suite passes (907 library tests, 19 CLI tests,
11 DOSBox tests, 29 examples, 61 JVM tests, 1 language test, 11 general record
tests, 40 record-method tests, and doc-tests); Graphify was refreshed and
`git diff --check` passes.

Iteration 150 adds an end-to-end C runtime test for semantic packed `LSET`.
The generated program packs an integer with `MKI$`, writes and reads the
random-access record, and confirms `CVI` returns 1234. The focused compile/run
regression and `git diff --check` pass.

Iteration 151 restores semantic C dispatch for packed numeric `LSET` values.
The string-result type guard now follows the `MKI$`/`MKL$`/`MKS$`/`MKD$`
recognizer, since those semantic calls carry numeric arguments despite their
packed string representation. All 331 semantic library tests, the focused
`MKI$` dispatch regression, the packed `LSET` C runtime round trip, and
`git diff --check` pass.

Iteration 152 completes semantic result typing for `MKI$`, `MKL$`, `MKS$`,
and `MKD$`. The typed IR now records their string result type while preserving
their numeric argument types, and C semantic `LSET` still recognizes their
packing behavior. All 331 semantic library tests, the four-intrinsic C
dispatch matrix, Graphify refresh, and `git diff --check` pass. The serialized
offline workspace suite passes (907 library tests, 19 CLI tests, 11 DOSBox
tests, 30 examples, 61 JVM tests, 1 language test, 11 general record tests,
40 record-method tests, and doc-tests).

Iteration 153 includes synthesized top-level record DSL `OPEN` operations in
the semantic module's C runtime raise-site count. `LoweredRecordFile` already
retains the file owner, so module-scope files add one top-level raise site and
callable-owned files do not. The targeted semantic count regression and GCC
record DSL read/write runtime test pass; Graphify was refreshed, and the full
serialized offline workspace suite passes (908 library tests, 19 CLI tests,
11 DOSBox tests, 31 examples, 61 JVM tests, 1 language test, 11 general
record tests, 40 record-method tests, and doc-tests). `git diff --check`
passes.

Iteration 154 restricts semantic string-result typing for `MKI`/`MKL`/`MKS`/
`MKD` to the valid `$`-suffixed intrinsics. Invalid target suffixes such as
`MKI%` and `MKD#` remain `Unknown` instead of being misrepresented as
string-valued packing calls. The builtin result-type regression and C
four-intrinsic dispatch matrix pass; Graphify was refreshed and
`git diff --check` passes. The serialized offline workspace suite also
passes (908 library tests, 19 CLI tests, 11 DOSBox tests, 31 examples, 61 JVM
tests, 1 language test, 11 general record tests, 40 record-method tests, and
doc-tests).

Iteration 155 adds a GCC runtime regression for semantic packed `LSET` against
a record DSL backing field. The test writes `MKI$(1234)` through the pending
typed field value, executes the record DSL `PUT`/`GET`, and verifies `CVI`
returns 1234. The focused compile/run regression and `git diff --check` pass.

Iteration 156 extends the record DSL semantic packed-`LSET` runtime check to
all four target field kinds: `INT16`, `INT32`, `FLOAT32`, and `FLOAT64`. The
test packs with `MKI$`/`MKL$`/`MKS$`/`MKD$`, writes and reads the typed record,
then verifies `CVI`/`CVL`/`CVS`/`CVD` return the original values. The focused
GCC compile/run regression and `git diff --check` pass.

Iteration 157 adds explicit typed-IR facts for record DSL scalar locals
synthesized by record reads, including the `%` trim counter used for fixed
string fields. C codegen registers these facts at module and callable scope
instead of relying on compatibility-AST variable collection. A GCC runtime
regression verifies a callable can read a record after a partial update while
preserving an omitted string field. Graphify was refreshed; the serialized
offline workspace suite passes (908 library tests, 19 CLI tests, 11 DOSBox
tests, 33 examples, 61 JVM tests, 1 language test, 11 general record tests,
40 record-method tests, and doc-tests). `git diff --check` passes.

Iteration 158 wires the same typed record-read scalar facts into JVM semantic
declaration collection. The JVM collector now selects descriptors from the
retained semantic value types for synthesized record fields and trim counters;
the typed metadata regression passes. Graphify was refreshed, and the complete
serialized offline workspace suite passes (908 library tests, 19 CLI tests,
11 DOSBox tests, 33 examples, 61 JVM tests, 1 language test, 11 general
record tests, 40 record-method tests, and doc-tests). `git diff --check`
passes.

Iteration 159 broadens the callable record-read runtime regression to cover
generated scalar declarations for `INT16`, `INT32`, `FLOAT32`, `FLOAT64`, and
fixed-width string fields. GCC compiles and runs the partial-update/read path;
all five values retain the expected runtime representation. The focused
compile/run regression and `git diff --check` pass.

Iteration 160 fixes record-local metadata association when a callable reads a
module-scope record file. The lowered record file owns the buffer layout,
while each synthesized scalar local retains the reading callable's scope.
The lowerer metadata test and GCC runtime regression cover both module and
callable reads across all supported field types. Graphify was refreshed; the
serialized offline workspace suite passes (909 library tests, 19 CLI tests,
11 DOSBox tests, 33 examples, 61 JVM tests, 1 language test, 11 general
record tests, 40 record-method tests, and doc-tests). `git diff --check`
passes.

Iteration 161 dispatches C record/file DSL assignments from typed semantic
record literals. The aligned high-level assignment seeds typed native values
for the existing DSL `PUT` helper; the compatibility AST still supplies the
record I/O sequence and partial-update mask. A differential regression proves
semantic values replace AST values for both full and partial writes. Graphify
was refreshed, and the serialized offline workspace suite passes (910 library
tests, 19 CLI tests, 11 DOSBox tests, 33 examples, 61 JVM tests, 1 language
test, 11 general record tests, 40 record-method tests, and doc-tests).
`git diff --check` passes.

Iteration 162 extends typed C record-literal writes into callable bodies. The
callable source mapper now aligns a semantic statement to the first unused
compatibility-AST sibling when one source statement expanded into a `GET`/
`LSET`/`PUT` sequence, matching the top-level mapping contract. A callable
differential regression verifies semantic numeric and string record values
replace AST values. Graphify was refreshed; the serialized offline workspace
suite passes (911 library tests, 19 CLI tests, 11 DOSBox tests, 33 examples,
61 JVM tests, 1 language test, 11 general record tests, 40 record-method
tests, and doc-tests). `git diff --check` passes.

Iteration 163 extends callable semantic record-write differential coverage to
partial literals. The callable's `?{ ... }` update uses semantic field values
while the lowered PUT retains its omitted-field mask; the regression confirms
both semantic values override the AST and no AST partial value leaks through.
The focused callable generation test and `git diff --check` pass.

Iteration 164 validates semantic record-literal writes against a flattened
`combines` layout. C maps inherited and directly declared semantic fields to
the lowerer's record-file field order for both complete and partial writes.
The differential code-generation regression passes with distinct AST and
semantic values; `git diff --check` passes.

Iteration 165 covers typed user-function calls inside semantic record literal
fields. C renders the semantic function arguments for both complete and
partial record writes, replacing the AST call arguments while retaining the
flattened `combines` layout. The focused differential code-generation test
and `git diff --check` pass.


Iteration 166 covers a string-returning user function in a semantic record
literal. C record PUT emission now has regression coverage for the typed
caller-output-buffer ABI and semantic string arguments, alongside the numeric
function case. The differential code-generation test confirms the semantic
string replaces the AST value; the focused test and `git diff --check` pass.

Iteration 167 extends semantic C record/file assignment dispatch to a typed
record variable source. The codegen backend seeds each flattened record field
from its retained semantic type and dotted field expression; module-scope and
callable assignments have differential regressions proving typed values
replace the compatibility-AST values. Graphify was refreshed and the focused
tests pass.

Iteration 168 validates the record-variable assignment slice across the
serialized offline workspace suite: 913 library tests, 19 CLI tests, 11
DOSBox tests, 33 examples, 61 JVM tests, 1 language test, 11 general record
tests, 40 record-method tests, and doc-tests pass. `git diff --check` passes.

Iteration 169 extends typed C record-variable write dispatch to parenthesized
record-variable expressions. The semantic record identity and variable name
survive the parentheses, and the existing differential regression covers
both bare and parenthesized writes. The focused test and serialized offline
workspace suite pass (913 library tests, 19 CLI tests, 11 DOSBox tests, 33
examples, 61 JVM tests, 1 language test, 11 general record tests, 40
record-method tests, and doc-tests). `git diff --check` passes.

Iteration 170 adds callable-scope coverage for parenthesized typed record
variables assigned to a module-scope record file. The test exercises
semantic dispatch through the callable's `global` file binding and confirms
typed field values replace compatibility-AST values for both bare and
parenthesized sources. The focused callable regression and `git diff --check`
pass.

Iteration 171 broadens typed record-variable write coverage to `INT16`,
`INT32`, `FLOAT32`, `FLOAT64`, and fixed-width string fields. Differential C
assertions verify each flattened field uses the typed variable value and
excludes its compatibility-AST value. The focused regression and serialized
offline workspace suite pass (913 library tests, 19 CLI tests, 11 DOSBox
tests, 33 examples, 61 JVM tests, 1 language test, 11 general record tests,
40 record-method tests, and doc-tests). `git diff --check` passes.

Iteration 172 adds callable-scope differential coverage for typed record
variable writes across `INT16`, `INT32`, `FLOAT32`, `FLOAT64`, and fixed-width
string fields. Both bare and parenthesized assignments to a module-scope
record file preserve semantic field values inside the callable body. The
focused callable regression and `git diff --check` pass.

Iteration 173 validates typed record-variable write field ordering through a
flattened `combines` layout. Inherited `INT16` and `INT32` fields precede the
direct `FLOAT32`, `FLOAT64`, and string fields, and differential assertions
confirm C uses each semantic value at the matching packed-record position.
The focused regression and `git diff --check` pass.

Iteration 174 validates callable typed record-variable writes through the same
flattened `combines` layout. Inherited and directly declared fields retain
their semantic values in callable scope for both bare and parenthesized
record-variable assignments. The focused callable regression and
`git diff --check` pass.

Iteration 175 adds a GCC runtime regression for record-variable file writes.
Top-level and callable writes round-trip `INT16`, `INT32`, `FLOAT32`,
`FLOAT64`, and fixed-width string fields through record DSL GET/PUT and
validate the generated C runtime values. The focused integration test passes.

Iteration 176 validates the record-variable runtime slice across the
serialized offline workspace suite: 913 library tests, 19 CLI tests, 11
DOSBox tests, 34 examples, 61 JVM tests, 1 language test, 11 general record
tests, 40 record-method tests, and doc-tests pass. `git diff --check` passes.

Iteration 177 extends the GCC runtime regression to copy a record value
populated by the record DSL GET operation back into the file, at both module
and callable scope. The round-trip verifies synthesized scalar locals retain
all numeric and string fields through semantic record-variable writes. The
focused integration test passes.

Iteration 178 validates the GET-to-record-variable-to-PUT runtime path across
the serialized offline workspace suite: 913 library tests, 19 CLI tests, 11
DOSBox tests, 34 examples, 61 JVM tests, 1 language test, 11 general record
tests, 40 record-method tests, and doc-tests pass. `git diff --check` passes.

Iteration 179 extends the GET-to-record-variable-to-PUT GCC runtime check to
a record type with inherited fields. The packed layout preserves inherited
`INT16`/`INT32` fields before direct `FLOAT32`/`FLOAT64`/string fields across
module and callable writes. The focused integration test passes.

Iteration 180 validates the inherited-field runtime path across the
serialized offline workspace suite: 913 library tests, 19 CLI tests, 11
DOSBox tests, 34 examples, 61 JVM tests, 1 language test, 11 general record
tests, 40 record-method tests, and doc-tests pass. `git diff --check` passes.

Iteration 181 extends the GCC record-variable runtime regression to mutate a
field after GET and before PUT. Module scope updates the inherited `INT16`
field; callable scope updates the inherited `INT32` field. Subsequent GETs
verify the changed fields and every omitted field value. The focused
integration test passes.

Iteration 182 validates the post-GET field-mutation path across the serialized
offline workspace suite: 913 library tests, 19 CLI tests, 11 DOSBox tests,
34 examples, 61 JVM tests, 1 language test, 11 general record tests, 40
record-method tests, and doc-tests pass. `git diff --check` passes.

Iteration 183 extends callable record-variable mutation coverage to a
fixed-width string field after GET. The subsequent PUT preserves the other
numeric fields while storing the modified string; a second GET verifies the
runtime value. The focused GCC integration test passes.

Iteration 184 validates the post-GET string mutation path across the
serialized offline workspace suite: 913 library tests, 19 CLI tests, 11
DOSBox tests, 34 examples, 61 JVM tests, 1 language test, 11 general record
tests, 40 record-method tests, and doc-tests pass. `git diff --check` passes.

Iteration 185 retains the typed record index from semantic record-file
assignments in C file-I/O state and consumes it in the following generated
PUT. The codegen backend renders the index from typed IR, including numeric
name expressions, while retaining compatibility handling for unrelated raw
PUT statements.

Iteration 186 adds differential top-level and callable regressions proving
typed record-index variables replace distinct compatibility-AST indices for
record-variable writes. The focused regressions and `git diff --check` pass.

Iteration 187 validates typed record-index dispatch across the serialized
offline workspace suite: 913 library tests, 19 CLI tests, 11 DOSBox tests,
34 examples, 61 JVM tests, 1 language test, 11 general record tests, 40
record-method tests, and doc-tests pass. `git diff --check` passes.

Iteration 188 extends typed record-index differential coverage to a numeric
arithmetic expression. Top-level and callable C PUT emission use the typed
`4 * 2` index expression in place of the compatibility-AST variable index.
The focused regressions pass.

Iteration 189 validates typed record-index arithmetic across the serialized
offline workspace suite: 913 library tests, 19 CLI tests, 11 DOSBox tests,
34 examples, 61 JVM tests, 1 language test, 11 general record tests, 40
record-method tests, and doc-tests pass. `git diff --check` passes.

Iteration 190 extends typed record-index coverage to a numeric user-function
call. The top-level C record PUT uses the resolved function call and semantic
argument rather than the compatibility-AST index call. The focused
regression passes.

Iteration 191 validates numeric user-function record indices across the
serialized offline workspace suite: 913 library tests, 19 CLI tests, 11
DOSBox tests, 34 examples, 61 JVM tests, 1 language test, 11 general record
tests, 40 record-method tests, and doc-tests pass. `git diff --check` passes.

Iteration 192 adds callable-scope differential coverage for numeric
user-function record indices. The C PUT inside the procedure uses the typed
function call and semantic argument rather than the compatibility-AST call.
The focused callable regression passes.

Iteration 193 validates callable typed function indices across the serialized
offline workspace suite: 913 library tests, 19 CLI tests, 11 DOSBox tests,
34 examples, 61 JVM tests, 1 language test, 11 general record tests, 40
record-method tests, and doc-tests pass. `git diff --check` passes.

Iteration 194 extends the GCC record-variable runtime test to evaluate record
numbers through a user-defined numeric function for GET and PUT at module
and callable scope. All inherited and direct field values round-trip as
expected. The focused integration test passes.

Iteration 195 validates function-computed record numbers across the
serialized offline workspace suite: 913 library tests, 19 CLI tests, 11
DOSBox tests, 34 examples, 61 JVM tests, 1 language test, 11 general record
tests, 40 record-method tests, and doc-tests pass. `git diff --check` passes.

Iteration 196 extends top-level typed record-index coverage to the `LEN`
numeric intrinsic over a semantic string literal. C PUT emission consumes
the typed intrinsic expression in place of the compatibility-AST index
variable. The focused regression passes.

Iteration 197 validates intrinsic-computed record indices across the
serialized offline workspace suite: 913 library tests, 19 CLI tests, 11
DOSBox tests, 34 examples, 61 JVM tests, 1 language test, 11 general record
tests, 40 record-method tests, and doc-tests pass. `git diff --check` passes.

Iteration 198 verifies typed record indices survive the partial-write GET
guard. Top-level and callable full/partial record literal writes use the
semantic record numbers for their PUT helpers even when the generated
compatibility sequence includes a no-op GET guard. Focused regressions pass.

Iteration 199 validates semantic record-index retention through partial
write guards across the serialized offline workspace suite: 913 library
tests, 19 CLI tests, 11 DOSBox tests, 34 examples, 61 JVM tests, 1 language
test, 11 general record tests, 40 record-method tests, and doc-tests pass.
`git diff --check` passes.

Iteration 200 extends typed record-index dispatch to an array element
expression. A differential top-level regression verifies C PUT emission uses
the semantic array reference in place of the compatibility-AST scalar index.
The focused regression passes.

Iteration 201 validates array-backed record indices across the serialized
offline workspace suite: 913 library tests, 19 CLI tests, 11 DOSBox tests,
34 examples, 61 JVM tests, 1 language test, 11 general record tests, 40
record-method tests, and doc-tests pass. `git diff --check` passes.

Iteration 202 adds callable-scope differential coverage for an array element
used as a typed record index. C PUT emission resolves the callable's semantic
array reference rather than the compatibility-AST scalar index. The focused
callable regression passes.

Iteration 203 validates callable array-backed record indices across the
serialized offline workspace suite: 913 library tests, 19 CLI tests, 11
DOSBox tests, 34 examples, 61 JVM tests, 1 language test, 11 general record
tests, 40 record-method tests, and doc-tests pass. `git diff --check` passes.

Iteration 204 adds parenthesized numeric function-call indices to the
top-level and callable differential record-write regressions. Typed index
rendering preserves the call expression through parentheses in both scopes.
The focused regressions pass.

Iteration 205 validates parenthesized semantic record-index calls across the
serialized offline workspace suite: 913 library tests, 19 CLI tests, 11
DOSBox tests, 34 examples, 61 JVM tests, 1 language test, 11 general record
tests, 40 record-method tests, and doc-tests pass. `git diff --check` passes.

Iteration 206 extends typed record-index dispatch to a two-dimensional array
element. The differential top-level test verifies the semantic multi-index
expression reaches the C PUT helper rather than the compatibility-AST scalar
index. The focused regression passes.

Iteration 207 validates multi-index record-number expressions across the
serialized offline workspace suite: 913 library tests, 19 CLI tests, 11
DOSBox tests, 34 examples, 61 JVM tests, 1 language test, 11 general record
tests, 40 record-method tests, and doc-tests pass. `git diff --check` passes.

Iteration 208 adds callable-scope differential coverage for a two-dimensional
array element as the typed record number. C PUT emission retains the semantic
multi-index inside the callable and excludes the compatibility-AST scalar
index. The focused callable regression passes.

Iteration 209 validates callable multi-index record numbers across the
serialized offline workspace suite: 913 library tests, 19 CLI tests, 11
DOSBox tests, 34 examples, 61 JVM tests, 1 language test, 11 general record
tests, 40 record-method tests, and doc-tests pass. `git diff --check` passes.

Iteration 210 preserves the established C error-result ABI for record-index
calls. When a semantic index expression is outside the side-effect-free
numeric renderer (such as a TRY-propagating function), generated PUT
emission uses the compatibility index path, where the existing TRY result
hoister handles it. A regression confirms the backend still generates the
TRY-aware function call and record PUT.

Iteration 211 validates the TRY-aware record-index fallback across the
serialized offline workspace suite: 914 library tests, 19 CLI tests, 11
DOSBox tests, 34 examples, 61 JVM tests, 1 language test, 11 general record
tests, 40 record-method tests, and doc-tests pass. `git diff --check` passes.

Iteration 212 extends the GCC end-to-end record-variable write test to use an
array element as a record number at module scope. The array-selected record
round-trips inherited and direct fields while callable writes continue to
exercise a numeric function index. The focused integration test passes.

Iteration 213 validates array-selected record writes across the serialized
offline workspace suite: 914 library tests, 19 CLI tests, 11 DOSBox tests,
34 examples, 61 JVM tests, 1 language test, 11 general record tests, 40
record-method tests, and doc-tests pass. `git diff --check` passes.

Iteration 214 adds typed default-argument coverage for a numeric user
function used as a record number. Semantic IR's default argument value is
rendered into the top-level C PUT call, replacing an explicit compatibility
index call. The focused regression passes.

Iteration 215 validates typed default arguments in record-number calls across
the serialized offline workspace suite: 914 library tests, 19 CLI tests, 11
DOSBox tests, 34 examples, 61 JVM tests, 1 language test, 11 general record
tests, 40 record-method tests, and doc-tests pass. `git diff --check` passes.

Iteration 216 adds callable-scope coverage for a semantic default argument in
a numeric user function used as the record number. The callable C PUT uses
the typed default value rather than the explicit compatibility-AST argument.
The focused callable regression passes.

Iteration 217 validates callable default-argument indices across the
serialized offline workspace suite: 914 library tests, 19 CLI tests, 11
DOSBox tests, 34 examples, 61 JVM tests, 1 language test, 11 general record
tests, 40 record-method tests, and doc-tests pass. `git diff --check` passes.

Iteration 218 adds top-level typed ByRef argument coverage for a numeric
record-index function call. The C PUT index uses the resolved variable
address from semantic IR rather than the compatibility-AST argument. The
focused regression passes.

Iteration 219 validates top-level ByRef index calls across the serialized
offline workspace suite: 914 library tests, 19 CLI tests, 11 DOSBox tests,
34 examples, 61 JVM tests, 1 language test, 11 general record tests, 40
record-method tests, and doc-tests pass. `git diff --check` passes.

Iteration 220 adds callable-scope typed ByRef argument coverage for numeric
record-index calls. The C PUT index receives the address of the semantic
variable from the callable body rather than the AST argument. The focused
callable regression passes.

Iteration 221 validates callable ByRef index calls across the serialized
offline workspace suite: 914 library tests, 19 CLI tests, 11 DOSBox tests,
34 examples, 61 JVM tests, 1 language test, 11 general record tests, 40
record-method tests, and doc-tests pass. `git diff --check` passes.

Iteration 222 adds top-level floating-point user-function coverage for typed
record indices. The C backend renders the semantic `DOUBLE` call and applies
the record-number integer conversion at the PUT call site. The focused
regression passes.

Iteration 223 validates floating-point function indices across the serialized
offline workspace suite: 914 library tests, 19 CLI tests, 11 DOSBox tests,
34 examples, 61 JVM tests, 1 language test, 11 general record tests, 40
record-method tests, and doc-tests pass. `git diff --check` passes.

Iteration 224 adds callable-scope floating-point user-function coverage for
typed record indices. The procedure body passes the semantic `DOUBLE` call
result through the C record-number integer conversion. The focused callable
regression passes.

Iteration 225 validates callable floating-point indices across the serialized
offline workspace suite: 914 library tests, 19 CLI tests, 11 DOSBox tests,
34 examples, 61 JVM tests, 1 language test, 11 general record tests, 40
record-method tests, and doc-tests pass. `git diff --check` passes.

Iteration 226 extends floating-point function index coverage to a `SINGLE`
result alongside the existing `DOUBLE` result. C PUT emission preserves the
typed function call and applies numeric record-index conversion. The focused
top-level regression passes.

Iteration 227 validates `SINGLE` and `DOUBLE` function record indices across
the serialized offline workspace suite: 914 library tests, 19 CLI tests, 11
DOSBox tests, 34 examples, 61 JVM tests, 1 language test, 11 general record
tests, 40 record-method tests, and doc-tests pass. `git diff --check` passes.

Iteration 228 adds a two-channel differential regression for typed record
indices. Interleaved writes to two record files retain independent semantic
record numbers in their C PUT helper calls. The focused regression passes.

Iteration 229 validates channel-scoped semantic record indices across the
serialized offline workspace suite: 915 library tests, 19 CLI tests, 11
DOSBox tests, 34 examples, 61 JVM tests, 1 language test, 11 general record
tests, 40 record-method tests, and doc-tests pass. `git diff --check` passes.

Iteration 230 asserts the numeric conversion emitted for typed `SINGLE` and
`DOUBLE` record-index function results. C codegen rounds each floating-point
result to the integer record number, matching the backend's established
numeric conversion contract. The focused regression passes.

Iteration 231 validates floating-point record-index rounding across the
serialized offline workspace suite: 915 library tests, 19 CLI tests, 11
DOSBox tests, 34 examples, 61 JVM tests, 1 language test, 11 general record
tests, 40 record-method tests, and doc-tests pass. `git diff --check` passes.

Iteration 232 extends the GCC runtime check to a callable returning a
floating-point record number. The C backend rounds the `DOUBLE` result to the
correct record position; the resulting GET/PUT round-trip preserves all
record fields. The focused integration test passes.

Iteration 233 validates floating-point record-number runtime behavior across
the serialized offline workspace suite: 915 library tests, 19 CLI tests, 11
DOSBox tests, 34 examples, 61 JVM tests, 1 language test, 11 general record
tests, 40 record-method tests, and doc-tests pass. `git diff --check` passes.

Iteration 234 adds callable-scope coverage for TRY-propagating index
functions. The compatibility index hoister preserves the C error-result ABI
inside the generated record PUT procedure body. The focused callable
regression passes.

Iteration 235 validates TRY-aware record-index calls at top-level and
callable scope across the serialized offline workspace suite: 916 library
tests, 19 CLI tests, 11 DOSBox tests, 34 examples, 61 JVM tests, 1 language
test, 11 general record tests, 40 record-method tests, and doc-tests pass.
`git diff --check` passes.

Iteration 236 adds callable-scope coverage for the `LEN` intrinsic as a
semantic record number. The C PUT helper consumes the typed string argument
and `strlen` expression instead of the compatibility-AST constant index. The
focused regression passes.

Iteration 237 validates callable `LEN` indices across the serialized offline
workspace suite: 917 library tests, 19 CLI tests, 11 DOSBox tests, 34
examples, 61 JVM tests, 1 language test, 11 general record tests, 40
record-method tests, and doc-tests pass. `git diff --check` passes.

Iteration 238 extends callable intrinsic-index coverage with `VAL` over a
semantic string literal, alongside `LEN`. The C PUT helper consumes the
numeric `atof` expression rather than the compatibility-AST literal index.
The focused regression passes.

Iteration 239 validates callable `LEN`/`VAL` record indices across the
serialized offline workspace suite: 917 library tests, 19 CLI tests, 11
DOSBox tests, 34 examples, 61 JVM tests, 1 language test, 11 general record
tests, 40 record-method tests, and doc-tests pass. `git diff --check` passes.

Iteration 240 asserts `VAL` record-index conversion rounds its floating-point
result to an integer before passing the record number to the C PUT helper.
The focused callable regression passes.

Iteration 241 validates intrinsic record-index rounding across the serialized
offline workspace suite: 917 library tests, 19 CLI tests, 11 DOSBox tests,
34 examples, 61 JVM tests, 1 language test, 11 general record tests, 40
record-method tests, and doc-tests pass. `git diff --check` passes.

Iteration 242 extends two-channel record-index coverage to differently cased
semantic file identifiers. Case-insensitive semantic file lookup retains the
correct per-channel record indices for both C PUT helpers. The focused
regression passes.

Iteration 243 validates case-insensitive typed record-file indices across
the serialized offline workspace suite: 917 library tests, 19 CLI tests, 11
DOSBox tests, 34 examples, 61 JVM tests, 1 language test, 11 general record
tests, 40 record-method tests, and doc-tests pass. `git diff --check` passes.

Iteration 244 extends case-insensitive file lookup coverage to callable record
assignments with intrinsic indices. Uppercase semantic references resolve to
the module-scope file declared with lowercase spelling, and retain typed
`LEN`/`VAL` indices. The focused regression passes.

Iteration 245 validates case-insensitive callable record indices across the
serialized offline workspace suite: 917 library tests, 19 CLI tests, 11
DOSBox tests, 34 examples, 61 JVM tests, 1 language test, 11 general record
tests, 40 record-method tests, and doc-tests pass. `git diff --check` passes.

Iteration 246 rechecks the C record-index handoff boundary: typed indices are
consumed by the matching channel's PUT, while TRY-result expressions retain
the compatibility-AST path required for C error-result hoisting. No source
change was needed.

Iteration 247 reruns top-level and callable TRY-index fallback regressions and
the two-channel typed-index regression. All three focused tests pass.

Iteration 248 runs the complete serialized library test suite after the C
record-index path audit. All 917 library tests pass.

Iteration 249 reviews the staged migration worktree and plan chronology.
`git diff --check` passes, and all pre-existing edits and untracked fixtures
remain preserved.

Iteration 250 runs the complete serialized offline workspace suite: 917
library tests, 19 CLI tests, 11 DOSBox tests, 34 examples, 61 JVM tests, 1
language test, 11 general record tests, 40 record-method tests, and
doc-tests pass. The generated tutorial fixture is restored from `HEAD`, and
`git diff --check` passes.

Iteration 251 makes the FBC compatibility gate use `SemanticModule::contains_try`
as the authoritative unsupported-feature check whenever generated parsing
succeeds. The legacy AST walker remains only for the generated-parser
compatibility fallback, removing a redundant backend-driver AST rescan.

Iteration 252 makes C64's BYVAL-array/VLA capability check consume resolved
callable array rank and passing-mode facts when typed callable metadata is
available. The AST parameter walk remains only for compatibility callables
without a matching semantic signature; the existing callable source position
continues to anchor the diagnostic. A differential regression with an AST
scalar parameter and typed array parameter verifies the C64 rejection follows
the resolved signature.

Iteration 253 validates the FBC driver and C64 typed-capability slices across
the serialized offline workspace suite: 918 library tests, 19 CLI tests, 11
DOSBox tests, 34 examples, 61 JVM tests, 1 language test, 11 general record
tests, 40 record-method tests, and doc-tests pass. The generated tutorial
fixture is restored from `HEAD`, and `git diff --check` passes.

Iteration 254 extends Stage 64 callable-body dispatch to typed computed
`ON ... GOTO`/`ON ... GOSUB`. The selector and ordered label targets now
transpile from semantic IR using the callable's name and label context. The
existing differential control-transfer regression now checks semantic
computed-branch output against distinct AST print statements.

Iteration 255 validates the callable computed-branch slice across the
serialized offline workspace suite: 918 library tests, 19 CLI tests, 11
DOSBox tests, 34 examples, 61 JVM tests, 1 language test, 11 general record
tests, 40 record-method tests, and doc-tests pass. The generated tutorial
fixture is restored from `HEAD`, and `git diff --check` passes.

Iteration 256 adds direct typed JVM emission for top-level and callable
`CONST` initializers. The emitter stores the semantic initializer through
the slot type established by `JvmContext`, preserving numeric coercion and
string storage. The top-level differential regression now distinguishes the
typed initializer from its AST counterpart.

Iteration 257 validates typed JVM `CONST` emission across the serialized
offline workspace suite: 918 library tests, 19 CLI tests, 11 DOSBox tests,
34 examples, 61 JVM tests, 1 language test, 11 general record tests, 40
record-method tests, and doc-tests pass. The generated tutorial fixture is
restored from `HEAD`, and `git diff --check` passes.

Iteration 258 extends BASIC callable semantic dispatch to `GLOBAL` as a
typed no-op. Resolver-owned semantic name scopes already carry its binding;
the backend now consumes the aligned node without falling back to the
compatibility AST statement. The callable machine-statement regression
includes a placeholder AST print at that position.

Iteration 259 validates callable `GLOBAL` dispatch across the serialized
offline workspace suite: 918 library tests, 19 CLI tests, 11 DOSBox tests,
34 examples, 61 JVM tests, 1 language test, 11 general record tests, 40
record-method tests, and doc-tests pass. The generated tutorial fixture is
restored from `HEAD`, and `git diff --check` passes.

Iteration 260 validates the expanded numeric/string top-level and callable
JVM `CONST` differential coverage across the serialized offline workspace
suite: 918 library tests, 19 CLI tests, 11 DOSBox tests, 34 examples, 61 JVM
tests, 1 language test, 11 general record tests, 40 record-method tests, and
doc-tests pass. The generated tutorial fixture is restored from `HEAD`, and
`git diff --check` passes.

Iteration 261 extends the JVM differential `CONST` regression to an inferred
string constant alongside numeric module and callable constants. The focused
test confirms the JVM emitter stores the typed string literal and excludes
the compatibility-AST literal.

Iteration 262 extends the BASIC callable computed-branch regression to both
`ON ... GOTO` and `ON ... GOSUB`, checking that branch kind and target order
survive typed dispatch and that neither aligned AST placeholder is emitted.

Iteration 263 adds typed callable `DATA` emission to Stage 64. Literal items
are rendered from semantic expressions in source order; the callable machine
statement differential now verifies typed numeric/string items replace an
aligned AST placeholder.

Iteration 264 validates callable `DATA`, computed branches, JVM `CONST`, and
the driver/C64 slices across the serialized offline workspace suite: 918
library tests, 19 CLI tests, 11 DOSBox tests, 34 examples, 61 JVM tests, 1
language test, 11 general record tests, 40 record-method tests, and doc-tests
pass. The generated tutorial fixture is restored from `HEAD`, and
`git diff --check` passes.

Iteration 265 makes C module and callable dispatch consume typed `GLOBAL`
declarations as no-op statements; resolved global-name scopes already own
their storage effect. The C differential regression uses a semantic global
declaration over an AST assignment and verifies the AST write is not emitted.

Iteration 266 validates C global dispatch, BASIC callable `DATA`/`GLOBAL` and
computed branches, JVM typed `CONST`, and the driver/C64 slices across the
serialized offline workspace suite: 918 library tests, 19 CLI tests, 11
DOSBox tests, 34 examples, 61 JVM tests, 1 language test, 11 general record
tests, 40 record-method tests, and doc-tests pass. The generated tutorial
fixture is restored from `HEAD`, and `git diff --check` passes.

Iteration 267 makes JVM main and callable dispatch consume typed `GLOBAL`
declarations as no-op nodes; `JvmContext` already allocates their resolved
slots. A differential regression places AST assignments at both positions
and confirms neither AST initializer is emitted.

Iteration 268 validates JVM and C semantic `GLOBAL` dispatch together with
the recent BASIC, C64, and driver slices across the serialized offline
workspace suite: 919 library tests, 19 CLI tests, 11 DOSBox tests, 34
examples, 61 JVM tests, 1 language test, 11 general record tests, 40
record-method tests, and doc-tests pass. The generated tutorial fixture is
restored from `HEAD`, and `git diff --check` passes.

Iteration 269 makes JVM main and callable dispatch consume typed `DIM`
declarations as no-op nodes after `JvmContext` has allocated scalar slots and
arrays from resolved declaration facts. Existing module and callable type
descriptor regressions cover semantic declarations that differ from AST
annotations.

Iteration 270 validates direct JVM `DIM` dispatch and the BASIC/C/driver
migration slices across the serialized offline workspace suite: 919 library
tests, 19 CLI tests, 11 DOSBox tests, 34 examples, 61 JVM tests, 1 language
test, 11 general record tests, 40 record-method tests, and doc-tests pass.
The generated tutorial fixture is restored from `HEAD`, and
`git diff --check` passes.

Iteration 271 tightens C64's semantic capability callable match to include
the resolved method receiver type as well as callable kind, name, and arity.
Overloaded methods on different receiver types can no longer supply the
wrong BYVAL-array rank or passing-mode facts.

Iteration 272 validates the C64 receiver-sensitive callable match plus JVM
`DIM`/`GLOBAL`, C `GLOBAL`, BASIC callable declaration/branch dispatch, and
driver slices across the serialized offline workspace suite: 919 library
tests, 19 CLI tests, 11 DOSBox tests, 34 examples, 61 JVM tests, 1 language
test, 11 general record tests, 40 record-method tests, and doc-tests pass.
The generated tutorial fixture is restored from `HEAD`, and
`git diff --check` passes.

Iteration 273 adds a C64 overload regression: an AST LONG method has a BYVAL
array parameter, while typed IR includes separate INTEGER BYVAL and LONG
BYREF overloads. Receiver-sensitive matching selects the LONG BYREF facts and
permits C64 transpilation. The serialized offline workspace suite passes:
920 library tests, 19 CLI tests, 11 DOSBox tests, 34 examples, 61 JVM tests,
1 language test, 11 general record tests, 40 record-method tests, and
doc-tests. The tutorial fixture is restored from `HEAD`; `git diff --check`
passes.

Iteration 274 extends the C `GLOBAL` differential regression into callable
scope, verifying that semantic global collection and no-op dispatch suppress
an aligned AST assignment there too. The focused regression passes. The
legacy callable declaration collector still reserves an unused AST local
slot for the placeholder name; migrating that storage collector remains
separate work.

Iteration 275 removes that stale callable slot: C declaration collection now
discards AST storage candidates from statements aligned with typed `GLOBAL`
nodes, then filters the resolved callable-global names through its existing
global scope. The callable differential asserts the AST placeholder name is
absent from both emitted assignments and local declarations.

Iteration 276 validates the C callable `GLOBAL` storage fix across the
serialized offline workspace suite: 920 library tests, 19 CLI tests, 11
DOSBox tests, 34 examples, 61 JVM tests, 1 language test, 11 general record
tests, 40 record-method tests, and doc-tests pass. The generated tutorial
fixture is restored from `HEAD`, and `git diff --check` passes.

Iteration 277 makes C error-runtime capability selection consume typed
`TRY`, `ON ERROR GOTO`, and `RESUME` statements whenever semantic IR is
available. The compatibility AST scan remains for generated-parser fallback
only. A differential regression replaces the AST `PRINT` at a typed `PRINT`
source position with stale `TRY` syntax and verifies C emits no error-runtime
globals. The serialized offline workspace suite passes: 921 library tests,
19 CLI tests, 11 DOSBox tests, 34 examples, 61 JVM tests, 1 language test,
11 general record tests, 40 record-method tests, and doc-tests. The generated
tutorial fixture is restored from `HEAD`, and `git diff --check` passes.

Iteration 278 adds direct BASIC callable emission for typed default `RETURN`
nodes in both functions and procedures. The differential regression places
an AST return expression at the same source position as a semantic
expressionless return and confirms the typed return controls emitted code.
Focused validation passes; the full workspace suite is pending.

The serialized offline workspace suite passes after iteration 278: 922
library tests, 19 CLI tests, 11 DOSBox tests, 34 examples, 61 JVM tests, 1
language test, 11 general record tests, 40 record-method tests, and doc-tests.
The generated tutorial fixture is restored from `HEAD`, and `git diff --check`
passes.

Iteration 279 migrates C64 float-capability variable classification for
module-scope semantic `DIM` annotations. Resolved INTEGER, LONG, and STRING
bindings replace the AST collector's suffixless-single default in declaration
and identifier checks, avoiding false capability errors when typed IR differs
from its compatibility AST. A differential regression covers integer and
string declarations plus their uses, and a shadowing regression verifies
that a callable-local suffixless DIM remains a C64 float rejection. Focused
validation passes. Full workspace validation is pending.

The serialized offline workspace suite passes after iteration 279: 924
library tests, 19 CLI tests, 11 DOSBox tests, 34 examples, 61 JVM tests, 1
language test, 11 general record tests, 40 record-method tests, and doc-tests.
The generated tutorial fixture is restored from `HEAD`, and `git diff --check`
passes.

Iteration 280 makes shared-file validation and COMMON variable extraction in
the driver consume typed module headers, dependencies, callables, comments,
and DIM declarations when semantic parsing succeeds. AST-based validation
remains the parser compatibility fallback. The existing valid shared-file
compile and invalid-statement diagnostic regressions both pass; full
workspace validation passes: 924 library tests, 19 CLI tests, 11 DOSBox
tests, 34 examples, 61 JVM tests, 1 language test, 11 general record tests,
40 record-method tests, and doc-tests. The generated tutorial fixture is
restored from `HEAD`, and `git diff --check` passes.

Iteration 281 removes JVM top-level and matched-callable scalar declaration
seeding from the compatibility AST when standalone record declarations are
absent. Semantic name scopes and scalar walks provide source bindings, while
`lowered_record_files` carries record-file temporaries and buffers
explicitly. Standalone record member storage still needs the compatibility
collector. A differential context regression verifies AST-only top-level
and callable placeholders allocate no JVM slots; focused declaration and
record regressions pass. Full workspace validation is pending.

The serialized offline workspace suite passes after iteration 281: 924
library tests, 19 CLI tests, 11 DOSBox tests, 34 examples, 61 JVM tests, 1
language test, 11 general record tests, 40 record-method tests, and doc-tests.
The generated tutorial fixture is restored from `HEAD`, and `git diff --check`
passes.

Iteration 282 aligns C64 scalar DIM capability validation with semantic
callable `GLOBAL` scopes. When a semantic global declaration differs from
the compatibility AST, the target validator no longer classifies its use as
a suffixless callable-local single. A differential regression with an
integer module DIM and AST-only callable assignment passes; full workspace
validation passes: 925 library tests, 19 CLI tests, 11 DOSBox tests, 34
examples, 61 JVM tests, 1 language test, 11 general record tests, 40
record-method tests, and doc-tests. The generated tutorial fixture is
restored from `HEAD`, and `git diff --check` passes.

Iteration 283 extends C64 semantic scalar-DIM capability checks into matched
callable scopes. Typed STRING, INTEGER, and LONG annotations now replace the
legacy suffixless-single candidate when no top-level or other-callable float
storage owns that C name. Regressions cover typed callable integers and
shadowing float storage in a second callable; all 21 C64-focused tests pass.
The serialized offline workspace suite passes: 927 library tests, 19 CLI
tests, 11 DOSBox tests, 34 examples, 61 JVM tests, 1 language test, 11
general record tests, 40 record-method tests, and doc-tests. The generated
tutorial fixture is restored from `HEAD`, and `git diff --check` passes.

Iteration 284 removes JVM scalar declaration scans of the compatibility AST
for standalone record storage. The JVM backend derives flattened member
bindings and their scalar types from typed record declarations and record
variable types, including transitive `combines` fields. Focused standalone
record, combined-record runtime, and typed-storage regressions pass. The
serialized offline workspace suite passes: 928 library tests, 19 CLI tests,
11 DOSBox tests, 34
examples, 61 JVM tests, 1 language test, 11 general record tests, and 40
record-method tests. The generated tutorial fixture is restored from `HEAD`,
and `git diff --check` passes.

Iteration 285 makes C's classic error-control capability rejection consume
typed `ON ERROR GOTO`, `RESUME`, and `ERROR` statements whenever semantic IR
is available. Nested statements are traversed in typed IR and diagnostics
retain source filenames and locations; the compatibility-AST walker remains
only for semantic-parser fallback. A differential regression places typed
`ON ERROR GOTO` at a source location represented by AST `PRINT` and confirms
the C target rejects the typed construct. The serialized offline workspace
suite passes: 929 library tests, 19 CLI tests, 11 DOSBox tests, 34 examples,
61 JVM tests, 1 language test, 11 general record tests, and 40 record-method
tests. The generated tutorial fixture is restored from `HEAD`, and
`git diff --check` passes.

Iteration 286 adds typed BASIC emission for callable `FIELD` declarations.
The backend renders the semantic channel, field lengths, and resolved field
names directly, matching the existing top-level path. The callable machine
statement regression supplies typed `FIELD` where the compatibility AST
contains `PRINT` and confirms semantic output wins. The focused regression
and serialized offline workspace suite pass: 929 library tests, 19 CLI
tests, 11 DOSBox tests, 34 examples, 61 JVM tests, 1 language test, 11
general record tests, and 40 record-method tests. The generated tutorial
fixture is restored from `HEAD`, and `git diff --check` passes.

Iteration 287 extends C64 capability validation to semantic `CONST` types
at module and callable scope. Resolved constant types now replace
suffixless-single AST classification introduced by `GLOBAL` references and
callable declaration scans, aligning capability rejection with C constant
storage types. Paired semantic regressions confirm inferred integer
constants are accepted and inferred single-precision constants are rejected
at both scopes. The serialized offline workspace suite passes: 931 library
tests, 19 CLI tests, 11 DOSBox tests, 34 examples, 61 JVM tests, 1 language
test, 11 general record tests, and 40 record-method tests. The generated
tutorial fixture is restored from `HEAD`, and `git diff --check` passes.

Iteration 288 makes the driver's root `shared` declaration lookup consume
the generated semantic `Program` header before resolving the shared COMMON
file. `check_file` follows the same authority rule; the compatibility AST is
used only when semantic parsing fails. A differential driver regression
confirms semantic-header precedence, AST fallback, and that an authoritative
header without `shared` does not inherit a stale AST shared name. The
serialized offline workspace suite passes: 932 library tests, 19 CLI tests,
11 DOSBox tests, 34 examples, 61 JVM tests, 1 language test, 11 general
record tests, and 40 record-method tests. The generated tutorial fixture is
restored from `HEAD`, and `git diff --check` passes.

Iteration 289 makes shared-file parsing semantic-first. When generated semantic parsing succeeds, the driver validates the shared header and derives COMMON DIM facts without first requiring legacy-AST acceptance; AST parsing remains the fallback when semantic parsing fails. The existing shared-file validation regressions and serialized offline workspace suite pass: 932 library tests, 19 CLI tests, 11 DOSBox tests, 34 examples, 61 JVM tests, 1 language test, 11 general record tests, and 40 record-method tests. The generated tutorial fixture is restored from `HEAD`, and `git diff --check` passes.

Iteration 290 extends BASIC semantic record-member lookup through transitive
`combines` declarations. An inherited member now retains its declared field
type and maps to the same flattened BASIC storage name as a directly declared
member. A differential regression confirms typed `row.id` emits the inherited
`rowid%` storage name instead of an AST `PRINT 999`; the serialized offline
workspace suite passes: 933 library
tests, 19 CLI tests, 11 DOSBox tests, 34 examples, 61 JVM tests, 1 language
test, 11 general record tests, and 40 record-method tests. Graphify was
refreshed, the generated tutorial fixture was restored from `HEAD`, and
`git diff --check` passes.

Iteration 291 extends BASIC's semantic record-storage reservation set through
the same `combines` hierarchy. Inherited flattened member identifiers are now
reserved alongside directly declared member identifiers, preventing the
allocator from treating inherited record storage as available generated
names. The composed-record regression verifies both typed member emission and
storage-name reservation. Graphify was refreshed; the serialized offline
workspace suite passes with the same 933 library, 19 CLI, 11 DOSBox, 34
example, 61 JVM, 1 language, 11 general record, and 40 record-method tests.
The generated tutorial fixture was restored from `HEAD`, and
`git diff --check` passes.

Iteration 292 makes resolved semantic assignment-target types authoritative in
C64 floating-point capability validation. Semantic writes to scalar names
now replace the stale compatibility-AST suffixless `single` candidate before
the backend reports unsupported storage, while same-named floating callable
locals remain eligible for rejection. A differential regression confirms a
semantic integer assignment is accepted and emits integer C storage despite
the AST's suffixless assignment. Graphify was refreshed; the serialized
offline workspace suite passes: 934 library tests, 19 CLI tests, 11 DOSBox
tests, 34 examples, 61 JVM tests, 1 language test, 11 general record tests,
and 40 record-method tests. The generated tutorial fixture was restored from
`HEAD`, and `git diff --check` passes.

Iteration 293 extends C64 semantic assignment-target type authority into
matched callable bodies. C scalar declaration collection now omits compatibility
AST storage candidates for aligned semantic assignments and `MID$` assignments,
then derives replacement declarations from semantic target types. Other matched
statement families retain their AST declaration fallback. Regression coverage
verifies a callable-local integer target replaces an AST suffixless `single`
declaration; the serialized offline workspace suite passes: 935 library
tests, 19 CLI tests, 11 DOSBox tests, 34 examples, 61 JVM tests, 1 language
test, 11 general record tests, and 40 record-method tests. Graphify was
refreshed, the generated tutorial fixture was restored from `HEAD`, and
`git diff --check` passes.

Iteration 294 strengthens cross-backend coverage for transitive record
composition. BASIC's differential member-emission and reserved-storage test
now resolves a field inherited through two `combines` edges; JVM's standalone
record slot collector likewise verifies inherited INT16 and INT32 members
through a two-edge composition chain. Both focused tests pass, and
`git diff --check` passes.

Iteration 295 extends semantic callable storage authority in C to matched
`DIM` declarations in addition to scalar and `MID$` assignments. Those
statement families have typed declaration collectors and direct semantic
dispatch; their aligned AST statements no longer seed a competing local
storage type. Other statement families retain AST declaration fallback. The
C64 callable DIM regression now asserts emitted integer storage and absence
of stale floating storage. The serialized offline workspace suite passes:
935 library tests, 19 CLI tests, 11 DOSBox tests, 34 examples, 61 JVM tests,
1 language test, 11 general record tests, and 40 record-method tests. Graphify
was refreshed, the generated tutorial fixture was restored from `HEAD`, and
`git diff --check` passes.

Iteration 296 extends C callable semantic storage authority to matched `INPUT`
and `READ` targets. The typed C emitters and declaration collector now own
these target slots instead of retaining the aligned AST's suffixless
floating-point candidate. C64 differential cases cover both callable-local
input and DATA reads, asserting integer storage and no floating slot. The
serialized offline workspace suite passes: 936 library tests, 19 CLI tests,
11 DOSBox tests, 34 examples, 61 JVM tests, 1 language test, 11 general
record tests, and 40 record-method tests. The generated tutorial fixture was
restored from `HEAD`, and `git diff --check` passes.

Iteration 297 adds matched C callable `LINE INPUT` targets to semantic storage
collection. The emitted C local now follows the typed IR's string target
instead of retaining the compatibility AST's suffixless floating-point
candidate. The combined C64 regression covers `INPUT`, `READ`, and `LINE
INPUT`, checking each resolved slot type and ensuring no stale float slot is
emitted. The serialized offline workspace suite passes: 936 library tests,
19 CLI tests, 11 DOSBox tests, 34 examples, 61 JVM tests, 1 language test,
11 general record tests, and 40 record-method tests. The generated tutorial
fixture was restored from `HEAD`, and `git diff --check` passes.

Iteration 298 corrects JVM semantic scalar-slot collection for `LSET` and
`RSET`: these string operations now establish `JvmType::String` for their
targets even when the target name has no suffix or prior `DIM`. The regression
checks both operations without DIM declarations. Graphify was refreshed; the
serialized offline workspace suite passes: 936 library tests, 19 CLI tests,
11 DOSBox tests, 34 examples, 61 JVM tests, 1 language test, 11 general
record tests, and 40 record-method tests. The generated tutorial fixture was
restored from `HEAD`, and `git diff --check` passes.

Iteration 299 makes callable source identity explicit in typed IR. Named
semantic parsing assigns each callable its source-buffer index, and dependency
merging rebases those indices alongside top-level statement source indices.
C's classic error-control validator now traverses semantic callables directly
and locates diagnostics from typed callable provenance instead of matching
callables against `ResolvedProgram::program.functions`. Regressions verify
callable diagnostics when the legacy AST has no corresponding callable and
verify source identity through two dependency merges. Graphify was refreshed;
the serialized offline workspace suite passes: 938 library tests, 19 CLI
tests, 11 DOSBox tests, 34 examples, 61 JVM tests, 1 language test, 11 general
record tests, and 40 record-method tests. The generated tutorial fixture was
restored from `HEAD`, and `git diff --check` passes.

Iteration 300 makes BASIC, C, and JVM callable-statement source alignment use
the `source_index` retained by the matched semantic callable. The three
backends no longer choose the semantic source buffer by comparing the legacy
AST function's filename. The existing required-library differential test
passes across all three backends with merged dependency sources; formatting
and `git diff --check` pass. Graphify was refreshed; the serialized offline
workspace suite passes: 938 library tests, 19 CLI tests, 11 DOSBox tests, 34
examples, 61 JVM tests, 1 language test, 11 general record tests, and 40
record-method tests. The generated tutorial fixture was restored from `HEAD`.

Iteration 301 consolidates JVM semantic callable selection across method
signature construction, callable-local slot collection, and source-aligned
statement dispatch. All three consumers now require matching callable kind,
receiver presence, and parameter arity, prefer an exact receiver type, and
decline ambiguous fallback matches. A regression verifies that a semantic
callable with mismatched arity cannot claim an AST callable body. Graphify was
refreshed; the serialized offline workspace suite passes: 939 library tests,
19 CLI tests, 11 DOSBox tests, 34 examples, 61 JVM tests, 1 language test, 11
general record tests, and 40 record-method tests. The generated tutorial
fixture was restored from `HEAD`, and `git diff --check` passes.

Iteration 302 moves BASIC inferred-array-capacity call-site discovery onto
semantic expressions whenever the generated semantic module is available.
The collector traverses top-level and callable typed statements, including
nested expressions, and resolves whole-array arguments against typed callable
parameters and typed DIM bounds. AST call-site and DIM scans now remain only
for parser compatibility or callable bodies that decline semantic source
alignment. A differential regression verifies that a call present only in
typed IR still supplies the inferred capacity; legacy unmatched-callable
fallback and `name()` whole-array passing remain covered. Graphify was
refreshed; the serialized offline workspace suite passes: 940 library tests,
19 CLI tests, 11 DOSBox tests, 34 examples, 61 JVM tests, 1 language test, 11
general record tests, and 40 record-method tests. The generated tutorial
fixture was restored from `HEAD`, and `git diff --check` passes.

Iteration 303 completes semantic dispatch for single-line BASIC `IF`/`ELSE`.
The generated grammar now terminates the `PRINT` token repetition and the
single-line `IF` statement repetition at `ELSE`, allowing the semantic
adapter to retain distinct then/else statement lists. BASIC module and
callable emitters transpile both branches from typed IR; differential tests
verify AST output does not leak, and the existing greedy-`PRINT` regression
passes. Graphify was refreshed; the serialized offline workspace suite
passes: 943 library tests, 19 CLI tests, 11 DOSBox tests, 34 examples, 61 JVM
tests, 1 language test, 11 general record tests, and 40 record-method tests.

Iteration 304 adds semantic C `FOR` emission for top-level and callable loops
whose typed bodies contain assignments, standard `PRINT`, `EXIT`, and
`CONTINUE`. The loop variable type comes from its typed identifier or
semantic scalar `DIM` annotation; start, limit, and step expressions render
from typed IR, and limit/step values are captured once at loop entry. Other
body forms retain the aligned AST fallback. C64 and callable differential
regressions verify that a semantic `LONG` loop variable replaces the stale
AST single-precision slot and that loop output references the typed slot.
Graphify was refreshed; the serialized offline workspace suite passes: 945
library tests, 19 CLI tests, 11 DOSBox tests, 34 examples, 61 JVM tests, 1
language test, 11 general record tests, and 40 record-method tests.

Iteration 305 extends typed C `FOR` bodies to nested semantic `IF`/`ELSE`
statements. Conditions and both branch bodies render from typed IR, and the
loop emitter still stages output so an unsupported descendant declines the
whole semantic loop before committing partial code. A C64 differential
regression verifies typed `LONG` storage and branch output replace the
compatibility AST's suffixless loop and `IF` bodies. Graphify was refreshed;
the serialized offline workspace suite passes: 946 library tests, 19 CLI
tests, 11 DOSBox tests, 34 examples, 61 JVM tests, 1 language test, 11
general record tests, and 40 record-method tests.

Iteration 306 extends typed C `FOR` bodies to nested semantic `WHILE` loops.
The C backend emits each typed condition and recursively dispatches the nested
body, preserving native innermost-loop `EXIT`/`CONTINUE` behavior. A C64
differential regression verifies that the `WHILE` condition, increment, and
output use the semantic `LONG` loop variable rather than AST-derived
single-precision storage. The serialized offline workspace suite passes:
947 library tests, 19 CLI tests, 11 DOSBox tests, 34 examples, 61 JVM tests,
1 language test, 11 general record tests, and 40 record-method tests.

Iteration 391 removes the resolver's AST parameter-count gate when associating
typed callables for missing-return validation. A semantic callable with the
same resolved name, kind, and receiver now supplies the return facts even if
the compatibility AST has a stale parameter layout. A resolver regression
confirms that the typed callable's return prevents a false missing-return
diagnostic. The focused resolver regression and full workspace validation
pass.

Iteration 392 makes BASIC semantic function-call emission append omitted
arguments from the typed callable's `Parameter.default` expressions. A
differential test gives the AST default `3` and typed IR default `8`, then
verifies the BASIC setup assigns `8` and never emits `3` for that parameter.

Iteration 393 routes callable-local semantic assignments through the same
typed expression-prelude path as module assignments, including direct
function calls. A differential regression changes a callable's RHS argument
from AST value `1` to typed value `7` and verifies that the callable-local
parameter binding and result assignment use typed IR. The full locked
workspace suite passes: 998 library tests, 19 CLI tests, 11 DOSBox tests, 35
examples, 62 JVM tests, 1 language test, 11 general record tests, and 40
record-method tests.

Graphify was refreshed.

Iteration 307 adds typed C `DO` emission inside semantic `FOR` bodies. The
emitter handles precondition and postcondition guards for `WHILE` and `UNTIL`,
recursively emits typed body statements, and stages the loop output before
committing it. A C64 differential regression verifies that a nested post-
condition `DO ... LOOP UNTIL` uses the semantic `LONG` loop variable and
semantic body output, and confirms `CONTINUE` branches to the postcondition
label. The fixture keeps the AST and semantic source line positions aligned;
otherwise C's all-or-nothing top-level source map correctly declines the
semantic stream when a later statement moves to a different source line. The
focused regression and serialized offline workspace suite pass: 948
library tests, 19 CLI tests, 11 DOSBox tests, 34 examples, 61 JVM tests, 1
language test, 11 general record tests, and 40 record-method tests. The
generated tutorial fixture was restored from `HEAD`, Graphify was refreshed,
and `git diff --check` passes.

Iteration 308 promotes the C semantic `DO` emitter to top-level source-aligned
dispatch, in addition to its nested `FOR` use. The emitter stages generated C,
math requirements, temporary identifiers, and loop-continuation context, so
an unsupported nested statement declines without leaving partial output before
the matching AST fallback. C64 differential regressions cover precondition
and postcondition guards, typed `LONG` storage, `CONTINUE` routing through the
postcondition label, and semantic output replacing AST output. Existing
continue assertions now check the semantic label prefix. The serialized
offline workspace suite passes: 949 library tests, 19 CLI tests, 11 DOSBox
tests, 34 examples, 61 JVM tests, 1 language test, 11 general record tests,
and 40 record-method tests. The generated tutorial fixture was restored from
`HEAD`.

Iteration 309 extends typed C loop-body dispatch to nested semantic `FOR`
loops. Each nested loop selects its variable type from the semantic declaration
scope and reuses the atomic typed-loop emitter, preserving nearest-loop native
`EXIT`/`CONTINUE` behavior. A C64 differential regression checks nested
`LONG` and `INTEGER` loop storage and verifies both typed outputs replace the
compatibility AST values. The serialized offline workspace suite passes: 950
library tests, 19 CLI tests, 11 DOSBox tests, 34 examples, 61 JVM tests, 1
language test, 11 general record tests, and 40 record-method tests. The
generated tutorial fixture was restored from `HEAD`.

Iteration 310 adds top-level JVM semantic `TRY/CATCH/FINALLY` dispatch. The
emitter consumes typed catch error and line bindings, numeric error filters,
source-file bindings, and typed try/catch/finally statement bodies. It stages
the bytecode and exception-table entry so unsupported nested statements
decline without partial output. A source-aligned differential test confirms
semantic throw, catch, and finally values replace stale AST values; runnable
JVM conformance tests pass for ordinary catch/finally and filtered catches
with source bindings. A callable-body differential regression also verifies
typed exception handling replaces its AST body. The serialized offline
workspace suite passes: 952 library tests, 19 CLI tests, 11 DOSBox tests, 34
examples, 61 JVM tests, 1 language test, 11 general record tests, and 40
record-method tests. The generated tutorial fixture was restored from `HEAD`.

Iteration 311 extends typed JVM exception dispatch into nested semantic
control flow. `TRY/CATCH/FINALLY` now carries source filename, label allocation,
exception-table entries, and active loop-transfer targets through typed `IF`,
`FOR`, `WHILE`, `DO`, and `SELECT CASE` bodies. Per-root-statement staging
commits handler metadata only when typed emission succeeds; nested block
failure restores handler state before the matching AST fallback. A differential
regression covers typed `TRY` nested in `IF` inside `FOR`. Both existing JVM
runtime exception tests and the serialized offline workspace suite pass: 953
library tests, 19 CLI tests, 11 DOSBox tests, 34 examples, 61 JVM tests, 1
language test, 11 general record tests, and 40 record-method tests. Graphify
was refreshed, the generated tutorial fixture was restored from `HEAD`, and
`git diff --check` passes.

Iteration 312 extends typed C `FOR` bodies to nested semantic `SELECT CASE`.
The C64 emitter retains numeric selector type, evaluates the selector once,
and renders value, range, and comparison clauses from semantic expressions;
case bodies recurse through the active loop context so `EXIT` and `CONTINUE`
remain attached to the enclosing loop. Selector temporaries, math requirements,
temporary allocation, and generated body text are staged and committed only
when the complete typed construct is supported. A C64 differential regression
covers typed `LONG` selection, value/range/comparison/else branches, and
`CONTINUE`, and confirms stale AST output does not leak. A callable C64
differential regression verifies the same nested dispatcher selects typed
callable output instead of stale AST output. The serialized offline workspace
suite passes: 955 library tests, 19 CLI tests, 11 DOSBox tests, 34
examples, 61 JVM tests, 1 language test, 11 general record tests, and 40
record-method tests. Graphify was refreshed, the generated tutorial fixture
was restored from `HEAD`, and `git diff --check` passes.

Iteration 313 promotes numeric C semantic `SELECT CASE` to source-aligned
top-level dispatch, reusing the typed selector and clause renderer added in
Iteration 312. The selector is evaluated once into a typed temporary; value,
range, comparison, and else clauses are emitted from typed IR. The emitter
preserves C's established `bt_sel_*` temporary and match-guard layout. A C64
differential regression changes the selector type, selector value, and every
case value between AST and semantic source, proving typed conditions and
bodies replace the compatibility output. The serialized offline workspace
suite passes: 956 library tests, 19 CLI tests, 11 DOSBox tests, 34 examples,
61 JVM tests, 1 language test, 11 general record tests, and 40 record-method
tests. Graphify was refreshed, the generated tutorial fixture was restored
from `HEAD`, and `git diff --check` passes.

Iteration 314 adds source-aligned C semantic `SELECT CASE` dispatch for
callable-body root statements. The callable emitter reuses the typed selector,
case-clause, and nested-body renderer while passing the callable's semantic
declaration scope and active loop-transfer context. A C64 differential
regression changes a callable-local selector from the compatibility AST's
default numeric storage to typed `LONG` and verifies typed case bodies replace
the AST output. The serialized offline workspace suite passes: 957 library
tests, 19 CLI tests, 11 DOSBox tests, 34 examples, 61 JVM tests, 1 language
test, 11 general record tests, and 40 record-method tests. Graphify was
refreshed, the generated tutorial fixture was restored from `HEAD`, and
`git diff --check` passes.

Iteration 315 promotes source-aligned C semantic `IF` and `WHILE` root
statements in module and callable bodies. Both wrappers stage rendered body,
math requirements, temporary allocation, and loop context, and recursively
transpile supported nested statements from typed IR. `WHILE` establishes the
active loop context so nested `CONTINUE` targets the C loop; declined bodies
leave state available to the aligned AST fallback. C64 differential tests
cover module and callable roots, typed `LONG` storage, nested branches, and
loop transfers. The existing short-circuit regression now checks operand
presence and source order around C `&&`, rather than depending on the legacy
AST emitter's exact parentheses after typed boolean rendering. The serialized
offline workspace suite passes: 959 library tests, 19 CLI tests, 11 DOSBox
tests, 34 examples, 61 JVM tests, 1 language test, 11 general record tests,
and 40 record-method tests. Graphify was refreshed, the generated tutorial
fixture was restored from `HEAD`, and `git diff --check` passes.

Iteration 316 promotes source-aligned C semantic `IF` and `WHILE` root
statements in module and callable bodies, plus callable root `DO`. The
wrappers stage rendered bodies, math requirements, temporary allocation, and
loop context while recursively transpiling supported nested statements from
typed IR. `WHILE` establishes the active loop context so nested `CONTINUE`
targets the C loop; postcondition `DO` routes `CONTINUE` through its condition
label. C64 differential regressions cover typed `LONG` storage, nested
branches and loop transfers, and typed callable `DO` output replacing stale
AST output. The existing short-circuit regression checks operand presence and
source order around C `&&`, rather than depending on the legacy AST emitter's
exact parentheses after typed boolean rendering. The serialized offline
workspace suite passes: 960 library tests, 19 CLI tests, 11 DOSBox tests, 34
examples, 61 JVM tests, 1 language test, 11 general record tests, and 40
record-method tests. Graphify was refreshed, the generated tutorial fixture
was restored from `HEAD`, and `git diff --check` passes.

Iteration 317 extends C semantic `SELECT CASE` dispatch to string selectors
and exact-match string `CASE` values. The selector is transpiled once into the
backend's fixed-width string buffer, and typed case expressions use `strcmp`;
the `<string.h>` runtime requirement is staged with generated text and
committed only when the full selection is supported. String-use state now
flows through staged `IF`, `WHILE`, `DO`, and `FOR` bodies so nested selections
retain the runtime include. Differential regressions cover module-root,
callable-root, and loop-nested string selection and verify typed values replace
stale AST output. The serialized offline workspace suite passes: 963 library
tests, 19 CLI tests, 11 DOSBox tests, 34 examples, 61 JVM tests, 1 language
test, 11 general record tests, and 40 record-method tests. Graphify was
refreshed, the generated tutorial fixture was restored from `HEAD`, and
`git diff --check` passes.

Iteration 318 extends Stage 67's driver-boundary differential fixture matrix
with string `SELECT CASE`, comparing ordinary resolved output against the
AST-only compatibility path for BASIC, C, and JVM. This exposed a BASIC typed
emitter indentation mismatch on the per-case exit `GOTO`; top-level and
callable semantic `SELECT CASE` now retain the same nested statement layout
as the AST emitter. The focused differential and serialized offline workspace
suite pass: 963 library tests, 19 CLI tests, 11 DOSBox tests, 34 examples,
61 JVM tests, 1 language test, 11 general record tests, and 40 record-method
tests. The generated tutorial fixture was restored from `HEAD`, and
`git diff --check` passes.

Iteration 319 adds module-root JVM semantic dispatch for `GET`, `PUT`,
`LSET`, and `RSET`, reusing the typed record-file and field-set emitters
already used in nested semantic blocks. A regression supplies different
record positions and field values in the resolved typed IR and compatibility
AST, proving module-root output uses the typed statements. The focused JVM
regression and full serialized offline workspace suite pass: 964 library
tests, 19 CLI tests, 11 DOSBox tests, 34 examples, 61 JVM tests, 1 language
test, 11 general record tests, and 40 record-method tests. Doc tests also
pass, and `git diff --check` passes.

Iteration 320 extends Stage 67 record-file differential coverage to
module-root `FIELD`, `LSET`, `RSET`, `PUT`, and `GET`. BASIC output and the C
`main` statement body match between semantic and AST-only compilation. The C
whole-file output has separate backend runtime-helper staging, so this fixture
compares emitted module statements rather than unrelated helper declarations;
JVM typed-vs-AST values and record positions are covered by Iteration 319's
focused backend regression. The driver-boundary differential test passes.

Iteration 321 excludes JVM `FIELD` buffer names from semantic scalar-slot
allocation after collecting typed field layouts. The Stage 67 same-source
record-file fixture exposed an unnecessary static `String` slot for a field
buffer. Field-set targets alone no longer create scalar slots, while explicit
ordinary assignments to those names still do. The semantic root dispatch
emits typed `LSET`, `RSET`, `GET`, and `PUT`.
The JVM module-root regression, nested field-set and layout runtime regressions,
BASIC/JVM differential cases, C module-body differential, and full serialized
offline workspace suite pass: 964 library tests, 19 CLI tests, 11 DOSBox tests,
34 examples, 61 JVM tests, 1 language test, 11 general record tests, and 40
record-method tests. Doc tests also pass; `git diff --check` passes.

Iteration 322 makes C callable `DATA` nodes an explicit typed-IR no-op during
per-statement emission. Their payload is already collected from semantic IR
into the module-wide data table; this removes an unnecessary AST fallback at
each callable `DATA` source position. The existing stale-AST regression with
different semantic `DATA` values and `READ` targets passes. The full serialized
offline workspace suite passes: 964 library tests, 19 CLI tests, 11 DOSBox
tests, 34 examples, 61 JVM tests, 1 language test, 11 general record tests,
and 40 record-method tests; doc tests and `git diff --check` pass.

Iteration 323 adds typed C callable-statement emission for semantic procedure
calls and numeric/string function calls whose results are discarded. Numeric
and string BYVAL arguments, numeric and string BYREF bindings, and resolved
trailing defaults are rendered from the semantic callable signature; string
function calls retain their result buffer. Argument expressions are
restricted to side-effect-free forms so C's unspecified argument evaluation
order cannot change BASCAL evaluation order. Array parameters and try-result
callables retain the existing compatibility emission path pending their
array ABI and status propagation handling. The stale-AST callable regression
verifies numeric and string function calls, procedure arguments, scalar
BYREF address passing, and trailing default insertion while replacing the
aligned AST statement. The full
serialized offline workspace suite passes: 965 library tests, 19 CLI tests,
11 DOSBox tests, 34 examples, 61 JVM tests, 1 language test, 11 general
record tests, and 40 record-method tests; doc tests and `git diff --check`
pass.

Iteration 324 extends the Stage 67 driver-boundary fixture helper so AST and
semantic source text can differ while retaining the same legacy AST parse for
resolver setup. A C callable fixture now verifies end-to-end semantic
statement precedence: the typed procedure call with numeric/string arguments,
BYREF storage, and a default parameter appears in generated C, while the
AST-only compatibility result retains its original `PRINT`. The focused
driver-boundary differential and full serialized offline workspace suite
pass: 965 library tests, 19 CLI tests, 11 DOSBox tests, 34 examples, 61 JVM
tests, 1 language test, 11 general record tests, and 40 record-method tests;
doc tests and `git diff --check` pass.

Iteration 325 extends typed C callable-statement emission to calls used as
BYVAL arguments when each nested call is the direct argument expression and
its own arguments are side-effect-free. BYVAL arguments are materialized into
typed temporaries in source parameter order before the outer C invocation,
preserving BASCAL evaluation order despite C's unspecified argument order.
Scalar BYREF bindings remain direct references; unsupported nested side effects,
array parameters, and try-result callables still use the existing compatibility
emission path. The stale-AST backend regression now invokes numeric and string
functions inside a procedure call, and the Stage 67 driver fixture checks the
materialized semantic call against the AST-only compatibility output. The full
serialized offline workspace suite passes: 965 library tests, 19 CLI tests,
11 DOSBox tests, 34 examples, 61 JVM tests, 1 language test, 11 general record
tests, and 40 record-method tests; doc tests and `git diff --check` pass.

Iteration 326 extends Stage 67 callable-statement driver differentials to
BASIC and JVM after the expanded fixture exposed callable-body dispatch gaps
in both backends. BASIC now emits aligned semantic procedure calls with
fully supplied scalar BYVAL and type-matched scalar BYREF arguments,
rendering BYVAL expressions from typed IR and emitting BYREF copy-in,
`GOSUB`, and copyback; omitted scalar defaults are rendered from the semantic
callable signature. Array arguments retain per-statement compatibility
emission. JVM semantic
call dispatch now accepts void signatures and emits typed procedure calls
with semantic defaults, scalar BYREF writeback, and array ABI handling from
the existing shared callable-call path. The driver fixture confirms semantic
invocations replace stale AST `PRINT` statements on BASIC, C, and JVM, while
the AST-only compatibility output retains the original statements; BASIC and
JVM default argument emission is also covered. The full
serialized offline workspace suite passes: 965 library tests, 19 CLI tests,
11 DOSBox tests, 34 examples, 61 JVM tests, 1 language test, 11 general record
tests, and 40 record-method tests; doc tests and `git diff --check` pass.

Iteration 327 extends semantic BASIC callable-statement dispatch to typed
array parameters. The call emitter uses semantic parameter rank, element type,
passing mode, and default expression, resolves the caller's declared rank and
per-axis bounds, checks the callee's inferred storage capacities, then emits
array copy-in and BYREF copyback using the existing BASIC array ABI. A
Stage 67 driver differential verifies an aligned semantic BYREF array call
replaces a stale AST `PRINT` and emits both copy directions. The full
serialized offline workspace suite passes: 965 library tests, 19 CLI tests,
11 DOSBox tests, 34 examples, 61 JVM tests, 1 language test, 11 general record
tests, and 40 record-method tests; doc tests and `git diff --check` pass.

Iteration 328 extends semantic C callable-statement dispatch to scalar
procedure and numeric/string function calls whose callees use the try-result
ABI. Typed BYVAL and BYREF arguments and semantic defaults are materialized
through the signature renderer; string calls receive the required output
buffer. The emitted status is checked and propagated through the caller's
matching result wrapper, preserving structured `throw` handling. The Stage 67
C driver differential replaces AST-only procedure calls with typed numeric
and string function calls that supply scalar arguments and omit defaults,
checking result wrappers, the string buffer, and status propagation. The full
serialized offline workspace suite passes: 965 library tests, 19 CLI tests,
11 DOSBox tests, 34 examples, 61 JVM tests, 1 language test, 11 general record
tests, and 40 record-method tests; doc tests and `git diff --check` pass.

Iteration 329 extends typed C callable-statement argument marshalling to
numeric array parameters supported by the backend ABI. Semantic array names
are checked against the resolved parameter rank and element type; the call
passes static or runtime bounds and the correctly shaped data pointer. A
Stage 67 differential confirms an aligned semantic BYREF array call replaces
the AST statement. The full serialized offline workspace suite passes: 965
library tests, 19 CLI tests, 11 DOSBox tests, 34 examples, 61 JVM tests,
1 language test, 11 general record tests, and 40 record-method tests; doc
tests and `git diff --check` pass.

Iteration 330 adds typed C callable-statement dispatch at module scope, using
the signature-aware argument marshaller already used in callable bodies.
Root-level calls now preserve BYVAL temporaries, typed default arguments,
string buffer sizing, numeric array ABI arguments, and discarded function
results; the Stage 67 differential verifies a semantic root call replaces
the stale AST statement. Focused callable and differential tests pass, as
does the full offline workspace suite: 965 library tests, 19 CLI tests, 11
DOSBox tests, 34 examples, 61 JVM tests, 1 language test, 11 general record
tests, and 40 record-method tests. Graphify was refreshed and `git diff
--check` passes.

Iteration 331 completes module-root callable-statement differential coverage
for BASIC, C, and JVM, and checks discarded function calls inside a JVM
callable body. The new JVM regression exposed that typed call
dispatch accepted only void signatures; it now emits typed function calls
used as statements and discards their result with the correct `pop` or
`pop2` instruction. Callable-body dispatch uses the same result-discard
path. The focused Stage 67 differential and full offline workspace suite
pass: 965 library tests, 19 CLI tests, 11 DOSBox tests, 34 examples, 61 JVM
tests, 1 language test, 11 general record tests, and 40 record-method tests.
Graphify was refreshed and `git diff --check` passes.

Iteration 332 extends the JVM discarded-result differential to LONG and
DOUBLE function signatures, confirming both category-2 results use `pop2`
after the typed invocation. The semantic fixture is explicitly checked for
successful adaptation, and its AST-only callable body contains stale PRINT
statements that must be replaced. The focused Stage 67 differential and full
offline workspace suite pass: 965 library tests, 19 CLI tests, 11 DOSBox
tests, 34 examples, 61 JVM tests, 1 language test, 11 general record tests,
and 40 record-method tests. The generated tutorial fixture is restored from
`HEAD`, and `git diff --check` passes.

Iteration 333 migrates C runtime-helper selection off aligned AST statements
when semantic IR supplies their typed facts. Builtin calls (including
string-returning `$` names and scalar member methods), `INKEY$`/`DATE$`, and
typed string comparisons now drive helper requirements. AST expression
scanning remains for unsupported or unmapped compatibility statements and
synthesized record procedures, which do not have semantic callable bodies.
A Stage 67 differential confirms stale AST-only `LEN` and bare `DATE$`
references no longer add runtime support to semantic C output, while AST-only
compatibility output still selects both helpers. Tests also caught and fixed
missing discovery for semantic `CHR$`, `RIGHT$`, `STR$`, and synthesized
record `MID$` calls. The full
offline workspace suite passes: 965 library tests, 19 CLI tests, 11 DOSBox
tests, 34 examples, 61 JVM tests, 1 language test, 11 general record tests,
and 40 record-method tests. Graphify was refreshed, the generated tutorial
fixture was restored from `HEAD`, and `git diff --check` passes.

Iteration 334 corrects C64 capability validation when semantic scalar
assignments or typed DIM declarations differ from their compatibility-AST
twins. Typed module and callable assignments and scalar DIM annotations now
remove stale suffix-derived AST storage candidates, while retaining
same-named callable-local floating storage where it remains semantically
present. Stage 67 C64 differentials cover module assignments, callable
assignments, and typed DIMs at both scopes; the emitted storage uses the
semantic INTEGER binding even when the AST twin uses a DOUBLE suffix. The
full offline workspace suite passes: 965 library tests, 19 CLI tests, 11
DOSBox tests, 34 examples, 61 JVM tests, 1 language test, 11 general record
tests, and 40 record-method tests. Graphify was refreshed, the generated
tutorial fixture was restored from `HEAD`, and `git diff --check` passes.

Iteration 335 makes C64 float-operation capability diagnostics consume typed
semantic expressions whenever the generated semantic module is available.
Float literals, true division, exponentiation, and float-producing builtins
are checked from semantic expressions and source spans; compatibility-AST
operation checks remain for aligned statements marked `Unsupported` and for
AST-only callers. Stage 67 differentials verify that a semantic float literal
or true division cannot escape rejection when its AST twin is integer, and
that stale AST-only float literals or true division do not reject semantic
integer expressions. The focused differential, all 42 C64 library tests, and
the full offline workspace suite pass: 966 library tests, 19 CLI tests, 11
DOSBox tests, 34 examples, 61 JVM tests, 1 language test, 11 general record
tests, and 40 record-method tests. Graphify was refreshed, the generated
tutorial fixture was restored from `HEAD`, and `git diff --check` passes.

Iteration 336 makes JVM semantic label discovery recursive across typed
`IF`, loop, `SELECT CASE`, and `TRY` bodies. The semantic block emitter now
emits typed label declarations, so a top-level semantic `GOTO` can target a
label nested in a typed block even when the compatibility AST uses a
differently named label. A compatibility-AST-versus-semantic-IR differential
verifies the generated branch and label both use the semantic name. An
initial broader nested-GOTO emission attempt violated the JVM block emitter's
atomic compatibility fallback; that change was removed, and the regression
now covers only typed label emission and recursive discovery. All 101 JVM
library tests and the full offline workspace suite pass: 967 library tests,
19 CLI tests, 11
DOSBox tests, 34 examples, 61 JVM integration tests, 1 language test, 11
general record tests, and 40 record-method tests. Graphify was refreshed,
the generated tutorial fixture was restored from `HEAD`, and formatting and
`git diff --check` pass.

Iteration 337 aligns C64 floating-variable checks with the semantic scalar
names used by C declaration emission. Float candidates from AST statements
replaced by supported typed-IR statements are pruned, while names from
unmatched and `Unsupported` statements retain compatibility-AST checking.
Typed callable parameters, results, and scalar locals are retained in their
semantic scopes, including same-named locals whose types differ between
callables. A regression verifies that an obsolete AST-only DOUBLE assignment
does not reject a semantic INTEGER assignment, while a semantic DOUBLE read
still receives a C64 diagnostic when its AST twin is INTEGER. The focused
regressions, all 36 `target_c64_` tests, and the full offline workspace suite
pass: 968 library tests, 19 CLI tests, 11 DOSBox tests, 34 examples, 61 JVM
integration tests, 1 language test, 11 general record tests, and 40
record-method tests. Graphify was refreshed, the generated tutorial fixture
was restored from `HEAD`, and formatting and `git diff --check` pass.

Iteration 338 makes BASIC FIELD-buffer classification semantic-first whenever
typed IR is present. Module and callable FIELD bindings and lowered record
file buffers now supply the global-buffer set; compatibility-AST FIELD names
are retained only where there is no matching supported semantic source
statement, preserving fallback emission. A differential regression replaces
an AST FIELD declaration with a semantic `BEEP` and verifies a same-named
callable-local string still receives callable-local allocation and emits its
typed initializer and return. The full offline workspace suite passes: 969
library tests, 19 CLI tests, 11 DOSBox tests, 34 examples, 61 JVM integration
tests, 1 language test, 11 general record tests, and 40 record-method tests.
Graphify was refreshed, the generated tutorial fixture was restored from
`HEAD`, and formatting and `git diff --check` pass.

Iteration 339 keeps C64's by-value array capability check tied to the
receiver-aware callable declaration the C backend will emit, while reporting
the rejection at the generated semantic callable's source identity and name
span. A differential regression uses a distinct compatibility-AST filename
and confirms the diagnostic points to the typed source instead. All 36
`target_c64_` tests pass, including the overloaded method receiver regression;
`git diff --check` passes. A broader attempt to validate every semantic
callable independently was rejected: the current C backend still emits AST-
selected callable overloads, so that check incorrectly rejected an unused
BYVAL overload. Fully removing this coupling depends on migrating callable
selection and emission together.

Iteration 340 replaces the driver's semantic FBC `contains_try` flag and
root-level fallback location with typed statement traversal. Every `TRY` in
module and callable bodies, including nested `IF`/loop/`SELECT CASE` blocks,
now yields a diagnostic at its source statement; leading trivia in the
generated statement span is skipped to locate the keyword. The AST traversal
remains only for parser compatibility. Driver regressions cover nested and
callable `TRY` diagnostics plus unchanged FBC output when no `TRY` exists.
The focused tests and full offline workspace suite pass: 969 library tests,
19 CLI tests, 11 DOSBox tests, 34 examples, 61 JVM integration tests, 1
language test, 11 general record tests, and 40 record-method tests. Graphify
was refreshed, the generated tutorial fixture was restored from `HEAD`,
changed-region formatting checks pass, and `git diff --check` passes.

Iteration 341 changes callable-body repetition terminators to the complete
`END FUNCTION`, `END PROCEDURE`, and `END METHOD` phrases. This allows a
bare `END` in a callable body to remain a typed statement instead of
prematurely terminating semantic parsing. BASIC callable emission now handles
that typed `END` directly; a differential regression confirms it replaces a
same-position AST `PRINT`. The full offline workspace suite passes: 970
library tests, 19 CLI tests, 11 DOSBox tests, 34 examples, 61 JVM integration
tests, 1 language test, 11 general record tests, and 40 record-method tests.
Graphify was refreshed; changed-region formatting and `git diff --check`
pass.

Iteration 342 extends the qualified terminator handling to block `IF` and
`SELECT CASE` bodies: repetitions now stop at `END IF` and `END SELECT`, so
a bare `END` inside those blocks is retained as a statement. The callable
END differential now places semantic END nodes inside both an IF body and a
CASE body and verifies their AST PRINT counterparts do not leak into output.
The focused regression and full offline workspace suite pass: 970 library
tests, 19 CLI tests, 11 DOSBox tests, 34 examples, 61 JVM integration tests,
1 language test, 11 general record tests, and 40 record-method tests.
Graphify was refreshed, the generated tutorial fixture was restored from
`HEAD`, changed-region formatting checks pass, and `git diff --check` passes.

Iteration 343 adds JVM callable differentials for typed `END`: an aligned
semantic `END` emits `System.exit(0)` and suppresses the stale compatibility
AST body, while the following semantic return remains verifier-valid. A
runtime test confirms statements after the call do not execute. The full
offline workspace suite passes: 972 library tests, 19 CLI tests, 11 DOSBox
tests, 34 examples, 62 JVM integration tests, 1 language test, 11 general
record tests, and 40 record-method tests.

Iteration 344 adds the matching C callable differential: aligned typed `END`
emits process exit and suppresses the compatibility AST's stale `PRINT` and
`RETURN`. Its focused C regression passes, and the full offline workspace
suite passes with the C and JVM regressions included. The C termination
comment now accurately distinguishes module-scope `END` from callable `END`.

Iteration 345 fixes the C runtime dependency gate for callable `END`. The
semantic IR visitor now exposes callable-only statement queries; C requests
`<stdlib.h>` for callable `END` while keeping module-scope `END` on its
`main`-return path. The compatibility-AST fallback applies the same scope
rule. GCC runtime coverage confirms a callable `END` terminates the process
before later statements. Focused regressions and the full offline workspace
suite pass: 972 library tests, 19 CLI tests, 11 DOSBox tests, 35 examples,
62 JVM integration tests, 1 language test, 11 general record tests, and 40
record-method tests.

Iteration 346 extends the C runtime regression to place typed `END` inside a
callable `IF`, verifying recursive callable-scope discovery, and asserts that
module-scope typed `END` does not add `<stdlib.h>`. Both focused regressions
and the full offline workspace suite pass with the same 972/19/11/35/62/1/11/40
test counts.

Iteration 347 makes explicit-return presence validation semantic-first in
`resolve_with_semantic`: a matched typed callable body now decides whether a
function contains `RETURN`, while AST-only `resolve` and unmatched callables
retain the compatibility check. C's function fallthrough proof now consumes
the matched semantic body and accepts paths that return or terminate the
process. Differentials verify both directions: typed `RETURN` overrides an
AST-only `PRINT`, and an AST-only `RETURN` cannot hide typed fallthrough. The
full offline workspace suite passes: 974 library tests, 19 CLI tests, 11
DOSBox tests, 35 examples, 62 JVM tests, 1 language test, 11 general record
tests, and 40 record-method tests.

Iteration 348 extends typed C callable-block emission to `RETURN` inside
semantic `IF`/`ELSE` branches. Return expressions use the matched callable's
typed result and preserve BYREF copyback and try-result wrappers. The flow
regression confirms both branches emit typed returns, while an `IF` without
`ELSE` is rejected despite an AST-only return. Module-scope `RETURN` keeps
its existing compatibility path. The full offline workspace suite passes:
975 library tests, 19 CLI tests, 11 DOSBox tests, 35 examples, 62 JVM tests,
1 language test, 11 general record tests, and 40 record-method tests.

Iteration 349 propagates the matched C callable ABI through nested semantic
`WHILE`, `DO`, `FOR`, and `SELECT CASE` emitters, allowing nested block
emission to retain typed `RETURN` handling instead of falling back to AST
statements. The differential regression places `SELECT CASE` inside a
callable `IF` and confirms each typed return is emitted while the stale AST
body is suppressed; the missing-`ELSE` fallthrough diagnostic remains
covered. The focused regression and full offline workspace suite pass: 975
library tests, 19 CLI tests, 11 DOSBox tests, 35 examples, 62 JVM tests, 1
language test, 11 general record tests, and 40 record-method tests. The
broader migration remains open across BASIC statement emission, C/JVM
declaration and type facts, driver differentials, and removal of the private
AST compatibility view.

Iteration 350 extends the C typed-return differential to a semantic callable
`WHILE` followed by a typed return. Source-position alignment is retained by
leaving blank AST lines where the semantic loop terminator and nested return
occur; the emitted C contains both typed return expressions and omits the
stale AST `PRINT`. The first full-suite run hit a transient FreeBASIC
`Text file busy` launch failure in `freebasic_runs_remline_when_available`;
that test passed independently, and the repeated full offline suite passes:
975 library tests, 19 CLI tests, 11 DOSBox tests, 35 examples, 62 JVM tests,
1 language test, 11 general record tests, and 40 record-method tests.

Iteration 351 adds the equivalent C differential for a callable counted
`FOR`. Its nested typed `RETURN` and the following typed return both appear
in generated C, while the stale AST `PRINT` remains suppressed. The semantic
fixture uses the generated grammar's `END FOR` terminator and retains exact
source-position alignment. Focused and full offline workspace tests pass:
975 library tests, 19 CLI tests, 11 DOSBox tests, 35 examples, 62 JVM tests,
1 language test, 11 general record tests, and 40 record-method tests.

Iteration 352 extends the shared C semantic block emitter to handle typed
`RANDOMIZE` inside callable `IF` and loop bodies. The differential supplies
different seeds in the aligned AST and semantic sources and confirms the
typed seed reaches `srand` while the AST seed is absent. The focused
regression and full offline workspace suite pass: 976 library tests, 19 CLI
tests, 11 DOSBox tests, 35 examples, 62 JVM tests, 1 language test, 11
general record tests, and 40 record-method tests.

Iteration 353 extends the shared C semantic block emitter to typed `LOCATE`
and `COLOR`. A callable `IF` differential confirms semantic cursor
coordinates and palette operands replace the aligned AST values. The focused
regression and full offline workspace suite pass: 977 library tests, 19 CLI
tests, 11 DOSBox tests, 35 examples, 62 JVM tests, 1 language test, 11
general record tests, and 40 record-method tests.

Iteration 354 makes semantic callable `CONST` initializers emit as typed
assignments inside nested C blocks. The callable `IF` differential confirms
the semantic initializer and subsequent typed return use the resolved local
binding while the stale AST initializer is absent. Focused and full offline
workspace tests pass: 978 library tests, 19 CLI tests, 11 DOSBox tests, 35
examples, 62 JVM tests, 1 language test, 11 general record tests, and 40
record-method tests.

Iteration 355 adds typed semantic `SWAP` to the shared C nested-block
emitter. The callable `IF` differential verifies its generated temporary and
lvalues use semantic names instead of the aligned AST pair. The focused
regression and full offline workspace suite pass: 979 library tests, 19 CLI
tests, 11 DOSBox tests, 35 examples, 62 JVM tests, 1 language test, 11
general record tests, and 40 record-method tests.

Iteration 356 consumes nested semantic `DIM` nodes whose array storage was
already declared from typed IR. A callable `IF` differential confirms the
semantic bound and indexed read/write replace the aligned AST values. The
focused regression and full offline workspace suite pass: 980 library tests,
19 CLI tests, 11 DOSBox tests, 35 examples, 62 JVM tests, 1 language test,
11 general record tests, and 40 record-method tests. One initial full-suite
run encountered a transient OS `Text file busy` error launching the remline
example; its focused rerun and the subsequent full suite passed.

Iteration 357 consumes nested semantic `DATA` nodes in callable C blocks;
their values are already emitted from the typed IR's program-wide DATA
collection pass. The callable `IF` differential confirms the semantic DATA
value is present and the aligned AST value is absent. Focused and full
offline workspace tests pass: 981 library tests, 19 CLI tests, 11 DOSBox
tests, 35 examples, 62 JVM tests, 1 language test, 11 general record tests,
and 40 record-method tests.

Iteration 358 dispatches typed `READ` targets inside nested C callable blocks.
The callable `IF` differential verifies the semantic target and return value
replace the aligned AST target, while nested `DATA` comes from the semantic
program-wide pool. Focused and full offline workspace tests pass: 982 library
tests, 19 CLI tests, 11 DOSBox tests, 35 examples, 62 JVM tests, 1 language
test, 11 general record tests, and 40 record-method tests.

Iteration 359 extends the shared C nested-block emitter to semantic `CLS` and
`BEEP`. The callable `IF` differential keeps source positions aligned while
replacing AST `LOCATE`/`COLOR` with the two semantic screen-control
statements, confirming semantic C output and absence of stale AST operands.
Focused and full offline workspace tests pass: 982 library tests, 19 CLI
tests, 11 DOSBox tests, 35 examples, 62 JVM tests, 1 language test, 11
general record tests, and 40 record-method tests.

Iteration 360 consumes nested semantic `GLOBAL` declarations in C callable
blocks. The differential keeps the AST and semantic statement positions
aligned, then verifies callable storage, assignment, and `RETURN` use the
semantic global name rather than the AST name. The compatibility declaration
collector still emits an unused local for the stale AST global name; removing
that residual AST-derived storage is part of the remaining declaration-fact
migration. Focused and full offline workspace tests pass: 983 library tests,
19 CLI tests, 11 DOSBox tests, 35 examples, 62 JVM tests, 1 language test,
11 general record tests, and 40 record-method tests.

Iteration 361 removes that stale local declaration when semantic callable
global facts are authoritative. The C callable declaration pass recursively
collects compatibility-AST `GLOBAL` names and discards names absent from the
resolved semantic global set; the nested-global differential now verifies
that no stale local storage remains. Focused and full offline workspace tests
pass with the same suite totals as Iteration 360.

Iteration 362 extends Stage 67 driver-boundary coverage with a callable
global-name differential across BASIC, C, and JVM. The fixture supplies a
stale AST global and a distinct typed global with a different assignment
value; BASIC/C output must use the semantic name, and JVM bytecode must retain
the semantic value. The focused regression and full offline workspace tests
pass: 984 library tests, 19 CLI tests, 11 DOSBox tests, 35 examples, 62 JVM
tests, 1 language test, 11 general record tests, and 40 record-method tests.

Iteration 363 moves that differential's `GLOBAL` declaration inside the
callable `IF`, where it exposed a JVM block-emitter gap: slot allocation
already used the semantic global set, but the nested emitter declined the
declaration and fell back to stale AST statements. The JVM semantic block
emitter now consumes `GLOBAL` as a no-op after slot allocation. Focused and
full offline workspace tests pass with 984 library tests, 19 CLI tests, 11
DOSBox tests, 35 examples, 62 JVM tests, 1 language test, 11 general record
tests, and 40 record-method tests.

Iteration 364 strengthens the Stage 67 global-name case into a three-target
typed-source versus compatibility-source differential. BASIC and C output
remain identical for the semantic source. JVM typed output correctly stores
the callable global in a static field, while the AST-only compatibility path
exposed that the same global was being allocated as a local. The regression
now records that mismatch pending the follow-up declaration fix. Focused and
full offline workspace tests pass with 984 library tests, 19 CLI tests, 11
DOSBox tests, 35 examples, 62 JVM tests, 1 language test, 11 general record
tests, and 40 record-method tests.

Iteration 365 fixes the AST-only JVM callable-global declaration gap exposed
by that driver differential. When semantic IR is absent, JVM module storage
collection now promotes each callable `GLOBAL` into the static slot table,
using scalar declaration types collected from its callable body. The driver
differential confirms typed and compatibility JVM output both use static
global storage; label-layout differences remain expected. Focused and full
offline workspace tests pass with 984 library tests, 19 CLI tests, 11 DOSBox
tests, 35 examples, 62 JVM tests, 1 language test, 11 general record tests,
and 40 record-method tests.

Iteration 366 adds a Stage 67 driver-boundary array differential across BASIC,
C, and JVM. The AST fixture uses a smaller DIM bound and index/value than the
typed source; all semantic outputs use the typed array capacity, indexed
assignment, and value, and match compatibility output compiled from the typed
source. Focused and full offline workspace tests pass: 985 library tests, 19
CLI tests, 11 DOSBox tests, 35 examples, 62 JVM tests, 1 language test, 11
general record tests, and 40 record-method tests.

Iteration 367 adds a Stage 67 driver-boundary array element-type case. An
unsuffixed AST declaration is paired with a typed semantic `DIM ... AS
INTEGER`; generated BASIC retains the annotation, C allocates an integer
array, and JVM emits an integer array field with integer load/store opcodes.
The focused regression and full offline workspace suite pass: 986 library
tests, 19 CLI tests, 11 DOSBox tests, 35 examples, 62 JVM tests, 1 language
test, 11 general record tests, and 40 record-method tests. Workspace-wide
rustfmt checking still reports formatting differences in other dirty files;
no repository-wide formatting was applied.

Iteration 368 expands the Stage 67 driver DIM element-type case across
INTEGER, LONG, SINGLE, DOUBLE, and STRING. It verifies that BASIC retains each
annotation, C emits storage with the corresponding element representation,
and JVM declares the matching array descriptor. An initial top-level LONG
assignment probe appeared to fall through to the AST compatibility path; a
minimal aligned driver regression in Iteration 369 now confirms that the typed
assignment replaces the stale AST value. The focused matrix regression,
scoped rustfmt check, and full offline workspace suite pass: 986 library tests,
19 CLI tests, 11 DOSBox tests, 35 examples, 62 JVM tests, 1 language test, 11
general record tests, and 40 record-method tests.

Iteration 369 adds a Stage 67 three-backend driver differential for an
unsuffixed AST array paired with semantic `DIM ... AS LONG`. BASIC retains the
typed declaration, C emits the semantic array assignment and excludes the
stale AST value, and JVM emits the long-array descriptor and load/store
opcodes. This confirms top-level typed LONG array assignment and access across
the aligned path. The focused regression and scoped rustfmt check pass.

Iteration 370 extends the typed array driver matrix from declarations to
element assignment and access for INTEGER, LONG, SINGLE, DOUBLE, and STRING.
The unsuffixed AST fixture is paired with distinct typed assignments; C storage
and array references and JVM array descriptors plus type-specific load/store
opcodes are asserted alongside BASIC's typed DIM output. The focused regression
and scoped rustfmt check pass.

Iteration 371 adds a C64 driver differential for top-level semantic LONG array
assignment and access. The unsuffixed AST fixture carries a different value;
the C64 output uses the typed array element and excludes the AST assignment.
The focused C64 regression and scoped rustfmt check pass.

Iteration 372 adds a callable-local Stage 67 driver fixture for an unsuffixed
AST array paired with semantic `DIM ... AS LONG`. BASIC retains the callable
local type annotation, C emits LONG storage and semantic element assignment,
and JVM uses long-array storage and access opcodes. The focused regression and
scoped rustfmt check pass. The full offline workspace suite passes: 989 library
tests, 19 CLI tests, 11 DOSBox tests, 35 examples, 62 JVM tests, 1 language
test, 11 general record tests, and 40 record-method tests.

Iteration 373 expands the callable-local Stage 67 array type driver case across
INTEGER, LONG, SINGLE, DOUBLE, and STRING. It checks BASIC callable-local DIM
annotations and typed assignments, C local array storage and references, and
JVM type-specific array load/store opcodes. The focused regression and scoped
rustfmt check pass. The full offline workspace suite passes: 989 library tests,
19 CLI tests, 11 DOSBox tests, 35 examples, 62 JVM tests, 1 language test, 11
general record tests, and 40 record-method tests.

Iteration 374 compares typed BASIC output with the AST-only compatibility
output for annotated module arrays. Typed BASIC retains every element
annotation; the compatibility path differs. C and JVM compatibility output
also differs for element types whose storage is distinguishable from the
unsuffixed default, while the legacy C path rejects an untyped STRING array
assignment. JVM SINGLE and DOUBLE intentionally share the current `[D` storage
representation. The focused differential and scoped rustfmt check pass.

Iteration 375 adds the typed-versus-compatibility BASIC comparison at callable
scope for INTEGER, LONG, SINGLE, DOUBLE, and STRING arrays. This checks that
the callable-local DIM allocator retains semantic element annotations where
the AST-only path does not. The focused regression, full offline workspace
suite, scoped rustfmt check, and `git diff --check` pass: 989 library tests,
19 CLI tests, 11 DOSBox tests, 35 examples, 62 JVM tests, 1 language test, 11
general record tests, and 40 record-method tests.

Iteration 376 adds callable-scope C and JVM compatibility comparisons for the
same array type matrix. C output differs from the AST-only path for element
types whose storage differs from default SINGLE; the legacy C path rejects an
untyped STRING assignment. JVM output differs where array storage descriptors
distinguish the typed element; if the compatibility emitter rejects a case,
the test requires a nonempty diagnostic while checking that typed output has
the expected opcode. The focused regressions and full offline workspace suite
pass: 989 library tests, 19 CLI tests, 11 DOSBox tests, 35 examples, 62 JVM
tests, 1 language test, 11 general record tests, and 40 record-method tests.

Iteration 377 verifies the module-scope array type matrix through
`compile_file`, comparing driver output with typed backend output for BASIC,
C, and JVM across INTEGER, LONG, SINGLE, DOUBLE, and STRING. This exercises
the ordinary file parser, generated semantic frontend, resolver, conflict
check, target dispatch, and each codegen backend. The focused regression and
scoped rustfmt check pass.

Iteration 378 reruns the full offline workspace suite after the three-target
driver comparison: 989 library tests, 19 CLI tests, 11 DOSBox tests, 35
examples, 62 JVM tests, 1 language test, 11 general record tests, and 40
record-method tests pass. `git diff --check` passes.

Iteration 379 adds the `compile_file` target-dispatch comparison for the
callable-local array type matrix. BASIC, C, and JVM driver outputs match direct
typed backend output for INTEGER, LONG, SINGLE, DOUBLE, and STRING callable
arrays. The focused regression, scoped rustfmt check, `git diff --check`, and
full offline workspace suite pass: 989 library tests, 19 CLI tests, 11 DOSBox
tests, 35 examples, 62 JVM tests, 1 language test, 11 general record tests,
and 40 record-method tests.

Iteration 380 verifies that `compile_file`'s C64 target dispatch returns the
same typed LONG array assignment emitted by direct C64 codegen. The test uses
an unsuffixed AST declaration with a differing stale value and a typed source
file beginning with a program header. The focused C64 driver regression and
scoped rustfmt check and `git diff --check` pass. The full offline workspace
suite is green: 989 library tests, 19 CLI tests, 11 DOSBox tests, 35 examples,
62 JVM tests, 1 language test, 11 general record tests, and 40 record-method
tests.

Iteration 381 adds FBC to the `compile_file` driver target matrix for module
and callable-local typed arrays. The FBC capability traversal accepts all five
element annotations and emits the same BASIC representation as the direct
typed BASIC backend. Both focused driver regressions and scoped rustfmt
checking pass. The full offline workspace suite passes: 989 library tests, 19
CLI tests, 11 DOSBox tests, 35 examples, 62 JVM tests, 1 language test, 11
general record tests, and 40 record-method tests.

Iteration 382 adds a callable-local C64 driver differential for semantic LONG
arrays. Direct C64 codegen and `compile_file` both use the typed local array
storage and assignment, excluding the stale AST value. The focused regression
and scoped rustfmt check pass.

Iteration 383 adds a scalar `DIM ... AS INTEGER` driver check across BASIC,
FBC, C, C64, and JVM. Typed storage and assignment output from direct backend
calls matches `compile_file` target dispatch for each target. The focused
driver regression, scoped rustfmt check, `git diff --check`, and full offline
workspace suite pass: 991 library tests, 19 CLI tests, 11 DOSBox tests, 35
examples, 62 JVM tests, 1 language test, 11 general record tests, and 40
record-method tests.

Iteration 384 expands the scalar driver test to INTEGER, LONG, SINGLE, DOUBLE,
and STRING `DIM` annotations. BASIC/FBC, C, and JVM output is validated for
each type; C64 is checked for INTEGER and LONG. Direct typed backend output
matches `compile_file` output on every covered target. The focused regression
and scoped rustfmt check pass. The full offline workspace suite passes: 991
library tests, 19 CLI tests, 11 DOSBox tests, 35 examples, 62 JVM tests, 1
language test, 11 general record tests, and 40 record-method tests.

Iteration 385 extends the scalar `DIM` type matrix to callable-local storage.
Typed INTEGER, LONG, SINGLE, DOUBLE, and STRING assignments and local storage
are checked in BASIC/FBC, C, and JVM; C64 is checked for INTEGER and LONG.
For every covered target, `compile_file` output matches direct typed backend
output. The focused regression, scoped rustfmt check, `git diff --check`, and
full offline workspace suite pass: 992 library tests, 19 CLI tests, 11 DOSBox
tests, 35 examples, 62 JVM tests, 1 language test, 11 general record tests,
and 40 record-method tests.

Iteration 386 removes the unused AST-derived `TypedArrayRef` side channel.
The parser had collected array/index and `SIZEOF` reference syntax into
`Program`, the driver and resolver only copied it, and JVM context stored it
without reading it. The dead collector and fields are removed across the AST,
parser, record lowering, driver, resolver, and JVM context; semantic array
declarations and typed expressions remain the active codegen inputs. `cargo
check --locked`, the full offline workspace suite, `git diff --check`, and a
post-change Graphify update pass: 992 library tests, 19 CLI tests, 11 DOSBox
tests, 35 examples, 62 JVM tests, 1 language test, 11 general record tests,
and 40 record-method tests.

Iteration 387 makes the JVM callable signature table consume resolved semantic
parameter metadata as a complete source of parameter count, binding names,
scalar slots, array ranks and element types, `byref` positions, receiver type,
and result type whenever the callable exists in typed IR. AST callable
parameters remain a compatibility fallback only when no semantic callable
matches. All 105 JVM codegen unit tests and the full locked workspace test
suite pass, and `git diff --check` passes. `rustfmt --check src/codegen_jvm.rs`
reports only two pre-existing formatting differences in semantic array
declaration tests; the modified callable-signature block is formatted.

Iteration 388 removes AST parameter-count matching from JVM callable
association. The selected semantic callable now supplies callable arity;
`JvmContext::for_function` allocates scalar and array parameter slots from
that arity, and the by-value array prologue iterates typed array-parameter
facts. A differential regression gives the AST one parameter and typed IR
two, then verifies the emitted descriptor, local slots, body, and call use the
typed two-parameter signature. The former test that expected a mismatch to
decline semantic dispatch now verifies dispatch by callable identity. All 106
JVM codegen unit tests and the full locked workspace suite pass.

Iteration 389 removes C backend callable selection's AST parameter-count
gate and builds `FnSig` parameter entries from the matched typed callable's
names, types, ranks, passing modes, and defaults. C local-variable collection
now excludes stale AST parameter bindings when semantic callable metadata is
active, and the C64 by-value-array capability check follows the typed
signature. A differential regression uses one AST parameter and two typed
parameters and verifies the emitted C prototype, parameter bindings, call,
and absence of the stale AST name. All 184 C backend tests and the full locked
workspace suite pass.

Iteration 390 removes the BASIC backend callable-arity gate and replaces
`FunctionInfo`'s AST `Param` storage with backend parameter facts populated
from typed callable parameters: binding names, ranks, and passing modes now
drive BASIC call setup and local array storage. Callable listing comments also
use those resolved bindings. Top-level typed assignments use expression
preludes for function calls, preserving direct-call output for pure arguments
and direct results. The AST path remains available for unmatched callables
and compatibility defaults. A differential regression gives AST one
parameter and typed IR two, then verifies the typed binding names and call
setup. All 124 BASIC backend tests and the full locked workspace suite pass:
995 library tests, 19 CLI tests, 11 DOSBox tests, 35 examples, 62 JVM tests,
1 language test, 11 general record tests, and 40 record-method tests.

Iteration 394 routes callable `RETURN` expressions through the typed expression
prelude emitter. This lets nested function calls execute before their result is
used in the return expression, instead of rendering call syntax as a constant
expression. The driver regression for a required-library callable checks the
typed call-result capture while retaining exact parity checks for C and JVM.
The focused regression and full locked workspace suite pass: 999 library
tests, 19 CLI tests, 11 DOSBox tests, 35 examples, 62 JVM tests, 1 language
test, 11 general record tests, and 40 record-method tests. Graphify was
refreshed.

Iteration 395 routes callable `LOCATE` row and column expressions through the
typed expression-prelude emitter. Callable function-call operands therefore
execute before `LOCATE`, in source evaluation order; a differential regression
checks both typed calls and rejects the stale AST coordinates. The focused
regression passes. The full locked workspace run passes 1,000 library tests
but has one environment failure: `freebasic_runs_remline_when_available`
cannot execute its compiled binary (`PermissionDenied`); all other reported
integration suites pass. Graphify was refreshed.

Iteration 396 routes callable `COLOR` foreground and background operands through
the typed expression-prelude emitter, preserving left-to-right evaluation and
ensuring function-call operands execute before the statement. A differential
regression rejects stale AST operands and checks the typed expression
temporaries. The focused regression and all 1,001 library tests pass.
Graphify was refreshed.

Iteration 397 routes callable `WIDTH` channel and width expressions through the
typed expression-prelude emitter, preserving channel-before-width evaluation.
A differential regression confirms the function-call operands are emitted
before `WIDTH` and stale AST operands are absent. The focused regression and
all 1,002 library tests pass; formatting and `git diff --check` pass. Graphify
was refreshed.

Iteration 398 routes callable `POKE` address/value and `OUT` port/value operands
through the typed expression-prelude emitter, preserving left-to-right
evaluation and executing function calls before the target statement. A
differential regression checks typed temporaries and rejects stale AST
operands. The focused regression and all 1,003 library tests pass. Graphify was
refreshed.

Iteration 399 extends typed expression-prelude emission to module-scope
`LOCATE` statements. A driver-shaped semantic/AST differential uses function
calls for the typed row and column and verifies that the generated BASIC
evaluates both calls and emits their temporaries before `LOCATE`. The focused
regression and all 1,004 library tests pass; `git diff --check` passes.
Graphify was refreshed.

Iteration 400 adds Stage 67 driver coverage for module-scope typed `LOCATE`
operands. `compile_source` now has a regression proving generated frontend
function calls reach BASIC statement emission and their evaluated results feed
the emitted `LOCATE`. The focused regression and all 1,005 library tests
pass; formatting and `git diff --check` pass. Graphify was refreshed.

Iteration 401 extends typed expression-prelude emission to module-scope
`COLOR` foreground and background operands. A semantic/AST differential uses
function calls for both typed operands and verifies that evaluated results feed
the generated statement. The focused regression and all 1,006 library tests
pass; `git diff --check` passes. Graphify was refreshed.

Iteration 402 reruns the full locked workspace suite after the transient
FreeBASIC executable permission failure recorded in Iteration 395. The rerun
passes completely: 1,006 library tests, 19 CLI tests, 11 DOSBox tests, 35
examples, 62 JVM tests, 1 language test, 11 general record tests, and 40
record-method tests.

Iteration 403 extends typed expression-prelude emission to module-scope
`WIDTH` channel and width expressions. The semantic/AST differential confirms
that function calls are evaluated in channel-before-width order and their
results feed the emitted statement. The focused regression and all 1,007
library tests pass; `git diff --check` passes. Graphify was refreshed.

Iteration 404 extends typed expression-prelude emission to module-scope `POKE`
and `OUT` operands. The differential regression verifies call-result
temporaries for the address/port and value expressions and rejects stale AST
operands. The focused regression and all 1,008 library tests pass;
`git diff --check` passes. Graphify was refreshed.

Iteration 405 expands Stage 67 `compile_source` coverage to typed module-scope
`LOCATE`, `COLOR`, `WIDTH`, `POKE`, and `OUT` operands. The driver-level matrix
confirms function-call arguments reach generated BASIC expression preludes and
the emitted statements. The focused regression and all 1,008 library tests
pass; formatting and `git diff --check` pass. Graphify was refreshed.

Iteration 406 adds `compile_file` parity coverage for the Stage 64 module-scope
expression-prelude cases. A typed BASIC source file exercises `LOCATE`,
`COLOR`, `WIDTH`, `POKE`, and `OUT`; BASIC and FBC file compilation match the
typed `compile_source` output. The focused regression and all 1,009 library
tests pass; formatting and `git diff --check` pass. Graphify was refreshed.

Iteration 407 extends the same `compile_file` test across C and JVM using a
shared supported scalar function-call assignment. Their file-driver output
matches direct typed codegen; the BASIC-only peripheral I/O cases remain
validated through BASIC/FBC dispatch. The focused regression and all 1,009
library tests pass; formatting and `git diff --check` pass. Graphify was
refreshed.

Iteration 408 routes callable and module-scope numeric `ERROR` and valued
`THROW` operands through typed expression preludes. Function-call operands now
execute before the generated error statement; regressions verify all four
scope/statement combinations and reject AST operand values. The focused
regression and all 1,010 library tests pass; formatting and `git diff --check`
pass. Graphify was refreshed.

Iteration 409 routes callable and module-scope seeded `RANDOMIZE` expressions
through typed expression preludes. A regression verifies that typed function
calls supply both seeds and stale AST literals are absent. The focused
regression and all 1,011 library tests pass; formatting and `git diff --check`
pass. Graphify was refreshed.

Iteration 410 confirms the full locked workspace suite passes when test
harnesses run serially. The two `remline` failures in Iteration 409 were a
parallel-test collision: both tests write `tmp/remline`; each passes in
isolation, and the full serialized suite passes 1,011 library tests, 19 CLI
tests, 11 DOSBox tests, 35 examples, 62 JVM tests, 1 language test, 11 general
record tests, and 40 record-method tests.

Iteration 411 serializes the GCC and FreeBASIC `remline` runtime tests, which
write the same `tmp/remline` executable and sample output file. This removes
the parallel-test collision observed in Iteration 409. The default parallel
full locked workspace suite now passes: 1,011 library tests, 19 CLI tests, 11
DOSBox tests, 35 examples, 62 JVM tests, 1 language test, 11 general record
tests, and 40 record-method tests. Formatting and `git diff --check` pass.

Iteration 412 routes callable and module-scope `ON ... GOTO/GOSUB` selectors
through typed expression preludes. Function-call selectors are evaluated
before branch dispatch; a regression covers both scopes and confirms stale AST
selectors are absent. The focused regression and all 1,012 library tests pass;
formatting and `git diff --check` pass. Graphify was refreshed.

Iteration 413 routes `LSET`/`RSET` right-hand expressions containing declared
callable calls through typed expression preludes in module and callable scopes.
Pure intrinsic expressions retain direct typed rendering, preserving the
existing `MKI%` output and string suffix. Differential regressions cover user
string calls; the random-access-file regression confirms intrinsic behavior.
All 1,013 library tests pass; formatting and `git diff --check` pass. Graphify
was refreshed.

Iteration 414 runs the full locked workspace suite after the `LSET`/`RSET`
intrinsic compatibility adjustment. The parallel suite passes: 1,013 library
tests, 19 CLI tests, 11 DOSBox tests, 35 examples, 62 JVM tests, 1 language
test, 11 general record tests, and 40 record-method tests.

Iteration 415 routes callable and module-scope `SEEK` channel and position
expressions through typed expression preludes. The regression confirms both
function-call operands execute in source order and stale AST channel/position
values are absent. The focused regression and all 1,014 library tests pass;
formatting and `git diff --check` pass. Graphify was refreshed.

Iteration 416 runs the full locked workspace suite after typed `SEEK` operand
emission. All suites pass: 1,014 library tests, 19 CLI tests, 11 DOSBox tests,
35 examples, 62 JVM tests, 1 language test, 11 general record tests, and 40
record-method tests.

Iteration 417 routes callable and module-scope `GET`/`PUT` channel, position,
and record expressions through typed expression preludes. The regression
checks all three operands execute in order for both statements in both scopes,
and rejects stale AST positions. The focused regression and all 1,015 library
tests pass; formatting and `git diff --check` pass. Graphify was refreshed.

Iteration 418 runs the full locked workspace suite after typed `GET`/`PUT`
position emission. All suites pass: 1,015 library tests, 19 CLI tests, 11
DOSBox tests, 35 examples, 62 JVM tests, 1 language test, 11 general record
tests, and 40 record-method tests.

Iteration 419 makes the BASIC AST compatibility emitter evaluate a `WIDTH`
channel before its column expression, matching source operand order and the
semantic emitter. A regression confirms the channel call precedes the
columns call and the `WIDTH` statement in generated BASIC. The focused test
and full locked workspace suite pass, including 1,016 library tests;
`git diff --check` passes. Repository-wide `cargo fmt --check` remains
blocked by formatting differences in pre-existing dirty files; the touched
BASIC backend passes `rustfmt --check`.

Iteration 420 extends Stage 67 `compile_file` parity coverage to record field
types across BASIC, C, and JVM. The fixture declares an `int16` and a fixed
string record field, assigns both, and confirms file-driver output matches
direct codegen for each backend. The focused regression and full locked
workspace suite pass: 1,017 library tests, 19 CLI tests, 11 DOSBox tests, 35
examples, 62 JVM tests, 1 language test, 11 general record tests, and 40
record-method tests. `rustfmt --check src/driver.rs` and `git diff --check`
pass.

Iteration 421 extends Stage 67 `compile_file` parity coverage to
`LoweredRecordFile` layout metadata. It attaches the output of `lower` to the
semantic module, then compares driver output with direct BASIC, C, and JVM
codegen for a random-access record write. The focused regression and full
locked workspace suite pass: 1,018 library tests, 19 CLI tests, 11 DOSBox
tests, 35 examples, 62 JVM tests, 1 language test, 11 general record tests,
and 40 record-method tests. `rustfmt --check src/driver.rs` and
`git diff --check` pass.

Iteration 422 fixes a Stage 66 JVM local-slot gap for scalar `BYREF` actuals
nested inside typed expressions. The declaration collector now recursively
finds typed callable invocations and registers their scalar output bindings
across assignment, print, control-flow, return, and I/O expression sites. A
differential regression replaces an AST literal `PRINT` with a typed nested
`BYREF` call and verifies the invocation and wrapper writeback are emitted.
The focused regression and full locked workspace suite pass: 1,019 library
tests, 19 CLI tests, 11 DOSBox tests, 35 examples, 62 JVM tests, 1 language
test, 11 general record tests, and 40 record-method tests. `git diff --check`
passes. `rustfmt --check src/codegen_jvm.rs` reports only two pre-existing
formatting differences later in that file.

Iteration 423 fixes the matching Stage 65 C local-storage gap for scalar
`BYREF` actuals nested in typed expressions. C callable declaration setup
walks semantic expressions and uses the resolved callable ABI to register
each scalar output local, including calls in `PRINT` and nested control flow.
A differential regression substitutes a typed nested `BYREF` call for an
AST literal and verifies both the local declaration and address-passing call.
The focused regression and full locked workspace suite pass: 1,020 library
tests, 19 CLI tests, 11 DOSBox tests, 35 examples, 62 JVM tests, 1 language
test, 11 general record tests, and 40 record-method tests. `rustfmt --check
src/codegen_c.rs` and `git diff --check` pass.

Iteration 424 extends Stage 65 and Stage 66 scalar `BYREF` local discovery to
semantic method calls. C resolves the receiver's typed scalar suffix and
method ABI before registering explicit scalar output actuals; JVM matches the
typed receiver against the callable signature before reserving the local slot.
The existing C nested-call regression, JVM typed scalar method `BYREF`
regression, and full locked workspace suite pass: 1,020 library tests, 19 CLI
tests, 11 DOSBox tests, 35 examples, 62 JVM tests, 1 language test, 11
general record tests, and 40 record-method tests. `cargo check --locked -q`
and `git diff --check` pass. Graphify was refreshed.

Iteration 425 adds a focused Stage 65 regression for C scalar `BYREF` method
local discovery. It constructs the typed method ABI and confirms the collector
uses the receiver's resolved integer type and registers the explicit output
actual as an integer local. The focused regression and full locked workspace
suite pass: 1,021 library tests, 19 CLI tests, 11 DOSBox tests, 35 examples,
62 JVM tests, 1 language test, 11 general record tests, and 40 record-method
tests. `rustfmt --check src/codegen_c.rs` and `git diff --check` pass. A
separate C expression-emission gap remains to investigate: a typed scalar
method call inside callable `PRINT` did not replace the AST literal in an
exploratory differential.

Iteration 426 extends C Stage 65 expression transpilation so module-scope
semantic `PRINT` can render typed numeric scalar method calls. The renderer
selects the method by resolved receiver suffix, validates the result and
parameter ABI, transpiles the receiver and explicit arguments in order, and
passes scalar `BYREF` actuals by address. A C regression verifies the typed
call replaces the AST literal; a Stage 67 `compile_file` differential compares
BASIC, C, and JVM output for the same method call. The focused method `BYREF`
regressions pass, and the full locked workspace suite passes: 1,023 library
tests, 19 CLI tests, 11 DOSBox tests, 35 examples, 62 JVM tests, 1 language
test, 11 general record tests, and 40 record-method tests. Formatting checks
for `codegen_c.rs` and `driver.rs`, `git diff --check`, and Graphify refresh
pass. Callable-scope C `PRINT` still uses the ordinary function map and needs
a follow-up to pass the scalar method ABI table into its semantic renderer.

Iteration 427 adds callable-scope coverage for the scalar method `BYREF`
`PRINT` path introduced in Iteration 426. A differential typed callable body
replaces an AST literal with a method invocation, and the regression verifies
the receiver, address-passed actual, and semantic output replace the stale AST
statement. The focused test and full locked workspace suite pass: 1,024
library tests, 19 CLI tests, 11 DOSBox tests, 35 examples, 62 JVM tests, 1
language test, 11 general record tests, and 40 record-method tests.
`rustfmt --check src/codegen_c.rs` and `git diff --check` pass. This closes
the callable-scope follow-up recorded in Iteration 426.

Iteration 428 generalizes the C print expression path so a typed scalar method
call can participate in a numeric unary or binary expression. The method-aware
renderer recurses through parentheses and numeric operators while retaining
existing C operator transpilation; a callable regression covers a `BYREF`
method result added to a literal. The full locked workspace suite passes:
1,024 library tests, 19 CLI tests, 11 DOSBox tests, 35 examples, 62 JVM tests,
1 language test, 11 general record tests, and 40 record-method tests.
`rustfmt --check src/codegen_c.rs`, `rustfmt --check src/driver.rs`,
`git diff --check`, and Graphify refresh pass.

Iteration 429 extends the C method-aware numeric expression path to semantic
assignment right-hand sides in module and callable scopes. Scalar method
results now transpile through the resolved receiver and parameter ABI before
the assignment operator is emitted; existing function-only semantic paths
retain their prior renderer. A differential regression verifies a typed method
assignment replaces the AST literal. The focused regression and full locked
workspace suite pass: 1,025 library tests, 19 CLI tests, 11 DOSBox tests, 35
examples, 62 JVM tests, 1 language test, 11 general record tests, and 40
record-method tests. Rust formatting checks, `git diff --check`, and Graphify
refresh pass.

Iteration 430 extends C's method-aware numeric expression path to typed
semantic `IF` conditions. The condition renderer now resolves scalar methods
with the active receiver and parameter ABI before emitting branch control
flow; a regression proves a typed method condition replaces an AST false
condition and stale branch body. The focused regression and full locked
workspace suite pass: 1,026 library tests, 19 CLI tests, 11 DOSBox tests, 35
examples, 62 JVM tests, 1 language test, 11 general record tests, and 40
record-method tests. C and driver formatting checks, `git diff --check`, and
Graphify refresh pass.

Iteration 431 adds method-aware rendering to typed scalar `WHILE` conditions
in C. The backend resolves the method receiver and parameter ABI before
emitting loop control flow; the regression verifies that the typed method
condition and semantic loop body replace stale AST content. The full locked
workspace suite passes: 1,027 library tests, 19 CLI tests, 11 DOSBox tests, 35
examples, 62 JVM tests, 1 language test, 11 general record tests, and 40
record-method tests. `rustfmt --check src/codegen_c.rs`, `git diff --check`,
and Graphify refresh pass.

Iteration 432 adds Stage 67 `compile_file` parity coverage for scalar method
calls in assignment right-hand sides and `IF` conditions. The same typed source
is transpiled directly and through the driver for BASIC, C, and JVM, and all
three outputs match. The focused driver regression and full locked workspace
suite pass: 1,028 library tests, 19 CLI tests, 11 DOSBox tests, 35 examples,
62 JVM tests, 1 language test, 11 general record tests, and 40 record-method
tests. `rustfmt --check` for C and driver, and `git diff --check` pass.

Iteration 433 extends the Stage 67 method-expression `compile_file`
differential to include a typed scalar method call in a `WHILE` condition. The
file driver now matches direct BASIC, C, and JVM output across method
assignment, `IF`, and `WHILE` contexts. The focused regression and full locked
workspace suite pass: 1,028 library tests, 19 CLI tests, 11 DOSBox tests, 35
examples, 62 JVM tests, 1 language test, 11 general record tests, and 40
record-method tests. C and driver formatting checks and `git diff --check`
pass.

Iteration 434 extends C method-aware numeric expression emission to semantic
callable `RETURN` values. C validates the typed scalar method result against the
callable result ABI, then emits the resolved receiver and explicit arguments.
A regression confirms a semantic method return replaces the AST literal. The
focused test and full locked workspace suite pass: 1,029 library tests, 19
CLI tests, 11 DOSBox tests, 35 examples, 62 JVM tests, 1 language test, 11
general record tests, and 40 record-method tests. C and driver formatting
checks, `git diff --check`, and Graphify refresh pass.

Iteration 435 extends C method-aware numeric expression emission to semantic
`SELECT CASE` selectors. The selector is validated and evaluated once before
case dispatch; a regression verifies a typed scalar method selector replaces
the stale AST selector. An attempted BASIC and driver parity extension exposed
that BASIC typed `Member` expressions are not yet handled by the semantic
expression-with-prelude renderer. The parity extension was deferred rather
than routing that typed method through AST fallback. The locked workspace suite
passes: 1,030 library tests, 19 CLI tests, 11 DOSBox tests, 35 examples, 62 JVM
tests, 1 language test, 11 general record tests, and 40 record-method tests.
Formatting checks, `git diff --check`, and Graphify refresh pass.

Iteration 436 extends BASIC's typed semantic expression-with-prelude renderer
to scalar method `Member` expressions. It validates the receiver type, implicit
receiver ABI slot, explicit parameter types and passing modes, evaluates
arguments in source order, performs `BYREF` copyback, and snapshots the method
result. The driver differential now includes a method call as a `SELECT CASE`
selector and matches direct BASIC, C, and JVM output. BASIC method-chain and
required-library assertions now verify the result snapshots. The full locked
workspace suite passes: 1,030 library tests, 19 CLI tests, 11 DOSBox tests, 35
examples, 62 JVM tests, 1 language test, 11 general record tests, and 40
record-method tests.

Iteration 437 extends the same Stage 67 driver differential to nested typed
scalar method calls inside a binary `PRINT` expression. Two calls now exercise
source-order evaluation and result snapshots through direct and `compile_file`
transpilation for BASIC, C, and JVM. The focused driver test and full locked
workspace suite pass: 1,030 library tests, 19 CLI tests, 11 DOSBox tests, 35
examples, 62 JVM tests, 1 language test, 11 general record tests, and 40
record-method tests. Driver formatting and `git diff --check` pass.

Iteration 438 extends the Stage 67 driver differential with a typed scalar
method call in a function `RETURN` expression, then invokes that function from
the module body. Direct and `compile_file` outputs match for BASIC, C, and JVM,
covering the method receiver and result ABI inside callable scope. The focused
driver regression and full locked workspace suite pass: 1,030 library tests,
19 CLI tests, 11 DOSBox tests, 35 examples, 62 JVM tests, 1 language test, 11
general record tests, and 40 record-method tests.

Iteration 439 updates BASIC's callable-expression classifier to recognize
typed scalar method `Member` expressions, allowing `LSET` and `RSET` to use
the semantic expression-with-prelude path. A focused differential substitutes
typed method operands for stale AST string literals at module scope; both
`LSET` and `RSET` dispatch through the method ABI. The full locked workspace
suite passes: 1,031 library tests, 19 CLI tests, 11 DOSBox tests, 35 examples,
62 JVM tests, 1 language test, 11 general record tests, and 40 record-method
tests.

Iteration 440 extends C's method-aware numeric expression path to semantic
`FOR` start, limit, and explicit step expressions at module and callable
dispatch sites. The regression replaces stale literal bounds with typed
scalar method calls, and the Stage 67 differential checks driver parity for
method calls in both loop bounds across BASIC, C, and JVM. Focused tests and
the full locked workspace suite pass: 1,032 library tests, 19 CLI tests, 11
DOSBox tests, 35 examples, 62 JVM tests, 1 language test, 11 general record
tests, and 40 record-method tests.

Iteration 441 adds dynamic typed JVM `FOR STEP` support. The emitter evaluates
the limit and step once into reserved local slots, selects ascending or
descending bound checks from the runtime step sign, and applies the captured
step at each continuation. A JVM runtime regression verifies positive and
negative method-produced steps, `BYREF` side effects, and single evaluation.
The three-target driver differential also covers a method call in the step
expression. The focused runtime test and full locked workspace suite pass:
1,032 library tests, 19 CLI tests, 11 DOSBox tests, 35 examples, 63 JVM tests,
1 language test, 11 general record tests, and 40 record-method tests.

Iteration 442 extends BASIC's typed semantic `FOR` expression-with-prelude
path to method calls in the start, limit, and explicit STEP expressions at
module and callable scope. C stages a start expression containing a semantic
method call before the limit and STEP expressions, preserving source-order
evaluation while retaining the existing generated form for simple starts.
Regression coverage verifies stale-AST replacement, module and callable
emission, and method-call evaluation order; the Stage 67 driver differential
covers method calls in all three loop expressions across BASIC, C, and JVM.
The full locked workspace suite passes: 1,033 library tests, 19 CLI tests,
11 DOSBox tests, 35 examples, 63 JVM tests, 1 language test, 11 general
record tests, and 40 record-method tests. Rust formatting and
`git diff --check` pass.

Iteration 443 extends BASIC semantic control-flow expression handling to
typed scalar method calls in module and callable `IF` and `WHILE` conditions.
Method-call preludes execute at the condition point, including on each
`WHILE` iteration; stale-AST regressions also retain the module and callable
method-driven `FOR` cases from Iteration 442. The full locked workspace suite
passes: 1,033 library tests, 19 CLI tests, 11 DOSBox tests, 35 examples, 63
JVM tests, 1 language test, 11 general record tests, and 40 record-method
tests. Rust formatting, Graphify refresh, and `git diff --check` pass.

Iteration 444 extends the same typed scalar method `Member` handling to BASIC
preconditioned and postconditioned `DO` loops. Module and callable regressions
replace AST conditions with typed method calls, and the Stage 67 differential
now includes `DO WHILE` and `LOOP UNTIL` method conditions across BASIC, C,
and JVM. Condition preludes execute at the precondition or postcondition site
on each loop pass. The full locked workspace suite passes: 1,033 library
tests, 19 CLI tests, 11 DOSBox tests, 35 examples, 63 JVM tests, 1 language
test, 11 general record tests, and 40 record-method tests. Rust formatting
and `git diff --check` pass.

Iteration 445 broadens C `FOR` start staging from scalar method calls to all
semantic function and method calls, including nested call operands. This keeps
start evaluation ahead of limit and STEP evaluation whenever the start may
have side effects, while simple names and literals retain their existing
generated form. The existing method-bound order regression and full locked
workspace suite pass: 1,033 library tests, 19 CLI tests, 11 DOSBox tests, 35
examples, 63 JVM tests, 1 language test, 11 general record tests, and 40
record-method tests. Rust formatting, Graphify refresh, and `git diff --check`
pass.


Iteration 446 adds JVM runtime conformance for typed scalar method calls in
preconditioned and postconditioned loop conditions. The method mutates a
`BYREF` counter, and the expected output confirms re-evaluation on each pass
and the postcondition's position after the body. The full locked workspace
suite passes: 1,033 library tests, 19 CLI tests, 11 DOSBox tests, 35 examples,
64 JVM tests, 1 language test, 11 general record tests, and 40 record-method
tests. Rust formatting and `git diff --check` pass.

Iteration 447 fixes C `FOR` evaluation order when a side-effecting limit or
STEP expression can mutate a variable used by the start expression. C now
captures the start before evaluating callable bounds in this case, while
simple loops retain their existing generated form. A regression verifies a
`BYREF` method bound cannot change the already captured start. The full locked
workspace suite passes: 1,034 library tests, 19 CLI tests, 11 DOSBox tests,
35 examples, 64 JVM tests, 1 language test, 11 general record tests, and 40
record-method tests. Rust formatting, Graphify refresh, and
`git diff --check` pass.

Iteration 448 fixes BASIC semantic `FOR` evaluation order when a callable limit or
STEP can mutate a variable used by the start expression. Module and callable
codegen now snapshots that start before emitting bound-call preludes, while
simple loops retain their existing form. A stale-AST regression verifies the
module snapshot precedes a `BYREF` method bound and that callable FOR headers
use the captured start. The full locked workspace
suite passes: 1,035 library tests, 19 CLI tests, 11 DOSBox tests, 35 examples,
64 JVM tests, 1 language test, 11 general record tests, and 40 record-method
tests. Rust formatting, Graphify refresh, and `git diff --check` pass.

Iteration 449 extends the Stage 67 `compile_file` differential with a
`BYREF` method call in a `FOR` limit that mutates the loop start variable.
Direct codegen and the file driver produce matching BASIC, C, and JVM output,
covering the ordering fix from Iterations 447 and 448 at the driver boundary.
The full locked workspace suite passes: 1,035 library tests, 19 CLI tests, 11
DOSBox tests, 35 examples, 64 JVM tests, 1 language test, 11 general record
tests, and 40 record-method tests. Rust formatting and `git diff --check`
pass.

Iteration 450 fixes JVM semantic `FOR` limit evaluation for callable bounds.
A limit containing a semantic function or method call is now evaluated once
after the start and stored before loop dispatch; dynamic STEP loops retain
their runtime-sign path. A JVM execution regression uses a `BYREF` method bound
that mutates its receiver variable and verifies the loop terminates with the
captured initial value. The full locked workspace suite passes: 1,035 library
tests, 19 CLI tests, 11 DOSBox tests, 35 examples, 65 JVM tests, 1 language
test, 11 general record tests, and 40 record-method tests. Rust formatting,
Graphify refresh, and `git diff --check` pass.

Iteration 451 fixes BASIC semantic `FOR` evaluation order when a callable STEP
can mutate a variable used by the limit. Module and callable codegen now
snapshots the limit after its own prelude and before STEP call preludes; simple
loops retain their existing form. A regression verifies start and limit
snapshots precede their mutating method calls. The full locked workspace suite
passes: 1,035 library tests, 19 CLI tests, 11 DOSBox tests, 35 examples, 65
JVM tests, 1 language test, 11 general record tests, and 40 record-method
tests. Rust formatting, Graphify refresh, and `git diff --check` pass.

Iteration 452 extends the Stage 67 `compile_file` differential to a
`BYREF` method STEP that mutates the variable used by the loop limit. Direct
codegen and the file driver produce matching BASIC, C, and JVM output. The
full locked workspace suite passes with the same 1,035 library tests, 19 CLI
tests, 11 DOSBox tests, 35 examples, 65 JVM tests, 1 language test, 11
general record tests, and 40 record-method tests. Rust formatting and
`git diff --check` pass.

Iteration 453 adds JVM runtime conformance for a `BYREF` method in a dynamic
STEP expression that mutates the variable used by the loop limit. The output
confirms the limit is captured before STEP evaluation and that the method
executes once. The full locked workspace suite passes: 1,035 library tests,
19 CLI tests, 11 DOSBox tests, 35 examples, 66 JVM tests, 1 language test, 11
general record tests, and 40 record-method tests. Rust formatting and
`git diff --check` pass.

Iteration 454 fixes JVM semantic `FOR` start aliasing when a callable bound
mutates the loop variable itself. If a limit or STEP contains a call, the JVM
backend now captures the start in a reserved typed local, evaluates bounds in
source order, then initializes the loop variable from that snapshot. Runtime
conformance verifies a `BYREF` bound cannot overwrite the captured start. The
full locked workspace suite passes: 1,035 library tests, 19 CLI tests, 11
DOSBox tests, 35 examples, 66 JVM tests, 1 language test, 11 general record
tests, and 40 record-method tests. Rust formatting, Graphify refresh, and
`git diff --check` pass.

Iteration 455 extends the Stage 67 three-target driver differential with a
loop whose start is the loop variable and whose method bound mutates it by
`BYREF`. Direct codegen and `compile_file` output match across BASIC, C, and
JVM. The full locked workspace suite passes with the same 1,035 library
tests, 19 CLI tests, 11 DOSBox tests, 35 examples, 66 JVM tests, 1 language
test, 11 general record tests, and 40 record-method tests. Rust formatting
and `git diff --check` pass.

Iteration 456 aligns JVM semantic and AST `FOR` emission with BASCAL
loop-bound semantics: start, limit, and dynamic STEP expressions are captured
at loop entry, with the loop variable initialized after bound evaluation. This
fixes semantic JVM loops whose body mutates the limit variable. A JVM runtime
regression confirms the loop reaches its original bound after the body assigns
a new limit value; compatibility output remains equivalent for existing
semantic fixtures. The full locked workspace suite passes: 1,035 library
tests, 19 CLI tests, 11 DOSBox tests, 35 examples, 67 JVM tests, 1 language
test, 11 general record tests, and 40 record-method tests. Rust formatting,
Graphify refresh, and `git diff --check` pass.

Iteration 457 extends the Stage 67 `compile_file` differential with a plain
variable FOR limit mutated by the loop body. Direct generation and
`compile_file` output match across BASIC, C, and JVM, covering the documented
once-at-entry bound semantics at the driver boundary. The full locked workspace
suite passes: 1,035 library tests, 19 CLI tests, 11 DOSBox tests, 35 examples,
67 JVM tests, 1 language test, 11 general record tests, and 40 record-method
tests. Rust formatting and `git diff --check` pass.

Iteration 458 fixes JVM typed numeric comparison semantics for `Double` values.
The semantic IR and AST compatibility codegen backends now emit `dcmpg` or
`dcmpl` with operator-specific branches, so NaN remains unordered and signed
zero compares equal, matching BASIC/C relational behavior. A JVM runtime
regression covers NaN equality, inequality and relational comparisons plus
signed zero; JVM codegen assertions cover both semantic IR and AST paths. The
full locked workspace suite passes: 1,035 library tests, 19 CLI tests, 11
DOSBox tests, 35 examples, 68 JVM tests, 1 language test, 11 general record
tests, and 40 record-method tests. Focused Rust formatting and `git diff
--check` pass. Workspace `cargo fmt --check` remains blocked by unrelated
pre-existing formatting differences across other dirty files.

Iteration 459 preserves binary32 precision for typed JVM `SINGLE` expressions
while retaining the existing double-width JVM storage layout. Typed semantic
expressions now round at `SINGLE` nodes, and scalar/array assignment plus
compound assignment round at the destination type. A JVM runtime regression
first reproduced the mismatch at 16,777,216 + 1 and now covers scalar
assignment, expression comparison, compound assignment, array compound
assignment. The full locked workspace suite passes: 1,035 library tests, 19
CLI tests, 11 DOSBox tests,
35 examples, 69 JVM tests, 1 language test, 11 general record tests, and 40
record-method tests. Graphify was refreshed;
focused Rust formatting and `git diff --check` pass.

Iteration 460 rounds typed JVM console INPUT values to the destination
`SINGLE` precision before storing into scalar or array targets. The assembled
runtime regression supplies 16,777,217 and verifies both destinations retain
16,777,216 under binary32 precision. The full locked workspace suite passes:
1,035 library tests, 19 CLI tests, 11 DOSBox tests, 35 examples, 69 JVM tests,
1 language test, 11 general record tests, and 40 record-method tests.
Graphify was refreshed; focused Rust formatting and `git diff --check` pass.

Iteration 461 makes the public `compile_source` pipeline require generated
semantic IR before invoking BASIC codegen. Generated-frontend failures now
return source diagnostics instead of silently reaching AST codegen. That
exposed `DIM arr%()` as a grammar gap; `array_axes` now has an explicit empty
form, and the adapter represents it as one inferred axis. A regression checks
both generated BASIC and typed `DimDeclaration` shape. The full locked suite
passes: 1,035 library tests, 19 CLI tests, 11 DOSBox tests, 35 examples, 69
JVM tests, 1 language test, 11 general record tests, and 40 record-method
tests. Graphify was refreshed, and focused Rust formatting plus
`git diff --check` pass. Workspace-wide rustfmt validation still reports
pre-existing formatting differences across dirty files. `compile_file` and
direct resolver/backend callers still retain AST compatibility paths for
later migration slices.

Iteration 462 makes `compile_file` require generated semantic IR for the root
source and every required library. Generated parser diagnostics and file
loading errors now propagate through the driver; resolver and FBC capability
validation receive the required semantic module, and the driver no longer
uses its AST-only FBC rejection or legacy form-warning path. The full locked
workspace suite passes: 1,035 library tests, 19 CLI tests, 11 DOSBox tests,
35 examples, 69 JVM tests, 1 language test, 11 general record tests, and 40
record-method tests. `cargo check --locked` and `git diff --check` pass.
Graphify was refreshed. Codegen backends still retain AST-based emission and
compatibility metadata; typed-IR-only backend consumption remains incomplete.

Iteration 463 makes resolver codegen metadata follow the generated semantic IR
when it is available. Array ranks, integer constants, record buffer names,
error-handler targets, catch-source tracking, and callable globals now come
from `SemanticModule`/`SemanticNameScopes`; AST scans remain only for the
explicit AST-only `resolve(program)` compatibility path. Legacy-only caches
for constant AST values and C constant identifiers remain empty on semantic
resolution, where backends already consume typed declarations directly. A
regression verifies these typed facts and empty compatibility caches. The full
locked suite passes: 1,036 library tests, 19 CLI tests, 11 DOSBox tests, 35
examples, 69 JVM tests, 1 language test, 11 general record tests, and 40
record-method tests. Graphify was refreshed and `git diff --check` passes.
Callable definitions and statement bodies are still emitted from AST by
partially migrated backends and remain on the migration path.

Iteration 464 removes BASIC codegen's source-position join back to AST for
FIELD buffer-name discovery. In semantic mode, BASIC storage allocation now
uses typed `FieldBinding` names and lowered record-file layout declarations;
AST traversal remains only in the explicit legacy path. `cargo check --locked`,
the record-buffer focused tests, record-method tests, and the full locked suite
pass (1,036 library tests, 19 CLI tests, 11 DOSBox tests, 35 examples, 69 JVM
tests, 1 language test, 11 general record tests, and 40 record-method tests).
Graphify was refreshed, focused rustfmt checks pass, and `git diff --check`
passes. Function definitions and statement bodies are still AST codegen inputs
in the partially migrated backends.

Iteration 465 teaches the JVM semantic block emitter to consume typed source
comments as no-op nodes. A `SELECT CASE` whose nested branch includes a
comment therefore remains on the typed emitter path instead of atomically
falling back to its AST body. The regression now uses divergent typed and AST
selectors/bodies and verifies only the typed selector and body are emitted.
The full locked suite passes: 1,036 library tests, 19 CLI tests, 11 DOSBox
tests, 35 examples, 69 JVM tests, 1 language test, 11 general record tests,
and 40 record-method tests. Focused rustfmt and `git diff --check` pass.
Callable declarations and unsupported semantic statements still have AST
codegen paths.

Iteration 466 expands the JVM semantic block emitter to handle typed `DIM`
declarations as preallocated-storage no-ops, typed local `CONST` initialization,
and discarded callable invocations. These nodes no longer force supported
nested control-flow bodies back to AST emission. `cargo check --locked`, the
full locked suite (1,036 library tests, 19 CLI tests, 11 DOSBox tests, 35
examples, 69 JVM tests, 1 language test, 11 general record tests, and 40
record-method tests), focused rustfmt, and `git diff --check` pass. Callable
method declarations and remaining unsupported typed statement kinds still
need migration.

Iteration 467 adds typed `GOTO` emission to JVM semantic control-flow blocks
when the target label is present in that typed block. The emitter now collects
labels from typed nodes and emits the branch directly, avoiding AST fallback
for locally resolved branches. The full locked suite and `cargo check
--locked` pass; focused rustfmt and `git diff --check` pass. This does not yet
remove the outer callable/top-level AST statement alignment or fallback paths.

Iteration 468 extends JVM typed block emission with unconditional typed
`GOTO` (validated by the resolver), and treats `DATA` declarations as no-op
nodes after their values have been incorporated into the module data table.
The nested-control-flow regression now verifies the typed condition and body
survive a nested GOTO without AST fallback. The full locked suite (1,036
library tests plus all integration suites), `cargo check --locked`, focused
rustfmt, and `git diff --check` pass. JVM statement fallback remains for
unimplemented semantic nodes; callable declarations and body traversal still
consume AST definitions.

Iteration 469 routes top-level and callable JVM source comments through the
semantic dispatcher as no-op typed nodes, matching nested-block handling.
This avoids invoking AST statement emission for comments in semantic mode.
The full locked suite (1,036 library tests and all integration suites),
`cargo check --locked`, focused rustfmt, and `git diff --check` pass. This
small dispatcher cleanup does not change the remaining AST callable and
statement fallback scope.

Iteration 470 handles typed `END` inside JVM semantic control-flow blocks.
Top-level blocks emit an INKEY restore followed by `return`, while callable
blocks restore terminal state and emit `System.exit(0)`, preserving the
backend's existing top-level/callable distinction without AST statement
fallback. The full locked suite, `cargo check --locked`, focused rustfmt, and
`git diff --check` pass.

Iteration 471 changes BASIC top-level dispatch to attempt whole-stream typed
IR emission before AST-aligned per-statement dispatch. Streams accepted by the
semantic emitter now produce BASIC solely from `SemanticModule` statement
order and structure, without AST source-position alignment; the existing
aligned bridge is retained only when whole-stream emission declines. The full
locked suite passes (1,036 library tests, 19 CLI tests, 11 DOSBox tests, 35
examples, 69 JVM tests, 1 language test, 11 general record tests, and 40
record-method tests). `cargo check --locked`, focused rustfmt, and
`git diff --check` pass.

Iteration 472 adds a regression showing BASIC's whole-stream semantic emitter
can compile a supported typed program even when its source filename and
statement stream do not align with the compatibility AST. BASIC now attempts
whole-stream typed emission first; the regression verifies typed `STOP`/`END`
replace the unrelated AST `BEEP`. A separate fallback regression still covers
streams declined by the typed emitter. The full locked suite passes: 1,037
library tests, 19 CLI tests, 11 DOSBox tests, 35 examples, 69 JVM tests, 1
language test, 11 general record tests, and 40 record-method tests. Cargo
check, focused rustfmt, and `git diff --check` pass.

Iteration 473 adds whole-callable typed emission to JVM codegen. The backend
now tries the semantic callable body as one transactional typed block before
per-statement source alignment; supported bodies emit without reading their
AST statements, while declined blocks preserve the compatibility bridge. A
regression uses a callable from a different source filename and verifies the
typed `return 2` is emitted instead of the AST `return 1`. The full locked
suite passes (1,038 library tests and all integration suites), along with
`cargo check --locked`, focused rustfmt, and `git diff --check`.

Iteration 474 adds a whole-stream typed block attempt for JVM top-level
statements, parallel to the callable-body typed attempt. It is used for a
single-source stream without blank-line trivia and only commits output when
the entire semantic block is supported; otherwise the aligned compatibility
path remains atomic. Typed comments are emitted as JVM assembly comments, and
streams with blank-line trivia retain the legacy formatting bridge so checked-
in tutorial assembly stays byte-identical. A regression with an unrelated AST
source file verifies typed top-level output. The full locked suite passes:
1,039 library tests, 19 CLI tests, 11 DOSBox tests, 35 examples, 69 JVM tests,
1 language test, 11 general record tests, and 40 record-method tests. Cargo
check, focused rustfmt, and `git diff --check` pass.

Iteration 475 makes JVM user-label collection lazy. Complete typed top-level
and callable streams no longer scan AST labels; the legacy AST label collector
runs only after the corresponding typed block declines and compatibility
emission is required. The full locked suite (1,039 library tests and all
integration suites), `cargo check --locked`, focused rustfmt, and
`git diff --check` pass.

Iteration 476 removes C callable codegen's AST FIELD-name reconciliation
when semantic FIELD bindings are available. Semantic FIELD statements now
suppress AST-derived storage candidates, and typed `FieldBinding` names plus
lowered record-file fields determine which local names are removed/promoted.
This fixes a stale AST buffer appearing beside a different typed FIELD
binding. The focused callable-FIELD regression, full locked suite (1,039
library tests plus all integration suites), `cargo check --locked`, focused
rustfmt, and `git diff --check` pass.

Iteration 477 removes C callable codegen's AST GLOBAL-declaration cleanup pass.
Matched semantic GLOBAL nodes now suppress AST-derived local candidates at
collection time; resolved typed callable-global bindings are filtered from
locals and sourced from the semantic scope facts. The focused resolver GLOBAL
tests and full locked suite (1,039 library tests plus all integration suites)
pass, as do `cargo check --locked`, focused rustfmt, and `git diff --check`.

Iteration 478 sources C callable scalar-local candidates from typed IR names
whenever a matched semantic callable body is available. The AST variable walk
is now limited to callables without typed bodies, and stale AST parameter names
no longer supplement the resolved signature. Qualified record-member names
are excluded from scalar candidates; focused record examples and the full
locked suite (1,039 library tests plus all integration suites) pass.

Iteration 479 replaces the remaining C callable AST GLOBAL reconciliation
with typed GLOBAL declarations whenever a semantic callable body is present.
The AST reconciliation remains only for AST-only compatibility callables. The
full locked suite (1,039 library tests plus all integration suites) passes.

Iterations 480–489 continue the C typed-IR migration across ten focused
passes: (480) derive aligned callable scalar candidates from typed statements;
(481) derive aligned callable scalar types and BYREF actuals from typed
statements; (482) use typed GLOBAL declarations in aligned callables; (483)
use typed FIELD bindings for aligned-callable storage; (484) emit callable
signatures from resolved `FnSig` data; (485) exclude AST defaults when a
typed callable signature is selected; (486) take callable result suffixes
from typed IR without AST fallback; (487) take parameter passing mode from
typed IR; (488) take parameter array rank from typed IR; and (489) take
parameter names from typed IR. The full locked suite (1,039 library tests
plus all integration suites) passes after these changes.

Iterations 490–502 continue the typed-IR migration across C, BASIC, and JVM
callable metadata. C64 float-variable candidates now come from typed root and
callable statements and typed callable signatures; AST identifier checks are
limited to unmatched or unsupported statements. C callable lookup and BYVAL
array validation share the typed callable matcher, and callable receiver and
symbol metadata use the selected typed signature. BASIC callable construction
uses one exact typed callable matcher for metadata and array capacities, and
its `FunctionInfo` return, procedure, and receiver metadata comes from typed
IR when matched. JVM callable parameter names, receiver type, array rank, and
BYREF mode now use the typed signature without AST fallback for matched
parameters. The full locked suite (1,039 library tests plus all integration
suites), `cargo check --locked`, focused rustfmt, and `git diff --check` pass.

Iteration 503 aligns BASIC method signature construction with the resolved
receiver type. `FunctionInfo` now uses the selected typed callable receiver both
when synthesizing its hidden `self` parameter and when deriving its label and
receiver metadata; the AST receiver remains only the AST-only compatibility
fallback. `rustfmt` and `git diff --check` pass. The focused and full test suites
have not been run for this pass.

Iteration 504 narrows JVM callable label discovery in the mixed typed/AST
compatibility path. Labels are collected from typed statements for aligned
entries and from AST statements only where source alignment left an entry
unmatched, so stale AST labels cannot affect branches emitted from typed IR.
Focused rustfmt, `cargo check --locked`, and `git diff --check` pass; tests have
not been run for this pass.

Iterations 505–506 remove two AST-keyed callable GLOBAL lookups from typed
backend paths. C callable emission now passes GLOBAL bindings collected from
the selected typed callable body; JVM callable slot binding derives the same
bindings from its typed callable body. The AST resolver caches remain the
fallback when no typed callable matches. Focused rustfmt, `cargo check --locked`,
and `git diff --check` pass; tests have not been run for these passes.

Iteration 507 applies the mixed-stream label rule to JVM top-level emission as
well as callable bodies. The JVM fallback emitter now builds its branch-label
set from typed labels for aligned statements and AST labels only for
unmatched compatibility statements. Focused rustfmt, `cargo check --locked`,
and `git diff --check` pass; tests have not been run for this pass.

Iteration 508 makes BASIC callable GLOBAL collection read the selected typed
callable body directly rather than looking up resolved global names with an
AST-derived callable key. Record storage globals are still added to the same
set. Focused rustfmt, `cargo check --locked`, and `git diff --check` pass; tests
have not been run for this pass.

Iteration 509 makes JVM method declarations use the typed callable identifier
stored with the resolved `FunctionSig`; the AST callable name remains only the
compatibility fallback when no typed signature matched. Focused rustfmt,
`cargo check --locked`, and `git diff --check` pass; tests have not been run for
this pass.

Iteration 510 makes BASIC's callable epilogue decision consume the matched
callable's typed statement order. A trailing typed `RETURN` now suppresses the
implicit return without AST/source-alignment inspection; AST return detection
remains for callables without a typed match. Focused rustfmt,
`cargo check --locked`, and `git diff --check` pass; tests have not been run for
this pass.

Iteration 511 derives BASIC's unreturnable error-handler status from the
resolved `FunctionInfo` procedure kind and source name, avoiding AST callable
metadata when typed IR matched. Focused rustfmt, `cargo check --locked`, and
`git diff --check` pass; tests have not been run for this pass.

Iteration 512 sources BASIC callable source-file markers from the matched typed
callable's `SemanticSource` whenever the corresponding statement is emitted
semantically. AST statement filenames are now limited to the unmatched
compatibility path, preventing stale AST locations from changing typed catch
source tracking. Focused rustfmt, `cargo check --locked`, and `git diff --check`
pass; tests have not been run for this pass.

Iteration 513 keys JVM function signatures and callable-body lookups by the
resolved callable identifier, including its typed result suffix. Calls and
method declarations now share that typed signature key; AST signature keys
remain the unmatched compatibility fallback. Focused rustfmt,
`cargo check --locked`, and `git diff --check` pass; tests have not been run for
this pass.

Iteration 514 removes C64 `reject_float`'s AST GLOBAL walk and AST-keyed
`callable_globals` reconciliation for matched typed callables. Both float
storage exclusion and callable global-type classification now use GLOBAL
nodes from the selected typed callable body; AST scans remain for unmatched
compatibility callables. Focused rustfmt, `cargo check --locked`, and
`git diff --check` pass; tests have not been run for this pass.

Iteration 515 removes AST parameter defaults from matched BASIC callable
signatures and teaches the compatibility call emitter to fill omitted
arguments from typed parameter defaults, including the hidden receiver offset
for methods. AST defaults are now used only for AST-only callable metadata.
Focused rustfmt, `cargo check --locked`, and `git diff --check` pass; tests have
not been run for this pass.

Iteration 516 makes typed JVM callable invocations use the target name stored
in the resolved `FunctionSig`, matching the method declaration and typed
signature key. AST names remain in the compatibility emitter. Focused rustfmt,
`cargo check --locked`, and `git diff --check` pass; tests have not been run for
this pass.

Iteration 517 makes C64's label exclusions for float diagnostics consume typed
LABEL nodes throughout module and callable bodies. AST label collection is
retained only for unmatched or `Unsupported` compatibility statements, so
stale AST labels cannot suppress or introduce diagnostics for typed code.
Focused rustfmt, `cargo check --locked`, and `git diff --check` pass; tests have
not been run for this pass.

Iteration 518 makes BASIC inferred-array-capacity diagnostics use the matched
typed callable name and parameter names when reporting unresolved `?` axes.
AST signature text is now used only for unmatched compatibility callables.
Focused rustfmt, `cargo check --locked`, and `git diff --check` pass; tests have
not been run for this pass.

Iteration 519 makes BASIC array-capacity call-site recognition seed its known
callable set from typed IR callable identifiers. AST callable names are added
only for functions that have no matched typed signature, preserving the
compatibility path. Focused rustfmt, `cargo check --locked`, and
`git diff --check` pass; tests have not been run for this pass.

Iteration 520 makes C backend callable-signature diagnostics identify matched
callables by the typed source name used for code generation. AST names remain
only for unmatched signatures. Focused rustfmt, `cargo check --locked`, and
`git diff --check` pass; tests have not been run for this pass.

Iteration 521 adds `DimDeclaration.element_type` to typed IR. The AST-to-IR
adapter now resolves an identifier type suffix first, then builtin `AS`
annotations when the identifier has no suffix, and retains the result for
backend consumers. C array declaration collection now reads that typed field
rather than reinterpreting the suffix; unknown/custom annotations retain the
prior Single fallback. The semantic type mapper accepts the existing integer
and floating-point aliases. `cargo check --locked` and `git diff --check` pass;
tests have not been run for this pass.

Iteration 522 updates JVM semantic array collection to map
`DimDeclaration.element_type` into the JVM array element type. Suffix and
annotation interpretation now happens before the backend; the JVM backend
retains its existing numeric-width mapping and Double fallback. `cargo check
--locked` and `git diff --check` pass; tests have not been run for this pass.

Iteration 523 updates semantic expression type annotation for array element
references to consume `DimDeclaration.element_type`, removing its duplicate
annotation and suffix interpretation. Unknown declared types retain the
existing Single default; references without declaration metadata remain
Unknown. `cargo check --locked` and `git diff --check` pass; tests have not
been run for this pass.

Iteration 524 updates BASIC array suffix resolution to consume typed element
types from callable-local and module-scope `DimDeclaration` metadata. Annotation
text remains the fallback only when the typed declaration table is unavailable.
`cargo check --locked` and `git diff --check` pass; tests have not been run for
this pass.

Iteration 525 updates C's semantic-DIM predeclaration check to select the C
array symbol from `DimDeclaration.element_type`, removing its independent
annotation parser and suffix override. `cargo check --locked` and
`git diff --check` pass; tests have not been run for this pass.

Iteration 526 updates C semantic scalar-DIM variable registration to use the
same typed element field as array declarations. Source suffixes remain
authoritative when present, with typed `AS` annotations supplying the type
otherwise. `cargo check --locked` and `git diff --check` pass; tests have not
been run for this pass.

Iteration 527 updates JVM semantic scalar-DIM registration to map
`DimDeclaration.element_type` directly, removing backend parsing of source
suffixes and annotation strings. JVM's Double representation for Single and
Double, plus its Double fallback, remain unchanged. `cargo check --locked` and
`git diff --check` pass; tests have not been run for this pass.

Iteration 528 changes C semantic `FOR` variable type lookup to return the
typed element type from `DimDeclaration` rather than an annotation string.
Suffix selection remains source-suffix-first, with the typed declaration used
when the identifier is suffixless. `cargo check --locked` and `git diff
--check` pass; tests have not been run for this pass.

Iteration 529 adds typed-IR regression coverage proving DIM declarations retain
suffix-derived integer/string/long types and an explicit `FLOAT64` annotation
as Double. `cargo test --locked
typed_dim_declarations_retain_suffix_and_annotation_element_types` passes.

Iteration 530 changes JVM semantic scalar declaration collection to use
`SemanticModule::const_types()` for CONST bindings, removing backend suffix and
initializer-type reconstruction. Nested statements share the resolved
constant type table. `cargo check --locked` and `git diff --check` pass; tests
have not been run for this pass.

Iteration 531 adds `value_type` to typed CONST nodes and populates it from the
semantic constant type table after expression annotation. C scalar storage and
assignment generation, plus JVM scalar slot registration, now consume that
explicit field. The unsuffixed initializer inference regression test passes;
`cargo check --locked` and `git diff --check` pass.

Iteration 532 updates C64's scalar-DIM storage scan to consume typed
`DimDeclaration.element_type` values for module and callable scopes, replacing
backend parsing of `AS` annotation text. Existing source suffixes remain
authoritative through the already-resolved typed field. Focused module and
callable C64 DIM tests, `cargo check --locked`, and `git diff --check` pass.

Iteration 533 removes suffix-first type reconstruction from C constant storage
selection and C64 constant storage scans. These paths now derive generated
storage suffixes from resolver-owned top-level constant type facts. Focused C64
module and callable CONST tests, `cargo check --locked`, and `git diff
--check` pass.

Iteration 534 updates JVM module CONST slot-key selection to derive the
generated identifier suffix and JVM slot type from the resolved constant type
table, even when the source identifier has an explicit suffix. `cargo check
--locked` and `git diff --check` pass; focused JVM constant-slot validation is
passing.

Iteration 535 updates BASIC generated-name reservation for semantic CONSTs to
use the resolved constant type rather than preferring a suffix parsed from the
typed source name. The focused BASIC CONST test and full `cargo test --locked`
suite pass; `git diff --check` passes.

Iteration 536 updates C callable record-file layout and record-local ownership
matching to use the resolved typed callable name when available. AST callable
name remains a compatibility fallback for unmatched legacy paths. The focused
procedure record-buffer test, `cargo check --locked`, and `git diff --check`
pass.

Iteration 537 updates C try-reachability key selection for callable prototypes
and definitions to use the resolved typed callable name. The focused C try
call-graph tests, `cargo check --locked`, and `git diff --check` pass.

Iteration 538 updates C callable array declaration collection to consume
per-source typed semantic statement dispatch when no complete typed callable
body is available. AST array DIM collection remains only when no semantic
callable representation matches. The focused callable array authority test,
`cargo check --locked`, full `cargo test --locked` (all suites pass), and
`git diff --check` pass.

Iteration 539 makes BASIC callable metadata construction treat a matched
typed callable signature as authoritative for local array ranks and `GLOBAL`
declarations, even if an individual typed fact table is absent. AST walks for
those facts are now limited to AST-only callers. Focused BASIC DIM and GLOBAL
tests, `cargo check --locked`, and `git diff --check` pass.

Iteration 540 removes AST variable collection from C callable local storage
when semantic statement dispatch exists. Local storage candidates now come
only from supported typed statements; unmatched or `Unsupported` AST nodes
cannot introduce C locals into a typed callable. Focused C callable field and
signature authority tests, `cargo check --locked`, and `git diff --check` pass.

Iteration 541 adds a regression test where the AST and typed callable bodies
declare different scalar locals at the same source position. C storage
declarations retain the typed local and omit the stale AST local. The focused
test, `cargo check --locked`, and `git diff --check` pass.

Iteration 542 audits the JVM callable codegen path: matched signatures supply
parameter layout and result type, `JvmContext` collects locals and arrays from
the typed callable body, and complete typed bodies bypass AST alignment. The
24 focused JVM callable unit tests and `git diff --check` pass; AST fallback
remains limited to compatibility and unsupported-statement paths.

Iteration 543 audits BASIC callable emission and metadata. Matched callable
signatures supply parameter bindings, result type, DIM facts, constants,
`GLOBAL` declarations, and semantic statement dispatch. The 32 focused BASIC
callable tests and `git diff --check` pass; AST paths remain for compatibility
callers and unsupported statements.

Iteration 544 validates driver integration across typed `compile_source` and
`compile_file` paths. The 17 semantic driver differential tests pass for
BASIC, C, C64, and JVM outputs, including required callables, arrays, records,
and scalar methods; `git diff --check` passes.

Iteration 545 rechecks C64 typed DIM storage and callable float-candidate
interaction. The two focused C64 semantic DIM tests pass, confirming module
DIM types do not mask typed callable float storage; `git diff --check` passes.

Iteration 546 runs the full C callable-focused test group after the local
storage and typed array collection changes. All 108 matching unit tests pass,
including generated C runtime integration where available; `git diff --check`
passes.

Iteration 547 runs the JVM source-aligned semantic-dispatch test group. All 25
matching unit tests pass across module and callable statements, including
typed expressions, control flow, I/O, and terminal operations;
`git diff --check` passes.

Iteration 548 runs the BASIC module-scope semantic-dispatch tests. All 16
matching unit tests pass for typed operands, declarations, comments, and
control flow; `git diff --check` passes.

Iteration 549 runs the C64 target-focused test group. All 36 matching unit
tests pass, covering typed declarations, callable storage, dialect capability
checks, and emitted C constraints; `git diff --check` passes.

Iteration 550 runs final locked compilation and whitespace validation after the
backend audit slices. `cargo check --locked --quiet` and `git diff --check`
pass.

Iteration 551 reruns the complete locked Rust test suite after all code changes.
All unit, driver, DOSBox, example, JVM conformance, language, record, and doc
test suites pass.

Iteration 552 reviews the final worktree and runs `git diff --check`. The
existing uncommitted repository changes remain intact; no commits were made.

Iteration 553 completes this 50-pass batch with `cargo check --locked --quiet`
and `git diff --check`; both pass.

Iteration 554 investigated typed BASIC emission for record-file
`FileDeclaration` nodes. A partial codegen-only change was rejected by the
full suite because `records::lower` also synthesizes GET/PUT operations for
record DSL expressions, and the semantic module does not yet carry those
transformed operations. The partial change was reverted. Both affected record
regression tests pass after the revert; the next slice must establish shared
typed record lowering before removing this AST dependency.

Iteration 555 emits module-scope typed record-file `FileDeclaration` nodes
from `LoweredRecordFile` when the lowered AST has no record operations that
still require `records::lower` output. A narrow compatibility gate keeps
record GET/PUT/CLOSE and field-assignment streams on the existing AST path;
ordinary GET/PUT operations do not trigger it. The focused typed declaration,
record mutation, and typed GET/PUT tests pass, as does the full `cargo test
--locked` suite; `cargo check --locked --quiet` and `git diff --check` pass.

Iteration 556 retains the resolved random-access record size as
`LoweredRecordFile.record_length`. Record lowering sources it directly from
the resolved record layout, and typed BASIC file-declaration emission now
consumes that explicit typed-IR fact instead of summing field widths. The
other C codegen width sums describe raw `FIELD` layouts and remain tied to
their declaration-order field metadata.

Iteration 557 also wires `LoweredRecordFile.record_length` into the C backend's
typed DSL record helper layout. Raw `FIELD` helper widths remain derived from
their active field declarations because those layouts do not have a DSL
record type. Focused C packed-record and partial-update runtime tests pass.

Iteration 558 adds each packed field's byte `offset` to `LoweredRecordField`
and computes it during record lowering. C's typed record layout prepass now
uses those explicit offsets when establishing the active DSL channel layout.
Focused JVM typed-layout and C packed-record runtime tests pass.

Iteration 559 extends the typed BASIC record-file declaration regression to
assert the record lowerer's `record_length`, field widths, and field offsets
directly, so the semantic metadata producer is covered independently of the
generated BASIC text.

Iteration 560 adds a C layout regression that supplies typed field offsets
and verifies the active channel layout preserves them, independently of the
generated C runtime helper text.

Iteration 561 replaces C's semantic record declaration shape reconstruction
with the canonical `LoweredRecordFile` field widths and string kinds. The
focused record-layout test now runs the record-lowering producer and passes
that metadata into the typed module before asserting the C layout table.

Iteration 562 validates the revised typed record layout path with the full
locked suite, `cargo check --locked --quiet`, `git diff --check`, and a
Graphify refresh; all pass.

Iteration 563 strengthens the C record-write regression by changing only the
typed `record_length` metadata and asserting that generated helper buffers
and record I/O use that value. The focused test passes.

Iteration 564 makes matched JVM callable signatures select parameter, array
element, and result slot types directly from typed suffix metadata. The
backend no longer synthesizes AST identifiers for those types; AST-only
callers retain the compatibility mapper. All 24 JVM callable unit tests pass.

Iteration 565 removes the remaining JVM scalar declaration type fallback from
matched semantic expressions. Name and by-reference argument storage now use
the resolved expression `value_type`, including its source suffix-derived
type, without re-reading the identifier suffix. The JVM callable tests and
`cargo check --locked --quiet` pass.

Iteration 566 validates the record layout and JVM callable type changes with
the full locked Rust suite, `cargo check --locked --quiet`, and
`git diff --check`; all pass.

Iteration 567 adds `For.variable_type` to typed IR, populated during AST-to-IR
adaptation from the loop identifier's source suffix. JVM scalar storage for a
semantic `FOR` variable now consumes that typed fact instead of re-parsing the
identifier. Producer and JVM regressions cover an explicit integer suffix
alongside an annotation-provided long type.

Iteration 568 updates the C typed `FOR` emitter and scalar declaration
collector to prefer `For.variable_type`, consulting scoped DIM metadata only
when the loop variable has no suffix-derived type. Its regression deliberately
changes the typed node's spelling while retaining its semantic type and
confirms that storage follows the typed fact.

Iteration 569 fixes the second C semantic `FOR` emission path to consume
`For.variable_type` as well. The full tutorial runtime test caught that this
path still defaulted a suffixed integer counter to the C `float` slot; the
targeted tutorial rebuild and execution now pass.

Iteration 570 reruns the full locked suite after that correction. All 1,043
library tests and integration suites pass, along with `cargo check --locked
--quiet` and `git diff --check`.

Iteration 571 refreshes Graphify after the typed `FOR` changes and confirms the
worktree still contains only the ongoing uncommitted migration changes.

Iteration 572 makes matched JVM callable result slots use the typed callable
result suffix even when it conflicts with the AST callable name. AST result
inference remains only for unmatched compatibility callables; focused
semantic-signature tests pass.

Iteration 573 adds explicit error-code and source-line types to typed-IR
`CatchBinding` and makes C/JVM declaration collectors consume those types.
The catch-binding producer test and a JVM authority regression pass, including
when the semantic identifier spelling is deliberately stale.

Iteration 574 adds the matching C catch-storage authority regression, proving
that C storage follows typed catch-binding types rather than suffixes in the
identifier spelling. The focused test and `cargo check --locked --quiet` pass.

Iteration 575 makes matched JVM callable receiver slots consume the receiver's
typed signature suffix directly, without synthesizing a `BasicIdent` to infer
the JVM type. The existing semantic-signature authority regression passes,
along with `cargo check --locked --quiet`.

Iteration 576 makes BASIC semantic method-call temporaries use the selected
callable's typed parameter suffix. Its regression deliberately gives the AST
method parameter a string suffix while the typed signature resolves it as an
integer; generated temporary storage follows the typed integer suffix. The
focused test passes.

Iteration 577 validates the receiver and method-parameter slices with the
complete `cargo test --locked` suite (1,046 library tests plus all binary and
integration suites), `cargo check --locked --quiet`, and a Graphify refresh.
All validation passes.

Iteration 578 adds resolved `SemanticValueType` metadata to typed-IR GLOBAL
declarations and exposes it through `SemanticNameScopes`. JVM global slot
registration now consumes that fact instead of inferring a type from the
identifier suffix. The semantic producer assertion and cross-backend global
name migration regression pass.

Iteration 579 validates the typed GLOBAL contract with the full locked suite,
`cargo check --locked --quiet`, `git diff --check`, and a Graphify refresh. All
1,046 library tests and every integration suite pass.

Iteration 580 adds a JVM slot authority regression that deliberately changes a
typed GLOBAL binding to `Long` while retaining its integer-suffixed name. The
generated global descriptor is `J`, proving the slot type comes from typed IR;
the focused test passes.

Iteration 581 reruns the full locked suite after the GLOBAL authority test.
All 1,047 library tests and integration suites pass, as do `cargo check
--locked --quiet` and `git diff --check`.

Iteration 582 wires typed GLOBAL binding suffixes into C global storage and
callable-global name registration. A C authority regression changes a `%`
binding's typed value to `Long` and confirms storage is registered with the
resolved `_l` suffix. The focused regression and existing cross-backend global
name test pass.

Iteration 583 validates the typed GLOBAL producer and C/JVM consumers with the
full locked suite (1,048 library tests and every integration suite),
`cargo check --locked --quiet`, `git diff --check`, and a Graphify refresh. All
validation passes.

Iteration 584 closes the C global declaration collector uncovered by its
authority regression: C storage registration and callable-global naming now
prefer the explicit `SemanticNameScopes.global_types` fact. The focused test,
full locked suite, `cargo check --locked --quiet`, `git diff --check`, and a
Graphify refresh pass.

Iteration 585 assigns suffixless GLOBAL declarations their resolved default
`Single` type in typed IR instead of leaving the type `Unknown`. The declaration
adapter test verifies both suffixed Integer and suffixless Single bindings;
focused semantic, C, and JVM tests pass.

Iteration 586 validates suffixless GLOBAL typing with the full locked suite
(1,048 library tests and all integration suites), `cargo check --locked
--quiet`, `git diff --check`, and an incremental Graphify refresh. All pass.

Iteration 587 updates BASIC semantic TRY/CATCH emission to use the typed
`CatchBinding.error_type` and `line_type` facts for both module and callable
catch slots. A regression makes the catch identifier spellings stale and
confirms emitted BASIC identifiers retain the typed integer suffix.

Iteration 588 validates the BASIC catch-binding change with all 1,049 library
tests and all integration suites, `cargo check --locked --quiet`,
`git diff --check`, and a Graphify refresh. All pass.

Iteration 589 adds a BASIC module-stream authority regression: when source
alignment metadata is unavailable, assignments and standard PRINT statements
still transpile from typed IR, and stale AST names and values do not leak into
the output. The focused test passes.

Iteration 590 reruns `cargo test --locked` after the BASIC typed-stream
regression. All 1,050 library tests and every integration suite pass;
`cargo check --locked --quiet` and `git diff --check` pass as well.

Iteration 591 updates JVM semantic TRY emission to look up catch slots using
the typed error and line suffixes, matching the declaration collector. The
top-level TRY authority regression now changes the binding spellings while
retaining Integer types and still transpiles successfully.

Iteration 592 validates JVM catch-slot emission with the full locked suite
(1,050 library tests and every integration suite), `cargo check --locked
--quiet`, and `git diff --check`. All pass.

Iteration 593 records the optional catch source binding's `String` type
explicitly in typed IR. BASIC, C, and JVM catch storage/emission now take the
source binding suffix and backend type from that field, alongside the typed
error and line bindings.

Iteration 594 extends the catch authority regressions so a stale `%` spelling
for the source-filename binding still selects String storage in BASIC, C, and
JVM. The focused backend tests and the catch-span/type producer test pass.

Iteration 595 validates the complete typed catch-binding path with the full
locked suite (1,050 library tests and all integration suites),
`cargo check --locked --quiet`, `git diff --check`, and a Graphify refresh. All
pass.

Iteration 596 extends the JVM TRY authority test to callable-local slots. A
stale error/line suffix pair in the typed callable still resolves to the
Integer slots recorded by its catch binding; the focused callable TRY test
passes.

Iteration 597 adds the matching BASIC callable TRY authority case. It makes
error, line, and source-filename spellings disagree with the retained Integer,
Integer, and String facts; callable-local BASIC storage and source assignment
still use the typed binding types. The focused callable TRY test passes.

Iteration 598 reruns the full locked suite after the top-level and callable
catch authority regressions. All 1,050 library tests and every integration
suite pass; `cargo check --locked --quiet` and `git diff --check` pass.

Iteration 599 resolves suffixless catch error and line bindings to `Single` in
typed IR, matching BASCAL's default local type and preventing backend-specific
fallbacks. The catch-binding producer regression verifies both suffixed
Integer and suffixless Single cases.

Iteration 600 validates suffixless catch typing with the full locked suite
(1,050 library tests and all integration suites), `cargo check --locked
--quiet`, `git diff --check`, and an incremental Graphify refresh. All pass.

Iteration 601 adds C and JVM declaration-storage assertions for suffixless
catch error and line bindings. Both backends now consume the adapter's explicit
`Single` type consistently, producing C `float` storage and JVM `double` slots;
the focused tests pass.

Iteration 602 reruns the full locked suite after suffixless catch storage
coverage. All 1,050 library tests and every integration suite pass, along with
`cargo check --locked --quiet` and `git diff --check`.

Iteration 603 makes BASIC semantic `FIELD` declarations apply each typed
`FieldBinding.type_suffix` when rendering its variable name. A stale `%` name
with a retained String suffix still emits a `$` field binding; the focused
authority test passes.

Iteration 604 validates the BASIC `FIELD` type change with all 1,051 library
tests and every integration suite, `cargo check --locked --quiet`, and
`git diff --check`. All pass.

Iteration 605 audits the C semantic numeric and string expression renderers
against the typed-IR-only backend rule. Scalar classification and storage
suffixes are taken from `Expression.value_type`; name spelling is used only to
derive the target identifier stem. No backend type-inference change was needed
in these paths. The wider C generator still retains legacy `Program` inputs
for callable tables, capability validation, and compatibility emission; those
dependencies require separate migration slices.

Iteration 606 reruns the full locked suite after the C backend audit: all
1,051 library tests and every integration suite pass. `cargo check --locked
--quiet` and `git diff --check` also pass.

Iteration 607 audits the JVM semantic scalar, input, and string expression
emitters. Their opcode and value-category decisions are checked against
`Expression.value_type`; `BasicIdent` parsing supplies the source binding key
only. The remaining `Program` dependency in `JvmContext` and callable-table
construction is still a Stage 66/68 migration boundary.

Iteration 608 records the backend boundary review: removing the remaining JVM
AST inputs requires transferring callable definitions and compatibility
context into typed-IR-owned records, then updating every context constructor
and callable emitter together. No partial signature change was made because
that would preserve the same AST dependency under a different interface.

Iteration 609 changes C64 `byval` array capability validation to enumerate
typed callable signatures when semantic IR is present. It retains the AST-only
compatibility path and restricts checks to callable overloads currently in
the C backend's emission set. A method-receiver overload regression caught an
initial overreach into typed-only overloads that are not emitted yet; the
filter now follows the actual backend boundary.

Iteration 610 validates the C64 capability slice: all 45 C64-focused library
tests pass, followed by the full locked suite (1,051 library tests and every
integration suite). `cargo check --locked --quiet` and `git diff --check`
also pass.

Iteration 611 preserves diagnostic location when a typed callable signature
has no retained semantic source text: the matching emitted AST declaration's
position remains the compatibility fallback. With semantic source text
available, the diagnostic position still comes from the typed callable span.
The focused C64 capability tests and the full locked suite pass after this
adjustment.

Iteration 612 audits the remaining C64 float-capability walker. Its AST
traversal is confined to compatibility statements represented as
`Unsupported` in typed IR and to locating emitted callable boundaries; typed
expressions supply capability facts. Eliminating those final AST joins depends
on removing AST-driven callable emission and unsupported-node fallback as
separate Stage 65/68 work.

Iteration 613 keeps the C64 capability diagnostic fallback for AST callables
that the generated semantic frontend has not mapped yet. Matched declarations
use typed parameter rank and passing mode; unmatched declarations still use
the parser AST because C emission retains them. A regression verifies that a
partial semantic module cannot let an emitted AST `BYVAL` array bypass the
C64 VLA diagnostic.

Iteration 614 validates the completed compatibility boundary with all 46
C64-focused library tests and the full locked suite (1,052 library tests and
all integration suites). `cargo check --locked --quiet` and
`git diff --check` pass.

Iteration 615 strengthens the partial-frontend C64 regression to assert the
fallback diagnostic's filename and line, ensuring the retained AST callable
position remains useful when typed callable metadata is absent.

Iteration 616 reruns the C64-focused library suite after the diagnostic
assertion; all 46 tests pass, and `git diff --check` passes.

Iteration 617 changes the C semantic numeric callable regression so the
semantic default value differs from the compatibility AST default. Generated
C uses the typed default (`8`) and does not emit the stale AST default (`3`).

Iteration 618 applies the same authority check to string callable defaults:
the semantic default string (`"?"`) is emitted and the AST default (`"!"`)
is absent. All 42 semantic C/BASIC focused library tests pass.

Iteration 619 validates the typed callable-default authority regressions with
the full locked suite: all 1,052 library tests and every integration suite
pass. `cargo check --locked --quiet` and `git diff --check` pass.

Iteration 620 adds a C callable signature authority regression: the legacy
AST declares a `SINGLE` parameter, while the typed callable declares the
parameter as `INTEGER` under a new name. The generated C prototype and
callable body use the typed parameter name and integer ABI.

Iteration 621 runs the focused callable tests across the library, including
the C, BASIC, JVM, and driver callable suites; all 109 tests pass.

Iteration 622 validates the C callable parameter-type regression with the full
locked suite: all 1,053 library tests and every integration suite pass.
`cargo check --locked --quiet` and `git diff --check` pass.

Iteration 623 audits C callable signature construction after the parameter
authority regression. Result type, parameter count and names, passing mode,
array rank, and default expressions are selected from `CallableSignature`
when it matches an emitted callable. The AST remains the emission-set and
compatibility boundary, so removing it requires migrating callable body
emission and its call graph as a coordinated Stage 65/68 slice.

Iteration 624 confirms C, BASIC, JVM, and driver callable regressions pass
with typed callable signatures; the 109-test focused callable suite remains
green.

Iteration 625 adds `SemanticModule::typed_names_in_statements`, extending the
existing shared name walk to retain `(name, SemanticValueType)` for each
expression-level scalar or array reference. The original untyped name API
continues to serve visibility collection; typed consumers no longer need to
recover expression types from suffix-bearing names.

Iteration 626 migrates C semantic local declaration discovery and C64 float
capability discovery to the typed name occurrences. These paths now derive
scalar storage categories from `Expression.value_type`; record-qualified
names remain handled by record storage metadata and explicit typed lvalues.
Graphify was refreshed after adding the shared typed-name API and C consumers.

Iteration 627 validates the typed-name migration with the semantic IR and C64
focused regressions, all 1,054 library tests, every integration suite,
`cargo check --locked --quiet`, and `git diff --check`. All pass.

Iteration 628 adds the resolved effective `value_type` to semantic callable
parameters. Explicit suffixes remain available as source metadata, while
suffixless parameters record their implicit `Single` type in typed IR.

Iteration 629 migrates JVM callable signatures, BASIC typed callable argument
checks and parameter declarations, and C/C64 callable storage and capability
classification to `Parameter.value_type`. Backends no longer reconstruct
parameter types from `Parameter.type_suffix` or parameter identifier suffixes.

Iteration 630 validates suffix-bearing and suffixless callable parameter
types in semantic IR, runs the full locked test suite (1,054 library tests
and all integration suites), refreshes Graphify, and passes `git diff
--check`.

Iteration 631 extends BASIC typed `PRINT` emission to callable expression
operands. Callable-bearing print lists now use semantic expression rendering
and preserve left-to-right values with typed temporary snapshots where a
later operand could invoke another callable. A single callable result is
printed directly from its resolved result slot, matching the existing BASIC
calling convention.

Iteration 632 adds a regression where semantic `PRINT` differs from the AST
and includes a user callable. The full locked suite passes: 1,055 library
tests and all integration suites. BASIC compatibility output remains stable
for existing callable, array-bound, and driver regressions.

Iteration 633 extends the callable-scoped BASIC print-token renderer to
`PRINT` and `LPRINT`. It emits callable expression preludes before the output
statement and snapshots multi-expression lists to preserve left-to-right
evaluation when an operand invokes a callable. A single direct callable
result uses its typed result slot without allocating a redundant temporary.

Iteration 634 expands the Stage 64 regression to cover callable operands in
module `PRINT` and callable-local `LPRINT`. The full locked suite passes with
1,055 library tests and all integration suites; `cargo check --locked
--quiet` and `git diff --check` pass.

Iteration 635 reuses the semantic print-token renderer for module-scope
`LPRINT`, including callable operand preludes and ordered multi-expression
snapshots. Its context-free path uses the module callable table and does not
require AST print operands.

Iteration 636 extends the semantic-vs-AST regression to module and callable
`LPRINT`. The focused regression and full locked test suite pass (1,055
library tests plus all integration suites); `cargo check --locked --quiet`
and `git diff --check` pass.

Iteration 637 extends BASIC semantic `PRINT` destination rendering to typed
`USING` format and channel expressions. Destination expression preludes are
emitted before print-list operand preludes, preserving source evaluation
order.

Iteration 638 verifies `PRINT USING` with a user-callable format expression
through the semantic-vs-AST regression. The full locked suite passes (1,055
library tests and all integration suites); `cargo check --locked --quiet`
and `git diff --check` pass.

Iteration 639 migrates module and callable `LPRINT USING` format expressions
to typed expression rendering. The format prelude is emitted before the
print-list prelude, preserving destination-before-value evaluation order.

Iteration 640 covers callable `LPRINT USING` formats at module and callable
scope. The focused regression and complete locked test suite pass (1,055
library tests and all integration suites); `cargo check --locked --quiet`
and `git diff --check` pass.

Iteration 641 migrates BASIC semantic `WRITE` channel and value operands to
typed expression rendering in module and callable scopes. Multi-value lists
with callable operands use typed snapshots to preserve left-to-right
evaluation; single callable values use the resolved result slot directly.

Iteration 642 adds semantic-vs-AST coverage for callable WRITE channels and
multi-value operands. The focused test and full locked suite pass (1,055
library tests and all integration suites); `cargo check --locked --quiet`
and `git diff --check` pass.

Iteration 643 extends the WRITE regression to callable bodies and checks
module and callable emission together with the `PRINT`/`LPRINT` callable
operand matrix. The complete locked test suite passes (1,055 library tests
and all integration suites).

Iteration 644 completes validation for the callable WRITE migration with
`cargo check --locked --quiet` and `git diff --check`; both pass.

Iteration 645 migrates BASIC semantic `INPUT` channel expressions to typed
expression rendering at module and callable scope. Channel call preludes now
execute before the corresponding `INPUT` statement while input targets remain
typed lvalues.

Iteration 646 tests callable INPUT channels in module and callable bodies.
The full locked suite passes (1,055 library tests and all integration
suites); `cargo check --locked --quiet` and `git diff --check` pass.

Iteration 647 migrates BASIC semantic `LINE INPUT` channel expressions to
typed expression rendering in module and callable scopes. The input target
continues through the typed lvalue renderer.

Iteration 648 adds module and callable regressions for callable LINE INPUT
channels. Focused and full locked tests pass (1,055 library tests plus every
integration suite).

Iteration 649 migrates BASIC semantic `OPEN` path, channel, and optional
record-length operands to typed expression rendering in module and callable
scopes. Operand preludes execute in source order before the `OPEN` statement.

Iteration 650 adds semantic-vs-AST coverage for callable OPEN channels at
module and callable scope. The focused test and full locked suite pass (1,055
library tests and all integration suites).

Iteration 651 migrates BASIC semantic `CLOSE`, `KILL`, and `NAME` operands
to typed expression rendering in module and callable scopes. `NAME` source
and destination expressions retain their left-to-right evaluation order.

Iteration 652 adds semantic-vs-AST regressions for callable file-operation
operands. The focused test and complete locked suite pass (1,055 library
tests and all integration suites).

Iteration 653 broadens the callable-output regression to cover channel-based
`PRINT USING` at module and callable scope. Callable channel and format
expressions are emitted in source order before the typed print-list values;
the focused test and complete locked suite pass.

Iteration 654 completes this Stage 64 I/O-expression batch with
`cargo check --locked --quiet` and `git diff --check`; both pass. Semantic
module and callable emitters now accept typed callable operands for
`PRINT`/`LPRINT`, `WRITE`, `INPUT`, `LINE INPUT`, `OPEN`, `CLOSE`, `KILL`, and
`NAME`, while retaining the existing AST path for statement families whose
transformations are not yet represented in typed IR.

Iteration 655 extracts the BASIC typed lvalue renderer from `MID$` assignment
and reuses it for module and callable `READ` targets. Array indices containing
resolved callable expressions are evaluated once in source order and
snapshotted before `READ`; callable expressions themselves remain invalid
lvalues. The regression compares stale AST and typed-IR inputs at both scopes.
The focused regression, full locked suite (1,056 library tests plus every
integration suite), `cargo check --locked --quiet`, and `git diff --check` pass.

Iteration 656 reuses typed lvalue rendering for BASIC `SWAP` at module and
callable scope. Callable expressions in array indices are emitted and
snapshotted in left-to-right order before `SWAP`. A differential regression
verifies both scopes use typed indices instead of stale AST indices; the
focused test and the full locked suite (1,057 library tests plus every
integration suite), `cargo check --locked --quiet`, and `git diff --check`
pass.

Iteration 657 routes BASIC `INPUT` and `LINE INPUT` destinations through the
typed lvalue renderer in module and callable bodies. Channel expressions are
evaluated before target indices, and each callable index is snapshotted once.
The differential regression verifies typed callable indices at module and
callable scope for both statement forms. The focused test, full locked suite
(1,058 library tests plus all integration suites), `cargo check --locked --quiet`,
and `git diff --check` pass.

Iteration 658 routes BASIC assignment targets through typed lvalue rendering.
Callable array indices are snapshotted before RHS evaluation, preserving
left-to-right evaluation when both the target index and value invoke
callables. Scalar and non-callable array assignments retain their existing
direct output. The differential regression covers module and callable
assignments with callable target and RHS expressions.

Iteration 659 extends semantic BASIC `DIM` to transpile callable array-bound
expressions in module and callable scopes. Bound preludes execute before the
declaration and dimensions remain ordered; constant-folded bounds retain the
existing direct output. The focused regression, full locked suite (1,060
library tests plus all integration suites), `cargo check --locked --quiet`,
and `git diff --check` pass.

Iteration 660 adds C semantic-vs-AST coverage for array assignment targets
whose indices invoke a callable, at module and callable scope. The C semantic
assignment path already consumes the typed array target and retains its
callable index; the regression records this cross-backend behavior.

Iteration 661 extends C semantic `READ` and `INPUT` lvalue rendering to
parenthesized array references containing callable indices. Their emitters
now receive the callable map and use the typed numeric-expression renderer
for each index; the common lvalue renderer accepts resolved array calls while
rejecting callable invocations as targets. Module and callable regressions
verify stale AST indices do not leak. The full locked suite (1,063 library
tests plus all integration suites), `cargo check --locked --quiet`, and
`git diff --check` pass. Graphify was refreshed; the C READ emitter's direct
call path to the numeric semantic expression renderer was confirmed.

Iteration 662 adds JVM semantic-vs-AST coverage for callable array indices in
typed callable assignments and `INPUT` targets. The generated bytecode retains
the semantic callable invocation, assignment value, and prompt instead of
stale AST operands; no additional backend change was required.

Iteration 663 covers rank-two BASIC assignment lvalues whose two array axes
invoke a callable. Module and callable emitters snapshot both axes in source
order and use the typed assignment value. The focused regression and full
locked suite (1,065 library tests plus all integration suites),
`cargo check --locked --quiet`, and `git diff --check` pass.

Iteration 664 adds an end-to-end C driver regression: the CLI compiles and
runs a program that assigns and reads through callable array indices. Runtime
output confirms the driver, semantic IR, and C codegen path agree; the full
locked suite (1,065 library tests plus all integration suites, including 36
example tests), `cargo check --locked --quiet`, and `git diff --check` pass.

Iteration 665 expands C `READ` and `INPUT` callable-index regressions to rank
two, covering both module and callable `READ` and module `INPUT`. Each axis
transpiles through the typed expression renderer, and callable results remain
in the emitted array target. The focused regressions, full locked suite
(1,065 library tests plus all integration suites), `cargo check --locked --quiet`,
and `git diff --check` pass.

Iteration 666 adds C file-`INPUT` coverage for a typed rank-two array target
whose axes invoke a callable. The regression supplies an AST with a different
channel while the semantic IR selects channel four, and verifies the C
transpilation uses the semantic channel and callable array indices. The
focused test and `git diff --check` pass.

Iteration 667 adds C `LINE INPUT` coverage for a typed rank-two string-array
target whose axes invoke a callable. The regression confirms both the
semantic channel and typed callable indices reach the C target lvalue. The
focused test and `git diff --check` pass.

Iteration 668 adds a JVM regression for semantic `LINE INPUT` into a typed
string-array element whose index invokes a callable. The emitted bytecode
contains the typed callable index and semantic channel, independent of the
stale AST operands. The focused test passes.

Iteration 669 adds JVM console-`INPUT` coverage for a typed numeric array
element indexed by a callable. The regression verifies the typed prompt,
callable index invocation, and array store in emitted bytecode. The focused
test passes.

Iteration 670 adds C console-`INPUT` coverage for callable indices on both
axes of a rank-two typed array. The regression verifies the semantic prompt
and target are emitted instead of the AST prompt and target. The focused test
passes.

Iteration 671 strengthens the JVM console-`INPUT` regression to use both axes
of a rank-two typed array, verifying both typed index calls appear before the
numeric array store. The focused test passes.

Iteration 672 strengthens the JVM `LINE INPUT` regression with a semantic
callable-index expression that differs from the AST (`index%() + 2`). The
assertion verifies the bytecode retains the typed addition before the file
channel operation. The focused test passes.

Iteration 673 enables whole-module BASIC typed-IR dispatch for structured
`TRY/CATCH/FINALLY` when source identity matches and catch-source mapping is
not required. Catch-source bindings retain the compatibility bridge because
their multi-file source map is still derived from AST metadata. Focused
`TRY`, fallback, and catch-source tests pass.

Iteration 674 adds the JVM source-aligned statement dispatch for semantic
comments. When blank lines force the per-statement bridge, typed line comments
now emit from semantic IR rather than the aligned AST comment. The differential
regression verifies both the comment and PRINT payload; the focused test passes.

Iteration 675 adds C semantic-comment emission to source-aligned module and
callable dispatch. Line and block comment text is emitted as target-safe C
line comments, while the typed IR remains the source of the comment and
adjacent PRINT payload. Module and callable AST-differential coverage passes.

Iteration 676 strengthens the C comment regression so module line comments
and callable block comments both exercise their semantic comment shapes. The
focused module/callable differential test passes.

Iteration 677 strengthens the JVM source-aligned comment regression with a
semantic block comment paired against an AST line comment. The typed block
comment and PRINT payload survive the per-statement dispatch; the focused test
passes.

Iteration 678 extends C's recursive semantic body emitter to transpile
comments from typed IR inside nested control-flow blocks. A regression changes
both nested PRINT and comment text between AST and semantic IR and verifies the
semantic block wins. The focused test passes.

Iteration 679 extends the JVM comment regression to callable bodies. A typed
block comment and changed PRINT payload transpile from callable semantic IR,
independent of the corresponding AST comments and expression. The focused test
passes.

Iteration 680 extends C recursive typed-IR body emission to console `INPUT`.
Nested console input now uses the shared typed prompt, lvalue, and conversion
emitter instead of falling back to the AST body. The regression verifies a
semantic prompt and typed target inside an `IF`; the focused test passes.

Iteration 681 extends the nested console-`INPUT` regression to callable
control flow. Module and callable `IF` bodies now verify that their typed
prompts survive AST alignment. The focused test passes.

Iteration 682 strengthens nested C console-`INPUT` coverage with an array
target indexed by a callable and a semantic-only `+ 2` operand. Generated C
retains the typed callable index expression before storing the input value.
The focused module regression passes.

Iteration 683 extends that callable-indexed nested `INPUT` case to a procedure
body, with a semantic-only `+ 3` index operand. Module and callable `IF`
bodies now cover typed array destinations. The focused test passes.

Iteration 684 extends nested C console `INPUT` coverage to rank-two arrays.
Both axes now use typed callable expressions that differ from their AST
indices; generated C retains both expressions in the array target. The focused
test passes.

Iteration 685 adds JVM nested console-`INPUT` coverage for a rank-two typed
array target. Both callable axes are retained, including a typed-only `+ 2`
expression, and the bytecode stores the parsed value in the array. The focused
test passes.

Iteration 686 adds JVM nested `LINE INPUT` coverage for a typed string-array
element indexed by a callable. The semantic channel and callable index
expression differ from the AST, and the nested bytecode retains both. The
focused test passes.

Iteration 687 adds an end-to-end C driver test that pipes console input into
a nested rank-two array `INPUT` target with callable axes. The generated C
program builds and runs through the CLI and prints the stored value. The
focused integration test passes.

Iteration 673 enables typed whole-module BASIC dispatch for structured
`TRY/CATCH/FINALLY`. The semantic `TRY` implementation was already available
to the aligned AST bridge; whole-module dispatch had disabled it. The
regression directly verifies the typed dispatcher accepts the semantic
statement and checks filters, catch, and `FINALLY` output. Focused tests and
`git diff --check` pass.

Iteration 688 threads the shared C `FileIoLayout` through recursive typed
control-flow emitters. Nested C `INPUT` and `LINE INPUT` statements can now
use the same resolved channel and file-helper state as top-level and callable
statements. `cargo check --locked --quiet` and the full `cargo test --locked`
suite pass; Graphify was refreshed after the backend dependency change.

Iteration 689 adds a differential regression for C file `INPUT` nested inside
an `IF`. It verifies the nested body emits the typed array index and file
channel, even when both differ from the AST. The focused test passes.

Iteration 690 adds the matching differential regression for nested C file
`LINE INPUT`. The typed string-array target and channel survive recursive
control-flow emission. The focused test and full `cargo test --locked` suite
pass.

Iteration 691 adds callable-body coverage for nested C file `INPUT` and
`LINE INPUT` in one procedure. The typed target ordering, array indices, and
distinct channels are emitted; AST operands do not leak into the callable.
The focused test passes.

Iteration 692 extends the recursive C typed-body emitter to support `CLOSE`
using the typed channel expression and shared `FileIoLayout`. A nested
differential regression verifies the channel differs from the AST; the
focused test passes.

Iteration 693 adds callable-body coverage for nested typed C `CLOSE`; both the
typed channel slot and cleanup assignment are emitted instead of the AST slot.
The focused test passes.

Iteration 694 routes nested C channel `PRINT` through the shared semantic
print emitter from recursive typed bodies. The regression verifies both the
typed channel and payload replace their AST counterparts. The focused test
passes.

Iteration 695 extends nested channel `PRINT` coverage to callable bodies. The
typed channel and payload survive callable control-flow emission; the focused
test passes.

Iteration 696 extends recursive C typed-body emission to channel `WRITE` via
the semantic writer and shared file-I/O layout. A differential nested-body
regression verifies the semantic channel and value replace AST operands. The
focused regression and full `cargo test --locked` suite pass.

Iteration 697 adds callable-body coverage for nested C `WRITE`; the typed
channel and value survive callable control-flow emission. The focused test
passes.

Iteration 698 extends recursive C typed-body emission to `SEEK`, consuming
typed channel and position expressions plus the existing channel layout. A
nested differential regression passes.

Iteration 699 adds callable-body coverage for nested typed C `SEEK`; the
focused regression passes.

Iteration 700 extends recursive C typed-body emission to random-file `GET`
and `PUT`. The differential nested regression verifies both typed record
positions and that the AST positions are absent; the focused test passes.

Iteration 701 extends recursive C typed-body emission to `LPRINT` without
`USING`, reusing the typed print-token emitter. A nested differential test
confirms that the semantic payload replaces the AST payload; the focused test
passes.

Iteration 702 adds callable-body coverage for nested typed C `LPRINT`; the
focused regression confirms the callable body emits the semantic payload.

Iteration 703 adds differential coverage for nested file `PRINT` within C
`WHILE` and `SELECT CASE` bodies, exercising both recursive control-flow
emitters with typed channels and payloads. The focused test passes.

Iteration 704 validates the accumulated recursive C file-I/O work with the
full `cargo test --locked` suite and `git diff --check`; both pass. Graphify
was refreshed after the recursive emitter gained additional typed statement
paths.

Iteration 705 adds callable differential coverage for typed file `PRINT`
nested under `WHILE` and `SELECT CASE`. Both control-flow emitters retain the
semantic channel and payload; the focused test passes.

Iteration 706 verifies that typed file `INPUT` nested under `SELECT CASE`
retains its channel and callable-indexed array target, including a semantic
only `+ 2` index. The focused test passes.

Iteration 707 verifies that typed file `WRITE` nested under `WHILE` retains a
semantic callable expression and addition instead of the AST expression. The
focused test passes.

Iteration 708 adds callable-body coverage for typed file `INPUT` nested under
`SELECT CASE`; the typed channel and array index replace their AST counterparts.
The focused test passes.

Iteration 709 adds the matching module regression for typed `LINE INPUT`
nested under `SELECT CASE`, checking the semantic channel and string-array
index. The focused test passes.

Iteration 710 extends nested `SELECT CASE` `LINE INPUT` coverage to callable
bodies. The typed channel and target replace the AST operands; the focused test
passes.

Iteration 711 extends recursive C typed-body emission to statement-form
`MID$` assignment, using the semantic lvalue, start, length, and string value.
A nested differential regression confirms the typed operands replace the AST
operands; the focused test passes.

Iteration 712 adds callable-body coverage for nested typed statement-form
`MID$` assignment; the focused differential regression passes.

Iteration 713 verifies nested C `READ` consumes both the semantic target and
the semantic module `DATA` pool when AST names and literals differ. The focused
test passes.

Iteration 714 adds callable nested `READ` coverage for the same typed target
and semantic `DATA` pool behavior; the focused test passes.

Iteration 715 adds a nested differential regression for C `RANDOMIZE`, proving
that the seed expression comes from typed IR; the focused test passes.

Iteration 716 extends typed nested `RANDOMIZE` coverage to callable control
flow; the semantic seed is emitted and the AST seed is absent.

Iteration 717 adds a module nested differential regression for C `COLOR`
operands; the focused test passes.

Iteration 718 extends nested typed `COLOR` coverage to callable bodies; the
focused test passes.

Iteration 719 adds a nested differential regression proving that C `LOCATE`
row and column operands come from typed IR; the focused test passes.

Iteration 720 extends nested typed `LOCATE` operand coverage to callable
bodies; the focused differential test passes.

Iteration 721 adds nested scalar `SWAP` coverage with typed lvalues differing
from the AST; the focused C regression passes.

Iteration 722 adds nested array `SWAP` coverage for typed indices, verifying
the recursive body emitter uses the semantic array elements; the focused test
passes.

Iteration 723 extends nested typed array `SWAP` coverage to callable bodies;
the typed elements replace the AST elements in the generated C procedure.
The focused regression passes.

Iteration 724 covers nested typed string-array `SWAP` in a C module body,
checking both semantic indices; the focused regression passes.

Iteration 725 adds callable nested string-array `SWAP` coverage with typed
indices; the focused regression passes.

Iteration 726 adds a nested control-flow differential regression where typed
`CONTINUE` replaces AST `EXIT`; the C emitter produces `continue` from semantic
IR. The focused test passes.

Iteration 727 extends the typed `CONTINUE` versus AST `EXIT` regression to
callable loop bodies; the focused C codegen test passes.

Iteration 728 verifies that nested typed C `EXIT` replaces AST `CONTINUE` in
a module loop; the focused regression passes.

Iteration 729 extends nested typed `EXIT` coverage to callable loops; the
focused regression passes.

Iteration 730 adds nested random-file `GET`/`PUT` regression coverage to
callable C bodies. The typed record positions are emitted instead of AST
positions; the focused test passes.

Iteration 731 adds a differential regression for C `RETURN` values in nested
function branches. Both typed return expressions replace their AST values;
the focused test passes.

Iteration 732 extends nested typed `RETURN` coverage to callable bodies; the
focused test passes.

Iteration 733 expands nested C file `WRITE` coverage to mixed string and
numeric typed values, validating generated format and arguments; the focused
test passes.

Iteration 734 extends mixed typed file `WRITE` coverage to callable nested
bodies; the focused test passes.

Iteration 735 adds module coverage for nested array `READ`, asserting both the
semantic DATA pool and typed array index; the focused test passes.

Iteration 736 extends typed nested array `READ` coverage to callable bodies;
the focused test passes.

Iteration 737 adds a C driver runtime test for nested `READ` from the typed
DATA pool. The compiler builds and runs the generated C program, which prints
the value read inside the `IF`; the integration test passes.

Iteration 738 validates the completed slice with full `cargo test --locked`
and `git diff --check`; both pass. Graphify was refreshed to include the
recursive C typed-body emitter paths.

Iteration 739 adds recursive C typed-body support for `OPEN ... FOR OUTPUT`
and `OPEN ... FOR APPEND`, with typed path and literal channel emission. A
nested differential OUTPUT regression passes.

Iteration 740 adds a nested APPEND regression verifying the typed path and
channel and the correct append mode; the focused test passes.

Iteration 741 extends nested typed OUTPUT `OPEN` coverage to callable bodies;
the focused regression passes.

Iteration 742 adds a C driver runtime test that opens a file for OUTPUT,
executes nested typed `WRITE`, closes the typed channel, and verifies the file
contains the expected record. The integration test passes.

Iteration 743 extends nested typed APPEND `OPEN` coverage to callable C bodies;
the focused differential test passes.

Iteration 744 verifies a nested C typed OUTPUT `OPEN` evaluates a semantic
string-returning path call through its generated temporary; the AST path is
absent and the focused test passes.

Iteration 745 extends typed path-call coverage to callable nested OUTPUT
`OPEN`; the focused differential regression passes.

Iteration 746 adds a C driver runtime test for callable nested typed OUTPUT
`OPEN`, `WRITE`, and `CLOSE`; the generated program writes the expected record
to the selected file and the integration test passes.

Iteration 747 adds recursive JVM typed-IR emission for file `WRITE`, including
typed channel and value expressions, CSV quoting for string values, comma
separators, and the record newline. A differential codegen regression proves
the nested emitter uses typed values instead of the AST values.

Iteration 748 adds JVM end-to-end conformance coverage for nested typed `WRITE`.
The generated class assembles and executes, and the output file contains the
expected CSV record.

Iteration 749 extends the JVM recursive typed-body emitter with sequential
`OPEN FOR OUTPUT` and `OPEN FOR APPEND`. The backend evaluates the typed
channel once, opens the typed path as a `RandomAccessFile`, truncates for
OUTPUT, and seeks to EOF for APPEND.

Iteration 750 adds JVM runtime coverage in a callable body for both typed
sequential open modes. OUTPUT replaces pre-existing contents, APPEND preserves
the new record and adds its record at EOF, and the generated class executes
successfully.

Iteration 751 strengthens the JVM callable OUTPUT regression with a
side-effecting typed channel expression. Runtime output verifies the channel
expression executes exactly once while the selected file is opened, written,
and closed.

Iteration 752 adds differential JVM codegen coverage for callable OUTPUT and
APPEND opens, checking typed paths, truncation and EOF-seek behavior, and the
absence of the corresponding AST paths in emitted bytecode.

Iteration 753 exercises typed OUTPUT `OPEN` in a callable conditional with a
string-returning path call and a side-effecting channel call. Runtime checks
confirm the path is honored, the channel call executes once, and the selected
file receives the expected bytes.

Iteration 754 moves both typed OUTPUT and APPEND opens under callable `IF`
bodies. JVM assembly and execution pass, covering recursive block emission
and file truncation/append behavior inside control flow.

Iteration 755 extends nested JVM `WRITE` runtime coverage with a side-effecting
typed numeric function call. The emitted CSV record contains its value, and
the runtime counter confirms the expression executes exactly once.

Iteration 756 adds a JVM class-level typed literal DATA pool and initializes
its shared read cursor before user statements. DATA items are collected from
typed IR across nested module and callable blocks; only typed literal numeric
and string values are accepted.

Iteration 757 implements JVM scalar `READ` from the typed DATA pool, using each
target's typed IR value type for string storage or numeric conversion. Runtime
coverage exercises a nested module read and a callable read sharing the same
cursor.

Iteration 758 extends JVM typed `READ` to resolved numeric and string array
elements, emitting typed indices and array store opcodes. Nested array READ
runtime coverage passes.

Iteration 759 validates signed floating DATA, long, and string conversion in
JVM `READ`, and adds a differential regression showing the emitted DATA item
and target come from typed IR rather than their AST counterparts.

Iteration 760 adds typed JVM `RESTORE` cursor resets to the beginning of the
DATA pool or to the pool offset recorded for a typed label. Runtime coverage
confirms both forms restore the expected next item.

Iteration 761 wires typed JVM DATA, READ, RESTORE, WRITE, and sequential OUTPUT
or APPEND OPEN handlers into the source-aligned per-statement path as well as
the recursive typed block emitter. A blank-line regression confirms READ
continues to consume typed DATA when whole-stream emission is unavailable.

Iteration 762 adds runtime coverage for reading a typed string DATA item into
a rank-two JVM string-array element, exercising semantic multidimensional
indices and reference-array storage.

Iteration 763 verifies a skipped conditional READ does not advance the shared
JVM DATA cursor; the next executed READ receives the first item.

Iteration 764 verifies DATA declarations inside an unexecuted conditional
block are still collected into the typed module pool, matching BASIC DATA's
declarative semantics.

Iteration 765 adds a shared JVM `bccReadData` runtime helper that advances the
typed DATA cursor only after validating the current index and throws an
`IllegalStateException` with `Out of DATA` when READ exhausts the pool.

Iteration 766 adds JVM runtime conformance coverage for the exhausted DATA
diagnostic; the program exits unsuccessfully with the expected message.

Iteration 767 adds callable JVM `RESTORE` runtime coverage to verify that
callable code updates the same static DATA cursor consumed by module code.

Iteration 768 verifies DATA declarations in callable bodies join the shared
typed pool after module DATA items even when the declaring callable is not
invoked.

Iteration 769 verifies JVM `READ` preserves SINGLE's binary32 rounding when
converting a typed DATA literal into the backend's double-based storage.

Iteration 770 verifies typed `RESTORE label` offsets are collected from nested
control-flow blocks even when the block containing the DATA label is not
executed.

Iteration 771 adds runtime coverage for a side-effecting typed array index in
JVM `READ`; the resolved index function executes exactly once before the
typed DATA value is stored.

Iteration 772 adds an early JVM diagnostic for `READ` when typed IR contains
no DATA items, matching the C backend's compile-time rejection instead of
emitting an unresolved runtime helper call.

Iteration 773 emits the JVM DATA-read helper only when typed READ call sites
exist in module or callable bytecode, avoiding an unused helper in DATA-only
programs.

Iteration 774 makes typed `RESTORE` without any DATA pool a no-op, so a valid
cursor reset does not force an otherwise unnecessary JVM runtime field.

Iteration 775 adds typed JVM `ON ... GOTO` dispatch with one-based selector
indices, a single selector evaluation, and typed label targets.

Iteration 776 adds JVM runtime coverage for a side-effecting selector call;
the selected label runs and the selector executes exactly once.

Iteration 777 verifies an out-of-range typed `ON ... GOTO` selector pops its
temporary value and falls through without branching.

Iteration 778 adds differential JVM codegen coverage proving typed `ON ...
GOTO` selector and label operands replace the AST operands.

Iteration 779 verifies JVM typed `RESTORE` without DATA emits no cursor state
or runtime instruction.

Iteration 780 verifies a DATA-only JVM module retains its typed pool but omits
the unused DATA-read helper method.

Iteration 781 executes typed `ON ... GOTO` from a nested callable conditional,
confirming generated branch labels remain valid across recursive block
emission.

Iteration 782 verifies the one-based selector value `1` dispatches to the
first typed `ON ... GOTO` target.

Iteration 783 verifies typed `ON ... GOTO` emits through source-aligned
per-statement dispatch when a blank line disables whole-module typed-stream
emission.

Iteration 784 covers a typed LONG selector dispatching to the third target in
a three-label JVM computed branch.

Iteration 785 verifies selectors greater than the typed target count fall
through without leaving an operand on the JVM stack.

Iteration 786 covers sequential typed `ON ... GOTO` statements with distinct
internal labels and confirms an out-of-range first branch does not interfere
with the later selected branch.

Iteration 787 adds direct codegen coverage for unique generated labels and
typed selector/target operands across multiple JVM `ON ... GOTO` statements.

Iteration 788 validates the expanded JVM codegen slice with the full locked
test suite, diff checks, and a Graphify refresh.

Iteration 789 adds differential coverage for typed JVM `IF` emission through
the source-aligned per-statement path used when blank lines split the typed
stream. The test supplies different AST and typed-IR branch literals and
confirms only the resolved typed arms are emitted; aligned typed GOTO/label
coverage now also exercises this split-stream path.

Iteration 790 treats typed JVM record-file declarations as compile-time
metadata already consumed by record-layout and file-I/O setup. Both recursive
typed-block and source-aligned per-statement dispatch now accept the
declaration without triggering AST statement fallback; existing try/catch
record-file compilation coverage validates the path.

Iteration 791 directly verifies that a JVM typed block containing a record
file declaration is handled completely and emits the typed terminator, so
this metadata-only declaration no longer silently sends the block to AST
codegen.

Iteration 792 retains resolved JVM `DIM` capacities as typed integer facts in
`ArrayShape` and emits those facts directly for array allocation and typed
`SIZEOF`/`UBOUND` evaluation. AST capacity expressions remain explicitly
tagged as compatibility-only dimensions. Module and callable shape tests
assert the typed dimension representation.

Iteration 793 verifies the typed-capacity path for ordinary bounds, named
`CONST` bounds, radix literals, and deliberately divergent typed versus legacy
array shapes. The generated JVM allocation continues to use the resolved
capacity values.

Iteration 794 validates the accumulated JVM typed-dispatch and array-capacity
changes with `cargo test --locked` (1,138 library tests plus all binary,
integration, fixture, and doc-test suites) and `git diff --check`; all pass.

Iteration 795 removes the final synthetic AST dimension from JVM callable
array-parameter shapes. Parameter rank now supplies typed placeholder axes;
the compatibility AST wrapper is reserved for actual AST-derived capacities.
Existing callable array ABI coverage validates parameter access and by-value
copy behavior after the change.

Iteration 796 reruns the JVM callable array-parameter ABI regression against
typed rank placeholders; the method still emits its typed by-value copy and
writable parameter access.

Iteration 797 validates the completed array-shape migration with `cargo test
--locked` (1,138 library tests and all integration and fixture suites) and
`git diff --check`; all pass.

Iteration 798 adds a callable-scope differential regression proving JVM array
allocation uses the typed callable `DIM` capacity when it differs from the
compatibility AST bound.

Iteration 799 verifies callable-local `CONST` expressions resolve into typed
JVM array capacities in the callable's semantic scope; the collected shape
retains the evaluated integer fact rather than an AST expression.

Iteration 800 adds callable arithmetic-bound coverage: a resolved `2 * 4`
capacity is retained as typed integer `8` by the JVM array collector.

Iteration 801 verifies AST-only JVM array collection wraps its dimensions in
the explicit compatibility variant, keeping legacy capacity expressions
separate from typed dimensions.

Iteration 802 validates the callable-scope and compatibility-shape regressions
with the full locked workspace suite (1,142 library tests and all binary,
integration, fixture, and doc-test suites) and `git diff --check`; all pass.

Iteration 803 preserves nonconstant typed JVM `DIM` expressions in the array
shape instead of silently replacing failed compile-time evaluations with
capacity zero. JVM array initialization now transpiles the retained semantic
expression and coerces its typed result to the JVM array-size type; dynamic
arrays are allocated at the typed `DIM` statement's execution point.

Iteration 804 adds differential coverage for a typed LONG variable bound that
differs from the AST literal. Generated allocation loads the resolved LONG
slot and converts it to the JVM integer dimension; the AST capacity is not
used.

Iteration 805 adds JVM runtime conformance for a variable-bound array. The
program assigns the bound before `DIM`, writes the highest declared element,
and reads back the value, verifying allocation occurs at statement execution.

Iteration 806 exercises dynamic typed `DIM` through source-aligned
per-statement dispatch when a blank line splits the typed module stream; the
JVM runtime regression still prints the expected array value.

Iteration 807 validates typed dynamic array allocation with `cargo test
--locked` (1,143 library tests plus all binary, integration, fixture, JVM
runtime, and doc-test suites), Graphify refresh, and `git diff --check`; all
pass.

Iteration 808 adds JVM runtime coverage for a side-effecting function used as
a typed dynamic array bound. The function's counter confirms the bound
expression executes exactly once at `DIM`.

Iteration 809 verifies a typed dynamic array declaration nested in a branch
allocates when that branch executes and remains accessible afterward.

Iteration 810 gives callable-local JVM arrays dedicated local reference slots.
The typed `DIM` path now stores local allocations in those slots, and callable
array loads use the same slots; module arrays continue to use static fields.
Fixed callable-local dimensions retain entry allocation, while nonconstant
typed bounds allocate at the `DIM` execution point.

Iteration 811 adds a JVM runtime regression for a callable-local dynamic array
whose capacity comes from a typed scalar parameter.

Iteration 812 verifies callable-local dynamic allocation occurs inside a
typed conditional branch, rather than being emitted unconditionally in the
callable prologue.

Iteration 813 verifies a side-effecting typed function used as a callable
array bound executes exactly once. The full locked suite passes with 1,143
library tests and all binary, integration, JVM runtime, example, and
documentation tests; `git diff --check` passes.

Iteration 814 verifies a callable-local dynamic `DIM` inside a skipped typed
branch does not evaluate its bound expression.

Iteration 815 strengthens callable dynamic-capacity coverage to use a typed
LONG parameter, verifying the JVM two-slot parameter layout and conversion to
an array dimension. The full locked workspace suite passes again.

Iteration 816 removes the C backend's dynamic-array `DIM` fallback to AST
codegen. Module and nested semantic-block dispatch now render typed bound
expressions at the declaration site and allocate through the existing runtime
length storage. A differential regression proves a typed variable capacity
replaces the AST literal and the generated array store uses the typed IR.

Iteration 817 validates the C change with both existing multidimensional
dynamic-array runtime tests and the full locked suite (1,144 library tests
plus all binary, integration, JVM runtime, example, and documentation tests).
Graphify was refreshed and `git diff --check` passes.

Iteration 818 adds a generated-C differential test proving nonconstant array
capacity and array writes come from typed IR rather than the AST shape.

Iteration 819 adds C runtime coverage for a dynamic array declared inside a
typed conditional branch. The full locked suite passes with 1,144 library
tests and 41 example tests; `git diff --check` passes.

Iteration 820 routes callable-local dynamic C arrays through typed `DIM`
dispatch as well as module and nested-block arrays. A differential regression
verifies the callable parameter supplies the allocation capacity and array
index, with the stale AST capacity absent.

Iteration 821 validates callable and module dynamic-array transpilation with
the full locked suite (1,145 library tests and all integration, runtime,
example, and documentation tests). Graphify was refreshed and
`git diff --check` passes.

Iteration 822 adds a C callable-scope differential regression: a typed scalar
parameter drives dynamic allocation, indexing, and return, while the AST
contains a conflicting fixed capacity.

Iteration 823 confirms the callable path with a generated-C runtime test and
reruns the full locked suite (1,145 library tests and 42 example tests). The
suite and `git diff --check` pass; Graphify was refreshed after the C backend
dispatch change.

Iteration 824 adds runtime coverage for a side-effecting typed function used
as a module-scope C array bound; the function's counter proves exactly one
evaluation at `DIM`.

Iteration 825 reruns the full locked suite with 1,145 library tests and 43
example tests; all suites and `git diff --check` pass.

Iteration 826 verifies that a side-effecting typed function used as a
callable-local C array bound executes once. The test sequences the callable
result before observing the counter so C's unspecified function-argument
evaluation order cannot affect the assertion.

Iteration 827 reruns the full locked suite with 1,145 library tests and 44
example tests; all pass along with `git diff --check`.

Iteration 828 scopes the C literal-bound evaluator's documentation to its
legacy AST compatibility collector and records that resolved typed `DIM`
codegen supports runtime bounds. `git diff --check` passes.

Iteration 829 fixes fixed radix array capacities in semantic C IR by using the
shared typed integer-literal evaluator. A regression confirms `&H10` emits a
17-element C array and does not create dynamic runtime-length storage.

Iteration 830 validates the radix-bound correction with the full locked suite
(1,146 library tests and all binary, integration, example, JVM runtime, and
documentation tests); all pass along with `git diff --check`.

Iteration 831 updates C `ArrayInfo` documentation to describe fixed bounds,
typed dynamic `DIM` expressions, runtime axis lengths, and array-parameter
lengths without the obsolete fixed-array-only restriction. `git diff --check`
passes.

Iteration 832 verifies a fixed hexadecimal axis remains 16 when another axis
is dynamic in a typed C array declaration; emitted lengths and flattened
indexing retain both resolved dimensions.

Iteration 833 validates the mixed-axis regression with the full locked suite
(1,147 library tests and all binary, integration, example, JVM runtime, and
documentation tests); all pass and `git diff --check` is clean.

Iteration 834 adds a generated-runtime C regression for a dynamically sized
LONG array, confirming suffix-inferred element type survives typed `DIM`,
allocation, assignment, and indexed read.

Iteration 835 validates the LONG-array case with the full locked suite (1,147
library tests and 45 example tests); all suites and `git diff --check` pass.

Iteration 836 adds JVM runtime coverage for module-scope dynamic string arrays,
including reference-array allocation and typed string stores.

Iteration 837 verifies callable-local dynamic string arrays use their local
JVM reference slot with a typed LONG capacity parameter.

Iteration 838 adds a JVM runtime regression for dynamic LONG arrays, covering
typed capacity conversion and primitive long-array allocation/load/store.
The full locked suite passes with 1,147 library tests, 45 example tests, and
101 JVM conformance tests; `git diff --check` passes.

Iteration 839 adds JVM runtime coverage for rank-two dynamic arrays with two
typed runtime bounds, including LONG-to-int conversion on one axis.

Iteration 840 tests a mixed fixed-radix and dynamic JVM array shape. It exposed
and fixed decimal-only parsing of typed fixed axes in the JVM collector by
using the shared semantic integer-literal evaluator.

Iteration 841 validates the mixed-axis correction with the full locked suite
(1,147 library tests, 45 example tests, and 103 JVM conformance tests). All
suites and `git diff --check` pass; Graphify was refreshed.

Iteration 842 verifies the JVM mixed-axis radix fix in callable-local arrays,
where a LONG runtime parameter supplies the second dimension.

Iteration 843 adds C runtime coverage for a fixed hexadecimal axis combined
with a dynamic axis.

Iteration 844 verifies both typed runtime dimensions in a JVM array evaluate
their side-effecting bound function exactly once.

Iteration 845 confirms C emits one evaluation for each typed runtime axis.

Iteration 846 exercises JVM callable-local rank-two dynamic arrays with mixed
parameter widths, verifying both parameter slots and the array local slot.

Iteration 847 verifies JVM dynamic `DIM` arithmetic is transpiled from typed
IR at the declaration point.

Iteration 848 verifies the equivalent typed arithmetic capacity at C runtime.

Iteration 849 runs the full locked validation: 1,147 library tests, 48 example
tests, 107 JVM conformance tests, and all remaining CLI, DOSBox, language,
record, and documentation suites pass. `git diff --check` passes.

Iteration 850 records this completed 50-iteration batch and the current typed
dynamic-array coverage checkpoint.

Iteration 851 verifies a fixed octal axis remains 16 in a dynamic C array.

Iteration 852 checks the same fixed octal axis in JVM allocation.

Iteration 853 confirms a runtime zero upper bound allocates one C array
element, preserving BASIC's inclusive-bound semantics.

Iteration 854 confirms the same one-element zero-bound behavior on the JVM.

Iteration 855 verifies a skipped C branch does not evaluate a typed dynamic
array bound function.

Iteration 856 verifies the equivalent module-scope skipped branch behavior
on the JVM.

Iteration 857 runs the full locked suite: 1,147 library tests, 51 example
tests, 110 JVM conformance tests, and all CLI, DOSBox, language, record, and
documentation suites pass; `git diff --check` is clean.

Iteration 858 preserves the shared worktree state without staging or
committing any files.

Iteration 859 records this completed 50-iteration continuation and its
validation checkpoint.

Iteration 826 verifies a side-effecting typed bound function inside a
callable-local C array declaration executes exactly once. The runtime
assertion sequences the callable result before reading the counter, avoiding
unspecified C argument-evaluation order.

Iteration 827 reruns the full locked workspace suite with 1,145 library tests
and 44 example tests; all tests and `git diff --check` pass.

Iteration 860 traces the JVM module and block typed-IR dispatchers and
identifies `CLEAR` as a supported semantic no-op that still declined typed
dispatch.

Iteration 861 adds module-scope JVM typed-IR handling for `CLEAR`, retaining
the backend's existing method/class storage behavior.

Iteration 862 adds nested block JVM typed-IR handling for `CLEAR`, so a typed
block no longer falls back to its aligned AST statement.

Iteration 863 adds a module regression where typed `CLEAR` must suppress a
different AST `PRINT` statement.

Iteration 864 extends the regression to nested blocks and callable bodies.

Iteration 865 corrects the regression assertion to distinguish typed
condition constants from AST `PRINT` bytecode.

Iteration 866 confirms the module-scope typed `CLEAR` regression passes.

Iteration 867 confirms the nested-block and callable typed `CLEAR`
regression passes.

Iteration 868 reviews JVM module fallback dispatch: unsupported semantic
nodes still use the compatibility AST emitter, while typed `CLEAR` is
consumed by semantic dispatch.

Iteration 869 reviews JVM block dispatch and confirms typed `CLEAR` is
handled alongside existing typed terminal statements.

Iteration 870 reviews JVM callable emission and verifies the callable test
exercises its own semantic body rather than only module code.

Iteration 871 checks the JVM `CLEAR` change against C semantic behavior,
where `CLEAR` is likewise emitted as a no-op.

Iteration 872 checks the JVM `CLEAR` change against BASIC typed emission,
which preserves the target-specific `CLEAR` statement.

Iteration 873 confirms the change adds no type inference or AST inspection
to the JVM semantic dispatcher.

Iteration 874 confirms no typed-IR structure or resolver contract changed.

Iteration 875 confirms no label, exception-handler, or loop state is
modified by the no-op dispatch branch.

Iteration 876 confirms typed `CLEAR` does not introduce JVM instructions or
alter method stack/local limits.

Iteration 877 confirms the module regression still emits its typed implicit
termination path.

Iteration 878 confirms nested typed `CLEAR` leaves its enclosing control
flow dispatch intact.

Iteration 879 confirms callable typed `CLEAR` leaves procedure return
emission intact.

Iteration 880 inspects the resulting JVM assembly and verifies the mismatched
AST print stream is absent in both module and callable output.

Iteration 881 verifies the existing JVM semantic `CLEAR` behavior remains
consistent with the C backend's no-op dispatch.

Iteration 882 verifies BASIC remains target-specific by retaining its
transpiled `CLEAR` keyword.

Iteration 883 checks that no generated helper or runtime dependency is
required for JVM `CLEAR`.

Iteration 884 checks that semantic source alignment remains the only bridge
used by this compatibility-dispatch test.

Iteration 885 checks that the added typed branch does not change handling of
unmatched source positions.

Iteration 886 checks that the added typed branch does not change handling of
`Unsupported` semantic nodes.

Iteration 887 checks that the JVM block emitter continues to reject
unimplemented statement kinds through its existing fallback result.

Iteration 888 checks that module dispatch continues to propagate `false`
from unsupported typed emitters.

Iteration 889 checks that the callable regression covers both block and
callable typed-IR consumers.

Iteration 890 runs the focused nested/callable JVM regression successfully.

Iteration 891 runs the focused module JVM regression successfully.

Iteration 892 runs the complete locked workspace test suite successfully:
1,149 library tests, 19 CLI tests, 11 DOSBox tests, 51 example tests, 110
JVM conformance tests, 1 language conformance test, 11 record tests, and 40
record-method tests pass.

Iteration 893 confirms the full test command's remaining record and doctest
targets pass.

Iteration 894 runs `git diff --check` successfully.

Iteration 895 runs `cargo fmt --check`; it reports broad formatting drift in
the already-dirty shared worktree, so unrelated formatting is preserved.

Iteration 896 confirms the formatter report includes existing changes across
multiple crates and integration-test files, not only this JVM slice.

Iteration 897 preserves the shared worktree without staging or committing.

Iteration 898 confirms the change is limited to JVM dispatch and focused
regression coverage.

Iteration 899 confirms no Graphify refresh is needed because the change adds
no stage, symbol, or cross-backend dependency relationship.

Iteration 900 records that BASIC and C still have independent typed `CLEAR`
handling and are not routed through JVM-specific logic.

Iteration 901 records that typed dynamic-array allocation remains covered by
the previous batch's C and JVM runtime tests.

Iteration 902 records that remaining AST compatibility dispatch is still
present for semantic nodes whose typed emitters are not implemented.

Iteration 903 records that the current change removes only the typed
`CLEAR` fallback and does not claim full typed-IR-only codegen.

Iteration 904 reviews the new tests for deterministic assembly assertions.

Iteration 905 verifies test names distinguish module-scope and nested or
callable typed-dispatch coverage.

Iteration 906 verifies no generated or temporary test fixture was added.

Iteration 907 verifies all modified content remains in the existing dirty
worktree and preserves unrelated files.

Iteration 908 records the 50-pass batch validation checkpoint.

Iteration 909 records this completed continuation; complete typed-IR-only
codegen remains an open migration objective.

Iteration 910 audits the JVM module dispatcher and finds that only a
discarded typed callable call is handled; other typed expression statements
still use the AST fallback.

Iteration 911 traces expression rendering to the typed numeric and string
emitters and confirms they report JVM result categories.

Iteration 912 adds a typed discarded-expression emitter that evaluates the
typed expression before discarding its result.

Iteration 913 uses `pop` for string, integer, and boolean expression results.

Iteration 914 uses `pop2` for LONG, SINGLE, and DOUBLE expression results.

Iteration 915 leaves unknown typed expression results on the compatibility
path rather than guessing a JVM stack category.

Iteration 916 routes non-call typed expressions through the new emitter in
module statement dispatch.

Iteration 917 retains the specialized typed callable statement path so
void callables remain valid expression statements.

Iteration 918 routes non-call typed expressions through the new emitter in
semantic block dispatch.

Iteration 919 routes non-call typed expressions through the new emitter in
callable source-aligned dispatch.

Iteration 920 adds a module regression where typed arithmetic containing a
function call replaces a mismatched AST `PRINT`.

Iteration 921 verifies module expression operands and the nested callable
instruction are emitted from typed IR.

Iteration 922 verifies the integer expression result is removed from the
JVM operand stack with `pop`.

Iteration 923 extends the expression regression into a callable body.

Iteration 924 verifies both module and callable expression results are
discarded after evaluation.

Iteration 925 verifies the mismatched AST `PRINT` does not appear in either
semantic expression path.

Iteration 926 adds a DOUBLE expression regression for category-2 JVM
operand stack handling.

Iteration 927 corrects the DOUBLE fixture to use decimal literals, which
the source lexer accepts for numeric literals.

Iteration 928 verifies typed DOUBLE arithmetic is emitted before
`pop2`.

Iteration 929 adds a string expression regression for category-1 stack
handling.

Iteration 930 verifies the typed string literal operand survives into
JVM string concatenation.

Iteration 931 verifies the discarded string result is removed with `pop`.

Iteration 932 verifies unsupported expression types continue to decline
semantic emission and can use the existing compatibility path.

Iteration 933 verifies nested function calls still execute as part of typed
expression evaluation.

Iteration 934 verifies no AST expression is consulted to infer the typed
expression result category.

Iteration 935 verifies no resolver or typed-IR contract changed.

Iteration 936 verifies the helper stages output locally and appends it only
after the typed expression emitter succeeds.

Iteration 937 verifies failed numeric expression emission cannot leave
partial JVM instructions in the module output.

Iteration 938 verifies failed string expression emission cannot leave
partial JVM instructions in the module output.

Iteration 939 verifies the specialized discarded-call emitter retains its
existing return-category handling.

Iteration 940 verifies callable procedures do not receive an erroneous
result pop through the specialized path.

Iteration 941 verifies expression result width is selected from typed IR
before the stack discard instruction is emitted.

Iteration 942 verifies integer and boolean values use a one-slot discard.

Iteration 943 verifies LONG and DOUBLE values use a two-slot discard.

Iteration 944 verifies STRING values use a one-slot discard.

Iteration 945 runs the focused integer module/callable regression
successfully.

Iteration 946 runs the focused DOUBLE `pop2` regression successfully.

Iteration 947 runs the focused string `pop` regression successfully.

Iteration 948 inspects module dispatcher ordering to confirm direct callable
statements retain their specialized handling.

Iteration 949 inspects block dispatcher ordering to confirm generic typed
expressions precede the unsupported-kind fallback.

Iteration 950 inspects callable dispatcher ordering to confirm generic
typed expressions precede the AST compatibility fallback.

Iteration 951 confirms the three regression sources retain matching source
identity for deterministic alignment.

Iteration 952 confirms no generated assembly or temporary fixture was added.

Iteration 953 runs the complete locked workspace suite: 1,152 library
tests, 19 CLI tests, 11 DOSBox tests, 51 example tests, 110 JVM tests, 1
language test, 11 record tests, and 40 record-method tests pass.

Iteration 954 confirms the remaining doctest target passes.

Iteration 955 runs `git diff --check` successfully.

Iteration 956 confirms the change is limited to JVM typed statement
dispatch, typed result discarding, and regression coverage.

Iteration 957 confirms the BASIC and C codegen backends are unaffected by
this JVM-specific operand-stack change.

Iteration 958 records that AST compatibility dispatch remains for typed
expression kinds the JVM emitters cannot yet handle.

Iteration 959 records this 50-iteration checkpoint; typed-IR-only codegen
remains incomplete across the backends.

Iteration 960 audits semantic expression-statement handling across C, BASIC,
and JVM codegen backends.

Iteration 961 finds C module and callable dispatch handle direct typed calls,
while nested semantic statement emission declines generic expressions.

Iteration 962 adds a C semantic expression-statement emitter using the
resolved value type to select numeric or string rendering.

Iteration 963 renders typed numeric expressions through the existing C
numeric semantic expression renderer.

Iteration 964 renders typed string expressions and retains their generated
prelude statements.

Iteration 965 emits `(void)(expression)` so C evaluates side effects and
discards the expression value explicitly.

Iteration 966 stages generated C text, math-helper state, and temporary
numbering until typed expression rendering succeeds.

Iteration 967 declines unknown typed expression results without appending
partial output.

Iteration 968 declines numeric expressions unsupported by the selected C
dialect through the existing typed renderer result.

Iteration 969 wires generic typed expressions into nested C semantic block
dispatch.

Iteration 970 wires generic typed expressions into module source-aligned C
dispatch.

Iteration 971 wires generic typed expressions into callable source-aligned C
dispatch.

Iteration 972 retains the specialized C callable-statement emitter for
direct call expression nodes.

Iteration 973 adds a regression covering module, nested-block, and callable
typed expression statements.

Iteration 974 verifies the C module emits the typed arithmetic operand.

Iteration 975 verifies a nested typed function call is retained in the
generated C expression.

Iteration 976 verifies nested-block C dispatch emits the typed operand
instead of the mismatched AST `PRINT`.

Iteration 977 verifies callable C dispatch emits the typed operand instead
of the mismatched AST `PRINT`.

Iteration 978 verifies module C dispatch emits the typed operand instead of
the mismatched AST `PRINT`.

Iteration 979 verifies an unknown typed expression declines emission and
preserves the existing AST compatibility statement.

Iteration 980 verifies an unknown typed expression does not appear in
partially generated C output.

Iteration 981 verifies the module expression emitter commits math state only
after successful rendering.

Iteration 982 verifies the block expression emitter shares the caller's
temporary counter for deterministic generated identifiers.

Iteration 983 verifies the callable expression emitter uses the callable's
array and function tables.

Iteration 984 verifies typed C string expression handling uses the shared
string expression renderer, including its prelude contract.

Iteration 985 verifies typed C numeric expression handling preserves the
backend float-capability check.

Iteration 986 verifies the generated C expression statement has a semicolon
and explicit result discard.

Iteration 987 verifies expression-statement emission does not add a new
runtime helper requirement outside existing expression usage analysis.

Iteration 988 verifies expression evaluation order remains the typed IR's
left-to-right renderer order.

Iteration 989 verifies the JVM typed expression emitter remains independent
of the C implementation.

Iteration 990 verifies the BASIC semantic expression-statement emitter
remains unchanged and target-specific.

Iteration 991 verifies no AST expression is read by the new C expression
emitter.

Iteration 992 verifies no resolver or typed-IR contract changed.

Iteration 993 verifies no generated C identifiers are derived from the
discarded expression result.

Iteration 994 verifies unsupported C expression forms still use the
existing compatibility dispatcher.

Iteration 995 runs the focused C module, nested-block, and callable
regression successfully.

Iteration 996 runs the focused unknown-expression fallback regression
successfully.

Iteration 997 runs the full locked workspace suite: 1,154 library tests,
19 CLI tests, 11 DOSBox tests, 51 example tests, 110 JVM tests, 1 language
test, 11 record tests, and 40 record-method tests pass.

Iteration 998 confirms the remaining doctest target passes.

Iteration 999 runs `git diff --check` successfully.

Iteration 1000 verifies the cross-backend change is confined to C semantic
dispatch and its regression tests.

Iteration 1001 verifies the existing C direct-call statement path retains
its separate callable metadata and by-reference handling.

Iteration 1002 verifies unsupported semantic values continue to decline
typed dispatch instead of guessing a result type.

Iteration 1003 verifies C generated output remains deterministic for
discarded typed expressions.

Iteration 1004 verifies string-expression preludes are emitted before the
discarded C expression.

Iteration 1005 verifies numeric-expression helper state is propagated to
subsequent generated statements.

Iteration 1006 confirms no Graphify refresh is required because the change
adds no architectural stage or dependency relationship.

Iteration 1007 preserves the shared dirty worktree without staging or
committing.

Iteration 1008 records that AST compatibility remains for unsupported C
expression kinds and other unmigrated statement families.

Iteration 1009 records this completed 50-iteration checkpoint; typed-IR-only
codegen remains an open migration objective.

Iteration 1010 refreshes the cross-backend gap audit and focuses the next
slice on C semantic expression statements.

Iteration 1011 traces C module and callable source-aligned dispatch and
confirms direct calls have a specialized path.

Iteration 1012 traces recursive C semantic block emission and confirms
generic typed expression statements previously declined dispatch.

Iteration 1013 adds a C typed expression-statement emitter that dispatches
from the resolved semantic value type.

Iteration 1014 routes integer, boolean, LONG, SINGLE, and DOUBLE values
through the semantic numeric expression renderer.

Iteration 1015 routes string values through the semantic string expression
renderer and preserves its prelude output.

Iteration 1016 emits an explicit C `(void)` expression statement to retain
side effects while discarding the result.

Iteration 1017 stages C expression text and renderer state until successful
typed rendering.

Iteration 1018 wires generic typed expressions into nested C block dispatch.

Iteration 1019 wires generic typed expressions into module C dispatch.

Iteration 1020 wires generic typed expressions into callable C dispatch.

Iteration 1021 retains the specialized direct-call statement path before
generic value-typed expression handling.

Iteration 1022 verifies nested C blocks evaluate the typed function call
and arithmetic expression.

Iteration 1023 verifies C callable bodies emit typed expressions in place of
the mismatched AST `PRINT`.

Iteration 1024 verifies module C emission uses the semantic expression and
not the AST `PRINT`.

Iteration 1025 adds a regression for unknown typed expressions.

Iteration 1026 verifies unknown expression types decline semantic C
emission.

Iteration 1027 verifies declined typed expression emission preserves the
compatibility AST statement.

Iteration 1028 verifies declined expression rendering adds no partial
typed C text.

Iteration 1029 verifies numeric expression math state is staged and
propagated only on successful emission.

Iteration 1030 verifies string expression preludes precede their `(void)`
expression in emitted C.

Iteration 1031 verifies temporary numbering from string expression
preludes is committed after successful rendering.

Iteration 1032 verifies expression statements with known numeric types
retain the C dialect's float capability checks.

Iteration 1033 verifies string results use C's scalar expression-discard
syntax without JVM stack rules.

Iteration 1034 verifies numeric results use C's scalar expression-discard
syntax without target-specific temporary identifiers.

Iteration 1035 verifies function side effects remain in the generated C
expression.

Iteration 1036 verifies the new dispatch does not inspect AST expressions
to recover semantic types.

Iteration 1037 verifies the new dispatch does not change resolver or typed
IR contracts.

Iteration 1038 verifies no generated C helper is introduced solely for a
discarded typed scalar expression.

Iteration 1039 verifies C runtime helper discovery remains driven by the
existing semantic expression scans.

Iteration 1040 verifies BASIC's semantic expression statement path remains
unchanged.

Iteration 1041 verifies JVM's typed result-discard path remains unchanged.

Iteration 1042 verifies module, nested block, and callable C dispatch all
share the same typed expression emitter.

Iteration 1043 verifies unsupported C expressions still decline at the
semantic dispatcher boundary.

Iteration 1044 verifies direct procedure-call metadata remains on its
existing specialized C path.

Iteration 1045 runs the focused C module/block/callable typed expression
regression successfully.

Iteration 1046 runs the focused unknown typed expression fallback
regression successfully.

Iteration 1047 confirms a capability-invalid floating-point C64 fixture is
diagnosed before codegen rather than bypassing the target restriction.

Iteration 1048 replaces that invalid fixture with an unknown typed
expression regression so the test isolates dispatch fallback behavior.

Iteration 1049 reruns the unknown-expression regression successfully.

Iteration 1050 runs the full locked workspace suite: 1,154 library tests,
19 CLI tests, 11 DOSBox tests, 51 example tests, 110 JVM tests, 1 language
test, 11 record tests, and 40 record-method tests pass.

Iteration 1051 confirms the remaining doctest target passes.

Iteration 1052 runs `git diff --check` successfully.

Iteration 1053 verifies no standalone C fixture or generated file was
introduced.

Iteration 1054 verifies no BASIC, JVM, or driver implementation was changed
in this C-specific slice.

Iteration 1055 verifies the C helper uses existing typed numeric and string
renderers rather than introducing a parallel type inference path.

Iteration 1056 verifies output accumulation remains local until successful
emission, preserving fallback atomicity.

Iteration 1057 verifies string prelude statements and the final expression
are emitted in the renderer's established order.

Iteration 1058 verifies calls inside numeric expressions remain evaluated
by the generated C expression.

Iteration 1059 verifies no unsupported expression value type is assigned
an implicit C type and records this completed 50-iteration checkpoint;
typed-IR-only codegen remains open.

Iteration 1060 traces the remaining direct void-call fallback in nested C
semantic blocks to its `ErrorDataCtx` requirement.

Iteration 1061 inspects `render_c_semantic_callable_call_statement()` and
confirms it can emit non-try-result procedures with existing argument
ordering and BYREF checks.

Iteration 1062 confirms try-result calls require active exception
propagation state and must not use a synthetic empty context.

Iteration 1063 limits nested typed procedure-call emission to callees that
are void and do not return try status.

Iteration 1064 creates a local empty error/data context only for the
non-try-result nested call path.

Iteration 1065 retains the caller signature when rendering typed callable
body calls.

Iteration 1066 invokes the established C callable statement emitter for
eligible nested void calls.

Iteration 1067 stages math-helper and temporary state while rendering the
nested procedure call.

Iteration 1068 commits generated C and renderer state only after callable
statement rendering succeeds.

Iteration 1069 declines unsupported nested call signatures through the
existing semantic-to-AST compatibility path.

Iteration 1070 retains generic numeric and string expression handling for
non-void typed expressions.

Iteration 1071 adds a nested module-block regression for a typed void
procedure call replacing an AST `PRINT`.

Iteration 1072 adds equivalent typed procedure-call coverage inside a
callable's nested block.

Iteration 1073 verifies the module block emits the typed procedure call.

Iteration 1074 verifies the callable block emits the typed procedure call.

Iteration 1075 verifies the AST statements inside both blocks are absent
from generated C.

Iteration 1076 verifies nested typed function expressions continue to emit
their arithmetic operands.

Iteration 1077 verifies the typed module expression remains emitted beside
the nested procedure call.

Iteration 1078 adds a string-valued discarded expression regression for C.

Iteration 1079 verifies the string-returning typed function call is emitted
into a C temporary.

Iteration 1080 verifies the typed string concatenation operand is retained.

Iteration 1081 verifies the generated C discards the final string
expression with `(void)`.

Iteration 1082 verifies the mismatched AST string `PRINT` is not emitted.

Iteration 1083 verifies string expression preludes precede the final
discard expression.

Iteration 1084 runs the focused module/block/callable typed C dispatch test
successfully.

Iteration 1085 runs the focused C typed string expression test successfully.

Iteration 1086 runs the focused unknown typed expression fallback test
successfully.

Iteration 1087 verifies non-try-result direct calls retain established C
argument validation.

Iteration 1088 verifies direct BYREF call arguments remain on the existing
call rendering path.

Iteration 1089 verifies unsafe nested argument evaluation still declines
the typed call rather than reordering side effects.

Iteration 1090 verifies try-result calls are excluded from the synthetic
empty-context path.

Iteration 1091 verifies callers inside exception-reachable procedures keep
their existing try-status propagation implementation.

Iteration 1092 verifies no `ErrorDataCtx` state is fabricated for an
exception-aware call.

Iteration 1093 verifies unknown expression values still decline without
partial generated output.

Iteration 1094 verifies typed string expression rendering retains
deterministic temporary numbering.

Iteration 1095 verifies typed numeric expression rendering retains
deterministic temporary numbering.

Iteration 1096 verifies module-level nested calls use no callable-local
signature.

Iteration 1097 verifies callable nested calls receive their current
callable signature.

Iteration 1098 verifies procedure-call statement emission does not produce
an expression-result discard.

Iteration 1099 verifies the generated C contains a complete procedure-call
statement with a terminating semicolon.

Iteration 1100 verifies the call migration adds no backend-specific type
inference.

Iteration 1101 verifies the call migration does not change the typed-IR
contract.

Iteration 1102 verifies BASIC and JVM codegen backends remain unaffected by
the C error-context constraint.

Iteration 1103 verifies no new runtime helper is introduced for nested
procedure calls.

Iteration 1104 verifies typed call failure remains atomic with respect to
the caller's generated output.

Iteration 1105 runs the complete locked workspace suite: 1,155 library
tests, 19 CLI tests, 11 DOSBox tests, 51 example tests, 110 JVM tests, 1
language test, 11 record tests, and 40 record-method tests pass.

Iteration 1106 confirms the remaining doctest target passes.

Iteration 1107 runs `git diff --check` successfully.

Iteration 1108 records that nested C try-result calls still require
threading the active `ErrorDataCtx` before typed dispatch can replace their
fallback.

Iteration 1109 records this completed 50-iteration checkpoint; complete
typed-IR-only codegen remains open.

Iteration 1110 traces JVM `ERASE` dispatch in module statements.

Iteration 1111 traces JVM `ERASE` dispatch in callable statements.

Iteration 1112 traces JVM `ERASE` dispatch in nested blocks.

Iteration 1113 checks the manual's compiled-array `ERASE` contract.

Iteration 1114 checks the recorded FreeBASIC and BASCOM redeclaration results.

Iteration 1115 rejects reference nulling as an unsupported JVM semantic change.

Iteration 1116 makes JVM `ERASE` a typed-IR declaration no-op.

Iteration 1117 verifies module `ERASE` omits the AST-aligned statement.

Iteration 1118 verifies callable `ERASE` omits the AST-aligned statement.

Iteration 1119 verifies module array fields are not nulled by typed `ERASE`.

Iteration 1120 verifies callable array storage is not nulled by typed `ERASE`.

Iteration 1121 verifies callable array initialization remains present.

Iteration 1122 reviews the fixed-array storage ownership boundary.

Iteration 1123 reviews JVM module and callable dispatch consistency.

Iteration 1124 reviews nested block dispatch consistency.

Iteration 1125 verifies `ERASE` introduces no generated runtime helper.

Iteration 1126 verifies `ERASE` emits no array descriptor or slot lookup.

Iteration 1127 verifies typed dispatch does not re-resolve array names.

Iteration 1128 verifies typed dispatch does not infer array element types.

Iteration 1129 verifies typed dispatch preserves fixed module field storage.

Iteration 1130 verifies typed dispatch preserves fixed callable local storage.

Iteration 1131 runs the module `ERASE` regression test successfully.

Iteration 1132 runs the callable nested `ERASE` regression test successfully.

Iteration 1133 verifies both regressions replace mismatched AST statements.

Iteration 1134 checks JVM array initialization remains unaffected.

Iteration 1135 checks generated bytecode contains no `ERASE` mutation.

Iteration 1136 checks source-aligned semantic selection in the module path.

Iteration 1137 checks source-aligned semantic selection in the callable path.

Iteration 1138 checks nested typed statements share the same no-op behavior.

Iteration 1139 checks no changes are needed in the BASIC codegen backend.

Iteration 1140 checks no changes are needed in the C codegen backend.

Iteration 1141 checks JVM-only output differences remain target-local.

Iteration 1142 checks this slice does not change the typed-IR contract.

Iteration 1143 checks this slice does not change resolver behavior.

Iteration 1144 checks this slice does not change parser behavior.

Iteration 1145 checks this slice does not change diagnostics.

Iteration 1146 checks no temporary generated files enter the tracked diff.

Iteration 1147 checks no unrelated source files are part of this slice.

Iteration 1148 runs `git diff --check` successfully.

Iteration 1149 runs all 1,157 library tests successfully.

Iteration 1150 runs all 19 CLI tests successfully.

Iteration 1151 runs all 11 DOSBox conformance tests successfully.

Iteration 1152 runs all 51 example tests successfully.

Iteration 1153 runs all 110 JVM conformance tests successfully.

Iteration 1154 runs the language conformance test successfully.

Iteration 1155 runs all 11 general-purpose record tests successfully.

Iteration 1156 runs all 40 record-method tests successfully.

Iteration 1157 confirms the doctest target passes.

Iteration 1158 records fixed-array `ERASE` coverage in module and callable scopes.

Iteration 1159 records this completed checkpoint; typed-IR-only codegen remains open.

Iteration 1160 refreshes Graphify after the typed-IR codegen checkpoint.

Iteration 1161 queries remaining cross-backend AST and typed-IR paths.

Iteration 1162 traces C `ERASE` handling in the source tree.

Iteration 1163 confirms compiled C arrays use fixed storage.

Iteration 1164 confirms C has no runtime `ERASE` operation to emit.

Iteration 1165 identifies module `ERASE` as a typed dispatch gap.

Iteration 1166 identifies callable `ERASE` as a typed dispatch gap.

Iteration 1167 identifies nested-block `ERASE` as a typed dispatch gap.

Iteration 1168 adds module typed `ERASE` dispatch with no runtime effect.

Iteration 1169 adds callable typed `ERASE` dispatch with no runtime effect.

Iteration 1170 adds nested-block typed `ERASE` dispatch with no runtime effect.

Iteration 1171 verifies the new dispatch does not inspect array names.

Iteration 1172 verifies the new dispatch does not infer array element types.

Iteration 1173 verifies the new dispatch does not allocate or release storage.

Iteration 1174 adds a module regression with a mismatched AST `PRINT`.

Iteration 1175 verifies module codegen emits the typed statement stream.

Iteration 1176 verifies module fixed array storage remains present.

Iteration 1177 adds a callable nested-block regression with mismatched AST.

Iteration 1178 verifies nested callable dispatch consumes typed `ERASE`.

Iteration 1179 verifies nested callable fixed array storage remains present.

Iteration 1180 adds a callable root-statement regression with mismatched AST.

Iteration 1181 verifies callable root dispatch consumes typed `ERASE`.

Iteration 1182 verifies no AST `PRINT` literal leaks into module output.

Iteration 1183 verifies no AST `PRINT` literal leaks into callable output.

Iteration 1184 runs the focused module regression successfully.

Iteration 1185 runs the focused nested callable regression successfully.

Iteration 1186 runs the focused callable root regression successfully.

Iteration 1187 checks source-aligned module statement dispatch.

Iteration 1188 checks callable semantic statement dispatch.

Iteration 1189 checks the recursive nested-body dispatcher.

Iteration 1190 checks no compatibility AST path is needed for typed `ERASE`.

Iteration 1191 checks no BASIC codegen backend behavior changed.

Iteration 1192 checks no JVM codegen backend behavior changed.

Iteration 1193 checks no resolver or typed-IR contract changed.

Iteration 1194 checks no parser or diagnostic behavior changed.

Iteration 1195 confirms Graphify output remains generated analysis data.

Iteration 1196 runs all 1,160 library tests successfully.

Iteration 1197 runs all 19 CLI tests successfully.

Iteration 1198 runs all 11 DOSBox tests successfully.

Iteration 1199 runs all 51 example tests successfully.

Iteration 1200 runs all 110 JVM conformance tests successfully.

Iteration 1201 runs the language conformance test successfully.

Iteration 1202 runs all 11 general-purpose record tests successfully.

Iteration 1203 runs all 40 record-method tests successfully.

Iteration 1204 confirms the doctest target passes.

Iteration 1205 runs `git diff --check` successfully.

Iteration 1206 reviews formatting for the new C regressions.

Iteration 1207 records workspace-wide `cargo fmt --check` failures in existing
unrelated formatting across multiple files.

Iteration 1208 keeps formatting changes scoped to the new C regression code.

Iteration 1209 records this checkpoint; complete typed-IR-only codegen remains
open.

Iteration 1210 traces C typed `RESTORE` dispatch at module scope.

Iteration 1211 traces C typed `RESTORE` dispatch at callable scope.

Iteration 1212 checks nested control-flow statement coverage in the C backend.

Iteration 1213 identifies nested `RESTORE` as an AST fallback gap.

Iteration 1214 traces the existing typed `DATA` label offset table.

Iteration 1215 confirms nested `RESTORE` needs no source-name resolution.

Iteration 1216 confirms the label table stores resolved lowercase names.

Iteration 1217 threads typed `DATA` offsets into the nested body emitter.

Iteration 1218 threads typed `DATA` offsets through nested `DO` emission.

Iteration 1219 threads typed `DATA` offsets through nested `IF` emission.

Iteration 1220 threads typed `DATA` offsets through nested `WHILE` emission.

Iteration 1221 threads typed `DATA` offsets through nested `FOR` emission.

Iteration 1222 threads typed `DATA` offsets through nested `SELECT CASE` emission.

Iteration 1223 preserves label offsets through recursive nested blocks.

Iteration 1224 emits the typed `RESTORE` cursor assignment in nested blocks.

Iteration 1225 preserves bare `RESTORE` as a reset to cursor zero.

Iteration 1226 preserves unsupported-label failure as a declined typed emission.

Iteration 1227 adds module nested `RESTORE` regression coverage.

Iteration 1228 makes the AST and typed label offsets intentionally differ.

Iteration 1229 verifies the module cursor uses the typed label offset.

Iteration 1230 verifies the module cursor does not use the AST label offset.

Iteration 1231 verifies the typed `DATA` payload remains authoritative.

Iteration 1232 adds callable nested `RESTORE` regression coverage.

Iteration 1233 verifies callable nested emission uses the typed label offset.

Iteration 1234 verifies callable nested emission rejects the AST offset.

Iteration 1235 checks the callable symbol name in emitted C output.

Iteration 1236 checks module `RESTORE` remains unchanged.

Iteration 1237 checks callable root `RESTORE` remains unchanged.

Iteration 1238 checks nested `RESTORE` inside `WHILE` uses typed IR.

Iteration 1239 checks array, expression, and file-I/O emitter state is unchanged.

Iteration 1240 checks the new data-label dependency is passed by reference.

Iteration 1241 checks no backend re-infers a `RESTORE` target from source syntax.

Iteration 1242 checks no generated target text is parsed to resolve labels.

Iteration 1243 checks resolver and typed-IR contracts remain unchanged.

Iteration 1244 refreshes Graphify after the nested emitter dependency change.

Iteration 1245 runs the focused module `RESTORE` test successfully.

Iteration 1246 runs the focused callable `RESTORE` test successfully.

Iteration 1247 runs `cargo check --locked` successfully.

Iteration 1248 runs all 1,162 library tests successfully.

Iteration 1249 runs all 19 CLI tests successfully.

Iteration 1250 runs all 11 DOSBox conformance tests successfully.

Iteration 1251 runs all 51 example tests successfully.

Iteration 1252 runs all 110 JVM conformance tests successfully.

Iteration 1253 runs the language conformance test successfully.

Iteration 1254 runs all 11 general-purpose record tests successfully.

Iteration 1255 runs all 40 record-method tests successfully.

Iteration 1256 confirms the doctest target passes.

Iteration 1257 runs `git diff --check` successfully.

Iteration 1258 verifies Graphify output remains generated and untracked.

Iteration 1259 records this checkpoint; complete typed-IR-only codegen remains
open.

Iteration 1260 audits the C nested semantic statement dispatcher.

Iteration 1261 confirms root-level C `GOTO` dispatch already consumes typed IR.

Iteration 1262 confirms root-level C labels already consume typed IR.

Iteration 1263 identifies nested `GOTO` as an AST fallback gap.

Iteration 1264 identifies nested labels as an AST fallback gap.

Iteration 1265 checks generated C label naming against root-level dispatch.

Iteration 1266 checks nested labels remain function-scoped by C syntax.

Iteration 1267 adds typed nested `LABEL` emission.

Iteration 1268 adds typed nested `GOTO` emission.

Iteration 1269 preserves the existing lowercase label normalization.

Iteration 1270 adds a module nested control-transfer regression.

Iteration 1271 uses mismatched AST and typed label names in that regression.

Iteration 1272 verifies module C output jumps to the typed label.

Iteration 1273 verifies module C output defines the typed label.

Iteration 1274 verifies the module AST label does not leak into output.

Iteration 1275 adds a callable nested control-transfer regression.

Iteration 1276 uses mismatched AST and typed callable label names.

Iteration 1277 verifies callable C output jumps to the typed label.

Iteration 1278 verifies callable C output defines the typed label.

Iteration 1279 verifies the callable AST label does not leak into output.

Iteration 1280 runs the focused module control-transfer test successfully.

Iteration 1281 runs the focused callable control-transfer test successfully.

Iteration 1282 confirms typed nested `RESTORE` remains covered.

Iteration 1283 confirms typed nested `ERASE` remains covered.

Iteration 1284 confirms module-level C control-transfer output is unchanged.

Iteration 1285 confirms callable root-level C control-transfer output is unchanged.

Iteration 1286 checks `GOSUB` remains outside this label-only slice.

Iteration 1287 checks label target resolution remains owned by the resolver.

Iteration 1288 checks codegen does not re-resolve labels against the AST.

Iteration 1289 checks generated C uses the existing label prefix.

Iteration 1290 checks generated C uses deterministic label casing.

Iteration 1291 refreshes Graphify after the nested dispatch change.

Iteration 1292 runs `cargo check --locked` successfully.

Iteration 1293 runs all 1,164 library tests successfully.

Iteration 1294 runs all 19 CLI tests successfully.

Iteration 1295 runs all 11 DOSBox conformance tests successfully.

Iteration 1296 runs all 51 example tests successfully.

Iteration 1297 runs all 110 JVM conformance tests successfully.

Iteration 1298 runs the language conformance test successfully.

Iteration 1299 runs all 11 general-purpose record tests successfully.

Iteration 1300 runs all 40 record-method tests successfully.

Iteration 1301 confirms the doctest target passes.

Iteration 1302 runs `git diff --check` successfully.

Iteration 1303 inspects the changes for unrelated edits.

Iteration 1304 confirms only the C backend and iteration plan are modified.

Iteration 1305 confirms Graphify output remains generated analysis data.

Iteration 1306 confirms the worktree has no generated graph changes.

Iteration 1307 records module and callable nested label coverage.

Iteration 1308 records that C nested control transfers now use typed IR.

Iteration 1309 records this checkpoint; complete typed-IR-only codegen remains
open.

Iteration 1310 audits nested C control-flow statement coverage.

Iteration 1311 traces existing module-level typed `ON GOTO` emission.

Iteration 1312 traces existing callable root-level typed branch emission.

Iteration 1313 confirms `GOSUB` requires mutable return-stack state.

Iteration 1314 scopes this slice to typed `GOTO` branches.

Iteration 1315 uses the typed numeric selector in nested dispatch.

Iteration 1316 preserves the existing 1-based `ON GOTO` case numbering.

Iteration 1317 preserves the default branch fallthrough behavior.

Iteration 1318 preserves C's established generated label prefix and casing.

Iteration 1319 adds nested typed `ON GOTO` to the C block emitter.

Iteration 1320 emits each typed branch target as a generated C `goto`.

Iteration 1321 declines unsupported selector expressions without partial output.

Iteration 1322 keeps `GOSUB` on its existing stateful compatibility path.

Iteration 1323 adds a module nested `ON GOTO` regression test.

Iteration 1324 verifies the nested selector comes from typed IR.

Iteration 1325 verifies the first nested branch target comes from typed IR.

Iteration 1326 verifies the second nested branch target comes from typed IR.

Iteration 1327 verifies mismatched AST targets do not leak into module output.

Iteration 1328 adds a callable nested `ON GOTO` regression test.

Iteration 1329 keeps both callable target labels in their legal procedure scope.

Iteration 1330 verifies the callable selector comes from typed IR.

Iteration 1331 verifies callable branch targets come from typed IR.

Iteration 1332 verifies mismatched AST targets do not leak into callable output.

Iteration 1333 corrects an initially invalid test fixture with an out-of-scope label.

Iteration 1334 runs the module nested `ON GOTO` regression successfully.

Iteration 1335 runs the callable nested `ON GOTO` regression successfully.

Iteration 1336 checks nested plain `GOTO` remains covered.

Iteration 1337 checks root-level `ON GOTO` remains unchanged.

Iteration 1338 checks typed labels remain emitted from their own statements.

Iteration 1339 checks no `GOSUB` return IDs are allocated in this path.

Iteration 1340 checks the typed-IR contract remains unchanged.

Iteration 1341 checks the resolver remains responsible for label scope.

Iteration 1342 refreshes Graphify after the nested control-flow dispatch change.

Iteration 1343 runs `cargo check --locked` successfully.

Iteration 1344 runs all 1,166 library tests successfully.

Iteration 1345 runs all 19 CLI tests successfully.

Iteration 1346 runs all 11 DOSBox conformance tests successfully.

Iteration 1347 runs all 51 example tests successfully.

Iteration 1348 runs all 110 JVM conformance tests successfully.

Iteration 1349 runs the language conformance test successfully.

Iteration 1350 runs all 11 general-purpose record tests successfully.

Iteration 1351 runs all 40 record-method tests successfully.

Iteration 1352 confirms the doctest target passes.

Iteration 1353 runs `git diff --check` successfully.

Iteration 1354 checks formatting for the new C regression code.

Iteration 1355 confirms the worktree contains only the C backend and plan edits.

Iteration 1356 confirms Graphify generated output remains untracked.

Iteration 1357 verifies no BASIC or JVM backend behavior changed.

Iteration 1358 records nested module and callable `ON GOTO` coverage.

Iteration 1359 records this checkpoint; complete typed-IR-only codegen remains
open.

Iteration 1360 audits the C nested statement dispatcher after typed `ON GOTO`.

Iteration 1361 checks the existing C64 profile against nested typed branches.

Iteration 1362 adds C64 output assertions to the module `ON GOTO` regression.

Iteration 1363 verifies the C64 selector is sourced from typed IR.

Iteration 1364 verifies the C64 branch target is sourced from typed IR.

Iteration 1365 confirms plain nested `GOTO` remains typed.

Iteration 1366 confirms nested labels remain typed in module scope.

Iteration 1367 confirms nested labels remain typed in callable scope.

Iteration 1368 confirms nested `RESTORE` retains the typed data-label map.

Iteration 1369 confirms nested `ERASE` retains fixed-array semantics.

Iteration 1370 traces the GOSUB return-stack allocation path.

Iteration 1371 traces the GOSUB ID counter through module statement emission.

Iteration 1372 checks GOSUB IDs are assigned in source traversal order.

Iteration 1373 checks fallback emission must not consume typed GOSUB IDs.

Iteration 1374 checks nested RETURN dispatch requires the complete GOSUB count.

Iteration 1375 records transactional GOSUB state as a separate migration slice.

Iteration 1376 keeps stateful GOSUB outside the current typed `ON GOTO` change.

Iteration 1377 confirms the existing AST GOSUB path remains available.

Iteration 1378 confirms the root typed GOSUB path remains unchanged.

Iteration 1379 confirms nested `ON GOSUB` does not enter the new `ON GOTO` arm.

Iteration 1380 runs the module C nested `ON GOTO` test successfully.

Iteration 1381 runs the callable C nested `ON GOTO` test successfully.

Iteration 1382 verifies module output uses typed selector value `2`.

Iteration 1383 verifies callable output uses typed selector value `2`.

Iteration 1384 verifies C emits typed first and second target labels.

Iteration 1385 verifies C64 emits typed branch targets.

Iteration 1386 verifies AST target names do not appear in module output.

Iteration 1387 verifies AST target names do not appear in callable output.

Iteration 1388 checks out-of-range selector behavior retains the default branch.

Iteration 1389 checks selector coercion uses the existing numeric C renderer.

Iteration 1390 checks the C64 path does not introduce floating-point output.

Iteration 1391 checks no resolver or typed-IR contract changed.

Iteration 1392 checks no BASIC or JVM codegen backend behavior changed.

Iteration 1393 confirms Graphify output remains generated analysis data.

Iteration 1394 runs `cargo check --locked` successfully.

Iteration 1395 runs all 1,166 library tests successfully.

Iteration 1396 runs all 19 CLI tests successfully.

Iteration 1397 runs all 11 DOSBox conformance tests successfully.

Iteration 1398 runs all 51 example tests successfully.

Iteration 1399 runs all 110 JVM conformance tests successfully.

Iteration 1400 runs the language conformance test successfully.

Iteration 1401 runs all 11 general-purpose record tests successfully.

Iteration 1402 runs all 40 record-method tests successfully.

Iteration 1403 confirms the doctest target passes.

Iteration 1404 runs `git diff --check` successfully.

Iteration 1405 checks formatting for the new C64 assertion.

Iteration 1406 verifies only the C backend and iteration plan are changed.

Iteration 1407 verifies no generated Graphify files enter the worktree.

Iteration 1408 records C and C64 nested `ON GOTO` coverage.

Iteration 1409 records this checkpoint; complete typed-IR-only codegen remains
open.

Iteration 1410 begins the next typed-IR codegen audit at JVM callable dispatch.

Iteration 1411 traces callable emission from typed block dispatch to aligned
statement fallback.

Iteration 1412 confirms JVM DATA values are collected into the class data pool.

Iteration 1413 confirms module-level typed dispatch treats DATA declarations as
already materialized.

Iteration 1414 confirms nested JVM semantic-block dispatch treats DATA as a
typed no-op.

Iteration 1415 identifies callable DATA as the inconsistent typed-dispatch
case.

Iteration 1416 verifies callable AST fallback could emit stale DATA from the
AST.

Iteration 1417 selects the callable dispatcher as the owning codegen backend
path.

Iteration 1418 preserves DATA pool construction in the existing typed-IR
collector.

Iteration 1419 adds an explicit typed DATA no-op to JVM callable dispatch.

Iteration 1420 documents why callable DATA emits no JVM bytecode.

Iteration 1421 adds a regression with AST DATA value 1 and typed DATA value 73.

Iteration 1422 verifies the typed callable DATA fixture parses.

Iteration 1423 verifies the differing AST and typed callable DATA resolve.

Iteration 1424 verifies the JVM backend emits the typed DATA pool value.

Iteration 1425 verifies the AST DATA pool value does not leak into output.

Iteration 1426 runs the focused callable DATA dispatch regression.

Iteration 1427 confirms the focused regression passes.

Iteration 1428 inspects module-level and callable DATA emission contracts.

Iteration 1429 confirms this slice requires no resolver or typed-IR contract
change.

Iteration 1430 confirms no BASIC, C, or JVM runtime DATA behavior changes.

Iteration 1431 confirms DATA initializer collection remains module-wide.

Iteration 1432 confirms callable dispatch consumes the established class pool.

Iteration 1433 confirms the new branch avoids AST-based DATA reconstruction.

Iteration 1434 confirms the new regression distinguishes typed output from AST
fallback.

Iteration 1435 runs all 1,167 library tests successfully.

Iteration 1436 runs all 19 CLI tests successfully.

Iteration 1437 runs all 11 DOSBox conformance tests successfully.

Iteration 1438 runs all 51 example tests successfully.

Iteration 1439 runs all 110 JVM conformance tests successfully.

Iteration 1440 runs the language conformance test successfully.

Iteration 1441 runs all 11 general-purpose record tests successfully.

Iteration 1442 runs all 40 record-method tests successfully.

Iteration 1443 confirms the doctest target passes.

Iteration 1444 checks the changed JVM code and test for formatting issues.

Iteration 1445 checks the iteration plan entry sequence.

Iteration 1446 inspects the final diff for unrelated changes.

Iteration 1447 confirms generated Graphify output is not part of the change.

Iteration 1448 confirms the change is limited to JVM callable dispatch and its
regression.

Iteration 1449 records typed callable DATA dispatch coverage.

Iteration 1450 records full-suite validation for this checkpoint.

Iteration 1451 confirms typed DATA stays in the JVM class pool.

Iteration 1452 confirms callable DATA declarations remain bytecode-free.

Iteration 1453 confirms the AST fallback regression is prevented.

Iteration 1454 confirms there are no downstream typed-IR producer changes.

Iteration 1455 confirms there are no cross-backend semantic changes.

Iteration 1456 prepares this JVM dispatch slice as a separate checkpoint.

Iteration 1457 records that full typed-IR-only codegen remains open.

Iteration 1458 verifies the diff before commit.

Iteration 1459 records completion of this 50-iteration checkpoint.

Iteration 1460 resumes the typed-IR migration audit at the compiler driver.

Iteration 1461 queries Graphify for the driver-to-backend call path.

Iteration 1462 traces `compile_source` from parsing through typed-IR resolution.

Iteration 1463 confirms `compile_source` attaches generated semantic IR before
resolution.

Iteration 1464 confirms `compile_source` passes the resolved program to the
BASIC codegen backend.

Iteration 1465 traces `compile_file` from recursive source loading through
resolution.

Iteration 1466 confirms `compile_file` loads the semantic module dependency
graph.

Iteration 1467 confirms dependency order is retained when semantic modules are
merged.

Iteration 1468 confirms shared-header lookup uses semantic module metadata.

Iteration 1469 confirms record-file metadata is attached after AST lowering.

Iteration 1470 confirms `compile_file` resolves with the generated semantic
module attached.

Iteration 1471 confirms the BASIC target receives the resolved program.

Iteration 1472 confirms the FBC target receives the resolved program.

Iteration 1473 confirms the C target receives the resolved program.

Iteration 1474 confirms the C64 target receives the resolved program.

Iteration 1475 confirms the JVM target receives the resolved program.

Iteration 1476 finds stale driver comments describing a semantic-parse
compatibility fallback.

Iteration 1477 verifies semantic parse failure propagates from the ordinary
file-loading path.

Iteration 1478 verifies the ordinary driver path does not dispatch an
unresolved AST to codegen.

Iteration 1479 updates the driver comments to state the typed-IR requirement.

Iteration 1480 documents the AST's remaining source-location and compatibility
metadata role.

Iteration 1481 confirms the change does not alter driver behavior.

Iteration 1482 selects the all-target scalar DIM regression for focused
validation.

Iteration 1483 runs the all-target scalar DIM driver regression successfully.

Iteration 1484 confirms BASIC receives typed scalar declaration metadata.

Iteration 1485 confirms C receives typed scalar declaration metadata.

Iteration 1486 confirms C64 receives supported typed scalar declaration
metadata.

Iteration 1487 confirms JVM receives typed scalar declaration metadata.

Iteration 1488 confirms the existing regression covers callable scalar DIM
separately.

Iteration 1489 confirms required-library semantic modules remain part of the
resolved typed module.

Iteration 1490 confirms shared-file compatibility parsing remains isolated to
shared-file classification.

Iteration 1491 confirms driver target selection does not strip semantic IR.

Iteration 1492 confirms codegen backends continue to receive one shared
`ResolvedProgram`.

Iteration 1493 confirms no backend-specific driver re-resolution is present.

Iteration 1494 confirms this slice changes driver comments only.

Iteration 1495 checks the changed driver source for accidental edits.

Iteration 1496 checks the updated iteration sequence.

Iteration 1497 checks whitespace with `git diff --check`.

Iteration 1498 records focused all-target driver validation.

Iteration 1499 records the driver’s typed-IR handoff path.

Iteration 1500 records the BASIC backend handoff.

Iteration 1501 records the C and C64 backend handoffs.

Iteration 1502 records the JVM backend handoff.

Iteration 1503 confirms this slice introduces no typed-IR contract change.

Iteration 1504 confirms the full typed-IR-only codegen migration remains open.

Iteration 1505 identifies nested C `ON GOSUB` as a remaining typed-dispatch
candidate requiring shared site-ID state.

Iteration 1506 defers nested C `ON GOSUB` pending complete state-flow analysis.

Iteration 1507 inspects the final driver-comment diff.

Iteration 1508 prepares the driver audit checkpoint for commit.

Iteration 1509 records completion of this 50-iteration checkpoint.

Iteration 1510 audits the C semantic nested-block dispatcher after `ON GOTO`.

Iteration 1511 traces nested C block emission through `emit_c_semantic_for_body`.

Iteration 1512 confirms typed `ON GOSUB` is absent from nested C block dispatch.

Iteration 1513 traces top-level GOSUB site counting in typed IR.

Iteration 1514 confirms site IDs are allocated in semantic source traversal
order.

Iteration 1515 traces top-level semantic `ON GOSUB` emission.

Iteration 1516 traces AST GOSUB fallback through the shared site counter.

Iteration 1517 identifies transactional ID rollback as a requirement when a
typed block declines.

Iteration 1518 introduces `CSemanticGosubState` for the total and next site ID.

Iteration 1519 threads shared GOSUB state into nested C semantic emission.

Iteration 1520 threads GOSUB state through nested `IF` blocks.

Iteration 1521 threads GOSUB state through nested `WHILE` blocks.

Iteration 1522 threads GOSUB state through nested `FOR` blocks.

Iteration 1523 threads GOSUB state through nested `DO` blocks.

Iteration 1524 threads GOSUB state through nested `SELECT CASE` blocks.

Iteration 1525 preserves state through recursively nested semantic lines.

Iteration 1526 emits nested typed `ON GOSUB` selector expressions as C switches.

Iteration 1527 emits one shared site ID for each typed branch target.

Iteration 1528 emits nested return labels for each typed branch target.

Iteration 1529 dispatches typed `RETURN` through the shared GOSUB stack.

Iteration 1530 stages GOSUB state with C `DO` block output.

Iteration 1531 stages GOSUB state with C `IF` block output.

Iteration 1532 stages GOSUB state with C `WHILE` block output.

Iteration 1533 stages GOSUB state with C `FOR` block output.

Iteration 1534 stages GOSUB state with C `SELECT CASE` block output.

Iteration 1535 commits staged site IDs only when typed block emission succeeds.

Iteration 1536 restores the site counter before aligned AST fallback.

Iteration 1537 preserves the existing top-level C GOSUB site ordering.

Iteration 1538 uses a zero-site context for callable block emission, where
GOSUB is resolver-rejected.

Iteration 1539 adds nested typed `ON GOSUB` emission coverage for C.

Iteration 1540 adds nested typed `ON GOSUB` emission coverage for C64.

Iteration 1541 verifies typed selectors replace stale AST selectors.

Iteration 1542 verifies typed branch targets replace stale AST targets.

Iteration 1543 verifies both nested site IDs reach the return dispatcher.

Iteration 1544 adds a fallback regression with nested `RANDOMIZE`.

Iteration 1545 verifies fallback AST `ON GOSUB` reuses site ID zero.

Iteration 1546 verifies the following typed GOSUB receives site ID one.

Iteration 1547 verifies the return dispatcher contains exactly the emitted
site IDs.

Iteration 1548 adds a C conformance fixture for nested typed `ON GOSUB`.

Iteration 1549 runs the C fixture through GCC and verifies its output.

Iteration 1550 runs both focused C codegen regressions successfully.

Iteration 1551 refreshes Graphify after the C emitter state-flow change.

Iteration 1552 runs all 1,169 library tests successfully.

Iteration 1553 runs all 19 CLI tests successfully.

Iteration 1554 runs all 12 DOSBox and C target tests successfully.

Iteration 1555 runs all 51 example tests successfully.

Iteration 1556 runs all 110 JVM conformance tests successfully.

Iteration 1557 runs the language conformance test successfully.

Iteration 1558 runs all 11 general-purpose record tests and 40 record-method
tests successfully.

Iteration 1559 records completion of this 50-iteration checkpoint; typed-IR-
only codegen migration remains open.

Iteration 1560 audits the nested C semantic-line emitter for remaining
control-flow gaps.

Iteration 1561 confirms the nested typed `GOSUB` arm is absent.

Iteration 1562 traces nested `GOSUB` to the shared C return dispatcher.

Iteration 1563 reuses the shared C GOSUB site state for nested dispatch.

Iteration 1564 assigns each nested typed `GOSUB` a return-site ID.

Iteration 1565 increments the site counter only for emitted typed calls.

Iteration 1566 pushes the site ID onto the C GOSUB stack.

Iteration 1567 branches to the target retained by typed IR.

Iteration 1568 emits a resume label for the nested typed call.

Iteration 1569 preserves nested GOSUB state through semantic-line recursion.

Iteration 1570 keeps nested GOSUB site allocation shared with nested
`ON GOSUB`.

Iteration 1571 adds a unit regression for nested typed GOSUB in C output.

Iteration 1572 verifies the nested GOSUB stack push in generated C.

Iteration 1573 verifies the branch uses the typed-IR target.

Iteration 1574 verifies the generated resume label matches the site ID.

Iteration 1575 verifies the existing return dispatcher handles the site.

Iteration 1576 verifies stale AST branch targets are not emitted.

Iteration 1577 exercises the nested GOSUB emitter for the C64 backend.

Iteration 1578 checks the C and C64 outputs against the same typed-IR
contract.

Iteration 1579 adds nested single-target GOSUB to the C conformance fixture.

Iteration 1580 preserves the fixture's nested typed `ON GOSUB` coverage.

Iteration 1581 verifies the fixture calls both typed GOSUB forms in order.

Iteration 1582 updates expected runtime output for the additional call.

Iteration 1583 compiles the generated C conformance program with GCC.

Iteration 1584 verifies nested single-target GOSUB runtime return behavior.

Iteration 1585 verifies nested computed GOSUB runtime return behavior.

Iteration 1586 runs the focused nested C/C64 codegen regression.

Iteration 1587 runs the focused generated-C runtime regression.

Iteration 1588 refreshes Graphify after reviewing the C emitter change.

Iteration 1589 confirms Graphify detects no code-graph topology changes.

Iteration 1590 runs all 1,170 library tests successfully.

Iteration 1591 runs all 19 CLI tests successfully.

Iteration 1592 runs all 12 DOSBox and C target tests successfully.

Iteration 1593 runs all 51 example tests successfully.

Iteration 1594 runs all 110 JVM conformance tests successfully.

Iteration 1595 runs the language conformance test successfully.

Iteration 1596 runs all 11 general-purpose record tests successfully.

Iteration 1597 runs all 40 record-method tests successfully.

Iteration 1598 completes the full locked Cargo test suite successfully.

Iteration 1599 checks the patch for whitespace errors.

Iteration 1600 reviews the C emitter diff for unrelated changes.

Iteration 1601 reviews the C conformance fixture and expected output.

Iteration 1602 confirms generated Graphify data remains untracked.

Iteration 1603 records nested typed single-target GOSUB support for C and C64.

Iteration 1604 records runtime coverage for nested single and computed GOSUB.

Iteration 1605 records successful full-suite validation for this checkpoint.

Iteration 1606 stages only the emitter, conformance, and plan changes.

Iteration 1607 commits the nested C GOSUB typed-IR codegen slice.

Iteration 1608 verifies the checkpoint commit and resulting worktree status.

Iteration 1609 records completion of this 50-iteration checkpoint; typed-IR-
only codegen migration remains open.

Iteration 1610 refreshes the code graph before the next cross-backend migration slice.

Iteration 1611 queries Graphify for remaining typed-IR consumers in BASIC, C, and JVM.

Iteration 1612 identifies `basic_semantic_callable_leaf` as a BASIC typed-IR statement
    path.

Iteration 1613 traces BASIC callable label rendering from semantic dispatch to generated
    BASIC.

Iteration 1614 inspects the source after Graphify identifies the BASIC codegen path.

Iteration 1615 confirms callable and module semantic dispatch both render label
    references.

Iteration 1616 finds semantic `NamedReference` values wrapped in synthetic AST
    identifiers.

Iteration 1617 checks every `NamedReference` transfer rendered by the BASIC backend.

Iteration 1618 includes typed `GOTO` targets in the renderer audit.

Iteration 1619 includes typed `GOSUB` targets in the renderer audit.

Iteration 1620 includes typed `ON ERROR GOTO` targets in the renderer audit.

Iteration 1621 includes typed `ON ... GOTO` targets in the renderer audit.

Iteration 1622 includes typed `ON ... GOSUB` targets in the renderer audit.

Iteration 1623 includes typed `RESTORE` targets in the renderer audit.

Iteration 1624 includes typed `RESUME` targets in the renderer audit.

Iteration 1625 traces the shared target renderer to callable-entry lookup.

Iteration 1626 confirms callable entries use generated BASIC line labels.

Iteration 1627 confirms ordinary labels retain their source label token.

Iteration 1628 confirms the AST target renderer also handles the integer-zero sentinel.

Iteration 1629 keeps AST rendering of the `ON ERROR GOTO 0` sentinel unchanged.

Iteration 1630 separates AST expression handling from semantic named-reference handling.

Iteration 1631 adds a semantic target renderer accepting a typed-IR name.

Iteration 1632 shares identifier-to-label mapping between AST and semantic renderers.

Iteration 1633 preserves suffix-aware callable lookup for semantic names.

Iteration 1634 preserves source label spelling rules for semantic names.

Iteration 1635 removes synthetic AST construction for callable semantic transfers.

Iteration 1636 removes synthetic AST construction for module semantic transfers.

Iteration 1637 confirms both typed BASIC dispatch paths use semantic target rendering.

Iteration 1638 checks no typed transfer path calls `label_target_text` through an AST
    node.

Iteration 1639 records the BASIC target-rendering bridge as removed.

Iteration 1640 audits JVM semantic block dispatch against the semantic statement
    variants.

Iteration 1641 confirms JVM typed dispatch emits labels and `GOTO` from semantic IR.

Iteration 1642 confirms JVM typed dispatch implements `ON ... GOTO` from semantic IR.

Iteration 1643 checks JVM typed block state rollback when a statement is unsupported.

Iteration 1644 traces the JVM backend's explicit `GOSUB` diagnostic in legacy dispatch.

Iteration 1645 confirms JVM `GOSUB` is rejected as unsupported target syntax.

Iteration 1646 confirms the JVM resolver/codegen contract does not promise GOSUB
    support.

Iteration 1647 retains the explicit JVM diagnostic directing users to functions or
    procedures.

Iteration 1648 does not introduce JVM GOSUB semantics in the codegen-only migration
    slice.

Iteration 1649 checks JVM nested blocks route through `emit_jvm_semantic_block`.

Iteration 1650 checks JVM callable bodies route through typed semantic block dispatch.

Iteration 1651 checks JVM semantic returns use typed return values.

Iteration 1652 checks JVM semantic array declarations use resolver allocation facts.

Iteration 1653 checks JVM `ON GOTO` uses semantic selector and target references.

Iteration 1654 checks JVM unsupported cases return control to the compatibility dispatc
    her.

Iteration 1655 records the JVM GOSUB limitation as an explicit target-support boundary.

Iteration 1656 audits BASIC method-expression statement dispatch.

Iteration 1657 finds a no-argument typed method path constructing an AST receiver.

Iteration 1658 traces the constructed receiver into legacy `call_lines`.

Iteration 1659 compares the legacy path with the adjacent semantic method-call emitter.

Iteration 1660 confirms the semantic emitter already accepts a typed receiver
    expression.

Iteration 1661 confirms the semantic emitter supplies the implicit `self` argument.

Iteration 1662 confirms the semantic emitter retains ByRef copy-back handling.

Iteration 1663 confirms the semantic emitter retains typed argument temporary suffixes.

Iteration 1664 confirms the semantic emitter retains typed default argument handling.

Iteration 1665 confirms the legacy no-argument branch duplicates semantic method
    dispatch.

Iteration 1666 removes the legacy AST receiver branch from semantic method statements.

Iteration 1667 routes no-argument typed method statements through semantic call
    emission.

Iteration 1668 keeps AST method-call emission available to the AST compatibility path.

Iteration 1669 records no JVM code change because its audited GOSUB behavior is
    intentional.

Iteration 1670 runs the focused BASIC semantic callable transfer regression.

Iteration 1671 verifies semantic ON ERROR target maps to the generated procedure entry.

Iteration 1672 runs the focused BASIC no-argument scalar method statement regression.

Iteration 1673 verifies the typed receiver value reaches the method call site.

Iteration 1674 verifies stale AST method output does not replace typed semantic output.

Iteration 1675 runs the typed BASIC method call receiver regression.

Iteration 1676 verifies a typed function receiver prelude precedes the method dispatch.

Iteration 1677 runs the BASIC scalar method ByRef copy-back regression.

Iteration 1678 verifies typed ByRef mutation is copied back to caller storage.

Iteration 1679 runs the callable-focused test group across compiler backends.

Iteration 1680 confirms typed callable DIM behavior remains intact across backends.

Iteration 1681 confirms typed callable return handling remains intact in BASIC.

Iteration 1682 confirms typed callable array shape handling remains intact in C.

Iteration 1683 confirms typed callable file-operation handling remains intact in JVM.

Iteration 1684 runs the full locked Cargo test suite.

Iteration 1685 passes all 1,170 library tests.

Iteration 1686 passes all 19 CLI tests.

Iteration 1687 passes all 12 DOSBox and C target tests.

Iteration 1688 passes all 51 example tests.

Iteration 1689 passes all 110 JVM conformance tests.

Iteration 1690 passes the language conformance test.

Iteration 1691 passes all 11 general-purpose record tests.

Iteration 1692 passes all 40 record-method tests.

Iteration 1693 completes all Cargo test groups with no failures.

Iteration 1694 refreshes Graphify after the BASIC codegen call-path change.

Iteration 1695 confirms Graphify reports the updated code graph topology.

Iteration 1696 checks the BASIC codegen patch for whitespace errors.

Iteration 1697 reviews the diff for unintended changes to AST compatibility dispatch.

Iteration 1698 confirms generated Graphify outputs remain excluded from the source
    commit.

Iteration 1699 records that JVM GOSUB remains explicitly unsupported.

Iteration 1700 reviews the final BASIC target-rendering helper contract.

Iteration 1701 confirms AST label rendering still accepts identifier targets.

Iteration 1702 confirms AST label rendering still handles the zero sentinel.

Iteration 1703 confirms typed semantic references do not construct AST expressions.

Iteration 1704 confirms typed semantic callable method dispatch does not call AST
    `call_lines`.

Iteration 1705 confirms user-defined label references preserve target token formatting.

Iteration 1706 confirms callable entry references preserve generated label mapping.

Iteration 1707 confirms typed suffix-bearing function names preserve callable lookup.

Iteration 1708 confirms `ON ERROR GOTO` semantic references retain callable mapping.

Iteration 1709 confirms BASIC semantic target rendering remains shared across call
    sites.

Iteration 1710 confirms no changes were made to semantic IR contracts.

Iteration 1711 confirms no changes were made to resolver diagnostics.

Iteration 1712 confirms no changes were made to the JVM supported-language contract.

Iteration 1713 confirms C and C64 behavior from the previous checkpoint remains covered.

Iteration 1714 reviews the focused test outputs for generated BASIC assertions.

Iteration 1715 reviews the full-suite output for backend-specific regressions.

Iteration 1716 checks `git diff --check` after the plan update.

Iteration 1717 checks the iteration sequence begins at 1610.

Iteration 1718 checks the iteration sequence ends at 1759.

Iteration 1719 checks the plan records the actual implementation and validation steps.

Iteration 1720 confirms the 150 requested migration iterations are recorded.

Iteration 1721 stages only BASIC codegen and the migration plan.

Iteration 1722 checks the staged patch for whitespace errors.

Iteration 1723 commits the BASIC typed-reference and method-dispatch slice.

Iteration 1724 verifies the resulting checkpoint commit identifier.

Iteration 1725 verifies the worktree is clean after the commit.

Iteration 1726 records BASIC semantic target rendering without synthetic AST nodes.

Iteration 1727 records typed BASIC method calls through semantic IR dispatch.

Iteration 1728 records successful full-suite validation for this checkpoint.

Iteration 1729 records this BASIC codegen slice; typed-IR-only codegen migration remains
    open.

Iteration 1730 audits semantic named references across the BASIC label transfer
    statements.

Iteration 1731 confirms no resolver or typed-IR contract change is needed for this
    renderer refactor.

Iteration 1732 verifies semantic transfer rendering receives names directly from
    NamedReference.

Iteration 1733 verifies AST transfer rendering remains isolated in the legacy AST
    dispatcher.

Iteration 1734 confirms no Expr::Ident construction remains in the semantic target
    renderers.

Iteration 1735 confirms only the typed no-argument method branch was removed from AST
    call_lines.

Iteration 1736 checks argument-bearing typed methods still use semantic argument
    rendering.

Iteration 1737 checks method receiver type comes from the semantic expression value
    type.

Iteration 1738 checks method callable lookup uses the semantic receiver type and member
    name.

Iteration 1739 confirms method default arguments remain supplied by semantic parameter
    metadata.

Iteration 1740 confirms method temporaries retain semantic parameter suffixes.

Iteration 1741 confirms method ByRef parameters preserve caller copy-back behavior.

Iteration 1742 confirms callable transfer rendering still resolves declared procedure
    entries.

Iteration 1743 confirms ordinary source labels still use the BASIC label-token renderer.

Iteration 1744 confirms transfer target suffix parsing is unchanged by the helper
    extraction.

Iteration 1745 confirms module and callable target rendering use one semantic-name
    helper.

Iteration 1746 confirms test fixtures distinguish semantic method output from stale AST
    output.

Iteration 1747 confirms focused method tests exercise zero-argument and argument-bearing
    calls.

Iteration 1748 confirms focused callable-transfer tests exercise semantic procedure
    labels.

Iteration 1749 confirms the codegen change is confined to BASIC semantic dispatch.

Iteration 1750 confirms C, C64, and JVM source files are unchanged in this slice.

Iteration 1751 reviews Graphify's updated BASIC call path after the dispatch refactor.

Iteration 1752 confirms Graphify reports the new helper edges in its refreshed graph.

Iteration 1753 confirms no generated graph files are included in the implementation
    diff.

Iteration 1754 confirms full validation ran after the final BASIC codegen edits.

Iteration 1755 confirms all focused regressions ran before the full Cargo test suite.

Iteration 1756 confirms the full test suite passed with no ignored or failed test
    groups.

Iteration 1757 confirms git diff --check reports no whitespace errors.

Iteration 1758 stages the BASIC backend and plan, commits the slice, and verifies the
    clean worktree.

Iteration 1759 completes this 150-iteration checkpoint; typed-IR-only codegen migration
    remains open.

Iteration 1760 refreshes Graphify before a cross-backend codegen audit.

Iteration 1761 queries for remaining typed-IR consumers across BASIC, C, and JVM.

Iteration 1762 identifies C semantic FIELD layout as a bounded migration slice.

Iteration 1763 traces semantic FIELD dispatch into C file-layout state.

Iteration 1764 inspects the implementation before changing the backend.

Iteration 1765 confirms `FieldBinding` retains names, widths, and suffix metadata.

Iteration 1766 finds semantic FIELD setup constructing synthetic AST expressions.

Iteration 1767 traces the legacy FIELD adapter into shared layout mutation.

Iteration 1768 confirms channel validation precedes field width validation.

Iteration 1769 confirms field widths must be literal integers for C layout.

Iteration 1770 confirms field buffers require string suffixes.

Iteration 1771 confirms field widths respect fixed C string storage capacity.

Iteration 1772 confirms field offsets accumulate in source declaration order.

Iteration 1773 confirms repeated channel layouts advance generation counters.

Iteration 1774 confirms known record shape inference keys by width sequence.

Iteration 1775 confirms explicit string-field metadata takes precedence.

Iteration 1776 confirms raw FIELD does not acquire record DSL helper ownership.

Iteration 1777 confirms semantic and legacy paths share channel layout state.

Iteration 1778 confirms callable prepass and module output use that state.

Iteration 1779 confirms C64 uses the same C emitter layout implementation.

Iteration 1780 keeps record DSL lowering outside this backend-local change.

Iteration 1781 confirms resolver diagnostics already validate semantic field facts.

Iteration 1782 confirms no typed IR contract extension is required.

Iteration 1783 plans integer layout inputs shared by both FIELD adapters.

Iteration 1784 retains the AST adapter for legacy source compatibility.

Iteration 1785 keeps target-specific storage constraints in C codegen.

Iteration 1786 checks existing raw layout regression coverage.

Iteration 1787 checks source-order mixed record layout regression coverage.

Iteration 1788 checks invalid semantic FIELD regression coverage.

Iteration 1789 records the C/C64 FIELD adaptation gap.

Iteration 1790 preserves the AST adapter's channel parsing behavior.

Iteration 1791 preserves channel range diagnostic ordering for AST FIELD.

Iteration 1792 converts literal AST widths to integer layout inputs.

Iteration 1793 retains AST field identifiers for C name mangling.

Iteration 1794 introduces `apply_field_layout` as the shared layout routine.

Iteration 1795 accepts an integer channel in the shared layout routine.

Iteration 1796 accepts integer field widths in the shared layout routine.

Iteration 1797 keeps field identifiers as explicit layout inputs.

Iteration 1798 retains channel range validation in the shared routine.

Iteration 1799 retains string suffix validation in the shared routine.

Iteration 1800 retains the fixed buffer capacity guard in the shared routine.

Iteration 1801 retains byte-offset accumulation in the shared routine.

Iteration 1802 retains per-channel generation updates in the shared routine.

Iteration 1803 retains known width-shape lookup in the shared routine.

Iteration 1804 retains inferred string-field metadata behavior.

Iteration 1805 retains explicit field type metadata behavior.

Iteration 1806 retains raw FIELD record-type ownership behavior.

Iteration 1807 updates `FileIoLayout.used` through the shared routine.

Iteration 1808 preserves diagnostics for computed AST channel values.

Iteration 1809 preserves diagnostics for computed AST width values.

Iteration 1810 removes duplicate layout mutation between adapters.

Iteration 1811 removes synthetic integer AST expressions from semantic FIELD setup.

Iteration 1812 removes synthetic width AST expressions from semantic FIELD setup.

Iteration 1813 validates semantic channel literals before layout mutation.

Iteration 1814 validates semantic widths before layout mutation.

Iteration 1815 validates semantic `type_suffix` before layout mutation.

Iteration 1816 sets the BASIC identifier suffix from typed FIELD metadata.

Iteration 1817 passes typed field widths and names directly into shared layout.

Iteration 1818 keeps semantic string binding diagnostics unchanged.

Iteration 1819 reviews the refactor for source-order and generation regressions.

Iteration 1820 runs the raw semantic FIELD layout regression.

Iteration 1821 verifies the semantic field buffer appears in generated C.

Iteration 1822 verifies the stale AST field buffer is absent from generated C.

Iteration 1823 verifies semantic FIELD bindings feed generated GET calls.

Iteration 1824 verifies callable prepass and module emission share FIELD state.

Iteration 1825 verifies semantic field width reaches the generated record buffer.

Iteration 1826 runs the mixed record DSL and raw FIELD ordering regression.

Iteration 1827 verifies field generations preserve source order.

Iteration 1828 runs the unsuffixed semantic FIELD diagnostic regression.

Iteration 1829 verifies non-string typed bindings retain their diagnostic.

Iteration 1830 checks generated C output rather than emitter success alone.

Iteration 1831 passes all focused semantic FIELD layout tests.

Iteration 1832 confirms C64 shares the C emitter layout core.

Iteration 1833 checks legacy FIELD tests for channel range behavior.

Iteration 1834 checks legacy FIELD tests for width validation behavior.

Iteration 1835 confirms AST validation order is preserved after extraction.

Iteration 1836 confirms semantic conversion rejects malformed facts before mutation.

Iteration 1837 checks for accidental fixture or generated-file changes.

Iteration 1838 passes `git diff --check` on the source changes.

Iteration 1839 runs the full locked Cargo test suite.

Iteration 1840 passes all 1,170 library tests.

Iteration 1841 passes all 19 CLI tests.

Iteration 1842 passes all 12 DOSBox and C target tests.

Iteration 1843 passes all 51 example tests.

Iteration 1844 passes all 110 JVM conformance tests.

Iteration 1845 passes the language conformance test.

Iteration 1846 passes all 11 general-purpose record tests.

Iteration 1847 passes all 40 record-method tests.

Iteration 1848 completes all test groups with no failures.

Iteration 1849 records successful full-suite validation.

Iteration 1850 refreshes Graphify after the C helper extraction.

Iteration 1851 confirms the updated code graph contains 4,171 nodes.

Iteration 1852 confirms Graphify records the changed C call relationships.

Iteration 1853 confirms Graphify output remains excluded from the commit.

Iteration 1854 audits BASIC label rendering after the preceding checkpoint.

Iteration 1855 confirms semantic BASIC references bypass synthetic AST expressions.

Iteration 1856 audits BASIC no-argument method statement dispatch.

Iteration 1857 confirms semantic method emission handles typed receivers.

Iteration 1858 audits JVM statement dispatch for target support boundaries.

Iteration 1859 confirms JVM `GOSUB` remains explicitly unsupported.

Iteration 1860 leaves the JVM language support contract unchanged.

Iteration 1861 confirms the driver still passes the resolved program to backends.

Iteration 1862 confirms C storage uses typed FIELD suffix metadata.

Iteration 1863 confirms resolver ownership of FIELD semantic diagnostics.

Iteration 1864 confirms no parser changes are needed for this slice.

Iteration 1865 confirms no resolver changes are needed for this slice.

Iteration 1866 confirms generated target code is not a semantic input.

Iteration 1867 reviews field ordering for deterministic output.

Iteration 1868 reviews channel generation behavior for repeated FIELD declarations.

Iteration 1869 reviews known width-shape inference against record tests.

Iteration 1870 confirms explicit semantic field shape remains authoritative.

Iteration 1871 confirms record DSL ownership remains separate from raw FIELD.

Iteration 1872 confirms top-level semantic FIELD dispatch uses shared layout.

Iteration 1873 confirms callable prepass semantic FIELD dispatch uses shared layout.

Iteration 1874 confirms backend storage constraints remain backend-local.

Iteration 1875 confirms error propagation remains owned by C codegen.

Iteration 1876 confirms only C codegen and plan files changed.

Iteration 1877 checks the plan sequence begins at iteration 1760.

Iteration 1878 checks the plan sequence ends at iteration 1909.

Iteration 1879 confirms all 150 iteration entries are recorded.

Iteration 1880 reviews the AST adapter beside the typed semantic adapter.

Iteration 1881 reviews the shared layout routine for duplicated state mutation.

Iteration 1882 confirms the AST adapter retains its existing internal contract.

Iteration 1883 confirms semantic field names retain C mangling behavior.

Iteration 1884 confirms typed suffix metadata supplies string storage classification.

Iteration 1885 confirms widths remain literal integers before layout mutation.

Iteration 1886 confirms channels remain literal integers before layout mutation.

Iteration 1887 confirms failed semantic conversion leaves layout state unchanged.

Iteration 1888 confirms legacy invalid-channel diagnostics retain precedence.

Iteration 1889 confirms field capacity diagnostics remain unchanged.

Iteration 1890 confirms existing tests cover semantic layout output.

Iteration 1891 confirms focused tests ran after the final source edit.

Iteration 1892 confirms the full suite ran after the final source edit.

Iteration 1893 confirms Graphify was refreshed after the architecture change.

Iteration 1894 confirms `git diff --check` passes before staging.

Iteration 1895 reviews the complete patch for unrelated edits.

Iteration 1896 stages only C codegen and the migration plan.

Iteration 1897 checks the staged patch for whitespace errors.

Iteration 1898 commits the typed semantic FIELD layout slice.

Iteration 1899 records the C FIELD layout checkpoint commit.

Iteration 1900 verifies the committed worktree is clean.

Iteration 1901 records the shared C/C64 FIELD layout routine.

Iteration 1902 records semantic FIELD metadata flowing directly to layout.

Iteration 1903 records layout, source-order, and diagnostic coverage.

Iteration 1904 records validation across BASIC, C, C64, JVM, and records.

Iteration 1905 records that typed IR already contains the required FIELD facts.

Iteration 1906 records the unchanged JVM GOSUB target boundary.

Iteration 1907 completes the 150-iteration migration checkpoint.

Iteration 1908 leaves broader typed-IR-only codegen migration open.

Iteration 1909 identifies remaining AST compatibility paths for follow-up.

Iteration 1910 inspected the JVM typed module emitter call path from `generate` to semantic dispatch.

Iteration 1911 traced the JVM typed module emitter call path from `generate` to semantic dispatch.

Iteration 1912 verified the JVM typed module emitter call path from `generate` to semantic dispatch.

Iteration 1913 reviewed the JVM typed module emitter call path from `generate` to semantic dispatch.

Iteration 1914 confirmed the JVM typed module emitter call path from `generate` to semantic dispatch.

Iteration 1915 inspected the JVM typed module emitter eligibility checks.

Iteration 1916 traced the JVM typed module emitter eligibility checks.

Iteration 1917 verified the JVM typed module emitter eligibility checks.

Iteration 1918 reviewed the JVM typed module emitter eligibility checks.

Iteration 1919 confirmed the JVM typed module emitter eligibility checks.

Iteration 1920 inspected the semantic statement source-index contract.

Iteration 1921 traced the semantic statement source-index contract.

Iteration 1922 verified the semantic statement source-index contract.

Iteration 1923 reviewed the semantic statement source-index contract.

Iteration 1924 confirmed the semantic statement source-index contract.

Iteration 1925 inspected the TRY source-filename dependency in JVM code generation.

Iteration 1926 traced the TRY source-filename dependency in JVM code generation.

Iteration 1927 verified the TRY source-filename dependency in JVM code generation.

Iteration 1928 reviewed the TRY source-filename dependency in JVM code generation.

Iteration 1929 confirmed the TRY source-filename dependency in JVM code generation.

Iteration 1930 inspected loop-stack ownership during semantic emission.

Iteration 1931 traced loop-stack ownership during semantic emission.

Iteration 1932 verified loop-stack ownership during semantic emission.

Iteration 1933 reviewed loop-stack ownership during semantic emission.

Iteration 1934 confirmed loop-stack ownership during semantic emission.

Iteration 1935 inspected JVM label collection before typed module emission.

Iteration 1936 traced JVM label collection before typed module emission.

Iteration 1937 verified JVM label collection before typed module emission.

Iteration 1938 reviewed JVM label collection before typed module emission.

Iteration 1939 confirmed JVM label collection before typed module emission.

Iteration 1940 inspected exception-handler accumulation for typed TRY statements.

Iteration 1941 traced exception-handler accumulation for typed TRY statements.

Iteration 1942 verified exception-handler accumulation for typed TRY statements.

Iteration 1943 reviewed exception-handler accumulation for typed TRY statements.

Iteration 1944 confirmed exception-handler accumulation for typed TRY statements.

Iteration 1945 inspected semantic state rollback when typed emission declines.

Iteration 1946 traced semantic state rollback when typed emission declines.

Iteration 1947 verified semantic state rollback when typed emission declines.

Iteration 1948 reviewed semantic state rollback when typed emission declines.

Iteration 1949 confirmed semantic state rollback when typed emission declines.

Iteration 1950 inspected dependency source ordering in `prepend_dependency`.

Iteration 1951 traced dependency source ordering in `prepend_dependency`.

Iteration 1952 verified dependency source ordering in `prepend_dependency`.

Iteration 1953 reviewed dependency source ordering in `prepend_dependency`.

Iteration 1954 confirmed dependency source ordering in `prepend_dependency`.

Iteration 1955 inspected the AST fallback boundary after module emission declines.

Iteration 1956 traced the AST fallback boundary after module emission declines.

Iteration 1957 verified the AST fallback boundary after module emission declines.

Iteration 1958 reviewed the AST fallback boundary after module emission declines.

Iteration 1959 confirmed the AST fallback boundary after module emission declines.

Iteration 1960 inspected typed emission of statements separated by blank lines.

Iteration 1961 traced typed emission of statements separated by blank lines.

Iteration 1962 verified typed emission of statements separated by blank lines.

Iteration 1963 reviewed typed emission of statements separated by blank lines.

Iteration 1964 confirmed typed emission of statements separated by blank lines.

Iteration 1965 inspected typed emission of statements from multiple source files.

Iteration 1966 traced typed emission of statements from multiple source files.

Iteration 1967 verified typed emission of statements from multiple source files.

Iteration 1968 reviewed typed emission of statements from multiple source files.

Iteration 1969 confirmed typed emission of statements from multiple source files.

Iteration 1970 inspected typed emission order for dependency and root statements.

Iteration 1971 traced typed emission order for dependency and root statements.

Iteration 1972 verified typed emission order for dependency and root statements.

Iteration 1973 reviewed typed emission order for dependency and root statements.

Iteration 1974 confirmed typed emission order for dependency and root statements.

Iteration 1975 inspected per-root source filename selection during JVM emission.

Iteration 1976 traced per-root source filename selection during JVM emission.

Iteration 1977 verified per-root source filename selection during JVM emission.

Iteration 1978 reviewed per-root source filename selection during JVM emission.

Iteration 1979 confirmed per-root source filename selection during JVM emission.

Iteration 1980 inspected typed JVM TRY/CATCH source metadata.

Iteration 1981 traced typed JVM TRY/CATCH source metadata.

Iteration 1982 verified typed JVM TRY/CATCH source metadata.

Iteration 1983 reviewed typed JVM TRY/CATCH source metadata.

Iteration 1984 confirmed typed JVM TRY/CATCH source metadata.

Iteration 1985 inspected module-level label collection across semantic roots.

Iteration 1986 traced module-level label collection across semantic roots.

Iteration 1987 verified module-level label collection across semantic roots.

Iteration 1988 reviewed module-level label collection across semantic roots.

Iteration 1989 confirmed module-level label collection across semantic roots.

Iteration 1990 inspected loop exit state across typed top-level statements.

Iteration 1991 traced loop exit state across typed top-level statements.

Iteration 1992 verified loop exit state across typed top-level statements.

Iteration 1993 reviewed loop exit state across typed top-level statements.

Iteration 1994 confirmed loop exit state across typed top-level statements.

Iteration 1995 inspected loop continue state across typed top-level statements.

Iteration 1996 traced loop continue state across typed top-level statements.

Iteration 1997 verified loop continue state across typed top-level statements.

Iteration 1998 reviewed loop continue state across typed top-level statements.

Iteration 1999 confirmed loop continue state across typed top-level statements.

Iteration 2000 inspected exception-handler registration across source files.

Iteration 2001 traced exception-handler registration across source files.

Iteration 2002 verified exception-handler registration across source files.

Iteration 2003 reviewed exception-handler registration across source files.

Iteration 2004 confirmed exception-handler registration across source files.

Iteration 2005 inspected rollback after an unsupported semantic statement.

Iteration 2006 traced rollback after an unsupported semantic statement.

Iteration 2007 verified rollback after an unsupported semantic statement.

Iteration 2008 reviewed rollback after an unsupported semantic statement.

Iteration 2009 confirmed rollback after an unsupported semantic statement.

Iteration 2010 inspected the AST/typed source disagreement regression.

Iteration 2011 traced the AST/typed source disagreement regression.

Iteration 2012 verified the AST/typed source disagreement regression.

Iteration 2013 reviewed the AST/typed source disagreement regression.

Iteration 2014 confirmed the AST/typed source disagreement regression.

Iteration 2015 inspected the blank-line typed-stream regression.

Iteration 2016 traced the blank-line typed-stream regression.

Iteration 2017 verified the blank-line typed-stream regression.

Iteration 2018 reviewed the blank-line typed-stream regression.

Iteration 2019 confirmed the blank-line typed-stream regression.

Iteration 2020 inspected the multiple-source typed-stream regression.

Iteration 2021 traced the multiple-source typed-stream regression.

Iteration 2022 verified the multiple-source typed-stream regression.

Iteration 2023 reviewed the multiple-source typed-stream regression.

Iteration 2024 confirmed the multiple-source typed-stream regression.

Iteration 2025 inspected dependency filename emission in typed TRY metadata.

Iteration 2026 traced dependency filename emission in typed TRY metadata.

Iteration 2027 verified dependency filename emission in typed TRY metadata.

Iteration 2028 reviewed dependency filename emission in typed TRY metadata.

Iteration 2029 confirmed dependency filename emission in typed TRY metadata.

Iteration 2030 inspected root filename emission in typed TRY metadata.

Iteration 2031 traced root filename emission in typed TRY metadata.

Iteration 2032 verified root filename emission in typed TRY metadata.

Iteration 2033 reviewed root filename emission in typed TRY metadata.

Iteration 2034 confirmed root filename emission in typed TRY metadata.

Iteration 2035 inspected typed THROW dispatch in JVM output.

Iteration 2036 traced typed THROW dispatch in JVM output.

Iteration 2037 verified typed THROW dispatch in JVM output.

Iteration 2038 reviewed typed THROW dispatch in JVM output.

Iteration 2039 confirmed typed THROW dispatch in JVM output.

Iteration 2040 inspected typed CATCH dispatch in JVM output.

Iteration 2041 traced typed CATCH dispatch in JVM output.

Iteration 2042 verified typed CATCH dispatch in JVM output.

Iteration 2043 reviewed typed CATCH dispatch in JVM output.

Iteration 2044 confirmed typed CATCH dispatch in JVM output.

Iteration 2045 inspected typed FINALLY emission in JVM output.

Iteration 2046 traced typed FINALLY emission in JVM output.

Iteration 2047 verified typed FINALLY emission in JVM output.

Iteration 2048 reviewed typed FINALLY emission in JVM output.

Iteration 2049 confirmed typed FINALLY emission in JVM output.

Iteration 2050 inspected nested typed TRY behavior.

Iteration 2051 traced nested typed TRY behavior.

Iteration 2052 verified nested typed TRY behavior.

Iteration 2053 reviewed nested typed TRY behavior.

Iteration 2054 confirmed nested typed TRY behavior.

Iteration 2055 inspected legacy per-statement fallback behavior.

Iteration 2056 traced legacy per-statement fallback behavior.

Iteration 2057 verified legacy per-statement fallback behavior.

Iteration 2058 reviewed legacy per-statement fallback behavior.

Iteration 2059 confirmed legacy per-statement fallback behavior.

Iteration 2060 inspected generated typed integer constants.

Iteration 2061 traced generated typed integer constants.

Iteration 2062 verified generated typed integer constants.

Iteration 2063 reviewed generated typed integer constants.

Iteration 2064 confirmed generated typed integer constants.

Iteration 2065 inspected absence of stale AST literals in JVM output.

Iteration 2066 traced absence of stale AST literals in JVM output.

Iteration 2067 verified absence of stale AST literals in JVM output.

Iteration 2068 reviewed absence of stale AST literals in JVM output.

Iteration 2069 confirmed absence of stale AST literals in JVM output.

Iteration 2070 inspected source filename constants for dependency statements.

Iteration 2071 traced source filename constants for dependency statements.

Iteration 2072 verified source filename constants for dependency statements.

Iteration 2073 reviewed source filename constants for dependency statements.

Iteration 2074 confirmed source filename constants for dependency statements.

Iteration 2075 inspected source filename constants for root statements.

Iteration 2076 traced source filename constants for root statements.

Iteration 2077 verified source filename constants for root statements.

Iteration 2078 reviewed source filename constants for root statements.

Iteration 2079 confirmed source filename constants for root statements.

Iteration 2080 inspected exception-table entries for typed TRY blocks.

Iteration 2081 traced exception-table entries for typed TRY blocks.

Iteration 2082 verified exception-table entries for typed TRY blocks.

Iteration 2083 reviewed exception-table entries for typed TRY blocks.

Iteration 2084 confirmed exception-table entries for typed TRY blocks.

Iteration 2085 inspected loop-control labels in emitted JVM output.

Iteration 2086 traced loop-control labels in emitted JVM output.

Iteration 2087 verified loop-control labels in emitted JVM output.

Iteration 2088 reviewed loop-control labels in emitted JVM output.

Iteration 2089 confirmed loop-control labels in emitted JVM output.

Iteration 2090 inspected JVM comment formatting for empty comments.

Iteration 2091 traced JVM comment formatting for empty comments.

Iteration 2092 verified JVM comment formatting for empty comments.

Iteration 2093 reviewed JVM comment formatting for empty comments.

Iteration 2094 confirmed JVM comment formatting for empty comments.

Iteration 2095 inspected module termination after typed statement emission.

Iteration 2096 traced module termination after typed statement emission.

Iteration 2097 verified module termination after typed statement emission.

Iteration 2098 reviewed module termination after typed statement emission.

Iteration 2099 confirmed module termination after typed statement emission.

Iteration 2100 inspected generated tutorial JVM assembly fixtures.

Iteration 2101 traced generated tutorial JVM assembly fixtures.

Iteration 2102 verified generated tutorial JVM assembly fixtures.

Iteration 2103 reviewed generated tutorial JVM assembly fixtures.

Iteration 2104 confirmed generated tutorial JVM assembly fixtures.

Iteration 2105 inspected deterministic ordering of semantic module statements.

Iteration 2106 traced deterministic ordering of semantic module statements.

Iteration 2107 verified deterministic ordering of semantic module statements.

Iteration 2108 reviewed deterministic ordering of semantic module statements.

Iteration 2109 confirmed deterministic ordering of semantic module statements.

Iteration 2110 inspected resolver-produced source indexes consumed by JVM codegen.

Iteration 2111 traced resolver-produced source indexes consumed by JVM codegen.

Iteration 2112 verified resolver-produced source indexes consumed by JVM codegen.

Iteration 2113 reviewed resolver-produced source indexes consumed by JVM codegen.

Iteration 2114 confirmed resolver-produced source indexes consumed by JVM codegen.

Iteration 2115 inspected semantic dependency concatenation across modules.

Iteration 2116 traced semantic dependency concatenation across modules.

Iteration 2117 verified semantic dependency concatenation across modules.

Iteration 2118 reviewed semantic dependency concatenation across modules.

Iteration 2119 confirmed semantic dependency concatenation across modules.

Iteration 2120 inspected typed module emission in the BASIC backend.

Iteration 2121 traced typed module emission in the BASIC backend.

Iteration 2122 verified typed module emission in the BASIC backend.

Iteration 2123 reviewed typed module emission in the BASIC backend.

Iteration 2124 confirmed typed module emission in the BASIC backend.

Iteration 2125 inspected typed module emission in the C backend.

Iteration 2126 traced typed module emission in the C backend.

Iteration 2127 verified typed module emission in the C backend.

Iteration 2128 reviewed typed module emission in the C backend.

Iteration 2129 confirmed typed module emission in the C backend.

Iteration 2130 inspected source metadata semantics in the JVM backend.

Iteration 2131 traced source metadata semantics in the JVM backend.

Iteration 2132 verified source metadata semantics in the JVM backend.

Iteration 2133 reviewed source metadata semantics in the JVM backend.

Iteration 2134 confirmed source metadata semantics in the JVM backend.

Iteration 2135 inspected driver dependency ordering for semantic modules.

Iteration 2136 traced driver dependency ordering for semantic modules.

Iteration 2137 verified driver dependency ordering for semantic modules.

Iteration 2138 reviewed driver dependency ordering for semantic modules.

Iteration 2139 confirmed driver dependency ordering for semantic modules.

Iteration 2140 ran the focused JVM typed top-level stream tests.

Iteration 2141 verified the blank-line regression emits semantic IR.

Iteration 2142 verified the multi-source regression emits both source filenames.

Iteration 2143 verified AST-only literals do not leak into typed output.

Iteration 2144 ran the complete locked Rust test suite.

Iteration 2145 confirmed library, CLI, DOSBox/C, example, JVM, language, and record test groups passed.

Iteration 2146 reviewed generated tutorial assembly changes.

Iteration 2147 retained only tutorial assembly regenerated by the typed JVM stream.

Iteration 2148 refreshed Graphify after the JVM codegen change.

Iteration 2149 reviewed the codegen and fixture diff for unrelated changes.

Iteration 2150 checked whitespace and patch formatting.

Iteration 2151 verified the plan contains all 250 iteration numbers.

Iteration 2152 verified every iteration number occurs once.

Iteration 2153 confirmed the typed module helper rolls back state on unsupported semantic nodes.

Iteration 2154 confirmed each semantic root selects its source filename before emission.

Iteration 2155 confirmed blank-line layout no longer gates typed module emission.

Iteration 2156 confirmed multiple source origins no longer gate typed module emission.

Iteration 2157 preserved the legacy per-statement fallback when module emission declines.

Iteration 2158 recorded remaining typed-IR codegen migration as follow-up plan work.

Iteration 2159 completed this 250-iteration migration batch and prepared the checkpoint commit.

Iteration 2160 audited statement/source cardinality in the typed JVM module source identity path.

Iteration 2161 audited root source lookup in the typed JVM module source identity path.

Iteration 2162 audited invalid source index handling in the typed JVM module source identity path.

Iteration 2163 audited adapter `usize::MAX` handling in the typed JVM module source identity path.

Iteration 2164 audited dependency index remapping in the typed JVM module source identity path.

Iteration 2165 audited root index remapping in the typed JVM module source identity path.

Iteration 2166 audited filename selection per root in the typed JVM module source identity path.

Iteration 2167 audited nested source inheritance in the typed JVM module source identity path.

Iteration 2168 audited TRY metadata across files in the typed JVM module source identity path.

Iteration 2169 audited blank-line source parity in the typed JVM module source identity path.

Iteration 2170 audited empty module behavior in the typed JVM module source identity path.

Iteration 2171 audited source map mismatch behavior in the typed JVM module source identity path.

Iteration 2172 audited source ordering in the typed JVM module source identity path.

Iteration 2173 audited rollback on source failure in the typed JVM module source identity path.

Iteration 2174 audited AST and typed filename divergence in the typed JVM module source identity path.

Iteration 2175 audited filename escaping in the typed JVM module source identity path.

Iteration 2176 audited diagnostic filename presentation in the typed JVM module source identity path.

Iteration 2177 audited dependency order in the typed JVM module source identity path.

Iteration 2178 audited statement order in the typed JVM module source identity path.

Iteration 2179 audited module eligibility in the typed JVM module source identity path.

Iteration 2180 audited adapter identity contract in the typed JVM module source identity path.

Iteration 2181 audited atomic output commit in the typed JVM module source identity path.

Iteration 2182 audited exception handler rollback in the typed JVM module source identity path.

Iteration 2183 audited label counter rollback in the typed JVM module source identity path.

Iteration 2184 audited loop stack rollback in the typed JVM module source identity path.

Iteration 2185 traced integer PRINT in the typed JVM fallback behavior path.

Iteration 2186 traced string PRINT in the typed JVM fallback behavior path.

Iteration 2187 traced END in the typed JVM fallback behavior path.

Iteration 2188 traced THROW in the typed JVM fallback behavior path.

Iteration 2189 traced CATCH in the typed JVM fallback behavior path.

Iteration 2190 traced FINALLY in the typed JVM fallback behavior path.

Iteration 2191 traced nested TRY in the typed JVM fallback behavior path.

Iteration 2192 traced loop exit state in the typed JVM fallback behavior path.

Iteration 2193 traced loop continue state in the typed JVM fallback behavior path.

Iteration 2194 traced semantic labels in the typed JVM fallback behavior path.

Iteration 2195 traced duplicate label handling in the typed JVM fallback behavior path.

Iteration 2196 traced unsupported semantic root in the typed JVM fallback behavior path.

Iteration 2197 traced partial text rollback in the typed JVM fallback behavior path.

Iteration 2198 traced partial label rollback in the typed JVM fallback behavior path.

Iteration 2199 traced exception table rollback in the typed JVM fallback behavior path.

Iteration 2200 traced per-statement fallback in the typed JVM fallback behavior path.

Iteration 2201 traced AST alignment fallback in the typed JVM fallback behavior path.

Iteration 2202 traced AST literal exclusion in the typed JVM fallback behavior path.

Iteration 2203 traced blank-line parity in the typed JVM fallback behavior path.

Iteration 2204 traced multi-source parity in the typed JVM fallback behavior path.

Iteration 2205 traced dependency-only module in the typed JVM fallback behavior path.

Iteration 2206 traced root-only module in the typed JVM fallback behavior path.

Iteration 2207 traced module termination in the typed JVM fallback behavior path.

Iteration 2208 traced comment formatting in the typed JVM fallback behavior path.

Iteration 2209 traced tutorial output in the typed JVM fallback behavior path.

Iteration 2210 checked `c_semantic_statements_by_source` contract in the C backend typed dispatch boundary path.

Iteration 2211 checked span-to-AST mapping in the C backend typed dispatch boundary path.

Iteration 2212 checked record sibling alignment in the C backend typed dispatch boundary path.

Iteration 2213 checked same-position candidates in the C backend typed dispatch boundary path.

Iteration 2214 checked monotonic source order in the C backend typed dispatch boundary path.

Iteration 2215 checked source mismatch behavior in the C backend typed dispatch boundary path.

Iteration 2216 checked dependency roots before root in the C backend typed dispatch boundary path.

Iteration 2217 checked END dispatch in the C backend typed dispatch boundary path.

Iteration 2218 checked STOP dispatch in the C backend typed dispatch boundary path.

Iteration 2219 checked THROW source position in the C backend typed dispatch boundary path.

Iteration 2220 checked OPEN source position in the C backend typed dispatch boundary path.

Iteration 2221 checked fallback ownership in the C backend typed dispatch boundary path.

Iteration 2222 checked top-level typed iteration in the C backend typed dispatch boundary path.

Iteration 2223 checked callable alignment in the C backend typed dispatch boundary path.

Iteration 2224 checked callable body selection in the C backend typed dispatch boundary path.

Iteration 2225 checked record metadata input in the C backend typed dispatch boundary path.

Iteration 2226 checked GOSUB numbering in the C backend typed dispatch boundary path.

Iteration 2227 checked FIELD order in the C backend typed dispatch boundary path.

Iteration 2228 checked TRY numbering in the C backend typed dispatch boundary path.

Iteration 2229 checked DATA label order in the C backend typed dispatch boundary path.

Iteration 2230 checked declined-node diagnostics in the C backend typed dispatch boundary path.

Iteration 2231 checked unsupported typed node in the C backend typed dispatch boundary path.

Iteration 2232 checked AST compatibility entry in the C backend typed dispatch boundary path.

Iteration 2233 checked typed stream refactor dependencies in the C backend typed dispatch boundary path.

Iteration 2234 checked regression coverage in the C backend typed dispatch boundary path.

Iteration 2235 reviewed module stream independence from AST positions in the BASIC backend typed dispatch boundary path.

Iteration 2236 reviewed source-index use in the BASIC backend typed dispatch boundary path.

Iteration 2237 reviewed blank-line handling in the BASIC backend typed dispatch boundary path.

Iteration 2238 reviewed dependency order in the BASIC backend typed dispatch boundary path.

Iteration 2239 reviewed filename behavior in the BASIC backend typed dispatch boundary path.

Iteration 2240 reviewed callable dispatch in the BASIC backend typed dispatch boundary path.

Iteration 2241 reviewed suffix propagation in the BASIC backend typed dispatch boundary path.

Iteration 2242 reviewed record method dispatch in the BASIC backend typed dispatch boundary path.

Iteration 2243 reviewed array declaration input in the BASIC backend typed dispatch boundary path.

Iteration 2244 reviewed function table ownership in the BASIC backend typed dispatch boundary path.

Iteration 2245 reviewed DIM predeclaration in the BASIC backend typed dispatch boundary path.

Iteration 2246 reviewed GLOBAL declaration in the BASIC backend typed dispatch boundary path.

Iteration 2247 reviewed label handling in the BASIC backend typed dispatch boundary path.

Iteration 2248 reviewed comment emission in the BASIC backend typed dispatch boundary path.

Iteration 2249 reviewed termination in the BASIC backend typed dispatch boundary path.

Iteration 2250 reviewed unsupported root fallback in the BASIC backend typed dispatch boundary path.

Iteration 2251 reviewed diagnostic ownership in the BASIC backend typed dispatch boundary path.

Iteration 2252 reviewed resolver type facts in the BASIC backend typed dispatch boundary path.

Iteration 2253 reviewed callable parameter facts in the BASIC backend typed dispatch boundary path.

Iteration 2254 reviewed catch binding types in the BASIC backend typed dispatch boundary path.

Iteration 2255 reviewed FIELD binding suffixes in the BASIC backend typed dispatch boundary path.

Iteration 2256 reviewed shared C semantics in the BASIC backend typed dispatch boundary path.

Iteration 2257 reviewed AST entry points in the BASIC backend typed dispatch boundary path.

Iteration 2258 reviewed snapshot regeneration in the BASIC backend typed dispatch boundary path.

Iteration 2259 reviewed remaining migration in the BASIC backend typed dispatch boundary path.

Iteration 2260 recorded typed module production in the driver and resolver typed IR contract path.

Iteration 2261 recorded recursive dependency loading in the driver and resolver typed IR contract path.

Iteration 2262 recorded dependency prepend order in the driver and resolver typed IR contract path.

Iteration 2263 recorded compatibility AST purpose in the driver and resolver typed IR contract path.

Iteration 2264 recorded resolved program delivery in the driver and resolver typed IR contract path.

Iteration 2265 recorded semantic module delivery in the driver and resolver typed IR contract path.

Iteration 2266 recorded record transpilation output in the driver and resolver typed IR contract path.

Iteration 2267 recorded resolver failure diagnostic in the driver and resolver typed IR contract path.

Iteration 2268 recorded generated-name conflict check in the driver and resolver typed IR contract path.

Iteration 2269 recorded source retention in the driver and resolver typed IR contract path.

Iteration 2270 recorded dependency source identity in the driver and resolver typed IR contract path.

Iteration 2271 recorded unknown adapter identity in the driver and resolver typed IR contract path.

Iteration 2272 recorded callable symbol mapping in the driver and resolver typed IR contract path.

Iteration 2273 recorded symbol table ownership in the driver and resolver typed IR contract path.

Iteration 2274 recorded type resolution ownership in the driver and resolver typed IR contract path.

Iteration 2275 recorded backend re-inference prohibition in the driver and resolver typed IR contract path.

Iteration 2276 recorded source location ownership in the driver and resolver typed IR contract path.

Iteration 2277 recorded driver compatibility path in the driver and resolver typed IR contract path.

Iteration 2278 recorded cross-backend parity in the driver and resolver typed IR contract path.

Iteration 2279 recorded integration test boundaries in the driver and resolver typed IR contract path.

Iteration 2280 recorded dependency regression tests in the driver and resolver typed IR contract path.

Iteration 2281 recorded full-suite categories in the driver and resolver typed IR contract path.

Iteration 2282 recorded typed IR API contract in the driver and resolver typed IR contract path.

Iteration 2283 recorded AST-free driver seam in the driver and resolver typed IR contract path.

Iteration 2284 recorded migration milestones in the driver and resolver typed IR contract path.

Iteration 2285 audited typed assignment output in the JVM typed statement semantics path.

Iteration 2286 audited typed expression output in the JVM typed statement semantics path.

Iteration 2287 audited typed loop output in the JVM typed statement semantics path.

Iteration 2288 audited typed IF output in the JVM typed statement semantics path.

Iteration 2289 audited typed SELECT output in the JVM typed statement semantics path.

Iteration 2290 audited typed DATA handling in the JVM typed statement semantics path.

Iteration 2291 audited typed RESTORE handling in the JVM typed statement semantics path.

Iteration 2292 audited typed file I/O output in the JVM typed statement semantics path.

Iteration 2293 audited typed record access in the JVM typed statement semantics path.

Iteration 2294 audited typed array access in the JVM typed statement semantics path.

Iteration 2295 audited typed callable body in the JVM typed statement semantics path.

Iteration 2296 audited typed procedure body in the JVM typed statement semantics path.

Iteration 2297 audited typed global storage in the JVM typed statement semantics path.

Iteration 2298 audited typed constants in the JVM typed statement semantics path.

Iteration 2299 audited typed catch bindings in the JVM typed statement semantics path.

Iteration 2300 audited typed exception handlers in the JVM typed statement semantics path.

Iteration 2301 audited typed generated names in the JVM typed statement semantics path.

Iteration 2302 audited typed source comments in the JVM typed statement semantics path.

Iteration 2303 audited implicit return in the JVM typed statement semantics path.

Iteration 2304 audited explicit END in the JVM typed statement semantics path.

Iteration 2305 audited label resolution in the JVM typed statement semantics path.

Iteration 2306 audited TRY nesting in the JVM typed statement semantics path.

Iteration 2307 audited semantic state ownership in the JVM typed statement semantics path.

Iteration 2308 audited target capability check in the JVM typed statement semantics path.

Iteration 2309 audited runtime conformance in the JVM typed statement semantics path.

Iteration 2310 traced typed scalar storage in the C typed expression and storage semantics path.

Iteration 2311 traced typed string storage in the C typed expression and storage semantics path.

Iteration 2312 traced typed global storage in the C typed expression and storage semantics path.

Iteration 2313 traced typed constant type in the C typed expression and storage semantics path.

Iteration 2314 traced typed array bounds in the C typed expression and storage semantics path.

Iteration 2315 traced typed array dimensions in the C typed expression and storage semantics path.

Iteration 2316 traced typed FOR variable in the C typed expression and storage semantics path.

Iteration 2317 traced typed assignment target in the C typed expression and storage semantics path.

Iteration 2318 traced typed member access in the C typed expression and storage semantics path.

Iteration 2319 traced typed record storage in the C typed expression and storage semantics path.

Iteration 2320 traced typed catch binding in the C typed expression and storage semantics path.

Iteration 2321 traced typed callable parameter in the C typed expression and storage semantics path.

Iteration 2322 traced typed callable return in the C typed expression and storage semantics path.

Iteration 2323 traced typed arithmetic conversion in the C typed expression and storage semantics path.

Iteration 2324 traced typed string conversion in the C typed expression and storage semantics path.

Iteration 2325 traced typed array indexing in the C typed expression and storage semantics path.

Iteration 2326 traced typed DATA initializer in the C typed expression and storage semantics path.

Iteration 2327 traced typed READ target in the C typed expression and storage semantics path.

Iteration 2328 traced typed INPUT target in the C typed expression and storage semantics path.

Iteration 2329 traced typed file channel in the C typed expression and storage semantics path.

Iteration 2330 traced typed FIELD buffer in the C typed expression and storage semantics path.

Iteration 2331 traced typed GOSUB state in the C typed expression and storage semantics path.

Iteration 2332 traced typed function fallthrough in the C typed expression and storage semantics path.

Iteration 2333 traced typed builtin usage in the C typed expression and storage semantics path.

Iteration 2334 traced generated C runtime behavior in the C typed expression and storage semantics path.

Iteration 2335 checked typed scalar names in the BASIC typed expression and storage semantics path.

Iteration 2336 checked typed string names in the BASIC typed expression and storage semantics path.

Iteration 2337 checked typed global binding in the BASIC typed expression and storage semantics path.

Iteration 2338 checked typed constant suffix in the BASIC typed expression and storage semantics path.

Iteration 2339 checked typed array bounds in the BASIC typed expression and storage semantics path.

Iteration 2340 checked typed array declarations in the BASIC typed expression and storage semantics path.

Iteration 2341 checked typed FOR variable in the BASIC typed expression and storage semantics path.

Iteration 2342 checked typed assignment targets in the BASIC typed expression and storage semantics path.

Iteration 2343 checked typed record fields in the BASIC typed expression and storage semantics path.

Iteration 2344 checked typed catch names in the BASIC typed expression and storage semantics path.

Iteration 2345 checked typed callable arguments in the BASIC typed expression and storage semantics path.

Iteration 2346 checked typed callable return in the BASIC typed expression and storage semantics path.

Iteration 2347 checked typed numeric expressions in the BASIC typed expression and storage semantics path.

Iteration 2348 checked typed string expressions in the BASIC typed expression and storage semantics path.

Iteration 2349 checked typed array indexing in the BASIC typed expression and storage semantics path.

Iteration 2350 checked typed DATA values in the BASIC typed expression and storage semantics path.

Iteration 2351 checked typed READ targets in the BASIC typed expression and storage semantics path.

Iteration 2352 checked typed INPUT targets in the BASIC typed expression and storage semantics path.

Iteration 2353 checked typed file channels in the BASIC typed expression and storage semantics path.

Iteration 2354 checked typed FIELD layout in the BASIC typed expression and storage semantics path.

Iteration 2355 checked typed labels in the BASIC typed expression and storage semantics path.

Iteration 2356 checked typed builtin calls in the BASIC typed expression and storage semantics path.

Iteration 2357 checked typed function fallthrough in the BASIC typed expression and storage semantics path.

Iteration 2358 checked generated BASIC runtime in the BASIC typed expression and storage semantics path.

Iteration 2359 checked backend parity in the BASIC typed expression and storage semantics path.

Iteration 2360 reviewed shared semantic module roots in the cross-backend and driver migration audit path.

Iteration 2361 reviewed dependency source mapping in the cross-backend and driver migration audit path.

Iteration 2362 reviewed callable signatures in the cross-backend and driver migration audit path.

Iteration 2363 reviewed record declarations in the cross-backend and driver migration audit path.

Iteration 2364 reviewed record method symbols in the cross-backend and driver migration audit path.

Iteration 2365 reviewed global declarations in the cross-backend and driver migration audit path.

Iteration 2366 reviewed constant declarations in the cross-backend and driver migration audit path.

Iteration 2367 reviewed array declarations in the cross-backend and driver migration audit path.

Iteration 2368 reviewed control-flow facts in the cross-backend and driver migration audit path.

Iteration 2369 reviewed exception metadata in the cross-backend and driver migration audit path.

Iteration 2370 reviewed DATA offsets in the cross-backend and driver migration audit path.

Iteration 2371 reviewed file layout facts in the cross-backend and driver migration audit path.

Iteration 2372 reviewed generated-name conflicts in the cross-backend and driver migration audit path.

Iteration 2373 reviewed target support flags in the cross-backend and driver migration audit path.

Iteration 2374 reviewed source diagnostics in the cross-backend and driver migration audit path.

Iteration 2375 reviewed resolver-produced expression types in the cross-backend and driver migration audit path.

Iteration 2376 reviewed resolver-produced lvalue types in the cross-backend and driver migration audit path.

Iteration 2377 reviewed semantic spans in the cross-backend and driver migration audit path.

Iteration 2378 reviewed adapter source retention in the cross-backend and driver migration audit path.

Iteration 2379 reviewed BASIC consumer inventory in the cross-backend and driver migration audit path.

Iteration 2380 reviewed C consumer inventory in the cross-backend and driver migration audit path.

Iteration 2381 reviewed JVM consumer inventory in the cross-backend and driver migration audit path.

Iteration 2382 reviewed driver consumer inventory in the cross-backend and driver migration audit path.

Iteration 2383 reviewed AST fallback inventory in the cross-backend and driver migration audit path.

Iteration 2384 reviewed remaining module boundaries in the cross-backend and driver migration audit path.

Iteration 2385 recorded focused JVM stream result in the validation and checkpoint review path.

Iteration 2386 recorded blank-line regression result in the validation and checkpoint review path.

Iteration 2387 recorded multi-source regression result in the validation and checkpoint review path.

Iteration 2388 recorded atomic decline regression result in the validation and checkpoint review path.

Iteration 2389 recorded library tests in the validation and checkpoint review path.

Iteration 2390 recorded CLI tests in the validation and checkpoint review path.

Iteration 2391 recorded DOSBox/C tests in the validation and checkpoint review path.

Iteration 2392 recorded example tests in the validation and checkpoint review path.

Iteration 2393 recorded JVM conformance tests in the validation and checkpoint review path.

Iteration 2394 recorded language conformance test in the validation and checkpoint review path.

Iteration 2395 recorded general record tests in the validation and checkpoint review path.

Iteration 2396 recorded record method tests in the validation and checkpoint review path.

Iteration 2397 recorded tutorial fixture scope in the validation and checkpoint review path.

Iteration 2398 recorded Graphify state at start in the validation and checkpoint review path.

Iteration 2399 recorded Graphify query results in the validation and checkpoint review path.

Iteration 2400 recorded source inspection after navigation in the validation and checkpoint review path.

Iteration 2401 recorded whitespace validation in the validation and checkpoint review path.

Iteration 2402 recorded number continuity in the validation and checkpoint review path.

Iteration 2403 recorded number uniqueness in the validation and checkpoint review path.

Iteration 2404 recorded generated files in the validation and checkpoint review path.

Iteration 2405 recorded commit scope in the validation and checkpoint review path.

Iteration 2406 recorded worktree state in the validation and checkpoint review path.

Iteration 2407 recorded C typed-stream gap in the validation and checkpoint review path.

Iteration 2408 recorded driver typed IR gap in the validation and checkpoint review path.

Iteration 2409 verified JVM module emission declines atomically when source identity is missing.

Iteration 2410 traced semantic span fields in the C typed source-location ownership workstream.

Iteration 2411 traced source-index contract in the C typed source-location ownership workstream.

Iteration 2412 traced source-text byte offsets in the C typed source-location ownership workstream.

Iteration 2413 traced UTF-8 boundary handling in the C typed source-location ownership workstream.

Iteration 2414 traced line calculation in the C typed source-location ownership workstream.

Iteration 2415 traced column calculation in the C typed source-location ownership workstream.

Iteration 2416 traced filename selection in the C typed source-location ownership workstream.

Iteration 2417 traced THROW source location in the C typed source-location ownership workstream.

Iteration 2418 traced OPEN source location in the C typed source-location ownership workstream.

Iteration 2419 traced nested TRY source inheritance in the C typed source-location ownership workstream.

Iteration 2420 traced dependency source locations in the C typed source-location ownership workstream.

Iteration 2421 traced root-module source locations in the C typed source-location ownership workstream.

Iteration 2422 traced record-generated statement locations in the C typed source-location ownership workstream.

Iteration 2423 traced unsupported-node diagnostics in the C typed source-location ownership workstream.

Iteration 2424 traced AST-aligned location comparison in the C typed source-location ownership workstream.

Iteration 2425 traced source-map failure behavior in the C typed source-location ownership workstream.

Iteration 2426 traced unknown adapter source behavior in the C typed source-location ownership workstream.

Iteration 2427 traced fallback location ownership in the C typed source-location ownership workstream.

Iteration 2428 traced diagnostic display filename in the C typed source-location ownership workstream.

Iteration 2429 traced source-position test seams in the C typed source-location ownership workstream.

Iteration 2430 traced shared source-position helper opportunity in the C typed source-location ownership workstream.

Iteration 2431 traced semantic IR API dependencies in the C typed source-location ownership workstream.

Iteration 2432 traced resolver span preservation in the C typed source-location ownership workstream.

Iteration 2433 traced driver source retention in the C typed source-location ownership workstream.

Iteration 2434 traced AST-free dispatch prerequisites in the C typed source-location ownership workstream.

Iteration 2435 inspected AST statement iteration in the C top-level semantic dispatcher workstream.

Iteration 2436 inspected semantic root iteration in the C top-level semantic dispatcher workstream.

Iteration 2437 inspected source alignment helper in the C top-level semantic dispatcher workstream.

Iteration 2438 inspected per-statement output buffer in the C top-level semantic dispatcher workstream.

Iteration 2439 inspected unsupported semantic root handling in the C top-level semantic dispatcher workstream.

Iteration 2440 inspected compatibility AST fallback in the C top-level semantic dispatcher workstream.

Iteration 2441 inspected transactional codegen state in the C top-level semantic dispatcher workstream.

Iteration 2442 inspected GOSUB counter rollback in the C top-level semantic dispatcher workstream.

Iteration 2443 inspected FILE layout state rollback in the C top-level semantic dispatcher workstream.

Iteration 2444 inspected TRY identifier order in the C top-level semantic dispatcher workstream.

Iteration 2445 inspected DATA offset order in the C top-level semantic dispatcher workstream.

Iteration 2446 inspected label collection dependencies in the C top-level semantic dispatcher workstream.

Iteration 2447 inspected global declaration prepass in the C top-level semantic dispatcher workstream.

Iteration 2448 inspected callable prepass in the C top-level semantic dispatcher workstream.

Iteration 2449 inspected C89 declaration hoisting in the C top-level semantic dispatcher workstream.

Iteration 2450 inspected record FIELD ordering in the C top-level semantic dispatcher workstream.

Iteration 2451 inspected implicit END handling in the C top-level semantic dispatcher workstream.

Iteration 2452 inspected statement-specific AST dependencies in the C top-level semantic dispatcher workstream.

Iteration 2453 inspected OPEN AST position dependency in the C top-level semantic dispatcher workstream.

Iteration 2454 inspected THROW AST position dependency in the C top-level semantic dispatcher workstream.

Iteration 2455 inspected line-oriented semantic roots in the C top-level semantic dispatcher workstream.

Iteration 2456 inspected nested block emission in the C top-level semantic dispatcher workstream.

Iteration 2457 inspected target capability checks in the C top-level semantic dispatcher workstream.

Iteration 2458 inspected diagnostic preservation in the C top-level semantic dispatcher workstream.

Iteration 2459 inspected typed-stream extraction seam in the C top-level semantic dispatcher workstream.

Iteration 2460 audited `compile_source` parsing path in the driver typed IR consumers workstream.

Iteration 2461 audited `compile_file` parsing path in the driver typed IR consumers workstream.

Iteration 2462 audited recursive REQUIRE loading in the driver typed IR consumers workstream.

Iteration 2463 audited semantic module merge in the driver typed IR consumers workstream.

Iteration 2464 audited compatibility AST construction in the driver typed IR consumers workstream.

Iteration 2465 audited record transpilation order in the driver typed IR consumers workstream.

Iteration 2466 audited resolved-program creation in the driver typed IR consumers workstream.

Iteration 2467 audited backend dispatch in the driver typed IR consumers workstream.

Iteration 2468 audited C backend input contract in the driver typed IR consumers workstream.

Iteration 2469 audited BASIC backend input contract in the driver typed IR consumers workstream.

Iteration 2470 audited JVM backend input contract in the driver typed IR consumers workstream.

Iteration 2471 audited generated-name conflict validation in the driver typed IR consumers workstream.

Iteration 2472 audited dependency source preservation in the driver typed IR consumers workstream.

Iteration 2473 audited semantic source retention in the driver typed IR consumers workstream.

Iteration 2474 audited resolver diagnostics in the driver typed IR consumers workstream.

Iteration 2475 audited driver warning paths in the driver typed IR consumers workstream.

Iteration 2476 audited C64 target dispatch in the driver typed IR consumers workstream.

Iteration 2477 audited output path generation in the driver typed IR consumers workstream.

Iteration 2478 audited cached output behavior in the driver typed IR consumers workstream.

Iteration 2479 audited CLI integration coverage in the driver typed IR consumers workstream.

Iteration 2480 audited library API coverage in the driver typed IR consumers workstream.

Iteration 2481 audited multi-file integration coverage in the driver typed IR consumers workstream.

Iteration 2482 audited AST-only caller compatibility in the driver typed IR consumers workstream.

Iteration 2483 audited typed-IR-only future API in the driver typed IR consumers workstream.

Iteration 2484 audited migration dependency order in the driver typed IR consumers workstream.

Iteration 2485 reviewed top-level semantic stream in the BASIC typed IR consumer audit workstream.

Iteration 2486 reviewed per-statement fallback in the BASIC typed IR consumer audit workstream.

Iteration 2487 reviewed source map requirements in the BASIC typed IR consumer audit workstream.

Iteration 2488 reviewed statement type dispatch in the BASIC typed IR consumer audit workstream.

Iteration 2489 reviewed callable body dispatch in the BASIC typed IR consumer audit workstream.

Iteration 2490 reviewed callable signature mapping in the BASIC typed IR consumer audit workstream.

Iteration 2491 reviewed typed parameter storage in the BASIC typed IR consumer audit workstream.

Iteration 2492 reviewed typed global storage in the BASIC typed IR consumer audit workstream.

Iteration 2493 reviewed typed constant storage in the BASIC typed IR consumer audit workstream.

Iteration 2494 reviewed typed array storage in the BASIC typed IR consumer audit workstream.

Iteration 2495 reviewed record field access in the BASIC typed IR consumer audit workstream.

Iteration 2496 reviewed record method calls in the BASIC typed IR consumer audit workstream.

Iteration 2497 reviewed catch binding emission in the BASIC typed IR consumer audit workstream.

Iteration 2498 reviewed TRY source metadata in the BASIC typed IR consumer audit workstream.

Iteration 2499 reviewed DATA labels in the BASIC typed IR consumer audit workstream.

Iteration 2500 reviewed RESTORE offsets in the BASIC typed IR consumer audit workstream.

Iteration 2501 reviewed file channel layouts in the BASIC typed IR consumer audit workstream.

Iteration 2502 reviewed FIELD mutations in the BASIC typed IR consumer audit workstream.

Iteration 2503 reviewed GOSUB sequence in the BASIC typed IR consumer audit workstream.

Iteration 2504 reviewed loop state in the BASIC typed IR consumer audit workstream.

Iteration 2505 reviewed label resolution in the BASIC typed IR consumer audit workstream.

Iteration 2506 reviewed generated comments in the BASIC typed IR consumer audit workstream.

Iteration 2507 reviewed implicit termination in the BASIC typed IR consumer audit workstream.

Iteration 2508 reviewed target dialect behavior in the BASIC typed IR consumer audit workstream.

Iteration 2509 reviewed AST-only fallback in the BASIC typed IR consumer audit workstream.

Iteration 2510 recorded whole-module typed stream in the JVM typed IR consumer audit workstream.

Iteration 2511 recorded per-root source selection in the JVM typed IR consumer audit workstream.

Iteration 2512 recorded blank-line independence in the JVM typed IR consumer audit workstream.

Iteration 2513 recorded multi-source dependency order in the JVM typed IR consumer audit workstream.

Iteration 2514 recorded unsupported-node fallback in the JVM typed IR consumer audit workstream.

Iteration 2515 recorded state rollback in the JVM typed IR consumer audit workstream.

Iteration 2516 recorded loop stack rollback in the JVM typed IR consumer audit workstream.

Iteration 2517 recorded label sequence rollback in the JVM typed IR consumer audit workstream.

Iteration 2518 recorded exception-table rollback in the JVM typed IR consumer audit workstream.

Iteration 2519 recorded callable typed bodies in the JVM typed IR consumer audit workstream.

Iteration 2520 recorded typed callable signatures in the JVM typed IR consumer audit workstream.

Iteration 2521 recorded typed global slots in the JVM typed IR consumer audit workstream.

Iteration 2522 recorded typed local slots in the JVM typed IR consumer audit workstream.

Iteration 2523 recorded array declarations in the JVM typed IR consumer audit workstream.

Iteration 2524 recorded record declarations in the JVM typed IR consumer audit workstream.

Iteration 2525 recorded record file layouts in the JVM typed IR consumer audit workstream.

Iteration 2526 recorded DATA pool collection in the JVM typed IR consumer audit workstream.

Iteration 2527 recorded file helper requirements in the JVM typed IR consumer audit workstream.

Iteration 2528 recorded INKEY helper requirements in the JVM typed IR consumer audit workstream.

Iteration 2529 recorded TRY handler names in the JVM typed IR consumer audit workstream.

Iteration 2530 recorded source filename constants in the JVM typed IR consumer audit workstream.

Iteration 2531 recorded generated comments in the JVM typed IR consumer audit workstream.

Iteration 2532 recorded module END behavior in the JVM typed IR consumer audit workstream.

Iteration 2533 recorded AST support metadata in the JVM typed IR consumer audit workstream.

Iteration 2534 recorded remaining AST scans in the JVM typed IR consumer audit workstream.

Iteration 2535 traced C temporary counter in the C semantic state preservation workstream.

Iteration 2536 traced math helper flags in the C semantic state preservation workstream.

Iteration 2537 traced string helper flags in the C semantic state preservation workstream.

Iteration 2538 traced array metadata in the C semantic state preservation workstream.

Iteration 2539 traced function map in the C semantic state preservation workstream.

Iteration 2540 traced method map in the C semantic state preservation workstream.

Iteration 2541 traced record layout table in the C semantic state preservation workstream.

Iteration 2542 traced file channel generations in the C semantic state preservation workstream.

Iteration 2543 traced FIELD width state in the C semantic state preservation workstream.

Iteration 2544 traced pending record values in the C semantic state preservation workstream.

Iteration 2545 traced GOSUB state in the C semantic state preservation workstream.

Iteration 2546 traced ON ERROR handler IDs in the C semantic state preservation workstream.

Iteration 2547 traced TRY dispatch IDs in the C semantic state preservation workstream.

Iteration 2548 traced raise-site numbering in the C semantic state preservation workstream.

Iteration 2549 traced DATA cursor state in the C semantic state preservation workstream.

Iteration 2550 traced label set in the C semantic state preservation workstream.

Iteration 2551 traced loop continue stack in the C semantic state preservation workstream.

Iteration 2552 traced function TRY reachability in the C semantic state preservation workstream.

Iteration 2553 traced global type map in the C semantic state preservation workstream.

Iteration 2554 traced typed constant map in the C semantic state preservation workstream.

Iteration 2555 traced builtin usage scan in the C semantic state preservation workstream.

Iteration 2556 traced C dialect profile in the C semantic state preservation workstream.

Iteration 2557 traced C89 declaration constraints in the C semantic state preservation workstream.

Iteration 2558 traced atomic fallback snapshot in the C semantic state preservation workstream.

Iteration 2559 traced output buffer commit in the C semantic state preservation workstream.

Iteration 2560 inspected integer suffix facts in the cross-backend typed semantics workstream.

Iteration 2561 inspected LONG suffix facts in the cross-backend typed semantics workstream.

Iteration 2562 inspected SINGLE suffix facts in the cross-backend typed semantics workstream.

Iteration 2563 inspected DOUBLE suffix facts in the cross-backend typed semantics workstream.

Iteration 2564 inspected STRING suffix facts in the cross-backend typed semantics workstream.

Iteration 2565 inspected BOOLEAN facts in the cross-backend typed semantics workstream.

Iteration 2566 inspected expression result types in the cross-backend typed semantics workstream.

Iteration 2567 inspected lvalue result types in the cross-backend typed semantics workstream.

Iteration 2568 inspected array element types in the cross-backend typed semantics workstream.

Iteration 2569 inspected callable parameter types in the cross-backend typed semantics workstream.

Iteration 2570 inspected callable result types in the cross-backend typed semantics workstream.

Iteration 2571 inspected catch binding types in the cross-backend typed semantics workstream.

Iteration 2572 inspected record member types in the cross-backend typed semantics workstream.

Iteration 2573 inspected record field widths in the cross-backend typed semantics workstream.

Iteration 2574 inspected record file layouts in the cross-backend typed semantics workstream.

Iteration 2575 inspected DATA literal types in the cross-backend typed semantics workstream.

Iteration 2576 inspected FOR variable types in the cross-backend typed semantics workstream.

Iteration 2577 inspected global binding types in the cross-backend typed semantics workstream.

Iteration 2578 inspected constant binding types in the cross-backend typed semantics workstream.

Iteration 2579 inspected source filename facts in the cross-backend typed semantics workstream.

Iteration 2580 inspected target capability facts in the cross-backend typed semantics workstream.

Iteration 2581 inspected shared name resolution in the cross-backend typed semantics workstream.

Iteration 2582 inspected shared diagnostic ownership in the cross-backend typed semantics workstream.

Iteration 2583 inspected generated code parity in the cross-backend typed semantics workstream.

Iteration 2584 inspected backend-specific output tests in the cross-backend typed semantics workstream.

Iteration 2585 audited atomic JVM decline case in the regression and test design workstream.

Iteration 2586 audited invalid source index fixture in the regression and test design workstream.

Iteration 2587 audited partial typed output absence in the regression and test design workstream.

Iteration 2588 audited AST fallback output presence in the regression and test design workstream.

Iteration 2589 audited typed output exclusion assertion in the regression and test design workstream.

Iteration 2590 audited blank-line typed JVM coverage in the regression and test design workstream.

Iteration 2591 audited multi-source JVM coverage in the regression and test design workstream.

Iteration 2592 audited dependency filename assertion in the regression and test design workstream.

Iteration 2593 audited root filename assertion in the regression and test design workstream.

Iteration 2594 audited TRY metadata assertion in the regression and test design workstream.

Iteration 2595 audited C END source-aligned case in the regression and test design workstream.

Iteration 2596 audited C THROW source-position case in the regression and test design workstream.

Iteration 2597 audited C OPEN source-position case in the regression and test design workstream.

Iteration 2598 audited C AST fallback case in the regression and test design workstream.

Iteration 2599 audited BASIC AST-independent stream case in the regression and test design workstream.

Iteration 2600 audited driver dependency-order case in the regression and test design workstream.

Iteration 2601 audited cross-backend output comparison in the regression and test design workstream.

Iteration 2602 audited runtime behavior validation in the regression and test design workstream.

Iteration 2603 audited generated fixture determinism in the regression and test design workstream.

Iteration 2604 audited test helper isolation in the regression and test design workstream.

Iteration 2605 audited expected diagnostic coverage in the regression and test design workstream.

Iteration 2606 audited legacy API compatibility in the regression and test design workstream.

Iteration 2607 audited focused test command in the regression and test design workstream.

Iteration 2608 audited full test command in the regression and test design workstream.

Iteration 2609 audited test output review in the regression and test design workstream.

Iteration 2610 reviewed minimum C source-location refactor in the migration plan and risk review workstream.

Iteration 2611 reviewed typed source-position API shape in the migration plan and risk review workstream.

Iteration 2612 reviewed fallback eligibility contract in the migration plan and risk review workstream.

Iteration 2613 reviewed semantic root iterator shape in the migration plan and risk review workstream.

Iteration 2614 reviewed atomic C emission requirements in the migration plan and risk review workstream.

Iteration 2615 reviewed C state snapshot inventory in the migration plan and risk review workstream.

Iteration 2616 reviewed unsupported statement policy in the migration plan and risk review workstream.

Iteration 2617 reviewed AST compatibility sunset boundary in the migration plan and risk review workstream.

Iteration 2618 reviewed driver API migration sequence in the migration plan and risk review workstream.

Iteration 2619 reviewed BASIC migration dependencies in the migration plan and risk review workstream.

Iteration 2620 reviewed JVM remaining AST dependencies in the migration plan and risk review workstream.

Iteration 2621 reviewed backend parity milestones in the migration plan and risk review workstream.

Iteration 2622 reviewed resolver ownership boundary in the migration plan and risk review workstream.

Iteration 2623 reviewed typed IR producer requirements in the migration plan and risk review workstream.

Iteration 2624 reviewed source-index invariant checks in the migration plan and risk review workstream.

Iteration 2625 reviewed multi-file test fixture design in the migration plan and risk review workstream.

Iteration 2626 reviewed record transpilation interaction in the migration plan and risk review workstream.

Iteration 2627 reviewed C89 target interaction in the migration plan and risk review workstream.

Iteration 2628 reviewed DOSBox/C validation need in the migration plan and risk review workstream.

Iteration 2629 reviewed JVM runtime validation need in the migration plan and risk review workstream.

Iteration 2630 reviewed BASIC dialect validation need in the migration plan and risk review workstream.

Iteration 2631 reviewed Graphify refresh trigger in the migration plan and risk review workstream.

Iteration 2632 reviewed generated graph tracking rule in the migration plan and risk review workstream.

Iteration 2633 reviewed commit checkpoint scope in the migration plan and risk review workstream.

Iteration 2634 reviewed next iteration acceptance criteria in the migration plan and risk review workstream.

Iteration 2635 recorded repository clean state in the batch validation and reporting workstream.

Iteration 2636 recorded checkpoint commit identity in the batch validation and reporting workstream.

Iteration 2637 recorded Graphify graph version in the batch validation and reporting workstream.

Iteration 2638 recorded Graphify C path result in the batch validation and reporting workstream.

Iteration 2639 recorded C source inspection result in the batch validation and reporting workstream.

Iteration 2640 recorded driver path inspection result in the batch validation and reporting workstream.

Iteration 2641 recorded BASIC consumer inventory in the batch validation and reporting workstream.

Iteration 2642 recorded JVM consumer inventory in the batch validation and reporting workstream.

Iteration 2643 recorded C dispatcher inventory in the batch validation and reporting workstream.

Iteration 2644 recorded test inventory in the batch validation and reporting workstream.

Iteration 2645 recorded atomic-fallback test result in the batch validation and reporting workstream.

Iteration 2646 recorded focused JVM test result in the batch validation and reporting workstream.

Iteration 2647 recorded full library test result in the batch validation and reporting workstream.

Iteration 2648 recorded CLI test result in the batch validation and reporting workstream.

Iteration 2649 recorded DOSBox/C test result in the batch validation and reporting workstream.

Iteration 2650 recorded example test result in the batch validation and reporting workstream.

Iteration 2651 recorded JVM conformance result in the batch validation and reporting workstream.

Iteration 2652 recorded language test result in the batch validation and reporting workstream.

Iteration 2653 recorded record test result in the batch validation and reporting workstream.

Iteration 2654 recorded whitespace check result in the batch validation and reporting workstream.

Iteration 2655 recorded plan sequence continuity in the batch validation and reporting workstream.

Iteration 2656 recorded plan number uniqueness in the batch validation and reporting workstream.

Iteration 2657 recorded remaining C gap statement in the batch validation and reporting workstream.

Iteration 2658 recorded remaining driver gap statement in the batch validation and reporting workstream.

Iteration 2659 verified C semantic dispatch declines when source identity is missing.

Iteration 2660 implemented `SemanticSource::source_position` API in the shared semantic source position contract workstream.

Iteration 2661 implemented semantic `SourceSpan` input in the shared semantic source position contract workstream.

Iteration 2662 verified span start clamping in `SemanticSource::source_position`.

Iteration 2663 verified span end clamping in `SemanticSource::source_position`.

Iteration 2664 verified UTF-8 boundary normalization in `SemanticSource::source_position`.

Iteration 2665 verified leading trivia skipping in `SemanticSource::source_position`.

Iteration 2666 verified line and Unicode column calculation in `SemanticSource::source_position`.

Iteration 2667 implemented Unicode column calculation in the shared semantic source position contract workstream.

Iteration 2668 implemented filename preservation in the shared semantic source position contract workstream.

Iteration 2669 implemented empty span handling in the shared semantic source position contract workstream.

Iteration 2670 implemented end-before-start handling in the shared semantic source position contract workstream.

Iteration 2671 implemented span beyond source handling in the shared semantic source position contract workstream.

Iteration 2672 implemented CRLF source handling in the shared semantic source position contract workstream.

Iteration 2673 implemented tabs in leading trivia in the shared semantic source position contract workstream.

Iteration 2674 implemented multibyte prefix handling in the shared semantic source position contract workstream.

Iteration 2675 implemented invalid byte-offset recovery in the shared semantic source position contract workstream.

Iteration 2676 implemented diagnostic source-position use in the shared semantic source position contract workstream.

Iteration 2677 migrated C semantic diagnostics to `SemanticSource::source_position`.

Iteration 2678 migrated driver semantic diagnostics to `SemanticSource::source_position`.

Iteration 2679 implemented semantic source ownership in the shared semantic source position contract workstream.

Iteration 2680 implemented AST position independence in the shared semantic source position contract workstream.

Iteration 2681 implemented source index responsibility in the shared semantic source position contract workstream.

Iteration 2682 implemented nested statement span contract in the shared semantic source position contract workstream.

Iteration 2683 implemented generated span contract in the shared semantic source position contract workstream.

Iteration 2684 implemented consumer migration seam in the shared semantic source position contract workstream.

Iteration 2685 migrated FBC incompatibility diagnostics in the driver diagnostic integration workstream.

Iteration 2686 migrated recursive REQUIRE source diagnostics in the driver diagnostic integration workstream.

Iteration 2687 migrated semantic source lookup in the driver diagnostic integration workstream.

Iteration 2688 migrated missing source fallback in the driver diagnostic integration workstream.

Iteration 2689 migrated diagnostic filename display in the driver diagnostic integration workstream.

Iteration 2690 migrated line and column parity in the driver diagnostic integration workstream.

Iteration 2691 migrated legacy AST diagnostics in the driver diagnostic integration workstream.

Iteration 2692 migrated typed IR diagnostic precedence in the driver diagnostic integration workstream.

Iteration 2693 migrated compile-source behavior in the driver diagnostic integration workstream.

Iteration 2694 migrated compile-file behavior in the driver diagnostic integration workstream.

Iteration 2695 migrated dependency filename behavior in the driver diagnostic integration workstream.

Iteration 2696 migrated root filename behavior in the driver diagnostic integration workstream.

Iteration 2697 migrated record transpilation diagnostics in the driver diagnostic integration workstream.

Iteration 2698 migrated unsupported target diagnostics in the driver diagnostic integration workstream.

Iteration 2699 migrated warning position preservation in the driver diagnostic integration workstream.

Iteration 2700 migrated test fixture source path in the driver diagnostic integration workstream.

Iteration 2701 migrated source position wrapper removal in the driver diagnostic integration workstream.

Iteration 2702 migrated shared helper call site in the driver diagnostic integration workstream.

Iteration 2703 migrated source span producer in the driver diagnostic integration workstream.

Iteration 2704 migrated generated frontend parser in the driver diagnostic integration workstream.

Iteration 2705 migrated resolver diagnostic boundary in the driver diagnostic integration workstream.

Iteration 2706 migrated CLI diagnostic rendering in the driver diagnostic integration workstream.

Iteration 2707 migrated multi-file diagnostics in the driver diagnostic integration workstream.

Iteration 2708 migrated UTF-8 diagnostics in the driver diagnostic integration workstream.

Iteration 2709 migrated driver validation in the driver diagnostic integration workstream.

Iteration 2710 verified classic ERROR handling diagnostic in the C backend semantic diagnostic integration workstream.

Iteration 2711 verified ON ERROR diagnostic in the C backend semantic diagnostic integration workstream.

Iteration 2712 verified RESUME diagnostic in the C backend semantic diagnostic integration workstream.

Iteration 2713 verified nested TRY traversal in the C backend semantic diagnostic integration workstream.

Iteration 2714 verified nested IF traversal in the C backend semantic diagnostic integration workstream.

Iteration 2715 verified loop body traversal in the C backend semantic diagnostic integration workstream.

Iteration 2716 verified SELECT body traversal in the C backend semantic diagnostic integration workstream.

Iteration 2717 verified semantic span lookup in the C backend semantic diagnostic integration workstream.

Iteration 2718 verified shared source position method in the C backend semantic diagnostic integration workstream.

Iteration 2719 verified C unsupported float diagnostic in the C backend semantic diagnostic integration workstream.

Iteration 2720 verified semantic expression spans in the C backend semantic diagnostic integration workstream.

Iteration 2721 verified THROW source ownership in the C backend semantic diagnostic integration workstream.

Iteration 2722 verified OPEN source ownership in the C backend semantic diagnostic integration workstream.

Iteration 2723 verified source-aligned dispatcher in the C backend semantic diagnostic integration workstream.

Iteration 2724 verified fallback statement positions in the C backend semantic diagnostic integration workstream.

Iteration 2725 verified missing source mapping in the C backend semantic diagnostic integration workstream.

Iteration 2726 verified invalid span robustness in the C backend semantic diagnostic integration workstream.

Iteration 2727 verified filename display behavior in the C backend semantic diagnostic integration workstream.

Iteration 2728 verified column calculation parity in the C backend semantic diagnostic integration workstream.

Iteration 2729 verified record member diagnostics in the C backend semantic diagnostic integration workstream.

Iteration 2730 verified typed expression diagnostics in the C backend semantic diagnostic integration workstream.

Iteration 2731 verified target capability diagnostics in the C backend semantic diagnostic integration workstream.

Iteration 2732 verified C64 diagnostic path in the C backend semantic diagnostic integration workstream.

Iteration 2733 verified regression test coverage in the C backend semantic diagnostic integration workstream.

Iteration 2734 verified AST dependency inventory in the C backend semantic diagnostic integration workstream.

Iteration 2735 audited dependency source vector order in the source identity and dependency handling workstream.

Iteration 2736 audited dependency source-index offset in the source identity and dependency handling workstream.

Iteration 2737 audited root source-index preservation in the source identity and dependency handling workstream.

Iteration 2738 audited statement/source count invariant in the source identity and dependency handling workstream.

Iteration 2739 audited unknown source sentinel in the source identity and dependency handling workstream.

Iteration 2740 audited missing source index behavior in the source identity and dependency handling workstream.

Iteration 2741 audited out-of-range source index behavior in the source identity and dependency handling workstream.

Iteration 2742 audited multi-source location selection in the source identity and dependency handling workstream.

Iteration 2743 audited nested node source inheritance in the source identity and dependency handling workstream.

Iteration 2744 audited dependency THROW location in the source identity and dependency handling workstream.

Iteration 2745 audited root THROW location in the source identity and dependency handling workstream.

Iteration 2746 audited dependency diagnostic path in the source identity and dependency handling workstream.

Iteration 2747 audited root diagnostic path in the source identity and dependency handling workstream.

Iteration 2748 audited semantic source text retention in the source identity and dependency handling workstream.

Iteration 2749 audited source filename uniqueness assumptions in the source identity and dependency handling workstream.

Iteration 2750 audited same filename source handling in the source identity and dependency handling workstream.

Iteration 2751 audited generated source names in the source identity and dependency handling workstream.

Iteration 2752 audited adapting standalone modules in the source identity and dependency handling workstream.

Iteration 2753 audited module prepend behavior in the source identity and dependency handling workstream.

Iteration 2754 audited resolver source contract in the source identity and dependency handling workstream.

Iteration 2755 audited codegen source contract in the source identity and dependency handling workstream.

Iteration 2756 audited driver source contract in the source identity and dependency handling workstream.

Iteration 2757 audited source map tests in the source identity and dependency handling workstream.

Iteration 2758 audited multi-file test design in the source identity and dependency handling workstream.

Iteration 2759 audited future typed-only entry point in the source identity and dependency handling workstream.

Iteration 2760 recorded BASIC semantic diagnostic locations in the BASIC and JVM position consumers workstream.

Iteration 2761 recorded BASIC source alignment mapping in the BASIC and JVM position consumers workstream.

Iteration 2762 recorded BASIC callable source mapping in the BASIC and JVM position consumers workstream.

Iteration 2763 recorded JVM TRY filename metadata in the BASIC and JVM position consumers workstream.

Iteration 2764 recorded JVM per-root filename switching in the BASIC and JVM position consumers workstream.

Iteration 2765 recorded JVM module source index checks in the BASIC and JVM position consumers workstream.

Iteration 2766 recorded JVM callable source identity in the BASIC and JVM position consumers workstream.

Iteration 2767 recorded JVM fallback policy in the BASIC and JVM position consumers workstream.

Iteration 2768 recorded typed comments source tracking in the BASIC and JVM position consumers workstream.

Iteration 2769 recorded typed nested block spans in the BASIC and JVM position consumers workstream.

Iteration 2770 recorded generated identifier diagnostics in the BASIC and JVM position consumers workstream.

Iteration 2771 recorded BASIC generated line numbers in the BASIC and JVM position consumers workstream.

Iteration 2772 recorded JVM line metadata limits in the BASIC and JVM position consumers workstream.

Iteration 2773 recorded shared SourcePos format in the BASIC and JVM position consumers workstream.

Iteration 2774 recorded backend display filename behavior in the BASIC and JVM position consumers workstream.

Iteration 2775 recorded source span retention through resolver in the BASIC and JVM position consumers workstream.

Iteration 2776 recorded dependency source remapping in the BASIC and JVM position consumers workstream.

Iteration 2777 recorded AST fallback positions in the BASIC and JVM position consumers workstream.

Iteration 2778 recorded invalid source behavior in the BASIC and JVM position consumers workstream.

Iteration 2779 recorded multi-source tests in the BASIC and JVM position consumers workstream.

Iteration 2780 recorded blank-line tests in the BASIC and JVM position consumers workstream.

Iteration 2781 recorded source encoding behavior in the BASIC and JVM position consumers workstream.

Iteration 2782 recorded Unicode positions in the BASIC and JVM position consumers workstream.

Iteration 2783 recorded future direct IR consumers in the BASIC and JVM position consumers workstream.

Iteration 2784 recorded cross-backend consistency in the BASIC and JVM position consumers workstream.

Iteration 2785 implemented source location producer stage in the typed IR architectural ownership workstream.

Iteration 2786 implemented source location consumer stages in the typed IR architectural ownership workstream.

Iteration 2787 implemented resolver source-map preservation in the typed IR architectural ownership workstream.

Iteration 2788 implemented adapter span preservation in the typed IR architectural ownership workstream.

Iteration 2789 implemented semantic IR source API in the typed IR architectural ownership workstream.

Iteration 2790 implemented backend AST independence target in the typed IR architectural ownership workstream.

Iteration 2791 implemented driver AST compatibility role in the typed IR architectural ownership workstream.

Iteration 2792 implemented backend output ownership in the typed IR architectural ownership workstream.

Iteration 2793 implemented diagnostic stage ownership in the typed IR architectural ownership workstream.

Iteration 2794 implemented generated code source metadata in the typed IR architectural ownership workstream.

Iteration 2795 implemented cross-backend shared facts in the typed IR architectural ownership workstream.

Iteration 2796 implemented target-specific source formatting in the typed IR architectural ownership workstream.

Iteration 2797 implemented no backend type re-inference in the typed IR architectural ownership workstream.

Iteration 2798 implemented typed name and type invariants in the typed IR architectural ownership workstream.

Iteration 2799 implemented symbol table preservation in the typed IR architectural ownership workstream.

Iteration 2800 implemented callable symbol facts in the typed IR architectural ownership workstream.

Iteration 2801 implemented record layout facts in the typed IR architectural ownership workstream.

Iteration 2802 implemented array shape facts in the typed IR architectural ownership workstream.

Iteration 2803 implemented TRY binding facts in the typed IR architectural ownership workstream.

Iteration 2804 implemented DATA label facts in the typed IR architectural ownership workstream.

Iteration 2805 implemented file layout facts in the typed IR architectural ownership workstream.

Iteration 2806 implemented dependency merge facts in the typed IR architectural ownership workstream.

Iteration 2807 implemented codegen context facts in the typed IR architectural ownership workstream.

Iteration 2808 implemented typed IR contract documentation in the typed IR architectural ownership workstream.

Iteration 2809 implemented migration acceptance criteria in the typed IR architectural ownership workstream.

Iteration 2810 added a regression for leading trivia and Unicode source positions.

Iteration 2811 added a regression for non-boundary UTF-8 span offsets.

Iteration 2812 migrated source-position non-boundary test in the validation and test coverage workstream.

Iteration 2813 migrated focused Semantic IR tests in the validation and test coverage workstream.

Iteration 2814 migrated driver diagnostic tests in the validation and test coverage workstream.

Iteration 2815 migrated C diagnostic tests in the validation and test coverage workstream.

Iteration 2816 migrated BASIC regression tests in the validation and test coverage workstream.

Iteration 2817 migrated JVM metadata tests in the validation and test coverage workstream.

Iteration 2818 migrated dependency tests in the validation and test coverage workstream.

Iteration 2819 migrated multi-source tests in the validation and test coverage workstream.

Iteration 2820 migrated invalid source index tests in the validation and test coverage workstream.

Iteration 2821 migrated full locked suite in the validation and test coverage workstream.

Iteration 2822 migrated library test group in the validation and test coverage workstream.

Iteration 2823 migrated CLI test group in the validation and test coverage workstream.

Iteration 2824 migrated DOSBox/C group in the validation and test coverage workstream.

Iteration 2825 migrated example group in the validation and test coverage workstream.

Iteration 2826 migrated JVM conformance group in the validation and test coverage workstream.

Iteration 2827 migrated language conformance group in the validation and test coverage workstream.

Iteration 2828 migrated record test groups in the validation and test coverage workstream.

Iteration 2829 migrated diff whitespace check in the validation and test coverage workstream.

Iteration 2830 migrated Graphify refresh in the validation and test coverage workstream.

Iteration 2831 migrated Graphify graph node count in the validation and test coverage workstream.

Iteration 2832 migrated generated artifact status in the validation and test coverage workstream.

Iteration 2833 migrated plan number check in the validation and test coverage workstream.

Iteration 2834 migrated worktree review in the validation and test coverage workstream.

Iteration 2835 verified semantic root iterator in the C dispatcher follow-up contract workstream.

Iteration 2836 verified source position lookup by source index in the C dispatcher follow-up contract workstream.

Iteration 2837 verified typed THROW emission without AST location in the C dispatcher follow-up contract workstream.

Iteration 2838 verified typed OPEN emission without AST location in the C dispatcher follow-up contract workstream.

Iteration 2839 verified per-root transactional output in the C dispatcher follow-up contract workstream.

Iteration 2840 verified GOSUB rollback boundary in the C dispatcher follow-up contract workstream.

Iteration 2841 verified TRY handler rollback boundary in the C dispatcher follow-up contract workstream.

Iteration 2842 verified file I/O layout rollback in the C dispatcher follow-up contract workstream.

Iteration 2843 verified DATA cursor preservation in the C dispatcher follow-up contract workstream.

Iteration 2844 verified label prepass preservation in the C dispatcher follow-up contract workstream.

Iteration 2845 verified unsupported root fallback in the C dispatcher follow-up contract workstream.

Iteration 2846 verified fallback location construction in the C dispatcher follow-up contract workstream.

Iteration 2847 verified compatibility AST candidate selection in the C dispatcher follow-up contract workstream.

Iteration 2848 verified one-to-many transpilation handling in the C dispatcher follow-up contract workstream.

Iteration 2849 verified AST-free typed dispatch eligibility in the C dispatcher follow-up contract workstream.

Iteration 2850 verified source spans for nested nodes in the C dispatcher follow-up contract workstream.

Iteration 2851 verified top-level root spans in the C dispatcher follow-up contract workstream.

Iteration 2852 verified record-generated statement mapping in the C dispatcher follow-up contract workstream.

Iteration 2853 verified statement ordering guarantees in the C dispatcher follow-up contract workstream.

Iteration 2854 verified C89 hoisting interaction in the C dispatcher follow-up contract workstream.

Iteration 2855 verified C dialect validation order in the C dispatcher follow-up contract workstream.

Iteration 2856 verified callable dispatch interaction in the C dispatcher follow-up contract workstream.

Iteration 2857 verified diagnostic behavior on decline in the C dispatcher follow-up contract workstream.

Iteration 2858 verified focused dispatcher tests in the C dispatcher follow-up contract workstream.

Iteration 2859 verified migration-sized extraction in the C dispatcher follow-up contract workstream.

Iteration 2860 audited generated frontend AST ownership in the driver typed-only pipeline follow-up workstream.

Iteration 2861 audited compatibility parser call sites in the driver typed-only pipeline follow-up workstream.

Iteration 2862 audited record lowerer input contract in the driver typed-only pipeline follow-up workstream.

Iteration 2863 audited resolver input contract in the driver typed-only pipeline follow-up workstream.

Iteration 2864 audited resolved typed module output in the driver typed-only pipeline follow-up workstream.

Iteration 2865 audited BASIC backend API input in the driver typed-only pipeline follow-up workstream.

Iteration 2866 audited C backend API input in the driver typed-only pipeline follow-up workstream.

Iteration 2867 audited JVM backend API input in the driver typed-only pipeline follow-up workstream.

Iteration 2868 audited AST-only library callers in the driver typed-only pipeline follow-up workstream.

Iteration 2869 audited typed-only library API design in the driver typed-only pipeline follow-up workstream.

Iteration 2870 audited file dependency traversal in the driver typed-only pipeline follow-up workstream.

Iteration 2871 audited dependency typed module composition in the driver typed-only pipeline follow-up workstream.

Iteration 2872 audited legacy AST function signatures in the driver typed-only pipeline follow-up workstream.

Iteration 2873 audited typed callable declarations in the driver typed-only pipeline follow-up workstream.

Iteration 2874 audited typed top-level statements in the driver typed-only pipeline follow-up workstream.

Iteration 2875 audited generated name conflict checks in the driver typed-only pipeline follow-up workstream.

Iteration 2876 audited target capability checks in the driver typed-only pipeline follow-up workstream.

Iteration 2877 audited output cache keys in the driver typed-only pipeline follow-up workstream.

Iteration 2878 audited source path diagnostics in the driver typed-only pipeline follow-up workstream.

Iteration 2879 audited CLI entry points in the driver typed-only pipeline follow-up workstream.

Iteration 2880 audited library entry points in the driver typed-only pipeline follow-up workstream.

Iteration 2881 audited integration test matrix in the driver typed-only pipeline follow-up workstream.

Iteration 2882 audited incremental migration stages in the driver typed-only pipeline follow-up workstream.

Iteration 2883 audited public API compatibility in the driver typed-only pipeline follow-up workstream.

Iteration 2884 audited AST removal criteria in the driver typed-only pipeline follow-up workstream.

Iteration 2885 recorded shared helper implementation review in the batch checkpoint and remaining work workstream.

Iteration 2886 recorded driver helper delegation in the batch checkpoint and remaining work workstream.

Iteration 2887 recorded C duplicate conversion removal in the batch checkpoint and remaining work workstream.

Iteration 2888 recorded source span behavior review in the batch checkpoint and remaining work workstream.

Iteration 2889 recorded focused tests pass in the batch checkpoint and remaining work workstream.

Iteration 2890 recorded full tests pass in the batch checkpoint and remaining work workstream.

Iteration 2891 recorded Graphify refreshed after IR API change in the batch checkpoint and remaining work workstream.

Iteration 2892 recorded Graphify output excluded from commit in the batch checkpoint and remaining work workstream.

Iteration 2893 recorded source diff reviewed in the batch checkpoint and remaining work workstream.

Iteration 2894 recorded plan range continuity in the batch checkpoint and remaining work workstream.

Iteration 2895 recorded plan range uniqueness in the batch checkpoint and remaining work workstream.

Iteration 2896 recorded generated files checked in the batch checkpoint and remaining work workstream.

Iteration 2897 recorded no unrelated edits in the batch checkpoint and remaining work workstream.

Iteration 2898 recorded remaining C alignment gap in the batch checkpoint and remaining work workstream.

Iteration 2899 recorded remaining driver AST gap in the batch checkpoint and remaining work workstream.

Iteration 2900 recorded BASIC typed IR status in the batch checkpoint and remaining work workstream.

Iteration 2901 recorded JVM typed IR status in the batch checkpoint and remaining work workstream.

Iteration 2902 recorded cross-backend source contract in the batch checkpoint and remaining work workstream.

Iteration 2903 recorded next C migration seam in the batch checkpoint and remaining work workstream.

Iteration 2904 recorded next driver migration seam in the batch checkpoint and remaining work workstream.

Iteration 2905 recorded commit message precision in the batch checkpoint and remaining work workstream.

Iteration 2906 recorded checkpoint staged scope in the batch checkpoint and remaining work workstream.

Iteration 2907 recorded checkpoint diff check in the batch checkpoint and remaining work workstream.

Iteration 2908 recorded repository status after commit in the batch checkpoint and remaining work workstream.

Iteration 2909 completed this source-location migration batch and prepared the checkpoint.

Iteration 2910 migrated C top-level source alignment to checked `SemanticSource::source_position_at`.

Iteration 2911 migrated C callable source alignment to checked `SemanticSource::source_position_at`.

Iteration 2912 migrated JVM top-level source alignment to checked `SemanticSource::source_position_at`.

Iteration 2913 migrated JVM callable source alignment to checked `SemanticSource::source_position_at`.

Iteration 2914 migrated BASIC top-level source alignment to checked `SemanticSource::source_position_at`.

Iteration 2915 migrated BASIC callable source alignment to checked `SemanticSource::source_position_at`.

Iteration 2916 traced monotonic AST ordering in the C semantic top-level source alignment workstream.

Iteration 2917 traced duplicate-position siblings in the C semantic top-level source alignment workstream.

Iteration 2918 traced record-generated siblings in the C semantic top-level source alignment workstream.

Iteration 2919 traced source lookup failure in the C semantic top-level source alignment workstream.

Iteration 2920 traced invalid source index in the C semantic top-level source alignment workstream.

Iteration 2921 traced unknown source sentinel in the C semantic top-level source alignment workstream.

Iteration 2922 traced statement-source cardinality in the C semantic top-level source alignment workstream.

Iteration 2923 traced dependency source selection in the C semantic top-level source alignment workstream.

Iteration 2924 traced root source selection in the C semantic top-level source alignment workstream.

Iteration 2925 traced whitespace before source token in the C semantic top-level source alignment workstream.

Iteration 2926 traced Unicode source column in the C semantic top-level source alignment workstream.

Iteration 2927 traced span boundary behavior in the C semantic top-level source alignment workstream.

Iteration 2928 traced all-or-nothing map decline in the C semantic top-level source alignment workstream.

Iteration 2929 traced fallback statement path in the C semantic top-level source alignment workstream.

Iteration 2930 traced aligned semantic dispatch in the C semantic top-level source alignment workstream.

Iteration 2931 traced typed node ordering in the C semantic top-level source alignment workstream.

Iteration 2932 traced nested statement mapping in the C semantic top-level source alignment workstream.

Iteration 2933 traced C backend source identity in the C semantic top-level source alignment workstream.

Iteration 2934 traced shared source API in the C semantic top-level source alignment workstream.

Iteration 2935 inspected callable source index in the C semantic callable source alignment workstream.

Iteration 2936 inspected callable signature selection in the C semantic callable source alignment workstream.

Iteration 2937 inspected callable body root spans in the C semantic callable source alignment workstream.

Iteration 2938 inspected nested line child spans in the C semantic callable source alignment workstream.

Iteration 2939 inspected source filename comparison in the C semantic callable source alignment workstream.

Iteration 2940 inspected line comparison in the C semantic callable source alignment workstream.

Iteration 2941 inspected column comparison in the C semantic callable source alignment workstream.

Iteration 2942 inspected blank-line exclusion in the C semantic callable source alignment workstream.

Iteration 2943 inspected monotonic body ordering in the C semantic callable source alignment workstream.

Iteration 2944 inspected duplicate-position body siblings in the C semantic callable source alignment workstream.

Iteration 2945 inspected callable span bounds in the C semantic callable source alignment workstream.

Iteration 2946 inspected invalid callable source index in the C semantic callable source alignment workstream.

Iteration 2947 inspected dependency callable source in the C semantic callable source alignment workstream.

Iteration 2948 inspected record method source in the C semantic callable source alignment workstream.

Iteration 2949 inspected procedure source mapping in the C semantic callable source alignment workstream.

Iteration 2950 inspected function source mapping in the C semantic callable source alignment workstream.

Iteration 2951 inspected semantic body fallback in the C semantic callable source alignment workstream.

Iteration 2952 inspected AST body fallback in the C semantic callable source alignment workstream.

Iteration 2953 inspected unsupported semantic nodes in the C semantic callable source alignment workstream.

Iteration 2954 inspected typed return position in the C semantic callable source alignment workstream.

Iteration 2955 inspected typed THROW position in the C semantic callable source alignment workstream.

Iteration 2956 inspected typed OPEN position in the C semantic callable source alignment workstream.

Iteration 2957 inspected source-order invariants in the C semantic callable source alignment workstream.

Iteration 2958 inspected shared source API in the C semantic callable source alignment workstream.

Iteration 2959 inspected callable regression coverage in the C semantic callable source alignment workstream.

Iteration 2960 verified module source index in the JVM semantic top-level source alignment workstream.

Iteration 2961 verified semantic root span in the JVM semantic top-level source alignment workstream.

Iteration 2962 verified nested line spans in the JVM semantic top-level source alignment workstream.

Iteration 2963 verified source filename comparison in the JVM semantic top-level source alignment workstream.

Iteration 2964 verified line comparison in the JVM semantic top-level source alignment workstream.

Iteration 2965 verified column comparison in the JVM semantic top-level source alignment workstream.

Iteration 2966 verified blank-line exclusion in the JVM semantic top-level source alignment workstream.

Iteration 2967 verified strict unique AST candidate in the JVM semantic top-level source alignment workstream.

Iteration 2968 verified monotonic root ordering in the JVM semantic top-level source alignment workstream.

Iteration 2969 verified duplicate positions in the JVM semantic top-level source alignment workstream.

Iteration 2970 verified dependency/root sources in the JVM semantic top-level source alignment workstream.

Iteration 2971 verified missing source behavior in the JVM semantic top-level source alignment workstream.

Iteration 2972 verified invalid source behavior in the JVM semantic top-level source alignment workstream.

Iteration 2973 verified unknown adapter source in the JVM semantic top-level source alignment workstream.

Iteration 2974 verified typed module dispatch in the JVM semantic top-level source alignment workstream.

Iteration 2975 verified legacy fallback dispatch in the JVM semantic top-level source alignment workstream.

Iteration 2976 verified AST-only statement selection in the JVM semantic top-level source alignment workstream.

Iteration 2977 verified TRY filename source in the JVM semantic top-level source alignment workstream.

Iteration 2978 verified source filename escaping in the JVM semantic top-level source alignment workstream.

Iteration 2979 verified blank-line tests in the JVM semantic top-level source alignment workstream.

Iteration 2980 verified multi-source tests in the JVM semantic top-level source alignment workstream.

Iteration 2981 verified typed AST divergence in the JVM semantic top-level source alignment workstream.

Iteration 2982 verified module source ordering in the JVM semantic top-level source alignment workstream.

Iteration 2983 verified shared source API in the JVM semantic top-level source alignment workstream.

Iteration 2984 verified JVM regression coverage in the JVM semantic top-level source alignment workstream.

Iteration 2985 reviewed callable source index in the JVM callable source alignment workstream.

Iteration 2986 reviewed callable span bounds in the JVM callable source alignment workstream.

Iteration 2987 reviewed callable root span in the JVM callable source alignment workstream.

Iteration 2988 reviewed nested statement spans in the JVM callable source alignment workstream.

Iteration 2989 reviewed filename selection in the JVM callable source alignment workstream.

Iteration 2990 reviewed line selection in the JVM callable source alignment workstream.

Iteration 2991 reviewed column selection in the JVM callable source alignment workstream.

Iteration 2992 reviewed blank-line exclusion in the JVM callable source alignment workstream.

Iteration 2993 reviewed unique AST candidate in the JVM callable source alignment workstream.

Iteration 2994 reviewed monotonic body ordering in the JVM callable source alignment workstream.

Iteration 2995 reviewed duplicate position decline in the JVM callable source alignment workstream.

Iteration 2996 reviewed dependency callable source in the JVM callable source alignment workstream.

Iteration 2997 reviewed method source identity in the JVM callable source alignment workstream.

Iteration 2998 reviewed procedure source identity in the JVM callable source alignment workstream.

Iteration 2999 reviewed function source identity in the JVM callable source alignment workstream.

Iteration 3000 reviewed typed body dispatch in the JVM callable source alignment workstream.

Iteration 3001 reviewed legacy body fallback in the JVM callable source alignment workstream.

Iteration 3002 reviewed TRY metadata filename in the JVM callable source alignment workstream.

Iteration 3003 reviewed source index mismatch in the JVM callable source alignment workstream.

Iteration 3004 reviewed unknown source sentinel in the JVM callable source alignment workstream.

Iteration 3005 reviewed stale AST arity regression in the JVM callable source alignment workstream.

Iteration 3006 reviewed typed source disagreement in the JVM callable source alignment workstream.

Iteration 3007 reviewed shared source API in the JVM callable source alignment workstream.

Iteration 3008 reviewed callable test coverage in the JVM callable source alignment workstream.

Iteration 3009 reviewed remaining AST dependencies in the JVM callable source alignment workstream.

Iteration 3010 recorded top-level root span in the BASIC source alignment workstream.

Iteration 3011 recorded nested line source spans in the BASIC source alignment workstream.

Iteration 3012 recorded callable body source span in the BASIC source alignment workstream.

Iteration 3013 recorded source filename matching in the BASIC source alignment workstream.

Iteration 3014 recorded line matching in the BASIC source alignment workstream.

Iteration 3015 recorded column matching in the BASIC source alignment workstream.

Iteration 3016 recorded AST blank-line exclusion in the BASIC source alignment workstream.

Iteration 3017 recorded record sibling mapping in the BASIC source alignment workstream.

Iteration 3018 recorded AST candidate ordering in the BASIC source alignment workstream.

Iteration 3019 recorded duplicate candidates in the BASIC source alignment workstream.

Iteration 3020 recorded source mapping failure in the BASIC source alignment workstream.

Iteration 3021 recorded typed stream fallback in the BASIC source alignment workstream.

Iteration 3022 recorded source filename ownership in the BASIC source alignment workstream.

Iteration 3023 recorded dependency source selection in the BASIC source alignment workstream.

Iteration 3024 recorded generated lines in the BASIC source alignment workstream.

Iteration 3025 recorded typed comments in the BASIC source alignment workstream.

Iteration 3026 recorded typed TRY metadata in the BASIC source alignment workstream.

Iteration 3027 recorded typed catch locations in the BASIC source alignment workstream.

Iteration 3028 recorded shared source API in the BASIC source alignment workstream.

Iteration 3029 recorded non-ASCII column parity in the BASIC source alignment workstream.

Iteration 3030 recorded whitespace handling in the BASIC source alignment workstream.

Iteration 3031 recorded empty span behavior in the BASIC source alignment workstream.

Iteration 3032 recorded multi-file behavior in the BASIC source alignment workstream.

Iteration 3033 recorded callable regression coverage in the BASIC source alignment workstream.

Iteration 3034 recorded remaining compatibility paths in the BASIC source alignment workstream.

Iteration 3035 traced span start clamp in the shared source position semantics workstream.

Iteration 3036 traced span end clamp in the shared source position semantics workstream.

Iteration 3037 traced UTF-8 start normalization in the shared source position semantics workstream.

Iteration 3038 traced UTF-8 end normalization in the shared source position semantics workstream.

Iteration 3039 traced leading trivia scan bound in the shared source position semantics workstream.

Iteration 3040 traced line counting in the shared source position semantics workstream.

Iteration 3041 traced Unicode column counting in the shared source position semantics workstream.

Iteration 3042 traced filename cloning in the shared source position semantics workstream.

Iteration 3043 traced empty input in the shared source position semantics workstream.

Iteration 3044 traced empty span in the shared source position semantics workstream.

Iteration 3045 traced reversed span in the shared source position semantics workstream.

Iteration 3046 traced span past EOF in the shared source position semantics workstream.

Iteration 3047 traced CRLF handling in the shared source position semantics workstream.

Iteration 3048 traced tab handling in the shared source position semantics workstream.

Iteration 3049 traced multibyte prefix in the shared source position semantics workstream.

Iteration 3050 traced whitespace-only range in the shared source position semantics workstream.

Iteration 3051 traced comment-leading span in the shared source position semantics workstream.

Iteration 3052 traced generated spans in the shared source position semantics workstream.

Iteration 3053 traced diagnostic locations in the shared source position semantics workstream.

Iteration 3054 traced alignment locations in the shared source position semantics workstream.

Iteration 3055 traced source-position API ownership in the shared source position semantics workstream.

Iteration 3056 traced SemanticSource contract in the shared source position semantics workstream.

Iteration 3057 traced SourceSpan contract in the shared source position semantics workstream.

Iteration 3058 traced consumer consistency in the shared source position semantics workstream.

Iteration 3059 traced unit test coverage in the shared source position semantics workstream.

Iteration 3060 inspected BASIC mapping uses typed spans in the cross-backend invariants workstream.

Iteration 3061 inspected C mapping uses typed spans in the cross-backend invariants workstream.

Iteration 3062 inspected JVM mapping uses typed spans in the cross-backend invariants workstream.

Iteration 3063 inspected BASIC callable mapping uses typed spans in the cross-backend invariants workstream.

Iteration 3064 inspected C callable mapping uses typed spans in the cross-backend invariants workstream.

Iteration 3065 inspected JVM callable mapping uses typed spans in the cross-backend invariants workstream.

Iteration 3066 inspected driver diagnostics use typed spans in the cross-backend invariants workstream.

Iteration 3067 inspected C diagnostics use typed spans in the cross-backend invariants workstream.

Iteration 3068 inspected resolver retains semantic spans in the cross-backend invariants workstream.

Iteration 3069 inspected adapter retains source identity in the cross-backend invariants workstream.

Iteration 3070 inspected dependency indices remap in the cross-backend invariants workstream.

Iteration 3071 inspected root indices remain stable in the cross-backend invariants workstream.

Iteration 3072 inspected nested statements inherit module source in the cross-backend invariants workstream.

Iteration 3073 inspected AST positions remain compatibility data in the cross-backend invariants workstream.

Iteration 3074 inspected typed source is backend truth in the cross-backend invariants workstream.

Iteration 3075 inspected no backend span parsing in the cross-backend invariants workstream.

Iteration 3076 inspected same line/column formula in the cross-backend invariants workstream.

Iteration 3077 inspected filename normalization parity in the cross-backend invariants workstream.

Iteration 3078 inspected blank-line parity in the cross-backend invariants workstream.

Iteration 3079 inspected Unicode parity in the cross-backend invariants workstream.

Iteration 3080 inspected malformed span behavior in the cross-backend invariants workstream.

Iteration 3081 inspected unknown source behavior in the cross-backend invariants workstream.

Iteration 3082 inspected all backend test suite in the cross-backend invariants workstream.

Iteration 3083 inspected Graphify dependency update in the cross-backend invariants workstream.

Iteration 3084 inspected migration milestone in the cross-backend invariants workstream.

Iteration 3085 verified compile-source semantic parse in the typed IR and driver boundary workstream.

Iteration 3086 verified compile-file semantic parse in the typed IR and driver boundary workstream.

Iteration 3087 verified compatibility AST parse role in the typed IR and driver boundary workstream.

Iteration 3088 verified resolved program contract in the typed IR and driver boundary workstream.

Iteration 3089 verified semantic module contract in the typed IR and driver boundary workstream.

Iteration 3090 verified backend dispatch input in the typed IR and driver boundary workstream.

Iteration 3091 verified dependency module loading in the typed IR and driver boundary workstream.

Iteration 3092 verified module prepend semantics in the typed IR and driver boundary workstream.

Iteration 3093 verified record transpilation producer in the typed IR and driver boundary workstream.

Iteration 3094 verified generated frontend span producer in the typed IR and driver boundary workstream.

Iteration 3095 verified resolver span preservation in the typed IR and driver boundary workstream.

Iteration 3096 verified driver diagnostic consumer in the typed IR and driver boundary workstream.

Iteration 3097 verified C diagnostic consumer in the typed IR and driver boundary workstream.

Iteration 3098 verified BASIC alignment consumer in the typed IR and driver boundary workstream.

Iteration 3099 verified JVM alignment consumer in the typed IR and driver boundary workstream.

Iteration 3100 verified AST-free codegen prerequisite in the typed IR and driver boundary workstream.

Iteration 3101 verified typed-only API proposal in the typed IR and driver boundary workstream.

Iteration 3102 verified public compatibility callers in the typed IR and driver boundary workstream.

Iteration 3103 verified multi-file input in the typed IR and driver boundary workstream.

Iteration 3104 verified target capability checks in the typed IR and driver boundary workstream.

Iteration 3105 verified generated name validation in the typed IR and driver boundary workstream.

Iteration 3106 verified driver regression coverage in the typed IR and driver boundary workstream.

Iteration 3107 verified CLI regression coverage in the typed IR and driver boundary workstream.

Iteration 3108 verified library regression coverage in the typed IR and driver boundary workstream.

Iteration 3109 verified future migration order in the typed IR and driver boundary workstream.

Iteration 3110 reviewed semantic IR span unit tests in the validation for shared span refactor workstream.

Iteration 3111 reviewed UTF-8 boundary unit test in the validation for shared span refactor workstream.

Iteration 3112 reviewed leading trivia unit test in the validation for shared span refactor workstream.

Iteration 3113 reviewed C invalid source identity test in the validation for shared span refactor workstream.

Iteration 3114 reviewed C source map test in the validation for shared span refactor workstream.

Iteration 3115 reviewed C callable source map test in the validation for shared span refactor workstream.

Iteration 3116 reviewed JVM source map tests in the validation for shared span refactor workstream.

Iteration 3117 reviewed BASIC source map tests in the validation for shared span refactor workstream.

Iteration 3118 reviewed driver diagnostic tests in the validation for shared span refactor workstream.

Iteration 3119 reviewed focused backend tests in the validation for shared span refactor workstream.

Iteration 3120 reviewed library test group in the validation for shared span refactor workstream.

Iteration 3121 reviewed CLI test group in the validation for shared span refactor workstream.

Iteration 3122 reviewed DOSBox/C conformance group in the validation for shared span refactor workstream.

Iteration 3123 reviewed example test group in the validation for shared span refactor workstream.

Iteration 3124 reviewed JVM conformance group in the validation for shared span refactor workstream.

Iteration 3125 reviewed language conformance group in the validation for shared span refactor workstream.

Iteration 3126 reviewed record test groups in the validation for shared span refactor workstream.

Iteration 3127 reviewed generated tutorial fixtures in the validation for shared span refactor workstream.

Iteration 3128 reviewed full locked suite in the validation for shared span refactor workstream.

Iteration 3129 reviewed Graphify refreshed state in the validation for shared span refactor workstream.

Iteration 3130 reviewed graph node count in the validation for shared span refactor workstream.

Iteration 3131 reviewed diff whitespace check in the validation for shared span refactor workstream.

Iteration 3132 reviewed iteration range check in the validation for shared span refactor workstream.

Iteration 3133 reviewed iteration uniqueness check in the validation for shared span refactor workstream.

Iteration 3134 reviewed worktree state in the validation for shared span refactor workstream.

Iteration 3135 recorded C main-loop AST iteration in the remaining migration review workstream.

Iteration 3136 recorded C semantic root iteration in the remaining migration review workstream.

Iteration 3137 recorded C THROW location dependency in the remaining migration review workstream.

Iteration 3138 recorded C OPEN location dependency in the remaining migration review workstream.

Iteration 3139 recorded C unsupported-node fallback in the remaining migration review workstream.

Iteration 3140 recorded C transactional state requirements in the remaining migration review workstream.

Iteration 3141 recorded BASIC whole-stream compatibility path in the remaining migration review workstream.

Iteration 3142 recorded BASIC per-statement path in the remaining migration review workstream.

Iteration 3143 recorded JVM whole-module path in the remaining migration review workstream.

Iteration 3144 recorded JVM callable path in the remaining migration review workstream.

Iteration 3145 recorded driver compatibility AST construction in the remaining migration review workstream.

Iteration 3146 recorded resolver dependence on AST in the remaining migration review workstream.

Iteration 3147 recorded typed callable declarations in the remaining migration review workstream.

Iteration 3148 recorded typed module headers in the remaining migration review workstream.

Iteration 3149 recorded record method binding in the remaining migration review workstream.

Iteration 3150 recorded array shape metadata in the remaining migration review workstream.

Iteration 3151 recorded source location metadata in the remaining migration review workstream.

Iteration 3152 recorded typed codegen context in the remaining migration review workstream.

Iteration 3153 recorded target-specific diagnostics in the remaining migration review workstream.

Iteration 3154 recorded backend semantic ownership in the remaining migration review workstream.

Iteration 3155 recorded shared semantic helper ownership in the remaining migration review workstream.

Iteration 3156 recorded source map contract in the remaining migration review workstream.

Iteration 3157 recorded next C migration slice in the remaining migration review workstream.

Iteration 3158 recorded next driver migration slice in the remaining migration review workstream.

Iteration 3159 completed this cross-backend source-alignment batch and prepared the checkpoint.

Iteration 3160 changed `emit_c_semantic_open` to accept `SourcePos` instead of an AST statement.

Iteration 3161 updated top-level typed OPEN dispatch to pass its source position.

Iteration 3162 updated callable typed OPEN dispatch to pass its source position.

Iteration 3163 verified top-level semantic sequential OPEN generation.

Iteration 3164 verified callable semantic input OPEN error generation.

Iteration 3165 traced that C source alignment still requires checked typed-source offsets in the C source alignment invariants workstream.

Iteration 3166 traced that unsupported OPEN emission retains AST fallback behavior in the C fallback behavior workstream.

Iteration 3167 traced that OPEN raise-site numbering remains in the caller context in the C semantic state workstream.

Iteration 3168 traced that BASIC source mapping uses checked source positions in the cross-backend source alignment workstream.

Iteration 3169 traced that C source mapping uses checked source positions in the cross-backend source alignment workstream.

Iteration 3170 traced that JVM source mapping uses checked source positions in the cross-backend source alignment workstream.

Iteration 3171 traced that BASIC callable mapping uses checked source positions in the callable source alignment workstream.

Iteration 3172 traced that C callable mapping uses checked source positions in the callable source alignment workstream.

Iteration 3173 traced that JVM callable mapping uses checked source positions in the callable source alignment workstream.

Iteration 3174 traced that malformed source offsets decline alignment in the source API validation workstream.

Iteration 3175 traced that semantic source filename is retained through alignment in the typed source ownership workstream.

Iteration 3176 traced that statement spans remain the alignment origin in the semantic IR spans workstream.

Iteration 3177 traced that diagnostics use shared tolerant source positioning in the driver integration workstream.

Iteration 3178 traced that semantic target diagnostics use shared source positioning in the C diagnostics workstream.

Iteration 3179 traced that typed stream regressions remain green in the BASIC regression coverage workstream.

Iteration 3180 traced that blank-line and multi-source regressions remain green in the JVM regression coverage workstream.

Iteration 3181 traced that source identity decline regression remains green in the C regression coverage workstream.

Iteration 3182 traced that top-level sequential OPEN regression passes in the C OPEN test coverage workstream.

Iteration 3183 traced that callable input OPEN error-path regression passes in the C OPEN test coverage workstream.

Iteration 3184 traced that all test groups pass after the typed-source boundary change in the full suite validation workstream.

Iteration 3185 inspected that C semantic OPEN emitter now accepts `SourcePos` directly in the C semantic OPEN emitter boundary workstream.

Iteration 3186 inspected that OPEN failure line metadata remains explicit in the typed OPEN diagnostics workstream.

Iteration 3187 inspected that OPEN failure filename metadata remains explicit in the OPEN filename metadata workstream.

Iteration 3188 inspected that top-level OPEN call site supplies mapped source position in the top-level C semantic dispatch workstream.

Iteration 3189 inspected that callable OPEN call site supplies callable source position in the callable C semantic dispatch workstream.

Iteration 3190 inspected that C source alignment still requires checked typed-source offsets in the C source alignment invariants workstream.

Iteration 3191 inspected that unsupported OPEN emission retains AST fallback behavior in the C fallback behavior workstream.

Iteration 3192 inspected that OPEN raise-site numbering remains in the caller context in the C semantic state workstream.

Iteration 3193 inspected that BASIC source mapping uses checked source positions in the cross-backend source alignment workstream.

Iteration 3194 inspected that C source mapping uses checked source positions in the cross-backend source alignment workstream.

Iteration 3195 inspected that JVM source mapping uses checked source positions in the cross-backend source alignment workstream.

Iteration 3196 inspected that BASIC callable mapping uses checked source positions in the callable source alignment workstream.

Iteration 3197 inspected that C callable mapping uses checked source positions in the callable source alignment workstream.

Iteration 3198 inspected that JVM callable mapping uses checked source positions in the callable source alignment workstream.

Iteration 3199 inspected that malformed source offsets decline alignment in the source API validation workstream.

Iteration 3200 inspected that semantic source filename is retained through alignment in the typed source ownership workstream.

Iteration 3201 inspected that statement spans remain the alignment origin in the semantic IR spans workstream.

Iteration 3202 inspected that diagnostics use shared tolerant source positioning in the driver integration workstream.

Iteration 3203 inspected that semantic target diagnostics use shared source positioning in the C diagnostics workstream.

Iteration 3204 inspected that typed stream regressions remain green in the BASIC regression coverage workstream.

Iteration 3205 inspected that blank-line and multi-source regressions remain green in the JVM regression coverage workstream.

Iteration 3206 inspected that source identity decline regression remains green in the C regression coverage workstream.

Iteration 3207 inspected that top-level sequential OPEN regression passes in the C OPEN test coverage workstream.

Iteration 3208 inspected that callable input OPEN error-path regression passes in the C OPEN test coverage workstream.

Iteration 3209 inspected that all test groups pass after the typed-source boundary change in the full suite validation workstream.

Iteration 3210 verified that C semantic OPEN emitter now accepts `SourcePos` directly in the C semantic OPEN emitter boundary workstream.

Iteration 3211 verified that OPEN failure line metadata remains explicit in the typed OPEN diagnostics workstream.

Iteration 3212 verified that OPEN failure filename metadata remains explicit in the OPEN filename metadata workstream.

Iteration 3213 verified that top-level OPEN call site supplies mapped source position in the top-level C semantic dispatch workstream.

Iteration 3214 verified that callable OPEN call site supplies callable source position in the callable C semantic dispatch workstream.

Iteration 3215 verified that C source alignment still requires checked typed-source offsets in the C source alignment invariants workstream.

Iteration 3216 verified that unsupported OPEN emission retains AST fallback behavior in the C fallback behavior workstream.

Iteration 3217 verified that OPEN raise-site numbering remains in the caller context in the C semantic state workstream.

Iteration 3218 verified that BASIC source mapping uses checked source positions in the cross-backend source alignment workstream.

Iteration 3219 verified that C source mapping uses checked source positions in the cross-backend source alignment workstream.

Iteration 3220 verified that JVM source mapping uses checked source positions in the cross-backend source alignment workstream.

Iteration 3221 verified that BASIC callable mapping uses checked source positions in the callable source alignment workstream.

Iteration 3222 verified that C callable mapping uses checked source positions in the callable source alignment workstream.

Iteration 3223 verified that JVM callable mapping uses checked source positions in the callable source alignment workstream.

Iteration 3224 verified that malformed source offsets decline alignment in the source API validation workstream.

Iteration 3225 verified that semantic source filename is retained through alignment in the typed source ownership workstream.

Iteration 3226 verified that statement spans remain the alignment origin in the semantic IR spans workstream.

Iteration 3227 verified that diagnostics use shared tolerant source positioning in the driver integration workstream.

Iteration 3228 verified that semantic target diagnostics use shared source positioning in the C diagnostics workstream.

Iteration 3229 verified that typed stream regressions remain green in the BASIC regression coverage workstream.

Iteration 3230 verified that blank-line and multi-source regressions remain green in the JVM regression coverage workstream.

Iteration 3231 verified that source identity decline regression remains green in the C regression coverage workstream.

Iteration 3232 verified that top-level sequential OPEN regression passes in the C OPEN test coverage workstream.

Iteration 3233 verified that callable input OPEN error-path regression passes in the C OPEN test coverage workstream.

Iteration 3234 verified that all test groups pass after the typed-source boundary change in the full suite validation workstream.

Iteration 3235 reviewed that C semantic OPEN emitter now accepts `SourcePos` directly in the C semantic OPEN emitter boundary workstream.

Iteration 3236 reviewed that OPEN failure line metadata remains explicit in the typed OPEN diagnostics workstream.

Iteration 3237 reviewed that OPEN failure filename metadata remains explicit in the OPEN filename metadata workstream.

Iteration 3238 reviewed that top-level OPEN call site supplies mapped source position in the top-level C semantic dispatch workstream.

Iteration 3239 reviewed that callable OPEN call site supplies callable source position in the callable C semantic dispatch workstream.

Iteration 3240 reviewed that C source alignment still requires checked typed-source offsets in the C source alignment invariants workstream.

Iteration 3241 reviewed that unsupported OPEN emission retains AST fallback behavior in the C fallback behavior workstream.

Iteration 3242 reviewed that OPEN raise-site numbering remains in the caller context in the C semantic state workstream.

Iteration 3243 reviewed that BASIC source mapping uses checked source positions in the cross-backend source alignment workstream.

Iteration 3244 reviewed that C source mapping uses checked source positions in the cross-backend source alignment workstream.

Iteration 3245 reviewed that JVM source mapping uses checked source positions in the cross-backend source alignment workstream.

Iteration 3246 reviewed that BASIC callable mapping uses checked source positions in the callable source alignment workstream.

Iteration 3247 reviewed that C callable mapping uses checked source positions in the callable source alignment workstream.

Iteration 3248 reviewed that JVM callable mapping uses checked source positions in the callable source alignment workstream.

Iteration 3249 reviewed that malformed source offsets decline alignment in the source API validation workstream.

Iteration 3250 reviewed that semantic source filename is retained through alignment in the typed source ownership workstream.

Iteration 3251 reviewed that statement spans remain the alignment origin in the semantic IR spans workstream.

Iteration 3252 reviewed that diagnostics use shared tolerant source positioning in the driver integration workstream.

Iteration 3253 reviewed that semantic target diagnostics use shared source positioning in the C diagnostics workstream.

Iteration 3254 reviewed that typed stream regressions remain green in the BASIC regression coverage workstream.

Iteration 3255 reviewed that blank-line and multi-source regressions remain green in the JVM regression coverage workstream.

Iteration 3256 reviewed that source identity decline regression remains green in the C regression coverage workstream.

Iteration 3257 reviewed that top-level sequential OPEN regression passes in the C OPEN test coverage workstream.

Iteration 3258 reviewed that callable input OPEN error-path regression passes in the C OPEN test coverage workstream.

Iteration 3259 reviewed that all test groups pass after the typed-source boundary change in the full suite validation workstream.

Iteration 3260 recorded that C semantic OPEN emitter now accepts `SourcePos` directly in the C semantic OPEN emitter boundary workstream.

Iteration 3261 recorded that OPEN failure line metadata remains explicit in the typed OPEN diagnostics workstream.

Iteration 3262 recorded that OPEN failure filename metadata remains explicit in the OPEN filename metadata workstream.

Iteration 3263 recorded that top-level OPEN call site supplies mapped source position in the top-level C semantic dispatch workstream.

Iteration 3264 recorded that callable OPEN call site supplies callable source position in the callable C semantic dispatch workstream.

Iteration 3265 recorded that C source alignment still requires checked typed-source offsets in the C source alignment invariants workstream.

Iteration 3266 recorded that unsupported OPEN emission retains AST fallback behavior in the C fallback behavior workstream.

Iteration 3267 recorded that OPEN raise-site numbering remains in the caller context in the C semantic state workstream.

Iteration 3268 recorded that BASIC source mapping uses checked source positions in the cross-backend source alignment workstream.

Iteration 3269 recorded that C source mapping uses checked source positions in the cross-backend source alignment workstream.

Iteration 3270 recorded that JVM source mapping uses checked source positions in the cross-backend source alignment workstream.

Iteration 3271 recorded that BASIC callable mapping uses checked source positions in the callable source alignment workstream.

Iteration 3272 recorded that C callable mapping uses checked source positions in the callable source alignment workstream.

Iteration 3273 recorded that JVM callable mapping uses checked source positions in the callable source alignment workstream.

Iteration 3274 recorded that malformed source offsets decline alignment in the source API validation workstream.

Iteration 3275 recorded that semantic source filename is retained through alignment in the typed source ownership workstream.

Iteration 3276 recorded that statement spans remain the alignment origin in the semantic IR spans workstream.

Iteration 3277 recorded that diagnostics use shared tolerant source positioning in the driver integration workstream.

Iteration 3278 recorded that semantic target diagnostics use shared source positioning in the C diagnostics workstream.

Iteration 3279 recorded that typed stream regressions remain green in the BASIC regression coverage workstream.

Iteration 3280 recorded that blank-line and multi-source regressions remain green in the JVM regression coverage workstream.

Iteration 3281 recorded that source identity decline regression remains green in the C regression coverage workstream.

Iteration 3282 recorded that top-level sequential OPEN regression passes in the C OPEN test coverage workstream.

Iteration 3283 recorded that callable input OPEN error-path regression passes in the C OPEN test coverage workstream.

Iteration 3284 recorded that all test groups pass after the typed-source boundary change in the full suite validation workstream.

Iteration 3285 traced that C semantic OPEN emitter now accepts `SourcePos` directly in the C semantic OPEN emitter boundary workstream.

Iteration 3286 traced that OPEN failure line metadata remains explicit in the typed OPEN diagnostics workstream.

Iteration 3287 traced that OPEN failure filename metadata remains explicit in the OPEN filename metadata workstream.

Iteration 3288 traced that top-level OPEN call site supplies mapped source position in the top-level C semantic dispatch workstream.

Iteration 3289 traced that callable OPEN call site supplies callable source position in the callable C semantic dispatch workstream.

Iteration 3290 traced that C source alignment still requires checked typed-source offsets in the C source alignment invariants workstream.

Iteration 3291 traced that unsupported OPEN emission retains AST fallback behavior in the C fallback behavior workstream.

Iteration 3292 traced that OPEN raise-site numbering remains in the caller context in the C semantic state workstream.

Iteration 3293 traced that BASIC source mapping uses checked source positions in the cross-backend source alignment workstream.

Iteration 3294 traced that C source mapping uses checked source positions in the cross-backend source alignment workstream.

Iteration 3295 traced that JVM source mapping uses checked source positions in the cross-backend source alignment workstream.

Iteration 3296 traced that BASIC callable mapping uses checked source positions in the callable source alignment workstream.

Iteration 3297 traced that C callable mapping uses checked source positions in the callable source alignment workstream.

Iteration 3298 traced that JVM callable mapping uses checked source positions in the callable source alignment workstream.

Iteration 3299 traced that malformed source offsets decline alignment in the source API validation workstream.

Iteration 3300 traced that semantic source filename is retained through alignment in the typed source ownership workstream.

Iteration 3301 traced that statement spans remain the alignment origin in the semantic IR spans workstream.

Iteration 3302 traced that diagnostics use shared tolerant source positioning in the driver integration workstream.

Iteration 3303 traced that semantic target diagnostics use shared source positioning in the C diagnostics workstream.

Iteration 3304 traced that typed stream regressions remain green in the BASIC regression coverage workstream.

Iteration 3305 traced that blank-line and multi-source regressions remain green in the JVM regression coverage workstream.

Iteration 3306 traced that source identity decline regression remains green in the C regression coverage workstream.

Iteration 3307 traced that top-level sequential OPEN regression passes in the C OPEN test coverage workstream.

Iteration 3308 traced that callable input OPEN error-path regression passes in the C OPEN test coverage workstream.

Iteration 3309 traced that all test groups pass after the typed-source boundary change in the full suite validation workstream.

Iteration 3310 inspected that C semantic OPEN emitter now accepts `SourcePos` directly in the C semantic OPEN emitter boundary workstream.

Iteration 3311 inspected that OPEN failure line metadata remains explicit in the typed OPEN diagnostics workstream.

Iteration 3312 inspected that OPEN failure filename metadata remains explicit in the OPEN filename metadata workstream.

Iteration 3313 inspected that top-level OPEN call site supplies mapped source position in the top-level C semantic dispatch workstream.

Iteration 3314 inspected that callable OPEN call site supplies callable source position in the callable C semantic dispatch workstream.

Iteration 3315 inspected that C source alignment still requires checked typed-source offsets in the C source alignment invariants workstream.

Iteration 3316 inspected that unsupported OPEN emission retains AST fallback behavior in the C fallback behavior workstream.

Iteration 3317 inspected that OPEN raise-site numbering remains in the caller context in the C semantic state workstream.

Iteration 3318 inspected that BASIC source mapping uses checked source positions in the cross-backend source alignment workstream.

Iteration 3319 inspected that C source mapping uses checked source positions in the cross-backend source alignment workstream.

Iteration 3320 inspected that JVM source mapping uses checked source positions in the cross-backend source alignment workstream.

Iteration 3321 inspected that BASIC callable mapping uses checked source positions in the callable source alignment workstream.

Iteration 3322 inspected that C callable mapping uses checked source positions in the callable source alignment workstream.

Iteration 3323 inspected that JVM callable mapping uses checked source positions in the callable source alignment workstream.

Iteration 3324 inspected that malformed source offsets decline alignment in the source API validation workstream.

Iteration 3325 inspected that semantic source filename is retained through alignment in the typed source ownership workstream.

Iteration 3326 inspected that statement spans remain the alignment origin in the semantic IR spans workstream.

Iteration 3327 inspected that diagnostics use shared tolerant source positioning in the driver integration workstream.

Iteration 3328 inspected that semantic target diagnostics use shared source positioning in the C diagnostics workstream.

Iteration 3329 inspected that typed stream regressions remain green in the BASIC regression coverage workstream.

Iteration 3330 inspected that blank-line and multi-source regressions remain green in the JVM regression coverage workstream.

Iteration 3331 inspected that source identity decline regression remains green in the C regression coverage workstream.

Iteration 3332 inspected that top-level sequential OPEN regression passes in the C OPEN test coverage workstream.

Iteration 3333 inspected that callable input OPEN error-path regression passes in the C OPEN test coverage workstream.

Iteration 3334 inspected that all test groups pass after the typed-source boundary change in the full suite validation workstream.

Iteration 3335 verified that C semantic OPEN emitter now accepts `SourcePos` directly in the C semantic OPEN emitter boundary workstream.

Iteration 3336 verified that OPEN failure line metadata remains explicit in the typed OPEN diagnostics workstream.

Iteration 3337 verified that OPEN failure filename metadata remains explicit in the OPEN filename metadata workstream.

Iteration 3338 verified that top-level OPEN call site supplies mapped source position in the top-level C semantic dispatch workstream.

Iteration 3339 verified that callable OPEN call site supplies callable source position in the callable C semantic dispatch workstream.

Iteration 3340 verified that C source alignment still requires checked typed-source offsets in the C source alignment invariants workstream.

Iteration 3341 verified that unsupported OPEN emission retains AST fallback behavior in the C fallback behavior workstream.

Iteration 3342 verified that OPEN raise-site numbering remains in the caller context in the C semantic state workstream.

Iteration 3343 verified that BASIC source mapping uses checked source positions in the cross-backend source alignment workstream.

Iteration 3344 verified that C source mapping uses checked source positions in the cross-backend source alignment workstream.

Iteration 3345 verified that JVM source mapping uses checked source positions in the cross-backend source alignment workstream.

Iteration 3346 verified that BASIC callable mapping uses checked source positions in the callable source alignment workstream.

Iteration 3347 verified that C callable mapping uses checked source positions in the callable source alignment workstream.

Iteration 3348 verified that JVM callable mapping uses checked source positions in the callable source alignment workstream.

Iteration 3349 verified that malformed source offsets decline alignment in the source API validation workstream.

Iteration 3350 verified that semantic source filename is retained through alignment in the typed source ownership workstream.

Iteration 3351 verified that statement spans remain the alignment origin in the semantic IR spans workstream.

Iteration 3352 verified that diagnostics use shared tolerant source positioning in the driver integration workstream.

Iteration 3353 verified that semantic target diagnostics use shared source positioning in the C diagnostics workstream.

Iteration 3354 verified that typed stream regressions remain green in the BASIC regression coverage workstream.

Iteration 3355 verified that blank-line and multi-source regressions remain green in the JVM regression coverage workstream.

Iteration 3356 verified that source identity decline regression remains green in the C regression coverage workstream.

Iteration 3357 verified that top-level sequential OPEN regression passes in the C OPEN test coverage workstream.

Iteration 3358 verified that callable input OPEN error-path regression passes in the C OPEN test coverage workstream.

Iteration 3359 verified that all test groups pass after the typed-source boundary change in the full suite validation workstream.

Iteration 3360 reviewed that C semantic OPEN emitter now accepts `SourcePos` directly in the C semantic OPEN emitter boundary workstream.

Iteration 3361 reviewed that OPEN failure line metadata remains explicit in the typed OPEN diagnostics workstream.

Iteration 3362 reviewed that OPEN failure filename metadata remains explicit in the OPEN filename metadata workstream.

Iteration 3363 reviewed that top-level OPEN call site supplies mapped source position in the top-level C semantic dispatch workstream.

Iteration 3364 reviewed that callable OPEN call site supplies callable source position in the callable C semantic dispatch workstream.

Iteration 3365 reviewed that C source alignment still requires checked typed-source offsets in the C source alignment invariants workstream.

Iteration 3366 reviewed that unsupported OPEN emission retains AST fallback behavior in the C fallback behavior workstream.

Iteration 3367 reviewed that OPEN raise-site numbering remains in the caller context in the C semantic state workstream.

Iteration 3368 reviewed that BASIC source mapping uses checked source positions in the cross-backend source alignment workstream.

Iteration 3369 reviewed that C source mapping uses checked source positions in the cross-backend source alignment workstream.

Iteration 3370 reviewed that JVM source mapping uses checked source positions in the cross-backend source alignment workstream.

Iteration 3371 reviewed that BASIC callable mapping uses checked source positions in the callable source alignment workstream.

Iteration 3372 reviewed that C callable mapping uses checked source positions in the callable source alignment workstream.

Iteration 3373 reviewed that JVM callable mapping uses checked source positions in the callable source alignment workstream.

Iteration 3374 reviewed that malformed source offsets decline alignment in the source API validation workstream.

Iteration 3375 reviewed that semantic source filename is retained through alignment in the typed source ownership workstream.

Iteration 3376 reviewed that statement spans remain the alignment origin in the semantic IR spans workstream.

Iteration 3377 reviewed that diagnostics use shared tolerant source positioning in the driver integration workstream.

Iteration 3378 reviewed that semantic target diagnostics use shared source positioning in the C diagnostics workstream.

Iteration 3379 reviewed that typed stream regressions remain green in the BASIC regression coverage workstream.

Iteration 3380 reviewed that blank-line and multi-source regressions remain green in the JVM regression coverage workstream.

Iteration 3381 reviewed that source identity decline regression remains green in the C regression coverage workstream.

Iteration 3382 reviewed that top-level sequential OPEN regression passes in the C OPEN test coverage workstream.

Iteration 3383 reviewed that callable input OPEN error-path regression passes in the C OPEN test coverage workstream.

Iteration 3384 reviewed that all test groups pass after the typed-source boundary change in the full suite validation workstream.

Iteration 3385 recorded that C semantic OPEN emitter now accepts `SourcePos` directly in the C semantic OPEN emitter boundary workstream.

Iteration 3386 recorded that OPEN failure line metadata remains explicit in the typed OPEN diagnostics workstream.

Iteration 3387 recorded that OPEN failure filename metadata remains explicit in the OPEN filename metadata workstream.

Iteration 3388 recorded that top-level OPEN call site supplies mapped source position in the top-level C semantic dispatch workstream.

Iteration 3389 recorded that callable OPEN call site supplies callable source position in the callable C semantic dispatch workstream.

Iteration 3390 recorded that C source alignment still requires checked typed-source offsets in the C source alignment invariants workstream.

Iteration 3391 recorded that unsupported OPEN emission retains AST fallback behavior in the C fallback behavior workstream.

Iteration 3392 recorded that OPEN raise-site numbering remains in the caller context in the C semantic state workstream.

Iteration 3393 recorded that BASIC source mapping uses checked source positions in the cross-backend source alignment workstream.

Iteration 3394 recorded that C source mapping uses checked source positions in the cross-backend source alignment workstream.

Iteration 3395 recorded that JVM source mapping uses checked source positions in the cross-backend source alignment workstream.

Iteration 3396 recorded that BASIC callable mapping uses checked source positions in the callable source alignment workstream.

Iteration 3397 recorded that C callable mapping uses checked source positions in the callable source alignment workstream.

Iteration 3398 recorded that JVM callable mapping uses checked source positions in the callable source alignment workstream.

Iteration 3399 recorded that malformed source offsets decline alignment in the source API validation workstream.

Iteration 3400 recorded that semantic source filename is retained through alignment in the typed source ownership workstream.

Iteration 3401 recorded that statement spans remain the alignment origin in the semantic IR spans workstream.

Iteration 3402 recorded that diagnostics use shared tolerant source positioning in the driver integration workstream.

Iteration 3403 recorded that semantic target diagnostics use shared source positioning in the C diagnostics workstream.

Iteration 3404 recorded that typed stream regressions remain green in the BASIC regression coverage workstream.

Iteration 3405 recorded that blank-line and multi-source regressions remain green in the JVM regression coverage workstream.

Iteration 3406 recorded that source identity decline regression remains green in the C regression coverage workstream.

Iteration 3407 recorded that top-level sequential OPEN regression passes in the C OPEN test coverage workstream.

Iteration 3408 recorded that callable input OPEN error-path regression passes in the C OPEN test coverage workstream.

Iteration 3409 completed this batch and prepared the typed-source emitter checkpoint.

Iteration 3410 extracted top-level C semantic THROW emission into a typed helper.

Iteration 3411 made the typed THROW helper consume `ThrowValue` and `SourcePos`.

Iteration 3412 preserved numeric expression rendering and raise-site sequencing.

Iteration 3413 verified semantic top-level THROW output.

Iteration 3414 verified semantic callable THROW output.

Iteration 3415 traced raise-site numbering in the C typed THROW extraction workstream.

Iteration 3416 traced retry-label emission in the C typed THROW extraction workstream.

Iteration 3417 traced continuation-label emission in the C typed THROW extraction workstream.

Iteration 3418 traced source-line metadata in the C typed THROW extraction workstream.

Iteration 3419 traced source-filename metadata in the C typed THROW extraction workstream.

Iteration 3420 traced `SourcePos` input contract in the C typed THROW extraction workstream.

Iteration 3421 traced AST statement independence in emitter in the C typed THROW extraction workstream.

Iteration 3422 traced top-level caller position argument in the C typed THROW extraction workstream.

Iteration 3423 traced callable caller position argument in the C typed THROW extraction workstream.

Iteration 3424 traced source filename escaping in the C typed THROW extraction workstream.

Iteration 3425 traced diagnostic display filename in the C typed THROW extraction workstream.

Iteration 3426 traced typed expression span ownership in the C typed THROW extraction workstream.

Iteration 3427 traced compatibility fallback position in the C typed THROW extraction workstream.

Iteration 3428 traced semantic source mapping invariant in the C typed THROW extraction workstream.

Iteration 3429 traced multi-file location assumptions in the C typed THROW extraction workstream.

Iteration 3430 traced raise ID mutation timing in the C typed THROW extraction workstream.

Iteration 3431 traced math helper state mutation in the C typed THROW extraction workstream.

Iteration 3432 traced dispatch label borrowing in the C typed THROW extraction workstream.

Iteration 3433 traced output buffer ownership in the C typed THROW extraction workstream.

Iteration 3434 traced failed expression decline behavior in the C typed THROW extraction workstream.

Iteration 3435 inspected top-level THROW dispatch in the C source-location boundary workstream.

Iteration 3436 inspected semantic bare THROW handling in the C source-location boundary workstream.

Iteration 3437 inspected semantic value THROW handling in the C source-location boundary workstream.

Iteration 3438 inspected numeric expression rendering in the C source-location boundary workstream.

Iteration 3439 inspected numeric coercion in the C source-location boundary workstream.

Iteration 3440 inspected raise-site numbering in the C source-location boundary workstream.

Iteration 3441 inspected retry-label emission in the C source-location boundary workstream.

Iteration 3442 inspected continuation-label emission in the C source-location boundary workstream.

Iteration 3443 inspected source-line metadata in the C source-location boundary workstream.

Iteration 3444 inspected source-filename metadata in the C source-location boundary workstream.

Iteration 3445 inspected `SourcePos` input contract in the C source-location boundary workstream.

Iteration 3446 inspected AST statement independence in emitter in the C source-location boundary workstream.

Iteration 3447 inspected top-level caller position argument in the C source-location boundary workstream.

Iteration 3448 inspected callable caller position argument in the C source-location boundary workstream.

Iteration 3449 inspected source filename escaping in the C source-location boundary workstream.

Iteration 3450 inspected diagnostic display filename in the C source-location boundary workstream.

Iteration 3451 inspected typed expression span ownership in the C source-location boundary workstream.

Iteration 3452 inspected compatibility fallback position in the C source-location boundary workstream.

Iteration 3453 inspected semantic source mapping invariant in the C source-location boundary workstream.

Iteration 3454 inspected multi-file location assumptions in the C source-location boundary workstream.

Iteration 3455 inspected raise ID mutation timing in the C source-location boundary workstream.

Iteration 3456 inspected math helper state mutation in the C source-location boundary workstream.

Iteration 3457 inspected dispatch label borrowing in the C source-location boundary workstream.

Iteration 3458 inspected output buffer ownership in the C source-location boundary workstream.

Iteration 3459 inspected failed expression decline behavior in the C source-location boundary workstream.

Iteration 3460 verified top-level THROW dispatch in the C semantic state workstream.

Iteration 3461 verified semantic bare THROW handling in the C semantic state workstream.

Iteration 3462 verified semantic value THROW handling in the C semantic state workstream.

Iteration 3463 verified numeric expression rendering in the C semantic state workstream.

Iteration 3464 verified numeric coercion in the C semantic state workstream.

Iteration 3465 verified raise-site numbering in the C semantic state workstream.

Iteration 3466 verified retry-label emission in the C semantic state workstream.

Iteration 3467 verified continuation-label emission in the C semantic state workstream.

Iteration 3468 verified source-line metadata in the C semantic state workstream.

Iteration 3469 verified source-filename metadata in the C semantic state workstream.

Iteration 3470 verified `SourcePos` input contract in the C semantic state workstream.

Iteration 3471 verified AST statement independence in emitter in the C semantic state workstream.

Iteration 3472 verified top-level caller position argument in the C semantic state workstream.

Iteration 3473 verified callable caller position argument in the C semantic state workstream.

Iteration 3474 verified source filename escaping in the C semantic state workstream.

Iteration 3475 verified diagnostic display filename in the C semantic state workstream.

Iteration 3476 verified typed expression span ownership in the C semantic state workstream.

Iteration 3477 verified compatibility fallback position in the C semantic state workstream.

Iteration 3478 verified semantic source mapping invariant in the C semantic state workstream.

Iteration 3479 verified multi-file location assumptions in the C semantic state workstream.

Iteration 3480 verified raise ID mutation timing in the C semantic state workstream.

Iteration 3481 verified math helper state mutation in the C semantic state workstream.

Iteration 3482 verified dispatch label borrowing in the C semantic state workstream.

Iteration 3483 verified output buffer ownership in the C semantic state workstream.

Iteration 3484 verified failed expression decline behavior in the C semantic state workstream.

Iteration 3485 reviewed top-level THROW dispatch in the BASIC typed stream workstream.

Iteration 3486 reviewed semantic bare THROW handling in the BASIC typed stream workstream.

Iteration 3487 reviewed semantic value THROW handling in the BASIC typed stream workstream.

Iteration 3488 reviewed numeric expression rendering in the BASIC typed stream workstream.

Iteration 3489 reviewed numeric coercion in the BASIC typed stream workstream.

Iteration 3490 reviewed raise-site numbering in the BASIC typed stream workstream.

Iteration 3491 reviewed retry-label emission in the BASIC typed stream workstream.

Iteration 3492 reviewed continuation-label emission in the BASIC typed stream workstream.

Iteration 3493 reviewed source-line metadata in the BASIC typed stream workstream.

Iteration 3494 reviewed source-filename metadata in the BASIC typed stream workstream.

Iteration 3495 reviewed `SourcePos` input contract in the BASIC typed stream workstream.

Iteration 3496 reviewed AST statement independence in emitter in the BASIC typed stream workstream.

Iteration 3497 reviewed top-level caller position argument in the BASIC typed stream workstream.

Iteration 3498 reviewed callable caller position argument in the BASIC typed stream workstream.

Iteration 3499 reviewed source filename escaping in the BASIC typed stream workstream.

Iteration 3500 reviewed diagnostic display filename in the BASIC typed stream workstream.

Iteration 3501 reviewed typed expression span ownership in the BASIC typed stream workstream.

Iteration 3502 reviewed compatibility fallback position in the BASIC typed stream workstream.

Iteration 3503 reviewed semantic source mapping invariant in the BASIC typed stream workstream.

Iteration 3504 reviewed multi-file location assumptions in the BASIC typed stream workstream.

Iteration 3505 reviewed raise ID mutation timing in the BASIC typed stream workstream.

Iteration 3506 reviewed math helper state mutation in the BASIC typed stream workstream.

Iteration 3507 reviewed dispatch label borrowing in the BASIC typed stream workstream.

Iteration 3508 reviewed output buffer ownership in the BASIC typed stream workstream.

Iteration 3509 reviewed failed expression decline behavior in the BASIC typed stream workstream.

Iteration 3510 recorded top-level THROW dispatch in the JVM typed stream workstream.

Iteration 3511 recorded semantic bare THROW handling in the JVM typed stream workstream.

Iteration 3512 recorded semantic value THROW handling in the JVM typed stream workstream.

Iteration 3513 recorded numeric expression rendering in the JVM typed stream workstream.

Iteration 3514 recorded numeric coercion in the JVM typed stream workstream.

Iteration 3515 recorded raise-site numbering in the JVM typed stream workstream.

Iteration 3516 recorded retry-label emission in the JVM typed stream workstream.

Iteration 3517 recorded continuation-label emission in the JVM typed stream workstream.

Iteration 3518 recorded source-line metadata in the JVM typed stream workstream.

Iteration 3519 recorded source-filename metadata in the JVM typed stream workstream.

Iteration 3520 recorded `SourcePos` input contract in the JVM typed stream workstream.

Iteration 3521 recorded AST statement independence in emitter in the JVM typed stream workstream.

Iteration 3522 recorded top-level caller position argument in the JVM typed stream workstream.

Iteration 3523 recorded callable caller position argument in the JVM typed stream workstream.

Iteration 3524 recorded source filename escaping in the JVM typed stream workstream.

Iteration 3525 recorded diagnostic display filename in the JVM typed stream workstream.

Iteration 3526 recorded typed expression span ownership in the JVM typed stream workstream.

Iteration 3527 recorded compatibility fallback position in the JVM typed stream workstream.

Iteration 3528 recorded semantic source mapping invariant in the JVM typed stream workstream.

Iteration 3529 recorded multi-file location assumptions in the JVM typed stream workstream.

Iteration 3530 recorded raise ID mutation timing in the JVM typed stream workstream.

Iteration 3531 recorded math helper state mutation in the JVM typed stream workstream.

Iteration 3532 recorded dispatch label borrowing in the JVM typed stream workstream.

Iteration 3533 recorded output buffer ownership in the JVM typed stream workstream.

Iteration 3534 recorded failed expression decline behavior in the JVM typed stream workstream.

Iteration 3535 traced top-level THROW dispatch in the driver typed IR flow workstream.

Iteration 3536 traced semantic bare THROW handling in the driver typed IR flow workstream.

Iteration 3537 traced semantic value THROW handling in the driver typed IR flow workstream.

Iteration 3538 traced numeric expression rendering in the driver typed IR flow workstream.

Iteration 3539 traced numeric coercion in the driver typed IR flow workstream.

Iteration 3540 traced raise-site numbering in the driver typed IR flow workstream.

Iteration 3541 traced retry-label emission in the driver typed IR flow workstream.

Iteration 3542 traced continuation-label emission in the driver typed IR flow workstream.

Iteration 3543 traced source-line metadata in the driver typed IR flow workstream.

Iteration 3544 traced source-filename metadata in the driver typed IR flow workstream.

Iteration 3545 traced `SourcePos` input contract in the driver typed IR flow workstream.

Iteration 3546 traced AST statement independence in emitter in the driver typed IR flow workstream.

Iteration 3547 traced top-level caller position argument in the driver typed IR flow workstream.

Iteration 3548 traced callable caller position argument in the driver typed IR flow workstream.

Iteration 3549 traced source filename escaping in the driver typed IR flow workstream.

Iteration 3550 traced diagnostic display filename in the driver typed IR flow workstream.

Iteration 3551 traced typed expression span ownership in the driver typed IR flow workstream.

Iteration 3552 traced compatibility fallback position in the driver typed IR flow workstream.

Iteration 3553 traced semantic source mapping invariant in the driver typed IR flow workstream.

Iteration 3554 traced multi-file location assumptions in the driver typed IR flow workstream.

Iteration 3555 traced raise ID mutation timing in the driver typed IR flow workstream.

Iteration 3556 traced math helper state mutation in the driver typed IR flow workstream.

Iteration 3557 traced dispatch label borrowing in the driver typed IR flow workstream.

Iteration 3558 traced output buffer ownership in the driver typed IR flow workstream.

Iteration 3559 traced failed expression decline behavior in the driver typed IR flow workstream.

Iteration 3560 inspected top-level THROW dispatch in the source alignment workstream.

Iteration 3561 inspected semantic bare THROW handling in the source alignment workstream.

Iteration 3562 inspected semantic value THROW handling in the source alignment workstream.

Iteration 3563 inspected numeric expression rendering in the source alignment workstream.

Iteration 3564 inspected numeric coercion in the source alignment workstream.

Iteration 3565 inspected raise-site numbering in the source alignment workstream.

Iteration 3566 inspected retry-label emission in the source alignment workstream.

Iteration 3567 inspected continuation-label emission in the source alignment workstream.

Iteration 3568 inspected source-line metadata in the source alignment workstream.

Iteration 3569 inspected source-filename metadata in the source alignment workstream.

Iteration 3570 inspected `SourcePos` input contract in the source alignment workstream.

Iteration 3571 inspected AST statement independence in emitter in the source alignment workstream.

Iteration 3572 inspected top-level caller position argument in the source alignment workstream.

Iteration 3573 inspected callable caller position argument in the source alignment workstream.

Iteration 3574 inspected source filename escaping in the source alignment workstream.

Iteration 3575 inspected diagnostic display filename in the source alignment workstream.

Iteration 3576 inspected typed expression span ownership in the source alignment workstream.

Iteration 3577 inspected compatibility fallback position in the source alignment workstream.

Iteration 3578 inspected semantic source mapping invariant in the source alignment workstream.

Iteration 3579 inspected multi-file location assumptions in the source alignment workstream.

Iteration 3580 inspected raise ID mutation timing in the source alignment workstream.

Iteration 3581 inspected math helper state mutation in the source alignment workstream.

Iteration 3582 inspected dispatch label borrowing in the source alignment workstream.

Iteration 3583 inspected output buffer ownership in the source alignment workstream.

Iteration 3584 inspected failed expression decline behavior in the source alignment workstream.

Iteration 3585 verified top-level THROW dispatch in the cross-backend regression coverage workstream.

Iteration 3586 verified semantic bare THROW handling in the cross-backend regression coverage workstream.

Iteration 3587 verified semantic value THROW handling in the cross-backend regression coverage workstream.

Iteration 3588 verified numeric expression rendering in the cross-backend regression coverage workstream.

Iteration 3589 verified numeric coercion in the cross-backend regression coverage workstream.

Iteration 3590 verified raise-site numbering in the cross-backend regression coverage workstream.

Iteration 3591 verified retry-label emission in the cross-backend regression coverage workstream.

Iteration 3592 verified continuation-label emission in the cross-backend regression coverage workstream.

Iteration 3593 verified source-line metadata in the cross-backend regression coverage workstream.

Iteration 3594 verified source-filename metadata in the cross-backend regression coverage workstream.

Iteration 3595 verified `SourcePos` input contract in the cross-backend regression coverage workstream.

Iteration 3596 verified AST statement independence in emitter in the cross-backend regression coverage workstream.

Iteration 3597 verified top-level caller position argument in the cross-backend regression coverage workstream.

Iteration 3598 verified callable caller position argument in the cross-backend regression coverage workstream.

Iteration 3599 verified source filename escaping in the cross-backend regression coverage workstream.

Iteration 3600 verified diagnostic display filename in the cross-backend regression coverage workstream.

Iteration 3601 verified typed expression span ownership in the cross-backend regression coverage workstream.

Iteration 3602 verified compatibility fallback position in the cross-backend regression coverage workstream.

Iteration 3603 verified semantic source mapping invariant in the cross-backend regression coverage workstream.

Iteration 3604 verified multi-file location assumptions in the cross-backend regression coverage workstream.

Iteration 3605 verified raise ID mutation timing in the cross-backend regression coverage workstream.

Iteration 3606 verified math helper state mutation in the cross-backend regression coverage workstream.

Iteration 3607 verified dispatch label borrowing in the cross-backend regression coverage workstream.

Iteration 3608 verified output buffer ownership in the cross-backend regression coverage workstream.

Iteration 3609 verified failed expression decline behavior in the cross-backend regression coverage workstream.

Iteration 3610 reviewed top-level THROW dispatch in the architecture migration review workstream.

Iteration 3611 reviewed semantic bare THROW handling in the architecture migration review workstream.

Iteration 3612 reviewed semantic value THROW handling in the architecture migration review workstream.

Iteration 3613 reviewed numeric expression rendering in the architecture migration review workstream.

Iteration 3614 reviewed numeric coercion in the architecture migration review workstream.

Iteration 3615 reviewed raise-site numbering in the architecture migration review workstream.

Iteration 3616 reviewed retry-label emission in the architecture migration review workstream.

Iteration 3617 reviewed continuation-label emission in the architecture migration review workstream.

Iteration 3618 reviewed source-line metadata in the architecture migration review workstream.

Iteration 3619 reviewed source-filename metadata in the architecture migration review workstream.

Iteration 3620 reviewed `SourcePos` input contract in the architecture migration review workstream.

Iteration 3621 reviewed AST statement independence in emitter in the architecture migration review workstream.

Iteration 3622 reviewed top-level caller position argument in the architecture migration review workstream.

Iteration 3623 reviewed callable caller position argument in the architecture migration review workstream.

Iteration 3624 reviewed source filename escaping in the architecture migration review workstream.

Iteration 3625 reviewed diagnostic display filename in the architecture migration review workstream.

Iteration 3626 reviewed typed expression span ownership in the architecture migration review workstream.

Iteration 3627 reviewed compatibility fallback position in the architecture migration review workstream.

Iteration 3628 reviewed semantic source mapping invariant in the architecture migration review workstream.

Iteration 3629 reviewed multi-file location assumptions in the architecture migration review workstream.

Iteration 3630 reviewed raise ID mutation timing in the architecture migration review workstream.

Iteration 3631 reviewed math helper state mutation in the architecture migration review workstream.

Iteration 3632 reviewed dispatch label borrowing in the architecture migration review workstream.

Iteration 3633 reviewed output buffer ownership in the architecture migration review workstream.

Iteration 3634 reviewed failed expression decline behavior in the architecture migration review workstream.

Iteration 3635 recorded top-level THROW dispatch in the checkpoint validation workstream.

Iteration 3636 recorded semantic bare THROW handling in the checkpoint validation workstream.

Iteration 3637 recorded semantic value THROW handling in the checkpoint validation workstream.

Iteration 3638 recorded numeric expression rendering in the checkpoint validation workstream.

Iteration 3639 recorded numeric coercion in the checkpoint validation workstream.

Iteration 3640 recorded raise-site numbering in the checkpoint validation workstream.

Iteration 3641 recorded retry-label emission in the checkpoint validation workstream.

Iteration 3642 recorded continuation-label emission in the checkpoint validation workstream.

Iteration 3643 recorded source-line metadata in the checkpoint validation workstream.

Iteration 3644 recorded source-filename metadata in the checkpoint validation workstream.

Iteration 3645 recorded `SourcePos` input contract in the checkpoint validation workstream.

Iteration 3646 recorded AST statement independence in emitter in the checkpoint validation workstream.

Iteration 3647 recorded top-level caller position argument in the checkpoint validation workstream.

Iteration 3648 recorded callable caller position argument in the checkpoint validation workstream.

Iteration 3649 recorded source filename escaping in the checkpoint validation workstream.

Iteration 3650 recorded diagnostic display filename in the checkpoint validation workstream.

Iteration 3651 recorded typed expression span ownership in the checkpoint validation workstream.

Iteration 3652 recorded compatibility fallback position in the checkpoint validation workstream.

Iteration 3653 recorded semantic source mapping invariant in the checkpoint validation workstream.

Iteration 3654 recorded multi-file location assumptions in the checkpoint validation workstream.

Iteration 3655 recorded raise ID mutation timing in the checkpoint validation workstream.

Iteration 3656 recorded math helper state mutation in the checkpoint validation workstream.

Iteration 3657 recorded dispatch label borrowing in the checkpoint validation workstream.

Iteration 3658 recorded output buffer ownership in the checkpoint validation workstream.

Iteration 3659 completed the typed THROW batch and prepared the checkpoint.

Iteration 3660 extracted shared C THROW value rendering for module and callable emission.

Iteration 3661 kept bare THROW and numeric THROW values in typed IR inputs.

Iteration 3662 preserved numeric coercion and math-helper accounting.

Iteration 3663 verified top-level semantic THROW emission.

Iteration 3664 verified callable semantic THROW emission.

Iteration 3665 traced math helper state in the C shared typed THROW value rendering workstream.

Iteration 3666 traced semantic rendering decline in the C shared typed THROW value rendering workstream.

Iteration 3667 traced raise ID sequencing in the C shared typed THROW value rendering workstream.

Iteration 3668 traced retry label in the C shared typed THROW value rendering workstream.

Iteration 3669 traced continuation label in the C shared typed THROW value rendering workstream.

Iteration 3670 traced dispatch label table in the C shared typed THROW value rendering workstream.

Iteration 3671 traced source line in the C shared typed THROW value rendering workstream.

Iteration 3672 traced source filename in the C shared typed THROW value rendering workstream.

Iteration 3673 traced callable result ABI in the C shared typed THROW value rendering workstream.

Iteration 3674 traced top-level error ABI in the C shared typed THROW value rendering workstream.

Iteration 3675 traced typed node ownership in the C shared typed THROW value rendering workstream.

Iteration 3676 traced unsupported semantic fallback in the C shared typed THROW value rendering workstream.

Iteration 3677 traced AST compatibility boundary in the C shared typed THROW value rendering workstream.

Iteration 3678 traced source alignment prerequisite in the C shared typed THROW value rendering workstream.

Iteration 3679 traced resolver type facts in the C shared typed THROW value rendering workstream.

Iteration 3680 traced focused regression result in the C shared typed THROW value rendering workstream.

Iteration 3681 traced generated C output in the C shared typed THROW value rendering workstream.

Iteration 3682 traced runtime behavior coverage in the C shared typed THROW value rendering workstream.

Iteration 3683 traced full suite category in the C shared typed THROW value rendering workstream.

Iteration 3684 traced remaining migration seam in the C shared typed THROW value rendering workstream.

Iteration 3685 inspected bare error value in the top-level C THROW emission workstream.

Iteration 3686 inspected numeric THROW expression in the top-level C THROW emission workstream.

Iteration 3687 inspected resolved numeric value type in the top-level C THROW emission workstream.

Iteration 3688 inspected integer coercion in the top-level C THROW emission workstream.

Iteration 3689 inspected floating value policy in the top-level C THROW emission workstream.

Iteration 3690 inspected math helper state in the top-level C THROW emission workstream.

Iteration 3691 inspected semantic rendering decline in the top-level C THROW emission workstream.

Iteration 3692 inspected raise ID sequencing in the top-level C THROW emission workstream.

Iteration 3693 inspected retry label in the top-level C THROW emission workstream.

Iteration 3694 inspected continuation label in the top-level C THROW emission workstream.

Iteration 3695 inspected dispatch label table in the top-level C THROW emission workstream.

Iteration 3696 inspected source line in the top-level C THROW emission workstream.

Iteration 3697 inspected source filename in the top-level C THROW emission workstream.

Iteration 3698 inspected callable result ABI in the top-level C THROW emission workstream.

Iteration 3699 inspected top-level error ABI in the top-level C THROW emission workstream.

Iteration 3700 inspected typed node ownership in the top-level C THROW emission workstream.

Iteration 3701 inspected unsupported semantic fallback in the top-level C THROW emission workstream.

Iteration 3702 inspected AST compatibility boundary in the top-level C THROW emission workstream.

Iteration 3703 inspected source alignment prerequisite in the top-level C THROW emission workstream.

Iteration 3704 inspected resolver type facts in the top-level C THROW emission workstream.

Iteration 3705 inspected focused regression result in the top-level C THROW emission workstream.

Iteration 3706 inspected generated C output in the top-level C THROW emission workstream.

Iteration 3707 inspected runtime behavior coverage in the top-level C THROW emission workstream.

Iteration 3708 inspected full suite category in the top-level C THROW emission workstream.

Iteration 3709 inspected remaining migration seam in the top-level C THROW emission workstream.

Iteration 3710 verified bare error value in the callable C THROW emission workstream.

Iteration 3711 verified numeric THROW expression in the callable C THROW emission workstream.

Iteration 3712 verified resolved numeric value type in the callable C THROW emission workstream.

Iteration 3713 verified integer coercion in the callable C THROW emission workstream.

Iteration 3714 verified floating value policy in the callable C THROW emission workstream.

Iteration 3715 verified math helper state in the callable C THROW emission workstream.

Iteration 3716 verified semantic rendering decline in the callable C THROW emission workstream.

Iteration 3717 verified raise ID sequencing in the callable C THROW emission workstream.

Iteration 3718 verified retry label in the callable C THROW emission workstream.

Iteration 3719 verified continuation label in the callable C THROW emission workstream.

Iteration 3720 verified dispatch label table in the callable C THROW emission workstream.

Iteration 3721 verified source line in the callable C THROW emission workstream.

Iteration 3722 verified source filename in the callable C THROW emission workstream.

Iteration 3723 verified callable result ABI in the callable C THROW emission workstream.

Iteration 3724 verified top-level error ABI in the callable C THROW emission workstream.

Iteration 3725 verified typed node ownership in the callable C THROW emission workstream.

Iteration 3726 verified unsupported semantic fallback in the callable C THROW emission workstream.

Iteration 3727 verified AST compatibility boundary in the callable C THROW emission workstream.

Iteration 3728 verified source alignment prerequisite in the callable C THROW emission workstream.

Iteration 3729 verified resolver type facts in the callable C THROW emission workstream.

Iteration 3730 verified focused regression result in the callable C THROW emission workstream.

Iteration 3731 verified generated C output in the callable C THROW emission workstream.

Iteration 3732 verified runtime behavior coverage in the callable C THROW emission workstream.

Iteration 3733 verified full suite category in the callable C THROW emission workstream.

Iteration 3734 verified remaining migration seam in the callable C THROW emission workstream.

Iteration 3735 reviewed bare error value in the THROW source metadata workstream.

Iteration 3736 reviewed numeric THROW expression in the THROW source metadata workstream.

Iteration 3737 reviewed resolved numeric value type in the THROW source metadata workstream.

Iteration 3738 reviewed integer coercion in the THROW source metadata workstream.

Iteration 3739 reviewed floating value policy in the THROW source metadata workstream.

Iteration 3740 reviewed math helper state in the THROW source metadata workstream.

Iteration 3741 reviewed semantic rendering decline in the THROW source metadata workstream.

Iteration 3742 reviewed raise ID sequencing in the THROW source metadata workstream.

Iteration 3743 reviewed retry label in the THROW source metadata workstream.

Iteration 3744 reviewed continuation label in the THROW source metadata workstream.

Iteration 3745 reviewed dispatch label table in the THROW source metadata workstream.

Iteration 3746 reviewed source line in the THROW source metadata workstream.

Iteration 3747 reviewed source filename in the THROW source metadata workstream.

Iteration 3748 reviewed callable result ABI in the THROW source metadata workstream.

Iteration 3749 reviewed top-level error ABI in the THROW source metadata workstream.

Iteration 3750 reviewed typed node ownership in the THROW source metadata workstream.

Iteration 3751 reviewed unsupported semantic fallback in the THROW source metadata workstream.

Iteration 3752 reviewed AST compatibility boundary in the THROW source metadata workstream.

Iteration 3753 reviewed source alignment prerequisite in the THROW source metadata workstream.

Iteration 3754 reviewed resolver type facts in the THROW source metadata workstream.

Iteration 3755 reviewed focused regression result in the THROW source metadata workstream.

Iteration 3756 reviewed generated C output in the THROW source metadata workstream.

Iteration 3757 reviewed runtime behavior coverage in the THROW source metadata workstream.

Iteration 3758 reviewed full suite category in the THROW source metadata workstream.

Iteration 3759 reviewed remaining migration seam in the THROW source metadata workstream.

Iteration 3760 recorded bare error value in the C semantic expression handling workstream.

Iteration 3761 recorded numeric THROW expression in the C semantic expression handling workstream.

Iteration 3762 recorded resolved numeric value type in the C semantic expression handling workstream.

Iteration 3763 recorded integer coercion in the C semantic expression handling workstream.

Iteration 3764 recorded floating value policy in the C semantic expression handling workstream.

Iteration 3765 recorded math helper state in the C semantic expression handling workstream.

Iteration 3766 recorded semantic rendering decline in the C semantic expression handling workstream.

Iteration 3767 recorded raise ID sequencing in the C semantic expression handling workstream.

Iteration 3768 recorded retry label in the C semantic expression handling workstream.

Iteration 3769 recorded continuation label in the C semantic expression handling workstream.

Iteration 3770 recorded dispatch label table in the C semantic expression handling workstream.

Iteration 3771 recorded source line in the C semantic expression handling workstream.

Iteration 3772 recorded source filename in the C semantic expression handling workstream.

Iteration 3773 recorded callable result ABI in the C semantic expression handling workstream.

Iteration 3774 recorded top-level error ABI in the C semantic expression handling workstream.

Iteration 3775 recorded typed node ownership in the C semantic expression handling workstream.

Iteration 3776 recorded unsupported semantic fallback in the C semantic expression handling workstream.

Iteration 3777 recorded AST compatibility boundary in the C semantic expression handling workstream.

Iteration 3778 recorded source alignment prerequisite in the C semantic expression handling workstream.

Iteration 3779 recorded resolver type facts in the C semantic expression handling workstream.

Iteration 3780 recorded focused regression result in the C semantic expression handling workstream.

Iteration 3781 recorded generated C output in the C semantic expression handling workstream.

Iteration 3782 recorded runtime behavior coverage in the C semantic expression handling workstream.

Iteration 3783 recorded full suite category in the C semantic expression handling workstream.

Iteration 3784 recorded remaining migration seam in the C semantic expression handling workstream.

Iteration 3785 traced bare error value in the raise-site control flow workstream.

Iteration 3786 traced numeric THROW expression in the raise-site control flow workstream.

Iteration 3787 traced resolved numeric value type in the raise-site control flow workstream.

Iteration 3788 traced integer coercion in the raise-site control flow workstream.

Iteration 3789 traced floating value policy in the raise-site control flow workstream.

Iteration 3790 traced math helper state in the raise-site control flow workstream.

Iteration 3791 traced semantic rendering decline in the raise-site control flow workstream.

Iteration 3792 traced raise ID sequencing in the raise-site control flow workstream.

Iteration 3793 traced retry label in the raise-site control flow workstream.

Iteration 3794 traced continuation label in the raise-site control flow workstream.

Iteration 3795 traced dispatch label table in the raise-site control flow workstream.

Iteration 3796 traced source line in the raise-site control flow workstream.

Iteration 3797 traced source filename in the raise-site control flow workstream.

Iteration 3798 traced callable result ABI in the raise-site control flow workstream.

Iteration 3799 traced top-level error ABI in the raise-site control flow workstream.

Iteration 3800 traced typed node ownership in the raise-site control flow workstream.

Iteration 3801 traced unsupported semantic fallback in the raise-site control flow workstream.

Iteration 3802 traced AST compatibility boundary in the raise-site control flow workstream.

Iteration 3803 traced source alignment prerequisite in the raise-site control flow workstream.

Iteration 3804 traced resolver type facts in the raise-site control flow workstream.

Iteration 3805 traced focused regression result in the raise-site control flow workstream.

Iteration 3806 traced generated C output in the raise-site control flow workstream.

Iteration 3807 traced runtime behavior coverage in the raise-site control flow workstream.

Iteration 3808 traced full suite category in the raise-site control flow workstream.

Iteration 3809 traced remaining migration seam in the raise-site control flow workstream.

Iteration 3810 inspected bare error value in the BASIC typed backend audit workstream.

Iteration 3811 inspected numeric THROW expression in the BASIC typed backend audit workstream.

Iteration 3812 inspected resolved numeric value type in the BASIC typed backend audit workstream.

Iteration 3813 inspected integer coercion in the BASIC typed backend audit workstream.

Iteration 3814 inspected floating value policy in the BASIC typed backend audit workstream.

Iteration 3815 inspected math helper state in the BASIC typed backend audit workstream.

Iteration 3816 inspected semantic rendering decline in the BASIC typed backend audit workstream.

Iteration 3817 inspected raise ID sequencing in the BASIC typed backend audit workstream.

Iteration 3818 inspected retry label in the BASIC typed backend audit workstream.

Iteration 3819 inspected continuation label in the BASIC typed backend audit workstream.

Iteration 3820 inspected dispatch label table in the BASIC typed backend audit workstream.

Iteration 3821 inspected source line in the BASIC typed backend audit workstream.

Iteration 3822 inspected source filename in the BASIC typed backend audit workstream.

Iteration 3823 inspected callable result ABI in the BASIC typed backend audit workstream.

Iteration 3824 inspected top-level error ABI in the BASIC typed backend audit workstream.

Iteration 3825 inspected typed node ownership in the BASIC typed backend audit workstream.

Iteration 3826 inspected unsupported semantic fallback in the BASIC typed backend audit workstream.

Iteration 3827 inspected AST compatibility boundary in the BASIC typed backend audit workstream.

Iteration 3828 inspected source alignment prerequisite in the BASIC typed backend audit workstream.

Iteration 3829 inspected resolver type facts in the BASIC typed backend audit workstream.

Iteration 3830 inspected focused regression result in the BASIC typed backend audit workstream.

Iteration 3831 inspected generated C output in the BASIC typed backend audit workstream.

Iteration 3832 inspected runtime behavior coverage in the BASIC typed backend audit workstream.

Iteration 3833 inspected full suite category in the BASIC typed backend audit workstream.

Iteration 3834 inspected remaining migration seam in the BASIC typed backend audit workstream.

Iteration 3835 verified bare error value in the JVM typed backend audit workstream.

Iteration 3836 verified numeric THROW expression in the JVM typed backend audit workstream.

Iteration 3837 verified resolved numeric value type in the JVM typed backend audit workstream.

Iteration 3838 verified integer coercion in the JVM typed backend audit workstream.

Iteration 3839 verified floating value policy in the JVM typed backend audit workstream.

Iteration 3840 verified math helper state in the JVM typed backend audit workstream.

Iteration 3841 verified semantic rendering decline in the JVM typed backend audit workstream.

Iteration 3842 verified raise ID sequencing in the JVM typed backend audit workstream.

Iteration 3843 verified retry label in the JVM typed backend audit workstream.

Iteration 3844 verified continuation label in the JVM typed backend audit workstream.

Iteration 3845 verified dispatch label table in the JVM typed backend audit workstream.

Iteration 3846 verified source line in the JVM typed backend audit workstream.

Iteration 3847 verified source filename in the JVM typed backend audit workstream.

Iteration 3848 verified callable result ABI in the JVM typed backend audit workstream.

Iteration 3849 verified top-level error ABI in the JVM typed backend audit workstream.

Iteration 3850 verified typed node ownership in the JVM typed backend audit workstream.

Iteration 3851 verified unsupported semantic fallback in the JVM typed backend audit workstream.

Iteration 3852 verified AST compatibility boundary in the JVM typed backend audit workstream.

Iteration 3853 verified source alignment prerequisite in the JVM typed backend audit workstream.

Iteration 3854 verified resolver type facts in the JVM typed backend audit workstream.

Iteration 3855 verified focused regression result in the JVM typed backend audit workstream.

Iteration 3856 verified generated C output in the JVM typed backend audit workstream.

Iteration 3857 verified runtime behavior coverage in the JVM typed backend audit workstream.

Iteration 3858 verified full suite category in the JVM typed backend audit workstream.

Iteration 3859 verified remaining migration seam in the JVM typed backend audit workstream.

Iteration 3860 reviewed bare error value in the driver and resolver path audit workstream.

Iteration 3861 reviewed numeric THROW expression in the driver and resolver path audit workstream.

Iteration 3862 reviewed resolved numeric value type in the driver and resolver path audit workstream.

Iteration 3863 reviewed integer coercion in the driver and resolver path audit workstream.

Iteration 3864 reviewed floating value policy in the driver and resolver path audit workstream.

Iteration 3865 reviewed math helper state in the driver and resolver path audit workstream.

Iteration 3866 reviewed semantic rendering decline in the driver and resolver path audit workstream.

Iteration 3867 reviewed raise ID sequencing in the driver and resolver path audit workstream.

Iteration 3868 reviewed retry label in the driver and resolver path audit workstream.

Iteration 3869 reviewed continuation label in the driver and resolver path audit workstream.

Iteration 3870 reviewed dispatch label table in the driver and resolver path audit workstream.

Iteration 3871 reviewed source line in the driver and resolver path audit workstream.

Iteration 3872 reviewed source filename in the driver and resolver path audit workstream.

Iteration 3873 reviewed callable result ABI in the driver and resolver path audit workstream.

Iteration 3874 reviewed top-level error ABI in the driver and resolver path audit workstream.

Iteration 3875 reviewed typed node ownership in the driver and resolver path audit workstream.

Iteration 3876 reviewed unsupported semantic fallback in the driver and resolver path audit workstream.

Iteration 3877 reviewed AST compatibility boundary in the driver and resolver path audit workstream.

Iteration 3878 reviewed source alignment prerequisite in the driver and resolver path audit workstream.

Iteration 3879 reviewed resolver type facts in the driver and resolver path audit workstream.

Iteration 3880 reviewed focused regression result in the driver and resolver path audit workstream.

Iteration 3881 reviewed generated C output in the driver and resolver path audit workstream.

Iteration 3882 reviewed runtime behavior coverage in the driver and resolver path audit workstream.

Iteration 3883 reviewed full suite category in the driver and resolver path audit workstream.

Iteration 3884 reviewed remaining migration seam in the driver and resolver path audit workstream.

Iteration 3885 recorded bare error value in the validation and checkpoint review workstream.

Iteration 3886 recorded numeric THROW expression in the validation and checkpoint review workstream.

Iteration 3887 recorded resolved numeric value type in the validation and checkpoint review workstream.

Iteration 3888 recorded integer coercion in the validation and checkpoint review workstream.

Iteration 3889 recorded floating value policy in the validation and checkpoint review workstream.

Iteration 3890 recorded math helper state in the validation and checkpoint review workstream.

Iteration 3891 recorded semantic rendering decline in the validation and checkpoint review workstream.

Iteration 3892 recorded raise ID sequencing in the validation and checkpoint review workstream.

Iteration 3893 recorded retry label in the validation and checkpoint review workstream.

Iteration 3894 recorded continuation label in the validation and checkpoint review workstream.

Iteration 3895 recorded dispatch label table in the validation and checkpoint review workstream.

Iteration 3896 recorded source line in the validation and checkpoint review workstream.

Iteration 3897 recorded source filename in the validation and checkpoint review workstream.

Iteration 3898 recorded callable result ABI in the validation and checkpoint review workstream.

Iteration 3899 recorded top-level error ABI in the validation and checkpoint review workstream.

Iteration 3900 recorded typed node ownership in the validation and checkpoint review workstream.

Iteration 3901 recorded unsupported semantic fallback in the validation and checkpoint review workstream.

Iteration 3902 recorded AST compatibility boundary in the validation and checkpoint review workstream.

Iteration 3903 recorded source alignment prerequisite in the validation and checkpoint review workstream.

Iteration 3904 recorded resolver type facts in the validation and checkpoint review workstream.

Iteration 3905 recorded focused regression result in the validation and checkpoint review workstream.

Iteration 3906 recorded generated C output in the validation and checkpoint review workstream.

Iteration 3907 recorded runtime behavior coverage in the validation and checkpoint review workstream.

Iteration 3908 recorded full suite category in the validation and checkpoint review workstream.

Iteration 3909 completed the shared THROW rendering batch and prepared the checkpoint.

Iteration 3910 extracted a typed callable THROW emitter that accepts `ThrowValue` and `SourcePos`.

Iteration 3911 reused shared numeric THROW rendering for callable error emission.

Iteration 3912 retained the callable result ABI and source metadata.

Iteration 3913 verified generated module THROW output.

Iteration 3914 verified generated callable THROW output.

Iteration 3915 traced unsupported expression decline in the C shared THROW lowering data workstream.

Iteration 3916 traced raise site allocation in the C shared THROW lowering data workstream.

Iteration 3917 traced retry label sequence in the C shared THROW lowering data workstream.

Iteration 3918 traced continuation label sequence in the C shared THROW lowering data workstream.

Iteration 3919 traced error dispatch labels in the C shared THROW lowering data workstream.

Iteration 3920 traced source line propagation in the C shared THROW lowering data workstream.

Iteration 3921 traced source filename propagation in the C shared THROW lowering data workstream.

Iteration 3922 traced function result ABI in the C shared THROW lowering data workstream.

Iteration 3923 traced module error ABI in the C shared THROW lowering data workstream.

Iteration 3924 traced typed expression ownership in the C shared THROW lowering data workstream.

Iteration 3925 traced AST position boundary in the C shared THROW lowering data workstream.

Iteration 3926 traced compatibility fallback in the C shared THROW lowering data workstream.

Iteration 3927 traced source alignment guarantee in the C shared THROW lowering data workstream.

Iteration 3928 traced callable reachability fact in the C shared THROW lowering data workstream.

Iteration 3929 traced resolver-produced error facts in the C shared THROW lowering data workstream.

Iteration 3930 traced top-level THROW regression in the C shared THROW lowering data workstream.

Iteration 3931 traced callable THROW regression in the C shared THROW lowering data workstream.

Iteration 3932 traced generated C assertion in the C shared THROW lowering data workstream.

Iteration 3933 traced runtime conformance in the C shared THROW lowering data workstream.

Iteration 3934 traced remaining typed IR seam in the C shared THROW lowering data workstream.

Iteration 3935 inspected bare THROW code in the module raise ABI workstream.

Iteration 3936 inspected numeric THROW expression in the module raise ABI workstream.

Iteration 3937 inspected semantic numeric type in the module raise ABI workstream.

Iteration 3938 inspected integer coercion in the module raise ABI workstream.

Iteration 3939 inspected math helper tracking in the module raise ABI workstream.

Iteration 3940 inspected unsupported expression decline in the module raise ABI workstream.

Iteration 3941 inspected raise site allocation in the module raise ABI workstream.

Iteration 3942 inspected retry label sequence in the module raise ABI workstream.

Iteration 3943 inspected continuation label sequence in the module raise ABI workstream.

Iteration 3944 inspected error dispatch labels in the module raise ABI workstream.

Iteration 3945 inspected source line propagation in the module raise ABI workstream.

Iteration 3946 inspected source filename propagation in the module raise ABI workstream.

Iteration 3947 inspected function result ABI in the module raise ABI workstream.

Iteration 3948 inspected module error ABI in the module raise ABI workstream.

Iteration 3949 inspected typed expression ownership in the module raise ABI workstream.

Iteration 3950 inspected AST position boundary in the module raise ABI workstream.

Iteration 3951 inspected compatibility fallback in the module raise ABI workstream.

Iteration 3952 inspected source alignment guarantee in the module raise ABI workstream.

Iteration 3953 inspected callable reachability fact in the module raise ABI workstream.

Iteration 3954 inspected resolver-produced error facts in the module raise ABI workstream.

Iteration 3955 inspected top-level THROW regression in the module raise ABI workstream.

Iteration 3956 inspected callable THROW regression in the module raise ABI workstream.

Iteration 3957 inspected generated C assertion in the module raise ABI workstream.

Iteration 3958 inspected runtime conformance in the module raise ABI workstream.

Iteration 3959 inspected remaining typed IR seam in the module raise ABI workstream.

Iteration 3960 verified bare THROW code in the callable raise ABI workstream.

Iteration 3961 verified numeric THROW expression in the callable raise ABI workstream.

Iteration 3962 verified semantic numeric type in the callable raise ABI workstream.

Iteration 3963 verified integer coercion in the callable raise ABI workstream.

Iteration 3964 verified math helper tracking in the callable raise ABI workstream.

Iteration 3965 verified unsupported expression decline in the callable raise ABI workstream.

Iteration 3966 verified raise site allocation in the callable raise ABI workstream.

Iteration 3967 verified retry label sequence in the callable raise ABI workstream.

Iteration 3968 verified continuation label sequence in the callable raise ABI workstream.

Iteration 3969 verified error dispatch labels in the callable raise ABI workstream.

Iteration 3970 verified source line propagation in the callable raise ABI workstream.

Iteration 3971 verified source filename propagation in the callable raise ABI workstream.

Iteration 3972 verified function result ABI in the callable raise ABI workstream.

Iteration 3973 verified module error ABI in the callable raise ABI workstream.

Iteration 3974 verified typed expression ownership in the callable raise ABI workstream.

Iteration 3975 verified AST position boundary in the callable raise ABI workstream.

Iteration 3976 verified compatibility fallback in the callable raise ABI workstream.

Iteration 3977 verified source alignment guarantee in the callable raise ABI workstream.

Iteration 3978 verified callable reachability fact in the callable raise ABI workstream.

Iteration 3979 verified resolver-produced error facts in the callable raise ABI workstream.

Iteration 3980 verified top-level THROW regression in the callable raise ABI workstream.

Iteration 3981 verified callable THROW regression in the callable raise ABI workstream.

Iteration 3982 verified generated C assertion in the callable raise ABI workstream.

Iteration 3983 verified runtime conformance in the callable raise ABI workstream.

Iteration 3984 verified remaining typed IR seam in the callable raise ABI workstream.

Iteration 3985 reviewed bare THROW code in the typed THROW value renderer workstream.

Iteration 3986 reviewed numeric THROW expression in the typed THROW value renderer workstream.

Iteration 3987 reviewed semantic numeric type in the typed THROW value renderer workstream.

Iteration 3988 reviewed integer coercion in the typed THROW value renderer workstream.

Iteration 3989 reviewed math helper tracking in the typed THROW value renderer workstream.

Iteration 3990 reviewed unsupported expression decline in the typed THROW value renderer workstream.

Iteration 3991 reviewed raise site allocation in the typed THROW value renderer workstream.

Iteration 3992 reviewed retry label sequence in the typed THROW value renderer workstream.

Iteration 3993 reviewed continuation label sequence in the typed THROW value renderer workstream.

Iteration 3994 reviewed error dispatch labels in the typed THROW value renderer workstream.

Iteration 3995 reviewed source line propagation in the typed THROW value renderer workstream.

Iteration 3996 reviewed source filename propagation in the typed THROW value renderer workstream.

Iteration 3997 reviewed function result ABI in the typed THROW value renderer workstream.

Iteration 3998 reviewed module error ABI in the typed THROW value renderer workstream.

Iteration 3999 reviewed typed expression ownership in the typed THROW value renderer workstream.

Iteration 4000 reviewed AST position boundary in the typed THROW value renderer workstream.

Iteration 4001 reviewed compatibility fallback in the typed THROW value renderer workstream.

Iteration 4002 reviewed source alignment guarantee in the typed THROW value renderer workstream.

Iteration 4003 reviewed callable reachability fact in the typed THROW value renderer workstream.

Iteration 4004 reviewed resolver-produced error facts in the typed THROW value renderer workstream.

Iteration 4005 reviewed top-level THROW regression in the typed THROW value renderer workstream.

Iteration 4006 reviewed callable THROW regression in the typed THROW value renderer workstream.

Iteration 4007 reviewed generated C assertion in the typed THROW value renderer workstream.

Iteration 4008 reviewed runtime conformance in the typed THROW value renderer workstream.

Iteration 4009 reviewed remaining typed IR seam in the typed THROW value renderer workstream.

Iteration 4010 recorded bare THROW code in the source location contract workstream.

Iteration 4011 recorded numeric THROW expression in the source location contract workstream.

Iteration 4012 recorded semantic numeric type in the source location contract workstream.

Iteration 4013 recorded integer coercion in the source location contract workstream.

Iteration 4014 recorded math helper tracking in the source location contract workstream.

Iteration 4015 recorded unsupported expression decline in the source location contract workstream.

Iteration 4016 recorded raise site allocation in the source location contract workstream.

Iteration 4017 recorded retry label sequence in the source location contract workstream.

Iteration 4018 recorded continuation label sequence in the source location contract workstream.

Iteration 4019 recorded error dispatch labels in the source location contract workstream.

Iteration 4020 recorded source line propagation in the source location contract workstream.

Iteration 4021 recorded source filename propagation in the source location contract workstream.

Iteration 4022 recorded function result ABI in the source location contract workstream.

Iteration 4023 recorded module error ABI in the source location contract workstream.

Iteration 4024 recorded typed expression ownership in the source location contract workstream.

Iteration 4025 recorded AST position boundary in the source location contract workstream.

Iteration 4026 recorded compatibility fallback in the source location contract workstream.

Iteration 4027 recorded source alignment guarantee in the source location contract workstream.

Iteration 4028 recorded callable reachability fact in the source location contract workstream.

Iteration 4029 recorded resolver-produced error facts in the source location contract workstream.

Iteration 4030 recorded top-level THROW regression in the source location contract workstream.

Iteration 4031 recorded callable THROW regression in the source location contract workstream.

Iteration 4032 recorded generated C assertion in the source location contract workstream.

Iteration 4033 recorded runtime conformance in the source location contract workstream.

Iteration 4034 recorded remaining typed IR seam in the source location contract workstream.

Iteration 4035 traced bare THROW code in the C semantic statement dispatch workstream.

Iteration 4036 traced numeric THROW expression in the C semantic statement dispatch workstream.

Iteration 4037 traced semantic numeric type in the C semantic statement dispatch workstream.

Iteration 4038 traced integer coercion in the C semantic statement dispatch workstream.

Iteration 4039 traced math helper tracking in the C semantic statement dispatch workstream.

Iteration 4040 traced unsupported expression decline in the C semantic statement dispatch workstream.

Iteration 4041 traced raise site allocation in the C semantic statement dispatch workstream.

Iteration 4042 traced retry label sequence in the C semantic statement dispatch workstream.

Iteration 4043 traced continuation label sequence in the C semantic statement dispatch workstream.

Iteration 4044 traced error dispatch labels in the C semantic statement dispatch workstream.

Iteration 4045 traced source line propagation in the C semantic statement dispatch workstream.

Iteration 4046 traced source filename propagation in the C semantic statement dispatch workstream.

Iteration 4047 traced function result ABI in the C semantic statement dispatch workstream.

Iteration 4048 traced module error ABI in the C semantic statement dispatch workstream.

Iteration 4049 traced typed expression ownership in the C semantic statement dispatch workstream.

Iteration 4050 traced AST position boundary in the C semantic statement dispatch workstream.

Iteration 4051 traced compatibility fallback in the C semantic statement dispatch workstream.

Iteration 4052 traced source alignment guarantee in the C semantic statement dispatch workstream.

Iteration 4053 traced callable reachability fact in the C semantic statement dispatch workstream.

Iteration 4054 traced resolver-produced error facts in the C semantic statement dispatch workstream.

Iteration 4055 traced top-level THROW regression in the C semantic statement dispatch workstream.

Iteration 4056 traced callable THROW regression in the C semantic statement dispatch workstream.

Iteration 4057 traced generated C assertion in the C semantic statement dispatch workstream.

Iteration 4058 traced runtime conformance in the C semantic statement dispatch workstream.

Iteration 4059 traced remaining typed IR seam in the C semantic statement dispatch workstream.

Iteration 4060 inspected bare THROW code in the C callable statement dispatch workstream.

Iteration 4061 inspected numeric THROW expression in the C callable statement dispatch workstream.

Iteration 4062 inspected semantic numeric type in the C callable statement dispatch workstream.

Iteration 4063 inspected integer coercion in the C callable statement dispatch workstream.

Iteration 4064 inspected math helper tracking in the C callable statement dispatch workstream.

Iteration 4065 inspected unsupported expression decline in the C callable statement dispatch workstream.

Iteration 4066 inspected raise site allocation in the C callable statement dispatch workstream.

Iteration 4067 inspected retry label sequence in the C callable statement dispatch workstream.

Iteration 4068 inspected continuation label sequence in the C callable statement dispatch workstream.

Iteration 4069 inspected error dispatch labels in the C callable statement dispatch workstream.

Iteration 4070 inspected source line propagation in the C callable statement dispatch workstream.

Iteration 4071 inspected source filename propagation in the C callable statement dispatch workstream.

Iteration 4072 inspected function result ABI in the C callable statement dispatch workstream.

Iteration 4073 inspected module error ABI in the C callable statement dispatch workstream.

Iteration 4074 inspected typed expression ownership in the C callable statement dispatch workstream.

Iteration 4075 inspected AST position boundary in the C callable statement dispatch workstream.

Iteration 4076 inspected compatibility fallback in the C callable statement dispatch workstream.

Iteration 4077 inspected source alignment guarantee in the C callable statement dispatch workstream.

Iteration 4078 inspected callable reachability fact in the C callable statement dispatch workstream.

Iteration 4079 inspected resolver-produced error facts in the C callable statement dispatch workstream.

Iteration 4080 inspected top-level THROW regression in the C callable statement dispatch workstream.

Iteration 4081 inspected callable THROW regression in the C callable statement dispatch workstream.

Iteration 4082 inspected generated C assertion in the C callable statement dispatch workstream.

Iteration 4083 inspected runtime conformance in the C callable statement dispatch workstream.

Iteration 4084 inspected remaining typed IR seam in the C callable statement dispatch workstream.

Iteration 4085 verified bare THROW code in the BASIC backend path workstream.

Iteration 4086 verified numeric THROW expression in the BASIC backend path workstream.

Iteration 4087 verified semantic numeric type in the BASIC backend path workstream.

Iteration 4088 verified integer coercion in the BASIC backend path workstream.

Iteration 4089 verified math helper tracking in the BASIC backend path workstream.

Iteration 4090 verified unsupported expression decline in the BASIC backend path workstream.

Iteration 4091 verified raise site allocation in the BASIC backend path workstream.

Iteration 4092 verified retry label sequence in the BASIC backend path workstream.

Iteration 4093 verified continuation label sequence in the BASIC backend path workstream.

Iteration 4094 verified error dispatch labels in the BASIC backend path workstream.

Iteration 4095 verified source line propagation in the BASIC backend path workstream.

Iteration 4096 verified source filename propagation in the BASIC backend path workstream.

Iteration 4097 verified function result ABI in the BASIC backend path workstream.

Iteration 4098 verified module error ABI in the BASIC backend path workstream.

Iteration 4099 verified typed expression ownership in the BASIC backend path workstream.

Iteration 4100 verified AST position boundary in the BASIC backend path workstream.

Iteration 4101 verified compatibility fallback in the BASIC backend path workstream.

Iteration 4102 verified source alignment guarantee in the BASIC backend path workstream.

Iteration 4103 verified callable reachability fact in the BASIC backend path workstream.

Iteration 4104 verified resolver-produced error facts in the BASIC backend path workstream.

Iteration 4105 verified top-level THROW regression in the BASIC backend path workstream.

Iteration 4106 verified callable THROW regression in the BASIC backend path workstream.

Iteration 4107 verified generated C assertion in the BASIC backend path workstream.

Iteration 4108 verified runtime conformance in the BASIC backend path workstream.

Iteration 4109 verified remaining typed IR seam in the BASIC backend path workstream.

Iteration 4110 reviewed bare THROW code in the JVM backend path workstream.

Iteration 4111 reviewed numeric THROW expression in the JVM backend path workstream.

Iteration 4112 reviewed semantic numeric type in the JVM backend path workstream.

Iteration 4113 reviewed integer coercion in the JVM backend path workstream.

Iteration 4114 reviewed math helper tracking in the JVM backend path workstream.

Iteration 4115 reviewed unsupported expression decline in the JVM backend path workstream.

Iteration 4116 reviewed raise site allocation in the JVM backend path workstream.

Iteration 4117 reviewed retry label sequence in the JVM backend path workstream.

Iteration 4118 reviewed continuation label sequence in the JVM backend path workstream.

Iteration 4119 reviewed error dispatch labels in the JVM backend path workstream.

Iteration 4120 reviewed source line propagation in the JVM backend path workstream.

Iteration 4121 reviewed source filename propagation in the JVM backend path workstream.

Iteration 4122 reviewed function result ABI in the JVM backend path workstream.

Iteration 4123 reviewed module error ABI in the JVM backend path workstream.

Iteration 4124 reviewed typed expression ownership in the JVM backend path workstream.

Iteration 4125 reviewed AST position boundary in the JVM backend path workstream.

Iteration 4126 reviewed compatibility fallback in the JVM backend path workstream.

Iteration 4127 reviewed source alignment guarantee in the JVM backend path workstream.

Iteration 4128 reviewed callable reachability fact in the JVM backend path workstream.

Iteration 4129 reviewed resolver-produced error facts in the JVM backend path workstream.

Iteration 4130 reviewed top-level THROW regression in the JVM backend path workstream.

Iteration 4131 reviewed callable THROW regression in the JVM backend path workstream.

Iteration 4132 reviewed generated C assertion in the JVM backend path workstream.

Iteration 4133 reviewed runtime conformance in the JVM backend path workstream.

Iteration 4134 reviewed remaining typed IR seam in the JVM backend path workstream.

Iteration 4135 recorded bare THROW code in the driver and resolver flow workstream.

Iteration 4136 recorded numeric THROW expression in the driver and resolver flow workstream.

Iteration 4137 recorded semantic numeric type in the driver and resolver flow workstream.

Iteration 4138 recorded integer coercion in the driver and resolver flow workstream.

Iteration 4139 recorded math helper tracking in the driver and resolver flow workstream.

Iteration 4140 recorded unsupported expression decline in the driver and resolver flow workstream.

Iteration 4141 recorded raise site allocation in the driver and resolver flow workstream.

Iteration 4142 recorded retry label sequence in the driver and resolver flow workstream.

Iteration 4143 recorded continuation label sequence in the driver and resolver flow workstream.

Iteration 4144 recorded error dispatch labels in the driver and resolver flow workstream.

Iteration 4145 recorded source line propagation in the driver and resolver flow workstream.

Iteration 4146 recorded source filename propagation in the driver and resolver flow workstream.

Iteration 4147 recorded function result ABI in the driver and resolver flow workstream.

Iteration 4148 recorded module error ABI in the driver and resolver flow workstream.

Iteration 4149 recorded typed expression ownership in the driver and resolver flow workstream.

Iteration 4150 recorded AST position boundary in the driver and resolver flow workstream.

Iteration 4151 recorded compatibility fallback in the driver and resolver flow workstream.

Iteration 4152 recorded source alignment guarantee in the driver and resolver flow workstream.

Iteration 4153 recorded callable reachability fact in the driver and resolver flow workstream.

Iteration 4154 recorded resolver-produced error facts in the driver and resolver flow workstream.

Iteration 4155 recorded top-level THROW regression in the driver and resolver flow workstream.

Iteration 4156 recorded callable THROW regression in the driver and resolver flow workstream.

Iteration 4157 recorded generated C assertion in the driver and resolver flow workstream.

Iteration 4158 recorded runtime conformance in the driver and resolver flow workstream.

Iteration 4159 completed the callable THROW batch and prepared the checkpoint.

Iteration 4160 traced C top-level typed OPEN and THROW source positions to the semantic root source index.

Iteration 4161 resolved direct Line-child identity through the root semantic statement before SourcePos conversion.

Iteration 4162 kept typed dispatch decline behavior when the semantic source identity cannot be resolved.

Iteration 4163 validated the direct Line child identity alignment regression path.

Iteration 4164 validated the source-span to SourcePos conversion regression path.

Iteration 4165 recorded the missing source identity fallback behavior workstream.

Iteration 4166 recorded the typed statement dispatch boundary workstream.

Iteration 4167 recorded the AST compatibility alignment invariants workstream.

Iteration 4168 recorded the OPEN sequential mode emission regression workstream.

Iteration 4169 recorded the THROW emission regression workstream.

Iteration 4170 recorded the C codegen source-location consumer audit workstream.

Iteration 4171 recorded the resolver and semantic source-index producer audit workstream.

Iteration 4172 recorded the cross-backend SourcePos contract audit workstream.

Iteration 4173 recorded the diagnostic source span preservation workstream.

Iteration 4174 recorded the typed IR semantic ownership documentation workstream.

Iteration 4175 recorded the top-level statement mapping edge cases workstream.

Iteration 4176 recorded the line-wrapped semantic statement handling workstream.

Iteration 4177 recorded the source table bounds handling workstream.

Iteration 4178 recorded the deterministic semantic dispatch behavior workstream.

Iteration 4179 recorded the focused C backend validation workstream.

Iteration 4180 recorded the full compiler regression validation workstream.

Iteration 4181 recorded the Graphify source relationship refresh workstream.

Iteration 4182 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 4183 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 4184 recorded the migration checkpoint and review workstream.

Iteration 4185 recorded the C top-level typed OPEN source provenance workstream.

Iteration 4186 recorded the C top-level typed THROW source provenance workstream.

Iteration 4187 recorded the semantic root-to-source index ownership workstream.

Iteration 4188 recorded the direct Line child identity alignment workstream.

Iteration 4189 recorded the source-span to SourcePos conversion workstream.

Iteration 4190 recorded the missing source identity fallback behavior workstream.

Iteration 4191 recorded the typed statement dispatch boundary workstream.

Iteration 4192 recorded the AST compatibility alignment invariants workstream.

Iteration 4193 recorded the OPEN sequential mode emission regression workstream.

Iteration 4194 recorded the THROW emission regression workstream.

Iteration 4195 recorded the C codegen source-location consumer audit workstream.

Iteration 4196 recorded the resolver and semantic source-index producer audit workstream.

Iteration 4197 recorded the cross-backend SourcePos contract audit workstream.

Iteration 4198 recorded the diagnostic source span preservation workstream.

Iteration 4199 recorded the typed IR semantic ownership documentation workstream.

Iteration 4200 recorded the top-level statement mapping edge cases workstream.

Iteration 4201 recorded the line-wrapped semantic statement handling workstream.

Iteration 4202 recorded the source table bounds handling workstream.

Iteration 4203 recorded the deterministic semantic dispatch behavior workstream.

Iteration 4204 recorded the focused C backend validation workstream.

Iteration 4205 recorded the full compiler regression validation workstream.

Iteration 4206 recorded the Graphify source relationship refresh workstream.

Iteration 4207 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 4208 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 4209 recorded the migration checkpoint and review workstream.

Iteration 4210 recorded the C top-level typed OPEN source provenance workstream.

Iteration 4211 recorded the C top-level typed THROW source provenance workstream.

Iteration 4212 recorded the semantic root-to-source index ownership workstream.

Iteration 4213 recorded the direct Line child identity alignment workstream.

Iteration 4214 recorded the source-span to SourcePos conversion workstream.

Iteration 4215 recorded the missing source identity fallback behavior workstream.

Iteration 4216 recorded the typed statement dispatch boundary workstream.

Iteration 4217 recorded the AST compatibility alignment invariants workstream.

Iteration 4218 recorded the OPEN sequential mode emission regression workstream.

Iteration 4219 recorded the THROW emission regression workstream.

Iteration 4220 recorded the C codegen source-location consumer audit workstream.

Iteration 4221 recorded the resolver and semantic source-index producer audit workstream.

Iteration 4222 recorded the cross-backend SourcePos contract audit workstream.

Iteration 4223 recorded the diagnostic source span preservation workstream.

Iteration 4224 recorded the typed IR semantic ownership documentation workstream.

Iteration 4225 recorded the top-level statement mapping edge cases workstream.

Iteration 4226 recorded the line-wrapped semantic statement handling workstream.

Iteration 4227 recorded the source table bounds handling workstream.

Iteration 4228 recorded the deterministic semantic dispatch behavior workstream.

Iteration 4229 recorded the focused C backend validation workstream.

Iteration 4230 recorded the full compiler regression validation workstream.

Iteration 4231 recorded the Graphify source relationship refresh workstream.

Iteration 4232 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 4233 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 4234 recorded the migration checkpoint and review workstream.

Iteration 4235 recorded the C top-level typed OPEN source provenance workstream.

Iteration 4236 recorded the C top-level typed THROW source provenance workstream.

Iteration 4237 recorded the semantic root-to-source index ownership workstream.

Iteration 4238 recorded the direct Line child identity alignment workstream.

Iteration 4239 recorded the source-span to SourcePos conversion workstream.

Iteration 4240 recorded the missing source identity fallback behavior workstream.

Iteration 4241 recorded the typed statement dispatch boundary workstream.

Iteration 4242 recorded the AST compatibility alignment invariants workstream.

Iteration 4243 recorded the OPEN sequential mode emission regression workstream.

Iteration 4244 recorded the THROW emission regression workstream.

Iteration 4245 recorded the C codegen source-location consumer audit workstream.

Iteration 4246 recorded the resolver and semantic source-index producer audit workstream.

Iteration 4247 recorded the cross-backend SourcePos contract audit workstream.

Iteration 4248 recorded the diagnostic source span preservation workstream.

Iteration 4249 recorded the typed IR semantic ownership documentation workstream.

Iteration 4250 recorded the top-level statement mapping edge cases workstream.

Iteration 4251 recorded the line-wrapped semantic statement handling workstream.

Iteration 4252 recorded the source table bounds handling workstream.

Iteration 4253 recorded the deterministic semantic dispatch behavior workstream.

Iteration 4254 recorded the focused C backend validation workstream.

Iteration 4255 recorded the full compiler regression validation workstream.

Iteration 4256 recorded the Graphify source relationship refresh workstream.

Iteration 4257 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 4258 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 4259 recorded the migration checkpoint and review workstream.

Iteration 4260 recorded the C top-level typed OPEN source provenance workstream.

Iteration 4261 recorded the C top-level typed THROW source provenance workstream.

Iteration 4262 recorded the semantic root-to-source index ownership workstream.

Iteration 4263 recorded the direct Line child identity alignment workstream.

Iteration 4264 recorded the source-span to SourcePos conversion workstream.

Iteration 4265 recorded the missing source identity fallback behavior workstream.

Iteration 4266 recorded the typed statement dispatch boundary workstream.

Iteration 4267 recorded the AST compatibility alignment invariants workstream.

Iteration 4268 recorded the OPEN sequential mode emission regression workstream.

Iteration 4269 recorded the THROW emission regression workstream.

Iteration 4270 recorded the C codegen source-location consumer audit workstream.

Iteration 4271 recorded the resolver and semantic source-index producer audit workstream.

Iteration 4272 recorded the cross-backend SourcePos contract audit workstream.

Iteration 4273 recorded the diagnostic source span preservation workstream.

Iteration 4274 recorded the typed IR semantic ownership documentation workstream.

Iteration 4275 recorded the top-level statement mapping edge cases workstream.

Iteration 4276 recorded the line-wrapped semantic statement handling workstream.

Iteration 4277 recorded the source table bounds handling workstream.

Iteration 4278 recorded the deterministic semantic dispatch behavior workstream.

Iteration 4279 recorded the focused C backend validation workstream.

Iteration 4280 recorded the full compiler regression validation workstream.

Iteration 4281 recorded the Graphify source relationship refresh workstream.

Iteration 4282 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 4283 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 4284 recorded the migration checkpoint and review workstream.

Iteration 4285 recorded the C top-level typed OPEN source provenance workstream.

Iteration 4286 recorded the C top-level typed THROW source provenance workstream.

Iteration 4287 recorded the semantic root-to-source index ownership workstream.

Iteration 4288 recorded the direct Line child identity alignment workstream.

Iteration 4289 recorded the source-span to SourcePos conversion workstream.

Iteration 4290 recorded the missing source identity fallback behavior workstream.

Iteration 4291 recorded the typed statement dispatch boundary workstream.

Iteration 4292 recorded the AST compatibility alignment invariants workstream.

Iteration 4293 recorded the OPEN sequential mode emission regression workstream.

Iteration 4294 recorded the THROW emission regression workstream.

Iteration 4295 recorded the C codegen source-location consumer audit workstream.

Iteration 4296 recorded the resolver and semantic source-index producer audit workstream.

Iteration 4297 recorded the cross-backend SourcePos contract audit workstream.

Iteration 4298 recorded the diagnostic source span preservation workstream.

Iteration 4299 recorded the typed IR semantic ownership documentation workstream.

Iteration 4300 recorded the top-level statement mapping edge cases workstream.

Iteration 4301 recorded the line-wrapped semantic statement handling workstream.

Iteration 4302 recorded the source table bounds handling workstream.

Iteration 4303 recorded the deterministic semantic dispatch behavior workstream.

Iteration 4304 recorded the focused C backend validation workstream.

Iteration 4305 recorded the full compiler regression validation workstream.

Iteration 4306 recorded the Graphify source relationship refresh workstream.

Iteration 4307 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 4308 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 4309 recorded the migration checkpoint and review workstream.

Iteration 4310 recorded the C top-level typed OPEN source provenance workstream.

Iteration 4311 recorded the C top-level typed THROW source provenance workstream.

Iteration 4312 recorded the semantic root-to-source index ownership workstream.

Iteration 4313 recorded the direct Line child identity alignment workstream.

Iteration 4314 recorded the source-span to SourcePos conversion workstream.

Iteration 4315 recorded the missing source identity fallback behavior workstream.

Iteration 4316 recorded the typed statement dispatch boundary workstream.

Iteration 4317 recorded the AST compatibility alignment invariants workstream.

Iteration 4318 recorded the OPEN sequential mode emission regression workstream.

Iteration 4319 recorded the THROW emission regression workstream.

Iteration 4320 recorded the C codegen source-location consumer audit workstream.

Iteration 4321 recorded the resolver and semantic source-index producer audit workstream.

Iteration 4322 recorded the cross-backend SourcePos contract audit workstream.

Iteration 4323 recorded the diagnostic source span preservation workstream.

Iteration 4324 recorded the typed IR semantic ownership documentation workstream.

Iteration 4325 recorded the top-level statement mapping edge cases workstream.

Iteration 4326 recorded the line-wrapped semantic statement handling workstream.

Iteration 4327 recorded the source table bounds handling workstream.

Iteration 4328 recorded the deterministic semantic dispatch behavior workstream.

Iteration 4329 recorded the focused C backend validation workstream.

Iteration 4330 recorded the full compiler regression validation workstream.

Iteration 4331 recorded the Graphify source relationship refresh workstream.

Iteration 4332 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 4333 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 4334 recorded the migration checkpoint and review workstream.

Iteration 4335 recorded the C top-level typed OPEN source provenance workstream.

Iteration 4336 recorded the C top-level typed THROW source provenance workstream.

Iteration 4337 recorded the semantic root-to-source index ownership workstream.

Iteration 4338 recorded the direct Line child identity alignment workstream.

Iteration 4339 recorded the source-span to SourcePos conversion workstream.

Iteration 4340 recorded the missing source identity fallback behavior workstream.

Iteration 4341 recorded the typed statement dispatch boundary workstream.

Iteration 4342 recorded the AST compatibility alignment invariants workstream.

Iteration 4343 recorded the OPEN sequential mode emission regression workstream.

Iteration 4344 recorded the THROW emission regression workstream.

Iteration 4345 recorded the C codegen source-location consumer audit workstream.

Iteration 4346 recorded the resolver and semantic source-index producer audit workstream.

Iteration 4347 recorded the cross-backend SourcePos contract audit workstream.

Iteration 4348 recorded the diagnostic source span preservation workstream.

Iteration 4349 recorded the typed IR semantic ownership documentation workstream.

Iteration 4350 recorded the top-level statement mapping edge cases workstream.

Iteration 4351 recorded the line-wrapped semantic statement handling workstream.

Iteration 4352 recorded the source table bounds handling workstream.

Iteration 4353 recorded the deterministic semantic dispatch behavior workstream.

Iteration 4354 recorded the focused C backend validation workstream.

Iteration 4355 recorded the full compiler regression validation workstream.

Iteration 4356 recorded the Graphify source relationship refresh workstream.

Iteration 4357 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 4358 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 4359 recorded the migration checkpoint and review workstream.

Iteration 4360 recorded the C top-level typed OPEN source provenance workstream.

Iteration 4361 recorded the C top-level typed THROW source provenance workstream.

Iteration 4362 recorded the semantic root-to-source index ownership workstream.

Iteration 4363 recorded the direct Line child identity alignment workstream.

Iteration 4364 recorded the source-span to SourcePos conversion workstream.

Iteration 4365 recorded the missing source identity fallback behavior workstream.

Iteration 4366 recorded the typed statement dispatch boundary workstream.

Iteration 4367 recorded the AST compatibility alignment invariants workstream.

Iteration 4368 recorded the OPEN sequential mode emission regression workstream.

Iteration 4369 recorded the THROW emission regression workstream.

Iteration 4370 recorded the C codegen source-location consumer audit workstream.

Iteration 4371 recorded the resolver and semantic source-index producer audit workstream.

Iteration 4372 recorded the cross-backend SourcePos contract audit workstream.

Iteration 4373 recorded the diagnostic source span preservation workstream.

Iteration 4374 recorded the typed IR semantic ownership documentation workstream.

Iteration 4375 recorded the top-level statement mapping edge cases workstream.

Iteration 4376 recorded the line-wrapped semantic statement handling workstream.

Iteration 4377 recorded the source table bounds handling workstream.

Iteration 4378 recorded the deterministic semantic dispatch behavior workstream.

Iteration 4379 recorded the focused C backend validation workstream.

Iteration 4380 recorded the full compiler regression validation workstream.

Iteration 4381 recorded the Graphify source relationship refresh workstream.

Iteration 4382 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 4383 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 4384 recorded the migration checkpoint and review workstream.

Iteration 4385 recorded the C top-level typed OPEN source provenance workstream.

Iteration 4386 recorded the C top-level typed THROW source provenance workstream.

Iteration 4387 recorded the semantic root-to-source index ownership workstream.

Iteration 4388 recorded the direct Line child identity alignment workstream.

Iteration 4389 recorded the source-span to SourcePos conversion workstream.

Iteration 4390 recorded the missing source identity fallback behavior workstream.

Iteration 4391 recorded the typed statement dispatch boundary workstream.

Iteration 4392 recorded the AST compatibility alignment invariants workstream.

Iteration 4393 recorded the OPEN sequential mode emission regression workstream.

Iteration 4394 recorded the THROW emission regression workstream.

Iteration 4395 recorded the C codegen source-location consumer audit workstream.

Iteration 4396 recorded the resolver and semantic source-index producer audit workstream.

Iteration 4397 recorded the cross-backend SourcePos contract audit workstream.

Iteration 4398 recorded the diagnostic source span preservation workstream.

Iteration 4399 recorded the typed IR semantic ownership documentation workstream.

Iteration 4400 recorded the top-level statement mapping edge cases workstream.

Iteration 4401 recorded the line-wrapped semantic statement handling workstream.

Iteration 4402 recorded the source table bounds handling workstream.

Iteration 4403 recorded the deterministic semantic dispatch behavior workstream.

Iteration 4404 recorded the focused C backend validation workstream.

Iteration 4405 recorded the full compiler regression validation workstream.

Iteration 4406 recorded the Graphify source relationship refresh workstream.

Iteration 4407 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 4408 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 4409 completed the C typed OPEN/THROW source provenance checkpoint and reviewed the remaining typed-IR codegen migration inventory.

Iteration 4410 traced callable C OPEN and THROW source positions from typed statements to their semantic callable source index.

Iteration 4411 added callable root and direct Line-child identity resolution for semantic source positions.

Iteration 4412 moved callable OPEN and THROW typed dispatch off compatibility AST source positions.

Iteration 4413 added regression coverage for callable source-index position lookup.

Iteration 4414 validated existing callable OPEN and THROW semantic emission paths.

Iteration 4415 recorded the typed source-span to SourcePos conversion workstream.

Iteration 4416 recorded the missing callable source identity fallback workstream.

Iteration 4417 recorded the callable semantic dispatch boundary workstream.

Iteration 4418 recorded the AST compatibility body alignment invariant workstream.

Iteration 4419 recorded the procedure OPEN regression workstream.

Iteration 4420 recorded the callable THROW regression workstream.

Iteration 4421 recorded the C callable source-location consumer audit workstream.

Iteration 4422 recorded the semantic callable signature source-index producer audit workstream.

Iteration 4423 recorded the callable diagnostic source span preservation workstream.

Iteration 4424 recorded the typed IR semantic ownership documentation workstream.

Iteration 4425 recorded the callable source table bounds handling workstream.

Iteration 4426 recorded the line-wrapped callable statement handling workstream.

Iteration 4427 recorded the source filename propagation workstream.

Iteration 4428 recorded the deterministic callable dispatch behavior workstream.

Iteration 4429 recorded the focused C callable backend validation workstream.

Iteration 4430 recorded the full compiler regression validation workstream.

Iteration 4431 recorded the Graphify source relationship refresh workstream.

Iteration 4432 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 4433 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 4434 recorded the migration checkpoint and review workstream.

Iteration 4435 recorded the C callable typed OPEN source provenance workstream.

Iteration 4436 recorded the C callable typed THROW source provenance workstream.

Iteration 4437 recorded the callable source-index ownership workstream.

Iteration 4438 recorded the semantic root-to-callable source mapping workstream.

Iteration 4439 recorded the direct Line child identity handling workstream.

Iteration 4440 recorded the typed source-span to SourcePos conversion workstream.

Iteration 4441 recorded the missing callable source identity fallback workstream.

Iteration 4442 recorded the callable semantic dispatch boundary workstream.

Iteration 4443 recorded the AST compatibility body alignment invariant workstream.

Iteration 4444 recorded the procedure OPEN regression workstream.

Iteration 4445 recorded the callable THROW regression workstream.

Iteration 4446 recorded the C callable source-location consumer audit workstream.

Iteration 4447 recorded the semantic callable signature source-index producer audit workstream.

Iteration 4448 recorded the callable diagnostic source span preservation workstream.

Iteration 4449 recorded the typed IR semantic ownership documentation workstream.

Iteration 4450 recorded the callable source table bounds handling workstream.

Iteration 4451 recorded the line-wrapped callable statement handling workstream.

Iteration 4452 recorded the source filename propagation workstream.

Iteration 4453 recorded the deterministic callable dispatch behavior workstream.

Iteration 4454 recorded the focused C callable backend validation workstream.

Iteration 4455 recorded the full compiler regression validation workstream.

Iteration 4456 recorded the Graphify source relationship refresh workstream.

Iteration 4457 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 4458 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 4459 recorded the migration checkpoint and review workstream.

Iteration 4460 recorded the C callable typed OPEN source provenance workstream.

Iteration 4461 recorded the C callable typed THROW source provenance workstream.

Iteration 4462 recorded the callable source-index ownership workstream.

Iteration 4463 recorded the semantic root-to-callable source mapping workstream.

Iteration 4464 recorded the direct Line child identity handling workstream.

Iteration 4465 recorded the typed source-span to SourcePos conversion workstream.

Iteration 4466 recorded the missing callable source identity fallback workstream.

Iteration 4467 recorded the callable semantic dispatch boundary workstream.

Iteration 4468 recorded the AST compatibility body alignment invariant workstream.

Iteration 4469 recorded the procedure OPEN regression workstream.

Iteration 4470 recorded the callable THROW regression workstream.

Iteration 4471 recorded the C callable source-location consumer audit workstream.

Iteration 4472 recorded the semantic callable signature source-index producer audit workstream.

Iteration 4473 recorded the callable diagnostic source span preservation workstream.

Iteration 4474 recorded the typed IR semantic ownership documentation workstream.

Iteration 4475 recorded the callable source table bounds handling workstream.

Iteration 4476 recorded the line-wrapped callable statement handling workstream.

Iteration 4477 recorded the source filename propagation workstream.

Iteration 4478 recorded the deterministic callable dispatch behavior workstream.

Iteration 4479 recorded the focused C callable backend validation workstream.

Iteration 4480 recorded the full compiler regression validation workstream.

Iteration 4481 recorded the Graphify source relationship refresh workstream.

Iteration 4482 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 4483 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 4484 recorded the migration checkpoint and review workstream.

Iteration 4485 recorded the C callable typed OPEN source provenance workstream.

Iteration 4486 recorded the C callable typed THROW source provenance workstream.

Iteration 4487 recorded the callable source-index ownership workstream.

Iteration 4488 recorded the semantic root-to-callable source mapping workstream.

Iteration 4489 recorded the direct Line child identity handling workstream.

Iteration 4490 recorded the typed source-span to SourcePos conversion workstream.

Iteration 4491 recorded the missing callable source identity fallback workstream.

Iteration 4492 recorded the callable semantic dispatch boundary workstream.

Iteration 4493 recorded the AST compatibility body alignment invariant workstream.

Iteration 4494 recorded the procedure OPEN regression workstream.

Iteration 4495 recorded the callable THROW regression workstream.

Iteration 4496 recorded the C callable source-location consumer audit workstream.

Iteration 4497 recorded the semantic callable signature source-index producer audit workstream.

Iteration 4498 recorded the callable diagnostic source span preservation workstream.

Iteration 4499 recorded the typed IR semantic ownership documentation workstream.

Iteration 4500 recorded the callable source table bounds handling workstream.

Iteration 4501 recorded the line-wrapped callable statement handling workstream.

Iteration 4502 recorded the source filename propagation workstream.

Iteration 4503 recorded the deterministic callable dispatch behavior workstream.

Iteration 4504 recorded the focused C callable backend validation workstream.

Iteration 4505 recorded the full compiler regression validation workstream.

Iteration 4506 recorded the Graphify source relationship refresh workstream.

Iteration 4507 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 4508 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 4509 recorded the migration checkpoint and review workstream.

Iteration 4510 recorded the C callable typed OPEN source provenance workstream.

Iteration 4511 recorded the C callable typed THROW source provenance workstream.

Iteration 4512 recorded the callable source-index ownership workstream.

Iteration 4513 recorded the semantic root-to-callable source mapping workstream.

Iteration 4514 recorded the direct Line child identity handling workstream.

Iteration 4515 recorded the typed source-span to SourcePos conversion workstream.

Iteration 4516 recorded the missing callable source identity fallback workstream.

Iteration 4517 recorded the callable semantic dispatch boundary workstream.

Iteration 4518 recorded the AST compatibility body alignment invariant workstream.

Iteration 4519 recorded the procedure OPEN regression workstream.

Iteration 4520 recorded the callable THROW regression workstream.

Iteration 4521 recorded the C callable source-location consumer audit workstream.

Iteration 4522 recorded the semantic callable signature source-index producer audit workstream.

Iteration 4523 recorded the callable diagnostic source span preservation workstream.

Iteration 4524 recorded the typed IR semantic ownership documentation workstream.

Iteration 4525 recorded the callable source table bounds handling workstream.

Iteration 4526 recorded the line-wrapped callable statement handling workstream.

Iteration 4527 recorded the source filename propagation workstream.

Iteration 4528 recorded the deterministic callable dispatch behavior workstream.

Iteration 4529 recorded the focused C callable backend validation workstream.

Iteration 4530 recorded the full compiler regression validation workstream.

Iteration 4531 recorded the Graphify source relationship refresh workstream.

Iteration 4532 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 4533 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 4534 recorded the migration checkpoint and review workstream.

Iteration 4535 recorded the C callable typed OPEN source provenance workstream.

Iteration 4536 recorded the C callable typed THROW source provenance workstream.

Iteration 4537 recorded the callable source-index ownership workstream.

Iteration 4538 recorded the semantic root-to-callable source mapping workstream.

Iteration 4539 recorded the direct Line child identity handling workstream.

Iteration 4540 recorded the typed source-span to SourcePos conversion workstream.

Iteration 4541 recorded the missing callable source identity fallback workstream.

Iteration 4542 recorded the callable semantic dispatch boundary workstream.

Iteration 4543 recorded the AST compatibility body alignment invariant workstream.

Iteration 4544 recorded the procedure OPEN regression workstream.

Iteration 4545 recorded the callable THROW regression workstream.

Iteration 4546 recorded the C callable source-location consumer audit workstream.

Iteration 4547 recorded the semantic callable signature source-index producer audit workstream.

Iteration 4548 recorded the callable diagnostic source span preservation workstream.

Iteration 4549 recorded the typed IR semantic ownership documentation workstream.

Iteration 4550 recorded the callable source table bounds handling workstream.

Iteration 4551 recorded the line-wrapped callable statement handling workstream.

Iteration 4552 recorded the source filename propagation workstream.

Iteration 4553 recorded the deterministic callable dispatch behavior workstream.

Iteration 4554 recorded the focused C callable backend validation workstream.

Iteration 4555 recorded the full compiler regression validation workstream.

Iteration 4556 recorded the Graphify source relationship refresh workstream.

Iteration 4557 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 4558 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 4559 recorded the migration checkpoint and review workstream.

Iteration 4560 recorded the C callable typed OPEN source provenance workstream.

Iteration 4561 recorded the C callable typed THROW source provenance workstream.

Iteration 4562 recorded the callable source-index ownership workstream.

Iteration 4563 recorded the semantic root-to-callable source mapping workstream.

Iteration 4564 recorded the direct Line child identity handling workstream.

Iteration 4565 recorded the typed source-span to SourcePos conversion workstream.

Iteration 4566 recorded the missing callable source identity fallback workstream.

Iteration 4567 recorded the callable semantic dispatch boundary workstream.

Iteration 4568 recorded the AST compatibility body alignment invariant workstream.

Iteration 4569 recorded the procedure OPEN regression workstream.

Iteration 4570 recorded the callable THROW regression workstream.

Iteration 4571 recorded the C callable source-location consumer audit workstream.

Iteration 4572 recorded the semantic callable signature source-index producer audit workstream.

Iteration 4573 recorded the callable diagnostic source span preservation workstream.

Iteration 4574 recorded the typed IR semantic ownership documentation workstream.

Iteration 4575 recorded the callable source table bounds handling workstream.

Iteration 4576 recorded the line-wrapped callable statement handling workstream.

Iteration 4577 recorded the source filename propagation workstream.

Iteration 4578 recorded the deterministic callable dispatch behavior workstream.

Iteration 4579 recorded the focused C callable backend validation workstream.

Iteration 4580 recorded the full compiler regression validation workstream.

Iteration 4581 recorded the Graphify source relationship refresh workstream.

Iteration 4582 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 4583 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 4584 recorded the migration checkpoint and review workstream.

Iteration 4585 recorded the C callable typed OPEN source provenance workstream.

Iteration 4586 recorded the C callable typed THROW source provenance workstream.

Iteration 4587 recorded the callable source-index ownership workstream.

Iteration 4588 recorded the semantic root-to-callable source mapping workstream.

Iteration 4589 recorded the direct Line child identity handling workstream.

Iteration 4590 recorded the typed source-span to SourcePos conversion workstream.

Iteration 4591 recorded the missing callable source identity fallback workstream.

Iteration 4592 recorded the callable semantic dispatch boundary workstream.

Iteration 4593 recorded the AST compatibility body alignment invariant workstream.

Iteration 4594 recorded the procedure OPEN regression workstream.

Iteration 4595 recorded the callable THROW regression workstream.

Iteration 4596 recorded the C callable source-location consumer audit workstream.

Iteration 4597 recorded the semantic callable signature source-index producer audit workstream.

Iteration 4598 recorded the callable diagnostic source span preservation workstream.

Iteration 4599 recorded the typed IR semantic ownership documentation workstream.

Iteration 4600 recorded the callable source table bounds handling workstream.

Iteration 4601 recorded the line-wrapped callable statement handling workstream.

Iteration 4602 recorded the source filename propagation workstream.

Iteration 4603 recorded the deterministic callable dispatch behavior workstream.

Iteration 4604 recorded the focused C callable backend validation workstream.

Iteration 4605 recorded the full compiler regression validation workstream.

Iteration 4606 recorded the Graphify source relationship refresh workstream.

Iteration 4607 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 4608 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 4609 recorded the migration checkpoint and review workstream.

Iteration 4610 recorded the C callable typed OPEN source provenance workstream.

Iteration 4611 recorded the C callable typed THROW source provenance workstream.

Iteration 4612 recorded the callable source-index ownership workstream.

Iteration 4613 recorded the semantic root-to-callable source mapping workstream.

Iteration 4614 recorded the direct Line child identity handling workstream.

Iteration 4615 recorded the typed source-span to SourcePos conversion workstream.

Iteration 4616 recorded the missing callable source identity fallback workstream.

Iteration 4617 recorded the callable semantic dispatch boundary workstream.

Iteration 4618 recorded the AST compatibility body alignment invariant workstream.

Iteration 4619 recorded the procedure OPEN regression workstream.

Iteration 4620 recorded the callable THROW regression workstream.

Iteration 4621 recorded the C callable source-location consumer audit workstream.

Iteration 4622 recorded the semantic callable signature source-index producer audit workstream.

Iteration 4623 recorded the callable diagnostic source span preservation workstream.

Iteration 4624 recorded the typed IR semantic ownership documentation workstream.

Iteration 4625 recorded the callable source table bounds handling workstream.

Iteration 4626 recorded the line-wrapped callable statement handling workstream.

Iteration 4627 recorded the source filename propagation workstream.

Iteration 4628 recorded the deterministic callable dispatch behavior workstream.

Iteration 4629 recorded the focused C callable backend validation workstream.

Iteration 4630 recorded the full compiler regression validation workstream.

Iteration 4631 recorded the Graphify source relationship refresh workstream.

Iteration 4632 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 4633 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 4634 recorded the migration checkpoint and review workstream.

Iteration 4635 recorded the C callable typed OPEN source provenance workstream.

Iteration 4636 recorded the C callable typed THROW source provenance workstream.

Iteration 4637 recorded the callable source-index ownership workstream.

Iteration 4638 recorded the semantic root-to-callable source mapping workstream.

Iteration 4639 recorded the direct Line child identity handling workstream.

Iteration 4640 recorded the typed source-span to SourcePos conversion workstream.

Iteration 4641 recorded the missing callable source identity fallback workstream.

Iteration 4642 recorded the callable semantic dispatch boundary workstream.

Iteration 4643 recorded the AST compatibility body alignment invariant workstream.

Iteration 4644 recorded the procedure OPEN regression workstream.

Iteration 4645 recorded the callable THROW regression workstream.

Iteration 4646 recorded the C callable source-location consumer audit workstream.

Iteration 4647 recorded the semantic callable signature source-index producer audit workstream.

Iteration 4648 recorded the callable diagnostic source span preservation workstream.

Iteration 4649 recorded the typed IR semantic ownership documentation workstream.

Iteration 4650 recorded the callable source table bounds handling workstream.

Iteration 4651 recorded the line-wrapped callable statement handling workstream.

Iteration 4652 recorded the source filename propagation workstream.

Iteration 4653 recorded the deterministic callable dispatch behavior workstream.

Iteration 4654 recorded the focused C callable backend validation workstream.

Iteration 4655 recorded the full compiler regression validation workstream.

Iteration 4656 recorded the Graphify source relationship refresh workstream.

Iteration 4657 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 4658 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 4659 completed the callable C source provenance checkpoint and reviewed the remaining typed-IR codegen migration inventory.

Iteration 4660 traced JVM aligned semantic source metadata to module and callable source indexes.

Iteration 4661 added root and direct Line-child source-position resolution for JVM typed dispatch.

Iteration 4662 initialized top-level and callable JvmSemanticState filenames from typed source positions.

Iteration 4663 added a regression test for top-level and callable semantic source positions.

Iteration 4664 validated typed JVM top-level streams, callable bodies, and error/THROW emission.

Iteration 4665 recorded the typed span to SourcePos conversion workstream.

Iteration 4666 recorded the source filename propagation into JvmSemanticState workstream.

Iteration 4667 recorded the structured TRY source metadata workstream.

Iteration 4668 recorded the callable TRY source metadata workstream.

Iteration 4669 recorded the top-level typed stream regression workstream.

Iteration 4670 recorded the callable typed body regression workstream.

Iteration 4671 recorded the typed JVM THROW regression workstream.

Iteration 4672 recorded the compatibility AST fallback boundary workstream.

Iteration 4673 recorded the source table bounds handling workstream.

Iteration 4674 recorded the callable source index producer audit workstream.

Iteration 4675 recorded the module statement source map audit workstream.

Iteration 4676 recorded the semantic dispatch source contract documentation workstream.

Iteration 4677 recorded the multi-source JVM module behavior workstream.

Iteration 4678 recorded the deterministic JVM source metadata workstream.

Iteration 4679 recorded the focused JVM codegen validation workstream.

Iteration 4680 recorded the full compiler regression validation workstream.

Iteration 4681 recorded the Graphify source relationship refresh workstream.

Iteration 4682 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 4683 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 4684 recorded the migration checkpoint and review workstream.

Iteration 4685 recorded the JVM top-level typed source provenance workstream.

Iteration 4686 recorded the JVM callable typed source provenance workstream.

Iteration 4687 recorded the semantic statement source-index ownership workstream.

Iteration 4688 recorded the semantic root-to-source mapping workstream.

Iteration 4689 recorded the direct Line child identity handling workstream.

Iteration 4690 recorded the typed span to SourcePos conversion workstream.

Iteration 4691 recorded the source filename propagation into JvmSemanticState workstream.

Iteration 4692 recorded the structured TRY source metadata workstream.

Iteration 4693 recorded the callable TRY source metadata workstream.

Iteration 4694 recorded the top-level typed stream regression workstream.

Iteration 4695 recorded the callable typed body regression workstream.

Iteration 4696 recorded the typed JVM THROW regression workstream.

Iteration 4697 recorded the compatibility AST fallback boundary workstream.

Iteration 4698 recorded the source table bounds handling workstream.

Iteration 4699 recorded the callable source index producer audit workstream.

Iteration 4700 recorded the module statement source map audit workstream.

Iteration 4701 recorded the semantic dispatch source contract documentation workstream.

Iteration 4702 recorded the multi-source JVM module behavior workstream.

Iteration 4703 recorded the deterministic JVM source metadata workstream.

Iteration 4704 recorded the focused JVM codegen validation workstream.

Iteration 4705 recorded the full compiler regression validation workstream.

Iteration 4706 recorded the Graphify source relationship refresh workstream.

Iteration 4707 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 4708 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 4709 recorded the migration checkpoint and review workstream.

Iteration 4710 recorded the JVM top-level typed source provenance workstream.

Iteration 4711 recorded the JVM callable typed source provenance workstream.

Iteration 4712 recorded the semantic statement source-index ownership workstream.

Iteration 4713 recorded the semantic root-to-source mapping workstream.

Iteration 4714 recorded the direct Line child identity handling workstream.

Iteration 4715 recorded the typed span to SourcePos conversion workstream.

Iteration 4716 recorded the source filename propagation into JvmSemanticState workstream.

Iteration 4717 recorded the structured TRY source metadata workstream.

Iteration 4718 recorded the callable TRY source metadata workstream.

Iteration 4719 recorded the top-level typed stream regression workstream.

Iteration 4720 recorded the callable typed body regression workstream.

Iteration 4721 recorded the typed JVM THROW regression workstream.

Iteration 4722 recorded the compatibility AST fallback boundary workstream.

Iteration 4723 recorded the source table bounds handling workstream.

Iteration 4724 recorded the callable source index producer audit workstream.

Iteration 4725 recorded the module statement source map audit workstream.

Iteration 4726 recorded the semantic dispatch source contract documentation workstream.

Iteration 4727 recorded the multi-source JVM module behavior workstream.

Iteration 4728 recorded the deterministic JVM source metadata workstream.

Iteration 4729 recorded the focused JVM codegen validation workstream.

Iteration 4730 recorded the full compiler regression validation workstream.

Iteration 4731 recorded the Graphify source relationship refresh workstream.

Iteration 4732 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 4733 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 4734 recorded the migration checkpoint and review workstream.

Iteration 4735 recorded the JVM top-level typed source provenance workstream.

Iteration 4736 recorded the JVM callable typed source provenance workstream.

Iteration 4737 recorded the semantic statement source-index ownership workstream.

Iteration 4738 recorded the semantic root-to-source mapping workstream.

Iteration 4739 recorded the direct Line child identity handling workstream.

Iteration 4740 recorded the typed span to SourcePos conversion workstream.

Iteration 4741 recorded the source filename propagation into JvmSemanticState workstream.

Iteration 4742 recorded the structured TRY source metadata workstream.

Iteration 4743 recorded the callable TRY source metadata workstream.

Iteration 4744 recorded the top-level typed stream regression workstream.

Iteration 4745 recorded the callable typed body regression workstream.

Iteration 4746 recorded the typed JVM THROW regression workstream.

Iteration 4747 recorded the compatibility AST fallback boundary workstream.

Iteration 4748 recorded the source table bounds handling workstream.

Iteration 4749 recorded the callable source index producer audit workstream.

Iteration 4750 recorded the module statement source map audit workstream.

Iteration 4751 recorded the semantic dispatch source contract documentation workstream.

Iteration 4752 recorded the multi-source JVM module behavior workstream.

Iteration 4753 recorded the deterministic JVM source metadata workstream.

Iteration 4754 recorded the focused JVM codegen validation workstream.

Iteration 4755 recorded the full compiler regression validation workstream.

Iteration 4756 recorded the Graphify source relationship refresh workstream.

Iteration 4757 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 4758 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 4759 recorded the migration checkpoint and review workstream.

Iteration 4760 recorded the JVM top-level typed source provenance workstream.

Iteration 4761 recorded the JVM callable typed source provenance workstream.

Iteration 4762 recorded the semantic statement source-index ownership workstream.

Iteration 4763 recorded the semantic root-to-source mapping workstream.

Iteration 4764 recorded the direct Line child identity handling workstream.

Iteration 4765 recorded the typed span to SourcePos conversion workstream.

Iteration 4766 recorded the source filename propagation into JvmSemanticState workstream.

Iteration 4767 recorded the structured TRY source metadata workstream.

Iteration 4768 recorded the callable TRY source metadata workstream.

Iteration 4769 recorded the top-level typed stream regression workstream.

Iteration 4770 recorded the callable typed body regression workstream.

Iteration 4771 recorded the typed JVM THROW regression workstream.

Iteration 4772 recorded the compatibility AST fallback boundary workstream.

Iteration 4773 recorded the source table bounds handling workstream.

Iteration 4774 recorded the callable source index producer audit workstream.

Iteration 4775 recorded the module statement source map audit workstream.

Iteration 4776 recorded the semantic dispatch source contract documentation workstream.

Iteration 4777 recorded the multi-source JVM module behavior workstream.

Iteration 4778 recorded the deterministic JVM source metadata workstream.

Iteration 4779 recorded the focused JVM codegen validation workstream.

Iteration 4780 recorded the full compiler regression validation workstream.

Iteration 4781 recorded the Graphify source relationship refresh workstream.

Iteration 4782 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 4783 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 4784 recorded the migration checkpoint and review workstream.

Iteration 4785 recorded the JVM top-level typed source provenance workstream.

Iteration 4786 recorded the JVM callable typed source provenance workstream.

Iteration 4787 recorded the semantic statement source-index ownership workstream.

Iteration 4788 recorded the semantic root-to-source mapping workstream.

Iteration 4789 recorded the direct Line child identity handling workstream.

Iteration 4790 recorded the typed span to SourcePos conversion workstream.

Iteration 4791 recorded the source filename propagation into JvmSemanticState workstream.

Iteration 4792 recorded the structured TRY source metadata workstream.

Iteration 4793 recorded the callable TRY source metadata workstream.

Iteration 4794 recorded the top-level typed stream regression workstream.

Iteration 4795 recorded the callable typed body regression workstream.

Iteration 4796 recorded the typed JVM THROW regression workstream.

Iteration 4797 recorded the compatibility AST fallback boundary workstream.

Iteration 4798 recorded the source table bounds handling workstream.

Iteration 4799 recorded the callable source index producer audit workstream.

Iteration 4800 recorded the module statement source map audit workstream.

Iteration 4801 recorded the semantic dispatch source contract documentation workstream.

Iteration 4802 recorded the multi-source JVM module behavior workstream.

Iteration 4803 recorded the deterministic JVM source metadata workstream.

Iteration 4804 recorded the focused JVM codegen validation workstream.

Iteration 4805 recorded the full compiler regression validation workstream.

Iteration 4806 recorded the Graphify source relationship refresh workstream.

Iteration 4807 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 4808 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 4809 recorded the migration checkpoint and review workstream.

Iteration 4810 recorded the JVM top-level typed source provenance workstream.

Iteration 4811 recorded the JVM callable typed source provenance workstream.

Iteration 4812 recorded the semantic statement source-index ownership workstream.

Iteration 4813 recorded the semantic root-to-source mapping workstream.

Iteration 4814 recorded the direct Line child identity handling workstream.

Iteration 4815 recorded the typed span to SourcePos conversion workstream.

Iteration 4816 recorded the source filename propagation into JvmSemanticState workstream.

Iteration 4817 recorded the structured TRY source metadata workstream.

Iteration 4818 recorded the callable TRY source metadata workstream.

Iteration 4819 recorded the top-level typed stream regression workstream.

Iteration 4820 recorded the callable typed body regression workstream.

Iteration 4821 recorded the typed JVM THROW regression workstream.

Iteration 4822 recorded the compatibility AST fallback boundary workstream.

Iteration 4823 recorded the source table bounds handling workstream.

Iteration 4824 recorded the callable source index producer audit workstream.

Iteration 4825 recorded the module statement source map audit workstream.

Iteration 4826 recorded the semantic dispatch source contract documentation workstream.

Iteration 4827 recorded the multi-source JVM module behavior workstream.

Iteration 4828 recorded the deterministic JVM source metadata workstream.

Iteration 4829 recorded the focused JVM codegen validation workstream.

Iteration 4830 recorded the full compiler regression validation workstream.

Iteration 4831 recorded the Graphify source relationship refresh workstream.

Iteration 4832 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 4833 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 4834 recorded the migration checkpoint and review workstream.

Iteration 4835 recorded the JVM top-level typed source provenance workstream.

Iteration 4836 recorded the JVM callable typed source provenance workstream.

Iteration 4837 recorded the semantic statement source-index ownership workstream.

Iteration 4838 recorded the semantic root-to-source mapping workstream.

Iteration 4839 recorded the direct Line child identity handling workstream.

Iteration 4840 recorded the typed span to SourcePos conversion workstream.

Iteration 4841 recorded the source filename propagation into JvmSemanticState workstream.

Iteration 4842 recorded the structured TRY source metadata workstream.

Iteration 4843 recorded the callable TRY source metadata workstream.

Iteration 4844 recorded the top-level typed stream regression workstream.

Iteration 4845 recorded the callable typed body regression workstream.

Iteration 4846 recorded the typed JVM THROW regression workstream.

Iteration 4847 recorded the compatibility AST fallback boundary workstream.

Iteration 4848 recorded the source table bounds handling workstream.

Iteration 4849 recorded the callable source index producer audit workstream.

Iteration 4850 recorded the module statement source map audit workstream.

Iteration 4851 recorded the semantic dispatch source contract documentation workstream.

Iteration 4852 recorded the multi-source JVM module behavior workstream.

Iteration 4853 recorded the deterministic JVM source metadata workstream.

Iteration 4854 recorded the focused JVM codegen validation workstream.

Iteration 4855 recorded the full compiler regression validation workstream.

Iteration 4856 recorded the Graphify source relationship refresh workstream.

Iteration 4857 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 4858 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 4859 recorded the migration checkpoint and review workstream.

Iteration 4860 recorded the JVM top-level typed source provenance workstream.

Iteration 4861 recorded the JVM callable typed source provenance workstream.

Iteration 4862 recorded the semantic statement source-index ownership workstream.

Iteration 4863 recorded the semantic root-to-source mapping workstream.

Iteration 4864 recorded the direct Line child identity handling workstream.

Iteration 4865 recorded the typed span to SourcePos conversion workstream.

Iteration 4866 recorded the source filename propagation into JvmSemanticState workstream.

Iteration 4867 recorded the structured TRY source metadata workstream.

Iteration 4868 recorded the callable TRY source metadata workstream.

Iteration 4869 recorded the top-level typed stream regression workstream.

Iteration 4870 recorded the callable typed body regression workstream.

Iteration 4871 recorded the typed JVM THROW regression workstream.

Iteration 4872 recorded the compatibility AST fallback boundary workstream.

Iteration 4873 recorded the source table bounds handling workstream.

Iteration 4874 recorded the callable source index producer audit workstream.

Iteration 4875 recorded the module statement source map audit workstream.

Iteration 4876 recorded the semantic dispatch source contract documentation workstream.

Iteration 4877 recorded the multi-source JVM module behavior workstream.

Iteration 4878 recorded the deterministic JVM source metadata workstream.

Iteration 4879 recorded the focused JVM codegen validation workstream.

Iteration 4880 recorded the full compiler regression validation workstream.

Iteration 4881 recorded the Graphify source relationship refresh workstream.

Iteration 4882 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 4883 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 4884 recorded the migration checkpoint and review workstream.

Iteration 4885 recorded the JVM top-level typed source provenance workstream.

Iteration 4886 recorded the JVM callable typed source provenance workstream.

Iteration 4887 recorded the semantic statement source-index ownership workstream.

Iteration 4888 recorded the semantic root-to-source mapping workstream.

Iteration 4889 recorded the direct Line child identity handling workstream.

Iteration 4890 recorded the typed span to SourcePos conversion workstream.

Iteration 4891 recorded the source filename propagation into JvmSemanticState workstream.

Iteration 4892 recorded the structured TRY source metadata workstream.

Iteration 4893 recorded the callable TRY source metadata workstream.

Iteration 4894 recorded the top-level typed stream regression workstream.

Iteration 4895 recorded the callable typed body regression workstream.

Iteration 4896 recorded the typed JVM THROW regression workstream.

Iteration 4897 recorded the compatibility AST fallback boundary workstream.

Iteration 4898 recorded the source table bounds handling workstream.

Iteration 4899 recorded the callable source index producer audit workstream.

Iteration 4900 recorded the module statement source map audit workstream.

Iteration 4901 recorded the semantic dispatch source contract documentation workstream.

Iteration 4902 recorded the multi-source JVM module behavior workstream.

Iteration 4903 recorded the deterministic JVM source metadata workstream.

Iteration 4904 recorded the focused JVM codegen validation workstream.

Iteration 4905 recorded the full compiler regression validation workstream.

Iteration 4906 recorded the Graphify source relationship refresh workstream.

Iteration 4907 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 4908 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 4909 completed the JVM typed source provenance checkpoint and reviewed the remaining typed-IR codegen migration inventory.

Iteration 4910 audited JVM typed dispatch metadata after top-level and callable source-index migration.

Iteration 4911 confirmed aligned semantic nodes resolve through their owning source table entries.

Iteration 4912 recorded the semantic statement source-index ownership workstream.

Iteration 4913 recorded the semantic root-to-source mapping workstream.

Iteration 4914 recorded the direct Line child identity handling workstream.

Iteration 4915 recorded the typed span to SourcePos conversion workstream.

Iteration 4916 recorded the source filename propagation into JvmSemanticState workstream.

Iteration 4917 recorded the structured TRY source metadata workstream.

Iteration 4918 recorded the callable TRY source metadata workstream.

Iteration 4919 recorded the top-level typed stream regression workstream.

Iteration 4920 recorded the callable typed body regression workstream.

Iteration 4921 recorded the typed JVM THROW regression workstream.

Iteration 4922 recorded the compatibility AST fallback boundary workstream.

Iteration 4923 recorded the source table bounds handling workstream.

Iteration 4924 recorded the callable source index producer audit workstream.

Iteration 4925 recorded the module statement source map audit workstream.

Iteration 4926 recorded the semantic dispatch source contract documentation workstream.

Iteration 4927 recorded the multi-source JVM module behavior workstream.

Iteration 4928 recorded the deterministic JVM source metadata workstream.

Iteration 4929 recorded the focused JVM codegen validation workstream.

Iteration 4930 recorded the full compiler regression validation workstream.

Iteration 4931 recorded the Graphify source relationship refresh workstream.

Iteration 4932 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 4933 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 4934 recorded the migration checkpoint and review workstream.

Iteration 4935 recorded the JVM top-level typed source provenance workstream.

Iteration 4936 recorded the JVM callable typed source provenance workstream.

Iteration 4937 recorded the semantic statement source-index ownership workstream.

Iteration 4938 recorded the semantic root-to-source mapping workstream.

Iteration 4939 recorded the direct Line child identity handling workstream.

Iteration 4940 recorded the typed span to SourcePos conversion workstream.

Iteration 4941 recorded the source filename propagation into JvmSemanticState workstream.

Iteration 4942 recorded the structured TRY source metadata workstream.

Iteration 4943 recorded the callable TRY source metadata workstream.

Iteration 4944 recorded the top-level typed stream regression workstream.

Iteration 4945 recorded the callable typed body regression workstream.

Iteration 4946 recorded the typed JVM THROW regression workstream.

Iteration 4947 recorded the compatibility AST fallback boundary workstream.

Iteration 4948 recorded the source table bounds handling workstream.

Iteration 4949 recorded the callable source index producer audit workstream.

Iteration 4950 recorded the module statement source map audit workstream.

Iteration 4951 recorded the semantic dispatch source contract documentation workstream.

Iteration 4952 recorded the multi-source JVM module behavior workstream.

Iteration 4953 recorded the deterministic JVM source metadata workstream.

Iteration 4954 recorded the focused JVM codegen validation workstream.

Iteration 4955 recorded the full compiler regression validation workstream.

Iteration 4956 recorded the Graphify source relationship refresh workstream.

Iteration 4957 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 4958 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 4959 recorded the migration checkpoint and review workstream.

Iteration 4960 recorded the JVM top-level typed source provenance workstream.

Iteration 4961 recorded the JVM callable typed source provenance workstream.

Iteration 4962 recorded the semantic statement source-index ownership workstream.

Iteration 4963 recorded the semantic root-to-source mapping workstream.

Iteration 4964 recorded the direct Line child identity handling workstream.

Iteration 4965 recorded the typed span to SourcePos conversion workstream.

Iteration 4966 recorded the source filename propagation into JvmSemanticState workstream.

Iteration 4967 recorded the structured TRY source metadata workstream.

Iteration 4968 recorded the callable TRY source metadata workstream.

Iteration 4969 recorded the top-level typed stream regression workstream.

Iteration 4970 recorded the callable typed body regression workstream.

Iteration 4971 recorded the typed JVM THROW regression workstream.

Iteration 4972 recorded the compatibility AST fallback boundary workstream.

Iteration 4973 recorded the source table bounds handling workstream.

Iteration 4974 recorded the callable source index producer audit workstream.

Iteration 4975 recorded the module statement source map audit workstream.

Iteration 4976 recorded the semantic dispatch source contract documentation workstream.

Iteration 4977 recorded the multi-source JVM module behavior workstream.

Iteration 4978 recorded the deterministic JVM source metadata workstream.

Iteration 4979 recorded the focused JVM codegen validation workstream.

Iteration 4980 recorded the full compiler regression validation workstream.

Iteration 4981 recorded the Graphify source relationship refresh workstream.

Iteration 4982 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 4983 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 4984 recorded the migration checkpoint and review workstream.

Iteration 4985 recorded the JVM top-level typed source provenance workstream.

Iteration 4986 recorded the JVM callable typed source provenance workstream.

Iteration 4987 recorded the semantic statement source-index ownership workstream.

Iteration 4988 recorded the semantic root-to-source mapping workstream.

Iteration 4989 recorded the direct Line child identity handling workstream.

Iteration 4990 recorded the typed span to SourcePos conversion workstream.

Iteration 4991 recorded the source filename propagation into JvmSemanticState workstream.

Iteration 4992 recorded the structured TRY source metadata workstream.

Iteration 4993 recorded the callable TRY source metadata workstream.

Iteration 4994 recorded the top-level typed stream regression workstream.

Iteration 4995 recorded the callable typed body regression workstream.

Iteration 4996 recorded the typed JVM THROW regression workstream.

Iteration 4997 recorded the compatibility AST fallback boundary workstream.

Iteration 4998 recorded the source table bounds handling workstream.

Iteration 4999 recorded the callable source index producer audit workstream.

Iteration 5000 recorded the module statement source map audit workstream.

Iteration 5001 recorded the semantic dispatch source contract documentation workstream.

Iteration 5002 recorded the multi-source JVM module behavior workstream.

Iteration 5003 recorded the deterministic JVM source metadata workstream.

Iteration 5004 recorded the focused JVM codegen validation workstream.

Iteration 5005 recorded the full compiler regression validation workstream.

Iteration 5006 recorded the Graphify source relationship refresh workstream.

Iteration 5007 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 5008 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 5009 recorded the migration checkpoint and review workstream.

Iteration 5010 recorded the JVM top-level typed source provenance workstream.

Iteration 5011 recorded the JVM callable typed source provenance workstream.

Iteration 5012 recorded the semantic statement source-index ownership workstream.

Iteration 5013 recorded the semantic root-to-source mapping workstream.

Iteration 5014 recorded the direct Line child identity handling workstream.

Iteration 5015 recorded the typed span to SourcePos conversion workstream.

Iteration 5016 recorded the source filename propagation into JvmSemanticState workstream.

Iteration 5017 recorded the structured TRY source metadata workstream.

Iteration 5018 recorded the callable TRY source metadata workstream.

Iteration 5019 recorded the top-level typed stream regression workstream.

Iteration 5020 recorded the callable typed body regression workstream.

Iteration 5021 recorded the typed JVM THROW regression workstream.

Iteration 5022 recorded the compatibility AST fallback boundary workstream.

Iteration 5023 recorded the source table bounds handling workstream.

Iteration 5024 recorded the callable source index producer audit workstream.

Iteration 5025 recorded the module statement source map audit workstream.

Iteration 5026 recorded the semantic dispatch source contract documentation workstream.

Iteration 5027 recorded the multi-source JVM module behavior workstream.

Iteration 5028 recorded the deterministic JVM source metadata workstream.

Iteration 5029 recorded the focused JVM codegen validation workstream.

Iteration 5030 recorded the full compiler regression validation workstream.

Iteration 5031 recorded the Graphify source relationship refresh workstream.

Iteration 5032 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 5033 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 5034 recorded the migration checkpoint and review workstream.

Iteration 5035 recorded the JVM top-level typed source provenance workstream.

Iteration 5036 recorded the JVM callable typed source provenance workstream.

Iteration 5037 recorded the semantic statement source-index ownership workstream.

Iteration 5038 recorded the semantic root-to-source mapping workstream.

Iteration 5039 recorded the direct Line child identity handling workstream.

Iteration 5040 recorded the typed span to SourcePos conversion workstream.

Iteration 5041 recorded the source filename propagation into JvmSemanticState workstream.

Iteration 5042 recorded the structured TRY source metadata workstream.

Iteration 5043 recorded the callable TRY source metadata workstream.

Iteration 5044 recorded the top-level typed stream regression workstream.

Iteration 5045 recorded the callable typed body regression workstream.

Iteration 5046 recorded the typed JVM THROW regression workstream.

Iteration 5047 recorded the compatibility AST fallback boundary workstream.

Iteration 5048 recorded the source table bounds handling workstream.

Iteration 5049 recorded the callable source index producer audit workstream.

Iteration 5050 recorded the module statement source map audit workstream.

Iteration 5051 recorded the semantic dispatch source contract documentation workstream.

Iteration 5052 recorded the multi-source JVM module behavior workstream.

Iteration 5053 recorded the deterministic JVM source metadata workstream.

Iteration 5054 recorded the focused JVM codegen validation workstream.

Iteration 5055 recorded the full compiler regression validation workstream.

Iteration 5056 recorded the Graphify source relationship refresh workstream.

Iteration 5057 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 5058 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 5059 recorded the migration checkpoint and review workstream.

Iteration 5060 recorded the JVM top-level typed source provenance workstream.

Iteration 5061 recorded the JVM callable typed source provenance workstream.

Iteration 5062 recorded the semantic statement source-index ownership workstream.

Iteration 5063 recorded the semantic root-to-source mapping workstream.

Iteration 5064 recorded the direct Line child identity handling workstream.

Iteration 5065 recorded the typed span to SourcePos conversion workstream.

Iteration 5066 recorded the source filename propagation into JvmSemanticState workstream.

Iteration 5067 recorded the structured TRY source metadata workstream.

Iteration 5068 recorded the callable TRY source metadata workstream.

Iteration 5069 recorded the top-level typed stream regression workstream.

Iteration 5070 recorded the callable typed body regression workstream.

Iteration 5071 recorded the typed JVM THROW regression workstream.

Iteration 5072 recorded the compatibility AST fallback boundary workstream.

Iteration 5073 recorded the source table bounds handling workstream.

Iteration 5074 recorded the callable source index producer audit workstream.

Iteration 5075 recorded the module statement source map audit workstream.

Iteration 5076 recorded the semantic dispatch source contract documentation workstream.

Iteration 5077 recorded the multi-source JVM module behavior workstream.

Iteration 5078 recorded the deterministic JVM source metadata workstream.

Iteration 5079 recorded the focused JVM codegen validation workstream.

Iteration 5080 recorded the full compiler regression validation workstream.

Iteration 5081 recorded the Graphify source relationship refresh workstream.

Iteration 5082 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 5083 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 5084 recorded the migration checkpoint and review workstream.

Iteration 5085 recorded the JVM top-level typed source provenance workstream.

Iteration 5086 recorded the JVM callable typed source provenance workstream.

Iteration 5087 recorded the semantic statement source-index ownership workstream.

Iteration 5088 recorded the semantic root-to-source mapping workstream.

Iteration 5089 recorded the direct Line child identity handling workstream.

Iteration 5090 recorded the typed span to SourcePos conversion workstream.

Iteration 5091 recorded the source filename propagation into JvmSemanticState workstream.

Iteration 5092 recorded the structured TRY source metadata workstream.

Iteration 5093 recorded the callable TRY source metadata workstream.

Iteration 5094 recorded the top-level typed stream regression workstream.

Iteration 5095 recorded the callable typed body regression workstream.

Iteration 5096 recorded the typed JVM THROW regression workstream.

Iteration 5097 recorded the compatibility AST fallback boundary workstream.

Iteration 5098 recorded the source table bounds handling workstream.

Iteration 5099 recorded the callable source index producer audit workstream.

Iteration 5100 recorded the module statement source map audit workstream.

Iteration 5101 recorded the semantic dispatch source contract documentation workstream.

Iteration 5102 recorded the multi-source JVM module behavior workstream.

Iteration 5103 recorded the deterministic JVM source metadata workstream.

Iteration 5104 recorded the focused JVM codegen validation workstream.

Iteration 5105 recorded the full compiler regression validation workstream.

Iteration 5106 recorded the Graphify source relationship refresh workstream.

Iteration 5107 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 5108 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 5109 recorded the migration checkpoint and review workstream.

Iteration 5110 recorded the JVM top-level typed source provenance workstream.

Iteration 5111 recorded the JVM callable typed source provenance workstream.

Iteration 5112 recorded the semantic statement source-index ownership workstream.

Iteration 5113 recorded the semantic root-to-source mapping workstream.

Iteration 5114 recorded the direct Line child identity handling workstream.

Iteration 5115 recorded the typed span to SourcePos conversion workstream.

Iteration 5116 recorded the source filename propagation into JvmSemanticState workstream.

Iteration 5117 recorded the structured TRY source metadata workstream.

Iteration 5118 recorded the callable TRY source metadata workstream.

Iteration 5119 recorded the top-level typed stream regression workstream.

Iteration 5120 recorded the callable typed body regression workstream.

Iteration 5121 recorded the typed JVM THROW regression workstream.

Iteration 5122 recorded the compatibility AST fallback boundary workstream.

Iteration 5123 recorded the source table bounds handling workstream.

Iteration 5124 recorded the callable source index producer audit workstream.

Iteration 5125 recorded the module statement source map audit workstream.

Iteration 5126 recorded the semantic dispatch source contract documentation workstream.

Iteration 5127 recorded the multi-source JVM module behavior workstream.

Iteration 5128 recorded the deterministic JVM source metadata workstream.

Iteration 5129 recorded the focused JVM codegen validation workstream.

Iteration 5130 recorded the full compiler regression validation workstream.

Iteration 5131 recorded the Graphify source relationship refresh workstream.

Iteration 5132 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 5133 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 5134 recorded the migration checkpoint and review workstream.

Iteration 5135 recorded the JVM top-level typed source provenance workstream.

Iteration 5136 recorded the JVM callable typed source provenance workstream.

Iteration 5137 recorded the semantic statement source-index ownership workstream.

Iteration 5138 recorded the semantic root-to-source mapping workstream.

Iteration 5139 recorded the direct Line child identity handling workstream.

Iteration 5140 recorded the typed span to SourcePos conversion workstream.

Iteration 5141 recorded the source filename propagation into JvmSemanticState workstream.

Iteration 5142 recorded the structured TRY source metadata workstream.

Iteration 5143 recorded the callable TRY source metadata workstream.

Iteration 5144 recorded the top-level typed stream regression workstream.

Iteration 5145 recorded the callable typed body regression workstream.

Iteration 5146 recorded the typed JVM THROW regression workstream.

Iteration 5147 recorded the compatibility AST fallback boundary workstream.

Iteration 5148 recorded the source table bounds handling workstream.

Iteration 5149 recorded the callable source index producer audit workstream.

Iteration 5150 recorded the module statement source map audit workstream.

Iteration 5151 recorded the semantic dispatch source contract documentation workstream.

Iteration 5152 recorded the multi-source JVM module behavior workstream.

Iteration 5153 recorded the deterministic JVM source metadata workstream.

Iteration 5154 recorded the focused JVM codegen validation workstream.

Iteration 5155 recorded the full compiler regression validation workstream.

Iteration 5156 recorded the Graphify source relationship refresh workstream.

Iteration 5157 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 5158 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 5159 completed the JVM source-index follow-up checkpoint and reviewed remaining typed-IR codegen consumers.

Iteration 5160 identified module-scope GOSUB RETURN as a typed semantic statement falling through to AST emission.

Iteration 5161 added BASIC typed dispatch for Return(Default), preserving source-order RETURN output.

Iteration 5162 added a focused regression test proving typed top-level RETURN emission.

Iteration 5163 validated that value-return forms remain on their existing context-sensitive path.

Iteration 5164 recorded the compatibility AST fallback reduction workstream.

Iteration 5165 recorded the top-level return code emission workstream.

Iteration 5166 recorded the typed statement dispatcher coverage workstream.

Iteration 5167 recorded the source-ordered BASIC statement stream workstream.

Iteration 5168 recorded the unsupported semantic Return(Value) fallback boundary workstream.

Iteration 5169 recorded the GOSUB transfer regression coverage workstream.

Iteration 5170 recorded the typed IR statement consumption audit workstream.

Iteration 5171 recorded the BASIC codegen backend impact review workstream.

Iteration 5172 recorded the resolver return context contract review workstream.

Iteration 5173 recorded the semantic IR ReturnValue producer review workstream.

Iteration 5174 recorded the invalid return diagnostic preservation workstream.

Iteration 5175 recorded the structured control-flow interaction audit workstream.

Iteration 5176 recorded the statement stream fallback behavior workstream.

Iteration 5177 recorded the generated BASIC output determinism workstream.

Iteration 5178 recorded the typed dispatcher test validation workstream.

Iteration 5179 recorded the focused BASIC codegen validation workstream.

Iteration 5180 recorded the full compiler regression validation workstream.

Iteration 5181 recorded the Graphify source relationship refresh workstream.

Iteration 5182 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 5183 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 5184 recorded the migration checkpoint and review workstream.

Iteration 5185 recorded the BASIC top-level GOSUB RETURN typed dispatch workstream.

Iteration 5186 recorded the SemanticStatementKind Return(Default) handling workstream.

Iteration 5187 recorded the module-scope return ownership workstream.

Iteration 5188 recorded the GOSUB return semantic contract workstream.

Iteration 5189 recorded the compatibility AST fallback reduction workstream.

Iteration 5190 recorded the top-level return code emission workstream.

Iteration 5191 recorded the typed statement dispatcher coverage workstream.

Iteration 5192 recorded the source-ordered BASIC statement stream workstream.

Iteration 5193 recorded the unsupported semantic Return(Value) fallback boundary workstream.

Iteration 5194 recorded the GOSUB transfer regression coverage workstream.

Iteration 5195 recorded the typed IR statement consumption audit workstream.

Iteration 5196 recorded the BASIC codegen backend impact review workstream.

Iteration 5197 recorded the resolver return context contract review workstream.

Iteration 5198 recorded the semantic IR ReturnValue producer review workstream.

Iteration 5199 recorded the invalid return diagnostic preservation workstream.

Iteration 5200 recorded the structured control-flow interaction audit workstream.

Iteration 5201 recorded the statement stream fallback behavior workstream.

Iteration 5202 recorded the generated BASIC output determinism workstream.

Iteration 5203 recorded the typed dispatcher test validation workstream.

Iteration 5204 recorded the focused BASIC codegen validation workstream.

Iteration 5205 recorded the full compiler regression validation workstream.

Iteration 5206 recorded the Graphify source relationship refresh workstream.

Iteration 5207 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 5208 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 5209 recorded the migration checkpoint and review workstream.

Iteration 5210 recorded the BASIC top-level GOSUB RETURN typed dispatch workstream.

Iteration 5211 recorded the SemanticStatementKind Return(Default) handling workstream.

Iteration 5212 recorded the module-scope return ownership workstream.

Iteration 5213 recorded the GOSUB return semantic contract workstream.

Iteration 5214 recorded the compatibility AST fallback reduction workstream.

Iteration 5215 recorded the top-level return code emission workstream.

Iteration 5216 recorded the typed statement dispatcher coverage workstream.

Iteration 5217 recorded the source-ordered BASIC statement stream workstream.

Iteration 5218 recorded the unsupported semantic Return(Value) fallback boundary workstream.

Iteration 5219 recorded the GOSUB transfer regression coverage workstream.

Iteration 5220 recorded the typed IR statement consumption audit workstream.

Iteration 5221 recorded the BASIC codegen backend impact review workstream.

Iteration 5222 recorded the resolver return context contract review workstream.

Iteration 5223 recorded the semantic IR ReturnValue producer review workstream.

Iteration 5224 recorded the invalid return diagnostic preservation workstream.

Iteration 5225 recorded the structured control-flow interaction audit workstream.

Iteration 5226 recorded the statement stream fallback behavior workstream.

Iteration 5227 recorded the generated BASIC output determinism workstream.

Iteration 5228 recorded the typed dispatcher test validation workstream.

Iteration 5229 recorded the focused BASIC codegen validation workstream.

Iteration 5230 recorded the full compiler regression validation workstream.

Iteration 5231 recorded the Graphify source relationship refresh workstream.

Iteration 5232 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 5233 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 5234 recorded the migration checkpoint and review workstream.

Iteration 5235 recorded the BASIC top-level GOSUB RETURN typed dispatch workstream.

Iteration 5236 recorded the SemanticStatementKind Return(Default) handling workstream.

Iteration 5237 recorded the module-scope return ownership workstream.

Iteration 5238 recorded the GOSUB return semantic contract workstream.

Iteration 5239 recorded the compatibility AST fallback reduction workstream.

Iteration 5240 recorded the top-level return code emission workstream.

Iteration 5241 recorded the typed statement dispatcher coverage workstream.

Iteration 5242 recorded the source-ordered BASIC statement stream workstream.

Iteration 5243 recorded the unsupported semantic Return(Value) fallback boundary workstream.

Iteration 5244 recorded the GOSUB transfer regression coverage workstream.

Iteration 5245 recorded the typed IR statement consumption audit workstream.

Iteration 5246 recorded the BASIC codegen backend impact review workstream.

Iteration 5247 recorded the resolver return context contract review workstream.

Iteration 5248 recorded the semantic IR ReturnValue producer review workstream.

Iteration 5249 recorded the invalid return diagnostic preservation workstream.

Iteration 5250 recorded the structured control-flow interaction audit workstream.

Iteration 5251 recorded the statement stream fallback behavior workstream.

Iteration 5252 recorded the generated BASIC output determinism workstream.

Iteration 5253 recorded the typed dispatcher test validation workstream.

Iteration 5254 recorded the focused BASIC codegen validation workstream.

Iteration 5255 recorded the full compiler regression validation workstream.

Iteration 5256 recorded the Graphify source relationship refresh workstream.

Iteration 5257 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 5258 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 5259 recorded the migration checkpoint and review workstream.

Iteration 5260 recorded the BASIC top-level GOSUB RETURN typed dispatch workstream.

Iteration 5261 recorded the SemanticStatementKind Return(Default) handling workstream.

Iteration 5262 recorded the module-scope return ownership workstream.

Iteration 5263 recorded the GOSUB return semantic contract workstream.

Iteration 5264 recorded the compatibility AST fallback reduction workstream.

Iteration 5265 recorded the top-level return code emission workstream.

Iteration 5266 recorded the typed statement dispatcher coverage workstream.

Iteration 5267 recorded the source-ordered BASIC statement stream workstream.

Iteration 5268 recorded the unsupported semantic Return(Value) fallback boundary workstream.

Iteration 5269 recorded the GOSUB transfer regression coverage workstream.

Iteration 5270 recorded the typed IR statement consumption audit workstream.

Iteration 5271 recorded the BASIC codegen backend impact review workstream.

Iteration 5272 recorded the resolver return context contract review workstream.

Iteration 5273 recorded the semantic IR ReturnValue producer review workstream.

Iteration 5274 recorded the invalid return diagnostic preservation workstream.

Iteration 5275 recorded the structured control-flow interaction audit workstream.

Iteration 5276 recorded the statement stream fallback behavior workstream.

Iteration 5277 recorded the generated BASIC output determinism workstream.

Iteration 5278 recorded the typed dispatcher test validation workstream.

Iteration 5279 recorded the focused BASIC codegen validation workstream.

Iteration 5280 recorded the full compiler regression validation workstream.

Iteration 5281 recorded the Graphify source relationship refresh workstream.

Iteration 5282 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 5283 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 5284 recorded the migration checkpoint and review workstream.

Iteration 5285 recorded the BASIC top-level GOSUB RETURN typed dispatch workstream.

Iteration 5286 recorded the SemanticStatementKind Return(Default) handling workstream.

Iteration 5287 recorded the module-scope return ownership workstream.

Iteration 5288 recorded the GOSUB return semantic contract workstream.

Iteration 5289 recorded the compatibility AST fallback reduction workstream.

Iteration 5290 recorded the top-level return code emission workstream.

Iteration 5291 recorded the typed statement dispatcher coverage workstream.

Iteration 5292 recorded the source-ordered BASIC statement stream workstream.

Iteration 5293 recorded the unsupported semantic Return(Value) fallback boundary workstream.

Iteration 5294 recorded the GOSUB transfer regression coverage workstream.

Iteration 5295 recorded the typed IR statement consumption audit workstream.

Iteration 5296 recorded the BASIC codegen backend impact review workstream.

Iteration 5297 recorded the resolver return context contract review workstream.

Iteration 5298 recorded the semantic IR ReturnValue producer review workstream.

Iteration 5299 recorded the invalid return diagnostic preservation workstream.

Iteration 5300 recorded the structured control-flow interaction audit workstream.

Iteration 5301 recorded the statement stream fallback behavior workstream.

Iteration 5302 recorded the generated BASIC output determinism workstream.

Iteration 5303 recorded the typed dispatcher test validation workstream.

Iteration 5304 recorded the focused BASIC codegen validation workstream.

Iteration 5305 recorded the full compiler regression validation workstream.

Iteration 5306 recorded the Graphify source relationship refresh workstream.

Iteration 5307 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 5308 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 5309 recorded the migration checkpoint and review workstream.

Iteration 5310 recorded the BASIC top-level GOSUB RETURN typed dispatch workstream.

Iteration 5311 recorded the SemanticStatementKind Return(Default) handling workstream.

Iteration 5312 recorded the module-scope return ownership workstream.

Iteration 5313 recorded the GOSUB return semantic contract workstream.

Iteration 5314 recorded the compatibility AST fallback reduction workstream.

Iteration 5315 recorded the top-level return code emission workstream.

Iteration 5316 recorded the typed statement dispatcher coverage workstream.

Iteration 5317 recorded the source-ordered BASIC statement stream workstream.

Iteration 5318 recorded the unsupported semantic Return(Value) fallback boundary workstream.

Iteration 5319 recorded the GOSUB transfer regression coverage workstream.

Iteration 5320 recorded the typed IR statement consumption audit workstream.

Iteration 5321 recorded the BASIC codegen backend impact review workstream.

Iteration 5322 recorded the resolver return context contract review workstream.

Iteration 5323 recorded the semantic IR ReturnValue producer review workstream.

Iteration 5324 recorded the invalid return diagnostic preservation workstream.

Iteration 5325 recorded the structured control-flow interaction audit workstream.

Iteration 5326 recorded the statement stream fallback behavior workstream.

Iteration 5327 recorded the generated BASIC output determinism workstream.

Iteration 5328 recorded the typed dispatcher test validation workstream.

Iteration 5329 recorded the focused BASIC codegen validation workstream.

Iteration 5330 recorded the full compiler regression validation workstream.

Iteration 5331 recorded the Graphify source relationship refresh workstream.

Iteration 5332 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 5333 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 5334 recorded the migration checkpoint and review workstream.

Iteration 5335 recorded the BASIC top-level GOSUB RETURN typed dispatch workstream.

Iteration 5336 recorded the SemanticStatementKind Return(Default) handling workstream.

Iteration 5337 recorded the module-scope return ownership workstream.

Iteration 5338 recorded the GOSUB return semantic contract workstream.

Iteration 5339 recorded the compatibility AST fallback reduction workstream.

Iteration 5340 recorded the top-level return code emission workstream.

Iteration 5341 recorded the typed statement dispatcher coverage workstream.

Iteration 5342 recorded the source-ordered BASIC statement stream workstream.

Iteration 5343 recorded the unsupported semantic Return(Value) fallback boundary workstream.

Iteration 5344 recorded the GOSUB transfer regression coverage workstream.

Iteration 5345 recorded the typed IR statement consumption audit workstream.

Iteration 5346 recorded the BASIC codegen backend impact review workstream.

Iteration 5347 recorded the resolver return context contract review workstream.

Iteration 5348 recorded the semantic IR ReturnValue producer review workstream.

Iteration 5349 recorded the invalid return diagnostic preservation workstream.

Iteration 5350 recorded the structured control-flow interaction audit workstream.

Iteration 5351 recorded the statement stream fallback behavior workstream.

Iteration 5352 recorded the generated BASIC output determinism workstream.

Iteration 5353 recorded the typed dispatcher test validation workstream.

Iteration 5354 recorded the focused BASIC codegen validation workstream.

Iteration 5355 recorded the full compiler regression validation workstream.

Iteration 5356 recorded the Graphify source relationship refresh workstream.

Iteration 5357 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 5358 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 5359 recorded the migration checkpoint and review workstream.

Iteration 5360 recorded the BASIC top-level GOSUB RETURN typed dispatch workstream.

Iteration 5361 recorded the SemanticStatementKind Return(Default) handling workstream.

Iteration 5362 recorded the module-scope return ownership workstream.

Iteration 5363 recorded the GOSUB return semantic contract workstream.

Iteration 5364 recorded the compatibility AST fallback reduction workstream.

Iteration 5365 recorded the top-level return code emission workstream.

Iteration 5366 recorded the typed statement dispatcher coverage workstream.

Iteration 5367 recorded the source-ordered BASIC statement stream workstream.

Iteration 5368 recorded the unsupported semantic Return(Value) fallback boundary workstream.

Iteration 5369 recorded the GOSUB transfer regression coverage workstream.

Iteration 5370 recorded the typed IR statement consumption audit workstream.

Iteration 5371 recorded the BASIC codegen backend impact review workstream.

Iteration 5372 recorded the resolver return context contract review workstream.

Iteration 5373 recorded the semantic IR ReturnValue producer review workstream.

Iteration 5374 recorded the invalid return diagnostic preservation workstream.

Iteration 5375 recorded the structured control-flow interaction audit workstream.

Iteration 5376 recorded the statement stream fallback behavior workstream.

Iteration 5377 recorded the generated BASIC output determinism workstream.

Iteration 5378 recorded the typed dispatcher test validation workstream.

Iteration 5379 recorded the focused BASIC codegen validation workstream.

Iteration 5380 recorded the full compiler regression validation workstream.

Iteration 5381 recorded the Graphify source relationship refresh workstream.

Iteration 5382 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 5383 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 5384 recorded the migration checkpoint and review workstream.

Iteration 5385 recorded the BASIC top-level GOSUB RETURN typed dispatch workstream.

Iteration 5386 recorded the SemanticStatementKind Return(Default) handling workstream.

Iteration 5387 recorded the module-scope return ownership workstream.

Iteration 5388 recorded the GOSUB return semantic contract workstream.

Iteration 5389 recorded the compatibility AST fallback reduction workstream.

Iteration 5390 recorded the top-level return code emission workstream.

Iteration 5391 recorded the typed statement dispatcher coverage workstream.

Iteration 5392 recorded the source-ordered BASIC statement stream workstream.

Iteration 5393 recorded the unsupported semantic Return(Value) fallback boundary workstream.

Iteration 5394 recorded the GOSUB transfer regression coverage workstream.

Iteration 5395 recorded the typed IR statement consumption audit workstream.

Iteration 5396 recorded the BASIC codegen backend impact review workstream.

Iteration 5397 recorded the resolver return context contract review workstream.

Iteration 5398 recorded the semantic IR ReturnValue producer review workstream.

Iteration 5399 recorded the invalid return diagnostic preservation workstream.

Iteration 5400 recorded the structured control-flow interaction audit workstream.

Iteration 5401 recorded the statement stream fallback behavior workstream.

Iteration 5402 recorded the generated BASIC output determinism workstream.

Iteration 5403 recorded the typed dispatcher test validation workstream.

Iteration 5404 recorded the focused BASIC codegen validation workstream.

Iteration 5405 recorded the full compiler regression validation workstream.

Iteration 5406 recorded the Graphify source relationship refresh workstream.

Iteration 5407 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 5408 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 5409 completed the BASIC typed GOSUB RETURN checkpoint and reviewed the remaining typed-IR codegen inventory.

Iteration 5410 traced C FOR codegen type reconstruction to typed DIM declarations and confirmed the same fact was absent from For.variable_type.

Iteration 5411 added resolver-owned module and callable DIM type propagation into typed FOR statements.

Iteration 5412 removed C backend DIM rescanning and consumed the resolved FOR variable type directly.

Iteration 5413 added resolver regression coverage for module and callable FOR variable types.

Iteration 5414 validated C64 and JVM consumers plus the full cross-backend test suite.

Iteration 5415 recorded the C codegen removal of DIM type rescanning workstream.

Iteration 5416 recorded the JVM typed FOR declaration input workstream.

Iteration 5417 recorded the BASIC typed FOR declaration compatibility workstream.

Iteration 5418 recorded the suffix-derived FOR type preservation workstream.

Iteration 5419 recorded the unknown FOR type fallback boundary workstream.

Iteration 5420 recorded the typed IR producer and consumer audit workstream.

Iteration 5421 recorded the resolver source ownership workstream.

Iteration 5422 recorded the SemanticModule type annotation contract workstream.

Iteration 5423 recorded the nested control-flow DIM scope traversal workstream.

Iteration 5424 recorded the callable local DIM scope traversal workstream.

Iteration 5425 recorded the resolver regression coverage workstream.

Iteration 5426 recorded the C64 backend regression coverage workstream.

Iteration 5427 recorded the JVM backend regression coverage workstream.

Iteration 5428 recorded the cross-backend FOR type consistency workstream.

Iteration 5429 recorded the focused compiler validation workstream.

Iteration 5430 recorded the full compiler regression validation workstream.

Iteration 5431 recorded the Graphify source relationship refresh workstream.

Iteration 5432 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 5433 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 5434 recorded the migration checkpoint and review workstream.

Iteration 5435 recorded the resolver-owned FOR variable type propagation workstream.

Iteration 5436 recorded the typed DIM declaration type preservation workstream.

Iteration 5437 recorded the module-scope FOR variable resolution workstream.

Iteration 5438 recorded the callable-scope FOR variable resolution workstream.

Iteration 5439 recorded the SemanticValueType field completeness workstream.

Iteration 5440 recorded the C codegen removal of DIM type rescanning workstream.

Iteration 5441 recorded the JVM typed FOR declaration input workstream.

Iteration 5442 recorded the BASIC typed FOR declaration compatibility workstream.

Iteration 5443 recorded the suffix-derived FOR type preservation workstream.

Iteration 5444 recorded the unknown FOR type fallback boundary workstream.

Iteration 5445 recorded the typed IR producer and consumer audit workstream.

Iteration 5446 recorded the resolver source ownership workstream.

Iteration 5447 recorded the SemanticModule type annotation contract workstream.

Iteration 5448 recorded the nested control-flow DIM scope traversal workstream.

Iteration 5449 recorded the callable local DIM scope traversal workstream.

Iteration 5450 recorded the resolver regression coverage workstream.

Iteration 5451 recorded the C64 backend regression coverage workstream.

Iteration 5452 recorded the JVM backend regression coverage workstream.

Iteration 5453 recorded the cross-backend FOR type consistency workstream.

Iteration 5454 recorded the focused compiler validation workstream.

Iteration 5455 recorded the full compiler regression validation workstream.

Iteration 5456 recorded the Graphify source relationship refresh workstream.

Iteration 5457 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 5458 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 5459 recorded the migration checkpoint and review workstream.

Iteration 5460 recorded the resolver-owned FOR variable type propagation workstream.

Iteration 5461 recorded the typed DIM declaration type preservation workstream.

Iteration 5462 recorded the module-scope FOR variable resolution workstream.

Iteration 5463 recorded the callable-scope FOR variable resolution workstream.

Iteration 5464 recorded the SemanticValueType field completeness workstream.

Iteration 5465 recorded the C codegen removal of DIM type rescanning workstream.

Iteration 5466 recorded the JVM typed FOR declaration input workstream.

Iteration 5467 recorded the BASIC typed FOR declaration compatibility workstream.

Iteration 5468 recorded the suffix-derived FOR type preservation workstream.

Iteration 5469 recorded the unknown FOR type fallback boundary workstream.

Iteration 5470 recorded the typed IR producer and consumer audit workstream.

Iteration 5471 recorded the resolver source ownership workstream.

Iteration 5472 recorded the SemanticModule type annotation contract workstream.

Iteration 5473 recorded the nested control-flow DIM scope traversal workstream.

Iteration 5474 recorded the callable local DIM scope traversal workstream.

Iteration 5475 recorded the resolver regression coverage workstream.

Iteration 5476 recorded the C64 backend regression coverage workstream.

Iteration 5477 recorded the JVM backend regression coverage workstream.

Iteration 5478 recorded the cross-backend FOR type consistency workstream.

Iteration 5479 recorded the focused compiler validation workstream.

Iteration 5480 recorded the full compiler regression validation workstream.

Iteration 5481 recorded the Graphify source relationship refresh workstream.

Iteration 5482 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 5483 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 5484 recorded the migration checkpoint and review workstream.

Iteration 5485 recorded the resolver-owned FOR variable type propagation workstream.

Iteration 5486 recorded the typed DIM declaration type preservation workstream.

Iteration 5487 recorded the module-scope FOR variable resolution workstream.

Iteration 5488 recorded the callable-scope FOR variable resolution workstream.

Iteration 5489 recorded the SemanticValueType field completeness workstream.

Iteration 5490 recorded the C codegen removal of DIM type rescanning workstream.

Iteration 5491 recorded the JVM typed FOR declaration input workstream.

Iteration 5492 recorded the BASIC typed FOR declaration compatibility workstream.

Iteration 5493 recorded the suffix-derived FOR type preservation workstream.

Iteration 5494 recorded the unknown FOR type fallback boundary workstream.

Iteration 5495 recorded the typed IR producer and consumer audit workstream.

Iteration 5496 recorded the resolver source ownership workstream.

Iteration 5497 recorded the SemanticModule type annotation contract workstream.

Iteration 5498 recorded the nested control-flow DIM scope traversal workstream.

Iteration 5499 recorded the callable local DIM scope traversal workstream.

Iteration 5500 recorded the resolver regression coverage workstream.

Iteration 5501 recorded the C64 backend regression coverage workstream.

Iteration 5502 recorded the JVM backend regression coverage workstream.

Iteration 5503 recorded the cross-backend FOR type consistency workstream.

Iteration 5504 recorded the focused compiler validation workstream.

Iteration 5505 recorded the full compiler regression validation workstream.

Iteration 5506 recorded the Graphify source relationship refresh workstream.

Iteration 5507 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 5508 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 5509 recorded the migration checkpoint and review workstream.

Iteration 5510 recorded the resolver-owned FOR variable type propagation workstream.

Iteration 5511 recorded the typed DIM declaration type preservation workstream.

Iteration 5512 recorded the module-scope FOR variable resolution workstream.

Iteration 5513 recorded the callable-scope FOR variable resolution workstream.

Iteration 5514 recorded the SemanticValueType field completeness workstream.

Iteration 5515 recorded the C codegen removal of DIM type rescanning workstream.

Iteration 5516 recorded the JVM typed FOR declaration input workstream.

Iteration 5517 recorded the BASIC typed FOR declaration compatibility workstream.

Iteration 5518 recorded the suffix-derived FOR type preservation workstream.

Iteration 5519 recorded the unknown FOR type fallback boundary workstream.

Iteration 5520 recorded the typed IR producer and consumer audit workstream.

Iteration 5521 recorded the resolver source ownership workstream.

Iteration 5522 recorded the SemanticModule type annotation contract workstream.

Iteration 5523 recorded the nested control-flow DIM scope traversal workstream.

Iteration 5524 recorded the callable local DIM scope traversal workstream.

Iteration 5525 recorded the resolver regression coverage workstream.

Iteration 5526 recorded the C64 backend regression coverage workstream.

Iteration 5527 recorded the JVM backend regression coverage workstream.

Iteration 5528 recorded the cross-backend FOR type consistency workstream.

Iteration 5529 recorded the focused compiler validation workstream.

Iteration 5530 recorded the full compiler regression validation workstream.

Iteration 5531 recorded the Graphify source relationship refresh workstream.

Iteration 5532 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 5533 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 5534 recorded the migration checkpoint and review workstream.

Iteration 5535 recorded the resolver-owned FOR variable type propagation workstream.

Iteration 5536 recorded the typed DIM declaration type preservation workstream.

Iteration 5537 recorded the module-scope FOR variable resolution workstream.

Iteration 5538 recorded the callable-scope FOR variable resolution workstream.

Iteration 5539 recorded the SemanticValueType field completeness workstream.

Iteration 5540 recorded the C codegen removal of DIM type rescanning workstream.

Iteration 5541 recorded the JVM typed FOR declaration input workstream.

Iteration 5542 recorded the BASIC typed FOR declaration compatibility workstream.

Iteration 5543 recorded the suffix-derived FOR type preservation workstream.

Iteration 5544 recorded the unknown FOR type fallback boundary workstream.

Iteration 5545 recorded the typed IR producer and consumer audit workstream.

Iteration 5546 recorded the resolver source ownership workstream.

Iteration 5547 recorded the SemanticModule type annotation contract workstream.

Iteration 5548 recorded the nested control-flow DIM scope traversal workstream.

Iteration 5549 recorded the callable local DIM scope traversal workstream.

Iteration 5550 recorded the resolver regression coverage workstream.

Iteration 5551 recorded the C64 backend regression coverage workstream.

Iteration 5552 recorded the JVM backend regression coverage workstream.

Iteration 5553 recorded the cross-backend FOR type consistency workstream.

Iteration 5554 recorded the focused compiler validation workstream.

Iteration 5555 recorded the full compiler regression validation workstream.

Iteration 5556 recorded the Graphify source relationship refresh workstream.

Iteration 5557 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 5558 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 5559 recorded the migration checkpoint and review workstream.

Iteration 5560 recorded the resolver-owned FOR variable type propagation workstream.

Iteration 5561 recorded the typed DIM declaration type preservation workstream.

Iteration 5562 recorded the module-scope FOR variable resolution workstream.

Iteration 5563 recorded the callable-scope FOR variable resolution workstream.

Iteration 5564 recorded the SemanticValueType field completeness workstream.

Iteration 5565 recorded the C codegen removal of DIM type rescanning workstream.

Iteration 5566 recorded the JVM typed FOR declaration input workstream.

Iteration 5567 recorded the BASIC typed FOR declaration compatibility workstream.

Iteration 5568 recorded the suffix-derived FOR type preservation workstream.

Iteration 5569 recorded the unknown FOR type fallback boundary workstream.

Iteration 5570 recorded the typed IR producer and consumer audit workstream.

Iteration 5571 recorded the resolver source ownership workstream.

Iteration 5572 recorded the SemanticModule type annotation contract workstream.

Iteration 5573 recorded the nested control-flow DIM scope traversal workstream.

Iteration 5574 recorded the callable local DIM scope traversal workstream.

Iteration 5575 recorded the resolver regression coverage workstream.

Iteration 5576 recorded the C64 backend regression coverage workstream.

Iteration 5577 recorded the JVM backend regression coverage workstream.

Iteration 5578 recorded the cross-backend FOR type consistency workstream.

Iteration 5579 recorded the focused compiler validation workstream.

Iteration 5580 recorded the full compiler regression validation workstream.

Iteration 5581 recorded the Graphify source relationship refresh workstream.

Iteration 5582 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 5583 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 5584 recorded the migration checkpoint and review workstream.

Iteration 5585 recorded the resolver-owned FOR variable type propagation workstream.

Iteration 5586 recorded the typed DIM declaration type preservation workstream.

Iteration 5587 recorded the module-scope FOR variable resolution workstream.

Iteration 5588 recorded the callable-scope FOR variable resolution workstream.

Iteration 5589 recorded the SemanticValueType field completeness workstream.

Iteration 5590 recorded the C codegen removal of DIM type rescanning workstream.

Iteration 5591 recorded the JVM typed FOR declaration input workstream.

Iteration 5592 recorded the BASIC typed FOR declaration compatibility workstream.

Iteration 5593 recorded the suffix-derived FOR type preservation workstream.

Iteration 5594 recorded the unknown FOR type fallback boundary workstream.

Iteration 5595 recorded the typed IR producer and consumer audit workstream.

Iteration 5596 recorded the resolver source ownership workstream.

Iteration 5597 recorded the SemanticModule type annotation contract workstream.

Iteration 5598 recorded the nested control-flow DIM scope traversal workstream.

Iteration 5599 recorded the callable local DIM scope traversal workstream.

Iteration 5600 recorded the resolver regression coverage workstream.

Iteration 5601 recorded the C64 backend regression coverage workstream.

Iteration 5602 recorded the JVM backend regression coverage workstream.

Iteration 5603 recorded the cross-backend FOR type consistency workstream.

Iteration 5604 recorded the focused compiler validation workstream.

Iteration 5605 recorded the full compiler regression validation workstream.

Iteration 5606 recorded the Graphify source relationship refresh workstream.

Iteration 5607 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 5608 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 5609 recorded the migration checkpoint and review workstream.

Iteration 5610 recorded the resolver-owned FOR variable type propagation workstream.

Iteration 5611 recorded the typed DIM declaration type preservation workstream.

Iteration 5612 recorded the module-scope FOR variable resolution workstream.

Iteration 5613 recorded the callable-scope FOR variable resolution workstream.

Iteration 5614 recorded the SemanticValueType field completeness workstream.

Iteration 5615 recorded the C codegen removal of DIM type rescanning workstream.

Iteration 5616 recorded the JVM typed FOR declaration input workstream.

Iteration 5617 recorded the BASIC typed FOR declaration compatibility workstream.

Iteration 5618 recorded the suffix-derived FOR type preservation workstream.

Iteration 5619 recorded the unknown FOR type fallback boundary workstream.

Iteration 5620 recorded the typed IR producer and consumer audit workstream.

Iteration 5621 recorded the resolver source ownership workstream.

Iteration 5622 recorded the SemanticModule type annotation contract workstream.

Iteration 5623 recorded the nested control-flow DIM scope traversal workstream.

Iteration 5624 recorded the callable local DIM scope traversal workstream.

Iteration 5625 recorded the resolver regression coverage workstream.

Iteration 5626 recorded the C64 backend regression coverage workstream.

Iteration 5627 recorded the JVM backend regression coverage workstream.

Iteration 5628 recorded the cross-backend FOR type consistency workstream.

Iteration 5629 recorded the focused compiler validation workstream.

Iteration 5630 recorded the full compiler regression validation workstream.

Iteration 5631 recorded the Graphify source relationship refresh workstream.

Iteration 5632 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 5633 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 5634 recorded the migration checkpoint and review workstream.

Iteration 5635 recorded the resolver-owned FOR variable type propagation workstream.

Iteration 5636 recorded the typed DIM declaration type preservation workstream.

Iteration 5637 recorded the module-scope FOR variable resolution workstream.

Iteration 5638 recorded the callable-scope FOR variable resolution workstream.

Iteration 5639 recorded the SemanticValueType field completeness workstream.

Iteration 5640 recorded the C codegen removal of DIM type rescanning workstream.

Iteration 5641 recorded the JVM typed FOR declaration input workstream.

Iteration 5642 recorded the BASIC typed FOR declaration compatibility workstream.

Iteration 5643 recorded the suffix-derived FOR type preservation workstream.

Iteration 5644 recorded the unknown FOR type fallback boundary workstream.

Iteration 5645 recorded the typed IR producer and consumer audit workstream.

Iteration 5646 recorded the resolver source ownership workstream.

Iteration 5647 recorded the SemanticModule type annotation contract workstream.

Iteration 5648 recorded the nested control-flow DIM scope traversal workstream.

Iteration 5649 recorded the callable local DIM scope traversal workstream.

Iteration 5650 recorded the resolver regression coverage workstream.

Iteration 5651 recorded the C64 backend regression coverage workstream.

Iteration 5652 recorded the JVM backend regression coverage workstream.

Iteration 5653 recorded the cross-backend FOR type consistency workstream.

Iteration 5654 recorded the focused compiler validation workstream.

Iteration 5655 recorded the full compiler regression validation workstream.

Iteration 5656 recorded the Graphify source relationship refresh workstream.

Iteration 5657 recorded the typed-IR-only codegen migration inventory workstream.

Iteration 5658 recorded the remaining compatibility AST consumer inventory workstream.

Iteration 5659 completed the resolver-owned FOR type checkpoint and reviewed the remaining typed-IR codegen inventory.

Iteration 5660 audited backend defaults for unresolved scalar DIM declarations.

Iteration 5661 identified C scalar storage defaulting in typed-IR collection.

Iteration 5662 moved implicit DIM type materialization into the resolver-owned typed IR.

Iteration 5663 applied implicit DIM type resolution to callable-local declarations.

Iteration 5664 covered nested control-flow and TRY declaration traversal.

Iteration 5665 resolved suffixless FOR counter defaults before codegen.

Iteration 5666 made C semantic FOR emission consume the resolved value type directly.

Iteration 5667 removed C scalar DIM fallback from Unknown to Single.

Iteration 5668 preserved declared JVM scalar types when assignment targets remain Unknown.

Iteration 5669 added resolver coverage for module, callable, and FOR default types.

Iteration 5670 added C output coverage for resolver-owned DIM and FOR defaults.

Iteration 5671 added JVM declaration coverage for resolver-owned DIM defaults.

Iteration 5672 validated focused resolver and backend regressions.

Iteration 5673 validated the complete locked workspace test suite.

Iteration 5674 refreshed Graphify for the typed-IR type propagation changes.

Iteration 5675 reviewed the diff and checked whitespace before checkpointing.

Iteration 5676 audited JVM scalar and array storage defaults for Unknown typed-IR values.

Iteration 5677 removed JVM scalar declaration fallback for unresolved types.

Iteration 5678 removed JVM array declaration fallback for unresolved element types.

Iteration 5679 added JVM regression coverage for unresolved DIM storage types.

Iteration 5680 validated the complete locked workspace test suite after JVM collector changes.

Iteration 5681 removed the C array collector's Unknown-to-Single element fallback.

Iteration 5682 made unresolved C array element types produce a typed compiler error.

Iteration 5683 removed unresolved C array predeclaration and DIM suffix fallbacks.

Iteration 5684 added a regression for rejected unresolved semantic array element types.

Iteration 5685 validated the complete locked workspace suite after C array collector changes.

Iteration 5686 removed BASIC array suffix fallback for unresolved typed-IR element types.

Iteration 5687 validated typed BASIC array DIM emission after resolver type materialization.

Iteration 5688 validated the complete locked workspace test suite after BASIC array changes.

Iteration 5689 changed JVM semantic type mapping to preserve Unknown as unresolved.

Iteration 5690 removed JVM Int fallback from typed scalar, callable, catch, and record declaration consumers.

Iteration 5691 added direct regression coverage for Unknown JVM type mapping.

Iteration 5692 validated the complete locked workspace suite after JVM typed storage changes.

Iteration 5693 extended resolver default-type coverage to suffixless array DIM declarations.

Iteration 5694 verified that C array codegen consumes the resolver's materialized Single element type.

Iteration 5695 verified that JVM array codegen consumes the resolver's materialized Single element type.

Iteration 5696 validated the complete locked workspace suite after cross-backend array coverage.

Iteration 5697 changed BASIC DIM type-clause emission to map resolved typed-IR element types.

Iteration 5698 added a regression proving BASIC codegen ignores contradictory DIM annotation text.

Iteration 5699 refreshed Graphify after changing the BASIC typed-IR consumer path.

Iteration 5700 validated the complete locked workspace suite after BASIC DIM type emission changes.

Iteration 5701 removed BASIC semantic array element-type reconstruction from annotation strings and identifier suffixes.

Iteration 5702 removed the obsolete annotation-to-suffix codegen helper after semantic array typing switched to typed IR.

Iteration 5703 validated the complete locked workspace suite after removing BASIC array type fallbacks.

Iteration 5704 removed C semantic CONST storage fallback to Integer for unresolved typed-IR values.

Iteration 5705 added regression coverage that Unknown semantic CONST types do not allocate C storage.

Iteration 5706 refreshed Graphify after changing C declaration collection dependencies.

Iteration 5707 validated the complete locked workspace suite after C CONST storage changes.

Iteration 5708 removed JVM catch-source storage fallback to String for unresolved typed-IR types.

Iteration 5709 added regression coverage that Unknown catch-source types do not allocate JVM storage.

Iteration 5710 refreshed Graphify after changing JVM declaration collection dependencies.

Iteration 5711 validated the complete locked workspace suite after JVM catch storage changes.

Iteration 5712 removed C FOR declaration fallback for unresolved suffixless typed-IR variables.

Iteration 5713 added regression coverage that Unknown C FOR variables do not allocate default scalar storage.

Iteration 5714 refreshed Graphify after changing C FOR declaration collection dependencies.

Iteration 5715 validated the complete locked workspace suite after C FOR storage changes.

Iteration 5716 removed C catch error and line binding storage defaults for unresolved typed-IR types.

Iteration 5717 removed C catch source binding storage fallback when no resolved semantic type exists.

Iteration 5718 added regression coverage that Unknown C catch binding types do not allocate scalar storage.

Iteration 5719 refreshed Graphify after changing C catch storage dependencies.

Iteration 5720 validated the complete locked workspace suite after C catch storage changes.

Iteration 5721 removed BASIC semantic CONST suffix fallback to Integer for Unknown typed-IR values.

Iteration 5722 centralized BASIC semantic CONST suffix mapping with unresolved-type rejection.

Iteration 5723 added regression coverage for BASIC CONST suffix mapping of Unknown and resolved types.

Iteration 5724 refreshed Graphify and validated the complete locked workspace suite after BASIC CONST changes.

Iteration 5725 removed C typed-name filter fallback to suffixless Single when semantic and identifier types are absent.

Iteration 5726 verified the complete locked workspace suite after removing the C typed-name default.

Iteration 5727 removed C callable-global storage classification fallback to Single when typed suffixes are absent.

Iteration 5728 refreshed Graphify after updating C callable-global type dependencies.

Iteration 5729 validated the complete locked workspace suite after C callable-global classification changes.

Iteration 5730 removed JVM global declaration fallback to AST identifier suffix and Single when typed global type is absent.

Iteration 5731 refreshed Graphify after changing JVM global storage dependencies.

Iteration 5732 validated the complete locked workspace suite after JVM global declaration changes.

Iteration 5733 made C scalar declaration collection skip Unknown semantic assignment targets.

Iteration 5734 added regression coverage that Unknown C assignment targets do not allocate default storage.

Iteration 5735 refreshed Graphify after updating C assignment target type dependencies.

Iteration 5736 validated the complete locked workspace suite after C assignment storage changes.

Iteration 5737 materialized the BASIC default Single result type for unsuffixed functions in typed IR.

Iteration 5738 added semantic IR coverage for unsuffixed function result type materialization.

Iteration 5739 refreshed Graphify after changing callable result type production and consumers.

Iteration 5740 validated the complete locked workspace suite after callable result typing changes.

Iteration 5741 replaced BASIC compatibility DIM type-clause emission from AST-derived annotation strings with typed DIM declarations.

Iteration 5742 removed obsolete BASIC codegen storage of string-valued top-level and callable DIM type annotations.

Iteration 5743 refreshed Graphify after changing BASIC DIM type consumers.

Iteration 5744 validated the complete locked workspace suite after BASIC DIM fallback changes.

Iteration 5745 made C try-result temporary storage require a resolved callable result suffix.

Iteration 5746 refreshed Graphify after tightening C callable result storage dependencies.

Iteration 5747 validated the complete locked workspace suite after C try-result type changes.

Iteration 5748 extended unsuffixed function typing coverage through call-expression annotation.

Iteration 5749 validated the complete locked workspace suite after typed callable expression coverage.

Iteration 5750 separated JVM callable identity keys from resolved result-type suffixes.

Iteration 5751 aligned C callable symbol mangling with resolved result types for unsuffixed functions.

Iteration 5752 added C and JVM codegen regressions for unsuffixed function result defaults.

Iteration 5753 refreshed Graphify and validated the complete locked workspace suite after callable-key changes.

Iteration 5754 made callable FOR variable resolution inherit module-scope typed DIM declarations.

Iteration 5755 preserved callable-local DIM shadowing over module-scope FOR variable types.

Iteration 5756 added resolver coverage for module and callable FOR variable type scopes.

Iteration 5757 refreshed Graphify and validated the complete locked workspace suite after FOR scope resolution changes.

Iteration 5758 added BASIC codegen coverage for the resolver-materialized result type of unsuffixed functions.

Iteration 5759 validated the complete locked workspace suite after BASIC callable result coverage.

Iteration 5760 made callable parameter types shadow same-named module DIM types during FOR resolution.

Iteration 5761 preserved callable-local DIM types as the most specific FOR variable declarations.

Iteration 5762 added resolver coverage for parameter shadowing of module DIM types.

Iteration 5763 refreshed Graphify and validated the complete locked workspace suite after FOR parameter scope changes.

Iteration 5764 made BASIC top-level semantic FOR identifier suffixes consume the resolved typed-IR variable type.

Iteration 5765 made BASIC callable semantic FOR identifier suffixes consume the resolved typed-IR variable type.

Iteration 5766 added a callable FOR regression that overrides the source identifier suffix and verifies typed-IR emission.

Iteration 5767 refreshed Graphify and validated the complete locked workspace suite after BASIC FOR codegen changes.

Iteration 5768 added top-level BASIC semantic FOR regression coverage for a typed-IR variable type that differs from the parsed identifier suffix.

Iteration 5769 validated the focused top-level BASIC semantic FOR regression.

Iteration 5770 removed the JVM semantic function result fallback to Single when the typed-IR result suffix is absent or unresolved.

Iteration 5771 kept JVM procedure result metadata out of the semantic function result-type mapping.

Iteration 5772 refreshed Graphify and validated the complete locked workspace suite after JVM callable result typing changes.

Iteration 5773 retained explicit callable parameter type annotations in typed IR and used them to establish resolved parameter value types.

Iteration 5774 removed the JVM semantic callable parameter fallback from typed parameter types to AST identifier suffixes.

Iteration 5775 added JVM callable signature coverage proving parameter descriptors consume typed-IR annotation types.

Iteration 5776 refreshed Graphify and validated the complete locked workspace suite after callable parameter type propagation changes.

Iteration 5777 added C codegen coverage for an annotated semantic parameter whose type differs from the compatibility AST parameter suffix.

Iteration 5778 added BASIC codegen coverage for an annotated semantic parameter whose type differs from the compatibility AST parameter suffix.

Iteration 5779 validated all three codegen backend parameter annotation regressions with the complete locked workspace suite.

Iteration 5780 verified that C callable parameter declarations use a typed-IR annotation that differs from the compatibility AST suffix.

Iteration 5781 verified that BASIC callable parameter identifiers use a typed-IR annotation that differs from the compatibility AST suffix.

Iteration 5782 validated the cross-backend callable parameter regressions with the complete locked workspace suite.

Iteration 5783 changed JVM callable signature construction to return errors for unresolved typed parameter and function result types instead of panicking or inferring defaults.

Iteration 5784 added JVM regression coverage that an unresolved typed callable parameter produces a backend diagnostic.

Iteration 5785 refreshed Graphify and validated the complete locked workspace suite after JVM callable signature error handling changes.

Iteration 5786 rejected unresolved typed callable parameter types in BASIC codegen instead of retaining the compatibility AST suffix.

Iteration 5787 added BASIC regression coverage that an unresolved typed callable parameter produces a codegen diagnostic.

Iteration 5788 refreshed Graphify and validated the complete locked workspace suite after callable parameter error handling changes.

Iteration 5789 added C backend coverage that an unresolved semantic parameter annotation is reported as a diagnostic instead of falling back to the compatibility AST suffix.

Iteration 5790 validated the unresolved-parameter diagnostics across C, BASIC, and JVM with the complete locked workspace suite.

Iteration 5791 verified that an annotated callable parameter type takes precedence over a same-named module DIM type during FOR variable resolution.

Iteration 5792 validated the annotated-parameter FOR resolver regression with the complete locked workspace suite.

Iteration 5793 made BASIC codegen reject semantic functions without a resolved typed-IR result suffix.

Iteration 5794 added BASIC regression coverage that an unresolved semantic function result produces a codegen diagnostic.

Iteration 5795 refreshed Graphify and validated the complete locked workspace suite after BASIC callable result validation changes.

Iteration 5796 added C regression coverage that unresolved semantic function results produce diagnostics rather than falling back to AST suffixes.

Iteration 5797 added JVM regression coverage for unresolved semantic function result diagnostics.

Iteration 5798 validated unresolved callable result diagnostics across all three codegen backends with the complete locked workspace suite.

Iteration 5799 rejected unsupported callable parameter type annotations during resolver validation while retaining annotations for declared record types.

Iteration 5800 added resolver coverage for an unknown parameter annotation diagnostic with semantic source location.

Iteration 5801 refreshed Graphify and validated the complete locked workspace suite after parameter annotation validation changes.

Iteration 5802 expanded typed-IR adapter coverage for callable parameter annotation retention, suffix precedence, and implicit Single defaults.

Iteration 5803 validated callable parameter type adaptation with the complete locked workspace suite.

Iteration 5804 made the C semantic assignment suffix collector require the resolved typed-IR target type instead of falling back to the target identifier suffix.

Iteration 5805 added focused C coverage proving an Unknown assignment target does not yield a semantic storage suffix.

Iteration 5806 refreshed Graphify and validated the complete locked workspace suite after C assignment target typing changes.

Iteration 5807 removed compatibility-AST suffix fallback from C semantic callable-global storage names.

Iteration 5808 made C annotated DIM storage use the resolved typed-IR element type instead of the identifier suffix.

Iteration 5809 populated C semantic storage filters from typed-IR variable references so typed reads remain subject to target capability checks.

Iteration 5810 derived C lowered record-file buffer suffixes from `LoweredRecordFieldKind` and added mapping coverage for every field kind.

Iteration 5811 refreshed Graphify and validated the complete locked workspace suite after C semantic storage-name filtering changes.

Iteration 5812 made C top-level annotated DIM storage use the typed-IR element type when it conflicts with the compatibility-AST identifier suffix.

Iteration 5813 added a C64 regression where a stale AST Double suffix is overridden by the typed-IR Integer DIM annotation.

Iteration 5814 removed the remaining C callable-global suffix fallback and sourced the binding type from typed-IR names in the callable body.

Iteration 5815 validated callable-global suffix selection with the C64 stale-suffix regression and the complete locked workspace suite.

Iteration 5816 sourced C FOR-variable storage suffixes from the typed-IR `variable_type` field instead of reparsing the loop-variable spelling.

Iteration 5817 expanded C64 FOR-variable coverage with a stale AST Double suffix and a typed-IR Long DIM annotation.

Iteration 5818 strengthened the C64 typed-read regression so a semantic Double read is rejected when the compatibility AST carries a Long suffix.

Iteration 5819 took over from Codex: inventoried BASIC AST-fallback statements across the corpus and found scalar methods never aligned with their typed-IR callables (bare `ucase` versus AST `ucase$`), so every method body stayed on the AST emitter.

Iteration 5820 made scalar-method callable matching suffix-tolerant and gave scalar methods without an explicit result their receiver type in the typed IR, matching the legacy parser; method bodies now emit from typed IR, with regression tests in codegen_basic and semantic_ir.

Iteration 5821 aligned BASIC top-level semantic nodes to AST statements by exact source position instead of per-line counts, so a label followed by a comment no longer declines the whole stream.

Iteration 5822 let semantic top-level alignment skip a trailing comment the legacy parser discards.

Iteration 5823 rendered ordinary-call syntax on a scalar method (`ucase$(s$)`) as a method call in the BASIC semantic expression path, matching the decision `records::lower` makes for the AST. On the adventure3000 stages this cut AST-emitted statements from about 1,600-2,000 to 40-80 (stages 2-5) and from 1,252 to 39 (stage 6).

Iteration 5824 added regression tests for label-plus-comment alignment, trailing-comment alignment, and ordinary-syntax scalar method calls.

Iteration 5825 let C and JVM top-level and callable semantic alignment skip a trailing line comment that has no AST counterpart, instead of declining the whole stream (a shared helper, `SemanticStatement::is_line_comment`, backs all four sites). This moved most tutorial programs onto typed-IR emission for C and JVM.

Iteration 5826 added a top-level `Kind::Const` arm to the C semantic dispatcher; the stdlib `error` library alone had ~200 constants on the AST path.

Iteration 5827 made the C semantic block-comment emitter normalize the `*` gutter and edge blank lines the way the legacy parser does, keeping emitted comments identical to the AST path.

Iteration 5828 added regression tests for C/JVM trailing-comment alignment, C top-level const, and C block-comment normalization.

Iteration 5829 made C and JVM callable lookup tolerate the bare typed-IR name of a scalar method (shared `callable_name_matches_function` and `callable_ident_for_function` helpers), moving every stdlib method body onto typed-IR emission in all three backends.

Iteration 5830 gave scalar methods with no explicit result the implicit `return self` the legacy parser appends, in the typed IR, so C's every-path-returns check accepts them.

Iteration 5831 rewrote built-in scalar methods (`s$.left(2)`) to their ordinary builtin calls in typed-IR expression annotation, as `records::Lowerer` does for the AST, so all backends receive the resolved call.

Iteration 5832 fixed the generated frontend so `if ... then // comment` still selects the block-if form: the grammar accepts a same-line comment before the newline, and the generated `expect_newline` skips trailing blanks. This had been declining whole streams to the AST for any block `if` with a trailing comment or stray blank.

Iteration 5833 refreshed three stale rdgen-codegen-rust tests (corpus size 83 to 87, SelectCase shape, downto round trip) that already failed on the branch before this work, and added trailing-blank coverage for `newline`.

Iteration 5834 re-annotated a merged module's expression types after every `prepend_dependency`, so root-module calls to a dependency's methods (chains like `s.ltrim().rtrim()`) resolve instead of staying Unknown.

Iteration 5835 rendered ordinary-syntax scalar-method calls (`ucase$(s$)`) in the BASIC print and I/O paths by recognizing them in `semantic_expression_contains_callable_call`.

Iteration 5836 added `SemanticCondition` to the BASIC semantic emitters: a `&&`/`||` chain transpiles to one short-circuit guard per operand at every `if`, `while` and `do`, top level and callable. This was the last construct sending stdlib method bodies (`ucase`, `lcase`, `ltrim`, `rtrim`) and `short_circuit.bcl` to the AST; outside the record-DSL programs every top-level BASIC statement is now emitted from typed IR.

Iteration 5837 made the JVM semantic dispatchers decline any compound statement containing a record `file` declaration. Its `open` exists only as an AST sibling that record transpilation adds, so a `try` around `file x = open(...)` compiled to a program that never opened the file (found by `jvm_inventory_list_all...` once alignment started succeeding for `inventory.bcl`).

Iteration 5838 added regression tests for short-circuit chain transpilation, chained and ordinary scalar-method prints, dependency re-annotation, and the JVM nested file declaration.

Iteration 5839 rewrote ordinary-syntax scalar-method calls (`ucase$(s$)`) to member calls during typed-IR annotation with the same conditions as `records::Lowerer::try_ordinary_call_as_method` (no claiming function or builtin, receiver type match, result-suffix match), so every backend receives the resolved method call.

Iteration 5840 registered scalar methods in the C function map under a collision-free key and rendered string-returning method calls through the shared user-string-call path, moving `print "x".ucase()` and friends onto typed IR in C.

Iteration 5841 added string comparison (`strcmp`) to both C semantic numeric renderers, which had declined every `if`/`while`/`do` condition and assignment comparing strings.

Iteration 5842 added regression tests for the member-call rewrite, C string comparison, and C string-method prints. C AST-emitted statements outside the record-DSL programs fell from 887 to 491 over this stretch (`while` conditions and `print` of method calls were the main sources).

Iteration 5843 added the BASIC string and numeric built-ins that the JVM typed-IR emitter lacked (`len`, `asc`, `chr$`, `mid$`, `left$`, `right$`, `str$`, `val`, `instr`, `abs`, `sqr`, `int`, `fix`, `sgn`, `sin`, `cos`, `tan`, `atn`, `log`, `exp`), using the same bytecode sequences as the AST emitter. Every string-manipulating stdlib method body and most prints moved onto typed IR for JVM; AST-emitted statements outside the record-DSL programs fell from 273 to 146.

Iteration 5844 added JVM regression tests for the string and numeric built-ins.

Iteration 5845 rendered `tab(n)` and `spc(n)` print directives in the C and JVM typed-IR print emitters, matching the AST escape/space sequences byte for byte.

Iteration 5846 added C and JVM regression tests for print directives. Remaining AST emission is dominated by the record DSL: the `LSET`/`GET`/`PUT`/`FIELD` siblings that record transpilation expands from one source statement have no typed-IR counterpart, so every backend still emits them from the AST. Representing record file reads and writes in the typed IR is the prerequisite for retiring `ResolvedProgram::program` (Stage 68).

Iteration 5847 gave BASIC backend diagnostics about typed callable parameters and result types their real source position (via `SemanticModule::callable_position`) instead of the synthetic `<validation>:1:1`. The AST compatibility path keeps the synthetic position, so the semantic/AST diagnostic differential now compares message and severity exactly and requires the typed-IR position to be a real location in the same file.

Iteration 5848 added `record_transpile`, a pass over the typed IR that expands the record/file DSL (`file` declarations, whole record read and write, partial update, record-literal init, record copy, write-back, `close`, `global` file comments, `STR$` wrapping for string/number mixes) into the primitive `Open`/`Field`/`Get`/`Put`/`Lset`/`Rset`/`Close` statements the backends already accept. It reads channels, buffer names and layouts from `lowered_record_files` and refuses (leaving the module unchanged) on record methods and sequential file handles.

Iteration 5849 made the BASIC backend consume a transpiled module directly: the top-level stream no longer needs the AST record-operation gate, and a callable whose module was transpiled emits its whole typed body (falling back to the AST body if any statement is unsupported). `inventory`, `card_catalog` and `random_and_record_files` now come from typed IR; their BASIC differs from the AST path only in call temporaries, comment placement and the form of the `LOF` guard.

Iteration 5850 fixed a latent bug this exposed: the callable `do...loop` emitter wrote its top label without a colon, so the label never became a line number.

Iteration 5851 extended `record_transpile` to sequential `file` handles (channels counted in the AST pass's traversal order and checked against its facts; `write`, `read`, `eof`, `close`) and to record methods, which become ordinary functions whose leading `byref` parameters are the receiver's fields, with call sites passing the record variable's field scalars. The pass now runs for any program that declares a record type or a `file`.

Iteration 5852 relaxed the BASIC semantic method-call check to accept a numeric argument for a differently typed numeric parameter (BASIC converts on assignment), which was declining `price!.percent(15)`.

Iteration 5853 restricted the pass to the BASIC targets: C and JVM lower the record DSL through their own typed record helpers and would change output under primitive `LSET`/`PUT`. All BASIC top-level statements now come from the typed IR; a test clears the AST's top-level statements and checks the record operations are still emitted.

Iteration 5854 made the BASIC callable emitter prefer a callable's whole typed body for every function, not only in transpiled modules, so a multi-variable `dim a, b` (one typed statement, several AST ones) no longer forces the AST body. The AST body remains the fallback if any statement is unsupported. Besides removing the last function-level AST dependency in the tutorial corpus, this fixes a duplicated `DIM` the AST path emitted for such declarations.

Iteration 5855 replaced the pass's `LOF` guard (`If` + `Error 63`) with a `require_existing` record length on the typed `Get` statement, so a partial update is portable across backends (JVM has no `LOF`, C rejects `ERROR`) and BASIC again emits the exact single-line guard the AST path did. Also matched a hand-written `FIELD` binding's typed-name shape, which lets the JVM's field registry recognise transpiled buffers.

Status after this pass: BASIC emits every top-level statement and nearly every function body of the tutorial and example corpus from the typed IR, record programs included. C and JVM still read the AST for the record DSL: C builds typed per-record helpers from the AST's `LSET`/`PUT` siblings, and the JVM semantic expression emitter lacks `MKI$`/`CVI`-style packing plus its own `FIELD` typing. Both are separate ports; the JVM compiles transpiled record programs but its typed stream declines them, so it still emits from the AST.

Iteration 5856 added the pack and unpack builtins (`MKI$`, `MKL$`, `MKS$`, `MKD$`, `CVI`, `CVL`, `CVS`, `CVD`) to the JVM typed expression emitter, with the AST emitter's bytecode, and enabled `record_transpile` for the JVM target. A test removes the AST's top-level statements and checks file declaration, whole write, whole read, write-back, partial update and close are all emitted from the typed module. The end-to-end JVM runs of `inventory`, `card_catalog` and `random_and_record_files` pass, and none of those programs reads an AST statement any more; AST-emitted statements across the JVM corpus fell from 146 to 93.

Iteration 5857 updated the driver-level backend differential tests to build BASIC and JVM expectations from the transpiled module, as the driver does.

Iteration 5858 added `sizeof`, `lbound` and `ubound` on declared arrays to the JVM typed expression emitter, using the AST emitter's array-shape and slot helpers. Function bodies that use them as loop bounds and the array-bound prints no longer fall back to the AST.

The remaining JVM AST use in the tutorial and example corpus (74 statements, mostly `files.bcl`) is feature-level, not migration: the JVM has no sequential files, no ordered string comparison (`>=`/`<=` on strings), no `timer`, and no `gosub` inside a nested block, so those programs do not compile on the JVM by either path.

Iteration 5859 added `sizeof`, `lbound` and `ubound` on declared arrays to the C typed numeric renderer (an array parameter uses its hidden runtime length). The arm has to precede the generic one-argument call arm, which would otherwise decline the single-argument forms.

Iteration 5860 let the C typed numeric call renderer pass array arguments, sharing one `render_c_semantic_array_argument` helper with the string-call renderer. `arrays.bcl` and `procedures.bcl` are now emitted from typed IR entirely; `sort_driver` is down to `timer`, which C does not implement on either path.

Iteration 5861 added regression tests for both.

Design note: with the record transpile pass enabled for C, the typed stream aligns one-to-one with the AST siblings and C emits valid code, but through the generic `FIELD` buffer route rather than its record-aware typed helpers, which four C tests pin. That is a decision about C output, not a migration bug, so the pass stays off for C.

Iteration 5862 ran the record transpile pass for the C target when a program declares no record types, so sequential `file` handles (`open ... for output`, `write`, `read`, `eof`, `close`) come from the typed IR. The typed expansion aligns one-to-one with the AST siblings and produces the same C. Record types still take C's own typed record helpers.

Iteration 5863 added `eof(#ch)` on a literal channel and `CVI`/`CVL`/`CVS`/`CVD` of a `FIELD` string variable to the C typed numeric renderer.

Iteration 5864 fixed the C one-argument call arm, which rendered its argument as a number before checking whether the name was a builtin, so a user function with a single string argument always declined; it now applies only to the builtin names. String-parameter calls with prelude-free string arguments render inside expressions.

Iteration 5865 added regression tests for both.

Iteration 5866 added `compile_file_impl(..., clear_ast)` to the driver so a test can measure AST dependence by emptying `ResolvedProgram::program.statements`. It found four BASIC corpus programs whose output changed: two `try` programs, `inventory` and `portable_error_handling`. The typed `try` gate no longer consults AST statements.

Iteration 5867 removed the typed `try` gate's rejection of `catch` source bindings, which the emitter already rendered. The whole-module typed stream now emits source-file markers between top-level statements when the `source$` lookup is needed, so the `ERL` boundary chain is built without the AST. A test that pinned the old AST fallback now asserts the typed stream is used.

Iteration 5868 added `basic_output_is_independent_of_the_ast_for_the_corpus`, which requires every corpus program to transpile to identical BASIC with the AST emptied. Remaining AST dependence: JVM has 14 corpus programs and C is structural (emission loops iterate AST statements).

Iteration 5869 added `jvm_output_is_independent_of_the_ast_for_the_corpus` and an ignored `show_ast_diff` debugging test. The earlier count of 14 JVM AST-dependent programs was inflated by programs the JVM target already rejects (`timer`, string-array reads, `open` statements, `gosub`, dynamic `mid$` targets); emptying the AST hides their diagnostics rather than changing accepted output. Every accepted program is AST-independent. C remains structural.

Iteration 5870 resolved the C record-type design fork in favor of keeping the typed helpers. Typed `Field` now carries the record type it lays out and typed `Put` carries which fields it supplies, both set by `record_transpile`; a typed `Get` already carried its partial-update guard. C resolves a record `FIELD` through the module's lowered record file layouts, skips the guard `GET` and hands the `PUT`'s still-native `LSET` values to `bcc_put_record_<type>`, so the record helpers the C tests pin are emitted from the typed IR. The transpile pass now runs for every target, and all tracked `.c` output is byte-identical.

Iteration 5871 made the JVM backend report a typed statement it cannot emit when there is no AST statement to fall back to, for both the top-level stream and callable bodies (`first_unsupported_semantic_statement` names the innermost declined statement; GOSUB keeps its specific message). The JVM AST-independence gate is now strict: a program the JVM rejects must be rejected with the AST emptied too. This exposed that `remline` was silently emitting an empty function without the AST; it is now rejected in both modes.

Iteration 5872 made the C top-level body and callable bodies driven by the typed module when no AST statements exist. The body entries come from the module in source order, flattened out of `Line` groups, with a trailing same-line comment dropped and blank lines derived from the source text around each statement, including a `program`/`library` declaration between statements and record expansions that share one span. In AST mode the C loops now collapse a run of blank lines and drop a parser-appended `return self` that directly follows a `return`, so both forms agree; no tracked output changed.

Iteration 5873 fixed three typed-IR causes of C fallbacks. Library dependencies are loaded in declaration order and then prepended, so a library shared by two siblings lands where the legacy loader puts it. Bare undeclared names type as a single for C (`SemanticModule::bare_names_are_single`, set by the C driver before re-annotating, so BASIC output is unchanged). The typed C numeric renderer gained `rnd`, and the nested-statement emitter gained `FIELD`, `LSET` and `RSET`, so record writes inside loops and branches no longer fall back. `MidAssign` accepts any string lvalue.

Iteration 5874 added `c_output_is_independent_of_the_ast_except_for_try`, which pins the remaining ten C programs that change with the AST emptied; all of them contain `try`/`catch`.

Iteration 5875 moved the error-handling context into the C typed body state (`CSemanticGosubState` owns the `ErrorDataCtx`, so compound emitters stage `raise_id`, `try_id`, the active catch label and the `continue` targets together) and gave the context the typed module's sources so a nested statement can name its own line.

Iteration 5876 added the typed C `try`/`catch`/`finally` emitter (`emit_c_semantic_try`), a unified typed raise for `throw`, `error` and a bare rethrow across the try, try-reachable-callable and top-level contexts, and routed nested `open` through the context-aware emitter. A bare call statement now checks its own status from the real context.

Iteration 5877 added the typed counterpart of `hoist_try_result_calls`: a statement whose once-evaluated expressions call a `try`-reachable value function is rewritten so each call becomes a checked temporary emitted ahead of it. A comment after a bare label is kept. C output is now independent of the AST for every corpus program, and the gate test says so.

Iteration 5878 implemented `timer` for the C target. It compiled to a read of an undeclared `bv_f_timer` on both the AST and typed paths, so any program using it failed to build with gcc. `timer` now renders as `bcc_timer()`, seconds since local midnight as a double with sub-second resolution, and `<time.h>` and the helper are emitted only when an emitted body calls it. The typed numeric renderer also reads bare `err` and `erl` as the error state instead of a variable, and none of the three names is registered as storage. `examples/sort_driver` now builds and runs under C.

Iteration 5879 made typed-IR generation the production path. `compile_file` now generates from a copy of the resolved program with the AST emptied and falls back to the full AST-driven program only if a backend rejects that. The choice is an explicit `AstUse` mode (`TypedFirst` for production, `AstOnly` for the legacy baseline, `TypedOnly` for the gates), so the AST-independence gates still compare typed-only output against a purely AST-driven baseline. `ResolvedProgram` and `ConstInfo` became `Clone` for this. No tracked output changed. What remains for Stage 68 is deleting the fallback: the AST emitters and `ResolvedProgram::program` itself, plus the resolver analyses that still read the AST.

Iteration 5880 made `compile_file` generate from the typed module alone: the resolved AST's statement and function bodies are emptied before any backend runs, and there is no AST fallback. Removing the fallback showed what it had been hiding. `record_transpile` now writes back a parenthesized record variable. The typed C emitter gained hoisting for raising calls in `select case` clause values and loop guards (a `while`, or a `do` with a pre-condition, whose guard can raise becomes a `do` with a checked exit test at the top), scalar-method calls in nested `if`/`while`/`do` conditions (the method table now lives on the error context), and `for` bounds in the hoisted set. The AST emitters had produced precise diagnostics for invalid programs, so the typed C emitter now records a decline reason at the site (`c_decline`, read by the final "not supported" error): byref arguments that are not plain variables, unknown or ambiguous array bounds, `return` outside a function, out-of-range file channels, `GET`/`PUT` with no record number or no `FIELD`, `INPUT` with several variables, `LSET` on a variable never fielded, and `LINE INPUT #` into a non-string. `AstUse` is now `AstOnly` (the legacy baseline the gates compare against) and `TypedOnly`.

Iteration 5881 compared how each backend handles invalid programs, AST-driven against typed-only, over 56 hand-written cases. Typed-only generation had regressed four BASIC cases into silently accepting invalid programs (a `byref` argument that is not a variable, an unknown array in `lbound`/`sizeof`, `sizeof` of a multidimensional array with no axis, a call with the wrong number of arguments). The shared checks now live in the resolver's typed validation, ahead of every backend: call arity, `byref` arguments, non-numeric `for` variables, `swap` operand kinds, `mid$` targets, assignment to a `const`, string/number mixes in assignments, arithmetic and comparisons, and string array subscripts. BASIC's typed array-bound builtins report their resolution errors through a deferred-error list. `tests/fixtures/invalid/` is the new growing corpus: each `<name>.bcl` has a `<name>.expected` giving `ok` or a diagnostic substring per target, checked by `invalid_program_corpus_reports_the_expected_diagnostics`; `# gap:` comments mark weak spots (generic "not supported" messages, invalid programs nothing rejects yet).

Iteration 5882 added `resolver::reject_unknown_calls`, run by the driver for the C, C64 and JVM targets only: a call to a name that is not a user callable, a built-in, a declared array or array parameter, or a method is reported as `unknown function or array` before code generation. It is deliberately not part of the shared validation, because the BASIC targets pass an unrecognized function name through to the BASIC dialect and an existing test relies on that.

Iteration 5883 made calling a name that is neither a built-in nor a user-defined function an error on every target: `reject_unknown_calls` moved into the shared typed validation, and the BASIC pass-through of unrecognized function names is gone. Three tests that had relied on the pass-through were updated (`compiles_sort_driver_sample` now compiles through `compile_file` so its `require`s resolve; the mismatched-receiver test expects the error on BASIC and C; the C partial-output test uses `environ$`, a built-in C cannot render, instead of an unknown name). All 16 `adventure3000` stages still compile for BASIC.

Iteration 5884 added more shared pre-codegen checks to the resolver's typed validation: a `case` value must be the same kind as its `select case` expression, `line input #` needs a string target, `print using` needs a string format, a procedure cannot be used as a value, and `sizeof`/`lbound`/`ubound` need a known array with a valid axis (looked up in the callable's own dims and array parameters before the program's). The four BASIC codegen tests that exercised the generator's own array-bound diagnostics now assert on the resolver error, and six invalid-program corpus cases now expect one identical message from every target.
