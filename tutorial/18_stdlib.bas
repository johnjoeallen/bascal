10 ' BASCAL generated BASIC
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Strips leading spaces from s$. Not a real MBASIC/BASCOM 2.00 builtin --
40 ' verified against a real IBM BASIC Compiler 2.00 under dosbox-x -- so
50 ' BASCAL ships its own.

60 ' Strips trailing spaces from s$. Not a real MBASIC/BASCOM 2.00 builtin --
70 ' verified against a real IBM BASIC Compiler 2.00 under dosbox-x -- so
80 ' BASCAL ships its own.

90 ' Upper-cases s$. Not a real MBASIC/BASCOM 2.00 builtin -- verified against
100 ' a real IBM BASIC Compiler 2.00 under dosbox-x -- so BASCAL ships its own.

110 ' Lower-cases s$. Not a real MBASIC/BASCOM 2.00 builtin -- verified against
120 ' a real IBM BASIC Compiler 2.00 under dosbox-x -- so BASCAL ships its own.

130 ' Maps an ERR code to its classic MBASIC/GW-BASIC/BASCOM message. Compiles
140 ' and links on a real IBM BASIC Compiler 2.00 as ERROR$, but silently
150 ' returns an empty string at runtime (verified under dosbox-x) -- so BASCAL
160 ' ships a working implementation.
170 ' 
180 ' Covers the classic error codes an ON ERROR GOTO + ERR handler is
190 ' realistically going to hit -- not the full table, but every code common
200 ' enough to be worth a real message instead of falling through to the
210 ' generic one.

220 ' Tutorial 18 — Standard library functions
230 ' 
240 ' com.bascal.stdlib is an ordinary require-able library, resolved the same
250 ' way as com.bascal.sort in tutorial 12 -- but bcc always adds its home
260 ' directory to the search path automatically, so no -L flag is needed to
270 ' reach it. It exists because LTRIM$, RTRIM$, UCASE$, LCASE$, and ERROR$
280 ' either aren't real MBASIC/BASCOM 2.00 builtins or don't work at runtime
290 ' (verified against a real IBM Personal Computer BASIC Compiler 2.00 under
300 ' dosbox-x) -- see the manual's "String and error-message functions"
310 ' section (https://johnjoeallen.github.io/bascal/manual/) for the full
320 ' story.
330 ' 
340 ' Run with:
350 ' bcc tutorial/18_stdlib.bcl

360 ltrimS0$ = "   padded left"
370 GOSUB 610
380 PRINT ("[" + ltrimResult0$) + "]"
390 rtrimS0$ = "padded right   "
400 GOSUB 710
410 PRINT ("[" + rtrimResult0$) + "]"
420 ucaseS0$ = "shout this"
430 GOSUB 810
440 PRINT ucaseResult0$
450 lcaseS0$ = "QUIET THIS DOWN"
460 GOSUB 940
470 PRINT lcaseResult0$

480 ' ERROR$ maps a classic MBASIC/GW-BASIC/BASCOM error code to a message;
490 ' pair it with ERR inside an ON ERROR GOTO handler in real code.
500 errorCode0% = 53
510 GOSUB 1070
520 PRINT errorResult0$
530 errorCode0% = 11
540 GOSUB 1070
550 PRINT errorResult0$
560 errorCode0% = 9999
570 GOSUB 1070
580 PRINT errorResult0$

590 END

600 ' function ltrim$(s$)
610     ltrimI0% = 1
620     IF (ltrimI0% <= LEN(ltrimS0$)) = 0 THEN GOTO 660
630     IF (MID$(ltrimS0$, ltrimI0%, 1) = " ") = 0 THEN GOTO 660
640         ltrimI0% = ltrimI0% + 1
650         GOTO 620
660     REM END WHILE
670     ltrimResult0$ = MID$(ltrimS0$, ltrimI0%)
680     RETURN
690 ' end function ltrim$

