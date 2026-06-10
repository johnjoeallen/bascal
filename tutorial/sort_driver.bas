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
150 DIM bubbleData%(5000)
160 DIM shakerData%(5000)
170 DIM shellData%(5000)
180 DIM quickData%(5000)

190 ' Fill with descending values: worst case for O(n^2) sorts.
200 FOR i% = 1 TO 5000
210     source%(i%) = 1001 - i%
220 NEXT i%

230 FOR i% = 1 TO 5000
240     bubbleData%(i%) = source%(i%)
250     shakerData%(i%) = source%(i%)
260     shellData%(i%) = source%(i%)
270     quickData%(i%) = source%(i%)
280 NEXT i%

290 tStart# = TIMER
300 bubblesort_count% = 5000
310 DIM bubblesort_data%(5000)

320 ' copy array argument into lowered function storage: bubbleData%() -> bubblesort_data%()
330 FOR BCC_T1% = 1 TO 5000
340     bubblesort_data%(BCC_T1%) = bubbleData%(BCC_T1%)
350 NEXT BCC_T1%

360 GOSUB 1310

370 ' copy mutated array argument back to caller storage: bubblesort_data%() -> bubbleData%()
380 FOR BCC_T2% = 1 TO 5000
390     bubbleData%(BCC_T2%) = bubblesort_data%(BCC_T2%)
400 NEXT BCC_T2%

410 tElapsed# = TIMER - tStart#
420 PRINT "Bubble sort time (ms):", tElapsed# * 5000
430 ok% = 1
440 FOR i% = 1 TO 4999
450     IF (bubbleData%(i%) > bubbleData%(i% + 1)) = 0 THEN GOTO 470
460         ok% = 0
470     REM END IF
480 NEXT i%
490 IF (ok% = 1) = 0 THEN GOTO 520
500     PRINT "Bubble: OK"
510     GOTO 530
520     PRINT "Bubble: FAILED"
530 REM END IF

540 tStart# = TIMER
550 shakersort_count% = 5000
560 DIM shakersort_data%(5000)

570 ' copy array argument into lowered function storage: shakerData%() -> shakersort_data%()
580 FOR BCC_T5% = 1 TO 5000
590     shakersort_data%(BCC_T5%) = shakerData%(BCC_T5%)
600 NEXT BCC_T5%

610 GOSUB 1460

620 ' copy mutated array argument back to caller storage: shakersort_data%() -> shakerData%()
630 FOR BCC_T6% = 1 TO 5000
640     shakerData%(BCC_T6%) = shakersort_data%(BCC_T6%)
650 NEXT BCC_T6%

660 tElapsed# = TIMER - tStart#
670 PRINT "Shaker sort time (ms):", tElapsed# * 5000
680 ok% = 1
690 FOR i% = 1 TO 4999
700     IF (shakerData%(i%) > shakerData%(i% + 1)) = 0 THEN GOTO 720
710         ok% = 0
720     REM END IF
730 NEXT i%
740 IF (ok% = 1) = 0 THEN GOTO 770
750     PRINT "Shaker: OK"
760     GOTO 780
770     PRINT "Shaker: FAILED"
780 REM END IF

790 tStart# = TIMER
800 shellsort_count% = 5000
810 DIM shellsort_data%(5000)

820 ' copy array argument into lowered function storage: shellData%() -> shellsort_data%()
830 FOR BCC_T9% = 1 TO 5000
840     shellsort_data%(BCC_T9%) = shellData%(BCC_T9%)
850 NEXT BCC_T9%

860 GOSUB 1830

870 ' copy mutated array argument back to caller storage: shellsort_data%() -> shellData%()
880 FOR BCC_T10% = 1 TO 5000
890     shellData%(BCC_T10%) = shellsort_data%(BCC_T10%)
900 NEXT BCC_T10%

910 tElapsed# = TIMER - tStart#
920 PRINT "Shell sort time (ms):", tElapsed# * 5000
930 ok% = 1
940 FOR i% = 1 TO 4999
950     IF (shellData%(i%) > shellData%(i% + 1)) = 0 THEN GOTO 970
960         ok% = 0
970     REM END IF
980 NEXT i%
990 IF (ok% = 1) = 0 THEN GOTO 1020
1000     PRINT "Shell: OK"
1010     GOTO 1030
1020     PRINT "Shell: FAILED"
1030 REM END IF

1040 tStart# = TIMER
1050 quicksort_count% = 5000
1060 DIM quicksort_data%(5000)

