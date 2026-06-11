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
' dim is needed only for arrays or when you want to be explicit.
' 
' const names a value that cannot change.  Use it for magic numbers
' so the intent is clear and the value lives in one place.

CONST max_score% = 100
CONST pass_mark% = 60
CONST app_name$ = "Grade Checker"
CONST tax_rate! = 0.2

' Variable assignment uses =
playername$ = "Alice"
score% = 87
temperature! = 36.6

' print mixes strings and numbers directly with ; (no str$() needed)
PRINT app_name$
PRINT "Player:      "; playername$
PRINT "Score:       "; score%; "/ "; max_score%
PRINT "Pass mark:   "; pass_mark%
PRINT "Temperature: "; temperature!
PRINT "Tax rate:    "; tax_rate!

' str$() is still available when you need to build a string value
greeting$ = "Score is " + STR$(score%)
PRINT greeting$

END
