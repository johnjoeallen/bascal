10 ' BASCAL generated BASIC
20 ' Functions are lowered to global variables, labels, and GOSUB

30 ' Tutorial 3 — Operators and Expressions
40 ' 
50 ' Arithmetic:   +  -  *  /  \  MOD  ^
60 ' Comparison:   =  <>  <  <=  >  >=   (result: -1 true, 0 false)
70 ' Logical:      AND  OR  NOT  XOR  (bitwise — see note below)
80 ' String:       + concatenates strings
90 ' 
100 ' Precedence (highest first):
110 ' ^                 exponentiation (right-associative)
120 ' unary -           negation
130 ' * /               multiply / divide
140 ' \                 integer (floor) division
150 ' MOD               modulus (remainder)
160 ' + -               add / subtract
170 ' = <> < <= > >=    comparison
180 ' NOT               bitwise NOT
190 ' AND               bitwise AND
200 ' OR                bitwise OR
210 ' XOR               bitwise XOR
220 ' 
230 ' IMPORTANT: NOT is bitwise, so NOT 1 = -2, not 0.
240 ' Test for false with (expr) = 0, not NOT expr.

250 ' Arithmetic — mix labels and numbers with ;
260 a% = 17
270 b% = 5
280 PRINT a%; "+ "; b%; "="; a% + b%
290 PRINT a%; "- "; b%; "="; a% - b%
300 PRINT a%; "* "; b%; "="; a% * b%
310 PRINT a%; "/ "; b%; "="; a% / b%

320 ' Integer division and MOD
330 PRINT a%; "\ "; b%; "="; a% \ b%
340 PRINT a%; "MOD "; b%; "="; a% MOD b%

350 ' Exponentiation — right-associative
360 PRINT "2 ^ 8 ="; 2 ^ 8
370 PRINT "2 ^ 3 ^ 2 ="; 2 ^ (3 ^ 2)

380 ' Precedence
390 PRINT 2 + (3 * 4); " (expect 14 — * before +)"
400 PRINT (2 + 3) * 4; " (expect 20 — parens first)"

410 ' Comparison — -1 means true, 0 means false
420 PRINT 10 > 3; " (expect -1)"
430 PRINT 10 < 3; " (expect  0)"
440 PRINT 7 = 7; " (expect -1)"
450 PRINT 7 <> 8; " (expect -1)"

460 ' Logical — AND, OR, XOR are bitwise but work correctly with 0/-1 values
470 x% = 7
480 IF ((x% > 0) AND (x% < 10)) = 0 THEN GOTO 500
490     PRINT x%; "is in 1..9"
500 REM END IF
510 PRINT 6 XOR 3; " (expect 5 — 110 XOR 011 = 101)"

520 ' String concatenation
530 PRINT (("Hello" + ", ") + "World") + "!"

540 ' Unary negation
550 n% = 42
560 PRINT -n%

570 END
