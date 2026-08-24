[Home](./) / BASCAL: Technical Challenges

<div class="prose" markdown="1">

Seven problems from this stretch of BASCAL's development that didn't have an obvious right answer on the first pass — each with the actual options that were tried, what was learned from trying them, and the reasoning behind what finally shipped. Grounded in `git log`, the actual diffs, and (for the last one) two real compilers. This is the engineering detail behind [BASCAL: The Journey](journey.md)'s chronological story — read that first for the narrative, come here for the "why," topic by topic.

Every challenge in this series shares one constraint, worth stating once instead of repeating seven times: **BASCAL's target has no call stack, no heap, and no `REDIM`.** Its only real primitives are `GOSUB`/`RETURN` and global variables. Every challenge below is a different angle on the same question — what does a modern convenience (real parameters, array passing, `sizeof()`, recursion safety) have to look like when it's built honestly out of only those two things?

------------------------------------------------------------------------

## Challenge 1 — Parameter Copy Semantics: When Does a Callee's Change Reach the Caller?

### The problem

BASCAL functions transpile to a shared global storage slot per parameter plus a `GOSUB`/`RETURN` pair — there's no call stack, so every call to a given function reuses the same generated variables. Passing an array in meant copying its elements into that shared storage before the `GOSUB`, and copying them back out afterward. That copy-out happened *unconditionally* — every array argument, every call, whether or not the function had any business mutating it. Scalars had no copy-out path at all: a function could never hand a value back to its caller except through its one `return` result.

Underneath that asymmetry sat a real, silent bug. `ident()`, the function that resolves a source identifier to its generated storage name, checked a function's own parameters *before* it checked `global` declarations:

    function f(arr%, count%)
        global arr%   ' silently does nothing -- arr% already means the parameter
        ...
    end function

The `global arr%` line transpiled without error and did nothing at all — `arr%` inside `f` always meant the parameter, and the parameter check simply won every time.

### Options tested

- **Leave copy-out unconditional for arrays, do nothing for scalars.** This was the status quo, not a real option — it meant a caller had no way to pass an array *without* getting a copy-out it never wanted, and no way to get a scalar value back except by inventing a fake array-of-one just to unlock the copy-out path. Rejected as not actually solving the asymmetry.
- **Make `global` win over a same-named parameter.** Would have silently fixed the specific bug above, but at the cost of `global x` becoming a way to *rename* a parameter mid-function depending on what else happens to be declared globally elsewhere — a new, subtler footgun in place of the old one.
- **Reject the shadowing outright, and add an explicit copy-mode qualifier for both scalars and arrays.** Chosen.

### Decision

`global` shadowing a parameter name became a transpile-time error, full stop — no silent behavior either way. Separately, every parameter (scalar or array) gained a `byref`/`byval` qualifier, borrowed deliberately from Visual Basic/QBasic's vocabulary, with `byval` as the default:

    function insertionSort%(byref arr%(?))   ' byref: copied in, then back out
    function indexOf%(arr%(?), target%)      ' unmarked = byval: copied in only

`byval` copies in only; `byref` copies in and back out. Applied uniformly, this made a `byref` scalar a genuinely new capability — a true output parameter, something MBASIC/BASCOM never had outside a shared global — not just a fix to the array case.

> **Evidence:** `d96fe88` — 11 new tests covering both copy directions for both parameter kinds and the two new diagnostics.

------------------------------------------------------------------------

## Challenge 2 — Justifying Copy-In/Copy-Out Instead of Just Using Globals

### The problem

Challenge 1 shipped the mechanism. The obvious follow-up challenge wasn't technical, it was rhetorical, and it deserved a real answer rather than a hand-wave: *why copy parameters in and out at all, instead of just having every routine work directly on globals — any remotely structured BASIC would do the same?* Classic MBASIC/BASCOM programmers were used to exactly that — a `GOSUB`-based "subroutine" touching a global array was always touching the *one* array that existed, mutations visible everywhere instantly, because there was never more than one copy. Copy-in/copy-out is real overhead against that baseline, and it deserved a justification that held up, not just a design that happened to work.

### Options tested

