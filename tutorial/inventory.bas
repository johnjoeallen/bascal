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
470 ' blank, CHR$(255)-flagged records) isn't reproduced here --
480 ' inven.dat must be pre-populated with 100 such blank records
490 ' before running this program, or isEmpty%() will read
500 ' uninitialized/zero-filled records as never-empty.
510 ' - The three original tab-position constants (T=20, U=25,
520 ' V=30) are collapsed into a single `tabCol% = 20`; a couple of
530 ' screens that used U=25 in the original (see showAddStockScreen
540 ' below) keep 25 as a literal rather than reusing tabCol%.
550 ' 
560 ' Tracks parts in a fixed 100-record file: check status, add,
570 ' edit, add/subtract stock, and a reorder report.
580 ' 
590 ' Error handling uses try/catch (GitHub issue #60), not the raw `on
600 ' error goto` / `resume next` fhb's original relies on: a failed menu
610 ' action is abandoned outright and the program returns straight to the
620 ' main menu, rather than resuming at the exact instruction after
630 ' whatever failed -- see reportInventoryError() below and
640 ' tutorial/inventory_try_catch.draft's own header comment for why. This
650 ' is a real, deliberate behavior change from an earlier on-error-goto
660 ' version of this file, which *was* verified against real BASCOM 2.00
670 ' under dosbox-x (only with the /E and /X switches -- error trapping
680 ' isn't linked in by default); the try/catch shape below transpiles to
690 ' the same ON ERROR GOTO/RESUME primitives BASCOM accepts, but hasn't
700 ' itself been independently re-verified against a real BASCOM compile.
710 ' ============================================================

720 ' BASCAL-ism: the record/file DSL. `record ... end record` plus
730 ' `file ... as ... = open(...)` below replace fhb's manual
740 ' FIELD #1,1 AS F$,30 AS D$,2 AS Q$,... buffer layout entirely --
750 ' bcc computes the field widths and record LEN from this
760 ' declaration and generates the FIELD statement itself. Named
770 ' field access (`p.flag`, `p.qty`, ...) and whole-record
780 ' read/write via `inv[n]` (see checkPart() below) replace fhb's
790 ' manual GET/PUT plus LSET/RSET and MKI$/MKS$/CVI$/CVS$ packing.

800 ' BASCAL-ism: `const` is a real compile-time constant, not a plain
810 ' variable assignment like fhb's `N=100` / `T=20` -- it can never
820 ' be reassigned, and resolves to the same value everywhere,
830 ' including inside every function/procedure below, with no
840 ' `global` declaration needed.
850 partcount% = 100
860 tabcol% = 20

870 ' `file ... = open(...)` is sugar for OPEN ... FOR RANDOM AS #n
880 ' LEN = <record width> plus the FIELD statement fhb wrote out by
890 ' hand at his line 550.
900 ' file inv as Part = open(...)  [39 bytes/record]
910 OPEN "inven.dat" FOR RANDOM AS #1 LEN = 39
920 FIELD #1, 1 AS invFlagBuf$, 30 AS invDescBuf$, 2 AS invQtyBuf$, 2 AS invReorderBuf$, 4 AS invPriceBuf$

930 ' -------------------- Pure functions (no file access) --------------------

940 ' BASCAL-ism: `function ... end function` with `return` replaces
950 ' fhb's convention of a GOSUB target plus a bare RETURN -- there's
960 ' no separate "subroutine label" and no shared/global result
970 ' variable to manage by hand; `isEmpty%(...)` is called like an
980 ' ordinary expression at every use below (e.g. `isEmpty%(p.flag)`).
990 ' A record whose flag byte is CHR$(255) is an empty/never-used slot.

1000 ' BASCAL-ism: `&&` and `||` are short-circuit AND/OR -- real
1010 ' MBASIC/BASCOM only has bitwise AND/OR (which fhb relies on here
1020 ' too, since `PART!<1 OR PART!>N!` never short-circuits anyway).
1030 ' BASCAL lowers `&&`/`||` into the equivalent branching so the
1040 ' short-circuit *is* real at the generated-BASIC level; see the
1050 ' manual's "Short-Circuit && and ||" section
1060 ' (https://johnjoeallen.github.io/bascal/manual/).

1070 ' -------------------- Keyboard input --------------------

1080 ' BASCAL-ism: `do ... loop until` is a structured post-check loop
1090 ' replacing fhb's `730 KP$=INKEY$:IF KP$="" THEN 730` GOTO-polling
1100 ' idiom. `inkey$` itself is the real INKEY$ builtin passed straight
1110 ' through, resolving correctly from inside a function/procedure
1120 ' body like this one -- every menu action below calls
1130 ' readKey$()/waitAnyKey() rather than polling INKEY$ inline.

1140 ' -------------------- Display procedures --------------------

1150 ' byref scalar parameters: gatherPartDetails writes the four editable
1160 ' fields for a part directly back into the caller's variables.

1170 ' -------------------- Menu actions --------------------

1180 ' -------------------- Program entry --------------------

1190 CLS

