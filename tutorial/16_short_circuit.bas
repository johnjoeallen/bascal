' BASCAL generated BASIC
' Functions are transpiled to global variables, labels, and GOSUB

' Tutorial 16 — Short-Circuit && and ||
' 
' Classic BASIC's AND/OR are bitwise and always evaluate both sides -- there
' is no short-circuit primitive in the generated BASIC at all. && and ||
' give BASCAL real short-circuit evaluation instead: the second operand is
' only evaluated once the first one hasn't already decided the answer.
' 
' a && b && c ...   -- true only if every operand is true; stops at the
' first false operand.
' a || b || c ...   -- true if any operand is true; stops at the first
' true operand.
' 
' && / || are only usable directly in the condition of if / elseif / while
' / do -- not as a general expression (can't be assigned to a variable or
' passed as a function argument). A condition may chain any number of the
' *same* operator; mixing && and || in one condition is a compile-time
' error -- split into nested if statements instead.

' ---- Guard clause: only check an array element when the index is valid ----

' n% -- value to test

DIM scores%(5)
scores%(0) = 10
scores%(1) = -5
scores%(2) = 30

' Long way: nested IF, so isPositive%() is only called when ptr% is valid.
PRINT "Long way (nested if), ptr% = -1:"
ptr% = -1
IF (ptr% >= 0) = 0 THEN GOTO 30
    ispositive_n_0% = scores%(ptr%)
    GOSUB 170
    IF (ispositive_result_0% > 0) = 0 THEN GOTO 10
        PRINT "  safe to read, value is positive"
        GOTO 20
10 PRINT "  value is not positive"
20 REM END IF
    GOTO 40
30 PRINT "  ptr% is out of range"
40 REM END IF

' Short way: && short-circuits -- same safety, one line, one IF. Watch for
' "(checking element)" in the output below: it does NOT print here, proving
' isPositive%() was never called for an out-of-range ptr%.
PRINT "Short way (&&), ptr% = -1:"
IF (ptr% >= 0) = 0 THEN GOTO 50
ispositive_n_0% = scores%(ptr%)
GOSUB 170
IF (ispositive_result_0% > 0) = 0 THEN GOTO 50
    PRINT "  safe to read, value is positive"
    GOTO 60
50 PRINT "  ptr% is out of range or value is not positive"
60 REM END IF

' Same short form, this time with a valid, positive element -- now
' "(checking element)" DOES print, since ptr% >= 0 no longer stops it early.
PRINT "Short way (&&), ptr% = 2:"
ptr% = 2
IF (ptr% >= 0) = 0 THEN GOTO 70
ispositive_n_0% = scores%(ptr%)
GOSUB 170
IF (ispositive_result_0% > 0) = 0 THEN GOTO 70
    PRINT "  safe to read, value is positive"
    GOTO 80
70 PRINT "  ptr% is out of range or value is not positive"
80 REM END IF

' ---- Retry loop: stop as soon as we succeed, or once out of attempts ----

' Long way: a bare DO with a separate exit for each stopping condition.
PRINT "Long way (nested checks), retry loop:"
attempts% = 0
maxattempts% = 3
succeeded% = 0
90 attempts% = attempts% + 1
    PRINT "  attempt "; attempts%
    IF (attempts% = 2) = 0 THEN GOTO 100
        succeeded% = 1
100 REM END IF
    IF (succeeded% <> 0) = 0 THEN GOTO 110
        GOTO 130
110 REM END IF
    IF (attempts% >= maxattempts%) = 0 THEN GOTO 120
        GOTO 130
120 REM END IF
    GOTO 90
130 REM END DO
PRINT "  stopped after "; attempts%; " attempt(s), succeeded% = "; succeeded%

' Short way: || short-circuits, so both stopping conditions live in the
' loop's own until-clause -- no scattered exit checks needed.
PRINT "Short way (||), retry loop:"
attempts% = 0
succeeded% = 0
140 IF (succeeded% <> 0) <> 0 THEN GOTO 160
IF (attempts% >= maxattempts%) <> 0 THEN GOTO 160
    attempts% = attempts% + 1
    PRINT "  attempt "; attempts%
    IF (attempts% = 2) = 0 THEN GOTO 150
        succeeded% = 1
150 REM END IF
    GOTO 140
160 REM END DO
PRINT "  stopped after "; attempts%; " attempt(s), succeeded% = "; succeeded%

END

' function ispositive%(n%)
170 ' A visible side effect, so the tutorial's own output proves whether
    ' this actually got called.
    PRINT "  (checking element)"
    ispositive_result_0% = ispositive_n_0%
    RETURN
' end function ispositive%
