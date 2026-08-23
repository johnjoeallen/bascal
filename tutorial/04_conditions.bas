10 ' BASCAL generated BASIC
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Tutorial — Conditions: IF / ELSEIF / ELSE / END IF
40 ' 
50 ' BASCAL supports multi-line block IF statements.  The compiler transpiles
60 ' them to numeric goto targets so the generated BASIC is compatible with
70 ' 1980s BASCOM.  You never write line numbers yourself.
80 ' 
90 ' Forms:
100 ' if cond then ... end if
110 ' if cond then ... else ... end if
120 ' if cond then ... elseif cond then ... else ... end if
130 ' if cond then statement                   (single-line, no end if)
140 ' if cond then statement else statement     (single-line, no end if)
150 ' 
160 ' A newline right after `then` selects the block form; a statement
170 ' directly after `then` on the same line selects the single-line form
180 ' instead -- that's the only difference. elseif isn't available
190 ' single-line, same as classic BASIC.

200 ' Simple IF
210 temperature% = 23
220 IF (temperature% > 30) = 0 THEN GOTO 240
230     PRINT "Hot day"
240 REM END IF

250 ' IF / ELSE
260 score% = 72
270 IF (score% >= 60) = 0 THEN GOTO 300
280     PRINT "Pass ("; score%; ")"
290     GOTO 310
300     PRINT "Fail ("; score%; ")"
310 REM END IF

320 ' IF / ELSEIF / ELSE — grade classification
330 points% = 85

340 IF (points% >= 90) = 0 THEN GOTO 370
350     grade$ = "A"
360     GOTO 500
370     IF (points% >= 80) = 0 THEN GOTO 400
380         grade$ = "B"
390         GOTO 490
400         IF (points% >= 70) = 0 THEN GOTO 430
410             grade$ = "C"
420             GOTO 480
430             IF (points% >= 60) = 0 THEN GOTO 460
440                 grade$ = "D"
450                 GOTO 470
460                 grade$ = "F"
470             REM END IF
480         REM END IF
490     REM END IF
500 REM END IF

510 PRINT "Grade: " + grade$

520 ' Nested IF
530 x% = 15
540 IF (x% > 0) = 0 THEN GOTO 610
550     IF (x% > 10) = 0 THEN GOTO 580
560         PRINT x%; "is large and positive"
570         GOTO 590
580         PRINT x%; "is small and positive"
590     REM END IF
600     GOTO 620
610     PRINT x%; "is not positive"
620 REM END IF

630 ' Single-line IF -- no end if needed
640 temperature% = 23
650 IF (temperature% > 30) = 0 THEN GOTO 670
660     PRINT "Hot day (single-line)"
670 REM END IF
680 IF (temperature% > 100) = 0 THEN GOTO 710
690     PRINT "Scorching"
700     GOTO 720
710     PRINT "Not scorching"
720 REM END IF

730 ' Compound conditions
740 age% = 25
750 income% = 45000
760 IF ((age% >= 18) AND (income% >= 30000)) = 0 THEN GOTO 790
770     PRINT "Eligible"
780     GOTO 800
790     PRINT "Not eligible"
800 REM END IF

810 END
