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

static char bcc_file_field_buf[256];

static char bcc_input_buf[256];

static char* bcc_strbuf_take(void);
static const char* bcc_mid(const char* s, int start, int length);
static const char* bcc_chr(int code);
static const char* bcc_stri(int value);
static const char* bcc_strd(double value);
static void bcc_read_string_field(char* field, const unsigned char* source, size_t width);
static void bcc_mki(char* out, int value);
static void bcc_mkl(char* out, int value);
static int bcc_cvi(const char* s);
static int bcc_cvl(const char* s);
static int bcc_read_record(FILE* file, void* buffer, size_t reclen, long record);
static void bcc_write_record(FILE* file, const void* buffer, size_t reclen, long record);
static void bcc_pad_string_field(unsigned char* dest, const char* value, size_t width);
static int bcc_put_record_header(FILE* file, long record, const int16_t* field_0, const char* field_1);
static int bcc_get_record_header(FILE* file, long record, char* field_0, char* field_1);
static int bcc_put_record_entry(FILE* file, long record, const char* field_0, const char* field_1, const char* field_2);
static int bcc_get_record_entry(FILE* file, long record, char* field_0, char* field_1, char* field_2);
static void bcc_mks(char* out, double value);
static void bcc_mkd(char* out, double value);
static float bcc_cvs(const char* s);
static double bcc_cvd(const char* s);
static int bcc_eof(FILE* file);
static void bcc_line_input_file(FILE* file, char* buf, size_t bufsize);
static void bcc_read_file_field(FILE* file, char* buf, size_t bufsize);
static void bcc_read_line(void);

static int bv_i_last_slot = 0;
static char bv_s_catalogauthorbuf[256] = {0};
static char bv_s_catalogsubjectbuf[256] = {0};
static char bv_s_catalogtitlebuf[256] = {0};
static char bv_s_headerreservedbuf[256] = {0};
static char bv_s_headersizebuf[256] = {0};

void bf_s_ltrim_s(const char* bv_s_self_in, char* bcc_out);
void bf_s_rtrim_s(const char* bv_s_self_in, char* bcc_out);
void bf_i_initcatalog(void);
void bf_i_additem(const char* bv_s_author_in, const char* bv_s_title_in, const char* bv_s_subject_in);
void bf_i_listall(void);
void bf_i_searchbyauthor(const char* bv_s_author_in);
void bf_i_searchbyauthortitle(const char* bv_s_author_in, const char* bv_s_title_in);
void bf_i_deleteitem(const char* bv_s_author_in, const char* bv_s_title_in);
void bf_i_mainmenu(void);

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

void bf_i_initcatalog(void) {
    int bv_i_i = 0;
    char bv_s_catalogauthorbuf[256] = {0};
    char bv_s_catalogsubjectbuf[256] = {0};
    char bv_s_catalogtitlebuf[256] = {0};
    char bv_s_headerreservedbuf[256] = {0};
    char bv_s_headersizebuf[256] = {0};

    // global header
    // global catalog
    // header[...] = { ... }  (whole-record write)
    int16_t bcc_tmp_0 = bv_i_last_slot;
    bcc_put_record_header(bcc_files[0], 1, &bcc_tmp_0, "");
    int bt_lim_1 = bv_i_last_slot;
    int bt_step_1 = 1;
    for (bv_i_i = 2; bt_step_1 >= 0 ? bv_i_i <= bt_lim_1 : bv_i_i >= bt_lim_1; bv_i_i += bt_step_1) {
        // catalog[...] = { ... }  (whole-record write)
        bcc_put_record_entry(bcc_files[1], bv_i_i, "", "", "");
    }
}

