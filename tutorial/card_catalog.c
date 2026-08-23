#include <stdio.h>
#include <string.h>
#include <stdint.h>
#include <stdlib.h>

#include "bcc_runtime.h"

static int bv_i_lastslot = 0;
static char bv_s_catalogauthorbuf[256] = {0};
static char bv_s_catalogsubjectbuf[256] = {0};
static char bv_s_catalogtitlebuf[256] = {0};
static char bv_s_headerreservedbuf[256] = {0};
static char bv_s_headersizebuf[256] = {0};

void bf_i_initcatalog(void);
void bf_i_additem(const char* bv_s_author_in, const char* bv_s_title_in, const char* bv_s_subject_in);
void bf_i_listall(void);
void bf_i_searchbyauthor(const char* bv_s_author_in);
void bf_i_searchbyauthortitle(const char* bv_s_author_in, const char* bv_s_title_in);
void bf_i_deleteitem(const char* bv_s_author_in, const char* bv_s_title_in);
void bf_i_mainmenu(void);

void bf_i_initcatalog(void) {
    int bv_i_i = 0;

    // header[...] = { ... }  (whole-record write)
    int16_t bcc_tmp_0 = bv_i_lastslot;
    bcc_put_record_header(bcc_files[0], 1, &bcc_tmp_0, "");
    int bt_lim_1 = bv_i_lastslot;
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
    int bv_i_eauthortrimi = 0;
    int bv_i_esubjecttrimi = 0;
    int bv_i_etitletrimi = 0;
    int bv_i_hreservedtrimi = 0;
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

    // let h = header[...]  (whole-record read)
    bcc_get_record_header(bcc_files[0], 1, bv_s_headersizebuf, bv_s_headerreservedbuf);
    bv_i_hsize = bcc_cvi(bv_s_headersizebuf);
    bv_i_hreservedtrimi = ((int)strlen(bv_s_headerreservedbuf));
    while (((-(bv_i_hreservedtrimi > 0)) && (-(strcmp(bcc_mid(bv_s_headerreservedbuf, bv_i_hreservedtrimi, 1), " ") == 0)))) {
        bv_i_hreservedtrimi = (bv_i_hreservedtrimi - 1);
    }
    snprintf(bv_s_hreserved, sizeof(bv_s_hreserved), "%s", bcc_mid(bv_s_headerreservedbuf, 1, bv_i_hreservedtrimi));
    bv_i_i = 1;
    bv_i_stop = 0;
    while (1) {
        if (!((-(bv_i_stop == 0)))) break;
        bv_i_i = (bv_i_i + 1);
        // let e = catalog[...]  (whole-record read)
        bcc_get_record_entry(bcc_files[1], bv_i_i, bv_s_catalogauthorbuf, bv_s_catalogtitlebuf, bv_s_catalogsubjectbuf);
        bv_i_eauthortrimi = ((int)strlen(bv_s_catalogauthorbuf));
        while (((-(bv_i_eauthortrimi > 0)) && (-(strcmp(bcc_mid(bv_s_catalogauthorbuf, bv_i_eauthortrimi, 1), " ") == 0)))) {
            bv_i_eauthortrimi = (bv_i_eauthortrimi - 1);
        }
        snprintf(bv_s_eauthor, sizeof(bv_s_eauthor), "%s", bcc_mid(bv_s_catalogauthorbuf, 1, bv_i_eauthortrimi));
        bv_i_etitletrimi = ((int)strlen(bv_s_catalogtitlebuf));
        while (((-(bv_i_etitletrimi > 0)) && (-(strcmp(bcc_mid(bv_s_catalogtitlebuf, bv_i_etitletrimi, 1), " ") == 0)))) {
            bv_i_etitletrimi = (bv_i_etitletrimi - 1);
        }
        snprintf(bv_s_etitle, sizeof(bv_s_etitle), "%s", bcc_mid(bv_s_catalogtitlebuf, 1, bv_i_etitletrimi));
        bv_i_esubjecttrimi = ((int)strlen(bv_s_catalogsubjectbuf));
        while (((-(bv_i_esubjecttrimi > 0)) && (-(strcmp(bcc_mid(bv_s_catalogsubjectbuf, bv_i_esubjecttrimi, 1), " ") == 0)))) {
            bv_i_esubjecttrimi = (bv_i_esubjecttrimi - 1);
        }
        snprintf(bv_s_esubject, sizeof(bv_s_esubject), "%s", bcc_mid(bv_s_catalogsubjectbuf, 1, bv_i_esubjecttrimi));
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
        char bt_s_2[256];
        snprintf(bt_s_2, sizeof(bt_s_2), "%s%s", "Catalog is full -- cannot add ", bv_s_author);
        printf("%s\n", bt_s_2);
    }
}

