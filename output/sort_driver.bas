' BASCAL generated BASIC
' Functions are lowered to global variables, labels, and GOSUB
' In-place bubble sort.
' data% is an array parameter; bcc lowers calls with copy-in/copy-out.
' In-place cocktail shaker sort.
' This is a bidirectional bubble sort over the active range.
' Tiny helper used to prove recursive require resolution.
' Shell sort demonstrates a nested dependency. The helper is intentionally
' trivial; the point is to exercise recursive require resolution.
' Selection-sort implementation behind the quickSort% selector.
' The name is kept to exercise dependency selection while avoiding recursion.
' Sort driver for the BASCAL example sort library.
' The require paths select source files; they do not create runtime namespaces.
DIM source%(10)
DIM bubbleData%(10)
DIM shakerData%(10)
DIM shellData%(10)
DIM quickData%(10)
' Populate one source array and copy it into one array per algorithm.
source%(1) = 42
source%(2) = 7
source%(3) = 19
source%(4) = 3
source%(5) = 88
source%(6) = 12
source%(7) = 55
source%(8) = 1
source%(9) = 34
source%(10) = 21
FOR i% = 1 TO 10
    bubbleData%(i%) = source%(i%)
    shakerData%(i%) = source%(i%)
    shellData%(i%) = source%(i%)
    quickData%(i%) = source%(i%)
NEXT i%
PRINT "Original data:"
FOR i% = 1 TO 10
    PRINT source%(i%)
NEXT i%
' Each sort mutates its array argument. The compiler lowers this to copy-in,
' GOSUB, and copy-out code in generated BASIC.
bubblesort_count% = 10
DIM bubblesort_data%(10)
' copy array argument into lowered function storage: bubbleData%() -> bubblesort_data%()
FOR BCC_COPY% = 1 TO 10
    bubblesort_data%(BCC_COPY%) = bubbleData%(BCC_COPY%)
NEXT BCC_COPY%
GOSUB 10
' copy mutated array argument back to caller storage: bubblesort_data%() -> bubbleData%()
FOR BCC_COPY% = 1 TO 10
    bubbleData%(BCC_COPY%) = bubblesort_data%(BCC_COPY%)
NEXT BCC_COPY%
shakersort_count% = 10
DIM shakersort_data%(10)
' copy array argument into lowered function storage: shakerData%() -> shakersort_data%()
FOR BCC_COPY% = 1 TO 10
    shakersort_data%(BCC_COPY%) = shakerData%(BCC_COPY%)
NEXT BCC_COPY%
GOSUB 30
' copy mutated array argument back to caller storage: shakersort_data%() -> shakerData%()
FOR BCC_COPY% = 1 TO 10
    shakerData%(BCC_COPY%) = shakersort_data%(BCC_COPY%)
NEXT BCC_COPY%
shellsort_count% = 10
DIM shellsort_data%(10)
' copy array argument into lowered function storage: shellData%() -> shellsort_data%()
FOR BCC_COPY% = 1 TO 10
    shellsort_data%(BCC_COPY%) = shellData%(BCC_COPY%)
NEXT BCC_COPY%
GOSUB 70
' copy mutated array argument back to caller storage: shellsort_data%() -> shellData%()
FOR BCC_COPY% = 1 TO 10
    shellData%(BCC_COPY%) = shellsort_data%(BCC_COPY%)
NEXT BCC_COPY%
quicksort_count% = 10
DIM quicksort_data%(10)
' copy array argument into lowered function storage: quickData%() -> quicksort_data%()
FOR BCC_COPY% = 1 TO 10
    quicksort_data%(BCC_COPY%) = quickData%(BCC_COPY%)
NEXT BCC_COPY%
GOSUB 80
' copy mutated array argument back to caller storage: quicksort_data%() -> quickData%()
FOR BCC_COPY% = 1 TO 10
    quickData%(BCC_COPY%) = quicksort_data%(BCC_COPY%)
NEXT BCC_COPY%
PRINT "Bubble sort result:"
FOR i% = 1 TO 10
    PRINT bubbleData%(i%)
NEXT i%
PRINT "Shaker sort result:"
FOR i% = 1 TO 10
    PRINT shakerData%(i%)
