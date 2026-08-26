[Home](../../) / [Tutorials](../) / Data, Read, Restore, Swap

<div class="prose" markdown="1">

`data` embeds literal values directly in the program; `read` consumes them in sequence. `data` statements can appear anywhere in the source — the generated BASIC places them after `END`. `swap` exchanges two variables (including array elements) without a temporary. See [RESTORE and DATA](restore_data.md) for rewinding to a named DATA block.

</div>

<div class="snippet" markdown="1">

### Loading a table with READ

```bascal
for i% = 1 to num_capitals%
    read country$(i%), capital$(i%)
end for

' ...

data "France",  "Paris"
data "Germany", "Berlin"
data "Japan",   "Tokyo"
```

</div>

<div class="snippet" markdown="1">

### SWAP, including array elements

```bascal
swap a%, b%

' Bubble-sort using swap -- no temp variable needed
if country$(i%) > country$(i% + 1) then
    swap country$(i%), country$(i% + 1)
    swap capital$(i%), capital$(i% + 1)
end if
```

</div>



[← Arrays](arrays.md)  ·  [File Input and Output →](files.md)


<!-- BEGIN generated tutorial source -->

<details class="source-embed" markdown="1">

<summary><code>tutorial/data.bcl</code></summary>



```bascal

// Tutorial — DATA, READ, SWAP, and RANDOMIZE
//
// data embeds literal values directly in the program.  read consumes
// them in sequence.  The data statements may appear anywhere in the program body;
// the generated BASIC places them after END.
//
// swap exchanges two variables atomically — no temporary needed.
//
// randomize seeds the BASIC RND function.  Pass timer for a
// time-based seed; pass a literal for reproducible results.
program data

const NUM_CAPITALS = 5

dim country$(NUM_CAPITALS)
dim capital$(NUM_CAPITALS)

/* Load the lookup table */
for i% = 1 to NUM_CAPITALS
    read country$(i%), capital$(i%)
end for

/* Print the table */
print "Country         Capital"
print "--------------- ---------------"
for i% = 1 to NUM_CAPITALS
    print country$(i%) + "        " + capital$(i%)
end for

/* swap — sort two variables without a temp */
a% = 42
b% = 17
print "Before swap: a=" + str$(a%) + " b=" + str$(b%)
swap a%, b%
print "After swap:  a=" + str$(a%) + " b=" + str$(b%)

/* Bubble-sort the country array using swap */
for pass% = 1 to NUM_CAPITALS - 1
    for i% = 1 to NUM_CAPITALS - pass%
        if country$(i%) > country$(i% + 1) then
            swap country$(i%), country$(i% + 1)
            swap capital$(i%), capital$(i% + 1)
        end if
    end for
end for
print "Sorted by country:"
for i% = 1 to NUM_CAPITALS
    print "  " + country$(i%) + " -> " + capital$(i%)
end for

/* randomize — seed with a literal for reproducible output */
randomize 99

end

data "France",  "Paris"
data "Germany", "Berlin"
data "Japan",   "Tokyo"
data "Brazil",  "Brasilia"
data "Egypt",   "Cairo"

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/data.bas</code></summary>



```basic

10 ' BASCAL generated BASIC -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Tutorial — DATA, READ, SWAP, and RANDOMIZE
40 '
50 ' data embeds literal values directly in the program.  read consumes
60 ' them in sequence.  The data statements may appear anywhere in the program body;
70 ' the generated BASIC places them after END.
80 '
90 ' swap exchanges two variables atomically — no temporary needed.
100 '
110 ' randomize seeds the BASIC RND function.  Pass timer for a
120 ' time-based seed; pass a literal for reproducible results.

130 numCAPITALS% = 5

140 DIM country$(numCAPITALS%)
150 BCCT1% = numCAPITALS%
160 DIM capital$(numCAPITALS%)
170 BCCT2% = numCAPITALS%

180 ' Load the lookup table
190 FOR i% = 1 TO numCAPITALS%
200     READ country$(i%), capital$(i%)
210 NEXT i%

220 ' Print the table
230 PRINT "Country         Capital"
240 PRINT "--------------- ---------------"
250 FOR i% = 1 TO numCAPITALS%
260     PRINT (country$(i%) + "        ") + capital$(i%)
270 NEXT i%

