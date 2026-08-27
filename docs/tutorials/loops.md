[Home](../../) / [Tutorials](../) / Loops

Generated sources: [BCL](https://github.com/johnjoeallen/bascal/blob/main/tutorial/loops.bcl), [BASIC](https://github.com/johnjoeallen/bascal/blob/main/tutorial/loops.bas), [C](https://github.com/johnjoeallen/bascal/blob/main/tutorial/loops.c), and [JVM assembly](https://github.com/johnjoeallen/bascal/blob/main/tutorial/loops.j).

<div class="prose" markdown="1">

BASCAL has three loop constructs: counted `for ... to [step n] ... end for`, pre-check `while ... end while` (classic BASIC's own `wend` works too), and `do`, which comes in two forms — `do [while/until cond] ... end do` tests the condition at the top (may run zero times), and `do ... loop [while/until cond]` tests it at the bottom (always runs at least once — BASCAL's direct `repeat`/`until` equivalent). All three loops share one early-exit statement, unqualified `exit` — not `exit for`/ `exit while`/`exit do` — the transpiler already knows which loop it's inside. See the [structured control-flow chapter](../home/control-flow.md) for what each one transpiles down to.

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



[← Conditions](conditions.md)  ·  [Select Case →](select_case.md)


<!-- BEGIN generated tutorial source -->

<details class="source-embed" markdown="1">

<summary><code>tutorial/loops.bcl</code></summary>



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



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/loops.bas</code></summary>



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



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/loops.c</code></summary>



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



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/loops.j</code></summary>



```basic

.version 50 0
.class public Loops
.super java/lang/Object

.method public static main : ([Ljava/lang/String;)V
    .limit stack 16
    .limit locals 6

    iconst_0
    istore 1
    iconst_0
    istore 2
    iconst_0
    istore 3
    iconst_0
    istore 4
    iconst_0
    istore 5
    ; Tutorial — Loops: for, WHILE, DO
    ;
    ; BASCAL provides three loop constructs:
    ;
    ; for var = start to end [STEP n] ... for END  (or bare END)
    ; Counted loop.  STEP defaults to 1; use negative STEP to count down.
    ;
    ; WHILE condition ... WHILE END  (or bare END)
    ; Condition tested before each iteration.
    ;
    ; DO [WHILE/UNTIL cond] ... END DO  (or bare END)
    ; Pre-check: condition tested at the top, before the body runs at all.
    ; DO ... LOOP [WHILE/UNTIL cond]
    ; Post-check: condition tested at the bottom, so the body always runs
    ; at least once.
    ;
    ; All three loops share one early-exit statement: exit. It's unqualified --
    ; no "exit for"/"exit while"/"exit do" -- the compiler already knows which
    ; loop it's inside from context.

    ; --- for / NEXT ---
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Squares 1..5:"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ldc 1
    istore 1
L_for_0_top:
    iload 1
    ldc 5
    if_icmpgt L_for_0_end
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 1
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "^2 = "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 1
    iload 1
    imul
    invokevirtual java/io/PrintStream/println (I)V
    iload 1
    ldc 1
    iadd
    istore 1
    goto L_for_0_top
L_for_0_end:

    ; Negative STEP — count down
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Countdown:"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ldc 3
    istore 3
L_for_1_top:
    iload 3
    ldc 1
    if_icmplt L_for_1_end
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 3
    invokevirtual java/io/PrintStream/println (I)V
    iload 3
    ldc 1
    ineg
    iadd
    istore 3
    goto L_for_1_top
L_for_1_end:
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  Go!"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V

    ; exit — stop early
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "First even > 4:"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ldc 1
    istore 1
L_for_2_top:
    iload 1
    ldc 20
    if_icmpgt L_for_2_end
    iload 1
    ldc 4
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    i2d
    dup2
    ldc2_w 0.5
    dup2_x2
    pop2
    invokestatic java/lang/Math/copySign (DD)D
    dadd
    d2l
    iload 1
    i2d
    ldc 2
    i2d
    ddiv
    ldc 2
    i2d
    dmul
    iload 1
    i2d
    invokestatic java/lang/Double/compare (DD)I
    dup
    ineg
    ior
    bipush 31
    iushr
    iconst_1
    ixor
    ineg
    i2d
    dup2
    ldc2_w 0.5
    dup2_x2
    pop2
    invokestatic java/lang/Math/copySign (DD)D
    dadd
    d2l
    land
    lconst_0
    lcmp
    ifeq L_if_3_else
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 1
    invokevirtual java/io/PrintStream/println (I)V
    goto L_for_2_end
L_if_3_else:
    iload 1
    ldc 1
    iadd
    istore 1
    goto L_for_2_top
L_for_2_end:

    ; --- WHILE / WEND ---
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Powers of 2 under 100:"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ldc 1
    istore 4
L_while_4_top:
    iload 4
    ldc 100
    invokestatic java/lang/Integer/compare (II)I
    bipush 31
    ishr
    ifeq L_while_4_end
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 4
    invokevirtual java/io/PrintStream/println (I)V
    iload 4
    ldc 2
    imul
    istore 4
    goto L_while_4_top
L_while_4_end:

    ; exit from a WHILE loop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Collatz from 27 (first 8 steps):"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ldc 27
    istore 3
    ldc 0
    istore 5
L_while_5_top:
    iload 3
    ldc 1
    invokestatic java/lang/Integer/compare (II)I
    dup
    ineg
    ior
    bipush 31
    iushr
    ineg
    ifeq L_while_5_end
    iload 5
    ldc 8
    invokestatic java/lang/Integer/compare (II)I
    dup
    ineg
    ior
    bipush 31
    iushr
    iconst_1
    ixor
    ineg
    ifeq L_if_6_else
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  ..."
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_while_5_end
L_if_6_else:
    iload 3
    i2d
    ldc 2
    i2d
    ddiv
    ldc 2
    i2d
    dmul
    iload 3
    i2d
    invokestatic java/lang/Double/compare (DD)I
    dup
    ineg
    ior
    bipush 31
    iushr
    iconst_1
    ixor
    ineg
    ifeq L_if_7_else
    iload 3
    i2d
    ldc 2
    i2d
    ddiv
    dup2
    ldc2_w 0.5
    dup2_x2
    pop2
    invokestatic java/lang/Math/copySign (DD)D
    dadd
    d2l
    l2i
    istore 3
    goto L_if_7_end
L_if_7_else:
    iload 3
    ldc 3
    imul
    ldc 1
    iadd
    istore 3
L_if_7_end:
    iload 5
    ldc 1
    iadd
    istore 5
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 3
    invokevirtual java/io/PrintStream/println (I)V
    goto L_while_5_top
L_while_5_end:

    ; --- DO / LOOP variants ---

    ; DO WHILE — test before body
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "DO WHILE:"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ldc 1
    istore 2
L_do_8_top:
    iload 2
    ldc 3
    invokestatic java/lang/Integer/compare (II)I
    iconst_1
    isub
    bipush 31
    ishr
    ifeq L_do_8_end
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 2
    invokevirtual java/io/PrintStream/println (I)V
    iload 2
    ldc 1
    iadd
    istore 2
    goto L_do_8_top
L_do_8_end:

    ; DO UNTIL — enter while condition is false
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "DO UNTIL:"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ldc 1
    istore 2
L_do_9_top:
    iload 2
    ldc 3
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifne L_do_9_end
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 2
    invokevirtual java/io/PrintStream/println (I)V
    iload 2
    ldc 1
    iadd
    istore 2
    goto L_do_9_top
L_do_9_end:

    ; DO ... LOOP UNTIL — post-check, body runs at least once
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "DO...LOOP UNTIL (body runs once even though already false):"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ldc 99
    istore 2
L_do_10_top:
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 2
    invokevirtual java/io/PrintStream/println (I)V
    iload 2
    ldc 1
    iadd
    istore 2
    iload 2
    ldc 3
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifne L_do_10_end
    goto L_do_10_top
L_do_10_end:

    ; exit from the middle of a DO loop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "exit at k% = 3:"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ldc 1
    istore 2
L_do_11_top:
    iload 2
    ldc 3
    invokestatic java/lang/Integer/compare (II)I
    dup
    ineg
    ior
    bipush 31
    iushr
    iconst_1
    ixor
    ineg
    ifeq L_if_12_else
    goto L_do_11_end
L_if_12_else:
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 2
    invokevirtual java/io/PrintStream/println (I)V
    iload 2
    ldc 1
    iadd
    istore 2
    goto L_do_11_top
L_do_11_end:

    return
.end method

```



</details>

<!-- END generated tutorial source -->