1070 ' copy array argument into lowered function storage: quickData%() -> quicksort_data%()
1080 FOR BCC_T13% = 1 TO 5000
1090     quicksort_data%(BCC_T13%) = quickData%(BCC_T13%)
1100 NEXT BCC_T13%

1110 GOSUB 2110

1120 ' copy mutated array argument back to caller storage: quicksort_data%() -> quickData%()
1130 FOR BCC_T14% = 1 TO 5000
1140     quickData%(BCC_T14%) = quicksort_data%(BCC_T14%)
1150 NEXT BCC_T14%

1160 tElapsed# = TIMER - tStart#
1170 PRINT "Quick sort time (ms):", tElapsed# * 5000
1180 ok% = 1
1190 FOR i% = 1 TO 4999
1200     IF (quickData%(i%) > quickData%(i% + 1)) = 0 THEN GOTO 1220
1210         ok% = 0
1220     REM END IF
1230 NEXT i%
1240 IF (ok% = 1) = 0 THEN GOTO 1270
1250     PRINT "Quick: OK"
1260     GOTO 1280
1270     PRINT "Quick: FAILED"
1280 REM END IF

1290 END

1300 ' function bubbleSort%(data%, count%)
1310     ' After each outer pass, the largest remaining value has bubbled right.
1320     FOR bubblesort_i% = 1 TO bubblesort_count% - 1
1330         FOR bubblesort_j% = 1 TO bubblesort_count% - bubblesort_i%
1340             IF (bubblesort_data%(bubblesort_j%) > bubblesort_data%(bubblesort_j% + 1)) = 0 THEN GOTO 1390
1350                 ' Swap adjacent out-of-order elements.
1360                 bubblesort_temp% = bubblesort_data%(bubblesort_j%)
1370                 bubblesort_data%(bubblesort_j%) = bubblesort_data%(bubblesort_j% + 1)
1380                 bubblesort_data%(bubblesort_j% + 1) = bubblesort_temp%
1390             REM END IF
1400         NEXT bubblesort_j%
1410     NEXT bubblesort_i%
1420     bubblesort_result% = 0
1430     RETURN
1440 ' end function bubbleSort%

1450 ' function shakerSort%(data%, count%)
1460     shakersort_left% = 1
1470     shakersort_right% = shakersort_count% - 1
1480     shakersort_swapped% = 1

1490     ' Continue until a full bidirectional pass performs no swaps.
1500     IF (shakersort_swapped%) = 0 THEN GOTO 1730
1510         shakersort_swapped% = 0

1520         ' Forward pass moves large values toward the right edge.
1530         FOR shakersort_i% = shakersort_left% TO shakersort_right%
1540             IF (shakersort_data%(shakersort_i%) > shakersort_data%(shakersort_i% + 1)) = 0 THEN GOTO 1590
1550                 shakersort_temp% = shakersort_data%(shakersort_i%)
1560                 shakersort_data%(shakersort_i%) = shakersort_data%(shakersort_i% + 1)
1570                 shakersort_data%(shakersort_i% + 1) = shakersort_temp%
1580                 shakersort_swapped% = 1
1590             REM END IF
1600         NEXT shakersort_i%

1610         shakersort_right% = shakersort_right% - 1

1620         ' Backward pass moves small values toward the left edge.
1630         FOR shakersort_i% = shakersort_right% TO shakersort_left% STEP -1
1640             IF (shakersort_data%(shakersort_i%) > shakersort_data%(shakersort_i% + 1)) = 0 THEN GOTO 1690
1650                 shakersort_temp% = shakersort_data%(shakersort_i%)
1660                 shakersort_data%(shakersort_i%) = shakersort_data%(shakersort_i% + 1)
1670                 shakersort_data%(shakersort_i% + 1) = shakersort_temp%
1680                 shakersort_swapped% = 1
1690             REM END IF
1700         NEXT shakersort_i%

1710         shakersort_left% = shakersort_left% + 1
1720         GOTO 1500
1730     REM END WHILE

1740     shakersort_result% = 0
1750     RETURN
1760 ' end function shakerSort%

1770 ' function touch%(value%)
1780     ' Return the value unchanged.
1790     touch_result% = touch_value%
1800     RETURN
1810 ' end function touch%

1820 ' function shellSort%(data%, count%)
1830     ' Normalize the count through a required helper so this file has its own
1840     ' dependency chain.
1850     touch_value% = shellsort_count%
1860     GOSUB 1780
1870     shellsort_count% = touch_result%
1880     shellsort_gap% = shellsort_count% / 2

