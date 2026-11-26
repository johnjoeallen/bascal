[Home](../../) / [Tutorials](../) / Case Study: Random-Access Inventory

<div class="prose" markdown="1">

Every other tutorial in this series is a short walkthrough of one feature. This one is different: it's a reconstruction of a real, complete program — ["Example program for RANDOM ACCESS FILE study"](http://www.geocities.ws/joseph_sixpack/binventory.md), written by fhb in 1998 for Joseph Sixpack's "Last Book of GW-Basic" collection, and credited there as "suggested from MS-BASIC manual." It's a menu-driven parts inventory: check a part's status, edit or add a part, add or subtract stock, and print a reorder report, all backed by a fixed 100-record random-access file. Rebuilt in BASCAL, it exercises [record files](random_and_record_files.md), [procedures](procedures.md) (including `byref` parameters), [short-circuit `&&`/`||`](short_circuit.md), and [error handling](labels_and_error_handling.md) together, at the scale where classic BASIC's line-number bookkeeping starts to really hurt — and where BASCAL's structuring pays off.

This is a reconstruction, not a line-by-line port. A few of fhb's original pieces have no BASCAL equivalent and were deliberately dropped rather than approximated: the GOTO-driven "subroutine roadmap" dispatcher meant for navigating the listing in the GW-BASIC interpreter itself; `KEY OFF`/`VIEW PRINT` console features BASCAL doesn't expose; and a numeric-ERR-code-to-message lookup table, collapsed here into a single line reporting the raw `ERR`/`ERL` values. See the header comment in the source below for the full list, including why `inven.dat` must be pre-populated with 100 blank records before the program will run correctly — fhb's original had a one-time hidden initializer for this that has no BASCAL-source equivalent in `inventory.bcl` itself.

Verified against real BASCOM 2.00 under dosbox-x: it compiles clean and links, but only when BASCOM is invoked with the `/E` and `/X` switches, since `on error goto`/`resume` isn't linked in by default. [`scripts/run-in-dosbox.sh`](https://github.com/johnjoeallen/bascal/blob/main/scripts/run-in-dosbox.sh) in the repo automates all of this — compiling with `bcc`, seeding a blank `inven.dat` with 100 records itself (reproducing fhb's one-time initializer at the tooling level, outside the language), adding the right BASCOM switches, and launching an interactive dosbox-x session — so you can actually run this one instead of just reading it.

</div>

<div class="snippet" markdown="1">

### The record/file DSL replaces fhb's manual FIELD layout and MKx$/CVx$ packing

bcc computes the field widths and record `LEN` from the `record` declaration and emits the `FIELD` statement itself. Named field access (`p.flag`, `p.qty`, ...) and whole-record reads via `inv[n]` replace fhb's manual `GET`/`PUT` plus `LSET`/`RSET` and `MKI$`/`MKS$`/`CVI$`/`CVS$` packing.

```bascal
record Part
    flag:    string(1)
    desc:    string(30)
    qty:     int16
    reorder: int16
    price:   float32
end record

file inv as Part = open("inven.dat")

function isEmpty%(flag$)
    return asc(flag$) = 255
end function

let p = inv[part%]
if isEmpty%(p.flag) then
    ' ...
end if
```

</div>

<div class="snippet" markdown="1">

### `byref` parameters write straight back into the caller's variables

One call gathers all four editable fields for a part; no shared globals, no separate "output" convention to remember.

```bascal
procedure gatherPartDetails(partNum%, byref desc$, byref qty%, byref reorder%, byref price!)
    input "      Description"; desc$
    input "Quantity in stock"; qty%
    input "    Reorder level"; reorder%
    input "       Unit price"; price!
end procedure
```

</div>

<div class="snippet" markdown="1">

### The error handler is a `procedure` — and bcc proves it's safe to be one

`on error goto` reaches a label or a procedure identically, via a plain `GOTO`, never a `GOSUB` — so a procedure used this way has no call frame for `RETURN` to pop. bcc's resolver checks for exactly that: any procedure named as an `on error goto` target must contain no `return` anywhere, and every path must be proven to end in `resume`/`resume next`/`resume <label>` rather than falling through. `errorTrap()`'s single trailing `resume next` satisfies that, so codegen skips the implicit `RETURN` it would otherwise append — there's nothing left to fall into even if the proof were somehow wrong. The same check also rejects calling it like an ordinary procedure elsewhere: something proven to never return can never come back to a normal caller either.

```bascal
on error goto errorTrap
' ...
procedure errorTrap()
    locate 25, 1
    print "There has been an error on line" + str$(erl) + ": " + error$(err)
    k$ = readKey$()
    resume next
end procedure
```

</div>



[← Standard Library Functions](stdlib.md)


<!-- BEGIN generated tutorial source -->

<details class="source-embed" markdown="1">

<summary><code>tutorial/inventory.bcl</code></summary>



```bascal

// ============================================================
// INVENTORY.BCL -- Random-Access Inventory Program
//
// A BASCAL reconstruction of "Example program for RANDOM ACCESS
// FILE study", by fhb, 8/19/98, from Joseph Sixpack's GW-BASIC
// programs page (part of his "Last Book of GW-Basic" collection):
//   http://www.geocities.ws/joseph_sixpack/binventory.html
// fhb's own header comment credits the original as "suggested
// from MS-BASIC manual".
//
// This is a reconstruction, not a line-by-line port -- some
// original pieces have no BASCAL equivalent and were dropped
// rather than approximated:
//  - The GOTO-driven "subroutine roadmap" dispatcher at the top
//    of fhb's listing (a `LIST 110-320` etc. navigation aid for
//    editing in the GW-BASIC interpreter) has no meaning once the
//    program is structured into named function/procedure blocks.
//  - `KEY OFF` / `KEY I,""` (clearing the function-key soft-label
//    row) and `VIEW PRINT` (scroll-region windowing for the list
//    screen) are interpreter/console features BASCAL doesn't
//    expose.
//  - fhb's own hand-rolled numeric-ERR-code-to-message lookup table
//    (ERR=1 "Input value overflow", ERR=2 "Syntax error", ... ERR=25)
//    is replaced below by BASCAL's com.bascal.stdlib.error library
//    (ERROR$(code%)) -- same idea, BASCAL's own table; it still
//    doesn't decode ERL, which errorTrap() reports as the raw line
//    number.
//  - fhb's one-time "hidden" datafile initializer (PUT-ing 100
//    blank, CHR$(255)-flagged records) is reproduced below as
//    initializeInventoryFileIfNew(), called once at program entry --
//    inven.dat no longer has to be pre-populated by hand.
//  - The three original tab-position constants (T=20, U=25,
//    V=30) are collapsed into a single `TAB_COL = 20`; a couple of
//    screens that used U=25 in the original (see showAddStockScreen
//    below) keep 25 as a literal rather than reusing TAB_COL.
//
// Tracks parts in a fixed 100-record file: check status, add,
// edit, add/subtract stock, and a reorder report.
//
// Error handling uses try/catch (GitHub issue #60), not the raw `on
// error goto` / `resume next` fhb's original relies on: a failed menu
// action is abandoned outright and the program returns straight to the
// main menu, rather than resuming at the exact instruction after
// whatever failed -- see reportInventoryError() below and
// tutorial/inventory_try_catch.draft's own header comment for why. This
// is a real, deliberate behavior change from an earlier on-error-goto
// version of this file, which *was* verified against real BASCOM 2.00
// under dosbox-x (only with the /E and /X switches -- error trapping
// isn't linked in by default); the try/catch shape below transpiles to
// the same ON ERROR GOTO/RESUME primitives BASCOM accepts, but hasn't
// itself been independently re-verified against a real BASCOM compile.
// ============================================================
program inventory

require com.bascal.stdlib.error

// BASCAL-ism: the record/file DSL. `record ... end record` plus
// `file ... as ... = open(...)` below replace fhb's manual
// FIELD #1,1 AS F$,30 AS D$,2 AS Q$,... buffer layout entirely --
// bcc computes the field widths and record LEN from this
// declaration and generates the FIELD statement itself. Named
// field access (`p.flag`, `p.qty`, ...) and whole-record
// read/write via `inv[n]` (see checkPart() below) replace fhb's
// manual GET/PUT plus LSET/RSET and MKI$/MKS$/CVI$/CVS$ packing.
record Part
    flag: string(1) lpad
    desc: string(30) lpad
    qty: int16
    reorder: int16
    price: float32
end record

// BASCAL-ism: `const` is a real compile-time constant, not a plain
// variable assignment like fhb's `N=100` / `T=20` -- it can never
// be reassigned, and resolves to the same value everywhere,
// including inside every function/procedure below, with no
// `global` declaration needed.
const PART_COUNT = 100
const TAB_COL = 20

// `file ... = open(...)` is sugar for OPEN ... FOR RANDOM AS #n
// LEN = <record width> plus the FIELD statement fhb wrote out by
// hand at his line 550. Wrapped in its own try/catch: a file that
// exists but can't be opened for random access (permissions, a
// read-only inven.dat, disk full on the fallback create) is a real,
// trappable error (code 75, "Path/File access error") on both
// targets now, not a hard crash -- report it and exit cleanly
// instead of leaving the program to fail confusingly the first time
// something tries to use an `inv` that was never actually opened.
try
    file inv as Part = open("inven.dat")
catch err%, erl%
    print "could not open inven.dat: " + error$(err%)
    end
end try

// -------------------- Pure functions (no file access) --------------------

// BASCAL-ism: `function ... end function` with `return` replaces
// fhb's convention of a GOSUB target plus a bare RETURN -- there's
// no separate "subroutine label" and no shared/global result
// variable to manage by hand; `isEmpty%(...)` is called like an
// ordinary expression at every use below (e.g. `isEmpty%(p.flag)`).
// A record whose flag byte is CHR$(255) is an empty/never-used slot.
function isEmpty%(flag$)
    return asc(flag$) = 255
end function

// BASCAL-ism: `&&` and `||` are short-circuit AND/OR -- real
// MBASIC/BASCOM only has bitwise AND/OR (which fhb relies on here
// too, since `PART!<1 OR PART!>N!` never short-circuits anyway).
// BASCAL lowers `&&`/`||` into the equivalent branching so the
// short-circuit *is* real at the generated-BASIC level; see the
// manual's "Short-Circuit && and ||" section
// (https://johnjoeallen.github.io/bascal/manual/).
function partInRange%(n%)
    if n% >= 1 && n% <= PART_COUNT then
        return 1
    end if
    return 0
end function

function readPartNumberInput$()
    input "Input part number"; s$
    return s$
end function

// -------------------- Keyboard input --------------------

// BASCAL-ism: `do ... loop until` is a structured post-check loop
// replacing fhb's `730 KP$=INKEY$:IF KP$="" THEN 730` GOTO-polling
// idiom. `inkey$` itself is the real INKEY$ builtin passed straight
// through, resolving correctly from inside a function/procedure
// body like this one -- every menu action below calls
// readKey$()/waitAnyKey() rather than polling INKEY$ inline.
function readKey$()
    do
        k$ = inkey$
    loop until k$ <> ""
    return k$
end function

procedure waitAnyKey()
    locate 25, 10
    print "Press the AnyKey to continue...";
    do
        k$ = inkey$
    loop until k$ <> ""
end procedure

// -------------------- Display procedures --------------------

procedure showMainMenu()
    cls
    color 14, 4
    cls
    locate 6, 1
    print
    // `tab(n)` passes straight through to real TAB(n), same as
    // fhb's own `PRINT TAB(V) "..."` -- but only as a bare item in
    // a PRINT list, juxtaposed or `;`-separated like here. Real
    // BASCOM rejects `"literal" + tab(n) + ...` (TAB isn't a real
    // string function you can concatenate); see printListHeader()
    // and printReorderHeader() below, which need `;` between a
    // preceding string and a `tab(n)` for exactly this reason.
    print tab(30) "Inventory Program"
    print
    print tab(TAB_COL) "1......C)heck a part"
    print tab(TAB_COL) "2......E)dit/overwrite/add a part"
    print tab(TAB_COL) "3......L)ist all" + str$(PART_COUNT) + "parts"
    print tab(TAB_COL) "4......A)dd stock"
    print tab(TAB_COL) "5......S)ubtract stock"
    print tab(TAB_COL) "6......R)eorder Report"
    print
    print tab(TAB_COL) "7......eX)it to system"
end procedure

procedure showBadPartNumber()
    cls
    locate 10, 10
    print "Part number is out of permissable range of 1 to" + str$(PART_COUNT)
end procedure

procedure showRangeRetryMessage()
    locate 10, 15
    print "The Part number is out of permissable range of 1 to" + str$(PART_COUNT)
    locate 25, 15
    print "Press the Anykey to reenter part number...";
end procedure

procedure showNullEntryMessage(partStr$)
    locate 10, TAB_COL
    print "Part number " + partStr$ + " is a null entry"
end procedure

procedure showPartStatus(partNum%, desc$, qty%, reorder%, price!)
    cls
    locate 5, 1
    print tab(TAB_COL) "Inventory Status for Individual Part Number"
    print tab(TAB_COL) "==========================================="
    print
    print
    print tab(TAB_COL) "     Part number:  " + str$(partNum%)
    print
    print tab(TAB_COL) "       Item name:  " + desc$
    print tab(TAB_COL) "Quantity on hand:  " + str$(qty%)
    print tab(TAB_COL) "   Reorder level:  " + str$(reorder%)
    print tab(TAB_COL) "      Unit price:  " + str$(price!)
end procedure

// BASCAL-ism: no `VIEW PRINT` (see the header note above), so this
// deliberately does NOT pin a "press any key" line to a fixed row the way
// fhb's original does -- a bare `LOCATE 25, ...` sitting under content
// that keeps printing past it (listAll()'s own items) collides with
// whatever's later written there, since nothing here scrolls a bounded
// region: waitAnyKey() is the only thing that ever touches row 25, and
// only right when it actually blocks (see listAll()'s own redraw-per-page
// structure below).
procedure printListHeader()
    cls
    print tab(25) "I N V E N T O R Y   L I S T I N G"; tab(65); str$(PART_COUNT) + "items"
    print "                                          Quantity       Reorder"
    print " Partno           Description             on hand         level"
end procedure

procedure printInventoryLine(partNum%, desc$, qty%, reorder%)
    print str$(partNum%) + "  " + desc$ + "   " + str$(qty%) + "          " + str$(reorder%)
end procedure

procedure printReorderHeader()
    cls
    locate 1, TAB_COL
    print "Reorder Report"; tab(55); date$
    print
    print "                                             Quantity       Reorder"
    print "    Partno           Description             on hand         level"
    print "   =======  ==============================   ========       ======="
end procedure

procedure printReorderLine(partNum%, desc$, qty%, reorder%)
    print "  " + str$(partNum%) + "  " + desc$ + "   " + str$(qty%) + "          " + str$(reorder%)
end procedure

// byref scalar parameters: gatherPartDetails writes the four editable
// fields for a part directly back into the caller's variables.
procedure gatherPartDetails(partNum%, byref desc$, byref qty%, byref reorder%, byref price!)
    cls
    locate 4, TAB_COL
    print "Adding or Overwriting a Record"
    locate 8, TAB_COL
    print "Record/Partno" + str$(partNum%)
    locate 11, 39
    print "------------------------------"
    locate 10, TAB_COL
    input "      Description"; desc$
    locate 12, TAB_COL
    input "Quantity in stock"; qty%
    locate 14, TAB_COL
    input "    Reorder level"; reorder%
    locate 16, TAB_COL
    input "       Unit price"; price!
    locate 18, TAB_COL
    print "Is information correct (Y/N)?"
end procedure

procedure showAddStockScreen(partNum%, desc$, qty%, reorder%)
    cls
    locate 4, 25
    print "Add to an inventory part number"
    locate 5, 25
    print "==============================="
    locate 8, TAB_COL
    print "     Part number: " + str$(partNum%)
    locate 9, TAB_COL
    print "Item description: " + desc$
    locate 10, TAB_COL
    print "Quantity on hand: " + str$(qty%)
    locate 11, TAB_COL
    print "   Reorder Level: " + str$(reorder%)
end procedure

procedure showNegativeQtyWarning()
    locate 17, 15
    print "The quantity to add must NOT be a negative number"
    locate 25, 1
    print "Please press the Anykey to reenter quantity to add...";
end procedure

procedure showSubtractStockScreen(partNum%, desc$, qty%, reorder%)
    cls
    locate 4, TAB_COL
    print "Subtract an inventory part number"
    locate 5, TAB_COL
    print "================================="
    locate 8, TAB_COL
    print "         Part number: " + str$(partNum%)
    locate 9, TAB_COL
    print "    Item description: " + desc$
    locate 10, TAB_COL
    print "    Quantity on hand: " + str$(qty%)
    locate 11, TAB_COL
    print "       Reorder Level: " + str$(reorder%)
end procedure

procedure showOverSubtractWarning(onHand%)
    locate 17, 5
    print "The quantity to SUBTRACT must NOT result in NEGATIVE inventory"
    locate 18, 5
    print "Only" + str$(onHand%) + " IN STOCK"
    locate 25, 1
    print "Please press the Anykey to reenter quantity to subtract...";
end procedure

// -------------------- Menu actions --------------------

procedure checkPart()
    global inv
    partStr$ = readPartNumberInput$()
    part% = val(partStr$)
    if partInRange%(part%) = 0 then
        showBadPartNumber()
        waitAnyKey()
        return
    end if
    // BASCAL-ism: `let p = inv[part%]` reads record `part%` of the
    // `inv` file into a local record variable `p` -- one expression
    // for what fhb's `GET #1, PART!` plus five separate field reads
    // (F$, D$, CVI(Q$), CVI(R$), CVS(P$)) did by hand. The write
    // side, `inv[part%] = { ... }` (see editRecord() below), is the
    // same sugar for PUT plus the LSET/MKx$ packing it replaces.
    let p = inv[part%]
    if isEmpty%(p.flag) then
        cls
        locate 10, 18
        print "Part number" + str$(part%) + "is still a null entry at this time"
        waitAnyKey()
        return
    end if
    showPartStatus(part%, p.desc, p.qty, p.reorder, p.price)
    waitAnyKey()
end procedure

procedure editRecord()
    global inv
    cls
    locate 10, TAB_COL
    partStr$ = readPartNumberInput$()
    part% = val(partStr$)
    if partInRange%(part%) = 0 then
        showBadPartNumber()
        waitAnyKey()
        return
    end if
    let p = inv[part%]
    if isEmpty%(p.flag) = 0 then
        locate 12, TAB_COL
        print "Overwrite existing part data?"
        kp$ = readKey$()
        if kp$ <> "Y" && kp$ <> "y" then
            return
        end if
    end if

    do
        gatherPartDetails(part%, editDesc$, editQty%, editReorder%, editPrice!)
        kp$ = readKey$()
    loop until kp$ = "Y" || kp$ = "y"
    inv[part%] = { flag: "1", desc: editDesc$, qty: editQty%, reorder: editReorder%, price: editPrice! }
end procedure

procedure listAll()
    global inv
    printListHeader()
    scrollCount% = 0
    for i% = 1 to PART_COUNT
        let p = inv[i%]
        printInventoryLine(i%, p.desc, p.qty, p.reorder)
        scrollCount% = scrollCount% + 1
        if scrollCount% = 20 then
            waitAnyKey()
            scrollCount% = 0
            // Redraw for the next page rather than let it keep scrolling
            // past row 25 -- see printListHeader()'s own note on why a
            // fixed-row prompt can't coexist with unbounded scrolling.
            if i% < PART_COUNT then
                printListHeader()
            end if
        end if
    end for
end procedure

procedure addStock()
    global inv
    cls
    locate 5, 25
    print "A D D I N G   S T O C K"

    do
        locate 8, 25
        partStr$ = readPartNumberInput$()
        part% = val(partStr$)
        validPart% = partInRange%(part%)
        if validPart% = 0 then
            showRangeRetryMessage()
            readKey$()
        end if
    loop until validPart% <> 0

    let p = inv[part%]
    if isEmpty%(p.flag) then
        showNullEntryMessage(partStr$)
        readKey$()
        return
    end if

    do
        showAddStockScreen(part%, p.desc, p.qty, p.reorder)
        locate 14, TAB_COL
        input " Quantity to add"; addStr$
        addAmt% = val(addStr$)
        if addAmt% < 0 then
            showNegativeQtyWarning()
            readKey$()
        end if
    loop until addAmt% >= 0

    p.qty = p.qty + addAmt%
    inv[part%] = p
