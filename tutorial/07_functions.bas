10 ' BASCAL generated BASIC
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Upper-cases s$. Not a real MBASIC/BASCOM 2.00 builtin -- verified against
40 ' a real IBM BASIC Compiler 2.00 under dosbox-x -- so BASCAL ships its own.
50 ' Lower-cases s$. Not a real MBASIC/BASCOM 2.00 builtin -- verified against
60 ' a real IBM BASIC Compiler 2.00 under dosbox-x -- so BASCAL ships its own.
70 ' Tutorial 7 — Functions
80 ' 
90 ' A BASCAL function is declared with FUNCTION ... END FUNCTION.
100 ' The function name carries the return type suffix.  Parameters
110 ' also carry type suffixes.  Every function must reach a RETURN.
120 ' 
130 ' Variables declared inside a function are local by default: the compiler
140 ' prefixes them with the function name.  To access a global variable from
150 ' inside a function, declare it with:  global varname
160 ' 
170 ' Functions cannot recurse, directly or indirectly (parameters would be
180 ' overwritten) -- the compiler checks the whole call graph and rejects
190 ' any cycle.  Use an explicit stack array for recursive algorithms.

200 ' Integer arithmetic functions
210 ' a% -- first value to compare
220 ' b% -- second value to compare

230 ' a% -- first value to compare
240 ' b% -- second value to compare

250 ' value% -- number to constrain
260 ' lo%    -- lower bound, inclusive
270 ' hi%    -- upper bound, inclusive

280 ' String functions
290 ' text$ -- string to repeat
300 ' n%    -- number of times to repeat it

310 ' word$ -- string to title-case

320 ' Local variable scoping — each function has its own i% and acc%
330 ' n% -- upper bound of the sum, inclusive

340 ' n% -- upper bound of the product, inclusive

350 ' Global variable accessed inside a function with the global keyword
360 runningtotal% = 0

370 ' x% -- amount to add to the running total

380 ' --- Exercise the functions ---

390 ' print mixes string labels and numeric results directly with ;
400 maxA0% = 4
410 maxB0% = 9
420 GOSUB 1320
430 PRINT "max(4, 9) = "; maxResult0%
440 minA0% = 4
450 minB0% = 9
460 GOSUB 1420
470 PRINT "min(4, 9) = "; minResult0%
480 clampValue0% = 15
490 clampLo0% = 1
500 clampHi0% = 10
510 GOSUB 1520
520 PRINT "clamp(15,1,10) = "; clampResult0%
530 clampValue0% = -3
540 clampLo0% = 1
550 clampHi0% = 10
560 GOSUB 1520
570 PRINT "clamp(-3,1,10) = "; clampResult0%
580 clampValue0% = 7
590 clampLo0% = 1
600 clampHi0% = 10
610 GOSUB 1520
620 PRINT "clamp(7,1,10)  = "; clampResult0%

630 repeatText0$ = "ab"
640 repeatN0% = 4
650 GOSUB 1630
660 PRINT repeatResult0$
670 titlecaseWord0$ = "bASCAL"
680 GOSUB 1720
690 PRINT titlecaseResult0$

700 ' Functions chained in expressions
710 maxA0% = 0
720 maxB0% = -5
730 GOSUB 1320
740 minA0% = maxResult0%
750 minB0% = 100
760 GOSUB 1420
770 lo% = minResult0%
780 PRINT "lo = "; lo%

790 ' Calling the same function twice — each result is captured separately
800 repeatText0$ = "x"
810 repeatN0% = 3
820 GOSUB 1630
830 a$ = repeatResult0$
840 repeatText0$ = "y"
850 repeatN0% = 2
860 GOSUB 1630
870 b$ = repeatResult0$
880 PRINT a$; " "; b$

890 ' Local scoping: sumTo% and productTo% each use i% without conflict
900 sumtoN0% = 5
910 GOSUB 1880
920 PRINT "sumTo(5)     = "; sumtoResult0%
930 producttoN0% = 5
940 GOSUB 1970
950 PRINT "productTo(5) = "; producttoResult0%

960 ' Global variable shared across calls
970 addtototalX0% = 10
980 GOSUB 2060
990 dummy% = addtototalResult0%
1000 addtototalX0% = 5
1010 GOSUB 2060
1020 dummy% = addtototalResult0%
1030 PRINT "runningTotal = "; runningtotal%

1040 END

