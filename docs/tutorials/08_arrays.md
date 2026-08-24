[Home](../) / [Tutorials](./) / Arrays

<div class="prose" markdown="1">

`dim name%(size)` declares a 1-D array of `size + 1` elements, indexed `0..size`; `dim name%(rows, cols)` declares a 2-D array, and more dimensions are allowed. An array parameter must declare its rank with one `?` per dimension — `arr%(?)` for 1-D, `grid%(?, ?)` for 2-D — and at the call site you just write the plain array name, no `()` and no size argument needed: the transpiler already knows the parameter is an array from its declaration, and carries its size alongside it automatically. Use `sizeof(arr%)` inside the function body wherever the size is needed. This isn't a reference — BASCAL copies elements in before the call, and copies them back out after only if the parameter is declared `byref`; unmarked (`byval`) is the default and leaves the caller's array untouched.

</div>

<div class="snippet" markdown="1">

### Insertion sort, in place

    function insertionSort%(byref arr%(?))
        for i% = 1 to sizeof(arr%) - 1
            key% = arr%(i%)
            j%   = i% - 1
            while j% >= 0 and arr%(j%) > key%
                arr%(j% + 1) = arr%(j%)
                j% = j% - 1
            end while
            arr%(j% + 1) = key%
        end for
        return 0
    end function

</div>

<div class="snippet" markdown="1">

### Calling it, and a 2-D array

    dim data%(N%)
    ' ... populate data%(0..N%-1) ...
    dummy% = insertionSort%(data%)   ' arr% is byref, so data% comes back sorted

    dim identity%(2, 2)
    identity%(1, 1) = 1

</div>



[← Functions](07_functions.md)  ·  [Data, Read, Restore, Swap →](09_data.md)


<!-- BEGIN generated tutorial source -->

### `tutorial/08_arrays.bcl`

```bascal

// Tutorial — Arrays
//
// dim name%(size) declares a 1-D array of size+1 elements, indexed 0..size.
// dim name%(rows, cols) declares a 2-D array; more dimensions are allowed.
// Array elements are accessed with parentheses: arr%(i%) or grid%(r%, c%).
//
// An array parameter must declare its rank with one ? per dimension:
// arr%(?) for 1-D, grid%(?, ?) for 2-D, and so on. At the call site, just
// write the plain array name -- no () and no size argument needed; the
// compiler already knows that parameter is an array from its declaration,
// and carries its size alongside it automatically. Use sizeof(arr%) inside
// the function body wherever the size is needed.
//
// An array parameter defaults to byval: the function gets its own private
// copy, and changes never reach the caller.  Write byref to copy the
// result back out after the call -- insertionSort% below needs it, since
// its whole job is to mutate the caller's array in place.
program arrays

/* Declare and populate */
const N% = 6
dim data%(N%)

data%(0) = 64
data%(1) = 25
data%(2) = 12
data%(3) = 22
data%(4) =  3
data%(5) = 11

/* Insertion sort — sorts data%() in place */
// arr% -- array to sort; byref because it's mutated in place
function insertionSort%(byref arr%(?))
    for i% = 1 to sizeof(arr%) - 1
        key% = arr%(i%)
        j%   = i% - 1
        while j% >= 0 and arr%(j%) > key%
            arr%(j% + 1) = arr%(j%)
            j% = j% - 1
        end while
        arr%(j% + 1) = key%
    end for
    return 0
end function

/* Linear search — returns index or -1 */
// arr%    -- array to search; byval, since indexOf% only reads it
// target% -- value to search for
function indexOf%(arr%(?), target%)
    for i% = 0 to sizeof(arr%) - 1
        if arr%(i%) = target% then
            return i%
        end if
    end for
    return -1
end function

/* Print the array on one line as  [ a b c ... ] */
// arr% -- array to print; byval, since printArray% only reads it
function printArray%(arr%(?))
    line$ = "["
    for i% = 0 to sizeof(arr%) - 1
        line$ = line$ + " " + str$(arr%(i%))
    end for
    print line$ + " ]"
    return 0
end function

/* Before sort */
print "Before: "
dummy% = printArray%(data%)

/* Sort and show */
dummy% = insertionSort%(data%)
print "After:  "
dummy% = printArray%(data%)

/* Search */
target% = 22
idx% = indexOf%(data%, target%)
if idx% >= 0 then
    print str$(target%) + " found at index " + str$(idx%)
else
    print str$(target%) + " not found"
end if

/* 2-D array — 3×3 identity matrix */
dim identity%(2, 2)
for r% = 0 to 2
    for c% = 0 to 2
        if r% = c% then
            identity%(r%, c%) = 1
        else
            identity%(r%, c%) = 0
        end if
    end for
end for

print "Identity matrix:"
for r% = 0 to 2
    print identity%(r%, 0); identity%(r%, 1); identity%(r%, 2)
end for

end

```

