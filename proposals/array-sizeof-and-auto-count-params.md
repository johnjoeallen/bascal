# Proposal: compiler-tracked array sizes, `sizeof()`, and auto-injected count parameters

Status: `sizeof(x%)` / `sizeof(x%, axis)` is **implemented** — freeze-at-DIM-time,
multi-axis, and the on-a-parameter resolution case are all shipped and
documented (`MANUAL.md`/`docs/manual.html` "sizeof()"). Everything else in
this file — **`byref`/`byval`**, **`global` shadowing**, and **explicit
array-parameter rank syntax** (`arr%(?)`, `grid%(?, ?)`, bare-identifier
call sites) — is also **implemented**. The one piece that shipped
*differently* than originally designed below is auto-injection: BASCAL
never silently fills in a count argument the caller didn't write — see
[Decided against: unconditional auto-injection](#decided-against-unconditional-auto-injection)
near the end for why, and what shipped instead (explicit `sizeof()` calls,
the caller writes them, same as any other argument).

## Motivation

Today, passing an array into a function/procedure requires the caller to
manually pass an element count as the very next argument after the array,
because classic BASIC has no way to ask an array its own length at runtime.
See `src/codegen.rs` — `call_lines`, `copy_bound` (~line 1645),
`array_copy_lines` (~line 1657) — for the current mechanism: the compiler
takes whatever argument comes immediately after the array argument at the
call site and uses its *rendered text* as the bound for the generated
`DIM`/copy-in/copy-out loops.

Two problems with the current convention:

1. It's entirely manual and undocumented as a hard requirement — nothing
   stops a caller from forgetting the count argument.
2. The fallback chain in `copy_bound` silently defaults to the literal `10`
   if there's no argument in that slot and no matching parameter either —
   a silent-wrong-size footgun, not a compile error.

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

### Decided against: unconditional auto-injection

The original idea here was to auto-inject unconditionally: whenever an
array is passed at a call site, the compiler supplies the frozen size
(one hidden argument per axis) itself, with nothing written at the call
site at all. That's not what shipped.

What shipped instead: `sizeof(x%)` / `sizeof(x%, axis)` as an ordinary,
explicit builtin the caller writes themselves —
`sumGrid%(g%, sizeof(g%, 0), sizeof(g%, 1))`, not `sumGrid%(g%)` with the
counts appearing from nowhere. The reasoning for staying explicit: a
BASCAL function's array-parameter contract already has two independent
numbers in play in real programs — the array's declared *capacity*
(what `sizeof` reads) and, separately, a caller's own tracked *logical
fill count* (e.g. records actually loaded from a file into a
fixed-capacity buffer, which is very often smaller than the array's
`DIM`). Those two aren't the same number, and a caller has to be able to
pass either one. Auto-injection would have hard-coded "always pass
capacity" into every call, silently overriding a caller who actually
meant to pass their own smaller fill count. `sizeof()` as an ordinary,
visible argument — pass it when you want capacity, pass something else
when you don't — keeps that choice with the caller instead of taking it
away. See [`sizeof()`](../MANUAL.md#sizeof) in the manual for the shipped
form, including the separate resolution rule for `sizeof` used *inside* a
function on one of its own array parameters (reads the existing count
parameter `copy_bound` already requires, no new state).

### Resolved: multi-D array parameters are now fully wired up

This gap is closed — `call_lines` now infers each array parameter's rank
from how the callee's own body indexes it, resolves the caller's array's
declared rank, rejects a mismatch as a compile error, and (when they agree)
emits a proper `DIM` with one bound per axis and nested copy loops (one
`FOR` per dimension) for both copy-in and `byref` copy-out. One argument
per axis is required after the array at the call site, in `DIM` order —
still fully manual, same as the 1-D convention.

Fixed alongside it: a pre-existing, unrelated codegen bug where any
2+-index array access (`grid%(r%, c%)`) parses as `Expr::Call`, not
`Expr::ArrayRef` (see `make_paren_ident_expr` in `parser.rs`), and the
`Expr::Call` fallback branch in `codegen.rs`'s `expr()` never resolved the
name through parameter/local scope the way the `Expr::ArrayRef` branch did
— so reading or writing *any* 2-D+ array (parameter or local) inside a
function body silently used the wrong, unmangled name. This wasn't a
multi-D-specific limitation so much as multi-D arrays never having been
exercised inside a function body at all before now.

`sizeof(x%, axis)` is now implemented and is the recommended way to supply
these per-axis bounds at a call site (`sumGrid%(g%, sizeof(g%, 0),
sizeof(g%, 1))`) instead of writing the literal bounds out by hand — see
[Decided against: unconditional auto-injection](#decided-against-unconditional-auto-injection)
above. The per-axis arguments themselves are still required at the call
site; `sizeof()` only saves the caller from having to know or restate the
array's bounds.

## Open questions / risks

- Does `sizeof` need to be usable as a general source-level expression
  (any `.bcl` code can call it), or only as an internal mechanism driving
  auto-injection? The write-up above assumes both, but that's not settled.
- Scoping: is a bound expression like `rows% - 1` always resolvable
  wherever a later `sizeof`/auto-injection call needs it? In practice bound
  expressions are almost always globals/consts/params, but a bound that
  referenced a function-local temp could be a problem — worth checking
  whether that's possible in valid BASCAL source at all.
- Whether auto-injection should still allow an explicit override at the
  call site for advanced cases, or whether it's unconditional as scoped
  above.
- Full multi-D array parameter passing (nested copy loops, `DIM` with
  multiple dynamic bounds) is a separate, larger piece of work than
  `sizeof` itself.

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