end procedure

procedure subtractStock()
    global inv
    cls
    locate 5, 20
    print "S U B T R A C T I N G    S T O C K"

    do
        locate 8, 25
        partStr$ = readPartNumberInput$()
        part% = val(partStr$)
        validPart% = partInRange%(part%)
        if validPart% = 0 then
            showRangeRetryMessage()
            readKey$()
        end if
    loop until validPart% <> 0

    let p = inv[part%]
    if isEmpty%(p.flag) then
        showNullEntryMessage(partStr$)
        readKey$()
        return
    end if

    do
        showSubtractStockScreen(part%, p.desc, p.qty, p.reorder)
        locate 14, TAB_COL
        input "Quantity to subtract"; subStr$
        subAmt% = val(subStr$)
        overSubtract% = 0
        if subAmt% >= 0 && p.qty - subAmt% < 0 then
            overSubtract% = 1
            showOverSubtractWarning(p.qty)
            readKey$()
        end if
    loop until subAmt% >= 0 && overSubtract% = 0

    p.qty = p.qty - subAmt%
    if p.qty <= p.reorder then
        locate 16, TAB_COL
    end if
    print "quantity now" + str$(p.qty) + " reorder level" + str$(p.reorder)
    inv[part%] = p
end procedure

procedure reorderReport()
    global inv
    printReorderHeader()
    reportLineCount% = 0
    for i% = 1 to PART_COUNT
        let p = inv[i%]
        if p.qty < p.reorder then
            printReorderLine(i%, p.desc, p.qty, p.reorder)
            reportLineCount% = reportLineCount% + 1
            if reportLineCount% > 15 then
                waitAnyKey()
                reportLineCount% = 0
                // Redraw for the next page rather than let it keep
                // scrolling past row 25 -- see printListHeader()'s own
                // note (same underlying issue, same fix) on why a
                // fixed-row prompt can't coexist with unbounded scrolling.
                if i% < PART_COUNT then
                    printReorderHeader()
                end if
            end if
        end if
    end for
    waitAnyKey()
end procedure

// fhb's own one-time "hidden" datafile initializer PUT-ing 100 blank,
// CHR$(255)-flagged records (see the header note above) -- reproduced
// here so inven.dat no longer has to be pre-populated by hand before
// running this program. A brand-new file OPEN created just now (rather
// than one that already existed) reads back as all-zero bytes: record
// 1's flag byte is CHR$(0), never CHR$(255) -- the one signal an
// already-populated file (whose record 1 flag is always either
// CHR$(255), still an empty slot, or a real part's own "1") could never
// produce, so it's what isEmpty%() itself can't use (see its own
// header note) but this one-time check safely can.
procedure initializeInventoryFileIfNew()
    global inv
    let p = inv[1]
    if asc(p.flag) = 0 then
        for i% = 1 to PART_COUNT
            inv[i%] = { flag: chr$(255), desc: "", qty: 0, reorder: 0, price: 0 }
        end for
    end if
end procedure

// -------------------- Program entry --------------------

cls
initializeInventoryFileIfNew()

do
    showMainMenu()
    kp$ = readKey$()
    if instr("1234567cCeElLaAsSrRxX", kp$) <> 0 then
        // BASCAL-ism: `select case` replaces fhb's chain of eight
        // `IF VAL(KP$)=n OR KP$="x" OR KP$="X" THEN GOTO ...` lines
        // (his 770-840) with one multi-way dispatch.
        //
        // BASCAL-ism: `try`/`catch` (issue #60) replaces fhb's own global
        // `ON ERROR GOTO` trap. A failed menu action is abandoned outright
        // here -- the `catch` below runs, then execution continues right
        // after `end try`, back at `loop until` -- rather than resuming at
        // the exact instruction after whatever failed inside checkPart()/
        // editRecord()/etc. the way fhb's `RESUME NEXT` did. See
        // reportInventoryError() below and tutorial/inventory_try_catch.
        // draft's own header comment for why that arbitrary resume-point
        // behavior isn't something try/catch reproduces.
        try
            select case kp$
                case "1", "c", "C"
                    checkPart()
                case "2", "e", "E"
                    editRecord()
                case "3", "l", "L"
                    listAll()
                case "4", "a", "A"
                    addStock()
                case "5", "s", "S"
                    subtractStock()
                case "6", "r", "R"
                    reorderReport()
                case "7", "x", "X"
                    // BASCAL-ism: `inv.close()` is sugar for `CLOSE #1`,
                    // matching fhb's own `90 CLOSE:SYSTEM`. fhb's original
                    // also had a separate "Quit to BASIC" option (his own
                    // 7, returning to the interpreter's command prompt
                    // rather than exiting to DOS) -- dropped here: a
                    // compiled program has no interpreter to return to,
                    // so it was never anything but a second syntax for
                    // this same close-and-exit action.
                    inv.close()
                    color 7, 0
                    cls
                    system
            end select
        catch err%, erl%
            reportInventoryError(err%, erl%)
        end try
    end if
end do

// -------------------- Error handling --------------------
// err%/erl% are ordinary locals scoped to the `catch` block above, not
// aliases for the ambient (readable-anywhere) `err`/`erl` pseudo-
// variables `on error goto` uses -- see `Statement::TryCatch`'s own doc
// comment in ast.rs. Passed straight through to ERROR$ here like fhb's
// own ERR/ERL (his 3390: "an error on line";ERL), decoded through
// BASCAL's own com.bascal.stdlib.error (ERROR$) instead of fhb's
// hand-rolled lookup table -- see the header note above. try/catch
// itself isn't documented in the manual yet (GitHub issue #60 tracks
// the still-unfinished C-target work; the manual page can follow once
// that lands) -- see ast.rs's own `Statement::TryCatch` doc comment for
// the full semantics meanwhile.
procedure reportInventoryError(err%, erl%)
    locate 25, 1
    print "There has been an error on line" + str$(erl%) + ": " + error$(err%)
    k$ = readKey$()
end procedure

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/inventory.bas</code></summary>



