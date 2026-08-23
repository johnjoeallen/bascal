#include <stdio.h>
#include <string.h>

#include "bcc_runtime.h"

static int bv_i_dummy = 0;
static int bv_i_lo = 0;
static int bv_i_runningtotal = 0;
static char bv_s_a[256] = {0};
static char bv_s_b[256] = {0};

void bf_s_ucase_s(const char* bv_s_self_in, char* bcc_out);
void bf_s_lcase_s(const char* bv_s_self_in, char* bcc_out);
int bf_i_max(int bv_i_a, int bv_i_b);
int bf_i_min(int bv_i_a, int bv_i_b);
int bf_i_clamp(int bv_i_value, int bv_i_lo, int bv_i_hi);
void bf_s_repeat(const char* bv_s_text_in, int bv_i_n, char* bcc_out);
void bf_s_titlecase(const char* bv_s_word_in, char* bcc_out);
int bf_i_sumto(int bv_i_n);
int bf_i_productto(int bv_i_n);
int bf_i_addtototal(int bv_i_x);

void bf_s_ucase_s(const char* bv_s_self_in, char* bcc_out) {
    char bv_s_self[256];
    snprintf(bv_s_self, sizeof(bv_s_self), "%s", bv_s_self_in);
    int bv_i_c = 0;
    int bv_i_i = 0;
    char bv_s_out[256] = {0};

    snprintf(bv_s_out, sizeof(bv_s_out), "%s", "");
    int bt_lim_0 = ((int)strlen(bv_s_self));
    int bt_step_0 = 1;
    for (bv_i_i = 1; bt_step_0 >= 0 ? bv_i_i <= bt_lim_0 : bv_i_i >= bt_lim_0; bv_i_i += bt_step_0) {
        bv_i_c = ((int)(unsigned char)bcc_mid(bv_s_self, bv_i_i, 1)[0]);
        if (((-(bv_i_c >= 97)) && (-(bv_i_c <= 122)))) {
            bv_i_c = (bv_i_c - 32);
        }
        char bt_s_1[256];
        snprintf(bt_s_1, sizeof(bt_s_1), "%s%s", bv_s_out, bcc_chr(bv_i_c));
        snprintf(bv_s_out, sizeof(bv_s_out), "%s", bt_s_1);
    }
    snprintf(bcc_out, 256, "%s", bv_s_out);
    return;
}

void bf_s_lcase_s(const char* bv_s_self_in, char* bcc_out) {
    char bv_s_self[256];
    snprintf(bv_s_self, sizeof(bv_s_self), "%s", bv_s_self_in);
    int bv_i_c = 0;
    int bv_i_i = 0;
    char bv_s_out[256] = {0};

    snprintf(bv_s_out, sizeof(bv_s_out), "%s", "");
    int bt_lim_2 = ((int)strlen(bv_s_self));
    int bt_step_2 = 1;
    for (bv_i_i = 1; bt_step_2 >= 0 ? bv_i_i <= bt_lim_2 : bv_i_i >= bt_lim_2; bv_i_i += bt_step_2) {
        bv_i_c = ((int)(unsigned char)bcc_mid(bv_s_self, bv_i_i, 1)[0]);
        if (((-(bv_i_c >= 65)) && (-(bv_i_c <= 90)))) {
            bv_i_c = (bv_i_c + 32);
        }
        char bt_s_3[256];
        snprintf(bt_s_3, sizeof(bt_s_3), "%s%s", bv_s_out, bcc_chr(bv_i_c));
        snprintf(bv_s_out, sizeof(bv_s_out), "%s", bt_s_3);
    }
    snprintf(bcc_out, 256, "%s", bv_s_out);
    return;
}

int bf_i_max(int bv_i_a, int bv_i_b) {
    if ((-(bv_i_a > bv_i_b))) {
        return bv_i_a;
    } else {
        return bv_i_b;
    }
}

