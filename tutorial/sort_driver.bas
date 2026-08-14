' BASCAL generated BASIC
' Functions are transpiled to global variables, labels, and GOSUB

' Storage for array parameters, sized to fit every call site
DIM bubblesort_data_0%(5000)
DIM shakersort_data_0%(5000)
DIM shellsort_data_0%(5000)
DIM quicksort_data_0%(5000)

' In-place bubble sort.
' data% is a byref array parameter, so bcc copies the sorted result back
' into the caller's array; byval (the default) would sort a private copy
' and discard it.
' data% -- array to sort, mutated in place
' In-place cocktail shaker sort.
' This is a bidirectional bubble sort over the active range. data% is
' byref so the sorted result is copied back to the caller.
' data% -- array to sort, mutated in place
' Tiny helper used to prove recursive require resolution.
' value% -- number passed through unchanged
' Shell sort demonstrates a nested dependency. The helper is intentionally
' trivial; the point is to exercise recursive require resolution.

' data% -- array to sort, mutated in place
' Iterative quicksort using an explicit stack for partition bounds.
' Middle-element pivot avoids O(n^2) on already-sorted or reverse-sorted input.
' data% is byref so the sorted result is copied back to the caller.
' data% -- array to sort, mutated in place
' Sort driver for the BASCAL example sort library.
' Uses 5000 reverse-sorted elements (worst case for comparison sorts).

DIM source%(5000)
DIM bubbledata%(5000)
DIM shakerdata%(5000)
DIM shelldata%(5000)
DIM quickdata%(5000)

' Fill with descending values: worst case for O(n^2) sorts.
FOR i% = 1 TO 5000
    source%(i%) = 1001 - i%
NEXT i%

FOR i% = 1 TO 5000
    bubbledata%(i%) = source%(i%)
    shakerdata%(i%) = source%(i%)
    shelldata%(i%) = source%(i%)
    quickdata%(i%) = source%(i%)
NEXT i%

tstart# = TIMER
bubblesort_data_dim0_0% = 5000
IF bubblesort_data_dim0_0% > 5000 THEN PRINT "runtime error: `data%` of `bubbleSort%` needs "; bubblesort_data_dim0_0%; " elements along axis 0, but its storage only holds 5000" : STOP

' copy array argument into transpiled function storage: bubbledata%() -> bubblesort_data_0%()
FOR BCC_T1% = 1 TO bubblesort_data_dim0_0%
    bubblesort_data_0%(BCC_T1%) = bubbledata%(BCC_T1%)
NEXT BCC_T1%

GOSUB 130

' copy mutated array argument back to caller storage: bubblesort_data_0%() -> bubbledata%()
FOR BCC_T2% = 1 TO bubblesort_data_dim0_0%
    bubbledata%(BCC_T2%) = bubblesort_data_0%(BCC_T2%)
NEXT BCC_T2%

telapsed# = TIMER - tstart#
PRINT "Bubble sort time (ms):", telapsed# * 5000
ok% = 1
FOR i% = 1 TO 4999
    IF (bubbledata%(i%) > bubbledata%(i% + 1)) = 0 THEN GOTO 10
        ok% = 0
10 REM END IF
NEXT i%
IF (ok% = 1) = 0 THEN GOTO 20
    PRINT "Bubble: OK"
    GOTO 30
20 PRINT "Bubble: FAILED"
30 REM END IF

tstart# = TIMER
shakersort_data_dim0_0% = 5000
IF shakersort_data_dim0_0% > 5000 THEN PRINT "runtime error: `data%` of `shakerSort%` needs "; shakersort_data_dim0_0%; " elements along axis 0, but its storage only holds 5000" : STOP

' copy array argument into transpiled function storage: shakerdata%() -> shakersort_data_0%()
FOR BCC_T5% = 1 TO shakersort_data_dim0_0%
    shakersort_data_0%(BCC_T5%) = shakerdata%(BCC_T5%)
