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
340 ' <file>[<n>] = ?{ field: value, ... }
350 ' Partial-record write: any subset of fields, unlisted ones are left
360 ' untouched on disk. Whether a GET is needed is decided at compile time
370 ' by comparing the given field names against the record's declared
380 ' fields: some fields missing -> GET first, then LSET just those fields,
390 ' then PUT; every field given anyway -> no GET, same as a plain `{...}`.
400 ' Unlike `{...}`, an *unknown* field name is still a compile-time error —
410 ' only *missing* fields are allowed, not misspelled ones.
420 ' 
430 ' let <var> = <file>[<n>]
440 ' <var>.<field> = value  (any number of times)
450 ' <file>[<n>] = <var>
460 ' Batched update: the `let` does one GET; each `<var>.<field> = value` is
470 ' a pure in-memory assignment (no I/O); the final `<file>[<n>] = <var>`
480 ' packs every field from `<var>` and does one PUT. One GET, one PUT, no
490 ' matter how many fields changed in between.
500 ' 
510 ' for <var> = <A> downto <B> ... end for
520 ' Sugar for `for <var> = <A> to <B> step -1`.
530 ' 
540 ' <file>.close()
550 ' Closes the file.

560 ' file db as Student = open(...)  [30 bytes/record]
570 OPEN "tutorial_students.dat" FOR RANDOM AS #1 LEN = 30
580 FIELD #1, 2 AS db_idbuf$, 20 AS db_namebuf$, 8 AS db_scorebuf$

590 ' ---- Write three records ----

600 ' Record 1: Alice, 95
610 ' db[...] = { ... }  (whole-record write)
620 LSET db_idbuf$ = MKI%(1)
630 LSET db_namebuf$ = "Alice"
640 LSET db_scorebuf$ = MKD#(95)
650 PUT #1, 1

660 ' Record 2: Bob, 54
670 ' db[...] = { ... }  (whole-record write)
680 LSET db_idbuf$ = MKI%(2)
690 LSET db_namebuf$ = "Bob"
700 LSET db_scorebuf$ = MKD#(54)
710 PUT #1, 2

720 ' Record 3: Carol, 78
730 ' db[...] = { ... }  (whole-record write)
740 LSET db_idbuf$ = MKI%(3)
750 LSET db_namebuf$ = "Carol"
760 LSET db_scorebuf$ = MKD#(78)
770 PUT #1, 3

780 ' ---- Read records in reverse order ----

790 PRINT "Reading records in reverse order:"

800 FOR i = 3 TO 1 STEP -1
810     ' let s = db[...]  (whole-record read)
820     GET #1, i
830     s_id% = CVI%(db_idbuf$)
840     s_name$ = RTRIM$(db_namebuf$)
850     s_score# = CVD#(db_scorebuf$)
860     PRINT (((("  [" + STR$(s_id%)) + "] ") + s_name$) + " ") + STR$(s_score#)
870 NEXT i

880 ' ---- Update one record in place ----

890 ' Bob just scraped a pass on re-mark
900 ' db[...].score = ...  (partial-field update)
910 GET #1, 2
920 LSET db_scorebuf$ = MKD#(61.5)
930 PUT #1, 2

940 ' ---- Partial-record write: several fields, still one GET and one PUT ----

950 ' Alice got married and re-sat the exam — update two fields in one shot.
960 ' `name` and `score` don't cover every field of Student, so this needs an
970 ' implicit GET first (id is preserved from the existing record).
980 ' db[...] = ?{ ... }  (partial-record write)
990 GET #1, 1
1000 LSET db_namebuf$ = "Alice Smith"
1010 LSET db_scorebuf$ = MKD#(91)
1020 PUT #1, 1

1030 ' ---- Batched update: change several fields with one GET and one PUT ----

1040 ' Carol changed her name and improved her score — read once, mutate the
1050 ' in-memory record twice, write back once.
1060 ' let carol = db[...]  (whole-record read)
1070 GET #1, 3
1080 carol_id% = CVI%(db_idbuf$)
1090 carol_name$ = RTRIM$(db_namebuf$)
1100 carol_score# = CVD#(db_scorebuf$)
1110 carol_name$ = "Carol Jones"
1120 carol_score# = 88
1130 ' db[...] = carol  (write back a let-bound record)
1140 LSET db_idbuf$ = MKI%(carol_id%)
1150 LSET db_namebuf$ = carol_name$
1160 LSET db_scorebuf$ = MKD#(carol_score#)
1170 PUT #1, 3

1180 ' ---- Verify the update ----

1190 PRINT "After update:"

1200 FOR i = 1 TO 3
1210     ' let s = db[...]  (whole-record read)
1220     GET #1, i
1230     s_id% = CVI%(db_idbuf$)
1240     s_name$ = RTRIM$(db_namebuf$)
1250     s_score# = CVD#(db_scorebuf$)
1260     PRINT (("  " + s_name$) + ": ") + STR$(s_score#)
1270 NEXT i

1280 ' db.close()
1290 CLOSE #1

1300 END
