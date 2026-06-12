10 ' BASCAL generated BASIC
20 ' Functions are lowered to global variables, labels, and GOSUB

30 ' In-place bubble sort.
40 ' data% is an array parameter; bcc lowers calls with copy-in/copy-out.
50 ' In-place cocktail shaker sort.
60 ' This is a bidirectional bubble sort over the active range.
70 ' Tiny helper used to prove recursive require resolution.
80 ' Shell sort demonstrates a nested dependency. The helper is intentionally
90 ' trivial; the point is to exercise recursive require resolution.

100 ' Iterative quicksort using an explicit stack for partition bounds.
110 ' Middle-element pivot avoids O(n^2) on already-sorted or reverse-sorted input.
120 ' Sort driver for the BASCAL example sort library.
130 ' Uses 5000 reverse-sorted elements (worst case for comparison sorts).

140 DIM source%(5000)
150 DIM bubbledata%(5000)
160 DIM shakerdata%(5000)
170 DIM shelldata%(5000)
180 DIM quickdata%(5000)

190 ' Fill with descending values: worst case for O(n^2) sorts.
200 FOR i% = 1 TO 5000
210     source%(i%) = 1001 - i%
220 NEXT i%

230 FOR i% = 1 TO 5000
240     bubbledata%(i%) = source%(i%)
250     shakerdata%(i%) = source%(i%)
260     shelldata%(i%) = source%(i%)
270     quickdata%(i%) = source%(i%)
280 NEXT i%

290 tstart# = TIMER
300 bubblesort_count_0% = 5000
310 DIM bubblesort_data_0%(5000)

320 ' copy array argument into lowered function storage: bubbledata%() -> bubblesort_data_0%()
330 FOR BCC_T1% = 1 TO 5000
340     bubblesort_data_0%(BCC_T1%) = bubbledata%(BCC_T1%)
350 NEXT BCC_T1%

360 GOSUB 1310

370 ' copy mutated array argument back to caller storage: bubblesort_data_0%() -> bubbledata%()
380 FOR BCC_T2% = 1 TO 5000
390     bubbledata%(BCC_T2%) = bubblesort_data_0%(BCC_T2%)
400 NEXT BCC_T2%

410 telapsed# = TIMER - tstart#
420 PRINT "Bubble sort time (ms):", telapsed# * 5000
430 ok% = 1
440 FOR i% = 1 TO 4999
450     IF (bubbledata%(i%) > bubbledata%(i% + 1)) = 0 THEN GOTO 470
460         ok% = 0
470     REM END IF
480 NEXT i%
490 IF (ok% = 1) = 0 THEN GOTO 520
500     PRINT "Bubble: OK"
510     GOTO 530
520     PRINT "Bubble: FAILED"
530 REM END IF

540 tstart# = TIMER
550 shakersort_count_0% = 5000
560 DIM shakersort_data_0%(5000)

570 ' copy array argument into lowered function storage: shakerdata%() -> shakersort_data_0%()
580 FOR BCC_T5% = 1 TO 5000
590     shakersort_data_0%(BCC_T5%) = shakerdata%(BCC_T5%)
600 NEXT BCC_T5%

610 GOSUB 1460

620 ' copy mutated array argument back to caller storage: shakersort_data_0%() -> shakerdata%()
630 FOR BCC_T6% = 1 TO 5000
640     shakerdata%(BCC_T6%) = shakersort_data_0%(BCC_T6%)
650 NEXT BCC_T6%

660 telapsed# = TIMER - tstart#
670 PRINT "Shaker sort time (ms):", telapsed# * 5000
680 ok% = 1
690 FOR i% = 1 TO 4999
700     IF (shakerdata%(i%) > shakerdata%(i% + 1)) = 0 THEN GOTO 720
710         ok% = 0
720     REM END IF
730 NEXT i%
740 IF (ok% = 1) = 0 THEN GOTO 770
750     PRINT "Shaker: OK"
760     GOTO 780
770     PRINT "Shaker: FAILED"
780 REM END IF

790 tstart# = TIMER
800 shellsort_count_0% = 5000
810 DIM shellsort_data_0%(5000)

820 ' copy array argument into lowered function storage: shelldata%() -> shellsort_data_0%()
830 FOR BCC_T9% = 1 TO 5000
840     shellsort_data_0%(BCC_T9%) = shelldata%(BCC_T9%)
850 NEXT BCC_T9%

860 GOSUB 1830

870 ' copy mutated array argument back to caller storage: shellsort_data_0%() -> shelldata%()
880 FOR BCC_T10% = 1 TO 5000
890     shelldata%(BCC_T10%) = shellsort_data_0%(BCC_T10%)
900 NEXT BCC_T10%

910 telapsed# = TIMER - tstart#
920 PRINT "Shell sort time (ms):", telapsed# * 5000
930 ok% = 1
940 FOR i% = 1 TO 4999
950     IF (shelldata%(i%) > shelldata%(i% + 1)) = 0 THEN GOTO 970
960         ok% = 0
970     REM END IF
980 NEXT i%
990 IF (ok% = 1) = 0 THEN GOTO 1020
1000     PRINT "Shell: OK"
1010     GOTO 1030
1020     PRINT "Shell: FAILED"
1030 REM END IF

