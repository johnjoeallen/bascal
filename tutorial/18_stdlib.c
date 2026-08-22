#include <stdio.h>
#include <string.h>

#define BCC_STRBUF_COUNT 8
static char bcc_strbuf[BCC_STRBUF_COUNT][256];
static int bcc_strbuf_next = 0;

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

void bf_s_ltrim(const char* bv_s_s_in, char* bcc_out);
void bf_s_rtrim(const char* bv_s_s_in, char* bcc_out);
void bf_s_ucase(const char* bv_s_s_in, char* bcc_out);
void bf_s_lcase(const char* bv_s_s_in, char* bcc_out);
void bf_s_error(int bv_i_code, char* bcc_out);

void bf_s_ltrim(const char* bv_s_s_in, char* bcc_out) {
    char bv_s_s[256];
    snprintf(bv_s_s, sizeof(bv_s_s), "%s", bv_s_s_in);
    int bv_i_i = 0;

    bv_i_i = 1;
    while (((-(bv_i_i <= ((int)strlen(bv_s_s)))) && (-(strcmp(bcc_mid(bv_s_s, bv_i_i, 1), " ") == 0)))) {
        bv_i_i = (bv_i_i + 1);
    }
    snprintf(bcc_out, 256, "%s", bcc_mid(bv_s_s, bv_i_i, 2147483647));
    return;
}

void bf_s_rtrim(const char* bv_s_s_in, char* bcc_out) {
    char bv_s_s[256];
    snprintf(bv_s_s, sizeof(bv_s_s), "%s", bv_s_s_in);
    int bv_i_i = 0;

    bv_i_i = ((int)strlen(bv_s_s));
    while (((-(bv_i_i > 0)) && (-(strcmp(bcc_mid(bv_s_s, bv_i_i, 1), " ") == 0)))) {
        bv_i_i = (bv_i_i - 1);
    }
    snprintf(bcc_out, 256, "%s", bcc_mid(bv_s_s, 1, bv_i_i));
    return;
}

