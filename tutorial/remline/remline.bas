10 ' BASCAL generated BASIC
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Upper-cases self$. Not a real MBASIC/BASCOM 2.00 builtin -- verified
40 ' against a real IBM BASIC Compiler 2.00 under dosbox-x -- so BASCAL ships
50 ' its own. Declared as a scalar method (see GitHub issue #41 and
60 ' ltrim.bcl's own doc comment for the reasoning) -- ucase$(s$) still works
70 ' via ordinary-call syntax resolving to this same declaration.

80 ' Shared string helpers for REMLINE.

90 ' UCASE$ isn't a real MBASIC/BASCOM 2.00 builtin (verified against a real
100 ' IBM BASIC Compiler 2.00 under dosbox-x), so upper$() below needs BASCAL's
110 ' own com.bascal.stdlib implementation rather than assuming the target
120 ' dialect provides one.

130 ' text$ -- string to trim

140 ' text$ -- string to uppercase

150 ' text$    -- string to test
160 ' keyword$ -- keyword to look for at the start of text$

170 ' Parse and strip leading decimal line numbers.

180 ' text$ -- source line to read a leading line number from

190 ' text$ -- source line to strip a leading line number from

200 ' Fixed-size reference tracking for the example.

210 ' lineNo% -- line number to record as referenced

220 ' lineNo% -- line number to test

230 ' line$ -- source line to scan for GOTO/GOSUB/THEN/... targets

240 ' line$    -- source line to scan
250 ' keyword$ -- keyword to look for (e.g. "GOTO")

260 ' REMLINE works on an input BASIC listing and writes a cleaned version.

270 DIM rawline$(1000)
280 DIM linetext$(1000)
290 DIM linenumber%(1000)
300 DIM keepline%(1000)
310 DIM refnumber%(1000)

320 ' REMLINE demo driver.
330 ' This version reads a line-numbered BASIC file and writes a cleaned copy.
340 ' The dependency graph is still real: the driver pulls in parsing, reference
350 ' collection, and string helpers through BASCAL's path-style require syntax.

360 inputfile$ = "tutorial/remline/sample/input.bas"
370 outputfile$ = "tutorial/remline/sample/output.bas"

380 PRINT "BASCAL REMLINE example"
390 PRINT "Input: " + inputfile$
400 PRINT "Output: " + outputfile$

410 GOSUB 2440
420 GOSUB 2690
430 GOSUB 2820

440 PRINT "Done"
450 END

460 ' function ucase$()
470     ucaseOut0$ = ""
480     FOR ucaseI0% = 1 TO LEN(ucaseSelf0$)
490         ucaseC0% = ASC(MID$(ucaseSelf0$, ucaseI0%, 1))
500         IF (ucaseC0% >= 97) = 0 THEN GOTO 530
510         IF (ucaseC0% <= 122) = 0 THEN GOTO 530
520             ucaseC0% = ucaseC0% - 32
530         REM END IF
540         ucaseOut0$ = ucaseOut0$ + CHR$(ucaseC0%)
550     NEXT ucaseI0%
560     ucaseResult0$ = ucaseOut0$
570     RETURN
580 ' end function ucase$

590 ' function trimleft$(text$)
600     ' Walk from the left until the first non-space character appears.
610     trimleftI0% = 1
620     IF (trimleftI0% <= LEN(trimleftText0$)) = 0 THEN GOTO 700
630         trimleftCh0$ = MID$(trimleftText0$, trimleftI0%, 1)
640         IF (trimleftCh0$ <> " ") = 0 THEN GOTO 670
650             trimleftResult0$ = MID$(trimleftText0$, trimleftI0%)
660             RETURN
670         REM END IF
680         trimleftI0% = trimleftI0% + 1
690         GOTO 620
700     REM END WHILE
710     trimleftResult0$ = ""
720     RETURN
730 ' end function trimleft$

740 ' function upper$(text$)
750     ucaseSelf0$ = upperText0$
760     GOSUB 470
770     upperResult0$ = ucaseResult0$
780     RETURN
790 ' end function upper$

800 ' function startswithkeyword%(text$, keyword$)
810     trimleftText0$ = startswithkeywordText0$
820     GOSUB 600
830     startswithkeywordT0$ = trimleftResult0$
840     startswithkeywordKw0$ = startswithkeywordKeyword0$
850     upperText0$ = startswithkeywordT0$
860     GOSUB 750
870     startswithkeywordT0$ = upperResult0$
880     upperText0$ = startswithkeywordKw0$
890     GOSUB 750
900     startswithkeywordKw0$ = upperResult0$
910     IF (LEN(startswithkeywordT0$) < LEN(startswithkeywordKw0$)) = 0 THEN GOTO 940
920         startswithkeywordResult0% = 0
930         RETURN
940     REM END IF
950     startswithkeywordResult0% = LEFT$(startswithkeywordT0$, LEN(startswithkeywordKw0$)) = startswithkeywordKw0$
960     RETURN
970 ' end function startswithkeyword%

980 ' function parselinenumber%(text$)
990     trimleftText0$ = parselinenumberText0$
1000     GOSUB 600
1010     parselinenumberText0$ = trimleftResult0$
1020     parselinenumberDigits0$ = ""
1030     parselinenumberI0% = 1
1040     parselinenumberDone0% = 0
1050     IF ((parselinenumberI0% <= LEN(parselinenumberText0$)) AND (parselinenumberDone0% = 0)) = 0 THEN GOTO 1140
1060         parselinenumberCh0$ = MID$(parselinenumberText0$, parselinenumberI0%, 1)
1070         IF ((parselinenumberCh0$ >= "0") AND (parselinenumberCh0$ <= "9")) = 0 THEN GOTO 1100
1080             parselinenumberDigits0$ = parselinenumberDigits0$ + parselinenumberCh0$
1090             GOTO 1110
1100             parselinenumberDone0% = 1
1110         REM END IF
1120         parselinenumberI0% = parselinenumberI0% + 1
1130         GOTO 1050
1140     REM END WHILE
1150     IF (LEN(parselinenumberDigits0$) = 0) = 0 THEN GOTO 1180
1160         parselinenumberResult0% = 0
1170         RETURN
1180     REM END IF
1190     parselinenumberResult0% = VAL(parselinenumberDigits0$)
1200     RETURN
1210 ' end function parselinenumber%

1220 ' function striplinenumber$(text$)
1230     trimleftText0$ = striplinenumberText0$
1240     GOSUB 600
1250     striplinenumberText0$ = trimleftResult0$
1260     striplinenumberI0% = 1
1270     striplinenumberDone0% = 0
1280     IF ((striplinenumberI0% <= LEN(striplinenumberText0$)) AND (striplinenumberDone0% = 0)) = 0 THEN GOTO 1360
1290         striplinenumberCh0$ = MID$(striplinenumberText0$, striplinenumberI0%, 1)
1300         IF ((striplinenumberCh0$ >= "0") AND (striplinenumberCh0$ <= "9")) = 0 THEN GOTO 1330
1310             striplinenumberI0% = striplinenumberI0% + 1
1320             GOTO 1340
1330             striplinenumberDone0% = 1
1340         REM END IF
1350         GOTO 1280
1360     REM END WHILE
1370     IF (striplinenumberI0% > LEN(striplinenumberText0$)) = 0 THEN GOTO 1400
1380         striplinenumberResult0$ = ""
1390         RETURN
1400     REM END IF
1410     IF (MID$(striplinenumberText0$, striplinenumberI0%, 1) = " ") = 0 THEN GOTO 1430
1420         striplinenumberI0% = striplinenumberI0% + 1
1430     REM END IF
1440     striplinenumberResult0$ = MID$(striplinenumberText0$, striplinenumberI0%)
1450     RETURN
1460 ' end function striplinenumber$

1470 ' function addref%(lineno%)
1480     IF (addrefLineNo0% = 0) = 0 THEN GOTO 1510
1490         addrefResult0% = 0
1500         RETURN
1510     REM END IF
1520     addrefI0% = 1
1530     IF (addrefI0% <= refcount%) = 0 THEN GOTO 1600
1540         IF (refnumber%(addrefI0%) = addrefLineNo0%) = 0 THEN GOTO 1570
1550             addrefResult0% = 0
1560             RETURN
1570         REM END IF
1580         addrefI0% = addrefI0% + 1
1590         GOTO 1530
1600     REM END WHILE
1610     IF (refcount% >= 1000) = 0 THEN GOTO 1640
1620         addrefResult0% = 0
1630         RETURN
1640     REM END IF
1650     refcount% = refcount% + 1
1660     refnumber%(refcount%) = addrefLineNo0%
1670     addrefResult0% = 1
1680     RETURN
1690 ' end function addref%

1700 ' function isreferenced%(lineno%)
1710     isreferencedI0% = 1
1720     IF (isreferencedI0% <= refcount%) = 0 THEN GOTO 1790
1730         IF (refnumber%(isreferencedI0%) = isreferencedLineNo0%) = 0 THEN GOTO 1760
1740             isreferencedResult0% = 1
1750             RETURN
1760         REM END IF
1770         isreferencedI0% = isreferencedI0% + 1
1780         GOTO 1720
1790     REM END WHILE
1800     isreferencedResult0% = 0
1810     RETURN
1820 ' end function isreferenced%

1830 ' function collectrefs%(line$)
1840     collectrefsFound0% = 0
1850     scankeywordrefsLine0$ = collectrefsLine0$
1860     scankeywordrefsKeyword0$ = "GOTO"
1870     GOSUB 2170
1880     collectrefsFound0% = collectrefsFound0% OR scankeywordrefsResult0%
1890     scankeywordrefsLine0$ = collectrefsLine0$
1900     scankeywordrefsKeyword0$ = "GOSUB"
1910     GOSUB 2170
1920     collectrefsFound0% = collectrefsFound0% OR scankeywordrefsResult0%
1930     scankeywordrefsLine0$ = collectrefsLine0$
1940     scankeywordrefsKeyword0$ = "THEN"
1950     GOSUB 2170
1960     collectrefsFound0% = collectrefsFound0% OR scankeywordrefsResult0%
1970     scankeywordrefsLine0$ = collectrefsLine0$
1980     scankeywordrefsKeyword0$ = "ELSE"
1990     GOSUB 2170
2000     collectrefsFound0% = collectrefsFound0% OR scankeywordrefsResult0%
2010     scankeywordrefsLine0$ = collectrefsLine0$
2020     scankeywordrefsKeyword0$ = "RESTORE"
2030     GOSUB 2170
2040     collectrefsFound0% = collectrefsFound0% OR scankeywordrefsResult0%
2050     scankeywordrefsLine0$ = collectrefsLine0$
2060     scankeywordrefsKeyword0$ = "RESUME"
2070     GOSUB 2170
2080     collectrefsFound0% = collectrefsFound0% OR scankeywordrefsResult0%
2090     scankeywordrefsLine0$ = collectrefsLine0$
2100     scankeywordrefsKeyword0$ = "RUN"
2110     GOSUB 2170
2120     collectrefsFound0% = collectrefsFound0% OR scankeywordrefsResult0%
2130     collectrefsResult0% = collectrefsFound0%
2140     RETURN
2150 ' end function collectrefs%

2160 ' function scankeywordrefs%(line$, keyword$)
2170     upperText0$ = scankeywordrefsLine0$
2180     GOSUB 750
2190     scankeywordrefsUl0$ = upperResult0$
2200     upperText0$ = scankeywordrefsKeyword0$
2210     GOSUB 750
2220     scankeywordrefsUk0$ = upperResult0$
2230     POS% = INSTR(scankeywordrefsUl0$, scankeywordrefsUk0$)
2240     IF (POS% = 0) = 0 THEN GOTO 2270
2250         scankeywordrefsResult0% = 0
2260         RETURN
2270     REM END IF
2280     trimleftText0$ = MID$(scankeywordrefsLine0$, POS% + LEN(scankeywordrefsKeyword0$))
2290     GOSUB 600
2300     scankeywordrefsAfter0$ = trimleftResult0$
2310     parselinenumberText0$ = scankeywordrefsAfter0$
2320     GOSUB 990
2330     scankeywordrefsRef0% = parselinenumberResult0%
2340     IF (scankeywordrefsRef0% > 0) = 0 THEN GOTO 2390
2350         addrefLineNo0% = scankeywordrefsRef0%
2360         GOSUB 1480
2370         scankeywordrefsResult0% = 1
2380         RETURN
2390     REM END IF
2400     scankeywordrefsResult0% = 0
2410     RETURN
2420 ' end function scankeywordrefs%

2430 ' function loadlines%()
2440     refcount% = 0
2450     linecount% = 0
2460     OPEN inputfile$ FOR INPUT AS #1
2470     IF (EOF(1) = 0) = 0 THEN GOTO 2510
2480         linecount% = linecount% + 1
2490         LINE INPUT #1, rawline$(linecount%)
2500         GOTO 2470
2510     REM END WHILE
2520     CLOSE #1
2530     loadlinesI0% = 1
2540     IF (loadlinesI0% <= linecount%) = 0 THEN GOTO 2640
2550         parselinenumberText0$ = rawline$(loadlinesI0%)
2560         GOSUB 990
2570         linenumber%(loadlinesI0%) = parselinenumberResult0%
2580         striplinenumberText0$ = rawline$(loadlinesI0%)
2590         GOSUB 1230
2600         linetext$(loadlinesI0%) = striplinenumberResult0$
2610         keepline%(loadlinesI0%) = 0
2620         loadlinesI0% = loadlinesI0% + 1
2630         GOTO 2540
2640     REM END WHILE
2650     loadlinesResult0% = 0
2660     RETURN
2670 ' end function loadlines%

2680 ' function collectallrefs%()
2690     refcount% = 0
2700     collectallrefsI0% = 1
2710     IF (collectallrefsI0% <= linecount%) = 0 THEN GOTO 2770
2720         collectrefsLine0$ = linetext$(collectallrefsI0%)
2730         GOSUB 1840
2740         keepline%(collectallrefsI0%) = collectrefsResult0%
2750         collectallrefsI0% = collectallrefsI0% + 1
2760         GOTO 2710
2770     REM END WHILE
2780     collectallrefsResult0% = 0
2790     RETURN
2800 ' end function collectallrefs%

2810 ' function transformlines%()
2820     OPEN outputfile$ FOR OUTPUT AS #2
2830     transformlinesI0% = 1
2840     IF (transformlinesI0% <= linecount%) = 0 THEN GOTO 3000
2850         IF (linenumber%(transformlinesI0%) > 0) = 0 THEN GOTO 2960
2860             isreferencedLineNo0% = linenumber%(transformlinesI0%)
2870             GOSUB 1710
2880             IF ((keepline%(transformlinesI0%) <> 0) OR (isreferencedResult0% <> 0)) = 0 THEN GOTO 2930
2890                 trimleftText0$ = STR$(linenumber%(transformlinesI0%))
2900                 GOSUB 600
2910                 PRINT #2, (trimleftResult0$ + " ") + linetext$(transformlinesI0%)
2920                 GOTO 2940
2930                 PRINT #2, linetext$(transformlinesI0%)
2940             REM END IF
2950             GOTO 2970
2960             PRINT #2, linetext$(transformlinesI0%)
2970         REM END IF
2980         transformlinesI0% = transformlinesI0% + 1
2990         GOTO 2840
3000     REM END WHILE
3010     CLOSE #2
3020     transformlinesResult0% = 0
3030     RETURN
3040 ' end function transformlines%
