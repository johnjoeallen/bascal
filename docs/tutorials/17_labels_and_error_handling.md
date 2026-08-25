[Home](../../) / [Tutorials](../) / Labels and Error Handling

<div class="prose" markdown="1">

BASCAL manages line numbers itself, so `.bcl` source can never target a line number directly. `goto`, `gosub`, `on error goto`, `resume`, `restore`, and `on ... goto`/`on ... gosub` all require a `name:` label instead — the transpiler assigns the real BASIC line number when it renders output, the same job it already does for every `if`/`while`/`do`/`select case` branch target. `on error goto 0` is the one numeric exception: `0` isn't a line number, it's the sentinel that disables the error trap. See the [control-flow comparison](../../#control-flow) on the homepage for a real before/after of the generated numbering.

</div>

<div class="snippet" markdown="1">

### try/catch is the portable, both-target alternative to on error goto/resume

`on error goto`/`resume`, shown further down, is the classic BASIC model — it's the BASIC target's own mechanism. `try`/`catch` transpiles unchanged under both `--target basic` and `--target c`.

It abandons the whole `try` region on a runtime error, exposes the error metadata to `catch`, and always continues right after `end try` — never back inside `try`, and with no `resume` equivalent. Require the error library and use its named constants to filter the errors this handler accepts; an unmatched error rethrows automatically:

```bascal
require com.bascal.stdlib.error

try
    open fileName$ for input as #2
catch err%(errFileNotFound%), erl%, source$
    print "caught error "; err%; " at "; source$; ":"; erl%
    print fileName$; " not found"
end try
```

On the BASIC target this transpiles straight onto real `ON ERROR GOTO`/`RESUME <label>` and covers every raise site. On the C target a raise is caught when it happens in the `try` block itself or inside any procedure/function called (directly or transitively) from there, including calls embedded in larger expressions. `try`/`catch` can't be nested on either target.

</div>

<div class="snippet" markdown="1">

### A label can share its line with the statement that follows it

```bascal
goto afterSkip
print "not reached"
afterSkip:
print "reached via goto"
```

</div>

<div class="snippet" markdown="1">

### RESUME to a label clears the error trap and continues past the whole try/handler region

A plain GOTO out of a handler would leave the runtime still marked "currently handling an error" — RESUME is what clears that state so a later error can still be trapped.

```bascal
on error goto handleOpenError
open fileName$ for input as #1
' ...
goto afterOpen

handleOpenError:
if err = errFileNotFound% then
    print "caught error "; err; ": "; fileName$; " not found"
    resume afterOpen
else
    error err
end if

afterOpen:
on error goto 0
```

</div>

<div class="snippet" markdown="1">

### RESTORE takes a label too, rewinding the DATA pointer to a specific block

```bascal
restore secondBatch
read secondCountry$
...
secondBatch:
data "Japan"
```

</div>



[← Short-Circuit && and \|\|](16_short_circuit.md)  ·  [Standard Library Functions →](18_stdlib.md)


<!-- BEGIN generated tutorial source -->

<details class="source-embed" markdown="1">

<summary><code>tutorial/17_labels_and_error_handling.bcl</code></summary>



```bascal

// Tutorial — Labels and Error Handling
//
// BASCAL manages line numbers itself -- goto, gosub, on error goto, resume,
// restore, and on ... goto / on ... gosub can never target a raw line
// number in .bcl source. Every one of them requires a name: label instead;
// the compiler assigns the real BASIC line number when it renders output,
// the same way it already numbers the branch targets inside if/while/do/
// select case.
//
// on error goto 0 is the one numeric exception -- 0 isn't a line number,
// it's the sentinel that disables the error trap.
program labelsAndErrorHandling
require com.bascal.stdlib.error

/* ---- goto / label basics ---- */

print "goto/label basics:"
goto afterSkip
print "  not reached"
afterSkip:
print "  reached via goto"

/* ---- portable procedure call (replaces BASIC-level GOSUB) ---- */

print "procedure call:"
printBanner()
print "  back after gosub"
goto afterBanner

procedure printBanner()
    print "  inside the gosub'd subroutine"
end procedure

afterBanner:

/* ---- error handling: on error goto, resume to a label, err ---- */
//
// Opening a file that doesn't exist raises BASIC runtime error 53
// ("file not found"). The handler below catches it, prints a message, and
// then RESUMEs at a label -- not the failing statement or "next", but a
// specific point past the whole try/handler region. RESUME (not a plain
// GOTO) is what clears the runtime's "currently handling an error" state,
// so a later error can still be trapped.

print "error handling, missing file:"
fileName$ = "does_not_exist.dat"
on error goto handleOpenError
open fileName$ for input as #1
print "  file opened (unexpected)"
close #1
goto afterOpen

handleOpenError:
if err = errFileNotFound% then
    print "  caught error "; err; ": "; fileName$; " not found"
    resume afterOpen
else
    print "  unexpected error "; err
    error err
end if

afterOpen:
on error goto 0

/* ---- try/catch: portable structured error recovery, and throw to rethrow ---- */
//
// try/catch (issue #60) is BASCAL's structured alternative to on error
// goto/resume above -- it transpiles unchanged under both --target basic
// and --target C. A failed statement anywhere in the try body abandons
// the rest of it and runs catch once, then execution always continues
// right after end try -- never back inside try, and with no resume
// equivalent at all. err%/erl%/source$ are ordinary locals scoped to the
// catch block, not aliases for the ambient err/erl on error goto above.
//
// The filter means this catch handles only errFileNotFound%; every other
// error automatically rethrows after the try block.

print "try/catch, missing file, with rethrow:"
fileName$ = "also_missing.dat"
try
    open fileName$ for input as #2
    print "  file opened (unexpected)"
    close #2
catch err%(errFileNotFound%), erl%, source$
    print "  caught error "; err%; " at "; source$; ":"; erl%
    print "  "; fileName$; " not found"
end try

/* ---- restore with a label: rewind the DATA pointer to a specific block ---- */

print "restore to a label:"
read firstCountry$
print "  first read: "; firstCountry$
restore secondBatch
read secondCountry$
print "  after restore secondBatch: "; secondCountry$

end

data "France"

secondBatch:
data "Japan"

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/17_labels_and_error_handling.bas</code></summary>



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

520 ' Tutorial — Labels and Error Handling
530 '
540 ' BASCAL manages line numbers itself -- goto, gosub, on error goto, resume,
550 ' restore, and on ... goto / on ... gosub can never target a raw line
560 ' number in .bcl source. Every one of them requires a name: label instead;
570 ' the compiler assigns the real BASIC line number when it renders output,
580 ' the same way it already numbers the branch targets inside if/while/do/
590 ' select case.
600 '
610 ' on error goto 0 is the one numeric exception -- 0 isn't a line number,
620 ' it's the sentinel that disables the error trap.

630 ' ---- goto / label basics ----

640 PRINT "goto/label basics:"
650 GOTO 670
660 PRINT "  not reached"
670 PRINT "  reached via goto"

680 ' ---- portable procedure call (replaces BASIC-level GOSUB) ----

690 PRINT "procedure call:"
700 GOSUB 2880
710 PRINT "  back after gosub"
720 GOTO 730

730 ' ---- error handling: on error goto, resume to a label, err ----
740 '
750 ' Opening a file that doesn't exist raises BASIC runtime error 53
760 ' ("file not found"). The handler below catches it, prints a message, and
770 ' then RESUMEs at a label -- not the failing statement or "next", but a
780 ' specific point past the whole try/handler region. RESUME (not a plain
790 ' GOTO) is what clears the runtime's "currently handling an error" state,
800 ' so a later error can still be trapped.

810 PRINT "error handling, missing file:"
820 filename$ = "does_not_exist.dat"
830 ON ERROR GOTO 880
840 OPEN filename$ FOR INPUT AS #1
850 PRINT "  file opened (unexpected)"
860 CLOSE #1
870 GOTO 950

880 IF (ERR = errfilenotfound%) = 0 THEN GOTO 920
890     PRINT "  caught error "; ERR; ": "; filename$; " not found"
900     RESUME 950
910     GOTO 940
920     PRINT "  unexpected error "; ERR
930     ERROR ERR
940 REM END IF

950 ON ERROR GOTO 0

960 ' ---- try/catch: portable structured error recovery, and throw to rethrow ----
970 '
980 ' try/catch (issue #60) is BASCAL's structured alternative to on error
990 ' goto/resume above -- it transpiles unchanged under both --target basic
1000 ' and --target C. A failed statement anywhere in the try body abandons
1010 ' the rest of it and runs catch once, then execution always continues
1020 ' right after end try -- never back inside try, and with no resume
1030 ' equivalent at all. err%/erl%/source$ are ordinary locals scoped to the
1040 ' catch block, not aliases for the ambient err/erl on error goto above.
1050 '
1060 ' The filter means this catch handles only errFileNotFound%; every other
1070 ' error automatically rethrows after the try block.

1080 PRINT "try/catch, missing file, with rethrow:"
1090 filename$ = "also_missing.dat"
1100 ON ERROR GOTO 1170
1110 BCC_TRY_0002_PENDING% = 0
1120     OPEN filename$ FOR INPUT AS #2
1130     PRINT "  file opened (unexpected)"
1140     CLOSE #2
1150 ON ERROR GOTO 0
1160 GOTO 1330
1170     BCC_TRY_0002_PENDING% = ERR
1180     IF (ERR = errfilenotfound%) THEN GOTO 1200
1190     RESUME 1330
1200     err% = ERR
1210     erl% = ERL
1220     GOSUB 2920
1230     source$ = BCC_SOURCE_FILE$
1240     RESUME 1250
1250 ON ERROR GOTO 1310
1260     PRINT "  caught error "; err%; " at "; source$; ":"; erl%
1270     PRINT "  "; filename$; " not found"
1280     BCC_TRY_0002_PENDING% = 0
1290     ON ERROR GOTO 0
1300     GOTO 1330
1310     BCC_TRY_0002_PENDING% = ERR
1320     RESUME 1330
1330 ON ERROR GOTO 0
1340     IF BCC_TRY_0002_PENDING% <> 0 THEN ERROR BCC_TRY_0002_PENDING%
1350 REM END TRY

1360 ' ---- restore with a label: rewind the DATA pointer to a specific block ----

1370 PRINT "restore to a label:"
1380 READ firstcountry$
1390 PRINT "  first read: "; firstcountry$
1400 RESTORE 1450
1410 READ secondcountry$
1420 PRINT "  after restore secondBatch: "; secondcountry$

1430 END

1440 DATA "France"

1450 DATA "Japan"
1460 END

1470 ' function error$(code%)
1480     BCCT4% = errorCode0%
1490     IF (BCCT4% = errsyntax%) <> 0 THEN GOTO 1830
1500     IF (BCCT4% = errreturnwithoutgosub%) <> 0 THEN GOTO 1860
1510     IF (BCCT4% = erroutofdata%) <> 0 THEN GOTO 1890
1520     IF (BCCT4% = errillegalfunctioncall%) <> 0 THEN GOTO 1920
1530     IF (BCCT4% = erroverflow%) <> 0 THEN GOTO 1950
1540     IF (BCCT4% = erroutofmemory%) <> 0 THEN GOTO 1980
1550     IF (BCCT4% = errsubscriptoutofrange%) <> 0 THEN GOTO 2010
1560     IF (BCCT4% = errduplicatedefinition%) <> 0 THEN GOTO 2040
1570     IF (BCCT4% = errdivisionbyzero%) <> 0 THEN GOTO 2070
1580     IF (BCCT4% = errtypemismatch%) <> 0 THEN GOTO 2100
1590     IF (BCCT4% = erroutofstringspace%) <> 0 THEN GOTO 2130
1600     IF (BCCT4% = errnoresume%) <> 0 THEN GOTO 2160
1610     IF (BCCT4% = errresumewithouterror%) <> 0 THEN GOTO 2190
1620     IF (BCCT4% = errdevicetimeout%) <> 0 THEN GOTO 2220
1630     IF (BCCT4% = errdevicefault%) <> 0 THEN GOTO 2250
1640     IF (BCCT4% = erroutofpaper%) <> 0 THEN GOTO 2280
1650     IF (BCCT4% = errbadfilenumber%) <> 0 THEN GOTO 2310
1660     IF (BCCT4% = errfilenotfound%) <> 0 THEN GOTO 2340
1670     IF (BCCT4% = errbadfilemode%) <> 0 THEN GOTO 2370
1680     IF (BCCT4% = errfilealreadyopen%) <> 0 THEN GOTO 2400
1690     IF (BCCT4% = errdeviceio%) <> 0 THEN GOTO 2430
1700     IF (BCCT4% = errfilealreadyexists%) <> 0 THEN GOTO 2460
1710     IF (BCCT4% = errdiskfull%) <> 0 THEN GOTO 2490
1720     IF (BCCT4% = errinputpastend%) <> 0 THEN GOTO 2520
1730     IF (BCCT4% = errbadrecordnumber%) <> 0 THEN GOTO 2550
1740     IF (BCCT4% = errbadfilename%) <> 0 THEN GOTO 2580
1750     IF (BCCT4% = errtoomanyfiles%) <> 0 THEN GOTO 2610
1760     IF (BCCT4% = errdeviceunavailable%) <> 0 THEN GOTO 2640
1770     IF (BCCT4% = errdiskwriteprotected%) <> 0 THEN GOTO 2670
1780     IF (BCCT4% = errdisknotready%) <> 0 THEN GOTO 2700
1790     IF (BCCT4% = errdiskmediaerror%) <> 0 THEN GOTO 2730
1800     IF (BCCT4% = errpathfileaccess%) <> 0 THEN GOTO 2760
1810     IF (BCCT4% = errpathnotfound%) <> 0 THEN GOTO 2790
1820     GOTO 2820
1830         errorResult0$ = "Syntax error"
1840         RETURN
1850         GOTO 2840
1860         errorResult0$ = "RETURN without GOSUB"
1870         RETURN
1880         GOTO 2840
1890         errorResult0$ = "Out of DATA"
1900         RETURN
1910         GOTO 2840
1920         errorResult0$ = "Illegal function call"
1930         RETURN
1940         GOTO 2840
1950         errorResult0$ = "Overflow"
1960         RETURN
1970         GOTO 2840
1980         errorResult0$ = "Out of memory"
1990         RETURN
2000         GOTO 2840
2010         errorResult0$ = "Subscript out of range"
2020         RETURN
2030         GOTO 2840
2040         errorResult0$ = "Duplicate Definition"
2050         RETURN
2060         GOTO 2840
2070         errorResult0$ = "Division by zero"
2080         RETURN
2090         GOTO 2840
2100         errorResult0$ = "Type mismatch"
2110         RETURN
2120         GOTO 2840
2130         errorResult0$ = "Out of string space"
2140         RETURN
2150         GOTO 2840
2160         errorResult0$ = "No RESUME"
2170         RETURN
2180         GOTO 2840
2190         errorResult0$ = "RESUME without error"
2200         RETURN
2210         GOTO 2840
2220         errorResult0$ = "Device timeout"
2230         RETURN
2240         GOTO 2840
2250         errorResult0$ = "Device fault"
2260         RETURN
2270         GOTO 2840
2280         errorResult0$ = "Out of paper"
2290         RETURN
2300         GOTO 2840
2310         errorResult0$ = "Bad file number"
2320         RETURN
2330         GOTO 2840
2340         errorResult0$ = "File not found"
2350         RETURN
2360         GOTO 2840
2370         errorResult0$ = "Bad file mode"
2380         RETURN
2390         GOTO 2840
2400         errorResult0$ = "File already open"
2410         RETURN
2420         GOTO 2840
2430         errorResult0$ = "Device I/O error"
2440         RETURN
2450         GOTO 2840
2460         errorResult0$ = "File already exists"
2470         RETURN
2480         GOTO 2840
2490         errorResult0$ = "Disk full"
2500         RETURN
2510         GOTO 2840
2520         errorResult0$ = "Input past end"
2530         RETURN
2540         GOTO 2840
2550         errorResult0$ = "Bad record number"
2560         RETURN
2570         GOTO 2840
2580         errorResult0$ = "Bad file name"
2590         RETURN
2600         GOTO 2840
2610         errorResult0$ = "Too many files"
2620         RETURN
2630         GOTO 2840
2640         errorResult0$ = "Device unavailable"
2650         RETURN
2660         GOTO 2840
2670         errorResult0$ = "Disk write protected"
2680         RETURN
2690         GOTO 2840
2700         errorResult0$ = "Disk not ready"
2710         RETURN
2720         GOTO 2840
2730         errorResult0$ = "Disk media error"
2740         RETURN
2750         GOTO 2840
2760         errorResult0$ = "Path/File access error"
2770         RETURN
2780         GOTO 2840
2790         errorResult0$ = "Path not found"
2800         RETURN
2810         GOTO 2840
2820         errorResult0$ = "Error " + STR$(errorCode0%)
2830         RETURN
2840     REM END SELECT
2850     RETURN
2860 ' end function error$

2870 ' procedure printbanner()
2880     PRINT "  inside the gosub'd subroutine"
2890     RETURN
2900 ' end procedure printbanner

2910 ' catch's optional source$ binding: map ERL back to its original .bcl file
2920     IF ERL <= 510 THEN BCC_SOURCE_FILE$ = "com/bascal/stdlib/error.bcl" : RETURN
2930     IF ERL <= 1470 THEN BCC_SOURCE_FILE$ = "tutorial/17_labels_and_error_handling.bcl" : RETURN
2940     IF ERL <= 2870 THEN BCC_SOURCE_FILE$ = "com/bascal/stdlib/error.bcl" : RETURN
2950     BCC_SOURCE_FILE$ = "tutorial/17_labels_and_error_handling.bcl"
2960     RETURN

```



</details>

<!-- END generated tutorial source -->
