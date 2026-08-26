[Home](../../) / [Tutorials](../) / Standard Library Functions

<div class="prose" markdown="1">

Generated sources: [BCL](https://github.com/johnjoeallen/bascal/blob/main/tutorial/18_stdlib.bcl), [BASIC](https://github.com/johnjoeallen/bascal/blob/main/tutorial/18_stdlib.bas), [C](https://github.com/johnjoeallen/bascal/blob/main/tutorial/18_stdlib.c), and [JVM assembly](https://github.com/johnjoeallen/bascal/blob/main/tutorial/18_stdlib.j).

`LTRIM$`, `RTRIM$`, `UCASE$`, `LCASE$`, and `ERROR$` either aren't real MBASIC/BASCOM 2.00 builtins at all, or (in `ERROR$`'s case) compile and link but silently return an empty string at runtime -- verified against a real IBM Personal Computer BASIC Compiler 2.00 running under dosbox-x. BASCAL ships its own implementations under `com.bascal.stdlib`, resolved exactly like `com.bascal.sort` in [tutorial 12](12_require.md) -- but `bcc` always adds the library's install location to its search path automatically (next to the binary, or `usr/share/bascal` for a `.deb`/`.rpm` install), so no `-L` flag is needed to reach it.

</div>

<div class="snippet" markdown="1">

### Requiring the functions you need, then calling them like any other

```bascal
require com.bascal.stdlib.ltrim
require com.bascal.stdlib.rtrim
require com.bascal.stdlib.ucase
require com.bascal.stdlib.lcase
require com.bascal.stdlib.error

print "[" + "   padded left".ltrim() + "]"
print "[" + "padded right   ".rtrim() + "]"
print "shout this".ucase()
print "QUIET THIS DOWN".lcase()
```

</div>

<div class="snippet" markdown="1">

### ERROR\$ maps a classic error code to a message

Pair it with ERR inside an ON ERROR GOTO handler in real code -- see [classic tutorial 17](17_labels_and_error_handling.md), or use [portable structured handling](21_portable_error_handling.md).

```bascal
print error$(53)   ' File not found
print error$(11)   ' Division by zero
print error$(9999) ' Error  9999 (falls through to STR$)
```

</div>



[← Portable Error Handling](21_portable_error_handling.md)  ·  [Case Study: Random-Access Inventory →](19_inventory.md)


<!-- BEGIN generated tutorial source -->

<details class="source-embed" markdown="1">

<summary><code>tutorial/18_stdlib.bcl</code></summary>



```bascal

// Tutorial — Standard library functions
//
// com.bascal.stdlib is an ordinary require-able library, resolved the same
// way as com.bascal.sort in tutorial 12 -- but bcc always adds its home
// directory to the search path automatically, so no -L flag is needed to
// reach it. It exists because LTRIM$, RTRIM$, UCASE$, LCASE$, and ERROR$
// either aren't real MBASIC/BASCOM 2.00 builtins or don't work at runtime
// (verified against a real IBM Personal Computer BASIC Compiler 2.00 under
// dosbox-x) -- see the manual's "String and error-message functions"
// section (https://johnjoeallen.github.io/bascal/manual/) for the full
// story.
//
// ltrim$/rtrim$/ucase$/lcase$ are scalar methods with a bracketed string
// receiver type, using self$ in place of an explicit s$ parameter -- see
// the "Declare and call a method" chapter. A method's receiver is really
// just an implicit first parameter, so the ordinary call form
// (ltrim$("...")) keeps working exactly as before: it resolves straight to
// the same method declaration, with the first argument filling self$. The
// examples below prefer the method-call form, written as "...".ltrim().
// error$ stays an ordinary function: an
// error code is a lookup key, not a value the call is naturally "operating
// on" the way the others operate on their string.
//
// Run with:
//   bcc tutorial/18_stdlib.bcl
program stdlib

require com.bascal.stdlib.ltrim
require com.bascal.stdlib.rtrim
require com.bascal.stdlib.ucase
require com.bascal.stdlib.lcase
require com.bascal.stdlib.error

print "[" + "   padded left".ltrim() + "]"
print "[" + "padded right   ".rtrim() + "]"
print "shout this".ucase()
print "QUIET THIS DOWN".lcase()

/* Same four functions, called as chained methods instead. */
print "[" + "  padded both sides  ".ltrim().rtrim() + "]"
print "  shout this too".ltrim().ucase()

/* ERROR$ maps a classic MBASIC/GW-BASIC/BASCOM error code to a message;
   pair it with ERR inside an ON ERROR GOTO handler in real code. */
print error$(53)
print error$(11)
print error$(9999)

end

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/18_stdlib.bas</code></summary>



```basic

