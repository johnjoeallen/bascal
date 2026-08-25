[Home](../../) / [Tutorials](../) / Arrays

<div class="prose" markdown="1">

Generated sources: [BCL](https://github.com/johnjoeallen/bascal/blob/main/tutorial/08_arrays.bcl), [BASIC](https://github.com/johnjoeallen/bascal/blob/main/tutorial/08_arrays.bas), [C](https://github.com/johnjoeallen/bascal/blob/main/tutorial/08_arrays.c), and [JVM assembly](https://github.com/johnjoeallen/bascal/blob/main/tutorial/08_arrays.j).

`dim name%(size)` declares a 1-D array of `size + 1` elements, indexed `0..size`; `dim name%(rows, cols)` declares a 2-D array, and more dimensions are allowed. An array parameter must declare its rank with one `?` per dimension — `arr%(?)` for 1-D, `grid%(?, ?)` for 2-D — and at the call site you just write the plain array name, no `()` and no size argument needed: the transpiler already knows the parameter is an array from its declaration, and carries its size alongside it automatically. Use `sizeof(arr%)` inside the function body wherever the size is needed. This isn't a reference — BASCAL copies elements in before the call, and copies them back out after only if the parameter is declared `byref`; unmarked (`byval`) is the default and leaves the caller's array untouched.

</div>

<div class="snippet" markdown="1">

### Insertion sort, in place

```bascal
function insertionSort%(byref arr%(?))
    for i% = 1 to sizeof(arr%) - 1
        key% = arr%(i%)
        j%   = i% - 1
        while j% >= 0 && arr%(j%) > key%
            arr%(j% + 1) = arr%(j%)
            j% = j% - 1
        end while
        arr%(j% + 1) = key%
    end for
    return 0
end function
```

</div>

<div class="snippet" markdown="1">

### Calling it, and a 2-D array

```bascal
dim data%(N%)
' ... populate data%(0..N%-1) ...
dummy% = insertionSort%(data%)   ' arr% is byref, so data% comes back sorted

dim identity%(2, 2)
identity%(1, 1) = 1
```

</div>



[← Functions](07_functions.md)  ·  [Data, Read, Restore, Swap →](09_data.md)


<!-- BEGIN generated tutorial source -->

<details class="source-embed" markdown="1">

<summary><code>tutorial/08_arrays.bcl</code></summary>



```bascal

// Tutorial — Arrays
//
// dim name%(size) declares a 1-D array of size+1 elements, indexed 0..size.
// dim name%(rows, cols) declares a 2-D array; more dimensions are allowed.
// Array elements are accessed with parentheses: arr%(i%) or grid%(r%, c%).
//
// An array parameter must declare its rank with one ? per dimension:
// arr%(?) for 1-D, grid%(?, ?) for 2-D, and so on. At the call site, just
// write the plain array name -- no () and no size argument needed; the
// compiler already knows that parameter is an array from its declaration,
// and carries its size alongside it automatically. Use sizeof(arr%) inside
// the function body wherever the size is needed.
//
// An array parameter defaults to byval: the function gets its own private
// copy, and changes never reach the caller.  Write byref to copy the
// result back out after the call -- insertionSort% below needs it, since
// its whole job is to mutate the caller's array in place.
program arrays

/* Declare and populate */
const N% = 6
dim data%(N%)

data%(0) = 64
data%(1) = 25
data%(2) = 12
data%(3) = 22
data%(4) =  3
data%(5) = 11

/* Insertion sort — sorts data%() in place */
// arr% -- array to sort; byref because it's mutated in place
function insertionSort%(byref arr%(?))
    for i% = 1 to sizeof(arr%) - 1
        key% = arr%(i%)
        j%   = i% - 1
        while j% >= 0 && arr%(j%) > key%
            arr%(j% + 1) = arr%(j%)
            j% = j% - 1
        end while
        arr%(j% + 1) = key%
    end for
    return 0
end function

/* Linear search — returns index or -1 */
// arr%    -- array to search; byval, since indexOf% only reads it
// target% -- value to search for
function indexOf%(arr%(?), target%)
    for i% = 0 to sizeof(arr%) - 1
        if arr%(i%) = target% then
            return i%
        end if
    end for
    return -1
end function

/* Print the array on one line as  [ a b c ... ] */
// arr% -- array to print; byval, since printArray% only reads it
function printArray%(arr%(?))
    line$ = "["
    for i% = 0 to sizeof(arr%) - 1
        line$ = line$ + " " + str$(arr%(i%))
    end for
    print line$ + " ]"
    return 0
end function

/* Before sort */
print "Before: "
dummy% = printArray%(data%)

/* Sort and show */
dummy% = insertionSort%(data%)
print "After:  "
dummy% = printArray%(data%)

/* Search */
target% = 22
idx% = indexOf%(data%, target%)
if idx% >= 0 then
    print str$(target%) + " found at index " + str$(idx%)
else
    print str$(target%) + " not found"
end if

/* 2-D array — 3×3 identity matrix */
dim identity%(2, 2)
for r% = 0 to 2
    for c% = 0 to 2
        if r% = c% then
            identity%(r%, c%) = 1
        else
            identity%(r%, c%) = 0
        end if
    end for
end for

print "Identity matrix:"
for r% = 0 to 2
    print identity%(r%, 0); identity%(r%, 1); identity%(r%, 2)
end for

end

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/08_arrays.bas</code></summary>



```basic

10 ' BASCAL generated BASIC -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Storage for array parameters, sized to fit every call site
40 DIM insertionsortArr0%(6)
50 DIM indexofArr0%(6)
60 DIM printarrayArr0%(6)

70 ' Tutorial — Arrays
80 '
90 ' dim name%(size) declares a 1-D array of size+1 elements, indexed 0..size.
100 ' dim name%(rows, cols) declares a 2-D array; more dimensions are allowed.
110 ' Array elements are accessed with parentheses: arr%(i%) or grid%(r%, c%).
120 '
130 ' An array parameter must declare its rank with one ? per dimension:
140 ' arr%(?) for 1-D, grid%(?, ?) for 2-D, and so on. At the call site, just
150 ' write the plain array name -- no () and no size argument needed; the
160 ' compiler already knows that parameter is an array from its declaration,
170 ' and carries its size alongside it automatically. Use sizeof(arr%) inside
180 ' the function body wherever the size is needed.
190 '
200 ' An array parameter defaults to byval: the function gets its own private
210 ' copy, and changes never reach the caller.  Write byref to copy the
220 ' result back out after the call -- insertionSort% below needs it, since
230 ' its whole job is to mutate the caller's array in place.

240 ' Declare and populate
250 n% = 6
260 DIM data%(n%)
270 BCCT1% = n%

280 data%(0) = 64
290 data%(1) = 25
300 data%(2) = 12
310 data%(3) = 22
320 data%(4) = 3
330 data%(5) = 11

340 ' Insertion sort — sorts data%() in place
350 ' arr% -- array to sort; byref because it's mutated in place

360 ' Linear search — returns index or -1
370 ' arr%    -- array to search; byval, since indexOf% only reads it
380 ' target% -- value to search for

390 ' Print the array on one line as  [ a b c ... ]
400 ' arr% -- array to print; byval, since printArray% only reads it

410 ' Before sort
420 PRINT "Before: "
430 printarrayArrDim00% = BCCT1%
440 IF printarrayArrDim00% > 6 THEN PRINT "runtime error: `arr%` of `printArray%` needs "; printarrayArrDim00%; " elements along axis 0, but its storage only holds 6" : STOP

450 ' copy array argument into transpiled function storage: data%() -> printarrayArr0%()
460 FOR BCCT2% = 0 TO printarrayArrDim00%
470     printarrayArr0%(BCCT2%) = data%(BCCT2%)
480 NEXT BCCT2%

490 GOSUB 1310
500 dummy% = printarrayResult0%

510 ' Sort and show
520 insertionsortArrDim00% = BCCT1%
530 IF insertionsortArrDim00% > 6 THEN PRINT "runtime error: `arr%` of `insertionSort%` needs "; insertionsortArrDim00%; " elements along axis 0, but its storage only holds 6" : STOP

540 ' copy array argument into transpiled function storage: data%() -> insertionsortArr0%()
550 FOR BCCT3% = 0 TO insertionsortArrDim00%
560     insertionsortArr0%(BCCT3%) = data%(BCCT3%)
570 NEXT BCCT3%

580 GOSUB 1060

590 ' copy mutated array argument back to caller storage: insertionsortArr0%() -> data%()
600 FOR BCCT4% = 0 TO insertionsortArrDim00%
610     data%(BCCT4%) = insertionsortArr0%(BCCT4%)
620 NEXT BCCT4%

630 dummy% = insertionsortResult0%
640 PRINT "After:  "
650 printarrayArrDim00% = BCCT1%
660 IF printarrayArrDim00% > 6 THEN PRINT "runtime error: `arr%` of `printArray%` needs "; printarrayArrDim00%; " elements along axis 0, but its storage only holds 6" : STOP

670 ' copy array argument into transpiled function storage: data%() -> printarrayArr0%()
680 FOR BCCT5% = 0 TO printarrayArrDim00%
690     printarrayArr0%(BCCT5%) = data%(BCCT5%)
700 NEXT BCCT5%

710 GOSUB 1310
720 dummy% = printarrayResult0%

730 ' Search
740 target% = 22
750 indexofTarget0% = target%
760 indexofArrDim00% = BCCT1%
770 IF indexofArrDim00% > 6 THEN PRINT "runtime error: `arr%` of `indexOf%` needs "; indexofArrDim00%; " elements along axis 0, but its storage only holds 6" : STOP

780 ' copy array argument into transpiled function storage: data%() -> indexofArr0%()
790 FOR BCCT6% = 0 TO indexofArrDim00%
800     indexofArr0%(BCCT6%) = data%(BCCT6%)
810 NEXT BCCT6%

820 GOSUB 1210
830 idx% = indexofResult0%
840 IF (idx% >= 0) = 0 THEN GOTO 870
850     PRINT (STR$(target%) + " found at index ") + STR$(idx%)
860     GOTO 880
870     PRINT STR$(target%) + " not found"
880 REM END IF

890 ' 2-D array — 3×3 identity matrix
900 DIM identity%(2, 2)
910 FOR r% = 0 TO 2
920     FOR c% = 0 TO 2
930         IF (r% = c%) = 0 THEN GOTO 960
940             identity%(r%, c%) = 1
950             GOTO 970
960             identity%(r%, c%) = 0
970         REM END IF
980     NEXT c%
990 NEXT r%

1000 PRINT "Identity matrix:"
1010 FOR r% = 0 TO 2
1020     PRINT identity%(r%, 0); identity%(r%, 1); identity%(r%, 2)
1030 NEXT r%

1040 END

1050 ' function insertionsort%(arr%)
1060     FOR insertionsortI0% = 1 TO (insertionsortArrDim00% + 1) - 1
1070         insertionsortKey0% = insertionsortArr0%(insertionsortI0%)
1080         insertionsortJ0% = insertionsortI0% - 1
1090         IF (insertionsortJ0% >= 0) = 0 THEN GOTO 1140
1100         IF (insertionsortArr0%(insertionsortJ0%) > insertionsortKey0%) = 0 THEN GOTO 1140
1110             insertionsortArr0%(insertionsortJ0% + 1) = insertionsortArr0%(insertionsortJ0%)
1120             insertionsortJ0% = insertionsortJ0% - 1
1130             GOTO 1090
1140         REM END WHILE
1150         insertionsortArr0%(insertionsortJ0% + 1) = insertionsortKey0%
1160     NEXT insertionsortI0%
1170     insertionsortResult0% = 0
1180     RETURN
1190 ' end function insertionsort%

1200 ' function indexof%(arr%, target%)
1210     FOR indexofI0% = 0 TO (indexofArrDim00% + 1) - 1
1220         IF (indexofArr0%(indexofI0%) = indexofTarget0%) = 0 THEN GOTO 1250
1230             indexofResult0% = indexofI0%
1240             RETURN
1250         REM END IF
1260     NEXT indexofI0%
1270     indexofResult0% = -1
1280     RETURN
1290 ' end function indexof%

1300 ' function printarray%(arr%)
1310     printarrayLine0$ = "["
1320     FOR printarrayI0% = 0 TO (printarrayArrDim00% + 1) - 1
1330         printarrayLine0$ = (printarrayLine0$ + " ") + STR$(printarrayArr0%(printarrayI0%))
1340     NEXT printarrayI0%
1350     PRINT printarrayLine0$ + " ]"
1360     printarrayResult0% = 0
1370     RETURN
1380 ' end function printarray%

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/08_arrays.c</code></summary>



```c

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

static int bv_i_c = 0;
static int bv_i_dummy = 0;
static int bv_i_idx = 0;
static int bv_i_n = 0;
static int bv_i_r = 0;
static int bv_i_target = 0;
static int bv_i_data[7] = {0};
static int bv_i_identity[3][3] = {0};

int bf_i_insertionsort(int* bv_i_arr, int bv_i_arr_len0);
int bf_i_indexof(int* bv_i_arr_in, int bv_i_arr_len0, int bv_i_target);
int bf_i_printarray(int* bv_i_arr_in, int bv_i_arr_len0);

int bf_i_insertionsort(int* bv_i_arr, int bv_i_arr_len0) {
    int bv_i_i = 0;
    int bv_i_j = 0;
    int bv_i_key = 0;

    int bt_lim_0 = (((bv_i_arr_len0 - 1) + 1) - 1);
    int bt_step_0 = 1;
    for (bv_i_i = 1; bt_step_0 >= 0 ? bv_i_i <= bt_lim_0 : bv_i_i >= bt_lim_0; bv_i_i += bt_step_0) {
        bv_i_key = bv_i_arr[(bv_i_i)];
        bv_i_j = (bv_i_i - 1);
        while (((-(bv_i_j >= 0)) && (-(bv_i_arr[(bv_i_j)] > bv_i_key)))) {
            bv_i_arr[((bv_i_j + 1))] = bv_i_arr[(bv_i_j)];
            bv_i_j = (bv_i_j - 1);
        }
        bv_i_arr[((bv_i_j + 1))] = bv_i_key;
    }
    return 0;
}

int bf_i_indexof(int* bv_i_arr_in, int bv_i_arr_len0, int bv_i_target) {
    int bv_i_arr[7] = {0};
    for (int bcc_i = 0; bcc_i < bv_i_arr_len0; bcc_i++) { bv_i_arr[bcc_i] = bv_i_arr_in[bcc_i]; }
    int bv_i_i = 0;

    int bt_lim_1 = (((bv_i_arr_len0 - 1) + 1) - 1);
    int bt_step_1 = 1;
    for (bv_i_i = 0; bt_step_1 >= 0 ? bv_i_i <= bt_lim_1 : bv_i_i >= bt_lim_1; bv_i_i += bt_step_1) {
        if ((-(bv_i_arr[(bv_i_i)] == bv_i_target))) {
            return bv_i_i;
        }
    }
    return -(1);
}

int bf_i_printarray(int* bv_i_arr_in, int bv_i_arr_len0) {
    int bv_i_arr[7] = {0};
    for (int bcc_i = 0; bcc_i < bv_i_arr_len0; bcc_i++) { bv_i_arr[bcc_i] = bv_i_arr_in[bcc_i]; }
    int bv_i_i = 0;
    char bv_s_line[256] = {0};

    snprintf(bv_s_line, sizeof(bv_s_line), "%s", "[");
    int bt_lim_2 = (((bv_i_arr_len0 - 1) + 1) - 1);
    int bt_step_2 = 1;
    for (bv_i_i = 0; bt_step_2 >= 0 ? bv_i_i <= bt_lim_2 : bv_i_i >= bt_lim_2; bv_i_i += bt_step_2) {
        char bt_s_3[256];
        snprintf(bt_s_3, sizeof(bt_s_3), "%s%s", bv_s_line, " ");
        char bt_s_4[256];
        snprintf(bt_s_4, sizeof(bt_s_4), "%s%s", bt_s_3, bcc_stri(bv_i_arr[(bv_i_i)]));
        snprintf(bv_s_line, sizeof(bv_s_line), "%s", bt_s_4);
    }
    char bt_s_5[256];
    snprintf(bt_s_5, sizeof(bt_s_5), "%s%s", bv_s_line, " ]");
    printf("%s\n", bt_s_5);
    return 0;
}

int main(void) {
    // Tutorial — Arrays
    //
    // dim name%(size) declares a 1-D array of size+1 elements, indexed 0..size.
    // dim name%(rows, cols) declares a 2-D array; more dimensions are allowed.
    // Array elements are accessed with parentheses: arr%(i%) or grid%(r%, c%).
    //
    // An array parameter must declare its rank with one ? per dimension:
    // arr%(?) for 1-D, grid%(?, ?) for 2-D, and so on. At the call site, just
    // write the plain array name -- no () and no size argument needed; the
    // compiler already knows that parameter is an array from its declaration,
    // and carries its size alongside it automatically. Use sizeof(arr%) inside
    // the function body wherever the size is needed.
    //
    // An array parameter defaults to byval: the function gets its own private
    // copy, and changes never reach the caller.  Write byref to copy the
    // result back out after the call -- insertionSort% below needs it, since
    // its whole job is to mutate the caller's array in place.

    // Declare and populate
    bv_i_n = 6;

    bv_i_data[(0)] = 64;
    bv_i_data[(1)] = 25;
    bv_i_data[(2)] = 12;
    bv_i_data[(3)] = 22;
    bv_i_data[(4)] = 3;
    bv_i_data[(5)] = 11;

    // Insertion sort — sorts data%() in place
    // arr% -- array to sort; byref because it's mutated in place

    // Linear search — returns index or -1
    // arr%    -- array to search; byval, since indexOf% only reads it
    // target% -- value to search for

    // Print the array on one line as  [ a b c ... ]
    // arr% -- array to print; byval, since printArray% only reads it

    // Before sort
    printf("Before: \n");
    bv_i_dummy = bf_i_printarray(bv_i_data, 7);

    // Sort and show
    bv_i_dummy = bf_i_insertionsort(bv_i_data, 7);
    printf("After:  \n");
    bv_i_dummy = bf_i_printarray(bv_i_data, 7);

    // Search
    bv_i_target = 22;
    bv_i_idx = bf_i_indexof(bv_i_data, 7, bv_i_target);
    if ((-(bv_i_idx >= 0))) {
        char bt_s_6[256];
        snprintf(bt_s_6, sizeof(bt_s_6), "%s%s", bcc_stri(bv_i_target), " found at index ");
        char bt_s_7[256];
        snprintf(bt_s_7, sizeof(bt_s_7), "%s%s", bt_s_6, bcc_stri(bv_i_idx));
        printf("%s\n", bt_s_7);
    } else {
        char bt_s_8[256];
        snprintf(bt_s_8, sizeof(bt_s_8), "%s%s", bcc_stri(bv_i_target), " not found");
        printf("%s\n", bt_s_8);
    }

    // 2-D array — 3×3 identity matrix
    int bt_lim_9 = 2;
    int bt_step_9 = 1;
    for (bv_i_r = 0; bt_step_9 >= 0 ? bv_i_r <= bt_lim_9 : bv_i_r >= bt_lim_9; bv_i_r += bt_step_9) {
        int bt_lim_10 = 2;
        int bt_step_10 = 1;
        for (bv_i_c = 0; bt_step_10 >= 0 ? bv_i_c <= bt_lim_10 : bv_i_c >= bt_lim_10; bv_i_c += bt_step_10) {
            if ((-(bv_i_r == bv_i_c))) {
                bv_i_identity[(bv_i_r)][(bv_i_c)] = 1;
            } else {
                bv_i_identity[(bv_i_r)][(bv_i_c)] = 0;
            }
        }
    }

    printf("Identity matrix:\n");
    int bt_lim_11 = 2;
    int bt_step_11 = 1;
    for (bv_i_r = 0; bt_step_11 >= 0 ? bv_i_r <= bt_lim_11 : bv_i_r >= bt_lim_11; bv_i_r += bt_step_11) {
        printf("%d%d%d\n", bv_i_identity[(bv_i_r)][(0)], bv_i_identity[(bv_i_r)][(1)], bv_i_identity[(bv_i_r)][(2)]);
    }

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


```



</details>

<!-- END generated tutorial source -->
