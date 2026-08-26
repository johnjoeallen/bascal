[Home](../) / BASCAL: The Journey

<div class="prose" markdown="1">

## Preface

The BASCAL compiler (`bcc`) transpiles structured `.bcl` source into line-numbered Microsoft BASIC — the dialect a working MBASIC or BASCOM programmer in the 1980s would recognize, built to run on real vintage toolchains and on FreeBASIC's QB-compatibility mode. It exists, in the author's own words, as a fun project — built to see what's possible, not to solve a problem anyone actually has anymore.

That framing matters for the story that follows, because most of the hardest work in this stretch of BASCAL's life wasn't inventing new language features. It was staying honest about a genuinely awkward constraint: the target language has no call stack, no heap, no `REDIM`, and (by design) no recursion. Every "modern" convenience BASCAL adds — real parameters, array passing, a `sizeof()` builtin — has to be *simulated* on top of `GOSUB`, `RETURN`, and global variables, and simulating something faithfully means the simple-looking answer is often wrong in a way that only shows up once you ask what the generated BASIC actually does at runtime, twice, in a loop, forever.

This is the story of getting that simulation right — including the two times it had to be taken apart and rebuilt because the first version, while it compiled and passed its own tests, wasn't what was actually needed.

Every claim below is checked against `git log` and the actual diffs, not against memory of how the work felt — commit hashes and dates are cited at the end of each chapter, and a full log appears as an appendix. This is the story of *what happened, in order*; the engineering detail behind each mechanism — the options that were tried and rejected, and the precise reasoning for what shipped — lives in a companion piece, [*BASCAL: Technical Challenges*](challenges.md), linked from each chapter below rather than repeated here.

### By the Numbers

This stretch of work spans **22 commits** across three calendar days — a trailing edit on 2026‑08‑11, a day of front-end work on 2026‑08‑12, and the bulk of the transpiler work compressed into 2026‑08‑13 and 2026‑08‑14. It touches **59 files**, **+5,170 / −1,595** lines. `src/codegen.rs`, the transpiler's code generator, grew from 2,045 lines to 3,222 — a 57% increase — and `src/lib.rs`'s test suite grew from 77 tests to 117, before counting the separate integration-test crate. One version tag was cut in the middle of it (`v0.99.9`, at `707730f`); three more feature commits landed after that tag and, per BASCAL's own release convention, none of them bumped the version on their own — a version bump happens only when someone actually asks for a release, not automatically per feature.

------------------------------------------------------------------------

## Chapter 1 — Face and Voice

The earliest work in this arc wasn't transpiler work at all. BASCAL's GitHub Pages site got a proper identity: a violet block-cursor favicon, and a retro green-screen computer graphic for the docs index and README, sized and positioned through a few rounds of "not quite" — too big, wrong spot, competing with the hero paragraph instead of sitting beside it. The prose got a pass too. An early draft called BASCAL a "pet project"; that read as more dismissive than intended, and settled instead on describing it as a fun project built to see what's possible — confident about its scope without either apologizing for it or overselling it. The whole documentation set then got a straight read for grammar, tone, and engineering credibility, and the findings got fixed in one pass.

None of this shows up in a diff of the transpiler, but it set the register for everything after: BASCAL's docs were going to be treated as seriously as its code.

> **Commits:** `4005bbb` favicon, `d134912`/`8f0061e` hero graphic (added, then resized and repositioned), `5492960`/`d0444f9`/`eee0f90` tone pass, `a55da8d` grammar/tone/precision review — all 2026‑08‑12.

------------------------------------------------------------------------

## Chapter 2 — What Does Copy-In/Copy-Out Even Mean Here?

The transpiler work starts with a question that sounds simple: *explain the array-size, count-variable convention.* Answering it honestly surfaced a real, silent bug hiding in plain sight: `global arr%` inside a function with `arr%` as a parameter transpiled clean and did *nothing at all*, because the parameter always resolved first — no diagnostic, no warning, just a dead line of source. Underneath that sat a deeper asymmetry: arrays always copied their result back to the caller, unconditionally, whether the function had any business mutating them or not; scalars, by contrast, had no way to hand a value back at all except through the function's one `return`.

