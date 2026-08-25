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
700 GOSUB 2380
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

880 IF (ERR = errfilenotfound%) = 0 THEN GOTO 920
890     PRINT "  caught error "; ERR; ": "; filename$; " not found"
900     RESUME 950
910     GOTO 940
920     PRINT "  unexpected error "; ERR
930     ERROR ERR
940 REM END IF

950 ON ERROR GOTO 0

960 END

970 ' function error$(code%)
980     BCCT3% = errorCode0%
990     IF (BCCT3% = errsyntax%) <> 0 THEN GOTO 1330
1000     IF (BCCT3% = errreturnwithoutgosub%) <> 0 THEN GOTO 1360
1010     IF (BCCT3% = erroutofdata%) <> 0 THEN GOTO 1390
1020     IF (BCCT3% = errillegalfunctioncall%) <> 0 THEN GOTO 1420
1030     IF (BCCT3% = erroverflow%) <> 0 THEN GOTO 1450
1040     IF (BCCT3% = erroutofmemory%) <> 0 THEN GOTO 1480
1050     IF (BCCT3% = errsubscriptoutofrange%) <> 0 THEN GOTO 1510
1060     IF (BCCT3% = errduplicatedefinition%) <> 0 THEN GOTO 1540
1070     IF (BCCT3% = errdivisionbyzero%) <> 0 THEN GOTO 1570
1080     IF (BCCT3% = errtypemismatch%) <> 0 THEN GOTO 1600
1090     IF (BCCT3% = erroutofstringspace%) <> 0 THEN GOTO 1630
1100     IF (BCCT3% = errnoresume%) <> 0 THEN GOTO 1660
1110     IF (BCCT3% = errresumewithouterror%) <> 0 THEN GOTO 1690
1120     IF (BCCT3% = errdevicetimeout%) <> 0 THEN GOTO 1720
1130     IF (BCCT3% = errdevicefault%) <> 0 THEN GOTO 1750
1140     IF (BCCT3% = erroutofpaper%) <> 0 THEN GOTO 1780
1150     IF (BCCT3% = errbadfilenumber%) <> 0 THEN GOTO 1810
1160     IF (BCCT3% = errfilenotfound%) <> 0 THEN GOTO 1840
1170     IF (BCCT3% = errbadfilemode%) <> 0 THEN GOTO 1870
1180     IF (BCCT3% = errfilealreadyopen%) <> 0 THEN GOTO 1900
1190     IF (BCCT3% = errdeviceio%) <> 0 THEN GOTO 1930
1200     IF (BCCT3% = errfilealreadyexists%) <> 0 THEN GOTO 1960
1210     IF (BCCT3% = errdiskfull%) <> 0 THEN GOTO 1990
1220     IF (BCCT3% = errinputpastend%) <> 0 THEN GOTO 2020
1230     IF (BCCT3% = errbadrecordnumber%) <> 0 THEN GOTO 2050
1240     IF (BCCT3% = errbadfilename%) <> 0 THEN GOTO 2080
1250     IF (BCCT3% = errtoomanyfiles%) <> 0 THEN GOTO 2110
1260     IF (BCCT3% = errdeviceunavailable%) <> 0 THEN GOTO 2140
1270     IF (BCCT3% = errdiskwriteprotected%) <> 0 THEN GOTO 2170
1280     IF (BCCT3% = errdisknotready%) <> 0 THEN GOTO 2200
1290     IF (BCCT3% = errdiskmediaerror%) <> 0 THEN GOTO 2230
1300     IF (BCCT3% = errpathfileaccess%) <> 0 THEN GOTO 2260
1310     IF (BCCT3% = errpathnotfound%) <> 0 THEN GOTO 2290
1320     GOTO 2320
1330         errorResult0$ = "Syntax error"
1340         RETURN
1350         GOTO 2340
1360         errorResult0$ = "RETURN without GOSUB"
1370         RETURN
1380         GOTO 2340
1390         errorResult0$ = "Out of DATA"
1400         RETURN
1410         GOTO 2340
1420         errorResult0$ = "Illegal function call"
1430         RETURN
1440         GOTO 2340
1450         errorResult0$ = "Overflow"
1460         RETURN
1470         GOTO 2340
1480         errorResult0$ = "Out of memory"
1490         RETURN
1500         GOTO 2340
1510         errorResult0$ = "Subscript out of range"
1520         RETURN
1530         GOTO 2340
1540         errorResult0$ = "Duplicate Definition"
1550         RETURN
1560         GOTO 2340
1570         errorResult0$ = "Division by zero"
1580         RETURN
1590         GOTO 2340
1600         errorResult0$ = "Type mismatch"
1610         RETURN
1620         GOTO 2340
1630         errorResult0$ = "Out of string space"
1640         RETURN
1650         GOTO 2340
1660         errorResult0$ = "No RESUME"
1670         RETURN
1680         GOTO 2340
1690         errorResult0$ = "RESUME without error"
1700         RETURN
1710         GOTO 2340
1720         errorResult0$ = "Device timeout"
1730         RETURN
1740         GOTO 2340
1750         errorResult0$ = "Device fault"
1760         RETURN
1770         GOTO 2340
1780         errorResult0$ = "Out of paper"
1790         RETURN
1800         GOTO 2340
1810         errorResult0$ = "Bad file number"
1820         RETURN
1830         GOTO 2340
1840         errorResult0$ = "File not found"
1850         RETURN
1860         GOTO 2340
1870         errorResult0$ = "Bad file mode"
1880         RETURN
1890         GOTO 2340
1900         errorResult0$ = "File already open"
1910         RETURN
1920         GOTO 2340
1930         errorResult0$ = "Device I/O error"
1940         RETURN
1950         GOTO 2340
1960         errorResult0$ = "File already exists"
1970         RETURN
1980         GOTO 2340
1990         errorResult0$ = "Disk full"
2000         RETURN
2010         GOTO 2340
2020         errorResult0$ = "Input past end"
2030         RETURN
2040         GOTO 2340
2050         errorResult0$ = "Bad record number"
2060         RETURN
2070         GOTO 2340
2080         errorResult0$ = "Bad file name"
2090         RETURN
2100         GOTO 2340
2110         errorResult0$ = "Too many files"
2120         RETURN
2130         GOTO 2340
2140         errorResult0$ = "Device unavailable"
2150         RETURN
2160         GOTO 2340
2170         errorResult0$ = "Disk write protected"
2180         RETURN
2190         GOTO 2340
2200         errorResult0$ = "Disk not ready"
2210         RETURN
2220         GOTO 2340
2230         errorResult0$ = "Disk media error"
2240         RETURN
2250         GOTO 2340
2260         errorResult0$ = "Path/File access error"
2270         RETURN
2280         GOTO 2340
2290         errorResult0$ = "Path not found"
2300         RETURN
2310         GOTO 2340
2320         errorResult0$ = "Error " + STR$(errorCode0%)
2330         RETURN
2340     REM END SELECT
2350     RETURN
2360 ' end function error$

2370 ' procedure printbanner()
2380     PRINT "  inside the gosub'd subroutine"
2390     RETURN
2400 ' end procedure printbanner
