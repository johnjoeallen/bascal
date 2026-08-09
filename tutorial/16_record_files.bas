10 ' BASCAL generated BASIC
20 ' Functions are lowered to global variables, labels, and GOSUB

30 ' Tutorial 16 — Record Files (record / file DSL)
40 ' 
50 ' This is the same random-access file I/O shown in Tutorial 15, but written
60 ' with BASCAL's higher-level `record`/`file` sugar instead of hand-written
70 ' FIELD/PUT/GET/LSET/MKx/CVx calls. The compiler computes the record's byte
80 ' width, allocates the file number, and lowers every DSL construct straight
90 ' down to the same low-level BASIC primitives from Tutorial 15 — see that
100 ' tutorial for what each one does.
110 ' 
120 ' record <Name> ... end record
130 ' Declares a fixed-layout record type. Supported field types: int16,
140 ' int32, float32, float64, and string(N). The record's total byte width
150 ' is the sum of its field widths, computed automatically.
160 ' 
170 ' file <var> as <RecordType> = open(<path>)
180 ' Opens (or creates) a random-access file sized for one record, and binds
190 ' FIELD buffer variables for every field. File numbers are allocated
200 ' automatically, starting at #1, in the order `file` declarations appear.
210 ' 
220 ' <file>[<n>] = { field: value, ... }
230 ' Whole-record write: packs every field (LSET, MKx% for numeric fields)
240 ' and writes record n.
250 ' 
260 ' let <var> = <file>[<n>]
270 ' Whole-record read: reads record n and unpacks every field (CVx$,
280 ' RTRIM$ for strings) into `<var>.<field>`.
290 ' 
300 ' <file>[<n>].<field> = value
310 ' Partial update: reads record n, rewrites just one field, writes it back.
320 ' 
330 ' for <var> = <A> downto <B> ... end for
340 ' Sugar for `for <var> = <A> to <B> step -1`.
350 ' 
360 ' <file>.close()
370 ' Closes the file.

380 ' file db as Student = open(...)  [30 bytes/record]
390 OPEN "tutorial_students.dat" FOR RANDOM AS #1 LEN = 30
400 FIELD #1, 2 AS db_idbuf$, 20 AS db_namebuf$, 8 AS db_scorebuf$

410 ' ---- Write three records ----

420 ' Record 1: Alice, 95
430 ' db[...] = { ... }  (whole-record write)
440 LSET db_idbuf$ = MKI%(1)
450 LSET db_namebuf$ = "Alice"
460 LSET db_scorebuf$ = MKD#(95)
470 PUT #1, 1

480 ' Record 2: Bob, 54
490 ' db[...] = { ... }  (whole-record write)
500 LSET db_idbuf$ = MKI%(2)
510 LSET db_namebuf$ = "Bob"
520 LSET db_scorebuf$ = MKD#(54)
530 PUT #1, 2

540 ' Record 3: Carol, 78
550 ' db[...] = { ... }  (whole-record write)
560 LSET db_idbuf$ = MKI%(3)
570 LSET db_namebuf$ = "Carol"
580 LSET db_scorebuf$ = MKD#(78)
590 PUT #1, 3

600 ' ---- Read records in reverse order ----

610 PRINT "Reading records in reverse order:"

620 FOR i = 3 TO 1 STEP -1
630     ' let s = db[...]  (whole-record read)
640     GET #1, i
650     s_id% = CVI%(db_idbuf$)
660     s_name$ = RTRIM$(db_namebuf$)
670     s_score# = CVD#(db_scorebuf$)
680     PRINT (((("  [" + STR$(s_id%)) + "] ") + s_name$) + " ") + STR$(s_score#)
690 NEXT i

700 ' ---- Update one record in place ----

710 ' Bob just scraped a pass on re-mark
720 ' db[...].score = ...  (partial-field update)
730 GET #1, 2
740 LSET db_scorebuf$ = MKD#(61.5)
750 PUT #1, 2

760 ' ---- Verify the update ----

770 PRINT "After update:"

780 FOR i = 1 TO 3
790     ' let s = db[...]  (whole-record read)
800     GET #1, i
810     s_id% = CVI%(db_idbuf$)
820     s_name$ = RTRIM$(db_namebuf$)
830     s_score# = CVD#(db_scorebuf$)
840     PRINT (("  " + s_name$) + ": ") + STR$(s_score#)
850 NEXT i

860 ' db.close()
870 CLOSE #1

880 END
