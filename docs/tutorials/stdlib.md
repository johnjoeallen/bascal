[Home](../../) / [Tutorials](../) / Standard Library Functions

<div class="prose" markdown="1">

Generated sources: [BCL](https://github.com/johnjoeallen/bascal/blob/main/tutorial/stdlib.bcl), [BASIC](https://github.com/johnjoeallen/bascal/blob/main/tutorial/stdlib.bas), [C](https://github.com/johnjoeallen/bascal/blob/main/tutorial/stdlib.c), and [JVM assembly](https://github.com/johnjoeallen/bascal/blob/main/tutorial/stdlib.j).

`LTRIM$`, `RTRIM$`, `UCASE$`, `LCASE$`, and `ERROR$` either aren't real MBASIC/BASCOM 2.00 builtins at all, or (in `ERROR$`'s case) compile and link but silently return an empty string at runtime -- verified against a real IBM Personal Computer BASIC Compiler 2.00 running under dosbox-x. BASCAL ships its own implementations under `com.bascal.stdlib`, resolved exactly like `com.bascal.sort` in [tutorial 12](require.md) -- but `bcc` always adds the library's install location to its search path automatically (next to the binary, or `usr/share/bascal` for a `.deb`/`.rpm` install), so no `-L` flag is needed to reach it.

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

Pair it with ERR inside an ON ERROR GOTO handler in real code -- see [classic tutorial 17](labels_and_error_handling.md), or use [portable structured handling](portable_error_handling.md).

```bascal
print error$(53)   ' File not found
print error$(11)   ' Division by zero
print error$(9999) ' Error  9999 (falls through to STR$)
```

</div>



[← Portable Error Handling](portable_error_handling.md)  ·  [Case Study: Random-Access Inventory →](inventory.md)


<!-- BEGIN generated tutorial source -->

<details class="source-embed" markdown="1">

<summary><code>tutorial/stdlib.bcl</code></summary>



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
//   bcc tutorial/stdlib.bcl
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

