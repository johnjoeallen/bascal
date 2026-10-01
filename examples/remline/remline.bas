10 ' BASCAL generated BASIC -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
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
360 inputfile$ = "examples/remline/sample/input.bas"
370 outputfile$ = "examples/remline/sample/output.bas"
380 PRINT "BASCAL REMLINE example"
390 PRINT "Input: " + inputfile$
400 PRINT "Output: " + outputfile$
410 GOSUB 2710
420 GOSUB 2960
430 GOSUB 3090
440 PRINT "Done"
450 END

460 ' function ucase$(self$)
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
620     IF (trimleftI0% <= LEN(trimleftText0$)) = 0 THEN GOTO 730
630         trimleftCh0$ = MID$(trimleftText0$, trimleftI0%, 1)
640         IF (trimleftCh0$ <> " ") = 0 THEN GOTO 700
650             BCCT5$ = trimleftText0$
660             BCCT6% = trimleftI0%
670             BCCT7$ = MID$(BCCT5$, BCCT6%)
680             trimleftResult0$ = BCCT7$
690             RETURN
700         REM END IF
710         trimleftI0% = trimleftI0% + 1
720     GOTO 620
730     REM END WHILE
740     trimleftResult0$ = ""
750     RETURN
760 ' end function trimleft$

770 ' function upper$(text$)
780     ucaseSelf0$ = upperText0$
790     GOSUB 470
800     BCCT9$ = ucaseResult0$
810     upperResult0$ = BCCT9$
820     RETURN
830 ' end function upper$

840 ' function startswithkeyword%(text$, keyword$)
850     trimleftText0$ = startswithkeywordText0$
860     GOSUB 600
870     startswithkeywordT0$ = trimleftResult0$
880     startswithkeywordKw0$ = startswithkeywordKeyword0$
890     upperText0$ = startswithkeywordT0$
900     GOSUB 780
910     startswithkeywordT0$ = upperResult0$
920     upperText0$ = startswithkeywordKw0$
930     GOSUB 780
940     startswithkeywordKw0$ = upperResult0$
950     IF (LEN(startswithkeywordT0$) < LEN(startswithkeywordKw0$)) = 0 THEN GOTO 980
960         startswithkeywordResult0% = 0
970         RETURN
980     REM END IF
990     BCCT11$ = startswithkeywordT0$
1000     BCCT12$ = startswithkeywordKw0$
1010     BCCT13% = LEN(BCCT12$)
1020     BCCT14% = BCCT13%
1030     BCCT15$ = LEFT$(BCCT11$, BCCT14%)
1040     startswithkeywordResult0% = BCCT15$ = startswithkeywordKw0$
1050     RETURN
1060 ' end function startswithkeyword%

1070 ' function parselinenumber%(text$)
1080     trimleftText0$ = parselinenumberText0$
1090     GOSUB 600
1100     parselinenumberText0$ = trimleftResult0$
1110     parselinenumberDigits0$ = ""
1120     parselinenumberI0% = 1
1130     parselinenumberDone0% = 0
1140     IF ((parselinenumberI0% <= LEN(parselinenumberText0$)) AND (parselinenumberDone0% = 0)) = 0 THEN GOTO 1230
1150         parselinenumberCh0$ = MID$(parselinenumberText0$, parselinenumberI0%, 1)
1160         IF ((parselinenumberCh0$ >= "0") AND (parselinenumberCh0$ <= "9")) = 0 THEN GOTO 1190
1170             parselinenumberDigits0$ = parselinenumberDigits0$ + parselinenumberCh0$
1180         GOTO 1200
1190             parselinenumberDone0% = 1
1200         REM END IF
1210         parselinenumberI0% = parselinenumberI0% + 1
1220     GOTO 1140
1230     REM END WHILE
1240     IF (LEN(parselinenumberDigits0$) = 0) = 0 THEN GOTO 1270
1250         parselinenumberResult0% = 0
1260         RETURN
1270     REM END IF
1280     BCCT19$ = parselinenumberDigits0$
1290     BCCT20! = VAL(BCCT19$)
1300     parselinenumberResult0% = BCCT20!
1310     RETURN
1320 ' end function parselinenumber%

