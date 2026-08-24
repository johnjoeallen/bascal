[Home](../../) / [Tutorials](../) / Labels and Error Handling

<div class="prose" markdown="1">

BASCAL manages line numbers itself, so `.bcl` source can never target a line number directly. `goto`, `gosub`, `on error goto`, `resume`, `restore`, and `on ... goto`/`on ... gosub` all require a `name:` label instead — the transpiler assigns the real BASIC line number when it renders output, the same job it already does for every `if`/`while`/`do`/`select case` branch target. `on error goto 0` is the one numeric exception: `0` isn't a line number, it's the sentinel that disables the error trap. See the [control-flow comparison](../../#control-flow) on the homepage for a real before/after of the generated numbering.

</div>

<div class="snippet" markdown="1">

### try/catch is the portable, both-target alternative to on error goto/resume

`on error goto`/`resume`, shown further down, is the classic BASIC model — it's the BASIC target's own mechanism. `try`/`catch` transpiles unchanged under both `--target basic` and `--target C`.

It abandons the whole `try` region on a runtime error, exposes the error metadata to `catch`, and always continues right after `end try` — never back inside `try`, and with no `resume` equivalent. `throw` with no argument re-raises whatever error `catch` just captured — useful for a case a given `catch` doesn't recognize, so it keeps propagating instead of being silently swallowed:

```bascal
try
    open fileName$ for input as #2
catch err%, erl%
    if err% = 53 then
        print "caught error "; err%; ": "; fileName$; " not found"
    else
        print "unexpected error "; err%; ", rethrowing"
        throw
    end if
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
if err = 53 then
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

### `tutorial/17_labels_and_error_handling.bcl`

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

/* ---- goto / label basics ---- */

print "goto/label basics:"
goto afterSkip
print "  not reached"
afterSkip:
print "  reached via goto"

/* ---- gosub / return (BASIC-level subroutine, distinct from BASCAL functions) ---- */

print "gosub/return:"
gosub printBanner
print "  back after gosub"
goto afterBanner

printBanner:
print "  inside the gosub'd subroutine"
return

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
if err = 53 then
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
// equivalent at all. err%/erl% are ordinary locals scoped to the catch
// block, not aliases for the ambient err/erl on error goto uses above.
//
// throw with no argument rethrows whatever error err%/erl% just
// captured -- the portable equivalent of on error goto's own "error err"
// above -- for an error this catch doesn't recognize, so it keeps
// propagating instead of being silently swallowed.

print "try/catch, missing file, with rethrow:"
fileName$ = "also_missing.dat"
try
    open fileName$ for input as #2
    print "  file opened (unexpected)"
    close #2
catch err%, erl%
    if err% = 53 then
        print "  caught error "; err%; ": "; fileName$; " not found"
    else
        print "  unexpected error "; err%; ", rethrowing"
        throw
    end if
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

### `tutorial/17_labels_and_error_handling.bas`

```basic

10 ' BASCAL generated BASIC
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Tutorial — Labels and Error Handling
40 '
50 ' BASCAL manages line numbers itself -- goto, gosub, on error goto, resume,
60 ' restore, and on ... goto / on ... gosub can never target a raw line
70 ' number in .bcl source. Every one of them requires a name: label instead;
80 ' the compiler assigns the real BASIC line number when it renders output,
90 ' the same way it already numbers the branch targets inside if/while/do/
100 ' select case.
110 '
120 ' on error goto 0 is the one numeric exception -- 0 isn't a line number,
130 ' it's the sentinel that disables the error trap.

140 ' ---- goto / label basics ----

150 PRINT "goto/label basics:"
160 GOTO 180
170 PRINT "  not reached"
180 PRINT "  reached via goto"

190 ' ---- gosub / return (BASIC-level subroutine, distinct from BASCAL functions) ----

200 PRINT "gosub/return:"
210 GOSUB 240
220 PRINT "  back after gosub"
230 GOTO 260

240 PRINT "  inside the gosub'd subroutine"
250 RETURN

260 ' ---- error handling: on error goto, resume to a label, err ----
270 '
280 ' Opening a file that doesn't exist raises BASIC runtime error 53
290 ' ("file not found"). The handler below catches it, prints a message, and
300 ' then RESUMEs at a label -- not the failing statement or "next", but a
310 ' specific point past the whole try/handler region. RESUME (not a plain
320 ' GOTO) is what clears the runtime's "currently handling an error" state,
330 ' so a later error can still be trapped.

340 PRINT "error handling, missing file:"
350 filename$ = "does_not_exist.dat"
360 ON ERROR GOTO 410
370 OPEN filename$ FOR INPUT AS #1
380 PRINT "  file opened (unexpected)"
390 CLOSE #1
400 GOTO 480

410 IF (ERR = 53) = 0 THEN GOTO 450
420     PRINT "  caught error "; ERR; ": "; filename$; " not found"
430     RESUME 480
440     GOTO 470
450     PRINT "  unexpected error "; ERR
460     ERROR ERR
470 REM END IF

480 ON ERROR GOTO 0

490 ' ---- try/catch: portable structured error recovery, and throw to rethrow ----
500 '
510 ' try/catch (issue #60) is BASCAL's structured alternative to on error
520 ' goto/resume above -- it transpiles unchanged under both --target basic
530 ' and --target C. A failed statement anywhere in the try body abandons
540 ' the rest of it and runs catch once, then execution always continues
550 ' right after end try -- never back inside try, and with no resume
560 ' equivalent at all. err%/erl% are ordinary locals scoped to the catch
570 ' block, not aliases for the ambient err/erl on error goto uses above.
580 '
590 ' throw with no argument rethrows whatever error err%/erl% just
600 ' captured -- the portable equivalent of on error goto's own "error err"
610 ' above -- for an error this catch doesn't recognize, so it keeps
620 ' propagating instead of being silently swallowed.

630 PRINT "try/catch, missing file, with rethrow:"
640 filename$ = "also_missing.dat"
650 ON ERROR GOTO 720
660 BCC_TRY_0002_PENDING% = 0
670     OPEN filename$ FOR INPUT AS #2
680     PRINT "  file opened (unexpected)"
690     CLOSE #2
700 ON ERROR GOTO 0
710 GOTO 880
720     BCC_TRY_0002_PENDING% = ERR
730     err% = ERR
740     erl% = ERL
750     RESUME 760
760 ON ERROR GOTO 860
770     IF (err% = 53) = 0 THEN GOTO 800
780         PRINT "  caught error "; err%; ": "; filename$; " not found"
790         GOTO 820
800         PRINT "  unexpected error "; err%; ", rethrowing"
810         ERROR ERR
820     REM END IF
830     BCC_TRY_0002_PENDING% = 0
840     ON ERROR GOTO 0
850     GOTO 880
860     BCC_TRY_0002_PENDING% = ERR
870     RESUME 880
880 ON ERROR GOTO 0
890     IF BCC_TRY_0002_PENDING% <> 0 THEN ERROR BCC_TRY_0002_PENDING%
900 REM END TRY

910 ' ---- restore with a label: rewind the DATA pointer to a specific block ----

920 PRINT "restore to a label:"
930 READ firstcountry$
940 PRINT "  first read: "; firstcountry$
950 RESTORE 1000
960 READ secondcountry$
970 PRINT "  after restore secondBatch: "; secondcountry$

980 END

990 DATA "France"

1000 DATA "Japan"

```

<!-- END generated tutorial source -->
