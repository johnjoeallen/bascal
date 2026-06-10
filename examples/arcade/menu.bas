' BASCAL generated BASIC
' Functions are lowered to global variables, labels, and GOSUB
COMMON score%, level%, playerName$
COMMON hiScore%

' ARCADE suite — entry program.
' Greets the player and initialises the shared COMMON variables before
' chaining to game.bas.  All programs in the arcade suite share score%,
' level%, playerName$ and hiScore% through the COMMON block.

INPUT "Your name: "; playerName$
score% = 0
hiScore% = 0
level% = 1

PRINT ("Welcome, " + playerName$) + "!"
PRINT ("Starting at level " + STR$(level%)) + ".  Good luck."

' In a real multi-program suite you would CHAIN to game.bas here.
' CHAIN "game.bas"
END
