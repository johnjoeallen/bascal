' BASCAL generated BASIC
' Functions are transpiled to global variables, labels, and GOSUB

' Storage for array parameters, sized to fit every call site
DIM insertionsort_arr_0%(6)
DIM indexof_arr_0%(6)
DIM printarray_arr_0%(6)

' Tutorial 8 — Arrays
' 
' dim name%(size) declares a 1-D array of size+1 elements, indexed 0..size.
' dim name%(rows, cols) declares a 2-D array; more dimensions are allowed.
' Array elements are accessed with parentheses: arr%(i%) or grid%(r%, c%).
' 
' An array parameter must declare its rank with one ? per dimension:
' arr%(?) for 1-D, grid%(?, ?) for 2-D, and so on. At the call site, just
' write the plain array name -- no () and no size argument needed; the
' compiler already knows that parameter is an array from its declaration,
' and carries its size alongside it automatically. Use sizeof(arr%) inside
' the function body wherever the size is needed.
' 
' An array parameter defaults to byval: the function gets its own private
' copy, and changes never reach the caller.  Write byref to copy the
' result back out after the call -- insertionSort% below needs it, since
' its whole job is to mutate the caller's array in place.

' Declare and populate
CONST n% = 6
DIM data%(n%)
BCC_T1% = n%

data%(0) = 64
data%(1) = 25
data%(2) = 12
data%(3) = 22
data%(4) = 3
data%(5) = 11

' Insertion sort — sorts data%() in place
' arr% -- array to sort; byref because it's mutated in place

' Linear search — returns index or -1
' arr%    -- array to search; byval, since indexOf% only reads it
' target% -- value to search for

' Print the array on one line as  [ a b c ... ]
' arr% -- array to print; byval, since printArray% only reads it

' Before sort
PRINT "Before: "
printarray_arr_dim0_0% = BCC_T1%
IF printarray_arr_dim0_0% > 6 THEN PRINT "runtime error: `arr%` of `printArray%` needs "; printarray_arr_dim0_0%; " elements along axis 0, but its storage only holds 6" : STOP

' copy array argument into transpiled function storage: data%() -> printarray_arr_0%()
FOR BCC_T2% = 1 TO printarray_arr_dim0_0%
    printarray_arr_0%(BCC_T2%) = data%(BCC_T2%)
NEXT BCC_T2%

GOSUB 100
dummy% = printarray_result_0%

' Sort and show
insertionsort_arr_dim0_0% = BCC_T1%
IF insertionsort_arr_dim0_0% > 6 THEN PRINT "runtime error: `arr%` of `insertionSort%` needs "; insertionsort_arr_dim0_0%; " elements along axis 0, but its storage only holds 6" : STOP

' copy array argument into transpiled function storage: data%() -> insertionsort_arr_0%()
FOR BCC_T3% = 1 TO insertionsort_arr_dim0_0%
    insertionsort_arr_0%(BCC_T3%) = data%(BCC_T3%)
NEXT BCC_T3%

GOSUB 50

' copy mutated array argument back to caller storage: insertionsort_arr_0%() -> data%()
FOR BCC_T4% = 1 TO insertionsort_arr_dim0_0%
    data%(BCC_T4%) = insertionsort_arr_0%(BCC_T4%)
NEXT BCC_T4%

dummy% = insertionsort_result_0%
PRINT "After:  "
printarray_arr_dim0_0% = BCC_T1%
IF printarray_arr_dim0_0% > 6 THEN PRINT "runtime error: `arr%` of `printArray%` needs "; printarray_arr_dim0_0%; " elements along axis 0, but its storage only holds 6" : STOP

' copy array argument into transpiled function storage: data%() -> printarray_arr_0%()
FOR BCC_T5% = 1 TO printarray_arr_dim0_0%
    printarray_arr_0%(BCC_T5%) = data%(BCC_T5%)
NEXT BCC_T5%

GOSUB 100
dummy% = printarray_result_0%

' Search
target% = 22
indexof_target_0% = target%
indexof_arr_dim0_0% = BCC_T1%
IF indexof_arr_dim0_0% > 6 THEN PRINT "runtime error: `arr%` of `indexOf%` needs "; indexof_arr_dim0_0%; " elements along axis 0, but its storage only holds 6" : STOP

' copy array argument into transpiled function storage: data%() -> indexof_arr_0%()
FOR BCC_T6% = 1 TO indexof_arr_dim0_0%
    indexof_arr_0%(BCC_T6%) = data%(BCC_T6%)
NEXT BCC_T6%

GOSUB 80
idx% = indexof_result_0%
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

' function insertionsort%(arr%)
50 FOR insertionsort_i_0% = 1 TO insertionsort_arr_dim0_0% - 1
        insertionsort_key_0% = insertionsort_arr_0%(insertionsort_i_0%)
        insertionsort_j_0% = insertionsort_i_0% - 1
60 IF ((insertionsort_j_0% >= 0) AND (insertionsort_arr_0%(insertionsort_j_0%) > insertionsort_key_0%)) = 0 THEN GOTO 70
            insertionsort_arr_0%(insertionsort_j_0% + 1) = insertionsort_arr_0%(insertionsort_j_0%)
            insertionsort_j_0% = insertionsort_j_0% - 1
            GOTO 60
70 REM END WHILE
        insertionsort_arr_0%(insertionsort_j_0% + 1) = insertionsort_key_0%
    NEXT insertionsort_i_0%
    insertionsort_result_0% = 0
    RETURN
' end function insertionsort%

' function indexof%(arr%, target%)
80 FOR indexof_i_0% = 0 TO indexof_arr_dim0_0% - 1
        IF (indexof_arr_0%(indexof_i_0%) = indexof_target_0%) = 0 THEN GOTO 90
            indexof_result_0% = indexof_i_0%
            RETURN
90 REM END IF
    NEXT indexof_i_0%
    indexof_result_0% = -1
    RETURN
' end function indexof%

' function printarray%(arr%)
100 printarray_line_0$ = "["
    FOR printarray_i_0% = 0 TO printarray_arr_dim0_0% - 1
        printarray_line_0$ = (printarray_line_0$ + " ") + STR$(printarray_arr_0%(printarray_i_0%))
    NEXT printarray_i_0%
    PRINT printarray_line_0$ + " ]"
    printarray_result_0% = 0
    RETURN
' end function printarray%