Both became one change: `global` shadowing a parameter is now a transpile-time error, and every parameter — scalar or array — gained an explicit `byref`/`byval` qualifier, `byval` (copy-in only) as the default. The full design — why that shape, what else was considered — is Challenge 1 and Challenge 2 in [*BASCAL: Technical Challenges*](challenges.md).

------------------------------------------------------------------------

## Chapter 3 — The Bug Under the Bug

The natural next question — *do we handle multi-dimensional array params?* — turned out to have two answers, and the second one was worse than the first: a visible feature gap (no rank-aware `DIM`/copy loops), and underneath it, a real bug that had simply never been exercised — *any* two-or-more-index array access, parameter or local, silently resolved to the wrong generated name inside a function body, because that code path had never been taught to resolve names through scope the way its single-index sibling was. Neither half had ever been caught, because nothing had tried to use a multi-dimensional array inside a function body before this. Fixing the visible gap is what surfaced the hidden one. Full technical detail: Challenge 3 in [*Technical Challenges*](challenges.md).

Chapters 2 and 3 shipped together as one commit — `byref`/`byval`, the `global`-shadowing error, and multi-dimensional array parameters all landed in the same change, with 11 new unit tests covering both directions of copy semantics for scalars and arrays, the two new diagnostics, 2-D/3-D copy generation, and both `Expr::Call` fixes. It also tagged **v0.99.8**.

> **Commit:** `d96fe88` "Add byref/byval parameter passing and multi-dimensional array parameters", tagged `9c8e9b4` → v0.99.8 — 2026‑08‑13.

------------------------------------------------------------------------

## Chapter 4 — Comments, Everywhere

Once functions had a real calling convention worth explaining, every example in the documentation needed to explain it. A stale-reference pass first caught leftover `byref`/copy-out claims on GitHub Pages that no longer matched the new default. Then a full sweep added a trailing comment to every parameter in every function and procedure example across the tutorials and the manual — not because any one comment was hard, but because doing it *everywhere*, consistently, is its own kind of correctness work. A few instances were missed on the first pass (one hiding in a markdown-indented code block a naive regex skipped over) and caught on re-scan; the sweep alone touched 23 files.

> **Commits:** `0052246` stale-reference fixes, `966198b` "comment every parameter in every example, repo and GH Pages" — both 2026‑08‑13.

------------------------------------------------------------------------

## Chapter 5 — Saying What You Mean in the Declaration

The next complaint aimed at something structural: *why do we omit `()` from parameter declarations, and why can an array parameter get away without stating its own dimensionality?* Under the hood, rank was purely *inferred* from how a function's body indexed its own parameter — invisible at the declaration, surfacing only as a transpile error if a caller eventually got it wrong. That's backwards for a signature: a reader shouldn't have to read the whole function body to know whether `arr%` is a scalar or a 5-D array.

The design conversation that followed went through real dead ends before landing anywhere. A first pass at "state the rank in the declaration" produced `arr%(,,)` for a 3-D array — rejected on sight as awful. `*` was considered and dropped. A bare rank count (`arr%(3)`) read more compactly but overloaded a bare number to mean two different things depending on context — a bound in `dim`, a rank count in a parameter — which was worse than no syntax at all. The syntax that stuck, `(?, ?, ...)`, one `?` per dimension, won because it wasn't actually new: BASCAL already used `?` for "deliberately incomplete, filled in elsewhere" in `?{ field: value }` partial record literals, and applying the same mark to a parameter's shape reused an existing convention instead of inventing a second one.

With direction confirmed — *implement this, don't ask any further questions* — the rank syntax shipped as a real breaking change on the declaration side (every existing `arr%` had to become `arr%(?)`), migrated by hand across every tutorial and doc example after a bulk `sed`-style edit was tried and explicitly rejected mid-flight in favor of precise, per-file edits. Along the way, regenerating one previously stale tutorial output turned up a genuine correctness regression sitting quietly in already-shipped output — the kind of thing that only surfaces when you actually regenerate every artifact instead of trusting it's still current. Full syntax rationale and the call-site consequence: Challenge 4 in [*Technical Challenges*](challenges.md).