- **Point to real structured BASIC dialects (QuickBASIC, Visual Basic) that already had genuine parameters, as evidence the copy-based approach is the established way to do this properly.** This was the first answer reached for, and it was the wrong comparison — those dialects have `SUB`/`FUNCTION` with real parameter lists built into the language, so citing them doesn't actually explain anything about why *BASCAL's* target needs to simulate that from scratch. It was corrected on the spot, once it was pointed out that the actual target dialect isn't QuickBASIC or VB at all.
- **Explain it purely in terms of MBASIC/BASCOM's own primitives** — the dialect BASCAL actually targets, which has no `SUB`, no `FUNCTION`, no parameter list of any kind, only `GOSUB`/`RETURN` plus a contract enforced solely by comments (*"expects `A$` and `B%` set, leaves the result in `C%`"*). Chosen.

### Decision

With only `GOSUB` and global variables as primitives, giving `.bcl` functions real parameters *means* simulating them out of exactly those two things — copy-in/copy-out is what that simulation necessarily looks like, not a stylistic alternative to something simpler that was available and passed over. A second, independent argument reinforces it structurally rather than just historically: each function body is transpiled exactly once, and every call site `GOSUB`s to that same shared label. A shared body needs one stable name for "its first parameter," but different call sites pass different things — different variable names, or a whole expression (`f(a% + b%)`, `f(5)`) that isn't a variable at all. There's no single caller-side location to just operate on directly in the general case, so the value has to land somewhere fixed before the shared body runs, regardless of which BASIC dialect is the reference point. `global` remains the deliberate escape hatch for the old always-shared behavior, precisely because committing to one hardcoded name forever is what makes it *not* a reusable, callable-with-different-data routine anymore.

> **Evidence:** `99c1809` — added "Why Copy-In/Copy-Out, Not Just Globals?" to `MANUAL.md`.

------------------------------------------------------------------------

## Challenge 3 — Multi-Dimensional Arrays: The Bug Hiding Under the Missing Feature

### The problem

Asking "do we handle multi-dimensional array params?" surfaced two answers, not one. The visible gap: `call_lines` only ever emitted a single-bound `DIM` and a single-axis copy loop, no matter how many dimensions the source array actually had — passing a 2-D array transpiled clean and generated *wrong* BASIC. The hidden gap underneath it was worse. `grid%(r%, c%)` — any array access with two or more indices — doesn't parse as `Expr::ArrayRef` at all; the parser only produces that node for an empty-parens or single-index access. Two-or-more-index accesses parse as `Expr::Call`, disambiguated from a real function call later purely by name. The `Expr::Call` fallback branch in codegen's `expr()` had never been taught to resolve a name through parameter or local scope the way `Expr::ArrayRef` already did — so reading or writing *any* 2-D-or-higher array, parameter or local, inside a function body silently used the wrong, unmangled name.

### Options tested

- **Fix only the visible gap (rank-aware `DIM`/copy loops).** Would still have generated broken output for any multi-dimensional array actually read or written inside a function body — the deeper bug is in a different code path entirely and wouldn't have been exercised by testing the copy machinery alone.
- **Trace why the deep bug had never been caught.** It hadn't, because nothing had ever indexed a multi-dimensional array inside a function body before — the visible feature gap and the hidden bug had been protecting each other from detection.
- **Fix both together, with a two-way cross-check as the permanent safety net.** Chosen.

### Decision

Array parameter rank is now inferred from how the function's own body indexes it, cross-checked against whatever rank the caller actually passes at each call site — a mismatch in either direction is a transpile-time error, not a best-effort generation. The `Expr::Call` fallback branch was fixed to resolve names through scope exactly like `Expr::ArrayRef` does, closing the silent-wrong-name bug at its source rather than only where it happened to be observed.

> **Evidence:** `d96fe88`, shipped alongside Challenge 1.

------------------------------------------------------------------------

## Challenge 4 — Declaring an Array Parameter's Shape

### The problem

Nothing in a parameter declaration said whether a parameter was an array at all, let alone how many dimensions it had. `arr%` in `function insertionSort%(byref arr%, count%)` looked identical whether the transpiler would infer it as a scalar or a 5-D array — rank was entirely inferred from body usage, invisible at the declaration, surfacing only as a transpile error if a caller eventually got it wrong. A reader had no way to know a parameter's shape without reading the whole function body.

