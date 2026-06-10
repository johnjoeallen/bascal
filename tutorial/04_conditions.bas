' BASCAL generated BASIC
' Functions are lowered to global variables, labels, and GOSUB

' Tutorial 4 — Conditions: IF / ELSEIF / ELSE / END IF
' 
' BASCAL supports multi-line block IF statements.  The compiler lowers
' them to numeric GOTO targets so the generated BASIC is compatible with
' 1980s BASCOM.  You never write line numbers yourself.
' 
' Forms:
' if cond then ... end if
' if cond then ... else ... end if
' if cond then ... elseif cond then ... else ... end if

' Simple IF
temperature% = 23
IF (temperature% > 30) = 0 THEN GOTO 10
    PRINT "Hot day"
10 REM END IF

' IF / ELSE
score% = 72
IF (score% >= 60) = 0 THEN GOTO 20
    PRINT ("Pass (" + STR$(score%)) + ")"
    GOTO 30
20 PRINT ("Fail (" + STR$(score%)) + ")"
30 REM END IF

' IF / ELSEIF / ELSE — grade classification
points% = 85

IF (points% >= 90) = 0 THEN GOTO 40
    grade$ = "A"
    GOTO 110
40 IF (points% >= 80) = 0 THEN GOTO 50
        grade$ = "B"
        GOTO 100
50 IF (points% >= 70) = 0 THEN GOTO 60
            grade$ = "C"
            GOTO 90
60 IF (points% >= 60) = 0 THEN GOTO 70
                grade$ = "D"
                GOTO 80
70 grade$ = "F"
80 REM END IF
90 REM END IF
100 REM END IF
110 REM END IF

PRINT "Grade: " + grade$

' Nested IF
x% = 15
IF (x% > 0) = 0 THEN GOTO 140
    IF (x% > 10) = 0 THEN GOTO 120
        PRINT STR$(x%) + " is large and positive"
        GOTO 130
120 PRINT STR$(x%) + " is small and positive"
130 REM END IF
    GOTO 150
140 PRINT STR$(x%) + " is not positive"
150 REM END IF

' Compound conditions
age% = 25
income% = 45000
IF ((age% >= 18) AND (income% >= 30000)) = 0 THEN GOTO 160
    PRINT "Eligible"
    GOTO 170
160 PRINT "Not eligible"
170 REM END IF

END
