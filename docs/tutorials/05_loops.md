[Home](../../) / [Tutorials](../) / Loops

<div class="prose" markdown="1">

BASCAL has three loop constructs: counted `for ... to [step n] ... end for`, pre-check `while ... end while` (classic BASIC's own `wend` works too), and `do`, which comes in two forms — `do [while/until cond] ... end do` tests the condition at the top (may run zero times), and `do ... loop [while/until cond]` tests it at the bottom (always runs at least once — BASCAL's direct `repeat`/`until` equivalent). All three loops share one early-exit statement, unqualified `exit` — not `exit for`/ `exit while`/`exit do` — the transpiler already knows which loop it's inside. See the [control-flow comparison](../../#control-flow) on the homepage for what each one transpiles down to.

</div>

<div class="snippet" markdown="1">

### for / next with an early exit

```bascal
for i% = 1 to 20
    if i% > 4 and (i% / 2) * 2 = i% then
        print "  "; i%
        exit
    end if
end for
```

</div>

<div class="snippet" markdown="1">

### DO WHILE and DO UNTIL

Two spellings of the same pre-check loop shape.

```bascal
do while k% <= 3
    print "  "; k%
    k% = k% + 1
end do

do until k% > 3
    print "  "; k%
    k% = k% + 1
end do
```

</div>

<div class="snippet" markdown="1">

### DO ... LOOP UNTIL — post-check, body runs at least once

The condition is tested after the body, not before, so this prints once even though k% starts already past the stopping value.

```bascal
k% = 99
do
    print "  "; k%
    k% = k% + 1
loop until k% > 3
```

</div>



[← Conditions](04_conditions.md)  ·  [Select Case →](06_select_case.md)


<!-- BEGIN generated tutorial source -->

### `tutorial/05_loops.bcl`

```bascal

// Tutorial — Loops: for, WHILE, DO
//
// BASCAL provides three loop constructs:
//
//   for var = start to end [STEP n] ... for END  (or bare END)
//     Counted loop.  STEP defaults to 1; use negative STEP to count down.
//
//   WHILE condition ... WHILE END  (or bare END)
//     Condition tested before each iteration.
//
//   DO [WHILE/UNTIL cond] ... END DO  (or bare END)
//     Pre-check: condition tested at the top, before the body runs at all.
//   DO ... LOOP [WHILE/UNTIL cond]
//     Post-check: condition tested at the bottom, so the body always runs
//     at least once.
//
// All three loops share one early-exit statement: exit. It's unqualified --
// no "exit for"/"exit while"/"exit do" -- the compiler already knows which
// loop it's inside from context.
program loops

/* --- for / NEXT --- */
print "Squares 1..5:"
for i% = 1 to 5
    print "  "; i%; "^2 = "; i% * i%
end for

/* Negative STEP — count down */
print "Countdown:"
for n% = 3 to 1 step -1
    print "  "; n%
end for
print "  Go!"

/* exit — stop early */
print "First even > 4:"
for i% = 1 to 20
    if i% > 4 and (i% / 2) * 2 = i% then
        print "  "; i%
        exit
    end if
end for

/* --- WHILE / WEND --- */
print "Powers of 2 under 100:"
p% = 1
while p% < 100
    print "  "; p%
    p% = p% * 2
wend

/* exit from a WHILE loop */
print "Collatz from 27 (first 8 steps):"
n% = 27
steps% = 0
while n% <> 1
    if steps% = 8 then
        print "  ..."
        exit
    end if
    if (n% / 2) * 2 = n% then  // even
        n% = n% / 2
    else
        n% = n% * 3 + 1
    end if
    steps% = steps% + 1
    print "  "; n%
end while

/* --- DO / LOOP variants --- */

// DO WHILE — test before body
print "DO WHILE:"
k% = 1
do while k% <= 3
    print "  "; k%
    k% = k% + 1
end do

// DO UNTIL — enter while condition is false
print "DO UNTIL:"
k% = 1
do until k% > 3
    print "  "; k%
    k% = k% + 1
end do

// DO ... LOOP UNTIL — post-check, body runs at least once
print "DO...LOOP UNTIL (body runs once even though already false):"
k% = 99
do
    print "  "; k%
    k% = k% + 1
loop until k% > 3

// exit from the middle of a DO loop
print "exit at k% = 3:"
k% = 1
do
    if k% = 3 then
        exit
    end if
    print "  "; k%
    k% = k% + 1
end do

end

```

