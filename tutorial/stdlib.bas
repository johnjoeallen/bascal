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
