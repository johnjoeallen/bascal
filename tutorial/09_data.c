#include <stdio.h>
#include <string.h>
#include <stdlib.h>

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

#define BCC_DATA_COUNT 10
static const char* bcc_data[BCC_DATA_COUNT] = { "France", "Paris", "Germany", "Berlin", "Japan", "Tokyo", "Brazil", "Brasilia", "Egypt", "Cairo" };

static int bcc_data_ptr = 0;

static const char* bcc_read_data(void) {
    if (bcc_data_ptr >= BCC_DATA_COUNT) {
        fprintf(stderr, "Out of DATA\n");
        exit(1);
    }
    return bcc_data[bcc_data_ptr++];
}

static int bv_i_a = 0;
static int bv_i_b = 0;
static int bv_i_i = 0;
static int bv_i_numcapitals = 0;
static int bv_i_pass = 0;
static char bv_s_firstcapital[256] = {0};
static char bv_s_firstcountry[256] = {0};
static char bv_s_capital[6][256] = {0};
static char bv_s_country[6][256] = {0};

int main(void) {
    // Tutorial 9 — data, read, restore, swap, randomize
    //
    // data embeds literal values directly in the program.  read consumes
    // them in sequence.  restore rewinds the pointer so data can be read
    // again.  The data statements may appear anywhere in the program body;
    // the generated BASIC places them after END.
    //
    // swap exchanges two variables atomically — no temporary needed.
    //
    // randomize seeds the BASIC RND function.  Pass timer for a
    // time-based seed; pass a literal for reproducible results.

    bv_i_numcapitals = 5;


    // Load the lookup table
    int bt_lim_0 = bv_i_numcapitals;
    int bt_step_0 = 1;
    for (bv_i_i = 1; bt_step_0 >= 0 ? bv_i_i <= bt_lim_0 : bv_i_i >= bt_lim_0; bv_i_i += bt_step_0) {
        snprintf(bv_s_country[(bv_i_i)], sizeof(bv_s_country[(bv_i_i)]), "%s", bcc_read_data());
        snprintf(bv_s_capital[(bv_i_i)], sizeof(bv_s_capital[(bv_i_i)]), "%s", bcc_read_data());
    }

    // Print the table
    printf("Country         Capital\n");
    printf("--------------- ---------------\n");
    int bt_lim_1 = bv_i_numcapitals;
    int bt_step_1 = 1;
    for (bv_i_i = 1; bt_step_1 >= 0 ? bv_i_i <= bt_lim_1 : bv_i_i >= bt_lim_1; bv_i_i += bt_step_1) {
        char bt_s_2[256];
        snprintf(bt_s_2, sizeof(bt_s_2), "%s%s", bv_s_country[(bv_i_i)], "        ");
        char bt_s_3[256];
        snprintf(bt_s_3, sizeof(bt_s_3), "%s%s", bt_s_2, bv_s_capital[(bv_i_i)]);
        printf("%s\n", bt_s_3);
    }

    // restore lets us re-read from the beginning
    bcc_data_ptr = 0;
    snprintf(bv_s_firstcountry, sizeof(bv_s_firstcountry), "%s", bcc_read_data());
    snprintf(bv_s_firstcapital, sizeof(bv_s_firstcapital), "%s", bcc_read_data());
    char bt_s_4[256];
    snprintf(bt_s_4, sizeof(bt_s_4), "%s%s", "First entry re-read: ", bv_s_firstcountry);
    char bt_s_5[256];
    snprintf(bt_s_5, sizeof(bt_s_5), "%s%s", bt_s_4, " -> ");
    char bt_s_6[256];
    snprintf(bt_s_6, sizeof(bt_s_6), "%s%s", bt_s_5, bv_s_firstcapital);
    printf("%s\n", bt_s_6);

    // swap — sort two variables without a temp
    bv_i_a = 42;
    bv_i_b = 17;
    char bt_s_7[256];
    snprintf(bt_s_7, sizeof(bt_s_7), "%s%s", "Before swap: a=", bcc_stri(bv_i_a));
    char bt_s_8[256];
    snprintf(bt_s_8, sizeof(bt_s_8), "%s%s", bt_s_7, " b=");
    char bt_s_9[256];
    snprintf(bt_s_9, sizeof(bt_s_9), "%s%s", bt_s_8, bcc_stri(bv_i_b));
    printf("%s\n", bt_s_9);
    int bt_swap_10 = bv_i_a;
    bv_i_a = bv_i_b;
    bv_i_b = bt_swap_10;
    char bt_s_11[256];
    snprintf(bt_s_11, sizeof(bt_s_11), "%s%s", "After swap:  a=", bcc_stri(bv_i_a));
    char bt_s_12[256];
    snprintf(bt_s_12, sizeof(bt_s_12), "%s%s", bt_s_11, " b=");
    char bt_s_13[256];
    snprintf(bt_s_13, sizeof(bt_s_13), "%s%s", bt_s_12, bcc_stri(bv_i_b));
    printf("%s\n", bt_s_13);

    // Bubble-sort the country array using swap
    int bt_lim_14 = (bv_i_numcapitals - 1);
    int bt_step_14 = 1;
    for (bv_i_pass = 1; bt_step_14 >= 0 ? bv_i_pass <= bt_lim_14 : bv_i_pass >= bt_lim_14; bv_i_pass += bt_step_14) {
        int bt_lim_15 = (bv_i_numcapitals - bv_i_pass);
        int bt_step_15 = 1;
        for (bv_i_i = 1; bt_step_15 >= 0 ? bv_i_i <= bt_lim_15 : bv_i_i >= bt_lim_15; bv_i_i += bt_step_15) {
            if ((-(strcmp(bv_s_country[(bv_i_i)], bv_s_country[((bv_i_i + 1))]) > 0))) {
                char bt_swap_16[256];
                snprintf(bt_swap_16, sizeof(bt_swap_16), "%s", bv_s_country[(bv_i_i)]);
                snprintf(bv_s_country[(bv_i_i)], sizeof(bv_s_country[(bv_i_i)]), "%s", bv_s_country[((bv_i_i + 1))]);
                snprintf(bv_s_country[((bv_i_i + 1))], sizeof(bv_s_country[((bv_i_i + 1))]), "%s", bt_swap_16);
                char bt_swap_17[256];
                snprintf(bt_swap_17, sizeof(bt_swap_17), "%s", bv_s_capital[(bv_i_i)]);
                snprintf(bv_s_capital[(bv_i_i)], sizeof(bv_s_capital[(bv_i_i)]), "%s", bv_s_capital[((bv_i_i + 1))]);
                snprintf(bv_s_capital[((bv_i_i + 1))], sizeof(bv_s_capital[((bv_i_i + 1))]), "%s", bt_swap_17);
            }
        }
    }
    printf("Sorted by country:\n");
    int bt_lim_18 = bv_i_numcapitals;
    int bt_step_18 = 1;
    for (bv_i_i = 1; bt_step_18 >= 0 ? bv_i_i <= bt_lim_18 : bv_i_i >= bt_lim_18; bv_i_i += bt_step_18) {
        char bt_s_19[256];
        snprintf(bt_s_19, sizeof(bt_s_19), "%s%s", "  ", bv_s_country[(bv_i_i)]);
        char bt_s_20[256];
        snprintf(bt_s_20, sizeof(bt_s_20), "%s%s", bt_s_19, " -> ");
        char bt_s_21[256];
        snprintf(bt_s_21, sizeof(bt_s_21), "%s%s", bt_s_20, bv_s_capital[(bv_i_i)]);
        printf("%s\n", bt_s_21);
    }

    // randomize — seed with a literal for reproducible output
    srand((unsigned int)(99));

    return 0;

    return 0;
}
