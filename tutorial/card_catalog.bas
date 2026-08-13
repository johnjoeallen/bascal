' BASCAL generated BASIC
' Functions are transpiled to global variables, labels, and GOSUB

' Card Catalog — a flagship example for the record/file DSL + procedures
' 
' Adapted from CLERK.BAS, a menu-driven card-catalog manager written by
' Carlos A. Lujan S. in February 1983 as an improved version of Alfred
' Fant's LIBRARIAN program (Microcomputing, December 1982). The original
' source lives in the PeatSoft GW-FILES collection, in the
' robhagemans/hoard-of-gwbasic archive on GitHub
' (PeatSoft/GWFILES/CLERK.BAS).
' 
' What's carried over from CLERK.BAS:
' - One random-access file holding a header record (the catalog's
' capacity) in slot 1, followed by author/title/subject entry records
' in the remaining slots.
' - NEW ITEM: linear-scan the entries for the first empty slot.
' - Searches by author, and by author + title together.
' - DELETE ITEM: linear-scan for the first author+title match, blank it.
' 
' What's adapted rather than ported line-for-line:
' - The menu is still interactive (INPUT-driven, like CLERK.BAS's own
' INKEY$/ON CHOICE GOSUB loop), but each menu action (NEW ITEM, list,
' the two searches, DELETE ITEM) is its own `procedure` — addItem,
' listAll, searchByAuthor, searchByAuthorTitle, deleteItem — called
' from a `mainMenu` dispatch procedure using `select case`, instead of
' CLERK.BAS's numbered GOTO/GOSUB sections. This is the canonical
' BASCAL style (see MANUAL.md's Procedures section), and specifically
' exercises record/file access from inside a procedure body, not just
' top-level code.
' - CLERK.BAS's original also supported a multi-diskette/multi-file
' registry (drive letter + FILEDAT), search-by-subject, and HARD COPY
' (LPRINT) output. This example keeps one catalog file and the two
' named searches, and drops the rest, to stay focused on what the
' record/file DSL and procedures are actually demonstrating here.

' The header occupies slot 1 of the same file, sized to match Entry's
' width (20+20+20 = 60 bytes) so both record types agree on where every
' slot starts. size is the last valid entry slot number, mirroring
' CLERK.BAS's own S = CVI(F$) header field.

CONST lastslot% = 11

' file header as Header = open(...)  [60 bytes/record]
OPEN "catalog.dat" FOR RANDOM AS #1 LEN = 60
FIELD #1, 2 AS header_sizebuf$, 58 AS header_reservedbuf$
' file catalog as Entry = open(...)  [60 bytes/record]
OPEN "catalog.dat" FOR RANDOM AS #2 LEN = 60
FIELD #2, 20 AS catalog_authorbuf$, 20 AS catalog_titlebuf$, 20 AS catalog_subjectbuf$

' ---- CHOICE=5 in CLERK.BAS: create/reset the catalog file ----

' ---- CHOICE=1 NEW ITEM in CLERK.BAS ----
' author$  -- new entry's author
' title$   -- new entry's title
' subject$ -- new entry's subject

' ---- MENU=1 subroutine in CLERK.BAS: list every non-empty entry ----

' ---- MENU=2 subroutine in CLERK.BAS: filter by author ----
' author$ -- author name to match

' ---- MENU=3 subroutine in CLERK.BAS: filter by author AND title ----
' author$ -- author name to match
' title$  -- title to match

' ---- CHOICE=3 DELETE ITEM in CLERK.BAS: first author+title match ----
' author$ -- author name to match
' title$  -- title to match

' ---- CLERK.BAS's own MENU / ON CHOICE GOSUB dispatch loop ----

' --- Drive the catalog ---

GOSUB 10
GOSUB 220

' header.close()
CLOSE #1
' catalog.close()
CLOSE #2

END

' procedure initcatalog()
10 ' header[...] = { ... }  (whole-record write)
    LSET header_sizebuf$ = MKI%(lastslot%)
    LSET header_reservedbuf$ = ""
    PUT #1, 1
    FOR initcatalog_i_0% = 2 TO lastslot%
        ' catalog[...] = { ... }  (whole-record write)
        LSET catalog_authorbuf$ = ""
        LSET catalog_titlebuf$ = ""
        LSET catalog_subjectbuf$ = ""
        PUT #2, initcatalog_i_0%
    NEXT initcatalog_i_0%
    RETURN
' end procedure initcatalog

' procedure additem(author$, title$, subject$)
20 ' let h = header[...]  (whole-record read)
    GET #1, 1
    additem_h_size_0% = CVI%(header_sizebuf$)
    additem_h_reserved_0$ = RTRIM$(header_reservedbuf$)
    additem_i_0% = 1
    additem_stop_0% = 0
