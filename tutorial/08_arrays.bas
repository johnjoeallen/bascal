10 ' BASCAL generated BASIC -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
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
460 FOR BCCT2% = 0 TO printarrayArrDim00%
470     printarrayArr0%(BCCT2%) = data%(BCCT2%)
480 NEXT BCCT2%

490 GOSUB 1310
500 dummy% = printarrayResult0%

510 ' Sort and show
520 insertionsortArrDim00% = BCCT1%
530 IF insertionsortArrDim00% > 6 THEN PRINT "runtime error: `arr%` of `insertionSort%` needs "; insertionsortArrDim00%; " elements along axis 0, but its storage only holds 6" : STOP

540 ' copy array argument into transpiled function storage: data%() -> insertionsortArr0%()
550 FOR BCCT3% = 0 TO insertionsortArrDim00%
560     insertionsortArr0%(BCCT3%) = data%(BCCT3%)
570 NEXT BCCT3%

580 GOSUB 1060

590 ' copy mutated array argument back to caller storage: insertionsortArr0%() -> data%()
600 FOR BCCT4% = 0 TO insertionsortArrDim00%
610     data%(BCCT4%) = insertionsortArr0%(BCCT4%)
620 NEXT BCCT4%

630 dummy% = insertionsortResult0%
640 PRINT "After:  "
650 printarrayArrDim00% = BCCT1%
660 IF printarrayArrDim00% > 6 THEN PRINT "runtime error: `arr%` of `printArray%` needs "; printarrayArrDim00%; " elements along axis 0, but its storage only holds 6" : STOP

670 ' copy array argument into transpiled function storage: data%() -> printarrayArr0%()
680 FOR BCCT5% = 0 TO printarrayArrDim00%
690     printarrayArr0%(BCCT5%) = data%(BCCT5%)
700 NEXT BCCT5%

710 GOSUB 1310
720 dummy% = printarrayResult0%

730 ' Search
740 target% = 22
750 indexofTarget0% = target%
760 indexofArrDim00% = BCCT1%
770 IF indexofArrDim00% > 6 THEN PRINT "runtime error: `arr%` of `indexOf%` needs "; indexofArrDim00%; " elements along axis 0, but its storage only holds 6" : STOP

780 ' copy array argument into transpiled function storage: data%() -> indexofArr0%()
790 FOR BCCT6% = 0 TO indexofArrDim00%
800     indexofArr0%(BCCT6%) = data%(BCCT6%)
810 NEXT BCCT6%

820 GOSUB 1210
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
1060     FOR insertionsortI0% = 1 TO (insertionsortArrDim00% + 1) - 1
1070         insertionsortKey0% = insertionsortArr0%(insertionsortI0%)
1080         insertionsortJ0% = insertionsortI0% - 1
1090         IF (insertionsortJ0% >= 0) = 0 THEN GOTO 1140
1100         IF (insertionsortArr0%(insertionsortJ0%) > insertionsortKey0%) = 0 THEN GOTO 1140
1110             insertionsortArr0%(insertionsortJ0% + 1) = insertionsortArr0%(insertionsortJ0%)
1120             insertionsortJ0% = insertionsortJ0% - 1
1130             GOTO 1090
1140         REM END WHILE
1150         insertionsortArr0%(insertionsortJ0% + 1) = insertionsortKey0%
1160     NEXT insertionsortI0%
1170     insertionsortResult0% = 0
1180     RETURN
1190 ' end function insertionsort%

1200 ' function indexof%(arr%, target%)
1210     FOR indexofI0% = 0 TO (indexofArrDim00% + 1) - 1
1220         IF (indexofArr0%(indexofI0%) = indexofTarget0%) = 0 THEN GOTO 1250
1230             indexofResult0% = indexofI0%
1240             RETURN
1250         REM END IF
1260     NEXT indexofI0%
1270     indexofResult0% = -1
1280     RETURN
1290 ' end function indexof%

1300 ' function printarray%(arr%)
1310     printarrayLine0$ = "["
1320     FOR printarrayI0% = 0 TO (printarrayArrDim00% + 1) - 1
1330         printarrayLine0$ = (printarrayLine0$ + " ") + STR$(printarrayArr0%(printarrayI0%))
1340     NEXT printarrayI0%
1350     PRINT printarrayLine0$ + " ]"
1360     printarrayResult0% = 0
1370     RETURN
1380 ' end function printarray%
