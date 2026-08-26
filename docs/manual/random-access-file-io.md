[Home](../../) / [Manual](../) / Random-Access File I/O

[← File Input and Output](file-input-and-output.md) [Record Files →](record-files.md)

<div class="prose" markdown="1">

From Part 1 of `tutorial/15_random_and_record_files.bcl`:

Random-access files store fixed-length records that can be read or written in any order, without scanning from the beginning.

BASCAL supports the classic statements below directly — `OPEN ... FOR RANDOM`, `FIELD`, `LSET`/`RSET`, `PUT`/`GET`, `SEEK`, and the `MKx`/`CVx` packing helpers all pass through as-is. But hand-summing field widths and hand-matching pack/unpack calls is exactly the bookkeeping a transpiler should do for you: see [Record Files](record-files.md#record-files) below for BASCAL's typed `record`/`file` syntax, the canonical way to do random-access I/O in BASCAL. This section stays useful for reading the code that syntax generates, or for files whose layout doesn't fit a fixed record type. A hand-typed `FIELD` statement still compiles, but `bcc` prints an advisory warning naming `record`/`file` as the preferred spelling — see [Legacy-Form Warnings](miscellaneous-statements.md#legacy-form-warnings).

### OPEN FOR RANDOM

```bascal
open filename$ for random as #1 len = recLen%
```

`len` sets the record size in bytes. Every record occupies exactly that many bytes on disk. Records are numbered from 1.

### FIELD

Binds string variables to regions of the shared file buffer:

```bascal
field #1, 2 as idBuf$, 20 as nameBuf$, 8 as scoreBuf$
```

The widths must sum to the record length. Only string variables may appear in a `FIELD` statement.

### LSET and RSET

Copy data into a field-bound buffer variable, padded to the field width:

```bascal
lset nameBuf$ = "Alice"    ' left-justified, padded with spaces on the right
rset idBuf$   = "42"       ' right-justified, padded with spaces on the left
```

### PUT and GET

Write or read a numbered record:

```bascal
put #1, recordNum%    ' write current buffer as record recordNum%
get #1, recordNum%    ' load record recordNum% into buffer variables
```

Omitting the record number reads/writes at the current file position.

### SEEK

Move the file pointer to a given record position:

```bascal
seek #1, recordNum%
```

### Packing Helpers

Numeric values must be packed into strings before storing in a `FIELD` buffer, and unpacked after reading:

| Pack       | Unpack     | Type           |
|------------|------------|----------------|
| `mki%(n%)` | `cvi%(s$)` | 2-byte integer |
| `mkl&(n&)` | `cvl&(s$)` | 4-byte long    |
| `mks!(n!)` | `cvs!(s$)` | 4-byte single  |
| `mkd#(n#)` | `cvd#(s$)` | 8-byte double  |

Example — writing and reading a numeric score:

```bascal
const rec_len% = 30

open "students.dat" for random as #1 len = rec_len%
field #1, 2 as idBuf$, 20 as nameBuf$, 8 as scoreBuf$

lset idBuf$    = mki%(1)
lset nameBuf$  = "Alice"
lset scoreBuf$ = mkd#(95.0)
put #1, 1

get #1, 1
print nameBuf$.rtrim() + ": " + str$(cvd#(scoreBuf$))
close #1
```

Output:

```bascal
Alice: 95
```

</div>

[← File Input and Output](file-input-and-output.md) [Record Files →](record-files.md)
