10 ' BASCAL generated BASIC
20 ' Functions are lowered to global variables, labels, and GOSUB

30 ' Shared string helpers for REMLINE.

40 ' Parse and strip leading decimal line numbers.

50 ' Fixed-size reference tracking for the example.

60 ' REMLINE works on an input BASIC listing and writes a cleaned version.

70 DIM rawline$(1000)
80 DIM linetext$(1000)
90 DIM linenumber%(1000)
100 DIM keepline%(1000)
110 DIM refnumber%(1000)

120 ' REMLINE demo driver.
130 ' This version reads a line-numbered BASIC file and writes a cleaned copy.
140 ' The dependency graph is still real: the driver pulls in parsing, reference
150 ' collection, and string helpers through BASCAL's path-style require syntax.

160 inputfile$ = "tutorial/remline/sample/input.bas"
170 outputfile$ = "tutorial/remline/sample/output.bas"

180 PRINT "BASCAL REMLINE example"
190 PRINT "Input: " + inputfile$
200 PRINT "Output: " + outputfile$

210 GOSUB 2090
220 GOSUB 2340
230 GOSUB 2470

240 PRINT "Done"
250 END

260 ' function trimleft$(text$)
270     ' Walk from the left until the first non-space character appears.
280     trimleft_i_0% = 1
290     IF (trimleft_i_0% <= LEN(trimleft_text_0$)) = 0 THEN GOTO 370
300         trimleft_ch_0$ = MID$(trimleft_text_0$, trimleft_i_0%, 1)
310         IF (trimleft_ch_0$ <> " ") = 0 THEN GOTO 340
320             trimleft_result_0$ = MID$(trimleft_text_0$, trimleft_i_0%)
330             RETURN
340         REM END IF
350         trimleft_i_0% = trimleft_i_0% + 1
360         GOTO 290
370     REM END WHILE
380     trimleft_result_0$ = ""
390     RETURN
400 ' end function trimleft$

410 ' function upper$(text$)
420     upper_result_0$ = UCASE$(upper_text_0$)
430     RETURN
440 ' end function upper$

450 ' function startswithkeyword%(text$, keyword$)
460     trimleft_text_0$ = startswithkeyword_text_0$
470     GOSUB 270
480     startswithkeyword_t_0$ = trimleft_result_0$
490     startswithkeyword_kw_0$ = startswithkeyword_keyword_0$
500     upper_text_0$ = startswithkeyword_t_0$
510     GOSUB 420
520     startswithkeyword_t_0$ = upper_result_0$
530     upper_text_0$ = startswithkeyword_kw_0$
540     GOSUB 420
550     startswithkeyword_kw_0$ = upper_result_0$
560     IF (LEN(startswithkeyword_t_0$) < LEN(startswithkeyword_kw_0$)) = 0 THEN GOTO 590
570         startswithkeyword_result_0% = 0
580         RETURN
590     REM END IF
600     startswithkeyword_result_0% = LEFT$(startswithkeyword_t_0$, LEN(startswithkeyword_kw_0$)) = startswithkeyword_kw_0$
610     RETURN
620 ' end function startswithkeyword%

630 ' function parselinenumber%(text$)
640     trimleft_text_0$ = parselinenumber_text_0$
650     GOSUB 270
660     parselinenumber_text_0$ = trimleft_result_0$
670     parselinenumber_digits_0$ = ""
680     parselinenumber_i_0% = 1
690     parselinenumber_done_0% = 0
700     IF ((parselinenumber_i_0% <= LEN(parselinenumber_text_0$)) AND (parselinenumber_done_0% = 0)) = 0 THEN GOTO 790
710         parselinenumber_ch_0$ = MID$(parselinenumber_text_0$, parselinenumber_i_0%, 1)
720         IF ((parselinenumber_ch_0$ >= "0") AND (parselinenumber_ch_0$ <= "9")) = 0 THEN GOTO 750
730             parselinenumber_digits_0$ = parselinenumber_digits_0$ + parselinenumber_ch_0$
740             GOTO 760
750             parselinenumber_done_0% = 1
760         REM END IF
770         parselinenumber_i_0% = parselinenumber_i_0% + 1
780         GOTO 700
790     REM END WHILE
800     IF (LEN(parselinenumber_digits_0$) = 0) = 0 THEN GOTO 830
810         parselinenumber_result_0% = 0
820         RETURN
830     REM END IF
840     parselinenumber_result_0% = VAL(parselinenumber_digits_0$)
850     RETURN
860 ' end function parselinenumber%

