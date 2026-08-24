[Home](../) / [Tutorials](./) / Require and Multi-File Projects

<div class="prose" markdown="1">

`require` loads another `.bcl` file and merges its functions into the generated output. The path is dot-separated and maps to a file: `require com.bascal.sort.bubbleSort` resolves to `com/bascal/sort/bubbleSort.bcl`. The `-L` flag adds extra search directories, e.g. `bcc tutorial/12_require.bcl -L tutorial/lib` so that `require stats` resolves to `tutorial/lib/stats.bcl`.

</div>

<div class="snippet" markdown="1">

### Requiring a library and calling its functions

```bascal
require stats

print "Mean:   "  + str$(mean!(scores%))
print "Max:    "  + str$(maximum%(scores%))
print "Min:    "  + str$(minimum%(scores%))
```

</div>



[← Screen I/O](11_screen.md)  ·  [Shared COMMON →](13_shared.md)


<!-- BEGIN generated tutorial source -->

### `tutorial/12_require.bcl`

```bascal

// Tutorial — REQUIRE and multi-file projects
//
// REQUIRE loads another .bcl file and merges its functions into the
// generated output.  The path is dot-separated and maps to a file:
//
//   require stats   →  stats.bcl  (in the same directory or a -L path)
//   require com.bascal.sort.bubbleSort
//                   →  com/bascal/sort/bubbleSort.bcl
//
// All required functions become part of the single generated .bas file.
// The original require line is preserved as a comment in the output.
//
// Run with:
//   bcc tutorial/12_require.bcl -L tutorial/lib
//
// The -L flag adds tutorial/lib/ to the search path so that
//   require stats   resolves to  tutorial/lib/stats.bcl
program requireDemo

require stats

const N% = 8
dim scores%(N% - 1)

scores%(0) = 74
scores%(1) = 91
scores%(2) = 63
scores%(3) = 88
scores%(4) = 55
scores%(5) = 97
scores%(6) = 72
scores%(7) = 84

print "Scores: 74 91 63 88 55 97 72 84"
print "Mean:   "  + str$(mean!(scores%))
print "Max:    "  + str$(maximum%(scores%))
print "Min:    "  + str$(minimum%(scores%))
print "Range:  "  + str$(rangeOf%(scores%))

end

```

### `tutorial/12_require.bas`