10 ' BASCAL generated BASIC -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Strips leading spaces from self$. Not a real MBASIC/BASCOM 2.00 builtin --
40 ' verified against a real IBM BASIC Compiler 2.00 under dosbox-x -- so
50 ' BASCAL ships its own. Declared as a scalar method (see GitHub issue #41)
60 ' so a required stdlib call reads the same way as a built-in method call
70 ' (docs/language/functions-and-procedures.html#built-in-methods). The
80 ' ordinary call form (ltrim$(s$)) still works -- a method's receiver is an
90 ' implicit first parameter, so ordinary-call syntax resolves straight to
100 ' this same declaration, with no separate function needed (and no longer
110 ' allowed: a function and a method sharing one name is a duplicate
120 ' declaration, since they'd both claim the same callable identity).

130 ' Strips trailing spaces from self$. Not a real MBASIC/BASCOM 2.00 builtin --
140 ' verified against a real IBM BASIC Compiler 2.00 under dosbox-x -- so
150 ' BASCAL ships its own. Declared as a scalar method (see GitHub issue #41
160 ' and ltrim.bcl's own doc comment for the reasoning) -- rtrim$(s$) still
170 ' works via ordinary-call syntax resolving to this same declaration.

180 ' Upper-cases self$. Not a real MBASIC/BASCOM 2.00 builtin -- verified
190 ' against a real IBM BASIC Compiler 2.00 under dosbox-x -- so BASCAL ships
200 ' its own. Declared as a scalar method (see GitHub issue #41 and
210 ' ltrim.bcl's own doc comment for the reasoning) -- ucase$(s$) still works
220 ' via ordinary-call syntax resolving to this same declaration.

230 ' Lower-cases self$. Not a real MBASIC/BASCOM 2.00 builtin -- verified
240 ' against a real IBM BASIC Compiler 2.00 under dosbox-x -- so BASCAL ships
250 ' its own. Declared as a scalar method (see GitHub issue #41 and
260 ' ltrim.bcl's own doc comment for the reasoning) -- lcase$(s$) still works
270 ' via ordinary-call syntax resolving to this same declaration.

280 ' Maps an ERR code to its classic MBASIC/GW-BASIC/BASCOM message. Compiles
290 ' and links on a real IBM BASIC Compiler 2.00 as ERROR$, but silently
300 ' returns an empty string at runtime (verified under dosbox-x) -- so BASCAL
310 ' ships a working implementation.
320 '
330 ' Covers the classic error codes an ON ERROR GOTO + ERR handler is
340 ' realistically going to hit -- not the full table, but every code common
350 ' enough to be worth a real message instead of falling through to the
360 ' generic one.
370 '
380 ' Deliberately NOT a scalar method (see GitHub issue #41, which asked for
390 ' this decision to be recorded either way): code% is an opaque lookup key,
400 ' not a value the call is naturally "operating on" the way ltrim$/rtrim$/
410 ' ucase$/lcase$ operate on their string -- code%.error() would read as if
420 ' the *error code itself* has a message, when really this is a lookup
430 ' table keyed by that code. Stays an ordinary function.

440 ' Tutorial — Standard library functions
450 '
460 ' com.bascal.stdlib is an ordinary require-able library, resolved the same
470 ' way as com.bascal.sort in tutorial 12 -- but bcc always adds its home
480 ' directory to the search path automatically, so no -L flag is needed to
490 ' reach it. It exists because LTRIM$, RTRIM$, UCASE$, LCASE$, and ERROR$
500 ' either aren't real MBASIC/BASCOM 2.00 builtins or don't work at runtime
510 ' (verified against a real IBM Personal Computer BASIC Compiler 2.00 under
520 ' dosbox-x) -- see the manual's "String and error-message functions"
530 ' section (https://johnjoeallen.github.io/bascal/manual/) for the full
540 ' story.
550 '
560 ' ltrim$/rtrim$/ucase$/lcase$ are declared as scalar methods (method$ ...
570 ' end method), using self$ in place of an explicit s$ parameter -- see
580 ' the "Declare and call a method" chapter. A method's receiver is really
590 ' just an implicit first parameter, so the ordinary call form below
600 ' (ltrim$("...")) keeps working exactly as before: it resolves straight to
610 ' the same method declaration, with the first argument filling self$. The
620 ' method-call form (below, chained) is the same declaration too -- just
630 ' written as "...".ltrim() instead. error$ stays an ordinary function: an
640 ' error code is a lookup key, not a value the call is naturally "operating
650 ' on" the way the others operate on their string.
660 '
670 ' Run with:
680 ' bcc tutorial/18_stdlib.bcl

690 ltrimSelf0$ = "   padded left"
700 GOSUB 1050
710 PRINT ("[" + ltrimResult0$) + "]"
720 rtrimSelf0$ = "padded right   "
730 GOSUB 1150
740 PRINT ("[" + rtrimResult0$) + "]"
750 ucaseSelf0$ = "shout this"
760 GOSUB 1250
770 PRINT ucaseResult0$
780 lcaseSelf0$ = "QUIET THIS DOWN"
790 GOSUB 1380
800 PRINT lcaseResult0$

810 ' Same four functions, called as chained methods instead.
820 ltrimSelf0$ = "  padded both sides  "
830 GOSUB 1050
840 rtrimSelf0$ = ltrimResult0$
850 GOSUB 1150
860 PRINT ("[" + rtrimResult0$) + "]"
870 ltrimSelf0$ = "  shout this too"
880 GOSUB 1050
890 ucaseSelf0$ = ltrimResult0$
900 GOSUB 1250
910 PRINT ucaseResult0$