1200     GOSUB 3710
1210     GOSUB 3560
1220     kp$ = readkeyResult0$
1230     IF (INSTR("12345678cCeElLaAsSrRqQxX", kp$) <> 0) = 0 THEN GOTO 1780
1240         ' BASCAL-ism: `select case` replaces fhb's chain of eight
1250         ' `IF VAL(KP$)=n OR KP$="x" OR KP$="X" THEN GOTO ...` lines
1260         ' (his 770-840) with one multi-way dispatch.
1270         ' 
1280         ' BASCAL-ism: `try`/`catch` (issue #60) replaces fhb's own global
1290         ' `ON ERROR GOTO` trap. A failed menu action is abandoned outright
1300         ' here -- the `catch` below runs, then execution continues right
1310         ' after `end try`, back at `loop until` -- rather than resuming at
1320         ' the exact instruction after whatever failed inside checkPart()/
1330         ' editRecord()/etc. the way fhb's `RESUME NEXT` did. See
1340         ' reportInventoryError() below and tutorial/inventory_try_catch.
1350         ' draft's own header comment for why that arbitrary resume-point
1360         ' behavior isn't something try/catch reproduces.
1370         ON ERROR GOTO 1710
1380             BCCT5$ = kp$
1390             IF (BCCT5$ = "1" OR BCCT5$ = "c" OR BCCT5$ = "C") <> 0 THEN GOTO 1480
1400             IF (BCCT5$ = "2" OR BCCT5$ = "e" OR BCCT5$ = "E") <> 0 THEN GOTO 1500
1410             IF (BCCT5$ = "3" OR BCCT5$ = "l" OR BCCT5$ = "L") <> 0 THEN GOTO 1520
1420             IF (BCCT5$ = "4" OR BCCT5$ = "a" OR BCCT5$ = "A") <> 0 THEN GOTO 1540
1430             IF (BCCT5$ = "5" OR BCCT5$ = "s" OR BCCT5$ = "S") <> 0 THEN GOTO 1560
1440             IF (BCCT5$ = "6" OR BCCT5$ = "r" OR BCCT5$ = "R") <> 0 THEN GOTO 1580
1450             IF (BCCT5$ = "7" OR BCCT5$ = "q" OR BCCT5$ = "Q") <> 0 THEN GOTO 1600
1460             IF (BCCT5$ = "8" OR BCCT5$ = "x" OR BCCT5$ = "X") <> 0 THEN GOTO 1620
1470             GOTO 1680
1480                 GOSUB 5250
1490                 GOTO 1680
1500                 GOSUB 5790
1510                 GOTO 1680
1520                 GOSUB 6480
1530                 GOTO 1680
1540                 GOSUB 6840
1550                 GOTO 1680
1560                 GOSUB 7520
1570                 GOTO 1680
1580                 GOSUB 8290
1590                 GOTO 1680
1600                 quitflag% = 1
1610                 GOTO 1680
1620                 ' BASCAL-ism: `inv.close()` is sugar for `CLOSE #1`,
1630                 ' matching fhb's own `90 CLOSE:SYSTEM`.
1640                 ' inv.close()
1650                 CLOSE #1
1660                 SYSTEM
1670                 GOTO 1680
1680             REM END SELECT
1690         ON ERROR GOTO 0
1700         GOTO 1770
1710             err% = ERR
1720             erl% = ERL
1730             reportinventoryerrorErr0% = err%
1740             reportinventoryerrorErl0% = erl%
1750             GOSUB 8680
1760             RESUME 1770
1770         REM END TRY
1780     REM END IF
1790     IF (quitflag% = 1) = 0 THEN GOTO 1200
1800 REM END DO

1810 ' inv.close()
1820 CLOSE #1
1830 END

