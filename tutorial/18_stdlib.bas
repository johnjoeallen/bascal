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
300 ' dosbox-x) -- see MANUAL.md's "String and error-message functions"
310 ' section for the full story.
320 ' 
330 ' Run with:
340 ' bcc tutorial/18_stdlib.bcl

350 ltrimS0$ = "   padded left"
360 GOSUB 600
370 PRINT ("[" + ltrimResult0$) + "]"
380 rtrimS0$ = "padded right   "
390 GOSUB 700
400 PRINT ("[" + rtrimResult0$) + "]"
410 ucaseS0$ = "shout this"
420 GOSUB 800
430 PRINT ucaseResult0$
440 lcaseS0$ = "QUIET THIS DOWN"
450 GOSUB 930
460 PRINT lcaseResult0$

470 ' ERROR$ maps a classic MBASIC/GW-BASIC/BASCOM error code to a message;
480 ' pair it with ERR inside an ON ERROR GOTO handler in real code.
490 errorCode0% = 53
500 GOSUB 1060
510 PRINT errorResult0$
520 errorCode0% = 11
530 GOSUB 1060
540 PRINT errorResult0$
550 errorCode0% = 9999
560 GOSUB 1060
570 PRINT errorResult0$

580 END

590 ' function ltrim$(s$)
600     ltrimI0% = 1
610     IF (ltrimI0% <= LEN(ltrimS0$)) = 0 THEN GOTO 650
620     IF (MID$(ltrimS0$, ltrimI0%, 1) = " ") = 0 THEN GOTO 650
630         ltrimI0% = ltrimI0% + 1
640         GOTO 610
650     REM END WHILE
660     ltrimResult0$ = MID$(ltrimS0$, ltrimI0%)
670     RETURN
680 ' end function ltrim$

690 ' function rtrim$(s$)
700     rtrimI0% = LEN(rtrimS0$)
710     IF (rtrimI0% > 0) = 0 THEN GOTO 750
720     IF (MID$(rtrimS0$, rtrimI0%, 1) = " ") = 0 THEN GOTO 750
730         rtrimI0% = rtrimI0% - 1
740         GOTO 710
750     REM END WHILE
760     rtrimResult0$ = LEFT$(rtrimS0$, rtrimI0%)
770     RETURN
780 ' end function rtrim$

790 ' function ucase$(s$)
800     ucaseOut0$ = ""
810     FOR ucaseI0% = 1 TO LEN(ucaseS0$)
820         ucaseC0% = ASC(MID$(ucaseS0$, ucaseI0%, 1))
830         IF (ucaseC0% >= 97) = 0 THEN GOTO 860
840         IF (ucaseC0% <= 122) = 0 THEN GOTO 860
850             ucaseC0% = ucaseC0% - 32
860         REM END IF
870         ucaseOut0$ = ucaseOut0$ + CHR$(ucaseC0%)
880     NEXT ucaseI0%
890     ucaseResult0$ = ucaseOut0$
900     RETURN
910 ' end function ucase$

920 ' function lcase$(s$)
930     lcaseOut0$ = ""
940     FOR lcaseI0% = 1 TO LEN(lcaseS0$)
950         lcaseC0% = ASC(MID$(lcaseS0$, lcaseI0%, 1))
960         IF (lcaseC0% >= 65) = 0 THEN GOTO 990
970         IF (lcaseC0% <= 90) = 0 THEN GOTO 990
980             lcaseC0% = lcaseC0% + 32
990         REM END IF
1000         lcaseOut0$ = lcaseOut0$ + CHR$(lcaseC0%)
1010     NEXT lcaseI0%
1020     lcaseResult0$ = lcaseOut0$
1030     RETURN
1040 ' end function lcase$