920 ' ERROR$ maps a classic MBASIC/GW-BASIC/BASCOM error code to a message;
930 ' pair it with ERR inside an ON ERROR GOTO handler in real code.
940 errorCode0% = 53
950 GOSUB 1510
960 PRINT errorResult0$
970 errorCode0% = 11
980 GOSUB 1510
990 PRINT errorResult0$
1000 errorCode0% = 9999
1010 GOSUB 1510
1020 PRINT errorResult0$

1030 END

1040 ' function ltrim$()
1050     ltrimI0% = 1
1060     IF (ltrimI0% <= LEN(ltrimSelf0$)) = 0 THEN GOTO 1100
1070     IF (MID$(ltrimSelf0$, ltrimI0%, 1) = " ") = 0 THEN GOTO 1100
1080         ltrimI0% = ltrimI0% + 1
1090         GOTO 1060
1100     REM END WHILE
1110     ltrimResult0$ = MID$(ltrimSelf0$, ltrimI0%)
1120     RETURN
1130 ' end function ltrim$

1140 ' function rtrim$()
1150     rtrimI0% = LEN(rtrimSelf0$)
1160     IF (rtrimI0% > 0) = 0 THEN GOTO 1200
1170     IF (MID$(rtrimSelf0$, rtrimI0%, 1) = " ") = 0 THEN GOTO 1200
1180         rtrimI0% = rtrimI0% - 1
1190         GOTO 1160
1200     REM END WHILE
1210     rtrimResult0$ = LEFT$(rtrimSelf0$, rtrimI0%)
1220     RETURN
1230 ' end function rtrim$

1240 ' function ucase$()
1250     ucaseOut0$ = ""
1260     FOR ucaseI0% = 1 TO LEN(ucaseSelf0$)
1270         ucaseC0% = ASC(MID$(ucaseSelf0$, ucaseI0%, 1))
1280         IF (ucaseC0% >= 97) = 0 THEN GOTO 1310
1290         IF (ucaseC0% <= 122) = 0 THEN GOTO 1310
1300             ucaseC0% = ucaseC0% - 32
1310         REM END IF
1320         ucaseOut0$ = ucaseOut0$ + CHR$(ucaseC0%)
1330     NEXT ucaseI0%
1340     ucaseResult0$ = ucaseOut0$
1350     RETURN
1360 ' end function ucase$

1370 ' function lcase$()
1380     lcaseOut0$ = ""
1390     FOR lcaseI0% = 1 TO LEN(lcaseSelf0$)
1400         lcaseC0% = ASC(MID$(lcaseSelf0$, lcaseI0%, 1))
1410         IF (lcaseC0% >= 65) = 0 THEN GOTO 1440
1420         IF (lcaseC0% <= 90) = 0 THEN GOTO 1440
1430             lcaseC0% = lcaseC0% + 32
1440         REM END IF
1450         lcaseOut0$ = lcaseOut0$ + CHR$(lcaseC0%)
1460     NEXT lcaseI0%
1470     lcaseResult0$ = lcaseOut0$
1480     RETURN
1490 ' end function lcase$

