10 ' BASCAL generated BASIC
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' ============================================================
40 ' INVENTORY.BCL -- Random-Access Inventory Program
50 ' 
60 ' A BASCAL reconstruction of "Example program for RANDOM ACCESS
70 ' FILE study", by fhb, 8/19/98, from Joseph Sixpack's GW-BASIC
80 ' programs page (part of his "Last Book of GW-Basic" collection):
90 ' http://www.geocities.ws/joseph_sixpack/binventory.html
100 ' fhb's own header comment credits the original as "suggested
110 ' from MS-BASIC manual".
120 ' 
130 ' This is a reconstruction, not a line-by-line port -- some
140 ' original pieces have no BASCAL equivalent and were dropped
150 ' rather than approximated:
160 ' - The GOTO-driven "subroutine roadmap" dispatcher at the top
170 ' of fhb's listing (a `LIST 110-320` etc. navigation aid for
180 ' editing in the GW-BASIC interpreter) has no meaning once the
190 ' program is structured into named function/procedure blocks.
200 ' - `KEY OFF` / `KEY I,""` (clearing the function-key soft-label
210 ' row) and `VIEW PRINT` (scroll-region windowing for the list
220 ' screen) are interpreter/console features BASCAL doesn't
230 ' expose.
240 ' - fhb's numeric-ERR-code-to-message lookup table (ERR=1 "Input
250 ' value overflow", ERR=2 "Syntax error", ... ERR=25) is
260 ' collapsed below into a single line reporting the raw ERR/ERL
270 ' values instead.
280 ' - fhb's one-time "hidden" datafile initializer (PUT-ing 100
290 ' blank, CHR$(255)-flagged records) isn't reproduced here --
300 ' inven.dat must be pre-populated with 100 such blank records
310 ' before running this program, or isEmpty%() will read
320 ' uninitialized/zero-filled records as never-empty.
330 ' - The three original tab-position constants (T=20, U=25,
340 ' V=30) are collapsed into a single `tabCol% = 20`; a couple of
350 ' screens that used U=25 in the original (see showAddStockScreen
360 ' below) keep 25 as a literal rather than reusing tabCol%.
370 ' 
380 ' Tracks parts in a fixed 100-record file: check status, add,
390 ' edit, add/subtract stock, and a reorder report.
400 ' 
410 ' This version assumes both of the following bcc bugs are fixed:
420 ' - `const` referenced inside a function/procedure body now
430 ' resolves to the real constant (partCount%/tabCol% are used
440 ' directly below, no more literal-inlining workaround).
450 ' - `INKEY$` referenced inside a function/procedure body now
460 ' resolves to the real builtin (readKey$()/waitAnyKey() call
470 ' it directly; no more shared top-level GOSUB-subroutine
480 ' indirection through a global `lastKey$`).
490 ' (The record/file DSL had this same "breaks inside a procedure"
500 ' problem too; that one was already fixed upstream before this
510 ' file existed -- inv[...] is used freely inside procedures.)
520 ' 
530 ' Verified against real BASCOM 2.00 under dosbox-x: compiles
540 ' clean and links, but -- because this program uses `on error
550 ' goto` / `resume` -- only when BASCOM is invoked with the /E and
560 ' /X switches (error trapping isn't linked in by default). See
570 ' `errorTrap:` at the bottom.
580 ' ============================================================

590 ' BASCAL-ism: the record/file DSL. `record ... end record` plus
600 ' `file ... as ... = open(...)` below replace fhb's manual
610 ' FIELD #1,1 AS F$,30 AS D$,2 AS Q$,... buffer layout entirely --
620 ' bcc computes the field widths and record LEN from this
630 ' declaration and generates the FIELD statement itself. Named
640 ' field access (`p.flag`, `p.qty`, ...) and whole-record
650 ' read/write via `inv[n]` (see checkPart() below) replace fhb's
660 ' manual GET/PUT plus LSET/RSET and MKI$/MKS$/CVI$/CVS$ packing.

670 ' BASCAL-ism: `const` is a real compile-time constant, not a plain
680 ' variable assignment like fhb's `N=100` / `T=20` -- it can never
690 ' be reassigned, and (per the header note above) resolves to the
700 ' same value everywhere, including inside every function/procedure
710 ' below, with no `global` declaration needed.
720 partcount% = 100
730 tabcol% = 20

740 ' `file ... = open(...)` is sugar for OPEN ... FOR RANDOM AS #n
750 ' LEN = <record width> plus the FIELD statement fhb wrote out by
760 ' hand at his line 550.
770 ' file inv as Part = open(...)  [39 bytes/record]
780 OPEN "inven.dat" FOR RANDOM AS #1 LEN = 39
790 FIELD #1, 1 AS invFlagBuf$, 30 AS invDescBuf$, 2 AS invQtyBuf$, 2 AS invReorderBuf$, 4 AS invPriceBuf$

800 ' -------------------- Pure functions (no file access) --------------------

810 ' BASCAL-ism: `function ... end function` with `return` replaces
820 ' fhb's convention of a GOSUB target plus a bare RETURN -- there's
830 ' no separate "subroutine label" and no shared/global result
840 ' variable to manage by hand; `isEmpty%(...)` is called like an
850 ' ordinary expression at every use below (e.g. `isEmpty%(p.flag)`).
860 ' A record whose flag byte is CHR$(255) is an empty/never-used slot.

870 ' BASCAL-ism: `&&` and `||` are short-circuit AND/OR -- real
880 ' MBASIC/BASCOM only has bitwise AND/OR (which fhb relies on here
890 ' too, since `PART!<1 OR PART!>N!` never short-circuits anyway).
900 ' BASCAL lowers `&&`/`||` into the equivalent branching so the
910 ' short-circuit *is* real at the generated-BASIC level; see
920 ' MANUAL.md's "Short-Circuit && and ||" section.

930 ' -------------------- Keyboard input --------------------

940 ' BASCAL-ism: `do ... loop until` is a structured post-check loop
950 ' replacing fhb's `730 KP$=INKEY$:IF KP$="" THEN 730` GOTO-polling
960 ' idiom. `inkey$` itself is the real INKEY$ builtin passed straight
970 ' through -- see the header note above on why it needed a bcc fix
980 ' to resolve correctly from inside a function/procedure body like
990 ' this one (every menu action below calls readKey$()/waitAnyKey()
1000 ' instead of polling INKEY$ inline, which is exactly the pattern
1010 ' that exposed the original bug).

1020 ' -------------------- Display procedures --------------------

1030 ' byref scalar parameters: gatherPartDetails writes the four editable
1040 ' fields for a part directly back into the caller's variables.

1050 ' -------------------- Menu actions --------------------

1060 ' -------------------- Program entry --------------------

1070 CLS
1080 ON ERROR GOTO 1590

1090     GOSUB 2080
1100     GOSUB 1930
1110     kp$ = readkeyResult0$
1120     IF (INSTR("12345678cCeElLaAsSrRqQxX", kp$) <> 0) = 0 THEN GOTO 1470
1130         ' BASCAL-ism: `select case` replaces fhb's chain of eight
1140         ' `IF VAL(KP$)=n OR KP$="x" OR KP$="X" THEN GOTO ...` lines
1150         ' (his 770-840) with one multi-way dispatch.
1160         BCCT4$ = kp$
1170         IF (BCCT4$ = "1" OR BCCT4$ = "c" OR BCCT4$ = "C") <> 0 THEN GOTO 1260
1180         IF (BCCT4$ = "2" OR BCCT4$ = "e" OR BCCT4$ = "E") <> 0 THEN GOTO 1280
1190         IF (BCCT4$ = "3" OR BCCT4$ = "l" OR BCCT4$ = "L") <> 0 THEN GOTO 1300
1200         IF (BCCT4$ = "4" OR BCCT4$ = "a" OR BCCT4$ = "A") <> 0 THEN GOTO 1320
1210         IF (BCCT4$ = "5" OR BCCT4$ = "s" OR BCCT4$ = "S") <> 0 THEN GOTO 1340
1220         IF (BCCT4$ = "6" OR BCCT4$ = "r" OR BCCT4$ = "R") <> 0 THEN GOTO 1360
1230         IF (BCCT4$ = "7" OR BCCT4$ = "q" OR BCCT4$ = "Q") <> 0 THEN GOTO 1380
1240         IF (BCCT4$ = "8" OR BCCT4$ = "x" OR BCCT4$ = "X") <> 0 THEN GOTO 1400
1250         GOTO 1460
1260             GOSUB 3620
1270             GOTO 1460
1280             GOSUB 4160
1290             GOTO 1460
1300             GOSUB 4850
1310             GOTO 1460
1320             GOSUB 5210
1330             GOTO 1460
1340             GOSUB 5890
1350             GOTO 1460
1360             GOSUB 6660
1370             GOTO 1460
1380             quitflag% = 1
1390             GOTO 1460
1400             ' BASCAL-ism: `inv.close()` is sugar for `CLOSE #1`,
1410             ' matching fhb's own `90 CLOSE:SYSTEM`.
1420             ' inv.close()
1430             CLOSE #1
1440             SYSTEM
1450             GOTO 1460
1460         REM END SELECT
1470     REM END IF
1480     IF (quitflag% = 1) = 0 THEN GOTO 1090
1490 REM END DO

1500 ' inv.close()
1510 CLOSE #1
1520 END

1530 ' -------------------- Error handling --------------------
1540 ' Still a raw label, not a procedure: ON ERROR GOTO's target and RESUME
1550 ' both need to act on the main program's execution state, not a nested
1560 ' GOSUB/function-call frame -- that's a language-design reason, unrelated
1570 ' to either compiler bug, so it stays a label regardless of the fixes
1580 ' above.
1590 LOCATE 25, 1
1600 ' `err` and `erl` are real numeric system pseudo-variables, passed
1610 ' straight through like fhb's own ERR/ERL (his 3390: "an error on
1620 ' line";ERL). Unlike fhb's version, this doesn't decode ERR into a
1630 ' message per the header note above -- it just reports the raw
1640 ' code. (These aren't referenced from inside a function/procedure
1650 ' body anywhere in this file, so the bcc `const`/INKEY$-in-a-
1660 ' procedure fixes this file exists to demonstrate don't apply to
1670 ' them here -- but they had the identical bug and fix upstream;
1680 ' see MANUAL.md's ON ERROR GOTO section.)
1690 PRINT (("There has been an error on line" + STR$(ERL)) + "  Error #") + STR$(ERR)
1700 GOSUB 1930
1710 k$ = readkeyResult0$
1720 RESUME NEXT
1730 END

1740 ' function isempty%(flag$)
1750     isemptyResult0% = ASC(isemptyFlag0$) = 255
1760     RETURN
1770 ' end function isempty%

1780 ' function partinrange%(n%)
1790     IF (partinrangeN0% >= 1) = 0 THEN GOTO 1830
1800     IF (partinrangeN0% <= partcount%) = 0 THEN GOTO 1830
1810         partinrangeResult0% = 1
1820         RETURN
1830     REM END IF
1840     partinrangeResult0% = 0
1850     RETURN
1860 ' end function partinrange%

1870 ' function readpartnumberinput$()
1880     INPUT "Input part number"; readpartnumberinputS0$
1890     readpartnumberinputResult0$ = readpartnumberinputS0$
1900     RETURN
1910 ' end function readpartnumberinput$

1920 ' function readkey$()
1930         readkeyK0$ = INKEY$
1940         IF (readkeyK0$ <> "") = 0 THEN GOTO 1930
1950     REM END DO
1960     readkeyResult0$ = readkeyK0$
1970     RETURN
1980 ' end function readkey$

1990 ' procedure waitanykey()
2000     LOCATE 25, 10
2010     PRINT "Press the AnyKey to continue...";
2020         waitanykeyK0$ = INKEY$
2030         IF (waitanykeyK0$ <> "") = 0 THEN GOTO 2020
2040     REM END DO
2050     RETURN
2060 ' end procedure waitanykey

2070 ' procedure showmainmenu()
2080     CLS
2090     COLOR 14, 4
2100     CLS
2110     LOCATE 6, 1
2120     PRINT
2130     ' `tab(n)` passes straight through to real TAB(n), same as
2140     ' fhb's own `PRINT TAB(V) "..."` -- but only as a bare item in
2150     ' a PRINT list, juxtaposed or `;`-separated like here. Real
2160     ' BASCOM rejects `"literal" + tab(n) + ...` (TAB isn't a real
2170     ' string function you can concatenate); see printListHeader()
2180     ' and printReorderHeader() below, which need `;` between a
2190     ' preceding string and a `tab(n)` for exactly this reason.
2200     PRINT TAB(30)"Inventory Program"
2210     PRINT
2220     PRINT TAB(tabcol%)"1......C)heck a part"
2230     PRINT TAB(tabcol%)"2......E)dit/overwrite/add a part"
2240     PRINT TAB(tabcol%)("3......L)ist all" + STR$(partcount%)) + "parts"
2250     PRINT TAB(tabcol%)"4......A)dd stock"
2260     PRINT TAB(tabcol%)"5......S)ubtract stock"
2270     PRINT TAB(tabcol%)"6......R)eorder Report"
2280     PRINT
2290     PRINT TAB(tabcol%)"7......Q)uit to BASIC"
2300     PRINT TAB(tabcol%)"8......eX)it to system"
2310     RETURN
2320 ' end procedure showmainmenu

2330 ' procedure showbadpartnumber()
2340     CLS
2350     LOCATE 10, 10
2360     PRINT "Part number is out of permissable range of 1 to" + STR$(partcount%)
2370     RETURN
2380 ' end procedure showbadpartnumber

2390 ' procedure showrangeretrymessage()
2400     LOCATE 10, 15
2410     PRINT "The Part number is out of permissable range of 1 to" + STR$(partcount%)
2420     LOCATE 25, 15
2430     PRINT "Press the Anykey to reenter part number...";
2440     RETURN
2450 ' end procedure showrangeretrymessage

2460 ' procedure shownullentrymessage(partstr$)
2470     LOCATE 10, tabcol%
2480     PRINT ("Part number " + shownullentrymessagePartStr0$) + " is a null entry"
2490     RETURN
2500 ' end procedure shownullentrymessage

2510 ' procedure showpartstatus(partnum%, desc$, qty%, reorder%, price!)
2520     CLS
2530     LOCATE 5, 1
2540     PRINT TAB(tabcol%)"Inventory Status for Individual Part Number"
2550     PRINT TAB(tabcol%)"==========================================="
2560     PRINT
2570     PRINT
2580     PRINT TAB(tabcol%)"     Part number:  " + STR$(showpartstatusPartNum0%)
2590     PRINT
2600     PRINT TAB(tabcol%)"       Item name:  " + showpartstatusDesc0$
2610     PRINT TAB(tabcol%)"Quantity on hand:  " + STR$(showpartstatusQty0%)
2620     PRINT TAB(tabcol%)"   Reorder level:  " + STR$(showpartstatusReorder0%)
2630     PRINT TAB(tabcol%)"      Unit price:  " + STR$(showpartstatusPrice0!)
2640     RETURN
2650 ' end procedure showpartstatus

2660 ' procedure printlistheader()
2670     CLS
2680     PRINT TAB(25)"I N V E N T O R Y   L I S T I N G"; TAB(65); STR$(partcount%) + "items"
2690     PRINT "                                          Quantity       Reorder"
2700     PRINT " Partno           Description             on hand         level"
2710     LOCATE 25, 1
2720     PRINT "Press the AnyKey to scroll listing...";
2730     RETURN
2740 ' end procedure printlistheader

2750 ' procedure printinventoryline(partnum%, desc$, qty%, reorder%)
2760     PRINT (((((STR$(printinventorylinePartNum0%) + "  ") + printinventorylineDesc0$) + "   ") + STR$(printinventorylineQty0%)) + "          ") + STR$(printinventorylineReorder0%)
2770     RETURN
2780 ' end procedure printinventoryline

2790 ' procedure printreorderheader()
2800     CLS
2810     LOCATE 1, tabcol%
2820     PRINT "Reorder Report"; TAB(55); DATE$
2830     PRINT
2840     PRINT "                                             Quantity       Reorder"
2850     PRINT "    Partno           Description             on hand         level"
2860     PRINT "   =======  ==============================   ========       ======="
2870     RETURN
2880 ' end procedure printreorderheader

2890 ' procedure printreorderline(partnum%, desc$, qty%, reorder%)
2900     PRINT (((((("  " + STR$(printreorderlinePartNum0%)) + "  ") + printreorderlineDesc0$) + "   ") + STR$(printreorderlineQty0%)) + "          ") + STR$(printreorderlineReorder0%)
2910     RETURN
2920 ' end procedure printreorderline

2930 ' procedure gatherpartdetails(partnum%, desc$, qty%, reorder%, price!)
2940     CLS
2950     LOCATE 4, tabcol%
2960     PRINT "Adding or Overwriting a Record"
2970     LOCATE 8, tabcol%
2980     PRINT "Record/Partno" + STR$(gatherpartdetailsPartNum0%)
2990     LOCATE 11, 39
3000     PRINT "------------------------------"
3010     LOCATE 10, tabcol%
3020     INPUT "      Description"; gatherpartdetailsDesc0$
3030     LOCATE 12, tabcol%
3040     INPUT "Quantity in stock"; gatherpartdetailsQty0%
3050     LOCATE 14, tabcol%
3060     INPUT "    Reorder level"; gatherpartdetailsReorder0%
3070     LOCATE 16, tabcol%
3080     INPUT "       Unit price"; gatherpartdetailsPrice0!
3090     LOCATE 18, tabcol%
3100     PRINT "Is information correct (Y/N)?"
3110     RETURN
3120 ' end procedure gatherpartdetails

3130 ' procedure showaddstockscreen(partnum%, desc$, qty%, reorder%)
3140     CLS
3150     LOCATE 4, 25
3160     PRINT "Add to an inventory part number"
3170     LOCATE 5, 25
3180     PRINT "==============================="
3190     LOCATE 8, tabcol%
3200     PRINT "     Part number: " + STR$(showaddstockscreenPartNum0%)
3210     LOCATE 9, tabcol%
3220     PRINT "Item description: " + showaddstockscreenDesc0$
3230     LOCATE 10, tabcol%
3240     PRINT "Quantity on hand: " + STR$(showaddstockscreenQty0%)
3250     LOCATE 11, tabcol%
3260     PRINT "   Reorder Level: " + STR$(showaddstockscreenReorder0%)
3270     RETURN
3280 ' end procedure showaddstockscreen

3290 ' procedure shownegativeqtywarning()
3300     LOCATE 17, 15
3310     PRINT "The quantity to add must NOT be a negative number"
3320     LOCATE 25, 1
3330     PRINT "Please press the Anykey to reenter quantity to add...";
3340     RETURN
3350 ' end procedure shownegativeqtywarning

3360 ' procedure showsubtractstockscreen(partnum%, desc$, qty%, reorder%)
3370     CLS
3380     LOCATE 4, tabcol%
3390     PRINT "Subtract an inventory part number"
3400     LOCATE 5, tabcol%
3410     PRINT "================================="
3420     LOCATE 8, tabcol%
3430     PRINT "         Part number: " + STR$(showsubtractstockscreenPartNum0%)
3440     LOCATE 9, tabcol%
3450     PRINT "    Item description: " + showsubtractstockscreenDesc0$
3460     LOCATE 10, tabcol%
3470     PRINT "    Quantity on hand: " + STR$(showsubtractstockscreenQty0%)
3480     LOCATE 11, tabcol%
3490     PRINT "       Reorder Level: " + STR$(showsubtractstockscreenReorder0%)
3500     RETURN
3510 ' end procedure showsubtractstockscreen

3520 ' procedure showoversubtractwarning(onhand%)
3530     LOCATE 17, 5
3540     PRINT "The quantity to SUBTRACT must NOT result in NEGATIVE inventory"
3550     LOCATE 18, 5
3560     PRINT ("Only" + STR$(showoversubtractwarningOnHand0%)) + " IN STOCK"
3570     LOCATE 25, 1
3580     PRINT "Please press the Anykey to reenter quantity to subtract...";
3590     RETURN
3600 ' end procedure showoversubtractwarning

3610 ' procedure checkpart()
3620     GOSUB 1880
3630     checkpartPartStr0$ = readpartnumberinputResult0$
3640     checkpartPart0% = VAL(checkpartPartStr0$)
3650     partinrangeN0% = checkpartPart0%
3660     GOSUB 1790
3670     IF (partinrangeResult0% = 0) = 0 THEN GOTO 3710
3680         GOSUB 2340
3690         GOSUB 2000
3700         RETURN
3710     REM END IF
3720     ' BASCAL-ism: `let p = inv[part%]` reads record `part%` of the
3730     ' `inv` file into a local record variable `p` -- one expression
3740     ' for what fhb's `GET #1, PART!` plus five separate field reads
3750     ' (F$, D$, CVI(Q$), CVI(R$), CVS(P$)) did by hand. The write
3760     ' side, `inv[part%] = { ... }` (see editRecord() below), is the
3770     ' same sugar for PUT plus the LSET/MKx$ packing it replaces.
3780     ' let p = inv[...]  (whole-record read)
3790     GET #1, checkpartPart0%
3800     checkpartPFlagTrimI0% = LEN(invFlagBuf$)
3810     IF (checkpartPFlagTrimI0% > 0) = 0 THEN GOTO 3850
3820     IF (MID$(invFlagBuf$, checkpartPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 3850
3830         checkpartPFlagTrimI0% = checkpartPFlagTrimI0% - 1
3840         GOTO 3810
3850     REM END WHILE
3860     checkpartPFlag0$ = LEFT$(invFlagBuf$, checkpartPFlagTrimI0%)
3870     checkpartPDescTrimI0% = LEN(invDescBuf$)
3880     IF (checkpartPDescTrimI0% > 0) = 0 THEN GOTO 3920
3890     IF (MID$(invDescBuf$, checkpartPDescTrimI0%, 1) = " ") = 0 THEN GOTO 3920
3900         checkpartPDescTrimI0% = checkpartPDescTrimI0% - 1
3910         GOTO 3880
3920     REM END WHILE
3930     checkpartPDesc0$ = LEFT$(invDescBuf$, checkpartPDescTrimI0%)
3940     checkpartPQty0% = CVI(invQtyBuf$)
3950     checkpartPReorder0% = CVI(invReorderBuf$)
3960     checkpartPPrice0! = CVS(invPriceBuf$)
3970     isemptyFlag0$ = checkpartPFlag0$
3980     GOSUB 1750
3990     IF (isemptyResult0%) = 0 THEN GOTO 4050
4000         CLS
4010         LOCATE 10, 18
4020         PRINT ("Part number" + STR$(checkpartPart0%)) + "is still a null entry at this time"
4030         GOSUB 2000
4040         RETURN
4050     REM END IF
4060     showpartstatusPartNum0% = checkpartPart0%
4070     showpartstatusDesc0$ = checkpartPDesc0$
4080     showpartstatusQty0% = checkpartPQty0%
4090     showpartstatusReorder0% = checkpartPReorder0%
4100     showpartstatusPrice0! = checkpartPPrice0!
4110     GOSUB 2520
4120     GOSUB 2000
4130     RETURN
4140 ' end procedure checkpart

4150 ' procedure editrecord()
4160     CLS
4170     LOCATE 10, tabcol%
4180     GOSUB 1880
4190     editrecordPartStr0$ = readpartnumberinputResult0$
4200     editrecordPart0% = VAL(editrecordPartStr0$)
4210     partinrangeN0% = editrecordPart0%
4220     GOSUB 1790
4230     IF (partinrangeResult0% = 0) = 0 THEN GOTO 4270
4240         GOSUB 2340
4250         GOSUB 2000
4260         RETURN
4270     REM END IF
4280     ' let p = inv[...]  (whole-record read)
4290     GET #1, editrecordPart0%
4300     editrecordPFlagTrimI0% = LEN(invFlagBuf$)
4310     IF (editrecordPFlagTrimI0% > 0) = 0 THEN GOTO 4350
4320     IF (MID$(invFlagBuf$, editrecordPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 4350
4330         editrecordPFlagTrimI0% = editrecordPFlagTrimI0% - 1
4340         GOTO 4310
4350     REM END WHILE
4360     editrecordPFlag0$ = LEFT$(invFlagBuf$, editrecordPFlagTrimI0%)
4370     editrecordPDescTrimI0% = LEN(invDescBuf$)
4380     IF (editrecordPDescTrimI0% > 0) = 0 THEN GOTO 4420
4390     IF (MID$(invDescBuf$, editrecordPDescTrimI0%, 1) = " ") = 0 THEN GOTO 4420
4400         editrecordPDescTrimI0% = editrecordPDescTrimI0% - 1
4410         GOTO 4380
4420     REM END WHILE
4430     editrecordPDesc0$ = LEFT$(invDescBuf$, editrecordPDescTrimI0%)
4440     editrecordPQty0% = CVI(invQtyBuf$)
4450     editrecordPReorder0% = CVI(invReorderBuf$)
4460     editrecordPPrice0! = CVS(invPriceBuf$)
4470     isemptyFlag0$ = editrecordPFlag0$
4480     GOSUB 1750
4490     IF (isemptyResult0% = 0) = 0 THEN GOTO 4580
4500         LOCATE 12, tabcol%
4510         PRINT "Overwrite existing part data?"
4520         GOSUB 1930
4530         editrecordKp0$ = readkeyResult0$
4540         IF (editrecordKp0$ <> "Y") = 0 THEN GOTO 4570
4550         IF (editrecordKp0$ <> "y") = 0 THEN GOTO 4570
4560             RETURN
4570         REM END IF
4580     REM END IF

4590         gatherpartdetailsPartNum0% = editrecordPart0%
4600         gatherpartdetailsDesc0$ = editrecordEditDesc0$
4610         gatherpartdetailsQty0% = editrecordEditQty0%
4620         gatherpartdetailsReorder0% = editrecordEditReorder0%
4630         gatherpartdetailsPrice0! = editrecordEditPrice0!
4640         GOSUB 2940
4650         editrecordEditDesc0$ = gatherpartdetailsDesc0$
4660         editrecordEditQty0% = gatherpartdetailsQty0%
4670         editrecordEditReorder0% = gatherpartdetailsReorder0%
4680         editrecordEditPrice0! = gatherpartdetailsPrice0!
4690         GOSUB 1930
4700         editrecordKp0$ = readkeyResult0$
4710         IF (editrecordKp0$ = "Y") <> 0 THEN GOTO 4740
4720         IF (editrecordKp0$ = "y") <> 0 THEN GOTO 4740
4730         GOTO 4590
4740     REM END DO
4750     ' inv[...] = { ... }  (whole-record write)
4760     LSET invFlagBuf$ = "1"
4770     LSET invDescBuf$ = editrecordEditDesc0$
4780     LSET invQtyBuf$ = MKI$(editrecordEditQty0%)
4790     LSET invReorderBuf$ = MKI$(editrecordEditReorder0%)
4800     LSET invPriceBuf$ = MKS$(editrecordEditPrice0!)
4810     PUT #1, editrecordPart0%
4820     RETURN
4830 ' end procedure editrecord

4840 ' procedure listall()
4850     GOSUB 2670
4860     listallScrollCount0% = 0
4870     FOR listallI0% = 1 TO partcount%
4880         ' let p = inv[...]  (whole-record read)
4890         GET #1, listallI0%
4900         listallPFlagTrimI0% = LEN(invFlagBuf$)
4910         IF (listallPFlagTrimI0% > 0) = 0 THEN GOTO 4950
4920         IF (MID$(invFlagBuf$, listallPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 4950
4930             listallPFlagTrimI0% = listallPFlagTrimI0% - 1
4940             GOTO 4910
4950         REM END WHILE
4960         listallPFlag0$ = LEFT$(invFlagBuf$, listallPFlagTrimI0%)
4970         listallPDescTrimI0% = LEN(invDescBuf$)
4980         IF (listallPDescTrimI0% > 0) = 0 THEN GOTO 5020
4990         IF (MID$(invDescBuf$, listallPDescTrimI0%, 1) = " ") = 0 THEN GOTO 5020
5000             listallPDescTrimI0% = listallPDescTrimI0% - 1
5010             GOTO 4980
5020         REM END WHILE
5030         listallPDesc0$ = LEFT$(invDescBuf$, listallPDescTrimI0%)
5040         listallPQty0% = CVI(invQtyBuf$)
5050         listallPReorder0% = CVI(invReorderBuf$)
5060         listallPPrice0! = CVS(invPriceBuf$)
5070         printinventorylinePartNum0% = listallI0%
5080         printinventorylineDesc0$ = listallPDesc0$
5090         printinventorylineQty0% = listallPQty0%
5100         printinventorylineReorder0% = listallPReorder0%
5110         GOSUB 2760
5120         listallScrollCount0% = listallScrollCount0% + 1
5130         IF (listallScrollCount0% = 20) = 0 THEN GOTO 5160
5140             GOSUB 2000
5150             listallScrollCount0% = 0
5160         REM END IF
5170     NEXT listallI0%
5180     RETURN
5190 ' end procedure listall

5200 ' procedure addstock()
5210     CLS
5220     LOCATE 5, 25
5230     PRINT "A D D I N G   S T O C K"

5240         LOCATE 8, 25
5250         GOSUB 1880
5260         addstockPartStr0$ = readpartnumberinputResult0$
5270         addstockPart0% = VAL(addstockPartStr0$)
5280         partinrangeN0% = addstockPart0%
5290         GOSUB 1790
5300         addstockValidPart0% = partinrangeResult0%
5310         IF (addstockValidPart0% = 0) = 0 THEN GOTO 5340
5320             GOSUB 2400
5330             GOSUB 1930
5340         REM END IF
5350         IF (addstockValidPart0% <> 0) = 0 THEN GOTO 5240
5360     REM END DO

5370     ' let p = inv[...]  (whole-record read)
5380     GET #1, addstockPart0%
5390     addstockPFlagTrimI0% = LEN(invFlagBuf$)
5400     IF (addstockPFlagTrimI0% > 0) = 0 THEN GOTO 5440
5410     IF (MID$(invFlagBuf$, addstockPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 5440
5420         addstockPFlagTrimI0% = addstockPFlagTrimI0% - 1
5430         GOTO 5400
5440     REM END WHILE
5450     addstockPFlag0$ = LEFT$(invFlagBuf$, addstockPFlagTrimI0%)
5460     addstockPDescTrimI0% = LEN(invDescBuf$)
5470     IF (addstockPDescTrimI0% > 0) = 0 THEN GOTO 5510
5480     IF (MID$(invDescBuf$, addstockPDescTrimI0%, 1) = " ") = 0 THEN GOTO 5510
5490         addstockPDescTrimI0% = addstockPDescTrimI0% - 1
5500         GOTO 5470
5510     REM END WHILE
5520     addstockPDesc0$ = LEFT$(invDescBuf$, addstockPDescTrimI0%)
5530     addstockPQty0% = CVI(invQtyBuf$)
5540     addstockPReorder0% = CVI(invReorderBuf$)
5550     addstockPPrice0! = CVS(invPriceBuf$)
5560     isemptyFlag0$ = addstockPFlag0$
5570     GOSUB 1750
5580     IF (isemptyResult0%) = 0 THEN GOTO 5630
5590         shownullentrymessagePartStr0$ = addstockPartStr0$
5600         GOSUB 2470
5610         GOSUB 1930
5620         RETURN
5630     REM END IF

5640         showaddstockscreenPartNum0% = addstockPart0%
5650         showaddstockscreenDesc0$ = addstockPDesc0$
5660         showaddstockscreenQty0% = addstockPQty0%
5670         showaddstockscreenReorder0% = addstockPReorder0%
5680         GOSUB 3140
5690         LOCATE 14, tabcol%
5700         INPUT " Quantity to add"; addstockAddStr0$
5710         addstockAddAmt0% = VAL(addstockAddStr0$)
5720         IF (addstockAddAmt0% < 0) = 0 THEN GOTO 5750
5730             GOSUB 3300
5740             GOSUB 1930
5750         REM END IF
5760         IF (addstockAddAmt0% >= 0) = 0 THEN GOTO 5640
5770     REM END DO

5780     addstockPQty0% = addstockPQty0% + addstockAddAmt0%
5790     ' inv[...] = p  (write back a let-bound record)
5800     LSET invFlagBuf$ = addstockPFlag0$
5810     LSET invDescBuf$ = addstockPDesc0$
5820     LSET invQtyBuf$ = MKI$(addstockPQty0%)
5830     LSET invReorderBuf$ = MKI$(addstockPReorder0%)
5840     LSET invPriceBuf$ = MKS$(addstockPPrice0!)
5850     PUT #1, addstockPart0%
5860     RETURN
5870 ' end procedure addstock

5880 ' procedure subtractstock()
5890     CLS
5900     LOCATE 5, 20
5910     PRINT "S U B T R A C T I N G    S T O C K"

5920         LOCATE 8, 25
5930         GOSUB 1880
5940         subtractstockPartStr0$ = readpartnumberinputResult0$
5950         subtractstockPart0% = VAL(subtractstockPartStr0$)
5960         partinrangeN0% = subtractstockPart0%
5970         GOSUB 1790
5980         subtractstockValidPart0% = partinrangeResult0%
5990         IF (subtractstockValidPart0% = 0) = 0 THEN GOTO 6020
6000             GOSUB 2400
6010             GOSUB 1930
6020         REM END IF
6030         IF (subtractstockValidPart0% <> 0) = 0 THEN GOTO 5920
6040     REM END DO

6050     ' let p = inv[...]  (whole-record read)
6060     GET #1, subtractstockPart0%
6070     subtractstockPFlagTrimI0% = LEN(invFlagBuf$)
6080     IF (subtractstockPFlagTrimI0% > 0) = 0 THEN GOTO 6120
6090     IF (MID$(invFlagBuf$, subtractstockPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 6120
6100         subtractstockPFlagTrimI0% = subtractstockPFlagTrimI0% - 1
6110         GOTO 6080
6120     REM END WHILE
6130     subtractstockPFlag0$ = LEFT$(invFlagBuf$, subtractstockPFlagTrimI0%)
6140     subtractstockPDescTrimI0% = LEN(invDescBuf$)
6150     IF (subtractstockPDescTrimI0% > 0) = 0 THEN GOTO 6190
6160     IF (MID$(invDescBuf$, subtractstockPDescTrimI0%, 1) = " ") = 0 THEN GOTO 6190
6170         subtractstockPDescTrimI0% = subtractstockPDescTrimI0% - 1
6180         GOTO 6150
6190     REM END WHILE
6200     subtractstockPDesc0$ = LEFT$(invDescBuf$, subtractstockPDescTrimI0%)
6210     subtractstockPQty0% = CVI(invQtyBuf$)
6220     subtractstockPReorder0% = CVI(invReorderBuf$)
6230     subtractstockPPrice0! = CVS(invPriceBuf$)
6240     isemptyFlag0$ = subtractstockPFlag0$
6250     GOSUB 1750
6260     IF (isemptyResult0%) = 0 THEN GOTO 6310
6270         shownullentrymessagePartStr0$ = subtractstockPartStr0$
6280         GOSUB 2470
6290         GOSUB 1930
6300         RETURN
6310     REM END IF

6320         showsubtractstockscreenPartNum0% = subtractstockPart0%
6330         showsubtractstockscreenDesc0$ = subtractstockPDesc0$
6340         showsubtractstockscreenQty0% = subtractstockPQty0%
6350         showsubtractstockscreenReorder0% = subtractstockPReorder0%
6360         GOSUB 3370
6370         LOCATE 14, tabcol%
6380         INPUT "Quantity to subtract"; subtractstockSubStr0$
6390         subtractstockSubAmt0% = VAL(subtractstockSubStr0$)
6400         subtractstockOverSubtract0% = 0
6410         IF (subtractstockSubAmt0% >= 0) = 0 THEN GOTO 6470
6420         IF ((subtractstockPQty0% - subtractstockSubAmt0%) < 0) = 0 THEN GOTO 6470
6430             subtractstockOverSubtract0% = 1
6440             showoversubtractwarningOnHand0% = subtractstockPQty0%
6450             GOSUB 3530
6460             GOSUB 1930
6470         REM END IF
6480         IF (subtractstockSubAmt0% >= 0) = 0 THEN GOTO 6320
6490         IF (subtractstockOverSubtract0% = 0) = 0 THEN GOTO 6320
6500     REM END DO

6510     subtractstockPQty0% = subtractstockPQty0% - subtractstockSubAmt0%
6520     IF (subtractstockPQty0% <= subtractstockPReorder0%) = 0 THEN GOTO 6540
6530         LOCATE 16, tabcol%
6540     REM END IF
6550     PRINT (("quantity now" + STR$(subtractstockPQty0%)) + " reorder level") + STR$(subtractstockPReorder0%)
6560     ' inv[...] = p  (write back a let-bound record)
6570     LSET invFlagBuf$ = subtractstockPFlag0$
6580     LSET invDescBuf$ = subtractstockPDesc0$
6590     LSET invQtyBuf$ = MKI$(subtractstockPQty0%)
6600     LSET invReorderBuf$ = MKI$(subtractstockPReorder0%)
6610     LSET invPriceBuf$ = MKS$(subtractstockPPrice0!)
6620     PUT #1, subtractstockPart0%
6630     RETURN
6640 ' end procedure subtractstock

6650 ' procedure reorderreport()
6660     GOSUB 2800
6670     reorderreportReportLineCount0% = 0
6680     FOR reorderreportI0% = 1 TO partcount%
6690         ' let p = inv[...]  (whole-record read)
6700         GET #1, reorderreportI0%
6710         reorderreportPFlagTrimI0% = LEN(invFlagBuf$)
6720         IF (reorderreportPFlagTrimI0% > 0) = 0 THEN GOTO 6760
6730         IF (MID$(invFlagBuf$, reorderreportPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 6760
6740             reorderreportPFlagTrimI0% = reorderreportPFlagTrimI0% - 1
6750             GOTO 6720
6760         REM END WHILE
6770         reorderreportPFlag0$ = LEFT$(invFlagBuf$, reorderreportPFlagTrimI0%)
6780         reorderreportPDescTrimI0% = LEN(invDescBuf$)
6790         IF (reorderreportPDescTrimI0% > 0) = 0 THEN GOTO 6830
6800         IF (MID$(invDescBuf$, reorderreportPDescTrimI0%, 1) = " ") = 0 THEN GOTO 6830
6810             reorderreportPDescTrimI0% = reorderreportPDescTrimI0% - 1
6820             GOTO 6790
6830         REM END WHILE
6840         reorderreportPDesc0$ = LEFT$(invDescBuf$, reorderreportPDescTrimI0%)
6850         reorderreportPQty0% = CVI(invQtyBuf$)
6860         reorderreportPReorder0% = CVI(invReorderBuf$)
6870         reorderreportPPrice0! = CVS(invPriceBuf$)
6880         IF (reorderreportPQty0% < reorderreportPReorder0%) = 0 THEN GOTO 6990
6890             printreorderlinePartNum0% = reorderreportI0%
6900             printreorderlineDesc0$ = reorderreportPDesc0$
6910             printreorderlineQty0% = reorderreportPQty0%
6920             printreorderlineReorder0% = reorderreportPReorder0%
6930             GOSUB 2900
6940             reorderreportReportLineCount0% = reorderreportReportLineCount0% + 1
6950             IF (reorderreportReportLineCount0% > 15) = 0 THEN GOTO 6980
6960                 GOSUB 2000
6970                 reorderreportReportLineCount0% = 0
6980             REM END IF
6990         REM END IF
7000     NEXT reorderreportI0%
7010     GOSUB 2000
7020     RETURN
7030 ' end procedure reorderreport
