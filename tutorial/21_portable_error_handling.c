// BASCAL generated C -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
#include <stdio.h>
#include <math.h>
#include <string.h>
#include <stdlib.h>

#define BCC_STRBUF_COUNT 8
static char bcc_strbuf[BCC_STRBUF_COUNT][256];
static int bcc_strbuf_next = 0;

static int bcc_err = 0;
static int bcc_on_error_target = -1;
static int bcc_in_handler = 0;
static int bcc_resume_id = -1;
static int bcc_erl = 0;
static const char *bcc_err_file = "";

static char* bcc_strbuf_take(void);
static const char* bcc_mid(const char* s, int start, int length);
static const char* bcc_chr(int code);
static const char* bcc_stri(int value);
static const char* bcc_strd(double value);

static float bv_f_err_file_already_open = 0;
static float bv_f_err_file_not_found = 0;
static int bv_i_erl = 0;
static int bv_i_err = 0;
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
static char bv_s_source[256] = {0};

void bf_s_error(int bv_i_code, char* bcc_out);

void bf_s_error(int bv_i_code, char* bcc_out) {
    {
        int bt_sel_0 = bv_i_code;
        int bt_sel_match_1 = 0;
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_syntax)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Syntax error");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_return_without_gosub)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "RETURN without GOSUB");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_out_of_data)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Out of DATA");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_illegal_function_call)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Illegal function call");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_overflow)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Overflow");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_out_of_memory)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Out of memory");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_subscript_out_of_range)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Subscript out of range");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_duplicate_definition)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Duplicate Definition");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_division_by_zero)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Division by zero");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_type_mismatch)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Type mismatch");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_out_of_string_space)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Out of string space");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_no_resume)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "No RESUME");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_resume_without_error)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "RESUME without error");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_device_timeout)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Device timeout");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_device_fault)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Device fault");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_out_of_paper)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Out of paper");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_bad_file_number)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Bad file number");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_file_not_found)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "File not found");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_bad_file_mode)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Bad file mode");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_file_already_open)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "File already open");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_device_io)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Device I/O error");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_file_already_exists)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "File already exists");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_disk_full)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Disk full");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_input_past_end)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Input past end");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_bad_record_number)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Bad record number");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_bad_file_name)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Bad file name");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_too_many_files)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Too many files");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_device_unavailable)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Device unavailable");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_disk_write_protected)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Disk write protected");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_disk_not_ready)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Disk not ready");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_disk_media_error)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Disk media error");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_path_file_access)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Path/File access error");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_path_not_found)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Path not found");
                return;
            }
        }
        if (!bt_sel_match_1) {
            char bt_s_2[256];
            snprintf(bt_s_2, sizeof(bt_s_2), "%s%s", "Error ", bcc_stri(bv_i_code));
            snprintf(bcc_out, 256, "%s", bt_s_2);
            return;
        }
    }
}

int main(void) {
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

    // Tutorial — Portable Structured Error Handling
    //
    // TRY/CATCH/FINALLY and THROW are BASCAL's portable error model.  A catch can
    // select several error codes and bind the originating source file.

    printf("portable try/catch:\n");
    int bcc_try_0_pending = 0;
    bcc_on_error_target = 0;
    bcc_err = ((int)round((double)(bv_f_err_file_not_found)));
    bcc_erl = 10;
    bcc_err_file = "tutorial/21_portable_error_handling.bcl";
    goto bcc_try_0_catch;
    bcc_on_error_target = -1;
    goto bcc_try_0_finally;
    bcc_try_0_catch: ;
    bcc_in_handler = 0;
    bcc_on_error_target = -1;
    if (!((bcc_err == ((int)round((double)(bv_f_err_file_not_found)))) || (bcc_err == ((int)round((double)(bv_f_err_file_already_open)))))) {
        bcc_try_0_pending = 1;
        goto bcc_try_0_finally;
    }
    bv_i_err = bcc_err;
    bv_i_erl = bcc_erl;
    snprintf(bv_s_source, 256, "%s", bcc_err_file);
    printf("  caught error %d at %s:%d\n", bv_i_err, bv_s_source, bv_i_erl);
    bcc_on_error_target = -1;
    goto bcc_try_0_finally;
    bcc_try_0_rethrow: ;
    bcc_try_0_pending = 1;
    bcc_try_0_finally: ;
    printf("  cleanup always runs\n");
    if (bcc_try_0_pending) {
        fprintf(stderr, "unhandled BASIC error %d\n", bcc_err);
        exit(1);
    }
    bcc_try_0_end: ;

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

