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
210 IF (BCCT2% >= 70 AND BCCT2% <= 79) <> 0 THEN GOTO 310
220 IF (BCCT2% >= 60 AND BCCT2% <= 69) <> 0 THEN GOTO 330
230 IF (BCCT2% >= 0) <> 0 THEN GOTO 350
240 GOTO 370
250     PRINT "Perfect!"
260     GOTO 380
270     PRINT "A  — Excellent"
280     GOTO 380
290     PRINT "B  — Good"
300     GOTO 380
310     PRINT "C  — Satisfactory"
320     GOTO 380
330     PRINT "D  — Passing"
340     GOTO 380
350     PRINT "F  — Fail"
360     GOTO 380
370     PRINT "Invalid score"
380 REM END SELECT

390 ' String select: day-of-week classification
400 day$ = "Saturday"

410 BCCT4$ = day$
420 IF (BCCT4$ = "Monday" OR BCCT4$ = "Tuesday" OR BCCT4$ = "Wednesday" OR BCCT4$ = "Thursday" OR BCCT4$ = "Friday") <> 0 THEN GOTO 450
430 IF (BCCT4$ = "Saturday" OR BCCT4$ = "Sunday") <> 0 THEN GOTO 470
440 GOTO 490
450     PRINT day$ + " is a weekday"
460     GOTO 500
470     PRINT day$ + " is a weekend"
480     GOTO 500
490     PRINT "Unknown day: " + day$
500 REM END SELECT

510 ' IS comparisons on temperature
520 temp% = -3

530 BCCT6% = temp%
540 IF (BCCT6% < 0) <> 0 THEN GOTO 590
550 IF (BCCT6% < 10) <> 0 THEN GOTO 610
560 IF (BCCT6% < 20) <> 0 THEN GOTO 630
570 IF (BCCT6% < 30) <> 0 THEN GOTO 650
580 GOTO 670
590     PRINT "Below freezing ("; temp%; "°)"
600     GOTO 680
610     PRINT "Cold ("; temp%; "°)"
620     GOTO 680
630     PRINT "Cool ("; temp%; "°)"
640     GOTO 680
650     PRINT "Warm ("; temp%; "°)"
660     GOTO 680
670     PRINT "Hot ("; temp%; "°)"
680 REM END SELECT

690 ' Multi-value list on a menu choice
700 choice% = 2

710 BCCT8% = choice%
720 IF (BCCT8% = 1) <> 0 THEN GOTO 760
730 IF (BCCT8% = 2 OR BCCT8% = 3) <> 0 THEN GOTO 780
740 IF (BCCT8% = 4) <> 0 THEN GOTO 800
750 GOTO 820
760     PRINT "New game"
770     GOTO 830
780     PRINT "Load game"
790     GOTO 830
800     PRINT "Options"
810     GOTO 830
820     PRINT "Quit"
830 REM END SELECT

840 END
