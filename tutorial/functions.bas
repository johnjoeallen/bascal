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
510 BCCT1$ = "max(4, 9) = "
520 maxA0% = 4
530 maxB0% = 9
540 GOSUB 1780
550 BCCT2% = maxResult0%
560 BCCT3% = BCCT2%
570 PRINT BCCT1$; BCCT3%
580 ' 9
590 BCCT4$ = "min(4, 9) = "
600 minA0% = 4
610 minB0% = 9
620 GOSUB 1880
630 BCCT5% = minResult0%
640 BCCT6% = BCCT5%
650 PRINT BCCT4$; BCCT6%
660 ' 4
670 BCCT7$ = "clamp(15,1,10) = "
680 clampValue0% = 15
690 clampLo0% = 1
700 clampHi0% = 10
710 GOSUB 1980
720 BCCT8% = clampResult0%
730 BCCT9% = BCCT8%
740 PRINT BCCT7$; BCCT9%
750 ' 10
760 BCCT10$ = "clamp(-3,1,10) = "
770 clampValue0% = -3
780 clampLo0% = 1
790 clampHi0% = 10
800 GOSUB 1980
810 BCCT11% = clampResult0%
820 BCCT12% = BCCT11%
830 PRINT BCCT10$; BCCT12%
840 ' 1
850 BCCT13$ = "clamp(7,1,10)  = "
860 clampValue0% = 7
870 clampLo0% = 1
880 clampHi0% = 10
890 GOSUB 1980
900 BCCT14% = clampResult0%
910 BCCT15% = BCCT14%
920 PRINT BCCT13$; BCCT15%
930 ' 7
940 repeatText0$ = "ab"
950 repeatN0% = 4
960 GOSUB 2120
970 PRINT repeatResult0$
980 ' abababab
990 titlecaseWord0$ = "bASCAL"
1000 GOSUB 2210
1010 PRINT titlecaseResult0$
1020 ' Bascal
1030 ' Functions chained in expressions
1040 maxA0% = 0
1050 maxB0% = -5
1060 GOSUB 1780
1070 BCCT16% = maxResult0%
1080 BCCT17% = BCCT16%
1090 minA0% = BCCT17%
1100 minB0% = 100
1110 GOSUB 1880
1120 lo% = minResult0%
1130 ' max(0,-5)=0, min(0,100)=0
1140 PRINT "lo = "; lo%
1150 ' Calling the same function twice — each result is captured separately
1160 repeatText0$ = "x"
1170 repeatN0% = 3
1180 GOSUB 2120
1190 a$ = repeatResult0$
1200 repeatText0$ = "y"
1210 repeatN0% = 2
1220 GOSUB 2120
1230 b$ = repeatResult0$
1240 PRINT a$; " "; b$
1250 ' xxx yy
1260 ' Local scoping: sumTo% and productTo% each use i% without conflict
1270 BCCT18$ = "sumTo(5)     = "
1280 sumtoN0% = 5
1290 GOSUB 2450
1300 BCCT19% = sumtoResult0%
1310 BCCT20% = BCCT19%
1320 PRINT BCCT18$; BCCT20%
1330 ' 15
1340 BCCT21$ = "productTo(5) = "
1350 producttoN0% = 5
1360 GOSUB 2540
1370 BCCT22% = producttoResult0%
1380 BCCT23% = BCCT22%
1390 PRINT BCCT21$; BCCT23%
1400 ' 120
1410 ' Global variable shared across calls
1420 addtototalX0% = 10
1430 GOSUB 2630
1440 dummy% = addtototalResult0%
1450 addtototalX0% = 5
1460 GOSUB 2630
1470 dummy% = addtototalResult0%
1480 PRINT "runningTotal = "; runningtotal%
1490 ' 15
1500 END

1510 ' function ucase$(self$)
1520     ucaseOut0$ = ""
1530     FOR ucaseI0% = 1 TO LEN(ucaseSelf0$)
1540         ucaseC0% = ASC(MID$(ucaseSelf0$, ucaseI0%, 1))
1550         IF (ucaseC0% >= 97) = 0 THEN GOTO 1580
1560         IF (ucaseC0% <= 122) = 0 THEN GOTO 1580
1570             ucaseC0% = ucaseC0% - 32
1580         REM END IF
1590         ucaseOut0$ = ucaseOut0$ + CHR$(ucaseC0%)
1600     NEXT ucaseI0%
1610     ucaseResult0$ = ucaseOut0$
1620     RETURN
1630 ' end function ucase$

