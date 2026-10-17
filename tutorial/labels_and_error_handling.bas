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
190 CONSTERRSYNTAX% = 2
200 CONSTERRRETURNWITHOUTGOSUB% = 3
210 CONSTERROUTOFDATA% = 4
220 CONSTERRILLEGALFUNCTIONCALL% = 5
230 CONSTERROVERFLOW% = 6
240 CONSTERROUTOFMEMORY% = 7
250 CONSTERRSUBSCRIPTOUTOFRANGE% = 9
260 CONSTERRDUPLICATEDEFINITION% = 10
270 CONSTERRDIVISIONBYZERO% = 11
280 CONSTERRTYPEMISMATCH% = 13
290 CONSTERROUTOFSTRINGSPACE% = 14
300 CONSTERRNORESUME% = 19
310 CONSTERRRESUMEWITHOUTERROR% = 20
320 CONSTERRDEVICETIMEOUT% = 24
330 CONSTERRDEVICEFAULT% = 25
340 CONSTERROUTOFPAPER% = 27
350 CONSTERRBADFILENUMBER% = 52
360 CONSTERRFILENOTFOUND% = 53
370 CONSTERRBADFILEMODE% = 54
380 CONSTERRFILEALREADYOPEN% = 55
390 CONSTERRDEVICEIO% = 57
400 CONSTERRFILEALREADYEXISTS% = 58
410 CONSTERRDISKFULL% = 61
420 CONSTERRINPUTPASTEND% = 62
430 CONSTERRBADRECORDNUMBER% = 63
440 CONSTERRBADFILENAME% = 64
450 CONSTERRTOOMANYFILES% = 67
460 CONSTERRDEVICEUNAVAILABLE% = 68
470 CONSTERRDISKWRITEPROTECTED% = 70
480 CONSTERRDISKNOTREADY% = 71
490 CONSTERRDISKMEDIAERROR% = 72
500 CONSTERRPATHFILEACCESS% = 75
510 CONSTERRPATHNOTFOUND% = 76
520 ' Tutorial — Labels and Error Handling
530 ' 
540 ' BASCAL manages line numbers itself -- goto, gosub, on error goto, resume,
550 ' restore, and on ... goto / on ... gosub can never target a raw line
560 ' number in .bcl source. Every one of them requires a name: label instead;
570 ' the compiler assigns the real BASIC line number when it renders output,
580 ' the same way it already numbers the branch targets inside if/while/do/
590 ' select case.
600 ' 
610 ' on error goto 0 is the one numeric exception -- 0 isn't a line number,
620 ' it's the sentinel that disables the error trap.
630 ' ---- goto / label basics ----
640 PRINT "goto/label basics:"
650 GOTO 670
660 PRINT "  not reached"
670 PRINT "  reached via goto"
680 ' ---- portable procedure call (replaces BASIC-level GOSUB) ----
690 PRINT "procedure call:"
700 GOSUB 2400
710 PRINT "  back after gosub"
720 GOTO 730
730 ' ---- error handling: on error goto, resume to a label, err ----
740 ' 
750 ' Opening a file that doesn't exist raises BASIC runtime error 53
760 ' ("file not found"). The handler below catches it, prints a message, and
770 ' then RESUMEs at a label -- not the failing statement or "next", but a
780 ' specific point past the whole try/handler region. RESUME (not a plain
790 ' GOTO) is what clears the runtime's "currently handling an error" state,
800 ' so a later error can still be trapped.
810 PRINT "error handling, missing file:"
820 filename$ = "does_not_exist.dat"
830 ON ERROR GOTO 880
840 OPEN filename$ FOR INPUT AS #1
850 PRINT "  file opened (unexpected)"
860 CLOSE #1
870 GOTO 950
880 IF (ERR = CONSTERRFILENOTFOUND%) = 0 THEN GOTO 920
890     PRINT "  caught error "; ERR; ": "; filename$; " not found"
900     RESUME 950
910 GOTO 940
920     PRINT "  unexpected error "; ERR
930     ERROR ERR
940 REM END IF
950 ON ERROR GOTO 0
960 END