1330 ' function striplinenumber$(text$)
1340     trimleftText0$ = striplinenumberText0$
1350     GOSUB 600
1360     striplinenumberText0$ = trimleftResult0$
1370     striplinenumberI0% = 1
1380     striplinenumberDone0% = 0
1390     IF ((striplinenumberI0% <= LEN(striplinenumberText0$)) AND (striplinenumberDone0% = 0)) = 0 THEN GOTO 1470
1400         striplinenumberCh0$ = MID$(striplinenumberText0$, striplinenumberI0%, 1)
1410         IF ((striplinenumberCh0$ >= "0") AND (striplinenumberCh0$ <= "9")) = 0 THEN GOTO 1440
1420             striplinenumberI0% = striplinenumberI0% + 1
1430         GOTO 1450
1440             striplinenumberDone0% = 1
1450         REM END IF
1460     GOTO 1390
1470     REM END WHILE
1480     IF (striplinenumberI0% > LEN(striplinenumberText0$)) = 0 THEN GOTO 1510
1490         striplinenumberResult0$ = ""
1500         RETURN
1510     REM END IF
1520     IF (MID$(striplinenumberText0$, striplinenumberI0%, 1) = " ") = 0 THEN GOTO 1540
1530         striplinenumberI0% = striplinenumberI0% + 1
1540     REM END IF
1550     BCCT25$ = striplinenumberText0$
1560     BCCT26% = striplinenumberI0%
1570     BCCT27$ = MID$(BCCT25$, BCCT26%)
1580     striplinenumberResult0$ = BCCT27$
1590     RETURN
1600 ' end function striplinenumber$

1610 ' function addref%(lineNo%)
1620     IF (addrefLineNo0% = 0) = 0 THEN GOTO 1650
1630         addrefResult0% = 0
1640         RETURN
1650     REM END IF
1660     addrefI0% = 1
1670     IF (addrefI0% <= refcount%) = 0 THEN GOTO 1740
1680         IF (refnumber%(addrefI0%) = addrefLineNo0%) = 0 THEN GOTO 1710
1690             addrefResult0% = 0
1700             RETURN
1710         REM END IF
1720         addrefI0% = addrefI0% + 1
1730     GOTO 1670
1740     REM END WHILE
1750     IF (refcount% >= 1000) = 0 THEN GOTO 1780
1760         addrefResult0% = 0
1770         RETURN
1780     REM END IF
1790     refcount% = refcount% + 1
1800     refnumber%(refcount%) = addrefLineNo0%
1810     addrefResult0% = 1
1820     RETURN
1830 ' end function addref%

1840 ' function isreferenced%(lineNo%)
1850     isreferencedI0% = 1
1860     IF (isreferencedI0% <= refcount%) = 0 THEN GOTO 1930
1870         IF (refnumber%(isreferencedI0%) = isreferencedLineNo0%) = 0 THEN GOTO 1900
1880             isreferencedResult0% = 1
1890             RETURN
1900         REM END IF
1910         isreferencedI0% = isreferencedI0% + 1
1920     GOTO 1860
1930     REM END WHILE
1940     isreferencedResult0% = 0
1950     RETURN
1960 ' end function isreferenced%

1970 ' function collectrefs%(line$)
1980     collectrefsFound0% = 0
1990     scankeywordrefsLine0$ = collectrefsLine0$
2000     scankeywordrefsKeyword0$ = "GOTO"
2010     GOSUB 2380
2020     BCCT34% = scankeywordrefsResult0%
2030     collectrefsFound0% = collectrefsFound0% OR BCCT34%
2040     scankeywordrefsLine0$ = collectrefsLine0$
2050     scankeywordrefsKeyword0$ = "GOSUB"
2060     GOSUB 2380
2070     BCCT35% = scankeywordrefsResult0%
2080     collectrefsFound0% = collectrefsFound0% OR BCCT35%
2090     scankeywordrefsLine0$ = collectrefsLine0$
2100     scankeywordrefsKeyword0$ = "THEN"
2110     GOSUB 2380
2120     BCCT36% = scankeywordrefsResult0%
2130     collectrefsFound0% = collectrefsFound0% OR BCCT36%
2140     scankeywordrefsLine0$ = collectrefsLine0$
2150     scankeywordrefsKeyword0$ = "ELSE"
2160     GOSUB 2380
2170     BCCT37% = scankeywordrefsResult0%
2180     collectrefsFound0% = collectrefsFound0% OR BCCT37%
2190     scankeywordrefsLine0$ = collectrefsLine0$
2200     scankeywordrefsKeyword0$ = "RESTORE"
2210     GOSUB 2380
2220     BCCT38% = scankeywordrefsResult0%
2230     collectrefsFound0% = collectrefsFound0% OR BCCT38%
2240     scankeywordrefsLine0$ = collectrefsLine0$
2250     scankeywordrefsKeyword0$ = "RESUME"
2260     GOSUB 2380
2270     BCCT39% = scankeywordrefsResult0%
2280     collectrefsFound0% = collectrefsFound0% OR BCCT39%
2290     scankeywordrefsLine0$ = collectrefsLine0$
2300     scankeywordrefsKeyword0$ = "RUN"
2310     GOSUB 2380
2320     BCCT40% = scankeywordrefsResult0%
2330     collectrefsFound0% = collectrefsFound0% OR BCCT40%
2340     collectrefsResult0% = collectrefsFound0%
2350     RETURN
2360 ' end function collectrefs%