1050 ' function ucase$(s$)
1060     ucaseOut0$ = ""
1070     FOR ucaseI0% = 1 TO LEN(ucaseS0$)
1080         ucaseC0% = ASC(MID$(ucaseS0$, ucaseI0%, 1))
1090         IF (ucaseC0% >= 97) = 0 THEN GOTO 1120
1100         IF (ucaseC0% <= 122) = 0 THEN GOTO 1120
1110             ucaseC0% = ucaseC0% - 32
1120         REM END IF
1130         ucaseOut0$ = ucaseOut0$ + CHR$(ucaseC0%)
1140     NEXT ucaseI0%
1150     ucaseResult0$ = ucaseOut0$
1160     RETURN
1170 ' end function ucase$

1180 ' function lcase$(s$)
1190     lcaseOut0$ = ""
1200     FOR lcaseI0% = 1 TO LEN(lcaseS0$)
1210         lcaseC0% = ASC(MID$(lcaseS0$, lcaseI0%, 1))
1220         IF (lcaseC0% >= 65) = 0 THEN GOTO 1250
1230         IF (lcaseC0% <= 90) = 0 THEN GOTO 1250
1240             lcaseC0% = lcaseC0% + 32
1250         REM END IF
1260         lcaseOut0$ = lcaseOut0$ + CHR$(lcaseC0%)
1270     NEXT lcaseI0%
1280     lcaseResult0$ = lcaseOut0$
1290     RETURN
1300 ' end function lcase$

1310 ' function max%(a%, b%)
1320     IF (maxA0% > maxB0%) = 0 THEN GOTO 1360
1330         maxResult0% = maxA0%
1340         RETURN
1350         GOTO 1380
1360         maxResult0% = maxB0%
1370         RETURN
1380     REM END IF
1390     RETURN
1400 ' end function max%

1410 ' function min%(a%, b%)
1420     IF (minA0% < minB0%) = 0 THEN GOTO 1460
1430         minResult0% = minA0%
1440         RETURN
1450         GOTO 1480
1460         minResult0% = minB0%
1470         RETURN
1480     REM END IF
1490     RETURN
1500 ' end function min%

1510 ' function clamp%(value%, lo%, hi%)
1520     ' Constrain value to [lo, hi].
1530     minA0% = clampValue0%
1540     minB0% = clampHi0%
1550     GOSUB 1420
1560     maxA0% = clampLo0%
1570     maxB0% = minResult0%
1580     GOSUB 1320
1590     clampResult0% = maxResult0%
1600     RETURN
1610 ' end function clamp%

1620 ' function repeat$(text$, n%)
1630     ' Concatenate text$ with itself n times.
1640     repeatAcc0$ = ""
1650     FOR repeatI0% = 1 TO repeatN0%
1660         repeatAcc0$ = repeatAcc0$ + repeatText0$
1670     NEXT repeatI0%
1680     repeatResult0$ = repeatAcc0$
1690     RETURN
1700 ' end function repeat$

1710 ' function titlecase$(word$)
1720     ' Capitalise first letter, lowercase remainder.
1730     ' UCASE$/LCASE$ aren't real MBASIC/BASCOM 2.00 builtins (verified
1740     ' against a real IBM BASIC Compiler 2.00 under dosbox-x), so this
1750     ' requires BASCAL's own com.bascal.stdlib implementations above.
1760     IF (LEN(titlecaseWord0$) = 0) = 0 THEN GOTO 1790
1770         titlecaseResult0$ = ""
1780         RETURN
1790     REM END IF
1800     ucaseS0$ = LEFT$(titlecaseWord0$, 1)
1810     GOSUB 1060
1820     lcaseS0$ = MID$(titlecaseWord0$, 2)
1830     GOSUB 1190
1840     titlecaseResult0$ = ucaseResult0$ + lcaseResult0$
1850     RETURN
1860 ' end function titlecase$

1870 ' function sumto%(n%)
1880     ' i% and acc% are local to sumTo%.
1890     sumtoAcc0% = 0
1900     FOR sumtoI0% = 1 TO sumtoN0%
1910         sumtoAcc0% = sumtoAcc0% + sumtoI0%
1920     NEXT sumtoI0%
1930     sumtoResult0% = sumtoAcc0%
1940     RETURN
1950 ' end function sumto%

1960 ' function productto%(n%)
1970     ' i% and acc% here are independent of sumTo%'s i% and acc%.
1980     producttoAcc0% = 1
1990     FOR producttoI0% = 1 TO producttoN0%
2000         producttoAcc0% = producttoAcc0% * producttoI0%
2010     NEXT producttoI0%
2020     producttoResult0% = producttoAcc0%
2030     RETURN
2040 ' end function productto%

2050 ' function addtototal%(x%)
2060     runningtotal% = runningtotal% + addtototalX0%
2070     addtototalResult0% = runningtotal%
2080     RETURN
2090 ' end function addtototal%
