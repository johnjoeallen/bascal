10 ' BASCAL generated BASIC
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Tutorial — Random-Access Files: hand-written, then with the record/file DSL
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

450 reclen% = 50
460 numrecs% = 3
470 dbfile$ = "tutorial_students.dat"

480 ' ============================================================
490 ' Part 1 — random-access files, written by hand
500 ' ============================================================

510 ' ---- Write three records ----

520 OPEN dbfile$ FOR RANDOM AS #1 LEN = reclen%
530 FIELD #1, 2 AS idbuf$, 20 AS namebuf$, 8 AS scorebuf$, 20 AS facultybuf$

540 ' Record 1: Alice, 95
550 LSET idbuf$ = MKI$(1)
560 LSET namebuf$ = "Alice"
570 LSET scorebuf$ = MKD$(95)
580 LSET facultybuf$ = "Engineering"
590 PUT #1, 1

600 ' Record 2: Bob, 54
610 LSET idbuf$ = MKI$(2)
620 LSET namebuf$ = "Bob"
630 LSET scorebuf$ = MKD$(54)
640 LSET facultybuf$ = "Arts"
650 PUT #1, 2

660 ' Record 3: Carol, 78
670 LSET idbuf$ = MKI$(3)
680 LSET namebuf$ = "Carol"
690 LSET scorebuf$ = MKD$(78)
700 LSET facultybuf$ = "Science"
710 PUT #1, 3

720 CLOSE #1

730 ' ---- Read records in reverse order ----

740 PRINT "Part 1 (hand-written) -- reading records in reverse order:"
750 OPEN dbfile$ FOR RANDOM AS #1 LEN = reclen%
760 FIELD #1, 2 AS idbuf$, 20 AS namebuf$, 8 AS scorebuf$, 20 AS facultybuf$

