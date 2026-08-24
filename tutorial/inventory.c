#include <stdio.h>
#include <math.h>
#include <string.h>
#include <stdint.h>
#include <stdlib.h>
#include <termios.h>
#include <unistd.h>

#define BCC_STRBUF_COUNT 8
static char bcc_strbuf[BCC_STRBUF_COUNT][256];
static int bcc_strbuf_next = 0;

static int bcc_err = 0;
static int bcc_on_error_target = -1;
static int bcc_in_handler = 0;
static int bcc_resume_id = -1;
static int bcc_erl = 0;

typedef struct { int status; } bcc_result_void;

#define BCC_MAX_CHANNELS 32
static FILE* bcc_files[BCC_MAX_CHANNELS];

static char bcc_input_buf[256];

static char* bcc_strbuf_take(void);
static const char* bcc_mid(const char* s, int start, int length);
static const char* bcc_chr(int code);
static const char* bcc_stri(int value);
static const char* bcc_strd(double value);
static int bcc_instr(const char* s, const char* needle);
static const char* bcc_inkey(void);
static void bcc_read_string_field(char* field, const unsigned char* source, size_t width);
static void bcc_mki(char* out, int value);
static void bcc_mkl(char* out, int value);
static void bcc_mks(char* out, double value);
static void bcc_mkd(char* out, double value);
static int bcc_cvi(const char* s);
static int bcc_cvl(const char* s);
static float bcc_cvs(const char* s);
static double bcc_cvd(const char* s);
static int bcc_read_record(FILE* file, void* buffer, size_t reclen, long record);
static void bcc_write_record(FILE* file, const void* buffer, size_t reclen, long record);
static void bcc_pad_string_field(unsigned char* dest, const char* value, size_t width);
static int bcc_put_record_part(FILE* file, long record, const char* field_0, const char* field_1, const int16_t* field_2, const int16_t* field_3, const float* field_4);
static int bcc_get_record_part(FILE* file, long record, char* field_0, char* field_1, char* field_2, char* field_3, char* field_4);
static void bcc_color(int fg, int bg);
static void bcc_read_line(void);

static int bv_i_erl = 0;
static int bv_i_err = 0;
static int bv_i_partcount = 0;
static int bv_i_tabcol = 0;
static char bv_s_invdescbuf[256] = {0};
static char bv_s_invflagbuf[256] = {0};
static char bv_s_invpricebuf[256] = {0};
static char bv_s_invqtybuf[256] = {0};
static char bv_s_invreorderbuf[256] = {0};
static char bv_s_kp[256] = {0};

void bf_s_error(int bv_i_code, char* bcc_out);
int bf_i_isempty(const char* bv_s_flag_in);
int bf_i_partinrange(int bv_i_n);
void bf_s_readpartnumberinput(char* bcc_out);
void bf_s_readkey(char* bcc_out);
bcc_result_void bf_i_waitanykey(void);
void bf_i_showmainmenu(void);
bcc_result_void bf_i_showbadpartnumber(void);
bcc_result_void bf_i_showrangeretrymessage(void);
bcc_result_void bf_i_shownullentrymessage(const char* bv_s_partstr_in);
bcc_result_void bf_i_showpartstatus(int bv_i_partnum, const char* bv_s_desc_in, int bv_i_qty, int bv_i_reorder, float bv_f_price);
bcc_result_void bf_i_printlistheader(void);
bcc_result_void bf_i_printinventoryline(int bv_i_partnum, const char* bv_s_desc_in, int bv_i_qty, int bv_i_reorder);
bcc_result_void bf_i_printreorderheader(void);
bcc_result_void bf_i_printreorderline(int bv_i_partnum, const char* bv_s_desc_in, int bv_i_qty, int bv_i_reorder);
bcc_result_void bf_i_gatherpartdetails(int bv_i_partnum, char* bv_s_desc_in, int* bv_i_qty_in, int* bv_i_reorder_in, float* bv_f_price_in);
bcc_result_void bf_i_showaddstockscreen(int bv_i_partnum, const char* bv_s_desc_in, int bv_i_qty, int bv_i_reorder);
bcc_result_void bf_i_shownegativeqtywarning(void);
bcc_result_void bf_i_showsubtractstockscreen(int bv_i_partnum, const char* bv_s_desc_in, int bv_i_qty, int bv_i_reorder);
bcc_result_void bf_i_showoversubtractwarning(int bv_i_onhand);
bcc_result_void bf_i_checkpart(void);
bcc_result_void bf_i_editrecord(void);
bcc_result_void bf_i_listall(void);
bcc_result_void bf_i_addstock(void);
bcc_result_void bf_i_subtractstock(void);
bcc_result_void bf_i_reorderreport(void);
void bf_i_initializeinventoryfileifnew(void);
void bf_i_reportinventoryerror(int bv_i_err, int bv_i_erl);

void bf_s_error(int bv_i_code, char* bcc_out) {
    {
        int bt_sel_0 = bv_i_code;
        if ((bt_sel_0 == 2)) {
            snprintf(bcc_out, 256, "%s", "Syntax error");
            return;
        } else if ((bt_sel_0 == 3)) {
            snprintf(bcc_out, 256, "%s", "RETURN without GOSUB");
            return;
        } else if ((bt_sel_0 == 4)) {
            snprintf(bcc_out, 256, "%s", "Out of DATA");
            return;
        } else if ((bt_sel_0 == 5)) {
            snprintf(bcc_out, 256, "%s", "Illegal function call");
            return;
        } else if ((bt_sel_0 == 6)) {
            snprintf(bcc_out, 256, "%s", "Overflow");
            return;
        } else if ((bt_sel_0 == 7)) {
            snprintf(bcc_out, 256, "%s", "Out of memory");
            return;
        } else if ((bt_sel_0 == 9)) {
            snprintf(bcc_out, 256, "%s", "Subscript out of range");
            return;
        } else if ((bt_sel_0 == 10)) {
            snprintf(bcc_out, 256, "%s", "Duplicate Definition");
            return;
        } else if ((bt_sel_0 == 11)) {
            snprintf(bcc_out, 256, "%s", "Division by zero");
            return;
        } else if ((bt_sel_0 == 13)) {
            snprintf(bcc_out, 256, "%s", "Type mismatch");
            return;
        } else if ((bt_sel_0 == 14)) {
            snprintf(bcc_out, 256, "%s", "Out of string space");
            return;
        } else if ((bt_sel_0 == 19)) {
            snprintf(bcc_out, 256, "%s", "No RESUME");
            return;
        } else if ((bt_sel_0 == 20)) {
            snprintf(bcc_out, 256, "%s", "RESUME without error");
            return;
        } else if ((bt_sel_0 == 24)) {
            snprintf(bcc_out, 256, "%s", "Device timeout");
            return;
        } else if ((bt_sel_0 == 25)) {
            snprintf(bcc_out, 256, "%s", "Device fault");
            return;
        } else if ((bt_sel_0 == 27)) {
            snprintf(bcc_out, 256, "%s", "Out of paper");
            return;
        } else if ((bt_sel_0 == 52)) {
            snprintf(bcc_out, 256, "%s", "Bad file number");
            return;
        } else if ((bt_sel_0 == 53)) {
            snprintf(bcc_out, 256, "%s", "File not found");
            return;
        } else if ((bt_sel_0 == 54)) {
            snprintf(bcc_out, 256, "%s", "Bad file mode");
            return;
        } else if ((bt_sel_0 == 55)) {
            snprintf(bcc_out, 256, "%s", "File already open");
            return;
        } else if ((bt_sel_0 == 57)) {
            snprintf(bcc_out, 256, "%s", "Device I/O error");
            return;
        } else if ((bt_sel_0 == 58)) {
            snprintf(bcc_out, 256, "%s", "File already exists");
            return;
        } else if ((bt_sel_0 == 61)) {
            snprintf(bcc_out, 256, "%s", "Disk full");
            return;
        } else if ((bt_sel_0 == 62)) {
            snprintf(bcc_out, 256, "%s", "Input past end");
            return;
        } else if ((bt_sel_0 == 63)) {
            snprintf(bcc_out, 256, "%s", "Bad record number");
            return;
        } else if ((bt_sel_0 == 64)) {
            snprintf(bcc_out, 256, "%s", "Bad file name");
            return;
        } else if ((bt_sel_0 == 67)) {
            snprintf(bcc_out, 256, "%s", "Too many files");
            return;
        } else if ((bt_sel_0 == 68)) {
            snprintf(bcc_out, 256, "%s", "Device unavailable");
            return;
        } else if ((bt_sel_0 == 70)) {
            snprintf(bcc_out, 256, "%s", "Disk write protected");
            return;
        } else if ((bt_sel_0 == 71)) {
            snprintf(bcc_out, 256, "%s", "Disk not ready");
            return;
        } else if ((bt_sel_0 == 72)) {
            snprintf(bcc_out, 256, "%s", "Disk media error");
            return;
        } else if ((bt_sel_0 == 75)) {
            snprintf(bcc_out, 256, "%s", "Path/File access error");
            return;
        } else if ((bt_sel_0 == 76)) {
            snprintf(bcc_out, 256, "%s", "Path not found");
            return;
        } else {
            char bt_s_1[256];
            snprintf(bt_s_1, sizeof(bt_s_1), "%s%s", "Error ", bcc_stri(bv_i_code));
            snprintf(bcc_out, 256, "%s", bt_s_1);
            return;
        }
    }
}

int bf_i_isempty(const char* bv_s_flag_in) {
    char bv_s_flag[256];
    snprintf(bv_s_flag, sizeof(bv_s_flag), "%s", bv_s_flag_in);

    return (-(((int)(unsigned char)bv_s_flag[0]) == 255));
}