int bf_i_min(int bv_i_a, int bv_i_b) {
    if ((-(bv_i_a < bv_i_b))) {
        return bv_i_a;
    } else {
        return bv_i_b;
    }
}

int bf_i_clamp(int bv_i_value, int bv_i_lo, int bv_i_hi) {
    // Constrain value to [lo, hi].
    return bf_i_max(bv_i_lo, bf_i_min(bv_i_value, bv_i_hi));
}

void bf_s_repeat(const char* bv_s_text_in, int bv_i_n, char* bcc_out) {
    char bv_s_text[256];
    snprintf(bv_s_text, sizeof(bv_s_text), "%s", bv_s_text_in);
    int bv_i_i = 0;
    char bv_s_acc[256] = {0};

    // Concatenate text$ with itself n times.
    snprintf(bv_s_acc, sizeof(bv_s_acc), "%s", "");
    int bt_lim_4 = bv_i_n;
    int bt_step_4 = 1;
    for (bv_i_i = 1; bt_step_4 >= 0 ? bv_i_i <= bt_lim_4 : bv_i_i >= bt_lim_4; bv_i_i += bt_step_4) {
        char bt_s_5[256];
        snprintf(bt_s_5, sizeof(bt_s_5), "%s%s", bv_s_acc, bv_s_text);
        snprintf(bv_s_acc, sizeof(bv_s_acc), "%s", bt_s_5);
    }
    snprintf(bcc_out, 256, "%s", bv_s_acc);
    return;
}

void bf_s_titlecase(const char* bv_s_word_in, char* bcc_out) {
    char bv_s_word[256];
    snprintf(bv_s_word, sizeof(bv_s_word), "%s", bv_s_word_in);

    // Capitalise first letter, lowercase remainder.
    // UCASE$/LCASE$ aren't real MBASIC/BASCOM 2.00 builtins (verified
    // against a real IBM BASIC Compiler 2.00 under dosbox-x), so this
    // requires BASCAL's own com.bascal.stdlib implementations above.
    if ((-(((int)strlen(bv_s_word)) == 0))) {
        snprintf(bcc_out, 256, "%s", "");
        return;
    }
    char bt_s_6[256];
    bf_s_ucase_s(bcc_mid(bv_s_word, 1, 1), bt_s_6);
    char bt_s_7[256];
    bf_s_lcase_s(bcc_mid(bv_s_word, 2, 2147483647), bt_s_7);
    char bt_s_8[256];
    snprintf(bt_s_8, sizeof(bt_s_8), "%s%s", bt_s_6, bt_s_7);
    snprintf(bcc_out, 256, "%s", bt_s_8);
    return;
}

int bf_i_sumto(int bv_i_n) {
    int bv_i_acc = 0;
    int bv_i_i = 0;

    // i% and acc% are local to sumTo%.
    bv_i_acc = 0;
    int bt_lim_9 = bv_i_n;
    int bt_step_9 = 1;
    for (bv_i_i = 1; bt_step_9 >= 0 ? bv_i_i <= bt_lim_9 : bv_i_i >= bt_lim_9; bv_i_i += bt_step_9) {
        bv_i_acc = (bv_i_acc + bv_i_i);
    }
    return bv_i_acc;
}

int bf_i_productto(int bv_i_n) {
    int bv_i_acc = 0;
    int bv_i_i = 0;

    // i% and acc% here are independent of sumTo%'s i% and acc%.
    bv_i_acc = 1;
    int bt_lim_10 = bv_i_n;
    int bt_step_10 = 1;
    for (bv_i_i = 1; bt_step_10 >= 0 ? bv_i_i <= bt_lim_10 : bv_i_i >= bt_lim_10; bv_i_i += bt_step_10) {
        bv_i_acc = (bv_i_acc * bv_i_i);
    }
    return bv_i_acc;
}

int bf_i_addtototal(int bv_i_x) {
    bv_i_runningtotal = (bv_i_runningtotal + bv_i_x);
    return bv_i_runningtotal;
}

