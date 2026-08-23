#include <stdio.h>
#include <string.h>

#include "bcc_runtime.h"

static int bv_i_count = 0;
static char bv_s_label[256] = {0};

int main(void) {
    // Tutorial — Shared COMMON, program 1 of 2
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

    // In a real multi-program application you would chain to the compiled
    // show.exe -- CHAIN takes a program name, not the .bas source it was
    // compiled from (verified against real BASCOM 2.00 under dosbox-x:
    // CHAIN "show.bas" tries to run the source file itself and corrupts):
    // CHAIN "show"
    return 0;
}
