10 ' BASCAL generated BASIC -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Tutorial — File Input and Output
40 ' 
50 ' This tutorial writes the *same* sequential file twice. Part 1 uses raw
60 ' BASIC file statements directly. Part 2 uses BASCAL's file-handle DSL,
70 ' which transpiles to exactly the same primitives, minus having to pick a
80 ' channel number and repeat it at every open/write/read/close. Read Part
90 ' 1 first; the comments in Part 2 explain what the DSL is buying you.
100 ' 
110 ' ---- Part 1 primitives ----
120 ' 
130 ' open filename$ for input  as #n   — read existing file
140 ' open filename$ for output as #n   — create or overwrite
150 ' open filename$ for append as #n   — add to end of existing file
160 ' close #n                          — flush and release the file
170 ' 
180 ' print #n, expr[, ...]   — write values separated by spaces
190 ' write #n, expr[, ...]   — write quoted strings, comma-separated
200 ' (produces data that input # can read back)
210 ' line input #n, var$     — read one complete line into var$
220 ' input #n, var[, ...]    — read comma-delimited values (matches write)
230 ' eof(n)                  — returns non-zero when file n is exhausted

240 csvfile$ = "tutorial_scores.csv"

250 ' ============================================================
260 ' Part 1 — sequential files, written by hand
270 ' ============================================================

280 ' Write three records
290 OPEN csvfile$ FOR OUTPUT AS #1
300 WRITE #1, "Alice", 95, "pass"
310 WRITE #1, "Bob", 54, "fail"
320 WRITE #1, "Carol", 78, "pass"
330 CLOSE #1

340 ' Append a fourth record
350 OPEN csvfile$ FOR APPEND AS #1
360 WRITE #1, "Dave", 88, "pass"
370 CLOSE #1

380 ' Read and print every record
390 PRINT ("Part 1 (hand-written) -- all records in " + csvfile$) + ":"
400 OPEN csvfile$ FOR INPUT AS #1
410 IF (EOF(1) = 0) = 0 THEN GOTO 450
420     INPUT #1, name$, score%, result$
430     PRINT ((((("  " + name$) + ": ") + STR$(score%)) + "  [") + result$) + "]"
440     GOTO 410
450 REM END WHILE
460 CLOSE #1

470 ' Read the file line by line using line input
480 PRINT "Part 1 (hand-written) -- raw lines:"
490 OPEN csvfile$ FOR INPUT AS #1
500 IF (EOF(1) = 0) = 0 THEN GOTO 540
510     LINE INPUT #1, line$
520     PRINT "  " + line$
530     GOTO 500
540 REM END WHILE
550 CLOSE #1

560 ' ============================================================
570 ' Part 2 — the same file, through the file-handle DSL
580 ' ============================================================

590 ' file <var> = open(<path>) for output/input/append
600 ' Opens a file the same way `open ... for ... as #n` does, except the
610 ' compiler picks the channel number for you and remembers it under
620 ' <var> — no #1/#2 to keep straight by hand, and no risk of two open
630 ' files quietly sharing a number.
640 ' 
650 ' <var>.write(expr, ...)   — WRITE #n, expr, ...
660 ' <var>.read(var, ...)     — INPUT #n, var, ...     (only valid `for input`)
670 ' <var>.eof()               — EOF(n) <> 0             (only valid `for input`)
680 ' <var>.close()             — CLOSE #n                (valid either way)
690 ' 
700 ' A `.read()`/`.eof()` on a file not opened `for input` -- or a
710 ' `.write()` on one not opened `for output`/`for append` -- is a
720 ' transpile-time error, the same way a misspelled record field is: the
730 ' compiler already knows which direction the file goes, so it checks
740 ' for you instead of failing at runtime against real data.

750 ' file out = open(...) for output
760 OPEN csvfile$ FOR OUTPUT AS #1
770 ' out.write(...)
780 WRITE #1, "Alice", 95, "pass"
790 ' out.write(...)
800 WRITE #1, "Bob", 54, "fail"
810 ' out.write(...)
820 WRITE #1, "Carol", 78, "pass"
830 ' out.close()
840 CLOSE #1

850 ' file appended = open(...) for append
860 OPEN csvfile$ FOR APPEND AS #2
870 ' appended.write(...)
880 WRITE #2, "Dave", 88, "pass"
890 ' appended.close()
900 CLOSE #2

910 PRINT ("Part 2 (file-handle DSL) -- all records in " + csvfile$) + ":"
920 ' file dbFile = open(...) for input
930 OPEN csvfile$ FOR INPUT AS #3
940 IF (NOT (EOF(3))) = 0 THEN GOTO 990
950     ' dbFile.read(...)
960     INPUT #3, name$, score%, result$
970     PRINT ((((("  " + name$) + ": ") + STR$(score%)) + "  [") + result$) + "]"
980     GOTO 940
990 REM END WHILE
1000 ' dbFile.close()
1010 CLOSE #3

1020 ' `line input`/`print #` don't have DSL sugar yet -- fall back to the
1030 ' raw form from Part 1 for those; the DSL only replaces `open`, `write`,
1040 ' `input`, `eof()`, and `close` so far.

1050 END
