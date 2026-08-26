[Home](../../) / [Tutorials](../) / Case Study: Random-Access Inventory

<div class="prose" markdown="1">

Every other tutorial in this series is a short walkthrough of one feature. This one is different: it's a reconstruction of a real, complete program — ["Example program for RANDOM ACCESS FILE study"](http://www.geocities.ws/joseph_sixpack/binventory.md), written by fhb in 1998 for Joseph Sixpack's "Last Book of GW-Basic" collection, and credited there as "suggested from MS-BASIC manual." It's a menu-driven parts inventory: check a part's status, edit or add a part, add or subtract stock, and print a reorder report, all backed by a fixed 100-record random-access file. Rebuilt in BASCAL, it exercises [record files](15_random_and_record_files.md), [procedures](14_procedures.md) (including `byref` parameters), [short-circuit `&&`/`||`](16_short_circuit.md), and [error handling](17_labels_and_error_handling.md) together, at the scale where classic BASIC's line-number bookkeeping starts to really hurt — and where BASCAL's structuring pays off.

This is a reconstruction, not a line-by-line port. A few of fhb's original pieces have no BASCAL equivalent and were deliberately dropped rather than approximated: the GOTO-driven "subroutine roadmap" dispatcher meant for navigating the listing in the GW-BASIC interpreter itself; `KEY OFF`/`VIEW PRINT` console features BASCAL doesn't expose; and a numeric-ERR-code-to-message lookup table, collapsed here into a single line reporting the raw `ERR`/`ERL` values. See the header comment in the source below for the full list, including why `inven.dat` must be pre-populated with 100 blank records before the program will run correctly — fhb's original had a one-time hidden initializer for this that has no BASCAL-source equivalent in `inventory.bcl` itself.

Verified against real BASCOM 2.00 under dosbox-x: it compiles clean and links, but only when BASCOM is invoked with the `/E` and `/X` switches, since `on error goto`/`resume` isn't linked in by default. [`scripts/run-in-dosbox.sh`](https://github.com/johnjoeallen/bascal/blob/main/scripts/run-in-dosbox.sh) in the repo automates all of this — compiling with `bcc`, seeding a blank `inven.dat` with 100 records itself (reproducing fhb's one-time initializer at the tooling level, outside the language), adding the right BASCOM switches, and launching an interactive dosbox-x session — so you can actually run this one instead of just reading it.

</div>

<div class="snippet" markdown="1">

### The record/file DSL replaces fhb's manual FIELD layout and MKx\$/CVx\$ packing

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



[← Standard Library Functions](18_stdlib.md)


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
    flag:    string(1) lpad
    desc:    string(30) lpad
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
    global inv
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
    global inv
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
    global inv
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



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/inventory.bas</code></summary>



```basic

10 ' BASCAL generated BASIC -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Maps an ERR code to its classic MBASIC/GW-BASIC/BASCOM message. Compiles
40 ' and links on a real IBM BASIC Compiler 2.00 as ERROR$, but silently
50 ' returns an empty string at runtime (verified under dosbox-x) -- so BASCAL
60 ' ships a working implementation.
70 '
80 ' The named constants below are the complete common subset supported by
90 ' ERROR$: use them in THROW and filtered CATCH clauses instead of magic
100 ' numbers.  Dialect-specific errors outside this shared MBASIC/GW-BASIC/
110 ' BASCOM subset still fall through to ERROR$'s generic message.
120 '
130 ' Deliberately NOT a scalar method (see GitHub issue #41, which asked for
140 ' this decision to be recorded either way): code% is an opaque lookup key,
150 ' not a value the call is naturally "operating on" the way ltrim$/rtrim$/
160 ' ucase$/lcase$ operate on their string -- code%.error() would read as if
170 ' the *error code itself* has a message, when really this is a lookup
180 ' table keyed by that code. Stays an ordinary function.

190 errsyntax% = 2
200 errreturnwithoutgosub% = 3
210 erroutofdata% = 4
220 errillegalfunctioncall% = 5
230 erroverflow% = 6
240 erroutofmemory% = 7
250 errsubscriptoutofrange% = 9
260 errduplicatedefinition% = 10
270 errdivisionbyzero% = 11
280 errtypemismatch% = 13
290 erroutofstringspace% = 14
300 errnoresume% = 19
310 errresumewithouterror% = 20
320 errdevicetimeout% = 24
330 errdevicefault% = 25
340 erroutofpaper% = 27
350 errbadfilenumber% = 52
360 errfilenotfound% = 53
370 errbadfilemode% = 54
380 errfilealreadyopen% = 55
390 errdeviceio% = 57
400 errfilealreadyexists% = 58
410 errdiskfull% = 61
420 errinputpastend% = 62
430 errbadrecordnumber% = 63
440 errbadfilename% = 64
450 errtoomanyfiles% = 67
460 errdeviceunavailable% = 68
470 errdiskwriteprotected% = 70
480 errdisknotready% = 71
490 errdiskmediaerror% = 72
500 errpathfileaccess% = 75
510 errpathnotfound% = 76

520 ' ============================================================
530 ' INVENTORY.BCL -- Random-Access Inventory Program
540 '
550 ' A BASCAL reconstruction of "Example program for RANDOM ACCESS
560 ' FILE study", by fhb, 8/19/98, from Joseph Sixpack's GW-BASIC
570 ' programs page (part of his "Last Book of GW-Basic" collection):
580 ' http://www.geocities.ws/joseph_sixpack/binventory.html
590 ' fhb's own header comment credits the original as "suggested
600 ' from MS-BASIC manual".
610 '
620 ' This is a reconstruction, not a line-by-line port -- some
630 ' original pieces have no BASCAL equivalent and were dropped
640 ' rather than approximated:
650 ' - The GOTO-driven "subroutine roadmap" dispatcher at the top
660 ' of fhb's listing (a `LIST 110-320` etc. navigation aid for
670 ' editing in the GW-BASIC interpreter) has no meaning once the
680 ' program is structured into named function/procedure blocks.
690 ' - `KEY OFF` / `KEY I,""` (clearing the function-key soft-label
700 ' row) and `VIEW PRINT` (scroll-region windowing for the list
710 ' screen) are interpreter/console features BASCAL doesn't
720 ' expose.
730 ' - fhb's own hand-rolled numeric-ERR-code-to-message lookup table
740 ' (ERR=1 "Input value overflow", ERR=2 "Syntax error", ... ERR=25)
750 ' is replaced below by BASCAL's com.bascal.stdlib.error library
760 ' (ERROR$(code%)) -- same idea, BASCAL's own table; it still
770 ' doesn't decode ERL, which errorTrap() reports as the raw line
780 ' number.
790 ' - fhb's one-time "hidden" datafile initializer (PUT-ing 100
800 ' blank, CHR$(255)-flagged records) is reproduced below as
810 ' initializeInventoryFileIfNew(), called once at program entry --
820 ' inven.dat no longer has to be pre-populated by hand.
830 ' - The three original tab-position constants (T=20, U=25,
840 ' V=30) are collapsed into a single `tabCol% = 20`; a couple of
850 ' screens that used U=25 in the original (see showAddStockScreen
860 ' below) keep 25 as a literal rather than reusing tabCol%.
870 '
880 ' Tracks parts in a fixed 100-record file: check status, add,
890 ' edit, add/subtract stock, and a reorder report.
900 '
910 ' Error handling uses try/catch (GitHub issue #60), not the raw `on
920 ' error goto` / `resume next` fhb's original relies on: a failed menu
930 ' action is abandoned outright and the program returns straight to the
940 ' main menu, rather than resuming at the exact instruction after
950 ' whatever failed -- see reportInventoryError() below and
960 ' tutorial/inventory_try_catch.draft's own header comment for why. This
970 ' is a real, deliberate behavior change from an earlier on-error-goto
980 ' version of this file, which *was* verified against real BASCOM 2.00
990 ' under dosbox-x (only with the /E and /X switches -- error trapping
1000 ' isn't linked in by default); the try/catch shape below transpiles to
1010 ' the same ON ERROR GOTO/RESUME primitives BASCOM accepts, but hasn't
1020 ' itself been independently re-verified against a real BASCOM compile.
1030 ' ============================================================

1040 ' BASCAL-ism: the record/file DSL. `record ... end record` plus
1050 ' `file ... as ... = open(...)` below replace fhb's manual
1060 ' FIELD #1,1 AS F$,30 AS D$,2 AS Q$,... buffer layout entirely --
1070 ' bcc computes the field widths and record LEN from this
1080 ' declaration and generates the FIELD statement itself. Named
1090 ' field access (`p.flag`, `p.qty`, ...) and whole-record
1100 ' read/write via `inv[n]` (see checkPart() below) replace fhb's
1110 ' manual GET/PUT plus LSET/RSET and MKI$/MKS$/CVI$/CVS$ packing.

1120 ' BASCAL-ism: `const` is a real compile-time constant, not a plain
1130 ' variable assignment like fhb's `N=100` / `T=20` -- it can never
1140 ' be reassigned, and resolves to the same value everywhere,
1150 ' including inside every function/procedure below, with no
1160 ' `global` declaration needed.
1170 partcount% = 100
1180 tabcol% = 20

1190 ' `file ... = open(...)` is sugar for OPEN ... FOR RANDOM AS #n
1200 ' LEN = <record width> plus the FIELD statement fhb wrote out by
1210 ' hand at his line 550. Wrapped in its own try/catch: a file that
1220 ' exists but can't be opened for random access (permissions, a
1230 ' read-only inven.dat, disk full on the fallback create) is a real,
1240 ' trappable error (code 75, "Path/File access error") on both
1250 ' targets now, not a hard crash -- report it and exit cleanly
1260 ' instead of leaving the program to fail confusingly the first time
1270 ' something tries to use an `inv` that was never actually opened.
1280 ON ERROR GOTO 1350
1290 BCC_TRY_0001_PENDING% = 0
1300     ' file inv as Part = open(...)  [39 bytes/record]
1310     OPEN "inven.dat" FOR RANDOM AS #1 LEN = 39
1320     FIELD #1, 1 AS invflagbuf$, 30 AS invdescbuf$, 2 AS invqtybuf$, 2 AS invreorderbuf$, 4 AS invpricebuf$
1330 ON ERROR GOTO 0
1340 GOTO 1490
1350     BCC_TRY_0001_PENDING% = ERR
1360     err% = ERR
1370     erl% = ERL
1380     RESUME 1390
1390 ON ERROR GOTO 1470
1400     errorCode0% = err%
1410     GOSUB 2800
1420     PRINT "could not open inven.dat: " + errorResult0$
1430     END
1440     BCC_TRY_0001_PENDING% = 0
1450     ON ERROR GOTO 0
1460     GOTO 1490
1470     BCC_TRY_0001_PENDING% = ERR
1480     RESUME 1490
1490 ON ERROR GOTO 0
1500     IF BCC_TRY_0001_PENDING% <> 0 THEN ERROR BCC_TRY_0001_PENDING%
1510 REM END TRY

1520 ' -------------------- Pure functions (no file access) --------------------

1530 ' BASCAL-ism: `function ... end function` with `return` replaces
1540 ' fhb's convention of a GOSUB target plus a bare RETURN -- there's
1550 ' no separate "subroutine label" and no shared/global result
1560 ' variable to manage by hand; `isEmpty%(...)` is called like an
1570 ' ordinary expression at every use below (e.g. `isEmpty%(p.flag)`).
1580 ' A record whose flag byte is CHR$(255) is an empty/never-used slot.

1590 ' BASCAL-ism: `&&` and `||` are short-circuit AND/OR -- real
1600 ' MBASIC/BASCOM only has bitwise AND/OR (which fhb relies on here
1610 ' too, since `PART!<1 OR PART!>N!` never short-circuits anyway).
1620 ' BASCAL lowers `&&`/`||` into the equivalent branching so the
1630 ' short-circuit *is* real at the generated-BASIC level; see the
1640 ' manual's "Short-Circuit && and ||" section
1650 ' (https://johnjoeallen.github.io/bascal/manual/).

1660 ' -------------------- Keyboard input --------------------

1670 ' BASCAL-ism: `do ... loop until` is a structured post-check loop
1680 ' replacing fhb's `730 KP$=INKEY$:IF KP$="" THEN 730` GOTO-polling
1690 ' idiom. `inkey$` itself is the real INKEY$ builtin passed straight
1700 ' through, resolving correctly from inside a function/procedure
1710 ' body like this one -- every menu action below calls
1720 ' readKey$()/waitAnyKey() rather than polling INKEY$ inline.

1730 ' -------------------- Display procedures --------------------

1740 ' byref scalar parameters: gatherPartDetails writes the four editable
1750 ' fields for a part directly back into the caller's variables.

1760 ' -------------------- Menu actions --------------------

1770 ' fhb's own one-time "hidden" datafile initializer PUT-ing 100 blank,
1780 ' CHR$(255)-flagged records (see the header note above) -- reproduced
1790 ' here so inven.dat no longer has to be pre-populated by hand before
1800 ' running this program. A brand-new file OPEN created just now (rather
1810 ' than one that already existed) reads back as all-zero bytes: record
1820 ' 1's flag byte is CHR$(0), never CHR$(255) -- the one signal an
1830 ' already-populated file (whose record 1 flag is always either
1840 ' CHR$(255), still an empty slot, or a real part's own "1") could never
1850 ' produce, so it's what isEmpty%() itself can't use (see its own
1860 ' header note) but this one-time check safely can.

