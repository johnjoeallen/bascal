' BASCAL generated BASIC
' Functions are transpiled to global variables, labels, and GOSUB

' Storage for array parameters, sized to fit every call site
DIM fillrange_arr_0%(5)

' Tutorial 14 — Procedures
' 
' A procedure is like a function but returns no value.  Declare it with
' PROCEDURE ... END PROCEDURE.  The name must not carry a type suffix.
' 
' Variables inside a procedure are LOCAL by default: the compiler prefixes
' them with the procedure name.  To access a global variable, declare it
' inside the body with:  global varname
' 
' Use procedures for actions that produce side effects (output, file I/O,
' modifying arrays) rather than for computing a value.
' 
' A bare RETURN exits a procedure early.  Falling through to END PROCEDURE
' is also valid — an implicit RETURN is emitted.

' Procedure with no parameters

' Procedure that prints a labelled value
' label$ -- text shown before the score
' score% -- value to print

' Procedure with early exit
' name$  -- person's name
' score% -- score to test against the passing threshold

' Procedure that modifies an array in place -- byref copies the result
' back to the caller; the default byval would fill a private copy only.
' arr%   -- array to fill; byref because it's mutated in place
' value% -- value written into every element

' Procedure that uses a global variable
globalcount% = 0

' --- Drive the procedures ---

GOSUB 10
printscore_label_0$ = "Alice"
printscore_score_0% = 91
GOSUB 20
printscore_label_0$ = "Bob"
printscore_score_0% = 54
GOSUB 20
printscore_label_0$ = "Carol"
printscore_score_0% = 78
GOSUB 20
GOSUB 10

PRINT "Passes only:"
printifpass_name_0$ = "Alice"
printifpass_score_0% = 91
GOSUB 30
printifpass_name_0$ = "Bob"
printifpass_score_0% = 54
GOSUB 30
printifpass_name_0$ = "Carol"
printifpass_score_0% = 78
GOSUB 30

CONST n% = 5
DIM data%(n%)
BCC_T1% = n%
fillrange_value_0% = 99
fillrange_arr_dim0_0% = BCC_T1%
IF fillrange_arr_dim0_0% > 5 THEN PRINT "runtime error: `arr%` of `fillRange` needs "; fillrange_arr_dim0_0%; " elements along axis 0, but its storage only holds 5" : STOP

' copy array argument into transpiled function storage: data%() -> fillrange_arr_0%()
FOR BCC_T2% = 1 TO fillrange_arr_dim0_0%
    fillrange_arr_0%(BCC_T2%) = data%(BCC_T2%)
NEXT BCC_T2%

GOSUB 50

' copy mutated array argument back to caller storage: fillrange_arr_0%() -> data%()
FOR BCC_T3% = 1 TO fillrange_arr_dim0_0%
    data%(BCC_T3%) = fillrange_arr_0%(BCC_T3%)
NEXT BCC_T3%

PRINT "Filled array:"
FOR i% = 0 TO n% - 1
    PRINT (("  data%(" + STR$(i%)) + ") = ") + STR$(data%(i%))
NEXT i%

GOSUB 60
GOSUB 60
GOSUB 60
PRINT "globalCount = " + STR$(globalcount%)

END

' procedure printseparator()
10 PRINT "----------------------------"
    RETURN
' end procedure printseparator

' procedure printscore(label$, score%)
20 PRINT (printscore_label_0$ + ": ") + STR$(printscore_score_0%)
    RETURN
' end procedure printscore

' procedure printifpass(name$, score%)
30 IF (printifpass_score_0% < 60) = 0 THEN GOTO 40
        RETURN
40 REM END IF
    PRINT (printifpass_name_0$ + " passed with ") + STR$(printifpass_score_0%)
    RETURN
' end procedure printifpass

' procedure fillrange(arr%, value%)
50 FOR fillrange_i_0% = 0 TO fillrange_arr_dim0_0% - 1
        fillrange_arr_0%(fillrange_i_0%) = fillrange_value_0%
    NEXT fillrange_i_0%
    RETURN
' end procedure fillrange

' procedure increment()
60 globalcount% = globalcount% + 1
    RETURN
' end procedure increment
