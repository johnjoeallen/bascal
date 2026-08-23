#include <stdio.h>
#include <math.h>
#include <string.h>
#include <stdint.h>
#include <stdlib.h>

#include "bcc_runtime.h"

static int bv_i_score = 0;
static char bv_s_csvfile[256] = {0};
static char bv_s_line[256] = {0};
static char bv_s_name[256] = {0};
static char bv_s_result[256] = {0};

int main(void) {
    // Tutorial 10 — File Input and Output
    //
    // This tutorial writes the *same* sequential file twice. Part 1 uses raw
    // BASIC file statements directly. Part 2 uses BASCAL's file-handle DSL,
    // which transpiles to exactly the same primitives, minus having to pick a
    // channel number and repeat it at every open/write/read/close. Read Part
    // 1 first; the comments in Part 2 explain what the DSL is buying you.
    //
    // ---- Part 1 primitives ----
    //
    // open filename$ for input  as #n   — read existing file
    // open filename$ for output as #n   — create or overwrite
    // open filename$ for append as #n   — add to end of existing file
    // close #n                          — flush and release the file
    //
    // print #n, expr[, ...]   — write values separated by spaces
    // write #n, expr[, ...]   — write quoted strings, comma-separated
    // (produces data that input # can read back)
    // line input #n, var$     — read one complete line into var$
    // input #n, var[, ...]    — read comma-delimited values (matches write)
    // eof(n)                  — returns non-zero when file n is exhausted

    snprintf(bv_s_csvfile, sizeof(bv_s_csvfile), "%s", "tutorial_scores.csv");

    // ============================================================
    // Part 1 — sequential files, written by hand
    // ============================================================

    // Write three records
    bcc_files[0] = fopen(bv_s_csvfile, "w");
    fprintf(bcc_files[0], "\"%s\",%d,\"%s\"\n", "Alice", 95, "pass");
    fprintf(bcc_files[0], "\"%s\",%d,\"%s\"\n", "Bob", 54, "fail");
    fprintf(bcc_files[0], "\"%s\",%d,\"%s\"\n", "Carol", 78, "pass");
    fclose(bcc_files[0]);
    bcc_files[0] = NULL;

    // Append a fourth record
    bcc_files[0] = fopen(bv_s_csvfile, "a");
    fprintf(bcc_files[0], "\"%s\",%d,\"%s\"\n", "Dave", 88, "pass");
    fclose(bcc_files[0]);
    bcc_files[0] = NULL;

    // Read and print every record
    char bt_s_0[256];
    snprintf(bt_s_0, sizeof(bt_s_0), "%s%s", "Part 1 (hand-written) -- all records in ", bv_s_csvfile);
    char bt_s_1[256];
    snprintf(bt_s_1, sizeof(bt_s_1), "%s%s", bt_s_0, ":");
    printf("%s\n", bt_s_1);
    bcc_raise_retry_0: ;
    bcc_files[0] = fopen(bv_s_csvfile, "r");
    if (!bcc_files[0]) {
        bcc_err = 53;
        bcc_resume_id = 0;
        if (bcc_on_error_target < 0 || bcc_in_handler) {
            fprintf(stderr, "unhandled BASIC error %d\n", bcc_err);
            exit(1);
        }
        bcc_in_handler = 1;
        switch (bcc_on_error_target) {
        }
    }
    bcc_raise_after_0: ;
    while ((-(bcc_eof(bcc_files[0]) == 0))) {
        bcc_read_file_field(bcc_files[0], bcc_file_field_buf, sizeof(bcc_file_field_buf));
        snprintf(bv_s_name, sizeof(bv_s_name), "%s", bcc_file_field_buf);
        bcc_read_file_field(bcc_files[0], bcc_file_field_buf, sizeof(bcc_file_field_buf));
        bv_i_score = atoi(bcc_file_field_buf);
        bcc_read_file_field(bcc_files[0], bcc_file_field_buf, sizeof(bcc_file_field_buf));
        snprintf(bv_s_result, sizeof(bv_s_result), "%s", bcc_file_field_buf);
        char bt_s_2[256];
        snprintf(bt_s_2, sizeof(bt_s_2), "%s%s", "  ", bv_s_name);
        char bt_s_3[256];
        snprintf(bt_s_3, sizeof(bt_s_3), "%s%s", bt_s_2, ": ");
        char bt_s_4[256];
        snprintf(bt_s_4, sizeof(bt_s_4), "%s%s", bt_s_3, bcc_stri(bv_i_score));
        char bt_s_5[256];
        snprintf(bt_s_5, sizeof(bt_s_5), "%s%s", bt_s_4, "  [");
        char bt_s_6[256];
        snprintf(bt_s_6, sizeof(bt_s_6), "%s%s", bt_s_5, bv_s_result);
        char bt_s_7[256];
        snprintf(bt_s_7, sizeof(bt_s_7), "%s%s", bt_s_6, "]");
        printf("%s\n", bt_s_7);
    }
    fclose(bcc_files[0]);
    bcc_files[0] = NULL;

    // Read the file line by line using line input
    printf("Part 1 (hand-written) -- raw lines:\n");
    bcc_raise_retry_1: ;
    bcc_files[0] = fopen(bv_s_csvfile, "r");
    if (!bcc_files[0]) {
        bcc_err = 53;
        bcc_resume_id = 1;
        if (bcc_on_error_target < 0 || bcc_in_handler) {
            fprintf(stderr, "unhandled BASIC error %d\n", bcc_err);
            exit(1);
        }
        bcc_in_handler = 1;
        switch (bcc_on_error_target) {
        }
    }
    bcc_raise_after_1: ;
    while ((-(bcc_eof(bcc_files[0]) == 0))) {
        bcc_line_input_file(bcc_files[0], bv_s_line, sizeof(bv_s_line));
        char bt_s_8[256];
        snprintf(bt_s_8, sizeof(bt_s_8), "%s%s", "  ", bv_s_line);
        printf("%s\n", bt_s_8);
    }
    fclose(bcc_files[0]);
    bcc_files[0] = NULL;

    // ============================================================
    // Part 2 — the same file, through the file-handle DSL
    // ============================================================

    // file <var> = open(<path>) for output/input/append
    // Opens a file the same way `open ... for ... as #n` does, except the
    // compiler picks the channel number for you and remembers it under
    // <var> — no #1/#2 to keep straight by hand, and no risk of two open
    // files quietly sharing a number.
    //
    // <var>.write(expr, ...)   — WRITE #n, expr, ...
    // <var>.read(var, ...)     — INPUT #n, var, ...     (only valid `for input`)
    // <var>.eof()               — EOF(n) <> 0             (only valid `for input`)
    // <var>.close()             — CLOSE #n                (valid either way)
    //
    // A `.read()`/`.eof()` on a file not opened `for input` -- or a
    // `.write()` on one not opened `for output`/`for append` -- is a
    // transpile-time error, the same way a misspelled record field is: the
    // compiler already knows which direction the file goes, so it checks
    // for you instead of failing at runtime against real data.

    // file out = open(...) for output
    bcc_files[0] = fopen(bv_s_csvfile, "w");
    // out.write(...)
    fprintf(bcc_files[0], "\"%s\",%d,\"%s\"\n", "Alice", 95, "pass");
    // out.write(...)
    fprintf(bcc_files[0], "\"%s\",%d,\"%s\"\n", "Bob", 54, "fail");
    // out.write(...)
    fprintf(bcc_files[0], "\"%s\",%d,\"%s\"\n", "Carol", 78, "pass");
    // out.close()
    fclose(bcc_files[0]);
    bcc_files[0] = NULL;

    // file appended = open(...) for append
    bcc_files[1] = fopen(bv_s_csvfile, "a");
    // appended.write(...)
    fprintf(bcc_files[1], "\"%s\",%d,\"%s\"\n", "Dave", 88, "pass");
    // appended.close()
    fclose(bcc_files[1]);
    bcc_files[1] = NULL;

    char bt_s_9[256];
    snprintf(bt_s_9, sizeof(bt_s_9), "%s%s", "Part 2 (file-handle DSL) -- all records in ", bv_s_csvfile);
    char bt_s_10[256];
    snprintf(bt_s_10, sizeof(bt_s_10), "%s%s", bt_s_9, ":");
    printf("%s\n", bt_s_10);
    // file dbFile = open(...) for input
    bcc_raise_retry_2: ;
    bcc_files[2] = fopen(bv_s_csvfile, "r");
    if (!bcc_files[2]) {
        bcc_err = 53;
        bcc_resume_id = 2;
        if (bcc_on_error_target < 0 || bcc_in_handler) {
            fprintf(stderr, "unhandled BASIC error %d\n", bcc_err);
            exit(1);
        }
        bcc_in_handler = 1;
        switch (bcc_on_error_target) {
        }
    }
    bcc_raise_after_2: ;
    while (((int)(~(long)round((double)bcc_eof(bcc_files[2]))))) {
        // dbFile.read(...)
        bcc_read_file_field(bcc_files[2], bcc_file_field_buf, sizeof(bcc_file_field_buf));
        snprintf(bv_s_name, sizeof(bv_s_name), "%s", bcc_file_field_buf);
        bcc_read_file_field(bcc_files[2], bcc_file_field_buf, sizeof(bcc_file_field_buf));
        bv_i_score = atoi(bcc_file_field_buf);
        bcc_read_file_field(bcc_files[2], bcc_file_field_buf, sizeof(bcc_file_field_buf));
        snprintf(bv_s_result, sizeof(bv_s_result), "%s", bcc_file_field_buf);
        char bt_s_11[256];
        snprintf(bt_s_11, sizeof(bt_s_11), "%s%s", "  ", bv_s_name);
        char bt_s_12[256];
        snprintf(bt_s_12, sizeof(bt_s_12), "%s%s", bt_s_11, ": ");
        char bt_s_13[256];
        snprintf(bt_s_13, sizeof(bt_s_13), "%s%s", bt_s_12, bcc_stri(bv_i_score));
        char bt_s_14[256];
        snprintf(bt_s_14, sizeof(bt_s_14), "%s%s", bt_s_13, "  [");
        char bt_s_15[256];
        snprintf(bt_s_15, sizeof(bt_s_15), "%s%s", bt_s_14, bv_s_result);
        char bt_s_16[256];
        snprintf(bt_s_16, sizeof(bt_s_16), "%s%s", bt_s_15, "]");
        printf("%s\n", bt_s_16);
    }
    // dbFile.close()
    fclose(bcc_files[2]);
    bcc_files[2] = NULL;

    // `line input`/`print #` don't have DSL sugar yet -- fall back to the
    // raw form from Part 1 for those; the DSL only replaces `open`, `write`,
    // `input`, `eof()`, and `close` so far.

    return 0;
}
