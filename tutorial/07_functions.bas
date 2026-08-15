10 ' BASCAL generated BASIC
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Tutorial 7 — Functions
40 ' 
50 ' A BASCAL function is declared with FUNCTION ... END FUNCTION.
60 ' The function name carries the return type suffix.  Parameters
70 ' also carry type suffixes.  Every function must reach a RETURN.
80 ' 
90 ' Variables declared inside a function are local by default: the compiler
100 ' prefixes them with the function name.  To access a global variable from
110 ' inside a function, declare it with:  global varname
120 ' 
130 ' Functions cannot recurse, directly or indirectly (parameters would be
140 ' overwritten) -- the compiler checks the whole call graph and rejects
150 ' any cycle.  Use an explicit stack array for recursive algorithms.

160 ' Integer arithmetic functions
170 ' a% -- first value to compare
180 ' b% -- second value to compare

190 ' a% -- first value to compare
200 ' b% -- second value to compare

210 ' value% -- number to constrain
220 ' lo%    -- lower bound, inclusive
230 ' hi%    -- upper bound, inclusive

240 ' String functions
250 ' text$ -- string to repeat
260 ' n%    -- number of times to repeat it

270 ' word$ -- string to title-case

280 ' Local variable scoping — each function has its own i% and acc%
290 ' n% -- upper bound of the sum, inclusive

300 ' n% -- upper bound of the product, inclusive

310 ' Global variable accessed inside a function with the global keyword
320 runningtotal% = 0

330 ' x% -- amount to add to the running total

340 ' --- Exercise the functions ---

350 ' print mixes string labels and numeric results directly with ;
360 maxA0% = 4
370 maxB0% = 9
380 GOSUB 1020
390 PRINT "max(4, 9) = "; maxResult0%
400 minA0% = 4
410 minB0% = 9
420 GOSUB 1120
430 PRINT "min(4, 9) = "; minResult0%
440 clampValue0% = 15
450 clampLo0% = 1
460 clampHi0% = 10
470 GOSUB 1220
480 PRINT "clamp(15,1,10) = "; clampResult0%
490 clampValue0% = -3
500 clampLo0% = 1
510 clampHi0% = 10
520 GOSUB 1220
530 PRINT "clamp(-3,1,10) = "; clampResult0%
540 clampValue0% = 7
550 clampLo0% = 1
560 clampHi0% = 10
570 GOSUB 1220
580 PRINT "clamp(7,1,10)  = "; clampResult0%

590 repeatText0$ = "ab"
600 repeatN0% = 4
610 GOSUB 1330
620 PRINT repeatResult0$
630 titlecaseWord0$ = "bASCAL"
640 GOSUB 1420
650 PRINT titlecaseResult0$

660 ' Functions chained in expressions
670 maxA0% = 0
680 maxB0% = -5
690 GOSUB 1020
700 minA0% = maxResult0%
710 minB0% = 100
720 GOSUB 1120
730 lo% = minResult0%
740 PRINT "lo = "; lo%

750 ' Calling the same function twice — each result is captured separately
760 repeatText0$ = "x"
770 repeatN0% = 3
780 GOSUB 1330
790 a$ = repeatResult0$
800 repeatText0$ = "y"
810 repeatN0% = 2
820 GOSUB 1330
830 b$ = repeatResult0$
840 PRINT a$; " "; b$

850 ' Local scoping: sumTo% and productTo% each use i% without conflict
860 sumtoN0% = 5
870 GOSUB 1520
880 PRINT "sumTo(5)     = "; sumtoResult0%
890 producttoN0% = 5
900 GOSUB 1610
910 PRINT "productTo(5) = "; producttoResult0%

920 ' Global variable shared across calls
930 addtototalX0% = 10
940 GOSUB 1700
950 dummy% = addtototalResult0%
960 addtototalX0% = 5
970 GOSUB 1700
980 dummy% = addtototalResult0%
990 PRINT "runningTotal = "; runningtotal%

1000 END

1010 ' function max%(a%, b%)
1020     IF (maxA0% > maxB0%) = 0 THEN GOTO 1060
1030         maxResult0% = maxA0%
1040         RETURN
1050         GOTO 1080
1060         maxResult0% = maxB0%
1070         RETURN
1080     REM END IF
1090     RETURN
1100 ' end function max%

1110 ' function min%(a%, b%)
1120     IF (minA0% < minB0%) = 0 THEN GOTO 1160
1130         minResult0% = minA0%
1140         RETURN
1150         GOTO 1180
1160         minResult0% = minB0%
1170         RETURN
1180     REM END IF
1190     RETURN
1200 ' end function min%

1210 ' function clamp%(value%, lo%, hi%)
1220     ' Constrain value to [lo, hi].
1230     minA0% = clampValue0%
1240     minB0% = clampHi0%
1250     GOSUB 1120
1260     maxA0% = clampLo0%
1270     maxB0% = minResult0%
1280     GOSUB 1020
1290     clampResult0% = maxResult0%
1300     RETURN
1310 ' end function clamp%

1320 ' function repeat$(text$, n%)
1330     ' Concatenate text$ with itself n times.
1340     repeatAcc0$ = ""
1350     FOR repeatI0% = 1 TO repeatN0%
1360         repeatAcc0$ = repeatAcc0$ + repeatText0$
1370     NEXT repeatI0%
1380     repeatResult0$ = repeatAcc0$
1390     RETURN
1400 ' end function repeat$

1410 ' function titlecase$(word$)
1420     ' Capitalise first letter, lowercase remainder.
1430     ' Relies on the BASIC runtime's UCASE$/LCASE$ built-ins.
1440     IF (LEN(titlecaseWord0$) = 0) = 0 THEN GOTO 1470
1450         titlecaseResult0$ = ""
1460         RETURN
1470     REM END IF
1480     titlecaseResult0$ = UCASE$(LEFT$(titlecaseWord0$, 1)) + LCASE$(MID$(titlecaseWord0$, 2))
1490     RETURN
1500 ' end function titlecase$

1510 ' function sumto%(n%)
1520     ' i% and acc% are local to sumTo%.
1530     sumtoAcc0% = 0
1540     FOR sumtoI0% = 1 TO sumtoN0%
1550         sumtoAcc0% = sumtoAcc0% + sumtoI0%
1560     NEXT sumtoI0%
1570     sumtoResult0% = sumtoAcc0%
1580     RETURN
1590 ' end function sumto%

1600 ' function productto%(n%)
1610     ' i% and acc% here are independent of sumTo%'s i% and acc%.
1620     producttoAcc0% = 1
1630     FOR producttoI0% = 1 TO producttoN0%
1640         producttoAcc0% = producttoAcc0% * producttoI0%
1650     NEXT producttoI0%
1660     producttoResult0% = producttoAcc0%
1670     RETURN
1680 ' end function productto%

1690 ' function addtototal%(x%)
1700     runningtotal% = runningtotal% + addtototalX0%
1710     addtototalResult0% = runningtotal%
1720     RETURN
1730 ' end function addtototal%
