10 ' BASCAL generated BASIC
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Storage for array parameters, sized to fit every call site
40 DIM mean_data_0%(8)
50 DIM maximum_data_0%(8)
60 DIM minimum_data_0%(8)
70 DIM rangeof_data_0%(8)

80 ' stats.bcl — basic statistics library for the BASCAL tutorial.
90 ' Loaded by tutorial/12_require.bcl via:
100 ' require stats
110 ' 
120 ' Provides: mean!, maximum%, minimum%, rangeOf%

130 ' data% -- array to average; byval, since mean! only reads it

140 ' data% -- array to search; byval, since maximum% only reads it

150 ' data% -- array to search; byval, since minimum% only reads it

160 ' data% -- array to measure; byval, since rangeOf% only reads it
170 ' Tutorial 12 — REQUIRE and multi-file projects
180 ' 
190 ' REQUIRE loads another .bcl file and merges its functions into the
200 ' generated output.  The path is dot-separated and maps to a file:
210 ' 
220 ' require stats   →  stats.bcl  (in the same directory or a -L path)
230 ' require com.bascal.sort.bubbleSort
240 ' →  com/bascal/sort/bubbleSort.bcl
250 ' 
260 ' All required functions become part of the single generated .bas file.
270 ' The original require line is preserved as a comment in the output.
280 ' 
290 ' Run with:
300 ' bcc tutorial/12_require.bcl -L tutorial/lib
310 ' 
320 ' The -L flag adds tutorial/lib/ to the search path so that
330 ' require stats   resolves to  tutorial/lib/stats.bcl

340 CONST n% = 8
350 DIM scores%(n%)
360 BCC_T1% = n%

370 scores%(0) = 74
380 scores%(1) = 91
390 scores%(2) = 63
400 scores%(3) = 88
410 scores%(4) = 55
420 scores%(5) = 97
430 scores%(6) = 72
440 scores%(7) = 84

450 PRINT "Scores: 74 91 63 88 55 97 72 84"
460 mean_data_dim0_0% = BCC_T1%
470 IF mean_data_dim0_0% > 8 THEN PRINT "runtime error: `data%` of `mean!` needs "; mean_data_dim0_0%; " elements along axis 0, but its storage only holds 8" : STOP

480 ' copy array argument into transpiled function storage: scores%() -> mean_data_0%()
490 FOR BCC_T2% = 1 TO mean_data_dim0_0%
500     mean_data_0%(BCC_T2%) = scores%(BCC_T2%)
510 NEXT BCC_T2%

520 GOSUB 800
530 PRINT "Mean:   " + STR$(mean_result_0!)
540 maximum_data_dim0_0% = BCC_T1%
550 IF maximum_data_dim0_0% > 8 THEN PRINT "runtime error: `data%` of `maximum%` needs "; maximum_data_dim0_0%; " elements along axis 0, but its storage only holds 8" : STOP

560 ' copy array argument into transpiled function storage: scores%() -> maximum_data_0%()
570 FOR BCC_T3% = 1 TO maximum_data_dim0_0%
580     maximum_data_0%(BCC_T3%) = scores%(BCC_T3%)
590 NEXT BCC_T3%

600 GOSUB 900
610 PRINT "Max:    " + STR$(maximum_result_0%)
620 minimum_data_dim0_0% = BCC_T1%
630 IF minimum_data_dim0_0% > 8 THEN PRINT "runtime error: `data%` of `minimum%` needs "; minimum_data_dim0_0%; " elements along axis 0, but its storage only holds 8" : STOP

640 ' copy array argument into transpiled function storage: scores%() -> minimum_data_0%()
650 FOR BCC_T4% = 1 TO minimum_data_dim0_0%
660     minimum_data_0%(BCC_T4%) = scores%(BCC_T4%)
670 NEXT BCC_T4%

