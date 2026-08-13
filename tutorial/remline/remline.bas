' BASCAL generated BASIC
' Functions are transpiled to global variables, labels, and GOSUB

' Shared string helpers for REMLINE.

' text$ -- string to trim

' text$ -- string to uppercase

' text$    -- string to test
' keyword$ -- keyword to look for at the start of text$

' Parse and strip leading decimal line numbers.

' text$ -- source line to read a leading line number from

' text$ -- source line to strip a leading line number from

' Fixed-size reference tracking for the example.

' lineNo% -- line number to record as referenced

' lineNo% -- line number to test

' line$ -- source line to scan for GOTO/GOSUB/THEN/... targets

' line$    -- source line to scan
' keyword$ -- keyword to look for (e.g. "GOTO")

' REMLINE works on an input BASIC listing and writes a cleaned version.

DIM rawline$(1000)
DIM linetext$(1000)
DIM linenumber%(1000)
DIM keepline%(1000)
DIM refnumber%(1000)

' REMLINE demo driver.
' This version reads a line-numbered BASIC file and writes a cleaned copy.
' The dependency graph is still real: the driver pulls in parsing, reference
' collection, and string helpers through BASCAL's path-style require syntax.

inputfile$ = "tutorial/remline/sample/input.bas"
outputfile$ = "tutorial/remline/sample/output.bas"

PRINT "BASCAL REMLINE example"
PRINT "Input: " + inputfile$
PRINT "Output: " + outputfile$

GOSUB 350
GOSUB 400
GOSUB 430

PRINT "Done"
END

' function trimleft$(text$)
10 ' Walk from the left until the first non-space character appears.
    trimleft_i_0% = 1
20 IF (trimleft_i_0% <= LEN(trimleft_text_0$)) = 0 THEN GOTO 40
        trimleft_ch_0$ = MID$(trimleft_text_0$, trimleft_i_0%, 1)
        IF (trimleft_ch_0$ <> " ") = 0 THEN GOTO 30
            trimleft_result_0$ = MID$(trimleft_text_0$, trimleft_i_0%)
            RETURN
30 REM END IF
        trimleft_i_0% = trimleft_i_0% + 1
        GOTO 20
40 REM END WHILE
    trimleft_result_0$ = ""
    RETURN
' end function trimleft$

' function upper$(text$)
50 upper_result_0$ = UCASE$(upper_text_0$)
    RETURN
' end function upper$

' function startswithkeyword%(text$, keyword$)
60 trimleft_text_0$ = startswithkeyword_text_0$
    GOSUB 10
    startswithkeyword_t_0$ = trimleft_result_0$
    startswithkeyword_kw_0$ = startswithkeyword_keyword_0$
    upper_text_0$ = startswithkeyword_t_0$
    GOSUB 50
    startswithkeyword_t_0$ = upper_result_0$
    upper_text_0$ = startswithkeyword_kw_0$
    GOSUB 50
    startswithkeyword_kw_0$ = upper_result_0$
    IF (LEN(startswithkeyword_t_0$) < LEN(startswithkeyword_kw_0$)) = 0 THEN GOTO 70
        startswithkeyword_result_0% = 0
        RETURN
70 REM END IF
    startswithkeyword_result_0% = LEFT$(startswithkeyword_t_0$, LEN(startswithkeyword_kw_0$)) = startswithkeyword_kw_0$
    RETURN
' end function startswithkeyword%

' function parselinenumber%(text$)
80 trimleft_text_0$ = parselinenumber_text_0$
    GOSUB 10
    parselinenumber_text_0$ = trimleft_result_0$
    parselinenumber_digits_0$ = ""
    parselinenumber_i_0% = 1
    parselinenumber_done_0% = 0
90 IF ((parselinenumber_i_0% <= LEN(parselinenumber_text_0$)) AND (parselinenumber_done_0% = 0)) = 0 THEN GOTO 120
        parselinenumber_ch_0$ = MID$(parselinenumber_text_0$, parselinenumber_i_0%, 1)
        IF ((parselinenumber_ch_0$ >= "0") AND (parselinenumber_ch_0$ <= "9")) = 0 THEN GOTO 100
            parselinenumber_digits_0$ = parselinenumber_digits_0$ + parselinenumber_ch_0$
            GOTO 110
100 parselinenumber_done_0% = 1
110 REM END IF
        parselinenumber_i_0% = parselinenumber_i_0% + 1
        GOTO 90
120 REM END WHILE
    IF (LEN(parselinenumber_digits_0$) = 0) = 0 THEN GOTO 130
        parselinenumber_result_0% = 0
        RETURN
