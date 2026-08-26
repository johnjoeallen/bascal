// BASCAL generated C -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
#include <stdio.h>
#include <math.h>
#include <string.h>
#include <stdint.h>
#include <stdlib.h>
#if defined(_WIN32)
#include <conio.h>
#else
#include <termios.h>
#include <unistd.h>
#endif

#define BCC_STRBUF_COUNT 8
static char bcc_strbuf[BCC_STRBUF_COUNT][256];
static int bcc_strbuf_next = 0;

static int bcc_err = 0;
static int bcc_on_error_target = -1;
static int bcc_in_handler = 0;
static int bcc_resume_id = -1;
static int bcc_erl = 0;
static const char *bcc_err_file = "";

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
static int bv_i_errbadfilemode = 0;
static int bv_i_errbadfilename = 0;
static int bv_i_errbadfilenumber = 0;
static int bv_i_errbadrecordnumber = 0;
static int bv_i_errdevicefault = 0;
static int bv_i_errdeviceio = 0;
static int bv_i_errdevicetimeout = 0;
static int bv_i_errdeviceunavailable = 0;
static int bv_i_errdiskfull = 0;
static int bv_i_errdiskmediaerror = 0;
static int bv_i_errdisknotready = 0;
static int bv_i_errdiskwriteprotected = 0;
static int bv_i_errdivisionbyzero = 0;
static int bv_i_errduplicatedefinition = 0;
static int bv_i_errfilealreadyexists = 0;
static int bv_i_errfilealreadyopen = 0;
static int bv_i_errfilenotfound = 0;
static int bv_i_errillegalfunctioncall = 0;
static int bv_i_errinputpastend = 0;
static int bv_i_errnoresume = 0;
static int bv_i_erroutofdata = 0;
static int bv_i_erroutofmemory = 0;
static int bv_i_erroutofpaper = 0;
static int bv_i_erroutofstringspace = 0;
static int bv_i_erroverflow = 0;
static int bv_i_errpathfileaccess = 0;
static int bv_i_errpathnotfound = 0;
static int bv_i_errresumewithouterror = 0;
static int bv_i_errreturnwithoutgosub = 0;
static int bv_i_errsubscriptoutofrange = 0;
static int bv_i_errsyntax = 0;
static int bv_i_errtoomanyfiles = 0;
static int bv_i_errtypemismatch = 0;
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
void bf_i_waitanykey(void);
void bf_i_showmainmenu(void);
void bf_i_showbadpartnumber(void);
void bf_i_showrangeretrymessage(void);
void bf_i_shownullentrymessage(const char* bv_s_partstr_in);
void bf_i_showpartstatus(int bv_i_partnum, const char* bv_s_desc_in, int bv_i_qty, int bv_i_reorder, float bv_f_price);
void bf_i_printlistheader(void);
void bf_i_printinventoryline(int bv_i_partnum, const char* bv_s_desc_in, int bv_i_qty, int bv_i_reorder);
void bf_i_printreorderheader(void);
void bf_i_printreorderline(int bv_i_partnum, const char* bv_s_desc_in, int bv_i_qty, int bv_i_reorder);
void bf_i_gatherpartdetails(int bv_i_partnum, char* bv_s_desc_in, int* bv_i_qty_in, int* bv_i_reorder_in, float* bv_f_price_in);
void bf_i_showaddstockscreen(int bv_i_partnum, const char* bv_s_desc_in, int bv_i_qty, int bv_i_reorder);
void bf_i_shownegativeqtywarning(void);
void bf_i_showsubtractstockscreen(int bv_i_partnum, const char* bv_s_desc_in, int bv_i_qty, int bv_i_reorder);
void bf_i_showoversubtractwarning(int bv_i_onhand);
void bf_i_checkpart(void);
void bf_i_editrecord(void);
void bf_i_listall(void);
void bf_i_addstock(void);
void bf_i_subtractstock(void);
void bf_i_reorderreport(void);
void bf_i_initializeinventoryfileifnew(void);
void bf_i_reportinventoryerror(int bv_i_err, int bv_i_erl);

