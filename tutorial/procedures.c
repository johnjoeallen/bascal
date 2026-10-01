// BASCAL generated C -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
#include <stdio.h>
#include <string.h>

#define BCC_STRBUF_COUNT 8
static char bcc_strbuf[BCC_STRBUF_COUNT][256];
static int bcc_strbuf_next = 0;

static char* bcc_strbuf_take(void);
static const char* bcc_mid(const char* s, int start, int length);
static const char* bcc_chr(int code);
static const char* bcc_stri(int value);
static const char* bcc_strd(double value);

static int bv_i_globalcount = 0;
static int bv_i_i = 0;
static int bv_i_n = 0;
static int bv_i_data[6] = {0};

void bf_i_printseparator(void);
void bf_i_printscore(const char* bv_s_label_in, int bv_i_score);
void bf_i_printifpass(const char* bv_s_name_in, int bv_i_score);
void bf_i_fillrange(int bv_i_arr_len0, int* bv_i_arr, int bv_i_value);
void bf_i_increment(void);

void bf_i_printseparator(void) {
    printf("----------------------------\n");
}

void bf_i_printscore(const char* bv_s_label_in, int bv_i_score) {
    char bv_s_label[256];
    snprintf(bv_s_label, sizeof(bv_s_label), "%s", bv_s_label_in);

    char bt_s_0[256];
    snprintf(bt_s_0, sizeof(bt_s_0), "%s%s", bv_s_label, ": ");
    char bt_s_1[256];
    snprintf(bt_s_1, sizeof(bt_s_1), "%s%s", bt_s_0, bcc_stri(bv_i_score));
    printf("%s\n", bt_s_1);
}

void bf_i_printifpass(const char* bv_s_name_in, int bv_i_score) {
    char bv_s_name[256];
    snprintf(bv_s_name, sizeof(bv_s_name), "%s", bv_s_name_in);

    if ((-(bv_i_score < 60))) {
        return;
        // early exit — nothing printed for failing scores
    }
    char bt_s_2[256];
    snprintf(bt_s_2, sizeof(bt_s_2), "%s%s", bv_s_name, " passed with ");
    char bt_s_3[256];
    snprintf(bt_s_3, sizeof(bt_s_3), "%s%s", bt_s_2, bcc_stri(bv_i_score));
    printf("%s\n", bt_s_3);
}

void bf_i_fillrange(int bv_i_arr_len0, int* bv_i_arr, int bv_i_value) {
    int bv_i_i = 0;

    int bt_start_4 = 0;
    int bt_lim_4 = (((bv_i_arr_len0 - 1) + 1) - 1);
    int bt_step_4 = 1;
    for (bv_i_i = bt_start_4; bt_step_4 >= 0 ? bv_i_i <= bt_lim_4 : bv_i_i >= bt_lim_4; bv_i_i += bt_step_4) {
        bv_i_arr[(bv_i_i)] = bv_i_value;
    }
}

void bf_i_increment(void) {
    bv_i_globalcount = (bv_i_globalcount + 1);
}

