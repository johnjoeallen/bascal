' BASCAL generated BASIC
' Functions are lowered to global variables, labels, and GOSUB

' Shared string helpers for REMLINE.

' Parse and strip leading decimal line numbers.

' Fixed-size reference tracking for the example.

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
    trimleft_i% = 1
20 IF (trimleft_i% <= LEN(trimleft_text$)) = 0 THEN GOTO 40
        trimleft_ch$ = MID$(trimleft_text$, trimleft_i%, 1)
        IF (trimleft_ch$ <> " ") = 0 THEN GOTO 30
            trimleft_result$ = MID$(trimleft_text$, trimleft_i%)
            RETURN
30 REM END IF
        trimleft_i% = trimleft_i% + 1
        GOTO 20
40 REM END WHILE
    trimleft_result$ = ""
    RETURN
' end function trimleft$

' function upper$(text$)
50 upper_result$ = UCASE$(upper_text$)
    RETURN
' end function upper$

' function startswithkeyword%(text$, keyword$)
60 trimleft_text$ = startswithkeyword_text$
    GOSUB 10
    startswithkeyword_t$ = trimleft_result$
    startswithkeyword_kw$ = startswithkeyword_keyword$
    upper_text$ = startswithkeyword_t$
    GOSUB 50
    startswithkeyword_t$ = upper_result$
    upper_text$ = startswithkeyword_kw$
    GOSUB 50
    startswithkeyword_kw$ = upper_result$
    IF (LEN(startswithkeyword_t$) < LEN(startswithkeyword_kw$)) = 0 THEN GOTO 70
        startswithkeyword_result% = 0
        RETURN
70 REM END IF
    startswithkeyword_result% = LEFT$(startswithkeyword_t$, LEN(startswithkeyword_kw$)) = startswithkeyword_kw$
    RETURN
' end function startswithkeyword%

' function parselinenumber%(text$)
80 trimleft_text$ = parselinenumber_text$
    GOSUB 10
    parselinenumber_text$ = trimleft_result$
    parselinenumber_digits$ = ""
    parselinenumber_i% = 1
    parselinenumber_done% = 0
90 IF ((parselinenumber_i% <= LEN(parselinenumber_text$)) AND (parselinenumber_done% = 0)) = 0 THEN GOTO 120
        parselinenumber_ch$ = MID$(parselinenumber_text$, parselinenumber_i%, 1)
        IF ((parselinenumber_ch$ >= "0") AND (parselinenumber_ch$ <= "9")) = 0 THEN GOTO 100
            parselinenumber_digits$ = parselinenumber_digits$ + parselinenumber_ch$
            GOTO 110
100 parselinenumber_done% = 1
110 REM END IF
        parselinenumber_i% = parselinenumber_i% + 1
        GOTO 90
120 REM END WHILE
    IF (LEN(parselinenumber_digits$) = 0) = 0 THEN GOTO 130
        parselinenumber_result% = 0
        RETURN
130 REM END IF
    parselinenumber_result% = VAL(parselinenumber_digits$)
    RETURN
' end function parselinenumber%

' function striplinenumber$(text$)
140 trimleft_text$ = striplinenumber_text$
    GOSUB 10
    striplinenumber_text$ = trimleft_result$
    striplinenumber_i% = 1
    striplinenumber_done% = 0
150 IF ((striplinenumber_i% <= LEN(striplinenumber_text$)) AND (striplinenumber_done% = 0)) = 0 THEN GOTO 180
        striplinenumber_ch$ = MID$(striplinenumber_text$, striplinenumber_i%, 1)
        IF ((striplinenumber_ch$ >= "0") AND (striplinenumber_ch$ <= "9")) = 0 THEN GOTO 160
            striplinenumber_i% = striplinenumber_i% + 1
            GOTO 170
160 striplinenumber_done% = 1
170 REM END IF
        GOTO 150
180 REM END WHILE
    IF (striplinenumber_i% > LEN(striplinenumber_text$)) = 0 THEN GOTO 190
        striplinenumber_result$ = ""
        RETURN
190 REM END IF
    IF (MID$(striplinenumber_text$, striplinenumber_i%, 1) = " ") = 0 THEN GOTO 200
        striplinenumber_i% = striplinenumber_i% + 1
200 REM END IF
    striplinenumber_result$ = MID$(striplinenumber_text$, striplinenumber_i%)
    RETURN
' end function striplinenumber$

' function addref%(lineno%)
210 IF (addref_lineno% = 0) = 0 THEN GOTO 220
        addref_result% = 0
        RETURN
220 REM END IF
    addref_i% = 1
230 IF (addref_i% <= refcount%) = 0 THEN GOTO 250
        IF (refnumber%(addref_i%) = addref_lineno%) = 0 THEN GOTO 240
            addref_result% = 0
            RETURN
240 REM END IF
        addref_i% = addref_i% + 1
        GOTO 230
250 REM END WHILE
    IF (refcount% >= 1000) = 0 THEN GOTO 260
        addref_result% = 0
        RETURN
260 REM END IF
    refcount% = refcount% + 1
    refnumber%(refcount%) = addref_lineno%
    addref_result% = 1
    RETURN
' end function addref%

' function isreferenced%(lineno%)
270 isreferenced_i% = 1
280 IF (isreferenced_i% <= refcount%) = 0 THEN GOTO 300
        IF (refnumber%(isreferenced_i%) = isreferenced_lineno%) = 0 THEN GOTO 290
            isreferenced_result% = 1
            RETURN
