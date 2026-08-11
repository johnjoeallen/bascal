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
' - CLERK.BAS is interactive (INKEY$/INPUT-driven menu, multi-diskette
' file registry, HARD COPY prompts). This example drives the same
' catalog operations with a fixed, deterministic call sequence instead,
' matching this repo's other standalone examples (tutorial/14_procedures.bcl,
' tutorial/sort_driver.bcl) so it compiles *and runs* without a keyboard.
' - Each menu action (NEW ITEM, list, the two searches, DELETE ITEM) is
' its own `procedure` — addItem, listAll, searchByAuthor,
' searchByAuthorTitle, deleteItem — instead of CLERK.BAS's numbered
' GOTO/GOSUB sections. This is the canonical BASCAL style (see
' MANUAL.md's Procedures section), and specifically exercises record/file
' access from inside a procedure body, not just top-level code.
' - CLERK.BAS's original also supported search-by-subject and a
' "another file"/multi-diskette workflow; this example keeps the two
' named searches and the single-catalog-file shape and drops the rest,
' to stay focused on what the record/file DSL and procedures are
' actually demonstrating here.

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

' ---- MENU=1 subroutine in CLERK.BAS: list every non-empty entry ----

' ---- MENU=2 subroutine in CLERK.BAS: filter by author ----

' ---- MENU=3 subroutine in CLERK.BAS: filter by author AND title ----

' ---- CHOICE=3 DELETE ITEM in CLERK.BAS: first author+title match ----

' --- Drive the catalog ---

GOSUB 10

additem_author_0$ = "Twain, Mark"
additem_title_0$ = "Adventures of Huckleberry Finn"
additem_subject_0$ = "Fiction"
GOSUB 20
additem_author_0$ = "Orwell, George"
additem_title_0$ = "Nineteen Eighty-Four"
additem_subject_0$ = "Fiction"
GOSUB 20
additem_author_0$ = "Fant, Alfred"
additem_title_0$ = "LIBRARIAN"
additem_subject_0$ = "Programming"
GOSUB 20
additem_author_0$ = "Lujan S., Carlos"
additem_title_0$ = "CLERK"
additem_subject_0$ = "Programming"
GOSUB 20

PRINT "-- Full catalog --"
GOSUB 90

PRINT ""
PRINT "-- Search by author: Orwell, George --"
searchbyauthor_author_0$ = "Orwell, George"
GOSUB 110

PRINT ""
PRINT "-- Search by author + title: Twain, Mark / Adventures of Huckleberry Finn --"
searchbyauthortitle_author_0$ = "Twain, Mark"
searchbyauthortitle_title_0$ = "Adventures of Huckleberry Finn"
GOSUB 130

PRINT ""
PRINT "-- Delete: Fant, Alfred / LIBRARIAN --"
deleteitem_author_0$ = "Fant, Alfred"
deleteitem_title_0$ = "LIBRARIAN"
GOSUB 150

PRINT ""
PRINT "-- Full catalog after delete --"
GOSUB 90

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
160 IF (deleteitem_stop_0% = 0) = 0 THEN GOTO 200
        deleteitem_i_0% = deleteitem_i_0% + 1
        ' let e = catalog[...]  (whole-record read)
        GET #2, deleteitem_i_0%
        deleteitem_e_author_0$ = RTRIM$(catalog_authorbuf$)
        deleteitem_e_title_0$ = RTRIM$(catalog_titlebuf$)
        deleteitem_e_subject_0$ = RTRIM$(catalog_subjectbuf$)
        IF (deleteitem_e_author_0$ = deleteitem_author_0$) = 0 THEN GOTO 180
            IF (deleteitem_e_title_0$ = deleteitem_title_0$) = 0 THEN GOTO 170
                deleteitem_stop_0% = 1
170 REM END IF
180 REM END IF
        IF (deleteitem_i_0% = deleteitem_h_size_0%) = 0 THEN GOTO 190
            deleteitem_stop_0% = 1
190 REM END IF
        GOTO 160
200 REM END DO
    IF (deleteitem_e_author_0$ = deleteitem_author_0$) = 0 THEN GOTO 210
    IF (deleteitem_e_title_0$ = deleteitem_title_0$) = 0 THEN GOTO 210
        PRINT (("Deleting: " + deleteitem_e_author_0$) + "  |  ") + deleteitem_e_title_0$
        ' catalog[...] = { ... }  (whole-record write)
        LSET catalog_authorbuf$ = ""
        LSET catalog_titlebuf$ = ""
        LSET catalog_subjectbuf$ = ""
        PUT #2, deleteitem_i_0%
        GOTO 220
210 PRINT (("Not found: " + deleteitem_author_0$) + "  |  ") + deleteitem_title_0$
220 REM END IF
    RETURN
' end procedure deleteitem