1870 ' -------------------- Program entry --------------------

1880 CLS
1890 GOSUB 9550

1900     GOSUB 4530
1910     GOSUB 4380
1920     kp$ = readkeyResult0$
1930     IF (INSTR("1234567cCeElLaAsSrRxX", kp$) <> 0) = 0 THEN GOTO 2630
1940         ' BASCAL-ism: `select case` replaces fhb's chain of eight
1950         ' `IF VAL(KP$)=n OR KP$="x" OR KP$="X" THEN GOTO ...` lines
1960         ' (his 770-840) with one multi-way dispatch.
1970         '
1980         ' BASCAL-ism: `try`/`catch` (issue #60) replaces fhb's own global
1990         ' `ON ERROR GOTO` trap. A failed menu action is abandoned outright
2000         ' here -- the `catch` below runs, then execution continues right
2010         ' after `end try`, back at `loop until` -- rather than resuming at
2020         ' the exact instruction after whatever failed inside checkPart()/
2030         ' editRecord()/etc. the way fhb's `RESUME NEXT` did. See
2040         ' reportInventoryError() below and tutorial/inventory_try_catch.
2050         ' draft's own header comment for why that arbitrary resume-point
2060         ' behavior isn't something try/catch reproduces.
2070         ON ERROR GOTO 2470
2080         BCC_TRY_0004_PENDING% = 0
2090             BCCT6$ = kp$
2100             IF (BCCT6$ = "1" OR BCCT6$ = "c" OR BCCT6$ = "C") <> 0 THEN GOTO 2180
2110             IF (BCCT6$ = "2" OR BCCT6$ = "e" OR BCCT6$ = "E") <> 0 THEN GOTO 2200
2120             IF (BCCT6$ = "3" OR BCCT6$ = "l" OR BCCT6$ = "L") <> 0 THEN GOTO 2220
2130             IF (BCCT6$ = "4" OR BCCT6$ = "a" OR BCCT6$ = "A") <> 0 THEN GOTO 2240
2140             IF (BCCT6$ = "5" OR BCCT6$ = "s" OR BCCT6$ = "S") <> 0 THEN GOTO 2260
2150             IF (BCCT6$ = "6" OR BCCT6$ = "r" OR BCCT6$ = "R") <> 0 THEN GOTO 2280
2160             IF (BCCT6$ = "7" OR BCCT6$ = "x" OR BCCT6$ = "X") <> 0 THEN GOTO 2300
2170             GOTO 2440
2180                 GOSUB 6060
2190                 GOTO 2440
2200                 GOSUB 6610
2210                 GOTO 2440
2220                 GOSUB 7310
2230                 GOTO 2440
2240                 GOSUB 7680
2250                 GOTO 2440
2260                 GOSUB 8370
2270                 GOTO 2440
2280                 GOSUB 9150
2290                 GOTO 2440
2300                 ' BASCAL-ism: `inv.close()` is sugar for `CLOSE #1`,
2310                 ' matching fhb's own `90 CLOSE:SYSTEM`. fhb's original
2320                 ' also had a separate "Quit to BASIC" option (his own
2330                 ' 7, returning to the interpreter's command prompt
2340                 ' rather than exiting to DOS) -- dropped here: a
2350                 ' compiled program has no interpreter to return to,
2360                 ' so it was never anything but a second spelling of
2370                 ' this same close-and-exit action.
2380                 ' inv.close()
2390                 CLOSE #1
2400                 COLOR 7, 0
2410                 CLS
2420                 SYSTEM
2430                 GOTO 2440
2440             REM END SELECT
2450         ON ERROR GOTO 0
2460         GOTO 2600
2470             BCC_TRY_0004_PENDING% = ERR
2480             err% = ERR
2490             erl% = ERL
2500             RESUME 2510
2510         ON ERROR GOTO 2580
2520             reportinventoryerrorErr0% = err%
2530             reportinventoryerrorErl0% = erl%
2540             GOSUB 9890
2550             BCC_TRY_0004_PENDING% = 0
2560             ON ERROR GOTO 0
2570             GOTO 2600
2580             BCC_TRY_0004_PENDING% = ERR
2590             RESUME 2600
2600         ON ERROR GOTO 0
2610             IF BCC_TRY_0004_PENDING% <> 0 THEN ERROR BCC_TRY_0004_PENDING%
2620         REM END TRY
2630     REM END IF
2640     GOTO 1900
2650 REM END DO

