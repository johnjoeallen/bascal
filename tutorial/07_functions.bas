10 ' BASCAL generated BASIC
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Upper-cases self$. Not a real MBASIC/BASCOM 2.00 builtin -- verified
40 ' against a real IBM BASIC Compiler 2.00 under dosbox-x -- so BASCAL ships
50 ' its own. Declared as a scalar method (see GitHub issue #41 and
60 ' ltrim.bcl's own doc comment for the reasoning) -- ucase$(s$) still works
70 ' via ordinary-call syntax resolving to this same declaration.

80 ' Lower-cases self$. Not a real MBASIC/BASCOM 2.00 builtin -- verified
90 ' against a real IBM BASIC Compiler 2.00 under dosbox-x -- so BASCAL ships
100 ' its own. Declared as a scalar method (see GitHub issue #41 and
110 ' ltrim.bcl's own doc comment for the reasoning) -- lcase$(s$) still works
120 ' via ordinary-call syntax resolving to this same declaration.

130 ' Tutorial 7 — Functions
140 ' 
150 ' A BASCAL function is declared with FUNCTION ... END FUNCTION.
160 ' The function name carries the return type suffix.  Parameters
170 ' also carry type suffixes.  Every function must reach a RETURN.
180 ' 
190 ' Variables declared inside a function are local by default: the compiler
200 ' prefixes them with the function name.  To access a global variable from
210 ' inside a function, declare it with:  global varname
220 ' 
230 ' Functions cannot recurse, directly or indirectly (parameters would be
240 ' overwritten) -- the compiler checks the whole call graph and rejects
250 ' any cycle.  Use an explicit stack array for recursive algorithms.

260 ' Integer arithmetic functions
270 ' a% -- first value to compare
280 ' b% -- second value to compare

290 ' a% -- first value to compare
300 ' b% -- second value to compare

310 ' value% -- number to constrain
320 ' lo%    -- lower bound, inclusive
330 ' hi%    -- upper bound, inclusive

340 ' String functions
350 ' text$ -- string to repeat
360 ' n%    -- number of times to repeat it

370 ' word$ -- string to title-case

380 ' Local variable scoping — each function has its own i% and acc%
390 ' n% -- upper bound of the sum, inclusive

400 ' n% -- upper bound of the product, inclusive

410 ' Global variable accessed inside a function with the global keyword
420 runningtotal% = 0

430 ' x% -- amount to add to the running total

440 ' --- Exercise the functions ---

450 ' print mixes string labels and numeric results directly with ;
460 maxA0% = 4
470 maxB0% = 9
480 GOSUB 1380
490 PRINT "max(4, 9) = "; maxResult0%
500 minA0% = 4
510 minB0% = 9
520 GOSUB 1480
530 PRINT "min(4, 9) = "; minResult0%
540 clampValue0% = 15
550 clampLo0% = 1
560 clampHi0% = 10
570 GOSUB 1580
580 PRINT "clamp(15,1,10) = "; clampResult0%
590 clampValue0% = -3
600 clampLo0% = 1
610 clampHi0% = 10
620 GOSUB 1580
630 PRINT "clamp(-3,1,10) = "; clampResult0%
640 clampValue0% = 7
650 clampLo0% = 1
660 clampHi0% = 10
670 GOSUB 1580
680 PRINT "clamp(7,1,10)  = "; clampResult0%

690 repeatText0$ = "ab"
700 repeatN0% = 4
710 GOSUB 1690
720 PRINT repeatResult0$
730 titlecaseWord0$ = "bASCAL"
740 GOSUB 1780
750 PRINT titlecaseResult0$

760 ' Functions chained in expressions
770 maxA0% = 0
780 maxB0% = -5
790 GOSUB 1380
800 minA0% = maxResult0%
810 minB0% = 100
820 GOSUB 1480
830 lo% = minResult0%
840 PRINT "lo = "; lo%

850 ' Calling the same function twice — each result is captured separately
860 repeatText0$ = "x"
870 repeatN0% = 3
880 GOSUB 1690
890 a$ = repeatResult0$
900 repeatText0$ = "y"
910 repeatN0% = 2
920 GOSUB 1690
930 b$ = repeatResult0$
940 PRINT a$; " "; b$

950 ' Local scoping: sumTo% and productTo% each use i% without conflict
960 sumtoN0% = 5
970 GOSUB 1940
980 PRINT "sumTo(5)     = "; sumtoResult0%
990 producttoN0% = 5
1000 GOSUB 2030
1010 PRINT "productTo(5) = "; producttoResult0%

1020 ' Global variable shared across calls
1030 addtototalX0% = 10
1040 GOSUB 2120
1050 dummy% = addtototalResult0%
1060 addtototalX0% = 5
1070 GOSUB 2120
1080 dummy% = addtototalResult0%
1090 PRINT "runningTotal = "; runningtotal%

1100 END

