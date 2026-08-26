10 ' BASCAL generated BASIC -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
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

130 ' Tutorial — Functions
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
260 ' 
270 ' Scalar methods are typed functions with an implicit receiver.  Calls use
280 ' dot syntax and can chain: word$.left(1).ucase().  The existing titleCase$
290 ' function below demonstrates this form; methods transpile to ordinary
300 ' calls for both targets.

310 ' Integer arithmetic functions
320 ' a% -- first value to compare
330 ' b% -- second value to compare

340 ' a% -- first value to compare
350 ' b% -- second value to compare

360 ' value% -- number to constrain
370 ' lo%    -- lower bound, inclusive
380 ' hi%    -- upper bound, inclusive

390 ' String functions
400 ' text$ -- string to repeat
410 ' n%    -- number of times to repeat it

420 ' word$ -- string to title-case

430 ' Local variable scoping — each function has its own i% and acc%
440 ' n% -- upper bound of the sum, inclusive

450 ' n% -- upper bound of the product, inclusive

460 ' Global variable accessed inside a function with the global keyword
470 runningtotal% = 0

480 ' x% -- amount to add to the running total

490 ' --- Exercise the functions ---

500 ' print mixes string labels and numeric results directly with ;
510 maxA0% = 4
520 maxB0% = 9
530 GOSUB 1430
540 PRINT "max(4, 9) = "; maxResult0%
550 minA0% = 4
560 minB0% = 9
570 GOSUB 1530
580 PRINT "min(4, 9) = "; minResult0%
590 clampValue0% = 15
600 clampLo0% = 1
610 clampHi0% = 10
620 GOSUB 1630
630 PRINT "clamp(15,1,10) = "; clampResult0%
640 clampValue0% = -3
650 clampLo0% = 1
660 clampHi0% = 10
670 GOSUB 1630
680 PRINT "clamp(-3,1,10) = "; clampResult0%
690 clampValue0% = 7
700 clampLo0% = 1
710 clampHi0% = 10
720 GOSUB 1630
730 PRINT "clamp(7,1,10)  = "; clampResult0%

740 repeatText0$ = "ab"
750 repeatN0% = 4
760 GOSUB 1740
770 PRINT repeatResult0$
780 titlecaseWord0$ = "bASCAL"
790 GOSUB 1830
800 PRINT titlecaseResult0$

810 ' Functions chained in expressions
820 maxA0% = 0
830 maxB0% = -5
840 GOSUB 1430
850 minA0% = maxResult0%
860 minB0% = 100
870 GOSUB 1530
880 lo% = minResult0%
890 PRINT "lo = "; lo%

900 ' Calling the same function twice — each result is captured separately
910 repeatText0$ = "x"
920 repeatN0% = 3
930 GOSUB 1740
940 a$ = repeatResult0$
950 repeatText0$ = "y"
960 repeatN0% = 2
970 GOSUB 1740
980 b$ = repeatResult0$
990 PRINT a$; " "; b$

1000 ' Local scoping: sumTo% and productTo% each use i% without conflict
1010 sumtoN0% = 5
1020 GOSUB 1990
1030 PRINT "sumTo(5)     = "; sumtoResult0%
1040 producttoN0% = 5
1050 GOSUB 2080
1060 PRINT "productTo(5) = "; producttoResult0%

1070 ' Global variable shared across calls
1080 addtototalX0% = 10
1090 GOSUB 2170
1100 dummy% = addtototalResult0%
1110 addtototalX0% = 5
1120 GOSUB 2170
1130 dummy% = addtototalResult0%
1140 PRINT "runningTotal = "; runningtotal%

1150 END

1160 ' function ucase$()
1170     ucaseOut0$ = ""
1180     FOR ucaseI0% = 1 TO LEN(ucaseSelf0$)
1190         ucaseC0% = ASC(MID$(ucaseSelf0$, ucaseI0%, 1))
1200         IF (ucaseC0% >= 97) = 0 THEN GOTO 1230
1210         IF (ucaseC0% <= 122) = 0 THEN GOTO 1230
1220             ucaseC0% = ucaseC0% - 32
1230         REM END IF
1240         ucaseOut0$ = ucaseOut0$ + CHR$(ucaseC0%)
1250     NEXT ucaseI0%
1260     ucaseResult0$ = ucaseOut0$
1270     RETURN
1280 ' end function ucase$

