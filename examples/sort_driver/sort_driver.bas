10 ' BASCAL generated BASIC -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Storage for array parameters, sized to fit every call site
40 DIM bubblesortData0%(5000)
50 DIM shakersortData0%(5000)
60 DIM shellsortData0%(5000)
70 DIM quicksortData0%(5000)

80 ' In-place bubble sort.
90 ' data% is a byref array parameter, so bcc copies the sorted result back
100 ' into the caller's array; byval (the default) would sort a private copy
110 ' and discard it.
120 ' data% -- array to sort, mutated in place
130 ' In-place cocktail shaker sort.
140 ' This is a bidirectional bubble sort over the active range. data% is
150 ' byref so the sorted result is copied back to the caller.
160 ' data% -- array to sort, mutated in place
170 ' Tiny helper used to prove recursive require resolution.
180 ' value% -- number passed through unchanged
190 ' Shell sort demonstrates a nested dependency. The helper is intentionally
200 ' trivial; the point is to exercise recursive require resolution.
210 ' data% -- array to sort, mutated in place
220 ' Iterative quicksort using an explicit stack for partition bounds.
230 ' Middle-element pivot avoids O(n^2) on already-sorted or reverse-sorted input.
240 ' data% is byref so the sorted result is copied back to the caller.
250 ' data% -- array to sort, mutated in place
260 ' Sort driver for the BASCAL example sort library.
270 ' Uses 5000 reverse-sorted elements (worst case for comparison sorts).
280 DIM source%(5000)
290 DIM bubbledata%(5000)
300 DIM shakerdata%(5000)
310 DIM shelldata%(5000)
320 DIM quickdata%(5000)
330 ' Fill with descending values: worst case for O(n^2) sorts.
340 FOR i% = 1 TO 5000
350     source%(i%) = 1001 - i%
360 NEXT i%
370 FOR i% = 1 TO 5000
380     bubbledata%(i%) = source%(i%)
390     shakerdata%(i%) = source%(i%)
400     shelldata%(i%) = source%(i%)
410     quickdata%(i%) = source%(i%)
420 NEXT i%
430 tstart# = TIMER
440 bubblesortDataDim00% = 5000
450 IF bubblesortDataDim00% > 5000 THEN PRINT "runtime error: `data%` of `bubbleSort%` needs "; bubblesortDataDim00%; " elements along axis 0, but its storage only holds 5000" : STOP

460 ' copy array argument into transpiled function storage: bubbledata%() -> bubblesortData0%()
470 FOR BCCT3% = 0 TO bubblesortDataDim00%
480     bubblesortData0%(BCCT3%) = bubbledata%(BCCT3%)
490 NEXT BCCT3%

500 GOSUB 1450

510 ' copy mutated array argument back to caller storage: bubblesortData0%() -> bubbledata%()
520 FOR BCCT4% = 0 TO bubblesortDataDim00%
530     bubbledata%(BCCT4%) = bubblesortData0%(BCCT4%)
540 NEXT BCCT4%

550 telapsed# = TIMER - tstart#
560 PRINT "Bubble sort time (ms):", telapsed# * 5000
570 ok% = 1
580 FOR i% = 1 TO 4999
590     IF (bubbledata%(i%) > bubbledata%(i% + 1)) = 0 THEN GOTO 610
600         ok% = 0
610     REM END IF
620 NEXT i%
630 IF (ok% = 1) = 0 THEN GOTO 660
640     PRINT "Bubble: OK"
650 GOTO 670
660     PRINT "Bubble: FAILED"
670 REM END IF
680 tstart# = TIMER
690 shakersortDataDim00% = 5000
700 IF shakersortDataDim00% > 5000 THEN PRINT "runtime error: `data%` of `shakerSort%` needs "; shakersortDataDim00%; " elements along axis 0, but its storage only holds 5000" : STOP

710 ' copy array argument into transpiled function storage: shakerdata%() -> shakersortData0%()
720 FOR BCCT8% = 0 TO shakersortDataDim00%
730     shakersortData0%(BCCT8%) = shakerdata%(BCCT8%)
740 NEXT BCCT8%