870 ' function striplinenumber$(text$)
880     trimleft_text_0$ = striplinenumber_text_0$
890     GOSUB 270
900     striplinenumber_text_0$ = trimleft_result_0$
910     striplinenumber_i_0% = 1
920     striplinenumber_done_0% = 0
930     IF ((striplinenumber_i_0% <= LEN(striplinenumber_text_0$)) AND (striplinenumber_done_0% = 0)) = 0 THEN GOTO 1010
940         striplinenumber_ch_0$ = MID$(striplinenumber_text_0$, striplinenumber_i_0%, 1)
950         IF ((striplinenumber_ch_0$ >= "0") AND (striplinenumber_ch_0$ <= "9")) = 0 THEN GOTO 980
960             striplinenumber_i_0% = striplinenumber_i_0% + 1
970             GOTO 990
980             striplinenumber_done_0% = 1
990         REM END IF
1000         GOTO 930
1010     REM END WHILE
1020     IF (striplinenumber_i_0% > LEN(striplinenumber_text_0$)) = 0 THEN GOTO 1050
1030         striplinenumber_result_0$ = ""
1040         RETURN
1050     REM END IF
1060     IF (MID$(striplinenumber_text_0$, striplinenumber_i_0%, 1) = " ") = 0 THEN GOTO 1080
1070         striplinenumber_i_0% = striplinenumber_i_0% + 1
1080     REM END IF
1090     striplinenumber_result_0$ = MID$(striplinenumber_text_0$, striplinenumber_i_0%)
1100     RETURN
1110 ' end function striplinenumber$

1120 ' function addref%(lineno%)
1130     IF (addref_lineno_0% = 0) = 0 THEN GOTO 1160
1140         addref_result_0% = 0
1150         RETURN
1160     REM END IF
1170     addref_i_0% = 1
1180     IF (addref_i_0% <= refcount%) = 0 THEN GOTO 1250
1190         IF (refnumber%(addref_i_0%) = addref_lineno_0%) = 0 THEN GOTO 1220
1200             addref_result_0% = 0
1210             RETURN
1220         REM END IF
1230         addref_i_0% = addref_i_0% + 1
1240         GOTO 1180
1250     REM END WHILE
1260     IF (refcount% >= 1000) = 0 THEN GOTO 1290
1270         addref_result_0% = 0
1280         RETURN
1290     REM END IF
1300     refcount% = refcount% + 1
1310     refnumber%(refcount%) = addref_lineno_0%
1320     addref_result_0% = 1
1330     RETURN
1340 ' end function addref%

1350 ' function isreferenced%(lineno%)
1360     isreferenced_i_0% = 1
1370     IF (isreferenced_i_0% <= refcount%) = 0 THEN GOTO 1440
1380         IF (refnumber%(isreferenced_i_0%) = isreferenced_lineno_0%) = 0 THEN GOTO 1410
1390             isreferenced_result_0% = 1
1400             RETURN
1410         REM END IF
1420         isreferenced_i_0% = isreferenced_i_0% + 1
1430         GOTO 1370
1440     REM END WHILE
1450     isreferenced_result_0% = 0
1460     RETURN
1470 ' end function isreferenced%

1480 ' function collectrefs%(line$)
1490     collectrefs_found_0% = 0
1500     scankeywordrefs_line_0$ = collectrefs_line_0$
1510     scankeywordrefs_keyword_0$ = "GOTO"
1520     GOSUB 1820
1530     collectrefs_found_0% = collectrefs_found_0% OR scankeywordrefs_result_0%
1540     scankeywordrefs_line_0$ = collectrefs_line_0$
1550     scankeywordrefs_keyword_0$ = "GOSUB"
1560     GOSUB 1820
1570     collectrefs_found_0% = collectrefs_found_0% OR scankeywordrefs_result_0%
1580     scankeywordrefs_line_0$ = collectrefs_line_0$
1590     scankeywordrefs_keyword_0$ = "THEN"
1600     GOSUB 1820
1610     collectrefs_found_0% = collectrefs_found_0% OR scankeywordrefs_result_0%
1620     scankeywordrefs_line_0$ = collectrefs_line_0$
1630     scankeywordrefs_keyword_0$ = "ELSE"
1640     GOSUB 1820
1650     collectrefs_found_0% = collectrefs_found_0% OR scankeywordrefs_result_0%
1660     scankeywordrefs_line_0$ = collectrefs_line_0$
1670     scankeywordrefs_keyword_0$ = "RESTORE"
1680     GOSUB 1820
1690     collectrefs_found_0% = collectrefs_found_0% OR scankeywordrefs_result_0%
1700     scankeywordrefs_line_0$ = collectrefs_line_0$
1710     scankeywordrefs_keyword_0$ = "RESUME"
1720     GOSUB 1820
1730     collectrefs_found_0% = collectrefs_found_0% OR scankeywordrefs_result_0%
1740     scankeywordrefs_line_0$ = collectrefs_line_0$
1750     scankeywordrefs_keyword_0$ = "RUN"
1760     GOSUB 1820
1770     collectrefs_found_0% = collectrefs_found_0% OR scankeywordrefs_result_0%
1780     collectrefs_result_0% = collectrefs_found_0%
1790     RETURN
1800 ' end function collectrefs%