int bf_i_partinrange(int bv_i_n) {
    if (((-(bv_i_n >= 1)) && (-(bv_i_n <= bv_i_partcount)))) {
        return 1;
    }
    return 0;
}

void bf_s_readpartnumberinput(char* bcc_out) {
    char bv_s_s[256] = {0};

    printf("Input part number? ");
    bcc_read_line();
    snprintf(bv_s_s, sizeof(bv_s_s), "%s", bcc_input_buf);
    snprintf(bcc_out, 256, "%s", bv_s_s);
    return;
}

void bf_s_readkey(char* bcc_out) {
    char bv_s_k[256] = {0};

    while (1) {
        snprintf(bv_s_k, sizeof(bv_s_k), "%s", bcc_inkey());
        if ((-(strcmp(bv_s_k, "") != 0))) break;
    }
    snprintf(bcc_out, 256, "%s", bv_s_k);
    return;
}

bcc_result_void bf_i_waitanykey(void) {
    char bv_s_k[256] = {0};

    printf("\x1b[%d;%dH", 25, 10);
    printf("Press the AnyKey to continue...");
    while (1) {
        snprintf(bv_s_k, sizeof(bv_s_k), "%s", bcc_inkey());
        if ((-(strcmp(bv_s_k, "") != 0))) break;
    }
    return (bcc_result_void){ .status = 0 };
}

void bf_i_showmainmenu(void) {
    printf("\x1b[2J\x1b[H");
    bcc_color(14, 4);
    printf("\x1b[2J\x1b[H");
    printf("\x1b[%d;%dH", 6, 1);
    printf("\n");
    // `tab(n)` passes straight through to real TAB(n), same as
    // fhb's own `PRINT TAB(V) "..."` -- but only as a bare item in
    // a PRINT list, juxtaposed or `;`-separated like here. Real
    // BASCOM rejects `"literal" + tab(n) + ...` (TAB isn't a real
    // string function you can concatenate); see printListHeader()
    // and printReorderHeader() below, which need `;` between a
    // preceding string and a `tab(n)` for exactly this reason.
    printf("\x1b[%dGInventory Program\n", 30);
    printf("\n");
    printf("\x1b[%dG1......C)heck a part\n", bv_i_tabcol);
    printf("\x1b[%dG2......E)dit/overwrite/add a part\n", bv_i_tabcol);
    char bt_s_2[256];
    snprintf(bt_s_2, sizeof(bt_s_2), "%s%s", "3......L)ist all", bcc_stri(bv_i_partcount));
    char bt_s_3[256];
    snprintf(bt_s_3, sizeof(bt_s_3), "%s%s", bt_s_2, "parts");
    printf("\x1b[%dG%s\n", bv_i_tabcol, bt_s_3);
    printf("\x1b[%dG4......A)dd stock\n", bv_i_tabcol);
    printf("\x1b[%dG5......S)ubtract stock\n", bv_i_tabcol);
    printf("\x1b[%dG6......R)eorder Report\n", bv_i_tabcol);
    printf("\n");
    printf("\x1b[%dG7......eX)it to system\n", bv_i_tabcol);
}

bcc_result_void bf_i_showbadpartnumber(void) {
    printf("\x1b[2J\x1b[H");
    printf("\x1b[%d;%dH", 10, 10);
    char bt_s_4[256];
    snprintf(bt_s_4, sizeof(bt_s_4), "%s%s", "Part number is out of permissable range of 1 to", bcc_stri(bv_i_partcount));
    printf("%s\n", bt_s_4);
    return (bcc_result_void){ .status = 0 };
}

bcc_result_void bf_i_showrangeretrymessage(void) {
    printf("\x1b[%d;%dH", 10, 15);
    char bt_s_5[256];
    snprintf(bt_s_5, sizeof(bt_s_5), "%s%s", "The Part number is out of permissable range of 1 to", bcc_stri(bv_i_partcount));
    printf("%s\n", bt_s_5);
    printf("\x1b[%d;%dH", 25, 15);
    printf("Press the Anykey to reenter part number...");
    return (bcc_result_void){ .status = 0 };
}

bcc_result_void bf_i_shownullentrymessage(const char* bv_s_partstr_in) {
    char bv_s_partstr[256];
    snprintf(bv_s_partstr, sizeof(bv_s_partstr), "%s", bv_s_partstr_in);

    printf("\x1b[%d;%dH", 10, bv_i_tabcol);
    char bt_s_6[256];
    snprintf(bt_s_6, sizeof(bt_s_6), "%s%s", "Part number ", bv_s_partstr);
    char bt_s_7[256];
    snprintf(bt_s_7, sizeof(bt_s_7), "%s%s", bt_s_6, " is a null entry");
    printf("%s\n", bt_s_7);
    return (bcc_result_void){ .status = 0 };
}

bcc_result_void bf_i_showpartstatus(int bv_i_partnum, const char* bv_s_desc_in, int bv_i_qty, int bv_i_reorder, float bv_f_price) {
    char bv_s_desc[256];
    snprintf(bv_s_desc, sizeof(bv_s_desc), "%s", bv_s_desc_in);

    printf("\x1b[2J\x1b[H");
    printf("\x1b[%d;%dH", 5, 1);
    printf("\x1b[%dGInventory Status for Individual Part Number\n", bv_i_tabcol);
    printf("\x1b[%dG===========================================\n", bv_i_tabcol);
    printf("\n");
    printf("\n");
    char bt_s_8[256];
    snprintf(bt_s_8, sizeof(bt_s_8), "%s%s", "     Part number:  ", bcc_stri(bv_i_partnum));
    printf("\x1b[%dG%s\n", bv_i_tabcol, bt_s_8);
    printf("\n");
    char bt_s_9[256];
    snprintf(bt_s_9, sizeof(bt_s_9), "%s%s", "       Item name:  ", bv_s_desc);
    printf("\x1b[%dG%s\n", bv_i_tabcol, bt_s_9);
    char bt_s_10[256];
    snprintf(bt_s_10, sizeof(bt_s_10), "%s%s", "Quantity on hand:  ", bcc_stri(bv_i_qty));
    printf("\x1b[%dG%s\n", bv_i_tabcol, bt_s_10);
    char bt_s_11[256];
    snprintf(bt_s_11, sizeof(bt_s_11), "%s%s", "   Reorder level:  ", bcc_stri(bv_i_reorder));
    printf("\x1b[%dG%s\n", bv_i_tabcol, bt_s_11);
    char bt_s_12[256];
    snprintf(bt_s_12, sizeof(bt_s_12), "%s%s", "      Unit price:  ", bcc_strd(bv_f_price));
    printf("\x1b[%dG%s\n", bv_i_tabcol, bt_s_12);
    return (bcc_result_void){ .status = 0 };
}

bcc_result_void bf_i_printlistheader(void) {
    printf("\x1b[2J\x1b[H");
    char bt_s_13[256];
    snprintf(bt_s_13, sizeof(bt_s_13), "%s%s", bcc_stri(bv_i_partcount), "items");
    printf("\x1b[%dGI N V E N T O R Y   L I S T I N G\x1b[%dG%s\n", 25, 65, bt_s_13);
    printf("                                          Quantity       Reorder\n");
    printf(" Partno           Description             on hand         level\n");
    printf("\x1b[%d;%dH", 25, 1);
    printf("Press the AnyKey to scroll listing...");
    return (bcc_result_void){ .status = 0 };
}

bcc_result_void bf_i_printinventoryline(int bv_i_partnum, const char* bv_s_desc_in, int bv_i_qty, int bv_i_reorder) {
    char bv_s_desc[256];
    snprintf(bv_s_desc, sizeof(bv_s_desc), "%s", bv_s_desc_in);

    char bt_s_14[256];
    snprintf(bt_s_14, sizeof(bt_s_14), "%s%s", bcc_stri(bv_i_partnum), "  ");
    char bt_s_15[256];
    snprintf(bt_s_15, sizeof(bt_s_15), "%s%s", bt_s_14, bv_s_desc);
    char bt_s_16[256];
    snprintf(bt_s_16, sizeof(bt_s_16), "%s%s", bt_s_15, "   ");
    char bt_s_17[256];
    snprintf(bt_s_17, sizeof(bt_s_17), "%s%s", bt_s_16, bcc_stri(bv_i_qty));
    char bt_s_18[256];
    snprintf(bt_s_18, sizeof(bt_s_18), "%s%s", bt_s_17, "          ");
    char bt_s_19[256];
    snprintf(bt_s_19, sizeof(bt_s_19), "%s%s", bt_s_18, bcc_stri(bv_i_reorder));
    printf("%s\n", bt_s_19);
    return (bcc_result_void){ .status = 0 };
}

bcc_result_void bf_i_printreorderheader(void) {
    char bv_s_date[256] = {0};

    printf("\x1b[2J\x1b[H");
    printf("\x1b[%d;%dH", 1, bv_i_tabcol);
    printf("Reorder Report\x1b[%dG%s\n", 55, bv_s_date);
    printf("\n");
    printf("                                             Quantity       Reorder\n");
    printf("    Partno           Description             on hand         level\n");
    printf("   =======  ==============================   ========       =======\n");
    return (bcc_result_void){ .status = 0 };
}

