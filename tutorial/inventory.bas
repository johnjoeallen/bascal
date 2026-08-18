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
410 ' Verified against real BASCOM 2.00 under dosbox-x: compiles
420 ' clean and links, but -- because this program uses `on error
430 ' goto` / `resume` -- only when BASCOM is invoked with the /E and
440 ' /X switches (error trapping isn't linked in by default). See
450 ' `errorTrap:` at the bottom.
460 ' ============================================================

470 ' BASCAL-ism: the record/file DSL. `record ... end record` plus
480 ' `file ... as ... = open(...)` below replace fhb's manual
490 ' FIELD #1,1 AS F$,30 AS D$,2 AS Q$,... buffer layout entirely --
500 ' bcc computes the field widths and record LEN from this
510 ' declaration and generates the FIELD statement itself. Named
520 ' field access (`p.flag`, `p.qty`, ...) and whole-record
530 ' read/write via `inv[n]` (see checkPart() below) replace fhb's
540 ' manual GET/PUT plus LSET/RSET and MKI$/MKS$/CVI$/CVS$ packing.

550 ' BASCAL-ism: `const` is a real compile-time constant, not a plain
560 ' variable assignment like fhb's `N=100` / `T=20` -- it can never
570 ' be reassigned, and resolves to the same value everywhere,
580 ' including inside every function/procedure below, with no
590 ' `global` declaration needed.
600 partcount% = 100
610 tabcol% = 20

620 ' `file ... = open(...)` is sugar for OPEN ... FOR RANDOM AS #n
630 ' LEN = <record width> plus the FIELD statement fhb wrote out by
640 ' hand at his line 550.
650 ' file inv as Part = open(...)  [39 bytes/record]
660 OPEN "inven.dat" FOR RANDOM AS #1 LEN = 39
670 FIELD #1, 1 AS invFlagBuf$, 30 AS invDescBuf$, 2 AS invQtyBuf$, 2 AS invReorderBuf$, 4 AS invPriceBuf$

680 ' -------------------- Pure functions (no file access) --------------------

690 ' BASCAL-ism: `function ... end function` with `return` replaces
700 ' fhb's convention of a GOSUB target plus a bare RETURN -- there's
710 ' no separate "subroutine label" and no shared/global result
720 ' variable to manage by hand; `isEmpty%(...)` is called like an
730 ' ordinary expression at every use below (e.g. `isEmpty%(p.flag)`).
740 ' A record whose flag byte is CHR$(255) is an empty/never-used slot.

750 ' BASCAL-ism: `&&` and `||` are short-circuit AND/OR -- real
760 ' MBASIC/BASCOM only has bitwise AND/OR (which fhb relies on here
770 ' too, since `PART!<1 OR PART!>N!` never short-circuits anyway).
780 ' BASCAL lowers `&&`/`||` into the equivalent branching so the
790 ' short-circuit *is* real at the generated-BASIC level; see
800 ' MANUAL.md's "Short-Circuit && and ||" section.

810 ' -------------------- Keyboard input --------------------

820 ' BASCAL-ism: `do ... loop until` is a structured post-check loop
830 ' replacing fhb's `730 KP$=INKEY$:IF KP$="" THEN 730` GOTO-polling
840 ' idiom. `inkey$` itself is the real INKEY$ builtin passed straight
850 ' through, resolving correctly from inside a function/procedure
860 ' body like this one -- every menu action below calls
870 ' readKey$()/waitAnyKey() rather than polling INKEY$ inline.

880 ' -------------------- Display procedures --------------------

890 ' byref scalar parameters: gatherPartDetails writes the four editable
900 ' fields for a part directly back into the caller's variables.

910 ' -------------------- Menu actions --------------------

920 ' -------------------- Program entry --------------------

930 CLS
940 ON ERROR GOTO 1450

