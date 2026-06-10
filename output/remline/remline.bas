' BASCAL generated BASIC
' Functions are lowered to global variables, labels, and GOSUB

' Shared string helpers for REMLINE.
' These helpers keep the parsing code readable without pretending BASCAL has
' local scope or modules.

' Parse and strip leading decimal line numbers.
' The example keeps parsing deliberately small and boring: a source line may
' start with digits, optionally followed by one separating space.

' Fixed-size reference tracking for the example.
' BASCAL keeps everything global, so the reference list is a small shared
' buffer that every helper can see.

' Embedded sample input matching sample/input.bas.
' The example prints the transformed listing to stdout, which keeps the first
' version simple while BASCAL grows file I/O support.
' In classic BASIC terms, this is the same kind of cleanup pass that utilities
' like REMLINE.BAS did for line-numbered source listings.

DIM rawLine$(1000)
DIM lineText$(1000)
DIM lineNumber%(1000)
DIM keepLine%(1000)
DIM refNumber%(1000)

' REMLINE demo driver.
' The first version uses embedded sample data because file I/O is not yet part
' of the BASCAL runtime surface.
' The dependency graph is still real: the driver pulls in parsing, reference
' collection, and string helpers through BASCAL's path-style require syntax.

GOSUB 350
GOSUB 380
GOSUB 410
END

' function trimLeft$(text$)
10 ' BASIC string handling is straightforward: walk from the left until the
    ' first non-space character appears.
    trimleft_i% = 1
20 IF (trimleft_i% <= len(trimleft_text$)) = 0 THEN GOTO 40
        trimleft_ch$ = mid$(trimleft_text$, trimleft_i%, 1)
        IF (trimleft_ch$ <> " ") = 0 THEN GOTO 30
            trimleft_result$ = mid$(trimleft_text$, trimleft_i%)
            RETURN
30 REM END IF
        trimleft_i% = trimleft_i% + 1
        GOTO 20
40 REM END WHILE
    trimleft_result$ = ""
    RETURN
' end function trimLeft$

' function upper$(text$)
50 ' Use the BASIC runtime's upper-case conversion directly.
    upper_result$ = UCASE$(upper_text$)
    RETURN
' end function upper$

' function startsWithKeyword%(text$, keyword$)
60 ' Convenience helper for future extensions and tests.
    trimleft_text$ = startswithkeyword_text$
    GOSUB 10
    startswithkeyword_text$ = trimleft_result$
    IF (len(startswithkeyword_text$) < len(startswithkeyword_keyword$)) = 0 THEN GOTO 70
        startswithkeyword_result% = 0
        RETURN
70 REM END IF
    upper_text$ = left$(startswithkeyword_text$, len(startswithkeyword_keyword$))
    GOSUB 50
    upper_text$ = startswithkeyword_keyword$
    GOSUB 50
    startswithkeyword_result% = upper_result$ = upper_result$
    RETURN
' end function startsWithKeyword%

' function parseLineNumber%(text$)
80 ' Trim only the left edge so the helper tolerates indented lines.
    trimleft_text$ = parselinenumber_text$
    GOSUB 10
    parselinenumber_text$ = trimleft_result$
    digits$ = ""
    parselinenumber_i% = 1
    parselinenumber_done% = 0
90 IF ((parselinenumber_i% <= len(parselinenumber_text$)) AND (parselinenumber_done% = 0)) = 0 THEN GOTO 120
        parselinenumber_ch$ = mid$(parselinenumber_text$, parselinenumber_i%, 1)
        IF ((parselinenumber_ch$ >= "0") AND (parselinenumber_ch$ <= "9")) = 0 THEN GOTO 100
            digits$ = digits$ + parselinenumber_ch$
            GOTO 110
100 parselinenumber_done% = 1
110 REM END IF
        parselinenumber_i% = parselinenumber_i% + 1
        GOTO 90