NEXT i%
PRINT "Shell sort result:"
FOR i% = 1 TO 10
    PRINT shellData%(i%)
NEXT i%
PRINT "Quick sort result:"
FOR i% = 1 TO 10
    PRINT quickData%(i%)
NEXT i%
END
' ===== BEGIN FUNCTION bubbleSort% =====
10     ' After each outer pass, the largest remaining value has bubbled right.
    FOR i% = 1 TO bubblesort_count% - 1
        FOR j% = 1 TO bubblesort_count% - i%
            IF NOT (bubblesort_data%(j%) > bubblesort_data%(j% + 1)) THEN GOTO 20
            ' Swap adjacent out-of-order elements.
            temp% = bubblesort_data%(j%)
            bubblesort_data%(j%) = bubblesort_data%(j% + 1)
            bubblesort_data%(j% + 1) = temp%
20             REM END IF
        NEXT j%
    NEXT i%
    bubblesort_result% = 0
    RETURN
' ===== END FUNCTION bubbleSort% =====
' ===== BEGIN FUNCTION shakerSort% =====
30     left% = 1
    right% = shakersort_count% - 1
    swapped% = 1
    ' Continue until a full bidirectional pass performs no swaps.
    WHILE swapped%
        swapped% = 0
        ' Forward pass moves large values toward the right edge.
        FOR i% = left% TO right%
            IF NOT (shakersort_data%(i%) > shakersort_data%(i% + 1)) THEN GOTO 40
            temp% = shakersort_data%(i%)
            shakersort_data%(i%) = shakersort_data%(i% + 1)
            shakersort_data%(i% + 1) = temp%
            swapped% = 1
40             REM END IF
        NEXT i%
        right% = right% - 1
        ' Backward pass moves small values toward the left edge.
        FOR i% = right% TO left% STEP -1
            IF NOT (shakersort_data%(i%) > shakersort_data%(i% + 1)) THEN GOTO 50
            temp% = shakersort_data%(i%)
            shakersort_data%(i%) = shakersort_data%(i% + 1)
            shakersort_data%(i% + 1) = temp%
            swapped% = 1
50             REM END IF
        NEXT i%
        left% = left% + 1
    WEND
    shakersort_result% = 0
    RETURN
' ===== END FUNCTION shakerSort% =====
' ===== BEGIN FUNCTION touch% =====
60     ' Return the value unchanged.
    touch_result% = touch_value%
    RETURN
' ===== END FUNCTION touch% =====
' ===== BEGIN FUNCTION shellSort% =====
70     ' Normalize the count through a required helper so this file has its own
    ' dependency chain.
    touch_value% = shellsort_count%
    GOSUB 60
    shellsort_count% = touch_result%
    gap% = shellsort_count% / 2
    ' Repeatedly insertion-sort elements that are gap positions apart.
    WHILE gap% > 0
        i% = gap% + 1
        WHILE i% <= shellsort_count%
            temp% = shellsort_data%(i%)
            j% = i%
            WHILE j% > gap% AND shellsort_data%(j% - gap%) > temp%
                shellsort_data%(j%) = shellsort_data%(j% - gap%)
                j% = j% - gap%
            WEND
            shellsort_data%(j%) = temp%
            i% = i% + 1
        WEND
        gap% = gap% / 2
    WEND
    shellsort_result% = 0
    RETURN
' ===== END FUNCTION shellSort% =====
' ===== BEGIN FUNCTION quickSort% =====
80     ' Pick the smallest remaining value and move it into position i%.
    FOR i% = 1 TO quicksort_count% - 1
        min% = i%
        FOR j% = i% + 1 TO quicksort_count%
            IF NOT (quicksort_data%(j%) < quicksort_data%(min%)) THEN GOTO 90
            min% = j%
90             REM END IF
        NEXT j%
        IF NOT (min% <> i%) THEN GOTO 100
        ' Swap the selected minimum into the sorted prefix.
        temp% = quicksort_data%(i%)
        quicksort_data%(i%) = quicksort_data%(min%)
        quicksort_data%(min%) = temp%
100         REM END IF
    NEXT i%
    quicksort_result% = 0
    RETURN
' ===== END FUNCTION quickSort% =====
