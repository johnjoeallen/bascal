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

680 ' ---- gosub / return (BASIC-level subroutine, distinct from BASCAL functions) ----

690 PRINT "gosub/return:"
700 GOSUB 730
710 PRINT "  back after gosub"
720 GOTO 750

730 PRINT "  inside the gosub'd subroutine"
740 RETURN

750 ' ---- error handling: on error goto, resume to a label, err ----
760 ' 
770 ' Opening a file that doesn't exist raises BASIC runtime error 53
780 ' ("file not found"). The handler below catches it, prints a message, and
790 ' then RESUMEs at a label -- not the failing statement or "next", but a
800 ' specific point past the whole try/handler region. RESUME (not a plain
810 ' GOTO) is what clears the runtime's "currently handling an error" state,
820 ' so a later error can still be trapped.

830 PRINT "error handling, missing file:"
840 filename$ = "does_not_exist.dat"
850 ON ERROR GOTO 900
860 OPEN filename$ FOR INPUT AS #1
870 PRINT "  file opened (unexpected)"
880 CLOSE #1
890 GOTO 970

900 IF (ERR = errfilenotfound%) = 0 THEN GOTO 940
910     PRINT "  caught error "; ERR; ": "; filename$; " not found"
920     RESUME 970
930     GOTO 960
940     PRINT "  unexpected error "; ERR
950     ERROR ERR
960 REM END IF

970 ON ERROR GOTO 0

980 ' ---- try/catch: portable structured error recovery, and throw to rethrow ----
990 ' 
1000 ' try/catch (issue #60) is BASCAL's structured alternative to on error
1010 ' goto/resume above -- it transpiles unchanged under both --target basic
1020 ' and --target C. A failed statement anywhere in the try body abandons
1030 ' the rest of it and runs catch once, then execution always continues
1040 ' right after end try -- never back inside try, and with no resume
1050 ' equivalent at all. err%/erl%/source$ are ordinary locals scoped to the
1060 ' catch block, not aliases for the ambient err/erl on error goto above.
1070 ' 
1080 ' The filter means this catch handles only errFileNotFound%; every other
1090 ' error automatically rethrows after the try block.

1100 PRINT "try/catch, missing file, with rethrow:"
1110 filename$ = "also_missing.dat"
1120 ON ERROR GOTO 1190
1130 BCC_TRY_0002_PENDING% = 0
1140     OPEN filename$ FOR INPUT AS #2
1150     PRINT "  file opened (unexpected)"
1160     CLOSE #2
1170 ON ERROR GOTO 0
1180 GOTO 1350
1190     BCC_TRY_0002_PENDING% = ERR
1200     IF (ERR = errfilenotfound%) THEN GOTO 1220
1210     RESUME 1350
1220     err% = ERR
1230     erl% = ERL
1240     GOSUB 2900
1250     source$ = BCC_SOURCE_FILE$
1260     RESUME 1270
1270 ON ERROR GOTO 1330
1280     PRINT "  caught error "; err%; " at "; source$; ":"; erl%
1290     PRINT "  "; filename$; " not found"
1300     BCC_TRY_0002_PENDING% = 0
1310     ON ERROR GOTO 0
1320     GOTO 1350
1330     BCC_TRY_0002_PENDING% = ERR
1340     RESUME 1350
1350 ON ERROR GOTO 0
1360     IF BCC_TRY_0002_PENDING% <> 0 THEN ERROR BCC_TRY_0002_PENDING%
1370 REM END TRY

1380 ' ---- restore with a label: rewind the DATA pointer to a specific block ----

1390 PRINT "restore to a label:"
1400 READ firstcountry$
1410 PRINT "  first read: "; firstcountry$
1420 RESTORE 1470
1430 READ secondcountry$
1440 PRINT "  after restore secondBatch: "; secondcountry$

1450 END

1460 DATA "France"

1470 DATA "Japan"
1480 END

