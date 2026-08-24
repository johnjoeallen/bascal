// BASCAL generated C -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
#include <stdio.h>
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
static int bcc_put_record_fields_1_0(FILE* file, long record, const char* field_0, const char* field_1, const char* field_2, const char* field_3);
static int bcc_get_record_fields_1_0(FILE* file, long record, char* field_0, char* field_1, char* field_2, char* field_3);
static int bcc_put_record_fields_1_1(FILE* file, long record, const char* field_0, const char* field_1, const char* field_2, const char* field_3);
static int bcc_get_record_fields_1_1(FILE* file, long record, char* field_0, char* field_1, char* field_2, char* field_3);
static int bcc_put_record_fields_1_2(FILE* file, long record, const char* field_0, const char* field_1, const char* field_2, const char* field_3);
static int bcc_get_record_fields_1_2(FILE* file, long record, char* field_0, char* field_1, char* field_2, char* field_3);
static int bcc_put_record_fields_1_3(FILE* file, long record, const char* field_0, const char* field_1, const char* field_2, const char* field_3);
static int bcc_get_record_fields_1_3(FILE* file, long record, char* field_0, char* field_1, char* field_2, char* field_3);
static int bcc_put_record_fields_1_4(FILE* file, long record, const char* field_0, const char* field_1, const char* field_2, const char* field_3);
static int bcc_get_record_fields_1_4(FILE* file, long record, char* field_0, char* field_1, char* field_2, char* field_3);
static int bcc_put_record_fields_1_5(FILE* file, long record, const char* field_0, const char* field_1, const char* field_2, const char* field_3);
static int bcc_get_record_fields_1_5(FILE* file, long record, char* field_0, char* field_1, char* field_2, char* field_3);
static int bcc_put_record_student(FILE* file, long record, const int16_t* field_0, const char* field_1, const double* field_2, const char* field_3);
static int bcc_get_record_student(FILE* file, long record, char* field_0, char* field_1, char* field_2, char* field_3);

static double bv_d_carolscore = 0;
static double bv_d_score = 0;
static double bv_d_sscore = 0;
static int bv_i_carolfacultytrimi = 0;
static int bv_i_carolid = 0;
static int bv_i_carolnametrimi = 0;
static int bv_i_i = 0;
static int bv_i_id = 0;
static int bv_i_numrecs = 0;
static int bv_i_reclen = 0;
static int bv_i_sfacultytrimi = 0;
static int bv_i_sid = 0;
static int bv_i_snametrimi = 0;
static char bv_s_carolfaculty[256] = {0};
static char bv_s_carolname[256] = {0};
static char bv_s_dbfacultybuf[256] = {0};
static char bv_s_dbfile[256] = {0};
static char bv_s_dbidbuf[256] = {0};
static char bv_s_dbnamebuf[256] = {0};
static char bv_s_dbscorebuf[256] = {0};
static char bv_s_facultybuf[256] = {0};
static char bv_s_idbuf[256] = {0};
static char bv_s_namebuf[256] = {0};
static char bv_s_scorebuf[256] = {0};
static char bv_s_sfaculty[256] = {0};
static char bv_s_sname[256] = {0};

void bf_s_trimmed(const char* bv_s_s_in, char* bcc_out);

