10 ' BASCAL generated BASIC -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
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
290 GOTO 310
300     PRINT "Fail ("; score%; ")"
310 REM END IF
320 ' IF / ELSEIF / ELSE — grade classification
330 points% = 85
340 IF (points% >= 90) = 0 THEN GOTO 370
350     grade$ = "A"
360 GOTO 510
370     IF (points% >= 80) = 0 THEN GOTO 410
380         grade$ = "B"
390         ' points% = 85 lands here
400     GOTO 500
410         IF (points% >= 70) = 0 THEN GOTO 440
420             grade$ = "C"
430         GOTO 490
440             IF (points% >= 60) = 0 THEN GOTO 470
450                 grade$ = "D"
460             GOTO 480
470                 grade$ = "F"
480             REM END IF
490         REM END IF
500     REM END IF
510 REM END IF
520 PRINT "Grade: " + grade$
530 ' Nested IF
540 x% = 15
550 IF (x% > 0) = 0 THEN GOTO 620
560     IF (x% > 10) = 0 THEN GOTO 590
570         PRINT x%; "is large and positive"
580     GOTO 600
590         PRINT x%; "is small and positive"
600     REM END IF
610 GOTO 630
620     PRINT x%; "is not positive"
630 REM END IF
640 ' Single-line IF -- no end if needed
650 temperature% = 23
660 IF (temperature% > 30) = 0 THEN GOTO 680
670     PRINT "Hot day (single-line)"
680 REM END IF
690 IF (temperature% > 100) = 0 THEN GOTO 720
700     PRINT "Scorching"
710 GOTO 730
720     PRINT "Not scorching"
730 REM END IF
740 ' Compound conditions
750 age% = 25
760 income% = 45000
770 IF ((age% >= 18) AND (income% >= 30000)) = 0 THEN GOTO 800
780     PRINT "Eligible"
790 GOTO 810
800     PRINT "Not eligible"
810 REM END IF
820 END
