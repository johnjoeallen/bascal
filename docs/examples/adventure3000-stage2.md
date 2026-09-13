[Home](../../) / [Examples](sort-driver.md) / [ADVENTURE/3000 Port](adventure3000.md) / Stage 2: Minimal Valid BASCAL

<div class="prose" markdown="1">

# Stage 2: Minimal Valid BASCAL


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

</div>

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

[← Stage 1: The Original, Untouched](adventure3000-stage1.md) [Next: Stage 3: Clean GOSUBs Become Procedures →](adventure3000-stage3.md)