1500 ' function error$(code%)
1510     BCCT6% = errorCode0%
1520     IF (BCCT6% = 2) <> 0 THEN GOTO 1860
1530     IF (BCCT6% = 3) <> 0 THEN GOTO 1890
1540     IF (BCCT6% = 4) <> 0 THEN GOTO 1920
1550     IF (BCCT6% = 5) <> 0 THEN GOTO 1950
1560     IF (BCCT6% = 6) <> 0 THEN GOTO 1980
1570     IF (BCCT6% = 7) <> 0 THEN GOTO 2010
1580     IF (BCCT6% = 9) <> 0 THEN GOTO 2040
1590     IF (BCCT6% = 10) <> 0 THEN GOTO 2070
1600     IF (BCCT6% = 11) <> 0 THEN GOTO 2100
1610     IF (BCCT6% = 13) <> 0 THEN GOTO 2130
1620     IF (BCCT6% = 14) <> 0 THEN GOTO 2160
1630     IF (BCCT6% = 19) <> 0 THEN GOTO 2190
1640     IF (BCCT6% = 20) <> 0 THEN GOTO 2220
1650     IF (BCCT6% = 24) <> 0 THEN GOTO 2250
1660     IF (BCCT6% = 25) <> 0 THEN GOTO 2280
1670     IF (BCCT6% = 27) <> 0 THEN GOTO 2310
1680     IF (BCCT6% = 52) <> 0 THEN GOTO 2340
1690     IF (BCCT6% = 53) <> 0 THEN GOTO 2370
1700     IF (BCCT6% = 54) <> 0 THEN GOTO 2400
1710     IF (BCCT6% = 55) <> 0 THEN GOTO 2430
1720     IF (BCCT6% = 57) <> 0 THEN GOTO 2460
1730     IF (BCCT6% = 58) <> 0 THEN GOTO 2490
1740     IF (BCCT6% = 61) <> 0 THEN GOTO 2520
1750     IF (BCCT6% = 62) <> 0 THEN GOTO 2550
1760     IF (BCCT6% = 63) <> 0 THEN GOTO 2580
1770     IF (BCCT6% = 64) <> 0 THEN GOTO 2610
1780     IF (BCCT6% = 67) <> 0 THEN GOTO 2640
1790     IF (BCCT6% = 68) <> 0 THEN GOTO 2670
1800     IF (BCCT6% = 70) <> 0 THEN GOTO 2700
1810     IF (BCCT6% = 71) <> 0 THEN GOTO 2730
1820     IF (BCCT6% = 72) <> 0 THEN GOTO 2760
1830     IF (BCCT6% = 75) <> 0 THEN GOTO 2790
1840     IF (BCCT6% = 76) <> 0 THEN GOTO 2820
1850     GOTO 2850
1860         errorResult0$ = "Syntax error"
1870         RETURN
1880         GOTO 2870
1890         errorResult0$ = "RETURN without GOSUB"
1900         RETURN
1910         GOTO 2870
1920         errorResult0$ = "Out of DATA"
1930         RETURN
1940         GOTO 2870
1950         errorResult0$ = "Illegal function call"
1960         RETURN
1970         GOTO 2870
1980         errorResult0$ = "Overflow"
1990         RETURN
2000         GOTO 2870
2010         errorResult0$ = "Out of memory"
2020         RETURN
2030         GOTO 2870
2040         errorResult0$ = "Subscript out of range"
2050         RETURN
2060         GOTO 2870
2070         errorResult0$ = "Duplicate Definition"
2080         RETURN
2090         GOTO 2870
2100         errorResult0$ = "Division by zero"
2110         RETURN
2120         GOTO 2870
2130         errorResult0$ = "Type mismatch"
2140         RETURN
2150         GOTO 2870
2160         errorResult0$ = "Out of string space"
2170         RETURN
2180         GOTO 2870
2190         errorResult0$ = "No RESUME"
2200         RETURN
2210         GOTO 2870
2220         errorResult0$ = "RESUME without error"
2230         RETURN
2240         GOTO 2870
2250         errorResult0$ = "Device timeout"
2260         RETURN
2270         GOTO 2870
2280         errorResult0$ = "Device fault"
2290         RETURN
2300         GOTO 2870
2310         errorResult0$ = "Out of paper"
2320         RETURN
2330         GOTO 2870
2340         errorResult0$ = "Bad file number"
2350         RETURN
2360         GOTO 2870
2370         errorResult0$ = "File not found"
2380         RETURN
2390         GOTO 2870
2400         errorResult0$ = "Bad file mode"
2410         RETURN
2420         GOTO 2870
2430         errorResult0$ = "File already open"
2440         RETURN
2450         GOTO 2870
2460         errorResult0$ = "Device I/O error"
2470         RETURN
2480         GOTO 2870
2490         errorResult0$ = "File already exists"
2500         RETURN
2510         GOTO 2870
2520         errorResult0$ = "Disk full"
2530         RETURN
2540         GOTO 2870
2550         errorResult0$ = "Input past end"
2560         RETURN
2570         GOTO 2870
2580         errorResult0$ = "Bad record number"
2590         RETURN
2600         GOTO 2870
2610         errorResult0$ = "Bad file name"
2620         RETURN
2630         GOTO 2870
2640         errorResult0$ = "Too many files"
2650         RETURN
2660         GOTO 2870
2670         errorResult0$ = "Device unavailable"
2680         RETURN
2690         GOTO 2870
2700         errorResult0$ = "Disk write protected"
2710         RETURN
2720         GOTO 2870
2730         errorResult0$ = "Disk not ready"
2740         RETURN
2750         GOTO 2870
2760         errorResult0$ = "Disk media error"
2770         RETURN
2780         GOTO 2870
2790         errorResult0$ = "Path/File access error"
2800         RETURN
2810         GOTO 2870
2820         errorResult0$ = "Path not found"
2830         RETURN
2840         GOTO 2870
2850         errorResult0$ = "Error " + STR$(errorCode0%)
2860         RETURN
2870     REM END SELECT
2880     RETURN
2890 ' end function error$

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/18_stdlib.c</code></summary>



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

void bf_s_ltrim_s(const char* bv_s_self_in, char* bcc_out);
void bf_s_rtrim_s(const char* bv_s_self_in, char* bcc_out);
void bf_s_ucase_s(const char* bv_s_self_in, char* bcc_out);
void bf_s_lcase_s(const char* bv_s_self_in, char* bcc_out);
void bf_s_error(int bv_i_code, char* bcc_out);