770 FOR i% = numrecs% TO 1 STEP -1
780     GET #1, i%
790     id% = CVI(idbuf$)
800     score# = CVD(scorebuf$)
810     trimmedS0$ = namebuf$
820     GOSUB 3500
830     PRINT (((("  [" + STR$(id%)) + "] ") + trimmedResult0$) + " -- ") + STR$(score#)
840 NEXT i%

850 CLOSE #1

860 ' ---- Update one field in place ----

870 OPEN dbfile$ FOR RANDOM AS #1 LEN = reclen%
880 FIELD #1, 2 AS idbuf$, 20 AS namebuf$, 8 AS scorebuf$, 20 AS facultybuf$

890 ' Bob just scraped a pass on re-mark. Only scoreBuf$ changes, but PUT
900 ' always writes the whole 50-byte buffer, so GET has to load the record
910 ' first even though idBuf$/nameBuf$/facultyBuf$ are just being written straight back
920 ' unchanged.
930 GET #1, 2
940 LSET scorebuf$ = MKD$(61.5)
950 PUT #1, 2

960 CLOSE #1

970 ' ---- Update two fields at once ----

980 OPEN dbfile$ FOR RANDOM AS #1 LEN = reclen%
990 FIELD #1, 2 AS idbuf$, 20 AS namebuf$, 8 AS scorebuf$, 20 AS facultybuf$

1000 ' Alice got married and re-sat the exam — `name` and `score` both change,
1010 ' `id` and `faculty` don't. Same problem as Bob's update, just with two fields instead
1020 ' of one: GET first (this is what preserves idBuf$ and facultyBuf$), LSET the two fields
1030 ' that actually changed, then PUT the whole buffer back. Nothing here is
1040 ' specific to "two" fields — five changed fields would look identical,
1050 ' just with five LSET lines between the GET and the PUT.
1060 GET #1, 1
1070 LSET namebuf$ = "Alice Smith"
1080 LSET scorebuf$ = MKD$(91)
1090 PUT #1, 1

1100 CLOSE #1

1110 ' ---- Same shape again ----

1120 OPEN dbfile$ FOR RANDOM AS #1 LEN = reclen%
1130 FIELD #1, 2 AS idbuf$, 20 AS namebuf$, 8 AS scorebuf$, 20 AS facultybuf$

1140 ' Carol changed her name and improved her score: the exact same
1150 ' GET / LSET / LSET / PUT shape as Alice's update above, just retyped by
1160 ' hand with Carol's record number and values.
1170 GET #1, 3
1180 LSET namebuf$ = "Carol Jones"
1190 LSET scorebuf$ = MKD$(88)
1200 PUT #1, 3

1210 CLOSE #1

1220 ' ---- Verify the updates ----

1230 PRINT "Part 1 (hand-written) -- after updates:"
1240 OPEN dbfile$ FOR RANDOM AS #1 LEN = reclen%
1250 FIELD #1, 2 AS idbuf$, 20 AS namebuf$, 8 AS scorebuf$, 20 AS facultybuf$

1260 FOR i% = 1 TO numrecs%
1270     GET #1, i%
1280     trimmedS0$ = namebuf$
1290     GOSUB 3500
1300     PRINT (("  " + trimmedResult0$) + ": ") + STR$(CVD(scorebuf$))
1310 NEXT i%

1320 CLOSE #1

1330 ' ------------------------------------------------------------------------
1340 ' What Part 1 actually cost:
1350 ' 
1360 ' - idBuf$/nameBuf$/scoreBuf$ and the FIELD statement binding them had to
1370 ' be repeated, identically, in every OPEN block — get it wrong in one
1380 ' of the five and you're reading or writing the wrong bytes.
1390 ' - recLen% (50) is 2+20+8+20 computed by hand; add a field to the record
1400 ' and every one of those numbers has to be updated together, or the
1410 ' file silently gets corrupted.
1420 ' - Each field's pack/unpack call (mki$/cvi, mkd$/cvd, or nothing for
1430 ' strings) has to be matched to that field's type by hand, every time
1440 ' it's touched — nothing stops mkd$() being used on the id field.
1450 ' - There's no RTRIM$ builtin on real MBASIC/BASCOM, so reading a string
1460 ' field back means hand-rolling a trim loop (trimmed$, above) and
1470 ' remembering to call it, every time.
1480 ' - Alice's and Carol's updates are the identical GET/LSET/LSET/PUT
1490 ' pattern, typed out twice, with every buffer/field name repeated.
1500 ' 
1510 ' None of this is hard, exactly — it's just bookkeeping a compiler should
1520 ' be doing for you. Part 2 is the same program again, with BASCAL's
1530 ' record/file DSL doing that bookkeeping.
1540 ' ------------------------------------------------------------------------

1550 ' ============================================================
1560 ' Part 2 — the same program with the record / file DSL
1570 ' ============================================================
1580 ' 
1590 ' record <Name> ... end record
1600 ' Declares a fixed-layout record type. Supported field types: int16,
1610 ' int32, float32, float64, and string(N). The record's total byte width
1620 ' (used as Part 1's recLen%) is the sum of its field widths, computed
1630 ' automatically.
1640 ' 
1650 ' file <var> as <RecordType> = open(<path>)
1660 ' Opens (or creates) a random-access file sized for one record, and binds
1670 ' FIELD buffer variables for every field. File numbers are allocated
1680 ' automatically, starting at #1, in the order `file` declarations appear.
1690 ' This one line replaces Part 1's recLen% constant, OPEN, and FIELD.
1700 ' 
1710 ' <file>[<n>] = { field: value, ... }
1720 ' Whole-record write: packs every field (LSET, MKx$ for numeric fields)
1730 ' and writes record n. Every declared field must be given — a missing one
1740 ' is a compile-time error.
1750 ' 
1760 ' let <var> = <file>[<n>]
1770 ' Whole-record read: reads record n and unpacks every field (CVx for
1780 ' numeric fields, an inline trim loop like Part 1's trimmed$ for strings)
1790 ' into `<var>.<field>`.
1800 ' 
1810 ' <file>[<n>].<field> = value
1820 ' Partial update: GET, LSET just that one field, PUT. The one-field
1830 ' version of Part 1's Bob update, with no buffer names to get wrong.
1840 ' 
1850 ' <file>[<n>] = ?{ field: value, ... }
1860 ' Partial-record write: any subset of fields; unlisted ones are left
1870 ' untouched on disk. Whether a GET is needed is decided at *compile
1880 ' time* by comparing the given field names against the record's declared
1890 ' fields: some fields missing -> GET first, LSET just those fields, then
1900 ' PUT (this is Alice's update from Part 1, minus the GET/LSET/LSET/PUT
1910 ' spelled out by hand); every field given anyway -> no GET, same as a
1920 ' plain `{...}`. Unlike `{...}`, an *unknown* field name is still a
1930 ' compile-time error — only *missing* fields are allowed, not misspelled
1940 ' ones.
1950 ' 
1960 ' let <var> = <file>[<n>]
1970 ' <var>.<field> = value  (any number of times)
1980 ' <file>[<n>] = <var>
1990 ' Batched update: the `let` does one GET; each `<var>.<field> = value` is
2000 ' a pure in-memory assignment (no I/O); the final `<file>[<n>] = <var>`
2010 ' packs every field from `<var>` and does one PUT. This is Carol's update
2020 ' from Part 1 — same GET/LSET/LSET/PUT shape as `?{...}`, just spelled as
2030 ' read-mutate-write instead of a single literal, useful when the new
2040 ' values come from more than a one-line expression.
2050 ' 
2060 ' for <var> = <A> downto <B> ... end for
2070 ' Sugar for `for <var> = <A> to <B> step -1`.
2080 ' 
2090 ' <file>.close()
2100 ' Closes the file.

