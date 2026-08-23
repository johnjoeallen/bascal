#include <stdio.h>
#include <math.h>

static int bv_i_age = 0;
static int bv_i_income = 0;
static int bv_i_points = 0;
static int bv_i_score = 0;
static int bv_i_temperature = 0;
static int bv_i_x = 0;
static char bv_s_grade[256] = {0};

int main(void) {
    // Tutorial — Conditions: IF / ELSEIF / ELSE / END IF
    //
    // BASCAL supports multi-line block IF statements.  The compiler transpiles
    // them to numeric goto targets so the generated BASIC is compatible with
    // 1980s BASCOM.  You never write line numbers yourself.
    //
    // Forms:
    // if cond then ... end if
    // if cond then ... else ... end if
    // if cond then ... elseif cond then ... else ... end if
    // if cond then statement                   (single-line, no end if)
    // if cond then statement else statement     (single-line, no end if)
    //
    // A newline right after `then` selects the block form; a statement
    // directly after `then` on the same line selects the single-line form
    // instead -- that's the only difference. elseif isn't available
    // single-line, same as classic BASIC.

    // Simple IF
    bv_i_temperature = 23;
    if ((-(bv_i_temperature > 30))) {
        printf("Hot day\n");
    }

    // IF / ELSE
    bv_i_score = 72;
    if ((-(bv_i_score >= 60))) {
        printf("Pass (%d)\n", bv_i_score);
    } else {
        printf("Fail (%d)\n", bv_i_score);
    }

    // IF / ELSEIF / ELSE — grade classification
    bv_i_points = 85;

    if ((-(bv_i_points >= 90))) {
        snprintf(bv_s_grade, sizeof(bv_s_grade), "%s", "A");
    } else {
        if ((-(bv_i_points >= 80))) {
            snprintf(bv_s_grade, sizeof(bv_s_grade), "%s", "B");
        } else {
            if ((-(bv_i_points >= 70))) {
                snprintf(bv_s_grade, sizeof(bv_s_grade), "%s", "C");
            } else {
                if ((-(bv_i_points >= 60))) {
                    snprintf(bv_s_grade, sizeof(bv_s_grade), "%s", "D");
                } else {
                    snprintf(bv_s_grade, sizeof(bv_s_grade), "%s", "F");
                }
            }
        }
    }

    char bt_s_0[256];
    snprintf(bt_s_0, sizeof(bt_s_0), "%s%s", "Grade: ", bv_s_grade);
    printf("%s\n", bt_s_0);

    // Nested IF
    bv_i_x = 15;
    if ((-(bv_i_x > 0))) {
        if ((-(bv_i_x > 10))) {
            printf("%dis large and positive\n", bv_i_x);
        } else {
            printf("%dis small and positive\n", bv_i_x);
        }
    } else {
        printf("%dis not positive\n", bv_i_x);
    }

    // Single-line IF -- no end if needed
    bv_i_temperature = 23;
    if ((-(bv_i_temperature > 30))) {
        printf("Hot day (single-line)\n");
    }
    if ((-(bv_i_temperature > 100))) {
        printf("Scorching\n");
    } else {
        printf("Not scorching\n");
    }

    // Compound conditions
    bv_i_age = 25;
    bv_i_income = 45000;
    if (((int)((long)round((double)(-(bv_i_age >= 18))) & (long)round((double)(-(bv_i_income >= 30000)))))) {
        printf("Eligible\n");
    } else {
        printf("Not eligible\n");
    }

    return 0;
}