1110 ' function ucase$()
1120     ucaseOut0$ = ""
1130     FOR ucaseI0% = 1 TO LEN(ucaseSelf0$)
1140         ucaseC0% = ASC(MID$(ucaseSelf0$, ucaseI0%, 1))
1150         IF (ucaseC0% >= 97) = 0 THEN GOTO 1180
1160         IF (ucaseC0% <= 122) = 0 THEN GOTO 1180
1170             ucaseC0% = ucaseC0% - 32
1180         REM END IF
1190         ucaseOut0$ = ucaseOut0$ + CHR$(ucaseC0%)
1200     NEXT ucaseI0%
1210     ucaseResult0$ = ucaseOut0$
1220     RETURN
1230 ' end function ucase$

1240 ' function lcase$()
1250     lcaseOut0$ = ""
1260     FOR lcaseI0% = 1 TO LEN(lcaseSelf0$)
1270         lcaseC0% = ASC(MID$(lcaseSelf0$, lcaseI0%, 1))
1280         IF (lcaseC0% >= 65) = 0 THEN GOTO 1310
1290         IF (lcaseC0% <= 90) = 0 THEN GOTO 1310
1300             lcaseC0% = lcaseC0% + 32
1310         REM END IF
1320         lcaseOut0$ = lcaseOut0$ + CHR$(lcaseC0%)
1330     NEXT lcaseI0%
1340     lcaseResult0$ = lcaseOut0$
1350     RETURN
1360 ' end function lcase$

1370 ' function max%(a%, b%)
1380     IF (maxA0% > maxB0%) = 0 THEN GOTO 1420
1390         maxResult0% = maxA0%
1400         RETURN
1410         GOTO 1440
1420         maxResult0% = maxB0%
1430         RETURN
1440     REM END IF
1450     RETURN
1460 ' end function max%

1470 ' function min%(a%, b%)
1480     IF (minA0% < minB0%) = 0 THEN GOTO 1520
1490         minResult0% = minA0%
1500         RETURN
1510         GOTO 1540
1520         minResult0% = minB0%
1530         RETURN
1540     REM END IF
1550     RETURN
1560 ' end function min%

1570 ' function clamp%(value%, lo%, hi%)
1580     ' Constrain value to [lo, hi].
1590     minA0% = clampValue0%
1600     minB0% = clampHi0%
1610     GOSUB 1480
1620     maxA0% = clampLo0%
1630     maxB0% = minResult0%
1640     GOSUB 1380
1650     clampResult0% = maxResult0%
1660     RETURN
1670 ' end function clamp%

1680 ' function repeat$(text$, n%)
1690     ' Concatenate text$ with itself n times.
1700     repeatAcc0$ = ""
1710     FOR repeatI0% = 1 TO repeatN0%
1720         repeatAcc0$ = repeatAcc0$ + repeatText0$
1730     NEXT repeatI0%
1740     repeatResult0$ = repeatAcc0$
1750     RETURN
1760 ' end function repeat$

1770 ' function titlecase$(word$)
1780     ' Capitalise first letter, lowercase remainder.
1790     ' UCASE$/LCASE$ aren't real MBASIC/BASCOM 2.00 builtins (verified
1800     ' against a real IBM BASIC Compiler 2.00 under dosbox-x), so this
1810     ' requires BASCAL's own com.bascal.stdlib implementations above.
1820     IF (LEN(titlecaseWord0$) = 0) = 0 THEN GOTO 1850
1830         titlecaseResult0$ = ""
1840         RETURN
1850     REM END IF
1860     ucaseSelf0$ = LEFT$(titlecaseWord0$, 1)
1870     GOSUB 1120
1880     lcaseSelf0$ = MID$(titlecaseWord0$, 2)
1890     GOSUB 1250
1900     titlecaseResult0$ = ucaseResult0$ + lcaseResult0$
1910     RETURN
1920 ' end function titlecase$

1930 ' function sumto%(n%)
1940     ' i% and acc% are local to sumTo%.
1950     sumtoAcc0% = 0
1960     FOR sumtoI0% = 1 TO sumtoN0%
1970         sumtoAcc0% = sumtoAcc0% + sumtoI0%
1980     NEXT sumtoI0%
1990     sumtoResult0% = sumtoAcc0%
2000     RETURN
2010 ' end function sumto%

2020 ' function productto%(n%)
2030     ' i% and acc% here are independent of sumTo%'s i% and acc%.
2040     producttoAcc0% = 1
2050     FOR producttoI0% = 1 TO producttoN0%
2060         producttoAcc0% = producttoAcc0% * producttoI0%
2070     NEXT producttoI0%
2080     producttoResult0% = producttoAcc0%
2090     RETURN
2100 ' end function productto%

2110 ' function addtototal%(x%)
2120     runningtotal% = runningtotal% + addtototalX0%
2130     addtototalResult0% = runningtotal%
2140     RETURN
2150 ' end function addtototal%
