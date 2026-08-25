10 ' BASCAL generated BASIC -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Tutorial — DATA, READ, and RESTORE
40 ' 
50 ' RESTORE rewinds the DATA pointer to a named DATA block.  Labels keep the
60 ' source readable while the transpiler assigns the generated line numbers.

70 PRINT "restore to a label:"
80 READ firstcountry$
90 PRINT "  first read: "; firstcountry$
100 RESTORE 150
110 READ secondcountry$
120 PRINT "  after restore secondBatch: "; secondcountry$

130 END

140 DATA "France"

150 DATA "Japan"