void bf_i_additem(const char* bv_s_author_in, const char* bv_s_title_in, const char* bv_s_subject_in) {
    char bv_s_author[256];
    snprintf(bv_s_author, sizeof(bv_s_author), "%s", bv_s_author_in);
    char bv_s_title[256];
    snprintf(bv_s_title, sizeof(bv_s_title), "%s", bv_s_title_in);
    char bv_s_subject[256];
    snprintf(bv_s_subject, sizeof(bv_s_subject), "%s", bv_s_subject_in);
    int bv_i_hsize = 0;
    int bv_i_i = 0;
    int bv_i_stop = 0;
    char bv_s_catalogauthorbuf[256] = {0};
    char bv_s_catalogsubjectbuf[256] = {0};
    char bv_s_catalogtitlebuf[256] = {0};
    char bv_s_eauthor[256] = {0};
    char bv_s_esubject[256] = {0};
    char bv_s_etitle[256] = {0};
    char bv_s_headerreservedbuf[256] = {0};
    char bv_s_headersizebuf[256] = {0};
    char bv_s_hreserved[256] = {0};

    // global header
    // global catalog
    // let h = header[...]  (whole-record read)
    bcc_get_record_header(bcc_files[0], 1, bv_s_headersizebuf, bv_s_headerreservedbuf);
    bv_i_hsize = bcc_cvi(bv_s_headersizebuf);
    char bt_s_2[256];
    bf_s_rtrim_s(bv_s_headerreservedbuf, bt_s_2);
    snprintf(bv_s_hreserved, sizeof(bv_s_hreserved), "%s", bt_s_2);
    bv_i_i = 1;
    bv_i_stop = 0;
    while (1) {
        if (!((-(bv_i_stop == 0)))) break;
        bv_i_i = (bv_i_i + 1);
        // let e = catalog[...]  (whole-record read)
        bcc_get_record_entry(bcc_files[1], bv_i_i, bv_s_catalogauthorbuf, bv_s_catalogtitlebuf, bv_s_catalogsubjectbuf);
        char bt_s_3[256];
        bf_s_rtrim_s(bv_s_catalogauthorbuf, bt_s_3);
        snprintf(bv_s_eauthor, sizeof(bv_s_eauthor), "%s", bt_s_3);
        char bt_s_4[256];
        bf_s_rtrim_s(bv_s_catalogtitlebuf, bt_s_4);
        snprintf(bv_s_etitle, sizeof(bv_s_etitle), "%s", bt_s_4);
        char bt_s_5[256];
        bf_s_rtrim_s(bv_s_catalogsubjectbuf, bt_s_5);
        snprintf(bv_s_esubject, sizeof(bv_s_esubject), "%s", bt_s_5);
        if ((-(strcmp(bv_s_eauthor, "") == 0))) {
            bv_i_stop = 1;
        }
        if ((-(bv_i_i == bv_i_hsize))) {
            bv_i_stop = 1;
        }
    }
    if ((-(strcmp(bv_s_eauthor, "") == 0))) {
        // catalog[...] = { ... }  (whole-record write)
        bcc_put_record_entry(bcc_files[1], bv_i_i, bv_s_author, bv_s_title, bv_s_subject);
    } else {
        char bt_s_6[256];
        snprintf(bt_s_6, sizeof(bt_s_6), "%s%s", "Catalog is full -- cannot add ", bv_s_author);
        printf("%s\n", bt_s_6);
    }
}