bcc_result_void bf_i_printreorderline(int bv_i_partnum, const char* bv_s_desc_in, int bv_i_qty, int bv_i_reorder) {
    char bv_s_desc[256];
    snprintf(bv_s_desc, sizeof(bv_s_desc), "%s", bv_s_desc_in);

    char bt_s_20[256];
    snprintf(bt_s_20, sizeof(bt_s_20), "%s%s", "  ", bcc_stri(bv_i_partnum));
    char bt_s_21[256];
    snprintf(bt_s_21, sizeof(bt_s_21), "%s%s", bt_s_20, "  ");
    char bt_s_22[256];
    snprintf(bt_s_22, sizeof(bt_s_22), "%s%s", bt_s_21, bv_s_desc);
    char bt_s_23[256];
    snprintf(bt_s_23, sizeof(bt_s_23), "%s%s", bt_s_22, "   ");
    char bt_s_24[256];
    snprintf(bt_s_24, sizeof(bt_s_24), "%s%s", bt_s_23, bcc_stri(bv_i_qty));
    char bt_s_25[256];
    snprintf(bt_s_25, sizeof(bt_s_25), "%s%s", bt_s_24, "          ");
    char bt_s_26[256];
    snprintf(bt_s_26, sizeof(bt_s_26), "%s%s", bt_s_25, bcc_stri(bv_i_reorder));
    printf("%s\n", bt_s_26);
    return (bcc_result_void){ .status = 0 };
}

bcc_result_void bf_i_gatherpartdetails(int bv_i_partnum, char* bv_s_desc_in, int* bv_i_qty_in, int* bv_i_reorder_in, float* bv_f_price_in) {
    char bv_s_desc[256];
    snprintf(bv_s_desc, sizeof(bv_s_desc), "%s", bv_s_desc_in);
    int bv_i_qty = *bv_i_qty_in;
    int bv_i_reorder = *bv_i_reorder_in;
    float bv_f_price = *bv_f_price_in;

    printf("\x1b[2J\x1b[H");
    printf("\x1b[%d;%dH", 4, bv_i_tabcol);
    printf("Adding or Overwriting a Record\n");
    printf("\x1b[%d;%dH", 8, bv_i_tabcol);
    char bt_s_27[256];
    snprintf(bt_s_27, sizeof(bt_s_27), "%s%s", "Record/Partno", bcc_stri(bv_i_partnum));
    printf("%s\n", bt_s_27);
    printf("\x1b[%d;%dH", 11, 39);
    printf("------------------------------\n");
    printf("\x1b[%d;%dH", 10, bv_i_tabcol);
    printf("      Description? ");
    bcc_read_line();
    snprintf(bv_s_desc, sizeof(bv_s_desc), "%s", bcc_input_buf);
    printf("\x1b[%d;%dH", 12, bv_i_tabcol);
    printf("Quantity in stock? ");
    bcc_read_line();
    bv_i_qty = atoi(bcc_input_buf);
    printf("\x1b[%d;%dH", 14, bv_i_tabcol);
    printf("    Reorder level? ");
    bcc_read_line();
    bv_i_reorder = atoi(bcc_input_buf);
    printf("\x1b[%d;%dH", 16, bv_i_tabcol);
    printf("       Unit price? ");
    bcc_read_line();
    bv_f_price = atof(bcc_input_buf);
    printf("\x1b[%d;%dH", 18, bv_i_tabcol);
    printf("Is information correct (Y/N)?\n");
    snprintf(bv_s_desc_in, 256, "%s", bv_s_desc);
    *bv_i_qty_in = bv_i_qty;
    *bv_i_reorder_in = bv_i_reorder;
    *bv_f_price_in = bv_f_price;
    return (bcc_result_void){ .status = 0 };
}

bcc_result_void bf_i_showaddstockscreen(int bv_i_partnum, const char* bv_s_desc_in, int bv_i_qty, int bv_i_reorder) {
    char bv_s_desc[256];
    snprintf(bv_s_desc, sizeof(bv_s_desc), "%s", bv_s_desc_in);

    printf("\x1b[2J\x1b[H");
    printf("\x1b[%d;%dH", 4, 25);
    printf("Add to an inventory part number\n");
    printf("\x1b[%d;%dH", 5, 25);
    printf("===============================\n");
    printf("\x1b[%d;%dH", 8, bv_i_tabcol);
    char bt_s_28[256];
    snprintf(bt_s_28, sizeof(bt_s_28), "%s%s", "     Part number: ", bcc_stri(bv_i_partnum));
    printf("%s\n", bt_s_28);
    printf("\x1b[%d;%dH", 9, bv_i_tabcol);
    char bt_s_29[256];
    snprintf(bt_s_29, sizeof(bt_s_29), "%s%s", "Item description: ", bv_s_desc);
    printf("%s\n", bt_s_29);
    printf("\x1b[%d;%dH", 10, bv_i_tabcol);
    char bt_s_30[256];
    snprintf(bt_s_30, sizeof(bt_s_30), "%s%s", "Quantity on hand: ", bcc_stri(bv_i_qty));
    printf("%s\n", bt_s_30);
    printf("\x1b[%d;%dH", 11, bv_i_tabcol);
    char bt_s_31[256];
    snprintf(bt_s_31, sizeof(bt_s_31), "%s%s", "   Reorder Level: ", bcc_stri(bv_i_reorder));
    printf("%s\n", bt_s_31);
    return (bcc_result_void){ .status = 0 };
}

bcc_result_void bf_i_shownegativeqtywarning(void) {
    printf("\x1b[%d;%dH", 17, 15);
    printf("The quantity to add must NOT be a negative number\n");
    printf("\x1b[%d;%dH", 25, 1);
    printf("Please press the Anykey to reenter quantity to add...");
    return (bcc_result_void){ .status = 0 };
}

bcc_result_void bf_i_showsubtractstockscreen(int bv_i_partnum, const char* bv_s_desc_in, int bv_i_qty, int bv_i_reorder) {
    char bv_s_desc[256];
    snprintf(bv_s_desc, sizeof(bv_s_desc), "%s", bv_s_desc_in);

    printf("\x1b[2J\x1b[H");
    printf("\x1b[%d;%dH", 4, bv_i_tabcol);
    printf("Subtract an inventory part number\n");
    printf("\x1b[%d;%dH", 5, bv_i_tabcol);
    printf("=================================\n");
    printf("\x1b[%d;%dH", 8, bv_i_tabcol);
    char bt_s_32[256];
    snprintf(bt_s_32, sizeof(bt_s_32), "%s%s", "         Part number: ", bcc_stri(bv_i_partnum));
    printf("%s\n", bt_s_32);
    printf("\x1b[%d;%dH", 9, bv_i_tabcol);
    char bt_s_33[256];
    snprintf(bt_s_33, sizeof(bt_s_33), "%s%s", "    Item description: ", bv_s_desc);
    printf("%s\n", bt_s_33);
    printf("\x1b[%d;%dH", 10, bv_i_tabcol);
    char bt_s_34[256];
    snprintf(bt_s_34, sizeof(bt_s_34), "%s%s", "    Quantity on hand: ", bcc_stri(bv_i_qty));
    printf("%s\n", bt_s_34);
    printf("\x1b[%d;%dH", 11, bv_i_tabcol);
    char bt_s_35[256];
    snprintf(bt_s_35, sizeof(bt_s_35), "%s%s", "       Reorder Level: ", bcc_stri(bv_i_reorder));
    printf("%s\n", bt_s_35);
    return (bcc_result_void){ .status = 0 };
}

bcc_result_void bf_i_showoversubtractwarning(int bv_i_onhand) {
    printf("\x1b[%d;%dH", 17, 5);
    printf("The quantity to SUBTRACT must NOT result in NEGATIVE inventory\n");
    printf("\x1b[%d;%dH", 18, 5);
    char bt_s_36[256];
    snprintf(bt_s_36, sizeof(bt_s_36), "%s%s", "Only", bcc_stri(bv_i_onhand));
    char bt_s_37[256];
    snprintf(bt_s_37, sizeof(bt_s_37), "%s%s", bt_s_36, " IN STOCK");
    printf("%s\n", bt_s_37);
    printf("\x1b[%d;%dH", 25, 1);
    printf("Please press the Anykey to reenter quantity to subtract...");
    return (bcc_result_void){ .status = 0 };
}

