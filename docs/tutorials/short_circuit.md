[Home](../../) / [Tutorials](../) / Short-Circuit && and \|\|

<div class="prose" markdown="1">

Classic BASIC's `AND`/`OR` are bitwise and always evaluate both sides — there's no short-circuit primitive in the generated BASIC at all. `&&`/`||` give BASCAL real short-circuit evaluation instead: the second operand is only evaluated once the first one hasn't already decided the answer. They're only usable directly in the condition of `if`/`elseif`/ `while`/`do` — not as a general expression. A condition may chain any number of the *same* operator; mixing `&&` and `||` in one condition is a transpile-time error — split into nested `if` statements instead. See the [structured control-flow chapter](../home/control-flow.md) for the exact generated `IF`/`GOTO` shape.

</div>

<div class="snippet" markdown="1">

### Guard clause -- the second operand is never evaluated when the first fails

```bascal
if ptr% >= 0 && isPositive%(scores%(ptr%)) > 0 then
    print "safe to read"
end if
```

</div>

<div class="snippet" markdown="1">

### Retry loop -- both stopping conditions live in the loop's own until-clause

```bascal
do until succeeded% <> 0 || attempts% >= maxAttempts%
    attempts% = attempts% + 1
    ...
end do
```

</div>



[← Random-Access and Record Files](random_and_record_files.md)  ·  [Classic BASIC Error Handling →](labels_and_error_handling.md)


<!-- BEGIN generated tutorial source -->

<details class="source-embed" markdown="1">

<summary><code>tutorial/short_circuit.bcl</code></summary>



```bascal

// Tutorial — Short-Circuit && and ||
//
// Classic BASIC's AND/OR are bitwise and always evaluate both sides -- there
// is no short-circuit primitive in the generated BASIC at all. && and ||
// give BASCAL real short-circuit evaluation instead: the second operand is
// only evaluated once the first one hasn't already decided the answer.
//
// a && b && c ...   -- true only if every operand is true; stops at the
//                      first false operand.
// a || b || c ...   -- true if any operand is true; stops at the first
//                      true operand.
//
// && / || are only usable directly in the condition of if / elseif / while
// / do -- not as a general expression (can't be assigned to a variable or
// passed as a function argument). A condition may chain any number of the
// *same* operator; mixing && and || in one condition is a compile-time
// error -- split into nested if statements instead.
program shortCircuit

/* ---- Guard clause: only check an array element when the index is valid ---- */

// n% -- value to test
function isPositive%(n%)
    // A visible side effect, so the tutorial's own output proves whether
    // this actually got called.
    print "  (checking element)"
    return n%
end function

dim scores%(5)
scores%(0) = 10
scores%(1) = -5
scores%(2) = 30

// Long way: nested IF, so isPositive%() is only called when ptr% is valid.
print "Long way (nested if), ptr% = -1:"
ptr% = -1
if ptr% >= 0 then
    if isPositive%(scores%(ptr%)) > 0 then
        print "  safe to read, value is positive"
    else
        print "  value is not positive"
    end if
else
    print "  ptr% is out of range"
end if

// Short way: && short-circuits -- same safety, one line, one IF. Watch for
// "(checking element)" in the output below: it does NOT print here, proving
// isPositive%() was never called for an out-of-range ptr%.
print "Short way (&&), ptr% = -1:"
if ptr% >= 0 && isPositive%(scores%(ptr%)) > 0 then
    print "  safe to read, value is positive"
else
    print "  ptr% is out of range or value is not positive"
end if

// Same short form, this time with a valid, positive element -- now
// "(checking element)" DOES print, since ptr% >= 0 no longer stops it early.
print "Short way (&&), ptr% = 2:"
ptr% = 2
if ptr% >= 0 && isPositive%(scores%(ptr%)) > 0 then
    print "  safe to read, value is positive"
else
    print "  ptr% is out of range or value is not positive"
end if

/* ---- Retry loop: stop as soon as we succeed, or once out of attempts ---- */

// Long way: a bare DO with a separate exit for each stopping condition.
print "Long way (nested checks), retry loop:"
attempts% = 0
maxAttempts% = 3
succeeded% = 0
do
    attempts% = attempts% + 1
    print "  attempt "; attempts%
    if attempts% = 2 then
        succeeded% = 1
    end if
    if succeeded% <> 0 then
        exit
    end if
    if attempts% >= maxAttempts% then
        exit
    end if
end do
print "  stopped after "; attempts%; " attempt(s), succeeded% = "; succeeded%

// Short way: || short-circuits, so both stopping conditions live in the
// loop's own until-clause -- no scattered exit checks needed.
print "Short way (||), retry loop:"
attempts% = 0
succeeded% = 0
do until succeeded% <> 0 || attempts% >= maxAttempts%
    attempts% = attempts% + 1
    print "  attempt "; attempts%
    if attempts% = 2 then
        succeeded% = 1
    end if
end do
print "  stopped after "; attempts%; " attempt(s), succeeded% = "; succeeded%

end

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/short_circuit.bas</code></summary>



```basic

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

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/short_circuit.c</code></summary>



