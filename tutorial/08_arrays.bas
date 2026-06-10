' BASCAL generated BASIC
' Functions are lowered to global variables, labels, and GOSUB

' Tutorial 8 — Arrays
' 
' DIM name%(size) declares an array of size+1 elements indexed 0..size.
' Array elements are accessed with parentheses: arr%(i%).
' 
' To pass an array to a function, use the plain variable name as the
' parameter (e.g. arr%, not arr%()).  At the call site, write arr%()
' to signal that an array is being passed.

' Declare and populate
CONST N% = 6
DIM data%(N%)

data%(0) = 64
data%(1) = 25
data%(2) = 12
data%(3) = 22
data%(4) = 3
data%(5) = 11

' Insertion sort — sorts data%() in place

' Linear search — returns index or -1

' Print the array on one line as  [ a b c ... ]

' Before sort
PRINT "Before: "
printarray_count% = N%
DIM printarray_arr%(N%)

' copy array argument into lowered function storage: data%() -> printarray_arr%()
FOR BCC_T1% = 1 TO N%
    printarray_arr%(BCC_T1%) = data%(BCC_T1%)
NEXT BCC_T1%

GOSUB 80

' copy mutated array argument back to caller storage: printarray_arr%() -> data%()
FOR BCC_T2% = 1 TO N%
    data%(BCC_T2%) = printarray_arr%(BCC_T2%)
NEXT BCC_T2%

dummy% = printarray_result%

' Sort and show
insertionsort_count% = N%
DIM insertionsort_arr%(N%)

' copy array argument into lowered function storage: data%() -> insertionsort_arr%()
FOR BCC_T3% = 1 TO N%
    insertionsort_arr%(BCC_T3%) = data%(BCC_T3%)
NEXT BCC_T3%

GOSUB 30

' copy mutated array argument back to caller storage: insertionsort_arr%() -> data%()
FOR BCC_T4% = 1 TO N%
    data%(BCC_T4%) = insertionsort_arr%(BCC_T4%)
NEXT BCC_T4%

dummy% = insertionsort_result%
PRINT "After:  "
printarray_count% = N%
DIM printarray_arr%(N%)

' copy array argument into lowered function storage: data%() -> printarray_arr%()
FOR BCC_T5% = 1 TO N%
    printarray_arr%(BCC_T5%) = data%(BCC_T5%)
NEXT BCC_T5%

GOSUB 80

' copy mutated array argument back to caller storage: printarray_arr%() -> data%()
FOR BCC_T6% = 1 TO N%
    data%(BCC_T6%) = printarray_arr%(BCC_T6%)
NEXT BCC_T6%

dummy% = printarray_result%

' Search
target% = 22
indexof_count% = N%
indexof_target% = target%
DIM indexof_arr%(N%)

' copy array argument into lowered function storage: data%() -> indexof_arr%()
FOR BCC_T7% = 1 TO N%
    indexof_arr%(BCC_T7%) = data%(BCC_T7%)
NEXT BCC_T7%

GOSUB 60

' copy mutated array argument back to caller storage: indexof_arr%() -> data%()
FOR BCC_T8% = 1 TO N%
    data%(BCC_T8%) = indexof_arr%(BCC_T8%)
NEXT BCC_T8%

idx% = indexof_result%
IF (idx% >= 0) = 0 THEN GOTO 10
    PRINT (STR$(target%) + " found at index ") + STR$(idx%)
    GOTO 20
10 PRINT STR$(target%) + " not found"
20 REM END IF

END

' function insertionSort%(arr%, count%)
30 FOR i% = 1 TO insertionsort_count% - 1
        key% = insertionsort_arr%(i%)
        j% = i% - 1
40 IF ((j% >= 0) AND (insertionsort_arr%(j%) > key%)) = 0 THEN GOTO 50
            insertionsort_arr%(j% + 1) = insertionsort_arr%(j%)
            j% = j% - 1
            GOTO 40
50 REM END WHILE
        insertionsort_arr%(j% + 1) = key%
    NEXT i%
    insertionsort_result% = 0
    RETURN
' end function insertionSort%

' function indexOf%(arr%, count%, target%)
60 FOR i% = 0 TO indexof_count% - 1
        IF (indexof_arr%(i%) = indexof_target%) = 0 THEN GOTO 70
            indexof_result% = i%
            RETURN
70 REM END IF
    NEXT i%
    indexof_result% = -1
    RETURN
' end function indexOf%

' function printArray%(arr%, count%)
80 line$ = "["
    FOR i% = 0 TO printarray_count% - 1
        line$ = (line$ + " ") + STR$(printarray_arr%(i%))
    NEXT i%
    PRINT line$ + " ]"
    printarray_result% = 0
    RETURN
' end function printArray%
