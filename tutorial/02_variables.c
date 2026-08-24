// BASCAL generated C -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
#include <stdio.h>
#include <string.h>

#define BCC_STRBUF_COUNT 8
static char bcc_strbuf[BCC_STRBUF_COUNT][256];
static int bcc_strbuf_next = 0;

static char* bcc_strbuf_take(void);
static const char* bcc_mid(const char* s, int start, int length);
static const char* bcc_chr(int code);
static const char* bcc_stri(int value);
static const char* bcc_strd(double value);

static float bv_f_taxrate = 0;
static float bv_f_temperature = 0;
static int bv_i_maxscore = 0;
static int bv_i_passmark = 0;
static int bv_i_score = 0;
static char bv_s_appname[256] = {0};
static char bv_s_greeting[256] = {0};
static char bv_s_playername[256] = {0};

int main(void) {
    // Tutorial — Variables and Constants
    //
    // Every name in BASCAL ends with a type suffix that tells the runtime
    // how to store the value:
    //
    // %   integer   — 16-bit signed, -32768 to 32767
    // $   string    — variable-length text
    // !   single    — 32-bit floating-point
    // #   double    — 64-bit floating-point
    // &   long      — 32-bit signed integer
    //
    // All variables are global.  They spring into existence on first use;
    // dim (or its synonym declare) is needed only for arrays or when you
    // want to be explicit -- declare tends to read better for a plain
    // scalar, dim for an array.
    //
    // const names a value that cannot change.  Use it for magic numbers
    // so the intent is clear and the value lives in one place.

    bv_i_maxscore = 100;
    bv_i_passmark = 60;
    snprintf(bv_s_appname, sizeof(bv_s_appname), "%s", "Grade Checker");
    bv_f_taxrate = 0.2;

    // Variable assignment uses =
    snprintf(bv_s_playername, sizeof(bv_s_playername), "%s", "Alice");
    bv_i_score = 87;
    bv_f_temperature = 36.6;

    // print mixes strings and numbers directly with ; (no str$() needed)
    printf("%s\n", bv_s_appname);
    printf("Player:      %s\n", bv_s_playername);
    printf("Score:       %d/ %d\n", bv_i_score, bv_i_maxscore);
    printf("Pass mark:   %d\n", bv_i_passmark);
    printf("Temperature: %g\n", bv_f_temperature);
    printf("Tax rate:    %g\n", bv_f_taxrate);

    // str$() is still available when you need to build a string value
    char bt_s_0[256];
    snprintf(bt_s_0, sizeof(bt_s_0), "%s%s", "Score is ", bcc_stri(bv_i_score));
    snprintf(bv_s_greeting, sizeof(bv_s_greeting), "%s", bt_s_0);
    printf("%s\n", bv_s_greeting);

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

