# Analysis: `bcc` front-end & pipeline architecture

Status: **analysis + refactor proposal**. Suggestions 1, 3, and 5 are
implemented, and suggestion 2 is partly implemented (see "Refactor suggestions"
below); the rest is unimplemented. Describes
the pipeline as it stands at the time of writing and argues for a cleaner
lexer → parser → AST → lowering → resolve → emit separation, motivated by two
already-shipped bug classes (FIELD-buffer re-namespacing, label substitution in
comments/strings) that a resolved typed IR would have made unrepresentable.

## Pipeline as implemented

```
.bcl source (one or more files, via require / import)
   │  lib.rs: load_program_recursive / search_roots
   ▼
Lexer  (src/lexer.rs, hand-written)          ──►  Vec<Token>   (flat, per file)
   │      no keyword classification; comments kept as tokens
   ▼
Parser (src/parser.rs, hand-written recursive descent + Pratt expressions)
   │      soft-keyword dispatch on Ident text; rejects raw line numbers
   ▼
AST    (src/ast.rs: Program { statements, functions, records, ... })
   │
   ├─► lower::lower(program) -> Lowered   (src/lower.rs — ordered post-parse AST→AST sub-passes)
   │      1. records::lower   record/file DSL desugar → FIELD/GET/PUT + buffer vars
   │      2. inject_mid_assign_helper_if_used   appends the midAssign stdlib FunctionDef when used
   │      └─ Lowered { program, synthesized_buffer_names }   (named, not a bare tuple)
   ├─► lib.rs populates Program.common   (IO: needs the `shared` file + CompileOptions; runs before lower)
   │
   ▼
resolver::validate(&Program) -> Result<(), Vec<Diagnostic>>   ← pure checker, returns (), produces no IR
   │
   ▼
CodeGenerator (src/codegen_basic.rs) — walks the AST directly, re-deriving every fact it needs
   │      accumulates into  self.output: String   (code lines + bare `label:` sentinel lines, unnumbered)
   ▼
number_basic_lines(&self.output) — TEXT post-pass on the string:
   │      assign line numbers (step 10) → build label→number map → whole-word text
   │      substitution of label names on every non-comment line → drop label lines
   ▼
final line-numbered Microsoft BASIC text
```

`--target C` (`codegen_c.rs`) and `--target jvm` (`codegen_jvm.rs`) branch at the
`CodeGenerator` step and consume the **same raw AST** the same way.

## 1. Lexer

**Hand-written.** `src/lexer.rs` is a ~500-line char-by-char scanner (`Vec<char>`
+ index/line/column). The only crate dependency in the project is `clap` — no
`logos` / `pest` / `lalrpop`.

**The token stream is raw.** `TokenKind` has `Ident(String)`, `Number`, `Float`,
`String`, `Comment`, `BlockComment`, `HexLit`, and one variant per punctuation
mark. **There is no keyword token** — `print`, `goto`, `record`, `end`, `data`,
`field` all arrive as `TokenKind::Ident("...")`. Keyword recognition is entirely
the parser's job (`check_keyword("print")` string-compares the ident).

BASIC quirks, handled at lex time only partially:

- **Type-suffix identifiers** — `ident()` folds a trailing `% $ ! # &` *into* the
  identifier string, so `count%` is a single `Ident("count%")`; `&` is
  disambiguated from `&&` / `&H` by lookahead. The suffix is re-split later by
  `BasicIdent::parse`.
- **Free-form, not line-oriented** — BASCAL is Pascal-shaped; `\n` is a `Newline`
  token and the parser is newline-significant. No column rules.
- **No raw line numbers** — a leading integer is just `Number`; the lexer and the
  language have no line-number concept.
- **Comments** — `'`, `//`, `/* */` all produce tokens *carrying their text*
  (not discarded) so the backend can re-emit them.
- **String literals** — `"..."` read verbatim to the next `"`; **no escape or
  `""`-doubling handling**.
- **Dotted identifiers** — `s.id`, `com.bascal.sort.bubble` lex as one `Ident`; a
  *standalone* `.` (e.g. after `]`) is a `Dot`. A lexer-level hack that pushes
  `a.b` ambiguity downstream: `s.id` and `s . id` tokenize differently.

