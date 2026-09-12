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

## The three stages

- **`stage1-original-basic/`** -- the original program, completely
  unmodified, checked in verbatim for reference and provenance.
- **`stage2-minimal-bascal/`** -- the same program as valid `.bcl` source,
  changed as little as the language actually allows. Variable names, the
  `GOSUB` spaghetti, single-letter globals, and the `DATA`/`READ` tables
  are untouched. What *did* have to change, because BASCAL genuinely has
  no equivalent, turned out to be more than expected:
  - Every numbered line becomes an `L<num>:` label -- BASCAL manages its
    own line numbers and requires named `goto`/`gosub`/`restore`/`then`/
    `else` targets, never raw line numbers.
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

  Everything else -- the actual game loop, the rest of room/command
  dispatch, combat, and puzzles -- is still exactly stage 2's label/`GOTO`/`GOSUB`
  structure, not yet refactored. Verified the same
  way as stage 2 (`bcc --check`, `fbc` build and manual smoke test) after
  each change, to confirm the refactor stayed behavior-preserving.

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