130 REM END IF
    parselinenumber_result_0% = VAL(parselinenumber_digits_0$)
    RETURN
' end function parselinenumber%

' function striplinenumber$(text$)
140 trimleft_text_0$ = striplinenumber_text_0$
    GOSUB 10
    striplinenumber_text_0$ = trimleft_result_0$
    striplinenumber_i_0% = 1
    striplinenumber_done_0% = 0
150 IF ((striplinenumber_i_0% <= LEN(striplinenumber_text_0$)) AND (striplinenumber_done_0% = 0)) = 0 THEN GOTO 180
        striplinenumber_ch_0$ = MID$(striplinenumber_text_0$, striplinenumber_i_0%, 1)
        IF ((striplinenumber_ch_0$ >= "0") AND (striplinenumber_ch_0$ <= "9")) = 0 THEN GOTO 160
            striplinenumber_i_0% = striplinenumber_i_0% + 1
            GOTO 170
160 striplinenumber_done_0% = 1
170 REM END IF
        GOTO 150
180 REM END WHILE
    IF (striplinenumber_i_0% > LEN(striplinenumber_text_0$)) = 0 THEN GOTO 190
        striplinenumber_result_0$ = ""
        RETURN
190 REM END IF
    IF (MID$(striplinenumber_text_0$, striplinenumber_i_0%, 1) = " ") = 0 THEN GOTO 200
        striplinenumber_i_0% = striplinenumber_i_0% + 1
200 REM END IF
    striplinenumber_result_0$ = MID$(striplinenumber_text_0$, striplinenumber_i_0%)
    RETURN
' end function striplinenumber$

' function addref%(lineno%)
210 IF (addref_lineno_0% = 0) = 0 THEN GOTO 220
        addref_result_0% = 0
        RETURN
220 REM END IF
    addref_i_0% = 1
230 IF (addref_i_0% <= refcount%) = 0 THEN GOTO 250
        IF (refnumber%(addref_i_0%) = addref_lineno_0%) = 0 THEN GOTO 240
            addref_result_0% = 0
            RETURN
240 REM END IF
        addref_i_0% = addref_i_0% + 1
        GOTO 230
250 REM END WHILE
    IF (refcount% >= 1000) = 0 THEN GOTO 260
        addref_result_0% = 0
        RETURN
260 REM END IF
    refcount% = refcount% + 1
    refnumber%(refcount%) = addref_lineno_0%
    addref_result_0% = 1
    RETURN
' end function addref%

' function isreferenced%(lineno%)
270 isreferenced_i_0% = 1
280 IF (isreferenced_i_0% <= refcount%) = 0 THEN GOTO 300
        IF (refnumber%(isreferenced_i_0%) = isreferenced_lineno_0%) = 0 THEN GOTO 290
            isreferenced_result_0% = 1
            RETURN
290 REM END IF
        isreferenced_i_0% = isreferenced_i_0% + 1
        GOTO 280
300 REM END WHILE
    isreferenced_result_0% = 0
    RETURN
' end function isreferenced%

' function collectrefs%(line$)
310 collectrefs_found_0% = 0
    scankeywordrefs_line_0$ = collectrefs_line_0$
    scankeywordrefs_keyword_0$ = "GOTO"
    GOSUB 320
    collectrefs_found_0% = collectrefs_found_0% OR scankeywordrefs_result_0%
    scankeywordrefs_line_0$ = collectrefs_line_0$
    scankeywordrefs_keyword_0$ = "GOSUB"
    GOSUB 320
    collectrefs_found_0% = collectrefs_found_0% OR scankeywordrefs_result_0%
    scankeywordrefs_line_0$ = collectrefs_line_0$
    scankeywordrefs_keyword_0$ = "THEN"
    GOSUB 320
    collectrefs_found_0% = collectrefs_found_0% OR scankeywordrefs_result_0%
    scankeywordrefs_line_0$ = collectrefs_line_0$
    scankeywordrefs_keyword_0$ = "ELSE"
    GOSUB 320
    collectrefs_found_0% = collectrefs_found_0% OR scankeywordrefs_result_0%
    scankeywordrefs_line_0$ = collectrefs_line_0$
    scankeywordrefs_keyword_0$ = "RESTORE"
    GOSUB 320
    collectrefs_found_0% = collectrefs_found_0% OR scankeywordrefs_result_0%
    scankeywordrefs_line_0$ = collectrefs_line_0$
    scankeywordrefs_keyword_0$ = "RESUME"
    GOSUB 320
    collectrefs_found_0% = collectrefs_found_0% OR scankeywordrefs_result_0%
    scankeywordrefs_line_0$ = collectrefs_line_0$
    scankeywordrefs_keyword_0$ = "RUN"
    GOSUB 320
    collectrefs_found_0% = collectrefs_found_0% OR scankeywordrefs_result_0%
    collectrefs_result_0% = collectrefs_found_0%
    RETURN
