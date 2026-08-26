[Home](../../) / [Tutorials](../) / RESTORE and DATA

<div class="prose" markdown="1">

Generated sources: [BCL](https://github.com/johnjoeallen/bascal/blob/main/tutorial/restore_data.bcl), [BASIC](https://github.com/johnjoeallen/bascal/blob/main/tutorial/restore_data.bas), and [C](https://github.com/johnjoeallen/bascal/blob/main/tutorial/restore_data.c).

`read` consumes values from the program's `data` statements. `restore label` rewinds the DATA pointer to the named block, making a second pass possible without counting generated line numbers. The JVM target does not yet implement embedded DATA/READ/RESTORE.

</div>

<!-- BEGIN generated tutorial source -->

<details class="source-embed" markdown="1">

<summary><code>tutorial/restore_data.bcl</code></summary>



```bascal

// Tutorial — DATA, READ, and RESTORE
//
// RESTORE rewinds the DATA pointer to a named DATA block.  Labels keep the
// source readable while the transpiler assigns the generated line numbers.
program restoreData

print "restore to a label:"
read firstCountry$
print "  first read: "; firstCountry$
restore secondBatch
read secondCountry$
print "  after restore secondBatch: "; secondCountry$

end

data "France"

secondBatch:
data "Japan"

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/restore_data.bas</code></summary>



```basic

10 ' BASCAL generated BASIC -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Tutorial — DATA, READ, and RESTORE
40 '
50 ' RESTORE rewinds the DATA pointer to a named DATA block.  Labels keep the
60 ' source readable while the transpiler assigns the generated line numbers.

70 PRINT "restore to a label:"
80 READ firstcountry$
90 PRINT "  first read: "; firstcountry$
100 RESTORE 150
110 READ secondcountry$
120 PRINT "  after restore secondBatch: "; secondcountry$

130 END

140 DATA "France"

150 DATA "Japan"

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/restore_data.c</code></summary>



```c

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


```



</details>

<!-- END generated tutorial source -->