void bf_s_error(int bv_i_code, char* bcc_out) {
    {
        int bt_sel_0 = bv_i_code;
        int bt_sel_match_1 = 0;
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errsyntax)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Syntax error");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errreturnwithoutgosub)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "RETURN without GOSUB");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_erroutofdata)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Out of DATA");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errillegalfunctioncall)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Illegal function call");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_erroverflow)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Overflow");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_erroutofmemory)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Out of memory");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errsubscriptoutofrange)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Subscript out of range");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errduplicatedefinition)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Duplicate Definition");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errdivisionbyzero)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Division by zero");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errtypemismatch)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Type mismatch");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_erroutofstringspace)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Out of string space");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errnoresume)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "No RESUME");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errresumewithouterror)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "RESUME without error");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errdevicetimeout)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Device timeout");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errdevicefault)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Device fault");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_erroutofpaper)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Out of paper");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errbadfilenumber)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Bad file number");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errfilenotfound)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "File not found");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errbadfilemode)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Bad file mode");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errfilealreadyopen)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "File already open");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errdeviceio)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Device I/O error");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errfilealreadyexists)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "File already exists");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errdiskfull)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Disk full");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errinputpastend)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Input past end");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errbadrecordnumber)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Bad record number");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errbadfilename)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Bad file name");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errtoomanyfiles)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Too many files");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errdeviceunavailable)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Device unavailable");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errdiskwriteprotected)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Disk write protected");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errdisknotready)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Disk not ready");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errdiskmediaerror)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Disk media error");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errpathfileaccess)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Path/File access error");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_errpathnotfound)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Path not found");
                return;
            }
        }
        if (!bt_sel_match_1) {
            char bt_s_2[256];
            snprintf(bt_s_2, sizeof(bt_s_2), "%s%s", "Error ", bcc_stri(bv_i_code));
            snprintf(bcc_out, 256, "%s", bt_s_2);
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

void bf_i_waitanykey(void) {
    char bv_s_k[256] = {0};

    printf("\x1b[%d;%dH", 25, 10);
    printf("Press the AnyKey to continue...");
    while (1) {
        snprintf(bv_s_k, sizeof(bv_s_k), "%s", bcc_inkey());
        if ((-(strcmp(bv_s_k, "") != 0))) break;
    }
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
    char bt_s_3[256];
    snprintf(bt_s_3, sizeof(bt_s_3), "%s%s", "3......L)ist all", bcc_stri(bv_i_partcount));
    char bt_s_4[256];
    snprintf(bt_s_4, sizeof(bt_s_4), "%s%s", bt_s_3, "parts");
    printf("\x1b[%dG%s\n", bv_i_tabcol, bt_s_4);
    printf("\x1b[%dG4......A)dd stock\n", bv_i_tabcol);
    printf("\x1b[%dG5......S)ubtract stock\n", bv_i_tabcol);
    printf("\x1b[%dG6......R)eorder Report\n", bv_i_tabcol);
    printf("\n");
    printf("\x1b[%dG7......eX)it to system\n", bv_i_tabcol);
}

void bf_i_showbadpartnumber(void) {
    printf("\x1b[2J\x1b[H");
    printf("\x1b[%d;%dH", 10, 10);
    char bt_s_5[256];
    snprintf(bt_s_5, sizeof(bt_s_5), "%s%s", "Part number is out of permissable range of 1 to", bcc_stri(bv_i_partcount));
    printf("%s\n", bt_s_5);
}

void bf_i_showrangeretrymessage(void) {
    printf("\x1b[%d;%dH", 10, 15);
    char bt_s_6[256];
    snprintf(bt_s_6, sizeof(bt_s_6), "%s%s", "The Part number is out of permissable range of 1 to", bcc_stri(bv_i_partcount));
    printf("%s\n", bt_s_6);
    printf("\x1b[%d;%dH", 25, 15);
    printf("Press the Anykey to reenter part number...");
}

void bf_i_shownullentrymessage(const char* bv_s_partstr_in) {
    char bv_s_partstr[256];
    snprintf(bv_s_partstr, sizeof(bv_s_partstr), "%s", bv_s_partstr_in);

    printf("\x1b[%d;%dH", 10, bv_i_tabcol);
    char bt_s_7[256];
    snprintf(bt_s_7, sizeof(bt_s_7), "%s%s", "Part number ", bv_s_partstr);
    char bt_s_8[256];
    snprintf(bt_s_8, sizeof(bt_s_8), "%s%s", bt_s_7, " is a null entry");
    printf("%s\n", bt_s_8);
}

void bf_i_showpartstatus(int bv_i_partnum, const char* bv_s_desc_in, int bv_i_qty, int bv_i_reorder, float bv_f_price) {
    char bv_s_desc[256];
    snprintf(bv_s_desc, sizeof(bv_s_desc), "%s", bv_s_desc_in);

    printf("\x1b[2J\x1b[H");
    printf("\x1b[%d;%dH", 5, 1);
    printf("\x1b[%dGInventory Status for Individual Part Number\n", bv_i_tabcol);
    printf("\x1b[%dG===========================================\n", bv_i_tabcol);
    printf("\n");
    printf("\n");
    char bt_s_9[256];
    snprintf(bt_s_9, sizeof(bt_s_9), "%s%s", "     Part number:  ", bcc_stri(bv_i_partnum));
    printf("\x1b[%dG%s\n", bv_i_tabcol, bt_s_9);
    printf("\n");
    char bt_s_10[256];
    snprintf(bt_s_10, sizeof(bt_s_10), "%s%s", "       Item name:  ", bv_s_desc);
    printf("\x1b[%dG%s\n", bv_i_tabcol, bt_s_10);
    char bt_s_11[256];
    snprintf(bt_s_11, sizeof(bt_s_11), "%s%s", "Quantity on hand:  ", bcc_stri(bv_i_qty));
    printf("\x1b[%dG%s\n", bv_i_tabcol, bt_s_11);
    char bt_s_12[256];
    snprintf(bt_s_12, sizeof(bt_s_12), "%s%s", "   Reorder level:  ", bcc_stri(bv_i_reorder));
    printf("\x1b[%dG%s\n", bv_i_tabcol, bt_s_12);
    char bt_s_13[256];
    snprintf(bt_s_13, sizeof(bt_s_13), "%s%s", "      Unit price:  ", bcc_strd(bv_f_price));
    printf("\x1b[%dG%s\n", bv_i_tabcol, bt_s_13);
}

void bf_i_printlistheader(void) {
    printf("\x1b[2J\x1b[H");
    char bt_s_14[256];
    snprintf(bt_s_14, sizeof(bt_s_14), "%s%s", bcc_stri(bv_i_partcount), "items");
    printf("\x1b[%dGI N V E N T O R Y   L I S T I N G\x1b[%dG%s\n", 25, 65, bt_s_14);
    printf("                                          Quantity       Reorder\n");
    printf(" Partno           Description             on hand         level\n");
    printf("\x1b[%d;%dH", 25, 1);
    printf("Press the AnyKey to scroll listing...");
}

void bf_i_printinventoryline(int bv_i_partnum, const char* bv_s_desc_in, int bv_i_qty, int bv_i_reorder) {
    char bv_s_desc[256];
    snprintf(bv_s_desc, sizeof(bv_s_desc), "%s", bv_s_desc_in);

    char bt_s_15[256];
    snprintf(bt_s_15, sizeof(bt_s_15), "%s%s", bcc_stri(bv_i_partnum), "  ");
    char bt_s_16[256];
    snprintf(bt_s_16, sizeof(bt_s_16), "%s%s", bt_s_15, bv_s_desc);
    char bt_s_17[256];
    snprintf(bt_s_17, sizeof(bt_s_17), "%s%s", bt_s_16, "   ");
    char bt_s_18[256];
    snprintf(bt_s_18, sizeof(bt_s_18), "%s%s", bt_s_17, bcc_stri(bv_i_qty));
    char bt_s_19[256];
    snprintf(bt_s_19, sizeof(bt_s_19), "%s%s", bt_s_18, "          ");
    char bt_s_20[256];
    snprintf(bt_s_20, sizeof(bt_s_20), "%s%s", bt_s_19, bcc_stri(bv_i_reorder));
    printf("%s\n", bt_s_20);
}

void bf_i_printreorderheader(void) {
    char bv_s_date[256] = {0};

    printf("\x1b[2J\x1b[H");
    printf("\x1b[%d;%dH", 1, bv_i_tabcol);
    printf("Reorder Report\x1b[%dG%s\n", 55, bv_s_date);
    printf("\n");
    printf("                                             Quantity       Reorder\n");
    printf("    Partno           Description             on hand         level\n");
    printf("   =======  ==============================   ========       =======\n");
}

void bf_i_printreorderline(int bv_i_partnum, const char* bv_s_desc_in, int bv_i_qty, int bv_i_reorder) {
    char bv_s_desc[256];
    snprintf(bv_s_desc, sizeof(bv_s_desc), "%s", bv_s_desc_in);

    char bt_s_21[256];
    snprintf(bt_s_21, sizeof(bt_s_21), "%s%s", "  ", bcc_stri(bv_i_partnum));
    char bt_s_22[256];
    snprintf(bt_s_22, sizeof(bt_s_22), "%s%s", bt_s_21, "  ");
    char bt_s_23[256];
    snprintf(bt_s_23, sizeof(bt_s_23), "%s%s", bt_s_22, bv_s_desc);
    char bt_s_24[256];
    snprintf(bt_s_24, sizeof(bt_s_24), "%s%s", bt_s_23, "   ");
    char bt_s_25[256];
    snprintf(bt_s_25, sizeof(bt_s_25), "%s%s", bt_s_24, bcc_stri(bv_i_qty));
    char bt_s_26[256];
    snprintf(bt_s_26, sizeof(bt_s_26), "%s%s", bt_s_25, "          ");
    char bt_s_27[256];
    snprintf(bt_s_27, sizeof(bt_s_27), "%s%s", bt_s_26, bcc_stri(bv_i_reorder));
    printf("%s\n", bt_s_27);
}

void bf_i_gatherpartdetails(int bv_i_partnum, char* bv_s_desc_in, int* bv_i_qty_in, int* bv_i_reorder_in, float* bv_f_price_in) {
    char bv_s_desc[256];
    snprintf(bv_s_desc, sizeof(bv_s_desc), "%s", bv_s_desc_in);
    int bv_i_qty = *bv_i_qty_in;
    int bv_i_reorder = *bv_i_reorder_in;
    float bv_f_price = *bv_f_price_in;

    printf("\x1b[2J\x1b[H");
    printf("\x1b[%d;%dH", 4, bv_i_tabcol);
    printf("Adding or Overwriting a Record\n");
    printf("\x1b[%d;%dH", 8, bv_i_tabcol);
    char bt_s_28[256];
    snprintf(bt_s_28, sizeof(bt_s_28), "%s%s", "Record/Partno", bcc_stri(bv_i_partnum));
    printf("%s\n", bt_s_28);
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
}

void bf_i_showaddstockscreen(int bv_i_partnum, const char* bv_s_desc_in, int bv_i_qty, int bv_i_reorder) {
    char bv_s_desc[256];
    snprintf(bv_s_desc, sizeof(bv_s_desc), "%s", bv_s_desc_in);

    printf("\x1b[2J\x1b[H");
    printf("\x1b[%d;%dH", 4, 25);
    printf("Add to an inventory part number\n");
    printf("\x1b[%d;%dH", 5, 25);
    printf("===============================\n");
    printf("\x1b[%d;%dH", 8, bv_i_tabcol);
    char bt_s_29[256];
    snprintf(bt_s_29, sizeof(bt_s_29), "%s%s", "     Part number: ", bcc_stri(bv_i_partnum));
    printf("%s\n", bt_s_29);
    printf("\x1b[%d;%dH", 9, bv_i_tabcol);
    char bt_s_30[256];
    snprintf(bt_s_30, sizeof(bt_s_30), "%s%s", "Item description: ", bv_s_desc);
    printf("%s\n", bt_s_30);
    printf("\x1b[%d;%dH", 10, bv_i_tabcol);
    char bt_s_31[256];
    snprintf(bt_s_31, sizeof(bt_s_31), "%s%s", "Quantity on hand: ", bcc_stri(bv_i_qty));
    printf("%s\n", bt_s_31);
    printf("\x1b[%d;%dH", 11, bv_i_tabcol);
    char bt_s_32[256];
    snprintf(bt_s_32, sizeof(bt_s_32), "%s%s", "   Reorder Level: ", bcc_stri(bv_i_reorder));
    printf("%s\n", bt_s_32);
}

void bf_i_shownegativeqtywarning(void) {
    printf("\x1b[%d;%dH", 17, 15);
    printf("The quantity to add must NOT be a negative number\n");
    printf("\x1b[%d;%dH", 25, 1);
    printf("Please press the Anykey to reenter quantity to add...");
}

void bf_i_showsubtractstockscreen(int bv_i_partnum, const char* bv_s_desc_in, int bv_i_qty, int bv_i_reorder) {
    char bv_s_desc[256];
    snprintf(bv_s_desc, sizeof(bv_s_desc), "%s", bv_s_desc_in);

    printf("\x1b[2J\x1b[H");
    printf("\x1b[%d;%dH", 4, bv_i_tabcol);
    printf("Subtract an inventory part number\n");
    printf("\x1b[%d;%dH", 5, bv_i_tabcol);
    printf("=================================\n");
    printf("\x1b[%d;%dH", 8, bv_i_tabcol);
    char bt_s_33[256];
    snprintf(bt_s_33, sizeof(bt_s_33), "%s%s", "         Part number: ", bcc_stri(bv_i_partnum));
    printf("%s\n", bt_s_33);
    printf("\x1b[%d;%dH", 9, bv_i_tabcol);
    char bt_s_34[256];
    snprintf(bt_s_34, sizeof(bt_s_34), "%s%s", "    Item description: ", bv_s_desc);
    printf("%s\n", bt_s_34);
    printf("\x1b[%d;%dH", 10, bv_i_tabcol);
    char bt_s_35[256];
    snprintf(bt_s_35, sizeof(bt_s_35), "%s%s", "    Quantity on hand: ", bcc_stri(bv_i_qty));
    printf("%s\n", bt_s_35);
    printf("\x1b[%d;%dH", 11, bv_i_tabcol);
    char bt_s_36[256];
    snprintf(bt_s_36, sizeof(bt_s_36), "%s%s", "       Reorder Level: ", bcc_stri(bv_i_reorder));
    printf("%s\n", bt_s_36);
}

void bf_i_showoversubtractwarning(int bv_i_onhand) {
    printf("\x1b[%d;%dH", 17, 5);
    printf("The quantity to SUBTRACT must NOT result in NEGATIVE inventory\n");
    printf("\x1b[%d;%dH", 18, 5);
    char bt_s_37[256];
    snprintf(bt_s_37, sizeof(bt_s_37), "%s%s", "Only", bcc_stri(bv_i_onhand));
    char bt_s_38[256];
    snprintf(bt_s_38, sizeof(bt_s_38), "%s%s", bt_s_37, " IN STOCK");
    printf("%s\n", bt_s_38);
    printf("\x1b[%d;%dH", 25, 1);
    printf("Please press the Anykey to reenter quantity to subtract...");
}

void bf_i_checkpart(void) {
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

    // global inv
    char bt_s_39[256];
    bf_s_readpartnumberinput(bt_s_39);
    snprintf(bv_s_partstr, sizeof(bv_s_partstr), "%s", bt_s_39);
    bv_i_part = ((int)round((double)(atof(bv_s_partstr))));
    if ((-(bf_i_partinrange(bv_i_part) == 0))) {
        bf_i_showbadpartnumber();
        bf_i_waitanykey();
        return;
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
        char bt_s_40[256];
        snprintf(bt_s_40, sizeof(bt_s_40), "%s%s", "Part number", bcc_stri(bv_i_part));
        char bt_s_41[256];
        snprintf(bt_s_41, sizeof(bt_s_41), "%s%s", bt_s_40, "is still a null entry at this time");
        printf("%s\n", bt_s_41);
        bf_i_waitanykey();
        return;
    }
    bf_i_showpartstatus(bv_i_part, bv_s_pdesc, bv_i_pqty, bv_i_preorder, bv_f_pprice);
    bf_i_waitanykey();
}

void bf_i_editrecord(void) {
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

    // global inv
    printf("\x1b[2J\x1b[H");
    printf("\x1b[%d;%dH", 10, bv_i_tabcol);
    char bt_s_42[256];
    bf_s_readpartnumberinput(bt_s_42);
    snprintf(bv_s_partstr, sizeof(bv_s_partstr), "%s", bt_s_42);
    bv_i_part = ((int)round((double)(atof(bv_s_partstr))));
    if ((-(bf_i_partinrange(bv_i_part) == 0))) {
        bf_i_showbadpartnumber();
        bf_i_waitanykey();
        return;
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
        char bt_s_43[256];
        bf_s_readkey(bt_s_43);
        snprintf(bv_s_kp, sizeof(bv_s_kp), "%s", bt_s_43);
        if (((-(strcmp(bv_s_kp, "Y") != 0)) && (-(strcmp(bv_s_kp, "y") != 0)))) {
            return;
        }
    }

    while (1) {
        bf_i_gatherpartdetails(bv_i_part, bv_s_editdesc, &bv_i_editqty, &bv_i_editreorder, &bv_f_editprice);
        char bt_s_44[256];
        bf_s_readkey(bt_s_44);
        snprintf(bv_s_kp, sizeof(bv_s_kp), "%s", bt_s_44);
        if (((-(strcmp(bv_s_kp, "Y") == 0)) || (-(strcmp(bv_s_kp, "y") == 0)))) break;
    }
    // inv[...] = { ... }  (whole-record write)
    int16_t bcc_tmp_45 = bv_i_editqty;
    int16_t bcc_tmp_46 = bv_i_editreorder;
    float bcc_tmp_47 = bv_f_editprice;
    bcc_put_record_part(bcc_files[0], bv_i_part, "1", bv_s_editdesc, &bcc_tmp_45, &bcc_tmp_46, &bcc_tmp_47);
}

void bf_i_listall(void) {
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

    // global inv
    bf_i_printlistheader();
    bv_i_scrollcount = 0;
    int bt_lim_48 = bv_i_partcount;
    int bt_step_48 = 1;
    for (bv_i_i = 1; bt_step_48 >= 0 ? bv_i_i <= bt_lim_48 : bv_i_i >= bt_lim_48; bv_i_i += bt_step_48) {
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
        bf_i_printinventoryline(bv_i_i, bv_s_pdesc, bv_i_pqty, bv_i_preorder);
        bv_i_scrollcount = (bv_i_scrollcount + 1);
        if ((-(bv_i_scrollcount == 20))) {
            bf_i_waitanykey();
            bv_i_scrollcount = 0;
        }
    }
}

void bf_i_addstock(void) {
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

    // global inv
    printf("\x1b[2J\x1b[H");
    printf("\x1b[%d;%dH", 5, 25);
    printf("A D D I N G   S T O C K\n");

    while (1) {
        printf("\x1b[%d;%dH", 8, 25);
        char bt_s_49[256];
        bf_s_readpartnumberinput(bt_s_49);
        snprintf(bv_s_partstr, sizeof(bv_s_partstr), "%s", bt_s_49);
        bv_i_part = ((int)round((double)(atof(bv_s_partstr))));
        bv_i_validpart = bf_i_partinrange(bv_i_part);
        if ((-(bv_i_validpart == 0))) {
            bf_i_showrangeretrymessage();
            char bt_s_50[256];
            bf_s_readkey(bt_s_50);
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
        bf_i_shownullentrymessage(bv_s_partstr);
        char bt_s_51[256];
        bf_s_readkey(bt_s_51);
        return;
    }

    while (1) {
        bf_i_showaddstockscreen(bv_i_part, bv_s_pdesc, bv_i_pqty, bv_i_preorder);
        printf("\x1b[%d;%dH", 14, bv_i_tabcol);
        printf(" Quantity to add? ");
        bcc_read_line();
        snprintf(bv_s_addstr, sizeof(bv_s_addstr), "%s", bcc_input_buf);
        bv_i_addamt = ((int)round((double)(atof(bv_s_addstr))));
        if ((-(bv_i_addamt < 0))) {
            bf_i_shownegativeqtywarning();
            char bt_s_52[256];
            bf_s_readkey(bt_s_52);
        }
        if ((-(bv_i_addamt >= 0))) break;
    }

    bv_i_pqty = (bv_i_pqty + bv_i_addamt);
    // inv[...] = p  (write back a let-bound record)
    int16_t bcc_tmp_53 = bv_i_pqty;
    int16_t bcc_tmp_54 = bv_i_preorder;
    float bcc_tmp_55 = bv_f_pprice;
    bcc_put_record_part(bcc_files[0], bv_i_part, bv_s_pflag, bv_s_pdesc, &bcc_tmp_53, &bcc_tmp_54, &bcc_tmp_55);
}

void bf_i_subtractstock(void) {
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

    // global inv
    printf("\x1b[2J\x1b[H");
    printf("\x1b[%d;%dH", 5, 20);
    printf("S U B T R A C T I N G    S T O C K\n");

    while (1) {
        printf("\x1b[%d;%dH", 8, 25);
        char bt_s_56[256];
        bf_s_readpartnumberinput(bt_s_56);
        snprintf(bv_s_partstr, sizeof(bv_s_partstr), "%s", bt_s_56);
        bv_i_part = ((int)round((double)(atof(bv_s_partstr))));
        bv_i_validpart = bf_i_partinrange(bv_i_part);
        if ((-(bv_i_validpart == 0))) {
            bf_i_showrangeretrymessage();
            char bt_s_57[256];
            bf_s_readkey(bt_s_57);
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
        bf_i_shownullentrymessage(bv_s_partstr);
        char bt_s_58[256];
        bf_s_readkey(bt_s_58);
        return;
    }

    while (1) {
        bf_i_showsubtractstockscreen(bv_i_part, bv_s_pdesc, bv_i_pqty, bv_i_preorder);
        printf("\x1b[%d;%dH", 14, bv_i_tabcol);
        printf("Quantity to subtract? ");
        bcc_read_line();
        snprintf(bv_s_substr, sizeof(bv_s_substr), "%s", bcc_input_buf);
        bv_i_subamt = ((int)round((double)(atof(bv_s_substr))));
        bv_i_oversubtract = 0;
        if (((-(bv_i_subamt >= 0)) && (-((bv_i_pqty - bv_i_subamt) < 0)))) {
            bv_i_oversubtract = 1;
            bf_i_showoversubtractwarning(bv_i_pqty);
            char bt_s_59[256];
            bf_s_readkey(bt_s_59);
        }
        if (((-(bv_i_subamt >= 0)) && (-(bv_i_oversubtract == 0)))) break;
    }

    bv_i_pqty = (bv_i_pqty - bv_i_subamt);
    if ((-(bv_i_pqty <= bv_i_preorder))) {
        printf("\x1b[%d;%dH", 16, bv_i_tabcol);
    }
    char bt_s_60[256];
    snprintf(bt_s_60, sizeof(bt_s_60), "%s%s", "quantity now", bcc_stri(bv_i_pqty));
    char bt_s_61[256];
    snprintf(bt_s_61, sizeof(bt_s_61), "%s%s", bt_s_60, " reorder level");
    char bt_s_62[256];
    snprintf(bt_s_62, sizeof(bt_s_62), "%s%s", bt_s_61, bcc_stri(bv_i_preorder));
    printf("%s\n", bt_s_62);
    // inv[...] = p  (write back a let-bound record)
    int16_t bcc_tmp_63 = bv_i_pqty;
    int16_t bcc_tmp_64 = bv_i_preorder;
    float bcc_tmp_65 = bv_f_pprice;
    bcc_put_record_part(bcc_files[0], bv_i_part, bv_s_pflag, bv_s_pdesc, &bcc_tmp_63, &bcc_tmp_64, &bcc_tmp_65);
}

void bf_i_reorderreport(void) {
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

    // global inv
    bf_i_printreorderheader();
    bv_i_reportlinecount = 0;
    int bt_lim_66 = bv_i_partcount;
    int bt_step_66 = 1;
    for (bv_i_i = 1; bt_step_66 >= 0 ? bv_i_i <= bt_lim_66 : bv_i_i >= bt_lim_66; bv_i_i += bt_step_66) {
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
            bf_i_printreorderline(bv_i_i, bv_s_pdesc, bv_i_pqty, bv_i_preorder);
            bv_i_reportlinecount = (bv_i_reportlinecount + 1);
            if ((-(bv_i_reportlinecount > 15))) {
                bf_i_waitanykey();
                bv_i_reportlinecount = 0;
            }
        }
    }
    bf_i_waitanykey();
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

    // global inv
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
        int bt_lim_67 = bv_i_partcount;
        int bt_step_67 = 1;
        for (bv_i_i = 1; bt_step_67 >= 0 ? bv_i_i <= bt_lim_67 : bv_i_i >= bt_lim_67; bv_i_i += bt_step_67) {
            // inv[...] = { ... }  (whole-record write)
            int16_t bcc_tmp_68 = 0;
            int16_t bcc_tmp_69 = 0;
            float bcc_tmp_70 = 0;
            bcc_put_record_part(bcc_files[0], bv_i_i, bcc_chr(255), "", &bcc_tmp_68, &bcc_tmp_69, &bcc_tmp_70);
        }
    }
}

void bf_i_reportinventoryerror(int bv_i_err, int bv_i_erl) {
    char bv_s_k[256] = {0};

    printf("\x1b[%d;%dH", 25, 1);
    char bt_s_71[256];
    snprintf(bt_s_71, sizeof(bt_s_71), "%s%s", "There has been an error on line", bcc_stri(bv_i_erl));
    char bt_s_72[256];
    snprintf(bt_s_72, sizeof(bt_s_72), "%s%s", bt_s_71, ": ");
    char bt_s_73[256];
    bf_s_error(bv_i_err, bt_s_73);
    char bt_s_74[256];
    snprintf(bt_s_74, sizeof(bt_s_74), "%s%s", bt_s_72, bt_s_73);
    printf("%s\n", bt_s_74);
    char bt_s_75[256];
    bf_s_readkey(bt_s_75);
    snprintf(bv_s_k, sizeof(bv_s_k), "%s", bt_s_75);
}

int main(void) {
    setvbuf(stdin, NULL, _IONBF, 0);
    // Maps an ERR code to its classic MBASIC/GW-BASIC/BASCOM message. Compiles
    // and links on a real IBM BASIC Compiler 2.00 as ERROR$, but silently
    // returns an empty string at runtime (verified under dosbox-x) -- so BASCAL
    // ships a working implementation.
    //
    // The named constants below are the complete common subset supported by
    // ERROR$: use them in THROW and filtered CATCH clauses instead of magic
    // numbers.  Dialect-specific errors outside this shared MBASIC/GW-BASIC/
    // BASCOM subset still fall through to ERROR$'s generic message.
    //
    // Deliberately NOT a scalar method (see GitHub issue #41, which asked for
    // this decision to be recorded either way): code% is an opaque lookup key,
    // not a value the call is naturally "operating on" the way ltrim$/rtrim$/
    // ucase$/lcase$ operate on their string -- code%.error() would read as if
    // the *error code itself* has a message, when really this is a lookup
    // table keyed by that code. Stays an ordinary function.

    bv_i_errsyntax = 2;
    bv_i_errreturnwithoutgosub = 3;
    bv_i_erroutofdata = 4;
    bv_i_errillegalfunctioncall = 5;
    bv_i_erroverflow = 6;
    bv_i_erroutofmemory = 7;
    bv_i_errsubscriptoutofrange = 9;
    bv_i_errduplicatedefinition = 10;
    bv_i_errdivisionbyzero = 11;
    bv_i_errtypemismatch = 13;
    bv_i_erroutofstringspace = 14;
    bv_i_errnoresume = 19;
    bv_i_errresumewithouterror = 20;
    bv_i_errdevicetimeout = 24;
    bv_i_errdevicefault = 25;
    bv_i_erroutofpaper = 27;
    bv_i_errbadfilenumber = 52;
    bv_i_errfilenotfound = 53;
    bv_i_errbadfilemode = 54;
    bv_i_errfilealreadyopen = 55;
    bv_i_errdeviceio = 57;
    bv_i_errfilealreadyexists = 58;
    bv_i_errdiskfull = 61;
    bv_i_errinputpastend = 62;
    bv_i_errbadrecordnumber = 63;
    bv_i_errbadfilename = 64;
    bv_i_errtoomanyfiles = 67;
    bv_i_errdeviceunavailable = 68;
    bv_i_errdiskwriteprotected = 70;
    bv_i_errdisknotready = 71;
    bv_i_errdiskmediaerror = 72;
    bv_i_errpathfileaccess = 75;
    bv_i_errpathnotfound = 76;

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
    // hand at his line 550. Wrapped in its own try/catch: a file that
    // exists but can't be opened for random access (permissions, a
    // read-only inven.dat, disk full on the fallback create) is a real,
    // trappable error (code 75, "Path/File access error") on both
    // targets now, not a hard crash -- report it and exit cleanly
    // instead of leaving the program to fail confusingly the first time
    // something tries to use an `inv` that was never actually opened.
    int bcc_try_0_pending = 0;
    bcc_on_error_target = 0;
    // file inv as Part = open(...)  [39 bytes/record]
    bcc_files[0] = fopen("inven.dat", "rb+");
    if (!bcc_files[0]) bcc_files[0] = fopen("inven.dat", "wb+");
    if (!bcc_files[0]) {
        bcc_err = 75;
        bcc_erl = 91;
        bcc_err_file = "tutorial/inventory.bcl";
        goto bcc_try_0_catch;
    }
    bcc_on_error_target = -1;
    goto bcc_try_0_finally;
    bcc_try_0_catch: ;
    bcc_in_handler = 0;
    bcc_on_error_target = -1;
    bv_i_err = bcc_err;
    bv_i_erl = bcc_erl;
    char bt_s_76[256];
    bf_s_error(bv_i_err, bt_s_76);
    char bt_s_77[256];
    snprintf(bt_s_77, sizeof(bt_s_77), "%s%s", "could not open inven.dat: ", bt_s_76);
    printf("%s\n", bt_s_77);
    return 0;
    bcc_on_error_target = -1;
    goto bcc_try_0_finally;
    bcc_try_0_rethrow: ;
    bcc_try_0_pending = 1;
    bcc_try_0_finally: ;
    if (bcc_try_0_pending) {
        fprintf(stderr, "unhandled BASIC error %d\n", bcc_err);
        exit(1);
    }
    bcc_try_0_end: ;

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
        char bt_s_78[256];
        bf_s_readkey(bt_s_78);
        snprintf(bv_s_kp, sizeof(bv_s_kp), "%s", bt_s_78);
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
            int bcc_try_1_pending = 0;
            bcc_on_error_target = 1;
            {
                char bt_sel_79[256];
                snprintf(bt_sel_79, sizeof(bt_sel_79), "%s", bv_s_kp);
                int bt_sel_match_80 = 0;
                if (!bt_sel_match_80) {
                    if ((strcmp(bt_sel_79, "1") == 0) || (strcmp(bt_sel_79, "c") == 0) || (strcmp(bt_sel_79, "C") == 0)) {
                        bt_sel_match_80 = 1;
                        bf_i_checkpart();
                    }
                }
                if (!bt_sel_match_80) {
                    if ((strcmp(bt_sel_79, "2") == 0) || (strcmp(bt_sel_79, "e") == 0) || (strcmp(bt_sel_79, "E") == 0)) {
                        bt_sel_match_80 = 1;
                        bf_i_editrecord();
                    }
                }
                if (!bt_sel_match_80) {
                    if ((strcmp(bt_sel_79, "3") == 0) || (strcmp(bt_sel_79, "l") == 0) || (strcmp(bt_sel_79, "L") == 0)) {
                        bt_sel_match_80 = 1;
                        bf_i_listall();
                    }
                }
                if (!bt_sel_match_80) {
                    if ((strcmp(bt_sel_79, "4") == 0) || (strcmp(bt_sel_79, "a") == 0) || (strcmp(bt_sel_79, "A") == 0)) {
                        bt_sel_match_80 = 1;
                        bf_i_addstock();
                    }
                }
                if (!bt_sel_match_80) {
                    if ((strcmp(bt_sel_79, "5") == 0) || (strcmp(bt_sel_79, "s") == 0) || (strcmp(bt_sel_79, "S") == 0)) {
                        bt_sel_match_80 = 1;
                        bf_i_subtractstock();
                    }
                }
                if (!bt_sel_match_80) {
                    if ((strcmp(bt_sel_79, "6") == 0) || (strcmp(bt_sel_79, "r") == 0) || (strcmp(bt_sel_79, "R") == 0)) {
                        bt_sel_match_80 = 1;
                        bf_i_reorderreport();
                    }
                }
                if (!bt_sel_match_80) {
                    if ((strcmp(bt_sel_79, "7") == 0) || (strcmp(bt_sel_79, "x") == 0) || (strcmp(bt_sel_79, "X") == 0)) {
                        bt_sel_match_80 = 1;
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
                        bcc_color(7, 0);
                        printf("\x1b[2J\x1b[H");
                        exit(0);
                    }
                }
            }
            bcc_on_error_target = -1;
            goto bcc_try_1_finally;
            bcc_try_1_catch: ;
            bcc_in_handler = 0;
            bcc_on_error_target = -1;
            bv_i_err = bcc_err;
            bv_i_erl = bcc_erl;
            bf_i_reportinventoryerror(bv_i_err, bv_i_erl);
            bcc_on_error_target = -1;
            goto bcc_try_1_finally;
            bcc_try_1_rethrow: ;
            bcc_try_1_pending = 1;
            bcc_try_1_finally: ;
            if (bcc_try_1_pending) {
                fprintf(stderr, "unhandled BASIC error %d\n", bcc_err);
                exit(1);
            }
            bcc_try_1_end: ;
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
    static char buf[2];
#if defined(_WIN32)
    if (_kbhit()) {
        buf[0] = (char)_getch();
        buf[1] = 0;
    } else {
        buf[0] = 0;
    }
#else
    struct termios orig, raw;
    tcgetattr(STDIN_FILENO, &orig);
    raw = orig;
    raw.c_lflag &= ~(ICANON | ECHO);
    raw.c_cc[VMIN] = 0;
    raw.c_cc[VTIME] = 0;
    tcsetattr(STDIN_FILENO, TCSANOW, &raw);

    unsigned char c;
    ssize_t n = read(STDIN_FILENO, &c, 1);
    if (n == 1) {
        buf[0] = (char)c;
        buf[1] = 0;
    } else {
        buf[0] = 0;
    }

    tcsetattr(STDIN_FILENO, TCSANOW, &orig);
#endif
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