```basic

10 ' BASCAL generated BASIC -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Strips leading spaces from self$. Not a real MBASIC/BASCOM 2.00 builtin --
40 ' verified against a real IBM BASIC Compiler 2.00 under dosbox-x -- so
50 ' BASCAL ships its own. Declared as a scalar method (see GitHub issue #41)
60 ' so a required stdlib call reads the same way as a built-in method call
70 ' (docs/language/functions-and-procedures.html#built-in-methods). The
80 ' ordinary call form (ltrim$(s$)) still works -- a method's receiver is an
90 ' implicit first parameter, so ordinary-call syntax resolves straight to
100 ' this same declaration, with no separate function needed (and no longer
110 ' allowed: a function and a method sharing one name is a duplicate
120 ' declaration, since they'd both claim the same callable identity).
130 ' Strips trailing spaces from self$. Not a real MBASIC/BASCOM 2.00 builtin --
140 ' verified against a real IBM BASIC Compiler 2.00 under dosbox-x -- so
150 ' BASCAL ships its own. Declared as a scalar method (see GitHub issue #41
160 ' and ltrim.bcl's own doc comment for the reasoning) -- rtrim$(s$) still
170 ' works via ordinary-call syntax resolving to this same declaration.
180 ' Maps an ERR code to its classic MBASIC/GW-BASIC/BASCOM message. Compiles
190 ' and links on a real IBM BASIC Compiler 2.00 as ERROR$, but silently
200 ' returns an empty string at runtime (verified under dosbox-x) -- so BASCAL
210 ' ships a working implementation.
220 '
230 ' The named constants below are the complete common subset supported by
240 ' ERROR$: use them in THROW and filtered CATCH clauses instead of magic
250 ' numbers.  Dialect-specific errors outside this shared MBASIC/GW-BASIC/
260 ' BASCOM subset still fall through to ERROR$'s generic message.
270 '
280 ' Deliberately NOT a scalar method (see GitHub issue #41, which asked for
290 ' this decision to be recorded either way): code% is an opaque lookup key,
300 ' not a value the call is naturally "operating on" the way ltrim$/rtrim$/
310 ' ucase$/lcase$ operate on their string -- code%.error() would read as if
320 ' the *error code itself* has a message, when really this is a lookup
330 ' table keyed by that code. Stays an ordinary function.
340 CONSTERRSYNTAX% = 2
350 CONSTERRRETURNWITHOUTGOSUB% = 3
360 CONSTERROUTOFDATA% = 4
370 CONSTERRILLEGALFUNCTIONCALL% = 5
380 CONSTERROVERFLOW% = 6
390 CONSTERROUTOFMEMORY% = 7
400 CONSTERRSUBSCRIPTOUTOFRANGE% = 9
410 CONSTERRDUPLICATEDEFINITION% = 10
420 CONSTERRDIVISIONBYZERO% = 11
430 CONSTERRTYPEMISMATCH% = 13
440 CONSTERROUTOFSTRINGSPACE% = 14
450 CONSTERRNORESUME% = 19
460 CONSTERRRESUMEWITHOUTERROR% = 20
470 CONSTERRDEVICETIMEOUT% = 24
480 CONSTERRDEVICEFAULT% = 25
490 CONSTERROUTOFPAPER% = 27
500 CONSTERRBADFILENUMBER% = 52
510 CONSTERRFILENOTFOUND% = 53
520 CONSTERRBADFILEMODE% = 54
530 CONSTERRFILEALREADYOPEN% = 55
540 CONSTERRDEVICEIO% = 57
550 CONSTERRFILEALREADYEXISTS% = 58
560 CONSTERRDISKFULL% = 61
570 CONSTERRINPUTPASTEND% = 62
580 CONSTERRBADRECORDNUMBER% = 63
590 CONSTERRBADFILENAME% = 64
600 CONSTERRTOOMANYFILES% = 67
610 CONSTERRDEVICEUNAVAILABLE% = 68
620 CONSTERRDISKWRITEPROTECTED% = 70
630 CONSTERRDISKNOTREADY% = 71
640 CONSTERRDISKMEDIAERROR% = 72
650 CONSTERRPATHFILEACCESS% = 75
660 CONSTERRPATHNOTFOUND% = 76
670 ' ============================================================
680 ' INVENTORY.BCL -- Random-Access Inventory Program
690 '
700 ' A BASCAL reconstruction of "Example program for RANDOM ACCESS
710 ' FILE study", by fhb, 8/19/98, from Joseph Sixpack's GW-BASIC
720 ' programs page (part of his "Last Book of GW-Basic" collection):
730 ' http://www.geocities.ws/joseph_sixpack/binventory.html
740 ' fhb's own header comment credits the original as "suggested
750 ' from MS-BASIC manual".
760 '
770 ' This is a reconstruction, not a line-by-line port -- some
780 ' original pieces have no BASCAL equivalent and were dropped
790 ' rather than approximated:
800 ' - The GOTO-driven "subroutine roadmap" dispatcher at the top
810 ' of fhb's listing (a `LIST 110-320` etc. navigation aid for
820 ' editing in the GW-BASIC interpreter) has no meaning once the
830 ' program is structured into named function/procedure blocks.
840 ' - `KEY OFF` / `KEY I,""` (clearing the function-key soft-label
850 ' row) and `VIEW PRINT` (scroll-region windowing for the list
860 ' screen) are interpreter/console features BASCAL doesn't
870 ' expose.
880 ' - fhb's own hand-rolled numeric-ERR-code-to-message lookup table
890 ' (ERR=1 "Input value overflow", ERR=2 "Syntax error", ... ERR=25)
900 ' is replaced below by BASCAL's com.bascal.stdlib.error library
910 ' (ERROR$(code%)) -- same idea, BASCAL's own table; it still
920 ' doesn't decode ERL, which errorTrap() reports as the raw line
930 ' number.
940 ' - fhb's one-time "hidden" datafile initializer (PUT-ing 100
950 ' blank, CHR$(255)-flagged records) is reproduced below as
960 ' initializeInventoryFileIfNew(), called once at program entry --
970 ' inven.dat no longer has to be pre-populated by hand.
980 ' - The three original tab-position constants (T=20, U=25,
990 ' V=30) are collapsed into a single `TAB_COL = 20`; a couple of
1000 ' screens that used U=25 in the original (see showAddStockScreen
1010 ' below) keep 25 as a literal rather than reusing TAB_COL.
1020 '
1030 ' Tracks parts in a fixed 100-record file: check status, add,
1040 ' edit, add/subtract stock, and a reorder report.
1050 '
1060 ' Error handling uses try/catch (GitHub issue #60), not the raw `on
1070 ' error goto` / `resume next` fhb's original relies on: a failed menu
1080 ' action is abandoned outright and the program returns straight to the
1090 ' main menu, rather than resuming at the exact instruction after
1100 ' whatever failed -- see reportInventoryError() below and
1110 ' tutorial/inventory_try_catch.draft's own header comment for why. This
1120 ' is a real, deliberate behavior change from an earlier on-error-goto
1130 ' version of this file, which *was* verified against real BASCOM 2.00
1140 ' under dosbox-x (only with the /E and /X switches -- error trapping
1150 ' isn't linked in by default); the try/catch shape below transpiles to
1160 ' the same ON ERROR GOTO/RESUME primitives BASCOM accepts, but hasn't
1170 ' itself been independently re-verified against a real BASCOM compile.
1180 ' ============================================================
1190 ' BASCAL-ism: the record/file DSL. `record ... end record` plus
1200 ' `file ... as ... = open(...)` below replace fhb's manual
1210 ' FIELD #1,1 AS F$,30 AS D$,2 AS Q$,... buffer layout entirely --
1220 ' bcc computes the field widths and record LEN from this
1230 ' declaration and generates the FIELD statement itself. Named
1240 ' field access (`p.flag`, `p.qty`, ...) and whole-record
1250 ' read/write via `inv[n]` (see checkPart() below) replace fhb's
1260 ' manual GET/PUT plus LSET/RSET and MKI$/MKS$/CVI$/CVS$ packing.
1270 ' BASCAL-ism: `const` is a real compile-time constant, not a plain
1280 ' variable assignment like fhb's `N=100` / `T=20` -- it can never
1290 ' be reassigned, and resolves to the same value everywhere,
1300 ' including inside every function/procedure below, with no
1310 ' `global` declaration needed.
1320 CONSTPARTCOUNT% = 100
1330 CONSTTABCOL% = 20
1340 ' `file ... = open(...)` is sugar for OPEN ... FOR RANDOM AS #n
1350 ' LEN = <record width> plus the FIELD statement fhb wrote out by
1360 ' hand at his line 550. Wrapped in its own try/catch: a file that
1370 ' exists but can't be opened for random access (permissions, a
1380 ' read-only inven.dat, disk full on the fallback create) is a real,
1390 ' trappable error (code 75, "Path/File access error") on both
1400 ' targets now, not a hard crash -- report it and exit cleanly
1410 ' instead of leaving the program to fail confusingly the first time
1420 ' something tries to use an `inv` that was never actually opened.
1430 ON ERROR GOTO 1500
1440 BCCTRY0001PENDING% = 0
1450 ' file inv as Part = open(...)  [39 bytes/record]
1460 OPEN "inven.dat" FOR RANDOM AS #1 LEN = 39
1470 FIELD #1, 1 AS invFlagBuf$, 30 AS invDescBuf$, 2 AS invQtyBuf$, 2 AS invReorderBuf$, 4 AS invPriceBuf$
1480 ON ERROR GOTO 0
1490 GOTO 1650
1500 BCCTRY0001PENDING% = ERR
1510 BCCERR% = ERR
1520 BCCERL% = ERL
1530 RESUME 1540
1540 ON ERROR GOTO 1630
1550 errorCode0% = BCCERR%
1560 GOSUB 3300
1570 BCCT2$ = errorResult0$
1580 PRINT "could not open inven.dat: " + BCCT2$
1590 END
1600 BCCTRY0001PENDING% = 0
1610 ON ERROR GOTO 0
1620 GOTO 1650
1630 BCCTRY0001PENDING% = ERR
1640 RESUME 1650
1650 ON ERROR GOTO 0
1660 IF BCCTRY0001PENDING% <> 0 THEN ERROR BCCTRY0001PENDING%
1670 REM END TRY
1680 ' -------------------- Pure functions (no file access) --------------------
1690 ' BASCAL-ism: `function ... end function` with `return` replaces
1700 ' fhb's convention of a GOSUB target plus a bare RETURN -- there's
1710 ' no separate "subroutine label" and no shared/global result
1720 ' variable to manage by hand; `isEmpty%(...)` is called like an
1730 ' ordinary expression at every use below (e.g. `isEmpty%(p.flag)`).
1740 ' A record whose flag byte is CHR$(255) is an empty/never-used slot.
1750 ' BASCAL-ism: `&&` and `||` are short-circuit AND/OR -- real
1760 ' MBASIC/BASCOM only has bitwise AND/OR (which fhb relies on here
1770 ' too, since `PART!<1 OR PART!>N!` never short-circuits anyway).
1780 ' BASCAL lowers `&&`/`||` into the equivalent branching so the
1790 ' short-circuit *is* real at the generated-BASIC level; see the
1800 ' manual's "Short-Circuit && and ||" section
1810 ' (https://johnjoeallen.github.io/bascal/manual/).
1820 ' -------------------- Keyboard input --------------------
1830 ' BASCAL-ism: `do ... loop until` is a structured post-check loop
1840 ' replacing fhb's `730 KP$=INKEY$:IF KP$="" THEN 730` GOTO-polling
1850 ' idiom. `inkey$` itself is the real INKEY$ builtin passed straight
1860 ' through, resolving correctly from inside a function/procedure
1870 ' body like this one -- every menu action below calls
1880 ' readKey$()/waitAnyKey() rather than polling INKEY$ inline.
1890 ' -------------------- Display procedures --------------------
1900 ' BASCAL-ism: no `VIEW PRINT` (see the header note above), so this
1910 ' deliberately does NOT pin a "press any key" line to a fixed row the way
1920 ' fhb's original does -- a bare `LOCATE 25, ...` sitting under content
1930 ' that keeps printing past it (listAll()'s own items) collides with
1940 ' whatever's later written there, since nothing here scrolls a bounded
1950 ' region: waitAnyKey() is the only thing that ever touches row 25, and
1960 ' only right when it actually blocks (see listAll()'s own redraw-per-page
1970 ' structure below).
1980 ' byref scalar parameters: gatherPartDetails writes the four editable
1990 ' fields for a part directly back into the caller's variables.
2000 ' -------------------- Menu actions --------------------
2010 ' fhb's own one-time "hidden" datafile initializer PUT-ing 100 blank,
2020 ' CHR$(255)-flagged records (see the header note above) -- reproduced
2030 ' here so inven.dat no longer has to be pre-populated by hand before
2040 ' running this program. A brand-new file OPEN created just now (rather
2050 ' than one that already existed) reads back as all-zero bytes: record
2060 ' 1's flag byte is CHR$(0), never CHR$(255) -- the one signal an
2070 ' already-populated file (whose record 1 flag is always either
2080 ' CHR$(255), still an empty slot, or a real part's own "1") could never
2090 ' produce, so it's what isEmpty%() itself can't use (see its own
2100 ' header note) but this one-time check safely can.
2110 ' -------------------- Program entry --------------------
2120 CLS
2130 GOSUB 9900
2140     GOSUB 5070
2150     GOSUB 4920
2160     kp$ = readkeyResult0$
2170     IF (INSTR("1234567cCeElLaAsSrRxX", kp$) <> 0) = 0 THEN GOTO 2870
2180         ' BASCAL-ism: `select case` replaces fhb's chain of eight
2190         ' `IF VAL(KP$)=n OR KP$="x" OR KP$="X" THEN GOTO ...` lines
2200         ' (his 770-840) with one multi-way dispatch.
2210         '
2220         ' BASCAL-ism: `try`/`catch` (issue #60) replaces fhb's own global
2230         ' `ON ERROR GOTO` trap. A failed menu action is abandoned outright
2240         ' here -- the `catch` below runs, then execution continues right
2250         ' after `end try`, back at `loop until` -- rather than resuming at
2260         ' the exact instruction after whatever failed inside checkPart()/
2270         ' editRecord()/etc. the way fhb's `RESUME NEXT` did. See
2280         ' reportInventoryError() below and tutorial/inventory_try_catch.
2290         ' draft's own header comment for why that arbitrary resume-point
2300         ' behavior isn't something try/catch reproduces.
2310         ON ERROR GOTO 2710
2320         BCCTRY0004PENDING% = 0
2330         BCCT6$ = kp$
2340         IF (BCCT6$ = "1" OR BCCT6$ = "c" OR BCCT6$ = "C") <> 0 THEN GOTO 2420
2350         IF (BCCT6$ = "2" OR BCCT6$ = "e" OR BCCT6$ = "E") <> 0 THEN GOTO 2440
2360         IF (BCCT6$ = "3" OR BCCT6$ = "l" OR BCCT6$ = "L") <> 0 THEN GOTO 2460
2370         IF (BCCT6$ = "4" OR BCCT6$ = "a" OR BCCT6$ = "A") <> 0 THEN GOTO 2480
2380         IF (BCCT6$ = "5" OR BCCT6$ = "s" OR BCCT6$ = "S") <> 0 THEN GOTO 2500
2390         IF (BCCT6$ = "6" OR BCCT6$ = "r" OR BCCT6$ = "R") <> 0 THEN GOTO 2520
2400         IF (BCCT6$ = "7" OR BCCT6$ = "x" OR BCCT6$ = "X") <> 0 THEN GOTO 2540
2410         GOTO 2680
2420             GOSUB 6580
2430             GOTO 2680
2440             GOSUB 7090
2450             GOTO 2680
2460             GOSUB 7750
2470             GOTO 2680
2480             GOSUB 8120
2490             GOTO 2680
2500             GOSUB 8760
2510             GOTO 2680
2520             GOSUB 9490
2530             GOTO 2680
2540             ' BASCAL-ism: `inv.close()` is sugar for `CLOSE #1`,
2550             ' matching fhb's own `90 CLOSE:SYSTEM`. fhb's original
2560             ' also had a separate "Quit to BASIC" option (his own
2570             ' 7, returning to the interpreter's command prompt
2580             ' rather than exiting to DOS) -- dropped here: a
2590             ' compiled program has no interpreter to return to,
2600             ' so it was never anything but a second syntax for
2610             ' this same close-and-exit action.
2620             ' inv.close()
2630             CLOSE #1
2640             COLOR 7, 0
2650             CLS
2660             SYSTEM
2670             GOTO 2680
2680         REM END SELECT
2690         ON ERROR GOTO 0
2700         GOTO 2840
2710         BCCTRY0004PENDING% = ERR
2720         BCCERR% = ERR
2730         BCCERL% = ERL
2740         RESUME 2750
2750         ON ERROR GOTO 2820
2760         reportinventoryerrorErr0% = BCCERR%
2770         reportinventoryerrorErl0% = BCCERL%
2780         GOSUB 10180
2790         BCCTRY0004PENDING% = 0
2800         ON ERROR GOTO 0
2810         GOTO 2840
2820         BCCTRY0004PENDING% = ERR
2830         RESUME 2840
2840         ON ERROR GOTO 0
2850         IF BCCTRY0004PENDING% <> 0 THEN ERROR BCCTRY0004PENDING%
2860         REM END TRY
2870     REM END IF
2880 GOTO 2140
2890 REM END DO
2900 ' -------------------- Error handling --------------------
2910 ' err%/erl% are ordinary locals scoped to the `catch` block above, not
2920 ' aliases for the ambient (readable-anywhere) `err`/`erl` pseudo-
2930 ' variables `on error goto` uses -- see `Statement::TryCatch`'s own doc
2940 ' comment in ast.rs. Passed straight through to ERROR$ here like fhb's
2950 ' own ERR/ERL (his 3390: "an error on line";ERL), decoded through
2960 ' BASCAL's own com.bascal.stdlib.error (ERROR$) instead of fhb's
2970 ' hand-rolled lookup table -- see the header note above. try/catch
2980 ' itself isn't documented in the manual yet (GitHub issue #60 tracks
2990 ' the still-unfinished C-target work; the manual page can follow once
3000 ' that lands) -- see ast.rs's own `Statement::TryCatch` doc comment for
3010 ' the full semantics meanwhile.
3020 END

3030 ' function ltrim$(self$)
3040     ltrimI0% = 1
3050     IF (ltrimI0% <= LEN(ltrimSelf0$)) = 0 THEN GOTO 3090
3060     IF (MID$(ltrimSelf0$, ltrimI0%, 1) = " ") = 0 THEN GOTO 3090
3070         ltrimI0% = ltrimI0% + 1
3080     GOTO 3050
3090     REM END WHILE
3100     BCCT10$ = ltrimSelf0$
3110     BCCT11% = ltrimI0%
3120     BCCT12$ = MID$(BCCT10$, BCCT11%)
3130     ltrimResult0$ = BCCT12$
3140     RETURN
3150 ' end function ltrim$

3160 ' function rtrim$(self$)
3170     rtrimI0% = LEN(rtrimSelf0$)
3180     IF (rtrimI0% > 0) = 0 THEN GOTO 3220
3190     IF (MID$(rtrimSelf0$, rtrimI0%, 1) = " ") = 0 THEN GOTO 3220
3200         rtrimI0% = rtrimI0% - 1
3210     GOTO 3180
3220     REM END WHILE
3230     BCCT15$ = rtrimSelf0$
3240     BCCT16% = rtrimI0%
3250     BCCT17$ = LEFT$(BCCT15$, BCCT16%)
3260     rtrimResult0$ = BCCT17$
3270     RETURN
3280 ' end function rtrim$

3290 ' function error$(code%)
3300     BCCT19% = errorCode0%
3310     IF (BCCT19% = CONSTERRSYNTAX%) <> 0 THEN GOTO 3650
3320     IF (BCCT19% = CONSTERRRETURNWITHOUTGOSUB%) <> 0 THEN GOTO 3680
3330     IF (BCCT19% = CONSTERROUTOFDATA%) <> 0 THEN GOTO 3710
3340     IF (BCCT19% = CONSTERRILLEGALFUNCTIONCALL%) <> 0 THEN GOTO 3740
3350     IF (BCCT19% = CONSTERROVERFLOW%) <> 0 THEN GOTO 3770
3360     IF (BCCT19% = CONSTERROUTOFMEMORY%) <> 0 THEN GOTO 3800
3370     IF (BCCT19% = CONSTERRSUBSCRIPTOUTOFRANGE%) <> 0 THEN GOTO 3830
3380     IF (BCCT19% = CONSTERRDUPLICATEDEFINITION%) <> 0 THEN GOTO 3860
3390     IF (BCCT19% = CONSTERRDIVISIONBYZERO%) <> 0 THEN GOTO 3890
3400     IF (BCCT19% = CONSTERRTYPEMISMATCH%) <> 0 THEN GOTO 3920
3410     IF (BCCT19% = CONSTERROUTOFSTRINGSPACE%) <> 0 THEN GOTO 3950
3420     IF (BCCT19% = CONSTERRNORESUME%) <> 0 THEN GOTO 3980
3430     IF (BCCT19% = CONSTERRRESUMEWITHOUTERROR%) <> 0 THEN GOTO 4010
3440     IF (BCCT19% = CONSTERRDEVICETIMEOUT%) <> 0 THEN GOTO 4040
3450     IF (BCCT19% = CONSTERRDEVICEFAULT%) <> 0 THEN GOTO 4070
3460     IF (BCCT19% = CONSTERROUTOFPAPER%) <> 0 THEN GOTO 4100
3470     IF (BCCT19% = CONSTERRBADFILENUMBER%) <> 0 THEN GOTO 4130
3480     IF (BCCT19% = CONSTERRFILENOTFOUND%) <> 0 THEN GOTO 4160
3490     IF (BCCT19% = CONSTERRBADFILEMODE%) <> 0 THEN GOTO 4190
3500     IF (BCCT19% = CONSTERRFILEALREADYOPEN%) <> 0 THEN GOTO 4220
3510     IF (BCCT19% = CONSTERRDEVICEIO%) <> 0 THEN GOTO 4250
3520     IF (BCCT19% = CONSTERRFILEALREADYEXISTS%) <> 0 THEN GOTO 4280
3530     IF (BCCT19% = CONSTERRDISKFULL%) <> 0 THEN GOTO 4310
3540     IF (BCCT19% = CONSTERRINPUTPASTEND%) <> 0 THEN GOTO 4340
3550     IF (BCCT19% = CONSTERRBADRECORDNUMBER%) <> 0 THEN GOTO 4370
3560     IF (BCCT19% = CONSTERRBADFILENAME%) <> 0 THEN GOTO 4400
3570     IF (BCCT19% = CONSTERRTOOMANYFILES%) <> 0 THEN GOTO 4430
3580     IF (BCCT19% = CONSTERRDEVICEUNAVAILABLE%) <> 0 THEN GOTO 4460
3590     IF (BCCT19% = CONSTERRDISKWRITEPROTECTED%) <> 0 THEN GOTO 4490
3600     IF (BCCT19% = CONSTERRDISKNOTREADY%) <> 0 THEN GOTO 4520
3610     IF (BCCT19% = CONSTERRDISKMEDIAERROR%) <> 0 THEN GOTO 4550
3620     IF (BCCT19% = CONSTERRPATHFILEACCESS%) <> 0 THEN GOTO 4580
3630     IF (BCCT19% = CONSTERRPATHNOTFOUND%) <> 0 THEN GOTO 4610
3640     GOTO 4640
3650         errorResult0$ = "Syntax error"
3660         RETURN
3670         GOTO 4680
3680         errorResult0$ = "RETURN without GOSUB"
3690         RETURN
3700         GOTO 4680
3710         errorResult0$ = "Out of DATA"
3720         RETURN
3730         GOTO 4680
3740         errorResult0$ = "Illegal function call"
3750         RETURN
3760         GOTO 4680
3770         errorResult0$ = "Overflow"
3780         RETURN
3790         GOTO 4680
3800         errorResult0$ = "Out of memory"
3810         RETURN
3820         GOTO 4680
3830         errorResult0$ = "Subscript out of range"
3840         RETURN
3850         GOTO 4680
3860         errorResult0$ = "Duplicate Definition"
3870         RETURN
3880         GOTO 4680
3890         errorResult0$ = "Division by zero"
3900         RETURN
3910         GOTO 4680
3920         errorResult0$ = "Type mismatch"
3930         RETURN
3940         GOTO 4680
3950         errorResult0$ = "Out of string space"
3960         RETURN
3970         GOTO 4680
3980         errorResult0$ = "No RESUME"
3990         RETURN
4000         GOTO 4680
4010         errorResult0$ = "RESUME without error"
4020         RETURN
4030         GOTO 4680
4040         errorResult0$ = "Device timeout"
4050         RETURN
4060         GOTO 4680
4070         errorResult0$ = "Device fault"
4080         RETURN
4090         GOTO 4680
4100         errorResult0$ = "Out of paper"
4110         RETURN
4120         GOTO 4680
4130         errorResult0$ = "Bad file number"
4140         RETURN
4150         GOTO 4680
4160         errorResult0$ = "File not found"
4170         RETURN
4180         GOTO 4680
4190         errorResult0$ = "Bad file mode"
4200         RETURN
4210         GOTO 4680
4220         errorResult0$ = "File already open"
4230         RETURN
4240         GOTO 4680
4250         errorResult0$ = "Device I/O error"
4260         RETURN
4270         GOTO 4680
4280         errorResult0$ = "File already exists"
4290         RETURN
4300         GOTO 4680
4310         errorResult0$ = "Disk full"
4320         RETURN
4330         GOTO 4680
4340         errorResult0$ = "Input past end"
4350         RETURN
4360         GOTO 4680
4370         errorResult0$ = "Bad record number"
4380         RETURN
4390         GOTO 4680
4400         errorResult0$ = "Bad file name"
4410         RETURN
4420         GOTO 4680
4430         errorResult0$ = "Too many files"
4440         RETURN
4450         GOTO 4680
4460         errorResult0$ = "Device unavailable"
4470         RETURN
4480         GOTO 4680
4490         errorResult0$ = "Disk write protected"
4500         RETURN
4510         GOTO 4680
4520         errorResult0$ = "Disk not ready"
4530         RETURN
4540         GOTO 4680
4550         errorResult0$ = "Disk media error"
4560         RETURN
4570         GOTO 4680
4580         errorResult0$ = "Path/File access error"
4590         RETURN
4600         GOTO 4680
4610         errorResult0$ = "Path not found"
4620         RETURN
4630         GOTO 4680
4640         BCCT20% = errorCode0%
4650         BCCT21$ = STR$(BCCT20%)
4660         errorResult0$ = "Error " + BCCT21$
4670         RETURN
4680     REM END SELECT
4690     RETURN
4700 ' end function error$

4710 ' function isempty%(flag$)
4720     BCCT22$ = isemptyFlag0$
4730     BCCT23% = ASC(BCCT22$)
4740     isemptyResult0% = BCCT23% = 255
4750     RETURN
4760 ' end function isempty%

4770 ' function partinrange%(n%)
4780     IF (partinrangeN0% >= 1) = 0 THEN GOTO 4820
4790     IF (partinrangeN0% <= CONSTPARTCOUNT%) = 0 THEN GOTO 4820
4800         partinrangeResult0% = 1
4810         RETURN
4820     REM END IF
4830     partinrangeResult0% = 0
4840     RETURN
4850 ' end function partinrange%

4860 ' function readpartnumberinput$()
4870     INPUT "Input part number"; readpartnumberinputS0$
4880     readpartnumberinputResult0$ = readpartnumberinputS0$
4890     RETURN
4900 ' end function readpartnumberinput$

4910 ' function readkey$()
4920         readkeyK0$ = INKEY$
4930         IF (readkeyK0$ <> "") = 0 THEN GOTO 4920
4940     REM END DO
4950     readkeyResult0$ = readkeyK0$
4960     RETURN
4970 ' end function readkey$

4980 ' procedure waitanykey()
4990     LOCATE 25, 10
5000     PRINT "Press the AnyKey to continue...";
5010         waitanykeyK0$ = INKEY$
5020         IF (waitanykeyK0$ <> "") = 0 THEN GOTO 5010
5030     REM END DO
5040     RETURN
5050 ' end procedure waitanykey

5060 ' procedure showmainmenu()
5070     CLS
5080     COLOR 14, 4
5090     CLS
5100     LOCATE 6, 1
5110     PRINT
5120     ' `tab(n)` passes straight through to real TAB(n), same as
5130     ' fhb's own `PRINT TAB(V) "..."` -- but only as a bare item in
5140     ' a PRINT list, juxtaposed or `;`-separated like here. Real
5150     ' BASCOM rejects `"literal" + tab(n) + ...` (TAB isn't a real
5160     ' string function you can concatenate); see printListHeader()
5170     ' and printReorderHeader() below, which need `;` between a
5180     ' preceding string and a `tab(n)` for exactly this reason.
5190     PRINT TAB(30)"Inventory Program"
5200     PRINT
5210     PRINT TAB(CONSTTABCOL%)"1......C)heck a part"
5220     PRINT TAB(CONSTTABCOL%)"2......E)dit/overwrite/add a part"
5230     PRINT TAB(CONSTTABCOL%)("3......L)ist all" + STR$(CONSTPARTCOUNT%)) + "parts"
5240     PRINT TAB(CONSTTABCOL%)"4......A)dd stock"
5250     PRINT TAB(CONSTTABCOL%)"5......S)ubtract stock"
5260     PRINT TAB(CONSTTABCOL%)"6......R)eorder Report"
5270     PRINT
5280     PRINT TAB(CONSTTABCOL%)"7......eX)it to system"
5290     RETURN
5300 ' end procedure showmainmenu

5310 ' procedure showbadpartnumber()
5320     CLS
5330     LOCATE 10, 10
5340     PRINT "Part number is out of permissable range of 1 to" + STR$(CONSTPARTCOUNT%)
5350     RETURN
5360 ' end procedure showbadpartnumber

5370 ' procedure showrangeretrymessage()
5380     LOCATE 10, 15
5390     PRINT "The Part number is out of permissable range of 1 to" + STR$(CONSTPARTCOUNT%)
5400     LOCATE 25, 15
5410     PRINT "Press the Anykey to reenter part number...";
5420     RETURN
5430 ' end procedure showrangeretrymessage

5440 ' procedure shownullentrymessage(partStr$)
5450     LOCATE 10, CONSTTABCOL%
5460     PRINT ("Part number " + shownullentrymessagePartStr0$) + " is a null entry"
5470     RETURN
5480 ' end procedure shownullentrymessage

5490 ' procedure showpartstatus(partNum%, desc$, qty%, reorder%, price!)
5500     CLS
5510     LOCATE 5, 1
5520     PRINT TAB(CONSTTABCOL%)"Inventory Status for Individual Part Number"
5530     PRINT TAB(CONSTTABCOL%)"==========================================="
5540     PRINT
5550     PRINT
5560     PRINT TAB(CONSTTABCOL%)"     Part number:  " + STR$(showpartstatusPartNum0%)
5570     PRINT
5580     PRINT TAB(CONSTTABCOL%)"       Item name:  " + showpartstatusDesc0$
5590     PRINT TAB(CONSTTABCOL%)"Quantity on hand:  " + STR$(showpartstatusQty0%)
5600     PRINT TAB(CONSTTABCOL%)"   Reorder level:  " + STR$(showpartstatusReorder0%)
5610     PRINT TAB(CONSTTABCOL%)"      Unit price:  " + STR$(showpartstatusPrice0!)
5620     RETURN
5630 ' end procedure showpartstatus

5640 ' procedure printlistheader()
5650     CLS
5660     PRINT TAB(25)"I N V E N T O R Y   L I S T I N G"; TAB(65); STR$(CONSTPARTCOUNT%) + "items"
5670     PRINT "                                          Quantity       Reorder"
5680     PRINT " Partno           Description             on hand         level"
5690     RETURN
5700 ' end procedure printlistheader

5710 ' procedure printinventoryline(partNum%, desc$, qty%, reorder%)
5720     PRINT (((((STR$(printinventorylinePartNum0%) + "  ") + printinventorylineDesc0$) + "   ") + STR$(printinventorylineQty0%)) + "          ") + STR$(printinventorylineReorder0%)
5730     RETURN
5740 ' end procedure printinventoryline

5750 ' procedure printreorderheader()
5760     CLS
5770     LOCATE 1, CONSTTABCOL%
5780     PRINT "Reorder Report"; TAB(55); DATE$
5790     PRINT
5800     PRINT "                                             Quantity       Reorder"
5810     PRINT "    Partno           Description             on hand         level"
5820     PRINT "   =======  ==============================   ========       ======="
5830     RETURN
5840 ' end procedure printreorderheader

5850 ' procedure printreorderline(partNum%, desc$, qty%, reorder%)
5860     PRINT (((((("  " + STR$(printreorderlinePartNum0%)) + "  ") + printreorderlineDesc0$) + "   ") + STR$(printreorderlineQty0%)) + "          ") + STR$(printreorderlineReorder0%)
5870     RETURN
5880 ' end procedure printreorderline

5890 ' procedure gatherpartdetails(partNum%, desc$, qty%, reorder%, price!)
5900     CLS
5910     LOCATE 4, CONSTTABCOL%
5920     PRINT "Adding or Overwriting a Record"
5930     LOCATE 8, CONSTTABCOL%
5940     PRINT "Record/Partno" + STR$(gatherpartdetailsPartNum0%)
5950     LOCATE 11, 39
5960     PRINT "------------------------------"
5970     LOCATE 10, CONSTTABCOL%
5980     INPUT "      Description"; gatherpartdetailsDesc0$
5990     LOCATE 12, CONSTTABCOL%
6000     INPUT "Quantity in stock"; gatherpartdetailsQty0%
6010     LOCATE 14, CONSTTABCOL%
6020     INPUT "    Reorder level"; gatherpartdetailsReorder0%
6030     LOCATE 16, CONSTTABCOL%
6040     INPUT "       Unit price"; gatherpartdetailsPrice0!
6050     LOCATE 18, CONSTTABCOL%
6060     PRINT "Is information correct (Y/N)?"
6070     RETURN
6080 ' end procedure gatherpartdetails

6090 ' procedure showaddstockscreen(partNum%, desc$, qty%, reorder%)
6100     CLS
6110     LOCATE 4, 25
6120     PRINT "Add to an inventory part number"
6130     LOCATE 5, 25
6140     PRINT "==============================="
6150     LOCATE 8, CONSTTABCOL%
6160     PRINT "     Part number: " + STR$(showaddstockscreenPartNum0%)
6170     LOCATE 9, CONSTTABCOL%
6180     PRINT "Item description: " + showaddstockscreenDesc0$
6190     LOCATE 10, CONSTTABCOL%
6200     PRINT "Quantity on hand: " + STR$(showaddstockscreenQty0%)
6210     LOCATE 11, CONSTTABCOL%
6220     PRINT "   Reorder Level: " + STR$(showaddstockscreenReorder0%)
6230     RETURN
6240 ' end procedure showaddstockscreen

6250 ' procedure shownegativeqtywarning()
6260     LOCATE 17, 15
6270     PRINT "The quantity to add must NOT be a negative number"
6280     LOCATE 25, 1
6290     PRINT "Please press the Anykey to reenter quantity to add...";
6300     RETURN
6310 ' end procedure shownegativeqtywarning

6320 ' procedure showsubtractstockscreen(partNum%, desc$, qty%, reorder%)
6330     CLS
6340     LOCATE 4, CONSTTABCOL%
6350     PRINT "Subtract an inventory part number"
6360     LOCATE 5, CONSTTABCOL%
6370     PRINT "================================="
6380     LOCATE 8, CONSTTABCOL%
6390     PRINT "         Part number: " + STR$(showsubtractstockscreenPartNum0%)
6400     LOCATE 9, CONSTTABCOL%
6410     PRINT "    Item description: " + showsubtractstockscreenDesc0$
6420     LOCATE 10, CONSTTABCOL%
6430     PRINT "    Quantity on hand: " + STR$(showsubtractstockscreenQty0%)
6440     LOCATE 11, CONSTTABCOL%
6450     PRINT "       Reorder Level: " + STR$(showsubtractstockscreenReorder0%)
6460     RETURN
6470 ' end procedure showsubtractstockscreen

6480 ' procedure showoversubtractwarning(onHand%)
6490     LOCATE 17, 5
6500     PRINT "The quantity to SUBTRACT must NOT result in NEGATIVE inventory"
6510     LOCATE 18, 5
6520     PRINT ("Only" + STR$(showoversubtractwarningOnHand0%)) + " IN STOCK"
6530     LOCATE 25, 1
6540     PRINT "Please press the Anykey to reenter quantity to subtract...";
6550     RETURN
6560 ' end procedure showoversubtractwarning

6570 ' procedure checkpart()
6580     ' global inv
6590     GOSUB 4870
6600     checkpartPartStr0$ = readpartnumberinputResult0$
6610     checkpartPart0% = VAL(checkpartPartStr0$)
6620     partinrangeN0% = checkpartPart0%
6630     GOSUB 4780
6640     BCCT28% = partinrangeResult0%
6650     IF (BCCT28% = 0) = 0 THEN GOTO 6690
6660         GOSUB 5320
6670         GOSUB 4990
6680         RETURN
6690     REM END IF
6700     ' BASCAL-ism: `let p = inv[part%]` reads record `part%` of the
6710     ' `inv` file into a local record variable `p` -- one expression
6720     ' for what fhb's `GET #1, PART!` plus five separate field reads
6730     ' (F$, D$, CVI(Q$), CVI(R$), CVS(P$)) did by hand. The write
6740     ' side, `inv[part%] = { ... }` (see editRecord() below), is the
6750     ' same sugar for PUT plus the LSET/MKx$ packing it replaces.
6760     ' let p = inv[...]  (whole-record read)
6770     GET #1, checkpartPart0%
6780     rtrimSelf0$ = invFlagBuf$
6790     GOSUB 3170
6800     BCCT30$ = rtrimResult0$
6810     checkpartPFlag0$ = BCCT30$
6820     rtrimSelf0$ = invDescBuf$
6830     GOSUB 3170
6840     BCCT31$ = rtrimResult0$
6850     checkpartPDesc0$ = BCCT31$
6860     checkpartPQty0% = CVI(invQtyBuf$)
6870     checkpartPReorder0% = CVI(invReorderBuf$)
6880     checkpartPPrice0! = CVS(invPriceBuf$)
6890     isemptyFlag0$ = checkpartPFlag0$
6900     GOSUB 4720
6910     BCCT32% = isemptyResult0%
6920     IF (BCCT32%) = 0 THEN GOTO 6980
6930         CLS
6940         LOCATE 10, 18
6950         PRINT ("Part number" + STR$(checkpartPart0%)) + "is still a null entry at this time"
6960         GOSUB 4990
6970         RETURN
6980     REM END IF
6990     showpartstatusPartNum0% = checkpartPart0%
7000     showpartstatusDesc0$ = checkpartPDesc0$
7010     showpartstatusQty0% = checkpartPQty0%
7020     showpartstatusReorder0% = checkpartPReorder0%
7030     showpartstatusPrice0! = checkpartPPrice0!
7040     GOSUB 5500
7050     GOSUB 4990
7060     RETURN
7070 ' end procedure checkpart

7080 ' procedure editrecord()
7090     ' global inv
7100     CLS
7110     LOCATE 10, CONSTTABCOL%
7120     GOSUB 4870
7130     editrecordPartStr0$ = readpartnumberinputResult0$
7140     editrecordPart0% = VAL(editrecordPartStr0$)
7150     partinrangeN0% = editrecordPart0%
7160     GOSUB 4780
7170     BCCT34% = partinrangeResult0%
7180     IF (BCCT34% = 0) = 0 THEN GOTO 7220
7190         GOSUB 5320
7200         GOSUB 4990
7210         RETURN
7220     REM END IF
7230     ' let p = inv[...]  (whole-record read)
7240     GET #1, editrecordPart0%
7250     rtrimSelf0$ = invFlagBuf$
7260     GOSUB 3170
7270     BCCT36$ = rtrimResult0$
7280     editrecordPFlag0$ = BCCT36$
7290     rtrimSelf0$ = invDescBuf$
7300     GOSUB 3170
7310     BCCT37$ = rtrimResult0$
7320     editrecordPDesc0$ = BCCT37$
7330     editrecordPQty0% = CVI(invQtyBuf$)
7340     editrecordPReorder0% = CVI(invReorderBuf$)
7350     editrecordPPrice0! = CVS(invPriceBuf$)
7360     isemptyFlag0$ = editrecordPFlag0$
7370     GOSUB 4720
7380     BCCT38% = isemptyResult0%
7390     IF (BCCT38% = 0) = 0 THEN GOTO 7480
7400         LOCATE 12, CONSTTABCOL%
7410         PRINT "Overwrite existing part data?"
7420         GOSUB 4920
7430         editrecordKp0$ = readkeyResult0$
7440         IF (editrecordKp0$ <> "Y") = 0 THEN GOTO 7470
7450         IF (editrecordKp0$ <> "y") = 0 THEN GOTO 7470
7460             RETURN
7470         REM END IF
7480     REM END IF
7490         gatherpartdetailsPartNum0% = editrecordPart0%
7500         gatherpartdetailsDesc0$ = editrecordEditDesc0$
7510         gatherpartdetailsQty0% = editrecordEditQty0%
7520         gatherpartdetailsReorder0% = editrecordEditReorder0%
7530         gatherpartdetailsPrice0! = editrecordEditPrice0!
7540         GOSUB 5900
7550         editrecordEditDesc0$ = gatherpartdetailsDesc0$
7560         editrecordEditQty0% = gatherpartdetailsQty0%
7570         editrecordEditReorder0% = gatherpartdetailsReorder0%
7580         editrecordEditPrice0! = gatherpartdetailsPrice0!
7590         GOSUB 4920
7600         editrecordKp0$ = readkeyResult0$
7610         IF (editrecordKp0$ = "Y") <> 0 THEN GOTO 7640
7620         IF (editrecordKp0$ = "y") <> 0 THEN GOTO 7640
7630         GOTO 7490
7640     REM END DO
7650     ' inv[...] = { ... }  (whole-record write)
7660     LSET invFlagBuf$ = "1"
7670     LSET invDescBuf$ = editrecordEditDesc0$
7680     LSET invQtyBuf$ = MKI$(editrecordEditQty0%)
7690     LSET invReorderBuf$ = MKI$(editrecordEditReorder0%)
7700     LSET invPriceBuf$ = MKS$(editrecordEditPrice0!)
7710     PUT #1, editrecordPart0%
7720     RETURN
7730 ' end procedure editrecord

7740 ' procedure listall()
7750     ' global inv
7760     GOSUB 5650
7770     listallScrollCount0% = 0
7780     FOR listallI0% = 1 TO CONSTPARTCOUNT%
7790         ' let p = inv[...]  (whole-record read)
7800         GET #1, listallI0%
7810         rtrimSelf0$ = invFlagBuf$
7820         GOSUB 3170
7830         BCCT45$ = rtrimResult0$
7840         listallPFlag0$ = BCCT45$
7850         rtrimSelf0$ = invDescBuf$
7860         GOSUB 3170
7870         BCCT46$ = rtrimResult0$
7880         listallPDesc0$ = BCCT46$
7890         listallPQty0% = CVI(invQtyBuf$)
7900         listallPReorder0% = CVI(invReorderBuf$)
7910         listallPPrice0! = CVS(invPriceBuf$)
7920         printinventorylinePartNum0% = listallI0%
7930         printinventorylineDesc0$ = listallPDesc0$
7940         printinventorylineQty0% = listallPQty0%
7950         printinventorylineReorder0% = listallPReorder0%
7960         GOSUB 5720
7970         listallScrollCount0% = listallScrollCount0% + 1
7980         IF (listallScrollCount0% = 20) = 0 THEN GOTO 8070
7990             GOSUB 4990
8000             listallScrollCount0% = 0
8010             ' Redraw for the next page rather than let it keep scrolling
8020             ' past row 25 -- see printListHeader()'s own note on why a
8030             ' fixed-row prompt can't coexist with unbounded scrolling.
8040             IF (listallI0% < CONSTPARTCOUNT%) = 0 THEN GOTO 8060
8050                 GOSUB 5650
8060             REM END IF
8070         REM END IF
8080     NEXT listallI0%
8090     RETURN
8100 ' end procedure listall

8110 ' procedure addstock()
8120     ' global inv
8130     CLS
8140     LOCATE 5, 25
8150     PRINT "A D D I N G   S T O C K"
8160         LOCATE 8, 25
8170         GOSUB 4870
8180         addstockPartStr0$ = readpartnumberinputResult0$
8190         addstockPart0% = VAL(addstockPartStr0$)
8200         partinrangeN0% = addstockPart0%
8210         GOSUB 4780
8220         addstockValidPart0% = partinrangeResult0%
8230         IF (addstockValidPart0% = 0) = 0 THEN GOTO 8260
8240             GOSUB 5380
8250             GOSUB 4920
8260         REM END IF
8270         IF (addstockValidPart0% <> 0) = 0 THEN GOTO 8160
8280     REM END DO
8290     ' let p = inv[...]  (whole-record read)
8300     GET #1, addstockPart0%
8310     rtrimSelf0$ = invFlagBuf$
8320     GOSUB 3170
8330     BCCT51$ = rtrimResult0$
8340     addstockPFlag0$ = BCCT51$
8350     rtrimSelf0$ = invDescBuf$
8360     GOSUB 3170
8370     BCCT52$ = rtrimResult0$
8380     addstockPDesc0$ = BCCT52$
8390     addstockPQty0% = CVI(invQtyBuf$)
8400     addstockPReorder0% = CVI(invReorderBuf$)
8410     addstockPPrice0! = CVS(invPriceBuf$)
8420     isemptyFlag0$ = addstockPFlag0$
8430     GOSUB 4720
8440     BCCT53% = isemptyResult0%
8450     IF (BCCT53%) = 0 THEN GOTO 8500
8460         shownullentrymessagePartStr0$ = addstockPartStr0$
8470         GOSUB 5450
8480         GOSUB 4920
8490         RETURN
8500     REM END IF
8510         showaddstockscreenPartNum0% = addstockPart0%
8520         showaddstockscreenDesc0$ = addstockPDesc0$
8530         showaddstockscreenQty0% = addstockPQty0%
8540         showaddstockscreenReorder0% = addstockPReorder0%
8550         GOSUB 6100
8560         LOCATE 14, CONSTTABCOL%
8570         INPUT " Quantity to add"; addstockAddStr0$
8580         addstockAddAmt0% = VAL(addstockAddStr0$)
8590         IF (addstockAddAmt0% < 0) = 0 THEN GOTO 8620
8600             GOSUB 6260
8610             GOSUB 4920
8620         REM END IF
8630         IF (addstockAddAmt0% >= 0) = 0 THEN GOTO 8510
8640     REM END DO
8650     addstockPQty0% = addstockPQty0% + addstockAddAmt0%
8660     ' inv[...] = p  (write back a let-bound record)
8670     LSET invFlagBuf$ = addstockPFlag0$
8680     LSET invDescBuf$ = addstockPDesc0$
8690     LSET invQtyBuf$ = MKI$(addstockPQty0%)
8700     LSET invReorderBuf$ = MKI$(addstockPReorder0%)
8710     LSET invPriceBuf$ = MKS$(addstockPPrice0!)
8720     PUT #1, addstockPart0%
8730     RETURN
8740 ' end procedure addstock

8750 ' procedure subtractstock()
8760     ' global inv
8770     CLS
8780     LOCATE 5, 20
8790     PRINT "S U B T R A C T I N G    S T O C K"
8800         LOCATE 8, 25
8810         GOSUB 4870
8820         subtractstockPartStr0$ = readpartnumberinputResult0$
8830         subtractstockPart0% = VAL(subtractstockPartStr0$)
8840         partinrangeN0% = subtractstockPart0%
8850         GOSUB 4780
8860         subtractstockValidPart0% = partinrangeResult0%
8870         IF (subtractstockValidPart0% = 0) = 0 THEN GOTO 8900
8880             GOSUB 5380
8890             GOSUB 4920
8900         REM END IF
8910         IF (subtractstockValidPart0% <> 0) = 0 THEN GOTO 8800
8920     REM END DO
8930     ' let p = inv[...]  (whole-record read)
8940     GET #1, subtractstockPart0%
8950     rtrimSelf0$ = invFlagBuf$
8960     GOSUB 3170
8970     BCCT59$ = rtrimResult0$
8980     subtractstockPFlag0$ = BCCT59$
8990     rtrimSelf0$ = invDescBuf$
9000     GOSUB 3170
9010     BCCT60$ = rtrimResult0$
9020     subtractstockPDesc0$ = BCCT60$
9030     subtractstockPQty0% = CVI(invQtyBuf$)
9040     subtractstockPReorder0% = CVI(invReorderBuf$)
9050     subtractstockPPrice0! = CVS(invPriceBuf$)
9060     isemptyFlag0$ = subtractstockPFlag0$
9070     GOSUB 4720
9080     BCCT61% = isemptyResult0%
9090     IF (BCCT61%) = 0 THEN GOTO 9140
9100         shownullentrymessagePartStr0$ = subtractstockPartStr0$
9110         GOSUB 5450
9120         GOSUB 4920
9130         RETURN
9140     REM END IF
9150         showsubtractstockscreenPartNum0% = subtractstockPart0%
9160         showsubtractstockscreenDesc0$ = subtractstockPDesc0$
9170         showsubtractstockscreenQty0% = subtractstockPQty0%
9180         showsubtractstockscreenReorder0% = subtractstockPReorder0%
9190         GOSUB 6330
9200         LOCATE 14, CONSTTABCOL%
9210         INPUT "Quantity to subtract"; subtractstockSubStr0$
9220         subtractstockSubAmt0% = VAL(subtractstockSubStr0$)
9230         subtractstockOverSubtract0% = 0
9240         IF (subtractstockSubAmt0% >= 0) = 0 THEN GOTO 9300
9250         IF ((subtractstockPQty0% - subtractstockSubAmt0%) < 0) = 0 THEN GOTO 9300
9260             subtractstockOverSubtract0% = 1
9270             showoversubtractwarningOnHand0% = subtractstockPQty0%
9280             GOSUB 6490
9290             GOSUB 4920
9300         REM END IF
9310         IF (subtractstockSubAmt0% >= 0) = 0 THEN GOTO 9150
9320         IF (subtractstockOverSubtract0% = 0) = 0 THEN GOTO 9150
9330     REM END DO
9340     subtractstockPQty0% = subtractstockPQty0% - subtractstockSubAmt0%
9350     IF (subtractstockPQty0% <= subtractstockPReorder0%) = 0 THEN GOTO 9370
9360         LOCATE 16, CONSTTABCOL%
9370     REM END IF
9380     PRINT (("quantity now" + STR$(subtractstockPQty0%)) + " reorder level") + STR$(subtractstockPReorder0%)
9390     ' inv[...] = p  (write back a let-bound record)
9400     LSET invFlagBuf$ = subtractstockPFlag0$
9410     LSET invDescBuf$ = subtractstockPDesc0$
9420     LSET invQtyBuf$ = MKI$(subtractstockPQty0%)
9430     LSET invReorderBuf$ = MKI$(subtractstockPReorder0%)
9440     LSET invPriceBuf$ = MKS$(subtractstockPPrice0!)
9450     PUT #1, subtractstockPart0%
9460     RETURN
9470 ' end procedure subtractstock

9480 ' procedure reorderreport()
9490     ' global inv
9500     GOSUB 5760
9510     reorderreportReportLineCount0% = 0
9520     FOR reorderreportI0% = 1 TO CONSTPARTCOUNT%
9530         ' let p = inv[...]  (whole-record read)
9540         GET #1, reorderreportI0%
9550         rtrimSelf0$ = invFlagBuf$
9560         GOSUB 3170
9570         BCCT69$ = rtrimResult0$
9580         reorderreportPFlag0$ = BCCT69$
9590         rtrimSelf0$ = invDescBuf$
9600         GOSUB 3170
9610         BCCT70$ = rtrimResult0$
9620         reorderreportPDesc0$ = BCCT70$
9630         reorderreportPQty0% = CVI(invQtyBuf$)
9640         reorderreportPReorder0% = CVI(invReorderBuf$)
9650         reorderreportPPrice0! = CVS(invPriceBuf$)
9660         IF (reorderreportPQty0% < reorderreportPReorder0%) = 0 THEN GOTO 9840
9670             printreorderlinePartNum0% = reorderreportI0%
9680             printreorderlineDesc0$ = reorderreportPDesc0$
9690             printreorderlineQty0% = reorderreportPQty0%
9700             printreorderlineReorder0% = reorderreportPReorder0%
9710             GOSUB 5860
9720             reorderreportReportLineCount0% = reorderreportReportLineCount0% + 1
9730             IF (reorderreportReportLineCount0% > 15) = 0 THEN GOTO 9830
9740                 GOSUB 4990
9750                 reorderreportReportLineCount0% = 0
9760                 ' Redraw for the next page rather than let it keep
9770                 ' scrolling past row 25 -- see printListHeader()'s own
9780                 ' note (same underlying issue, same fix) on why a
9790                 ' fixed-row prompt can't coexist with unbounded scrolling.
9800                 IF (reorderreportI0% < CONSTPARTCOUNT%) = 0 THEN GOTO 9820
9810                     GOSUB 5760
9820                 REM END IF
9830             REM END IF
9840         REM END IF
9850     NEXT reorderreportI0%
9860     GOSUB 4990
9870     RETURN
9880 ' end procedure reorderreport

9890 ' procedure initializeinventoryfileifnew()
9900     ' global inv
9910     ' let p = inv[...]  (whole-record read)
9920     GET #1, 1
9930     rtrimSelf0$ = invFlagBuf$
9940     GOSUB 3170
9950     BCCT74$ = rtrimResult0$
9960     initializeinventoryfileifnewPFlag0$ = BCCT74$
9970     rtrimSelf0$ = invDescBuf$
9980     GOSUB 3170
9990     BCCT75$ = rtrimResult0$
10000     initializeinventoryfileifnewPDesc0$ = BCCT75$
10010     initializeinventoryfileifnewPQty0% = CVI(invQtyBuf$)
10020     initializeinventoryfileifnewPReorder0% = CVI(invReorderBuf$)
10030     initializeinventoryfileifnewPPrice0! = CVS(invPriceBuf$)
10040     IF (ASC(initializeinventoryfileifnewPFlag0$) = 0) = 0 THEN GOTO 10140
10050         FOR initializeinventoryfileifnewI0% = 1 TO CONSTPARTCOUNT%
10060             ' inv[...] = { ... }  (whole-record write)
10070             LSET invFlagBuf$ = CHR$(255)
10080             LSET invDescBuf$ = ""
10090             LSET invQtyBuf$ = MKI$(0)
10100             LSET invReorderBuf$ = MKI$(0)
10110             LSET invPriceBuf$ = MKS$(0)
10120             PUT #1, initializeinventoryfileifnewI0%
10130         NEXT initializeinventoryfileifnewI0%
10140     REM END IF
10150     RETURN
10160 ' end procedure initializeinventoryfileifnew

10170 ' procedure reportinventoryerror(err%, erl%)
10180     LOCATE 25, 1
10190     BCCT78% = reportinventoryerrorErl0%
10200     BCCT79$ = STR$(BCCT78%)
10210     errorCode0% = reportinventoryerrorErr0%
10220     GOSUB 3300
10230     BCCT80$ = errorResult0$
10240     PRINT (("There has been an error on line" + BCCT79$) + ": ") + BCCT80$
10250     GOSUB 4920
10260     reportinventoryerrorK0$ = readkeyResult0$
10270     RETURN
10280 ' end procedure reportinventoryerror

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/inventory.c</code></summary>



```c

// BASCAL generated C -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
#include <stdio.h>
#include <math.h>
#include <string.h>
#include <stdint.h>
#include <stdlib.h>
#include <time.h>
#if defined(_WIN32)
#include <conio.h>
#include <windows.h>
#include <io.h>
#else
#include <termios.h>
#include <unistd.h>
#endif

#define BCC_STRBUF_COUNT 8
static char bcc_strbuf[BCC_STRBUF_COUNT][256];
static int bcc_strbuf_next = 0;

static int bcc_err = 0;
static int bcc_on_error_target = -1;
static int bcc_in_handler = 0;
static int bcc_resume_id = -1;
static int bcc_erl = 0;
static const char *bcc_err_file = "";

#define BCC_MAX_CHANNELS 32
static FILE* bcc_files[BCC_MAX_CHANNELS];

static char bcc_file_field_buf[256];

static char bcc_input_buf[256];

static char* bcc_strbuf_take(void);
static const char* bcc_mid(const char* s, int start, int length);
static const char* bcc_chr(int code);
static const char* bcc_stri(int value);
static const char* bcc_strd(double value);
static int bcc_instr(const char* s, const char* needle);
static const char* bcc_inkey(void);
static const char* bcc_date(void);
static void bcc_read_string_field(char* field, const unsigned char* source, size_t width);
static void bcc_mki(char* out, int value);
static void bcc_mkl(char* out, int value);
static int bcc_cvi(const char* s);
static int bcc_cvl(const char* s);
static int bcc_read_record(FILE* file, void* buffer, size_t reclen, long record);
static void bcc_write_record(FILE* file, const void* buffer, size_t reclen, long record);
static void bcc_pad_string_field(unsigned char* dest, const char* value, size_t width);
static int bcc_put_record_part(FILE* file, long record, const char* field_0, const char* field_1, const int16_t* field_2, const int16_t* field_3, const float* field_4);
static int bcc_get_record_part(FILE* file, long record, char* field_0, char* field_1, char* field_2, char* field_3, char* field_4);
static void bcc_mks(char* out, double value);
static void bcc_mkd(char* out, double value);
static float bcc_cvs(const char* s);
static double bcc_cvd(const char* s);
static int bcc_eof(FILE* file);
static void bcc_line_input_file(FILE* file, char* buf, size_t bufsize);
static void bcc_read_file_field(FILE* file, char* buf, size_t bufsize);
static void bcc_color(int fg, int bg);
static void bcc_read_line(void);

static int bv_i_erl = 0;
static int bv_i_err = 0;
static int bv_i_err_bad_file_mode = 0;
static int bv_i_err_bad_file_name = 0;
static int bv_i_err_bad_file_number = 0;
static int bv_i_err_bad_record_number = 0;
static int bv_i_err_device_fault = 0;
static int bv_i_err_device_io = 0;
static int bv_i_err_device_timeout = 0;
static int bv_i_err_device_unavailable = 0;
static int bv_i_err_disk_full = 0;
static int bv_i_err_disk_media_error = 0;
static int bv_i_err_disk_not_ready = 0;
static int bv_i_err_disk_write_protected = 0;
static int bv_i_err_division_by_zero = 0;
static int bv_i_err_duplicate_definition = 0;
static int bv_i_err_file_already_exists = 0;
static int bv_i_err_file_already_open = 0;
static int bv_i_err_file_not_found = 0;
static int bv_i_err_illegal_function_call = 0;
static int bv_i_err_input_past_end = 0;
static int bv_i_err_no_resume = 0;
static int bv_i_err_out_of_data = 0;
static int bv_i_err_out_of_memory = 0;
static int bv_i_err_out_of_paper = 0;
static int bv_i_err_out_of_string_space = 0;
static int bv_i_err_overflow = 0;
static int bv_i_err_path_file_access = 0;
static int bv_i_err_path_not_found = 0;
static int bv_i_err_resume_without_error = 0;
static int bv_i_err_return_without_gosub = 0;
static int bv_i_err_subscript_out_of_range = 0;
static int bv_i_err_syntax = 0;
static int bv_i_err_too_many_files = 0;
static int bv_i_err_type_mismatch = 0;
static int bv_i_part_count = 0;
static int bv_i_tab_col = 0;
static char bv_s_invdescbuf[256] = {0};
static char bv_s_invflagbuf[256] = {0};
static char bv_s_invpricebuf[256] = {0};
static char bv_s_invqtybuf[256] = {0};
static char bv_s_invreorderbuf[256] = {0};
static char bv_s_kp[256] = {0};

void bf_s_ltrim_s(const char* bv_s_self_in, char* bcc_out);
void bf_s_rtrim_s(const char* bv_s_self_in, char* bcc_out);
void bf_s_error(int bv_i_code, char* bcc_out);
int bf_i_isempty(const char* bv_s_flag_in);
int bf_i_partinrange(int bv_i_n);
void bf_s_readpartnumberinput(char* bcc_out);
void bf_s_readkey(char* bcc_out);
void bf_i_waitanykey(void);
void bf_i_showmainmenu(void);
void bf_i_showbadpartnumber(void);
void bf_i_showrangeretrymessage(void);
void bf_i_shownullentrymessage(const char* bv_s_partstr_in);
void bf_i_showpartstatus(int bv_i_partnum, const char* bv_s_desc_in, int bv_i_qty, int bv_i_reorder, float bv_f_price);
void bf_i_printlistheader(void);
void bf_i_printinventoryline(int bv_i_partnum, const char* bv_s_desc_in, int bv_i_qty, int bv_i_reorder);
void bf_i_printreorderheader(void);
void bf_i_printreorderline(int bv_i_partnum, const char* bv_s_desc_in, int bv_i_qty, int bv_i_reorder);
void bf_i_gatherpartdetails(int bv_i_partnum, char* bv_s_desc_in, int* bv_i_qty_in, int* bv_i_reorder_in, float* bv_f_price_in);
void bf_i_showaddstockscreen(int bv_i_partnum, const char* bv_s_desc_in, int bv_i_qty, int bv_i_reorder);
void bf_i_shownegativeqtywarning(void);
void bf_i_showsubtractstockscreen(int bv_i_partnum, const char* bv_s_desc_in, int bv_i_qty, int bv_i_reorder);
void bf_i_showoversubtractwarning(int bv_i_onhand);
void bf_i_checkpart(void);
void bf_i_editrecord(void);
void bf_i_listall(void);
void bf_i_addstock(void);
void bf_i_subtractstock(void);
void bf_i_reorderreport(void);
void bf_i_initializeinventoryfileifnew(void);
void bf_i_reportinventoryerror(int bv_i_err, int bv_i_erl);

void bf_s_ltrim_s(const char* bv_s_self_in, char* bcc_out) {
    char bv_s_self[256];
    snprintf(bv_s_self, sizeof(bv_s_self), "%s", bv_s_self_in);
    int bv_i_i = 0;

    bv_i_i = 1;
    while ((-(((-(bv_i_i <= ((int)strlen(bv_s_self))))) != 0 && ((-(strcmp(bcc_mid(bv_s_self, bv_i_i, 1), " ") == 0))) != 0))) {
        bv_i_i = (bv_i_i + 1);
    }
    snprintf(bcc_out, 256, "%s", bcc_mid(bv_s_self, bv_i_i, 2147483647));
    return;
}

void bf_s_rtrim_s(const char* bv_s_self_in, char* bcc_out) {
    char bv_s_self[256];
    snprintf(bv_s_self, sizeof(bv_s_self), "%s", bv_s_self_in);
    int bv_i_i = 0;

    bv_i_i = ((int)strlen(bv_s_self));
    while ((-(((-(bv_i_i > 0))) != 0 && ((-(strcmp(bcc_mid(bv_s_self, bv_i_i, 1), " ") == 0))) != 0))) {
        bv_i_i = (bv_i_i - 1);
    }
    snprintf(bcc_out, 256, "%s", bcc_mid(bv_s_self, 1, bv_i_i));
    return;
}

void bf_s_error(int bv_i_code, char* bcc_out) {
    {
        int bt_sel_0 = bv_i_code;
        int bt_sel_match_1 = 0;
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_syntax)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Syntax error");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_return_without_gosub)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "RETURN without GOSUB");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_out_of_data)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Out of DATA");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_illegal_function_call)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Illegal function call");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_overflow)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Overflow");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_out_of_memory)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Out of memory");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_subscript_out_of_range)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Subscript out of range");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_duplicate_definition)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Duplicate Definition");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_division_by_zero)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Division by zero");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_type_mismatch)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Type mismatch");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_out_of_string_space)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Out of string space");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_no_resume)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "No RESUME");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_resume_without_error)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "RESUME without error");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_device_timeout)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Device timeout");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_device_fault)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Device fault");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_out_of_paper)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Out of paper");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_bad_file_number)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Bad file number");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_file_not_found)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "File not found");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_bad_file_mode)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Bad file mode");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_file_already_open)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "File already open");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_device_io)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Device I/O error");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_file_already_exists)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "File already exists");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_disk_full)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Disk full");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_input_past_end)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Input past end");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_bad_record_number)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Bad record number");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_bad_file_name)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Bad file name");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_too_many_files)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Too many files");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_device_unavailable)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Device unavailable");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_disk_write_protected)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Disk write protected");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_disk_not_ready)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Disk not ready");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_disk_media_error)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Disk media error");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_path_file_access)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Path/File access error");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_path_not_found)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Path not found");
                return;
            }
        }
        if (!bt_sel_match_1) {
            char bt_s_2[256];
            snprintf(bt_s_2, sizeof(bt_s_2), "%s%s", "Error ", bcc_stri(bv_i_code));
            snprintf(bcc_out, 256, "%s", bt_s_2);
            return;
        }
    }
}

int bf_i_isempty(const char* bv_s_flag_in) {
    char bv_s_flag[256];
    snprintf(bv_s_flag, sizeof(bv_s_flag), "%s", bv_s_flag_in);

    return (-(((int)(unsigned char)bv_s_flag[0]) == 255));
}

int bf_i_partinrange(int bv_i_n) {
    if ((-(((-(bv_i_n >= 1))) != 0 && ((-(bv_i_n <= bv_i_part_count))) != 0))) {
        return 1;
    }
    return 0;
}

void bf_s_readpartnumberinput(char* bcc_out) {
    char bv_s_s[256] = {0};

    printf("Input part number? ");
    fflush(stdout);
    bcc_read_line();
    snprintf(bv_s_s, sizeof(bv_s_s), "%s", bcc_input_buf);
    snprintf(bcc_out, 256, "%s", bv_s_s);
    return;
}

void bf_s_readkey(char* bcc_out) {
    char bv_s_k[256] = {0};

    while (1) {
        snprintf(bv_s_k, sizeof(bv_s_k), "%s", bcc_inkey());
        bcc_semantic_continue_3: ;
        if ((-(strcmp(bv_s_k, "") != 0))) break;
    }
    snprintf(bcc_out, 256, "%s", bv_s_k);
    return;
}

void bf_i_waitanykey(void) {
    char bv_s_k[256] = {0};

    printf("\x1b[%d;%dH", 25, 10);
    printf("Press the AnyKey to continue...");
    fflush(stdout);
    while (1) {
        snprintf(bv_s_k, sizeof(bv_s_k), "%s", bcc_inkey());
        bcc_semantic_continue_4: ;
        if ((-(strcmp(bv_s_k, "") != 0))) break;
    }
}

void bf_i_showmainmenu(void) {
    printf("\x1b[2J\x1b[H");
    bcc_color(14, 4);
    printf("\x1b[2J\x1b[H");
    printf("\x1b[%d;%dH", 6, 1);
    printf("\n");
    // `tab(n)` passes straight through to real TAB(n), same as
    // fhb's own `PRINT TAB(V) "..."` -- but only as a bare item in
    // a PRINT list, juxtaposed or `;`-separated like here. Real
    // BASCOM rejects `"literal" + tab(n) + ...` (TAB isn't a real
    // string function you can concatenate); see printListHeader()
    // and printReorderHeader() below, which need `;` between a
    // preceding string and a `tab(n)` for exactly this reason.
    printf("\x1b[%dGInventory Program\n", 30);
    printf("\n");
    printf("\x1b[%dG1......C)heck a part\n", bv_i_tab_col);
    printf("\x1b[%dG2......E)dit/overwrite/add a part\n", bv_i_tab_col);
    char bt_s_5[256];
    snprintf(bt_s_5, sizeof(bt_s_5), "%s%s", "3......L)ist all", bcc_stri(bv_i_part_count));
    char bt_s_6[256];
    snprintf(bt_s_6, sizeof(bt_s_6), "%s%s", bt_s_5, "parts");
    printf("\x1b[%dG%s\n", bv_i_tab_col, bt_s_6);
    printf("\x1b[%dG4......A)dd stock\n", bv_i_tab_col);
    printf("\x1b[%dG5......S)ubtract stock\n", bv_i_tab_col);
    printf("\x1b[%dG6......R)eorder Report\n", bv_i_tab_col);
    printf("\n");
    printf("\x1b[%dG7......eX)it to system\n", bv_i_tab_col);
}

void bf_i_showbadpartnumber(void) {
    printf("\x1b[2J\x1b[H");
    printf("\x1b[%d;%dH", 10, 10);
    char bt_s_7[256];
    snprintf(bt_s_7, sizeof(bt_s_7), "%s%s", "Part number is out of permissable range of 1 to", bcc_stri(bv_i_part_count));
    printf("%s\n", bt_s_7);
}

void bf_i_showrangeretrymessage(void) {
    printf("\x1b[%d;%dH", 10, 15);
    char bt_s_8[256];
    snprintf(bt_s_8, sizeof(bt_s_8), "%s%s", "The Part number is out of permissable range of 1 to", bcc_stri(bv_i_part_count));
    printf("%s\n", bt_s_8);
    printf("\x1b[%d;%dH", 25, 15);
    printf("Press the Anykey to reenter part number...");
    fflush(stdout);
}

void bf_i_shownullentrymessage(const char* bv_s_partstr_in) {
    char bv_s_partstr[256];
    snprintf(bv_s_partstr, sizeof(bv_s_partstr), "%s", bv_s_partstr_in);

    printf("\x1b[%d;%dH", 10, bv_i_tab_col);
    char bt_s_9[256];
    snprintf(bt_s_9, sizeof(bt_s_9), "%s%s", "Part number ", bv_s_partstr);
    char bt_s_10[256];
    snprintf(bt_s_10, sizeof(bt_s_10), "%s%s", bt_s_9, " is a null entry");
    printf("%s\n", bt_s_10);
}

void bf_i_showpartstatus(int bv_i_partnum, const char* bv_s_desc_in, int bv_i_qty, int bv_i_reorder, float bv_f_price) {
    char bv_s_desc[256];
    snprintf(bv_s_desc, sizeof(bv_s_desc), "%s", bv_s_desc_in);

    printf("\x1b[2J\x1b[H");
    printf("\x1b[%d;%dH", 5, 1);
    printf("\x1b[%dGInventory Status for Individual Part Number\n", bv_i_tab_col);
    printf("\x1b[%dG===========================================\n", bv_i_tab_col);
    printf("\n");
    printf("\n");
    char bt_s_11[256];
    snprintf(bt_s_11, sizeof(bt_s_11), "%s%s", "     Part number:  ", bcc_stri(bv_i_partnum));
    printf("\x1b[%dG%s\n", bv_i_tab_col, bt_s_11);
    printf("\n");
    char bt_s_12[256];
    snprintf(bt_s_12, sizeof(bt_s_12), "%s%s", "       Item name:  ", bv_s_desc);
    printf("\x1b[%dG%s\n", bv_i_tab_col, bt_s_12);
    char bt_s_13[256];
    snprintf(bt_s_13, sizeof(bt_s_13), "%s%s", "Quantity on hand:  ", bcc_stri(bv_i_qty));
    printf("\x1b[%dG%s\n", bv_i_tab_col, bt_s_13);
    char bt_s_14[256];
    snprintf(bt_s_14, sizeof(bt_s_14), "%s%s", "   Reorder level:  ", bcc_stri(bv_i_reorder));
    printf("\x1b[%dG%s\n", bv_i_tab_col, bt_s_14);
    char bt_s_15[256];
    snprintf(bt_s_15, sizeof(bt_s_15), "%s%s", "      Unit price:  ", bcc_strd(bv_f_price));
    printf("\x1b[%dG%s\n", bv_i_tab_col, bt_s_15);
}

void bf_i_printlistheader(void) {
    printf("\x1b[2J\x1b[H");
    char bt_s_16[256];
    snprintf(bt_s_16, sizeof(bt_s_16), "%s%s", bcc_stri(bv_i_part_count), "items");
    printf("\x1b[%dGI N V E N T O R Y   L I S T I N G\x1b[%dG%s\n", 25, 65, bt_s_16);
    printf("                                          Quantity       Reorder\n");
    printf(" Partno           Description             on hand         level\n");
}

void bf_i_printinventoryline(int bv_i_partnum, const char* bv_s_desc_in, int bv_i_qty, int bv_i_reorder) {
    char bv_s_desc[256];
    snprintf(bv_s_desc, sizeof(bv_s_desc), "%s", bv_s_desc_in);

    char bt_s_17[256];
    snprintf(bt_s_17, sizeof(bt_s_17), "%s%s", bcc_stri(bv_i_partnum), "  ");
    char bt_s_18[256];
    snprintf(bt_s_18, sizeof(bt_s_18), "%s%s", bt_s_17, bv_s_desc);
    char bt_s_19[256];
    snprintf(bt_s_19, sizeof(bt_s_19), "%s%s", bt_s_18, "   ");
    char bt_s_20[256];
    snprintf(bt_s_20, sizeof(bt_s_20), "%s%s", bt_s_19, bcc_stri(bv_i_qty));
    char bt_s_21[256];
    snprintf(bt_s_21, sizeof(bt_s_21), "%s%s", bt_s_20, "          ");
    char bt_s_22[256];
    snprintf(bt_s_22, sizeof(bt_s_22), "%s%s", bt_s_21, bcc_stri(bv_i_reorder));
    printf("%s\n", bt_s_22);
}

void bf_i_printreorderheader(void) {
    printf("\x1b[2J\x1b[H");
    printf("\x1b[%d;%dH", 1, bv_i_tab_col);
    printf("Reorder Report\x1b[%dG%s\n", 55, bcc_date());
    printf("\n");
    printf("                                             Quantity       Reorder\n");
    printf("    Partno           Description             on hand         level\n");
    printf("   =======  ==============================   ========       =======\n");
}

void bf_i_printreorderline(int bv_i_partnum, const char* bv_s_desc_in, int bv_i_qty, int bv_i_reorder) {
    char bv_s_desc[256];
    snprintf(bv_s_desc, sizeof(bv_s_desc), "%s", bv_s_desc_in);

    char bt_s_23[256];
    snprintf(bt_s_23, sizeof(bt_s_23), "%s%s", "  ", bcc_stri(bv_i_partnum));
    char bt_s_24[256];
    snprintf(bt_s_24, sizeof(bt_s_24), "%s%s", bt_s_23, "  ");
    char bt_s_25[256];
    snprintf(bt_s_25, sizeof(bt_s_25), "%s%s", bt_s_24, bv_s_desc);
    char bt_s_26[256];
    snprintf(bt_s_26, sizeof(bt_s_26), "%s%s", bt_s_25, "   ");
    char bt_s_27[256];
    snprintf(bt_s_27, sizeof(bt_s_27), "%s%s", bt_s_26, bcc_stri(bv_i_qty));
    char bt_s_28[256];
    snprintf(bt_s_28, sizeof(bt_s_28), "%s%s", bt_s_27, "          ");
    char bt_s_29[256];
    snprintf(bt_s_29, sizeof(bt_s_29), "%s%s", bt_s_28, bcc_stri(bv_i_reorder));
    printf("%s\n", bt_s_29);
}

void bf_i_gatherpartdetails(int bv_i_partnum, char* bv_s_desc_in, int* bv_i_qty_in, int* bv_i_reorder_in, float* bv_f_price_in) {
    char bv_s_desc[256];
    snprintf(bv_s_desc, sizeof(bv_s_desc), "%s", bv_s_desc_in);
    int bv_i_qty = *bv_i_qty_in;
    int bv_i_reorder = *bv_i_reorder_in;
    float bv_f_price = *bv_f_price_in;

    printf("\x1b[2J\x1b[H");
    printf("\x1b[%d;%dH", 4, bv_i_tab_col);
    printf("Adding or Overwriting a Record\n");
    printf("\x1b[%d;%dH", 8, bv_i_tab_col);
    char bt_s_30[256];
    snprintf(bt_s_30, sizeof(bt_s_30), "%s%s", "Record/Partno", bcc_stri(bv_i_partnum));
    printf("%s\n", bt_s_30);
    printf("\x1b[%d;%dH", 11, 39);
    printf("------------------------------\n");
    printf("\x1b[%d;%dH", 10, bv_i_tab_col);
    printf("      Description? ");
    fflush(stdout);
    bcc_read_line();
    snprintf(bv_s_desc, sizeof(bv_s_desc), "%s", bcc_input_buf);
    printf("\x1b[%d;%dH", 12, bv_i_tab_col);
    printf("Quantity in stock? ");
    fflush(stdout);
    bcc_read_line();
    bv_i_qty = atoi(bcc_input_buf);
    printf("\x1b[%d;%dH", 14, bv_i_tab_col);
    printf("    Reorder level? ");
    fflush(stdout);
    bcc_read_line();
    bv_i_reorder = atoi(bcc_input_buf);
    printf("\x1b[%d;%dH", 16, bv_i_tab_col);
    printf("       Unit price? ");
    fflush(stdout);
    bcc_read_line();
    bv_f_price = atof(bcc_input_buf);
    printf("\x1b[%d;%dH", 18, bv_i_tab_col);
    printf("Is information correct (Y/N)?\n");
    snprintf(bv_s_desc_in, 256, "%s", bv_s_desc);
    *bv_i_qty_in = bv_i_qty;
    *bv_i_reorder_in = bv_i_reorder;
    *bv_f_price_in = bv_f_price;
}

void bf_i_showaddstockscreen(int bv_i_partnum, const char* bv_s_desc_in, int bv_i_qty, int bv_i_reorder) {
    char bv_s_desc[256];
    snprintf(bv_s_desc, sizeof(bv_s_desc), "%s", bv_s_desc_in);

    printf("\x1b[2J\x1b[H");
    printf("\x1b[%d;%dH", 4, 25);
    printf("Add to an inventory part number\n");
    printf("\x1b[%d;%dH", 5, 25);
    printf("===============================\n");
    printf("\x1b[%d;%dH", 8, bv_i_tab_col);
    char bt_s_31[256];
    snprintf(bt_s_31, sizeof(bt_s_31), "%s%s", "     Part number: ", bcc_stri(bv_i_partnum));
    printf("%s\n", bt_s_31);
    printf("\x1b[%d;%dH", 9, bv_i_tab_col);
    char bt_s_32[256];
    snprintf(bt_s_32, sizeof(bt_s_32), "%s%s", "Item description: ", bv_s_desc);
    printf("%s\n", bt_s_32);
    printf("\x1b[%d;%dH", 10, bv_i_tab_col);
    char bt_s_33[256];
    snprintf(bt_s_33, sizeof(bt_s_33), "%s%s", "Quantity on hand: ", bcc_stri(bv_i_qty));
    printf("%s\n", bt_s_33);
    printf("\x1b[%d;%dH", 11, bv_i_tab_col);
    char bt_s_34[256];
    snprintf(bt_s_34, sizeof(bt_s_34), "%s%s", "   Reorder Level: ", bcc_stri(bv_i_reorder));
    printf("%s\n", bt_s_34);
}

void bf_i_shownegativeqtywarning(void) {
    printf("\x1b[%d;%dH", 17, 15);
    printf("The quantity to add must NOT be a negative number\n");
    printf("\x1b[%d;%dH", 25, 1);
    printf("Please press the Anykey to reenter quantity to add...");
    fflush(stdout);
}

void bf_i_showsubtractstockscreen(int bv_i_partnum, const char* bv_s_desc_in, int bv_i_qty, int bv_i_reorder) {
    char bv_s_desc[256];
    snprintf(bv_s_desc, sizeof(bv_s_desc), "%s", bv_s_desc_in);

    printf("\x1b[2J\x1b[H");
    printf("\x1b[%d;%dH", 4, bv_i_tab_col);
    printf("Subtract an inventory part number\n");
    printf("\x1b[%d;%dH", 5, bv_i_tab_col);
    printf("=================================\n");
    printf("\x1b[%d;%dH", 8, bv_i_tab_col);
    char bt_s_35[256];
    snprintf(bt_s_35, sizeof(bt_s_35), "%s%s", "         Part number: ", bcc_stri(bv_i_partnum));
    printf("%s\n", bt_s_35);
    printf("\x1b[%d;%dH", 9, bv_i_tab_col);
    char bt_s_36[256];
    snprintf(bt_s_36, sizeof(bt_s_36), "%s%s", "    Item description: ", bv_s_desc);
    printf("%s\n", bt_s_36);
    printf("\x1b[%d;%dH", 10, bv_i_tab_col);
    char bt_s_37[256];
    snprintf(bt_s_37, sizeof(bt_s_37), "%s%s", "    Quantity on hand: ", bcc_stri(bv_i_qty));
    printf("%s\n", bt_s_37);
    printf("\x1b[%d;%dH", 11, bv_i_tab_col);
    char bt_s_38[256];
    snprintf(bt_s_38, sizeof(bt_s_38), "%s%s", "       Reorder Level: ", bcc_stri(bv_i_reorder));
    printf("%s\n", bt_s_38);
}

void bf_i_showoversubtractwarning(int bv_i_onhand) {
    printf("\x1b[%d;%dH", 17, 5);
    printf("The quantity to SUBTRACT must NOT result in NEGATIVE inventory\n");
    printf("\x1b[%d;%dH", 18, 5);
    char bt_s_39[256];
    snprintf(bt_s_39, sizeof(bt_s_39), "%s%s", "Only", bcc_stri(bv_i_onhand));
    char bt_s_40[256];
    snprintf(bt_s_40, sizeof(bt_s_40), "%s%s", bt_s_39, " IN STOCK");
    printf("%s\n", bt_s_40);
    printf("\x1b[%d;%dH", 25, 1);
    printf("Please press the Anykey to reenter quantity to subtract...");
    fflush(stdout);
}

void bf_i_checkpart(void) {
    float bv_f_pprice = 0;
    int bv_i_part = 0;
    int bv_i_pqty = 0;
    int bv_i_preorder = 0;
    char bv_s_invdescbuf[256] = {0};
    char bv_s_invflagbuf[256] = {0};
    char bv_s_invpricebuf[256] = {0};
    char bv_s_invqtybuf[256] = {0};
    char bv_s_invreorderbuf[256] = {0};
    char bv_s_partstr[256] = {0};
    char bv_s_pdesc[256] = {0};
    char bv_s_pflag[256] = {0};

    // global inv
    char bt_s_41[256];
    bf_s_readpartnumberinput(bt_s_41);
    snprintf(bv_s_partstr, sizeof(bv_s_partstr), "%s", bt_s_41);
    bv_i_part = ((int)round((double)(atof(bv_s_partstr))));
    if ((-(bf_i_partinrange(bv_i_part) == 0))) {
        bf_i_showbadpartnumber();
        bf_i_waitanykey();
        return;
    }
    // BASCAL-ism: `let p = inv[part%]` reads record `part%` of the
    // `inv` file into a local record variable `p` -- one expression
    // for what fhb's `GET #1, PART!` plus five separate field reads
    // (F$, D$, CVI(Q$), CVI(R$), CVS(P$)) did by hand. The write
    // side, `inv[part%] = { ... }` (see editRecord() below), is the
    // same sugar for PUT plus the LSET/MKx$ packing it replaces.
    // let p = inv[...]  (whole-record read)
    bcc_get_record_part(bcc_files[0], bv_i_part, bv_s_invflagbuf, bv_s_invdescbuf, bv_s_invqtybuf, bv_s_invreorderbuf, bv_s_invpricebuf);
    char bt_s_42[256];
    bf_s_rtrim_s(bv_s_invflagbuf, bt_s_42);
    snprintf(bv_s_pflag, sizeof(bv_s_pflag), "%s", bt_s_42);
    char bt_s_43[256];
    bf_s_rtrim_s(bv_s_invdescbuf, bt_s_43);
    snprintf(bv_s_pdesc, sizeof(bv_s_pdesc), "%s", bt_s_43);
    bv_i_pqty = bcc_cvi(bv_s_invqtybuf);
    bv_i_preorder = bcc_cvi(bv_s_invreorderbuf);
    bv_f_pprice = bcc_cvs(bv_s_invpricebuf);
    if (bf_i_isempty(bv_s_pflag)) {
        printf("\x1b[2J\x1b[H");
        printf("\x1b[%d;%dH", 10, 18);
        char bt_s_44[256];
        snprintf(bt_s_44, sizeof(bt_s_44), "%s%s", "Part number", bcc_stri(bv_i_part));
        char bt_s_45[256];
        snprintf(bt_s_45, sizeof(bt_s_45), "%s%s", bt_s_44, "is still a null entry at this time");
        printf("%s\n", bt_s_45);
        bf_i_waitanykey();
        return;
    }
    int bt_arg_46 = bv_i_part;
    char bt_arg_47[256];
    snprintf(bt_arg_47, sizeof(bt_arg_47), "%s", bv_s_pdesc);
    int bt_arg_48 = bv_i_pqty;
    int bt_arg_49 = bv_i_preorder;
    float bt_arg_50 = bv_f_pprice;
    bf_i_showpartstatus(bt_arg_46, bt_arg_47, bt_arg_48, bt_arg_49, bt_arg_50);
    bf_i_waitanykey();
}