### `tutorial/08_arrays.bas`

```basic

10 ' BASCAL generated BASIC
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Storage for array parameters, sized to fit every call site
40 DIM insertionsortArr0%(6)
50 DIM indexofArr0%(6)
60 DIM printarrayArr0%(6)

70 ' Tutorial — Arrays
80 '
90 ' dim name%(size) declares a 1-D array of size+1 elements, indexed 0..size.
100 ' dim name%(rows, cols) declares a 2-D array; more dimensions are allowed.
110 ' Array elements are accessed with parentheses: arr%(i%) or grid%(r%, c%).
120 '
130 ' An array parameter must declare its rank with one ? per dimension:
140 ' arr%(?) for 1-D, grid%(?, ?) for 2-D, and so on. At the call site, just
150 ' write the plain array name -- no () and no size argument needed; the
160 ' compiler already knows that parameter is an array from its declaration,
170 ' and carries its size alongside it automatically. Use sizeof(arr%) inside
180 ' the function body wherever the size is needed.
190 '
200 ' An array parameter defaults to byval: the function gets its own private
210 ' copy, and changes never reach the caller.  Write byref to copy the
220 ' result back out after the call -- insertionSort% below needs it, since
230 ' its whole job is to mutate the caller's array in place.

240 ' Declare and populate
250 n% = 6
260 DIM data%(n%)
270 BCCT1% = n%

280 data%(0) = 64
290 data%(1) = 25
300 data%(2) = 12
310 data%(3) = 22
320 data%(4) = 3
330 data%(5) = 11

340 ' Insertion sort — sorts data%() in place
350 ' arr% -- array to sort; byref because it's mutated in place

360 ' Linear search — returns index or -1
370 ' arr%    -- array to search; byval, since indexOf% only reads it
380 ' target% -- value to search for

390 ' Print the array on one line as  [ a b c ... ]
400 ' arr% -- array to print; byval, since printArray% only reads it

410 ' Before sort
420 PRINT "Before: "
430 printarrayArrDim00% = BCCT1%
440 IF printarrayArrDim00% > 6 THEN PRINT "runtime error: `arr%` of `printArray%` needs "; printarrayArrDim00%; " elements along axis 0, but its storage only holds 6" : STOP

450 ' copy array argument into transpiled function storage: data%() -> printarrayArr0%()
460 FOR BCCT2% = 1 TO printarrayArrDim00%
470     printarrayArr0%(BCCT2%) = data%(BCCT2%)
480 NEXT BCCT2%

490 GOSUB 1300
500 dummy% = printarrayResult0%

510 ' Sort and show
520 insertionsortArrDim00% = BCCT1%
530 IF insertionsortArrDim00% > 6 THEN PRINT "runtime error: `arr%` of `insertionSort%` needs "; insertionsortArrDim00%; " elements along axis 0, but its storage only holds 6" : STOP

540 ' copy array argument into transpiled function storage: data%() -> insertionsortArr0%()
550 FOR BCCT3% = 1 TO insertionsortArrDim00%
560     insertionsortArr0%(BCCT3%) = data%(BCCT3%)
570 NEXT BCCT3%

580 GOSUB 1060

590 ' copy mutated array argument back to caller storage: insertionsortArr0%() -> data%()
600 FOR BCCT4% = 1 TO insertionsortArrDim00%
610     data%(BCCT4%) = insertionsortArr0%(BCCT4%)
620 NEXT BCCT4%

630 dummy% = insertionsortResult0%
640 PRINT "After:  "
650 printarrayArrDim00% = BCCT1%
660 IF printarrayArrDim00% > 6 THEN PRINT "runtime error: `arr%` of `printArray%` needs "; printarrayArrDim00%; " elements along axis 0, but its storage only holds 6" : STOP

670 ' copy array argument into transpiled function storage: data%() -> printarrayArr0%()
680 FOR BCCT5% = 1 TO printarrayArrDim00%
690     printarrayArr0%(BCCT5%) = data%(BCCT5%)
700 NEXT BCCT5%

710 GOSUB 1300
720 dummy% = printarrayResult0%

730 ' Search
740 target% = 22
750 indexofTarget0% = target%
760 indexofArrDim00% = BCCT1%
770 IF indexofArrDim00% > 6 THEN PRINT "runtime error: `arr%` of `indexOf%` needs "; indexofArrDim00%; " elements along axis 0, but its storage only holds 6" : STOP

780 ' copy array argument into transpiled function storage: data%() -> indexofArr0%()
790 FOR BCCT6% = 1 TO indexofArrDim00%
800     indexofArr0%(BCCT6%) = data%(BCCT6%)
810 NEXT BCCT6%

820 GOSUB 1200
830 idx% = indexofResult0%
840 IF (idx% >= 0) = 0 THEN GOTO 870
850     PRINT (STR$(target%) + " found at index ") + STR$(idx%)
860     GOTO 880
870     PRINT STR$(target%) + " not found"
880 REM END IF

890 ' 2-D array — 3×3 identity matrix
900 DIM identity%(2, 2)
910 FOR r% = 0 TO 2
920     FOR c% = 0 TO 2
930         IF (r% = c%) = 0 THEN GOTO 960
940             identity%(r%, c%) = 1
950             GOTO 970
960             identity%(r%, c%) = 0
970         REM END IF
980     NEXT c%
990 NEXT r%

1000 PRINT "Identity matrix:"
1010 FOR r% = 0 TO 2
1020     PRINT identity%(r%, 0); identity%(r%, 1); identity%(r%, 2)
1030 NEXT r%

1040 END

1050 ' function insertionsort%(arr%)
1060     FOR insertionsortI0% = 1 TO insertionsortArrDim00% - 1
1070         insertionsortKey0% = insertionsortArr0%(insertionsortI0%)
1080         insertionsortJ0% = insertionsortI0% - 1
1090         IF ((insertionsortJ0% >= 0) AND (insertionsortArr0%(insertionsortJ0%) > insertionsortKey0%)) = 0 THEN GOTO 1130
1100             insertionsortArr0%(insertionsortJ0% + 1) = insertionsortArr0%(insertionsortJ0%)
1110             insertionsortJ0% = insertionsortJ0% - 1
1120             GOTO 1090
1130         REM END WHILE
1140         insertionsortArr0%(insertionsortJ0% + 1) = insertionsortKey0%
1150     NEXT insertionsortI0%
1160     insertionsortResult0% = 0
1170     RETURN
1180 ' end function insertionsort%

1190 ' function indexof%(arr%, target%)
1200     FOR indexofI0% = 0 TO indexofArrDim00% - 1
1210         IF (indexofArr0%(indexofI0%) = indexofTarget0%) = 0 THEN GOTO 1240
1220             indexofResult0% = indexofI0%
1230             RETURN
1240         REM END IF
1250     NEXT indexofI0%
1260     indexofResult0% = -1
1270     RETURN
1280 ' end function indexof%

1290 ' function printarray%(arr%)
1300     printarrayLine0$ = "["
1310     FOR printarrayI0% = 0 TO printarrayArrDim00% - 1
1320         printarrayLine0$ = (printarrayLine0$ + " ") + STR$(printarrayArr0%(printarrayI0%))
1330     NEXT printarrayI0%
1340     PRINT printarrayLine0$ + " ]"
1350     printarrayResult0% = 0
1360     RETURN
1370 ' end function printarray%

```

<!-- END generated tutorial source -->
