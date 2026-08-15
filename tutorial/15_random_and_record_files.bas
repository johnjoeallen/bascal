10 ' BASCAL generated BASIC
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Tutorial 15 — Random-Access Files: hand-written, then with the record/file DSL
40 ' 
50 ' This tutorial writes the *same* program twice. Part 1 uses BASIC's raw
60 ' random-access file primitives directly. Part 2 uses BASCAL's `record`/
70 ' `file` DSL, which transpiles to exactly the same primitives — nothing about
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
300 ' mki$(n%)  — pack a 2-byte integer into a 2-character string
310 ' mkl$(n&)  — pack a 4-byte long
320 ' mks$(n!)  — pack a 4-byte single
330 ' mkd$(n#)  — pack an 8-byte double
340 ' cvi(s$)   — unpack a 2-byte integer from a string
350 ' cvl(s$)   — unpack a 4-byte long
360 ' cvs(s$)   — unpack a 4-byte single
370 ' cvd(s$)   — unpack an 8-byte double
380 ' 
390 ' Every MKx$ always returns a string (never a type-suffixed MKI%/MKD#/etc —
400 ' those aren't real MBASIC/BASCOM functions), and every CVx takes no suffix
410 ' at all. There's also no RTRIM$ builtin on real MBASIC/BASCOM -- trimming a
420 ' fixed-width, space-padded FIELD buffer back down to its real length needs
430 ' a hand-rolled loop, like trimmed$ below.

440 ' trimmed$ -- right-trim trailing spaces from a fixed-width FIELD buffer.

450 reclen% = 30
460 numrecs% = 3
470 dbfile$ = "tutorial_students.dat"

480 ' ============================================================
490 ' Part 1 — random-access files, written by hand
500 ' ============================================================

510 ' ---- Write three records ----

520 OPEN dbfile$ FOR RANDOM AS #1 LEN = reclen%
530 FIELD #1, 2 AS idbuf$, 20 AS namebuf$, 8 AS scorebuf$

540 ' Record 1: Alice, 95
550 LSET idbuf$ = MKI$(1)
560 LSET namebuf$ = "Alice"
570 LSET scorebuf$ = MKD$(95)
580 PUT #1, 1

590 ' Record 2: Bob, 54
600 LSET idbuf$ = MKI$(2)
610 LSET namebuf$ = "Bob"
620 LSET scorebuf$ = MKD$(54)
630 PUT #1, 2

640 ' Record 3: Carol, 78
650 LSET idbuf$ = MKI$(3)
660 LSET namebuf$ = "Carol"
670 LSET scorebuf$ = MKD$(78)
680 PUT #1, 3

690 CLOSE #1

700 ' ---- Read records in reverse order ----

710 PRINT "Part 1 (hand-written) -- reading records in reverse order:"
720 OPEN dbfile$ FOR RANDOM AS #1 LEN = reclen%
730 FIELD #1, 2 AS idbuf$, 20 AS namebuf$, 8 AS scorebuf$

