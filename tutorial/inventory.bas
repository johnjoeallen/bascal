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
