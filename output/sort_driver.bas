' BASCAL generated BASIC
' Functions are lowered to global variables, labels, and GOSUB

' In-place bubble sort.
' data% is an array parameter; bcc lowers calls with copy-in/copy-out.
' In-place cocktail shaker sort.
' This is a bidirectional bubble sort over the active range.
' Tiny helper used to prove recursive require resolution.
' Shell sort demonstrates a nested dependency. The helper is intentionally
' trivial; the point is to exercise recursive require resolution.

' Iterative quicksort using an explicit stack for partition bounds.
' Middle-element pivot avoids O(n^2) on already-sorted or reverse-sorted input.
' Sort driver for the BASCAL example sort library.
' Uses 5000 reverse-sorted elements (worst case for comparison sorts).

DIM source%(5000)
DIM bubbleData%(5000)
DIM shakerData%(5000)
DIM shellData%(5000)
DIM quickData%(5000)

' Fill with descending values: worst case for O(n^2) sorts.
FOR i% = 1 TO 5000
    source%(i%) = 1001 - i%
NEXT i%

FOR i% = 1 TO 5000
    bubbleData%(i%) = source%(i%)
    shakerData%(i%) = source%(i%)
    shellData%(i%) = source%(i%)
    quickData%(i%) = source%(i%)
NEXT i%

tStart# = TIMER
bubblesort_count% = 5000
DIM bubblesort_data%(5000)

' copy array argument into lowered function storage: bubbleData%() -> bubblesort_data%()
FOR BCC_T1% = 1 TO 5000
    bubblesort_data%(BCC_T1%) = bubbleData%(BCC_T1%)
NEXT BCC_T1%

GOSUB 130

' copy mutated array argument back to caller storage: bubblesort_data%() -> bubbleData%()
FOR BCC_T2% = 1 TO 5000
    bubbleData%(BCC_T2%) = bubblesort_data%(BCC_T2%)
NEXT BCC_T2%

tElapsed# = TIMER - tStart#
PRINT "Bubble sort time (ms):", tElapsed# * 5000
ok% = 1
FOR i% = 1 TO 4999
    IF (bubbleData%(i%) > bubbleData%(i% + 1)) = 0 THEN GOTO 10
        ok% = 0
10 REM END IF
NEXT i%
IF (ok% = 1) = 0 THEN GOTO 20
    PRINT "Bubble: OK"
    GOTO 30
20 PRINT "Bubble: FAILED"
30 REM END IF

tStart# = TIMER
shakersort_count% = 5000
DIM shakersort_data%(5000)

' copy array argument into lowered function storage: shakerData%() -> shakersort_data%()
FOR BCC_T5% = 1 TO 5000
    shakersort_data%(BCC_T5%) = shakerData%(BCC_T5%)
NEXT BCC_T5%

GOSUB 150

' copy mutated array argument back to caller storage: shakersort_data%() -> shakerData%()
FOR BCC_T6% = 1 TO 5000
    shakerData%(BCC_T6%) = shakersort_data%(BCC_T6%)
NEXT BCC_T6%

tElapsed# = TIMER - tStart#
PRINT "Shaker sort time (ms):", tElapsed# * 5000
ok% = 1
FOR i% = 1 TO 4999
    IF (shakerData%(i%) > shakerData%(i% + 1)) = 0 THEN GOTO 40
        ok% = 0
40 REM END IF
NEXT i%
IF (ok% = 1) = 0 THEN GOTO 50
    PRINT "Shaker: OK"
    GOTO 60
50 PRINT "Shaker: FAILED"
60 REM END IF

tStart# = TIMER
shellsort_count% = 5000
DIM shellsort_data%(5000)

' copy array argument into lowered function storage: shellData%() -> shellsort_data%()
FOR BCC_T9% = 1 TO 5000
    shellsort_data%(BCC_T9%) = shellData%(BCC_T9%)
NEXT BCC_T9%

GOSUB 210

' copy mutated array argument back to caller storage: shellsort_data%() -> shellData%()
FOR BCC_T10% = 1 TO 5000
    shellData%(BCC_T10%) = shellsort_data%(BCC_T10%)
NEXT BCC_T10%

tElapsed# = TIMER - tStart#
PRINT "Shell sort time (ms):", tElapsed# * 5000
ok% = 1
FOR i% = 1 TO 4999
    IF (shellData%(i%) > shellData%(i% + 1)) = 0 THEN GOTO 70
        ok% = 0
70 REM END IF
NEXT i%
IF (ok% = 1) = 0 THEN GOTO 80
    PRINT "Shell: OK"
    GOTO 90
80 PRINT "Shell: FAILED"
90 REM END IF

tStart# = TIMER
quicksort_count% = 5000
DIM quicksort_data%(5000)

' copy array argument into lowered function storage: quickData%() -> quicksort_data%()
FOR BCC_T13% = 1 TO 5000
    quicksort_data%(BCC_T13%) = quickData%(BCC_T13%)
NEXT BCC_T13%

GOSUB 280

' copy mutated array argument back to caller storage: quicksort_data%() -> quickData%()
FOR BCC_T14% = 1 TO 5000
    quickData%(BCC_T14%) = quicksort_data%(BCC_T14%)
NEXT BCC_T14%

tElapsed# = TIMER - tStart#
PRINT "Quick sort time (ms):", tElapsed# * 5000
ok% = 1
FOR i% = 1 TO 4999
    IF (quickData%(i%) > quickData%(i% + 1)) = 0 THEN GOTO 100
        ok% = 0
