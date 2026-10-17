10 ' BASCAL generated BASIC -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Strips leading spaces from self$. Not a real MBASIC/BASCOM 2.00 builtin --
40 ' verified against a real IBM BASIC Compiler 2.00 under dosbox-x -- so
50 ' BASCAL ships its own. Declared as a scalar method (see GitHub issue #41)
60 ' so a required stdlib call reads the same way as a built-in method call
70 ' (docs/language/functions-and-procedures.html#built-in-methods). The
80 ' ordinary call form (ltrim$(s$)) still works -- a method's receiver is an
90 ' implicit first parameter, so ordinary-call syntax resolves straight to
100 ' this same declaration, with no separate function needed (and no longer
110 ' allowed: a function and a method sharing one name is a duplicate
120 ' declaration, since they'd both claim the same callable identity).
130 ' Strips trailing spaces from self$. Not a real MBASIC/BASCOM 2.00 builtin --
140 ' verified against a real IBM BASIC Compiler 2.00 under dosbox-x -- so
150 ' BASCAL ships its own. Declared as a scalar method (see GitHub issue #41
160 ' and ltrim.bcl's own doc comment for the reasoning) -- rtrim$(s$) still
170 ' works via ordinary-call syntax resolving to this same declaration.
180 ' Card Catalog — a flagship example for the record/file DSL + procedures
190 ' 
200 ' Adapted from CLERK.BAS, a menu-driven card-catalog manager written by
210 ' Carlos A. Lujan S. in February 1983 as an improved version of Alfred
220 ' Fant's LIBRARIAN program (Microcomputing, December 1982). The original
230 ' source lives in the PeatSoft GW-FILES collection, in the
240 ' robhagemans/hoard-of-gwbasic archive on GitHub
250 ' (PeatSoft/GWFILES/CLERK.BAS).
260 ' 
270 ' What's carried over from CLERK.BAS:
280 ' - One random-access file holding a header record (the catalog's
290 ' capacity) in slot 1, followed by author/title/subject entry records
300 ' in the remaining slots.
310 ' - NEW ITEM: linear-scan the entries for the first empty slot.
320 ' - Searches by author, and by author + title together.
330 ' - DELETE ITEM: linear-scan for the first author+title match, blank it.
340 ' 
350 ' What's adapted rather than ported line-for-line:
360 ' - The menu is still interactive (INPUT-driven, like CLERK.BAS's own
370 ' INKEY$/ON CHOICE GOSUB loop), but each menu action (NEW ITEM, list,
380 ' the two searches, DELETE ITEM) is its own `procedure` — addItem,
390 ' listAll, searchByAuthor, searchByAuthorTitle, deleteItem — called
400 ' from a `mainMenu` dispatch procedure using `select case`, instead of
410 ' CLERK.BAS's numbered GOTO/GOSUB sections. This is the canonical
420 ' BASCAL style (see the manual's Procedures section at
430 ' https://johnjoeallen.github.io/bascal/manual/), and specifically
440 ' exercises record/file access from inside a procedure body, not just
450 ' top-level code.
460 ' - CLERK.BAS's original also supported a multi-diskette/multi-file
470 ' registry (drive letter + FILEDAT), search-by-subject, and HARD COPY
480 ' (LPRINT) output. This example keeps one catalog file and the two
490 ' named searches, and drops the rest, to stay focused on what the
500 ' record/file DSL and procedures are actually demonstrating here.
510 ' The header occupies slot 1 of the same file, sized to match Entry's
520 ' width (20+20+20 = 60 bytes) so both record types agree on where every
530 ' slot starts. size is the last valid entry slot number, mirroring
540 ' CLERK.BAS's own S = CVI(F$) header field.
550 CONSTLASTSLOT% = 11
560 ' 10 usable entry slots + 1 -- slot 1 is the header, 2..11 are entries
570 ' file header as Header = open(...)  [60 bytes/record]
580 OPEN "catalog.dat" FOR RANDOM AS #1 LEN = 60
590 FIELD #1, 2 AS headerSizeBuf$, 58 AS headerReservedBuf$
600 ' file catalog as Entry = open(...)  [60 bytes/record]
610 OPEN "catalog.dat" FOR RANDOM AS #2 LEN = 60
620 FIELD #2, 20 AS catalogAuthorBuf$, 20 AS catalogTitleBuf$, 20 AS catalogSubjectBuf$
630 ' ---- CHOICE=5 in CLERK.BAS: create/reset the catalog file ----
640 ' ---- CHOICE=1 NEW ITEM in CLERK.BAS ----
650 ' author$  -- new entry's author
660 ' title$   -- new entry's title
670 ' subject$ -- new entry's subject
680 ' ---- MENU=1 subroutine in CLERK.BAS: list every non-empty entry ----
690 ' ---- MENU=2 subroutine in CLERK.BAS: filter by author ----
700 ' author$ -- author name to match
710 ' ---- MENU=3 subroutine in CLERK.BAS: filter by author AND title ----
720 ' author$ -- author name to match
730 ' title$  -- title to match
740 ' ---- CHOICE=3 DELETE ITEM in CLERK.BAS: first author+title match ----
750 ' author$ -- author name to match
760 ' title$  -- title to match
770 ' ---- CLERK.BAS's own MENU / ON CHOICE GOSUB dispatch loop ----
780 ' --- Drive the catalog ---
790 GOSUB 1130
800 GOSUB 3200
810 ' header.close()
820 CLOSE #1
830 ' catalog.close()
840 CLOSE #2
850 END

