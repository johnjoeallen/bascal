10 ' BASCAL generated BASIC
20 ' Functions are lowered to global variables, labels, and GOSUB

30 ' Tutorial 4 — Conditions: IF / ELSEIF / ELSE / END IF
40 ' 
50 ' BASCAL supports multi-line block IF statements.  The compiler lowers
60 ' them to numeric goto targets so the generated BASIC is compatible with
70 ' 1980s BASCOM.  You never write line numbers yourself.
80 ' 
90 ' Forms:
100 ' if cond then ... end if
110 ' if cond then ... else ... end if
120 ' if cond then ... elseif cond then ... else ... end if

130 ' Simple IF
140 temperature% = 23
150 IF (temperature% > 30) = 0 THEN GOTO 170
160     PRINT "Hot day"
170 REM END IF

180 ' IF / ELSE
190 score% = 72
200 IF (score% >= 60) = 0 THEN GOTO 230
210     PRINT "Pass ("; score%; ")"
220     GOTO 240
230     PRINT "Fail ("; score%; ")"
240 REM END IF

250 ' IF / ELSEIF / ELSE — grade classification
260 points% = 85

270 IF (points% >= 90) = 0 THEN GOTO 300
280     grade$ = "A"
290     GOTO 430
300     IF (points% >= 80) = 0 THEN GOTO 330
310         grade$ = "B"
320         GOTO 420
330         IF (points% >= 70) = 0 THEN GOTO 360
340             grade$ = "C"
350             GOTO 410
360             IF (points% >= 60) = 0 THEN GOTO 390
370                 grade$ = "D"
380                 GOTO 400
390                 grade$ = "F"
400             REM END IF
410         REM END IF
420     REM END IF
430 REM END IF

440 PRINT "Grade: " + grade$

450 ' Nested IF
460 x% = 15
470 IF (x% > 0) = 0 THEN GOTO 540
480     IF (x% > 10) = 0 THEN GOTO 510
490         PRINT x%; "is large and positive"
500         GOTO 520
510         PRINT x%; "is small and positive"
520     REM END IF
530     GOTO 550
540     PRINT x%; "is not positive"
550 REM END IF

560 ' Compound conditions
570 age% = 25
580 income% = 45000
590 IF ((age% >= 18) AND (income% >= 30000)) = 0 THEN GOTO 620
600     PRINT "Eligible"
610     GOTO 630
620     PRINT "Not eligible"
630 REM END IF

640 END
