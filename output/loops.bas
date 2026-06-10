' BASCAL generated BASIC
' Functions are lowered to global variables, labels, and GOSUB

' Loop example covering FOR/NEXT and WHILE/WEND.
DIM values%(10)

' Fill an array with even numbers.
FOR i% = 1 TO 10
    values%(i%) = i% * 2
NEXT i%

' Walk the array with WHILE/WEND.
i% = 1
10 IF (i% <= 10) = 0 THEN GOTO 20
    PRINT values%(i%)
    i% = i% + 1
    GOTO 10
20 REM END WHILE

END