1040 tstart# = TIMER
1050 quicksort_count_0% = 5000
1060 DIM quicksort_data_0%(5000)

1070 ' copy array argument into lowered function storage: quickdata%() -> quicksort_data_0%()
1080 FOR BCC_T13% = 1 TO 5000
1090     quicksort_data_0%(BCC_T13%) = quickdata%(BCC_T13%)
1100 NEXT BCC_T13%

1110 GOSUB 2110

1120 ' copy mutated array argument back to caller storage: quicksort_data_0%() -> quickdata%()
1130 FOR BCC_T14% = 1 TO 5000
1140     quickdata%(BCC_T14%) = quicksort_data_0%(BCC_T14%)
1150 NEXT BCC_T14%

1160 telapsed# = TIMER - tstart#
1170 PRINT "Quick sort time (ms):", telapsed# * 5000
1180 ok% = 1
1190 FOR i% = 1 TO 4999
1200     IF (quickdata%(i%) > quickdata%(i% + 1)) = 0 THEN GOTO 1220
1210         ok% = 0
1220     REM END IF
1230 NEXT i%
1240 IF (ok% = 1) = 0 THEN GOTO 1270
1250     PRINT "Quick: OK"
1260     GOTO 1280
1270     PRINT "Quick: FAILED"
1280 REM END IF

1290 END

1300 ' function bubblesort%(data%, count%)
1310     ' After each outer pass, the largest remaining value has bubbled right.
1320     FOR bubblesort_i_0% = 1 TO bubblesort_count_0% - 1
1330         FOR bubblesort_j_0% = 1 TO bubblesort_count_0% - bubblesort_i_0%
1340             IF (bubblesort_data_0%(bubblesort_j_0%) > bubblesort_data_0%(bubblesort_j_0% + 1)) = 0 THEN GOTO 1390
1350                 ' Swap adjacent out-of-order elements.
1360                 bubblesort_temp_0% = bubblesort_data_0%(bubblesort_j_0%)
1370                 bubblesort_data_0%(bubblesort_j_0%) = bubblesort_data_0%(bubblesort_j_0% + 1)
1380                 bubblesort_data_0%(bubblesort_j_0% + 1) = bubblesort_temp_0%
1390             REM END IF
1400         NEXT bubblesort_j_0%
1410     NEXT bubblesort_i_0%
1420     bubblesort_result_0% = 0
1430     RETURN
1440 ' end function bubblesort%

1450 ' function shakersort%(data%, count%)
1460     LEFT% = 1
1470     RIGHT% = shakersort_count_0% - 1
1480     shakersort_swapped_0% = 1

1490     ' Continue until a full bidirectional pass performs no swaps.
1500     IF (shakersort_swapped_0%) = 0 THEN GOTO 1730
1510         shakersort_swapped_0% = 0

1520         ' Forward pass moves large values toward the right edge.
1530         FOR shakersort_i_0% = LEFT% TO RIGHT%
1540             IF (shakersort_data_0%(shakersort_i_0%) > shakersort_data_0%(shakersort_i_0% + 1)) = 0 THEN GOTO 1590
1550                 shakersort_temp_0% = shakersort_data_0%(shakersort_i_0%)
1560                 shakersort_data_0%(shakersort_i_0%) = shakersort_data_0%(shakersort_i_0% + 1)
1570                 shakersort_data_0%(shakersort_i_0% + 1) = shakersort_temp_0%
1580                 shakersort_swapped_0% = 1
1590             REM END IF
1600         NEXT shakersort_i_0%

1610         RIGHT% = RIGHT% - 1

1620         ' Backward pass moves small values toward the left edge.
1630         FOR shakersort_i_0% = RIGHT% TO LEFT% STEP -1
1640             IF (shakersort_data_0%(shakersort_i_0%) > shakersort_data_0%(shakersort_i_0% + 1)) = 0 THEN GOTO 1690
1650                 shakersort_temp_0% = shakersort_data_0%(shakersort_i_0%)
1660                 shakersort_data_0%(shakersort_i_0%) = shakersort_data_0%(shakersort_i_0% + 1)
1670                 shakersort_data_0%(shakersort_i_0% + 1) = shakersort_temp_0%
1680                 shakersort_swapped_0% = 1
1690             REM END IF
1700         NEXT shakersort_i_0%

1710         LEFT% = LEFT% + 1
1720         GOTO 1500
1730     REM END WHILE

1740     shakersort_result_0% = 0
1750     RETURN
1760 ' end function shakersort%

1770 ' function touch%(value%)
1780     ' Return the value unchanged.
1790     touch_result_0% = touch_value_0%
1800     RETURN
1810 ' end function touch%

1820 ' function shellsort%(data%, count%)
1830     ' Normalize the count through a required helper so this file has its own
1840     ' dependency chain.
1850     touch_value_0% = shellsort_count_0%
1860     GOSUB 1780
1870     shellsort_count_0% = touch_result_0%
1880     shellsort_gap_0% = shellsort_count_0% / 2

