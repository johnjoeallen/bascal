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
880 ' hand at his line 550.
890 ' file inv as Part = open(...)  [39 bytes/record]
900 OPEN "inven.dat" FOR RANDOM AS #1 LEN = 39
910 FIELD #1, 1 AS invFlagBuf$, 30 AS invDescBuf$, 2 AS invQtyBuf$, 2 AS invReorderBuf$, 4 AS invPriceBuf$

920 ' -------------------- Pure functions (no file access) --------------------

930 ' BASCAL-ism: `function ... end function` with `return` replaces
940 ' fhb's convention of a GOSUB target plus a bare RETURN -- there's
950 ' no separate "subroutine label" and no shared/global result
960 ' variable to manage by hand; `isEmpty%(...)` is called like an
970 ' ordinary expression at every use below (e.g. `isEmpty%(p.flag)`).
980 ' A record whose flag byte is CHR$(255) is an empty/never-used slot.

990 ' BASCAL-ism: `&&` and `||` are short-circuit AND/OR -- real
1000 ' MBASIC/BASCOM only has bitwise AND/OR (which fhb relies on here
1010 ' too, since `PART!<1 OR PART!>N!` never short-circuits anyway).
1020 ' BASCAL lowers `&&`/`||` into the equivalent branching so the
1030 ' short-circuit *is* real at the generated-BASIC level; see the
1040 ' manual's "Short-Circuit && and ||" section
1050 ' (https://johnjoeallen.github.io/bascal/manual/).

1060 ' -------------------- Keyboard input --------------------

1070 ' BASCAL-ism: `do ... loop until` is a structured post-check loop
1080 ' replacing fhb's `730 KP$=INKEY$:IF KP$="" THEN 730` GOTO-polling
1090 ' idiom. `inkey$` itself is the real INKEY$ builtin passed straight
1100 ' through, resolving correctly from inside a function/procedure
1110 ' body like this one -- every menu action below calls
1120 ' readKey$()/waitAnyKey() rather than polling INKEY$ inline.

1130 ' -------------------- Display procedures --------------------

1140 ' byref scalar parameters: gatherPartDetails writes the four editable
1150 ' fields for a part directly back into the caller's variables.

1160 ' -------------------- Menu actions --------------------

1170 ' fhb's own one-time "hidden" datafile initializer PUT-ing 100 blank,
1180 ' CHR$(255)-flagged records (see the header note above) -- reproduced
1190 ' here so inven.dat no longer has to be pre-populated by hand before
1200 ' running this program. A brand-new file OPEN created just now (rather
1210 ' than one that already existed) reads back as all-zero bytes: record
1220 ' 1's flag byte is CHR$(0), never CHR$(255) -- the one signal an
1230 ' already-populated file (whose record 1 flag is always either
1240 ' CHR$(255), still an empty slot, or a real part's own "1") could never
1250 ' produce, so it's what isEmpty%() itself can't use (see its own
1260 ' header note) but this one-time check safely can.

1270 ' -------------------- Program entry --------------------

1280 CLS
1290 GOSUB 8770

1300     GOSUB 3810
1310     GOSUB 3660
1320     kp$ = readkeyResult0$
1330     IF (INSTR("1234567cCeElLaAsSrRxX", kp$) <> 0) = 0 THEN GOTO 1910
1340         ' BASCAL-ism: `select case` replaces fhb's chain of eight
1350         ' `IF VAL(KP$)=n OR KP$="x" OR KP$="X" THEN GOTO ...` lines
1360         ' (his 770-840) with one multi-way dispatch.
1370         ' 
1380         ' BASCAL-ism: `try`/`catch` (issue #60) replaces fhb's own global
1390         ' `ON ERROR GOTO` trap. A failed menu action is abandoned outright
1400         ' here -- the `catch` below runs, then execution continues right
1410         ' after `end try`, back at `loop until` -- rather than resuming at
1420         ' the exact instruction after whatever failed inside checkPart()/
1430         ' editRecord()/etc. the way fhb's `RESUME NEXT` did. See
1440         ' reportInventoryError() below and tutorial/inventory_try_catch.
1450         ' draft's own header comment for why that arbitrary resume-point
1460         ' behavior isn't something try/catch reproduces.
1470         ON ERROR GOTO 1840
1480             BCCT5$ = kp$
1490             IF (BCCT5$ = "1" OR BCCT5$ = "c" OR BCCT5$ = "C") <> 0 THEN GOTO 1570
1500             IF (BCCT5$ = "2" OR BCCT5$ = "e" OR BCCT5$ = "E") <> 0 THEN GOTO 1590
1510             IF (BCCT5$ = "3" OR BCCT5$ = "l" OR BCCT5$ = "L") <> 0 THEN GOTO 1610
1520             IF (BCCT5$ = "4" OR BCCT5$ = "a" OR BCCT5$ = "A") <> 0 THEN GOTO 1630
1530             IF (BCCT5$ = "5" OR BCCT5$ = "s" OR BCCT5$ = "S") <> 0 THEN GOTO 1650
1540             IF (BCCT5$ = "6" OR BCCT5$ = "r" OR BCCT5$ = "R") <> 0 THEN GOTO 1670
1550             IF (BCCT5$ = "7" OR BCCT5$ = "x" OR BCCT5$ = "X") <> 0 THEN GOTO 1690
1560             GOTO 1810
1570                 GOSUB 5340
1580                 GOTO 1810
1590                 GOSUB 5880
1600                 GOTO 1810
1610                 GOSUB 6570
1620                 GOTO 1810
1630                 GOSUB 6930
1640                 GOTO 1810
1650                 GOSUB 7610
1660                 GOTO 1810
1670                 GOSUB 8380
1680                 GOTO 1810
1690                 ' BASCAL-ism: `inv.close()` is sugar for `CLOSE #1`,
1700                 ' matching fhb's own `90 CLOSE:SYSTEM`. fhb's original
1710                 ' also had a separate "Quit to BASIC" option (his own
1720                 ' 7, returning to the interpreter's command prompt
1730                 ' rather than exiting to DOS) -- dropped here: a
1740                 ' compiled program has no interpreter to return to,
1750                 ' so it was never anything but a second spelling of
1760                 ' this same close-and-exit action.
1770                 ' inv.close()
1780                 CLOSE #1
1790                 SYSTEM
1800                 GOTO 1810
1810             REM END SELECT
1820         ON ERROR GOTO 0
1830         GOTO 1900
1840             err% = ERR
1850             erl% = ERL
1860             reportinventoryerrorErr0% = err%
1870             reportinventoryerrorErl0% = erl%
1880             GOSUB 9100
1890             RESUME 1900
1900         REM END TRY
1910     REM END IF
1920     GOTO 1300
1930 REM END DO