750 GOSUB 1610

760 ' copy mutated array argument back to caller storage: shakersortData0%() -> shakerdata%()
770 FOR BCCT9% = 0 TO shakersortDataDim00%
780     shakerdata%(BCCT9%) = shakersortData0%(BCCT9%)
790 NEXT BCCT9%

800 telapsed# = TIMER - tstart#
810 PRINT "Shaker sort time (ms):", telapsed# * 5000
820 ok% = 1
830 FOR i% = 1 TO 4999
840     IF (shakerdata%(i%) > shakerdata%(i% + 1)) = 0 THEN GOTO 860
850         ok% = 0
860     REM END IF
870 NEXT i%
880 IF (ok% = 1) = 0 THEN GOTO 910
890     PRINT "Shaker: OK"
900 GOTO 920
910     PRINT "Shaker: FAILED"
920 REM END IF
930 tstart# = TIMER
940 shellsortDataDim00% = 5000
950 IF shellsortDataDim00% > 5000 THEN PRINT "runtime error: `data%` of `shellSort%` needs "; shellsortDataDim00%; " elements along axis 0, but its storage only holds 5000" : STOP

960 ' copy array argument into transpiled function storage: shelldata%() -> shellsortData0%()
970 FOR BCCT13% = 0 TO shellsortDataDim00%
980     shellsortData0%(BCCT13%) = shelldata%(BCCT13%)
990 NEXT BCCT13%

1000 GOSUB 1990

1010 ' copy mutated array argument back to caller storage: shellsortData0%() -> shelldata%()
1020 FOR BCCT14% = 0 TO shellsortDataDim00%
1030     shelldata%(BCCT14%) = shellsortData0%(BCCT14%)
1040 NEXT BCCT14%

1050 telapsed# = TIMER - tstart#
1060 PRINT "Shell sort time (ms):", telapsed# * 5000
1070 ok% = 1
1080 FOR i% = 1 TO 4999
1090     IF (shelldata%(i%) > shelldata%(i% + 1)) = 0 THEN GOTO 1110
1100         ok% = 0
1110     REM END IF
1120 NEXT i%
1130 IF (ok% = 1) = 0 THEN GOTO 1160
1140     PRINT "Shell: OK"
1150 GOTO 1170
1160     PRINT "Shell: FAILED"
1170 REM END IF
1180 tstart# = TIMER
1190 quicksortDataDim00% = 5000
1200 IF quicksortDataDim00% > 5000 THEN PRINT "runtime error: `data%` of `quickSort%` needs "; quicksortDataDim00%; " elements along axis 0, but its storage only holds 5000" : STOP

1210 ' copy array argument into transpiled function storage: quickdata%() -> quicksortData0%()
1220 FOR BCCT18% = 0 TO quicksortDataDim00%
1230     quicksortData0%(BCCT18%) = quickdata%(BCCT18%)
1240 NEXT BCCT18%

1250 GOSUB 2290

1260 ' copy mutated array argument back to caller storage: quicksortData0%() -> quickdata%()
1270 FOR BCCT19% = 0 TO quicksortDataDim00%
1280     quickdata%(BCCT19%) = quicksortData0%(BCCT19%)
1290 NEXT BCCT19%

1300 telapsed# = TIMER - tstart#
1310 PRINT "Quick sort time (ms):", telapsed# * 5000
1320 ok% = 1
1330 FOR i% = 1 TO 4999
1340     IF (quickdata%(i%) > quickdata%(i% + 1)) = 0 THEN GOTO 1360
1350         ok% = 0
1360     REM END IF
1370 NEXT i%
1380 IF (ok% = 1) = 0 THEN GOTO 1410
1390     PRINT "Quick: OK"
1400 GOTO 1420
1410     PRINT "Quick: FAILED"
1420 REM END IF
1430 END

