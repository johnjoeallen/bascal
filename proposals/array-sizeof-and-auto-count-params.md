# Proposal: compiler-tracked array sizes, `sizeof()`, and auto-injected count parameters

Status: everything in this file is **implemented** — `sizeof(x%)` /
`sizeof(x%, axis)` (freeze-at-DIM-time, multi-axis, and the on-a-parameter
resolution case), unconditional auto-injection of array bounds at call
sites, **array parameter storage capacity inference** (with a compile-time
and a runtime backstop check), **`byref`/`byval`**, **`global` shadowing**,
and **explicit array-parameter rank syntax** (`arr%(?)`, `grid%(?, ?)`,
bare-identifier call sites). See `MANUAL.md`/`docs/manual.html` "sizeof()",
"Multi-Dimensional Array Parameters", and "Array Parameter Storage
Capacity" for the shipped, documented form.

The manual-count-parameter convention this whole proposal set out to
replace is now gone entirely, not just optional: an array parameter's
signature carries no trailing count parameter, and a call site passes
nothing but the array itself. The compiler carries the array's bounds
alongside it automatically, in a hidden variable per axis that the
caller sets immediately before the call and the callee's own `sizeof()`
reads back. An earlier draft of this document briefly proposed shipping
`sizeof()` as an explicit, caller-written call-site argument instead
(`sumGrid%(g%, sizeof(g%, 0), sizeof(g%, 1))`) and framed unconditional
auto-injection as rejected -- that was a wrong turn, corrected before
release; see [Auto-injection at call sites](#auto-injection-at-call-sites)
below for the shipped design and why it doesn't reintroduce the
capacity-vs-fill-count problem it might look like at first.

## Motivation

*(Historical — describes the pre-`sizeof()` state of the compiler. `copy_bound`
no longer exists; see [Auto-injection at call sites](#auto-injection-at-call-sites)
for what replaced it.)*

Before this proposal, passing an array into a function/procedure required
the caller to manually pass an element count as the very next argument
after the array, because classic BASIC has no way to ask an array its own
length at runtime. The old mechanism (`call_lines`, `copy_bound`,
`array_copy_lines` in `src/codegen.rs`) took whatever argument came
immediately after the array argument at the call site and used its
*rendered text* as the bound for the generated `DIM`/copy-in/copy-out
loops.

Two problems with that old convention:

1. It was entirely manual and undocumented as a hard requirement — nothing
   stopped a caller from forgetting the count argument.
2. The fallback chain in `copy_bound` silently defaulted to the literal
   `10` if there was no argument in that slot and no matching parameter
   either — a silent-wrong-size footgun, not a compile error.

## Proposed direction

Track array bounds in the compiler's own symbol table at parse/codegen time,
so the compiler always knows an array's declared size and can:

- Expose it to source via a `sizeof(x%)` builtin.
- Auto-inject it as a hidden count argument at call sites, removing the
  manual-passing requirement (and the silent-10 footgun) entirely.

### What `DIM` already allows

Per `MANUAL.md`, `DIM` bounds may be **any integer expression, including a
constant** — not just literals:

```bascal
dim scores%(100)                  ' literal
dim matrix%(rows% - 1, cols% - 1) ' expression, up to 8 dimensions
```

So the size to track isn't always a compile-time constant — sometimes it's
an expression involving a variable.

### Freezing the size at DIM time

If the bound expression involves a mutable variable (`dim x%(a%)`), the
compiler cannot just re-emit `a%` later wherever a size is needed — `a%`
might be reassigned after the `DIM`, and BASIC's `DIM` fixes the array's
size at the moment it executes, not as a live binding. Re-emitting the bare
variable name later would silently return the *current* value of `a%`, not
the value the array was actually allocated with.

Fix: capture the bound expression's value into a compiler-generated,
never-reassigned temp variable right at the `DIM` site (e.g.
`BCC_DIM_x_0% = a%`), and have the symbol table point at that temp instead
of the original expression. Every later `sizeof(x%)` — and every
auto-injected call-site argument — resolves through the frozen temp, so it
can never drift from the size the array actually got.

This capture step is only needed when the bound isn't already a compile-time
constant. A literal or `const` bound needs no temp; the compiler can just
re-emit it directly forever.

The capture point is unambiguous because classic BASIC does not allow
re-`DIM`ing an already-dimensioned array (a second `DIM` on the same array
is a runtime `Duplicate Definition` error) — there is exactly one `DIM`
execution per array to hang the temp off of.

Since BASCAL doesn't support recursion, the temp can be an ordinary
uniquely-indexed generated global/local, the same treatment function locals
already get (`fname_var_0%`, …) — no per-call-frame storage needed.

### Multi-dimensional arrays: `sizeof(x%, axis)`

BASIC has no multi-value returns anywhere in the language — every
function/procedure call already collapses to one scalar via one generated
result variable. `sizeof()` shouldn't be the first thing to break that
model (no tuple returns, no destructuring assignment invented just for
this).

Instead, take the axis as a second argument, called once per dimension:

```bascal
dim grid%(rows% - 1, cols% - 1)

sizeof(grid%, 0)   ' rows% - 1, frozen at DIM time
sizeof(grid%, 1)   ' cols% - 1, frozen at DIM time
```

This mirrors how `DIM` itself already takes bounds as a positional,
comma-separated list — `sizeof(x%, axis)` just indexes into the same
per-axis structure `DIM` established. `axis` must be a compile-time-constant
integer (it selects *which frozen temp* to substitute, not a runtime
lookup), and the symbol-table entry per array becomes a small array of
per-axis frozen temps rather than a single value.

### Auto-injection at call sites

Shipped as originally scoped: auto-inject unconditionally. Whenever an
array is passed at a call site, the compiler resolves its bounds (from
its own frozen `DIM`, or — if it's itself a parameter being forwarded
onward — from what *its own* caller already carries for it) and assigns
them into one compiler-synthesized hidden variable per axis, immediately
before the `GOSUB`. Nothing about this is written by the `.bcl` author,
at the call site or in the callee's signature — `sumGrid%(g%)`, not
`sumGrid%(g%, sizeof(g%, 0), sizeof(g%, 1))` and not
`sumGrid%(g%, rows%, cols%)`. Inside the callee, `sizeof(grid%, 0)` on
one of its own array parameters just reads the hidden variable its
caller set.

This does **not** reintroduce the capacity-vs-logical-count problem it
might look like at first: the injected value only answers a mechanical
question — how many slots to copy in `DIM`/copy-in/copy-out so nothing
is truncated or reads garbage. A program that tracks its own "how much
of this buffer is actually filled" count (e.g. records loaded from a
file into a fixed-capacity array) keeps that as its own ordinary,
explicit, developer-owned parameter, same as today — it's answering a
different question than the injected capacity value, so the two coexist
without conflict. The old single `count%` convention conflated these two
numbers into one manually-passed value, which was arguably the worse
design; decoupling them, with capacity handled automatically and a
logical fill count (when a program even needs one) passed as its own
plain parameter, is what shipped. See [`sizeof()`](../MANUAL.md#sizeof)
and [Multi-Dimensional Array Parameters](../MANUAL.md#multi-dimensional-array-parameters)
in the manual for the shipped form.

### Resolved: multi-D array parameters are now fully wired up

This gap is closed — `call_lines` now infers each array parameter's rank
from how the callee's own body indexes it, resolves the caller's array's
declared rank, rejects a mismatch as a compile error, and (when they agree)
emits a proper `DIM` with one bound per axis and nested copy loops (one
`FOR` per dimension) for both copy-in and `byref` copy-out. No argument is
required at the call site beyond the array itself, at any rank — see
[Auto-injection at call sites](#auto-injection-at-call-sites) above; the
per-axis bounds are resolved and carried automatically, the same
mechanism regardless of rank.

Fixed alongside it: a pre-existing, unrelated codegen bug where any
2+-index array access (`grid%(r%, c%)`) parses as `Expr::Call`, not
`Expr::ArrayRef` (see `make_paren_ident_expr` in `parser.rs`), and the
`Expr::Call` fallback branch in `codegen.rs`'s `expr()` never resolved the
name through parameter/local scope the way the `Expr::ArrayRef` branch did
— so reading or writing *any* 2-D+ array (parameter or local) inside a
function body silently used the wrong, unmangled name. This wasn't a
multi-D-specific limitation so much as multi-D arrays never having been
exercised inside a function body at all before now.

## Resolved questions (kept for history)

- `sizeof()` is usable as a general source-level expression, not just an
  internal mechanism — any `.bcl` code can call it on a known array.
- Scoping turned out not to be a problem in practice: a frozen bound is
  always either a top-level global, a function-local temp visible for the
  rest of that function, or (for a forwarded array parameter) the hidden
  bound variable set by that function's own caller — all cases the
  existing `ident()`/scope-resolution machinery already handled.
- Auto-injection is unconditional, with no call-site override — see
  [Auto-injection at call sites](#auto-injection-at-call-sites) above.
- Full multi-D array parameter passing (nested copy loops, `DIM` with
  multiple dynamic bounds) shipped alongside `sizeof()` rather than as
  separate follow-up work — see
  [Resolved: multi-D array parameters](#resolved-multi-d-array-parameters-are-now-fully-wired-up)
  above.

---

## Implemented: array parameter storage capacity

### The bug this closes

Auto-injection (above) fixed *how big a copy loop should run* per call,
but left a separate, more basic problem completely unaddressed: *how big
the shared storage array itself is allowed to be*, for the whole life of
the program. `call_lines` DIMed a parameter's storage at every call site
that used it — meaning any function with an array parameter, called more
than once anywhere (two different call sites, or one call site inside a
loop), emitted the same `DIM` more than once. Classic BASIC has no
`REDIM`; a second `DIM` on an already-`DIM`ed array is a fatal runtime
"Duplicate Definition" error. This wasn't hypothetical — it was already
present in the shipped tutorials before this fix, e.g.
`tutorial/08_arrays.bcl` calling `printArray%` twice generated `DIM
printarray_arr_0%(...)` twice.

This predates `sizeof()` entirely — it was already true under the
original manual-count-parameter convention, since the `DIM` was always
emitted per call site regardless of where the bound came from.

### Decision: capacity is inferred, decided once, `DIM`ed once

An array parameter's storage now gets `DIM`ed exactly once, at the very
top of the generated program, before any call happens. Its size is a
fixed *capacity* — a new, separate number from the per-call *actual*
size `sizeof()`/auto-injection already tracks — resolved one of two ways:

- **Inferred** (`arr%(?)`, the default): the compiler scans every call
  site of the function across the whole program and takes the largest
  resolved size, *only* when every one of those sizes is itself resolvable
  at compile time — a literal `DIM` bound, a `const` (recursively
  evaluated through simple `+`/`-`/`*` arithmetic), or, when the array
  being passed is itself another function's array parameter forwarded
  onward, that parameter's own already-resolved capacity. Since BASCAL
  rejects every call cycle (direct or indirect), that forwarding chain is
  always finite, so resolving it is a straightforward fixed-point: repeat
  scanning until nothing new resolves, same shape as any other
  cycle-free dependency resolution.
- **Explicit** (`arr%(100)`, a literal in place of `?`): required only
  when inference genuinely can't produce a safe number — at least one
  call site's array size is a real runtime value (e.g. `dim data%(n%)`
  where `n%` came from `input`). There's no way to know that size ahead
  of time, so there's no way to auto-size storage for it; the author has
  to say how much to allow for.

A capacity that's provably too small — a call site whose size *is*
resolvable at compile time and exceeds the (inferred or explicit)
capacity — is a compile-time error, not a wait-and-see runtime one.

### Decision: a runtime check too, regardless

Every call site also emits a runtime check comparing the argument's
actual resolved size against the parameter's capacity, immediately after
setting the auto-injected bound variable and before the copy-in loop:

```basic
IF sumarr_arr_dim0_0% > 100 THEN PRINT "runtime error: ..." : STOP
```

This is deliberately unconditional, not gated on "only when the compiler
couldn't already prove it's safe." A call site the compiler *did* prove
safe can never trip this check (it's dead code for that call), but
emitting it anyway is cheap insurance against a mistake in the inference
itself, and it's the only defense at all for the explicit-capacity case,
where a compile-time-unprovable call is exactly the situation that led to
writing an explicit capacity in the first place.

### Implementation

- `Param.rank: Option<usize>` became `Param.axes: Option<Vec<Option<i64>>>`
  — `None` per axis is `?`, `Some(n)` is an explicit capacity written in
  its place. `Param::rank()` derives the old dimension-count value from
  `axes.len()` for the few call sites that only ever needed that.
- `infer_array_param_capacities` (`codegen.rs`) is a standalone,
  whole-program prepass over the parsed `Program` — it doesn't touch
  `CodeGenerator` state, so it runs before any `FunctionInfo` exists and
  its results (`HashMap<function name, Vec<Vec<i64>>>`) feed into
  `FunctionInfo::from_def` as a new `param_capacities` field, parallel to
  `params`.
- `const_eval` folds a small constant-expression language (integer
  literals, unambiguous `const` references, `+`/`-`/`*` between two
  foldable operands) — deliberately narrow; a genuinely dynamic bound is
  supposed to fall through to "unresolvable," not be guessed at.
- `emit_array_param_storage_dims` (`CodeGenerator`) emits the hoisted,
  one-time `DIM` block right after `COMMON`/dependency declarations and
  before the program's own top-level statements run.
- `call_lines` no longer DIMs a parameter's storage at all — it only sets
  the auto-injected bound variable, emits the runtime check against
  `info.param_capacities`, and runs the copy loop against storage that
  was already DIMed once, up top.

---

## Implemented: `byref` / `byval` parameter passing mode

Unrelated to `sizeof`, but surfaced from the same conversation while
digging into how array parameters actually copy today.

### Current behavior (confirmed against source, not assumed)

- **Arrays**: `call_lines` (`codegen.rs:1000-1064`) always does copy-in
  before the `GOSUB` and copy-out after, unconditionally, for every array
  argument — regardless of whether the function's own `return` value is
  used. Confirmed via all three call sites that reach `call_lines`
  (`codegen.rs:926`, `947`, `1072`) and via the existing
  `tutorial/08_arrays.bcl` example, where `insertionSort%` returns an
  unrelated `0` yet the array comes back sorted — the array copy-out and
  the function's scalar `return` are fully decoupled mechanisms.
- **Scalars**: copy-in only, always. There is no write-back path for a
  scalar parameter today — no mechanism exists for a function to hand a
  value back to the caller except its single `return` result.

### The problem

Copy-out for arrays being unconditional means:

- A caller has no way to pass an array **without** getting a (potentially
  large) copy-out it never wanted.
- There's no way for a *scalar* parameter to behave like a true output
  parameter — genuinely new capability, not present in any form today.

### Decision

Add a `byref` / `byval` qualifier, written before the parameter name in a
`function`/`procedure` declaration:

```bascal
function insertionSort%(byref arr%, count%)
    for i% = 1 to count% - 1
        ...
    end function
```

- **Unmarked parameter defaults to `byval`.**
- **`byval`** — copy the argument in before the call; nothing is written
  back after. This becomes the new default behavior for array parameters
  (today's always-copy-out goes away unless `byref` is written), and
  matches scalar parameters' existing (unchanged) behavior.
- **`byref`** — copy in before the call, copy back out after. For arrays,
  this is exactly today's existing (and only) behavior, now opt-in instead
  of automatic. For scalars, this is new: a genuine output parameter.
- **Applies uniformly to all parameter types** — scalar and array alike,
  same keyword, same semantics (copies back out or doesn't), not an
  array-only feature.

Naming rationale: reuses Visual Basic / QBasic's own `ByRef`/`ByVal`
vocabulary rather than inventing new syntax — already half-familiar to
BASIC-literate readers, and honest about the actual mechanism (it was never
true zero-copy reference semantics even in VB; it's "does the caller see
the mutation," which is precisely the copy-out-or-not switch this adds).
Considered but rejected: Pascal's `var` (thematically apt, given the origin
story's Turbo Pascal influence, but would read as a graft against otherwise
BASIC-flavored declaration syntax).

### Implementation notes

- Gate the array copy-out emission in `call_lines` on the parameter's mode
  instead of doing it unconditionally.
- For a `byref` scalar, the call-site argument must be a plain assignable
  variable (an lvalue) — `f(byref x%)` is fine, `f(byref x% + 1)` is not,
  and should be a compile-time error, same restriction every other
  language with ref/out parameters imposes.
- `FunctionDef.params` (`ast.rs:74`, currently `Vec<BasicIdent>`) needs to
  carry a mode per parameter, which then threads through
  `parse_ident_list` (`parser.rs:270`), `FunctionInfo.params`
  (`codegen.rs:64`, currently `Vec<(BasicIdent, BasicIdent)>`), and every
  place that iterates `.params` (`codegen.rs:168, 915, 1015, 1029, 1048,
  1094, 1191, 1650, 1965, 1980`).

## Implemented: reject `global` shadowed by a same-named parameter

### The bug

`ident()` (`codegen.rs:1090-1127`) checks `info.params` **before**
`info.globals`. So:

```bascal
function f(arr%, count%)
    global arr%    ' arr% is already a parameter name
    ...
end function
```

The parameter check wins first — every `arr%` inside `f` resolves to the
parameter's mangled storage, and the `global arr%` line is silently inert.
No validation pass catches this today; it's not a compile error, just a
no-op that looks like it should do something.

### Decision

Reject this at compile time. Same validation shape as the existing
`reject_duplicate_functions` check (`resolver.rs:8`, `resolver.rs:19`) —
walk each function/procedure body's `GlobalDecl` statements and raise a
diagnostic if the declared name matches one of that function's own
parameter names.

---

## Implemented: explicit array-parameter rank syntax

Surfaced from a documentation complaint: today, nothing in a parameter
declaration says a parameter is an array, let alone how many dimensions
it has — `arr%` in `function insertionSort%(byref arr%, count%)` looks
identical whether the compiler will later infer it as a scalar or a 5-D
array. Rank is entirely inferred from how the function's own body indexes
the parameter (implemented earlier this session — `infer_param_ranks` in
`codegen.rs`), which is invisible at the declaration itself and only
surfaces as a compile error if a caller gets it wrong.

### Decision: `(?, ?, ...)` — one `?` per dimension, in the declaration

```bascal
function insertionSort%(byref arr%(?), count%)
function sumGrid%(byref grid%(?, ?), rows%, cols%)
function sumCube%(byref cube%(?, ?, ?), a%, b%, c%)
```

- A scalar parameter stays a bare name, unchanged.
- An array parameter's rank is written directly at the declaration as a
  parenthesized, comma-separated list of `?` — one per dimension. No
  bounds go here (those still can't be known at the call site in general);
  `?` marks "a dimension exists here, sized by whoever calls this."
- `?` was picked over `*` (also considered) specifically because BASCAL
  already uses `?` for "deliberately incomplete, filled in elsewhere" in
  `?{ field: value }` partial record literals — same idea applied to a
  parameter's shape instead of a record's fields, reusing an existing
  convention instead of adding a second one. A bare rank count (`arr%(2)`)
  was also considered and reads more compactly at high rank, but was
  rejected because it makes a bare number mean two different things
  depending on context (a bound in `dim`, a rank count in a parameter) --
  `(?, ?)` stays visually self-counting, the same instinct as counting
  commas in `dim grid%(9, 9)`.

### Consequence: the call site no longer needs `()` for a known array

Once the declaration states a parameter's rank, the compiler doesn't need
a call-site marker to know argument position N expects an array — it can
tell from what the identifier resolves to:

```bascal
dim g%(2, 2)
print sumGrid%(g%, 3, 3)   ' g%, not g%() -- signature already says rank 2
```

`g%` already parses as plain `Expr::Ident` today; no parser change is
needed for the call site, only a codegen change in argument
classification (`call_lines` in `codegen.rs`): treat a bare `Expr::Ident`
as an array-pass whenever it resolves to a declared array *and* the
callee's corresponding parameter is declared with a matching rank, the
same way an empty-parens `Expr::ArrayRef` is treated today.

Edge case: classic BASIC allows a scalar and an array to share one name,
disambiguated only by whether parens appear at each use site (this is why
`Statement::Dim` tracks "were parens written at all" separately from its
bound list already). A bare `g%` argument is ambiguous if `g%` is *both*
a declared scalar and a declared array in the same scope. Rather than
preserve `()` everywhere to cover that case, the better fix is probably to
reject a name reused as both a scalar and an array in the same scope as
its own error — that's a footgun worth flagging regardless of this
change, not a legitimate pattern worth keeping `()` around to support.

### Relationship to the already-implemented rank inference

`infer_param_ranks` (rank inferred from body usage, built earlier this
session) doesn't go away — it becomes a **cross-check** instead of the
only source of truth: a parameter declared `arr%(?, ?)` but only ever
indexed with one subscript in the body is now a *declaration-vs-usage*
mismatch, catchable with the same kind of diagnostic already used for
*caller-vs-parameter* rank mismatches (`call_lines`'s existing
`array_ranks` check). Two independent signals agreeing is strictly safer
than inferring from one.

### Known cost: breaking change to every existing array-parameter example

This changes required syntax for every array parameter that exists today.
`tutorial/**/*.bcl`, `MANUAL.md`, and `docs/manual.html` all currently
declare array parameters as a bare name (`byref arr%`, `byref data%`,
`byref grid%`, …) — every one of those becomes `byref arr%(?)` /
`byref data%(?)` / `byref grid%(?, ?)`, on top of the parameter-comment
sweep already done for the same declarations. Not a reason not to do it,
but real follow-up work, not just a codegen change.
