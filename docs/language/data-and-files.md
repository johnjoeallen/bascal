## Two shapes of persistent data

Some data is naturally a stream: lines of text, written once and read back in order. Other data is naturally a table: fixed-size records you want to reach by number, without reading everything before them. BASCAL supports both. Sequential files use the same statements as classic BASIC; random-access data has a structured form of its own, described below.

## Sequential files

```bascal
const scoreFile$ = "scores.csv"

file scores = open(scoreFile$) for output
scores.write("Ada", 98.5)
scores.write("Grace", 92.0)
scores.close()

file readScores = open(scoreFile$) for input
while not readScores.eof()
    readScores.read(name$, score!)
    print name$; ": "; score!
end while
readScores.close()
```

`file <var> = open(<path>) for output` creates a file and truncates it if it already exists; `for append` adds to the end instead; `for input` opens an existing one for reading. The compiler picks the channel number itself — nothing about it appears in the source — so `<var>.write(...)`, `<var>.read(...)`, and `<var>.eof()` never need one either. `write` stores each value in a quoted, comma-separated form that `read` reads back exactly, including strings that themselves contain commas. `<var>.close()` closes the file exactly like closing any other open channel — the same method works on a record file too.

A file’s declared direction is enforced when the program is compiled: calling `.read(...)` or `.eof()` on a file opened `for output`, or `.write(...)` on one opened `for input`, is rejected before it ever reaches a real file — the same kind of mistake real BASIC would only catch once something went wrong at runtime.

## Random-access files

For fixed-size random-access data, BASCAL lets you express the record layout in the program instead of tracking byte offsets by hand. The compiler takes care of the target’s `FIELD`, conversion, and `GET`/`PUT` sequence.

```bascal
record Student
    id:      int16
    name:    string(20) lpad
    score:   float64
    faculty: string(20) lpad
end record

file students as Student = open("students.dat")
students[1] = { id: 1, name: "Alice", score: 95.0, faculty: "Engineering" }
```

The source says that a file contains students and that the first student has four named fields. It does not ask you to repeat field offsets and buffer assignments at every write. A `{ ... }` literal must supply every declared field, in any order; a value of the wrong type, or a string too wide for its `string(N)` field, is rejected when the program is compiled rather than discovered later against real data.

Records may also declare a plain `string` member when they are used in memory. Such a member is variable-length and therefore has no packed byte width; a record containing one cannot be used in a random-access `file` declaration.

## Changing part of a record

Rewriting an entire record just to change one field would mean reading it first by hand. `?{ ... }` does that for you: name only the fields you are changing, and the rest keep whatever is already on disk.

```bascal
students[1] = ?{ name: "Alice Smith", score: 91.0 }
```

`id` and `faculty` weren’t mentioned, so BASCAL decides at compile time whether they need preserving: since `name` and `score` don’t cover every declared field, it reads the record before writing the fields you named. Had you named every field, it would write straight through with no extra read. Either way, an unknown field name is still an error — `?{ ... }` allows fields to be missing, not misspelled.

## Reading a record back

```bascal
let a = students[1]
print a.name; " (" + a.faculty + ") scored "; a.score

a.score = a.score + 1.5
students[1] = a
students.close()
```

`let a = students[1]` reads record 1 into named fields you can use like any other value. Changing `a.score` here only changes the copy in memory; nothing reaches the file until you write `a` back with `students[1] = a`, which takes one read and one write no matter how many of `a`’s fields you touched in between — `id` and `faculty` go along for the ride unchanged.

That is different from writing a field directly against the file, as in `students[1].field = value`: with no `let` in between, that form reads and writes the record immediately, on the spot. Prefer it for a single field changed once; prefer reading into a variable first when you are changing several fields together. `students.close()` closes the file exactly like closing any other open channel.

## What the record/file DSL is sugar over

Nothing about `record`/`file` is magic — it generates the same statements classic BASIC has always used for random access, just without asking you to write them by hand. The whole-record write from the first example above expands to this:

```bascal
open "students.dat" for random as #1 len = 50
field #1, 2 as idBuf$, 20 as nameBuf$, 8 as scoreBuf$, 20 as facultyBuf$

lset idBuf$ = mki$(1)
lset nameBuf$ = "Alice"
lset scoreBuf$ = mkd$(95.0)
lset facultyBuf$ = "Engineering"
put #1, 1

get #1, 1
print cvi(idBuf$); " "; cvd(scoreBuf$)
close #1
```

`open ... for random ... len = n` opens a file whose records are all exactly `n` bytes; `field` then carves that byte width into named string buffers. `lset` copies a value left-justified into one of those buffers, padding a string with trailing spaces to fill it exactly; a number has to be packed into the same fixed width first, with `mki$`/`mkl$`/`mks$`/`mkd$` for a 2/4/4/8-byte integer, single, or double. `put`/`get` then write or read one whole record by number, and `cvi`/`cvl`/`cvs`/`cvd` unpack a numeric buffer back into a usable value.

**Prefer `record`/`file` over writing this by hand.** Every piece above is bookkeeping that has to agree across every read and write of the same layout — the byte width, the buffer names, which pack/unpack function matches which field — and classic BASIC gives you no help keeping them in sync: a `field` statement that drifts from its file’s real `len`, or an `mkd$` swapped for an `mks$`, is a runtime bug that a working program can hide for a long time. `record` states the layout exactly once, and the compiler derives everything else from it — including catching a missing or misspelled field at compile time, long before it would reach a real file. Reach for the raw form only when a program genuinely needs to interoperate with an existing hand-rolled layout it doesn’t control.

## Keep the target available

Under `--target basic`, BASCAL remains a strict superset of the target dialect. Existing sequential file operations and hand-written BASIC file statements can pass through when you need them — the compiler never forces a rewrite onto working code just because a more structured way to say the same thing now exists. `--target c` supports the same random-access/sequential file I/O natively too; see [Portability across backends](../manual/command-line-reference.md#portability-across-backends) for where BASCAL is no longer a strict superset once `--target c`/`--target jvm` enter the picture.