### `tutorial/05_loops.bas`

```basic

10 ' BASCAL generated BASIC -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Tutorial — Loops: for, WHILE, DO
40 '
50 ' BASCAL provides three loop constructs:
60 '
70 ' for var = start to end [STEP n] ... for END  (or bare END)
80 ' Counted loop.  STEP defaults to 1; use negative STEP to count down.
90 '
100 ' WHILE condition ... WHILE END  (or bare END)
110 ' Condition tested before each iteration.
120 '
130 ' DO [WHILE/UNTIL cond] ... END DO  (or bare END)
140 ' Pre-check: condition tested at the top, before the body runs at all.
150 ' DO ... LOOP [WHILE/UNTIL cond]
160 ' Post-check: condition tested at the bottom, so the body always runs
170 ' at least once.
180 '
190 ' All three loops share one early-exit statement: exit. It's unqualified --
200 ' no "exit for"/"exit while"/"exit do" -- the compiler already knows which
210 ' loop it's inside from context.

220 ' --- for / NEXT ---
230 PRINT "Squares 1..5:"
240 FOR i% = 1 TO 5
250     PRINT "  "; i%; "^2 = "; i% * i%
260 NEXT i%

270 ' Negative STEP — count down
280 PRINT "Countdown:"
290 FOR n% = 3 TO 1 STEP -1
300     PRINT "  "; n%
310 NEXT n%
320 PRINT "  Go!"

330 ' exit — stop early
340 PRINT "First even > 4:"
350 FOR i% = 1 TO 20
360     IF ((i% > 4) AND (((i% / 2) * 2) = i%)) = 0 THEN GOTO 390
370         PRINT "  "; i%
380         EXIT FOR
390     REM END IF
400 NEXT i%

410 ' --- WHILE / WEND ---
420 PRINT "Powers of 2 under 100:"
430 p% = 1
440 IF (p% < 100) = 0 THEN GOTO 480
450     PRINT "  "; p%
460     p% = p% * 2
470     GOTO 440
480 REM END WHILE

490 ' exit from a WHILE loop
500 PRINT "Collatz from 27 (first 8 steps):"
510 n% = 27
520 steps% = 0
530 IF (n% <> 1) = 0 THEN GOTO 660
540     IF (steps% = 8) = 0 THEN GOTO 570
550         PRINT "  ..."
560         GOTO 660
570     REM END IF
580     IF (((n% / 2) * 2) = n%) = 0 THEN GOTO 610
590         n% = n% / 2
600         GOTO 620
610         n% = (n% * 3) + 1
620     REM END IF
630     steps% = steps% + 1
640     PRINT "  "; n%
650     GOTO 530
660 REM END WHILE

670 ' --- DO / LOOP variants ---

680 ' DO WHILE — test before body
690 PRINT "DO WHILE:"
700 k% = 1
710 IF (k% <= 3) = 0 THEN GOTO 750
720     PRINT "  "; k%
730     k% = k% + 1
740     GOTO 710
750 REM END DO

760 ' DO UNTIL — enter while condition is false
770 PRINT "DO UNTIL:"
780 k% = 1
790 IF (k% > 3) <> 0 THEN GOTO 830
800     PRINT "  "; k%
810     k% = k% + 1
820     GOTO 790
830 REM END DO

840 ' DO ... LOOP UNTIL — post-check, body runs at least once
850 PRINT "DO...LOOP UNTIL (body runs once even though already false):"
860 k% = 99
870     PRINT "  "; k%
880     k% = k% + 1
890     IF (k% > 3) = 0 THEN GOTO 870
900 REM END DO

910 ' exit from the middle of a DO loop
920 PRINT "exit at k% = 3:"
930 k% = 1
940     IF (k% = 3) = 0 THEN GOTO 960
950         GOTO 1000
960     REM END IF
970     PRINT "  "; k%
980     k% = k% + 1
990     GOTO 940
1000 REM END DO

1010 END

```

### `tutorial/05_loops.c`