1490 ' function error$(code%)
1500     BCCT4% = errorCode0%
1510     IF (BCCT4% = 2) <> 0 THEN GOTO 1850
1520     IF (BCCT4% = 3) <> 0 THEN GOTO 1880
1530     IF (BCCT4% = 4) <> 0 THEN GOTO 1910
1540     IF (BCCT4% = 5) <> 0 THEN GOTO 1940
1550     IF (BCCT4% = 6) <> 0 THEN GOTO 1970
1560     IF (BCCT4% = 7) <> 0 THEN GOTO 2000
1570     IF (BCCT4% = 9) <> 0 THEN GOTO 2030
1580     IF (BCCT4% = 10) <> 0 THEN GOTO 2060
1590     IF (BCCT4% = 11) <> 0 THEN GOTO 2090
1600     IF (BCCT4% = 13) <> 0 THEN GOTO 2120
1610     IF (BCCT4% = 14) <> 0 THEN GOTO 2150
1620     IF (BCCT4% = 19) <> 0 THEN GOTO 2180
1630     IF (BCCT4% = 20) <> 0 THEN GOTO 2210
1640     IF (BCCT4% = 24) <> 0 THEN GOTO 2240
1650     IF (BCCT4% = 25) <> 0 THEN GOTO 2270
1660     IF (BCCT4% = 27) <> 0 THEN GOTO 2300
1670     IF (BCCT4% = 52) <> 0 THEN GOTO 2330
1680     IF (BCCT4% = 53) <> 0 THEN GOTO 2360
1690     IF (BCCT4% = 54) <> 0 THEN GOTO 2390
1700     IF (BCCT4% = 55) <> 0 THEN GOTO 2420
1710     IF (BCCT4% = 57) <> 0 THEN GOTO 2450
1720     IF (BCCT4% = 58) <> 0 THEN GOTO 2480
1730     IF (BCCT4% = 61) <> 0 THEN GOTO 2510
1740     IF (BCCT4% = 62) <> 0 THEN GOTO 2540
1750     IF (BCCT4% = 63) <> 0 THEN GOTO 2570
1760     IF (BCCT4% = 64) <> 0 THEN GOTO 2600
1770     IF (BCCT4% = 67) <> 0 THEN GOTO 2630
1780     IF (BCCT4% = 68) <> 0 THEN GOTO 2660
1790     IF (BCCT4% = 70) <> 0 THEN GOTO 2690
1800     IF (BCCT4% = 71) <> 0 THEN GOTO 2720
1810     IF (BCCT4% = 72) <> 0 THEN GOTO 2750
1820     IF (BCCT4% = 75) <> 0 THEN GOTO 2780
1830     IF (BCCT4% = 76) <> 0 THEN GOTO 2810
1840     GOTO 2840
1850         errorResult0$ = "Syntax error"
1860         RETURN
1870         GOTO 2860
1880         errorResult0$ = "RETURN without GOSUB"
1890         RETURN
1900         GOTO 2860
1910         errorResult0$ = "Out of DATA"
1920         RETURN
1930         GOTO 2860
1940         errorResult0$ = "Illegal function call"
1950         RETURN
1960         GOTO 2860
1970         errorResult0$ = "Overflow"
1980         RETURN
1990         GOTO 2860
2000         errorResult0$ = "Out of memory"
2010         RETURN
2020         GOTO 2860
2030         errorResult0$ = "Subscript out of range"
2040         RETURN
2050         GOTO 2860
2060         errorResult0$ = "Duplicate Definition"
2070         RETURN
2080         GOTO 2860
2090         errorResult0$ = "Division by zero"
2100         RETURN
2110         GOTO 2860
2120         errorResult0$ = "Type mismatch"
2130         RETURN
2140         GOTO 2860
2150         errorResult0$ = "Out of string space"
2160         RETURN
2170         GOTO 2860
2180         errorResult0$ = "No RESUME"
2190         RETURN
2200         GOTO 2860
2210         errorResult0$ = "RESUME without error"
2220         RETURN
2230         GOTO 2860
2240         errorResult0$ = "Device timeout"
2250         RETURN
2260         GOTO 2860
2270         errorResult0$ = "Device fault"
2280         RETURN
2290         GOTO 2860
2300         errorResult0$ = "Out of paper"
2310         RETURN
2320         GOTO 2860
2330         errorResult0$ = "Bad file number"
2340         RETURN
2350         GOTO 2860
2360         errorResult0$ = "File not found"
2370         RETURN
2380         GOTO 2860
2390         errorResult0$ = "Bad file mode"
2400         RETURN
2410         GOTO 2860
2420         errorResult0$ = "File already open"
2430         RETURN
2440         GOTO 2860
2450         errorResult0$ = "Device I/O error"
2460         RETURN
2470         GOTO 2860
2480         errorResult0$ = "File already exists"
2490         RETURN
2500         GOTO 2860
2510         errorResult0$ = "Disk full"
2520         RETURN
2530         GOTO 2860
2540         errorResult0$ = "Input past end"
2550         RETURN
2560         GOTO 2860
2570         errorResult0$ = "Bad record number"
2580         RETURN
2590         GOTO 2860
2600         errorResult0$ = "Bad file name"
2610         RETURN
2620         GOTO 2860
2630         errorResult0$ = "Too many files"
2640         RETURN
2650         GOTO 2860
2660         errorResult0$ = "Device unavailable"
2670         RETURN
2680         GOTO 2860
2690         errorResult0$ = "Disk write protected"
2700         RETURN
2710         GOTO 2860
2720         errorResult0$ = "Disk not ready"
2730         RETURN
2740         GOTO 2860
2750         errorResult0$ = "Disk media error"
2760         RETURN
2770         GOTO 2860
2780         errorResult0$ = "Path/File access error"
2790         RETURN
2800         GOTO 2860
2810         errorResult0$ = "Path not found"
2820         RETURN
2830         GOTO 2860
2840         errorResult0$ = "Error " + STR$(errorCode0%)
2850         RETURN
2860     REM END SELECT
2870     RETURN
2880 ' end function error$

2890 ' catch's optional source$ binding: map ERL back to its original .bcl file
2900     IF ERL <= 510 THEN BCC_SOURCE_FILE$ = "com/bascal/stdlib/error.bcl" : RETURN
2910     IF ERL <= 1490 THEN BCC_SOURCE_FILE$ = "tutorial/17_labels_and_error_handling.bcl" : RETURN
2920     BCC_SOURCE_FILE$ = "com/bascal/stdlib/error.bcl"
2930     RETURN
