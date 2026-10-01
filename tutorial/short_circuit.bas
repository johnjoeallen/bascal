10 ' BASCAL generated BASIC -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Tutorial — Short-Circuit && and ||
40 ' 
50 ' Classic BASIC's AND/OR are bitwise and always evaluate both sides -- there
60 ' is no short-circuit primitive in the generated BASIC at all. && and ||
70 ' give BASCAL real short-circuit evaluation instead: the second operand is
80 ' only evaluated once the first one hasn't already decided the answer.
90 ' 
100 ' a && b && c ...   -- true only if every operand is true; stops at the
110 ' first false operand.
120 ' a || b || c ...   -- true if any operand is true; stops at the first
130 ' true operand.
140 ' 
150 ' && / || are only usable directly in the condition of if / elseif / while
160 ' / do -- not as a general expression (can't be assigned to a variable or
170 ' passed as a function argument). A condition may chain any number of the
180 ' *same* operator; mixing && and || in one condition is a compile-time
190 ' error -- split into nested if statements instead.
200 ' ---- Guard clause: only check an array element when the index is valid ----
210 ' n% -- value to test
220 DIM scores%(5)
230 scores%(0) = 10
240 scores%(1) = -5
250 scores%(2) = 30
260 ' Long way: nested IF, so isPositive%() is only called when ptr% is valid.
270 PRINT "Long way (nested if), ptr% = -1:"
280 ptr% = -1
290 IF (ptr% >= 0) = 0 THEN GOTO 390
300     ispositiveN0% = scores%(ptr%)
310     GOSUB 1040
320     BCCT1% = ispositiveResult0%
330     IF (BCCT1% > 0) = 0 THEN GOTO 360
340         PRINT "  safe to read, value is positive"
350     GOTO 370
360         PRINT "  value is not positive"
370     REM END IF
380 GOTO 400
390     PRINT "  ptr% is out of range"
400 REM END IF
410 ' Short way: && short-circuits -- same safety, one line, one IF. Watch for
420 ' "(checking element)" in the output below: it does NOT print here, proving
430 ' isPositive%() was never called for an out-of-range ptr%.
440 PRINT "Short way (&&), ptr% = -1:"
450 IF (ptr% >= 0) = 0 THEN GOTO 520
460 ispositiveN0% = scores%(ptr%)
470 GOSUB 1040
480 BCCT4% = ispositiveResult0%
490 IF (BCCT4% > 0) = 0 THEN GOTO 520
500     PRINT "  safe to read, value is positive"
510 GOTO 530
520     PRINT "  ptr% is out of range or value is not positive"
530 REM END IF
540 ' Same short form, this time with a valid, positive element -- now
550 ' "(checking element)" DOES print, since ptr% >= 0 no longer stops it early.
560 PRINT "Short way (&&), ptr% = 2:"
570 ptr% = 2
580 IF (ptr% >= 0) = 0 THEN GOTO 650
590 ispositiveN0% = scores%(ptr%)
600 GOSUB 1040
610 BCCT7% = ispositiveResult0%
620 IF (BCCT7% > 0) = 0 THEN GOTO 650
630     PRINT "  safe to read, value is positive"
640 GOTO 660
650     PRINT "  ptr% is out of range or value is not positive"
660 REM END IF
670 ' ---- Retry loop: stop as soon as we succeed, or once out of attempts ----
680 ' Long way: a bare DO with a separate exit for each stopping condition.
690 PRINT "Long way (nested checks), retry loop:"
700 attempts% = 0
710 maxattempts% = 3
720 succeeded% = 0
730     attempts% = attempts% + 1
740     PRINT "  attempt "; attempts%
750     IF (attempts% = 2) = 0 THEN GOTO 770
760         succeeded% = 1
770     REM END IF
780     IF (succeeded% <> 0) = 0 THEN GOTO 800
790         GOTO 850
800     REM END IF
810     IF (attempts% >= maxattempts%) = 0 THEN GOTO 830
820         GOTO 850
830     REM END IF
840 GOTO 730
850 REM END DO
860 PRINT "  stopped after "; attempts%; " attempt(s), succeeded% = "; succeeded%
870 ' Short way: || short-circuits, so both stopping conditions live in the
880 ' loop's own until-clause -- no scattered exit checks needed.
890 PRINT "Short way (||), retry loop:"
900 attempts% = 0
910 succeeded% = 0
920 IF (succeeded% <> 0) <> 0 THEN GOTO 1000
930 IF (attempts% >= maxattempts%) <> 0 THEN GOTO 1000
940     attempts% = attempts% + 1
950     PRINT "  attempt "; attempts%
960     IF (attempts% = 2) = 0 THEN GOTO 980
970         succeeded% = 1
980     REM END IF
990 GOTO 920
1000 REM END DO
1010 PRINT "  stopped after "; attempts%; " attempt(s), succeeded% = "; succeeded%
1020 END

1030 ' function ispositive%(n%)
1040     ' A visible side effect, so the tutorial's own output proves whether
1050     ' this actually got called.
1060     PRINT "  (checking element)"
1070     ispositiveResult0% = ispositiveN0%
1080     RETURN
1090 ' end function ispositive%