860 ' function ltrim$(self$)
870     ltrimI0% = 1
880     IF (ltrimI0% <= LEN(ltrimSelf0$)) = 0 THEN GOTO 920
890     IF (MID$(ltrimSelf0$, ltrimI0%, 1) = " ") = 0 THEN GOTO 920
900         ltrimI0% = ltrimI0% + 1
910     GOTO 880
920     REM END WHILE
930     BCCT3$ = ltrimSelf0$
940     BCCT4% = ltrimI0%
950     BCCT5$ = MID$(BCCT3$, BCCT4%)
960     ltrimResult0$ = BCCT5$
970     RETURN
980 ' end function ltrim$

990 ' function rtrim$(self$)
1000     rtrimI0% = LEN(rtrimSelf0$)
1010     IF (rtrimI0% > 0) = 0 THEN GOTO 1050
1020     IF (MID$(rtrimSelf0$, rtrimI0%, 1) = " ") = 0 THEN GOTO 1050
1030         rtrimI0% = rtrimI0% - 1
1040     GOTO 1010
1050     REM END WHILE
1060     BCCT8$ = rtrimSelf0$
1070     BCCT9% = rtrimI0%
1080     BCCT10$ = LEFT$(BCCT8$, BCCT9%)
1090     rtrimResult0$ = BCCT10$
1100     RETURN
1110 ' end function rtrim$

1120 ' procedure initcatalog()
1130     ' global header
1140     ' global catalog
1150     ' header[...] = { ... }  (whole-record write)
1160     LSET headerSizeBuf$ = MKI$(CONSTLASTSLOT%)
1170     LSET headerReservedBuf$ = ""
1180     PUT #1, 1
1190     FOR initcatalogI0% = 2 TO CONSTLASTSLOT%
1200         ' catalog[...] = { ... }  (whole-record write)
1210         LSET catalogAuthorBuf$ = ""
1220         LSET catalogTitleBuf$ = ""
1230         LSET catalogSubjectBuf$ = ""
1240         PUT #2, initcatalogI0%
1250     NEXT initcatalogI0%
1260     RETURN
1270 ' end procedure initcatalog