<summary><code>tutorial/stdlib.bas</code></summary>



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
330 ' The named constants below are the complete common subset supported by
340 ' ERROR$: use them in THROW and filtered CATCH clauses instead of magic
350 ' numbers.  Dialect-specific errors outside this shared MBASIC/GW-BASIC/
360 ' BASCOM subset still fall through to ERROR$'s generic message.
370 '
380 ' Deliberately NOT a scalar method (see GitHub issue #41, which asked for
390 ' this decision to be recorded either way): code% is an opaque lookup key,
400 ' not a value the call is naturally "operating on" the way ltrim$/rtrim$/
410 ' ucase$/lcase$ operate on their string -- code%.error() would read as if
420 ' the *error code itself* has a message, when really this is a lookup
430 ' table keyed by that code. Stays an ordinary function.
440 CONSTERRSYNTAX% = 2
450 CONSTERRRETURNWITHOUTGOSUB% = 3
460 CONSTERROUTOFDATA% = 4
470 CONSTERRILLEGALFUNCTIONCALL% = 5
480 CONSTERROVERFLOW% = 6
490 CONSTERROUTOFMEMORY% = 7
500 CONSTERRSUBSCRIPTOUTOFRANGE% = 9
510 CONSTERRDUPLICATEDEFINITION% = 10
520 CONSTERRDIVISIONBYZERO% = 11
530 CONSTERRTYPEMISMATCH% = 13
540 CONSTERROUTOFSTRINGSPACE% = 14
550 CONSTERRNORESUME% = 19
560 CONSTERRRESUMEWITHOUTERROR% = 20
570 CONSTERRDEVICETIMEOUT% = 24
580 CONSTERRDEVICEFAULT% = 25
590 CONSTERROUTOFPAPER% = 27
600 CONSTERRBADFILENUMBER% = 52
610 CONSTERRFILENOTFOUND% = 53
620 CONSTERRBADFILEMODE% = 54
630 CONSTERRFILEALREADYOPEN% = 55
640 CONSTERRDEVICEIO% = 57
650 CONSTERRFILEALREADYEXISTS% = 58
660 CONSTERRDISKFULL% = 61
670 CONSTERRINPUTPASTEND% = 62
680 CONSTERRBADRECORDNUMBER% = 63
690 CONSTERRBADFILENAME% = 64
700 CONSTERRTOOMANYFILES% = 67
710 CONSTERRDEVICEUNAVAILABLE% = 68
720 CONSTERRDISKWRITEPROTECTED% = 70
730 CONSTERRDISKNOTREADY% = 71
740 CONSTERRDISKMEDIAERROR% = 72
750 CONSTERRPATHFILEACCESS% = 75
760 CONSTERRPATHNOTFOUND% = 76
770 ' Tutorial — Standard library functions
780 '
790 ' com.bascal.stdlib is an ordinary require-able library, resolved the same
800 ' way as com.bascal.sort in tutorial 12 -- but bcc always adds its home
810 ' directory to the search path automatically, so no -L flag is needed to
820 ' reach it. It exists because LTRIM$, RTRIM$, UCASE$, LCASE$, and ERROR$
830 ' either aren't real MBASIC/BASCOM 2.00 builtins or don't work at runtime
840 ' (verified against a real IBM Personal Computer BASIC Compiler 2.00 under
850 ' dosbox-x) -- see the manual's "String and error-message functions"
860 ' section (https://johnjoeallen.github.io/bascal/manual/) for the full
870 ' story.
880 '
890 ' ltrim$/rtrim$/ucase$/lcase$ are scalar methods with a bracketed string
900 ' receiver type, using self$ in place of an explicit s$ parameter -- see
910 ' the "Declare and call a method" chapter. A method's receiver is really
920 ' just an implicit first parameter, so the ordinary call form
930 ' (ltrim$("...")) keeps working exactly as before: it resolves straight to
940 ' the same method declaration, with the first argument filling self$. The
950 ' examples below prefer the method-call form, written as "...".ltrim().
960 ' error$ stays an ordinary function: an
970 ' error code is a lookup key, not a value the call is naturally "operating
980 ' on" the way the others operate on their string.
990 '
1000 ' Run with:
1010 ' bcc tutorial/stdlib.bcl
1020 ltrimSelf0$ = "   padded left"
1030 GOSUB 1460
1040 BCCT1$ = ltrimResult0$
1050 PRINT ("[" + BCCT1$) + "]"
1060 rtrimSelf0$ = "padded right   "
1070 GOSUB 1590
1080 BCCT2$ = rtrimResult0$
1090 PRINT ("[" + BCCT2$) + "]"
1100 ucaseSelf0$ = "shout this"
1110 GOSUB 1720
1120 BCCT3$ = ucaseResult0$
1130 PRINT BCCT3$
1140 lcaseSelf0$ = "QUIET THIS DOWN"
1150 GOSUB 1850
1160 BCCT4$ = lcaseResult0$
1170 PRINT BCCT4$
1180 ' Same four functions, called as chained methods instead.
1190 ltrimSelf0$ = "  padded both sides  "
1200 GOSUB 1460
1210 BCCT5$ = ltrimResult0$
1220 rtrimSelf0$ = BCCT5$
1230 GOSUB 1590
1240 BCCT6$ = rtrimResult0$
1250 PRINT ("[" + BCCT6$) + "]"
1260 ltrimSelf0$ = "  shout this too"
1270 GOSUB 1460
1280 BCCT7$ = ltrimResult0$
1290 ucaseSelf0$ = BCCT7$
1300 GOSUB 1720
1310 BCCT8$ = ucaseResult0$
1320 PRINT BCCT8$
1330 ' ERROR$ maps a classic MBASIC/GW-BASIC/BASCOM error code to a message;
1340 ' pair it with ERR inside an ON ERROR GOTO handler in real code.
1350 errorCode0% = 53
1360 GOSUB 1980
1370 PRINT errorResult0$
1380 errorCode0% = 11
1390 GOSUB 1980
1400 PRINT errorResult0$
1410 errorCode0% = 9999
1420 GOSUB 1980
1430 PRINT errorResult0$
1440 END

1450 ' function ltrim$(self$)
1460     ltrimI0% = 1
1470     IF (ltrimI0% <= LEN(ltrimSelf0$)) = 0 THEN GOTO 1510
1480     IF (MID$(ltrimSelf0$, ltrimI0%, 1) = " ") = 0 THEN GOTO 1510
1490         ltrimI0% = ltrimI0% + 1
1500     GOTO 1470
1510     REM END WHILE
1520     BCCT11$ = ltrimSelf0$
1530     BCCT12% = ltrimI0%
1540     BCCT13$ = MID$(BCCT11$, BCCT12%)
1550     ltrimResult0$ = BCCT13$
1560     RETURN
1570 ' end function ltrim$