30 IF (additem_stop_0% = 0) = 0 THEN GOTO 60
        additem_i_0% = additem_i_0% + 1
        ' let e = catalog[...]  (whole-record read)
        GET #2, additem_i_0%
        additem_e_author_0$ = RTRIM$(catalog_authorbuf$)
        additem_e_title_0$ = RTRIM$(catalog_titlebuf$)
        additem_e_subject_0$ = RTRIM$(catalog_subjectbuf$)
        IF (additem_e_author_0$ = "") = 0 THEN GOTO 40
            additem_stop_0% = 1
40 REM END IF
        IF (additem_i_0% = additem_h_size_0%) = 0 THEN GOTO 50
            additem_stop_0% = 1
50 REM END IF
        GOTO 30
60 REM END DO
    IF (additem_e_author_0$ = "") = 0 THEN GOTO 70
        ' catalog[...] = { ... }  (whole-record write)
        LSET catalog_authorbuf$ = additem_author_0$
        LSET catalog_titlebuf$ = additem_title_0$
        LSET catalog_subjectbuf$ = additem_subject_0$
        PUT #2, additem_i_0%
        GOTO 80
70 PRINT "Catalog is full -- cannot add " + additem_author_0$
80 REM END IF
    RETURN
' end procedure additem

' procedure listall()
90 ' let h = header[...]  (whole-record read)
    GET #1, 1
    listall_h_size_0% = CVI%(header_sizebuf$)
    listall_h_reserved_0$ = RTRIM$(header_reservedbuf$)
    FOR listall_i_0% = 2 TO listall_h_size_0%
        ' let e = catalog[...]  (whole-record read)
        GET #2, listall_i_0%
        listall_e_author_0$ = RTRIM$(catalog_authorbuf$)
        listall_e_title_0$ = RTRIM$(catalog_titlebuf$)
        listall_e_subject_0$ = RTRIM$(catalog_subjectbuf$)
        IF (listall_e_author_0$ <> "") = 0 THEN GOTO 100
            PRINT (((listall_e_author_0$ + "  |  ") + listall_e_title_0$) + "  |  ") + listall_e_subject_0$
100 REM END IF
    NEXT listall_i_0%
    RETURN
' end procedure listall

' procedure searchbyauthor(author$)
110 ' let h = header[...]  (whole-record read)
    GET #1, 1
    searchbyauthor_h_size_0% = CVI%(header_sizebuf$)
    searchbyauthor_h_reserved_0$ = RTRIM$(header_reservedbuf$)
    FOR searchbyauthor_i_0% = 2 TO searchbyauthor_h_size_0%
        ' let e = catalog[...]  (whole-record read)
        GET #2, searchbyauthor_i_0%
        searchbyauthor_e_author_0$ = RTRIM$(catalog_authorbuf$)
        searchbyauthor_e_title_0$ = RTRIM$(catalog_titlebuf$)
        searchbyauthor_e_subject_0$ = RTRIM$(catalog_subjectbuf$)
        IF (searchbyauthor_e_author_0$ = searchbyauthor_author_0$) = 0 THEN GOTO 120
            PRINT (((searchbyauthor_e_author_0$ + "  |  ") + searchbyauthor_e_title_0$) + "  |  ") + searchbyauthor_e_subject_0$
120 REM END IF
    NEXT searchbyauthor_i_0%
    RETURN
' end procedure searchbyauthor

' procedure searchbyauthortitle(author$, title$)
130 ' let h = header[...]  (whole-record read)
    GET #1, 1
    searchbyauthortitle_h_size_0% = CVI%(header_sizebuf$)
    searchbyauthortitle_h_reserved_0$ = RTRIM$(header_reservedbuf$)
    FOR searchbyauthortitle_i_0% = 2 TO searchbyauthortitle_h_size_0%
        ' let e = catalog[...]  (whole-record read)
        GET #2, searchbyauthortitle_i_0%
        searchbyauthortitle_e_author_0$ = RTRIM$(catalog_authorbuf$)
        searchbyauthortitle_e_title_0$ = RTRIM$(catalog_titlebuf$)
        searchbyauthortitle_e_subject_0$ = RTRIM$(catalog_subjectbuf$)
        IF (searchbyauthortitle_e_author_0$ = searchbyauthortitle_author_0$) = 0 THEN GOTO 140
        IF (searchbyauthortitle_e_title_0$ = searchbyauthortitle_title_0$) = 0 THEN GOTO 140
            PRINT (((searchbyauthortitle_e_author_0$ + "  |  ") + searchbyauthortitle_e_title_0$) + "  |  ") + searchbyauthortitle_e_subject_0$
140 REM END IF
    NEXT searchbyauthortitle_i_0%
    RETURN