1280 ' procedure additem(author$, title$, subject$)
1290     ' global header
1300     ' global catalog
1310     ' let h = header[...]  (whole-record read)
1320     GET #1, 1
1330     additemHSize0% = CVI(headerSizeBuf$)
1340     rtrimSelf0$ = headerReservedBuf$
1350     GOSUB 1000
1360     BCCT12$ = rtrimResult0$
1370     additemHReserved0$ = BCCT12$
1380     additemI0% = 1
1390     additemStop0% = 0
1400     IF (additemStop0% = 0) = 0 THEN GOTO 1630
1410         additemI0% = additemI0% + 1
1420         ' let e = catalog[...]  (whole-record read)
1430         GET #2, additemI0%
1440         rtrimSelf0$ = catalogAuthorBuf$
1450         GOSUB 1000
1460         BCCT14$ = rtrimResult0$
1470         additemEAuthor0$ = BCCT14$
1480         rtrimSelf0$ = catalogTitleBuf$
1490         GOSUB 1000
1500         BCCT15$ = rtrimResult0$
1510         additemETitle0$ = BCCT15$
1520         rtrimSelf0$ = catalogSubjectBuf$
1530         GOSUB 1000
1540         BCCT16$ = rtrimResult0$
1550         additemESubject0$ = BCCT16$
1560         IF (additemEAuthor0$ = "") = 0 THEN GOTO 1580
1570             additemStop0% = 1
1580         REM END IF
1590         IF (additemI0% = additemHSize0%) = 0 THEN GOTO 1610
1600             additemStop0% = 1
1610         REM END IF
1620     GOTO 1400
1630     REM END DO
1640     IF (additemEAuthor0$ = "") = 0 THEN GOTO 1710
1650         ' catalog[...] = { ... }  (whole-record write)
1660         LSET catalogAuthorBuf$ = additemAuthor0$
1670         LSET catalogTitleBuf$ = additemTitle0$
1680         LSET catalogSubjectBuf$ = additemSubject0$
1690         PUT #2, additemI0%
1700     GOTO 1720
1710         PRINT "Catalog is full -- cannot add " + additemAuthor0$
1720     REM END IF
1730     RETURN
1740 ' end procedure additem

1750 ' procedure listall()
1760     ' global header
1770     ' global catalog
1780     ' let h = header[...]  (whole-record read)
1790     GET #1, 1
1800     listallHSize0% = CVI(headerSizeBuf$)
1810     rtrimSelf0$ = headerReservedBuf$
1820     GOSUB 1000
1830     BCCT20$ = rtrimResult0$
1840     listallHReserved0$ = BCCT20$
1850     FOR listallI0% = 2 TO listallHSize0%
1860         ' let e = catalog[...]  (whole-record read)
1870         GET #2, listallI0%
1880         rtrimSelf0$ = catalogAuthorBuf$
1890         GOSUB 1000
1900         BCCT22$ = rtrimResult0$
1910         listallEAuthor0$ = BCCT22$
1920         rtrimSelf0$ = catalogTitleBuf$
1930         GOSUB 1000
1940         BCCT23$ = rtrimResult0$
1950         listallETitle0$ = BCCT23$
1960         rtrimSelf0$ = catalogSubjectBuf$
1970         GOSUB 1000
1980         BCCT24$ = rtrimResult0$
1990         listallESubject0$ = BCCT24$
2000         IF (listallEAuthor0$ <> "") = 0 THEN GOTO 2020
2010             PRINT (((listallEAuthor0$ + "  |  ") + listallETitle0$) + "  |  ") + listallESubject0$
2020         REM END IF
2030     NEXT listallI0%
2040     RETURN
2050 ' end procedure listall

2060 ' procedure searchbyauthor(author$)
2070     ' global header
2080     ' global catalog
2090     ' let h = header[...]  (whole-record read)
2100     GET #1, 1
2110     searchbyauthorHSize0% = CVI(headerSizeBuf$)
2120     rtrimSelf0$ = headerReservedBuf$
2130     GOSUB 1000
2140     BCCT26$ = rtrimResult0$
2150     searchbyauthorHReserved0$ = BCCT26$
2160     FOR searchbyauthorI0% = 2 TO searchbyauthorHSize0%
2170         ' let e = catalog[...]  (whole-record read)
2180         GET #2, searchbyauthorI0%
2190         rtrimSelf0$ = catalogAuthorBuf$
2200         GOSUB 1000
2210         BCCT28$ = rtrimResult0$
2220         searchbyauthorEAuthor0$ = BCCT28$
2230         rtrimSelf0$ = catalogTitleBuf$
2240         GOSUB 1000
2250         BCCT29$ = rtrimResult0$
2260         searchbyauthorETitle0$ = BCCT29$
2270         rtrimSelf0$ = catalogSubjectBuf$
2280         GOSUB 1000
2290         BCCT30$ = rtrimResult0$
2300         searchbyauthorESubject0$ = BCCT30$
2310         IF (searchbyauthorEAuthor0$ = searchbyauthorAuthor0$) = 0 THEN GOTO 2330
2320             PRINT (((searchbyauthorEAuthor0$ + "  |  ") + searchbyauthorETitle0$) + "  |  ") + searchbyauthorESubject0$
2330         REM END IF
2340     NEXT searchbyauthorI0%
2350     RETURN
2360 ' end procedure searchbyauthor

