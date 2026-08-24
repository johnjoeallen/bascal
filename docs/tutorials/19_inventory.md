[Home](../) / [Tutorials](./) / Case Study: Random-Access Inventory

<div class="prose" markdown="1">

Every other tutorial in this series is a short walkthrough of one feature. This one is different: it's a reconstruction of a real, complete program — ["Example program for RANDOM ACCESS FILE study"](http://www.geocities.ws/joseph_sixpack/binventory.md), written by fhb in 1998 for Joseph Sixpack's "Last Book of GW-Basic" collection, and credited there as "suggested from MS-BASIC manual." It's a menu-driven parts inventory: check a part's status, edit or add a part, add or subtract stock, and print a reorder report, all backed by a fixed 100-record random-access file. Rebuilt in BASCAL, it exercises [record files](15_random_and_record_files.md), [procedures](14_procedures.md) (including `byref` parameters), [short-circuit `&&`/`||`](16_short_circuit.md), and [error handling](17_labels_and_error_handling.md) together, at the scale where classic BASIC's line-number bookkeeping starts to really hurt — and where BASCAL's structuring pays off.

This is a reconstruction, not a line-by-line port. A few of fhb's original pieces have no BASCAL equivalent and were deliberately dropped rather than approximated: the GOTO-driven "subroutine roadmap" dispatcher meant for navigating the listing in the GW-BASIC interpreter itself; `KEY OFF`/`VIEW PRINT` console features BASCAL doesn't expose; and a numeric-ERR-code-to-message lookup table, collapsed here into a single line reporting the raw `ERR`/`ERL` values. See the header comment in the source below for the full list, including why `inven.dat` must be pre-populated with 100 blank records before the program will run correctly — fhb's original had a one-time hidden initializer for this that has no BASCAL-source equivalent in `inventory.bcl` itself.

Verified against real BASCOM 2.00 under dosbox-x: it compiles clean and links, but only when BASCOM is invoked with the `/E` and `/X` switches, since `on error goto`/`resume` isn't linked in by default. [`scripts/run-in-dosbox.sh`](https://github.com/johnjoeallen/bascal/blob/main/scripts/run-in-dosbox.sh) in the repo automates all of this — compiling with `bcc`, seeding a blank `inven.dat` with 100 records itself (reproducing fhb's one-time initializer at the tooling level, outside the language), adding the right BASCOM switches, and launching an interactive dosbox-x session — so you can actually run this one instead of just reading it.

</div>

<div class="snippet" markdown="1">

### The record/file DSL replaces fhb's manual FIELD layout and MKx\$/CVx\$ packing

bcc computes the field widths and record `LEN` from the `record` declaration and emits the `FIELD` statement itself. Named field access (`p.flag`, `p.qty`, ...) and whole-record reads via `inv[n]` replace fhb's manual `GET`/`PUT` plus `LSET`/`RSET` and `MKI$`/`MKS$`/`CVI$`/`CVS$` packing.

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

</div>

<div class="snippet" markdown="1">

### `byref` parameters write straight back into the caller's variables

One call gathers all four editable fields for a part; no shared globals, no separate "output" convention to remember.

    procedure gatherPartDetails(partNum%, byref desc$, byref qty%, byref reorder%, byref price!)
        input "      Description"; desc$
        input "Quantity in stock"; qty%
        input "    Reorder level"; reorder%
        input "       Unit price"; price!
    end procedure

</div>

<div class="snippet" markdown="1">

### The error handler is a `procedure` — and bcc proves it's safe to be one

`on error goto` reaches a label or a procedure identically, via a plain `GOTO`, never a `GOSUB` — so a procedure used this way has no call frame for `RETURN` to pop. bcc's resolver checks for exactly that: any procedure named as an `on error goto` target must contain no `return` anywhere, and every path must be proven to end in `resume`/`resume next`/`resume <label>` rather than falling through. `errorTrap()`'s single trailing `resume next` satisfies that, so codegen skips the implicit `RETURN` it would otherwise append — there's nothing left to fall into even if the proof were somehow wrong. The same check also rejects calling it like an ordinary procedure elsewhere: something proven to never return can never come back to a normal caller either.

    on error goto errorTrap
    ' ...
    procedure errorTrap()
        locate 25, 1
        print "There has been an error on line" + str$(erl) + ": " + error$(err)
        k$ = readKey$()
        resume next
    end procedure

</div>



[← Standard Library Functions](18_stdlib.md)


<!-- BEGIN generated tutorial source -->

### `tutorial/inventory.bcl`

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
//    V=30) are collapsed into a single `tabCol% = 20`; a couple of
//    screens that used U=25 in the original (see showAddStockScreen
//    below) keep 25 as a literal rather than reusing tabCol%.
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
    flag:    string(1)
    desc:    string(30)
    qty:     int16
    reorder: int16
    price:   float32
end record

// BASCAL-ism: `const` is a real compile-time constant, not a plain
// variable assignment like fhb's `N=100` / `T=20` -- it can never
// be reassigned, and resolves to the same value everywhere,
// including inside every function/procedure below, with no
// `global` declaration needed.
const partCount% = 100
const tabCol% = 20

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
    if n% >= 1 && n% <= partCount% then
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
    print tab(tabCol%) "1......C)heck a part"
    print tab(tabCol%) "2......E)dit/overwrite/add a part"
    print tab(tabCol%) "3......L)ist all" + str$(partCount%) + "parts"
    print tab(tabCol%) "4......A)dd stock"
    print tab(tabCol%) "5......S)ubtract stock"
    print tab(tabCol%) "6......R)eorder Report"
    print
    print tab(tabCol%) "7......eX)it to system"
end procedure

procedure showBadPartNumber()
    cls
    locate 10, 10
    print "Part number is out of permissable range of 1 to" + str$(partCount%)
end procedure

procedure showRangeRetryMessage()
    locate 10, 15
    print "The Part number is out of permissable range of 1 to" + str$(partCount%)
    locate 25, 15
    print "Press the Anykey to reenter part number...";
end procedure

procedure showNullEntryMessage(partStr$)
    locate 10, tabCol%
    print "Part number " + partStr$ + " is a null entry"
end procedure

procedure showPartStatus(partNum%, desc$, qty%, reorder%, price!)
    cls
    locate 5, 1
    print tab(tabCol%) "Inventory Status for Individual Part Number"
    print tab(tabCol%) "==========================================="
    print
    print
    print tab(tabCol%) "     Part number:  " + str$(partNum%)
    print
    print tab(tabCol%) "       Item name:  " + desc$
    print tab(tabCol%) "Quantity on hand:  " + str$(qty%)
    print tab(tabCol%) "   Reorder level:  " + str$(reorder%)
    print tab(tabCol%) "      Unit price:  " + str$(price!)
end procedure

procedure printListHeader()
    cls
    print tab(25) "I N V E N T O R Y   L I S T I N G"; tab(65); str$(partCount%) + "items"
    print "                                          Quantity       Reorder"
    print " Partno           Description             on hand         level"
    locate 25, 1
    print "Press the AnyKey to scroll listing...";
end procedure

procedure printInventoryLine(partNum%, desc$, qty%, reorder%)
    print str$(partNum%) + "  " + desc$ + "   " + str$(qty%) + "          " + str$(reorder%)
end procedure

procedure printReorderHeader()
    cls
    locate 1, tabCol%
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
    locate 4, tabCol%
    print "Adding or Overwriting a Record"
    locate 8, tabCol%
    print "Record/Partno" + str$(partNum%)
    locate 11, 39
    print "------------------------------"
    locate 10, tabCol%
    input "      Description"; desc$
    locate 12, tabCol%
    input "Quantity in stock"; qty%
    locate 14, tabCol%
    input "    Reorder level"; reorder%
    locate 16, tabCol%
    input "       Unit price"; price!
    locate 18, tabCol%
    print "Is information correct (Y/N)?"
end procedure