' end procedure searchbyauthortitle

' procedure deleteitem(author$, title$)
150 ' let h = header[...]  (whole-record read)
    GET #1, 1
    deleteitem_h_size_0% = CVI%(header_sizebuf$)
    deleteitem_h_reserved_0$ = RTRIM$(header_reservedbuf$)
    deleteitem_i_0% = 1
    deleteitem_stop_0% = 0
160 IF (deleteitem_stop_0% = 0) = 0 THEN GOTO 190
        deleteitem_i_0% = deleteitem_i_0% + 1
        ' let e = catalog[...]  (whole-record read)
        GET #2, deleteitem_i_0%
        deleteitem_e_author_0$ = RTRIM$(catalog_authorbuf$)
        deleteitem_e_title_0$ = RTRIM$(catalog_titlebuf$)
        deleteitem_e_subject_0$ = RTRIM$(catalog_subjectbuf$)
        IF (deleteitem_e_author_0$ = deleteitem_author_0$) = 0 THEN GOTO 170
        IF (deleteitem_e_title_0$ = deleteitem_title_0$) = 0 THEN GOTO 170
            deleteitem_stop_0% = 1
170 REM END IF
        IF (deleteitem_i_0% = deleteitem_h_size_0%) = 0 THEN GOTO 180
            deleteitem_stop_0% = 1
180 REM END IF
        GOTO 160
190 REM END DO
    IF (deleteitem_e_author_0$ = deleteitem_author_0$) = 0 THEN GOTO 200
    IF (deleteitem_e_title_0$ = deleteitem_title_0$) = 0 THEN GOTO 200
        PRINT (("Deleting: " + deleteitem_e_author_0$) + "  |  ") + deleteitem_e_title_0$
        ' catalog[...] = { ... }  (whole-record write)
        LSET catalog_authorbuf$ = ""
        LSET catalog_titlebuf$ = ""
        LSET catalog_subjectbuf$ = ""
        PUT #2, deleteitem_i_0%
        GOTO 210
200 PRINT (("Not found: " + deleteitem_author_0$) + "  |  ") + deleteitem_title_0$
210 REM END IF
    RETURN
' end procedure deleteitem

' procedure mainmenu()
220 mainmenu_running_0% = 1
230 IF (mainmenu_running_0% = 1) = 0 THEN GOTO 320
        PRINT ""
        PRINT "MENU.          1 ) LIST ALL ITEMS"
        PRINT "               2 ) NEW ITEM"
        PRINT "               3 ) SEARCH BY AUTHOR"
        PRINT "               4 ) SEARCH BY AUTHOR + TITLE"
        PRINT "               5 ) DELETE ITEM"
        PRINT "               6 ) STOP"
        PRINT ""
        INPUT "CHOICE: "; mainmenu_choice_0%

        BCC_T14% = mainmenu_choice_0%
        IF (BCC_T14% = 1) <> 0 THEN GOTO 240
        IF (BCC_T14% = 2) <> 0 THEN GOTO 250
        IF (BCC_T14% = 3) <> 0 THEN GOTO 260
        IF (BCC_T14% = 4) <> 0 THEN GOTO 270
        IF (BCC_T14% = 5) <> 0 THEN GOTO 280
        IF (BCC_T14% = 6) <> 0 THEN GOTO 290
        GOTO 300
240 GOSUB 90
            GOTO 310
250 INPUT "AUTHOR  "; mainmenu_author_0$
            INPUT "TITLE   "; mainmenu_title_0$
            INPUT "SUBJECT "; mainmenu_subject_0$
            additem_author_0$ = mainmenu_author_0$
            additem_title_0$ = mainmenu_title_0$
            additem_subject_0$ = mainmenu_subject_0$
            GOSUB 20
            GOTO 310
260 INPUT "AUTHOR "; mainmenu_author_0$
            searchbyauthor_author_0$ = mainmenu_author_0$
            GOSUB 110
            GOTO 310
270 INPUT "AUTHOR "; mainmenu_author_0$
            INPUT "TITLE  "; mainmenu_title_0$
            searchbyauthortitle_author_0$ = mainmenu_author_0$
            searchbyauthortitle_title_0$ = mainmenu_title_0$
            GOSUB 130
            GOTO 310
280 INPUT "AUTHOR (to delete) "; mainmenu_author_0$
            INPUT "TITLE  (to delete) "; mainmenu_title_0$
            deleteitem_author_0$ = mainmenu_author_0$
            deleteitem_title_0$ = mainmenu_title_0$
            GOSUB 150
            GOTO 310
290 mainmenu_running_0% = 0
            GOTO 310
300 PRINT "Invalid choice"
310 REM END SELECT
        GOTO 230
320 REM END DO
    RETURN
' end procedure mainmenu