### Options tested

- **`arr%(,,)`** — one comma per extra dimension after the first, mirroring `dim`'s own bound list shape. Rejected on sight as visually awful and hard to count at a glance.
- **`*`, one per dimension.** Considered and dropped without a strong reason to prefer it over the alternative below.
- **A bare rank count, `arr%(3)`.** Reads more compactly at high rank, but overloads a bare number to mean two *different* things depending on context — a bound in `dim`, a dimension count in a parameter declaration. Rejected: a number that means different things in adjacent contexts is worse than an unfamiliar syntax that means one thing consistently.
- **`(?, ?, ...)`, one `?` per dimension.** Chosen — not actually new syntax, since BASCAL already used `?` for "deliberately incomplete, filled in elsewhere" in `?{ field: value }` partial record literals. Reusing that convention for a parameter's shape was more consistent than inventing a second symbol for the same idea.

A second question followed once the first was settled: did the *call site* also need a marker? Once a declaration states a parameter's rank, the transpiler doesn't need one — it can tell an argument is meant as an array from what the identifier resolves to. `sumGrid%(g%(), 3, 3)` and the bare `sumGrid%(g%, 3, 3)` were made to generate byte-identical output, so the old `()` marker stayed valid rather than being deprecated.

### Decision

    function insertionSort%(byref arr%(?))
    function sumGrid%(byref grid%(?, ?))
    function sumCube%(byref cube%(?, ?, ?))

A scalar parameter stays a bare name; an array parameter states its rank directly, one `?` per axis, cross-checked against body usage the same way Challenge 3 already checked it against the caller. This was a real breaking change on the declaration side — a parameter indexed as an array with no declared rank became a transpile-time error — but purely additive at the call site.

> **Evidence:** `48a3866`, `c7fce0e`, `fda32cd` — 5 new tests, migrated every existing array-parameter declaration across the tutorials and manual.

------------------------------------------------------------------------

## Challenge 5 — Stopping Recursion Before It Corrupts Shared State

### The problem

Every BASCAL function's storage is one shared, generated global, reused by every call — there's no call stack to give each invocation its own frame. A function calling itself, directly or through a cycle of other functions, would have two active invocations racing to overwrite the exact same storage mid-execution. Nothing in the transpiler caught this before it reached generated BASIC.

### Options tested

- **Detect only direct self-recursion** (`f%` calling `f%` somewhere in its own body). The simplest check, and the one that would catch the most obvious case, but leaves the door open to `isEven%` calling `isOdd%` calling `isEven%` — a cycle just as fatal, one hop removed from the obvious case.
- **Full call-graph cycle detection**, treating every function and procedure as a node and every call as an edge, regardless of how many hops the cycle takes or whether it mixes functions and procedures. Chosen.

Implementing the full version turned out to be cheaper than it sounded, because the exhaustive AST walkers needed to find "does this code call that function anywhere" (`statements_call_function`, `expr_calls_function`) already existed from other checks, already correct for calls nested inside `if`/`while`/`for`/`select case`. A standard DFS with white/gray/black coloring, built on top of those walkers, gave general cycle detection almost for free — and inherited the nested-call correctness of the walkers it was built on, verified directly with a call buried four levels deep (`for` → `select case` → `while` → `if`) between two mutually recursive functions.

### Decision

`reject_call_cycles` walks the whole program's call graph and reports every distinct cycle it finds, at any depth, wherever the calls are nested — not just direct self-recursion. This shipped as version 0.99.9.

> **Evidence:** `e389d3e` (5 new tests), `306cd1d` (the conditional-nesting regression test), tag `v0.99.9`.

------------------------------------------------------------------------

## Challenge 6 — Telling a Callee How Big Its Array Argument Is

### The problem

The manual's own flagship multi-dimensional example had a live bug: `dim g%(2, 2)` (valid indices `0..2`) called through `sumGrid%(g%, 3, 3)` — the caller had to type the array's bounds by hand at every call site, and here they didn't match, so the generated copy loop read one row and one column past the real array into whatever memory happened to follow it. The deeper problem wasn't this one typo — it was that *any* hand-typed count was one keystroke away from drifting out of sync with the array's real size, silently, with no transpiler check possible on a value it never saw connected to the array it described.

