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

## Stage 1: the original, untouched

The starting point, checked in verbatim for reference and provenance: 1149
lines of classic line-numbered BASIC, `GOSUB`/`GOTO` spaghetti, single-letter
globals, and four sequential `DATA` files loaded with `FSEEK`-based random
access. Nothing is changed here -- it exists so every later stage has
something exact to diff against.

## Stage 2: the same program, as minimal valid BASCAL

The same program as valid `.bcl` source, changed as little as the language
actually allows -- variable names, the `GOSUB` spaghetti, and the `DATA`/
`READ` tables are all untouched. What *did* have to change, because BASCAL
genuinely has no equivalent, turned out to be more than expected:

- Every numbered line that's an actual branch target keeps an `L<num>:`
  label matching its original line number -- BASCAL manages its own line
  numbers and requires named targets, never raw line numbers.
- `REM` isn't a BASCAL comment keyword (only `'`, `//`, `/* */` are) --
  every `REM` became `'`.
- BASCAL's `for` loop has no raw `NEXT`-terminated form -- it's always a
  structured `for ... end for` block. Most of the original's ~30 loops
  convert directly, but around a third jump out of the loop body
  mid-iteration via `GOTO` (classic spaghetti-BASIC), which a structured
  block can't represent -- those became manual `X = A : L: if X > B then
  goto Lend ... X = X + step : goto L : Lend:` loops instead.
- A related, initially-missed bug: several loops also contain an internal
  `GOTO` to their own `NEXT` line as a plain "skip the rest of this
  iteration" continue -- a universal classic-BASIC idiom, since a `NEXT`
  line's own machinery is exactly "increment, then loop back if not done."
  A naive conversion silently turned every such continue into a full early
  `break` instead (e.g. the inventory listing would stop at the *first*
  uncarried item). Fixed across 15 loops, either by inverting the guard
  into a real `if`/`end if`, or by adding a separate `LCONT<n>:` label at
  the increment step for the manual-loop cases.
- The original's `FSEEK`-based random access into `ADESCRIP`/`AITEMS`/
  `AMESSAGE` has no BASCAL or classic-BASIC equivalent -- neither supports
  seeking within a sequential file. All three are now read fully into
  string arrays once at startup, indexed by array position.
- `OPEN ... ELSE <linenum>` (classic HP BASIC's inline open-failure branch)
  became `on error goto`/`on error goto 0` around a plain `OPEN`.
- PyBASIC-specific `UPPER$`/`LOWER$` became BASCAL's stdlib `UCASE$`/
  `LCASE$`. A no-op `MIN` call and a dead call to an undefined `FNA(25)`
  (apparent leftover breakage in PyBasic's own port) were both dropped.

Verified with `fbc`: the game compiles, starts, prints its intro and initial
room, and accepts movement/`LOOK`/`INVENTORY`/`QUIT` commands.

## Stage 3: the clean `GOSUB` subroutines become procedures

Stage 2 restructured, piece by piece, toward BASCAL's own constructs --
**a genuine work in progress, not a finished rewrite**, since a program this
size (1149 lines, ~150 `GOSUB` call sites, extensive shared global state) is
a large refactor to carry through all at once. This stage converts everything
that was a genuine `GOSUB ... RETURN` subroutine -- single-entry and
(once a couple of mid-subroutine jumps were accounted for) effectively
single-exit, which maps directly onto a BASCAL `procedure`/`function` and a
plain `return`:

- The startup file-loading sequence becomes five named procedures --
  `loadAdescrip()`, `loadAitems()`, `loadAmessage()`, `buildMessageIndex()`,
  `loadItemNames()`.
- The "seek a yes or no" `GOSUB` becomes `askYesNo%()`.
- The `z59 = N : gosub 7620` message-printing convention (91 call sites)
  becomes `printMessage(N)` -- the `z59` global disappears entirely.
- The room-description dispatch becomes `select case d0` calling
  `shortDescription()`, `longDescription()`, and `describeRoomOnEntry()`.
- The item-listing, dwarf-check, and pirate-check subroutines become
  `describeRoomContents()`, `checkDwarf()`, and `checkPirate()`.
  `checkDwarf()`'s dwarf-attack half is further split into its own
  `checkDwarfAttack()`, since one `WAVE`-command call site jumps directly
  into just that half of the original subroutine.

**Where this stops, deliberately, for now**: the bulk of what's left -- the
command loop itself, covering combat and puzzles -- is a different shape: a
~700-line flat `GOTO` chain entered from the keyword dispatcher, with
handlers that exit to *several different* downstream labels depending on the
path taken. A plain procedure call can only `return` to one place, so
wrapping this properly means restructuring how the whole command loop hands
control to its handlers -- left as clearly-marked future work (stage 4)
rather than attempted partially here.

## Stage 4: the 40-way dispatch becomes `SELECT CASE`

The insight that made the command loop tractable: its main dispatch is a
single 40-way `ON z1 GOTO <40 labels>` -- the exact anti-pattern `bcc` itself
warns about, recommending `SELECT CASE` -- and a `GOTO` can jump out of a
`SELECT CASE` branch to anywhere, same as it always could out of a flat
`IF`. So each of the 40 verb handlers' own internal `GOTO`-heavy logic stays
**completely untouched**, just given its own `case N` boundary -- no handler
needed to become a separate procedure at all.

It wasn't purely mechanical: auditing all 40 targets turned up two places
where the original authors reused code between two different verbs --
`YES` (meaningful only in the dragon's lair) was physically embedded inside
`ATTACK`'s own body, and `OPEN`/`CLOSE` literally shared one line (a genuine
pre-existing bug -- `CLOSE` silently does nothing in the original game, and
still does nothing here, just via its own copy of the line instead of
overlapping `OPEN`'s, so each verb gets a proper `case` block). A second,
smaller 4-way `ON t+1 GOTO` (troll state) got the same treatment. A pass
verifying no `GOTO` jumps into the middle of a procedure turned up one real
pre-existing bug -- a `THROW`-at-dwarf call site left calling a label that
had become a dead comment -- fixed here and backported to stage 3.

Five more clean `GOSUB` subroutines became procedures in this pass:
`situationDescriptions()`, `findMatchedItems()`, `findFirstNamedItem()`,
`checkCarryingItem()`, and the `computeScore()`/`printScore()` pair. All
three `.bcl` stages were also re-indented (4 spaces per block-nesting level)
now that most of the original's one-label-per-line noise was gone.

## Stage 5: the command-parsing cascade and the movement engine

Picks up on the two things stage 4's own notes flagged as untouched.

**First pass**: the command-parsing cascade between the keyword-matching
`DATA` table and the verb dispatch's `SELECT CASE` turned out to contain
eight genuinely self-contained "scan until match" loops -- keyword matching,
the exotic-word/direction-word/direction-of-movement checks, the
narrow-tunnel item check, and two "was it a keyword/an item" checks -- none
of them touching the 40 verb handlers themselves. All eight are now `WHILE`/
`FOR` loops. Two GOTO-reached mini-routines shared by multiple loops turned
out not to be loops at all, so they became procedures instead:
`askWhatToDoWithItem()` and `printDontUnderstand()`.

**Second pass**: the movement-execution engine is now four functions instead
of a GOTO web -- confirmed to be a de facto shared subroutine, not a loop at
all, reached from inside the movement logic itself *and* directly from
`PLUGH`/`XYZZY`/`PLOVER`/`CLIMB`/`CROSS`/`ENTER`/`LEAVE` as a "teleport"
mechanism:

- `attemptMove%(d%)` -- looks up the destination room for direction `d%`.
- `checkSpecialRoomAndMove%(d%, z2%)` -- the special-room/special-direction
  checks (grate, nugget, bridge, snake, narrow tunnel, troll, dragon), a
  straightforward `elseif` chain over mutually exclusive room numbers.
- `performMove%(z2%)` -- the actual room transition.

A function can't `GOTO` a top-level label the way the original targets
could, so every call site now checks the returned 0/1 and branches itself
instead. Also verified with a real BASCOM compile under dosbox-x.

One thing remained: the outer game loop every one of the 40 verb handlers
`GOTO`s back into once it's done -- flagged as stage 6's job.

## Stage 6: structuring the outer game loop

The outer redisplay loop and the inner "get one command" loop are now a
`WHILE TRUE` nested inside another. This became possible only once BASCAL
grew a `continue` statement (added alongside this stage's own work,
unqualified like `exit` -- the transpiler resolves which enclosing loop it
leaves): `continue` for a verb handler that wants another command without
redisplaying, `exit` for one that wants the room redisplayed. The original
loop-entry labels stay real labels rather than disappearing entirely -- a
handful of stage 5's own scan loops need to jump out *two* loop levels at
once to reach them, which `continue`/`exit` can't do (each only ever escapes
its own innermost loop), so those specific sites keep an explicit `GOTO`
instead.

The reincarnation/pit-death cascade is now two procedures, `reincarnate()`
and `checkPitsAndReincarnateIfNeeded()`, plus a third, `endGame()`, pulled
out separately once it turned out two verb handlers outside the cascade
entirely -- QUIT's "don't save" path and SAVE GAME's own "also quit" check
-- shared that same ending sequence.

Verified with a real `fbc` build and smoke tests covering movement,
inventory, GET/DROP, SAVE/LOAD, QUIT with and without saving, SCORE,
SHORT/LONG/BRIEF, an exotic word, an item named with no verb, JUMP, ATTACK,
and CLIMB -- byte-identical output against stage 5 throughout. The
pit-fall/reincarnation cascade itself, not reachable from a fresh game
within a short smoke test (every room reachable in a few moves is lit), was
exercised separately in a scratch copy with the darkness/pit checks
temporarily patched to trigger on an early, reachable room.

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

<!-- BEGIN generated stage source -->

<div class="prose" markdown="1">

## Full source, stage by stage

Each stage below is the complete, unabridged file -- collapsed by default; click to expand.

</div>

<details class="source-embed" markdown="1">

<summary><code>stage1-original-basic/adventure.bas</code> -- Stage 1 — the original, untouched</summary>

```basic

10 rem ADVENTURE/3000 VERSION 3.2     27 FEB 1979 AT 5:30 PM
11 rem THIS PROGRAM IS RELATIVELY BUG-FREE, BUT ONE STILL MUST
12 rem realize that murphy 'S LAW STILL PREVAILS!
13 rem
14 rem ADVENTURE: PROGRAMMED IN HP/3000 BASIC BY BENJAMIN MOSER
15 rem JAMES MADISON HIGH SCHOOL, VIENNA, VIRGINIA.  THE BASIC LAYOUT OF THE
16 rem GAME WAS CONCEIVED BY DON WOODS & WILLIE CROWTHER, BOTH OF M.I.T.
17 rem
18 rem ADVENTURE WAS PORTED TO THE MACINTOSH PLUS BY THE ELIZABETH AND DAVID HUNTER
19 rem IN MARCH 1998 AND THEN TO PYBASIC FOR THE RASPBERRY RP2040 IN JUNE 2021
20 rem PRINT "Adventure 3.2 on ";date$;" at ";time$
21 PRINT "Adventure 3.2 for PyBASIC"
22 open "AMESSAGE" for INPUT as #3:open "AMOVING" for INPUT as #4
23 open "ADESCRIP" for INPUT as #1:open "AITEMS" for INPUT as #2
44 rem dirs is an array of possible room directions, it replaces file AMOVING
45 dim dirs(100,10)
46 dim indx(303)
47 dim fraindx(10)
50 dim s(99)
51 dim v(100)
52 dim k(200)
53 dim o(15)
70 PRINT:PRINT "Initializing.";
110 rem initialize
130 rem total rooms, items,and keywords
150 l1 = int(RND(1)*4)+1 : l2 = l1
160 g = 0 : b0 = 1 : sn = 1 : d1 = 1 : d2 = 0 : t = 1 : b1 = 0 : b2 = 0 : p1 = 0: dead = 0
161 l = 0 : c = 0 : d3 = 0 : b3 = 0 : d0 = 2 : t1 = 100 : t2 = 35 : t3 = 149 : r0 = 0 : c0 = 0: c$=""
162 KC = 1.02
170 for ii = 1 to 99
171  s(ii) = 0 : v(ii) = 0
172 next ii
175 PRINT ".";:v(100) = 0
182 indx(1) = -1:indx(2)=-1
190 rem   read in possible movement direction array
192 for z2 = 1 to 100
193  INPUT #4,dx1,dx2,dx3,dx4,dx5,dx6,dx7,dx8,dx9,dx0
194  dirs(z2,1)=dx1:dirs(z2,2)=dx2:dirs(z2,3)=dx3:dirs(z2,4)=dx4:dirs(z2,5)=dx5
195  dirs(z2,6)=dx6:dirs(z2,7)=dx7:dirs(z2,8)=dx8:dirs(z2,9)=dx9:dirs(z2,10)=dx0
196  PRINT ".";
197 next z2
198 close #4
200 restore 230
210 rem    read in locations of items
220 for z2 = 1 to T2 step 5
221 read dx1,dx2,dx3,dx4,dx5:s(z2)=dx1:s(z2+1)=dx2:s(z2+2)=dx3:s(z2+3)=dx4:s(z2+4)=dx5
222 PRINT ".";
228 next z2
230 data 18,25,23,24,21
231 data 52,0,71,74,58
232 data 59,69,66,82,100
233 data 7,49,7,7,7
234 data 7,12,13,40,38
235 data 69,0,46,0,0
236 data 15,60,82,22,250
240 for ii = 1 to 15 step 5 
241 read dx1,dx2,dx3,dx4,dx5:o(ii)=dx1:o(ii+1)=dx2:o(ii+2)=dx3:o(ii+3)=dx4:o(ii+4)=dx5
242 PRINT ".";
243 next ii
245 data 1,2,2,2,2
246 data 3,4,3,3,2
247 data 5,3,2,3,3
260 rem ASK IF HE WANTS DIRECTIONS
263 z59 = 301:gosub 7620
264 gosub 9860
266 if z0 then 267 else 320
267 z59 = 302:gosub 7620
280 rem    command INPUT routine
300 rem PRINT room, items
310 rem If it's dark don't let him see anything
320 if l1 < 13 or l1 = 58 then 350
321 if l = 1 and (s(18) = l1 or s(18) = -1) then 350
330 z59 = 45:gosub 7620
340 goto 400
350 on d0+1 gosub 6780,7990,8180
360 v(l1) = 1
370 gosub 6680
375 if dead = 1 then 9540
380 gosub 7800
390 rem INPUT LOOP --- MULTIPLE COMMAANDS, REMOVE JUNK CHARACTERS
400 if len(c$) > 0 then 500
410 INPUT ">";c$:c$=upper$(c$)
420 if c$ = "" then 410
425 PRINT:PRINT
430 c$ = upper$(c$)
449 rem    65-90 = A-Z               48-57 = 0-9         46 = . 44 = ,   
450 for x = 1 to len(c$)
460 z5 = asc(mid$(c$,x,1))
470 if (z5 > 64 and z5 < 91) or (z5 > 47 and z5 < 58) or z5 = 44 then 480 else 471
471 c$ = mid$(c$,1,x-1)+" "+mid$(c$,x+1)
480 next x
490 if mid$(c$,len(c$),1) = "," then 500
491 c$ = c$+","
500 z4 = instr(c$,",")
510 a$ = upper$(mid$(c$,1,z4-1)) : c$ = mid$(c$,z4+1)
550 a$ = " "+a$+" "
560 rem search a$ for keywords,puut kwd code into k(x)
570 rem items
580 data 1,"GOLD",1,"NUGGET",2,"BARS",2,"SILVER",3,"JEWELRY",4,"COINS"
590 data 5,"DIAMONDS",6,"MING",6,"VASE",7,"PEARL",8,"EGGS",8,"NEST"
600 data 9,"TRIDENT",10,"EMERALD",11,"PLATINUM",11,"PYRAMID",12,"CHAIN"
610 data 13,"SPICES",14,"PERSIAN",14,"RUG",15,"TREASURE",15,"CHEST"
620 data 16,"WATER",17,"OIL",18,"LAMP",18,"LANTERN",19,"KEYS",20,"FOOD",21,"BOTTLE"
630 data 22,"CAGE",23,"ROD",23,"WAND",24,"CLAM",25,"MAGAZINE",26,"BEAR"
640 data 27,"AXE",28,"VELVET",28,"PILLOW",29,"SHARDS",30,"OYSTER"
650 data 31,"BIRD",32,"TROLL",33,"DRAGON",34,"SNAKE",35,"DWARF"
660 data 36,"ROCK",36,"BOULDER",37,"STAIRS",38,"STEPS",39,"HOUSE",39,"BUILDING"
665 data 40,"GRATE",41,"STREAM",42,"ROOM",43,"BRIDGE",44,"PIT",45,"VOLCANO"
667 data 46,"ROAD",47,"ALL",47,"EVERYTHING"
670 rem DIRECTIONS
680 data 100,"N",100,"NORTH",101,"NE",101,"NORTHEAST",102,"E",102,"EAST"
682 data 103,"SE",103,"SOUTHEAST",104,"S",104,"SOUTH",105,"SW",105,"SOUTHWEST"
684 data 106,"W",106,"WEST",107,"NW",107,"NORTHWEST",108,"U",108,"UP",109,"D",109,"DOWN"
690 rem VERBS
700 data 110,"PLUGH",111,"XYZZY",112,"PLOVER",113,"CROSS",114,"CLIMB",115,"JUMP"
710 data 116,"FILL",117,"EMPTY",117,"POUR",118,"LOOK",118,"L",119,"LIGHT",119,"ON",120,"EXTINGUISH"
720 data 120,"OFF",121,"IN",121,"ENTER",122,"LEAVE",122,"OUT",123,"INVENTORY",123,"I"
730 data 124,"GET",124,"CATCH",124,"TAKE",125,"DROP",125,"DUMP",126,"THROW",127,"ATTACK"
740 data 127,"KILL",128,"FEED",129,"WATER",130,"LOCK",131,"UNLOCK"
750 data 132,"FREE",132,"RELEASE",133,"WAVE",134,"OPEN",135,"CLOSE"
760 data 136,"OIL",137,"EAT",138,"DRINK",139,"FEE FIE FOE FOO"
765 data 140,"SHORT",141,"LONG",142,"BRIEF",143,"QUIT",143,"STOP",143,"END"
770 data 144,"SCORE",145,"SAVE",146,"LOAD",147,"READ",147,"EXAMINE"
775 data 148,"YES",148,"Y",149,"BUG",150,"*"
780 restore 580
790 for i = 1 to 200
791 k(i) = 0
792 next i
800 z1 = 0 : z3 = 0
810 rem T3=TOTAL NUMBER OF KEYWORDS
820 if z1 > t3 then 900
830 read z1,b$
840 b$ = " "+b$+" "
850 rem IF KEYWORD WAS FOUND, NOTE THIS IN K(Z1)
860 if instr(a$,b$) = 0 then goto 820
861 k(z1) = 1
870 rem KEYWORDK #Z1 NOT FOUND
880 goto 820
890 rem EXOTIC WORDS
900 z0 = 36
910 for x = z0 to 46
920 if k(x) = 0 then 960
930 gosub 8390
940 print "What do you want to do with the ";d$;"?"
950 goto 410
960 next x
970 for x = 110 to t3
980 if k(x) = 1 then 1950
990 next x
1000 rem THEN IT'S A DIRECTION
1010 for d = 1 to 10
1020 if k(d+99) = 1 then 1070
1030 next d
1040 rem COMMAND NOT A DIRECTION
1050 goto 1950
1060 rem CAN HE MOVE THAT WAY?
1070 z2 = dirs(L1,d)
1130 if z2 = 255 then 1470
1140 if z2 < 1 or z2 > 254 then 1220
1150 rem NORMAL MOVING
1160 rem CHECK FOR SPECIAL MOVE CONDITIONS
1170 goto 1260
1180 l2 = l1 : l1 = z2
1190 if s(35) = l2 then 1191 else 1200
1191 s(35) = l1
1200 goto 9780
1220 z59 = 1
1221 gosub 7620
1230 goto 400
1240 rem SPECIAL ROOM DIRECTIONS
1250 rem GRATE
1260 if L1 = 10 and (d = 10 or d = 5) THEN 1280
1261 IF L1 = 11 and (d = 9 or d = 3) then 1280 else 1320
1270 rem IF GRATE IS OPEN (G=0) MOVE HIM
1280 if g = 1 then 1180
1290 z59 = 10:gosub 7620
1300 goto 400
1310 rem CAN'T TAKE NUGGET UPPSTAIRS
1320 if not (l1 = 17 and d = 9 and s(1) = -1) then 1360
1330 z59 = 38:gosub 7620
1340 goto 400
1350 rem CRYSTAL BRIDGE AND FISSURE
1360 if not (l1 = 19 and d = 7 or l1 = 20 and d = 3) then 1410
1370 if b2 then 1180
1380 z59 = 3:gosub 7620
1390 goto 400
1400 rem MT. KING & SNAKE
1410 if not (l1 = 22 and d <> 3 and d <> 9) then 1690
1420 if sn = 0 then 1180
1430 z59 = 50:gosub 7620
1440 goto 400
1460 rem bedquilt and random directiosn
1470 if l1 <> 44 then 1590
1480 if RND(1) > 0.5 then 1510
1490 z59 = 52:gosub 7620
1500 goto 400
1510 restore 1530
1520 rem ROOMS TO JU
1530 data 33,37,45,92,76
1540 for z3 = 1 to int(RND(1)*5)+1
1550  read z2
1560 next z3
1570 goto 1180
1580 rem WITT's END
1590 if l1 <> 39 then 1220
1600 rem SHOULD WE LET HIM OUT?
1610 if RND(1) < 0.15 then 1650
1620 rem NO
1630 z59 = 52:gosub 7620
1640 goto 400
1650 rem YES
1660 z2 = 38
1670 goto 1180
1680 rem Narrow Tunnel
1690 if not (l1 = 57 or l1 = 58) then 1780
1691 IF K(102)=0 AND K(106)=0 THEN 1780
1700 for z3 = 1 to t2
1710  if z3 = 10 then 1750
1720  if s(z3) <> -1 then 1750
1730  z59 = 53:gosub 7620
1740  goto 400
1750 next z3
1760 goto 1180
1770 rem TROLL
1780 IF L1=60 AND D=2 THEN 1790
1781 IF L1=61 AND D=6 THEN 1790 ELSE 1860
1790 on t+1 goto 1180,1800,1820,1840
1800 z59 = 55:gosub 7620
1810 goto 400
1820 z59 = 56:gosub 7620
1821 z59 = 55:gosub 7620
1830 T=1:goto 400
1840 t = 2
1850 goto 1180
1860 if not (l1 = 73 and d = 1 and d2 = 0) then 1890
1870 z59 = 57:gosub 7620
1880 goto 400
1890 if not (l1 = 82 and s(33) = l1 and d = 1) then 1180
1900 rem DRAGON
1910 z59 = 51:gosub 7620
1920 goto 400
1940 rem OTHER COMMANDS
1950 for z1 = 100 to t3
1960 if k(z1)=1 then 2090
1970 next z1
1980 rem ITEM BO NO VERB?
1990 restore 9961
2000 for x = 1 to 35
2010  read d$
2020  if k(x) = 1 then 940
2030 next x
2040 restore 2070
2050 for x = 1 to int(RND(1)*4)+1
2051  read b$
2052 next x
2060 PRINT b$
2070 data "What?","I don't understand.","I can't understand that.","I don't know that word."
2080 goto 400
2090 z1 = z1-109
2095 on z1 goto 2120,2220,2300,2390,2570,2640,2680,2860,2930,3000,3080,3130,3290,3450,3590,3920,4160,4430,4630,4800,4950,5060,5190,5410,5560,5750,5810,5920,6000,6090,6230,6270,6310,6350,6410,8970,9080,9220,4490,9330
2110 goto 2040
2120 REM *** PLUGH ***
2130 IF L1<>7 THEN 2170
2140 IF S(35)=L1 THEN S(35)=0
2150 Z2=26
2160 GOTO 1180
2170 IF L1<>26 THEN 2200
2180 Z2=7
2190 GOTO 1180
2200 z59 = 2:gosub 7620
2210 GOTO 400
2220 REM *** XYZZY ***
2230 IF L1<>7 THEN 2270
2240 IF S(35)=L1 THEN S(35)=0
2250 Z2=13
2260 GOTO 1180
2270 IF L1<>13 THEN 2200
2280 Z2=7
2290 GOTO 1180
2300 REM *** PLOVER *** (CAN'T BRING EMERALD WITH HIM)
2310 IF L1>26 THEN 2360
2320 IF S(35)<>L1 THEN 2330
2325 S(35) = 0
2330 IF S(10)<>-1 THEN 2340
2335 S(10) = L1
2340 Z2 = 58
2350 GOTO 1180
2360 IF L1<>58 THEN 2200
2370 Z2=26
2380 GOTO 1180
2390 REM *** CROSS ***
2400 IF L1<>19 THEN 2470
2410 IF B2<>0 THEN 2440
2420 z59 = 3:gosub 7620
2430 GOTO 400
2440 D=7
2450 REM JUST GIVE NEW DIRECTION, USE MOVE ROUTINE
2460 GOTO 1070
2470 IF L1<>20 THEN 2510
2480 IF B2=0 THEN 2420
2490 D=3
2500 GOTO 1070
2510 IF L1<>60 THEN 2540
2520 D=2
2530 GOTO 1070
2540 IF L1<>61 THEN 2200
2550 D=6
2560 GOTO 1070
2570 REM *** CLIMB ***
2580 IF L1<>50 THEN 2200
2590 REM CAN HE CLIMB BEANSTALK?
2600 IF P1<2 THEN 2200
2610 REM YES
2620 Z2=70
2630 GOTO 1180
2640 REM *** JUMP *** STRICTLY SUICIDAL
2650 IF L1<>16 AND L1<>19 AND L1<>20 AND L1<>27 THEN 2200
2660 z59 = 4:gosub 7620
2670 GOTO 9550
2680 REM FILL
2690 IF S(21)=-1 THEN 2730
2700 B$="bottle"
2710 PRINT "You don't have the ";b$
2720 goto 410
2730 IF B0=0 THEN 2760
2740 z59 = 5:gosub 7620
2750 GOTO 410
2760 IF L1<>7 AND L1<>8 AND L1<>9 AND L1<>35 AND L1<>74 AND L1<>81 THEN 2790
2770 B0=1:S(16)=-1
2780 GOTO 2840
2790 IF L1=49 THEN 2830
2800 B$="oil"
2810 PRINT "I see no ";B$;" here."
2820 GOTO 400
2830 B0=2:S(17)=-1
2840 PRINT "The bottle is now filled."
2850 GOTO 400
2860 REM *** EMPTY ***
2870 IF S(21)=-1 THEN 2890
2880 GOTO 2700
2890 REM EMPTY BOTTLE (ASSUMED FULL)
2900 S(B0+15)=0:B0=0
2910 PRINT "Emptied"
2920 GOTO 400
2930 rem *** LOOK ***
2940 if l1 < 13 or l1 = 58 then 2970
2941 if l = 1 and (s(18) = l1 or s(18) = -1) then 2970
2950 z59 = 45:gosub 7620
2960 goto 400
2970 gosub 8050
2980 gosub 6680
2990 goto 400
3000 rem *** LIGHT ***
3010 if s(18) = -1 then 3040
3020 b$ = "lamp"
3030 goto 2710
3040 l = 1
3050 b$ = "on"
3060 PRINT "The lamp is now ";b$
3070 goto 2940
3080 REM *** OFF (EXTINGUSIH) ***
3090 IF S(18)=-1 THEN 3110
3100 GOTO 3020
3110 L=0:B$="off"
3120 GOTO 3060
3130 REM *** ENTER ***
3140 IF L1<>6 THEN 3180
3150 REM TO HOUSE
3160 D=3
3170 GOTO 1070
3180 IF L1<>68 THEN 3240
3190 REM TO BARREN ROOM
3200 D=3
3210 GOTO 1070
3240 FOR D=10 TO 1 step -1
3250  Z2 = DIRS(L1,D)
3260  IF Z2>0 AND Z2<101 THEN 1150
3270 NEXT D
3280 GOTO 2200
3290 REM ** LEAVE ***
3300 IF L1<>7 THEN 3340
3310 REM LEAVE HOUSE
3320 D=7
3330 GOTO 1070
3340 IF L1<>69 THEN 3400
3350 REM LEAVE BARREN ROOM
3360 D = 7
3370 GOTO 1070
3400 FOR D = 1 TO 10
3410  Z2 = DIRS(L1,D)
3420  IF Z2>0 AND Z2<101 THEN 1150
3430 NEXT D
3440 GOTO 2200
3450 REM *** INVENTORY ***
3470 Z0=0
3480 PRINT "You are carrying:";
3490 FOR X=1 TO T2
3510  IF S(X)<>-1 THEN 3540
3511  restore 9960+x:read b$
3520  PRINT B$
3530  Z0 = Z0 + 1
3540 NEXT X
3550 IF Z0=0 THEN 3551 else 3560
3551 PRINT "nothing."
3560 PRINT
3570 GOTO 400
3590 rem *** GET ***
3630 if k(47) = 1 then 3680
3640 gosub 8280
3650 if z8 > 0 then 3680
3660 PRINT "Get what?"
3670 goto 2040
3680 for z3 = 1 to t2
3710 if k(47) = 1 then 3730
3720 if k(z3) = 0 then 3900
3730 if s(z3) <> l1 then 3750
3740 if s(z3) = l1 then 3790
3750 if k(47) = 1 then 3900
3760 restore 9960+z3:read a$:PRINT a$;" not here."
3770 goto 3900
3780 rem MUST CHECK NOW FOR LEGALITY OF TAKING ITEM
3790 z8 = 0
3800 for x = 1 to t2
3810 if s(x) <> -1 then 3820
3811 z8 = z8+1
3820 next x
3830 if z8 < 7 then 3870
3840 rem CARRYING TOO MUCH
3850 z59 = 54:gosub 7620
3860 goto 410
3870 goto 6880
3880 s(z3) = -1
3890 restore 9960+z3:read a$:PRINT a$;":taken."
3900 next z3
3910 goto 400
3920 REM *** DROP ***
3940 if k(47) = 1 then 4000
3950 gosub 8280
3960 IF Z8>0 THEN 4000
3970 PRINT "Drop what?"
3980 GOTO 2040
4000 FOR Z3=1 TO T2
4030  IF K(47)=1 THEN 4060
4040  IF K(Z3)<>1 THEN 4140
4050  IF S(Z3)=0 THEN 4140
4060  IF S(Z3)=-1 THEN 4100
4070  IF K(47)=1 THEN 4140
4080  restore 9960+z3:read b$:PRINT "You don't have the ";B$
4090  GOTO 4140
4100  REM STILL NEED TO ELABORATE ON DROP (BIRD IN CAGE, BOTTLE)
4110  GOTO 7380
4120  restore 9960+z3:read b$:PRINT B$;":dropped."
4130  S(Z3)=L1
4140 NEXT Z3
4150 GOTO 400
4160 REM *** THROW ***
4170 GOSUB 8280
4180 IF Z8>0 THEN 4210
4190 PRINT "Throw what?"
4200 GOTO 2040
4210 IF S(Z3)<>-1 THEN 2710
4220 IF NOT (Z3<16 AND S(32)=L1) THEN 4260
4230 REM THROW TREASURE TO TROLL
4240 z59 = 27:gosub 7620
4241 S(Z3)=0:T=3:GOTO 400
4260 IF NOT (Z3=27 AND S(32)=L1) THEN 4300
4270 REM TRYING TO BUTCHER TROLL?
4280 z59 = 26:gosub 7620
4281 S(27)=L1:GOTO 400
4300 IF NOT (Z3=27 AND S(35)=L1) THEN 4380
4310 REM TRYING TO KILL DWARF
4320 IF RND(1)>0.5 THEN 4360
4330 z59 = 29:gosub 7620
4340 GOSUB 8650
4350 GOTO 4410
4360 z59 = 30:gosub 7620
4361 S(35)=0:GOTO 4410
4380 REM NOTHING SPECIAL, JUST DROP ITEM
4390 IF S(35)<>L1 THEN 4400
4391 GOSUB 8550
4400 PRINT "Thrown."
4410 S(Z3) = L1
4415 if dead = 1 then 9540
4420 GOTO 400
4430 REM *** ATTACK ***
4440 GOSUB 8280
4450 IF NOT (Z3=33 AND S(Z3)=L1 AND L1=82) THEN 4520
4460 REM HE CAN KILL DRAGON
4470 z59 = 68:gosub 7620
4480 GOTO 410
4490 IF L1<>82 THEN 2040
4500 z59 = 69:gosub 7620
4501 S(33)=0:D1=0:GOTO 400
4520 IF S(32)<>L1 THEN 4560
4530 REM TRYING TO MUNGE TROLL
4540 Z9=FNA(25):GOTO 400
4560 IF NOT (Z3=26 OR Z3>30) THEN 4600
4570 REM DANGEROUS TO ATTACK THESE
4580 z59 = 70:gosub 7620
4590 GOTO 400
4600 REM NOTHING TO ATTACK
4610 z59 = 71:gosub 7620
4620 GOTO 400
4630 REM *** FEED ***
4640 GOSUB 8280
4650 IF Z3<>35 THEN 4690
4660 REM CAN'T FEED DWARF!
4670 z59 = 24:gosub 7620
4680 GOTO 400
4690 IF S(20) = -1 THEN 4720
4700 B$ = "FOOD":GOTO 2710
4720 IF L1=69 THEN 4760
4730 PRINT "I can't feed it."
4740 z59 = 23:gosub 7620
4750 GOTO 400
4760 IF S(20)=L1 THEN 7600
4770 B1=1:S(20)=0:z59 = 6:gosub 7620
4790 GOTO 400
4800 REM *** WATER ***
4810 IF S(16) = -1 THEN 4840
4820 B$ = "water":GOTO 2710
4840 IF L1<>50 THEN 2200
4850 REM GOTO P1+1 OF 4860,4890,4920
4851 IF P1 = 0 THEN 4860
4852 IF P1 = 1 THEN 4890
4853 IF P1 = 2 THEN 4920
4860 z59 = 7:gosub 7620
4870 P1=1:S(16)=0:B0=0:GOTO 400
4890 z59 = 8:gosub 7620
4900 P1=2:S(16)=0:B0=0:GOTO 400
4920 z59 = 9:gosub 7620
4930 P1=0:S(16)=0:B0=0:GOTO 400
4950 REM *** LOCK ***
4960 IF L1=10 OR L1=11 THEN 4990
4970 REM NOTHING LOCKABLE
4980 GOTO 2200
4990 IF S(19)=-1 THEN 5020
5000 B$="keys":goto 2710
5020 G=0:z59 = 10:gosub 7620
5040 GOTO 400
5060 REM *** UNLOCK ***
5070 IF S(19)<>-1 THEN 5000
5080 IF L1<>10 AND L1<>11 THEN 5120
5090 G=1:z59 = 11:gosub 7620
5110 GOTO 400
5120 IF L1<>69 THEN 2200
5130 IF B1>0 THEN 5160
5140 z59 = 12:gosub 7620
5150 goto 400
5160 IF C<>0 THEN 5170
5165 C=1:B1=2
5170 z59 = 13:gosub 7620
5180 GOTO 400
5190 REM *** FREE ***
5200 IF K(31) = 1 THEN 5240
5210 REM CAN'T FREE ANYTHING BUT BIRD
5220 z59 = 2:gosub 7620
5230 GOTO 410
5240 IF S(31)<>-1 THEN 5220
5250 S(31) = L1:B3=0
5260 PRINT "Freed."
5270 IF L1<>22 THEN 5350
5280 IF SN<>1 THEN 400
5290 B$="snake"
5300 PRINT "The little bird attacks the green ";B$;" and"
5310 IF L1=82 THEN 5380
5320 PRINT "drives it off"
5330 SN=0:S(34)=0:GOTO 400
5350 IF L1<>82 THEN 400
5360 B$="dragon":GOTO 5300
5380 PRINT "gets burned to a crisp"
5390 S(31)=0
5400 GOTO 400
5410 REM *** WAVE ***
5420 IF K(23) <> 1 THEN 2200
5430 IF S(23)=-1 THEN 5460
5440 B$="rod":GOTO 2710
5460 REM  IS HERE NEAR FISSURE
5470 IF L1<>19 AND L1<>20 THEN 2200
5480 REM yes
5490 REM GOTO B2+1 OF 5500,5530
5491 IF B2=0 THEN 5500
5492 IF B2=1 THEN 5530
5500 z59 = 14:gosub 7620
5510 B2=1:GOTO 400
5530 z59 = 15:gosub 7620
5540 B2=0:GOTO 400
5560 REM *** OPEN ***
5570 GOSUB 8280
5580 IF Z3>0 THEN 5610
5590 PRINT "Open ";:goto 2040
5610 IF Z3=40 THEN 5070
5620 IF S(Z3)=L1 THEN 5650
5630 PRINT "I see no ";b$;" here.":goto 400
5650 if z3=24 THEN 5680
5660 PRINT "I don't know how to open a ";B$:GOTO 400
5680 IF S(9)=-1 THEN 5710
5690 z59 = 16:gosub 7620
5700 GOTO 400
5710 IF S(Z3) = 0 THEN 2200
5720 REM HE'S OPENED CLAM, SO PRINT DESCRIPTION OF THIS
5730 REM PUT PEARL IN CUL-DE-SAC
5740 S(7)=43:S(24)=0:S(30)=L1:z59 = 17:gosub 7620
5750 GOTO 400
5760 REM *** CLOSE ***
5770 GOSUB 8280
5780 IF Z3=40 THEN 4960
5790 z59 = 18:gosub 7620
5800 GOTO 400
5810 REM OIL
5820 IF K(17)=0 THEN 2200
5830 IF S(17)=-1 THEN 5860
5840 B$="oil":GOTO 5630
5860 IF L1<>73 THEN 2200
5870 REM IS DOOR STILL RUSTED
5880 IF D2=1 THEN 2200
5890 D2=1:S(17)=0:B0=0:z59 = 19:gosub 7620
5900 GOTO 400
5910 REM *** EAT ***
5920 IF K(20) = 1 THEN 5950
5930 z59 = 20:gosub 7620
5940 GOTO 410
5950 Z3=20:GOSUB 8490
5970 IF Z5=0 THEN 410
5980 z59 = 73:gosub 7620
5381 S(20)=0:B0=0:GOTO 400
6000 REM *** DRINK ***
6010 IF K(16) =1 THEN 6040
6020 z59 = 21:gosub 7620
6030 GOTO 410
6040 Z3=16:GOSUB 8490
6060 IF Z5=0 THEN 410
6070 z59 = 22:gosub 7620
6071 S(17)=0:B0=0:GOTO 400
6090 REM *** FEE FIE FOE FOO ***
6100 IF L1=71 THEN 6130
6110 z59 = 2:gosub 7620
6120 GOTO 410
6130 IF S(8)<>L1 THEN 6180
6140 REM MAKE NEST VANISH
6150 z59 = 79:gosub 7620
6160 S(8)=0:GOTO 400
6180 REM IF S(8)=0 THEN 6110
6190 S(8)=L1
6200 rem MAKE NEST RE-APPEAR
6210 z59 = 81:gosub 7620
6220 GOTO 400
6230 REM *** SHORT ***
6240 PRINT "Short descriptions"
6250 D0=0:GOTO 400
6270 REM *** LONG ***
6280 PRINT "Long descriptions"
6290 D0=1:GOTO 400
6310 REM *** BRIEF ***
6320 PRINT "OK, I'll only describe the room in detail the first time."
6330 D0=2:GOTO 400
6350 REM *** QUIT ***
6360 PRINT "Save game";
6370 GOSUB 9860
6380 IF Z0=1 THEN 8970
6390 GOTO 9750
6400 REM SCORE ***
6410 GOSUB 6430
6420 GOTO 400
6430 REM PRINT OUT SCORE DATA
6440 GOSUB 6510
6450 PRINT "Your score is now ";S0
6451 PRINT "You have explored ";(Z9/T1)*T1;"% of the cave."
6460 RESTORE 6470
6470 DATA "beginner","novice","experienced","advanced","expert"
6480 Z9 = INT((S0-1)/100)
6481 IF Z9 <= 4 THEN 6483
6482 Z9=4
6483 FOR Z0=0 TO Z9
6484  READ D$
6485 NEXT Z0
6490 PRINT "That makes you a ";D$;" adventurer."
6500 RETURN
6510 REM COMPUTE CURRENT SCORE
6520 RESTORE 230
6530 Z9=0:S0=0
6540 FOR Z0=1 TO 15
6550  READ Z1
6560  IF Z1=0 THEN 6590
6570  IF V(Z1)<>1 THEN 6580
6575  S0=S0+4*O(Z0)
6580  IF S(Z0)<>7 THEN 6590
6585  S0=S0+4*O(Z0)
6590 NEXT Z0
6600 S0=(G=1)*10 + S0:S0=(SN=0)*20 + S0:S0=(D1=0)*30 + S0:S0=(T=0)*30 + S0:S0=(B1=2)*20 + S0
6605 S0=(B2=1)*20 + S0:S0=(P1=2)*20 + S0:S0=(D2=1)*20 + S0:S0=(C=1)*20 + S0
6610 FOR Z0 = 1 TO T1
6620  IF V(Z0)<>1 THEN 6630
6621  S0=S0+1:Z9=Z9+1
6630 NEXT Z0
6640 RETURN
6660 rem list items at location l1
6680 fseek #2,0
6690 for z1 = 1 to t2
6700  INPUT #2,a$
6710  if s(z1) <> l1 then 6720
6711  PRINT a$
6720 next z1
6721 IF S(26)<>-1 THEN 6730
6722 z59 = 67:gosub 7620
6730 rem CHECK FOR DWARF,PIRATE
6740 gosub 8560
6745 if dead = 1 then 6770
6750 gosub 8800
6760 PRINT
6770 return
6775 rem Print Short room description
6780 fseek #1,0
6820 for z1 = 1 to l1
6830  INPUT #1,a$
6840 next z1
6850 v(l1) = 1
6860 PRINT a$
6870 return
6880 rem SPECIAL GETS
6890 if not (z3 = 24 or z3 = 30 or z3 > 31) then 6930
6900 rem CAN'T GET THESE FOR SOME REASON
6910 z59 = 61:gosub 7620
6920 goto 400
6930 if not (z3 = 12 and c = 0) then 6970
6940 rem CHAIN
6950 z59 = 58:gosub 7620
6960 goto 3900
6970 rem BEAR IS HE FED? UNLOCKED?
6980 if not (z3 = 26 and b1 <> 2) then 7010
6990 z59 = 61:gosub 7620
7000 goto 3900
7010 if not (z3 = 14 and d1 = 1) then 7050
7020 rem DRAGON AND RUG
7030 z59 = 59:gosub 7620
7040 goto 3900
7050 if not (z3 = 16 or z3 = 17) then 7090
7060 rem OIL AND WATER DO SAME AS FILL
7070 PRINT "Why not say 'fill'?"
7080 goto 3900
7090 if not (z3 = 22 and b3) then 7140
7100 rem TAKE BIRD SINCE IT'S IN CAGE
7110 s(31) = -1:PRINT "Bird and ";:goto 3880
7140 if z3 <> 31 then 7310
7150 rem GETTING BIRD
7160 if b3 <> 1 then 7210
7170 rem TAKE CAGE, SINCE BIRD IS IN IT
7180 PRINT "Cage and ";:s(22) = -1:goto 3880
7210 if s(22) = -1 then 7240
7220 b$ = "cage":goto 2810
7240 if s(23) = -1 then 7280
7250 rem OK TO TAKE BIRD
7260 s(31) = -1 : b3 = 1:goto 3890
7280 rem ROD SCARES BIRD
7290 z59 = 37:gosub 7620
7300 goto 3900
7310 rem BOTTLE FULL? IF SO, GET CONTENTS
7320 if not (z3 = 21 and b0) then 7360
7330 PRINT "Contents and the ";
7340 s(b0+15) = -1
7360 goto 3880
7370 rem SPECIAL "DROP"
7380 IF Z3<>31 THEN 7440
7390 REM BIRD IN CAGE
7400 S(31)=L1:S(22)=L1:B3=1
7410 IF Z3<>31 THEN 7420
7415 PRINT "Cage and ";
7420 IF Z3=22 THEN 7430
7425 PRINT "Bird and ";
7430 goto 4120
7440 if z3=22 and b3=1 then 7400
7450 IF Z3<>21 THEN 7520
7460 REM BOTTLE
7470 IF B0=0 THEN 4120
7480 REM BOTTLE IS FULL, DO DROP CONTENTS TOO
7490 PRINT "Contents and ";
7500 S(15+B0)=L1:GOTO 4120
7520 IF NOT (Z3=16 OR Z3=17) THEN 7541
7530 PRINT "Try saying 'empty'":goto 4140
7541 IF Z3<>26 OR T<>1 OR (L1<>60 AND L1<>61) THEN 7550
7542 z59 = 28:gosub 7620
7543 T=0:S(26)=L1:S(32)=0:GOTO 400
7550 IF Z3<>6 THEN 4120
7560 IF S(28)=L1 THEN 7600
7570 REM GOODBYE, FRAGILE VASE!
7580 z59 = 43:gosub 7620
7581 S(6)=0:S(29)=L1:GOTO 400
7600 z59 = 60:gosub 7620
7610 GOTO 4120
7619 rem PRINT MESSAGE
7620 if indx(1) >= 0 then 7640
7630 gosub 12500
7640 z59 = int(z59):xtmp = indx(z59)
7645 if z59 <> 2 and z59 <> 61 then 7660
7650 xtmp = fraindx(xtmp+min(int(RND(1)*5),5))
7660 fseek #3,xtmp
7670 INPUT #3,b1$
7672 if len(b1$) = 0 then 7673 else 7690
7673 b1$ = " "
7690 if instr(b1$,"#") = 0 then 7670
7700 z4 = val(mid$(b1$,2))
7705 if int(z4) = z59 then 7720
7710 if int(z4) < z59 then 7670
7712 if int(z4) > z59 then 7760
7720 INPUT #3,b1$
7730 if mid$(b1$,1,1) = "#" then 7770
7740 PRINT b1$
7750 goto 7720
7760 PRINT "NO DESC. # ";z59;" IN FILE AMESSAGE"
7770 return
7800 rem
7810 rem SITUATION DESCRIPTIONS
7820 rem
7830 rem GRATE
7840 if l1 = 10 or l1 = 11 then 7842 else 7860
7842 z59 = (g+10):gosub 7620
7850 rem CRYSTAL BRIDGE
7860 if (l1 = 19 or l1 = 20) and b2 = 1 then 7861 else 7880
7861 z59 = 14:gosub 7620
7870 rem PLUGH NOISE
7880 if l1 = 26 and RND(1) > 0.3 then 7881 else 7900
7881 z59 = 41:gosub 7620
7890 rem IRON DOOR
7900 if l1 = 73 and d2 = 0 then 7901 else 7920
7901 z59 = 57:gosub 7620
7910 rem TROLL
7920 if (l1 = 60 or l1 = 61) and t = 1 then 7921 else 7940
7921 z59 = 63:gosub 7620
7930 rem BEAR
7940 if l1 = 69 and b1 = 0 then 7941 else 7950
7941 z59 = 64:gosub 7620
7950 if l1 = 69 and b1 = 1 then 7951 else 7970
7951 z59 = 66:gosub 7620
7960 rem PLANT IN PIT
7970 if l1 = 48 or l1 = 50 then 7971 else 7980
7971 z59 = 47+p1:gosub 7620
7980 return
7990 rem
8000 rem   PRINT long room description from "amessage" file
8010 rem   description is noormally l1+200 except for
8020 rem   maze or forest
8030 rem set v(l1)=1 so as not to repeat long desc(brief mode
8040 v(l1) = 1
8050 if l1 > 4 then 8080
8060 z59 = 200:gosub 7620
8070 goto 8130
8080 if not (l1 > 88 and l1 < 98 or l1 = 99) then 8110
8090 z59 = 288:gosub 7620
8100 goto 8130
8110 rem normal description
8120 z59 = 200+l1:gosub 7620
8130 return
8180 rem always give long description for forest and maze
8190 if l1 < 5 or (l1 > 88 and l1 < 98) or l1 = 99 then 8220
8200 if v(l1)=1 then 8201 else 8220
8201 gosub 6780
8202 goto 8235
8210 rem he hassn't seen this room, so give a long desc.
8220 v(l1) = 1
8230 gosub 7990
8235 return
8240 rem
8250 rem   fetch first item code in k(1 to t2)
8260 rem   z8=total # of items found in list
8270 rem   z3=item code first found
8280 z8 = 0 : z3 = 0: d$ = ""
8300 for z5 = 1 to 45
8320 if k(z5) = 0 then 8360
8330 z8 = z8+1
8340 restore 9960+z5:read b$:d$ = b$
8350 if k(z5)=1 and z8 = 1 then 8352 else 8360
8352 z3 = z5
8360 next z5
8370 b$ = d$
8380 return
8390 rem FIND FIRST ITEM AT ROOM
8400 x1 = 0
8410 FOR Z1=1 TO 47
8415  if x1=1 then 8440
8430  IF K(Z1) = 1 THEN 8431 else 8440
8431  restore 9960+z1:read d$:x1=1
8440 NEXT Z1
8450 RETURN
8460 REM 
8470 REM MAKE SURE HE'S CARRYING ITEM * Z3
8480 REM
8490 IF S(Z3)=-1 THEN 8530
8500 PRINT "You don't have the ";A$
8510 Z5=0
8520 RETURN
8530 Z5=1
8540 RETURN
8550  rem  *** DWARF ***
8560 if d3 <> 0 then 8640
8570 rem SHOULD DWARF GIVE AWAY AXE?
8580 if l1 < 13 then 8790
8590 if RND(1) > 0.05 then 8790
8600 rem GIVE AWAY AXE
8610 z59 = 80:gosub 7620
8620 s(27) = l1 : d3 = 1
8630 goto 8790
8640 rem SHOULD DWARF ATTACK?
8650 if L1 >= 13 then 8660
8652 s(35) = 0:goto 8790
8660 if s(35) <> L1 then 8770
8661 if (l1 <> 60 and l1 <> 61) or t <> 1 then 8670
8662 z59 = 299:gosub 7620
8663 s(35) = 0: goto 8790
8670 if RND(1) > 0.5 then 8790
8680 rem YES!
8690 z59 = 32:gosub 7620
8700 rem DOES THE KNIFE KILL THE PLAYER?
8705 KC = KC - 0.02
8706 IF KC >= 0.75 THEN 8710
8707 KC = 0.75
8710 if RND(1) <= KC then 8750
8720 rem YES
8730 PRINT "It gets you!"
8740 dead = 1 : goto 8790
8750 PRINT "It misses!"
8760 goto 8790
8770 rem SHOULD WE PUT A DWARF HERE?
8780 if RND(1) >= 0.05 then 8790
8781 if (l1 = 60 or l1 = 61) and t = 1 then 8790
8785 s(35) = l1
8786 z59=31:gosub 7620
8790 return
8800 rem *** PIRATE ***
8810 rem FIRST, DOES HE HAVE ANYTHING WORTH STEALING?
8820 z3 = 0
8830 if l1 < 13 then 8960
8840 for x = 1 to 15
8850  if s(x) <> -1 then 8860
8855  z3 = z3+1
8860 next x
8870 if z3 < int(RND(1)*4)+1 then 8960
8880 rem SHOULD WE RIP OFF HIS VALUABLES?
8890 if RND(1) < 0.05 then 8920
8900 z59 = 34:gosub 7620
8910 goto 8960
8920 z59 = 33:gosub 7620
8930 for x = 1 to 15
8940  if s(x) <> -1 then 8950
8945  s(x) = 100
8950 next x
8960 return
8970 REM *** SAVE GAME ***
8980 INPUT "What do you want to call the save file? ";A$
8990 OPEN A$ FOR OUTPUT AS #5 ELSE 9010
9000 GOTO 9030
9010 PRINT "File ";a$;" not created"
9020 GOTO 410
9030 PRINT #5,T1;",";T2;",";T3;",";L1;",";L2;",";G;",";B0;",";SN;",";D1;",";D2;",";D0;",";T;",";B1;",";B2;",";P1;",";L;",";C;",";D3;",";B3;",";R0;",";KC
9040 FOR X=1 TO 99
9041  PRINT #5,S(X);",";V(X)
9042 NEXT X
9043 PRINT #5,V(100)
9044 CLOSE #5
9050 PRINT "Game saved"
9051 C0 = 0
9060 if k(143) = 1 then 9750
9070 GOTO 410
9080 REM *** LOAD OLD GAME ***
9090 IF C0=0 THEN 9120
9100 PRINT "You already have a loaded game!"
9110 GOTO 410
9120 INPUT "Save file name? ";A$
9130 OPEN A$ FOR INPUT AS #5 ELSE 9150
9140 GOTO 9170
9150 PRINT "Unable to use file ";A$
9160 GOTO 410
9170 INPUT #5,T1,T2,T3,L1,L2,G,B0,SN,D1,D2,D0,T,B1,B2,P1,L,C,D3,B3,R0,KCX$
9171 kc = val(kcx$)
9180 FOR X=1 TO 99
9191  INPUT #5,SX,VX:s(x)=sx:v(x)=vx
9192 NEXT X
9193 INPUT #5,vx:v(100)=vx
9194 CLOSE #5
9200 C0=1
9210 GOTO 300
9220 rem *** READ THE MAGAZINE ***
9230 GOSUB 8280
9240 IF Z3=25 THEN 9270
9250 z59 = 74:gosub 7620
9260 GOTO 410
9270 IF S(25)=-1 THEN 9300
9280 B$="magazine"
9290 GOTO 2710
9300 REM OK, LET HIM READ IT
9310 z59 = 303:gosub 7620
9320 GOTO 400
9330 REM *** BUG ***
9340 A$ = "ADVBUGS.TXT"
9341 OPEN A$ FOR APPEND AS #5 ELSE 9150
9390 INPUT "Your name: ";A$
9400 A$=A$+" "+DATE$
9410 PRINT #5,A$
9420 PRINT "Enter your gripe in up to five lines (hit return to quit):"
9430 FOR Z0=1 TO 5
9440  PRINT Z0;
9450  INPUT A$
9460  IF A$="" THEN 9490
9470  PRINT #5,A$
9480 NEXT Z0
9490 PRINT "Message recorded. Thank you!"
9500 CLOSE #5
9510 GOTO 410
9540 REM REINCARNATE HIM
9550 R0=R0+1
9560 IF R0=1 THEN 9580
9561 IF R0=2 THEN 9610
9562 IF R0=3 THEN 9740
9570 REM ASK HIM IF HE WANTS TO BE REINCARNATED
9580 z59 = 75:gosub 7620
9590 GOSUB 9860
9600 GOTO 9630
9610 z59 = 77:gosub 7620
9620 GOTO 9580
9630 IF Z0=0 THEN 9750
9640 z59 = 76:gosub 7620
9650 REM PUT HIM BACK IN HOUSE, REARRANGE HIS STUFF
9660 S(18)=7:L=0:DEAD=0:KC=1.03
9670 FOR X=1 TO T2
9680  IF S(X)<>-1 THEN 9690
9685  S(X)=L1
9690 NEXT X
9700 REM WE'VE PUT THE LAMP IN HOUSE AND OTHER ITEMS WHERE HE DIED
9710 L1=INT(RND(1)*4)+1:L2=L1
9720 GOTO 320
9730 REM THIRD DEATH--END OF GAME
9740 z59 = 78:gosub 7620
9750 PRINT "Oh well..."
9760 GOSUB 6430
9762 close #1:close #2:close #3
9770 STOP
9780 rem *** PITS ***
9790 if l1 < 13 THEN 300
9791 IF l = 1 and (s(18) = -1 or s(18) = l1) then 300
9800 rem IS HE GOING TO FALL INTO A PIT?
9810 if l1 = 16 or l1 = 17 or l1 = 19 or l1 = 20 or l1 = 25 or l1 = 47 or l1 = 48 or l1 = 59 or l1 = 60 or l1 = 61 or l1 = 75 or l1 = 76 or l1 = 98 then 9840
9820 goto 300
9830 rem he fell into a pit
9840 z59 = 44:gosub 7620
9850 goto 9540
9860 rem *** SEEK A "YES" OR "NO"
9870 INPUT a$
9875 if len(a$) <> 0 then 9880
9877 a$ = " "
9880 a$ = LOWER$(mid$(a$,1,1))
9890 if a$ <> "y" and a$ <> "n" then 9930
9900 if a$ = "y" then 9901 else 9910
9901 z0 = 1
9910 if a$ = "n" then 9911 else 9920
9911 z0 = 0
9920 goto 9945
9930 PRINT "Yes or No-";
9940 goto 9870
9945 return
9950 rem   ---- SHORT NAMES FOR STUFF ----
9961 data "large gold nugget"
9962 data "bars of silver"
9963 data "precious jewelry"
9964 data "many coins"
9965 data "several diamonds"
9966 data "fragile ming vase"
9967 data "glistening pearl"
9968 data "nest of golden eggs"
9969 data "jewel-encrusted trident"
9970 data "egg-sized emerald"
9971 data "platinum pyramid"
9972 data "golden chain"
9973 data "rare spices"
9974 data "persian rug"
9975 data "treasure chest"
9976 data "water"
9977 data "oil"
9978 data "brass lamp"
9979 data "keys"
9980 data "food"
9981 data "bottle"
9982 data "wicker cage"
9983 data "3-foot black rod"
9984 data "clam"
9985 data "magazine"
9986 data "bear"
9987 data "axe"
9988 data "velvet pillow"
9989 data "shards of pottery"
9990 data "oyster"
9991 data "bird"
9992 data "troll"
9993 data "dragon"
9994 data "snake"
9995 data "dwarf"
9996 data "rock"
9997 data "stairs"
9998 data "steps"
9999 data "house"
10000 data "grate"
10001 data "stream"
10002 data "room"
10003 data "bridge"
10004 data "pit"
10005 data "volcano"
10006 data "road"
10007 data "everything"
12500 rem INITIALIZE MESSAGE INDEX
12501 open "AMESSAGE.IDX" for INPUT as #6 else 12505
12502 goto 12610
12504 rem Message indx file doesn't exist, create it
12505 OPEN "AMESSAGE.IDX" FOR OUTPUT AS #6
12506 fpos = -2:b$ = "":fracnt= 0:lastfra=-1:fseek #3,0
12520 fpos = fpos+len(b$)+2
12522 INPUT #3,b$
12530 if len(b$) = 0 then 12531 else 12540
12531 b$ = " " : fpos = fpos-1
12540 if b$="#" then 12591
12550 if instr(b$,"#") = 0 then 12520
12560 z4 = val(mid$(b$,2))
12570 if int(z4) = z4 then 12580
12571 fracnt = fracnt + 1
12572 if lastfra = int(z4) then 12574
12573 indx(int(z4)) = fracnt:lastfra = int(z4):PRINT #6,int(z4);",";FRACNT
12574 fraindx(fracnt) = fpos
12576 goto 12585
12580 indx(int(z4)) = fpos
12581 PRINT #6,int(z4);",";FPOS
12585 PRINT "*";
12590 goto 12520
12591 PRINT #6,-999;",";-999
12592 FOR I = 1 TO FRACNT
12593 PRINT #6,FRAINDX(I)
12594 NEXT I
12595 PRINT #6,-999
12596 close #3
12597 open "AMESSAGE" for INPUT as #3
12600 GOTO 12670
12604 REM READ AMESSAGE.IDX FILE INTO ARRAYS
12610 INPUT #6,II,I
12612 PRINT "*";
12615 IF I=-999 THEN 12630
12620 INDX(II)=I:GOTO 12610
12630 II = 0
12640 INPUT #6,I
12641 PRINT "*";
12650 IF I=-999 THEN 12670
12660 II = II +1:FRAINDX(II)=I:GOTO 12640
12670 CLOSE #6:PRINT:PRINT
12680 RETURN

```

</details>

<details class="source-embed" markdown="1">

<summary><code>stage2-minimal-bascal/adventure.bcl</code> -- Stage 2 — minimal valid BASCAL</summary>

```bascal

// ADVENTURE/3000 -- Stage 2: minimally-changed BASCAL port.
// See ../README.md for the port's provenance and staging.
//
// Every original numbered line that's an actual goto/gosub/restore/
// on-goto/on-gosub target keeps an L<num>: label matching its original
// line number; lines nothing jumps to no longer carry one (BASCAL
// numbers unlabeled lines itself, so there was nothing to preserve by
// keeping them). A handful of syntax forms BASCAL doesn't accept
// verbatim -- numeric goto/gosub/then/else/restore targets, OPEN ...
// ELSE, and FSEEK -- are rewritten as noted inline. Everything else
// (variable names, GOSUB structure, DATA/READ tables) is untouched
// from the original.
program adventure3000

' The original used PyBASIC's UPPER$/LOWER$, which real BASIC (and BASCAL's
' own basic-target output) doesn't have; BASCAL's stdlib UCASE$/LCASE$ are
' the direct equivalent, so every UPPER$/LOWER$ call below was rewritten to
' use them instead. See ../README.md.
require com.bascal.stdlib.ucase
require com.bascal.stdlib.lcase

' ADVENTURE/3000 VERSION 3.2     27 FEB 1979 AT 5:30 PM
' THIS PROGRAM IS RELATIVELY BUG-FREE, BUT ONE STILL MUST
' realize that murphy 'S LAW STILL PREVAILS!
'
' ADVENTURE: PROGRAMMED IN HP/3000 BASIC BY BENJAMIN MOSER
' JAMES MADISON HIGH SCHOOL, VIENNA, VIRGINIA.  THE BASIC LAYOUT OF THE
' GAME WAS CONCEIVED BY DON WOODS & WILLIE CROWTHER, BOTH OF M.I.T.
'
' ADVENTURE WAS PORTED TO THE MACINTOSH PLUS BY THE ELIZABETH AND DAVID HUNTER
' IN MARCH 1998 AND THEN TO PYBASIC FOR THE RASPBERRY RP2040 IN JUNE 2021
' PRINT "Adventure 3.2 on ";date$;" at ";time$
PRINT "Adventure 3.2 for PyBASIC"
open "AMOVING" for INPUT as #4
' ADESCRIP, AITEMS, and AMESSAGE are loaded fully into memory
' below instead of opened here -- this port replaces the original's
' fseek-based random access into them, which BASCAL (and classic
' Microsoft BASIC) has no equivalent for. See ../README.md.
' dirs is an array of possible room directions, it replaces file AMOVING
dim dirs(100,10)
dim indx(303)
dim fraindx(10)
dim s(99)
dim v(100)
dim k(200)
dim o(15)
PRINT: PRINT "Initializing.";
' initialize
' total rooms, items,and keywords
l1 = int(RND(1)*4)+1 : l2 = l1
g = 0 : b0 = 1 : sn = 1 : d1 = 1 : d2 = 0 : t = 1 : b1 = 0 : b2 = 0 : p1 = 0: dead = 0
l = 0 : c = 0 : d3 = 0 : b3 = 0 : d0 = 2 : t1 = 100 : t2 = 35 : t3 = 149 : r0 = 0 : c0 = 0: c$=""
KC = 1.02
for ii = 1 to 99
    s(ii) = 0 : v(ii) = 0
end for

PRINT ".";:v(100) = 0
indx(1) = -1:indx(2)=-1
'   read in possible movement direction array
for z2 = 1 to 100
    INPUT #4,dx1,dx2,dx3,dx4,dx5,dx6,dx7,dx8,dx9,dx0
    dirs(z2,1)=dx1:dirs(z2,2)=dx2:dirs(z2,3)=dx3:dirs(z2,4)=dx4:dirs(z2,5)=dx5
    dirs(z2,6)=dx6:dirs(z2,7)=dx7:dirs(z2,8)=dx8:dirs(z2,9)=dx9:dirs(z2,10)=dx0
    PRINT ".";
end for

close #4
LA100: ' load ADESCRIP into memory (replaces fseek #1,0 + line-counted read)
dim descrip$(200)
open "ADESCRIP" for INPUT as #1
dcount = 0
LA101: if EOF(1) then goto LA102
dcount = dcount + 1
' INPUT # into an array element directly isn't supported by the minimal
' C backend yet (GitHub issue #151) -- only a bare scalar variable is --
' so read into one first, then assign it into the array.
input #1, line$
descrip$(dcount) = line$
PRINT ".";
goto LA101
LA102: close #1
LA110: ' load AITEMS into memory (replaces fseek #2,0 + full rescan)
dim items$(200)
open "AITEMS" for INPUT as #2
icount = 0
LA111: if EOF(2) then goto LA112
icount = icount + 1
input #2, line$
items$(icount) = line$
PRINT ".";
goto LA111
LA112: close #2
LA120: ' load AMESSAGE into memory (replaces AMESSAGE.IDX + fseek #3,xtmp)
dim msg$(2500)
open "AMESSAGE" for INPUT as #3
mcount = 0
LA121: if EOF(3) then goto LA122
mcount = mcount + 1
input #3, line$
msg$(mcount) = line$
goto LA121
LA122: close #3
gosub L12500
LA130: ' load the short item/object names (originally addressed by a
' computed `restore 9960+x` -- BASCAL restore targets must be a fixed
' label, not an expression, so these become an array instead)
dim itemname$(47)
restore L9961
for ianame = 1 to 47
    read itemname$(ianame)
end for
PRINT
restore L230
'    read in locations of items
for z2 = 1 to T2 step 5
    read dx1,dx2,dx3,dx4,dx5:s(z2)=dx1:s(z2+1)=dx2:s(z2+2)=dx3:s(z2+3)=dx4:s(z2+4)=dx5
    PRINT ".";
end for

L230: data 18,25,23,24,21
data 52,0,71,74,58
data 59,69,66,82,100
data 7,49,7,7,7
data 7,12,13,40,38
data 69,0,46,0,0
data 15,60,82,22,250
for ii = 1 to 15 step 5
    read dx1,dx2,dx3,dx4,dx5:o(ii)=dx1:o(ii+1)=dx2:o(ii+2)=dx3:o(ii+3)=dx4:o(ii+4)=dx5
    PRINT ".";
end for

data 1,2,2,2,2
data 3,4,3,3,2
data 5,3,2,3,3
' ASK IF HE WANTS DIRECTIONS
z59 = 301:gosub L7620
gosub L9860
if z0 then goto L267 else goto L320
L267: z59 = 302:gosub L7620
'    command INPUT routine
L300: ' PRINT room, items
' If it's dark don't let him see anything
L320: if l1 < 13 or l1 = 58 then goto L350
if l = 1 and (s(18) = l1 or s(18) = -1) then goto L350
z59 = 45:gosub L7620
goto L400
L350: on d0+1 gosub L6780,L7990,L8180
v(l1) = 1
gosub L6680
if dead = 1 then goto L9540
gosub L7800
' INPUT LOOP --- MULTIPLE COMMAANDS, REMOVE JUNK CHARACTERS
L400: if len(c$) > 0 then goto L500
L410: INPUT ">";c$:c$=ucase$(c$)
if c$ = "" then goto L410
PRINT: PRINT
c$ = ucase$(c$)
'    65-90 = A-Z               48-57 = 0-9         46 = . 44 = ,
for x = 1 to len(c$)
    z5 = asc(mid$(c$,x,1))
    if not ((z5 > 64 and z5 < 91) or (z5 > 47 and z5 < 58) or z5 = 44) then
        c$ = mid$(c$,1,x-1)+" "+mid$(c$,x+1)
    end if
end for

if mid$(c$,len(c$),1) = "," then goto L500
c$ = c$+","
L500: z4 = instr(c$,",")
a$ = ucase$(mid$(c$,1,z4-1)) : c$ = mid$(c$,z4+1)
a$ = " "+a$+" "
' search a$ for keywords,puut kwd code into k(x)
' items
L580: data 1,"GOLD",1,"NUGGET",2,"BARS",2,"SILVER",3,"JEWELRY",4,"COINS"
data 5,"DIAMONDS",6,"MING",6,"VASE",7,"PEARL",8,"EGGS",8,"NEST"
data 9,"TRIDENT",10,"EMERALD",11,"PLATINUM",11,"PYRAMID",12,"CHAIN"
data 13,"SPICES",14,"PERSIAN",14,"RUG",15,"TREASURE",15,"CHEST"
data 16,"WATER",17,"OIL",18,"LAMP",18,"LANTERN",19,"KEYS",20,"FOOD",21,"BOTTLE"
data 22,"CAGE",23,"ROD",23,"WAND",24,"CLAM",25,"MAGAZINE",26,"BEAR"
data 27,"AXE",28,"VELVET",28,"PILLOW",29,"SHARDS",30,"OYSTER"
data 31,"BIRD",32,"TROLL",33,"DRAGON",34,"SNAKE",35,"DWARF"
data 36,"ROCK",36,"BOULDER",37,"STAIRS",38,"STEPS",39,"HOUSE",39,"BUILDING"
data 40,"GRATE",41,"STREAM",42,"ROOM",43,"BRIDGE",44,"PIT",45,"VOLCANO"
data 46,"ROAD",47,"ALL",47,"EVERYTHING"
' DIRECTIONS
data 100,"N",100,"NORTH",101,"NE",101,"NORTHEAST",102,"E",102,"EAST"
data 103,"SE",103,"SOUTHEAST",104,"S",104,"SOUTH",105,"SW",105,"SOUTHWEST"
data 106,"W",106,"WEST",107,"NW",107,"NORTHWEST",108,"U",108,"UP",109,"D",109,"DOWN"
' VERBS
data 110,"PLUGH",111,"XYZZY",112,"PLOVER",113,"CROSS",114,"CLIMB",115,"JUMP"
data 116,"FILL",117,"EMPTY",117,"POUR",118,"LOOK",118,"L",119,"LIGHT",119,"ON",120,"EXTINGUISH"
data 120,"OFF",121,"IN",121,"ENTER",122,"LEAVE",122,"OUT",123,"INVENTORY",123,"I"
data 124,"GET",124,"CATCH",124,"TAKE",125,"DROP",125,"DUMP",126,"THROW",127,"ATTACK"
data 127,"KILL",128,"FEED",129,"WATER",130,"LOCK",131,"UNLOCK"
data 132,"FREE",132,"RELEASE",133,"WAVE",134,"OPEN",135,"CLOSE"
data 136,"OIL",137,"EAT",138,"DRINK",139,"FEE FIE FOE FOO"
data 140,"SHORT",141,"LONG",142,"BRIEF",143,"QUIT",143,"STOP",143,"END"
data 144,"SCORE",145,"SAVE",146,"LOAD",147,"READ",147,"EXAMINE"
data 148,"YES",148,"Y",149,"BUG",150,"*"
restore L580
for i = 1 to 200
    k(i) = 0
end for

z1 = 0 : z3 = 0
' T3=TOTAL NUMBER OF KEYWORDS
L820: if z1 > t3 then goto L900
read z1,b$
b$ = " "+b$+" "
' IF KEYWORD WAS FOUND, NOTE THIS IN K(Z1)
if instr(a$,b$) = 0 then goto L820
k(z1) = 1
' KEYWORDK #Z1 NOT FOUND
goto L820
' EXOTIC WORDS
L900: z0 = 36
x = z0
LW910: if x > 46 then goto L960
if k(x) = 0 then goto LCONT910
gosub L8390
L940: print "What do you want to do with the ";d$;"?"
goto L410
LCONT910: x = x + (1)
goto LW910
L960:
x = 110
LW970: if x > t3 then goto L990
if k(x) = 1 then goto L1950
x = x + (1)
goto LW970
L990:
' THEN IT'S A DIRECTION
d = 1
LW1010: if d > 10 then goto L1030
if k(d+99) = 1 then goto L1070
d = d + (1)
goto LW1010
L1030:
' COMMAND NOT A DIRECTION
goto L1950
' CAN HE MOVE THAT WAY?
L1070: z2 = dirs(L1,d)
if z2 = 255 then goto L1470
if z2 < 1 or z2 > 254 then goto L1220
L1150: ' NORMAL MOVING
' CHECK FOR SPECIAL MOVE CONDITIONS
goto L1260
L1180: l2 = l1 : l1 = z2
if s(35) = l2 then goto L1191 else goto L1200
L1191: s(35) = l1
L1200: goto L9780
L1220: z59 = 1
gosub L7620
goto L400
' SPECIAL ROOM DIRECTIONS
' GRATE
L1260: if L1 = 10 and (d = 10 or d = 5) THEN goto L1280
IF L1 = 11 and (d = 9 or d = 3) then goto L1280 else goto L1320
' IF GRATE IS OPEN (G=0) MOVE HIM
L1280: if g = 1 then goto L1180
z59 = 10:gosub L7620
goto L400
' CAN'T TAKE NUGGET UPPSTAIRS
L1320: if not (l1 = 17 and d = 9 and s(1) = -1) then goto L1360
z59 = 38:gosub L7620
goto L400
' CRYSTAL BRIDGE AND FISSURE
L1360: if not (l1 = 19 and d = 7 or l1 = 20 and d = 3) then goto L1410
if b2 then goto L1180
z59 = 3:gosub L7620
goto L400
' MT. KING & SNAKE
L1410: if not (l1 = 22 and d <> 3 and d <> 9) then goto L1690
if sn = 0 then goto L1180
z59 = 50:gosub L7620
goto L400
' bedquilt and random directiosn
L1470: if l1 <> 44 then goto L1590
if RND(1) > 0.5 then goto L1510
z59 = 52:gosub L7620
goto L400
L1510: restore L1530
' ROOMS TO JU
L1530: data 33,37,45,92,76
for z3 = 1 to int(RND(1)*5)+1
    read z2
end for

goto L1180
' WITT's END
L1590: if l1 <> 39 then goto L1220
' SHOULD WE LET HIM OUT?
if RND(1) < 0.15 then goto L1650
' NO
z59 = 52:gosub L7620
goto L400
L1650: ' YES
z2 = 38
goto L1180
' Narrow Tunnel
L1690: if not (l1 = 57 or l1 = 58) then goto L1780
IF K(102)=0 AND K(106)=0 THEN goto L1780
z3 = 1
LW1700: if z3 > t2 then goto L1750
if z3 = 10 then goto LCONT1700
if s(z3) <> -1 then goto LCONT1700
z59 = 53:gosub L7620
goto L400
LCONT1700: z3 = z3 + (1)
goto LW1700
L1750:
goto L1180
' TROLL
L1780: IF L1=60 AND D=2 THEN goto L1790
IF L1=61 AND D=6 THEN goto L1790 ELSE goto L1860
L1790: on t+1 goto L1180,L1800,L1820,L1840
L1800: z59 = 55:gosub L7620
goto L400
L1820: z59 = 56:gosub L7620
z59 = 55:gosub L7620
T=1:goto L400
L1840: t = 2
goto L1180
L1860: if not (l1 = 73 and d = 1 and d2 = 0) then goto L1890
z59 = 57:gosub L7620
goto L400
L1890: if not (l1 = 82 and s(33) = l1 and d = 1) then goto L1180
' DRAGON
z59 = 51:gosub L7620
goto L400
' OTHER COMMANDS
L1950: z1 = 100
LW1950: if z1 > t3 then goto L1970
if k(z1)=1 then goto L2090
z1 = z1 + (1)
goto LW1950
L1970:
' ITEM BO NO VERB?
' (was: restore L9961 -- item names now come from itemname$())
x = 1
LW2000: if x > 35 then goto L2030
d$ = itemname$(x)
if k(x) = 1 then goto L940
x = x + (1)
goto LW2000
L2030:
L2040: restore L2070
for x = 1 to int(RND(1)*4)+1
    read b$
end for

PRINT b$
L2070: data "What?","I don't understand.","I can't understand that.","I don't know that word."
goto L400
L2090: z1 = z1-109
on z1 goto L2120,L2220,L2300,L2390,L2570,L2640,L2680,L2860,L2930,L3000,L3080,L3130,L3290,L3450,L3590,L3920,L4160,L4430,L4630,L4800,L4950,L5060,L5190,L5410,L5560,L5750,L5810,L5920,L6000,L6090,L6230,L6270,L6310,L6350,L6410,L8970,L9080,L9220,L4490,L9330
goto L2040
L2120: ' *** PLUGH ***
IF L1<>7 THEN goto L2170
IF S(35)=L1 THEN S(35)=0
Z2=26
GOTO L1180
L2170: IF L1<>26 THEN goto L2200
Z2=7
GOTO L1180
L2200: z59 = 2:gosub L7620
GOTO L400
L2220: ' *** XYZZY ***
IF L1<>7 THEN goto L2270
IF S(35)=L1 THEN S(35)=0
Z2=13
GOTO L1180
L2270: IF L1<>13 THEN goto L2200
Z2=7
GOTO L1180
L2300: ' *** PLOVER *** (CAN'T BRING EMERALD WITH HIM)
IF L1>26 THEN goto L2360
IF S(35)<>L1 THEN goto L2330
S(35) = 0
L2330: IF S(10)<>-1 THEN goto L2340
S(10) = L1
L2340: Z2 = 58
GOTO L1180
L2360: IF L1<>58 THEN goto L2200
Z2=26
GOTO L1180
L2390: ' *** CROSS ***
IF L1<>19 THEN goto L2470
IF B2<>0 THEN goto L2440
L2420: z59 = 3:gosub L7620
GOTO L400
L2440: D=7
' JUST GIVE NEW DIRECTION, USE MOVE ROUTINE
GOTO L1070
L2470: IF L1<>20 THEN goto L2510
IF B2=0 THEN goto L2420
D=3
GOTO L1070
L2510: IF L1<>60 THEN goto L2540
D=2
GOTO L1070
L2540: IF L1<>61 THEN goto L2200
D=6
GOTO L1070
L2570: ' *** CLIMB ***
IF L1<>50 THEN goto L2200
' CAN HE CLIMB BEANSTALK?
IF P1<2 THEN goto L2200
' YES
Z2=70
GOTO L1180
L2640: ' *** JUMP *** STRICTLY SUICIDAL
IF L1<>16 AND L1<>19 AND L1<>20 AND L1<>27 THEN goto L2200
z59 = 4:gosub L7620
GOTO L9550
L2680: ' FILL
IF S(21)=-1 THEN goto L2730
L2700: B$="bottle"
L2710: PRINT "You don't have the ";b$
goto L410
L2730: IF B0=0 THEN goto L2760
z59 = 5:gosub L7620
GOTO L410
L2760: IF L1<>7 AND L1<>8 AND L1<>9 AND L1<>35 AND L1<>74 AND L1<>81 THEN goto L2790
B0=1:S(16)=-1
GOTO L2840
L2790: IF L1=49 THEN goto L2830
B$="oil"
L2810: PRINT "I see no ";B$;" here."
GOTO L400
L2830: B0=2:S(17)=-1
L2840: PRINT "The bottle is now filled."
GOTO L400
L2860: ' *** EMPTY ***
IF S(21)=-1 THEN goto L2890
GOTO L2700
L2890: ' EMPTY BOTTLE (ASSUMED FULL)
S(B0+15)=0:B0=0
PRINT "Emptied"
GOTO L400
L2930: ' *** LOOK ***
L2940: if l1 < 13 or l1 = 58 then goto L2970
if l = 1 and (s(18) = l1 or s(18) = -1) then goto L2970
z59 = 45:gosub L7620
goto L400
L2970: gosub L8050
gosub L6680
goto L400
L3000: ' *** LIGHT ***
if s(18) = -1 then goto L3040
L3020: b$ = "lamp"
goto L2710
L3040: l = 1
b$ = "on"
L3060: PRINT "The lamp is now ";b$
goto L2940
L3080: ' *** OFF (EXTINGUSIH) ***
IF S(18)=-1 THEN goto L3110
GOTO L3020
L3110: L=0:B$="off"
GOTO L3060
L3130: ' *** ENTER ***
IF L1<>6 THEN goto L3180
' TO HOUSE
D=3
GOTO L1070
L3180: IF L1<>68 THEN goto L3240
' TO BARREN ROOM
D=3
GOTO L1070
L3240: D = 10
LW3240: if D < 1 then goto L3270
Z2 = DIRS(L1,D)
IF Z2>0 AND Z2<101 THEN goto L1150
D = D + (-1)
goto LW3240
L3270:
GOTO L2200
L3290: ' ** LEAVE ***
IF L1<>7 THEN goto L3340
' LEAVE HOUSE
D=7
GOTO L1070
L3340: IF L1<>69 THEN goto L3400
' LEAVE BARREN ROOM
D = 7
GOTO L1070
L3400: D = 1
LW3400: if D > 10 then goto L3430
Z2 = DIRS(L1,D)
IF Z2>0 AND Z2<101 THEN goto L1150
D = D + (1)
goto LW3400
L3430:
GOTO L2200
L3450: ' *** INVENTORY ***
Z0=0
PRINT "You are carrying:";
FOR X=1 TO T2
    IF S(X)=-1 THEN
        b$ = itemname$(x)
        PRINT B$
        Z0 = Z0 + 1
    end if
end for

IF Z0=0 THEN goto L3551 else goto L3560
L3551: PRINT "nothing."
L3560: PRINT
GOTO L400
L3590: ' *** GET ***
if k(47) = 1 then goto L3680
gosub L8280
if z8 > 0 then goto L3680
PRINT "Get what?"
goto L2040
L3680: z3 = 1
LW3680: if z3 > t2 then goto L3900
if k(47) = 1 then goto L3730
if k(z3) = 0 then goto LCONT3680
L3730: if s(z3) <> l1 then goto L3750
if s(z3) = l1 then goto L3790
L3750: if k(47) = 1 then goto LCONT3680
a$ = itemname$(z3):PRINT a$;" not here."
goto LCONT3680
' MUST CHECK NOW FOR LEGALITY OF TAKING ITEM
L3790: z8 = 0
for x = 1 to t2
    if s(x) = -1 then
        z8 = z8+1
    end if
end for

if z8 < 7 then goto L3870
' CARRYING TOO MUCH
z59 = 54:gosub L7620
goto L410
L3870: goto L6880
L3880: s(z3) = -1
L3890: a$ = itemname$(z3):PRINT a$;":taken."
LCONT3680: z3 = z3 + (1)
goto LW3680
L3900:
goto L400
L3920: ' *** DROP ***
if k(47) = 1 then goto L4000
gosub L8280
IF Z8>0 THEN goto L4000
PRINT "Drop what?"
GOTO L2040
L4000: Z3 = 1
LW4000: if Z3 > T2 then goto L4140
IF K(47)=1 THEN goto L4060
IF K(Z3)<>1 THEN goto LCONT4000
IF S(Z3)=0 THEN goto LCONT4000
L4060: IF S(Z3)=-1 THEN goto L4100
IF K(47)=1 THEN goto LCONT4000
b$ = itemname$(z3):PRINT "You don't have the ";B$
GOTO LCONT4000
L4100: ' STILL NEED TO ELABORATE ON DROP (BIRD IN CAGE, BOTTLE)
GOTO L7380
L4120: b$ = itemname$(z3):PRINT B$;":dropped."
S(Z3)=L1
LCONT4000: Z3 = Z3 + (1)
goto LW4000
L4140:
GOTO L400
L4160: ' *** THROW ***
GOSUB L8280
IF Z8>0 THEN goto L4210
PRINT "Throw what?"
GOTO L2040
L4210: IF S(Z3)<>-1 THEN goto L2710
IF NOT (Z3<16 AND S(32)=L1) THEN goto L4260
' THROW TREASURE TO TROLL
z59 = 27:gosub L7620
S(Z3)=0:T=3:GOTO L400
L4260: IF NOT (Z3=27 AND S(32)=L1) THEN goto L4300
' TRYING TO BUTCHER TROLL?
z59 = 26:gosub L7620
S(27)=L1:GOTO L400
L4300: IF NOT (Z3=27 AND S(35)=L1) THEN goto L4380
' TRYING TO KILL DWARF
IF RND(1)>0.5 THEN goto L4360
z59 = 29:gosub L7620
GOSUB L8650
GOTO L4410
L4360: z59 = 30:gosub L7620
S(35)=0:GOTO L4410
L4380: ' NOTHING SPECIAL, JUST DROP ITEM
IF S(35)<>L1 THEN goto L4400
GOSUB L8550
L4400: PRINT "Thrown."
L4410: S(Z3) = L1
if dead = 1 then goto L9540
GOTO L400
L4430: ' *** ATTACK ***
GOSUB L8280
IF NOT (Z3=33 AND S(Z3)=L1 AND L1=82) THEN goto L4520
' HE CAN KILL DRAGON
z59 = 68:gosub L7620
GOTO L410
L4490: IF L1<>82 THEN goto L2040
z59 = 69:gosub L7620
S(33)=0:D1=0:GOTO L400
L4520: IF S(32)<>L1 THEN goto L4560
' TRYING TO MUNGE TROLL
' (was: Z9=FNA(25):GOTO 400 -- FNA is called with no matching DEF FN
' anywhere in the original source; this looks like leftover/broken code
' from an earlier version of the upstream port, not something this port
' introduced. Z9's assignment here isn't read before it's next assigned
' elsewhere, so dropping the call changes nothing observable.)
GOTO L400
L4560: IF NOT (Z3=26 OR Z3>30) THEN goto L4600
' DANGEROUS TO ATTACK THESE
z59 = 70:gosub L7620
GOTO L400
L4600: ' NOTHING TO ATTACK
z59 = 71:gosub L7620
GOTO L400
L4630: ' *** FEED ***
GOSUB L8280
IF Z3<>35 THEN goto L4690
' CAN'T FEED DWARF!
z59 = 24:gosub L7620
GOTO L400
L4690: IF S(20) = -1 THEN goto L4720
B$ = "FOOD":GOTO L2710
L4720: IF L1=69 THEN goto L4760
PRINT "I can't feed it."
z59 = 23:gosub L7620
GOTO L400
L4760: IF S(20)=L1 THEN goto L7600
B1=1:S(20)=0:z59 = 6:gosub L7620
GOTO L400
L4800: ' *** WATER ***
IF S(16) = -1 THEN goto L4840
B$ = "water":GOTO L2710
L4840: IF L1<>50 THEN goto L2200
' GOTO P1+1 OF 4860,4890,4920
IF P1 = 0 THEN goto L4860
IF P1 = 1 THEN goto L4890
IF P1 = 2 THEN goto L4920
L4860: z59 = 7:gosub L7620
P1=1:S(16)=0:B0=0:GOTO L400
L4890: z59 = 8:gosub L7620
P1=2:S(16)=0:B0=0:GOTO L400
L4920: z59 = 9:gosub L7620
P1=0:S(16)=0:B0=0:GOTO L400
L4950: ' *** LOCK ***
L4960: IF L1=10 OR L1=11 THEN goto L4990
' NOTHING LOCKABLE
GOTO L2200
L4990: IF S(19)=-1 THEN goto L5020
L5000: B$="keys":goto L2710
L5020: G=0:z59 = 10:gosub L7620
GOTO L400
L5060: ' *** UNLOCK ***
L5070: IF S(19)<>-1 THEN goto L5000
IF L1<>10 AND L1<>11 THEN goto L5120
G=1:z59 = 11:gosub L7620
GOTO L400
L5120: IF L1<>69 THEN goto L2200
IF B1>0 THEN goto L5160
z59 = 12:gosub L7620
goto L400
L5160: IF C<>0 THEN goto L5170
C=1:B1=2
L5170: z59 = 13:gosub L7620
GOTO L400
L5190: ' *** FREE ***
IF K(31) = 1 THEN goto L5240
' CAN'T FREE ANYTHING BUT BIRD
L5220: z59 = 2:gosub L7620
GOTO L410
L5240: IF S(31)<>-1 THEN goto L5220
S(31) = L1:B3=0
PRINT "Freed."
IF L1<>22 THEN goto L5350
IF SN<>1 THEN goto L400
B$="snake"
L5300: PRINT "The little bird attacks the green ";B$;" and"
IF L1=82 THEN goto L5380
PRINT "drives it off"
SN=0:S(34)=0:GOTO L400
L5350: IF L1<>82 THEN goto L400
B$="dragon":GOTO L5300
L5380: PRINT "gets burned to a crisp"
S(31)=0
GOTO L400
L5410: ' *** WAVE ***
IF K(23) <> 1 THEN goto L2200
IF S(23)=-1 THEN goto L5460
B$="rod":GOTO L2710
L5460: '  IS HERE NEAR FISSURE
IF L1<>19 AND L1<>20 THEN goto L2200
' yes
' GOTO B2+1 OF 5500,5530
IF B2=0 THEN goto L5500
IF B2=1 THEN goto L5530
L5500: z59 = 14:gosub L7620
B2=1:GOTO L400
L5530: z59 = 15:gosub L7620
B2=0:GOTO L400
L5560: ' *** OPEN ***
GOSUB L8280
IF Z3>0 THEN goto L5610
PRINT "Open ";:goto L2040
L5610: IF Z3=40 THEN goto L5070
IF S(Z3)=L1 THEN goto L5650
L5630: PRINT "I see no ";b$;" here.":goto L400
L5650: if z3=24 THEN goto L5680
PRINT "I don't know how to open a ";B$:GOTO L400
L5680: IF S(9)=-1 THEN goto L5710
z59 = 16:gosub L7620
GOTO L400
L5710: IF S(Z3) = 0 THEN goto L2200
' HE'S OPENED CLAM, SO PRINT DESCRIPTION OF THIS
' PUT PEARL IN CUL-DE-SAC
S(7)=43:S(24)=0:S(30)=L1:z59 = 17:gosub L7620
L5750: GOTO L400
' *** CLOSE ***
GOSUB L8280
IF Z3=40 THEN goto L4960
z59 = 18:gosub L7620
GOTO L400
L5810: ' OIL
IF K(17)=0 THEN goto L2200
IF S(17)=-1 THEN goto L5860
B$="oil":GOTO L5630
L5860: IF L1<>73 THEN goto L2200
' IS DOOR STILL RUSTED
IF D2=1 THEN goto L2200
D2=1:S(17)=0:B0=0:z59 = 19:gosub L7620
GOTO L400
' *** EAT ***
L5920: IF K(20) = 1 THEN goto L5950
z59 = 20:gosub L7620
GOTO L410
L5950: Z3=20:GOSUB L8490
IF Z5=0 THEN goto L410
z59 = 73:gosub L7620
S(20)=0:B0=0:GOTO L400
L6000: ' *** DRINK ***
IF K(16) =1 THEN goto L6040
z59 = 21:gosub L7620
GOTO L410
L6040: Z3=16:GOSUB L8490
IF Z5=0 THEN goto L410
z59 = 22:gosub L7620
S(17)=0:B0=0:GOTO L400
L6090: ' *** FEE FIE FOE FOO ***
IF L1=71 THEN goto L6130
z59 = 2:gosub L7620
GOTO L410
L6130: IF S(8)<>L1 THEN goto L6180
' MAKE NEST VANISH
z59 = 79:gosub L7620
S(8)=0:GOTO L400
L6180: ' IF S(8)=0 THEN goto L6110
S(8)=L1
' MAKE NEST RE-APPEAR
z59 = 81:gosub L7620
GOTO L400
L6230: ' *** SHORT ***
PRINT "Short descriptions"
D0=0:GOTO L400
L6270: ' *** LONG ***
PRINT "Long descriptions"
D0=1:GOTO L400
L6310: ' *** BRIEF ***
PRINT "OK, I'll only describe the room in detail the first time."
D0=2:GOTO L400
L6350: ' *** QUIT ***
PRINT "Save game";
GOSUB L9860
IF Z0=1 THEN goto L8970
GOTO L9750
' SCORE ***
L6410: GOSUB L6430
GOTO L400
L6430: ' PRINT OUT SCORE DATA
GOSUB L6510
PRINT "Your score is now ";S0
PRINT "You have explored ";(Z9/T1)*T1;"% of the cave."
RESTORE L6470
L6470: DATA "beginner","novice","experienced","advanced","expert"
Z9 = INT((S0-1)/100)
IF Z9 <= 4 THEN goto L6483
Z9=4
L6483: FOR Z0=0 TO Z9
    READ D$
end for

PRINT "That makes you a ";D$;" adventurer."
RETURN
L6510: ' COMPUTE CURRENT SCORE
RESTORE L230
Z9=0:S0=0
FOR Z0=1 TO 15
    READ Z1
    IF Z1<>0 THEN
        IF V(Z1)=1 THEN
            S0=S0+4*O(Z0)
        end if
        IF S(Z0)=7 THEN
            S0=S0+4*O(Z0)
        end if
    end if
end for

S0=(G=1)*10 + S0:S0=(SN=0)*20 + S0:S0=(D1=0)*30 + S0:S0=(T=0)*30 + S0:S0=(B1=2)*20 + S0
S0=(B2=1)*20 + S0:S0=(P1=2)*20 + S0:S0=(D2=1)*20 + S0:S0=(C=1)*20 + S0
FOR Z0 = 1 TO T1
    IF V(Z0)=1 THEN
        S0=S0+1:Z9=Z9+1
    end if
end for

RETURN
' list items at location l1
L6680: ' (was: fseek #2,0)
for z1 = 1 to t2
    if s(z1) = l1 then PRINT items$(z1)
end for
IF S(26)<>-1 THEN goto L6730
z59 = 67:gosub L7620
L6730: ' CHECK FOR DWARF,PIRATE
gosub L8560
if dead = 1 then goto L6770
gosub L8800
PRINT
L6770: return
' Print Short room description
L6780: ' (was: fseek #1,0 + read l1 lines forward)
a$ = descrip$(l1)
v(l1) = 1
PRINT a$
return
L6880: ' SPECIAL GETS
if not (z3 = 24 or z3 = 30 or z3 > 31) then goto L6930
' CAN'T GET THESE FOR SOME REASON
z59 = 61:gosub L7620
goto L400
L6930: if not (z3 = 12 and c = 0) then goto L6970
' CHAIN
z59 = 58:gosub L7620
goto L3900
L6970: ' BEAR IS HE FED? UNLOCKED?
if not (z3 = 26 and b1 <> 2) then goto L7010
z59 = 61:gosub L7620
goto L3900
L7010: if not (z3 = 14 and d1 = 1) then goto L7050
' DRAGON AND RUG
z59 = 59:gosub L7620
goto L3900
L7050: if not (z3 = 16 or z3 = 17) then goto L7090
' OIL AND WATER DO SAME AS FILL
PRINT "Why not say 'fill'?"
goto L3900
L7090: if not (z3 = 22 and b3) then goto L7140
' TAKE BIRD SINCE IT'S IN CAGE
s(31) = -1:PRINT "Bird and ";:goto L3880
L7140: if z3 <> 31 then goto L7310
' GETTING BIRD
if b3 <> 1 then goto L7210
' TAKE CAGE, SINCE BIRD IS IN IT
PRINT "Cage and ";:s(22) = -1:goto L3880
L7210: if s(22) = -1 then goto L7240
b$ = "cage":goto L2810
L7240: if s(23) = -1 then goto L7280
' OK TO TAKE BIRD
s(31) = -1 : b3 = 1:goto L3890
L7280: ' ROD SCARES BIRD
z59 = 37:gosub L7620
goto L3900
L7310: ' BOTTLE FULL? IF SO, GET CONTENTS
if not (z3 = 21 and b0) then goto L7360
PRINT "Contents and the ";
s(b0+15) = -1
L7360: goto L3880
' SPECIAL "DROP"
L7380: IF Z3<>31 THEN goto L7440
' BIRD IN CAGE
L7400: S(31)=L1:S(22)=L1:B3=1
IF Z3<>31 THEN goto L7420
PRINT "Cage and ";
L7420: IF Z3=22 THEN goto L7430
PRINT "Bird and ";
L7430: goto L4120
L7440: if z3=22 and b3=1 then goto L7400
IF Z3<>21 THEN goto L7520
' BOTTLE
IF B0=0 THEN goto L4120
' BOTTLE IS FULL, DO DROP CONTENTS TOO
PRINT "Contents and ";
S(15+B0)=L1:GOTO L4120
L7520: IF NOT (Z3=16 OR Z3=17) THEN goto L7541
PRINT "Try saying 'empty'":goto L4140
L7541: IF Z3<>26 OR T<>1 OR (L1<>60 AND L1<>61) THEN goto L7550
z59 = 28:gosub L7620
T=0:S(26)=L1:S(32)=0:GOTO L400
L7550: IF Z3<>6 THEN goto L4120
IF S(28)=L1 THEN goto L7600
' GOODBYE, FRAGILE VASE!
z59 = 43:gosub L7620
S(6)=0:S(29)=L1:GOTO L400
L7600: z59 = 60:gosub L7620
GOTO L4120
' PRINT MESSAGE
L7620: z59 = int(z59):xtmp = indx(z59)
if z59 <> 2 and z59 <> 61 then goto L7660
xtmp = fraindx(xtmp+int(RND(1)*5)) ' (was: ...+min(int(RND(1)*5),5) -- BASIC has no MIN, and int(RND(1)*5) is always 0-4, so min(_,5) was always a no-op)
L7660: mpos = xtmp
b1$ = msg$(mpos)
if mid$(b1$,1,1) = "#" and int(val(mid$(b1$,2))) = z59 then goto L7720
goto L7760
L7720: mpos = mpos + 1
b1$ = msg$(mpos)
if mid$(b1$,1,1) = "#" then goto L7770
PRINT b1$
goto L7720
L7760: PRINT "NO DESC. # ";z59;" IN FILE AMESSAGE"
L7770: return
L7800: '
' SITUATION DESCRIPTIONS
'
' GRATE
if l1 = 10 or l1 = 11 then goto L7842 else goto L7860
L7842: z59 = (g+10):gosub L7620
' CRYSTAL BRIDGE
L7860: if (l1 = 19 or l1 = 20) and b2 = 1 then goto L7861 else goto L7880
L7861: z59 = 14:gosub L7620
' PLUGH NOISE
L7880: if l1 = 26 and RND(1) > 0.3 then goto L7881 else goto L7900
L7881: z59 = 41:gosub L7620
' IRON DOOR
L7900: if l1 = 73 and d2 = 0 then goto L7901 else goto L7920
L7901: z59 = 57:gosub L7620
' TROLL
L7920: if (l1 = 60 or l1 = 61) and t = 1 then goto L7921 else goto L7940
L7921: z59 = 63:gosub L7620
' BEAR
L7940: if l1 = 69 and b1 = 0 then goto L7941 else goto L7950
L7941: z59 = 64:gosub L7620
L7950: if l1 = 69 and b1 = 1 then goto L7951 else goto L7970
L7951: z59 = 66:gosub L7620
' PLANT IN PIT
L7970: if l1 = 48 or l1 = 50 then goto L7971 else goto L7980
L7971: z59 = 47+p1:gosub L7620
L7980: return
L7990: '
'   PRINT long room description from "amessage" file
'   description is noormally l1+200 except for
'   maze or forest
' set v(l1)=1 so as not to repeat long desc(brief mode
v(l1) = 1
L8050: if l1 > 4 then goto L8080
z59 = 200:gosub L7620
goto L8130
L8080: if not (l1 > 88 and l1 < 98 or l1 = 99) then goto L8110
z59 = 288:gosub L7620
goto L8130
L8110: ' normal description
z59 = 200+l1:gosub L7620
L8130: return
L8180: ' always give long description for forest and maze
if l1 < 5 or (l1 > 88 and l1 < 98) or l1 = 99 then goto L8220
if v(l1)=1 then goto L8201 else goto L8220
L8201: gosub L6780
goto L8235
' he hassn't seen this room, so give a long desc.
L8220: v(l1) = 1
gosub L7990
L8235: return
'
'   fetch first item code in k(1 to t2)
'   z8=total # of items found in list
'   z3=item code first found
L8280: z8 = 0 : z3 = 0: d$ = ""
for z5 = 1 to 45
    if k(z5) <> 0 then
        z8 = z8+1
        b$ = itemname$(z5):d$ = b$
        if k(z5)=1 and z8 = 1 then
            z3 = z5
        end if
    end if
end for

b$ = d$
return
L8390: ' FIND FIRST ITEM AT ROOM
x1 = 0
FOR Z1=1 TO 47
    if x1<>1 then
        IF K(Z1) = 1 THEN
            d$ = itemname$(z1):x1=1
        end if
    end if
end for

RETURN
'
' MAKE SURE HE'S CARRYING ITEM * Z3
'
L8490: IF S(Z3)=-1 THEN goto L8530
PRINT "You don't have the ";A$
Z5=0
RETURN
L8530: Z5=1
RETURN
L8550: '  *** DWARF ***
L8560: if d3 <> 0 then goto L8640
' SHOULD DWARF GIVE AWAY AXE?
if l1 < 13 then goto L8790
if RND(1) > 0.05 then goto L8790
' GIVE AWAY AXE
z59 = 80:gosub L7620
s(27) = l1 : d3 = 1
goto L8790
L8640: ' SHOULD DWARF ATTACK?
L8650: if L1 >= 13 then goto L8660
s(35) = 0:goto L8790
L8660: if s(35) <> L1 then goto L8770
if (l1 <> 60 and l1 <> 61) or t <> 1 then goto L8670
z59 = 299:gosub L7620
s(35) = 0: goto L8790
L8670: if RND(1) > 0.5 then goto L8790
' YES!
z59 = 32:gosub L7620
' DOES THE KNIFE KILL THE PLAYER?
KC = KC - 0.02
IF KC >= 0.75 THEN goto L8710
KC = 0.75
L8710: if RND(1) <= KC then goto L8750
' YES
PRINT "It gets you!"
dead = 1 : goto L8790
L8750: PRINT "It misses!"
goto L8790
L8770: ' SHOULD WE PUT A DWARF HERE?
if RND(1) >= 0.05 then goto L8790
if (l1 = 60 or l1 = 61) and t = 1 then goto L8790
s(35) = l1
z59=31:gosub L7620
L8790: return
L8800: ' *** PIRATE ***
' FIRST, DOES HE HAVE ANYTHING WORTH STEALING?
z3 = 0
if l1 < 13 then goto L8960
for x = 1 to 15
    if s(x) = -1 then
        z3 = z3+1
    end if
end for

if z3 < int(RND(1)*4)+1 then goto L8960
' SHOULD WE RIP OFF HIS VALUABLES?
if RND(1) < 0.05 then goto L8920
z59 = 34:gosub L7620
goto L8960
L8920: z59 = 33:gosub L7620
for x = 1 to 15
    if s(x) = -1 then
        s(x) = 100
    end if
end for

L8960: return
L8970: ' *** SAVE GAME ***
INPUT "What do you want to call the save file? ";A$
' `on error goto` (not `try`/`catch`) here on purpose -- see the
' README's own note on why, and GitHub issues #61 (on error goto
' is permanently unsupported under --target c, by design) & #100
' (try/catch's RESUME output isn't accepted by real fbc).
on error goto L9010
OPEN A$ FOR OUTPUT AS #5
on error goto 0
GOTO L9030
L9010: PRINT "File ";a$;" not created"
GOTO L410
L9030: PRINT #5,T1;",";T2;",";T3;",";L1;",";L2;",";G;",";B0;",";SN;",";D1;",";D2;",";D0;",";T;",";B1;",";B2;",";P1;",";L;",";C;",";D3;",";B3;",";R0;",";KC
FOR X=1 TO 99
    PRINT #5,S(X);",";V(X)
end for

PRINT #5,V(100)
CLOSE #5
PRINT "Game saved"
C0 = 0
if k(143) = 1 then goto L9750
GOTO L410
L9080: ' *** LOAD OLD GAME ***
IF C0=0 THEN goto L9120
PRINT "You already have a loaded game!"
GOTO L410
L9120: INPUT "Save file name? ";A$
on error goto L9150
OPEN A$ FOR INPUT AS #5
on error goto 0
GOTO L9170
L9150: PRINT "Unable to use file ";A$
GOTO L410
L9170: INPUT #5,T1,T2,T3,L1,L2,G,B0,SN,D1,D2,D0,T,B1,B2,P1,L,C,D3,B3,R0,KCX$
kc = val(kcx$)
FOR X=1 TO 99
    INPUT #5,SX,VX:s(x)=sx:v(x)=vx
end for

INPUT #5,vx:v(100)=vx
CLOSE #5
C0=1
GOTO L300
L9220: ' *** READ THE MAGAZINE ***
GOSUB L8280
IF Z3=25 THEN goto L9270
z59 = 74:gosub L7620
GOTO L410
L9270: IF S(25)=-1 THEN goto L9300
B$="magazine"
GOTO L2710
L9300: ' OK, LET HIM READ IT
z59 = 303:gosub L7620
GOTO L400
L9330: ' *** BUG ***
A$ = "ADVBUGS.TXT"
on error goto L9150
OPEN A$ FOR APPEND AS #5
on error goto 0
INPUT "Your name: ";A$
A$=A$+" "+DATE$
PRINT #5,A$
PRINT "Enter your gripe in up to five lines (hit return to quit):"
Z0 = 1
LW9430: if Z0 > 5 then goto L9480
PRINT Z0;
INPUT A$
IF A$="" THEN goto L9490
PRINT #5,A$
Z0 = Z0 + (1)
goto LW9430
L9480:
L9490: PRINT "Message recorded. Thank you!"
CLOSE #5
GOTO L410
L9540: ' REINCARNATE HIM
L9550: R0=R0+1
IF R0=1 THEN goto L9580
IF R0=2 THEN goto L9610
IF R0=3 THEN goto L9740
' ASK HIM IF HE WANTS TO BE REINCARNATED
L9580: z59 = 75:gosub L7620
GOSUB L9860
GOTO L9630
L9610: z59 = 77:gosub L7620
GOTO L9580
L9630: IF Z0=0 THEN goto L9750
z59 = 76:gosub L7620
' PUT HIM BACK IN HOUSE, REARRANGE HIS STUFF
S(18)=7:L=0:DEAD=0:KC=1.03
FOR X=1 TO T2
    IF S(X)=-1 THEN
        S(X)=L1
    end if
end for

' WE'VE PUT THE LAMP IN HOUSE AND OTHER ITEMS WHERE HE DIED
L1=INT(RND(1)*4)+1:L2=L1
GOTO L320
' THIRD DEATH--END OF GAME
L9740: z59 = 78:gosub L7620
L9750: PRINT "Oh well..."
GOSUB L6430
' (ADESCRIP/AITEMS/AMESSAGE are in-memory arrays now -- nothing to close)
STOP
L9780: ' *** PITS ***
if l1 < 13 THEN goto L300
IF l = 1 and (s(18) = -1 or s(18) = l1) then goto L300
' IS HE GOING TO FALL INTO A PIT?
if l1 = 16 or l1 = 17 or l1 = 19 or l1 = 20 or l1 = 25 or l1 = 47 or l1 = 48 or l1 = 59 or l1 = 60 or l1 = 61 or l1 = 75 or l1 = 76 or l1 = 98 then goto L9840
goto L300
' he fell into a pit
L9840: z59 = 44:gosub L7620
goto L9540
L9860: ' *** SEEK A "YES" OR "NO"
L9870: INPUT a$
if len(a$) <> 0 then goto L9880
a$ = " "
L9880: a$ = lcase$(mid$(a$,1,1))
if a$ <> "y" and a$ <> "n" then goto L9930
if a$ = "y" then goto L9901 else goto L9910
L9901: z0 = 1
L9910: if a$ = "n" then goto L9911 else goto L9920
L9911: z0 = 0
L9920: goto L9945
L9930: PRINT "Yes or No-";
goto L9870
L9945: return
'   ---- SHORT NAMES FOR STUFF ----
L9961: data "large gold nugget"
data "bars of silver"
data "precious jewelry"
data "many coins"
data "several diamonds"
data "fragile ming vase"
data "glistening pearl"
data "nest of golden eggs"
data "jewel-encrusted trident"
data "egg-sized emerald"
data "platinum pyramid"
data "golden chain"
data "rare spices"
data "persian rug"
data "treasure chest"
data "water"
data "oil"
data "brass lamp"
data "keys"
data "food"
data "bottle"
data "wicker cage"
data "3-foot black rod"
data "clam"
data "magazine"
data "bear"
data "axe"
data "velvet pillow"
data "shards of pottery"
data "oyster"
data "bird"
data "troll"
data "dragon"
data "snake"
data "dwarf"
data "rock"
data "stairs"
data "steps"
data "house"
data "grate"
data "stream"
data "room"
data "bridge"
data "pit"
data "volcano"
data "road"
data "everything"
L12500: ' INITIALIZE MESSAGE INDEX (in-memory version -- replaces the
' original AMESSAGE.IDX disk cache + fseek-based lookup; see README)
fpos = 0:fracnt = 0:lastfra = -1
L12520: fpos = fpos + 1
if fpos > mcount then goto L12680
b$ = msg$(fpos)
if b$ = "#" then goto L12680
if instr(b$,"#") = 0 then goto L12520
z4 = val(mid$(b$,2))
if int(z4) = z4 then goto L12580
fracnt = fracnt + 1
if lastfra = int(z4) then goto L12574
indx(int(z4)) = fracnt:lastfra = int(z4)
L12574: fraindx(fracnt) = fpos
goto L12520
L12580: indx(int(z4)) = fpos
goto L12520
L12680: RETURN

```

</details>

<details class="source-embed" markdown="1">

<summary><code>stage3-refactored-bascal/adventure.bcl</code> -- Stage 3 — clean GOSUBs become procedures</summary>

```bascal

// ADVENTURE/3000 -- Stage 3: partial refactor toward idiomatic BASCAL.
// See ../README.md for the port's provenance, staging, and exactly how
// far this refactor has gotten so far (it is NOT a complete rewrite).
//
// Refactored so far: the startup file-loading sequence (previously
// inline GOTO-based code under LA1xx labels) is now a handful of named
// procedures, and the "seek a yes or no" GOSUB is now a real function.
// Everything else -- the actual game loop, room logic, combat, puzzles,
// and the ~70-site message-printing GOSUB -- is still exactly stage 2's
// label/GOTO/GOSUB structure, unrefactored.
program adventure3000

' The original used PyBASIC's UPPER$/LOWER$, which real BASIC (and BASCAL's
' own basic-target output) doesn't have; BASCAL's stdlib UCASE$/LCASE$ are
' the direct equivalent, so every UPPER$/LOWER$ call below was rewritten to
' use them instead. See ../README.md.
require com.bascal.stdlib.ucase
require com.bascal.stdlib.lcase

' Global arrays the new procedures below operate on via `global` --
' moved up from where stage 2 first `dim`s them inline, so every
' procedure that needs one can declare it `global` regardless of where
' in the file it's defined.
dim descrip$(200)
dim items$(200)
dim msg$(2500)
dim indx(303)
dim fraindx(10)
dim itemname$(47)

' Loads ADESCRIP (room descriptions) into descrip$(), indexed 1..dcount
' by file line order. Replaces stage 2's inline LA100-LA102 loading code.
procedure loadAdescrip()
    global descrip$
    dim dcount, line$
    open "ADESCRIP" for INPUT as #1
    while not(EOF(1))
        dcount = dcount + 1
        ' INPUT # into an array element directly isn't supported by the
        ' minimal C backend yet (GitHub issue #151) -- only a bare
        ' scalar variable is -- so read into one first, then assign it
        ' into the array.
        input #1, line$
        descrip$(dcount) = line$
        print ".";
    end while
    close #1
end procedure

' Loads AITEMS (item-at-location messages) into items$(). Replaces
' stage 2's inline LA110-LA112 loading code.
procedure loadAitems()
    global items$
    dim icount, line$
    open "AITEMS" for INPUT as #2
    while not(EOF(2))
        icount = icount + 1
        input #2, line$
        items$(icount) = line$
        print ".";
    end while
    close #2
end procedure

' Loads AMESSAGE (the numbered "#N" message table) into msg$(1..mcount).
' Replaces stage 2's inline LA120-LA122 loading code.
procedure loadAmessage()
    global msg$
    global mcount
    dim line$
    mcount = 0
    open "AMESSAGE" for INPUT as #3
    while not(EOF(3))
        mcount = mcount + 1
        input #3, line$
        msg$(mcount) = line$
    end while
    close #3
end procedure

' Scans the in-memory msg$() built by loadAmessage() and records where
' each numbered message starts, so printMessage's `gosub L7620` can jump
' straight there instead of scanning from the top every time. Replaces
' the original AMESSAGE.IDX disk cache and this port's own stage-2
' in-memory equivalent (the old `gosub L12500` target).
procedure buildMessageIndex()
    global msg$
    global mcount
    global indx
    global fraindx
    dim fpos, fracnt, lastfra, b$, z4
    fpos = 0 : fracnt = 0 : lastfra = -1
    while fpos <= mcount
        fpos = fpos + 1
        if fpos > mcount then
            return
        end if
        b$ = msg$(fpos)
        if b$ = "#" then
            return
        end if
        if instr(b$, "#") <> 0 then
            z4 = val(mid$(b$, 2))
            if int(z4) = z4 then
                indx(int(z4)) = fpos
            else
                fracnt = fracnt + 1
                if lastfra <> int(z4) then
                    indx(int(z4)) = fracnt
                    lastfra = int(z4)
                end if
                fraindx(fracnt) = fpos
            end if
        end if
    end while
end procedure

' Loads the short item/object names (DATA 9961 onward) into itemname$(),
' addressed by object code 1-47 instead of the original's computed
' `restore 9960+x` (BASCAL restore targets are fixed labels, not
' expressions).
procedure loadItemNames()
    global itemname$
    dim ianame
    restore L9961
    for ianame = 1 to 47
        read itemname$(ianame)
    end for
end procedure

' Reads a line and returns 1 for "yes", 0 for "no" -- reprompting until
' it gets one. Replaces the original's `gosub 9860` + global z0.
function askYesNo%()
    dim a$
    while true
        input a$
        if len(a$) = 0 then
            a$ = " "
        end if
        a$ = lcase$(mid$(a$, 1, 1))
        if a$ = "y" then
            return 1
        elseif a$ = "n" then
            return 0
        end if
        print "Yes or No-";
    end while
    return 0 ' unreachable -- every path through the loop above already
    ' returns; the minimal C backend requires a function's literal last
    ' top-level statement to be `return`, though, since it doesn't try
    ' to prove a loop always returns the way the basic target's own
    ' fallthrough-is-fine model doesn't need to
end function

' Prints the numbered message from AMESSAGE (msg$()/indx()/fraindx(),
' built by buildMessageIndex()). Replaces the original `z59 = N : gosub
' 7620` convention -- every call site is now `printMessage(N)`, passing
' the message number as a real argument instead of setting the global
' z59 first.
procedure printMessage(msgNum%)
    global indx
    global fraindx
    global msg$
    dim xtmp, mpos, b1$
    xtmp = indx(msgNum%)
    if msgNum% = 2 or msgNum% = 61 then
        ' Messages 2 and 61 have several interchangeable variants
        ' (see AMESSAGE's "#2.1".."#2.5" entries); pick one at random.
        xtmp = fraindx(xtmp + int(RND(1) * 5))
    end if
    mpos = xtmp
    b1$ = msg$(mpos)
    if mid$(b1$, 1, 1) <> "#" or int(val(mid$(b1$, 2))) <> msgNum% then
        print "NO DESC. # "; msgNum%; " IN FILE AMESSAGE"
        return
    end if
    while true
        mpos = mpos + 1
        b1$ = msg$(mpos)
        if mid$(b1$, 1, 1) = "#" then
            return
        end if
        print b1$
    end while
end procedure

' Prints the room's short (one-line) description and marks it visited.
' Replaces stage 2's L6780 GOSUB target.
procedure shortDescription()
    global l1
    global v
    global descrip$
    dim a$
    a$ = descrip$(l1)
    v(l1) = 1
    print a$
end procedure

' Prints the room's long (multi-paragraph, AMESSAGE-driven) description
' and marks it visited. Message number is normally l1+200, except a
' shared message for the forest (any l1 <= 4) and one for the maze-like
' rooms near the end. Replaces stage 2's L7990 GOSUB target.
procedure longDescription()
    global l1
    global v
    v(l1) = 1
    if l1 <= 4 then
        printMessage(200)
    elseif (l1 > 88 and l1 < 98) or l1 = 99 then
        printMessage(288)
    else
        printMessage(200 + l1)
    end if
end procedure

' Lists the items sitting in the current room, checks whether the golden
' bird's message should play, and then checks the dwarf and pirate
' encounters. Replaces stage 2's L6680 GOSUB target.
procedure describeRoomContents()
    global l1
    global t2
    global s
    global dead
    global items$
    for z1 = 1 to t2
        if s(z1) = l1 then
            print items$(z1)
        end if
    end for
    if s(26) = -1 then
        printMessage(67)
    end if
    ' CHECK FOR DWARF, PIRATE
    checkDwarf()
    if dead <> 1 then
        checkPirate()
        print
    end if
end procedure

' Checks whether the dwarf gives away his axe (first time in a deep
' room), attacks with his knife, or appears for the first time.
' Structurally identical to stage 2's L8560 GOSUB target -- same
' internal labels, `goto L8790` (the old shared exit) just became
' `return`, and the final label+return collapsed into the implicit
' return at the end of the procedure body.
procedure checkDwarf()
    global d3
    global l1
    global s
    if d3 <> 0 then
        checkDwarfAttack()
        return
    end if
    ' SHOULD DWARF GIVE AWAY AXE?
    if l1 < 13 then return
    if RND(1) > 0.05 then return
    ' GIVE AWAY AXE
    printMessage(80)
    s(27) = l1 : d3 = 1
end procedure

' Just the "should the dwarf attack" half of checkDwarf() -- one call
' site (stage 2's L4340, `GOSUB 8650`) jumps directly into the middle of
' the original subroutine to run only this part, deliberately skipping
' the axe-giving check above. Structurally identical to stage 2's
' L8640-L8790 span.
procedure checkDwarfAttack()
    global l1
    global s
    global t
    global dead
    global KC
    if L1 >= 13 then goto L8660
    s(35) = 0 : return
    L8660: if s(35) <> L1 then goto L8770
    if (l1 <> 60 and l1 <> 61) or t <> 1 then goto L8670
    printMessage(299)
    s(35) = 0 : return
    L8670: if RND(1) > 0.5 then return
    ' YES!
    printMessage(32)
    ' DOES THE KNIFE KILL THE PLAYER?
    KC = KC - 0.02
    IF KC >= 0.75 THEN goto L8710
    KC = 0.75
    L8710: if RND(1) <= KC then goto L8750
    ' YES
    PRINT "It gets you!"
    dead = 1
    return
    L8750: PRINT "It misses!"
    return
    L8770: ' SHOULD WE PUT A DWARF HERE?
    if RND(1) >= 0.05 then return
    if (l1 = 60 or l1 = 61) and t = 1 then return
    s(35) = l1
    printMessage(31)
end procedure

' Checks whether the pirate steals the player's valuables (only in
' rooms l1 >= 13). Structurally identical to stage 2's L8800 GOSUB
' target, with `goto L8960` (the old shared exit) replaced by `return`.
procedure checkPirate()
    global l1
    global s
    dim z3, x
    ' FIRST, DOES HE HAVE ANYTHING WORTH STEALING?
    z3 = 0
    if l1 < 13 then return
    for x = 1 to 15
        if s(x) = -1 then
            z3 = z3 + 1
        end if
    end for
    if z3 < int(RND(1) * 4) + 1 then return
    ' SHOULD WE RIP OFF HIS VALUABLES?
    if RND(1) < 0.05 then
        printMessage(33)
        for x = 1 to 15
            if s(x) = -1 then
                s(x) = 100
            end if
        end for
    else
        printMessage(34)
    end if
end procedure

' On entering a room: the forest and maze-like rooms always get their
' long description (it's short enough not to be worth abbreviating);
' every other room gets the long description only the first time it's
' visited, and the short one on repeat visits. Replaces stage 2's L8180
' GOSUB target, itself called from the `on d0+1 gosub` dispatch below.
procedure describeRoomOnEntry()
    global l1
    global v
    if l1 < 5 or (l1 > 88 and l1 < 98) or l1 = 99 then
        longDescription()
    elseif v(l1) = 1 then
        shortDescription()
    else
        longDescription()
    end if
end procedure

' ADVENTURE/3000 VERSION 3.2     27 FEB 1979 AT 5:30 PM
' THIS PROGRAM IS RELATIVELY BUG-FREE, BUT ONE STILL MUST
' realize that murphy 'S LAW STILL PREVAILS!
'
' ADVENTURE: PROGRAMMED IN HP/3000 BASIC BY BENJAMIN MOSER
' JAMES MADISON HIGH SCHOOL, VIENNA, VIRGINIA.  THE BASIC LAYOUT OF THE
' GAME WAS CONCEIVED BY DON WOODS & WILLIE CROWTHER, BOTH OF M.I.T.
'
' ADVENTURE WAS PORTED TO THE MACINTOSH PLUS BY THE ELIZABETH AND DAVID HUNTER
' IN MARCH 1998 AND THEN TO PYBASIC FOR THE RASPBERRY RP2040 IN JUNE 2021
' PRINT "Adventure 3.2 on ";date$;" at ";time$
PRINT "Adventure 3.2 for PyBASIC"
open "AMOVING" for INPUT as #4
' ADESCRIP, AITEMS, and AMESSAGE are loaded fully into memory
' below instead of opened here -- this port replaces the original's
' fseek-based random access into them, which BASCAL (and classic
' Microsoft BASIC) has no equivalent for. See ../README.md.
' dirs is an array of possible room directions, it replaces file AMOVING
dim dirs(100,10)
' indx()/fraindx() are now dim'd near the top of the file (see the
' procedure declarations), alongside descrip$()/items$()/msg$()/itemname$().
dim s(99)
dim v(100)
dim k(200)
dim o(15)
PRINT: PRINT "Initializing.";
' initialize
' total rooms, items,and keywords
l1 = int(RND(1)*4)+1 : l2 = l1
g = 0 : b0 = 1 : sn = 1 : d1 = 1 : d2 = 0 : t = 1 : b1 = 0 : b2 = 0 : p1 = 0: dead = 0
l = 0 : c = 0 : d3 = 0 : b3 = 0 : d0 = 2 : t1 = 100 : t2 = 35 : t3 = 149 : r0 = 0 : c0 = 0: c$=""
KC = 1.02
for ii = 1 to 99
    s(ii) = 0 : v(ii) = 0
end for

PRINT ".";:v(100) = 0
indx(1) = -1:indx(2)=-1
'   read in possible movement direction array
for z2 = 1 to 100
    INPUT #4,dx1,dx2,dx3,dx4,dx5,dx6,dx7,dx8,dx9,dx0
    dirs(z2,1)=dx1:dirs(z2,2)=dx2:dirs(z2,3)=dx3:dirs(z2,4)=dx4:dirs(z2,5)=dx5
    dirs(z2,6)=dx6:dirs(z2,7)=dx7:dirs(z2,8)=dx8:dirs(z2,9)=dx9:dirs(z2,10)=dx0
    PRINT ".";
end for

close #4
LA100: ' Load ADESCRIP, AITEMS, AMESSAGE (+ its index), and the short
' item names -- see the procedure declarations near the top of this
' file for what each replaces.
loadAdescrip()
loadAitems()
loadAmessage()
buildMessageIndex()
loadItemNames()
PRINT
restore L230
'    read in locations of items
for z2 = 1 to T2 step 5
    read dx1,dx2,dx3,dx4,dx5:s(z2)=dx1:s(z2+1)=dx2:s(z2+2)=dx3:s(z2+3)=dx4:s(z2+4)=dx5
    PRINT ".";
end for

L230: data 18,25,23,24,21
data 52,0,71,74,58
data 59,69,66,82,100
data 7,49,7,7,7
data 7,12,13,40,38
data 69,0,46,0,0
data 15,60,82,22,250
for ii = 1 to 15 step 5
    read dx1,dx2,dx3,dx4,dx5:o(ii)=dx1:o(ii+1)=dx2:o(ii+2)=dx3:o(ii+3)=dx4:o(ii+4)=dx5
    PRINT ".";
end for

data 1,2,2,2,2
data 3,4,3,3,2
data 5,3,2,3,3
' ASK IF HE WANTS DIRECTIONS
printMessage(301)
z0 = askYesNo%()
if z0 then goto L267 else goto L320
L267: printMessage(302)
'    command INPUT routine
L300: ' PRINT room, items
' If it's dark don't let him see anything
L320: if l1 < 13 or l1 = 58 then goto L350
if l = 1 and (s(18) = l1 or s(18) = -1) then goto L350
printMessage(45)
goto L400
L350: select case d0
    case 0
        shortDescription()
    case 1
        longDescription()
    case else
        describeRoomOnEntry()
end select
v(l1) = 1
describeRoomContents()
if dead = 1 then goto L9540
gosub L7800
' INPUT LOOP --- MULTIPLE COMMAANDS, REMOVE JUNK CHARACTERS
L400: if len(c$) > 0 then goto L500
L410: INPUT ">";c$:c$=ucase$(c$)
if c$ = "" then goto L410
PRINT: PRINT
c$ = ucase$(c$)
'    65-90 = A-Z               48-57 = 0-9         46 = . 44 = ,
for x = 1 to len(c$)
    z5 = asc(mid$(c$,x,1))
    if not ((z5 > 64 and z5 < 91) or (z5 > 47 and z5 < 58) or z5 = 44) then
        c$ = mid$(c$,1,x-1)+" "+mid$(c$,x+1)
    end if
end for

if mid$(c$,len(c$),1) = "," then goto L500
c$ = c$+","
L500: z4 = instr(c$,",")
a$ = ucase$(mid$(c$,1,z4-1)) : c$ = mid$(c$,z4+1)
a$ = " "+a$+" "
' search a$ for keywords,puut kwd code into k(x)
' items
L580: data 1,"GOLD",1,"NUGGET",2,"BARS",2,"SILVER",3,"JEWELRY",4,"COINS"
data 5,"DIAMONDS",6,"MING",6,"VASE",7,"PEARL",8,"EGGS",8,"NEST"
data 9,"TRIDENT",10,"EMERALD",11,"PLATINUM",11,"PYRAMID",12,"CHAIN"
data 13,"SPICES",14,"PERSIAN",14,"RUG",15,"TREASURE",15,"CHEST"
data 16,"WATER",17,"OIL",18,"LAMP",18,"LANTERN",19,"KEYS",20,"FOOD",21,"BOTTLE"
data 22,"CAGE",23,"ROD",23,"WAND",24,"CLAM",25,"MAGAZINE",26,"BEAR"
data 27,"AXE",28,"VELVET",28,"PILLOW",29,"SHARDS",30,"OYSTER"
data 31,"BIRD",32,"TROLL",33,"DRAGON",34,"SNAKE",35,"DWARF"
data 36,"ROCK",36,"BOULDER",37,"STAIRS",38,"STEPS",39,"HOUSE",39,"BUILDING"
data 40,"GRATE",41,"STREAM",42,"ROOM",43,"BRIDGE",44,"PIT",45,"VOLCANO"
data 46,"ROAD",47,"ALL",47,"EVERYTHING"
' DIRECTIONS
data 100,"N",100,"NORTH",101,"NE",101,"NORTHEAST",102,"E",102,"EAST"
data 103,"SE",103,"SOUTHEAST",104,"S",104,"SOUTH",105,"SW",105,"SOUTHWEST"
data 106,"W",106,"WEST",107,"NW",107,"NORTHWEST",108,"U",108,"UP",109,"D",109,"DOWN"
' VERBS
data 110,"PLUGH",111,"XYZZY",112,"PLOVER",113,"CROSS",114,"CLIMB",115,"JUMP"
data 116,"FILL",117,"EMPTY",117,"POUR",118,"LOOK",118,"L",119,"LIGHT",119,"ON",120,"EXTINGUISH"
data 120,"OFF",121,"IN",121,"ENTER",122,"LEAVE",122,"OUT",123,"INVENTORY",123,"I"
data 124,"GET",124,"CATCH",124,"TAKE",125,"DROP",125,"DUMP",126,"THROW",127,"ATTACK"
data 127,"KILL",128,"FEED",129,"WATER",130,"LOCK",131,"UNLOCK"
data 132,"FREE",132,"RELEASE",133,"WAVE",134,"OPEN",135,"CLOSE"
data 136,"OIL",137,"EAT",138,"DRINK",139,"FEE FIE FOE FOO"
data 140,"SHORT",141,"LONG",142,"BRIEF",143,"QUIT",143,"STOP",143,"END"
data 144,"SCORE",145,"SAVE",146,"LOAD",147,"READ",147,"EXAMINE"
data 148,"YES",148,"Y",149,"BUG",150,"*"
restore L580
for i = 1 to 200
    k(i) = 0
end for

z1 = 0 : z3 = 0
' T3=TOTAL NUMBER OF KEYWORDS
L820: if z1 > t3 then goto L900
read z1,b$
b$ = " "+b$+" "
' IF KEYWORD WAS FOUND, NOTE THIS IN K(Z1)
if instr(a$,b$) = 0 then goto L820
k(z1) = 1
' KEYWORDK #Z1 NOT FOUND
goto L820
' EXOTIC WORDS
L900: z0 = 36
x = z0
LW910: if x > 46 then goto L960
if k(x) = 0 then goto LCONT910
gosub L8390
L940: print "What do you want to do with the ";d$;"?"
goto L410
LCONT910: x = x + (1)
goto LW910
L960:
x = 110
LW970: if x > t3 then goto L990
if k(x) = 1 then goto L1950
x = x + (1)
goto LW970
L990:
' THEN IT'S A DIRECTION
d = 1
LW1010: if d > 10 then goto L1030
if k(d+99) = 1 then goto L1070
d = d + (1)
goto LW1010
L1030:
' COMMAND NOT A DIRECTION
goto L1950
' CAN HE MOVE THAT WAY?
L1070: z2 = dirs(L1,d)
if z2 = 255 then goto L1470
if z2 < 1 or z2 > 254 then goto L1220
L1150: ' NORMAL MOVING
' CHECK FOR SPECIAL MOVE CONDITIONS
goto L1260
L1180: l2 = l1 : l1 = z2
if s(35) = l2 then goto L1191 else goto L1200
L1191: s(35) = l1
L1200: goto L9780
L1220: printMessage(1)
goto L400
' SPECIAL ROOM DIRECTIONS
' GRATE
L1260: if L1 = 10 and (d = 10 or d = 5) THEN goto L1280
IF L1 = 11 and (d = 9 or d = 3) then goto L1280 else goto L1320
' IF GRATE IS OPEN (G=0) MOVE HIM
L1280: if g = 1 then goto L1180
printMessage(10)
goto L400
' CAN'T TAKE NUGGET UPPSTAIRS
L1320: if not (l1 = 17 and d = 9 and s(1) = -1) then goto L1360
printMessage(38)
goto L400
' CRYSTAL BRIDGE AND FISSURE
L1360: if not (l1 = 19 and d = 7 or l1 = 20 and d = 3) then goto L1410
if b2 then goto L1180
printMessage(3)
goto L400
' MT. KING & SNAKE
L1410: if not (l1 = 22 and d <> 3 and d <> 9) then goto L1690
if sn = 0 then goto L1180
printMessage(50)
goto L400
' bedquilt and random directiosn
L1470: if l1 <> 44 then goto L1590
if RND(1) > 0.5 then goto L1510
printMessage(52)
goto L400
L1510: restore L1530
' ROOMS TO JU
L1530: data 33,37,45,92,76
for z3 = 1 to int(RND(1)*5)+1
    read z2
end for

goto L1180
' WITT's END
L1590: if l1 <> 39 then goto L1220
' SHOULD WE LET HIM OUT?
if RND(1) < 0.15 then goto L1650
' NO
printMessage(52)
goto L400
L1650: ' YES
z2 = 38
goto L1180
' Narrow Tunnel
L1690: if not (l1 = 57 or l1 = 58) then goto L1780
IF K(102)=0 AND K(106)=0 THEN goto L1780
z3 = 1
LW1700: if z3 > t2 then goto L1750
if z3 = 10 then goto LCONT1700
if s(z3) <> -1 then goto LCONT1700
printMessage(53)
goto L400
LCONT1700: z3 = z3 + (1)
goto LW1700
L1750:
goto L1180
' TROLL
L1780: IF L1=60 AND D=2 THEN goto L1790
IF L1=61 AND D=6 THEN goto L1790 ELSE goto L1860
L1790: on t+1 goto L1180,L1800,L1820,L1840
L1800: printMessage(55)
goto L400
L1820: printMessage(56)
printMessage(55)
T=1:goto L400
L1840: t = 2
goto L1180
L1860: if not (l1 = 73 and d = 1 and d2 = 0) then goto L1890
printMessage(57)
goto L400
L1890: if not (l1 = 82 and s(33) = l1 and d = 1) then goto L1180
' DRAGON
printMessage(51)
goto L400
' OTHER COMMANDS
L1950: z1 = 100
LW1950: if z1 > t3 then goto L1970
if k(z1)=1 then goto L2090
z1 = z1 + (1)
goto LW1950
L1970:
' ITEM BO NO VERB?
' (was: restore L9961 -- item names now come from itemname$())
x = 1
LW2000: if x > 35 then goto L2030
d$ = itemname$(x)
if k(x) = 1 then goto L940
x = x + (1)
goto LW2000
L2030:
L2040: restore L2070
for x = 1 to int(RND(1)*4)+1
    read b$
end for

PRINT b$
L2070: data "What?","I don't understand.","I can't understand that.","I don't know that word."
goto L400
L2090: z1 = z1-109
on z1 goto L2120,L2220,L2300,L2390,L2570,L2640,L2680,L2860,L2930,L3000,L3080,L3130,L3290,L3450,L3590,L3920,L4160,L4430,L4630,L4800,L4950,L5060,L5190,L5410,L5560,L5750,L5810,L5920,L6000,L6090,L6230,L6270,L6310,L6350,L6410,L8970,L9080,L9220,L4490,L9330
goto L2040
L2120: ' *** PLUGH ***
IF L1<>7 THEN goto L2170
IF S(35)=L1 THEN S(35)=0
Z2=26
GOTO L1180
L2170: IF L1<>26 THEN goto L2200
Z2=7
GOTO L1180
L2200: printMessage(2)
GOTO L400
L2220: ' *** XYZZY ***
IF L1<>7 THEN goto L2270
IF S(35)=L1 THEN S(35)=0
Z2=13
GOTO L1180
L2270: IF L1<>13 THEN goto L2200
Z2=7
GOTO L1180
L2300: ' *** PLOVER *** (CAN'T BRING EMERALD WITH HIM)
IF L1>26 THEN goto L2360
IF S(35)<>L1 THEN goto L2330
S(35) = 0
L2330: IF S(10)<>-1 THEN goto L2340
S(10) = L1
L2340: Z2 = 58
GOTO L1180
L2360: IF L1<>58 THEN goto L2200
Z2=26
GOTO L1180
L2390: ' *** CROSS ***
IF L1<>19 THEN goto L2470
IF B2<>0 THEN goto L2440
L2420: printMessage(3)
GOTO L400
L2440: D=7
' JUST GIVE NEW DIRECTION, USE MOVE ROUTINE
GOTO L1070
L2470: IF L1<>20 THEN goto L2510
IF B2=0 THEN goto L2420
D=3
GOTO L1070
L2510: IF L1<>60 THEN goto L2540
D=2
GOTO L1070
L2540: IF L1<>61 THEN goto L2200
D=6
GOTO L1070
L2570: ' *** CLIMB ***
IF L1<>50 THEN goto L2200
' CAN HE CLIMB BEANSTALK?
IF P1<2 THEN goto L2200
' YES
Z2=70
GOTO L1180
L2640: ' *** JUMP *** STRICTLY SUICIDAL
IF L1<>16 AND L1<>19 AND L1<>20 AND L1<>27 THEN goto L2200
printMessage(4)
GOTO L9550
L2680: ' FILL
IF S(21)=-1 THEN goto L2730
L2700: B$="bottle"
L2710: PRINT "You don't have the ";b$
goto L410
L2730: IF B0=0 THEN goto L2760
printMessage(5)
GOTO L410
L2760: IF L1<>7 AND L1<>8 AND L1<>9 AND L1<>35 AND L1<>74 AND L1<>81 THEN goto L2790
B0=1:S(16)=-1
GOTO L2840
L2790: IF L1=49 THEN goto L2830
B$="oil"
L2810: PRINT "I see no ";B$;" here."
GOTO L400
L2830: B0=2:S(17)=-1
L2840: PRINT "The bottle is now filled."
GOTO L400
L2860: ' *** EMPTY ***
IF S(21)=-1 THEN goto L2890
GOTO L2700
L2890: ' EMPTY BOTTLE (ASSUMED FULL)
S(B0+15)=0:B0=0
PRINT "Emptied"
GOTO L400
L2930: ' *** LOOK ***
L2940: if l1 < 13 or l1 = 58 then goto L2970
if l = 1 and (s(18) = l1 or s(18) = -1) then goto L2970
printMessage(45)
goto L400
L2970: longDescription() ' (was: gosub 8050 -- jumped past the old
' subroutine's own `v(l1)=1` to avoid redundantly re-marking the room
' visited; by the time LOOK is typeable the room's already been entered
' via describeRoomOnEntry(), which already sets v(l1)=1, so calling the
' full longDescription() here just re-does that no-op assignment)
describeRoomContents()
goto L400
L3000: ' *** LIGHT ***
if s(18) = -1 then goto L3040
L3020: b$ = "lamp"
goto L2710
L3040: l = 1
b$ = "on"
L3060: PRINT "The lamp is now ";b$
goto L2940
L3080: ' *** OFF (EXTINGUSIH) ***
IF S(18)=-1 THEN goto L3110
GOTO L3020
L3110: L=0:B$="off"
GOTO L3060
L3130: ' *** ENTER ***
IF L1<>6 THEN goto L3180
' TO HOUSE
D=3
GOTO L1070
L3180: IF L1<>68 THEN goto L3240
' TO BARREN ROOM
D=3
GOTO L1070
L3240: D = 10
LW3240: if D < 1 then goto L3270
Z2 = DIRS(L1,D)
IF Z2>0 AND Z2<101 THEN goto L1150
D = D + (-1)
goto LW3240
L3270:
GOTO L2200
L3290: ' ** LEAVE ***
IF L1<>7 THEN goto L3340
' LEAVE HOUSE
D=7
GOTO L1070
L3340: IF L1<>69 THEN goto L3400
' LEAVE BARREN ROOM
D = 7
GOTO L1070
L3400: D = 1
LW3400: if D > 10 then goto L3430
Z2 = DIRS(L1,D)
IF Z2>0 AND Z2<101 THEN goto L1150
D = D + (1)
goto LW3400
L3430:
GOTO L2200
L3450: ' *** INVENTORY ***
Z0=0
PRINT "You are carrying:";
FOR X=1 TO T2
    IF S(X)=-1 THEN
        b$ = itemname$(x)
        PRINT B$
        Z0 = Z0 + 1
    end if
end for

IF Z0=0 THEN goto L3551 else goto L3560
L3551: PRINT "nothing."
L3560: PRINT
GOTO L400
L3590: ' *** GET ***
if k(47) = 1 then goto L3680
gosub L8280
if z8 > 0 then goto L3680
PRINT "Get what?"
goto L2040
L3680: z3 = 1
LW3680: if z3 > t2 then goto L3900
if k(47) = 1 then goto L3730
if k(z3) = 0 then goto LCONT3680
L3730: if s(z3) <> l1 then goto L3750
if s(z3) = l1 then goto L3790
L3750: if k(47) = 1 then goto LCONT3680
a$ = itemname$(z3):PRINT a$;" not here."
goto LCONT3680
' MUST CHECK NOW FOR LEGALITY OF TAKING ITEM
L3790: z8 = 0
for x = 1 to t2
    if s(x) = -1 then
        z8 = z8+1
    end if
end for

if z8 < 7 then goto L3870
' CARRYING TOO MUCH
printMessage(54)
goto L410
L3870: goto L6880
L3880: s(z3) = -1
L3890: a$ = itemname$(z3):PRINT a$;":taken."
LCONT3680: z3 = z3 + (1)
goto LW3680
L3900:
goto L400
L3920: ' *** DROP ***
if k(47) = 1 then goto L4000
gosub L8280
IF Z8>0 THEN goto L4000
PRINT "Drop what?"
GOTO L2040
L4000: Z3 = 1
LW4000: if Z3 > T2 then goto L4140
IF K(47)=1 THEN goto L4060
IF K(Z3)<>1 THEN goto LCONT4000
IF S(Z3)=0 THEN goto LCONT4000
L4060: IF S(Z3)=-1 THEN goto L4100
IF K(47)=1 THEN goto LCONT4000
b$ = itemname$(z3):PRINT "You don't have the ";B$
GOTO LCONT4000
L4100: ' STILL NEED TO ELABORATE ON DROP (BIRD IN CAGE, BOTTLE)
GOTO L7380
L4120: b$ = itemname$(z3):PRINT B$;":dropped."
S(Z3)=L1
LCONT4000: Z3 = Z3 + (1)
goto LW4000
L4140:
GOTO L400
L4160: ' *** THROW ***
GOSUB L8280
IF Z8>0 THEN goto L4210
PRINT "Throw what?"
GOTO L2040
L4210: IF S(Z3)<>-1 THEN goto L2710
IF NOT (Z3<16 AND S(32)=L1) THEN goto L4260
' THROW TREASURE TO TROLL
printMessage(27)
S(Z3)=0:T=3:GOTO L400
L4260: IF NOT (Z3=27 AND S(32)=L1) THEN goto L4300
' TRYING TO BUTCHER TROLL?
printMessage(26)
S(27)=L1:GOTO L400
L4300: IF NOT (Z3=27 AND S(35)=L1) THEN goto L4380
' TRYING TO KILL DWARF
IF RND(1)>0.5 THEN goto L4360
printMessage(29)
checkDwarfAttack() ' (was: GOSUB 8650 -- jumped straight into the
' attack-check half of the original dwarf subroutine, deliberately
' skipping the axe-giving check; see checkDwarfAttack()'s own comment)
GOTO L4410
L4360: printMessage(30)
S(35)=0:GOTO L4410
L4380: ' NOTHING SPECIAL, JUST DROP ITEM
IF S(35)<>L1 THEN goto L4400
checkDwarf() ' (was: GOSUB L8550 -- L8550 was just a comment
' immediately before the real dwarf subroutine's first line, L8560, so
' this call wanted the full checkDwarf() behavior, axe-check included --
' a call site missed when checkDwarf()/checkDwarfAttack() were split out)
L4400: PRINT "Thrown."
L4410: S(Z3) = L1
if dead = 1 then goto L9540
GOTO L400
L4430: ' *** ATTACK ***
GOSUB L8280
IF NOT (Z3=33 AND S(Z3)=L1 AND L1=82) THEN goto L4520
' HE CAN KILL DRAGON
printMessage(68)
GOTO L410
L4490: IF L1<>82 THEN goto L2040
printMessage(69)
S(33)=0:D1=0:GOTO L400
L4520: IF S(32)<>L1 THEN goto L4560
' TRYING TO MUNGE TROLL
' (was: Z9=FNA(25):GOTO 400 -- FNA is called with no matching DEF FN
' anywhere in the original source; this looks like leftover/broken code
' from an earlier version of the upstream port, not something this port
' introduced. Z9's assignment here isn't read before it's next assigned
' elsewhere, so dropping the call changes nothing observable.)
GOTO L400
L4560: IF NOT (Z3=26 OR Z3>30) THEN goto L4600
' DANGEROUS TO ATTACK THESE
printMessage(70)
GOTO L400
L4600: ' NOTHING TO ATTACK
printMessage(71)
GOTO L400
L4630: ' *** FEED ***
GOSUB L8280
IF Z3<>35 THEN goto L4690
' CAN'T FEED DWARF!
printMessage(24)
GOTO L400
L4690: IF S(20) = -1 THEN goto L4720
B$ = "FOOD":GOTO L2710
L4720: IF L1=69 THEN goto L4760
PRINT "I can't feed it."
printMessage(23)
GOTO L400
L4760: IF S(20)=L1 THEN goto L7600
B1=1:S(20)=0:printMessage(6)
GOTO L400
L4800: ' *** WATER ***
IF S(16) = -1 THEN goto L4840
B$ = "water":GOTO L2710
L4840: IF L1<>50 THEN goto L2200
' GOTO P1+1 OF 4860,4890,4920
IF P1 = 0 THEN goto L4860
IF P1 = 1 THEN goto L4890
IF P1 = 2 THEN goto L4920
L4860: printMessage(7)
P1=1:S(16)=0:B0=0:GOTO L400
L4890: printMessage(8)
P1=2:S(16)=0:B0=0:GOTO L400
L4920: printMessage(9)
P1=0:S(16)=0:B0=0:GOTO L400
L4950: ' *** LOCK ***
L4960: IF L1=10 OR L1=11 THEN goto L4990
' NOTHING LOCKABLE
GOTO L2200
L4990: IF S(19)=-1 THEN goto L5020
L5000: B$="keys":goto L2710
L5020: G=0:printMessage(10)
GOTO L400
L5060: ' *** UNLOCK ***
L5070: IF S(19)<>-1 THEN goto L5000
IF L1<>10 AND L1<>11 THEN goto L5120
G=1:printMessage(11)
GOTO L400
L5120: IF L1<>69 THEN goto L2200
IF B1>0 THEN goto L5160
printMessage(12)
goto L400
L5160: IF C<>0 THEN goto L5170
C=1:B1=2
L5170: printMessage(13)
GOTO L400
L5190: ' *** FREE ***
IF K(31) = 1 THEN goto L5240
' CAN'T FREE ANYTHING BUT BIRD
L5220: printMessage(2)
GOTO L410
L5240: IF S(31)<>-1 THEN goto L5220
S(31) = L1:B3=0
PRINT "Freed."
IF L1<>22 THEN goto L5350
IF SN<>1 THEN goto L400
B$="snake"
L5300: PRINT "The little bird attacks the green ";B$;" and"
IF L1=82 THEN goto L5380
PRINT "drives it off"
SN=0:S(34)=0:GOTO L400
L5350: IF L1<>82 THEN goto L400
B$="dragon":GOTO L5300
L5380: PRINT "gets burned to a crisp"
S(31)=0
GOTO L400
L5410: ' *** WAVE ***
IF K(23) <> 1 THEN goto L2200
IF S(23)=-1 THEN goto L5460
B$="rod":GOTO L2710
L5460: '  IS HERE NEAR FISSURE
IF L1<>19 AND L1<>20 THEN goto L2200
' yes
' GOTO B2+1 OF 5500,5530
IF B2=0 THEN goto L5500
IF B2=1 THEN goto L5530
L5500: printMessage(14)
B2=1:GOTO L400
L5530: printMessage(15)
B2=0:GOTO L400
L5560: ' *** OPEN ***
GOSUB L8280
IF Z3>0 THEN goto L5610
PRINT "Open ";:goto L2040
L5610: IF Z3=40 THEN goto L5070
IF S(Z3)=L1 THEN goto L5650
L5630: PRINT "I see no ";b$;" here.":goto L400
L5650: if z3=24 THEN goto L5680
PRINT "I don't know how to open a ";B$:GOTO L400
L5680: IF S(9)=-1 THEN goto L5710
printMessage(16)
GOTO L400
L5710: IF S(Z3) = 0 THEN goto L2200
' HE'S OPENED CLAM, SO PRINT DESCRIPTION OF THIS
' PUT PEARL IN CUL-DE-SAC
S(7)=43:S(24)=0:S(30)=L1:printMessage(17)
L5750: GOTO L400
' *** CLOSE ***
GOSUB L8280
IF Z3=40 THEN goto L4960
printMessage(18)
GOTO L400
L5810: ' OIL
IF K(17)=0 THEN goto L2200
IF S(17)=-1 THEN goto L5860
B$="oil":GOTO L5630
L5860: IF L1<>73 THEN goto L2200
' IS DOOR STILL RUSTED
IF D2=1 THEN goto L2200
D2=1:S(17)=0:B0=0:printMessage(19)
GOTO L400
' *** EAT ***
L5920: IF K(20) = 1 THEN goto L5950
printMessage(20)
GOTO L410
L5950: Z3=20:GOSUB L8490
IF Z5=0 THEN goto L410
printMessage(73)
S(20)=0:B0=0:GOTO L400
L6000: ' *** DRINK ***
IF K(16) =1 THEN goto L6040
printMessage(21)
GOTO L410
L6040: Z3=16:GOSUB L8490
IF Z5=0 THEN goto L410
printMessage(22)
S(17)=0:B0=0:GOTO L400
L6090: ' *** FEE FIE FOE FOO ***
IF L1=71 THEN goto L6130
printMessage(2)
GOTO L410
L6130: IF S(8)<>L1 THEN goto L6180
' MAKE NEST VANISH
printMessage(79)
S(8)=0:GOTO L400
L6180: ' IF S(8)=0 THEN goto L6110
S(8)=L1
' MAKE NEST RE-APPEAR
printMessage(81)
GOTO L400
L6230: ' *** SHORT ***
PRINT "Short descriptions"
D0=0:GOTO L400
L6270: ' *** LONG ***
PRINT "Long descriptions"
D0=1:GOTO L400
L6310: ' *** BRIEF ***
PRINT "OK, I'll only describe the room in detail the first time."
D0=2:GOTO L400
L6350: ' *** QUIT ***
PRINT "Save game";
Z0 = askYesNo%()
IF Z0=1 THEN goto L8970
GOTO L9750
' SCORE ***
L6410: GOSUB L6430
GOTO L400
L6430: ' PRINT OUT SCORE DATA
GOSUB L6510
PRINT "Your score is now ";S0
PRINT "You have explored ";(Z9/T1)*T1;"% of the cave."
RESTORE L6470
L6470: DATA "beginner","novice","experienced","advanced","expert"
Z9 = INT((S0-1)/100)
IF Z9 <= 4 THEN goto L6483
Z9=4
L6483: FOR Z0=0 TO Z9
    READ D$
end for

PRINT "That makes you a ";D$;" adventurer."
RETURN
L6510: ' COMPUTE CURRENT SCORE
RESTORE L230
Z9=0:S0=0
FOR Z0=1 TO 15
    READ Z1
    IF Z1<>0 THEN
        IF V(Z1)=1 THEN
            S0=S0+4*O(Z0)
        end if
        IF S(Z0)=7 THEN
            S0=S0+4*O(Z0)
        end if
    end if
end for

S0=(G=1)*10 + S0:S0=(SN=0)*20 + S0:S0=(D1=0)*30 + S0:S0=(T=0)*30 + S0:S0=(B1=2)*20 + S0
S0=(B2=1)*20 + S0:S0=(P1=2)*20 + S0:S0=(D2=1)*20 + S0:S0=(C=1)*20 + S0
FOR Z0 = 1 TO T1
    IF V(Z0)=1 THEN
        S0=S0+1:Z9=Z9+1
    end if
end for

RETURN
' list items at location l1
' room-contents listing + dwarf/pirate checks are now the
' describeRoomContents() procedure above.
' Print Short room description
' short room description is now the shortDescription() procedure above.
L6880: ' SPECIAL GETS
if not (z3 = 24 or z3 = 30 or z3 > 31) then goto L6930
' CAN'T GET THESE FOR SOME REASON
printMessage(61)
goto L400
L6930: if not (z3 = 12 and c = 0) then goto L6970
' CHAIN
printMessage(58)
goto L3900
L6970: ' BEAR IS HE FED? UNLOCKED?
if not (z3 = 26 and b1 <> 2) then goto L7010
printMessage(61)
goto L3900
L7010: if not (z3 = 14 and d1 = 1) then goto L7050
' DRAGON AND RUG
printMessage(59)
goto L3900
L7050: if not (z3 = 16 or z3 = 17) then goto L7090
' OIL AND WATER DO SAME AS FILL
PRINT "Why not say 'fill'?"
goto L3900
L7090: if not (z3 = 22 and b3) then goto L7140
' TAKE BIRD SINCE IT'S IN CAGE
s(31) = -1:PRINT "Bird and ";:goto L3880
L7140: if z3 <> 31 then goto L7310
' GETTING BIRD
if b3 <> 1 then goto L7210
' TAKE CAGE, SINCE BIRD IS IN IT
PRINT "Cage and ";:s(22) = -1:goto L3880
L7210: if s(22) = -1 then goto L7240
b$ = "cage":goto L2810
L7240: if s(23) = -1 then goto L7280
' OK TO TAKE BIRD
s(31) = -1 : b3 = 1:goto L3890
L7280: ' ROD SCARES BIRD
printMessage(37)
goto L3900
L7310: ' BOTTLE FULL? IF SO, GET CONTENTS
if not (z3 = 21 and b0) then goto L7360
PRINT "Contents and the ";
s(b0+15) = -1
L7360: goto L3880
' SPECIAL "DROP"
L7380: IF Z3<>31 THEN goto L7440
' BIRD IN CAGE
L7400: S(31)=L1:S(22)=L1:B3=1
IF Z3<>31 THEN goto L7420
PRINT "Cage and ";
L7420: IF Z3=22 THEN goto L7430
PRINT "Bird and ";
L7430: goto L4120
L7440: if z3=22 and b3=1 then goto L7400
IF Z3<>21 THEN goto L7520
' BOTTLE
IF B0=0 THEN goto L4120
' BOTTLE IS FULL, DO DROP CONTENTS TOO
PRINT "Contents and ";
S(15+B0)=L1:GOTO L4120
L7520: IF NOT (Z3=16 OR Z3=17) THEN goto L7541
PRINT "Try saying 'empty'":goto L4140
L7541: IF Z3<>26 OR T<>1 OR (L1<>60 AND L1<>61) THEN goto L7550
printMessage(28)
T=0:S(26)=L1:S(32)=0:GOTO L400
L7550: IF Z3<>6 THEN goto L4120
IF S(28)=L1 THEN goto L7600
' GOODBYE, FRAGILE VASE!
printMessage(43)
S(6)=0:S(29)=L1:GOTO L400
L7600: printMessage(60)
GOTO L4120
' PRINT MESSAGE is now the printMessage() procedure above -- every
' `z59 = N : gosub 7620` call site became `printMessage(N)`.
L7800: '
' SITUATION DESCRIPTIONS
'
' GRATE
if l1 = 10 or l1 = 11 then goto L7842 else goto L7860
L7842: printMessage((g+10))
' CRYSTAL BRIDGE
L7860: if (l1 = 19 or l1 = 20) and b2 = 1 then goto L7861 else goto L7880
L7861: printMessage(14)
' PLUGH NOISE
L7880: if l1 = 26 and RND(1) > 0.3 then goto L7881 else goto L7900
L7881: printMessage(41)
' IRON DOOR
L7900: if l1 = 73 and d2 = 0 then goto L7901 else goto L7920
L7901: printMessage(57)
' TROLL
L7920: if (l1 = 60 or l1 = 61) and t = 1 then goto L7921 else goto L7940
L7921: printMessage(63)
' BEAR
L7940: if l1 = 69 and b1 = 0 then goto L7941 else goto L7950
L7941: printMessage(64)
L7950: if l1 = 69 and b1 = 1 then goto L7951 else goto L7970
L7951: printMessage(66)
' PLANT IN PIT
L7970: if l1 = 48 or l1 = 50 then goto L7971 else goto L7980
L7971: printMessage(47+p1)
L7980: return
' long room description is now the longDescription() procedure above.
' the short-vs-long-on-entry decision is now the describeRoomOnEntry()
' procedure above.
'
'   fetch first item code in k(1 to t2)
'   z8=total # of items found in list
'   z3=item code first found
L8280: z8 = 0 : z3 = 0: d$ = ""
for z5 = 1 to 45
    if k(z5) <> 0 then
        z8 = z8+1
        b$ = itemname$(z5):d$ = b$
        if k(z5)=1 and z8 = 1 then
            z3 = z5
        end if
    end if
end for

b$ = d$
return
L8390: ' FIND FIRST ITEM AT ROOM
x1 = 0
FOR Z1=1 TO 47
    if x1<>1 then
        IF K(Z1) = 1 THEN
            d$ = itemname$(z1):x1=1
        end if
    end if
end for

RETURN
'
' MAKE SURE HE'S CARRYING ITEM * Z3
'
L8490: IF S(Z3)=-1 THEN goto L8530
PRINT "You don't have the ";A$
Z5=0
RETURN
L8530: Z5=1
RETURN
' dwarf/pirate checks are now checkDwarf()/checkDwarfAttack()/
' checkPirate() above.
L8970: ' *** SAVE GAME ***
INPUT "What do you want to call the save file? ";A$
' `on error goto` (not `try`/`catch`) here on purpose -- see the
' README's own note on why, and GitHub issues #61 (on error goto
' is permanently unsupported under --target c, by design) & #100
' (try/catch's RESUME output isn't accepted by real fbc).
on error goto L9010
OPEN A$ FOR OUTPUT AS #5
on error goto 0
GOTO L9030
L9010: PRINT "File ";a$;" not created"
GOTO L410
L9030: PRINT #5,T1;",";T2;",";T3;",";L1;",";L2;",";G;",";B0;",";SN;",";D1;",";D2;",";D0;",";T;",";B1;",";B2;",";P1;",";L;",";C;",";D3;",";B3;",";R0;",";KC
FOR X=1 TO 99
    PRINT #5,S(X);",";V(X)
end for

PRINT #5,V(100)
CLOSE #5
PRINT "Game saved"
C0 = 0
if k(143) = 1 then goto L9750
GOTO L410
L9080: ' *** LOAD OLD GAME ***
IF C0=0 THEN goto L9120
PRINT "You already have a loaded game!"
GOTO L410
L9120: INPUT "Save file name? ";A$
on error goto L9150
OPEN A$ FOR INPUT AS #5
on error goto 0
GOTO L9170
L9150: PRINT "Unable to use file ";A$
GOTO L410
L9170: INPUT #5,T1,T2,T3,L1,L2,G,B0,SN,D1,D2,D0,T,B1,B2,P1,L,C,D3,B3,R0,KCX$
kc = val(kcx$)
FOR X=1 TO 99
    INPUT #5,SX,VX:s(x)=sx:v(x)=vx
end for

INPUT #5,vx:v(100)=vx
CLOSE #5
C0=1
GOTO L300
L9220: ' *** READ THE MAGAZINE ***
GOSUB L8280
IF Z3=25 THEN goto L9270
printMessage(74)
GOTO L410
L9270: IF S(25)=-1 THEN goto L9300
B$="magazine"
GOTO L2710
L9300: ' OK, LET HIM READ IT
printMessage(303)
GOTO L400
L9330: ' *** BUG ***
A$ = "ADVBUGS.TXT"
on error goto L9150
OPEN A$ FOR APPEND AS #5
on error goto 0
INPUT "Your name: ";A$
A$=A$+" "+DATE$
PRINT #5,A$
PRINT "Enter your gripe in up to five lines (hit return to quit):"
Z0 = 1
LW9430: if Z0 > 5 then goto L9480
PRINT Z0;
INPUT A$
IF A$="" THEN goto L9490
PRINT #5,A$
Z0 = Z0 + (1)
goto LW9430
L9480:
L9490: PRINT "Message recorded. Thank you!"
CLOSE #5
GOTO L410
L9540: ' REINCARNATE HIM
L9550: R0=R0+1
IF R0=1 THEN goto L9580
IF R0=2 THEN goto L9610
IF R0=3 THEN goto L9740
' ASK HIM IF HE WANTS TO BE REINCARNATED
L9580: printMessage(75)
Z0 = askYesNo%()
GOTO L9630
L9610: printMessage(77)
GOTO L9580
L9630: IF Z0=0 THEN goto L9750
printMessage(76)
' PUT HIM BACK IN HOUSE, REARRANGE HIS STUFF
S(18)=7:L=0:DEAD=0:KC=1.03
FOR X=1 TO T2
    IF S(X)=-1 THEN
        S(X)=L1
    end if
end for

' WE'VE PUT THE LAMP IN HOUSE AND OTHER ITEMS WHERE HE DIED
L1=INT(RND(1)*4)+1:L2=L1
GOTO L320
' THIRD DEATH--END OF GAME
L9740: printMessage(78)
L9750: PRINT "Oh well..."
GOSUB L6430
' (ADESCRIP/AITEMS/AMESSAGE are in-memory arrays now -- nothing to close)
STOP
L9780: ' *** PITS ***
if l1 < 13 THEN goto L300
IF l = 1 and (s(18) = -1 or s(18) = l1) then goto L300
' IS HE GOING TO FALL INTO A PIT?
if l1 = 16 or l1 = 17 or l1 = 19 or l1 = 20 or l1 = 25 or l1 = 47 or l1 = 48 or l1 = 59 or l1 = 60 or l1 = 61 or l1 = 75 or l1 = 76 or l1 = 98 then goto L9840
goto L300
' he fell into a pit
L9840: printMessage(44)
goto L9540
' *** SEEK A "YES" OR "NO" is now the askYesNo%() function above.
' ---- SHORT NAMES FOR STUFF ----
L9961: data "large gold nugget"
data "bars of silver"
data "precious jewelry"
data "many coins"
data "several diamonds"
data "fragile ming vase"
data "glistening pearl"
data "nest of golden eggs"
data "jewel-encrusted trident"
data "egg-sized emerald"
data "platinum pyramid"
data "golden chain"
data "rare spices"
data "persian rug"
data "treasure chest"
data "water"
data "oil"
data "brass lamp"
data "keys"
data "food"
data "bottle"
data "wicker cage"
data "3-foot black rod"
data "clam"
data "magazine"
data "bear"
data "axe"
data "velvet pillow"
data "shards of pottery"
data "oyster"
data "bird"
data "troll"
data "dragon"
data "snake"
data "dwarf"
data "rock"
data "stairs"
data "steps"
data "house"
data "grate"
data "stream"
data "room"
data "bridge"
data "pit"
data "volcano"
data "road"
data "everything"
' INITIALIZE MESSAGE INDEX is now the buildMessageIndex() procedure above.

```

</details>

<details class="source-embed" markdown="1">

<summary><code>stage4-refactored-bascal/adventure.bcl</code> -- Stage 4 — the 40-way dispatch becomes SELECT CASE</summary>

```bascal

// ADVENTURE/3000 -- Stage 4: the 40-way verb dispatch is SELECT CASE.
// See ../README.md for the port's provenance, staging, and exactly how
// far this refactor has gotten so far (it is NOT a complete rewrite).
//
// Picks up where stage 3 left off, on the command loop stage 3 stopped
// short of: the main dispatch's `ON z1 GOTO <40 labels>` is now
// `select case z1`, each of the 40 verb handlers kept as its own case
// block -- their own internal GOTO-heavy logic untouched, since a GOTO
// can jump out of a SELECT CASE branch to anywhere, same as it always
// could out of a flat IF. All standalone GOSUB subroutines have also
// become procedures/functions. What's still exactly stage 2's raw
// label/GOTO structure: the outer game loop every verb handler jumps
// back into, and the command-parsing/movement engine that runs before
// dispatch -- both flagged as `stage 5`'s job, not attempted here.
program adventure3000

' The original used PyBASIC's UPPER$/LOWER$, which real BASIC (and BASCAL's
' own basic-target output) doesn't have; BASCAL's stdlib UCASE$/LCASE$ are
' the direct equivalent, so every UPPER$/LOWER$ call below was rewritten to
' use them instead. See ../README.md.
require com.bascal.stdlib.ucase
require com.bascal.stdlib.lcase

' Global arrays the new procedures below operate on via `global` --
' moved up from where stage 2 first `dim`s them inline, so every
' procedure that needs one can declare it `global` regardless of where
' in the file it's defined.
dim descrip$(200)
dim items$(200)
dim msg$(2500)
dim indx(303)
dim fraindx(10)
dim itemname$(47)

' Loads ADESCRIP (room descriptions) into descrip$(), indexed 1..dcount
' by file line order. Replaces stage 2's inline LA100-LA102 loading code.
procedure loadAdescrip()
    global descrip$
    dim dcount, line$
    open "ADESCRIP" for INPUT as #1
    while not(EOF(1))
        dcount = dcount + 1
        ' INPUT # into an array element directly isn't supported by the
        ' minimal C backend yet -- only a bare scalar variable is -- so
        ' read into one first, then assign it into the array.
        input #1, line$
        descrip$(dcount) = line$
        print ".";
    end while
    close #1
end procedure

' Loads AITEMS (item-at-location messages) into items$(). Replaces
' stage 2's inline LA110-LA112 loading code.
procedure loadAitems()
    global items$
    dim icount, line$
    open "AITEMS" for INPUT as #2
    while not(EOF(2))
        icount = icount + 1
        input #2, line$
        items$(icount) = line$
        print ".";
    end while
    close #2
end procedure

' Loads AMESSAGE (the numbered "#N" message table) into msg$(1..mcount).
' Replaces stage 2's inline LA120-LA122 loading code.
procedure loadAmessage()
    global msg$
    global mcount
    dim line$
    mcount = 0
    open "AMESSAGE" for INPUT as #3
    while not(EOF(3))
        mcount = mcount + 1
        input #3, line$
        msg$(mcount) = line$
    end while
    close #3
end procedure

' Scans the in-memory msg$() built by loadAmessage() and records where
' each numbered message starts, so printMessage's `gosub L7620` can jump
' straight there instead of scanning from the top every time. Replaces
' the original AMESSAGE.IDX disk cache and this port's own stage-2
' in-memory equivalent (the old `gosub L12500` target).
procedure buildMessageIndex()
    global msg$
    global mcount
    global indx
    global fraindx
    dim fpos, fracnt, lastfra, b$, z4
    fpos = 0 : fracnt = 0 : lastfra = -1
    while fpos <= mcount
        fpos = fpos + 1
        if fpos > mcount then
            return
        end if
        b$ = msg$(fpos)
        if b$ = "#" then
            return
        end if
        if instr(b$, "#") <> 0 then
            z4 = val(mid$(b$, 2))
            if int(z4) = z4 then
                indx(int(z4)) = fpos
            else
                fracnt = fracnt + 1
                if lastfra <> int(z4) then
                    indx(int(z4)) = fracnt
                    lastfra = int(z4)
                end if
                fraindx(fracnt) = fpos
            end if
        end if
    end while
end procedure

' Loads the short item/object names (DATA 9961 onward) into itemname$(),
' addressed by object code 1-47 instead of the original's computed
' `restore 9960+x` (BASCAL restore targets are fixed labels, not
' expressions).
procedure loadItemNames()
    global itemname$
    dim ianame
    restore L9961
    for ianame = 1 to 47
        read itemname$(ianame)
    end for
end procedure

' Reads a line and returns 1 for "yes", 0 for "no" -- reprompting until
' it gets one. Replaces the original's `gosub 9860` + global z0.
function askYesNo%()
    dim a$
    while true
        input a$
        if len(a$) = 0 then
            a$ = " "
        end if
        a$ = lcase$(mid$(a$, 1, 1))
        if a$ = "y" then
            return 1
        elseif a$ = "n" then
            return 0
        end if
        print "Yes or No-";
    end while
    return 0 ' unreachable -- every path through the loop above already
    ' returns; the minimal C backend requires a function's literal last
    ' top-level statement to be `return`, though, since it doesn't try
    ' to prove a loop always returns the way the basic target's own
    ' fallthrough-is-fine model doesn't need to
end function

' Prints the numbered message from AMESSAGE (msg$()/indx()/fraindx(),
' built by buildMessageIndex()). Replaces the original `z59 = N : gosub
' 7620` convention -- every call site is now `printMessage(N)`, passing
' the message number as a real argument instead of setting the global
' z59 first.
procedure printMessage(msgNum%)
    global indx
    global fraindx
    global msg$
    dim xtmp, mpos, b1$
    xtmp = indx(msgNum%)
    if msgNum% = 2 or msgNum% = 61 then
        ' Messages 2 and 61 have several interchangeable variants
        ' (see AMESSAGE's "#2.1".."#2.5" entries); pick one at random.
        xtmp = fraindx(xtmp + int(RND(1) * 5))
    end if
    mpos = xtmp
    b1$ = msg$(mpos)
    if mid$(b1$, 1, 1) <> "#" or int(val(mid$(b1$, 2))) <> msgNum% then
        print "NO DESC. # "; msgNum%; " IN FILE AMESSAGE"
        return
    end if
    while true
        mpos = mpos + 1
        b1$ = msg$(mpos)
        if mid$(b1$, 1, 1) = "#" then
            return
        end if
        print b1$
    end while
end procedure

' Prints the room's short (one-line) description and marks it visited.
' Replaces stage 2's L6780 GOSUB target.
procedure shortDescription()
    global l1
    global v
    global descrip$
    dim a$
    a$ = descrip$(l1)
    v(l1) = 1
    print a$
end procedure

' Prints the room's long (multi-paragraph, AMESSAGE-driven) description
' and marks it visited. Message number is normally l1+200, except a
' shared message for the forest (any l1 <= 4) and one for the maze-like
' rooms near the end. Replaces stage 2's L7990 GOSUB target.
procedure longDescription()
    global l1
    global v
    v(l1) = 1
    if l1 <= 4 then
        printMessage(200)
    elseif (l1 > 88 and l1 < 98) or l1 = 99 then
        printMessage(288)
    else
        printMessage(200 + l1)
    end if
end procedure

' Lists the items sitting in the current room, checks whether the golden
' bird's message should play, and then checks the dwarf and pirate
' encounters. Replaces stage 2's L6680 GOSUB target.
procedure describeRoomContents()
    global l1
    global t2
    global s
    global dead
    global items$
    for z1 = 1 to t2
        if s(z1) = l1 then
            print items$(z1)
        end if
    end for
    if s(26) = -1 then
        printMessage(67)
    end if
    ' CHECK FOR DWARF, PIRATE
    checkDwarf()
    if dead <> 1 then
        checkPirate()
        print
    end if
end procedure

' Checks whether the dwarf gives away his axe (first time in a deep
' room), attacks with his knife, or appears for the first time.
' Structurally identical to stage 2's L8560 GOSUB target -- same
' internal labels, `goto L8790` (the old shared exit) just became
' `return`, and the final label+return collapsed into the implicit
' return at the end of the procedure body.
procedure checkDwarf()
    global d3
    global l1
    global s
    if d3 <> 0 then
        checkDwarfAttack()
        return
    end if
    ' SHOULD DWARF GIVE AWAY AXE?
    if l1 < 13 then return
    if RND(1) > 0.05 then return
    ' GIVE AWAY AXE
    printMessage(80)
    s(27) = l1 : d3 = 1
end procedure

' Just the "should the dwarf attack" half of checkDwarf() -- one call
' site (stage 2's L4340, `GOSUB 8650`) jumps directly into the middle of
' the original subroutine to run only this part, deliberately skipping
' the axe-giving check above. Structurally identical to stage 2's
' L8640-L8790 span.
procedure checkDwarfAttack()
    global l1
    global s
    global t
    global dead
    global KC
    if L1 >= 13 then goto L8660
    s(35) = 0 : return
    L8660: if s(35) <> L1 then goto L8770
    if (l1 <> 60 and l1 <> 61) or t <> 1 then goto L8670
    printMessage(299)
    s(35) = 0 : return
    L8670: if RND(1) > 0.5 then return
    ' YES!
    printMessage(32)
    ' DOES THE KNIFE KILL THE PLAYER?
    KC = KC - 0.02
    IF KC >= 0.75 THEN goto L8710
    KC = 0.75
    L8710: if RND(1) <= KC then goto L8750
    ' YES
    PRINT "It gets you!"
    dead = 1
    return
    L8750: PRINT "It misses!"
    return
    L8770: ' SHOULD WE PUT A DWARF HERE?
    if RND(1) >= 0.05 then return
    if (l1 = 60 or l1 = 61) and t = 1 then return
    s(35) = l1
    printMessage(31)
end procedure

' Checks whether the pirate steals the player's valuables (only in
' rooms l1 >= 13). Structurally identical to stage 2's L8800 GOSUB
' target, with `goto L8960` (the old shared exit) replaced by `return`.
procedure checkPirate()
    global l1
    global s
    dim z3, x
    ' FIRST, DOES HE HAVE ANYTHING WORTH STEALING?
    z3 = 0
    if l1 < 13 then return
    for x = 1 to 15
        if s(x) = -1 then
            z3 = z3 + 1
        end if
    end for
    if z3 < int(RND(1) * 4) + 1 then return
    ' SHOULD WE RIP OFF HIS VALUABLES?
    if RND(1) < 0.05 then
        printMessage(33)
        for x = 1 to 15
            if s(x) = -1 then
                s(x) = 100
            end if
        end for
    else
        printMessage(34)
    end if
end procedure

' On entering a room: the forest and maze-like rooms always get their
' long description (it's short enough not to be worth abbreviating);
' every other room gets the long description only the first time it's
' visited, and the short one on repeat visits. Replaces stage 2's L8180
' GOSUB target, itself called from the `on d0+1 gosub` dispatch below.
procedure describeRoomOnEntry()
    global l1
    global v
    if l1 < 5 or (l1 > 88 and l1 < 98) or l1 = 99 then
        longDescription()
    elseif v(l1) = 1 then
        shortDescription()
    else
        longDescription()
    end if
end procedure

' Checks a handful of situational, room-dependent one-off messages (the
' grate, crystal bridge, plugh noise, iron door, troll, bear, and plant
' in the pit) after a room's normal description prints. Each check is
' independent, not a mutually-exclusive chain, matching the original's
' own if/then/else-fallthrough-to-the-next-check structure exactly.
' Replaces stage 2's L7800 GOSUB target.
procedure situationDescriptions()
    global l1
    global g
    global b2
    global d2
    global t
    global b1
    global p1
    ' GRATE
    if l1 = 10 or l1 = 11 then
        printMessage(g + 10)
    end if
    ' CRYSTAL BRIDGE
    if (l1 = 19 or l1 = 20) and b2 = 1 then
        printMessage(14)
    end if
    ' PLUGH NOISE
    if l1 = 26 and RND(1) > 0.3 then
        printMessage(41)
    end if
    ' IRON DOOR
    if l1 = 73 and d2 = 0 then
        printMessage(57)
    end if
    ' TROLL
    if (l1 = 60 or l1 = 61) and t = 1 then
        printMessage(63)
    end if
    ' BEAR
    if l1 = 69 and b1 = 0 then
        printMessage(64)
    end if
    if l1 = 69 and b1 = 1 then
        printMessage(66)
    end if
    ' PLANT IN PIT
    if l1 = 48 or l1 = 50 then
        printMessage(47 + p1)
    end if
end procedure

' Scans the parsed keyword codes k(1..45) for item names the player
' typed: z8 counts how many matched, z3 remembers the first exact-item
' match's own code, and d$/b$ end up holding the last match's display
' name (itemname$'s own text). Replaces stage 2's L8280 GOSUB target --
' by far the most-called of the remaining GOSUB subroutines (8 call
' sites), used by GET/DROP/EAT/DRINK/ATTACK/FEED and others to work out
' which item, if any, the player's command actually named.
procedure findMatchedItems()
    global k
    global z8
    global z3
    global d$
    global b$
    global itemname$
    z8 = 0 : z3 = 0 : d$ = ""
    for z5 = 1 to 45
        if k(z5) <> 0 then
            z8 = z8 + 1
            b$ = itemname$(z5) : d$ = b$
            if k(z5) = 1 and z8 = 1 then
                z3 = z5
            end if
        end if
    end for
    b$ = d$
end procedure

' Finds the first object code (1-47, the same range itemname$() covers)
' the player's command mentioned, and sets d$ to its display name --
' used for messages like "What do you want to do with the LAMP?" where
' the exact item doesn't matter, just naming *something* the player
' typed. Replaces stage 2's L8390 GOSUB target.
procedure findFirstNamedItem()
    global k
    global d$
    global itemname$
    dim x1
    x1 = 0
    for z1 = 1 to 47
        if x1 <> 1 then
            if k(z1) = 1 then
                d$ = itemname$(z1) : x1 = 1
            end if
        end if
    end for
end procedure

' Checks whether the player is carrying item z3 (s(z3) = -1 marks an
' item as carried); sets z5 to 1 if so, 0 (and prints "You don't have
' the <a$>") if not -- a$ is whatever the player's command last named,
' set well before this runs, back in the command-parsing code. Replaces
' stage 2's L8490 GOSUB target.
procedure checkCarryingItem()
    global z3
    global s
    global a$
    global z5
    if s(z3) = -1 then
        z5 = 1
    else
        print "You don't have the "; a$
        z5 = 0
    end if
end procedure

' Recomputes the current score from scratch (treasures deposited/found,
' game milestones reached, rooms visited) into s0, and the rooms-visited
' count into z9. Replaces stage 2's L6510 GOSUB target, called only from
' printScore() below.
procedure computeScore()
    global s0
    global z9
    global g
    global sn
    global d1
    global t
    global b1
    global b2
    global p1
    global d2
    global c
    global v
    global t1
    global o
    global s
    restore L230
    z9 = 0 : s0 = 0
    for z0 = 1 to 15
        read z1
        if z1 <> 0 then
            if v(z1) = 1 then
                s0 = s0 + 4 * o(z0)
            end if
            if s(z0) = 7 then
                s0 = s0 + 4 * o(z0)
            end if
        end if
    end for
    s0 = (g = 1) * 10 + s0 : s0 = (sn = 0) * 20 + s0 : s0 = (d1 = 0) * 30 + s0
    s0 = (t = 0) * 30 + s0 : s0 = (b1 = 2) * 20 + s0 : s0 = (b2 = 1) * 20 + s0
    s0 = (p1 = 2) * 20 + s0 : s0 = (d2 = 1) * 20 + s0 : s0 = (c = 1) * 20 + s0
    for z0 = 1 to t1
        if v(z0) = 1 then
            s0 = s0 + 1 : z9 = z9 + 1
        end if
    end for
end procedure

' Prints the player's current score, exploration percentage, and skill
' title (looked up by score tier from the DATA table at L6470 -- z9's
' own rooms-visited value is only needed for the percentage line above,
' so the tier lookup uses its own local variable rather than reusing
' z9 for a second, unrelated purpose the way stage 2 does). Replaces
' stage 2's L6430 GOSUB target.
procedure printScore()
    global s0
    global z9
    global t1
    dim tier, d$
    computeScore()
    print "Your score is now "; s0
    print "You have explored "; (z9 / t1) * t1; "% of the cave."
    restore L6470
    tier = int((s0 - 1) / 100)
    if tier > 4 then
        tier = 4
    end if
    for z0 = 0 to tier
        read d$
    end for
    print "That makes you a "; d$; " adventurer."
end procedure

' ADVENTURE/3000 VERSION 3.2     27 FEB 1979 AT 5:30 PM
' THIS PROGRAM IS RELATIVELY BUG-FREE, BUT ONE STILL MUST
' realize that murphy 'S LAW STILL PREVAILS!
'
' ADVENTURE: PROGRAMMED IN HP/3000 BASIC BY BENJAMIN MOSER
' JAMES MADISON HIGH SCHOOL, VIENNA, VIRGINIA.  THE BASIC LAYOUT OF THE
' GAME WAS CONCEIVED BY DON WOODS & WILLIE CROWTHER, BOTH OF M.I.T.
'
' ADVENTURE WAS PORTED TO THE MACINTOSH PLUS BY THE ELIZABETH AND DAVID HUNTER
' IN MARCH 1998 AND THEN TO PYBASIC FOR THE RASPBERRY RP2040 IN JUNE 2021
' PRINT "Adventure 3.2 on ";date$;" at ";time$
PRINT "Adventure 3.2 for PyBASIC"
open "AMOVING" for INPUT as #4
' ADESCRIP, AITEMS, and AMESSAGE are loaded fully into memory
' below instead of opened here -- this port replaces the original's
' fseek-based random access into them, which BASCAL (and classic
' Microsoft BASIC) has no equivalent for. See ../README.md.
' dirs is an array of possible room directions, it replaces file AMOVING
dim dirs(100,10)
' indx()/fraindx() are now dim'd near the top of the file (see the
' procedure declarations), alongside descrip$()/items$()/msg$()/itemname$().
dim s(99)
dim v(100)
dim k(200)
dim o(15)
PRINT: PRINT "Initializing.";
' initialize
' total rooms, items,and keywords
l1 = int(RND(1)*4)+1 : l2 = l1
g = 0 : b0 = 1 : sn = 1 : d1 = 1 : d2 = 0 : t = 1 : b1 = 0 : b2 = 0 : p1 = 0: dead = 0
l = 0 : c = 0 : d3 = 0 : b3 = 0 : d0 = 2 : t1 = 100 : t2 = 35 : t3 = 149 : r0 = 0 : c0 = 0: c$=""
KC = 1.02
for ii = 1 to 99
    s(ii) = 0 : v(ii) = 0
end for

PRINT ".";:v(100) = 0
indx(1) = -1:indx(2)=-1
'   read in possible movement direction array
for z2 = 1 to 100
    INPUT #4,dx1,dx2,dx3,dx4,dx5,dx6,dx7,dx8,dx9,dx0
    dirs(z2,1)=dx1:dirs(z2,2)=dx2:dirs(z2,3)=dx3:dirs(z2,4)=dx4:dirs(z2,5)=dx5
    dirs(z2,6)=dx6:dirs(z2,7)=dx7:dirs(z2,8)=dx8:dirs(z2,9)=dx9:dirs(z2,10)=dx0
    PRINT ".";
end for

close #4
LA100: ' Load ADESCRIP, AITEMS, AMESSAGE (+ its index), and the short
' item names -- see the procedure declarations near the top of this
' file for what each replaces.
loadAdescrip()
loadAitems()
loadAmessage()
buildMessageIndex()
loadItemNames()
PRINT
restore L230
'    read in locations of items
for z2 = 1 to T2 step 5
    read dx1,dx2,dx3,dx4,dx5:s(z2)=dx1:s(z2+1)=dx2:s(z2+2)=dx3:s(z2+3)=dx4:s(z2+4)=dx5
    PRINT ".";
end for

L230: data 18,25,23,24,21
data 52,0,71,74,58
data 59,69,66,82,100
data 7,49,7,7,7
data 7,12,13,40,38
data 69,0,46,0,0
data 15,60,82,22,250
for ii = 1 to 15 step 5
    read dx1,dx2,dx3,dx4,dx5:o(ii)=dx1:o(ii+1)=dx2:o(ii+2)=dx3:o(ii+3)=dx4:o(ii+4)=dx5
    PRINT ".";
end for

data 1,2,2,2,2
data 3,4,3,3,2
data 5,3,2,3,3
' ASK IF HE WANTS DIRECTIONS
printMessage(301)
z0 = askYesNo%()
if z0 then goto L267 else goto L320
L267: printMessage(302)
'    command INPUT routine
L300: ' PRINT room, items
' If it's dark don't let him see anything
L320: if l1 < 13 or l1 = 58 then goto L350
if l = 1 and (s(18) = l1 or s(18) = -1) then goto L350
printMessage(45)
goto L400
L350: select case d0
    case 0
        shortDescription()
    case 1
        longDescription()
    case else
        describeRoomOnEntry()
end select
v(l1) = 1
describeRoomContents()
if dead = 1 then goto L9540
situationDescriptions()
' INPUT LOOP --- MULTIPLE COMMAANDS, REMOVE JUNK CHARACTERS
L400: if len(c$) > 0 then goto L500
L410: INPUT ">";c$:c$=ucase$(c$)
if c$ = "" then goto L410
PRINT: PRINT
c$ = ucase$(c$)
'    65-90 = A-Z               48-57 = 0-9         46 = . 44 = ,
for x = 1 to len(c$)
    z5 = asc(mid$(c$,x,1))
    if not ((z5 > 64 and z5 < 91) or (z5 > 47 and z5 < 58) or z5 = 44) then
        c$ = mid$(c$,1,x-1)+" "+mid$(c$,x+1)
    end if
end for

if mid$(c$,len(c$),1) = "," then goto L500
c$ = c$+","
L500: z4 = instr(c$,",")
a$ = ucase$(mid$(c$,1,z4-1)) : c$ = mid$(c$,z4+1)
a$ = " "+a$+" "
' search a$ for keywords,puut kwd code into k(x)
' items
L580: data 1,"GOLD",1,"NUGGET",2,"BARS",2,"SILVER",3,"JEWELRY",4,"COINS"
data 5,"DIAMONDS",6,"MING",6,"VASE",7,"PEARL",8,"EGGS",8,"NEST"
data 9,"TRIDENT",10,"EMERALD",11,"PLATINUM",11,"PYRAMID",12,"CHAIN"
data 13,"SPICES",14,"PERSIAN",14,"RUG",15,"TREASURE",15,"CHEST"
data 16,"WATER",17,"OIL",18,"LAMP",18,"LANTERN",19,"KEYS",20,"FOOD",21,"BOTTLE"
data 22,"CAGE",23,"ROD",23,"WAND",24,"CLAM",25,"MAGAZINE",26,"BEAR"
data 27,"AXE",28,"VELVET",28,"PILLOW",29,"SHARDS",30,"OYSTER"
data 31,"BIRD",32,"TROLL",33,"DRAGON",34,"SNAKE",35,"DWARF"
data 36,"ROCK",36,"BOULDER",37,"STAIRS",38,"STEPS",39,"HOUSE",39,"BUILDING"
data 40,"GRATE",41,"STREAM",42,"ROOM",43,"BRIDGE",44,"PIT",45,"VOLCANO"
data 46,"ROAD",47,"ALL",47,"EVERYTHING"
' DIRECTIONS
data 100,"N",100,"NORTH",101,"NE",101,"NORTHEAST",102,"E",102,"EAST"
data 103,"SE",103,"SOUTHEAST",104,"S",104,"SOUTH",105,"SW",105,"SOUTHWEST"
data 106,"W",106,"WEST",107,"NW",107,"NORTHWEST",108,"U",108,"UP",109,"D",109,"DOWN"
' VERBS
data 110,"PLUGH",111,"XYZZY",112,"PLOVER",113,"CROSS",114,"CLIMB",115,"JUMP"
data 116,"FILL",117,"EMPTY",117,"POUR",118,"LOOK",118,"L",119,"LIGHT",119,"ON",120,"EXTINGUISH"
data 120,"OFF",121,"IN",121,"ENTER",122,"LEAVE",122,"OUT",123,"INVENTORY",123,"I"
data 124,"GET",124,"CATCH",124,"TAKE",125,"DROP",125,"DUMP",126,"THROW",127,"ATTACK"
data 127,"KILL",128,"FEED",129,"WATER",130,"LOCK",131,"UNLOCK"
data 132,"FREE",132,"RELEASE",133,"WAVE",134,"OPEN",135,"CLOSE"
data 136,"OIL",137,"EAT",138,"DRINK",139,"FEE FIE FOE FOO"
data 140,"SHORT",141,"LONG",142,"BRIEF",143,"QUIT",143,"STOP",143,"END"
data 144,"SCORE",145,"SAVE",146,"LOAD",147,"READ",147,"EXAMINE"
data 148,"YES",148,"Y",149,"BUG",150,"*"
restore L580
for i = 1 to 200
    k(i) = 0
end for

z1 = 0 : z3 = 0
' T3=TOTAL NUMBER OF KEYWORDS
L820: if z1 > t3 then goto L900
read z1,b$
b$ = " "+b$+" "
' IF KEYWORD WAS FOUND, NOTE THIS IN K(Z1)
if instr(a$,b$) = 0 then goto L820
k(z1) = 1
' KEYWORDK #Z1 NOT FOUND
goto L820
' EXOTIC WORDS
L900: z0 = 36
x = z0
LW910: if x > 46 then goto L960
if k(x) = 0 then goto LCONT910
findFirstNamedItem()
L940: print "What do you want to do with the ";d$;"?"
goto L410
LCONT910: x = x + (1)
goto LW910
L960:
x = 110
LW970: if x > t3 then goto L990
if k(x) = 1 then goto L1950
x = x + (1)
goto LW970
L990:
' THEN IT'S A DIRECTION
d = 1
LW1010: if d > 10 then goto L1030
if k(d+99) = 1 then goto L1070
d = d + (1)
goto LW1010
L1030:
' COMMAND NOT A DIRECTION
goto L1950
' CAN HE MOVE THAT WAY?
L1070: z2 = dirs(L1,d)
if z2 = 255 then goto L1470
if z2 < 1 or z2 > 254 then goto L1220
L1150: ' NORMAL MOVING
' CHECK FOR SPECIAL MOVE CONDITIONS
goto L1260
L1180: l2 = l1 : l1 = z2
if s(35) = l2 then goto L1191 else goto L1200
L1191: s(35) = l1
L1200: goto L9780
L1220: printMessage(1)
goto L400
' SPECIAL ROOM DIRECTIONS
' GRATE
L1260: if L1 = 10 and (d = 10 or d = 5) THEN goto L1280
IF L1 = 11 and (d = 9 or d = 3) then goto L1280 else goto L1320
' IF GRATE IS OPEN (G=0) MOVE HIM
L1280: if g = 1 then goto L1180
printMessage(10)
goto L400
' CAN'T TAKE NUGGET UPPSTAIRS
L1320: if not (l1 = 17 and d = 9 and s(1) = -1) then goto L1360
printMessage(38)
goto L400
' CRYSTAL BRIDGE AND FISSURE
L1360: if not (l1 = 19 and d = 7 or l1 = 20 and d = 3) then goto L1410
if b2 then goto L1180
printMessage(3)
goto L400
' MT. KING & SNAKE
L1410: if not (l1 = 22 and d <> 3 and d <> 9) then goto L1690
if sn = 0 then goto L1180
printMessage(50)
goto L400
' bedquilt and random directiosn
L1470: if l1 <> 44 then goto L1590
if RND(1) > 0.5 then goto L1510
printMessage(52)
goto L400
L1510: restore L1530
' ROOMS TO JU
L1530: data 33,37,45,92,76
for z3 = 1 to int(RND(1)*5)+1
    read z2
end for

goto L1180
' WITT's END
L1590: if l1 <> 39 then goto L1220
' SHOULD WE LET HIM OUT?
if RND(1) < 0.15 then goto L1650
' NO
printMessage(52)
goto L400
L1650: ' YES
z2 = 38
goto L1180
' Narrow Tunnel
L1690: if not (l1 = 57 or l1 = 58) then goto L1780
IF K(102)=0 AND K(106)=0 THEN goto L1780
z3 = 1
LW1700: if z3 > t2 then goto L1750
if z3 = 10 then goto LCONT1700
if s(z3) <> -1 then goto LCONT1700
printMessage(53)
goto L400
LCONT1700: z3 = z3 + (1)
goto LW1700
L1750:
goto L1180
' TROLL
L1780: IF L1=60 AND D=2 THEN goto L1790
IF L1=61 AND D=6 THEN goto L1790 ELSE goto L1860
L1790: ' t is 0 (no troll met yet), 1 (troll appeased, gone), or 2 (troll
' killed) -- L1180 (case 1) is the shared "do the move" label used all
' over the program, not something owned by this dispatch, so that case
' is just a redirect rather than embedding L1180's own body here.
select case t+1
    case 1
        goto L1180
    case 2
        printMessage(55)
        goto L400
    case 3
        printMessage(56)
        printMessage(55)
        T=1
        goto L400
    case 4
        t = 2
        goto L1180
end select
L1860: if not (l1 = 73 and d = 1 and d2 = 0) then goto L1890
printMessage(57)
goto L400
L1890: if not (l1 = 82 and s(33) = l1 and d = 1) then goto L1180
' DRAGON
printMessage(51)
goto L400
' OTHER COMMANDS
L1950: z1 = 100
LW1950: if z1 > t3 then goto L1970
if k(z1)=1 then goto L2090
z1 = z1 + (1)
goto LW1950
L1970:
' ITEM BO NO VERB?
' (was: restore L9961 -- item names now come from itemname$())
x = 1
LW2000: if x > 35 then goto L2030
d$ = itemname$(x)
if k(x) = 1 then goto L940
x = x + (1)
goto LW2000
L2030:
L2040: restore L2070
for x = 1 to int(RND(1)*4)+1
    read b$
end for

PRINT b$
L2070: data "What?","I don't understand.","I can't understand that.","I don't know that word."
goto L400
L2090: z1 = z1-109
select case z1
    case 1
        ' *** PLUGH ***
        IF L1<>7 THEN goto L2170
        IF S(35)=L1 THEN S(35)=0
        Z2=26
        GOTO L1180
        L2170: IF L1<>26 THEN goto L2200
        Z2=7
        GOTO L1180
        L2200: printMessage(2)
        GOTO L400
    case 2
        ' *** XYZZY ***
        IF L1<>7 THEN goto L2270
        IF S(35)=L1 THEN S(35)=0
        Z2=13
        GOTO L1180
        L2270: IF L1<>13 THEN goto L2200
        Z2=7
        GOTO L1180
    case 3
        ' *** PLOVER *** (CAN'T BRING EMERALD WITH HIM)
        IF L1>26 THEN goto L2360
        IF S(35)<>L1 THEN goto L2330
        S(35) = 0
        L2330: IF S(10)<>-1 THEN goto L2340
        S(10) = L1
        L2340: Z2 = 58
        GOTO L1180
        L2360: IF L1<>58 THEN goto L2200
        Z2=26
        GOTO L1180
    case 4
        ' *** CROSS ***
        IF L1<>19 THEN goto L2470
        IF B2<>0 THEN goto L2440
        L2420: printMessage(3)
        GOTO L400
        L2440: D=7
        ' JUST GIVE NEW DIRECTION, USE MOVE ROUTINE
        GOTO L1070
        L2470: IF L1<>20 THEN goto L2510
        IF B2=0 THEN goto L2420
        D=3
        GOTO L1070
        L2510: IF L1<>60 THEN goto L2540
        D=2
        GOTO L1070
        L2540: IF L1<>61 THEN goto L2200
        D=6
        GOTO L1070
    case 5
        ' *** CLIMB ***
        IF L1<>50 THEN goto L2200
        ' CAN HE CLIMB BEANSTALK?
        IF P1<2 THEN goto L2200
        ' YES
        Z2=70
        GOTO L1180
    case 6
        ' *** JUMP *** STRICTLY SUICIDAL
        IF L1<>16 AND L1<>19 AND L1<>20 AND L1<>27 THEN goto L2200
        printMessage(4)
        GOTO L9550
    case 7
        ' FILL
        IF S(21)=-1 THEN goto L2730
        L2700: B$="bottle"
        L2710: PRINT "You don't have the ";b$
        goto L410
        L2730: IF B0=0 THEN goto L2760
        printMessage(5)
        GOTO L410
        L2760: IF L1<>7 AND L1<>8 AND L1<>9 AND L1<>35 AND L1<>74 AND L1<>81 THEN goto L2790
        B0=1:S(16)=-1
        GOTO L2840
        L2790: IF L1=49 THEN goto L2830
        B$="oil"
        L2810: PRINT "I see no ";B$;" here."
        GOTO L400
        L2830: B0=2:S(17)=-1
        L2840: PRINT "The bottle is now filled."
        GOTO L400
    case 8
        ' *** EMPTY ***
        IF S(21)=-1 THEN goto L2890
        GOTO L2700
        L2890: ' EMPTY BOTTLE (ASSUMED FULL)
        S(B0+15)=0:B0=0
        PRINT "Emptied"
        GOTO L400
    case 9
        ' *** LOOK ***
        L2940: if l1 < 13 or l1 = 58 then goto L2970
        if l = 1 and (s(18) = l1 or s(18) = -1) then goto L2970
        printMessage(45)
        goto L400
        L2970: longDescription() ' (was: gosub 8050 -- jumped past the old
        ' subroutine's own `v(l1)=1` to avoid redundantly re-marking the room
        ' visited; by the time LOOK is typeable the room's already been entered
        ' via describeRoomOnEntry(), which already sets v(l1)=1, so calling the
        ' full longDescription() here just re-does that no-op assignment)
        describeRoomContents()
        goto L400
    case 10
        ' *** LIGHT ***
        if s(18) = -1 then goto L3040
        L3020: b$ = "lamp"
        goto L2710
        L3040: l = 1
        b$ = "on"
        L3060: PRINT "The lamp is now ";b$
        goto L2940
    case 11
        ' *** OFF (EXTINGUSIH) ***
        IF S(18)=-1 THEN goto L3110
        GOTO L3020
        L3110: L=0:B$="off"
        GOTO L3060
    case 12
        ' *** ENTER ***
        IF L1<>6 THEN goto L3180
        ' TO HOUSE
        D=3
        GOTO L1070
        L3180: IF L1<>68 THEN goto L3240
        ' TO BARREN ROOM
        D=3
        GOTO L1070
        L3240: D = 10
        LW3240: if D < 1 then goto L3270
        Z2 = DIRS(L1,D)
        IF Z2>0 AND Z2<101 THEN goto L1150
        D = D + (-1)
        goto LW3240
        L3270:
        GOTO L2200
    case 13
        ' ** LEAVE ***
        IF L1<>7 THEN goto L3340
        ' LEAVE HOUSE
        D=7
        GOTO L1070
        L3340: IF L1<>69 THEN goto L3400
        ' LEAVE BARREN ROOM
        D = 7
        GOTO L1070
        L3400: D = 1
        LW3400: if D > 10 then goto L3430
        Z2 = DIRS(L1,D)
        IF Z2>0 AND Z2<101 THEN goto L1150
        D = D + (1)
        goto LW3400
        L3430:
        GOTO L2200
    case 14
        ' *** INVENTORY ***
        Z0=0
        PRINT "You are carrying:";
        FOR X=1 TO T2
            IF S(X)=-1 THEN
                b$ = itemname$(x)
                PRINT B$
                Z0 = Z0 + 1
            end if
        end for

        IF Z0=0 THEN goto L3551 else goto L3560
        L3551: PRINT "nothing."
        L3560: PRINT
        GOTO L400
    case 15
        ' *** GET ***
        if k(47) = 1 then goto L3680
        findMatchedItems()
        if z8 > 0 then goto L3680
        PRINT "Get what?"
        goto L2040
        L3680: z3 = 1
        LW3680: if z3 > t2 then goto L3900
        if k(47) = 1 then goto L3730
        if k(z3) = 0 then goto LCONT3680
        L3730: if s(z3) <> l1 then goto L3750
        if s(z3) = l1 then goto L3790
        L3750: if k(47) = 1 then goto LCONT3680
        a$ = itemname$(z3):PRINT a$;" not here."
        goto LCONT3680
        ' MUST CHECK NOW FOR LEGALITY OF TAKING ITEM
        L3790: z8 = 0
        for x = 1 to t2
            if s(x) = -1 then
                z8 = z8+1
            end if
        end for

        if z8 < 7 then goto L3870
        ' CARRYING TOO MUCH
        printMessage(54)
        goto L410
        L3870: goto L6880
        L3880: s(z3) = -1
        L3890: a$ = itemname$(z3):PRINT a$;":taken."
        LCONT3680: z3 = z3 + (1)
        goto LW3680
        L3900:
        goto L400
    case 16
        ' *** DROP ***
        if k(47) = 1 then goto L4000
        findMatchedItems()
        IF Z8>0 THEN goto L4000
        PRINT "Drop what?"
        GOTO L2040
        L4000: Z3 = 1
        LW4000: if Z3 > T2 then goto L4140
        IF K(47)=1 THEN goto L4060
        IF K(Z3)<>1 THEN goto LCONT4000
        IF S(Z3)=0 THEN goto LCONT4000
        L4060: IF S(Z3)=-1 THEN goto L4100
        IF K(47)=1 THEN goto LCONT4000
        b$ = itemname$(z3):PRINT "You don't have the ";B$
        GOTO LCONT4000
        L4100: ' STILL NEED TO ELABORATE ON DROP (BIRD IN CAGE, BOTTLE)
        GOTO L7380
        L4120: b$ = itemname$(z3):PRINT B$;":dropped."
        S(Z3)=L1
        LCONT4000: Z3 = Z3 + (1)
        goto LW4000
        L4140:
        GOTO L400
    case 17
        ' *** THROW ***
        findMatchedItems()
        IF Z8>0 THEN goto L4210
        PRINT "Throw what?"
        GOTO L2040
        L4210: IF S(Z3)<>-1 THEN goto L2710
        IF NOT (Z3<16 AND S(32)=L1) THEN goto L4260
        ' THROW TREASURE TO TROLL
        printMessage(27)
        S(Z3)=0:T=3:GOTO L400
        L4260: IF NOT (Z3=27 AND S(32)=L1) THEN goto L4300
        ' TRYING TO BUTCHER TROLL?
        printMessage(26)
        S(27)=L1:GOTO L400
        L4300: IF NOT (Z3=27 AND S(35)=L1) THEN goto L4380
        ' TRYING TO KILL DWARF
        IF RND(1)>0.5 THEN goto L4360
        printMessage(29)
        checkDwarfAttack() ' (was: GOSUB 8650 -- jumped straight into the
        ' attack-check half of the original dwarf subroutine, deliberately
        ' skipping the axe-giving check; see checkDwarfAttack()'s own comment)
        GOTO L4410
        L4360: printMessage(30)
        S(35)=0:GOTO L4410
        L4380: ' NOTHING SPECIAL, JUST DROP ITEM
        IF S(35)<>L1 THEN goto L4400
        checkDwarf() ' (was: GOSUB L8550 -- L8550 was just a comment
        ' immediately before the real dwarf subroutine's first line, L8560, so
        ' this call wanted the full checkDwarf() behavior, axe-check included --
        ' a call site missed when checkDwarf()/checkDwarfAttack() were split out)
        L4400: PRINT "Thrown."
        L4410: S(Z3) = L1
        if dead = 1 then goto L9540
        GOTO L400
    case 18
        ' *** ATTACK ***
        findMatchedItems()
        IF NOT (Z3=33 AND S(Z3)=L1 AND L1=82) THEN goto L4520
        ' HE CAN KILL DRAGON
        printMessage(68)
        GOTO L410
        L4520: IF S(32)<>L1 THEN goto L4560
        ' TRYING TO MUNGE TROLL
        ' (was: Z9=FNA(25):GOTO 400 -- FNA is called with no matching DEF FN
        ' anywhere in the original source; this looks like leftover/broken code
        ' from an earlier version of the upstream port, not something this port
        ' introduced. Z9's assignment here isn't read before it's next assigned
        ' elsewhere, so dropping the call changes nothing observable.)
        GOTO L400
        L4560: IF NOT (Z3=26 OR Z3>30) THEN goto L4600
        ' DANGEROUS TO ATTACK THESE
        printMessage(70)
        GOTO L400
        L4600: ' NOTHING TO ATTACK
        printMessage(71)
        GOTO L400
    case 19
        ' *** FEED ***
        findMatchedItems()
        IF Z3<>35 THEN goto L4690
        ' CAN'T FEED DWARF!
        printMessage(24)
        GOTO L400
        L4690: IF S(20) = -1 THEN goto L4720
        B$ = "FOOD":GOTO L2710
        L4720: IF L1=69 THEN goto L4760
        PRINT "I can't feed it."
        printMessage(23)
        GOTO L400
        L4760: IF S(20)=L1 THEN goto L7600
        B1=1:S(20)=0:printMessage(6)
        GOTO L400
    case 20
        ' *** WATER ***
        IF S(16) = -1 THEN goto L4840
        B$ = "water":GOTO L2710
        L4840: IF L1<>50 THEN goto L2200
        ' GOTO P1+1 OF 4860,4890,4920
        IF P1 = 0 THEN goto L4860
        IF P1 = 1 THEN goto L4890
        IF P1 = 2 THEN goto L4920
        L4860: printMessage(7)
        P1=1:S(16)=0:B0=0:GOTO L400
        L4890: printMessage(8)
        P1=2:S(16)=0:B0=0:GOTO L400
        L4920: printMessage(9)
        P1=0:S(16)=0:B0=0:GOTO L400
    case 21
        ' *** LOCK ***
        L4960: IF L1=10 OR L1=11 THEN goto L4990
        ' NOTHING LOCKABLE
        GOTO L2200
        L4990: IF S(19)=-1 THEN goto L5020
        L5000: B$="keys":goto L2710
        L5020: G=0:printMessage(10)
        GOTO L400
    case 22
        ' *** UNLOCK ***
        L5070: IF S(19)<>-1 THEN goto L5000
        IF L1<>10 AND L1<>11 THEN goto L5120
        G=1:printMessage(11)
        GOTO L400
        L5120: IF L1<>69 THEN goto L2200
        IF B1>0 THEN goto L5160
        printMessage(12)
        goto L400
        L5160: IF C<>0 THEN goto L5170
        C=1:B1=2
        L5170: printMessage(13)
        GOTO L400
    case 23
        ' *** FREE ***
        IF K(31) = 1 THEN goto L5240
        ' CAN'T FREE ANYTHING BUT BIRD
        L5220: printMessage(2)
        GOTO L410
        L5240: IF S(31)<>-1 THEN goto L5220
        S(31) = L1:B3=0
        PRINT "Freed."
        IF L1<>22 THEN goto L5350
        IF SN<>1 THEN goto L400
        B$="snake"
        L5300: PRINT "The little bird attacks the green ";B$;" and"
        IF L1=82 THEN goto L5380
        PRINT "drives it off"
        SN=0:S(34)=0:GOTO L400
        L5350: IF L1<>82 THEN goto L400
        B$="dragon":GOTO L5300
        L5380: PRINT "gets burned to a crisp"
        S(31)=0
        GOTO L400
    case 24
        ' *** WAVE ***
        IF K(23) <> 1 THEN goto L2200
        IF S(23)=-1 THEN goto L5460
        B$="rod":GOTO L2710
        L5460: '  IS HERE NEAR FISSURE
        IF L1<>19 AND L1<>20 THEN goto L2200
        ' yes
        ' GOTO B2+1 OF 5500,5530
        IF B2=0 THEN goto L5500
        IF B2=1 THEN goto L5530
        L5500: printMessage(14)
        B2=1:GOTO L400
        L5530: printMessage(15)
        B2=0:GOTO L400
    case 25
        ' *** OPEN ***
        findMatchedItems()
        IF Z3>0 THEN goto L5610
        PRINT "Open ";:goto L2040
        L5610: IF Z3=40 THEN goto L5070
        IF S(Z3)=L1 THEN goto L5650
        L5630: PRINT "I see no ";b$;" here.":goto L400
        L5650: if z3=24 THEN goto L5680
        PRINT "I don't know how to open a ";B$:GOTO L400
        L5680: IF S(9)=-1 THEN goto L5710
        printMessage(16)
        GOTO L400
        L5710: IF S(Z3) = 0 THEN goto L2200
        ' HE'S OPENED CLAM, SO PRINT DESCRIPTION OF THIS
        ' PUT PEARL IN CUL-DE-SAC
        S(7)=43:S(24)=0:S(30)=L1:printMessage(17)
        GOTO L400
    case 26
        ' *** CLOSE *** -- an upstream bug (present already in stage 2/3,
        ' not introduced by this refactor): the dispatch table points CLOSE's
        ' entry at this exact "GOTO L400" line -- the same line OPEN's own
        ' handler ends on above -- rather than at the real CLOSE logic just
        ' below (L5760-L5800), which is consequently dead code, unreachable
        ' from anywhere. Preserved exactly as-is (CLOSE silently does nothing,
        ' matching the original's real behavior) rather than "fixed" to call
        ' the logic that was clearly intended; this is a faithfulness refactor,
        ' not a gameplay bugfix. Split into its own copy of the shared line so
        ' each verb has its own case block below.
        GOTO L400
        ' *** CLOSE *** (dead code -- see the comment above)
        findMatchedItems()
        IF Z3=40 THEN goto L4960
        printMessage(18)
        GOTO L400
    case 27
        ' OIL
        IF K(17)=0 THEN goto L2200
        IF S(17)=-1 THEN goto L5860
        B$="oil":GOTO L5630
        L5860: IF L1<>73 THEN goto L2200
        ' IS DOOR STILL RUSTED
        IF D2=1 THEN goto L2200
        D2=1:S(17)=0:B0=0:printMessage(19)
        GOTO L400
        ' *** EAT ***
    case 28
        IF K(20) = 1 THEN goto L5950
        printMessage(20)
        GOTO L410
        L5950: Z3=20:checkCarryingItem()
        IF Z5=0 THEN goto L410
        printMessage(73)
        S(20)=0:B0=0:GOTO L400
    case 29
        ' *** DRINK ***
        IF K(16) =1 THEN goto L6040
        printMessage(21)
        GOTO L410
        L6040: Z3=16:checkCarryingItem()
        IF Z5=0 THEN goto L410
        printMessage(22)
        S(17)=0:B0=0:GOTO L400
    case 30
        ' *** FEE FIE FOE FOO ***
        IF L1=71 THEN goto L6130
        printMessage(2)
        GOTO L410
        L6130: IF S(8)<>L1 THEN goto L6180
        ' MAKE NEST VANISH
        printMessage(79)
        S(8)=0:GOTO L400
        L6180: ' IF S(8)=0 THEN goto L6110
        S(8)=L1
        ' MAKE NEST RE-APPEAR
        printMessage(81)
        GOTO L400
    case 31
        ' *** SHORT ***
        PRINT "Short descriptions"
        D0=0:GOTO L400
    case 32
        ' *** LONG ***
        PRINT "Long descriptions"
        D0=1:GOTO L400
    case 33
        ' *** BRIEF ***
        PRINT "OK, I'll only describe the room in detail the first time."
        D0=2:GOTO L400
    case 34
        ' *** QUIT ***
        PRINT "Save game";
        Z0 = askYesNo%()
        IF Z0=1 THEN goto L8970
        GOTO L9750
        ' SCORE ***
    case 35
        printScore()
        GOTO L400
        L6430: ' score computation/printing are now computeScore()/
        ' printScore() above -- the DATA line right below is still needed
        ' here (RESTORE targets must be top-level, per issue #149/PR #150).
        L6470: DATA "beginner","novice","experienced","advanced","expert"
        ' list items at location l1
        ' room-contents listing + dwarf/pirate checks are now the
        ' describeRoomContents() procedure above.
        ' Print Short room description
        ' short room description is now the shortDescription() procedure above.
        L6880: ' SPECIAL GETS
        if not (z3 = 24 or z3 = 30 or z3 > 31) then goto L6930
        ' CAN'T GET THESE FOR SOME REASON
        printMessage(61)
        goto L400
        L6930: if not (z3 = 12 and c = 0) then goto L6970
        ' CHAIN
        printMessage(58)
        goto L3900
        L6970: ' BEAR IS HE FED? UNLOCKED?
        if not (z3 = 26 and b1 <> 2) then goto L7010
        printMessage(61)
        goto L3900
        L7010: if not (z3 = 14 and d1 = 1) then goto L7050
        ' DRAGON AND RUG
        printMessage(59)
        goto L3900
        L7050: if not (z3 = 16 or z3 = 17) then goto L7090
        ' OIL AND WATER DO SAME AS FILL
        PRINT "Why not say 'fill'?"
        goto L3900
        L7090: if not (z3 = 22 and b3) then goto L7140
        ' TAKE BIRD SINCE IT'S IN CAGE
        s(31) = -1:PRINT "Bird and ";:goto L3880
        L7140: if z3 <> 31 then goto L7310
        ' GETTING BIRD
        if b3 <> 1 then goto L7210
        ' TAKE CAGE, SINCE BIRD IS IN IT
        PRINT "Cage and ";:s(22) = -1:goto L3880
        L7210: if s(22) = -1 then goto L7240
        b$ = "cage":goto L2810
        L7240: if s(23) = -1 then goto L7280
        ' OK TO TAKE BIRD
        s(31) = -1 : b3 = 1:goto L3890
        L7280: ' ROD SCARES BIRD
        printMessage(37)
        goto L3900
        L7310: ' BOTTLE FULL? IF SO, GET CONTENTS
        if not (z3 = 21 and b0) then goto L7360
        PRINT "Contents and the ";
        s(b0+15) = -1
        L7360: goto L3880
        ' SPECIAL "DROP"
        L7380: IF Z3<>31 THEN goto L7440
        ' BIRD IN CAGE
        L7400: S(31)=L1:S(22)=L1:B3=1
        IF Z3<>31 THEN goto L7420
        PRINT "Cage and ";
        L7420: IF Z3=22 THEN goto L7430
        PRINT "Bird and ";
        L7430: goto L4120
        L7440: if z3=22 and b3=1 then goto L7400
        IF Z3<>21 THEN goto L7520
        ' BOTTLE
        IF B0=0 THEN goto L4120
        ' BOTTLE IS FULL, DO DROP CONTENTS TOO
        PRINT "Contents and ";
        S(15+B0)=L1:GOTO L4120
        L7520: IF NOT (Z3=16 OR Z3=17) THEN goto L7541
        PRINT "Try saying 'empty'":goto L4140
        L7541: IF Z3<>26 OR T<>1 OR (L1<>60 AND L1<>61) THEN goto L7550
        printMessage(28)
        T=0:S(26)=L1:S(32)=0:GOTO L400
        L7550: IF Z3<>6 THEN goto L4120
        IF S(28)=L1 THEN goto L7600
        ' GOODBYE, FRAGILE VASE!
        printMessage(43)
        S(6)=0:S(29)=L1:GOTO L400
        L7600: printMessage(60)
        GOTO L4120
        ' PRINT MESSAGE is now the printMessage() procedure above -- every
        ' `z59 = N : gosub 7620` call site became `printMessage(N)`.
        L7800: ' situation descriptions are now the situationDescriptions()
        ' procedure above.
        ' long room description is now the longDescription() procedure above.
        ' the short-vs-long-on-entry decision is now the describeRoomOnEntry()
        ' procedure above.
        '
        '   fetch first item code in k(1 to t2)
        '   z8=total # of items found in list
        '   z3=item code first found
        L8280: ' find-matched-items/find-first-named-item/check-carrying-item
        ' are now findMatchedItems()/findFirstNamedItem()/checkCarryingItem()
        ' above.
        ' dwarf/pirate checks are now checkDwarf()/checkDwarfAttack()/
        ' checkPirate() above.
    case 36
        L8970: ' *** SAVE GAME ***
        INPUT "What do you want to call the save file? ";A$
        ' `on error goto` (not `try`/`catch`) here on purpose -- see the
        ' README's own note on why, and GitHub issues #61 (on error goto
        ' is permanently unsupported under --target c, by design) & #100
        ' (try/catch's RESUME output isn't accepted by real fbc).
        on error goto L9010
        OPEN A$ FOR OUTPUT AS #5
        on error goto 0
        GOTO L9030
        L9010: PRINT "File ";a$;" not created"
        GOTO L410
        L9030: PRINT #5,T1;",";T2;",";T3;",";L1;",";L2;",";G;",";B0;",";SN;",";D1;",";D2;",";D0;",";T;",";B1;",";B2;",";P1;",";L;",";C;",";D3;",";B3;",";R0;",";KC
        FOR X=1 TO 99
            PRINT #5,S(X);",";V(X)
        end for

        PRINT #5,V(100)
        CLOSE #5
        PRINT "Game saved"
        C0 = 0
        if k(143) = 1 then goto L9750
        GOTO L410
    case 37
        ' *** LOAD OLD GAME ***
        IF C0=0 THEN goto L9120
        PRINT "You already have a loaded game!"
        GOTO L410
        L9120: INPUT "Save file name? ";A$
        on error goto L9150
        OPEN A$ FOR INPUT AS #5
        on error goto 0
        GOTO L9170
        L9150: PRINT "Unable to use file ";A$
        GOTO L410
        L9170: INPUT #5,T1,T2,T3,L1,L2,G,B0,SN,D1,D2,D0,T,B1,B2,P1,L,C,D3,B3,R0,KCX$
        kc = val(kcx$)
        FOR X=1 TO 99
            INPUT #5,SX,VX:s(x)=sx:v(x)=vx
        end for

        INPUT #5,vx:v(100)=vx
        CLOSE #5
        C0=1
        GOTO L300
    case 38
        ' *** READ THE MAGAZINE ***
        findMatchedItems()
        IF Z3=25 THEN goto L9270
        printMessage(74)
        GOTO L410
        L9270: IF S(25)=-1 THEN goto L9300
        B$="magazine"
        GOTO L2710
        L9300: ' OK, LET HIM READ IT
        printMessage(303)
        GOTO L400
    case 39
        ' *** YES (only meaningful in the dragon's lair, room 82) *** --
        ' physically relocated here from inside the ATTACK handler's own body
        ' (stage 2/3 had it at its original numeric position, L4490, sitting
        ' between L4480 and L4520 -- reachable only via the dispatch table,
        ' never by ATTACK's own fall-through, so moving it doesn't change
        ' behavior) so its case can be its own block in the SELECT CASE below.
        IF L1<>82 THEN goto L2040
        printMessage(69)
        S(33)=0:D1=0:GOTO L400
    case 40
        ' *** BUG ***
        A$ = "ADVBUGS.TXT"
        on error goto L9150
        OPEN A$ FOR APPEND AS #5
        on error goto 0
        INPUT "Your name: ";A$
        A$=A$+" "+DATE$
        PRINT #5,A$
        PRINT "Enter your gripe in up to five lines (hit return to quit):"
        Z0 = 1
        LW9430: if Z0 > 5 then goto L9480
        PRINT Z0;
        INPUT A$
        IF A$="" THEN goto L9490
        PRINT #5,A$
        Z0 = Z0 + (1)
        goto LW9430
        L9480:
        L9490: PRINT "Message recorded. Thank you!"
        CLOSE #5
        GOTO L410
    case else
        goto L2040
end select
L9540: ' REINCARNATE HIM
L9550: R0=R0+1
IF R0=1 THEN goto L9580
IF R0=2 THEN goto L9610
IF R0=3 THEN goto L9740
' ASK HIM IF HE WANTS TO BE REINCARNATED
L9580: printMessage(75)
Z0 = askYesNo%()
GOTO L9630
L9610: printMessage(77)
GOTO L9580
L9630: IF Z0=0 THEN goto L9750
printMessage(76)
' PUT HIM BACK IN HOUSE, REARRANGE HIS STUFF
S(18)=7:L=0:DEAD=0:KC=1.03
FOR X=1 TO T2
    IF S(X)=-1 THEN
        S(X)=L1
    end if
end for

' WE'VE PUT THE LAMP IN HOUSE AND OTHER ITEMS WHERE HE DIED
L1=INT(RND(1)*4)+1:L2=L1
GOTO L320
' THIRD DEATH--END OF GAME
L9740: printMessage(78)
L9750: PRINT "Oh well..."
printScore()
' (ADESCRIP/AITEMS/AMESSAGE are in-memory arrays now -- nothing to close)
STOP
L9780: ' *** PITS ***
if l1 < 13 THEN goto L300
IF l = 1 and (s(18) = -1 or s(18) = l1) then goto L300
' IS HE GOING TO FALL INTO A PIT?
if l1 = 16 or l1 = 17 or l1 = 19 or l1 = 20 or l1 = 25 or l1 = 47 or l1 = 48 or l1 = 59 or l1 = 60 or l1 = 61 or l1 = 75 or l1 = 76 or l1 = 98 then goto L9840
goto L300
' he fell into a pit
L9840: printMessage(44)
goto L9540
' *** SEEK A "YES" OR "NO" is now the askYesNo%() function above.
' ---- SHORT NAMES FOR STUFF ----
L9961: data "large gold nugget"
data "bars of silver"
data "precious jewelry"
data "many coins"
data "several diamonds"
data "fragile ming vase"
data "glistening pearl"
data "nest of golden eggs"
data "jewel-encrusted trident"
data "egg-sized emerald"
data "platinum pyramid"
data "golden chain"
data "rare spices"
data "persian rug"
data "treasure chest"
data "water"
data "oil"
data "brass lamp"
data "keys"
data "food"
data "bottle"
data "wicker cage"
data "3-foot black rod"
data "clam"
data "magazine"
data "bear"
data "axe"
data "velvet pillow"
data "shards of pottery"
data "oyster"
data "bird"
data "troll"
data "dragon"
data "snake"
data "dwarf"
data "rock"
data "stairs"
data "steps"
data "house"
data "grate"
data "stream"
data "room"
data "bridge"
data "pit"
data "volcano"
data "road"
data "everything"
' INITIALIZE MESSAGE INDEX is now the buildMessageIndex() procedure above.

```

</details>

<details class="source-embed" markdown="1">

<summary><code>stage5-refactored-bascal/adventure.bcl</code> -- Stage 5 — the command-parsing cascade and movement engine</summary>

```bascal

// ADVENTURE/3000 -- Stage 5: structuring the outer game loop and the
// command-parsing/movement engine -- picks up exactly where stage 4
// left off. See ../README.md for the port's provenance, staging, and
// exactly how far this refactor has gotten so far (it is NOT a
// complete rewrite).
//
// Started as an exact copy of stage 4. Progress so far: the
// command-parsing cascade between the keyword-matching DATA table and
// the verb dispatch's own SELECT CASE -- eight genuinely self-contained
// "scan until match" loops (keyword matching, exotic-word/direction-
// word/direction-of-movement checks, the narrow-tunnel item check, and
// the two "was it a keyword/an item" checks right before dispatch) --
// are now WHILE/FOR loops, verified byte-identical output against
// stage 4 for the same input. Two shared GOTO-reached mini-routines
// uncovered along the way became procedures instead of loops, since
// they were never loops to begin with -- askWhatToDoWithItem() (was
// L940, reached from two different loops) and printDontUnderstand()
// (was L2040, reached from six different verb handlers' own "didn't
// understand" case, its DATA line deliberately left at its original
// position -- see printDontUnderstand()'s own comment for why).
//
// Second pass: the movement-execution engine (was L1070 onward) is now
// four functions instead of a GOTO web -- confirmed to be a de facto
// shared subroutine, not a loop at all (bcc's own loop-detection never
// flagged it), reached via GOTO from inside the movement logic itself
// *and* directly from PLUGH/XYZZY/PLOVER/CLIMB/CROSS/ENTER/LEAVE as a
// "teleport"/"retry with a new direction" mechanism -- every one of
// those call sites now calls the same functions instead:
//   - attemptMove%(d%) (was L1070) -- looks up the destination room
//     for direction d%, handling the "pick one at random" sentinel
//     dirs() can return (attemptRandomMove%(), was L1470) and an
//     out-of-range lookup (the original L1220 message) itself.
//   - checkSpecialRoomAndMove%(d%, z2%) (was L1260 onward) -- the
//     special-room/special-direction checks (grate, nugget, bridge,
//     snake, narrow tunnel, troll, dragon), reachable either through
//     attemptMove%() or directly from a couple of verb handlers that
//     already know the destination room. A straightforward `elseif`
//     chain, not a restructuring of the original logic: every
//     condition tests a mutually exclusive room number, so the
//     original's linear GOTO cascade and this chain are behaviorally
//     identical, confirmed byte-identical output against stage 4
//     across six smoke tests covering every branch (movement in all
//     8 directions, PLUGH/XYZZY/PLOVER teleports, CLIMB, CROSS,
//     ENTER/LEAVE).
//   - performMove%(z2%) (was L1180) -- the actual room transition.
// A function can't GOTO a top-level label the way the original
// GOTO-based targets could, so every call site now checks the
// returned 0/1 and GOTOs L400 (a message was printed) or L9780 (the
// move happened) itself instead.
//
// One thing remains: the outer game loop (`L300`/`L320`/`L400`/
// `L410`) every one of the 40 verb handlers `GOTO`s back into once
// it's done -- almost all of `bcc`'s remaining warnings on this file
// are sites jumping back into it, not loops of their own. Converting
// it means restructuring the whole file's top-level control flow, not
// a per-handler change -- flagged as `stage 6`'s job, not attempted
// here.
program adventure3000

' The original used PyBASIC's UPPER$/LOWER$, which real BASIC (and BASCAL's
' own basic-target output) doesn't have; BASCAL's stdlib UCASE$/LCASE$ are
' the direct equivalent, so every UPPER$/LOWER$ call below was rewritten to
' use them instead. See ../README.md.
require com.bascal.stdlib.ucase
require com.bascal.stdlib.lcase

' Global arrays the new procedures below operate on via `global` --
' moved up from where stage 2 first `dim`s them inline, so every
' procedure that needs one can declare it `global` regardless of where
' in the file it's defined.
dim descrip$(200)
dim items$(200)
dim msg$(2500)
dim indx(303)
dim fraindx(10)
dim itemname$(47)

' Loads ADESCRIP (room descriptions) into descrip$(), indexed 1..dcount
' by file line order. Replaces stage 2's inline LA100-LA102 loading code.
procedure loadAdescrip()
    global descrip$
    dim dcount, line$
    open "ADESCRIP" for INPUT as #1
    while not(EOF(1))
        dcount = dcount + 1
        ' INPUT # into an array element directly isn't supported by the
        ' minimal C backend yet -- only a bare scalar variable is -- so
        ' read into one first, then assign it into the array.
        input #1, line$
        descrip$(dcount) = line$
        print ".";
    end while
    close #1
end procedure

' Loads AITEMS (item-at-location messages) into items$(). Replaces
' stage 2's inline LA110-LA112 loading code.
procedure loadAitems()
    global items$
    dim icount, line$
    open "AITEMS" for INPUT as #2
    while not(EOF(2))
        icount = icount + 1
        input #2, line$
        items$(icount) = line$
        print ".";
    end while
    close #2
end procedure

' Loads AMESSAGE (the numbered "#N" message table) into msg$(1..mcount).
' Replaces stage 2's inline LA120-LA122 loading code.
procedure loadAmessage()
    global msg$
    global mcount
    dim line$
    mcount = 0
    open "AMESSAGE" for INPUT as #3
    while not(EOF(3))
        mcount = mcount + 1
        input #3, line$
        msg$(mcount) = line$
    end while
    close #3
end procedure

' Scans the in-memory msg$() built by loadAmessage() and records where
' each numbered message starts, so printMessage's `gosub L7620` can jump
' straight there instead of scanning from the top every time. Replaces
' the original AMESSAGE.IDX disk cache and this port's own stage-2
' in-memory equivalent (the old `gosub L12500` target).
procedure buildMessageIndex()
    global msg$
    global mcount
    global indx
    global fraindx
    dim fpos, fracnt, lastfra, b$, z4
    fpos = 0 : fracnt = 0 : lastfra = -1
    while fpos <= mcount
        fpos = fpos + 1
        if fpos > mcount then
            return
        end if
        b$ = msg$(fpos)
        if b$ = "#" then
            return
        end if
        if instr(b$, "#") <> 0 then
            z4 = val(mid$(b$, 2))
            if int(z4) = z4 then
                indx(int(z4)) = fpos
            else
                fracnt = fracnt + 1
                if lastfra <> int(z4) then
                    indx(int(z4)) = fracnt
                    lastfra = int(z4)
                end if
                fraindx(fracnt) = fpos
            end if
        end if
    end while
end procedure

' Loads the short item/object names (DATA 9961 onward) into itemname$(),
' addressed by object code 1-47 instead of the original's computed
' `restore 9960+x` (BASCAL restore targets are fixed labels, not
' expressions).
procedure loadItemNames()
    global itemname$
    dim ianame
    restore L9961
    for ianame = 1 to 47
        read itemname$(ianame)
    end for
end procedure

' Reads a line and returns 1 for "yes", 0 for "no" -- reprompting until
' it gets one. Replaces the original's `gosub 9860` + global z0.
function askYesNo%()
    dim a$
    while true
        input a$
        if len(a$) = 0 then
            a$ = " "
        end if
        a$ = lcase$(mid$(a$, 1, 1))
        if a$ = "y" then
            return 1
        elseif a$ = "n" then
            return 0
        end if
        print "Yes or No-";
    end while
    return 0 ' unreachable -- every path through the loop above already
    ' returns; the minimal C backend requires a function's literal last
    ' top-level statement to be `return`, though, since it doesn't try
    ' to prove a loop always returns the way the basic target's own
    ' fallthrough-is-fine model doesn't need to
end function

' Prints the numbered message from AMESSAGE (msg$()/indx()/fraindx(),
' built by buildMessageIndex()). Replaces the original `z59 = N : gosub
' 7620` convention -- every call site is now `printMessage(N)`, passing
' the message number as a real argument instead of setting the global
' z59 first.
procedure printMessage(msgNum%)
    global indx
    global fraindx
    global msg$
    dim xtmp, mpos, b1$
    xtmp = indx(msgNum%)
    if msgNum% = 2 or msgNum% = 61 then
        ' Messages 2 and 61 have several interchangeable variants
        ' (see AMESSAGE's "#2.1".."#2.5" entries); pick one at random.
        xtmp = fraindx(xtmp + int(RND(1) * 5))
    end if
    mpos = xtmp
    b1$ = msg$(mpos)
    if mid$(b1$, 1, 1) <> "#" or int(val(mid$(b1$, 2))) <> msgNum% then
        print "NO DESC. # "; msgNum%; " IN FILE AMESSAGE"
        return
    end if
    while true
        mpos = mpos + 1
        b1$ = msg$(mpos)
        if mid$(b1$, 1, 1) = "#" then
            return
        end if
        print b1$
    end while
end procedure

' Prints the room's short (one-line) description and marks it visited.
' Replaces stage 2's L6780 GOSUB target.
procedure shortDescription()
    global l1
    global v
    global descrip$
    dim a$
    a$ = descrip$(l1)
    v(l1) = 1
    print a$
end procedure

' Prints the room's long (multi-paragraph, AMESSAGE-driven) description
' and marks it visited. Message number is normally l1+200, except a
' shared message for the forest (any l1 <= 4) and one for the maze-like
' rooms near the end. Replaces stage 2's L7990 GOSUB target.
procedure longDescription()
    global l1
    global v
    v(l1) = 1
    if l1 <= 4 then
        printMessage(200)
    elseif (l1 > 88 and l1 < 98) or l1 = 99 then
        printMessage(288)
    else
        printMessage(200 + l1)
    end if
end procedure

' Lists the items sitting in the current room, checks whether the golden
' bird's message should play, and then checks the dwarf and pirate
' encounters. Replaces stage 2's L6680 GOSUB target.
procedure describeRoomContents()
    global l1
    global t2
    global s
    global dead
    global items$
    for z1 = 1 to t2
        if s(z1) = l1 then
            print items$(z1)
        end if
    end for
    if s(26) = -1 then
        printMessage(67)
    end if
    ' CHECK FOR DWARF, PIRATE
    checkDwarf()
    if dead <> 1 then
        checkPirate()
        print
    end if
end procedure

' Checks whether the dwarf gives away his axe (first time in a deep
' room), attacks with his knife, or appears for the first time.
' Structurally identical to stage 2's L8560 GOSUB target -- same
' internal labels, `goto L8790` (the old shared exit) just became
' `return`, and the final label+return collapsed into the implicit
' return at the end of the procedure body.
procedure checkDwarf()
    global d3
    global l1
    global s
    if d3 <> 0 then
        checkDwarfAttack()
        return
    end if
    ' SHOULD DWARF GIVE AWAY AXE?
    if l1 < 13 then return
    if RND(1) > 0.05 then return
    ' GIVE AWAY AXE
    printMessage(80)
    s(27) = l1 : d3 = 1
end procedure

' Just the "should the dwarf attack" half of checkDwarf() -- one call
' site (stage 2's L4340, `GOSUB 8650`) jumps directly into the middle of
' the original subroutine to run only this part, deliberately skipping
' the axe-giving check above. Structurally identical to stage 2's
' L8640-L8790 span.
procedure checkDwarfAttack()
    global l1
    global s
    global t
    global dead
    global KC
    if L1 >= 13 then goto L8660
    s(35) = 0 : return
    L8660: if s(35) <> L1 then goto L8770
    if (l1 <> 60 and l1 <> 61) or t <> 1 then goto L8670
    printMessage(299)
    s(35) = 0 : return
    L8670: if RND(1) > 0.5 then return
    ' YES!
    printMessage(32)
    ' DOES THE KNIFE KILL THE PLAYER?
    KC = KC - 0.02
    IF KC >= 0.75 THEN goto L8710
    KC = 0.75
    L8710: if RND(1) <= KC then goto L8750
    ' YES
    PRINT "It gets you!"
    dead = 1
    return
    L8750: PRINT "It misses!"
    return
    L8770: ' SHOULD WE PUT A DWARF HERE?
    if RND(1) >= 0.05 then return
    if (l1 = 60 or l1 = 61) and t = 1 then return
    s(35) = l1
    printMessage(31)
end procedure

' Checks whether the pirate steals the player's valuables (only in
' rooms l1 >= 13). Structurally identical to stage 2's L8800 GOSUB
' target, with `goto L8960` (the old shared exit) replaced by `return`.
procedure checkPirate()
    global l1
    global s
    dim z3, x
    ' FIRST, DOES HE HAVE ANYTHING WORTH STEALING?
    z3 = 0
    if l1 < 13 then return
    for x = 1 to 15
        if s(x) = -1 then
            z3 = z3 + 1
        end if
    end for
    if z3 < int(RND(1) * 4) + 1 then return
    ' SHOULD WE RIP OFF HIS VALUABLES?
    if RND(1) < 0.05 then
        printMessage(33)
        for x = 1 to 15
            if s(x) = -1 then
                s(x) = 100
            end if
        end for
    else
        printMessage(34)
    end if
end procedure

' On entering a room: the forest and maze-like rooms always get their
' long description (it's short enough not to be worth abbreviating);
' every other room gets the long description only the first time it's
' visited, and the short one on repeat visits. Replaces stage 2's L8180
' GOSUB target, itself called from the `on d0+1 gosub` dispatch below.
procedure describeRoomOnEntry()
    global l1
    global v
    if l1 < 5 or (l1 > 88 and l1 < 98) or l1 = 99 then
        longDescription()
    elseif v(l1) = 1 then
        shortDescription()
    else
        longDescription()
    end if
end procedure

' Checks a handful of situational, room-dependent one-off messages (the
' grate, crystal bridge, plugh noise, iron door, troll, bear, and plant
' in the pit) after a room's normal description prints. Each check is
' independent, not a mutually-exclusive chain, matching the original's
' own if/then/else-fallthrough-to-the-next-check structure exactly.
' Replaces stage 2's L7800 GOSUB target.
procedure situationDescriptions()
    global l1
    global g
    global b2
    global d2
    global t
    global b1
    global p1
    ' GRATE
    if l1 = 10 or l1 = 11 then
        printMessage(g + 10)
    end if
    ' CRYSTAL BRIDGE
    if (l1 = 19 or l1 = 20) and b2 = 1 then
        printMessage(14)
    end if
    ' PLUGH NOISE
    if l1 = 26 and RND(1) > 0.3 then
        printMessage(41)
    end if
    ' IRON DOOR
    if l1 = 73 and d2 = 0 then
        printMessage(57)
    end if
    ' TROLL
    if (l1 = 60 or l1 = 61) and t = 1 then
        printMessage(63)
    end if
    ' BEAR
    if l1 = 69 and b1 = 0 then
        printMessage(64)
    end if
    if l1 = 69 and b1 = 1 then
        printMessage(66)
    end if
    ' PLANT IN PIT
    if l1 = 48 or l1 = 50 then
        printMessage(47 + p1)
    end if
end procedure

' Scans the parsed keyword codes k(1..45) for item names the player
' typed: z8 counts how many matched, z3 remembers the first exact-item
' match's own code, and d$/b$ end up holding the last match's display
' name (itemname$'s own text). Replaces stage 2's L8280 GOSUB target --
' by far the most-called of the remaining GOSUB subroutines (8 call
' sites), used by GET/DROP/EAT/DRINK/ATTACK/FEED and others to work out
' which item, if any, the player's command actually named.
procedure findMatchedItems()
    global k
    global z8
    global z3
    global d$
    global b$
    global itemname$
    z8 = 0 : z3 = 0 : d$ = ""
    for z5 = 1 to 45
        if k(z5) <> 0 then
            z8 = z8 + 1
            b$ = itemname$(z5) : d$ = b$
            if k(z5) = 1 and z8 = 1 then
                z3 = z5
            end if
        end if
    end for
    b$ = d$
end procedure

' Finds the first object code (1-47, the same range itemname$() covers)
' the player's command mentioned, and sets d$ to its display name --
' used for messages like "What do you want to do with the LAMP?" where
' the exact item doesn't matter, just naming *something* the player
' typed. Replaces stage 2's L8390 GOSUB target.
procedure findFirstNamedItem()
    global k
    global d$
    global itemname$
    dim x1
    x1 = 0
    for z1 = 1 to 47
        if x1 <> 1 then
            if k(z1) = 1 then
                d$ = itemname$(z1) : x1 = 1
            end if
        end if
    end for
end procedure

' Checks whether the player is carrying item z3 (s(z3) = -1 marks an
' item as carried); sets z5 to 1 if so, 0 (and prints "You don't have
' the <a$>") if not -- a$ is whatever the player's command last named,
' set well before this runs, back in the command-parsing code. Replaces
' stage 2's L8490 GOSUB target.
procedure checkCarryingItem()
    global z3
    global s
    global a$
    global z5
    if s(z3) = -1 then
        z5 = 1
    else
        print "You don't have the "; a$
        z5 = 0
    end if
end procedure

' Recomputes the current score from scratch (treasures deposited/found,
' game milestones reached, rooms visited) into s0, and the rooms-visited
' count into z9. Replaces stage 2's L6510 GOSUB target, called only from
' printScore() below.
procedure computeScore()
    global s0
    global z9
    global g
    global sn
    global d1
    global t
    global b1
    global b2
    global p1
    global d2
    global c
    global v
    global t1
    global o
    global s
    restore L230
    z9 = 0 : s0 = 0
    for z0 = 1 to 15
        read z1
        if z1 <> 0 then
            if v(z1) = 1 then
                s0 = s0 + 4 * o(z0)
            end if
            if s(z0) = 7 then
                s0 = s0 + 4 * o(z0)
            end if
        end if
    end for
    s0 = (g = 1) * 10 + s0 : s0 = (sn = 0) * 20 + s0 : s0 = (d1 = 0) * 30 + s0
    s0 = (t = 0) * 30 + s0 : s0 = (b1 = 2) * 20 + s0 : s0 = (b2 = 1) * 20 + s0
    s0 = (p1 = 2) * 20 + s0 : s0 = (d2 = 1) * 20 + s0 : s0 = (c = 1) * 20 + s0
    for z0 = 1 to t1
        if v(z0) = 1 then
            s0 = s0 + 1 : z9 = z9 + 1
        end if
    end for
end procedure

' Prints the player's current score, exploration percentage, and skill
' title (looked up by score tier from the DATA table at L6470 -- z9's
' own rooms-visited value is only needed for the percentage line above,
' so the tier lookup uses its own local variable rather than reusing
' z9 for a second, unrelated purpose the way stage 2 does). Replaces
' stage 2's L6430 GOSUB target.
procedure printScore()
    global s0
    global z9
    global t1
    dim tier, d$
    computeScore()
    print "Your score is now "; s0
    print "You have explored "; (z9 / t1) * t1; "% of the cave."
    restore L6470
    tier = int((s0 - 1) / 100)
    if tier > 4 then
        tier = 4
    end if
    for z0 = 0 to tier
        read d$
    end for
    print "That makes you a "; d$; " adventurer."
end procedure

' Asks what the player wants to do with the item named by d$ (already
' set by the caller, e.g. via findFirstNamedItem()) -- replaces stage
' 2's L940 GOSUB-like target, reached via GOTO from two different
' places (the "exotic words" and "item but no verb" checks below) that
' both then GOTO L410 themselves afterward, since a procedure can't
' GOTO a top-level label the way the original shared code could.
procedure askWhatToDoWithItem()
    global d$
    print "What do you want to do with the ";d$;"?"
end procedure

' Prints one of four random "I don't understand"-style messages --
' replaces stage 2's L2040 GOSUB-like target (its DATA line, L2070,
' deliberately stays at its original top-level position further down
' rather than moving up here with the rest of this logic -- RESTORE
' can target a label anywhere in the file, and leaving DATA statement
' order completely undisturbed avoids any risk to some other, unrelated
' unrestored sequential READ elsewhere in this 1149-line program).
' Reached via GOTO from every verb handler that couldn't make sense of
' the player's command (the fallback for an unrecognized word entirely,
' and several verb handlers' own "you didn't say what" case). Every
' call site GOTOs L400 right after, same reason as
' askWhatToDoWithItem() above.
procedure printDontUnderstand()
    global b$
    restore L2070
    for x = 1 to int(RND(1)*4)+1
        read b$
    end for
    PRINT b$
end procedure

' Performs the actual room transition, tracking whether the pirate
' (the one item at s(35)) needs to follow the player -- replaces stage
' 2's L1180 GOSUB-like target. Always succeeds; returns 1 (not a plain
' procedure) only so its callers, all of which end in `return
' performMove%(...)`, can share the same "0 = message printed, no
' move; 1 = moved, check pits next" signal the rest of the movement
' cascade below uses.
function performMove%(z2%)
    global l1
    global l2
    global s
    l2 = l1 : l1 = z2%
    if s(35) = l2 then
        s(35) = l1
    end if
    return 1
end function

' The special-room/special-direction checks run after a destination
' room (z2%) has already been found for direction d% -- replaces stage
' 2's L1260 GOSUB-like target (reached either from attemptMove%() below
' after its own dirs() lookup, or directly from a couple of verb
' handlers that search for a valid direction themselves first). Every
' condition here tests a specific, mutually exclusive room number, so
' this is a straightforward `elseif` chain even though the original
' cascaded through a linear sequence of `GOTO`s that would, for some
' rooms, redundantly test conditions that could never be true there --
' see the case-by-case comments below for exactly which original label
' each branch replaces. Returns 0/1 the same way performMove%() does.
function checkSpecialRoomAndMove%(d%, z2%)
    global l1
    global g
    global b2
    global sn
    global t
    global d2
    global s
    global k
    global t2
    dim z3%
    if (l1 = 10 and (d% = 10 or d% = 5)) or (l1 = 11 and (d% = 9 or d% = 3)) then
        ' GRATE (was L1260/L1280) -- IF GRATE IS OPEN (G=0) MOVE HIM
        if g = 1 then
            return performMove%(z2%)
        end if
        printMessage(10)
        return 0
    elseif l1 = 17 and d% = 9 and s(1) = -1 then
        ' CAN'T TAKE NUGGET UPSTAIRS (was L1320)
        printMessage(38)
        return 0
    elseif (l1 = 19 and d% = 7) or (l1 = 20 and d% = 3) then
        ' CRYSTAL BRIDGE AND FISSURE (was L1360)
        if b2 then
            return performMove%(z2%)
        end if
        printMessage(3)
        return 0
    elseif l1 = 22 and d% <> 3 and d% <> 9 then
        ' MT. KING & SNAKE (was L1410)
        if sn = 0 then
            return performMove%(z2%)
        end if
        printMessage(50)
        return 0
    elseif l1 = 57 or l1 = 58 then
        ' Narrow Tunnel (was L1690) -- K(102)/K(106) are this turn's own
        ' "did the player type E"/"...W" keyword flags (see the DATA
        ' table's direction codes 100-109); the carried-item check below
        ' only applies when moving E/W literally, not via some other
        ' direction synonym.
        if k(102) <> 0 or k(106) <> 0 then
            for z3% = 1 to t2
                if z3% <> 10 and s(z3%) = -1 then
                    printMessage(53)
                    return 0
                end if
            end for
        end if
        return performMove%(z2%)
    elseif (l1 = 60 and d% = 2) or (l1 = 61 and d% = 6) then
        ' TROLL (was L1780/L1790) -- t is 0 (no troll met yet), 1 (troll
        ' appeased, gone), or 2 (troll killed).
        select case t+1
            case 1
                return performMove%(z2%)
            case 2
                printMessage(55)
                return 0
            case 3
                printMessage(56)
                printMessage(55)
                t = 1
                return 0
            case 4
                t = 2
                return performMove%(z2%)
        end select
    elseif l1 = 73 and d% = 1 and d2 = 0 then
        ' (was L1860)
        printMessage(57)
        return 0
    elseif l1 = 82 and s(33) = l1 and d% = 1 then
        ' DRAGON (was L1890)
        printMessage(51)
        return 0
    else
        ' Normal, unimpeded move (was L1890's own final `else`).
        return performMove%(z2%)
    end if
end function

' dirs() returned the "pick one of several rooms at random" sentinel
' (255) for the room the player's currently in -- replaces stage 2's
' L1470 GOSUB-like target. 255 is only ever set for rooms 44 (bedquilt)
' and 39 (Witt's End) in AMOVING; anything else falls through to the
' same "you can't go that way" message L1220 itself would print outside
' this sentinel case.
function attemptRandomMove%()
    global l1
    dim z2%, z3%
    if l1 = 44 then
        ' BEDQUILT: 50/50 chance of landing in one of five rooms picked
        ' at random from the DATA table below (was L1470/L1510/L1530).
        if RND(1) > 0.5 then
            restore L1530
            for z3% = 1 to int(RND(1)*5)+1
                read z2%
            end for
            return performMove%(z2%)
        end if
        printMessage(52)
        return 0
    end if
    if l1 = 39 then
        ' WITT'S END: 15% chance of escaping to room 38 (was L1590/L1650).
        if RND(1) < 0.15 then
            return performMove%(38)
        end if
        printMessage(52)
        return 0
    end if
    printMessage(1)
    return 0
end function
L1530: data 33,37,45,92,76

' Looks up the destination room for direction d% and either moves there
' or prints why not -- replaces stage 2's L1070 GOSUB-like target.
' z2=255 is the "pick one of several rooms at random" sentinel (see
' attemptRandomMove%()); 1..254 is a real destination; anything else
' means "you can't go that way." Returns 1 if the move happened (the
' caller should GOTO L9780, which checks for a pit to fall into and
' re-displays the room), 0 if a message was printed instead (the
' caller should GOTO L400) -- a plain GOTO to either, the way the
' original GOSUB-like target used, isn't possible from inside a
' function.
function attemptMove%(d%)
    global l1
    global dirs
    dim z2%
    z2% = dirs(l1, d%)
    if z2% = 255 then
        return attemptRandomMove%()
    end if
    if z2% < 1 or z2% > 254 then
        printMessage(1)
        return 0
    end if
    return checkSpecialRoomAndMove%(d%, z2%)
end function

' ADVENTURE/3000 VERSION 3.2     27 FEB 1979 AT 5:30 PM
' THIS PROGRAM IS RELATIVELY BUG-FREE, BUT ONE STILL MUST
' realize that murphy 'S LAW STILL PREVAILS!
'
' ADVENTURE: PROGRAMMED IN HP/3000 BASIC BY BENJAMIN MOSER
' JAMES MADISON HIGH SCHOOL, VIENNA, VIRGINIA.  THE BASIC LAYOUT OF THE
' GAME WAS CONCEIVED BY DON WOODS & WILLIE CROWTHER, BOTH OF M.I.T.
'
' ADVENTURE WAS PORTED TO THE MACINTOSH PLUS BY THE ELIZABETH AND DAVID HUNTER
' IN MARCH 1998 AND THEN TO PYBASIC FOR THE RASPBERRY RP2040 IN JUNE 2021
' PRINT "Adventure 3.2 on ";date$;" at ";time$
PRINT "Adventure 3.2 for PyBASIC"
open "AMOVING" for INPUT as #4
' ADESCRIP, AITEMS, and AMESSAGE are loaded fully into memory
' below instead of opened here -- this port replaces the original's
' fseek-based random access into them, which BASCAL (and classic
' Microsoft BASIC) has no equivalent for. See ../README.md.
' dirs is an array of possible room directions, it replaces file AMOVING
dim dirs(100,10)
' indx()/fraindx() are now dim'd near the top of the file (see the
' procedure declarations), alongside descrip$()/items$()/msg$()/itemname$().
dim s(99)
dim v(100)
dim k(200)
dim o(15)
PRINT: PRINT "Initializing.";
' initialize
' total rooms, items,and keywords
l1 = int(RND(1)*4)+1 : l2 = l1
g = 0 : b0 = 1 : sn = 1 : d1 = 1 : d2 = 0 : t = 1 : b1 = 0 : b2 = 0 : p1 = 0: dead = 0
l = 0 : c = 0 : d3 = 0 : b3 = 0 : d0 = 2 : t1 = 100 : t2 = 35 : t3 = 149 : r0 = 0 : c0 = 0: c$=""
KC = 1.02
for ii = 1 to 99
    s(ii) = 0 : v(ii) = 0
end for

PRINT ".";:v(100) = 0
indx(1) = -1:indx(2)=-1
'   read in possible movement direction array
for z2 = 1 to 100
    INPUT #4,dx1,dx2,dx3,dx4,dx5,dx6,dx7,dx8,dx9,dx0
    dirs(z2,1)=dx1:dirs(z2,2)=dx2:dirs(z2,3)=dx3:dirs(z2,4)=dx4:dirs(z2,5)=dx5
    dirs(z2,6)=dx6:dirs(z2,7)=dx7:dirs(z2,8)=dx8:dirs(z2,9)=dx9:dirs(z2,10)=dx0
    PRINT ".";
end for

close #4
LA100: ' Load ADESCRIP, AITEMS, AMESSAGE (+ its index), and the short
' item names -- see the procedure declarations near the top of this
' file for what each replaces.
loadAdescrip()
loadAitems()
loadAmessage()
buildMessageIndex()
loadItemNames()
PRINT
restore L230
'    read in locations of items
for z2 = 1 to T2 step 5
    read dx1,dx2,dx3,dx4,dx5:s(z2)=dx1:s(z2+1)=dx2:s(z2+2)=dx3:s(z2+3)=dx4:s(z2+4)=dx5
    PRINT ".";
end for

L230: data 18,25,23,24,21
data 52,0,71,74,58
data 59,69,66,82,100
data 7,49,7,7,7
data 7,12,13,40,38
data 69,0,46,0,0
data 15,60,82,22,250
for ii = 1 to 15 step 5
    read dx1,dx2,dx3,dx4,dx5:o(ii)=dx1:o(ii+1)=dx2:o(ii+2)=dx3:o(ii+3)=dx4:o(ii+4)=dx5
    PRINT ".";
end for

data 1,2,2,2,2
data 3,4,3,3,2
data 5,3,2,3,3
' ASK IF HE WANTS DIRECTIONS
printMessage(301)
z0 = askYesNo%()
if z0 then goto L267 else goto L320
L267: printMessage(302)
'    command INPUT routine
L300: ' PRINT room, items
' If it's dark don't let him see anything
L320: if l1 < 13 or l1 = 58 then goto L350
if l = 1 and (s(18) = l1 or s(18) = -1) then goto L350
printMessage(45)
goto L400
L350: select case d0
    case 0
        shortDescription()
    case 1
        longDescription()
    case else
        describeRoomOnEntry()
end select
v(l1) = 1
describeRoomContents()
if dead = 1 then goto L9540
situationDescriptions()
' INPUT LOOP --- MULTIPLE COMMAANDS, REMOVE JUNK CHARACTERS
L400: if len(c$) > 0 then goto L500
L410: INPUT ">";c$:c$=ucase$(c$)
if c$ = "" then goto L410
PRINT: PRINT
c$ = ucase$(c$)
'    65-90 = A-Z               48-57 = 0-9         46 = . 44 = ,
for x = 1 to len(c$)
    z5 = asc(mid$(c$,x,1))
    if not ((z5 > 64 and z5 < 91) or (z5 > 47 and z5 < 58) or z5 = 44) then
        c$ = mid$(c$,1,x-1)+" "+mid$(c$,x+1)
    end if
end for

if mid$(c$,len(c$),1) = "," then goto L500
c$ = c$+","
L500: z4 = instr(c$,",")
a$ = ucase$(mid$(c$,1,z4-1)) : c$ = mid$(c$,z4+1)
a$ = " "+a$+" "
' search a$ for keywords,puut kwd code into k(x)
' items
L580: data 1,"GOLD",1,"NUGGET",2,"BARS",2,"SILVER",3,"JEWELRY",4,"COINS"
data 5,"DIAMONDS",6,"MING",6,"VASE",7,"PEARL",8,"EGGS",8,"NEST"
data 9,"TRIDENT",10,"EMERALD",11,"PLATINUM",11,"PYRAMID",12,"CHAIN"
data 13,"SPICES",14,"PERSIAN",14,"RUG",15,"TREASURE",15,"CHEST"
data 16,"WATER",17,"OIL",18,"LAMP",18,"LANTERN",19,"KEYS",20,"FOOD",21,"BOTTLE"
data 22,"CAGE",23,"ROD",23,"WAND",24,"CLAM",25,"MAGAZINE",26,"BEAR"
data 27,"AXE",28,"VELVET",28,"PILLOW",29,"SHARDS",30,"OYSTER"
data 31,"BIRD",32,"TROLL",33,"DRAGON",34,"SNAKE",35,"DWARF"
data 36,"ROCK",36,"BOULDER",37,"STAIRS",38,"STEPS",39,"HOUSE",39,"BUILDING"
data 40,"GRATE",41,"STREAM",42,"ROOM",43,"BRIDGE",44,"PIT",45,"VOLCANO"
data 46,"ROAD",47,"ALL",47,"EVERYTHING"
' DIRECTIONS
data 100,"N",100,"NORTH",101,"NE",101,"NORTHEAST",102,"E",102,"EAST"
data 103,"SE",103,"SOUTHEAST",104,"S",104,"SOUTH",105,"SW",105,"SOUTHWEST"
data 106,"W",106,"WEST",107,"NW",107,"NORTHWEST",108,"U",108,"UP",109,"D",109,"DOWN"
' VERBS
data 110,"PLUGH",111,"XYZZY",112,"PLOVER",113,"CROSS",114,"CLIMB",115,"JUMP"
data 116,"FILL",117,"EMPTY",117,"POUR",118,"LOOK",118,"L",119,"LIGHT",119,"ON",120,"EXTINGUISH"
data 120,"OFF",121,"IN",121,"ENTER",122,"LEAVE",122,"OUT",123,"INVENTORY",123,"I"
data 124,"GET",124,"CATCH",124,"TAKE",125,"DROP",125,"DUMP",126,"THROW",127,"ATTACK"
data 127,"KILL",128,"FEED",129,"WATER",130,"LOCK",131,"UNLOCK"
data 132,"FREE",132,"RELEASE",133,"WAVE",134,"OPEN",135,"CLOSE"
data 136,"OIL",137,"EAT",138,"DRINK",139,"FEE FIE FOE FOO"
data 140,"SHORT",141,"LONG",142,"BRIEF",143,"QUIT",143,"STOP",143,"END"
data 144,"SCORE",145,"SAVE",146,"LOAD",147,"READ",147,"EXAMINE"
data 148,"YES",148,"Y",149,"BUG",150,"*"
restore L580
for i = 1 to 200
    k(i) = 0
end for

z1 = 0 : z3 = 0
' T3=TOTAL NUMBER OF KEYWORDS
while z1 <= t3
    read z1,b$
    b$ = " "+b$+" "
    ' IF KEYWORD WAS FOUND, NOTE THIS IN K(Z1)
    if instr(a$,b$) <> 0 then
        k(z1) = 1
    end if
end while
' EXOTIC WORDS
for x = 36 to 46
    if k(x) = 1 then
        findFirstNamedItem()
        askWhatToDoWithItem()
        goto L410
    end if
end for
for x = 110 to t3
    if k(x) = 1 then
        goto L1950
    end if
end for
' THEN IT'S A DIRECTION
for d = 1 to 10
    if k(d+99) = 1 then
        if attemptMove%(d) then
            goto L9780
        else
            goto L400
        end if
    end if
end for
' COMMAND NOT A DIRECTION
goto L1950
' OTHER COMMANDS
' L1950 is a shared entry point jumped to directly from the exotic-
' words/direction resolution above (its own initial `z1 = 100` folded
' into the `for` below), so its label stays put even though the loop
' body is now structured.
L1950: for z1 = 100 to t3
    if k(z1) = 1 then
        goto L2090
    end if
end for
' ITEM BO NO VERB?
' (was: restore L9961 -- item names now come from itemname$())
for x = 1 to 35
    d$ = itemname$(x)
    if k(x) = 1 then
        askWhatToDoWithItem()
        goto L410
    end if
end for
printDontUnderstand()
goto L400
L2070: data "What?","I don't understand.","I can't understand that.","I don't know that word."
L2090: z1 = z1-109
select case z1
    case 1
        ' *** PLUGH ***
        IF L1<>7 THEN goto L2170
        IF S(35)=L1 THEN S(35)=0
        Z2=26
        performMove%(Z2)
        goto L9780
        L2170: IF L1<>26 THEN goto L2200
        Z2=7
        performMove%(Z2)
        goto L9780
        L2200: printMessage(2)
        GOTO L400
    case 2
        ' *** XYZZY ***
        IF L1<>7 THEN goto L2270
        IF S(35)=L1 THEN S(35)=0
        Z2=13
        performMove%(Z2)
        goto L9780
        L2270: IF L1<>13 THEN goto L2200
        Z2=7
        performMove%(Z2)
        goto L9780
    case 3
        ' *** PLOVER *** (CAN'T BRING EMERALD WITH HIM)
        IF L1>26 THEN goto L2360
        IF S(35)<>L1 THEN goto L2330
        S(35) = 0
        L2330: IF S(10)<>-1 THEN goto L2340
        S(10) = L1
        L2340: Z2 = 58
        performMove%(Z2)
        goto L9780
        L2360: IF L1<>58 THEN goto L2200
        Z2=26
        performMove%(Z2)
        goto L9780
    case 4
        ' *** CROSS ***
        IF L1<>19 THEN goto L2470
        IF B2<>0 THEN goto L2440
        L2420: printMessage(3)
        GOTO L400
        L2440: D=7
        ' JUST GIVE NEW DIRECTION, USE MOVE ROUTINE
        if attemptMove%(D) then
            goto L9780
        else
            goto L400
        end if
        L2470: IF L1<>20 THEN goto L2510
        IF B2=0 THEN goto L2420
        D=3
        if attemptMove%(D) then
            goto L9780
        else
            goto L400
        end if
        L2510: IF L1<>60 THEN goto L2540
        D=2
        if attemptMove%(D) then
            goto L9780
        else
            goto L400
        end if
        L2540: IF L1<>61 THEN goto L2200
        D=6
        if attemptMove%(D) then
            goto L9780
        else
            goto L400
        end if
    case 5
        ' *** CLIMB ***
        IF L1<>50 THEN goto L2200
        ' CAN HE CLIMB BEANSTALK?
        IF P1<2 THEN goto L2200
        ' YES
        Z2=70
        performMove%(Z2)
        goto L9780
    case 6
        ' *** JUMP *** STRICTLY SUICIDAL
        IF L1<>16 AND L1<>19 AND L1<>20 AND L1<>27 THEN goto L2200
        printMessage(4)
        GOTO L9550
    case 7
        ' FILL
        IF S(21)=-1 THEN goto L2730
        L2700: B$="bottle"
        L2710: PRINT "You don't have the ";b$
        goto L410
        L2730: IF B0=0 THEN goto L2760
        printMessage(5)
        GOTO L410
        L2760: IF L1<>7 AND L1<>8 AND L1<>9 AND L1<>35 AND L1<>74 AND L1<>81 THEN goto L2790
        B0=1:S(16)=-1
        GOTO L2840
        L2790: IF L1=49 THEN goto L2830
        B$="oil"
        L2810: PRINT "I see no ";B$;" here."
        GOTO L400
        L2830: B0=2:S(17)=-1
        L2840: PRINT "The bottle is now filled."
        GOTO L400
    case 8
        ' *** EMPTY ***
        IF S(21)=-1 THEN goto L2890
        GOTO L2700
        L2890: ' EMPTY BOTTLE (ASSUMED FULL)
        S(B0+15)=0:B0=0
        PRINT "Emptied"
        GOTO L400
    case 9
        ' *** LOOK ***
        L2940: if l1 < 13 or l1 = 58 then goto L2970
        if l = 1 and (s(18) = l1 or s(18) = -1) then goto L2970
        printMessage(45)
        goto L400
        L2970: longDescription() ' (was: gosub 8050 -- jumped past the old
        ' subroutine's own `v(l1)=1` to avoid redundantly re-marking the room
        ' visited; by the time LOOK is typeable the room's already been entered
        ' via describeRoomOnEntry(), which already sets v(l1)=1, so calling the
        ' full longDescription() here just re-does that no-op assignment)
        describeRoomContents()
        goto L400
    case 10
        ' *** LIGHT ***
        if s(18) = -1 then goto L3040
        L3020: b$ = "lamp"
        goto L2710
        L3040: l = 1
        b$ = "on"
        L3060: PRINT "The lamp is now ";b$
        goto L2940
    case 11
        ' *** OFF (EXTINGUSIH) ***
        IF S(18)=-1 THEN goto L3110
        GOTO L3020
        L3110: L=0:B$="off"
        GOTO L3060
    case 12
        ' *** ENTER ***
        IF L1<>6 THEN goto L3180
        ' TO HOUSE
        D=3
        if attemptMove%(D) then
            goto L9780
        else
            goto L400
        end if
        L3180: IF L1<>68 THEN goto L3240
        ' TO BARREN ROOM
        D=3
        if attemptMove%(D) then
            goto L9780
        else
            goto L400
        end if
        L3240: D = 10
        LW3240: if D < 1 then goto L3270
        Z2 = DIRS(L1,D)
        IF Z2>0 AND Z2<101 THEN
            if checkSpecialRoomAndMove%(D, Z2) then
                goto L9780
            else
                goto L400
            end if
        end if
        D = D + (-1)
        goto LW3240
        L3270:
        GOTO L2200
    case 13
        ' ** LEAVE ***
        IF L1<>7 THEN goto L3340
        ' LEAVE HOUSE
        D=7
        if attemptMove%(D) then
            goto L9780
        else
            goto L400
        end if
        L3340: IF L1<>69 THEN goto L3400
        ' LEAVE BARREN ROOM
        D = 7
        if attemptMove%(D) then
            goto L9780
        else
            goto L400
        end if
        L3400: D = 1
        LW3400: if D > 10 then goto L3430
        Z2 = DIRS(L1,D)
        IF Z2>0 AND Z2<101 THEN
            if checkSpecialRoomAndMove%(D, Z2) then
                goto L9780
            else
                goto L400
            end if
        end if
        D = D + (1)
        goto LW3400
        L3430:
        GOTO L2200
    case 14
        ' *** INVENTORY ***
        Z0=0
        PRINT "You are carrying:";
        FOR X=1 TO T2
            IF S(X)=-1 THEN
                b$ = itemname$(x)
                PRINT B$
                Z0 = Z0 + 1
            end if
        end for

        IF Z0=0 THEN goto L3551 else goto L3560
        L3551: PRINT "nothing."
        L3560: PRINT
        GOTO L400
    case 15
        ' *** GET ***
        if k(47) = 1 then goto L3680
        findMatchedItems()
        if z8 > 0 then goto L3680
        PRINT "Get what?"
        printDontUnderstand()
        goto L400
        L3680: z3 = 1
        LW3680: if z3 > t2 then goto L3900
        if k(47) = 1 then goto L3730
        if k(z3) = 0 then goto LCONT3680
        L3730: if s(z3) <> l1 then goto L3750
        if s(z3) = l1 then goto L3790
        L3750: if k(47) = 1 then goto LCONT3680
        a$ = itemname$(z3):PRINT a$;" not here."
        goto LCONT3680
        ' MUST CHECK NOW FOR LEGALITY OF TAKING ITEM
        L3790: z8 = 0
        for x = 1 to t2
            if s(x) = -1 then
                z8 = z8+1
            end if
        end for

        if z8 < 7 then goto L3870
        ' CARRYING TOO MUCH
        printMessage(54)
        goto L410
        L3870: goto L6880
        L3880: s(z3) = -1
        L3890: a$ = itemname$(z3):PRINT a$;":taken."
        LCONT3680: z3 = z3 + (1)
        goto LW3680
        L3900:
        goto L400
    case 16
        ' *** DROP ***
        if k(47) = 1 then goto L4000
        findMatchedItems()
        IF Z8>0 THEN goto L4000
        PRINT "Drop what?"
        printDontUnderstand()
        goto L400
        L4000: Z3 = 1
        LW4000: if Z3 > T2 then goto L4140
        IF K(47)=1 THEN goto L4060
        IF K(Z3)<>1 THEN goto LCONT4000
        IF S(Z3)=0 THEN goto LCONT4000
        L4060: IF S(Z3)=-1 THEN goto L4100
        IF K(47)=1 THEN goto LCONT4000
        b$ = itemname$(z3):PRINT "You don't have the ";B$
        GOTO LCONT4000
        L4100: ' STILL NEED TO ELABORATE ON DROP (BIRD IN CAGE, BOTTLE)
        GOTO L7380
        L4120: b$ = itemname$(z3):PRINT B$;":dropped."
        S(Z3)=L1
        LCONT4000: Z3 = Z3 + (1)
        goto LW4000
        L4140:
        GOTO L400
    case 17
        ' *** THROW ***
        findMatchedItems()
        IF Z8>0 THEN goto L4210
        PRINT "Throw what?"
        printDontUnderstand()
        goto L400
        L4210: IF S(Z3)<>-1 THEN goto L2710
        IF NOT (Z3<16 AND S(32)=L1) THEN goto L4260
        ' THROW TREASURE TO TROLL
        printMessage(27)
        S(Z3)=0:T=3:GOTO L400
        L4260: IF NOT (Z3=27 AND S(32)=L1) THEN goto L4300
        ' TRYING TO BUTCHER TROLL?
        printMessage(26)
        S(27)=L1:GOTO L400
        L4300: IF NOT (Z3=27 AND S(35)=L1) THEN goto L4380
        ' TRYING TO KILL DWARF
        IF RND(1)>0.5 THEN goto L4360
        printMessage(29)
        checkDwarfAttack() ' (was: GOSUB 8650 -- jumped straight into the
        ' attack-check half of the original dwarf subroutine, deliberately
        ' skipping the axe-giving check; see checkDwarfAttack()'s own comment)
        GOTO L4410
        L4360: printMessage(30)
        S(35)=0:GOTO L4410
        L4380: ' NOTHING SPECIAL, JUST DROP ITEM
        IF S(35)<>L1 THEN goto L4400
        checkDwarf() ' (was: GOSUB L8550 -- L8550 was just a comment
        ' immediately before the real dwarf subroutine's first line, L8560, so
        ' this call wanted the full checkDwarf() behavior, axe-check included --
        ' a call site missed when checkDwarf()/checkDwarfAttack() were split out)
        L4400: PRINT "Thrown."
        L4410: S(Z3) = L1
        if dead = 1 then goto L9540
        GOTO L400
    case 18
        ' *** ATTACK ***
        findMatchedItems()
        IF NOT (Z3=33 AND S(Z3)=L1 AND L1=82) THEN goto L4520
        ' HE CAN KILL DRAGON
        printMessage(68)
        GOTO L410
        L4520: IF S(32)<>L1 THEN goto L4560
        ' TRYING TO MUNGE TROLL
        ' (was: Z9=FNA(25):GOTO 400 -- FNA is called with no matching DEF FN
        ' anywhere in the original source; this looks like leftover/broken code
        ' from an earlier version of the upstream port, not something this port
        ' introduced. Z9's assignment here isn't read before it's next assigned
        ' elsewhere, so dropping the call changes nothing observable.)
        GOTO L400
        L4560: IF NOT (Z3=26 OR Z3>30) THEN goto L4600
        ' DANGEROUS TO ATTACK THESE
        printMessage(70)
        GOTO L400
        L4600: ' NOTHING TO ATTACK
        printMessage(71)
        GOTO L400
    case 19
        ' *** FEED ***
        findMatchedItems()
        IF Z3<>35 THEN goto L4690
        ' CAN'T FEED DWARF!
        printMessage(24)
        GOTO L400
        L4690: IF S(20) = -1 THEN goto L4720
        B$ = "FOOD":GOTO L2710
        L4720: IF L1=69 THEN goto L4760
        PRINT "I can't feed it."
        printMessage(23)
        GOTO L400
        L4760: IF S(20)=L1 THEN goto L7600
        B1=1:S(20)=0:printMessage(6)
        GOTO L400
    case 20
        ' *** WATER ***
        IF S(16) = -1 THEN goto L4840
        B$ = "water":GOTO L2710
        L4840: IF L1<>50 THEN goto L2200
        ' GOTO P1+1 OF 4860,4890,4920
        IF P1 = 0 THEN goto L4860
        IF P1 = 1 THEN goto L4890
        IF P1 = 2 THEN goto L4920
        L4860: printMessage(7)
        P1=1:S(16)=0:B0=0:GOTO L400
        L4890: printMessage(8)
        P1=2:S(16)=0:B0=0:GOTO L400
        L4920: printMessage(9)
        P1=0:S(16)=0:B0=0:GOTO L400
    case 21
        ' *** LOCK ***
        L4960: IF L1=10 OR L1=11 THEN goto L4990
        ' NOTHING LOCKABLE
        GOTO L2200
        L4990: IF S(19)=-1 THEN goto L5020
        L5000: B$="keys":goto L2710
        L5020: G=0:printMessage(10)
        GOTO L400
    case 22
        ' *** UNLOCK ***
        L5070: IF S(19)<>-1 THEN goto L5000
        IF L1<>10 AND L1<>11 THEN goto L5120
        G=1:printMessage(11)
        GOTO L400
        L5120: IF L1<>69 THEN goto L2200
        IF B1>0 THEN goto L5160
        printMessage(12)
        goto L400
        L5160: IF C<>0 THEN goto L5170
        C=1:B1=2
        L5170: printMessage(13)
        GOTO L400
    case 23
        ' *** FREE ***
        IF K(31) = 1 THEN goto L5240
        ' CAN'T FREE ANYTHING BUT BIRD
        L5220: printMessage(2)
        GOTO L410
        L5240: IF S(31)<>-1 THEN goto L5220
        S(31) = L1:B3=0
        PRINT "Freed."
        IF L1<>22 THEN goto L5350
        IF SN<>1 THEN goto L400
        B$="snake"
        L5300: PRINT "The little bird attacks the green ";B$;" and"
        IF L1=82 THEN goto L5380
        PRINT "drives it off"
        SN=0:S(34)=0:GOTO L400
        L5350: IF L1<>82 THEN goto L400
        B$="dragon":GOTO L5300
        L5380: PRINT "gets burned to a crisp"
        S(31)=0
        GOTO L400
    case 24
        ' *** WAVE ***
        IF K(23) <> 1 THEN goto L2200
        IF S(23)=-1 THEN goto L5460
        B$="rod":GOTO L2710
        L5460: '  IS HERE NEAR FISSURE
        IF L1<>19 AND L1<>20 THEN goto L2200
        ' yes
        ' GOTO B2+1 OF 5500,5530
        IF B2=0 THEN goto L5500
        IF B2=1 THEN goto L5530
        L5500: printMessage(14)
        B2=1:GOTO L400
        L5530: printMessage(15)
        B2=0:GOTO L400
    case 25
        ' *** OPEN ***
        findMatchedItems()
        IF Z3>0 THEN goto L5610
        PRINT "Open ";
        printDontUnderstand()
        goto L400
        L5610: IF Z3=40 THEN goto L5070
        IF S(Z3)=L1 THEN goto L5650
        L5630: PRINT "I see no ";b$;" here.":goto L400
        L5650: if z3=24 THEN goto L5680
        PRINT "I don't know how to open a ";B$:GOTO L400
        L5680: IF S(9)=-1 THEN goto L5710
        printMessage(16)
        GOTO L400
        L5710: IF S(Z3) = 0 THEN goto L2200
        ' HE'S OPENED CLAM, SO PRINT DESCRIPTION OF THIS
        ' PUT PEARL IN CUL-DE-SAC
        S(7)=43:S(24)=0:S(30)=L1:printMessage(17)
        GOTO L400
    case 26
        ' *** CLOSE *** -- an upstream bug (present already in stage 2/3,
        ' not introduced by this refactor): the dispatch table points CLOSE's
        ' entry at this exact "GOTO L400" line -- the same line OPEN's own
        ' handler ends on above -- rather than at the real CLOSE logic just
        ' below (L5760-L5800), which is consequently dead code, unreachable
        ' from anywhere. Preserved exactly as-is (CLOSE silently does nothing,
        ' matching the original's real behavior) rather than "fixed" to call
        ' the logic that was clearly intended; this is a faithfulness refactor,
        ' not a gameplay bugfix. Split into its own copy of the shared line so
        ' each verb has its own case block below.
        GOTO L400
        ' *** CLOSE *** (dead code -- see the comment above)
        findMatchedItems()
        IF Z3=40 THEN goto L4960
        printMessage(18)
        GOTO L400
    case 27
        ' OIL
        IF K(17)=0 THEN goto L2200
        IF S(17)=-1 THEN goto L5860
        B$="oil":GOTO L5630
        L5860: IF L1<>73 THEN goto L2200
        ' IS DOOR STILL RUSTED
        IF D2=1 THEN goto L2200
        D2=1:S(17)=0:B0=0:printMessage(19)
        GOTO L400
        ' *** EAT ***
    case 28
        IF K(20) = 1 THEN goto L5950
        printMessage(20)
        GOTO L410
        L5950: Z3=20:checkCarryingItem()
        IF Z5=0 THEN goto L410
        printMessage(73)
        S(20)=0:B0=0:GOTO L400
    case 29
        ' *** DRINK ***
        IF K(16) =1 THEN goto L6040
        printMessage(21)
        GOTO L410
        L6040: Z3=16:checkCarryingItem()
        IF Z5=0 THEN goto L410
        printMessage(22)
        S(17)=0:B0=0:GOTO L400
    case 30
        ' *** FEE FIE FOE FOO ***
        IF L1=71 THEN goto L6130
        printMessage(2)
        GOTO L410
        L6130: IF S(8)<>L1 THEN goto L6180
        ' MAKE NEST VANISH
        printMessage(79)
        S(8)=0:GOTO L400
        L6180: ' IF S(8)=0 THEN goto L6110
        S(8)=L1
        ' MAKE NEST RE-APPEAR
        printMessage(81)
        GOTO L400
    case 31
        ' *** SHORT ***
        PRINT "Short descriptions"
        D0=0:GOTO L400
    case 32
        ' *** LONG ***
        PRINT "Long descriptions"
        D0=1:GOTO L400
    case 33
        ' *** BRIEF ***
        PRINT "OK, I'll only describe the room in detail the first time."
        D0=2:GOTO L400
    case 34
        ' *** QUIT ***
        PRINT "Save game";
        Z0 = askYesNo%()
        IF Z0=1 THEN goto L8970
        GOTO L9750
        ' SCORE ***
    case 35
        printScore()
        GOTO L400
        L6430: ' score computation/printing are now computeScore()/
        ' printScore() above -- the DATA line right below is still needed
        ' here (RESTORE targets must be top-level, per issue #149/PR #150).
        L6470: DATA "beginner","novice","experienced","advanced","expert"
        ' list items at location l1
        ' room-contents listing + dwarf/pirate checks are now the
        ' describeRoomContents() procedure above.
        ' Print Short room description
        ' short room description is now the shortDescription() procedure above.
        L6880: ' SPECIAL GETS
        if not (z3 = 24 or z3 = 30 or z3 > 31) then goto L6930
        ' CAN'T GET THESE FOR SOME REASON
        printMessage(61)
        goto L400
        L6930: if not (z3 = 12 and c = 0) then goto L6970
        ' CHAIN
        printMessage(58)
        goto L3900
        L6970: ' BEAR IS HE FED? UNLOCKED?
        if not (z3 = 26 and b1 <> 2) then goto L7010
        printMessage(61)
        goto L3900
        L7010: if not (z3 = 14 and d1 = 1) then goto L7050
        ' DRAGON AND RUG
        printMessage(59)
        goto L3900
        L7050: if not (z3 = 16 or z3 = 17) then goto L7090
        ' OIL AND WATER DO SAME AS FILL
        PRINT "Why not say 'fill'?"
        goto L3900
        L7090: if not (z3 = 22 and b3) then goto L7140
        ' TAKE BIRD SINCE IT'S IN CAGE
        s(31) = -1:PRINT "Bird and ";:goto L3880
        L7140: if z3 <> 31 then goto L7310
        ' GETTING BIRD
        if b3 <> 1 then goto L7210
        ' TAKE CAGE, SINCE BIRD IS IN IT
        PRINT "Cage and ";:s(22) = -1:goto L3880
        L7210: if s(22) = -1 then goto L7240
        b$ = "cage":goto L2810
        L7240: if s(23) = -1 then goto L7280
        ' OK TO TAKE BIRD
        s(31) = -1 : b3 = 1:goto L3890
        L7280: ' ROD SCARES BIRD
        printMessage(37)
        goto L3900
        L7310: ' BOTTLE FULL? IF SO, GET CONTENTS
        if not (z3 = 21 and b0) then goto L7360
        PRINT "Contents and the ";
        s(b0+15) = -1
        L7360: goto L3880
        ' SPECIAL "DROP"
        L7380: IF Z3<>31 THEN goto L7440
        ' BIRD IN CAGE
        L7400: S(31)=L1:S(22)=L1:B3=1
        IF Z3<>31 THEN goto L7420
        PRINT "Cage and ";
        L7420: IF Z3=22 THEN goto L7430
        PRINT "Bird and ";
        L7430: goto L4120
        L7440: if z3=22 and b3=1 then goto L7400
        IF Z3<>21 THEN goto L7520
        ' BOTTLE
        IF B0=0 THEN goto L4120
        ' BOTTLE IS FULL, DO DROP CONTENTS TOO
        PRINT "Contents and ";
        S(15+B0)=L1:GOTO L4120
        L7520: IF NOT (Z3=16 OR Z3=17) THEN goto L7541
        PRINT "Try saying 'empty'":goto L4140
        L7541: IF Z3<>26 OR T<>1 OR (L1<>60 AND L1<>61) THEN goto L7550
        printMessage(28)
        T=0:S(26)=L1:S(32)=0:GOTO L400
        L7550: IF Z3<>6 THEN goto L4120
        IF S(28)=L1 THEN goto L7600
        ' GOODBYE, FRAGILE VASE!
        printMessage(43)
        S(6)=0:S(29)=L1:GOTO L400
        L7600: printMessage(60)
        GOTO L4120
        ' PRINT MESSAGE is now the printMessage() procedure above -- every
        ' `z59 = N : gosub 7620` call site became `printMessage(N)`.
        L7800: ' situation descriptions are now the situationDescriptions()
        ' procedure above.
        ' long room description is now the longDescription() procedure above.
        ' the short-vs-long-on-entry decision is now the describeRoomOnEntry()
        ' procedure above.
        '
        '   fetch first item code in k(1 to t2)
        '   z8=total # of items found in list
        '   z3=item code first found
        L8280: ' find-matched-items/find-first-named-item/check-carrying-item
        ' are now findMatchedItems()/findFirstNamedItem()/checkCarryingItem()
        ' above.
        ' dwarf/pirate checks are now checkDwarf()/checkDwarfAttack()/
        ' checkPirate() above.
    case 36
        L8970: ' *** SAVE GAME ***
        INPUT "What do you want to call the save file? ";A$
        ' `on error goto` (not `try`/`catch`) here on purpose -- see the
        ' README's own note on why, and GitHub issues #61 (on error goto
        ' is permanently unsupported under --target c, by design) & #100
        ' (try/catch's RESUME output isn't accepted by real fbc).
        on error goto L9010
        OPEN A$ FOR OUTPUT AS #5
        on error goto 0
        GOTO L9030
        L9010: PRINT "File ";a$;" not created"
        GOTO L410
        L9030: PRINT #5,T1;",";T2;",";T3;",";L1;",";L2;",";G;",";B0;",";SN;",";D1;",";D2;",";D0;",";T;",";B1;",";B2;",";P1;",";L;",";C;",";D3;",";B3;",";R0;",";KC
        FOR X=1 TO 99
            PRINT #5,S(X);",";V(X)
        end for

        PRINT #5,V(100)
        CLOSE #5
        PRINT "Game saved"
        C0 = 0
        if k(143) = 1 then goto L9750
        GOTO L410
    case 37
        ' *** LOAD OLD GAME ***
        IF C0=0 THEN goto L9120
        PRINT "You already have a loaded game!"
        GOTO L410
        L9120: INPUT "Save file name? ";A$
        on error goto L9150
        OPEN A$ FOR INPUT AS #5
        on error goto 0
        GOTO L9170
        L9150: PRINT "Unable to use file ";A$
        GOTO L410
        L9170: INPUT #5,T1,T2,T3,L1,L2,G,B0,SN,D1,D2,D0,T,B1,B2,P1,L,C,D3,B3,R0,KCX$
        kc = val(kcx$)
        FOR X=1 TO 99
            INPUT #5,SX,VX:s(x)=sx:v(x)=vx
        end for

        INPUT #5,vx:v(100)=vx
        CLOSE #5
        C0=1
        GOTO L300
    case 38
        ' *** READ THE MAGAZINE ***
        findMatchedItems()
        IF Z3=25 THEN goto L9270
        printMessage(74)
        GOTO L410
        L9270: IF S(25)=-1 THEN goto L9300
        B$="magazine"
        GOTO L2710
        L9300: ' OK, LET HIM READ IT
        printMessage(303)
        GOTO L400
    case 39
        ' *** YES (only meaningful in the dragon's lair, room 82) *** --
        ' physically relocated here from inside the ATTACK handler's own body
        ' (stage 2/3 had it at its original numeric position, L4490, sitting
        ' between L4480 and L4520 -- reachable only via the dispatch table,
        ' never by ATTACK's own fall-through, so moving it doesn't change
        ' behavior) so its case can be its own block in the SELECT CASE below.
        if L1 <> 82 then
            printDontUnderstand()
            goto L400
        end if
        printMessage(69)
        S(33)=0:D1=0:GOTO L400
    case 40
        ' *** BUG ***
        A$ = "ADVBUGS.TXT"
        on error goto L9150
        OPEN A$ FOR APPEND AS #5
        on error goto 0
        INPUT "Your name: ";A$
        A$=A$+" "+DATE$
        PRINT #5,A$
        PRINT "Enter your gripe in up to five lines (hit return to quit):"
        Z0 = 1
        LW9430: if Z0 > 5 then goto L9480
        PRINT Z0;
        INPUT A$
        IF A$="" THEN goto L9490
        PRINT #5,A$
        Z0 = Z0 + (1)
        goto LW9430
        L9480:
        L9490: PRINT "Message recorded. Thank you!"
        CLOSE #5
        GOTO L410
    case else
        printDontUnderstand()
        goto L400
end select
L9540: ' REINCARNATE HIM
L9550: R0=R0+1
IF R0=1 THEN goto L9580
IF R0=2 THEN goto L9610
IF R0=3 THEN goto L9740
' ASK HIM IF HE WANTS TO BE REINCARNATED
L9580: printMessage(75)
Z0 = askYesNo%()
GOTO L9630
L9610: printMessage(77)
GOTO L9580
L9630: IF Z0=0 THEN goto L9750
printMessage(76)
' PUT HIM BACK IN HOUSE, REARRANGE HIS STUFF
S(18)=7:L=0:DEAD=0:KC=1.03
FOR X=1 TO T2
    IF S(X)=-1 THEN
        S(X)=L1
    end if
end for

' WE'VE PUT THE LAMP IN HOUSE AND OTHER ITEMS WHERE HE DIED
L1=INT(RND(1)*4)+1:L2=L1
GOTO L320
' THIRD DEATH--END OF GAME
L9740: printMessage(78)
L9750: PRINT "Oh well..."
printScore()
' (ADESCRIP/AITEMS/AMESSAGE are in-memory arrays now -- nothing to close)
STOP
L9780: ' *** PITS ***
if l1 < 13 THEN goto L300
IF l = 1 and (s(18) = -1 or s(18) = l1) then goto L300
' IS HE GOING TO FALL INTO A PIT?
if l1 = 16 or l1 = 17 or l1 = 19 or l1 = 20 or l1 = 25 or l1 = 47 or l1 = 48 or l1 = 59 or l1 = 60 or l1 = 61 or l1 = 75 or l1 = 76 or l1 = 98 then goto L9840
goto L300
' he fell into a pit
L9840: printMessage(44)
goto L9540
' *** SEEK A "YES" OR "NO" is now the askYesNo%() function above.
' ---- SHORT NAMES FOR STUFF ----
L9961: data "large gold nugget"
data "bars of silver"
data "precious jewelry"
data "many coins"
data "several diamonds"
data "fragile ming vase"
data "glistening pearl"
data "nest of golden eggs"
data "jewel-encrusted trident"
data "egg-sized emerald"
data "platinum pyramid"
data "golden chain"
data "rare spices"
data "persian rug"
data "treasure chest"
data "water"
data "oil"
data "brass lamp"
data "keys"
data "food"
data "bottle"
data "wicker cage"
data "3-foot black rod"
data "clam"
data "magazine"
data "bear"
data "axe"
data "velvet pillow"
data "shards of pottery"
data "oyster"
data "bird"
data "troll"
data "dragon"
data "snake"
data "dwarf"
data "rock"
data "stairs"
data "steps"
data "house"
data "grate"
data "stream"
data "room"
data "bridge"
data "pit"
data "volcano"
data "road"
data "everything"
' INITIALIZE MESSAGE INDEX is now the buildMessageIndex() procedure above.

```

</details>

<details class="source-embed" markdown="1">

<summary><code>stage6-refactored-bascal/adventure.bcl</code> -- Stage 6 — the outer game loop</summary>

```bascal

// ADVENTURE/3000 -- Stage 6: structuring the outer game loop that every
// one of the 40 verb handlers `GOTO`s back into -- picks up exactly
// where stage 5 left off, the one thing its own header flagged as not
// attempted. See ../README.md for the port's provenance, staging, and
// exactly how far this refactor has gotten so far (it is NOT a
// complete rewrite).
//
// Started as an exact copy of stage 5. The outer redisplay loop
// (was `L300`/`L320`) and the inner "get one command" loop (was
// `L400`/`L410`) are now a `WHILE TRUE` nested inside another, closing
// over the whole room-display-then-command-dispatch body. This became
// possible only once BASCAL grew a `continue` statement (added
// alongside this stage's own work) -- unqualified, like `exit`,
// resolving to whichever of the two loops actually encloses each call
// site: `continue` for a verb handler that wants another command
// without redisplaying (was `GOTO L400`/`GOTO L410`, `c$=""` first for
// the `L410` case so the next INPUT actually prompts), `exit` for one
// that's done and wants the room redisplayed (was a direct `GOTO
// L300`, or the pit-check's own `GOTO L9780` once no pit was hit).
// `L300`/`L400`/`L410` themselves stay real labels rather than
// disappearing entirely: a handful of stage 5's own scan loops (the
// exotic-word/item-but-no-verb `FOR`s, and the direction-of-movement
// `FOR`) need to jump out two loop levels at once to reach them, which
// `continue`/`exit` can't do (each only ever escapes its own innermost
// loop) -- those sites keep an explicit `GOTO` to the label instead.
//
// The reincarnation/pit-death cascade (was `L9540`-`L9840`) is now two
// procedures: reincarnate() (was `L9540`/`L9550`'s own `R0=R0+1` cascade
// through `L9580`/`L9610`/`L9630`/`L9740`) and
// checkPitsAndReincarnateIfNeeded() (was `L9780`/`L9840`). A third,
// endGame() (was `L9750`'s "Oh well..."/printScore()/STOP), got pulled
// out of reincarnate() once it turned out two verb handlers outside the
// cascade entirely -- QUIT's "don't save" path and SAVE GAME's own
// "also quit" check -- shared that same `GOTO L9750` target. Every
// checkPitsAndReincarnateIfNeeded() call site follows it with `exit`
// (or, for the one inside the direction-of-movement `FOR`, `GOTO L300`
// directly, for the two-loop-levels reason above) to reach the outer
// loop's redisplay, whether or not a pit was actually fallen into.
//
// Verified with `bcc --check`, a real `fbc` build, and smoke tests
// against stage 5 covering movement in all directions, inventory,
// GET/DROP, SAVE/LOAD, QUIT with and without saving, SCORE,
// SHORT/LONG/BRIEF, an exotic word (XYZZY), an item named with no verb,
// JUMP, ATTACK, and CLIMB -- byte-identical output throughout. The
// pit-fall/reincarnation cascade itself (not reachable from the game's
// own start within a short smoke test, since every room a fresh game
// can reach in a few moves is lit) was exercised separately, in a
// scratch copy with the "is this room dark" and "which rooms have
// pits" checks temporarily patched to trigger on an early, reachable
// room: confirmed the first and second deaths' distinct messages,
// reincarnation actually resetting state and respawning, and a "no" to
// "reincarnate you?" correctly ending the game via endGame().
program adventure3000

' The original used PyBASIC's UPPER$/LOWER$, which real BASIC (and BASCAL's
' own basic-target output) doesn't have; BASCAL's stdlib UCASE$/LCASE$ are
' the direct equivalent, so every UPPER$/LOWER$ call below was rewritten to
' use them instead. See ../README.md.
require com.bascal.stdlib.ucase
require com.bascal.stdlib.lcase

' Global arrays the new procedures below operate on via `global` --
' moved up from where stage 2 first `dim`s them inline, so every
' procedure that needs one can declare it `global` regardless of where
' in the file it's defined.
dim descrip$(200)
dim items$(200)
dim msg$(2500)
dim indx(303)
dim fraindx(10)
dim itemname$(47)

' Loads ADESCRIP (room descriptions) into descrip$(), indexed 1..dcount
' by file line order. Replaces stage 2's inline LA100-LA102 loading code.
procedure loadAdescrip()
    global descrip$
    dim dcount, line$
    open "ADESCRIP" for INPUT as #1
    while not(EOF(1))
        dcount = dcount + 1
        ' INPUT # into an array element directly isn't supported by the
        ' minimal C backend yet -- only a bare scalar variable is -- so
        ' read into one first, then assign it into the array.
        input #1, line$
        descrip$(dcount) = line$
        print ".";
    end while
    close #1
end procedure

' Loads AITEMS (item-at-location messages) into items$(). Replaces
' stage 2's inline LA110-LA112 loading code.
procedure loadAitems()
    global items$
    dim icount, line$
    open "AITEMS" for INPUT as #2
    while not(EOF(2))
        icount = icount + 1
        input #2, line$
        items$(icount) = line$
        print ".";
    end while
    close #2
end procedure

' Loads AMESSAGE (the numbered "#N" message table) into msg$(1..mcount).
' Replaces stage 2's inline LA120-LA122 loading code.
procedure loadAmessage()
    global msg$
    global mcount
    dim line$
    mcount = 0
    open "AMESSAGE" for INPUT as #3
    while not(EOF(3))
        mcount = mcount + 1
        input #3, line$
        msg$(mcount) = line$
    end while
    close #3
end procedure

' Scans the in-memory msg$() built by loadAmessage() and records where
' each numbered message starts, so printMessage's `gosub L7620` can jump
' straight there instead of scanning from the top every time. Replaces
' the original AMESSAGE.IDX disk cache and this port's own stage-2
' in-memory equivalent (the old `gosub L12500` target).
procedure buildMessageIndex()
    global msg$
    global mcount
    global indx
    global fraindx
    dim fpos, fracnt, lastfra, b$, z4
    fpos = 0 : fracnt = 0 : lastfra = -1
    while fpos <= mcount
        fpos = fpos + 1
        if fpos > mcount then
            return
        end if
        b$ = msg$(fpos)
        if b$ = "#" then
            return
        end if
        if instr(b$, "#") <> 0 then
            z4 = val(mid$(b$, 2))
            if int(z4) = z4 then
                indx(int(z4)) = fpos
            else
                fracnt = fracnt + 1
                if lastfra <> int(z4) then
                    indx(int(z4)) = fracnt
                    lastfra = int(z4)
                end if
                fraindx(fracnt) = fpos
            end if
        end if
    end while
end procedure

' Loads the short item/object names (DATA 9961 onward) into itemname$(),
' addressed by object code 1-47 instead of the original's computed
' `restore 9960+x` (BASCAL restore targets are fixed labels, not
' expressions).
procedure loadItemNames()
    global itemname$
    dim ianame
    restore L9961
    for ianame = 1 to 47
        read itemname$(ianame)
    end for
end procedure

' Reads a line and returns 1 for "yes", 0 for "no" -- reprompting until
' it gets one. Replaces the original's `gosub 9860` + global z0.
function askYesNo%()
    dim a$
    while true
        input a$
        if len(a$) = 0 then
            a$ = " "
        end if
        a$ = lcase$(mid$(a$, 1, 1))
        if a$ = "y" then
            return 1
        elseif a$ = "n" then
            return 0
        end if
        print "Yes or No-";
    end while
    return 0 ' unreachable -- every path through the loop above already
    ' returns; the minimal C backend requires a function's literal last
    ' top-level statement to be `return`, though, since it doesn't try
    ' to prove a loop always returns the way the basic target's own
    ' fallthrough-is-fine model doesn't need to
end function

' Prints the numbered message from AMESSAGE (msg$()/indx()/fraindx(),
' built by buildMessageIndex()). Replaces the original `z59 = N : gosub
' 7620` convention -- every call site is now `printMessage(N)`, passing
' the message number as a real argument instead of setting the global
' z59 first.
procedure printMessage(msgNum%)
    global indx
    global fraindx
    global msg$
    dim xtmp, mpos, b1$
    xtmp = indx(msgNum%)
    if msgNum% = 2 or msgNum% = 61 then
        ' Messages 2 and 61 have several interchangeable variants
        ' (see AMESSAGE's "#2.1".."#2.5" entries); pick one at random.
        xtmp = fraindx(xtmp + int(RND(1) * 5))
    end if
    mpos = xtmp
    b1$ = msg$(mpos)
    if mid$(b1$, 1, 1) <> "#" or int(val(mid$(b1$, 2))) <> msgNum% then
        print "NO DESC. # "; msgNum%; " IN FILE AMESSAGE"
        return
    end if
    while true
        mpos = mpos + 1
        b1$ = msg$(mpos)
        if mid$(b1$, 1, 1) = "#" then
            return
        end if
        print b1$
    end while
end procedure

' Prints the room's short (one-line) description and marks it visited.
' Replaces stage 2's L6780 GOSUB target.
procedure shortDescription()
    global l1
    global v
    global descrip$
    dim a$
    a$ = descrip$(l1)
    v(l1) = 1
    print a$
end procedure

' Prints the room's long (multi-paragraph, AMESSAGE-driven) description
' and marks it visited. Message number is normally l1+200, except a
' shared message for the forest (any l1 <= 4) and one for the maze-like
' rooms near the end. Replaces stage 2's L7990 GOSUB target.
procedure longDescription()
    global l1
    global v
    v(l1) = 1
    if l1 <= 4 then
        printMessage(200)
    elseif (l1 > 88 and l1 < 98) or l1 = 99 then
        printMessage(288)
    else
        printMessage(200 + l1)
    end if
end procedure

' Lists the items sitting in the current room, checks whether the golden
' bird's message should play, and then checks the dwarf and pirate
' encounters. Replaces stage 2's L6680 GOSUB target.
procedure describeRoomContents()
    global l1
    global t2
    global s
    global dead
    global items$
    for z1 = 1 to t2
        if s(z1) = l1 then
            print items$(z1)
        end if
    end for
    if s(26) = -1 then
        printMessage(67)
    end if
    ' CHECK FOR DWARF, PIRATE
    checkDwarf()
    if dead <> 1 then
        checkPirate()
        print
    end if
end procedure

' Checks whether the dwarf gives away his axe (first time in a deep
' room), attacks with his knife, or appears for the first time.
' Structurally identical to stage 2's L8560 GOSUB target -- same
' internal labels, `goto L8790` (the old shared exit) just became
' `return`, and the final label+return collapsed into the implicit
' return at the end of the procedure body.
procedure checkDwarf()
    global d3
    global l1
    global s
    if d3 <> 0 then
        checkDwarfAttack()
        return
    end if
    ' SHOULD DWARF GIVE AWAY AXE?
    if l1 < 13 then return
    if RND(1) > 0.05 then return
    ' GIVE AWAY AXE
    printMessage(80)
    s(27) = l1 : d3 = 1
end procedure

' Just the "should the dwarf attack" half of checkDwarf() -- one call
' site (stage 2's L4340, `GOSUB 8650`) jumps directly into the middle of
' the original subroutine to run only this part, deliberately skipping
' the axe-giving check above. Structurally identical to stage 2's
' L8640-L8790 span.
procedure checkDwarfAttack()
    global l1
    global s
    global t
    global dead
    global KC
    if L1 >= 13 then goto L8660
    s(35) = 0 : return
    L8660: if s(35) <> L1 then goto L8770
    if (l1 <> 60 and l1 <> 61) or t <> 1 then goto L8670
    printMessage(299)
    s(35) = 0 : return
    L8670: if RND(1) > 0.5 then return
    ' YES!
    printMessage(32)
    ' DOES THE KNIFE KILL THE PLAYER?
    KC = KC - 0.02
    IF KC >= 0.75 THEN goto L8710
    KC = 0.75
    L8710: if RND(1) <= KC then goto L8750
    ' YES
    PRINT "It gets you!"
    dead = 1
    return
    L8750: PRINT "It misses!"
    return
    L8770: ' SHOULD WE PUT A DWARF HERE?
    if RND(1) >= 0.05 then return
    if (l1 = 60 or l1 = 61) and t = 1 then return
    s(35) = l1
    printMessage(31)
end procedure

' Checks whether the pirate steals the player's valuables (only in
' rooms l1 >= 13). Structurally identical to stage 2's L8800 GOSUB
' target, with `goto L8960` (the old shared exit) replaced by `return`.
procedure checkPirate()
    global l1
    global s
    dim z3, x
    ' FIRST, DOES HE HAVE ANYTHING WORTH STEALING?
    z3 = 0
    if l1 < 13 then return
    for x = 1 to 15
        if s(x) = -1 then
            z3 = z3 + 1
        end if
    end for
    if z3 < int(RND(1) * 4) + 1 then return
    ' SHOULD WE RIP OFF HIS VALUABLES?
    if RND(1) < 0.05 then
        printMessage(33)
        for x = 1 to 15
            if s(x) = -1 then
                s(x) = 100
            end if
        end for
    else
        printMessage(34)
    end if
end procedure

' On entering a room: the forest and maze-like rooms always get their
' long description (it's short enough not to be worth abbreviating);
' every other room gets the long description only the first time it's
' visited, and the short one on repeat visits. Replaces stage 2's L8180
' GOSUB target, itself called from the `on d0+1 gosub` dispatch below.
procedure describeRoomOnEntry()
    global l1
    global v
    if l1 < 5 or (l1 > 88 and l1 < 98) or l1 = 99 then
        longDescription()
    elseif v(l1) = 1 then
        shortDescription()
    else
        longDescription()
    end if
end procedure

' Checks a handful of situational, room-dependent one-off messages (the
' grate, crystal bridge, plugh noise, iron door, troll, bear, and plant
' in the pit) after a room's normal description prints. Each check is
' independent, not a mutually-exclusive chain, matching the original's
' own if/then/else-fallthrough-to-the-next-check structure exactly.
' Replaces stage 2's L7800 GOSUB target.
procedure situationDescriptions()
    global l1
    global g
    global b2
    global d2
    global t
    global b1
    global p1
    ' GRATE
    if l1 = 10 or l1 = 11 then
        printMessage(g + 10)
    end if
    ' CRYSTAL BRIDGE
    if (l1 = 19 or l1 = 20) and b2 = 1 then
        printMessage(14)
    end if
    ' PLUGH NOISE
    if l1 = 26 and RND(1) > 0.3 then
        printMessage(41)
    end if
    ' IRON DOOR
    if l1 = 73 and d2 = 0 then
        printMessage(57)
    end if
    ' TROLL
    if (l1 = 60 or l1 = 61) and t = 1 then
        printMessage(63)
    end if
    ' BEAR
    if l1 = 69 and b1 = 0 then
        printMessage(64)
    end if
    if l1 = 69 and b1 = 1 then
        printMessage(66)
    end if
    ' PLANT IN PIT
    if l1 = 48 or l1 = 50 then
        printMessage(47 + p1)
    end if
end procedure

' Scans the parsed keyword codes k(1..45) for item names the player
' typed: z8 counts how many matched, z3 remembers the first exact-item
' match's own code, and d$/b$ end up holding the last match's display
' name (itemname$'s own text). Replaces stage 2's L8280 GOSUB target --
' by far the most-called of the remaining GOSUB subroutines (8 call
' sites), used by GET/DROP/EAT/DRINK/ATTACK/FEED and others to work out
' which item, if any, the player's command actually named.
procedure findMatchedItems()
    global k
    global z8
    global z3
    global d$
    global b$
    global itemname$
    z8 = 0 : z3 = 0 : d$ = ""
    for z5 = 1 to 45
        if k(z5) <> 0 then
            z8 = z8 + 1
            b$ = itemname$(z5) : d$ = b$
            if k(z5) = 1 and z8 = 1 then
                z3 = z5
            end if
        end if
    end for
    b$ = d$
end procedure

' Finds the first object code (1-47, the same range itemname$() covers)
' the player's command mentioned, and sets d$ to its display name --
' used for messages like "What do you want to do with the LAMP?" where
' the exact item doesn't matter, just naming *something* the player
' typed. Replaces stage 2's L8390 GOSUB target.
procedure findFirstNamedItem()
    global k
    global d$
    global itemname$
    dim x1
    x1 = 0
    for z1 = 1 to 47
        if x1 <> 1 then
            if k(z1) = 1 then
                d$ = itemname$(z1) : x1 = 1
            end if
        end if
    end for
end procedure

' Checks whether the player is carrying item z3 (s(z3) = -1 marks an
' item as carried); sets z5 to 1 if so, 0 (and prints "You don't have
' the <a$>") if not -- a$ is whatever the player's command last named,
' set well before this runs, back in the command-parsing code. Replaces
' stage 2's L8490 GOSUB target.
procedure checkCarryingItem()
    global z3
    global s
    global a$
    global z5
    if s(z3) = -1 then
        z5 = 1
    else
        print "You don't have the "; a$
        z5 = 0
    end if
end procedure

' Recomputes the current score from scratch (treasures deposited/found,
' game milestones reached, rooms visited) into s0, and the rooms-visited
' count into z9. Replaces stage 2's L6510 GOSUB target, called only from
' printScore() below.
procedure computeScore()
    global s0
    global z9
    global g
    global sn
    global d1
    global t
    global b1
    global b2
    global p1
    global d2
    global c
    global v
    global t1
    global o
    global s
    restore L230
    z9 = 0 : s0 = 0
    for z0 = 1 to 15
        read z1
        if z1 <> 0 then
            if v(z1) = 1 then
                s0 = s0 + 4 * o(z0)
            end if
            if s(z0) = 7 then
                s0 = s0 + 4 * o(z0)
            end if
        end if
    end for
    s0 = (g = 1) * 10 + s0 : s0 = (sn = 0) * 20 + s0 : s0 = (d1 = 0) * 30 + s0
    s0 = (t = 0) * 30 + s0 : s0 = (b1 = 2) * 20 + s0 : s0 = (b2 = 1) * 20 + s0
    s0 = (p1 = 2) * 20 + s0 : s0 = (d2 = 1) * 20 + s0 : s0 = (c = 1) * 20 + s0
    for z0 = 1 to t1
        if v(z0) = 1 then
            s0 = s0 + 1 : z9 = z9 + 1
        end if
    end for
end procedure

' Prints the player's current score, exploration percentage, and skill
' title (looked up by score tier from the DATA table at L6470 -- z9's
' own rooms-visited value is only needed for the percentage line above,
' so the tier lookup uses its own local variable rather than reusing
' z9 for a second, unrelated purpose the way stage 2 does). Replaces
' stage 2's L6430 GOSUB target.
procedure printScore()
    global s0
    global z9
    global t1
    dim tier, d$
    computeScore()
    print "Your score is now "; s0
    print "You have explored "; (z9 / t1) * t1; "% of the cave."
    restore L6470
    tier = int((s0 - 1) / 100)
    if tier > 4 then
        tier = 4
    end if
    for z0 = 0 to tier
        read d$
    end for
    print "That makes you a "; d$; " adventurer."
end procedure

' Asks what the player wants to do with the item named by d$ (already
' set by the caller, e.g. via findFirstNamedItem()) -- replaces stage
' 2's L940 GOSUB-like target, reached via GOTO from two different
' places (the "exotic words" and "item but no verb" checks below) that
' both then GOTO L410 themselves afterward, since a procedure can't
' GOTO a top-level label the way the original shared code could.
procedure askWhatToDoWithItem()
    global d$
    print "What do you want to do with the ";d$;"?"
end procedure

' Prints one of four random "I don't understand"-style messages --
' replaces stage 2's L2040 GOSUB-like target (its DATA line, L2070,
' deliberately stays at its original top-level position further down
' rather than moving up here with the rest of this logic -- RESTORE
' can target a label anywhere in the file, and leaving DATA statement
' order completely undisturbed avoids any risk to some other, unrelated
' unrestored sequential READ elsewhere in this 1149-line program).
' Reached via GOTO from every verb handler that couldn't make sense of
' the player's command (the fallback for an unrecognized word entirely,
' and several verb handlers' own "you didn't say what" case). Every
' call site GOTOs L400 right after, same reason as
' askWhatToDoWithItem() above.
procedure printDontUnderstand()
    global b$
    restore L2070
    for x = 1 to int(RND(1)*4)+1
        read b$
    end for
    PRINT b$
end procedure

' Ends the game right now -- replaces stage 2's L9750 GOSUB-like
' target, shared by QUIT (with or without a save first), reincarnate()
' below on the third death or a "no" answer, and originally reached by
' plain fallthrough from L9740's own "THIRD DEATH" message besides.
procedure endGame()
    print "Oh well..."
    printScore()
    ' (ADESCRIP/AITEMS/AMESSAGE are in-memory arrays now -- nothing to close)
    stop
end procedure

' Handles a death, however it happened (falling into a pit, or set
' directly elsewhere e.g. by the troll/dragon) -- replaces stage 2's
' L9540/L9550 GOSUB-like target and its own L9580/L9610/L9630/L9740
' cascade. Calls endGame() on the third death or a "no" answer to "do
' you want to be reincarnated"; otherwise resets state and respawns in
' a random forest room, exactly like the original -- including its own
' apparent lack of a cap on r0 (a 4th, 5th, ... death asks the same as
' the first, never reaching endGame() itself; only "answering no" or
' the literal 3rd death do).
procedure reincarnate()
    global r0
    global z0
    global s
    global l
    global dead
    global kc
    global t2
    global l1
    global l2
    r0 = r0 + 1
    if r0 = 3 then
        printMessage(78)
        endGame()
    end if
    if r0 = 2 then
        printMessage(77)
    end if
    ' r0 = 1 reaches here directly; r0 = 2 reaches here after its own
    ' extra message above; r0 >= 4 also reaches here, same as r0 = 1 --
    ' see this procedure's own doc comment.
    printMessage(75)
    z0 = askYesNo%()
    if z0 = 0 then
        endGame()
    end if
    printMessage(76)
    ' PUT HIM BACK IN HOUSE, REARRANGE HIS STUFF
    s(18) = 7 : l = 0 : dead = 0 : kc = 1.03
    for x = 1 to t2
        if s(x) = -1 then
            s(x) = l1
        end if
    end for
    ' WE'VE PUT THE LAMP IN HOUSE AND OTHER ITEMS WHERE HE DIED
    l1 = int(RND(1)*4)+1 : l2 = l1
end procedure

' Checks whether the room just moved into is dark and has a pit to fall
' into -- replaces stage 2's L9780/L9840 GOSUB-like target. Every call
' site follows this with `exit`, breaking back out to the outer game
' loop's own redisplay, whether or not reincarnate() actually ran (a
' safe room needs exactly the same "go redisplay" outcome as a
' fallen-into-and-recovered-from pit).
procedure checkPitsAndReincarnateIfNeeded()
    global l1
    global l
    global s
    if l1 < 13 or (l = 1 and (s(18) = -1 or s(18) = l1)) then
        return
    end if
    ' IS HE GOING TO FALL INTO A PIT?
    if l1 = 16 or l1 = 17 or l1 = 19 or l1 = 20 or l1 = 25 or l1 = 47 or l1 = 48 or l1 = 59 or l1 = 60 or l1 = 61 or l1 = 75 or l1 = 76 or l1 = 98 then
        printMessage(44)
        reincarnate()
    end if
end procedure

' Performs the actual room transition, tracking whether the pirate
' (the one item at s(35)) needs to follow the player -- replaces stage
' 2's L1180 GOSUB-like target. Always succeeds; returns 1 (not a plain
' procedure) only so its callers, all of which end in `return
' performMove%(...)`, can share the same "0 = message printed, no
' move; 1 = moved, check pits next" signal the rest of the movement
' cascade below uses.
function performMove%(z2%)
    global l1
    global l2
    global s
    l2 = l1 : l1 = z2%
    if s(35) = l2 then
        s(35) = l1
    end if
    return 1
end function

' The special-room/special-direction checks run after a destination
' room (z2%) has already been found for direction d% -- replaces stage
' 2's L1260 GOSUB-like target (reached either from attemptMove%() below
' after its own dirs() lookup, or directly from a couple of verb
' handlers that search for a valid direction themselves first). Every
' condition here tests a specific, mutually exclusive room number, so
' this is a straightforward `elseif` chain even though the original
' cascaded through a linear sequence of `GOTO`s that would, for some
' rooms, redundantly test conditions that could never be true there --
' see the case-by-case comments below for exactly which original label
' each branch replaces. Returns 0/1 the same way performMove%() does.
function checkSpecialRoomAndMove%(d%, z2%)
    global l1
    global g
    global b2
    global sn
    global t
    global d2
    global s
    global k
    global t2
    dim z3%
    if (l1 = 10 and (d% = 10 or d% = 5)) or (l1 = 11 and (d% = 9 or d% = 3)) then
        ' GRATE (was L1260/L1280) -- IF GRATE IS OPEN (G=0) MOVE HIM
        if g = 1 then
            return performMove%(z2%)
        end if
        printMessage(10)
        return 0
    elseif l1 = 17 and d% = 9 and s(1) = -1 then
        ' CAN'T TAKE NUGGET UPSTAIRS (was L1320)
        printMessage(38)
        return 0
    elseif (l1 = 19 and d% = 7) or (l1 = 20 and d% = 3) then
        ' CRYSTAL BRIDGE AND FISSURE (was L1360)
        if b2 then
            return performMove%(z2%)
        end if
        printMessage(3)
        return 0
    elseif l1 = 22 and d% <> 3 and d% <> 9 then
        ' MT. KING & SNAKE (was L1410)
        if sn = 0 then
            return performMove%(z2%)
        end if
        printMessage(50)
        return 0
    elseif l1 = 57 or l1 = 58 then
        ' Narrow Tunnel (was L1690) -- K(102)/K(106) are this turn's own
        ' "did the player type E"/"...W" keyword flags (see the DATA
        ' table's direction codes 100-109); the carried-item check below
        ' only applies when moving E/W literally, not via some other
        ' direction synonym.
        if k(102) <> 0 or k(106) <> 0 then
            for z3% = 1 to t2
                if z3% <> 10 and s(z3%) = -1 then
                    printMessage(53)
                    return 0
                end if
            end for
        end if
        return performMove%(z2%)
    elseif (l1 = 60 and d% = 2) or (l1 = 61 and d% = 6) then
        ' TROLL (was L1780/L1790) -- t is 0 (no troll met yet), 1 (troll
        ' appeased, gone), or 2 (troll killed).
        select case t+1
            case 1
                return performMove%(z2%)
            case 2
                printMessage(55)
                return 0
            case 3
                printMessage(56)
                printMessage(55)
                t = 1
                return 0
            case 4
                t = 2
                return performMove%(z2%)
        end select
    elseif l1 = 73 and d% = 1 and d2 = 0 then
        ' (was L1860)
        printMessage(57)
        return 0
    elseif l1 = 82 and s(33) = l1 and d% = 1 then
        ' DRAGON (was L1890)
        printMessage(51)
        return 0
    else
        ' Normal, unimpeded move (was L1890's own final `else`).
        return performMove%(z2%)
    end if
end function

' dirs() returned the "pick one of several rooms at random" sentinel
' (255) for the room the player's currently in -- replaces stage 2's
' L1470 GOSUB-like target. 255 is only ever set for rooms 44 (bedquilt)
' and 39 (Witt's End) in AMOVING; anything else falls through to the
' same "you can't go that way" message L1220 itself would print outside
' this sentinel case.
function attemptRandomMove%()
    global l1
    dim z2%, z3%
    if l1 = 44 then
        ' BEDQUILT: 50/50 chance of landing in one of five rooms picked
        ' at random from the DATA table below (was L1470/L1510/L1530).
        if RND(1) > 0.5 then
            restore L1530
            for z3% = 1 to int(RND(1)*5)+1
                read z2%
            end for
            return performMove%(z2%)
        end if
        printMessage(52)
        return 0
    end if
    if l1 = 39 then
        ' WITT'S END: 15% chance of escaping to room 38 (was L1590/L1650).
        if RND(1) < 0.15 then
            return performMove%(38)
        end if
        printMessage(52)
        return 0
    end if
    printMessage(1)
    return 0
end function
L1530: data 33,37,45,92,76

' Looks up the destination room for direction d% and either moves there
' or prints why not -- replaces stage 2's L1070 GOSUB-like target.
' z2=255 is the "pick one of several rooms at random" sentinel (see
' attemptRandomMove%()); 1..254 is a real destination; anything else
' means "you can't go that way." Returns 1 if the move happened (the
' caller should GOTO L9780, which checks for a pit to fall into and
' re-displays the room), 0 if a message was printed instead (the
' caller should GOTO L400) -- a plain GOTO to either, the way the
' original GOSUB-like target used, isn't possible from inside a
' function.
function attemptMove%(d%)
    global l1
    global dirs
    dim z2%
    z2% = dirs(l1, d%)
    if z2% = 255 then
        return attemptRandomMove%()
    end if
    if z2% < 1 or z2% > 254 then
        printMessage(1)
        return 0
    end if
    return checkSpecialRoomAndMove%(d%, z2%)
end function

' ADVENTURE/3000 VERSION 3.2     27 FEB 1979 AT 5:30 PM
' THIS PROGRAM IS RELATIVELY BUG-FREE, BUT ONE STILL MUST
' realize that murphy 'S LAW STILL PREVAILS!
'
' ADVENTURE: PROGRAMMED IN HP/3000 BASIC BY BENJAMIN MOSER
' JAMES MADISON HIGH SCHOOL, VIENNA, VIRGINIA.  THE BASIC LAYOUT OF THE
' GAME WAS CONCEIVED BY DON WOODS & WILLIE CROWTHER, BOTH OF M.I.T.
'
' ADVENTURE WAS PORTED TO THE MACINTOSH PLUS BY THE ELIZABETH AND DAVID HUNTER
' IN MARCH 1998 AND THEN TO PYBASIC FOR THE RASPBERRY RP2040 IN JUNE 2021
' PRINT "Adventure 3.2 on ";date$;" at ";time$
PRINT "Adventure 3.2 for PyBASIC"
open "AMOVING" for INPUT as #4
' ADESCRIP, AITEMS, and AMESSAGE are loaded fully into memory
' below instead of opened here -- this port replaces the original's
' fseek-based random access into them, which BASCAL (and classic
' Microsoft BASIC) has no equivalent for. See ../README.md.
' dirs is an array of possible room directions, it replaces file AMOVING
dim dirs(100,10)
' indx()/fraindx() are now dim'd near the top of the file (see the
' procedure declarations), alongside descrip$()/items$()/msg$()/itemname$().
dim s(99)
dim v(100)
dim k(200)
dim o(15)
PRINT: PRINT "Initializing.";
' initialize
' total rooms, items,and keywords
l1 = int(RND(1)*4)+1 : l2 = l1
g = 0 : b0 = 1 : sn = 1 : d1 = 1 : d2 = 0 : t = 1 : b1 = 0 : b2 = 0 : p1 = 0: dead = 0
l = 0 : c = 0 : d3 = 0 : b3 = 0 : d0 = 2 : t1 = 100 : t2 = 35 : t3 = 149 : r0 = 0 : c0 = 0: c$=""
KC = 1.02
for ii = 1 to 99
    s(ii) = 0 : v(ii) = 0
end for

PRINT ".";:v(100) = 0
indx(1) = -1:indx(2)=-1
'   read in possible movement direction array
for z2 = 1 to 100
    INPUT #4,dx1,dx2,dx3,dx4,dx5,dx6,dx7,dx8,dx9,dx0
    dirs(z2,1)=dx1:dirs(z2,2)=dx2:dirs(z2,3)=dx3:dirs(z2,4)=dx4:dirs(z2,5)=dx5
    dirs(z2,6)=dx6:dirs(z2,7)=dx7:dirs(z2,8)=dx8:dirs(z2,9)=dx9:dirs(z2,10)=dx0
    PRINT ".";
end for

close #4
LA100: ' Load ADESCRIP, AITEMS, AMESSAGE (+ its index), and the short
' item names -- see the procedure declarations near the top of this
' file for what each replaces.
loadAdescrip()
loadAitems()
loadAmessage()
buildMessageIndex()
loadItemNames()
PRINT
restore L230
'    read in locations of items
for z2 = 1 to T2 step 5
    read dx1,dx2,dx3,dx4,dx5:s(z2)=dx1:s(z2+1)=dx2:s(z2+2)=dx3:s(z2+3)=dx4:s(z2+4)=dx5
    PRINT ".";
end for

L230: data 18,25,23,24,21
data 52,0,71,74,58
data 59,69,66,82,100
data 7,49,7,7,7
data 7,12,13,40,38
data 69,0,46,0,0
data 15,60,82,22,250
for ii = 1 to 15 step 5
    read dx1,dx2,dx3,dx4,dx5:o(ii)=dx1:o(ii+1)=dx2:o(ii+2)=dx3:o(ii+3)=dx4:o(ii+4)=dx5
    PRINT ".";
end for

data 1,2,2,2,2
data 3,4,3,3,2
data 5,3,2,3,3
' ASK IF HE WANTS DIRECTIONS
printMessage(301)
z0 = askYesNo%()
if z0 then printMessage(302)
'    command INPUT routine
' outer loop: (re)display the room, then run the command loop below until
' something (a move, a pit, death) needs the room redisplayed -- replaces
' every stage 2 "GOTO L300"/"GOTO L320" (the two landed on the same next
' statement, since L300 was just a comment immediately before L320).
while true
    ' L300 stays a real label -- not every jump back to full redisplay is a
    ' single loop level away (the direction-scan `for` below is one), and
    ' `exit` only ever escapes its own innermost loop.
    L300: ' PRINT room, items
    ' If it's dark don't let him see anything
    if l1 < 13 or l1 = 58 or (l = 1 and (s(18) = l1 or s(18) = -1)) then
        select case d0
            case 0
                shortDescription()
            case 1
                longDescription()
            case else
                describeRoomOnEntry()
        end select
        v(l1) = 1
        describeRoomContents()
        if dead = 1 then reincarnate() : continue
        situationDescriptions()
    else
        printMessage(45)
    end if
    ' inner loop: get/parse/dispatch one command -- replaces every stage 2
    ' "GOTO L400"/"GOTO L410" (a verb handler's `continue` here means "get
    ' another command without redisplaying"; `exit` below means "done with
    ' commands, redisplay the room").
    ' INPUT LOOP --- MULTIPLE COMMAANDS, REMOVE JUNK CHARACTERS
    ' L400/L410 stay real labels for the same reason as L300 above -- a
    ' few of the command-parsing scan loops below need to jump here from
    ' two loop levels deep, where `continue` can't reach.
    while true
        L400: if len(c$) > 0 then goto L500
        L410: INPUT ">";c$:c$=ucase$(c$)
        if c$ = "" then c$="" : continue
        PRINT: PRINT
        c$ = ucase$(c$)
        '    65-90 = A-Z               48-57 = 0-9         46 = . 44 = ,
        for x = 1 to len(c$)
            z5 = asc(mid$(c$,x,1))
            if not ((z5 > 64 and z5 < 91) or (z5 > 47 and z5 < 58) or z5 = 44) then
                c$ = mid$(c$,1,x-1)+" "+mid$(c$,x+1)
            end if
        end for

        if mid$(c$,len(c$),1) = "," then goto L500
        c$ = c$+","
        L500: z4 = instr(c$,",")
        a$ = ucase$(mid$(c$,1,z4-1)) : c$ = mid$(c$,z4+1)
        a$ = " "+a$+" "
        ' search a$ for keywords,puut kwd code into k(x)
        ' items
        L580: data 1,"GOLD",1,"NUGGET",2,"BARS",2,"SILVER",3,"JEWELRY",4,"COINS"
        data 5,"DIAMONDS",6,"MING",6,"VASE",7,"PEARL",8,"EGGS",8,"NEST"
        data 9,"TRIDENT",10,"EMERALD",11,"PLATINUM",11,"PYRAMID",12,"CHAIN"
        data 13,"SPICES",14,"PERSIAN",14,"RUG",15,"TREASURE",15,"CHEST"
        data 16,"WATER",17,"OIL",18,"LAMP",18,"LANTERN",19,"KEYS",20,"FOOD",21,"BOTTLE"
        data 22,"CAGE",23,"ROD",23,"WAND",24,"CLAM",25,"MAGAZINE",26,"BEAR"
        data 27,"AXE",28,"VELVET",28,"PILLOW",29,"SHARDS",30,"OYSTER"
        data 31,"BIRD",32,"TROLL",33,"DRAGON",34,"SNAKE",35,"DWARF"
        data 36,"ROCK",36,"BOULDER",37,"STAIRS",38,"STEPS",39,"HOUSE",39,"BUILDING"
        data 40,"GRATE",41,"STREAM",42,"ROOM",43,"BRIDGE",44,"PIT",45,"VOLCANO"
        data 46,"ROAD",47,"ALL",47,"EVERYTHING"
        ' DIRECTIONS
        data 100,"N",100,"NORTH",101,"NE",101,"NORTHEAST",102,"E",102,"EAST"
        data 103,"SE",103,"SOUTHEAST",104,"S",104,"SOUTH",105,"SW",105,"SOUTHWEST"
        data 106,"W",106,"WEST",107,"NW",107,"NORTHWEST",108,"U",108,"UP",109,"D",109,"DOWN"
        ' VERBS
        data 110,"PLUGH",111,"XYZZY",112,"PLOVER",113,"CROSS",114,"CLIMB",115,"JUMP"
        data 116,"FILL",117,"EMPTY",117,"POUR",118,"LOOK",118,"L",119,"LIGHT",119,"ON",120,"EXTINGUISH"
        data 120,"OFF",121,"IN",121,"ENTER",122,"LEAVE",122,"OUT",123,"INVENTORY",123,"I"
        data 124,"GET",124,"CATCH",124,"TAKE",125,"DROP",125,"DUMP",126,"THROW",127,"ATTACK"
        data 127,"KILL",128,"FEED",129,"WATER",130,"LOCK",131,"UNLOCK"
        data 132,"FREE",132,"RELEASE",133,"WAVE",134,"OPEN",135,"CLOSE"
        data 136,"OIL",137,"EAT",138,"DRINK",139,"FEE FIE FOE FOO"
        data 140,"SHORT",141,"LONG",142,"BRIEF",143,"QUIT",143,"STOP",143,"END"
        data 144,"SCORE",145,"SAVE",146,"LOAD",147,"READ",147,"EXAMINE"
        data 148,"YES",148,"Y",149,"BUG",150,"*"
        restore L580
        for i = 1 to 200
            k(i) = 0
        end for

        z1 = 0 : z3 = 0
        ' T3=TOTAL NUMBER OF KEYWORDS
        while z1 <= t3
            read z1,b$
            b$ = " "+b$+" "
            ' IF KEYWORD WAS FOUND, NOTE THIS IN K(Z1)
            if instr(a$,b$) <> 0 then
                k(z1) = 1
            end if
        end while
        ' EXOTIC WORDS
        for x = 36 to 46
            if k(x) = 1 then
                findFirstNamedItem()
                askWhatToDoWithItem()
                ' goto, not `continue` -- this is two loop levels away from the
                ' inner command loop's own top (this `for` is level one), and
                ' `continue` only ever escapes its own innermost loop.
                c$="" : goto L410
            end if
        end for
        for x = 110 to t3
            if k(x) = 1 then
                goto L1950
            end if
        end for
        ' THEN IT'S A DIRECTION
        ' Both branches below `goto` a real label instead of using `exit`/
        ' `continue` -- this `for` is one loop level away from both the outer
        ' redisplay loop (`L300`) and the inner command loop's own top
        ' (`L400`), and `exit`/`continue` only ever reach their own innermost
        ' loop.
        for d = 1 to 10
            if k(d+99) = 1 then
                if attemptMove%(d) then
                    checkPitsAndReincarnateIfNeeded() : goto L300
                else
                    goto L400
                end if
            end if
        end for
        ' COMMAND NOT A DIRECTION
        goto L1950
        ' OTHER COMMANDS
        ' L1950 is a shared entry point jumped to directly from the exotic-
        ' words/direction resolution above (its own initial `z1 = 100` folded
        ' into the `for` below), so its label stays put even though the loop
        ' body is now structured.
        L1950: for z1 = 100 to t3
            if k(z1) = 1 then
                goto L2090
            end if
        end for
        ' ITEM BO NO VERB?
        ' (was: restore L9961 -- item names now come from itemname$())
        for x = 1 to 35
            d$ = itemname$(x)
            if k(x) = 1 then
                askWhatToDoWithItem()
                ' goto, not `continue` -- see the exotic-words `for` above.
                c$="" : goto L410
            end if
        end for
        printDontUnderstand()
        continue
        L2070: data "What?","I don't understand.","I can't understand that.","I don't know that word."
        L2090: z1 = z1-109
        select case z1
            case 1
                ' *** PLUGH ***
                IF L1<>7 THEN goto L2170
                IF S(35)=L1 THEN S(35)=0
                Z2=26
                performMove%(Z2)
                checkPitsAndReincarnateIfNeeded() : exit
                L2170: IF L1<>26 THEN goto L2200
                Z2=7
                performMove%(Z2)
                checkPitsAndReincarnateIfNeeded() : exit
                L2200: printMessage(2)
                continue
            case 2
                ' *** XYZZY ***
                IF L1<>7 THEN goto L2270
                IF S(35)=L1 THEN S(35)=0
                Z2=13
                performMove%(Z2)
                checkPitsAndReincarnateIfNeeded() : exit
                L2270: IF L1<>13 THEN goto L2200
                Z2=7
                performMove%(Z2)
                checkPitsAndReincarnateIfNeeded() : exit
            case 3
                ' *** PLOVER *** (CAN'T BRING EMERALD WITH HIM)
                IF L1>26 THEN goto L2360
                IF S(35)<>L1 THEN goto L2330
                S(35) = 0
                L2330: IF S(10)<>-1 THEN goto L2340
                S(10) = L1
                L2340: Z2 = 58
                performMove%(Z2)
                checkPitsAndReincarnateIfNeeded() : exit
                L2360: IF L1<>58 THEN goto L2200
                Z2=26
                performMove%(Z2)
                checkPitsAndReincarnateIfNeeded() : exit
            case 4
                ' *** CROSS ***
                IF L1<>19 THEN goto L2470
                IF B2<>0 THEN goto L2440
                L2420: printMessage(3)
                continue
                L2440: D=7
                ' JUST GIVE NEW DIRECTION, USE MOVE ROUTINE
                if attemptMove%(D) then
                    checkPitsAndReincarnateIfNeeded() : exit
                else
                    continue
                end if
                L2470: IF L1<>20 THEN goto L2510
                IF B2=0 THEN goto L2420
                D=3
                if attemptMove%(D) then
                    checkPitsAndReincarnateIfNeeded() : exit
                else
                    continue
                end if
                L2510: IF L1<>60 THEN goto L2540
                D=2
                if attemptMove%(D) then
                    checkPitsAndReincarnateIfNeeded() : exit
                else
                    continue
                end if
                L2540: IF L1<>61 THEN goto L2200
                D=6
                if attemptMove%(D) then
                    checkPitsAndReincarnateIfNeeded() : exit
                else
                    continue
                end if
            case 5
                ' *** CLIMB ***
                IF L1<>50 THEN goto L2200
                ' CAN HE CLIMB BEANSTALK?
                IF P1<2 THEN goto L2200
                ' YES
                Z2=70
                performMove%(Z2)
                checkPitsAndReincarnateIfNeeded() : exit
            case 6
                ' *** JUMP *** STRICTLY SUICIDAL
                IF L1<>16 AND L1<>19 AND L1<>20 AND L1<>27 THEN goto L2200
                printMessage(4)
                reincarnate() : exit
            case 7
                ' FILL
                IF S(21)=-1 THEN goto L2730
                L2700: B$="bottle"
                L2710: PRINT "You don't have the ";b$
                c$="" : continue
                L2730: IF B0=0 THEN goto L2760
                printMessage(5)
                c$="" : continue
                L2760: IF L1<>7 AND L1<>8 AND L1<>9 AND L1<>35 AND L1<>74 AND L1<>81 THEN goto L2790
                B0=1:S(16)=-1
                GOTO L2840
                L2790: IF L1=49 THEN goto L2830
                B$="oil"
                L2810: PRINT "I see no ";B$;" here."
                continue
                L2830: B0=2:S(17)=-1
                L2840: PRINT "The bottle is now filled."
                continue
            case 8
                ' *** EMPTY ***
                IF S(21)=-1 THEN goto L2890
                GOTO L2700
                L2890: ' EMPTY BOTTLE (ASSUMED FULL)
                S(B0+15)=0:B0=0
                PRINT "Emptied"
                continue
            case 9
                ' *** LOOK ***
                L2940: if l1 < 13 or l1 = 58 then goto L2970
                if l = 1 and (s(18) = l1 or s(18) = -1) then goto L2970
                printMessage(45)
                continue
                L2970: longDescription() ' (was: gosub 8050 -- jumped past the old
                ' subroutine's own `v(l1)=1` to avoid redundantly re-marking the room
                ' visited; by the time LOOK is typeable the room's already been entered
                ' via describeRoomOnEntry(), which already sets v(l1)=1, so calling the
                ' full longDescription() here just re-does that no-op assignment)
                describeRoomContents()
                continue
            case 10
                ' *** LIGHT ***
                if s(18) = -1 then goto L3040
                L3020: b$ = "lamp"
                goto L2710
                L3040: l = 1
                b$ = "on"
                L3060: PRINT "The lamp is now ";b$
                goto L2940
            case 11
                ' *** OFF (EXTINGUSIH) ***
                IF S(18)=-1 THEN goto L3110
                GOTO L3020
                L3110: L=0:B$="off"
                GOTO L3060
            case 12
                ' *** ENTER ***
                IF L1<>6 THEN goto L3180
                ' TO HOUSE
                D=3
                if attemptMove%(D) then
                    checkPitsAndReincarnateIfNeeded() : exit
                else
                    continue
                end if
                L3180: IF L1<>68 THEN goto L3240
                ' TO BARREN ROOM
                D=3
                if attemptMove%(D) then
                    checkPitsAndReincarnateIfNeeded() : exit
                else
                    continue
                end if
                L3240: D = 10
                LW3240: if D < 1 then goto L3270
                Z2 = DIRS(L1,D)
                IF Z2>0 AND Z2<101 THEN
                    if checkSpecialRoomAndMove%(D, Z2) then
                        checkPitsAndReincarnateIfNeeded() : exit
                    else
                        continue
                    end if
                end if
                D = D + (-1)
                goto LW3240
                L3270:
                GOTO L2200
            case 13
                ' ** LEAVE ***
                IF L1<>7 THEN goto L3340
                ' LEAVE HOUSE
                D=7
                if attemptMove%(D) then
                    checkPitsAndReincarnateIfNeeded() : exit
                else
                    continue
                end if
                L3340: IF L1<>69 THEN goto L3400
                ' LEAVE BARREN ROOM
                D = 7
                if attemptMove%(D) then
                    checkPitsAndReincarnateIfNeeded() : exit
                else
                    continue
                end if
                L3400: D = 1
                LW3400: if D > 10 then goto L3430
                Z2 = DIRS(L1,D)
                IF Z2>0 AND Z2<101 THEN
                    if checkSpecialRoomAndMove%(D, Z2) then
                        checkPitsAndReincarnateIfNeeded() : exit
                    else
                        continue
                    end if
                end if
                D = D + (1)
                goto LW3400
                L3430:
                GOTO L2200
            case 14
                ' *** INVENTORY ***
                Z0=0
                PRINT "You are carrying:";
                FOR X=1 TO T2
                    IF S(X)=-1 THEN
                        b$ = itemname$(x)
                        PRINT B$
                        Z0 = Z0 + 1
                    end if
                end for

                IF Z0=0 THEN goto L3551 else goto L3560
                L3551: PRINT "nothing."
                L3560: PRINT
                continue
            case 15
                ' *** GET ***
                if k(47) = 1 then goto L3680
                findMatchedItems()
                if z8 > 0 then goto L3680
                PRINT "Get what?"
                printDontUnderstand()
                continue
                L3680: z3 = 1
                LW3680: if z3 > t2 then goto L3900
                if k(47) = 1 then goto L3730
                if k(z3) = 0 then goto LCONT3680
                L3730: if s(z3) <> l1 then goto L3750
                if s(z3) = l1 then goto L3790
                L3750: if k(47) = 1 then goto LCONT3680
                a$ = itemname$(z3):PRINT a$;" not here."
                goto LCONT3680
                ' MUST CHECK NOW FOR LEGALITY OF TAKING ITEM
                L3790: z8 = 0
                for x = 1 to t2
                    if s(x) = -1 then
                        z8 = z8+1
                    end if
                end for

                if z8 < 7 then goto L3870
                ' CARRYING TOO MUCH
                printMessage(54)
                c$="" : continue
                L3870: goto L6880
                L3880: s(z3) = -1
                L3890: a$ = itemname$(z3):PRINT a$;":taken."
                LCONT3680: z3 = z3 + (1)
                goto LW3680
                L3900:
                continue
            case 16
                ' *** DROP ***
                if k(47) = 1 then goto L4000
                findMatchedItems()
                IF Z8>0 THEN goto L4000
                PRINT "Drop what?"
                printDontUnderstand()
                continue
                L4000: Z3 = 1
                LW4000: if Z3 > T2 then goto L4140
                IF K(47)=1 THEN goto L4060
                IF K(Z3)<>1 THEN goto LCONT4000
                IF S(Z3)=0 THEN goto LCONT4000
                L4060: IF S(Z3)=-1 THEN goto L4100
                IF K(47)=1 THEN goto LCONT4000
                b$ = itemname$(z3):PRINT "You don't have the ";B$
                GOTO LCONT4000
                L4100: ' STILL NEED TO ELABORATE ON DROP (BIRD IN CAGE, BOTTLE)
                GOTO L7380
                L4120: b$ = itemname$(z3):PRINT B$;":dropped."
                S(Z3)=L1
                LCONT4000: Z3 = Z3 + (1)
                goto LW4000
                L4140:
                continue
            case 17
                ' *** THROW ***
                findMatchedItems()
                IF Z8>0 THEN goto L4210
                PRINT "Throw what?"
                printDontUnderstand()
                continue
                L4210: IF S(Z3)<>-1 THEN goto L2710
                IF NOT (Z3<16 AND S(32)=L1) THEN goto L4260
                ' THROW TREASURE TO TROLL
                printMessage(27)
                S(Z3)=0:T=3:continue
                L4260: IF NOT (Z3=27 AND S(32)=L1) THEN goto L4300
                ' TRYING TO BUTCHER TROLL?
                printMessage(26)
                S(27)=L1:continue
                L4300: IF NOT (Z3=27 AND S(35)=L1) THEN goto L4380
                ' TRYING TO KILL DWARF
                IF RND(1)>0.5 THEN goto L4360
                printMessage(29)
                checkDwarfAttack() ' (was: GOSUB 8650 -- jumped straight into the
                ' attack-check half of the original dwarf subroutine, deliberately
                ' skipping the axe-giving check; see checkDwarfAttack()'s own comment)
                GOTO L4410
                L4360: printMessage(30)
                S(35)=0:GOTO L4410
                L4380: ' NOTHING SPECIAL, JUST DROP ITEM
                IF S(35)<>L1 THEN goto L4400
                checkDwarf() ' (was: GOSUB L8550 -- L8550 was just a comment
                ' immediately before the real dwarf subroutine's first line, L8560, so
                ' this call wanted the full checkDwarf() behavior, axe-check included --
                ' a call site missed when checkDwarf()/checkDwarfAttack() were split out)
                L4400: PRINT "Thrown."
                L4410: S(Z3) = L1
                if dead = 1 then reincarnate() : exit
                continue
            case 18
                ' *** ATTACK ***
                findMatchedItems()
                IF NOT (Z3=33 AND S(Z3)=L1 AND L1=82) THEN goto L4520
                ' HE CAN KILL DRAGON
                printMessage(68)
                c$="" : continue
                L4520: IF S(32)<>L1 THEN goto L4560
                ' TRYING TO MUNGE TROLL
                ' (was: Z9=FNA(25):GOTO 400 -- FNA is called with no matching DEF FN
                ' anywhere in the original source; this looks like leftover/broken code
                ' from an earlier version of the upstream port, not something this port
                ' introduced. Z9's assignment here isn't read before it's next assigned
                ' elsewhere, so dropping the call changes nothing observable.)
                continue
                L4560: IF NOT (Z3=26 OR Z3>30) THEN goto L4600
                ' DANGEROUS TO ATTACK THESE
                printMessage(70)
                continue
                L4600: ' NOTHING TO ATTACK
                printMessage(71)
                continue
            case 19
                ' *** FEED ***
                findMatchedItems()
                IF Z3<>35 THEN goto L4690
                ' CAN'T FEED DWARF!
                printMessage(24)
                continue
                L4690: IF S(20) = -1 THEN goto L4720
                B$ = "FOOD":GOTO L2710
                L4720: IF L1=69 THEN goto L4760
                PRINT "I can't feed it."
                printMessage(23)
                continue
                L4760: IF S(20)=L1 THEN goto L7600
                B1=1:S(20)=0:printMessage(6)
                continue
            case 20
                ' *** WATER ***
                IF S(16) = -1 THEN goto L4840
                B$ = "water":GOTO L2710
                L4840: IF L1<>50 THEN goto L2200
                ' GOTO P1+1 OF 4860,4890,4920
                IF P1 = 0 THEN goto L4860
                IF P1 = 1 THEN goto L4890
                IF P1 = 2 THEN goto L4920
                L4860: printMessage(7)
                P1=1:S(16)=0:B0=0:continue
                L4890: printMessage(8)
                P1=2:S(16)=0:B0=0:continue
                L4920: printMessage(9)
                P1=0:S(16)=0:B0=0:continue
            case 21
                ' *** LOCK ***
                L4960: IF L1=10 OR L1=11 THEN goto L4990
                ' NOTHING LOCKABLE
                GOTO L2200
                L4990: IF S(19)=-1 THEN goto L5020
                L5000: B$="keys":goto L2710
                L5020: G=0:printMessage(10)
                continue
            case 22
                ' *** UNLOCK ***
                L5070: IF S(19)<>-1 THEN goto L5000
                IF L1<>10 AND L1<>11 THEN goto L5120
                G=1:printMessage(11)
                continue
                L5120: IF L1<>69 THEN goto L2200
                IF B1>0 THEN goto L5160
                printMessage(12)
                continue
                L5160: IF C<>0 THEN goto L5170
                C=1:B1=2
                L5170: printMessage(13)
                continue
            case 23
                ' *** FREE ***
                IF K(31) = 1 THEN goto L5240
                ' CAN'T FREE ANYTHING BUT BIRD
                L5220: printMessage(2)
                c$="" : continue
                L5240: IF S(31)<>-1 THEN goto L5220
                S(31) = L1:B3=0
                PRINT "Freed."
                IF L1<>22 THEN goto L5350
                IF SN<>1 THEN continue
                B$="snake"
                L5300: PRINT "The little bird attacks the green ";B$;" and"
                IF L1=82 THEN goto L5380
                PRINT "drives it off"
                SN=0:S(34)=0:continue
                L5350: IF L1<>82 THEN continue
                B$="dragon":GOTO L5300
                L5380: PRINT "gets burned to a crisp"
                S(31)=0
                continue
            case 24
                ' *** WAVE ***
                IF K(23) <> 1 THEN goto L2200
                IF S(23)=-1 THEN goto L5460
                B$="rod":GOTO L2710
                L5460: '  IS HERE NEAR FISSURE
                IF L1<>19 AND L1<>20 THEN goto L2200
                ' yes
                ' GOTO B2+1 OF 5500,5530
                IF B2=0 THEN goto L5500
                IF B2=1 THEN goto L5530
                L5500: printMessage(14)
                B2=1:continue
                L5530: printMessage(15)
                B2=0:continue
            case 25
                ' *** OPEN ***
                findMatchedItems()
                IF Z3>0 THEN goto L5610
                PRINT "Open ";
                printDontUnderstand()
                continue
                L5610: IF Z3=40 THEN goto L5070
                IF S(Z3)=L1 THEN goto L5650
                L5630: PRINT "I see no ";b$;" here.":continue
                L5650: if z3=24 THEN goto L5680
                PRINT "I don't know how to open a ";B$:continue
                L5680: IF S(9)=-1 THEN goto L5710
                printMessage(16)
                continue
                L5710: IF S(Z3) = 0 THEN goto L2200
                ' HE'S OPENED CLAM, SO PRINT DESCRIPTION OF THIS
                ' PUT PEARL IN CUL-DE-SAC
                S(7)=43:S(24)=0:S(30)=L1:printMessage(17)
                continue
            case 26
                ' *** CLOSE *** -- an upstream bug (present already in stage 2/3,
                ' not introduced by this refactor): the dispatch table points CLOSE's
                ' entry at this exact "GOTO L400" line -- the same line OPEN's own
                ' handler ends on above -- rather than at the real CLOSE logic just
                ' below (L5760-L5800), which is consequently dead code, unreachable
                ' from anywhere. Preserved exactly as-is (CLOSE silently does nothing,
                ' matching the original's real behavior) rather than "fixed" to call
                ' the logic that was clearly intended; this is a faithfulness refactor,
                ' not a gameplay bugfix. Split into its own copy of the shared line so
                ' each verb has its own case block below.
                continue
                ' *** CLOSE *** (dead code -- see the comment above)
                findMatchedItems()
                IF Z3=40 THEN goto L4960
                printMessage(18)
                continue
            case 27
                ' OIL
                IF K(17)=0 THEN goto L2200
                IF S(17)=-1 THEN goto L5860
                B$="oil":GOTO L5630
                L5860: IF L1<>73 THEN goto L2200
                ' IS DOOR STILL RUSTED
                IF D2=1 THEN goto L2200
                D2=1:S(17)=0:B0=0:printMessage(19)
                continue
                ' *** EAT ***
            case 28
                IF K(20) = 1 THEN goto L5950
                printMessage(20)
                c$="" : continue
                L5950: Z3=20:checkCarryingItem()
                IF Z5=0 THEN c$="" : continue
                printMessage(73)
                S(20)=0:B0=0:continue
            case 29
                ' *** DRINK ***
                IF K(16) =1 THEN goto L6040
                printMessage(21)
                c$="" : continue
                L6040: Z3=16:checkCarryingItem()
                IF Z5=0 THEN c$="" : continue
                printMessage(22)
                S(17)=0:B0=0:continue
            case 30
                ' *** FEE FIE FOE FOO ***
                IF L1=71 THEN goto L6130
                printMessage(2)
                c$="" : continue
                L6130: IF S(8)<>L1 THEN goto L6180
                ' MAKE NEST VANISH
                printMessage(79)
                S(8)=0:continue
                L6180: ' IF S(8)=0 THEN goto L6110
                S(8)=L1
                ' MAKE NEST RE-APPEAR
                printMessage(81)
                continue
            case 31
                ' *** SHORT ***
                PRINT "Short descriptions"
                D0=0:continue
            case 32
                ' *** LONG ***
                PRINT "Long descriptions"
                D0=1:continue
            case 33
                ' *** BRIEF ***
                PRINT "OK, I'll only describe the room in detail the first time."
                D0=2:continue
            case 34
                ' *** QUIT ***
                PRINT "Save game";
                Z0 = askYesNo%()
                IF Z0=1 THEN goto L8970
                endGame()
                ' SCORE ***
            case 35
                printScore()
                continue
                L6430: ' score computation/printing are now computeScore()/
                ' printScore() above -- the DATA line right below is still needed
                ' here (RESTORE targets must be top-level, per issue #149/PR #150).
                L6470: DATA "beginner","novice","experienced","advanced","expert"
                ' list items at location l1
                ' room-contents listing + dwarf/pirate checks are now the
                ' describeRoomContents() procedure above.
                ' Print Short room description
                ' short room description is now the shortDescription() procedure above.
                L6880: ' SPECIAL GETS
                if not (z3 = 24 or z3 = 30 or z3 > 31) then goto L6930
                ' CAN'T GET THESE FOR SOME REASON
                printMessage(61)
                continue
                L6930: if not (z3 = 12 and c = 0) then goto L6970
                ' CHAIN
                printMessage(58)
                goto L3900
                L6970: ' BEAR IS HE FED? UNLOCKED?
                if not (z3 = 26 and b1 <> 2) then goto L7010
                printMessage(61)
                goto L3900
                L7010: if not (z3 = 14 and d1 = 1) then goto L7050
                ' DRAGON AND RUG
                printMessage(59)
                goto L3900
                L7050: if not (z3 = 16 or z3 = 17) then goto L7090
                ' OIL AND WATER DO SAME AS FILL
                PRINT "Why not say 'fill'?"
                goto L3900
                L7090: if not (z3 = 22 and b3) then goto L7140
                ' TAKE BIRD SINCE IT'S IN CAGE
                s(31) = -1:PRINT "Bird and ";:goto L3880
                L7140: if z3 <> 31 then goto L7310
                ' GETTING BIRD
                if b3 <> 1 then goto L7210
                ' TAKE CAGE, SINCE BIRD IS IN IT
                PRINT "Cage and ";:s(22) = -1:goto L3880
                L7210: if s(22) = -1 then goto L7240
                b$ = "cage":goto L2810
                L7240: if s(23) = -1 then goto L7280
                ' OK TO TAKE BIRD
                s(31) = -1 : b3 = 1:goto L3890
                L7280: ' ROD SCARES BIRD
                printMessage(37)
                goto L3900
                L7310: ' BOTTLE FULL? IF SO, GET CONTENTS
                if not (z3 = 21 and b0) then goto L7360
                PRINT "Contents and the ";
                s(b0+15) = -1
                L7360: goto L3880
                ' SPECIAL "DROP"
                L7380: IF Z3<>31 THEN goto L7440
                ' BIRD IN CAGE
                L7400: S(31)=L1:S(22)=L1:B3=1
                IF Z3<>31 THEN goto L7420
                PRINT "Cage and ";
                L7420: IF Z3=22 THEN goto L7430
                PRINT "Bird and ";
                L7430: goto L4120
                L7440: if z3=22 and b3=1 then goto L7400
                IF Z3<>21 THEN goto L7520
                ' BOTTLE
                IF B0=0 THEN goto L4120
                ' BOTTLE IS FULL, DO DROP CONTENTS TOO
                PRINT "Contents and ";
                S(15+B0)=L1:GOTO L4120
                L7520: IF NOT (Z3=16 OR Z3=17) THEN goto L7541
                PRINT "Try saying 'empty'":goto L4140
                L7541: IF Z3<>26 OR T<>1 OR (L1<>60 AND L1<>61) THEN goto L7550
                printMessage(28)
                T=0:S(26)=L1:S(32)=0:continue
                L7550: IF Z3<>6 THEN goto L4120
                IF S(28)=L1 THEN goto L7600
                ' GOODBYE, FRAGILE VASE!
                printMessage(43)
                S(6)=0:S(29)=L1:continue
                L7600: printMessage(60)
                GOTO L4120
                ' PRINT MESSAGE is now the printMessage() procedure above -- every
                ' `z59 = N : gosub 7620` call site became `printMessage(N)`.
                L7800: ' situation descriptions are now the situationDescriptions()
                ' procedure above.
                ' long room description is now the longDescription() procedure above.
                ' the short-vs-long-on-entry decision is now the describeRoomOnEntry()
                ' procedure above.
                '
                '   fetch first item code in k(1 to t2)
                '   z8=total # of items found in list
                '   z3=item code first found
                L8280: ' find-matched-items/find-first-named-item/check-carrying-item
                ' are now findMatchedItems()/findFirstNamedItem()/checkCarryingItem()
                ' above.
                ' dwarf/pirate checks are now checkDwarf()/checkDwarfAttack()/
                ' checkPirate() above.
            case 36
                L8970: ' *** SAVE GAME ***
                INPUT "What do you want to call the save file? ";A$
                ' `on error goto` (not `try`/`catch`) here on purpose -- see the
                ' README's own note on why, and GitHub issues #61 (on error goto
                ' is permanently unsupported under --target c, by design) & #100
                ' (try/catch's RESUME output isn't accepted by real fbc).
                on error goto L9010
                OPEN A$ FOR OUTPUT AS #5
                on error goto 0
                GOTO L9030
                L9010: PRINT "File ";a$;" not created"
                c$="" : continue
                L9030: PRINT #5,T1;",";T2;",";T3;",";L1;",";L2;",";G;",";B0;",";SN;",";D1;",";D2;",";D0;",";T;",";B1;",";B2;",";P1;",";L;",";C;",";D3;",";B3;",";R0;",";KC
                FOR X=1 TO 99
                    PRINT #5,S(X);",";V(X)
                end for

                PRINT #5,V(100)
                CLOSE #5
                PRINT "Game saved"
                C0 = 0
                if k(143) = 1 then endGame()
                c$="" : continue
            case 37
                ' *** LOAD OLD GAME ***
                IF C0=0 THEN goto L9120
                PRINT "You already have a loaded game!"
                c$="" : continue
                L9120: INPUT "Save file name? ";A$
                on error goto L9150
                OPEN A$ FOR INPUT AS #5
                on error goto 0
                GOTO L9170
                L9150: PRINT "Unable to use file ";A$
                c$="" : continue
                L9170: INPUT #5,T1,T2,T3,L1,L2,G,B0,SN,D1,D2,D0,T,B1,B2,P1,L,C,D3,B3,R0,KCX$
                kc = val(kcx$)
                FOR X=1 TO 99
                    INPUT #5,SX,VX:s(x)=sx:v(x)=vx
                end for

                INPUT #5,vx:v(100)=vx
                CLOSE #5
                C0=1
                exit
            case 38
                ' *** READ THE MAGAZINE ***
                findMatchedItems()
                IF Z3=25 THEN goto L9270
                printMessage(74)
                c$="" : continue
                L9270: IF S(25)=-1 THEN goto L9300
                B$="magazine"
                GOTO L2710
                L9300: ' OK, LET HIM READ IT
                printMessage(303)
                continue
            case 39
                ' *** YES (only meaningful in the dragon's lair, room 82) *** --
                ' physically relocated here from inside the ATTACK handler's own body
                ' (stage 2/3 had it at its original numeric position, L4490, sitting
                ' between L4480 and L4520 -- reachable only via the dispatch table,
                ' never by ATTACK's own fall-through, so moving it doesn't change
                ' behavior) so its case can be its own block in the SELECT CASE below.
                if L1 <> 82 then
                    printDontUnderstand()
                    continue
                end if
                printMessage(69)
                S(33)=0:D1=0:continue
            case 40
                ' *** BUG ***
                A$ = "ADVBUGS.TXT"
                on error goto L9150
                OPEN A$ FOR APPEND AS #5
                on error goto 0
                INPUT "Your name: ";A$
                A$=A$+" "+DATE$
                PRINT #5,A$
                PRINT "Enter your gripe in up to five lines (hit return to quit):"
                Z0 = 1
                LW9430: if Z0 > 5 then goto L9480
                PRINT Z0;
                INPUT A$
                IF A$="" THEN goto L9490
                PRINT #5,A$
                Z0 = Z0 + (1)
                goto LW9430
                L9480:
                L9490: PRINT "Message recorded. Thank you!"
                CLOSE #5
                c$="" : continue
            case else
                printDontUnderstand()
                continue
        end select
    end while
end while
' *** SEEK A "YES" OR "NO" is now the askYesNo%() function above.
' ---- SHORT NAMES FOR STUFF ----
L9961: data "large gold nugget"
data "bars of silver"
data "precious jewelry"
data "many coins"
data "several diamonds"
data "fragile ming vase"
data "glistening pearl"
data "nest of golden eggs"
data "jewel-encrusted trident"
data "egg-sized emerald"
data "platinum pyramid"
data "golden chain"
data "rare spices"
data "persian rug"
data "treasure chest"
data "water"
data "oil"
data "brass lamp"
data "keys"
data "food"
data "bottle"
data "wicker cage"
data "3-foot black rod"
data "clam"
data "magazine"
data "bear"
data "axe"
data "velvet pillow"
data "shards of pottery"
data "oyster"
data "bird"
data "troll"
data "dragon"
data "snake"
data "dwarf"
data "rock"
data "stairs"
data "steps"
data "house"
data "grate"
data "stream"
data "room"
data "bridge"
data "pit"
data "volcano"
data "road"
data "everything"
' INITIALIZE MESSAGE INDEX is now the buildMessageIndex() procedure above.

```

</details>

<!-- END generated stage source -->

[← Adventure Game](adventure.md)
