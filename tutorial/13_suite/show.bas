10 ' BASCAL generated BASIC
20 ' Functions are lowered to global variables, labels, and GOSUB
30 COMMON count%, label$

40 ' Tutorial 13 — Suite COMMON, program 2 of 2
50 ' 
60 ' This program shares the same suite as start.bcl.  Its generated BASIC
70 ' will begin with the same COMMON block, so count% and label$ contain
80 ' whatever values start.bas left in them when it CHAINed here.
90 ' 
100 ' Compile:
110 ' bcc tutorial/13_suite/show.bcl

120 PRINT "Label:  " + label$
130 PRINT "Count:  " + STR$(count%)

140 IF (count% > 0) = 0 THEN GOTO 170
150     PRINT ("Counter was incremented " + STR$(count%)) + " time(s)."
160     GOTO 180
170     PRINT "Counter was never incremented."
180 REM END IF

190 END