2660 ' -------------------- Error handling --------------------
2670 ' err%/erl% are ordinary locals scoped to the `catch` block above, not
2680 ' aliases for the ambient (readable-anywhere) `err`/`erl` pseudo-
2690 ' variables `on error goto` uses -- see `Statement::TryCatch`'s own doc
2700 ' comment in ast.rs. Passed straight through to ERROR$ here like fhb's
2710 ' own ERR/ERL (his 3390: "an error on line";ERL), decoded through
2720 ' BASCAL's own com.bascal.stdlib.error (ERROR$) instead of fhb's
2730 ' hand-rolled lookup table -- see the header note above. try/catch
2740 ' itself isn't documented in the manual yet (GitHub issue #60 tracks
2750 ' the still-unfinished C-target work; the manual page can follow once
2760 ' that lands) -- see ast.rs's own `Statement::TryCatch` doc comment for
2770 ' the full semantics meanwhile.
2780 END

2790 ' function error$(code%)
2800     BCCT8% = errorCode0%
2810     IF (BCCT8% = errsyntax%) <> 0 THEN GOTO 3150
2820     IF (BCCT8% = errreturnwithoutgosub%) <> 0 THEN GOTO 3180
2830     IF (BCCT8% = erroutofdata%) <> 0 THEN GOTO 3210
2840     IF (BCCT8% = errillegalfunctioncall%) <> 0 THEN GOTO 3240
2850     IF (BCCT8% = erroverflow%) <> 0 THEN GOTO 3270
2860     IF (BCCT8% = erroutofmemory%) <> 0 THEN GOTO 3300
2870     IF (BCCT8% = errsubscriptoutofrange%) <> 0 THEN GOTO 3330
2880     IF (BCCT8% = errduplicatedefinition%) <> 0 THEN GOTO 3360
2890     IF (BCCT8% = errdivisionbyzero%) <> 0 THEN GOTO 3390
2900     IF (BCCT8% = errtypemismatch%) <> 0 THEN GOTO 3420
2910     IF (BCCT8% = erroutofstringspace%) <> 0 THEN GOTO 3450
2920     IF (BCCT8% = errnoresume%) <> 0 THEN GOTO 3480
2930     IF (BCCT8% = errresumewithouterror%) <> 0 THEN GOTO 3510
2940     IF (BCCT8% = errdevicetimeout%) <> 0 THEN GOTO 3540
2950     IF (BCCT8% = errdevicefault%) <> 0 THEN GOTO 3570
2960     IF (BCCT8% = erroutofpaper%) <> 0 THEN GOTO 3600
2970     IF (BCCT8% = errbadfilenumber%) <> 0 THEN GOTO 3630
2980     IF (BCCT8% = errfilenotfound%) <> 0 THEN GOTO 3660
2990     IF (BCCT8% = errbadfilemode%) <> 0 THEN GOTO 3690
3000     IF (BCCT8% = errfilealreadyopen%) <> 0 THEN GOTO 3720
3010     IF (BCCT8% = errdeviceio%) <> 0 THEN GOTO 3750
3020     IF (BCCT8% = errfilealreadyexists%) <> 0 THEN GOTO 3780
3030     IF (BCCT8% = errdiskfull%) <> 0 THEN GOTO 3810
3040     IF (BCCT8% = errinputpastend%) <> 0 THEN GOTO 3840
3050     IF (BCCT8% = errbadrecordnumber%) <> 0 THEN GOTO 3870
3060     IF (BCCT8% = errbadfilename%) <> 0 THEN GOTO 3900
3070     IF (BCCT8% = errtoomanyfiles%) <> 0 THEN GOTO 3930
3080     IF (BCCT8% = errdeviceunavailable%) <> 0 THEN GOTO 3960
3090     IF (BCCT8% = errdiskwriteprotected%) <> 0 THEN GOTO 3990
3100     IF (BCCT8% = errdisknotready%) <> 0 THEN GOTO 4020
3110     IF (BCCT8% = errdiskmediaerror%) <> 0 THEN GOTO 4050
3120     IF (BCCT8% = errpathfileaccess%) <> 0 THEN GOTO 4080
3130     IF (BCCT8% = errpathnotfound%) <> 0 THEN GOTO 4110
3140     GOTO 4140
3150         errorResult0$ = "Syntax error"
3160         RETURN
3170         GOTO 4160
3180         errorResult0$ = "RETURN without GOSUB"
3190         RETURN
3200         GOTO 4160
3210         errorResult0$ = "Out of DATA"
3220         RETURN
3230         GOTO 4160
3240         errorResult0$ = "Illegal function call"
3250         RETURN
3260         GOTO 4160
3270         errorResult0$ = "Overflow"
3280         RETURN
3290         GOTO 4160
3300         errorResult0$ = "Out of memory"
3310         RETURN
3320         GOTO 4160
3330         errorResult0$ = "Subscript out of range"
3340         RETURN
3350         GOTO 4160
3360         errorResult0$ = "Duplicate Definition"
3370         RETURN
3380         GOTO 4160
3390         errorResult0$ = "Division by zero"
3400         RETURN
3410         GOTO 4160
3420         errorResult0$ = "Type mismatch"
3430         RETURN
3440         GOTO 4160
3450         errorResult0$ = "Out of string space"
3460         RETURN
3470         GOTO 4160
3480         errorResult0$ = "No RESUME"
3490         RETURN
3500         GOTO 4160
3510         errorResult0$ = "RESUME without error"
3520         RETURN
3530         GOTO 4160
3540         errorResult0$ = "Device timeout"
3550         RETURN
3560         GOTO 4160
3570         errorResult0$ = "Device fault"
3580         RETURN
3590         GOTO 4160
3600         errorResult0$ = "Out of paper"
3610         RETURN
3620         GOTO 4160
3630         errorResult0$ = "Bad file number"
3640         RETURN
3650         GOTO 4160
3660         errorResult0$ = "File not found"
3670         RETURN
3680         GOTO 4160
3690         errorResult0$ = "Bad file mode"
3700         RETURN
3710         GOTO 4160
3720         errorResult0$ = "File already open"
3730         RETURN
3740         GOTO 4160
3750         errorResult0$ = "Device I/O error"
3760         RETURN
3770         GOTO 4160
3780         errorResult0$ = "File already exists"
3790         RETURN
3800         GOTO 4160
3810         errorResult0$ = "Disk full"
3820         RETURN
3830         GOTO 4160
3840         errorResult0$ = "Input past end"
3850         RETURN
3860         GOTO 4160
3870         errorResult0$ = "Bad record number"
3880         RETURN
3890         GOTO 4160
3900         errorResult0$ = "Bad file name"
3910         RETURN
3920         GOTO 4160
3930         errorResult0$ = "Too many files"
3940         RETURN
3950         GOTO 4160
3960         errorResult0$ = "Device unavailable"
3970         RETURN
3980         GOTO 4160
3990         errorResult0$ = "Disk write protected"
4000         RETURN
4010         GOTO 4160
4020         errorResult0$ = "Disk not ready"
4030         RETURN
4040         GOTO 4160
4050         errorResult0$ = "Disk media error"
4060         RETURN
4070         GOTO 4160
4080         errorResult0$ = "Path/File access error"
4090         RETURN
4100         GOTO 4160
4110         errorResult0$ = "Path not found"
4120         RETURN
4130         GOTO 4160
4140         errorResult0$ = "Error " + STR$(errorCode0%)
4150         RETURN
4160     REM END SELECT
4170     RETURN
4180 ' end function error$

4190 ' function isempty%(flag$)
4200     isemptyResult0% = ASC(isemptyFlag0$) = 255
4210     RETURN
4220 ' end function isempty%

4230 ' function partinrange%(n%)
4240     IF (partinrangeN0% >= 1) = 0 THEN GOTO 4280
4250     IF (partinrangeN0% <= partcount%) = 0 THEN GOTO 4280
4260         partinrangeResult0% = 1
4270         RETURN
4280     REM END IF
4290     partinrangeResult0% = 0
4300     RETURN
4310 ' end function partinrange%

4320 ' function readpartnumberinput$()
4330     INPUT "Input part number"; readpartnumberinputS0$
4340     readpartnumberinputResult0$ = readpartnumberinputS0$
4350     RETURN
4360 ' end function readpartnumberinput$

4370 ' function readkey$()
4380         readkeyK0$ = INKEY$
4390         IF (readkeyK0$ <> "") = 0 THEN GOTO 4380
4400     REM END DO
4410     readkeyResult0$ = readkeyK0$
4420     RETURN
4430 ' end function readkey$

4440 ' procedure waitanykey()
4450     LOCATE 25, 10
4460     PRINT "Press the AnyKey to continue...";
4470         waitanykeyK0$ = INKEY$
4480         IF (waitanykeyK0$ <> "") = 0 THEN GOTO 4470
4490     REM END DO
4500     RETURN
4510 ' end procedure waitanykey

4520 ' procedure showmainmenu()
4530     CLS
4540     COLOR 14, 4
4550     CLS
4560     LOCATE 6, 1
4570     PRINT
4580     ' `tab(n)` passes straight through to real TAB(n), same as
4590     ' fhb's own `PRINT TAB(V) "..."` -- but only as a bare item in
4600     ' a PRINT list, juxtaposed or `;`-separated like here. Real
4610     ' BASCOM rejects `"literal" + tab(n) + ...` (TAB isn't a real
4620     ' string function you can concatenate); see printListHeader()
4630     ' and printReorderHeader() below, which need `;` between a
4640     ' preceding string and a `tab(n)` for exactly this reason.
4650     PRINT TAB(30)"Inventory Program"
4660     PRINT
4670     PRINT TAB(tabcol%)"1......C)heck a part"
4680     PRINT TAB(tabcol%)"2......E)dit/overwrite/add a part"
4690     PRINT TAB(tabcol%)("3......L)ist all" + STR$(partcount%)) + "parts"
4700     PRINT TAB(tabcol%)"4......A)dd stock"
4710     PRINT TAB(tabcol%)"5......S)ubtract stock"
4720     PRINT TAB(tabcol%)"6......R)eorder Report"
4730     PRINT
4740     PRINT TAB(tabcol%)"7......eX)it to system"
4750     RETURN
4760 ' end procedure showmainmenu

4770 ' procedure showbadpartnumber()
4780     CLS
4790     LOCATE 10, 10
4800     PRINT "Part number is out of permissable range of 1 to" + STR$(partcount%)
4810     RETURN
4820 ' end procedure showbadpartnumber

4830 ' procedure showrangeretrymessage()
4840     LOCATE 10, 15
4850     PRINT "The Part number is out of permissable range of 1 to" + STR$(partcount%)
4860     LOCATE 25, 15
4870     PRINT "Press the Anykey to reenter part number...";
4880     RETURN
4890 ' end procedure showrangeretrymessage

4900 ' procedure shownullentrymessage(partstr$)
4910     LOCATE 10, tabcol%
4920     PRINT ("Part number " + shownullentrymessagePartStr0$) + " is a null entry"
4930     RETURN
4940 ' end procedure shownullentrymessage

4950 ' procedure showpartstatus(partnum%, desc$, qty%, reorder%, price!)
4960     CLS
4970     LOCATE 5, 1
4980     PRINT TAB(tabcol%)"Inventory Status for Individual Part Number"
4990     PRINT TAB(tabcol%)"==========================================="
5000     PRINT
5010     PRINT
5020     PRINT TAB(tabcol%)"     Part number:  " + STR$(showpartstatusPartNum0%)
5030     PRINT
5040     PRINT TAB(tabcol%)"       Item name:  " + showpartstatusDesc0$
5050     PRINT TAB(tabcol%)"Quantity on hand:  " + STR$(showpartstatusQty0%)
5060     PRINT TAB(tabcol%)"   Reorder level:  " + STR$(showpartstatusReorder0%)
5070     PRINT TAB(tabcol%)"      Unit price:  " + STR$(showpartstatusPrice0!)
5080     RETURN
5090 ' end procedure showpartstatus

5100 ' procedure printlistheader()
5110     CLS
5120     PRINT TAB(25)"I N V E N T O R Y   L I S T I N G"; TAB(65); STR$(partcount%) + "items"
5130     PRINT "                                          Quantity       Reorder"
5140     PRINT " Partno           Description             on hand         level"
5150     LOCATE 25, 1
5160     PRINT "Press the AnyKey to scroll listing...";
5170     RETURN
5180 ' end procedure printlistheader

5190 ' procedure printinventoryline(partnum%, desc$, qty%, reorder%)
5200     PRINT (((((STR$(printinventorylinePartNum0%) + "  ") + printinventorylineDesc0$) + "   ") + STR$(printinventorylineQty0%)) + "          ") + STR$(printinventorylineReorder0%)
5210     RETURN
5220 ' end procedure printinventoryline

5230 ' procedure printreorderheader()
5240     CLS
5250     LOCATE 1, tabcol%
5260     PRINT "Reorder Report"; TAB(55); DATE$
5270     PRINT
5280     PRINT "                                             Quantity       Reorder"
5290     PRINT "    Partno           Description             on hand         level"
5300     PRINT "   =======  ==============================   ========       ======="
5310     RETURN
5320 ' end procedure printreorderheader

5330 ' procedure printreorderline(partnum%, desc$, qty%, reorder%)
5340     PRINT (((((("  " + STR$(printreorderlinePartNum0%)) + "  ") + printreorderlineDesc0$) + "   ") + STR$(printreorderlineQty0%)) + "          ") + STR$(printreorderlineReorder0%)
5350     RETURN
5360 ' end procedure printreorderline

5370 ' procedure gatherpartdetails(partnum%, desc$, qty%, reorder%, price!)
5380     CLS
5390     LOCATE 4, tabcol%
5400     PRINT "Adding or Overwriting a Record"
5410     LOCATE 8, tabcol%
5420     PRINT "Record/Partno" + STR$(gatherpartdetailsPartNum0%)
5430     LOCATE 11, 39
5440     PRINT "------------------------------"
5450     LOCATE 10, tabcol%
5460     INPUT "      Description"; gatherpartdetailsDesc0$
5470     LOCATE 12, tabcol%
5480     INPUT "Quantity in stock"; gatherpartdetailsQty0%
5490     LOCATE 14, tabcol%
5500     INPUT "    Reorder level"; gatherpartdetailsReorder0%
5510     LOCATE 16, tabcol%
5520     INPUT "       Unit price"; gatherpartdetailsPrice0!
5530     LOCATE 18, tabcol%
5540     PRINT "Is information correct (Y/N)?"
5550     RETURN
5560 ' end procedure gatherpartdetails

5570 ' procedure showaddstockscreen(partnum%, desc$, qty%, reorder%)
5580     CLS
5590     LOCATE 4, 25
5600     PRINT "Add to an inventory part number"
5610     LOCATE 5, 25
5620     PRINT "==============================="
5630     LOCATE 8, tabcol%
5640     PRINT "     Part number: " + STR$(showaddstockscreenPartNum0%)
5650     LOCATE 9, tabcol%
5660     PRINT "Item description: " + showaddstockscreenDesc0$
5670     LOCATE 10, tabcol%
5680     PRINT "Quantity on hand: " + STR$(showaddstockscreenQty0%)
5690     LOCATE 11, tabcol%
5700     PRINT "   Reorder Level: " + STR$(showaddstockscreenReorder0%)
5710     RETURN
5720 ' end procedure showaddstockscreen

5730 ' procedure shownegativeqtywarning()
5740     LOCATE 17, 15
5750     PRINT "The quantity to add must NOT be a negative number"
5760     LOCATE 25, 1
5770     PRINT "Please press the Anykey to reenter quantity to add...";
5780     RETURN
5790 ' end procedure shownegativeqtywarning

5800 ' procedure showsubtractstockscreen(partnum%, desc$, qty%, reorder%)
5810     CLS
5820     LOCATE 4, tabcol%
5830     PRINT "Subtract an inventory part number"
5840     LOCATE 5, tabcol%
5850     PRINT "================================="
5860     LOCATE 8, tabcol%
5870     PRINT "         Part number: " + STR$(showsubtractstockscreenPartNum0%)
5880     LOCATE 9, tabcol%
5890     PRINT "    Item description: " + showsubtractstockscreenDesc0$
5900     LOCATE 10, tabcol%
5910     PRINT "    Quantity on hand: " + STR$(showsubtractstockscreenQty0%)
5920     LOCATE 11, tabcol%
5930     PRINT "       Reorder Level: " + STR$(showsubtractstockscreenReorder0%)
5940     RETURN
5950 ' end procedure showsubtractstockscreen

5960 ' procedure showoversubtractwarning(onhand%)
5970     LOCATE 17, 5
5980     PRINT "The quantity to SUBTRACT must NOT result in NEGATIVE inventory"
5990     LOCATE 18, 5
6000     PRINT ("Only" + STR$(showoversubtractwarningOnHand0%)) + " IN STOCK"
6010     LOCATE 25, 1
6020     PRINT "Please press the Anykey to reenter quantity to subtract...";
6030     RETURN
6040 ' end procedure showoversubtractwarning

6050 ' procedure checkpart()
6060     ' global inv
6070     GOSUB 4330
6080     checkpartPartStr0$ = readpartnumberinputResult0$
6090     checkpartPart0% = VAL(checkpartPartStr0$)
6100     partinrangeN0% = checkpartPart0%
6110     GOSUB 4240
6120     IF (partinrangeResult0% = 0) = 0 THEN GOTO 6160
6130         GOSUB 4780
6140         GOSUB 4450
6150         RETURN
6160     REM END IF
6170     ' BASCAL-ism: `let p = inv[part%]` reads record `part%` of the
6180     ' `inv` file into a local record variable `p` -- one expression
6190     ' for what fhb's `GET #1, PART!` plus five separate field reads
6200     ' (F$, D$, CVI(Q$), CVI(R$), CVS(P$)) did by hand. The write
6210     ' side, `inv[part%] = { ... }` (see editRecord() below), is the
6220     ' same sugar for PUT plus the LSET/MKx$ packing it replaces.
6230     ' let p = inv[...]  (whole-record read)
6240     GET #1, checkpartPart0%
6250     checkpartPFlagTrimI0% = LEN(checkpartInvFlagBuf0$)
6260     IF (checkpartPFlagTrimI0% > 0) = 0 THEN GOTO 6300
6270     IF (MID$(checkpartInvFlagBuf0$, checkpartPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 6300
6280         checkpartPFlagTrimI0% = checkpartPFlagTrimI0% - 1
6290         GOTO 6260
6300     REM END WHILE
6310     checkpartPFlag0$ = LEFT$(checkpartInvFlagBuf0$, checkpartPFlagTrimI0%)
6320     checkpartPDescTrimI0% = LEN(checkpartInvDescBuf0$)
6330     IF (checkpartPDescTrimI0% > 0) = 0 THEN GOTO 6370
6340     IF (MID$(checkpartInvDescBuf0$, checkpartPDescTrimI0%, 1) = " ") = 0 THEN GOTO 6370
6350         checkpartPDescTrimI0% = checkpartPDescTrimI0% - 1
6360         GOTO 6330
6370     REM END WHILE
6380     checkpartPDesc0$ = LEFT$(checkpartInvDescBuf0$, checkpartPDescTrimI0%)
6390     checkpartPQty0% = CVI(checkpartInvQtyBuf0$)
6400     checkpartPReorder0% = CVI(checkpartInvReorderBuf0$)
6410     checkpartPPrice0! = CVS(checkpartInvPriceBuf0$)
6420     isemptyFlag0$ = checkpartPFlag0$
6430     GOSUB 4200
6440     IF (isemptyResult0%) = 0 THEN GOTO 6500
6450         CLS
6460         LOCATE 10, 18
6470         PRINT ("Part number" + STR$(checkpartPart0%)) + "is still a null entry at this time"
6480         GOSUB 4450
6490         RETURN
6500     REM END IF
6510     showpartstatusPartNum0% = checkpartPart0%
6520     showpartstatusDesc0$ = checkpartPDesc0$
6530     showpartstatusQty0% = checkpartPQty0%
6540     showpartstatusReorder0% = checkpartPReorder0%
6550     showpartstatusPrice0! = checkpartPPrice0!
6560     GOSUB 4960
6570     GOSUB 4450
6580     RETURN
6590 ' end procedure checkpart

6600 ' procedure editrecord()
6610     ' global inv
6620     CLS
6630     LOCATE 10, tabcol%
6640     GOSUB 4330
6650     editrecordPartStr0$ = readpartnumberinputResult0$
6660     editrecordPart0% = VAL(editrecordPartStr0$)
6670     partinrangeN0% = editrecordPart0%
6680     GOSUB 4240
6690     IF (partinrangeResult0% = 0) = 0 THEN GOTO 6730
6700         GOSUB 4780
6710         GOSUB 4450
6720         RETURN
6730     REM END IF
6740     ' let p = inv[...]  (whole-record read)
6750     GET #1, editrecordPart0%
6760     editrecordPFlagTrimI0% = LEN(editrecordInvFlagBuf0$)
6770     IF (editrecordPFlagTrimI0% > 0) = 0 THEN GOTO 6810
6780     IF (MID$(editrecordInvFlagBuf0$, editrecordPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 6810
6790         editrecordPFlagTrimI0% = editrecordPFlagTrimI0% - 1
6800         GOTO 6770
6810     REM END WHILE
6820     editrecordPFlag0$ = LEFT$(editrecordInvFlagBuf0$, editrecordPFlagTrimI0%)
6830     editrecordPDescTrimI0% = LEN(editrecordInvDescBuf0$)
6840     IF (editrecordPDescTrimI0% > 0) = 0 THEN GOTO 6880
6850     IF (MID$(editrecordInvDescBuf0$, editrecordPDescTrimI0%, 1) = " ") = 0 THEN GOTO 6880
6860         editrecordPDescTrimI0% = editrecordPDescTrimI0% - 1
6870         GOTO 6840
6880     REM END WHILE
6890     editrecordPDesc0$ = LEFT$(editrecordInvDescBuf0$, editrecordPDescTrimI0%)
6900     editrecordPQty0% = CVI(editrecordInvQtyBuf0$)
6910     editrecordPReorder0% = CVI(editrecordInvReorderBuf0$)
6920     editrecordPPrice0! = CVS(editrecordInvPriceBuf0$)
6930     isemptyFlag0$ = editrecordPFlag0$
6940     GOSUB 4200
6950     IF (isemptyResult0% = 0) = 0 THEN GOTO 7040
6960         LOCATE 12, tabcol%
6970         PRINT "Overwrite existing part data?"
6980         GOSUB 4380
6990         editrecordKp0$ = readkeyResult0$
7000         IF (editrecordKp0$ <> "Y") = 0 THEN GOTO 7030
7010         IF (editrecordKp0$ <> "y") = 0 THEN GOTO 7030
7020             RETURN
7030         REM END IF
7040     REM END IF

7050         gatherpartdetailsPartNum0% = editrecordPart0%
7060         gatherpartdetailsDesc0$ = editrecordEditDesc0$
7070         gatherpartdetailsQty0% = editrecordEditQty0%
7080         gatherpartdetailsReorder0% = editrecordEditReorder0%
7090         gatherpartdetailsPrice0! = editrecordEditPrice0!
7100         GOSUB 5380
7110         editrecordEditDesc0$ = gatherpartdetailsDesc0$
7120         editrecordEditQty0% = gatherpartdetailsQty0%
7130         editrecordEditReorder0% = gatherpartdetailsReorder0%
7140         editrecordEditPrice0! = gatherpartdetailsPrice0!
7150         GOSUB 4380
7160         editrecordKp0$ = readkeyResult0$
7170         IF (editrecordKp0$ = "Y") <> 0 THEN GOTO 7200
7180         IF (editrecordKp0$ = "y") <> 0 THEN GOTO 7200
7190         GOTO 7050
7200     REM END DO
7210     ' inv[...] = { ... }  (whole-record write)
7220     LSET editrecordInvFlagBuf0$ = "1"
7230     LSET editrecordInvDescBuf0$ = editrecordEditDesc0$
7240     LSET editrecordInvQtyBuf0$ = MKI$(editrecordEditQty0%)
7250     LSET editrecordInvReorderBuf0$ = MKI$(editrecordEditReorder0%)
7260     LSET editrecordInvPriceBuf0$ = MKS$(editrecordEditPrice0!)
7270     PUT #1, editrecordPart0%
7280     RETURN
7290 ' end procedure editrecord

7300 ' procedure listall()
7310     ' global inv
7320     GOSUB 5110
7330     listallScrollCount0% = 0
7340     FOR listallI0% = 1 TO partcount%
7350         ' let p = inv[...]  (whole-record read)
7360         GET #1, listallI0%
7370         listallPFlagTrimI0% = LEN(listallInvFlagBuf0$)
7380         IF (listallPFlagTrimI0% > 0) = 0 THEN GOTO 7420
7390         IF (MID$(listallInvFlagBuf0$, listallPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 7420
7400             listallPFlagTrimI0% = listallPFlagTrimI0% - 1
7410             GOTO 7380
7420         REM END WHILE
7430         listallPFlag0$ = LEFT$(listallInvFlagBuf0$, listallPFlagTrimI0%)
7440         listallPDescTrimI0% = LEN(listallInvDescBuf0$)
7450         IF (listallPDescTrimI0% > 0) = 0 THEN GOTO 7490
7460         IF (MID$(listallInvDescBuf0$, listallPDescTrimI0%, 1) = " ") = 0 THEN GOTO 7490
7470             listallPDescTrimI0% = listallPDescTrimI0% - 1
7480             GOTO 7450
7490         REM END WHILE
7500         listallPDesc0$ = LEFT$(listallInvDescBuf0$, listallPDescTrimI0%)
7510         listallPQty0% = CVI(listallInvQtyBuf0$)
7520         listallPReorder0% = CVI(listallInvReorderBuf0$)
7530         listallPPrice0! = CVS(listallInvPriceBuf0$)
7540         printinventorylinePartNum0% = listallI0%
7550         printinventorylineDesc0$ = listallPDesc0$
7560         printinventorylineQty0% = listallPQty0%
7570         printinventorylineReorder0% = listallPReorder0%
7580         GOSUB 5200
7590         listallScrollCount0% = listallScrollCount0% + 1
7600         IF (listallScrollCount0% = 20) = 0 THEN GOTO 7630
7610             GOSUB 4450
7620             listallScrollCount0% = 0
7630         REM END IF
7640     NEXT listallI0%
7650     RETURN
7660 ' end procedure listall

7670 ' procedure addstock()
7680     ' global inv
7690     CLS
7700     LOCATE 5, 25
7710     PRINT "A D D I N G   S T O C K"

7720         LOCATE 8, 25
7730         GOSUB 4330
7740         addstockPartStr0$ = readpartnumberinputResult0$
7750         addstockPart0% = VAL(addstockPartStr0$)
7760         partinrangeN0% = addstockPart0%
7770         GOSUB 4240
7780         addstockValidPart0% = partinrangeResult0%
7790         IF (addstockValidPart0% = 0) = 0 THEN GOTO 7820
7800             GOSUB 4840
7810             GOSUB 4380
7820         REM END IF
7830         IF (addstockValidPart0% <> 0) = 0 THEN GOTO 7720
7840     REM END DO

7850     ' let p = inv[...]  (whole-record read)
7860     GET #1, addstockPart0%
7870     addstockPFlagTrimI0% = LEN(addstockInvFlagBuf0$)
7880     IF (addstockPFlagTrimI0% > 0) = 0 THEN GOTO 7920
7890     IF (MID$(addstockInvFlagBuf0$, addstockPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 7920
7900         addstockPFlagTrimI0% = addstockPFlagTrimI0% - 1
7910         GOTO 7880
7920     REM END WHILE
7930     addstockPFlag0$ = LEFT$(addstockInvFlagBuf0$, addstockPFlagTrimI0%)
7940     addstockPDescTrimI0% = LEN(addstockInvDescBuf0$)
7950     IF (addstockPDescTrimI0% > 0) = 0 THEN GOTO 7990
7960     IF (MID$(addstockInvDescBuf0$, addstockPDescTrimI0%, 1) = " ") = 0 THEN GOTO 7990
7970         addstockPDescTrimI0% = addstockPDescTrimI0% - 1
7980         GOTO 7950
7990     REM END WHILE
8000     addstockPDesc0$ = LEFT$(addstockInvDescBuf0$, addstockPDescTrimI0%)
8010     addstockPQty0% = CVI(addstockInvQtyBuf0$)
8020     addstockPReorder0% = CVI(addstockInvReorderBuf0$)
8030     addstockPPrice0! = CVS(addstockInvPriceBuf0$)
8040     isemptyFlag0$ = addstockPFlag0$
8050     GOSUB 4200
8060     IF (isemptyResult0%) = 0 THEN GOTO 8110
8070         shownullentrymessagePartStr0$ = addstockPartStr0$
8080         GOSUB 4910
8090         GOSUB 4380
8100         RETURN
8110     REM END IF

8120         showaddstockscreenPartNum0% = addstockPart0%
8130         showaddstockscreenDesc0$ = addstockPDesc0$
8140         showaddstockscreenQty0% = addstockPQty0%
8150         showaddstockscreenReorder0% = addstockPReorder0%
8160         GOSUB 5580
8170         LOCATE 14, tabcol%
8180         INPUT " Quantity to add"; addstockAddStr0$
8190         addstockAddAmt0% = VAL(addstockAddStr0$)
8200         IF (addstockAddAmt0% < 0) = 0 THEN GOTO 8230
8210             GOSUB 5740
8220             GOSUB 4380
8230         REM END IF
8240         IF (addstockAddAmt0% >= 0) = 0 THEN GOTO 8120
8250     REM END DO

8260     addstockPQty0% = addstockPQty0% + addstockAddAmt0%
8270     ' inv[...] = p  (write back a let-bound record)
8280     LSET addstockInvFlagBuf0$ = addstockPFlag0$
8290     LSET addstockInvDescBuf0$ = addstockPDesc0$
8300     LSET addstockInvQtyBuf0$ = MKI$(addstockPQty0%)
8310     LSET addstockInvReorderBuf0$ = MKI$(addstockPReorder0%)
8320     LSET addstockInvPriceBuf0$ = MKS$(addstockPPrice0!)
8330     PUT #1, addstockPart0%
8340     RETURN
8350 ' end procedure addstock

8360 ' procedure subtractstock()
8370     ' global inv
8380     CLS
8390     LOCATE 5, 20
8400     PRINT "S U B T R A C T I N G    S T O C K"

8410         LOCATE 8, 25
8420         GOSUB 4330
8430         subtractstockPartStr0$ = readpartnumberinputResult0$
8440         subtractstockPart0% = VAL(subtractstockPartStr0$)
8450         partinrangeN0% = subtractstockPart0%
8460         GOSUB 4240
8470         subtractstockValidPart0% = partinrangeResult0%
8480         IF (subtractstockValidPart0% = 0) = 0 THEN GOTO 8510
8490             GOSUB 4840
8500             GOSUB 4380
8510         REM END IF
8520         IF (subtractstockValidPart0% <> 0) = 0 THEN GOTO 8410
8530     REM END DO

8540     ' let p = inv[...]  (whole-record read)
8550     GET #1, subtractstockPart0%
8560     subtractstockPFlagTrimI0% = LEN(subtractstockInvFlagBuf0$)
8570     IF (subtractstockPFlagTrimI0% > 0) = 0 THEN GOTO 8610
8580     IF (MID$(subtractstockInvFlagBuf0$, subtractstockPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 8610
8590         subtractstockPFlagTrimI0% = subtractstockPFlagTrimI0% - 1
8600         GOTO 8570
8610     REM END WHILE
8620     subtractstockPFlag0$ = LEFT$(subtractstockInvFlagBuf0$, subtractstockPFlagTrimI0%)
8630     subtractstockPDescTrimI0% = LEN(subtractstockInvDescBuf0$)
8640     IF (subtractstockPDescTrimI0% > 0) = 0 THEN GOTO 8680
8650     IF (MID$(subtractstockInvDescBuf0$, subtractstockPDescTrimI0%, 1) = " ") = 0 THEN GOTO 8680
8660         subtractstockPDescTrimI0% = subtractstockPDescTrimI0% - 1
8670         GOTO 8640
8680     REM END WHILE
8690     subtractstockPDesc0$ = LEFT$(subtractstockInvDescBuf0$, subtractstockPDescTrimI0%)
8700     subtractstockPQty0% = CVI(subtractstockInvQtyBuf0$)
8710     subtractstockPReorder0% = CVI(subtractstockInvReorderBuf0$)
8720     subtractstockPPrice0! = CVS(subtractstockInvPriceBuf0$)
8730     isemptyFlag0$ = subtractstockPFlag0$
8740     GOSUB 4200
8750     IF (isemptyResult0%) = 0 THEN GOTO 8800
8760         shownullentrymessagePartStr0$ = subtractstockPartStr0$
8770         GOSUB 4910
8780         GOSUB 4380
8790         RETURN
8800     REM END IF

8810         showsubtractstockscreenPartNum0% = subtractstockPart0%
8820         showsubtractstockscreenDesc0$ = subtractstockPDesc0$
8830         showsubtractstockscreenQty0% = subtractstockPQty0%
8840         showsubtractstockscreenReorder0% = subtractstockPReorder0%
8850         GOSUB 5810
8860         LOCATE 14, tabcol%
8870         INPUT "Quantity to subtract"; subtractstockSubStr0$
8880         subtractstockSubAmt0% = VAL(subtractstockSubStr0$)
8890         subtractstockOverSubtract0% = 0
8900         IF (subtractstockSubAmt0% >= 0) = 0 THEN GOTO 8960
8910         IF ((subtractstockPQty0% - subtractstockSubAmt0%) < 0) = 0 THEN GOTO 8960
8920             subtractstockOverSubtract0% = 1
8930             showoversubtractwarningOnHand0% = subtractstockPQty0%
8940             GOSUB 5970
8950             GOSUB 4380
8960         REM END IF
8970         IF (subtractstockSubAmt0% >= 0) = 0 THEN GOTO 8810
8980         IF (subtractstockOverSubtract0% = 0) = 0 THEN GOTO 8810
8990     REM END DO

9000     subtractstockPQty0% = subtractstockPQty0% - subtractstockSubAmt0%
9010     IF (subtractstockPQty0% <= subtractstockPReorder0%) = 0 THEN GOTO 9030
9020         LOCATE 16, tabcol%
9030     REM END IF
9040     PRINT (("quantity now" + STR$(subtractstockPQty0%)) + " reorder level") + STR$(subtractstockPReorder0%)
9050     ' inv[...] = p  (write back a let-bound record)
9060     LSET subtractstockInvFlagBuf0$ = subtractstockPFlag0$
9070     LSET subtractstockInvDescBuf0$ = subtractstockPDesc0$
9080     LSET subtractstockInvQtyBuf0$ = MKI$(subtractstockPQty0%)
9090     LSET subtractstockInvReorderBuf0$ = MKI$(subtractstockPReorder0%)
9100     LSET subtractstockInvPriceBuf0$ = MKS$(subtractstockPPrice0!)
9110     PUT #1, subtractstockPart0%
9120     RETURN
9130 ' end procedure subtractstock

9140 ' procedure reorderreport()
9150     ' global inv
9160     GOSUB 5240
9170     reorderreportReportLineCount0% = 0
9180     FOR reorderreportI0% = 1 TO partcount%
9190         ' let p = inv[...]  (whole-record read)
9200         GET #1, reorderreportI0%
9210         reorderreportPFlagTrimI0% = LEN(reorderreportInvFlagBuf0$)
9220         IF (reorderreportPFlagTrimI0% > 0) = 0 THEN GOTO 9260
9230         IF (MID$(reorderreportInvFlagBuf0$, reorderreportPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 9260
9240             reorderreportPFlagTrimI0% = reorderreportPFlagTrimI0% - 1
9250             GOTO 9220
9260         REM END WHILE
9270         reorderreportPFlag0$ = LEFT$(reorderreportInvFlagBuf0$, reorderreportPFlagTrimI0%)
9280         reorderreportPDescTrimI0% = LEN(reorderreportInvDescBuf0$)
9290         IF (reorderreportPDescTrimI0% > 0) = 0 THEN GOTO 9330
9300         IF (MID$(reorderreportInvDescBuf0$, reorderreportPDescTrimI0%, 1) = " ") = 0 THEN GOTO 9330
9310             reorderreportPDescTrimI0% = reorderreportPDescTrimI0% - 1
9320             GOTO 9290
9330         REM END WHILE
9340         reorderreportPDesc0$ = LEFT$(reorderreportInvDescBuf0$, reorderreportPDescTrimI0%)
9350         reorderreportPQty0% = CVI(reorderreportInvQtyBuf0$)
9360         reorderreportPReorder0% = CVI(reorderreportInvReorderBuf0$)
9370         reorderreportPPrice0! = CVS(reorderreportInvPriceBuf0$)
9380         IF (reorderreportPQty0% < reorderreportPReorder0%) = 0 THEN GOTO 9490
9390             printreorderlinePartNum0% = reorderreportI0%
9400             printreorderlineDesc0$ = reorderreportPDesc0$
9410             printreorderlineQty0% = reorderreportPQty0%
9420             printreorderlineReorder0% = reorderreportPReorder0%
9430             GOSUB 5340
9440             reorderreportReportLineCount0% = reorderreportReportLineCount0% + 1
9450             IF (reorderreportReportLineCount0% > 15) = 0 THEN GOTO 9480
9460                 GOSUB 4450
9470                 reorderreportReportLineCount0% = 0
9480             REM END IF
9490         REM END IF
9500     NEXT reorderreportI0%
9510     GOSUB 4450
9520     RETURN
9530 ' end procedure reorderreport

9540 ' procedure initializeinventoryfileifnew()
9550     ' global inv
9560     ' let p = inv[...]  (whole-record read)
9570     GET #1, 1
9580     initializeinventoryfileifnewPFlagTrimI0% = LEN(initializeinventoryfileifnewInvFlagBuf0$)
9590     IF (initializeinventoryfileifnewPFlagTrimI0% > 0) = 0 THEN GOTO 9630
9600     IF (MID$(initializeinventoryfileifnewInvFlagBuf0$, initializeinventoryfileifnewPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 9630
9610         initializeinventoryfileifnewPFlagTrimI0% = initializeinventoryfileifnewPFlagTrimI0% - 1
9620         GOTO 9590
9630     REM END WHILE
9640     initializeinventoryfileifnewPFlag0$ = LEFT$(initializeinventoryfileifnewInvFlagBuf0$, initializeinventoryfileifnewPFlagTrimI0%)
9650     initializeinventoryfileifnewPDescTrimI0% = LEN(initializeinventoryfileifnewInvDescBuf0$)
9660     IF (initializeinventoryfileifnewPDescTrimI0% > 0) = 0 THEN GOTO 9700
9670     IF (MID$(initializeinventoryfileifnewInvDescBuf0$, initializeinventoryfileifnewPDescTrimI0%, 1) = " ") = 0 THEN GOTO 9700
9680         initializeinventoryfileifnewPDescTrimI0% = initializeinventoryfileifnewPDescTrimI0% - 1
9690         GOTO 9660
9700     REM END WHILE
9710     initializeinventoryfileifnewPDesc0$ = LEFT$(initializeinventoryfileifnewInvDescBuf0$, initializeinventoryfileifnewPDescTrimI0%)
9720     initializeinventoryfileifnewPQty0% = CVI(initializeinventoryfileifnewInvQtyBuf0$)
9730     initializeinventoryfileifnewPReorder0% = CVI(initializeinventoryfileifnewInvReorderBuf0$)
9740     initializeinventoryfileifnewPPrice0! = CVS(initializeinventoryfileifnewInvPriceBuf0$)
9750     IF (ASC(initializeinventoryfileifnewPFlag0$) = 0) = 0 THEN GOTO 9850
9760         FOR initializeinventoryfileifnewI0% = 1 TO partcount%
9770             ' inv[...] = { ... }  (whole-record write)
9780             LSET initializeinventoryfileifnewInvFlagBuf0$ = CHR$(255)
9790             LSET initializeinventoryfileifnewInvDescBuf0$ = ""
9800             LSET initializeinventoryfileifnewInvQtyBuf0$ = MKI$(0)
9810             LSET initializeinventoryfileifnewInvReorderBuf0$ = MKI$(0)
9820             LSET initializeinventoryfileifnewInvPriceBuf0$ = MKS$(0)
9830             PUT #1, initializeinventoryfileifnewI0%
9840         NEXT initializeinventoryfileifnewI0%
9850     REM END IF
9860     RETURN
9870 ' end procedure initializeinventoryfileifnew

9880 ' procedure reportinventoryerror(err%, erl%)
9890     LOCATE 25, 1
9900     errorCode0% = reportinventoryerrorErr0%
9910     GOSUB 2800
9920     PRINT (("There has been an error on line" + STR$(reportinventoryerrorErl0%)) + ": ") + errorResult0$
9930     GOSUB 4380
9940     reportinventoryerrorK0$ = readkeyResult0$
9950     RETURN
9960 ' end procedure reportinventoryerror

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
#if defined(_WIN32)
#include <conio.h>
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

static char bcc_input_buf[256];

static char* bcc_strbuf_take(void);
static const char* bcc_mid(const char* s, int start, int length);
static const char* bcc_chr(int code);
static const char* bcc_stri(int value);
static const char* bcc_strd(double value);
static int bcc_instr(const char* s, const char* needle);
static const char* bcc_inkey(void);
static void bcc_read_string_field(char* field, const unsigned char* source, size_t width);
static void bcc_mki(char* out, int value);
static void bcc_mkl(char* out, int value);
static void bcc_mks(char* out, double value);
static void bcc_mkd(char* out, double value);
static int bcc_cvi(const char* s);
static int bcc_cvl(const char* s);
static float bcc_cvs(const char* s);
static double bcc_cvd(const char* s);
static int bcc_read_record(FILE* file, void* buffer, size_t reclen, long record);
static void bcc_write_record(FILE* file, const void* buffer, size_t reclen, long record);
static void bcc_pad_string_field(unsigned char* dest, const char* value, size_t width);
static int bcc_put_record_part(FILE* file, long record, const char* field_0, const char* field_1, const int16_t* field_2, const int16_t* field_3, const float* field_4);
static int bcc_get_record_part(FILE* file, long record, char* field_0, char* field_1, char* field_2, char* field_3, char* field_4);
static void bcc_color(int fg, int bg);
static void bcc_read_line(void);

static int bv_i_erl = 0;
static int bv_i_err = 0;
static int bv_i_errbadfilemode = 0;
static int bv_i_errbadfilename = 0;
static int bv_i_errbadfilenumber = 0;
static int bv_i_errbadrecordnumber = 0;
static int bv_i_errdevicefault = 0;
static int bv_i_errdeviceio = 0;
static int bv_i_errdevicetimeout = 0;
static int bv_i_errdeviceunavailable = 0;
static int bv_i_errdiskfull = 0;
static int bv_i_errdiskmediaerror = 0;
static int bv_i_errdisknotready = 0;
static int bv_i_errdiskwriteprotected = 0;
static int bv_i_errdivisionbyzero = 0;
static int bv_i_errduplicatedefinition = 0;
static int bv_i_errfilealreadyexists = 0;
static int bv_i_errfilealreadyopen = 0;
static int bv_i_errfilenotfound = 0;
static int bv_i_errillegalfunctioncall = 0;
static int bv_i_errinputpastend = 0;
static int bv_i_errnoresume = 0;
static int bv_i_erroutofdata = 0;
static int bv_i_erroutofmemory = 0;
static int bv_i_erroutofpaper = 0;
static int bv_i_erroutofstringspace = 0;
static int bv_i_erroverflow = 0;
static int bv_i_errpathfileaccess = 0;
static int bv_i_errpathnotfound = 0;
static int bv_i_errresumewithouterror = 0;
static int bv_i_errreturnwithoutgosub = 0;
static int bv_i_errsubscriptoutofrange = 0;
static int bv_i_errsyntax = 0;
static int bv_i_errtoomanyfiles = 0;
static int bv_i_errtypemismatch = 0;
static int bv_i_partcount = 0;
static int bv_i_tabcol = 0;
static char bv_s_invdescbuf[256] = {0};
static char bv_s_invflagbuf[256] = {0};
static char bv_s_invpricebuf[256] = {0};
static char bv_s_invqtybuf[256] = {0};
static char bv_s_invreorderbuf[256] = {0};
static char bv_s_kp[256] = {0};

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

void bf_s_error(int bv_i_code, char* bcc_out) {
    {
        int bt_sel_0 = bv_i_code;
        int bt_sel_match_1 = 0;
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errsyntax)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Syntax error");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errreturnwithoutgosub)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "RETURN without GOSUB");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_erroutofdata)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Out of DATA");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errillegalfunctioncall)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Illegal function call");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_erroverflow)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Overflow");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_erroutofmemory)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Out of memory");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errsubscriptoutofrange)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Subscript out of range");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errduplicatedefinition)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Duplicate Definition");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errdivisionbyzero)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Division by zero");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errtypemismatch)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Type mismatch");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_erroutofstringspace)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Out of string space");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errnoresume)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "No RESUME");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errresumewithouterror)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "RESUME without error");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errdevicetimeout)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Device timeout");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errdevicefault)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Device fault");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_erroutofpaper)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Out of paper");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errbadfilenumber)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Bad file number");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errfilenotfound)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "File not found");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errbadfilemode)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Bad file mode");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errfilealreadyopen)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "File already open");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errdeviceio)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Device I/O error");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errfilealreadyexists)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "File already exists");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errdiskfull)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Disk full");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errinputpastend)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Input past end");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errbadrecordnumber)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Bad record number");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errbadfilename)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Bad file name");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errtoomanyfiles)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Too many files");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errdeviceunavailable)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Device unavailable");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errdiskwriteprotected)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Disk write protected");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errdisknotready)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Disk not ready");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errdiskmediaerror)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Disk media error");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errpathfileaccess)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Path/File access error");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errpathnotfound)) {
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
    if (((-(bv_i_n >= 1)) && (-(bv_i_n <= bv_i_partcount)))) {
        return 1;
    }
    return 0;
}

