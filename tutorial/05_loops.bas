' BASCAL generated BASIC
' Functions are lowered to global variables, labels, and GOSUB

' Tutorial 5 — Loops: FOR, WHILE, DO
' 
' BASCAL provides three loop constructs:
' 
' FOR var = start TO end [STEP n] ... NEXT [var]
' Counted loop.  STEP defaults to 1; use negative STEP to count down.
' EXIT FOR jumps past NEXT immediately.
' 
' WHILE condition ... WEND
' Condition tested before each iteration.
' EXIT WHILE jumps past WEND immediately.
' 
' DO [WHILE/UNTIL cond] ... LOOP [WHILE/UNTIL cond]
' Four forms depending on where the condition is placed.
' EXIT DO jumps past LOOP immediately.

' --- FOR / NEXT ---
PRINT "Squares 1..5:"
FOR i% = 1 TO 5
    PRINT (("  " + STR$(i%)) + "^2 = ") + STR$(i% * i%)
NEXT i%

' Negative STEP — count down
PRINT "Countdown:"
FOR n% = 3 TO 1 STEP -1
    PRINT "  " + STR$(n%)
NEXT n%
PRINT "  Go!"

' EXIT FOR — stop early
PRINT "First even > 4:"
FOR i% = 1 TO 20
    IF ((i% > 4) AND (((i% / 2) * 2) = i%)) = 0 THEN GOTO 10
        PRINT "  " + STR$(i%)
        EXIT FOR
10 REM END IF
NEXT i%

' --- WHILE / WEND ---
PRINT "Powers of 2 under 100:"
p% = 1
20 IF (p% < 100) = 0 THEN GOTO 30
    PRINT "  " + STR$(p%)
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
    PRINT "  " + STR$(n%)
    GOTO 40
80 REM END WHILE

' --- DO / LOOP variants ---

' DO WHILE — test before body
PRINT "DO WHILE:"
k% = 1
90 IF (k% <= 3) = 0 THEN GOTO 100
    PRINT "  " + STR$(k%)
    k% = k% + 1
100 REM END DO

' DO UNTIL — enter while condition is false
PRINT "DO UNTIL:"
k% = 1
110 IF (k% > 3) <> 0 THEN GOTO 120
    PRINT "  " + STR$(k%)
    k% = k% + 1
120 REM END DO

' DO ... LOOP WHILE — body runs at least once
PRINT "DO...LOOP WHILE (body runs once even though false):"
k% = 99
130 PRINT "  " + STR$(k%)
    k% = k% + 1
    IF (k% <= 3) <> 0 THEN GOTO 130
140 REM END DO

' EXIT DO
PRINT "EXIT DO at 3:"
k% = 1
150 IF (k% = 3) = 0 THEN GOTO 160
        GOTO 170
160 REM END IF
    PRINT "  " + STR$(k%)
    k% = k% + 1
    GOTO 150
170 REM END DO

END