```c

// BASCAL generated C -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
#include <stdio.h>
#include <math.h>

static int bv_i_i = 0;
static int bv_i_k = 0;
static int bv_i_n = 0;
static int bv_i_p = 0;
static int bv_i_steps = 0;

int main(void) {
    // Tutorial — Loops: for, WHILE, DO
    //
    // BASCAL provides three loop constructs:
    //
    // for var = start to end [STEP n] ... for END  (or bare END)
    // Counted loop.  STEP defaults to 1; use negative STEP to count down.
    //
    // WHILE condition ... WHILE END  (or bare END)
    // Condition tested before each iteration.
    //
    // DO [WHILE/UNTIL cond] ... END DO  (or bare END)
    // Pre-check: condition tested at the top, before the body runs at all.
    // DO ... LOOP [WHILE/UNTIL cond]
    // Post-check: condition tested at the bottom, so the body always runs
    // at least once.
    //
    // All three loops share one early-exit statement: exit. It's unqualified --
    // no "exit for"/"exit while"/"exit do" -- the compiler already knows which
    // loop it's inside from context.

    // --- for / NEXT ---
    printf("Squares 1..5:\n");
    int bt_lim_0 = 5;
    int bt_step_0 = 1;
    for (bv_i_i = 1; bt_step_0 >= 0 ? bv_i_i <= bt_lim_0 : bv_i_i >= bt_lim_0; bv_i_i += bt_step_0) {
        printf("  %d^2 = %d\n", bv_i_i, (bv_i_i * bv_i_i));
    }

    // Negative STEP — count down
    printf("Countdown:\n");
    int bt_lim_1 = 1;
    int bt_step_1 = -(1);
    for (bv_i_n = 3; bt_step_1 >= 0 ? bv_i_n <= bt_lim_1 : bv_i_n >= bt_lim_1; bv_i_n += bt_step_1) {
        printf("  %d\n", bv_i_n);
    }
    printf("  Go!\n");

    // exit — stop early
    printf("First even > 4:\n");
    int bt_lim_2 = 20;
    int bt_step_2 = 1;
    for (bv_i_i = 1; bt_step_2 >= 0 ? bv_i_i <= bt_lim_2 : bv_i_i >= bt_lim_2; bv_i_i += bt_step_2) {
        if (((int)((long)round((double)(-(bv_i_i > 4))) & (long)round((double)(-((((double)bv_i_i / (double)2) * 2) == bv_i_i)))))) {
            printf("  %d\n", bv_i_i);
            break;
        }
    }

    // --- WHILE / WEND ---
    printf("Powers of 2 under 100:\n");
    bv_i_p = 1;
    while ((-(bv_i_p < 100))) {
        printf("  %d\n", bv_i_p);
        bv_i_p = (bv_i_p * 2);
    }

    // exit from a WHILE loop
    printf("Collatz from 27 (first 8 steps):\n");
    bv_i_n = 27;
    bv_i_steps = 0;
    while ((-(bv_i_n != 1))) {
        if ((-(bv_i_steps == 8))) {
            printf("  ...\n");
            break;
        }
        if ((-((((double)bv_i_n / (double)2) * 2) == bv_i_n))) {
            bv_i_n = ((int)round((double)(((double)bv_i_n / (double)2))));
        } else {
            bv_i_n = ((bv_i_n * 3) + 1);
        }
        bv_i_steps = (bv_i_steps + 1);
        printf("  %d\n", bv_i_n);
    }

    // --- DO / LOOP variants ---

    // DO WHILE — test before body
    printf("DO WHILE:\n");
    bv_i_k = 1;
    while (1) {
        if (!((-(bv_i_k <= 3)))) break;
        printf("  %d\n", bv_i_k);
        bv_i_k = (bv_i_k + 1);
    }

    // DO UNTIL — enter while condition is false
    printf("DO UNTIL:\n");
    bv_i_k = 1;
    while (1) {
        if ((-(bv_i_k > 3))) break;
        printf("  %d\n", bv_i_k);
        bv_i_k = (bv_i_k + 1);
    }

    // DO ... LOOP UNTIL — post-check, body runs at least once
    printf("DO...LOOP UNTIL (body runs once even though already false):\n");
    bv_i_k = 99;
    while (1) {
        printf("  %d\n", bv_i_k);
        bv_i_k = (bv_i_k + 1);
        if ((-(bv_i_k > 3))) break;
    }

    // exit from the middle of a DO loop
    printf("exit at k%% = 3:\n");
    bv_i_k = 1;
    while (1) {
        if ((-(bv_i_k == 3))) {
            break;
        }
        printf("  %d\n", bv_i_k);
        bv_i_k = (bv_i_k + 1);
    }

    return 0;
}

```

<!-- END generated tutorial source -->
