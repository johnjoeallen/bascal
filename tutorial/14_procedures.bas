10 ' BASCAL generated BASIC
20 ' Functions are lowered to global variables, labels, and GOSUB

30 ' Tutorial 14 — Procedures
40 ' 
50 ' A procedure is like a function but returns no value.  Declare it with
60 ' PROCEDURE ... END PROCEDURE.  The name must not carry a type suffix.
70 ' 
80 ' Variables inside a procedure are LOCAL by default: the compiler prefixes
90 ' them with the procedure name.  To access a global variable, declare it
100 ' inside the body with:  global varname
110 ' 
120 ' Use procedures for actions that produce side effects (output, file I/O,
130 ' modifying arrays) rather than for computing a value.
140 ' 
150 ' A bare RETURN exits a procedure early.  Falling through to END PROCEDURE
160 ' is also valid — an implicit RETURN is emitted.

170 ' Procedure with no parameters

180 ' Procedure that prints a labelled value

190 ' Procedure with early exit

200 ' Procedure that modifies an array in place (copy-in / copy-out)

210 ' Procedure that uses a global variable
220 globalcount% = 0

230 ' --- Drive the procedures ---

240 GOSUB 690
250 printscore_label_0$ = "Alice"
260 printscore_score_0% = 91
270 GOSUB 730
280 printscore_label_0$ = "Bob"
290 printscore_score_0% = 54
300 GOSUB 730
310 printscore_label_0$ = "Carol"
320 printscore_score_0% = 78
330 GOSUB 730
340 GOSUB 690

350 PRINT "Passes only:"
360 printifpass_name_0$ = "Alice"
370 printifpass_score_0% = 91
380 GOSUB 770
390 printifpass_name_0$ = "Bob"
400 printifpass_score_0% = 54
410 GOSUB 770
420 printifpass_name_0$ = "Carol"
430 printifpass_score_0% = 78
440 GOSUB 770

450 CONST n% = 5
460 DIM data%(n%)
470 fillrange_count_0% = n%
480 fillrange_value_0% = 99
490 DIM fillrange_arr_0%(n%)

500 ' copy array argument into lowered function storage: data%() -> fillrange_arr_0%()
510 FOR BCC_T1% = 1 TO n%
520     fillrange_arr_0%(BCC_T1%) = data%(BCC_T1%)
530 NEXT BCC_T1%

540 GOSUB 840

550 ' copy mutated array argument back to caller storage: fillrange_arr_0%() -> data%()
560 FOR BCC_T2% = 1 TO n%
570     data%(BCC_T2%) = fillrange_arr_0%(BCC_T2%)
580 NEXT BCC_T2%

590 PRINT "Filled array:"
600 FOR i% = 0 TO n% - 1
610     PRINT (("  data%(" + STR$(i%)) + ") = ") + STR$(data%(i%))
620 NEXT i%

630 GOSUB 900
640 GOSUB 900
650 GOSUB 900
660 PRINT "globalCount = " + STR$(globalcount%)

670 END

680 ' procedure printseparator()
690     PRINT "----------------------------"
700     RETURN
710 ' end procedure printseparator

720 ' procedure printscore(label$, score%)
730     PRINT (printscore_label_0$ + ": ") + STR$(printscore_score_0%)
740     RETURN
750 ' end procedure printscore

760 ' procedure printifpass(name$, score%)
770     IF (printifpass_score_0% < 60) = 0 THEN GOTO 790
780         RETURN
790     REM END IF
800     PRINT (printifpass_name_0$ + " passed with ") + STR$(printifpass_score_0%)
810     RETURN
820 ' end procedure printifpass

830 ' procedure fillrange(arr%, count%, value%)
840     FOR fillrange_i_0% = 0 TO fillrange_count_0% - 1
850         fillrange_arr_0%(fillrange_i_0%) = fillrange_value_0%
860     NEXT fillrange_i_0%
870     RETURN
880 ' end procedure fillrange

890 ' procedure increment()
900     globalcount% = globalcount% + 1
910     RETURN
920 ' end procedure increment
