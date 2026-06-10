' BASCAL generated BASIC
' Functions are lowered to global variables, labels, and GOSUB

' Tutorial 7 — Functions
' 
' A BASCAL function is declared with FUNCTION ... END FUNCTION.
' The function name carries the return type suffix.  Parameters
' also carry type suffixes.  Every function must reach a RETURN.
' 
' The compiler lowers each call to:
' 1. Assign arguments to global variables  fname_paramname
' 2. GOSUB to the function's generated label
' 3. Copy  fname_result  into the destination variable
' 
' Functions cannot call themselves recursively (parameters would be
' overwritten).  Use an explicit stack array for recursive algorithms.

' Integer arithmetic functions

' String functions

' --- Exercise the functions ---

max_a% = 4
max_b% = 9
GOSUB 10
PRINT "max(4, 9) = " + STR$(max_result%)
min_a% = 4
min_b% = 9
GOSUB 40
PRINT "min(4, 9) = " + STR$(min_result%)
clamp_value% = 15
clamp_lo% = 1
clamp_hi% = 10
GOSUB 70
PRINT "clamp(15,1,10) = " + STR$(clamp_result%)
clamp_value% = -3
clamp_lo% = 1
clamp_hi% = 10
GOSUB 70
PRINT "clamp(-3,1,10) = " + STR$(clamp_result%)
clamp_value% = 7
clamp_lo% = 1
clamp_hi% = 10
GOSUB 70
PRINT "clamp(7,1,10)  = " + STR$(clamp_result%)

repeat_text$ = "ab"
repeat_n% = 4
GOSUB 80
PRINT repeat_result$
titlecase_word$ = "bASCAL"
GOSUB 90
PRINT titlecase_result$

' Functions chained in expressions
max_a% = 0
max_b% = -5
GOSUB 10
min_a% = max_result%
min_b% = 100
GOSUB 40
lo% = min_result%
PRINT "lo = " + STR$(lo%)

END

' function max%(a%, b%)
10 IF (max_a% > max_b%) = 0 THEN GOTO 20
        max_result% = max_a%
        RETURN
        GOTO 30
20 max_result% = max_b%
        RETURN
30 REM END IF
    RETURN
' end function max%

' function min%(a%, b%)
40 IF (min_a% < min_b%) = 0 THEN GOTO 50
        min_result% = min_a%
        RETURN
        GOTO 60
50 min_result% = min_b%
        RETURN
60 REM END IF
    RETURN
' end function min%

' function clamp%(value%, lo%, hi%)
70 ' Constrain value to [lo, hi].
    min_a% = clamp_value%
    min_b% = clamp_hi%
    GOSUB 40
    max_a% = clamp_lo%
    max_b% = min_result%
    GOSUB 10
    clamp_result% = max_result%
    RETURN
' end function clamp%

' function repeat$(text$, n%)
80 ' Concatenate text$ with itself n times.
    result$ = ""
    FOR i% = 1 TO repeat_n%
        result$ = result$ + repeat_text$
    NEXT i%
    repeat_result$ = result$
    RETURN
' end function repeat$

' function titleCase$(word$)
90 ' Capitalise first letter, lowercase remainder.
    ' Relies on the BASIC runtime's UCASE$/LCASE$ built-ins.
    IF (LEN(titlecase_word$) = 0) = 0 THEN GOTO 100
        titlecase_result$ = ""
        RETURN
100 REM END IF
    titlecase_result$ = UCASE$(LEFT$(titlecase_word$, 1)) + LCASE$(MID$(titlecase_word$, 2))
    RETURN
' end function titleCase$