void bf_i_editrecord(void) {
    float bv_f_editprice = 0;
    float bv_f_pprice = 0;
    int bv_i_editqty = 0;
    int bv_i_editreorder = 0;
    int bv_i_part = 0;
    int bv_i_pqty = 0;
    int bv_i_preorder = 0;
    char bv_s_editdesc[256] = {0};
    char bv_s_invdescbuf[256] = {0};
    char bv_s_invflagbuf[256] = {0};
    char bv_s_invpricebuf[256] = {0};
    char bv_s_invqtybuf[256] = {0};
    char bv_s_invreorderbuf[256] = {0};
    char bv_s_kp[256] = {0};
    char bv_s_partstr[256] = {0};
    char bv_s_pdesc[256] = {0};
    char bv_s_pflag[256] = {0};

    // global inv
    printf("\x1b[2J\x1b[H");
    printf("\x1b[%d;%dH", 10, bv_i_tab_col);
    char bt_s_51[256];
    bf_s_readpartnumberinput(bt_s_51);
    snprintf(bv_s_partstr, sizeof(bv_s_partstr), "%s", bt_s_51);
    bv_i_part = ((int)round((double)(atof(bv_s_partstr))));
    if ((-(bf_i_partinrange(bv_i_part) == 0))) {
        bf_i_showbadpartnumber();
        bf_i_waitanykey();
        return;
    }
    // let p = inv[...]  (whole-record read)
    bcc_get_record_part(bcc_files[0], bv_i_part, bv_s_invflagbuf, bv_s_invdescbuf, bv_s_invqtybuf, bv_s_invreorderbuf, bv_s_invpricebuf);
    char bt_s_52[256];
    bf_s_rtrim_s(bv_s_invflagbuf, bt_s_52);
    snprintf(bv_s_pflag, sizeof(bv_s_pflag), "%s", bt_s_52);
    char bt_s_53[256];
    bf_s_rtrim_s(bv_s_invdescbuf, bt_s_53);
    snprintf(bv_s_pdesc, sizeof(bv_s_pdesc), "%s", bt_s_53);
    bv_i_pqty = bcc_cvi(bv_s_invqtybuf);
    bv_i_preorder = bcc_cvi(bv_s_invreorderbuf);
    bv_f_pprice = bcc_cvs(bv_s_invpricebuf);
    if ((-(bf_i_isempty(bv_s_pflag) == 0))) {
        printf("\x1b[%d;%dH", 12, bv_i_tab_col);
        printf("Overwrite existing part data?\n");
        char bt_s_54[256];
        bf_s_readkey(bt_s_54);
        snprintf(bv_s_kp, sizeof(bv_s_kp), "%s", bt_s_54);
        if ((-(((-(strcmp(bv_s_kp, "Y") != 0))) != 0 && ((-(strcmp(bv_s_kp, "y") != 0))) != 0))) {
            return;
        }
    }

    while (1) {
        int bt_arg_56 = bv_i_part;
        bf_i_gatherpartdetails(bt_arg_56, bv_s_editdesc, &bv_i_editqty, &bv_i_editreorder, &bv_f_editprice);
        char bt_s_57[256];
        bf_s_readkey(bt_s_57);
        snprintf(bv_s_kp, sizeof(bv_s_kp), "%s", bt_s_57);
        bcc_semantic_continue_55: ;
        if ((-(((-(strcmp(bv_s_kp, "Y") == 0))) != 0 || ((-(strcmp(bv_s_kp, "y") == 0))) != 0))) break;
    }
    // inv[...] = { ... }  (whole-record write)
    int16_t bcc_tmp_58 = bv_i_editqty;
    int16_t bcc_tmp_59 = bv_i_editreorder;
    float bcc_tmp_60 = bv_f_editprice;
    bcc_put_record_part(bcc_files[0], bv_i_part, "1", bv_s_editdesc, &bcc_tmp_58, &bcc_tmp_59, &bcc_tmp_60);
}

