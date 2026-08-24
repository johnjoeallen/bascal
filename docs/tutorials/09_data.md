[Home](../../) / [Tutorials](../) / Data, Read, Restore, Swap

<div class="prose" markdown="1">

`data` embeds literal values directly in the program; `read` consumes them in sequence; `restore` rewinds the pointer so the same data can be read again. `data` statements can appear anywhere in the source — the generated BASIC places them after `END`. `swap` exchanges two variables (including array elements) without a temporary.

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



[← Arrays](08_arrays.md)  ·  [File Input and Output →](10_files.md)


<!-- BEGIN generated tutorial source -->

<details class="source-embed" markdown="1">

<summary><code>tutorial/09_data.bcl</code></summary>



```bascal

// Tutorial — data, read, restore, swap, randomize
//
// data embeds literal values directly in the program.  read consumes
// them in sequence.  restore rewinds the pointer so data can be read
// again.  The data statements may appear anywhere in the program body;
// the generated BASIC places them after END.
//
// swap exchanges two variables atomically — no temporary needed.
//
// randomize seeds the BASIC RND function.  Pass timer for a
// time-based seed; pass a literal for reproducible results.
program data

const numCapitals% = 5

dim country$(numCapitals%)
dim capital$(numCapitals%)

/* Load the lookup table */
for i% = 1 to numCapitals%
    read country$(i%), capital$(i%)
end for

/* Print the table */
print "Country         Capital"
print "--------------- ---------------"
for i% = 1 to numCapitals%
    print country$(i%) + "        " + capital$(i%)
end for

/* restore lets us re-read from the beginning */
restore
read firstCountry$, firstCapital$
print "First entry re-read: " + firstCountry$ + " -> " + firstCapital$

/* swap — sort two variables without a temp */
a% = 42
b% = 17
print "Before swap: a=" + str$(a%) + " b=" + str$(b%)
swap a%, b%
print "After swap:  a=" + str$(a%) + " b=" + str$(b%)

/* Bubble-sort the country array using swap */
for pass% = 1 to numCapitals% - 1
    for i% = 1 to numCapitals% - pass%
        if country$(i%) > country$(i% + 1) then
            swap country$(i%), country$(i% + 1)
            swap capital$(i%), capital$(i% + 1)
        end if
    end for
end for
print "Sorted by country:"
for i% = 1 to numCapitals%
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

<summary><code>tutorial/09_data.bas</code></summary>



```basic

10 ' BASCAL generated BASIC -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Tutorial — data, read, restore, swap, randomize
40 '
50 ' data embeds literal values directly in the program.  read consumes
60 ' them in sequence.  restore rewinds the pointer so data can be read
70 ' again.  The data statements may appear anywhere in the program body;
80 ' the generated BASIC places them after END.
90 '
100 ' swap exchanges two variables atomically — no temporary needed.
110 '
120 ' randomize seeds the BASIC RND function.  Pass timer for a
130 ' time-based seed; pass a literal for reproducible results.

140 numcapitals% = 5

150 DIM country$(numcapitals%)
160 BCCT1% = numcapitals%
170 DIM capital$(numcapitals%)
180 BCCT2% = numcapitals%

190 ' Load the lookup table
200 FOR i% = 1 TO numcapitals%
210     READ country$(i%), capital$(i%)
220 NEXT i%

230 ' Print the table
240 PRINT "Country         Capital"
250 PRINT "--------------- ---------------"
260 FOR i% = 1 TO numcapitals%
270     PRINT (country$(i%) + "        ") + capital$(i%)
280 NEXT i%

290 ' restore lets us re-read from the beginning
300 RESTORE
310 READ firstcountry$, firstcapital$
320 PRINT (("First entry re-read: " + firstcountry$) + " -> ") + firstcapital$

330 ' swap — sort two variables without a temp
340 a% = 42
350 b% = 17
360 PRINT (("Before swap: a=" + STR$(a%)) + " b=") + STR$(b%)
370 SWAP a%, b%
380 PRINT (("After swap:  a=" + STR$(a%)) + " b=") + STR$(b%)

