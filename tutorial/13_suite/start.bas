10 ' BASCAL generated BASIC
20 ' Functions are lowered to global variables, labels, and GOSUB
30 COMMON count%, label$

40 ' Tutorial 13 — Suite COMMON, program 1 of 2
50 ' 
60 ' "program name suite suitename" tells bcc to load suitename.bcl and
70 ' emit its COMMON declarations at the very top of the generated output.
80 ' All programs in the same suite emit the same COMMON block, so the
90 ' variables survive a CHAIN to the next program.
100 ' 
110 ' Compile:
120 ' bcc tutorial/13_suite/start.bcl
130 ' 
140 ' The generated .bas will open with COMMON count%, label$ followed by
150 ' the program body below.

160 label$ = "Counter demo"
170 count% = 0

180 count% = count% + 1
190 count% = count% + 1
200 count% = count% + 1

210 PRINT "Initialised: " + label$
220 PRINT "Count after 3 increments: " + STR$(count%)

230 ' In a real multi-program application you would chain to show.bas:
240 ' CHAIN "show.bas"
250 END
