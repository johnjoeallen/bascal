[Home](../../) / [Tutorials](../) / Portable Structured Error Handling

<div class="prose" markdown="1">

Generated sources: [BCL](https://github.com/johnjoeallen/bascal/blob/main/tutorial/21_portable_error_handling.bcl), [BASIC](https://github.com/johnjoeallen/bascal/blob/main/tutorial/21_portable_error_handling.bas), [C](https://github.com/johnjoeallen/bascal/blob/main/tutorial/21_portable_error_handling.c), and [JVM assembly](https://github.com/johnjoeallen/bascal/blob/main/tutorial/21_portable_error_handling.j).

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

<summary><code>tutorial/21_portable_error_handling.bcl</code></summary>



```bascal

// Tutorial — Portable Structured Error Handling
//
// TRY/CATCH/FINALLY and THROW are BASCAL's portable error model.  A catch can
// select several error codes and bind the originating source file.
program portableErrorHandling
require com.bascal.stdlib.error

print "portable try/catch:"
try
    throw errFileNotFound%
catch err%(errFileNotFound%, errFileAlreadyOpen%), erl%, source$
    print "  caught error "; err%; " at "; source$; ":"; erl%
finally
    print "  cleanup always runs"
end try

end

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/21_portable_error_handling.bas</code></summary>



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

190 errsyntax% = 2
200 errreturnwithoutgosub% = 3
210 erroutofdata% = 4
220 errillegalfunctioncall% = 5
230 erroverflow% = 6
240 erroutofmemory% = 7
250 errsubscriptoutofrange% = 9
260 errduplicatedefinition% = 10
270 errdivisionbyzero% = 11
280 errtypemismatch% = 13
290 erroutofstringspace% = 14
300 errnoresume% = 19
310 errresumewithouterror% = 20
320 errdevicetimeout% = 24
330 errdevicefault% = 25
340 erroutofpaper% = 27
350 errbadfilenumber% = 52
360 errfilenotfound% = 53
370 errbadfilemode% = 54
380 errfilealreadyopen% = 55
390 errdeviceio% = 57
400 errfilealreadyexists% = 58
410 errdiskfull% = 61
420 errinputpastend% = 62
430 errbadrecordnumber% = 63
440 errbadfilename% = 64
450 errtoomanyfiles% = 67
460 errdeviceunavailable% = 68
470 errdiskwriteprotected% = 70
480 errdisknotready% = 71
490 errdiskmediaerror% = 72
500 errpathfileaccess% = 75
510 errpathnotfound% = 76

520 ' Tutorial — Portable Structured Error Handling
530 '
540 ' TRY/CATCH/FINALLY and THROW are BASCAL's portable error model.  A catch can
550 ' select several error codes and bind the originating source file.

560 PRINT "portable try/catch:"
570 ON ERROR GOTO 620
580 BCC_TRY_0001_PENDING% = 0
590     ERROR errfilenotfound%
600 ON ERROR GOTO 0
610 GOTO 770
620     BCC_TRY_0001_PENDING% = ERR
630     IF (ERR = errfilenotfound%) OR (ERR = errfilealreadyopen%) THEN GOTO 650
640     RESUME 770
650     err% = ERR
660     erl% = ERL
670     GOSUB 2230
680     source$ = BCC_SOURCE_FILE$
690     RESUME 700
700 ON ERROR GOTO 750
710     PRINT "  caught error "; err%; " at "; source$; ":"; erl%
720     BCC_TRY_0001_PENDING% = 0
730     ON ERROR GOTO 0
740     GOTO 770
750     BCC_TRY_0001_PENDING% = ERR
760     RESUME 770
770 ON ERROR GOTO 0
780     PRINT "  cleanup always runs"
790     IF BCC_TRY_0001_PENDING% <> 0 THEN ERROR BCC_TRY_0001_PENDING%
800 REM END TRY

810 END

820 ' function error$(code%)
830     BCCT3% = errorCode0%
840     IF (BCCT3% = errsyntax%) <> 0 THEN GOTO 1180
850     IF (BCCT3% = errreturnwithoutgosub%) <> 0 THEN GOTO 1210
860     IF (BCCT3% = erroutofdata%) <> 0 THEN GOTO 1240
870     IF (BCCT3% = errillegalfunctioncall%) <> 0 THEN GOTO 1270
880     IF (BCCT3% = erroverflow%) <> 0 THEN GOTO 1300
890     IF (BCCT3% = erroutofmemory%) <> 0 THEN GOTO 1330
900     IF (BCCT3% = errsubscriptoutofrange%) <> 0 THEN GOTO 1360
910     IF (BCCT3% = errduplicatedefinition%) <> 0 THEN GOTO 1390
920     IF (BCCT3% = errdivisionbyzero%) <> 0 THEN GOTO 1420
930     IF (BCCT3% = errtypemismatch%) <> 0 THEN GOTO 1450
940     IF (BCCT3% = erroutofstringspace%) <> 0 THEN GOTO 1480
950     IF (BCCT3% = errnoresume%) <> 0 THEN GOTO 1510
960     IF (BCCT3% = errresumewithouterror%) <> 0 THEN GOTO 1540
970     IF (BCCT3% = errdevicetimeout%) <> 0 THEN GOTO 1570
980     IF (BCCT3% = errdevicefault%) <> 0 THEN GOTO 1600
990     IF (BCCT3% = erroutofpaper%) <> 0 THEN GOTO 1630
1000     IF (BCCT3% = errbadfilenumber%) <> 0 THEN GOTO 1660
1010     IF (BCCT3% = errfilenotfound%) <> 0 THEN GOTO 1690
1020     IF (BCCT3% = errbadfilemode%) <> 0 THEN GOTO 1720
1030     IF (BCCT3% = errfilealreadyopen%) <> 0 THEN GOTO 1750
1040     IF (BCCT3% = errdeviceio%) <> 0 THEN GOTO 1780
1050     IF (BCCT3% = errfilealreadyexists%) <> 0 THEN GOTO 1810
1060     IF (BCCT3% = errdiskfull%) <> 0 THEN GOTO 1840
1070     IF (BCCT3% = errinputpastend%) <> 0 THEN GOTO 1870
1080     IF (BCCT3% = errbadrecordnumber%) <> 0 THEN GOTO 1900
1090     IF (BCCT3% = errbadfilename%) <> 0 THEN GOTO 1930
1100     IF (BCCT3% = errtoomanyfiles%) <> 0 THEN GOTO 1960
1110     IF (BCCT3% = errdeviceunavailable%) <> 0 THEN GOTO 1990
1120     IF (BCCT3% = errdiskwriteprotected%) <> 0 THEN GOTO 2020
1130     IF (BCCT3% = errdisknotready%) <> 0 THEN GOTO 2050
1140     IF (BCCT3% = errdiskmediaerror%) <> 0 THEN GOTO 2080
1150     IF (BCCT3% = errpathfileaccess%) <> 0 THEN GOTO 2110
1160     IF (BCCT3% = errpathnotfound%) <> 0 THEN GOTO 2140
1170     GOTO 2170
1180         errorResult0$ = "Syntax error"
1190         RETURN
1200         GOTO 2190
1210         errorResult0$ = "RETURN without GOSUB"
1220         RETURN
1230         GOTO 2190
1240         errorResult0$ = "Out of DATA"
1250         RETURN
1260         GOTO 2190
1270         errorResult0$ = "Illegal function call"
1280         RETURN
1290         GOTO 2190
1300         errorResult0$ = "Overflow"
1310         RETURN
1320         GOTO 2190
1330         errorResult0$ = "Out of memory"
1340         RETURN
1350         GOTO 2190
1360         errorResult0$ = "Subscript out of range"
1370         RETURN
1380         GOTO 2190
1390         errorResult0$ = "Duplicate Definition"
1400         RETURN
1410         GOTO 2190
1420         errorResult0$ = "Division by zero"
1430         RETURN
1440         GOTO 2190
1450         errorResult0$ = "Type mismatch"
1460         RETURN
1470         GOTO 2190
1480         errorResult0$ = "Out of string space"
1490         RETURN
1500         GOTO 2190
1510         errorResult0$ = "No RESUME"
1520         RETURN
1530         GOTO 2190
1540         errorResult0$ = "RESUME without error"
1550         RETURN
1560         GOTO 2190
1570         errorResult0$ = "Device timeout"
1580         RETURN
1590         GOTO 2190
1600         errorResult0$ = "Device fault"
1610         RETURN
1620         GOTO 2190
1630         errorResult0$ = "Out of paper"
1640         RETURN
1650         GOTO 2190
1660         errorResult0$ = "Bad file number"
1670         RETURN
1680         GOTO 2190
1690         errorResult0$ = "File not found"
1700         RETURN
1710         GOTO 2190
1720         errorResult0$ = "Bad file mode"
1730         RETURN
1740         GOTO 2190
1750         errorResult0$ = "File already open"
1760         RETURN
1770         GOTO 2190
1780         errorResult0$ = "Device I/O error"
1790         RETURN
1800         GOTO 2190
1810         errorResult0$ = "File already exists"
1820         RETURN
1830         GOTO 2190
1840         errorResult0$ = "Disk full"
1850         RETURN
1860         GOTO 2190
1870         errorResult0$ = "Input past end"
1880         RETURN
1890         GOTO 2190
1900         errorResult0$ = "Bad record number"
1910         RETURN
1920         GOTO 2190
1930         errorResult0$ = "Bad file name"
1940         RETURN
1950         GOTO 2190
1960         errorResult0$ = "Too many files"
1970         RETURN
1980         GOTO 2190
1990         errorResult0$ = "Device unavailable"
2000         RETURN
2010         GOTO 2190
2020         errorResult0$ = "Disk write protected"
2030         RETURN
2040         GOTO 2190
2050         errorResult0$ = "Disk not ready"
2060         RETURN
2070         GOTO 2190
2080         errorResult0$ = "Disk media error"
2090         RETURN
2100         GOTO 2190
2110         errorResult0$ = "Path/File access error"
2120         RETURN
2130         GOTO 2190
2140         errorResult0$ = "Path not found"
2150         RETURN
2160         GOTO 2190
2170         errorResult0$ = "Error " + STR$(errorCode0%)
2180         RETURN
2190     REM END SELECT
2200     RETURN
2210 ' end function error$

2220 ' catch's optional source$ binding: map ERL back to its original .bcl file
2230     IF ERL <= 510 THEN BCC_SOURCE_FILE$ = "com/bascal/stdlib/error.bcl" : RETURN
2240     IF ERL <= 820 THEN BCC_SOURCE_FILE$ = "tutorial/21_portable_error_handling.bcl" : RETURN
2250     BCC_SOURCE_FILE$ = "com/bascal/stdlib/error.bcl"
2260     RETURN

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/21_portable_error_handling.c</code></summary>



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
static char bv_s_source[256] = {0};

void bf_s_error(int bv_i_code, char* bcc_out);

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

    // Tutorial — Portable Structured Error Handling
    //
    // TRY/CATCH/FINALLY and THROW are BASCAL's portable error model.  A catch can
    // select several error codes and bind the originating source file.

    printf("portable try/catch:\n");
    int bcc_try_0_pending = 0;
    bcc_on_error_target = 0;
    bcc_err = bv_i_errfilenotfound;
    bcc_erl = 10;
    bcc_err_file = "tutorial/21_portable_error_handling.bcl";
    goto bcc_try_0_catch;
    bcc_on_error_target = -1;
    goto bcc_try_0_finally;
    bcc_try_0_catch: ;
    bcc_in_handler = 0;
    bcc_on_error_target = -1;
    if (!((bcc_err == bv_i_errfilenotfound) || (bcc_err == bv_i_errfilealreadyopen))) {
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

<summary><code>tutorial/21_portable_error_handling.j</code></summary>



```basic

.version 50 0
.class public PortableErrorHandling
.super java/lang/Object

.field public static g1 I
.field public static g2 I
.field public static g3 Ljava/lang/String;
.method public static error : (I)Ljava/lang/String;
    .limit stack 16
    .limit locals 1

    iload 0
    dup
    ldc 2
    isub
    ifeq L_select_0_case_0
    goto L_select_0_next_0
L_select_0_next_0:
    dup
    ldc 3
    isub
    ifeq L_select_0_case_1
    goto L_select_0_next_1
L_select_0_next_1:
    dup
    ldc 4
    isub
    ifeq L_select_0_case_2
    goto L_select_0_next_2
L_select_0_next_2:
    dup
    ldc 5
    isub
    ifeq L_select_0_case_3
    goto L_select_0_next_3
L_select_0_next_3:
    dup
    ldc 6
    isub
    ifeq L_select_0_case_4
    goto L_select_0_next_4
L_select_0_next_4:
    dup
    ldc 7
    isub
    ifeq L_select_0_case_5
    goto L_select_0_next_5
L_select_0_next_5:
    dup
    ldc 9
    isub
    ifeq L_select_0_case_6
    goto L_select_0_next_6
L_select_0_next_6:
    dup
    ldc 10
    isub
    ifeq L_select_0_case_7
    goto L_select_0_next_7
L_select_0_next_7:
    dup
    ldc 11
    isub
    ifeq L_select_0_case_8
    goto L_select_0_next_8
L_select_0_next_8:
    dup
    ldc 13
    isub
    ifeq L_select_0_case_9
    goto L_select_0_next_9
L_select_0_next_9:
    dup
    ldc 14
    isub
    ifeq L_select_0_case_10
    goto L_select_0_next_10
L_select_0_next_10:
    dup
    ldc 19
    isub
    ifeq L_select_0_case_11
    goto L_select_0_next_11
L_select_0_next_11:
    dup
    ldc 20
    isub
    ifeq L_select_0_case_12
    goto L_select_0_next_12
L_select_0_next_12:
    dup
    ldc 24
    isub
    ifeq L_select_0_case_13
    goto L_select_0_next_13
L_select_0_next_13:
    dup
    ldc 25
    isub
    ifeq L_select_0_case_14
    goto L_select_0_next_14
L_select_0_next_14:
    dup
    ldc 27
    isub
    ifeq L_select_0_case_15
    goto L_select_0_next_15
L_select_0_next_15:
    dup
    ldc 52
    isub
    ifeq L_select_0_case_16
    goto L_select_0_next_16
L_select_0_next_16:
    dup
    ldc 53
    isub
    ifeq L_select_0_case_17
    goto L_select_0_next_17
L_select_0_next_17:
    dup
    ldc 54
    isub
    ifeq L_select_0_case_18
    goto L_select_0_next_18
L_select_0_next_18:
    dup
    ldc 55
    isub
    ifeq L_select_0_case_19
    goto L_select_0_next_19
L_select_0_next_19:
    dup
    ldc 57
    isub
    ifeq L_select_0_case_20
    goto L_select_0_next_20
L_select_0_next_20:
    dup
    ldc 58
    isub
    ifeq L_select_0_case_21
    goto L_select_0_next_21
L_select_0_next_21:
    dup
    ldc 61
    isub
    ifeq L_select_0_case_22
    goto L_select_0_next_22
L_select_0_next_22:
    dup
    ldc 62
    isub
    ifeq L_select_0_case_23
    goto L_select_0_next_23
L_select_0_next_23:
    dup
    ldc 63
    isub
    ifeq L_select_0_case_24
    goto L_select_0_next_24
L_select_0_next_24:
    dup
    ldc 64
    isub
    ifeq L_select_0_case_25
    goto L_select_0_next_25
L_select_0_next_25:
    dup
    ldc 67
    isub
    ifeq L_select_0_case_26
    goto L_select_0_next_26
L_select_0_next_26:
    dup
    ldc 68
    isub
    ifeq L_select_0_case_27
    goto L_select_0_next_27
L_select_0_next_27:
    dup
    ldc 70
    isub
    ifeq L_select_0_case_28
    goto L_select_0_next_28
L_select_0_next_28:
    dup
    ldc 71
    isub
    ifeq L_select_0_case_29
    goto L_select_0_next_29
L_select_0_next_29:
    dup
    ldc 72
    isub
    ifeq L_select_0_case_30
    goto L_select_0_next_30
L_select_0_next_30:
    dup
    ldc 75
    isub
    ifeq L_select_0_case_31
    goto L_select_0_next_31
L_select_0_next_31:
    dup
    ldc 76
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
    .limit stack 16
    .limit locals 4

    iconst_0
    putstatic PortableErrorHandling/g1 I
    iconst_0
    putstatic PortableErrorHandling/g2 I
    ldc ""
    putstatic PortableErrorHandling/g3 Ljava/lang/String;
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
    ldc 53
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
    ldc "tutorial/21_portable_error_handling.bcl"
    putstatic PortableErrorHandling/g3 Ljava/lang/String;
    getstatic PortableErrorHandling/g2 I
    ldc 53
    if_icmpeq L_try_0_matched
    getstatic PortableErrorHandling/g2 I
    ldc 55
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
    getstatic PortableErrorHandling/g3 Ljava/lang/String;
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