int main(void) {
    // Tutorial — Procedures
    //
    // A procedure is like a function but returns no value.  Declare it with
    // PROCEDURE ... END PROCEDURE.  The name must not carry a type suffix.
    //
    // Variables inside a procedure are LOCAL by default: the compiler prefixes
    // them with the procedure name.  To access a global variable, declare it
    // inside the body with:  global varname
    //
    // Use procedures for actions that produce side effects (output, file I/O,
    // modifying arrays) rather than for computing a value.
    //
    // A bare RETURN exits a procedure early.  Falling through to END PROCEDURE
    // is also valid — an implicit RETURN is emitted.

    // Procedure with no parameters

    // Procedure that prints a labelled value
    // label$ -- text shown before the score
    // score% -- value to print

    // Procedure with early exit
    // name$  -- person's name
    // score% -- score to test against the passing threshold

    // Procedure that modifies an array in place -- byref copies the result
    // back to the caller; the default byval would fill a private copy only.
    // arr%   -- array to fill; byref because it's mutated in place
    // value% -- value written into every element

    // Procedure that uses a global variable
    bv_i_globalcount = 0;

    // --- Drive the procedures ---

    bf_i_printseparator();
    char bt_arg_5[256];
    snprintf(bt_arg_5, sizeof(bt_arg_5), "%s", "Alice");
    int bt_arg_6 = 91;
    bf_i_printscore(bt_arg_5, bt_arg_6);
    char bt_arg_7[256];
    snprintf(bt_arg_7, sizeof(bt_arg_7), "%s", "Bob");
    int bt_arg_8 = 54;
    bf_i_printscore(bt_arg_7, bt_arg_8);
    char bt_arg_9[256];
    snprintf(bt_arg_9, sizeof(bt_arg_9), "%s", "Carol");
    int bt_arg_10 = 78;
    bf_i_printscore(bt_arg_9, bt_arg_10);
    bf_i_printseparator();

    printf("Passes only:\n");
    char bt_arg_11[256];
    snprintf(bt_arg_11, sizeof(bt_arg_11), "%s", "Alice");
    int bt_arg_12 = 91;
    bf_i_printifpass(bt_arg_11, bt_arg_12);
    char bt_arg_13[256];
    snprintf(bt_arg_13, sizeof(bt_arg_13), "%s", "Bob");
    int bt_arg_14 = 54;
    bf_i_printifpass(bt_arg_13, bt_arg_14);
    char bt_arg_15[256];
    snprintf(bt_arg_15, sizeof(bt_arg_15), "%s", "Carol");
    int bt_arg_16 = 78;
    bf_i_printifpass(bt_arg_15, bt_arg_16);

    bv_i_n = 5;
    int bt_arg_17 = 99;
    bf_i_fillrange(6, bv_i_data, bt_arg_17);
    printf("Filled array:\n");
    int bt_lim_18 = (bv_i_n - 1);
    int bt_step_18 = 1;
    for (bv_i_i = 0; bt_step_18 >= 0 ? bv_i_i <= bt_lim_18 : bv_i_i >= bt_lim_18; bv_i_i += bt_step_18) {
        char bt_s_19[256];
        snprintf(bt_s_19, sizeof(bt_s_19), "%s%s", "  data%(", bcc_stri(bv_i_i));
        char bt_s_20[256];
        snprintf(bt_s_20, sizeof(bt_s_20), "%s%s", bt_s_19, ") = ");
        char bt_s_21[256];
        snprintf(bt_s_21, sizeof(bt_s_21), "%s%s", bt_s_20, bcc_stri(bv_i_data[(bv_i_i)]));
        printf("%s\n", bt_s_21);
    }

    bf_i_increment();
    bf_i_increment();
    bf_i_increment();
    char bt_s_22[256];
    snprintf(bt_s_22, sizeof(bt_s_22), "%s%s", "globalCount = ", bcc_stri(bv_i_globalcount));
    printf("%s\n", bt_s_22);

    return 0;
}

static char* bcc_strbuf_take(void) {
    char* buf = bcc_strbuf[bcc_strbuf_next];
    bcc_strbuf_next = (bcc_strbuf_next + 1) % BCC_STRBUF_COUNT;
    return buf;
}

static const char* bcc_mid(const char* s, int start, int length) {
    char* out = bcc_strbuf_take();
    int len = (int)strlen(s);
    int from = start - 1;
    if (from < 0) from = 0;
    if (from > len) from = len;
    int avail = len - from;
    if (length < 0) length = 0;
    if (length > avail) length = avail;
    snprintf(out, 256, "%.*s", length, s + from);
    return out;
}

static const char* bcc_chr(int code) {
    char* out = bcc_strbuf_take();
    snprintf(out, 256, "%c", code);
    return out;
}

static const char* bcc_stri(int value) {
    char* out = bcc_strbuf_take();
    snprintf(out, 256, "% d", value);
    return out;
}

static const char* bcc_strd(double value) {
    char* out = bcc_strbuf_take();
    snprintf(out, 256, "% g", value);
    return out;
}

