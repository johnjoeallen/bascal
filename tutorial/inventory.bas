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
940 ON ERROR GOTO 1500

950     GOSUB 1950
960     GOSUB 1800
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
1120             GOSUB 3490
1130             GOTO 1320
1140             GOSUB 4030
1150             GOTO 1320
1160             GOSUB 4720
1170             GOTO 1320
1180             GOSUB 5080
1190             GOTO 1320
1200             GOSUB 5760
1210             GOTO 1320
1220             GOSUB 6530
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
1400 ' A raw label, not a procedure -- ON ERROR GOTO reaches either one with
1410 ' a plain GOTO, so that's not the deciding factor. What is: a procedure
1420 ' with a body that doesn't end in `return` gets an unconditional RETURN
1430 ' appended by bcc (see codegen's ends_with_return), and this body ends
1440 ' in `resume next`, not `return`. Normally harmless dead code -- RESUME
1450 ' jumps away first -- but if a future edit ever dropped that trailing
1460 ' RESUME, execution would fall into the auto-appended RETURN with no
1470 ' GOSUB frame on the stack to pop (GOTO never pushes one), crashing
1480 ' with "RETURN without GOSUB". Staying a label sidesteps that risk
1490 ' entirely: bcc never appends anything after a label's body.
1500 LOCATE 25, 1
1510 ' `err` and `erl` are real numeric system pseudo-variables, passed
1520 ' straight through like fhb's own ERR/ERL (his 3390: "an error on
1530 ' line";ERL). Unlike fhb's version, this doesn't decode ERR into a
1540 ' message per the header note above -- it just reports the raw
1550 ' code. See MANUAL.md's ON ERROR GOTO section.
1560 PRINT (("There has been an error on line" + STR$(ERL)) + "  Error #") + STR$(ERR)
1570 GOSUB 1800
1580 k$ = readkeyResult0$
1590 RESUME NEXT
1600 END

1610 ' function isempty%(flag$)
1620     isemptyResult0% = ASC(isemptyFlag0$) = 255
1630     RETURN
1640 ' end function isempty%

1650 ' function partinrange%(n%)
1660     IF (partinrangeN0% >= 1) = 0 THEN GOTO 1700
1670     IF (partinrangeN0% <= partcount%) = 0 THEN GOTO 1700
1680         partinrangeResult0% = 1
1690         RETURN
1700     REM END IF
1710     partinrangeResult0% = 0
1720     RETURN
1730 ' end function partinrange%

1740 ' function readpartnumberinput$()
1750     INPUT "Input part number"; readpartnumberinputS0$
1760     readpartnumberinputResult0$ = readpartnumberinputS0$
1770     RETURN
1780 ' end function readpartnumberinput$

1790 ' function readkey$()
1800         readkeyK0$ = INKEY$
1810         IF (readkeyK0$ <> "") = 0 THEN GOTO 1800
1820     REM END DO
1830     readkeyResult0$ = readkeyK0$
1840     RETURN
1850 ' end function readkey$

1860 ' procedure waitanykey()
1870     LOCATE 25, 10
1880     PRINT "Press the AnyKey to continue...";
1890         waitanykeyK0$ = INKEY$
1900         IF (waitanykeyK0$ <> "") = 0 THEN GOTO 1890
1910     REM END DO
1920     RETURN
1930 ' end procedure waitanykey

1940 ' procedure showmainmenu()
1950     CLS
1960     COLOR 14, 4
1970     CLS
1980     LOCATE 6, 1
1990     PRINT
2000     ' `tab(n)` passes straight through to real TAB(n), same as
2010     ' fhb's own `PRINT TAB(V) "..."` -- but only as a bare item in
2020     ' a PRINT list, juxtaposed or `;`-separated like here. Real
2030     ' BASCOM rejects `"literal" + tab(n) + ...` (TAB isn't a real
2040     ' string function you can concatenate); see printListHeader()
2050     ' and printReorderHeader() below, which need `;` between a
2060     ' preceding string and a `tab(n)` for exactly this reason.
2070     PRINT TAB(30)"Inventory Program"
2080     PRINT
2090     PRINT TAB(tabcol%)"1......C)heck a part"
2100     PRINT TAB(tabcol%)"2......E)dit/overwrite/add a part"
2110     PRINT TAB(tabcol%)("3......L)ist all" + STR$(partcount%)) + "parts"
2120     PRINT TAB(tabcol%)"4......A)dd stock"
2130     PRINT TAB(tabcol%)"5......S)ubtract stock"
2140     PRINT TAB(tabcol%)"6......R)eorder Report"
2150     PRINT
2160     PRINT TAB(tabcol%)"7......Q)uit to BASIC"
2170     PRINT TAB(tabcol%)"8......eX)it to system"
2180     RETURN
2190 ' end procedure showmainmenu

2200 ' procedure showbadpartnumber()
2210     CLS
2220     LOCATE 10, 10
2230     PRINT "Part number is out of permissable range of 1 to" + STR$(partcount%)
2240     RETURN
2250 ' end procedure showbadpartnumber

2260 ' procedure showrangeretrymessage()
2270     LOCATE 10, 15
2280     PRINT "The Part number is out of permissable range of 1 to" + STR$(partcount%)
2290     LOCATE 25, 15
2300     PRINT "Press the Anykey to reenter part number...";
2310     RETURN
2320 ' end procedure showrangeretrymessage

2330 ' procedure shownullentrymessage(partstr$)
2340     LOCATE 10, tabcol%
2350     PRINT ("Part number " + shownullentrymessagePartStr0$) + " is a null entry"
2360     RETURN
2370 ' end procedure shownullentrymessage

2380 ' procedure showpartstatus(partnum%, desc$, qty%, reorder%, price!)
2390     CLS
2400     LOCATE 5, 1
2410     PRINT TAB(tabcol%)"Inventory Status for Individual Part Number"
2420     PRINT TAB(tabcol%)"==========================================="
2430     PRINT
2440     PRINT
2450     PRINT TAB(tabcol%)"     Part number:  " + STR$(showpartstatusPartNum0%)
2460     PRINT
2470     PRINT TAB(tabcol%)"       Item name:  " + showpartstatusDesc0$
2480     PRINT TAB(tabcol%)"Quantity on hand:  " + STR$(showpartstatusQty0%)
2490     PRINT TAB(tabcol%)"   Reorder level:  " + STR$(showpartstatusReorder0%)
2500     PRINT TAB(tabcol%)"      Unit price:  " + STR$(showpartstatusPrice0!)
2510     RETURN
2520 ' end procedure showpartstatus

2530 ' procedure printlistheader()
2540     CLS
2550     PRINT TAB(25)"I N V E N T O R Y   L I S T I N G"; TAB(65); STR$(partcount%) + "items"
2560     PRINT "                                          Quantity       Reorder"
2570     PRINT " Partno           Description             on hand         level"
2580     LOCATE 25, 1
2590     PRINT "Press the AnyKey to scroll listing...";
2600     RETURN
2610 ' end procedure printlistheader

2620 ' procedure printinventoryline(partnum%, desc$, qty%, reorder%)
2630     PRINT (((((STR$(printinventorylinePartNum0%) + "  ") + printinventorylineDesc0$) + "   ") + STR$(printinventorylineQty0%)) + "          ") + STR$(printinventorylineReorder0%)
2640     RETURN
2650 ' end procedure printinventoryline

2660 ' procedure printreorderheader()
2670     CLS
2680     LOCATE 1, tabcol%
2690     PRINT "Reorder Report"; TAB(55); DATE$
2700     PRINT
2710     PRINT "                                             Quantity       Reorder"
2720     PRINT "    Partno           Description             on hand         level"
2730     PRINT "   =======  ==============================   ========       ======="
2740     RETURN
2750 ' end procedure printreorderheader

2760 ' procedure printreorderline(partnum%, desc$, qty%, reorder%)
2770     PRINT (((((("  " + STR$(printreorderlinePartNum0%)) + "  ") + printreorderlineDesc0$) + "   ") + STR$(printreorderlineQty0%)) + "          ") + STR$(printreorderlineReorder0%)
2780     RETURN
2790 ' end procedure printreorderline

2800 ' procedure gatherpartdetails(partnum%, desc$, qty%, reorder%, price!)
2810     CLS
2820     LOCATE 4, tabcol%
2830     PRINT "Adding or Overwriting a Record"
2840     LOCATE 8, tabcol%
2850     PRINT "Record/Partno" + STR$(gatherpartdetailsPartNum0%)
2860     LOCATE 11, 39
2870     PRINT "------------------------------"
2880     LOCATE 10, tabcol%
2890     INPUT "      Description"; gatherpartdetailsDesc0$
2900     LOCATE 12, tabcol%
2910     INPUT "Quantity in stock"; gatherpartdetailsQty0%
2920     LOCATE 14, tabcol%
2930     INPUT "    Reorder level"; gatherpartdetailsReorder0%
2940     LOCATE 16, tabcol%
2950     INPUT "       Unit price"; gatherpartdetailsPrice0!
2960     LOCATE 18, tabcol%
2970     PRINT "Is information correct (Y/N)?"
2980     RETURN
2990 ' end procedure gatherpartdetails

3000 ' procedure showaddstockscreen(partnum%, desc$, qty%, reorder%)
3010     CLS
3020     LOCATE 4, 25
3030     PRINT "Add to an inventory part number"
3040     LOCATE 5, 25
3050     PRINT "==============================="
3060     LOCATE 8, tabcol%
3070     PRINT "     Part number: " + STR$(showaddstockscreenPartNum0%)
3080     LOCATE 9, tabcol%
3090     PRINT "Item description: " + showaddstockscreenDesc0$
3100     LOCATE 10, tabcol%
3110     PRINT "Quantity on hand: " + STR$(showaddstockscreenQty0%)
3120     LOCATE 11, tabcol%
3130     PRINT "   Reorder Level: " + STR$(showaddstockscreenReorder0%)
3140     RETURN
3150 ' end procedure showaddstockscreen

3160 ' procedure shownegativeqtywarning()
3170     LOCATE 17, 15
3180     PRINT "The quantity to add must NOT be a negative number"
3190     LOCATE 25, 1
3200     PRINT "Please press the Anykey to reenter quantity to add...";
3210     RETURN
3220 ' end procedure shownegativeqtywarning

3230 ' procedure showsubtractstockscreen(partnum%, desc$, qty%, reorder%)
3240     CLS
3250     LOCATE 4, tabcol%
3260     PRINT "Subtract an inventory part number"
3270     LOCATE 5, tabcol%
3280     PRINT "================================="
3290     LOCATE 8, tabcol%
3300     PRINT "         Part number: " + STR$(showsubtractstockscreenPartNum0%)
3310     LOCATE 9, tabcol%
3320     PRINT "    Item description: " + showsubtractstockscreenDesc0$
3330     LOCATE 10, tabcol%
3340     PRINT "    Quantity on hand: " + STR$(showsubtractstockscreenQty0%)
3350     LOCATE 11, tabcol%
3360     PRINT "       Reorder Level: " + STR$(showsubtractstockscreenReorder0%)
3370     RETURN
3380 ' end procedure showsubtractstockscreen

3390 ' procedure showoversubtractwarning(onhand%)
3400     LOCATE 17, 5
3410     PRINT "The quantity to SUBTRACT must NOT result in NEGATIVE inventory"
3420     LOCATE 18, 5
3430     PRINT ("Only" + STR$(showoversubtractwarningOnHand0%)) + " IN STOCK"
3440     LOCATE 25, 1
3450     PRINT "Please press the Anykey to reenter quantity to subtract...";
3460     RETURN
3470 ' end procedure showoversubtractwarning

3480 ' procedure checkpart()
3490     GOSUB 1750
3500     checkpartPartStr0$ = readpartnumberinputResult0$
3510     checkpartPart0% = VAL(checkpartPartStr0$)
3520     partinrangeN0% = checkpartPart0%
3530     GOSUB 1660
3540     IF (partinrangeResult0% = 0) = 0 THEN GOTO 3580
3550         GOSUB 2210
3560         GOSUB 1870
3570         RETURN
3580     REM END IF
3590     ' BASCAL-ism: `let p = inv[part%]` reads record `part%` of the
3600     ' `inv` file into a local record variable `p` -- one expression
3610     ' for what fhb's `GET #1, PART!` plus five separate field reads
3620     ' (F$, D$, CVI(Q$), CVI(R$), CVS(P$)) did by hand. The write
3630     ' side, `inv[part%] = { ... }` (see editRecord() below), is the
3640     ' same sugar for PUT plus the LSET/MKx$ packing it replaces.
3650     ' let p = inv[...]  (whole-record read)
3660     GET #1, checkpartPart0%
3670     checkpartPFlagTrimI0% = LEN(invFlagBuf$)
3680     IF (checkpartPFlagTrimI0% > 0) = 0 THEN GOTO 3720
3690     IF (MID$(invFlagBuf$, checkpartPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 3720
3700         checkpartPFlagTrimI0% = checkpartPFlagTrimI0% - 1
3710         GOTO 3680
3720     REM END WHILE
3730     checkpartPFlag0$ = LEFT$(invFlagBuf$, checkpartPFlagTrimI0%)
3740     checkpartPDescTrimI0% = LEN(invDescBuf$)
3750     IF (checkpartPDescTrimI0% > 0) = 0 THEN GOTO 3790
3760     IF (MID$(invDescBuf$, checkpartPDescTrimI0%, 1) = " ") = 0 THEN GOTO 3790
3770         checkpartPDescTrimI0% = checkpartPDescTrimI0% - 1
3780         GOTO 3750
3790     REM END WHILE
3800     checkpartPDesc0$ = LEFT$(invDescBuf$, checkpartPDescTrimI0%)
3810     checkpartPQty0% = CVI(invQtyBuf$)
3820     checkpartPReorder0% = CVI(invReorderBuf$)
3830     checkpartPPrice0! = CVS(invPriceBuf$)
3840     isemptyFlag0$ = checkpartPFlag0$
3850     GOSUB 1620
3860     IF (isemptyResult0%) = 0 THEN GOTO 3920
3870         CLS
3880         LOCATE 10, 18
3890         PRINT ("Part number" + STR$(checkpartPart0%)) + "is still a null entry at this time"
3900         GOSUB 1870
3910         RETURN
3920     REM END IF
3930     showpartstatusPartNum0% = checkpartPart0%
3940     showpartstatusDesc0$ = checkpartPDesc0$
3950     showpartstatusQty0% = checkpartPQty0%
3960     showpartstatusReorder0% = checkpartPReorder0%
3970     showpartstatusPrice0! = checkpartPPrice0!
3980     GOSUB 2390
3990     GOSUB 1870
4000     RETURN
4010 ' end procedure checkpart

4020 ' procedure editrecord()
4030     CLS
4040     LOCATE 10, tabcol%
4050     GOSUB 1750
4060     editrecordPartStr0$ = readpartnumberinputResult0$
4070     editrecordPart0% = VAL(editrecordPartStr0$)
4080     partinrangeN0% = editrecordPart0%
4090     GOSUB 1660
4100     IF (partinrangeResult0% = 0) = 0 THEN GOTO 4140
4110         GOSUB 2210
4120         GOSUB 1870
4130         RETURN
4140     REM END IF
4150     ' let p = inv[...]  (whole-record read)
4160     GET #1, editrecordPart0%
4170     editrecordPFlagTrimI0% = LEN(invFlagBuf$)
4180     IF (editrecordPFlagTrimI0% > 0) = 0 THEN GOTO 4220
4190     IF (MID$(invFlagBuf$, editrecordPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 4220
4200         editrecordPFlagTrimI0% = editrecordPFlagTrimI0% - 1
4210         GOTO 4180
4220     REM END WHILE
4230     editrecordPFlag0$ = LEFT$(invFlagBuf$, editrecordPFlagTrimI0%)
4240     editrecordPDescTrimI0% = LEN(invDescBuf$)
4250     IF (editrecordPDescTrimI0% > 0) = 0 THEN GOTO 4290
4260     IF (MID$(invDescBuf$, editrecordPDescTrimI0%, 1) = " ") = 0 THEN GOTO 4290
4270         editrecordPDescTrimI0% = editrecordPDescTrimI0% - 1
4280         GOTO 4250
4290     REM END WHILE
4300     editrecordPDesc0$ = LEFT$(invDescBuf$, editrecordPDescTrimI0%)
4310     editrecordPQty0% = CVI(invQtyBuf$)
4320     editrecordPReorder0% = CVI(invReorderBuf$)
4330     editrecordPPrice0! = CVS(invPriceBuf$)
4340     isemptyFlag0$ = editrecordPFlag0$
4350     GOSUB 1620
4360     IF (isemptyResult0% = 0) = 0 THEN GOTO 4450
4370         LOCATE 12, tabcol%
4380         PRINT "Overwrite existing part data?"
4390         GOSUB 1800
4400         editrecordKp0$ = readkeyResult0$
4410         IF (editrecordKp0$ <> "Y") = 0 THEN GOTO 4440
4420         IF (editrecordKp0$ <> "y") = 0 THEN GOTO 4440
4430             RETURN
4440         REM END IF
4450     REM END IF

4460         gatherpartdetailsPartNum0% = editrecordPart0%
4470         gatherpartdetailsDesc0$ = editrecordEditDesc0$
4480         gatherpartdetailsQty0% = editrecordEditQty0%
4490         gatherpartdetailsReorder0% = editrecordEditReorder0%
4500         gatherpartdetailsPrice0! = editrecordEditPrice0!
4510         GOSUB 2810
4520         editrecordEditDesc0$ = gatherpartdetailsDesc0$
4530         editrecordEditQty0% = gatherpartdetailsQty0%
4540         editrecordEditReorder0% = gatherpartdetailsReorder0%
4550         editrecordEditPrice0! = gatherpartdetailsPrice0!
4560         GOSUB 1800
4570         editrecordKp0$ = readkeyResult0$
4580         IF (editrecordKp0$ = "Y") <> 0 THEN GOTO 4610
4590         IF (editrecordKp0$ = "y") <> 0 THEN GOTO 4610
4600         GOTO 4460
4610     REM END DO
4620     ' inv[...] = { ... }  (whole-record write)
4630     LSET invFlagBuf$ = "1"
4640     LSET invDescBuf$ = editrecordEditDesc0$
4650     LSET invQtyBuf$ = MKI$(editrecordEditQty0%)
4660     LSET invReorderBuf$ = MKI$(editrecordEditReorder0%)
4670     LSET invPriceBuf$ = MKS$(editrecordEditPrice0!)
4680     PUT #1, editrecordPart0%
4690     RETURN
4700 ' end procedure editrecord

4710 ' procedure listall()
4720     GOSUB 2540
4730     listallScrollCount0% = 0
4740     FOR listallI0% = 1 TO partcount%
4750         ' let p = inv[...]  (whole-record read)
4760         GET #1, listallI0%
4770         listallPFlagTrimI0% = LEN(invFlagBuf$)
4780         IF (listallPFlagTrimI0% > 0) = 0 THEN GOTO 4820
4790         IF (MID$(invFlagBuf$, listallPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 4820
4800             listallPFlagTrimI0% = listallPFlagTrimI0% - 1
4810             GOTO 4780
4820         REM END WHILE
4830         listallPFlag0$ = LEFT$(invFlagBuf$, listallPFlagTrimI0%)
4840         listallPDescTrimI0% = LEN(invDescBuf$)
4850         IF (listallPDescTrimI0% > 0) = 0 THEN GOTO 4890
4860         IF (MID$(invDescBuf$, listallPDescTrimI0%, 1) = " ") = 0 THEN GOTO 4890
4870             listallPDescTrimI0% = listallPDescTrimI0% - 1
4880             GOTO 4850
4890         REM END WHILE
4900         listallPDesc0$ = LEFT$(invDescBuf$, listallPDescTrimI0%)
4910         listallPQty0% = CVI(invQtyBuf$)
4920         listallPReorder0% = CVI(invReorderBuf$)
4930         listallPPrice0! = CVS(invPriceBuf$)
4940         printinventorylinePartNum0% = listallI0%
4950         printinventorylineDesc0$ = listallPDesc0$
4960         printinventorylineQty0% = listallPQty0%
4970         printinventorylineReorder0% = listallPReorder0%
4980         GOSUB 2630
4990         listallScrollCount0% = listallScrollCount0% + 1
5000         IF (listallScrollCount0% = 20) = 0 THEN GOTO 5030
5010             GOSUB 1870
5020             listallScrollCount0% = 0
5030         REM END IF
5040     NEXT listallI0%
5050     RETURN
5060 ' end procedure listall

5070 ' procedure addstock()
5080     CLS
5090     LOCATE 5, 25
5100     PRINT "A D D I N G   S T O C K"

5110         LOCATE 8, 25
5120         GOSUB 1750
5130         addstockPartStr0$ = readpartnumberinputResult0$
5140         addstockPart0% = VAL(addstockPartStr0$)
5150         partinrangeN0% = addstockPart0%
5160         GOSUB 1660
5170         addstockValidPart0% = partinrangeResult0%
5180         IF (addstockValidPart0% = 0) = 0 THEN GOTO 5210
5190             GOSUB 2270
5200             GOSUB 1800
5210         REM END IF
5220         IF (addstockValidPart0% <> 0) = 0 THEN GOTO 5110
5230     REM END DO

5240     ' let p = inv[...]  (whole-record read)
5250     GET #1, addstockPart0%
5260     addstockPFlagTrimI0% = LEN(invFlagBuf$)
5270     IF (addstockPFlagTrimI0% > 0) = 0 THEN GOTO 5310
5280     IF (MID$(invFlagBuf$, addstockPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 5310
5290         addstockPFlagTrimI0% = addstockPFlagTrimI0% - 1
5300         GOTO 5270
5310     REM END WHILE
5320     addstockPFlag0$ = LEFT$(invFlagBuf$, addstockPFlagTrimI0%)
5330     addstockPDescTrimI0% = LEN(invDescBuf$)
5340     IF (addstockPDescTrimI0% > 0) = 0 THEN GOTO 5380
5350     IF (MID$(invDescBuf$, addstockPDescTrimI0%, 1) = " ") = 0 THEN GOTO 5380
5360         addstockPDescTrimI0% = addstockPDescTrimI0% - 1
5370         GOTO 5340
5380     REM END WHILE
5390     addstockPDesc0$ = LEFT$(invDescBuf$, addstockPDescTrimI0%)
5400     addstockPQty0% = CVI(invQtyBuf$)
5410     addstockPReorder0% = CVI(invReorderBuf$)
5420     addstockPPrice0! = CVS(invPriceBuf$)
5430     isemptyFlag0$ = addstockPFlag0$
5440     GOSUB 1620
5450     IF (isemptyResult0%) = 0 THEN GOTO 5500
5460         shownullentrymessagePartStr0$ = addstockPartStr0$
5470         GOSUB 2340
5480         GOSUB 1800
5490         RETURN
5500     REM END IF

5510         showaddstockscreenPartNum0% = addstockPart0%
5520         showaddstockscreenDesc0$ = addstockPDesc0$
5530         showaddstockscreenQty0% = addstockPQty0%
5540         showaddstockscreenReorder0% = addstockPReorder0%
5550         GOSUB 3010
5560         LOCATE 14, tabcol%
5570         INPUT " Quantity to add"; addstockAddStr0$
5580         addstockAddAmt0% = VAL(addstockAddStr0$)
5590         IF (addstockAddAmt0% < 0) = 0 THEN GOTO 5620
5600             GOSUB 3170
5610             GOSUB 1800
5620         REM END IF
5630         IF (addstockAddAmt0% >= 0) = 0 THEN GOTO 5510
5640     REM END DO

5650     addstockPQty0% = addstockPQty0% + addstockAddAmt0%
5660     ' inv[...] = p  (write back a let-bound record)
5670     LSET invFlagBuf$ = addstockPFlag0$
5680     LSET invDescBuf$ = addstockPDesc0$
5690     LSET invQtyBuf$ = MKI$(addstockPQty0%)
5700     LSET invReorderBuf$ = MKI$(addstockPReorder0%)
5710     LSET invPriceBuf$ = MKS$(addstockPPrice0!)
5720     PUT #1, addstockPart0%
5730     RETURN
5740 ' end procedure addstock

5750 ' procedure subtractstock()
5760     CLS
5770     LOCATE 5, 20
5780     PRINT "S U B T R A C T I N G    S T O C K"

5790         LOCATE 8, 25
5800         GOSUB 1750
5810         subtractstockPartStr0$ = readpartnumberinputResult0$
5820         subtractstockPart0% = VAL(subtractstockPartStr0$)
5830         partinrangeN0% = subtractstockPart0%
5840         GOSUB 1660
5850         subtractstockValidPart0% = partinrangeResult0%
5860         IF (subtractstockValidPart0% = 0) = 0 THEN GOTO 5890
5870             GOSUB 2270
5880             GOSUB 1800
5890         REM END IF
5900         IF (subtractstockValidPart0% <> 0) = 0 THEN GOTO 5790
5910     REM END DO

5920     ' let p = inv[...]  (whole-record read)
5930     GET #1, subtractstockPart0%
5940     subtractstockPFlagTrimI0% = LEN(invFlagBuf$)
5950     IF (subtractstockPFlagTrimI0% > 0) = 0 THEN GOTO 5990
5960     IF (MID$(invFlagBuf$, subtractstockPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 5990
5970         subtractstockPFlagTrimI0% = subtractstockPFlagTrimI0% - 1
5980         GOTO 5950
5990     REM END WHILE
6000     subtractstockPFlag0$ = LEFT$(invFlagBuf$, subtractstockPFlagTrimI0%)
6010     subtractstockPDescTrimI0% = LEN(invDescBuf$)
6020     IF (subtractstockPDescTrimI0% > 0) = 0 THEN GOTO 6060
6030     IF (MID$(invDescBuf$, subtractstockPDescTrimI0%, 1) = " ") = 0 THEN GOTO 6060
6040         subtractstockPDescTrimI0% = subtractstockPDescTrimI0% - 1
6050         GOTO 6020
6060     REM END WHILE
6070     subtractstockPDesc0$ = LEFT$(invDescBuf$, subtractstockPDescTrimI0%)
6080     subtractstockPQty0% = CVI(invQtyBuf$)
6090     subtractstockPReorder0% = CVI(invReorderBuf$)
6100     subtractstockPPrice0! = CVS(invPriceBuf$)
6110     isemptyFlag0$ = subtractstockPFlag0$
6120     GOSUB 1620
6130     IF (isemptyResult0%) = 0 THEN GOTO 6180
6140         shownullentrymessagePartStr0$ = subtractstockPartStr0$
6150         GOSUB 2340
6160         GOSUB 1800
6170         RETURN
6180     REM END IF

6190         showsubtractstockscreenPartNum0% = subtractstockPart0%
6200         showsubtractstockscreenDesc0$ = subtractstockPDesc0$
6210         showsubtractstockscreenQty0% = subtractstockPQty0%
6220         showsubtractstockscreenReorder0% = subtractstockPReorder0%
6230         GOSUB 3240
6240         LOCATE 14, tabcol%
6250         INPUT "Quantity to subtract"; subtractstockSubStr0$
6260         subtractstockSubAmt0% = VAL(subtractstockSubStr0$)
6270         subtractstockOverSubtract0% = 0
6280         IF (subtractstockSubAmt0% >= 0) = 0 THEN GOTO 6340
6290         IF ((subtractstockPQty0% - subtractstockSubAmt0%) < 0) = 0 THEN GOTO 6340
6300             subtractstockOverSubtract0% = 1
6310             showoversubtractwarningOnHand0% = subtractstockPQty0%
6320             GOSUB 3400
6330             GOSUB 1800
6340         REM END IF
6350         IF (subtractstockSubAmt0% >= 0) = 0 THEN GOTO 6190
6360         IF (subtractstockOverSubtract0% = 0) = 0 THEN GOTO 6190
6370     REM END DO

6380     subtractstockPQty0% = subtractstockPQty0% - subtractstockSubAmt0%
6390     IF (subtractstockPQty0% <= subtractstockPReorder0%) = 0 THEN GOTO 6410
6400         LOCATE 16, tabcol%
6410     REM END IF
6420     PRINT (("quantity now" + STR$(subtractstockPQty0%)) + " reorder level") + STR$(subtractstockPReorder0%)
6430     ' inv[...] = p  (write back a let-bound record)
6440     LSET invFlagBuf$ = subtractstockPFlag0$
6450     LSET invDescBuf$ = subtractstockPDesc0$
6460     LSET invQtyBuf$ = MKI$(subtractstockPQty0%)
6470     LSET invReorderBuf$ = MKI$(subtractstockPReorder0%)
6480     LSET invPriceBuf$ = MKS$(subtractstockPPrice0!)
6490     PUT #1, subtractstockPart0%
6500     RETURN
6510 ' end procedure subtractstock

6520 ' procedure reorderreport()
6530     GOSUB 2670
6540     reorderreportReportLineCount0% = 0
6550     FOR reorderreportI0% = 1 TO partcount%
6560         ' let p = inv[...]  (whole-record read)
6570         GET #1, reorderreportI0%
6580         reorderreportPFlagTrimI0% = LEN(invFlagBuf$)
6590         IF (reorderreportPFlagTrimI0% > 0) = 0 THEN GOTO 6630
6600         IF (MID$(invFlagBuf$, reorderreportPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 6630
6610             reorderreportPFlagTrimI0% = reorderreportPFlagTrimI0% - 1
6620             GOTO 6590
6630         REM END WHILE
6640         reorderreportPFlag0$ = LEFT$(invFlagBuf$, reorderreportPFlagTrimI0%)
6650         reorderreportPDescTrimI0% = LEN(invDescBuf$)
6660         IF (reorderreportPDescTrimI0% > 0) = 0 THEN GOTO 6700
6670         IF (MID$(invDescBuf$, reorderreportPDescTrimI0%, 1) = " ") = 0 THEN GOTO 6700
6680             reorderreportPDescTrimI0% = reorderreportPDescTrimI0% - 1
6690             GOTO 6660
6700         REM END WHILE
6710         reorderreportPDesc0$ = LEFT$(invDescBuf$, reorderreportPDescTrimI0%)
6720         reorderreportPQty0% = CVI(invQtyBuf$)
6730         reorderreportPReorder0% = CVI(invReorderBuf$)
6740         reorderreportPPrice0! = CVS(invPriceBuf$)
6750         IF (reorderreportPQty0% < reorderreportPReorder0%) = 0 THEN GOTO 6860
6760             printreorderlinePartNum0% = reorderreportI0%
6770             printreorderlineDesc0$ = reorderreportPDesc0$
6780             printreorderlineQty0% = reorderreportPQty0%
6790             printreorderlineReorder0% = reorderreportPReorder0%
6800             GOSUB 2770
6810             reorderreportReportLineCount0% = reorderreportReportLineCount0% + 1
6820             IF (reorderreportReportLineCount0% > 15) = 0 THEN GOTO 6850
6830                 GOSUB 1870
6840                 reorderreportReportLineCount0% = 0
6850             REM END IF
6860         REM END IF
6870     NEXT reorderreportI0%
6880     GOSUB 1870
6890     RETURN
6900 ' end procedure reorderreport