procedure showAddStockScreen(partNum%, desc$, qty%, reorder%)
    cls
    locate 4, 25
    print "Add to an inventory part number"
    locate 5, 25
    print "==============================="
    locate 8, tabCol%
    print "     Part number: " + str$(partNum%)
    locate 9, tabCol%
    print "Item description: " + desc$
    locate 10, tabCol%
    print "Quantity on hand: " + str$(qty%)
    locate 11, tabCol%
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
    locate 4, tabCol%
    print "Subtract an inventory part number"
    locate 5, tabCol%
    print "================================="
    locate 8, tabCol%
    print "         Part number: " + str$(partNum%)
    locate 9, tabCol%
    print "    Item description: " + desc$
    locate 10, tabCol%
    print "    Quantity on hand: " + str$(qty%)
    locate 11, tabCol%
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
    cls
    locate 10, tabCol%
    partStr$ = readPartNumberInput$()
    part% = val(partStr$)
    if partInRange%(part%) = 0 then
        showBadPartNumber()
        waitAnyKey()
        return
    end if
    let p = inv[part%]
    if isEmpty%(p.flag) = 0 then
        locate 12, tabCol%
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
    printListHeader()
    scrollCount% = 0
    for i% = 1 to partCount%
        let p = inv[i%]
        printInventoryLine(i%, p.desc, p.qty, p.reorder)
        scrollCount% = scrollCount% + 1
        if scrollCount% = 20 then
            waitAnyKey()
            scrollCount% = 0
        end if
    end for
end procedure

procedure addStock()
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
        locate 14, tabCol%
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
        locate 14, tabCol%
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
        locate 16, tabCol%
    end if
    print "quantity now" + str$(p.qty) + " reorder level" + str$(p.reorder)
    inv[part%] = p
end procedure

procedure reorderReport()
    printReorderHeader()
    reportLineCount% = 0
    for i% = 1 to partCount%
        let p = inv[i%]
        if p.qty < p.reorder then
            printReorderLine(i%, p.desc, p.qty, p.reorder)
            reportLineCount% = reportLineCount% + 1
            if reportLineCount% > 15 then
                waitAnyKey()
                reportLineCount% = 0
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
    let p = inv[1]
    if asc(p.flag) = 0 then
        for i% = 1 to partCount%
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
                    // so it was never anything but a second spelling of
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
loop

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

### `tutorial/inventory.bas`

```basic

10 ' BASCAL generated BASIC
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Maps an ERR code to its classic MBASIC/GW-BASIC/BASCOM message. Compiles
40 ' and links on a real IBM BASIC Compiler 2.00 as ERROR$, but silently
50 ' returns an empty string at runtime (verified under dosbox-x) -- so BASCAL
60 ' ships a working implementation.
70 '
80 ' Covers the classic error codes an ON ERROR GOTO + ERR handler is
90 ' realistically going to hit -- not the full table, but every code common
100 ' enough to be worth a real message instead of falling through to the
110 ' generic one.
120 '
130 ' Deliberately NOT a scalar method (see GitHub issue #41, which asked for
140 ' this decision to be recorded either way): code% is an opaque lookup key,
150 ' not a value the call is naturally "operating on" the way ltrim$/rtrim$/
160 ' ucase$/lcase$ operate on their string -- code%.error() would read as if
170 ' the *error code itself* has a message, when really this is a lookup
180 ' table keyed by that code. Stays an ordinary function.

190 ' ============================================================
200 ' INVENTORY.BCL -- Random-Access Inventory Program
210 '
220 ' A BASCAL reconstruction of "Example program for RANDOM ACCESS
230 ' FILE study", by fhb, 8/19/98, from Joseph Sixpack's GW-BASIC
240 ' programs page (part of his "Last Book of GW-Basic" collection):
250 ' http://www.geocities.ws/joseph_sixpack/binventory.html
260 ' fhb's own header comment credits the original as "suggested
270 ' from MS-BASIC manual".
280 '
290 ' This is a reconstruction, not a line-by-line port -- some
300 ' original pieces have no BASCAL equivalent and were dropped
310 ' rather than approximated:
320 ' - The GOTO-driven "subroutine roadmap" dispatcher at the top
330 ' of fhb's listing (a `LIST 110-320` etc. navigation aid for
340 ' editing in the GW-BASIC interpreter) has no meaning once the
350 ' program is structured into named function/procedure blocks.
360 ' - `KEY OFF` / `KEY I,""` (clearing the function-key soft-label
370 ' row) and `VIEW PRINT` (scroll-region windowing for the list
380 ' screen) are interpreter/console features BASCAL doesn't
390 ' expose.
400 ' - fhb's own hand-rolled numeric-ERR-code-to-message lookup table
410 ' (ERR=1 "Input value overflow", ERR=2 "Syntax error", ... ERR=25)
420 ' is replaced below by BASCAL's com.bascal.stdlib.error library
430 ' (ERROR$(code%)) -- same idea, BASCAL's own table; it still
440 ' doesn't decode ERL, which errorTrap() reports as the raw line
450 ' number.
460 ' - fhb's one-time "hidden" datafile initializer (PUT-ing 100
470 ' blank, CHR$(255)-flagged records) is reproduced below as
480 ' initializeInventoryFileIfNew(), called once at program entry --
490 ' inven.dat no longer has to be pre-populated by hand.
500 ' - The three original tab-position constants (T=20, U=25,
510 ' V=30) are collapsed into a single `tabCol% = 20`; a couple of
520 ' screens that used U=25 in the original (see showAddStockScreen
530 ' below) keep 25 as a literal rather than reusing tabCol%.
540 '
550 ' Tracks parts in a fixed 100-record file: check status, add,
560 ' edit, add/subtract stock, and a reorder report.
570 '
580 ' Error handling uses try/catch (GitHub issue #60), not the raw `on
590 ' error goto` / `resume next` fhb's original relies on: a failed menu
600 ' action is abandoned outright and the program returns straight to the
610 ' main menu, rather than resuming at the exact instruction after
620 ' whatever failed -- see reportInventoryError() below and
630 ' tutorial/inventory_try_catch.draft's own header comment for why. This
640 ' is a real, deliberate behavior change from an earlier on-error-goto
650 ' version of this file, which *was* verified against real BASCOM 2.00
660 ' under dosbox-x (only with the /E and /X switches -- error trapping
670 ' isn't linked in by default); the try/catch shape below transpiles to
680 ' the same ON ERROR GOTO/RESUME primitives BASCOM accepts, but hasn't
690 ' itself been independently re-verified against a real BASCOM compile.
700 ' ============================================================

710 ' BASCAL-ism: the record/file DSL. `record ... end record` plus
720 ' `file ... as ... = open(...)` below replace fhb's manual
730 ' FIELD #1,1 AS F$,30 AS D$,2 AS Q$,... buffer layout entirely --
740 ' bcc computes the field widths and record LEN from this
750 ' declaration and generates the FIELD statement itself. Named
760 ' field access (`p.flag`, `p.qty`, ...) and whole-record
770 ' read/write via `inv[n]` (see checkPart() below) replace fhb's
780 ' manual GET/PUT plus LSET/RSET and MKI$/MKS$/CVI$/CVS$ packing.

790 ' BASCAL-ism: `const` is a real compile-time constant, not a plain
800 ' variable assignment like fhb's `N=100` / `T=20` -- it can never
810 ' be reassigned, and resolves to the same value everywhere,
820 ' including inside every function/procedure below, with no
830 ' `global` declaration needed.
840 partcount% = 100
850 tabcol% = 20