void bf_s_ltrim_s(const char* bv_s_self_in, char* bcc_out) {
    char bv_s_self[256];
    snprintf(bv_s_self, sizeof(bv_s_self), "%s", bv_s_self_in);
    int bv_i_i = 0;

    bv_i_i = 1;
    while (((-(bv_i_i <= ((int)strlen(bv_s_self)))) && (-(strcmp(bcc_mid(bv_s_self, bv_i_i, 1), " ") == 0)))) {
        bv_i_i = (bv_i_i + 1);
    }
    snprintf(bcc_out, 256, "%s", bcc_mid(bv_s_self, bv_i_i, 2147483647));
    return;
}

void bf_s_rtrim_s(const char* bv_s_self_in, char* bcc_out) {
    char bv_s_self[256];
    snprintf(bv_s_self, sizeof(bv_s_self), "%s", bv_s_self_in);
    int bv_i_i = 0;

    bv_i_i = ((int)strlen(bv_s_self));
    while (((-(bv_i_i > 0)) && (-(strcmp(bcc_mid(bv_s_self, bv_i_i, 1), " ") == 0)))) {
        bv_i_i = (bv_i_i - 1);
    }
    snprintf(bcc_out, 256, "%s", bcc_mid(bv_s_self, 1, bv_i_i));
    return;
}

void bf_s_ucase_s(const char* bv_s_self_in, char* bcc_out) {
    char bv_s_self[256];
    snprintf(bv_s_self, sizeof(bv_s_self), "%s", bv_s_self_in);
    int bv_i_c = 0;
    int bv_i_i = 0;
    char bv_s_out[256] = {0};

    snprintf(bv_s_out, sizeof(bv_s_out), "%s", "");
    int bt_lim_0 = ((int)strlen(bv_s_self));
    int bt_step_0 = 1;
    for (bv_i_i = 1; bt_step_0 >= 0 ? bv_i_i <= bt_lim_0 : bv_i_i >= bt_lim_0; bv_i_i += bt_step_0) {
        bv_i_c = ((int)(unsigned char)bcc_mid(bv_s_self, bv_i_i, 1)[0]);
        if (((-(bv_i_c >= 97)) && (-(bv_i_c <= 122)))) {
            bv_i_c = (bv_i_c - 32);
        }
        char bt_s_1[256];
        snprintf(bt_s_1, sizeof(bt_s_1), "%s%s", bv_s_out, bcc_chr(bv_i_c));
        snprintf(bv_s_out, sizeof(bv_s_out), "%s", bt_s_1);
    }
    snprintf(bcc_out, 256, "%s", bv_s_out);
    return;
}

void bf_s_lcase_s(const char* bv_s_self_in, char* bcc_out) {
    char bv_s_self[256];
    snprintf(bv_s_self, sizeof(bv_s_self), "%s", bv_s_self_in);
    int bv_i_c = 0;
    int bv_i_i = 0;
    char bv_s_out[256] = {0};

    snprintf(bv_s_out, sizeof(bv_s_out), "%s", "");
    int bt_lim_2 = ((int)strlen(bv_s_self));
    int bt_step_2 = 1;
    for (bv_i_i = 1; bt_step_2 >= 0 ? bv_i_i <= bt_lim_2 : bv_i_i >= bt_lim_2; bv_i_i += bt_step_2) {
        bv_i_c = ((int)(unsigned char)bcc_mid(bv_s_self, bv_i_i, 1)[0]);
        if (((-(bv_i_c >= 65)) && (-(bv_i_c <= 90)))) {
            bv_i_c = (bv_i_c + 32);
        }
        char bt_s_3[256];
        snprintf(bt_s_3, sizeof(bt_s_3), "%s%s", bv_s_out, bcc_chr(bv_i_c));
        snprintf(bv_s_out, sizeof(bv_s_out), "%s", bt_s_3);
    }
    snprintf(bcc_out, 256, "%s", bv_s_out);
    return;
}

void bf_s_error(int bv_i_code, char* bcc_out) {
    {
        int bt_sel_4 = bv_i_code;
        int bt_sel_match_5 = 0;
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == 2)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Syntax error");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == 3)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "RETURN without GOSUB");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == 4)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Out of DATA");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == 5)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Illegal function call");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == 6)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Overflow");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == 7)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Out of memory");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == 9)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Subscript out of range");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == 10)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Duplicate Definition");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == 11)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Division by zero");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == 13)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Type mismatch");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == 14)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Out of string space");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == 19)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "No RESUME");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == 20)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "RESUME without error");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == 24)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Device timeout");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == 25)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Device fault");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == 27)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Out of paper");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == 52)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Bad file number");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == 53)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "File not found");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == 54)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Bad file mode");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == 55)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "File already open");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == 57)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Device I/O error");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == 58)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "File already exists");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == 61)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Disk full");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == 62)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Input past end");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == 63)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Bad record number");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == 64)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Bad file name");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == 67)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Too many files");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == 68)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Device unavailable");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == 70)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Disk write protected");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == 71)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Disk not ready");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == 72)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Disk media error");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == 75)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Path/File access error");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == 76)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Path not found");
                return;
            }
        }
        if (!bt_sel_match_5) {
            char bt_s_6[256];
            snprintf(bt_s_6, sizeof(bt_s_6), "%s%s", "Error ", bcc_stri(bv_i_code));
            snprintf(bcc_out, 256, "%s", bt_s_6);
            return;
        }
    }
}

