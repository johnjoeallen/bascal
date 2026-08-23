#include <stdio.h>
#include <string.h>

#include "bcc_runtime.h"

void bf_s_ltrim_s(const char* bv_s_self_in, char* bcc_out);
void bf_s_rtrim_s(const char* bv_s_self_in, char* bcc_out);
void bf_s_ucase_s(const char* bv_s_self_in, char* bcc_out);
void bf_s_lcase_s(const char* bv_s_self_in, char* bcc_out);
void bf_s_error(int bv_i_code, char* bcc_out);

void bf_s_ltrim_s(const char* bv_s_self_in, char* bcc_out) {
    char bv_s_self[256];
    snprintf(bv_s_self, sizeof(bv_s_self), "%s", bv_s_self_in);
    int bv_i_i = 0;

    bv_i_i = 1;
    while (((-(bv_i_i <= ((int)strlen(bv_s_self)))) && (-(strcmp(bcc_mid(bv_s_self, bv_i_i, 1), " ") == 0)))) {
        bv_i_i = (bv_i_i + 1);
    }
    snprintf(bcc_out, 256, "%s", bcc_mid(bv_s_self, bv_i_i, 2147483647));
    return;
}

void bf_s_rtrim_s(const char* bv_s_self_in, char* bcc_out) {
    char bv_s_self[256];
    snprintf(bv_s_self, sizeof(bv_s_self), "%s", bv_s_self_in);
    int bv_i_i = 0;

    bv_i_i = ((int)strlen(bv_s_self));
    while (((-(bv_i_i > 0)) && (-(strcmp(bcc_mid(bv_s_self, bv_i_i, 1), " ") == 0)))) {
        bv_i_i = (bv_i_i - 1);
    }
    snprintf(bcc_out, 256, "%s", bcc_mid(bv_s_self, 1, bv_i_i));
    return;
}

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
    // Strips leading spaces from self$. Not a real MBASIC/BASCOM 2.00 builtin --
    // verified against a real IBM BASIC Compiler 2.00 under dosbox-x -- so
    // BASCAL ships its own. Declared as a scalar method (see GitHub issue #41)
    // so a required stdlib call reads the same way as a built-in method call
    // (docs/language/functions-and-procedures.html#built-in-methods). The
    // ordinary call form (ltrim$(s$)) still works -- a method's receiver is an
    // implicit first parameter, so ordinary-call syntax resolves straight to
    // this same declaration, with no separate function needed (and no longer
    // allowed: a function and a method sharing one name is a duplicate
    // declaration, since they'd both claim the same callable identity).

    // Strips trailing spaces from self$. Not a real MBASIC/BASCOM 2.00 builtin --
    // verified against a real IBM BASIC Compiler 2.00 under dosbox-x -- so
    // BASCAL ships its own. Declared as a scalar method (see GitHub issue #41
    // and ltrim.bcl's own doc comment for the reasoning) -- rtrim$(s$) still
    // works via ordinary-call syntax resolving to this same declaration.

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

    // Maps an ERR code to its classic MBASIC/GW-BASIC/BASCOM message. Compiles
    // and links on a real IBM BASIC Compiler 2.00 as ERROR$, but silently
    // returns an empty string at runtime (verified under dosbox-x) -- so BASCAL
    // ships a working implementation.
    //
    // Covers the classic error codes an ON ERROR GOTO + ERR handler is
    // realistically going to hit -- not the full table, but every code common
    // enough to be worth a real message instead of falling through to the
    // generic one.
    //
    // Deliberately NOT a scalar method (see GitHub issue #41, which asked for
    // this decision to be recorded either way): code% is an opaque lookup key,
    // not a value the call is naturally "operating on" the way ltrim$/rtrim$/
    // ucase$/lcase$ operate on their string -- code%.error() would read as if
    // the *error code itself* has a message, when really this is a lookup
    // table keyed by that code. Stays an ordinary function.

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
    // ltrim$/rtrim$/ucase$/lcase$ are declared as scalar methods (method$ ...
    // end method), using self$ in place of an explicit s$ parameter -- see
    // the "Declare and call a method" chapter. A method's receiver is really
    // just an implicit first parameter, so the ordinary call form below
    // (ltrim$("...")) keeps working exactly as before: it resolves straight to
    // the same method declaration, with the first argument filling self$. The
    // method-call form (below, chained) is the same declaration too -- just
    // written as "...".ltrim() instead. error$ stays an ordinary function: an
    // error code is a lookup key, not a value the call is naturally "operating
    // on" the way the others operate on their string.
    //
    // Run with:
    // bcc tutorial/18_stdlib.bcl


    char bt_s_6[256];
    bf_s_ltrim_s("   padded left", bt_s_6);
    char bt_s_7[256];
    snprintf(bt_s_7, sizeof(bt_s_7), "%s%s", "[", bt_s_6);
    char bt_s_8[256];
    snprintf(bt_s_8, sizeof(bt_s_8), "%s%s", bt_s_7, "]");
    printf("%s\n", bt_s_8);
    char bt_s_9[256];
    bf_s_rtrim_s("padded right   ", bt_s_9);
    char bt_s_10[256];
    snprintf(bt_s_10, sizeof(bt_s_10), "%s%s", "[", bt_s_9);
    char bt_s_11[256];
    snprintf(bt_s_11, sizeof(bt_s_11), "%s%s", bt_s_10, "]");
    printf("%s\n", bt_s_11);
    char bt_s_12[256];
    bf_s_ucase_s("shout this", bt_s_12);
    printf("%s\n", bt_s_12);
    char bt_s_13[256];
    bf_s_lcase_s("QUIET THIS DOWN", bt_s_13);
    printf("%s\n", bt_s_13);

    // Same four functions, called as chained methods instead.
    char bt_s_14[256];
    bf_s_ltrim_s("  padded both sides  ", bt_s_14);
    char bt_s_15[256];
    bf_s_rtrim_s(bt_s_14, bt_s_15);
    char bt_s_16[256];
    snprintf(bt_s_16, sizeof(bt_s_16), "%s%s", "[", bt_s_15);
    char bt_s_17[256];
    snprintf(bt_s_17, sizeof(bt_s_17), "%s%s", bt_s_16, "]");
    printf("%s\n", bt_s_17);
    char bt_s_18[256];
    bf_s_ltrim_s("  shout this too", bt_s_18);
    char bt_s_19[256];
    bf_s_ucase_s(bt_s_18, bt_s_19);
    printf("%s\n", bt_s_19);

    // ERROR$ maps a classic MBASIC/GW-BASIC/BASCOM error code to a message;
    // pair it with ERR inside an ON ERROR GOTO handler in real code.
    char bt_s_20[256];
    bf_s_error(53, bt_s_20);
    printf("%s\n", bt_s_20);
    char bt_s_21[256];
    bf_s_error(11, bt_s_21);
    printf("%s\n", bt_s_21);
    char bt_s_22[256];
    bf_s_error(9999, bt_s_22);
    printf("%s\n", bt_s_22);

    return 0;
}