2370 ' function scankeywordrefs%(line$, keyword$)
2380     upperText0$ = scankeywordrefsLine0$
2390     GOSUB 780
2400     scankeywordrefsUl0$ = upperResult0$
2410     upperText0$ = scankeywordrefsKeyword0$
2420     GOSUB 780
2430     scankeywordrefsUk0$ = upperResult0$
2440     POS% = INSTR(scankeywordrefsUl0$, scankeywordrefsUk0$)
2450     IF (POS% = 0) = 0 THEN GOTO 2480
2460         scankeywordrefsResult0% = 0
2470         RETURN
2480     REM END IF
2490     BCCT42$ = scankeywordrefsLine0$
2500     BCCT43$ = scankeywordrefsKeyword0$
2510     BCCT44% = LEN(BCCT43$)
2520     BCCT45% = POS% + BCCT44%
2530     BCCT46$ = MID$(BCCT42$, BCCT45%)
2540     BCCT47$ = BCCT46$
2550     trimleftText0$ = BCCT47$
2560     GOSUB 600
2570     scankeywordrefsAfter0$ = trimleftResult0$
2580     parselinenumberText0$ = scankeywordrefsAfter0$
2590     GOSUB 1080
2600     scankeywordrefsRef0% = parselinenumberResult0%
2610     IF (scankeywordrefsRef0% > 0) = 0 THEN GOTO 2660
2620         addrefLineNo0% = scankeywordrefsRef0%
2630         GOSUB 1620
2640         scankeywordrefsResult0% = 1
2650         RETURN
2660     REM END IF
2670     scankeywordrefsResult0% = 0
2680     RETURN
2690 ' end function scankeywordrefs%

2700 ' function loadlines%()
2710     refcount% = 0
2720     linecount% = 0
2730     OPEN inputfile$ FOR INPUT AS #1
2740     IF (EOF(1) = 0) = 0 THEN GOTO 2780
2750         linecount% = linecount% + 1
2760         LINE INPUT #1, rawline$(linecount%)
2770     GOTO 2740
2780     REM END WHILE
2790     CLOSE #1
2800     loadlinesI0% = 1
2810     IF (loadlinesI0% <= linecount%) = 0 THEN GOTO 2910
2820         parselinenumberText0$ = rawline$(loadlinesI0%)
2830         GOSUB 1080
2840         linenumber%(loadlinesI0%) = parselinenumberResult0%
2850         striplinenumberText0$ = rawline$(loadlinesI0%)
2860         GOSUB 1340
2870         linetext$(loadlinesI0%) = striplinenumberResult0$
2880         keepline%(loadlinesI0%) = 0
2890         loadlinesI0% = loadlinesI0% + 1
2900     GOTO 2810
2910     REM END WHILE
2920     loadlinesResult0% = 0
2930     RETURN
2940 ' end function loadlines%

2950 ' function collectallrefs%()
2960     refcount% = 0
2970     collectallrefsI0% = 1
2980     IF (collectallrefsI0% <= linecount%) = 0 THEN GOTO 3040
2990         collectrefsLine0$ = linetext$(collectallrefsI0%)
3000         GOSUB 1980
3010         keepline%(collectallrefsI0%) = collectrefsResult0%
3020         collectallrefsI0% = collectallrefsI0% + 1
3030     GOTO 2980
3040     REM END WHILE
3050     collectallrefsResult0% = 0
3060     RETURN
3070 ' end function collectallrefs%

3080 ' function transformlines%()
3090     OPEN outputfile$ FOR OUTPUT AS #2
3100     transformlinesI0% = 1
3110     IF (transformlinesI0% <= linecount%) = 0 THEN GOTO 3320
3120         IF (linenumber%(transformlinesI0%) > 0) = 0 THEN GOTO 3280
3130             isreferencedLineNo0% = linenumber%(transformlinesI0%)
3140             GOSUB 1850
3150             BCCT53% = isreferencedResult0%
3160             IF ((keepline%(transformlinesI0%) <> 0) OR (BCCT53% <> 0)) = 0 THEN GOTO 3250
3170                 BCCT54% = linenumber%(transformlinesI0%)
3180                 BCCT55$ = STR$(BCCT54%)
3190                 BCCT56$ = BCCT55$
3200                 trimleftText0$ = BCCT56$
3210                 GOSUB 600
3220                 BCCT57$ = trimleftResult0$
3230                 PRINT #2, (BCCT57$ + " ") + linetext$(transformlinesI0%)
3240             GOTO 3260
3250                 PRINT #2, linetext$(transformlinesI0%)
3260             REM END IF
3270         GOTO 3290
3280             PRINT #2, linetext$(transformlinesI0%)
3290         REM END IF
3300         transformlinesI0% = transformlinesI0% + 1
3310     GOTO 3110
3320     REM END WHILE
3330     CLOSE #2
3340     transformlinesResult0% = 0
3350     RETURN
3360 ' end function transformlines%