int main(void) {
    // Strips leading spaces from self$. Not a real MBASIC/BASCOM 2.00 builtin --
    // verified against a real IBM BASIC Compiler 2.00 under dosbox-x -- so
    // BASCAL ships its own. Declared as a scalar method (see GitHub issue #41)
    // so a required stdlib call reads the same way as a built-in method call
    // (docs/language/functions-and-procedures.html#built-in-methods). The
    // ordinary call form (ltrim$(s$)) still works -- a method's receiver is an
    // implicit first parameter, so ordinary-call syntax resolves straight to
    // this same declaration, with no separate function needed (and no longer
    // allowed: a function and a method sharing one name is a duplicate
    // declaration, since they'd both claim the same callable identity).

    // Strips trailing spaces from self$. Not a real MBASIC/BASCOM 2.00 builtin --
    // verified against a real IBM BASIC Compiler 2.00 under dosbox-x -- so
    // BASCAL ships its own. Declared as a scalar method (see GitHub issue #41
    // and ltrim.bcl's own doc comment for the reasoning) -- rtrim$(s$) still
    // works via ordinary-call syntax resolving to this same declaration.

    // Upper-cases self$. Not a real MBASIC/BASCOM 2.00 builtin -- verified
    // against a real IBM BASIC Compiler 2.00 under dosbox-x -- so BASCAL ships
    // its own. Declared as a scalar method (see GitHub issue #41 and
    // ltrim.bcl's own doc comment for the reasoning) -- ucase$(s$) still works
    // via ordinary-call syntax resolving to this same declaration.

    // Lower-cases self$. Not a real MBASIC/BASCOM 2.00 builtin -- verified
    // against a real IBM BASIC Compiler 2.00 under dosbox-x -- so BASCAL ships
    // its own. Declared as a scalar method (see GitHub issue #41 and
    // ltrim.bcl's own doc comment for the reasoning) -- lcase$(s$) still works
    // via ordinary-call syntax resolving to this same declaration.

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

    // Tutorial — Standard library functions
    //
    // com.bascal.stdlib is an ordinary require-able library, resolved the same
    // way as com.bascal.sort in tutorial 12 -- but bcc always adds its home
    // directory to the search path automatically, so no -L flag is needed to
    // reach it. It exists because LTRIM$, RTRIM$, UCASE$, LCASE$, and ERROR$
    // either aren't real MBASIC/BASCOM 2.00 builtins or don't work at runtime
    // (verified against a real IBM Personal Computer BASIC Compiler 2.00 under
    // dosbox-x) -- see the manual's "String and error-message functions"
    // section (https://johnjoeallen.github.io/bascal/manual/) for the full
    // story.
    //
    // ltrim$/rtrim$/ucase$/lcase$ are declared as scalar methods (method$ ...
    // end method), using self$ in place of an explicit s$ parameter -- see
    // the "Declare and call a method" chapter. A method's receiver is really
    // just an implicit first parameter, so the ordinary call form below
    // (ltrim$("...")) keeps working exactly as before: it resolves straight to
    // the same method declaration, with the first argument filling self$. The
    // method-call form (below, chained) is the same declaration too -- just
    // written as "...".ltrim() instead. error$ stays an ordinary function: an
    // error code is a lookup key, not a value the call is naturally "operating
    // on" the way the others operate on their string.
    //
    // Run with:
    // bcc tutorial/18_stdlib.bcl


    char bt_s_7[256];
    bf_s_ltrim_s("   padded left", bt_s_7);
    char bt_s_8[256];
    snprintf(bt_s_8, sizeof(bt_s_8), "%s%s", "[", bt_s_7);
    char bt_s_9[256];
    snprintf(bt_s_9, sizeof(bt_s_9), "%s%s", bt_s_8, "]");
    printf("%s\n", bt_s_9);
    char bt_s_10[256];
    bf_s_rtrim_s("padded right   ", bt_s_10);
    char bt_s_11[256];
    snprintf(bt_s_11, sizeof(bt_s_11), "%s%s", "[", bt_s_10);
    char bt_s_12[256];
    snprintf(bt_s_12, sizeof(bt_s_12), "%s%s", bt_s_11, "]");
    printf("%s\n", bt_s_12);
    char bt_s_13[256];
    bf_s_ucase_s("shout this", bt_s_13);
    printf("%s\n", bt_s_13);
    char bt_s_14[256];
    bf_s_lcase_s("QUIET THIS DOWN", bt_s_14);
    printf("%s\n", bt_s_14);

    // Same four functions, called as chained methods instead.
    char bt_s_15[256];
    bf_s_ltrim_s("  padded both sides  ", bt_s_15);
    char bt_s_16[256];
    bf_s_rtrim_s(bt_s_15, bt_s_16);
    char bt_s_17[256];
    snprintf(bt_s_17, sizeof(bt_s_17), "%s%s", "[", bt_s_16);
    char bt_s_18[256];
    snprintf(bt_s_18, sizeof(bt_s_18), "%s%s", bt_s_17, "]");
    printf("%s\n", bt_s_18);
    char bt_s_19[256];
    bf_s_ltrim_s("  shout this too", bt_s_19);
    char bt_s_20[256];
    bf_s_ucase_s(bt_s_19, bt_s_20);
    printf("%s\n", bt_s_20);

    // ERROR$ maps a classic MBASIC/GW-BASIC/BASCOM error code to a message;
    // pair it with ERR inside an ON ERROR GOTO handler in real code.
    char bt_s_21[256];
    bf_s_error(53, bt_s_21);
    printf("%s\n", bt_s_21);
    char bt_s_22[256];
    bf_s_error(11, bt_s_22);
    printf("%s\n", bt_s_22);
    char bt_s_23[256];
    bf_s_error(9999, bt_s_23);
    printf("%s\n", bt_s_23);

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

<summary><code>tutorial/18_stdlib.j</code></summary>



```basic

.version 50 0
.class public Stdlib
.super java/lang/Object

.method public static ltrim : (Ljava/lang/String;)Ljava/lang/String;
    .limit stack 16
    .limit locals 2

    iconst_0
    istore 1
    ldc 1
    istore 1
L_while_0_top:
    iload 1
    aload 0
    invokevirtual java/lang/String/length ()I
    invokestatic java/lang/Integer/compare (II)I
    iconst_1
    isub
    bipush 31
    ishr
    ifeq L_while_0_end
    aload 0
    iload 1
    iconst_1
    isub
    dup
    ldc 1
    iadd
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    ldc " "
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ineg
    ifeq L_while_0_end
    iload 1
    ldc 1
    iadd
    istore 1
    goto L_while_0_top
L_while_0_end:
    aload 0
    iload 1
    iconst_1
    isub
    invokevirtual java/lang/String/substring (I)Ljava/lang/String;
    areturn
    ldc ""
    areturn
.end method

.method public static rtrim : (Ljava/lang/String;)Ljava/lang/String;
    .limit stack 16
    .limit locals 2

    iconst_0
    istore 1
    aload 0
    invokevirtual java/lang/String/length ()I
    istore 1
L_while_0_top:
    iload 1
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_while_0_end
    aload 0
    iload 1
    iconst_1
    isub
    dup
    ldc 1
    iadd
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    ldc " "
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ineg
    ifeq L_while_0_end
    iload 1
    ldc 1
    isub
    istore 1
    goto L_while_0_top
L_while_0_end:
    aload 0
    iconst_0
    iload 1
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    areturn
    ldc ""
    areturn
.end method

.method public static ucase : (Ljava/lang/String;)Ljava/lang/String;
    .limit stack 16
    .limit locals 4

    iconst_0
    istore 1
    iconst_0
    istore 2
    ldc ""
    astore 3
    ldc ""
    astore 3
    ldc 1
    istore 2
L_for_0_top:
    iload 2
    aload 0
    invokevirtual java/lang/String/length ()I
    if_icmpgt L_for_0_end
    aload 0
    iload 2
    iconst_1
    isub
    dup
    ldc 1
    iadd
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    iconst_0
    invokevirtual java/lang/String/charAt (I)C
    istore 1
    iload 1
    ldc 97
    invokestatic java/lang/Integer/compare (II)I
    ineg
    iconst_1
    isub
    bipush 31
    ishr
    ifeq L_if_1_else
    iload 1
    ldc 122
    invokestatic java/lang/Integer/compare (II)I
    iconst_1
    isub
    bipush 31
    ishr
    ifeq L_if_1_else
    iload 1
    ldc 32
    isub
    istore 1
L_if_1_else:
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    aload 3
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    iload 1
    i2c
    invokestatic java/lang/String/valueOf (C)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    astore 3
    iload 2
    ldc 1
    iadd
    istore 2
    goto L_for_0_top
L_for_0_end:
    aload 3
    areturn
    ldc ""
    areturn
.end method

.method public static lcase : (Ljava/lang/String;)Ljava/lang/String;
    .limit stack 16
    .limit locals 4

    iconst_0
    istore 1
    iconst_0
    istore 2
    ldc ""
    astore 3
    ldc ""
    astore 3
    ldc 1
    istore 2
L_for_0_top:
    iload 2
    aload 0
    invokevirtual java/lang/String/length ()I
    if_icmpgt L_for_0_end
    aload 0
    iload 2
    iconst_1
    isub
    dup
    ldc 1
    iadd
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    iconst_0
    invokevirtual java/lang/String/charAt (I)C
    istore 1
    iload 1
    ldc 65
    invokestatic java/lang/Integer/compare (II)I
    ineg
    iconst_1
    isub
    bipush 31
    ishr
    ifeq L_if_1_else
    iload 1
    ldc 90
    invokestatic java/lang/Integer/compare (II)I
    iconst_1
    isub
    bipush 31
    ishr
    ifeq L_if_1_else
    iload 1
    ldc 32
    iadd
    istore 1
L_if_1_else:
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    aload 3
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    iload 1
    i2c
    invokestatic java/lang/String/valueOf (C)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    astore 3
    iload 2
    ldc 1
    iadd
    istore 2
    goto L_for_0_top
L_for_0_end:
    aload 3
    areturn
    ldc ""
    areturn
.end method

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
    .limit locals 1

    ; Strips leading spaces from self$. Not a real MBASIC/BASCOM 2.00 builtin --
    ; verified against a real IBM BASIC Compiler 2.00 under dosbox-x -- so
    ; BASCAL ships its own. Declared as a scalar method (see GitHub issue #41)
    ; so a required stdlib call reads the same way as a built-in method call
    ; (docs/language/functions-and-procedures.html#built-in-methods). The
    ; ordinary call form (ltrim$(s$)) still works -- a method's receiver is an
    ; implicit first parameter, so ordinary-call syntax resolves straight to
    ; this same declaration, with no separate function needed (and no longer
    ; allowed: a function and a method sharing one name is a duplicate
    ; declaration, since they'd both claim the same callable identity).

    ; Strips trailing spaces from self$. Not a real MBASIC/BASCOM 2.00 builtin --
    ; verified against a real IBM BASIC Compiler 2.00 under dosbox-x -- so
    ; BASCAL ships its own. Declared as a scalar method (see GitHub issue #41
    ; and ltrim.bcl's own doc comment for the reasoning) -- rtrim$(s$) still
    ; works via ordinary-call syntax resolving to this same declaration.

    ; Upper-cases self$. Not a real MBASIC/BASCOM 2.00 builtin -- verified
    ; against a real IBM BASIC Compiler 2.00 under dosbox-x -- so BASCAL ships
    ; its own. Declared as a scalar method (see GitHub issue #41 and
    ; ltrim.bcl's own doc comment for the reasoning) -- ucase$(s$) still works
    ; via ordinary-call syntax resolving to this same declaration.

    ; Lower-cases self$. Not a real MBASIC/BASCOM 2.00 builtin -- verified
    ; against a real IBM BASIC Compiler 2.00 under dosbox-x -- so BASCAL ships
    ; its own. Declared as a scalar method (see GitHub issue #41 and
    ; ltrim.bcl's own doc comment for the reasoning) -- lcase$(s$) still works
    ; via ordinary-call syntax resolving to this same declaration.

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


    ; Tutorial — Standard library functions
    ;
    ; com.bascal.stdlib is an ordinary require-able library, resolved the same
    ; way as com.bascal.sort in tutorial 12 -- but bcc always adds its home
    ; directory to the search path automatically, so no -L flag is needed to
    ; reach it. It exists because LTRIM$, RTRIM$, UCASE$, LCASE$, and ERROR$
    ; either aren't real MBASIC/BASCOM 2.00 builtins or don't work at runtime
    ; (verified against a real IBM Personal Computer BASIC Compiler 2.00 under
    ; dosbox-x) -- see the manual's "String and error-message functions"
    ; section (https://johnjoeallen.github.io/bascal/manual/) for the full
    ; story.
    ;
    ; ltrim$/rtrim$/ucase$/lcase$ are declared as scalar methods (method$ ...
    ; end method), using self$ in place of an explicit s$ parameter -- see
    ; the "Declare and call a method" chapter. A method's receiver is really
    ; just an implicit first parameter, so the ordinary call form below
    ; (ltrim$("...")) keeps working exactly as before: it resolves straight to
    ; the same method declaration, with the first argument filling self$. The
    ; method-call form (below, chained) is the same declaration too -- just
    ; written as "...".ltrim() instead. error$ stays an ordinary function: an
    ; error code is a lookup key, not a value the call is naturally "operating
    ; on" the way the others operate on their string.
    ;
    ; Run with:
    ; bcc tutorial/18_stdlib.bcl


    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc "   padded left"
    invokestatic Stdlib/ltrim (Ljava/lang/String;)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc "]"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc "padded right   "
    invokestatic Stdlib/rtrim (Ljava/lang/String;)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc "]"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "shout this"
    invokestatic Stdlib/ucase (Ljava/lang/String;)Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "QUIET THIS DOWN"
    invokestatic Stdlib/lcase (Ljava/lang/String;)Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V

    ; Same four functions, called as chained methods instead.
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc "  padded both sides  "
    invokestatic Stdlib/ltrim (Ljava/lang/String;)Ljava/lang/String;
    invokestatic Stdlib/rtrim (Ljava/lang/String;)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc "]"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  shout this too"
    invokestatic Stdlib/ltrim (Ljava/lang/String;)Ljava/lang/String;
    invokestatic Stdlib/ucase (Ljava/lang/String;)Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V

    ; ERROR$ maps a classic MBASIC/GW-BASIC/BASCOM error code to a message;
    ; pair it with ERR inside an ON ERROR GOTO handler in real code.
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc 53
    invokestatic Stdlib/error (I)Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc 11
    invokestatic Stdlib/error (I)Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc 9999
    invokestatic Stdlib/error (I)Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V

    return
.end method

```



</details>

<!-- END generated tutorial source -->