int main(void) {
    // Upper-cases self$. Not a real MBASIC/BASCOM 2.00 builtin -- verified
    // against a real IBM BASIC Compiler 2.00 under dosbox-x -- so BASCAL ships
    // its own. Declared as a scalar method (see GitHub issue #41 and
    // ltrim.bcl's own doc comment for the reasoning) -- ucase$(s$) still works
    // via ordinary-call syntax resolving to this same declaration.

    // Lower-cases self$. Not a real MBASIC/BASCOM 2.00 builtin -- verified
    // against a real IBM BASIC Compiler 2.00 under dosbox-x -- so BASCAL ships
    // its own. Declared as a scalar method (see GitHub issue #41 and
    // ltrim.bcl's own doc comment for the reasoning) -- lcase$(s$) still works
    // via ordinary-call syntax resolving to this same declaration.

    // Tutorial — Functions
    //
    // A BASCAL function is declared with FUNCTION ... END FUNCTION.
    // The function name carries the return type suffix.  Parameters
    // also carry type suffixes.  Every function must reach a RETURN.
    //
    // Variables declared inside a function are local by default: the compiler
    // prefixes them with the function name.  To access a global variable from
    // inside a function, declare it with:  global varname
    //
    // Functions cannot recurse, directly or indirectly (parameters would be
    // overwritten) -- the compiler checks the whole call graph and rejects
    // any cycle.  Use an explicit stack array for recursive algorithms.


    // Integer arithmetic functions
    // a% -- first value to compare
    // b% -- second value to compare

    // a% -- first value to compare
    // b% -- second value to compare

    // value% -- number to constrain
    // lo%    -- lower bound, inclusive
    // hi%    -- upper bound, inclusive

    // String functions
    // text$ -- string to repeat
    // n%    -- number of times to repeat it

    // word$ -- string to title-case

    // Local variable scoping — each function has its own i% and acc%
    // n% -- upper bound of the sum, inclusive

    // n% -- upper bound of the product, inclusive

    // Global variable accessed inside a function with the global keyword
    bv_i_runningtotal = 0;

    // x% -- amount to add to the running total

    // --- Exercise the functions ---

    // print mixes string labels and numeric results directly with ;
    printf("max(4, 9) = %d\n", bf_i_max(4, 9));
    printf("min(4, 9) = %d\n", bf_i_min(4, 9));
    printf("clamp(15,1,10) = %d\n", bf_i_clamp(15, 1, 10));
    printf("clamp(-3,1,10) = %d\n", bf_i_clamp(-(3), 1, 10));
    printf("clamp(7,1,10)  = %d\n", bf_i_clamp(7, 1, 10));

    char bt_s_11[256];
    bf_s_repeat("ab", 4, bt_s_11);
    printf("%s\n", bt_s_11);
    char bt_s_12[256];
    bf_s_titlecase("bASCAL", bt_s_12);
    printf("%s\n", bt_s_12);

    // Functions chained in expressions
    bv_i_lo = bf_i_min(bf_i_max(0, -(5)), 100);
    printf("lo = %d\n", bv_i_lo);

    // Calling the same function twice — each result is captured separately
    char bt_s_13[256];
    bf_s_repeat("x", 3, bt_s_13);
    snprintf(bv_s_a, sizeof(bv_s_a), "%s", bt_s_13);
    char bt_s_14[256];
    bf_s_repeat("y", 2, bt_s_14);
    snprintf(bv_s_b, sizeof(bv_s_b), "%s", bt_s_14);
    printf("%s %s\n", bv_s_a, bv_s_b);

    // Local scoping: sumTo% and productTo% each use i% without conflict
    printf("sumTo(5)     = %d\n", bf_i_sumto(5));
    printf("productTo(5) = %d\n", bf_i_productto(5));

    // Global variable shared across calls
    bv_i_dummy = bf_i_addtototal(10);
    bv_i_dummy = bf_i_addtototal(5);
    printf("runningTotal = %d\n", bv_i_runningtotal);

    return 0;
}
