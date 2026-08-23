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
