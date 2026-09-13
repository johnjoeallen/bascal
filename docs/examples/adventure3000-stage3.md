[Home](../../) / [Examples](sort-driver.md) / [ADVENTURE/3000 Port](adventure3000.md) / Stage 3: Clean GOSUBs Become Procedures

<div class="prose" markdown="1">

# Stage 3: Clean GOSUBs Become Procedures


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

</div>

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

[← Stage 2: Minimal Valid BASCAL](adventure3000-stage2.md) [Next: Stage 4: The 40-Way Dispatch Becomes SELECT CASE →](adventure3000-stage4.md)
