10 ' BASCAL generated BASIC
20 ' Functions are lowered to global variables, labels, and GOSUB

30 ' Tutorial 15 — Random-Access Files: hand-written, then with the record/file DSL
40 ' 
50 ' This tutorial writes the *same* program twice. Part 1 uses BASIC's raw
60 ' random-access file primitives directly. Part 2 uses BASCAL's `record`/
70 ' `file` DSL, which lowers to exactly the same primitives — nothing about
80 ' the *generated* BASIC changes, only how much of it you have to write by
90 ' hand. Read Part 1 first; the comments between the two parts explain what
100 ' the DSL is buying you and why.
110 ' 
120 ' ---- Part 1 primitives ----
130 ' 
140 ' open filename$ for random as #n len = recLen%
150 ' Open (or create) a random-access file.  len specifies the record length
160 ' in bytes; every record occupies exactly that many bytes.
170 ' 
180 ' field #n, width1% as var1$, width2% as var2$, ...
190 ' Bind string variables to regions of the file buffer.  The sum of all
200 ' widths must equal the record length.  Only string variables may be used
210 ' in a FIELD statement.
220 ' 
230 ' lset var$ = expr$   — copy into a field buffer, left-justified (padded)
240 ' rset var$ = expr$   — copy into a field buffer, right-justified (padded)
250 ' 
260 ' put #n, recordNumber%   — write the current buffer as record n (1-based)
270 ' get #n, recordNumber%   — read record n into the buffer variables
280 ' 
290 ' Packing helpers (BASIC builtins):
300 ' mki%(n%)  — pack a 2-byte integer into a 2-character string
310 ' mkl&(n&)  — pack a 4-byte long
320 ' mks!(n!)  — pack a 4-byte single
330 ' mkd#(n#)  — pack an 8-byte double
340 ' cvi%(s$)  — unpack a 2-byte integer from a string
350 ' cvl&(s$)  — unpack a 4-byte long
360 ' cvs!(s$)  — unpack a 4-byte single
370 ' cvd#(s$)  — unpack an 8-byte double

380 CONST rec_len% = 30
390 CONST num_recs% = 3
400 CONST db_file$ = "tutorial_students.dat"

410 ' ============================================================
420 ' Part 1 — random-access files, written by hand
430 ' ============================================================

440 ' ---- Write three records ----

450 OPEN db_file$ FOR RANDOM AS #1 LEN = rec_len%
460 FIELD #1, 2 AS idbuf$, 20 AS namebuf$, 8 AS scorebuf$

470 ' Record 1: Alice, 95
480 LSET idbuf$ = MKI%(1)
490 LSET namebuf$ = "Alice"
500 LSET scorebuf$ = MKD#(95)
510 PUT #1, 1

520 ' Record 2: Bob, 54
530 LSET idbuf$ = MKI%(2)
540 LSET namebuf$ = "Bob"
550 LSET scorebuf$ = MKD#(54)
560 PUT #1, 2

570 ' Record 3: Carol, 78
580 LSET idbuf$ = MKI%(3)
590 LSET namebuf$ = "Carol"
600 LSET scorebuf$ = MKD#(78)
610 PUT #1, 3

620 CLOSE #1

630 ' ---- Read records in reverse order ----

640 PRINT "Part 1 (hand-written) -- reading records in reverse order:"
650 OPEN db_file$ FOR RANDOM AS #1 LEN = rec_len%
660 FIELD #1, 2 AS idbuf$, 20 AS namebuf$, 8 AS scorebuf$

