#include <stdio.h>
#include <string.h>

int main(void) {
    int bv_i_choice = 0;
    int bv_i_score = 0;
    int bv_i_temp = 0;
    char bv_s_day[256] = {0};

    // Tutorial 6 — SELECT CASE
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
        if ((bt_sel_0 == 100)) {
            printf("Perfect!\n");
        } else if ((bt_sel_0 >= 90 && bt_sel_0 <= 99)) {
            printf("A  — Excellent\n");
        } else if ((bt_sel_0 >= 80 && bt_sel_0 <= 89)) {
            printf("B  — Good\n");
        } else if ((bt_sel_0 >= 70 && bt_sel_0 <= 79)) {
            printf("C  — Satisfactory\n");
        } else if ((bt_sel_0 >= 60 && bt_sel_0 <= 69)) {
            printf("D  — Passing\n");
        } else if ((bt_sel_0 >= 0)) {
            printf("F  — Fail\n");
        } else {
            printf("Invalid score\n");
        }
    }

    // String select: day-of-week classification
    snprintf(bv_s_day, sizeof(bv_s_day), "%s", "Saturday");

    {
        char bt_sel_1[256];
        snprintf(bt_sel_1, sizeof(bt_sel_1), "%s", bv_s_day);
        if ((strcmp(bt_sel_1, "Monday") == 0) || (strcmp(bt_sel_1, "Tuesday") == 0) || (strcmp(bt_sel_1, "Wednesday") == 0) || (strcmp(bt_sel_1, "Thursday") == 0) || (strcmp(bt_sel_1, "Friday") == 0)) {
            char bt_s_2[256];
            snprintf(bt_s_2, sizeof(bt_s_2), "%s%s", bv_s_day, " is a weekday");
            printf("%s\n", bt_s_2);
        } else if ((strcmp(bt_sel_1, "Saturday") == 0) || (strcmp(bt_sel_1, "Sunday") == 0)) {
            char bt_s_3[256];
            snprintf(bt_s_3, sizeof(bt_s_3), "%s%s", bv_s_day, " is a weekend");
            printf("%s\n", bt_s_3);
        } else {
            char bt_s_4[256];
            snprintf(bt_s_4, sizeof(bt_s_4), "%s%s", "Unknown day: ", bv_s_day);
            printf("%s\n", bt_s_4);
        }
    }

    // IS comparisons on temperature
    bv_i_temp = -(3);

    {
        int bt_sel_5 = bv_i_temp;
        if ((bt_sel_5 < 0)) {
            printf("Below freezing (%d°)\n", bv_i_temp);
        } else if ((bt_sel_5 < 10)) {
            printf("Cold (%d°)\n", bv_i_temp);
        } else if ((bt_sel_5 < 20)) {
            printf("Cool (%d°)\n", bv_i_temp);
        } else if ((bt_sel_5 < 30)) {
            printf("Warm (%d°)\n", bv_i_temp);
        } else {
            printf("Hot (%d°)\n", bv_i_temp);
        }
    }

    // Multi-value list on a menu choice
    bv_i_choice = 2;

    {
        int bt_sel_6 = bv_i_choice;
        if ((bt_sel_6 == 1)) {
            printf("New game\n");
        } else if ((bt_sel_6 == 2) || (bt_sel_6 == 3)) {
            printf("Load game\n");
        } else if ((bt_sel_6 == 4)) {
            printf("Options\n");
        } else {
            printf("Quit\n");
        }
    }

    return 0;
}