280 ' swap — sort two variables without a temp
290 a% = 42
300 b% = 17
310 PRINT (("Before swap: a=" + STR$(a%)) + " b=") + STR$(b%)
320 SWAP a%, b%
330 PRINT (("After swap:  a=" + STR$(a%)) + " b=") + STR$(b%)

340 ' Bubble-sort the country array using swap
350 FOR pass% = 1 TO numCAPITALS% - 1
360     FOR i% = 1 TO numCAPITALS% - pass%
370         IF (country$(i%) > country$(i% + 1)) = 0 THEN GOTO 400
380             SWAP country$(i%), country$(i% + 1)
390             SWAP capital$(i%), capital$(i% + 1)
400         REM END IF
410     NEXT i%
420 NEXT pass%
430 PRINT "Sorted by country:"
440 FOR i% = 1 TO numCAPITALS%
450     PRINT (("  " + country$(i%)) + " -> ") + capital$(i%)
460 NEXT i%

470 ' randomize — seed with a literal for reproducible output
480 RANDOMIZE 99

490 END

500 DATA "France", "Paris"
510 DATA "Germany", "Berlin"
520 DATA "Japan", "Tokyo"
530 DATA "Brazil", "Brasilia"
540 DATA "Egypt", "Cairo"

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/data.c</code></summary>



