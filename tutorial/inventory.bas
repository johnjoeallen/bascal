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
1290 GOSUB 8780

1300     GOSUB 3810
1310     GOSUB 3660
1320     kp$ = readkeyResult0$
1330     IF (INSTR("12345678cCeElLaAsSrRqQxX", kp$) <> 0) = 0 THEN GOTO 1880
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
1470         ON ERROR GOTO 1810
1480             BCCT5$ = kp$
1490             IF (BCCT5$ = "1" OR BCCT5$ = "c" OR BCCT5$ = "C") <> 0 THEN GOTO 1580
1500             IF (BCCT5$ = "2" OR BCCT5$ = "e" OR BCCT5$ = "E") <> 0 THEN GOTO 1600
1510             IF (BCCT5$ = "3" OR BCCT5$ = "l" OR BCCT5$ = "L") <> 0 THEN GOTO 1620
1520             IF (BCCT5$ = "4" OR BCCT5$ = "a" OR BCCT5$ = "A") <> 0 THEN GOTO 1640
1530             IF (BCCT5$ = "5" OR BCCT5$ = "s" OR BCCT5$ = "S") <> 0 THEN GOTO 1660
1540             IF (BCCT5$ = "6" OR BCCT5$ = "r" OR BCCT5$ = "R") <> 0 THEN GOTO 1680
1550             IF (BCCT5$ = "7" OR BCCT5$ = "q" OR BCCT5$ = "Q") <> 0 THEN GOTO 1700
1560             IF (BCCT5$ = "8" OR BCCT5$ = "x" OR BCCT5$ = "X") <> 0 THEN GOTO 1720
1570             GOTO 1780
1580                 GOSUB 5350
1590                 GOTO 1780
1600                 GOSUB 5890
1610                 GOTO 1780
1620                 GOSUB 6580
1630                 GOTO 1780
1640                 GOSUB 6940
1650                 GOTO 1780
1660                 GOSUB 7620
1670                 GOTO 1780
1680                 GOSUB 8390
1690                 GOTO 1780
1700                 quitflag% = 1
1710                 GOTO 1780
1720                 ' BASCAL-ism: `inv.close()` is sugar for `CLOSE #1`,
1730                 ' matching fhb's own `90 CLOSE:SYSTEM`.
1740                 ' inv.close()
1750                 CLOSE #1
1760                 SYSTEM
1770                 GOTO 1780
1780             REM END SELECT
1790         ON ERROR GOTO 0
1800         GOTO 1870
1810             err% = ERR
1820             erl% = ERL
1830             reportinventoryerrorErr0% = err%
1840             reportinventoryerrorErl0% = erl%
1850             GOSUB 9110
1860             RESUME 1870
1870         REM END TRY
1880     REM END IF
1890     IF (quitflag% = 1) = 0 THEN GOTO 1300
1900 REM END DO

1910 ' inv.close()
1920 CLOSE #1
1930 END

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
4020     PRINT TAB(tabcol%)"7......Q)uit to BASIC"
4030     PRINT TAB(tabcol%)"8......eX)it to system"
4040     RETURN
4050 ' end procedure showmainmenu

4060 ' procedure showbadpartnumber()
4070     CLS
4080     LOCATE 10, 10
4090     PRINT "Part number is out of permissable range of 1 to" + STR$(partcount%)
4100     RETURN
4110 ' end procedure showbadpartnumber

4120 ' procedure showrangeretrymessage()
4130     LOCATE 10, 15
4140     PRINT "The Part number is out of permissable range of 1 to" + STR$(partcount%)
4150     LOCATE 25, 15
4160     PRINT "Press the Anykey to reenter part number...";
4170     RETURN
4180 ' end procedure showrangeretrymessage

4190 ' procedure shownullentrymessage(partstr$)
4200     LOCATE 10, tabcol%
4210     PRINT ("Part number " + shownullentrymessagePartStr0$) + " is a null entry"
4220     RETURN
4230 ' end procedure shownullentrymessage