void bf_s_readpartnumberinput(char* bcc_out) {
    char bv_s_s[256] = {0};

    printf("Input part number? ");
    bcc_read_line();
    snprintf(bv_s_s, sizeof(bv_s_s), "%s", bcc_input_buf);
    snprintf(bcc_out, 256, "%s", bv_s_s);
    return;
}

void bf_s_readkey(char* bcc_out) {
    char bv_s_k[256] = {0};

    while (1) {
        snprintf(bv_s_k, sizeof(bv_s_k), "%s", bcc_inkey());
        if ((-(strcmp(bv_s_k, "") != 0))) break;
    }
    snprintf(bcc_out, 256, "%s", bv_s_k);
    return;
}

void bf_i_waitanykey(void) {
    char bv_s_k[256] = {0};

    printf("\x1b[%d;%dH", 25, 10);
    printf("Press the AnyKey to continue...");
    while (1) {
        snprintf(bv_s_k, sizeof(bv_s_k), "%s", bcc_inkey());
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
    printf("\x1b[%dG1......C)heck a part\n", bv_i_tabcol);
    printf("\x1b[%dG2......E)dit/overwrite/add a part\n", bv_i_tabcol);
    char bt_s_3[256];
    snprintf(bt_s_3, sizeof(bt_s_3), "%s%s", "3......L)ist all", bcc_stri(bv_i_partcount));
    char bt_s_4[256];
    snprintf(bt_s_4, sizeof(bt_s_4), "%s%s", bt_s_3, "parts");
    printf("\x1b[%dG%s\n", bv_i_tabcol, bt_s_4);
    printf("\x1b[%dG4......A)dd stock\n", bv_i_tabcol);
    printf("\x1b[%dG5......S)ubtract stock\n", bv_i_tabcol);
    printf("\x1b[%dG6......R)eorder Report\n", bv_i_tabcol);
    printf("\n");
    printf("\x1b[%dG7......eX)it to system\n", bv_i_tabcol);
}

void bf_i_showbadpartnumber(void) {
    printf("\x1b[2J\x1b[H");
    printf("\x1b[%d;%dH", 10, 10);
    char bt_s_5[256];
    snprintf(bt_s_5, sizeof(bt_s_5), "%s%s", "Part number is out of permissable range of 1 to", bcc_stri(bv_i_partcount));
    printf("%s\n", bt_s_5);
}

void bf_i_showrangeretrymessage(void) {
    printf("\x1b[%d;%dH", 10, 15);
    char bt_s_6[256];
    snprintf(bt_s_6, sizeof(bt_s_6), "%s%s", "The Part number is out of permissable range of 1 to", bcc_stri(bv_i_partcount));
    printf("%s\n", bt_s_6);
    printf("\x1b[%d;%dH", 25, 15);
    printf("Press the Anykey to reenter part number...");
}

void bf_i_shownullentrymessage(const char* bv_s_partstr_in) {
    char bv_s_partstr[256];
    snprintf(bv_s_partstr, sizeof(bv_s_partstr), "%s", bv_s_partstr_in);

    printf("\x1b[%d;%dH", 10, bv_i_tabcol);
    char bt_s_7[256];
    snprintf(bt_s_7, sizeof(bt_s_7), "%s%s", "Part number ", bv_s_partstr);
    char bt_s_8[256];
    snprintf(bt_s_8, sizeof(bt_s_8), "%s%s", bt_s_7, " is a null entry");
    printf("%s\n", bt_s_8);
}

void bf_i_showpartstatus(int bv_i_partnum, const char* bv_s_desc_in, int bv_i_qty, int bv_i_reorder, float bv_f_price) {
    char bv_s_desc[256];
    snprintf(bv_s_desc, sizeof(bv_s_desc), "%s", bv_s_desc_in);

    printf("\x1b[2J\x1b[H");
    printf("\x1b[%d;%dH", 5, 1);
    printf("\x1b[%dGInventory Status for Individual Part Number\n", bv_i_tabcol);
    printf("\x1b[%dG===========================================\n", bv_i_tabcol);
    printf("\n");
    printf("\n");
    char bt_s_9[256];
    snprintf(bt_s_9, sizeof(bt_s_9), "%s%s", "     Part number:  ", bcc_stri(bv_i_partnum));
    printf("\x1b[%dG%s\n", bv_i_tabcol, bt_s_9);
    printf("\n");
    char bt_s_10[256];
    snprintf(bt_s_10, sizeof(bt_s_10), "%s%s", "       Item name:  ", bv_s_desc);
    printf("\x1b[%dG%s\n", bv_i_tabcol, bt_s_10);
    char bt_s_11[256];
    snprintf(bt_s_11, sizeof(bt_s_11), "%s%s", "Quantity on hand:  ", bcc_stri(bv_i_qty));
    printf("\x1b[%dG%s\n", bv_i_tabcol, bt_s_11);
    char bt_s_12[256];
    snprintf(bt_s_12, sizeof(bt_s_12), "%s%s", "   Reorder level:  ", bcc_stri(bv_i_reorder));
    printf("\x1b[%dG%s\n", bv_i_tabcol, bt_s_12);
    char bt_s_13[256];
    snprintf(bt_s_13, sizeof(bt_s_13), "%s%s", "      Unit price:  ", bcc_strd(bv_f_price));
    printf("\x1b[%dG%s\n", bv_i_tabcol, bt_s_13);
}

void bf_i_printlistheader(void) {
    printf("\x1b[2J\x1b[H");
    char bt_s_14[256];
    snprintf(bt_s_14, sizeof(bt_s_14), "%s%s", bcc_stri(bv_i_partcount), "items");
    printf("\x1b[%dGI N V E N T O R Y   L I S T I N G\x1b[%dG%s\n", 25, 65, bt_s_14);
    printf("                                          Quantity       Reorder\n");
    printf(" Partno           Description             on hand         level\n");
    printf("\x1b[%d;%dH", 25, 1);
    printf("Press the AnyKey to scroll listing...");
}

void bf_i_printinventoryline(int bv_i_partnum, const char* bv_s_desc_in, int bv_i_qty, int bv_i_reorder) {
    char bv_s_desc[256];
    snprintf(bv_s_desc, sizeof(bv_s_desc), "%s", bv_s_desc_in);

    char bt_s_15[256];
    snprintf(bt_s_15, sizeof(bt_s_15), "%s%s", bcc_stri(bv_i_partnum), "  ");
    char bt_s_16[256];
    snprintf(bt_s_16, sizeof(bt_s_16), "%s%s", bt_s_15, bv_s_desc);
    char bt_s_17[256];
    snprintf(bt_s_17, sizeof(bt_s_17), "%s%s", bt_s_16, "   ");
    char bt_s_18[256];
    snprintf(bt_s_18, sizeof(bt_s_18), "%s%s", bt_s_17, bcc_stri(bv_i_qty));
    char bt_s_19[256];
    snprintf(bt_s_19, sizeof(bt_s_19), "%s%s", bt_s_18, "          ");
    char bt_s_20[256];
    snprintf(bt_s_20, sizeof(bt_s_20), "%s%s", bt_s_19, bcc_stri(bv_i_reorder));
    printf("%s\n", bt_s_20);
}

void bf_i_printreorderheader(void) {
    char bv_s_date[256] = {0};

    printf("\x1b[2J\x1b[H");
    printf("\x1b[%d;%dH", 1, bv_i_tabcol);
    printf("Reorder Report\x1b[%dG%s\n", 55, bv_s_date);
    printf("\n");
    printf("                                             Quantity       Reorder\n");
    printf("    Partno           Description             on hand         level\n");
    printf("   =======  ==============================   ========       =======\n");
}

void bf_i_printreorderline(int bv_i_partnum, const char* bv_s_desc_in, int bv_i_qty, int bv_i_reorder) {
    char bv_s_desc[256];
    snprintf(bv_s_desc, sizeof(bv_s_desc), "%s", bv_s_desc_in);

    char bt_s_21[256];
    snprintf(bt_s_21, sizeof(bt_s_21), "%s%s", "  ", bcc_stri(bv_i_partnum));
    char bt_s_22[256];
    snprintf(bt_s_22, sizeof(bt_s_22), "%s%s", bt_s_21, "  ");
    char bt_s_23[256];
    snprintf(bt_s_23, sizeof(bt_s_23), "%s%s", bt_s_22, bv_s_desc);
    char bt_s_24[256];
    snprintf(bt_s_24, sizeof(bt_s_24), "%s%s", bt_s_23, "   ");
    char bt_s_25[256];
    snprintf(bt_s_25, sizeof(bt_s_25), "%s%s", bt_s_24, bcc_stri(bv_i_qty));
    char bt_s_26[256];
    snprintf(bt_s_26, sizeof(bt_s_26), "%s%s", bt_s_25, "          ");
    char bt_s_27[256];
    snprintf(bt_s_27, sizeof(bt_s_27), "%s%s", bt_s_26, bcc_stri(bv_i_reorder));
    printf("%s\n", bt_s_27);
}

void bf_i_gatherpartdetails(int bv_i_partnum, char* bv_s_desc_in, int* bv_i_qty_in, int* bv_i_reorder_in, float* bv_f_price_in) {
    char bv_s_desc[256];
    snprintf(bv_s_desc, sizeof(bv_s_desc), "%s", bv_s_desc_in);
    int bv_i_qty = *bv_i_qty_in;
    int bv_i_reorder = *bv_i_reorder_in;
    float bv_f_price = *bv_f_price_in;

    printf("\x1b[2J\x1b[H");
    printf("\x1b[%d;%dH", 4, bv_i_tabcol);
    printf("Adding or Overwriting a Record\n");
    printf("\x1b[%d;%dH", 8, bv_i_tabcol);
    char bt_s_28[256];
    snprintf(bt_s_28, sizeof(bt_s_28), "%s%s", "Record/Partno", bcc_stri(bv_i_partnum));
    printf("%s\n", bt_s_28);
    printf("\x1b[%d;%dH", 11, 39);
    printf("------------------------------\n");
    printf("\x1b[%d;%dH", 10, bv_i_tabcol);
    printf("      Description? ");
    bcc_read_line();
    snprintf(bv_s_desc, sizeof(bv_s_desc), "%s", bcc_input_buf);
    printf("\x1b[%d;%dH", 12, bv_i_tabcol);
    printf("Quantity in stock? ");
    bcc_read_line();
    bv_i_qty = atoi(bcc_input_buf);
    printf("\x1b[%d;%dH", 14, bv_i_tabcol);
    printf("    Reorder level? ");
    bcc_read_line();
    bv_i_reorder = atoi(bcc_input_buf);
    printf("\x1b[%d;%dH", 16, bv_i_tabcol);
    printf("       Unit price? ");
    bcc_read_line();
    bv_f_price = atof(bcc_input_buf);
    printf("\x1b[%d;%dH", 18, bv_i_tabcol);
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
    printf("\x1b[%d;%dH", 8, bv_i_tabcol);
    char bt_s_29[256];
    snprintf(bt_s_29, sizeof(bt_s_29), "%s%s", "     Part number: ", bcc_stri(bv_i_partnum));
    printf("%s\n", bt_s_29);
    printf("\x1b[%d;%dH", 9, bv_i_tabcol);
    char bt_s_30[256];
    snprintf(bt_s_30, sizeof(bt_s_30), "%s%s", "Item description: ", bv_s_desc);
    printf("%s\n", bt_s_30);
    printf("\x1b[%d;%dH", 10, bv_i_tabcol);
    char bt_s_31[256];
    snprintf(bt_s_31, sizeof(bt_s_31), "%s%s", "Quantity on hand: ", bcc_stri(bv_i_qty));
    printf("%s\n", bt_s_31);
    printf("\x1b[%d;%dH", 11, bv_i_tabcol);
    char bt_s_32[256];
    snprintf(bt_s_32, sizeof(bt_s_32), "%s%s", "   Reorder Level: ", bcc_stri(bv_i_reorder));
    printf("%s\n", bt_s_32);
}

void bf_i_shownegativeqtywarning(void) {
    printf("\x1b[%d;%dH", 17, 15);
    printf("The quantity to add must NOT be a negative number\n");
    printf("\x1b[%d;%dH", 25, 1);
    printf("Please press the Anykey to reenter quantity to add...");
}

void bf_i_showsubtractstockscreen(int bv_i_partnum, const char* bv_s_desc_in, int bv_i_qty, int bv_i_reorder) {
    char bv_s_desc[256];
    snprintf(bv_s_desc, sizeof(bv_s_desc), "%s", bv_s_desc_in);

    printf("\x1b[2J\x1b[H");
    printf("\x1b[%d;%dH", 4, bv_i_tabcol);
    printf("Subtract an inventory part number\n");
    printf("\x1b[%d;%dH", 5, bv_i_tabcol);
    printf("=================================\n");
    printf("\x1b[%d;%dH", 8, bv_i_tabcol);
    char bt_s_33[256];
    snprintf(bt_s_33, sizeof(bt_s_33), "%s%s", "         Part number: ", bcc_stri(bv_i_partnum));
    printf("%s\n", bt_s_33);
    printf("\x1b[%d;%dH", 9, bv_i_tabcol);
    char bt_s_34[256];
    snprintf(bt_s_34, sizeof(bt_s_34), "%s%s", "    Item description: ", bv_s_desc);
    printf("%s\n", bt_s_34);
    printf("\x1b[%d;%dH", 10, bv_i_tabcol);
    char bt_s_35[256];
    snprintf(bt_s_35, sizeof(bt_s_35), "%s%s", "    Quantity on hand: ", bcc_stri(bv_i_qty));
    printf("%s\n", bt_s_35);
    printf("\x1b[%d;%dH", 11, bv_i_tabcol);
    char bt_s_36[256];
    snprintf(bt_s_36, sizeof(bt_s_36), "%s%s", "       Reorder Level: ", bcc_stri(bv_i_reorder));
    printf("%s\n", bt_s_36);
}

void bf_i_showoversubtractwarning(int bv_i_onhand) {
    printf("\x1b[%d;%dH", 17, 5);
    printf("The quantity to SUBTRACT must NOT result in NEGATIVE inventory\n");
    printf("\x1b[%d;%dH", 18, 5);
    char bt_s_37[256];
    snprintf(bt_s_37, sizeof(bt_s_37), "%s%s", "Only", bcc_stri(bv_i_onhand));
    char bt_s_38[256];
    snprintf(bt_s_38, sizeof(bt_s_38), "%s%s", bt_s_37, " IN STOCK");
    printf("%s\n", bt_s_38);
    printf("\x1b[%d;%dH", 25, 1);
    printf("Please press the Anykey to reenter quantity to subtract...");
}

void bf_i_checkpart(void) {
    float bv_f_pprice = 0;
    int bv_i_part = 0;
    int bv_i_pdesctrimi = 0;
    int bv_i_pflagtrimi = 0;
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
    char bt_s_39[256];
    bf_s_readpartnumberinput(bt_s_39);
    snprintf(bv_s_partstr, sizeof(bv_s_partstr), "%s", bt_s_39);
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
    bv_i_pflagtrimi = ((int)strlen(bv_s_invflagbuf));
    while (((-(bv_i_pflagtrimi > 0)) && (-(strcmp(bcc_mid(bv_s_invflagbuf, bv_i_pflagtrimi, 1), " ") == 0)))) {
        bv_i_pflagtrimi = (bv_i_pflagtrimi - 1);
    }
    snprintf(bv_s_pflag, sizeof(bv_s_pflag), "%s", bcc_mid(bv_s_invflagbuf, 1, bv_i_pflagtrimi));
    bv_i_pdesctrimi = ((int)strlen(bv_s_invdescbuf));
    while (((-(bv_i_pdesctrimi > 0)) && (-(strcmp(bcc_mid(bv_s_invdescbuf, bv_i_pdesctrimi, 1), " ") == 0)))) {
        bv_i_pdesctrimi = (bv_i_pdesctrimi - 1);
    }
    snprintf(bv_s_pdesc, sizeof(bv_s_pdesc), "%s", bcc_mid(bv_s_invdescbuf, 1, bv_i_pdesctrimi));
    bv_i_pqty = bcc_cvi(bv_s_invqtybuf);
    bv_i_preorder = bcc_cvi(bv_s_invreorderbuf);
    bv_f_pprice = bcc_cvs(bv_s_invpricebuf);
    if (bf_i_isempty(bv_s_pflag)) {
        printf("\x1b[2J\x1b[H");
        printf("\x1b[%d;%dH", 10, 18);
        char bt_s_40[256];
        snprintf(bt_s_40, sizeof(bt_s_40), "%s%s", "Part number", bcc_stri(bv_i_part));
        char bt_s_41[256];
        snprintf(bt_s_41, sizeof(bt_s_41), "%s%s", bt_s_40, "is still a null entry at this time");
        printf("%s\n", bt_s_41);
        bf_i_waitanykey();
        return;
    }
    bf_i_showpartstatus(bv_i_part, bv_s_pdesc, bv_i_pqty, bv_i_preorder, bv_f_pprice);
    bf_i_waitanykey();
}

void bf_i_editrecord(void) {
    float bv_f_editprice = 0;
    float bv_f_pprice = 0;
    int bv_i_editqty = 0;
    int bv_i_editreorder = 0;
    int bv_i_part = 0;
    int bv_i_pdesctrimi = 0;
    int bv_i_pflagtrimi = 0;
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
    printf("\x1b[%d;%dH", 10, bv_i_tabcol);
    char bt_s_42[256];
    bf_s_readpartnumberinput(bt_s_42);
    snprintf(bv_s_partstr, sizeof(bv_s_partstr), "%s", bt_s_42);
    bv_i_part = ((int)round((double)(atof(bv_s_partstr))));
    if ((-(bf_i_partinrange(bv_i_part) == 0))) {
        bf_i_showbadpartnumber();
        bf_i_waitanykey();
        return;
    }
    // let p = inv[...]  (whole-record read)
    bcc_get_record_part(bcc_files[0], bv_i_part, bv_s_invflagbuf, bv_s_invdescbuf, bv_s_invqtybuf, bv_s_invreorderbuf, bv_s_invpricebuf);
    bv_i_pflagtrimi = ((int)strlen(bv_s_invflagbuf));
    while (((-(bv_i_pflagtrimi > 0)) && (-(strcmp(bcc_mid(bv_s_invflagbuf, bv_i_pflagtrimi, 1), " ") == 0)))) {
        bv_i_pflagtrimi = (bv_i_pflagtrimi - 1);
    }
    snprintf(bv_s_pflag, sizeof(bv_s_pflag), "%s", bcc_mid(bv_s_invflagbuf, 1, bv_i_pflagtrimi));
    bv_i_pdesctrimi = ((int)strlen(bv_s_invdescbuf));
    while (((-(bv_i_pdesctrimi > 0)) && (-(strcmp(bcc_mid(bv_s_invdescbuf, bv_i_pdesctrimi, 1), " ") == 0)))) {
        bv_i_pdesctrimi = (bv_i_pdesctrimi - 1);
    }
    snprintf(bv_s_pdesc, sizeof(bv_s_pdesc), "%s", bcc_mid(bv_s_invdescbuf, 1, bv_i_pdesctrimi));
    bv_i_pqty = bcc_cvi(bv_s_invqtybuf);
    bv_i_preorder = bcc_cvi(bv_s_invreorderbuf);
    bv_f_pprice = bcc_cvs(bv_s_invpricebuf);
    if ((-(bf_i_isempty(bv_s_pflag) == 0))) {
        printf("\x1b[%d;%dH", 12, bv_i_tabcol);
        printf("Overwrite existing part data?\n");
        char bt_s_43[256];
        bf_s_readkey(bt_s_43);
        snprintf(bv_s_kp, sizeof(bv_s_kp), "%s", bt_s_43);
        if (((-(strcmp(bv_s_kp, "Y") != 0)) && (-(strcmp(bv_s_kp, "y") != 0)))) {
            return;
        }
    }

    while (1) {
        bf_i_gatherpartdetails(bv_i_part, bv_s_editdesc, &bv_i_editqty, &bv_i_editreorder, &bv_f_editprice);
        char bt_s_44[256];
        bf_s_readkey(bt_s_44);
        snprintf(bv_s_kp, sizeof(bv_s_kp), "%s", bt_s_44);
        if (((-(strcmp(bv_s_kp, "Y") == 0)) || (-(strcmp(bv_s_kp, "y") == 0)))) break;
    }
    // inv[...] = { ... }  (whole-record write)
    int16_t bcc_tmp_45 = bv_i_editqty;
    int16_t bcc_tmp_46 = bv_i_editreorder;
    float bcc_tmp_47 = bv_f_editprice;
    bcc_put_record_part(bcc_files[0], bv_i_part, "1", bv_s_editdesc, &bcc_tmp_45, &bcc_tmp_46, &bcc_tmp_47);
}

void bf_i_listall(void) {
    float bv_f_pprice = 0;
    int bv_i_i = 0;
    int bv_i_pdesctrimi = 0;
    int bv_i_pflagtrimi = 0;
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
    int bt_lim_48 = bv_i_partcount;
    int bt_step_48 = 1;
    for (bv_i_i = 1; bt_step_48 >= 0 ? bv_i_i <= bt_lim_48 : bv_i_i >= bt_lim_48; bv_i_i += bt_step_48) {
        // let p = inv[...]  (whole-record read)
        bcc_get_record_part(bcc_files[0], bv_i_i, bv_s_invflagbuf, bv_s_invdescbuf, bv_s_invqtybuf, bv_s_invreorderbuf, bv_s_invpricebuf);
        bv_i_pflagtrimi = ((int)strlen(bv_s_invflagbuf));
        while (((-(bv_i_pflagtrimi > 0)) && (-(strcmp(bcc_mid(bv_s_invflagbuf, bv_i_pflagtrimi, 1), " ") == 0)))) {
            bv_i_pflagtrimi = (bv_i_pflagtrimi - 1);
        }
        snprintf(bv_s_pflag, sizeof(bv_s_pflag), "%s", bcc_mid(bv_s_invflagbuf, 1, bv_i_pflagtrimi));
        bv_i_pdesctrimi = ((int)strlen(bv_s_invdescbuf));
        while (((-(bv_i_pdesctrimi > 0)) && (-(strcmp(bcc_mid(bv_s_invdescbuf, bv_i_pdesctrimi, 1), " ") == 0)))) {
            bv_i_pdesctrimi = (bv_i_pdesctrimi - 1);
        }
        snprintf(bv_s_pdesc, sizeof(bv_s_pdesc), "%s", bcc_mid(bv_s_invdescbuf, 1, bv_i_pdesctrimi));
        bv_i_pqty = bcc_cvi(bv_s_invqtybuf);
        bv_i_preorder = bcc_cvi(bv_s_invreorderbuf);
        bv_f_pprice = bcc_cvs(bv_s_invpricebuf);
        bf_i_printinventoryline(bv_i_i, bv_s_pdesc, bv_i_pqty, bv_i_preorder);
        bv_i_scrollcount = (bv_i_scrollcount + 1);
        if ((-(bv_i_scrollcount == 20))) {
            bf_i_waitanykey();
            bv_i_scrollcount = 0;
        }
    }
}

void bf_i_addstock(void) {
    float bv_f_pprice = 0;
    int bv_i_addamt = 0;
    int bv_i_part = 0;
    int bv_i_pdesctrimi = 0;
    int bv_i_pflagtrimi = 0;
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
        char bt_s_49[256];
        bf_s_readpartnumberinput(bt_s_49);
        snprintf(bv_s_partstr, sizeof(bv_s_partstr), "%s", bt_s_49);
        bv_i_part = ((int)round((double)(atof(bv_s_partstr))));
        bv_i_validpart = bf_i_partinrange(bv_i_part);
        if ((-(bv_i_validpart == 0))) {
            bf_i_showrangeretrymessage();
            char bt_s_50[256];
            bf_s_readkey(bt_s_50);
        }
        if ((-(bv_i_validpart != 0))) break;
    }

    // let p = inv[...]  (whole-record read)
    bcc_get_record_part(bcc_files[0], bv_i_part, bv_s_invflagbuf, bv_s_invdescbuf, bv_s_invqtybuf, bv_s_invreorderbuf, bv_s_invpricebuf);
    bv_i_pflagtrimi = ((int)strlen(bv_s_invflagbuf));
    while (((-(bv_i_pflagtrimi > 0)) && (-(strcmp(bcc_mid(bv_s_invflagbuf, bv_i_pflagtrimi, 1), " ") == 0)))) {
        bv_i_pflagtrimi = (bv_i_pflagtrimi - 1);
    }
    snprintf(bv_s_pflag, sizeof(bv_s_pflag), "%s", bcc_mid(bv_s_invflagbuf, 1, bv_i_pflagtrimi));
    bv_i_pdesctrimi = ((int)strlen(bv_s_invdescbuf));
    while (((-(bv_i_pdesctrimi > 0)) && (-(strcmp(bcc_mid(bv_s_invdescbuf, bv_i_pdesctrimi, 1), " ") == 0)))) {
        bv_i_pdesctrimi = (bv_i_pdesctrimi - 1);
    }
    snprintf(bv_s_pdesc, sizeof(bv_s_pdesc), "%s", bcc_mid(bv_s_invdescbuf, 1, bv_i_pdesctrimi));
    bv_i_pqty = bcc_cvi(bv_s_invqtybuf);
    bv_i_preorder = bcc_cvi(bv_s_invreorderbuf);
    bv_f_pprice = bcc_cvs(bv_s_invpricebuf);
    if (bf_i_isempty(bv_s_pflag)) {
        bf_i_shownullentrymessage(bv_s_partstr);
        char bt_s_51[256];
        bf_s_readkey(bt_s_51);
        return;
    }

    while (1) {
        bf_i_showaddstockscreen(bv_i_part, bv_s_pdesc, bv_i_pqty, bv_i_preorder);
        printf("\x1b[%d;%dH", 14, bv_i_tabcol);
        printf(" Quantity to add? ");
        bcc_read_line();
        snprintf(bv_s_addstr, sizeof(bv_s_addstr), "%s", bcc_input_buf);
        bv_i_addamt = ((int)round((double)(atof(bv_s_addstr))));
        if ((-(bv_i_addamt < 0))) {
            bf_i_shownegativeqtywarning();
            char bt_s_52[256];
            bf_s_readkey(bt_s_52);
        }
        if ((-(bv_i_addamt >= 0))) break;
    }

    bv_i_pqty = (bv_i_pqty + bv_i_addamt);
    // inv[...] = p  (write back a let-bound record)
    int16_t bcc_tmp_53 = bv_i_pqty;
    int16_t bcc_tmp_54 = bv_i_preorder;
    float bcc_tmp_55 = bv_f_pprice;
    bcc_put_record_part(bcc_files[0], bv_i_part, bv_s_pflag, bv_s_pdesc, &bcc_tmp_53, &bcc_tmp_54, &bcc_tmp_55);
}

