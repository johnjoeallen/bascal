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

490 ' ---- restore with a label: rewind the DATA pointer to a specific block ----

500 PRINT "restore to a label:"
510 READ firstcountry$
520 PRINT "  first read: "; firstcountry$
530 RESTORE 580
540 READ secondcountry$
550 PRINT "  after restore secondBatch: "; secondcountry$

560 END

570 DATA "France"

580 DATA "Japan"
