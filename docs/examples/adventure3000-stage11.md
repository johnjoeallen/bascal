[Home](../../) / [Examples](index.md) / [ADVENTURE/3000 Port](adventure3000.md) / Stage 11: The Last GOTO in checkDwarfAttack

<div class="prose" markdown="1">

# Stage 11: The Last GOTO in checkDwarfAttack

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

</div>

<details class="source-embed" markdown="1">

<summary><code>stage11-refactored-bascal/adventure.bcl</code> -- Stage 11: The Last GOTO in checkDwarfAttack</summary>

```bascal

// ADVENTURE/3000 -- Stage 11: a small mop-up on stage 9's own goal --
// modernizing `checkDwarfAttack()`, the one procedure stage 9 missed.
// Stage 9 was scoped to the 40 `verbXxx%()` functions only, and this
// helper (called from `checkDwarf()` and `verbThrow%()`, not a verb
// function itself) still had classic one-line `IF cond THEN goto Label`
// chains read verbatim from stage 2. Started as an exact copy of
// stage 10. See ../README.md for the port's provenance and staging.
//
// `checkDwarfAttack()`'s `GOTO`-chained branches became `IF`/`ELSEIF`/
// `ELSE` blocks, the same treatment stage 9 gave every verb function.
// A sweep of every other procedure and function in the file (not just
// verbs) turned up nothing else -- this genuinely was the one leftover.
//
// The rest of this file's remaining `GOTO`s are either the outer game
// loop's own intentional `L300`/`L400`/`L410` labels (see stage 6's
// header for why those can't become `continue`/`exit`), `ON ERROR
// GOTO`'s replacement `TRY`/`CATCH` doesn't touch procedure-internal
// control flow, or the command-parsing cascade's own shared re-entry
// labels (`L500`/`L1950`/`L2090`, from stage 5) -- that cascade's
// internal structure is its own, larger, not-yet-attempted piece of
// work, flagged here rather than folded into this stage's small scope.
//
// Verified with `bcc --check` and the stage 10 smoke suite (run under
// `--target c`, now that it's available, rather than `fbc`) against a
// stage 10 baseline -- byte-identical throughout, including a scratch
// copy with an axe and a dwarf forced into the starting room to
// exercise `checkDwarfAttack()`'s own branches specifically (attack
// roll, the bridge/troll-appeased message, the knife-throw kill/miss
// roll), confirmed across several different room-entry counts to vary
// which point in the random-number sequence each run consumes.
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
    if l1 < 13 then
        s(35) = 0
        return
    end if
    if s(35) <> l1 then
        ' SHOULD WE PUT A DWARF HERE?
        if RND(1) >= 0.05 then return
        if (l1 = 60 or l1 = 61) and t = 1 then return
        s(35) = l1
        printMessage(31)
        return
    end if
    if (l1 <> 60 and l1 <> 61) or t <> 1 then
        if RND(1) > 0.5 then return
        ' YES!
        printMessage(32)
        ' DOES THE KNIFE KILL THE PLAYER?
        KC = KC - 0.02
        if KC < 0.75 then
            KC = 0.75
        end if
        if RND(1) <= KC then
            PRINT "It gets you!"
            dead = 1
        else
            PRINT "It misses!"
        end if
    else
        printMessage(299)
        s(35) = 0
    end if
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

' Shows the room as LOOK does: the full long description unless it's
' dark and unlit, in which case just the "too dark to see" message --
' replaces stage 2's L2940-L2970 GOSUB-like target, shared by LOOK
' itself and by LIGHT/OFF (both re-run this same check right after
' changing the lamp's state, since that can change whether it applies).
procedure describeRoomForLook()
    global l1
    global l
    global s
    if l1 < 13 or l1 = 58 or (l = 1 and (s(18) = l1 or s(18) = -1)) then
        longDescription() ' (was: gosub 8050 -- jumped past the old
        ' subroutine's own `v(l1)=1` to avoid redundantly re-marking the
        ' room visited; by the time LOOK is typeable the room's already
        ' been entered via describeRoomOnEntry(), which already sets
        ' v(l1)=1, so calling the full longDescription() here just
        ' re-does that no-op assignment)
        describeRoomContents()
    else
        printMessage(45)
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
            case else
                ' unreachable in practice (t is always 0, 1, or 2 --
                ' see this function's own comment above) -- exists only
                ' so every path through this SELECT CASE has an explicit
                ' return, which the C backend requires.
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

