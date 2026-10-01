10 ' BASCAL generated BASIC -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Strips leading spaces from self$. Not a real MBASIC/BASCOM 2.00 builtin --
40 ' verified against a real IBM BASIC Compiler 2.00 under dosbox-x -- so
50 ' BASCAL ships its own. Declared as a scalar method (see GitHub issue #41)
60 ' so a required stdlib call reads the same way as a built-in method call
70 ' (docs/language/functions-and-procedures.html#built-in-methods). The
80 ' ordinary call form (ltrim$(s$)) still works -- a method's receiver is an
90 ' implicit first parameter, so ordinary-call syntax resolves straight to
100 ' this same declaration, with no separate function needed (and no longer
110 ' allowed: a function and a method sharing one name is a duplicate
120 ' declaration, since they'd both claim the same callable identity).
130 ' Strips trailing spaces from self$. Not a real MBASIC/BASCOM 2.00 builtin --
140 ' verified against a real IBM BASIC Compiler 2.00 under dosbox-x -- so
150 ' BASCAL ships its own. Declared as a scalar method (see GitHub issue #41
160 ' and ltrim.bcl's own doc comment for the reasoning) -- rtrim$(s$) still
170 ' works via ordinary-call syntax resolving to this same declaration.
180 ' Tutorial — Random-Access Files: hand-written, then with the record/file DSL
190 ' 
200 ' This tutorial writes the *same* program twice. Part 1 uses BASIC's raw
210 ' random-access file primitives directly. Part 2 uses BASCAL's `record`/
220 ' `file` DSL, which transpiles to exactly the same primitives — nothing about
230 ' the *generated* BASIC changes, only how much of it you have to write by
240 ' hand. Read Part 1 first; the comments between the two parts explain what
250 ' the DSL is buying you and why.
260 ' 
270 ' ---- Part 1 primitives ----
280 ' 
290 ' open filename$ for random as #n len = REC_LEN
300 ' Open (or create) a random-access file.  len specifies the record length
310 ' in bytes; every record occupies exactly that many bytes.
320 ' 
330 ' field #n, width1% as var1$, width2% as var2$, ...
340 ' Bind string variables to regions of the file buffer.  The sum of all
350 ' widths must equal the record length.  Only string variables may be used
360 ' in a FIELD statement.
370 ' 
380 ' lset var$ = expr$   — copy into a field buffer, left-justified (padded)
390 ' rset var$ = expr$   — copy into a field buffer, right-justified (padded)
400 ' 
410 ' put #n, recordNumber%   — write the current buffer as record n (1-based)
420 ' get #n, recordNumber%   — read record n into the buffer variables
430 ' 
440 ' Packing helpers (BASIC builtins):
450 ' mki$(n%)  — pack a 2-byte integer into a 2-character string
460 ' mkl$(n&)  — pack a 4-byte long
470 ' mks$(n!)  — pack a 4-byte single
480 ' mkd$(n#)  — pack an 8-byte double
490 ' cvi(s$)   — unpack a 2-byte integer from a string
500 ' cvl(s$)   — unpack a 4-byte long
510 ' cvs(s$)   — unpack a 4-byte single
520 ' cvd(s$)   — unpack an 8-byte double
530 ' 
540 ' Every MKx$ always returns a string (never a type-suffixed MKI%/MKD#/etc —
550 ' those aren't real MBASIC/BASCOM functions), and every CVx takes no suffix
560 ' at all. There's also no RTRIM$ builtin on real MBASIC/BASCOM -- trimming a
570 ' fixed-width, space-padded FIELD buffer back down to its real length needs
580 ' a hand-rolled loop, like trimmed$ below.
590 ' trimmed$ -- right-trim trailing spaces from a fixed-width FIELD buffer.
600 CONSTRECLEN% = 50
610 ' 2 bytes id + 20 bytes name + 8 bytes score + 20 bytes faculty
620 CONSTNUMRECS% = 3
630 CONSTDBFILE$ = "tutorial_students.dat"
640 ' ============================================================
650 ' Part 1 — random-access files, written by hand
660 ' ============================================================
670 ' ---- Write three records ----
680 OPEN CONSTDBFILE$ FOR RANDOM AS #1 LEN = CONSTRECLEN%
690 FIELD #1, 2 AS idbuf$, 20 AS namebuf$, 8 AS scorebuf$, 20 AS facultybuf$
700 ' Record 1: Alice, 95
710 LSET idbuf$ = MKI$(1)
720 LSET namebuf$ = "Alice"
730 LSET scorebuf$ = MKD$(95.0)
740 LSET facultybuf$ = "Engineering"
750 PUT #1, 1
760 ' Record 2: Bob, 54
770 LSET idbuf$ = MKI$(2)
780 LSET namebuf$ = "Bob"
790 LSET scorebuf$ = MKD$(54.0)
800 LSET facultybuf$ = "Arts"
810 PUT #1, 2
820 ' Record 3: Carol, 78
830 LSET idbuf$ = MKI$(3)
840 LSET namebuf$ = "Carol"
850 LSET scorebuf$ = MKD$(78.0)
860 LSET facultybuf$ = "Science"
870 PUT #1, 3
880 CLOSE #1
890 ' ---- Read records in reverse order ----
900 PRINT "Part 1 (hand-written) -- reading records in reverse order:"
910 OPEN CONSTDBFILE$ FOR RANDOM AS #1 LEN = CONSTRECLEN%
920 FIELD #1, 2 AS idbuf$, 20 AS namebuf$, 8 AS scorebuf$, 20 AS facultybuf$
930 FOR i% = CONSTNUMRECS% TO 1 STEP -1
940     GET #1, i%
950     id% = CVI(idbuf$)
960     score# = CVD(scorebuf$)
970     BCCT2% = id%
980     BCCT3$ = STR$(BCCT2%)
990     trimmedS0$ = namebuf$
1000     GOSUB 3840
1010     BCCT4$ = trimmedResult0$
1020     BCCT5# = score#
1030     BCCT6$ = STR$(BCCT5#)
1040     PRINT (((("  [" + BCCT3$) + "] ") + BCCT4$) + " -- ") + BCCT6$
1050 NEXT i%
1060 CLOSE #1
1070 ' ---- Update one field in place ----
1080 OPEN CONSTDBFILE$ FOR RANDOM AS #1 LEN = CONSTRECLEN%
1090 FIELD #1, 2 AS idbuf$, 20 AS namebuf$, 8 AS scorebuf$, 20 AS facultybuf$
1100 ' Bob just scraped a pass on re-mark. Only scoreBuf$ changes, but PUT
1110 ' always writes the whole 50-byte buffer, so GET has to load the record
1120 ' first even though idBuf$/nameBuf$/facultyBuf$ are just being written straight back
1130 ' unchanged.
1140 GET #1, 2
1150 LSET scorebuf$ = MKD$(61.5)
1160 PUT #1, 2
1170 CLOSE #1
1180 ' ---- Update two fields at once ----
1190 OPEN CONSTDBFILE$ FOR RANDOM AS #1 LEN = CONSTRECLEN%
1200 FIELD #1, 2 AS idbuf$, 20 AS namebuf$, 8 AS scorebuf$, 20 AS facultybuf$
1210 ' Alice got married and re-sat the exam — `name` and `score` both change,
1220 ' `id` and `faculty` don't. Same problem as Bob's update, just with two fields instead
1230 ' of one: GET first (this is what preserves idBuf$ and facultyBuf$), LSET the two fields
1240 ' that actually changed, then PUT the whole buffer back. Nothing here is
1250 ' specific to "two" fields — five changed fields would look identical,
1260 ' just with five LSET lines between the GET and the PUT.
1270 GET #1, 1
1280 LSET namebuf$ = "Alice Smith"
1290 LSET scorebuf$ = MKD$(91.0)
1300 PUT #1, 1
1310 CLOSE #1
1320 ' ---- Same shape again ----
1330 OPEN CONSTDBFILE$ FOR RANDOM AS #1 LEN = CONSTRECLEN%
1340 FIELD #1, 2 AS idbuf$, 20 AS namebuf$, 8 AS scorebuf$, 20 AS facultybuf$
1350 ' Carol changed her name and improved her score: the exact same
1360 ' GET / LSET / LSET / PUT shape as Alice's update above, just retyped by
1370 ' hand with Carol's record number and values.
1380 GET #1, 3
1390 LSET namebuf$ = "Carol Jones"
1400 LSET scorebuf$ = MKD$(88.0)
1410 PUT #1, 3
1420 CLOSE #1
1430 ' ---- Verify the updates ----
1440 PRINT "Part 1 (hand-written) -- after updates:"
1450 OPEN CONSTDBFILE$ FOR RANDOM AS #1 LEN = CONSTRECLEN%
1460 FIELD #1, 2 AS idbuf$, 20 AS namebuf$, 8 AS scorebuf$, 20 AS facultybuf$
1470 FOR i% = 1 TO CONSTNUMRECS%
1480     GET #1, i%
1490     trimmedS0$ = namebuf$
1500     GOSUB 3840
1510     BCCT8$ = trimmedResult0$
1520     BCCT9$ = scorebuf$
1530     BCCT10# = CVD(BCCT9$)
1540     BCCT11# = BCCT10#
1550     BCCT12$ = STR$(BCCT11#)
1560     PRINT (("  " + BCCT8$) + ": ") + BCCT12$
1570 NEXT i%
1580 CLOSE #1
1590 ' ------------------------------------------------------------------------
1600 ' What Part 1 actually cost:
1610 ' 
1620 ' - idBuf$/nameBuf$/scoreBuf$ and the FIELD statement binding them had to
1630 ' be repeated, identically, in every OPEN block — get it wrong in one
1640 ' of the five and you're reading or writing the wrong bytes.
1650 ' - REC_LEN (50) is 2+20+8+20 computed by hand; add a field to the record
1660 ' and every one of those numbers has to be updated together, or the
1670 ' file silently gets corrupted.
1680 ' - Each field's pack/unpack call (mki$/cvi, mkd$/cvd, or nothing for
1690 ' strings) has to be matched to that field's type by hand, every time
1700 ' it's touched — nothing stops mkd$() being used on the id field.
1710 ' - There's no RTRIM$ builtin on real MBASIC/BASCOM, so reading a string
1720 ' field back means hand-rolling a trim loop (trimmed$, above) and
1730 ' remembering to call it, every time.
1740 ' - Alice's and Carol's updates are the identical GET/LSET/LSET/PUT
1750 ' pattern, typed out twice, with every buffer/field name repeated.
1760 ' 
1770 ' None of this is hard, exactly — it's just bookkeeping a compiler should
1780 ' be doing for you. Part 2 is the same program again, with BASCAL's
1790 ' record/file DSL doing that bookkeeping.
1800 ' ------------------------------------------------------------------------
1810 ' ============================================================
1820 ' Part 2 — the same program with the record / file DSL
1830 ' ============================================================
1840 ' 
1850 ' record <Name> ... end record
1860 ' Declares a fixed-layout record type. Supported field types: int16,
1870 ' int32, float32, float64, and string(N). The record's total byte width
1880 ' (used as Part 1's REC_LEN) is the sum of its field widths, computed
1890 ' automatically.
1900 ' 
1910 ' file <var> as <RecordType> = open(<path>)
1920 ' Opens (or creates) a random-access file sized for one record, and binds
1930 ' FIELD buffer variables for every field. File numbers are allocated
1940 ' automatically, starting at #1, in the order `file` declarations appear.
1950 ' This one line replaces Part 1's REC_LEN constant, OPEN, and FIELD.
1960 ' 
1970 ' <file>[<n>] = { field: value, ... }
1980 ' Whole-record write: packs every field (LSET, MKx$ for numeric fields)
1990 ' and writes record n. Every declared field must be given — a missing one
2000 ' is a compile-time error.
2010 ' 
2020 ' let <var> = <file>[<n>]
2030 ' Whole-record read: reads record n and unpacks every field (CVx for
2040 ' numeric fields, an inline trim loop like Part 1's trimmed$ for strings)
2050 ' into `<var>.<field>`.
2060 ' 
2070 ' <file>[<n>].<field> = value
2080 ' Partial update: GET, LSET just that one field, PUT. The one-field
2090 ' version of Part 1's Bob update, with no buffer names to get wrong.
2100 ' 
2110 ' <file>[<n>] = ?{ field: value, ... }
2120 ' Partial-record write: any subset of fields; unlisted ones are left
2130 ' untouched on disk. Whether a GET is needed is decided at *compile
2140 ' time* by comparing the given field names against the record's declared
2150 ' fields: some fields missing -> GET first, LSET just those fields, then
2160 ' PUT (this is Alice's update from Part 1, minus the GET/LSET/LSET/PUT
2170 ' spelled out by hand); every field given anyway -> no GET, same as a
2180 ' plain `{...}`. Unlike `{...}`, an *unknown* field name is still a
2190 ' compile-time error — only *missing* fields are allowed, not misspelled
2200 ' ones.
2210 ' 
2220 ' let <var> = <file>[<n>]
2230 ' <var>.<field> = value  (any number of times)
2240 ' <file>[<n>] = <var>
2250 ' Batched update: the `let` does one GET; each `<var>.<field> = value` is
2260 ' a pure in-memory assignment (no I/O); the final `<file>[<n>] = <var>`
2270 ' packs every field from `<var>` and does one PUT. This is Carol's update
2280 ' from Part 1 — same GET/LSET/LSET/PUT shape as `?{...}`, just spelled as
2290 ' read-mutate-write instead of a single literal, useful when the new
2300 ' values come from more than a one-line expression.
2310 ' 
2320 ' for <var> = <A> downto <B> ... end for
2330 ' Sugar for `for <var> = <A> to <B> step -1`.
2340 ' 
2350 ' <file>.close()
2360 ' Closes the file.
2370 ' file db as Student = open(...)  [50 bytes/record]
2380 OPEN "tutorial_records.dat" FOR RANDOM AS #1 LEN = 50
2390 FIELD #1, 2 AS dbIdBuf$, 20 AS dbNameBuf$, 8 AS dbScoreBuf$, 20 AS dbFacultyBuf$
2400 ' ---- Write three records ----
2410 ' Record 1: Alice, 95
2420 ' db[...] = { ... }  (whole-record write)
2430 LSET dbIdBuf$ = MKI$(1)
2440 LSET dbNameBuf$ = "Alice"
2450 LSET dbScoreBuf$ = MKD$(95.0)
2460 LSET dbFacultyBuf$ = "Engineering"
2470 PUT #1, 1
2480 ' Record 2: Bob, 54
2490 ' db[...] = { ... }  (whole-record write)
2500 LSET dbIdBuf$ = MKI$(2)
2510 LSET dbNameBuf$ = "Bob"
2520 LSET dbScoreBuf$ = MKD$(54.0)
2530 LSET dbFacultyBuf$ = "Arts"
2540 PUT #1, 2
2550 ' Record 3: Carol, 78
2560 ' db[...] = { ... }  (whole-record write)
2570 LSET dbIdBuf$ = MKI$(3)
2580 LSET dbNameBuf$ = "Carol"
2590 LSET dbScoreBuf$ = MKD$(78.0)
2600 LSET dbFacultyBuf$ = "Science"
2610 PUT #1, 3
2620 ' ---- Read records in reverse order ----
2630 PRINT "Part 2 (record/file DSL) -- reading records in reverse order:"
2640 FOR i% = 3 TO 1 STEP -1
2650     ' let s = db[...]  (whole-record read)
2660     GET #1, i%
2670     sid% = CVI(dbIdBuf$)
2680     rtrimSelf0$ = dbNameBuf$
2690     GOSUB 3710
2700     BCCT14$ = rtrimResult0$
2710     sname$ = BCCT14$
2720     sscore# = CVD(dbScoreBuf$)
2730     rtrimSelf0$ = dbFacultyBuf$
2740     GOSUB 3710
2750     BCCT15$ = rtrimResult0$
2760     sfaculty$ = BCCT15$
2770     PRINT (((("  [" + STR$(sid%)) + "] ") + sname$) + " -- ") + STR$(sscore#)
2780 NEXT i%
2790 ' ---- Update one field in place ----
2800 ' Bob just scraped a pass on re-mark. Compare to Part 1: no REC_LEN, no
2810 ' idBuf$/nameBuf$/scoreBuf$/facultyBuf$, no mkd$() — just the field that's changing.
2820 ' db[...].score = ...  (partial-field update)
2830 IF LOF(#1) < (2) * 50 THEN ERROR 63
2840 GET #1, 2
2850 LSET dbScoreBuf$ = MKD$(61.5)
2860 PUT #1, 2
2870 ' ---- Update two fields at once, still one GET and one PUT ----
2880 ' Alice got married and re-sat the exam. `name` and `score` don't cover
2890 ' every field of Student, so this needs an implicit GET first (id and
2900 ' faculty are preserved from the existing record) -- exactly Part 1's GET / LSET /
2910 ' LSET / PUT for Alice, minus having to write out the GET, the buffer
2920 ' names, or the packing calls. Which fields need a GET is worked out by
2930 ' the compiler by comparing `name`/`score` against Student's declared
2940 ' fields — not decided at runtime.
2950 ' db[...] = ?{ ... }  (partial-record write)
2960 IF LOF(#1) < (1) * 50 THEN ERROR 63
2970 GET #1, 1
2980 LSET dbNameBuf$ = "Alice Smith"
2990 LSET dbScoreBuf$ = MKD$(91.0)
3000 PUT #1, 1
3010 ' ---- Batched update: read once, mutate twice, write back once ----
3020 ' Carol changed her name and improved her score — the read-mutate-write
3030 ' spelling of the same one-GET-one-PUT update, useful when the new values
3040 ' aren't just a couple of literals.
3050 ' let carol = db[...]  (whole-record read)
3060 GET #1, 3
3070 carolid% = CVI(dbIdBuf$)
3080 rtrimSelf0$ = dbNameBuf$
3090 GOSUB 3710
3100 BCCT16$ = rtrimResult0$
3110 carolname$ = BCCT16$
3120 carolscore# = CVD(dbScoreBuf$)
3130 rtrimSelf0$ = dbFacultyBuf$
3140 GOSUB 3710
3150 BCCT17$ = rtrimResult0$
3160 carolfaculty$ = BCCT17$
3170 carolname$ = "Carol Jones"
3180 carolscore# = 88.0
3190 ' db[...] = carol  (write back a let-bound record)
3200 LSET dbIdBuf$ = MKI$(carolid%)
3210 LSET dbNameBuf$ = carolname$
3220 LSET dbScoreBuf$ = MKD$(carolscore#)
3230 LSET dbFacultyBuf$ = carolfaculty$
3240 PUT #1, 3
3250 ' ---- Verify the updates ----
3260 PRINT "Part 2 (record/file DSL) -- after updates:"
3270 FOR i% = 1 TO 3
3280     ' let s = db[...]  (whole-record read)
3290     GET #1, i%
3300     sid% = CVI(dbIdBuf$)
3310     rtrimSelf0$ = dbNameBuf$
3320     GOSUB 3710
3330     BCCT19$ = rtrimResult0$
3340     sname$ = BCCT19$
3350     sscore# = CVD(dbScoreBuf$)
3360     rtrimSelf0$ = dbFacultyBuf$
3370     GOSUB 3710
3380     BCCT20$ = rtrimResult0$
3390     sfaculty$ = BCCT20$
3400     PRINT (("  " + sname$) + ": ") + STR$(sscore#)
3410 NEXT i%
3420 ' db.close()
3430 CLOSE #1
3440 ' ------------------------------------------------------------------------
3450 ' Part 2 is the same three writes, the same reverse-order read, and the
3460 ' same three updates as Part 1 — Alice's and Bob's and Carol's updates
3470 ' still transpile to exactly one GET and one PUT each, nothing runs slower.
3480 ' What's gone is everything that was bookkeeping rather than logic: the
3490 ' hand-computed record width, the repeated buffer-variable/FIELD
3500 ' boilerplate in every block, the pack/unpack call picked by hand per
3510 ' field, and the GET-or-not decision for a partial write, which the
3520 ' compiler now makes for you at compile time by simply comparing field
3530 ' names -- get a field name wrong (`db[1] = ?{ nmae: ... }`) and it's a
3540 ' compile error instead of a silently corrupted record.
3550 ' ------------------------------------------------------------------------
3560 END

3570 ' function ltrim$(self$)
3580     ltrimI0% = 1
3590     IF (ltrimI0% <= LEN(ltrimSelf0$)) = 0 THEN GOTO 3630
3600     IF (MID$(ltrimSelf0$, ltrimI0%, 1) = " ") = 0 THEN GOTO 3630
3610         ltrimI0% = ltrimI0% + 1
3620     GOTO 3590
3630     REM END WHILE
3640     BCCT23$ = ltrimSelf0$
3650     BCCT24% = ltrimI0%
3660     BCCT25$ = MID$(BCCT23$, BCCT24%)
3670     ltrimResult0$ = BCCT25$
3680     RETURN
3690 ' end function ltrim$

3700 ' function rtrim$(self$)
3710     rtrimI0% = LEN(rtrimSelf0$)
3720     IF (rtrimI0% > 0) = 0 THEN GOTO 3760
3730     IF (MID$(rtrimSelf0$, rtrimI0%, 1) = " ") = 0 THEN GOTO 3760
3740         rtrimI0% = rtrimI0% - 1
3750     GOTO 3720
3760     REM END WHILE
3770     BCCT28$ = rtrimSelf0$
3780     BCCT29% = rtrimI0%
3790     BCCT30$ = LEFT$(BCCT28$, BCCT29%)
3800     rtrimResult0$ = BCCT30$
3810     RETURN
3820 ' end function rtrim$

3830 ' function trimmed$(s$)
3840     trimmedI0% = LEN(trimmedS0$)
3850     IF (trimmedI0% > 0) = 0 THEN GOTO 3890
3860     IF (MID$(trimmedS0$, trimmedI0%, 1) = " ") = 0 THEN GOTO 3890
3870         trimmedI0% = trimmedI0% - 1
3880     GOTO 3850
3890     REM END WHILE
3900     BCCT33$ = trimmedS0$
3910     BCCT34% = trimmedI0%
3920     BCCT35$ = LEFT$(BCCT33$, BCCT34%)
3930     trimmedResult0$ = BCCT35$
3940     RETURN
3950 ' end function trimmed$