740 FOR i% = numrecs% TO 1 STEP -1
750     GET #1, i%
760     id% = CVI(idbuf$)
770     score# = CVD(scorebuf$)
780     trimmedS0$ = namebuf$
790     GOSUB 3200
800     PRINT (((("  [" + STR$(id%)) + "] ") + trimmedResult0$) + " -- ") + STR$(score#)
810 NEXT i%

820 CLOSE #1

830 ' ---- Update one field in place ----

840 OPEN dbfile$ FOR RANDOM AS #1 LEN = reclen%
850 FIELD #1, 2 AS idbuf$, 20 AS namebuf$, 8 AS scorebuf$

860 ' Bob just scraped a pass on re-mark. Only scoreBuf$ changes, but PUT
870 ' always writes the whole 30-byte buffer, so GET has to load the record
880 ' first even though idBuf$/nameBuf$ are just being written straight back
890 ' unchanged.
900 GET #1, 2
910 LSET scorebuf$ = MKD$(61.5)
920 PUT #1, 2

930 CLOSE #1

940 ' ---- Update two fields at once ----

950 OPEN dbfile$ FOR RANDOM AS #1 LEN = reclen%
960 FIELD #1, 2 AS idbuf$, 20 AS namebuf$, 8 AS scorebuf$

970 ' Alice got married and re-sat the exam — `name` and `score` both change,
980 ' `id` doesn't. Same problem as Bob's update, just with two fields instead
990 ' of one: GET first (this is what preserves idBuf$), LSET the two fields
1000 ' that actually changed, then PUT the whole buffer back. Nothing here is
1010 ' specific to "two" fields — five changed fields would look identical,
1020 ' just with five LSET lines between the GET and the PUT.
1030 GET #1, 1
1040 LSET namebuf$ = "Alice Smith"
1050 LSET scorebuf$ = MKD$(91)
1060 PUT #1, 1

1070 CLOSE #1

1080 ' ---- Same shape again ----

1090 OPEN dbfile$ FOR RANDOM AS #1 LEN = reclen%
1100 FIELD #1, 2 AS idbuf$, 20 AS namebuf$, 8 AS scorebuf$

1110 ' Carol changed her name and improved her score: the exact same
1120 ' GET / LSET / LSET / PUT shape as Alice's update above, just retyped by
1130 ' hand with Carol's record number and values.
1140 GET #1, 3
1150 LSET namebuf$ = "Carol Jones"
1160 LSET scorebuf$ = MKD$(88)
1170 PUT #1, 3

1180 CLOSE #1

1190 ' ---- Verify the updates ----

1200 PRINT "Part 1 (hand-written) -- after updates:"
1210 OPEN dbfile$ FOR RANDOM AS #1 LEN = reclen%
1220 FIELD #1, 2 AS idbuf$, 20 AS namebuf$, 8 AS scorebuf$

1230 FOR i% = 1 TO numrecs%
1240     GET #1, i%
1250     trimmedS0$ = namebuf$
1260     GOSUB 3200
1270     PRINT (("  " + trimmedResult0$) + ": ") + STR$(CVD(scorebuf$))
1280 NEXT i%

1290 CLOSE #1

1300 ' ------------------------------------------------------------------------
1310 ' What Part 1 actually cost:
1320 ' 
1330 ' - idBuf$/nameBuf$/scoreBuf$ and the FIELD statement binding them had to
1340 ' be repeated, identically, in every OPEN block — get it wrong in one
1350 ' of the five and you're reading or writing the wrong bytes.
1360 ' - recLen% (30) is 2+20+8 computed by hand; add a field to the record
1370 ' and every one of those numbers has to be updated together, or the
1380 ' file silently gets corrupted.
1390 ' - Each field's pack/unpack call (mki$/cvi, mkd$/cvd, or nothing for
1400 ' strings) has to be matched to that field's type by hand, every time
1410 ' it's touched — nothing stops mkd$() being used on the id field.
1420 ' - There's no RTRIM$ builtin on real MBASIC/BASCOM, so reading a string
1430 ' field back means hand-rolling a trim loop (trimmed$, above) and
1440 ' remembering to call it, every time.
1450 ' - Alice's and Carol's updates are the identical GET/LSET/LSET/PUT
1460 ' pattern, typed out twice, with every buffer/field name repeated.
1470 ' 
1480 ' None of this is hard, exactly — it's just bookkeeping a compiler should
1490 ' be doing for you. Part 2 is the same program again, with BASCAL's
1500 ' record/file DSL doing that bookkeeping.
1510 ' ------------------------------------------------------------------------

1520 ' ============================================================
1530 ' Part 2 — the same program with the record / file DSL
1540 ' ============================================================
1550 ' 
1560 ' record <Name> ... end record
1570 ' Declares a fixed-layout record type. Supported field types: int16,
1580 ' int32, float32, float64, and string(N). The record's total byte width
1590 ' (used as Part 1's recLen%) is the sum of its field widths, computed
1600 ' automatically.
1610 ' 
1620 ' file <var> as <RecordType> = open(<path>)
1630 ' Opens (or creates) a random-access file sized for one record, and binds
1640 ' FIELD buffer variables for every field. File numbers are allocated
1650 ' automatically, starting at #1, in the order `file` declarations appear.
1660 ' This one line replaces Part 1's recLen% constant, OPEN, and FIELD.
1670 ' 
1680 ' <file>[<n>] = { field: value, ... }
1690 ' Whole-record write: packs every field (LSET, MKx$ for numeric fields)
1700 ' and writes record n. Every declared field must be given — a missing one
1710 ' is a compile-time error.
1720 ' 
1730 ' let <var> = <file>[<n>]
1740 ' Whole-record read: reads record n and unpacks every field (CVx for
1750 ' numeric fields, an inline trim loop like Part 1's trimmed$ for strings)
1760 ' into `<var>.<field>`.
1770 ' 
1780 ' <file>[<n>].<field> = value
1790 ' Partial update: GET, LSET just that one field, PUT. The one-field
1800 ' version of Part 1's Bob update, with no buffer names to get wrong.
1810 ' 
1820 ' <file>[<n>] = ?{ field: value, ... }
1830 ' Partial-record write: any subset of fields; unlisted ones are left
1840 ' untouched on disk. Whether a GET is needed is decided at *compile
1850 ' time* by comparing the given field names against the record's declared
1860 ' fields: some fields missing -> GET first, LSET just those fields, then
1870 ' PUT (this is Alice's update from Part 1, minus the GET/LSET/LSET/PUT
1880 ' spelled out by hand); every field given anyway -> no GET, same as a
1890 ' plain `{...}`. Unlike `{...}`, an *unknown* field name is still a
1900 ' compile-time error — only *missing* fields are allowed, not misspelled
1910 ' ones.
1920 ' 
1930 ' let <var> = <file>[<n>]
1940 ' <var>.<field> = value  (any number of times)
1950 ' <file>[<n>] = <var>
1960 ' Batched update: the `let` does one GET; each `<var>.<field> = value` is
1970 ' a pure in-memory assignment (no I/O); the final `<file>[<n>] = <var>`
1980 ' packs every field from `<var>` and does one PUT. This is Carol's update
1990 ' from Part 1 — same GET/LSET/LSET/PUT shape as `?{...}`, just spelled as
2000 ' read-mutate-write instead of a single literal, useful when the new
2010 ' values come from more than a one-line expression.
2020 ' 
2030 ' for <var> = <A> downto <B> ... end for
2040 ' Sugar for `for <var> = <A> to <B> step -1`.
2050 ' 
2060 ' <file>.close()
2070 ' Closes the file.