### Options tested

- **`sizeof()` as an explicit, caller-written call-site argument** — `sumGrid%(g%, sizeof(g%, 0), sizeof(g%, 1))`. Built first, shipped, documented, tested. It solved the immediate bug (the sizes now came from the array's own `DIM` instead of being retyped), but it wasn't actually what had been asked for: the caller still had to write something at every call site, just a `sizeof()` expression instead of a literal. The design proposal that led here had originally sketched full auto-injection and talked itself out of it, on its own reasoning, without confirming that reasoning against what was actually wanted.
- **Unconditional auto-injection**: the caller writes nothing beyond the array itself; the transpiler resolves the real argument's bounds (from its own frozen `DIM`, or — for a forwarded array parameter — from what its own caller already carries) and assigns them into transpiler-synthesized hidden variables, set immediately before `GOSUB`, invisible in `.bcl` source. Chosen — reversing the first option entirely once it became clear that was the actual goal.

The reversal is documented in the commit that made it, in terms worth quoting directly rather than paraphrasing:

> "This replaces the sizeof()-as-explicit-call-site-argument design from the previous commit, which asked the caller to write `sumGrid%(g%, sizeof(g%, 0), sizeof(g%, 1))` by hand — the actual goal was for the caller to write nothing beyond the array itself (`sumGrid%(g%)`) and have the compiler carry the sizes automatically."

### Decision

    function sumGrid%(byref grid%(?, ?))
        total% = 0
        for r% = 0 to sizeof(grid%, 0) - 1
            for c% = 0 to sizeof(grid%, 1) - 1
                total% = total% + grid%(r%, c%)
            end for
        end for
        return total%
    end function

    dim g%(2, 2)
    print sumGrid%(g%)

`sizeof()` still exists and reads exactly the same either way — the only thing that changed is who writes the size expression at the call site: nobody. The freeze-at-`DIM`-time machinery that makes `sizeof()` possible, previously gated to fire only for arrays it was directly called on, became unconditional, since any array might now get passed into a function anywhere in the program.

> **Evidence:** `8478b0d` (the first version, 9 tests), `afc6b46` (the reversal), `548bf58` (two missed GitHub Pages snippets caught and fixed afterward).

------------------------------------------------------------------------

## Challenge 7 — Sizing an Array Parameter's Shared Storage, Once, Safely

### The problem

Auto-injection (Challenge 6) fixed how big a *copy loop* should run per call. It left a more basic problem completely unaddressed: how big the *storage array itself* is allowed to be, for the entire life of the program. `call_lines` `DIM`ed a parameter's shared storage at *every call site* that used it. Since storage is one global reused by every call — the same reason scalar parameters share storage — and classic BASIC has no `REDIM`, a second `DIM` on an already-`DIM`ed array is a fatal runtime "Duplicate Definition" error. Any function with an array parameter called more than once anywhere — two different call sites, or one call site inside a loop — was already generating BASIC that would crash the moment it actually ran. This wasn't new: it had been present since the very first array-copy-in/copy-out implementation, and the proof was already sitting in the shipped tutorials — `printArray%` in `tutorial/08_arrays.bcl` is called twice, and its storage was being `DIM`ed twice.

### Options tested

- **`ERASE`-then-`DIM` at each call site**, scoped tightly around the call: `DIM`, call, copy out if `byref`, `ERASE`. Classic MBASIC/BASICA/GW-BASIC support exactly this idiom — `ERASE` deallocates an array, clearing the "already dimensioned" state a plain re-`DIM` would trip over, even letting the new `DIM` use a different size or rank. If it had worked, this would have replaced almost everything else in this challenge: no capacity inference, no constant-folding, no fixed-point resolution, no fallback syntax. It was checked directly, against two independent real targets, rather than assumed from memory of the idiom:
  - `fbc -lang qb` (FreeBASIC 1.10.1, the actual dialect BASCAL's own execution tests run generated output through): `DIM a%(3)` / `ERASE a%` / `DIM a%(9)` fails with `error 4: Duplicated definition, a`, on the second `DIM`, in every variant tried (same size, different size, different rank, a variable bound). Identical under `-lang fblite`.
  - A genuine IBM/Microsoft BASIC Compiler 2.00 (1985), sourced as a disk image from a software-preservation archive and run in DOSBox-X: the same test, and a plain double-`DIM`-with-no-`ERASE` control, both report `DD` (Duplicate Definition) as a compile-time severe error — on the redeclaring `DIM`, whether or not `ERASE` preceded it. The compiler still linked and ran the result anyway, and the output showed exactly the silent-corruption failure mode this whole feature exists to prevent: one redim's value happened to print correctly by incidental data-segment padding luck; the rank-changing redim printed nothing at all.

  Rejected — real MBASIC-family idiom, but it belongs to the *interactive interpreter* (`MBASIC.EXE`, `GW-BASIC`), which allocates array storage dynamically at the moment a `DIM` statement executes. BASCOM is a *compiler*: every array is already committed to a fixed data-segment slot by the time `DIM` is translated, so there's no allocation left for `ERASE` to hand back. Not a FreeBASIC-specific gap — a genuine property of the compiled-output model BASCAL actually targets.
- **Always require an explicit, author-written capacity.** Would have worked, but reintroduces exactly the manual-number burden Challenge 6 had just removed, for every array parameter, even the overwhelming majority whose safe size the transpiler can already work out for itself from the program it's already parsing.
- **Infer a capacity automatically, with an explicit fallback only where inference genuinely can't work.** Chosen.

### Decision

A parameter's storage is `DIM`ed exactly once, at the top of the generated program, before any call happens. Its size — a *capacity*, a new and separate number from `sizeof()`'s per-call actual size — is resolved one of two ways:

- **Inferred** (`arr%(?)`, the default): the transpiler scans every call site of the function across the whole program and takes the largest resolved size, but only when every one of those sizes is itself a transpile-time constant — a literal `DIM` bound, a `const` (folded recursively through simple arithmetic), or an array that's itself another function's array parameter forwarded onward, resolved through *that* parameter's own already-settled capacity. Because BASCAL rejects every call cycle (Challenge 5), that forwarding chain is always finite, so resolving it is a straightforward fixed-point pass.
- **Explicit** (`arr%(100)`, a literal in place of `?`): required only when some call site's array size is a genuine runtime value with no way to know it ahead of time.

A capacity provably too small for a resolvable call site is a transpile-time error. Every call site, resolved or not, also carries an unconditional runtime check as a backstop:

    IF sumarr_arr_dim0_0% > 100 THEN PRINT "runtime error: ..." : STOP

The `100` here isn't a fixed constant baked into the transpiler — it's this specific parameter's own resolved capacity, substituted in literally by whichever of the two branches above decided it (the largest inferred call-site size, or the explicit number the author wrote). A different array parameter with a different capacity gets the same check shape with its own number in place of `100`; the check itself is harmless dead code on a call the transpiler already proved safe, and the only defense at all for the one case that's genuinely unprovable ahead of time.

> **Evidence:** `763eecf` (9 new tests, including the direct regression test: a function called three times still gets exactly one `DIM`); `ERASE`/`DIM` findings recorded in [proposals/array-sizeof-and-auto-count-params.md](https://github.com/johnjoeallen/bascal/blob/main/proposals/array-sizeof-and-auto-count-params.md), "Considered and rejected: ERASE-then-DIM re-declaration."

------------------------------------------------------------------------

## Reading across all seven

Every accepted decision above has the same shape: find the smallest amount of new machinery that tells the truth about what the target can actually do, and push everything else onto something the transpiler can verify instead of something the author has to remember correctly by hand. Every rejected option, without exception, either asked the author to keep two things in sync by hand (a size and the array it describes, in Challenges 6 and the abandoned parts of 7) or quietly assumed a runtime capability the target didn't actually have (dynamic redimensioning, in Challenge 7's main rejected option). The two challenges that got real evidence from outside the codebase — the FreeBASIC/BASCOM tests in Challenge 7 — are also the two where a plausible answer from memory turned out to be wrong in a way only running the actual target compiler could show.

</div>

[← Read the narrative: BASCAL: The Journey](journey.md)