void bf_i_listall(void) {
    int bv_i_hsize = 0;
    int bv_i_i = 0;
    char bv_s_catalogauthorbuf[256] = {0};
    char bv_s_catalogsubjectbuf[256] = {0};
    char bv_s_catalogtitlebuf[256] = {0};
    char bv_s_eauthor[256] = {0};
    char bv_s_esubject[256] = {0};
    char bv_s_etitle[256] = {0};
    char bv_s_headerreservedbuf[256] = {0};
    char bv_s_headersizebuf[256] = {0};
    char bv_s_hreserved[256] = {0};

    // global header
    // global catalog
    // let h = header[...]  (whole-record read)
    bcc_get_record_header(bcc_files[0], 1, bv_s_headersizebuf, bv_s_headerreservedbuf);
    bv_i_hsize = bcc_cvi(bv_s_headersizebuf);
    char bt_s_7[256];
    bf_s_rtrim_s(bv_s_headerreservedbuf, bt_s_7);
    snprintf(bv_s_hreserved, sizeof(bv_s_hreserved), "%s", bt_s_7);
    int bt_lim_8 = bv_i_hsize;
    int bt_step_8 = 1;
    for (bv_i_i = 2; bt_step_8 >= 0 ? bv_i_i <= bt_lim_8 : bv_i_i >= bt_lim_8; bv_i_i += bt_step_8) {
        // let e = catalog[...]  (whole-record read)
        bcc_get_record_entry(bcc_files[1], bv_i_i, bv_s_catalogauthorbuf, bv_s_catalogtitlebuf, bv_s_catalogsubjectbuf);
        char bt_s_9[256];
        bf_s_rtrim_s(bv_s_catalogauthorbuf, bt_s_9);
        snprintf(bv_s_eauthor, sizeof(bv_s_eauthor), "%s", bt_s_9);
        char bt_s_10[256];
        bf_s_rtrim_s(bv_s_catalogtitlebuf, bt_s_10);
        snprintf(bv_s_etitle, sizeof(bv_s_etitle), "%s", bt_s_10);
        char bt_s_11[256];
        bf_s_rtrim_s(bv_s_catalogsubjectbuf, bt_s_11);
        snprintf(bv_s_esubject, sizeof(bv_s_esubject), "%s", bt_s_11);
        if ((-(strcmp(bv_s_eauthor, "") != 0))) {
            char bt_s_12[256];
            snprintf(bt_s_12, sizeof(bt_s_12), "%s%s", bv_s_eauthor, "  |  ");
            char bt_s_13[256];
            snprintf(bt_s_13, sizeof(bt_s_13), "%s%s", bt_s_12, bv_s_etitle);
            char bt_s_14[256];
            snprintf(bt_s_14, sizeof(bt_s_14), "%s%s", bt_s_13, "  |  ");
            char bt_s_15[256];
            snprintf(bt_s_15, sizeof(bt_s_15), "%s%s", bt_s_14, bv_s_esubject);
            printf("%s\n", bt_s_15);
        }
    }
}

void bf_i_searchbyauthor(const char* bv_s_author_in) {
    char bv_s_author[256];
    snprintf(bv_s_author, sizeof(bv_s_author), "%s", bv_s_author_in);
    int bv_i_hsize = 0;
    int bv_i_i = 0;
    char bv_s_catalogauthorbuf[256] = {0};
    char bv_s_catalogsubjectbuf[256] = {0};
    char bv_s_catalogtitlebuf[256] = {0};
    char bv_s_eauthor[256] = {0};
    char bv_s_esubject[256] = {0};
    char bv_s_etitle[256] = {0};
    char bv_s_headerreservedbuf[256] = {0};
    char bv_s_headersizebuf[256] = {0};
    char bv_s_hreserved[256] = {0};

    // global header
    // global catalog
    // let h = header[...]  (whole-record read)
    bcc_get_record_header(bcc_files[0], 1, bv_s_headersizebuf, bv_s_headerreservedbuf);
    bv_i_hsize = bcc_cvi(bv_s_headersizebuf);
    char bt_s_16[256];
    bf_s_rtrim_s(bv_s_headerreservedbuf, bt_s_16);
    snprintf(bv_s_hreserved, sizeof(bv_s_hreserved), "%s", bt_s_16);
    int bt_lim_17 = bv_i_hsize;
    int bt_step_17 = 1;
    for (bv_i_i = 2; bt_step_17 >= 0 ? bv_i_i <= bt_lim_17 : bv_i_i >= bt_lim_17; bv_i_i += bt_step_17) {
        // let e = catalog[...]  (whole-record read)
        bcc_get_record_entry(bcc_files[1], bv_i_i, bv_s_catalogauthorbuf, bv_s_catalogtitlebuf, bv_s_catalogsubjectbuf);
        char bt_s_18[256];
        bf_s_rtrim_s(bv_s_catalogauthorbuf, bt_s_18);
        snprintf(bv_s_eauthor, sizeof(bv_s_eauthor), "%s", bt_s_18);
        char bt_s_19[256];
        bf_s_rtrim_s(bv_s_catalogtitlebuf, bt_s_19);
        snprintf(bv_s_etitle, sizeof(bv_s_etitle), "%s", bt_s_19);
        char bt_s_20[256];
        bf_s_rtrim_s(bv_s_catalogsubjectbuf, bt_s_20);
        snprintf(bv_s_esubject, sizeof(bv_s_esubject), "%s", bt_s_20);
        if ((-(strcmp(bv_s_eauthor, bv_s_author) == 0))) {
            char bt_s_21[256];
            snprintf(bt_s_21, sizeof(bt_s_21), "%s%s", bv_s_eauthor, "  |  ");
            char bt_s_22[256];
            snprintf(bt_s_22, sizeof(bt_s_22), "%s%s", bt_s_21, bv_s_etitle);
            char bt_s_23[256];
            snprintf(bt_s_23, sizeof(bt_s_23), "%s%s", bt_s_22, "  |  ");
            char bt_s_24[256];
            snprintf(bt_s_24, sizeof(bt_s_24), "%s%s", bt_s_23, bv_s_esubject);
            printf("%s\n", bt_s_24);
        }
    }
}

