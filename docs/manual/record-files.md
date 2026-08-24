[Home](../../) / [Manual](../) / Record Files

[← Random-Access File I/O](random-access-file-io.md) [Data Statements →](data-statements.md)

<div class="prose" markdown="1">

From Part 2 of `tutorial/15_random_and_record_files.bcl`:

The `record` / `file` DSL is sugar over everything in [Random-Access File I/O](random-access-file-io.md#random-access-file-io) above. It computes the record's byte width, allocates the file number, and generates the `OPEN`/`FIELD`/`LSET`/`RSET`/`PUT`/`GET`/`MKx`/`CVx` calls for you — nothing about the *generated* BASIC changes; only the BASCAL source you write does.

### record ... end record

Declares a fixed-layout record type:

```bascal
record Student
    id:    int16
    name:  string(20)
    score: float64
end record
```

Supported field types and their packed width: `int16` (2 bytes), `int32` (4 bytes), `float32` (4 bytes), `float64` (8 bytes), `string(N)` (N bytes). The record's total width — used as the `OPEN ... LEN =` value — is the sum of its field widths, in declaration order.

### file ... as ... = open(...)

```bascal
file db as Student = open("students.dat")
```

Transpiles to one `OPEN ... FOR RANDOM AS #n LEN = <width>` plus one matching `FIELD #n, ...` statement, binding one string buffer variable per field. File numbers are allocated automatically, starting at `#1`, in the order `file` declarations appear in the source.

### Whole-record write

```bascal
db[1] = { id: 1, name: "Alice", score: 95.0 }
```

Every declared field must be supplied exactly once. Transpiles to one `LSET` per field — numeric fields are packed first (`MKI$`/`MKL$`/`MKS$`/`MKD$`), string fields are assigned directly — followed by a single `PUT #n, 1`. `LSET` is used for every field, numeric or string: once a numeric value is packed, the result is exact-width binary, so left/right justification makes no difference (this matches real BASCOM practice).

Note `MKx$` always carries a `$` suffix, never a type suffix matching the value being packed (`MKI%`, `MKD#`, etc. are not real MBASIC/BASCOM functions) — every `MKx$` variant returns a string, which is what `LSET` requires.

A record literal missing a declared field is a **transpile-time error** — this is a safety net that real BASIC's raw `FIELD`/`LSET`/`PUT` gives you no equivalent of (see [Partial-record write](#partial-record-write) for the deliberately-incomplete form).

### Partial-record write

```bascal
db[2] = ?{ score: 61.5 }
```

`?{ ... }` is `{ ... }`'s deliberately-incomplete counterpart: any subset of fields is allowed, and unlisted fields are left untouched on disk rather than erroring. `?` doesn't collide with anything — it isn't tokenized at all outside this position.

Whether the fields you *didn't* mention need preserving is fully decided at transpile time, by comparing the field names you gave against the record's declared fields — there's no runtime check:

- If the listed fields don't cover every declared field, an implicit `GET #n, i` is emitted first (so the unlisted fields keep their current on-disk values), then `LSET` for only the fields given, then `PUT #n, i`.
- If the listed fields happen to cover every declared field anyway, no `GET` is emitted — it transpiles exactly like a plain `{ ... }` literal.

Unlike `{ ... }`, an unknown field name inside `?{ ... }` is still a transpile-time error — only *missing* fields are permitted, not *misspelled* ones.

Note: `GET`ing a record number past the current end of a random-access file doesn't error in real BASIC (records can be sparse), but the fields you meant to "preserve" will simply read back as zero/blank the first time a given record number is touched, since there was nothing on disk yet to preserve.

### Whole-record read

```bascal
let s = db[i]
```

Transpiles to `GET #n, i` followed by one unpacking assignment per field (`CVI`/`CVL`/`CVS`/`CVD` for numeric fields, taking no suffix at all on real MBASIC/BASCOM), each one written into a scalar named `<var><Field>` — e.g. `sId%`, `sName$`, `sScore#`. Later references to `s.id`, `s.name`, `s.score` in the source resolve directly to those scalars; no `Ident` named literally `s.id` is ever emitted.

String fields aren't unpacked with `RTRIM$` — it isn't a real MBASIC/BASCOM builtin. Instead, the transpiler builds an inline right-trim loop directly from `LEN`/`MID$`/`LEFT$`, walking back from the end of the fixed-width buffer past trailing spaces.

Because BASIC doesn't auto-convert numbers to strings for concatenation, writing a numeric field next to a string with `+` (as in `print "[" + s.id + "]"`) automatically wraps the numeric side in `STR$(...)` — but only where a record field is actually involved; ordinary BASCAL `+` expressions are untouched.

Once `s` exists, `s.field = value` reassigns only the in-memory scalar (`s_field`) — it does **not** touch the file. Assignment alone never causes disk I/O; see [Writing a record variable back](#writing-a-record-variable-back) for the explicit commit step.

### Partial-field update

```bascal
db[i].field = value
```

For a single field, on its own, this is the terse form: it transpiles to an implicit `GET #n, i`, a single `LSET` for just that field, then `PUT #n, i`.

This form does its own `GET`/`PUT` every time it appears, so chaining several of them against the same record index costs one full round trip per field. To change several fields on one record with a single `GET`/`PUT`, either use [a partial-record write](#partial-record-write) (`db[i] = ?{ ... }`) with several fields at once, or read it into a variable first and write it back once — see below.

### Writing a record variable back

```bascal
let s = db[i]
s.name  = "Alicia"
s.score = 99.0
db[i] = s
```

`db[i] = s` — where `s` was bound by an earlier `let s = db[...]` — packs every field straight from `s`'s scalars and issues a single `PUT #n, i`, regardless of how many of `s`'s fields were reassigned first. Combined with the fact that `s.field = value` is pure in-memory assignment, this is the one-`GET`-one-`PUT` way to change multiple fields: exactly one `GET` (from the `let`) and one `PUT` (from the write-back) no matter how many fields in between were changed. `s` must have been read from a `file` of the same record type as the target; writing an `A` into a `file` of `B`s is a transpile-time error.

### file.close()

```bascal
db.close()
```

Transpiles to `CLOSE #n`.

### downto

```bascal
for i = 3 downto 1
    ...
end for
```

Sugar for `for i = 3 to 1 step -1`; ascending `for i = A to B` is unchanged.

### Type checking

The transpilation pass rejects, at transpile time: field names not declared on the record (in a record literal or a `.field` access), a record literal that is missing a declared field or repeats one, a string literal that is wider than its `string(N)` field, a string literal assigned to a numeric field (or vice versa), an unknown record type named by `file ... as ...`, and any reference to a `file` or `let`-bound record variable that was never declared.

</div>

[← Random-Access File I/O](random-access-file-io.md) [Data Statements →](data-statements.md)
