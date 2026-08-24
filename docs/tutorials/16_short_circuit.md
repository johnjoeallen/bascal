[Home](../../) / [Tutorials](../) / Short-Circuit && and \|\|

<div class="prose" markdown="1">

Classic BASIC's `AND`/`OR` are bitwise and always evaluate both sides — there's no short-circuit primitive in the generated BASIC at all. `&&`/`||` give BASCAL real short-circuit evaluation instead: the second operand is only evaluated once the first one hasn't already decided the answer. They're only usable directly in the condition of `if`/`elseif`/ `while`/`do` — not as a general expression. A condition may chain any number of the *same* operator; mixing `&&` and `||` in one condition is a transpile-time error — split into nested `if` statements instead. See the [control-flow comparison](../../#control-flow) on the homepage for the exact generated `IF`/`GOTO` shape.

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



[← Random-Access and Record Files](15_random_and_record_files.md)  ·  [Labels and Error Handling →](17_labels_and_error_handling.md)


<!-- BEGIN generated tutorial source -->

<details class="source-embed" markdown="1">

<summary><code>tutorial/16_short_circuit.bcl</code></summary>



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

<summary><code>tutorial/16_short_circuit.bas</code></summary>



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

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/16_short_circuit.c</code></summary>



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
    if (((-(bv_i_ptr >= 0)) && (-(bf_i_ispositive(bv_i_scores[(bv_i_ptr)]) > 0)))) {
        printf("  safe to read, value is positive\n");
    } else {
        printf("  ptr%% is out of range or value is not positive\n");
    }

    // Same short form, this time with a valid, positive element -- now
    // "(checking element)" DOES print, since ptr% >= 0 no longer stops it early.
    printf("Short way (&&), ptr%% = 2:\n");
    bv_i_ptr = 2;
    if (((-(bv_i_ptr >= 0)) && (-(bf_i_ispositive(bv_i_scores[(bv_i_ptr)]) > 0)))) {
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
        if (((-(bv_i_succeeded != 0)) || (-(bv_i_attempts >= bv_i_maxattempts)))) break;
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
