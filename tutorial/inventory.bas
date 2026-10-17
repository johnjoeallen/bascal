10 ' BASCAL generated BASIC -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Strips leading spaces from self$. Not a real MBASIC/BASCOM 2.00 builtin --
40 ' verified against a real IBM BASIC Compiler 2.00 under dosbox-x -- so
50 ' BASCAL ships its own. Declared as a scalar method (see GitHub issue #41)
60 ' so a required stdlib call reads the same way as a built-in method call
70 ' (docs/language/functions-and-procedures.html#built-in-methods). The
80 ' ordinary call form (ltrim$(s$)) still works -- a method's receiver is an
90 ' implicit first parameter, so ordinary-call syntax resolves straight to
100 ' this same declaration, with no separate function needed (and no longer
110 ' allowed: a function and a method sharing one name is a duplicate
120 ' declaration, since they'd both claim the same callable identity).
130 ' Strips trailing spaces from self$. Not a real MBASIC/BASCOM 2.00 builtin --
140 ' verified against a real IBM BASIC Compiler 2.00 under dosbox-x -- so
150 ' BASCAL ships its own. Declared as a scalar method (see GitHub issue #41
160 ' and ltrim.bcl's own doc comment for the reasoning) -- rtrim$(s$) still
170 ' works via ordinary-call syntax resolving to this same declaration.
180 ' Maps an ERR code to its classic MBASIC/GW-BASIC/BASCOM message. Compiles
190 ' and links on a real IBM BASIC Compiler 2.00 as ERROR$, but silently
200 ' returns an empty string at runtime (verified under dosbox-x) -- so BASCAL
210 ' ships a working implementation.
220 ' 
230 ' The named constants below are the complete common subset supported by
240 ' ERROR$: use them in THROW and filtered CATCH clauses instead of magic
250 ' numbers.  Dialect-specific errors outside this shared MBASIC/GW-BASIC/
260 ' BASCOM subset still fall through to ERROR$'s generic message.
270 ' 
280 ' Deliberately NOT a scalar method (see GitHub issue #41, which asked for
290 ' this decision to be recorded either way): code% is an opaque lookup key,
300 ' not a value the call is naturally "operating on" the way ltrim$/rtrim$/
310 ' ucase$/lcase$ operate on their string -- code%.error() would read as if
320 ' the *error code itself* has a message, when really this is a lookup
330 ' table keyed by that code. Stays an ordinary function.
340 CONSTERRSYNTAX% = 2
350 CONSTERRRETURNWITHOUTGOSUB% = 3
360 CONSTERROUTOFDATA% = 4
370 CONSTERRILLEGALFUNCTIONCALL% = 5
380 CONSTERROVERFLOW% = 6
390 CONSTERROUTOFMEMORY% = 7
400 CONSTERRSUBSCRIPTOUTOFRANGE% = 9
410 CONSTERRDUPLICATEDEFINITION% = 10
420 CONSTERRDIVISIONBYZERO% = 11
430 CONSTERRTYPEMISMATCH% = 13
440 CONSTERROUTOFSTRINGSPACE% = 14
450 CONSTERRNORESUME% = 19
460 CONSTERRRESUMEWITHOUTERROR% = 20
470 CONSTERRDEVICETIMEOUT% = 24
480 CONSTERRDEVICEFAULT% = 25
490 CONSTERROUTOFPAPER% = 27
500 CONSTERRBADFILENUMBER% = 52
510 CONSTERRFILENOTFOUND% = 53
520 CONSTERRBADFILEMODE% = 54
530 CONSTERRFILEALREADYOPEN% = 55
540 CONSTERRDEVICEIO% = 57
550 CONSTERRFILEALREADYEXISTS% = 58
560 CONSTERRDISKFULL% = 61
570 CONSTERRINPUTPASTEND% = 62
580 CONSTERRBADRECORDNUMBER% = 63
590 CONSTERRBADFILENAME% = 64
600 CONSTERRTOOMANYFILES% = 67
610 CONSTERRDEVICEUNAVAILABLE% = 68
620 CONSTERRDISKWRITEPROTECTED% = 70
630 CONSTERRDISKNOTREADY% = 71
640 CONSTERRDISKMEDIAERROR% = 72
650 CONSTERRPATHFILEACCESS% = 75
660 CONSTERRPATHNOTFOUND% = 76
670 ' ============================================================
680 ' INVENTORY.BCL -- Random-Access Inventory Program
690 ' 
700 ' A BASCAL reconstruction of "Example program for RANDOM ACCESS
710 ' FILE study", by fhb, 8/19/98, from Joseph Sixpack's GW-BASIC
720 ' programs page (part of his "Last Book of GW-Basic" collection):
730 ' http://www.geocities.ws/joseph_sixpack/binventory.html
740 ' fhb's own header comment credits the original as "suggested
750 ' from MS-BASIC manual".
760 ' 
770 ' This is a reconstruction, not a line-by-line port -- some
780 ' original pieces have no BASCAL equivalent and were dropped
790 ' rather than approximated:
800 ' - The GOTO-driven "subroutine roadmap" dispatcher at the top
810 ' of fhb's listing (a `LIST 110-320` etc. navigation aid for
820 ' editing in the GW-BASIC interpreter) has no meaning once the
830 ' program is structured into named function/procedure blocks.
840 ' - `KEY OFF` / `KEY I,""` (clearing the function-key soft-label
850 ' row) and `VIEW PRINT` (scroll-region windowing for the list
860 ' screen) are interpreter/console features BASCAL doesn't
870 ' expose.
880 ' - fhb's own hand-rolled numeric-ERR-code-to-message lookup table
890 ' (ERR=1 "Input value overflow", ERR=2 "Syntax error", ... ERR=25)
900 ' is replaced below by BASCAL's com.bascal.stdlib.error library
910 ' (ERROR$(code%)) -- same idea, BASCAL's own table; it still
920 ' doesn't decode ERL, which errorTrap() reports as the raw line
930 ' number.
940 ' - fhb's one-time "hidden" datafile initializer (PUT-ing 100
950 ' blank, CHR$(255)-flagged records) is reproduced below as
960 ' initializeInventoryFileIfNew(), called once at program entry --
970 ' inven.dat no longer has to be pre-populated by hand.
980 ' - The three original tab-position constants (T=20, U=25,
990 ' V=30) are collapsed into a single `TAB_COL = 20`; a couple of
1000 ' screens that used U=25 in the original (see showAddStockScreen
1010 ' below) keep 25 as a literal rather than reusing TAB_COL.
1020 ' 
1030 ' Tracks parts in a fixed 100-record file: check status, add,
1040 ' edit, add/subtract stock, and a reorder report.
1050 ' 
1060 ' Error handling uses try/catch (GitHub issue #60), not the raw `on
1070 ' error goto` / `resume next` fhb's original relies on: a failed menu
1080 ' action is abandoned outright and the program returns straight to the
1090 ' main menu, rather than resuming at the exact instruction after
1100 ' whatever failed -- see reportInventoryError() below and
1110 ' tutorial/inventory_try_catch.draft's own header comment for why. This
1120 ' is a real, deliberate behavior change from an earlier on-error-goto
1130 ' version of this file, which *was* verified against real BASCOM 2.00
1140 ' under dosbox-x (only with the /E and /X switches -- error trapping
1150 ' isn't linked in by default); the try/catch shape below transpiles to
1160 ' the same ON ERROR GOTO/RESUME primitives BASCOM accepts, but hasn't
1170 ' itself been independently re-verified against a real BASCOM compile.
1180 ' ============================================================
1190 ' BASCAL-ism: the record/file DSL. `record ... end record` plus
1200 ' `file ... as ... = open(...)` below replace fhb's manual
1210 ' FIELD #1,1 AS F$,30 AS D$,2 AS Q$,... buffer layout entirely --
1220 ' bcc computes the field widths and record LEN from this
1230 ' declaration and generates the FIELD statement itself. Named
1240 ' field access (`p.flag`, `p.qty`, ...) and whole-record
1250 ' read/write via `inv[n]` (see checkPart() below) replace fhb's
1260 ' manual GET/PUT plus LSET/RSET and MKI$/MKS$/CVI$/CVS$ packing.
1270 ' BASCAL-ism: `const` is a real compile-time constant, not a plain
1280 ' variable assignment like fhb's `N=100` / `T=20` -- it can never
1290 ' be reassigned, and resolves to the same value everywhere,
1300 ' including inside every function/procedure below, with no
1310 ' `global` declaration needed.
1320 CONSTPARTCOUNT% = 100
1330 CONSTTABCOL% = 20
1340 ' `file ... = open(...)` is sugar for OPEN ... FOR RANDOM AS #n
1350 ' LEN = <record width> plus the FIELD statement fhb wrote out by
1360 ' hand at his line 550. Wrapped in its own try/catch: a file that
1370 ' exists but can't be opened for random access (permissions, a
1380 ' read-only inven.dat, disk full on the fallback create) is a real,
1390 ' trappable error (code 75, "Path/File access error") on both
1400 ' targets now, not a hard crash -- report it and exit cleanly
1410 ' instead of leaving the program to fail confusingly the first time
1420 ' something tries to use an `inv` that was never actually opened.
1430 ON ERROR GOTO 1500
1440 BCCTRY0001PENDING% = 0
1450 ' file inv as Part = open(...)  [39 bytes/record]
1460 OPEN "inven.dat" FOR RANDOM AS #1 LEN = 39
1470 FIELD #1, 1 AS invFlagBuf$, 30 AS invDescBuf$, 2 AS invQtyBuf$, 2 AS invReorderBuf$, 4 AS invPriceBuf$
1480 ON ERROR GOTO 0
1490 GOTO 1650
1500 BCCTRY0001PENDING% = ERR
1510 BCCERR% = ERR
1520 BCCERL% = ERL
1530 RESUME 1540
1540 ON ERROR GOTO 1630
1550 errorCode0% = BCCERR%
1560 GOSUB 3300
1570 BCCT2$ = errorResult0$
1580 PRINT "could not open inven.dat: " + BCCT2$
1590 END
1600 BCCTRY0001PENDING% = 0
1610 ON ERROR GOTO 0
1620 GOTO 1650
1630 BCCTRY0001PENDING% = ERR
1640 RESUME 1650
1650 ON ERROR GOTO 0
1660 IF BCCTRY0001PENDING% <> 0 THEN ERROR BCCTRY0001PENDING%
1670 REM END TRY
1680 ' -------------------- Pure functions (no file access) --------------------
1690 ' BASCAL-ism: `function ... end function` with `return` replaces
1700 ' fhb's convention of a GOSUB target plus a bare RETURN -- there's
1710 ' no separate "subroutine label" and no shared/global result
1720 ' variable to manage by hand; `isEmpty%(...)` is called like an
1730 ' ordinary expression at every use below (e.g. `isEmpty%(p.flag)`).
1740 ' A record whose flag byte is CHR$(255) is an empty/never-used slot.
1750 ' BASCAL-ism: `&&` and `||` are short-circuit AND/OR -- real
1760 ' MBASIC/BASCOM only has bitwise AND/OR (which fhb relies on here
1770 ' too, since `PART!<1 OR PART!>N!` never short-circuits anyway).
1780 ' BASCAL lowers `&&`/`||` into the equivalent branching so the
1790 ' short-circuit *is* real at the generated-BASIC level; see the
1800 ' manual's "Short-Circuit && and ||" section
1810 ' (https://johnjoeallen.github.io/bascal/manual/).
1820 ' -------------------- Keyboard input --------------------
1830 ' BASCAL-ism: `do ... loop until` is a structured post-check loop
1840 ' replacing fhb's `730 KP$=INKEY$:IF KP$="" THEN 730` GOTO-polling
1850 ' idiom. `inkey$` itself is the real INKEY$ builtin passed straight
1860 ' through, resolving correctly from inside a function/procedure
1870 ' body like this one -- every menu action below calls
1880 ' readKey$()/waitAnyKey() rather than polling INKEY$ inline.
1890 ' -------------------- Display procedures --------------------
1900 ' BASCAL-ism: no `VIEW PRINT` (see the header note above), so this
1910 ' deliberately does NOT pin a "press any key" line to a fixed row the way
1920 ' fhb's original does -- a bare `LOCATE 25, ...` sitting under content
1930 ' that keeps printing past it (listAll()'s own items) collides with
1940 ' whatever's later written there, since nothing here scrolls a bounded
1950 ' region: waitAnyKey() is the only thing that ever touches row 25, and
1960 ' only right when it actually blocks (see listAll()'s own redraw-per-page
1970 ' structure below).
1980 ' byref scalar parameters: gatherPartDetails writes the four editable
1990 ' fields for a part directly back into the caller's variables.
2000 ' -------------------- Menu actions --------------------
2010 ' fhb's own one-time "hidden" datafile initializer PUT-ing 100 blank,
2020 ' CHR$(255)-flagged records (see the header note above) -- reproduced
2030 ' here so inven.dat no longer has to be pre-populated by hand before
2040 ' running this program. A brand-new file OPEN created just now (rather
2050 ' than one that already existed) reads back as all-zero bytes: record
2060 ' 1's flag byte is CHR$(0), never CHR$(255) -- the one signal an
2070 ' already-populated file (whose record 1 flag is always either
2080 ' CHR$(255), still an empty slot, or a real part's own "1") could never
2090 ' produce, so it's what isEmpty%() itself can't use (see its own
2100 ' header note) but this one-time check safely can.
2110 ' -------------------- Program entry --------------------
2120 CLS
2130 GOSUB 9900
2140     GOSUB 5070
2150     GOSUB 4920
2160     kp$ = readkeyResult0$
2170     IF (INSTR("1234567cCeElLaAsSrRxX", kp$) <> 0) = 0 THEN GOTO 2870
2180         ' BASCAL-ism: `select case` replaces fhb's chain of eight
2190         ' `IF VAL(KP$)=n OR KP$="x" OR KP$="X" THEN GOTO ...` lines
2200         ' (his 770-840) with one multi-way dispatch.
2210         ' 
2220         ' BASCAL-ism: `try`/`catch` (issue #60) replaces fhb's own global
2230         ' `ON ERROR GOTO` trap. A failed menu action is abandoned outright
2240         ' here -- the `catch` below runs, then execution continues right
2250         ' after `end try`, back at `loop until` -- rather than resuming at
2260         ' the exact instruction after whatever failed inside checkPart()/
2270         ' editRecord()/etc. the way fhb's `RESUME NEXT` did. See
2280         ' reportInventoryError() below and tutorial/inventory_try_catch.
2290         ' draft's own header comment for why that arbitrary resume-point
2300         ' behavior isn't something try/catch reproduces.
2310         ON ERROR GOTO 2710
2320         BCCTRY0004PENDING% = 0
2330         BCCT6$ = kp$
2340         IF (BCCT6$ = "1" OR BCCT6$ = "c" OR BCCT6$ = "C") <> 0 THEN GOTO 2420
2350         IF (BCCT6$ = "2" OR BCCT6$ = "e" OR BCCT6$ = "E") <> 0 THEN GOTO 2440
2360         IF (BCCT6$ = "3" OR BCCT6$ = "l" OR BCCT6$ = "L") <> 0 THEN GOTO 2460
2370         IF (BCCT6$ = "4" OR BCCT6$ = "a" OR BCCT6$ = "A") <> 0 THEN GOTO 2480
2380         IF (BCCT6$ = "5" OR BCCT6$ = "s" OR BCCT6$ = "S") <> 0 THEN GOTO 2500
2390         IF (BCCT6$ = "6" OR BCCT6$ = "r" OR BCCT6$ = "R") <> 0 THEN GOTO 2520
2400         IF (BCCT6$ = "7" OR BCCT6$ = "x" OR BCCT6$ = "X") <> 0 THEN GOTO 2540
2410         GOTO 2680
2420             GOSUB 6580
2430             GOTO 2680
2440             GOSUB 7090
2450             GOTO 2680
2460             GOSUB 7750
2470             GOTO 2680
2480             GOSUB 8120
2490             GOTO 2680
2500             GOSUB 8760
2510             GOTO 2680
2520             GOSUB 9490
2530             GOTO 2680
2540             ' BASCAL-ism: `inv.close()` is sugar for `CLOSE #1`,
2550             ' matching fhb's own `90 CLOSE:SYSTEM`. fhb's original
2560             ' also had a separate "Quit to BASIC" option (his own
2570             ' 7, returning to the interpreter's command prompt
2580             ' rather than exiting to DOS) -- dropped here: a
2590             ' compiled program has no interpreter to return to,
2600             ' so it was never anything but a second spelling of
2610             ' this same close-and-exit action.
2620             ' inv.close()
2630             CLOSE #1
2640             COLOR 7, 0
2650             CLS
2660             SYSTEM
2670             GOTO 2680
2680         REM END SELECT
2690         ON ERROR GOTO 0
2700         GOTO 2840
2710         BCCTRY0004PENDING% = ERR
2720         BCCERR% = ERR
2730         BCCERL% = ERL
2740         RESUME 2750
2750         ON ERROR GOTO 2820
2760         reportinventoryerrorErr0% = BCCERR%
2770         reportinventoryerrorErl0% = BCCERL%
2780         GOSUB 10180
2790         BCCTRY0004PENDING% = 0
2800         ON ERROR GOTO 0
2810         GOTO 2840
2820         BCCTRY0004PENDING% = ERR
2830         RESUME 2840
2840         ON ERROR GOTO 0
2850         IF BCCTRY0004PENDING% <> 0 THEN ERROR BCCTRY0004PENDING%
2860         REM END TRY
2870     REM END IF
2880 GOTO 2140
2890 REM END DO
2900 ' -------------------- Error handling --------------------
2910 ' err%/erl% are ordinary locals scoped to the `catch` block above, not
2920 ' aliases for the ambient (readable-anywhere) `err`/`erl` pseudo-
2930 ' variables `on error goto` uses -- see `Statement::TryCatch`'s own doc
2940 ' comment in ast.rs. Passed straight through to ERROR$ here like fhb's
2950 ' own ERR/ERL (his 3390: "an error on line";ERL), decoded through
2960 ' BASCAL's own com.bascal.stdlib.error (ERROR$) instead of fhb's
2970 ' hand-rolled lookup table -- see the header note above. try/catch
2980 ' itself isn't documented in the manual yet (GitHub issue #60 tracks
2990 ' the still-unfinished C-target work; the manual page can follow once
3000 ' that lands) -- see ast.rs's own `Statement::TryCatch` doc comment for
3010 ' the full semantics meanwhile.
3020 END

3030 ' function ltrim$(self$)
3040     ltrimI0% = 1
3050     IF (ltrimI0% <= LEN(ltrimSelf0$)) = 0 THEN GOTO 3090
3060     IF (MID$(ltrimSelf0$, ltrimI0%, 1) = " ") = 0 THEN GOTO 3090
3070         ltrimI0% = ltrimI0% + 1
3080     GOTO 3050
3090     REM END WHILE
3100     BCCT10$ = ltrimSelf0$
3110     BCCT11% = ltrimI0%
3120     BCCT12$ = MID$(BCCT10$, BCCT11%)
3130     ltrimResult0$ = BCCT12$
3140     RETURN
3150 ' end function ltrim$

3160 ' function rtrim$(self$)
3170     rtrimI0% = LEN(rtrimSelf0$)
3180     IF (rtrimI0% > 0) = 0 THEN GOTO 3220
3190     IF (MID$(rtrimSelf0$, rtrimI0%, 1) = " ") = 0 THEN GOTO 3220
3200         rtrimI0% = rtrimI0% - 1
3210     GOTO 3180
3220     REM END WHILE
3230     BCCT15$ = rtrimSelf0$
3240     BCCT16% = rtrimI0%
3250     BCCT17$ = LEFT$(BCCT15$, BCCT16%)
3260     rtrimResult0$ = BCCT17$
3270     RETURN
3280 ' end function rtrim$

3290 ' function error$(code%)
3300     BCCT19% = errorCode0%
3310     IF (BCCT19% = CONSTERRSYNTAX%) <> 0 THEN GOTO 3650
3320     IF (BCCT19% = CONSTERRRETURNWITHOUTGOSUB%) <> 0 THEN GOTO 3680
3330     IF (BCCT19% = CONSTERROUTOFDATA%) <> 0 THEN GOTO 3710
3340     IF (BCCT19% = CONSTERRILLEGALFUNCTIONCALL%) <> 0 THEN GOTO 3740
3350     IF (BCCT19% = CONSTERROVERFLOW%) <> 0 THEN GOTO 3770
3360     IF (BCCT19% = CONSTERROUTOFMEMORY%) <> 0 THEN GOTO 3800
3370     IF (BCCT19% = CONSTERRSUBSCRIPTOUTOFRANGE%) <> 0 THEN GOTO 3830
3380     IF (BCCT19% = CONSTERRDUPLICATEDEFINITION%) <> 0 THEN GOTO 3860
3390     IF (BCCT19% = CONSTERRDIVISIONBYZERO%) <> 0 THEN GOTO 3890
3400     IF (BCCT19% = CONSTERRTYPEMISMATCH%) <> 0 THEN GOTO 3920
3410     IF (BCCT19% = CONSTERROUTOFSTRINGSPACE%) <> 0 THEN GOTO 3950
3420     IF (BCCT19% = CONSTERRNORESUME%) <> 0 THEN GOTO 3980
3430     IF (BCCT19% = CONSTERRRESUMEWITHOUTERROR%) <> 0 THEN GOTO 4010
3440     IF (BCCT19% = CONSTERRDEVICETIMEOUT%) <> 0 THEN GOTO 4040
3450     IF (BCCT19% = CONSTERRDEVICEFAULT%) <> 0 THEN GOTO 4070
3460     IF (BCCT19% = CONSTERROUTOFPAPER%) <> 0 THEN GOTO 4100
3470     IF (BCCT19% = CONSTERRBADFILENUMBER%) <> 0 THEN GOTO 4130
3480     IF (BCCT19% = CONSTERRFILENOTFOUND%) <> 0 THEN GOTO 4160
3490     IF (BCCT19% = CONSTERRBADFILEMODE%) <> 0 THEN GOTO 4190
3500     IF (BCCT19% = CONSTERRFILEALREADYOPEN%) <> 0 THEN GOTO 4220
3510     IF (BCCT19% = CONSTERRDEVICEIO%) <> 0 THEN GOTO 4250
3520     IF (BCCT19% = CONSTERRFILEALREADYEXISTS%) <> 0 THEN GOTO 4280
3530     IF (BCCT19% = CONSTERRDISKFULL%) <> 0 THEN GOTO 4310
3540     IF (BCCT19% = CONSTERRINPUTPASTEND%) <> 0 THEN GOTO 4340
3550     IF (BCCT19% = CONSTERRBADRECORDNUMBER%) <> 0 THEN GOTO 4370
3560     IF (BCCT19% = CONSTERRBADFILENAME%) <> 0 THEN GOTO 4400
3570     IF (BCCT19% = CONSTERRTOOMANYFILES%) <> 0 THEN GOTO 4430
3580     IF (BCCT19% = CONSTERRDEVICEUNAVAILABLE%) <> 0 THEN GOTO 4460
3590     IF (BCCT19% = CONSTERRDISKWRITEPROTECTED%) <> 0 THEN GOTO 4490
3600     IF (BCCT19% = CONSTERRDISKNOTREADY%) <> 0 THEN GOTO 4520
3610     IF (BCCT19% = CONSTERRDISKMEDIAERROR%) <> 0 THEN GOTO 4550
3620     IF (BCCT19% = CONSTERRPATHFILEACCESS%) <> 0 THEN GOTO 4580
3630     IF (BCCT19% = CONSTERRPATHNOTFOUND%) <> 0 THEN GOTO 4610
3640     GOTO 4640
3650         errorResult0$ = "Syntax error"
3660         RETURN
3670         GOTO 4680
3680         errorResult0$ = "RETURN without GOSUB"
3690         RETURN
3700         GOTO 4680
3710         errorResult0$ = "Out of DATA"
3720         RETURN
3730         GOTO 4680
3740         errorResult0$ = "Illegal function call"
3750         RETURN
3760         GOTO 4680
3770         errorResult0$ = "Overflow"
3780         RETURN
3790         GOTO 4680
3800         errorResult0$ = "Out of memory"
3810         RETURN
3820         GOTO 4680
3830         errorResult0$ = "Subscript out of range"
3840         RETURN
3850         GOTO 4680
3860         errorResult0$ = "Duplicate Definition"
3870         RETURN
3880         GOTO 4680
3890         errorResult0$ = "Division by zero"
3900         RETURN
3910         GOTO 4680
3920         errorResult0$ = "Type mismatch"
3930         RETURN
3940         GOTO 4680
3950         errorResult0$ = "Out of string space"
3960         RETURN
3970         GOTO 4680
3980         errorResult0$ = "No RESUME"
3990         RETURN
4000         GOTO 4680
4010         errorResult0$ = "RESUME without error"
4020         RETURN
4030         GOTO 4680
4040         errorResult0$ = "Device timeout"
4050         RETURN
4060         GOTO 4680
4070         errorResult0$ = "Device fault"
4080         RETURN
4090         GOTO 4680
4100         errorResult0$ = "Out of paper"
4110         RETURN
4120         GOTO 4680
4130         errorResult0$ = "Bad file number"
4140         RETURN
4150         GOTO 4680
4160         errorResult0$ = "File not found"
4170         RETURN
4180         GOTO 4680
4190         errorResult0$ = "Bad file mode"
4200         RETURN
4210         GOTO 4680
4220         errorResult0$ = "File already open"
4230         RETURN
4240         GOTO 4680
4250         errorResult0$ = "Device I/O error"
4260         RETURN
4270         GOTO 4680
4280         errorResult0$ = "File already exists"
4290         RETURN
4300         GOTO 4680
4310         errorResult0$ = "Disk full"
4320         RETURN
4330         GOTO 4680
4340         errorResult0$ = "Input past end"
4350         RETURN
4360         GOTO 4680
4370         errorResult0$ = "Bad record number"
4380         RETURN
4390         GOTO 4680
4400         errorResult0$ = "Bad file name"
4410         RETURN
4420         GOTO 4680
4430         errorResult0$ = "Too many files"
4440         RETURN
4450         GOTO 4680
4460         errorResult0$ = "Device unavailable"
4470         RETURN
4480         GOTO 4680
4490         errorResult0$ = "Disk write protected"
4500         RETURN
4510         GOTO 4680
4520         errorResult0$ = "Disk not ready"
4530         RETURN
4540         GOTO 4680
4550         errorResult0$ = "Disk media error"
4560         RETURN
4570         GOTO 4680
4580         errorResult0$ = "Path/File access error"
4590         RETURN
4600         GOTO 4680
4610         errorResult0$ = "Path not found"
4620         RETURN
4630         GOTO 4680
4640         BCCT20% = errorCode0%
4650         BCCT21$ = STR$(BCCT20%)
4660         errorResult0$ = "Error " + BCCT21$
4670         RETURN
4680     REM END SELECT
4690     RETURN
4700 ' end function error$

4710 ' function isempty%(flag$)
4720     BCCT22$ = isemptyFlag0$
4730     BCCT23% = ASC(BCCT22$)
4740     isemptyResult0% = BCCT23% = 255
4750     RETURN
4760 ' end function isempty%

4770 ' function partinrange%(n%)
4780     IF (partinrangeN0% >= 1) = 0 THEN GOTO 4820
4790     IF (partinrangeN0% <= CONSTPARTCOUNT%) = 0 THEN GOTO 4820
4800         partinrangeResult0% = 1
4810         RETURN
4820     REM END IF
4830     partinrangeResult0% = 0
4840     RETURN
4850 ' end function partinrange%

4860 ' function readpartnumberinput$()
4870     INPUT "Input part number"; readpartnumberinputS0$
4880     readpartnumberinputResult0$ = readpartnumberinputS0$
4890     RETURN
4900 ' end function readpartnumberinput$

4910 ' function readkey$()
4920         readkeyK0$ = INKEY$
4930         IF (readkeyK0$ <> "") = 0 THEN GOTO 4920
4940     REM END DO
4950     readkeyResult0$ = readkeyK0$
4960     RETURN
4970 ' end function readkey$

4980 ' procedure waitanykey()
4990     LOCATE 25, 10
5000     PRINT "Press the AnyKey to continue...";
5010         waitanykeyK0$ = INKEY$
5020         IF (waitanykeyK0$ <> "") = 0 THEN GOTO 5010
5030     REM END DO
5040     RETURN
5050 ' end procedure waitanykey

5060 ' procedure showmainmenu()
5070     CLS
5080     COLOR 14, 4
5090     CLS
5100     LOCATE 6, 1
5110     PRINT
5120     ' `tab(n)` passes straight through to real TAB(n), same as
5130     ' fhb's own `PRINT TAB(V) "..."` -- but only as a bare item in
5140     ' a PRINT list, juxtaposed or `;`-separated like here. Real
5150     ' BASCOM rejects `"literal" + tab(n) + ...` (TAB isn't a real
5160     ' string function you can concatenate); see printListHeader()
5170     ' and printReorderHeader() below, which need `;` between a
5180     ' preceding string and a `tab(n)` for exactly this reason.
5190     PRINT TAB(30)"Inventory Program"
5200     PRINT
5210     PRINT TAB(CONSTTABCOL%)"1......C)heck a part"
5220     PRINT TAB(CONSTTABCOL%)"2......E)dit/overwrite/add a part"
5230     PRINT TAB(CONSTTABCOL%)("3......L)ist all" + STR$(CONSTPARTCOUNT%)) + "parts"
5240     PRINT TAB(CONSTTABCOL%)"4......A)dd stock"
5250     PRINT TAB(CONSTTABCOL%)"5......S)ubtract stock"
5260     PRINT TAB(CONSTTABCOL%)"6......R)eorder Report"
5270     PRINT
5280     PRINT TAB(CONSTTABCOL%)"7......eX)it to system"
5290     RETURN
5300 ' end procedure showmainmenu

5310 ' procedure showbadpartnumber()
5320     CLS
5330     LOCATE 10, 10
5340     PRINT "Part number is out of permissable range of 1 to" + STR$(CONSTPARTCOUNT%)
5350     RETURN
5360 ' end procedure showbadpartnumber

5370 ' procedure showrangeretrymessage()
5380     LOCATE 10, 15
5390     PRINT "The Part number is out of permissable range of 1 to" + STR$(CONSTPARTCOUNT%)
5400     LOCATE 25, 15
5410     PRINT "Press the Anykey to reenter part number...";
5420     RETURN
5430 ' end procedure showrangeretrymessage

5440 ' procedure shownullentrymessage(partStr$)
5450     LOCATE 10, CONSTTABCOL%
5460     PRINT ("Part number " + shownullentrymessagePartStr0$) + " is a null entry"
5470     RETURN
5480 ' end procedure shownullentrymessage

5490 ' procedure showpartstatus(partNum%, desc$, qty%, reorder%, price!)
5500     CLS
5510     LOCATE 5, 1
5520     PRINT TAB(CONSTTABCOL%)"Inventory Status for Individual Part Number"
5530     PRINT TAB(CONSTTABCOL%)"==========================================="
5540     PRINT
5550     PRINT
5560     PRINT TAB(CONSTTABCOL%)"     Part number:  " + STR$(showpartstatusPartNum0%)
5570     PRINT
5580     PRINT TAB(CONSTTABCOL%)"       Item name:  " + showpartstatusDesc0$
5590     PRINT TAB(CONSTTABCOL%)"Quantity on hand:  " + STR$(showpartstatusQty0%)
5600     PRINT TAB(CONSTTABCOL%)"   Reorder level:  " + STR$(showpartstatusReorder0%)
5610     PRINT TAB(CONSTTABCOL%)"      Unit price:  " + STR$(showpartstatusPrice0!)
5620     RETURN
5630 ' end procedure showpartstatus

5640 ' procedure printlistheader()
5650     CLS
5660     PRINT TAB(25)"I N V E N T O R Y   L I S T I N G"; TAB(65); STR$(CONSTPARTCOUNT%) + "items"
5670     PRINT "                                          Quantity       Reorder"
5680     PRINT " Partno           Description             on hand         level"
5690     RETURN
5700 ' end procedure printlistheader

5710 ' procedure printinventoryline(partNum%, desc$, qty%, reorder%)
5720     PRINT (((((STR$(printinventorylinePartNum0%) + "  ") + printinventorylineDesc0$) + "   ") + STR$(printinventorylineQty0%)) + "          ") + STR$(printinventorylineReorder0%)
5730     RETURN
5740 ' end procedure printinventoryline

5750 ' procedure printreorderheader()
5760     CLS
5770     LOCATE 1, CONSTTABCOL%
5780     PRINT "Reorder Report"; TAB(55); DATE$
5790     PRINT
5800     PRINT "                                             Quantity       Reorder"
5810     PRINT "    Partno           Description             on hand         level"
5820     PRINT "   =======  ==============================   ========       ======="
5830     RETURN
5840 ' end procedure printreorderheader

5850 ' procedure printreorderline(partNum%, desc$, qty%, reorder%)
5860     PRINT (((((("  " + STR$(printreorderlinePartNum0%)) + "  ") + printreorderlineDesc0$) + "   ") + STR$(printreorderlineQty0%)) + "          ") + STR$(printreorderlineReorder0%)
5870     RETURN
5880 ' end procedure printreorderline

5890 ' procedure gatherpartdetails(partNum%, desc$, qty%, reorder%, price!)
5900     CLS
5910     LOCATE 4, CONSTTABCOL%
5920     PRINT "Adding or Overwriting a Record"
5930     LOCATE 8, CONSTTABCOL%
5940     PRINT "Record/Partno" + STR$(gatherpartdetailsPartNum0%)
5950     LOCATE 11, 39
5960     PRINT "------------------------------"
5970     LOCATE 10, CONSTTABCOL%
5980     INPUT "      Description"; gatherpartdetailsDesc0$
5990     LOCATE 12, CONSTTABCOL%
6000     INPUT "Quantity in stock"; gatherpartdetailsQty0%
6010     LOCATE 14, CONSTTABCOL%
6020     INPUT "    Reorder level"; gatherpartdetailsReorder0%
6030     LOCATE 16, CONSTTABCOL%
6040     INPUT "       Unit price"; gatherpartdetailsPrice0!
6050     LOCATE 18, CONSTTABCOL%
6060     PRINT "Is information correct (Y/N)?"
6070     RETURN
6080 ' end procedure gatherpartdetails

6090 ' procedure showaddstockscreen(partNum%, desc$, qty%, reorder%)
6100     CLS
6110     LOCATE 4, 25
6120     PRINT "Add to an inventory part number"
6130     LOCATE 5, 25
6140     PRINT "==============================="
6150     LOCATE 8, CONSTTABCOL%
6160     PRINT "     Part number: " + STR$(showaddstockscreenPartNum0%)
6170     LOCATE 9, CONSTTABCOL%
6180     PRINT "Item description: " + showaddstockscreenDesc0$
6190     LOCATE 10, CONSTTABCOL%
6200     PRINT "Quantity on hand: " + STR$(showaddstockscreenQty0%)
6210     LOCATE 11, CONSTTABCOL%
6220     PRINT "   Reorder Level: " + STR$(showaddstockscreenReorder0%)
6230     RETURN
6240 ' end procedure showaddstockscreen

6250 ' procedure shownegativeqtywarning()
6260     LOCATE 17, 15
6270     PRINT "The quantity to add must NOT be a negative number"
6280     LOCATE 25, 1
6290     PRINT "Please press the Anykey to reenter quantity to add...";
6300     RETURN
6310 ' end procedure shownegativeqtywarning

6320 ' procedure showsubtractstockscreen(partNum%, desc$, qty%, reorder%)
6330     CLS
6340     LOCATE 4, CONSTTABCOL%
6350     PRINT "Subtract an inventory part number"
6360     LOCATE 5, CONSTTABCOL%
6370     PRINT "================================="
6380     LOCATE 8, CONSTTABCOL%
6390     PRINT "         Part number: " + STR$(showsubtractstockscreenPartNum0%)
6400     LOCATE 9, CONSTTABCOL%
6410     PRINT "    Item description: " + showsubtractstockscreenDesc0$
6420     LOCATE 10, CONSTTABCOL%
6430     PRINT "    Quantity on hand: " + STR$(showsubtractstockscreenQty0%)
6440     LOCATE 11, CONSTTABCOL%
6450     PRINT "       Reorder Level: " + STR$(showsubtractstockscreenReorder0%)
6460     RETURN
6470 ' end procedure showsubtractstockscreen

6480 ' procedure showoversubtractwarning(onHand%)
6490     LOCATE 17, 5
6500     PRINT "The quantity to SUBTRACT must NOT result in NEGATIVE inventory"
6510     LOCATE 18, 5
6520     PRINT ("Only" + STR$(showoversubtractwarningOnHand0%)) + " IN STOCK"
6530     LOCATE 25, 1
6540     PRINT "Please press the Anykey to reenter quantity to subtract...";
6550     RETURN
6560 ' end procedure showoversubtractwarning

6570 ' procedure checkpart()
6580     ' global inv
6590     GOSUB 4870
6600     checkpartPartStr0$ = readpartnumberinputResult0$
6610     checkpartPart0% = VAL(checkpartPartStr0$)
6620     partinrangeN0% = checkpartPart0%
6630     GOSUB 4780
6640     BCCT28% = partinrangeResult0%
6650     IF (BCCT28% = 0) = 0 THEN GOTO 6690
6660         GOSUB 5320
6670         GOSUB 4990
6680         RETURN
6690     REM END IF
6700     ' BASCAL-ism: `let p = inv[part%]` reads record `part%` of the
6710     ' `inv` file into a local record variable `p` -- one expression
6720     ' for what fhb's `GET #1, PART!` plus five separate field reads
6730     ' (F$, D$, CVI(Q$), CVI(R$), CVS(P$)) did by hand. The write
6740     ' side, `inv[part%] = { ... }` (see editRecord() below), is the
6750     ' same sugar for PUT plus the LSET/MKx$ packing it replaces.
6760     ' let p = inv[...]  (whole-record read)
6770     GET #1, checkpartPart0%
6780     rtrimSelf0$ = invFlagBuf$
6790     GOSUB 3170
6800     BCCT30$ = rtrimResult0$
6810     checkpartPFlag0$ = BCCT30$
6820     rtrimSelf0$ = invDescBuf$
6830     GOSUB 3170
6840     BCCT31$ = rtrimResult0$
6850     checkpartPDesc0$ = BCCT31$
6860     checkpartPQty0% = CVI(invQtyBuf$)
6870     checkpartPReorder0% = CVI(invReorderBuf$)
6880     checkpartPPrice0! = CVS(invPriceBuf$)
6890     isemptyFlag0$ = checkpartPFlag0$
6900     GOSUB 4720
6910     BCCT32% = isemptyResult0%
6920     IF (BCCT32%) = 0 THEN GOTO 6980
6930         CLS
6940         LOCATE 10, 18
6950         PRINT ("Part number" + STR$(checkpartPart0%)) + "is still a null entry at this time"
6960         GOSUB 4990
6970         RETURN
6980     REM END IF
6990     showpartstatusPartNum0% = checkpartPart0%
7000     showpartstatusDesc0$ = checkpartPDesc0$
7010     showpartstatusQty0% = checkpartPQty0%
7020     showpartstatusReorder0% = checkpartPReorder0%
7030     showpartstatusPrice0! = checkpartPPrice0!
7040     GOSUB 5500
7050     GOSUB 4990
7060     RETURN
7070 ' end procedure checkpart

7080 ' procedure editrecord()
7090     ' global inv
7100     CLS
7110     LOCATE 10, CONSTTABCOL%
7120     GOSUB 4870
7130     editrecordPartStr0$ = readpartnumberinputResult0$
7140     editrecordPart0% = VAL(editrecordPartStr0$)
7150     partinrangeN0% = editrecordPart0%
7160     GOSUB 4780
7170     BCCT34% = partinrangeResult0%
7180     IF (BCCT34% = 0) = 0 THEN GOTO 7220
7190         GOSUB 5320
7200         GOSUB 4990
7210         RETURN
7220     REM END IF
7230     ' let p = inv[...]  (whole-record read)
7240     GET #1, editrecordPart0%
7250     rtrimSelf0$ = invFlagBuf$
7260     GOSUB 3170
7270     BCCT36$ = rtrimResult0$
7280     editrecordPFlag0$ = BCCT36$
7290     rtrimSelf0$ = invDescBuf$
7300     GOSUB 3170
7310     BCCT37$ = rtrimResult0$
7320     editrecordPDesc0$ = BCCT37$
7330     editrecordPQty0% = CVI(invQtyBuf$)
7340     editrecordPReorder0% = CVI(invReorderBuf$)
7350     editrecordPPrice0! = CVS(invPriceBuf$)
7360     isemptyFlag0$ = editrecordPFlag0$
7370     GOSUB 4720
7380     BCCT38% = isemptyResult0%
7390     IF (BCCT38% = 0) = 0 THEN GOTO 7480
7400         LOCATE 12, CONSTTABCOL%
7410         PRINT "Overwrite existing part data?"
7420         GOSUB 4920
7430         editrecordKp0$ = readkeyResult0$
7440         IF (editrecordKp0$ <> "Y") = 0 THEN GOTO 7470
7450         IF (editrecordKp0$ <> "y") = 0 THEN GOTO 7470
7460             RETURN
7470         REM END IF
7480     REM END IF
7490         gatherpartdetailsPartNum0% = editrecordPart0%
7500         gatherpartdetailsDesc0$ = editrecordEditDesc0$
7510         gatherpartdetailsQty0% = editrecordEditQty0%
7520         gatherpartdetailsReorder0% = editrecordEditReorder0%
7530         gatherpartdetailsPrice0! = editrecordEditPrice0!
7540         GOSUB 5900
7550         editrecordEditDesc0$ = gatherpartdetailsDesc0$
7560         editrecordEditQty0% = gatherpartdetailsQty0%
7570         editrecordEditReorder0% = gatherpartdetailsReorder0%
7580         editrecordEditPrice0! = gatherpartdetailsPrice0!
7590         GOSUB 4920
7600         editrecordKp0$ = readkeyResult0$
7610         IF (editrecordKp0$ = "Y") <> 0 THEN GOTO 7640
7620         IF (editrecordKp0$ = "y") <> 0 THEN GOTO 7640
7630         GOTO 7490
7640     REM END DO
7650     ' inv[...] = { ... }  (whole-record write)
7660     LSET invFlagBuf$ = "1"
7670     LSET invDescBuf$ = editrecordEditDesc0$
7680     LSET invQtyBuf$ = MKI$(editrecordEditQty0%)
7690     LSET invReorderBuf$ = MKI$(editrecordEditReorder0%)
7700     LSET invPriceBuf$ = MKS$(editrecordEditPrice0!)
7710     PUT #1, editrecordPart0%
7720     RETURN
7730 ' end procedure editrecord

7740 ' procedure listall()
7750     ' global inv
7760     GOSUB 5650
7770     listallScrollCount0% = 0
7780     FOR listallI0% = 1 TO CONSTPARTCOUNT%
7790         ' let p = inv[...]  (whole-record read)
7800         GET #1, listallI0%
7810         rtrimSelf0$ = invFlagBuf$
7820         GOSUB 3170
7830         BCCT45$ = rtrimResult0$
7840         listallPFlag0$ = BCCT45$
7850         rtrimSelf0$ = invDescBuf$
7860         GOSUB 3170
7870         BCCT46$ = rtrimResult0$
7880         listallPDesc0$ = BCCT46$
7890         listallPQty0% = CVI(invQtyBuf$)
7900         listallPReorder0% = CVI(invReorderBuf$)
7910         listallPPrice0! = CVS(invPriceBuf$)
7920         printinventorylinePartNum0% = listallI0%
7930         printinventorylineDesc0$ = listallPDesc0$
7940         printinventorylineQty0% = listallPQty0%
7950         printinventorylineReorder0% = listallPReorder0%
7960         GOSUB 5720
7970         listallScrollCount0% = listallScrollCount0% + 1
7980         IF (listallScrollCount0% = 20) = 0 THEN GOTO 8070
7990             GOSUB 4990
8000             listallScrollCount0% = 0
8010             ' Redraw for the next page rather than let it keep scrolling
8020             ' past row 25 -- see printListHeader()'s own note on why a
8030             ' fixed-row prompt can't coexist with unbounded scrolling.
8040             IF (listallI0% < CONSTPARTCOUNT%) = 0 THEN GOTO 8060
8050                 GOSUB 5650
8060             REM END IF
8070         REM END IF
8080     NEXT listallI0%
8090     RETURN
8100 ' end procedure listall

8110 ' procedure addstock()
8120     ' global inv
8130     CLS
8140     LOCATE 5, 25
8150     PRINT "A D D I N G   S T O C K"
8160         LOCATE 8, 25
8170         GOSUB 4870
8180         addstockPartStr0$ = readpartnumberinputResult0$
8190         addstockPart0% = VAL(addstockPartStr0$)
8200         partinrangeN0% = addstockPart0%
8210         GOSUB 4780
8220         addstockValidPart0% = partinrangeResult0%
8230         IF (addstockValidPart0% = 0) = 0 THEN GOTO 8260
8240             GOSUB 5380
8250             GOSUB 4920
8260         REM END IF
8270         IF (addstockValidPart0% <> 0) = 0 THEN GOTO 8160
8280     REM END DO
8290     ' let p = inv[...]  (whole-record read)
8300     GET #1, addstockPart0%
8310     rtrimSelf0$ = invFlagBuf$
8320     GOSUB 3170
8330     BCCT51$ = rtrimResult0$
8340     addstockPFlag0$ = BCCT51$
8350     rtrimSelf0$ = invDescBuf$
8360     GOSUB 3170
8370     BCCT52$ = rtrimResult0$
8380     addstockPDesc0$ = BCCT52$
8390     addstockPQty0% = CVI(invQtyBuf$)
8400     addstockPReorder0% = CVI(invReorderBuf$)
8410     addstockPPrice0! = CVS(invPriceBuf$)
8420     isemptyFlag0$ = addstockPFlag0$
8430     GOSUB 4720
8440     BCCT53% = isemptyResult0%
8450     IF (BCCT53%) = 0 THEN GOTO 8500
8460         shownullentrymessagePartStr0$ = addstockPartStr0$
8470         GOSUB 5450
8480         GOSUB 4920
8490         RETURN
8500     REM END IF
8510         showaddstockscreenPartNum0% = addstockPart0%
8520         showaddstockscreenDesc0$ = addstockPDesc0$
8530         showaddstockscreenQty0% = addstockPQty0%
8540         showaddstockscreenReorder0% = addstockPReorder0%
8550         GOSUB 6100
8560         LOCATE 14, CONSTTABCOL%
8570         INPUT " Quantity to add"; addstockAddStr0$
8580         addstockAddAmt0% = VAL(addstockAddStr0$)
8590         IF (addstockAddAmt0% < 0) = 0 THEN GOTO 8620
8600             GOSUB 6260
8610             GOSUB 4920
8620         REM END IF
8630         IF (addstockAddAmt0% >= 0) = 0 THEN GOTO 8510
8640     REM END DO
8650     addstockPQty0% = addstockPQty0% + addstockAddAmt0%
8660     ' inv[...] = p  (write back a let-bound record)
8670     LSET invFlagBuf$ = addstockPFlag0$
8680     LSET invDescBuf$ = addstockPDesc0$
8690     LSET invQtyBuf$ = MKI$(addstockPQty0%)
8700     LSET invReorderBuf$ = MKI$(addstockPReorder0%)
8710     LSET invPriceBuf$ = MKS$(addstockPPrice0!)
8720     PUT #1, addstockPart0%
8730     RETURN
8740 ' end procedure addstock

8750 ' procedure subtractstock()
8760     ' global inv
8770     CLS
8780     LOCATE 5, 20
8790     PRINT "S U B T R A C T I N G    S T O C K"
8800         LOCATE 8, 25
8810         GOSUB 4870
8820         subtractstockPartStr0$ = readpartnumberinputResult0$
8830         subtractstockPart0% = VAL(subtractstockPartStr0$)
8840         partinrangeN0% = subtractstockPart0%
8850         GOSUB 4780
8860         subtractstockValidPart0% = partinrangeResult0%
8870         IF (subtractstockValidPart0% = 0) = 0 THEN GOTO 8900
8880             GOSUB 5380
8890             GOSUB 4920
8900         REM END IF
8910         IF (subtractstockValidPart0% <> 0) = 0 THEN GOTO 8800
8920     REM END DO
8930     ' let p = inv[...]  (whole-record read)
8940     GET #1, subtractstockPart0%
8950     rtrimSelf0$ = invFlagBuf$
8960     GOSUB 3170
8970     BCCT59$ = rtrimResult0$
8980     subtractstockPFlag0$ = BCCT59$
8990     rtrimSelf0$ = invDescBuf$
9000     GOSUB 3170
9010     BCCT60$ = rtrimResult0$
9020     subtractstockPDesc0$ = BCCT60$
9030     subtractstockPQty0% = CVI(invQtyBuf$)
9040     subtractstockPReorder0% = CVI(invReorderBuf$)
9050     subtractstockPPrice0! = CVS(invPriceBuf$)
9060     isemptyFlag0$ = subtractstockPFlag0$
9070     GOSUB 4720
9080     BCCT61% = isemptyResult0%
9090     IF (BCCT61%) = 0 THEN GOTO 9140
9100         shownullentrymessagePartStr0$ = subtractstockPartStr0$
9110         GOSUB 5450
9120         GOSUB 4920
9130         RETURN
9140     REM END IF
9150         showsubtractstockscreenPartNum0% = subtractstockPart0%
9160         showsubtractstockscreenDesc0$ = subtractstockPDesc0$
9170         showsubtractstockscreenQty0% = subtractstockPQty0%
9180         showsubtractstockscreenReorder0% = subtractstockPReorder0%
9190         GOSUB 6330
9200         LOCATE 14, CONSTTABCOL%
9210         INPUT "Quantity to subtract"; subtractstockSubStr0$
9220         subtractstockSubAmt0% = VAL(subtractstockSubStr0$)
9230         subtractstockOverSubtract0% = 0
9240         IF (subtractstockSubAmt0% >= 0) = 0 THEN GOTO 9300
9250         IF ((subtractstockPQty0% - subtractstockSubAmt0%) < 0) = 0 THEN GOTO 9300
9260             subtractstockOverSubtract0% = 1
9270             showoversubtractwarningOnHand0% = subtractstockPQty0%
9280             GOSUB 6490
9290             GOSUB 4920
9300         REM END IF
9310         IF (subtractstockSubAmt0% >= 0) = 0 THEN GOTO 9150
9320         IF (subtractstockOverSubtract0% = 0) = 0 THEN GOTO 9150
9330     REM END DO
9340     subtractstockPQty0% = subtractstockPQty0% - subtractstockSubAmt0%
9350     IF (subtractstockPQty0% <= subtractstockPReorder0%) = 0 THEN GOTO 9370
9360         LOCATE 16, CONSTTABCOL%
9370     REM END IF
9380     PRINT (("quantity now" + STR$(subtractstockPQty0%)) + " reorder level") + STR$(subtractstockPReorder0%)
9390     ' inv[...] = p  (write back a let-bound record)
9400     LSET invFlagBuf$ = subtractstockPFlag0$
9410     LSET invDescBuf$ = subtractstockPDesc0$
9420     LSET invQtyBuf$ = MKI$(subtractstockPQty0%)
9430     LSET invReorderBuf$ = MKI$(subtractstockPReorder0%)
9440     LSET invPriceBuf$ = MKS$(subtractstockPPrice0!)
9450     PUT #1, subtractstockPart0%
9460     RETURN
9470 ' end procedure subtractstock

9480 ' procedure reorderreport()
9490     ' global inv
9500     GOSUB 5760
9510     reorderreportReportLineCount0% = 0
9520     FOR reorderreportI0% = 1 TO CONSTPARTCOUNT%
9530         ' let p = inv[...]  (whole-record read)
9540         GET #1, reorderreportI0%
9550         rtrimSelf0$ = invFlagBuf$
9560         GOSUB 3170
9570         BCCT69$ = rtrimResult0$
9580         reorderreportPFlag0$ = BCCT69$
9590         rtrimSelf0$ = invDescBuf$
9600         GOSUB 3170
9610         BCCT70$ = rtrimResult0$
9620         reorderreportPDesc0$ = BCCT70$
9630         reorderreportPQty0% = CVI(invQtyBuf$)
9640         reorderreportPReorder0% = CVI(invReorderBuf$)
9650         reorderreportPPrice0! = CVS(invPriceBuf$)
9660         IF (reorderreportPQty0% < reorderreportPReorder0%) = 0 THEN GOTO 9840
9670             printreorderlinePartNum0% = reorderreportI0%
9680             printreorderlineDesc0$ = reorderreportPDesc0$
9690             printreorderlineQty0% = reorderreportPQty0%
9700             printreorderlineReorder0% = reorderreportPReorder0%
9710             GOSUB 5860
9720             reorderreportReportLineCount0% = reorderreportReportLineCount0% + 1
9730             IF (reorderreportReportLineCount0% > 15) = 0 THEN GOTO 9830
9740                 GOSUB 4990
9750                 reorderreportReportLineCount0% = 0
9760                 ' Redraw for the next page rather than let it keep
9770                 ' scrolling past row 25 -- see printListHeader()'s own
9780                 ' note (same underlying issue, same fix) on why a
9790                 ' fixed-row prompt can't coexist with unbounded scrolling.
9800                 IF (reorderreportI0% < CONSTPARTCOUNT%) = 0 THEN GOTO 9820
9810                     GOSUB 5760
9820                 REM END IF
9830             REM END IF
9840         REM END IF
9850     NEXT reorderreportI0%
9860     GOSUB 4990
9870     RETURN
9880 ' end procedure reorderreport

9890 ' procedure initializeinventoryfileifnew()
9900     ' global inv
9910     ' let p = inv[...]  (whole-record read)
9920     GET #1, 1
9930     rtrimSelf0$ = invFlagBuf$
9940     GOSUB 3170
9950     BCCT74$ = rtrimResult0$
9960     initializeinventoryfileifnewPFlag0$ = BCCT74$
9970     rtrimSelf0$ = invDescBuf$
9980     GOSUB 3170
9990     BCCT75$ = rtrimResult0$
10000     initializeinventoryfileifnewPDesc0$ = BCCT75$
10010     initializeinventoryfileifnewPQty0% = CVI(invQtyBuf$)
10020     initializeinventoryfileifnewPReorder0% = CVI(invReorderBuf$)
10030     initializeinventoryfileifnewPPrice0! = CVS(invPriceBuf$)
10040     IF (ASC(initializeinventoryfileifnewPFlag0$) = 0) = 0 THEN GOTO 10140
10050         FOR initializeinventoryfileifnewI0% = 1 TO CONSTPARTCOUNT%
10060             ' inv[...] = { ... }  (whole-record write)
10070             LSET invFlagBuf$ = CHR$(255)
10080             LSET invDescBuf$ = ""
10090             LSET invQtyBuf$ = MKI$(0)
10100             LSET invReorderBuf$ = MKI$(0)
10110             LSET invPriceBuf$ = MKS$(0)
10120             PUT #1, initializeinventoryfileifnewI0%
10130         NEXT initializeinventoryfileifnewI0%
10140     REM END IF
10150     RETURN
10160 ' end procedure initializeinventoryfileifnew

10170 ' procedure reportinventoryerror(err%, erl%)
10180     LOCATE 25, 1
10190     BCCT78% = reportinventoryerrorErl0%
10200     BCCT79$ = STR$(BCCT78%)
10210     errorCode0% = reportinventoryerrorErr0%
10220     GOSUB 3300
10230     BCCT80$ = errorResult0$
10240     PRINT (("There has been an error on line" + BCCT79$) + ": ") + BCCT80$
10250     GOSUB 4920
10260     reportinventoryerrorK0$ = readkeyResult0$
10270     RETURN
10280 ' end procedure reportinventoryerror
