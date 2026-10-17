# BASCAL semantic frontend plan

`bascal.bcl.rdg` is concrete syntax.  Its generated parser must eventually
produce a semantic, source-span-aware BASCAL AST suitable for interpretation
and transpilation to BASIC, C, and JVM assembly.  It must not expose the
grammar's recursive-descent implementation strategy to downstream consumers.

This document is the contract for that migration.  It deliberately does not
add name resolution, type checking, method dispatch, or target code
generation to rdgen.

## Current inventory

The current Rust emitter still generates a mixed AST, but BASCAL expressions
now use one shared `Expr` output type. The first migration step removes the
public precedence-layer enums; expression constructors still need a later
normalization pass to become the final `Unary`/`Binary`/`Call` shape.

| Current category | Examples | Classification | Migration |
| --- | --- | --- | --- |
| File and declaration wrappers | `Program::File`, `FileItem`, `RecordDecl`, `FunctionDecl` | Mostly semantic, but wrapped by grammar rules | Retain their language concepts; remove routing-only wrappers. |
| Statement wrappers | `Statement`, `StatementCore`, `TopLevelStatement` | Parser artifacts | Emit one semantic statement enum and parser-owned statement-list logic. |
| Expression layers | `XorExpr`, `OrExpr`, `AddExpr`, `PostfixExpr`, `Primary` are now hidden; `Expr` contains their current constructors | Transitional semantic output | Fold operator repetitions and rename remaining parser-shaped constructors to the target `Unary`/`Binary`/`Call`/`Member`/`Index` nodes. |
| Lexical rules | `Identifier::Token`, `IntegerLiteral::Token`, comments | Provisional concrete payloads | Retain spelling and add source spans; convert only where a semantic literal value is required. |
| Named operators and modes | `AddOp`, `OpenMode`, `BranchKind` | Mixed | Operator and mode are semantic enums; their parser-rule wrappers are not. |
| Default alternatives | `AltN` | Provisional | Replace with explicit semantic constructors or reject them for the semantic frontend. |
| Repetition and optional tuples | `Option<(Token, Box<...>)>`, `Vec<(Token, Box<...>)>` | Parser artifacts | Convert to named semantic fields and ordinary lists. |

The current generated `Token(pub String)` preserves lexical spelling but has
no source span.  `rdgen_ir::Span` describes positions in the grammar DSL, not
positions in BASCAL source.  Parser diagnostics currently carry a byte offset,
but parsed nodes do not.

## Target public AST

The generated semantic frontend will expose a BASCAL-specific public AST with
a byte-range `SourceSpan` on every node.  The range covers the complete source
construct, including enclosing delimiters where they are part of the construct.
Operators retain their own span as well as the enclosing expression span.

```text
SourceSpan { start: usize, end: usize }
Spanned<T> { span: SourceSpan, value: T }

Program { items: Vec<FileItem>, span }
FileItem = ProgramDecl | LibraryDecl | SharedDecl | Require | Import
         | Record | Subprogram | Statement | Comment

Declaration = Record | Function | Procedure | Method | InlineMethod
Record { name, combined_records, fields, methods, span }
Field { name, type_ref, span }
Subprogram { kind, name, receiver, parameters, result, body, span }
Parameter { passing, name, axes, default, type_annotation, span }
TypeRef = Scalar | Record

Statement = Label | Comment | Declaration | Assignment | MidAssignment
          | Print | FileIo | Return | If | For | While | Do | SelectCase
          | Try | Transfer | Runtime | Expression

Expr = Name | IntegerLiteral | FloatLiteral | StringLiteral | TextBlock
     | Boolean | Unary | Binary | Call | Member | Index | RecordLiteral
```

The exact Rust type spelling remains a backend concern.  The semantic shape
does not: another emitter must be able to represent the same model without a
shared runtime tree.

## Syntax-directed normalization

The parser or generated semantic-construction layer performs only local,
syntax-directed normalization:

* `for x = first downto last` constructs `For { step: IntegerLiteral(-1) }`.
* `target += value` constructs ordinary `Assignment` with a `Binary` RHS;
  the target is cloned as the left operand only when that is semantically safe
  for BASCAL's lvalue model.  Otherwise the semantic AST records an explicit
  compound-assignment normalization marker for a later frontend pass.
* `let` has no semantic node.
* `true` and `false` use BASCAL's selected runtime representation consistently
  while retaining their source spans.
* Colon-separated compact bodies construct ordinary statement lists.  The
  enclosing body span covers the full compact range; each statement retains
  its own range.

Normalization must be visible in the IR or generated AST contract.  A backend
consumer must never infer it from `AltN`, a precedence rule name, or a tuple
that happens to contain a literal token.

## Line-oriented parsing contract

Statement-list parsing is an outer parser concern.  A physical newline and a
generic `:` are statement terminators; a label's defining colon is consumed by
the label itself.  Single-line `if` selection depends on a physical newline
after `then`, not merely on trivia availability.  Block headers, bodies, and
terminators must keep that physical-line distinction.

The semantic AST never exposes terminator tokens, recursive
`statement ':' statement` structure, or parser-only same-line wrappers.

## Validation boundary

Syntactic recognition remains permissive where BASCAL needs contextual
validation.  The semantic frontend records enough context for a subsequent
validation pass to reject:

* invalid `&&` or `||` placement;
* duplicate combined-record members;
* `byref` parameters with default values;
* invalid method receiver or inline-record-method context; and
* array-reference versus call choices requiring later semantic knowledge.

These are diagnostics over the semantic AST, not grammar backtracking rules
and not target-backend heuristics.

## Migration order

1. Add generated runtime `SourceSpan` and ensure every lexical match and
   semantic constructor can report an enclosing range.
2. Extend the shared IR/grammar annotations with semantic constructor and
   normalization metadata, retaining language neutrality.
3. Replace precedence-layer output with direct `Expr::Unary` and
   `Expr::Binary` construction, including operator spans.
4. Move colon/newline handling into generated statement-list parsing.
5. Convert declarations, parameters, statements, control flow, and comments
   to the target semantic concepts.
6. Add semantic snapshots for representative BASCAL programs and preserve the
   full corpus-recognition probe.
7. Make unsupported semantic output explicit at grammar-compilation time;
   semantic BASCAL generation must not silently expose `AltN` variants.

The IR must remain independent of SGDL.  SGDL may later lower into the same
semantic-constructor and normalization contract, but rdgen does not assume an
EBNF-only semantic AST model.