4240 ' procedure showpartstatus(partnum%, desc$, qty%, reorder%, price!)
4250     CLS
4260     LOCATE 5, 1
4270     PRINT TAB(tabcol%)"Inventory Status for Individual Part Number"
4280     PRINT TAB(tabcol%)"==========================================="
4290     PRINT
4300     PRINT
4310     PRINT TAB(tabcol%)"     Part number:  " + STR$(showpartstatusPartNum0%)
4320     PRINT
4330     PRINT TAB(tabcol%)"       Item name:  " + showpartstatusDesc0$
4340     PRINT TAB(tabcol%)"Quantity on hand:  " + STR$(showpartstatusQty0%)
4350     PRINT TAB(tabcol%)"   Reorder level:  " + STR$(showpartstatusReorder0%)
4360     PRINT TAB(tabcol%)"      Unit price:  " + STR$(showpartstatusPrice0!)
4370     RETURN
4380 ' end procedure showpartstatus

4390 ' procedure printlistheader()
4400     CLS
4410     PRINT TAB(25)"I N V E N T O R Y   L I S T I N G"; TAB(65); STR$(partcount%) + "items"
4420     PRINT "                                          Quantity       Reorder"
4430     PRINT " Partno           Description             on hand         level"
4440     LOCATE 25, 1
4450     PRINT "Press the AnyKey to scroll listing...";
4460     RETURN
4470 ' end procedure printlistheader

4480 ' procedure printinventoryline(partnum%, desc$, qty%, reorder%)
4490     PRINT (((((STR$(printinventorylinePartNum0%) + "  ") + printinventorylineDesc0$) + "   ") + STR$(printinventorylineQty0%)) + "          ") + STR$(printinventorylineReorder0%)
4500     RETURN
4510 ' end procedure printinventoryline

4520 ' procedure printreorderheader()
4530     CLS
4540     LOCATE 1, tabcol%
4550     PRINT "Reorder Report"; TAB(55); DATE$
4560     PRINT
4570     PRINT "                                             Quantity       Reorder"
4580     PRINT "    Partno           Description             on hand         level"
4590     PRINT "   =======  ==============================   ========       ======="
4600     RETURN
4610 ' end procedure printreorderheader

4620 ' procedure printreorderline(partnum%, desc$, qty%, reorder%)
4630     PRINT (((((("  " + STR$(printreorderlinePartNum0%)) + "  ") + printreorderlineDesc0$) + "   ") + STR$(printreorderlineQty0%)) + "          ") + STR$(printreorderlineReorder0%)
4640     RETURN
4650 ' end procedure printreorderline

4660 ' procedure gatherpartdetails(partnum%, desc$, qty%, reorder%, price!)
4670     CLS
4680     LOCATE 4, tabcol%
4690     PRINT "Adding or Overwriting a Record"
4700     LOCATE 8, tabcol%
4710     PRINT "Record/Partno" + STR$(gatherpartdetailsPartNum0%)
4720     LOCATE 11, 39
4730     PRINT "------------------------------"
4740     LOCATE 10, tabcol%
4750     INPUT "      Description"; gatherpartdetailsDesc0$
4760     LOCATE 12, tabcol%
4770     INPUT "Quantity in stock"; gatherpartdetailsQty0%
4780     LOCATE 14, tabcol%
4790     INPUT "    Reorder level"; gatherpartdetailsReorder0%
4800     LOCATE 16, tabcol%
4810     INPUT "       Unit price"; gatherpartdetailsPrice0!
4820     LOCATE 18, tabcol%
4830     PRINT "Is information correct (Y/N)?"
4840     RETURN
4850 ' end procedure gatherpartdetails

4860 ' procedure showaddstockscreen(partnum%, desc$, qty%, reorder%)
4870     CLS
4880     LOCATE 4, 25
4890     PRINT "Add to an inventory part number"
4900     LOCATE 5, 25
4910     PRINT "==============================="
4920     LOCATE 8, tabcol%
4930     PRINT "     Part number: " + STR$(showaddstockscreenPartNum0%)
4940     LOCATE 9, tabcol%
4950     PRINT "Item description: " + showaddstockscreenDesc0$
4960     LOCATE 10, tabcol%
4970     PRINT "Quantity on hand: " + STR$(showaddstockscreenQty0%)
4980     LOCATE 11, tabcol%
4990     PRINT "   Reorder Level: " + STR$(showaddstockscreenReorder0%)
5000     RETURN
5010 ' end procedure showaddstockscreen

