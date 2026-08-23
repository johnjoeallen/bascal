10 ' BASCAL generated BASIC
20 ' Functions are transpiled to global variables, labels, and GOSUB
30 COMMON count%, label$

40 ' Tutorial — Shared COMMON, program 2 of 2
50 ' 
60 ' This program references the same shared file as start.bcl.  Its
70 ' generated BASIC will begin with the same COMMON block, so count% and
80 ' label$ contain whatever values start.bas left in them when it CHAINed
90 ' here.
100 ' 
110 ' Compile:
120 ' bcc tutorial/13_shared/show.bcl

130 PRINT "Label:  " + label$
140 PRINT "Count:  " + STR$(count%)

150 IF (count% > 0) = 0 THEN GOTO 180
160     PRINT ("Counter was incremented " + STR$(count%)) + " time(s)."
170     GOTO 190
180     PRINT "Counter was never incremented."
190 REM END IF

200 END