' ==========================================================================
' Stage 8: each of the 40 verb handlers below is now its own function,
' returning 0 ("get another command") or 1 ("redisplay the room") for the
' dispatcher (the SELECT CASE further down) to act on with `continue`/
' `exit` itself -- a procedure can only ever `return` to its own caller,
' never `continue`/`exit` a loop in the *caller's* scope the way every
' handler used to do directly when it was inline in that SELECT CASE, so
' this return code is what replaces every one of those. Every internal
' GOTO that used to escape all the way out to the outer command loop (was
' `goto L400`/`goto L410`/`goto L300`) is now just `return 0`/`return 1`
' instead -- a `return` unwinds the whole function regardless of loop
' nesting, so GET's and DROP's and ENTER's and LEAVE's own scan loops no
' longer need the two-loop-levels `goto` workaround stage 6/7 needed when
' this logic still lived directly inside the SELECT CASE itself. A bare
' `continue` inside one of those scan loops still means exactly what it
' always did: try the next item/direction, not the outer command loop.
' ==========================================================================

function verbPlugh%()
    global l1
    global s
    global z2
    ' *** PLUGH ***
    if l1 = 7 then
        if s(35) = l1 then
            s(35) = 0
        end if
        z2 = 26
    elseif l1 = 26 then
        z2 = 7
    else
        printMessage(2)
        return 0
    end if
    performMove%(z2)
    checkPitsAndReincarnateIfNeeded()
    return 1
end function

function verbXyzzy%()
    global l1
    global s
    global z2
    ' *** XYZZY ***
    if l1 = 7 then
        if s(35) = l1 then
            s(35) = 0
        end if
        z2 = 13
    elseif l1 = 13 then
        z2 = 7
    else
        printMessage(2)
        return 0
    end if
    performMove%(z2)
    checkPitsAndReincarnateIfNeeded()
    return 1
end function

function verbPlover%()
    global l1
    global s
    global z2
    ' *** PLOVER *** (CAN'T BRING EMERALD WITH HIM)
    if l1 <= 26 then
        if s(35) = l1 then
            s(35) = 0
        end if
        if s(10) = -1 then
            s(10) = l1
        end if
        z2 = 58
    elseif l1 = 58 then
        z2 = 26
    else
        printMessage(2)
        return 0
    end if
    performMove%(z2)
    checkPitsAndReincarnateIfNeeded()
    return 1
end function

function verbCross%()
    global l1
    global b2
    global d
    global z2
    ' *** CROSS ***
    if l1 = 19 and b2 = 0 then
        printMessage(3)
    elseif l1 = 19 then
        ' JUST GIVE NEW DIRECTION, USE MOVE ROUTINE
        d = 7
        if attemptMove%(d) then
            checkPitsAndReincarnateIfNeeded()
            return 1
        end if
    elseif l1 = 20 and b2 = 0 then
        printMessage(3)
    elseif l1 = 20 then
        d = 3
        if attemptMove%(d) then
            checkPitsAndReincarnateIfNeeded()
            return 1
        end if
    elseif l1 = 60 then
        d = 2
        if attemptMove%(d) then
            checkPitsAndReincarnateIfNeeded()
            return 1
        end if
    elseif l1 = 61 then
        d = 6
        if attemptMove%(d) then
            checkPitsAndReincarnateIfNeeded()
            return 1
        end if
    else
        printMessage(2)
    end if
    return 0
end function

function verbClimb%()
    global l1
    global p1
    global z2
    ' *** CLIMB ***
    IF L1<>50 THEN printMessage(2) : return 0
    ' CAN HE CLIMB BEANSTALK?
    IF P1<2 THEN printMessage(2) : return 0
    ' YES
    Z2=70
    performMove%(Z2)
    checkPitsAndReincarnateIfNeeded()
    return 1
end function

function verbJump%()
    global l1
    ' *** JUMP *** STRICTLY SUICIDAL
    IF L1<>16 AND L1<>19 AND L1<>20 AND L1<>27 THEN printMessage(2) : return 0
    printMessage(4)
    reincarnate()
    return 1
end function

function verbFill%()
    global l1
    global s
    global b0
    global b$
    global c$
    ' FILL
    if S(21) <> -1 then
        B$="bottle"
        PRINT "You don't have the ";b$
        c$=""
        return 0
    end if
    if B0 <> 0 then
        printMessage(5)
        c$=""
        return 0
    end if
    if L1=7 or L1=8 or L1=9 or L1=35 or L1=74 or L1=81 then
        B0=1:S(16)=-1
    elseif L1=49 then
        B0=2:S(17)=-1
    else
        B$="oil"
        PRINT "I see no ";B$;" here."
        return 0
    end if
    PRINT "The bottle is now filled."
    return 0
