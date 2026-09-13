[Home](../../) / [Examples](sort-driver.md) / ADVENTURE/3000 Port

<div class="prose" markdown="1">

# Case Study: Porting ADVENTURE/3000

This is a staged port of **ADVENTURE/3000**, a large 1979 HP/3000 BASIC text
adventure by Benjamin Moser (James Madison High School, Vienna, Virginia),
based on the map and gameplay of Crowther & Woods' original *Adventure*. It
was later ported to Macintosh BASIC (Elizabeth and David Hunter, 1998) and
then to [PyBasic](https://github.com/richpl/PyBasic) for the Raspberry Pi
Pico (2021), whose `examples/adventure.bas` and companion data files
(`ADESCRIP`, `AITEMS`, `AMESSAGE`, `AMOVING`) are the direct source for this
case study. PyBasic is GPL-3.0 licensed, the same license as BASCAL itself.

This is a different program from [the Huw Collingbourne adventure game case
study](adventure.md) -- that one is a small, from-scratch Delphi port used as
a language-design exercise for BASCAL's record model. This one is a large,
real historical BASIC program (1149 lines, four sequential data files,
hundreds of rooms/items/keywords), used as a case study in **porting existing
BASIC**, not in designing new BASCAL source from scratch. Each of its six
stages is a complete, independently runnable program with its own copy of the
four data files, checked in under [`examples/adventure3000`](https://github.com/johnjoeallen/bascal/tree/main/examples/adventure3000)
-- so the port's progress can be inspected, run, and diffed stage by stage
rather than only at the end. Every stage after the first has been verified
with `bcc --check`, a real `fbc` (FreeBASIC) build, and smoke tests
diffed for byte-identical output against the previous stage.

## The six stages

Each stage page below has that stage's own commentary and its complete,
unabridged, syntax-highlighted source.

- **[Stage 1: The Original, Untouched](adventure3000-stage1.md)** -- the
  starting point, checked in verbatim for provenance: 1149 lines of classic
  line-numbered BASIC, unchanged.
- **[Stage 2: Minimal Valid BASCAL](adventure3000-stage2.md)** -- the same
  program as valid `.bcl` source, changed as little as the language allows.
- **[Stage 3: Clean GOSUBs Become Procedures](adventure3000-stage3.md)** --
  every genuine, single-entry/single-exit `GOSUB` subroutine becomes a real
  `procedure`/`function`.
- **[Stage 4: The 40-Way Dispatch Becomes SELECT CASE](adventure3000-stage4.md)**
  -- the command loop's 40-way `ON z1 GOTO` dispatch becomes `SELECT CASE`,
  without needing to touch any handler's own internal `GOTO`-heavy logic.
- **[Stage 5: The Command-Parsing Cascade and Movement Engine](adventure3000-stage5.md)**
  -- eight "scan until match" loops become `WHILE`/`FOR`, and the movement
  engine becomes four functions.
- **[Stage 6: The Outer Game Loop](adventure3000-stage6.md)** -- the outer
  game loop every verb handler `GOTO`s back into becomes a nested
  `WHILE TRUE`, using BASCAL's `continue`/`exit` statements.

## Verification

Given the game's size (over 300 rooms/items/keywords and dozens of
subroutines), it has not been exhaustively played through end to end, and
(like the other case studies under `examples/`, as opposed to `tutorial/`)
isn't part of the automated test suite -- treat it as "starts and responds
correctly to a wide range of commands," not "every path verified."

### `--target C` and `--target jvm`

- **`--target jvm` cannot run any stage.** Every stage `OPEN`s its data
  files, and the JVM backend has no file I/O support at all yet.
- **`--target c` cannot currently build any stage either, but for a more
  interesting reason.** All stages use classic `on error goto` at three call
  sites (SAVE GAME, LOAD OLD GAME, and BUG report), and the C backend
  permanently rejects `on error goto`/`resume`/`error` by design (GitHub
  issue #61). BASCAL's documented portable alternative, `try`/`catch`/
  `finally`, does make every stage build and run under `--target c` --
  but its generated BASIC uses `RESUME <linenum>`, which real `fbc` rejects
  outright, a genuine pre-existing `bcc` bug (issue #100) confirmed to
  predate this port. Until #100 is fixed, no BASCAL program can use
  `try`/`catch` for error handling *and* build cleanly under real `fbc`, so
  every stage keeps `on error goto`, preserving the `fbc`-verified BASIC
  target at the cost of `--target c` support. Stages 2 and 3 have an
  independent, additional blocker: their still-`ON ... GOTO` legacy
  dispatch forms aren't supported by the C backend at all.

In short: `--target c` compatibility for this case study is blocked on two
already-tracked upstream `bcc` issues, not on anything about the port
itself -- #61 is permanent by design, #100 is a real bug that could in
principle be fixed.

The complete port, all six stages, is in
[`examples/adventure3000`](https://github.com/johnjoeallen/bascal/tree/main/examples/adventure3000).

</div>

[← Adventure Game](adventure.md) [Start: Stage 1: The Original, Untouched →](adventure3000-stage1.md)
