10 ' BASCAL generated BASIC -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Storage for array parameters, sized to fit every call site
40 DIM fillrangeArr0%(5)

50 ' Tutorial — Procedures
60 ' 
70 ' A procedure is like a function but returns no value.  Declare it with
80 ' PROCEDURE ... END PROCEDURE.  The name must not carry a type suffix.
90 ' 
100 ' Variables inside a procedure are LOCAL by default: the compiler prefixes
110 ' them with the procedure name.  To access a global variable, declare it
120 ' inside the body with:  global varname
130 ' 
140 ' Use procedures for actions that produce side effects (output, file I/O,
150 ' modifying arrays) rather than for computing a value.
160 ' 
170 ' A bare RETURN exits a procedure early.  Falling through to END PROCEDURE
180 ' is also valid — an implicit RETURN is emitted.
190 ' Procedure with no parameters
200 ' Procedure that prints a labelled value
210 ' label$ -- text shown before the score
220 ' score% -- value to print
230 ' Procedure with early exit
240 ' name$  -- person's name
250 ' score% -- score to test against the passing threshold
260 ' Procedure that modifies an array in place -- byref copies the result
270 ' back to the caller; the default byval would fill a private copy only.
280 ' arr%   -- array to fill; byref because it's mutated in place
290 ' value% -- value written into every element
300 ' Procedure that uses a global variable
310 globalcount% = 0
320 ' --- Drive the procedures ---
330 GOSUB 830
340 printscoreLabel0$ = "Alice"
350 printscoreScore0% = 91
360 GOSUB 870
370 printscoreLabel0$ = "Bob"
380 printscoreScore0% = 54
390 GOSUB 870
400 printscoreLabel0$ = "Carol"
410 printscoreScore0% = 78
420 GOSUB 870
430 GOSUB 830
440 PRINT "Passes only:"
450 printifpassName0$ = "Alice"
460 printifpassScore0% = 91
470 GOSUB 910
480 ' printed
490 printifpassName0$ = "Bob"
500 printifpassScore0% = 54
510 GOSUB 910
520 ' skipped (score < 60)
530 printifpassName0$ = "Carol"
540 printifpassScore0% = 78
550 GOSUB 910
560 ' printed
570 CONSTN% = 5
580 DIM data%(CONSTN%)
590 BCCT1% = CONSTN%
600 fillrangeArrDim00% = BCCT1%
610 IF fillrangeArrDim00% > 5 THEN PRINT "runtime error: `arr%` of `fillRange` needs "; fillrangeArrDim00%; " elements along axis 0, but its storage only holds 5" : STOP

620 ' copy array argument into transpiled procedure storage: data%() -> fillrangeArr0%()
630 FOR BCCT2% = 0 TO fillrangeArrDim00%
640     fillrangeArr0%(BCCT2%) = data%(BCCT2%)
650 NEXT BCCT2%

660 fillrangeValue0% = 99
670 GOSUB 990

680 ' copy mutated array argument back to caller storage: fillrangeArr0%() -> data%()
690 FOR BCCT3% = 0 TO fillrangeArrDim00%
700     data%(BCCT3%) = fillrangeArr0%(BCCT3%)
710 NEXT BCCT3%

720 PRINT "Filled array:"
730 FOR i% = 0 TO CONSTN% - 1
740     PRINT (("  data%(" + STR$(i%)) + ") = ") + STR$(data%(i%))
750 NEXT i%
760 GOSUB 1050
770 GOSUB 1050
780 GOSUB 1050
790 PRINT "globalCount = " + STR$(globalcount%)
800 ' 3
810 END

820 ' procedure printseparator()
830     PRINT "----------------------------"
840     RETURN
850 ' end procedure printseparator

860 ' procedure printscore(label$, score%)
870     PRINT (printscoreLabel0$ + ": ") + STR$(printscoreScore0%)
880     RETURN
890 ' end procedure printscore

900 ' procedure printifpass(name$, score%)
910     IF (printifpassScore0% < 60) = 0 THEN GOTO 940
920         RETURN
930         ' early exit — nothing printed for failing scores
940     REM END IF
950     PRINT (printifpassName0$ + " passed with ") + STR$(printifpassScore0%)
960     RETURN
970 ' end procedure printifpass

980 ' procedure fillrange(arr%, value%)
990     FOR fillrangeI0% = 0 TO (fillrangeArrDim00% + 1) - 1
1000         fillrangeArr0%(fillrangeI0%) = fillrangeValue0%
1010     NEXT fillrangeI0%
1020     RETURN
1030 ' end procedure fillrange

1040 ' procedure increment()
1050     globalcount% = globalcount% + 1
1060     RETURN
1070 ' end procedure increment
