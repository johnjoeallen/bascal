10 ' BASCAL generated BASIC
20 ' Functions are lowered to global variables, labels, and GOSUB

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
670 IF (k% <= 3) = 0 THEN GOTO 700
680     PRINT "  "; k%
690     k% = k% + 1
700 REM END DO

710 ' DO UNTIL — enter while condition is false
720 PRINT "DO UNTIL:"
730 k% = 1
740 IF (k% > 3) <> 0 THEN GOTO 770
750     PRINT "  "; k%
760     k% = k% + 1
770 REM END DO

780 ' DO ... DO END with post-check — body runs at least once
790 PRINT "DO...DO END (body runs once even though false):"
800 k% = 99
810     PRINT "  "; k%
820     k% = k% + 1
830     IF (k% > 3) = 0 THEN GOTO 850
840         GOTO 870
850     REM END IF
860     GOTO 810
870 REM END DO

880 ' EXIT DO
890 PRINT "EXIT DO at 3:"
900 k% = 1
910     IF (k% = 3) = 0 THEN GOTO 930
920         GOTO 970
930     REM END IF
940     PRINT "  "; k%
950     k% = k% + 1
960     GOTO 910
970 REM END DO

980 END