2110 ' file db as Student = open(...)  [50 bytes/record]
2120 OPEN "tutorial_records.dat" FOR RANDOM AS #1 LEN = 50
2130 FIELD #1, 2 AS dbIdBuf$, 20 AS dbNameBuf$, 8 AS dbScoreBuf$, 20 AS dbFacultyBuf$

2140 ' ---- Write three records ----

2150 ' Record 1: Alice, 95
2160 ' db[...] = { ... }  (whole-record write)
2170 LSET dbIdBuf$ = MKI$(1)
2180 LSET dbNameBuf$ = "Alice"
2190 LSET dbScoreBuf$ = MKD$(95)
2200 LSET dbFacultyBuf$ = "Engineering"
2210 PUT #1, 1

2220 ' Record 2: Bob, 54
2230 ' db[...] = { ... }  (whole-record write)
2240 LSET dbIdBuf$ = MKI$(2)
2250 LSET dbNameBuf$ = "Bob"
2260 LSET dbScoreBuf$ = MKD$(54)
2270 LSET dbFacultyBuf$ = "Arts"
2280 PUT #1, 2

2290 ' Record 3: Carol, 78
2300 ' db[...] = { ... }  (whole-record write)
2310 LSET dbIdBuf$ = MKI$(3)
2320 LSET dbNameBuf$ = "Carol"
2330 LSET dbScoreBuf$ = MKD$(78)
2340 LSET dbFacultyBuf$ = "Science"
2350 PUT #1, 3

2360 ' ---- Read records in reverse order ----

2370 PRINT "Part 2 (record/file DSL) -- reading records in reverse order:"