1050 ' function error$(code%)
1060     BCCT6% = errorCode0%
1070     IF (BCCT6% = 2) <> 0 THEN GOTO 1410
1080     IF (BCCT6% = 3) <> 0 THEN GOTO 1440
1090     IF (BCCT6% = 4) <> 0 THEN GOTO 1470
1100     IF (BCCT6% = 5) <> 0 THEN GOTO 1500
1110     IF (BCCT6% = 6) <> 0 THEN GOTO 1530
1120     IF (BCCT6% = 7) <> 0 THEN GOTO 1560
1130     IF (BCCT6% = 9) <> 0 THEN GOTO 1590
1140     IF (BCCT6% = 10) <> 0 THEN GOTO 1620
1150     IF (BCCT6% = 11) <> 0 THEN GOTO 1650
1160     IF (BCCT6% = 13) <> 0 THEN GOTO 1680
1170     IF (BCCT6% = 14) <> 0 THEN GOTO 1710
1180     IF (BCCT6% = 19) <> 0 THEN GOTO 1740
1190     IF (BCCT6% = 20) <> 0 THEN GOTO 1770
1200     IF (BCCT6% = 24) <> 0 THEN GOTO 1800
1210     IF (BCCT6% = 25) <> 0 THEN GOTO 1830
1220     IF (BCCT6% = 27) <> 0 THEN GOTO 1860
1230     IF (BCCT6% = 52) <> 0 THEN GOTO 1890
1240     IF (BCCT6% = 53) <> 0 THEN GOTO 1920
1250     IF (BCCT6% = 54) <> 0 THEN GOTO 1950
1260     IF (BCCT6% = 55) <> 0 THEN GOTO 1980
1270     IF (BCCT6% = 57) <> 0 THEN GOTO 2010
1280     IF (BCCT6% = 58) <> 0 THEN GOTO 2040
1290     IF (BCCT6% = 61) <> 0 THEN GOTO 2070
1300     IF (BCCT6% = 62) <> 0 THEN GOTO 2100
1310     IF (BCCT6% = 63) <> 0 THEN GOTO 2130
1320     IF (BCCT6% = 64) <> 0 THEN GOTO 2160
1330     IF (BCCT6% = 67) <> 0 THEN GOTO 2190
1340     IF (BCCT6% = 68) <> 0 THEN GOTO 2220
1350     IF (BCCT6% = 70) <> 0 THEN GOTO 2250
1360     IF (BCCT6% = 71) <> 0 THEN GOTO 2280
1370     IF (BCCT6% = 72) <> 0 THEN GOTO 2310
1380     IF (BCCT6% = 75) <> 0 THEN GOTO 2340
1390     IF (BCCT6% = 76) <> 0 THEN GOTO 2370
1400     GOTO 2400
1410         errorResult0$ = "Syntax error"
1420         RETURN
1430         GOTO 2420
1440         errorResult0$ = "RETURN without GOSUB"
1450         RETURN
1460         GOTO 2420
1470         errorResult0$ = "Out of DATA"
1480         RETURN
1490         GOTO 2420
1500         errorResult0$ = "Illegal function call"
1510         RETURN
1520         GOTO 2420
1530         errorResult0$ = "Overflow"
1540         RETURN
1550         GOTO 2420
1560         errorResult0$ = "Out of memory"
1570         RETURN
1580         GOTO 2420
1590         errorResult0$ = "Subscript out of range"
1600         RETURN
1610         GOTO 2420
1620         errorResult0$ = "Duplicate Definition"
1630         RETURN
1640         GOTO 2420
1650         errorResult0$ = "Division by zero"
1660         RETURN
1670         GOTO 2420
1680         errorResult0$ = "Type mismatch"
1690         RETURN
1700         GOTO 2420
1710         errorResult0$ = "Out of string space"
1720         RETURN
1730         GOTO 2420
1740         errorResult0$ = "No RESUME"
1750         RETURN
1760         GOTO 2420
1770         errorResult0$ = "RESUME without error"
1780         RETURN
1790         GOTO 2420
1800         errorResult0$ = "Device timeout"
1810         RETURN
1820         GOTO 2420
1830         errorResult0$ = "Device fault"
1840         RETURN
1850         GOTO 2420
1860         errorResult0$ = "Out of paper"
1870         RETURN
1880         GOTO 2420
1890         errorResult0$ = "Bad file number"
1900         RETURN
1910         GOTO 2420
1920         errorResult0$ = "File not found"
1930         RETURN
1940         GOTO 2420
1950         errorResult0$ = "Bad file mode"
1960         RETURN
1970         GOTO 2420
1980         errorResult0$ = "File already open"
1990         RETURN
2000         GOTO 2420
2010         errorResult0$ = "Device I/O error"
2020         RETURN
2030         GOTO 2420
2040         errorResult0$ = "File already exists"
2050         RETURN
2060         GOTO 2420
2070         errorResult0$ = "Disk full"
2080         RETURN
2090         GOTO 2420
2100         errorResult0$ = "Input past end"
2110         RETURN
2120         GOTO 2420
2130         errorResult0$ = "Bad record number"
2140         RETURN
2150         GOTO 2420
2160         errorResult0$ = "Bad file name"
2170         RETURN
2180         GOTO 2420
2190         errorResult0$ = "Too many files"
2200         RETURN
2210         GOTO 2420
2220         errorResult0$ = "Device unavailable"
2230         RETURN
2240         GOTO 2420
2250         errorResult0$ = "Disk write protected"
2260         RETURN
2270         GOTO 2420
2280         errorResult0$ = "Disk not ready"
2290         RETURN
2300         GOTO 2420
2310         errorResult0$ = "Disk media error"
2320         RETURN
2330         GOTO 2420
2340         errorResult0$ = "Path/File access error"
2350         RETURN
2360         GOTO 2420
2370         errorResult0$ = "Path not found"
2380         RETURN
2390         GOTO 2420
2400         errorResult0$ = "Error " + STR$(errorCode0%)
2410         RETURN
2420     REM END SELECT
2430     RETURN
2440 ' end function error$