2080 ' file db as Student = open(...)  [30 bytes/record]
2090 OPEN "tutorial_records.dat" FOR RANDOM AS #1 LEN = 30
2100 FIELD #1, 2 AS dbIdBuf$, 20 AS dbNameBuf$, 8 AS dbScoreBuf$

2110 ' ---- Write three records ----

2120 ' Record 1: Alice, 95
2130 ' db[...] = { ... }  (whole-record write)
2140 LSET dbIdBuf$ = MKI$(1)
2150 LSET dbNameBuf$ = "Alice"
2160 LSET dbScoreBuf$ = MKD$(95)
2170 PUT #1, 1

2180 ' Record 2: Bob, 54
2190 ' db[...] = { ... }  (whole-record write)
2200 LSET dbIdBuf$ = MKI$(2)
2210 LSET dbNameBuf$ = "Bob"
2220 LSET dbScoreBuf$ = MKD$(54)
2230 PUT #1, 2

2240 ' Record 3: Carol, 78
2250 ' db[...] = { ... }  (whole-record write)
2260 LSET dbIdBuf$ = MKI$(3)
2270 LSET dbNameBuf$ = "Carol"
2280 LSET dbScoreBuf$ = MKD$(78)
2290 PUT #1, 3

2300 ' ---- Read records in reverse order ----

2310 PRINT "Part 2 (record/file DSL) -- reading records in reverse order:"

