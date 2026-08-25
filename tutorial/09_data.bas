10 ' BASCAL generated BASIC -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Tutorial — DATA, READ, SWAP, and RANDOMIZE
40 ' 
50 ' data embeds literal values directly in the program.  read consumes
60 ' them in sequence.  The data statements may appear anywhere in the program body;
70 ' the generated BASIC places them after END.
80 '
90 ' swap exchanges two variables atomically — no temporary needed.
100 '
110 ' randomize seeds the BASIC RND function.  Pass timer for a
120 ' time-based seed; pass a literal for reproducible results.

130 numcapitals% = 5

140 DIM country$(numcapitals%)
150 BCCT1% = numcapitals%
160 DIM capital$(numcapitals%)
170 BCCT2% = numcapitals%

180 ' Load the lookup table
190 FOR i% = 1 TO numcapitals%
200     READ country$(i%), capital$(i%)
210 NEXT i%

220 ' Print the table
230 PRINT "Country         Capital"
240 PRINT "--------------- ---------------"
250 FOR i% = 1 TO numcapitals%
260     PRINT (country$(i%) + "        ") + capital$(i%)
270 NEXT i%

280 ' swap — sort two variables without a temp
290 a% = 42
300 b% = 17
310 PRINT (("Before swap: a=" + STR$(a%)) + " b=") + STR$(b%)
320 SWAP a%, b%
330 PRINT (("After swap:  a=" + STR$(a%)) + " b=") + STR$(b%)

340 ' Bubble-sort the country array using swap
350 FOR pass% = 1 TO numcapitals% - 1
360     FOR i% = 1 TO numcapitals% - pass%
370         IF (country$(i%) > country$(i% + 1)) = 0 THEN GOTO 400
380             SWAP country$(i%), country$(i% + 1)
390             SWAP capital$(i%), capital$(i% + 1)
400         REM END IF
410     NEXT i%
420 NEXT pass%
430 PRINT "Sorted by country:"
440 FOR i% = 1 TO numcapitals%
450     PRINT (("  " + country$(i%)) + " -> ") + capital$(i%)
460 NEXT i%

470 ' randomize — seed with a literal for reproducible output
480 RANDOMIZE 99

490 END

500 DATA "France", "Paris"
510 DATA "Germany", "Berlin"
520 DATA "Japan", "Tokyo"
530 DATA "Brazil", "Brasilia"
540 DATA "Egypt", "Cairo"