end function

function verbEmpty%()
    global s
    global b0
    global b$
    global c$
    ' *** EMPTY ***
    if S(21)<>-1 then
        B$="bottle" : PRINT "You don't have the ";b$ : c$=""
        return 0
    end if
    ' EMPTY BOTTLE (ASSUMED FULL)
    S(B0+15)=0:B0=0
    PRINT "Emptied"
    return 0
end function

function verbLook%()
    ' *** LOOK ***
    describeRoomForLook()
    return 0
end function

function verbLight%()
    global s
    global l
    global b$
    global c$
    ' *** LIGHT ***
    if s(18) = -1 then
        l = 1
        b$ = "on"
    else
        b$ = "lamp"
        PRINT "You don't have the ";b$
        c$=""
        return 0
    end if
    PRINT "The lamp is now ";b$
    describeRoomForLook()
    return 0
end function

function verbOff%()
    global s
    global l
    global b$
    global c$
    ' *** OFF (EXTINGUSIH) ***
    if s(18) = -1 then
        L=0:B$="off"
    else
        b$ = "lamp"
        PRINT "You don't have the ";b$
        c$=""
        return 0
    end if
    PRINT "The lamp is now ";b$
    describeRoomForLook()
    return 0
end function

function verbEnter%()
    global l1
    global d
    global z2
    global dirs
    ' *** ENTER ***
    if l1 = 6 or l1 = 68 then
        ' TO HOUSE / TO BARREN ROOM
        D=3
        if attemptMove%(D) then
            checkPitsAndReincarnateIfNeeded()
            return 1
        end if
        return 0
    end if
    ' Scan every direction for one that works, starting from the last
    ' (D=10, "down") -- was a manual D=10 downto 1 GOTO loop (LW3240).
    ' `return` here, not `exit`/`continue` -- it unwinds this whole
    ' function regardless of the `for` loop it's inside.
    for D = 10 to 1 step -1
        Z2 = DIRS(L1,D)
        IF Z2>0 AND Z2<101 THEN
            if checkSpecialRoomAndMove%(D, Z2) then
                checkPitsAndReincarnateIfNeeded()
                return 1
            else
                return 0
            end if
        end if
    end for
    printMessage(2)
    return 0
end function

function verbLeave%()
    global l1
    global d
    global z2
    global dirs
    ' ** LEAVE ***
    if l1 = 7 or l1 = 69 then
        ' LEAVE HOUSE / LEAVE BARREN ROOM
        D=7
        if attemptMove%(D) then
            checkPitsAndReincarnateIfNeeded()
            return 1
        end if
        return 0
    end if
    ' Scan every direction for one that works, starting from the first
    ' (D=1, "north") -- was a manual D=1 to 10 GOTO loop (LW3400). See
    ' ENTER's own comment above for why this uses `return` instead of
    ' `exit`/`continue`.
    for D = 1 to 10
        Z2 = DIRS(L1,D)
        IF Z2>0 AND Z2<101 THEN
            if checkSpecialRoomAndMove%(D, Z2) then
                checkPitsAndReincarnateIfNeeded()
                return 1
            else
                return 0
            end if
        end if
    end for
    printMessage(2)
    return 0
end function

function verbInventory%()
    global z0
    global s
    global t2
    global b$
    global itemname$
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

    if Z0=0 then
        PRINT "nothing."
    end if
    PRINT
    return 0
end function