```c

// BASCAL generated C -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
#include <stdio.h>
#include <string.h>
#include <stdlib.h>

#define BCC_STRBUF_COUNT 8
static char bcc_strbuf[BCC_STRBUF_COUNT][256];
static int bcc_strbuf_next = 0;

static int bcc_data_ptr = 0;

static char* bcc_strbuf_take(void);
static const char* bcc_mid(const char* s, int start, int length);
static const char* bcc_chr(int code);
static const char* bcc_stri(int value);
static const char* bcc_strd(double value);
static const char* bcc_read_data(void);

static int bv_i_a = 0;
static int bv_i_b = 0;
static int bv_i_i = 0;
static int bv_i_num_capitals = 0;
static int bv_i_pass = 0;
static char bv_s_capital[6][256] = {0};
static char bv_s_country[6][256] = {0};

int main(void) {
    // Tutorial — DATA, READ, SWAP, and RANDOMIZE
    //
    // data embeds literal values directly in the program.  read consumes
    // them in sequence.  The data statements may appear anywhere in the program body;
    // the generated BASIC places them after END.
    //
    // swap exchanges two variables atomically — no temporary needed.
    //
    // randomize seeds the BASIC RND function.  Pass timer for a
    // time-based seed; pass a literal for reproducible results.

    bv_i_num_capitals = 5;


    // Load the lookup table
    int bt_lim_0 = bv_i_num_capitals;
    int bt_step_0 = 1;
    for (bv_i_i = 1; bt_step_0 >= 0 ? bv_i_i <= bt_lim_0 : bv_i_i >= bt_lim_0; bv_i_i += bt_step_0) {
        snprintf(bv_s_country[(bv_i_i)], sizeof(bv_s_country[(bv_i_i)]), "%s", bcc_read_data());
        snprintf(bv_s_capital[(bv_i_i)], sizeof(bv_s_capital[(bv_i_i)]), "%s", bcc_read_data());
    }

    // Print the table
    printf("Country         Capital\n");
    printf("--------------- ---------------\n");
    int bt_lim_1 = bv_i_num_capitals;
    int bt_step_1 = 1;
    for (bv_i_i = 1; bt_step_1 >= 0 ? bv_i_i <= bt_lim_1 : bv_i_i >= bt_lim_1; bv_i_i += bt_step_1) {
        char bt_s_2[256];
        snprintf(bt_s_2, sizeof(bt_s_2), "%s%s", bv_s_country[(bv_i_i)], "        ");
        char bt_s_3[256];
        snprintf(bt_s_3, sizeof(bt_s_3), "%s%s", bt_s_2, bv_s_capital[(bv_i_i)]);
        printf("%s\n", bt_s_3);
    }

    // swap — sort two variables without a temp
    bv_i_a = 42;
    bv_i_b = 17;
    char bt_s_4[256];
    snprintf(bt_s_4, sizeof(bt_s_4), "%s%s", "Before swap: a=", bcc_stri(bv_i_a));
    char bt_s_5[256];
    snprintf(bt_s_5, sizeof(bt_s_5), "%s%s", bt_s_4, " b=");
    char bt_s_6[256];
    snprintf(bt_s_6, sizeof(bt_s_6), "%s%s", bt_s_5, bcc_stri(bv_i_b));
    printf("%s\n", bt_s_6);
    int bt_swap_7 = bv_i_a;
    bv_i_a = bv_i_b;
    bv_i_b = bt_swap_7;
    char bt_s_8[256];
    snprintf(bt_s_8, sizeof(bt_s_8), "%s%s", "After swap:  a=", bcc_stri(bv_i_a));
    char bt_s_9[256];
    snprintf(bt_s_9, sizeof(bt_s_9), "%s%s", bt_s_8, " b=");
    char bt_s_10[256];
    snprintf(bt_s_10, sizeof(bt_s_10), "%s%s", bt_s_9, bcc_stri(bv_i_b));
    printf("%s\n", bt_s_10);

    // Bubble-sort the country array using swap
    int bt_lim_11 = (bv_i_num_capitals - 1);
    int bt_step_11 = 1;
    for (bv_i_pass = 1; bt_step_11 >= 0 ? bv_i_pass <= bt_lim_11 : bv_i_pass >= bt_lim_11; bv_i_pass += bt_step_11) {
        int bt_lim_12 = (bv_i_num_capitals - bv_i_pass);
        int bt_step_12 = 1;
        for (bv_i_i = 1; bt_step_12 >= 0 ? bv_i_i <= bt_lim_12 : bv_i_i >= bt_lim_12; bv_i_i += bt_step_12) {
            if ((-(strcmp(bv_s_country[(bv_i_i)], bv_s_country[((bv_i_i + 1))]) > 0))) {
                char bt_swap_13[256];
                snprintf(bt_swap_13, sizeof(bt_swap_13), "%s", bv_s_country[(bv_i_i)]);
                snprintf(bv_s_country[(bv_i_i)], sizeof(bv_s_country[(bv_i_i)]), "%s", bv_s_country[((bv_i_i + 1))]);
                snprintf(bv_s_country[((bv_i_i + 1))], sizeof(bv_s_country[((bv_i_i + 1))]), "%s", bt_swap_13);
                char bt_swap_14[256];
                snprintf(bt_swap_14, sizeof(bt_swap_14), "%s", bv_s_capital[(bv_i_i)]);
                snprintf(bv_s_capital[(bv_i_i)], sizeof(bv_s_capital[(bv_i_i)]), "%s", bv_s_capital[((bv_i_i + 1))]);
                snprintf(bv_s_capital[((bv_i_i + 1))], sizeof(bv_s_capital[((bv_i_i + 1))]), "%s", bt_swap_14);
            }
        }
    }
    printf("Sorted by country:\n");
    int bt_lim_15 = bv_i_num_capitals;
    int bt_step_15 = 1;
    for (bv_i_i = 1; bt_step_15 >= 0 ? bv_i_i <= bt_lim_15 : bv_i_i >= bt_lim_15; bv_i_i += bt_step_15) {
        char bt_s_16[256];
        snprintf(bt_s_16, sizeof(bt_s_16), "%s%s", "  ", bv_s_country[(bv_i_i)]);
        char bt_s_17[256];
        snprintf(bt_s_17, sizeof(bt_s_17), "%s%s", bt_s_16, " -> ");
        char bt_s_18[256];
        snprintf(bt_s_18, sizeof(bt_s_18), "%s%s", bt_s_17, bv_s_capital[(bv_i_i)]);
        printf("%s\n", bt_s_18);
    }

    // randomize — seed with a literal for reproducible output
    srand((unsigned int)(99));

    return 0;

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

#define BCC_DATA_COUNT 10
static const char* bcc_data[BCC_DATA_COUNT] = { "France", "Paris", "Germany", "Berlin", "Japan", "Tokyo", "Brazil", "Brasilia", "Egypt", "Cairo" };

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
