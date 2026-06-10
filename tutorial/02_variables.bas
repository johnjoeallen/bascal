10 ' BASCAL generated BASIC
20 ' Functions are lowered to global variables, labels, and GOSUB

30 ' Tutorial 2 — Variables and Constants
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
150 ' dim is needed only for arrays or when you want to be explicit.
160 ' 
170 ' const names a value that cannot change.  Use it for magic numbers
180 ' so the intent is clear and the value lives in one place.

190 CONST max_score% = 100
200 CONST pass_mark% = 60
210 CONST app_name$ = "Grade Checker"
220 CONST tax_rate! = 0.2

230 ' Variable assignment uses =
240 playername$ = "Alice"
250 score% = 87
260 temperature! = 36.6

270 ' str$() converts a number to a string for concatenation
280 PRINT app_name$
290 PRINT "Player:      " + playername$
300 PRINT (("Score:       " + STR$(score%)) + " / ") + STR$(max_score%)
310 PRINT "Pass mark:   " + STR$(pass_mark%)
320 PRINT "Temperature: " + STR$(temperature!)
330 PRINT "Tax rate:    " + STR$(tax_rate!)

340 ' let is optional; both forms are identical
350 greeting$ = "Score is " + STR$(score%)
360 PRINT greeting$

370 END