1440 ' function bubblesort%(data%)
1450     bubblesortCount0% = (bubblesortDataDim00% + 1)
1460     ' After each outer pass, the largest remaining value has bubbled right.
1470     FOR bubblesortI0% = 1 TO bubblesortCount0% - 1
1480         FOR bubblesortJ0% = 1 TO bubblesortCount0% - bubblesortI0%
1490             IF (bubblesortData0%(bubblesortJ0%) > bubblesortData0%(bubblesortJ0% + 1)) = 0 THEN GOTO 1540
1500                 ' Swap adjacent out-of-order elements.
1510                 bubblesortTemp0% = bubblesortData0%(bubblesortJ0%)
1520                 bubblesortData0%(bubblesortJ0%) = bubblesortData0%(bubblesortJ0% + 1)
1530                 bubblesortData0%(bubblesortJ0% + 1) = bubblesortTemp0%
1540             REM END IF
1550         NEXT bubblesortJ0%
1560     NEXT bubblesortI0%
1570     bubblesortResult0% = 0
1580     RETURN
1590 ' end function bubblesort%

1600 ' function shakersort%(data%)
1610     shakersortCount0% = (shakersortDataDim00% + 1)
1620     LEFT% = 1
1630     RIGHT% = shakersortCount0% - 1
1640     shakersortSwapped0% = 1
1650     ' Continue until a full bidirectional pass performs no swaps.
1660     IF (shakersortSwapped0%) = 0 THEN GOTO 1890
1670         shakersortSwapped0% = 0
1680         ' Forward pass moves large values toward the right edge.
1690         FOR shakersortI0% = LEFT% TO RIGHT%
1700             IF (shakersortData0%(shakersortI0%) > shakersortData0%(shakersortI0% + 1)) = 0 THEN GOTO 1750
1710                 shakersortTemp0% = shakersortData0%(shakersortI0%)
1720                 shakersortData0%(shakersortI0%) = shakersortData0%(shakersortI0% + 1)
1730                 shakersortData0%(shakersortI0% + 1) = shakersortTemp0%
1740                 shakersortSwapped0% = 1
1750             REM END IF
1760         NEXT shakersortI0%
1770         RIGHT% = RIGHT% - 1
1780         ' Backward pass moves small values toward the left edge.
1790         FOR shakersortI0% = RIGHT% TO LEFT% STEP -1
1800             IF (shakersortData0%(shakersortI0%) > shakersortData0%(shakersortI0% + 1)) = 0 THEN GOTO 1850
1810                 shakersortTemp0% = shakersortData0%(shakersortI0%)
1820                 shakersortData0%(shakersortI0%) = shakersortData0%(shakersortI0% + 1)
1830                 shakersortData0%(shakersortI0% + 1) = shakersortTemp0%
1840                 shakersortSwapped0% = 1
1850             REM END IF
1860         NEXT shakersortI0%
1870         LEFT% = LEFT% + 1
1880     GOTO 1660
1890     REM END WHILE
1900     shakersortResult0% = 0
1910     RETURN
1920 ' end function shakersort%

1930 ' function touch%(value%)
1940     ' Return the value unchanged.
1950     touchResult0% = touchValue0%
1960     RETURN
1970 ' end function touch%

1980 ' function shellsort%(data%)
1990     ' Normalize the count through a required helper so this file has its own
2000     ' dependency chain.
2010     BCCT31% = (shellsortDataDim00% + 1)
2020     BCCT32% = BCCT31%
2030     touchValue0% = BCCT32%
2040     GOSUB 1940
2050     shellsortCount0% = touchResult0%
2060     shellsortGap0% = shellsortCount0% \ 2
2070     ' Repeatedly insertion-sort elements that are gap positions apart.
2080     IF (shellsortGap0% > 0) = 0 THEN GOTO 2240
2090         shellsortI0% = shellsortGap0% + 1
2100         IF (shellsortI0% <= shellsortCount0%) = 0 THEN GOTO 2210
2110             shellsortTemp0% = shellsortData0%(shellsortI0%)
2120             shellsortJ0% = shellsortI0%
2130             IF ((shellsortJ0% > shellsortGap0%) AND (shellsortData0%(shellsortJ0% - shellsortGap0%) > shellsortTemp0%)) = 0 THEN GOTO 2170
2140                 shellsortData0%(shellsortJ0%) = shellsortData0%(shellsortJ0% - shellsortGap0%)
2150                 shellsortJ0% = shellsortJ0% - shellsortGap0%
2160             GOTO 2130
2170             REM END WHILE
2180             shellsortData0%(shellsortJ0%) = shellsortTemp0%
2190             shellsortI0% = shellsortI0% + 1
2200         GOTO 2100
2210         REM END WHILE
2220         shellsortGap0% = shellsortGap0% \ 2
2230     GOTO 2080
2240     REM END WHILE
2250     shellsortResult0% = 0
2260     RETURN
2270 ' end function shellsort%

