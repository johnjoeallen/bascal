# Case Study: ADVENTURE/3000

A staged port of **ADVENTURE/3000**, a large 1979 HP/3000 BASIC text
adventure by Benjamin Moser (James Madison High School, Vienna, Virginia),
based on the map and gameplay of Crowther & Woods' original Adventure. It
was later ported to Macintosh BASIC (Elizabeth and David Hunter, 1998) and
then to [PyBasic](https://github.com/richpl/PyBasic) for the Raspberry Pi
Pico (2021), whose `examples/adventure.bas` and companion data files
(`ADESCRIP`, `AITEMS`, `AMESSAGE`, `AMOVING`) are the direct source for this
case study. PyBasic is GPL-3.0 licensed, the same license as BASCAL itself.

This is a different program from [the Huw Collingbourne adventure game
case study](../adventure/) (`examples/adventure/`) -- that one is a small,
from-scratch Delphi port used as a language-design exercise for BASCAL's
record model. This one is a large, real historical BASIC program (1149
lines, four sequential data files, hundreds of rooms/items/keywords), used
as a case study in **porting existing BASIC**, not in designing new BASCAL
source from scratch.

## The four stages

- **`stage1-original-basic/`** -- the original program, completely
  unmodified, checked in verbatim for reference and provenance.
- **`stage2-minimal-bascal/`** -- the same program as valid `.bcl` source,
  changed as little as the language actually allows. Variable names, the
  `GOSUB` spaghetti, single-letter globals, and the `DATA`/`READ` tables
  are untouched. What *did* have to change, because BASCAL genuinely has
  no equivalent, turned out to be more than expected:
  - Every numbered line that's an actual `goto`/`gosub`/`restore`/
    `then`/`else`/`on ... goto`/`on ... gosub` target keeps an
    `L<num>:` label matching its original line number -- BASCAL manages
    its own line numbers and requires named targets, never raw line
    numbers -- and lines nothing jumps to don't carry one.
  - `REM` isn't a BASCAL comment keyword at all (only `'`, `//`, `/* */`
    are) -- every `REM` became `'`.
  - BASCAL's `for` loop has no raw `NEXT`-terminated form -- it's always
    a structured `for ... end for` block. Most of the original's ~30
    loops convert to that directly, but around a third jump out of the
    loop body mid-iteration via `GOTO` (classic spaghetti-BASIC), which a
    structured block can't represent -- those became manual
    `X = A : L: if X > B then goto Lend ... X = X + step : goto L : Lend:`
    loops instead, verified individually to actually be jumped out of
    before converting them (see the stage's own source comments).
    **A related, initially-missed bug**: several loops (both the
    structured and the manual-while kind) also contain an internal
    `GOTO` to their own `NEXT` line as a plain "skip the rest of this
    iteration" continue -- a universal classic-BASIC idiom, since a
    `NEXT` line's own machinery is exactly "increment, then loop back if
    not done." A naive conversion placed the loop's exit label *after*
    `end for` (or after the increment+loop-back, for the manual-while
    form) to serve external "skip past the whole loop" references,
    which silently turned every such continue into a full early
    `break` instead -- e.g. the inventory listing (`FOR X=1 TO T2`
    around line 3490) would stop at the *first* uncarried item and
    never show anything carried after it. Fixed by inverting the guard
    condition into a real `if`/`end if` wrapping the skipped statements
    (for structured loops), or adding a separate `LCONT<n>:` label right
    at the increment step that continues jump to instead of the true
    exit (for the manual-while loops) -- 15 loops across both stages
    needed one of these two fixes; see the commit history for the
    full list.
  - The original's `FSEEK`-based random access into `ADESCRIP`/`AITEMS`/
    `AMESSAGE` (plus a hand-rolled `AMESSAGE.IDX` byte-offset cache) has
    no BASCAL or classic-BASIC equivalent at all -- neither supports
    seeking within a sequential file. All three are now read fully into
    string arrays once at startup, indexed by array position instead of
    byte offset; the on-disk index cache is gone entirely.
  - `OPEN ... ELSE <linenum>` (classic HP BASIC's inline open-failure
    branch) isn't supported -- rewritten as `on error goto`/`on error
    goto 0` around a plain `OPEN`.
  - `UPPER$`/`LOWER$` are PyBASIC-specific; real BASIC (and BASCAL's own
    `basic`-target output) has neither -- rewritten to BASCAL's stdlib
    `UCASE$`/`LCASE$` (`require com.bascal.stdlib.ucase`/`.lcase`).
  - `MIN` isn't a `basic`-target builtin; the one call site's `min(x, 5)`
    was provably a no-op (`x` was always 0-4) and was simplified away.
  - One dead call to an undefined `FNA(25)` -- no matching `DEF FN`
    exists anywhere in the upstream source, so this looks like leftover
    breakage in PyBasic's own port rather than something this port
    introduced -- was dropped; its result was never read before being
    reassigned anyway.

  Verified with `fbc` (FreeBASIC): the game compiles, starts, prints its
  intro and initial room, accepts movement/`LOOK`/`INVENTORY`/`QUIT`
  commands, and exits cleanly. It has not been exhaustively played
  through -- see "Verification" below.
- **`stage3-refactored-bascal/`** -- stage 2 restructured, piece by piece,
  toward BASCAL's own constructs. **This is a genuine work in progress,
  not a finished rewrite** -- a program this size (1149 lines, ~150
  `GOSUB` call sites, extensive shared global state) is a large refactor
  to carry all the way through. So far:
  - The startup file-loading sequence (previously inline code under
    `LA1xx` labels) is five named procedures --
    `loadAdescrip()`/`loadAitems()`/`loadAmessage()`/
    `buildMessageIndex()`/`loadItemNames()` -- called explicitly from the
    top of the program, each declaring `global` on the arrays it fills.
  - The "seek a yes or no" `GOSUB` is now a real function,
    `askYesNo%()`, called as `z0 = askYesNo%()` at its 3 call sites,
    replacing the implicit-global-`z0`-as-return-value convention.
  - The message-printing convention -- `z59 = N : gosub 7620`, at 91
    call sites -- is now a real procedure, `printMessage(N)`, taking the
    message number as an argument instead of a global set just before
    the call. The `z59` global is gone from the program entirely.
  - The room-description dispatch (`on d0+1 gosub` across three GOSUB
    targets) is now `select case d0` calling three named procedures --
    `shortDescription()`, `longDescription()`, and
    `describeRoomOnEntry()` (which itself calls the first two). One call
    site (`LOOK`) used to jump directly into the *middle* of the old
    long-description subroutine to skip its `v(l1) = 1` line; that's
    provably a no-op by the time `LOOK` is typeable (the room's already
    been entered and marked visited by then), so it now just calls
    `longDescription()` in full -- documented at that call site.
  - The item-listing, dwarf-check, and pirate-check `GOSUB`s are now
    `describeRoomContents()`, `checkDwarf()`, and `checkPirate()`.
    `checkDwarf()` keeps its original internal labels and `GOTO`s almost
    verbatim (just `return` in place of the old shared exit label);
    `checkPirate()`'s two small item-scanning loops turned out to have
    the continue/break bug described in stage 2's own entry above, so
    restructuring them into real `for`/`if` blocks fixed that as a side
    effect. One more mid-subroutine jump turned up here too: a
    `WAVE`-command call site (`GOSUB 8650`) enters only the "should the
    dwarf attack" half of the original dwarf subroutine, deliberately
    skipping the axe-giving check -- so that half is its own procedure,
    `checkDwarfAttack()`, called both by that site directly and by
    `checkDwarf()` internally.

  **Where this stops, deliberately, for now**: everything refactored
  above shared one property that made it low-risk to extract -- each was
  a genuine `GOSUB ... RETURN` subroutine, single-entry and (once
  mid-subroutine jumps like the two above were accounted for)
  effectively single-exit, which maps directly onto a BASCAL
  `procedure`/`function` and a plain `return`. The bulk of what's left
  -- the command loop itself, starting around `GET`/`DROP` and covering
  combat and puzzles -- is a different shape: a ~700-line flat `GOTO`
  chain entered from the keyword dispatcher, with handlers that exit to
  *several different* downstream labels depending on the path taken
  (`GOTO L400` normally, but `GOTO L410`, `GOTO L6880`, `GOTO L7380`,
  ... on other branches). A plain procedure call can only `return` to
  one place, so wrapping this properly means restructuring how the
  whole command loop hands control to its handlers (e.g. each handler
  returning a code or setting a flag the caller branches on) -- a much
  larger, higher-risk change than anything done so far, closer in size
  to everything above combined. Left as clearly-marked future work
  rather than attempted partially.

  Verified the same way as stage 2 (`bcc --check`, `fbc` build and
  manual smoke test) after each change, to confirm the refactor stayed
  behavior-preserving.
- **`stage4-refactored-bascal/`** -- picks up where stage 3 left off, on
  exactly the command loop it stopped short of. The insight that made
  this tractable: the loop's main dispatch is a single 40-way
  `ON z1 GOTO <40 labels>` (the exact anti-pattern `bcc` itself warns
  about, recommending `SELECT CASE`) -- and a `GOTO` can jump out of a
  `SELECT CASE` branch to anywhere, same as it always could out of a
  flat `IF`. So each of the 40 verb handlers' own internal `GOTO`-heavy
  logic could stay **completely untouched**, just given its own `case N`
  boundary, without the multi-exit-point problem stage 3 stopped at --
  no handler needed to become a separate procedure at all.

  It wasn't purely mechanical, though: `SELECT CASE` requires each
  case's code to be a genuinely separate block, and auditing all 40
  targets turned up two places where this 1979 codebase's original
  authors reused code between what are dispatched as two different
  verbs:
  - Case 39 (`YES`, meaningful only in the dragon's lair) was a small
    handler physically embedded *inside* case 18's (`ATTACK`) body --
    reachable only via the dispatch table, never by `ATTACK`'s own
    fall-through -- so it was relocated next to case 40, its neighbor in
    dispatch order, with a comment explaining the move.
  - Cases 25 and 26 (`OPEN`/`CLOSE`) literally shared one line: the
    dispatch table points `CLOSE`'s entry at `OPEN`'s own final
    `GOTO L400`, rather than at `CLOSE`'s real logic sitting right
    below it -- which is consequently dead code, unreachable from
    anywhere. This is a genuine bug in the original game (`CLOSE`
    silently does nothing), not something any stage of this port
    introduced. In keeping with every other stage's rule of preserving
    *behavior*, not "fixing" gameplay, `CLOSE` still does nothing here
    too -- just via its own copy of that line instead of overlapping
    `OPEN`'s, so each verb has a proper case block.

  The rest of the file is untouched: `select case z1` now replaces the
  `on z1 goto ...` line and the `goto L2040` it used to fall through to
  on an out-of-range `z1` (now `case else`), and nothing inside any of
  the 40 handlers changed. Verified with `bcc --check`, an `fbc` build,
  and smoke tests exercising several different verbs across the
  dispatch range (`LOOK`, `GET`, `PLUGH`, `XYZZY`, `INVENTORY`, `SCORE`).

  A second, smaller `ON t+1 GOTO` (4-way, troll state) got the same
  treatment. With both converted, `bcc` no longer warns about a
  computed-branch dispatch anywhere in the file.

  A pass verifying that no `GOTO` (in the new `SELECT CASE` blocks or
  anywhere else) jumps into the middle of a procedure or a still-GOSUB-
  based subroutine turned up one real, pre-existing bug: a `THROW`-at-
  dwarf call site missed when `checkDwarf()`/`checkDwarfAttack()` were
  split out, left calling a label that had become a dead comment --
  fixed in both stage 3 and stage 4. All the labels left over from
  stage 2's original mechanical "every line gets a label" conversion
  that nothing actually jumps to were also stripped, across all three
  `.bcl` stages, leaving only genuine branch targets labeled. All three
  stages were then re-indented (4 spaces per block-nesting level,
  matching `tutorial/*.bcl`'s own convention) now that most of the
  original's one-label-per-line noise was gone -- purely cosmetic,
  confirmed by diffing the transpiled BASIC output before and after.

  Five more clean `GOSUB` subroutines became procedures:
  `situationDescriptions()` (the grate/bridge/plugh/troll/bear/pit
  one-off messages checked after every room description),
  `findMatchedItems()` (by far the most-called of the remaining
  subroutines at 8 call sites -- works out which item, if any, the
  player's command named), `findFirstNamedItem()` (names *something*
  the player mentioned, for messages that don't care which), and
  `checkCarryingItem()`/the `computeScore()`/`printScore()` pair
  (the `SCORE` command and the two end-of-game summaries). The score
  pair fixed one small wart along the way: stage 2 reused the global
  `z9` (rooms-visited count) for an unrelated tier-lookup calculation
  right after printing it, matching the original -- `printScore()` uses
  its own local variable for that instead, since nothing was ever
  reading `z9`'s clobbered value afterward anyway.

  `RESTORE` isn't a control-flow statement, so it falls under neither
  goto/gosub rule above -- but `computeScore()`/`printScore()` still
  `restore` to two `DATA` lines kept at the top level (rather than
  moved into either procedure), just to keep them visibly close to the
  rest of the game's `DATA` tables.

Each stage is a complete, independently runnable program with its own copy
of the four data files, so the port's progress can be checked stage by
stage rather than only at the end.

## Verification

Stage 2 has been checked with `bcc --check`, transpiled to BASIC, and built
and run with `fbc` (FreeBASIC): the intro, initialization, room
descriptions, movement between rooms, `LOOK`, `INVENTORY`, and `QUIT` (with
its own save-game prompt) all work as expected in manual smoke testing.
Given the game's size (over 300 rooms/items/keywords and dozens of GOSUB
subroutines), it has not been exhaustively played through end to end, and
(like the other case studies under `examples/`, as opposed to `tutorial/`)
isn't part of the automated test suite -- treat it as "starts and responds
correctly to basic commands," not "every path verified."