void bf_i_searchbyauthortitle(const char* bv_s_author_in, const char* bv_s_title_in) {
    char bv_s_author[256];
    snprintf(bv_s_author, sizeof(bv_s_author), "%s", bv_s_author_in);
    char bv_s_title[256];
    snprintf(bv_s_title, sizeof(bv_s_title), "%s", bv_s_title_in);
    int bv_i_hsize = 0;
    int bv_i_i = 0;
    char bv_s_catalogauthorbuf[256] = {0};
    char bv_s_catalogsubjectbuf[256] = {0};
    char bv_s_catalogtitlebuf[256] = {0};
    char bv_s_eauthor[256] = {0};
    char bv_s_esubject[256] = {0};
    char bv_s_etitle[256] = {0};
    char bv_s_headerreservedbuf[256] = {0};
    char bv_s_headersizebuf[256] = {0};
    char bv_s_hreserved[256] = {0};

    // global header
    // global catalog
    // let h = header[...]  (whole-record read)
    bcc_get_record_header(bcc_files[0], 1, bv_s_headersizebuf, bv_s_headerreservedbuf);
    bv_i_hsize = bcc_cvi(bv_s_headersizebuf);
    char bt_s_25[256];
    bf_s_rtrim_s(bv_s_headerreservedbuf, bt_s_25);
    snprintf(bv_s_hreserved, sizeof(bv_s_hreserved), "%s", bt_s_25);
    int bt_lim_26 = bv_i_hsize;
    int bt_step_26 = 1;
    for (bv_i_i = 2; bt_step_26 >= 0 ? bv_i_i <= bt_lim_26 : bv_i_i >= bt_lim_26; bv_i_i += bt_step_26) {
        // let e = catalog[...]  (whole-record read)
        bcc_get_record_entry(bcc_files[1], bv_i_i, bv_s_catalogauthorbuf, bv_s_catalogtitlebuf, bv_s_catalogsubjectbuf);
        char bt_s_27[256];
        bf_s_rtrim_s(bv_s_catalogauthorbuf, bt_s_27);
        snprintf(bv_s_eauthor, sizeof(bv_s_eauthor), "%s", bt_s_27);
        char bt_s_28[256];
        bf_s_rtrim_s(bv_s_catalogtitlebuf, bt_s_28);
        snprintf(bv_s_etitle, sizeof(bv_s_etitle), "%s", bt_s_28);
        char bt_s_29[256];
        bf_s_rtrim_s(bv_s_catalogsubjectbuf, bt_s_29);
        snprintf(bv_s_esubject, sizeof(bv_s_esubject), "%s", bt_s_29);
        if ((-(((-(strcmp(bv_s_eauthor, bv_s_author) == 0))) != 0 && ((-(strcmp(bv_s_etitle, bv_s_title) == 0))) != 0))) {
            char bt_s_30[256];
            snprintf(bt_s_30, sizeof(bt_s_30), "%s%s", bv_s_eauthor, "  |  ");
            char bt_s_31[256];
            snprintf(bt_s_31, sizeof(bt_s_31), "%s%s", bt_s_30, bv_s_etitle);
            char bt_s_32[256];
            snprintf(bt_s_32, sizeof(bt_s_32), "%s%s", bt_s_31, "  |  ");
            char bt_s_33[256];
            snprintf(bt_s_33, sizeof(bt_s_33), "%s%s", bt_s_32, bv_s_esubject);
            printf("%s\n", bt_s_33);
        }
    }
}

