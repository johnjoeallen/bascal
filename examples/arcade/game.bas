' BASCAL generated BASIC
' Functions are lowered to global variables, labels, and GOSUB
COMMON score%, level%, playerName$
COMMON hiScore%

' ARCADE suite — game program.
' Assumes the COMMON variables were initialised by menu.bas and survived
' the CHAIN.  Simulates one round: awards points, advances the level, and
' tracks the all-time high score.

PRINT "Player: " + playerName$
PRINT "Level:  " + STR$(level%)

score% = score% + (50 * level%)
level% = level% + 1

IF (score% > hiScore%) = 0 THEN GOTO 10
    hiScore% = score%
    PRINT ("*** New high score: " + STR$(hiScore%)) + " ***"
10 REM END IF

PRINT "Score: " + STR$(score%)

' Chain back to menu or to the next level.
' CHAIN "menu.bas"
END