> **Commits:** `48a3866` clarified the old behavior first (no array syntax existed there at all), then `c7fce0e` recorded the decided design, then `fda32cd` "Add explicit array-parameter rank syntax: arr%(?), grid%(?, ?)" (5 new tests) shipped it — 2026‑08‑13 into 2026‑08‑14.

------------------------------------------------------------------------

## Chapter 6 — A Transpiler That Refuses to Recurse

Mid-stream, a new, unrelated requirement arrived: *a transpile-time check for recursion, and it's a fatal error.* BASCAL's whole calling convention depends on it — every function's storage is one shared global, reused by every call, since there's no call stack. A function calling itself, directly or through a cycle of other functions, would have two active invocations racing to overwrite that same storage: not a style problem, silent data corruption. The fix generalized past the obvious case — not just direct self-recursion, but any cycle at all, at any depth, nested inside conditionals and loops — by building on exhaustive AST walkers that already existed for other checks. Full mechanism: Challenge 5 in [*Technical Challenges*](challenges.md).

This work shipped as **version 0.99.9** — documented, committed, tagged, pushed, with GitHub release notes written to match.

> **Commits:** `e389d3e` "Reject indirect recursion at compile time, not just direct self-calls" (5 new tests), `306cd1d` the conditional-nesting regression test, `707730f` version bump, tag `v0.99.9` — 2026‑08‑14.

------------------------------------------------------------------------

## Chapter 7 — Justifying the Whole Approach

With `byref`/`byval` in place, an obvious challenge followed: *why copy in and out at all, instead of just having every routine work directly on globals — any remotely structured BASIC would do the same?* The first answer reached for the wrong comparison — real structured dialects like QuickBASIC and Visual Basic, which already had genuine parameters — and got corrected on the spot: the actual target, MBASIC/BASCOM, has no `SUB`, no `FUNCTION`, no parameter list at all, only `GOSUB`/`RETURN`. Restated in those terms, the justification became a permanent manual section: *Why Copy-In/Copy-Out, Not Just Globals?* Full argument, including the corrected comparison: Challenge 2 in [*Technical Challenges*](challenges.md).

> **Commit:** `99c1809` "docs: explain why parameters copy in/out instead of just using globals" — 2026‑08‑14.

------------------------------------------------------------------------

## Chapter 8 — The Bug That Started `sizeof()`

The `sumGrid%` example in the manual — the flagship illustration of multi-dimensional array parameters — had a real, live bug: a 2×2 array, `dim g%(2, 2)`, called through `sumGrid%(g%, 3, 3)`. Valid indices on `g%` run `0..2`; the call claimed `3, 3`, and the generated copy loop would read one row and one column past the actual array, into whatever memory happened to follow it.

The fix that was actually asked for was narrow and specific: *we should not pass the sizes — we should use `sizeof()`.* At the time, `sizeof()` didn't exist at all — only the array-rank-declaration syntax did, a materially smaller feature. The harder half of building it was flagged directly by the person steering the work, mid-design: *the messy bit is passed parameters — `sizeof()` on those needs to use the source array's info, which needs to get magic'd into the real passed params in BASIC, so `sizeof()` behaves differently in that case.* That framing pointed straight at the answer that shipped, reusing state the existing array-copy machinery already required rather than inventing anything new. Full mechanism: Challenge 6 in [*Technical Challenges*](challenges.md). The buggy `sumGrid%` example got rewritten to call `sizeof(g%, 0)` / `sizeof(g%, 1)` instead of typing bounds by hand.

> **Commit:** `8478b0d` "feat: add sizeof() builtin for array bounds" — 2026‑08‑14.

------------------------------------------------------------------------

## Chapter 9 — The First Rebuild: "That's Not What I Wanted"

What shipped in Chapter 8 was real, tested, and documented — and it was the wrong design. The correction landed plainly: *the user would leave the `rows%`, `cols%` out; the transpiler would push/pass the sizes with the array; the generated code would use the passed sizes for `sizeof()` in the function.* The intent had never been an explicit, caller-written `sizeof()` argument at every call site — it was for the caller to write nothing beyond the array itself, with the transpiler carrying its size along invisibly.

