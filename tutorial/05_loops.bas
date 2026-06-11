' BASCAL generated BASIC
' Functions are lowered to global variables, labels, and GOSUB

' Tutorial 5 — Loops: for, WHILE, DO
' 
' BASCAL provides three loop constructs:
' 
' for var = start to end [STEP n] ... for END  (or bare END)
' Counted loop.  STEP defaults to 1; use negative STEP to count down.
' EXIT for exits early.
' 
' WHILE condition ... WHILE END  (or bare END)
' Condition tested before each iteration.
' EXIT WHILE exits early.
' 
' DO [WHILE/UNTIL cond] ... DO END  (or bare END)
' Condition tested at the top; use EXIT DO to break out.
' EXIT DO exits early.

' --- for / NEXT ---
PRINT "Squares 1..5:"
FOR i% = 1 TO 5
    PRINT "  "; i%; "^2 = "; i% * i%
NEXT i%

' Negative STEP — count down
PRINT "Countdown:"
FOR n% = 3 TO 1 STEP -1
    PRINT "  "; n%
NEXT n%
PRINT "  Go!"

' EXIT for — stop early
PRINT "First even > 4:"
FOR i% = 1 TO 20
    IF ((i% > 4) AND (((i% / 2) * 2) = i%)) = 0 THEN GOTO 10
        PRINT "  "; i%
        EXIT FOR
10 REM END IF
NEXT i%

' --- WHILE / WEND ---
PRINT "Powers of 2 under 100:"
p% = 1
20 IF (p% < 100) = 0 THEN GOTO 30
    PRINT "  "; p%
    p% = p% * 2
    GOTO 20
30 REM END WHILE

' EXIT WHILE
PRINT "Collatz from 27 (first 8 steps):"
n% = 27
steps% = 0
40 IF (n% <> 1) = 0 THEN GOTO 80
    IF (steps% = 8) = 0 THEN GOTO 50
        PRINT "  ..."
        GOTO 80
50 REM END IF
    IF (((n% / 2) * 2) = n%) = 0 THEN GOTO 60
        n% = n% / 2
        GOTO 70
60 n% = (n% * 3) + 1
70 REM END IF
    steps% = steps% + 1
    PRINT "  "; n%
    GOTO 40
80 REM END WHILE

' --- DO / LOOP variants ---

' DO WHILE — test before body
PRINT "DO WHILE:"
k% = 1
90 IF (k% <= 3) = 0 THEN GOTO 100
    PRINT "  "; k%
    k% = k% + 1
100 REM END DO

' DO UNTIL — enter while condition is false
PRINT "DO UNTIL:"
k% = 1
110 IF (k% > 3) <> 0 THEN GOTO 120
    PRINT "  "; k%
    k% = k% + 1
120 REM END DO

' DO ... DO END with post-check — body runs at least once
PRINT "DO...DO END (body runs once even though false):"
k% = 99
130 PRINT "  "; k%
    k% = k% + 1
    IF (k% > 3) = 0 THEN GOTO 140
        GOTO 150
140 REM END IF
    GOTO 130
150 REM END DO

' EXIT DO
PRINT "EXIT DO at 3:"
k% = 1
160 IF (k% = 3) = 0 THEN GOTO 170
        GOTO 180
170 REM END IF
    PRINT "  "; k%
    k% = k% + 1
    GOTO 160
180 REM END DO

END