void bf_i_deleteitem(const char* bv_s_author_in, const char* bv_s_title_in) {
    char bv_s_author[256];
    snprintf(bv_s_author, sizeof(bv_s_author), "%s", bv_s_author_in);
    char bv_s_title[256];
    snprintf(bv_s_title, sizeof(bv_s_title), "%s", bv_s_title_in);
    int bv_i_hsize = 0;
    int bv_i_i = 0;
    int bv_i_stop = 0;
    char bv_s_catalogauthorbuf[256] = {0};
    char bv_s_catalogsubjectbuf[256] = {0};
    char bv_s_catalogtitlebuf[256] = {0};
    char bv_s_eauthor[256] = {0};
    char bv_s_esubject[256] = {0};
    char bv_s_etitle[256] = {0};
    char bv_s_headerreservedbuf[256] = {0};
    char bv_s_headersizebuf[256] = {0};
    char bv_s_hreserved[256] = {0};

    // global header
    // global catalog
    // let h = header[...]  (whole-record read)
    bcc_get_record_header(bcc_files[0], 1, bv_s_headersizebuf, bv_s_headerreservedbuf);
    bv_i_hsize = bcc_cvi(bv_s_headersizebuf);
    char bt_s_34[256];
    bf_s_rtrim_s(bv_s_headerreservedbuf, bt_s_34);
    snprintf(bv_s_hreserved, sizeof(bv_s_hreserved), "%s", bt_s_34);
    bv_i_i = 1;
    bv_i_stop = 0;
    while (1) {
        if (!((-(bv_i_stop == 0)))) break;
        bv_i_i = (bv_i_i + 1);
        // let e = catalog[...]  (whole-record read)
        bcc_get_record_entry(bcc_files[1], bv_i_i, bv_s_catalogauthorbuf, bv_s_catalogtitlebuf, bv_s_catalogsubjectbuf);
        char bt_s_35[256];
        bf_s_rtrim_s(bv_s_catalogauthorbuf, bt_s_35);
        snprintf(bv_s_eauthor, sizeof(bv_s_eauthor), "%s", bt_s_35);
        char bt_s_36[256];
        bf_s_rtrim_s(bv_s_catalogtitlebuf, bt_s_36);
        snprintf(bv_s_etitle, sizeof(bv_s_etitle), "%s", bt_s_36);
        char bt_s_37[256];
        bf_s_rtrim_s(bv_s_catalogsubjectbuf, bt_s_37);
        snprintf(bv_s_esubject, sizeof(bv_s_esubject), "%s", bt_s_37);
        if ((-(((-(strcmp(bv_s_eauthor, bv_s_author) == 0))) != 0 && ((-(strcmp(bv_s_etitle, bv_s_title) == 0))) != 0))) {
            bv_i_stop = 1;
        }
        if ((-(bv_i_i == bv_i_hsize))) {
            bv_i_stop = 1;
        }
    }
    if ((-(((-(strcmp(bv_s_eauthor, bv_s_author) == 0))) != 0 && ((-(strcmp(bv_s_etitle, bv_s_title) == 0))) != 0))) {
        char bt_s_38[256];
        snprintf(bt_s_38, sizeof(bt_s_38), "%s%s", "Deleting: ", bv_s_eauthor);
        char bt_s_39[256];
        snprintf(bt_s_39, sizeof(bt_s_39), "%s%s", bt_s_38, "  |  ");
        char bt_s_40[256];
        snprintf(bt_s_40, sizeof(bt_s_40), "%s%s", bt_s_39, bv_s_etitle);
        printf("%s\n", bt_s_40);
        // catalog[...] = { ... }  (whole-record write)
        bcc_put_record_entry(bcc_files[1], bv_i_i, "", "", "");
    } else {
        char bt_s_41[256];
        snprintf(bt_s_41, sizeof(bt_s_41), "%s%s", "Not found: ", bv_s_author);
        char bt_s_42[256];
        snprintf(bt_s_42, sizeof(bt_s_42), "%s%s", bt_s_41, "  |  ");
        char bt_s_43[256];
        snprintf(bt_s_43, sizeof(bt_s_43), "%s%s", bt_s_42, bv_s_title);
        printf("%s\n", bt_s_43);
    }
}