void bf_i_listall(void) {
    int bv_i_eauthortrimi = 0;
    int bv_i_esubjecttrimi = 0;
    int bv_i_etitletrimi = 0;
    int bv_i_hreservedtrimi = 0;
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

    // let h = header[...]  (whole-record read)
    bcc_get_record_header(bcc_files[0], 1, bv_s_headersizebuf, bv_s_headerreservedbuf);
    bv_i_hsize = bcc_cvi(bv_s_headersizebuf);
    bv_i_hreservedtrimi = ((int)strlen(bv_s_headerreservedbuf));
    while (((-(bv_i_hreservedtrimi > 0)) && (-(strcmp(bcc_mid(bv_s_headerreservedbuf, bv_i_hreservedtrimi, 1), " ") == 0)))) {
        bv_i_hreservedtrimi = (bv_i_hreservedtrimi - 1);
    }
    snprintf(bv_s_hreserved, sizeof(bv_s_hreserved), "%s", bcc_mid(bv_s_headerreservedbuf, 1, bv_i_hreservedtrimi));
    int bt_lim_3 = bv_i_hsize;
    int bt_step_3 = 1;
    for (bv_i_i = 2; bt_step_3 >= 0 ? bv_i_i <= bt_lim_3 : bv_i_i >= bt_lim_3; bv_i_i += bt_step_3) {
        // let e = catalog[...]  (whole-record read)
        bcc_get_record_entry(bcc_files[1], bv_i_i, bv_s_catalogauthorbuf, bv_s_catalogtitlebuf, bv_s_catalogsubjectbuf);
        bv_i_eauthortrimi = ((int)strlen(bv_s_catalogauthorbuf));
        while (((-(bv_i_eauthortrimi > 0)) && (-(strcmp(bcc_mid(bv_s_catalogauthorbuf, bv_i_eauthortrimi, 1), " ") == 0)))) {
            bv_i_eauthortrimi = (bv_i_eauthortrimi - 1);
        }
        snprintf(bv_s_eauthor, sizeof(bv_s_eauthor), "%s", bcc_mid(bv_s_catalogauthorbuf, 1, bv_i_eauthortrimi));
        bv_i_etitletrimi = ((int)strlen(bv_s_catalogtitlebuf));
        while (((-(bv_i_etitletrimi > 0)) && (-(strcmp(bcc_mid(bv_s_catalogtitlebuf, bv_i_etitletrimi, 1), " ") == 0)))) {
            bv_i_etitletrimi = (bv_i_etitletrimi - 1);
        }
        snprintf(bv_s_etitle, sizeof(bv_s_etitle), "%s", bcc_mid(bv_s_catalogtitlebuf, 1, bv_i_etitletrimi));
        bv_i_esubjecttrimi = ((int)strlen(bv_s_catalogsubjectbuf));
        while (((-(bv_i_esubjecttrimi > 0)) && (-(strcmp(bcc_mid(bv_s_catalogsubjectbuf, bv_i_esubjecttrimi, 1), " ") == 0)))) {
            bv_i_esubjecttrimi = (bv_i_esubjecttrimi - 1);
        }
        snprintf(bv_s_esubject, sizeof(bv_s_esubject), "%s", bcc_mid(bv_s_catalogsubjectbuf, 1, bv_i_esubjecttrimi));
        if ((-(strcmp(bv_s_eauthor, "") != 0))) {
            char bt_s_4[256];
            snprintf(bt_s_4, sizeof(bt_s_4), "%s%s", bv_s_eauthor, "  |  ");
            char bt_s_5[256];
            snprintf(bt_s_5, sizeof(bt_s_5), "%s%s", bt_s_4, bv_s_etitle);
            char bt_s_6[256];
            snprintf(bt_s_6, sizeof(bt_s_6), "%s%s", bt_s_5, "  |  ");
            char bt_s_7[256];
            snprintf(bt_s_7, sizeof(bt_s_7), "%s%s", bt_s_6, bv_s_esubject);
            printf("%s\n", bt_s_7);
        }
    }
}

