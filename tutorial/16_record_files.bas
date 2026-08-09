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
320 ' Own GET+PUT every time — fine for a single field, wasteful if chained.
330 ' 
340 ' let <var> = <file>[<n>]
350 ' <var>.<field> = value  (any number of times)
360 ' <file>[<n>] = <var>
370 ' Batched update: the `let` does one GET; each `<var>.<field> = value` is
380 ' a pure in-memory assignment (no I/O); the final `<file>[<n>] = <var>`
390 ' packs every field from `<var>` and does one PUT. One GET, one PUT, no
400 ' matter how many fields changed in between.
410 ' 
420 ' for <var> = <A> downto <B> ... end for
430 ' Sugar for `for <var> = <A> to <B> step -1`.
440 ' 
450 ' <file>.close()
460 ' Closes the file.

470 ' file db as Student = open(...)  [30 bytes/record]
480 OPEN "tutorial_students.dat" FOR RANDOM AS #1 LEN = 30
490 FIELD #1, 2 AS db_idbuf$, 20 AS db_namebuf$, 8 AS db_scorebuf$

500 ' ---- Write three records ----

510 ' Record 1: Alice, 95
520 ' db[...] = { ... }  (whole-record write)
530 LSET db_idbuf$ = MKI%(1)
540 LSET db_namebuf$ = "Alice"
550 LSET db_scorebuf$ = MKD#(95)
560 PUT #1, 1

570 ' Record 2: Bob, 54
580 ' db[...] = { ... }  (whole-record write)
590 LSET db_idbuf$ = MKI%(2)
600 LSET db_namebuf$ = "Bob"
610 LSET db_scorebuf$ = MKD#(54)
620 PUT #1, 2

630 ' Record 3: Carol, 78
640 ' db[...] = { ... }  (whole-record write)
650 LSET db_idbuf$ = MKI%(3)
660 LSET db_namebuf$ = "Carol"
670 LSET db_scorebuf$ = MKD#(78)
680 PUT #1, 3

690 ' ---- Read records in reverse order ----

700 PRINT "Reading records in reverse order:"

710 FOR i = 3 TO 1 STEP -1
720     ' let s = db[...]  (whole-record read)
730     GET #1, i
740     s_id% = CVI%(db_idbuf$)
750     s_name$ = RTRIM$(db_namebuf$)
760     s_score# = CVD#(db_scorebuf$)
770     PRINT (((("  [" + STR$(s_id%)) + "] ") + s_name$) + " ") + STR$(s_score#)
780 NEXT i

790 ' ---- Update one record in place ----

800 ' Bob just scraped a pass on re-mark
810 ' db[...].score = ...  (partial-field update)
820 GET #1, 2
830 LSET db_scorebuf$ = MKD#(61.5)
840 PUT #1, 2

850 ' ---- Batched update: change several fields with one GET and one PUT ----

860 ' Carol changed her name and improved her score — read once, mutate the
870 ' in-memory record twice, write back once.
880 ' let carol = db[...]  (whole-record read)
890 GET #1, 3
900 carol_id% = CVI%(db_idbuf$)
910 carol_name$ = RTRIM$(db_namebuf$)
920 carol_score# = CVD#(db_scorebuf$)
930 carol_name$ = "Carol Jones"
940 carol_score# = 88
950 ' db[...] = carol  (write back a let-bound record)
960 LSET db_idbuf$ = MKI%(carol_id%)
970 LSET db_namebuf$ = carol_name$
980 LSET db_scorebuf$ = MKD#(carol_score#)
990 PUT #1, 3

1000 ' ---- Verify the update ----

1010 PRINT "After update:"

1020 FOR i = 1 TO 3
1030     ' let s = db[...]  (whole-record read)
1040     GET #1, i
1050     s_id% = CVI%(db_idbuf$)
1060     s_name$ = RTRIM$(db_namebuf$)
1070     s_score# = CVD#(db_scorebuf$)
1080     PRINT (("  " + s_name$) + ": ") + STR$(s_score#)
1090 NEXT i

1100 ' db.close()
1110 CLOSE #1

1120 END