1940 ' -------------------- Error handling --------------------
1950 ' err%/erl% are ordinary locals scoped to the `catch` block above, not
1960 ' aliases for the ambient (readable-anywhere) `err`/`erl` pseudo-
1970 ' variables `on error goto` uses -- see `Statement::TryCatch`'s own doc
1980 ' comment in ast.rs. Passed straight through to ERROR$ here like fhb's
1990 ' own ERR/ERL (his 3390: "an error on line";ERL), decoded through
2000 ' BASCAL's own com.bascal.stdlib.error (ERROR$) instead of fhb's
2010 ' hand-rolled lookup table -- see the header note above. try/catch
2020 ' itself isn't documented in the manual yet (GitHub issue #60 tracks
2030 ' the still-unfinished C-target work; the manual page can follow once
2040 ' that lands) -- see ast.rs's own `Statement::TryCatch` doc comment for
2050 ' the full semantics meanwhile.
2060 END

2070 ' function error$(code%)
2080     BCCT7% = errorCode0%
2090     IF (BCCT7% = 2) <> 0 THEN GOTO 2430
2100     IF (BCCT7% = 3) <> 0 THEN GOTO 2460
2110     IF (BCCT7% = 4) <> 0 THEN GOTO 2490
2120     IF (BCCT7% = 5) <> 0 THEN GOTO 2520
2130     IF (BCCT7% = 6) <> 0 THEN GOTO 2550
2140     IF (BCCT7% = 7) <> 0 THEN GOTO 2580
2150     IF (BCCT7% = 9) <> 0 THEN GOTO 2610
2160     IF (BCCT7% = 10) <> 0 THEN GOTO 2640
2170     IF (BCCT7% = 11) <> 0 THEN GOTO 2670
2180     IF (BCCT7% = 13) <> 0 THEN GOTO 2700
2190     IF (BCCT7% = 14) <> 0 THEN GOTO 2730
2200     IF (BCCT7% = 19) <> 0 THEN GOTO 2760
2210     IF (BCCT7% = 20) <> 0 THEN GOTO 2790
2220     IF (BCCT7% = 24) <> 0 THEN GOTO 2820
2230     IF (BCCT7% = 25) <> 0 THEN GOTO 2850
2240     IF (BCCT7% = 27) <> 0 THEN GOTO 2880
2250     IF (BCCT7% = 52) <> 0 THEN GOTO 2910
2260     IF (BCCT7% = 53) <> 0 THEN GOTO 2940
2270     IF (BCCT7% = 54) <> 0 THEN GOTO 2970
2280     IF (BCCT7% = 55) <> 0 THEN GOTO 3000
2290     IF (BCCT7% = 57) <> 0 THEN GOTO 3030
2300     IF (BCCT7% = 58) <> 0 THEN GOTO 3060
2310     IF (BCCT7% = 61) <> 0 THEN GOTO 3090
2320     IF (BCCT7% = 62) <> 0 THEN GOTO 3120
2330     IF (BCCT7% = 63) <> 0 THEN GOTO 3150
2340     IF (BCCT7% = 64) <> 0 THEN GOTO 3180
2350     IF (BCCT7% = 67) <> 0 THEN GOTO 3210
2360     IF (BCCT7% = 68) <> 0 THEN GOTO 3240
2370     IF (BCCT7% = 70) <> 0 THEN GOTO 3270
2380     IF (BCCT7% = 71) <> 0 THEN GOTO 3300
2390     IF (BCCT7% = 72) <> 0 THEN GOTO 3330
2400     IF (BCCT7% = 75) <> 0 THEN GOTO 3360
2410     IF (BCCT7% = 76) <> 0 THEN GOTO 3390
2420     GOTO 3420
2430         errorResult0$ = "Syntax error"
2440         RETURN
2450         GOTO 3440
2460         errorResult0$ = "RETURN without GOSUB"
2470         RETURN
2480         GOTO 3440
2490         errorResult0$ = "Out of DATA"
2500         RETURN
2510         GOTO 3440
2520         errorResult0$ = "Illegal function call"
2530         RETURN
2540         GOTO 3440
2550         errorResult0$ = "Overflow"
2560         RETURN
2570         GOTO 3440
2580         errorResult0$ = "Out of memory"
2590         RETURN
2600         GOTO 3440
2610         errorResult0$ = "Subscript out of range"
2620         RETURN
2630         GOTO 3440
2640         errorResult0$ = "Duplicate Definition"
2650         RETURN
2660         GOTO 3440
2670         errorResult0$ = "Division by zero"
2680         RETURN
2690         GOTO 3440
2700         errorResult0$ = "Type mismatch"
2710         RETURN
2720         GOTO 3440
2730         errorResult0$ = "Out of string space"
2740         RETURN
2750         GOTO 3440
2760         errorResult0$ = "No RESUME"
2770         RETURN
2780         GOTO 3440
2790         errorResult0$ = "RESUME without error"
2800         RETURN
2810         GOTO 3440
2820         errorResult0$ = "Device timeout"
2830         RETURN
2840         GOTO 3440
2850         errorResult0$ = "Device fault"
2860         RETURN
2870         GOTO 3440
2880         errorResult0$ = "Out of paper"
2890         RETURN
2900         GOTO 3440
2910         errorResult0$ = "Bad file number"
2920         RETURN
2930         GOTO 3440
2940         errorResult0$ = "File not found"
2950         RETURN
2960         GOTO 3440
2970         errorResult0$ = "Bad file mode"
2980         RETURN
2990         GOTO 3440
3000         errorResult0$ = "File already open"
3010         RETURN
3020         GOTO 3440
3030         errorResult0$ = "Device I/O error"
3040         RETURN
3050         GOTO 3440
3060         errorResult0$ = "File already exists"
3070         RETURN
3080         GOTO 3440
3090         errorResult0$ = "Disk full"
3100         RETURN
3110         GOTO 3440
3120         errorResult0$ = "Input past end"
3130         RETURN
3140         GOTO 3440
3150         errorResult0$ = "Bad record number"
3160         RETURN
3170         GOTO 3440
3180         errorResult0$ = "Bad file name"
3190         RETURN
3200         GOTO 3440
3210         errorResult0$ = "Too many files"
3220         RETURN
3230         GOTO 3440
3240         errorResult0$ = "Device unavailable"
3250         RETURN
3260         GOTO 3440
3270         errorResult0$ = "Disk write protected"
3280         RETURN
3290         GOTO 3440
3300         errorResult0$ = "Disk not ready"
3310         RETURN
3320         GOTO 3440
3330         errorResult0$ = "Disk media error"
3340         RETURN
3350         GOTO 3440
3360         errorResult0$ = "Path/File access error"
3370         RETURN
3380         GOTO 3440
3390         errorResult0$ = "Path not found"
3400         RETURN
3410         GOTO 3440
3420         errorResult0$ = "Error " + STR$(errorCode0%)
3430         RETURN
3440     REM END SELECT
3450     RETURN
3460 ' end function error$

3470 ' function isempty%(flag$)
3480     isemptyResult0% = ASC(isemptyFlag0$) = 255
3490     RETURN
3500 ' end function isempty%

3510 ' function partinrange%(n%)
3520     IF (partinrangeN0% >= 1) = 0 THEN GOTO 3560
3530     IF (partinrangeN0% <= partcount%) = 0 THEN GOTO 3560
3540         partinrangeResult0% = 1
3550         RETURN
3560     REM END IF
3570     partinrangeResult0% = 0
3580     RETURN
3590 ' end function partinrange%

3600 ' function readpartnumberinput$()
3610     INPUT "Input part number"; readpartnumberinputS0$
3620     readpartnumberinputResult0$ = readpartnumberinputS0$
3630     RETURN
3640 ' end function readpartnumberinput$

3650 ' function readkey$()
3660         readkeyK0$ = INKEY$
3670         IF (readkeyK0$ <> "") = 0 THEN GOTO 3660
3680     REM END DO
3690     readkeyResult0$ = readkeyK0$
3700     RETURN
3710 ' end function readkey$

3720 ' procedure waitanykey()
3730     LOCATE 25, 10
3740     PRINT "Press the AnyKey to continue...";
3750         waitanykeyK0$ = INKEY$
3760         IF (waitanykeyK0$ <> "") = 0 THEN GOTO 3750
3770     REM END DO
3780     RETURN
3790 ' end procedure waitanykey

3800 ' procedure showmainmenu()
3810     CLS
3820     COLOR 14, 4
3830     CLS
3840     LOCATE 6, 1
3850     PRINT
3860     ' `tab(n)` passes straight through to real TAB(n), same as
3870     ' fhb's own `PRINT TAB(V) "..."` -- but only as a bare item in
3880     ' a PRINT list, juxtaposed or `;`-separated like here. Real
3890     ' BASCOM rejects `"literal" + tab(n) + ...` (TAB isn't a real
3900     ' string function you can concatenate); see printListHeader()
3910     ' and printReorderHeader() below, which need `;` between a
3920     ' preceding string and a `tab(n)` for exactly this reason.
3930     PRINT TAB(30)"Inventory Program"
3940     PRINT
3950     PRINT TAB(tabcol%)"1......C)heck a part"
3960     PRINT TAB(tabcol%)"2......E)dit/overwrite/add a part"
3970     PRINT TAB(tabcol%)("3......L)ist all" + STR$(partcount%)) + "parts"
3980     PRINT TAB(tabcol%)"4......A)dd stock"
3990     PRINT TAB(tabcol%)"5......S)ubtract stock"
4000     PRINT TAB(tabcol%)"6......R)eorder Report"
4010     PRINT
4020     PRINT TAB(tabcol%)"7......eX)it to system"
4030     RETURN
4040 ' end procedure showmainmenu

4050 ' procedure showbadpartnumber()
4060     CLS
4070     LOCATE 10, 10
4080     PRINT "Part number is out of permissable range of 1 to" + STR$(partcount%)
4090     RETURN
4100 ' end procedure showbadpartnumber

4110 ' procedure showrangeretrymessage()
4120     LOCATE 10, 15
4130     PRINT "The Part number is out of permissable range of 1 to" + STR$(partcount%)
4140     LOCATE 25, 15
4150     PRINT "Press the Anykey to reenter part number...";
4160     RETURN
4170 ' end procedure showrangeretrymessage

4180 ' procedure shownullentrymessage(partstr$)
4190     LOCATE 10, tabcol%
4200     PRINT ("Part number " + shownullentrymessagePartStr0$) + " is a null entry"
4210     RETURN
4220 ' end procedure shownullentrymessage

4230 ' procedure showpartstatus(partnum%, desc$, qty%, reorder%, price!)
4240     CLS
4250     LOCATE 5, 1
4260     PRINT TAB(tabcol%)"Inventory Status for Individual Part Number"
4270     PRINT TAB(tabcol%)"==========================================="
4280     PRINT
4290     PRINT
4300     PRINT TAB(tabcol%)"     Part number:  " + STR$(showpartstatusPartNum0%)
4310     PRINT
4320     PRINT TAB(tabcol%)"       Item name:  " + showpartstatusDesc0$
4330     PRINT TAB(tabcol%)"Quantity on hand:  " + STR$(showpartstatusQty0%)
4340     PRINT TAB(tabcol%)"   Reorder level:  " + STR$(showpartstatusReorder0%)
4350     PRINT TAB(tabcol%)"      Unit price:  " + STR$(showpartstatusPrice0!)
4360     RETURN
4370 ' end procedure showpartstatus

4380 ' procedure printlistheader()
4390     CLS
4400     PRINT TAB(25)"I N V E N T O R Y   L I S T I N G"; TAB(65); STR$(partcount%) + "items"
4410     PRINT "                                          Quantity       Reorder"
4420     PRINT " Partno           Description             on hand         level"
4430     LOCATE 25, 1
4440     PRINT "Press the AnyKey to scroll listing...";
4450     RETURN
4460 ' end procedure printlistheader

4470 ' procedure printinventoryline(partnum%, desc$, qty%, reorder%)
4480     PRINT (((((STR$(printinventorylinePartNum0%) + "  ") + printinventorylineDesc0$) + "   ") + STR$(printinventorylineQty0%)) + "          ") + STR$(printinventorylineReorder0%)
4490     RETURN
4500 ' end procedure printinventoryline

4510 ' procedure printreorderheader()
4520     CLS
4530     LOCATE 1, tabcol%
4540     PRINT "Reorder Report"; TAB(55); DATE$
4550     PRINT
4560     PRINT "                                             Quantity       Reorder"
4570     PRINT "    Partno           Description             on hand         level"
4580     PRINT "   =======  ==============================   ========       ======="
4590     RETURN
4600 ' end procedure printreorderheader

4610 ' procedure printreorderline(partnum%, desc$, qty%, reorder%)
4620     PRINT (((((("  " + STR$(printreorderlinePartNum0%)) + "  ") + printreorderlineDesc0$) + "   ") + STR$(printreorderlineQty0%)) + "          ") + STR$(printreorderlineReorder0%)
4630     RETURN
4640 ' end procedure printreorderline

4650 ' procedure gatherpartdetails(partnum%, desc$, qty%, reorder%, price!)
4660     CLS
4670     LOCATE 4, tabcol%
4680     PRINT "Adding or Overwriting a Record"
4690     LOCATE 8, tabcol%
4700     PRINT "Record/Partno" + STR$(gatherpartdetailsPartNum0%)
4710     LOCATE 11, 39
4720     PRINT "------------------------------"
4730     LOCATE 10, tabcol%
4740     INPUT "      Description"; gatherpartdetailsDesc0$
4750     LOCATE 12, tabcol%
4760     INPUT "Quantity in stock"; gatherpartdetailsQty0%
4770     LOCATE 14, tabcol%
4780     INPUT "    Reorder level"; gatherpartdetailsReorder0%
4790     LOCATE 16, tabcol%
4800     INPUT "       Unit price"; gatherpartdetailsPrice0!
4810     LOCATE 18, tabcol%
4820     PRINT "Is information correct (Y/N)?"
4830     RETURN
4840 ' end procedure gatherpartdetails

4850 ' procedure showaddstockscreen(partnum%, desc$, qty%, reorder%)
4860     CLS
4870     LOCATE 4, 25
4880     PRINT "Add to an inventory part number"
4890     LOCATE 5, 25
4900     PRINT "==============================="
4910     LOCATE 8, tabcol%
4920     PRINT "     Part number: " + STR$(showaddstockscreenPartNum0%)
4930     LOCATE 9, tabcol%
4940     PRINT "Item description: " + showaddstockscreenDesc0$
4950     LOCATE 10, tabcol%
4960     PRINT "Quantity on hand: " + STR$(showaddstockscreenQty0%)
4970     LOCATE 11, tabcol%
4980     PRINT "   Reorder Level: " + STR$(showaddstockscreenReorder0%)
4990     RETURN
5000 ' end procedure showaddstockscreen

5010 ' procedure shownegativeqtywarning()
5020     LOCATE 17, 15
5030     PRINT "The quantity to add must NOT be a negative number"
5040     LOCATE 25, 1
5050     PRINT "Please press the Anykey to reenter quantity to add...";
5060     RETURN
5070 ' end procedure shownegativeqtywarning

5080 ' procedure showsubtractstockscreen(partnum%, desc$, qty%, reorder%)
5090     CLS
5100     LOCATE 4, tabcol%
5110     PRINT "Subtract an inventory part number"
5120     LOCATE 5, tabcol%
5130     PRINT "================================="
5140     LOCATE 8, tabcol%
5150     PRINT "         Part number: " + STR$(showsubtractstockscreenPartNum0%)
5160     LOCATE 9, tabcol%
5170     PRINT "    Item description: " + showsubtractstockscreenDesc0$
5180     LOCATE 10, tabcol%
5190     PRINT "    Quantity on hand: " + STR$(showsubtractstockscreenQty0%)
5200     LOCATE 11, tabcol%
5210     PRINT "       Reorder Level: " + STR$(showsubtractstockscreenReorder0%)
5220     RETURN
5230 ' end procedure showsubtractstockscreen

5240 ' procedure showoversubtractwarning(onhand%)
5250     LOCATE 17, 5
5260     PRINT "The quantity to SUBTRACT must NOT result in NEGATIVE inventory"
5270     LOCATE 18, 5
5280     PRINT ("Only" + STR$(showoversubtractwarningOnHand0%)) + " IN STOCK"
5290     LOCATE 25, 1
5300     PRINT "Please press the Anykey to reenter quantity to subtract...";
5310     RETURN
5320 ' end procedure showoversubtractwarning

5330 ' procedure checkpart()
5340     GOSUB 3610
5350     checkpartPartStr0$ = readpartnumberinputResult0$
5360     checkpartPart0% = VAL(checkpartPartStr0$)
5370     partinrangeN0% = checkpartPart0%
5380     GOSUB 3520
5390     IF (partinrangeResult0% = 0) = 0 THEN GOTO 5430
5400         GOSUB 4060
5410         GOSUB 3730
5420         RETURN
5430     REM END IF
5440     ' BASCAL-ism: `let p = inv[part%]` reads record `part%` of the
5450     ' `inv` file into a local record variable `p` -- one expression
5460     ' for what fhb's `GET #1, PART!` plus five separate field reads
5470     ' (F$, D$, CVI(Q$), CVI(R$), CVS(P$)) did by hand. The write
5480     ' side, `inv[part%] = { ... }` (see editRecord() below), is the
5490     ' same sugar for PUT plus the LSET/MKx$ packing it replaces.
5500     ' let p = inv[...]  (whole-record read)
5510     GET #1, checkpartPart0%
5520     checkpartPFlagTrimI0% = LEN(invFlagBuf$)
5530     IF (checkpartPFlagTrimI0% > 0) = 0 THEN GOTO 5570
5540     IF (MID$(invFlagBuf$, checkpartPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 5570
5550         checkpartPFlagTrimI0% = checkpartPFlagTrimI0% - 1
5560         GOTO 5530
5570     REM END WHILE
5580     checkpartPFlag0$ = LEFT$(invFlagBuf$, checkpartPFlagTrimI0%)
5590     checkpartPDescTrimI0% = LEN(invDescBuf$)
5600     IF (checkpartPDescTrimI0% > 0) = 0 THEN GOTO 5640
5610     IF (MID$(invDescBuf$, checkpartPDescTrimI0%, 1) = " ") = 0 THEN GOTO 5640
5620         checkpartPDescTrimI0% = checkpartPDescTrimI0% - 1
5630         GOTO 5600
5640     REM END WHILE
5650     checkpartPDesc0$ = LEFT$(invDescBuf$, checkpartPDescTrimI0%)
5660     checkpartPQty0% = CVI(invQtyBuf$)
5670     checkpartPReorder0% = CVI(invReorderBuf$)
5680     checkpartPPrice0! = CVS(invPriceBuf$)
5690     isemptyFlag0$ = checkpartPFlag0$
5700     GOSUB 3480
5710     IF (isemptyResult0%) = 0 THEN GOTO 5770
5720         CLS
5730         LOCATE 10, 18
5740         PRINT ("Part number" + STR$(checkpartPart0%)) + "is still a null entry at this time"
5750         GOSUB 3730
5760         RETURN
5770     REM END IF
5780     showpartstatusPartNum0% = checkpartPart0%
5790     showpartstatusDesc0$ = checkpartPDesc0$
5800     showpartstatusQty0% = checkpartPQty0%
5810     showpartstatusReorder0% = checkpartPReorder0%
5820     showpartstatusPrice0! = checkpartPPrice0!
5830     GOSUB 4240
5840     GOSUB 3730
5850     RETURN
5860 ' end procedure checkpart

5870 ' procedure editrecord()
5880     CLS
5890     LOCATE 10, tabcol%
5900     GOSUB 3610
5910     editrecordPartStr0$ = readpartnumberinputResult0$
5920     editrecordPart0% = VAL(editrecordPartStr0$)
5930     partinrangeN0% = editrecordPart0%
5940     GOSUB 3520
5950     IF (partinrangeResult0% = 0) = 0 THEN GOTO 5990
5960         GOSUB 4060
5970         GOSUB 3730
5980         RETURN
5990     REM END IF
6000     ' let p = inv[...]  (whole-record read)
6010     GET #1, editrecordPart0%
6020     editrecordPFlagTrimI0% = LEN(invFlagBuf$)
6030     IF (editrecordPFlagTrimI0% > 0) = 0 THEN GOTO 6070
6040     IF (MID$(invFlagBuf$, editrecordPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 6070
6050         editrecordPFlagTrimI0% = editrecordPFlagTrimI0% - 1
6060         GOTO 6030
6070     REM END WHILE
6080     editrecordPFlag0$ = LEFT$(invFlagBuf$, editrecordPFlagTrimI0%)
6090     editrecordPDescTrimI0% = LEN(invDescBuf$)
6100     IF (editrecordPDescTrimI0% > 0) = 0 THEN GOTO 6140
6110     IF (MID$(invDescBuf$, editrecordPDescTrimI0%, 1) = " ") = 0 THEN GOTO 6140
6120         editrecordPDescTrimI0% = editrecordPDescTrimI0% - 1
6130         GOTO 6100
6140     REM END WHILE
6150     editrecordPDesc0$ = LEFT$(invDescBuf$, editrecordPDescTrimI0%)
6160     editrecordPQty0% = CVI(invQtyBuf$)
6170     editrecordPReorder0% = CVI(invReorderBuf$)
6180     editrecordPPrice0! = CVS(invPriceBuf$)
6190     isemptyFlag0$ = editrecordPFlag0$
6200     GOSUB 3480
6210     IF (isemptyResult0% = 0) = 0 THEN GOTO 6300
6220         LOCATE 12, tabcol%
6230         PRINT "Overwrite existing part data?"
6240         GOSUB 3660
6250         editrecordKp0$ = readkeyResult0$
6260         IF (editrecordKp0$ <> "Y") = 0 THEN GOTO 6290
6270         IF (editrecordKp0$ <> "y") = 0 THEN GOTO 6290
6280             RETURN
6290         REM END IF
6300     REM END IF

6310         gatherpartdetailsPartNum0% = editrecordPart0%
6320         gatherpartdetailsDesc0$ = editrecordEditDesc0$
6330         gatherpartdetailsQty0% = editrecordEditQty0%
6340         gatherpartdetailsReorder0% = editrecordEditReorder0%
6350         gatherpartdetailsPrice0! = editrecordEditPrice0!
6360         GOSUB 4660
6370         editrecordEditDesc0$ = gatherpartdetailsDesc0$
6380         editrecordEditQty0% = gatherpartdetailsQty0%
6390         editrecordEditReorder0% = gatherpartdetailsReorder0%
6400         editrecordEditPrice0! = gatherpartdetailsPrice0!
6410         GOSUB 3660
6420         editrecordKp0$ = readkeyResult0$
6430         IF (editrecordKp0$ = "Y") <> 0 THEN GOTO 6460
6440         IF (editrecordKp0$ = "y") <> 0 THEN GOTO 6460
6450         GOTO 6310
6460     REM END DO
6470     ' inv[...] = { ... }  (whole-record write)
6480     LSET invFlagBuf$ = "1"
6490     LSET invDescBuf$ = editrecordEditDesc0$
6500     LSET invQtyBuf$ = MKI$(editrecordEditQty0%)
6510     LSET invReorderBuf$ = MKI$(editrecordEditReorder0%)
6520     LSET invPriceBuf$ = MKS$(editrecordEditPrice0!)
6530     PUT #1, editrecordPart0%
6540     RETURN
6550 ' end procedure editrecord

6560 ' procedure listall()
6570     GOSUB 4390
6580     listallScrollCount0% = 0
6590     FOR listallI0% = 1 TO partcount%
6600         ' let p = inv[...]  (whole-record read)
6610         GET #1, listallI0%
6620         listallPFlagTrimI0% = LEN(invFlagBuf$)
6630         IF (listallPFlagTrimI0% > 0) = 0 THEN GOTO 6670
6640         IF (MID$(invFlagBuf$, listallPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 6670
6650             listallPFlagTrimI0% = listallPFlagTrimI0% - 1
6660             GOTO 6630
6670         REM END WHILE
6680         listallPFlag0$ = LEFT$(invFlagBuf$, listallPFlagTrimI0%)
6690         listallPDescTrimI0% = LEN(invDescBuf$)
6700         IF (listallPDescTrimI0% > 0) = 0 THEN GOTO 6740
6710         IF (MID$(invDescBuf$, listallPDescTrimI0%, 1) = " ") = 0 THEN GOTO 6740
6720             listallPDescTrimI0% = listallPDescTrimI0% - 1
6730             GOTO 6700
6740         REM END WHILE
6750         listallPDesc0$ = LEFT$(invDescBuf$, listallPDescTrimI0%)
6760         listallPQty0% = CVI(invQtyBuf$)
6770         listallPReorder0% = CVI(invReorderBuf$)
6780         listallPPrice0! = CVS(invPriceBuf$)
6790         printinventorylinePartNum0% = listallI0%
6800         printinventorylineDesc0$ = listallPDesc0$
6810         printinventorylineQty0% = listallPQty0%
6820         printinventorylineReorder0% = listallPReorder0%
6830         GOSUB 4480
6840         listallScrollCount0% = listallScrollCount0% + 1
6850         IF (listallScrollCount0% = 20) = 0 THEN GOTO 6880
6860             GOSUB 3730
6870             listallScrollCount0% = 0
6880         REM END IF
6890     NEXT listallI0%
6900     RETURN
6910 ' end procedure listall

6920 ' procedure addstock()
6930     CLS
6940     LOCATE 5, 25
6950     PRINT "A D D I N G   S T O C K"

6960         LOCATE 8, 25
6970         GOSUB 3610
6980         addstockPartStr0$ = readpartnumberinputResult0$
6990         addstockPart0% = VAL(addstockPartStr0$)
7000         partinrangeN0% = addstockPart0%
7010         GOSUB 3520
7020         addstockValidPart0% = partinrangeResult0%
7030         IF (addstockValidPart0% = 0) = 0 THEN GOTO 7060
7040             GOSUB 4120
7050             GOSUB 3660
7060         REM END IF
7070         IF (addstockValidPart0% <> 0) = 0 THEN GOTO 6960
7080     REM END DO

7090     ' let p = inv[...]  (whole-record read)
7100     GET #1, addstockPart0%
7110     addstockPFlagTrimI0% = LEN(invFlagBuf$)
7120     IF (addstockPFlagTrimI0% > 0) = 0 THEN GOTO 7160
7130     IF (MID$(invFlagBuf$, addstockPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 7160
7140         addstockPFlagTrimI0% = addstockPFlagTrimI0% - 1
7150         GOTO 7120
7160     REM END WHILE
7170     addstockPFlag0$ = LEFT$(invFlagBuf$, addstockPFlagTrimI0%)
7180     addstockPDescTrimI0% = LEN(invDescBuf$)
7190     IF (addstockPDescTrimI0% > 0) = 0 THEN GOTO 7230
7200     IF (MID$(invDescBuf$, addstockPDescTrimI0%, 1) = " ") = 0 THEN GOTO 7230
7210         addstockPDescTrimI0% = addstockPDescTrimI0% - 1
7220         GOTO 7190
7230     REM END WHILE
7240     addstockPDesc0$ = LEFT$(invDescBuf$, addstockPDescTrimI0%)
7250     addstockPQty0% = CVI(invQtyBuf$)
7260     addstockPReorder0% = CVI(invReorderBuf$)
7270     addstockPPrice0! = CVS(invPriceBuf$)
7280     isemptyFlag0$ = addstockPFlag0$
7290     GOSUB 3480
7300     IF (isemptyResult0%) = 0 THEN GOTO 7350
7310         shownullentrymessagePartStr0$ = addstockPartStr0$
7320         GOSUB 4190
7330         GOSUB 3660
7340         RETURN
7350     REM END IF

7360         showaddstockscreenPartNum0% = addstockPart0%
7370         showaddstockscreenDesc0$ = addstockPDesc0$
7380         showaddstockscreenQty0% = addstockPQty0%
7390         showaddstockscreenReorder0% = addstockPReorder0%
7400         GOSUB 4860
7410         LOCATE 14, tabcol%
7420         INPUT " Quantity to add"; addstockAddStr0$
7430         addstockAddAmt0% = VAL(addstockAddStr0$)
7440         IF (addstockAddAmt0% < 0) = 0 THEN GOTO 7470
7450             GOSUB 5020
7460             GOSUB 3660
7470         REM END IF
7480         IF (addstockAddAmt0% >= 0) = 0 THEN GOTO 7360
7490     REM END DO

7500     addstockPQty0% = addstockPQty0% + addstockAddAmt0%
7510     ' inv[...] = p  (write back a let-bound record)
7520     LSET invFlagBuf$ = addstockPFlag0$
7530     LSET invDescBuf$ = addstockPDesc0$
7540     LSET invQtyBuf$ = MKI$(addstockPQty0%)
7550     LSET invReorderBuf$ = MKI$(addstockPReorder0%)
7560     LSET invPriceBuf$ = MKS$(addstockPPrice0!)
7570     PUT #1, addstockPart0%
7580     RETURN
7590 ' end procedure addstock

7600 ' procedure subtractstock()
7610     CLS
7620     LOCATE 5, 20
7630     PRINT "S U B T R A C T I N G    S T O C K"

7640         LOCATE 8, 25
7650         GOSUB 3610
7660         subtractstockPartStr0$ = readpartnumberinputResult0$
7670         subtractstockPart0% = VAL(subtractstockPartStr0$)
7680         partinrangeN0% = subtractstockPart0%
7690         GOSUB 3520
7700         subtractstockValidPart0% = partinrangeResult0%
7710         IF (subtractstockValidPart0% = 0) = 0 THEN GOTO 7740
7720             GOSUB 4120
7730             GOSUB 3660
7740         REM END IF
7750         IF (subtractstockValidPart0% <> 0) = 0 THEN GOTO 7640
7760     REM END DO

7770     ' let p = inv[...]  (whole-record read)
7780     GET #1, subtractstockPart0%
7790     subtractstockPFlagTrimI0% = LEN(invFlagBuf$)
7800     IF (subtractstockPFlagTrimI0% > 0) = 0 THEN GOTO 7840
7810     IF (MID$(invFlagBuf$, subtractstockPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 7840
7820         subtractstockPFlagTrimI0% = subtractstockPFlagTrimI0% - 1
7830         GOTO 7800
7840     REM END WHILE
7850     subtractstockPFlag0$ = LEFT$(invFlagBuf$, subtractstockPFlagTrimI0%)
7860     subtractstockPDescTrimI0% = LEN(invDescBuf$)
7870     IF (subtractstockPDescTrimI0% > 0) = 0 THEN GOTO 7910
7880     IF (MID$(invDescBuf$, subtractstockPDescTrimI0%, 1) = " ") = 0 THEN GOTO 7910
7890         subtractstockPDescTrimI0% = subtractstockPDescTrimI0% - 1
7900         GOTO 7870
7910     REM END WHILE
7920     subtractstockPDesc0$ = LEFT$(invDescBuf$, subtractstockPDescTrimI0%)
7930     subtractstockPQty0% = CVI(invQtyBuf$)
7940     subtractstockPReorder0% = CVI(invReorderBuf$)
7950     subtractstockPPrice0! = CVS(invPriceBuf$)
7960     isemptyFlag0$ = subtractstockPFlag0$
7970     GOSUB 3480
7980     IF (isemptyResult0%) = 0 THEN GOTO 8030
7990         shownullentrymessagePartStr0$ = subtractstockPartStr0$
8000         GOSUB 4190
8010         GOSUB 3660
8020         RETURN
8030     REM END IF

8040         showsubtractstockscreenPartNum0% = subtractstockPart0%
8050         showsubtractstockscreenDesc0$ = subtractstockPDesc0$
8060         showsubtractstockscreenQty0% = subtractstockPQty0%
8070         showsubtractstockscreenReorder0% = subtractstockPReorder0%
8080         GOSUB 5090
8090         LOCATE 14, tabcol%
8100         INPUT "Quantity to subtract"; subtractstockSubStr0$
8110         subtractstockSubAmt0% = VAL(subtractstockSubStr0$)
8120         subtractstockOverSubtract0% = 0
8130         IF (subtractstockSubAmt0% >= 0) = 0 THEN GOTO 8190
8140         IF ((subtractstockPQty0% - subtractstockSubAmt0%) < 0) = 0 THEN GOTO 8190
8150             subtractstockOverSubtract0% = 1
8160             showoversubtractwarningOnHand0% = subtractstockPQty0%
8170             GOSUB 5250
8180             GOSUB 3660
8190         REM END IF
8200         IF (subtractstockSubAmt0% >= 0) = 0 THEN GOTO 8040
8210         IF (subtractstockOverSubtract0% = 0) = 0 THEN GOTO 8040
8220     REM END DO

8230     subtractstockPQty0% = subtractstockPQty0% - subtractstockSubAmt0%
8240     IF (subtractstockPQty0% <= subtractstockPReorder0%) = 0 THEN GOTO 8260
8250         LOCATE 16, tabcol%
8260     REM END IF
8270     PRINT (("quantity now" + STR$(subtractstockPQty0%)) + " reorder level") + STR$(subtractstockPReorder0%)
8280     ' inv[...] = p  (write back a let-bound record)
8290     LSET invFlagBuf$ = subtractstockPFlag0$
8300     LSET invDescBuf$ = subtractstockPDesc0$
8310     LSET invQtyBuf$ = MKI$(subtractstockPQty0%)
8320     LSET invReorderBuf$ = MKI$(subtractstockPReorder0%)
8330     LSET invPriceBuf$ = MKS$(subtractstockPPrice0!)
8340     PUT #1, subtractstockPart0%
8350     RETURN
8360 ' end procedure subtractstock

8370 ' procedure reorderreport()
8380     GOSUB 4520
8390     reorderreportReportLineCount0% = 0
8400     FOR reorderreportI0% = 1 TO partcount%
8410         ' let p = inv[...]  (whole-record read)
8420         GET #1, reorderreportI0%
8430         reorderreportPFlagTrimI0% = LEN(invFlagBuf$)
8440         IF (reorderreportPFlagTrimI0% > 0) = 0 THEN GOTO 8480
8450         IF (MID$(invFlagBuf$, reorderreportPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 8480
8460             reorderreportPFlagTrimI0% = reorderreportPFlagTrimI0% - 1
8470             GOTO 8440
8480         REM END WHILE
8490         reorderreportPFlag0$ = LEFT$(invFlagBuf$, reorderreportPFlagTrimI0%)
8500         reorderreportPDescTrimI0% = LEN(invDescBuf$)
8510         IF (reorderreportPDescTrimI0% > 0) = 0 THEN GOTO 8550
8520         IF (MID$(invDescBuf$, reorderreportPDescTrimI0%, 1) = " ") = 0 THEN GOTO 8550
8530             reorderreportPDescTrimI0% = reorderreportPDescTrimI0% - 1
8540             GOTO 8510
8550         REM END WHILE
8560         reorderreportPDesc0$ = LEFT$(invDescBuf$, reorderreportPDescTrimI0%)
8570         reorderreportPQty0% = CVI(invQtyBuf$)
8580         reorderreportPReorder0% = CVI(invReorderBuf$)
8590         reorderreportPPrice0! = CVS(invPriceBuf$)
8600         IF (reorderreportPQty0% < reorderreportPReorder0%) = 0 THEN GOTO 8710
8610             printreorderlinePartNum0% = reorderreportI0%
8620             printreorderlineDesc0$ = reorderreportPDesc0$
8630             printreorderlineQty0% = reorderreportPQty0%
8640             printreorderlineReorder0% = reorderreportPReorder0%
8650             GOSUB 4620
8660             reorderreportReportLineCount0% = reorderreportReportLineCount0% + 1
8670             IF (reorderreportReportLineCount0% > 15) = 0 THEN GOTO 8700
8680                 GOSUB 3730
8690                 reorderreportReportLineCount0% = 0
8700             REM END IF
8710         REM END IF
8720     NEXT reorderreportI0%
8730     GOSUB 3730
8740     RETURN
8750 ' end procedure reorderreport

8760 ' procedure initializeinventoryfileifnew()
8770     ' let p = inv[...]  (whole-record read)
8780     GET #1, 1
8790     initializeinventoryfileifnewPFlagTrimI0% = LEN(invFlagBuf$)
8800     IF (initializeinventoryfileifnewPFlagTrimI0% > 0) = 0 THEN GOTO 8840
8810     IF (MID$(invFlagBuf$, initializeinventoryfileifnewPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 8840
8820         initializeinventoryfileifnewPFlagTrimI0% = initializeinventoryfileifnewPFlagTrimI0% - 1
8830         GOTO 8800
8840     REM END WHILE
8850     initializeinventoryfileifnewPFlag0$ = LEFT$(invFlagBuf$, initializeinventoryfileifnewPFlagTrimI0%)
8860     initializeinventoryfileifnewPDescTrimI0% = LEN(invDescBuf$)
8870     IF (initializeinventoryfileifnewPDescTrimI0% > 0) = 0 THEN GOTO 8910
8880     IF (MID$(invDescBuf$, initializeinventoryfileifnewPDescTrimI0%, 1) = " ") = 0 THEN GOTO 8910
8890         initializeinventoryfileifnewPDescTrimI0% = initializeinventoryfileifnewPDescTrimI0% - 1
8900         GOTO 8870
8910     REM END WHILE
8920     initializeinventoryfileifnewPDesc0$ = LEFT$(invDescBuf$, initializeinventoryfileifnewPDescTrimI0%)
8930     initializeinventoryfileifnewPQty0% = CVI(invQtyBuf$)
8940     initializeinventoryfileifnewPReorder0% = CVI(invReorderBuf$)
8950     initializeinventoryfileifnewPPrice0! = CVS(invPriceBuf$)
8960     IF (ASC(initializeinventoryfileifnewPFlag0$) = 0) = 0 THEN GOTO 9060
8970         FOR initializeinventoryfileifnewI0% = 1 TO partcount%
8980             ' inv[...] = { ... }  (whole-record write)
8990             LSET invFlagBuf$ = CHR$(255)
9000             LSET invDescBuf$ = ""
9010             LSET invQtyBuf$ = MKI$(0)
9020             LSET invReorderBuf$ = MKI$(0)
9030             LSET invPriceBuf$ = MKS$(0)
9040             PUT #1, initializeinventoryfileifnewI0%
9050         NEXT initializeinventoryfileifnewI0%
9060     REM END IF
9070     RETURN
9080 ' end procedure initializeinventoryfileifnew

9090 ' procedure reportinventoryerror(err%, erl%)
9100     LOCATE 25, 1
9110     errorCode0% = reportinventoryerrorErr0%
9120     GOSUB 2080
9130     PRINT (("There has been an error on line" + STR$(reportinventoryerrorErl0%)) + ": ") + errorResult0$
9140     GOSUB 3660
9150     reportinventoryerrorK0$ = readkeyResult0$
9160     RETURN
9170 ' end procedure reportinventoryerror