void bf_s_ucase(const char* bv_s_s_in, char* bcc_out) {
    char bv_s_s[256];
    snprintf(bv_s_s, sizeof(bv_s_s), "%s", bv_s_s_in);
    int bv_i_c = 0;
    int bv_i_i = 0;
    char bv_s_out[256] = {0};

    snprintf(bv_s_out, sizeof(bv_s_out), "%s", "");
    int bt_lim_0 = ((int)strlen(bv_s_s));
    int bt_step_0 = 1;
    for (bv_i_i = 1; bt_step_0 >= 0 ? bv_i_i <= bt_lim_0 : bv_i_i >= bt_lim_0; bv_i_i += bt_step_0) {
        bv_i_c = ((int)(unsigned char)bcc_mid(bv_s_s, bv_i_i, 1)[0]);
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

void bf_s_lcase(const char* bv_s_s_in, char* bcc_out) {
    char bv_s_s[256];
    snprintf(bv_s_s, sizeof(bv_s_s), "%s", bv_s_s_in);
    int bv_i_c = 0;
    int bv_i_i = 0;
    char bv_s_out[256] = {0};

    snprintf(bv_s_out, sizeof(bv_s_out), "%s", "");
    int bt_lim_2 = ((int)strlen(bv_s_s));
    int bt_step_2 = 1;
    for (bv_i_i = 1; bt_step_2 >= 0 ? bv_i_i <= bt_lim_2 : bv_i_i >= bt_lim_2; bv_i_i += bt_step_2) {
        bv_i_c = ((int)(unsigned char)bcc_mid(bv_s_s, bv_i_i, 1)[0]);
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

void bf_s_error(int bv_i_code, char* bcc_out) {
    {
        int bt_sel_4 = bv_i_code;
        if ((bt_sel_4 == 2)) {
            snprintf(bcc_out, 256, "%s", "Syntax error");
            return;
        } else if ((bt_sel_4 == 3)) {
            snprintf(bcc_out, 256, "%s", "RETURN without GOSUB");
            return;
        } else if ((bt_sel_4 == 4)) {
            snprintf(bcc_out, 256, "%s", "Out of DATA");
            return;
        } else if ((bt_sel_4 == 5)) {
            snprintf(bcc_out, 256, "%s", "Illegal function call");
            return;
        } else if ((bt_sel_4 == 6)) {
            snprintf(bcc_out, 256, "%s", "Overflow");
            return;
        } else if ((bt_sel_4 == 7)) {
            snprintf(bcc_out, 256, "%s", "Out of memory");
            return;
        } else if ((bt_sel_4 == 9)) {
            snprintf(bcc_out, 256, "%s", "Subscript out of range");
            return;
        } else if ((bt_sel_4 == 10)) {
            snprintf(bcc_out, 256, "%s", "Duplicate Definition");
            return;
        } else if ((bt_sel_4 == 11)) {
            snprintf(bcc_out, 256, "%s", "Division by zero");
            return;
        } else if ((bt_sel_4 == 13)) {
            snprintf(bcc_out, 256, "%s", "Type mismatch");
            return;
        } else if ((bt_sel_4 == 14)) {
            snprintf(bcc_out, 256, "%s", "Out of string space");
            return;
        } else if ((bt_sel_4 == 19)) {
            snprintf(bcc_out, 256, "%s", "No RESUME");
            return;
        } else if ((bt_sel_4 == 20)) {
            snprintf(bcc_out, 256, "%s", "RESUME without error");
            return;
        } else if ((bt_sel_4 == 24)) {
            snprintf(bcc_out, 256, "%s", "Device timeout");
            return;
        } else if ((bt_sel_4 == 25)) {
            snprintf(bcc_out, 256, "%s", "Device fault");
            return;
        } else if ((bt_sel_4 == 27)) {
            snprintf(bcc_out, 256, "%s", "Out of paper");
            return;
        } else if ((bt_sel_4 == 52)) {
            snprintf(bcc_out, 256, "%s", "Bad file number");
            return;
        } else if ((bt_sel_4 == 53)) {
            snprintf(bcc_out, 256, "%s", "File not found");
            return;
        } else if ((bt_sel_4 == 54)) {
            snprintf(bcc_out, 256, "%s", "Bad file mode");
            return;
        } else if ((bt_sel_4 == 55)) {
            snprintf(bcc_out, 256, "%s", "File already open");
            return;
        } else if ((bt_sel_4 == 57)) {
            snprintf(bcc_out, 256, "%s", "Device I/O error");
            return;
        } else if ((bt_sel_4 == 58)) {
            snprintf(bcc_out, 256, "%s", "File already exists");
            return;
        } else if ((bt_sel_4 == 61)) {
            snprintf(bcc_out, 256, "%s", "Disk full");
            return;
        } else if ((bt_sel_4 == 62)) {
            snprintf(bcc_out, 256, "%s", "Input past end");
            return;
        } else if ((bt_sel_4 == 63)) {
            snprintf(bcc_out, 256, "%s", "Bad record number");
            return;
        } else if ((bt_sel_4 == 64)) {
            snprintf(bcc_out, 256, "%s", "Bad file name");
            return;
        } else if ((bt_sel_4 == 67)) {
            snprintf(bcc_out, 256, "%s", "Too many files");
            return;
        } else if ((bt_sel_4 == 68)) {
            snprintf(bcc_out, 256, "%s", "Device unavailable");
            return;
        } else if ((bt_sel_4 == 70)) {
            snprintf(bcc_out, 256, "%s", "Disk write protected");
            return;
        } else if ((bt_sel_4 == 71)) {
            snprintf(bcc_out, 256, "%s", "Disk not ready");
            return;
        } else if ((bt_sel_4 == 72)) {
            snprintf(bcc_out, 256, "%s", "Disk media error");
            return;
        } else if ((bt_sel_4 == 75)) {
            snprintf(bcc_out, 256, "%s", "Path/File access error");
            return;
        } else if ((bt_sel_4 == 76)) {
            snprintf(bcc_out, 256, "%s", "Path not found");
            return;
        } else {
            char bt_s_5[256];
            snprintf(bt_s_5, sizeof(bt_s_5), "%s%s", "Error ", bcc_stri(bv_i_code));
            snprintf(bcc_out, 256, "%s", bt_s_5);
            return;
        }
    }
}

int main(void) {
    // Strips leading spaces from s$. Not a real MBASIC/BASCOM 2.00 builtin --
    // verified against a real IBM BASIC Compiler 2.00 under dosbox-x -- so
    // BASCAL ships its own.

    // Strips trailing spaces from s$. Not a real MBASIC/BASCOM 2.00 builtin --
    // verified against a real IBM BASIC Compiler 2.00 under dosbox-x -- so
    // BASCAL ships its own.

    // Upper-cases s$. Not a real MBASIC/BASCOM 2.00 builtin -- verified against
    // a real IBM BASIC Compiler 2.00 under dosbox-x -- so BASCAL ships its own.

    // Lower-cases s$. Not a real MBASIC/BASCOM 2.00 builtin -- verified against
    // a real IBM BASIC Compiler 2.00 under dosbox-x -- so BASCAL ships its own.

    // Maps an ERR code to its classic MBASIC/GW-BASIC/BASCOM message. Compiles
    // and links on a real IBM BASIC Compiler 2.00 as ERROR$, but silently
    // returns an empty string at runtime (verified under dosbox-x) -- so BASCAL
    // ships a working implementation.
    //
    // Covers the classic error codes an ON ERROR GOTO + ERR handler is
    // realistically going to hit -- not the full table, but every code common
    // enough to be worth a real message instead of falling through to the
    // generic one.

    // Tutorial 18 — Standard library functions
    //
    // com.bascal.stdlib is an ordinary require-able library, resolved the same
    // way as com.bascal.sort in tutorial 12 -- but bcc always adds its home
    // directory to the search path automatically, so no -L flag is needed to
    // reach it. It exists because LTRIM$, RTRIM$, UCASE$, LCASE$, and ERROR$
    // either aren't real MBASIC/BASCOM 2.00 builtins or don't work at runtime
    // (verified against a real IBM Personal Computer BASIC Compiler 2.00 under
    // dosbox-x) -- see the manual's "String and error-message functions"
    // section (https://johnjoeallen.github.io/bascal/manual/) for the full
    // story.
    //
    // Run with:
    // bcc tutorial/18_stdlib.bcl


    char bt_s_6[256];
    bf_s_ltrim("   padded left", bt_s_6);
    char bt_s_7[256];
    snprintf(bt_s_7, sizeof(bt_s_7), "%s%s", "[", bt_s_6);
    char bt_s_8[256];
    snprintf(bt_s_8, sizeof(bt_s_8), "%s%s", bt_s_7, "]");
    printf("%s\n", bt_s_8);
    char bt_s_9[256];
    bf_s_rtrim("padded right   ", bt_s_9);
    char bt_s_10[256];
    snprintf(bt_s_10, sizeof(bt_s_10), "%s%s", "[", bt_s_9);
    char bt_s_11[256];
    snprintf(bt_s_11, sizeof(bt_s_11), "%s%s", bt_s_10, "]");
    printf("%s\n", bt_s_11);
    char bt_s_12[256];
    bf_s_ucase("shout this", bt_s_12);
    printf("%s\n", bt_s_12);
    char bt_s_13[256];
    bf_s_lcase("QUIET THIS DOWN", bt_s_13);
    printf("%s\n", bt_s_13);

    // ERROR$ maps a classic MBASIC/GW-BASIC/BASCOM error code to a message;
    // pair it with ERR inside an ON ERROR GOTO handler in real code.
    char bt_s_14[256];
    bf_s_error(53, bt_s_14);
    printf("%s\n", bt_s_14);
    char bt_s_15[256];
    bf_s_error(11, bt_s_15);
    printf("%s\n", bt_s_15);
    char bt_s_16[256];
    bf_s_error(9999, bt_s_16);
    printf("%s\n", bt_s_16);

    return 0;
}
