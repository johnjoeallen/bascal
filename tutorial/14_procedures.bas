10 ' BASCAL generated BASIC
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

330 GOSUB 790
340 printscoreLabel0$ = "Alice"
350 printscoreScore0% = 91
360 GOSUB 830
370 printscoreLabel0$ = "Bob"
380 printscoreScore0% = 54
390 GOSUB 830
400 printscoreLabel0$ = "Carol"
410 printscoreScore0% = 78
420 GOSUB 830
430 GOSUB 790

440 PRINT "Passes only:"
450 printifpassName0$ = "Alice"
460 printifpassScore0% = 91
470 GOSUB 870
480 printifpassName0$ = "Bob"
490 printifpassScore0% = 54
500 GOSUB 870
510 printifpassName0$ = "Carol"
520 printifpassScore0% = 78
530 GOSUB 870

540 n% = 5
550 DIM data%(n%)
560 BCCT1% = n%
570 fillrangeValue0% = 99
580 fillrangeArrDim00% = BCCT1%
590 IF fillrangeArrDim00% > 5 THEN PRINT "runtime error: `arr%` of `fillRange` needs "; fillrangeArrDim00%; " elements along axis 0, but its storage only holds 5" : STOP

600 ' copy array argument into transpiled function storage: data%() -> fillrangeArr0%()
610 FOR BCCT2% = 1 TO fillrangeArrDim00%
620     fillrangeArr0%(BCCT2%) = data%(BCCT2%)
630 NEXT BCCT2%

640 GOSUB 940

650 ' copy mutated array argument back to caller storage: fillrangeArr0%() -> data%()
660 FOR BCCT3% = 1 TO fillrangeArrDim00%
670     data%(BCCT3%) = fillrangeArr0%(BCCT3%)
680 NEXT BCCT3%

690 PRINT "Filled array:"
700 FOR i% = 0 TO n% - 1
710     PRINT (("  data%(" + STR$(i%)) + ") = ") + STR$(data%(i%))
720 NEXT i%

730 GOSUB 1000
740 GOSUB 1000
750 GOSUB 1000
760 PRINT "globalCount = " + STR$(globalcount%)

770 END

780 ' procedure printseparator()
790     PRINT "----------------------------"
800     RETURN
810 ' end procedure printseparator

820 ' procedure printscore(label$, score%)
830     PRINT (printscoreLabel0$ + ": ") + STR$(printscoreScore0%)
840     RETURN
850 ' end procedure printscore

860 ' procedure printifpass(name$, score%)
870     IF (printifpassScore0% < 60) = 0 THEN GOTO 890
880         RETURN
890     REM END IF
900     PRINT (printifpassName0$ + " passed with ") + STR$(printifpassScore0%)
910     RETURN
920 ' end procedure printifpass

930 ' procedure fillrange(arr%, value%)
940     FOR fillrangeI0% = 0 TO fillrangeArrDim00% - 1
950         fillrangeArr0%(fillrangeI0%) = fillrangeValue0%
960     NEXT fillrangeI0%
970     RETURN
980 ' end procedure fillrange

990 ' procedure increment()
1000     globalcount% = globalcount% + 1
1010     RETURN
1020 ' end procedure increment
