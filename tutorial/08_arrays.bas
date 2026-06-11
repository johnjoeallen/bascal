' BASCAL generated BASIC
' Functions are lowered to global variables, labels, and GOSUB

' Tutorial 8 — Arrays
' 
' dim name%(size) declares a 1-D array of size+1 elements, indexed 0..size.
' dim name%(rows, cols) declares a 2-D array; more dimensions are allowed.
' Array elements are accessed with parentheses: arr%(i%) or grid%(r%, c%).
' 
' To pass an array to a function, use the plain variable name as the
' parameter (e.g. arr%, not arr%()).  At the call site, write arr%()
' to signal that an array is being passed.

' Declare and populate
CONST n% = 6
DIM data%(n%)

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
printarray_count% = n%
DIM printarray_arr%(n%)

' copy array argument into lowered function storage: data%() -> printarray_arr%()
FOR BCC_T1% = 1 TO n%
    printarray_arr%(BCC_T1%) = data%(BCC_T1%)
NEXT BCC_T1%

GOSUB 100

' copy mutated array argument back to caller storage: printarray_arr%() -> data%()
FOR BCC_T2% = 1 TO n%
    data%(BCC_T2%) = printarray_arr%(BCC_T2%)
NEXT BCC_T2%

dummy% = printarray_result%

' Sort and show
insertionsort_count% = n%
DIM insertionsort_arr%(n%)

' copy array argument into lowered function storage: data%() -> insertionsort_arr%()
FOR BCC_T3% = 1 TO n%
    insertionsort_arr%(BCC_T3%) = data%(BCC_T3%)
NEXT BCC_T3%

GOSUB 50

' copy mutated array argument back to caller storage: insertionsort_arr%() -> data%()
FOR BCC_T4% = 1 TO n%
    data%(BCC_T4%) = insertionsort_arr%(BCC_T4%)
NEXT BCC_T4%

dummy% = insertionsort_result%
PRINT "After:  "
printarray_count% = n%
DIM printarray_arr%(n%)

' copy array argument into lowered function storage: data%() -> printarray_arr%()
FOR BCC_T5% = 1 TO n%
    printarray_arr%(BCC_T5%) = data%(BCC_T5%)
NEXT BCC_T5%

GOSUB 100

' copy mutated array argument back to caller storage: printarray_arr%() -> data%()
FOR BCC_T6% = 1 TO n%
    data%(BCC_T6%) = printarray_arr%(BCC_T6%)
NEXT BCC_T6%

dummy% = printarray_result%

' Search
target% = 22
indexof_count% = n%
indexof_target% = target%
DIM indexof_arr%(n%)

' copy array argument into lowered function storage: data%() -> indexof_arr%()
FOR BCC_T7% = 1 TO n%
    indexof_arr%(BCC_T7%) = data%(BCC_T7%)
NEXT BCC_T7%

GOSUB 80

' copy mutated array argument back to caller storage: indexof_arr%() -> data%()
FOR BCC_T8% = 1 TO n%
    data%(BCC_T8%) = indexof_arr%(BCC_T8%)
NEXT BCC_T8%

idx% = indexof_result%
IF (idx% >= 0) = 0 THEN GOTO 10
    PRINT (STR$(target%) + " found at index ") + STR$(idx%)
    GOTO 20
10 PRINT STR$(target%) + " not found"
20 REM END IF

' 2-D array — 3×3 identity matrix
DIM identity%(2, 2)
FOR r% = 0 TO 2
    FOR c% = 0 TO 2
        IF (r% = c%) = 0 THEN GOTO 30
            identity%(r%, c%) = 1
            GOTO 40
30 identity%(r%, c%) = 0
40 REM END IF
    NEXT c%
NEXT r%

PRINT "Identity matrix:"
FOR r% = 0 TO 2
    PRINT identity%(r%, 0); identity%(r%, 1); identity%(r%, 2)
NEXT r%

END

' function insertionsort%(arr%, count%)
50 FOR insertionsort_i% = 1 TO insertionsort_count% - 1
        insertionsort_key% = insertionsort_arr%(insertionsort_i%)
        insertionsort_j% = insertionsort_i% - 1
60 IF ((insertionsort_j% >= 0) AND (insertionsort_arr%(insertionsort_j%) > insertionsort_key%)) = 0 THEN GOTO 70
            insertionsort_arr%(insertionsort_j% + 1) = insertionsort_arr%(insertionsort_j%)
            insertionsort_j% = insertionsort_j% - 1
            GOTO 60
70 REM END WHILE
        insertionsort_arr%(insertionsort_j% + 1) = insertionsort_key%
    NEXT insertionsort_i%
    insertionsort_result% = 0
    RETURN
' end function insertionsort%

' function indexof%(arr%, count%, target%)
80 FOR indexof_i% = 0 TO indexof_count% - 1
        IF (indexof_arr%(indexof_i%) = indexof_target%) = 0 THEN GOTO 90
            indexof_result% = indexof_i%
            RETURN
90 REM END IF
    NEXT indexof_i%
    indexof_result% = -1
    RETURN
' end function indexof%

' function printarray%(arr%, count%)
100 printarray_line$ = "["
    FOR printarray_i% = 0 TO printarray_count% - 1
        printarray_line$ = (printarray_line$ + " ") + STR$(printarray_arr%(printarray_i%))
    NEXT printarray_i%
    PRINT printarray_line$ + " ]"
    printarray_result% = 0
    RETURN
' end function printarray%
