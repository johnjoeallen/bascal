[Home](../../) / [Tutorials](../) / Random-Access and Record Files

<div class="prose" markdown="1">

This tutorial writes one program twice: first with classic random-access primitives (`OPEN ... FOR RANDOM`, `FIELD`, `GET`/`PUT`, `LSET`, `MKx`/`CVx`), then again with BASCAL's typed `record`/`file` syntax, which generates exactly that primitive code for you. The [record-files chapter](../home/record-files.md) introduces the `?{ ... }` partial-write syntax; this page shows where the two halves of the file meet.

</div>

<div class="snippet" markdown="1">

### Declaring the record shape

Replaces a hand-computed byte width and a matching FIELD statement.

```bascal
record Student
    id:    int16
    name:  string(20)
    score: float64
    faculty: string(20)
end record

file db as Student = open("students.dat")
```

</div>

<div class="snippet" markdown="1">

### A partial update, still one GET and one PUT

See the homepage for why this uses the `?{ ... }` syntax.

```bascal
db[1] = ?{ name: "Alice Smith", score: 91.0 }
```

</div>



[← Procedures](procedures.md)  ·  [Short-Circuit && and \|\| →](short_circuit.md)


<!-- BEGIN generated tutorial source -->

<details class="source-embed" markdown="1">

<summary><code>tutorial/random_and_record_files.bcl</code></summary>



