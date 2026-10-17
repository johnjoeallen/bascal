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
520 ' Tutorial — Portable Structured Error Handling
530 ' 
540 ' TRY/CATCH/FINALLY and THROW are BASCAL's portable error model.  A catch can
550 ' select several error codes and bind the originating source file.
560 PRINT "portable try/catch:"
570 ON ERROR GOTO 620
580 BCCTRY0001PENDING% = 0
590 ERROR CONSTERRFILENOTFOUND%
600 ON ERROR GOTO 0
610 GOTO 770
620 BCCTRY0001PENDING% = ERR
630 IF (ERR = CONSTERRFILENOTFOUND%) OR (ERR = CONSTERRFILEALREADYOPEN%) THEN GOTO 650
640 RESUME 770
650 BCCERR% = ERR
660 BCCERL% = ERL
670 GOSUB 2250
680 source$ = BCCSOURCEFILE$
690 RESUME 700
700 ON ERROR GOTO 750
710 PRINT "  caught error "; BCCERR%; " at "; source$; ":"; BCCERL%
720 BCCTRY0001PENDING% = 0
730 ON ERROR GOTO 0
740 GOTO 770
750 BCCTRY0001PENDING% = ERR
760 RESUME 770
770 ON ERROR GOTO 0
780 PRINT "  cleanup always runs"
790 IF BCCTRY0001PENDING% <> 0 THEN ERROR BCCTRY0001PENDING%
800 REM END TRY
810 END