bcc_result_void bf_i_checkpart(void) {
    float bv_f_pprice = 0;
    int bv_i_part = 0;
    int bv_i_pdesctrimi = 0;
    int bv_i_pflagtrimi = 0;
    int bv_i_pqty = 0;
    int bv_i_preorder = 0;
    char bv_s_invdescbuf[256] = {0};
    char bv_s_invflagbuf[256] = {0};
    char bv_s_invpricebuf[256] = {0};
    char bv_s_invqtybuf[256] = {0};
    char bv_s_invreorderbuf[256] = {0};
    char bv_s_partstr[256] = {0};
    char bv_s_pdesc[256] = {0};
    char bv_s_pflag[256] = {0};

    char bt_s_38[256];
    bf_s_readpartnumberinput(bt_s_38);
    snprintf(bv_s_partstr, sizeof(bv_s_partstr), "%s", bt_s_38);
    bv_i_part = ((int)round((double)(atof(bv_s_partstr))));
    if ((-(bf_i_partinrange(bv_i_part) == 0))) {
        bcc_result_void bcc_st_39 = bf_i_showbadpartnumber();
        if (bcc_st_39.status) return bcc_st_39;
        bcc_result_void bcc_st_40 = bf_i_waitanykey();
        if (bcc_st_40.status) return bcc_st_40;
        return (bcc_result_void){ .status = 0 };
    }
    // BASCAL-ism: `let p = inv[part%]` reads record `part%` of the
    // `inv` file into a local record variable `p` -- one expression
    // for what fhb's `GET #1, PART!` plus five separate field reads
    // (F$, D$, CVI(Q$), CVI(R$), CVS(P$)) did by hand. The write
    // side, `inv[part%] = { ... }` (see editRecord() below), is the
    // same sugar for PUT plus the LSET/MKx$ packing it replaces.
    // let p = inv[...]  (whole-record read)
    bcc_get_record_part(bcc_files[0], bv_i_part, bv_s_invflagbuf, bv_s_invdescbuf, bv_s_invqtybuf, bv_s_invreorderbuf, bv_s_invpricebuf);
    bv_i_pflagtrimi = ((int)strlen(bv_s_invflagbuf));
    while (((-(bv_i_pflagtrimi > 0)) && (-(strcmp(bcc_mid(bv_s_invflagbuf, bv_i_pflagtrimi, 1), " ") == 0)))) {
        bv_i_pflagtrimi = (bv_i_pflagtrimi - 1);
    }
    snprintf(bv_s_pflag, sizeof(bv_s_pflag), "%s", bcc_mid(bv_s_invflagbuf, 1, bv_i_pflagtrimi));
    bv_i_pdesctrimi = ((int)strlen(bv_s_invdescbuf));
    while (((-(bv_i_pdesctrimi > 0)) && (-(strcmp(bcc_mid(bv_s_invdescbuf, bv_i_pdesctrimi, 1), " ") == 0)))) {
        bv_i_pdesctrimi = (bv_i_pdesctrimi - 1);
    }
    snprintf(bv_s_pdesc, sizeof(bv_s_pdesc), "%s", bcc_mid(bv_s_invdescbuf, 1, bv_i_pdesctrimi));
    bv_i_pqty = bcc_cvi(bv_s_invqtybuf);
    bv_i_preorder = bcc_cvi(bv_s_invreorderbuf);
    bv_f_pprice = bcc_cvs(bv_s_invpricebuf);
    if (bf_i_isempty(bv_s_pflag)) {
        printf("\x1b[2J\x1b[H");
        printf("\x1b[%d;%dH", 10, 18);
        char bt_s_41[256];
        snprintf(bt_s_41, sizeof(bt_s_41), "%s%s", "Part number", bcc_stri(bv_i_part));
        char bt_s_42[256];
        snprintf(bt_s_42, sizeof(bt_s_42), "%s%s", bt_s_41, "is still a null entry at this time");
        printf("%s\n", bt_s_42);
        bcc_result_void bcc_st_43 = bf_i_waitanykey();
        if (bcc_st_43.status) return bcc_st_43;
        return (bcc_result_void){ .status = 0 };
    }
    bcc_result_void bcc_st_44 = bf_i_showpartstatus(bv_i_part, bv_s_pdesc, bv_i_pqty, bv_i_preorder, bv_f_pprice);
    if (bcc_st_44.status) return bcc_st_44;
    bcc_result_void bcc_st_45 = bf_i_waitanykey();
    if (bcc_st_45.status) return bcc_st_45;
    return (bcc_result_void){ .status = 0 };
}

bcc_result_void bf_i_editrecord(void) {
    float bv_f_editprice = 0;
    float bv_f_pprice = 0;
    int bv_i_editqty = 0;
    int bv_i_editreorder = 0;
    int bv_i_part = 0;
    int bv_i_pdesctrimi = 0;
    int bv_i_pflagtrimi = 0;
    int bv_i_pqty = 0;
    int bv_i_preorder = 0;
    char bv_s_editdesc[256] = {0};
    char bv_s_invdescbuf[256] = {0};
    char bv_s_invflagbuf[256] = {0};
    char bv_s_invpricebuf[256] = {0};
    char bv_s_invqtybuf[256] = {0};
    char bv_s_invreorderbuf[256] = {0};
    char bv_s_kp[256] = {0};
    char bv_s_partstr[256] = {0};
    char bv_s_pdesc[256] = {0};
    char bv_s_pflag[256] = {0};

    printf("\x1b[2J\x1b[H");
    printf("\x1b[%d;%dH", 10, bv_i_tabcol);
    char bt_s_46[256];
    bf_s_readpartnumberinput(bt_s_46);
    snprintf(bv_s_partstr, sizeof(bv_s_partstr), "%s", bt_s_46);
    bv_i_part = ((int)round((double)(atof(bv_s_partstr))));
    if ((-(bf_i_partinrange(bv_i_part) == 0))) {
        bcc_result_void bcc_st_47 = bf_i_showbadpartnumber();
        if (bcc_st_47.status) return bcc_st_47;
        bcc_result_void bcc_st_48 = bf_i_waitanykey();
        if (bcc_st_48.status) return bcc_st_48;
        return (bcc_result_void){ .status = 0 };
    }
    // let p = inv[...]  (whole-record read)
    bcc_get_record_part(bcc_files[0], bv_i_part, bv_s_invflagbuf, bv_s_invdescbuf, bv_s_invqtybuf, bv_s_invreorderbuf, bv_s_invpricebuf);
    bv_i_pflagtrimi = ((int)strlen(bv_s_invflagbuf));
    while (((-(bv_i_pflagtrimi > 0)) && (-(strcmp(bcc_mid(bv_s_invflagbuf, bv_i_pflagtrimi, 1), " ") == 0)))) {
        bv_i_pflagtrimi = (bv_i_pflagtrimi - 1);
    }
    snprintf(bv_s_pflag, sizeof(bv_s_pflag), "%s", bcc_mid(bv_s_invflagbuf, 1, bv_i_pflagtrimi));
    bv_i_pdesctrimi = ((int)strlen(bv_s_invdescbuf));
    while (((-(bv_i_pdesctrimi > 0)) && (-(strcmp(bcc_mid(bv_s_invdescbuf, bv_i_pdesctrimi, 1), " ") == 0)))) {
        bv_i_pdesctrimi = (bv_i_pdesctrimi - 1);
    }
    snprintf(bv_s_pdesc, sizeof(bv_s_pdesc), "%s", bcc_mid(bv_s_invdescbuf, 1, bv_i_pdesctrimi));
    bv_i_pqty = bcc_cvi(bv_s_invqtybuf);
    bv_i_preorder = bcc_cvi(bv_s_invreorderbuf);
    bv_f_pprice = bcc_cvs(bv_s_invpricebuf);
    if ((-(bf_i_isempty(bv_s_pflag) == 0))) {
        printf("\x1b[%d;%dH", 12, bv_i_tabcol);
        printf("Overwrite existing part data?\n");
        char bt_s_49[256];
        bf_s_readkey(bt_s_49);
        snprintf(bv_s_kp, sizeof(bv_s_kp), "%s", bt_s_49);
        if (((-(strcmp(bv_s_kp, "Y") != 0)) && (-(strcmp(bv_s_kp, "y") != 0)))) {
            return (bcc_result_void){ .status = 0 };
        }
    }

    while (1) {
        bcc_result_void bcc_st_50 = bf_i_gatherpartdetails(bv_i_part, bv_s_editdesc, &bv_i_editqty, &bv_i_editreorder, &bv_f_editprice);
        if (bcc_st_50.status) return bcc_st_50;
        char bt_s_51[256];
        bf_s_readkey(bt_s_51);
        snprintf(bv_s_kp, sizeof(bv_s_kp), "%s", bt_s_51);
        if (((-(strcmp(bv_s_kp, "Y") == 0)) || (-(strcmp(bv_s_kp, "y") == 0)))) break;
    }
    // inv[...] = { ... }  (whole-record write)
    int16_t bcc_tmp_52 = bv_i_editqty;
    int16_t bcc_tmp_53 = bv_i_editreorder;
    float bcc_tmp_54 = bv_f_editprice;
    bcc_put_record_part(bcc_files[0], bv_i_part, "1", bv_s_editdesc, &bcc_tmp_52, &bcc_tmp_53, &bcc_tmp_54);
    return (bcc_result_void){ .status = 0 };
}

bcc_result_void bf_i_listall(void) {
    float bv_f_pprice = 0;
    int bv_i_i = 0;
    int bv_i_pdesctrimi = 0;
    int bv_i_pflagtrimi = 0;
    int bv_i_pqty = 0;
    int bv_i_preorder = 0;
    int bv_i_scrollcount = 0;
    char bv_s_invdescbuf[256] = {0};
    char bv_s_invflagbuf[256] = {0};
    char bv_s_invpricebuf[256] = {0};
    char bv_s_invqtybuf[256] = {0};
    char bv_s_invreorderbuf[256] = {0};
    char bv_s_pdesc[256] = {0};
    char bv_s_pflag[256] = {0};

    bcc_result_void bcc_st_55 = bf_i_printlistheader();
    if (bcc_st_55.status) return bcc_st_55;
    bv_i_scrollcount = 0;
    int bt_lim_56 = bv_i_partcount;
    int bt_step_56 = 1;
    for (bv_i_i = 1; bt_step_56 >= 0 ? bv_i_i <= bt_lim_56 : bv_i_i >= bt_lim_56; bv_i_i += bt_step_56) {
        // let p = inv[...]  (whole-record read)
        bcc_get_record_part(bcc_files[0], bv_i_i, bv_s_invflagbuf, bv_s_invdescbuf, bv_s_invqtybuf, bv_s_invreorderbuf, bv_s_invpricebuf);
        bv_i_pflagtrimi = ((int)strlen(bv_s_invflagbuf));
        while (((-(bv_i_pflagtrimi > 0)) && (-(strcmp(bcc_mid(bv_s_invflagbuf, bv_i_pflagtrimi, 1), " ") == 0)))) {
            bv_i_pflagtrimi = (bv_i_pflagtrimi - 1);
        }
        snprintf(bv_s_pflag, sizeof(bv_s_pflag), "%s", bcc_mid(bv_s_invflagbuf, 1, bv_i_pflagtrimi));
        bv_i_pdesctrimi = ((int)strlen(bv_s_invdescbuf));
        while (((-(bv_i_pdesctrimi > 0)) && (-(strcmp(bcc_mid(bv_s_invdescbuf, bv_i_pdesctrimi, 1), " ") == 0)))) {
            bv_i_pdesctrimi = (bv_i_pdesctrimi - 1);
        }
        snprintf(bv_s_pdesc, sizeof(bv_s_pdesc), "%s", bcc_mid(bv_s_invdescbuf, 1, bv_i_pdesctrimi));
        bv_i_pqty = bcc_cvi(bv_s_invqtybuf);
        bv_i_preorder = bcc_cvi(bv_s_invreorderbuf);
        bv_f_pprice = bcc_cvs(bv_s_invpricebuf);
        bcc_result_void bcc_st_57 = bf_i_printinventoryline(bv_i_i, bv_s_pdesc, bv_i_pqty, bv_i_preorder);
        if (bcc_st_57.status) return bcc_st_57;
        bv_i_scrollcount = (bv_i_scrollcount + 1);
        if ((-(bv_i_scrollcount == 20))) {
            bcc_result_void bcc_st_58 = bf_i_waitanykey();
            if (bcc_st_58.status) return bcc_st_58;
            bv_i_scrollcount = 0;
        }
    }
    return (bcc_result_void){ .status = 0 };
}