void bf_i_subtractstock(void) {
    float bv_f_pprice = 0;
    int bv_i_oversubtract = 0;
    int bv_i_part = 0;
    int bv_i_pdesctrimi = 0;
    int bv_i_pflagtrimi = 0;
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
        char bt_s_56[256];
        bf_s_readpartnumberinput(bt_s_56);
        snprintf(bv_s_partstr, sizeof(bv_s_partstr), "%s", bt_s_56);
        bv_i_part = ((int)round((double)(atof(bv_s_partstr))));
        bv_i_validpart = bf_i_partinrange(bv_i_part);
        if ((-(bv_i_validpart == 0))) {
            bf_i_showrangeretrymessage();
            char bt_s_57[256];
            bf_s_readkey(bt_s_57);
        }
        if ((-(bv_i_validpart != 0))) break;
    }

    // let p = inv[...]  (whole-record read)
    bcc_get_record_part(bcc_files[0], bv_i_part, bv_s_invflagbuf, bv_s_invdescbuf, bv_s_invqtybuf, bv_s_invreorderbuf, bv_s_invpricebuf);
    bv_i_pflagtrimi = ((int)strlen(bv_s_invflagbuf));
    while (((-(bv_i_pflagtrimi > 0)) && (-(strcmp(bcc_mid(bv_s_invflagbuf, bv_i_pflagtrimi, 1), " ") == 0)))) {
        bv_i_pflagtrimi = (bv_i_pflagtrimi - 1);
    }
    snprintf(bv_s_pflag, sizeof(bv_s_pflag), "%s", bcc_mid(bv_s_invflagbuf, 1, bv_i_pflagtrimi));
    bv_i_pdesctrimi = ((int)strlen(bv_s_invdescbuf));
    while (((-(bv_i_pdesctrimi > 0)) && (-(strcmp(bcc_mid(bv_s_invdescbuf, bv_i_pdesctrimi, 1), " ") == 0)))) {
        bv_i_pdesctrimi = (bv_i_pdesctrimi - 1);
    }
    snprintf(bv_s_pdesc, sizeof(bv_s_pdesc), "%s", bcc_mid(bv_s_invdescbuf, 1, bv_i_pdesctrimi));
    bv_i_pqty = bcc_cvi(bv_s_invqtybuf);
    bv_i_preorder = bcc_cvi(bv_s_invreorderbuf);
    bv_f_pprice = bcc_cvs(bv_s_invpricebuf);
    if (bf_i_isempty(bv_s_pflag)) {
        bf_i_shownullentrymessage(bv_s_partstr);
        char bt_s_58[256];
        bf_s_readkey(bt_s_58);
        return;
    }

    while (1) {
        bf_i_showsubtractstockscreen(bv_i_part, bv_s_pdesc, bv_i_pqty, bv_i_preorder);
        printf("\x1b[%d;%dH", 14, bv_i_tabcol);
        printf("Quantity to subtract? ");
        bcc_read_line();
        snprintf(bv_s_substr, sizeof(bv_s_substr), "%s", bcc_input_buf);
        bv_i_subamt = ((int)round((double)(atof(bv_s_substr))));
        bv_i_oversubtract = 0;
        if (((-(bv_i_subamt >= 0)) && (-((bv_i_pqty - bv_i_subamt) < 0)))) {
            bv_i_oversubtract = 1;
            bf_i_showoversubtractwarning(bv_i_pqty);
            char bt_s_59[256];
            bf_s_readkey(bt_s_59);
        }
        if (((-(bv_i_subamt >= 0)) && (-(bv_i_oversubtract == 0)))) break;
    }

    bv_i_pqty = (bv_i_pqty - bv_i_subamt);
    if ((-(bv_i_pqty <= bv_i_preorder))) {
        printf("\x1b[%d;%dH", 16, bv_i_tabcol);
    }
    char bt_s_60[256];
    snprintf(bt_s_60, sizeof(bt_s_60), "%s%s", "quantity now", bcc_stri(bv_i_pqty));
    char bt_s_61[256];
    snprintf(bt_s_61, sizeof(bt_s_61), "%s%s", bt_s_60, " reorder level");
    char bt_s_62[256];
    snprintf(bt_s_62, sizeof(bt_s_62), "%s%s", bt_s_61, bcc_stri(bv_i_preorder));
    printf("%s\n", bt_s_62);
    // inv[...] = p  (write back a let-bound record)
    int16_t bcc_tmp_63 = bv_i_pqty;
    int16_t bcc_tmp_64 = bv_i_preorder;
    float bcc_tmp_65 = bv_f_pprice;
    bcc_put_record_part(bcc_files[0], bv_i_part, bv_s_pflag, bv_s_pdesc, &bcc_tmp_63, &bcc_tmp_64, &bcc_tmp_65);
}