5020 ' procedure shownegativeqtywarning()
5030     LOCATE 17, 15
5040     PRINT "The quantity to add must NOT be a negative number"
5050     LOCATE 25, 1
5060     PRINT "Please press the Anykey to reenter quantity to add...";
5070     RETURN
5080 ' end procedure shownegativeqtywarning

5090 ' procedure showsubtractstockscreen(partnum%, desc$, qty%, reorder%)
5100     CLS
5110     LOCATE 4, tabcol%
5120     PRINT "Subtract an inventory part number"
5130     LOCATE 5, tabcol%
5140     PRINT "================================="
5150     LOCATE 8, tabcol%
5160     PRINT "         Part number: " + STR$(showsubtractstockscreenPartNum0%)
5170     LOCATE 9, tabcol%
5180     PRINT "    Item description: " + showsubtractstockscreenDesc0$
5190     LOCATE 10, tabcol%
5200     PRINT "    Quantity on hand: " + STR$(showsubtractstockscreenQty0%)
5210     LOCATE 11, tabcol%
5220     PRINT "       Reorder Level: " + STR$(showsubtractstockscreenReorder0%)
5230     RETURN
5240 ' end procedure showsubtractstockscreen

5250 ' procedure showoversubtractwarning(onhand%)
5260     LOCATE 17, 5
5270     PRINT "The quantity to SUBTRACT must NOT result in NEGATIVE inventory"
5280     LOCATE 18, 5
5290     PRINT ("Only" + STR$(showoversubtractwarningOnHand0%)) + " IN STOCK"
5300     LOCATE 25, 1
5310     PRINT "Please press the Anykey to reenter quantity to subtract...";
5320     RETURN
5330 ' end procedure showoversubtractwarning

