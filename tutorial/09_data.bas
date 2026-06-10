10 ' BASCAL generated BASIC
20 ' Functions are lowered to global variables, labels, and GOSUB

30 ' Tutorial 9 — DATA, READ, RESTORE, SWAP, RANDOMIZE
40 ' 
50 ' DATA embeds literal values directly in the program.  READ consumes
60 ' them in sequence.  RESTORE rewinds the pointer so data can be read
70 ' again.  The DATA statements may appear anywhere in the program body;
80 ' the generated BASIC places them after END.
90 ' 
100 ' SWAP exchanges two variables atomically — no temporary needed.
110 ' 
120 ' RANDOMIZE seeds the BASIC RND function.  Pass TIMER for a
130 ' time-based seed; pass a literal for reproducible results.

140 CONST NUM_CAPITALS% = 5

150 DIM country$(NUM_CAPITALS%)
160 DIM capital$(NUM_CAPITALS%)

170 ' Load the lookup table
180 FOR i% = 1 TO NUM_CAPITALS%
190     READ country$(i%), capital$(i%)
200 NEXT i%

210 ' Print the table
220 PRINT "Country         Capital"
230 PRINT "--------------- ---------------"
240 FOR i% = 1 TO NUM_CAPITALS%
250     PRINT (country$(i%) + "        ") + capital$(i%)
260 NEXT i%

270 ' RESTORE lets us re-read from the beginning
280 RESTORE
290 READ firstCountry$, firstCapital$
300 PRINT (("First entry re-read: " + firstCountry$) + " -> ") + firstCapital$

310 ' SWAP — sort two variables without a temp
320 a% = 42
330 b% = 17
340 PRINT (("Before SWAP: a=" + STR$(a%)) + " b=") + STR$(b%)
350 SWAP a%, b%
360 PRINT (("After SWAP:  a=" + STR$(a%)) + " b=") + STR$(b%)

370 ' Bubble-sort the country array using SWAP
380 FOR pass% = 1 TO NUM_CAPITALS% - 1
390     FOR i% = 1 TO NUM_CAPITALS% - pass%
400         IF (country$(i%) > country$(i% + 1)) = 0 THEN GOTO 430
410             SWAP country$(i%), country$(i% + 1)
420             SWAP capital$(i%), capital$(i% + 1)
430         REM END IF
440     NEXT i%
450 NEXT pass%
460 PRINT "Sorted by country:"
470 FOR i% = 1 TO NUM_CAPITALS%
480     PRINT (("  " + country$(i%)) + " -> ") + capital$(i%)
490 NEXT i%

500 ' RANDOMIZE — seed with a literal for reproducible output
510 RANDOMIZE 99

520 END

530 DATA "France", "Paris"
540 DATA "Germany", "Berlin"
550 DATA "Japan", "Tokyo"
560 DATA "Brazil", "Brasilia"
570 DATA "Egypt", "Cairo"
