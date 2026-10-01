10 ' BASCAL generated BASIC -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Storage for array parameters, sized to fit every call site
40 DIM meanData0%(7)
50 DIM maximumData0%(7)
60 DIM minimumData0%(7)
70 DIM rangeofData0%(7)

80 ' stats.bcl — basic statistics library for the BASCAL tutorial.
90 ' Loaded by tutorial/require.bcl via:
100 ' require stats
110 ' 
120 ' Provides: mean!, maximum%, minimum%, rangeOf%
130 ' data% -- array to average; byval, since mean! only reads it
140 ' data% -- array to search; byval, since maximum% only reads it
150 ' data% -- array to search; byval, since minimum% only reads it
160 ' data% -- array to measure; byval, since rangeOf% only reads it
170 ' Tutorial — REQUIRE and multi-file projects
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
300 ' bcc tutorial/require.bcl -L tutorial/lib
310 ' 
320 ' The -L flag adds tutorial/lib/ to the search path so that
330 ' require stats   resolves to  tutorial/lib/stats.bcl
340 CONSTN% = 8
350 DIM scores%(CONSTN% - 1)
360 BCCT1% = CONSTN% - 1
370 scores%(0) = 74
380 scores%(1) = 91
390 scores%(2) = 63
400 scores%(3) = 88
410 scores%(4) = 55
420 scores%(5) = 97
430 scores%(6) = 72
440 scores%(7) = 84
450 PRINT "Scores: 74 91 63 88 55 97 72 84"
460 meanDataDim00% = BCCT1%
470 IF meanDataDim00% > 7 THEN PRINT "runtime error: `data%` of `mean!` needs "; meanDataDim00%; " elements along axis 0, but its storage only holds 7" : STOP

480 ' copy array argument into transpiled function storage: scores%() -> meanData0%()
490 FOR BCCT2% = 0 TO meanDataDim00%
500     meanData0%(BCCT2%) = scores%(BCCT2%)
510 NEXT BCCT2%

520 GOSUB 920
530 BCCT3! = meanResult0!
540 BCCT4! = BCCT3!
550 BCCT5$ = STR$(BCCT4!)
560 PRINT "Mean:   " + BCCT5$
570 maximumDataDim00% = BCCT1%
580 IF maximumDataDim00% > 7 THEN PRINT "runtime error: `data%` of `maximum%` needs "; maximumDataDim00%; " elements along axis 0, but its storage only holds 7" : STOP

590 ' copy array argument into transpiled function storage: scores%() -> maximumData0%()
600 FOR BCCT6% = 0 TO maximumDataDim00%
610     maximumData0%(BCCT6%) = scores%(BCCT6%)
620 NEXT BCCT6%

630 GOSUB 1020
640 BCCT7% = maximumResult0%
650 BCCT8% = BCCT7%
660 BCCT9$ = STR$(BCCT8%)
670 PRINT "Max:    " + BCCT9$
680 minimumDataDim00% = BCCT1%
690 IF minimumDataDim00% > 7 THEN PRINT "runtime error: `data%` of `minimum%` needs "; minimumDataDim00%; " elements along axis 0, but its storage only holds 7" : STOP

700 ' copy array argument into transpiled function storage: scores%() -> minimumData0%()
710 FOR BCCT10% = 0 TO minimumDataDim00%
720     minimumData0%(BCCT10%) = scores%(BCCT10%)
730 NEXT BCCT10%

740 GOSUB 1130
750 BCCT11% = minimumResult0%
760 BCCT12% = BCCT11%
770 BCCT13$ = STR$(BCCT12%)
780 PRINT "Min:    " + BCCT13$
790 rangeofDataDim00% = BCCT1%
800 IF rangeofDataDim00% > 7 THEN PRINT "runtime error: `data%` of `rangeOf%` needs "; rangeofDataDim00%; " elements along axis 0, but its storage only holds 7" : STOP

810 ' copy array argument into transpiled function storage: scores%() -> rangeofData0%()
820 FOR BCCT14% = 0 TO rangeofDataDim00%
830     rangeofData0%(BCCT14%) = scores%(BCCT14%)
840 NEXT BCCT14%

850 GOSUB 1240
860 BCCT15% = rangeofResult0%
870 BCCT16% = BCCT15%
880 BCCT17$ = STR$(BCCT16%)
890 PRINT "Range:  " + BCCT17$
900 END

910 ' function mean!(data%)
920     ' Arithmetic mean of data%(0..sizeof(data%)-1).
930     meanSum0% = 0
940     meanCount0% = (meanDataDim00% + 1)
950     FOR meanI0% = 0 TO meanCount0% - 1
960         meanSum0% = meanSum0% + meanData0%(meanI0%)
970     NEXT meanI0%
980     meanResult0! = meanSum0% / meanCount0%
990     RETURN
1000 ' end function mean!

1010 ' function maximum%(data%)
1020     ' Largest element in data%(0..sizeof(data%)-1).
1030     maximumBest0% = maximumData0%(0)
1040     FOR maximumI0% = 1 TO (maximumDataDim00% + 1) - 1
1050         IF (maximumData0%(maximumI0%) > maximumBest0%) = 0 THEN GOTO 1070
1060             maximumBest0% = maximumData0%(maximumI0%)
1070         REM END IF
1080     NEXT maximumI0%
1090     maximumResult0% = maximumBest0%
1100     RETURN
1110 ' end function maximum%

1120 ' function minimum%(data%)
1130     ' Smallest element in data%(0..sizeof(data%)-1).
1140     minimumBest0% = minimumData0%(0)
1150     FOR minimumI0% = 1 TO (minimumDataDim00% + 1) - 1
1160         IF (minimumData0%(minimumI0%) < minimumBest0%) = 0 THEN GOTO 1180
1170             minimumBest0% = minimumData0%(minimumI0%)
1180         REM END IF
1190     NEXT minimumI0%
1200     minimumResult0% = minimumBest0%
1210     RETURN
1220 ' end function minimum%

1230 ' function rangeof%(data%)
1240     ' Difference between maximum and minimum.
1250     maximumDataDim00% = rangeofDataDim00%
1260     IF maximumDataDim00% > 7 THEN PRINT "runtime error: `data%` of `maximum%` needs "; maximumDataDim00%; " elements along axis 0, but its storage only holds 7" : STOP

1270     ' copy array argument into transpiled function storage: rangeofData0%() -> maximumData0%()
1280     FOR BCCT23% = 0 TO maximumDataDim00%
1290         maximumData0%(BCCT23%) = rangeofData0%(BCCT23%)
1300     NEXT BCCT23%

1310     GOSUB 1020
1320     BCCT24% = maximumResult0%
1330     minimumDataDim00% = rangeofDataDim00%
1340     IF minimumDataDim00% > 7 THEN PRINT "runtime error: `data%` of `minimum%` needs "; minimumDataDim00%; " elements along axis 0, but its storage only holds 7" : STOP

1350     ' copy array argument into transpiled function storage: rangeofData0%() -> minimumData0%()
1360     FOR BCCT25% = 0 TO minimumDataDim00%
1370         minimumData0%(BCCT25%) = rangeofData0%(BCCT25%)
1380     NEXT BCCT25%

1390     GOSUB 1130
1400     BCCT26% = minimumResult0%
1410     rangeofResult0% = BCCT24% - BCCT26%
1420     RETURN
1430 ' end function rangeof%
