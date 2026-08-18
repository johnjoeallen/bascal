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
940 ON ERROR GOTO 1440

950     GOSUB 1890
960     GOSUB 1740
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
1120             GOSUB 3430
1130             GOTO 1320
1140             GOSUB 3970
1150             GOTO 1320
1160             GOSUB 4660
1170             GOTO 1320
1180             GOSUB 5020
1190             GOTO 1320
1200             GOSUB 5700
1210             GOTO 1320
1220             GOSUB 6470
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
1420 ' GOSUB/function-call frame -- that's a language-design reason, so it
1430 ' stays a label.
1440 LOCATE 25, 1
1450 ' `err` and `erl` are real numeric system pseudo-variables, passed
1460 ' straight through like fhb's own ERR/ERL (his 3390: "an error on
1470 ' line";ERL). Unlike fhb's version, this doesn't decode ERR into a
1480 ' message per the header note above -- it just reports the raw
1490 ' code. See MANUAL.md's ON ERROR GOTO section.
1500 PRINT (("There has been an error on line" + STR$(ERL)) + "  Error #") + STR$(ERR)
1510 GOSUB 1740
1520 k$ = readkeyResult0$
1530 RESUME NEXT
1540 END

1550 ' function isempty%(flag$)
1560     isemptyResult0% = ASC(isemptyFlag0$) = 255
1570     RETURN
1580 ' end function isempty%

1590 ' function partinrange%(n%)
1600     IF (partinrangeN0% >= 1) = 0 THEN GOTO 1640
1610     IF (partinrangeN0% <= partcount%) = 0 THEN GOTO 1640
1620         partinrangeResult0% = 1
1630         RETURN
1640     REM END IF
1650     partinrangeResult0% = 0
1660     RETURN
1670 ' end function partinrange%

1680 ' function readpartnumberinput$()
1690     INPUT "Input part number"; readpartnumberinputS0$
1700     readpartnumberinputResult0$ = readpartnumberinputS0$
1710     RETURN
1720 ' end function readpartnumberinput$

1730 ' function readkey$()
1740         readkeyK0$ = INKEY$
1750         IF (readkeyK0$ <> "") = 0 THEN GOTO 1740
1760     REM END DO
1770     readkeyResult0$ = readkeyK0$
1780     RETURN
1790 ' end function readkey$

1800 ' procedure waitanykey()
1810     LOCATE 25, 10
1820     PRINT "Press the AnyKey to continue...";
1830         waitanykeyK0$ = INKEY$
1840         IF (waitanykeyK0$ <> "") = 0 THEN GOTO 1830
1850     REM END DO
1860     RETURN
1870 ' end procedure waitanykey

1880 ' procedure showmainmenu()
1890     CLS
1900     COLOR 14, 4
1910     CLS
1920     LOCATE 6, 1
1930     PRINT
1940     ' `tab(n)` passes straight through to real TAB(n), same as
1950     ' fhb's own `PRINT TAB(V) "..."` -- but only as a bare item in
1960     ' a PRINT list, juxtaposed or `;`-separated like here. Real
1970     ' BASCOM rejects `"literal" + tab(n) + ...` (TAB isn't a real
1980     ' string function you can concatenate); see printListHeader()
1990     ' and printReorderHeader() below, which need `;` between a
2000     ' preceding string and a `tab(n)` for exactly this reason.
2010     PRINT TAB(30)"Inventory Program"
2020     PRINT
2030     PRINT TAB(tabcol%)"1......C)heck a part"
2040     PRINT TAB(tabcol%)"2......E)dit/overwrite/add a part"
2050     PRINT TAB(tabcol%)("3......L)ist all" + STR$(partcount%)) + "parts"
2060     PRINT TAB(tabcol%)"4......A)dd stock"
2070     PRINT TAB(tabcol%)"5......S)ubtract stock"
2080     PRINT TAB(tabcol%)"6......R)eorder Report"
2090     PRINT
2100     PRINT TAB(tabcol%)"7......Q)uit to BASIC"
2110     PRINT TAB(tabcol%)"8......eX)it to system"
2120     RETURN
2130 ' end procedure showmainmenu

2140 ' procedure showbadpartnumber()
2150     CLS
2160     LOCATE 10, 10
2170     PRINT "Part number is out of permissable range of 1 to" + STR$(partcount%)
2180     RETURN
2190 ' end procedure showbadpartnumber

2200 ' procedure showrangeretrymessage()
2210     LOCATE 10, 15
2220     PRINT "The Part number is out of permissable range of 1 to" + STR$(partcount%)
2230     LOCATE 25, 15
2240     PRINT "Press the Anykey to reenter part number...";
2250     RETURN
2260 ' end procedure showrangeretrymessage

2270 ' procedure shownullentrymessage(partstr$)
2280     LOCATE 10, tabcol%
2290     PRINT ("Part number " + shownullentrymessagePartStr0$) + " is a null entry"
2300     RETURN
2310 ' end procedure shownullentrymessage

2320 ' procedure showpartstatus(partnum%, desc$, qty%, reorder%, price!)
2330     CLS
2340     LOCATE 5, 1
2350     PRINT TAB(tabcol%)"Inventory Status for Individual Part Number"
2360     PRINT TAB(tabcol%)"==========================================="
2370     PRINT
2380     PRINT
2390     PRINT TAB(tabcol%)"     Part number:  " + STR$(showpartstatusPartNum0%)
2400     PRINT
2410     PRINT TAB(tabcol%)"       Item name:  " + showpartstatusDesc0$
2420     PRINT TAB(tabcol%)"Quantity on hand:  " + STR$(showpartstatusQty0%)
2430     PRINT TAB(tabcol%)"   Reorder level:  " + STR$(showpartstatusReorder0%)
2440     PRINT TAB(tabcol%)"      Unit price:  " + STR$(showpartstatusPrice0!)
2450     RETURN
2460 ' end procedure showpartstatus

2470 ' procedure printlistheader()
2480     CLS
2490     PRINT TAB(25)"I N V E N T O R Y   L I S T I N G"; TAB(65); STR$(partcount%) + "items"
2500     PRINT "                                          Quantity       Reorder"
2510     PRINT " Partno           Description             on hand         level"
2520     LOCATE 25, 1
2530     PRINT "Press the AnyKey to scroll listing...";
2540     RETURN
2550 ' end procedure printlistheader

2560 ' procedure printinventoryline(partnum%, desc$, qty%, reorder%)
2570     PRINT (((((STR$(printinventorylinePartNum0%) + "  ") + printinventorylineDesc0$) + "   ") + STR$(printinventorylineQty0%)) + "          ") + STR$(printinventorylineReorder0%)
2580     RETURN
2590 ' end procedure printinventoryline

2600 ' procedure printreorderheader()
2610     CLS
2620     LOCATE 1, tabcol%
2630     PRINT "Reorder Report"; TAB(55); DATE$
2640     PRINT
2650     PRINT "                                             Quantity       Reorder"
2660     PRINT "    Partno           Description             on hand         level"
2670     PRINT "   =======  ==============================   ========       ======="
2680     RETURN
2690 ' end procedure printreorderheader

2700 ' procedure printreorderline(partnum%, desc$, qty%, reorder%)
2710     PRINT (((((("  " + STR$(printreorderlinePartNum0%)) + "  ") + printreorderlineDesc0$) + "   ") + STR$(printreorderlineQty0%)) + "          ") + STR$(printreorderlineReorder0%)
2720     RETURN
2730 ' end procedure printreorderline

2740 ' procedure gatherpartdetails(partnum%, desc$, qty%, reorder%, price!)
2750     CLS
2760     LOCATE 4, tabcol%
2770     PRINT "Adding or Overwriting a Record"
2780     LOCATE 8, tabcol%
2790     PRINT "Record/Partno" + STR$(gatherpartdetailsPartNum0%)
2800     LOCATE 11, 39
2810     PRINT "------------------------------"
2820     LOCATE 10, tabcol%
2830     INPUT "      Description"; gatherpartdetailsDesc0$
2840     LOCATE 12, tabcol%
2850     INPUT "Quantity in stock"; gatherpartdetailsQty0%
2860     LOCATE 14, tabcol%
2870     INPUT "    Reorder level"; gatherpartdetailsReorder0%
2880     LOCATE 16, tabcol%
2890     INPUT "       Unit price"; gatherpartdetailsPrice0!
2900     LOCATE 18, tabcol%
2910     PRINT "Is information correct (Y/N)?"
2920     RETURN
2930 ' end procedure gatherpartdetails

2940 ' procedure showaddstockscreen(partnum%, desc$, qty%, reorder%)
2950     CLS
2960     LOCATE 4, 25
2970     PRINT "Add to an inventory part number"
2980     LOCATE 5, 25
2990     PRINT "==============================="
3000     LOCATE 8, tabcol%
3010     PRINT "     Part number: " + STR$(showaddstockscreenPartNum0%)
3020     LOCATE 9, tabcol%
3030     PRINT "Item description: " + showaddstockscreenDesc0$
3040     LOCATE 10, tabcol%
3050     PRINT "Quantity on hand: " + STR$(showaddstockscreenQty0%)
3060     LOCATE 11, tabcol%
3070     PRINT "   Reorder Level: " + STR$(showaddstockscreenReorder0%)
3080     RETURN
3090 ' end procedure showaddstockscreen

3100 ' procedure shownegativeqtywarning()
3110     LOCATE 17, 15
3120     PRINT "The quantity to add must NOT be a negative number"
3130     LOCATE 25, 1
3140     PRINT "Please press the Anykey to reenter quantity to add...";
3150     RETURN
3160 ' end procedure shownegativeqtywarning

3170 ' procedure showsubtractstockscreen(partnum%, desc$, qty%, reorder%)
3180     CLS
3190     LOCATE 4, tabcol%
3200     PRINT "Subtract an inventory part number"
3210     LOCATE 5, tabcol%
3220     PRINT "================================="
3230     LOCATE 8, tabcol%
3240     PRINT "         Part number: " + STR$(showsubtractstockscreenPartNum0%)
3250     LOCATE 9, tabcol%
3260     PRINT "    Item description: " + showsubtractstockscreenDesc0$
3270     LOCATE 10, tabcol%
3280     PRINT "    Quantity on hand: " + STR$(showsubtractstockscreenQty0%)
3290     LOCATE 11, tabcol%
3300     PRINT "       Reorder Level: " + STR$(showsubtractstockscreenReorder0%)
3310     RETURN
3320 ' end procedure showsubtractstockscreen

3330 ' procedure showoversubtractwarning(onhand%)
3340     LOCATE 17, 5
3350     PRINT "The quantity to SUBTRACT must NOT result in NEGATIVE inventory"
3360     LOCATE 18, 5
3370     PRINT ("Only" + STR$(showoversubtractwarningOnHand0%)) + " IN STOCK"
3380     LOCATE 25, 1
3390     PRINT "Please press the Anykey to reenter quantity to subtract...";
3400     RETURN
3410 ' end procedure showoversubtractwarning

3420 ' procedure checkpart()
3430     GOSUB 1690
3440     checkpartPartStr0$ = readpartnumberinputResult0$
3450     checkpartPart0% = VAL(checkpartPartStr0$)
3460     partinrangeN0% = checkpartPart0%
3470     GOSUB 1600
3480     IF (partinrangeResult0% = 0) = 0 THEN GOTO 3520
3490         GOSUB 2150
3500         GOSUB 1810
3510         RETURN
3520     REM END IF
3530     ' BASCAL-ism: `let p = inv[part%]` reads record `part%` of the
3540     ' `inv` file into a local record variable `p` -- one expression
3550     ' for what fhb's `GET #1, PART!` plus five separate field reads
3560     ' (F$, D$, CVI(Q$), CVI(R$), CVS(P$)) did by hand. The write
3570     ' side, `inv[part%] = { ... }` (see editRecord() below), is the
3580     ' same sugar for PUT plus the LSET/MKx$ packing it replaces.
3590     ' let p = inv[...]  (whole-record read)
3600     GET #1, checkpartPart0%
3610     checkpartPFlagTrimI0% = LEN(invFlagBuf$)
3620     IF (checkpartPFlagTrimI0% > 0) = 0 THEN GOTO 3660
3630     IF (MID$(invFlagBuf$, checkpartPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 3660
3640         checkpartPFlagTrimI0% = checkpartPFlagTrimI0% - 1
3650         GOTO 3620
3660     REM END WHILE
3670     checkpartPFlag0$ = LEFT$(invFlagBuf$, checkpartPFlagTrimI0%)
3680     checkpartPDescTrimI0% = LEN(invDescBuf$)
3690     IF (checkpartPDescTrimI0% > 0) = 0 THEN GOTO 3730
3700     IF (MID$(invDescBuf$, checkpartPDescTrimI0%, 1) = " ") = 0 THEN GOTO 3730
3710         checkpartPDescTrimI0% = checkpartPDescTrimI0% - 1
3720         GOTO 3690
3730     REM END WHILE
3740     checkpartPDesc0$ = LEFT$(invDescBuf$, checkpartPDescTrimI0%)
3750     checkpartPQty0% = CVI(invQtyBuf$)
3760     checkpartPReorder0% = CVI(invReorderBuf$)
3770     checkpartPPrice0! = CVS(invPriceBuf$)
3780     isemptyFlag0$ = checkpartPFlag0$
3790     GOSUB 1560
3800     IF (isemptyResult0%) = 0 THEN GOTO 3860
3810         CLS
3820         LOCATE 10, 18
3830         PRINT ("Part number" + STR$(checkpartPart0%)) + "is still a null entry at this time"
3840         GOSUB 1810
3850         RETURN
3860     REM END IF
3870     showpartstatusPartNum0% = checkpartPart0%
3880     showpartstatusDesc0$ = checkpartPDesc0$
3890     showpartstatusQty0% = checkpartPQty0%
3900     showpartstatusReorder0% = checkpartPReorder0%
3910     showpartstatusPrice0! = checkpartPPrice0!
3920     GOSUB 2330
3930     GOSUB 1810
3940     RETURN
3950 ' end procedure checkpart

3960 ' procedure editrecord()
3970     CLS
3980     LOCATE 10, tabcol%
3990     GOSUB 1690
4000     editrecordPartStr0$ = readpartnumberinputResult0$
4010     editrecordPart0% = VAL(editrecordPartStr0$)
4020     partinrangeN0% = editrecordPart0%
4030     GOSUB 1600
4040     IF (partinrangeResult0% = 0) = 0 THEN GOTO 4080
4050         GOSUB 2150
4060         GOSUB 1810
4070         RETURN
4080     REM END IF
4090     ' let p = inv[...]  (whole-record read)
4100     GET #1, editrecordPart0%
4110     editrecordPFlagTrimI0% = LEN(invFlagBuf$)
4120     IF (editrecordPFlagTrimI0% > 0) = 0 THEN GOTO 4160
4130     IF (MID$(invFlagBuf$, editrecordPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 4160
4140         editrecordPFlagTrimI0% = editrecordPFlagTrimI0% - 1
4150         GOTO 4120
4160     REM END WHILE
4170     editrecordPFlag0$ = LEFT$(invFlagBuf$, editrecordPFlagTrimI0%)
4180     editrecordPDescTrimI0% = LEN(invDescBuf$)
4190     IF (editrecordPDescTrimI0% > 0) = 0 THEN GOTO 4230
4200     IF (MID$(invDescBuf$, editrecordPDescTrimI0%, 1) = " ") = 0 THEN GOTO 4230
4210         editrecordPDescTrimI0% = editrecordPDescTrimI0% - 1
4220         GOTO 4190
4230     REM END WHILE
4240     editrecordPDesc0$ = LEFT$(invDescBuf$, editrecordPDescTrimI0%)
4250     editrecordPQty0% = CVI(invQtyBuf$)
4260     editrecordPReorder0% = CVI(invReorderBuf$)
4270     editrecordPPrice0! = CVS(invPriceBuf$)
4280     isemptyFlag0$ = editrecordPFlag0$
4290     GOSUB 1560
4300     IF (isemptyResult0% = 0) = 0 THEN GOTO 4390
4310         LOCATE 12, tabcol%
4320         PRINT "Overwrite existing part data?"
4330         GOSUB 1740
4340         editrecordKp0$ = readkeyResult0$
4350         IF (editrecordKp0$ <> "Y") = 0 THEN GOTO 4380
4360         IF (editrecordKp0$ <> "y") = 0 THEN GOTO 4380
4370             RETURN
4380         REM END IF
4390     REM END IF

4400         gatherpartdetailsPartNum0% = editrecordPart0%
4410         gatherpartdetailsDesc0$ = editrecordEditDesc0$
4420         gatherpartdetailsQty0% = editrecordEditQty0%
4430         gatherpartdetailsReorder0% = editrecordEditReorder0%
4440         gatherpartdetailsPrice0! = editrecordEditPrice0!
4450         GOSUB 2750
4460         editrecordEditDesc0$ = gatherpartdetailsDesc0$
4470         editrecordEditQty0% = gatherpartdetailsQty0%
4480         editrecordEditReorder0% = gatherpartdetailsReorder0%
4490         editrecordEditPrice0! = gatherpartdetailsPrice0!
4500         GOSUB 1740
4510         editrecordKp0$ = readkeyResult0$
4520         IF (editrecordKp0$ = "Y") <> 0 THEN GOTO 4550
4530         IF (editrecordKp0$ = "y") <> 0 THEN GOTO 4550
4540         GOTO 4400
4550     REM END DO
4560     ' inv[...] = { ... }  (whole-record write)
4570     LSET invFlagBuf$ = "1"
4580     LSET invDescBuf$ = editrecordEditDesc0$
4590     LSET invQtyBuf$ = MKI$(editrecordEditQty0%)
4600     LSET invReorderBuf$ = MKI$(editrecordEditReorder0%)
4610     LSET invPriceBuf$ = MKS$(editrecordEditPrice0!)
4620     PUT #1, editrecordPart0%
4630     RETURN
4640 ' end procedure editrecord

4650 ' procedure listall()
4660     GOSUB 2480
4670     listallScrollCount0% = 0
4680     FOR listallI0% = 1 TO partcount%
4690         ' let p = inv[...]  (whole-record read)
4700         GET #1, listallI0%
4710         listallPFlagTrimI0% = LEN(invFlagBuf$)
4720         IF (listallPFlagTrimI0% > 0) = 0 THEN GOTO 4760
4730         IF (MID$(invFlagBuf$, listallPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 4760
4740             listallPFlagTrimI0% = listallPFlagTrimI0% - 1
4750             GOTO 4720
4760         REM END WHILE
4770         listallPFlag0$ = LEFT$(invFlagBuf$, listallPFlagTrimI0%)
4780         listallPDescTrimI0% = LEN(invDescBuf$)
4790         IF (listallPDescTrimI0% > 0) = 0 THEN GOTO 4830
4800         IF (MID$(invDescBuf$, listallPDescTrimI0%, 1) = " ") = 0 THEN GOTO 4830
4810             listallPDescTrimI0% = listallPDescTrimI0% - 1
4820             GOTO 4790
4830         REM END WHILE
4840         listallPDesc0$ = LEFT$(invDescBuf$, listallPDescTrimI0%)
4850         listallPQty0% = CVI(invQtyBuf$)
4860         listallPReorder0% = CVI(invReorderBuf$)
4870         listallPPrice0! = CVS(invPriceBuf$)
4880         printinventorylinePartNum0% = listallI0%
4890         printinventorylineDesc0$ = listallPDesc0$
4900         printinventorylineQty0% = listallPQty0%
4910         printinventorylineReorder0% = listallPReorder0%
4920         GOSUB 2570
4930         listallScrollCount0% = listallScrollCount0% + 1
4940         IF (listallScrollCount0% = 20) = 0 THEN GOTO 4970
4950             GOSUB 1810
4960             listallScrollCount0% = 0
4970         REM END IF
4980     NEXT listallI0%
4990     RETURN
5000 ' end procedure listall

5010 ' procedure addstock()
5020     CLS
5030     LOCATE 5, 25
5040     PRINT "A D D I N G   S T O C K"

5050         LOCATE 8, 25
5060         GOSUB 1690
5070         addstockPartStr0$ = readpartnumberinputResult0$
5080         addstockPart0% = VAL(addstockPartStr0$)
5090         partinrangeN0% = addstockPart0%
5100         GOSUB 1600
5110         addstockValidPart0% = partinrangeResult0%
5120         IF (addstockValidPart0% = 0) = 0 THEN GOTO 5150
5130             GOSUB 2210
5140             GOSUB 1740
5150         REM END IF
5160         IF (addstockValidPart0% <> 0) = 0 THEN GOTO 5050
5170     REM END DO

5180     ' let p = inv[...]  (whole-record read)
5190     GET #1, addstockPart0%
5200     addstockPFlagTrimI0% = LEN(invFlagBuf$)
5210     IF (addstockPFlagTrimI0% > 0) = 0 THEN GOTO 5250
5220     IF (MID$(invFlagBuf$, addstockPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 5250
5230         addstockPFlagTrimI0% = addstockPFlagTrimI0% - 1
5240         GOTO 5210
5250     REM END WHILE
5260     addstockPFlag0$ = LEFT$(invFlagBuf$, addstockPFlagTrimI0%)
5270     addstockPDescTrimI0% = LEN(invDescBuf$)
5280     IF (addstockPDescTrimI0% > 0) = 0 THEN GOTO 5320
5290     IF (MID$(invDescBuf$, addstockPDescTrimI0%, 1) = " ") = 0 THEN GOTO 5320
5300         addstockPDescTrimI0% = addstockPDescTrimI0% - 1
5310         GOTO 5280
5320     REM END WHILE
5330     addstockPDesc0$ = LEFT$(invDescBuf$, addstockPDescTrimI0%)
5340     addstockPQty0% = CVI(invQtyBuf$)
5350     addstockPReorder0% = CVI(invReorderBuf$)
5360     addstockPPrice0! = CVS(invPriceBuf$)
5370     isemptyFlag0$ = addstockPFlag0$
5380     GOSUB 1560
5390     IF (isemptyResult0%) = 0 THEN GOTO 5440
5400         shownullentrymessagePartStr0$ = addstockPartStr0$
5410         GOSUB 2280
5420         GOSUB 1740
5430         RETURN
5440     REM END IF

5450         showaddstockscreenPartNum0% = addstockPart0%
5460         showaddstockscreenDesc0$ = addstockPDesc0$
5470         showaddstockscreenQty0% = addstockPQty0%
5480         showaddstockscreenReorder0% = addstockPReorder0%
5490         GOSUB 2950
5500         LOCATE 14, tabcol%
5510         INPUT " Quantity to add"; addstockAddStr0$
5520         addstockAddAmt0% = VAL(addstockAddStr0$)
5530         IF (addstockAddAmt0% < 0) = 0 THEN GOTO 5560
5540             GOSUB 3110
5550             GOSUB 1740
5560         REM END IF
5570         IF (addstockAddAmt0% >= 0) = 0 THEN GOTO 5450
5580     REM END DO

5590     addstockPQty0% = addstockPQty0% + addstockAddAmt0%
5600     ' inv[...] = p  (write back a let-bound record)
5610     LSET invFlagBuf$ = addstockPFlag0$
5620     LSET invDescBuf$ = addstockPDesc0$
5630     LSET invQtyBuf$ = MKI$(addstockPQty0%)
5640     LSET invReorderBuf$ = MKI$(addstockPReorder0%)
5650     LSET invPriceBuf$ = MKS$(addstockPPrice0!)
5660     PUT #1, addstockPart0%
5670     RETURN
5680 ' end procedure addstock

5690 ' procedure subtractstock()
5700     CLS
5710     LOCATE 5, 20
5720     PRINT "S U B T R A C T I N G    S T O C K"

5730         LOCATE 8, 25
5740         GOSUB 1690
5750         subtractstockPartStr0$ = readpartnumberinputResult0$
5760         subtractstockPart0% = VAL(subtractstockPartStr0$)
5770         partinrangeN0% = subtractstockPart0%
5780         GOSUB 1600
5790         subtractstockValidPart0% = partinrangeResult0%
5800         IF (subtractstockValidPart0% = 0) = 0 THEN GOTO 5830
5810             GOSUB 2210
5820             GOSUB 1740
5830         REM END IF
5840         IF (subtractstockValidPart0% <> 0) = 0 THEN GOTO 5730
5850     REM END DO

5860     ' let p = inv[...]  (whole-record read)
5870     GET #1, subtractstockPart0%
5880     subtractstockPFlagTrimI0% = LEN(invFlagBuf$)
5890     IF (subtractstockPFlagTrimI0% > 0) = 0 THEN GOTO 5930
5900     IF (MID$(invFlagBuf$, subtractstockPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 5930
5910         subtractstockPFlagTrimI0% = subtractstockPFlagTrimI0% - 1
5920         GOTO 5890
5930     REM END WHILE
5940     subtractstockPFlag0$ = LEFT$(invFlagBuf$, subtractstockPFlagTrimI0%)
5950     subtractstockPDescTrimI0% = LEN(invDescBuf$)
5960     IF (subtractstockPDescTrimI0% > 0) = 0 THEN GOTO 6000
5970     IF (MID$(invDescBuf$, subtractstockPDescTrimI0%, 1) = " ") = 0 THEN GOTO 6000
5980         subtractstockPDescTrimI0% = subtractstockPDescTrimI0% - 1
5990         GOTO 5960
6000     REM END WHILE
6010     subtractstockPDesc0$ = LEFT$(invDescBuf$, subtractstockPDescTrimI0%)
6020     subtractstockPQty0% = CVI(invQtyBuf$)
6030     subtractstockPReorder0% = CVI(invReorderBuf$)
6040     subtractstockPPrice0! = CVS(invPriceBuf$)
6050     isemptyFlag0$ = subtractstockPFlag0$
6060     GOSUB 1560
6070     IF (isemptyResult0%) = 0 THEN GOTO 6120
6080         shownullentrymessagePartStr0$ = subtractstockPartStr0$
6090         GOSUB 2280
6100         GOSUB 1740
6110         RETURN
6120     REM END IF

6130         showsubtractstockscreenPartNum0% = subtractstockPart0%
6140         showsubtractstockscreenDesc0$ = subtractstockPDesc0$
6150         showsubtractstockscreenQty0% = subtractstockPQty0%
6160         showsubtractstockscreenReorder0% = subtractstockPReorder0%
6170         GOSUB 3180
6180         LOCATE 14, tabcol%
6190         INPUT "Quantity to subtract"; subtractstockSubStr0$
6200         subtractstockSubAmt0% = VAL(subtractstockSubStr0$)
6210         subtractstockOverSubtract0% = 0
6220         IF (subtractstockSubAmt0% >= 0) = 0 THEN GOTO 6280
6230         IF ((subtractstockPQty0% - subtractstockSubAmt0%) < 0) = 0 THEN GOTO 6280
6240             subtractstockOverSubtract0% = 1
6250             showoversubtractwarningOnHand0% = subtractstockPQty0%
6260             GOSUB 3340
6270             GOSUB 1740
6280         REM END IF
6290         IF (subtractstockSubAmt0% >= 0) = 0 THEN GOTO 6130
6300         IF (subtractstockOverSubtract0% = 0) = 0 THEN GOTO 6130
6310     REM END DO

6320     subtractstockPQty0% = subtractstockPQty0% - subtractstockSubAmt0%
6330     IF (subtractstockPQty0% <= subtractstockPReorder0%) = 0 THEN GOTO 6350
6340         LOCATE 16, tabcol%
6350     REM END IF
6360     PRINT (("quantity now" + STR$(subtractstockPQty0%)) + " reorder level") + STR$(subtractstockPReorder0%)
6370     ' inv[...] = p  (write back a let-bound record)
6380     LSET invFlagBuf$ = subtractstockPFlag0$
6390     LSET invDescBuf$ = subtractstockPDesc0$
6400     LSET invQtyBuf$ = MKI$(subtractstockPQty0%)
6410     LSET invReorderBuf$ = MKI$(subtractstockPReorder0%)
6420     LSET invPriceBuf$ = MKS$(subtractstockPPrice0!)
6430     PUT #1, subtractstockPart0%
6440     RETURN
6450 ' end procedure subtractstock

6460 ' procedure reorderreport()
6470     GOSUB 2610
6480     reorderreportReportLineCount0% = 0
6490     FOR reorderreportI0% = 1 TO partcount%
6500         ' let p = inv[...]  (whole-record read)
6510         GET #1, reorderreportI0%
6520         reorderreportPFlagTrimI0% = LEN(invFlagBuf$)
6530         IF (reorderreportPFlagTrimI0% > 0) = 0 THEN GOTO 6570
6540         IF (MID$(invFlagBuf$, reorderreportPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 6570
6550             reorderreportPFlagTrimI0% = reorderreportPFlagTrimI0% - 1
6560             GOTO 6530
6570         REM END WHILE
6580         reorderreportPFlag0$ = LEFT$(invFlagBuf$, reorderreportPFlagTrimI0%)
6590         reorderreportPDescTrimI0% = LEN(invDescBuf$)
6600         IF (reorderreportPDescTrimI0% > 0) = 0 THEN GOTO 6640
6610         IF (MID$(invDescBuf$, reorderreportPDescTrimI0%, 1) = " ") = 0 THEN GOTO 6640
6620             reorderreportPDescTrimI0% = reorderreportPDescTrimI0% - 1
6630             GOTO 6600
6640         REM END WHILE
6650         reorderreportPDesc0$ = LEFT$(invDescBuf$, reorderreportPDescTrimI0%)
6660         reorderreportPQty0% = CVI(invQtyBuf$)
6670         reorderreportPReorder0% = CVI(invReorderBuf$)
6680         reorderreportPPrice0! = CVS(invPriceBuf$)
6690         IF (reorderreportPQty0% < reorderreportPReorder0%) = 0 THEN GOTO 6800
6700             printreorderlinePartNum0% = reorderreportI0%
6710             printreorderlineDesc0$ = reorderreportPDesc0$
6720             printreorderlineQty0% = reorderreportPQty0%
6730             printreorderlineReorder0% = reorderreportPReorder0%
6740             GOSUB 2710
6750             reorderreportReportLineCount0% = reorderreportReportLineCount0% + 1
6760             IF (reorderreportReportLineCount0% > 15) = 0 THEN GOTO 6790
6770                 GOSUB 1810
6780                 reorderreportReportLineCount0% = 0
6790             REM END IF
6800         REM END IF
6810     NEXT reorderreportI0%
6820     GOSUB 1810
6830     RETURN
6840 ' end procedure reorderreport
