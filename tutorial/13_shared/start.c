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
    // Tutorial 13 — Shared COMMON, program 1 of 2
    //
    // "program name shared sharedname" tells bcc to load sharedname.bcl and
    // emit its COMMON declarations at the very top of the generated output.
    // Every program referencing the same shared file emits the same COMMON
    // block, so the variables survive a CHAIN to the next program.
    //
    // Compile:
    // bcc tutorial/13_shared/start.bcl
    //
    // The generated .bas will open with COMMON count%, label$ followed by
    // the program body below.


    snprintf(bv_s_label, sizeof(bv_s_label), "%s", "Counter demo");
    bv_i_count = 0;

    bv_i_count = (bv_i_count + 1);
    bv_i_count = (bv_i_count + 1);
    bv_i_count = (bv_i_count + 1);

    char bt_s_0[256];
    snprintf(bt_s_0, sizeof(bt_s_0), "%s%s", "Initialised: ", bv_s_label);
    printf("%s\n", bt_s_0);
    char bt_s_1[256];
    snprintf(bt_s_1, sizeof(bt_s_1), "%s%s", "Count after 3 increments: ", bcc_stri(bv_i_count));
    printf("%s\n", bt_s_1);

    // In a real multi-program application you would chain to show.bas:
    // CHAIN "show.bas"
    return 0;
}