void bf_i_searchbyauthor(const char* bv_s_author_in) {
    char bv_s_author[256];
    snprintf(bv_s_author, sizeof(bv_s_author), "%s", bv_s_author_in);
    int bv_i_eauthortrimi = 0;
    int bv_i_esubjecttrimi = 0;
    int bv_i_etitletrimi = 0;
    int bv_i_hreservedtrimi = 0;
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

    // let h = header[...]  (whole-record read)
    bcc_get_record_header(bcc_files[0], 1, bv_s_headersizebuf, bv_s_headerreservedbuf);
    bv_i_hsize = bcc_cvi(bv_s_headersizebuf);
    bv_i_hreservedtrimi = ((int)strlen(bv_s_headerreservedbuf));
    while (((-(bv_i_hreservedtrimi > 0)) && (-(strcmp(bcc_mid(bv_s_headerreservedbuf, bv_i_hreservedtrimi, 1), " ") == 0)))) {
        bv_i_hreservedtrimi = (bv_i_hreservedtrimi - 1);
    }
    snprintf(bv_s_hreserved, sizeof(bv_s_hreserved), "%s", bcc_mid(bv_s_headerreservedbuf, 1, bv_i_hreservedtrimi));
    int bt_lim_8 = bv_i_hsize;
    int bt_step_8 = 1;
    for (bv_i_i = 2; bt_step_8 >= 0 ? bv_i_i <= bt_lim_8 : bv_i_i >= bt_lim_8; bv_i_i += bt_step_8) {
        // let e = catalog[...]  (whole-record read)
        bcc_get_record_entry(bcc_files[1], bv_i_i, bv_s_catalogauthorbuf, bv_s_catalogtitlebuf, bv_s_catalogsubjectbuf);
        bv_i_eauthortrimi = ((int)strlen(bv_s_catalogauthorbuf));
        while (((-(bv_i_eauthortrimi > 0)) && (-(strcmp(bcc_mid(bv_s_catalogauthorbuf, bv_i_eauthortrimi, 1), " ") == 0)))) {
            bv_i_eauthortrimi = (bv_i_eauthortrimi - 1);
        }
        snprintf(bv_s_eauthor, sizeof(bv_s_eauthor), "%s", bcc_mid(bv_s_catalogauthorbuf, 1, bv_i_eauthortrimi));
        bv_i_etitletrimi = ((int)strlen(bv_s_catalogtitlebuf));
        while (((-(bv_i_etitletrimi > 0)) && (-(strcmp(bcc_mid(bv_s_catalogtitlebuf, bv_i_etitletrimi, 1), " ") == 0)))) {
            bv_i_etitletrimi = (bv_i_etitletrimi - 1);
        }
        snprintf(bv_s_etitle, sizeof(bv_s_etitle), "%s", bcc_mid(bv_s_catalogtitlebuf, 1, bv_i_etitletrimi));
        bv_i_esubjecttrimi = ((int)strlen(bv_s_catalogsubjectbuf));
        while (((-(bv_i_esubjecttrimi > 0)) && (-(strcmp(bcc_mid(bv_s_catalogsubjectbuf, bv_i_esubjecttrimi, 1), " ") == 0)))) {
            bv_i_esubjecttrimi = (bv_i_esubjecttrimi - 1);
        }
        snprintf(bv_s_esubject, sizeof(bv_s_esubject), "%s", bcc_mid(bv_s_catalogsubjectbuf, 1, bv_i_esubjecttrimi));
        if ((-(strcmp(bv_s_eauthor, bv_s_author) == 0))) {
            char bt_s_9[256];
            snprintf(bt_s_9, sizeof(bt_s_9), "%s%s", bv_s_eauthor, "  |  ");
            char bt_s_10[256];
            snprintf(bt_s_10, sizeof(bt_s_10), "%s%s", bt_s_9, bv_s_etitle);
            char bt_s_11[256];
            snprintf(bt_s_11, sizeof(bt_s_11), "%s%s", bt_s_10, "  |  ");
            char bt_s_12[256];
            snprintf(bt_s_12, sizeof(bt_s_12), "%s%s", bt_s_11, bv_s_esubject);
            printf("%s\n", bt_s_12);
        }
    }
}

