[Home](../) / [Tutorials](./) / Standard Library Functions

<div class="prose" markdown="1">

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

print "[" + ltrim$("   padded left") + "]"
print "[" + rtrim$("padded right   ") + "]"
print ucase$("shout this")
print lcase$("QUIET THIS DOWN")
```

</div>

<div class="snippet" markdown="1">

### ERROR\$ maps a classic error code to a message

Pair it with ERR inside an ON ERROR GOTO handler in real code -- see [tutorial 17](17_labels_and_error_handling.md).

```bascal
print error$(53)   ' File not found
print error$(11)   ' Division by zero
print error$(9999) ' Error  9999 (falls through to STR$)
```

</div>



[← Labels and Error Handling](17_labels_and_error_handling.md)  ·  [Case Study: Random-Access Inventory →](19_inventory.md)


<!-- BEGIN generated tutorial source -->

### `tutorial/18_stdlib.bcl`

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
//   bcc tutorial/18_stdlib.bcl
program stdlib

require com.bascal.stdlib.ltrim
require com.bascal.stdlib.rtrim
require com.bascal.stdlib.ucase
require com.bascal.stdlib.lcase
require com.bascal.stdlib.error

print "[" + ltrim$("   padded left") + "]"
print "[" + rtrim$("padded right   ") + "]"
print ucase$("shout this")
print lcase$("QUIET THIS DOWN")

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

### `tutorial/18_stdlib.bas`

```basic

10 ' BASCAL generated BASIC
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

<!-- END generated tutorial source -->
