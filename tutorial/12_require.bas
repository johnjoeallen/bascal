10 ' BASCAL generated BASIC
20 ' Functions are lowered to global variables, labels, and GOSUB

30 ' stats.bcl — basic statistics library for the BASCAL tutorial.
40 ' Loaded by tutorial/12_require.bcl via:
50 ' require stats
60 ' 
70 ' Provides: mean!, maximum%, minimum%, rangeOf%

80 ' Tutorial 12 — REQUIRE and multi-file projects
90 ' 
100 ' REQUIRE loads another .bcl file and merges its functions into the
110 ' generated output.  The path is dot-separated and maps to a file:
120 ' 
130 ' require stats   →  stats.bcl  (in the same directory or a -L path)
140 ' require com.bascal.sort.bubbleSort
150 ' →  com/bascal/sort/bubbleSort.bcl
160 ' 
170 ' All required functions become part of the single generated .bas file.
180 ' The original require line is preserved as a comment in the output.
190 ' 
200 ' Run with:
210 ' bcc tutorial/12_require.bcl -L tutorial/lib
220 ' 
230 ' The -L flag adds tutorial/lib/ to the search path so that
240 ' require stats   resolves to  tutorial/lib/stats.bcl

250 CONST n% = 8
260 DIM scores%(n%)

270 scores%(0) = 74
280 scores%(1) = 91
290 scores%(2) = 63
300 scores%(3) = 88
310 scores%(4) = 55
320 scores%(5) = 97
330 scores%(6) = 72
340 scores%(7) = 84

350 PRINT "Scores: 74 91 63 88 55 97 72 84"
360 mean_count_0% = n%
370 DIM mean_data_0%(n%)

380 ' copy array argument into lowered function storage: scores%() -> mean_data_0%()
390 FOR BCC_T1% = 1 TO n%
400     mean_data_0%(BCC_T1%) = scores%(BCC_T1%)
410 NEXT BCC_T1%

420 GOSUB 860

430 ' copy mutated array argument back to caller storage: mean_data_0%() -> scores%()
440 FOR BCC_T2% = 1 TO n%
450     scores%(BCC_T2%) = mean_data_0%(BCC_T2%)
460 NEXT BCC_T2%

470 PRINT "Mean:   " + STR$(mean_result_0!)
480 maximum_count_0% = n%
490 DIM maximum_data_0%(n%)

500 ' copy array argument into lowered function storage: scores%() -> maximum_data_0%()
510 FOR BCC_T3% = 1 TO n%
520     maximum_data_0%(BCC_T3%) = scores%(BCC_T3%)
530 NEXT BCC_T3%

540 GOSUB 950

550 ' copy mutated array argument back to caller storage: maximum_data_0%() -> scores%()
560 FOR BCC_T4% = 1 TO n%
570     scores%(BCC_T4%) = maximum_data_0%(BCC_T4%)
580 NEXT BCC_T4%

590 PRINT "Max:    " + STR$(maximum_result_0%)
600 minimum_count_0% = n%
610 DIM minimum_data_0%(n%)

620 ' copy array argument into lowered function storage: scores%() -> minimum_data_0%()
630 FOR BCC_T5% = 1 TO n%
640     minimum_data_0%(BCC_T5%) = scores%(BCC_T5%)
650 NEXT BCC_T5%

660 GOSUB 1060

670 ' copy mutated array argument back to caller storage: minimum_data_0%() -> scores%()
680 FOR BCC_T6% = 1 TO n%
690     scores%(BCC_T6%) = minimum_data_0%(BCC_T6%)
700 NEXT BCC_T6%

710 PRINT "Min:    " + STR$(minimum_result_0%)
720 rangeof_count_0% = n%
730 DIM rangeof_data_0%(n%)

740 ' copy array argument into lowered function storage: scores%() -> rangeof_data_0%()
750 FOR BCC_T7% = 1 TO n%
760     rangeof_data_0%(BCC_T7%) = scores%(BCC_T7%)
770 NEXT BCC_T7%