function verbGet%()
    global k
    global z8
    global z3
    global s
    global l1
    global c
    global b1
    global d1
    global b3
    global b0
    global t2
    global itemname$
    global a$
    global b$
    global c$
    ' *** GET ***
    if k(47) <> 1 then
        findMatchedItems()
        if z8 = 0 then
            PRINT "Get what?"
            printDontUnderstand()
            return 0
        end if
    end if
    ' Scan every item for one matching the player's command and present
    ' in this room -- was a manual z3=1 to t2 GOTO loop (LW3680). `return`
    ' below, not `exit`/`continue`, wherever the *original* GOTO's
    ' ultimate target meant "abandon the scan and get another command"
    ' rather than "try the next item" -- `return` unwinds this whole
    ' function regardless of the `for` loop it's inside. A bare
    ' `continue` below really does mean "try the next item": every such
    ' site was the old loop's own LCONT3680 (or, for the special-gets
    ' checks folded in below, a genuine take that should keep scanning
    ' for more when GET ALL is in effect).
    for z3 = 1 to t2
        if k(47) <> 1 and k(z3) = 0 then continue
        if s(z3) <> l1 then
            if k(47) <> 1 then
                a$ = itemname$(z3) : PRINT a$;" not here."
            end if
            continue
        end if
        ' MUST CHECK NOW FOR LEGALITY OF TAKING ITEM
        z8 = 0
        for x = 1 to t2
            if s(x) = -1 then
                z8 = z8+1
            end if
        end for

        if z8 >= 7 then
            ' CARRYING TOO MUCH
            printMessage(54)
            c$=""
            return 0
        end if
        ' SPECIAL GETS -- was L6880, a GOSUB-like target reached only from
        ' this scan.
        if z3 = 24 or z3 = 30 or z3 > 31 then
            ' CAN'T GET THESE FOR SOME REASON
            printMessage(61)
            return 0
        elseif z3 = 12 and c = 0 then
            ' CHAIN
            printMessage(58)
            return 0
        elseif z3 = 26 and b1 <> 2 then
            ' BEAR IS HE FED? UNLOCKED?
            printMessage(61)
            return 0
        elseif z3 = 14 and d1 = 1 then
            ' DRAGON AND RUG
            printMessage(59)
            return 0
        elseif z3 = 16 or z3 = 17 then
            ' OIL AND WATER DO SAME AS FILL
            PRINT "Why not say 'fill'?"
            return 0
        elseif z3 = 22 and b3 then
            ' TAKE BIRD SINCE IT'S IN CAGE
            s(31) = -1 : PRINT "Bird and ";
        elseif z3 = 31 then
            ' GETTING BIRD
            if b3 = 1 then
                ' TAKE CAGE, SINCE BIRD IS IN IT
                PRINT "Cage and "; : s(22) = -1
            elseif s(22) <> -1 then
                b$ = "cage" : PRINT "I see no ";b$;" here."
                return 0
            elseif s(23) = -1 then
                ' ROD SCARES BIRD
                printMessage(37)
                return 0
            else
                ' OK TO TAKE BIRD
                b3 = 1
            end if
        elseif z3 = 21 and b0 then
            ' BOTTLE FULL? IF SO, GET CONTENTS
            PRINT "Contents and the ";
            s(b0+15) = -1
        end if
        s(z3) = -1
        a$ = itemname$(z3):PRINT a$;":taken."
    end for
    return 0
end function

function verbDrop%()
    global k
    global z8
    global z3
    global s
    global l1
    global b3
    global b0
    global t
    global t2
    global b$
    global itemname$
    ' *** DROP ***
    if k(47) <> 1 then
        findMatchedItems()
        if Z8 = 0 then
            PRINT "Drop what?"
            printDontUnderstand()
            return 0
        end if
    end if
    ' Scan every item the player is carrying that matches the command --
    ' was a manual Z3=1 to T2 GOTO loop (LW4000). See GET's own comment
    ' above for why this uses `return` instead of `exit`/`continue`.
    for Z3 = 1 to T2
        if K(47) <> 1 then
            if K(Z3) <> 1 or S(Z3) = 0 then continue
        end if
        if S(Z3) <> -1 then
            if K(47) <> 1 then
                b$ = itemname$(z3):PRINT "You don't have the ";B$
            end if
            continue
        end if
        ' STILL NEED TO ELABORATE ON DROP (BIRD IN CAGE, BOTTLE) -- was
        ' L7380 onward, a GOSUB-like target reached only from here.
        if Z3 = 31 or (Z3 = 22 and B3 = 1) then
            ' BIRD IN CAGE
            S(31)=L1:S(22)=L1:B3=1
            if Z3 = 31 then
                PRINT "Cage and ";
                PRINT "Bird and ";
            end if
        elseif Z3 = 21 and B0 <> 0 then
            ' BOTTLE IS FULL, DO DROP CONTENTS TOO
            PRINT "Contents and ";
            S(15+B0)=L1
        elseif Z3 = 16 or Z3 = 17 then
            PRINT "Try saying 'empty'"
            return 0
        elseif Z3 = 26 and T = 1 and (L1 = 60 or L1 = 61) then
            printMessage(28)
            T=0:S(26)=L1:S(32)=0
            return 0
        elseif Z3 = 6 and S(28) <> L1 then
            ' GOODBYE, FRAGILE VASE!
            printMessage(43)
            S(6)=0:S(29)=L1
            return 0
        elseif Z3 = 6 then
            printMessage(60)
        end if
        b$ = itemname$(z3):PRINT B$;":dropped."
        S(Z3)=L1
    end for
    return 0
end function

function verbThrow%()
    global z8
    global z3
    global s
    global b$
    global c$
    global l1
    global t
    global dead
    ' *** THROW ***
    findMatchedItems()
    if z8 = 0 then
        PRINT "Throw what?"
        printDontUnderstand()
        return 0
    end if
    if s(z3) <> -1 then
        PRINT "You don't have the ";b$ : c$="" : return 0
    end if
    if z3 < 16 and s(32) = l1 then
        ' THROW TREASURE TO TROLL
        printMessage(27)
        s(z3) = 0 : t = 3
        return 0
    elseif z3 = 27 and s(32) = l1 then
        ' TRYING TO BUTCHER TROLL?
        printMessage(26)
        s(27) = l1
        return 0
    elseif z3 = 27 and s(35) = l1 then
        ' TRYING TO KILL DWARF
        if rnd(1) <= 0.5 then
            printMessage(29)
            checkDwarfAttack() ' (was: GOSUB 8650 -- jumped straight into
            ' the attack-check half of the original dwarf subroutine,
            ' deliberately skipping the axe-giving check; see
            ' checkDwarfAttack()'s own comment)
        else
            printMessage(30)
            s(35) = 0
        end if
    else
        ' NOTHING SPECIAL, JUST DROP ITEM
        if s(35) = l1 then
            checkDwarf() ' (was: GOSUB L8550 -- L8550 was just a comment
            ' immediately before the real dwarf subroutine's first line,
            ' L8560, so this call wanted the full checkDwarf() behavior,
            ' axe-check included -- a call site missed when checkDwarf()/
            ' checkDwarfAttack() were split out)
        end if
        PRINT "Thrown."
    end if
    s(z3) = l1
    if dead = 1 then
        reincarnate()
        return 1
    end if
    return 0