1840 ' -------------------- Error handling --------------------
1850 ' err%/erl% are ordinary locals scoped to the `catch` block above, not
1860 ' aliases for the ambient (readable-anywhere) `err`/`erl` pseudo-
1870 ' variables `on error goto` uses -- see `Statement::TryCatch`'s own doc
1880 ' comment in ast.rs. Passed straight through to ERROR$ here like fhb's
1890 ' own ERR/ERL (his 3390: "an error on line";ERL), decoded through
1900 ' BASCAL's own com.bascal.stdlib.error (ERROR$) instead of fhb's
1910 ' hand-rolled lookup table -- see the header note above. try/catch
1920 ' itself isn't documented in the manual yet (GitHub issue #60 tracks
1930 ' the still-unfinished C-target work; the manual page can follow once
1940 ' that lands) -- see ast.rs's own `Statement::TryCatch` doc comment for
1950 ' the full semantics meanwhile.
1960 END

1970 ' function error$(code%)
1980     BCCT7% = errorCode0%
1990     IF (BCCT7% = 2) <> 0 THEN GOTO 2330
2000     IF (BCCT7% = 3) <> 0 THEN GOTO 2360
2010     IF (BCCT7% = 4) <> 0 THEN GOTO 2390
2020     IF (BCCT7% = 5) <> 0 THEN GOTO 2420
2030     IF (BCCT7% = 6) <> 0 THEN GOTO 2450
2040     IF (BCCT7% = 7) <> 0 THEN GOTO 2480
2050     IF (BCCT7% = 9) <> 0 THEN GOTO 2510
2060     IF (BCCT7% = 10) <> 0 THEN GOTO 2540
2070     IF (BCCT7% = 11) <> 0 THEN GOTO 2570
2080     IF (BCCT7% = 13) <> 0 THEN GOTO 2600
2090     IF (BCCT7% = 14) <> 0 THEN GOTO 2630
2100     IF (BCCT7% = 19) <> 0 THEN GOTO 2660
2110     IF (BCCT7% = 20) <> 0 THEN GOTO 2690
2120     IF (BCCT7% = 24) <> 0 THEN GOTO 2720
2130     IF (BCCT7% = 25) <> 0 THEN GOTO 2750
2140     IF (BCCT7% = 27) <> 0 THEN GOTO 2780
2150     IF (BCCT7% = 52) <> 0 THEN GOTO 2810
2160     IF (BCCT7% = 53) <> 0 THEN GOTO 2840
2170     IF (BCCT7% = 54) <> 0 THEN GOTO 2870
2180     IF (BCCT7% = 55) <> 0 THEN GOTO 2900
2190     IF (BCCT7% = 57) <> 0 THEN GOTO 2930
2200     IF (BCCT7% = 58) <> 0 THEN GOTO 2960
2210     IF (BCCT7% = 61) <> 0 THEN GOTO 2990
2220     IF (BCCT7% = 62) <> 0 THEN GOTO 3020
2230     IF (BCCT7% = 63) <> 0 THEN GOTO 3050
2240     IF (BCCT7% = 64) <> 0 THEN GOTO 3080
2250     IF (BCCT7% = 67) <> 0 THEN GOTO 3110
2260     IF (BCCT7% = 68) <> 0 THEN GOTO 3140
2270     IF (BCCT7% = 70) <> 0 THEN GOTO 3170
2280     IF (BCCT7% = 71) <> 0 THEN GOTO 3200
2290     IF (BCCT7% = 72) <> 0 THEN GOTO 3230
2300     IF (BCCT7% = 75) <> 0 THEN GOTO 3260
2310     IF (BCCT7% = 76) <> 0 THEN GOTO 3290
2320     GOTO 3320
2330         errorResult0$ = "Syntax error"
2340         RETURN
2350         GOTO 3340
2360         errorResult0$ = "RETURN without GOSUB"
2370         RETURN
2380         GOTO 3340
2390         errorResult0$ = "Out of DATA"
2400         RETURN
2410         GOTO 3340
2420         errorResult0$ = "Illegal function call"
2430         RETURN
2440         GOTO 3340
2450         errorResult0$ = "Overflow"
2460         RETURN
2470         GOTO 3340
2480         errorResult0$ = "Out of memory"
2490         RETURN
2500         GOTO 3340
2510         errorResult0$ = "Subscript out of range"
2520         RETURN
2530         GOTO 3340
2540         errorResult0$ = "Duplicate Definition"
2550         RETURN
2560         GOTO 3340
2570         errorResult0$ = "Division by zero"
2580         RETURN
2590         GOTO 3340
2600         errorResult0$ = "Type mismatch"
2610         RETURN
2620         GOTO 3340
2630         errorResult0$ = "Out of string space"
2640         RETURN
2650         GOTO 3340
2660         errorResult0$ = "No RESUME"
2670         RETURN
2680         GOTO 3340
2690         errorResult0$ = "RESUME without error"
2700         RETURN
2710         GOTO 3340
2720         errorResult0$ = "Device timeout"
2730         RETURN
2740         GOTO 3340
2750         errorResult0$ = "Device fault"
2760         RETURN
2770         GOTO 3340
2780         errorResult0$ = "Out of paper"
2790         RETURN
2800         GOTO 3340
2810         errorResult0$ = "Bad file number"
2820         RETURN
2830         GOTO 3340
2840         errorResult0$ = "File not found"
2850         RETURN
2860         GOTO 3340
2870         errorResult0$ = "Bad file mode"
2880         RETURN
2890         GOTO 3340
2900         errorResult0$ = "File already open"
2910         RETURN
2920         GOTO 3340
2930         errorResult0$ = "Device I/O error"
2940         RETURN
2950         GOTO 3340
2960         errorResult0$ = "File already exists"
2970         RETURN
2980         GOTO 3340
2990         errorResult0$ = "Disk full"
3000         RETURN
3010         GOTO 3340
3020         errorResult0$ = "Input past end"
3030         RETURN
3040         GOTO 3340
3050         errorResult0$ = "Bad record number"
3060         RETURN
3070         GOTO 3340
3080         errorResult0$ = "Bad file name"
3090         RETURN
3100         GOTO 3340
3110         errorResult0$ = "Too many files"
3120         RETURN
3130         GOTO 3340
3140         errorResult0$ = "Device unavailable"
3150         RETURN
3160         GOTO 3340
3170         errorResult0$ = "Disk write protected"
3180         RETURN
3190         GOTO 3340
3200         errorResult0$ = "Disk not ready"
3210         RETURN
3220         GOTO 3340
3230         errorResult0$ = "Disk media error"
3240         RETURN
3250         GOTO 3340
3260         errorResult0$ = "Path/File access error"
3270         RETURN
3280         GOTO 3340
3290         errorResult0$ = "Path not found"
3300         RETURN
3310         GOTO 3340
3320         errorResult0$ = "Error " + STR$(errorCode0%)
3330         RETURN
3340     REM END SELECT
3350     RETURN
3360 ' end function error$

3370 ' function isempty%(flag$)
3380     isemptyResult0% = ASC(isemptyFlag0$) = 255
3390     RETURN
3400 ' end function isempty%

3410 ' function partinrange%(n%)
3420     IF (partinrangeN0% >= 1) = 0 THEN GOTO 3460
3430     IF (partinrangeN0% <= partcount%) = 0 THEN GOTO 3460
3440         partinrangeResult0% = 1
3450         RETURN
3460     REM END IF
3470     partinrangeResult0% = 0
3480     RETURN
3490 ' end function partinrange%

3500 ' function readpartnumberinput$()
3510     INPUT "Input part number"; readpartnumberinputS0$
3520     readpartnumberinputResult0$ = readpartnumberinputS0$
3530     RETURN
3540 ' end function readpartnumberinput$

3550 ' function readkey$()
3560         readkeyK0$ = INKEY$
3570         IF (readkeyK0$ <> "") = 0 THEN GOTO 3560
3580     REM END DO
3590     readkeyResult0$ = readkeyK0$
3600     RETURN
3610 ' end function readkey$

3620 ' procedure waitanykey()
3630     LOCATE 25, 10
3640     PRINT "Press the AnyKey to continue...";
3650         waitanykeyK0$ = INKEY$
3660         IF (waitanykeyK0$ <> "") = 0 THEN GOTO 3650
3670     REM END DO
3680     RETURN
3690 ' end procedure waitanykey

3700 ' procedure showmainmenu()
3710     CLS
3720     COLOR 14, 4
3730     CLS
3740     LOCATE 6, 1
3750     PRINT
3760     ' `tab(n)` passes straight through to real TAB(n), same as
3770     ' fhb's own `PRINT TAB(V) "..."` -- but only as a bare item in
3780     ' a PRINT list, juxtaposed or `;`-separated like here. Real
3790     ' BASCOM rejects `"literal" + tab(n) + ...` (TAB isn't a real
3800     ' string function you can concatenate); see printListHeader()
3810     ' and printReorderHeader() below, which need `;` between a
3820     ' preceding string and a `tab(n)` for exactly this reason.
3830     PRINT TAB(30)"Inventory Program"
3840     PRINT
3850     PRINT TAB(tabcol%)"1......C)heck a part"
3860     PRINT TAB(tabcol%)"2......E)dit/overwrite/add a part"
3870     PRINT TAB(tabcol%)("3......L)ist all" + STR$(partcount%)) + "parts"
3880     PRINT TAB(tabcol%)"4......A)dd stock"
3890     PRINT TAB(tabcol%)"5......S)ubtract stock"
3900     PRINT TAB(tabcol%)"6......R)eorder Report"
3910     PRINT
3920     PRINT TAB(tabcol%)"7......Q)uit to BASIC"
3930     PRINT TAB(tabcol%)"8......eX)it to system"
3940     RETURN
3950 ' end procedure showmainmenu

3960 ' procedure showbadpartnumber()
3970     CLS
3980     LOCATE 10, 10
3990     PRINT "Part number is out of permissable range of 1 to" + STR$(partcount%)
4000     RETURN
4010 ' end procedure showbadpartnumber

4020 ' procedure showrangeretrymessage()
4030     LOCATE 10, 15
4040     PRINT "The Part number is out of permissable range of 1 to" + STR$(partcount%)
4050     LOCATE 25, 15
4060     PRINT "Press the Anykey to reenter part number...";
4070     RETURN
4080 ' end procedure showrangeretrymessage

4090 ' procedure shownullentrymessage(partstr$)
4100     LOCATE 10, tabcol%
4110     PRINT ("Part number " + shownullentrymessagePartStr0$) + " is a null entry"
4120     RETURN
4130 ' end procedure shownullentrymessage

4140 ' procedure showpartstatus(partnum%, desc$, qty%, reorder%, price!)
4150     CLS
4160     LOCATE 5, 1
4170     PRINT TAB(tabcol%)"Inventory Status for Individual Part Number"
4180     PRINT TAB(tabcol%)"==========================================="
4190     PRINT
4200     PRINT
4210     PRINT TAB(tabcol%)"     Part number:  " + STR$(showpartstatusPartNum0%)
4220     PRINT
4230     PRINT TAB(tabcol%)"       Item name:  " + showpartstatusDesc0$
4240     PRINT TAB(tabcol%)"Quantity on hand:  " + STR$(showpartstatusQty0%)
4250     PRINT TAB(tabcol%)"   Reorder level:  " + STR$(showpartstatusReorder0%)
4260     PRINT TAB(tabcol%)"      Unit price:  " + STR$(showpartstatusPrice0!)
4270     RETURN
4280 ' end procedure showpartstatus

4290 ' procedure printlistheader()
4300     CLS
4310     PRINT TAB(25)"I N V E N T O R Y   L I S T I N G"; TAB(65); STR$(partcount%) + "items"
4320     PRINT "                                          Quantity       Reorder"
4330     PRINT " Partno           Description             on hand         level"
4340     LOCATE 25, 1
4350     PRINT "Press the AnyKey to scroll listing...";
4360     RETURN
4370 ' end procedure printlistheader

4380 ' procedure printinventoryline(partnum%, desc$, qty%, reorder%)
4390     PRINT (((((STR$(printinventorylinePartNum0%) + "  ") + printinventorylineDesc0$) + "   ") + STR$(printinventorylineQty0%)) + "          ") + STR$(printinventorylineReorder0%)
4400     RETURN
4410 ' end procedure printinventoryline

4420 ' procedure printreorderheader()
4430     CLS
4440     LOCATE 1, tabcol%
4450     PRINT "Reorder Report"; TAB(55); DATE$
4460     PRINT
4470     PRINT "                                             Quantity       Reorder"
4480     PRINT "    Partno           Description             on hand         level"
4490     PRINT "   =======  ==============================   ========       ======="
4500     RETURN
4510 ' end procedure printreorderheader

4520 ' procedure printreorderline(partnum%, desc$, qty%, reorder%)
4530     PRINT (((((("  " + STR$(printreorderlinePartNum0%)) + "  ") + printreorderlineDesc0$) + "   ") + STR$(printreorderlineQty0%)) + "          ") + STR$(printreorderlineReorder0%)
4540     RETURN
4550 ' end procedure printreorderline

4560 ' procedure gatherpartdetails(partnum%, desc$, qty%, reorder%, price!)
4570     CLS
4580     LOCATE 4, tabcol%
4590     PRINT "Adding or Overwriting a Record"
4600     LOCATE 8, tabcol%
4610     PRINT "Record/Partno" + STR$(gatherpartdetailsPartNum0%)
4620     LOCATE 11, 39
4630     PRINT "------------------------------"
4640     LOCATE 10, tabcol%
4650     INPUT "      Description"; gatherpartdetailsDesc0$
4660     LOCATE 12, tabcol%
4670     INPUT "Quantity in stock"; gatherpartdetailsQty0%
4680     LOCATE 14, tabcol%
4690     INPUT "    Reorder level"; gatherpartdetailsReorder0%
4700     LOCATE 16, tabcol%
4710     INPUT "       Unit price"; gatherpartdetailsPrice0!
4720     LOCATE 18, tabcol%
4730     PRINT "Is information correct (Y/N)?"
4740     RETURN
4750 ' end procedure gatherpartdetails

4760 ' procedure showaddstockscreen(partnum%, desc$, qty%, reorder%)
4770     CLS
4780     LOCATE 4, 25
4790     PRINT "Add to an inventory part number"
4800     LOCATE 5, 25
4810     PRINT "==============================="
4820     LOCATE 8, tabcol%
4830     PRINT "     Part number: " + STR$(showaddstockscreenPartNum0%)
4840     LOCATE 9, tabcol%
4850     PRINT "Item description: " + showaddstockscreenDesc0$
4860     LOCATE 10, tabcol%
4870     PRINT "Quantity on hand: " + STR$(showaddstockscreenQty0%)
4880     LOCATE 11, tabcol%
4890     PRINT "   Reorder Level: " + STR$(showaddstockscreenReorder0%)
4900     RETURN
4910 ' end procedure showaddstockscreen

4920 ' procedure shownegativeqtywarning()
4930     LOCATE 17, 15
4940     PRINT "The quantity to add must NOT be a negative number"
4950     LOCATE 25, 1
4960     PRINT "Please press the Anykey to reenter quantity to add...";
4970     RETURN
4980 ' end procedure shownegativeqtywarning

4990 ' procedure showsubtractstockscreen(partnum%, desc$, qty%, reorder%)
5000     CLS
5010     LOCATE 4, tabcol%
5020     PRINT "Subtract an inventory part number"
5030     LOCATE 5, tabcol%
5040     PRINT "================================="
5050     LOCATE 8, tabcol%
5060     PRINT "         Part number: " + STR$(showsubtractstockscreenPartNum0%)
5070     LOCATE 9, tabcol%
5080     PRINT "    Item description: " + showsubtractstockscreenDesc0$
5090     LOCATE 10, tabcol%
5100     PRINT "    Quantity on hand: " + STR$(showsubtractstockscreenQty0%)
5110     LOCATE 11, tabcol%
5120     PRINT "       Reorder Level: " + STR$(showsubtractstockscreenReorder0%)
5130     RETURN
5140 ' end procedure showsubtractstockscreen

5150 ' procedure showoversubtractwarning(onhand%)
5160     LOCATE 17, 5
5170     PRINT "The quantity to SUBTRACT must NOT result in NEGATIVE inventory"
5180     LOCATE 18, 5
5190     PRINT ("Only" + STR$(showoversubtractwarningOnHand0%)) + " IN STOCK"
5200     LOCATE 25, 1
5210     PRINT "Please press the Anykey to reenter quantity to subtract...";
5220     RETURN
5230 ' end procedure showoversubtractwarning

5240 ' procedure checkpart()
5250     GOSUB 3510
5260     checkpartPartStr0$ = readpartnumberinputResult0$
5270     checkpartPart0% = VAL(checkpartPartStr0$)
5280     partinrangeN0% = checkpartPart0%
5290     GOSUB 3420
5300     IF (partinrangeResult0% = 0) = 0 THEN GOTO 5340
5310         GOSUB 3970
5320         GOSUB 3630
5330         RETURN
5340     REM END IF
5350     ' BASCAL-ism: `let p = inv[part%]` reads record `part%` of the
5360     ' `inv` file into a local record variable `p` -- one expression
5370     ' for what fhb's `GET #1, PART!` plus five separate field reads
5380     ' (F$, D$, CVI(Q$), CVI(R$), CVS(P$)) did by hand. The write
5390     ' side, `inv[part%] = { ... }` (see editRecord() below), is the
5400     ' same sugar for PUT plus the LSET/MKx$ packing it replaces.
5410     ' let p = inv[...]  (whole-record read)
5420     GET #1, checkpartPart0%
5430     checkpartPFlagTrimI0% = LEN(invFlagBuf$)
5440     IF (checkpartPFlagTrimI0% > 0) = 0 THEN GOTO 5480
5450     IF (MID$(invFlagBuf$, checkpartPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 5480
5460         checkpartPFlagTrimI0% = checkpartPFlagTrimI0% - 1
5470         GOTO 5440
5480     REM END WHILE
5490     checkpartPFlag0$ = LEFT$(invFlagBuf$, checkpartPFlagTrimI0%)
5500     checkpartPDescTrimI0% = LEN(invDescBuf$)
5510     IF (checkpartPDescTrimI0% > 0) = 0 THEN GOTO 5550
5520     IF (MID$(invDescBuf$, checkpartPDescTrimI0%, 1) = " ") = 0 THEN GOTO 5550
5530         checkpartPDescTrimI0% = checkpartPDescTrimI0% - 1
5540         GOTO 5510
5550     REM END WHILE
5560     checkpartPDesc0$ = LEFT$(invDescBuf$, checkpartPDescTrimI0%)
5570     checkpartPQty0% = CVI(invQtyBuf$)
5580     checkpartPReorder0% = CVI(invReorderBuf$)
5590     checkpartPPrice0! = CVS(invPriceBuf$)
5600     isemptyFlag0$ = checkpartPFlag0$
5610     GOSUB 3380
5620     IF (isemptyResult0%) = 0 THEN GOTO 5680
5630         CLS
5640         LOCATE 10, 18
5650         PRINT ("Part number" + STR$(checkpartPart0%)) + "is still a null entry at this time"
5660         GOSUB 3630
5670         RETURN
5680     REM END IF
5690     showpartstatusPartNum0% = checkpartPart0%
5700     showpartstatusDesc0$ = checkpartPDesc0$
5710     showpartstatusQty0% = checkpartPQty0%
5720     showpartstatusReorder0% = checkpartPReorder0%
5730     showpartstatusPrice0! = checkpartPPrice0!
5740     GOSUB 4150
5750     GOSUB 3630
5760     RETURN
5770 ' end procedure checkpart

5780 ' procedure editrecord()
5790     CLS
5800     LOCATE 10, tabcol%
5810     GOSUB 3510
5820     editrecordPartStr0$ = readpartnumberinputResult0$
5830     editrecordPart0% = VAL(editrecordPartStr0$)
5840     partinrangeN0% = editrecordPart0%
5850     GOSUB 3420
5860     IF (partinrangeResult0% = 0) = 0 THEN GOTO 5900
5870         GOSUB 3970
5880         GOSUB 3630
5890         RETURN
5900     REM END IF
5910     ' let p = inv[...]  (whole-record read)
5920     GET #1, editrecordPart0%
5930     editrecordPFlagTrimI0% = LEN(invFlagBuf$)
5940     IF (editrecordPFlagTrimI0% > 0) = 0 THEN GOTO 5980
5950     IF (MID$(invFlagBuf$, editrecordPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 5980
5960         editrecordPFlagTrimI0% = editrecordPFlagTrimI0% - 1
5970         GOTO 5940
5980     REM END WHILE
5990     editrecordPFlag0$ = LEFT$(invFlagBuf$, editrecordPFlagTrimI0%)
6000     editrecordPDescTrimI0% = LEN(invDescBuf$)
6010     IF (editrecordPDescTrimI0% > 0) = 0 THEN GOTO 6050
6020     IF (MID$(invDescBuf$, editrecordPDescTrimI0%, 1) = " ") = 0 THEN GOTO 6050
6030         editrecordPDescTrimI0% = editrecordPDescTrimI0% - 1
6040         GOTO 6010
6050     REM END WHILE
6060     editrecordPDesc0$ = LEFT$(invDescBuf$, editrecordPDescTrimI0%)
6070     editrecordPQty0% = CVI(invQtyBuf$)
6080     editrecordPReorder0% = CVI(invReorderBuf$)
6090     editrecordPPrice0! = CVS(invPriceBuf$)
6100     isemptyFlag0$ = editrecordPFlag0$
6110     GOSUB 3380
6120     IF (isemptyResult0% = 0) = 0 THEN GOTO 6210
6130         LOCATE 12, tabcol%
6140         PRINT "Overwrite existing part data?"
6150         GOSUB 3560
6160         editrecordKp0$ = readkeyResult0$
6170         IF (editrecordKp0$ <> "Y") = 0 THEN GOTO 6200
6180         IF (editrecordKp0$ <> "y") = 0 THEN GOTO 6200
6190             RETURN
6200         REM END IF
6210     REM END IF

6220         gatherpartdetailsPartNum0% = editrecordPart0%
6230         gatherpartdetailsDesc0$ = editrecordEditDesc0$
6240         gatherpartdetailsQty0% = editrecordEditQty0%
6250         gatherpartdetailsReorder0% = editrecordEditReorder0%
6260         gatherpartdetailsPrice0! = editrecordEditPrice0!
6270         GOSUB 4570
6280         editrecordEditDesc0$ = gatherpartdetailsDesc0$
6290         editrecordEditQty0% = gatherpartdetailsQty0%
6300         editrecordEditReorder0% = gatherpartdetailsReorder0%
6310         editrecordEditPrice0! = gatherpartdetailsPrice0!
6320         GOSUB 3560
6330         editrecordKp0$ = readkeyResult0$
6340         IF (editrecordKp0$ = "Y") <> 0 THEN GOTO 6370
6350         IF (editrecordKp0$ = "y") <> 0 THEN GOTO 6370
6360         GOTO 6220
6370     REM END DO
6380     ' inv[...] = { ... }  (whole-record write)
6390     LSET invFlagBuf$ = "1"
6400     LSET invDescBuf$ = editrecordEditDesc0$
6410     LSET invQtyBuf$ = MKI$(editrecordEditQty0%)
6420     LSET invReorderBuf$ = MKI$(editrecordEditReorder0%)
6430     LSET invPriceBuf$ = MKS$(editrecordEditPrice0!)
6440     PUT #1, editrecordPart0%
6450     RETURN
6460 ' end procedure editrecord

6470 ' procedure listall()
6480     GOSUB 4300
6490     listallScrollCount0% = 0
6500     FOR listallI0% = 1 TO partcount%
6510         ' let p = inv[...]  (whole-record read)
6520         GET #1, listallI0%
6530         listallPFlagTrimI0% = LEN(invFlagBuf$)
6540         IF (listallPFlagTrimI0% > 0) = 0 THEN GOTO 6580
6550         IF (MID$(invFlagBuf$, listallPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 6580
6560             listallPFlagTrimI0% = listallPFlagTrimI0% - 1
6570             GOTO 6540
6580         REM END WHILE
6590         listallPFlag0$ = LEFT$(invFlagBuf$, listallPFlagTrimI0%)
6600         listallPDescTrimI0% = LEN(invDescBuf$)
6610         IF (listallPDescTrimI0% > 0) = 0 THEN GOTO 6650
6620         IF (MID$(invDescBuf$, listallPDescTrimI0%, 1) = " ") = 0 THEN GOTO 6650
6630             listallPDescTrimI0% = listallPDescTrimI0% - 1
6640             GOTO 6610
6650         REM END WHILE
6660         listallPDesc0$ = LEFT$(invDescBuf$, listallPDescTrimI0%)
6670         listallPQty0% = CVI(invQtyBuf$)
6680         listallPReorder0% = CVI(invReorderBuf$)
6690         listallPPrice0! = CVS(invPriceBuf$)
6700         printinventorylinePartNum0% = listallI0%
6710         printinventorylineDesc0$ = listallPDesc0$
6720         printinventorylineQty0% = listallPQty0%
6730         printinventorylineReorder0% = listallPReorder0%
6740         GOSUB 4390
6750         listallScrollCount0% = listallScrollCount0% + 1
6760         IF (listallScrollCount0% = 20) = 0 THEN GOTO 6790
6770             GOSUB 3630
6780             listallScrollCount0% = 0
6790         REM END IF
6800     NEXT listallI0%
6810     RETURN
6820 ' end procedure listall

6830 ' procedure addstock()
6840     CLS
6850     LOCATE 5, 25
6860     PRINT "A D D I N G   S T O C K"

6870         LOCATE 8, 25
6880         GOSUB 3510
6890         addstockPartStr0$ = readpartnumberinputResult0$
6900         addstockPart0% = VAL(addstockPartStr0$)
6910         partinrangeN0% = addstockPart0%
6920         GOSUB 3420
6930         addstockValidPart0% = partinrangeResult0%
6940         IF (addstockValidPart0% = 0) = 0 THEN GOTO 6970
6950             GOSUB 4030
6960             GOSUB 3560
6970         REM END IF
6980         IF (addstockValidPart0% <> 0) = 0 THEN GOTO 6870
6990     REM END DO

7000     ' let p = inv[...]  (whole-record read)
7010     GET #1, addstockPart0%
7020     addstockPFlagTrimI0% = LEN(invFlagBuf$)
7030     IF (addstockPFlagTrimI0% > 0) = 0 THEN GOTO 7070
7040     IF (MID$(invFlagBuf$, addstockPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 7070
7050         addstockPFlagTrimI0% = addstockPFlagTrimI0% - 1
7060         GOTO 7030
7070     REM END WHILE
7080     addstockPFlag0$ = LEFT$(invFlagBuf$, addstockPFlagTrimI0%)
7090     addstockPDescTrimI0% = LEN(invDescBuf$)
7100     IF (addstockPDescTrimI0% > 0) = 0 THEN GOTO 7140
7110     IF (MID$(invDescBuf$, addstockPDescTrimI0%, 1) = " ") = 0 THEN GOTO 7140
7120         addstockPDescTrimI0% = addstockPDescTrimI0% - 1
7130         GOTO 7100
7140     REM END WHILE
7150     addstockPDesc0$ = LEFT$(invDescBuf$, addstockPDescTrimI0%)
7160     addstockPQty0% = CVI(invQtyBuf$)
7170     addstockPReorder0% = CVI(invReorderBuf$)
7180     addstockPPrice0! = CVS(invPriceBuf$)
7190     isemptyFlag0$ = addstockPFlag0$
7200     GOSUB 3380
7210     IF (isemptyResult0%) = 0 THEN GOTO 7260
7220         shownullentrymessagePartStr0$ = addstockPartStr0$
7230         GOSUB 4100
7240         GOSUB 3560
7250         RETURN
7260     REM END IF

7270         showaddstockscreenPartNum0% = addstockPart0%
7280         showaddstockscreenDesc0$ = addstockPDesc0$
7290         showaddstockscreenQty0% = addstockPQty0%
7300         showaddstockscreenReorder0% = addstockPReorder0%
7310         GOSUB 4770
7320         LOCATE 14, tabcol%
7330         INPUT " Quantity to add"; addstockAddStr0$
7340         addstockAddAmt0% = VAL(addstockAddStr0$)
7350         IF (addstockAddAmt0% < 0) = 0 THEN GOTO 7380
7360             GOSUB 4930
7370             GOSUB 3560
7380         REM END IF
7390         IF (addstockAddAmt0% >= 0) = 0 THEN GOTO 7270
7400     REM END DO

7410     addstockPQty0% = addstockPQty0% + addstockAddAmt0%
7420     ' inv[...] = p  (write back a let-bound record)
7430     LSET invFlagBuf$ = addstockPFlag0$
7440     LSET invDescBuf$ = addstockPDesc0$
7450     LSET invQtyBuf$ = MKI$(addstockPQty0%)
7460     LSET invReorderBuf$ = MKI$(addstockPReorder0%)
7470     LSET invPriceBuf$ = MKS$(addstockPPrice0!)
7480     PUT #1, addstockPart0%
7490     RETURN
7500 ' end procedure addstock

7510 ' procedure subtractstock()
7520     CLS
7530     LOCATE 5, 20
7540     PRINT "S U B T R A C T I N G    S T O C K"

7550         LOCATE 8, 25
7560         GOSUB 3510
7570         subtractstockPartStr0$ = readpartnumberinputResult0$
7580         subtractstockPart0% = VAL(subtractstockPartStr0$)
7590         partinrangeN0% = subtractstockPart0%
7600         GOSUB 3420
7610         subtractstockValidPart0% = partinrangeResult0%
7620         IF (subtractstockValidPart0% = 0) = 0 THEN GOTO 7650
7630             GOSUB 4030
7640             GOSUB 3560
7650         REM END IF
7660         IF (subtractstockValidPart0% <> 0) = 0 THEN GOTO 7550
7670     REM END DO

7680     ' let p = inv[...]  (whole-record read)
7690     GET #1, subtractstockPart0%
7700     subtractstockPFlagTrimI0% = LEN(invFlagBuf$)
7710     IF (subtractstockPFlagTrimI0% > 0) = 0 THEN GOTO 7750
7720     IF (MID$(invFlagBuf$, subtractstockPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 7750
7730         subtractstockPFlagTrimI0% = subtractstockPFlagTrimI0% - 1
7740         GOTO 7710
7750     REM END WHILE
7760     subtractstockPFlag0$ = LEFT$(invFlagBuf$, subtractstockPFlagTrimI0%)
7770     subtractstockPDescTrimI0% = LEN(invDescBuf$)
7780     IF (subtractstockPDescTrimI0% > 0) = 0 THEN GOTO 7820
7790     IF (MID$(invDescBuf$, subtractstockPDescTrimI0%, 1) = " ") = 0 THEN GOTO 7820
7800         subtractstockPDescTrimI0% = subtractstockPDescTrimI0% - 1
7810         GOTO 7780
7820     REM END WHILE
7830     subtractstockPDesc0$ = LEFT$(invDescBuf$, subtractstockPDescTrimI0%)
7840     subtractstockPQty0% = CVI(invQtyBuf$)
7850     subtractstockPReorder0% = CVI(invReorderBuf$)
7860     subtractstockPPrice0! = CVS(invPriceBuf$)
7870     isemptyFlag0$ = subtractstockPFlag0$
7880     GOSUB 3380
7890     IF (isemptyResult0%) = 0 THEN GOTO 7940
7900         shownullentrymessagePartStr0$ = subtractstockPartStr0$
7910         GOSUB 4100
7920         GOSUB 3560
7930         RETURN
7940     REM END IF

7950         showsubtractstockscreenPartNum0% = subtractstockPart0%
7960         showsubtractstockscreenDesc0$ = subtractstockPDesc0$
7970         showsubtractstockscreenQty0% = subtractstockPQty0%
7980         showsubtractstockscreenReorder0% = subtractstockPReorder0%
7990         GOSUB 5000
8000         LOCATE 14, tabcol%
8010         INPUT "Quantity to subtract"; subtractstockSubStr0$
8020         subtractstockSubAmt0% = VAL(subtractstockSubStr0$)
8030         subtractstockOverSubtract0% = 0
8040         IF (subtractstockSubAmt0% >= 0) = 0 THEN GOTO 8100
8050         IF ((subtractstockPQty0% - subtractstockSubAmt0%) < 0) = 0 THEN GOTO 8100
8060             subtractstockOverSubtract0% = 1
8070             showoversubtractwarningOnHand0% = subtractstockPQty0%
8080             GOSUB 5160
8090             GOSUB 3560
8100         REM END IF
8110         IF (subtractstockSubAmt0% >= 0) = 0 THEN GOTO 7950
8120         IF (subtractstockOverSubtract0% = 0) = 0 THEN GOTO 7950
8130     REM END DO

8140     subtractstockPQty0% = subtractstockPQty0% - subtractstockSubAmt0%
8150     IF (subtractstockPQty0% <= subtractstockPReorder0%) = 0 THEN GOTO 8170
8160         LOCATE 16, tabcol%
8170     REM END IF
8180     PRINT (("quantity now" + STR$(subtractstockPQty0%)) + " reorder level") + STR$(subtractstockPReorder0%)
8190     ' inv[...] = p  (write back a let-bound record)
8200     LSET invFlagBuf$ = subtractstockPFlag0$
8210     LSET invDescBuf$ = subtractstockPDesc0$
8220     LSET invQtyBuf$ = MKI$(subtractstockPQty0%)
8230     LSET invReorderBuf$ = MKI$(subtractstockPReorder0%)
8240     LSET invPriceBuf$ = MKS$(subtractstockPPrice0!)
8250     PUT #1, subtractstockPart0%
8260     RETURN
8270 ' end procedure subtractstock

8280 ' procedure reorderreport()
8290     GOSUB 4430
8300     reorderreportReportLineCount0% = 0
8310     FOR reorderreportI0% = 1 TO partcount%
8320         ' let p = inv[...]  (whole-record read)
8330         GET #1, reorderreportI0%
8340         reorderreportPFlagTrimI0% = LEN(invFlagBuf$)
8350         IF (reorderreportPFlagTrimI0% > 0) = 0 THEN GOTO 8390
8360         IF (MID$(invFlagBuf$, reorderreportPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 8390
8370             reorderreportPFlagTrimI0% = reorderreportPFlagTrimI0% - 1
8380             GOTO 8350
8390         REM END WHILE
8400         reorderreportPFlag0$ = LEFT$(invFlagBuf$, reorderreportPFlagTrimI0%)
8410         reorderreportPDescTrimI0% = LEN(invDescBuf$)
8420         IF (reorderreportPDescTrimI0% > 0) = 0 THEN GOTO 8460
8430         IF (MID$(invDescBuf$, reorderreportPDescTrimI0%, 1) = " ") = 0 THEN GOTO 8460
8440             reorderreportPDescTrimI0% = reorderreportPDescTrimI0% - 1
8450             GOTO 8420
8460         REM END WHILE
8470         reorderreportPDesc0$ = LEFT$(invDescBuf$, reorderreportPDescTrimI0%)
8480         reorderreportPQty0% = CVI(invQtyBuf$)
8490         reorderreportPReorder0% = CVI(invReorderBuf$)
8500         reorderreportPPrice0! = CVS(invPriceBuf$)
8510         IF (reorderreportPQty0% < reorderreportPReorder0%) = 0 THEN GOTO 8620
8520             printreorderlinePartNum0% = reorderreportI0%
8530             printreorderlineDesc0$ = reorderreportPDesc0$
8540             printreorderlineQty0% = reorderreportPQty0%
8550             printreorderlineReorder0% = reorderreportPReorder0%
8560             GOSUB 4530
8570             reorderreportReportLineCount0% = reorderreportReportLineCount0% + 1
8580             IF (reorderreportReportLineCount0% > 15) = 0 THEN GOTO 8610
8590                 GOSUB 3630
8600                 reorderreportReportLineCount0% = 0
8610             REM END IF
8620         REM END IF
8630     NEXT reorderreportI0%
8640     GOSUB 3630
8650     RETURN
8660 ' end procedure reorderreport

8670 ' procedure reportinventoryerror(err%, erl%)
8680     LOCATE 25, 1
8690     errorCode0% = reportinventoryerrorErr0%
8700     GOSUB 1980
8710     PRINT (("There has been an error on line" + STR$(reportinventoryerrorErl0%)) + ": ") + errorResult0$
8720     GOSUB 3560
8730     reportinventoryerrorK0$ = readkeyResult0$
8740     RETURN
8750 ' end procedure reportinventoryerror