That was, in fact, the *original* idea sketched in the design proposal — and it had been talked out of unconditional auto-injection on its own reasoning, without confirming that reasoning against what was actually wanted. The self-authored justification for staying explicit — that a capacity number and a caller-tracked logical fill-count are two different things a program might need independently — wasn't wrong as an argument, but it had been used to reject the very design that was actually being asked for.

The rebuild was substantial: the manual count-parameter convention disappeared entirely, not just from the call site but from the function signature too — `sumGrid%(byref grid%(?, ?), rows%, cols%)` became `sumGrid%(byref grid%(?, ?))`, full stop, with the transpiler carrying every size silently through hidden, transpiler-synthesized variables instead. Full mechanism: Challenge 6 in [*Technical Challenges*](challenges.md).

Every array-parameter example across the manual, the GitHub Pages mirror, and the tutorial sort and statistics libraries got migrated to the new convention in the same pass — 18 files, +642/−630 lines. Two of the GitHub Pages tutorial snippets were missed in that sweep — `arrays.html` and `require.html` still showed the old, manually-counted calling style — and got caught and fixed once specifically asked whether a `sizeof()` tutorial existed at all. The reversal is documented in the commit's own words, worth quoting rather than paraphrasing:

> "This replaces the sizeof()-as-explicit-call-site-argument design from the previous commit, which asked the caller to write `sumGrid%(g%, sizeof(g%, 0), sizeof(g%, 1))` by hand — the actual goal was for the caller to write nothing beyond the array itself (`sumGrid%(g%)`) and have the compiler carry the sizes automatically."

> **Commits:** `afc6b46` "feat: auto-inject array bounds at call sites, remove manual count params", `548bf58` fixed the two missed GitHub Pages snippets — both 2026‑08‑14.

------------------------------------------------------------------------

## Chapter 10 — The Second Rebuild: Storage Has to Be Real

The correction that followed cut deeper than syntax. *Function params are really just normal vars, so they have to have a dimension — I'm not sure `?, ?` can work, since we need to copy into that array and it needs to be at least as big as the biggest arg we want to pass.*

Checking that claim against the actual generated output confirmed it immediately, and worse than expected: it wasn't a limitation of the new auto-injection design specifically — it was a bug that had been present since the very first array-copy-in/copy-out implementation, unnoticed the entire time. `call_lines` emitted a fresh `DIM` for a parameter's shared storage at *every call site*. A function's storage is one global, reused by every call — the same reason scalar parameters share storage — but arrays additionally need a fixed *size*, and classic BASIC has no `REDIM`: a second `DIM` on an already-`DIM`ed array is a fatal runtime "Duplicate Definition" error. Any function with an array parameter called more than once anywhere — two different call sites, or one call site inside a loop — was already generating BASIC that would crash the moment it actually ran. The proof was sitting in the shipped tutorials the whole time: `printArray%` in `tutorial/arrays.bcl` is called twice, and its storage was being `DIM`ed twice.

The follow-up from the person who'd caught the bug went straight to the crux of the fix: *we parse all source for generation, so we can determine the max dimensions of any passed array — but for dynamically sized arrays, we need to know the value at compile time, and I don't think that's possible.* That's exactly right, and it shaped the design into two tiers: storage `DIM`ed exactly once, its size *inferred* automatically from every call site when every one of those sizes is a transpile-time constant, falling back to an *explicit* author-written number only when it genuinely isn't. One more requirement landed as direct confirmation before it shipped: *yes, and we still check the actual array capacity before calling, at runtime, and emit a runtime error* — so every call site also carries an unconditional runtime backstop, regardless of what the transpiler already proved. Full two-tier design, the fixed-point resolution through forwarded parameters, and the runtime check: Challenge 7 in [*Technical Challenges*](challenges.md).

Regenerating the affected tutorial outputs confirmed the fix concretely: `tutorial/arrays.bas`, where `printArray%` is called twice, now emits `DIM printarray_arr_0%(6)` exactly once, at the top of the file, instead of twice at each call site.