670 FOR i% = num_recs% TO 1 STEP -1
680     GET #1, i%
690     id% = CVI%(idbuf$)
700     score# = CVD#(scorebuf$)
710     PRINT (((("  [" + STR$(id%)) + "] ") + RTRIM$(namebuf$)) + " -- ") + STR$(score#)
720 NEXT i%

730 CLOSE #1

740 ' ---- Update one field in place ----

750 OPEN db_file$ FOR RANDOM AS #1 LEN = rec_len%
760 FIELD #1, 2 AS idbuf$, 20 AS namebuf$, 8 AS scorebuf$

770 ' Bob just scraped a pass on re-mark. Only scoreBuf$ changes, but PUT
780 ' always writes the whole 30-byte buffer, so GET has to load the record
790 ' first even though idBuf$/nameBuf$ are just being written straight back
800 ' unchanged.
810 GET #1, 2
820 LSET scorebuf$ = MKD#(61.5)
830 PUT #1, 2

840 CLOSE #1

850 ' ---- Update two fields at once ----

860 OPEN db_file$ FOR RANDOM AS #1 LEN = rec_len%
870 FIELD #1, 2 AS idbuf$, 20 AS namebuf$, 8 AS scorebuf$

880 ' Alice got married and re-sat the exam — `name` and `score` both change,
890 ' `id` doesn't. Same problem as Bob's update, just with two fields instead
900 ' of one: GET first (this is what preserves idBuf$), LSET the two fields
910 ' that actually changed, then PUT the whole buffer back. Nothing here is
920 ' specific to "two" fields — five changed fields would look identical,
930 ' just with five LSET lines between the GET and the PUT.
940 GET #1, 1
950 LSET namebuf$ = "Alice Smith"
960 LSET scorebuf$ = MKD#(91)
970 PUT #1, 1

980 CLOSE #1

990 ' ---- Same shape again ----

1000 OPEN db_file$ FOR RANDOM AS #1 LEN = rec_len%
1010 FIELD #1, 2 AS idbuf$, 20 AS namebuf$, 8 AS scorebuf$

1020 ' Carol changed her name and improved her score: the exact same
1030 ' GET / LSET / LSET / PUT shape as Alice's update above, just retyped by
1040 ' hand with Carol's record number and values.
1050 GET #1, 3
1060 LSET namebuf$ = "Carol Jones"
1070 LSET scorebuf$ = MKD#(88)
1080 PUT #1, 3

1090 CLOSE #1

1100 ' ---- Verify the updates ----

1110 PRINT "Part 1 (hand-written) -- after updates:"
1120 OPEN db_file$ FOR RANDOM AS #1 LEN = rec_len%
1130 FIELD #1, 2 AS idbuf$, 20 AS namebuf$, 8 AS scorebuf$

1140 FOR i% = 1 TO num_recs%
1150     GET #1, i%
1160     PRINT (("  " + RTRIM$(namebuf$)) + ": ") + STR$(CVD#(scorebuf$))
1170 NEXT i%

1180 CLOSE #1

1190 ' ------------------------------------------------------------------------
1200 ' What Part 1 actually cost:
1210 ' 
1220 ' - idBuf$/nameBuf$/scoreBuf$ and the FIELD statement binding them had to
1230 ' be repeated, identically, in every OPEN block — get it wrong in one
1240 ' of the five and you're reading or writing the wrong bytes.
1250 ' - rec_len% (30) is 2+20+8 computed by hand; add a field to the record
1260 ' and every one of those numbers has to be updated together, or the
1270 ' file silently gets corrupted.
1280 ' - Each field's pack/unpack call (mki%/cvi%, mkd#/cvd#, or nothing for
1290 ' strings) has to be matched to that field's type by hand, every time
1300 ' it's touched — nothing stops mkd#() being used on the id field.
1310 ' - Alice's and Carol's updates are the identical GET/LSET/LSET/PUT
1320 ' pattern, typed out twice, with every buffer/field name repeated.
1330 ' 
1340 ' None of this is hard, exactly — it's just bookkeeping a compiler should
1350 ' be doing for you. Part 2 is the same program again, with BASCAL's
1360 ' record/file DSL doing that bookkeeping.
1370 ' ------------------------------------------------------------------------

1380 ' ============================================================
1390 ' Part 2 — the same program with the record / file DSL
1400 ' ============================================================
1410 ' 
1420 ' record <Name> ... end record
1430 ' Declares a fixed-layout record type. Supported field types: int16,
1440 ' int32, float32, float64, and string(N). The record's total byte width
1450 ' (used as Part 1's rec_len%) is the sum of its field widths, computed
1460 ' automatically.
1470 ' 
1480 ' file <var> as <RecordType> = open(<path>)
1490 ' Opens (or creates) a random-access file sized for one record, and binds
1500 ' FIELD buffer variables for every field. File numbers are allocated
1510 ' automatically, starting at #1, in the order `file` declarations appear.
1520 ' This one line replaces Part 1's rec_len% constant, OPEN, and FIELD.
1530 ' 
1540 ' <file>[<n>] = { field: value, ... }
1550 ' Whole-record write: packs every field (LSET, MKx% for numeric fields)
1560 ' and writes record n. Every declared field must be given — a missing one
1570 ' is a compile-time error.
1580 ' 
1590 ' let <var> = <file>[<n>]
1600 ' Whole-record read: reads record n and unpacks every field (CVx$,
1610 ' RTRIM$ for strings) into `<var>.<field>`.
1620 ' 
1630 ' <file>[<n>].<field> = value
1640 ' Partial update: GET, LSET just that one field, PUT. The one-field
1650 ' version of Part 1's Bob update, with no buffer names to get wrong.
1660 ' 
1670 ' <file>[<n>] = ?{ field: value, ... }
1680 ' Partial-record write: any subset of fields; unlisted ones are left
1690 ' untouched on disk. Whether a GET is needed is decided at *compile
1700 ' time* by comparing the given field names against the record's declared
1710 ' fields: some fields missing -> GET first, LSET just those fields, then
1720 ' PUT (this is Alice's update from Part 1, minus the GET/LSET/LSET/PUT
1730 ' spelled out by hand); every field given anyway -> no GET, same as a
1740 ' plain `{...}`. Unlike `{...}`, an *unknown* field name is still a
1750 ' compile-time error — only *missing* fields are allowed, not misspelled
1760 ' ones.
1770 ' 
1780 ' let <var> = <file>[<n>]
1790 ' <var>.<field> = value  (any number of times)
1800 ' <file>[<n>] = <var>
1810 ' Batched update: the `let` does one GET; each `<var>.<field> = value` is
1820 ' a pure in-memory assignment (no I/O); the final `<file>[<n>] = <var>`
1830 ' packs every field from `<var>` and does one PUT. This is Carol's update
1840 ' from Part 1 — same GET/LSET/LSET/PUT shape as `?{...}`, just spelled as
1850 ' read-mutate-write instead of a single literal, useful when the new
1860 ' values come from more than a one-line expression.
1870 ' 
1880 ' for <var> = <A> downto <B> ... end for
1890 ' Sugar for `for <var> = <A> to <B> step -1`.
1900 ' 
1910 ' <file>.close()
1920 ' Closes the file.

1930 ' file db as Student = open(...)  [30 bytes/record]
1940 OPEN "tutorial_records.dat" FOR RANDOM AS #1 LEN = 30
1950 FIELD #1, 2 AS db_idbuf$, 20 AS db_namebuf$, 8 AS db_scorebuf$

1960 ' ---- Write three records ----

1970 ' Record 1: Alice, 95
1980 ' db[...] = { ... }  (whole-record write)
1990 LSET db_idbuf$ = MKI%(1)
2000 LSET db_namebuf$ = "Alice"
2010 LSET db_scorebuf$ = MKD#(95)
2020 PUT #1, 1

2030 ' Record 2: Bob, 54
2040 ' db[...] = { ... }  (whole-record write)
2050 LSET db_idbuf$ = MKI%(2)
2060 LSET db_namebuf$ = "Bob"
2070 LSET db_scorebuf$ = MKD#(54)
2080 PUT #1, 2

2090 ' Record 3: Carol, 78
2100 ' db[...] = { ... }  (whole-record write)
2110 LSET db_idbuf$ = MKI%(3)
2120 LSET db_namebuf$ = "Carol"
2130 LSET db_scorebuf$ = MKD#(78)
2140 PUT #1, 3

2150 ' ---- Read records in reverse order ----

2160 PRINT "Part 2 (record/file DSL) -- reading records in reverse order:"

2170 FOR i = 3 TO 1 STEP -1
2180     ' let s = db[...]  (whole-record read)
2190     GET #1, i
2200     s_id% = CVI%(db_idbuf$)
2210     s_name$ = RTRIM$(db_namebuf$)
2220     s_score# = CVD#(db_scorebuf$)
2230     PRINT (((("  [" + STR$(s_id%)) + "] ") + s_name$) + " -- ") + STR$(s_score#)
2240 NEXT i

2250 ' ---- Update one field in place ----

2260 ' Bob just scraped a pass on re-mark. Compare to Part 1: no rec_len%, no
2270 ' idBuf$/nameBuf$/scoreBuf$, no mkd#() — just the field that's changing.
2280 ' db[...].score = ...  (partial-field update)
2290 GET #1, 2
2300 LSET db_scorebuf$ = MKD#(61.5)
2310 PUT #1, 2

2320 ' ---- Update two fields at once, still one GET and one PUT ----

2330 ' Alice got married and re-sat the exam. `name` and `score` don't cover
2340 ' every field of Student, so this needs an implicit GET first (id is
2350 ' preserved from the existing record) -- exactly Part 1's GET / LSET /
2360 ' LSET / PUT for Alice, minus having to write out the GET, the buffer
2370 ' names, or the packing calls. Which fields need a GET is worked out by
2380 ' the compiler by comparing `name`/`score` against Student's declared
2390 ' fields — not decided at runtime.
2400 ' db[...] = ?{ ... }  (partial-record write)
2410 GET #1, 1
2420 LSET db_namebuf$ = "Alice Smith"
2430 LSET db_scorebuf$ = MKD#(91)
2440 PUT #1, 1

2450 ' ---- Batched update: read once, mutate twice, write back once ----

2460 ' Carol changed her name and improved her score — the read-mutate-write
2470 ' spelling of the same one-GET-one-PUT update, useful when the new values
2480 ' aren't just a couple of literals.
2490 ' let carol = db[...]  (whole-record read)
2500 GET #1, 3
2510 carol_id% = CVI%(db_idbuf$)
2520 carol_name$ = RTRIM$(db_namebuf$)
2530 carol_score# = CVD#(db_scorebuf$)
2540 carol_name$ = "Carol Jones"
2550 carol_score# = 88
2560 ' db[...] = carol  (write back a let-bound record)
2570 LSET db_idbuf$ = MKI%(carol_id%)
2580 LSET db_namebuf$ = carol_name$
2590 LSET db_scorebuf$ = MKD#(carol_score#)
2600 PUT #1, 3

2610 ' ---- Verify the updates ----

2620 PRINT "Part 2 (record/file DSL) -- after updates:"

2630 FOR i = 1 TO 3
2640     ' let s = db[...]  (whole-record read)
2650     GET #1, i
2660     s_id% = CVI%(db_idbuf$)
2670     s_name$ = RTRIM$(db_namebuf$)
2680     s_score# = CVD#(db_scorebuf$)
2690     PRINT (("  " + s_name$) + ": ") + STR$(s_score#)
2700 NEXT i

2710 ' db.close()
2720 CLOSE #1

2730 ' ------------------------------------------------------------------------
2740 ' Part 2 is the same three writes, the same reverse-order read, and the
2750 ' same three updates as Part 1 — Alice's and Bob's and Carol's updates
2760 ' still lower to exactly one GET and one PUT each, nothing runs slower.
2770 ' What's gone is everything that was bookkeeping rather than logic: the
2780 ' hand-computed record width, the repeated buffer-variable/FIELD
2790 ' boilerplate in every block, the pack/unpack call picked by hand per
2800 ' field, and the GET-or-not decision for a partial write, which the
2810 ' compiler now makes for you at compile time by simply comparing field
2820 ' names -- get a field name wrong (`db[1] = ?{ nmae: ... }`) and it's a
2830 ' compile error instead of a silently corrupted record.
2840 ' ------------------------------------------------------------------------

2850 END