860 ' `file ... = open(...)` is sugar for OPEN ... FOR RANDOM AS #n
870 ' LEN = <record width> plus the FIELD statement fhb wrote out by
880 ' hand at his line 550. Wrapped in its own try/catch: a file that
890 ' exists but can't be opened for random access (permissions, a
900 ' read-only inven.dat, disk full on the fallback create) is a real,
910 ' trappable error (code 75, "Path/File access error") on both
920 ' targets now, not a hard crash -- report it and exit cleanly
930 ' instead of leaving the program to fail confusingly the first time
940 ' something tries to use an `inv` that was never actually opened.
950 ON ERROR GOTO 1020
960 BCC_TRY_0001_PENDING% = 0
970     ' file inv as Part = open(...)  [39 bytes/record]
980     OPEN "inven.dat" FOR RANDOM AS #1 LEN = 39
990     FIELD #1, 1 AS invflagbuf$, 30 AS invdescbuf$, 2 AS invqtybuf$, 2 AS invreorderbuf$, 4 AS invpricebuf$
1000 ON ERROR GOTO 0
1010 GOTO 1160
1020     BCC_TRY_0001_PENDING% = ERR
1030     err% = ERR
1040     erl% = ERL
1050     RESUME 1060
1060 ON ERROR GOTO 1140
1070     errorCode0% = err%
1080     GOSUB 2470
1090     PRINT "could not open inven.dat: " + errorResult0$
1100     END
1110     BCC_TRY_0001_PENDING% = 0
1120     ON ERROR GOTO 0
1130     GOTO 1160
1140     BCC_TRY_0001_PENDING% = ERR
1150     RESUME 1160
1160 ON ERROR GOTO 0
1170     IF BCC_TRY_0001_PENDING% <> 0 THEN ERROR BCC_TRY_0001_PENDING%
1180 REM END TRY

1190 ' -------------------- Pure functions (no file access) --------------------

1200 ' BASCAL-ism: `function ... end function` with `return` replaces
1210 ' fhb's convention of a GOSUB target plus a bare RETURN -- there's
1220 ' no separate "subroutine label" and no shared/global result
1230 ' variable to manage by hand; `isEmpty%(...)` is called like an
1240 ' ordinary expression at every use below (e.g. `isEmpty%(p.flag)`).
1250 ' A record whose flag byte is CHR$(255) is an empty/never-used slot.

1260 ' BASCAL-ism: `&&` and `||` are short-circuit AND/OR -- real
1270 ' MBASIC/BASCOM only has bitwise AND/OR (which fhb relies on here
1280 ' too, since `PART!<1 OR PART!>N!` never short-circuits anyway).
1290 ' BASCAL lowers `&&`/`||` into the equivalent branching so the
1300 ' short-circuit *is* real at the generated-BASIC level; see the
1310 ' manual's "Short-Circuit && and ||" section
1320 ' (https://johnjoeallen.github.io/bascal/manual/).

1330 ' -------------------- Keyboard input --------------------

1340 ' BASCAL-ism: `do ... loop until` is a structured post-check loop
1350 ' replacing fhb's `730 KP$=INKEY$:IF KP$="" THEN 730` GOTO-polling
1360 ' idiom. `inkey$` itself is the real INKEY$ builtin passed straight
1370 ' through, resolving correctly from inside a function/procedure
1380 ' body like this one -- every menu action below calls
1390 ' readKey$()/waitAnyKey() rather than polling INKEY$ inline.

1400 ' -------------------- Display procedures --------------------

1410 ' byref scalar parameters: gatherPartDetails writes the four editable
1420 ' fields for a part directly back into the caller's variables.

1430 ' -------------------- Menu actions --------------------

1440 ' fhb's own one-time "hidden" datafile initializer PUT-ing 100 blank,
1450 ' CHR$(255)-flagged records (see the header note above) -- reproduced
1460 ' here so inven.dat no longer has to be pre-populated by hand before
1470 ' running this program. A brand-new file OPEN created just now (rather
1480 ' than one that already existed) reads back as all-zero bytes: record
1490 ' 1's flag byte is CHR$(0), never CHR$(255) -- the one signal an
1500 ' already-populated file (whose record 1 flag is always either
1510 ' CHR$(255), still an empty slot, or a real part's own "1") could never
1520 ' produce, so it's what isEmpty%() itself can't use (see its own
1530 ' header note) but this one-time check safely can.

1540 ' -------------------- Program entry --------------------

1550 CLS
1560 GOSUB 9160

1570     GOSUB 4200
1580     GOSUB 4050
1590     kp$ = readkeyResult0$
1600     IF (INSTR("1234567cCeElLaAsSrRxX", kp$) <> 0) = 0 THEN GOTO 2300
1610         ' BASCAL-ism: `select case` replaces fhb's chain of eight
1620         ' `IF VAL(KP$)=n OR KP$="x" OR KP$="X" THEN GOTO ...` lines
1630         ' (his 770-840) with one multi-way dispatch.
1640         '
1650         ' BASCAL-ism: `try`/`catch` (issue #60) replaces fhb's own global
1660         ' `ON ERROR GOTO` trap. A failed menu action is abandoned outright
1670         ' here -- the `catch` below runs, then execution continues right
1680         ' after `end try`, back at `loop until` -- rather than resuming at
1690         ' the exact instruction after whatever failed inside checkPart()/
1700         ' editRecord()/etc. the way fhb's `RESUME NEXT` did. See
1710         ' reportInventoryError() below and tutorial/inventory_try_catch.
1720         ' draft's own header comment for why that arbitrary resume-point
1730         ' behavior isn't something try/catch reproduces.
1740         ON ERROR GOTO 2140
1750         BCC_TRY_0004_PENDING% = 0
1760             BCCT6$ = kp$
1770             IF (BCCT6$ = "1" OR BCCT6$ = "c" OR BCCT6$ = "C") <> 0 THEN GOTO 1850
1780             IF (BCCT6$ = "2" OR BCCT6$ = "e" OR BCCT6$ = "E") <> 0 THEN GOTO 1870
1790             IF (BCCT6$ = "3" OR BCCT6$ = "l" OR BCCT6$ = "L") <> 0 THEN GOTO 1890
1800             IF (BCCT6$ = "4" OR BCCT6$ = "a" OR BCCT6$ = "A") <> 0 THEN GOTO 1910
1810             IF (BCCT6$ = "5" OR BCCT6$ = "s" OR BCCT6$ = "S") <> 0 THEN GOTO 1930
1820             IF (BCCT6$ = "6" OR BCCT6$ = "r" OR BCCT6$ = "R") <> 0 THEN GOTO 1950
1830             IF (BCCT6$ = "7" OR BCCT6$ = "x" OR BCCT6$ = "X") <> 0 THEN GOTO 1970
1840             GOTO 2110
1850                 GOSUB 5730
1860                 GOTO 2110
1870                 GOSUB 6270
1880                 GOTO 2110
1890                 GOSUB 6960
1900                 GOTO 2110
1910                 GOSUB 7320
1920                 GOTO 2110
1930                 GOSUB 8000
1940                 GOTO 2110
1950                 GOSUB 8770
1960                 GOTO 2110
1970                 ' BASCAL-ism: `inv.close()` is sugar for `CLOSE #1`,
1980                 ' matching fhb's own `90 CLOSE:SYSTEM`. fhb's original
1990                 ' also had a separate "Quit to BASIC" option (his own
2000                 ' 7, returning to the interpreter's command prompt
2010                 ' rather than exiting to DOS) -- dropped here: a
2020                 ' compiled program has no interpreter to return to,
2030                 ' so it was never anything but a second spelling of
2040                 ' this same close-and-exit action.
2050                 ' inv.close()
2060                 CLOSE #1
2070                 COLOR 7, 0
2080                 CLS
2090                 SYSTEM
2100                 GOTO 2110
2110             REM END SELECT
2120         ON ERROR GOTO 0
2130         GOTO 2270
2140             BCC_TRY_0004_PENDING% = ERR
2150             err% = ERR
2160             erl% = ERL
2170             RESUME 2180
2180         ON ERROR GOTO 2250
2190             reportinventoryerrorErr0% = err%
2200             reportinventoryerrorErl0% = erl%
2210             GOSUB 9490
2220             BCC_TRY_0004_PENDING% = 0
2230             ON ERROR GOTO 0
2240             GOTO 2270
2250             BCC_TRY_0004_PENDING% = ERR
2260             RESUME 2270
2270         ON ERROR GOTO 0
2280             IF BCC_TRY_0004_PENDING% <> 0 THEN ERROR BCC_TRY_0004_PENDING%
2290         REM END TRY
2300     REM END IF
2310     GOTO 1570
2320 REM END DO