700 ' function rtrim$(s$)
710     rtrimI0% = LEN(rtrimS0$)
720     IF (rtrimI0% > 0) = 0 THEN GOTO 760
730     IF (MID$(rtrimS0$, rtrimI0%, 1) = " ") = 0 THEN GOTO 760
740         rtrimI0% = rtrimI0% - 1
750         GOTO 720
760     REM END WHILE
770     rtrimResult0$ = LEFT$(rtrimS0$, rtrimI0%)
780     RETURN
790 ' end function rtrim$

800 ' function ucase$(s$)
810     ucaseOut0$ = ""
820     FOR ucaseI0% = 1 TO LEN(ucaseS0$)
830         ucaseC0% = ASC(MID$(ucaseS0$, ucaseI0%, 1))
840         IF (ucaseC0% >= 97) = 0 THEN GOTO 870
850         IF (ucaseC0% <= 122) = 0 THEN GOTO 870
860             ucaseC0% = ucaseC0% - 32
870         REM END IF
880         ucaseOut0$ = ucaseOut0$ + CHR$(ucaseC0%)
890     NEXT ucaseI0%
900     ucaseResult0$ = ucaseOut0$
910     RETURN
920 ' end function ucase$

930 ' function lcase$(s$)
940     lcaseOut0$ = ""
950     FOR lcaseI0% = 1 TO LEN(lcaseS0$)
960         lcaseC0% = ASC(MID$(lcaseS0$, lcaseI0%, 1))
970         IF (lcaseC0% >= 65) = 0 THEN GOTO 1000
980         IF (lcaseC0% <= 90) = 0 THEN GOTO 1000
990             lcaseC0% = lcaseC0% + 32
1000         REM END IF
1010         lcaseOut0$ = lcaseOut0$ + CHR$(lcaseC0%)
1020     NEXT lcaseI0%
1030     lcaseResult0$ = lcaseOut0$
1040     RETURN
1050 ' end function lcase$

