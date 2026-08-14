10 ' BASCAL generated BASIC
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' stats.bcl — basic statistics library for the BASCAL tutorial.
40 ' Loaded by tutorial/12_require.bcl via:
50 ' require stats
60 ' 
70 ' Provides: mean!, maximum%, minimum%, rangeOf%

80 ' data% -- array to average; byval, since mean! only reads it

90 ' data% -- array to search; byval, since maximum% only reads it

100 ' data% -- array to search; byval, since minimum% only reads it

110 ' data% -- array to measure; byval, since rangeOf% only reads it
120 ' Tutorial 12 — REQUIRE and multi-file projects
130 ' 
140 ' REQUIRE loads another .bcl file and merges its functions into the
150 ' generated output.  The path is dot-separated and maps to a file:
160 ' 
170 ' require stats   →  stats.bcl  (in the same directory or a -L path)
180 ' require com.bascal.sort.bubbleSort
190 ' →  com/bascal/sort/bubbleSort.bcl
200 ' 
210 ' All required functions become part of the single generated .bas file.
220 ' The original require line is preserved as a comment in the output.
230 ' 
240 ' Run with:
250 ' bcc tutorial/12_require.bcl -L tutorial/lib
260 ' 
270 ' The -L flag adds tutorial/lib/ to the search path so that
280 ' require stats   resolves to  tutorial/lib/stats.bcl

290 CONST n% = 8
300 DIM scores%(n%)
310 BCC_T1% = n%

320 scores%(0) = 74
330 scores%(1) = 91
340 scores%(2) = 63
350 scores%(3) = 88
360 scores%(4) = 55
370 scores%(5) = 97
380 scores%(6) = 72
390 scores%(7) = 84

400 PRINT "Scores: 74 91 63 88 55 97 72 84"
410 mean_data_dim0_0% = BCC_T1%
420 DIM mean_data_0%(mean_data_dim0_0%)

430 ' copy array argument into transpiled function storage: scores%() -> mean_data_0%()
440 FOR BCC_T2% = 1 TO mean_data_dim0_0%
450     mean_data_0%(BCC_T2%) = scores%(BCC_T2%)
460 NEXT BCC_T2%

470 GOSUB 750
480 PRINT "Mean:   " + STR$(mean_result_0!)
490 maximum_data_dim0_0% = BCC_T1%
500 DIM maximum_data_0%(maximum_data_dim0_0%)

510 ' copy array argument into transpiled function storage: scores%() -> maximum_data_0%()
520 FOR BCC_T3% = 1 TO maximum_data_dim0_0%
530     maximum_data_0%(BCC_T3%) = scores%(BCC_T3%)
540 NEXT BCC_T3%

550 GOSUB 850
560 PRINT "Max:    " + STR$(maximum_result_0%)
570 minimum_data_dim0_0% = BCC_T1%
580 DIM minimum_data_0%(minimum_data_dim0_0%)

590 ' copy array argument into transpiled function storage: scores%() -> minimum_data_0%()
600 FOR BCC_T4% = 1 TO minimum_data_dim0_0%
610     minimum_data_0%(BCC_T4%) = scores%(BCC_T4%)
620 NEXT BCC_T4%

630 GOSUB 960
640 PRINT "Min:    " + STR$(minimum_result_0%)
650 rangeof_data_dim0_0% = BCC_T1%
660 DIM rangeof_data_0%(rangeof_data_dim0_0%)

670 ' copy array argument into transpiled function storage: scores%() -> rangeof_data_0%()
680 FOR BCC_T5% = 1 TO rangeof_data_dim0_0%
690     rangeof_data_0%(BCC_T5%) = scores%(BCC_T5%)
700 NEXT BCC_T5%

710 GOSUB 1070
720 PRINT "Range:  " + STR$(rangeof_result_0%)

730 END

740 ' function mean!(data%)
750     ' Arithmetic mean of data%(0..sizeof(data%)-1).
760     mean_sum_0% = 0
770     mean_count_0% = mean_data_dim0_0%
780     FOR mean_i_0% = 0 TO mean_count_0% - 1
790         mean_sum_0% = mean_sum_0% + mean_data_0%(mean_i_0%)
800     NEXT mean_i_0%
810     mean_result_0! = mean_sum_0% / mean_count_0%
820     RETURN
830 ' end function mean!

840 ' function maximum%(data%)
850     ' Largest element in data%(0..sizeof(data%)-1).
860     maximum_best_0% = maximum_data_0%(0)
870     FOR maximum_i_0% = 1 TO maximum_data_dim0_0% - 1
880         IF (maximum_data_0%(maximum_i_0%) > maximum_best_0%) = 0 THEN GOTO 900
890             maximum_best_0% = maximum_data_0%(maximum_i_0%)
900         REM END IF
910     NEXT maximum_i_0%
920     maximum_result_0% = maximum_best_0%
930     RETURN
940 ' end function maximum%

950 ' function minimum%(data%)
960     ' Smallest element in data%(0..sizeof(data%)-1).
970     minimum_best_0% = minimum_data_0%(0)
980     FOR minimum_i_0% = 1 TO minimum_data_dim0_0% - 1
990         IF (minimum_data_0%(minimum_i_0%) < minimum_best_0%) = 0 THEN GOTO 1010
1000             minimum_best_0% = minimum_data_0%(minimum_i_0%)
1010         REM END IF
1020     NEXT minimum_i_0%
1030     minimum_result_0% = minimum_best_0%
1040     RETURN
1050 ' end function minimum%

1060 ' function rangeof%(data%)
1070     ' Difference between maximum and minimum.
1080     maximum_data_dim0_0% = rangeof_data_dim0_0%
1090     DIM maximum_data_0%(maximum_data_dim0_0%)

1100     ' copy array argument into transpiled function storage: rangeof_data_0%() -> maximum_data_0%()
1110     FOR BCC_T8% = 1 TO maximum_data_dim0_0%
1120         maximum_data_0%(BCC_T8%) = rangeof_data_0%(BCC_T8%)
1130     NEXT BCC_T8%

1140     GOSUB 850
1150     minimum_data_dim0_0% = rangeof_data_dim0_0%
1160     DIM minimum_data_0%(minimum_data_dim0_0%)

1170     ' copy array argument into transpiled function storage: rangeof_data_0%() -> minimum_data_0%()
1180     FOR BCC_T9% = 1 TO minimum_data_dim0_0%
1190         minimum_data_0%(BCC_T9%) = rangeof_data_0%(BCC_T9%)
1200     NEXT BCC_T9%

1210     GOSUB 960
1220     rangeof_result_0% = maximum_result_0% - minimum_result_0%
1230     RETURN
1240 ' end function rangeof%