2370 ' procedure searchbyauthortitle(author$, title$)
2380     ' global header
2390     ' global catalog
2400     ' let h = header[...]  (whole-record read)
2410     GET #1, 1
2420     searchbyauthortitleHSize0% = CVI(headerSizeBuf$)
2430     rtrimSelf0$ = headerReservedBuf$
2440     GOSUB 1000
2450     BCCT32$ = rtrimResult0$
2460     searchbyauthortitleHReserved0$ = BCCT32$
2470     FOR searchbyauthortitleI0% = 2 TO searchbyauthortitleHSize0%
2480         ' let e = catalog[...]  (whole-record read)
2490         GET #2, searchbyauthortitleI0%
2500         rtrimSelf0$ = catalogAuthorBuf$
2510         GOSUB 1000
2520         BCCT34$ = rtrimResult0$
2530         searchbyauthortitleEAuthor0$ = BCCT34$
2540         rtrimSelf0$ = catalogTitleBuf$
2550         GOSUB 1000
2560         BCCT35$ = rtrimResult0$
2570         searchbyauthortitleETitle0$ = BCCT35$
2580         rtrimSelf0$ = catalogSubjectBuf$
2590         GOSUB 1000
2600         BCCT36$ = rtrimResult0$
2610         searchbyauthortitleESubject0$ = BCCT36$
2620         IF (searchbyauthortitleEAuthor0$ = searchbyauthortitleAuthor0$) = 0 THEN GOTO 2650
2630         IF (searchbyauthortitleETitle0$ = searchbyauthortitleTitle0$) = 0 THEN GOTO 2650
2640             PRINT (((searchbyauthortitleEAuthor0$ + "  |  ") + searchbyauthortitleETitle0$) + "  |  ") + searchbyauthortitleESubject0$
2650         REM END IF
2660     NEXT searchbyauthortitleI0%
2670     RETURN
2680 ' end procedure searchbyauthortitle

2690 ' procedure deleteitem(author$, title$)
2700     ' global header
2710     ' global catalog
2720     ' let h = header[...]  (whole-record read)
2730     GET #1, 1
2740     deleteitemHSize0% = CVI(headerSizeBuf$)
2750     rtrimSelf0$ = headerReservedBuf$
2760     GOSUB 1000
2770     BCCT39$ = rtrimResult0$
2780     deleteitemHReserved0$ = BCCT39$
2790     deleteitemI0% = 1
2800     deleteitemStop0% = 0
2810     IF (deleteitemStop0% = 0) = 0 THEN GOTO 3050
2820         deleteitemI0% = deleteitemI0% + 1
2830         ' let e = catalog[...]  (whole-record read)
2840         GET #2, deleteitemI0%
2850         rtrimSelf0$ = catalogAuthorBuf$
2860         GOSUB 1000
2870         BCCT41$ = rtrimResult0$
2880         deleteitemEAuthor0$ = BCCT41$
2890         rtrimSelf0$ = catalogTitleBuf$
2900         GOSUB 1000
2910         BCCT42$ = rtrimResult0$
2920         deleteitemETitle0$ = BCCT42$
2930         rtrimSelf0$ = catalogSubjectBuf$
2940         GOSUB 1000
2950         BCCT43$ = rtrimResult0$
2960         deleteitemESubject0$ = BCCT43$
2970         IF (deleteitemEAuthor0$ = deleteitemAuthor0$) = 0 THEN GOTO 3000
2980         IF (deleteitemETitle0$ = deleteitemTitle0$) = 0 THEN GOTO 3000
2990             deleteitemStop0% = 1
3000         REM END IF
3010         IF (deleteitemI0% = deleteitemHSize0%) = 0 THEN GOTO 3030
3020             deleteitemStop0% = 1
3030         REM END IF
3040     GOTO 2810
3050     REM END DO
3060     IF (deleteitemEAuthor0$ = deleteitemAuthor0$) = 0 THEN GOTO 3150
3070     IF (deleteitemETitle0$ = deleteitemTitle0$) = 0 THEN GOTO 3150
3080         PRINT (("Deleting: " + deleteitemEAuthor0$) + "  |  ") + deleteitemETitle0$
3090         ' catalog[...] = { ... }  (whole-record write)
3100         LSET catalogAuthorBuf$ = ""
3110         LSET catalogTitleBuf$ = ""
3120         LSET catalogSubjectBuf$ = ""
3130         PUT #2, deleteitemI0%
3140     GOTO 3160
3150         PRINT (("Not found: " + deleteitemAuthor0$) + "  |  ") + deleteitemTitle0$
3160     REM END IF
3170     RETURN
3180 ' end procedure deleteitem

