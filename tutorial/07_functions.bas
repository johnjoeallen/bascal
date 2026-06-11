' BASCAL generated BASIC
' Functions are lowered to global variables, labels, and GOSUB

' Tutorial 7 — Functions
' 
' A BASCAL function is declared with FUNCTION ... END FUNCTION.
' The function name carries the return type suffix.  Parameters
' also carry type suffixes.  Every function must reach a RETURN.
' 
' Variables declared inside a function are local by default: the compiler
' prefixes them with the function name.  To access a global variable from
' inside a function, declare it with:  global varname
' 
' Functions cannot call themselves recursively (parameters would be
' overwritten).  Use an explicit stack array for recursive algorithms.

' Integer arithmetic functions

' String functions

' Local variable scoping — each function has its own i% and acc%

' Global variable accessed inside a function with the global keyword
runningtotal% = 0

' --- Exercise the functions ---

' print mixes string labels and numeric results directly with ;
max_a% = 4
max_b% = 9
GOSUB 10
PRINT "max(4, 9) = "; max_result%
min_a% = 4
min_b% = 9
GOSUB 40
PRINT "min(4, 9) = "; min_result%
clamp_value% = 15
clamp_lo% = 1
clamp_hi% = 10
GOSUB 70
PRINT "clamp(15,1,10) = "; clamp_result%
clamp_value% = -3
clamp_lo% = 1
clamp_hi% = 10
GOSUB 70
PRINT "clamp(-3,1,10) = "; clamp_result%
clamp_value% = 7
clamp_lo% = 1
clamp_hi% = 10
GOSUB 70
PRINT "clamp(7,1,10)  = "; clamp_result%

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
PRINT "lo = "; lo%

' Calling the same function twice — each result is captured separately
repeat_text$ = "x"
repeat_n% = 3
GOSUB 80
a$ = repeat_result$
repeat_text$ = "y"
repeat_n% = 2
GOSUB 80
b$ = repeat_result$
PRINT a$; " "; b$

' Local scoping: sumTo% and productTo% each use i% without conflict
sumto_n% = 5
GOSUB 110
PRINT "sumTo(5)     = "; sumto_result%
productto_n% = 5
GOSUB 120
PRINT "productTo(5) = "; productto_result%

' Global variable shared across calls
addtototal_x% = 10
GOSUB 130
dummy% = addtototal_result%
addtototal_x% = 5
GOSUB 130
dummy% = addtototal_result%
PRINT "runningTotal = "; runningtotal%

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
    repeat_acc$ = ""
    FOR repeat_i% = 1 TO repeat_n%
        repeat_acc$ = repeat_acc$ + repeat_text$
    NEXT repeat_i%
    repeat_result$ = repeat_acc$
    RETURN
' end function repeat$

' function titlecase$(word$)
90 ' Capitalise first letter, lowercase remainder.
    ' Relies on the BASIC runtime's UCASE$/LCASE$ built-ins.
    IF (LEN(titlecase_word$) = 0) = 0 THEN GOTO 100
        titlecase_result$ = ""
        RETURN
100 REM END IF
    titlecase_result$ = UCASE$(LEFT$(titlecase_word$, 1)) + LCASE$(MID$(titlecase_word$, 2))
    RETURN
' end function titlecase$

' function sumto%(n%)
110 ' i% and acc% are local to sumTo%.
    sumto_acc% = 0
    FOR sumto_i% = 1 TO sumto_n%
        sumto_acc% = sumto_acc% + sumto_i%
    NEXT sumto_i%
    sumto_result% = sumto_acc%
    RETURN
' end function sumto%

' function productto%(n%)
120 ' i% and acc% here are independent of sumTo%'s i% and acc%.
    productto_acc% = 1
    FOR productto_i% = 1 TO productto_n%
        productto_acc% = productto_acc% * productto_i%
    NEXT productto_i%
    productto_result% = productto_acc%
    RETURN
' end function productto%

' function addtototal%(x%)
130 runningtotal% = runningtotal% + addtototal_x%
    addtototal_result% = runningtotal%
    RETURN
' end function addtototal%
