// BASCAL generated C -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
#include <stdio.h>
#include <stdlib.h>

static int bcc_data_ptr = 0;

static const char* bcc_read_data(void);

static char bv_s_firstcountry[256] = {0};
static char bv_s_secondcountry[256] = {0};

int main(void) {
    // Tutorial — DATA, READ, and RESTORE
    //
    // RESTORE rewinds the DATA pointer to a named DATA block.  Labels keep the
    // source readable while the transpiler assigns the generated line numbers.

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

#define BCC_DATA_COUNT 2
static const char* bcc_data[BCC_DATA_COUNT] = { "France", "Japan" };

static const char* bcc_read_data(void) {
    if (bcc_data_ptr >= BCC_DATA_COUNT) {
        fprintf(stderr, "Out of DATA\n");
        exit(1);
    }
    return bcc_data[bcc_data_ptr++];
}