2380 FOR i% = 3 TO 1 STEP -1
2390     ' let s = db[...]  (whole-record read)
2400     GET #1, i%
2410     sid% = CVI(dbIdBuf$)
2420     snametrimi% = LEN(dbNameBuf$)
2430     IF (snametrimi% > 0) = 0 THEN GOTO 2470
2440     IF (MID$(dbNameBuf$, snametrimi%, 1) = " ") = 0 THEN GOTO 2470
2450         snametrimi% = snametrimi% - 1
2460         GOTO 2430
2470     REM END WHILE
2480     sname$ = LEFT$(dbNameBuf$, snametrimi%)
2490     sscore# = CVD(dbScoreBuf$)
2500     sfacultytrimi% = LEN(dbFacultyBuf$)
2510     IF (sfacultytrimi% > 0) = 0 THEN GOTO 2550
2520     IF (MID$(dbFacultyBuf$, sfacultytrimi%, 1) = " ") = 0 THEN GOTO 2550
2530         sfacultytrimi% = sfacultytrimi% - 1
2540         GOTO 2510
2550     REM END WHILE
2560     sfaculty$ = LEFT$(dbFacultyBuf$, sfacultytrimi%)
2570     PRINT (((("  [" + STR$(sid%)) + "] ") + sname$) + " -- ") + STR$(sscore#)
2580 NEXT i%

2590 ' ---- Update one field in place ----

2600 ' Bob just scraped a pass on re-mark. Compare to Part 1: no recLen%, no
2610 ' idBuf$/nameBuf$/scoreBuf$/facultyBuf$, no mkd$() — just the field that's changing.
2620 ' db[...].score = ...  (partial-field update)
2630 IF LOF(#1) < (2) * 50 THEN ERROR 63
2640 GET #1, 2
2650 LSET dbScoreBuf$ = MKD$(61.5)
2660 PUT #1, 2

2670 ' ---- Update two fields at once, still one GET and one PUT ----

2680 ' Alice got married and re-sat the exam. `name` and `score` don't cover
2690 ' every field of Student, so this needs an implicit GET first (id and
2700 ' faculty are preserved from the existing record) -- exactly Part 1's GET / LSET /
2710 ' LSET / PUT for Alice, minus having to write out the GET, the buffer
2720 ' names, or the packing calls. Which fields need a GET is worked out by
2730 ' the compiler by comparing `name`/`score` against Student's declared
2740 ' fields — not decided at runtime.
2750 ' db[...] = ?{ ... }  (partial-record write)
2760 IF LOF(#1) < (1) * 50 THEN ERROR 63
2770 GET #1, 1
2780 LSET dbNameBuf$ = "Alice Smith"
2790 LSET dbScoreBuf$ = MKD$(91)
2800 PUT #1, 1

2810 ' ---- Batched update: read once, mutate twice, write back once ----

2820 ' Carol changed her name and improved her score — the read-mutate-write
2830 ' spelling of the same one-GET-one-PUT update, useful when the new values
2840 ' aren't just a couple of literals.
2850 ' let carol = db[...]  (whole-record read)
2860 GET #1, 3
2870 carolid% = CVI(dbIdBuf$)
2880 carolnametrimi% = LEN(dbNameBuf$)
2890 IF (carolnametrimi% > 0) = 0 THEN GOTO 2930
2900 IF (MID$(dbNameBuf$, carolnametrimi%, 1) = " ") = 0 THEN GOTO 2930
2910     carolnametrimi% = carolnametrimi% - 1
2920     GOTO 2890
2930 REM END WHILE
2940 carolname$ = LEFT$(dbNameBuf$, carolnametrimi%)
2950 carolscore# = CVD(dbScoreBuf$)
2960 carolfacultytrimi% = LEN(dbFacultyBuf$)
2970 IF (carolfacultytrimi% > 0) = 0 THEN GOTO 3010
2980 IF (MID$(dbFacultyBuf$, carolfacultytrimi%, 1) = " ") = 0 THEN GOTO 3010
2990     carolfacultytrimi% = carolfacultytrimi% - 1
3000     GOTO 2970
3010 REM END WHILE
3020 carolfaculty$ = LEFT$(dbFacultyBuf$, carolfacultytrimi%)
3030 carolname$ = "Carol Jones"
3040 carolscore# = 88
3050 ' db[...] = carol  (write back a let-bound record)
3060 LSET dbIdBuf$ = MKI$(carolid%)
3070 LSET dbNameBuf$ = carolname$
3080 LSET dbScoreBuf$ = MKD$(carolscore#)
3090 LSET dbFacultyBuf$ = carolfaculty$
3100 PUT #1, 3

3110 ' ---- Verify the updates ----

3120 PRINT "Part 2 (record/file DSL) -- after updates:"

3130 FOR i% = 1 TO 3
3140     ' let s = db[...]  (whole-record read)
3150     GET #1, i%
3160     sid% = CVI(dbIdBuf$)
3170     snametrimi% = LEN(dbNameBuf$)
3180     IF (snametrimi% > 0) = 0 THEN GOTO 3220
3190     IF (MID$(dbNameBuf$, snametrimi%, 1) = " ") = 0 THEN GOTO 3220
3200         snametrimi% = snametrimi% - 1
3210         GOTO 3180
3220     REM END WHILE
3230     sname$ = LEFT$(dbNameBuf$, snametrimi%)
3240     sscore# = CVD(dbScoreBuf$)
3250     sfacultytrimi% = LEN(dbFacultyBuf$)
3260     IF (sfacultytrimi% > 0) = 0 THEN GOTO 3300
3270     IF (MID$(dbFacultyBuf$, sfacultytrimi%, 1) = " ") = 0 THEN GOTO 3300
3280         sfacultytrimi% = sfacultytrimi% - 1
3290         GOTO 3260
3300     REM END WHILE
3310     sfaculty$ = LEFT$(dbFacultyBuf$, sfacultytrimi%)
3320     PRINT (("  " + sname$) + ": ") + STR$(sscore#)
3330 NEXT i%

3340 ' db.close()
3350 CLOSE #1

3360 ' ------------------------------------------------------------------------
3370 ' Part 2 is the same three writes, the same reverse-order read, and the
3380 ' same three updates as Part 1 — Alice's and Bob's and Carol's updates
3390 ' still transpile to exactly one GET and one PUT each, nothing runs slower.
3400 ' What's gone is everything that was bookkeeping rather than logic: the
3410 ' hand-computed record width, the repeated buffer-variable/FIELD
3420 ' boilerplate in every block, the pack/unpack call picked by hand per
3430 ' field, and the GET-or-not decision for a partial write, which the
3440 ' compiler now makes for you at compile time by simply comparing field
3450 ' names -- get a field name wrong (`db[1] = ?{ nmae: ... }`) and it's a
3460 ' compile error instead of a silently corrupted record.
3470 ' ------------------------------------------------------------------------

3480 END

3490 ' function trimmed$(s$)
3500     trimmedI0% = LEN(trimmedS0$)
3510     IF (trimmedI0% > 0) = 0 THEN GOTO 3550
3520     IF (MID$(trimmedS0$, trimmedI0%, 1) = " ") = 0 THEN GOTO 3550
3530         trimmedI0% = trimmedI0% - 1
3540         GOTO 3510
3550     REM END WHILE
3560     trimmedResult0$ = LEFT$(trimmedS0$, trimmedI0%)
3570     RETURN
3580 ' end function trimmed$
