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
150 ' DO [WHILE/UNTIL cond] ... DO END  (or bare END)
160 ' Condition tested at the top; use EXIT DO to break out.
170 ' EXIT DO exits early.

180 ' --- for / NEXT ---
190 PRINT "Squares 1..5:"
200 FOR i% = 1 TO 5
210     PRINT "  "; i%; "^2 = "; i% * i%
220 NEXT i%

230 ' Negative STEP — count down
240 PRINT "Countdown:"
250 FOR n% = 3 TO 1 STEP -1
260     PRINT "  "; n%
270 NEXT n%
280 PRINT "  Go!"

290 ' EXIT for — stop early
300 PRINT "First even > 4:"
310 FOR i% = 1 TO 20
320     IF ((i% > 4) AND (((i% / 2) * 2) = i%)) = 0 THEN GOTO 350
330         PRINT "  "; i%
340         EXIT FOR
350     REM END IF
360 NEXT i%

370 ' --- WHILE / WEND ---
380 PRINT "Powers of 2 under 100:"
390 p% = 1
400 IF (p% < 100) = 0 THEN GOTO 440
410     PRINT "  "; p%
420     p% = p% * 2
430     GOTO 400
440 REM END WHILE

450 ' EXIT WHILE
460 PRINT "Collatz from 27 (first 8 steps):"
470 n% = 27
480 steps% = 0
490 IF (n% <> 1) = 0 THEN GOTO 620
500     IF (steps% = 8) = 0 THEN GOTO 530
510         PRINT "  ..."
520         GOTO 620
530     REM END IF
540     IF (((n% / 2) * 2) = n%) = 0 THEN GOTO 570
550         n% = n% / 2
560         GOTO 580
570         n% = (n% * 3) + 1
580     REM END IF
590     steps% = steps% + 1
600     PRINT "  "; n%
610     GOTO 490
620 REM END WHILE

630 ' --- DO / LOOP variants ---

640 ' DO WHILE — test before body
650 PRINT "DO WHILE:"
660 k% = 1
670 IF (k% <= 3) = 0 THEN GOTO 710
680     PRINT "  "; k%
690     k% = k% + 1
700     GOTO 670
710 REM END DO

720 ' DO UNTIL — enter while condition is false
730 PRINT "DO UNTIL:"
740 k% = 1
750 IF (k% > 3) <> 0 THEN GOTO 790
760     PRINT "  "; k%
770     k% = k% + 1
780     GOTO 750
790 REM END DO

800 ' DO ... DO END with post-check — body runs at least once
810 PRINT "DO...DO END (body runs once even though false):"
820 k% = 99
830     PRINT "  "; k%
840     k% = k% + 1
850     IF (k% > 3) = 0 THEN GOTO 870
860         GOTO 890
870     REM END IF
880     GOTO 830
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
