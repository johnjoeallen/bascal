[Home](../../) / [Tutorials](../) / File Input and Output

<div class="prose" markdown="1">

Sequential files use `open filename$ for input|output|append as #n` and `close #n`. `write #n, ...` writes quoted, comma-separated values that `input #n, ...` can read back exactly; `print #n, ...` writes without the quoting; `line input #n, var$` reads one whole line; `eof(n)` is non-zero once the file is exhausted.

</div>

<div class="snippet" markdown="1">

### Write, append, then read it all back

```bascal
open csvFile$ for output as #1
write #1, "Alice", 95, "pass"
write #1, "Bob",   54, "fail"
close #1

open csvFile$ for input as #1
while eof(1) = 0
    input #1, name$, score%, result$
    print "  " + name$ + ": " + str$(score%) + "  [" + result$ + "]"
end while
close #1
```

</div>

<div class="prose" markdown="1">

The file-handle DSL sugars over exactly this — `#1` and friends never appear in the source, and the compiler rejects a `.read()`/`.eof()` on a file not opened `for input`, or a `.write()` on one not opened `for output`/`for append`, at transpile time rather than letting it fail against a real file.

</div>

<div class="snippet" markdown="1">

### The same write/read, through a file handle

```bascal
file out = open(csvFile$) for output
out.write("Alice", 95, "pass")
out.write("Bob", 54, "fail")
out.close()

file scores = open(csvFile$) for input
while not scores.eof()
    scores.read(name$, score%, result$)
    print "  " + name$ + ": " + str$(score%) + "  [" + result$ + "]"
end while
scores.close()
```

</div>



[← Data, Read, Restore, Swap](data.md)  ·  [Screen I/O →](screen.md)


<!-- BEGIN generated tutorial source -->

<details class="source-embed" markdown="1">

<summary><code>tutorial/files.bcl</code></summary>



```bascal

// Tutorial — File Input and Output
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
//                           (produces data that input # can read back)
// line input #n, var$     — read one complete line into var$
// input #n, var[, ...]    — read comma-delimited values (matches write)
// eof(n)                  — returns non-zero when file n is exhausted
program files

csvFile$ = "tutorial_scores.csv"

/* ============================================================ */
/* Part 1 — sequential files, written by hand                   */
/* ============================================================ */

/* Write three records */
open csvFile$ for output as #1
write #1, "Alice", 95, "pass"
write #1, "Bob",   54, "fail"
write #1, "Carol", 78, "pass"
close #1

/* Append a fourth record */
open csvFile$ for append as #1
write #1, "Dave", 88, "pass"
close #1

/* Read and print every record */
print "Part 1 (hand-written) -- all records in " + csvFile$ + ":"
open csvFile$ for input as #1
while eof(1) = 0
    input #1, name$, score%, result$
    print "  " + name$ + ": " + str$(score%) + "  [" + result$ + "]"
end while
close #1

/* Read the file line by line using line input */
print "Part 1 (hand-written) -- raw lines:"
open csvFile$ for input as #1
while eof(1) = 0
    line input #1, line$
    print "  " + line$
end while
close #1

/* ============================================================ */
/* Part 2 — the same file, through the file-handle DSL           */
/* ============================================================ */

// file <var> = open(<path>) for output/input/append
//   Opens a file the same way `open ... for ... as #n` does, except the
//   compiler picks the channel number for you and remembers it under
//   <var> — no #1/#2 to keep straight by hand, and no risk of two open
//   files quietly sharing a number.
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

file out = open(csvFile$) for output
out.write("Alice", 95, "pass")
out.write("Bob", 54, "fail")
out.write("Carol", 78, "pass")
out.close()

file appended = open(csvFile$) for append
appended.write("Dave", 88, "pass")
appended.close()

print "Part 2 (file-handle DSL) -- all records in " + csvFile$ + ":"
file dbFile = open(csvFile$) for input
while not dbFile.eof()
    dbFile.read(name$, score%, result$)
    print "  " + name$ + ": " + str$(score%) + "  [" + result$ + "]"
end while
dbFile.close()

// `line input`/`print #` don't have DSL sugar yet -- fall back to the
// raw form from Part 1 for those; the DSL only replaces `open`, `write`,
// `input`, `eof()`, and `close` so far.

end

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/files.bas</code></summary>



