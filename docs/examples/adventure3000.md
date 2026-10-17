[Home](../../) / [Examples](index.md) / ADVENTURE/3000 Port

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
BASIC**, not in designing new BASCAL source from scratch. Each of its
seventeen stages is a complete, independently runnable program with its own
copy of the four data files, checked in under [`examples/adventure3000`](https://github.com/johnjoeallen/bascal/tree/main/examples/adventure3000)
-- so the port's progress can be inspected, run, and diffed stage by stage
rather than only at the end. Every stage after the first has been verified
with `bcc --check`, a real `fbc` (FreeBASIC) build, and smoke tests
diffed for byte-identical output against the previous stage.

Seventeen stages take it from that unmodified original to fully structured,
cleanly documented BASCAL, each with its own commentary and complete,
syntax-highlighted source -- start at Stage 1 below and follow each page's
Next link through to Stage 17.

## Verification

Given the game's size (over 300 rooms/items/keywords and dozens of
subroutines), it has not been exhaustively played through end to end, and
(like the other case studies under `examples/`, as opposed to `tutorial/`)
isn't part of the automated test suite -- treat it as "starts and responds
correctly to a wide range of commands," not "every path verified."

### `--target C` and `--target jvm`

- **`--target jvm` cannot run any stage.** The JVM backend doesn't
  implement `DATA`/`READ` at all yet (every stage hits this immediately,
  in the Bedquilt room-pick table), and even past that, it only supports
  `OPEN ... FOR RANDOM` (fixed-width record I/O) -- not the sequential
  `OPEN ... FOR INPUT`/`OUTPUT`/`APPEND` every stage uses to load its data
  files and to SAVE/LOAD/BUG.
- **`--target c` works for stages 10-17, not for stages 1-9.** Stages 1-9
  use classic `on error goto` at three call sites (SAVE GAME, LOAD OLD
  GAME, and BUG report), and the C backend permanently rejects `on error
  goto`/`resume`/`error` by design (GitHub issue #61) -- stages 2 and 3
  have an independent, additional blocker besides, since their
  still-`ON ... GOTO` legacy dispatch forms aren't supported by the C
  backend at all. Stage 10 switched those three call sites to BASCAL's
  portable `try`/`catch`/`finally` instead, which the C backend fully
  supports -- confirmed end to end (SAVE/LOAD/BUG all work under a real
  native `--target c` build, including the file-not-found `catch` path).
  That's a trade, not a pure upgrade, though: `try`/`catch`'s generated
  `RESUME <lineno>` is valid under real BASCOM but real `fbc` rejects it
  outright (issue #100/#153), so stages 10-17 no longer build under
  `--target fbc` the way stages 1-9 still do. `--target basic` (real
  BASCOM, this case study's primary verification target throughout) works
  for every stage regardless.

In short: no stage runs under `--target jvm` yet (tracked, in-progress
backend gaps); `--target c` works for the second half of the port
(stages 10-17) at the cost of `--target fbc`, a deliberate trade stage 10
documents in full.

The complete port, all seventeen stages, is in
[`examples/adventure3000`](https://github.com/johnjoeallen/bascal/tree/main/examples/adventure3000).

</div>

[← Adventure Game](adventure.md) [Start: Stage 1: The Original, Untouched →](adventure3000-stage1.md)
