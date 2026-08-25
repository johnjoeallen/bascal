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

520 ' Tutorial — Portable Structured Error Handling
530 ' 
540 ' TRY/CATCH/FINALLY and THROW are BASCAL's portable error model.  A catch can
550 ' select several error codes and bind the originating source file.

560 PRINT "portable try/catch:"
570 ON ERROR GOTO 620
580 BCC_TRY_0001_PENDING% = 0
590     ERROR errfilenotfound%
600 ON ERROR GOTO 0
610 GOTO 770
620     BCC_TRY_0001_PENDING% = ERR
630     IF (ERR = errfilenotfound%) OR (ERR = errfilealreadyopen%) THEN GOTO 650
640     RESUME 770
650     err% = ERR
660     erl% = ERL
670     GOSUB 2230
680     source$ = BCC_SOURCE_FILE$
690     RESUME 700
700 ON ERROR GOTO 750
710     PRINT "  caught error "; err%; " at "; source$; ":"; erl%
720     BCC_TRY_0001_PENDING% = 0
730     ON ERROR GOTO 0
740     GOTO 770
750     BCC_TRY_0001_PENDING% = ERR
760     RESUME 770
770 ON ERROR GOTO 0
780     PRINT "  cleanup always runs"
790     IF BCC_TRY_0001_PENDING% <> 0 THEN ERROR BCC_TRY_0001_PENDING%
800 REM END TRY

810 END