void bf_i_listall(void) {
    float bv_f_pprice = 0;
    int bv_i_i = 0;
    int bv_i_pqty = 0;
    int bv_i_preorder = 0;
    int bv_i_scrollcount = 0;
    char bv_s_invdescbuf[256] = {0};
    char bv_s_invflagbuf[256] = {0};
    char bv_s_invpricebuf[256] = {0};
    char bv_s_invqtybuf[256] = {0};
    char bv_s_invreorderbuf[256] = {0};
    char bv_s_pdesc[256] = {0};
    char bv_s_pflag[256] = {0};

    // global inv
    bf_i_printlistheader();
    bv_i_scrollcount = 0;
    int bt_lim_61 = bv_i_part_count;
    int bt_step_61 = 1;
    for (bv_i_i = 1; bt_step_61 >= 0 ? bv_i_i <= bt_lim_61 : bv_i_i >= bt_lim_61; bv_i_i += bt_step_61) {
        // let p = inv[...]  (whole-record read)
        bcc_get_record_part(bcc_files[0], bv_i_i, bv_s_invflagbuf, bv_s_invdescbuf, bv_s_invqtybuf, bv_s_invreorderbuf, bv_s_invpricebuf);
        char bt_s_62[256];
        bf_s_rtrim_s(bv_s_invflagbuf, bt_s_62);
        snprintf(bv_s_pflag, sizeof(bv_s_pflag), "%s", bt_s_62);
        char bt_s_63[256];
        bf_s_rtrim_s(bv_s_invdescbuf, bt_s_63);
        snprintf(bv_s_pdesc, sizeof(bv_s_pdesc), "%s", bt_s_63);
        bv_i_pqty = bcc_cvi(bv_s_invqtybuf);
        bv_i_preorder = bcc_cvi(bv_s_invreorderbuf);
        bv_f_pprice = bcc_cvs(bv_s_invpricebuf);
        int bt_arg_64 = bv_i_i;
        char bt_arg_65[256];
        snprintf(bt_arg_65, sizeof(bt_arg_65), "%s", bv_s_pdesc);
        int bt_arg_66 = bv_i_pqty;
        int bt_arg_67 = bv_i_preorder;
        bf_i_printinventoryline(bt_arg_64, bt_arg_65, bt_arg_66, bt_arg_67);
        bv_i_scrollcount = (bv_i_scrollcount + 1);
        if ((-(bv_i_scrollcount == 20))) {
            bf_i_waitanykey();
            bv_i_scrollcount = 0;
            // Redraw for the next page rather than let it keep scrolling
            // past row 25 -- see printListHeader()'s own note on why a
            // fixed-row prompt can't coexist with unbounded scrolling.
            if ((-(bv_i_i < bv_i_part_count))) {
                bf_i_printlistheader();
            }
        }
    }
}

