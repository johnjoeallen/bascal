[Home](../index.md) / Record files

<div id="tutorial" class="section" markdown="1">

## Record files: BASCAL's record/file syntax vs standard BASIC

BASCAL supports classic random-access file I/O directly (`OPEN ... FOR RANDOM`, `FIELD`, `GET`/`PUT`, `LSET`, `MKx`/`CVx` all still pass through as-is), but writing it by hand is exactly the kind of repetitive, error-prone bookkeeping a transpiler should do instead. The `record`/`file` syntax below is the canonical way to do this in BASCAL: every pane on the right is what you write, and it **generates** the pane on the left — this syntax doesn't change what runs, only how much of it you have to type and keep in sync yourself. Full source: [tutorial/random_and_record_files.bcl](https://github.com/johnjoeallen/bascal/blob/main/tutorial/random_and_record_files.bcl).

<div class="compare" markdown="1">

### 1. Declare the record shape and open the file

BASCAL sums the field widths for you and generates the matching `FIELD` binding — get one field's width wrong by hand and every record after it gets read or written off by that many bytes.

<div class="compare-grid" markdown="1">

<div class="pane old" markdown="1">

<span class="tag">Generated BASIC</span>

```bascal
const rec_len%  = 30   ' 2+20+8, by hand
const db_file$  = "students.dat"

open db_file$ for random as #1 len = rec_len%
field #1, 2 as idBuf$, 20 as nameBuf$, 8 as scoreBuf$
```

</div>

<div class="pane new" markdown="1">

<span class="tag">BASCAL</span>

```bascal
record Student
    id:    int16
    name:  string(20) lpad
    score: float64
end record

file db as Student = open("students.dat")
```

</div>

</div>

</div>

<div class="compare" markdown="1">

### 2. Write a whole record

Every declared field is required in the record literal — a field forgotten by hand ships silently; a field forgotten in `{ ... }` is a transpile error.

<div class="compare-grid" markdown="1">

<div class="pane old" markdown="1">

<span class="tag">Generated BASIC</span>

```bascal
lset idBuf$    = mki%(1)
lset nameBuf$  = "Alice"
lset scoreBuf$ = mkd#(95.0)
put #1, 1
```

</div>

<div class="pane new" markdown="1">

<span class="tag">BASCAL</span>

```bascal
db[1] = { id: 1, name: "Alice", score: 95.0 }
```

</div>

</div>

</div>

<div class="compare" markdown="1">

### 3. Read records back in reverse order

`downto` is sugar for `step -1`; `s.id`/`s.name`/`s.score` resolve straight to the unpacked scalars — no `cvi%`/`cvd#`/`rtrim$` to remember or mismatch.

<div class="compare-grid" markdown="1">

<div class="pane old" markdown="1">

<span class="tag">Generated BASIC</span>

```bascal
for i% = num_recs% to 1 step -1
    get #1, i%
    id%    = cvi%(idBuf$)
    score# = cvd#(scoreBuf$)
    print "[" + str$(id%) + "] " _
        + rtrim$(nameBuf$) + " -- " + str$(score#)
end for
```

</div>

<div class="pane new" markdown="1">

<span class="tag">BASCAL</span>

```bascal
for i = 3 downto 1
    let s = db[i]
    print "[" + s.id + "] " + s.name + " -- " + s.score
end for
```

</div>

</div>

</div>

<div class="compare" markdown="1">

### 4. Update a single field

Bob just scraped a pass on re-mark — only the score changes, but `PUT` always writes the whole buffer, so a `GET` has to come first either way.

<div class="compare-grid" markdown="1">

<div class="pane old" markdown="1">

<span class="tag">Generated BASIC</span>

```bascal
get #1, 2
lset scoreBuf$ = mkd#(61.5)
put #1, 2
```

</div>

<div class="pane new" markdown="1">

<span class="tag">BASCAL</span>

```bascal
db[2].score = 61.5
```

</div>

</div>

</div>

<div class="compare" markdown="1">

### 5. Update two fields in one shot

Alice got married and re-sat the exam. Whether the fields you *didn't* list need a `GET` first is decided at **transpile time** — the transpiler compares the field names you gave against the record's declared fields. Give it every field and the `GET` disappears entirely, same as a plain `{ ... }`. Misspell a field name and it's still a transpile error, not silent data loss.

<div class="compare-grid" markdown="1">

<div class="pane old" markdown="1">

<span class="tag">Generated BASIC</span>

```bascal
get #1, 1
lset nameBuf$  = "Alice Smith"
lset scoreBuf$ = mkd#(91.0)
put #1, 1
```

</div>

<div class="pane new" markdown="1">

<span class="tag">BASCAL</span>

```bascal
db[1] = ?{ name: "Alice Smith", score: 91.0 }
```

</div>

</div>

<div class="tally" markdown="1">

**Generated BASIC:** 4 lines, 3 buffer names, 1 pack call, repeated per edit **BASCAL:** 1 line, still exactly one `GET` + one `PUT` generated

</div>

> **Why `?{ ... }`?** A few spellings for "partial record literal" were considered — `.{ ... }`, `<-{ ... }` — but `?` won because it reads as "this might be incomplete," the same sense it carries for optional values in most other languages, and it was free: nothing else in BASCAL's grammar used a bare `?`, so adding it couldn't collide with or reinterpret any existing program. It's also deliberately a *second* spelling rather than a relaxed `{ ... }` — keeping `{ ... }` strict means a record literal that's missing a field by accident is still always a transpile error. `?{ ... }` exists so that incompleteness has to be opted into explicitly, one call site at a time, instead of silently allowed everywhere.

</div>

<div class="compare" markdown="1">

### 6. Batched update via read → mutate → write back

Same one-`GET`-one-`PUT` shape as `?{ ... }` above, spelled as read/mutate/write-back instead — useful when the new values aren't just one-line literals. `s.field = value` alone is pure in-memory assignment; nothing touches disk until the final write-back.

<div class="compare-grid" markdown="1">

<div class="pane old" markdown="1">

<span class="tag">Generated BASIC</span>

```bascal
get #1, 3
lset nameBuf$  = "Carol Jones"
lset scoreBuf$ = mkd#(88.0)
put #1, 3
```

</div>

<div class="pane new" markdown="1">

<span class="tag">BASCAL</span>

```bascal
let carol = db[3]
carol.name  = "Carol Jones"
carol.score = 88.0
db[3] = carol
```

</div>

</div>

</div>

<div class="compare" markdown="1">

### 7. Close

<div class="compare-grid" markdown="1">

<div class="pane old" markdown="1">

<span class="tag">Generated BASIC</span>

```bascal
close #1
```

</div>

<div class="pane new" markdown="1">

<span class="tag">BASCAL</span>

```bascal
db.close()
```

</div>

</div>

</div>

Nothing above runs slower — every line here transpiles to exactly the same `OPEN`/`FIELD`/`LSET`/`PUT`/`GET`/`MKx`/`CVx` calls shown on the left, generated for you instead of typed and kept in sync by hand. See the [Record Files section of the manual](../manual/record-files.md) for the full semantics, including the exact static rule for when a partial write needs a `GET`.

</div>