void bf_i_reorderreport(void) {
    float bv_f_pprice = 0;
    int bv_i_i = 0;
    int bv_i_pdesctrimi = 0;
    int bv_i_pflagtrimi = 0;
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
    int bt_lim_66 = bv_i_partcount;
    int bt_step_66 = 1;
    for (bv_i_i = 1; bt_step_66 >= 0 ? bv_i_i <= bt_lim_66 : bv_i_i >= bt_lim_66; bv_i_i += bt_step_66) {
        // let p = inv[...]  (whole-record read)
        bcc_get_record_part(bcc_files[0], bv_i_i, bv_s_invflagbuf, bv_s_invdescbuf, bv_s_invqtybuf, bv_s_invreorderbuf, bv_s_invpricebuf);
        bv_i_pflagtrimi = ((int)strlen(bv_s_invflagbuf));
        while (((-(bv_i_pflagtrimi > 0)) && (-(strcmp(bcc_mid(bv_s_invflagbuf, bv_i_pflagtrimi, 1), " ") == 0)))) {
            bv_i_pflagtrimi = (bv_i_pflagtrimi - 1);
        }
        snprintf(bv_s_pflag, sizeof(bv_s_pflag), "%s", bcc_mid(bv_s_invflagbuf, 1, bv_i_pflagtrimi));
        bv_i_pdesctrimi = ((int)strlen(bv_s_invdescbuf));
        while (((-(bv_i_pdesctrimi > 0)) && (-(strcmp(bcc_mid(bv_s_invdescbuf, bv_i_pdesctrimi, 1), " ") == 0)))) {
            bv_i_pdesctrimi = (bv_i_pdesctrimi - 1);
        }
        snprintf(bv_s_pdesc, sizeof(bv_s_pdesc), "%s", bcc_mid(bv_s_invdescbuf, 1, bv_i_pdesctrimi));
        bv_i_pqty = bcc_cvi(bv_s_invqtybuf);
        bv_i_preorder = bcc_cvi(bv_s_invreorderbuf);
        bv_f_pprice = bcc_cvs(bv_s_invpricebuf);
        if ((-(bv_i_pqty < bv_i_preorder))) {
            bf_i_printreorderline(bv_i_i, bv_s_pdesc, bv_i_pqty, bv_i_preorder);
            bv_i_reportlinecount = (bv_i_reportlinecount + 1);
            if ((-(bv_i_reportlinecount > 15))) {
                bf_i_waitanykey();
                bv_i_reportlinecount = 0;
            }
        }
    }
    bf_i_waitanykey();
}

