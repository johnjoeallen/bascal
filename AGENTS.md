# BASCAL — Agent Instructions

## Audience and documentation style

BASCAL is a technical project: a compiler aimed at programmers who already
know a language like BASIC, Pascal, or C.

Documentation, commit messages, and code comments should use precise
technical/compiler terminology rather than plain-English paraphrases.

Use terms such as:

* lexer
* parser
* AST
* resolver
* typed IR
* symbol table
* scope
* shadowing
* diagnostic
* codegen backend
* transpile

Do not replace these with informal descriptions such as "the part that reads
the code", "turns it into", or similar lay terminology.

Prefer the exact term a compiler engineer would use. Do not soften or simplify
technical terminology for a general audience.

## Terminology

When describing how BASCAL source becomes generated BASIC, C, JVM, or other
target code — in documentation, commit messages, comments, or other prose —
use **"transpile"**, not "lower" or "lowering".

"Lower/lowering" is compiler-internals jargon; BASCAL documentation
consistently uses "transpile" instead.

Use **"typed IR"** for the resolved intermediate representation consumed
after name and type resolution.

Use **"codegen backend"** for a backend responsible for producing target code.

## Typed Intermediate Representation

After name and type resolution, every compiler stage must retain the full
compile-time type information required by subsequent transformations and
codegen backends.

Backends must consume this resolved typed IR.

They must not re-infer source types from:

* source syntax
* unresolved AST nodes
* identifier naming conventions
* generated text
* backend-specific transpiled output
* incidental properties of the target language

Type and semantic information required by later compiler stages must be
represented explicitly in the typed IR rather than reconstructed
opportunistically by a backend.

Changes to the resolver, type system, AST-to-IR transformation, or typed IR
must consider every downstream consumer and every affected codegen backend.

A backend must not compensate for missing semantic information that should
have been established by the resolver or typed-IR construction stage.

## Repository exploration

This repository contains an existing Graphify knowledge graph under
`graphify-out/`, and the Graphify CLI is installed.

There is no project-scoped Graphify skill exposed under `.agents/` or
`.codex/`.

Do not assume that `$graphify` or a project-specific Graphify agent skill is
available.

Use the Graphify CLI directly.

Graphify is intended primarily for non-local repository exploration,
including:

* architecture
* dependency analysis
* call paths
* symbol relationships
* change impact
* cross-cutting compiler changes
* changes spanning multiple codegen backends

Typical uses include tracing:

* lexer -> parser -> AST -> resolver -> typed IR -> codegen backend
* declaration and symbol resolution
* type propagation
* expression parsing and precedence
* function calls
* procedure calls
* method calls
* record and type relationships
* diagnostic generation
* behaviour shared by multiple backends

When `graphify-out/graph.json` already exists, prefer scoped Graphify queries
over recursively reading large parts of the repository.

Use:

```text
graphify query "<architectural or implementation question>"
```

for general architectural or implementation questions.

Use:

```text
graphify path "<symbol-or-concept-A>" "<symbol-or-concept-B>"
```

when tracing the relationship between two known symbols, components, or
concepts.

Use:

```text
graphify explain "<symbol-or-concept>"
```

for focused inspection of a symbol, subsystem, compiler stage, or concept.

Use Graphify to identify relevant:

* symbols
* files
* callers
* callees
* dependencies
* implementation paths
* affected compiler stages
* affected codegen backends
* tests

Then inspect the actual source before modifying it.

The source tree is authoritative.

Graphify is an architectural-navigation and impact-analysis aid. It is not a
substitute for reading the implementation.

If Graphify output appears incomplete, stale, or inconsistent with recently
changed source, refresh the graph with:

```text
graphify update .
```

and repeat the relevant query.

Refresh the graph before beginning a new cross-cutting or cross-backend slice
when repository changes may not yet be reflected in the existing graph.

After making changes that materially affect architecture, dependencies,
symbols, or cross-stage relationships, run:

```text
graphify update .
```

before performing further Graphify-based analysis.

Prefer incremental updates. Do not rebuild the graph from scratch when
`graphify update .` is sufficient.

Do not invoke Graphify unnecessarily for small, localised changes where the
relevant files and symbols are already known.

Read `graphify-out/GRAPH_REPORT.md` when a broad architecture overview is
useful or when scoped `query`, `path`, or `explain` operations do not provide
enough context.

## Change analysis

Before making a non-trivial compiler change:

1. Identify the compiler stages affected.
2. Trace the relevant implementation path.
3. Identify upstream producers and downstream consumers of any changed AST,
   symbol, type, or typed IR structure.
