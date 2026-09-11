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

190 errSYNTAX% = 2
200 errRETURNWITHOUTGOSUB% = 3
210 errOUTOFDATA% = 4
220 errILLEGALFUNCTIONCALL% = 5
230 errOVERFLOW% = 6
240 errOUTOFMEMORY% = 7
250 errSUBSCRIPTOUTOFRANGE% = 9
260 errDUPLICATEDEFINITION% = 10
270 errDIVISIONBYZERO% = 11
280 errTYPEMISMATCH% = 13
290 errOUTOFSTRINGSPACE% = 14
300 errNORESUME% = 19
310 errRESUMEWITHOUTERROR% = 20
320 errDEVICETIMEOUT% = 24
330 errDEVICEFAULT% = 25
340 errOUTOFPAPER% = 27
350 errBADFILENUMBER% = 52
360 errFILENOTFOUND% = 53
370 errBADFILEMODE% = 54
380 errFILEALREADYOPEN% = 55
390 errDEVICEIO% = 57
400 errFILEALREADYEXISTS% = 58
410 errDISKFULL% = 61
420 errINPUTPASTEND% = 62
430 errBADRECORDNUMBER% = 63
440 errBADFILENAME% = 64
450 errTOOMANYFILES% = 67
460 errDEVICEUNAVAILABLE% = 68
470 errDISKWRITEPROTECTED% = 70
480 errDISKNOTREADY% = 71
490 errDISKMEDIAERROR% = 72
500 errPATHFILEACCESS% = 75
510 errPATHNOTFOUND% = 76

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
840 ' V=30) are collapsed into a single `TAB_COL = 20`; a couple of
850 ' screens that used U=25 in the original (see showAddStockScreen
860 ' below) keep 25 as a literal rather than reusing TAB_COL.
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
1170 partCOUNT% = 100
1180 tabCOL% = 20

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
1320     FIELD #1, 1 AS invFlagBuf$, 30 AS invDescBuf$, 2 AS invQtyBuf$, 2 AS invReorderBuf$, 4 AS invPriceBuf$
1330 ON ERROR GOTO 0
1340 GOTO 1490
1350     BCC_TRY_0001_PENDING% = ERR
1360     err% = ERR
1370     erl% = ERL
1380     RESUME 1390
1390 ON ERROR GOTO 1470
1400     errorCode0% = err%
1410     GOSUB 2880
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

1740 ' BASCAL-ism: no `VIEW PRINT` (see the header note above), so this
1750 ' deliberately does NOT pin a "press any key" line to a fixed row the way
1760 ' fhb's original does -- a bare `LOCATE 25, ...` sitting under content
1770 ' that keeps printing past it (listAll()'s own items) collides with
1780 ' whatever's later written there, since nothing here scrolls a bounded
1790 ' region: waitAnyKey() is the only thing that ever touches row 25, and
1800 ' only right when it actually blocks (see listAll()'s own redraw-per-page
1810 ' structure below).

1820 ' byref scalar parameters: gatherPartDetails writes the four editable
1830 ' fields for a part directly back into the caller's variables.

1840 ' -------------------- Menu actions --------------------

1850 ' fhb's own one-time "hidden" datafile initializer PUT-ing 100 blank,
1860 ' CHR$(255)-flagged records (see the header note above) -- reproduced
1870 ' here so inven.dat no longer has to be pre-populated by hand before
1880 ' running this program. A brand-new file OPEN created just now (rather
1890 ' than one that already existed) reads back as all-zero bytes: record
1900 ' 1's flag byte is CHR$(0), never CHR$(255) -- the one signal an
1910 ' already-populated file (whose record 1 flag is always either
1920 ' CHR$(255), still an empty slot, or a real part's own "1") could never
1930 ' produce, so it's what isEmpty%() itself can't use (see its own
1940 ' header note) but this one-time check safely can.

1950 ' -------------------- Program entry --------------------

1960 CLS
1970 GOSUB 9740

1980     GOSUB 4610
1990     GOSUB 4460
2000     kp$ = readkeyResult0$
2010     IF (INSTR("1234567cCeElLaAsSrRxX", kp$) <> 0) = 0 THEN GOTO 2710
2020         ' BASCAL-ism: `select case` replaces fhb's chain of eight
2030         ' `IF VAL(KP$)=n OR KP$="x" OR KP$="X" THEN GOTO ...` lines
2040         ' (his 770-840) with one multi-way dispatch.
2050         ' 
2060         ' BASCAL-ism: `try`/`catch` (issue #60) replaces fhb's own global
2070         ' `ON ERROR GOTO` trap. A failed menu action is abandoned outright
2080         ' here -- the `catch` below runs, then execution continues right
2090         ' after `end try`, back at `loop until` -- rather than resuming at
2100         ' the exact instruction after whatever failed inside checkPart()/
2110         ' editRecord()/etc. the way fhb's `RESUME NEXT` did. See
2120         ' reportInventoryError() below and tutorial/inventory_try_catch.
2130         ' draft's own header comment for why that arbitrary resume-point
2140         ' behavior isn't something try/catch reproduces.
2150         ON ERROR GOTO 2550
2160         BCC_TRY_0004_PENDING% = 0
2170             BCCT6$ = kp$
2180             IF (BCCT6$ = "1" OR BCCT6$ = "c" OR BCCT6$ = "C") <> 0 THEN GOTO 2260
2190             IF (BCCT6$ = "2" OR BCCT6$ = "e" OR BCCT6$ = "E") <> 0 THEN GOTO 2280
2200             IF (BCCT6$ = "3" OR BCCT6$ = "l" OR BCCT6$ = "L") <> 0 THEN GOTO 2300
2210             IF (BCCT6$ = "4" OR BCCT6$ = "a" OR BCCT6$ = "A") <> 0 THEN GOTO 2320
2220             IF (BCCT6$ = "5" OR BCCT6$ = "s" OR BCCT6$ = "S") <> 0 THEN GOTO 2340
2230             IF (BCCT6$ = "6" OR BCCT6$ = "r" OR BCCT6$ = "R") <> 0 THEN GOTO 2360
2240             IF (BCCT6$ = "7" OR BCCT6$ = "x" OR BCCT6$ = "X") <> 0 THEN GOTO 2380
2250             GOTO 2520
2260                 GOSUB 6120
2270                 GOTO 2520
2280                 GOSUB 6670
2290                 GOTO 2520
2300                 GOSUB 7370
2310                 GOTO 2520
2320                 GOSUB 7800
2330                 GOTO 2520
2340                 GOSUB 8490
2350                 GOTO 2520
2360                 GOSUB 9270
2370                 GOTO 2520
2380                 ' BASCAL-ism: `inv.close()` is sugar for `CLOSE #1`,
2390                 ' matching fhb's own `90 CLOSE:SYSTEM`. fhb's original
2400                 ' also had a separate "Quit to BASIC" option (his own
2410                 ' 7, returning to the interpreter's command prompt
2420                 ' rather than exiting to DOS) -- dropped here: a
2430                 ' compiled program has no interpreter to return to,
2440                 ' so it was never anything but a second spelling of
2450                 ' this same close-and-exit action.
2460                 ' inv.close()
2470                 CLOSE #1
2480                 COLOR 7, 0
2490                 CLS
2500                 SYSTEM
2510                 GOTO 2520
2520             REM END SELECT
2530         ON ERROR GOTO 0
2540         GOTO 2680
2550             BCC_TRY_0004_PENDING% = ERR
2560             err% = ERR
2570             erl% = ERL
2580             RESUME 2590
2590         ON ERROR GOTO 2660
2600             reportinventoryerrorErr0% = err%
2610             reportinventoryerrorErl0% = erl%
2620             GOSUB 10080
2630             BCC_TRY_0004_PENDING% = 0
2640             ON ERROR GOTO 0
2650             GOTO 2680
2660             BCC_TRY_0004_PENDING% = ERR
2670             RESUME 2680
2680         ON ERROR GOTO 0
2690             IF BCC_TRY_0004_PENDING% <> 0 THEN ERROR BCC_TRY_0004_PENDING%
2700         REM END TRY
2710     REM END IF
2720     GOTO 1980
2730 REM END DO