2320 FOR i = 3 TO 1 STEP -1
2330     ' let s = db[...]  (whole-record read)
2340     GET #1, i
2350     sid% = CVI(dbIdBuf$)
2360     snametrimi% = LEN(dbNameBuf$)
2370     IF (snametrimi% > 0) = 0 THEN GOTO 2410
2380     IF (MID$(dbNameBuf$, snametrimi%, 1) = " ") = 0 THEN GOTO 2410
2390         snametrimi% = snametrimi% - 1
2400         GOTO 2370
2410     REM END WHILE
2420     sname$ = LEFT$(dbNameBuf$, snametrimi%)
2430     sscore# = CVD(dbScoreBuf$)
2440     PRINT (((("  [" + STR$(sid%)) + "] ") + sname$) + " -- ") + STR$(sscore#)
2450 NEXT i

2460 ' ---- Update one field in place ----

2470 ' Bob just scraped a pass on re-mark. Compare to Part 1: no recLen%, no
2480 ' idBuf$/nameBuf$/scoreBuf$, no mkd$() — just the field that's changing.
2490 ' db[...].score = ...  (partial-field update)
2500 GET #1, 2
2510 LSET dbScoreBuf$ = MKD$(61.5)
2520 PUT #1, 2

2530 ' ---- Update two fields at once, still one GET and one PUT ----

2540 ' Alice got married and re-sat the exam. `name` and `score` don't cover
2550 ' every field of Student, so this needs an implicit GET first (id is
2560 ' preserved from the existing record) -- exactly Part 1's GET / LSET /
2570 ' LSET / PUT for Alice, minus having to write out the GET, the buffer
2580 ' names, or the packing calls. Which fields need a GET is worked out by
2590 ' the compiler by comparing `name`/`score` against Student's declared
2600 ' fields — not decided at runtime.
2610 ' db[...] = ?{ ... }  (partial-record write)
2620 GET #1, 1
2630 LSET dbNameBuf$ = "Alice Smith"
2640 LSET dbScoreBuf$ = MKD$(91)
2650 PUT #1, 1

2660 ' ---- Batched update: read once, mutate twice, write back once ----

2670 ' Carol changed her name and improved her score — the read-mutate-write
2680 ' spelling of the same one-GET-one-PUT update, useful when the new values
2690 ' aren't just a couple of literals.
2700 ' let carol = db[...]  (whole-record read)
2710 GET #1, 3
2720 carolid% = CVI(dbIdBuf$)
2730 carolnametrimi% = LEN(dbNameBuf$)
2740 IF (carolnametrimi% > 0) = 0 THEN GOTO 2780
2750 IF (MID$(dbNameBuf$, carolnametrimi%, 1) = " ") = 0 THEN GOTO 2780
2760     carolnametrimi% = carolnametrimi% - 1
2770     GOTO 2740
2780 REM END WHILE
2790 carolname$ = LEFT$(dbNameBuf$, carolnametrimi%)
2800 carolscore# = CVD(dbScoreBuf$)
2810 carolname$ = "Carol Jones"
2820 carolscore# = 88
2830 ' db[...] = carol  (write back a let-bound record)
2840 LSET dbIdBuf$ = MKI$(carolid%)
2850 LSET dbNameBuf$ = carolname$
2860 LSET dbScoreBuf$ = MKD$(carolscore#)
2870 PUT #1, 3

2880 ' ---- Verify the updates ----

2890 PRINT "Part 2 (record/file DSL) -- after updates:"

2900 FOR i = 1 TO 3
2910     ' let s = db[...]  (whole-record read)
2920     GET #1, i
2930     sid% = CVI(dbIdBuf$)
2940     snametrimi% = LEN(dbNameBuf$)
2950     IF (snametrimi% > 0) = 0 THEN GOTO 2990
2960     IF (MID$(dbNameBuf$, snametrimi%, 1) = " ") = 0 THEN GOTO 2990
2970         snametrimi% = snametrimi% - 1
2980         GOTO 2950
2990     REM END WHILE
3000     sname$ = LEFT$(dbNameBuf$, snametrimi%)
3010     sscore# = CVD(dbScoreBuf$)
3020     PRINT (("  " + sname$) + ": ") + STR$(sscore#)
3030 NEXT i

3040 ' db.close()
3050 CLOSE #1

3060 ' ------------------------------------------------------------------------
3070 ' Part 2 is the same three writes, the same reverse-order read, and the
3080 ' same three updates as Part 1 — Alice's and Bob's and Carol's updates
3090 ' still transpile to exactly one GET and one PUT each, nothing runs slower.
3100 ' What's gone is everything that was bookkeeping rather than logic: the
3110 ' hand-computed record width, the repeated buffer-variable/FIELD
3120 ' boilerplate in every block, the pack/unpack call picked by hand per
3130 ' field, and the GET-or-not decision for a partial write, which the
3140 ' compiler now makes for you at compile time by simply comparing field
3150 ' names -- get a field name wrong (`db[1] = ?{ nmae: ... }`) and it's a
3160 ' compile error instead of a silently corrupted record.
3170 ' ------------------------------------------------------------------------

3180 END

3190 ' function trimmed$(s$)
3200     trimmedI0% = LEN(trimmedS0$)
3210     IF (trimmedI0% > 0) = 0 THEN GOTO 3250
3220     IF (MID$(trimmedS0$, trimmedI0%, 1) = " ") = 0 THEN GOTO 3250
3230         trimmedI0% = trimmedI0% - 1
3240         GOTO 3210
3250     REM END WHILE
3260     trimmedResult0$ = LEFT$(trimmedS0$, trimmedI0%)
3270     RETURN
3280 ' end function trimmed$