' end function collectrefs%

' function scankeywordrefs%(line$, keyword$)
320 upper_text_0$ = scankeywordrefs_line_0$
    GOSUB 50
    scankeywordrefs_ul_0$ = upper_result_0$
    upper_text_0$ = scankeywordrefs_keyword_0$
    GOSUB 50
    scankeywordrefs_uk_0$ = upper_result_0$
    POS% = INSTR(scankeywordrefs_ul_0$, scankeywordrefs_uk_0$)
    IF (POS% = 0) = 0 THEN GOTO 330
        scankeywordrefs_result_0% = 0
        RETURN
330 REM END IF
    trimleft_text_0$ = MID$(scankeywordrefs_line_0$, POS% + LEN(scankeywordrefs_keyword_0$))
    GOSUB 10
    scankeywordrefs_after_0$ = trimleft_result_0$
    parselinenumber_text_0$ = scankeywordrefs_after_0$
    GOSUB 80
    scankeywordrefs_ref_0% = parselinenumber_result_0%
    IF (scankeywordrefs_ref_0% > 0) = 0 THEN GOTO 340
        addref_lineno_0% = scankeywordrefs_ref_0%
        GOSUB 210
        scankeywordrefs_result_0% = 1
        RETURN
340 REM END IF
    scankeywordrefs_result_0% = 0
    RETURN
' end function scankeywordrefs%

' function loadlines%()
350 refcount% = 0
    linecount% = 0
    OPEN inputfile$ FOR INPUT AS #1
360 IF (EOF(1) = 0) = 0 THEN GOTO 370
        linecount% = linecount% + 1
        LINE INPUT #1, rawline$(linecount%)
        GOTO 360
370 REM END WHILE
    CLOSE #1
    loadlines_i_0% = 1
380 IF (loadlines_i_0% <= linecount%) = 0 THEN GOTO 390
        parselinenumber_text_0$ = rawline$(loadlines_i_0%)
        GOSUB 80
        linenumber%(loadlines_i_0%) = parselinenumber_result_0%
        striplinenumber_text_0$ = rawline$(loadlines_i_0%)
        GOSUB 140
        linetext$(loadlines_i_0%) = striplinenumber_result_0$
        keepline%(loadlines_i_0%) = 0
        loadlines_i_0% = loadlines_i_0% + 1
        GOTO 380
390 REM END WHILE
    loadlines_result_0% = 0
    RETURN
' end function loadlines%

' function collectallrefs%()
400 refcount% = 0
    collectallrefs_i_0% = 1
410 IF (collectallrefs_i_0% <= linecount%) = 0 THEN GOTO 420
        collectrefs_line_0$ = linetext$(collectallrefs_i_0%)
        GOSUB 310
        keepline%(collectallrefs_i_0%) = collectrefs_result_0%
        collectallrefs_i_0% = collectallrefs_i_0% + 1
        GOTO 410
420 REM END WHILE
    collectallrefs_result_0% = 0
    RETURN
' end function collectallrefs%

' function transformlines%()
430 OPEN outputfile$ FOR OUTPUT AS #2
    transformlines_i_0% = 1
440 IF (transformlines_i_0% <= linecount%) = 0 THEN GOTO 490
        IF (linenumber%(transformlines_i_0%) > 0) = 0 THEN GOTO 470
            isreferenced_lineno_0% = linenumber%(transformlines_i_0%)
            GOSUB 270
            IF ((keepline%(transformlines_i_0%) <> 0) OR (isreferenced_result_0% <> 0)) = 0 THEN GOTO 450
                trimleft_text_0$ = STR$(linenumber%(transformlines_i_0%))
                GOSUB 10
                PRINT #2, (trimleft_result_0$ + " ") + linetext$(transformlines_i_0%)
                GOTO 460
450 PRINT #2, linetext$(transformlines_i_0%)
460 REM END IF
            GOTO 480
470 PRINT #2, linetext$(transformlines_i_0%)
480 REM END IF
        transformlines_i_0% = transformlines_i_0% + 1
        GOTO 440
490 REM END WHILE
    CLOSE #2
    transformlines_result_0% = 0
    RETURN
' end function transformlines%
