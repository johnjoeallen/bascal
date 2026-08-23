#include <stdio.h>
#include <string.h>
#include <stdint.h>
#include <stdlib.h>

#include "bcc_runtime.h"

static char bv_s_filename[256] = {0};
static char bv_s_firstcountry[256] = {0};
static char bv_s_secondcountry[256] = {0};

int main(void) {
    // Tutorial — Labels and Error Handling
    //
    // BASCAL manages line numbers itself -- goto, gosub, on error goto, resume,
    // restore, and on ... goto / on ... gosub can never target a raw line
    // number in .bcl source. Every one of them requires a name: label instead;
    // the compiler assigns the real BASIC line number when it renders output,
    // the same way it already numbers the branch targets inside if/while/do/
    // select case.
    //
    // on error goto 0 is the one numeric exception -- 0 isn't a line number,
    // it's the sentinel that disables the error trap.

    // ---- goto / label basics ----

    printf("goto/label basics:\n");
    goto bcc_lbl_afterskip;
    printf("  not reached\n");
    bcc_lbl_afterskip:;
    printf("  reached via goto\n");

    // ---- gosub / return (BASIC-level subroutine, distinct from BASCAL functions) ----

    printf("gosub/return:\n");
    bcc_gosub_stack[bcc_gosub_sp++] = 0;
    goto bcc_lbl_printbanner;
    bcc_ret_0:;
    printf("  back after gosub\n");
    goto bcc_lbl_afterbanner;

    bcc_lbl_printbanner:;
    printf("  inside the gosub'd subroutine\n");
    switch (bcc_gosub_stack[--bcc_gosub_sp]) {
        case 0: goto bcc_ret_0;
    }

    bcc_lbl_afterbanner:;

    // ---- error handling: on error goto, resume to a label, err ----
    //
    // Opening a file that doesn't exist raises BASIC runtime error 53
    // ("file not found"). The handler below catches it, prints a message, and
    // then RESUMEs at a label -- not the failing statement or "next", but a
    // specific point past the whole try/handler region. RESUME (not a plain
    // GOTO) is what clears the runtime's "currently handling an error" state,
    // so a later error can still be trapped.

    printf("error handling, missing file:\n");
    snprintf(bv_s_filename, sizeof(bv_s_filename), "%s", "does_not_exist.dat");
    bcc_on_error_target = 0;
    bcc_raise_retry_0: ;
    bcc_files[0] = fopen(bv_s_filename, "r");
    if (!bcc_files[0]) {
        bcc_err = 53;
        bcc_resume_id = 0;
        if (bcc_on_error_target < 0 || bcc_in_handler) {
            fprintf(stderr, "unhandled BASIC error %d\n", bcc_err);
            exit(1);
        }
        bcc_in_handler = 1;
        switch (bcc_on_error_target) {
            case 0: goto bcc_lbl_handleopenerror;
        }
    }
    bcc_raise_after_0: ;
    printf("  file opened (unexpected)\n");
    fclose(bcc_files[0]);
    bcc_files[0] = NULL;
    goto bcc_lbl_afteropen;

    bcc_lbl_handleopenerror:;
    if ((-(bcc_err == 53))) {
        printf("  caught error %d: %s not found\n", bcc_err, bv_s_filename);
        bcc_in_handler = 0;
        goto bcc_lbl_afteropen;
    } else {
        printf("  unexpected error %d\n", bcc_err);
        bcc_raise_retry_1: ;
        bcc_err = bcc_err;
        bcc_resume_id = 1;
        if (bcc_on_error_target < 0 || bcc_in_handler) {
            fprintf(stderr, "unhandled BASIC error %d\n", bcc_err);
            exit(1);
        }
        bcc_in_handler = 1;
        switch (bcc_on_error_target) {
            case 0: goto bcc_lbl_handleopenerror;
        }
        bcc_raise_after_1: ;
    }

    bcc_lbl_afteropen:;
    bcc_on_error_target = -1;

    // ---- restore with a label: rewind the DATA pointer to a specific block ----

    printf("restore to a label:\n");
    snprintf(bv_s_firstcountry, sizeof(bv_s_firstcountry), "%s", bcc_read_data());
    printf("  first read: %s\n", bv_s_firstcountry);
    bcc_data_ptr = 1;
    snprintf(bv_s_secondcountry, sizeof(bv_s_secondcountry), "%s", bcc_read_data());
    printf("  after restore secondBatch: %s\n", bv_s_secondcountry);

    return 0;


    bcc_lbl_secondbatch:;
    return 0;
}
