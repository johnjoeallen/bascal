10 ' BASCAL generated BASIC
20 ' Functions are lowered to global variables, labels, and GOSUB

30 ' Tutorial 3 — Operators and Expressions
40 ' 
50 ' Arithmetic:   +  -  *  /
60 ' Comparison:   =  <>  <  <=  >  >=   (result: -1 true, 0 false)
70 ' Logical:      AND  OR  NOT  (bitwise — see note below)
80 ' String:       + concatenates strings
90 ' 
100 ' Precedence (highest first):
110 ' unary - NOT       level 7
120 ' * /               level 6
130 ' + -               level 5
140 ' = <> < <= > >=    level 4
150 ' AND               level 3
160 ' OR                level 2
170 ' 
180 ' IMPORTANT: NOT is bitwise, so NOT 1 = -2, not 0.
190 ' Test for false with (expr) = 0, not NOT expr.

200 ' Arithmetic
210 a% = 17
220 b% = 5
230 PRINT (((STR$(a%) + " + ") + STR$(b%)) + " = ") + STR$(a% + b%)
240 PRINT (((STR$(a%) + " - ") + STR$(b%)) + " = ") + STR$(a% - b%)
250 PRINT (((STR$(a%) + " * ") + STR$(b%)) + " = ") + STR$(a% * b%)
260 PRINT (((STR$(a%) + " / ") + STR$(b%)) + " = ") + STR$(a% / b%)

270 ' Precedence
280 PRINT STR$(2 + (3 * 4)) + "  (expect 14 — * before +)"
290 PRINT STR$((2 + 3) * 4) + "  (expect 20 — parens first)"

300 ' Comparison — -1 means true, 0 means false
310 PRINT STR$(10 > 3) + "  (expect -1)"
320 PRINT STR$(10 < 3) + "  (expect  0)"
330 PRINT STR$(7 = 7) + "  (expect -1)"
340 PRINT STR$(7 <> 8) + "  (expect -1)"

350 ' Logical — AND and OR are bitwise but work correctly with 0/-1 values
360 x% = 7
370 IF ((x% > 0) AND (x% < 10)) = 0 THEN GOTO 390
380     PRINT STR$(x%) + " is in 1..9"
390 REM END IF

400 ' String concatenation
410 PRINT (("Hello" + ", ") + "World") + "!"

420 ' Unary negation
430 n% = 42
440 PRINT STR$(-n%)

450 END
