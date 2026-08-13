' BASCAL generated BASIC
' Functions are transpiled to global variables, labels, and GOSUB

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
' a% -- first value to compare
' b% -- second value to compare

' a% -- first value to compare
' b% -- second value to compare

' value% -- number to constrain
' lo%    -- lower bound, inclusive
' hi%    -- upper bound, inclusive

' String functions
' text$ -- string to repeat
' n%    -- number of times to repeat it

' word$ -- string to title-case

' Local variable scoping — each function has its own i% and acc%
' n% -- upper bound of the sum, inclusive

' n% -- upper bound of the product, inclusive

' Global variable accessed inside a function with the global keyword
runningtotal% = 0

' x% -- amount to add to the running total

' --- Exercise the functions ---

' print mixes string labels and numeric results directly with ;
max_a_0% = 4
max_b_0% = 9
GOSUB 10
PRINT "max(4, 9) = "; max_result_0%
min_a_0% = 4
min_b_0% = 9
GOSUB 40
PRINT "min(4, 9) = "; min_result_0%
clamp_value_0% = 15
clamp_lo_0% = 1
clamp_hi_0% = 10
GOSUB 70
PRINT "clamp(15,1,10) = "; clamp_result_0%
clamp_value_0% = -3
clamp_lo_0% = 1
clamp_hi_0% = 10
GOSUB 70
PRINT "clamp(-3,1,10) = "; clamp_result_0%
clamp_value_0% = 7
clamp_lo_0% = 1
clamp_hi_0% = 10
GOSUB 70
PRINT "clamp(7,1,10)  = "; clamp_result_0%

repeat_text_0$ = "ab"
repeat_n_0% = 4
GOSUB 80
PRINT repeat_result_0$
titlecase_word_0$ = "bASCAL"
GOSUB 90
PRINT titlecase_result_0$

' Functions chained in expressions
max_a_0% = 0
max_b_0% = -5
GOSUB 10
min_a_0% = max_result_0%
min_b_0% = 100
GOSUB 40
lo% = min_result_0%
PRINT "lo = "; lo%

' Calling the same function twice — each result is captured separately
repeat_text_0$ = "x"
repeat_n_0% = 3
GOSUB 80
a$ = repeat_result_0$
repeat_text_0$ = "y"
repeat_n_0% = 2
GOSUB 80
b$ = repeat_result_0$
PRINT a$; " "; b$

' Local scoping: sumTo% and productTo% each use i% without conflict
sumto_n_0% = 5
GOSUB 110
PRINT "sumTo(5)     = "; sumto_result_0%
productto_n_0% = 5
GOSUB 120
PRINT "productTo(5) = "; productto_result_0%

' Global variable shared across calls
addtototal_x_0% = 10
GOSUB 130
dummy% = addtototal_result_0%
addtototal_x_0% = 5
GOSUB 130
dummy% = addtototal_result_0%
PRINT "runningTotal = "; runningtotal%

END

' function max%(a%, b%)
10 IF (max_a_0% > max_b_0%) = 0 THEN GOTO 20
        max_result_0% = max_a_0%
        RETURN
        GOTO 30
20 max_result_0% = max_b_0%
        RETURN
30 REM END IF
    RETURN
' end function max%

' function min%(a%, b%)
40 IF (min_a_0% < min_b_0%) = 0 THEN GOTO 50
        min_result_0% = min_a_0%
        RETURN
        GOTO 60
50 min_result_0% = min_b_0%
        RETURN
60 REM END IF
    RETURN
' end function min%

' function clamp%(value%, lo%, hi%)
70 ' Constrain value to [lo, hi].
    min_a_0% = clamp_value_0%
    min_b_0% = clamp_hi_0%
    GOSUB 40
    max_a_0% = clamp_lo_0%
    max_b_0% = min_result_0%
    GOSUB 10
    clamp_result_0% = max_result_0%
    RETURN
' end function clamp%

' function repeat$(text$, n%)
80 ' Concatenate text$ with itself n times.
    repeat_acc_0$ = ""
    FOR repeat_i_0% = 1 TO repeat_n_0%
        repeat_acc_0$ = repeat_acc_0$ + repeat_text_0$
    NEXT repeat_i_0%
    repeat_result_0$ = repeat_acc_0$
    RETURN
' end function repeat$

' function titlecase$(word$)
90 ' Capitalise first letter, lowercase remainder.
    ' Relies on the BASIC runtime's UCASE$/LCASE$ built-ins.
    IF (LEN(titlecase_word_0$) = 0) = 0 THEN GOTO 100
        titlecase_result_0$ = ""
        RETURN
100 REM END IF
    titlecase_result_0$ = UCASE$(LEFT$(titlecase_word_0$, 1)) + LCASE$(MID$(titlecase_word_0$, 2))
    RETURN
' end function titlecase$

' function sumto%(n%)
110 ' i% and acc% are local to sumTo%.
    sumto_acc_0% = 0
    FOR sumto_i_0% = 1 TO sumto_n_0%
        sumto_acc_0% = sumto_acc_0% + sumto_i_0%
    NEXT sumto_i_0%
    sumto_result_0% = sumto_acc_0%
    RETURN
' end function sumto%

' function productto%(n%)
120 ' i% and acc% here are independent of sumTo%'s i% and acc%.
    productto_acc_0% = 1
    FOR productto_i_0% = 1 TO productto_n_0%
        productto_acc_0% = productto_acc_0% * productto_i_0%
    NEXT productto_i_0%
    productto_result_0% = productto_acc_0%
    RETURN
' end function productto%

' function addtototal%(x%)
130 runningtotal% = runningtotal% + addtototal_x_0%
    addtototal_result_0% = runningtotal%
    RETURN
' end function addtototal%
