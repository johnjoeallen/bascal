10 ' BASCAL generated BASIC
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' stats.bcl — basic statistics library for the BASCAL tutorial.
40 ' Loaded by tutorial/12_require.bcl via:
50 ' require stats
60 ' 
70 ' Provides: mean!, maximum%, minimum%, rangeOf%

80 ' data%  -- array to average; byval, since mean! only reads it
90 ' count% -- number of elements in data%

100 ' data%  -- array to search; byval, since maximum% only reads it
110 ' count% -- number of elements in data%

120 ' data%  -- array to search; byval, since minimum% only reads it
130 ' count% -- number of elements in data%

140 ' data%  -- array to measure; byval, since rangeOf% only reads it
150 ' count% -- number of elements in data%
160 ' Tutorial 12 — REQUIRE and multi-file projects
170 ' 
180 ' REQUIRE loads another .bcl file and merges its functions into the
190 ' generated output.  The path is dot-separated and maps to a file:
200 ' 
210 ' require stats   →  stats.bcl  (in the same directory or a -L path)
220 ' require com.bascal.sort.bubbleSort
230 ' →  com/bascal/sort/bubbleSort.bcl
240 ' 
250 ' All required functions become part of the single generated .bas file.
260 ' The original require line is preserved as a comment in the output.
270 ' 
280 ' Run with:
290 ' bcc tutorial/12_require.bcl -L tutorial/lib
300 ' 
310 ' The -L flag adds tutorial/lib/ to the search path so that
320 ' require stats   resolves to  tutorial/lib/stats.bcl

330 CONST n% = 8
340 DIM scores%(n%)

350 scores%(0) = 74
360 scores%(1) = 91
370 scores%(2) = 63
380 scores%(3) = 88
390 scores%(4) = 55
400 scores%(5) = 97
410 scores%(6) = 72
420 scores%(7) = 84

430 PRINT "Scores: 74 91 63 88 55 97 72 84"
440 mean_count_0% = n%
450 DIM mean_data_0%(n%)

460 ' copy array argument into transpiled function storage: scores%() -> mean_data_0%()
470 FOR BCC_T1% = 1 TO n%
480     mean_data_0%(BCC_T1%) = scores%(BCC_T1%)
490 NEXT BCC_T1%

500 GOSUB 780
510 PRINT "Mean:   " + STR$(mean_result_0!)
520 maximum_count_0% = n%
530 DIM maximum_data_0%(n%)

540 ' copy array argument into transpiled function storage: scores%() -> maximum_data_0%()
550 FOR BCC_T2% = 1 TO n%
560     maximum_data_0%(BCC_T2%) = scores%(BCC_T2%)
570 NEXT BCC_T2%

580 GOSUB 870
590 PRINT "Max:    " + STR$(maximum_result_0%)
600 minimum_count_0% = n%
610 DIM minimum_data_0%(n%)

620 ' copy array argument into transpiled function storage: scores%() -> minimum_data_0%()
630 FOR BCC_T3% = 1 TO n%
640     minimum_data_0%(BCC_T3%) = scores%(BCC_T3%)
650 NEXT BCC_T3%

660 GOSUB 980
670 PRINT "Min:    " + STR$(minimum_result_0%)
680 rangeof_count_0% = n%
690 DIM rangeof_data_0%(n%)

700 ' copy array argument into transpiled function storage: scores%() -> rangeof_data_0%()
710 FOR BCC_T4% = 1 TO n%
720     rangeof_data_0%(BCC_T4%) = scores%(BCC_T4%)
730 NEXT BCC_T4%

740 GOSUB 1090
750 PRINT "Range:  " + STR$(rangeof_result_0%)

760 END

770 ' function mean!(data%, count%)
780     ' Arithmetic mean of data%(0..count%-1).
790     mean_sum_0% = 0
800     FOR mean_i_0% = 0 TO mean_count_0% - 1
810         mean_sum_0% = mean_sum_0% + mean_data_0%(mean_i_0%)
820     NEXT mean_i_0%
830     mean_result_0! = mean_sum_0% / mean_count_0%
840     RETURN
850 ' end function mean!

860 ' function maximum%(data%, count%)
870     ' Largest element in data%(0..count%-1).
880     maximum_best_0% = maximum_data_0%(0)
890     FOR maximum_i_0% = 1 TO maximum_count_0% - 1
900         IF (maximum_data_0%(maximum_i_0%) > maximum_best_0%) = 0 THEN GOTO 920
910             maximum_best_0% = maximum_data_0%(maximum_i_0%)
920         REM END IF
930     NEXT maximum_i_0%
940     maximum_result_0% = maximum_best_0%
950     RETURN
960 ' end function maximum%

970 ' function minimum%(data%, count%)
980     ' Smallest element in data%(0..count%-1).
990     minimum_best_0% = minimum_data_0%(0)
1000     FOR minimum_i_0% = 1 TO minimum_count_0% - 1
1010         IF (minimum_data_0%(minimum_i_0%) < minimum_best_0%) = 0 THEN GOTO 1030
1020             minimum_best_0% = minimum_data_0%(minimum_i_0%)
1030         REM END IF
1040     NEXT minimum_i_0%
1050     minimum_result_0% = minimum_best_0%
1060     RETURN
1070 ' end function minimum%

1080 ' function rangeof%(data%, count%)
1090     ' Difference between maximum and minimum.
1100     maximum_count_0% = rangeof_count_0%
1110     DIM maximum_data_0%(rangeof_count_0%)

1120     ' copy array argument into transpiled function storage: rangeof_data_0%() -> maximum_data_0%()
1130     FOR BCC_T7% = 1 TO rangeof_count_0%
1140         maximum_data_0%(BCC_T7%) = rangeof_data_0%(BCC_T7%)
1150     NEXT BCC_T7%

1160     GOSUB 870
1170     minimum_count_0% = rangeof_count_0%
1180     DIM minimum_data_0%(rangeof_count_0%)

1190     ' copy array argument into transpiled function storage: rangeof_data_0%() -> minimum_data_0%()
1200     FOR BCC_T8% = 1 TO rangeof_count_0%
1210         minimum_data_0%(BCC_T8%) = rangeof_data_0%(BCC_T8%)
1220     NEXT BCC_T8%

1230     GOSUB 980
1240     rangeof_result_0% = maximum_result_0% - minimum_result_0%
1250     RETURN
1260 ' end function rangeof%