970 ' function error$(code%)
980     BCCT3% = errorCode0%
990     IF (BCCT3% = CONSTERRSYNTAX%) <> 0 THEN GOTO 1330
1000     IF (BCCT3% = CONSTERRRETURNWITHOUTGOSUB%) <> 0 THEN GOTO 1360
1010     IF (BCCT3% = CONSTERROUTOFDATA%) <> 0 THEN GOTO 1390
1020     IF (BCCT3% = CONSTERRILLEGALFUNCTIONCALL%) <> 0 THEN GOTO 1420
1030     IF (BCCT3% = CONSTERROVERFLOW%) <> 0 THEN GOTO 1450
1040     IF (BCCT3% = CONSTERROUTOFMEMORY%) <> 0 THEN GOTO 1480
1050     IF (BCCT3% = CONSTERRSUBSCRIPTOUTOFRANGE%) <> 0 THEN GOTO 1510
1060     IF (BCCT3% = CONSTERRDUPLICATEDEFINITION%) <> 0 THEN GOTO 1540
1070     IF (BCCT3% = CONSTERRDIVISIONBYZERO%) <> 0 THEN GOTO 1570
1080     IF (BCCT3% = CONSTERRTYPEMISMATCH%) <> 0 THEN GOTO 1600
1090     IF (BCCT3% = CONSTERROUTOFSTRINGSPACE%) <> 0 THEN GOTO 1630
1100     IF (BCCT3% = CONSTERRNORESUME%) <> 0 THEN GOTO 1660
1110     IF (BCCT3% = CONSTERRRESUMEWITHOUTERROR%) <> 0 THEN GOTO 1690
1120     IF (BCCT3% = CONSTERRDEVICETIMEOUT%) <> 0 THEN GOTO 1720
1130     IF (BCCT3% = CONSTERRDEVICEFAULT%) <> 0 THEN GOTO 1750
1140     IF (BCCT3% = CONSTERROUTOFPAPER%) <> 0 THEN GOTO 1780
1150     IF (BCCT3% = CONSTERRBADFILENUMBER%) <> 0 THEN GOTO 1810
1160     IF (BCCT3% = CONSTERRFILENOTFOUND%) <> 0 THEN GOTO 1840
1170     IF (BCCT3% = CONSTERRBADFILEMODE%) <> 0 THEN GOTO 1870
1180     IF (BCCT3% = CONSTERRFILEALREADYOPEN%) <> 0 THEN GOTO 1900
1190     IF (BCCT3% = CONSTERRDEVICEIO%) <> 0 THEN GOTO 1930
1200     IF (BCCT3% = CONSTERRFILEALREADYEXISTS%) <> 0 THEN GOTO 1960
1210     IF (BCCT3% = CONSTERRDISKFULL%) <> 0 THEN GOTO 1990
1220     IF (BCCT3% = CONSTERRINPUTPASTEND%) <> 0 THEN GOTO 2020
1230     IF (BCCT3% = CONSTERRBADRECORDNUMBER%) <> 0 THEN GOTO 2050
1240     IF (BCCT3% = CONSTERRBADFILENAME%) <> 0 THEN GOTO 2080
1250     IF (BCCT3% = CONSTERRTOOMANYFILES%) <> 0 THEN GOTO 2110
1260     IF (BCCT3% = CONSTERRDEVICEUNAVAILABLE%) <> 0 THEN GOTO 2140
1270     IF (BCCT3% = CONSTERRDISKWRITEPROTECTED%) <> 0 THEN GOTO 2170
1280     IF (BCCT3% = CONSTERRDISKNOTREADY%) <> 0 THEN GOTO 2200
1290     IF (BCCT3% = CONSTERRDISKMEDIAERROR%) <> 0 THEN GOTO 2230
1300     IF (BCCT3% = CONSTERRPATHFILEACCESS%) <> 0 THEN GOTO 2260
1310     IF (BCCT3% = CONSTERRPATHNOTFOUND%) <> 0 THEN GOTO 2290
1320     GOTO 2320
1330         errorResult0$ = "Syntax error"
1340         RETURN
1350         GOTO 2360
1360         errorResult0$ = "RETURN without GOSUB"
1370         RETURN
1380         GOTO 2360
1390         errorResult0$ = "Out of DATA"
1400         RETURN
1410         GOTO 2360
1420         errorResult0$ = "Illegal function call"
1430         RETURN
1440         GOTO 2360
1450         errorResult0$ = "Overflow"
1460         RETURN
1470         GOTO 2360
1480         errorResult0$ = "Out of memory"
1490         RETURN
1500         GOTO 2360
1510         errorResult0$ = "Subscript out of range"
1520         RETURN
1530         GOTO 2360
1540         errorResult0$ = "Duplicate Definition"
1550         RETURN
1560         GOTO 2360
1570         errorResult0$ = "Division by zero"
1580         RETURN
1590         GOTO 2360
1600         errorResult0$ = "Type mismatch"
1610         RETURN
1620         GOTO 2360
1630         errorResult0$ = "Out of string space"
1640         RETURN
1650         GOTO 2360
1660         errorResult0$ = "No RESUME"
1670         RETURN
1680         GOTO 2360
1690         errorResult0$ = "RESUME without error"
1700         RETURN
1710         GOTO 2360
1720         errorResult0$ = "Device timeout"
1730         RETURN
1740         GOTO 2360
1750         errorResult0$ = "Device fault"
1760         RETURN
1770         GOTO 2360
1780         errorResult0$ = "Out of paper"
1790         RETURN
1800         GOTO 2360
1810         errorResult0$ = "Bad file number"
1820         RETURN
1830         GOTO 2360
1840         errorResult0$ = "File not found"
1850         RETURN
1860         GOTO 2360
1870         errorResult0$ = "Bad file mode"
1880         RETURN
1890         GOTO 2360
1900         errorResult0$ = "File already open"
1910         RETURN
1920         GOTO 2360
1930         errorResult0$ = "Device I/O error"
1940         RETURN
1950         GOTO 2360
1960         errorResult0$ = "File already exists"
1970         RETURN
1980         GOTO 2360
1990         errorResult0$ = "Disk full"
2000         RETURN
2010         GOTO 2360
2020         errorResult0$ = "Input past end"
2030         RETURN
2040         GOTO 2360
2050         errorResult0$ = "Bad record number"
2060         RETURN
2070         GOTO 2360
2080         errorResult0$ = "Bad file name"
2090         RETURN
2100         GOTO 2360
2110         errorResult0$ = "Too many files"
2120         RETURN
2130         GOTO 2360
2140         errorResult0$ = "Device unavailable"
2150         RETURN
2160         GOTO 2360
2170         errorResult0$ = "Disk write protected"
2180         RETURN
2190         GOTO 2360
2200         errorResult0$ = "Disk not ready"
2210         RETURN
2220         GOTO 2360
2230         errorResult0$ = "Disk media error"
2240         RETURN
2250         GOTO 2360
2260         errorResult0$ = "Path/File access error"
2270         RETURN
2280         GOTO 2360
2290         errorResult0$ = "Path not found"
2300         RETURN
2310         GOTO 2360
2320         BCCT4% = errorCode0%
2330         BCCT5$ = STR$(BCCT4%)
2340         errorResult0$ = "Error " + BCCT5$
2350         RETURN
2360     REM END SELECT
2370     RETURN
2380 ' end function error$

2390 ' procedure printbanner()
2400     PRINT "  inside the gosub'd subroutine"
2410     RETURN
2420 ' end procedure printbanner
