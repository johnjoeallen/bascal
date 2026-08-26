10 ' BASCAL generated BASIC -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Tutorial — Variables and Constants
40 ' 
50 ' Every name in BASCAL ends with a type suffix that tells the runtime
60 ' how to store the value:
70 ' 
80 ' %   integer   — 16-bit signed, -32768 to 32767
90 ' $   string    — variable-length text
100 ' !   single    — 32-bit floating-point
110 ' #   double    — 64-bit floating-point
120 ' &   long      — 32-bit signed integer
130 ' 
140 ' All variables are global.  They spring into existence on first use;
150 ' dim (or its synonym declare) is needed only for arrays or when you
160 ' want to be explicit -- declare tends to read better for a plain
170 ' scalar, dim for an array.
180 ' 
190 ' const names a value that cannot change.  Use it for magic numbers
200 ' so the intent is clear and the value lives in one place.

210 maxSCORE% = 100
220 passMARK% = 60
230 appNAME$ = "Grade Checker"
240 taxRATE! = 0.2

250 ' Variable assignment uses =
260 playername$ = "Alice"
270 score% = 87
280 temperature! = 36.6

290 ' print mixes strings and numbers directly with ; (no str$() needed)
300 PRINT appNAME$
310 PRINT "Player:      "; playername$
320 PRINT "Score:       "; score%; "/ "; maxSCORE%
330 PRINT "Pass mark:   "; passMARK%
340 PRINT "Temperature: "; temperature!
350 PRINT "Tax rate:    "; taxRATE!

360 ' str$() is still available when you need to build a string value
370 greeting$ = "Score is " + STR$(score%)
380 PRINT greeting$

390 END