390 ' Bubble-sort the country array using swap
400 FOR pass% = 1 TO numcapitals% - 1
410     FOR i% = 1 TO numcapitals% - pass%
420         IF (country$(i%) > country$(i% + 1)) = 0 THEN GOTO 450
430             SWAP country$(i%), country$(i% + 1)
440             SWAP capital$(i%), capital$(i% + 1)
450         REM END IF
460     NEXT i%
470 NEXT pass%
480 PRINT "Sorted by country:"
490 FOR i% = 1 TO numcapitals%
500     PRINT (("  " + country$(i%)) + " -> ") + capital$(i%)
510 NEXT i%

520 ' randomize — seed with a literal for reproducible output
530 RANDOMIZE 99

540 END

550 DATA "France", "Paris"
560 DATA "Germany", "Berlin"
570 DATA "Japan", "Tokyo"
580 DATA "Brazil", "Brasilia"
590 DATA "Egypt", "Cairo"

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/09_data.c</code></summary>



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
static int bv_i_numcapitals = 0;
static int bv_i_pass = 0;
static char bv_s_firstcapital[256] = {0};
static char bv_s_firstcountry[256] = {0};
static char bv_s_capital[6][256] = {0};
static char bv_s_country[6][256] = {0};

int main(void) {
    // Tutorial — data, read, restore, swap, randomize
    //
    // data embeds literal values directly in the program.  read consumes
    // them in sequence.  restore rewinds the pointer so data can be read
    // again.  The data statements may appear anywhere in the program body;
    // the generated BASIC places them after END.
    //
    // swap exchanges two variables atomically — no temporary needed.
    //
    // randomize seeds the BASIC RND function.  Pass timer for a
    // time-based seed; pass a literal for reproducible results.

    bv_i_numcapitals = 5;


    // Load the lookup table
    int bt_lim_0 = bv_i_numcapitals;
    int bt_step_0 = 1;
    for (bv_i_i = 1; bt_step_0 >= 0 ? bv_i_i <= bt_lim_0 : bv_i_i >= bt_lim_0; bv_i_i += bt_step_0) {
        snprintf(bv_s_country[(bv_i_i)], sizeof(bv_s_country[(bv_i_i)]), "%s", bcc_read_data());
        snprintf(bv_s_capital[(bv_i_i)], sizeof(bv_s_capital[(bv_i_i)]), "%s", bcc_read_data());
    }

    // Print the table
    printf("Country         Capital\n");
    printf("--------------- ---------------\n");
    int bt_lim_1 = bv_i_numcapitals;
    int bt_step_1 = 1;
    for (bv_i_i = 1; bt_step_1 >= 0 ? bv_i_i <= bt_lim_1 : bv_i_i >= bt_lim_1; bv_i_i += bt_step_1) {
        char bt_s_2[256];
        snprintf(bt_s_2, sizeof(bt_s_2), "%s%s", bv_s_country[(bv_i_i)], "        ");
        char bt_s_3[256];
        snprintf(bt_s_3, sizeof(bt_s_3), "%s%s", bt_s_2, bv_s_capital[(bv_i_i)]);
        printf("%s\n", bt_s_3);
    }

    // restore lets us re-read from the beginning
    bcc_data_ptr = 0;
    snprintf(bv_s_firstcountry, sizeof(bv_s_firstcountry), "%s", bcc_read_data());
    snprintf(bv_s_firstcapital, sizeof(bv_s_firstcapital), "%s", bcc_read_data());
    char bt_s_4[256];
    snprintf(bt_s_4, sizeof(bt_s_4), "%s%s", "First entry re-read: ", bv_s_firstcountry);
    char bt_s_5[256];
    snprintf(bt_s_5, sizeof(bt_s_5), "%s%s", bt_s_4, " -> ");
    char bt_s_6[256];
    snprintf(bt_s_6, sizeof(bt_s_6), "%s%s", bt_s_5, bv_s_firstcapital);
    printf("%s\n", bt_s_6);

    // swap — sort two variables without a temp
    bv_i_a = 42;
    bv_i_b = 17;
    char bt_s_7[256];
    snprintf(bt_s_7, sizeof(bt_s_7), "%s%s", "Before swap: a=", bcc_stri(bv_i_a));
    char bt_s_8[256];
    snprintf(bt_s_8, sizeof(bt_s_8), "%s%s", bt_s_7, " b=");
    char bt_s_9[256];
    snprintf(bt_s_9, sizeof(bt_s_9), "%s%s", bt_s_8, bcc_stri(bv_i_b));
    printf("%s\n", bt_s_9);
    int bt_swap_10 = bv_i_a;
    bv_i_a = bv_i_b;
    bv_i_b = bt_swap_10;
    char bt_s_11[256];
    snprintf(bt_s_11, sizeof(bt_s_11), "%s%s", "After swap:  a=", bcc_stri(bv_i_a));
    char bt_s_12[256];
    snprintf(bt_s_12, sizeof(bt_s_12), "%s%s", bt_s_11, " b=");
    char bt_s_13[256];
    snprintf(bt_s_13, sizeof(bt_s_13), "%s%s", bt_s_12, bcc_stri(bv_i_b));
    printf("%s\n", bt_s_13);

    // Bubble-sort the country array using swap
    int bt_lim_14 = (bv_i_numcapitals - 1);
    int bt_step_14 = 1;
    for (bv_i_pass = 1; bt_step_14 >= 0 ? bv_i_pass <= bt_lim_14 : bv_i_pass >= bt_lim_14; bv_i_pass += bt_step_14) {
        int bt_lim_15 = (bv_i_numcapitals - bv_i_pass);
        int bt_step_15 = 1;
        for (bv_i_i = 1; bt_step_15 >= 0 ? bv_i_i <= bt_lim_15 : bv_i_i >= bt_lim_15; bv_i_i += bt_step_15) {
            if ((-(strcmp(bv_s_country[(bv_i_i)], bv_s_country[((bv_i_i + 1))]) > 0))) {
                char bt_swap_16[256];
                snprintf(bt_swap_16, sizeof(bt_swap_16), "%s", bv_s_country[(bv_i_i)]);
                snprintf(bv_s_country[(bv_i_i)], sizeof(bv_s_country[(bv_i_i)]), "%s", bv_s_country[((bv_i_i + 1))]);
                snprintf(bv_s_country[((bv_i_i + 1))], sizeof(bv_s_country[((bv_i_i + 1))]), "%s", bt_swap_16);
                char bt_swap_17[256];
                snprintf(bt_swap_17, sizeof(bt_swap_17), "%s", bv_s_capital[(bv_i_i)]);
                snprintf(bv_s_capital[(bv_i_i)], sizeof(bv_s_capital[(bv_i_i)]), "%s", bv_s_capital[((bv_i_i + 1))]);
                snprintf(bv_s_capital[((bv_i_i + 1))], sizeof(bv_s_capital[((bv_i_i + 1))]), "%s", bt_swap_17);
            }
        }
    }
    printf("Sorted by country:\n");
    int bt_lim_18 = bv_i_numcapitals;
    int bt_step_18 = 1;
    for (bv_i_i = 1; bt_step_18 >= 0 ? bv_i_i <= bt_lim_18 : bv_i_i >= bt_lim_18; bv_i_i += bt_step_18) {
        char bt_s_19[256];
        snprintf(bt_s_19, sizeof(bt_s_19), "%s%s", "  ", bv_s_country[(bv_i_i)]);
        char bt_s_20[256];
        snprintf(bt_s_20, sizeof(bt_s_20), "%s%s", bt_s_19, " -> ");
        char bt_s_21[256];
        snprintf(bt_s_21, sizeof(bt_s_21), "%s%s", bt_s_20, bv_s_capital[(bv_i_i)]);
        printf("%s\n", bt_s_21);
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