end function

function verbAttack%()
    global z3
    global s
    global l1
    global c$
    ' *** ATTACK ***
    findMatchedItems()
    if z3 = 33 and s(z3) = l1 and l1 = 82 then
        ' HE CAN KILL DRAGON
        printMessage(68)
        c$=""
    elseif s(32) = l1 then
        ' TRYING TO MUNGE TROLL
        ' (was: Z9=FNA(25):GOTO 400 -- FNA is called with no matching DEF FN
        ' anywhere in the original source; this looks like leftover/broken
        ' code from an earlier version of the upstream port, not something
        ' this port introduced. Z9's assignment here isn't read before it's
        ' next assigned elsewhere, so dropping the call changes nothing
        ' observable.)
    elseif z3 = 26 or z3 > 30 then
        ' DANGEROUS TO ATTACK THESE
        printMessage(70)
    else
        ' NOTHING TO ATTACK
        printMessage(71)
    end if
    return 0
end function

function verbFeed%()
    global z3
    global s
    global l1
    global b$
    global c$
    global b1
    global itemname$
    ' *** FEED ***
    findMatchedItems()
    if z3 = 35 then
        ' CAN'T FEED DWARF!
        printMessage(24)
        return 0
    end if
    if s(20) <> -1 then
        B$ = "FOOD" : PRINT "You don't have the ";b$ : c$="" : return 0
    end if
    if l1 <> 69 then
        PRINT "I can't feed it."
        printMessage(23)
        return 0
    end if
    if S(20)=L1 then
        ' was: `GOTO L7600`, itself `printMessage(60):GOTO L4120` -- L4120
        ' was DROP's own per-item "dropped" epilogue, reused here verbatim
        ' (feeding the bear food already here also "drops" it via the
        ' exact same z3/S() update DROP's own scan loop does, now inlined
        ' there instead of living at a shared label).
        printMessage(60)
        b$ = itemname$(z3):PRINT b$;":dropped."
        S(Z3)=L1
    else
        B1=1:S(20)=0:printMessage(6)
    end if
    return 0
end function

function verbWater%()
    global s
    global b$
    global c$
    global l1
    global p1
    global b0
    ' *** WATER ***
    if s(16) <> -1 then
        B$ = "water" : PRINT "You don't have the ";b$ : c$="" : return 0
    end if
    if l1 <> 50 then
        printMessage(2)
        return 0
    end if
    ' GOTO P1+1 OF 4860,4890,4920
    if p1 = 0 then
        printMessage(7)
        p1 = 1
    elseif p1 = 1 then
        printMessage(8)
        p1 = 2
    else
        printMessage(9)
        p1 = 0
    end if
    S(16)=0:B0=0
    return 0
