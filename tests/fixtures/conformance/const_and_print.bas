10 ' BASCAL generated BASIC
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Conformance fixture: CONST lowering, arithmetic, and string/number PRINT
40 ' concatenation -- exercises the CONST-is-not-real-BASCOM fix directly.

50 maxscore% = 100
60 taxrate! = 0.2

70 score% = 87
80 bonus% = 5
90 total% = score% + bonus%

100 PRINT "Score: "; total%; " / "; maxscore%
110 PRINT "Tax:   "; taxrate!

120 IF (total% > maxscore%) = 0 THEN GOTO 150
130     PRINT "Capped"
140     GOTO 160
150     PRINT "Under cap"
160 REM END IF

170 FOR i% = 1 TO 3
180     PRINT "Lap "; i%
190 NEXT i%

200 END