void bf_i_searchbyauthortitle(const char* bv_s_author_in, const char* bv_s_title_in) {
    char bv_s_author[256];
    snprintf(bv_s_author, sizeof(bv_s_author), "%s", bv_s_author_in);
    char bv_s_title[256];
    snprintf(bv_s_title, sizeof(bv_s_title), "%s", bv_s_title_in);
    int bv_i_eauthortrimi = 0;
    int bv_i_esubjecttrimi = 0;
    int bv_i_etitletrimi = 0;
    int bv_i_hreservedtrimi = 0;
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

    // let h = header[...]  (whole-record read)
    bcc_get_record_header(bcc_files[0], 1, bv_s_headersizebuf, bv_s_headerreservedbuf);
    bv_i_hsize = bcc_cvi(bv_s_headersizebuf);
    bv_i_hreservedtrimi = ((int)strlen(bv_s_headerreservedbuf));
    while (((-(bv_i_hreservedtrimi > 0)) && (-(strcmp(bcc_mid(bv_s_headerreservedbuf, bv_i_hreservedtrimi, 1), " ") == 0)))) {
        bv_i_hreservedtrimi = (bv_i_hreservedtrimi - 1);
    }
    snprintf(bv_s_hreserved, sizeof(bv_s_hreserved), "%s", bcc_mid(bv_s_headerreservedbuf, 1, bv_i_hreservedtrimi));
    int bt_lim_13 = bv_i_hsize;
    int bt_step_13 = 1;
    for (bv_i_i = 2; bt_step_13 >= 0 ? bv_i_i <= bt_lim_13 : bv_i_i >= bt_lim_13; bv_i_i += bt_step_13) {
        // let e = catalog[...]  (whole-record read)
        bcc_get_record_entry(bcc_files[1], bv_i_i, bv_s_catalogauthorbuf, bv_s_catalogtitlebuf, bv_s_catalogsubjectbuf);
        bv_i_eauthortrimi = ((int)strlen(bv_s_catalogauthorbuf));
        while (((-(bv_i_eauthortrimi > 0)) && (-(strcmp(bcc_mid(bv_s_catalogauthorbuf, bv_i_eauthortrimi, 1), " ") == 0)))) {
            bv_i_eauthortrimi = (bv_i_eauthortrimi - 1);
        }
        snprintf(bv_s_eauthor, sizeof(bv_s_eauthor), "%s", bcc_mid(bv_s_catalogauthorbuf, 1, bv_i_eauthortrimi));
        bv_i_etitletrimi = ((int)strlen(bv_s_catalogtitlebuf));
        while (((-(bv_i_etitletrimi > 0)) && (-(strcmp(bcc_mid(bv_s_catalogtitlebuf, bv_i_etitletrimi, 1), " ") == 0)))) {
            bv_i_etitletrimi = (bv_i_etitletrimi - 1);
        }
        snprintf(bv_s_etitle, sizeof(bv_s_etitle), "%s", bcc_mid(bv_s_catalogtitlebuf, 1, bv_i_etitletrimi));
        bv_i_esubjecttrimi = ((int)strlen(bv_s_catalogsubjectbuf));
        while (((-(bv_i_esubjecttrimi > 0)) && (-(strcmp(bcc_mid(bv_s_catalogsubjectbuf, bv_i_esubjecttrimi, 1), " ") == 0)))) {
            bv_i_esubjecttrimi = (bv_i_esubjecttrimi - 1);
        }
        snprintf(bv_s_esubject, sizeof(bv_s_esubject), "%s", bcc_mid(bv_s_catalogsubjectbuf, 1, bv_i_esubjecttrimi));
        if (((-(strcmp(bv_s_eauthor, bv_s_author) == 0)) && (-(strcmp(bv_s_etitle, bv_s_title) == 0)))) {
            char bt_s_14[256];
            snprintf(bt_s_14, sizeof(bt_s_14), "%s%s", bv_s_eauthor, "  |  ");
            char bt_s_15[256];
            snprintf(bt_s_15, sizeof(bt_s_15), "%s%s", bt_s_14, bv_s_etitle);
            char bt_s_16[256];
            snprintf(bt_s_16, sizeof(bt_s_16), "%s%s", bt_s_15, "  |  ");
            char bt_s_17[256];
            snprintf(bt_s_17, sizeof(bt_s_17), "%s%s", bt_s_16, bv_s_esubject);
            printf("%s\n", bt_s_17);
        }
    }
}