> **Commit:** `763eecf` "fix: array parameter storage is DIMed once, sized to a resolved capacity" — 11 files, +1,122/−183 — 2026‑08‑14. No version bump; `Cargo.toml` still reads `0.99.9`, the tag cut back in Chapter 6.

------------------------------------------------------------------------

## Chapter 11 — Checking It Against the Real Thing

The capacity design closed the bug, but it also raised an obvious question, worth asking before treating the fix as final: did BASCAL actually need to build this at all? Classic MBASIC-family BASIC has a documented idiom for resizing an array — `ERASE arrayname` followed by a fresh `DIM`, even with a different size or rank, since `ERASE` deallocates the array and clears the "already dimensioned" state a plain re-`DIM` would trip over. If that idiom held for BASCAL's actual target, a much simpler design was on the table: `DIM`, call, copy out if `byref`, `ERASE` — scoped tightly around each call, with no capacity inference, no `const_eval`, no fixed-point pass, and no explicit-capacity fallback syntax needed anywhere.

It was checked directly, against two independent real targets, rather than assumed from memory of the idiom.

First, `fbc`'s QB-compatibility mode — the actual dialect the existing FreeBASIC execution tests run generated output through. `DIM a%(3)` / `ERASE a%` / `DIM a%(9)` fails to compile: `error 4: Duplicated definition, a`, on the second `DIM`, in every variant tried — same size, different size, different rank, a variable bound instead of a literal. Identical under `-lang fblite` too.

That result alone was enough to keep the shipped design, but it left open whether this was some FreeBASIC-specific gap in its QB emulation rather than a real limit of the target dialect. So the second check went further: a genuine copy of IBM/Microsoft BASIC Compiler 2.00 — the actual 1985 compiler, not an emulation of its dialect — sourced as a disk image from a software-preservation archive, extracted with the archive's own conversion tooling, and run headless in a freshly-installed DOSBox-X: mount, compile, link, run, each step's output readable straight off the host filesystem, no interactive keystroke required.

The result was more decisive than the FreeBASIC check, and came with a genuine surprise attached. A plain double-`DIM`-with-no-`ERASE` control case reported `DD` — Duplicate Definition — exactly as expected. But so did the `ERASE`-then-`DIM` case: real BASCOM 2.00 rejects the redeclaration too, as a compile-time severe error, on both the same-rank resize and the rank-changing one. And despite reporting those as severe errors, the compiler still linked and produced a runnable `.exe` — running it showed precisely the silent-corruption failure mode the whole capacity feature exists to prevent: the first resize's value happened to print correctly (the original, never-actually-resized allocation had enough incidental data-segment padding to absorb it by luck), but the rank-changing resize printed nothing at all for a value that should have been there.

The likely explanation reconciles the memory of the idiom with both results without either being wrong: `ERASE`-then-`DIM` resizing is real, but it belongs to the *interactive interpreter* — `MBASIC.EXE`, `GW-BASIC` — which allocates each array's storage at the moment a `DIM` statement actually executes. BASCOM is a *compiler*; by the time it's translating source, every array is already committed to a fixed slot in the executable's data segment, so there's no allocation left for `ERASE` to hand back later. BASCAL targets exactly that compiled-output model, so the idiom was never actually available to it — not a FreeBASIC emulation gap, a genuine property of the target. The capacity-inference design stayed exactly as shipped, now with real evidence behind the decision instead of an assumption about a forty-year-old dialect quirk.

------------------------------------------------------------------------

## Retrospective

Two things run through this whole stretch of work, and they're worth naming plainly rather than leaving implicit.

The first is that the two real reversals — explicit `sizeof()` at the call site, and per-call-site `DIM` — both compiled cleanly, passed their own tests, and were still wrong. Neither failure was a syntax error or a type error; both were failures to check the actual generated BASIC against what the target dialect can genuinely do at runtime, under repetition. A design that looks complete because it type-checks and has green tests is not the same claim as a design that's correct for a language with no call stack, no heap, and no `REDIM`. The second time, the fix came from being asked to verify a claim (*I don't think that's possible*) against real behavior, not from being told the fix outright — and confirming it by actually reading the doubled `DIM` in already-shipped tutorial output made the bug undeniable before a single line of the fix was written.