120 REM END WHILE
    IF (len(digits$) = 0) = 0 THEN GOTO 130
        parselinenumber_result% = 0
        RETURN
130 REM END IF
    parselinenumber_result% = val(digits$)
    RETURN
' end function parseLineNumber%

' function stripLineNumber$(text$)
140 ' Remove an initial decimal line number and one following space if present.
    trimleft_text$ = striplinenumber_text$
    GOSUB 10
    striplinenumber_text$ = trimleft_result$
    striplinenumber_i% = 1
    striplinenumber_done% = 0
150 IF ((striplinenumber_i% <= len(striplinenumber_text$)) AND (striplinenumber_done% = 0)) = 0 THEN GOTO 180
        striplinenumber_ch$ = mid$(striplinenumber_text$, striplinenumber_i%, 1)
        IF ((striplinenumber_ch$ >= "0") AND (striplinenumber_ch$ <= "9")) = 0 THEN GOTO 160
            striplinenumber_i% = striplinenumber_i% + 1
            GOTO 170
160 striplinenumber_done% = 1
170 REM END IF
        GOTO 150
180 REM END WHILE
    IF (striplinenumber_i% > len(striplinenumber_text$)) = 0 THEN GOTO 190
        striplinenumber_result$ = ""
        RETURN
190 REM END IF
    IF (mid$(striplinenumber_text$, striplinenumber_i%, 1) = " ") = 0 THEN GOTO 200
        striplinenumber_i% = striplinenumber_i% + 1
200 REM END IF
    striplinenumber_result$ = mid$(striplinenumber_text$, striplinenumber_i%)
    RETURN
' end function stripLineNumber$

' function addRef%(lineNo%)
210 ' Zero is not a useful line-number target.
    IF (addref_lineno% = 0) = 0 THEN GOTO 220
        addref_result% = 0
        RETURN
220 REM END IF

    ' Do not record the same branch target twice.
    addref_i% = 1
230 IF (addref_i% <= refCount%) = 0 THEN GOTO 250
        IF (refNumber%(addref_i%) = addref_lineno%) = 0 THEN GOTO 240
            addref_result% = 0
            RETURN
240 REM END IF
        addref_i% = addref_i% + 1
        GOTO 230
250 REM END WHILE

    IF (refCount% >= 1000) = 0 THEN GOTO 260
        addref_result% = 0
        RETURN
260 REM END IF

    refCount% = refCount% + 1
    refNumber%(refCount%) = addref_lineno%
    addref_result% = 1
    RETURN
' end function addRef%

' function isReferenced%(lineNo%)
270 isreferenced_i% = 1
280 IF (isreferenced_i% <= refCount%) = 0 THEN GOTO 300
        IF (refNumber%(isreferenced_i%) = isreferenced_lineno%) = 0 THEN GOTO 290
            isreferenced_result% = 1
            RETURN
290 REM END IF
        isreferenced_i% = isreferenced_i% + 1
        GOTO 280
300 REM END WHILE
    isreferenced_result% = 0
    RETURN
' end function isReferenced%

' function collectRefs%(line$)
310 ' Scan for the small set of direct numeric flow-control forms this example
    ' supports.
    collectrefs_found% = 0
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
' end function collectRefs%

' function scanKeywordRefs%(line$, keyword$)
320 ' We keep the scan intentionally simple: locate the keyword and read the
    ' first number that follows it.
    scankeywordrefs_upper_line$ = scankeywordrefs_line$
    scankeywordrefs_upper_keyword$ = scankeywordrefs_keyword$
    upper_text$ = scankeywordrefs_upper_line$
    GOSUB 50
    scankeywordrefs_upper_line$ = upper_result$
    upper_text$ = scankeywordrefs_upper_keyword$
    GOSUB 50
    scankeywordrefs_upper_keyword$ = upper_result$
    scankeywordrefs_pos% = INSTR(scankeywordrefs_upper_line$, scankeywordrefs_upper_keyword$)
    IF (scankeywordrefs_pos% = 0) = 0 THEN GOTO 330
        scankeywordrefs_result% = 0
        RETURN