bcc_result_void bf_i_addstock(void) {
    float bv_f_pprice = 0;
    int bv_i_addamt = 0;
    int bv_i_part = 0;
    int bv_i_pdesctrimi = 0;
    int bv_i_pflagtrimi = 0;
    int bv_i_pqty = 0;
    int bv_i_preorder = 0;
    int bv_i_validpart = 0;
    char bv_s_addstr[256] = {0};
    char bv_s_invdescbuf[256] = {0};
    char bv_s_invflagbuf[256] = {0};
    char bv_s_invpricebuf[256] = {0};
    char bv_s_invqtybuf[256] = {0};
    char bv_s_invreorderbuf[256] = {0};
    char bv_s_partstr[256] = {0};
    char bv_s_pdesc[256] = {0};
    char bv_s_pflag[256] = {0};

    printf("\x1b[2J\x1b[H");
    printf("\x1b[%d;%dH", 5, 25);
    printf("A D D I N G   S T O C K\n");

    while (1) {
        printf("\x1b[%d;%dH", 8, 25);
        char bt_s_59[256];
        bf_s_readpartnumberinput(bt_s_59);
        snprintf(bv_s_partstr, sizeof(bv_s_partstr), "%s", bt_s_59);
        bv_i_part = ((int)round((double)(atof(bv_s_partstr))));
        bv_i_validpart = bf_i_partinrange(bv_i_part);
        if ((-(bv_i_validpart == 0))) {
            bcc_result_void bcc_st_60 = bf_i_showrangeretrymessage();
            if (bcc_st_60.status) return bcc_st_60;
            char bt_s_61[256];
            bf_s_readkey(bt_s_61);
        }
        if ((-(bv_i_validpart != 0))) break;
    }

    // let p = inv[...]  (whole-record read)
    bcc_get_record_part(bcc_files[0], bv_i_part, bv_s_invflagbuf, bv_s_invdescbuf, bv_s_invqtybuf, bv_s_invreorderbuf, bv_s_invpricebuf);
    bv_i_pflagtrimi = ((int)strlen(bv_s_invflagbuf));
    while (((-(bv_i_pflagtrimi > 0)) && (-(strcmp(bcc_mid(bv_s_invflagbuf, bv_i_pflagtrimi, 1), " ") == 0)))) {
        bv_i_pflagtrimi = (bv_i_pflagtrimi - 1);
    }
    snprintf(bv_s_pflag, sizeof(bv_s_pflag), "%s", bcc_mid(bv_s_invflagbuf, 1, bv_i_pflagtrimi));
    bv_i_pdesctrimi = ((int)strlen(bv_s_invdescbuf));
    while (((-(bv_i_pdesctrimi > 0)) && (-(strcmp(bcc_mid(bv_s_invdescbuf, bv_i_pdesctrimi, 1), " ") == 0)))) {
        bv_i_pdesctrimi = (bv_i_pdesctrimi - 1);
    }
    snprintf(bv_s_pdesc, sizeof(bv_s_pdesc), "%s", bcc_mid(bv_s_invdescbuf, 1, bv_i_pdesctrimi));
    bv_i_pqty = bcc_cvi(bv_s_invqtybuf);
    bv_i_preorder = bcc_cvi(bv_s_invreorderbuf);
    bv_f_pprice = bcc_cvs(bv_s_invpricebuf);
    if (bf_i_isempty(bv_s_pflag)) {
        bcc_result_void bcc_st_62 = bf_i_shownullentrymessage(bv_s_partstr);
        if (bcc_st_62.status) return bcc_st_62;
        char bt_s_63[256];
        bf_s_readkey(bt_s_63);
        return (bcc_result_void){ .status = 0 };
    }

    while (1) {
        bcc_result_void bcc_st_64 = bf_i_showaddstockscreen(bv_i_part, bv_s_pdesc, bv_i_pqty, bv_i_preorder);
        if (bcc_st_64.status) return bcc_st_64;
        printf("\x1b[%d;%dH", 14, bv_i_tabcol);
        printf(" Quantity to add? ");
        bcc_read_line();
        snprintf(bv_s_addstr, sizeof(bv_s_addstr), "%s", bcc_input_buf);
        bv_i_addamt = ((int)round((double)(atof(bv_s_addstr))));
        if ((-(bv_i_addamt < 0))) {
            bcc_result_void bcc_st_65 = bf_i_shownegativeqtywarning();
            if (bcc_st_65.status) return bcc_st_65;
            char bt_s_66[256];
            bf_s_readkey(bt_s_66);
        }
        if ((-(bv_i_addamt >= 0))) break;
    }

    bv_i_pqty = (bv_i_pqty + bv_i_addamt);
    // inv[...] = p  (write back a let-bound record)
    int16_t bcc_tmp_67 = bv_i_pqty;
    int16_t bcc_tmp_68 = bv_i_preorder;
    float bcc_tmp_69 = bv_f_pprice;
    bcc_put_record_part(bcc_files[0], bv_i_part, bv_s_pflag, bv_s_pdesc, &bcc_tmp_67, &bcc_tmp_68, &bcc_tmp_69);
    return (bcc_result_void){ .status = 0 };
}