4. Identify all codegen backends affected by the change.
5. Identify existing tests covering the behaviour.
6. Determine whether new lexer, parser, resolver, type-system, typed-IR,
   diagnostic, backend, or integration tests are required.

For cross-cutting or cross-backend changes, use Graphify where it reduces
repository exploration or improves change-impact analysis.

Do not modify code until the actual implementation of the relevant path has
been inspected.

Graphify may identify the path; the source establishes the behaviour.

## Compiler-stage ownership

Fix defects in the compiler stage that owns the responsibility.

Do not work around an earlier-stage defect in a later stage.

In particular:

* do not compensate for lexer behaviour in the parser
* do not compensate for parser defects in the resolver
* do not compensate for parser or resolver defects in a backend
* do not resolve names during code generation
* do not re-infer resolved types in a backend
* do not reconstruct semantic information from generated target code
* do not encode semantic information implicitly in generated identifiers
* do not make one backend responsible for semantics that should be shared

If semantic information is required by multiple backends, establish it before
backend code generation and represent it explicitly in the typed IR.

## Implementation discipline

Make the smallest coherent change that correctly implements the requirement.

Avoid unrelated refactoring unless it is required to make the requested
change correct or maintain the compiler's existing invariants.

Do not mix opportunistic cleanup with the requested change merely because
nearby code is being edited.

Where a change alters an AST, symbol, type, or typed-IR contract, update all
producers and consumers as part of the same coherent change.

When changing shared compiler semantics, avoid backend-specific fixes unless
the behaviour is genuinely target-specific.

Preserve existing behaviour unless the requirement explicitly changes it.

## Cross-backend behaviour

Shared language semantics belong in shared compiler stages, not in individual
codegen backends.

When a feature affects multiple backends:

1. Establish or update the semantic representation in the appropriate shared
   compiler stage.
2. Ensure the typed IR contains everything required by each backend.
3. Update every affected backend.
4. Add or update backend-specific tests where emitted code differs.
5. Add shared semantic tests where behaviour should remain backend-independent.

Do not use one backend's implementation as the semantic definition for
another backend.

Generated BASIC, C, JVM bytecode, or any future target is output of the
compiler, not a source of semantic truth.

## Validation

For implementation changes, run the most specific relevant tests first.

Then run broader validation where practical.

Relevant test categories include:

* lexer tests
* parser tests
* precedence and associativity tests
* AST tests
* resolver tests
* symbol-table and scope tests
* type-system tests
* typed-IR tests
* diagnostic tests
* backend-specific codegen tests
* generated-code execution tests
* integration tests
* end-to-end compiler tests
* regression tests

When a change affects behaviour shared across codegen backends, verify every
affected backend rather than assuming behaviour observed in one backend
generalises to another.

Do not treat successful compilation alone as sufficient validation of a
semantic or code-generation change.

Where practical, validate both:

* the generated representation
* the runtime behaviour of generated code

For cross-cutting changes, run the full relevant test suite after focused
tests pass.

## Test quality

Prefer tests that demonstrate semantic behaviour rather than tests coupled
unnecessarily to implementation details.

For parser changes, test both accepted and rejected syntax where relevant.

For precedence or associativity changes, include cases that distinguish the
old and new parse trees.

For resolver or type-system changes, include:

* valid cases
* invalid cases
* expected diagnostics
* scope or shadowing interactions where relevant

For typed-IR changes, verify that required resolved type and semantic
information survives into downstream stages.

For backend changes, test the emitted result rather than only testing that
the backend completed without error.

Add regression tests for defects whenever a stable reproduction is possible.

## Diagnostics

Diagnostics are part of the language implementation.

When changing parser, resolver, or type-system behaviour:

* preserve useful source locations
* preserve or improve diagnostic specificity
* do not replace a precise diagnostic with a generic failure
* avoid backend failures for errors that should have been diagnosed earlier

Invalid BASCAL source should normally fail in the earliest compiler stage
that has enough semantic information to diagnose the problem correctly.

## Generated code

Generated target code should be deterministic where practical.

Do not rely on accidental iteration order, unstable collection ordering, or
environment-specific behaviour when emitting code.

Generated identifiers must avoid collisions with:

* user-defined symbols
* compiler-generated symbols
* target-language reserved identifiers

Do not encode information into generated names when that information belongs
in the typed IR.

Target-specific code-generation constraints should remain inside the relevant
codegen backend unless they affect language semantics.

## Refactoring

Do not perform broad refactoring as part of a narrowly scoped behavioural
change unless the refactoring is necessary to implement the change safely.