1060 ' function error$(code%)
1070     BCCT6% = errorCode0%
1080     IF (BCCT6% = 2) <> 0 THEN GOTO 1420
1090     IF (BCCT6% = 3) <> 0 THEN GOTO 1450
1100     IF (BCCT6% = 4) <> 0 THEN GOTO 1480
1110     IF (BCCT6% = 5) <> 0 THEN GOTO 1510
1120     IF (BCCT6% = 6) <> 0 THEN GOTO 1540
1130     IF (BCCT6% = 7) <> 0 THEN GOTO 1570
1140     IF (BCCT6% = 9) <> 0 THEN GOTO 1600
1150     IF (BCCT6% = 10) <> 0 THEN GOTO 1630
1160     IF (BCCT6% = 11) <> 0 THEN GOTO 1660
1170     IF (BCCT6% = 13) <> 0 THEN GOTO 1690
1180     IF (BCCT6% = 14) <> 0 THEN GOTO 1720
1190     IF (BCCT6% = 19) <> 0 THEN GOTO 1750
1200     IF (BCCT6% = 20) <> 0 THEN GOTO 1780
1210     IF (BCCT6% = 24) <> 0 THEN GOTO 1810
1220     IF (BCCT6% = 25) <> 0 THEN GOTO 1840
1230     IF (BCCT6% = 27) <> 0 THEN GOTO 1870
1240     IF (BCCT6% = 52) <> 0 THEN GOTO 1900
1250     IF (BCCT6% = 53) <> 0 THEN GOTO 1930
1260     IF (BCCT6% = 54) <> 0 THEN GOTO 1960
1270     IF (BCCT6% = 55) <> 0 THEN GOTO 1990
1280     IF (BCCT6% = 57) <> 0 THEN GOTO 2020
1290     IF (BCCT6% = 58) <> 0 THEN GOTO 2050
1300     IF (BCCT6% = 61) <> 0 THEN GOTO 2080
1310     IF (BCCT6% = 62) <> 0 THEN GOTO 2110
1320     IF (BCCT6% = 63) <> 0 THEN GOTO 2140
1330     IF (BCCT6% = 64) <> 0 THEN GOTO 2170
1340     IF (BCCT6% = 67) <> 0 THEN GOTO 2200
1350     IF (BCCT6% = 68) <> 0 THEN GOTO 2230
1360     IF (BCCT6% = 70) <> 0 THEN GOTO 2260
1370     IF (BCCT6% = 71) <> 0 THEN GOTO 2290
1380     IF (BCCT6% = 72) <> 0 THEN GOTO 2320
1390     IF (BCCT6% = 75) <> 0 THEN GOTO 2350
1400     IF (BCCT6% = 76) <> 0 THEN GOTO 2380
1410     GOTO 2410
1420         errorResult0$ = "Syntax error"
1430         RETURN
1440         GOTO 2430
1450         errorResult0$ = "RETURN without GOSUB"
1460         RETURN
1470         GOTO 2430
1480         errorResult0$ = "Out of DATA"
1490         RETURN
1500         GOTO 2430
1510         errorResult0$ = "Illegal function call"
1520         RETURN
1530         GOTO 2430
1540         errorResult0$ = "Overflow"
1550         RETURN
1560         GOTO 2430
1570         errorResult0$ = "Out of memory"
1580         RETURN
1590         GOTO 2430
1600         errorResult0$ = "Subscript out of range"
1610         RETURN
1620         GOTO 2430
1630         errorResult0$ = "Duplicate Definition"
1640         RETURN
1650         GOTO 2430
1660         errorResult0$ = "Division by zero"
1670         RETURN
1680         GOTO 2430
1690         errorResult0$ = "Type mismatch"
1700         RETURN
1710         GOTO 2430
1720         errorResult0$ = "Out of string space"
1730         RETURN
1740         GOTO 2430
1750         errorResult0$ = "No RESUME"
1760         RETURN
1770         GOTO 2430
1780         errorResult0$ = "RESUME without error"
1790         RETURN
1800         GOTO 2430
1810         errorResult0$ = "Device timeout"
1820         RETURN
1830         GOTO 2430
1840         errorResult0$ = "Device fault"
1850         RETURN
1860         GOTO 2430
1870         errorResult0$ = "Out of paper"
1880         RETURN
1890         GOTO 2430
1900         errorResult0$ = "Bad file number"
1910         RETURN
1920         GOTO 2430
1930         errorResult0$ = "File not found"
1940         RETURN
1950         GOTO 2430
1960         errorResult0$ = "Bad file mode"
1970         RETURN
1980         GOTO 2430
1990         errorResult0$ = "File already open"
2000         RETURN
2010         GOTO 2430
2020         errorResult0$ = "Device I/O error"
2030         RETURN
2040         GOTO 2430
2050         errorResult0$ = "File already exists"
2060         RETURN
2070         GOTO 2430
2080         errorResult0$ = "Disk full"
2090         RETURN
2100         GOTO 2430
2110         errorResult0$ = "Input past end"
2120         RETURN
2130         GOTO 2430
2140         errorResult0$ = "Bad record number"
2150         RETURN
2160         GOTO 2430
2170         errorResult0$ = "Bad file name"
2180         RETURN
2190         GOTO 2430
2200         errorResult0$ = "Too many files"
2210         RETURN
2220         GOTO 2430
2230         errorResult0$ = "Device unavailable"
2240         RETURN
2250         GOTO 2430
2260         errorResult0$ = "Disk write protected"
2270         RETURN
2280         GOTO 2430
2290         errorResult0$ = "Disk not ready"
2300         RETURN
2310         GOTO 2430
2320         errorResult0$ = "Disk media error"
2330         RETURN
2340         GOTO 2430
2350         errorResult0$ = "Path/File access error"
2360         RETURN
2370         GOTO 2430
2380         errorResult0$ = "Path not found"
2390         RETURN
2400         GOTO 2430
2410         errorResult0$ = "Error " + STR$(errorCode0%)
2420         RETURN
2430     REM END SELECT
2440     RETURN
2450 ' end function error$
