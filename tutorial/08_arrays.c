#include <stdio.h>
#include <math.h>
#include <string.h>

#include "bcc_runtime.h"

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

    int bt_lim_0 = ((bv_i_arr_len0 - 1) - 1);
    int bt_step_0 = 1;
    for (bv_i_i = 1; bt_step_0 >= 0 ? bv_i_i <= bt_lim_0 : bv_i_i >= bt_lim_0; bv_i_i += bt_step_0) {
        bv_i_key = bv_i_arr[(bv_i_i)];
        bv_i_j = (bv_i_i - 1);
        while (((int)((long)round((double)(-(bv_i_j >= 0))) & (long)round((double)(-(bv_i_arr[(bv_i_j)] > bv_i_key)))))) {
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

    int bt_lim_1 = ((bv_i_arr_len0 - 1) - 1);
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
    int bt_lim_2 = ((bv_i_arr_len0 - 1) - 1);
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
    // Tutorial 8 — Arrays
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