void bf_i_addstock(void) {
    float bv_f_pprice = 0;
    int bv_i_addamt = 0;
    int bv_i_part = 0;
    int bv_i_pqty = 0;
    int bv_i_preorder = 0;
    int bv_i_validpart = 0;
    char bv_s_addstr[256] = {0};
    char bv_s_invdescbuf[256] = {0};
    char bv_s_invflagbuf[256] = {0};
    char bv_s_invpricebuf[256] = {0};
    char bv_s_invqtybuf[256] = {0};
    char bv_s_invreorderbuf[256] = {0};
    char bv_s_partstr[256] = {0};
    char bv_s_pdesc[256] = {0};
    char bv_s_pflag[256] = {0};

    // global inv
    printf("\x1b[2J\x1b[H");
    printf("\x1b[%d;%dH", 5, 25);
    printf("A D D I N G   S T O C K\n");

    while (1) {
        printf("\x1b[%d;%dH", 8, 25);
        char bt_s_69[256];
        bf_s_readpartnumberinput(bt_s_69);
        snprintf(bv_s_partstr, sizeof(bv_s_partstr), "%s", bt_s_69);
        bv_i_part = ((int)round((double)(atof(bv_s_partstr))));
        bv_i_validpart = bf_i_partinrange(bv_i_part);
        if ((-(bv_i_validpart == 0))) {
            bf_i_showrangeretrymessage();
            char bt_s_70[256];
            bf_s_readkey(bt_s_70);
            (void)(bt_s_70);
        }
        bcc_semantic_continue_68: ;
        if ((-(bv_i_validpart != 0))) break;
    }

    // let p = inv[...]  (whole-record read)
    bcc_get_record_part(bcc_files[0], bv_i_part, bv_s_invflagbuf, bv_s_invdescbuf, bv_s_invqtybuf, bv_s_invreorderbuf, bv_s_invpricebuf);
    char bt_s_71[256];
    bf_s_rtrim_s(bv_s_invflagbuf, bt_s_71);
    snprintf(bv_s_pflag, sizeof(bv_s_pflag), "%s", bt_s_71);
    char bt_s_72[256];
    bf_s_rtrim_s(bv_s_invdescbuf, bt_s_72);
    snprintf(bv_s_pdesc, sizeof(bv_s_pdesc), "%s", bt_s_72);
    bv_i_pqty = bcc_cvi(bv_s_invqtybuf);
    bv_i_preorder = bcc_cvi(bv_s_invreorderbuf);
    bv_f_pprice = bcc_cvs(bv_s_invpricebuf);
    if (bf_i_isempty(bv_s_pflag)) {
        char bt_arg_73[256];
        snprintf(bt_arg_73, sizeof(bt_arg_73), "%s", bv_s_partstr);
        bf_i_shownullentrymessage(bt_arg_73);
        char bt_s_74[256];
        bf_s_readkey(bt_s_74);
        (void)(bt_s_74);
        return;
    }

    while (1) {
        int bt_arg_76 = bv_i_part;
        char bt_arg_77[256];
        snprintf(bt_arg_77, sizeof(bt_arg_77), "%s", bv_s_pdesc);
        int bt_arg_78 = bv_i_pqty;
        int bt_arg_79 = bv_i_preorder;
        bf_i_showaddstockscreen(bt_arg_76, bt_arg_77, bt_arg_78, bt_arg_79);
        printf("\x1b[%d;%dH", 14, bv_i_tab_col);
        printf(" Quantity to add? ");
        fflush(stdout);
        bcc_read_line();
        snprintf(bv_s_addstr, sizeof(bv_s_addstr), "%s", bcc_input_buf);
        bv_i_addamt = ((int)round((double)(atof(bv_s_addstr))));
        if ((-(bv_i_addamt < 0))) {
            bf_i_shownegativeqtywarning();
            char bt_s_80[256];
            bf_s_readkey(bt_s_80);
            (void)(bt_s_80);
        }
        bcc_semantic_continue_75: ;
        if ((-(bv_i_addamt >= 0))) break;
    }

    bv_i_pqty = (bv_i_pqty + bv_i_addamt);
    // inv[...] = p  (write back a let-bound record)
    int16_t bcc_tmp_81 = bv_i_pqty;
    int16_t bcc_tmp_82 = bv_i_preorder;
    float bcc_tmp_83 = bv_f_pprice;
    bcc_put_record_part(bcc_files[0], bv_i_part, bv_s_pflag, bv_s_pdesc, &bcc_tmp_81, &bcc_tmp_82, &bcc_tmp_83);
}