1890     ' Repeatedly insertion-sort elements that are gap positions apart.
1900     IF (shellsort_gap% > 0) = 0 THEN GOTO 2060
1910         shellsort_i% = shellsort_gap% + 1

1920         IF (shellsort_i% <= shellsort_count%) = 0 THEN GOTO 2030
1930             shellsort_temp% = shellsort_data%(shellsort_i%)
1940             shellsort_j% = shellsort_i%

1950             IF ((shellsort_j% > shellsort_gap%) AND (shellsort_data%(shellsort_j% - shellsort_gap%) > shellsort_temp%)) = 0 THEN GOTO 1990
1960                 shellsort_data%(shellsort_j%) = shellsort_data%(shellsort_j% - shellsort_gap%)
1970                 shellsort_j% = shellsort_j% - shellsort_gap%
1980                 GOTO 1950
1990             REM END WHILE

2000             shellsort_data%(shellsort_j%) = shellsort_temp%
2010             shellsort_i% = shellsort_i% + 1
2020             GOTO 1920
2030         REM END WHILE

2040         shellsort_gap% = shellsort_gap% / 2
2050         GOTO 1900
2060     REM END WHILE

2070     shellsort_result% = 0
2080     RETURN
2090 ' end function shellSort%

2100 ' function quickSort%(data%, count%)
2110     DIM quicksort_slow%(64)
2120     DIM quicksort_shigh%(64)

2130     quicksort_stop% = 1
2140     quicksort_slow%(1) = 1
2150     quicksort_shigh%(1) = quicksort_count%

2160     IF (quicksort_stop% > 0) = 0 THEN GOTO 2540
2170         quicksort_qhigh% = quicksort_shigh%(quicksort_stop%)
2180         quicksort_qlow% = quicksort_slow%(quicksort_stop%)
2190         quicksort_stop% = quicksort_stop% - 1

2200         IF (quicksort_qhigh% > quicksort_qlow%) = 0 THEN GOTO 2520
2210             ' Swap middle element to high as pivot.
2220             quicksort_mid% = quicksort_qlow% + ((quicksort_qhigh% - quicksort_qlow%) / 2)
2230             quicksort_temp% = quicksort_data%(quicksort_mid%)
2240             quicksort_data%(quicksort_mid%) = quicksort_data%(quicksort_qhigh%)
2250             quicksort_data%(quicksort_qhigh%) = quicksort_temp%

2260             ' Partition: move elements <= pivot left of wall.
2270             quicksort_pivot% = quicksort_data%(quicksort_qhigh%)
2280             quicksort_wall% = quicksort_qlow% - 1

2290             FOR quicksort_j% = quicksort_qlow% TO quicksort_qhigh% - 1
2300                 IF (quicksort_data%(quicksort_j%) <= quicksort_pivot%) = 0 THEN GOTO 2350
2310                     quicksort_wall% = quicksort_wall% + 1
2320                     quicksort_temp% = quicksort_data%(quicksort_wall%)
2330                     quicksort_data%(quicksort_wall%) = quicksort_data%(quicksort_j%)
2340                     quicksort_data%(quicksort_j%) = quicksort_temp%
2350                 REM END IF
2360             NEXT quicksort_j%

2370             ' Place pivot at wall.
2380             quicksort_wall% = quicksort_wall% + 1
2390             quicksort_temp% = quicksort_data%(quicksort_wall%)
2400             quicksort_data%(quicksort_wall%) = quicksort_data%(quicksort_qhigh%)
2410             quicksort_data%(quicksort_qhigh%) = quicksort_temp%

2420             IF ((quicksort_wall% - 1) > quicksort_qlow%) = 0 THEN GOTO 2460
2430                 quicksort_stop% = quicksort_stop% + 1
2440                 quicksort_slow%(quicksort_stop%) = quicksort_qlow%
2450                 quicksort_shigh%(quicksort_stop%) = quicksort_wall% - 1
2460             REM END IF

2470             IF ((quicksort_wall% + 1) < quicksort_qhigh%) = 0 THEN GOTO 2510
2480                 quicksort_stop% = quicksort_stop% + 1
2490                 quicksort_slow%(quicksort_stop%) = quicksort_wall% + 1
2500                 quicksort_shigh%(quicksort_stop%) = quicksort_qhigh%
2510             REM END IF
2520         REM END IF
2530         GOTO 2160
2540     REM END WHILE

2550     quicksort_result% = 0
2560     RETURN
2570 ' end function quickSort%
