10 ' BASCAL generated BASIC
20 ' Functions are lowered to global variables, labels, and GOSUB

30 ' Tutorial 8 — Arrays
40 ' 
50 ' dim name%(size) declares a 1-D array of size+1 elements, indexed 0..size.
60 ' dim name%(rows, cols) declares a 2-D array; more dimensions are allowed.
70 ' Array elements are accessed with parentheses: arr%(i%) or grid%(r%, c%).
80 ' 
90 ' To pass an array to a function, use the plain variable name as the
100 ' parameter (e.g. arr%, not arr%()).  At the call site, write arr%()
110 ' to signal that an array is being passed.

120 ' Declare and populate
130 CONST n% = 6
140 DIM data%(n%)

150 data%(0) = 64
160 data%(1) = 25
170 data%(2) = 12
180 data%(3) = 22
190 data%(4) = 3
200 data%(5) = 11

210 ' Insertion sort — sorts data%() in place

220 ' Linear search — returns index or -1

230 ' Print the array on one line as  [ a b c ... ]

240 ' Before sort
250 PRINT "Before: "
260 printarray_count_0% = n%
270 DIM printarray_arr_0%(n%)

280 ' copy array argument into lowered function storage: data%() -> printarray_arr_0%()
290 FOR BCC_T1% = 1 TO n%
300     printarray_arr_0%(BCC_T1%) = data%(BCC_T1%)
310 NEXT BCC_T1%

320 GOSUB 1250

330 ' copy mutated array argument back to caller storage: printarray_arr_0%() -> data%()
340 FOR BCC_T2% = 1 TO n%
350     data%(BCC_T2%) = printarray_arr_0%(BCC_T2%)
360 NEXT BCC_T2%

370 dummy% = printarray_result_0%

380 ' Sort and show
390 insertionsort_count_0% = n%
400 DIM insertionsort_arr_0%(n%)

410 ' copy array argument into lowered function storage: data%() -> insertionsort_arr_0%()
420 FOR BCC_T3% = 1 TO n%
430     insertionsort_arr_0%(BCC_T3%) = data%(BCC_T3%)
440 NEXT BCC_T3%

450 GOSUB 1010

460 ' copy mutated array argument back to caller storage: insertionsort_arr_0%() -> data%()
470 FOR BCC_T4% = 1 TO n%
480     data%(BCC_T4%) = insertionsort_arr_0%(BCC_T4%)
490 NEXT BCC_T4%

500 dummy% = insertionsort_result_0%
510 PRINT "After:  "
520 printarray_count_0% = n%
530 DIM printarray_arr_0%(n%)

540 ' copy array argument into lowered function storage: data%() -> printarray_arr_0%()
550 FOR BCC_T5% = 1 TO n%
560     printarray_arr_0%(BCC_T5%) = data%(BCC_T5%)
570 NEXT BCC_T5%

580 GOSUB 1250

590 ' copy mutated array argument back to caller storage: printarray_arr_0%() -> data%()
600 FOR BCC_T6% = 1 TO n%
610     data%(BCC_T6%) = printarray_arr_0%(BCC_T6%)
620 NEXT BCC_T6%

630 dummy% = printarray_result_0%

640 ' Search
650 target% = 22
660 indexof_count_0% = n%
670 indexof_target_0% = target%
680 DIM indexof_arr_0%(n%)

690 ' copy array argument into lowered function storage: data%() -> indexof_arr_0%()
700 FOR BCC_T7% = 1 TO n%
710     indexof_arr_0%(BCC_T7%) = data%(BCC_T7%)
720 NEXT BCC_T7%

730 GOSUB 1150

740 ' copy mutated array argument back to caller storage: indexof_arr_0%() -> data%()
750 FOR BCC_T8% = 1 TO n%
760     data%(BCC_T8%) = indexof_arr_0%(BCC_T8%)
770 NEXT BCC_T8%

780 idx% = indexof_result_0%
790 IF (idx% >= 0) = 0 THEN GOTO 820
800     PRINT (STR$(target%) + " found at index ") + STR$(idx%)
810     GOTO 830
820     PRINT STR$(target%) + " not found"
830 REM END IF

840 ' 2-D array — 3×3 identity matrix
850 DIM identity%(2, 2)
860 FOR r% = 0 TO 2
870     FOR c% = 0 TO 2
880         IF (r% = c%) = 0 THEN GOTO 910
890             identity%(r%, c%) = 1
900             GOTO 920
910             identity%(r%, c%) = 0
920         REM END IF
930     NEXT c%
940 NEXT r%

950 PRINT "Identity matrix:"
960 FOR r% = 0 TO 2
970     PRINT identity%(r%, 0); identity%(r%, 1); identity%(r%, 2)
980 NEXT r%

990 END

1000 ' function insertionsort%(arr%, count%)
1010     FOR insertionsort_i_0% = 1 TO insertionsort_count_0% - 1
1020         insertionsort_key_0% = insertionsort_arr_0%(insertionsort_i_0%)
1030         insertionsort_j_0% = insertionsort_i_0% - 1
1040         IF ((insertionsort_j_0% >= 0) AND (insertionsort_arr_0%(insertionsort_j_0%) > insertionsort_key_0%)) = 0 THEN GOTO 1080
1050             insertionsort_arr_0%(insertionsort_j_0% + 1) = insertionsort_arr_0%(insertionsort_j_0%)
1060             insertionsort_j_0% = insertionsort_j_0% - 1
1070             GOTO 1040
1080         REM END WHILE
1090         insertionsort_arr_0%(insertionsort_j_0% + 1) = insertionsort_key_0%
1100     NEXT insertionsort_i_0%
1110     insertionsort_result_0% = 0
1120     RETURN
1130 ' end function insertionsort%

1140 ' function indexof%(arr%, count%, target%)
1150     FOR indexof_i_0% = 0 TO indexof_count_0% - 1
1160         IF (indexof_arr_0%(indexof_i_0%) = indexof_target_0%) = 0 THEN GOTO 1190
1170             indexof_result_0% = indexof_i_0%
1180             RETURN
1190         REM END IF
1200     NEXT indexof_i_0%
1210     indexof_result_0% = -1
1220     RETURN
1230 ' end function indexof%

1240 ' function printarray%(arr%, count%)
1250     printarray_line_0$ = "["
1260     FOR printarray_i_0% = 0 TO printarray_count_0% - 1
1270         printarray_line_0$ = (printarray_line_0$ + " ") + STR$(printarray_arr_0%(printarray_i_0%))
1280     NEXT printarray_i_0%
1290     PRINT printarray_line_0$ + " ]"
1300     printarray_result_0% = 0
1310     RETURN
1320 ' end function printarray%