When refactoring:

* preserve compiler behaviour unless explicitly changing it
* preserve or improve test coverage
* avoid changing public or internal contracts unnecessarily
* update all affected backends
* keep semantic transformations in their owning compiler stages

If a refactor materially changes architecture or symbol relationships,
refresh Graphify afterwards with:

```text
graphify update .
```

## Git workflow

Keep changes logically scoped.

Commit useful checkpoints early enough that experimental or incorrect agent
changes can be rolled back cleanly to a known state.

Prefer multiple coherent commits over one large mixed commit when the work
naturally divides into independently understandable stages.

Do not combine unrelated fixes into the same commit merely because they were
discovered while working on the requested change.

Before committing:

1. inspect the diff
2. remove accidental or unrelated edits
3. confirm relevant tests pass
4. confirm generated or temporary files are not unintentionally included

Commit messages should use precise compiler terminology and describe the
actual semantic or architectural change.

## Graphify generated files

Graphify output under `graphify-out/` is generated analysis data.

Do not treat generated graph output as authoritative source.

Do not manually edit generated graph data to make it agree with the code.

Refresh it from the source with:

```text
graphify update .
```

Generated Graphify analysis output should normally remain untracked unless
there is an explicit project decision to version it.

If Graphify-related agent configuration is later added under `.agents/`,
`.codex/`, or another project configuration directory, follow that
configuration in addition to these instructions, provided it does not
contradict the source tree or this file.

## Agent behaviour

Do not guess about compiler architecture when it can be established from the
repository.

For non-trivial changes:

1. inspect the relevant architecture
2. use Graphify where useful
3. inspect the actual source
4. identify affected tests
5. implement the smallest coherent change
6. run focused validation
7. run broader validation where appropriate
8. inspect the diff
9. update Graphify when architectural relationships have changed
10. commit at a useful logical checkpoint

When uncertain whether behaviour belongs in the parser, resolver, typed IR,
or backend, trace the existing data flow before changing code.

Prefer preserving compiler-stage separation over introducing a local shortcut.

Do not declare a change complete merely because the immediate failing test
passes if the same semantic path is consumed by other stages or backends.

For cross-backend work, explicitly consider every backend before concluding
that the implementation is complete.

## Workflow orchestration

These sections are how work is organised; the compiler-specific rules above
(stage ownership, typed IR, cross-backend behaviour, validation, git workflow)
still apply to everything below.

### 1. Plan mode default

- Enter plan mode for ANY non-trivial task (3+ steps or architectural decisions)
- If something goes sideways, STOP and re-plan immediately - don't keep pushing
- Use plan mode for verification steps, not just building
- Write detailed specs upfront to reduce ambiguity

### 2. Subagent strategy

- Use subagents liberally to keep the main context window clean
- Offload research, exploration, and parallel analysis to subagents
- For complex problems, throw more compute at it via subagents
- One task per subagent for focused execution

### 3. Self-improvement loop

- After ANY correction from the user: update `tasks/lessons.md` with the pattern
- Write rules for yourself that prevent the same mistake
- Ruthlessly iterate on these lessons until the mistake rate drops
- Review lessons at session start for the relevant project

### 4. Verification before done

- Never mark a task complete without proving it works
- Diff behavior between main and your changes when relevant
- Ask yourself: "Would a Staff engineer approve this?"
- Run tests, check logs, demonstrate correctness (see Validation above for
  what that means for compiler changes)

### 5. Demand elegance (balanced)

- For non-trivial changes: pause and ask "is there a more elegant way?"
- If a fix feels hacky: "Knowing everything I know now, implement the elegant solution"
- Skip this for simple, obvious fixes - don't over-engineer
- Challenge your own work before presenting it

### 6. Autonomous bug fixing

- When given a bug report: just fix it. Don't ask for hand-holding
- Point at logs, errors, failing tests - then resolve them
- Zero context switching required from the user
- Go fix failing CI tests without being told how

## Task management

1. **Plan first**: write the plan to `tasks/todo.md` with checkable items
2. **Verify plan**: check in before starting implementation
3. **Track progress**: mark items complete as you go
4. **Explain changes**: high-level summary at each step
5. **Document results**: add a review section to `tasks/todo.md`
6. **Capture lessons**: update `tasks/lessons.md` after corrections

## Core principles

- **Simplicity first**: make every change as simple as possible. Impact minimal code.
- **No laziness**: find root causes. No temporary fixes. Senior developer standards.
- **Minimal impact**: changes should only touch what's necessary. Avoid introducing bugs.