```bascal

// Tutorial — Random-Access Files: hand-written, then with the record/file DSL
//
// This tutorial writes the *same* program twice. Part 1 uses BASIC's raw
// random-access file primitives directly. Part 2 uses BASCAL's `record`/
// `file` DSL, which transpiles to exactly the same primitives — nothing about
// the *generated* BASIC changes, only how much of it you have to write by
// hand. Read Part 1 first; the comments between the two parts explain what
// the DSL is buying you and why.
//
// ---- Part 1 primitives ----
//
// open filename$ for random as #n len = REC_LEN
//   Open (or create) a random-access file.  len specifies the record length
//   in bytes; every record occupies exactly that many bytes.
//
// field #n, width1% as var1$, width2% as var2$, ...
//   Bind string variables to regions of the file buffer.  The sum of all
//   widths must equal the record length.  Only string variables may be used
//   in a FIELD statement.
//
// lset var$ = expr$   — copy into a field buffer, left-justified (padded)
// rset var$ = expr$   — copy into a field buffer, right-justified (padded)
//
// put #n, recordNumber%   — write the current buffer as record n (1-based)
// get #n, recordNumber%   — read record n into the buffer variables
//
// Packing helpers (BASIC builtins):
//   mki$(n%)  — pack a 2-byte integer into a 2-character string
//   mkl$(n&)  — pack a 4-byte long
//   mks$(n!)  — pack a 4-byte single
//   mkd$(n#)  — pack an 8-byte double
//   cvi(s$)   — unpack a 2-byte integer from a string
//   cvl(s$)   — unpack a 4-byte long
//   cvs(s$)   — unpack a 4-byte single
//   cvd(s$)   — unpack an 8-byte double
//
// Every MKx$ always returns a string (never a type-suffixed MKI%/MKD#/etc —
// those aren't real MBASIC/BASCOM functions), and every CVx takes no suffix
// at all. There's also no RTRIM$ builtin on real MBASIC/BASCOM -- trimming a
// fixed-width, space-padded FIELD buffer back down to its real length needs
// a hand-rolled loop, like trimmed$ below.
program randomAndRecordFiles

// trimmed$ -- right-trim trailing spaces from a fixed-width FIELD buffer.
function trimmed$(s$)
    i% = s$.len()
    while i% > 0 && s$.mid(i%, 1) = " "
        i% = i% - 1
    end while
    return s$.left(i%)
end function

const REC_LEN = 50 // 2 bytes id + 20 bytes name + 8 bytes score + 20 bytes faculty
const NUM_RECS = 3
const DB_FILE = "tutorial_students.dat"

/* ============================================================ */
/* Part 1 — random-access files, written by hand                */
/* ============================================================ */

/* ---- Write three records ---- */

open DB_FILE for random as #1 len = REC_LEN
field #1, 2 as idBuf$, 20 as nameBuf$, 8 as scoreBuf$, 20 as facultyBuf$

// Record 1: Alice, 95
lset idBuf$ = mki$(1)
lset nameBuf$ = "Alice"
lset scoreBuf$ = mkd$(95.0)
lset facultyBuf$ = "Engineering"
put #1, 1

// Record 2: Bob, 54
lset idBuf$ = mki$(2)
lset nameBuf$ = "Bob"
lset scoreBuf$ = mkd$(54.0)
lset facultyBuf$ = "Arts"
put #1, 2

// Record 3: Carol, 78
lset idBuf$ = mki$(3)
lset nameBuf$ = "Carol"
lset scoreBuf$ = mkd$(78.0)
lset facultyBuf$ = "Science"
put #1, 3

close #1

/* ---- Read records in reverse order ---- */

print "Part 1 (hand-written) -- reading records in reverse order:"
open DB_FILE for random as #1 len = REC_LEN
field #1, 2 as idBuf$, 20 as nameBuf$, 8 as scoreBuf$, 20 as facultyBuf$

for i% = NUM_RECS to 1 step -1
    get #1, i%
    id% = cvi(idBuf$)
    score# = cvd(scoreBuf$)
    print "  [" + str$(id%) + "] " + trimmed$(nameBuf$) + " -- " + str$(score#)
end for

close #1

/* ---- Update one field in place ---- */

open DB_FILE for random as #1 len = REC_LEN
field #1, 2 as idBuf$, 20 as nameBuf$, 8 as scoreBuf$, 20 as facultyBuf$

// Bob just scraped a pass on re-mark. Only scoreBuf$ changes, but PUT
// always writes the whole 50-byte buffer, so GET has to load the record
// first even though idBuf$/nameBuf$/facultyBuf$ are just being written straight back
// unchanged.
get #1, 2
lset scoreBuf$ = mkd$(61.5)
put #1, 2

close #1

/* ---- Update two fields at once ---- */

open DB_FILE for random as #1 len = REC_LEN
field #1, 2 as idBuf$, 20 as nameBuf$, 8 as scoreBuf$, 20 as facultyBuf$

// Alice got married and re-sat the exam — `name` and `score` both change,
// `id` and `faculty` don't. Same problem as Bob's update, just with two fields instead
// of one: GET first (this is what preserves idBuf$ and facultyBuf$), LSET the two fields
// that actually changed, then PUT the whole buffer back. Nothing here is
// specific to "two" fields — five changed fields would look identical,
// just with five LSET lines between the GET and the PUT.
get #1, 1
lset nameBuf$ = "Alice Smith"
lset scoreBuf$ = mkd$(91.0)
put #1, 1

close #1

/* ---- Same shape again ---- */

open DB_FILE for random as #1 len = REC_LEN
field #1, 2 as idBuf$, 20 as nameBuf$, 8 as scoreBuf$, 20 as facultyBuf$

// Carol changed her name and improved her score: the exact same
// GET / LSET / LSET / PUT shape as Alice's update above, just retyped by
// hand with Carol's record number and values.
get #1, 3
lset nameBuf$ = "Carol Jones"
lset scoreBuf$ = mkd$(88.0)
put #1, 3

close #1

/* ---- Verify the updates ---- */

print "Part 1 (hand-written) -- after updates:"
open DB_FILE for random as #1 len = REC_LEN
field #1, 2 as idBuf$, 20 as nameBuf$, 8 as scoreBuf$, 20 as facultyBuf$

for i% = 1 to NUM_RECS
    get #1, i%
    print "  " + trimmed$(nameBuf$) + ": " + str$(cvd(scoreBuf$))
end for

close #1

// ------------------------------------------------------------------------
// What Part 1 actually cost:
//
//   - idBuf$/nameBuf$/scoreBuf$ and the FIELD statement binding them had to
//     be repeated, identically, in every OPEN block — get it wrong in one
//     of the five and you're reading or writing the wrong bytes.
//   - REC_LEN (50) is 2+20+8+20 computed by hand; add a field to the record
//     and every one of those numbers has to be updated together, or the
//     file silently gets corrupted.
//   - Each field's pack/unpack call (mki$/cvi, mkd$/cvd, or nothing for
//     strings) has to be matched to that field's type by hand, every time
//     it's touched — nothing stops mkd$() being used on the id field.
//   - There's no RTRIM$ builtin on real MBASIC/BASCOM, so reading a string
//     field back means hand-rolling a trim loop (trimmed$, above) and
//     remembering to call it, every time.
//   - Alice's and Carol's updates are the identical GET/LSET/LSET/PUT
//     pattern, typed out twice, with every buffer/field name repeated.
//
// None of this is hard, exactly — it's just bookkeeping a compiler should
// be doing for you. Part 2 is the same program again, with BASCAL's
// record/file DSL doing that bookkeeping.
// ------------------------------------------------------------------------

/* ============================================================ */
/* Part 2 — the same program with the record / file DSL         */
/* ============================================================ */
//
// record <Name> ... end record
//   Declares a fixed-layout record type. Supported field types: int16,
//   int32, float32, float64, and string(N). The record's total byte width
//   (used as Part 1's REC_LEN) is the sum of its field widths, computed
//   automatically.
//
// file <var> as <RecordType> = open(<path>)
//   Opens (or creates) a random-access file sized for one record, and binds
//   FIELD buffer variables for every field. File numbers are allocated
//   automatically, starting at #1, in the order `file` declarations appear.
//   This one line replaces Part 1's REC_LEN constant, OPEN, and FIELD.
//
// <file>[<n>] = { field: value, ... }
//   Whole-record write: packs every field (LSET, MKx$ for numeric fields)
//   and writes record n. Every declared field must be given — a missing one
//   is a compile-time error.
//
// let <var> = <file>[<n>]
//   Whole-record read: reads record n and unpacks every field (CVx for
//   numeric fields, an inline trim loop like Part 1's trimmed$ for strings)
//   into `<var>.<field>`.
//
// <file>[<n>].<field> = value
//   Partial update: GET, LSET just that one field, PUT. The one-field
//   version of Part 1's Bob update, with no buffer names to get wrong.
//
// <file>[<n>] = ?{ field: value, ... }
//   Partial-record write: any subset of fields; unlisted ones are left
//   untouched on disk. Whether a GET is needed is decided at *compile
//   time* by comparing the given field names against the record's declared
//   fields: some fields missing -> GET first, LSET just those fields, then
//   PUT (this is Alice's update from Part 1, minus the GET/LSET/LSET/PUT
//   written out by hand); every field given anyway -> no GET, same as a
//   plain `{...}`. Unlike `{...}`, an *unknown* field name is still a
//   compile-time error — only *missing* fields are allowed, not misspelled
//   ones.
//
// let <var> = <file>[<n>]
// <var>.<field> = value  (any number of times)
// <file>[<n>] = <var>
//   Batched update: the `let` does one GET; each `<var>.<field> = value` is
//   a pure in-memory assignment (no I/O); the final `<file>[<n>] = <var>`
//   packs every field from `<var>` and does one PUT. This is Carol's update
//   from Part 1 — same GET/LSET/LSET/PUT shape as `?{...}`, just written as
//   read-mutate-write instead of a single literal, useful when the new
//   values come from more than a one-line expression.
//
// for <var> = <A> downto <B> ... end for
//   Sugar for `for <var> = <A> to <B> step -1`.
//
// <file>.close()
//   Closes the file.

record Student
    id: int16
    name: string(20) lpad
    score: float64
    faculty: string(20) lpad
end record

file db as Student = open("tutorial_records.dat")

/* ---- Write three records ---- */

// Record 1: Alice, 95
db[1] = { id: 1, name: "Alice", score: 95.0, faculty: "Engineering" }

// Record 2: Bob, 54
db[2] = { id: 2, name: "Bob", score: 54.0, faculty: "Arts" }

// Record 3: Carol, 78
db[3] = { id: 3, name: "Carol", score: 78.0, faculty: "Science" }

/* ---- Read records in reverse order ---- */

print "Part 2 (record/file DSL) -- reading records in reverse order:"

for i% = 3 downto 1
    let s = db[i%]
    print "  [" + s.id + "] " + s.name + " -- " + s.score
end for

/* ---- Update one field in place ---- */

// Bob just scraped a pass on re-mark. Compare to Part 1: no REC_LEN, no
// idBuf$/nameBuf$/scoreBuf$/facultyBuf$, no mkd$() — just the field that's changing.
db[2].score = 61.5

/* ---- Update two fields at once, still one GET and one PUT ---- */

// Alice got married and re-sat the exam. `name` and `score` don't cover
// every field of Student, so this needs an implicit GET first (id and
// faculty are preserved from the existing record) -- exactly Part 1's GET / LSET /
// LSET / PUT for Alice, minus having to write out the GET, the buffer
// names, or the packing calls. Which fields need a GET is worked out by
// the compiler by comparing `name`/`score` against Student's declared
// fields — not decided at runtime.
db[1] = ?{ name: "Alice Smith", score: 91.0 }

/* ---- Batched update: read once, mutate twice, write back once ---- */

// Carol changed her name and improved her score — the read-mutate-write
// form of the same one-GET-one-PUT update, useful when the new values
// aren't just a couple of literals.
let carol = db[3]
carol.name = "Carol Jones"
carol.score = 88.0
db[3] = carol

/* ---- Verify the updates ---- */

print "Part 2 (record/file DSL) -- after updates:"

for i% = 1 to 3
    let s = db[i%]
    print "  " + s.name + ": " + s.score
end for

db.close()

// ------------------------------------------------------------------------
// Part 2 is the same three writes, the same reverse-order read, and the
// same three updates as Part 1 — Alice's and Bob's and Carol's updates
// still transpile to exactly one GET and one PUT each, nothing runs slower.
// What's gone is everything that was bookkeeping rather than logic: the
// hand-computed record width, the repeated buffer-variable/FIELD
// boilerplate in every block, the pack/unpack call picked by hand per
// field, and the GET-or-not decision for a partial write, which the
// compiler now makes for you at compile time by simply comparing field
// names -- get a field name wrong (`db[1] = ?{ nmae: ... }`) and it's a
// compile error instead of a silently corrupted record.
// ------------------------------------------------------------------------

end

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/random_and_record_files.bas</code></summary>



```basic

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
2170 ' written out by hand); every field given anyway -> no GET, same as a
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
2280 ' from Part 1 — same GET/LSET/LSET/PUT shape as `?{...}`, just written as
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
3030 ' form of the same one-GET-one-PUT update, useful when the new values
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

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/random_and_record_files.c</code></summary>