## 2. Parser

**Hand-written recursive descent**, one `parse_*` method per construct, dispatched
from `parse_statement_kind` by a ~50-arm `check_keyword` if-chain. Expressions use
a **Pratt / precedence-climbing** parser (`parse_expr(min_bp: u8)`). No generator
or combinator library.

**It builds a single AST with almost no semantic action.** No code is emitted, no
labels resolved, no line numbers assigned during parsing. Two small exceptions:
(a) `pending_statements: VecDeque<Stmt>` lets one source form expand to N
statements at parse time (`dim x% = 5`, single-line `if`); (b) the parser tracks
`const_types` so a later suffix-less reference to a `const` picks up its declared
type. Neither touches codegen or resolution.

BASIC constructs:

- **Labels vs line numbers** — `name:` → `Statement::Label(String)`. `goto` /
  `gosub` / `on...goto` / `resume` / `restore` targets are parsed as expressions
  then *constrained* (`parse_label_target`) to `Expr::Ident` only; a numeric
  target is a **hard parse error** ("BASCAL manages line numbers itself"),
  `on error goto 0` excepted.
- **DATA/READ** — `Statement::Data(Vec<Expr>)` / `Read(Vec<Expr>)`, parsed
  straight, no cross-linking; DATA-pointer wiring is a backend concern.
- **OPEN/FIELD/GET/PUT** — each has a dedicated `parse_*` and a dedicated
  `Statement` variant (`Open`, `Field`, `Get`, `Put`, `Lset`, `Rset`, `Seek`).
  Raw hand-written `FIELD` is preserved as-is.
- **`record ... end record` + file DSL** — `parse_record_def` (program level
  only), `parse_file_decl` (`file v as T = open(...)`), plus expression sugar
  `Expr::FileIndex` (`db[i]`), `FieldAccess` (`s.field`), `MethodCall`
  (`db.close()`), `RecordLit` (`{ field: value }`, `?{...}` partial). All parsed
  as first-class AST nodes and left for `records::lower` to eliminate.

## 3. Intermediate representation(s)

**There is one real IR: the AST**, and it is **mutated in place** rather than
transformed into successive typed forms.

- `src/ast.rs` defines a proper AST (`Program`, `Stmt` = `Statement` +
  `SourcePos`, `Expr`, `FunctionDef`, `RecordDef`, ...), fully separate from
  parser state.
- **`records::lower(program) -> (Program, HashSet<String>)`** is a genuine
  AST → AST pass: it desugars the record/file DSL into `FIELD` + `GET` / `PUT` +
  `MKx$` / `CVx$` + synthesized buffer variables. The closest thing to a lowering
  stage.
- Several `Program` fields are **populated after parsing by `lib.rs`**, not by
  the parser: `common` (built from a `shared` file's `dim`s — there is no
  `common` keyword), `typed_arrays` / `typed_array_refs`. The AST is a mutable
  bag that accretes data across passes.
- **`resolver::validate(&Program) -> Result<(), Vec<Diagnostic>>` returns `()`.**
  It is a pure checker. It computes name resolution, ranks, strict-var
  enforcement, error-handler reachability proofs, etc. — **and throws all of it
  away.** No annotated / typed IR reaches the backends. *(Partly addressed
  Sept 2026 — suggestion 2: `resolver::resolve` now returns a `ResolvedProgram`
  carrying the whole-program facts codegen_basic used to recompute per
  `generate()` call.)*
- Consequently **each backend re-derives everything** from the bare AST via its
  own `collect_*` scans: `collect_program_names`, `collect_record_buffer_names`,
  `collect_consts`, `infer_array_param_capacities`, `dim_ranks_in_body`,
  `collect_call_sites`, `error_handler_targets`, ... The `CodeGenerator` struct is
  largely a cache of these re-derived facts. *(Sept 2026: `record_buffer_names`,
  `const_names`, `top_level_array_ranks`, `error_handler_procedures`, and the
  `catch`-source-var flag now come pre-computed on `ResolvedProgram`;
  `collect_program_names` / `infer_array_param_capacities` still run in
  `generate()` — the first seeds a set that is then mutated, the second emits
  diagnostics.)* This contradicts `AGENTS.md`'s
  "Typed Intermediate Representation ... backends must consume this resolved
  typed IR; they must not re-infer" — the note reads as an aspiration the code
  has not met.