bcc_result_void bf_i_subtractstock(void) {
    float bv_f_pprice = 0;
    int bv_i_oversubtract = 0;
    int bv_i_part = 0;
    int bv_i_pdesctrimi = 0;
    int bv_i_pflagtrimi = 0;
    int bv_i_pqty = 0;
    int bv_i_preorder = 0;
    int bv_i_subamt = 0;
    int bv_i_validpart = 0;
    char bv_s_invdescbuf[256] = {0};
    char bv_s_invflagbuf[256] = {0};
    char bv_s_invpricebuf[256] = {0};
    char bv_s_invqtybuf[256] = {0};
    char bv_s_invreorderbuf[256] = {0};
    char bv_s_partstr[256] = {0};
    char bv_s_pdesc[256] = {0};
    char bv_s_pflag[256] = {0};
    char bv_s_substr[256] = {0};

    printf("\x1b[2J\x1b[H");
    printf("\x1b[%d;%dH", 5, 20);
    printf("S U B T R A C T I N G    S T O C K\n");

    while (1) {
        printf("\x1b[%d;%dH", 8, 25);
        char bt_s_70[256];
        bf_s_readpartnumberinput(bt_s_70);
        snprintf(bv_s_partstr, sizeof(bv_s_partstr), "%s", bt_s_70);
        bv_i_part = ((int)round((double)(atof(bv_s_partstr))));
        bv_i_validpart = bf_i_partinrange(bv_i_part);
        if ((-(bv_i_validpart == 0))) {
            bcc_result_void bcc_st_71 = bf_i_showrangeretrymessage();
            if (bcc_st_71.status) return bcc_st_71;
            char bt_s_72[256];
            bf_s_readkey(bt_s_72);
        }
        if ((-(bv_i_validpart != 0))) break;
    }

    // let p = inv[...]  (whole-record read)
    bcc_get_record_part(bcc_files[0], bv_i_part, bv_s_invflagbuf, bv_s_invdescbuf, bv_s_invqtybuf, bv_s_invreorderbuf, bv_s_invpricebuf);
    bv_i_pflagtrimi = ((int)strlen(bv_s_invflagbuf));
    while (((-(bv_i_pflagtrimi > 0)) && (-(strcmp(bcc_mid(bv_s_invflagbuf, bv_i_pflagtrimi, 1), " ") == 0)))) {
        bv_i_pflagtrimi = (bv_i_pflagtrimi - 1);
    }
    snprintf(bv_s_pflag, sizeof(bv_s_pflag), "%s", bcc_mid(bv_s_invflagbuf, 1, bv_i_pflagtrimi));
    bv_i_pdesctrimi = ((int)strlen(bv_s_invdescbuf));
    while (((-(bv_i_pdesctrimi > 0)) && (-(strcmp(bcc_mid(bv_s_invdescbuf, bv_i_pdesctrimi, 1), " ") == 0)))) {
        bv_i_pdesctrimi = (bv_i_pdesctrimi - 1);
    }
    snprintf(bv_s_pdesc, sizeof(bv_s_pdesc), "%s", bcc_mid(bv_s_invdescbuf, 1, bv_i_pdesctrimi));
    bv_i_pqty = bcc_cvi(bv_s_invqtybuf);
    bv_i_preorder = bcc_cvi(bv_s_invreorderbuf);
    bv_f_pprice = bcc_cvs(bv_s_invpricebuf);
    if (bf_i_isempty(bv_s_pflag)) {
        bcc_result_void bcc_st_73 = bf_i_shownullentrymessage(bv_s_partstr);
        if (bcc_st_73.status) return bcc_st_73;
        char bt_s_74[256];
        bf_s_readkey(bt_s_74);
        return (bcc_result_void){ .status = 0 };
    }

    while (1) {
        bcc_result_void bcc_st_75 = bf_i_showsubtractstockscreen(bv_i_part, bv_s_pdesc, bv_i_pqty, bv_i_preorder);
        if (bcc_st_75.status) return bcc_st_75;
        printf("\x1b[%d;%dH", 14, bv_i_tabcol);
        printf("Quantity to subtract? ");
        bcc_read_line();
        snprintf(bv_s_substr, sizeof(bv_s_substr), "%s", bcc_input_buf);
        bv_i_subamt = ((int)round((double)(atof(bv_s_substr))));
        bv_i_oversubtract = 0;
        if (((-(bv_i_subamt >= 0)) && (-((bv_i_pqty - bv_i_subamt) < 0)))) {
            bv_i_oversubtract = 1;
            bcc_result_void bcc_st_76 = bf_i_showoversubtractwarning(bv_i_pqty);
            if (bcc_st_76.status) return bcc_st_76;
            char bt_s_77[256];
            bf_s_readkey(bt_s_77);
        }
        if (((-(bv_i_subamt >= 0)) && (-(bv_i_oversubtract == 0)))) break;
    }

    bv_i_pqty = (bv_i_pqty - bv_i_subamt);
    if ((-(bv_i_pqty <= bv_i_preorder))) {
        printf("\x1b[%d;%dH", 16, bv_i_tabcol);
    }
    char bt_s_78[256];
    snprintf(bt_s_78, sizeof(bt_s_78), "%s%s", "quantity now", bcc_stri(bv_i_pqty));
    char bt_s_79[256];
    snprintf(bt_s_79, sizeof(bt_s_79), "%s%s", bt_s_78, " reorder level");
    char bt_s_80[256];
    snprintf(bt_s_80, sizeof(bt_s_80), "%s%s", bt_s_79, bcc_stri(bv_i_preorder));
    printf("%s\n", bt_s_80);
    // inv[...] = p  (write back a let-bound record)
    int16_t bcc_tmp_81 = bv_i_pqty;
    int16_t bcc_tmp_82 = bv_i_preorder;
    float bcc_tmp_83 = bv_f_pprice;
    bcc_put_record_part(bcc_files[0], bv_i_part, bv_s_pflag, bv_s_pdesc, &bcc_tmp_81, &bcc_tmp_82, &bcc_tmp_83);
    return (bcc_result_void){ .status = 0 };
}

bcc_result_void bf_i_reorderreport(void) {
    float bv_f_pprice = 0;
    int bv_i_i = 0;
    int bv_i_pdesctrimi = 0;
    int bv_i_pflagtrimi = 0;
    int bv_i_pqty = 0;
    int bv_i_preorder = 0;
    int bv_i_reportlinecount = 0;
    char bv_s_invdescbuf[256] = {0};
    char bv_s_invflagbuf[256] = {0};
    char bv_s_invpricebuf[256] = {0};
    char bv_s_invqtybuf[256] = {0};
    char bv_s_invreorderbuf[256] = {0};
    char bv_s_pdesc[256] = {0};
    char bv_s_pflag[256] = {0};

    bcc_result_void bcc_st_84 = bf_i_printreorderheader();
    if (bcc_st_84.status) return bcc_st_84;
    bv_i_reportlinecount = 0;
    int bt_lim_85 = bv_i_partcount;
    int bt_step_85 = 1;
    for (bv_i_i = 1; bt_step_85 >= 0 ? bv_i_i <= bt_lim_85 : bv_i_i >= bt_lim_85; bv_i_i += bt_step_85) {
        // let p = inv[...]  (whole-record read)
        bcc_get_record_part(bcc_files[0], bv_i_i, bv_s_invflagbuf, bv_s_invdescbuf, bv_s_invqtybuf, bv_s_invreorderbuf, bv_s_invpricebuf);
        bv_i_pflagtrimi = ((int)strlen(bv_s_invflagbuf));
        while (((-(bv_i_pflagtrimi > 0)) && (-(strcmp(bcc_mid(bv_s_invflagbuf, bv_i_pflagtrimi, 1), " ") == 0)))) {
            bv_i_pflagtrimi = (bv_i_pflagtrimi - 1);
        }
        snprintf(bv_s_pflag, sizeof(bv_s_pflag), "%s", bcc_mid(bv_s_invflagbuf, 1, bv_i_pflagtrimi));
        bv_i_pdesctrimi = ((int)strlen(bv_s_invdescbuf));
        while (((-(bv_i_pdesctrimi > 0)) && (-(strcmp(bcc_mid(bv_s_invdescbuf, bv_i_pdesctrimi, 1), " ") == 0)))) {
            bv_i_pdesctrimi = (bv_i_pdesctrimi - 1);
        }
        snprintf(bv_s_pdesc, sizeof(bv_s_pdesc), "%s", bcc_mid(bv_s_invdescbuf, 1, bv_i_pdesctrimi));
        bv_i_pqty = bcc_cvi(bv_s_invqtybuf);
        bv_i_preorder = bcc_cvi(bv_s_invreorderbuf);
        bv_f_pprice = bcc_cvs(bv_s_invpricebuf);
        if ((-(bv_i_pqty < bv_i_preorder))) {
            bcc_result_void bcc_st_86 = bf_i_printreorderline(bv_i_i, bv_s_pdesc, bv_i_pqty, bv_i_preorder);
            if (bcc_st_86.status) return bcc_st_86;
            bv_i_reportlinecount = (bv_i_reportlinecount + 1);
            if ((-(bv_i_reportlinecount > 15))) {
                bcc_result_void bcc_st_87 = bf_i_waitanykey();
                if (bcc_st_87.status) return bcc_st_87;
                bv_i_reportlinecount = 0;
            }
        }
    }
    bcc_result_void bcc_st_88 = bf_i_waitanykey();
    if (bcc_st_88.status) return bcc_st_88;
    return (bcc_result_void){ .status = 0 };
}

void bf_i_initializeinventoryfileifnew(void) {
    float bv_f_pprice = 0;
    int bv_i_i = 0;
    int bv_i_pdesctrimi = 0;
    int bv_i_pflagtrimi = 0;
    int bv_i_pqty = 0;
    int bv_i_preorder = 0;
    char bv_s_invdescbuf[256] = {0};
    char bv_s_invflagbuf[256] = {0};
    char bv_s_invpricebuf[256] = {0};
    char bv_s_invqtybuf[256] = {0};
    char bv_s_invreorderbuf[256] = {0};
    char bv_s_pdesc[256] = {0};
    char bv_s_pflag[256] = {0};

    // let p = inv[...]  (whole-record read)
    bcc_get_record_part(bcc_files[0], 1, bv_s_invflagbuf, bv_s_invdescbuf, bv_s_invqtybuf, bv_s_invreorderbuf, bv_s_invpricebuf);
    bv_i_pflagtrimi = ((int)strlen(bv_s_invflagbuf));
    while (((-(bv_i_pflagtrimi > 0)) && (-(strcmp(bcc_mid(bv_s_invflagbuf, bv_i_pflagtrimi, 1), " ") == 0)))) {
        bv_i_pflagtrimi = (bv_i_pflagtrimi - 1);
    }
    snprintf(bv_s_pflag, sizeof(bv_s_pflag), "%s", bcc_mid(bv_s_invflagbuf, 1, bv_i_pflagtrimi));
    bv_i_pdesctrimi = ((int)strlen(bv_s_invdescbuf));
    while (((-(bv_i_pdesctrimi > 0)) && (-(strcmp(bcc_mid(bv_s_invdescbuf, bv_i_pdesctrimi, 1), " ") == 0)))) {
        bv_i_pdesctrimi = (bv_i_pdesctrimi - 1);
    }
    snprintf(bv_s_pdesc, sizeof(bv_s_pdesc), "%s", bcc_mid(bv_s_invdescbuf, 1, bv_i_pdesctrimi));
    bv_i_pqty = bcc_cvi(bv_s_invqtybuf);
    bv_i_preorder = bcc_cvi(bv_s_invreorderbuf);
    bv_f_pprice = bcc_cvs(bv_s_invpricebuf);
    if ((-(((int)(unsigned char)bv_s_pflag[0]) == 0))) {
        int bt_lim_89 = bv_i_partcount;
        int bt_step_89 = 1;
        for (bv_i_i = 1; bt_step_89 >= 0 ? bv_i_i <= bt_lim_89 : bv_i_i >= bt_lim_89; bv_i_i += bt_step_89) {
            // inv[...] = { ... }  (whole-record write)
            int16_t bcc_tmp_90 = 0;
            int16_t bcc_tmp_91 = 0;
            float bcc_tmp_92 = 0;
            bcc_put_record_part(bcc_files[0], bv_i_i, bcc_chr(255), "", &bcc_tmp_90, &bcc_tmp_91, &bcc_tmp_92);
        }
    }
}