950     GOSUB 1900
960     GOSUB 1750
970     kp$ = readkeyResult0$
980     IF (INSTR("12345678cCeElLaAsSrRqQxX", kp$) <> 0) = 0 THEN GOTO 1330
990         ' BASCAL-ism: `select case` replaces fhb's chain of eight
1000         ' `IF VAL(KP$)=n OR KP$="x" OR KP$="X" THEN GOTO ...` lines
1010         ' (his 770-840) with one multi-way dispatch.
1020         BCCT4$ = kp$
1030         IF (BCCT4$ = "1" OR BCCT4$ = "c" OR BCCT4$ = "C") <> 0 THEN GOTO 1120
1040         IF (BCCT4$ = "2" OR BCCT4$ = "e" OR BCCT4$ = "E") <> 0 THEN GOTO 1140
1050         IF (BCCT4$ = "3" OR BCCT4$ = "l" OR BCCT4$ = "L") <> 0 THEN GOTO 1160
1060         IF (BCCT4$ = "4" OR BCCT4$ = "a" OR BCCT4$ = "A") <> 0 THEN GOTO 1180
1070         IF (BCCT4$ = "5" OR BCCT4$ = "s" OR BCCT4$ = "S") <> 0 THEN GOTO 1200
1080         IF (BCCT4$ = "6" OR BCCT4$ = "r" OR BCCT4$ = "R") <> 0 THEN GOTO 1220
1090         IF (BCCT4$ = "7" OR BCCT4$ = "q" OR BCCT4$ = "Q") <> 0 THEN GOTO 1240
1100         IF (BCCT4$ = "8" OR BCCT4$ = "x" OR BCCT4$ = "X") <> 0 THEN GOTO 1260
1110         GOTO 1320
1120             GOSUB 3440
1130             GOTO 1320
1140             GOSUB 3980
1150             GOTO 1320
1160             GOSUB 4670
1170             GOTO 1320
1180             GOSUB 5030
1190             GOTO 1320
1200             GOSUB 5710
1210             GOTO 1320
1220             GOSUB 6480
1230             GOTO 1320
1240             quitflag% = 1
1250             GOTO 1320
1260             ' BASCAL-ism: `inv.close()` is sugar for `CLOSE #1`,
1270             ' matching fhb's own `90 CLOSE:SYSTEM`.
1280             ' inv.close()
1290             CLOSE #1
1300             SYSTEM
1310             GOTO 1320
1320         REM END SELECT
1330     REM END IF
1340     IF (quitflag% = 1) = 0 THEN GOTO 950
1350 REM END DO

1360 ' inv.close()
1370 CLOSE #1
1380 END

