' BASCAL generated BASIC
' Functions are lowered to global variables, labels, and GOSUB

' Nested multiline IF example.
score% = 97

' The compiler lowers each block IF to numeric GOTO targets.
IF (score% >= 90) = 0 THEN GOTO 30
    IF (score% >= 95) = 0 THEN GOTO 10
        PRINT "A+"
        GOTO 20
10 PRINT "A"
20 REM END IF
    GOTO 40
30 PRINT "Not A"
40 REM END IF

END
