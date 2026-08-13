# Proposal: compiler-tracked array sizes, `sizeof()`, and auto-injected count parameters

Status: the `sizeof()`/auto-injection sections below are still discussion
only — not implemented, not committed to. The **`byref`/`byval`** and
**`global` shadowing** sections near the end are **implemented** (parser,
resolver, codegen, tests, `MANUAL.md`/`docs/manual.html`, and the affected
tutorial sources all updated — see `git log` for the commit). This file
remains a write-up of a design conversation for the still-open pieces.

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

### Auto-injection at call sites

Auto-inject unconditionally: whenever an array is passed at a call site
(`arr%()`), the compiler supplies the frozen size (one hidden argument per
axis) itself, rather than requiring the caller to pass it.

This does **not** reintroduce the capacity-vs-logical-count problem it might
look like at first: the injected value only answers a mechanical question —
how many slots to copy in `DIM`/copy-in/copy-out so nothing is truncated or
reads garbage. A program that tracks its own "how much of this buffer is
actually filled" count (e.g. records loaded from a file into a
fixed-capacity array) keeps that as its own ordinary, explicit,
developer-owned parameter, same as today — it's answering a different
question than the injected capacity value, so the two coexist without
conflict. Today's single `count%` convention conflates these two numbers
into one manually-passed value; decoupling them is arguably cleaner than
what exists now.

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

`sizeof(x%, axis)` (still proposed, not implemented) would now have real
codegen to plug into for auto-injecting the per-axis bounds this section
still requires manually.

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
