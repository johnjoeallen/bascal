' BASCAL generated BASIC
' Functions are lowered to global variables, labels, and GOSUB

' Tutorial 3 — Operators and Expressions
' 
' Arithmetic:   +  -  *  /  \  MOD  ^
' Comparison:   =  <>  <  <=  >  >=   (result: -1 true, 0 false)
' Logical:      AND  OR  NOT  XOR  (bitwise — see note below)
' String:       + concatenates strings
' 
' Precedence (highest first):
' ^                 exponentiation (right-associative)
' unary -           negation
' * /               multiply / divide
' \                 integer (floor) division
' MOD               modulus (remainder)
' + -               add / subtract
' = <> < <= > >=    comparison
' NOT               bitwise NOT
' AND               bitwise AND
' OR                bitwise OR
' XOR               bitwise XOR
' 
' IMPORTANT: NOT is bitwise, so NOT 1 = -2, not 0.
' Test for false with (expr) = 0, not NOT expr.

' Arithmetic — mix labels and numbers with ;
a% = 17
b% = 5
PRINT a%; "+ "; b%; "="; a% + b%
PRINT a%; "- "; b%; "="; a% - b%
PRINT a%; "* "; b%; "="; a% * b%
PRINT a%; "/ "; b%; "="; a% / b%

' Integer division and MOD
PRINT a%; "\ "; b%; "="; a% \ b%
PRINT a%; "MOD "; b%; "="; a% MOD b%

' Exponentiation — right-associative
PRINT "2 ^ 8 ="; 2 ^ 8
PRINT "2 ^ 3 ^ 2 ="; 2 ^ (3 ^ 2)

' Precedence
PRINT 2 + (3 * 4); " (expect 14 — * before +)"
PRINT (2 + 3) * 4; " (expect 20 — parens first)"

' Comparison — -1 means true, 0 means false
PRINT 10 > 3; " (expect -1)"
PRINT 10 < 3; " (expect  0)"
PRINT 7 = 7; " (expect -1)"
PRINT 7 <> 8; " (expect -1)"

' Logical — AND, OR, XOR are bitwise but work correctly with 0/-1 values
x% = 7
IF ((x% > 0) AND (x% < 10)) = 0 THEN GOTO 10
    PRINT x%; "is in 1..9"
10 REM END IF
PRINT 6 XOR 3; " (expect 5 — 110 XOR 011 = 101)"

' String concatenation
PRINT (("Hello" + ", ") + "World") + "!"

' Unary negation
n% = 42
PRINT -n%

END
