[Home](../../) / [Manual](../) / Data Statements

[← Record Files](record-files.md) [Miscellaneous Statements →](miscellaneous-statements.md)

<div class="prose" markdown="1">

`DATA`, `READ`, and `RESTORE` provide an embedded data table read at run time. `DATA` statements may appear anywhere in the program body; the generated BASIC places them after `END`.

From `tutorial/data.bcl`:

```bascal
CONST NUM_CAPITALS = 5

DIM country$(NUM_CAPITALS)
DIM capital$(NUM_CAPITALS)

for i% = 1 to NUM_CAPITALS
    READ country$(i%), capital$(i%)
end for

PRINT "Country         Capital"
PRINT "--------------- ---------------"
for i% = 1 to NUM_CAPITALS
    PRINT country$(i%) + "        " + capital$(i%)
end for

' RESTORE rewinds to the first DATA element
RESTORE
READ firstCountry$, firstCapital$
PRINT "First entry re-read: " + firstCountry$ + " -> " + firstCapital$

END

DATA "France",  "Paris"
DATA "Germany", "Berlin"
DATA "Japan",   "Tokyo"
DATA "Brazil",  "Brasilia"
DATA "Egypt",   "Cairo"
```

### RESTORE

Resets the `DATA` pointer to the beginning (or to a specific label).

```bascal
RESTORE           ' rewind to the first DATA
RESTORE fromHere  ' rewind to the DATA right after the `fromHere:` label
```

</div>

[← Record Files](record-files.md) [Miscellaneous Statements →](miscellaneous-statements.md)
