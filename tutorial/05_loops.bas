10 ' BASCAL generated BASIC
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Tutorial 5 — Loops: for, WHILE, DO
40 ' 
50 ' BASCAL provides three loop constructs:
60 ' 
70 ' for var = start to end [STEP n] ... for END  (or bare END)
80 ' Counted loop.  STEP defaults to 1; use negative STEP to count down.
90 ' EXIT for exits early.
100 ' 
110 ' WHILE condition ... WHILE END  (or bare END)
120 ' Condition tested before each iteration.
130 ' EXIT WHILE exits early.
140 ' 
150 ' DO [WHILE/UNTIL cond] ... END DO  (or bare END)
160 ' Pre-check: condition tested at the top, before the body runs at all.
170 ' DO ... LOOP [WHILE/UNTIL cond]
180 ' Post-check: condition tested at the bottom, so the body always runs
190 ' at least once.
200 ' Either form: EXIT DO exits early, from anywhere in the body.

210 ' --- for / NEXT ---
220 PRINT "Squares 1..5:"
230 FOR i% = 1 TO 5
240     PRINT "  "; i%; "^2 = "; i% * i%
250 NEXT i%

260 ' Negative STEP — count down
270 PRINT "Countdown:"
280 FOR n% = 3 TO 1 STEP -1
290     PRINT "  "; n%
300 NEXT n%
310 PRINT "  Go!"

320 ' EXIT for — stop early
330 PRINT "First even > 4:"
340 FOR i% = 1 TO 20
350     IF ((i% > 4) AND (((i% / 2) * 2) = i%)) = 0 THEN GOTO 380
360         PRINT "  "; i%
370         EXIT FOR
380     REM END IF
390 NEXT i%

400 ' --- WHILE / WEND ---
410 PRINT "Powers of 2 under 100:"
420 p% = 1
430 IF (p% < 100) = 0 THEN GOTO 470
440     PRINT "  "; p%
450     p% = p% * 2
460     GOTO 430
470 REM END WHILE

480 ' EXIT WHILE
490 PRINT "Collatz from 27 (first 8 steps):"
500 n% = 27
510 steps% = 0
520 IF (n% <> 1) = 0 THEN GOTO 650
530     IF (steps% = 8) = 0 THEN GOTO 560
540         PRINT "  ..."
550         GOTO 650
560     REM END IF
570     IF (((n% / 2) * 2) = n%) = 0 THEN GOTO 600
580         n% = n% / 2
590         GOTO 610
600         n% = (n% * 3) + 1
610     REM END IF
620     steps% = steps% + 1
630     PRINT "  "; n%
640     GOTO 520
650 REM END WHILE

660 ' --- DO / LOOP variants ---

670 ' DO WHILE — test before body
680 PRINT "DO WHILE:"
690 k% = 1
700 IF (k% <= 3) = 0 THEN GOTO 740
710     PRINT "  "; k%
720     k% = k% + 1
730     GOTO 700
740 REM END DO

750 ' DO UNTIL — enter while condition is false
760 PRINT "DO UNTIL:"
770 k% = 1
780 IF (k% > 3) <> 0 THEN GOTO 820
790     PRINT "  "; k%
800     k% = k% + 1
810     GOTO 780
820 REM END DO

830 ' DO ... LOOP UNTIL — post-check, body runs at least once
840 PRINT "DO...LOOP UNTIL (body runs once even though already false):"
850 k% = 99
860     PRINT "  "; k%
870     k% = k% + 1
880     IF (k% > 3) = 0 THEN GOTO 860
890 REM END DO

900 ' EXIT DO
910 PRINT "EXIT DO at 3:"
920 k% = 1
930     IF (k% = 3) = 0 THEN GOTO 950
940         GOTO 990
950     REM END IF
960     PRINT "  "; k%
970     k% = k% + 1
980     GOTO 930
990 REM END DO

1000 END
