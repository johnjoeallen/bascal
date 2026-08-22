#include <stdio.h>
#include <math.h>

static int bv_i_i = 0;
static int bv_i_k = 0;
static int bv_i_n = 0;
static int bv_i_p = 0;
static int bv_i_steps = 0;

int main(void) {
    // Tutorial 5 — Loops: for, WHILE, DO
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
