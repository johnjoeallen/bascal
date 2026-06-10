' BASCAL generated BASIC
' Functions are lowered to global variables, labels, and GOSUB

' Tutorial 6 — SELECT CASE
' 
' SELECT CASE tests one expression against multiple patterns.  The
' compiler evaluates the expression once, stores it in a temporary
' variable, and emits an IF/GOTO dispatch chain.
' 
' Pattern forms:
' case value               — exact match
' case v1, v2, v3          — any of the listed values
' case low to high         — inclusive range
' case is <op> value       — comparison (=  <>  <  <=  >  >=)
' case else                — default; must be the last clause

' Integer select: convert numeric score to letter grade
score% = 85

BCC_T2% = score%
IF (BCC_T2% = 100) <> 0 THEN GOTO 10
IF (BCC_T2% >= 90 AND BCC_T2% <= 99) <> 0 THEN GOTO 20
IF (BCC_T2% >= 80 AND BCC_T2% <= 89) <> 0 THEN GOTO 30
IF (BCC_T2% >= 70 AND BCC_T2% <= 79) <> 0 THEN GOTO 40
IF (BCC_T2% >= 60 AND BCC_T2% <= 69) <> 0 THEN GOTO 50
IF (BCC_T2% >= 0) <> 0 THEN GOTO 60
GOTO 70
10 PRINT "Perfect!"
    GOTO 80
20 PRINT "A  — Excellent"
    GOTO 80
30 PRINT "B  — Good"
    GOTO 80
40 PRINT "C  — Satisfactory"
    GOTO 80
50 PRINT "D  — Passing"
    GOTO 80
60 PRINT "F  — Fail"
    GOTO 80
70 PRINT "Invalid score"
80 REM END SELECT

' String select: day-of-week classification
day$ = "Saturday"

BCC_T4$ = day$
IF (BCC_T4$ = "Monday" OR BCC_T4$ = "Tuesday" OR BCC_T4$ = "Wednesday" OR BCC_T4$ = "Thursday" OR BCC_T4$ = "Friday") <> 0 THEN GOTO 90
IF (BCC_T4$ = "Saturday" OR BCC_T4$ = "Sunday") <> 0 THEN GOTO 100
GOTO 110
90 PRINT day$ + " is a weekday"
    GOTO 120
100 PRINT day$ + " is a weekend"
    GOTO 120
110 PRINT "Unknown day: " + day$
120 REM END SELECT

' IS comparisons on temperature
temp% = -3

BCC_T6% = temp%
IF (BCC_T6% < 0) <> 0 THEN GOTO 130
IF (BCC_T6% < 10) <> 0 THEN GOTO 140
IF (BCC_T6% < 20) <> 0 THEN GOTO 150
IF (BCC_T6% < 30) <> 0 THEN GOTO 160
GOTO 170
130 PRINT ("Below freezing (" + STR$(temp%)) + "°)"
    GOTO 180
140 PRINT ("Cold (" + STR$(temp%)) + "°)"
    GOTO 180
150 PRINT ("Cool (" + STR$(temp%)) + "°)"
    GOTO 180
160 PRINT ("Warm (" + STR$(temp%)) + "°)"
    GOTO 180
170 PRINT ("Hot (" + STR$(temp%)) + "°)"
180 REM END SELECT

' Multi-value list on a menu choice
choice% = 2

BCC_T8% = choice%
IF (BCC_T8% = 1) <> 0 THEN GOTO 190
IF (BCC_T8% = 2 OR BCC_T8% = 3) <> 0 THEN GOTO 200
IF (BCC_T8% = 4) <> 0 THEN GOTO 210
GOTO 220
190 PRINT "New game"
    GOTO 230
200 PRINT "Load game"
    GOTO 230
210 PRINT "Options"
    GOTO 230
220 PRINT "Quit"
230 REM END SELECT

END
