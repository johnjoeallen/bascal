#include <stdio.h>
#include <string.h>
#include <stdint.h>
#include <stdlib.h>

#define BCC_MAX_GOSUB_DEPTH 64
static int bcc_gosub_stack[BCC_MAX_GOSUB_DEPTH];
static int bcc_gosub_sp = 0;

static int bcc_err = 0;
static int bcc_on_error_target = -1;
static int bcc_in_handler = 0;
static int bcc_resume_id = -1;

#define BCC_DATA_COUNT 2
static const char* bcc_data[BCC_DATA_COUNT] = { "France", "Japan" };

static int bcc_data_ptr = 0;

static const char* bcc_read_data(void) {
    if (bcc_data_ptr >= BCC_DATA_COUNT) {
        fprintf(stderr, "Out of DATA\n");
        exit(1);
    }
    return bcc_data[bcc_data_ptr++];
}

#define BCC_MAX_CHANNELS 32
static FILE* bcc_files[BCC_MAX_CHANNELS];

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

static char bcc_file_field_buf[256];

static char bv_s_filename[256] = {0};
static char bv_s_firstcountry[256] = {0};
static char bv_s_secondcountry[256] = {0};

int main(void) {
    // Tutorial 17 — Labels and Error Handling
    //
    // BASCAL manages line numbers itself -- goto, gosub, on error goto, resume,
    // restore, and on ... goto / on ... gosub can never target a raw line
    // number in .bcl source. Every one of them requires a name: label instead;
    // the compiler assigns the real BASIC line number when it renders output,
    // the same way it already numbers the branch targets inside if/while/do/
    // select case.
    //
    // on error goto 0 is the one numeric exception -- 0 isn't a line number,
    // it's the sentinel that disables the error trap.

    // ---- goto / label basics ----

    printf("goto/label basics:\n");
    goto bcc_lbl_afterskip;
    printf("  not reached\n");
    bcc_lbl_afterskip:;
    printf("  reached via goto\n");

    // ---- gosub / return (BASIC-level subroutine, distinct from BASCAL functions) ----

    printf("gosub/return:\n");
    bcc_gosub_stack[bcc_gosub_sp++] = 0;
    goto bcc_lbl_printbanner;
    bcc_ret_0:;
    printf("  back after gosub\n");
    goto bcc_lbl_afterbanner;

    bcc_lbl_printbanner:;
    printf("  inside the gosub'd subroutine\n");
    switch (bcc_gosub_stack[--bcc_gosub_sp]) {
        case 0: goto bcc_ret_0;
    }

    bcc_lbl_afterbanner:;

    // ---- error handling: on error goto, resume to a label, err ----
    //
    // Opening a file that doesn't exist raises BASIC runtime error 53
    // ("file not found"). The handler below catches it, prints a message, and
    // then RESUMEs at a label -- not the failing statement or "next", but a
    // specific point past the whole try/handler region. RESUME (not a plain
    // GOTO) is what clears the runtime's "currently handling an error" state,
    // so a later error can still be trapped.

    printf("error handling, missing file:\n");
    snprintf(bv_s_filename, sizeof(bv_s_filename), "%s", "does_not_exist.dat");
    bcc_on_error_target = 0;
    bcc_raise_retry_0: ;
    bcc_files[0] = fopen(bv_s_filename, "r");
    if (!bcc_files[0]) {
        bcc_err = 53;
        bcc_resume_id = 0;
        if (bcc_on_error_target < 0 || bcc_in_handler) {
            fprintf(stderr, "unhandled BASIC error %d\n", bcc_err);
            exit(1);
        }
        bcc_in_handler = 1;
        switch (bcc_on_error_target) {
            case 0: goto bcc_lbl_handleopenerror;
        }
    }
    bcc_raise_after_0: ;
    printf("  file opened (unexpected)\n");
    fclose(bcc_files[0]);
    bcc_files[0] = NULL;
    goto bcc_lbl_afteropen;

    bcc_lbl_handleopenerror:;
    if ((-(bcc_err == 53))) {
        printf("  caught error %d: %s not found\n", bcc_err, bv_s_filename);
        bcc_in_handler = 0;
        goto bcc_lbl_afteropen;
    } else {
        printf("  unexpected error %d\n", bcc_err);
        bcc_raise_retry_1: ;
        bcc_err = bcc_err;
        bcc_resume_id = 1;
        if (bcc_on_error_target < 0 || bcc_in_handler) {
            fprintf(stderr, "unhandled BASIC error %d\n", bcc_err);
            exit(1);
        }
        bcc_in_handler = 1;
        switch (bcc_on_error_target) {
            case 0: goto bcc_lbl_handleopenerror;
        }
        bcc_raise_after_1: ;
    }

    bcc_lbl_afteropen:;
    bcc_on_error_target = -1;

    // ---- restore with a label: rewind the DATA pointer to a specific block ----

    printf("restore to a label:\n");
    snprintf(bv_s_firstcountry, sizeof(bv_s_firstcountry), "%s", bcc_read_data());
    printf("  first read: %s\n", bv_s_firstcountry);
    bcc_data_ptr = 1;
    snprintf(bv_s_secondcountry, sizeof(bv_s_secondcountry), "%s", bcc_read_data());
    printf("  after restore secondBatch: %s\n", bv_s_secondcountry);

    return 0;


    bcc_lbl_secondbatch:;
    return 0;
}