5340 ' procedure checkpart()
5350     GOSUB 3610
5360     checkpartPartStr0$ = readpartnumberinputResult0$
5370     checkpartPart0% = VAL(checkpartPartStr0$)
5380     partinrangeN0% = checkpartPart0%
5390     GOSUB 3520
5400     IF (partinrangeResult0% = 0) = 0 THEN GOTO 5440
5410         GOSUB 4070
5420         GOSUB 3730
5430         RETURN
5440     REM END IF
5450     ' BASCAL-ism: `let p = inv[part%]` reads record `part%` of the
5460     ' `inv` file into a local record variable `p` -- one expression
5470     ' for what fhb's `GET #1, PART!` plus five separate field reads
5480     ' (F$, D$, CVI(Q$), CVI(R$), CVS(P$)) did by hand. The write
5490     ' side, `inv[part%] = { ... }` (see editRecord() below), is the
5500     ' same sugar for PUT plus the LSET/MKx$ packing it replaces.
5510     ' let p = inv[...]  (whole-record read)
5520     GET #1, checkpartPart0%
5530     checkpartPFlagTrimI0% = LEN(invFlagBuf$)
5540     IF (checkpartPFlagTrimI0% > 0) = 0 THEN GOTO 5580
5550     IF (MID$(invFlagBuf$, checkpartPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 5580
5560         checkpartPFlagTrimI0% = checkpartPFlagTrimI0% - 1
5570         GOTO 5540
5580     REM END WHILE
5590     checkpartPFlag0$ = LEFT$(invFlagBuf$, checkpartPFlagTrimI0%)
5600     checkpartPDescTrimI0% = LEN(invDescBuf$)
5610     IF (checkpartPDescTrimI0% > 0) = 0 THEN GOTO 5650
5620     IF (MID$(invDescBuf$, checkpartPDescTrimI0%, 1) = " ") = 0 THEN GOTO 5650
5630         checkpartPDescTrimI0% = checkpartPDescTrimI0% - 1
5640         GOTO 5610
5650     REM END WHILE
5660     checkpartPDesc0$ = LEFT$(invDescBuf$, checkpartPDescTrimI0%)
5670     checkpartPQty0% = CVI(invQtyBuf$)
5680     checkpartPReorder0% = CVI(invReorderBuf$)
5690     checkpartPPrice0! = CVS(invPriceBuf$)
5700     isemptyFlag0$ = checkpartPFlag0$
5710     GOSUB 3480
5720     IF (isemptyResult0%) = 0 THEN GOTO 5780
5730         CLS
5740         LOCATE 10, 18
5750         PRINT ("Part number" + STR$(checkpartPart0%)) + "is still a null entry at this time"
5760         GOSUB 3730
5770         RETURN
5780     REM END IF
5790     showpartstatusPartNum0% = checkpartPart0%
5800     showpartstatusDesc0$ = checkpartPDesc0$
5810     showpartstatusQty0% = checkpartPQty0%
5820     showpartstatusReorder0% = checkpartPReorder0%
5830     showpartstatusPrice0! = checkpartPPrice0!
5840     GOSUB 4250
5850     GOSUB 3730
5860     RETURN
5870 ' end procedure checkpart

5880 ' procedure editrecord()
5890     CLS
5900     LOCATE 10, tabcol%
5910     GOSUB 3610
5920     editrecordPartStr0$ = readpartnumberinputResult0$
5930     editrecordPart0% = VAL(editrecordPartStr0$)
5940     partinrangeN0% = editrecordPart0%
5950     GOSUB 3520
5960     IF (partinrangeResult0% = 0) = 0 THEN GOTO 6000
5970         GOSUB 4070
5980         GOSUB 3730
5990         RETURN
6000     REM END IF
6010     ' let p = inv[...]  (whole-record read)
6020     GET #1, editrecordPart0%
6030     editrecordPFlagTrimI0% = LEN(invFlagBuf$)
6040     IF (editrecordPFlagTrimI0% > 0) = 0 THEN GOTO 6080
6050     IF (MID$(invFlagBuf$, editrecordPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 6080
6060         editrecordPFlagTrimI0% = editrecordPFlagTrimI0% - 1
6070         GOTO 6040
6080     REM END WHILE
6090     editrecordPFlag0$ = LEFT$(invFlagBuf$, editrecordPFlagTrimI0%)
6100     editrecordPDescTrimI0% = LEN(invDescBuf$)
6110     IF (editrecordPDescTrimI0% > 0) = 0 THEN GOTO 6150
6120     IF (MID$(invDescBuf$, editrecordPDescTrimI0%, 1) = " ") = 0 THEN GOTO 6150
6130         editrecordPDescTrimI0% = editrecordPDescTrimI0% - 1
6140         GOTO 6110
6150     REM END WHILE
6160     editrecordPDesc0$ = LEFT$(invDescBuf$, editrecordPDescTrimI0%)
6170     editrecordPQty0% = CVI(invQtyBuf$)
6180     editrecordPReorder0% = CVI(invReorderBuf$)
6190     editrecordPPrice0! = CVS(invPriceBuf$)
6200     isemptyFlag0$ = editrecordPFlag0$
6210     GOSUB 3480
6220     IF (isemptyResult0% = 0) = 0 THEN GOTO 6310
6230         LOCATE 12, tabcol%
6240         PRINT "Overwrite existing part data?"
6250         GOSUB 3660
6260         editrecordKp0$ = readkeyResult0$
6270         IF (editrecordKp0$ <> "Y") = 0 THEN GOTO 6300
6280         IF (editrecordKp0$ <> "y") = 0 THEN GOTO 6300
6290             RETURN
6300         REM END IF
6310     REM END IF

6320         gatherpartdetailsPartNum0% = editrecordPart0%
6330         gatherpartdetailsDesc0$ = editrecordEditDesc0$
6340         gatherpartdetailsQty0% = editrecordEditQty0%
6350         gatherpartdetailsReorder0% = editrecordEditReorder0%
6360         gatherpartdetailsPrice0! = editrecordEditPrice0!
6370         GOSUB 4670
6380         editrecordEditDesc0$ = gatherpartdetailsDesc0$
6390         editrecordEditQty0% = gatherpartdetailsQty0%
6400         editrecordEditReorder0% = gatherpartdetailsReorder0%
6410         editrecordEditPrice0! = gatherpartdetailsPrice0!
6420         GOSUB 3660
6430         editrecordKp0$ = readkeyResult0$
6440         IF (editrecordKp0$ = "Y") <> 0 THEN GOTO 6470
6450         IF (editrecordKp0$ = "y") <> 0 THEN GOTO 6470
6460         GOTO 6320
6470     REM END DO
6480     ' inv[...] = { ... }  (whole-record write)
6490     LSET invFlagBuf$ = "1"
6500     LSET invDescBuf$ = editrecordEditDesc0$
6510     LSET invQtyBuf$ = MKI$(editrecordEditQty0%)
6520     LSET invReorderBuf$ = MKI$(editrecordEditReorder0%)
6530     LSET invPriceBuf$ = MKS$(editrecordEditPrice0!)
6540     PUT #1, editrecordPart0%
6550     RETURN
6560 ' end procedure editrecord

6570 ' procedure listall()
6580     GOSUB 4400
6590     listallScrollCount0% = 0
6600     FOR listallI0% = 1 TO partcount%
6610         ' let p = inv[...]  (whole-record read)
6620         GET #1, listallI0%
6630         listallPFlagTrimI0% = LEN(invFlagBuf$)
6640         IF (listallPFlagTrimI0% > 0) = 0 THEN GOTO 6680
6650         IF (MID$(invFlagBuf$, listallPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 6680
6660             listallPFlagTrimI0% = listallPFlagTrimI0% - 1
6670             GOTO 6640
6680         REM END WHILE
6690         listallPFlag0$ = LEFT$(invFlagBuf$, listallPFlagTrimI0%)
6700         listallPDescTrimI0% = LEN(invDescBuf$)
6710         IF (listallPDescTrimI0% > 0) = 0 THEN GOTO 6750
6720         IF (MID$(invDescBuf$, listallPDescTrimI0%, 1) = " ") = 0 THEN GOTO 6750
6730             listallPDescTrimI0% = listallPDescTrimI0% - 1
6740             GOTO 6710
6750         REM END WHILE
6760         listallPDesc0$ = LEFT$(invDescBuf$, listallPDescTrimI0%)
6770         listallPQty0% = CVI(invQtyBuf$)
6780         listallPReorder0% = CVI(invReorderBuf$)
6790         listallPPrice0! = CVS(invPriceBuf$)
6800         printinventorylinePartNum0% = listallI0%
6810         printinventorylineDesc0$ = listallPDesc0$
6820         printinventorylineQty0% = listallPQty0%
6830         printinventorylineReorder0% = listallPReorder0%
6840         GOSUB 4490
6850         listallScrollCount0% = listallScrollCount0% + 1
6860         IF (listallScrollCount0% = 20) = 0 THEN GOTO 6890
6870             GOSUB 3730
6880             listallScrollCount0% = 0
6890         REM END IF
6900     NEXT listallI0%
6910     RETURN
6920 ' end procedure listall

6930 ' procedure addstock()
6940     CLS
6950     LOCATE 5, 25
6960     PRINT "A D D I N G   S T O C K"

6970         LOCATE 8, 25
6980         GOSUB 3610
6990         addstockPartStr0$ = readpartnumberinputResult0$
7000         addstockPart0% = VAL(addstockPartStr0$)
7010         partinrangeN0% = addstockPart0%
7020         GOSUB 3520
7030         addstockValidPart0% = partinrangeResult0%
7040         IF (addstockValidPart0% = 0) = 0 THEN GOTO 7070
7050             GOSUB 4130
7060             GOSUB 3660
7070         REM END IF
7080         IF (addstockValidPart0% <> 0) = 0 THEN GOTO 6970
7090     REM END DO

7100     ' let p = inv[...]  (whole-record read)
7110     GET #1, addstockPart0%
7120     addstockPFlagTrimI0% = LEN(invFlagBuf$)
7130     IF (addstockPFlagTrimI0% > 0) = 0 THEN GOTO 7170
7140     IF (MID$(invFlagBuf$, addstockPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 7170
7150         addstockPFlagTrimI0% = addstockPFlagTrimI0% - 1
7160         GOTO 7130
7170     REM END WHILE
7180     addstockPFlag0$ = LEFT$(invFlagBuf$, addstockPFlagTrimI0%)
7190     addstockPDescTrimI0% = LEN(invDescBuf$)
7200     IF (addstockPDescTrimI0% > 0) = 0 THEN GOTO 7240
7210     IF (MID$(invDescBuf$, addstockPDescTrimI0%, 1) = " ") = 0 THEN GOTO 7240
7220         addstockPDescTrimI0% = addstockPDescTrimI0% - 1
7230         GOTO 7200
7240     REM END WHILE
7250     addstockPDesc0$ = LEFT$(invDescBuf$, addstockPDescTrimI0%)
7260     addstockPQty0% = CVI(invQtyBuf$)
7270     addstockPReorder0% = CVI(invReorderBuf$)
7280     addstockPPrice0! = CVS(invPriceBuf$)
7290     isemptyFlag0$ = addstockPFlag0$
7300     GOSUB 3480
7310     IF (isemptyResult0%) = 0 THEN GOTO 7360
7320         shownullentrymessagePartStr0$ = addstockPartStr0$
7330         GOSUB 4200
7340         GOSUB 3660
7350         RETURN
7360     REM END IF

7370         showaddstockscreenPartNum0% = addstockPart0%
7380         showaddstockscreenDesc0$ = addstockPDesc0$
7390         showaddstockscreenQty0% = addstockPQty0%
7400         showaddstockscreenReorder0% = addstockPReorder0%
7410         GOSUB 4870
7420         LOCATE 14, tabcol%
7430         INPUT " Quantity to add"; addstockAddStr0$
7440         addstockAddAmt0% = VAL(addstockAddStr0$)
7450         IF (addstockAddAmt0% < 0) = 0 THEN GOTO 7480
7460             GOSUB 5030
7470             GOSUB 3660
7480         REM END IF
7490         IF (addstockAddAmt0% >= 0) = 0 THEN GOTO 7370
7500     REM END DO

7510     addstockPQty0% = addstockPQty0% + addstockAddAmt0%
7520     ' inv[...] = p  (write back a let-bound record)
7530     LSET invFlagBuf$ = addstockPFlag0$
7540     LSET invDescBuf$ = addstockPDesc0$
7550     LSET invQtyBuf$ = MKI$(addstockPQty0%)
7560     LSET invReorderBuf$ = MKI$(addstockPReorder0%)
7570     LSET invPriceBuf$ = MKS$(addstockPPrice0!)
7580     PUT #1, addstockPart0%
7590     RETURN
7600 ' end procedure addstock

7610 ' procedure subtractstock()
7620     CLS
7630     LOCATE 5, 20
7640     PRINT "S U B T R A C T I N G    S T O C K"

7650         LOCATE 8, 25
7660         GOSUB 3610
7670         subtractstockPartStr0$ = readpartnumberinputResult0$
7680         subtractstockPart0% = VAL(subtractstockPartStr0$)
7690         partinrangeN0% = subtractstockPart0%
7700         GOSUB 3520
7710         subtractstockValidPart0% = partinrangeResult0%
7720         IF (subtractstockValidPart0% = 0) = 0 THEN GOTO 7750
7730             GOSUB 4130
7740             GOSUB 3660
7750         REM END IF
7760         IF (subtractstockValidPart0% <> 0) = 0 THEN GOTO 7650
7770     REM END DO

7780     ' let p = inv[...]  (whole-record read)
7790     GET #1, subtractstockPart0%
7800     subtractstockPFlagTrimI0% = LEN(invFlagBuf$)
7810     IF (subtractstockPFlagTrimI0% > 0) = 0 THEN GOTO 7850
7820     IF (MID$(invFlagBuf$, subtractstockPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 7850
7830         subtractstockPFlagTrimI0% = subtractstockPFlagTrimI0% - 1
7840         GOTO 7810
7850     REM END WHILE
7860     subtractstockPFlag0$ = LEFT$(invFlagBuf$, subtractstockPFlagTrimI0%)
7870     subtractstockPDescTrimI0% = LEN(invDescBuf$)
7880     IF (subtractstockPDescTrimI0% > 0) = 0 THEN GOTO 7920
7890     IF (MID$(invDescBuf$, subtractstockPDescTrimI0%, 1) = " ") = 0 THEN GOTO 7920
7900         subtractstockPDescTrimI0% = subtractstockPDescTrimI0% - 1
7910         GOTO 7880
7920     REM END WHILE
7930     subtractstockPDesc0$ = LEFT$(invDescBuf$, subtractstockPDescTrimI0%)
7940     subtractstockPQty0% = CVI(invQtyBuf$)
7950     subtractstockPReorder0% = CVI(invReorderBuf$)
7960     subtractstockPPrice0! = CVS(invPriceBuf$)
7970     isemptyFlag0$ = subtractstockPFlag0$
7980     GOSUB 3480
7990     IF (isemptyResult0%) = 0 THEN GOTO 8040
8000         shownullentrymessagePartStr0$ = subtractstockPartStr0$
8010         GOSUB 4200
8020         GOSUB 3660
8030         RETURN
8040     REM END IF

8050         showsubtractstockscreenPartNum0% = subtractstockPart0%
8060         showsubtractstockscreenDesc0$ = subtractstockPDesc0$
8070         showsubtractstockscreenQty0% = subtractstockPQty0%
8080         showsubtractstockscreenReorder0% = subtractstockPReorder0%
8090         GOSUB 5100
8100         LOCATE 14, tabcol%
8110         INPUT "Quantity to subtract"; subtractstockSubStr0$
8120         subtractstockSubAmt0% = VAL(subtractstockSubStr0$)
8130         subtractstockOverSubtract0% = 0
8140         IF (subtractstockSubAmt0% >= 0) = 0 THEN GOTO 8200
8150         IF ((subtractstockPQty0% - subtractstockSubAmt0%) < 0) = 0 THEN GOTO 8200
8160             subtractstockOverSubtract0% = 1
8170             showoversubtractwarningOnHand0% = subtractstockPQty0%
8180             GOSUB 5260
8190             GOSUB 3660
8200         REM END IF
8210         IF (subtractstockSubAmt0% >= 0) = 0 THEN GOTO 8050
8220         IF (subtractstockOverSubtract0% = 0) = 0 THEN GOTO 8050
8230     REM END DO

8240     subtractstockPQty0% = subtractstockPQty0% - subtractstockSubAmt0%
8250     IF (subtractstockPQty0% <= subtractstockPReorder0%) = 0 THEN GOTO 8270
8260         LOCATE 16, tabcol%
8270     REM END IF
8280     PRINT (("quantity now" + STR$(subtractstockPQty0%)) + " reorder level") + STR$(subtractstockPReorder0%)
8290     ' inv[...] = p  (write back a let-bound record)
8300     LSET invFlagBuf$ = subtractstockPFlag0$
8310     LSET invDescBuf$ = subtractstockPDesc0$
8320     LSET invQtyBuf$ = MKI$(subtractstockPQty0%)
8330     LSET invReorderBuf$ = MKI$(subtractstockPReorder0%)
8340     LSET invPriceBuf$ = MKS$(subtractstockPPrice0!)
8350     PUT #1, subtractstockPart0%
8360     RETURN
8370 ' end procedure subtractstock

8380 ' procedure reorderreport()
8390     GOSUB 4530
8400     reorderreportReportLineCount0% = 0
8410     FOR reorderreportI0% = 1 TO partcount%
8420         ' let p = inv[...]  (whole-record read)
8430         GET #1, reorderreportI0%
8440         reorderreportPFlagTrimI0% = LEN(invFlagBuf$)
8450         IF (reorderreportPFlagTrimI0% > 0) = 0 THEN GOTO 8490
8460         IF (MID$(invFlagBuf$, reorderreportPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 8490
8470             reorderreportPFlagTrimI0% = reorderreportPFlagTrimI0% - 1
8480             GOTO 8450
8490         REM END WHILE
8500         reorderreportPFlag0$ = LEFT$(invFlagBuf$, reorderreportPFlagTrimI0%)
8510         reorderreportPDescTrimI0% = LEN(invDescBuf$)
8520         IF (reorderreportPDescTrimI0% > 0) = 0 THEN GOTO 8560
8530         IF (MID$(invDescBuf$, reorderreportPDescTrimI0%, 1) = " ") = 0 THEN GOTO 8560
8540             reorderreportPDescTrimI0% = reorderreportPDescTrimI0% - 1
8550             GOTO 8520
8560         REM END WHILE
8570         reorderreportPDesc0$ = LEFT$(invDescBuf$, reorderreportPDescTrimI0%)
8580         reorderreportPQty0% = CVI(invQtyBuf$)
8590         reorderreportPReorder0% = CVI(invReorderBuf$)
8600         reorderreportPPrice0! = CVS(invPriceBuf$)
8610         IF (reorderreportPQty0% < reorderreportPReorder0%) = 0 THEN GOTO 8720
8620             printreorderlinePartNum0% = reorderreportI0%
8630             printreorderlineDesc0$ = reorderreportPDesc0$
8640             printreorderlineQty0% = reorderreportPQty0%
8650             printreorderlineReorder0% = reorderreportPReorder0%
8660             GOSUB 4630
8670             reorderreportReportLineCount0% = reorderreportReportLineCount0% + 1
8680             IF (reorderreportReportLineCount0% > 15) = 0 THEN GOTO 8710
8690                 GOSUB 3730
8700                 reorderreportReportLineCount0% = 0
8710             REM END IF
8720         REM END IF
8730     NEXT reorderreportI0%
8740     GOSUB 3730
8750     RETURN
8760 ' end procedure reorderreport

8770 ' procedure initializeinventoryfileifnew()
8780     ' let p = inv[...]  (whole-record read)
8790     GET #1, 1
8800     initializeinventoryfileifnewPFlagTrimI0% = LEN(invFlagBuf$)
8810     IF (initializeinventoryfileifnewPFlagTrimI0% > 0) = 0 THEN GOTO 8850
8820     IF (MID$(invFlagBuf$, initializeinventoryfileifnewPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 8850
8830         initializeinventoryfileifnewPFlagTrimI0% = initializeinventoryfileifnewPFlagTrimI0% - 1
8840         GOTO 8810
8850     REM END WHILE
8860     initializeinventoryfileifnewPFlag0$ = LEFT$(invFlagBuf$, initializeinventoryfileifnewPFlagTrimI0%)
8870     initializeinventoryfileifnewPDescTrimI0% = LEN(invDescBuf$)
8880     IF (initializeinventoryfileifnewPDescTrimI0% > 0) = 0 THEN GOTO 8920
8890     IF (MID$(invDescBuf$, initializeinventoryfileifnewPDescTrimI0%, 1) = " ") = 0 THEN GOTO 8920
8900         initializeinventoryfileifnewPDescTrimI0% = initializeinventoryfileifnewPDescTrimI0% - 1
8910         GOTO 8880
8920     REM END WHILE
8930     initializeinventoryfileifnewPDesc0$ = LEFT$(invDescBuf$, initializeinventoryfileifnewPDescTrimI0%)
8940     initializeinventoryfileifnewPQty0% = CVI(invQtyBuf$)
8950     initializeinventoryfileifnewPReorder0% = CVI(invReorderBuf$)
8960     initializeinventoryfileifnewPPrice0! = CVS(invPriceBuf$)
8970     IF (ASC(initializeinventoryfileifnewPFlag0$) = 0) = 0 THEN GOTO 9070
8980         FOR initializeinventoryfileifnewI0% = 1 TO partcount%
8990             ' inv[...] = { ... }  (whole-record write)
9000             LSET invFlagBuf$ = CHR$(255)
9010             LSET invDescBuf$ = ""
9020             LSET invQtyBuf$ = MKI$(0)
9030             LSET invReorderBuf$ = MKI$(0)
9040             LSET invPriceBuf$ = MKS$(0)
9050             PUT #1, initializeinventoryfileifnewI0%
9060         NEXT initializeinventoryfileifnewI0%
9070     REM END IF
9080     RETURN
9090 ' end procedure initializeinventoryfileifnew

9100 ' procedure reportinventoryerror(err%, erl%)
9110     LOCATE 25, 1
9120     errorCode0% = reportinventoryerrorErr0%
9130     GOSUB 2080
9140     PRINT (("There has been an error on line" + STR$(reportinventoryerrorErl0%)) + ": ") + errorResult0$
9150     GOSUB 3660
9160     reportinventoryerrorK0$ = readkeyResult0$
9170     RETURN
9180 ' end procedure reportinventoryerror