1390 ' -------------------- Error handling --------------------
1400 ' Still a raw label, not a procedure: ON ERROR GOTO's target and RESUME
1410 ' both need to act on the main program's execution state, not a nested
1420 ' GOSUB/function-call frame -- that's a language-design reason, unrelated
1430 ' to either compiler bug, so it stays a label regardless of the fixes
1440 ' above.
1450 LOCATE 25, 1
1460 ' `err` and `erl` are real numeric system pseudo-variables, passed
1470 ' straight through like fhb's own ERR/ERL (his 3390: "an error on
1480 ' line";ERL). Unlike fhb's version, this doesn't decode ERR into a
1490 ' message per the header note above -- it just reports the raw
1500 ' code. See MANUAL.md's ON ERROR GOTO section.
1510 PRINT (("There has been an error on line" + STR$(ERL)) + "  Error #") + STR$(ERR)
1520 GOSUB 1750
1530 k$ = readkeyResult0$
1540 RESUME NEXT
1550 END

1560 ' function isempty%(flag$)
1570     isemptyResult0% = ASC(isemptyFlag0$) = 255
1580     RETURN
1590 ' end function isempty%

1600 ' function partinrange%(n%)
1610     IF (partinrangeN0% >= 1) = 0 THEN GOTO 1650
1620     IF (partinrangeN0% <= partcount%) = 0 THEN GOTO 1650
1630         partinrangeResult0% = 1
1640         RETURN
1650     REM END IF
1660     partinrangeResult0% = 0
1670     RETURN
1680 ' end function partinrange%

1690 ' function readpartnumberinput$()
1700     INPUT "Input part number"; readpartnumberinputS0$
1710     readpartnumberinputResult0$ = readpartnumberinputS0$
1720     RETURN
1730 ' end function readpartnumberinput$

1740 ' function readkey$()
1750         readkeyK0$ = INKEY$
1760         IF (readkeyK0$ <> "") = 0 THEN GOTO 1750
1770     REM END DO
1780     readkeyResult0$ = readkeyK0$
1790     RETURN
1800 ' end function readkey$

1810 ' procedure waitanykey()
1820     LOCATE 25, 10
1830     PRINT "Press the AnyKey to continue...";
1840         waitanykeyK0$ = INKEY$
1850         IF (waitanykeyK0$ <> "") = 0 THEN GOTO 1840
1860     REM END DO
1870     RETURN
1880 ' end procedure waitanykey

1890 ' procedure showmainmenu()
1900     CLS
1910     COLOR 14, 4
1920     CLS
1930     LOCATE 6, 1
1940     PRINT
1950     ' `tab(n)` passes straight through to real TAB(n), same as
1960     ' fhb's own `PRINT TAB(V) "..."` -- but only as a bare item in
1970     ' a PRINT list, juxtaposed or `;`-separated like here. Real
1980     ' BASCOM rejects `"literal" + tab(n) + ...` (TAB isn't a real
1990     ' string function you can concatenate); see printListHeader()
2000     ' and printReorderHeader() below, which need `;` between a
2010     ' preceding string and a `tab(n)` for exactly this reason.
2020     PRINT TAB(30)"Inventory Program"
2030     PRINT
2040     PRINT TAB(tabcol%)"1......C)heck a part"
2050     PRINT TAB(tabcol%)"2......E)dit/overwrite/add a part"
2060     PRINT TAB(tabcol%)("3......L)ist all" + STR$(partcount%)) + "parts"
2070     PRINT TAB(tabcol%)"4......A)dd stock"
2080     PRINT TAB(tabcol%)"5......S)ubtract stock"
2090     PRINT TAB(tabcol%)"6......R)eorder Report"
2100     PRINT
2110     PRINT TAB(tabcol%)"7......Q)uit to BASIC"
2120     PRINT TAB(tabcol%)"8......eX)it to system"
2130     RETURN
2140 ' end procedure showmainmenu

2150 ' procedure showbadpartnumber()
2160     CLS
2170     LOCATE 10, 10
2180     PRINT "Part number is out of permissable range of 1 to" + STR$(partcount%)
2190     RETURN
2200 ' end procedure showbadpartnumber

2210 ' procedure showrangeretrymessage()
2220     LOCATE 10, 15
2230     PRINT "The Part number is out of permissable range of 1 to" + STR$(partcount%)
2240     LOCATE 25, 15
2250     PRINT "Press the Anykey to reenter part number...";
2260     RETURN
2270 ' end procedure showrangeretrymessage

2280 ' procedure shownullentrymessage(partstr$)
2290     LOCATE 10, tabcol%
2300     PRINT ("Part number " + shownullentrymessagePartStr0$) + " is a null entry"
2310     RETURN
2320 ' end procedure shownullentrymessage

2330 ' procedure showpartstatus(partnum%, desc$, qty%, reorder%, price!)
2340     CLS
2350     LOCATE 5, 1
2360     PRINT TAB(tabcol%)"Inventory Status for Individual Part Number"
2370     PRINT TAB(tabcol%)"==========================================="
2380     PRINT
2390     PRINT
2400     PRINT TAB(tabcol%)"     Part number:  " + STR$(showpartstatusPartNum0%)
2410     PRINT
2420     PRINT TAB(tabcol%)"       Item name:  " + showpartstatusDesc0$
2430     PRINT TAB(tabcol%)"Quantity on hand:  " + STR$(showpartstatusQty0%)
2440     PRINT TAB(tabcol%)"   Reorder level:  " + STR$(showpartstatusReorder0%)
2450     PRINT TAB(tabcol%)"      Unit price:  " + STR$(showpartstatusPrice0!)
2460     RETURN
2470 ' end procedure showpartstatus

2480 ' procedure printlistheader()
2490     CLS
2500     PRINT TAB(25)"I N V E N T O R Y   L I S T I N G"; TAB(65); STR$(partcount%) + "items"
2510     PRINT "                                          Quantity       Reorder"
2520     PRINT " Partno           Description             on hand         level"
2530     LOCATE 25, 1
2540     PRINT "Press the AnyKey to scroll listing...";
2550     RETURN
2560 ' end procedure printlistheader

2570 ' procedure printinventoryline(partnum%, desc$, qty%, reorder%)
2580     PRINT (((((STR$(printinventorylinePartNum0%) + "  ") + printinventorylineDesc0$) + "   ") + STR$(printinventorylineQty0%)) + "          ") + STR$(printinventorylineReorder0%)
2590     RETURN
2600 ' end procedure printinventoryline

2610 ' procedure printreorderheader()
2620     CLS
2630     LOCATE 1, tabcol%
2640     PRINT "Reorder Report"; TAB(55); DATE$
2650     PRINT
2660     PRINT "                                             Quantity       Reorder"
2670     PRINT "    Partno           Description             on hand         level"
2680     PRINT "   =======  ==============================   ========       ======="
2690     RETURN
2700 ' end procedure printreorderheader

2710 ' procedure printreorderline(partnum%, desc$, qty%, reorder%)
2720     PRINT (((((("  " + STR$(printreorderlinePartNum0%)) + "  ") + printreorderlineDesc0$) + "   ") + STR$(printreorderlineQty0%)) + "          ") + STR$(printreorderlineReorder0%)
2730     RETURN
2740 ' end procedure printreorderline

2750 ' procedure gatherpartdetails(partnum%, desc$, qty%, reorder%, price!)
2760     CLS
2770     LOCATE 4, tabcol%
2780     PRINT "Adding or Overwriting a Record"
2790     LOCATE 8, tabcol%
2800     PRINT "Record/Partno" + STR$(gatherpartdetailsPartNum0%)
2810     LOCATE 11, 39
2820     PRINT "------------------------------"
2830     LOCATE 10, tabcol%
2840     INPUT "      Description"; gatherpartdetailsDesc0$
2850     LOCATE 12, tabcol%
2860     INPUT "Quantity in stock"; gatherpartdetailsQty0%
2870     LOCATE 14, tabcol%
2880     INPUT "    Reorder level"; gatherpartdetailsReorder0%
2890     LOCATE 16, tabcol%
2900     INPUT "       Unit price"; gatherpartdetailsPrice0!
2910     LOCATE 18, tabcol%
2920     PRINT "Is information correct (Y/N)?"
2930     RETURN
2940 ' end procedure gatherpartdetails

2950 ' procedure showaddstockscreen(partnum%, desc$, qty%, reorder%)
2960     CLS
2970     LOCATE 4, 25
2980     PRINT "Add to an inventory part number"
2990     LOCATE 5, 25
3000     PRINT "==============================="
3010     LOCATE 8, tabcol%
3020     PRINT "     Part number: " + STR$(showaddstockscreenPartNum0%)
3030     LOCATE 9, tabcol%
3040     PRINT "Item description: " + showaddstockscreenDesc0$
3050     LOCATE 10, tabcol%
3060     PRINT "Quantity on hand: " + STR$(showaddstockscreenQty0%)
3070     LOCATE 11, tabcol%
3080     PRINT "   Reorder Level: " + STR$(showaddstockscreenReorder0%)
3090     RETURN
3100 ' end procedure showaddstockscreen

3110 ' procedure shownegativeqtywarning()
3120     LOCATE 17, 15
3130     PRINT "The quantity to add must NOT be a negative number"
3140     LOCATE 25, 1
3150     PRINT "Please press the Anykey to reenter quantity to add...";
3160     RETURN
3170 ' end procedure shownegativeqtywarning

3180 ' procedure showsubtractstockscreen(partnum%, desc$, qty%, reorder%)
3190     CLS
3200     LOCATE 4, tabcol%
3210     PRINT "Subtract an inventory part number"
3220     LOCATE 5, tabcol%
3230     PRINT "================================="
3240     LOCATE 8, tabcol%
3250     PRINT "         Part number: " + STR$(showsubtractstockscreenPartNum0%)
3260     LOCATE 9, tabcol%
3270     PRINT "    Item description: " + showsubtractstockscreenDesc0$
3280     LOCATE 10, tabcol%
3290     PRINT "    Quantity on hand: " + STR$(showsubtractstockscreenQty0%)
3300     LOCATE 11, tabcol%
3310     PRINT "       Reorder Level: " + STR$(showsubtractstockscreenReorder0%)
3320     RETURN
3330 ' end procedure showsubtractstockscreen

3340 ' procedure showoversubtractwarning(onhand%)
3350     LOCATE 17, 5
3360     PRINT "The quantity to SUBTRACT must NOT result in NEGATIVE inventory"
3370     LOCATE 18, 5
3380     PRINT ("Only" + STR$(showoversubtractwarningOnHand0%)) + " IN STOCK"
3390     LOCATE 25, 1
3400     PRINT "Please press the Anykey to reenter quantity to subtract...";
3410     RETURN
3420 ' end procedure showoversubtractwarning

3430 ' procedure checkpart()
3440     GOSUB 1700
3450     checkpartPartStr0$ = readpartnumberinputResult0$
3460     checkpartPart0% = VAL(checkpartPartStr0$)
3470     partinrangeN0% = checkpartPart0%
3480     GOSUB 1610
3490     IF (partinrangeResult0% = 0) = 0 THEN GOTO 3530
3500         GOSUB 2160
3510         GOSUB 1820
3520         RETURN
3530     REM END IF
3540     ' BASCAL-ism: `let p = inv[part%]` reads record `part%` of the
3550     ' `inv` file into a local record variable `p` -- one expression
3560     ' for what fhb's `GET #1, PART!` plus five separate field reads
3570     ' (F$, D$, CVI(Q$), CVI(R$), CVS(P$)) did by hand. The write
3580     ' side, `inv[part%] = { ... }` (see editRecord() below), is the
3590     ' same sugar for PUT plus the LSET/MKx$ packing it replaces.
3600     ' let p = inv[...]  (whole-record read)
3610     GET #1, checkpartPart0%
3620     checkpartPFlagTrimI0% = LEN(invFlagBuf$)
3630     IF (checkpartPFlagTrimI0% > 0) = 0 THEN GOTO 3670
3640     IF (MID$(invFlagBuf$, checkpartPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 3670
3650         checkpartPFlagTrimI0% = checkpartPFlagTrimI0% - 1
3660         GOTO 3630
3670     REM END WHILE
3680     checkpartPFlag0$ = LEFT$(invFlagBuf$, checkpartPFlagTrimI0%)
3690     checkpartPDescTrimI0% = LEN(invDescBuf$)
3700     IF (checkpartPDescTrimI0% > 0) = 0 THEN GOTO 3740
3710     IF (MID$(invDescBuf$, checkpartPDescTrimI0%, 1) = " ") = 0 THEN GOTO 3740
3720         checkpartPDescTrimI0% = checkpartPDescTrimI0% - 1
3730         GOTO 3700
3740     REM END WHILE
3750     checkpartPDesc0$ = LEFT$(invDescBuf$, checkpartPDescTrimI0%)
3760     checkpartPQty0% = CVI(invQtyBuf$)
3770     checkpartPReorder0% = CVI(invReorderBuf$)
3780     checkpartPPrice0! = CVS(invPriceBuf$)
3790     isemptyFlag0$ = checkpartPFlag0$
3800     GOSUB 1570
3810     IF (isemptyResult0%) = 0 THEN GOTO 3870
3820         CLS
3830         LOCATE 10, 18
3840         PRINT ("Part number" + STR$(checkpartPart0%)) + "is still a null entry at this time"
3850         GOSUB 1820
3860         RETURN
3870     REM END IF
3880     showpartstatusPartNum0% = checkpartPart0%
3890     showpartstatusDesc0$ = checkpartPDesc0$
3900     showpartstatusQty0% = checkpartPQty0%
3910     showpartstatusReorder0% = checkpartPReorder0%
3920     showpartstatusPrice0! = checkpartPPrice0!
3930     GOSUB 2340
3940     GOSUB 1820
3950     RETURN
3960 ' end procedure checkpart

3970 ' procedure editrecord()
3980     CLS
3990     LOCATE 10, tabcol%
4000     GOSUB 1700
4010     editrecordPartStr0$ = readpartnumberinputResult0$
4020     editrecordPart0% = VAL(editrecordPartStr0$)
4030     partinrangeN0% = editrecordPart0%
4040     GOSUB 1610
4050     IF (partinrangeResult0% = 0) = 0 THEN GOTO 4090
4060         GOSUB 2160
4070         GOSUB 1820
4080         RETURN
4090     REM END IF
4100     ' let p = inv[...]  (whole-record read)
4110     GET #1, editrecordPart0%
4120     editrecordPFlagTrimI0% = LEN(invFlagBuf$)
4130     IF (editrecordPFlagTrimI0% > 0) = 0 THEN GOTO 4170
4140     IF (MID$(invFlagBuf$, editrecordPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 4170
4150         editrecordPFlagTrimI0% = editrecordPFlagTrimI0% - 1
4160         GOTO 4130
4170     REM END WHILE
4180     editrecordPFlag0$ = LEFT$(invFlagBuf$, editrecordPFlagTrimI0%)
4190     editrecordPDescTrimI0% = LEN(invDescBuf$)
4200     IF (editrecordPDescTrimI0% > 0) = 0 THEN GOTO 4240
4210     IF (MID$(invDescBuf$, editrecordPDescTrimI0%, 1) = " ") = 0 THEN GOTO 4240
4220         editrecordPDescTrimI0% = editrecordPDescTrimI0% - 1
4230         GOTO 4200
4240     REM END WHILE
4250     editrecordPDesc0$ = LEFT$(invDescBuf$, editrecordPDescTrimI0%)
4260     editrecordPQty0% = CVI(invQtyBuf$)
4270     editrecordPReorder0% = CVI(invReorderBuf$)
4280     editrecordPPrice0! = CVS(invPriceBuf$)
4290     isemptyFlag0$ = editrecordPFlag0$
4300     GOSUB 1570
4310     IF (isemptyResult0% = 0) = 0 THEN GOTO 4400
4320         LOCATE 12, tabcol%
4330         PRINT "Overwrite existing part data?"
4340         GOSUB 1750
4350         editrecordKp0$ = readkeyResult0$
4360         IF (editrecordKp0$ <> "Y") = 0 THEN GOTO 4390
4370         IF (editrecordKp0$ <> "y") = 0 THEN GOTO 4390
4380             RETURN
4390         REM END IF
4400     REM END IF

4410         gatherpartdetailsPartNum0% = editrecordPart0%
4420         gatherpartdetailsDesc0$ = editrecordEditDesc0$
4430         gatherpartdetailsQty0% = editrecordEditQty0%
4440         gatherpartdetailsReorder0% = editrecordEditReorder0%
4450         gatherpartdetailsPrice0! = editrecordEditPrice0!
4460         GOSUB 2760
4470         editrecordEditDesc0$ = gatherpartdetailsDesc0$
4480         editrecordEditQty0% = gatherpartdetailsQty0%
4490         editrecordEditReorder0% = gatherpartdetailsReorder0%
4500         editrecordEditPrice0! = gatherpartdetailsPrice0!
4510         GOSUB 1750
4520         editrecordKp0$ = readkeyResult0$
4530         IF (editrecordKp0$ = "Y") <> 0 THEN GOTO 4560
4540         IF (editrecordKp0$ = "y") <> 0 THEN GOTO 4560
4550         GOTO 4410
4560     REM END DO
4570     ' inv[...] = { ... }  (whole-record write)
4580     LSET invFlagBuf$ = "1"
4590     LSET invDescBuf$ = editrecordEditDesc0$
4600     LSET invQtyBuf$ = MKI$(editrecordEditQty0%)
4610     LSET invReorderBuf$ = MKI$(editrecordEditReorder0%)
4620     LSET invPriceBuf$ = MKS$(editrecordEditPrice0!)
4630     PUT #1, editrecordPart0%
4640     RETURN
4650 ' end procedure editrecord

4660 ' procedure listall()
4670     GOSUB 2490
4680     listallScrollCount0% = 0
4690     FOR listallI0% = 1 TO partcount%
4700         ' let p = inv[...]  (whole-record read)
4710         GET #1, listallI0%
4720         listallPFlagTrimI0% = LEN(invFlagBuf$)
4730         IF (listallPFlagTrimI0% > 0) = 0 THEN GOTO 4770
4740         IF (MID$(invFlagBuf$, listallPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 4770
4750             listallPFlagTrimI0% = listallPFlagTrimI0% - 1
4760             GOTO 4730
4770         REM END WHILE
4780         listallPFlag0$ = LEFT$(invFlagBuf$, listallPFlagTrimI0%)
4790         listallPDescTrimI0% = LEN(invDescBuf$)
4800         IF (listallPDescTrimI0% > 0) = 0 THEN GOTO 4840
4810         IF (MID$(invDescBuf$, listallPDescTrimI0%, 1) = " ") = 0 THEN GOTO 4840
4820             listallPDescTrimI0% = listallPDescTrimI0% - 1
4830             GOTO 4800
4840         REM END WHILE
4850         listallPDesc0$ = LEFT$(invDescBuf$, listallPDescTrimI0%)
4860         listallPQty0% = CVI(invQtyBuf$)
4870         listallPReorder0% = CVI(invReorderBuf$)
4880         listallPPrice0! = CVS(invPriceBuf$)
4890         printinventorylinePartNum0% = listallI0%
4900         printinventorylineDesc0$ = listallPDesc0$
4910         printinventorylineQty0% = listallPQty0%
4920         printinventorylineReorder0% = listallPReorder0%
4930         GOSUB 2580
4940         listallScrollCount0% = listallScrollCount0% + 1
4950         IF (listallScrollCount0% = 20) = 0 THEN GOTO 4980
4960             GOSUB 1820
4970             listallScrollCount0% = 0
4980         REM END IF
4990     NEXT listallI0%
5000     RETURN
5010 ' end procedure listall

5020 ' procedure addstock()
5030     CLS
5040     LOCATE 5, 25
5050     PRINT "A D D I N G   S T O C K"

5060         LOCATE 8, 25
5070         GOSUB 1700
5080         addstockPartStr0$ = readpartnumberinputResult0$
5090         addstockPart0% = VAL(addstockPartStr0$)
5100         partinrangeN0% = addstockPart0%
5110         GOSUB 1610
5120         addstockValidPart0% = partinrangeResult0%
5130         IF (addstockValidPart0% = 0) = 0 THEN GOTO 5160
5140             GOSUB 2220
5150             GOSUB 1750
5160         REM END IF
5170         IF (addstockValidPart0% <> 0) = 0 THEN GOTO 5060
5180     REM END DO

5190     ' let p = inv[...]  (whole-record read)
5200     GET #1, addstockPart0%
5210     addstockPFlagTrimI0% = LEN(invFlagBuf$)
5220     IF (addstockPFlagTrimI0% > 0) = 0 THEN GOTO 5260
5230     IF (MID$(invFlagBuf$, addstockPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 5260
5240         addstockPFlagTrimI0% = addstockPFlagTrimI0% - 1
5250         GOTO 5220
5260     REM END WHILE
5270     addstockPFlag0$ = LEFT$(invFlagBuf$, addstockPFlagTrimI0%)
5280     addstockPDescTrimI0% = LEN(invDescBuf$)
5290     IF (addstockPDescTrimI0% > 0) = 0 THEN GOTO 5330
5300     IF (MID$(invDescBuf$, addstockPDescTrimI0%, 1) = " ") = 0 THEN GOTO 5330
5310         addstockPDescTrimI0% = addstockPDescTrimI0% - 1
5320         GOTO 5290
5330     REM END WHILE
5340     addstockPDesc0$ = LEFT$(invDescBuf$, addstockPDescTrimI0%)
5350     addstockPQty0% = CVI(invQtyBuf$)
5360     addstockPReorder0% = CVI(invReorderBuf$)
5370     addstockPPrice0! = CVS(invPriceBuf$)
5380     isemptyFlag0$ = addstockPFlag0$
5390     GOSUB 1570
5400     IF (isemptyResult0%) = 0 THEN GOTO 5450
5410         shownullentrymessagePartStr0$ = addstockPartStr0$
5420         GOSUB 2290
5430         GOSUB 1750
5440         RETURN
5450     REM END IF

5460         showaddstockscreenPartNum0% = addstockPart0%
5470         showaddstockscreenDesc0$ = addstockPDesc0$
5480         showaddstockscreenQty0% = addstockPQty0%
5490         showaddstockscreenReorder0% = addstockPReorder0%
5500         GOSUB 2960
5510         LOCATE 14, tabcol%
5520         INPUT " Quantity to add"; addstockAddStr0$
5530         addstockAddAmt0% = VAL(addstockAddStr0$)
5540         IF (addstockAddAmt0% < 0) = 0 THEN GOTO 5570
5550             GOSUB 3120
5560             GOSUB 1750
5570         REM END IF
5580         IF (addstockAddAmt0% >= 0) = 0 THEN GOTO 5460
5590     REM END DO

5600     addstockPQty0% = addstockPQty0% + addstockAddAmt0%
5610     ' inv[...] = p  (write back a let-bound record)
5620     LSET invFlagBuf$ = addstockPFlag0$
5630     LSET invDescBuf$ = addstockPDesc0$
5640     LSET invQtyBuf$ = MKI$(addstockPQty0%)
5650     LSET invReorderBuf$ = MKI$(addstockPReorder0%)
5660     LSET invPriceBuf$ = MKS$(addstockPPrice0!)
5670     PUT #1, addstockPart0%
5680     RETURN
5690 ' end procedure addstock

5700 ' procedure subtractstock()
5710     CLS
5720     LOCATE 5, 20
5730     PRINT "S U B T R A C T I N G    S T O C K"

5740         LOCATE 8, 25
5750         GOSUB 1700
5760         subtractstockPartStr0$ = readpartnumberinputResult0$
5770         subtractstockPart0% = VAL(subtractstockPartStr0$)
5780         partinrangeN0% = subtractstockPart0%
5790         GOSUB 1610
5800         subtractstockValidPart0% = partinrangeResult0%
5810         IF (subtractstockValidPart0% = 0) = 0 THEN GOTO 5840
5820             GOSUB 2220
5830             GOSUB 1750
5840         REM END IF
5850         IF (subtractstockValidPart0% <> 0) = 0 THEN GOTO 5740
5860     REM END DO

5870     ' let p = inv[...]  (whole-record read)
5880     GET #1, subtractstockPart0%
5890     subtractstockPFlagTrimI0% = LEN(invFlagBuf$)
5900     IF (subtractstockPFlagTrimI0% > 0) = 0 THEN GOTO 5940
5910     IF (MID$(invFlagBuf$, subtractstockPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 5940
5920         subtractstockPFlagTrimI0% = subtractstockPFlagTrimI0% - 1
5930         GOTO 5900
5940     REM END WHILE
5950     subtractstockPFlag0$ = LEFT$(invFlagBuf$, subtractstockPFlagTrimI0%)
5960     subtractstockPDescTrimI0% = LEN(invDescBuf$)
5970     IF (subtractstockPDescTrimI0% > 0) = 0 THEN GOTO 6010
5980     IF (MID$(invDescBuf$, subtractstockPDescTrimI0%, 1) = " ") = 0 THEN GOTO 6010
5990         subtractstockPDescTrimI0% = subtractstockPDescTrimI0% - 1
6000         GOTO 5970
6010     REM END WHILE
6020     subtractstockPDesc0$ = LEFT$(invDescBuf$, subtractstockPDescTrimI0%)
6030     subtractstockPQty0% = CVI(invQtyBuf$)
6040     subtractstockPReorder0% = CVI(invReorderBuf$)
6050     subtractstockPPrice0! = CVS(invPriceBuf$)
6060     isemptyFlag0$ = subtractstockPFlag0$
6070     GOSUB 1570
6080     IF (isemptyResult0%) = 0 THEN GOTO 6130
6090         shownullentrymessagePartStr0$ = subtractstockPartStr0$
6100         GOSUB 2290
6110         GOSUB 1750
6120         RETURN
6130     REM END IF

6140         showsubtractstockscreenPartNum0% = subtractstockPart0%
6150         showsubtractstockscreenDesc0$ = subtractstockPDesc0$
6160         showsubtractstockscreenQty0% = subtractstockPQty0%
6170         showsubtractstockscreenReorder0% = subtractstockPReorder0%
6180         GOSUB 3190
6190         LOCATE 14, tabcol%
6200         INPUT "Quantity to subtract"; subtractstockSubStr0$
6210         subtractstockSubAmt0% = VAL(subtractstockSubStr0$)
6220         subtractstockOverSubtract0% = 0
6230         IF (subtractstockSubAmt0% >= 0) = 0 THEN GOTO 6290
6240         IF ((subtractstockPQty0% - subtractstockSubAmt0%) < 0) = 0 THEN GOTO 6290
6250             subtractstockOverSubtract0% = 1
6260             showoversubtractwarningOnHand0% = subtractstockPQty0%
6270             GOSUB 3350
6280             GOSUB 1750
6290         REM END IF
6300         IF (subtractstockSubAmt0% >= 0) = 0 THEN GOTO 6140
6310         IF (subtractstockOverSubtract0% = 0) = 0 THEN GOTO 6140
6320     REM END DO

6330     subtractstockPQty0% = subtractstockPQty0% - subtractstockSubAmt0%
6340     IF (subtractstockPQty0% <= subtractstockPReorder0%) = 0 THEN GOTO 6360
6350         LOCATE 16, tabcol%
6360     REM END IF
6370     PRINT (("quantity now" + STR$(subtractstockPQty0%)) + " reorder level") + STR$(subtractstockPReorder0%)
6380     ' inv[...] = p  (write back a let-bound record)
6390     LSET invFlagBuf$ = subtractstockPFlag0$
6400     LSET invDescBuf$ = subtractstockPDesc0$
6410     LSET invQtyBuf$ = MKI$(subtractstockPQty0%)
6420     LSET invReorderBuf$ = MKI$(subtractstockPReorder0%)
6430     LSET invPriceBuf$ = MKS$(subtractstockPPrice0!)
6440     PUT #1, subtractstockPart0%
6450     RETURN
6460 ' end procedure subtractstock

6470 ' procedure reorderreport()
6480     GOSUB 2620
6490     reorderreportReportLineCount0% = 0
6500     FOR reorderreportI0% = 1 TO partcount%
6510         ' let p = inv[...]  (whole-record read)
6520         GET #1, reorderreportI0%
6530         reorderreportPFlagTrimI0% = LEN(invFlagBuf$)
6540         IF (reorderreportPFlagTrimI0% > 0) = 0 THEN GOTO 6580
6550         IF (MID$(invFlagBuf$, reorderreportPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 6580
6560             reorderreportPFlagTrimI0% = reorderreportPFlagTrimI0% - 1
6570             GOTO 6540
6580         REM END WHILE
6590         reorderreportPFlag0$ = LEFT$(invFlagBuf$, reorderreportPFlagTrimI0%)
6600         reorderreportPDescTrimI0% = LEN(invDescBuf$)
6610         IF (reorderreportPDescTrimI0% > 0) = 0 THEN GOTO 6650
6620         IF (MID$(invDescBuf$, reorderreportPDescTrimI0%, 1) = " ") = 0 THEN GOTO 6650
6630             reorderreportPDescTrimI0% = reorderreportPDescTrimI0% - 1
6640             GOTO 6610
6650         REM END WHILE
6660         reorderreportPDesc0$ = LEFT$(invDescBuf$, reorderreportPDescTrimI0%)
6670         reorderreportPQty0% = CVI(invQtyBuf$)
6680         reorderreportPReorder0% = CVI(invReorderBuf$)
6690         reorderreportPPrice0! = CVS(invPriceBuf$)
6700         IF (reorderreportPQty0% < reorderreportPReorder0%) = 0 THEN GOTO 6810
6710             printreorderlinePartNum0% = reorderreportI0%
6720             printreorderlineDesc0$ = reorderreportPDesc0$
6730             printreorderlineQty0% = reorderreportPQty0%
6740             printreorderlineReorder0% = reorderreportPReorder0%
6750             GOSUB 2720
6760             reorderreportReportLineCount0% = reorderreportReportLineCount0% + 1
6770             IF (reorderreportReportLineCount0% > 15) = 0 THEN GOTO 6800
6780                 GOSUB 1820
6790                 reorderreportReportLineCount0% = 0
6800             REM END IF
6810         REM END IF
6820     NEXT reorderreportI0%
6830     GOSUB 1820
6840     RETURN
6850 ' end procedure reorderreport
