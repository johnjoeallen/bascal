10 ' BASCAL generated BASIC
20 ' Functions are lowered to global variables, labels, and GOSUB

30 ' Tutorial 15 — Random-Access File I/O
40 ' 
50 ' Random-access files store fixed-length records that can be read or written
60 ' in any order without scanning from the beginning.
70 ' 
80 ' open filename$ for random as #n len = recLen%
90 ' Open (or create) a random-access file.  len specifies the record length
100 ' in bytes; every record occupies exactly that many bytes.
110 ' 
120 ' field #n, width1% as var1$, width2% as var2$, ...
130 ' Bind string variables to regions of the file buffer.  The sum of all
140 ' widths must equal the record length.  Only string variables may be used
150 ' in a FIELD statement.
160 ' 
170 ' lset var$ = expr$   — copy into a field buffer, left-justified (padded)
180 ' rset var$ = expr$   — copy into a field buffer, right-justified (padded)
190 ' 
200 ' put #n, recordNumber%   — write the current buffer as record n (1-based)
210 ' get #n, recordNumber%   — read record n into the buffer variables
220 ' 
230 ' seek #n, recordNumber%  — position file pointer (affects next get/put
240 ' when record number is omitted)
250 ' 
260 ' Packing helpers (BASIC builtins):
270 ' mki%(n%)  — pack a 2-byte integer into a 2-character string
280 ' mkl&(n&)  — pack a 4-byte long
290 ' mks!(n!)  — pack a 4-byte single
300 ' mkd#(n#)  — pack an 8-byte double
310 ' cvi%(s$)  — unpack a 2-byte integer from a string
320 ' cvl&(s$)  — unpack a 4-byte long
330 ' cvs!(s$)  — unpack a 4-byte single
340 ' cvd#(s$)  — unpack an 8-byte double

350 CONST rec_len% = 30
360 CONST num_recs% = 3
370 CONST db_file$ = "tutorial_students.dat"

380 ' ---- Write three records ----

390 OPEN db_file$ FOR RANDOM AS #1 LEN = rec_len%

400 FIELD #1, 2 AS idbuf$, 20 AS namebuf$, 8 AS scorebuf$

410 ' Record 1: Alice, 95
420 LSET idbuf$ = MKI%(1)
430 LSET namebuf$ = "Alice"
440 LSET scorebuf$ = MKD#(95)
450 PUT #1, 1

460 ' Record 2: Bob, 54
470 LSET idbuf$ = MKI%(2)
480 LSET namebuf$ = "Bob"
490 LSET scorebuf$ = MKD#(54)
500 PUT #1, 2

510 ' Record 3: Carol, 78
520 LSET idbuf$ = MKI%(3)
530 LSET namebuf$ = "Carol"
540 LSET scorebuf$ = MKD#(78)
550 PUT #1, 3

560 CLOSE #1

570 ' ---- Read records in reverse order ----

580 PRINT "Reading records in reverse order:"
590 OPEN db_file$ FOR RANDOM AS #1 LEN = rec_len%

600 FIELD #1, 2 AS idbuf$, 20 AS namebuf$, 8 AS scorebuf$

610 FOR i% = num_recs% TO 1 STEP -1
620     GET #1, i%
630     id% = CVI%(idbuf$)
640     score# = CVD#(scorebuf$)
650     PRINT (((("  [" + STR$(id%)) + "] ") + RTRIM$(namebuf$)) + " — ") + STR$(score#)
660 NEXT i%

670 CLOSE #1

680 ' ---- Update one record in place ----

690 OPEN db_file$ FOR RANDOM AS #1 LEN = rec_len%

700 FIELD #1, 2 AS idbuf$, 20 AS namebuf$, 8 AS scorebuf$

710 ' Read record 2, update score, write it back
720 GET #1, 2
730 LSET scorebuf$ = MKD#(61.5)
740 PUT #1, 2

750 CLOSE #1

760 ' ---- Verify the update ----

770 PRINT "After update:"
780 OPEN db_file$ FOR RANDOM AS #1 LEN = rec_len%

790 FIELD #1, 2 AS idbuf$, 20 AS namebuf$, 8 AS scorebuf$

800 FOR i% = 1 TO num_recs%
810     GET #1, i%
820     PRINT (("  " + RTRIM$(namebuf$)) + ": ") + STR$(CVD#(scorebuf$))
830 NEXT i%

840 CLOSE #1

850 END
