10 ' BASCAL generated BASIC
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
290 IF (ptr% >= 0) = 0 THEN GOTO 380
300     ispositiveN0% = scores%(ptr%)
310     GOSUB 1010
320     IF (ispositiveResult0% > 0) = 0 THEN GOTO 350
330         PRINT "  safe to read, value is positive"
340         GOTO 360
350         PRINT "  value is not positive"
360     REM END IF
370     GOTO 390
380     PRINT "  ptr% is out of range"
390 REM END IF

400 ' Short way: && short-circuits -- same safety, one line, one IF. Watch for
410 ' "(checking element)" in the output below: it does NOT print here, proving
420 ' isPositive%() was never called for an out-of-range ptr%.
430 PRINT "Short way (&&), ptr% = -1:"
440 IF (ptr% >= 0) = 0 THEN GOTO 500
450 ispositiveN0% = scores%(ptr%)
460 GOSUB 1010
470 IF (ispositiveResult0% > 0) = 0 THEN GOTO 500
480     PRINT "  safe to read, value is positive"
490     GOTO 510
500     PRINT "  ptr% is out of range or value is not positive"
510 REM END IF

520 ' Same short form, this time with a valid, positive element -- now
530 ' "(checking element)" DOES print, since ptr% >= 0 no longer stops it early.
540 PRINT "Short way (&&), ptr% = 2:"
550 ptr% = 2
560 IF (ptr% >= 0) = 0 THEN GOTO 620
570 ispositiveN0% = scores%(ptr%)
580 GOSUB 1010
590 IF (ispositiveResult0% > 0) = 0 THEN GOTO 620
600     PRINT "  safe to read, value is positive"
610     GOTO 630
620     PRINT "  ptr% is out of range or value is not positive"
630 REM END IF

640 ' ---- Retry loop: stop as soon as we succeed, or once out of attempts ----

650 ' Long way: a bare DO with a separate exit for each stopping condition.
660 PRINT "Long way (nested checks), retry loop:"
670 attempts% = 0
680 maxattempts% = 3
690 succeeded% = 0
700     attempts% = attempts% + 1
710     PRINT "  attempt "; attempts%
720     IF (attempts% = 2) = 0 THEN GOTO 740
730         succeeded% = 1
740     REM END IF
750     IF (succeeded% <> 0) = 0 THEN GOTO 770
760         GOTO 820
770     REM END IF
780     IF (attempts% >= maxattempts%) = 0 THEN GOTO 800
790         GOTO 820
800     REM END IF
810     GOTO 700
820 REM END DO
830 PRINT "  stopped after "; attempts%; " attempt(s), succeeded% = "; succeeded%

840 ' Short way: || short-circuits, so both stopping conditions live in the
850 ' loop's own until-clause -- no scattered exit checks needed.
860 PRINT "Short way (||), retry loop:"
870 attempts% = 0
880 succeeded% = 0
890 IF (succeeded% <> 0) <> 0 THEN GOTO 970
900 IF (attempts% >= maxattempts%) <> 0 THEN GOTO 970
910     attempts% = attempts% + 1
920     PRINT "  attempt "; attempts%
930     IF (attempts% = 2) = 0 THEN GOTO 950
940         succeeded% = 1
950     REM END IF
960     GOTO 890
970 REM END DO
980 PRINT "  stopped after "; attempts%; " attempt(s), succeeded% = "; succeeded%

990 END

1000 ' function ispositive%(n%)
1010     ' A visible side effect, so the tutorial's own output proves whether
1020     ' this actually got called.
1030     PRINT "  (checking element)"
1040     ispositiveResult0% = ispositiveN0%
1050     RETURN
1060 ' end function ispositive%
