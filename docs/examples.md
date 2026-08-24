[Home](./) / Examples

<div class="prose" markdown="1">

The [tutorials](tutorials/) introduce BASCAL one construct at a time. These three programs are complete, runnable applications instead — each one picking a real task to exercise a different part of the language end to end.

</div>

<div class="snippet" markdown="1">

### Sort driver

[`tutorial/sort_driver.bcl`](https://github.com/johnjoeallen/bascal/blob/main/tutorial/sort_driver.bcl) exercises recursive `require`, array argument passing, and timing: it fills 5000 reverse-sorted elements (worst case for comparison sorts) and runs four sort implementations — bubble, shaker, shell, and quick — each pulled in from [`tutorial/com/bascal/sort`](https://github.com/johnjoeallen/bascal/tree/main/tutorial/com/bascal/sort) by dotted path.

```bascal
bcc tutorial/sort_driver.bcl
fbc -lang qb tutorial/sort_driver.bas -x tmp/sort_driver
./tmp/sort_driver
```

Expected output (timings vary by machine):

```bascal
Bubble sort time (ms):       ~200
Bubble: OK
Shaker sort time (ms):       ~180
Shaker: OK
Shell sort time (ms):        ~1
Shell: OK
Quick sort time (ms):        ~1
Quick: OK
```

</div>

<div class="snippet" markdown="1">

### REMLINE

[`tutorial/remline/remline.bcl`](https://github.com/johnjoeallen/bascal/blob/main/tutorial/remline/remline.bcl) is a real-world BASCAL program inspired by old BASIC line-number utilities. It analyses a line-numbered BASIC program and removes unnecessary line numbers while preserving every line that's still a real jump target — parsing, reference collection, and string helpers each arrive through their own `require`. It reads [`tutorial/remline/sample/input.bas`](https://github.com/johnjoeallen/bascal/blob/main/tutorial/remline/sample/input.bas) and writes the cleaned listing to `tutorial/remline/sample/output.bas`.

```bascal
bcc tutorial/remline/remline.bcl -L tutorial/remline
fbc -lang qb tutorial/remline/remline.bas -x tmp/remline
./tmp/remline
diff -u tutorial/remline/sample/expected.bas tutorial/remline/sample/output.bas
```

A clean run produces no diff output — `output.bas` matches the checked-in `expected.bas` exactly.

</div>

<div class="snippet" markdown="1">

### Card catalog

[`tutorial/card_catalog.bcl`](https://github.com/johnjoeallen/bascal/blob/main/tutorial/card_catalog.bcl) is the flagship [record/file DSL](manual/record-files.md) example: two record types (`Header`, `Entry`) sharing one random-access file, and five `procedure`s (`addItem`, `listAll`, `searchByAuthor`, `searchByAuthorTitle`, `deleteItem`) that each read and write those records from inside their own body — exercising record/file access from procedure scope, not just top-level code. A `mainMenu` procedure drives them from an interactive, `INPUT`-based menu loop. It's adapted from `CLERK.BAS`, a 1983 card-catalog manager by Carlos A. Lujan S.; see the comment header in the source for the full attribution and porting notes.

```bascal
bcc tutorial/card_catalog.bcl
fbc -lang qb tutorial/card_catalog.bas -x tmp/card_catalog
./tmp/card_catalog   # interactive -- follow the on-screen menu
```

</div>

<div class="prose" markdown="1">

Looking for the shared-`COMMON`/`CHAIN` example instead? See [Shared COMMON tutorial](tutorials/13_shared.md) — two programs coordinating score, level, and player state across a `CHAIN`.

</div>