void bf_i_mainmenu(void) {
    int bv_i_choice = 0;
    int bv_i_running = 0;
    char bv_s_author[256] = {0};
    char bv_s_subject[256] = {0};
    char bv_s_title[256] = {0};

    bv_i_running = 1;
    while (1) {
        if (!((-(bv_i_running == 1)))) break;
        printf("\n");
        printf("MENU.          1 ) LIST ALL ITEMS\n");
        printf("               2 ) NEW ITEM\n");
        printf("               3 ) SEARCH BY AUTHOR\n");
        printf("               4 ) SEARCH BY AUTHOR + TITLE\n");
        printf("               5 ) DELETE ITEM\n");
        printf("               6 ) STOP\n");
        printf("\n");
        printf("CHOICE: ? ");
        fflush(stdout);
        bcc_read_line();
        bv_i_choice = atoi(bcc_input_buf);
        {
            int bt_sel_44 = bv_i_choice;
            int bt_sel_match_45 = 0;
            if (!bt_sel_match_45) {
                if ((bt_sel_44 == 1)) {
                    bt_sel_match_45 = 1;
                    bf_i_listall();
                }
            }
            if (!bt_sel_match_45) {
                if ((bt_sel_44 == 2)) {
                    bt_sel_match_45 = 1;
                    printf("AUTHOR  ? ");
                    fflush(stdout);
                    bcc_read_line();
                    snprintf(bv_s_author, sizeof(bv_s_author), "%s", bcc_input_buf);
                    printf("TITLE   ? ");
                    fflush(stdout);
                    bcc_read_line();
                    snprintf(bv_s_title, sizeof(bv_s_title), "%s", bcc_input_buf);
                    printf("SUBJECT ? ");
                    fflush(stdout);
                    bcc_read_line();
                    snprintf(bv_s_subject, sizeof(bv_s_subject), "%s", bcc_input_buf);
                    char bt_arg_46[256];
                    snprintf(bt_arg_46, sizeof(bt_arg_46), "%s", bv_s_author);
                    char bt_arg_47[256];
                    snprintf(bt_arg_47, sizeof(bt_arg_47), "%s", bv_s_title);
                    char bt_arg_48[256];
                    snprintf(bt_arg_48, sizeof(bt_arg_48), "%s", bv_s_subject);
                    bf_i_additem(bt_arg_46, bt_arg_47, bt_arg_48);
                }
            }
            if (!bt_sel_match_45) {
                if ((bt_sel_44 == 3)) {
                    bt_sel_match_45 = 1;
                    printf("AUTHOR ? ");
                    fflush(stdout);
                    bcc_read_line();
                    snprintf(bv_s_author, sizeof(bv_s_author), "%s", bcc_input_buf);
                    char bt_arg_49[256];
                    snprintf(bt_arg_49, sizeof(bt_arg_49), "%s", bv_s_author);
                    bf_i_searchbyauthor(bt_arg_49);
                }
            }
            if (!bt_sel_match_45) {
                if ((bt_sel_44 == 4)) {
                    bt_sel_match_45 = 1;
                    printf("AUTHOR ? ");
                    fflush(stdout);
                    bcc_read_line();
                    snprintf(bv_s_author, sizeof(bv_s_author), "%s", bcc_input_buf);
                    printf("TITLE  ? ");
                    fflush(stdout);
                    bcc_read_line();
                    snprintf(bv_s_title, sizeof(bv_s_title), "%s", bcc_input_buf);
                    char bt_arg_50[256];
                    snprintf(bt_arg_50, sizeof(bt_arg_50), "%s", bv_s_author);
                    char bt_arg_51[256];
                    snprintf(bt_arg_51, sizeof(bt_arg_51), "%s", bv_s_title);
                    bf_i_searchbyauthortitle(bt_arg_50, bt_arg_51);
                }
            }
            if (!bt_sel_match_45) {
                if ((bt_sel_44 == 5)) {
                    bt_sel_match_45 = 1;
                    printf("AUTHOR (to delete) ? ");
                    fflush(stdout);
                    bcc_read_line();
                    snprintf(bv_s_author, sizeof(bv_s_author), "%s", bcc_input_buf);
                    printf("TITLE  (to delete) ? ");
                    fflush(stdout);
                    bcc_read_line();
                    snprintf(bv_s_title, sizeof(bv_s_title), "%s", bcc_input_buf);
                    char bt_arg_52[256];
                    snprintf(bt_arg_52, sizeof(bt_arg_52), "%s", bv_s_author);
                    char bt_arg_53[256];
                    snprintf(bt_arg_53, sizeof(bt_arg_53), "%s", bv_s_title);
                    bf_i_deleteitem(bt_arg_52, bt_arg_53);
                }
            }
            if (!bt_sel_match_45) {
                if ((bt_sel_44 == 6)) {
                    bt_sel_match_45 = 1;
                    bv_i_running = 0;
                }
            }
            if (!bt_sel_match_45) {
                printf("Invalid choice\n");
            }
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

    // Card Catalog — a flagship example for the record/file DSL + procedures
    //
    // Adapted from CLERK.BAS, a menu-driven card-catalog manager written by
    // Carlos A. Lujan S. in February 1983 as an improved version of Alfred
    // Fant's LIBRARIAN program (Microcomputing, December 1982). The original
    // source lives in the PeatSoft GW-FILES collection, in the
    // robhagemans/hoard-of-gwbasic archive on GitHub
    // (PeatSoft/GWFILES/CLERK.BAS).
    //
    // What's carried over from CLERK.BAS:
    // - One random-access file holding a header record (the catalog's
    // capacity) in slot 1, followed by author/title/subject entry records
    // in the remaining slots.
    // - NEW ITEM: linear-scan the entries for the first empty slot.
    // - Searches by author, and by author + title together.
    // - DELETE ITEM: linear-scan for the first author+title match, blank it.
    //
    // What's adapted rather than ported line-for-line:
    // - The menu is still interactive (INPUT-driven, like CLERK.BAS's own
    // INKEY$/ON CHOICE GOSUB loop), but each menu action (NEW ITEM, list,
    // the two searches, DELETE ITEM) is its own `procedure` — addItem,
    // listAll, searchByAuthor, searchByAuthorTitle, deleteItem — called
    // from a `mainMenu` dispatch procedure using `select case`, instead of
    // CLERK.BAS's numbered GOTO/GOSUB sections. This is the canonical
    // BASCAL style (see the manual's Procedures section at
    // https://johnjoeallen.github.io/bascal/manual/), and specifically
    // exercises record/file access from inside a procedure body, not just
    // top-level code.
    // - CLERK.BAS's original also supported a multi-diskette/multi-file
    // registry (drive letter + FILEDAT), search-by-subject, and HARD COPY
    // (LPRINT) output. This example keeps one catalog file and the two
    // named searches, and drops the rest, to stay focused on what the
    // record/file DSL and procedures are actually demonstrating here.

    // The header occupies slot 1 of the same file, sized to match Entry's
    // width (20+20+20 = 60 bytes) so both record types agree on where every
    // slot starts. size is the last valid entry slot number, mirroring
    // CLERK.BAS's own S = CVI(F$) header field.

    bv_i_last_slot = 11;

    // file header as Header = open(...)  [60 bytes/record]
    bcc_raise_retry_0: ;
    bcc_files[0] = fopen("catalog.dat", "rb+");
    if (!bcc_files[0]) bcc_files[0] = fopen("catalog.dat", "wb+");
    if (!bcc_files[0]) {
        bcc_err = 75;
        bcc_resume_id = 0;
        bcc_erl = 53;
        bcc_err_file = "examples/card_catalog/card_catalog.bcl";
        if (bcc_on_error_target < 0 || bcc_in_handler) {
            fprintf(stderr, "unhandled BASIC error %d\n", bcc_err);
            exit(1);
        }
        bcc_in_handler = 1;
        switch (bcc_on_error_target) {
        }
    }
    bcc_raise_after_0: ;
    // file catalog as Entry = open(...)  [60 bytes/record]
    bcc_raise_retry_1: ;
    bcc_files[1] = fopen("catalog.dat", "rb+");
    if (!bcc_files[1]) bcc_files[1] = fopen("catalog.dat", "wb+");
    if (!bcc_files[1]) {
        bcc_err = 75;
        bcc_resume_id = 1;
        bcc_erl = 54;
        bcc_err_file = "examples/card_catalog/card_catalog.bcl";
        if (bcc_on_error_target < 0 || bcc_in_handler) {
            fprintf(stderr, "unhandled BASIC error %d\n", bcc_err);
            exit(1);
        }
        bcc_in_handler = 1;
        switch (bcc_on_error_target) {
        }
    }
    bcc_raise_after_1: ;

    // ---- CHOICE=5 in CLERK.BAS: create/reset the catalog file ----

    // ---- CHOICE=1 NEW ITEM in CLERK.BAS ----
    // author$  -- new entry's author
    // title$   -- new entry's title
    // subject$ -- new entry's subject

    // ---- MENU=1 subroutine in CLERK.BAS: list every non-empty entry ----

    // ---- MENU=2 subroutine in CLERK.BAS: filter by author ----
    // author$ -- author name to match

    // ---- MENU=3 subroutine in CLERK.BAS: filter by author AND title ----
    // author$ -- author name to match
    // title$  -- title to match

    // ---- CHOICE=3 DELETE ITEM in CLERK.BAS: first author+title match ----
    // author$ -- author name to match
    // title$  -- title to match

    // ---- CLERK.BAS's own MENU / ON CHOICE GOSUB dispatch loop ----

    // --- Drive the catalog ---

    bf_i_initcatalog();
    bf_i_mainmenu();

    // header.close()
    fclose(bcc_files[0]);
    bcc_files[0] = NULL;
    // catalog.close()
    fclose(bcc_files[1]);
    bcc_files[1] = NULL;

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

static int bcc_put_record_header(FILE* file, long record, const int16_t* field_0, const char* field_1) {
    unsigned char buffer[60];
    if ((!field_0 || !field_1) && !bcc_read_record(file, buffer, 60, record)) return 0;
    (void)(field_0 && memcpy(buffer + 0, field_0, 2));
    if (field_1) bcc_pad_string_field(buffer + 2, field_1, 58);
    bcc_write_record(file, buffer, 60, record);
    return 1;
}

static int bcc_get_record_header(FILE* file, long record, char* field_0, char* field_1) {
    unsigned char buffer[60];
    if (!bcc_read_record(file, buffer, 60, record)) return 0;
    memcpy(field_0, buffer + 0, 2);
    field_0[2] = 0;
    bcc_read_string_field(field_1, buffer + 2, 58);
    return 1;
}

static int bcc_put_record_entry(FILE* file, long record, const char* field_0, const char* field_1, const char* field_2) {
    unsigned char buffer[60];
    if ((!field_0 || !field_1 || !field_2) && !bcc_read_record(file, buffer, 60, record)) return 0;
    if (field_0) bcc_pad_string_field(buffer + 0, field_0, 20);
    if (field_1) bcc_pad_string_field(buffer + 20, field_1, 20);
    if (field_2) bcc_pad_string_field(buffer + 40, field_2, 20);
    bcc_write_record(file, buffer, 60, record);
    return 1;
}

static int bcc_get_record_entry(FILE* file, long record, char* field_0, char* field_1, char* field_2) {
    unsigned char buffer[60];
    if (!bcc_read_record(file, buffer, 60, record)) return 0;
    bcc_read_string_field(field_0, buffer + 0, 20);
    bcc_read_string_field(field_1, buffer + 20, 20);
    bcc_read_string_field(field_2, buffer + 40, 20);
    return 1;
}

static void bcc_mks(char* out, double value) {
    float v = (float)value;
    memcpy(out, &v, 4);
}

static void bcc_mkd(char* out, double value) {
    memcpy(out, &value, 8);
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

static void bcc_read_line(void) {
    if (fgets(bcc_input_buf, sizeof(bcc_input_buf), stdin) == NULL) {
        bcc_input_buf[0] = 0;
        return;
    }
    bcc_input_buf[strcspn(bcc_input_buf, "\r\n")] = 0;
}

