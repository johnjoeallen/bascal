// BASCAL generated C -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
#include <stdio.h>
#include <string.h>

static int bv_i_choice = 0;
static int bv_i_score = 0;
static int bv_i_temp = 0;
static char bv_s_day[256] = {0};

int main(void) {
    // Tutorial — SELECT CASE
    //
    // SELECT CASE tests one expression against multiple patterns.  The
    // compiler evaluates the expression once, stores it in a temporary
    // variable, and emits an IF/goto dispatch chain.
    //
    // Pattern forms:
    // case value               — exact match
    // case v1, v2, v3          — any of the listed values
    // case low to high         — inclusive range
    // case is <op> value       — comparison (=  <>  <  <=  >  >=)
    // case else                — default; must be the last clause

    // Integer select: convert numeric score to letter grade
    bv_i_score = 85;

    {
        int bt_sel_0 = bv_i_score;
        int bt_sel_match_1 = 0;
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == 100)) {
                bt_sel_match_1 = 1;
                printf("Perfect!\n");
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 >= 90 && bt_sel_0 <= 99)) {
                bt_sel_match_1 = 1;
                printf("A  — Excellent\n");
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 >= 80 && bt_sel_0 <= 89)) {
                bt_sel_match_1 = 1;
                printf("B  — Good\n");
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 >= 70 && bt_sel_0 <= 79)) {
                bt_sel_match_1 = 1;
                printf("C  — Satisfactory\n");
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 >= 60 && bt_sel_0 <= 69)) {
                bt_sel_match_1 = 1;
                printf("D  — Passing\n");
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 >= 0)) {
                bt_sel_match_1 = 1;
                printf("F  — Fail\n");
            }
        }
        if (!bt_sel_match_1) {
            printf("Invalid score\n");
        }
    }

    // String select: day-of-week classification
    snprintf(bv_s_day, sizeof(bv_s_day), "%s", "Saturday");

    {
        char bt_sel_2[256];
        snprintf(bt_sel_2, sizeof(bt_sel_2), "%s", bv_s_day);
        int bt_sel_match_3 = 0;
        if (!bt_sel_match_3) {
            if ((strcmp(bt_sel_2, "Monday") == 0) || (strcmp(bt_sel_2, "Tuesday") == 0) || (strcmp(bt_sel_2, "Wednesday") == 0) || (strcmp(bt_sel_2, "Thursday") == 0) || (strcmp(bt_sel_2, "Friday") == 0)) {
                bt_sel_match_3 = 1;
                char bt_s_4[256];
                snprintf(bt_s_4, sizeof(bt_s_4), "%s%s", bv_s_day, " is a weekday");
                printf("%s\n", bt_s_4);
            }
        }
        if (!bt_sel_match_3) {
            if ((strcmp(bt_sel_2, "Saturday") == 0) || (strcmp(bt_sel_2, "Sunday") == 0)) {
                bt_sel_match_3 = 1;
                char bt_s_5[256];
                snprintf(bt_s_5, sizeof(bt_s_5), "%s%s", bv_s_day, " is a weekend");
                printf("%s\n", bt_s_5);
            }
        }
        if (!bt_sel_match_3) {
            char bt_s_6[256];
            snprintf(bt_s_6, sizeof(bt_s_6), "%s%s", "Unknown day: ", bv_s_day);
            printf("%s\n", bt_s_6);
        }
    }

    // IS comparisons on temperature
    bv_i_temp = -(3);

    {
        int bt_sel_7 = bv_i_temp;
        int bt_sel_match_8 = 0;
        if (!bt_sel_match_8) {
            if ((bt_sel_7 < 0)) {
                bt_sel_match_8 = 1;
                printf("Below freezing (%d°)\n", bv_i_temp);
            }
        }
        if (!bt_sel_match_8) {
            if ((bt_sel_7 < 10)) {
                bt_sel_match_8 = 1;
                printf("Cold (%d°)\n", bv_i_temp);
            }
        }
        if (!bt_sel_match_8) {
            if ((bt_sel_7 < 20)) {
                bt_sel_match_8 = 1;
                printf("Cool (%d°)\n", bv_i_temp);
            }
        }
        if (!bt_sel_match_8) {
            if ((bt_sel_7 < 30)) {
                bt_sel_match_8 = 1;
                printf("Warm (%d°)\n", bv_i_temp);
            }
        }
        if (!bt_sel_match_8) {
            printf("Hot (%d°)\n", bv_i_temp);
        }
    }

    // Multi-value list on a menu choice
    bv_i_choice = 2;

    {
        int bt_sel_9 = bv_i_choice;
        int bt_sel_match_10 = 0;
        if (!bt_sel_match_10) {
            if ((bt_sel_9 == 1)) {
                bt_sel_match_10 = 1;
                printf("New game\n");
            }
        }
        if (!bt_sel_match_10) {
            if ((bt_sel_9 == 2) || (bt_sel_9 == 3)) {
                bt_sel_match_10 = 1;
                printf("Load game\n");
            }
        }
        if (!bt_sel_match_10) {
            if ((bt_sel_9 == 4)) {
                bt_sel_match_10 = 1;
                printf("Options\n");
            }
        }
        if (!bt_sel_match_10) {
            printf("Quit\n");
        }
    }

    return 0;
}