```basic

10 ' BASCAL generated BASIC
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Storage for array parameters, sized to fit every call site
40 DIM meanData0%(8)
50 DIM maximumData0%(8)
60 DIM minimumData0%(8)
70 DIM rangeofData0%(8)

80 ' stats.bcl — basic statistics library for the BASCAL tutorial.
90 ' Loaded by tutorial/12_require.bcl via:
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
300 ' bcc tutorial/12_require.bcl -L tutorial/lib
310 '
320 ' The -L flag adds tutorial/lib/ to the search path so that
330 ' require stats   resolves to  tutorial/lib/stats.bcl

340 n% = 8
350 DIM scores%(n%)
360 BCCT1% = n%

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
470 IF meanDataDim00% > 8 THEN PRINT "runtime error: `data%` of `mean!` needs "; meanDataDim00%; " elements along axis 0, but its storage only holds 8" : STOP

480 ' copy array argument into transpiled function storage: scores%() -> meanData0%()
490 FOR BCCT2% = 1 TO meanDataDim00%
500     meanData0%(BCCT2%) = scores%(BCCT2%)
510 NEXT BCCT2%

520 GOSUB 800
530 PRINT "Mean:   " + STR$(meanResult0!)
540 maximumDataDim00% = BCCT1%
550 IF maximumDataDim00% > 8 THEN PRINT "runtime error: `data%` of `maximum%` needs "; maximumDataDim00%; " elements along axis 0, but its storage only holds 8" : STOP

560 ' copy array argument into transpiled function storage: scores%() -> maximumData0%()
570 FOR BCCT3% = 1 TO maximumDataDim00%
580     maximumData0%(BCCT3%) = scores%(BCCT3%)
590 NEXT BCCT3%

600 GOSUB 900
610 PRINT "Max:    " + STR$(maximumResult0%)
620 minimumDataDim00% = BCCT1%
630 IF minimumDataDim00% > 8 THEN PRINT "runtime error: `data%` of `minimum%` needs "; minimumDataDim00%; " elements along axis 0, but its storage only holds 8" : STOP

640 ' copy array argument into transpiled function storage: scores%() -> minimumData0%()
650 FOR BCCT4% = 1 TO minimumDataDim00%
660     minimumData0%(BCCT4%) = scores%(BCCT4%)
670 NEXT BCCT4%

680 GOSUB 1010
690 PRINT "Min:    " + STR$(minimumResult0%)
700 rangeofDataDim00% = BCCT1%
710 IF rangeofDataDim00% > 8 THEN PRINT "runtime error: `data%` of `rangeOf%` needs "; rangeofDataDim00%; " elements along axis 0, but its storage only holds 8" : STOP

720 ' copy array argument into transpiled function storage: scores%() -> rangeofData0%()
730 FOR BCCT5% = 1 TO rangeofDataDim00%
740     rangeofData0%(BCCT5%) = scores%(BCCT5%)
750 NEXT BCCT5%

760 GOSUB 1120
770 PRINT "Range:  " + STR$(rangeofResult0%)

780 END

790 ' function mean!(data%)
800     ' Arithmetic mean of data%(0..sizeof(data%)-1).
810     meanSum0% = 0
820     meanCount0% = meanDataDim00%
830     FOR meanI0% = 0 TO meanCount0% - 1
840         meanSum0% = meanSum0% + meanData0%(meanI0%)
850     NEXT meanI0%
860     meanResult0! = meanSum0% / meanCount0%
870     RETURN
880 ' end function mean!

890 ' function maximum%(data%)
900     ' Largest element in data%(0..sizeof(data%)-1).
910     maximumBest0% = maximumData0%(0)
920     FOR maximumI0% = 1 TO maximumDataDim00% - 1
930         IF (maximumData0%(maximumI0%) > maximumBest0%) = 0 THEN GOTO 950
940             maximumBest0% = maximumData0%(maximumI0%)
950         REM END IF
960     NEXT maximumI0%
970     maximumResult0% = maximumBest0%
980     RETURN
990 ' end function maximum%

1000 ' function minimum%(data%)
1010     ' Smallest element in data%(0..sizeof(data%)-1).
1020     minimumBest0% = minimumData0%(0)
1030     FOR minimumI0% = 1 TO minimumDataDim00% - 1
1040         IF (minimumData0%(minimumI0%) < minimumBest0%) = 0 THEN GOTO 1060
1050             minimumBest0% = minimumData0%(minimumI0%)
1060         REM END IF
1070     NEXT minimumI0%
1080     minimumResult0% = minimumBest0%
1090     RETURN
1100 ' end function minimum%

1110 ' function rangeof%(data%)
1120     ' Difference between maximum and minimum.
1130     maximumDataDim00% = rangeofDataDim00%
1140     IF maximumDataDim00% > 8 THEN PRINT "runtime error: `data%` of `maximum%` needs "; maximumDataDim00%; " elements along axis 0, but its storage only holds 8" : STOP

1150     ' copy array argument into transpiled function storage: rangeofData0%() -> maximumData0%()
1160     FOR BCCT8% = 1 TO maximumDataDim00%
1170         maximumData0%(BCCT8%) = rangeofData0%(BCCT8%)
1180     NEXT BCCT8%

1190     GOSUB 900
1200     minimumDataDim00% = rangeofDataDim00%
1210     IF minimumDataDim00% > 8 THEN PRINT "runtime error: `data%` of `minimum%` needs "; minimumDataDim00%; " elements along axis 0, but its storage only holds 8" : STOP

1220     ' copy array argument into transpiled function storage: rangeofData0%() -> minimumData0%()
1230     FOR BCCT9% = 1 TO minimumDataDim00%
1240         minimumData0%(BCCT9%) = rangeofData0%(BCCT9%)
1250     NEXT BCCT9%

1260     GOSUB 1010
1270     rangeofResult0% = maximumResult0% - minimumResult0%
1280     RETURN
1290 ' end function rangeof%

```

<!-- END generated tutorial source -->
