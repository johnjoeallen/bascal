' BASCAL generated BASIC
' Functions are lowered to global variables, labels, and GOSUB

' Tutorial 2 — Variables and Constants
' 
' Every name in BASCAL ends with a type suffix that tells the runtime
' how to store the value:
' 
' %   integer   — 16-bit signed, -32768 to 32767
' $   string    — variable-length text
' !   single    — 32-bit floating-point
' #   double    — 64-bit floating-point
' &   long      — 32-bit signed integer
' 
' All variables are global.  They spring into existence on first use;
' DIM is needed only for arrays or when you want to be explicit.
' 
' CONST names a value that cannot change.  Use it for magic numbers
' so the intent is clear and the value lives in one place.

CONST MAX_SCORE% = 100
CONST PASS_MARK% = 60
CONST APP_NAME$ = "Grade Checker"
CONST TAX_RATE! = 0.2

' Variable assignment uses =
playerName$ = "Alice"
score% = 87
temperature! = 36.6

' STR$() converts a number to a string for concatenation
PRINT APP_NAME$
PRINT "Player:      " + playerName$
PRINT (("Score:       " + STR$(score%)) + " / ") + STR$(MAX_SCORE%)
PRINT "Pass mark:   " + STR$(PASS_MARK%)
PRINT "Temperature: " + STR$(temperature!)
PRINT "Tax rate:    " + STR$(TAX_RATE!)

' LET is optional; both forms are identical
greeting$ = "Score is " + STR$(score%)
PRINT greeting$

END