780 GOSUB 1170

790 ' copy mutated array argument back to caller storage: rangeof_data_0%() -> scores%()
800 FOR BCC_T8% = 1 TO n%
810     scores%(BCC_T8%) = rangeof_data_0%(BCC_T8%)
820 NEXT BCC_T8%

830 PRINT "Range:  " + STR$(rangeof_result_0%)

840 END

850 ' function mean!(data%, count%)
860     ' Arithmetic mean of data%(0..count%-1).
870     mean_sum_0% = 0
880     FOR mean_i_0% = 0 TO mean_count_0% - 1
890         mean_sum_0% = mean_sum_0% + mean_data_0%(mean_i_0%)
900     NEXT mean_i_0%
910     mean_result_0! = mean_sum_0% / mean_count_0%
920     RETURN
930 ' end function mean!

940 ' function maximum%(data%, count%)
950     ' Largest element in data%(0..count%-1).
960     maximum_best_0% = maximum_data_0%(0)
970     FOR maximum_i_0% = 1 TO maximum_count_0% - 1
980         IF (maximum_data_0%(maximum_i_0%) > maximum_best_0%) = 0 THEN GOTO 1000
990             maximum_best_0% = maximum_data_0%(maximum_i_0%)
1000         REM END IF
1010     NEXT maximum_i_0%
1020     maximum_result_0% = maximum_best_0%
1030     RETURN
1040 ' end function maximum%

1050 ' function minimum%(data%, count%)
1060     ' Smallest element in data%(0..count%-1).
1070     minimum_best_0% = minimum_data_0%(0)
1080     FOR minimum_i_0% = 1 TO minimum_count_0% - 1
1090         IF (minimum_data_0%(minimum_i_0%) < minimum_best_0%) = 0 THEN GOTO 1110
1100             minimum_best_0% = minimum_data_0%(minimum_i_0%)
1110         REM END IF
1120     NEXT minimum_i_0%
1130     minimum_result_0% = minimum_best_0%
1140     RETURN
1150 ' end function minimum%

1160 ' function rangeof%(data%, count%)
1170     ' Difference between maximum and minimum.
1180     maximum_count_0% = rangeof_count_0%
1190     DIM maximum_data_0%(rangeof_count_0%)

1200     ' copy array argument into lowered function storage: rangeof_data_0%() -> maximum_data_0%()
1210     FOR BCC_T11% = 1 TO rangeof_count_0%
1220         maximum_data_0%(BCC_T11%) = rangeof_data_0%(BCC_T11%)
1230     NEXT BCC_T11%

1240     GOSUB 950

1250     ' copy mutated array argument back to caller storage: maximum_data_0%() -> rangeof_data_0%()
1260     FOR BCC_T12% = 1 TO rangeof_count_0%
1270         rangeof_data_0%(BCC_T12%) = maximum_data_0%(BCC_T12%)
1280     NEXT BCC_T12%

1290     minimum_count_0% = rangeof_count_0%
1300     DIM minimum_data_0%(rangeof_count_0%)

1310     ' copy array argument into lowered function storage: rangeof_data_0%() -> minimum_data_0%()
1320     FOR BCC_T13% = 1 TO rangeof_count_0%
1330         minimum_data_0%(BCC_T13%) = rangeof_data_0%(BCC_T13%)
1340     NEXT BCC_T13%

1350     GOSUB 1060

1360     ' copy mutated array argument back to caller storage: minimum_data_0%() -> rangeof_data_0%()
1370     FOR BCC_T14% = 1 TO rangeof_count_0%
1380         rangeof_data_0%(BCC_T14%) = minimum_data_0%(BCC_T14%)
1390     NEXT BCC_T14%

1400     rangeof_result_0% = maximum_result_0% - minimum_result_0%
1410     RETURN
1420 ' end function rangeof%
