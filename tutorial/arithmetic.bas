10 ' BASCAL generated BASIC -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Tutorial — Operators and Expressions
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
290 ' 17 + 5 = 22
300 PRINT a%; "- "; b%; "="; a% - b%
310 ' 17 - 5 = 12
320 PRINT a%; "* "; b%; "="; a% * b%
330 ' 17 * 5 = 85
340 PRINT a%; "/ "; b%; "="; a% / b%
350 ' 17 / 5 = 3  (truncates)
360 ' Integer division and MOD
370 PRINT a%; "\ "; b%; "="; a% \ b%
380 ' 17 \ 5 = 3  (integer quotient)
390 PRINT a%; "MOD "; b%; "="; a% MOD b%
400 ' 17 MOD 5 = 2  (remainder)
410 ' Exponentiation — right-associative
420 PRINT "2 ^ 8 ="; 2 ^ 8
430 ' 256
440 PRINT "2 ^ 3 ^ 2 ="; 2 ^ (3 ^ 2)
450 ' 512  (= 2 ^ (3^2) = 2^9)
460 ' Precedence
470 PRINT 2 + (3 * 4); " (expect 14 — * before +)"
480 PRINT (2 + 3) * 4; " (expect 20 — parens first)"
490 ' Comparison — -1 means true, 0 means false
500 PRINT 10 > 3; " (expect -1)"
510 PRINT 10 < 3; " (expect  0)"
520 PRINT 7 = 7; " (expect -1)"
530 PRINT 7 <> 8; " (expect -1)"
540 ' Logical — AND, OR, XOR are bitwise but work correctly with 0/-1 values
550 x% = 7
560 IF ((x% > 0) AND (x% < 10)) = 0 THEN GOTO 580
570     PRINT x%; "is in 1..9"
580 REM END IF
590 PRINT 6 XOR 3; " (expect 5 — 110 XOR 011 = 101)"
600 ' String concatenation
610 PRINT (("Hello" + ", ") + "World") + "!"
620 ' Unary negation
630 n% = 42
640 PRINT -n%
650 ' -42
660 END