```c

// BASCAL generated C -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
#include <stdio.h>
#include <string.h>
#include <stdint.h>
#include <stdlib.h>

#define BCC_STRBUF_COUNT 8
static char bcc_strbuf[BCC_STRBUF_COUNT][256];
static int bcc_strbuf_next = 0;

static int bcc_err = 0;
static int bcc_on_error_target = -1;
static int bcc_in_handler = 0;
static int bcc_resume_id = -1;
static int bcc_erl = 0;
static const char *bcc_err_file = "";

#define BCC_MAX_CHANNELS 32
static FILE* bcc_files[BCC_MAX_CHANNELS];

static char bcc_file_field_buf[256];

static char* bcc_strbuf_take(void);
static const char* bcc_mid(const char* s, int start, int length);
static const char* bcc_chr(int code);
static const char* bcc_stri(int value);
static const char* bcc_strd(double value);
static void bcc_read_string_field(char* field, const unsigned char* source, size_t width);
static void bcc_mki(char* out, int value);
static void bcc_mkl(char* out, int value);
static int bcc_cvi(const char* s);
static int bcc_cvl(const char* s);
static int bcc_read_record(FILE* file, void* buffer, size_t reclen, long record);
static void bcc_write_record(FILE* file, const void* buffer, size_t reclen, long record);
static void bcc_pad_string_field(unsigned char* dest, const char* value, size_t width);
static int bcc_put_record_fields_1_0(FILE* file, long record, const char* field_0, const char* field_1, const char* field_2, const char* field_3);
static int bcc_get_record_fields_1_0(FILE* file, long record, char* field_0, char* field_1, char* field_2, char* field_3);
static int bcc_put_record_fields_1_1(FILE* file, long record, const char* field_0, const char* field_1, const char* field_2, const char* field_3);
static int bcc_get_record_fields_1_1(FILE* file, long record, char* field_0, char* field_1, char* field_2, char* field_3);
static int bcc_put_record_fields_1_2(FILE* file, long record, const char* field_0, const char* field_1, const char* field_2, const char* field_3);
static int bcc_get_record_fields_1_2(FILE* file, long record, char* field_0, char* field_1, char* field_2, char* field_3);
static int bcc_put_record_fields_1_3(FILE* file, long record, const char* field_0, const char* field_1, const char* field_2, const char* field_3);
static int bcc_get_record_fields_1_3(FILE* file, long record, char* field_0, char* field_1, char* field_2, char* field_3);
static int bcc_put_record_fields_1_4(FILE* file, long record, const char* field_0, const char* field_1, const char* field_2, const char* field_3);
static int bcc_get_record_fields_1_4(FILE* file, long record, char* field_0, char* field_1, char* field_2, char* field_3);
static int bcc_put_record_fields_1_5(FILE* file, long record, const char* field_0, const char* field_1, const char* field_2, const char* field_3);
static int bcc_get_record_fields_1_5(FILE* file, long record, char* field_0, char* field_1, char* field_2, char* field_3);
static int bcc_put_record_student(FILE* file, long record, const int16_t* field_0, const char* field_1, const double* field_2, const char* field_3);
static int bcc_get_record_student(FILE* file, long record, char* field_0, char* field_1, char* field_2, char* field_3);
static void bcc_mks(char* out, double value);
static void bcc_mkd(char* out, double value);
static float bcc_cvs(const char* s);
static double bcc_cvd(const char* s);
static int bcc_eof(FILE* file);
static void bcc_line_input_file(FILE* file, char* buf, size_t bufsize);
static void bcc_read_file_field(FILE* file, char* buf, size_t bufsize);

static double bv_d_carolscore = 0;
static double bv_d_score = 0;
static double bv_d_sscore = 0;
static int bv_i_carolid = 0;
static int bv_i_i = 0;
static int bv_i_id = 0;
static int bv_i_num_recs = 0;
static int bv_i_rec_len = 0;
static int bv_i_sid = 0;
static char bv_s_carolfaculty[256] = {0};
static char bv_s_carolname[256] = {0};
static char bv_s_db_file[256] = {0};
static char bv_s_dbfacultybuf[256] = {0};
static char bv_s_dbidbuf[256] = {0};
static char bv_s_dbnamebuf[256] = {0};
static char bv_s_dbscorebuf[256] = {0};
static char bv_s_facultybuf[256] = {0};
static char bv_s_idbuf[256] = {0};
static char bv_s_namebuf[256] = {0};
static char bv_s_scorebuf[256] = {0};
static char bv_s_sfaculty[256] = {0};
static char bv_s_sname[256] = {0};

void bf_s_ltrim_s(const char* bv_s_self_in, char* bcc_out);
void bf_s_rtrim_s(const char* bv_s_self_in, char* bcc_out);
void bf_s_trimmed(const char* bv_s_s_in, char* bcc_out);

void bf_s_ltrim_s(const char* bv_s_self_in, char* bcc_out) {
    char bv_s_self[256];
    snprintf(bv_s_self, sizeof(bv_s_self), "%s", bv_s_self_in);
    int bv_i_i = 0;

    bv_i_i = 1;
    while ((-(((-(bv_i_i <= ((int)strlen(bv_s_self))))) != 0 && ((-(strcmp(bcc_mid(bv_s_self, bv_i_i, 1), " ") == 0))) != 0))) {
        bv_i_i = (bv_i_i + 1);
    }
    snprintf(bcc_out, 256, "%s", bcc_mid(bv_s_self, bv_i_i, 2147483647));
    return;
}

void bf_s_rtrim_s(const char* bv_s_self_in, char* bcc_out) {
    char bv_s_self[256];
    snprintf(bv_s_self, sizeof(bv_s_self), "%s", bv_s_self_in);
    int bv_i_i = 0;

    bv_i_i = ((int)strlen(bv_s_self));
    while ((-(((-(bv_i_i > 0))) != 0 && ((-(strcmp(bcc_mid(bv_s_self, bv_i_i, 1), " ") == 0))) != 0))) {
        bv_i_i = (bv_i_i - 1);
    }
    snprintf(bcc_out, 256, "%s", bcc_mid(bv_s_self, 1, bv_i_i));
    return;
}

void bf_s_trimmed(const char* bv_s_s_in, char* bcc_out) {
    char bv_s_s[256];
    snprintf(bv_s_s, sizeof(bv_s_s), "%s", bv_s_s_in);
    int bv_i_i = 0;

    bv_i_i = ((int)strlen(bv_s_s));
    while ((-(((-(bv_i_i > 0))) != 0 && ((-(strcmp(bcc_mid(bv_s_s, bv_i_i, 1), " ") == 0))) != 0))) {
        bv_i_i = (bv_i_i - 1);
    }
    snprintf(bcc_out, 256, "%s", bcc_mid(bv_s_s, 1, bv_i_i));
    return;
}

int main(void) {
    // Strips leading spaces from self$. Not a real MBASIC/BASCOM 2.00 builtin --
    // verified against a real IBM BASIC Compiler 2.00 under dosbox-x -- so
    // BASCAL ships its own. Declared as a scalar method (see GitHub issue #41)
    // so a required stdlib call reads the same way as a built-in method call
    // (docs/language/functions-and-procedures.html#built-in-methods). The
    // ordinary call form (ltrim$(s$)) still works -- a method's receiver is an
    // implicit first parameter, so ordinary-call syntax resolves straight to
    // this same declaration, with no separate function needed (and no longer
    // allowed: a function and a method sharing one name is a duplicate
    // declaration, since they'd both claim the same callable identity).

    // Strips trailing spaces from self$. Not a real MBASIC/BASCOM 2.00 builtin --
    // verified against a real IBM BASIC Compiler 2.00 under dosbox-x -- so
    // BASCAL ships its own. Declared as a scalar method (see GitHub issue #41
    // and ltrim.bcl's own doc comment for the reasoning) -- rtrim$(s$) still
    // works via ordinary-call syntax resolving to this same declaration.

    // Tutorial — Random-Access Files: hand-written, then with the record/file DSL
    //
    // This tutorial writes the *same* program twice. Part 1 uses BASIC's raw
    // random-access file primitives directly. Part 2 uses BASCAL's `record`/
    // `file` DSL, which transpiles to exactly the same primitives — nothing about
    // the *generated* BASIC changes, only how much of it you have to write by
    // hand. Read Part 1 first; the comments between the two parts explain what
    // the DSL is buying you and why.
    //
    // ---- Part 1 primitives ----
    //
    // open filename$ for random as #n len = REC_LEN
    // Open (or create) a random-access file.  len specifies the record length
    // in bytes; every record occupies exactly that many bytes.
    //
    // field #n, width1% as var1$, width2% as var2$, ...
    // Bind string variables to regions of the file buffer.  The sum of all
    // widths must equal the record length.  Only string variables may be used
    // in a FIELD statement.
    //
    // lset var$ = expr$   — copy into a field buffer, left-justified (padded)
    // rset var$ = expr$   — copy into a field buffer, right-justified (padded)
    //
    // put #n, recordNumber%   — write the current buffer as record n (1-based)
    // get #n, recordNumber%   — read record n into the buffer variables
    //
    // Packing helpers (BASIC builtins):
    // mki$(n%)  — pack a 2-byte integer into a 2-character string
    // mkl$(n&)  — pack a 4-byte long
    // mks$(n!)  — pack a 4-byte single
    // mkd$(n#)  — pack an 8-byte double
    // cvi(s$)   — unpack a 2-byte integer from a string
    // cvl(s$)   — unpack a 4-byte long
    // cvs(s$)   — unpack a 4-byte single
    // cvd(s$)   — unpack an 8-byte double
    //
    // Every MKx$ always returns a string (never a type-suffixed MKI%/MKD#/etc —
    // those aren't real MBASIC/BASCOM functions), and every CVx takes no suffix
    // at all. There's also no RTRIM$ builtin on real MBASIC/BASCOM -- trimming a
    // fixed-width, space-padded FIELD buffer back down to its real length needs
    // a hand-rolled loop, like trimmed$ below.

    // trimmed$ -- right-trim trailing spaces from a fixed-width FIELD buffer.

    bv_i_rec_len = 50;
    bv_i_num_recs = 3;
    snprintf(bv_s_db_file, sizeof(bv_s_db_file), "%s", "tutorial_students.dat");

    // ============================================================
    // Part 1 — random-access files, written by hand
    // ============================================================

    // ---- Write three records ----

    bcc_raise_retry_0: ;
    bcc_files[0] = fopen(bv_s_db_file, "rb+");
    if (!bcc_files[0]) bcc_files[0] = fopen(bv_s_db_file, "wb+");
    if (!bcc_files[0]) {
        bcc_err = 75;
        bcc_resume_id = 0;
        bcc_erl = 63;
        bcc_err_file = "tutorial/random_and_record_files.bcl";
        if (bcc_on_error_target < 0 || bcc_in_handler) {
            fprintf(stderr, "unhandled BASIC error %d\n", bcc_err);
            exit(1);
        }
        bcc_in_handler = 1;
        switch (bcc_on_error_target) {
        }
    }
    bcc_raise_after_0: ;

    // Record 1: Alice, 95
    bcc_mki(bv_s_idbuf, 1);
    snprintf(bv_s_namebuf, sizeof(bv_s_namebuf), "%-*.*s", 20, 20, "Alice");
    bcc_mkd(bv_s_scorebuf, 95.0);
    snprintf(bv_s_facultybuf, sizeof(bv_s_facultybuf), "%-*.*s", 20, 20, "Engineering");
    bcc_put_record_fields_1_0(bcc_files[0], 1, bv_s_idbuf, bv_s_namebuf, bv_s_scorebuf, bv_s_facultybuf);

    // Record 2: Bob, 54
    bcc_mki(bv_s_idbuf, 2);
    snprintf(bv_s_namebuf, sizeof(bv_s_namebuf), "%-*.*s", 20, 20, "Bob");
    bcc_mkd(bv_s_scorebuf, 54.0);
    snprintf(bv_s_facultybuf, sizeof(bv_s_facultybuf), "%-*.*s", 20, 20, "Arts");
    bcc_put_record_fields_1_0(bcc_files[0], 2, bv_s_idbuf, bv_s_namebuf, bv_s_scorebuf, bv_s_facultybuf);

    // Record 3: Carol, 78
    bcc_mki(bv_s_idbuf, 3);
    snprintf(bv_s_namebuf, sizeof(bv_s_namebuf), "%-*.*s", 20, 20, "Carol");
    bcc_mkd(bv_s_scorebuf, 78.0);
    snprintf(bv_s_facultybuf, sizeof(bv_s_facultybuf), "%-*.*s", 20, 20, "Science");
    bcc_put_record_fields_1_0(bcc_files[0], 3, bv_s_idbuf, bv_s_namebuf, bv_s_scorebuf, bv_s_facultybuf);

    fclose(bcc_files[0]);
    bcc_files[0] = NULL;

    // ---- Read records in reverse order ----

    printf("Part 1 (hand-written) -- reading records in reverse order:\n");
    bcc_raise_retry_1: ;
    bcc_files[0] = fopen(bv_s_db_file, "rb+");
    if (!bcc_files[0]) bcc_files[0] = fopen(bv_s_db_file, "wb+");
    if (!bcc_files[0]) {
        bcc_err = 75;
        bcc_resume_id = 1;
        bcc_erl = 92;
        bcc_err_file = "tutorial/random_and_record_files.bcl";
        if (bcc_on_error_target < 0 || bcc_in_handler) {
            fprintf(stderr, "unhandled BASIC error %d\n", bcc_err);
            exit(1);
        }
        bcc_in_handler = 1;
        switch (bcc_on_error_target) {
        }
    }
    bcc_raise_after_1: ;

    int bt_lim_0 = 1;
    int bt_step_0 = -(1);
    for (bv_i_i = bv_i_num_recs; bt_step_0 >= 0 ? bv_i_i <= bt_lim_0 : bv_i_i >= bt_lim_0; bv_i_i += bt_step_0) {
        bcc_get_record_fields_1_1(bcc_files[0], bv_i_i, bv_s_idbuf, bv_s_namebuf, bv_s_scorebuf, bv_s_facultybuf);
        bv_i_id = bcc_cvi(bv_s_idbuf);
        bv_d_score = bcc_cvd(bv_s_scorebuf);
        char bt_s_1[256];
        snprintf(bt_s_1, sizeof(bt_s_1), "%s%s", "  [", bcc_stri(bv_i_id));
        char bt_s_2[256];
        snprintf(bt_s_2, sizeof(bt_s_2), "%s%s", bt_s_1, "] ");
        char bt_s_3[256];
        bf_s_trimmed(bv_s_namebuf, bt_s_3);
        char bt_s_4[256];
        snprintf(bt_s_4, sizeof(bt_s_4), "%s%s", bt_s_2, bt_s_3);
        char bt_s_5[256];
        snprintf(bt_s_5, sizeof(bt_s_5), "%s%s", bt_s_4, " -- ");
        char bt_s_6[256];
        snprintf(bt_s_6, sizeof(bt_s_6), "%s%s", bt_s_5, bcc_strd(bv_d_score));
        printf("%s\n", bt_s_6);
    }

    fclose(bcc_files[0]);
    bcc_files[0] = NULL;

    // ---- Update one field in place ----

    bcc_raise_retry_2: ;
    bcc_files[0] = fopen(bv_s_db_file, "rb+");
    if (!bcc_files[0]) bcc_files[0] = fopen(bv_s_db_file, "wb+");
    if (!bcc_files[0]) {
        bcc_err = 75;
        bcc_resume_id = 2;
        bcc_erl = 106;
        bcc_err_file = "tutorial/random_and_record_files.bcl";
        if (bcc_on_error_target < 0 || bcc_in_handler) {
            fprintf(stderr, "unhandled BASIC error %d\n", bcc_err);
            exit(1);
        }
        bcc_in_handler = 1;
        switch (bcc_on_error_target) {
        }
    }
    bcc_raise_after_2: ;

    // Bob just scraped a pass on re-mark. Only scoreBuf$ changes, but PUT
    // always writes the whole 50-byte buffer, so GET has to load the record
    // first even though idBuf$/nameBuf$/facultyBuf$ are just being written straight back
    // unchanged.
    bcc_get_record_fields_1_2(bcc_files[0], 2, bv_s_idbuf, bv_s_namebuf, bv_s_scorebuf, bv_s_facultybuf);
    bcc_mkd(bv_s_scorebuf, 61.5);
    bcc_put_record_fields_1_2(bcc_files[0], 2, bv_s_idbuf, bv_s_namebuf, bv_s_scorebuf, bv_s_facultybuf);

    fclose(bcc_files[0]);
    bcc_files[0] = NULL;

    // ---- Update two fields at once ----

    bcc_raise_retry_3: ;
    bcc_files[0] = fopen(bv_s_db_file, "rb+");
    if (!bcc_files[0]) bcc_files[0] = fopen(bv_s_db_file, "wb+");
    if (!bcc_files[0]) {
        bcc_err = 75;
        bcc_resume_id = 3;
        bcc_erl = 121;
        bcc_err_file = "tutorial/random_and_record_files.bcl";
        if (bcc_on_error_target < 0 || bcc_in_handler) {
            fprintf(stderr, "unhandled BASIC error %d\n", bcc_err);
            exit(1);
        }
        bcc_in_handler = 1;
        switch (bcc_on_error_target) {
        }
    }
    bcc_raise_after_3: ;

    // Alice got married and re-sat the exam — `name` and `score` both change,
    // `id` and `faculty` don't. Same problem as Bob's update, just with two fields instead
    // of one: GET first (this is what preserves idBuf$ and facultyBuf$), LSET the two fields
    // that actually changed, then PUT the whole buffer back. Nothing here is
    // specific to "two" fields — five changed fields would look identical,
    // just with five LSET lines between the GET and the PUT.
    bcc_get_record_fields_1_3(bcc_files[0], 1, bv_s_idbuf, bv_s_namebuf, bv_s_scorebuf, bv_s_facultybuf);
    snprintf(bv_s_namebuf, sizeof(bv_s_namebuf), "%-*.*s", 20, 20, "Alice Smith");
    bcc_mkd(bv_s_scorebuf, 91.0);
    bcc_put_record_fields_1_3(bcc_files[0], 1, bv_s_idbuf, bv_s_namebuf, bv_s_scorebuf, bv_s_facultybuf);

    fclose(bcc_files[0]);
    bcc_files[0] = NULL;

    // ---- Same shape again ----

    bcc_raise_retry_4: ;
    bcc_files[0] = fopen(bv_s_db_file, "rb+");
    if (!bcc_files[0]) bcc_files[0] = fopen(bv_s_db_file, "wb+");
    if (!bcc_files[0]) {
        bcc_err = 75;
        bcc_resume_id = 4;
        bcc_erl = 139;
        bcc_err_file = "tutorial/random_and_record_files.bcl";
        if (bcc_on_error_target < 0 || bcc_in_handler) {
            fprintf(stderr, "unhandled BASIC error %d\n", bcc_err);
            exit(1);
        }
        bcc_in_handler = 1;
        switch (bcc_on_error_target) {
        }
    }
    bcc_raise_after_4: ;

    // Carol changed her name and improved her score: the exact same
    // GET / LSET / LSET / PUT shape as Alice's update above, just retyped by
    // hand with Carol's record number and values.
    bcc_get_record_fields_1_4(bcc_files[0], 3, bv_s_idbuf, bv_s_namebuf, bv_s_scorebuf, bv_s_facultybuf);
    snprintf(bv_s_namebuf, sizeof(bv_s_namebuf), "%-*.*s", 20, 20, "Carol Jones");
    bcc_mkd(bv_s_scorebuf, 88.0);
    bcc_put_record_fields_1_4(bcc_files[0], 3, bv_s_idbuf, bv_s_namebuf, bv_s_scorebuf, bv_s_facultybuf);

    fclose(bcc_files[0]);
    bcc_files[0] = NULL;

    // ---- Verify the updates ----

    printf("Part 1 (hand-written) -- after updates:\n");
    bcc_raise_retry_5: ;
    bcc_files[0] = fopen(bv_s_db_file, "rb+");
    if (!bcc_files[0]) bcc_files[0] = fopen(bv_s_db_file, "wb+");
    if (!bcc_files[0]) {
        bcc_err = 75;
        bcc_resume_id = 5;
        bcc_erl = 155;
        bcc_err_file = "tutorial/random_and_record_files.bcl";
        if (bcc_on_error_target < 0 || bcc_in_handler) {
            fprintf(stderr, "unhandled BASIC error %d\n", bcc_err);
            exit(1);
        }
        bcc_in_handler = 1;
        switch (bcc_on_error_target) {
        }
    }
    bcc_raise_after_5: ;

    int bt_lim_7 = bv_i_num_recs;
    int bt_step_7 = 1;
    for (bv_i_i = 1; bt_step_7 >= 0 ? bv_i_i <= bt_lim_7 : bv_i_i >= bt_lim_7; bv_i_i += bt_step_7) {
        bcc_get_record_fields_1_5(bcc_files[0], bv_i_i, bv_s_idbuf, bv_s_namebuf, bv_s_scorebuf, bv_s_facultybuf);
        char bt_s_8[256];
        bf_s_trimmed(bv_s_namebuf, bt_s_8);
        char bt_s_9[256];
        snprintf(bt_s_9, sizeof(bt_s_9), "%s%s", "  ", bt_s_8);
        char bt_s_10[256];
        snprintf(bt_s_10, sizeof(bt_s_10), "%s%s", bt_s_9, ": ");
        char bt_s_11[256];
        snprintf(bt_s_11, sizeof(bt_s_11), "%s%s", bt_s_10, bcc_strd(bcc_cvd(bv_s_scorebuf)));
        printf("%s\n", bt_s_11);
    }

    fclose(bcc_files[0]);
    bcc_files[0] = NULL;

    // ------------------------------------------------------------------------
    // What Part 1 actually cost:
    //
    // - idBuf$/nameBuf$/scoreBuf$ and the FIELD statement binding them had to
    // be repeated, identically, in every OPEN block — get it wrong in one
    // of the five and you're reading or writing the wrong bytes.
    // - REC_LEN (50) is 2+20+8+20 computed by hand; add a field to the record
    // and every one of those numbers has to be updated together, or the
    // file silently gets corrupted.
    // - Each field's pack/unpack call (mki$/cvi, mkd$/cvd, or nothing for
    // strings) has to be matched to that field's type by hand, every time
    // it's touched — nothing stops mkd$() being used on the id field.
    // - There's no RTRIM$ builtin on real MBASIC/BASCOM, so reading a string
    // field back means hand-rolling a trim loop (trimmed$, above) and
    // remembering to call it, every time.
    // - Alice's and Carol's updates are the identical GET/LSET/LSET/PUT
    // pattern, typed out twice, with every buffer/field name repeated.
    //
    // None of this is hard, exactly — it's just bookkeeping a compiler should
    // be doing for you. Part 2 is the same program again, with BASCAL's
    // record/file DSL doing that bookkeeping.
    // ------------------------------------------------------------------------

    // ============================================================
    // Part 2 — the same program with the record / file DSL
    // ============================================================
    //
    // record <Name> ... end record
    // Declares a fixed-layout record type. Supported field types: int16,
    // int32, float32, float64, and string(N). The record's total byte width
    // (used as Part 1's REC_LEN) is the sum of its field widths, computed
    // automatically.
    //
    // file <var> as <RecordType> = open(<path>)
    // Opens (or creates) a random-access file sized for one record, and binds
    // FIELD buffer variables for every field. File numbers are allocated
    // automatically, starting at #1, in the order `file` declarations appear.
    // This one line replaces Part 1's REC_LEN constant, OPEN, and FIELD.
    //
    // <file>[<n>] = { field: value, ... }
    // Whole-record write: packs every field (LSET, MKx$ for numeric fields)
    // and writes record n. Every declared field must be given — a missing one
    // is a compile-time error.
    //
    // let <var> = <file>[<n>]
    // Whole-record read: reads record n and unpacks every field (CVx for
    // numeric fields, an inline trim loop like Part 1's trimmed$ for strings)
    // into `<var>.<field>`.
    //
    // <file>[<n>].<field> = value
    // Partial update: GET, LSET just that one field, PUT. The one-field
    // version of Part 1's Bob update, with no buffer names to get wrong.
    //
    // <file>[<n>] = ?{ field: value, ... }
    // Partial-record write: any subset of fields; unlisted ones are left
    // untouched on disk. Whether a GET is needed is decided at *compile
    // time* by comparing the given field names against the record's declared
    // fields: some fields missing -> GET first, LSET just those fields, then
    // PUT (this is Alice's update from Part 1, minus the GET/LSET/LSET/PUT
    // written out by hand); every field given anyway -> no GET, same as a
    // plain `{...}`. Unlike `{...}`, an *unknown* field name is still a
    // compile-time error — only *missing* fields are allowed, not misspelled
    // ones.
    //
    // let <var> = <file>[<n>]
    // <var>.<field> = value  (any number of times)
    // <file>[<n>] = <var>
    // Batched update: the `let` does one GET; each `<var>.<field> = value` is
    // a pure in-memory assignment (no I/O); the final `<file>[<n>] = <var>`
    // packs every field from `<var>` and does one PUT. This is Carol's update
    // from Part 1 — same GET/LSET/LSET/PUT shape as `?{...}`, just written as
    // read-mutate-write instead of a single literal, useful when the new
    // values come from more than a one-line expression.
    //
    // for <var> = <A> downto <B> ... end for
    // Sugar for `for <var> = <A> to <B> step -1`.
    //
    // <file>.close()
    // Closes the file.

    // file db as Student = open(...)  [50 bytes/record]
    bcc_raise_retry_6: ;
    bcc_files[0] = fopen("tutorial_records.dat", "rb+");
    if (!bcc_files[0]) bcc_files[0] = fopen("tutorial_records.dat", "wb+");
    if (!bcc_files[0]) {
        bcc_err = 75;
        bcc_resume_id = 6;
        bcc_erl = 252;
        bcc_err_file = "tutorial/random_and_record_files.bcl";
        if (bcc_on_error_target < 0 || bcc_in_handler) {
            fprintf(stderr, "unhandled BASIC error %d\n", bcc_err);
            exit(1);
        }
        bcc_in_handler = 1;
        switch (bcc_on_error_target) {
        }
    }
    bcc_raise_after_6: ;

    // ---- Write three records ----

    // Record 1: Alice, 95
    // db[...] = { ... }  (whole-record write)
    int16_t bcc_tmp_12 = 1;
    double bcc_tmp_13 = 95.0;
    bcc_put_record_student(bcc_files[0], 1, &bcc_tmp_12, "Alice", &bcc_tmp_13, "Engineering");

    // Record 2: Bob, 54
    // db[...] = { ... }  (whole-record write)
    int16_t bcc_tmp_14 = 2;
    double bcc_tmp_15 = 54.0;
    bcc_put_record_student(bcc_files[0], 2, &bcc_tmp_14, "Bob", &bcc_tmp_15, "Arts");

    // Record 3: Carol, 78
    // db[...] = { ... }  (whole-record write)
    int16_t bcc_tmp_16 = 3;
    double bcc_tmp_17 = 78.0;
    bcc_put_record_student(bcc_files[0], 3, &bcc_tmp_16, "Carol", &bcc_tmp_17, "Science");

    // ---- Read records in reverse order ----

    printf("Part 2 (record/file DSL) -- reading records in reverse order:\n");

    int bt_lim_18 = 1;
    int bt_step_18 = -1;
    for (bv_i_i = 3; bt_step_18 >= 0 ? bv_i_i <= bt_lim_18 : bv_i_i >= bt_lim_18; bv_i_i += bt_step_18) {
        // let s = db[...]  (whole-record read)
        bcc_get_record_student(bcc_files[0], bv_i_i, bv_s_dbidbuf, bv_s_dbnamebuf, bv_s_dbscorebuf, bv_s_dbfacultybuf);
        bv_i_sid = bcc_cvi(bv_s_dbidbuf);
        char bt_s_19[256];
        bf_s_rtrim_s(bv_s_dbnamebuf, bt_s_19);
        snprintf(bv_s_sname, sizeof(bv_s_sname), "%s", bt_s_19);
        bv_d_sscore = bcc_cvd(bv_s_dbscorebuf);
        char bt_s_20[256];
        bf_s_rtrim_s(bv_s_dbfacultybuf, bt_s_20);
        snprintf(bv_s_sfaculty, sizeof(bv_s_sfaculty), "%s", bt_s_20);
        char bt_s_21[256];
        snprintf(bt_s_21, sizeof(bt_s_21), "%s%s", "  [", bcc_stri(bv_i_sid));
        char bt_s_22[256];
        snprintf(bt_s_22, sizeof(bt_s_22), "%s%s", bt_s_21, "] ");
        char bt_s_23[256];
        snprintf(bt_s_23, sizeof(bt_s_23), "%s%s", bt_s_22, bv_s_sname);
        char bt_s_24[256];
        snprintf(bt_s_24, sizeof(bt_s_24), "%s%s", bt_s_23, " -- ");
        char bt_s_25[256];
        snprintf(bt_s_25, sizeof(bt_s_25), "%s%s", bt_s_24, bcc_strd(bv_d_sscore));
        printf("%s\n", bt_s_25);
    }

    // ---- Update one field in place ----

    // Bob just scraped a pass on re-mark. Compare to Part 1: no REC_LEN, no
    // idBuf$/nameBuf$/scoreBuf$/facultyBuf$, no mkd$() — just the field that's changing.
    // db[...].score = ...  (partial-field update)
    double bcc_tmp_26 = 61.5;
    if (!bcc_put_record_student(bcc_files[0], 2, NULL, NULL, &bcc_tmp_26, NULL)) { fprintf(stderr, "BASCAL: record %ld does not exist\n", (long)2); exit(1); }

    // ---- Update two fields at once, still one GET and one PUT ----

    // Alice got married and re-sat the exam. `name` and `score` don't cover
    // every field of Student, so this needs an implicit GET first (id and
    // faculty are preserved from the existing record) -- exactly Part 1's GET / LSET /
    // LSET / PUT for Alice, minus having to write out the GET, the buffer
    // names, or the packing calls. Which fields need a GET is worked out by
    // the compiler by comparing `name`/`score` against Student's declared
    // fields — not decided at runtime.
    // db[...] = ?{ ... }  (partial-record write)
    double bcc_tmp_27 = 91.0;
    if (!bcc_put_record_student(bcc_files[0], 1, NULL, "Alice Smith", &bcc_tmp_27, NULL)) { fprintf(stderr, "BASCAL: record %ld does not exist\n", (long)1); exit(1); }

    // ---- Batched update: read once, mutate twice, write back once ----

    // Carol changed her name and improved her score — the read-mutate-write
    // form of the same one-GET-one-PUT update, useful when the new values
    // aren't just a couple of literals.
    // let carol = db[...]  (whole-record read)
    bcc_get_record_student(bcc_files[0], 3, bv_s_dbidbuf, bv_s_dbnamebuf, bv_s_dbscorebuf, bv_s_dbfacultybuf);
    bv_i_carolid = bcc_cvi(bv_s_dbidbuf);
    char bt_s_28[256];
    bf_s_rtrim_s(bv_s_dbnamebuf, bt_s_28);
    snprintf(bv_s_carolname, sizeof(bv_s_carolname), "%s", bt_s_28);
    bv_d_carolscore = bcc_cvd(bv_s_dbscorebuf);
    char bt_s_29[256];
    bf_s_rtrim_s(bv_s_dbfacultybuf, bt_s_29);
    snprintf(bv_s_carolfaculty, sizeof(bv_s_carolfaculty), "%s", bt_s_29);
    snprintf(bv_s_carolname, sizeof(bv_s_carolname), "%s", "Carol Jones");
    bv_d_carolscore = 88.0;
    // db[...] = carol  (write back a let-bound record)
    int16_t bcc_tmp_30 = bv_i_carolid;
    double bcc_tmp_31 = bv_d_carolscore;
    bcc_put_record_student(bcc_files[0], 3, &bcc_tmp_30, bv_s_carolname, &bcc_tmp_31, bv_s_carolfaculty);

    // ---- Verify the updates ----

    printf("Part 2 (record/file DSL) -- after updates:\n");

    int bt_lim_32 = 3;
    int bt_step_32 = 1;
    for (bv_i_i = 1; bt_step_32 >= 0 ? bv_i_i <= bt_lim_32 : bv_i_i >= bt_lim_32; bv_i_i += bt_step_32) {
        // let s = db[...]  (whole-record read)
        bcc_get_record_student(bcc_files[0], bv_i_i, bv_s_dbidbuf, bv_s_dbnamebuf, bv_s_dbscorebuf, bv_s_dbfacultybuf);
        bv_i_sid = bcc_cvi(bv_s_dbidbuf);
        char bt_s_33[256];
        bf_s_rtrim_s(bv_s_dbnamebuf, bt_s_33);
        snprintf(bv_s_sname, sizeof(bv_s_sname), "%s", bt_s_33);
        bv_d_sscore = bcc_cvd(bv_s_dbscorebuf);
        char bt_s_34[256];
        bf_s_rtrim_s(bv_s_dbfacultybuf, bt_s_34);
        snprintf(bv_s_sfaculty, sizeof(bv_s_sfaculty), "%s", bt_s_34);
        char bt_s_35[256];
        snprintf(bt_s_35, sizeof(bt_s_35), "%s%s", "  ", bv_s_sname);
        char bt_s_36[256];
        snprintf(bt_s_36, sizeof(bt_s_36), "%s%s", bt_s_35, ": ");
        char bt_s_37[256];
        snprintf(bt_s_37, sizeof(bt_s_37), "%s%s", bt_s_36, bcc_strd(bv_d_sscore));
        printf("%s\n", bt_s_37);
    }

    // db.close()
    fclose(bcc_files[0]);
    bcc_files[0] = NULL;

    // ------------------------------------------------------------------------
    // Part 2 is the same three writes, the same reverse-order read, and the
    // same three updates as Part 1 — Alice's and Bob's and Carol's updates
    // still transpile to exactly one GET and one PUT each, nothing runs slower.
    // What's gone is everything that was bookkeeping rather than logic: the
    // hand-computed record width, the repeated buffer-variable/FIELD
    // boilerplate in every block, the pack/unpack call picked by hand per
    // field, and the GET-or-not decision for a partial write, which the
    // compiler now makes for you at compile time by simply comparing field
    // names -- get a field name wrong (`db[1] = ?{ nmae: ... }`) and it's a
    // compile error instead of a silently corrupted record.
    // ------------------------------------------------------------------------

    return 0;
}

static char* bcc_strbuf_take(void) {
    char* buf = bcc_strbuf[bcc_strbuf_next];
    bcc_strbuf_next = (bcc_strbuf_next + 1) % BCC_STRBUF_COUNT;
    return buf;
}

static const char* bcc_mid(const char* s, int start, int length) {
    char* out = bcc_strbuf_take();
    int len = (int)strlen(s);
    int from = start - 1;
    if (from < 0) from = 0;
    if (from > len) from = len;
    int avail = len - from;
    if (length < 0) length = 0;
    if (length > avail) length = avail;
    snprintf(out, 256, "%.*s", length, s + from);
    return out;
}

static const char* bcc_chr(int code) {
    char* out = bcc_strbuf_take();
    snprintf(out, 256, "%c", code);
    return out;
}

static const char* bcc_stri(int value) {
    char* out = bcc_strbuf_take();
    snprintf(out, 256, "% d", value);
    return out;
}

static const char* bcc_strd(double value) {
    char* out = bcc_strbuf_take();
    snprintf(out, 256, "% g", value);
    return out;
}

static void bcc_read_string_field(char* field, const unsigned char* source, size_t width) {
    memcpy(field, source, width);
    field[width] = 0;
    while (width > 0 && field[width - 1] == ' ') field[--width] = 0;
}

static void bcc_mki(char* out, int value) {
    int16_t v = (int16_t)value;
    memcpy(out, &v, 2);
}

static void bcc_mkl(char* out, int value) {
    int32_t v = (int32_t)value;
    memcpy(out, &v, 4);
}

static int bcc_cvi(const char* s) {
    int16_t v;
    memcpy(&v, s, 2);
    return (int)v;
}

static int bcc_cvl(const char* s) {
    int32_t v;
    memcpy(&v, s, 4);
    return (int)v;
}

static int bcc_read_record(FILE* file, void* buffer, size_t reclen, long record) {
    if (fseek(file, (record - 1) * (long)reclen, SEEK_SET) != 0) return 0;
    return fread(buffer, 1, reclen, file) == reclen;
}

static void bcc_write_record(FILE* file, const void* buffer, size_t reclen, long record) {
    fseek(file, (record - 1) * (long)reclen, SEEK_SET);
    fwrite(buffer, 1, reclen, file);
}

static void bcc_pad_string_field(unsigned char* dest, const char* value, size_t width) {
    size_t len = strlen(value);
    if (len > width) len = width;
    memcpy(dest, value, len);
    memset(dest + len, ' ', width - len);
}

static int bcc_put_record_fields_1_0(FILE* file, long record, const char* field_0, const char* field_1, const char* field_2, const char* field_3) {
    unsigned char buffer[50];
    memcpy(buffer + 0, field_0, 2);
    memcpy(buffer + 2, field_1, 20);
    memcpy(buffer + 22, field_2, 8);
    memcpy(buffer + 30, field_3, 20);
    bcc_write_record(file, buffer, 50, record);
    return 1;
}

static int bcc_get_record_fields_1_0(FILE* file, long record, char* field_0, char* field_1, char* field_2, char* field_3) {
    unsigned char buffer[50];
    if (!bcc_read_record(file, buffer, 50, record)) return 0;
    memcpy(field_0, buffer + 0, 2);
    field_0[2] = 0;
    bcc_read_string_field(field_1, buffer + 2, 20);
    memcpy(field_2, buffer + 22, 8);
    field_2[8] = 0;
    bcc_read_string_field(field_3, buffer + 30, 20);
    return 1;
}

static int bcc_put_record_fields_1_1(FILE* file, long record, const char* field_0, const char* field_1, const char* field_2, const char* field_3) {
    unsigned char buffer[50];
    memcpy(buffer + 0, field_0, 2);
    memcpy(buffer + 2, field_1, 20);
    memcpy(buffer + 22, field_2, 8);
    memcpy(buffer + 30, field_3, 20);
    bcc_write_record(file, buffer, 50, record);
    return 1;
}

static int bcc_get_record_fields_1_1(FILE* file, long record, char* field_0, char* field_1, char* field_2, char* field_3) {
    unsigned char buffer[50];
    if (!bcc_read_record(file, buffer, 50, record)) return 0;
    memcpy(field_0, buffer + 0, 2);
    field_0[2] = 0;
    bcc_read_string_field(field_1, buffer + 2, 20);
    memcpy(field_2, buffer + 22, 8);
    field_2[8] = 0;
    bcc_read_string_field(field_3, buffer + 30, 20);
    return 1;
}

static int bcc_put_record_fields_1_2(FILE* file, long record, const char* field_0, const char* field_1, const char* field_2, const char* field_3) {
    unsigned char buffer[50];
    memcpy(buffer + 0, field_0, 2);
    memcpy(buffer + 2, field_1, 20);
    memcpy(buffer + 22, field_2, 8);
    memcpy(buffer + 30, field_3, 20);
    bcc_write_record(file, buffer, 50, record);
    return 1;
}

static int bcc_get_record_fields_1_2(FILE* file, long record, char* field_0, char* field_1, char* field_2, char* field_3) {
    unsigned char buffer[50];
    if (!bcc_read_record(file, buffer, 50, record)) return 0;
    memcpy(field_0, buffer + 0, 2);
    field_0[2] = 0;
    bcc_read_string_field(field_1, buffer + 2, 20);
    memcpy(field_2, buffer + 22, 8);
    field_2[8] = 0;
    bcc_read_string_field(field_3, buffer + 30, 20);
    return 1;
}

static int bcc_put_record_fields_1_3(FILE* file, long record, const char* field_0, const char* field_1, const char* field_2, const char* field_3) {
    unsigned char buffer[50];
    memcpy(buffer + 0, field_0, 2);
    memcpy(buffer + 2, field_1, 20);
    memcpy(buffer + 22, field_2, 8);
    memcpy(buffer + 30, field_3, 20);
    bcc_write_record(file, buffer, 50, record);
    return 1;
}

static int bcc_get_record_fields_1_3(FILE* file, long record, char* field_0, char* field_1, char* field_2, char* field_3) {
    unsigned char buffer[50];
    if (!bcc_read_record(file, buffer, 50, record)) return 0;
    memcpy(field_0, buffer + 0, 2);
    field_0[2] = 0;
    bcc_read_string_field(field_1, buffer + 2, 20);
    memcpy(field_2, buffer + 22, 8);
    field_2[8] = 0;
    bcc_read_string_field(field_3, buffer + 30, 20);
    return 1;
}

static int bcc_put_record_fields_1_4(FILE* file, long record, const char* field_0, const char* field_1, const char* field_2, const char* field_3) {
    unsigned char buffer[50];
    memcpy(buffer + 0, field_0, 2);
    memcpy(buffer + 2, field_1, 20);
    memcpy(buffer + 22, field_2, 8);
    memcpy(buffer + 30, field_3, 20);
    bcc_write_record(file, buffer, 50, record);
    return 1;
}

static int bcc_get_record_fields_1_4(FILE* file, long record, char* field_0, char* field_1, char* field_2, char* field_3) {
    unsigned char buffer[50];
    if (!bcc_read_record(file, buffer, 50, record)) return 0;
    memcpy(field_0, buffer + 0, 2);
    field_0[2] = 0;
    bcc_read_string_field(field_1, buffer + 2, 20);
    memcpy(field_2, buffer + 22, 8);
    field_2[8] = 0;
    bcc_read_string_field(field_3, buffer + 30, 20);
    return 1;
}

static int bcc_put_record_fields_1_5(FILE* file, long record, const char* field_0, const char* field_1, const char* field_2, const char* field_3) {
    unsigned char buffer[50];
    memcpy(buffer + 0, field_0, 2);
    memcpy(buffer + 2, field_1, 20);
    memcpy(buffer + 22, field_2, 8);
    memcpy(buffer + 30, field_3, 20);
    bcc_write_record(file, buffer, 50, record);
    return 1;
}

static int bcc_get_record_fields_1_5(FILE* file, long record, char* field_0, char* field_1, char* field_2, char* field_3) {
    unsigned char buffer[50];
    if (!bcc_read_record(file, buffer, 50, record)) return 0;
    memcpy(field_0, buffer + 0, 2);
    field_0[2] = 0;
    bcc_read_string_field(field_1, buffer + 2, 20);
    memcpy(field_2, buffer + 22, 8);
    field_2[8] = 0;
    bcc_read_string_field(field_3, buffer + 30, 20);
    return 1;
}

static int bcc_put_record_student(FILE* file, long record, const int16_t* field_0, const char* field_1, const double* field_2, const char* field_3) {
    unsigned char buffer[50];
    if ((!field_0 || !field_1 || !field_2 || !field_3) && !bcc_read_record(file, buffer, 50, record)) return 0;
    (void)(field_0 && memcpy(buffer + 0, field_0, 2));
    if (field_1) bcc_pad_string_field(buffer + 2, field_1, 20);
    (void)(field_2 && memcpy(buffer + 22, field_2, 8));
    if (field_3) bcc_pad_string_field(buffer + 30, field_3, 20);
    bcc_write_record(file, buffer, 50, record);
    return 1;
}

static int bcc_get_record_student(FILE* file, long record, char* field_0, char* field_1, char* field_2, char* field_3) {
    unsigned char buffer[50];
    if (!bcc_read_record(file, buffer, 50, record)) return 0;
    memcpy(field_0, buffer + 0, 2);
    field_0[2] = 0;
    bcc_read_string_field(field_1, buffer + 2, 20);
    memcpy(field_2, buffer + 22, 8);
    field_2[8] = 0;
    bcc_read_string_field(field_3, buffer + 30, 20);
    return 1;
}

static void bcc_mks(char* out, double value) {
    float v = (float)value;
    memcpy(out, &v, 4);
}

static void bcc_mkd(char* out, double value) {
    memcpy(out, &value, 8);
}

static float bcc_cvs(const char* s) {
    float v;
    memcpy(&v, s, 4);
    return v;
}

static double bcc_cvd(const char* s) {
    double v;
    memcpy(&v, s, 8);
    return v;
}

static int bcc_eof(FILE* file) {
    int c = fgetc(file);
    if (c == EOF) return -1;
    ungetc(c, file);
    return 0;
}

static void bcc_line_input_file(FILE* file, char* buf, size_t bufsize) {
    if (fgets(buf, (int)bufsize, file) == NULL) {
        buf[0] = 0;
        return;
    }
    buf[strcspn(buf, "\r\n")] = 0;
}

static void bcc_read_file_field(FILE* file, char* buf, size_t bufsize) {
    int c = fgetc(file);
    while (c == ' ') c = fgetc(file);
    size_t len = 0;
    if (c == '"') {
        c = fgetc(file);
        while (c != EOF && c != '"') {
            if (len + 1 < bufsize) buf[len++] = (char)c;
            c = fgetc(file);
        }
        c = fgetc(file);
        while (c != EOF && c != ',' && c != '\n') c = fgetc(file);
    } else {
        while (c != EOF && c != ',' && c != '\n' && c != '\r') {
            if (len + 1 < bufsize) buf[len++] = (char)c;
            c = fgetc(file);
        }
        if (c == '\r') {
            int c2 = fgetc(file);
            if (c2 != '\n' && c2 != EOF) ungetc(c2, file);
        }
    }
    buf[len] = 0;
}


```



</details>

<!-- END generated tutorial source -->