1580 ' function rtrim$(self$)
1590     rtrimI0% = LEN(rtrimSelf0$)
1600     IF (rtrimI0% > 0) = 0 THEN GOTO 1640
1610     IF (MID$(rtrimSelf0$, rtrimI0%, 1) = " ") = 0 THEN GOTO 1640
1620         rtrimI0% = rtrimI0% - 1
1630     GOTO 1600
1640     REM END WHILE
1650     BCCT16$ = rtrimSelf0$
1660     BCCT17% = rtrimI0%
1670     BCCT18$ = LEFT$(BCCT16$, BCCT17%)
1680     rtrimResult0$ = BCCT18$
1690     RETURN
1700 ' end function rtrim$

1710 ' function ucase$(self$)
1720     ucaseOut0$ = ""
1730     FOR ucaseI0% = 1 TO LEN(ucaseSelf0$)
1740         ucaseC0% = ASC(MID$(ucaseSelf0$, ucaseI0%, 1))
1750         IF (ucaseC0% >= 97) = 0 THEN GOTO 1780
1760         IF (ucaseC0% <= 122) = 0 THEN GOTO 1780
1770             ucaseC0% = ucaseC0% - 32
1780         REM END IF
1790         ucaseOut0$ = ucaseOut0$ + CHR$(ucaseC0%)
1800     NEXT ucaseI0%
1810     ucaseResult0$ = ucaseOut0$
1820     RETURN
1830 ' end function ucase$

1840 ' function lcase$(self$)
1850     lcaseOut0$ = ""
1860     FOR lcaseI0% = 1 TO LEN(lcaseSelf0$)
1870         lcaseC0% = ASC(MID$(lcaseSelf0$, lcaseI0%, 1))
1880         IF (lcaseC0% >= 65) = 0 THEN GOTO 1910
1890         IF (lcaseC0% <= 90) = 0 THEN GOTO 1910
1900             lcaseC0% = lcaseC0% + 32
1910         REM END IF
1920         lcaseOut0$ = lcaseOut0$ + CHR$(lcaseC0%)
1930     NEXT lcaseI0%
1940     lcaseResult0$ = lcaseOut0$
1950     RETURN
1960 ' end function lcase$

