10 ' BASCAL generated BASIC
20 ' Functions are lowered to global variables, labels, and GOSUB

30 ' Tutorial 10 — File Input and Output
40 ' 
50 ' BASCAL supports sequential file I/O through OPEN, CLOSE, and the
60 ' file-channel variants of PRINT, INPUT, LINE INPUT, and WRITE.
70 ' 
80 ' OPEN filename$ FOR INPUT  AS #n   — read existing file
90 ' OPEN filename$ FOR OUTPUT AS #n   — create or overwrite
100 ' OPEN filename$ FOR APPEND AS #n   — add to end of existing file
110 ' CLOSE #n                          — flush and release the file
120 ' 
130 ' PRINT #n, expr[, ...]   — write values separated by spaces
140 ' WRITE #n, expr[, ...]   — write quoted strings, comma-separated
150 ' (produces data that INPUT # can read back)
160 ' LINE INPUT #n, var$     — read one complete line into var$
170 ' INPUT #n, var[, ...]    — read comma-delimited values (matches WRITE)
180 ' EOF(n)                  — returns non-zero when file n is exhausted

190 csvFile$ = "tutorial_scores.csv"

200 ' Write three records
210 OPEN csvFile$ FOR OUTPUT AS #1
220 WRITE #1, "Alice", 95, "pass"
230 WRITE #1, "Bob", 54, "fail"
240 WRITE #1, "Carol", 78, "pass"
250 CLOSE #1

260 ' Append a fourth record
270 OPEN csvFile$ FOR APPEND AS #1
280 WRITE #1, "Dave", 88, "pass"
290 CLOSE #1

300 ' Read and print every record
310 PRINT ("All records in " + csvFile$) + ":"
320 OPEN csvFile$ FOR INPUT AS #1
330 IF (EOF(1) = 0) = 0 THEN GOTO 370
340     INPUT #1, name$, score%, result$
350     PRINT ((((("  " + name$) + ": ") + STR$(score%)) + "  [") + result$) + "]"
360     GOTO 330
370 REM END WHILE
380 CLOSE #1

390 ' Read the file line by line using LINE INPUT
400 PRINT "Raw lines:"
410 OPEN csvFile$ FOR INPUT AS #1
420 IF (EOF(1) = 0) = 0 THEN GOTO 460
430     LINE INPUT #1, line$
440     PRINT "  " + line$
450     GOTO 420
460 REM END WHILE
470 CLOSE #1

480 END