```basic

10 ' BASCAL generated BASIC -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Tutorial — File Input and Output
40 '
50 ' This tutorial writes the *same* sequential file twice. Part 1 uses raw
60 ' BASIC file statements directly. Part 2 uses BASCAL's file-handle DSL,
70 ' which transpiles to exactly the same primitives, minus having to pick a
80 ' channel number and repeat it at every open/write/read/close. Read Part
90 ' 1 first; the comments in Part 2 explain what the DSL is buying you.
100 '
110 ' ---- Part 1 primitives ----
120 '
130 ' open filename$ for input  as #n   — read existing file
140 ' open filename$ for output as #n   — create or overwrite
150 ' open filename$ for append as #n   — add to end of existing file
160 ' close #n                          — flush and release the file
170 '
180 ' print #n, expr[, ...]   — write values separated by spaces
190 ' write #n, expr[, ...]   — write quoted strings, comma-separated
200 ' (produces data that input # can read back)
210 ' line input #n, var$     — read one complete line into var$
220 ' input #n, var[, ...]    — read comma-delimited values (matches write)
230 ' eof(n)                  — returns non-zero when file n is exhausted

240 csvfile$ = "tutorial_scores.csv"

250 ' ============================================================
260 ' Part 1 — sequential files, written by hand
270 ' ============================================================

280 ' Write three records
290 OPEN csvfile$ FOR OUTPUT AS #1
300 WRITE #1, "Alice", 95, "pass"
310 WRITE #1, "Bob", 54, "fail"
320 WRITE #1, "Carol", 78, "pass"
330 CLOSE #1

340 ' Append a fourth record
350 OPEN csvfile$ FOR APPEND AS #1
360 WRITE #1, "Dave", 88, "pass"
370 CLOSE #1

380 ' Read and print every record
390 PRINT ("Part 1 (hand-written) -- all records in " + csvfile$) + ":"
400 OPEN csvfile$ FOR INPUT AS #1
410 IF (EOF(1) = 0) = 0 THEN GOTO 450
420     INPUT #1, name$, score%, result$
430     PRINT ((((("  " + name$) + ": ") + STR$(score%)) + "  [") + result$) + "]"
440     GOTO 410
450 REM END WHILE
460 CLOSE #1

470 ' Read the file line by line using line input
480 PRINT "Part 1 (hand-written) -- raw lines:"
490 OPEN csvfile$ FOR INPUT AS #1
500 IF (EOF(1) = 0) = 0 THEN GOTO 540
510     LINE INPUT #1, line$
520     PRINT "  " + line$
530     GOTO 500
540 REM END WHILE
550 CLOSE #1

560 ' ============================================================
570 ' Part 2 — the same file, through the file-handle DSL
580 ' ============================================================

590 ' file <var> = open(<path>) for output/input/append
600 ' Opens a file the same way `open ... for ... as #n` does, except the
610 ' compiler picks the channel number for you and remembers it under
620 ' <var> — no #1/#2 to keep straight by hand, and no risk of two open
630 ' files quietly sharing a number.
640 '
650 ' <var>.write(expr, ...)   — WRITE #n, expr, ...
660 ' <var>.read(var, ...)     — INPUT #n, var, ...     (only valid `for input`)
670 ' <var>.eof()               — EOF(n) <> 0             (only valid `for input`)
680 ' <var>.close()             — CLOSE #n                (valid either way)
690 '
700 ' A `.read()`/`.eof()` on a file not opened `for input` -- or a
710 ' `.write()` on one not opened `for output`/`for append` -- is a
720 ' transpile-time error, the same way a misspelled record field is: the
730 ' compiler already knows which direction the file goes, so it checks
740 ' for you instead of failing at runtime against real data.

750 ' file out = open(...) for output
760 OPEN csvfile$ FOR OUTPUT AS #1
770 ' out.write(...)
780 WRITE #1, "Alice", 95, "pass"
790 ' out.write(...)
800 WRITE #1, "Bob", 54, "fail"
810 ' out.write(...)
820 WRITE #1, "Carol", 78, "pass"
830 ' out.close()
840 CLOSE #1

850 ' file appended = open(...) for append
860 OPEN csvfile$ FOR APPEND AS #2
870 ' appended.write(...)
880 WRITE #2, "Dave", 88, "pass"
890 ' appended.close()
900 CLOSE #2

910 PRINT ("Part 2 (file-handle DSL) -- all records in " + csvfile$) + ":"
920 ' file dbFile = open(...) for input
930 OPEN csvfile$ FOR INPUT AS #3
940 IF (NOT (EOF(3))) = 0 THEN GOTO 990
950     ' dbFile.read(...)
960     INPUT #3, name$, score%, result$
970     PRINT ((((("  " + name$) + ": ") + STR$(score%)) + "  [") + result$) + "]"
980     GOTO 940
990 REM END WHILE
1000 ' dbFile.close()
1010 CLOSE #3

1020 ' `line input`/`print #` don't have DSL sugar yet -- fall back to the
1030 ' raw form from Part 1 for those; the DSL only replaces `open`, `write`,
1040 ' `input`, `eof()`, and `close` so far.

1050 END

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/files.c</code></summary>