820 ' function error$(code%)
830     BCCT3% = errorCode0%
840     IF (BCCT3% = CONSTERRSYNTAX%) <> 0 THEN GOTO 1180
850     IF (BCCT3% = CONSTERRRETURNWITHOUTGOSUB%) <> 0 THEN GOTO 1210
860     IF (BCCT3% = CONSTERROUTOFDATA%) <> 0 THEN GOTO 1240
870     IF (BCCT3% = CONSTERRILLEGALFUNCTIONCALL%) <> 0 THEN GOTO 1270
880     IF (BCCT3% = CONSTERROVERFLOW%) <> 0 THEN GOTO 1300
890     IF (BCCT3% = CONSTERROUTOFMEMORY%) <> 0 THEN GOTO 1330
900     IF (BCCT3% = CONSTERRSUBSCRIPTOUTOFRANGE%) <> 0 THEN GOTO 1360
910     IF (BCCT3% = CONSTERRDUPLICATEDEFINITION%) <> 0 THEN GOTO 1390
920     IF (BCCT3% = CONSTERRDIVISIONBYZERO%) <> 0 THEN GOTO 1420
930     IF (BCCT3% = CONSTERRTYPEMISMATCH%) <> 0 THEN GOTO 1450
940     IF (BCCT3% = CONSTERROUTOFSTRINGSPACE%) <> 0 THEN GOTO 1480
950     IF (BCCT3% = CONSTERRNORESUME%) <> 0 THEN GOTO 1510
960     IF (BCCT3% = CONSTERRRESUMEWITHOUTERROR%) <> 0 THEN GOTO 1540
970     IF (BCCT3% = CONSTERRDEVICETIMEOUT%) <> 0 THEN GOTO 1570
980     IF (BCCT3% = CONSTERRDEVICEFAULT%) <> 0 THEN GOTO 1600
990     IF (BCCT3% = CONSTERROUTOFPAPER%) <> 0 THEN GOTO 1630
1000     IF (BCCT3% = CONSTERRBADFILENUMBER%) <> 0 THEN GOTO 1660
1010     IF (BCCT3% = CONSTERRFILENOTFOUND%) <> 0 THEN GOTO 1690
1020     IF (BCCT3% = CONSTERRBADFILEMODE%) <> 0 THEN GOTO 1720
1030     IF (BCCT3% = CONSTERRFILEALREADYOPEN%) <> 0 THEN GOTO 1750
1040     IF (BCCT3% = CONSTERRDEVICEIO%) <> 0 THEN GOTO 1780
1050     IF (BCCT3% = CONSTERRFILEALREADYEXISTS%) <> 0 THEN GOTO 1810
1060     IF (BCCT3% = CONSTERRDISKFULL%) <> 0 THEN GOTO 1840
1070     IF (BCCT3% = CONSTERRINPUTPASTEND%) <> 0 THEN GOTO 1870
1080     IF (BCCT3% = CONSTERRBADRECORDNUMBER%) <> 0 THEN GOTO 1900
1090     IF (BCCT3% = CONSTERRBADFILENAME%) <> 0 THEN GOTO 1930
1100     IF (BCCT3% = CONSTERRTOOMANYFILES%) <> 0 THEN GOTO 1960
1110     IF (BCCT3% = CONSTERRDEVICEUNAVAILABLE%) <> 0 THEN GOTO 1990
1120     IF (BCCT3% = CONSTERRDISKWRITEPROTECTED%) <> 0 THEN GOTO 2020
1130     IF (BCCT3% = CONSTERRDISKNOTREADY%) <> 0 THEN GOTO 2050
1140     IF (BCCT3% = CONSTERRDISKMEDIAERROR%) <> 0 THEN GOTO 2080
1150     IF (BCCT3% = CONSTERRPATHFILEACCESS%) <> 0 THEN GOTO 2110
1160     IF (BCCT3% = CONSTERRPATHNOTFOUND%) <> 0 THEN GOTO 2140
1170     GOTO 2170
1180         errorResult0$ = "Syntax error"
1190         RETURN
1200         GOTO 2210
1210         errorResult0$ = "RETURN without GOSUB"
1220         RETURN
1230         GOTO 2210
1240         errorResult0$ = "Out of DATA"
1250         RETURN
1260         GOTO 2210
1270         errorResult0$ = "Illegal function call"
1280         RETURN
1290         GOTO 2210
1300         errorResult0$ = "Overflow"
1310         RETURN
1320         GOTO 2210
1330         errorResult0$ = "Out of memory"
1340         RETURN
1350         GOTO 2210
1360         errorResult0$ = "Subscript out of range"
1370         RETURN
1380         GOTO 2210
1390         errorResult0$ = "Duplicate Definition"
1400         RETURN
1410         GOTO 2210
1420         errorResult0$ = "Division by zero"
1430         RETURN
1440         GOTO 2210
1450         errorResult0$ = "Type mismatch"
1460         RETURN
1470         GOTO 2210
1480         errorResult0$ = "Out of string space"
1490         RETURN
1500         GOTO 2210
1510         errorResult0$ = "No RESUME"
1520         RETURN
1530         GOTO 2210
1540         errorResult0$ = "RESUME without error"
1550         RETURN
1560         GOTO 2210
1570         errorResult0$ = "Device timeout"
1580         RETURN
1590         GOTO 2210
1600         errorResult0$ = "Device fault"
1610         RETURN
1620         GOTO 2210
1630         errorResult0$ = "Out of paper"
1640         RETURN
1650         GOTO 2210
1660         errorResult0$ = "Bad file number"
1670         RETURN
1680         GOTO 2210
1690         errorResult0$ = "File not found"
1700         RETURN
1710         GOTO 2210
1720         errorResult0$ = "Bad file mode"
1730         RETURN
1740         GOTO 2210
1750         errorResult0$ = "File already open"
1760         RETURN
1770         GOTO 2210
1780         errorResult0$ = "Device I/O error"
1790         RETURN
1800         GOTO 2210
1810         errorResult0$ = "File already exists"
1820         RETURN
1830         GOTO 2210
1840         errorResult0$ = "Disk full"
1850         RETURN
1860         GOTO 2210
1870         errorResult0$ = "Input past end"
1880         RETURN
1890         GOTO 2210
1900         errorResult0$ = "Bad record number"
1910         RETURN
1920         GOTO 2210
1930         errorResult0$ = "Bad file name"
1940         RETURN
1950         GOTO 2210
1960         errorResult0$ = "Too many files"
1970         RETURN
1980         GOTO 2210
1990         errorResult0$ = "Device unavailable"
2000         RETURN
2010         GOTO 2210
2020         errorResult0$ = "Disk write protected"
2030         RETURN
2040         GOTO 2210
2050         errorResult0$ = "Disk not ready"
2060         RETURN
2070         GOTO 2210
2080         errorResult0$ = "Disk media error"
2090         RETURN
2100         GOTO 2210
2110         errorResult0$ = "Path/File access error"
2120         RETURN
2130         GOTO 2210
2140         errorResult0$ = "Path not found"
2150         RETURN
2160         GOTO 2210
2170         BCCT4% = errorCode0%
2180         BCCT5$ = STR$(BCCT4%)
2190         errorResult0$ = "Error " + BCCT5$
2200         RETURN
2210     REM END SELECT
2220     RETURN
2230 ' end function error$

2240 ' catch's optional source$ binding: map ERL back to its original .bcl file
2250     IF ERL <= 510 THEN BCCSOURCEFILE$ = "com/bascal/stdlib/error.bcl" : RETURN
2260     IF ERL <= 820 THEN BCCSOURCEFILE$ = "tutorial/portable_error_handling.bcl" : RETURN
2270     BCCSOURCEFILE$ = "com/bascal/stdlib/error.bcl"
2280     RETURN