2280 ' function quicksort%(data%)
2290     DIM quicksortSLow0%(64)
2300     DIM quicksortSHigh0%(64)
2310     quicksortSTop0% = 1
2320     quicksortSLow0%(1) = 1
2330     quicksortSHigh0%(1) = (quicksortDataDim00% + 1)
2340     IF (quicksortSTop0% > 0) = 0 THEN GOTO 2720
2350         quicksortQHigh0% = quicksortSHigh0%(quicksortSTop0%)
2360         quicksortQLow0% = quicksortSLow0%(quicksortSTop0%)
2370         quicksortSTop0% = quicksortSTop0% - 1
2380         IF (quicksortQHigh0% > quicksortQLow0%) = 0 THEN GOTO 2700
2390             ' Swap middle element to high as pivot.
2400             MID% = quicksortQLow0% + ((quicksortQHigh0% - quicksortQLow0%) / 2)
2410             quicksortTemp0% = quicksortData0%(MID%)
2420             quicksortData0%(MID%) = quicksortData0%(quicksortQHigh0%)
2430             quicksortData0%(quicksortQHigh0%) = quicksortTemp0%
2440             ' Partition: move elements <= pivot left of wall.
2450             quicksortPivot0% = quicksortData0%(quicksortQHigh0%)
2460             quicksortWall0% = quicksortQLow0% - 1
2470             FOR quicksortJ0% = quicksortQLow0% TO quicksortQHigh0% - 1
2480                 IF (quicksortData0%(quicksortJ0%) <= quicksortPivot0%) = 0 THEN GOTO 2530
2490                     quicksortWall0% = quicksortWall0% + 1
2500                     quicksortTemp0% = quicksortData0%(quicksortWall0%)
2510                     quicksortData0%(quicksortWall0%) = quicksortData0%(quicksortJ0%)
2520                     quicksortData0%(quicksortJ0%) = quicksortTemp0%
2530                 REM END IF
2540             NEXT quicksortJ0%
2550             ' Place pivot at wall.
2560             quicksortWall0% = quicksortWall0% + 1
2570             quicksortTemp0% = quicksortData0%(quicksortWall0%)
2580             quicksortData0%(quicksortWall0%) = quicksortData0%(quicksortQHigh0%)
2590             quicksortData0%(quicksortQHigh0%) = quicksortTemp0%
2600             IF ((quicksortWall0% - 1) > quicksortQLow0%) = 0 THEN GOTO 2640
2610                 quicksortSTop0% = quicksortSTop0% + 1
2620                 quicksortSLow0%(quicksortSTop0%) = quicksortQLow0%
2630                 quicksortSHigh0%(quicksortSTop0%) = quicksortWall0% - 1
2640             REM END IF
2650             IF ((quicksortWall0% + 1) < quicksortQHigh0%) = 0 THEN GOTO 2690
2660                 quicksortSTop0% = quicksortSTop0% + 1
2670                 quicksortSLow0%(quicksortSTop0%) = quicksortWall0% + 1
2680                 quicksortSHigh0%(quicksortSTop0%) = quicksortQHigh0%
2690             REM END IF
2700         REM END IF
2710     GOTO 2340
2720     REM END WHILE
2730     quicksortResult0% = 0
2740     RETURN
2750 ' end function quicksort%