3190 ' procedure mainmenu()
3200     mainmenuRunning0% = 1
3210     IF (mainmenuRunning0% = 1) = 0 THEN GOTO 3700
3220         PRINT ""
3230         PRINT "MENU.          1 ) LIST ALL ITEMS"
3240         PRINT "               2 ) NEW ITEM"
3250         PRINT "               3 ) SEARCH BY AUTHOR"
3260         PRINT "               4 ) SEARCH BY AUTHOR + TITLE"
3270         PRINT "               5 ) DELETE ITEM"
3280         PRINT "               6 ) STOP"
3290         PRINT ""
3300         INPUT "CHOICE: "; mainmenuChoice0%
3310         BCCT51% = mainmenuChoice0%
3320         IF (BCCT51% = 1) <> 0 THEN GOTO 3390
3330         IF (BCCT51% = 2) <> 0 THEN GOTO 3410
3340         IF (BCCT51% = 3) <> 0 THEN GOTO 3490
3350         IF (BCCT51% = 4) <> 0 THEN GOTO 3530
3360         IF (BCCT51% = 5) <> 0 THEN GOTO 3590
3370         IF (BCCT51% = 6) <> 0 THEN GOTO 3650
3380         GOTO 3670
3390             GOSUB 1760
3400             GOTO 3680
3410             INPUT "AUTHOR  "; mainmenuAuthor0$
3420             INPUT "TITLE   "; mainmenuTitle0$
3430             INPUT "SUBJECT "; mainmenuSubject0$
3440             additemAuthor0$ = mainmenuAuthor0$
3450             additemTitle0$ = mainmenuTitle0$
3460             additemSubject0$ = mainmenuSubject0$
3470             GOSUB 1290
3480             GOTO 3680
3490             INPUT "AUTHOR "; mainmenuAuthor0$
3500             searchbyauthorAuthor0$ = mainmenuAuthor0$
3510             GOSUB 2070
3520             GOTO 3680
3530             INPUT "AUTHOR "; mainmenuAuthor0$
3540             INPUT "TITLE  "; mainmenuTitle0$
3550             searchbyauthortitleAuthor0$ = mainmenuAuthor0$
3560             searchbyauthortitleTitle0$ = mainmenuTitle0$
3570             GOSUB 2380
3580             GOTO 3680
3590             INPUT "AUTHOR (to delete) "; mainmenuAuthor0$
3600             INPUT "TITLE  (to delete) "; mainmenuTitle0$
3610             deleteitemAuthor0$ = mainmenuAuthor0$
3620             deleteitemTitle0$ = mainmenuTitle0$
3630             GOSUB 2700
3640             GOTO 3680
3650             mainmenuRunning0% = 0
3660             GOTO 3680
3670             PRINT "Invalid choice"
3680         REM END SELECT
3690     GOTO 3210
3700     REM END DO
3710     RETURN
3720 ' end procedure mainmenu
