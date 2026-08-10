10 ' BASCAL generated BASIC
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Tutorial 16 — Short-Circuit && and ||
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

210 DIM scores%(5)
220 scores%(0) = 10
230 scores%(1) = -5
240 scores%(2) = 30

250 ' Long way: nested IF, so isPositive%() is only called when ptr% is valid.
260 PRINT "Long way (nested if), ptr% = -1:"
270 ptr% = -1
280 IF (ptr% >= 0) = 0 THEN GOTO 370
290     ispositive_n_0% = scores%(ptr%)
300     GOSUB 1000
310     IF (ispositive_result_0% > 0) = 0 THEN GOTO 340
320         PRINT "  safe to read, value is positive"
330         GOTO 350
340         PRINT "  value is not positive"
350     REM END IF
360     GOTO 380
370     PRINT "  ptr% is out of range"
380 REM END IF

390 ' Short way: && short-circuits -- same safety, one line, one IF. Watch for
400 ' "(checking element)" in the output below: it does NOT print here, proving
410 ' isPositive%() was never called for an out-of-range ptr%.
420 PRINT "Short way (&&), ptr% = -1:"
430 IF (ptr% >= 0) = 0 THEN GOTO 490
440 ispositive_n_0% = scores%(ptr%)
450 GOSUB 1000
460 IF (ispositive_result_0% > 0) = 0 THEN GOTO 490
470     PRINT "  safe to read, value is positive"
480     GOTO 500
490     PRINT "  ptr% is out of range or value is not positive"
500 REM END IF

510 ' Same short form, this time with a valid, positive element -- now
520 ' "(checking element)" DOES print, since ptr% >= 0 no longer stops it early.
530 PRINT "Short way (&&), ptr% = 2:"
540 ptr% = 2
550 IF (ptr% >= 0) = 0 THEN GOTO 610
560 ispositive_n_0% = scores%(ptr%)
570 GOSUB 1000
580 IF (ispositive_result_0% > 0) = 0 THEN GOTO 610
590     PRINT "  safe to read, value is positive"
600     GOTO 620
610     PRINT "  ptr% is out of range or value is not positive"
620 REM END IF

630 ' ---- Retry loop: stop as soon as we succeed, or once out of attempts ----

640 ' Long way: a bare DO with a separate exit for each stopping condition.
650 PRINT "Long way (nested checks), retry loop:"
660 attempts% = 0
670 maxattempts% = 3
680 succeeded% = 0
690     attempts% = attempts% + 1
700     PRINT "  attempt "; attempts%
710     IF (attempts% = 2) = 0 THEN GOTO 730
720         succeeded% = 1
730     REM END IF
740     IF (succeeded% <> 0) = 0 THEN GOTO 760
750         GOTO 810
760     REM END IF
770     IF (attempts% >= maxattempts%) = 0 THEN GOTO 790
780         GOTO 810
790     REM END IF
800     GOTO 690
810 REM END DO
820 PRINT "  stopped after "; attempts%; " attempt(s), succeeded% = "; succeeded%

830 ' Short way: || short-circuits, so both stopping conditions live in the
840 ' loop's own until-clause -- no scattered exit checks needed.
850 PRINT "Short way (||), retry loop:"
860 attempts% = 0
870 succeeded% = 0
880 IF (succeeded% <> 0) <> 0 THEN GOTO 960
890 IF (attempts% >= maxattempts%) <> 0 THEN GOTO 960
900     attempts% = attempts% + 1
910     PRINT "  attempt "; attempts%
920     IF (attempts% = 2) = 0 THEN GOTO 940
930         succeeded% = 1
940     REM END IF
950     GOTO 880
960 REM END DO
970 PRINT "  stopped after "; attempts%; " attempt(s), succeeded% = "; succeeded%

980 END

990 ' function ispositive%(n%)
1000     ' A visible side effect, so the tutorial's own output proves whether
1010     ' this actually got called.
1020     PRINT "  (checking element)"
1030     ispositive_result_0% = ispositive_n_0%
1040     RETURN
1050 ' end function ispositive%
