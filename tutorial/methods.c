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

static float bv_f_price = 0;
static int bv_i_length = 0;
static int bv_i_score = 0;
static int bv_l_cardcopies = 0;
static int bv_l_signedcopies = 0;
static char bv_s_cardauthor[256] = {0};
static char bv_s_cardtitle[256] = {0};
static char bv_s_firstthree[256] = {0};
static char bv_s_name[256] = {0};
static char bv_s_result[256] = {0};
static char bv_s_shoutresult[256] = {0};
static char bv_s_signedauthor[256] = {0};
static char bv_s_signedsignature[256] = {0};
static char bv_s_signedtitle[256] = {0};

void bf_s_ucase_s(const char* bv_s_self_in, char* bcc_out);
void bf_s_shout_s(const char* bv_s_self_in, char* bcc_out);
void bf_s_surround_s(const char* bv_s_self_in, const char* bv_s_left_in, const char* bv_s_right_in, char* bcc_out);
int bf_i_clamp_i(int bv_i_self, int bv_i_low, int bv_i_high);
float bf_f_percent_f(float bv_f_self, float bv_f_rate);
void bf_i_cardrestock(char* bv_s_selftitle_in, char* bv_s_selfauthor_in, int* bv_l_selfcopies_in, int bv_i_amount);
void bf_i_signedcardrestock(char* bv_s_selftitle_in, char* bv_s_selfauthor_in, int* bv_l_selfcopies_in, char* bv_s_selfsignature_in, int bv_i_amount);
void bf_s_carddisplay(char* bv_s_selftitle_in, char* bv_s_selfauthor_in, int* bv_l_selfcopies_in, char* bcc_out);
void bf_s_signedcarddisplay(char* bv_s_selftitle_in, char* bv_s_selfauthor_in, int* bv_l_selfcopies_in, char* bv_s_selfsignature_in, char* bcc_out);

void bf_s_ucase_s(const char* bv_s_self_in, char* bcc_out) {
    char bv_s_self[256];
    snprintf(bv_s_self, sizeof(bv_s_self), "%s", bv_s_self_in);
    int bv_i_c = 0;
    int bv_i_i = 0;
    char bv_s_out[256] = {0};

    snprintf(bv_s_out, sizeof(bv_s_out), "%s", "");
    int bt_lim_0 = ((int)strlen(bv_s_self));
    int bt_step_0 = 1;
    for (bv_i_i = 1; bt_step_0 >= 0 ? bv_i_i <= bt_lim_0 : bv_i_i >= bt_lim_0; bv_i_i += bt_step_0) {
        bv_i_c = ((int)(unsigned char)bcc_mid(bv_s_self, bv_i_i, 1)[0]);
        if (((-(bv_i_c >= 97)) && (-(bv_i_c <= 122)))) {
            bv_i_c = (bv_i_c - 32);
        }
        char bt_s_1[256];
        snprintf(bt_s_1, sizeof(bt_s_1), "%s%s", bv_s_out, bcc_chr(bv_i_c));
        snprintf(bv_s_out, sizeof(bv_s_out), "%s", bt_s_1);
    }
    snprintf(bcc_out, 256, "%s", bv_s_out);
    return;
    snprintf(bcc_out, 256, "%s", bv_s_self);
    return;
}

void bf_s_shout_s(const char* bv_s_self_in, char* bcc_out) {
    char bv_s_self[256];
    snprintf(bv_s_self, sizeof(bv_s_self), "%s", bv_s_self_in);

    char bt_s_2[256];
    bf_s_ucase_s(bv_s_self, bt_s_2);
    char bt_s_3[256];
    snprintf(bt_s_3, sizeof(bt_s_3), "%s%s", bt_s_2, "!");
    snprintf(bcc_out, 256, "%s", bt_s_3);
    return;
    snprintf(bcc_out, 256, "%s", bv_s_self);
    return;
}