The second is more of a throughline than a lesson: almost everything in this document is one recurring shape. BASCAL's target dialect gives you exactly two primitives — `GOSUB`/`RETURN` and global variables — and every feature here is an argument about how to simulate something richer out of only those two, faithfully, without pretending the simulation is something it isn't. `byref` doesn't create a real reference; it copies twice instead of once. `sizeof()` doesn't inspect an array at runtime; it resolves to a value fixed at transpile time, sometimes literally, sometimes through a hidden variable set moments before the call that reads it. Storage capacity isn't dynamic; it's a ceiling chosen once, as high as the whole program will ever need, because the alternative — pretending an array could just get bigger — was never actually available. Chapter 11 is the same throughline from the opposite direction: rather than assume the honest shape of the simulation from memory of a forty-year-old idiom, it went and asked the actual 1985 compiler, and got a real answer instead of a plausible one.

------------------------------------------------------------------------

## Appendix — Commit Log

Every commit cited above, in order, exactly as `git log` records it. Chapter 11 is investigation, not a code change — no commit corresponds to it; the finding is recorded in [proposals/array-sizeof-and-auto-count-params.md](https://github.com/johnjoeallen/bascal/blob/main/proposals/array-sizeof-and-auto-count-params.md) under "Considered and rejected: ERASE-then-DIM re-declaration" instead.

| Commit    | Date  | Message                                                                  |
|-----------|-------|--------------------------------------------------------------------------|
| `4005bbb` | 08‑12 | docs: add violet cursor favicon across the site                          |
| `d134912` | 08‑12 | docs: add retro all-in-one computer hero graphic                         |
| `8f0061e` | 08‑12 | docs: shrink hero graphic and float it beside the opening paragraph      |
| `5492960` | 08‑12 | docs: note that BASCAL is a pet project, not a production tool           |
| `d0444f9` | 08‑12 | docs: contain the hero float and soften the pet-project note             |
| `eee0f90` | 08‑12 | docs: swap "old frustration" for "old itch" in the origin story          |
| `a55da8d` | 08‑12 | docs: fix grammar, tone, and precision issues found in review            |
| `d96fe88` | 08‑13 | Add byref/byval parameter passing and multi-dimensional array parameters |
| `9c8e9b4` | 08‑13 | chore: bump version to 0.99.8 — tagged **v0.99.8**                       |
| `0052246` | 08‑13 | docs: fix remaining stale byref/copy-out references on GH Pages          |
| `966198b` | 08‑13 | docs: comment every parameter in every example, repo and GH Pages        |
| `48a3866` | 08‑13 | docs: clarify byref simulation and why parameters have no array syntax   |
| `c7fce0e` | 08‑13 | docs: record the decided (?, ?) array-parameter rank syntax design       |
| `fda32cd` | 08‑14 | Add explicit array-parameter rank syntax: arr%(?), grid%(?, ?)           |
| `e389d3e` | 08‑14 | Reject indirect recursion at compile time, not just direct self-calls    |
| `306cd1d` | 08‑14 | test: confirm indirect recursion detection sees conditional calls        |
| `707730f` | 08‑14 | chore: bump version to 0.99.9 — tagged **v0.99.9**                       |
| `99c1809` | 08‑14 | docs: explain why parameters copy in/out instead of just using globals   |
| `8478b0d` | 08‑14 | feat: add sizeof() builtin for array bounds                              |
| `afc6b46` | 08‑14 | feat: auto-inject array bounds at call sites, remove manual count params |
| `548bf58` | 08‑14 | docs: fix GH Pages tutorial snippets left stale by the sizeof() redesign |
| `763eecf` | 08‑14 | fix: array parameter storage is DIMed once, sized to a resolved capacity |

All dates 2026. Verify any of this directly: `git log --oneline 3de2e8c..763eecf`.

</div>

[← Back to Home](../)