### The MID$ mechanism is not a two-pass placeholder/patch scheme

What actually happens: `lib::inject_mid_assign_helper_if_used` scans the AST for
the `MID$(...) = ...` statement form and, if present anywhere, parses
`com/bascal/stdlib/midAssign.bcl` and **appends its `FunctionDef` to
`program.functions`**. Codegen then emits an ordinary GOSUB call to it
(`call_lines_from_rendered_scalars`, fed pre-rendered argument text so `target`
isn't evaluated twice). No placeholders, no patch pass, no usage counts.

The "patch once counts are known" pattern *does* exist, but for a different
feature: `infer_array_param_capacities` walks `collect_call_sites` to size
`?`-declared array parameters from the largest array passed across all call
sites (`placeholders = vec!["?"; used]`). That is a codegen-time whole-program
analysis, still not a separate IR pass.

## 4. Code emission / backend

**Label → line-number resolution is a dedicated pass — but a *text* pass, not a
structural one.** The backend walks the AST and appends to
`CodeGenerator.output: String`. Control-flow labels (`WHILE_0001_TOP:`) and user
labels (`Statement::Label`) are written as literal `name:` lines into that
string. Then **`number_basic_lines(&self.output)`**:

1. `str::lines()` the accumulated string;
2. classifies each line via `is_label_line` (`word:` where `word` is
   `[A-Za-z0-9_]+`);
3. assigns numbers (step 10) to target lines (all lines, or only branch-target
   lines in the default sparse mode);
4. builds `label_numbers: HashMap<String, usize>`;
5. for every non-comment output line, calls `replace_label_word(text, label,
   number)` — a **whole-word string substitution** that skips `"..."` spans;
6. drops the `label:` lines, folds blank runs.

When `catch err%, erl%, source$` is used, `number_basic_lines` runs an **extra
throwaway pass** first (number everything, learn line-range → file mapping,
discard the text, append a lookup subroutine, number again).

**Global symbol handling is resolved lazily during emission, with no shared
symbol table.** BASIC has one flat namespace and no locals, so the backend
invents per-function BASIC names for params/results/locals on the fly. State
lives in `CodeGenerator`: `taken_names: RefCell<HashSet<String>>`,
`record_buffer_names` / `synthesized_buffer_names` / `const_names` (name sets that
must *not* be per-function-localized), `top_level_array_ranks` /
`top_level_array_bounds`, `known_callables`. Most are recomputed at the top of
`generate()` by scanning the AST. `synthesized_buffer_names` is the one fact
threaded forward as data from `records::lower`; its superset
`record_buffer_names` is re-scanned from the AST anyway.

## 5. Pipeline shape overall & the two known bugs

Conflated stages:

- **Resolution and codegen are effectively fused.** The resolver validates but
  emits nothing usable; the backend re-runs the same analyses. "Two passes" that
  are really "one checker + one re-analyzer."
- **Codegen and label linking are split by a stringly-typed boundary.** The
  backend's output is a `String`; label resolution is regex/substring surgery on
  that string. There is no "emitted instruction list" data structure between the
  AST walk and the final text.
- **Lowering is scattered.** `records::lower` (a real pass),
  `inject_mid_assign_helper_if_used` (an ad-hoc AST mutation), and
  `Program.common` / `typed_arrays` population (in `lib.rs`) are three different
  places that transform the AST post-parse, with no unifying "lowering" phase.
  *(Resolved Sept 2026 — suggestion 3. `src/lower.rs` is now that phase;
  `Program.common` load stays in the driver as IO, `typed_arrays` turned out to
  be parser-populated.)*
- **Three backends independently re-derive semantics** from the AST instead of
  consuming resolver output.

The two known bug classes — **both are emission-layer failures, both from the
same root cause:**

| Bug | Regression test | Stage that let it through | Why |
|---|---|---|---|
| Label substitution corrupting comments / string text | `label_name_matching_string_literal_text_is_not_corrupted` (lib.rs); guard at codegen_basic.rs:4176 | Emission layer — `number_basic_lines` | Label references are resolved by *whole-word text replacement on rendered lines*, not by resolving a typed "label reference" node. A user label named `done` / `loop` collides with the same word in `PRINT "...done..."` or a comment. Fixed twice by plaster: `replace_label_word` skips `"..."` spans, and line 4176 skips lines starting with `'`. Both patches exist *because* resolution is textual. **Resolved Sept 2026 (suggestion 1/5): user-label references are sentinel-wrapped and resolved structurally — a string or comment can no longer be a candidate.** |
| FIELD buffer variables re-namespaced per procedure | `FIELD buffer names must never be re-namespaced per procedure` (lib.rs) | Emission layer — `CodeGenerator::ident` name-mangling | The per-function local-name allocator didn't know a name was a `records::lower`-created FIELD buffer (which must stay one global name so the FIELD binding holds program-wide). The fact was established by lowering but not carried as a typed property — the backend re-discovers it via `collect_record_buffer_names(program)` scanning for `Statement::Field`, and the fix is a `HashSet` exclusion check inside `ident`. |

Neither bug is a lexer or parser bug. Both are *"the backend re-derived a fact by
scanning text / AST and got it wrong"* — exactly the class of bug a resolved
typed IR is supposed to make unrepresentable.

## Refactor suggestions

Ordered by payoff for isolating the bug classes above.

1. **Make `number_basic_lines` operate on a typed line list, not a `String`.**
   Have the backend emit `Vec<EmittedLine>` where a line is
   `{ segments: Vec<Segment>, kind }` and `Segment` is
   `Text(String) | LabelRef(LabelId) | Comment(String) | StringLit(String)`.
   Line numbering resolves `LabelRef` structurally — it is *impossible* for a
   comment or string to be a substitution target. Kills the label-in-comment bug
   class outright (delete `replace_label_word`'s string-skipping and the
   `starts_with('\'')` guard). Small, self-contained, high value.

   **Implemented (partial), Sept 2026.** Rather than a full `Vec<Segment>`
   rewrite of every emission site, `codegen_basic` now wraps every *user*-label
   reference (`Statement::Label` targets of `GOTO` / `GOSUB` / `ON ERROR GOTO` /
   `RESUME` / `RESTORE` / `ON ... GOTO`) in `\x01`/`\x02` sentinel delimiters at
   both the declaration line and each reference (`user_label_token`).
   `number_basic_lines` resolves those by exact structural replacement — a
   comment or string literal can never be a candidate. Transpiler-internal
   control-flow labels (`WHILE_0001_TOP`, `FN_foo`, …) keep the whole-word
   `replace_label_word` path: their distinctive prefixes make collision a
   non-issue, and the `starts_with('\'')` guard is retained as cheap insurance
   for the `FN_<user-name>` case. This kills the user-label half of the bug
   class — the half that ever actually bit — without touching the thousands of
   non-label `self.line` call sites. A later full segment-list pass can subsume
   it.

2. **Have `resolver` return a `ResolvedProgram`, and make backends consume it.**
   Even a thin wrapper — `ResolvedProgram { ast, name_binding, array_ranks,
   buffer_vars, const_names, callables, param_capacities }` — computed once.
   Delete the ~8 `collect_*` re-scans duplicated across
   `codegen_basic` / `codegen_c` / `codegen_jvm`. "Is this a global buffer var"
   becomes a field lookup, not an AST re-scan that can miss a case. This is the
   change `AGENTS.md` already asks for.

   **Partly implemented, Sept 2026.** `resolver::resolve(Program) -> Result<
   ResolvedProgram, _>` runs `validate` then computes, once, the whole-program
   facts `codegen_basic` previously rebuilt at the top of every `generate()`:
   `record_buffer_names` (the field the "FIELD buffer re-namespaced per
   procedure" bug got wrong — now a struct field, not a re-scan),
   `const_names`, `top_level_array_ranks`, `error_handler_procedures`, and the
   `catch`-source-var flag. `CodeGenerator::generate` takes `&ResolvedProgram`.
   The pure AST scans that feed these still physically live in `codegen_basic`
   (they have other callers there) and are called by `resolve`; a later move
   into a dedicated analysis module is possible but not required for the
   payoff. Not done: `collect_program_names` (seeds a set `generate` then
   mutates), `infer_array_param_capacities` (emits diagnostics, deeply
   call-site-specific), and wiring `codegen_c` / `codegen_jvm` — which have
   their *own* separate analysis, not copies of `codegen_basic`'s, so the
   "duplicated across three backends" framing overstated the overlap.

3. **Introduce one explicit lowering phase.** Fold `records::lower`,
   `inject_mid_assign_helper_if_used`, and the `lib.rs` `common` / `typed_array`
   population into a single `lower(Program) -> Program` (or `-> LoweredProgram`)
   with an ordered list of sub-passes. A named phase also gives the
   `synthesized_buffer_names` side channel a home instead of a tuple return.

   **Implemented, Sept 2026.** New `src/lower.rs` exposes
   `lower(Program) -> Result<Lowered, _>` running the ordered sub-passes
   (`records::lower`, then `inject_mid_assign_helper_if_used`, both moved here).
   `Lowered { program, synthesized_buffer_names }` replaces `records::lower`'s
   bare tuple return, giving that side channel a named home. Both
   `compile_source` and `compile_file` now call the single `lower::lower`.
   Two things stayed out: `typed_arrays` / `typed_array_refs` are populated by
   the *parser* (`parser.rs`), not post-parse in `lib.rs` — the original note
   was inaccurate — so there is nothing to fold; and `Program.common` needs
   filesystem + `CompileOptions` context to locate the `shared` file, so it
   stays in the IO-aware driver and runs just before `lower`.

4. **Name allocation as a pass, not a lazy `RefCell` side effect.**
   `taken_names: RefCell<HashSet<String>>` mutated during the emission recursion
   is where the re-namespacing bug lived. Resolve every BASCAL identifier to its
   final BASIC name in a dedicated pass over the resolved IR (producing a
   `HashMap<VarId, BasicName>`), *before* emission. Emission becomes a pure
   function of the IR; buffer vars, consts, and locals get their final names in
   one place with one rule set.

5. **Give labels IDs at parse/lower time.** `Statement::Label(String)` and
   `Goto(Expr::Ident)` both carry raw strings the backend re-matches by name.
   Assign each label a `LabelId` during resolution and make
   `Goto` / `Gosub` / `OnErrorGoto` / `Resume` carry `LabelId`. Then (1) has
   nothing to string-match at all.

   **Implemented, Sept 2026 — as a codegen-local token rather than an AST
   `LabelId`.** BASIC has one flat label namespace and the resolver already
   rejects duplicate labels, so a label *name* is already a unique identifier;
   threading a numeric `LabelId` through the AST and all three backends (only
   `codegen_basic` does text matching — `codegen_c` / `codegen_jvm` emit real
   labels) bought little. Instead the name is carried inside the `\x01`/`\x02`
   sentinel described under (1), which gives the reference the same "structural,
   not textual" property a `LabelId` would. If the AST ever grows a `LabelId`
   for other reasons, the sentinel payload becomes that id.

6. **Split `lib.rs` (~8 300 lines).** It is the orchestrator *and*
   `require`-resolution *and* the bulk of the test suite. Move `compile_source` /
   `compile_file` / `search_roots` / `load_program_recursive` into a `driver.rs`;
   the pipeline shape should be readable in ~100 lines.

7. **Lex keywords, or at least intern them.** The `check_keyword` if-chain in
   `parse_statement_kind` re-does a case-insensitive string compare per candidate
   per statement. A `Keyword(Kw)` token (soft keywords still possible via "was
   this `Ident` or `Keyword` in this position") makes dispatch a `match` and
   removes a class of "forgot to handle this keyword here" gaps.

8. **Minor:** handle `""` in string literals in the lexer, and stop consuming `.`
   inside `ident()` — lex `.` always as `Dot` and let the parser build
   `FieldAccess`. The current dotted-ident hack means `s.id` and `s . id`
   tokenize differently and member access only works in some positions.
