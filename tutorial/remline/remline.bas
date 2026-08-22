10 ' BASCAL generated BASIC
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Upper-cases s$. Not a real MBASIC/BASCOM 2.00 builtin -- verified against
40 ' a real IBM BASIC Compiler 2.00 under dosbox-x -- so BASCAL ships its own.

50 ' Shared string helpers for REMLINE.

60 ' UCASE$ isn't a real MBASIC/BASCOM 2.00 builtin (verified against a real
70 ' IBM BASIC Compiler 2.00 under dosbox-x), so upper$() below needs BASCAL's
80 ' own com.bascal.stdlib implementation rather than assuming the target
90 ' dialect provides one.

100 ' text$ -- string to trim

110 ' text$ -- string to uppercase

120 ' text$    -- string to test
130 ' keyword$ -- keyword to look for at the start of text$

140 ' Parse and strip leading decimal line numbers.

150 ' text$ -- source line to read a leading line number from

160 ' text$ -- source line to strip a leading line number from

170 ' Fixed-size reference tracking for the example.

180 ' lineNo% -- line number to record as referenced

190 ' lineNo% -- line number to test

200 ' line$ -- source line to scan for GOTO/GOSUB/THEN/... targets

210 ' line$    -- source line to scan
220 ' keyword$ -- keyword to look for (e.g. "GOTO")

230 ' REMLINE works on an input BASIC listing and writes a cleaned version.

240 DIM rawline$(1000)
250 DIM linetext$(1000)
260 DIM linenumber%(1000)
270 DIM keepline%(1000)
280 DIM refnumber%(1000)

290 ' REMLINE demo driver.
300 ' This version reads a line-numbered BASIC file and writes a cleaned copy.
310 ' The dependency graph is still real: the driver pulls in parsing, reference
320 ' collection, and string helpers through BASCAL's path-style require syntax.

330 inputfile$ = "tutorial/remline/sample/input.bas"
340 outputfile$ = "tutorial/remline/sample/output.bas"

350 PRINT "BASCAL REMLINE example"
360 PRINT "Input: " + inputfile$
370 PRINT "Output: " + outputfile$

380 GOSUB 2410
390 GOSUB 2660
400 GOSUB 2790

410 PRINT "Done"
420 END

430 ' function ucase$(s$)
440     ucaseOut0$ = ""
450     FOR ucaseI0% = 1 TO LEN(ucaseS0$)
460         ucaseC0% = ASC(MID$(ucaseS0$, ucaseI0%, 1))
470         IF (ucaseC0% >= 97) = 0 THEN GOTO 500
480         IF (ucaseC0% <= 122) = 0 THEN GOTO 500
490             ucaseC0% = ucaseC0% - 32
500         REM END IF
510         ucaseOut0$ = ucaseOut0$ + CHR$(ucaseC0%)
520     NEXT ucaseI0%
530     ucaseResult0$ = ucaseOut0$
540     RETURN
550 ' end function ucase$

560 ' function trimleft$(text$)
570     ' Walk from the left until the first non-space character appears.
580     trimleftI0% = 1
590     IF (trimleftI0% <= LEN(trimleftText0$)) = 0 THEN GOTO 670
600         trimleftCh0$ = MID$(trimleftText0$, trimleftI0%, 1)
610         IF (trimleftCh0$ <> " ") = 0 THEN GOTO 640
620             trimleftResult0$ = MID$(trimleftText0$, trimleftI0%)
630             RETURN
640         REM END IF
650         trimleftI0% = trimleftI0% + 1
660         GOTO 590
670     REM END WHILE
680     trimleftResult0$ = ""
690     RETURN
700 ' end function trimleft$

710 ' function upper$(text$)
720     ucaseS0$ = upperText0$
730     GOSUB 440
740     upperResult0$ = ucaseResult0$
750     RETURN
760 ' end function upper$

770 ' function startswithkeyword%(text$, keyword$)
780     trimleftText0$ = startswithkeywordText0$
790     GOSUB 570
800     startswithkeywordT0$ = trimleftResult0$
810     startswithkeywordKw0$ = startswithkeywordKeyword0$
820     upperText0$ = startswithkeywordT0$
830     GOSUB 720
840     startswithkeywordT0$ = upperResult0$
850     upperText0$ = startswithkeywordKw0$
860     GOSUB 720
870     startswithkeywordKw0$ = upperResult0$
880     IF (LEN(startswithkeywordT0$) < LEN(startswithkeywordKw0$)) = 0 THEN GOTO 910
890         startswithkeywordResult0% = 0
900         RETURN
910     REM END IF
920     startswithkeywordResult0% = LEFT$(startswithkeywordT0$, LEN(startswithkeywordKw0$)) = startswithkeywordKw0$
930     RETURN
940 ' end function startswithkeyword%

950 ' function parselinenumber%(text$)
960     trimleftText0$ = parselinenumberText0$
970     GOSUB 570
980     parselinenumberText0$ = trimleftResult0$
990     parselinenumberDigits0$ = ""
1000     parselinenumberI0% = 1
1010     parselinenumberDone0% = 0
1020     IF ((parselinenumberI0% <= LEN(parselinenumberText0$)) AND (parselinenumberDone0% = 0)) = 0 THEN GOTO 1110
1030         parselinenumberCh0$ = MID$(parselinenumberText0$, parselinenumberI0%, 1)
1040         IF ((parselinenumberCh0$ >= "0") AND (parselinenumberCh0$ <= "9")) = 0 THEN GOTO 1070
1050             parselinenumberDigits0$ = parselinenumberDigits0$ + parselinenumberCh0$
1060             GOTO 1080
1070             parselinenumberDone0% = 1
1080         REM END IF
1090         parselinenumberI0% = parselinenumberI0% + 1
1100         GOTO 1020
1110     REM END WHILE
1120     IF (LEN(parselinenumberDigits0$) = 0) = 0 THEN GOTO 1150
1130         parselinenumberResult0% = 0
1140         RETURN
1150     REM END IF
1160     parselinenumberResult0% = VAL(parselinenumberDigits0$)
1170     RETURN
1180 ' end function parselinenumber%

1190 ' function striplinenumber$(text$)
1200     trimleftText0$ = striplinenumberText0$
1210     GOSUB 570
1220     striplinenumberText0$ = trimleftResult0$
1230     striplinenumberI0% = 1
1240     striplinenumberDone0% = 0
1250     IF ((striplinenumberI0% <= LEN(striplinenumberText0$)) AND (striplinenumberDone0% = 0)) = 0 THEN GOTO 1330
1260         striplinenumberCh0$ = MID$(striplinenumberText0$, striplinenumberI0%, 1)
1270         IF ((striplinenumberCh0$ >= "0") AND (striplinenumberCh0$ <= "9")) = 0 THEN GOTO 1300
1280             striplinenumberI0% = striplinenumberI0% + 1
1290             GOTO 1310
1300             striplinenumberDone0% = 1
1310         REM END IF
1320         GOTO 1250
1330     REM END WHILE
1340     IF (striplinenumberI0% > LEN(striplinenumberText0$)) = 0 THEN GOTO 1370
1350         striplinenumberResult0$ = ""
1360         RETURN
1370     REM END IF
1380     IF (MID$(striplinenumberText0$, striplinenumberI0%, 1) = " ") = 0 THEN GOTO 1400
1390         striplinenumberI0% = striplinenumberI0% + 1
1400     REM END IF
1410     striplinenumberResult0$ = MID$(striplinenumberText0$, striplinenumberI0%)
1420     RETURN
1430 ' end function striplinenumber$

1440 ' function addref%(lineno%)
1450     IF (addrefLineNo0% = 0) = 0 THEN GOTO 1480
1460         addrefResult0% = 0
1470         RETURN
1480     REM END IF
1490     addrefI0% = 1
1500     IF (addrefI0% <= refcount%) = 0 THEN GOTO 1570
1510         IF (refnumber%(addrefI0%) = addrefLineNo0%) = 0 THEN GOTO 1540
1520             addrefResult0% = 0
1530             RETURN
1540         REM END IF
1550         addrefI0% = addrefI0% + 1
1560         GOTO 1500
1570     REM END WHILE
1580     IF (refcount% >= 1000) = 0 THEN GOTO 1610
1590         addrefResult0% = 0
1600         RETURN
1610     REM END IF
1620     refcount% = refcount% + 1
1630     refnumber%(refcount%) = addrefLineNo0%
1640     addrefResult0% = 1
1650     RETURN
1660 ' end function addref%

1670 ' function isreferenced%(lineno%)
1680     isreferencedI0% = 1
1690     IF (isreferencedI0% <= refcount%) = 0 THEN GOTO 1760
1700         IF (refnumber%(isreferencedI0%) = isreferencedLineNo0%) = 0 THEN GOTO 1730
1710             isreferencedResult0% = 1
1720             RETURN
1730         REM END IF
1740         isreferencedI0% = isreferencedI0% + 1
1750         GOTO 1690
1760     REM END WHILE
1770     isreferencedResult0% = 0
1780     RETURN
1790 ' end function isreferenced%

1800 ' function collectrefs%(line$)
1810     collectrefsFound0% = 0
1820     scankeywordrefsLine0$ = collectrefsLine0$
1830     scankeywordrefsKeyword0$ = "GOTO"
1840     GOSUB 2140
1850     collectrefsFound0% = collectrefsFound0% OR scankeywordrefsResult0%
1860     scankeywordrefsLine0$ = collectrefsLine0$
1870     scankeywordrefsKeyword0$ = "GOSUB"
1880     GOSUB 2140
1890     collectrefsFound0% = collectrefsFound0% OR scankeywordrefsResult0%
1900     scankeywordrefsLine0$ = collectrefsLine0$
1910     scankeywordrefsKeyword0$ = "THEN"
1920     GOSUB 2140
1930     collectrefsFound0% = collectrefsFound0% OR scankeywordrefsResult0%
1940     scankeywordrefsLine0$ = collectrefsLine0$
1950     scankeywordrefsKeyword0$ = "ELSE"
1960     GOSUB 2140
1970     collectrefsFound0% = collectrefsFound0% OR scankeywordrefsResult0%
1980     scankeywordrefsLine0$ = collectrefsLine0$
1990     scankeywordrefsKeyword0$ = "RESTORE"
2000     GOSUB 2140
2010     collectrefsFound0% = collectrefsFound0% OR scankeywordrefsResult0%
2020     scankeywordrefsLine0$ = collectrefsLine0$
2030     scankeywordrefsKeyword0$ = "RESUME"
2040     GOSUB 2140
2050     collectrefsFound0% = collectrefsFound0% OR scankeywordrefsResult0%
2060     scankeywordrefsLine0$ = collectrefsLine0$
2070     scankeywordrefsKeyword0$ = "RUN"
2080     GOSUB 2140
2090     collectrefsFound0% = collectrefsFound0% OR scankeywordrefsResult0%
2100     collectrefsResult0% = collectrefsFound0%
2110     RETURN
2120 ' end function collectrefs%

2130 ' function scankeywordrefs%(line$, keyword$)
2140     upperText0$ = scankeywordrefsLine0$
2150     GOSUB 720
2160     scankeywordrefsUl0$ = upperResult0$
2170     upperText0$ = scankeywordrefsKeyword0$
2180     GOSUB 720
2190     scankeywordrefsUk0$ = upperResult0$
2200     POS% = INSTR(scankeywordrefsUl0$, scankeywordrefsUk0$)
2210     IF (POS% = 0) = 0 THEN GOTO 2240
2220         scankeywordrefsResult0% = 0
2230         RETURN
2240     REM END IF
2250     trimleftText0$ = MID$(scankeywordrefsLine0$, POS% + LEN(scankeywordrefsKeyword0$))
2260     GOSUB 570
2270     scankeywordrefsAfter0$ = trimleftResult0$
2280     parselinenumberText0$ = scankeywordrefsAfter0$
2290     GOSUB 960
2300     scankeywordrefsRef0% = parselinenumberResult0%
2310     IF (scankeywordrefsRef0% > 0) = 0 THEN GOTO 2360
2320         addrefLineNo0% = scankeywordrefsRef0%
2330         GOSUB 1450
2340         scankeywordrefsResult0% = 1
2350         RETURN
2360     REM END IF
2370     scankeywordrefsResult0% = 0
2380     RETURN
2390 ' end function scankeywordrefs%

2400 ' function loadlines%()
2410     refcount% = 0
2420     linecount% = 0
2430     OPEN inputfile$ FOR INPUT AS #1
2440     IF (EOF(1) = 0) = 0 THEN GOTO 2480
2450         linecount% = linecount% + 1
2460         LINE INPUT #1, rawline$(linecount%)
2470         GOTO 2440
2480     REM END WHILE
2490     CLOSE #1
2500     loadlinesI0% = 1
2510     IF (loadlinesI0% <= linecount%) = 0 THEN GOTO 2610
2520         parselinenumberText0$ = rawline$(loadlinesI0%)
2530         GOSUB 960
2540         linenumber%(loadlinesI0%) = parselinenumberResult0%
2550         striplinenumberText0$ = rawline$(loadlinesI0%)
2560         GOSUB 1200
2570         linetext$(loadlinesI0%) = striplinenumberResult0$
2580         keepline%(loadlinesI0%) = 0
2590         loadlinesI0% = loadlinesI0% + 1
2600         GOTO 2510
2610     REM END WHILE
2620     loadlinesResult0% = 0
2630     RETURN
2640 ' end function loadlines%

2650 ' function collectallrefs%()
2660     refcount% = 0
2670     collectallrefsI0% = 1
2680     IF (collectallrefsI0% <= linecount%) = 0 THEN GOTO 2740
2690         collectrefsLine0$ = linetext$(collectallrefsI0%)
2700         GOSUB 1810
2710         keepline%(collectallrefsI0%) = collectrefsResult0%
2720         collectallrefsI0% = collectallrefsI0% + 1
2730         GOTO 2680
2740     REM END WHILE
2750     collectallrefsResult0% = 0
2760     RETURN
2770 ' end function collectallrefs%

2780 ' function transformlines%()
2790     OPEN outputfile$ FOR OUTPUT AS #2
2800     transformlinesI0% = 1
2810     IF (transformlinesI0% <= linecount%) = 0 THEN GOTO 2970
2820         IF (linenumber%(transformlinesI0%) > 0) = 0 THEN GOTO 2930
2830             isreferencedLineNo0% = linenumber%(transformlinesI0%)
2840             GOSUB 1680
2850             IF ((keepline%(transformlinesI0%) <> 0) OR (isreferencedResult0% <> 0)) = 0 THEN GOTO 2900
2860                 trimleftText0$ = STR$(linenumber%(transformlinesI0%))
2870                 GOSUB 570
2880                 PRINT #2, (trimleftResult0$ + " ") + linetext$(transformlinesI0%)
2890                 GOTO 2910
2900                 PRINT #2, linetext$(transformlinesI0%)
2910             REM END IF
2920             GOTO 2940
2930             PRINT #2, linetext$(transformlinesI0%)
2940         REM END IF
2950         transformlinesI0% = transformlinesI0% + 1
2960         GOTO 2810
2970     REM END WHILE
2980     CLOSE #2
2990     transformlinesResult0% = 0
3000     RETURN
3010 ' end function transformlines%