void bf_s_trimmed(const char* bv_s_s_in, char* bcc_out) {
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

int main(void) {
    // Tutorial — Random-Access Files: hand-written, then with the record/file DSL
    //
    // This tutorial writes the *same* program twice. Part 1 uses BASIC's raw
    // random-access file primitives directly. Part 2 uses BASCAL's `record`/
    // `file` DSL, which transpiles to exactly the same primitives — nothing about
    // the *generated* BASIC changes, only how much of it you have to write by
    // hand. Read Part 1 first; the comments between the two parts explain what
    // the DSL is buying you and why.
    //
    // ---- Part 1 primitives ----
    //
    // open filename$ for random as #n len = recLen%
    // Open (or create) a random-access file.  len specifies the record length
    // in bytes; every record occupies exactly that many bytes.
    //
    // field #n, width1% as var1$, width2% as var2$, ...
    // Bind string variables to regions of the file buffer.  The sum of all
    // widths must equal the record length.  Only string variables may be used
    // in a FIELD statement.
    //
    // lset var$ = expr$   — copy into a field buffer, left-justified (padded)
    // rset var$ = expr$   — copy into a field buffer, right-justified (padded)
    //
    // put #n, recordNumber%   — write the current buffer as record n (1-based)
    // get #n, recordNumber%   — read record n into the buffer variables
    //
    // Packing helpers (BASIC builtins):
    // mki$(n%)  — pack a 2-byte integer into a 2-character string
    // mkl$(n&)  — pack a 4-byte long
    // mks$(n!)  — pack a 4-byte single
    // mkd$(n#)  — pack an 8-byte double
    // cvi(s$)   — unpack a 2-byte integer from a string
    // cvl(s$)   — unpack a 4-byte long
    // cvs(s$)   — unpack a 4-byte single
    // cvd(s$)   — unpack an 8-byte double
    //
    // Every MKx$ always returns a string (never a type-suffixed MKI%/MKD#/etc —
    // those aren't real MBASIC/BASCOM functions), and every CVx takes no suffix
    // at all. There's also no RTRIM$ builtin on real MBASIC/BASCOM -- trimming a
    // fixed-width, space-padded FIELD buffer back down to its real length needs
    // a hand-rolled loop, like trimmed$ below.

    // trimmed$ -- right-trim trailing spaces from a fixed-width FIELD buffer.

    bv_i_reclen = 50;
    bv_i_numrecs = 3;
    snprintf(bv_s_dbfile, sizeof(bv_s_dbfile), "%s", "tutorial_students.dat");

    // ============================================================
    // Part 1 — random-access files, written by hand
    // ============================================================

    // ---- Write three records ----

    bcc_raise_retry_0: ;
    bcc_files[0] = fopen(bv_s_dbfile, "rb+");
    if (!bcc_files[0]) bcc_files[0] = fopen(bv_s_dbfile, "wb+");
    if (!bcc_files[0]) {
        bcc_err = 75;
        bcc_resume_id = 0;
        bcc_erl = 63;
        bcc_err_file = "tutorial/15_random_and_record_files.bcl";
        if (bcc_on_error_target < 0 || bcc_in_handler) {
            fprintf(stderr, "unhandled BASIC error %d\n", bcc_err);
            exit(1);
        }
        bcc_in_handler = 1;
        switch (bcc_on_error_target) {
        }
    }
    bcc_raise_after_0: ;

    // Record 1: Alice, 95
    bcc_mki(bv_s_idbuf, 1);
    snprintf(bv_s_namebuf, sizeof(bv_s_namebuf), "%-*.*s", 20, 20, "Alice");
    bcc_mkd(bv_s_scorebuf, 95.0);
    snprintf(bv_s_facultybuf, sizeof(bv_s_facultybuf), "%-*.*s", 20, 20, "Engineering");
    bcc_put_record_fields_1_0(bcc_files[0], 1, bv_s_idbuf, bv_s_namebuf, bv_s_scorebuf, bv_s_facultybuf);

    // Record 2: Bob, 54
    bcc_mki(bv_s_idbuf, 2);
    snprintf(bv_s_namebuf, sizeof(bv_s_namebuf), "%-*.*s", 20, 20, "Bob");
    bcc_mkd(bv_s_scorebuf, 54.0);
    snprintf(bv_s_facultybuf, sizeof(bv_s_facultybuf), "%-*.*s", 20, 20, "Arts");
    bcc_put_record_fields_1_0(bcc_files[0], 2, bv_s_idbuf, bv_s_namebuf, bv_s_scorebuf, bv_s_facultybuf);

    // Record 3: Carol, 78
    bcc_mki(bv_s_idbuf, 3);
    snprintf(bv_s_namebuf, sizeof(bv_s_namebuf), "%-*.*s", 20, 20, "Carol");
    bcc_mkd(bv_s_scorebuf, 78.0);
    snprintf(bv_s_facultybuf, sizeof(bv_s_facultybuf), "%-*.*s", 20, 20, "Science");
    bcc_put_record_fields_1_0(bcc_files[0], 3, bv_s_idbuf, bv_s_namebuf, bv_s_scorebuf, bv_s_facultybuf);

    fclose(bcc_files[0]);
    bcc_files[0] = NULL;

    // ---- Read records in reverse order ----

    printf("Part 1 (hand-written) -- reading records in reverse order:\n");
    bcc_raise_retry_1: ;
    bcc_files[0] = fopen(bv_s_dbfile, "rb+");
    if (!bcc_files[0]) bcc_files[0] = fopen(bv_s_dbfile, "wb+");
    if (!bcc_files[0]) {
        bcc_err = 75;
        bcc_resume_id = 1;
        bcc_erl = 92;
        bcc_err_file = "tutorial/15_random_and_record_files.bcl";
        if (bcc_on_error_target < 0 || bcc_in_handler) {
            fprintf(stderr, "unhandled BASIC error %d\n", bcc_err);
            exit(1);
        }
        bcc_in_handler = 1;
        switch (bcc_on_error_target) {
        }
    }
    bcc_raise_after_1: ;

    int bt_lim_0 = 1;
    int bt_step_0 = -(1);
    for (bv_i_i = bv_i_numrecs; bt_step_0 >= 0 ? bv_i_i <= bt_lim_0 : bv_i_i >= bt_lim_0; bv_i_i += bt_step_0) {
        bcc_get_record_fields_1_1(bcc_files[0], bv_i_i, bv_s_idbuf, bv_s_namebuf, bv_s_scorebuf, bv_s_facultybuf);
        bv_i_id = bcc_cvi(bv_s_idbuf);
        bv_d_score = bcc_cvd(bv_s_scorebuf);
        char bt_s_1[256];
        snprintf(bt_s_1, sizeof(bt_s_1), "%s%s", "  [", bcc_stri(bv_i_id));
        char bt_s_2[256];
        snprintf(bt_s_2, sizeof(bt_s_2), "%s%s", bt_s_1, "] ");
        char bt_s_3[256];
        bf_s_trimmed(bv_s_namebuf, bt_s_3);
        char bt_s_4[256];
        snprintf(bt_s_4, sizeof(bt_s_4), "%s%s", bt_s_2, bt_s_3);
        char bt_s_5[256];
        snprintf(bt_s_5, sizeof(bt_s_5), "%s%s", bt_s_4, " -- ");
        char bt_s_6[256];
        snprintf(bt_s_6, sizeof(bt_s_6), "%s%s", bt_s_5, bcc_strd(bv_d_score));
        printf("%s\n", bt_s_6);
    }

    fclose(bcc_files[0]);
    bcc_files[0] = NULL;

    // ---- Update one field in place ----

    bcc_raise_retry_2: ;
    bcc_files[0] = fopen(bv_s_dbfile, "rb+");
    if (!bcc_files[0]) bcc_files[0] = fopen(bv_s_dbfile, "wb+");
    if (!bcc_files[0]) {
        bcc_err = 75;
        bcc_resume_id = 2;
        bcc_erl = 106;
        bcc_err_file = "tutorial/15_random_and_record_files.bcl";
        if (bcc_on_error_target < 0 || bcc_in_handler) {
            fprintf(stderr, "unhandled BASIC error %d\n", bcc_err);
            exit(1);
        }
        bcc_in_handler = 1;
        switch (bcc_on_error_target) {
        }
    }
    bcc_raise_after_2: ;

    // Bob just scraped a pass on re-mark. Only scoreBuf$ changes, but PUT
    // always writes the whole 50-byte buffer, so GET has to load the record
    // first even though idBuf$/nameBuf$/facultyBuf$ are just being written straight back
    // unchanged.
    bcc_get_record_fields_1_2(bcc_files[0], 2, bv_s_idbuf, bv_s_namebuf, bv_s_scorebuf, bv_s_facultybuf);
    bcc_mkd(bv_s_scorebuf, 61.5);
    bcc_put_record_fields_1_2(bcc_files[0], 2, bv_s_idbuf, bv_s_namebuf, bv_s_scorebuf, bv_s_facultybuf);

    fclose(bcc_files[0]);
    bcc_files[0] = NULL;

    // ---- Update two fields at once ----

    bcc_raise_retry_3: ;
    bcc_files[0] = fopen(bv_s_dbfile, "rb+");
    if (!bcc_files[0]) bcc_files[0] = fopen(bv_s_dbfile, "wb+");
    if (!bcc_files[0]) {
        bcc_err = 75;
        bcc_resume_id = 3;
        bcc_erl = 121;
        bcc_err_file = "tutorial/15_random_and_record_files.bcl";
        if (bcc_on_error_target < 0 || bcc_in_handler) {
            fprintf(stderr, "unhandled BASIC error %d\n", bcc_err);
            exit(1);
        }
        bcc_in_handler = 1;
        switch (bcc_on_error_target) {
        }
    }
    bcc_raise_after_3: ;

    // Alice got married and re-sat the exam — `name` and `score` both change,
    // `id` and `faculty` don't. Same problem as Bob's update, just with two fields instead
    // of one: GET first (this is what preserves idBuf$ and facultyBuf$), LSET the two fields
    // that actually changed, then PUT the whole buffer back. Nothing here is
    // specific to "two" fields — five changed fields would look identical,
    // just with five LSET lines between the GET and the PUT.
    bcc_get_record_fields_1_3(bcc_files[0], 1, bv_s_idbuf, bv_s_namebuf, bv_s_scorebuf, bv_s_facultybuf);
    snprintf(bv_s_namebuf, sizeof(bv_s_namebuf), "%-*.*s", 20, 20, "Alice Smith");
    bcc_mkd(bv_s_scorebuf, 91.0);
    bcc_put_record_fields_1_3(bcc_files[0], 1, bv_s_idbuf, bv_s_namebuf, bv_s_scorebuf, bv_s_facultybuf);

    fclose(bcc_files[0]);
    bcc_files[0] = NULL;

    // ---- Same shape again ----

    bcc_raise_retry_4: ;
    bcc_files[0] = fopen(bv_s_dbfile, "rb+");
    if (!bcc_files[0]) bcc_files[0] = fopen(bv_s_dbfile, "wb+");
    if (!bcc_files[0]) {
        bcc_err = 75;
        bcc_resume_id = 4;
        bcc_erl = 139;
        bcc_err_file = "tutorial/15_random_and_record_files.bcl";
        if (bcc_on_error_target < 0 || bcc_in_handler) {
            fprintf(stderr, "unhandled BASIC error %d\n", bcc_err);
            exit(1);
        }
        bcc_in_handler = 1;
        switch (bcc_on_error_target) {
        }
    }
    bcc_raise_after_4: ;

    // Carol changed her name and improved her score: the exact same
    // GET / LSET / LSET / PUT shape as Alice's update above, just retyped by
    // hand with Carol's record number and values.
    bcc_get_record_fields_1_4(bcc_files[0], 3, bv_s_idbuf, bv_s_namebuf, bv_s_scorebuf, bv_s_facultybuf);
    snprintf(bv_s_namebuf, sizeof(bv_s_namebuf), "%-*.*s", 20, 20, "Carol Jones");
    bcc_mkd(bv_s_scorebuf, 88.0);
    bcc_put_record_fields_1_4(bcc_files[0], 3, bv_s_idbuf, bv_s_namebuf, bv_s_scorebuf, bv_s_facultybuf);

    fclose(bcc_files[0]);
    bcc_files[0] = NULL;

    // ---- Verify the updates ----

    printf("Part 1 (hand-written) -- after updates:\n");
    bcc_raise_retry_5: ;
    bcc_files[0] = fopen(bv_s_dbfile, "rb+");
    if (!bcc_files[0]) bcc_files[0] = fopen(bv_s_dbfile, "wb+");
    if (!bcc_files[0]) {
        bcc_err = 75;
        bcc_resume_id = 5;
        bcc_erl = 155;
        bcc_err_file = "tutorial/15_random_and_record_files.bcl";
        if (bcc_on_error_target < 0 || bcc_in_handler) {
            fprintf(stderr, "unhandled BASIC error %d\n", bcc_err);
            exit(1);
        }
        bcc_in_handler = 1;
        switch (bcc_on_error_target) {
        }
    }
    bcc_raise_after_5: ;

    int bt_lim_7 = bv_i_numrecs;
    int bt_step_7 = 1;
    for (bv_i_i = 1; bt_step_7 >= 0 ? bv_i_i <= bt_lim_7 : bv_i_i >= bt_lim_7; bv_i_i += bt_step_7) {
        bcc_get_record_fields_1_5(bcc_files[0], bv_i_i, bv_s_idbuf, bv_s_namebuf, bv_s_scorebuf, bv_s_facultybuf);
        char bt_s_8[256];
        bf_s_trimmed(bv_s_namebuf, bt_s_8);
        char bt_s_9[256];
        snprintf(bt_s_9, sizeof(bt_s_9), "%s%s", "  ", bt_s_8);
        char bt_s_10[256];
        snprintf(bt_s_10, sizeof(bt_s_10), "%s%s", bt_s_9, ": ");
        char bt_s_11[256];
        snprintf(bt_s_11, sizeof(bt_s_11), "%s%s", bt_s_10, bcc_strd(bcc_cvd(bv_s_scorebuf)));
        printf("%s\n", bt_s_11);
    }

    fclose(bcc_files[0]);
    bcc_files[0] = NULL;

    // ------------------------------------------------------------------------
    // What Part 1 actually cost:
    //
    // - idBuf$/nameBuf$/scoreBuf$ and the FIELD statement binding them had to
    // be repeated, identically, in every OPEN block — get it wrong in one
    // of the five and you're reading or writing the wrong bytes.
    // - recLen% (50) is 2+20+8+20 computed by hand; add a field to the record
    // and every one of those numbers has to be updated together, or the
    // file silently gets corrupted.
    // - Each field's pack/unpack call (mki$/cvi, mkd$/cvd, or nothing for
    // strings) has to be matched to that field's type by hand, every time
    // it's touched — nothing stops mkd$() being used on the id field.
    // - There's no RTRIM$ builtin on real MBASIC/BASCOM, so reading a string
    // field back means hand-rolling a trim loop (trimmed$, above) and
    // remembering to call it, every time.
    // - Alice's and Carol's updates are the identical GET/LSET/LSET/PUT
    // pattern, typed out twice, with every buffer/field name repeated.
    //
    // None of this is hard, exactly — it's just bookkeeping a compiler should
    // be doing for you. Part 2 is the same program again, with BASCAL's
    // record/file DSL doing that bookkeeping.
    // ------------------------------------------------------------------------

    // ============================================================
    // Part 2 — the same program with the record / file DSL
    // ============================================================
    //
    // record <Name> ... end record
    // Declares a fixed-layout record type. Supported field types: int16,
    // int32, float32, float64, and string(N). The record's total byte width
    // (used as Part 1's recLen%) is the sum of its field widths, computed
    // automatically.
    //
    // file <var> as <RecordType> = open(<path>)
    // Opens (or creates) a random-access file sized for one record, and binds
    // FIELD buffer variables for every field. File numbers are allocated
    // automatically, starting at #1, in the order `file` declarations appear.
    // This one line replaces Part 1's recLen% constant, OPEN, and FIELD.
    //
    // <file>[<n>] = { field: value, ... }
    // Whole-record write: packs every field (LSET, MKx$ for numeric fields)
    // and writes record n. Every declared field must be given — a missing one
    // is a compile-time error.
    //
    // let <var> = <file>[<n>]
    // Whole-record read: reads record n and unpacks every field (CVx for
    // numeric fields, an inline trim loop like Part 1's trimmed$ for strings)
    // into `<var>.<field>`.
    //
    // <file>[<n>].<field> = value
    // Partial update: GET, LSET just that one field, PUT. The one-field
    // version of Part 1's Bob update, with no buffer names to get wrong.
    //
    // <file>[<n>] = ?{ field: value, ... }
    // Partial-record write: any subset of fields; unlisted ones are left
    // untouched on disk. Whether a GET is needed is decided at *compile
    // time* by comparing the given field names against the record's declared
    // fields: some fields missing -> GET first, LSET just those fields, then
    // PUT (this is Alice's update from Part 1, minus the GET/LSET/LSET/PUT
    // spelled out by hand); every field given anyway -> no GET, same as a
    // plain `{...}`. Unlike `{...}`, an *unknown* field name is still a
    // compile-time error — only *missing* fields are allowed, not misspelled
    // ones.
    //
    // let <var> = <file>[<n>]
    // <var>.<field> = value  (any number of times)
    // <file>[<n>] = <var>
    // Batched update: the `let` does one GET; each `<var>.<field> = value` is
    // a pure in-memory assignment (no I/O); the final `<file>[<n>] = <var>`
    // packs every field from `<var>` and does one PUT. This is Carol's update
    // from Part 1 — same GET/LSET/LSET/PUT shape as `?{...}`, just spelled as
    // read-mutate-write instead of a single literal, useful when the new
    // values come from more than a one-line expression.
    //
    // for <var> = <A> downto <B> ... end for
    // Sugar for `for <var> = <A> to <B> step -1`.
    //
    // <file>.close()
    // Closes the file.


    // file db as Student = open(...)  [50 bytes/record]
    bcc_raise_retry_6: ;
    bcc_files[0] = fopen("tutorial_records.dat", "rb+");
    if (!bcc_files[0]) bcc_files[0] = fopen("tutorial_records.dat", "wb+");
    if (!bcc_files[0]) {
        bcc_err = 75;
        bcc_resume_id = 6;
        bcc_erl = 252;
        bcc_err_file = "tutorial/15_random_and_record_files.bcl";
        if (bcc_on_error_target < 0 || bcc_in_handler) {
            fprintf(stderr, "unhandled BASIC error %d\n", bcc_err);
            exit(1);
        }
        bcc_in_handler = 1;
        switch (bcc_on_error_target) {
        }
    }
    bcc_raise_after_6: ;

    // ---- Write three records ----

    // Record 1: Alice, 95
    // db[...] = { ... }  (whole-record write)
    int16_t bcc_tmp_12 = 1;
    double bcc_tmp_13 = 95.0;
    bcc_put_record_student(bcc_files[0], 1, &bcc_tmp_12, "Alice", &bcc_tmp_13, "Engineering");

    // Record 2: Bob, 54
    // db[...] = { ... }  (whole-record write)
    int16_t bcc_tmp_14 = 2;
    double bcc_tmp_15 = 54.0;
    bcc_put_record_student(bcc_files[0], 2, &bcc_tmp_14, "Bob", &bcc_tmp_15, "Arts");

    // Record 3: Carol, 78
    // db[...] = { ... }  (whole-record write)
    int16_t bcc_tmp_16 = 3;
    double bcc_tmp_17 = 78.0;
    bcc_put_record_student(bcc_files[0], 3, &bcc_tmp_16, "Carol", &bcc_tmp_17, "Science");

    // ---- Read records in reverse order ----

    printf("Part 2 (record/file DSL) -- reading records in reverse order:\n");

    int bt_lim_18 = 1;
    int bt_step_18 = -1;
    for (bv_i_i = 3; bt_step_18 >= 0 ? bv_i_i <= bt_lim_18 : bv_i_i >= bt_lim_18; bv_i_i += bt_step_18) {
        // let s = db[...]  (whole-record read)
        bcc_get_record_student(bcc_files[0], bv_i_i, bv_s_dbidbuf, bv_s_dbnamebuf, bv_s_dbscorebuf, bv_s_dbfacultybuf);
        bv_i_sid = bcc_cvi(bv_s_dbidbuf);
        bv_i_snametrimi = ((int)strlen(bv_s_dbnamebuf));
        while (((-(bv_i_snametrimi > 0)) && (-(strcmp(bcc_mid(bv_s_dbnamebuf, bv_i_snametrimi, 1), " ") == 0)))) {
            bv_i_snametrimi = (bv_i_snametrimi - 1);
        }
        snprintf(bv_s_sname, sizeof(bv_s_sname), "%s", bcc_mid(bv_s_dbnamebuf, 1, bv_i_snametrimi));
        bv_d_sscore = bcc_cvd(bv_s_dbscorebuf);
        bv_i_sfacultytrimi = ((int)strlen(bv_s_dbfacultybuf));
        while (((-(bv_i_sfacultytrimi > 0)) && (-(strcmp(bcc_mid(bv_s_dbfacultybuf, bv_i_sfacultytrimi, 1), " ") == 0)))) {
            bv_i_sfacultytrimi = (bv_i_sfacultytrimi - 1);
        }
        snprintf(bv_s_sfaculty, sizeof(bv_s_sfaculty), "%s", bcc_mid(bv_s_dbfacultybuf, 1, bv_i_sfacultytrimi));
        char bt_s_19[256];
        snprintf(bt_s_19, sizeof(bt_s_19), "%s%s", "  [", bcc_stri(bv_i_sid));
        char bt_s_20[256];
        snprintf(bt_s_20, sizeof(bt_s_20), "%s%s", bt_s_19, "] ");
        char bt_s_21[256];
        snprintf(bt_s_21, sizeof(bt_s_21), "%s%s", bt_s_20, bv_s_sname);
        char bt_s_22[256];
        snprintf(bt_s_22, sizeof(bt_s_22), "%s%s", bt_s_21, " -- ");
        char bt_s_23[256];
        snprintf(bt_s_23, sizeof(bt_s_23), "%s%s", bt_s_22, bcc_strd(bv_d_sscore));
        printf("%s\n", bt_s_23);
    }

    // ---- Update one field in place ----

    // Bob just scraped a pass on re-mark. Compare to Part 1: no recLen%, no
    // idBuf$/nameBuf$/scoreBuf$/facultyBuf$, no mkd$() — just the field that's changing.
    // db[...].score = ...  (partial-field update)
    double bcc_tmp_24 = 61.5;
    if (!bcc_put_record_student(bcc_files[0], 2, NULL, NULL, &bcc_tmp_24, NULL)) { fprintf(stderr, "BASCAL: record %ld does not exist\n", (long)2); exit(1); }

    // ---- Update two fields at once, still one GET and one PUT ----

    // Alice got married and re-sat the exam. `name` and `score` don't cover
    // every field of Student, so this needs an implicit GET first (id and
    // faculty are preserved from the existing record) -- exactly Part 1's GET / LSET /
    // LSET / PUT for Alice, minus having to write out the GET, the buffer
    // names, or the packing calls. Which fields need a GET is worked out by
    // the compiler by comparing `name`/`score` against Student's declared
    // fields — not decided at runtime.
    // db[...] = ?{ ... }  (partial-record write)
    double bcc_tmp_25 = 91.0;
    if (!bcc_put_record_student(bcc_files[0], 1, NULL, "Alice Smith", &bcc_tmp_25, NULL)) { fprintf(stderr, "BASCAL: record %ld does not exist\n", (long)1); exit(1); }

    // ---- Batched update: read once, mutate twice, write back once ----

    // Carol changed her name and improved her score — the read-mutate-write
    // spelling of the same one-GET-one-PUT update, useful when the new values
    // aren't just a couple of literals.
    // let carol = db[...]  (whole-record read)
    bcc_get_record_student(bcc_files[0], 3, bv_s_dbidbuf, bv_s_dbnamebuf, bv_s_dbscorebuf, bv_s_dbfacultybuf);
    bv_i_carolid = bcc_cvi(bv_s_dbidbuf);
    bv_i_carolnametrimi = ((int)strlen(bv_s_dbnamebuf));
    while (((-(bv_i_carolnametrimi > 0)) && (-(strcmp(bcc_mid(bv_s_dbnamebuf, bv_i_carolnametrimi, 1), " ") == 0)))) {
        bv_i_carolnametrimi = (bv_i_carolnametrimi - 1);
    }
    snprintf(bv_s_carolname, sizeof(bv_s_carolname), "%s", bcc_mid(bv_s_dbnamebuf, 1, bv_i_carolnametrimi));
    bv_d_carolscore = bcc_cvd(bv_s_dbscorebuf);
    bv_i_carolfacultytrimi = ((int)strlen(bv_s_dbfacultybuf));
    while (((-(bv_i_carolfacultytrimi > 0)) && (-(strcmp(bcc_mid(bv_s_dbfacultybuf, bv_i_carolfacultytrimi, 1), " ") == 0)))) {
        bv_i_carolfacultytrimi = (bv_i_carolfacultytrimi - 1);
    }
    snprintf(bv_s_carolfaculty, sizeof(bv_s_carolfaculty), "%s", bcc_mid(bv_s_dbfacultybuf, 1, bv_i_carolfacultytrimi));
    snprintf(bv_s_carolname, sizeof(bv_s_carolname), "%s", "Carol Jones");
    bv_d_carolscore = 88.0;
    // db[...] = carol  (write back a let-bound record)
    int16_t bcc_tmp_26 = bv_i_carolid;
    double bcc_tmp_27 = bv_d_carolscore;
    bcc_put_record_student(bcc_files[0], 3, &bcc_tmp_26, bv_s_carolname, &bcc_tmp_27, bv_s_carolfaculty);

    // ---- Verify the updates ----

    printf("Part 2 (record/file DSL) -- after updates:\n");

    int bt_lim_28 = 3;
    int bt_step_28 = 1;
    for (bv_i_i = 1; bt_step_28 >= 0 ? bv_i_i <= bt_lim_28 : bv_i_i >= bt_lim_28; bv_i_i += bt_step_28) {
        // let s = db[...]  (whole-record read)
        bcc_get_record_student(bcc_files[0], bv_i_i, bv_s_dbidbuf, bv_s_dbnamebuf, bv_s_dbscorebuf, bv_s_dbfacultybuf);
        bv_i_sid = bcc_cvi(bv_s_dbidbuf);
        bv_i_snametrimi = ((int)strlen(bv_s_dbnamebuf));
        while (((-(bv_i_snametrimi > 0)) && (-(strcmp(bcc_mid(bv_s_dbnamebuf, bv_i_snametrimi, 1), " ") == 0)))) {
            bv_i_snametrimi = (bv_i_snametrimi - 1);
        }
        snprintf(bv_s_sname, sizeof(bv_s_sname), "%s", bcc_mid(bv_s_dbnamebuf, 1, bv_i_snametrimi));
        bv_d_sscore = bcc_cvd(bv_s_dbscorebuf);
        bv_i_sfacultytrimi = ((int)strlen(bv_s_dbfacultybuf));
        while (((-(bv_i_sfacultytrimi > 0)) && (-(strcmp(bcc_mid(bv_s_dbfacultybuf, bv_i_sfacultytrimi, 1), " ") == 0)))) {
            bv_i_sfacultytrimi = (bv_i_sfacultytrimi - 1);
        }
        snprintf(bv_s_sfaculty, sizeof(bv_s_sfaculty), "%s", bcc_mid(bv_s_dbfacultybuf, 1, bv_i_sfacultytrimi));
        char bt_s_29[256];
        snprintf(bt_s_29, sizeof(bt_s_29), "%s%s", "  ", bv_s_sname);
        char bt_s_30[256];
        snprintf(bt_s_30, sizeof(bt_s_30), "%s%s", bt_s_29, ": ");
        char bt_s_31[256];
        snprintf(bt_s_31, sizeof(bt_s_31), "%s%s", bt_s_30, bcc_strd(bv_d_sscore));
        printf("%s\n", bt_s_31);
    }

    // db.close()
    fclose(bcc_files[0]);
    bcc_files[0] = NULL;

    // ------------------------------------------------------------------------
    // Part 2 is the same three writes, the same reverse-order read, and the
    // same three updates as Part 1 — Alice's and Bob's and Carol's updates
    // still transpile to exactly one GET and one PUT each, nothing runs slower.
    // What's gone is everything that was bookkeeping rather than logic: the
    // hand-computed record width, the repeated buffer-variable/FIELD
    // boilerplate in every block, the pack/unpack call picked by hand per
    // field, and the GET-or-not decision for a partial write, which the
    // compiler now makes for you at compile time by simply comparing field
    // names -- get a field name wrong (`db[1] = ?{ nmae: ... }`) and it's a
    // compile error instead of a silently corrupted record.
    // ------------------------------------------------------------------------

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

static int bcc_put_record_fields_1_0(FILE* file, long record, const char* field_0, const char* field_1, const char* field_2, const char* field_3) {
    unsigned char buffer[50];
    memcpy(buffer + 0, field_0, 2);
    memcpy(buffer + 2, field_1, 20);
    memcpy(buffer + 22, field_2, 8);
    memcpy(buffer + 30, field_3, 20);
    bcc_write_record(file, buffer, 50, record);
    return 1;
}

static int bcc_get_record_fields_1_0(FILE* file, long record, char* field_0, char* field_1, char* field_2, char* field_3) {
    unsigned char buffer[50];
    if (!bcc_read_record(file, buffer, 50, record)) return 0;
    memcpy(field_0, buffer + 0, 2);
    field_0[2] = 0;
    bcc_read_string_field(field_1, buffer + 2, 20);
    memcpy(field_2, buffer + 22, 8);
    field_2[8] = 0;
    bcc_read_string_field(field_3, buffer + 30, 20);
    return 1;
}

static int bcc_put_record_fields_1_1(FILE* file, long record, const char* field_0, const char* field_1, const char* field_2, const char* field_3) {
    unsigned char buffer[50];
    memcpy(buffer + 0, field_0, 2);
    memcpy(buffer + 2, field_1, 20);
    memcpy(buffer + 22, field_2, 8);
    memcpy(buffer + 30, field_3, 20);
    bcc_write_record(file, buffer, 50, record);
    return 1;
}

static int bcc_get_record_fields_1_1(FILE* file, long record, char* field_0, char* field_1, char* field_2, char* field_3) {
    unsigned char buffer[50];
    if (!bcc_read_record(file, buffer, 50, record)) return 0;
    memcpy(field_0, buffer + 0, 2);
    field_0[2] = 0;
    bcc_read_string_field(field_1, buffer + 2, 20);
    memcpy(field_2, buffer + 22, 8);
    field_2[8] = 0;
    bcc_read_string_field(field_3, buffer + 30, 20);
    return 1;
}

static int bcc_put_record_fields_1_2(FILE* file, long record, const char* field_0, const char* field_1, const char* field_2, const char* field_3) {
    unsigned char buffer[50];
    memcpy(buffer + 0, field_0, 2);
    memcpy(buffer + 2, field_1, 20);
    memcpy(buffer + 22, field_2, 8);
    memcpy(buffer + 30, field_3, 20);
    bcc_write_record(file, buffer, 50, record);
    return 1;
}

static int bcc_get_record_fields_1_2(FILE* file, long record, char* field_0, char* field_1, char* field_2, char* field_3) {
    unsigned char buffer[50];
    if (!bcc_read_record(file, buffer, 50, record)) return 0;
    memcpy(field_0, buffer + 0, 2);
    field_0[2] = 0;
    bcc_read_string_field(field_1, buffer + 2, 20);
    memcpy(field_2, buffer + 22, 8);
    field_2[8] = 0;
    bcc_read_string_field(field_3, buffer + 30, 20);
    return 1;
}

static int bcc_put_record_fields_1_3(FILE* file, long record, const char* field_0, const char* field_1, const char* field_2, const char* field_3) {
    unsigned char buffer[50];
    memcpy(buffer + 0, field_0, 2);
    memcpy(buffer + 2, field_1, 20);
    memcpy(buffer + 22, field_2, 8);
    memcpy(buffer + 30, field_3, 20);
    bcc_write_record(file, buffer, 50, record);
    return 1;
}

static int bcc_get_record_fields_1_3(FILE* file, long record, char* field_0, char* field_1, char* field_2, char* field_3) {
    unsigned char buffer[50];
    if (!bcc_read_record(file, buffer, 50, record)) return 0;
    memcpy(field_0, buffer + 0, 2);
    field_0[2] = 0;
    bcc_read_string_field(field_1, buffer + 2, 20);
    memcpy(field_2, buffer + 22, 8);
    field_2[8] = 0;
    bcc_read_string_field(field_3, buffer + 30, 20);
    return 1;
}

static int bcc_put_record_fields_1_4(FILE* file, long record, const char* field_0, const char* field_1, const char* field_2, const char* field_3) {
    unsigned char buffer[50];
    memcpy(buffer + 0, field_0, 2);
    memcpy(buffer + 2, field_1, 20);
    memcpy(buffer + 22, field_2, 8);
    memcpy(buffer + 30, field_3, 20);
    bcc_write_record(file, buffer, 50, record);
    return 1;
}

static int bcc_get_record_fields_1_4(FILE* file, long record, char* field_0, char* field_1, char* field_2, char* field_3) {
    unsigned char buffer[50];
    if (!bcc_read_record(file, buffer, 50, record)) return 0;
    memcpy(field_0, buffer + 0, 2);
    field_0[2] = 0;
    bcc_read_string_field(field_1, buffer + 2, 20);
    memcpy(field_2, buffer + 22, 8);
    field_2[8] = 0;
    bcc_read_string_field(field_3, buffer + 30, 20);
    return 1;
}

static int bcc_put_record_fields_1_5(FILE* file, long record, const char* field_0, const char* field_1, const char* field_2, const char* field_3) {
    unsigned char buffer[50];
    memcpy(buffer + 0, field_0, 2);
    memcpy(buffer + 2, field_1, 20);
    memcpy(buffer + 22, field_2, 8);
    memcpy(buffer + 30, field_3, 20);
    bcc_write_record(file, buffer, 50, record);
    return 1;
}

static int bcc_get_record_fields_1_5(FILE* file, long record, char* field_0, char* field_1, char* field_2, char* field_3) {
    unsigned char buffer[50];
    if (!bcc_read_record(file, buffer, 50, record)) return 0;
    memcpy(field_0, buffer + 0, 2);
    field_0[2] = 0;
    bcc_read_string_field(field_1, buffer + 2, 20);
    memcpy(field_2, buffer + 22, 8);
    field_2[8] = 0;
    bcc_read_string_field(field_3, buffer + 30, 20);
    return 1;
}

static int bcc_put_record_student(FILE* file, long record, const int16_t* field_0, const char* field_1, const double* field_2, const char* field_3) {
    unsigned char buffer[50];
    if ((!field_0 || !field_1 || !field_2 || !field_3) && !bcc_read_record(file, buffer, 50, record)) return 0;
    (void)(field_0 && memcpy(buffer + 0, field_0, 2));
    if (field_1) bcc_pad_string_field(buffer + 2, field_1, 20);
    (void)(field_2 && memcpy(buffer + 22, field_2, 8));
    if (field_3) bcc_pad_string_field(buffer + 30, field_3, 20);
    bcc_write_record(file, buffer, 50, record);
    return 1;
}

static int bcc_get_record_student(FILE* file, long record, char* field_0, char* field_1, char* field_2, char* field_3) {
    unsigned char buffer[50];
    if (!bcc_read_record(file, buffer, 50, record)) return 0;
    memcpy(field_0, buffer + 0, 2);
    field_0[2] = 0;
    bcc_read_string_field(field_1, buffer + 2, 20);
    memcpy(field_2, buffer + 22, 8);
    field_2[8] = 0;
    bcc_read_string_field(field_3, buffer + 30, 20);
    return 1;
}