820 ' function error$(code%)
830     BCCT3% = errorCode0%
840     IF (BCCT3% = errsyntax%) <> 0 THEN GOTO 1180
850     IF (BCCT3% = errreturnwithoutgosub%) <> 0 THEN GOTO 1210
860     IF (BCCT3% = erroutofdata%) <> 0 THEN GOTO 1240
870     IF (BCCT3% = errillegalfunctioncall%) <> 0 THEN GOTO 1270
880     IF (BCCT3% = erroverflow%) <> 0 THEN GOTO 1300
890     IF (BCCT3% = erroutofmemory%) <> 0 THEN GOTO 1330
900     IF (BCCT3% = errsubscriptoutofrange%) <> 0 THEN GOTO 1360
910     IF (BCCT3% = errduplicatedefinition%) <> 0 THEN GOTO 1390
920     IF (BCCT3% = errdivisionbyzero%) <> 0 THEN GOTO 1420
930     IF (BCCT3% = errtypemismatch%) <> 0 THEN GOTO 1450
940     IF (BCCT3% = erroutofstringspace%) <> 0 THEN GOTO 1480
950     IF (BCCT3% = errnoresume%) <> 0 THEN GOTO 1510
960     IF (BCCT3% = errresumewithouterror%) <> 0 THEN GOTO 1540
970     IF (BCCT3% = errdevicetimeout%) <> 0 THEN GOTO 1570
980     IF (BCCT3% = errdevicefault%) <> 0 THEN GOTO 1600
990     IF (BCCT3% = erroutofpaper%) <> 0 THEN GOTO 1630
1000     IF (BCCT3% = errbadfilenumber%) <> 0 THEN GOTO 1660
1010     IF (BCCT3% = errfilenotfound%) <> 0 THEN GOTO 1690
1020     IF (BCCT3% = errbadfilemode%) <> 0 THEN GOTO 1720
1030     IF (BCCT3% = errfilealreadyopen%) <> 0 THEN GOTO 1750
1040     IF (BCCT3% = errdeviceio%) <> 0 THEN GOTO 1780
1050     IF (BCCT3% = errfilealreadyexists%) <> 0 THEN GOTO 1810
1060     IF (BCCT3% = errdiskfull%) <> 0 THEN GOTO 1840
1070     IF (BCCT3% = errinputpastend%) <> 0 THEN GOTO 1870
1080     IF (BCCT3% = errbadrecordnumber%) <> 0 THEN GOTO 1900
1090     IF (BCCT3% = errbadfilename%) <> 0 THEN GOTO 1930
1100     IF (BCCT3% = errtoomanyfiles%) <> 0 THEN GOTO 1960
1110     IF (BCCT3% = errdeviceunavailable%) <> 0 THEN GOTO 1990
1120     IF (BCCT3% = errdiskwriteprotected%) <> 0 THEN GOTO 2020
1130     IF (BCCT3% = errdisknotready%) <> 0 THEN GOTO 2050
1140     IF (BCCT3% = errdiskmediaerror%) <> 0 THEN GOTO 2080
1150     IF (BCCT3% = errpathfileaccess%) <> 0 THEN GOTO 2110
1160     IF (BCCT3% = errpathnotfound%) <> 0 THEN GOTO 2140
1170     GOTO 2170
1180         errorResult0$ = "Syntax error"
1190         RETURN
1200         GOTO 2190
1210         errorResult0$ = "RETURN without GOSUB"
1220         RETURN
1230         GOTO 2190
1240         errorResult0$ = "Out of DATA"
1250         RETURN
1260         GOTO 2190
1270         errorResult0$ = "Illegal function call"
1280         RETURN
1290         GOTO 2190
1300         errorResult0$ = "Overflow"
1310         RETURN
1320         GOTO 2190
1330         errorResult0$ = "Out of memory"
1340         RETURN
1350         GOTO 2190
1360         errorResult0$ = "Subscript out of range"
1370         RETURN
1380         GOTO 2190
1390         errorResult0$ = "Duplicate Definition"
1400         RETURN
1410         GOTO 2190
1420         errorResult0$ = "Division by zero"
1430         RETURN
1440         GOTO 2190
1450         errorResult0$ = "Type mismatch"
1460         RETURN
1470         GOTO 2190
1480         errorResult0$ = "Out of string space"
1490         RETURN
1500         GOTO 2190
1510         errorResult0$ = "No RESUME"
1520         RETURN
1530         GOTO 2190
1540         errorResult0$ = "RESUME without error"
1550         RETURN
1560         GOTO 2190
1570         errorResult0$ = "Device timeout"
1580         RETURN
1590         GOTO 2190
1600         errorResult0$ = "Device fault"
1610         RETURN
1620         GOTO 2190
1630         errorResult0$ = "Out of paper"
1640         RETURN
1650         GOTO 2190
1660         errorResult0$ = "Bad file number"
1670         RETURN
1680         GOTO 2190
1690         errorResult0$ = "File not found"
1700         RETURN
1710         GOTO 2190
1720         errorResult0$ = "Bad file mode"
1730         RETURN
1740         GOTO 2190
1750         errorResult0$ = "File already open"
1760         RETURN
1770         GOTO 2190
1780         errorResult0$ = "Device I/O error"
1790         RETURN
1800         GOTO 2190
1810         errorResult0$ = "File already exists"
1820         RETURN
1830         GOTO 2190
1840         errorResult0$ = "Disk full"
1850         RETURN
1860         GOTO 2190
1870         errorResult0$ = "Input past end"
1880         RETURN
1890         GOTO 2190
1900         errorResult0$ = "Bad record number"
1910         RETURN
1920         GOTO 2190
1930         errorResult0$ = "Bad file name"
1940         RETURN
1950         GOTO 2190
1960         errorResult0$ = "Too many files"
1970         RETURN
1980         GOTO 2190
1990         errorResult0$ = "Device unavailable"
2000         RETURN
2010         GOTO 2190
2020         errorResult0$ = "Disk write protected"
2030         RETURN
2040         GOTO 2190
2050         errorResult0$ = "Disk not ready"
2060         RETURN
2070         GOTO 2190
2080         errorResult0$ = "Disk media error"
2090         RETURN
2100         GOTO 2190
2110         errorResult0$ = "Path/File access error"
2120         RETURN
2130         GOTO 2190
2140         errorResult0$ = "Path not found"
2150         RETURN
2160         GOTO 2190
2170         errorResult0$ = "Error " + STR$(errorCode0%)
2180         RETURN
2190     REM END SELECT
2200     RETURN
2210 ' end function error$

2220 ' catch's optional source$ binding: map ERL back to its original .bcl file
2230     IF ERL <= 510 THEN BCC_SOURCE_FILE$ = "com/bascal/stdlib/error.bcl" : RETURN
2240     IF ERL <= 820 THEN BCC_SOURCE_FILE$ = "tutorial/21_portable_error_handling.bcl" : RETURN
2250     BCC_SOURCE_FILE$ = "com/bascal/stdlib/error.bcl"
2260     RETURN