void bf_i_subtractstock(void) {
    float bv_f_pprice = 0;
    int bv_i_oversubtract = 0;
    int bv_i_part = 0;
    int bv_i_pqty = 0;
    int bv_i_preorder = 0;
    int bv_i_subamt = 0;
    int bv_i_validpart = 0;
    char bv_s_invdescbuf[256] = {0};
    char bv_s_invflagbuf[256] = {0};
    char bv_s_invpricebuf[256] = {0};
    char bv_s_invqtybuf[256] = {0};
    char bv_s_invreorderbuf[256] = {0};
    char bv_s_partstr[256] = {0};
    char bv_s_pdesc[256] = {0};
    char bv_s_pflag[256] = {0};
    char bv_s_substr[256] = {0};

    // global inv
    printf("\x1b[2J\x1b[H");
    printf("\x1b[%d;%dH", 5, 20);
    printf("S U B T R A C T I N G    S T O C K\n");

    while (1) {
        printf("\x1b[%d;%dH", 8, 25);
        char bt_s_85[256];
        bf_s_readpartnumberinput(bt_s_85);
        snprintf(bv_s_partstr, sizeof(bv_s_partstr), "%s", bt_s_85);
        bv_i_part = ((int)round((double)(atof(bv_s_partstr))));
        bv_i_validpart = bf_i_partinrange(bv_i_part);
        if ((-(bv_i_validpart == 0))) {
            bf_i_showrangeretrymessage();
            char bt_s_86[256];
            bf_s_readkey(bt_s_86);
            (void)(bt_s_86);
        }
        bcc_semantic_continue_84: ;
        if ((-(bv_i_validpart != 0))) break;
    }

    // let p = inv[...]  (whole-record read)
    bcc_get_record_part(bcc_files[0], bv_i_part, bv_s_invflagbuf, bv_s_invdescbuf, bv_s_invqtybuf, bv_s_invreorderbuf, bv_s_invpricebuf);
    char bt_s_87[256];
    bf_s_rtrim_s(bv_s_invflagbuf, bt_s_87);
    snprintf(bv_s_pflag, sizeof(bv_s_pflag), "%s", bt_s_87);
    char bt_s_88[256];
    bf_s_rtrim_s(bv_s_invdescbuf, bt_s_88);
    snprintf(bv_s_pdesc, sizeof(bv_s_pdesc), "%s", bt_s_88);
    bv_i_pqty = bcc_cvi(bv_s_invqtybuf);
    bv_i_preorder = bcc_cvi(bv_s_invreorderbuf);
    bv_f_pprice = bcc_cvs(bv_s_invpricebuf);
    if (bf_i_isempty(bv_s_pflag)) {
        char bt_arg_89[256];
        snprintf(bt_arg_89, sizeof(bt_arg_89), "%s", bv_s_partstr);
        bf_i_shownullentrymessage(bt_arg_89);
        char bt_s_90[256];
        bf_s_readkey(bt_s_90);
        (void)(bt_s_90);
        return;
    }

    while (1) {
        int bt_arg_92 = bv_i_part;
        char bt_arg_93[256];
        snprintf(bt_arg_93, sizeof(bt_arg_93), "%s", bv_s_pdesc);
        int bt_arg_94 = bv_i_pqty;
        int bt_arg_95 = bv_i_preorder;
        bf_i_showsubtractstockscreen(bt_arg_92, bt_arg_93, bt_arg_94, bt_arg_95);
        printf("\x1b[%d;%dH", 14, bv_i_tab_col);
        printf("Quantity to subtract? ");
        fflush(stdout);
        bcc_read_line();
        snprintf(bv_s_substr, sizeof(bv_s_substr), "%s", bcc_input_buf);
        bv_i_subamt = ((int)round((double)(atof(bv_s_substr))));
        bv_i_oversubtract = 0;
        if ((-(((-(bv_i_subamt >= 0))) != 0 && ((-((bv_i_pqty - bv_i_subamt) < 0))) != 0))) {
            bv_i_oversubtract = 1;
            int bt_arg_96 = bv_i_pqty;
            bf_i_showoversubtractwarning(bt_arg_96);
            char bt_s_97[256];
            bf_s_readkey(bt_s_97);
            (void)(bt_s_97);
        }
        bcc_semantic_continue_91: ;
        if ((-(((-(bv_i_subamt >= 0))) != 0 && ((-(bv_i_oversubtract == 0))) != 0))) break;
    }

    bv_i_pqty = (bv_i_pqty - bv_i_subamt);
    if ((-(bv_i_pqty <= bv_i_preorder))) {
        printf("\x1b[%d;%dH", 16, bv_i_tab_col);
    }
    char bt_s_98[256];
    snprintf(bt_s_98, sizeof(bt_s_98), "%s%s", "quantity now", bcc_stri(bv_i_pqty));
    char bt_s_99[256];
    snprintf(bt_s_99, sizeof(bt_s_99), "%s%s", bt_s_98, " reorder level");
    char bt_s_100[256];
    snprintf(bt_s_100, sizeof(bt_s_100), "%s%s", bt_s_99, bcc_stri(bv_i_preorder));
    printf("%s\n", bt_s_100);
    // inv[...] = p  (write back a let-bound record)
    int16_t bcc_tmp_101 = bv_i_pqty;
    int16_t bcc_tmp_102 = bv_i_preorder;
    float bcc_tmp_103 = bv_f_pprice;
    bcc_put_record_part(bcc_files[0], bv_i_part, bv_s_pflag, bv_s_pdesc, &bcc_tmp_101, &bcc_tmp_102, &bcc_tmp_103);
}

void bf_i_reorderreport(void) {
    float bv_f_pprice = 0;
    int bv_i_i = 0;
    int bv_i_pqty = 0;
    int bv_i_preorder = 0;
    int bv_i_reportlinecount = 0;
    char bv_s_invdescbuf[256] = {0};
    char bv_s_invflagbuf[256] = {0};
    char bv_s_invpricebuf[256] = {0};
    char bv_s_invqtybuf[256] = {0};
    char bv_s_invreorderbuf[256] = {0};
    char bv_s_pdesc[256] = {0};
    char bv_s_pflag[256] = {0};

    // global inv
    bf_i_printreorderheader();
    bv_i_reportlinecount = 0;
    int bt_lim_104 = bv_i_part_count;
    int bt_step_104 = 1;
    for (bv_i_i = 1; bt_step_104 >= 0 ? bv_i_i <= bt_lim_104 : bv_i_i >= bt_lim_104; bv_i_i += bt_step_104) {
        // let p = inv[...]  (whole-record read)
        bcc_get_record_part(bcc_files[0], bv_i_i, bv_s_invflagbuf, bv_s_invdescbuf, bv_s_invqtybuf, bv_s_invreorderbuf, bv_s_invpricebuf);
        char bt_s_105[256];
        bf_s_rtrim_s(bv_s_invflagbuf, bt_s_105);
        snprintf(bv_s_pflag, sizeof(bv_s_pflag), "%s", bt_s_105);
        char bt_s_106[256];
        bf_s_rtrim_s(bv_s_invdescbuf, bt_s_106);
        snprintf(bv_s_pdesc, sizeof(bv_s_pdesc), "%s", bt_s_106);
        bv_i_pqty = bcc_cvi(bv_s_invqtybuf);
        bv_i_preorder = bcc_cvi(bv_s_invreorderbuf);
        bv_f_pprice = bcc_cvs(bv_s_invpricebuf);
        if ((-(bv_i_pqty < bv_i_preorder))) {
            int bt_arg_107 = bv_i_i;
            char bt_arg_108[256];
            snprintf(bt_arg_108, sizeof(bt_arg_108), "%s", bv_s_pdesc);
            int bt_arg_109 = bv_i_pqty;
            int bt_arg_110 = bv_i_preorder;
            bf_i_printreorderline(bt_arg_107, bt_arg_108, bt_arg_109, bt_arg_110);
            bv_i_reportlinecount = (bv_i_reportlinecount + 1);
            if ((-(bv_i_reportlinecount > 15))) {
                bf_i_waitanykey();
                bv_i_reportlinecount = 0;
                // Redraw for the next page rather than let it keep
                // scrolling past row 25 -- see printListHeader()'s own
                // note (same underlying issue, same fix) on why a
                // fixed-row prompt can't coexist with unbounded scrolling.
                if ((-(bv_i_i < bv_i_part_count))) {
                    bf_i_printreorderheader();
                }
            }
        }
    }
    bf_i_waitanykey();
}

void bf_i_initializeinventoryfileifnew(void) {
    float bv_f_pprice = 0;
    int bv_i_i = 0;
    int bv_i_pqty = 0;
    int bv_i_preorder = 0;
    char bv_s_invdescbuf[256] = {0};
    char bv_s_invflagbuf[256] = {0};
    char bv_s_invpricebuf[256] = {0};
    char bv_s_invqtybuf[256] = {0};
    char bv_s_invreorderbuf[256] = {0};
    char bv_s_pdesc[256] = {0};
    char bv_s_pflag[256] = {0};

    // global inv
    // let p = inv[...]  (whole-record read)
    bcc_get_record_part(bcc_files[0], 1, bv_s_invflagbuf, bv_s_invdescbuf, bv_s_invqtybuf, bv_s_invreorderbuf, bv_s_invpricebuf);
    char bt_s_111[256];
    bf_s_rtrim_s(bv_s_invflagbuf, bt_s_111);
    snprintf(bv_s_pflag, sizeof(bv_s_pflag), "%s", bt_s_111);
    char bt_s_112[256];
    bf_s_rtrim_s(bv_s_invdescbuf, bt_s_112);
    snprintf(bv_s_pdesc, sizeof(bv_s_pdesc), "%s", bt_s_112);
    bv_i_pqty = bcc_cvi(bv_s_invqtybuf);
    bv_i_preorder = bcc_cvi(bv_s_invreorderbuf);
    bv_f_pprice = bcc_cvs(bv_s_invpricebuf);
    if ((-(((int)(unsigned char)bv_s_pflag[0]) == 0))) {
        int bt_lim_113 = bv_i_part_count;
        int bt_step_113 = 1;
        for (bv_i_i = 1; bt_step_113 >= 0 ? bv_i_i <= bt_lim_113 : bv_i_i >= bt_lim_113; bv_i_i += bt_step_113) {
            // inv[...] = { ... }  (whole-record write)
            int16_t bcc_tmp_114 = 0;
            int16_t bcc_tmp_115 = 0;
            float bcc_tmp_116 = 0;
            bcc_put_record_part(bcc_files[0], bv_i_i, bcc_chr(255), "", &bcc_tmp_114, &bcc_tmp_115, &bcc_tmp_116);
        }
    }
}

void bf_i_reportinventoryerror(int bv_i_err, int bv_i_erl) {
    char bv_s_k[256] = {0};

    printf("\x1b[%d;%dH", 25, 1);
    char bt_s_117[256];
    snprintf(bt_s_117, sizeof(bt_s_117), "%s%s", "There has been an error on line", bcc_stri(bv_i_erl));
    char bt_s_118[256];
    snprintf(bt_s_118, sizeof(bt_s_118), "%s%s", bt_s_117, ": ");
    char bt_s_119[256];
    bf_s_error(bv_i_err, bt_s_119);
    char bt_s_120[256];
    snprintf(bt_s_120, sizeof(bt_s_120), "%s%s", bt_s_118, bt_s_119);
    printf("%s\n", bt_s_120);
    char bt_s_121[256];
    bf_s_readkey(bt_s_121);
    snprintf(bv_s_k, sizeof(bv_s_k), "%s", bt_s_121);
}

