#include <stdio.h>
#include <string.h>

#include "bcc_runtime.h"

static float bv_f_taxrate = 0;
static float bv_f_temperature = 0;
static int bv_i_maxscore = 0;
static int bv_i_passmark = 0;
static int bv_i_score = 0;
static char bv_s_appname[256] = {0};
static char bv_s_greeting[256] = {0};
static char bv_s_playername[256] = {0};

int main(void) {
    // Tutorial 2 — Variables and Constants
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