1640 ' function lcase$(self$)
1650     lcaseOut0$ = ""
1660     FOR lcaseI0% = 1 TO LEN(lcaseSelf0$)
1670         lcaseC0% = ASC(MID$(lcaseSelf0$, lcaseI0%, 1))
1680         IF (lcaseC0% >= 65) = 0 THEN GOTO 1710
1690         IF (lcaseC0% <= 90) = 0 THEN GOTO 1710
1700             lcaseC0% = lcaseC0% + 32
1710         REM END IF
1720         lcaseOut0$ = lcaseOut0$ + CHR$(lcaseC0%)
1730     NEXT lcaseI0%
1740     lcaseResult0$ = lcaseOut0$
1750     RETURN
1760 ' end function lcase$

1770 ' function max%(a%, b%)
1780     IF (maxA0% > maxB0%) = 0 THEN GOTO 1820
1790         maxResult0% = maxA0%
1800         RETURN
1810     GOTO 1840
1820         maxResult0% = maxB0%
1830         RETURN
1840     REM END IF
1850     RETURN
1860 ' end function max%

1870 ' function min%(a%, b%)
1880     IF (minA0% < minB0%) = 0 THEN GOTO 1920
1890         minResult0% = minA0%
1900         RETURN
1910     GOTO 1940
1920         minResult0% = minB0%
1930         RETURN
1940     REM END IF
1950     RETURN
1960 ' end function min%

1970 ' function clamp%(value%, lo%, hi%)
1980     ' Constrain value to [lo, hi].
1990     minA0% = clampValue0%
2000     minB0% = clampHi0%
2010     GOSUB 1880
2020     BCCT32% = minResult0%
2030     BCCT33% = BCCT32%
2040     maxA0% = clampLo0%
2050     maxB0% = BCCT33%
2060     GOSUB 1780
2070     BCCT34% = maxResult0%
2080     clampResult0% = BCCT34%
2090     RETURN
2100 ' end function clamp%

2110 ' function repeat$(text$, n%)
2120     ' Concatenate text$ with itself n times.
2130     repeatAcc0$ = ""
2140     FOR repeatI0% = 1 TO repeatN0%
2150         repeatAcc0$ = repeatAcc0$ + repeatText0$
2160     NEXT repeatI0%
2170     repeatResult0$ = repeatAcc0$
2180     RETURN
2190 ' end function repeat$

2200 ' function titlecase$(word$)
2210     ' Capitalise first letter, lowercase remainder.
2220     ' UCASE$/LCASE$ aren't real MBASIC/BASCOM 2.00 builtins (verified
2230     ' against a real IBM BASIC Compiler 2.00 under dosbox-x), so this
2240     ' requires BASCAL's own com.bascal.stdlib implementations above.
2250     IF (LEN(titlecaseWord0$) = 0) = 0 THEN GOTO 2280
2260         titlecaseResult0$ = ""
2270         RETURN
2280     REM END IF
2290     BCCT37$ = titlecaseWord0$
2300     BCCT38% = 1
2310     BCCT39$ = LEFT$(BCCT37$, BCCT38%)
2320     ucaseSelf0$ = BCCT39$
2330     GOSUB 1520
2340     BCCT40$ = ucaseResult0$
2350     BCCT41$ = titlecaseWord0$
2360     BCCT42% = 2
2370     BCCT43$ = MID$(BCCT41$, BCCT42%)
2380     lcaseSelf0$ = BCCT43$
2390     GOSUB 1650
2400     BCCT44$ = lcaseResult0$
2410     titlecaseResult0$ = BCCT40$ + BCCT44$
2420     RETURN
2430 ' end function titlecase$

2440 ' function sumto%(n%)
2450     ' i% and acc% are local to sumTo%.
2460     sumtoAcc0% = 0
2470     FOR sumtoI0% = 1 TO sumtoN0%
2480         sumtoAcc0% = sumtoAcc0% + sumtoI0%
2490     NEXT sumtoI0%
2500     sumtoResult0% = sumtoAcc0%
2510     RETURN
2520 ' end function sumto%

2530 ' function productto%(n%)
2540     ' i% and acc% here are independent of sumTo%'s i% and acc%.
2550     producttoAcc0% = 1
2560     FOR producttoI0% = 1 TO producttoN0%
2570         producttoAcc0% = producttoAcc0% * producttoI0%
2580     NEXT producttoI0%
2590     producttoResult0% = producttoAcc0%
2600     RETURN
2610 ' end function productto%

2620 ' function addtototal%(x%)
2630     runningtotal% = runningtotal% + addtototalX0%
2640     addtototalResult0% = runningtotal%
2650     RETURN
2660 ' end function addtototal%