```c

// BASCAL generated C -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
#include <stdio.h>
#include <math.h>
#include <string.h>
#include <stdint.h>
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

#define BCC_MAX_CHANNELS 32
static FILE* bcc_files[BCC_MAX_CHANNELS];

static char bcc_file_field_buf[256];

static char* bcc_strbuf_take(void);
static const char* bcc_mid(const char* s, int start, int length);
static const char* bcc_chr(int code);
static const char* bcc_stri(int value);
static const char* bcc_strd(double value);
static void bcc_read_string_field(char* field, const unsigned char* source, size_t width);
static void bcc_mki(char* out, int value);
static void bcc_mkl(char* out, int value);
static void bcc_mks(char* out, double value);
static void bcc_mkd(char* out, double value);
static int bcc_cvi(const char* s);
static int bcc_cvl(const char* s);
static float bcc_cvs(const char* s);
static double bcc_cvd(const char* s);
static int bcc_read_record(FILE* file, void* buffer, size_t reclen, long record);
static void bcc_write_record(FILE* file, const void* buffer, size_t reclen, long record);
static void bcc_pad_string_field(unsigned char* dest, const char* value, size_t width);
static int bcc_eof(FILE* file);
static void bcc_line_input_file(FILE* file, char* buf, size_t bufsize);
static void bcc_read_file_field(FILE* file, char* buf, size_t bufsize);

static int bv_i_score = 0;
static char bv_s_csvfile[256] = {0};
static char bv_s_line[256] = {0};
static char bv_s_name[256] = {0};
static char bv_s_result[256] = {0};

int main(void) {
    // Tutorial — File Input and Output
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
        bcc_erl = 44;
        bcc_err_file = "tutorial/files.bcl";
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
        bcc_erl = 53;
        bcc_err_file = "tutorial/files.bcl";
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
        bcc_erl = 92;
        bcc_err_file = "tutorial/files.bcl";
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

static void bcc_read_string_field(char* field, const unsigned char* source, size_t width) {
    memcpy(field, source, width);
    field[width] = 0;
    while (width > 0 && field[width - 1] == ' ') field[--width] = 0;
}

static void bcc_mki(char* out, int value) {
    int16_t v = (int16_t)value;
    memcpy(out, &v, 2);
}

static void bcc_mkl(char* out, int value) {
    int32_t v = (int32_t)value;
    memcpy(out, &v, 4);
}

static void bcc_mks(char* out, double value) {
    float v = (float)value;
    memcpy(out, &v, 4);
}

static void bcc_mkd(char* out, double value) {
    memcpy(out, &value, 8);
}

static int bcc_cvi(const char* s) {
    int16_t v;
    memcpy(&v, s, 2);
    return (int)v;
}

static int bcc_cvl(const char* s) {
    int32_t v;
    memcpy(&v, s, 4);
    return (int)v;
}

static float bcc_cvs(const char* s) {
    float v;
    memcpy(&v, s, 4);
    return v;
}

static double bcc_cvd(const char* s) {
    double v;
    memcpy(&v, s, 8);
    return v;
}

static int bcc_read_record(FILE* file, void* buffer, size_t reclen, long record) {
    if (fseek(file, (record - 1) * (long)reclen, SEEK_SET) != 0) return 0;
    return fread(buffer, 1, reclen, file) == reclen;
}

static void bcc_write_record(FILE* file, const void* buffer, size_t reclen, long record) {
    fseek(file, (record - 1) * (long)reclen, SEEK_SET);
    fwrite(buffer, 1, reclen, file);
}

static void bcc_pad_string_field(unsigned char* dest, const char* value, size_t width) {
    size_t len = strlen(value);
    if (len > width) len = width;
    memcpy(dest, value, len);
    memset(dest + len, ' ', width - len);
}

static int bcc_eof(FILE* file) {
    int c = fgetc(file);
    if (c == EOF) return -1;
    ungetc(c, file);
    return 0;
}

static void bcc_line_input_file(FILE* file, char* buf, size_t bufsize) {
    if (fgets(buf, (int)bufsize, file) == NULL) {
        buf[0] = 0;
        return;
    }
    buf[strcspn(buf, "\r\n")] = 0;
}

static void bcc_read_file_field(FILE* file, char* buf, size_t bufsize) {
    int c = fgetc(file);
    while (c == ' ') c = fgetc(file);
    size_t len = 0;
    if (c == '"') {
        c = fgetc(file);
        while (c != EOF && c != '"') {
            if (len + 1 < bufsize) buf[len++] = (char)c;
            c = fgetc(file);
        }
        c = fgetc(file);
        while (c != EOF && c != ',' && c != '\n') c = fgetc(file);
    } else {
        while (c != EOF && c != ',' && c != '\n' && c != '\r') {
            if (len + 1 < bufsize) buf[len++] = (char)c;
            c = fgetc(file);
        }
        if (c == '\r') {
            int c2 = fgetc(file);
            if (c2 != '\n' && c2 != EOF) ungetc(c2, file);
        }
    }
    buf[len] = 0;
}


```



</details>

<!-- END generated tutorial source -->