end function

function verbLock%()
    global l1
    global s
    global g
    global b$
    global c$
    ' *** LOCK ***
    L4960: if L1=10 or L1=11 then
        if S(19)=-1 then
            G=0:printMessage(10)
        else
            B$="keys"
            PRINT "You don't have the ";b$ : c$="" : return 0
        end if
    else
        ' NOTHING LOCKABLE
        printMessage(2)
    end if
    return 0
end function

function verbUnlock%()
    global s
    global l1
    global g
    global b1
    global c
    global b$
    global c$
    ' *** UNLOCK ***
    L5070: if S(19) <> -1 then
        B$="keys"
        PRINT "You don't have the ";b$ : c$="" : return 0
    end if
    if L1=10 or L1=11 then
        G=1:printMessage(11)
    elseif L1=69 then
        if B1>0 then
            if C=0 then
                C=1:B1=2
            end if
            printMessage(13)
        else
            printMessage(12)
        end if
    else
        printMessage(2)
    end if
    return 0
end function

function verbFree%()
    global k
    global s
    global l1
    global b3
    global sn
    global b$
    global c$
    ' *** FREE ***
    if K(31) <> 1 or S(31) <> -1 then
        ' CAN'T FREE ANYTHING BUT BIRD
        printMessage(2)
        c$=""
        return 0
    end if
    S(31) = L1:B3=0
    PRINT "Freed."
    if L1 = 22 and SN = 1 then
        B$ = "snake"
    elseif L1 = 82 then
        B$ = "dragon"
    else
        return 0
    end if
    PRINT "The little bird attacks the green ";B$;" and"
    if L1 = 82 then
        PRINT "gets burned to a crisp"
        S(31)=0
    else
        PRINT "drives it off"
        SN=0:S(34)=0
    end if
    return 0
end function

function verbWave%()
    global k
    global s
    global b$
    global c$
    global l1
    global b2
    ' *** WAVE ***
    if K(23) <> 1 then
        printMessage(2)
    elseif S(23) <> -1 then
        B$="rod" : PRINT "You don't have the ";b$ : c$="" : return 0
    elseif L1<>19 and L1<>20 then
        ' NOT NEAR FISSURE
        printMessage(2)
    elseif B2=0 then
        printMessage(14)
        B2=1
    else
        printMessage(15)
        B2=0
    end if
    return 0
end function

function verbOpen%()
    global z3
    global s
    global l1
    global b$
    global c$
    ' *** OPEN ***
    findMatchedItems()
    if Z3=0 then
        PRINT "Open ";
        printDontUnderstand()
        return 0
    end if
    if Z3=40 then return verbUnlock%() ' OPEN a lock is the same as UNLOCK
    if S(Z3)<>L1 then
        PRINT "I see no ";b$;" here."
        return 0
    end if
    if z3<>24 then
        PRINT "I don't know how to open a ";B$
        return 0
    end if
    if S(9)=-1 then
        printMessage(16)
        return 0
    end if
    if S(Z3) = 0 then
        printMessage(2)
        return 0
    end if
    ' HE'S OPENED CLAM, SO PRINT DESCRIPTION OF THIS
    ' PUT PEARL IN CUL-DE-SAC
    S(7)=43:S(24)=0:S(30)=L1:printMessage(17)
    return 0
end function

function verbClose%()
    global z3
    global s
    ' *** CLOSE *** -- an upstream bug (present already in stage 2/3, not
    ' introduced by this refactor): the dispatch table points CLOSE's
    ' entry at this exact "GOTO L400" line -- the same line OPEN's own
    ' handler ends on above -- rather than at the real CLOSE logic just
    ' below (L5760-L5800), which is consequently dead code, unreachable
    ' from anywhere. Preserved exactly as-is (CLOSE silently does nothing,
    ' matching the original's real behavior) rather than "fixed" to call
    ' the logic that was clearly intended; this is a faithfulness
    ' refactor, not a gameplay bugfix.
    return 0
    ' *** CLOSE *** (dead code -- see the comment above)
    findMatchedItems()
    IF Z3=40 THEN return verbLock%()
    printMessage(18)
    return 0
end function

