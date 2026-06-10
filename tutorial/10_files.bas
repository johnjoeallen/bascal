10 ' BASCAL generated BASIC
20 ' Functions are lowered to global variables, labels, and GOSUB

30 ' Tutorial 10 — File Input and Output
40 ' 
50 ' BASCAL supports sequential file I/O through open, close, and the
60 ' file-channel variants of print, input, line input, and write.
70 ' 
80 ' open filename$ for input  as #n   — read existing file
90 ' open filename$ for output as #n   — create or overwrite
100 ' open filename$ for append as #n   — add to end of existing file
110 ' close #n                          — flush and release the file
120 ' 
130 ' print #n, expr[, ...]   — write values separated by spaces
140 ' write #n, expr[, ...]   — write quoted strings, comma-separated
150 ' (produces data that input # can read back)
160 ' line input #n, var$     — read one complete line into var$
170 ' input #n, var[, ...]    — read comma-delimited values (matches write)
180 ' eof(n)                  — returns non-zero when file n is exhausted

190 csvfile$ = "tutorial_scores.csv"

200 ' Write three records
210 OPEN csvfile$ FOR OUTPUT AS #1
220 WRITE #1, "Alice", 95, "pass"
230 WRITE #1, "Bob", 54, "fail"
240 WRITE #1, "Carol", 78, "pass"
250 CLOSE #1

260 ' Append a fourth record
270 OPEN csvfile$ FOR APPEND AS #1
280 WRITE #1, "Dave", 88, "pass"
290 CLOSE #1

300 ' Read and print every record
310 PRINT ("All records in " + csvfile$) + ":"
320 OPEN csvfile$ FOR INPUT AS #1
330 IF (EOF(1) = 0) = 0 THEN GOTO 370
340     INPUT #1, name$, score%, result$
350     PRINT ((((("  " + name$) + ": ") + STR$(score%)) + "  [") + result$) + "]"
360     GOTO 330
370 REM END WHILE
380 CLOSE #1

390 ' Read the file line by line using line input
400 PRINT "Raw lines:"
410 OPEN csvfile$ FOR INPUT AS #1
420 IF (EOF(1) = 0) = 0 THEN GOTO 460
430     LINE INPUT #1, line$
440     PRINT "  " + line$
450     GOTO 420
460 REM END WHILE
470 CLOSE #1

480 END