void bf_s_surround_s(const char* bv_s_self_in, const char* bv_s_left_in, const char* bv_s_right_in, char* bcc_out) {
    char bv_s_self[256];
    snprintf(bv_s_self, sizeof(bv_s_self), "%s", bv_s_self_in);
    char bv_s_left[256];
    snprintf(bv_s_left, sizeof(bv_s_left), "%s", bv_s_left_in);
    char bv_s_right[256];
    snprintf(bv_s_right, sizeof(bv_s_right), "%s", bv_s_right_in);

    char bt_s_4[256];
    snprintf(bt_s_4, sizeof(bt_s_4), "%s%s", bv_s_left, bv_s_self);
    char bt_s_5[256];
    snprintf(bt_s_5, sizeof(bt_s_5), "%s%s", bt_s_4, bv_s_right);
    snprintf(bcc_out, 256, "%s", bt_s_5);
    return;
    snprintf(bcc_out, 256, "%s", bv_s_self);
    return;
}

int bf_i_clamp_i(int bv_i_self, int bv_i_low, int bv_i_high) {
    if ((-(bv_i_self < bv_i_low))) {
        return bv_i_low;
    } else {
        if ((-(bv_i_self > bv_i_high))) {
            return bv_i_high;
        }
    }
    return bv_i_self;
    return bv_i_self;
}

float bf_f_percent_f(float bv_f_self, float bv_f_rate) {
    return ((double)(bv_f_self * bv_f_rate) / (double)100);
    return bv_f_self;
}

void bf_i_cardrestock(char* bv_s_selftitle_in, char* bv_s_selfauthor_in, int* bv_l_selfcopies_in, int bv_i_amount) {
    char bv_s_selftitle[256];
    snprintf(bv_s_selftitle, sizeof(bv_s_selftitle), "%s", bv_s_selftitle_in);
    char bv_s_selfauthor[256];
    snprintf(bv_s_selfauthor, sizeof(bv_s_selfauthor), "%s", bv_s_selfauthor_in);
    int bv_l_selfcopies = *bv_l_selfcopies_in;

    bv_l_selfcopies = (bv_l_selfcopies + bv_i_amount);
    snprintf(bv_s_selftitle_in, 256, "%s", bv_s_selftitle);
    snprintf(bv_s_selfauthor_in, 256, "%s", bv_s_selfauthor);
    *bv_l_selfcopies_in = bv_l_selfcopies;
}

void bf_i_signedcardrestock(char* bv_s_selftitle_in, char* bv_s_selfauthor_in, int* bv_l_selfcopies_in, char* bv_s_selfsignature_in, int bv_i_amount) {
    char bv_s_selftitle[256];
    snprintf(bv_s_selftitle, sizeof(bv_s_selftitle), "%s", bv_s_selftitle_in);
    char bv_s_selfauthor[256];
    snprintf(bv_s_selfauthor, sizeof(bv_s_selfauthor), "%s", bv_s_selfauthor_in);
    int bv_l_selfcopies = *bv_l_selfcopies_in;
    char bv_s_selfsignature[256];
    snprintf(bv_s_selfsignature, sizeof(bv_s_selfsignature), "%s", bv_s_selfsignature_in);

    bv_l_selfcopies = (bv_l_selfcopies + bv_i_amount);
    snprintf(bv_s_selftitle_in, 256, "%s", bv_s_selftitle);
    snprintf(bv_s_selfauthor_in, 256, "%s", bv_s_selfauthor);
    *bv_l_selfcopies_in = bv_l_selfcopies;
    snprintf(bv_s_selfsignature_in, 256, "%s", bv_s_selfsignature);
}