function verbOil%()
    global k
    global s
    global l1
    global d2
    global b0
    global b$
    ' OIL
    if K(17)=0 then
        printMessage(2)
    elseif S(17)<>-1 then
        B$="oil" : PRINT "I see no ";b$;" here."
    elseif L1<>73 then
        printMessage(2)
    elseif D2=1 then
        ' IS DOOR STILL RUSTED
        printMessage(2)
    else
        D2=1:S(17)=0:B0=0:printMessage(19)
    end if
    return 0
end function

function verbEat%()
    global k
    global z3
    global z5
    global s
    global b0
    global c$
    ' *** EAT ***
    if K(20) <> 1 then
        printMessage(20)
        c$=""
        return 0
    end if
    Z3=20:checkCarryingItem()
    if Z5=0 then
        c$=""
        return 0
    end if
    printMessage(73)
    S(20)=0:B0=0
    return 0
end function

function verbDrink%()
    global k
    global z3
    global z5
    global s
    global b0
    global c$
    ' *** DRINK ***
    if K(16) <> 1 then
        printMessage(21)
        c$=""
        return 0
    end if
    Z3=16:checkCarryingItem()
    if Z5=0 then
        c$=""
        return 0
    end if
    printMessage(22)
    S(17)=0:B0=0
    return 0
end function

function verbFeeFieFoeFoo%()
    global l1
    global s
    global c$
    ' *** FEE FIE FOE FOO ***
    if l1 <> 71 then
        printMessage(2)
        c$=""
    elseif s(8) = l1 then
        ' MAKE NEST VANISH
        printMessage(79)
        S(8)=0
    else
        ' IF S(8)=0 THEN goto L6110
        S(8)=L1
        ' MAKE NEST RE-APPEAR
        printMessage(81)
    end if
    return 0
end function

function verbShort%()
    global d0
    ' *** SHORT ***
    PRINT "Short descriptions"
    D0=0
    return 0
end function

function verbLong%()
    global d0
    ' *** LONG ***
    PRINT "Long descriptions"
    D0=1
    return 0
end function

function verbBrief%()
    global d0
    ' *** BRIEF ***
    PRINT "OK, I'll only describe the room in detail the first time."
    D0=2
    return 0
end function

function verbQuit%()
    global z0
    ' *** QUIT ***
    PRINT "Save game";
    Z0 = askYesNo%()
    IF Z0=1 THEN return verbSaveGame%()
    endGame()
    return 0
end function

function verbScore%()
    printScore()
    return 0
end function

function verbSaveGame%()
    global a$
    global t1
    global t2
    global t3
    global l1
    global l2
    global g
    global b0
    global sn
    global d1
    global d2
    global d0
    global t
    global b1
    global b2
    global p1
    global l
    global c
    global d3
    global b3
    global r0
    global kc
    global s
    global v
    global c0
    global c$
    global k
    ' *** SAVE GAME ***
    INPUT "What do you want to call the save file? ";A$
    try
        OPEN A$ FOR OUTPUT AS #5
    catch err%, erl%
        PRINT "File ";a$;" not created"
        c$=""
        return 0
    end try
    PRINT #5,T1;",";T2;",";T3;",";L1;",";L2;",";G;",";B0;",";SN;",";D1;",";D2;",";D0;",";T;",";B1;",";B2;",";P1;",";L;",";C;",";D3;",";B3;",";R0;",";KC
    FOR X=1 TO 99
        PRINT #5,S(X);",";V(X)
    end for

    PRINT #5,V(100)
    CLOSE #5
    PRINT "Game saved"
    C0 = 0
    if k(143) = 1 then
        endGame()
    end if
    c$=""
    return 0
end function

function verbLoadOldGame%()
    global a$
    global t1
    global t2
    global t3
    global l1
    global l2
    global g
    global b0
    global sn
    global d1
    global d2
    global d0
    global t
    global b1
    global b2
    global p1
    global l
    global c
    global d3
    global b3
    global r0
    global kc
    global s
    global v
    global c0
    global c$
    ' *** LOAD OLD GAME ***
    if C0<>0 then
        PRINT "You already have a loaded game!"
        c$=""
        return 0
    end if
    INPUT "Save file name? ";A$
    try
        OPEN A$ FOR INPUT AS #5
    catch err%, erl%
        PRINT "Unable to use file ";A$
        c$=""
        return 0
    end try
    INPUT #5,T1,T2,T3,L1,L2,G,B0,SN,D1,D2,D0,T,B1,B2,P1,L,C,D3,B3,R0,KCX$
    kc = val(kcx$)
    FOR X=1 TO 99
        INPUT #5,SX,VX:s(x)=sx:v(x)=vx
    end for

    INPUT #5,vx:v(100)=vx
    CLOSE #5
    C0=1
    return 1