330 REM END IF

    trimleft_text$ = mid$(scankeywordrefs_line$, scankeywordrefs_pos% + len(scankeywordrefs_keyword$))
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
' end function scanKeywordRefs%

' function loadSample%()
350 ' Reset the small fixed-size buffers used by the sample.
    refCount% = 0
    lineCount% = 13

    ' A tiny but representative BASIC listing. Line numbers that are branch
    ' targets must survive the cleanup pass.
    rawLine$(1) = "10 REM SAMPLE BASIC PROGRAM"
    rawLine$(2) = "20 GOSUB 1000"
    rawLine$(3) = "30 FOR I = 1 TO 5"
    rawLine$(4) = "40 PRINT I"
    rawLine$(5) = "50 NEXT I"
    rawLine$(6) = "60 IF I > 5 THEN 200"
    rawLine$(7) = (("70 PRINT " + CHR$(34)) + "SHOULD NOT HAPPEN") + CHR$(34)
    rawLine$(8) = "80 GOTO 300"
    rawLine$(9) = (("200 PRINT " + CHR$(34)) + "DONE") + CHR$(34)
    rawLine$(10) = "300 END"
    rawLine$(11) = "1000 REM HELPER ROUTINE"
    rawLine$(12) = (("1010 PRINT " + CHR$(34)) + "IN SUBROUTINE") + CHR$(34)
    rawLine$(13) = "1020 RETURN"

    loadsample_i% = 1
360 IF (loadsample_i% <= lineCount%) = 0 THEN GOTO 370
        ' First pass: split each input line into its numeric prefix and the
        ' remaining source text.
        parselinenumber_text$ = rawLine$(loadsample_i%)
        GOSUB 80
        lineNumber%(loadsample_i%) = parselinenumber_result%
        striplinenumber_text$ = rawLine$(loadsample_i%)
        GOSUB 140
        lineText$(loadsample_i%) = striplinenumber_result$
        keepLine%(loadsample_i%) = 0
        loadsample_i% = loadsample_i% + 1
        GOTO 360
370 REM END WHILE

    loadsample_result% = 0
    RETURN
' end function loadSample%

' function collectAllRefs%()
380 ' Second pass: scan each line for direct numeric branch targets.
    refCount% = 0
    collectallrefs_i% = 1
390 IF (collectallrefs_i% <= lineCount%) = 0 THEN GOTO 400
        collectrefs_line$ = lineText$(collectallrefs_i%)
        GOSUB 310
        keepLine%(collectallrefs_i%) = collectrefs_result%
        collectallrefs_i% = collectallrefs_i% + 1
        GOTO 390
400 REM END WHILE
    collectallrefs_result% = 0
    RETURN
' end function collectAllRefs%

' function transformLines%()
410 ' Final pass: emit a cleaned listing. Referenced targets keep their line
    ' numbers; unreferenced ordinary lines are written without them.
    transformlines_i% = 1
420 IF (transformlines_i% <= lineCount%) = 0 THEN GOTO 470
        IF (lineNumber%(transformlines_i%) > 0) = 0 THEN GOTO 450
            isreferenced_lineno% = lineNumber%(transformlines_i%)
            GOSUB 270
            IF ((keepLine%(transformlines_i%) <> 0) OR (isreferenced_result% <> 0)) = 0 THEN GOTO 430
                trimleft_text$ = STR$(lineNumber%(transformlines_i%))
                GOSUB 10
                PRINT (trimleft_result$ + " ") + lineText$(transformlines_i%)
                GOTO 440
430 PRINT lineText$(transformlines_i%)
440 REM END IF
            GOTO 460
450 PRINT lineText$(transformlines_i%)
460 REM END IF
        transformlines_i% = transformlines_i% + 1
        GOTO 420
470 REM END WHILE
    transformlines_result% = 0
    RETURN
' end function transformLines%