NEXT BCC_T5%

GOSUB 150

' copy mutated array argument back to caller storage: shakersort_data_0%() -> shakerdata%()
FOR BCC_T6% = 1 TO shakersort_data_dim0_0%
    shakerdata%(BCC_T6%) = shakersort_data_0%(BCC_T6%)
NEXT BCC_T6%

telapsed# = TIMER - tstart#
PRINT "Shaker sort time (ms):", telapsed# * 5000
ok% = 1
FOR i% = 1 TO 4999
    IF (shakerdata%(i%) > shakerdata%(i% + 1)) = 0 THEN GOTO 40
        ok% = 0
40 REM END IF
NEXT i%
IF (ok% = 1) = 0 THEN GOTO 50
    PRINT "Shaker: OK"
    GOTO 60
50 PRINT "Shaker: FAILED"
60 REM END IF

tstart# = TIMER
shellsort_data_dim0_0% = 5000
IF shellsort_data_dim0_0% > 5000 THEN PRINT "runtime error: `data%` of `shellSort%` needs "; shellsort_data_dim0_0%; " elements along axis 0, but its storage only holds 5000" : STOP

' copy array argument into transpiled function storage: shelldata%() -> shellsort_data_0%()
FOR BCC_T9% = 1 TO shellsort_data_dim0_0%
    shellsort_data_0%(BCC_T9%) = shelldata%(BCC_T9%)
NEXT BCC_T9%

GOSUB 210

' copy mutated array argument back to caller storage: shellsort_data_0%() -> shelldata%()
FOR BCC_T10% = 1 TO shellsort_data_dim0_0%
    shelldata%(BCC_T10%) = shellsort_data_0%(BCC_T10%)
NEXT BCC_T10%

telapsed# = TIMER - tstart#
PRINT "Shell sort time (ms):", telapsed# * 5000
ok% = 1
FOR i% = 1 TO 4999
    IF (shelldata%(i%) > shelldata%(i% + 1)) = 0 THEN GOTO 70
        ok% = 0
70 REM END IF
NEXT i%
IF (ok% = 1) = 0 THEN GOTO 80
    PRINT "Shell: OK"
    GOTO 90
80 PRINT "Shell: FAILED"
90 REM END IF

tstart# = TIMER
quicksort_data_dim0_0% = 5000
IF quicksort_data_dim0_0% > 5000 THEN PRINT "runtime error: `data%` of `quickSort%` needs "; quicksort_data_dim0_0%; " elements along axis 0, but its storage only holds 5000" : STOP

' copy array argument into transpiled function storage: quickdata%() -> quicksort_data_0%()
FOR BCC_T13% = 1 TO quicksort_data_dim0_0%
    quicksort_data_0%(BCC_T13%) = quickdata%(BCC_T13%)
NEXT BCC_T13%

GOSUB 280

' copy mutated array argument back to caller storage: quicksort_data_0%() -> quickdata%()
FOR BCC_T14% = 1 TO quicksort_data_dim0_0%
    quickdata%(BCC_T14%) = quicksort_data_0%(BCC_T14%)
NEXT BCC_T14%

telapsed# = TIMER - tstart#
PRINT "Quick sort time (ms):", telapsed# * 5000
ok% = 1
FOR i% = 1 TO 4999
    IF (quickdata%(i%) > quickdata%(i% + 1)) = 0 THEN GOTO 100
        ok% = 0
100 REM END IF
NEXT i%
IF (ok% = 1) = 0 THEN GOTO 110
    PRINT "Quick: OK"
    GOTO 120
110 PRINT "Quick: FAILED"
120 REM END IF

END

