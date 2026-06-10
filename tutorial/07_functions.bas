10 ' BASCAL generated BASIC
20 ' Functions are lowered to global variables, labels, and GOSUB

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
130 ' Functions cannot call themselves recursively (parameters would be
140 ' overwritten).  Use an explicit stack array for recursive algorithms.

150 ' Integer arithmetic functions

160 ' String functions

170 ' Local variable scoping — each function has its own i% and acc%

180 ' Global variable accessed inside a function with the global keyword
190 runningtotal% = 0

200 ' --- Exercise the functions ---

210 max_a% = 4
220 max_b% = 9
230 GOSUB 870
240 PRINT "max(4, 9) = " + STR$(max_result%)
250 min_a% = 4
260 min_b% = 9
270 GOSUB 970
280 PRINT "min(4, 9) = " + STR$(min_result%)
290 clamp_value% = 15
300 clamp_lo% = 1
310 clamp_hi% = 10
320 GOSUB 1070
330 PRINT "clamp(15,1,10) = " + STR$(clamp_result%)
340 clamp_value% = -3
350 clamp_lo% = 1
360 clamp_hi% = 10
370 GOSUB 1070
380 PRINT "clamp(-3,1,10) = " + STR$(clamp_result%)
390 clamp_value% = 7
400 clamp_lo% = 1
410 clamp_hi% = 10
420 GOSUB 1070
430 PRINT "clamp(7,1,10)  = " + STR$(clamp_result%)

440 repeat_text$ = "ab"
450 repeat_n% = 4
460 GOSUB 1180
470 PRINT repeat_result$
480 titlecase_word$ = "bASCAL"
490 GOSUB 1270
500 PRINT titlecase_result$

510 ' Functions chained in expressions
520 max_a% = 0
530 max_b% = -5
540 GOSUB 870
550 min_a% = max_result%
560 min_b% = 100
570 GOSUB 970
580 lo% = min_result%
590 PRINT "lo = " + STR$(lo%)

600 ' Calling the same function twice — each result is captured separately
610 repeat_text$ = "x"
620 repeat_n% = 3
630 GOSUB 1180
640 a$ = repeat_result$
650 repeat_text$ = "y"
660 repeat_n% = 2
670 GOSUB 1180
680 b$ = repeat_result$
690 PRINT (a$ + " ") + b$

700 ' Local scoping: sumTo% and productTo% each use i% without conflict
710 sumto_n% = 5
720 GOSUB 1370
730 PRINT "sumTo(5)     = " + STR$(sumto_result%)
740 productto_n% = 5
750 GOSUB 1460
760 PRINT "productTo(5) = " + STR$(productto_result%)

770 ' Global variable shared across calls
780 addtototal_x% = 10
790 GOSUB 1550
800 dummy% = addtototal_result%
810 addtototal_x% = 5
820 GOSUB 1550
830 dummy% = addtototal_result%
840 PRINT "runningTotal = " + STR$(runningtotal%)

850 END

860 ' function max%(a%, b%)
870     IF (max_a% > max_b%) = 0 THEN GOTO 910
880         max_result% = max_a%
890         RETURN
900         GOTO 930
910         max_result% = max_b%
920         RETURN
930     REM END IF
940     RETURN
950 ' end function max%

960 ' function min%(a%, b%)
970     IF (min_a% < min_b%) = 0 THEN GOTO 1010
980         min_result% = min_a%
990         RETURN
1000         GOTO 1030
1010         min_result% = min_b%
1020         RETURN
1030     REM END IF
1040     RETURN
1050 ' end function min%

1060 ' function clamp%(value%, lo%, hi%)
1070     ' Constrain value to [lo, hi].
1080     min_a% = clamp_value%
1090     min_b% = clamp_hi%
1100     GOSUB 970
1110     max_a% = clamp_lo%
1120     max_b% = min_result%
1130     GOSUB 870
1140     clamp_result% = max_result%
1150     RETURN
1160 ' end function clamp%

1170 ' function repeat$(text$, n%)
1180     ' Concatenate text$ with itself n times.
1190     repeat_acc$ = ""
1200     FOR repeat_i% = 1 TO repeat_n%
1210         repeat_acc$ = repeat_acc$ + repeat_text$
1220     NEXT repeat_i%
1230     repeat_result$ = repeat_acc$
1240     RETURN
1250 ' end function repeat$

1260 ' function titlecase$(word$)
1270     ' Capitalise first letter, lowercase remainder.
1280     ' Relies on the BASIC runtime's UCASE$/LCASE$ built-ins.
1290     IF (LEN(titlecase_word$) = 0) = 0 THEN GOTO 1320
1300         titlecase_result$ = ""
1310         RETURN
1320     REM END IF
1330     titlecase_result$ = UCASE$(LEFT$(titlecase_word$, 1)) + LCASE$(MID$(titlecase_word$, 2))
1340     RETURN
1350 ' end function titlecase$

1360 ' function sumto%(n%)
1370     ' i% and acc% are local to sumTo%.
1380     sumto_acc% = 0
1390     FOR sumto_i% = 1 TO sumto_n%
1400         sumto_acc% = sumto_acc% + sumto_i%
1410     NEXT sumto_i%
1420     sumto_result% = sumto_acc%
1430     RETURN
1440 ' end function sumto%

1450 ' function productto%(n%)
1460     ' i% and acc% here are independent of sumTo%'s i% and acc%.
1470     productto_acc% = 1
1480     FOR productto_i% = 1 TO productto_n%
1490         productto_acc% = productto_acc% * productto_i%
1500     NEXT productto_i%
1510     productto_result% = productto_acc%
1520     RETURN
1530 ' end function productto%

1540 ' function addtototal%(x%)
1550     runningtotal% = runningtotal% + addtototal_x%
1560     addtototal_result% = runningtotal%
1570     RETURN
1580 ' end function addtototal%