1290 ' function lcase$()
1300     lcaseOut0$ = ""
1310     FOR lcaseI0% = 1 TO LEN(lcaseSelf0$)
1320         lcaseC0% = ASC(MID$(lcaseSelf0$, lcaseI0%, 1))
1330         IF (lcaseC0% >= 65) = 0 THEN GOTO 1360
1340         IF (lcaseC0% <= 90) = 0 THEN GOTO 1360
1350             lcaseC0% = lcaseC0% + 32
1360         REM END IF
1370         lcaseOut0$ = lcaseOut0$ + CHR$(lcaseC0%)
1380     NEXT lcaseI0%
1390     lcaseResult0$ = lcaseOut0$
1400     RETURN
1410 ' end function lcase$

1420 ' function max%(a%, b%)
1430     IF (maxA0% > maxB0%) = 0 THEN GOTO 1470
1440         maxResult0% = maxA0%
1450         RETURN
1460         GOTO 1490
1470         maxResult0% = maxB0%
1480         RETURN
1490     REM END IF
1500     RETURN
1510 ' end function max%

1520 ' function min%(a%, b%)
1530     IF (minA0% < minB0%) = 0 THEN GOTO 1570
1540         minResult0% = minA0%
1550         RETURN
1560         GOTO 1590
1570         minResult0% = minB0%
1580         RETURN
1590     REM END IF
1600     RETURN
1610 ' end function min%

1620 ' function clamp%(value%, lo%, hi%)
1630     ' Constrain value to [lo, hi].
1640     minA0% = clampValue0%
1650     minB0% = clampHi0%
1660     GOSUB 1530
1670     maxA0% = clampLo0%
1680     maxB0% = minResult0%
1690     GOSUB 1430
1700     clampResult0% = maxResult0%
1710     RETURN
1720 ' end function clamp%

1730 ' function repeat$(text$, n%)
1740     ' Concatenate text$ with itself n times.
1750     repeatAcc0$ = ""
1760     FOR repeatI0% = 1 TO repeatN0%
1770         repeatAcc0$ = repeatAcc0$ + repeatText0$
1780     NEXT repeatI0%
1790     repeatResult0$ = repeatAcc0$
1800     RETURN
1810 ' end function repeat$

1820 ' function titlecase$(word$)
1830     ' Capitalise first letter, lowercase remainder.
1840     ' UCASE$/LCASE$ aren't real MBASIC/BASCOM 2.00 builtins (verified
1850     ' against a real IBM BASIC Compiler 2.00 under dosbox-x), so this
1860     ' requires BASCAL's own com.bascal.stdlib implementations above.
1870     IF (LEN(titlecaseWord0$) = 0) = 0 THEN GOTO 1900
1880         titlecaseResult0$ = ""
1890         RETURN
1900     REM END IF
1910     ucaseSelf0$ = LEFT$(titlecaseWord0$, 1)
1920     GOSUB 1170
1930     lcaseSelf0$ = MID$(titlecaseWord0$, 2)
1940     GOSUB 1300
1950     titlecaseResult0$ = ucaseResult0$ + lcaseResult0$
1960     RETURN
1970 ' end function titlecase$

1980 ' function sumto%(n%)
1990     ' i% and acc% are local to sumTo%.
2000     sumtoAcc0% = 0
2010     FOR sumtoI0% = 1 TO sumtoN0%
2020         sumtoAcc0% = sumtoAcc0% + sumtoI0%
2030     NEXT sumtoI0%
2040     sumtoResult0% = sumtoAcc0%
2050     RETURN
2060 ' end function sumto%

2070 ' function productto%(n%)
2080     ' i% and acc% here are independent of sumTo%'s i% and acc%.
2090     producttoAcc0% = 1
2100     FOR producttoI0% = 1 TO producttoN0%
2110         producttoAcc0% = producttoAcc0% * producttoI0%
2120     NEXT producttoI0%
2130     producttoResult0% = producttoAcc0%
2140     RETURN
2150 ' end function productto%

2160 ' function addtototal%(x%)
2170     runningtotal% = runningtotal% + addtototalX0%
2180     addtototalResult0% = runningtotal%
2190     RETURN
2200 ' end function addtototal%