1810 ' function scankeywordrefs%(line$, keyword$)
1820     upper_text_0$ = scankeywordrefs_line_0$
1830     GOSUB 420
1840     scankeywordrefs_ul_0$ = upper_result_0$
1850     upper_text_0$ = scankeywordrefs_keyword_0$
1860     GOSUB 420
1870     scankeywordrefs_uk_0$ = upper_result_0$
1880     POS% = INSTR(scankeywordrefs_ul_0$, scankeywordrefs_uk_0$)
1890     IF (POS% = 0) = 0 THEN GOTO 1920
1900         scankeywordrefs_result_0% = 0
1910         RETURN
1920     REM END IF
1930     trimleft_text_0$ = MID$(scankeywordrefs_line_0$, POS% + LEN(scankeywordrefs_keyword_0$))
1940     GOSUB 270
1950     scankeywordrefs_after_0$ = trimleft_result_0$
1960     parselinenumber_text_0$ = scankeywordrefs_after_0$
1970     GOSUB 640
1980     scankeywordrefs_ref_0% = parselinenumber_result_0%
1990     IF (scankeywordrefs_ref_0% > 0) = 0 THEN GOTO 2040
2000         addref_lineno_0% = scankeywordrefs_ref_0%
2010         GOSUB 1130
2020         scankeywordrefs_result_0% = 1
2030         RETURN
2040     REM END IF
2050     scankeywordrefs_result_0% = 0
2060     RETURN
2070 ' end function scankeywordrefs%

2080 ' function loadlines%()
2090     refcount% = 0
2100     linecount% = 0
2110     OPEN inputfile$ FOR INPUT AS #1
2120     IF (EOF(1) = 0) = 0 THEN GOTO 2160
2130         linecount% = linecount% + 1
2140         LINE INPUT #1, rawline$(linecount%)
2150         GOTO 2120
2160     REM END WHILE
2170     CLOSE #1
2180     loadlines_i_0% = 1
2190     IF (loadlines_i_0% <= linecount%) = 0 THEN GOTO 2290
2200         parselinenumber_text_0$ = rawline$(loadlines_i_0%)
2210         GOSUB 640
2220         linenumber%(loadlines_i_0%) = parselinenumber_result_0%
2230         striplinenumber_text_0$ = rawline$(loadlines_i_0%)
2240         GOSUB 880
2250         linetext$(loadlines_i_0%) = striplinenumber_result_0$
2260         keepline%(loadlines_i_0%) = 0
2270         loadlines_i_0% = loadlines_i_0% + 1
2280         GOTO 2190
2290     REM END WHILE
2300     loadlines_result_0% = 0
2310     RETURN
2320 ' end function loadlines%

2330 ' function collectallrefs%()
2340     refcount% = 0
2350     collectallrefs_i_0% = 1
2360     IF (collectallrefs_i_0% <= linecount%) = 0 THEN GOTO 2420
2370         collectrefs_line_0$ = linetext$(collectallrefs_i_0%)
2380         GOSUB 1490
2390         keepline%(collectallrefs_i_0%) = collectrefs_result_0%
2400         collectallrefs_i_0% = collectallrefs_i_0% + 1
2410         GOTO 2360
2420     REM END WHILE
2430     collectallrefs_result_0% = 0
2440     RETURN
2450 ' end function collectallrefs%

2460 ' function transformlines%()
2470     OPEN outputfile$ FOR OUTPUT AS #2
2480     transformlines_i_0% = 1
2490     IF (transformlines_i_0% <= linecount%) = 0 THEN GOTO 2650
2500         IF (linenumber%(transformlines_i_0%) > 0) = 0 THEN GOTO 2610
2510             isreferenced_lineno_0% = linenumber%(transformlines_i_0%)
2520             GOSUB 1360
2530             IF ((keepline%(transformlines_i_0%) <> 0) OR (isreferenced_result_0% <> 0)) = 0 THEN GOTO 2580
2540                 trimleft_text_0$ = STR$(linenumber%(transformlines_i_0%))
2550                 GOSUB 270
2560                 PRINT #2, (trimleft_result_0$ + " ") + linetext$(transformlines_i_0%)
2570                 GOTO 2590
2580                 PRINT #2, linetext$(transformlines_i_0%)
2590             REM END IF
2600             GOTO 2620
2610             PRINT #2, linetext$(transformlines_i_0%)
2620         REM END IF
2630         transformlines_i_0% = transformlines_i_0% + 1
2640         GOTO 2490
2650     REM END WHILE
2660     CLOSE #2
2670     transformlines_result_0% = 0
2680     RETURN
2690 ' end function transformlines%