680 GOSUB 1010
690 PRINT "Min:    " + STR$(minimum_result_0%)
700 rangeof_data_dim0_0% = BCC_T1%
710 IF rangeof_data_dim0_0% > 8 THEN PRINT "runtime error: `data%` of `rangeOf%` needs "; rangeof_data_dim0_0%; " elements along axis 0, but its storage only holds 8" : STOP

720 ' copy array argument into transpiled function storage: scores%() -> rangeof_data_0%()
730 FOR BCC_T5% = 1 TO rangeof_data_dim0_0%
740     rangeof_data_0%(BCC_T5%) = scores%(BCC_T5%)
750 NEXT BCC_T5%

760 GOSUB 1120
770 PRINT "Range:  " + STR$(rangeof_result_0%)

780 END

790 ' function mean!(data%)
800     ' Arithmetic mean of data%(0..sizeof(data%)-1).
810     mean_sum_0% = 0
820     mean_count_0% = mean_data_dim0_0%
830     FOR mean_i_0% = 0 TO mean_count_0% - 1
840         mean_sum_0% = mean_sum_0% + mean_data_0%(mean_i_0%)
850     NEXT mean_i_0%
860     mean_result_0! = mean_sum_0% / mean_count_0%
870     RETURN
880 ' end function mean!

890 ' function maximum%(data%)
900     ' Largest element in data%(0..sizeof(data%)-1).
910     maximum_best_0% = maximum_data_0%(0)
920     FOR maximum_i_0% = 1 TO maximum_data_dim0_0% - 1
930         IF (maximum_data_0%(maximum_i_0%) > maximum_best_0%) = 0 THEN GOTO 950
940             maximum_best_0% = maximum_data_0%(maximum_i_0%)
950         REM END IF
960     NEXT maximum_i_0%
970     maximum_result_0% = maximum_best_0%
980     RETURN
990 ' end function maximum%

1000 ' function minimum%(data%)
1010     ' Smallest element in data%(0..sizeof(data%)-1).
1020     minimum_best_0% = minimum_data_0%(0)
1030     FOR minimum_i_0% = 1 TO minimum_data_dim0_0% - 1
1040         IF (minimum_data_0%(minimum_i_0%) < minimum_best_0%) = 0 THEN GOTO 1060
1050             minimum_best_0% = minimum_data_0%(minimum_i_0%)
1060         REM END IF
1070     NEXT minimum_i_0%
1080     minimum_result_0% = minimum_best_0%
1090     RETURN
1100 ' end function minimum%

1110 ' function rangeof%(data%)
1120     ' Difference between maximum and minimum.
1130     maximum_data_dim0_0% = rangeof_data_dim0_0%
1140     IF maximum_data_dim0_0% > 8 THEN PRINT "runtime error: `data%` of `maximum%` needs "; maximum_data_dim0_0%; " elements along axis 0, but its storage only holds 8" : STOP

1150     ' copy array argument into transpiled function storage: rangeof_data_0%() -> maximum_data_0%()
1160     FOR BCC_T8% = 1 TO maximum_data_dim0_0%
1170         maximum_data_0%(BCC_T8%) = rangeof_data_0%(BCC_T8%)
1180     NEXT BCC_T8%

1190     GOSUB 900
1200     minimum_data_dim0_0% = rangeof_data_dim0_0%
1210     IF minimum_data_dim0_0% > 8 THEN PRINT "runtime error: `data%` of `minimum%` needs "; minimum_data_dim0_0%; " elements along axis 0, but its storage only holds 8" : STOP

1220     ' copy array argument into transpiled function storage: rangeof_data_0%() -> minimum_data_0%()
1230     FOR BCC_T9% = 1 TO minimum_data_dim0_0%
1240         minimum_data_0%(BCC_T9%) = rangeof_data_0%(BCC_T9%)
1250     NEXT BCC_T9%

1260     GOSUB 1010
1270     rangeof_result_0% = maximum_result_0% - minimum_result_0%
1280     RETURN
1290 ' end function rangeof%
