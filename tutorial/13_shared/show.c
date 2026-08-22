#include <stdio.h>
#include <string.h>

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

static int bv_i_count = 0;
static char bv_s_label[256] = {0};

int main(void) {
    // Tutorial 13 — Shared COMMON, program 2 of 2
    //
    // This program references the same shared file as start.bcl.  Its
    // generated BASIC will begin with the same COMMON block, so count% and
    // label$ contain whatever values start.bas left in them when it CHAINed
    // here.
    //
    // Compile:
    // bcc tutorial/13_shared/show.bcl


    char bt_s_0[256];
    snprintf(bt_s_0, sizeof(bt_s_0), "%s%s", "Label:  ", bv_s_label);
    printf("%s\n", bt_s_0);
    char bt_s_1[256];
    snprintf(bt_s_1, sizeof(bt_s_1), "%s%s", "Count:  ", bcc_stri(bv_i_count));
    printf("%s\n", bt_s_1);

    if ((-(bv_i_count > 0))) {
        char bt_s_2[256];
        snprintf(bt_s_2, sizeof(bt_s_2), "%s%s", "Counter was incremented ", bcc_stri(bv_i_count));
        char bt_s_3[256];
        snprintf(bt_s_3, sizeof(bt_s_3), "%s%s", bt_s_2, " time(s).");
        printf("%s\n", bt_s_3);
    } else {
        printf("Counter was never incremented.\n");
    }

    return 0;
}