2740 ' -------------------- Error handling --------------------
2750 ' err%/erl% are ordinary locals scoped to the `catch` block above, not
2760 ' aliases for the ambient (readable-anywhere) `err`/`erl` pseudo-
2770 ' variables `on error goto` uses -- see `Statement::TryCatch`'s own doc
2780 ' comment in ast.rs. Passed straight through to ERROR$ here like fhb's
2790 ' own ERR/ERL (his 3390: "an error on line";ERL), decoded through
2800 ' BASCAL's own com.bascal.stdlib.error (ERROR$) instead of fhb's
2810 ' hand-rolled lookup table -- see the header note above. try/catch
2820 ' itself isn't documented in the manual yet (GitHub issue #60 tracks
2830 ' the still-unfinished C-target work; the manual page can follow once
2840 ' that lands) -- see ast.rs's own `Statement::TryCatch` doc comment for
2850 ' the full semantics meanwhile.
2860 END

2870 ' function error$(code%)
2880     BCCT8% = errorCode0%
2890     IF (BCCT8% = errSYNTAX%) <> 0 THEN GOTO 3230
2900     IF (BCCT8% = errRETURNWITHOUTGOSUB%) <> 0 THEN GOTO 3260
2910     IF (BCCT8% = errOUTOFDATA%) <> 0 THEN GOTO 3290
2920     IF (BCCT8% = errILLEGALFUNCTIONCALL%) <> 0 THEN GOTO 3320
2930     IF (BCCT8% = errOVERFLOW%) <> 0 THEN GOTO 3350
2940     IF (BCCT8% = errOUTOFMEMORY%) <> 0 THEN GOTO 3380
2950     IF (BCCT8% = errSUBSCRIPTOUTOFRANGE%) <> 0 THEN GOTO 3410
2960     IF (BCCT8% = errDUPLICATEDEFINITION%) <> 0 THEN GOTO 3440
2970     IF (BCCT8% = errDIVISIONBYZERO%) <> 0 THEN GOTO 3470
2980     IF (BCCT8% = errTYPEMISMATCH%) <> 0 THEN GOTO 3500
2990     IF (BCCT8% = errOUTOFSTRINGSPACE%) <> 0 THEN GOTO 3530
3000     IF (BCCT8% = errNORESUME%) <> 0 THEN GOTO 3560
3010     IF (BCCT8% = errRESUMEWITHOUTERROR%) <> 0 THEN GOTO 3590
3020     IF (BCCT8% = errDEVICETIMEOUT%) <> 0 THEN GOTO 3620
3030     IF (BCCT8% = errDEVICEFAULT%) <> 0 THEN GOTO 3650
3040     IF (BCCT8% = errOUTOFPAPER%) <> 0 THEN GOTO 3680
3050     IF (BCCT8% = errBADFILENUMBER%) <> 0 THEN GOTO 3710
3060     IF (BCCT8% = errFILENOTFOUND%) <> 0 THEN GOTO 3740
3070     IF (BCCT8% = errBADFILEMODE%) <> 0 THEN GOTO 3770
3080     IF (BCCT8% = errFILEALREADYOPEN%) <> 0 THEN GOTO 3800
3090     IF (BCCT8% = errDEVICEIO%) <> 0 THEN GOTO 3830
3100     IF (BCCT8% = errFILEALREADYEXISTS%) <> 0 THEN GOTO 3860
3110     IF (BCCT8% = errDISKFULL%) <> 0 THEN GOTO 3890
3120     IF (BCCT8% = errINPUTPASTEND%) <> 0 THEN GOTO 3920
3130     IF (BCCT8% = errBADRECORDNUMBER%) <> 0 THEN GOTO 3950
3140     IF (BCCT8% = errBADFILENAME%) <> 0 THEN GOTO 3980
3150     IF (BCCT8% = errTOOMANYFILES%) <> 0 THEN GOTO 4010
3160     IF (BCCT8% = errDEVICEUNAVAILABLE%) <> 0 THEN GOTO 4040
3170     IF (BCCT8% = errDISKWRITEPROTECTED%) <> 0 THEN GOTO 4070
3180     IF (BCCT8% = errDISKNOTREADY%) <> 0 THEN GOTO 4100
3190     IF (BCCT8% = errDISKMEDIAERROR%) <> 0 THEN GOTO 4130
3200     IF (BCCT8% = errPATHFILEACCESS%) <> 0 THEN GOTO 4160
3210     IF (BCCT8% = errPATHNOTFOUND%) <> 0 THEN GOTO 4190
3220     GOTO 4220
3230         errorResult0$ = "Syntax error"
3240         RETURN
3250         GOTO 4240
3260         errorResult0$ = "RETURN without GOSUB"
3270         RETURN
3280         GOTO 4240
3290         errorResult0$ = "Out of DATA"
3300         RETURN
3310         GOTO 4240
3320         errorResult0$ = "Illegal function call"
3330         RETURN
3340         GOTO 4240
3350         errorResult0$ = "Overflow"
3360         RETURN
3370         GOTO 4240
3380         errorResult0$ = "Out of memory"
3390         RETURN
3400         GOTO 4240
3410         errorResult0$ = "Subscript out of range"
3420         RETURN
3430         GOTO 4240
3440         errorResult0$ = "Duplicate Definition"
3450         RETURN
3460         GOTO 4240
3470         errorResult0$ = "Division by zero"
3480         RETURN
3490         GOTO 4240
3500         errorResult0$ = "Type mismatch"
3510         RETURN
3520         GOTO 4240
3530         errorResult0$ = "Out of string space"
3540         RETURN
3550         GOTO 4240
3560         errorResult0$ = "No RESUME"
3570         RETURN
3580         GOTO 4240
3590         errorResult0$ = "RESUME without error"
3600         RETURN
3610         GOTO 4240
3620         errorResult0$ = "Device timeout"
3630         RETURN
3640         GOTO 4240
3650         errorResult0$ = "Device fault"
3660         RETURN
3670         GOTO 4240
3680         errorResult0$ = "Out of paper"
3690         RETURN
3700         GOTO 4240
3710         errorResult0$ = "Bad file number"
3720         RETURN
3730         GOTO 4240
3740         errorResult0$ = "File not found"
3750         RETURN
3760         GOTO 4240
3770         errorResult0$ = "Bad file mode"
3780         RETURN
3790         GOTO 4240
3800         errorResult0$ = "File already open"
3810         RETURN
3820         GOTO 4240
3830         errorResult0$ = "Device I/O error"
3840         RETURN
3850         GOTO 4240
3860         errorResult0$ = "File already exists"
3870         RETURN
3880         GOTO 4240
3890         errorResult0$ = "Disk full"
3900         RETURN
3910         GOTO 4240
3920         errorResult0$ = "Input past end"
3930         RETURN
3940         GOTO 4240
3950         errorResult0$ = "Bad record number"
3960         RETURN
3970         GOTO 4240
3980         errorResult0$ = "Bad file name"
3990         RETURN
4000         GOTO 4240
4010         errorResult0$ = "Too many files"
4020         RETURN
4030         GOTO 4240
4040         errorResult0$ = "Device unavailable"
4050         RETURN
4060         GOTO 4240
4070         errorResult0$ = "Disk write protected"
4080         RETURN
4090         GOTO 4240
4100         errorResult0$ = "Disk not ready"
4110         RETURN
4120         GOTO 4240
4130         errorResult0$ = "Disk media error"
4140         RETURN
4150         GOTO 4240
4160         errorResult0$ = "Path/File access error"
4170         RETURN
4180         GOTO 4240
4190         errorResult0$ = "Path not found"
4200         RETURN
4210         GOTO 4240
4220         errorResult0$ = "Error " + STR$(errorCode0%)
4230         RETURN
4240     REM END SELECT
4250     RETURN
4260 ' end function error$

4270 ' function isempty%(flag$)
4280     isemptyResult0% = ASC(isemptyFlag0$) = 255
4290     RETURN
4300 ' end function isempty%

4310 ' function partinrange%(n%)
4320     IF (partinrangeN0% >= 1) = 0 THEN GOTO 4360
4330     IF (partinrangeN0% <= partCOUNT%) = 0 THEN GOTO 4360
4340         partinrangeResult0% = 1
4350         RETURN
4360     REM END IF
4370     partinrangeResult0% = 0
4380     RETURN
4390 ' end function partinrange%

4400 ' function readpartnumberinput$()
4410     INPUT "Input part number"; readpartnumberinputS0$
4420     readpartnumberinputResult0$ = readpartnumberinputS0$
4430     RETURN
4440 ' end function readpartnumberinput$

4450 ' function readkey$()
4460         readkeyK0$ = INKEY$
4470         IF (readkeyK0$ <> "") = 0 THEN GOTO 4460
4480     REM END DO
4490     readkeyResult0$ = readkeyK0$
4500     RETURN
4510 ' end function readkey$

4520 ' procedure waitanykey()
4530     LOCATE 25, 10
4540     PRINT "Press the AnyKey to continue...";
4550         waitanykeyK0$ = INKEY$
4560         IF (waitanykeyK0$ <> "") = 0 THEN GOTO 4550
4570     REM END DO
4580     RETURN
4590 ' end procedure waitanykey

4600 ' procedure showmainmenu()
4610     CLS
4620     COLOR 14, 4
4630     CLS
4640     LOCATE 6, 1
4650     PRINT
4660     ' `tab(n)` passes straight through to real TAB(n), same as
4670     ' fhb's own `PRINT TAB(V) "..."` -- but only as a bare item in
4680     ' a PRINT list, juxtaposed or `;`-separated like here. Real
4690     ' BASCOM rejects `"literal" + tab(n) + ...` (TAB isn't a real
4700     ' string function you can concatenate); see printListHeader()
4710     ' and printReorderHeader() below, which need `;` between a
4720     ' preceding string and a `tab(n)` for exactly this reason.
4730     PRINT TAB(30)"Inventory Program"
4740     PRINT
4750     PRINT TAB(tabCOL%)"1......C)heck a part"
4760     PRINT TAB(tabCOL%)"2......E)dit/overwrite/add a part"
4770     PRINT TAB(tabCOL%)("3......L)ist all" + STR$(partCOUNT%)) + "parts"
4780     PRINT TAB(tabCOL%)"4......A)dd stock"
4790     PRINT TAB(tabCOL%)"5......S)ubtract stock"
4800     PRINT TAB(tabCOL%)"6......R)eorder Report"
4810     PRINT
4820     PRINT TAB(tabCOL%)"7......eX)it to system"
4830     RETURN
4840 ' end procedure showmainmenu

4850 ' procedure showbadpartnumber()
4860     CLS
4870     LOCATE 10, 10
4880     PRINT "Part number is out of permissable range of 1 to" + STR$(partCOUNT%)
4890     RETURN
4900 ' end procedure showbadpartnumber

4910 ' procedure showrangeretrymessage()
4920     LOCATE 10, 15
4930     PRINT "The Part number is out of permissable range of 1 to" + STR$(partCOUNT%)
4940     LOCATE 25, 15
4950     PRINT "Press the Anykey to reenter part number...";
4960     RETURN
4970 ' end procedure showrangeretrymessage

4980 ' procedure shownullentrymessage(partstr$)
4990     LOCATE 10, tabCOL%
5000     PRINT ("Part number " + shownullentrymessagePartStr0$) + " is a null entry"
5010     RETURN
5020 ' end procedure shownullentrymessage

5030 ' procedure showpartstatus(partnum%, desc$, qty%, reorder%, price!)
5040     CLS
5050     LOCATE 5, 1
5060     PRINT TAB(tabCOL%)"Inventory Status for Individual Part Number"
5070     PRINT TAB(tabCOL%)"==========================================="
5080     PRINT
5090     PRINT
5100     PRINT TAB(tabCOL%)"     Part number:  " + STR$(showpartstatusPartNum0%)
5110     PRINT
5120     PRINT TAB(tabCOL%)"       Item name:  " + showpartstatusDesc0$
5130     PRINT TAB(tabCOL%)"Quantity on hand:  " + STR$(showpartstatusQty0%)
5140     PRINT TAB(tabCOL%)"   Reorder level:  " + STR$(showpartstatusReorder0%)
5150     PRINT TAB(tabCOL%)"      Unit price:  " + STR$(showpartstatusPrice0!)
5160     RETURN
5170 ' end procedure showpartstatus

5180 ' procedure printlistheader()
5190     CLS
5200     PRINT TAB(25)"I N V E N T O R Y   L I S T I N G"; TAB(65); STR$(partCOUNT%) + "items"
5210     PRINT "                                          Quantity       Reorder"
5220     PRINT " Partno           Description             on hand         level"
5230     RETURN
5240 ' end procedure printlistheader

5250 ' procedure printinventoryline(partnum%, desc$, qty%, reorder%)
5260     PRINT (((((STR$(printinventorylinePartNum0%) + "  ") + printinventorylineDesc0$) + "   ") + STR$(printinventorylineQty0%)) + "          ") + STR$(printinventorylineReorder0%)
5270     RETURN
5280 ' end procedure printinventoryline

5290 ' procedure printreorderheader()
5300     CLS
5310     LOCATE 1, tabCOL%
5320     PRINT "Reorder Report"; TAB(55); DATE$
5330     PRINT
5340     PRINT "                                             Quantity       Reorder"
5350     PRINT "    Partno           Description             on hand         level"
5360     PRINT "   =======  ==============================   ========       ======="
5370     RETURN
5380 ' end procedure printreorderheader

5390 ' procedure printreorderline(partnum%, desc$, qty%, reorder%)
5400     PRINT (((((("  " + STR$(printreorderlinePartNum0%)) + "  ") + printreorderlineDesc0$) + "   ") + STR$(printreorderlineQty0%)) + "          ") + STR$(printreorderlineReorder0%)
5410     RETURN
5420 ' end procedure printreorderline

5430 ' procedure gatherpartdetails(partnum%, desc$, qty%, reorder%, price!)
5440     CLS
5450     LOCATE 4, tabCOL%
5460     PRINT "Adding or Overwriting a Record"
5470     LOCATE 8, tabCOL%
5480     PRINT "Record/Partno" + STR$(gatherpartdetailsPartNum0%)
5490     LOCATE 11, 39
5500     PRINT "------------------------------"
5510     LOCATE 10, tabCOL%
5520     INPUT "      Description"; gatherpartdetailsDesc0$
5530     LOCATE 12, tabCOL%
5540     INPUT "Quantity in stock"; gatherpartdetailsQty0%
5550     LOCATE 14, tabCOL%
5560     INPUT "    Reorder level"; gatherpartdetailsReorder0%
5570     LOCATE 16, tabCOL%
5580     INPUT "       Unit price"; gatherpartdetailsPrice0!
5590     LOCATE 18, tabCOL%
5600     PRINT "Is information correct (Y/N)?"
5610     RETURN
5620 ' end procedure gatherpartdetails

5630 ' procedure showaddstockscreen(partnum%, desc$, qty%, reorder%)
5640     CLS
5650     LOCATE 4, 25
5660     PRINT "Add to an inventory part number"
5670     LOCATE 5, 25
5680     PRINT "==============================="
5690     LOCATE 8, tabCOL%
5700     PRINT "     Part number: " + STR$(showaddstockscreenPartNum0%)
5710     LOCATE 9, tabCOL%
5720     PRINT "Item description: " + showaddstockscreenDesc0$
5730     LOCATE 10, tabCOL%
5740     PRINT "Quantity on hand: " + STR$(showaddstockscreenQty0%)
5750     LOCATE 11, tabCOL%
5760     PRINT "   Reorder Level: " + STR$(showaddstockscreenReorder0%)
5770     RETURN
5780 ' end procedure showaddstockscreen

5790 ' procedure shownegativeqtywarning()
5800     LOCATE 17, 15
5810     PRINT "The quantity to add must NOT be a negative number"
5820     LOCATE 25, 1
5830     PRINT "Please press the Anykey to reenter quantity to add...";
5840     RETURN
5850 ' end procedure shownegativeqtywarning

5860 ' procedure showsubtractstockscreen(partnum%, desc$, qty%, reorder%)
5870     CLS
5880     LOCATE 4, tabCOL%
5890     PRINT "Subtract an inventory part number"
5900     LOCATE 5, tabCOL%
5910     PRINT "================================="
5920     LOCATE 8, tabCOL%
5930     PRINT "         Part number: " + STR$(showsubtractstockscreenPartNum0%)
5940     LOCATE 9, tabCOL%
5950     PRINT "    Item description: " + showsubtractstockscreenDesc0$
5960     LOCATE 10, tabCOL%
5970     PRINT "    Quantity on hand: " + STR$(showsubtractstockscreenQty0%)
5980     LOCATE 11, tabCOL%
5990     PRINT "       Reorder Level: " + STR$(showsubtractstockscreenReorder0%)
6000     RETURN
6010 ' end procedure showsubtractstockscreen

6020 ' procedure showoversubtractwarning(onhand%)
6030     LOCATE 17, 5
6040     PRINT "The quantity to SUBTRACT must NOT result in NEGATIVE inventory"
6050     LOCATE 18, 5
6060     PRINT ("Only" + STR$(showoversubtractwarningOnHand0%)) + " IN STOCK"
6070     LOCATE 25, 1
6080     PRINT "Please press the Anykey to reenter quantity to subtract...";
6090     RETURN
6100 ' end procedure showoversubtractwarning

6110 ' procedure checkpart()
6120     ' global inv
6130     GOSUB 4410
6140     checkpartPartStr0$ = readpartnumberinputResult0$
6150     checkpartPart0% = VAL(checkpartPartStr0$)
6160     partinrangeN0% = checkpartPart0%
6170     GOSUB 4320
6180     IF (partinrangeResult0% = 0) = 0 THEN GOTO 6220
6190         GOSUB 4860
6200         GOSUB 4530
6210         RETURN
6220     REM END IF
6230     ' BASCAL-ism: `let p = inv[part%]` reads record `part%` of the
6240     ' `inv` file into a local record variable `p` -- one expression
6250     ' for what fhb's `GET #1, PART!` plus five separate field reads
6260     ' (F$, D$, CVI(Q$), CVI(R$), CVS(P$)) did by hand. The write
6270     ' side, `inv[part%] = { ... }` (see editRecord() below), is the
6280     ' same sugar for PUT plus the LSET/MKx$ packing it replaces.
6290     ' let p = inv[...]  (whole-record read)
6300     GET #1, checkpartPart0%
6310     checkpartPFlagTrimI0% = LEN(invFlagBuf$)
6320     IF (checkpartPFlagTrimI0% > 0) = 0 THEN GOTO 6360
6330     IF (MID$(invFlagBuf$, checkpartPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 6360
6340         checkpartPFlagTrimI0% = checkpartPFlagTrimI0% - 1
6350         GOTO 6320
6360     REM END WHILE
6370     checkpartPFlag0$ = LEFT$(invFlagBuf$, checkpartPFlagTrimI0%)
6380     checkpartPDescTrimI0% = LEN(invDescBuf$)
6390     IF (checkpartPDescTrimI0% > 0) = 0 THEN GOTO 6430
6400     IF (MID$(invDescBuf$, checkpartPDescTrimI0%, 1) = " ") = 0 THEN GOTO 6430
6410         checkpartPDescTrimI0% = checkpartPDescTrimI0% - 1
6420         GOTO 6390
6430     REM END WHILE
6440     checkpartPDesc0$ = LEFT$(invDescBuf$, checkpartPDescTrimI0%)
6450     checkpartPQty0% = CVI(invQtyBuf$)
6460     checkpartPReorder0% = CVI(invReorderBuf$)
6470     checkpartPPrice0! = CVS(invPriceBuf$)
6480     isemptyFlag0$ = checkpartPFlag0$
6490     GOSUB 4280
6500     IF (isemptyResult0%) = 0 THEN GOTO 6560
6510         CLS
6520         LOCATE 10, 18
6530         PRINT ("Part number" + STR$(checkpartPart0%)) + "is still a null entry at this time"
6540         GOSUB 4530
6550         RETURN
6560     REM END IF
6570     showpartstatusPartNum0% = checkpartPart0%
6580     showpartstatusDesc0$ = checkpartPDesc0$
6590     showpartstatusQty0% = checkpartPQty0%
6600     showpartstatusReorder0% = checkpartPReorder0%
6610     showpartstatusPrice0! = checkpartPPrice0!
6620     GOSUB 5040
6630     GOSUB 4530
6640     RETURN
6650 ' end procedure checkpart

6660 ' procedure editrecord()
6670     ' global inv
6680     CLS
6690     LOCATE 10, tabCOL%
6700     GOSUB 4410
6710     editrecordPartStr0$ = readpartnumberinputResult0$
6720     editrecordPart0% = VAL(editrecordPartStr0$)
6730     partinrangeN0% = editrecordPart0%
6740     GOSUB 4320
6750     IF (partinrangeResult0% = 0) = 0 THEN GOTO 6790
6760         GOSUB 4860
6770         GOSUB 4530
6780         RETURN
6790     REM END IF
6800     ' let p = inv[...]  (whole-record read)
6810     GET #1, editrecordPart0%
6820     editrecordPFlagTrimI0% = LEN(invFlagBuf$)
6830     IF (editrecordPFlagTrimI0% > 0) = 0 THEN GOTO 6870
6840     IF (MID$(invFlagBuf$, editrecordPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 6870
6850         editrecordPFlagTrimI0% = editrecordPFlagTrimI0% - 1
6860         GOTO 6830
6870     REM END WHILE
6880     editrecordPFlag0$ = LEFT$(invFlagBuf$, editrecordPFlagTrimI0%)
6890     editrecordPDescTrimI0% = LEN(invDescBuf$)
6900     IF (editrecordPDescTrimI0% > 0) = 0 THEN GOTO 6940
6910     IF (MID$(invDescBuf$, editrecordPDescTrimI0%, 1) = " ") = 0 THEN GOTO 6940
6920         editrecordPDescTrimI0% = editrecordPDescTrimI0% - 1
6930         GOTO 6900
6940     REM END WHILE
6950     editrecordPDesc0$ = LEFT$(invDescBuf$, editrecordPDescTrimI0%)
6960     editrecordPQty0% = CVI(invQtyBuf$)
6970     editrecordPReorder0% = CVI(invReorderBuf$)
6980     editrecordPPrice0! = CVS(invPriceBuf$)
6990     isemptyFlag0$ = editrecordPFlag0$
7000     GOSUB 4280
7010     IF (isemptyResult0% = 0) = 0 THEN GOTO 7100
7020         LOCATE 12, tabCOL%
7030         PRINT "Overwrite existing part data?"
7040         GOSUB 4460
7050         editrecordKp0$ = readkeyResult0$
7060         IF (editrecordKp0$ <> "Y") = 0 THEN GOTO 7090
7070         IF (editrecordKp0$ <> "y") = 0 THEN GOTO 7090
7080             RETURN
7090         REM END IF
7100     REM END IF

7110         gatherpartdetailsPartNum0% = editrecordPart0%
7120         gatherpartdetailsDesc0$ = editrecordEditDesc0$
7130         gatherpartdetailsQty0% = editrecordEditQty0%
7140         gatherpartdetailsReorder0% = editrecordEditReorder0%
7150         gatherpartdetailsPrice0! = editrecordEditPrice0!
7160         GOSUB 5440
7170         editrecordEditDesc0$ = gatherpartdetailsDesc0$
7180         editrecordEditQty0% = gatherpartdetailsQty0%
7190         editrecordEditReorder0% = gatherpartdetailsReorder0%
7200         editrecordEditPrice0! = gatherpartdetailsPrice0!
7210         GOSUB 4460
7220         editrecordKp0$ = readkeyResult0$
7230         IF (editrecordKp0$ = "Y") <> 0 THEN GOTO 7260
7240         IF (editrecordKp0$ = "y") <> 0 THEN GOTO 7260
7250         GOTO 7110
7260     REM END DO
7270     ' inv[...] = { ... }  (whole-record write)
7280     LSET invFlagBuf$ = "1"
7290     LSET invDescBuf$ = editrecordEditDesc0$
7300     LSET invQtyBuf$ = MKI$(editrecordEditQty0%)
7310     LSET invReorderBuf$ = MKI$(editrecordEditReorder0%)
7320     LSET invPriceBuf$ = MKS$(editrecordEditPrice0!)
7330     PUT #1, editrecordPart0%
7340     RETURN
7350 ' end procedure editrecord

7360 ' procedure listall()
7370     ' global inv
7380     GOSUB 5190
7390     listallScrollCount0% = 0
7400     FOR listallI0% = 1 TO partCOUNT%
7410         ' let p = inv[...]  (whole-record read)
7420         GET #1, listallI0%
7430         listallPFlagTrimI0% = LEN(invFlagBuf$)
7440         IF (listallPFlagTrimI0% > 0) = 0 THEN GOTO 7480
7450         IF (MID$(invFlagBuf$, listallPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 7480
7460             listallPFlagTrimI0% = listallPFlagTrimI0% - 1
7470             GOTO 7440
7480         REM END WHILE
7490         listallPFlag0$ = LEFT$(invFlagBuf$, listallPFlagTrimI0%)
7500         listallPDescTrimI0% = LEN(invDescBuf$)
7510         IF (listallPDescTrimI0% > 0) = 0 THEN GOTO 7550
7520         IF (MID$(invDescBuf$, listallPDescTrimI0%, 1) = " ") = 0 THEN GOTO 7550
7530             listallPDescTrimI0% = listallPDescTrimI0% - 1
7540             GOTO 7510
7550         REM END WHILE
7560         listallPDesc0$ = LEFT$(invDescBuf$, listallPDescTrimI0%)
7570         listallPQty0% = CVI(invQtyBuf$)
7580         listallPReorder0% = CVI(invReorderBuf$)
7590         listallPPrice0! = CVS(invPriceBuf$)
7600         printinventorylinePartNum0% = listallI0%
7610         printinventorylineDesc0$ = listallPDesc0$
7620         printinventorylineQty0% = listallPQty0%
7630         printinventorylineReorder0% = listallPReorder0%
7640         GOSUB 5260
7650         listallScrollCount0% = listallScrollCount0% + 1
7660         IF (listallScrollCount0% = 20) = 0 THEN GOTO 7750
7670             GOSUB 4530
7680             listallScrollCount0% = 0
7690             ' Redraw for the next page rather than let it keep scrolling
7700             ' past row 25 -- see printListHeader()'s own note on why a
7710             ' fixed-row prompt can't coexist with unbounded scrolling.
7720             IF (listallI0% < partCOUNT%) = 0 THEN GOTO 7740
7730                 GOSUB 5190
7740             REM END IF
7750         REM END IF
7760     NEXT listallI0%
7770     RETURN
7780 ' end procedure listall

7790 ' procedure addstock()
7800     ' global inv
7810     CLS
7820     LOCATE 5, 25
7830     PRINT "A D D I N G   S T O C K"

7840         LOCATE 8, 25
7850         GOSUB 4410
7860         addstockPartStr0$ = readpartnumberinputResult0$
7870         addstockPart0% = VAL(addstockPartStr0$)
7880         partinrangeN0% = addstockPart0%
7890         GOSUB 4320
7900         addstockValidPart0% = partinrangeResult0%
7910         IF (addstockValidPart0% = 0) = 0 THEN GOTO 7940
7920             GOSUB 4920
7930             GOSUB 4460
7940         REM END IF
7950         IF (addstockValidPart0% <> 0) = 0 THEN GOTO 7840
7960     REM END DO

7970     ' let p = inv[...]  (whole-record read)
7980     GET #1, addstockPart0%
7990     addstockPFlagTrimI0% = LEN(invFlagBuf$)
8000     IF (addstockPFlagTrimI0% > 0) = 0 THEN GOTO 8040
8010     IF (MID$(invFlagBuf$, addstockPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 8040
8020         addstockPFlagTrimI0% = addstockPFlagTrimI0% - 1
8030         GOTO 8000
8040     REM END WHILE
8050     addstockPFlag0$ = LEFT$(invFlagBuf$, addstockPFlagTrimI0%)
8060     addstockPDescTrimI0% = LEN(invDescBuf$)
8070     IF (addstockPDescTrimI0% > 0) = 0 THEN GOTO 8110
8080     IF (MID$(invDescBuf$, addstockPDescTrimI0%, 1) = " ") = 0 THEN GOTO 8110
8090         addstockPDescTrimI0% = addstockPDescTrimI0% - 1
8100         GOTO 8070
8110     REM END WHILE
8120     addstockPDesc0$ = LEFT$(invDescBuf$, addstockPDescTrimI0%)
8130     addstockPQty0% = CVI(invQtyBuf$)
8140     addstockPReorder0% = CVI(invReorderBuf$)
8150     addstockPPrice0! = CVS(invPriceBuf$)
8160     isemptyFlag0$ = addstockPFlag0$
8170     GOSUB 4280
8180     IF (isemptyResult0%) = 0 THEN GOTO 8230
8190         shownullentrymessagePartStr0$ = addstockPartStr0$
8200         GOSUB 4990
8210         GOSUB 4460
8220         RETURN
8230     REM END IF

8240         showaddstockscreenPartNum0% = addstockPart0%
8250         showaddstockscreenDesc0$ = addstockPDesc0$
8260         showaddstockscreenQty0% = addstockPQty0%
8270         showaddstockscreenReorder0% = addstockPReorder0%
8280         GOSUB 5640
8290         LOCATE 14, tabCOL%
8300         INPUT " Quantity to add"; addstockAddStr0$
8310         addstockAddAmt0% = VAL(addstockAddStr0$)
8320         IF (addstockAddAmt0% < 0) = 0 THEN GOTO 8350
8330             GOSUB 5800
8340             GOSUB 4460
8350         REM END IF
8360         IF (addstockAddAmt0% >= 0) = 0 THEN GOTO 8240
8370     REM END DO

8380     addstockPQty0% = addstockPQty0% + addstockAddAmt0%
8390     ' inv[...] = p  (write back a let-bound record)
8400     LSET invFlagBuf$ = addstockPFlag0$
8410     LSET invDescBuf$ = addstockPDesc0$
8420     LSET invQtyBuf$ = MKI$(addstockPQty0%)
8430     LSET invReorderBuf$ = MKI$(addstockPReorder0%)
8440     LSET invPriceBuf$ = MKS$(addstockPPrice0!)
8450     PUT #1, addstockPart0%
8460     RETURN
8470 ' end procedure addstock

8480 ' procedure subtractstock()
8490     ' global inv
8500     CLS
8510     LOCATE 5, 20
8520     PRINT "S U B T R A C T I N G    S T O C K"

8530         LOCATE 8, 25
8540         GOSUB 4410
8550         subtractstockPartStr0$ = readpartnumberinputResult0$
8560         subtractstockPart0% = VAL(subtractstockPartStr0$)
8570         partinrangeN0% = subtractstockPart0%
8580         GOSUB 4320
8590         subtractstockValidPart0% = partinrangeResult0%
8600         IF (subtractstockValidPart0% = 0) = 0 THEN GOTO 8630
8610             GOSUB 4920
8620             GOSUB 4460
8630         REM END IF
8640         IF (subtractstockValidPart0% <> 0) = 0 THEN GOTO 8530
8650     REM END DO

8660     ' let p = inv[...]  (whole-record read)
8670     GET #1, subtractstockPart0%
8680     subtractstockPFlagTrimI0% = LEN(invFlagBuf$)
8690     IF (subtractstockPFlagTrimI0% > 0) = 0 THEN GOTO 8730
8700     IF (MID$(invFlagBuf$, subtractstockPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 8730
8710         subtractstockPFlagTrimI0% = subtractstockPFlagTrimI0% - 1
8720         GOTO 8690
8730     REM END WHILE
8740     subtractstockPFlag0$ = LEFT$(invFlagBuf$, subtractstockPFlagTrimI0%)
8750     subtractstockPDescTrimI0% = LEN(invDescBuf$)
8760     IF (subtractstockPDescTrimI0% > 0) = 0 THEN GOTO 8800
8770     IF (MID$(invDescBuf$, subtractstockPDescTrimI0%, 1) = " ") = 0 THEN GOTO 8800
8780         subtractstockPDescTrimI0% = subtractstockPDescTrimI0% - 1
8790         GOTO 8760
8800     REM END WHILE
8810     subtractstockPDesc0$ = LEFT$(invDescBuf$, subtractstockPDescTrimI0%)
8820     subtractstockPQty0% = CVI(invQtyBuf$)
8830     subtractstockPReorder0% = CVI(invReorderBuf$)
8840     subtractstockPPrice0! = CVS(invPriceBuf$)
8850     isemptyFlag0$ = subtractstockPFlag0$
8860     GOSUB 4280
8870     IF (isemptyResult0%) = 0 THEN GOTO 8920
8880         shownullentrymessagePartStr0$ = subtractstockPartStr0$
8890         GOSUB 4990
8900         GOSUB 4460
8910         RETURN
8920     REM END IF

8930         showsubtractstockscreenPartNum0% = subtractstockPart0%
8940         showsubtractstockscreenDesc0$ = subtractstockPDesc0$
8950         showsubtractstockscreenQty0% = subtractstockPQty0%
8960         showsubtractstockscreenReorder0% = subtractstockPReorder0%
8970         GOSUB 5870
8980         LOCATE 14, tabCOL%
8990         INPUT "Quantity to subtract"; subtractstockSubStr0$
9000         subtractstockSubAmt0% = VAL(subtractstockSubStr0$)
9010         subtractstockOverSubtract0% = 0
9020         IF (subtractstockSubAmt0% >= 0) = 0 THEN GOTO 9080
9030         IF ((subtractstockPQty0% - subtractstockSubAmt0%) < 0) = 0 THEN GOTO 9080
9040             subtractstockOverSubtract0% = 1
9050             showoversubtractwarningOnHand0% = subtractstockPQty0%
9060             GOSUB 6030
9070             GOSUB 4460
9080         REM END IF
9090         IF (subtractstockSubAmt0% >= 0) = 0 THEN GOTO 8930
9100         IF (subtractstockOverSubtract0% = 0) = 0 THEN GOTO 8930
9110     REM END DO

9120     subtractstockPQty0% = subtractstockPQty0% - subtractstockSubAmt0%
9130     IF (subtractstockPQty0% <= subtractstockPReorder0%) = 0 THEN GOTO 9150
9140         LOCATE 16, tabCOL%
9150     REM END IF
9160     PRINT (("quantity now" + STR$(subtractstockPQty0%)) + " reorder level") + STR$(subtractstockPReorder0%)
9170     ' inv[...] = p  (write back a let-bound record)
9180     LSET invFlagBuf$ = subtractstockPFlag0$
9190     LSET invDescBuf$ = subtractstockPDesc0$
9200     LSET invQtyBuf$ = MKI$(subtractstockPQty0%)
9210     LSET invReorderBuf$ = MKI$(subtractstockPReorder0%)
9220     LSET invPriceBuf$ = MKS$(subtractstockPPrice0!)
9230     PUT #1, subtractstockPart0%
9240     RETURN
9250 ' end procedure subtractstock

9260 ' procedure reorderreport()
9270     ' global inv
9280     GOSUB 5300
9290     reorderreportReportLineCount0% = 0
9300     FOR reorderreportI0% = 1 TO partCOUNT%
9310         ' let p = inv[...]  (whole-record read)
9320         GET #1, reorderreportI0%
9330         reorderreportPFlagTrimI0% = LEN(invFlagBuf$)
9340         IF (reorderreportPFlagTrimI0% > 0) = 0 THEN GOTO 9380
9350         IF (MID$(invFlagBuf$, reorderreportPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 9380
9360             reorderreportPFlagTrimI0% = reorderreportPFlagTrimI0% - 1
9370             GOTO 9340
9380         REM END WHILE
9390         reorderreportPFlag0$ = LEFT$(invFlagBuf$, reorderreportPFlagTrimI0%)
9400         reorderreportPDescTrimI0% = LEN(invDescBuf$)
9410         IF (reorderreportPDescTrimI0% > 0) = 0 THEN GOTO 9450
9420         IF (MID$(invDescBuf$, reorderreportPDescTrimI0%, 1) = " ") = 0 THEN GOTO 9450
9430             reorderreportPDescTrimI0% = reorderreportPDescTrimI0% - 1
9440             GOTO 9410
9450         REM END WHILE
9460         reorderreportPDesc0$ = LEFT$(invDescBuf$, reorderreportPDescTrimI0%)
9470         reorderreportPQty0% = CVI(invQtyBuf$)
9480         reorderreportPReorder0% = CVI(invReorderBuf$)
9490         reorderreportPPrice0! = CVS(invPriceBuf$)
9500         IF (reorderreportPQty0% < reorderreportPReorder0%) = 0 THEN GOTO 9680
9510             printreorderlinePartNum0% = reorderreportI0%
9520             printreorderlineDesc0$ = reorderreportPDesc0$
9530             printreorderlineQty0% = reorderreportPQty0%
9540             printreorderlineReorder0% = reorderreportPReorder0%
9550             GOSUB 5400
9560             reorderreportReportLineCount0% = reorderreportReportLineCount0% + 1
9570             IF (reorderreportReportLineCount0% > 15) = 0 THEN GOTO 9670
9580                 GOSUB 4530
9590                 reorderreportReportLineCount0% = 0
9600                 ' Redraw for the next page rather than let it keep
9610                 ' scrolling past row 25 -- see printListHeader()'s own
9620                 ' note (same underlying issue, same fix) on why a
9630                 ' fixed-row prompt can't coexist with unbounded scrolling.
9640                 IF (reorderreportI0% < partCOUNT%) = 0 THEN GOTO 9660
9650                     GOSUB 5300
9660                 REM END IF
9670             REM END IF
9680         REM END IF
9690     NEXT reorderreportI0%
9700     GOSUB 4530
9710     RETURN
9720 ' end procedure reorderreport

9730 ' procedure initializeinventoryfileifnew()
9740     ' global inv
9750     ' let p = inv[...]  (whole-record read)
9760     GET #1, 1
9770     initializeinventoryfileifnewPFlagTrimI0% = LEN(invFlagBuf$)
9780     IF (initializeinventoryfileifnewPFlagTrimI0% > 0) = 0 THEN GOTO 9820
9790     IF (MID$(invFlagBuf$, initializeinventoryfileifnewPFlagTrimI0%, 1) = " ") = 0 THEN GOTO 9820
9800         initializeinventoryfileifnewPFlagTrimI0% = initializeinventoryfileifnewPFlagTrimI0% - 1
9810         GOTO 9780
9820     REM END WHILE
9830     initializeinventoryfileifnewPFlag0$ = LEFT$(invFlagBuf$, initializeinventoryfileifnewPFlagTrimI0%)
9840     initializeinventoryfileifnewPDescTrimI0% = LEN(invDescBuf$)
9850     IF (initializeinventoryfileifnewPDescTrimI0% > 0) = 0 THEN GOTO 9890
9860     IF (MID$(invDescBuf$, initializeinventoryfileifnewPDescTrimI0%, 1) = " ") = 0 THEN GOTO 9890
9870         initializeinventoryfileifnewPDescTrimI0% = initializeinventoryfileifnewPDescTrimI0% - 1
9880         GOTO 9850
9890     REM END WHILE
9900     initializeinventoryfileifnewPDesc0$ = LEFT$(invDescBuf$, initializeinventoryfileifnewPDescTrimI0%)
9910     initializeinventoryfileifnewPQty0% = CVI(invQtyBuf$)
9920     initializeinventoryfileifnewPReorder0% = CVI(invReorderBuf$)
9930     initializeinventoryfileifnewPPrice0! = CVS(invPriceBuf$)
9940     IF (ASC(initializeinventoryfileifnewPFlag0$) = 0) = 0 THEN GOTO 10040
9950         FOR initializeinventoryfileifnewI0% = 1 TO partCOUNT%
9960             ' inv[...] = { ... }  (whole-record write)
9970             LSET invFlagBuf$ = CHR$(255)
9980             LSET invDescBuf$ = ""
9990             LSET invQtyBuf$ = MKI$(0)
10000             LSET invReorderBuf$ = MKI$(0)
10010             LSET invPriceBuf$ = MKS$(0)
10020             PUT #1, initializeinventoryfileifnewI0%
10030         NEXT initializeinventoryfileifnewI0%
10040     REM END IF
10050     RETURN
10060 ' end procedure initializeinventoryfileifnew

10070 ' procedure reportinventoryerror(err%, erl%)
10080     LOCATE 25, 1
10090     errorCode0% = reportinventoryerrorErr0%
10100     GOSUB 2880
10110     PRINT (("There has been an error on line" + STR$(reportinventoryerrorErl0%)) + ": ") + errorResult0$
10120     GOSUB 4460
10130     reportinventoryerrorK0$ = readkeyResult0$
10140     RETURN
10150 ' end procedure reportinventoryerror