' function bubblesort%(data%)
130 bubblesort_count_0% = bubblesort_data_dim0_0%
    ' After each outer pass, the largest remaining value has bubbled right.
    FOR bubblesort_i_0% = 1 TO bubblesort_count_0% - 1
        FOR bubblesort_j_0% = 1 TO bubblesort_count_0% - bubblesort_i_0%
            IF (bubblesort_data_0%(bubblesort_j_0%) > bubblesort_data_0%(bubblesort_j_0% + 1)) = 0 THEN GOTO 140
                ' Swap adjacent out-of-order elements.
                bubblesort_temp_0% = bubblesort_data_0%(bubblesort_j_0%)
                bubblesort_data_0%(bubblesort_j_0%) = bubblesort_data_0%(bubblesort_j_0% + 1)
                bubblesort_data_0%(bubblesort_j_0% + 1) = bubblesort_temp_0%
140 REM END IF
        NEXT bubblesort_j_0%
    NEXT bubblesort_i_0%
    bubblesort_result_0% = 0
    RETURN
' end function bubblesort%

' function shakersort%(data%)
150 shakersort_count_0% = shakersort_data_dim0_0%
    LEFT% = 1
    RIGHT% = shakersort_count_0% - 1
    shakersort_swapped_0% = 1

    ' Continue until a full bidirectional pass performs no swaps.
160 IF (shakersort_swapped_0%) = 0 THEN GOTO 190
        shakersort_swapped_0% = 0

        ' Forward pass moves large values toward the right edge.
        FOR shakersort_i_0% = LEFT% TO RIGHT%
            IF (shakersort_data_0%(shakersort_i_0%) > shakersort_data_0%(shakersort_i_0% + 1)) = 0 THEN GOTO 170
                shakersort_temp_0% = shakersort_data_0%(shakersort_i_0%)
                shakersort_data_0%(shakersort_i_0%) = shakersort_data_0%(shakersort_i_0% + 1)
                shakersort_data_0%(shakersort_i_0% + 1) = shakersort_temp_0%
                shakersort_swapped_0% = 1
170 REM END IF
        NEXT shakersort_i_0%

        RIGHT% = RIGHT% - 1

        ' Backward pass moves small values toward the left edge.
        FOR shakersort_i_0% = RIGHT% TO LEFT% STEP -1
            IF (shakersort_data_0%(shakersort_i_0%) > shakersort_data_0%(shakersort_i_0% + 1)) = 0 THEN GOTO 180
                shakersort_temp_0% = shakersort_data_0%(shakersort_i_0%)
                shakersort_data_0%(shakersort_i_0%) = shakersort_data_0%(shakersort_i_0% + 1)
                shakersort_data_0%(shakersort_i_0% + 1) = shakersort_temp_0%
                shakersort_swapped_0% = 1
180 REM END IF
        NEXT shakersort_i_0%

        LEFT% = LEFT% + 1
        GOTO 160
190 REM END WHILE

    shakersort_result_0% = 0
    RETURN
' end function shakersort%

' function touch%(value%)
200 ' Return the value unchanged.
    touch_result_0% = touch_value_0%
    RETURN
' end function touch%

' function shellsort%(data%)
210 ' Normalize the count through a required helper so this file has its own
    ' dependency chain.
    touch_value_0% = shellsort_data_dim0_0%
    GOSUB 200
    shellsort_count_0% = touch_result_0%
    shellsort_gap_0% = shellsort_count_0% / 2

    ' Repeatedly insertion-sort elements that are gap positions apart.
220 IF (shellsort_gap_0% > 0) = 0 THEN GOTO 270
        shellsort_i_0% = shellsort_gap_0% + 1

230 IF (shellsort_i_0% <= shellsort_count_0%) = 0 THEN GOTO 260
            shellsort_temp_0% = shellsort_data_0%(shellsort_i_0%)
            shellsort_j_0% = shellsort_i_0%

240 IF ((shellsort_j_0% > shellsort_gap_0%) AND (shellsort_data_0%(shellsort_j_0% - shellsort_gap_0%) > shellsort_temp_0%)) = 0 THEN GOTO 250
                shellsort_data_0%(shellsort_j_0%) = shellsort_data_0%(shellsort_j_0% - shellsort_gap_0%)
                shellsort_j_0% = shellsort_j_0% - shellsort_gap_0%
                GOTO 240