void bf_i_reportinventoryerror(int bv_i_err, int bv_i_erl) {
    char bv_s_k[256] = {0};

    printf("\x1b[%d;%dH", 25, 1);
    char bt_s_93[256];
    snprintf(bt_s_93, sizeof(bt_s_93), "%s%s", "There has been an error on line", bcc_stri(bv_i_erl));
    char bt_s_94[256];
    snprintf(bt_s_94, sizeof(bt_s_94), "%s%s", bt_s_93, ": ");
    char bt_s_95[256];
    bf_s_error(bv_i_err, bt_s_95);
    char bt_s_96[256];
    snprintf(bt_s_96, sizeof(bt_s_96), "%s%s", bt_s_94, bt_s_95);
    printf("%s\n", bt_s_96);
    char bt_s_97[256];
    bf_s_readkey(bt_s_97);
    snprintf(bv_s_k, sizeof(bv_s_k), "%s", bt_s_97);
}

int main(void) {
    setvbuf(stdin, NULL, _IONBF, 0);
    // Maps an ERR code to its classic MBASIC/GW-BASIC/BASCOM message. Compiles
    // and links on a real IBM BASIC Compiler 2.00 as ERROR$, but silently
    // returns an empty string at runtime (verified under dosbox-x) -- so BASCAL
    // ships a working implementation.
    //
    // Covers the classic error codes an ON ERROR GOTO + ERR handler is
    // realistically going to hit -- not the full table, but every code common
    // enough to be worth a real message instead of falling through to the
    // generic one.
    //
    // Deliberately NOT a scalar method (see GitHub issue #41, which asked for
    // this decision to be recorded either way): code% is an opaque lookup key,
    // not a value the call is naturally "operating on" the way ltrim$/rtrim$/
    // ucase$/lcase$ operate on their string -- code%.error() would read as if
    // the *error code itself* has a message, when really this is a lookup
    // table keyed by that code. Stays an ordinary function.

    // ============================================================
    // INVENTORY.BCL -- Random-Access Inventory Program
    //
    // A BASCAL reconstruction of "Example program for RANDOM ACCESS
    // FILE study", by fhb, 8/19/98, from Joseph Sixpack's GW-BASIC
    // programs page (part of his "Last Book of GW-Basic" collection):
    // http://www.geocities.ws/joseph_sixpack/binventory.html
    // fhb's own header comment credits the original as "suggested
    // from MS-BASIC manual".
    //
    // This is a reconstruction, not a line-by-line port -- some
    // original pieces have no BASCAL equivalent and were dropped
    // rather than approximated:
    // - The GOTO-driven "subroutine roadmap" dispatcher at the top
    // of fhb's listing (a `LIST 110-320` etc. navigation aid for
    // editing in the GW-BASIC interpreter) has no meaning once the
    // program is structured into named function/procedure blocks.
    // - `KEY OFF` / `KEY I,""` (clearing the function-key soft-label
    // row) and `VIEW PRINT` (scroll-region windowing for the list
    // screen) are interpreter/console features BASCAL doesn't
    // expose.
    // - fhb's own hand-rolled numeric-ERR-code-to-message lookup table
    // (ERR=1 "Input value overflow", ERR=2 "Syntax error", ... ERR=25)
    // is replaced below by BASCAL's com.bascal.stdlib.error library
    // (ERROR$(code%)) -- same idea, BASCAL's own table; it still
    // doesn't decode ERL, which errorTrap() reports as the raw line
    // number.
    // - fhb's one-time "hidden" datafile initializer (PUT-ing 100
    // blank, CHR$(255)-flagged records) is reproduced below as
    // initializeInventoryFileIfNew(), called once at program entry --
    // inven.dat no longer has to be pre-populated by hand.
    // - The three original tab-position constants (T=20, U=25,
    // V=30) are collapsed into a single `tabCol% = 20`; a couple of
    // screens that used U=25 in the original (see showAddStockScreen
    // below) keep 25 as a literal rather than reusing tabCol%.
    //
    // Tracks parts in a fixed 100-record file: check status, add,
    // edit, add/subtract stock, and a reorder report.
    //
    // Error handling uses try/catch (GitHub issue #60), not the raw `on
    // error goto` / `resume next` fhb's original relies on: a failed menu
    // action is abandoned outright and the program returns straight to the
    // main menu, rather than resuming at the exact instruction after
    // whatever failed -- see reportInventoryError() below and
    // tutorial/inventory_try_catch.draft's own header comment for why. This
    // is a real, deliberate behavior change from an earlier on-error-goto
    // version of this file, which *was* verified against real BASCOM 2.00
    // under dosbox-x (only with the /E and /X switches -- error trapping
    // isn't linked in by default); the try/catch shape below transpiles to
    // the same ON ERROR GOTO/RESUME primitives BASCOM accepts, but hasn't
    // itself been independently re-verified against a real BASCOM compile.
    // ============================================================


    // BASCAL-ism: the record/file DSL. `record ... end record` plus
    // `file ... as ... = open(...)` below replace fhb's manual
    // FIELD #1,1 AS F$,30 AS D$,2 AS Q$,... buffer layout entirely --
    // bcc computes the field widths and record LEN from this
    // declaration and generates the FIELD statement itself. Named
    // field access (`p.flag`, `p.qty`, ...) and whole-record
    // read/write via `inv[n]` (see checkPart() below) replace fhb's
    // manual GET/PUT plus LSET/RSET and MKI$/MKS$/CVI$/CVS$ packing.

    // BASCAL-ism: `const` is a real compile-time constant, not a plain
    // variable assignment like fhb's `N=100` / `T=20` -- it can never
    // be reassigned, and resolves to the same value everywhere,
    // including inside every function/procedure below, with no
    // `global` declaration needed.
    bv_i_partcount = 100;
    bv_i_tabcol = 20;

    // `file ... = open(...)` is sugar for OPEN ... FOR RANDOM AS #n
    // LEN = <record width> plus the FIELD statement fhb wrote out by
    // hand at his line 550.
    // file inv as Part = open(...)  [39 bytes/record]
    bcc_files[0] = fopen("inven.dat", "rb+");
    if (!bcc_files[0]) bcc_files[0] = fopen("inven.dat", "wb+");
    if (!bcc_files[0]) {
        fprintf(stderr, "could not open %s for random access\n", "inven.dat");
        exit(1);
    }

    // -------------------- Pure functions (no file access) --------------------

    // BASCAL-ism: `function ... end function` with `return` replaces
    // fhb's convention of a GOSUB target plus a bare RETURN -- there's
    // no separate "subroutine label" and no shared/global result
    // variable to manage by hand; `isEmpty%(...)` is called like an
    // ordinary expression at every use below (e.g. `isEmpty%(p.flag)`).
    // A record whose flag byte is CHR$(255) is an empty/never-used slot.

    // BASCAL-ism: `&&` and `||` are short-circuit AND/OR -- real
    // MBASIC/BASCOM only has bitwise AND/OR (which fhb relies on here
    // too, since `PART!<1 OR PART!>N!` never short-circuits anyway).
    // BASCAL lowers `&&`/`||` into the equivalent branching so the
    // short-circuit *is* real at the generated-BASIC level; see the
    // manual's "Short-Circuit && and ||" section
    // (https://johnjoeallen.github.io/bascal/manual/).


    // -------------------- Keyboard input --------------------

    // BASCAL-ism: `do ... loop until` is a structured post-check loop
    // replacing fhb's `730 KP$=INKEY$:IF KP$="" THEN 730` GOTO-polling
    // idiom. `inkey$` itself is the real INKEY$ builtin passed straight
    // through, resolving correctly from inside a function/procedure
    // body like this one -- every menu action below calls
    // readKey$()/waitAnyKey() rather than polling INKEY$ inline.


    // -------------------- Display procedures --------------------










    // byref scalar parameters: gatherPartDetails writes the four editable
    // fields for a part directly back into the caller's variables.





    // -------------------- Menu actions --------------------







    // fhb's own one-time "hidden" datafile initializer PUT-ing 100 blank,
    // CHR$(255)-flagged records (see the header note above) -- reproduced
    // here so inven.dat no longer has to be pre-populated by hand before
    // running this program. A brand-new file OPEN created just now (rather
    // than one that already existed) reads back as all-zero bytes: record
    // 1's flag byte is CHR$(0), never CHR$(255) -- the one signal an
    // already-populated file (whose record 1 flag is always either
    // CHR$(255), still an empty slot, or a real part's own "1") could never
    // produce, so it's what isEmpty%() itself can't use (see its own
    // header note) but this one-time check safely can.

    // -------------------- Program entry --------------------

    printf("\x1b[2J\x1b[H");
    bf_i_initializeinventoryfileifnew();

    while (1) {
        bf_i_showmainmenu();
        char bt_s_98[256];
        bf_s_readkey(bt_s_98);
        snprintf(bv_s_kp, sizeof(bv_s_kp), "%s", bt_s_98);
        if ((-(bcc_instr("1234567cCeElLaAsSrRxX", bv_s_kp) != 0))) {
            // BASCAL-ism: `select case` replaces fhb's chain of eight
            // `IF VAL(KP$)=n OR KP$="x" OR KP$="X" THEN GOTO ...` lines
            // (his 770-840) with one multi-way dispatch.
            //
            // BASCAL-ism: `try`/`catch` (issue #60) replaces fhb's own global
            // `ON ERROR GOTO` trap. A failed menu action is abandoned outright
            // here -- the `catch` below runs, then execution continues right
            // after `end try`, back at `loop until` -- rather than resuming at
            // the exact instruction after whatever failed inside checkPart()/
            // editRecord()/etc. the way fhb's `RESUME NEXT` did. See
            // reportInventoryError() below and tutorial/inventory_try_catch.
            // draft's own header comment for why that arbitrary resume-point
            // behavior isn't something try/catch reproduces.
            bcc_on_error_target = 0;
            {
                char bt_sel_99[256];
                snprintf(bt_sel_99, sizeof(bt_sel_99), "%s", bv_s_kp);
                if ((strcmp(bt_sel_99, "1") == 0) || (strcmp(bt_sel_99, "c") == 0) || (strcmp(bt_sel_99, "C") == 0)) {
                    bcc_result_void bcc_st_100 = bf_i_checkpart();
                    if (bcc_st_100.status) goto bcc_try_0_catch;
                } else if ((strcmp(bt_sel_99, "2") == 0) || (strcmp(bt_sel_99, "e") == 0) || (strcmp(bt_sel_99, "E") == 0)) {
                    bcc_result_void bcc_st_101 = bf_i_editrecord();
                    if (bcc_st_101.status) goto bcc_try_0_catch;
                } else if ((strcmp(bt_sel_99, "3") == 0) || (strcmp(bt_sel_99, "l") == 0) || (strcmp(bt_sel_99, "L") == 0)) {
                    bcc_result_void bcc_st_102 = bf_i_listall();
                    if (bcc_st_102.status) goto bcc_try_0_catch;
                } else if ((strcmp(bt_sel_99, "4") == 0) || (strcmp(bt_sel_99, "a") == 0) || (strcmp(bt_sel_99, "A") == 0)) {
                    bcc_result_void bcc_st_103 = bf_i_addstock();
                    if (bcc_st_103.status) goto bcc_try_0_catch;
                } else if ((strcmp(bt_sel_99, "5") == 0) || (strcmp(bt_sel_99, "s") == 0) || (strcmp(bt_sel_99, "S") == 0)) {
                    bcc_result_void bcc_st_104 = bf_i_subtractstock();
                    if (bcc_st_104.status) goto bcc_try_0_catch;
                } else if ((strcmp(bt_sel_99, "6") == 0) || (strcmp(bt_sel_99, "r") == 0) || (strcmp(bt_sel_99, "R") == 0)) {
                    bcc_result_void bcc_st_105 = bf_i_reorderreport();
                    if (bcc_st_105.status) goto bcc_try_0_catch;
                } else if ((strcmp(bt_sel_99, "7") == 0) || (strcmp(bt_sel_99, "x") == 0) || (strcmp(bt_sel_99, "X") == 0)) {
                    // BASCAL-ism: `inv.close()` is sugar for `CLOSE #1`,
                    // matching fhb's own `90 CLOSE:SYSTEM`. fhb's original
                    // also had a separate "Quit to BASIC" option (his own
                    // 7, returning to the interpreter's command prompt
                    // rather than exiting to DOS) -- dropped here: a
                    // compiled program has no interpreter to return to,
                    // so it was never anything but a second spelling of
                    // this same close-and-exit action.
                    // inv.close()
                    fclose(bcc_files[0]);
                    bcc_files[0] = NULL;
                    exit(0);
                }
            }
            bcc_on_error_target = -1;
            goto bcc_try_0_end;
            bcc_try_0_catch: ;
            bcc_in_handler = 0;
            bcc_on_error_target = -1;
            bv_i_err = bcc_err;
            bv_i_erl = bcc_erl;
            bf_i_reportinventoryerror(bv_i_err, bv_i_erl);
            bcc_try_0_end: ;
        }
    }

    // -------------------- Error handling --------------------
    // err%/erl% are ordinary locals scoped to the `catch` block above, not
    // aliases for the ambient (readable-anywhere) `err`/`erl` pseudo-
    // variables `on error goto` uses -- see `Statement::TryCatch`'s own doc
    // comment in ast.rs. Passed straight through to ERROR$ here like fhb's
    // own ERR/ERL (his 3390: "an error on line";ERL), decoded through
    // BASCAL's own com.bascal.stdlib.error (ERROR$) instead of fhb's
    // hand-rolled lookup table -- see the header note above. try/catch
    // itself isn't documented in the manual yet (GitHub issue #60 tracks
    // the still-unfinished C-target work; the manual page can follow once
    // that lands) -- see ast.rs's own `Statement::TryCatch` doc comment for
    // the full semantics meanwhile.
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

static int bcc_instr(const char* s, const char* needle) {
    const char* found = strstr(s, needle);
    return found ? (int)(found - s) + 1 : 0;
}

static const char* bcc_inkey(void) {
    struct termios orig, raw;
    tcgetattr(STDIN_FILENO, &orig);
    raw = orig;
    raw.c_lflag &= ~(ICANON | ECHO);
    raw.c_cc[VMIN] = 0;
    raw.c_cc[VTIME] = 0;
    tcsetattr(STDIN_FILENO, TCSANOW, &raw);

    static char buf[2];
    unsigned char c;
    ssize_t n = read(STDIN_FILENO, &c, 1);
    if (n == 1) {
        buf[0] = (char)c;
        buf[1] = 0;
    } else {
        buf[0] = 0;
    }

    tcsetattr(STDIN_FILENO, TCSANOW, &orig);
    return buf;
}

static void bcc_read_string_field(char* field, const unsigned char* source, size_t width) {
    memcpy(field, source, width);
    field[width] = 0;
    while (width > 0 && field[width - 1] == ' ') field[--width] = 0;
}

static void bcc_mki(char* out, int value) {
    int16_t v = (int16_t)value;
    memcpy(out, &v, 2);
}

static void bcc_mkl(char* out, int value) {
    int32_t v = (int32_t)value;
    memcpy(out, &v, 4);
}

static void bcc_mks(char* out, double value) {
    float v = (float)value;
    memcpy(out, &v, 4);
}

static void bcc_mkd(char* out, double value) {
    memcpy(out, &value, 8);
}

static int bcc_cvi(const char* s) {
    int16_t v;
    memcpy(&v, s, 2);
    return (int)v;
}

static int bcc_cvl(const char* s) {
    int32_t v;
    memcpy(&v, s, 4);
    return (int)v;
}

static float bcc_cvs(const char* s) {
    float v;
    memcpy(&v, s, 4);
    return v;
}

static double bcc_cvd(const char* s) {
    double v;
    memcpy(&v, s, 8);
    return v;
}

static int bcc_read_record(FILE* file, void* buffer, size_t reclen, long record) {
    if (fseek(file, (record - 1) * (long)reclen, SEEK_SET) != 0) return 0;
    return fread(buffer, 1, reclen, file) == reclen;
}

static void bcc_write_record(FILE* file, const void* buffer, size_t reclen, long record) {
    fseek(file, (record - 1) * (long)reclen, SEEK_SET);
    fwrite(buffer, 1, reclen, file);
}

static void bcc_pad_string_field(unsigned char* dest, const char* value, size_t width) {
    size_t len = strlen(value);
    if (len > width) len = width;
    memcpy(dest, value, len);
    memset(dest + len, ' ', width - len);
}

static int bcc_put_record_part(FILE* file, long record, const char* field_0, const char* field_1, const int16_t* field_2, const int16_t* field_3, const float* field_4) {
    unsigned char buffer[39];
    if ((!field_0 || !field_1 || !field_2 || !field_3 || !field_4) && !bcc_read_record(file, buffer, 39, record)) return 0;
    if (field_0) bcc_pad_string_field(buffer + 0, field_0, 1);
    if (field_1) bcc_pad_string_field(buffer + 1, field_1, 30);
    (void)(field_2 && memcpy(buffer + 31, field_2, 2));
    (void)(field_3 && memcpy(buffer + 33, field_3, 2));
    (void)(field_4 && memcpy(buffer + 35, field_4, 4));
    bcc_write_record(file, buffer, 39, record);
    return 1;
}

static int bcc_get_record_part(FILE* file, long record, char* field_0, char* field_1, char* field_2, char* field_3, char* field_4) {
    unsigned char buffer[39];
    if (!bcc_read_record(file, buffer, 39, record)) return 0;
    bcc_read_string_field(field_0, buffer + 0, 1);
    bcc_read_string_field(field_1, buffer + 1, 30);
    memcpy(field_2, buffer + 31, 2);
    field_2[2] = 0;
    memcpy(field_3, buffer + 33, 2);
    field_3[2] = 0;
    memcpy(field_4, buffer + 35, 4);
    field_4[4] = 0;
    return 1;
}

static const int bcc_ansi_fg[16] = {30, 34, 32, 36, 31, 35, 33, 37, 90, 94, 92, 96, 91, 95, 93, 97};
static const int bcc_ansi_bg[8] = {40, 44, 42, 46, 41, 45, 43, 47};
static int bcc_color_used = 0;

static void bcc_color_reset(void) {
    printf("\x1b[0m");
}

static void bcc_color(int fg, int bg) {
    if (!bcc_color_used) {
        atexit(bcc_color_reset);
        bcc_color_used = 1;
    }
    printf("\x1b[%dm", bcc_ansi_fg[fg & 15]);
    if (bg >= 0) {
        printf("\x1b[%dm", bcc_ansi_bg[bg & 7]);
    }
}

static void bcc_read_line(void) {
    if (fgets(bcc_input_buf, sizeof(bcc_input_buf), stdin) == NULL) {
        bcc_input_buf[0] = 0;
        return;
    }
    bcc_input_buf[strcspn(bcc_input_buf, "\r\n")] = 0;
}