100 REM END IF
NEXT i%
IF (ok% = 1) = 0 THEN GOTO 110
    PRINT "Quick: OK"
    GOTO 120
110 PRINT "Quick: FAILED"
120 REM END IF

END

' function bubbleSort%(data%, count%)
130 ' After each outer pass, the largest remaining value has bubbled right.
    FOR i% = 1 TO bubblesort_count% - 1
        FOR j% = 1 TO bubblesort_count% - i%
            IF (bubblesort_data%(j%) > bubblesort_data%(j% + 1)) = 0 THEN GOTO 140
                ' Swap adjacent out-of-order elements.
                temp% = bubblesort_data%(j%)
                bubblesort_data%(j%) = bubblesort_data%(j% + 1)
                bubblesort_data%(j% + 1) = temp%
140 REM END IF
        NEXT j%
    NEXT i%
    bubblesort_result% = 0
    RETURN
' end function bubbleSort%

' function shakerSort%(data%, count%)
150 left% = 1
    right% = shakersort_count% - 1
    swapped% = 1

    ' Continue until a full bidirectional pass performs no swaps.
160 IF (swapped%) = 0 THEN GOTO 190
        swapped% = 0

        ' Forward pass moves large values toward the right edge.
        FOR i% = left% TO right%
            IF (shakersort_data%(i%) > shakersort_data%(i% + 1)) = 0 THEN GOTO 170
                temp% = shakersort_data%(i%)
                shakersort_data%(i%) = shakersort_data%(i% + 1)
                shakersort_data%(i% + 1) = temp%
                swapped% = 1
170 REM END IF
        NEXT i%

        right% = right% - 1

        ' Backward pass moves small values toward the left edge.
        FOR i% = right% TO left% STEP -1
            IF (shakersort_data%(i%) > shakersort_data%(i% + 1)) = 0 THEN GOTO 180
                temp% = shakersort_data%(i%)
                shakersort_data%(i%) = shakersort_data%(i% + 1)
                shakersort_data%(i% + 1) = temp%
                swapped% = 1
180 REM END IF
        NEXT i%

        left% = left% + 1
        GOTO 160
190 REM END WHILE

    shakersort_result% = 0
    RETURN
' end function shakerSort%

' function touch%(value%)
200 ' Return the value unchanged.
    touch_result% = touch_value%
    RETURN
' end function touch%

' function shellSort%(data%, count%)
210 ' Normalize the count through a required helper so this file has its own
    ' dependency chain.
    touch_value% = shellsort_count%
    GOSUB 200
    shellsort_count% = touch_result%
    gap% = shellsort_count% / 2

    ' Repeatedly insertion-sort elements that are gap positions apart.
220 IF (gap% > 0) = 0 THEN GOTO 270
        i% = gap% + 1

230 IF (i% <= shellsort_count%) = 0 THEN GOTO 260
            temp% = shellsort_data%(i%)
            j% = i%

240 IF ((j% > gap%) AND (shellsort_data%(j% - gap%) > temp%)) = 0 THEN GOTO 250
                shellsort_data%(j%) = shellsort_data%(j% - gap%)
                j% = j% - gap%
                GOTO 240
250 REM END WHILE

            shellsort_data%(j%) = temp%
            i% = i% + 1
            GOTO 230
260 REM END WHILE

        gap% = gap% / 2
        GOTO 220
270 REM END WHILE

    shellsort_result% = 0
    RETURN
' end function shellSort%

' function quickSort%(data%, count%)
280 DIM sLow%(64)
    DIM sHigh%(64)

    sTop% = 1
    sLow%(1) = 1
    sHigh%(1) = quicksort_count%

290 IF (sTop% > 0) = 0 THEN GOTO 340
        qHigh% = sHigh%(sTop%)
        qLow% = sLow%(sTop%)
        sTop% = sTop% - 1

        IF (qHigh% > qLow%) = 0 THEN GOTO 330
            ' Swap middle element to high as pivot.
            mid% = qLow% + ((qHigh% - qLow%) / 2)
            temp% = quicksort_data%(mid%)
            quicksort_data%(mid%) = quicksort_data%(qHigh%)
            quicksort_data%(qHigh%) = temp%

            ' Partition: move elements <= pivot left of wall.
            pivot% = quicksort_data%(qHigh%)
            wall% = qLow% - 1

            FOR j% = qLow% TO qHigh% - 1
                IF (quicksort_data%(j%) <= pivot%) = 0 THEN GOTO 300
                    wall% = wall% + 1
                    temp% = quicksort_data%(wall%)
                    quicksort_data%(wall%) = quicksort_data%(j%)
                    quicksort_data%(j%) = temp%
300 REM END IF
            NEXT j%

            ' Place pivot at wall.
            wall% = wall% + 1
            temp% = quicksort_data%(wall%)
            quicksort_data%(wall%) = quicksort_data%(qHigh%)
            quicksort_data%(qHigh%) = temp%

            IF ((wall% - 1) > qLow%) = 0 THEN GOTO 310
                sTop% = sTop% + 1
                sLow%(sTop%) = qLow%
                sHigh%(sTop%) = wall% - 1
310 REM END IF

            IF ((wall% + 1) < qHigh%) = 0 THEN GOTO 320
                sTop% = sTop% + 1
                sLow%(sTop%) = wall% + 1
                sHigh%(sTop%) = qHigh%
320 REM END IF
330 REM END IF
        GOTO 290
340 REM END WHILE

    quicksort_result% = 0
    RETURN
' end function quickSort%
