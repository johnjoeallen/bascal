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

210 ' print mixes string labels and numeric results directly with ;
220 max_a_0% = 4
230 max_b_0% = 9
240 GOSUB 880
250 PRINT "max(4, 9) = "; max_result_0%
260 min_a_0% = 4
270 min_b_0% = 9
280 GOSUB 980
290 PRINT "min(4, 9) = "; min_result_0%
300 clamp_value_0% = 15
310 clamp_lo_0% = 1
320 clamp_hi_0% = 10
330 GOSUB 1080
340 PRINT "clamp(15,1,10) = "; clamp_result_0%
350 clamp_value_0% = -3
360 clamp_lo_0% = 1
370 clamp_hi_0% = 10
380 GOSUB 1080
390 PRINT "clamp(-3,1,10) = "; clamp_result_0%
400 clamp_value_0% = 7
410 clamp_lo_0% = 1
420 clamp_hi_0% = 10
430 GOSUB 1080
440 PRINT "clamp(7,1,10)  = "; clamp_result_0%

450 repeat_text_0$ = "ab"
460 repeat_n_0% = 4
470 GOSUB 1190
480 PRINT repeat_result_0$
490 titlecase_word_0$ = "bASCAL"
500 GOSUB 1280
510 PRINT titlecase_result_0$

520 ' Functions chained in expressions
530 max_a_0% = 0
540 max_b_0% = -5
550 GOSUB 880
560 min_a_0% = max_result_0%
570 min_b_0% = 100
580 GOSUB 980
590 lo% = min_result_0%
600 PRINT "lo = "; lo%

610 ' Calling the same function twice — each result is captured separately
620 repeat_text_0$ = "x"
630 repeat_n_0% = 3
640 GOSUB 1190
650 a$ = repeat_result_0$
660 repeat_text_0$ = "y"
670 repeat_n_0% = 2
680 GOSUB 1190
690 b$ = repeat_result_0$
700 PRINT a$; " "; b$

710 ' Local scoping: sumTo% and productTo% each use i% without conflict
720 sumto_n_0% = 5
730 GOSUB 1380
740 PRINT "sumTo(5)     = "; sumto_result_0%
750 productto_n_0% = 5
760 GOSUB 1470
770 PRINT "productTo(5) = "; productto_result_0%

780 ' Global variable shared across calls
790 addtototal_x_0% = 10
800 GOSUB 1560
810 dummy% = addtototal_result_0%
820 addtototal_x_0% = 5
830 GOSUB 1560
840 dummy% = addtototal_result_0%
850 PRINT "runningTotal = "; runningtotal%

860 END

870 ' function max%(a%, b%)
880     IF (max_a_0% > max_b_0%) = 0 THEN GOTO 920
890         max_result_0% = max_a_0%
900         RETURN
910         GOTO 940
920         max_result_0% = max_b_0%
930         RETURN
940     REM END IF
950     RETURN
960 ' end function max%

970 ' function min%(a%, b%)
980     IF (min_a_0% < min_b_0%) = 0 THEN GOTO 1020
990         min_result_0% = min_a_0%
1000         RETURN
1010         GOTO 1040
1020         min_result_0% = min_b_0%
1030         RETURN
1040     REM END IF
1050     RETURN
1060 ' end function min%

1070 ' function clamp%(value%, lo%, hi%)
1080     ' Constrain value to [lo, hi].
1090     min_a_0% = clamp_value_0%
1100     min_b_0% = clamp_hi_0%
1110     GOSUB 980
1120     max_a_0% = clamp_lo_0%
1130     max_b_0% = min_result_0%
1140     GOSUB 880
1150     clamp_result_0% = max_result_0%
1160     RETURN
1170 ' end function clamp%

1180 ' function repeat$(text$, n%)
1190     ' Concatenate text$ with itself n times.
1200     repeat_acc_0$ = ""
1210     FOR repeat_i_0% = 1 TO repeat_n_0%
1220         repeat_acc_0$ = repeat_acc_0$ + repeat_text_0$
1230     NEXT repeat_i_0%
1240     repeat_result_0$ = repeat_acc_0$
1250     RETURN
1260 ' end function repeat$

1270 ' function titlecase$(word$)
1280     ' Capitalise first letter, lowercase remainder.
1290     ' Relies on the BASIC runtime's UCASE$/LCASE$ built-ins.
1300     IF (LEN(titlecase_word_0$) = 0) = 0 THEN GOTO 1330
1310         titlecase_result_0$ = ""
1320         RETURN
1330     REM END IF
1340     titlecase_result_0$ = UCASE$(LEFT$(titlecase_word_0$, 1)) + LCASE$(MID$(titlecase_word_0$, 2))
1350     RETURN
1360 ' end function titlecase$

1370 ' function sumto%(n%)
1380     ' i% and acc% are local to sumTo%.
1390     sumto_acc_0% = 0
1400     FOR sumto_i_0% = 1 TO sumto_n_0%
1410         sumto_acc_0% = sumto_acc_0% + sumto_i_0%
1420     NEXT sumto_i_0%
1430     sumto_result_0% = sumto_acc_0%
1440     RETURN
1450 ' end function sumto%

1460 ' function productto%(n%)
1470     ' i% and acc% here are independent of sumTo%'s i% and acc%.
1480     productto_acc_0% = 1
1490     FOR productto_i_0% = 1 TO productto_n_0%
1500         productto_acc_0% = productto_acc_0% * productto_i_0%
1510     NEXT productto_i_0%
1520     productto_result_0% = productto_acc_0%
1530     RETURN
1540 ' end function productto%

1550 ' function addtototal%(x%)
1560     runningtotal% = runningtotal% + addtototal_x_0%
1570     addtototal_result_0% = runningtotal%
1580     RETURN
1590 ' end function addtototal%
