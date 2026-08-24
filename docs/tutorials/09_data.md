[Home](../) / [Tutorials](./) / Data, Read, Restore, Swap

<div class="prose" markdown="1">

`data` embeds literal values directly in the program; `read` consumes them in sequence; `restore` rewinds the pointer so the same data can be read again. `data` statements can appear anywhere in the source — the generated BASIC places them after `END`. `swap` exchanges two variables (including array elements) without a temporary.

</div>

<div class="snippet" markdown="1">

### Loading a table with READ

```bascal
for i% = 1 to num_capitals%
    read country$(i%), capital$(i%)
end for

' ...

data "France",  "Paris"
data "Germany", "Berlin"
data "Japan",   "Tokyo"
```

</div>

<div class="snippet" markdown="1">

### SWAP, including array elements

```bascal
swap a%, b%

' Bubble-sort using swap -- no temp variable needed
if country$(i%) > country$(i% + 1) then
    swap country$(i%), country$(i% + 1)
    swap capital$(i%), capital$(i% + 1)
end if
```

</div>



[← Arrays](08_arrays.md)  ·  [File Input and Output →](10_files.md)


<!-- BEGIN generated tutorial source -->

### `tutorial/09_data.bcl`

```bascal

// Tutorial — data, read, restore, swap, randomize
//
// data embeds literal values directly in the program.  read consumes
// them in sequence.  restore rewinds the pointer so data can be read
// again.  The data statements may appear anywhere in the program body;
// the generated BASIC places them after END.
//
// swap exchanges two variables atomically — no temporary needed.
//
// randomize seeds the BASIC RND function.  Pass timer for a
// time-based seed; pass a literal for reproducible results.
program data

const numCapitals% = 5

dim country$(numCapitals%)
dim capital$(numCapitals%)

/* Load the lookup table */
for i% = 1 to numCapitals%
    read country$(i%), capital$(i%)
end for

/* Print the table */
print "Country         Capital"
print "--------------- ---------------"
for i% = 1 to numCapitals%
    print country$(i%) + "        " + capital$(i%)
end for

/* restore lets us re-read from the beginning */
restore
read firstCountry$, firstCapital$
print "First entry re-read: " + firstCountry$ + " -> " + firstCapital$

/* swap — sort two variables without a temp */
a% = 42
b% = 17
print "Before swap: a=" + str$(a%) + " b=" + str$(b%)
swap a%, b%
print "After swap:  a=" + str$(a%) + " b=" + str$(b%)

/* Bubble-sort the country array using swap */
for pass% = 1 to numCapitals% - 1
    for i% = 1 to numCapitals% - pass%
        if country$(i%) > country$(i% + 1) then
            swap country$(i%), country$(i% + 1)
            swap capital$(i%), capital$(i% + 1)
        end if
    end for
end for
print "Sorted by country:"
for i% = 1 to numCapitals%
    print "  " + country$(i%) + " -> " + capital$(i%)
end for

/* randomize — seed with a literal for reproducible output */
randomize 99

end

data "France",  "Paris"
data "Germany", "Berlin"
data "Japan",   "Tokyo"
data "Brazil",  "Brasilia"
data "Egypt",   "Cairo"

```

### `tutorial/09_data.bas`

```basic

10 ' BASCAL generated BASIC
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Tutorial — data, read, restore, swap, randomize
40 '
50 ' data embeds literal values directly in the program.  read consumes
60 ' them in sequence.  restore rewinds the pointer so data can be read
70 ' again.  The data statements may appear anywhere in the program body;
80 ' the generated BASIC places them after END.
90 '
100 ' swap exchanges two variables atomically — no temporary needed.
110 '
120 ' randomize seeds the BASIC RND function.  Pass timer for a
130 ' time-based seed; pass a literal for reproducible results.

140 numcapitals% = 5

150 DIM country$(numcapitals%)
160 BCCT1% = numcapitals%
170 DIM capital$(numcapitals%)
180 BCCT2% = numcapitals%

190 ' Load the lookup table
200 FOR i% = 1 TO numcapitals%
210     READ country$(i%), capital$(i%)
220 NEXT i%

230 ' Print the table
240 PRINT "Country         Capital"
250 PRINT "--------------- ---------------"
260 FOR i% = 1 TO numcapitals%
270     PRINT (country$(i%) + "        ") + capital$(i%)
280 NEXT i%

290 ' restore lets us re-read from the beginning
300 RESTORE
310 READ firstcountry$, firstcapital$
320 PRINT (("First entry re-read: " + firstcountry$) + " -> ") + firstcapital$

330 ' swap — sort two variables without a temp
340 a% = 42
350 b% = 17
360 PRINT (("Before swap: a=" + STR$(a%)) + " b=") + STR$(b%)
370 SWAP a%, b%
380 PRINT (("After swap:  a=" + STR$(a%)) + " b=") + STR$(b%)

390 ' Bubble-sort the country array using swap
400 FOR pass% = 1 TO numcapitals% - 1
410     FOR i% = 1 TO numcapitals% - pass%
420         IF (country$(i%) > country$(i% + 1)) = 0 THEN GOTO 450
430             SWAP country$(i%), country$(i% + 1)
440             SWAP capital$(i%), capital$(i% + 1)
450         REM END IF
460     NEXT i%
470 NEXT pass%
480 PRINT "Sorted by country:"
490 FOR i% = 1 TO numcapitals%
500     PRINT (("  " + country$(i%)) + " -> ") + capital$(i%)
510 NEXT i%

520 ' randomize — seed with a literal for reproducible output
530 RANDOMIZE 99

540 END

550 DATA "France", "Paris"
560 DATA "Germany", "Berlin"
570 DATA "Japan", "Tokyo"
580 DATA "Brazil", "Brasilia"
590 DATA "Egypt", "Cairo"

```

<!-- END generated tutorial source -->