1970 ' function error$(code%)
1980     BCCT26% = errorCode0%
1990     IF (BCCT26% = CONSTERRSYNTAX%) <> 0 THEN GOTO 2330
2000     IF (BCCT26% = CONSTERRRETURNWITHOUTGOSUB%) <> 0 THEN GOTO 2360
2010     IF (BCCT26% = CONSTERROUTOFDATA%) <> 0 THEN GOTO 2390
2020     IF (BCCT26% = CONSTERRILLEGALFUNCTIONCALL%) <> 0 THEN GOTO 2420
2030     IF (BCCT26% = CONSTERROVERFLOW%) <> 0 THEN GOTO 2450
2040     IF (BCCT26% = CONSTERROUTOFMEMORY%) <> 0 THEN GOTO 2480
2050     IF (BCCT26% = CONSTERRSUBSCRIPTOUTOFRANGE%) <> 0 THEN GOTO 2510
2060     IF (BCCT26% = CONSTERRDUPLICATEDEFINITION%) <> 0 THEN GOTO 2540
2070     IF (BCCT26% = CONSTERRDIVISIONBYZERO%) <> 0 THEN GOTO 2570
2080     IF (BCCT26% = CONSTERRTYPEMISMATCH%) <> 0 THEN GOTO 2600
2090     IF (BCCT26% = CONSTERROUTOFSTRINGSPACE%) <> 0 THEN GOTO 2630
2100     IF (BCCT26% = CONSTERRNORESUME%) <> 0 THEN GOTO 2660
2110     IF (BCCT26% = CONSTERRRESUMEWITHOUTERROR%) <> 0 THEN GOTO 2690
2120     IF (BCCT26% = CONSTERRDEVICETIMEOUT%) <> 0 THEN GOTO 2720
2130     IF (BCCT26% = CONSTERRDEVICEFAULT%) <> 0 THEN GOTO 2750
2140     IF (BCCT26% = CONSTERROUTOFPAPER%) <> 0 THEN GOTO 2780
2150     IF (BCCT26% = CONSTERRBADFILENUMBER%) <> 0 THEN GOTO 2810
2160     IF (BCCT26% = CONSTERRFILENOTFOUND%) <> 0 THEN GOTO 2840
2170     IF (BCCT26% = CONSTERRBADFILEMODE%) <> 0 THEN GOTO 2870
2180     IF (BCCT26% = CONSTERRFILEALREADYOPEN%) <> 0 THEN GOTO 2900
2190     IF (BCCT26% = CONSTERRDEVICEIO%) <> 0 THEN GOTO 2930
2200     IF (BCCT26% = CONSTERRFILEALREADYEXISTS%) <> 0 THEN GOTO 2960
2210     IF (BCCT26% = CONSTERRDISKFULL%) <> 0 THEN GOTO 2990
2220     IF (BCCT26% = CONSTERRINPUTPASTEND%) <> 0 THEN GOTO 3020
2230     IF (BCCT26% = CONSTERRBADRECORDNUMBER%) <> 0 THEN GOTO 3050
2240     IF (BCCT26% = CONSTERRBADFILENAME%) <> 0 THEN GOTO 3080
2250     IF (BCCT26% = CONSTERRTOOMANYFILES%) <> 0 THEN GOTO 3110
2260     IF (BCCT26% = CONSTERRDEVICEUNAVAILABLE%) <> 0 THEN GOTO 3140
2270     IF (BCCT26% = CONSTERRDISKWRITEPROTECTED%) <> 0 THEN GOTO 3170
2280     IF (BCCT26% = CONSTERRDISKNOTREADY%) <> 0 THEN GOTO 3200
2290     IF (BCCT26% = CONSTERRDISKMEDIAERROR%) <> 0 THEN GOTO 3230
2300     IF (BCCT26% = CONSTERRPATHFILEACCESS%) <> 0 THEN GOTO 3260
2310     IF (BCCT26% = CONSTERRPATHNOTFOUND%) <> 0 THEN GOTO 3290
2320     GOTO 3320
2330         errorResult0$ = "Syntax error"
2340         RETURN
2350         GOTO 3360
2360         errorResult0$ = "RETURN without GOSUB"
2370         RETURN
2380         GOTO 3360
2390         errorResult0$ = "Out of DATA"
2400         RETURN
2410         GOTO 3360
2420         errorResult0$ = "Illegal function call"
2430         RETURN
2440         GOTO 3360
2450         errorResult0$ = "Overflow"
2460         RETURN
2470         GOTO 3360
2480         errorResult0$ = "Out of memory"
2490         RETURN
2500         GOTO 3360
2510         errorResult0$ = "Subscript out of range"
2520         RETURN
2530         GOTO 3360
2540         errorResult0$ = "Duplicate Definition"
2550         RETURN
2560         GOTO 3360
2570         errorResult0$ = "Division by zero"
2580         RETURN
2590         GOTO 3360
2600         errorResult0$ = "Type mismatch"
2610         RETURN
2620         GOTO 3360
2630         errorResult0$ = "Out of string space"
2640         RETURN
2650         GOTO 3360
2660         errorResult0$ = "No RESUME"
2670         RETURN
2680         GOTO 3360
2690         errorResult0$ = "RESUME without error"
2700         RETURN
2710         GOTO 3360
2720         errorResult0$ = "Device timeout"
2730         RETURN
2740         GOTO 3360
2750         errorResult0$ = "Device fault"
2760         RETURN
2770         GOTO 3360
2780         errorResult0$ = "Out of paper"
2790         RETURN
2800         GOTO 3360
2810         errorResult0$ = "Bad file number"
2820         RETURN
2830         GOTO 3360
2840         errorResult0$ = "File not found"
2850         RETURN
2860         GOTO 3360
2870         errorResult0$ = "Bad file mode"
2880         RETURN
2890         GOTO 3360
2900         errorResult0$ = "File already open"
2910         RETURN
2920         GOTO 3360
2930         errorResult0$ = "Device I/O error"
2940         RETURN
2950         GOTO 3360
2960         errorResult0$ = "File already exists"
2970         RETURN
2980         GOTO 3360
2990         errorResult0$ = "Disk full"
3000         RETURN
3010         GOTO 3360
3020         errorResult0$ = "Input past end"
3030         RETURN
3040         GOTO 3360
3050         errorResult0$ = "Bad record number"
3060         RETURN
3070         GOTO 3360
3080         errorResult0$ = "Bad file name"
3090         RETURN
3100         GOTO 3360
3110         errorResult0$ = "Too many files"
3120         RETURN
3130         GOTO 3360
3140         errorResult0$ = "Device unavailable"
3150         RETURN
3160         GOTO 3360
3170         errorResult0$ = "Disk write protected"
3180         RETURN
3190         GOTO 3360
3200         errorResult0$ = "Disk not ready"
3210         RETURN
3220         GOTO 3360
3230         errorResult0$ = "Disk media error"
3240         RETURN
3250         GOTO 3360
3260         errorResult0$ = "Path/File access error"
3270         RETURN
3280         GOTO 3360
3290         errorResult0$ = "Path not found"
3300         RETURN
3310         GOTO 3360
3320         BCCT27% = errorCode0%
3330         BCCT28$ = STR$(BCCT27%)
3340         errorResult0$ = "Error " + BCCT28$
3350         RETURN
3360     REM END SELECT
3370     RETURN
3380 ' end function error$

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/stdlib.c</code></summary>



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
    while ((-(((-(bv_i_i <= ((int)strlen(bv_s_self))))) != 0 && ((-(strcmp(bcc_mid(bv_s_self, bv_i_i, 1), " ") == 0))) != 0))) {
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
    while ((-(((-(bv_i_i > 0))) != 0 && ((-(strcmp(bcc_mid(bv_s_self, bv_i_i, 1), " ") == 0))) != 0))) {
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
    int bt_start_0 = 1;
    int bt_lim_0 = ((int)strlen(bv_s_self));
    int bt_step_0 = 1;
    for (bv_i_i = bt_start_0; bt_step_0 >= 0 ? bv_i_i <= bt_lim_0 : bv_i_i >= bt_lim_0; bv_i_i += bt_step_0) {
        bv_i_c = ((int)(unsigned char)bcc_mid(bv_s_self, bv_i_i, 1)[0]);
        if ((-(((-(bv_i_c >= 97))) != 0 && ((-(bv_i_c <= 122))) != 0))) {
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
    int bt_start_2 = 1;
    int bt_lim_2 = ((int)strlen(bv_s_self));
    int bt_step_2 = 1;
    for (bv_i_i = bt_start_2; bt_step_2 >= 0 ? bv_i_i <= bt_lim_2 : bv_i_i >= bt_lim_2; bv_i_i += bt_step_2) {
        bv_i_c = ((int)(unsigned char)bcc_mid(bv_s_self, bv_i_i, 1)[0]);
        if ((-(((-(bv_i_c >= 65))) != 0 && ((-(bv_i_c <= 90))) != 0))) {
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
            if ((bt_sel_4 == bv_i_err_syntax)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Syntax error");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_return_without_gosub)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "RETURN without GOSUB");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_out_of_data)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Out of DATA");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_illegal_function_call)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Illegal function call");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_overflow)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Overflow");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_out_of_memory)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Out of memory");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_subscript_out_of_range)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Subscript out of range");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_duplicate_definition)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Duplicate Definition");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_division_by_zero)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Division by zero");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_type_mismatch)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Type mismatch");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_out_of_string_space)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Out of string space");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_no_resume)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "No RESUME");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_resume_without_error)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "RESUME without error");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_device_timeout)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Device timeout");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_device_fault)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Device fault");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_out_of_paper)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Out of paper");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_bad_file_number)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Bad file number");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_file_not_found)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "File not found");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_bad_file_mode)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Bad file mode");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_file_already_open)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "File already open");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_device_io)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Device I/O error");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_file_already_exists)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "File already exists");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_disk_full)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Disk full");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_input_past_end)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Input past end");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_bad_record_number)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Bad record number");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_bad_file_name)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Bad file name");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_too_many_files)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Too many files");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_device_unavailable)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Device unavailable");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_disk_write_protected)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Disk write protected");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_disk_not_ready)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Disk not ready");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_disk_media_error)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Disk media error");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_path_file_access)) {
                bt_sel_match_5 = 1;
                snprintf(bcc_out, 256, "%s", "Path/File access error");
                return;
            }
        }
        if (!bt_sel_match_5) {
            if ((bt_sel_4 == bv_i_err_path_not_found)) {
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
    // bcc tutorial/stdlib.bcl

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

<summary><code>tutorial/stdlib.j</code></summary>



```basic

.class public Stdlib
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
.method public static ltrim : (Ljava/lang/String;)Ljava/lang/String;

    iconst_0
    istore 1
    ldc 1
    istore 1
L_condition_0:
    iload 1
    aload 0
    invokevirtual java/lang/String/length ()I
    invokestatic java/lang/Integer/compare (II)I
    iconst_1
    isub
    bipush 31
    ishr
    ifeq L_condition_2
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
    ifne L_condition_4
    iconst_0
    goto L_condition_5
L_condition_4:
    iconst_m1
L_condition_5:
    goto L_condition_3
L_condition_2:
    iconst_0
L_condition_3:
    ifeq L_condition_1
    iload 1
    ldc 1
    iadd
    istore 1
    goto L_condition_0
L_condition_1:
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

    iconst_0
    istore 1
    aload 0
    invokevirtual java/lang/String/length ()I
    istore 1
L_condition_0:
    iload 1
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_condition_2
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
    ifne L_condition_4
    iconst_0
    goto L_condition_5
L_condition_4:
    iconst_m1
L_condition_5:
    goto L_condition_3
L_condition_2:
    iconst_0
L_condition_3:
    ifeq L_condition_1
    iload 1
    ldc 1
    isub
    istore 1
    goto L_condition_0
L_condition_1:
    aload 0
    iconst_0
    iload 1
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    areturn
    ldc ""
    areturn
.end method

.method public static ucase : (Ljava/lang/String;)Ljava/lang/String;

    iconst_0
    istore 1
    iconst_0
    istore 2
    ldc ""
    astore 3
    ldc ""
    astore 3
    ldc 1
    istore 4
    aload 0
    invokevirtual java/lang/String/length ()I
    istore 5
    iload 4
    istore 2
L_for_0_top:
    iload 2
    iload 5
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
    ifeq L_condition_1
    iload 1
    ldc 122
    invokestatic java/lang/Integer/compare (II)I
    iconst_1
    isub
    bipush 31
    ishr
    ifne L_condition_3
    iconst_0
    goto L_condition_4
L_condition_3:
    iconst_m1
L_condition_4:
    goto L_condition_2
L_condition_1:
    iconst_0
L_condition_2:
    ifeq L_condition_5
    iload 1
    ldc 32
    isub
    istore 1
L_condition_5:
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
L_for_0_continue:
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

    iconst_0
    istore 1
    iconst_0
    istore 2
    ldc ""
    astore 3
    ldc ""
    astore 3
    ldc 1
    istore 4
    aload 0
    invokevirtual java/lang/String/length ()I
    istore 5
    iload 4
    istore 2
L_for_0_top:
    iload 2
    iload 5
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
    ifeq L_condition_1
    iload 1
    ldc 90
    invokestatic java/lang/Integer/compare (II)I
    iconst_1
    isub
    bipush 31
    ishr
    ifne L_condition_3
    iconst_0
    goto L_condition_4
L_condition_3:
    iconst_m1
L_condition_4:
    goto L_condition_2
L_condition_1:
    iconst_0
L_condition_2:
    ifeq L_condition_5
    iload 1
    ldc 32
    iadd
    istore 1
L_condition_5:
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
L_for_0_continue:
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

    iload 0
    dup
    getstatic Stdlib/g31 I
    isub
    ifeq L_select_0_case_0
    goto L_select_0_next_0
L_select_0_next_0:
    dup
    getstatic Stdlib/g29 I
    isub
    ifeq L_select_0_case_1
    goto L_select_0_next_1
L_select_0_next_1:
    dup
    getstatic Stdlib/g21 I
    isub
    ifeq L_select_0_case_2
    goto L_select_0_next_2
L_select_0_next_2:
    dup
    getstatic Stdlib/g18 I
    isub
    ifeq L_select_0_case_3
    goto L_select_0_next_3
L_select_0_next_3:
    dup
    getstatic Stdlib/g25 I
    isub
    ifeq L_select_0_case_4
    goto L_select_0_next_4
L_select_0_next_4:
    dup
    getstatic Stdlib/g22 I
    isub
    ifeq L_select_0_case_5
    goto L_select_0_next_5
L_select_0_next_5:
    dup
    getstatic Stdlib/g30 I
    isub
    ifeq L_select_0_case_6
    goto L_select_0_next_6
L_select_0_next_6:
    dup
    getstatic Stdlib/g14 I
    isub
    ifeq L_select_0_case_7
    goto L_select_0_next_7
L_select_0_next_7:
    dup
    getstatic Stdlib/g13 I
    isub
    ifeq L_select_0_case_8
    goto L_select_0_next_8
L_select_0_next_8:
    dup
    getstatic Stdlib/g33 I
    isub
    ifeq L_select_0_case_9
    goto L_select_0_next_9
L_select_0_next_9:
    dup
    getstatic Stdlib/g24 I
    isub
    ifeq L_select_0_case_10
    goto L_select_0_next_10
L_select_0_next_10:
    dup
    getstatic Stdlib/g20 I
    isub
    ifeq L_select_0_case_11
    goto L_select_0_next_11
L_select_0_next_11:
    dup
    getstatic Stdlib/g28 I
    isub
    ifeq L_select_0_case_12
    goto L_select_0_next_12
L_select_0_next_12:
    dup
    getstatic Stdlib/g7 I
    isub
    ifeq L_select_0_case_13
    goto L_select_0_next_13
L_select_0_next_13:
    dup
    getstatic Stdlib/g5 I
    isub
    ifeq L_select_0_case_14
    goto L_select_0_next_14
L_select_0_next_14:
    dup
    getstatic Stdlib/g23 I
    isub
    ifeq L_select_0_case_15
    goto L_select_0_next_15
L_select_0_next_15:
    dup
    getstatic Stdlib/g3 I
    isub
    ifeq L_select_0_case_16
    goto L_select_0_next_16
L_select_0_next_16:
    dup
    getstatic Stdlib/g17 I
    isub
    ifeq L_select_0_case_17
    goto L_select_0_next_17
L_select_0_next_17:
    dup
    getstatic Stdlib/g1 I
    isub
    ifeq L_select_0_case_18
    goto L_select_0_next_18
L_select_0_next_18:
    dup
    getstatic Stdlib/g16 I
    isub
    ifeq L_select_0_case_19
    goto L_select_0_next_19
L_select_0_next_19:
    dup
    getstatic Stdlib/g6 I
    isub
    ifeq L_select_0_case_20
    goto L_select_0_next_20
L_select_0_next_20:
    dup
    getstatic Stdlib/g15 I
    isub
    ifeq L_select_0_case_21
    goto L_select_0_next_21
L_select_0_next_21:
    dup
    getstatic Stdlib/g9 I
    isub
    ifeq L_select_0_case_22
    goto L_select_0_next_22
L_select_0_next_22:
    dup
    getstatic Stdlib/g19 I
    isub
    ifeq L_select_0_case_23
    goto L_select_0_next_23
L_select_0_next_23:
    dup
    getstatic Stdlib/g4 I
    isub
    ifeq L_select_0_case_24
    goto L_select_0_next_24
L_select_0_next_24:
    dup
    getstatic Stdlib/g2 I
    isub
    ifeq L_select_0_case_25
    goto L_select_0_next_25
L_select_0_next_25:
    dup
    getstatic Stdlib/g32 I
    isub
    ifeq L_select_0_case_26
    goto L_select_0_next_26
L_select_0_next_26:
    dup
    getstatic Stdlib/g8 I
    isub
    ifeq L_select_0_case_27
    goto L_select_0_next_27
L_select_0_next_27:
    dup
    getstatic Stdlib/g12 I
    isub
    ifeq L_select_0_case_28
    goto L_select_0_next_28
L_select_0_next_28:
    dup
    getstatic Stdlib/g11 I
    isub
    ifeq L_select_0_case_29
    goto L_select_0_next_29
L_select_0_next_29:
    dup
    getstatic Stdlib/g10 I
    isub
    ifeq L_select_0_case_30
    goto L_select_0_next_30
L_select_0_next_30:
    dup
    getstatic Stdlib/g26 I
    isub
    ifeq L_select_0_case_31
    goto L_select_0_next_31
L_select_0_next_31:
    dup
    getstatic Stdlib/g27 I
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
    putstatic Stdlib/g1 I
    iconst_0
    putstatic Stdlib/g2 I
    iconst_0
    putstatic Stdlib/g3 I
    iconst_0
    putstatic Stdlib/g4 I
    iconst_0
    putstatic Stdlib/g5 I
    iconst_0
    putstatic Stdlib/g6 I
    iconst_0
    putstatic Stdlib/g7 I
    iconst_0
    putstatic Stdlib/g8 I
    iconst_0
    putstatic Stdlib/g9 I
    iconst_0
    putstatic Stdlib/g10 I
    iconst_0
    putstatic Stdlib/g11 I
    iconst_0
    putstatic Stdlib/g12 I
    iconst_0
    putstatic Stdlib/g13 I
    iconst_0
    putstatic Stdlib/g14 I
    iconst_0
    putstatic Stdlib/g15 I
    iconst_0
    putstatic Stdlib/g16 I
    iconst_0
    putstatic Stdlib/g17 I
    iconst_0
    putstatic Stdlib/g18 I
    iconst_0
    putstatic Stdlib/g19 I
    iconst_0
    putstatic Stdlib/g20 I
    iconst_0
    putstatic Stdlib/g21 I
    iconst_0
    putstatic Stdlib/g22 I
    iconst_0
    putstatic Stdlib/g23 I
    iconst_0
    putstatic Stdlib/g24 I
    iconst_0
    putstatic Stdlib/g25 I
    iconst_0
    putstatic Stdlib/g26 I
    iconst_0
    putstatic Stdlib/g27 I
    iconst_0
    putstatic Stdlib/g28 I
    iconst_0
    putstatic Stdlib/g29 I
    iconst_0
    putstatic Stdlib/g30 I
    iconst_0
    putstatic Stdlib/g31 I
    iconst_0
    putstatic Stdlib/g32 I
    iconst_0
    putstatic Stdlib/g33 I
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
    ldc 2
    putstatic Stdlib/g31 I
    ldc 3
    putstatic Stdlib/g29 I
    ldc 4
    putstatic Stdlib/g21 I
    ldc 5
    putstatic Stdlib/g18 I
    ldc 6
    putstatic Stdlib/g25 I
    ldc 7
    putstatic Stdlib/g22 I
    ldc 9
    putstatic Stdlib/g30 I
    ldc 10
    putstatic Stdlib/g14 I
    ldc 11
    putstatic Stdlib/g13 I
    ldc 13
    putstatic Stdlib/g33 I
    ldc 14
    putstatic Stdlib/g24 I
    ldc 19
    putstatic Stdlib/g20 I
    ldc 20
    putstatic Stdlib/g28 I
    ldc 24
    putstatic Stdlib/g7 I
    ldc 25
    putstatic Stdlib/g5 I
    ldc 27
    putstatic Stdlib/g23 I
    ldc 52
    putstatic Stdlib/g3 I
    ldc 53
    putstatic Stdlib/g17 I
    ldc 54
    putstatic Stdlib/g1 I
    ldc 55
    putstatic Stdlib/g16 I
    ldc 57
    putstatic Stdlib/g6 I
    ldc 58
    putstatic Stdlib/g15 I
    ldc 61
    putstatic Stdlib/g9 I
    ldc 62
    putstatic Stdlib/g19 I
    ldc 63
    putstatic Stdlib/g4 I
    ldc 64
    putstatic Stdlib/g2 I
    ldc 67
    putstatic Stdlib/g32 I
    ldc 68
    putstatic Stdlib/g8 I
    ldc 70
    putstatic Stdlib/g12 I
    ldc 71
    putstatic Stdlib/g11 I
    ldc 72
    putstatic Stdlib/g10 I
    ldc 75
    putstatic Stdlib/g26 I
    ldc 76
    putstatic Stdlib/g27 I
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
    ; ltrim$/rtrim$/ucase$/lcase$ are scalar methods with a bracketed string
    ; receiver type, using self$ in place of an explicit s$ parameter -- see
    ; the "Declare and call a method" chapter. A method's receiver is really
    ; just an implicit first parameter, so the ordinary call form
    ; (ltrim$("...")) keeps working exactly as before: it resolves straight to
    ; the same method declaration, with the first argument filling self$. The
    ; examples below prefer the method-call form, written as "...".ltrim().
    ; error$ stays an ordinary function: an
    ; error code is a lookup key, not a value the call is naturally "operating
    ; on" the way the others operate on their string.
    ;
    ; Run with:
    ; bcc tutorial/stdlib.bcl
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
