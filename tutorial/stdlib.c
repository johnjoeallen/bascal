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

static int bv_i_err_bad_file_mode = 0;
static int bv_i_err_bad_file_name = 0;
static int bv_i_err_bad_file_number = 0;
static int bv_i_err_bad_record_number = 0;
static int bv_i_err_device_fault = 0;
static int bv_i_err_device_io = 0;
static int bv_i_err_device_timeout = 0;
static int bv_i_err_device_unavailable = 0;
static int bv_i_err_disk_full = 0;
static int bv_i_err_disk_media_error = 0;
static int bv_i_err_disk_not_ready = 0;
static int bv_i_err_disk_write_protected = 0;
static int bv_i_err_division_by_zero = 0;
static int bv_i_err_duplicate_definition = 0;
static int bv_i_err_file_already_exists = 0;
static int bv_i_err_file_already_open = 0;
static int bv_i_err_file_not_found = 0;
static int bv_i_err_illegal_function_call = 0;
static int bv_i_err_input_past_end = 0;
static int bv_i_err_no_resume = 0;
static int bv_i_err_out_of_data = 0;
static int bv_i_err_out_of_memory = 0;
static int bv_i_err_out_of_paper = 0;
static int bv_i_err_out_of_string_space = 0;
static int bv_i_err_overflow = 0;
static int bv_i_err_path_file_access = 0;
static int bv_i_err_path_not_found = 0;
static int bv_i_err_resume_without_error = 0;
static int bv_i_err_return_without_gosub = 0;
static int bv_i_err_subscript_out_of_range = 0;
static int bv_i_err_syntax = 0;
static int bv_i_err_too_many_files = 0;
static int bv_i_err_type_mismatch = 0;

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
    while ((-(((-(bv_i_i <= ((int)strlen(bv_s_self))))) != 0 && ((-(strcmp(bcc_mid(bv_s_self, bv_i_i, 1), " ") == 0))) != 0))) {
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
    while ((-(((-(bv_i_i > 0))) != 0 && ((-(strcmp(bcc_mid(bv_s_self, bv_i_i, 1), " ") == 0))) != 0))) {
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
    int bt_start_0 = 1;
    int bt_lim_0 = ((int)strlen(bv_s_self));
    int bt_step_0 = 1;
    for (bv_i_i = bt_start_0; bt_step_0 >= 0 ? bv_i_i <= bt_lim_0 : bv_i_i >= bt_lim_0; bv_i_i += bt_step_0) {
        bv_i_c = ((int)(unsigned char)bcc_mid(bv_s_self, bv_i_i, 1)[0]);
        if ((-(((-(bv_i_c >= 97))) != 0 && ((-(bv_i_c <= 122))) != 0))) {
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
    int bt_start_2 = 1;
    int bt_lim_2 = ((int)strlen(bv_s_self));
    int bt_step_2 = 1;
    for (bv_i_i = bt_start_2; bt_step_2 >= 0 ? bv_i_i <= bt_lim_2 : bv_i_i >= bt_lim_2; bv_i_i += bt_step_2) {
        bv_i_c = ((int)(unsigned char)bcc_mid(bv_s_self, bv_i_i, 1)[0]);
        if ((-(((-(bv_i_c >= 65))) != 0 && ((-(bv_i_c <= 90))) != 0))) {
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
        int bt_sel_match_5 = 0;
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_syntax)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Syntax error");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_return_without_gosub)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "RETURN without GOSUB");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_out_of_data)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Out of DATA");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_illegal_function_call)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Illegal function call");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_overflow)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Overflow");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_out_of_memory)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Out of memory");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_subscript_out_of_range)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Subscript out of range");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_duplicate_definition)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Duplicate Definition");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_division_by_zero)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Division by zero");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_type_mismatch)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Type mismatch");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_out_of_string_space)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Out of string space");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_no_resume)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "No RESUME");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_resume_without_error)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "RESUME without error");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_device_timeout)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Device timeout");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_device_fault)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Device fault");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_out_of_paper)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Out of paper");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_bad_file_number)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Bad file number");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_file_not_found)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "File not found");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_bad_file_mode)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Bad file mode");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_file_already_open)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "File already open");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_device_io)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Device I/O error");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_file_already_exists)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "File already exists");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_disk_full)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Disk full");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_input_past_end)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Input past end");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_bad_record_number)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Bad record number");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_bad_file_name)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Bad file name");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_too_many_files)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Too many files");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_device_unavailable)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Device unavailable");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_disk_write_protected)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Disk write protected");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_disk_not_ready)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Disk not ready");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_disk_media_error)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Disk media error");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_path_file_access)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Path/File access error");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_path_not_found)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Path not found");
                return;
            }
        }
        if (!bt_sel_match_5) {
            char bt_s_6[256];
            snprintf(bt_s_6, sizeof(bt_s_6), "%s%s", "Error ", bcc_stri(bv_i_code));
            snprintf(bcc_out, 256, "%s", bt_s_6);
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
    // The named constants below are the complete common subset supported by
    // ERROR$: use them in THROW and filtered CATCH clauses instead of magic
    // numbers.  Dialect-specific errors outside this shared MBASIC/GW-BASIC/
    // BASCOM subset still fall through to ERROR$'s generic message.
    //
    // Deliberately NOT a scalar method (see GitHub issue #41, which asked for
    // this decision to be recorded either way): code% is an opaque lookup key,
    // not a value the call is naturally "operating on" the way ltrim$/rtrim$/
    // ucase$/lcase$ operate on their string -- code%.error() would read as if
    // the *error code itself* has a message, when really this is a lookup
    // table keyed by that code. Stays an ordinary function.

    bv_i_err_syntax = 2;
    bv_i_err_return_without_gosub = 3;
    bv_i_err_out_of_data = 4;
    bv_i_err_illegal_function_call = 5;
    bv_i_err_overflow = 6;
    bv_i_err_out_of_memory = 7;
    bv_i_err_subscript_out_of_range = 9;
    bv_i_err_duplicate_definition = 10;
    bv_i_err_division_by_zero = 11;
    bv_i_err_type_mismatch = 13;
    bv_i_err_out_of_string_space = 14;
    bv_i_err_no_resume = 19;
    bv_i_err_resume_without_error = 20;
    bv_i_err_device_timeout = 24;
    bv_i_err_device_fault = 25;
    bv_i_err_out_of_paper = 27;
    bv_i_err_bad_file_number = 52;
    bv_i_err_file_not_found = 53;
    bv_i_err_bad_file_mode = 54;
    bv_i_err_file_already_open = 55;
    bv_i_err_device_io = 57;
    bv_i_err_file_already_exists = 58;
    bv_i_err_disk_full = 61;
    bv_i_err_input_past_end = 62;
    bv_i_err_bad_record_number = 63;
    bv_i_err_bad_file_name = 64;
    bv_i_err_too_many_files = 67;
    bv_i_err_device_unavailable = 68;
    bv_i_err_disk_write_protected = 70;
    bv_i_err_disk_not_ready = 71;
    bv_i_err_disk_media_error = 72;
    bv_i_err_path_file_access = 75;
    bv_i_err_path_not_found = 76;

    // Tutorial — Standard library functions
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
    // ltrim$/rtrim$/ucase$/lcase$ are scalar methods with a bracketed string
    // receiver type, using self$ in place of an explicit s$ parameter -- see
    // the "Declare and call a method" chapter. A method's receiver is really
    // just an implicit first parameter, so the ordinary call form
    // (ltrim$("...")) keeps working exactly as before: it resolves straight to
    // the same method declaration, with the first argument filling self$. The
    // examples below prefer the method-call form, written as "...".ltrim().
    // error$ stays an ordinary function: an
    // error code is a lookup key, not a value the call is naturally "operating
    // on" the way the others operate on their string.
    //
    // Run with:
    // bcc tutorial/stdlib.bcl

    char bt_s_7[256];
    bf_s_ltrim_s("   padded left", bt_s_7);
    char bt_s_8[256];
    snprintf(bt_s_8, sizeof(bt_s_8), "%s%s", "[", bt_s_7);
    char bt_s_9[256];
    snprintf(bt_s_9, sizeof(bt_s_9), "%s%s", bt_s_8, "]");
    printf("%s\n", bt_s_9);
    char bt_s_10[256];
    bf_s_rtrim_s("padded right   ", bt_s_10);
    char bt_s_11[256];
    snprintf(bt_s_11, sizeof(bt_s_11), "%s%s", "[", bt_s_10);
    char bt_s_12[256];
    snprintf(bt_s_12, sizeof(bt_s_12), "%s%s", bt_s_11, "]");
    printf("%s\n", bt_s_12);
    char bt_s_13[256];
    bf_s_ucase_s("shout this", bt_s_13);
    printf("%s\n", bt_s_13);
    char bt_s_14[256];
    bf_s_lcase_s("QUIET THIS DOWN", bt_s_14);
    printf("%s\n", bt_s_14);

    // Same four functions, called as chained methods instead.
    char bt_s_15[256];
    bf_s_ltrim_s("  padded both sides  ", bt_s_15);
    char bt_s_16[256];
    bf_s_rtrim_s(bt_s_15, bt_s_16);
    char bt_s_17[256];
    snprintf(bt_s_17, sizeof(bt_s_17), "%s%s", "[", bt_s_16);
    char bt_s_18[256];
    snprintf(bt_s_18, sizeof(bt_s_18), "%s%s", bt_s_17, "]");
    printf("%s\n", bt_s_18);
    char bt_s_19[256];
    bf_s_ltrim_s("  shout this too", bt_s_19);
    char bt_s_20[256];
    bf_s_ucase_s(bt_s_19, bt_s_20);
    printf("%s\n", bt_s_20);

    // ERROR$ maps a classic MBASIC/GW-BASIC/BASCOM error code to a message;
    // pair it with ERR inside an ON ERROR GOTO handler in real code.
    char bt_s_21[256];
    bf_s_error(53, bt_s_21);
    printf("%s\n", bt_s_21);
    char bt_s_22[256];
    bf_s_error(11, bt_s_22);
    printf("%s\n", bt_s_22);
    char bt_s_23[256];
    bf_s_error(9999, bt_s_23);
    printf("%s\n", bt_s_23);

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

