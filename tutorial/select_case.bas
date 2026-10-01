10 ' BASCAL generated BASIC -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Tutorial — SELECT CASE
40 ' 
50 ' SELECT CASE tests one expression against multiple patterns.  The
60 ' compiler evaluates the expression once, stores it in a temporary
70 ' variable, and emits an IF/goto dispatch chain.
80 ' 
90 ' Pattern forms:
100 ' case value               — exact match
110 ' case v1, v2, v3          — any of the listed values
120 ' case low to high         — inclusive range
130 ' case is <op> value       — comparison (=  <>  <  <=  >  >=)
140 ' case else                — default; must be the last clause
150 ' Integer select: convert numeric score to letter grade
160 score% = 85
170 BCCT2% = score%
180 IF (BCCT2% = 100) <> 0 THEN GOTO 250
190 IF (BCCT2% >= 90 AND BCCT2% <= 99) <> 0 THEN GOTO 270
200 IF (BCCT2% >= 80 AND BCCT2% <= 89) <> 0 THEN GOTO 290
210 IF (BCCT2% >= 70 AND BCCT2% <= 79) <> 0 THEN GOTO 320
220 IF (BCCT2% >= 60 AND BCCT2% <= 69) <> 0 THEN GOTO 340
230 IF (BCCT2% >= 0) <> 0 THEN GOTO 360
240 GOTO 380
250     PRINT "Perfect!"
260     GOTO 390
270     PRINT "A  — Excellent"
280     GOTO 390
290     PRINT "B  — Good"
300     ' score% = 85 matches here
310     GOTO 390
320     PRINT "C  — Satisfactory"
330     GOTO 390
340     PRINT "D  — Passing"
350     GOTO 390
360     PRINT "F  — Fail"
370     GOTO 390
380     PRINT "Invalid score"
390 REM END SELECT
400 ' String select: day-of-week classification
410 day$ = "Saturday"
420 BCCT4$ = day$
430 IF (BCCT4$ = "Monday" OR BCCT4$ = "Tuesday" OR BCCT4$ = "Wednesday" OR BCCT4$ = "Thursday" OR BCCT4$ = "Friday") <> 0 THEN GOTO 460
440 IF (BCCT4$ = "Saturday" OR BCCT4$ = "Sunday") <> 0 THEN GOTO 480
450 GOTO 510
460     PRINT day$ + " is a weekday"
470     GOTO 520
480     PRINT day$ + " is a weekend"
490     ' matches here
500     GOTO 520
510     PRINT "Unknown day: " + day$
520 REM END SELECT
530 ' IS comparisons on temperature
540 temp% = -3
550 BCCT6% = temp%
560 IF (BCCT6% < 0) <> 0 THEN GOTO 610
570 IF (BCCT6% < 10) <> 0 THEN GOTO 640
580 IF (BCCT6% < 20) <> 0 THEN GOTO 660
590 IF (BCCT6% < 30) <> 0 THEN GOTO 680
600 GOTO 700
610     PRINT "Below freezing ("; temp%; "°)"
620     ' matches here
630     GOTO 710
640     PRINT "Cold ("; temp%; "°)"
650     GOTO 710
660     PRINT "Cool ("; temp%; "°)"
670     GOTO 710
680     PRINT "Warm ("; temp%; "°)"
690     GOTO 710
700     PRINT "Hot ("; temp%; "°)"
710 REM END SELECT
720 ' Multi-value list on a menu choice
730 choice% = 2
740 BCCT8% = choice%
750 IF (BCCT8% = 1) <> 0 THEN GOTO 790
760 IF (BCCT8% = 2 OR BCCT8% = 3) <> 0 THEN GOTO 810
770 IF (BCCT8% = 4) <> 0 THEN GOTO 840
780 GOTO 860
790     PRINT "New game"
800     GOTO 870
810     PRINT "Load game"
820     ' choice% = 2 matches
830     GOTO 870
840     PRINT "Options"
850     GOTO 870
860     PRINT "Quit"
870 REM END SELECT
880 END