void bf_i_deleteitem(const char* bv_s_author_in, const char* bv_s_title_in) {
    char bv_s_author[256];
    snprintf(bv_s_author, sizeof(bv_s_author), "%s", bv_s_author_in);
    char bv_s_title[256];
    snprintf(bv_s_title, sizeof(bv_s_title), "%s", bv_s_title_in);
    int bv_i_eauthortrimi = 0;
    int bv_i_esubjecttrimi = 0;
    int bv_i_etitletrimi = 0;
    int bv_i_hreservedtrimi = 0;
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

    // let h = header[...]  (whole-record read)
    bcc_get_record_header(bcc_files[0], 1, bv_s_headersizebuf, bv_s_headerreservedbuf);
    bv_i_hsize = bcc_cvi(bv_s_headersizebuf);
    bv_i_hreservedtrimi = ((int)strlen(bv_s_headerreservedbuf));
    while (((-(bv_i_hreservedtrimi > 0)) && (-(strcmp(bcc_mid(bv_s_headerreservedbuf, bv_i_hreservedtrimi, 1), " ") == 0)))) {
        bv_i_hreservedtrimi = (bv_i_hreservedtrimi - 1);
    }
    snprintf(bv_s_hreserved, sizeof(bv_s_hreserved), "%s", bcc_mid(bv_s_headerreservedbuf, 1, bv_i_hreservedtrimi));
    bv_i_i = 1;
    bv_i_stop = 0;
    while (1) {
        if (!((-(bv_i_stop == 0)))) break;
        bv_i_i = (bv_i_i + 1);
        // let e = catalog[...]  (whole-record read)
        bcc_get_record_entry(bcc_files[1], bv_i_i, bv_s_catalogauthorbuf, bv_s_catalogtitlebuf, bv_s_catalogsubjectbuf);
        bv_i_eauthortrimi = ((int)strlen(bv_s_catalogauthorbuf));
        while (((-(bv_i_eauthortrimi > 0)) && (-(strcmp(bcc_mid(bv_s_catalogauthorbuf, bv_i_eauthortrimi, 1), " ") == 0)))) {
            bv_i_eauthortrimi = (bv_i_eauthortrimi - 1);
        }
        snprintf(bv_s_eauthor, sizeof(bv_s_eauthor), "%s", bcc_mid(bv_s_catalogauthorbuf, 1, bv_i_eauthortrimi));
        bv_i_etitletrimi = ((int)strlen(bv_s_catalogtitlebuf));
        while (((-(bv_i_etitletrimi > 0)) && (-(strcmp(bcc_mid(bv_s_catalogtitlebuf, bv_i_etitletrimi, 1), " ") == 0)))) {
            bv_i_etitletrimi = (bv_i_etitletrimi - 1);
        }
        snprintf(bv_s_etitle, sizeof(bv_s_etitle), "%s", bcc_mid(bv_s_catalogtitlebuf, 1, bv_i_etitletrimi));
        bv_i_esubjecttrimi = ((int)strlen(bv_s_catalogsubjectbuf));
        while (((-(bv_i_esubjecttrimi > 0)) && (-(strcmp(bcc_mid(bv_s_catalogsubjectbuf, bv_i_esubjecttrimi, 1), " ") == 0)))) {
            bv_i_esubjecttrimi = (bv_i_esubjecttrimi - 1);
        }
        snprintf(bv_s_esubject, sizeof(bv_s_esubject), "%s", bcc_mid(bv_s_catalogsubjectbuf, 1, bv_i_esubjecttrimi));
        if (((-(strcmp(bv_s_eauthor, bv_s_author) == 0)) && (-(strcmp(bv_s_etitle, bv_s_title) == 0)))) {
            bv_i_stop = 1;
        }
        if ((-(bv_i_i == bv_i_hsize))) {
            bv_i_stop = 1;
        }
    }
    if (((-(strcmp(bv_s_eauthor, bv_s_author) == 0)) && (-(strcmp(bv_s_etitle, bv_s_title) == 0)))) {
        char bt_s_18[256];
        snprintf(bt_s_18, sizeof(bt_s_18), "%s%s", "Deleting: ", bv_s_eauthor);
        char bt_s_19[256];
        snprintf(bt_s_19, sizeof(bt_s_19), "%s%s", bt_s_18, "  |  ");
        char bt_s_20[256];
        snprintf(bt_s_20, sizeof(bt_s_20), "%s%s", bt_s_19, bv_s_etitle);
        printf("%s\n", bt_s_20);
        // catalog[...] = { ... }  (whole-record write)
        bcc_put_record_entry(bcc_files[1], bv_i_i, "", "", "");
    } else {
        char bt_s_21[256];
        snprintf(bt_s_21, sizeof(bt_s_21), "%s%s", "Not found: ", bv_s_author);
        char bt_s_22[256];
        snprintf(bt_s_22, sizeof(bt_s_22), "%s%s", bt_s_21, "  |  ");
        char bt_s_23[256];
        snprintf(bt_s_23, sizeof(bt_s_23), "%s%s", bt_s_22, bv_s_title);
        printf("%s\n", bt_s_23);
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
        bcc_read_line();
        bv_i_choice = atoi(bcc_input_buf);

        {
            int bt_sel_24 = bv_i_choice;
            if ((bt_sel_24 == 1)) {
                bf_i_listall();
            } else if ((bt_sel_24 == 2)) {
                printf("AUTHOR  ? ");
                bcc_read_line();
                snprintf(bv_s_author, sizeof(bv_s_author), "%s", bcc_input_buf);
                printf("TITLE   ? ");
                bcc_read_line();
                snprintf(bv_s_title, sizeof(bv_s_title), "%s", bcc_input_buf);
                printf("SUBJECT ? ");
                bcc_read_line();
                snprintf(bv_s_subject, sizeof(bv_s_subject), "%s", bcc_input_buf);
                bf_i_additem(bv_s_author, bv_s_title, bv_s_subject);
            } else if ((bt_sel_24 == 3)) {
                printf("AUTHOR ? ");
                bcc_read_line();
                snprintf(bv_s_author, sizeof(bv_s_author), "%s", bcc_input_buf);
                bf_i_searchbyauthor(bv_s_author);
            } else if ((bt_sel_24 == 4)) {
                printf("AUTHOR ? ");
                bcc_read_line();
                snprintf(bv_s_author, sizeof(bv_s_author), "%s", bcc_input_buf);
                printf("TITLE  ? ");
                bcc_read_line();
                snprintf(bv_s_title, sizeof(bv_s_title), "%s", bcc_input_buf);
                bf_i_searchbyauthortitle(bv_s_author, bv_s_title);
            } else if ((bt_sel_24 == 5)) {
                printf("AUTHOR (to delete) ? ");
                bcc_read_line();
                snprintf(bv_s_author, sizeof(bv_s_author), "%s", bcc_input_buf);
                printf("TITLE  (to delete) ? ");
                bcc_read_line();
                snprintf(bv_s_title, sizeof(bv_s_title), "%s", bcc_input_buf);
                bf_i_deleteitem(bv_s_author, bv_s_title);
            } else if ((bt_sel_24 == 6)) {
                bv_i_running = 0;
            } else {
                printf("Invalid choice\n");
            }
        }
    }
}

int main(void) {
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

    bv_i_lastslot = 11;

    // file header as Header = open(...)  [60 bytes/record]
    bcc_files[0] = fopen("catalog.dat", "rb+");
    if (!bcc_files[0]) bcc_files[0] = fopen("catalog.dat", "wb+");
    // file catalog as Entry = open(...)  [60 bytes/record]
    bcc_files[1] = fopen("catalog.dat", "rb+");
    if (!bcc_files[1]) bcc_files[1] = fopen("catalog.dat", "wb+");

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