1890     ' Repeatedly insertion-sort elements that are gap positions apart.
1900     IF (shellsort_gap_0% > 0) = 0 THEN GOTO 2060
1910         shellsort_i_0% = shellsort_gap_0% + 1

1920         IF (shellsort_i_0% <= shellsort_count_0%) = 0 THEN GOTO 2030
1930             shellsort_temp_0% = shellsort_data_0%(shellsort_i_0%)
1940             shellsort_j_0% = shellsort_i_0%

1950             IF ((shellsort_j_0% > shellsort_gap_0%) AND (shellsort_data_0%(shellsort_j_0% - shellsort_gap_0%) > shellsort_temp_0%)) = 0 THEN GOTO 1990
1960                 shellsort_data_0%(shellsort_j_0%) = shellsort_data_0%(shellsort_j_0% - shellsort_gap_0%)
1970                 shellsort_j_0% = shellsort_j_0% - shellsort_gap_0%
1980                 GOTO 1950
1990             REM END WHILE

2000             shellsort_data_0%(shellsort_j_0%) = shellsort_temp_0%
2010             shellsort_i_0% = shellsort_i_0% + 1
2020             GOTO 1920
2030         REM END WHILE

2040         shellsort_gap_0% = shellsort_gap_0% / 2
2050         GOTO 1900
2060     REM END WHILE

2070     shellsort_result_0% = 0
2080     RETURN
2090 ' end function shellsort%

2100 ' function quicksort%(data%, count%)
2110     DIM quicksort_slow_0%(64)
2120     DIM quicksort_shigh_0%(64)

2130     quicksort_stop_0% = 1
2140     quicksort_slow_0%(1) = 1
2150     quicksort_shigh_0%(1) = quicksort_count_0%

2160     IF (quicksort_stop_0% > 0) = 0 THEN GOTO 2540
2170         quicksort_qhigh_0% = quicksort_shigh_0%(quicksort_stop_0%)
2180         quicksort_qlow_0% = quicksort_slow_0%(quicksort_stop_0%)
2190         quicksort_stop_0% = quicksort_stop_0% - 1

2200         IF (quicksort_qhigh_0% > quicksort_qlow_0%) = 0 THEN GOTO 2520
2210             ' Swap middle element to high as pivot.
2220             MID% = quicksort_qlow_0% + ((quicksort_qhigh_0% - quicksort_qlow_0%) / 2)
2230             quicksort_temp_0% = quicksort_data_0%(MID%)
2240             quicksort_data_0%(MID%) = quicksort_data_0%(quicksort_qhigh_0%)
2250             quicksort_data_0%(quicksort_qhigh_0%) = quicksort_temp_0%

2260             ' Partition: move elements <= pivot left of wall.
2270             quicksort_pivot_0% = quicksort_data_0%(quicksort_qhigh_0%)
2280             quicksort_wall_0% = quicksort_qlow_0% - 1

2290             FOR quicksort_j_0% = quicksort_qlow_0% TO quicksort_qhigh_0% - 1
2300                 IF (quicksort_data_0%(quicksort_j_0%) <= quicksort_pivot_0%) = 0 THEN GOTO 2350
2310                     quicksort_wall_0% = quicksort_wall_0% + 1
2320                     quicksort_temp_0% = quicksort_data_0%(quicksort_wall_0%)
2330                     quicksort_data_0%(quicksort_wall_0%) = quicksort_data_0%(quicksort_j_0%)
2340                     quicksort_data_0%(quicksort_j_0%) = quicksort_temp_0%
2350                 REM END IF
2360             NEXT quicksort_j_0%

2370             ' Place pivot at wall.
2380             quicksort_wall_0% = quicksort_wall_0% + 1
2390             quicksort_temp_0% = quicksort_data_0%(quicksort_wall_0%)
2400             quicksort_data_0%(quicksort_wall_0%) = quicksort_data_0%(quicksort_qhigh_0%)
2410             quicksort_data_0%(quicksort_qhigh_0%) = quicksort_temp_0%

2420             IF ((quicksort_wall_0% - 1) > quicksort_qlow_0%) = 0 THEN GOTO 2460
2430                 quicksort_stop_0% = quicksort_stop_0% + 1
2440                 quicksort_slow_0%(quicksort_stop_0%) = quicksort_qlow_0%
2450                 quicksort_shigh_0%(quicksort_stop_0%) = quicksort_wall_0% - 1
2460             REM END IF

2470             IF ((quicksort_wall_0% + 1) < quicksort_qhigh_0%) = 0 THEN GOTO 2510
2480                 quicksort_stop_0% = quicksort_stop_0% + 1
2490                 quicksort_slow_0%(quicksort_stop_0%) = quicksort_wall_0% + 1
2500                 quicksort_shigh_0%(quicksort_stop_0%) = quicksort_qhigh_0%
2510             REM END IF
2520         REM END IF
2530         GOTO 2160
2540     REM END WHILE

2550     quicksort_result_0% = 0
2560     RETURN
2570 ' end function quicksort%
