10 ' BASCAL generated BASIC
20 ' Functions are lowered to global variables, labels, and GOSUB

30 ' Tutorial 8 — Arrays
40 ' 
50 ' dim name%(size) declares an array of size+1 elements indexed 0..size.
60 ' Array elements are accessed with parentheses: arr%(i%).
70 ' 
80 ' To pass an array to a function, use the plain variable name as the
90 ' parameter (e.g. arr%, not arr%()).  At the call site, write arr%()
100 ' to signal that an array is being passed.

110 ' Declare and populate
120 CONST n% = 6
130 DIM data%(n%)

140 data%(0) = 64
150 data%(1) = 25
160 data%(2) = 12
170 data%(3) = 22
180 data%(4) = 3
190 data%(5) = 11

200 ' Insertion sort — sorts data%() in place

210 ' Linear search — returns index or -1

220 ' Print the array on one line as  [ a b c ... ]

230 ' Before sort
240 PRINT "Before: "
250 printarray_count% = n%
260 DIM printarray_arr%(n%)

270 ' copy array argument into lowered function storage: data%() -> printarray_arr%()
280 FOR BCC_T1% = 1 TO n%
290     printarray_arr%(BCC_T1%) = data%(BCC_T1%)
300 NEXT BCC_T1%

310 GOSUB 1090

320 ' copy mutated array argument back to caller storage: printarray_arr%() -> data%()
330 FOR BCC_T2% = 1 TO n%
340     data%(BCC_T2%) = printarray_arr%(BCC_T2%)
350 NEXT BCC_T2%

360 dummy% = printarray_result%

370 ' Sort and show
380 insertionsort_count% = n%
390 DIM insertionsort_arr%(n%)

400 ' copy array argument into lowered function storage: data%() -> insertionsort_arr%()
410 FOR BCC_T3% = 1 TO n%
420     insertionsort_arr%(BCC_T3%) = data%(BCC_T3%)
430 NEXT BCC_T3%

440 GOSUB 850

450 ' copy mutated array argument back to caller storage: insertionsort_arr%() -> data%()
460 FOR BCC_T4% = 1 TO n%
470     data%(BCC_T4%) = insertionsort_arr%(BCC_T4%)
480 NEXT BCC_T4%

490 dummy% = insertionsort_result%
500 PRINT "After:  "
510 printarray_count% = n%
520 DIM printarray_arr%(n%)

530 ' copy array argument into lowered function storage: data%() -> printarray_arr%()
540 FOR BCC_T5% = 1 TO n%
550     printarray_arr%(BCC_T5%) = data%(BCC_T5%)
560 NEXT BCC_T5%

570 GOSUB 1090

580 ' copy mutated array argument back to caller storage: printarray_arr%() -> data%()
590 FOR BCC_T6% = 1 TO n%
600     data%(BCC_T6%) = printarray_arr%(BCC_T6%)
610 NEXT BCC_T6%

620 dummy% = printarray_result%

630 ' Search
640 target% = 22
650 indexof_count% = n%
660 indexof_target% = target%
670 DIM indexof_arr%(n%)

680 ' copy array argument into lowered function storage: data%() -> indexof_arr%()
690 FOR BCC_T7% = 1 TO n%
700     indexof_arr%(BCC_T7%) = data%(BCC_T7%)
710 NEXT BCC_T7%

720 GOSUB 990

730 ' copy mutated array argument back to caller storage: indexof_arr%() -> data%()
740 FOR BCC_T8% = 1 TO n%
750     data%(BCC_T8%) = indexof_arr%(BCC_T8%)
760 NEXT BCC_T8%

770 idx% = indexof_result%
780 IF (idx% >= 0) = 0 THEN GOTO 810
790     PRINT (STR$(target%) + " found at index ") + STR$(idx%)
800     GOTO 820
810     PRINT STR$(target%) + " not found"
820 REM END IF

830 END

840 ' function insertionsort%(arr%, count%)
850     FOR insertionsort_i% = 1 TO insertionsort_count% - 1
860         insertionsort_key% = insertionsort_arr%(insertionsort_i%)
870         insertionsort_j% = insertionsort_i% - 1
880         IF ((insertionsort_j% >= 0) AND (insertionsort_arr%(insertionsort_j%) > insertionsort_key%)) = 0 THEN GOTO 920
890             insertionsort_arr%(insertionsort_j% + 1) = insertionsort_arr%(insertionsort_j%)
900             insertionsort_j% = insertionsort_j% - 1
910             GOTO 880
920         REM END WHILE
930         insertionsort_arr%(insertionsort_j% + 1) = insertionsort_key%
940     NEXT insertionsort_i%
950     insertionsort_result% = 0
960     RETURN
970 ' end function insertionsort%

980 ' function indexof%(arr%, count%, target%)
990     FOR indexof_i% = 0 TO indexof_count% - 1
1000         IF (indexof_arr%(indexof_i%) = indexof_target%) = 0 THEN GOTO 1030
1010             indexof_result% = indexof_i%
1020             RETURN
1030         REM END IF
1040     NEXT indexof_i%
1050     indexof_result% = -1
1060     RETURN
1070 ' end function indexof%

1080 ' function printarray%(arr%, count%)
1090     printarray_line$ = "["
1100     FOR printarray_i% = 0 TO printarray_count% - 1
1110         printarray_line$ = (printarray_line$ + " ") + STR$(printarray_arr%(printarray_i%))
1120     NEXT printarray_i%
1130     PRINT printarray_line$ + " ]"
1140     printarray_result% = 0
1150     RETURN
1160 ' end function printarray%