void bf_s_carddisplay(char* bv_s_selftitle_in, char* bv_s_selfauthor_in, int* bv_l_selfcopies_in, char* bcc_out) {
    char bv_s_selftitle[256];
    snprintf(bv_s_selftitle, sizeof(bv_s_selftitle), "%s", bv_s_selftitle_in);
    char bv_s_selfauthor[256];
    snprintf(bv_s_selfauthor, sizeof(bv_s_selfauthor), "%s", bv_s_selfauthor_in);
    int bv_l_selfcopies = *bv_l_selfcopies_in;

    char bt_s_6[256];
    snprintf(bt_s_6, sizeof(bt_s_6), "%s%s", bv_s_selftitle, " by ");
    char bt_s_7[256];
    snprintf(bt_s_7, sizeof(bt_s_7), "%s%s", bt_s_6, bv_s_selfauthor);
    snprintf(bcc_out, 256, "%s", bt_s_7);
    snprintf(bv_s_selftitle_in, 256, "%s", bv_s_selftitle);
    snprintf(bv_s_selfauthor_in, 256, "%s", bv_s_selfauthor);
    *bv_l_selfcopies_in = bv_l_selfcopies;
    return;
    snprintf(bv_s_selftitle_in, 256, "%s", bv_s_selftitle);
    snprintf(bv_s_selfauthor_in, 256, "%s", bv_s_selfauthor);
    *bv_l_selfcopies_in = bv_l_selfcopies;
}

void bf_s_signedcarddisplay(char* bv_s_selftitle_in, char* bv_s_selfauthor_in, int* bv_l_selfcopies_in, char* bv_s_selfsignature_in, char* bcc_out) {
    char bv_s_selftitle[256];
    snprintf(bv_s_selftitle, sizeof(bv_s_selftitle), "%s", bv_s_selftitle_in);
    char bv_s_selfauthor[256];
    snprintf(bv_s_selfauthor, sizeof(bv_s_selfauthor), "%s", bv_s_selfauthor_in);
    int bv_l_selfcopies = *bv_l_selfcopies_in;
    char bv_s_selfsignature[256];
    snprintf(bv_s_selfsignature, sizeof(bv_s_selfsignature), "%s", bv_s_selfsignature_in);

    char bt_s_8[256];
    snprintf(bt_s_8, sizeof(bt_s_8), "%s%s", bv_s_selftitle, " by ");
    char bt_s_9[256];
    snprintf(bt_s_9, sizeof(bt_s_9), "%s%s", bt_s_8, bv_s_selfauthor);
    char bt_s_10[256];
    snprintf(bt_s_10, sizeof(bt_s_10), "%s%s", bt_s_9, ", signed ");
    char bt_s_11[256];
    snprintf(bt_s_11, sizeof(bt_s_11), "%s%s", bt_s_10, bv_s_selfsignature);
    snprintf(bcc_out, 256, "%s", bt_s_11);
    snprintf(bv_s_selftitle_in, 256, "%s", bv_s_selftitle);
    snprintf(bv_s_selfauthor_in, 256, "%s", bv_s_selfauthor);
    *bv_l_selfcopies_in = bv_l_selfcopies;
    snprintf(bv_s_selfsignature_in, 256, "%s", bv_s_selfsignature);
    return;
    snprintf(bv_s_selftitle_in, 256, "%s", bv_s_selftitle);
    snprintf(bv_s_selfauthor_in, 256, "%s", bv_s_selfauthor);
    *bv_l_selfcopies_in = bv_l_selfcopies;
    snprintf(bv_s_selfsignature_in, 256, "%s", bv_s_selfsignature);
}