```c

// BASCAL generated C -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
#include <stdio.h>

static int bv_i_attempts = 0;
static int bv_i_maxattempts = 0;
static int bv_i_ptr = 0;
static int bv_i_succeeded = 0;
static int bv_i_scores[6] = {0};

int bf_i_ispositive(int bv_i_n);

int bf_i_ispositive(int bv_i_n) {
    // A visible side effect, so the tutorial's own output proves whether
    // this actually got called.
    printf("  (checking element)\n");
    return bv_i_n;
}

int main(void) {
    // Tutorial — Short-Circuit && and ||
    //
    // Classic BASIC's AND/OR are bitwise and always evaluate both sides -- there
    // is no short-circuit primitive in the generated BASIC at all. && and ||
    // give BASCAL real short-circuit evaluation instead: the second operand is
    // only evaluated once the first one hasn't already decided the answer.
    //
    // a && b && c ...   -- true only if every operand is true; stops at the
    // first false operand.
    // a || b || c ...   -- true if any operand is true; stops at the first
    // true operand.
    //
    // && / || are only usable directly in the condition of if / elseif / while
    // / do -- not as a general expression (can't be assigned to a variable or
    // passed as a function argument). A condition may chain any number of the
    // *same* operator; mixing && and || in one condition is a compile-time
    // error -- split into nested if statements instead.

    // ---- Guard clause: only check an array element when the index is valid ----

    // n% -- value to test

    bv_i_scores[(0)] = 10;
    bv_i_scores[(1)] = -(5);
    bv_i_scores[(2)] = 30;

    // Long way: nested IF, so isPositive%() is only called when ptr% is valid.
    printf("Long way (nested if), ptr%% = -1:\n");
    bv_i_ptr = -(1);
    if ((-(bv_i_ptr >= 0))) {
        if ((-(bf_i_ispositive(bv_i_scores[(bv_i_ptr)]) > 0))) {
            printf("  safe to read, value is positive\n");
        } else {
            printf("  value is not positive\n");
        }
    } else {
        printf("  ptr%% is out of range\n");
    }

    // Short way: && short-circuits -- same safety, one line, one IF. Watch for
    // "(checking element)" in the output below: it does NOT print here, proving
    // isPositive%() was never called for an out-of-range ptr%.
    printf("Short way (&&), ptr%% = -1:\n");
    if ((-(((-(bv_i_ptr >= 0))) != 0 && ((-(bf_i_ispositive(bv_i_scores[(bv_i_ptr)]) > 0))) != 0))) {
        printf("  safe to read, value is positive\n");
    } else {
        printf("  ptr%% is out of range or value is not positive\n");
    }

    // Same short form, this time with a valid, positive element -- now
    // "(checking element)" DOES print, since ptr% >= 0 no longer stops it early.
    printf("Short way (&&), ptr%% = 2:\n");
    bv_i_ptr = 2;
    if ((-(((-(bv_i_ptr >= 0))) != 0 && ((-(bf_i_ispositive(bv_i_scores[(bv_i_ptr)]) > 0))) != 0))) {
        printf("  safe to read, value is positive\n");
    } else {
        printf("  ptr%% is out of range or value is not positive\n");
    }

    // ---- Retry loop: stop as soon as we succeed, or once out of attempts ----

    // Long way: a bare DO with a separate exit for each stopping condition.
    printf("Long way (nested checks), retry loop:\n");
    bv_i_attempts = 0;
    bv_i_maxattempts = 3;
    bv_i_succeeded = 0;
    while (1) {
        bv_i_attempts = (bv_i_attempts + 1);
        printf("  attempt %d\n", bv_i_attempts);
        if ((-(bv_i_attempts == 2))) {
            bv_i_succeeded = 1;
        }
        if ((-(bv_i_succeeded != 0))) {
            break;
        }
        if ((-(bv_i_attempts >= bv_i_maxattempts))) {
            break;
        }
    }
    printf("  stopped after %d attempt(s), succeeded%% = %d\n", bv_i_attempts, bv_i_succeeded);

    // Short way: || short-circuits, so both stopping conditions live in the
    // loop's own until-clause -- no scattered exit checks needed.
    printf("Short way (||), retry loop:\n");
    bv_i_attempts = 0;
    bv_i_succeeded = 0;
    while (1) {
        if ((-(((-(bv_i_succeeded != 0))) != 0 || ((-(bv_i_attempts >= bv_i_maxattempts))) != 0))) break;
        bv_i_attempts = (bv_i_attempts + 1);
        printf("  attempt %d\n", bv_i_attempts);
        if ((-(bv_i_attempts == 2))) {
            bv_i_succeeded = 1;
        }
    }
    printf("  stopped after %d attempt(s), succeeded%% = %d\n", bv_i_attempts, bv_i_succeeded);

    return 0;
}

```



</details>

<!-- END generated tutorial source -->