2330 ' -------------------- Error handling --------------------
2340 ' err%/erl% are ordinary locals scoped to the `catch` block above, not
2350 ' aliases for the ambient (readable-anywhere) `err`/`erl` pseudo-
2360 ' variables `on error goto` uses -- see `Statement::TryCatch`'s own doc
2370 ' comment in ast.rs. Passed straight through to ERROR$ here like fhb's
2380 ' own ERR/ERL (his 3390: "an error on line";ERL), decoded through
2390 ' BASCAL's own com.bascal.stdlib.error (ERROR$) instead of fhb's
2400 ' hand-rolled lookup table -- see the header note above. try/catch
2410 ' itself isn't documented in the manual yet (GitHub issue #60 tracks
2420 ' the still-unfinished C-target work; the manual page can follow once
2430 ' that lands) -- see ast.rs's own `Statement::TryCatch` doc comment for
2440 ' the full semantics meanwhile.
2450 END

2460 ' function error$(code%)
2470     BCCT8% = errorCode0%
2480     IF (BCCT8% = 2) <> 0 THEN GOTO 2820
2490     IF (BCCT8% = 3) <> 0 THEN GOTO 2850
2500     IF (BCCT8% = 4) <> 0 THEN GOTO 2880
2510     IF (BCCT8% = 5) <> 0 THEN GOTO 2910
2520     IF (BCCT8% = 6) <> 0 THEN GOTO 2940
2530     IF (BCCT8% = 7) <> 0 THEN GOTO 2970
2540     IF (BCCT8% = 9) <> 0 THEN GOTO 3000
2550     IF (BCCT8% = 10) <> 0 THEN GOTO 3030
2560     IF (BCCT8% = 11) <> 0 THEN GOTO 3060
2570     IF (BCCT8% = 13) <> 0 THEN GOTO 3090
2580     IF (BCCT8% = 14) <> 0 THEN GOTO 3120
2590     IF (BCCT8% = 19) <> 0 THEN GOTO 3150
2600     IF (BCCT8% = 20) <> 0 THEN GOTO 3180
2610     IF (BCCT8% = 24) <> 0 THEN GOTO 3210
2620     IF (BCCT8% = 25) <> 0 THEN GOTO 3240
2630     IF (BCCT8% = 27) <> 0 THEN GOTO 3270
2640     IF (BCCT8% = 52) <> 0 THEN GOTO 3300
2650     IF (BCCT8% = 53) <> 0 THEN GOTO 3330
2660     IF (BCCT8% = 54) <> 0 THEN GOTO 3360
2670     IF (BCCT8% = 55) <> 0 THEN GOTO 3390
2680     IF (BCCT8% = 57) <> 0 THEN GOTO 3420
2690     IF (BCCT8% = 58) <> 0 THEN GOTO 3450
2700     IF (BCCT8% = 61) <> 0 THEN GOTO 3480
2710     IF (BCCT8% = 62) <> 0 THEN GOTO 3510
2720     IF (BCCT8% = 63) <> 0 THEN GOTO 3540
2730     IF (BCCT8% = 64) <> 0 THEN GOTO 3570
2740     IF (BCCT8% = 67) <> 0 THEN GOTO 3600
2750     IF (BCCT8% = 68) <> 0 THEN GOTO 3630
2760     IF (BCCT8% = 70) <> 0 THEN GOTO 3660
2770     IF (BCCT8% = 71) <> 0 THEN GOTO 3690
2780     IF (BCCT8% = 72) <> 0 THEN GOTO 3720
2790     IF (BCCT8% = 75) <> 0 THEN GOTO 3750
2800     IF (BCCT8% = 76) <> 0 THEN GOTO 3780
2810     GOTO 3810
2820         errorResult0$ = "Syntax error"
2830         RETURN
2840         GOTO 3830
2850         errorResult0$ = "RETURN without GOSUB"
2860         RETURN
2870         GOTO 3830
2880         errorResult0$ = "Out of DATA"
2890         RETURN
2900         GOTO 3830
2910         errorResult0$ = "Illegal function call"
2920         RETURN
2930         GOTO 3830
2940         errorResult0$ = "Overflow"
2950         RETURN
2960         GOTO 3830
2970         errorResult0$ = "Out of memory"
2980         RETURN
2990         GOTO 3830
3000         errorResult0$ = "Subscript out of range"
3010         RETURN
3020         GOTO 3830
3030         errorResult0$ = "Duplicate Definition"
3040         RETURN
3050         GOTO 3830
3060         errorResult0$ = "Division by zero"
3070         RETURN
3080         GOTO 3830
3090         errorResult0$ = "Type mismatch"
3100         RETURN
3110         GOTO 3830
3120         errorResult0$ = "Out of string space"
3130         RETURN
3140         GOTO 3830
3150         errorResult0$ = "No RESUME"
3160         RETURN
3170         GOTO 3830
3180         errorResult0$ = "RESUME without error"
3190         RETURN
3200         GOTO 3830
3210         errorResult0$ = "Device timeout"
3220         RETURN
3230         GOTO 3830
3240         errorResult0$ = "Device fault"
3250         RETURN
3260         GOTO 3830
3270         errorResult0$ = "Out of paper"
3280         RETURN
3290         GOTO 3830
3300         errorResult0$ = "Bad file number"
3310         RETURN
3320         GOTO 3830
3330         errorResult0$ = "File not found"
3340         RETURN
3350         GOTO 3830
3360         errorResult0$ = "Bad file mode"
3370         RETURN
3380         GOTO 3830
3390         errorResult0$ = "File already open"
3400         RETURN
3410         GOTO 3830
3420         errorResult0$ = "Device I/O error"
3430         RETURN
3440         GOTO 3830
3450         errorResult0$ = "File already exists"
3460         RETURN
3470         GOTO 3830
3480         errorResult0$ = "Disk full"
3490         RETURN
3500         GOTO 3830
3510         errorResult0$ = "Input past end"
3520         RETURN
3530         GOTO 3830
3540         errorResult0$ = "Bad record number"
3550         RETURN
3560         GOTO 3830
3570         errorResult0$ = "Bad file name"
3580         RETURN
3590         GOTO 3830
3600         errorResult0$ = "Too many files"
3610         RETURN
3620         GOTO 3830
3630         errorResult0$ = "Device unavailable"
3640         RETURN
3650         GOTO 3830
3660         errorResult0$ = "Disk write protected"
3670         RETURN
3680         GOTO 3830
3690         errorResult0$ = "Disk not ready"
3700         RETURN
3710         GOTO 3830
3720         errorResult0$ = "Disk media error"
3730         RETURN
3740         GOTO 3830
3750         errorResult0$ = "Path/File access error"
3760         RETURN
3770         GOTO 3830
3780         errorResult0$ = "Path not found"
3790         RETURN
3800         GOTO 3830
3810         errorResult0$ = "Error " + STR$(errorCode0%)
3820         RETURN
3830     REM END SELECT
3840     RETURN
3850 ' end function error$

3860 ' function isempty%(flag$)
3870     isemptyResult0% = ASC(isemptyFlag0$) = 255
3880     RETURN
3890 ' end function isempty%

3900 ' function partinrange%(n%)
3910     IF (partinrangeN0% >= 1) = 0 THEN GOTO 3950
3920     IF (partinrangeN0% <= partcount%) = 0 THEN GOTO 3950
3930         partinrangeResult0% = 1
3940         RETURN
3950     REM END IF
3960     partinrangeResult0% = 0
3970     RETURN
3980 ' end function partinrange%

3990 ' function readpartnumberinput$()
4000     INPUT "Input part number"; readpartnumberinputS0$
4010     readpartnumberinputResult0$ = readpartnumberinputS0$
4020     RETURN
4030 ' end function readpartnumberinput$

4040 ' function readkey$()
4050         readkeyK0$ = INKEY$
4060         IF (readkeyK0$ <> "") = 0 THEN GOTO 4050
4070     REM END DO
4080     readkeyResult0$ = readkeyK0$
4090     RETURN
4100 ' end function readkey$

4110 ' procedure waitanykey()
4120     LOCATE 25, 10
4130     PRINT "Press the AnyKey to continue...";
4140         waitanykeyK0$ = INKEY$
4150         IF (waitanykeyK0$ <> "") = 0 THEN GOTO 4140
4160     REM END DO
4170     RETURN
4180 ' end procedure waitanykey

4190 ' procedure showmainmenu()
4200     CLS
4210     COLOR 14, 4
4220     CLS
4230     LOCATE 6, 1
4240     PRINT
4250     ' `tab(n)` passes straight through to real TAB(n), same as
4260     ' fhb's own `PRINT TAB(V) "..."` -- but only as a bare item in
4270     ' a PRINT list, juxtaposed or `;`-separated like here. Real
4280     ' BASCOM rejects `"literal" + tab(n) + ...` (TAB isn't a real
4290     ' string function you can concatenate); see printListHeader()
4300     ' and printReorderHeader() below, which need `;` between a
4310     ' preceding string and a `tab(n)` for exactly this reason.
4320     PRINT TAB(30)"Inventory Program"
4330     PRINT
4340     PRINT TAB(tabcol%)"1......C)heck a part"
4350     PRINT TAB(tabcol%)"2......E)dit/overwrite/add a part"
4360     PRINT TAB(tabcol%)("3......L)ist all" + STR$(partcount%)) + "parts"
4370     PRINT TAB(tabcol%)"4......A)dd stock"
4380     PRINT TAB(tabcol%)"5......S)ubtract stock"
4390     PRINT TAB(tabcol%)"6......R)eorder Report"
4400     PRINT
4410     PRINT TAB(tabcol%)"7......eX)it to system"
4420     RETURN
4430 ' end procedure showmainmenu

4440 ' procedure showbadpartnumber()
4450     CLS
4460     LOCATE 10, 10
4470     PRINT "Part number is out of permissable range of 1 to" + STR$(partcount%)
4480     RETURN
4490 ' end procedure showbadpartnumber

4500 ' procedure showrangeretrymessage()
4510     LOCATE 10, 15
4520     PRINT "The Part number is out of permissable range of 1 to" + STR$(partcount%)
4530     LOCATE 25, 15
4540     PRINT "Press the Anykey to reenter part number...";
4550     RETURN
4560 ' end procedure showrangeretrymessage

4570 ' procedure shownullentrymessage(partstr$)
4580     LOCATE 10, tabcol%
4590     PRINT ("Part number " + shownullentrymessagePartStr0$) + " is a null entry"
4600     RETURN
4610 ' end procedure shownullentrymessage

4620 ' procedure showpartstatus(partnum%, desc$, qty%, reorder%, price!)
4630     CLS
4640     LOCATE 5, 1
4650     PRINT TAB(tabcol%)"Inventory Status for Individual Part Number"
4660     PRINT TAB(tabcol%)"==========================================="
4670     PRINT
4680     PRINT
4690     PRINT TAB(tabcol%)"     Part number:  " + STR$(showpartstatusPartNum0%)
4700     PRINT
4710     PRINT TAB(tabcol%)"       Item name:  " + showpartstatusDesc0$
4720     PRINT TAB(tabcol%)"Quantity on hand:  " + STR$(showpartstatusQty0%)
4730     PRINT TAB(tabcol%)"   Reorder level:  " + STR$(showpartstatusReorder0%)
4740     PRINT TAB(tabcol%)"      Unit price:  " + STR$(showpartstatusPrice0!)
4750     RETURN
4760 ' end procedure showpartstatus

4770 ' procedure printlistheader()
4780     CLS
4790     PRINT TAB(25)"I N V E N T O R Y   L I S T I N G"; TAB(65); STR$(partcount%) + "items"
4800     PRINT "                                          Quantity       Reorder"
4810     PRINT " Partno           Description             on hand         level"
4820     LOCATE 25, 1
4830     PRINT "Press the AnyKey to scroll listing...";
4840     RETURN
4850 ' end procedure printlistheader

4860 ' procedure printinventoryline(partnum%, desc$, qty%, reorder%)
4870     PRINT (((((STR$(printinventorylinePartNum0%) + "  ") + printinventorylineDesc0$) + "   ") + STR$(printinventorylineQty0%)) + "          ") + STR$(printinventorylineReorder0%)
4880     RETURN
4890 ' end procedure printinventoryline

4900 ' procedure printreorderheader()
4910     CLS
4920     LOCATE 1, tabcol%
4930     PRINT "Reorder Report"; TAB(55); DATE$
4940     PRINT
4950     PRINT "                                             Quantity       Reorder"
4960     PRINT "    Partno           Description             on hand         level"
4970     PRINT "   =======  ==============================   ========       ======="
4980     RETURN
4990 ' end procedure printreorderheader

5000 ' procedure printreorderline(partnum%, desc$, qty%, reorder%)
5010     PRINT (((((("  " + STR$(printreorderlinePartNum0%)) + "  ") + printreorderlineDesc0$) + "   ") + STR$(printreorderlineQty0%)) + "          ") + STR$(printreorderlineReorder0%)
5020     RETURN
5030 ' end procedure printreorderline

5040 ' procedure gatherpartdetails(partnum%, desc$, qty%, reorder%, price!)
5050     CLS
5060     LOCATE 4, tabcol%
5070     PRINT "Adding or Overwriting a Record"
5080     LOCATE 8, tabcol%
5090     PRINT "Record/Partno" + STR$(gatherpartdetailsPartNum0%)
5100     LOCATE 11, 39
5110     PRINT "------------------------------"
5120     LOCATE 10, tabcol%
5130     INPUT "      Description"; gatherpartdetailsDesc0$
5140     LOCATE 12, tabcol%
5150     INPUT "Quantity in stock"; gatherpartdetailsQty0%
5160     LOCATE 14, tabcol%
5170     INPUT "    Reorder level"; gatherpartdetailsReorder0%
5180     LOCATE 16, tabcol%
5190     INPUT "       Unit price"; gatherpartdetailsPrice0!
5200     LOCATE 18, tabcol%
5210     PRINT "Is information correct (Y/N)?"
5220     RETURN
5230 ' end procedure gatherpartdetails

5240 ' procedure showaddstockscreen(partnum%, desc$, qty%, reorder%)
5250     CLS
5260     LOCATE 4, 25
5270     PRINT "Add to an inventory part number"
5280     LOCATE 5, 25
5290     PRINT "==============================="
5300     LOCATE 8, tabcol%
5310     PRINT "     Part number: " + STR$(showaddstockscreenPartNum0%)
5320     LOCATE 9, tabcol%
5330     PRINT "Item description: " + showaddstockscreenDesc0$
5340     LOCATE 10, tabcol%
5350     PRINT "Quantity on hand: " + STR$(showaddstockscreenQty0%)
5360     LOCATE 11, tabcol%
5370     PRINT "   Reorder Level: " + STR$(showaddstockscreenReorder0%)
5380     RETURN
5390 ' end procedure showaddstockscreen

5400 ' procedure shownegativeqtywarning()
5410     LOCATE 17, 15
5420     PRINT "The quantity to add must NOT be a negative number"
5430     LOCATE 25, 1
5440     PRINT "Please press the Anykey to reenter quantity to add...";
5450     RETURN
5460 ' end procedure shownegativeqtywarning

5470 ' procedure showsubtractstockscreen(partnum%, desc$, qty%, reorder%)
5480     CLS
5490     LOCATE 4, tabcol%
5500     PRINT "Subtract an inventory part number"
5510     LOCATE 5, tabcol%
5520     PRINT "================================="
5530     LOCATE 8, tabcol%
5540     PRINT "         Part number: " + STR$(showsubtractstockscreenPartNum0%)
5550     LOCATE 9, tabcol%
5560     PRINT "    Item description: " + showsubtractstockscreenDesc0$
5570     LOCATE 10, tabcol%
5580     PRINT "    Quantity on hand: " + STR$(showsubtractstockscreenQty0%)
5590     LOCATE 11, tabcol%
5600     PRINT "       Reorder Level: " + STR$(showsubtractstockscreenReorder0%)
5610     RETURN
5620 ' end procedure showsubtractstockscreen

5630 ' procedure showoversubtractwarning(onhand%)
5640     LOCATE 17, 5
5650     PRINT "The quantity to SUBTRACT must NOT result in NEGATIVE inventory"
5660     LOCATE 18, 5
5670     PRINT ("Only" + STR$(showoversubtractwarningOnHand0%)) + " IN STOCK"
5680     LOCATE 25, 1
5690     PRINT "Please press the Anykey to reenter quantity to subtract...";
5700     RETURN
5710 ' end procedure showoversubtractwarning

5720 ' procedure checkpart()
5730     GOSUB 4000
5740     checkpartPartStr0$ = readpartnumberinputResult0$
5750     checkpartPart0% = VAL(checkpartPartStr0$)
5760     partinrangeN0% = checkpartPart0%
5770     GOSUB 3910
5780     IF (partinrangeResult0% = 0) = 0 THEN GOTO 5820
5790         GOSUB 4450
5800         GOSUB 4120
5810         RETURN
5820     REM END IF
5830     ' BASCAL-ism: `let p = inv[part%]` reads record `part%` of the
5840     ' `inv` file into a local record variable `p` -- one expression
5850     ' for what fhb's `GET #1, PART!` plus five separate field reads
5860     ' (F$, D$, CVI(Q$), CVI(R$), CVS(P$)) did by hand. The write
5870     ' side, `inv[part%] = { ... }` (see editRecord() below), is the
5880     ' same sugar for PUT plus the LSET/MKx$ packing it replaces.
5890     ' let p = inv[...]  (whole-record read)
5900     GET #1, checkpartPart0%
5910     checkpartPFlagTrimI0% = LEN(checkpartInvFlagBuf0$)
5920     IF (checkpartPFlagTrimI0% > 0) = 0 THEN GOTO 5960
5930     IF (MID$(checkpartInvFlagBuf0$, checkpartPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 5960
5940         checkpartPFlagTrimI0% = checkpartPFlagTrimI0% - 1
5950         GOTO 5920
5960     REM END WHILE
5970     checkpartPFlag0$ = LEFT$(checkpartInvFlagBuf0$, checkpartPFlagTrimI0%)
5980     checkpartPDescTrimI0% = LEN(checkpartInvDescBuf0$)
5990     IF (checkpartPDescTrimI0% > 0) = 0 THEN GOTO 6030
6000     IF (MID$(checkpartInvDescBuf0$, checkpartPDescTrimI0%, 1) = " ") = 0 THEN GOTO 6030
6010         checkpartPDescTrimI0% = checkpartPDescTrimI0% - 1
6020         GOTO 5990
6030     REM END WHILE
6040     checkpartPDesc0$ = LEFT$(checkpartInvDescBuf0$, checkpartPDescTrimI0%)
6050     checkpartPQty0% = CVI(checkpartInvQtyBuf0$)
6060     checkpartPReorder0% = CVI(checkpartInvReorderBuf0$)
6070     checkpartPPrice0! = CVS(checkpartInvPriceBuf0$)
6080     isemptyFlag0$ = checkpartPFlag0$
6090     GOSUB 3870
6100     IF (isemptyResult0%) = 0 THEN GOTO 6160
6110         CLS
6120         LOCATE 10, 18
6130         PRINT ("Part number" + STR$(checkpartPart0%)) + "is still a null entry at this time"
6140         GOSUB 4120
6150         RETURN
6160     REM END IF
6170     showpartstatusPartNum0% = checkpartPart0%
6180     showpartstatusDesc0$ = checkpartPDesc0$
6190     showpartstatusQty0% = checkpartPQty0%
6200     showpartstatusReorder0% = checkpartPReorder0%
6210     showpartstatusPrice0! = checkpartPPrice0!
6220     GOSUB 4630
6230     GOSUB 4120
6240     RETURN
6250 ' end procedure checkpart

6260 ' procedure editrecord()
6270     CLS
6280     LOCATE 10, tabcol%
6290     GOSUB 4000
6300     editrecordPartStr0$ = readpartnumberinputResult0$
6310     editrecordPart0% = VAL(editrecordPartStr0$)
6320     partinrangeN0% = editrecordPart0%
6330     GOSUB 3910
6340     IF (partinrangeResult0% = 0) = 0 THEN GOTO 6380
6350         GOSUB 4450
6360         GOSUB 4120
6370         RETURN
6380     REM END IF
6390     ' let p = inv[...]  (whole-record read)
6400     GET #1, editrecordPart0%
6410     editrecordPFlagTrimI0% = LEN(editrecordInvFlagBuf0$)
6420     IF (editrecordPFlagTrimI0% > 0) = 0 THEN GOTO 6460
6430     IF (MID$(editrecordInvFlagBuf0$, editrecordPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 6460
6440         editrecordPFlagTrimI0% = editrecordPFlagTrimI0% - 1
6450         GOTO 6420
6460     REM END WHILE
6470     editrecordPFlag0$ = LEFT$(editrecordInvFlagBuf0$, editrecordPFlagTrimI0%)
6480     editrecordPDescTrimI0% = LEN(editrecordInvDescBuf0$)
6490     IF (editrecordPDescTrimI0% > 0) = 0 THEN GOTO 6530
6500     IF (MID$(editrecordInvDescBuf0$, editrecordPDescTrimI0%, 1) = " ") = 0 THEN GOTO 6530
6510         editrecordPDescTrimI0% = editrecordPDescTrimI0% - 1
6520         GOTO 6490
6530     REM END WHILE
6540     editrecordPDesc0$ = LEFT$(editrecordInvDescBuf0$, editrecordPDescTrimI0%)
6550     editrecordPQty0% = CVI(editrecordInvQtyBuf0$)
6560     editrecordPReorder0% = CVI(editrecordInvReorderBuf0$)
6570     editrecordPPrice0! = CVS(editrecordInvPriceBuf0$)
6580     isemptyFlag0$ = editrecordPFlag0$
6590     GOSUB 3870
6600     IF (isemptyResult0% = 0) = 0 THEN GOTO 6690
6610         LOCATE 12, tabcol%
6620         PRINT "Overwrite existing part data?"
6630         GOSUB 4050
6640         editrecordKp0$ = readkeyResult0$
6650         IF (editrecordKp0$ <> "Y") = 0 THEN GOTO 6680
6660         IF (editrecordKp0$ <> "y") = 0 THEN GOTO 6680
6670             RETURN
6680         REM END IF
6690     REM END IF

6700         gatherpartdetailsPartNum0% = editrecordPart0%
6710         gatherpartdetailsDesc0$ = editrecordEditDesc0$
6720         gatherpartdetailsQty0% = editrecordEditQty0%
6730         gatherpartdetailsReorder0% = editrecordEditReorder0%
6740         gatherpartdetailsPrice0! = editrecordEditPrice0!
6750         GOSUB 5050
6760         editrecordEditDesc0$ = gatherpartdetailsDesc0$
6770         editrecordEditQty0% = gatherpartdetailsQty0%
6780         editrecordEditReorder0% = gatherpartdetailsReorder0%
6790         editrecordEditPrice0! = gatherpartdetailsPrice0!
6800         GOSUB 4050
6810         editrecordKp0$ = readkeyResult0$
6820         IF (editrecordKp0$ = "Y") <> 0 THEN GOTO 6850
6830         IF (editrecordKp0$ = "y") <> 0 THEN GOTO 6850
6840         GOTO 6700
6850     REM END DO
6860     ' inv[...] = { ... }  (whole-record write)
6870     LSET editrecordInvFlagBuf0$ = "1"
6880     LSET editrecordInvDescBuf0$ = editrecordEditDesc0$
6890     LSET editrecordInvQtyBuf0$ = MKI$(editrecordEditQty0%)
6900     LSET editrecordInvReorderBuf0$ = MKI$(editrecordEditReorder0%)
6910     LSET editrecordInvPriceBuf0$ = MKS$(editrecordEditPrice0!)
6920     PUT #1, editrecordPart0%
6930     RETURN
6940 ' end procedure editrecord

6950 ' procedure listall()
6960     GOSUB 4780
6970     listallScrollCount0% = 0
6980     FOR listallI0% = 1 TO partcount%
6990         ' let p = inv[...]  (whole-record read)
7000         GET #1, listallI0%
7010         listallPFlagTrimI0% = LEN(listallInvFlagBuf0$)
7020         IF (listallPFlagTrimI0% > 0) = 0 THEN GOTO 7060
7030         IF (MID$(listallInvFlagBuf0$, listallPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 7060
7040             listallPFlagTrimI0% = listallPFlagTrimI0% - 1
7050             GOTO 7020
7060         REM END WHILE
7070         listallPFlag0$ = LEFT$(listallInvFlagBuf0$, listallPFlagTrimI0%)
7080         listallPDescTrimI0% = LEN(listallInvDescBuf0$)
7090         IF (listallPDescTrimI0% > 0) = 0 THEN GOTO 7130
7100         IF (MID$(listallInvDescBuf0$, listallPDescTrimI0%, 1) = " ") = 0 THEN GOTO 7130
7110             listallPDescTrimI0% = listallPDescTrimI0% - 1
7120             GOTO 7090
7130         REM END WHILE
7140         listallPDesc0$ = LEFT$(listallInvDescBuf0$, listallPDescTrimI0%)
7150         listallPQty0% = CVI(listallInvQtyBuf0$)
7160         listallPReorder0% = CVI(listallInvReorderBuf0$)
7170         listallPPrice0! = CVS(listallInvPriceBuf0$)
7180         printinventorylinePartNum0% = listallI0%
7190         printinventorylineDesc0$ = listallPDesc0$
7200         printinventorylineQty0% = listallPQty0%
7210         printinventorylineReorder0% = listallPReorder0%
7220         GOSUB 4870
7230         listallScrollCount0% = listallScrollCount0% + 1
7240         IF (listallScrollCount0% = 20) = 0 THEN GOTO 7270
7250             GOSUB 4120
7260             listallScrollCount0% = 0
7270         REM END IF
7280     NEXT listallI0%
7290     RETURN
7300 ' end procedure listall

7310 ' procedure addstock()
7320     CLS
7330     LOCATE 5, 25
7340     PRINT "A D D I N G   S T O C K"

7350         LOCATE 8, 25
7360         GOSUB 4000
7370         addstockPartStr0$ = readpartnumberinputResult0$
7380         addstockPart0% = VAL(addstockPartStr0$)
7390         partinrangeN0% = addstockPart0%
7400         GOSUB 3910
7410         addstockValidPart0% = partinrangeResult0%
7420         IF (addstockValidPart0% = 0) = 0 THEN GOTO 7450
7430             GOSUB 4510
7440             GOSUB 4050
7450         REM END IF
7460         IF (addstockValidPart0% <> 0) = 0 THEN GOTO 7350
7470     REM END DO

7480     ' let p = inv[...]  (whole-record read)
7490     GET #1, addstockPart0%
7500     addstockPFlagTrimI0% = LEN(addstockInvFlagBuf0$)
7510     IF (addstockPFlagTrimI0% > 0) = 0 THEN GOTO 7550
7520     IF (MID$(addstockInvFlagBuf0$, addstockPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 7550
7530         addstockPFlagTrimI0% = addstockPFlagTrimI0% - 1
7540         GOTO 7510
7550     REM END WHILE
7560     addstockPFlag0$ = LEFT$(addstockInvFlagBuf0$, addstockPFlagTrimI0%)
7570     addstockPDescTrimI0% = LEN(addstockInvDescBuf0$)
7580     IF (addstockPDescTrimI0% > 0) = 0 THEN GOTO 7620
7590     IF (MID$(addstockInvDescBuf0$, addstockPDescTrimI0%, 1) = " ") = 0 THEN GOTO 7620
7600         addstockPDescTrimI0% = addstockPDescTrimI0% - 1
7610         GOTO 7580
7620     REM END WHILE
7630     addstockPDesc0$ = LEFT$(addstockInvDescBuf0$, addstockPDescTrimI0%)
7640     addstockPQty0% = CVI(addstockInvQtyBuf0$)
7650     addstockPReorder0% = CVI(addstockInvReorderBuf0$)
7660     addstockPPrice0! = CVS(addstockInvPriceBuf0$)
7670     isemptyFlag0$ = addstockPFlag0$
7680     GOSUB 3870
7690     IF (isemptyResult0%) = 0 THEN GOTO 7740
7700         shownullentrymessagePartStr0$ = addstockPartStr0$
7710         GOSUB 4580
7720         GOSUB 4050
7730         RETURN
7740     REM END IF

7750         showaddstockscreenPartNum0% = addstockPart0%
7760         showaddstockscreenDesc0$ = addstockPDesc0$
7770         showaddstockscreenQty0% = addstockPQty0%
7780         showaddstockscreenReorder0% = addstockPReorder0%
7790         GOSUB 5250
7800         LOCATE 14, tabcol%
7810         INPUT " Quantity to add"; addstockAddStr0$
7820         addstockAddAmt0% = VAL(addstockAddStr0$)
7830         IF (addstockAddAmt0% < 0) = 0 THEN GOTO 7860
7840             GOSUB 5410
7850             GOSUB 4050
7860         REM END IF
7870         IF (addstockAddAmt0% >= 0) = 0 THEN GOTO 7750
7880     REM END DO

7890     addstockPQty0% = addstockPQty0% + addstockAddAmt0%
7900     ' inv[...] = p  (write back a let-bound record)
7910     LSET addstockInvFlagBuf0$ = addstockPFlag0$
7920     LSET addstockInvDescBuf0$ = addstockPDesc0$
7930     LSET addstockInvQtyBuf0$ = MKI$(addstockPQty0%)
7940     LSET addstockInvReorderBuf0$ = MKI$(addstockPReorder0%)
7950     LSET addstockInvPriceBuf0$ = MKS$(addstockPPrice0!)
7960     PUT #1, addstockPart0%
7970     RETURN
7980 ' end procedure addstock

7990 ' procedure subtractstock()
8000     CLS
8010     LOCATE 5, 20
8020     PRINT "S U B T R A C T I N G    S T O C K"

8030         LOCATE 8, 25
8040         GOSUB 4000
8050         subtractstockPartStr0$ = readpartnumberinputResult0$
8060         subtractstockPart0% = VAL(subtractstockPartStr0$)
8070         partinrangeN0% = subtractstockPart0%
8080         GOSUB 3910
8090         subtractstockValidPart0% = partinrangeResult0%
8100         IF (subtractstockValidPart0% = 0) = 0 THEN GOTO 8130
8110             GOSUB 4510
8120             GOSUB 4050
8130         REM END IF
8140         IF (subtractstockValidPart0% <> 0) = 0 THEN GOTO 8030
8150     REM END DO

8160     ' let p = inv[...]  (whole-record read)
8170     GET #1, subtractstockPart0%
8180     subtractstockPFlagTrimI0% = LEN(subtractstockInvFlagBuf0$)
8190     IF (subtractstockPFlagTrimI0% > 0) = 0 THEN GOTO 8230
8200     IF (MID$(subtractstockInvFlagBuf0$, subtractstockPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 8230
8210         subtractstockPFlagTrimI0% = subtractstockPFlagTrimI0% - 1
8220         GOTO 8190
8230     REM END WHILE
8240     subtractstockPFlag0$ = LEFT$(subtractstockInvFlagBuf0$, subtractstockPFlagTrimI0%)
8250     subtractstockPDescTrimI0% = LEN(subtractstockInvDescBuf0$)
8260     IF (subtractstockPDescTrimI0% > 0) = 0 THEN GOTO 8300
8270     IF (MID$(subtractstockInvDescBuf0$, subtractstockPDescTrimI0%, 1) = " ") = 0 THEN GOTO 8300
8280         subtractstockPDescTrimI0% = subtractstockPDescTrimI0% - 1
8290         GOTO 8260
8300     REM END WHILE
8310     subtractstockPDesc0$ = LEFT$(subtractstockInvDescBuf0$, subtractstockPDescTrimI0%)
8320     subtractstockPQty0% = CVI(subtractstockInvQtyBuf0$)
8330     subtractstockPReorder0% = CVI(subtractstockInvReorderBuf0$)
8340     subtractstockPPrice0! = CVS(subtractstockInvPriceBuf0$)
8350     isemptyFlag0$ = subtractstockPFlag0$
8360     GOSUB 3870
8370     IF (isemptyResult0%) = 0 THEN GOTO 8420
8380         shownullentrymessagePartStr0$ = subtractstockPartStr0$
8390         GOSUB 4580
8400         GOSUB 4050
8410         RETURN
8420     REM END IF

8430         showsubtractstockscreenPartNum0% = subtractstockPart0%
8440         showsubtractstockscreenDesc0$ = subtractstockPDesc0$
8450         showsubtractstockscreenQty0% = subtractstockPQty0%
8460         showsubtractstockscreenReorder0% = subtractstockPReorder0%
8470         GOSUB 5480
8480         LOCATE 14, tabcol%
8490         INPUT "Quantity to subtract"; subtractstockSubStr0$
8500         subtractstockSubAmt0% = VAL(subtractstockSubStr0$)
8510         subtractstockOverSubtract0% = 0
8520         IF (subtractstockSubAmt0% >= 0) = 0 THEN GOTO 8580
8530         IF ((subtractstockPQty0% - subtractstockSubAmt0%) < 0) = 0 THEN GOTO 8580
8540             subtractstockOverSubtract0% = 1
8550             showoversubtractwarningOnHand0% = subtractstockPQty0%
8560             GOSUB 5640
8570             GOSUB 4050
8580         REM END IF
8590         IF (subtractstockSubAmt0% >= 0) = 0 THEN GOTO 8430
8600         IF (subtractstockOverSubtract0% = 0) = 0 THEN GOTO 8430
8610     REM END DO

8620     subtractstockPQty0% = subtractstockPQty0% - subtractstockSubAmt0%
8630     IF (subtractstockPQty0% <= subtractstockPReorder0%) = 0 THEN GOTO 8650
8640         LOCATE 16, tabcol%
8650     REM END IF
8660     PRINT (("quantity now" + STR$(subtractstockPQty0%)) + " reorder level") + STR$(subtractstockPReorder0%)
8670     ' inv[...] = p  (write back a let-bound record)
8680     LSET subtractstockInvFlagBuf0$ = subtractstockPFlag0$
8690     LSET subtractstockInvDescBuf0$ = subtractstockPDesc0$
8700     LSET subtractstockInvQtyBuf0$ = MKI$(subtractstockPQty0%)
8710     LSET subtractstockInvReorderBuf0$ = MKI$(subtractstockPReorder0%)
8720     LSET subtractstockInvPriceBuf0$ = MKS$(subtractstockPPrice0!)
8730     PUT #1, subtractstockPart0%
8740     RETURN
8750 ' end procedure subtractstock

8760 ' procedure reorderreport()
8770     GOSUB 4910
8780     reorderreportReportLineCount0% = 0
8790     FOR reorderreportI0% = 1 TO partcount%
8800         ' let p = inv[...]  (whole-record read)
8810         GET #1, reorderreportI0%
8820         reorderreportPFlagTrimI0% = LEN(reorderreportInvFlagBuf0$)
8830         IF (reorderreportPFlagTrimI0% > 0) = 0 THEN GOTO 8870
8840         IF (MID$(reorderreportInvFlagBuf0$, reorderreportPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 8870
8850             reorderreportPFlagTrimI0% = reorderreportPFlagTrimI0% - 1
8860             GOTO 8830
8870         REM END WHILE
8880         reorderreportPFlag0$ = LEFT$(reorderreportInvFlagBuf0$, reorderreportPFlagTrimI0%)
8890         reorderreportPDescTrimI0% = LEN(reorderreportInvDescBuf0$)
8900         IF (reorderreportPDescTrimI0% > 0) = 0 THEN GOTO 8940
8910         IF (MID$(reorderreportInvDescBuf0$, reorderreportPDescTrimI0%, 1) = " ") = 0 THEN GOTO 8940
8920             reorderreportPDescTrimI0% = reorderreportPDescTrimI0% - 1
8930             GOTO 8900
8940         REM END WHILE
8950         reorderreportPDesc0$ = LEFT$(reorderreportInvDescBuf0$, reorderreportPDescTrimI0%)
8960         reorderreportPQty0% = CVI(reorderreportInvQtyBuf0$)
8970         reorderreportPReorder0% = CVI(reorderreportInvReorderBuf0$)
8980         reorderreportPPrice0! = CVS(reorderreportInvPriceBuf0$)
8990         IF (reorderreportPQty0% < reorderreportPReorder0%) = 0 THEN GOTO 9100
9000             printreorderlinePartNum0% = reorderreportI0%
9010             printreorderlineDesc0$ = reorderreportPDesc0$
9020             printreorderlineQty0% = reorderreportPQty0%
9030             printreorderlineReorder0% = reorderreportPReorder0%
9040             GOSUB 5010
9050             reorderreportReportLineCount0% = reorderreportReportLineCount0% + 1
9060             IF (reorderreportReportLineCount0% > 15) = 0 THEN GOTO 9090
9070                 GOSUB 4120
9080                 reorderreportReportLineCount0% = 0
9090             REM END IF
9100         REM END IF
9110     NEXT reorderreportI0%
9120     GOSUB 4120
9130     RETURN
9140 ' end procedure reorderreport

9150 ' procedure initializeinventoryfileifnew()
9160     ' let p = inv[...]  (whole-record read)
9170     GET #1, 1
9180     initializeinventoryfileifnewPFlagTrimI0% = LEN(initializeinventoryfileifnewInvFlagBuf0$)
9190     IF (initializeinventoryfileifnewPFlagTrimI0% > 0) = 0 THEN GOTO 9230
9200     IF (MID$(initializeinventoryfileifnewInvFlagBuf0$, initializeinventoryfileifnewPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 9230
9210         initializeinventoryfileifnewPFlagTrimI0% = initializeinventoryfileifnewPFlagTrimI0% - 1
9220         GOTO 9190
9230     REM END WHILE
9240     initializeinventoryfileifnewPFlag0$ = LEFT$(initializeinventoryfileifnewInvFlagBuf0$, initializeinventoryfileifnewPFlagTrimI0%)
9250     initializeinventoryfileifnewPDescTrimI0% = LEN(initializeinventoryfileifnewInvDescBuf0$)
9260     IF (initializeinventoryfileifnewPDescTrimI0% > 0) = 0 THEN GOTO 9300
9270     IF (MID$(initializeinventoryfileifnewInvDescBuf0$, initializeinventoryfileifnewPDescTrimI0%, 1) = " ") = 0 THEN GOTO 9300
9280         initializeinventoryfileifnewPDescTrimI0% = initializeinventoryfileifnewPDescTrimI0% - 1
9290         GOTO 9260
9300     REM END WHILE
9310     initializeinventoryfileifnewPDesc0$ = LEFT$(initializeinventoryfileifnewInvDescBuf0$, initializeinventoryfileifnewPDescTrimI0%)
9320     initializeinventoryfileifnewPQty0% = CVI(initializeinventoryfileifnewInvQtyBuf0$)
9330     initializeinventoryfileifnewPReorder0% = CVI(initializeinventoryfileifnewInvReorderBuf0$)
9340     initializeinventoryfileifnewPPrice0! = CVS(initializeinventoryfileifnewInvPriceBuf0$)
9350     IF (ASC(initializeinventoryfileifnewPFlag0$) = 0) = 0 THEN GOTO 9450
9360         FOR initializeinventoryfileifnewI0% = 1 TO partcount%
9370             ' inv[...] = { ... }  (whole-record write)
9380             LSET initializeinventoryfileifnewInvFlagBuf0$ = CHR$(255)
9390             LSET initializeinventoryfileifnewInvDescBuf0$ = ""
9400             LSET initializeinventoryfileifnewInvQtyBuf0$ = MKI$(0)
9410             LSET initializeinventoryfileifnewInvReorderBuf0$ = MKI$(0)
9420             LSET initializeinventoryfileifnewInvPriceBuf0$ = MKS$(0)
9430             PUT #1, initializeinventoryfileifnewI0%
9440         NEXT initializeinventoryfileifnewI0%
9450     REM END IF
9460     RETURN
9470 ' end procedure initializeinventoryfileifnew

9480 ' procedure reportinventoryerror(err%, erl%)
9490     LOCATE 25, 1
9500     errorCode0% = reportinventoryerrorErr0%
9510     GOSUB 2470
9520     PRINT (("There has been an error on line" + STR$(reportinventoryerrorErl0%)) + ": ") + errorResult0$
9530     GOSUB 4050
9540     reportinventoryerrorK0$ = readkeyResult0$
9550     RETURN
9560 ' end procedure reportinventoryerror

```

<!-- END generated tutorial source -->