int main(void) {
    // Upper-cases self$. Not a real MBASIC/BASCOM 2.00 builtin -- verified
    // against a real IBM BASIC Compiler 2.00 under dosbox-x -- so BASCAL ships
    // its own. Declared as a scalar method (see GitHub issue #41 and
    // ltrim.bcl's own doc comment for the reasoning) -- ucase$(s$) still works
    // via ordinary-call syntax resolving to this same declaration.

    // Tutorial — Methods
    //
    // A method is a statically resolved callable with an implicit receiver,
    // written in brackets after the method name: `method shout[string]()`
    // receives a string. The return type follows the parameter list after
    // `:` (or its suffix shorthand, `: $`); omitting it for a scalar receiver
    // makes the method return the receiver's own type, falling through to an
    // implicit `return self`. Dot calls chain when each result has the next
    // receiver's type. Methods transpile to ordinary typed calls for every
    // backend -- there is no runtime method object, virtual dispatch, or
    // vtable of any kind.
    //
    // A record type is a valid receiver too, declared either externally (in
    // brackets, same as a scalar receiver) or inline, directly inside the
    // record itself. `self.field` is then ordinary field access against the
    // receiver, and mutating it is visible to the caller once the call
    // returns -- the receiver is passed by reference, not by copy.






    snprintf(bv_s_name, sizeof(bv_s_name), "%s", "bascal");
    char bt_s_12[256];
    bf_s_surround_s(bv_s_name, "[", "]", bt_s_12);
    snprintf(bv_s_result, sizeof(bv_s_result), "%s", bt_s_12);
    printf("%s\n", bv_s_result);
    char bt_s_13[256];
    bf_s_shout_s(bv_s_name, bt_s_13);
    snprintf(bv_s_shoutresult, sizeof(bv_s_shoutresult), "%s", bt_s_13);
    bv_i_length = ((int)strlen(bcc_mid(bv_s_name, 1, 5)));
    printf("length = %d\n", bv_i_length);

    bv_i_score = 125;
    printf("clamped score = %d\n", bf_i_clamp_i(bv_i_score, 0, 100));

    bv_f_price = 80;
    printf("discount amount = %g\n", bf_f_percent_f(bv_f_price, 15));

    char bt_s_14[256];
    bf_s_ucase_s(bcc_mid(bv_s_name, 1, 3), bt_s_14);
    snprintf(bv_s_firstthree, sizeof(bv_s_firstthree), "%s", bt_s_14);
    printf("first three = %s\n", bv_s_firstthree);

    // -------------------- Record methods --------------------

    // Declared inline: the enclosing record supplies the receiver type, so
    // there is no `[Card]` bracket here at all.

    // Declared externally: same receiver, same callable identity as an
    // inline method -- an external method just lets behavior be attached to
    // a record without editing its own declaration.

    snprintf(bv_s_cardtitle, sizeof(bv_s_cardtitle), "%s", "Dune");
    snprintf(bv_s_cardauthor, sizeof(bv_s_cardauthor), "%s", "Frank Herbert");
    bv_l_cardcopies = 2;
    char bt_s_15[256];
    bf_s_carddisplay(bv_s_cardtitle, bv_s_cardauthor, &bv_l_cardcopies, bt_s_15);
    printf("%s\n", bt_s_15);
    printf("copies on hand = %d\n", bv_l_cardcopies);

    bf_i_cardrestock(bv_s_cardtitle, bv_s_cardauthor, &bv_l_cardcopies, 3);
    printf("copies after restock = %d\n", bv_l_cardcopies);

    // `combines` is structural field composition only, not inheritance:
    // SignedCard's own field list is Card's fields (title, author, copies)
    // followed by its own (signature), so its record literal accepts all
    // four -- but Card's methods aren't combined in. SignedCard needs its
    // own display() (below); calling signed.restock(...) without declaring
    // SignedCard's own restock() would be a compile error, since a method
    // is only ever visible for the exact record type it was declared for.


    snprintf(bv_s_signedtitle, sizeof(bv_s_signedtitle), "%s", "Dune");
    snprintf(bv_s_signedauthor, sizeof(bv_s_signedauthor), "%s", "Frank Herbert");
    bv_l_signedcopies = 1;
    snprintf(bv_s_signedsignature, sizeof(bv_s_signedsignature), "%s", "F.H.");
    char bt_s_16[256];
    bf_s_signedcarddisplay(bv_s_signedtitle, bv_s_signedauthor, &bv_l_signedcopies, bv_s_signedsignature, bt_s_16);
    printf("%s\n", bt_s_16);
    bf_i_signedcardrestock(bv_s_signedtitle, bv_s_signedauthor, &bv_l_signedcopies, bv_s_signedsignature, 2);
    printf("signed copies after restock = %d\n", bv_l_signedcopies);

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

