[Home](../../) / [Tutorials](../) / Portable Structured Error Handling

<div class="prose" markdown="1">

Generated sources: [BCL](https://github.com/johnjoeallen/bascal/blob/main/tutorial/portable_error_handling.bcl), [BASIC](https://github.com/johnjoeallen/bascal/blob/main/tutorial/portable_error_handling.bas), [C](https://github.com/johnjoeallen/bascal/blob/main/tutorial/portable_error_handling.c), and [JVM assembly](https://github.com/johnjoeallen/bascal/blob/main/tutorial/portable_error_handling.j).

`try`/`catch`/`finally` is the portable alternative to classic `ON ERROR GOTO`/`RESUME`. A parenthesized catch filter can list multiple error codes, and the optional `source$` binding identifies the source file where the error originated. An unmatched error is re-raised after `finally`.

</div>

```bascal
require com.bascal.stdlib.error

try
    throw errFileNotFound%
catch err%(errFileNotFound%, errFileAlreadyOpen%), erl%, source$
    print "caught error "; err%; " at "; source$; ":"; erl%
finally
    print "cleanup always runs"
end try
```

<!-- BEGIN generated tutorial source -->

<details class="source-embed" markdown="1">

<summary><code>tutorial/portable_error_handling.bcl</code></summary>



```bascal

// Tutorial — Portable Structured Error Handling
//
// TRY/CATCH/FINALLY and THROW are BASCAL's portable error model.  A catch can
// select several error codes and bind the originating source file.
program portableErrorHandling
require com.bascal.stdlib.error

print "portable try/catch:"
try
    throw ERR_FILE_NOT_FOUND
catch err%(ERR_FILE_NOT_FOUND, ERR_FILE_ALREADY_OPEN), erl%, source$
    print "  caught error "; err%; " at "; source$; ":"; erl%
finally
    print "  cleanup always runs"
end try

end

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/portable_error_handling.bas</code></summary>



```basic

10 ' BASCAL generated BASIC -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Maps an ERR code to its classic MBASIC/GW-BASIC/BASCOM message. Compiles
40 ' and links on a real IBM BASIC Compiler 2.00 as ERROR$, but silently
50 ' returns an empty string at runtime (verified under dosbox-x) -- so BASCAL
60 ' ships a working implementation.
70 '
80 ' The named constants below are the complete common subset supported by
90 ' ERROR$: use them in THROW and filtered CATCH clauses instead of magic
100 ' numbers.  Dialect-specific errors outside this shared MBASIC/GW-BASIC/
110 ' BASCOM subset still fall through to ERROR$'s generic message.
120 '
130 ' Deliberately NOT a scalar method (see GitHub issue #41, which asked for
140 ' this decision to be recorded either way): code% is an opaque lookup key,
150 ' not a value the call is naturally "operating on" the way ltrim$/rtrim$/
160 ' ucase$/lcase$ operate on their string -- code%.error() would read as if
170 ' the *error code itself* has a message, when really this is a lookup
180 ' table keyed by that code. Stays an ordinary function.
190 CONSTERRSYNTAX% = 2
200 CONSTERRRETURNWITHOUTGOSUB% = 3
210 CONSTERROUTOFDATA% = 4
220 CONSTERRILLEGALFUNCTIONCALL% = 5
230 CONSTERROVERFLOW% = 6
240 CONSTERROUTOFMEMORY% = 7
250 CONSTERRSUBSCRIPTOUTOFRANGE% = 9
260 CONSTERRDUPLICATEDEFINITION% = 10
270 CONSTERRDIVISIONBYZERO% = 11
280 CONSTERRTYPEMISMATCH% = 13
290 CONSTERROUTOFSTRINGSPACE% = 14
300 CONSTERRNORESUME% = 19
310 CONSTERRRESUMEWITHOUTERROR% = 20
320 CONSTERRDEVICETIMEOUT% = 24
330 CONSTERRDEVICEFAULT% = 25
340 CONSTERROUTOFPAPER% = 27
350 CONSTERRBADFILENUMBER% = 52
360 CONSTERRFILENOTFOUND% = 53
370 CONSTERRBADFILEMODE% = 54
380 CONSTERRFILEALREADYOPEN% = 55
390 CONSTERRDEVICEIO% = 57
400 CONSTERRFILEALREADYEXISTS% = 58
410 CONSTERRDISKFULL% = 61
420 CONSTERRINPUTPASTEND% = 62
430 CONSTERRBADRECORDNUMBER% = 63
440 CONSTERRBADFILENAME% = 64
450 CONSTERRTOOMANYFILES% = 67
460 CONSTERRDEVICEUNAVAILABLE% = 68
470 CONSTERRDISKWRITEPROTECTED% = 70
480 CONSTERRDISKNOTREADY% = 71
490 CONSTERRDISKMEDIAERROR% = 72
500 CONSTERRPATHFILEACCESS% = 75
510 CONSTERRPATHNOTFOUND% = 76
520 ' Tutorial — Portable Structured Error Handling
530 '
540 ' TRY/CATCH/FINALLY and THROW are BASCAL's portable error model.  A catch can
550 ' select several error codes and bind the originating source file.
560 PRINT "portable try/catch:"
570 ON ERROR GOTO 620
580 BCCTRY0001PENDING% = 0
590 ERROR CONSTERRFILENOTFOUND%
600 ON ERROR GOTO 0
610 GOTO 770
620 BCCTRY0001PENDING% = ERR
630 IF (ERR = CONSTERRFILENOTFOUND%) OR (ERR = CONSTERRFILEALREADYOPEN%) THEN GOTO 650
640 RESUME 770
650 BCCERR% = ERR
660 BCCERL% = ERL
670 GOSUB 2250
680 source$ = BCCSOURCEFILE$
690 RESUME 700
700 ON ERROR GOTO 750
710 PRINT "  caught error "; BCCERR%; " at "; source$; ":"; BCCERL%
720 BCCTRY0001PENDING% = 0
730 ON ERROR GOTO 0
740 GOTO 770
750 BCCTRY0001PENDING% = ERR
760 RESUME 770
770 ON ERROR GOTO 0
780 PRINT "  cleanup always runs"
790 IF BCCTRY0001PENDING% <> 0 THEN ERROR BCCTRY0001PENDING%
800 REM END TRY
810 END

820 ' function error$(code%)
830     BCCT3% = errorCode0%
840     IF (BCCT3% = CONSTERRSYNTAX%) <> 0 THEN GOTO 1180
850     IF (BCCT3% = CONSTERRRETURNWITHOUTGOSUB%) <> 0 THEN GOTO 1210
860     IF (BCCT3% = CONSTERROUTOFDATA%) <> 0 THEN GOTO 1240
870     IF (BCCT3% = CONSTERRILLEGALFUNCTIONCALL%) <> 0 THEN GOTO 1270
880     IF (BCCT3% = CONSTERROVERFLOW%) <> 0 THEN GOTO 1300
890     IF (BCCT3% = CONSTERROUTOFMEMORY%) <> 0 THEN GOTO 1330
900     IF (BCCT3% = CONSTERRSUBSCRIPTOUTOFRANGE%) <> 0 THEN GOTO 1360
910     IF (BCCT3% = CONSTERRDUPLICATEDEFINITION%) <> 0 THEN GOTO 1390
920     IF (BCCT3% = CONSTERRDIVISIONBYZERO%) <> 0 THEN GOTO 1420
930     IF (BCCT3% = CONSTERRTYPEMISMATCH%) <> 0 THEN GOTO 1450
940     IF (BCCT3% = CONSTERROUTOFSTRINGSPACE%) <> 0 THEN GOTO 1480
950     IF (BCCT3% = CONSTERRNORESUME%) <> 0 THEN GOTO 1510
960     IF (BCCT3% = CONSTERRRESUMEWITHOUTERROR%) <> 0 THEN GOTO 1540
970     IF (BCCT3% = CONSTERRDEVICETIMEOUT%) <> 0 THEN GOTO 1570
980     IF (BCCT3% = CONSTERRDEVICEFAULT%) <> 0 THEN GOTO 1600
990     IF (BCCT3% = CONSTERROUTOFPAPER%) <> 0 THEN GOTO 1630
1000     IF (BCCT3% = CONSTERRBADFILENUMBER%) <> 0 THEN GOTO 1660
1010     IF (BCCT3% = CONSTERRFILENOTFOUND%) <> 0 THEN GOTO 1690
1020     IF (BCCT3% = CONSTERRBADFILEMODE%) <> 0 THEN GOTO 1720
1030     IF (BCCT3% = CONSTERRFILEALREADYOPEN%) <> 0 THEN GOTO 1750
1040     IF (BCCT3% = CONSTERRDEVICEIO%) <> 0 THEN GOTO 1780
1050     IF (BCCT3% = CONSTERRFILEALREADYEXISTS%) <> 0 THEN GOTO 1810
1060     IF (BCCT3% = CONSTERRDISKFULL%) <> 0 THEN GOTO 1840
1070     IF (BCCT3% = CONSTERRINPUTPASTEND%) <> 0 THEN GOTO 1870
1080     IF (BCCT3% = CONSTERRBADRECORDNUMBER%) <> 0 THEN GOTO 1900
1090     IF (BCCT3% = CONSTERRBADFILENAME%) <> 0 THEN GOTO 1930
1100     IF (BCCT3% = CONSTERRTOOMANYFILES%) <> 0 THEN GOTO 1960
1110     IF (BCCT3% = CONSTERRDEVICEUNAVAILABLE%) <> 0 THEN GOTO 1990
1120     IF (BCCT3% = CONSTERRDISKWRITEPROTECTED%) <> 0 THEN GOTO 2020
1130     IF (BCCT3% = CONSTERRDISKNOTREADY%) <> 0 THEN GOTO 2050
1140     IF (BCCT3% = CONSTERRDISKMEDIAERROR%) <> 0 THEN GOTO 2080
1150     IF (BCCT3% = CONSTERRPATHFILEACCESS%) <> 0 THEN GOTO 2110
1160     IF (BCCT3% = CONSTERRPATHNOTFOUND%) <> 0 THEN GOTO 2140
1170     GOTO 2170
1180         errorResult0$ = "Syntax error"
1190         RETURN
1200         GOTO 2210
1210         errorResult0$ = "RETURN without GOSUB"
1220         RETURN
1230         GOTO 2210
1240         errorResult0$ = "Out of DATA"
1250         RETURN
1260         GOTO 2210
1270         errorResult0$ = "Illegal function call"
1280         RETURN
1290         GOTO 2210
1300         errorResult0$ = "Overflow"
1310         RETURN
1320         GOTO 2210
1330         errorResult0$ = "Out of memory"
1340         RETURN
1350         GOTO 2210
1360         errorResult0$ = "Subscript out of range"
1370         RETURN
1380         GOTO 2210
1390         errorResult0$ = "Duplicate Definition"
1400         RETURN
1410         GOTO 2210
1420         errorResult0$ = "Division by zero"
1430         RETURN
1440         GOTO 2210
1450         errorResult0$ = "Type mismatch"
1460         RETURN
1470         GOTO 2210
1480         errorResult0$ = "Out of string space"
1490         RETURN
1500         GOTO 2210
1510         errorResult0$ = "No RESUME"
1520         RETURN
1530         GOTO 2210
1540         errorResult0$ = "RESUME without error"
1550         RETURN
1560         GOTO 2210
1570         errorResult0$ = "Device timeout"
1580         RETURN
1590         GOTO 2210
1600         errorResult0$ = "Device fault"
1610         RETURN
1620         GOTO 2210
1630         errorResult0$ = "Out of paper"
1640         RETURN
1650         GOTO 2210
1660         errorResult0$ = "Bad file number"
1670         RETURN
1680         GOTO 2210
1690         errorResult0$ = "File not found"
1700         RETURN
1710         GOTO 2210
1720         errorResult0$ = "Bad file mode"
1730         RETURN
1740         GOTO 2210
1750         errorResult0$ = "File already open"
1760         RETURN
1770         GOTO 2210
1780         errorResult0$ = "Device I/O error"
1790         RETURN
1800         GOTO 2210
1810         errorResult0$ = "File already exists"
1820         RETURN
1830         GOTO 2210
1840         errorResult0$ = "Disk full"
1850         RETURN
1860         GOTO 2210
1870         errorResult0$ = "Input past end"
1880         RETURN
1890         GOTO 2210
1900         errorResult0$ = "Bad record number"
1910         RETURN
1920         GOTO 2210
1930         errorResult0$ = "Bad file name"
1940         RETURN
1950         GOTO 2210
1960         errorResult0$ = "Too many files"
1970         RETURN
1980         GOTO 2210
1990         errorResult0$ = "Device unavailable"
2000         RETURN
2010         GOTO 2210
2020         errorResult0$ = "Disk write protected"
2030         RETURN
2040         GOTO 2210
2050         errorResult0$ = "Disk not ready"
2060         RETURN
2070         GOTO 2210
2080         errorResult0$ = "Disk media error"
2090         RETURN
2100         GOTO 2210
2110         errorResult0$ = "Path/File access error"
2120         RETURN
2130         GOTO 2210
2140         errorResult0$ = "Path not found"
2150         RETURN
2160         GOTO 2210
2170         BCCT4% = errorCode0%
2180         BCCT5$ = STR$(BCCT4%)
2190         errorResult0$ = "Error " + BCCT5$
2200         RETURN
2210     REM END SELECT
2220     RETURN
2230 ' end function error$

2240 ' catch's optional source$ binding: map ERL back to its original .bcl file
2250     IF ERL <= 510 THEN BCCSOURCEFILE$ = "com/bascal/stdlib/error.bcl" : RETURN
2260     IF ERL <= 820 THEN BCCSOURCEFILE$ = "tutorial/portable_error_handling.bcl" : RETURN
2270     BCCSOURCEFILE$ = "com/bascal/stdlib/error.bcl"
2280     RETURN

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/portable_error_handling.c</code></summary>



```c

// BASCAL generated C -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
#include <stdio.h>
#include <string.h>
#include <stdlib.h>

#define BCC_STRBUF_COUNT 8
static char bcc_strbuf[BCC_STRBUF_COUNT][256];
static int bcc_strbuf_next = 0;

static int bcc_err = 0;
static int bcc_on_error_target = -1;
static int bcc_in_handler = 0;
static int bcc_resume_id = -1;
static int bcc_erl = 0;
static const char *bcc_err_file = "";

static char* bcc_strbuf_take(void);
static const char* bcc_mid(const char* s, int start, int length);
static const char* bcc_chr(int code);
static const char* bcc_stri(int value);
static const char* bcc_strd(double value);

static int bv_i_erl = 0;
static int bv_i_err = 0;
static int bv_i_err_bad_file_mode = 0;
static int bv_i_err_bad_file_name = 0;
static int bv_i_err_bad_file_number = 0;
static int bv_i_err_bad_record_number = 0;
static int bv_i_err_device_fault = 0;
static int bv_i_err_device_io = 0;
static int bv_i_err_device_timeout = 0;
static int bv_i_err_device_unavailable = 0;
static int bv_i_err_disk_full = 0;
static int bv_i_err_disk_media_error = 0;
static int bv_i_err_disk_not_ready = 0;
static int bv_i_err_disk_write_protected = 0;
static int bv_i_err_division_by_zero = 0;
static int bv_i_err_duplicate_definition = 0;
static int bv_i_err_file_already_exists = 0;
static int bv_i_err_file_already_open = 0;
static int bv_i_err_file_not_found = 0;
static int bv_i_err_illegal_function_call = 0;
static int bv_i_err_input_past_end = 0;
static int bv_i_err_no_resume = 0;
static int bv_i_err_out_of_data = 0;
static int bv_i_err_out_of_memory = 0;
static int bv_i_err_out_of_paper = 0;
static int bv_i_err_out_of_string_space = 0;
static int bv_i_err_overflow = 0;
static int bv_i_err_path_file_access = 0;
static int bv_i_err_path_not_found = 0;
static int bv_i_err_resume_without_error = 0;
static int bv_i_err_return_without_gosub = 0;
static int bv_i_err_subscript_out_of_range = 0;
static int bv_i_err_syntax = 0;
static int bv_i_err_too_many_files = 0;
static int bv_i_err_type_mismatch = 0;
static char bv_s_source[256] = {0};

void bf_s_error(int bv_i_code, char* bcc_out);

void bf_s_error(int bv_i_code, char* bcc_out) {
    {
        int bt_sel_0 = bv_i_code;
        int bt_sel_match_1 = 0;
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_syntax)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Syntax error");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_return_without_gosub)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "RETURN without GOSUB");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_out_of_data)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Out of DATA");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_illegal_function_call)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Illegal function call");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_overflow)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Overflow");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_out_of_memory)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Out of memory");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_subscript_out_of_range)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Subscript out of range");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_duplicate_definition)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Duplicate Definition");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_division_by_zero)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Division by zero");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_type_mismatch)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Type mismatch");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_out_of_string_space)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Out of string space");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_no_resume)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "No RESUME");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_resume_without_error)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "RESUME without error");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_device_timeout)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Device timeout");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_device_fault)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Device fault");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_out_of_paper)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Out of paper");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_bad_file_number)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Bad file number");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_file_not_found)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "File not found");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_bad_file_mode)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Bad file mode");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_file_already_open)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "File already open");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_device_io)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Device I/O error");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_file_already_exists)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "File already exists");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_disk_full)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Disk full");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_input_past_end)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Input past end");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_bad_record_number)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Bad record number");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_bad_file_name)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Bad file name");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_too_many_files)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Too many files");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_device_unavailable)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Device unavailable");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_disk_write_protected)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Disk write protected");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_disk_not_ready)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Disk not ready");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_disk_media_error)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Disk media error");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_path_file_access)) {
                bt_sel_match_1 = 1;
                snprintf(bcc_out, 256, "%s", "Path/File access error");
                return;
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == bv_i_err_path_not_found)) {
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

int main(void) {
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

    bv_i_err_syntax = 2;
    bv_i_err_return_without_gosub = 3;
    bv_i_err_out_of_data = 4;
    bv_i_err_illegal_function_call = 5;
    bv_i_err_overflow = 6;
    bv_i_err_out_of_memory = 7;
    bv_i_err_subscript_out_of_range = 9;
    bv_i_err_duplicate_definition = 10;
    bv_i_err_division_by_zero = 11;
    bv_i_err_type_mismatch = 13;
    bv_i_err_out_of_string_space = 14;
    bv_i_err_no_resume = 19;
    bv_i_err_resume_without_error = 20;
    bv_i_err_device_timeout = 24;
    bv_i_err_device_fault = 25;
    bv_i_err_out_of_paper = 27;
    bv_i_err_bad_file_number = 52;
    bv_i_err_file_not_found = 53;
    bv_i_err_bad_file_mode = 54;
    bv_i_err_file_already_open = 55;
    bv_i_err_device_io = 57;
    bv_i_err_file_already_exists = 58;
    bv_i_err_disk_full = 61;
    bv_i_err_input_past_end = 62;
    bv_i_err_bad_record_number = 63;
    bv_i_err_bad_file_name = 64;
    bv_i_err_too_many_files = 67;
    bv_i_err_device_unavailable = 68;
    bv_i_err_disk_write_protected = 70;
    bv_i_err_disk_not_ready = 71;
    bv_i_err_disk_media_error = 72;
    bv_i_err_path_file_access = 75;
    bv_i_err_path_not_found = 76;

    // Tutorial — Portable Structured Error Handling
    //
    // TRY/CATCH/FINALLY and THROW are BASCAL's portable error model.  A catch can
    // select several error codes and bind the originating source file.

    printf("portable try/catch:\n");
    int bcc_try_0_pending = 0;
    bcc_on_error_target = 0;
    bcc_err = bv_i_err_file_not_found;
    bcc_erl = 10;
    bcc_err_file = "tutorial/portable_error_handling.bcl";
    goto bcc_try_0_catch;
    bcc_on_error_target = -1;
    goto bcc_try_0_finally;
    bcc_try_0_catch: ;
    bcc_in_handler = 0;
    bcc_on_error_target = -1;
    if (!((bcc_err == bv_i_err_file_not_found) || (bcc_err == bv_i_err_file_already_open))) {
        bcc_try_0_pending = 1;
        goto bcc_try_0_finally;
    }
    bv_i_err = bcc_err;
    bv_i_erl = bcc_erl;
    snprintf(bv_s_source, 256, "%s", bcc_err_file);
    printf("  caught error %d at %s:%d\n", bv_i_err, bv_s_source, bv_i_erl);
    bcc_on_error_target = -1;
    goto bcc_try_0_finally;
    bcc_try_0_rethrow: ;
    bcc_try_0_pending = 1;
    bcc_try_0_finally: ;
    printf("  cleanup always runs\n");
    if (bcc_try_0_pending) {
        fprintf(stderr, "unhandled BASIC error %d\n", bcc_err);
        exit(1);
    }
    bcc_try_0_end: ;

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

<details class="source-embed" markdown="1">

<summary><code>tutorial/portable_error_handling.j</code></summary>



```basic

.class public PortableErrorHandling
.super java/lang/Object

.field public static g1 I
.field public static g2 I
.field public static g3 I
.field public static g4 I
.field public static g5 I
.field public static g6 I
.field public static g7 I
.field public static g8 I
.field public static g9 I
.field public static g10 I
.field public static g11 I
.field public static g12 I
.field public static g13 I
.field public static g14 I
.field public static g15 I
.field public static g16 I
.field public static g17 I
.field public static g18 I
.field public static g19 I
.field public static g20 I
.field public static g21 I
.field public static g22 I
.field public static g23 I
.field public static g24 I
.field public static g25 I
.field public static g26 I
.field public static g27 I
.field public static g28 I
.field public static g29 I
.field public static g30 I
.field public static g31 I
.field public static g32 I
.field public static g33 I
.field public static g34 I
.field public static g35 I
.field public static g36 Ljava/lang/String;
.method public static error : (I)Ljava/lang/String;

    iload 0
    dup
    getstatic PortableErrorHandling/g33 I
    isub
    ifeq L_select_0_case_0
    goto L_select_0_next_0
L_select_0_next_0:
    dup
    getstatic PortableErrorHandling/g31 I
    isub
    ifeq L_select_0_case_1
    goto L_select_0_next_1
L_select_0_next_1:
    dup
    getstatic PortableErrorHandling/g23 I
    isub
    ifeq L_select_0_case_2
    goto L_select_0_next_2
L_select_0_next_2:
    dup
    getstatic PortableErrorHandling/g20 I
    isub
    ifeq L_select_0_case_3
    goto L_select_0_next_3
L_select_0_next_3:
    dup
    getstatic PortableErrorHandling/g27 I
    isub
    ifeq L_select_0_case_4
    goto L_select_0_next_4
L_select_0_next_4:
    dup
    getstatic PortableErrorHandling/g24 I
    isub
    ifeq L_select_0_case_5
    goto L_select_0_next_5
L_select_0_next_5:
    dup
    getstatic PortableErrorHandling/g32 I
    isub
    ifeq L_select_0_case_6
    goto L_select_0_next_6
L_select_0_next_6:
    dup
    getstatic PortableErrorHandling/g16 I
    isub
    ifeq L_select_0_case_7
    goto L_select_0_next_7
L_select_0_next_7:
    dup
    getstatic PortableErrorHandling/g15 I
    isub
    ifeq L_select_0_case_8
    goto L_select_0_next_8
L_select_0_next_8:
    dup
    getstatic PortableErrorHandling/g35 I
    isub
    ifeq L_select_0_case_9
    goto L_select_0_next_9
L_select_0_next_9:
    dup
    getstatic PortableErrorHandling/g26 I
    isub
    ifeq L_select_0_case_10
    goto L_select_0_next_10
L_select_0_next_10:
    dup
    getstatic PortableErrorHandling/g22 I
    isub
    ifeq L_select_0_case_11
    goto L_select_0_next_11
L_select_0_next_11:
    dup
    getstatic PortableErrorHandling/g30 I
    isub
    ifeq L_select_0_case_12
    goto L_select_0_next_12
L_select_0_next_12:
    dup
    getstatic PortableErrorHandling/g9 I
    isub
    ifeq L_select_0_case_13
    goto L_select_0_next_13
L_select_0_next_13:
    dup
    getstatic PortableErrorHandling/g7 I
    isub
    ifeq L_select_0_case_14
    goto L_select_0_next_14
L_select_0_next_14:
    dup
    getstatic PortableErrorHandling/g25 I
    isub
    ifeq L_select_0_case_15
    goto L_select_0_next_15
L_select_0_next_15:
    dup
    getstatic PortableErrorHandling/g5 I
    isub
    ifeq L_select_0_case_16
    goto L_select_0_next_16
L_select_0_next_16:
    dup
    getstatic PortableErrorHandling/g19 I
    isub
    ifeq L_select_0_case_17
    goto L_select_0_next_17
L_select_0_next_17:
    dup
    getstatic PortableErrorHandling/g3 I
    isub
    ifeq L_select_0_case_18
    goto L_select_0_next_18
L_select_0_next_18:
    dup
    getstatic PortableErrorHandling/g18 I
    isub
    ifeq L_select_0_case_19
    goto L_select_0_next_19
L_select_0_next_19:
    dup
    getstatic PortableErrorHandling/g8 I
    isub
    ifeq L_select_0_case_20
    goto L_select_0_next_20
L_select_0_next_20:
    dup
    getstatic PortableErrorHandling/g17 I
    isub
    ifeq L_select_0_case_21
    goto L_select_0_next_21
L_select_0_next_21:
    dup
    getstatic PortableErrorHandling/g11 I
    isub
    ifeq L_select_0_case_22
    goto L_select_0_next_22
L_select_0_next_22:
    dup
    getstatic PortableErrorHandling/g21 I
    isub
    ifeq L_select_0_case_23
    goto L_select_0_next_23
L_select_0_next_23:
    dup
    getstatic PortableErrorHandling/g6 I
    isub
    ifeq L_select_0_case_24
    goto L_select_0_next_24
L_select_0_next_24:
    dup
    getstatic PortableErrorHandling/g4 I
    isub
    ifeq L_select_0_case_25
    goto L_select_0_next_25
L_select_0_next_25:
    dup
    getstatic PortableErrorHandling/g34 I
    isub
    ifeq L_select_0_case_26
    goto L_select_0_next_26
L_select_0_next_26:
    dup
    getstatic PortableErrorHandling/g10 I
    isub
    ifeq L_select_0_case_27
    goto L_select_0_next_27
L_select_0_next_27:
    dup
    getstatic PortableErrorHandling/g14 I
    isub
    ifeq L_select_0_case_28
    goto L_select_0_next_28
L_select_0_next_28:
    dup
    getstatic PortableErrorHandling/g13 I
    isub
    ifeq L_select_0_case_29
    goto L_select_0_next_29
L_select_0_next_29:
    dup
    getstatic PortableErrorHandling/g12 I
    isub
    ifeq L_select_0_case_30
    goto L_select_0_next_30
L_select_0_next_30:
    dup
    getstatic PortableErrorHandling/g28 I
    isub
    ifeq L_select_0_case_31
    goto L_select_0_next_31
L_select_0_next_31:
    dup
    getstatic PortableErrorHandling/g29 I
    isub
    ifeq L_select_0_case_32
    goto L_select_0_next_32
L_select_0_next_32:
    pop
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "Error "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    iload 0
    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    areturn
    goto L_select_0_end
L_select_0_case_0:
    pop
    ldc "Syntax error"
    areturn
    goto L_select_0_end
L_select_0_case_1:
    pop
    ldc "RETURN without GOSUB"
    areturn
    goto L_select_0_end
L_select_0_case_2:
    pop
    ldc "Out of DATA"
    areturn
    goto L_select_0_end
L_select_0_case_3:
    pop
    ldc "Illegal function call"
    areturn
    goto L_select_0_end
L_select_0_case_4:
    pop
    ldc "Overflow"
    areturn
    goto L_select_0_end
L_select_0_case_5:
    pop
    ldc "Out of memory"
    areturn
    goto L_select_0_end
L_select_0_case_6:
    pop
    ldc "Subscript out of range"
    areturn
    goto L_select_0_end
L_select_0_case_7:
    pop
    ldc "Duplicate Definition"
    areturn
    goto L_select_0_end
L_select_0_case_8:
    pop
    ldc "Division by zero"
    areturn
    goto L_select_0_end
L_select_0_case_9:
    pop
    ldc "Type mismatch"
    areturn
    goto L_select_0_end
L_select_0_case_10:
    pop
    ldc "Out of string space"
    areturn
    goto L_select_0_end
L_select_0_case_11:
    pop
    ldc "No RESUME"
    areturn
    goto L_select_0_end
L_select_0_case_12:
    pop
    ldc "RESUME without error"
    areturn
    goto L_select_0_end
L_select_0_case_13:
    pop
    ldc "Device timeout"
    areturn
    goto L_select_0_end
L_select_0_case_14:
    pop
    ldc "Device fault"
    areturn
    goto L_select_0_end
L_select_0_case_15:
    pop
    ldc "Out of paper"
    areturn
    goto L_select_0_end
L_select_0_case_16:
    pop
    ldc "Bad file number"
    areturn
    goto L_select_0_end
L_select_0_case_17:
    pop
    ldc "File not found"
    areturn
    goto L_select_0_end
L_select_0_case_18:
    pop
    ldc "Bad file mode"
    areturn
    goto L_select_0_end
L_select_0_case_19:
    pop
    ldc "File already open"
    areturn
    goto L_select_0_end
L_select_0_case_20:
    pop
    ldc "Device I/O error"
    areturn
    goto L_select_0_end
L_select_0_case_21:
    pop
    ldc "File already exists"
    areturn
    goto L_select_0_end
L_select_0_case_22:
    pop
    ldc "Disk full"
    areturn
    goto L_select_0_end
L_select_0_case_23:
    pop
    ldc "Input past end"
    areturn
    goto L_select_0_end
L_select_0_case_24:
    pop
    ldc "Bad record number"
    areturn
    goto L_select_0_end
L_select_0_case_25:
    pop
    ldc "Bad file name"
    areturn
    goto L_select_0_end
L_select_0_case_26:
    pop
    ldc "Too many files"
    areturn
    goto L_select_0_end
L_select_0_case_27:
    pop
    ldc "Device unavailable"
    areturn
    goto L_select_0_end
L_select_0_case_28:
    pop
    ldc "Disk write protected"
    areturn
    goto L_select_0_end
L_select_0_case_29:
    pop
    ldc "Disk not ready"
    areturn
    goto L_select_0_end
L_select_0_case_30:
    pop
    ldc "Disk media error"
    areturn
    goto L_select_0_end
L_select_0_case_31:
    pop
    ldc "Path/File access error"
    areturn
    goto L_select_0_end
L_select_0_case_32:
    pop
    ldc "Path not found"
    areturn
    goto L_select_0_end
L_select_0_end:
    ldc ""
    areturn
.end method

.method public static main : ([Ljava/lang/String;)V

    iconst_0
    putstatic PortableErrorHandling/g1 I
    iconst_0
    putstatic PortableErrorHandling/g2 I
    iconst_0
    putstatic PortableErrorHandling/g3 I
    iconst_0
    putstatic PortableErrorHandling/g4 I
    iconst_0
    putstatic PortableErrorHandling/g5 I
    iconst_0
    putstatic PortableErrorHandling/g6 I
    iconst_0
    putstatic PortableErrorHandling/g7 I
    iconst_0
    putstatic PortableErrorHandling/g8 I
    iconst_0
    putstatic PortableErrorHandling/g9 I
    iconst_0
    putstatic PortableErrorHandling/g10 I
    iconst_0
    putstatic PortableErrorHandling/g11 I
    iconst_0
    putstatic PortableErrorHandling/g12 I
    iconst_0
    putstatic PortableErrorHandling/g13 I
    iconst_0
    putstatic PortableErrorHandling/g14 I
    iconst_0
    putstatic PortableErrorHandling/g15 I
    iconst_0
    putstatic PortableErrorHandling/g16 I
    iconst_0
    putstatic PortableErrorHandling/g17 I
    iconst_0
    putstatic PortableErrorHandling/g18 I
    iconst_0
    putstatic PortableErrorHandling/g19 I
    iconst_0
    putstatic PortableErrorHandling/g20 I
    iconst_0
    putstatic PortableErrorHandling/g21 I
    iconst_0
    putstatic PortableErrorHandling/g22 I
    iconst_0
    putstatic PortableErrorHandling/g23 I
    iconst_0
    putstatic PortableErrorHandling/g24 I
    iconst_0
    putstatic PortableErrorHandling/g25 I
    iconst_0
    putstatic PortableErrorHandling/g26 I
    iconst_0
    putstatic PortableErrorHandling/g27 I
    iconst_0
    putstatic PortableErrorHandling/g28 I
    iconst_0
    putstatic PortableErrorHandling/g29 I
    iconst_0
    putstatic PortableErrorHandling/g30 I
    iconst_0
    putstatic PortableErrorHandling/g31 I
    iconst_0
    putstatic PortableErrorHandling/g32 I
    iconst_0
    putstatic PortableErrorHandling/g33 I
    iconst_0
    putstatic PortableErrorHandling/g34 I
    iconst_0
    putstatic PortableErrorHandling/g35 I
    ldc ""
    putstatic PortableErrorHandling/g36 Ljava/lang/String;
    ; Maps an ERR code to its classic MBASIC/GW-BASIC/BASCOM message. Compiles
    ; and links on a real IBM BASIC Compiler 2.00 as ERROR$, but silently
    ; returns an empty string at runtime (verified under dosbox-x) -- so BASCAL
    ; ships a working implementation.
    ;
    ; The named constants below are the complete common subset supported by
    ; ERROR$: use them in THROW and filtered CATCH clauses instead of magic
    ; numbers.  Dialect-specific errors outside this shared MBASIC/GW-BASIC/
    ; BASCOM subset still fall through to ERROR$'s generic message.
    ;
    ; Deliberately NOT a scalar method (see GitHub issue #41, which asked for
    ; this decision to be recorded either way): code% is an opaque lookup key,
    ; not a value the call is naturally "operating on" the way ltrim$/rtrim$/
    ; ucase$/lcase$ operate on their string -- code%.error() would read as if
    ; the *error code itself* has a message, when really this is a lookup
    ; table keyed by that code. Stays an ordinary function.
    ldc 2
    putstatic PortableErrorHandling/g33 I
    ldc 3
    putstatic PortableErrorHandling/g31 I
    ldc 4
    putstatic PortableErrorHandling/g23 I
    ldc 5
    putstatic PortableErrorHandling/g20 I
    ldc 6
    putstatic PortableErrorHandling/g27 I
    ldc 7
    putstatic PortableErrorHandling/g24 I
    ldc 9
    putstatic PortableErrorHandling/g32 I
    ldc 10
    putstatic PortableErrorHandling/g16 I
    ldc 11
    putstatic PortableErrorHandling/g15 I
    ldc 13
    putstatic PortableErrorHandling/g35 I
    ldc 14
    putstatic PortableErrorHandling/g26 I
    ldc 19
    putstatic PortableErrorHandling/g22 I
    ldc 20
    putstatic PortableErrorHandling/g30 I
    ldc 24
    putstatic PortableErrorHandling/g9 I
    ldc 25
    putstatic PortableErrorHandling/g7 I
    ldc 27
    putstatic PortableErrorHandling/g25 I
    ldc 52
    putstatic PortableErrorHandling/g5 I
    ldc 53
    putstatic PortableErrorHandling/g19 I
    ldc 54
    putstatic PortableErrorHandling/g3 I
    ldc 55
    putstatic PortableErrorHandling/g18 I
    ldc 57
    putstatic PortableErrorHandling/g8 I
    ldc 58
    putstatic PortableErrorHandling/g17 I
    ldc 61
    putstatic PortableErrorHandling/g11 I
    ldc 62
    putstatic PortableErrorHandling/g21 I
    ldc 63
    putstatic PortableErrorHandling/g6 I
    ldc 64
    putstatic PortableErrorHandling/g4 I
    ldc 67
    putstatic PortableErrorHandling/g34 I
    ldc 68
    putstatic PortableErrorHandling/g10 I
    ldc 70
    putstatic PortableErrorHandling/g14 I
    ldc 71
    putstatic PortableErrorHandling/g13 I
    ldc 72
    putstatic PortableErrorHandling/g12 I
    ldc 75
    putstatic PortableErrorHandling/g28 I
    ldc 76
    putstatic PortableErrorHandling/g29 I
    ; Tutorial — Portable Structured Error Handling
    ;
    ; TRY/CATCH/FINALLY and THROW are BASCAL's portable error model.  A catch can
    ; select several error codes and bind the originating source file.
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "portable try/catch:"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
L_try_0_start:
    new java/lang/RuntimeException
    dup
    getstatic PortableErrorHandling/g19 I
    invokestatic java/lang/Integer/toString (I)Ljava/lang/String;
    invokespecial java/lang/RuntimeException/<init> (Ljava/lang/String;)V
    athrow
    goto L_try_0_finish
L_try_0_end:
L_try_0_catch:
    invokevirtual java/lang/Throwable/getMessage ()Ljava/lang/String;
    invokestatic java/lang/Integer/parseInt (Ljava/lang/String;)I
    putstatic PortableErrorHandling/g2 I
    iconst_0
    putstatic PortableErrorHandling/g1 I
    ldc "tutorial/portable_error_handling.bcl"
    putstatic PortableErrorHandling/g36 Ljava/lang/String;
    getstatic PortableErrorHandling/g2 I
    getstatic PortableErrorHandling/g19 I
    if_icmpeq L_try_0_matched
    getstatic PortableErrorHandling/g2 I
    getstatic PortableErrorHandling/g18 I
    if_icmpeq L_try_0_matched
    goto L_try_0_rethrow
L_try_0_matched:
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  caught error "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic PortableErrorHandling/g2 I
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc " at "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic PortableErrorHandling/g36 Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc ":"
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic PortableErrorHandling/g1 I
    invokevirtual java/io/PrintStream/println (I)V
    goto L_try_0_finish
L_try_0_rethrow:
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  cleanup always runs"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    new java/lang/RuntimeException
    dup
    getstatic PortableErrorHandling/g2 I
    invokestatic java/lang/Integer/toString (I)Ljava/lang/String;
    invokespecial java/lang/RuntimeException/<init> (Ljava/lang/String;)V
    athrow
L_try_0_finish:
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  cleanup always runs"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    return
    .catch java/lang/RuntimeException from L_try_0_start to L_try_0_end using L_try_0_catch
.end method

```



</details>

<!-- END generated tutorial source -->