int main(void) {
    setvbuf(stdin, NULL, _IONBF, 0);
    // Strips leading spaces from self$. Not a real MBASIC/BASCOM 2.00 builtin --
    // verified against a real IBM BASIC Compiler 2.00 under dosbox-x -- so
    // BASCAL ships its own. Declared as a scalar method (see GitHub issue #41)
    // so a required stdlib call reads the same way as a built-in method call
    // (docs/language/functions-and-procedures.html#built-in-methods). The
    // ordinary call form (ltrim$(s$)) still works -- a method's receiver is an
    // implicit first parameter, so ordinary-call syntax resolves straight to
    // this same declaration, with no separate function needed (and no longer
    // allowed: a function and a method sharing one name is a duplicate
    // declaration, since they'd both claim the same callable identity).

    // Strips trailing spaces from self$. Not a real MBASIC/BASCOM 2.00 builtin --
    // verified against a real IBM BASIC Compiler 2.00 under dosbox-x -- so
    // BASCAL ships its own. Declared as a scalar method (see GitHub issue #41
    // and ltrim.bcl's own doc comment for the reasoning) -- rtrim$(s$) still
    // works via ordinary-call syntax resolving to this same declaration.

    // Maps an ERR code to its classic MBASIC/GW-BASIC/BASCOM message. Compiles
    // and links on a real IBM BASIC Compiler 2.00 as ERROR$, but silently
    // returns an empty string at runtime (verified under dosbox-x) -- so BASCAL
    // ships a working implementation.
    //
    // The named constants below are the complete common subset supported by
    // ERROR$: use them in THROW and filtered CATCH clauses instead of magic
    // numbers.  Dialect-specific errors outside this shared MBASIC/GW-BASIC/
    // BASCOM subset still fall through to ERROR$'s generic message.
    //
    // Deliberately NOT a scalar method (see GitHub issue #41, which asked for
    // this decision to be recorded either way): code% is an opaque lookup key,
    // not a value the call is naturally "operating on" the way ltrim$/rtrim$/
    // ucase$/lcase$ operate on their string -- code%.error() would read as if
    // the *error code itself* has a message, when really this is a lookup
    // table keyed by that code. Stays an ordinary function.

    bv_i_err_syntax = 2;
    bv_i_err_return_without_gosub = 3;
    bv_i_err_out_of_data = 4;
    bv_i_err_illegal_function_call = 5;
    bv_i_err_overflow = 6;
    bv_i_err_out_of_memory = 7;
    bv_i_err_subscript_out_of_range = 9;
    bv_i_err_duplicate_definition = 10;
    bv_i_err_division_by_zero = 11;
    bv_i_err_type_mismatch = 13;
    bv_i_err_out_of_string_space = 14;
    bv_i_err_no_resume = 19;
    bv_i_err_resume_without_error = 20;
    bv_i_err_device_timeout = 24;
    bv_i_err_device_fault = 25;
    bv_i_err_out_of_paper = 27;
    bv_i_err_bad_file_number = 52;
    bv_i_err_file_not_found = 53;
    bv_i_err_bad_file_mode = 54;
    bv_i_err_file_already_open = 55;
    bv_i_err_device_io = 57;
    bv_i_err_file_already_exists = 58;
    bv_i_err_disk_full = 61;
    bv_i_err_input_past_end = 62;
    bv_i_err_bad_record_number = 63;
    bv_i_err_bad_file_name = 64;
    bv_i_err_too_many_files = 67;
    bv_i_err_device_unavailable = 68;
    bv_i_err_disk_write_protected = 70;
    bv_i_err_disk_not_ready = 71;
    bv_i_err_disk_media_error = 72;
    bv_i_err_path_file_access = 75;
    bv_i_err_path_not_found = 76;

    // ============================================================
    // INVENTORY.BCL -- Random-Access Inventory Program
    //
    // A BASCAL reconstruction of "Example program for RANDOM ACCESS
    // FILE study", by fhb, 8/19/98, from Joseph Sixpack's GW-BASIC
    // programs page (part of his "Last Book of GW-Basic" collection):
    // http://www.geocities.ws/joseph_sixpack/binventory.html
    // fhb's own header comment credits the original as "suggested
    // from MS-BASIC manual".
    //
    // This is a reconstruction, not a line-by-line port -- some
    // original pieces have no BASCAL equivalent and were dropped
    // rather than approximated:
    // - The GOTO-driven "subroutine roadmap" dispatcher at the top
    // of fhb's listing (a `LIST 110-320` etc. navigation aid for
    // editing in the GW-BASIC interpreter) has no meaning once the
    // program is structured into named function/procedure blocks.
    // - `KEY OFF` / `KEY I,""` (clearing the function-key soft-label
    // row) and `VIEW PRINT` (scroll-region windowing for the list
    // screen) are interpreter/console features BASCAL doesn't
    // expose.
    // - fhb's own hand-rolled numeric-ERR-code-to-message lookup table
    // (ERR=1 "Input value overflow", ERR=2 "Syntax error", ... ERR=25)
    // is replaced below by BASCAL's com.bascal.stdlib.error library
    // (ERROR$(code%)) -- same idea, BASCAL's own table; it still
    // doesn't decode ERL, which errorTrap() reports as the raw line
    // number.
    // - fhb's one-time "hidden" datafile initializer (PUT-ing 100
    // blank, CHR$(255)-flagged records) is reproduced below as
    // initializeInventoryFileIfNew(), called once at program entry --
    // inven.dat no longer has to be pre-populated by hand.
    // - The three original tab-position constants (T=20, U=25,
    // V=30) are collapsed into a single `TAB_COL = 20`; a couple of
    // screens that used U=25 in the original (see showAddStockScreen
    // below) keep 25 as a literal rather than reusing TAB_COL.
    //
    // Tracks parts in a fixed 100-record file: check status, add,
    // edit, add/subtract stock, and a reorder report.
    //
    // Error handling uses try/catch (GitHub issue #60), not the raw `on
    // error goto` / `resume next` fhb's original relies on: a failed menu
    // action is abandoned outright and the program returns straight to the
    // main menu, rather than resuming at the exact instruction after
    // whatever failed -- see reportInventoryError() below and
    // tutorial/inventory_try_catch.draft's own header comment for why. This
    // is a real, deliberate behavior change from an earlier on-error-goto
    // version of this file, which *was* verified against real BASCOM 2.00
    // under dosbox-x (only with the /E and /X switches -- error trapping
    // isn't linked in by default); the try/catch shape below transpiles to
    // the same ON ERROR GOTO/RESUME primitives BASCOM accepts, but hasn't
    // itself been independently re-verified against a real BASCOM compile.
    // ============================================================

    // BASCAL-ism: the record/file DSL. `record ... end record` plus
    // `file ... as ... = open(...)` below replace fhb's manual
    // FIELD #1,1 AS F$,30 AS D$,2 AS Q$,... buffer layout entirely --
    // bcc computes the field widths and record LEN from this
    // declaration and generates the FIELD statement itself. Named
    // field access (`p.flag`, `p.qty`, ...) and whole-record
    // read/write via `inv[n]` (see checkPart() below) replace fhb's
    // manual GET/PUT plus LSET/RSET and MKI$/MKS$/CVI$/CVS$ packing.

    // BASCAL-ism: `const` is a real compile-time constant, not a plain
    // variable assignment like fhb's `N=100` / `T=20` -- it can never
    // be reassigned, and resolves to the same value everywhere,
    // including inside every function/procedure below, with no
    // `global` declaration needed.
    bv_i_part_count = 100;
    bv_i_tab_col = 20;

    // `file ... = open(...)` is sugar for OPEN ... FOR RANDOM AS #n
    // LEN = <record width> plus the FIELD statement fhb wrote out by
    // hand at his line 550. Wrapped in its own try/catch: a file that
    // exists but can't be opened for random access (permissions, a
    // read-only inven.dat, disk full on the fallback create) is a real,
    // trappable error (code 75, "Path/File access error") on both
    // targets now, not a hard crash -- report it and exit cleanly
    // instead of leaving the program to fail confusingly the first time
    // something tries to use an `inv` that was never actually opened.
    int bcc_try_0_pending = 0;
    bcc_on_error_target = 0;
    // file inv as Part = open(...)  [39 bytes/record]
    bcc_files[0] = fopen("inven.dat", "rb+");
    if (!bcc_files[0]) bcc_files[0] = fopen("inven.dat", "wb+");
    if (!bcc_files[0]) {
        bcc_err = 75;
        bcc_erl = 91;
        bcc_err_file = "tutorial/inventory.bcl";
        goto bcc_try_0_catch;
    }
    bcc_on_error_target = -1;
    goto bcc_try_0_finally;
    bcc_try_0_catch: ;
    bcc_in_handler = 0;
    bcc_on_error_target = -1;
    bv_i_err = bcc_err;
    bv_i_erl = bcc_erl;
    char bt_s_122[256];
    bf_s_error(bv_i_err, bt_s_122);
    char bt_s_123[256];
    snprintf(bt_s_123, sizeof(bt_s_123), "%s%s", "could not open inven.dat: ", bt_s_122);
    printf("%s\n", bt_s_123);
    return 0;
    bcc_on_error_target = -1;
    goto bcc_try_0_finally;
    bcc_try_0_rethrow: ;
    bcc_try_0_pending = 1;
    bcc_try_0_finally: ;
    if (bcc_try_0_pending) {
        fprintf(stderr, "unhandled BASIC error %d\n", bcc_err);
        exit(1);
    }
    bcc_try_0_end: ;

    // -------------------- Pure functions (no file access) --------------------

    // BASCAL-ism: `function ... end function` with `return` replaces
    // fhb's convention of a GOSUB target plus a bare RETURN -- there's
    // no separate "subroutine label" and no shared/global result
    // variable to manage by hand; `isEmpty%(...)` is called like an
    // ordinary expression at every use below (e.g. `isEmpty%(p.flag)`).
    // A record whose flag byte is CHR$(255) is an empty/never-used slot.

    // BASCAL-ism: `&&` and `||` are short-circuit AND/OR -- real
    // MBASIC/BASCOM only has bitwise AND/OR (which fhb relies on here
    // too, since `PART!<1 OR PART!>N!` never short-circuits anyway).
    // BASCAL lowers `&&`/`||` into the equivalent branching so the
    // short-circuit *is* real at the generated-BASIC level; see the
    // manual's "Short-Circuit && and ||" section
    // (https://johnjoeallen.github.io/bascal/manual/).

    // -------------------- Keyboard input --------------------

    // BASCAL-ism: `do ... loop until` is a structured post-check loop
    // replacing fhb's `730 KP$=INKEY$:IF KP$="" THEN 730` GOTO-polling
    // idiom. `inkey$` itself is the real INKEY$ builtin passed straight
    // through, resolving correctly from inside a function/procedure
    // body like this one -- every menu action below calls
    // readKey$()/waitAnyKey() rather than polling INKEY$ inline.

    // -------------------- Display procedures --------------------

    // BASCAL-ism: no `VIEW PRINT` (see the header note above), so this
    // deliberately does NOT pin a "press any key" line to a fixed row the way
    // fhb's original does -- a bare `LOCATE 25, ...` sitting under content
    // that keeps printing past it (listAll()'s own items) collides with
    // whatever's later written there, since nothing here scrolls a bounded
    // region: waitAnyKey() is the only thing that ever touches row 25, and
    // only right when it actually blocks (see listAll()'s own redraw-per-page
    // structure below).

    // byref scalar parameters: gatherPartDetails writes the four editable
    // fields for a part directly back into the caller's variables.

    // -------------------- Menu actions --------------------

    // fhb's own one-time "hidden" datafile initializer PUT-ing 100 blank,
    // CHR$(255)-flagged records (see the header note above) -- reproduced
    // here so inven.dat no longer has to be pre-populated by hand before
    // running this program. A brand-new file OPEN created just now (rather
    // than one that already existed) reads back as all-zero bytes: record
    // 1's flag byte is CHR$(0), never CHR$(255) -- the one signal an
    // already-populated file (whose record 1 flag is always either
    // CHR$(255), still an empty slot, or a real part's own "1") could never
    // produce, so it's what isEmpty%() itself can't use (see its own
    // header note) but this one-time check safely can.

    // -------------------- Program entry --------------------

    printf("\x1b[2J\x1b[H");
    bf_i_initializeinventoryfileifnew();

    while (1) {
        bf_i_showmainmenu();
        char bt_s_124[256];
        bf_s_readkey(bt_s_124);
        snprintf(bv_s_kp, sizeof(bv_s_kp), "%s", bt_s_124);
        if ((-(bcc_instr("1234567cCeElLaAsSrRxX", bv_s_kp) != 0))) {
            // BASCAL-ism: `select case` replaces fhb's chain of eight
            // `IF VAL(KP$)=n OR KP$="x" OR KP$="X" THEN GOTO ...` lines
            // (his 770-840) with one multi-way dispatch.
            //
            // BASCAL-ism: `try`/`catch` (issue #60) replaces fhb's own global
            // `ON ERROR GOTO` trap. A failed menu action is abandoned outright
            // here -- the `catch` below runs, then execution continues right
            // after `end try`, back at `loop until` -- rather than resuming at
            // the exact instruction after whatever failed inside checkPart()/
            // editRecord()/etc. the way fhb's `RESUME NEXT` did. See
            // reportInventoryError() below and tutorial/inventory_try_catch.
            // draft's own header comment for why that arbitrary resume-point
            // behavior isn't something try/catch reproduces.
            int bcc_try_1_pending = 0;
            bcc_on_error_target = 1;
            {
                char bt_sel_125[256];
                snprintf(bt_sel_125, sizeof(bt_sel_125), "%s", bv_s_kp);
                int bt_sel_match_126 = 0;
                if (!bt_sel_match_126) {
                    if ((strcmp(bt_sel_125, "1") == 0) || (strcmp(bt_sel_125, "c") == 0) || (strcmp(bt_sel_125, "C") == 0)) {
                        bt_sel_match_126 = 1;
                        bf_i_checkpart();
                    }
                }
                if (!bt_sel_match_126) {
                    if ((strcmp(bt_sel_125, "2") == 0) || (strcmp(bt_sel_125, "e") == 0) || (strcmp(bt_sel_125, "E") == 0)) {
                        bt_sel_match_126 = 1;
                        bf_i_editrecord();
                    }
                }
                if (!bt_sel_match_126) {
                    if ((strcmp(bt_sel_125, "3") == 0) || (strcmp(bt_sel_125, "l") == 0) || (strcmp(bt_sel_125, "L") == 0)) {
                        bt_sel_match_126 = 1;
                        bf_i_listall();
                    }
                }
                if (!bt_sel_match_126) {
                    if ((strcmp(bt_sel_125, "4") == 0) || (strcmp(bt_sel_125, "a") == 0) || (strcmp(bt_sel_125, "A") == 0)) {
                        bt_sel_match_126 = 1;
                        bf_i_addstock();
                    }
                }
                if (!bt_sel_match_126) {
                    if ((strcmp(bt_sel_125, "5") == 0) || (strcmp(bt_sel_125, "s") == 0) || (strcmp(bt_sel_125, "S") == 0)) {
                        bt_sel_match_126 = 1;
                        bf_i_subtractstock();
                    }
                }
                if (!bt_sel_match_126) {
                    if ((strcmp(bt_sel_125, "6") == 0) || (strcmp(bt_sel_125, "r") == 0) || (strcmp(bt_sel_125, "R") == 0)) {
                        bt_sel_match_126 = 1;
                        bf_i_reorderreport();
                    }
                }
                if (!bt_sel_match_126) {
                    if ((strcmp(bt_sel_125, "7") == 0) || (strcmp(bt_sel_125, "x") == 0) || (strcmp(bt_sel_125, "X") == 0)) {
                        bt_sel_match_126 = 1;
                        // BASCAL-ism: `inv.close()` is sugar for `CLOSE #1`,
                        // matching fhb's own `90 CLOSE:SYSTEM`. fhb's original
                        // also had a separate "Quit to BASIC" option (his own
                        // 7, returning to the interpreter's command prompt
                        // rather than exiting to DOS) -- dropped here: a
                        // compiled program has no interpreter to return to,
                        // so it was never anything but a second syntax for
                        // this same close-and-exit action.
                        // inv.close()
                        fclose(bcc_files[0]);
                        bcc_files[0] = NULL;
                        bcc_color(7, 0);
                        printf("\x1b[2J\x1b[H");
                        exit(0);
                    }
                }
            }
            bcc_on_error_target = -1;
            goto bcc_try_1_finally;
            bcc_try_1_catch: ;
            bcc_in_handler = 0;
            bcc_on_error_target = -1;
            bv_i_err = bcc_err;
            bv_i_erl = bcc_erl;
            int bt_arg_127 = bv_i_err;
            int bt_arg_128 = bv_i_erl;
            bf_i_reportinventoryerror(bt_arg_127, bt_arg_128);
            bcc_on_error_target = -1;
            goto bcc_try_1_finally;
            bcc_try_1_rethrow: ;
            bcc_try_1_pending = 1;
            bcc_try_1_finally: ;
            if (bcc_try_1_pending) {
                fprintf(stderr, "unhandled BASIC error %d\n", bcc_err);
                exit(1);
            }
            bcc_try_1_end: ;
        }
    }

    // -------------------- Error handling --------------------
    // err%/erl% are ordinary locals scoped to the `catch` block above, not
    // aliases for the ambient (readable-anywhere) `err`/`erl` pseudo-
    // variables `on error goto` uses -- see `Statement::TryCatch`'s own doc
    // comment in ast.rs. Passed straight through to ERROR$ here like fhb's
    // own ERR/ERL (his 3390: "an error on line";ERL), decoded through
    // BASCAL's own com.bascal.stdlib.error (ERROR$) instead of fhb's
    // hand-rolled lookup table -- see the header note above. try/catch
    // itself isn't documented in the manual yet (GitHub issue #60 tracks
    // the still-unfinished C-target work; the manual page can follow once
    // that lands) -- see ast.rs's own `Statement::TryCatch` doc comment for
    // the full semantics meanwhile.
    return 0;
}

static char* bcc_strbuf_take(void) {
    char* buf = bcc_strbuf[bcc_strbuf_next];
    bcc_strbuf_next = (bcc_strbuf_next + 1) % BCC_STRBUF_COUNT;
    return buf;
}

static const char* bcc_mid(const char* s, int start, int length) {
    char* out = bcc_strbuf_take();
    int len = (int)strlen(s);
    int from = start - 1;
    if (from < 0) from = 0;
    if (from > len) from = len;
    int avail = len - from;
    if (length < 0) length = 0;
    if (length > avail) length = avail;
    snprintf(out, 256, "%.*s", length, s + from);
    return out;
}

static const char* bcc_chr(int code) {
    char* out = bcc_strbuf_take();
    snprintf(out, 256, "%c", code);
    return out;
}

static const char* bcc_stri(int value) {
    char* out = bcc_strbuf_take();
    snprintf(out, 256, "% d", value);
    return out;
}

static const char* bcc_strd(double value) {
    char* out = bcc_strbuf_take();
    snprintf(out, 256, "% g", value);
    return out;
}

static int bcc_instr(const char* s, const char* needle) {
    const char* found = strstr(s, needle);
    return found ? (int)(found - s) + 1 : 0;
}

static const char* bcc_inkey(void) {
    static char buf[2];
#if defined(_WIN32)
    HANDLE input = GetStdHandle(STD_INPUT_HANDLE);
    DWORD input_type = GetFileType(input);
    if (input_type == FILE_TYPE_PIPE) {
        DWORD available = 0;
        if (PeekNamedPipe(input, NULL, 0, NULL, &available, NULL) && available > 0) {
            DWORD read_count = 0;
            ReadFile(input, buf, 1, &read_count, NULL);
            buf[read_count == 1 ? 1 : 0] = 0;
        } else {
            buf[0] = 0;
        }
    } else if (input_type == FILE_TYPE_DISK) {
        int c = _read(_fileno(stdin), buf, 1);
        buf[c == 1 ? 1 : 0] = 0;
    } else if (_kbhit()) {
        buf[0] = (char)_getch();
        buf[1] = 0;
    } else {
        buf[0] = 0;
    }
#else
    struct termios orig, raw;
    tcgetattr(STDIN_FILENO, &orig);
    raw = orig;
    raw.c_lflag &= ~(ICANON | ECHO);
    raw.c_cc[VMIN] = 0;
    raw.c_cc[VTIME] = 0;
    tcsetattr(STDIN_FILENO, TCSANOW, &raw);

    unsigned char c;
    ssize_t n = read(STDIN_FILENO, &c, 1);
    if (n == 1) {
        buf[0] = (char)c;
        buf[1] = 0;
    } else {
        buf[0] = 0;
    }

    tcsetattr(STDIN_FILENO, TCSANOW, &orig);
#endif
    return buf;
}

static const char* bcc_date(void) {
    static char buf[11];
    time_t t = time(NULL);
    struct tm* tm_info = localtime(&t);
    snprintf(buf, sizeof(buf), "%02d-%02d-%04d", tm_info->tm_mon + 1, tm_info->tm_mday, tm_info->tm_year + 1900);
    return buf;
}

static void bcc_read_string_field(char* field, const unsigned char* source, size_t width) {
    memcpy(field, source, width);
    field[width] = 0;
    while (width > 0 && field[width - 1] == ' ') field[--width] = 0;
}

static void bcc_mki(char* out, int value) {
    int16_t v = (int16_t)value;
    memcpy(out, &v, 2);
}

static void bcc_mkl(char* out, int value) {
    int32_t v = (int32_t)value;
    memcpy(out, &v, 4);
}

static int bcc_cvi(const char* s) {
    int16_t v;
    memcpy(&v, s, 2);
    return (int)v;
}

static int bcc_cvl(const char* s) {
    int32_t v;
    memcpy(&v, s, 4);
    return (int)v;
}

static int bcc_read_record(FILE* file, void* buffer, size_t reclen, long record) {
    if (fseek(file, (record - 1) * (long)reclen, SEEK_SET) != 0) return 0;
    return fread(buffer, 1, reclen, file) == reclen;
}

static void bcc_write_record(FILE* file, const void* buffer, size_t reclen, long record) {
    fseek(file, (record - 1) * (long)reclen, SEEK_SET);
    fwrite(buffer, 1, reclen, file);
}

static void bcc_pad_string_field(unsigned char* dest, const char* value, size_t width) {
    size_t len = strlen(value);
    if (len > width) len = width;
    memcpy(dest, value, len);
    memset(dest + len, ' ', width - len);
}

static int bcc_put_record_part(FILE* file, long record, const char* field_0, const char* field_1, const int16_t* field_2, const int16_t* field_3, const float* field_4) {
    unsigned char buffer[39];
    if ((!field_0 || !field_1 || !field_2 || !field_3 || !field_4) && !bcc_read_record(file, buffer, 39, record)) return 0;
    if (field_0) bcc_pad_string_field(buffer + 0, field_0, 1);
    if (field_1) bcc_pad_string_field(buffer + 1, field_1, 30);
    (void)(field_2 && memcpy(buffer + 31, field_2, 2));
    (void)(field_3 && memcpy(buffer + 33, field_3, 2));
    (void)(field_4 && memcpy(buffer + 35, field_4, 4));
    bcc_write_record(file, buffer, 39, record);
    return 1;
}

static int bcc_get_record_part(FILE* file, long record, char* field_0, char* field_1, char* field_2, char* field_3, char* field_4) {
    unsigned char buffer[39];
    if (!bcc_read_record(file, buffer, 39, record)) return 0;
    bcc_read_string_field(field_0, buffer + 0, 1);
    bcc_read_string_field(field_1, buffer + 1, 30);
    memcpy(field_2, buffer + 31, 2);
    field_2[2] = 0;
    memcpy(field_3, buffer + 33, 2);
    field_3[2] = 0;
    memcpy(field_4, buffer + 35, 4);
    field_4[4] = 0;
    return 1;
}

static void bcc_mks(char* out, double value) {
    float v = (float)value;
    memcpy(out, &v, 4);
}

static void bcc_mkd(char* out, double value) {
    memcpy(out, &value, 8);
}

static float bcc_cvs(const char* s) {
    float v;
    memcpy(&v, s, 4);
    return v;
}

static double bcc_cvd(const char* s) {
    double v;
    memcpy(&v, s, 8);
    return v;
}

static int bcc_eof(FILE* file) {
    int c = fgetc(file);
    if (c == EOF) return -1;
    ungetc(c, file);
    return 0;
}

static void bcc_line_input_file(FILE* file, char* buf, size_t bufsize) {
    if (fgets(buf, (int)bufsize, file) == NULL) {
        buf[0] = 0;
        return;
    }
    buf[strcspn(buf, "\r\n")] = 0;
}

static void bcc_read_file_field(FILE* file, char* buf, size_t bufsize) {
    int c = fgetc(file);
    while (c == ' ') c = fgetc(file);
    size_t len = 0;
    if (c == '"') {
        c = fgetc(file);
        while (c != EOF && c != '"') {
            if (len + 1 < bufsize) buf[len++] = (char)c;
            c = fgetc(file);
        }
        c = fgetc(file);
        while (c != EOF && c != ',' && c != '\n') c = fgetc(file);
    } else {
        while (c != EOF && c != ',' && c != '\n' && c != '\r') {
            if (len + 1 < bufsize) buf[len++] = (char)c;
            c = fgetc(file);
        }
        if (c == '\r') {
            int c2 = fgetc(file);
            if (c2 != '\n' && c2 != EOF) ungetc(c2, file);
        }
    }
    buf[len] = 0;
}

static const int bcc_ansi_fg[16] = {30, 34, 32, 36, 31, 35, 33, 37, 90, 94, 92, 96, 91, 95, 93, 97};
static const int bcc_ansi_bg[8] = {40, 44, 42, 46, 41, 45, 43, 47};
static int bcc_color_used = 0;

static void bcc_color_reset(void) {
    printf("\x1b[0m");
}

static void bcc_color(int fg, int bg) {
    if (!bcc_color_used) {
        atexit(bcc_color_reset);
        bcc_color_used = 1;
    }
    printf("\x1b[%dm", bcc_ansi_fg[fg & 15]);
    if (bg >= 0) {
        printf("\x1b[%dm", bcc_ansi_bg[bg & 7]);
    }
}

static void bcc_read_line(void) {
    if (fgets(bcc_input_buf, sizeof(bcc_input_buf), stdin) == NULL) {
        bcc_input_buf[0] = 0;
        return;
    }
    bcc_input_buf[strcspn(bcc_input_buf, "\r\n")] = 0;
}


```



</details>

<!-- END generated tutorial source -->