290 REM END IF
        isreferenced_i% = isreferenced_i% + 1
        GOTO 280
300 REM END WHILE
    isreferenced_result% = 0
    RETURN
' end function isreferenced%

' function collectrefs%(line$)
310 collectrefs_found% = 0
    scankeywordrefs_line$ = collectrefs_line$
    scankeywordrefs_keyword$ = "GOTO"
    GOSUB 320
    collectrefs_found% = collectrefs_found% OR scankeywordrefs_result%
    scankeywordrefs_line$ = collectrefs_line$
    scankeywordrefs_keyword$ = "GOSUB"
    GOSUB 320
    collectrefs_found% = collectrefs_found% OR scankeywordrefs_result%
    scankeywordrefs_line$ = collectrefs_line$
    scankeywordrefs_keyword$ = "THEN"
    GOSUB 320
    collectrefs_found% = collectrefs_found% OR scankeywordrefs_result%
    scankeywordrefs_line$ = collectrefs_line$
    scankeywordrefs_keyword$ = "ELSE"
    GOSUB 320
    collectrefs_found% = collectrefs_found% OR scankeywordrefs_result%
    scankeywordrefs_line$ = collectrefs_line$
    scankeywordrefs_keyword$ = "RESTORE"
    GOSUB 320
    collectrefs_found% = collectrefs_found% OR scankeywordrefs_result%
    scankeywordrefs_line$ = collectrefs_line$
    scankeywordrefs_keyword$ = "RESUME"
    GOSUB 320
    collectrefs_found% = collectrefs_found% OR scankeywordrefs_result%
    scankeywordrefs_line$ = collectrefs_line$
    scankeywordrefs_keyword$ = "RUN"
    GOSUB 320
    collectrefs_found% = collectrefs_found% OR scankeywordrefs_result%
    collectrefs_result% = collectrefs_found%
    RETURN
' end function collectrefs%

' function scankeywordrefs%(line$, keyword$)
320 upper_text$ = scankeywordrefs_line$
    GOSUB 50
    scankeywordrefs_ul$ = upper_result$
    upper_text$ = scankeywordrefs_keyword$
    GOSUB 50
    scankeywordrefs_uk$ = upper_result$
    scankeywordrefs_pos% = INSTR(scankeywordrefs_ul$, scankeywordrefs_uk$)
    IF (scankeywordrefs_pos% = 0) = 0 THEN GOTO 330
        scankeywordrefs_result% = 0
        RETURN
330 REM END IF
    trimleft_text$ = MID$(scankeywordrefs_line$, scankeywordrefs_pos% + LEN(scankeywordrefs_keyword$))
    GOSUB 10
    scankeywordrefs_after$ = trimleft_result$
    parselinenumber_text$ = scankeywordrefs_after$
    GOSUB 80
    scankeywordrefs_ref% = parselinenumber_result%
    IF (scankeywordrefs_ref% > 0) = 0 THEN GOTO 340
        addref_lineno% = scankeywordrefs_ref%
        GOSUB 210
        scankeywordrefs_result% = 1
        RETURN
340 REM END IF
    scankeywordrefs_result% = 0
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
    loadlines_i% = 1
380 IF (loadlines_i% <= linecount%) = 0 THEN GOTO 390
        parselinenumber_text$ = rawline$(loadlines_i%)
        GOSUB 80
        linenumber%(loadlines_i%) = parselinenumber_result%
        striplinenumber_text$ = rawline$(loadlines_i%)
        GOSUB 140
        linetext$(loadlines_i%) = striplinenumber_result$
        keepline%(loadlines_i%) = 0
        loadlines_i% = loadlines_i% + 1
        GOTO 380
390 REM END WHILE
    loadlines_result% = 0
    RETURN
' end function loadlines%

' function collectallrefs%()
400 refcount% = 0
    collectallrefs_i% = 1
410 IF (collectallrefs_i% <= linecount%) = 0 THEN GOTO 420
        collectrefs_line$ = linetext$(collectallrefs_i%)
        GOSUB 310
        keepline%(collectallrefs_i%) = collectrefs_result%
        collectallrefs_i% = collectallrefs_i% + 1
        GOTO 410
420 REM END WHILE
    collectallrefs_result% = 0
    RETURN
' end function collectallrefs%

' function transformlines%()
430 OPEN outputfile$ FOR OUTPUT AS #2
    transformlines_i% = 1
440 IF (transformlines_i% <= linecount%) = 0 THEN GOTO 490
        IF (linenumber%(transformlines_i%) > 0) = 0 THEN GOTO 470
            isreferenced_lineno% = linenumber%(transformlines_i%)
            GOSUB 270
            IF ((keepline%(transformlines_i%) <> 0) OR (isreferenced_result% <> 0)) = 0 THEN GOTO 450
                trimleft_text$ = STR$(linenumber%(transformlines_i%))
                GOSUB 10
                PRINT #2, (trimleft_result$ + " ") + linetext$(transformlines_i%)
                GOTO 460
450 PRINT #2, linetext$(transformlines_i%)
460 REM END IF
            GOTO 480
470 PRINT #2, linetext$(transformlines_i%)
480 REM END IF
        transformlines_i% = transformlines_i% + 1
        GOTO 440
490 REM END WHILE
    CLOSE #2
    transformlines_result% = 0
    RETURN
' end function transformlines%
