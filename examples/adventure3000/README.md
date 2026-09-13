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

## The five stages

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

- **`stage5-refactored-bascal/`** -- picks up exactly where stage 4 left
  off, on the two things its own README section flagged as untouched.
  Started as an exact copy of stage 4 (verified identical `bcc --check`,
  `fbc` build, and smoke-test behavior before any further change).

  First pass: the command-parsing cascade between the keyword-matching
  `DATA` table and the verb dispatch's own `SELECT CASE` turned out to
  contain eight genuinely self-contained "scan until match" loops --
  keyword matching, the exotic-word/direction-word/direction-of-movement
  checks, the narrow-tunnel item check, and the two "was it a keyword/an
  item" checks run right before dispatch -- none of them touching the
  40 verb handlers' own internal logic or the movement engine below.
  All eight are now `WHILE`/`FOR` loops, verified byte-identical output
  against stage 4 for the same smoke-test input (movement, inventory,
  save/quit, and specifically exercising the converted checks:
  GET/DROP/THROW/OPEN with no match, `XYZZY`/`PLUGH`, an unrecognized
  word, and multi-word direction names).

  Two GOTO-reached mini-routines shared by multiple loops turned out
  not to be loops at all, so they became procedures instead:
  `askWhatToDoWithItem()` (was `L940`, reached from two different
  loops) and `printDontUnderstand()` (was `L2040`, reached from six
  different verb handlers' own "didn't understand" case). `printDont
  Understand()`'s own `DATA` line (`L2070`) deliberately stays at its
  original top-level position rather than moving into the procedure
  with the rest of its logic -- `RESTORE` can target a label anywhere
  in the file, and this avoids any risk to some other, unrelated
  unrestored sequential `READ` elsewhere in this 1149-line program.

  Second pass: the movement-execution engine (was `L1070` onward) is
  now four functions instead of a GOTO web, confirmed to be a de facto
  shared subroutine rather than a loop at all (`bcc`'s own
  loop-detection never flagged it) -- reached via GOTO from inside the
  movement logic itself *and* directly from `PLUGH`/`XYZZY`/`PLOVER`/
  `CLIMB`/`CROSS`/`ENTER`/`LEAVE` as a "teleport"/"retry with a new
  direction" mechanism, every one of those call sites now calling the
  same functions instead:
  - `attemptMove%(d%)` (was `L1070`) -- looks up the destination room
    for direction `d%`, handling `dirs()`'s "pick one at random"
    sentinel (`attemptRandomMove%()`, was `L1470`) and an out-of-range
    lookup itself.
  - `checkSpecialRoomAndMove%(d%, z2%)` (was `L1260` onward) -- the
    special-room/special-direction checks (grate, nugget, bridge,
    snake, narrow tunnel, troll, dragon), reachable either through
    `attemptMove%()` or directly from `ENTER`/`LEAVE`, which already
    know the destination room. A straightforward `elseif` chain, not a
    restructuring of the original logic -- every condition tests a
    mutually exclusive room number, so the original's linear GOTO
    cascade and this chain are behaviorally identical, confirmed
    byte-identical output against stage 4 across six smoke tests
    covering every branch (movement in all 8 directions, the three
    teleport verbs, `CLIMB`, `CROSS`, `ENTER`/`LEAVE`).
  - `performMove%(z2%)` (was `L1180`) -- the actual room transition.

  A function can't `GOTO` a top-level label the way the original
  GOTO-based targets could, so every call site now checks the returned
  0/1 and `GOTO`s `L400` (a message was printed) or `L9780` (the move
  happened) itself instead. Also verified with `bcc --check`, a real
  `fbc` build, and a real BASCOM compile under dosbox-x (0 severe
  errors).

  One thing remains: the outer game loop (`L300`/`L320`/`L400`/`L410`)
  every one of the 40 verb handlers `GOTO`s back into once it's done --
  almost all of `bcc`'s remaining warnings on this file are sites
  jumping back into it, not loops of their own. Converting it means
  restructuring the whole file's top-level control flow, not a
  per-handler change -- flagged as **stage 6**'s job, not attempted
  here.

- **`stage6-refactored-bascal/`** -- picks up exactly where stage 5 left
  off: the outer game loop. Started as an exact copy of stage 5.

  The outer redisplay loop (was `L300`/`L320`) and the inner "get one
  command" loop (was `L400`/`L410`) are now a `WHILE TRUE` nested
  inside another. This became possible only once BASCAL grew a
  `continue` statement (added alongside this stage's own work,
  unqualified like `exit` -- the transpiler resolves which enclosing
  loop it leaves): `continue` for a verb handler that wants another
  command without redisplaying (was `GOTO L400`/`GOTO L410`), `exit`
  for one that wants the room redisplayed (was a direct `GOTO L300`, or
  the pit-check's own `GOTO L9780` once no pit was hit). `L300`/`L400`/
  `L410` stay real labels rather than disappearing entirely -- a
  handful of stage 5's own scan loops need to jump out two loop levels
  at once to reach them, which `continue`/`exit` can't do (each only
  ever escapes its own innermost loop), so those sites keep an explicit
  `GOTO` instead.

  The reincarnation/pit-death cascade (was `L9540`-`L9840`) is now two
  procedures, `reincarnate()` and `checkPitsAndReincarnateIfNeeded()`,
  plus a third, `endGame()` (was `L9750`'s "Oh well..."/`printScore()`/
  `STOP`), pulled out separately once it turned out two verb handlers
  outside the cascade entirely -- QUIT's "don't save" path and SAVE
  GAME's own "also quit" check -- shared that same target.

  Verified with `bcc --check`, a real `fbc` build, and smoke tests
  against stage 5 covering movement, inventory, GET/DROP, SAVE/LOAD,
  QUIT with and without saving, SCORE, SHORT/LONG/BRIEF, an exotic word
  (XYZZY), an item named with no verb, JUMP, ATTACK, and CLIMB --
  byte-identical output throughout. The pit-fall/reincarnation cascade
  itself was exercised separately in a scratch copy with the
  darkness/pit checks temporarily patched to trigger on an early room,
  confirming both death messages, a successful reincarnation, and a
  "no" answer correctly ending the game via `endGame()`.

- **`stage7-refactored-bascal/`** -- reducing GOTO to a minimum inside the
  40 verb handlers themselves, and cleaning up the resulting massive
  `SELECT CASE` -- picks up where stage 6 left off. **In progress**, not a
  finished pass. Started as an exact copy of stage 6.

  Three kinds of GOTO cleaned up so far, all without changing a single line
  of actual game logic: small shared message-and-continue targets embedded
  in one case but reached by `GOTO` from several others (e.g. `L2200`'s
  `printMessage(2)`, 17 call sites) are now duplicated inline instead of
  shared by label -- simpler than a procedure for a one- or two-statement
  body, though LIGHT/OFF's bigger shared "redo LOOK's dark-room check"
  became a real procedure, `describeRoomForLook()`; ENTER's and LEAVE's own
  "try every direction" manual scan loops became real `FOR` loops, the same
  way stage 5 converted the command-parsing cascade's; and GET and DROP --
  by far the two most GOTO-heavy handlers -- are now each a single
  structured `FOR z3 = 1 TO T2` scan, with their own "special case" logic
  (chain, bear, dragon, bird-in-cage, bottle, oil/water, the vase, and
  FEED's own reuse of DROP's tail) folded directly into the loop body as
  `IF`/`ELSEIF` instead of living as separate GOSUB-like targets reached
  mid-scan.

  A real bug turned up along the way: converting LOCK/UNLOCK's shared
  "you don't have the keys" check to structured `IF` initially inverted the
  `S(19) = -1` ("carrying") test, silently swapping which branch UNLOCK
  took -- caught immediately by this stage's own smoke tests, fixed, and
  reverified byte-identical.

  Verified with `bcc --check`, a real `fbc` build, and smoke tests against
  stage 6 covering movement, inventory, SAVE/LOAD, QUIT, SCORE,
  SHORT/LONG/BRIEF, an exotic word, JUMP, ATTACK, CLIMB, LIGHT/OFF/LOOK,
  FILL/EMPTY, LOCK/UNLOCK, FREE, WAVE, OPEN/CLOSE, OIL, and FEED/WATER/
  THROW -- byte-identical throughout. GET's and DROP's own special-case
  branches were each exercised separately in a scratch copy with the
  relevant items patched to be carried/present from the start.

  **Where this stops for now**: the 40-case `SELECT CASE` dispatch is still
  one massive block with every handler's logic inline, not yet split into
  its own procedure per verb. Extracting each case into a procedure isn't
  purely mechanical -- a procedure can only `return` to its own caller,
  never `continue`/`exit` a loop in the *caller's* scope the way every
  handler here does directly today, so each extracted verb procedure would
  need to return a code (get another command vs. redisplay) for the
  dispatcher to act on after the call -- the same restructuring problem
  stage 3's own README section hit and stage 4 deliberately routed around.
  A few more small GOTO targets also remain unconverted. Left as clearly-
  marked future work rather than attempted partially.

- **`stage8-refactored-bascal/`** -- splitting the 40-case `SELECT CASE`
  dispatch into its own procedure per verb, the piece of stage 7's own goal
  that stage explicitly stopped short of. Started as an exact copy of
  stage 7.

  Every verb handler is now its own `function verbXxx%()`, returning 0
  ("get another command") or 1 ("redisplay the room") for the `SELECT
  CASE` -- now a lean one-line-per-case dispatch table -- to act on with
  `continue`/`exit` itself, replacing the direct `continue`/`exit` every
  handler used when it was inline. This turned out to simplify more than
  it complicated: a `return` unwinds the whole function regardless of loop
  nesting, so GET's, DROP's, ENTER's, and LEAVE's own scan loops no longer
  need stage 6/7's two-loop-levels `GOTO L300`/`GOTO L400` workaround --
  every one of those became a plain `return 0`/`return 1` instead.

  A few handlers' own GOTOs crossed into what's now a different function
  entirely (labels are function-scoped, so a GOTO can no longer reach
  across): QUIT's "yes, save first" path and OPEN's/CLOSE's "it's actually
  a lock" cases became direct calls to `verbSaveGame%()`/`verbUnlock%()`/
  `verbLock%()` instead, using the same 0/1 convention. BUG's error
  handler used to reuse LOAD's own error label the same way; since `on
  error goto`'s target must live in the same function, it now has its own
  copy of the same message. SCORE's own `DATA` line moved to the top
  level, since `RESTORE` targets must stay top-level (issue #149/PR #150)
  and can no longer live inside `verbScore%()` with the rest of that
  case's logic.

  Every verb function declares `global` for each shared variable it
  touches; missing even one silently creates a fresh, always-zero local
  instead of erroring, so this was checked both by an automated scan
  (every scalar/array name known to be global anywhere in the program,
  cross-referenced against each function's own `global` list) and by the
  smoke tests -- two real omissions (`t2` in GET and DROP) were caught
  this way before ever reaching a manual test.

  Verified with `bcc --check`, a real `fbc` build, and the same smoke
  tests as stage 7 against a stage 7 baseline -- byte-identical
  throughout, including GET/DROP/FEED's special-case branches and the
  pit-fall/reincarnation cascade, each exercised separately in a scratch
  copy with the relevant items/rooms patched. `bcc`'s own hand-wired-loop
  warnings on this file are down to 5, all the outer loop's own
  intentional labels plus one small remaining leftover (CROSS's own
  message) -- not attempted here, since this stage's own goal is
  otherwise complete.

- **`stage9-refactored-bascal/`** -- modernizing the verb functions' own
  internal style, now that stage 8 has given every verb its own function.
  Started as an exact copy of stage 8.

  Every verb function's own `GOTO`-chained branches (classic one-line `IF
  cond THEN goto Label` chains, and the ALL-CAPS labels they jumped to,
  read verbatim from stage 2) became structured, lowercase `IF`/`ELSEIF`/
  `END IF` -- the same style the rest of this port has used since stage 3.
  Several near-duplicate branches the original's labels converged on
  folded into shared fall-through or a single `OR` condition instead --
  PLUGH/XYZZY's two teleport-or-fail tails, ENTER/LEAVE's "house"/"barren
  room" cases. `ON ERROR GOTO` (SAVE GAME, LOAD OLD GAME, BUG) stays
  exactly as-is, with no structured equivalent; CLOSE's own dead-code
  branch (a preserved upstream bug) was left alone too, since there's no
  live code there to modernize.

  A real bug turned up along the way: converting PLOVER's "was he
  carrying the emerald" check to structured `IF` inverted the `S(10) =
  -1` ("carrying") test the same way stage 7's LOCK/UNLOCK bug did --
  caught by this stage's own smoke tests (a missing room-contents line
  after teleporting via PLOVER), fixed, and reverified byte-identical.

  Verified with `bcc --check`, a real `fbc` build, and the stage 8 smoke
  suite against a stage 8 baseline -- byte-identical throughout. `bcc`'s
  own hand-wired-loop warnings are down to 4 (all the outer loop's own
  intentional labels); total GOTO count in the file down from 105 to 59,
  nearly all of what's left being `ON ERROR GOTO` or comments documenting
  the port's own history.

- **`stage10-refactored-bascal/`** -- replacing SAVE GAME's, LOAD OLD
  GAME's, and BUG's `ON ERROR GOTO` with `TRY`/`CATCH`, BASCAL's portable
  error model. Started as an exact copy of stage 9.

  A genuine trade, not a pure upgrade: `on error goto` builds under both
  `--target basic` (real BASCOM) and `--target fbc`, but is permanently
  rejected under `--target c` (issue #61). `try`/`catch` builds under
  `--target basic` and `--target c`, but `--target fbc` now rejects it
  outright at compile time (issue #153 -- the same underlying limitation
  issue #100 originally reported, now diagnosed instead of just failing
  at `fbc`'s own compile step). Converting gains `--target c` for the
  first time in this case study's history, at the cost of `--target fbc`,
  every earlier stage's primary verification backend.

  Unblocking `--target c` surfaced one unrelated, genuinely pre-existing
  limitation: `checkSpecialRoomAndMove%()`'s troll-state `SELECT CASE`
  had no `CASE ELSE`, and the C backend requires every function to
  visibly return on every path -- never reached before, since `on error
  goto` always failed compilation first. Fixed with a `CASE ELSE`
  matching `CASE 4`'s own body; provably unreachable, not a behavior
  change.

  Verified three ways: `bcc --check`; a real BASCOM build (headless,
  under `dosbox-x`) of both this stage and stage 9, run against identical
  scripted input (movement, SAVE, LOAD, BUG, and a LOAD of a nonexistent
  file to exercise the `catch`/`on error goto` path itself) with stdin/
  stdout redirected inside the DOS batch file -- byte-identical output
  between the two stages in both the success and failure cases; and a
  real native `--target c` build, run interactively, confirming SAVE/
  LOAD/BUG all work correctly under the backend this stage exists to
  unblock, including the file-not-found `catch` path.

- **`stage11-refactored-bascal/`** -- a small mop-up on stage 9's own goal:
  modernizing `checkDwarfAttack()`, the one procedure stage 9 missed (it was
  scoped to the 40 `verbXxx%()` functions only; this helper is called from
  `checkDwarf()` and `verbThrow%()`, not a verb function itself). Started as
  an exact copy of stage 10.

  `checkDwarfAttack()`'s `GOTO`-chained branches became `IF`/`ELSEIF`/
  `ELSE`, the same treatment stage 9 gave every verb function. A sweep of
  every other procedure and function in the file turned up nothing else --
  this genuinely was the one leftover. The remaining `GOTO`s in the file
  are either the outer game loop's own intentional `L300`/`L400`/`L410`
  labels (stage 6), or the command-parsing cascade's own shared re-entry
  labels (`L500`/`L1950`/`L2090`, from stage 5) -- that cascade's internal
  structure is its own, larger, not-yet-attempted piece of work.

  Verified with `bcc --check` and the stage 10 smoke suite (run under
  `--target c`, now that it's available, rather than `fbc`) against a
  stage 10 baseline -- byte-identical throughout, including a scratch copy
  with an axe and a dwarf forced into the starting room to exercise
  `checkDwarfAttack()`'s own branches specifically, confirmed across
  several different room-entry counts to vary which point in the
  random-number sequence each run consumes.

- **`stage12-refactored-bascal/`** -- the command-parsing cascade (keyword
  matching, then dispatch) becomes two functions of its own, the last piece
  of this port's GOTO-reduction work. Started as an exact copy of stage 11.

  `dispatchVerb%(z1%)` is the old 40-case `SELECT CASE` moved verbatim into
  its own function. `parseAndDispatchCommand%()` is everything from stage
  6's own `L400` onward: read a line, split it, match keywords, call
  `dispatchVerb%()`. Both return 0/1, the same convention every `verbXxx%()`
  function already uses. This eliminates every `GOTO` this stretch needed
  purely to escape two loop levels at once -- `L300`, `L400`/`L410`, and
  the scan loops' own `GOTO`s to them -- the same way extracting the verb
  handlers did for `GET`/`DROP`/`ENTER`/`LEAVE` in stage 8: a `return`
  unwinds a function regardless of loop nesting. `L1950`'s own redundant
  re-scan (the original verb-word check never kept its own matched index,
  so a second scan had to re-find it) is gone too -- the new `FOR` just
  captures its index directly.

  The keyword-table `DATA` couldn't move into these new functions with the
  code that reads it: `RESTORE` targets must stay top-level, and
  `--target c` needed this literally, not just as style -- it rejected
  `RESTORE` inside the new function outright, a genuine backend limitation
  `--target basic` didn't share.

  `bcc`'s own hand-wired-loop warnings are down to **zero** for the first
  time in this port's history -- every `GOTO` left in the file is inside a
  comment. Verified with `bcc --check`; the stage 11 smoke suite
  (`--target c`) against a stage 11 baseline -- byte-identical, covering
  every keyword class; and, since this touches the keyword parser itself,
  a real BASCOM run (headless, under `dosbox-x`) of both stages against a
  longer scripted session -- byte-identical except for BASCOM's own
  end-of-program `STOP in line N` diagnostic, which necessarily differs
  since code moved around (confirmed by diffing everything before that
  line, which matched exactly).

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

All three stages (2, 3, and 4) are re-verified the same way -- `bcc --check`,
an `fbc` build, and the same smoke test -- after every refactoring pass, and
all three currently build and run correctly.

### `--target C` and `--target jvm`

Besides the `fbc`/BASIC-target verification above, each stage was also
checked against BASCAL's other two backends, with mixed results:

- **`--target jvm` cannot run any stage.** Every stage `OPEN`s its data
  files (`AMOVING`, then `ADESCRIP`/`AITEMS`/`AMESSAGE`), and the JVM
  backend has no file I/O support at all yet -- `bcc` reports `Open { .. }
  is not supported by the minimal JVM backend yet` at the first `OPEN`.
  This isn't fixable from the example side; it's a capability the JVM
  backend simply doesn't have yet.

- **`--target c` cannot currently build any stage either, but for a more
  interesting reason.** All three stages use classic `on error goto` at
  three call sites (SAVE GAME, LOAD OLD GAME, and BUG report, which share
  a handler), and the C backend permanently rejects `on error goto`/
  `resume`/`error` by design (see GitHub issue #61) -- its GOSUB/return-
  address-stack model can't safely cross real C function boundaries, so
  there's no near-term fix planned there. BASCAL's documented portable
  alternative is `try`/`catch`/`finally`, and converting those three sites
  to it does make every stage build and run under `--target c` (stage 4
  was verified compiling and playing correctly this way). It was reverted
  in all three stages, though: `try`/`catch`'s generated BASIC output uses
  `RESUME <linenum>`, which real `fbc` rejects outright (`error 3:
  Expected End-of-Line`) -- a genuine, pre-existing bcc bug, already
  tracked as GitHub issue #100, and confirmed (by testing the project's
  own `tutorial/portable_error_handling.bcl` under real `fbc`) to predate
  this port entirely. Until #100 is fixed, there is no way to make a
  BASCAL program use `try`/`catch` for error handling *and* build cleanly
  under real `fbc`, so all three stages keep `on error goto` -- preserving
  the `fbc`-verified BASIC target, at the cost of `--target c` support --
  with a comment at each site pointing at #61 and #100.

  Stage 2 and stage 3 have an independent, additional `--target c`
  blocker even setting the above aside: their still-`ON ... GOTO`/
  `ON ... GOSUB` legacy dispatch forms aren't supported by the C backend
  (`OnBranch { .. } is not supported by the minimal C backend yet`) --
  only stage 4's `SELECT CASE` conversion (see above) avoids this, so
  `--target c` support, if #100 is ever fixed, would land as a stage-4-
  only milestone rather than something retrofittable to stage 2/3 without
  their own `SELECT CASE` conversion.

  In short: `--target c` compatibility for this case study is currently
  blocked on two already-tracked upstream bcc issues, not on anything
  about the port itself -- #61 is permanent by design, #100 is a real bug
  that could in principle be fixed.

Stage 5 started as an exact copy of stage 4 and hasn't diverged from it
yet, so everything above about stage 4 -- `--target basic`/`bascom`
verified against real BASCOM, `--target fbc`/`--target c`/`--target jvm`
each blocked the same way -- currently applies to it identically.

**Stage 10 changes this for itself only.** Every stage through 9 keeps
`on error goto`, so everything above still applies to stages 1-9
unchanged. Stage 10 converts to `try`/`catch` instead (see its own bullet
above), which flips its own `--target c`/`--target fbc` status: it builds
and runs correctly under `--target c` (verified interactively, including
the file-not-found `catch` path) for the first time in this case study,
but `--target fbc` now rejects it outright at compile time (issue #153).
`--target basic` (real BASCOM) keeps working for stage 10, same as every
other stage; `--target jvm` remains blocked for the same file-I/O reason
as always.