end function

function verbReadMagazine%()
    global z3
    global s
    global b$
    global c$
    ' *** READ THE MAGAZINE ***
    findMatchedItems()
    if z3 <> 25 then
        printMessage(74)
        c$=""
    elseif s(25) <> -1 then
        B$="magazine"
        PRINT "You don't have the ";b$ : c$=""
    else
        ' OK, LET HIM READ IT
        printMessage(303)
    end if
    return 0
end function

function verbYes%()
    global l1
    global s
    global d1
    ' *** YES (only meaningful in the dragon's lair, room 82) *** --
    ' physically relocated here from inside the ATTACK handler's own body
    ' (stage 2/3 had it at its original numeric position, L4490, sitting
    ' between L4480 and L4520 -- reachable only via the dispatch table,
    ' never by ATTACK's own fall-through, so moving it doesn't change
    ' behavior) so its case can be its own block below.
    if L1 <> 82 then
        printDontUnderstand()
        return 0
    end if
    printMessage(69)
    S(33)=0:D1=0
    return 0
end function

function verbBug%()
    global a$
    global z0
    global c$
    ' *** BUG ***
    A$ = "ADVBUGS.TXT"
    try
        OPEN A$ FOR APPEND AS #5
    catch err%, erl%
        PRINT "Unable to use file ";A$
        c$=""
        return 0
    end try
    INPUT "Your name: ";A$
    A$=A$+" "+DATE$
    PRINT #5,A$
    PRINT "Enter your gripe in up to five lines (hit return to quit):"
    Z0 = 1
    while Z0 <= 5
        PRINT Z0;
        INPUT A$
        if A$="" then
            exit
        end if
        PRINT #5,A$
        Z0 = Z0 + (1)
    end while
    PRINT "Message recorded. Thank you!"
    CLOSE #5
    c$=""
    return 0
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
                rc = verbPlugh%()
            case 2
                rc = verbXyzzy%()
            case 3
                rc = verbPlover%()
            case 4
                rc = verbCross%()
            case 5
                rc = verbClimb%()
            case 6
                rc = verbJump%()
            case 7
                rc = verbFill%()
            case 8
                rc = verbEmpty%()
            case 9
                rc = verbLook%()
            case 10
                rc = verbLight%()
            case 11
                rc = verbOff%()
            case 12
                rc = verbEnter%()
            case 13
                rc = verbLeave%()
            case 14
                rc = verbInventory%()
            case 15
                rc = verbGet%()
            case 16
                rc = verbDrop%()
            case 17
                rc = verbThrow%()
            case 18
                rc = verbAttack%()
            case 19
                rc = verbFeed%()
            case 20
                rc = verbWater%()
            case 21
                rc = verbLock%()
            case 22
                rc = verbUnlock%()
            case 23
                rc = verbFree%()
            case 24
                rc = verbWave%()
            case 25
                rc = verbOpen%()
            case 26
                rc = verbClose%()
            case 27
                rc = verbOil%()
            case 28
                rc = verbEat%()
            case 29
                rc = verbDrink%()
            case 30
                rc = verbFeeFieFoeFoo%()
            case 31
                rc = verbShort%()
            case 32
                rc = verbLong%()
            case 33
                rc = verbBrief%()
            case 34
                rc = verbQuit%()
            case 35
                rc = verbScore%()
            case 36
                rc = verbSaveGame%()
            case 37
                rc = verbLoadOldGame%()
            case 38
                rc = verbReadMagazine%()
            case 39
                rc = verbYes%()
            case 40
                rc = verbBug%()
            case else
                printDontUnderstand()
                rc = 0
        end select
        if rc = 1 then exit
        continue
    end while
end while
' *** SEEK A "YES" OR "NO" is now the askYesNo%() function above.
' score computation/printing are now computeScore()/printScore() above,
' called from verbScore%() -- the DATA line right below is still needed
' here, at the top level (not inside verbScore%() itself), since RESTORE
' targets must be top-level per issue #149/PR #150.
L6470: DATA "beginner","novice","experienced","advanced","expert"
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

[← Stage 10: Try/Catch Error Handling](adventure3000-stage10.md) [Next: Stage 12: Extracting the Command Parser →](adventure3000-stage12.md)
