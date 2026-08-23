#include <stdio.h>
#include <string.h>

#include "bcc_runtime.h"

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