void bf_i_initializeinventoryfileifnew(void) {
    float bv_f_pprice = 0;
    int bv_i_i = 0;
    int bv_i_pdesctrimi = 0;
    int bv_i_pflagtrimi = 0;
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
    bv_i_pflagtrimi = ((int)strlen(bv_s_invflagbuf));
    while (((-(bv_i_pflagtrimi > 0)) && (-(strcmp(bcc_mid(bv_s_invflagbuf, bv_i_pflagtrimi, 1), " ") == 0)))) {
        bv_i_pflagtrimi = (bv_i_pflagtrimi - 1);
    }
    snprintf(bv_s_pflag, sizeof(bv_s_pflag), "%s", bcc_mid(bv_s_invflagbuf, 1, bv_i_pflagtrimi));
    bv_i_pdesctrimi = ((int)strlen(bv_s_invdescbuf));
    while (((-(bv_i_pdesctrimi > 0)) && (-(strcmp(bcc_mid(bv_s_invdescbuf, bv_i_pdesctrimi, 1), " ") == 0)))) {
        bv_i_pdesctrimi = (bv_i_pdesctrimi - 1);
    }
    snprintf(bv_s_pdesc, sizeof(bv_s_pdesc), "%s", bcc_mid(bv_s_invdescbuf, 1, bv_i_pdesctrimi));
    bv_i_pqty = bcc_cvi(bv_s_invqtybuf);
    bv_i_preorder = bcc_cvi(bv_s_invreorderbuf);
    bv_f_pprice = bcc_cvs(bv_s_invpricebuf);
    if ((-(((int)(unsigned char)bv_s_pflag[0]) == 0))) {
        int bt_lim_67 = bv_i_partcount;
        int bt_step_67 = 1;
        for (bv_i_i = 1; bt_step_67 >= 0 ? bv_i_i <= bt_lim_67 : bv_i_i >= bt_lim_67; bv_i_i += bt_step_67) {
            // inv[...] = { ... }  (whole-record write)
            int16_t bcc_tmp_68 = 0;
            int16_t bcc_tmp_69 = 0;
            float bcc_tmp_70 = 0;
            bcc_put_record_part(bcc_files[0], bv_i_i, bcc_chr(255), "", &bcc_tmp_68, &bcc_tmp_69, &bcc_tmp_70);
        }
    }
}