250 REM END WHILE

            shellsort_data_0%(shellsort_j_0%) = shellsort_temp_0%
            shellsort_i_0% = shellsort_i_0% + 1
            GOTO 230
260 REM END WHILE

        shellsort_gap_0% = shellsort_gap_0% / 2
        GOTO 220
270 REM END WHILE

    shellsort_result_0% = 0
    RETURN
' end function shellsort%

' function quicksort%(data%)
280 DIM quicksort_slow_0%(64)
    DIM quicksort_shigh_0%(64)

    quicksort_stop_0% = 1
    quicksort_slow_0%(1) = 1
    quicksort_shigh_0%(1) = quicksort_data_dim0_0%

290 IF (quicksort_stop_0% > 0) = 0 THEN GOTO 340
        quicksort_qhigh_0% = quicksort_shigh_0%(quicksort_stop_0%)
        quicksort_qlow_0% = quicksort_slow_0%(quicksort_stop_0%)
        quicksort_stop_0% = quicksort_stop_0% - 1

        IF (quicksort_qhigh_0% > quicksort_qlow_0%) = 0 THEN GOTO 330
            ' Swap middle element to high as pivot.
            MID% = quicksort_qlow_0% + ((quicksort_qhigh_0% - quicksort_qlow_0%) / 2)
            quicksort_temp_0% = quicksort_data_0%(MID%)
            quicksort_data_0%(MID%) = quicksort_data_0%(quicksort_qhigh_0%)
            quicksort_data_0%(quicksort_qhigh_0%) = quicksort_temp_0%

            ' Partition: move elements <= pivot left of wall.
            quicksort_pivot_0% = quicksort_data_0%(quicksort_qhigh_0%)
            quicksort_wall_0% = quicksort_qlow_0% - 1

            FOR quicksort_j_0% = quicksort_qlow_0% TO quicksort_qhigh_0% - 1
                IF (quicksort_data_0%(quicksort_j_0%) <= quicksort_pivot_0%) = 0 THEN GOTO 300
                    quicksort_wall_0% = quicksort_wall_0% + 1
                    quicksort_temp_0% = quicksort_data_0%(quicksort_wall_0%)
                    quicksort_data_0%(quicksort_wall_0%) = quicksort_data_0%(quicksort_j_0%)
                    quicksort_data_0%(quicksort_j_0%) = quicksort_temp_0%
300 REM END IF
            NEXT quicksort_j_0%

            ' Place pivot at wall.
            quicksort_wall_0% = quicksort_wall_0% + 1
            quicksort_temp_0% = quicksort_data_0%(quicksort_wall_0%)
            quicksort_data_0%(quicksort_wall_0%) = quicksort_data_0%(quicksort_qhigh_0%)
            quicksort_data_0%(quicksort_qhigh_0%) = quicksort_temp_0%

            IF ((quicksort_wall_0% - 1) > quicksort_qlow_0%) = 0 THEN GOTO 310
                quicksort_stop_0% = quicksort_stop_0% + 1
                quicksort_slow_0%(quicksort_stop_0%) = quicksort_qlow_0%
                quicksort_shigh_0%(quicksort_stop_0%) = quicksort_wall_0% - 1
310 REM END IF

            IF ((quicksort_wall_0% + 1) < quicksort_qhigh_0%) = 0 THEN GOTO 320
                quicksort_stop_0% = quicksort_stop_0% + 1
                quicksort_slow_0%(quicksort_stop_0%) = quicksort_wall_0% + 1
                quicksort_shigh_0%(quicksort_stop_0%) = quicksort_qhigh_0%
320 REM END IF
330 REM END IF
        GOTO 290
340 REM END WHILE

    quicksort_result_0% = 0
    RETURN
' end function quicksort%