void bf_i_reportinventoryerror(int bv_i_err, int bv_i_erl) {
    char bv_s_k[256] = {0};

    printf("\x1b[%d;%dH", 25, 1);
    char bt_s_71[256];
    snprintf(bt_s_71, sizeof(bt_s_71), "%s%s", "There has been an error on line", bcc_stri(bv_i_erl));
    char bt_s_72[256];
    snprintf(bt_s_72, sizeof(bt_s_72), "%s%s", bt_s_71, ": ");
    char bt_s_73[256];
    bf_s_error(bv_i_err, bt_s_73);
    char bt_s_74[256];
    snprintf(bt_s_74, sizeof(bt_s_74), "%s%s", bt_s_72, bt_s_73);
    printf("%s\n", bt_s_74);
    char bt_s_75[256];
    bf_s_readkey(bt_s_75);
    snprintf(bv_s_k, sizeof(bv_s_k), "%s", bt_s_75);
}

int main(void) {
    setvbuf(stdin, NULL, _IONBF, 0);
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

    bv_i_errsyntax = 2;
    bv_i_errreturnwithoutgosub = 3;
    bv_i_erroutofdata = 4;
    bv_i_errillegalfunctioncall = 5;
    bv_i_erroverflow = 6;
    bv_i_erroutofmemory = 7;
    bv_i_errsubscriptoutofrange = 9;
    bv_i_errduplicatedefinition = 10;
    bv_i_errdivisionbyzero = 11;
    bv_i_errtypemismatch = 13;
    bv_i_erroutofstringspace = 14;
    bv_i_errnoresume = 19;
    bv_i_errresumewithouterror = 20;
    bv_i_errdevicetimeout = 24;
    bv_i_errdevicefault = 25;
    bv_i_erroutofpaper = 27;
    bv_i_errbadfilenumber = 52;
    bv_i_errfilenotfound = 53;
    bv_i_errbadfilemode = 54;
    bv_i_errfilealreadyopen = 55;
    bv_i_errdeviceio = 57;
    bv_i_errfilealreadyexists = 58;
    bv_i_errdiskfull = 61;
    bv_i_errinputpastend = 62;
    bv_i_errbadrecordnumber = 63;
    bv_i_errbadfilename = 64;
    bv_i_errtoomanyfiles = 67;
    bv_i_errdeviceunavailable = 68;
    bv_i_errdiskwriteprotected = 70;
    bv_i_errdisknotready = 71;
    bv_i_errdiskmediaerror = 72;
    bv_i_errpathfileaccess = 75;
    bv_i_errpathnotfound = 76;

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
    // V=30) are collapsed into a single `tabCol% = 20`; a couple of
    // screens that used U=25 in the original (see showAddStockScreen
    // below) keep 25 as a literal rather than reusing tabCol%.
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
    bv_i_partcount = 100;
    bv_i_tabcol = 20;

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
    char bt_s_76[256];
    bf_s_error(bv_i_err, bt_s_76);
    char bt_s_77[256];
    snprintf(bt_s_77, sizeof(bt_s_77), "%s%s", "could not open inven.dat: ", bt_s_76);
    printf("%s\n", bt_s_77);
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
        char bt_s_78[256];
        bf_s_readkey(bt_s_78);
        snprintf(bv_s_kp, sizeof(bv_s_kp), "%s", bt_s_78);
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
                char bt_sel_79[256];
                snprintf(bt_sel_79, sizeof(bt_sel_79), "%s", bv_s_kp);
                int bt_sel_match_80 = 0;
                if (!bt_sel_match_80) {
                    if ((strcmp(bt_sel_79, "1") == 0) || (strcmp(bt_sel_79, "c") == 0) || (strcmp(bt_sel_79, "C") == 0)) {
                        bt_sel_match_80 = 1;
                        bf_i_checkpart();
                    }
                }
                if (!bt_sel_match_80) {
                    if ((strcmp(bt_sel_79, "2") == 0) || (strcmp(bt_sel_79, "e") == 0) || (strcmp(bt_sel_79, "E") == 0)) {
                        bt_sel_match_80 = 1;
                        bf_i_editrecord();
                    }
                }
                if (!bt_sel_match_80) {
                    if ((strcmp(bt_sel_79, "3") == 0) || (strcmp(bt_sel_79, "l") == 0) || (strcmp(bt_sel_79, "L") == 0)) {
                        bt_sel_match_80 = 1;
                        bf_i_listall();
                    }
                }
                if (!bt_sel_match_80) {
                    if ((strcmp(bt_sel_79, "4") == 0) || (strcmp(bt_sel_79, "a") == 0) || (strcmp(bt_sel_79, "A") == 0)) {
                        bt_sel_match_80 = 1;
                        bf_i_addstock();
                    }
                }
                if (!bt_sel_match_80) {
                    if ((strcmp(bt_sel_79, "5") == 0) || (strcmp(bt_sel_79, "s") == 0) || (strcmp(bt_sel_79, "S") == 0)) {
                        bt_sel_match_80 = 1;
                        bf_i_subtractstock();
                    }
                }
                if (!bt_sel_match_80) {
                    if ((strcmp(bt_sel_79, "6") == 0) || (strcmp(bt_sel_79, "r") == 0) || (strcmp(bt_sel_79, "R") == 0)) {
                        bt_sel_match_80 = 1;
                        bf_i_reorderreport();
                    }
                }
                if (!bt_sel_match_80) {
                    if ((strcmp(bt_sel_79, "7") == 0) || (strcmp(bt_sel_79, "x") == 0) || (strcmp(bt_sel_79, "X") == 0)) {
                        bt_sel_match_80 = 1;
                        // BASCAL-ism: `inv.close()` is sugar for `CLOSE #1`,
                        // matching fhb's own `90 CLOSE:SYSTEM`. fhb's original
                        // also had a separate "Quit to BASIC" option (his own
                        // 7, returning to the interpreter's command prompt
                        // rather than exiting to DOS) -- dropped here: a
                        // compiled program has no interpreter to return to,
                        // so it was never anything but a second spelling of
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
            bf_i_reportinventoryerror(bv_i_err, bv_i_erl);
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
    if (_kbhit()) {
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

static void bcc_mks(char* out, double value) {
    float v = (float)value;
    memcpy(out, &v, 4);
}

static void bcc_mkd(char* out, double value) {
    memcpy(out, &value, 8);
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
