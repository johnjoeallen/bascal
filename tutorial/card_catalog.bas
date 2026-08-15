10 ' BASCAL generated BASIC
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Card Catalog — a flagship example for the record/file DSL + procedures
40 ' 
50 ' Adapted from CLERK.BAS, a menu-driven card-catalog manager written by
60 ' Carlos A. Lujan S. in February 1983 as an improved version of Alfred
70 ' Fant's LIBRARIAN program (Microcomputing, December 1982). The original
80 ' source lives in the PeatSoft GW-FILES collection, in the
90 ' robhagemans/hoard-of-gwbasic archive on GitHub
100 ' (PeatSoft/GWFILES/CLERK.BAS).
110 ' 
120 ' What's carried over from CLERK.BAS:
130 ' - One random-access file holding a header record (the catalog's
140 ' capacity) in slot 1, followed by author/title/subject entry records
150 ' in the remaining slots.
160 ' - NEW ITEM: linear-scan the entries for the first empty slot.
170 ' - Searches by author, and by author + title together.
180 ' - DELETE ITEM: linear-scan for the first author+title match, blank it.
190 ' 
200 ' What's adapted rather than ported line-for-line:
210 ' - The menu is still interactive (INPUT-driven, like CLERK.BAS's own
220 ' INKEY$/ON CHOICE GOSUB loop), but each menu action (NEW ITEM, list,
230 ' the two searches, DELETE ITEM) is its own `procedure` — addItem,
240 ' listAll, searchByAuthor, searchByAuthorTitle, deleteItem — called
250 ' from a `mainMenu` dispatch procedure using `select case`, instead of
260 ' CLERK.BAS's numbered GOTO/GOSUB sections. This is the canonical
270 ' BASCAL style (see MANUAL.md's Procedures section), and specifically
280 ' exercises record/file access from inside a procedure body, not just
290 ' top-level code.
300 ' - CLERK.BAS's original also supported a multi-diskette/multi-file
310 ' registry (drive letter + FILEDAT), search-by-subject, and HARD COPY
320 ' (LPRINT) output. This example keeps one catalog file and the two
330 ' named searches, and drops the rest, to stay focused on what the
340 ' record/file DSL and procedures are actually demonstrating here.

350 ' The header occupies slot 1 of the same file, sized to match Entry's
360 ' width (20+20+20 = 60 bytes) so both record types agree on where every
370 ' slot starts. size is the last valid entry slot number, mirroring
380 ' CLERK.BAS's own S = CVI(F$) header field.

390 lastslot% = 11

400 ' file header as Header = open(...)  [60 bytes/record]
410 OPEN "catalog.dat" FOR RANDOM AS #1 LEN = 60
420 FIELD #1, 2 AS headerSizeBuf$, 58 AS headerReservedBuf$
430 ' file catalog as Entry = open(...)  [60 bytes/record]
440 OPEN "catalog.dat" FOR RANDOM AS #2 LEN = 60
450 FIELD #2, 20 AS catalogAuthorBuf$, 20 AS catalogTitleBuf$, 20 AS catalogSubjectBuf$

460 ' ---- CHOICE=5 in CLERK.BAS: create/reset the catalog file ----

470 ' ---- CHOICE=1 NEW ITEM in CLERK.BAS ----
480 ' author$  -- new entry's author
490 ' title$   -- new entry's title
500 ' subject$ -- new entry's subject

510 ' ---- MENU=1 subroutine in CLERK.BAS: list every non-empty entry ----

520 ' ---- MENU=2 subroutine in CLERK.BAS: filter by author ----
530 ' author$ -- author name to match

540 ' ---- MENU=3 subroutine in CLERK.BAS: filter by author AND title ----
550 ' author$ -- author name to match
560 ' title$  -- title to match

570 ' ---- CHOICE=3 DELETE ITEM in CLERK.BAS: first author+title match ----
580 ' author$ -- author name to match
590 ' title$  -- title to match

600 ' ---- CLERK.BAS's own MENU / ON CHOICE GOSUB dispatch loop ----

610 ' --- Drive the catalog ---

620 GOSUB 700
630 GOSUB 3250

640 ' header.close()
650 CLOSE #1
660 ' catalog.close()
670 CLOSE #2

680 END

690 ' procedure initcatalog()
700     ' header[...] = { ... }  (whole-record write)
710     LSET headerSizeBuf$ = MKI$(lastslot%)
720     LSET headerReservedBuf$ = ""
730     PUT #1, 1
740     FOR initcatalogI0% = 2 TO lastslot%
750         ' catalog[...] = { ... }  (whole-record write)
760         LSET catalogAuthorBuf$ = ""
770         LSET catalogTitleBuf$ = ""
780         LSET catalogSubjectBuf$ = ""
790         PUT #2, initcatalogI0%
800     NEXT initcatalogI0%
810     RETURN
820 ' end procedure initcatalog

830 ' procedure additem(author$, title$, subject$)
840     ' let h = header[...]  (whole-record read)
850     GET #1, 1
860     additemHSize0% = CVI(headerSizeBuf$)
870     additemHReservedTrimI0% = LEN(headerReservedBuf$)
880     IF (additemHReservedTrimI0% > 0) = 0 THEN GOTO 920
890     IF (MID$(headerReservedBuf$, additemHReservedTrimI0%, 1) = " ") = 0 THEN GOTO 920
900         additemHReservedTrimI0% = additemHReservedTrimI0% - 1
910         GOTO 880
920     REM END WHILE
930     additemHReserved0$ = LEFT$(headerReservedBuf$, additemHReservedTrimI0%)
940     additemI0% = 1
950     additemStop0% = 0
960     IF (additemStop0% = 0) = 0 THEN GOTO 1280
970         additemI0% = additemI0% + 1
980         ' let e = catalog[...]  (whole-record read)
990         GET #2, additemI0%
1000         additemEAuthorTrimI0% = LEN(catalogAuthorBuf$)
1010         IF (additemEAuthorTrimI0% > 0) = 0 THEN GOTO 1050
1020         IF (MID$(catalogAuthorBuf$, additemEAuthorTrimI0%, 1) = " ") = 0 THEN GOTO 1050
1030             additemEAuthorTrimI0% = additemEAuthorTrimI0% - 1
1040             GOTO 1010
1050         REM END WHILE
1060         additemEAuthor0$ = LEFT$(catalogAuthorBuf$, additemEAuthorTrimI0%)
1070         additemETitleTrimI0% = LEN(catalogTitleBuf$)
1080         IF (additemETitleTrimI0% > 0) = 0 THEN GOTO 1120
1090         IF (MID$(catalogTitleBuf$, additemETitleTrimI0%, 1) = " ") = 0 THEN GOTO 1120
1100             additemETitleTrimI0% = additemETitleTrimI0% - 1
1110             GOTO 1080
1120         REM END WHILE
1130         additemETitle0$ = LEFT$(catalogTitleBuf$, additemETitleTrimI0%)
1140         additemESubjectTrimI0% = LEN(catalogSubjectBuf$)
1150         IF (additemESubjectTrimI0% > 0) = 0 THEN GOTO 1190
1160         IF (MID$(catalogSubjectBuf$, additemESubjectTrimI0%, 1) = " ") = 0 THEN GOTO 1190
1170             additemESubjectTrimI0% = additemESubjectTrimI0% - 1
1180             GOTO 1150
1190         REM END WHILE
1200         additemESubject0$ = LEFT$(catalogSubjectBuf$, additemESubjectTrimI0%)
1210         IF (additemEAuthor0$ = "") = 0 THEN GOTO 1230
1220             additemStop0% = 1
1230         REM END IF
1240         IF (additemI0% = additemHSize0%) = 0 THEN GOTO 1260
1250             additemStop0% = 1
1260         REM END IF
1270         GOTO 960
1280     REM END DO
1290     IF (additemEAuthor0$ = "") = 0 THEN GOTO 1360
1300         ' catalog[...] = { ... }  (whole-record write)
1310         LSET catalogAuthorBuf$ = additemAuthor0$
1320         LSET catalogTitleBuf$ = additemTitle0$
1330         LSET catalogSubjectBuf$ = additemSubject0$
1340         PUT #2, additemI0%
1350         GOTO 1370
1360         PRINT "Catalog is full -- cannot add " + additemAuthor0$
1370     REM END IF
1380     RETURN
1390 ' end procedure additem

1400 ' procedure listall()
1410     ' let h = header[...]  (whole-record read)
1420     GET #1, 1
1430     listallHSize0% = CVI(headerSizeBuf$)
1440     listallHReservedTrimI0% = LEN(headerReservedBuf$)
1450     IF (listallHReservedTrimI0% > 0) = 0 THEN GOTO 1490
1460     IF (MID$(headerReservedBuf$, listallHReservedTrimI0%, 1) = " ") = 0 THEN GOTO 1490
1470         listallHReservedTrimI0% = listallHReservedTrimI0% - 1
1480         GOTO 1450
1490     REM END WHILE
1500     listallHReserved0$ = LEFT$(headerReservedBuf$, listallHReservedTrimI0%)
1510     FOR listallI0% = 2 TO listallHSize0%
1520         ' let e = catalog[...]  (whole-record read)
1530         GET #2, listallI0%
1540         listallEAuthorTrimI0% = LEN(catalogAuthorBuf$)
1550         IF (listallEAuthorTrimI0% > 0) = 0 THEN GOTO 1590
1560         IF (MID$(catalogAuthorBuf$, listallEAuthorTrimI0%, 1) = " ") = 0 THEN GOTO 1590
1570             listallEAuthorTrimI0% = listallEAuthorTrimI0% - 1
1580             GOTO 1550
1590         REM END WHILE
1600         listallEAuthor0$ = LEFT$(catalogAuthorBuf$, listallEAuthorTrimI0%)
1610         listallETitleTrimI0% = LEN(catalogTitleBuf$)
1620         IF (listallETitleTrimI0% > 0) = 0 THEN GOTO 1660
1630         IF (MID$(catalogTitleBuf$, listallETitleTrimI0%, 1) = " ") = 0 THEN GOTO 1660
1640             listallETitleTrimI0% = listallETitleTrimI0% - 1
1650             GOTO 1620
1660         REM END WHILE
1670         listallETitle0$ = LEFT$(catalogTitleBuf$, listallETitleTrimI0%)
1680         listallESubjectTrimI0% = LEN(catalogSubjectBuf$)
1690         IF (listallESubjectTrimI0% > 0) = 0 THEN GOTO 1730
1700         IF (MID$(catalogSubjectBuf$, listallESubjectTrimI0%, 1) = " ") = 0 THEN GOTO 1730
1710             listallESubjectTrimI0% = listallESubjectTrimI0% - 1
1720             GOTO 1690
1730         REM END WHILE
1740         listallESubject0$ = LEFT$(catalogSubjectBuf$, listallESubjectTrimI0%)
1750         IF (listallEAuthor0$ <> "") = 0 THEN GOTO 1770
1760             PRINT (((listallEAuthor0$ + "  |  ") + listallETitle0$) + "  |  ") + listallESubject0$
1770         REM END IF
1780     NEXT listallI0%
1790     RETURN
1800 ' end procedure listall

1810 ' procedure searchbyauthor(author$)
1820     ' let h = header[...]  (whole-record read)
1830     GET #1, 1
1840     searchbyauthorHSize0% = CVI(headerSizeBuf$)
1850     searchbyauthorHReservedTrimI0% = LEN(headerReservedBuf$)
1860     IF (searchbyauthorHReservedTrimI0% > 0) = 0 THEN GOTO 1900
1870     IF (MID$(headerReservedBuf$, searchbyauthorHReservedTrimI0%, 1) = " ") = 0 THEN GOTO 1900
1880         searchbyauthorHReservedTrimI0% = searchbyauthorHReservedTrimI0% - 1
1890         GOTO 1860
1900     REM END WHILE
1910     searchbyauthorHReserved0$ = LEFT$(headerReservedBuf$, searchbyauthorHReservedTrimI0%)
1920     FOR searchbyauthorI0% = 2 TO searchbyauthorHSize0%
1930         ' let e = catalog[...]  (whole-record read)
1940         GET #2, searchbyauthorI0%
1950         searchbyauthorEAuthorTrimI0% = LEN(catalogAuthorBuf$)
1960         IF (searchbyauthorEAuthorTrimI0% > 0) = 0 THEN GOTO 2000
1970         IF (MID$(catalogAuthorBuf$, searchbyauthorEAuthorTrimI0%, 1) = " ") = 0 THEN GOTO 2000
1980             searchbyauthorEAuthorTrimI0% = searchbyauthorEAuthorTrimI0% - 1
1990             GOTO 1960
2000         REM END WHILE
2010         searchbyauthorEAuthor0$ = LEFT$(catalogAuthorBuf$, searchbyauthorEAuthorTrimI0%)
2020         searchbyauthorETitleTrimI0% = LEN(catalogTitleBuf$)
2030         IF (searchbyauthorETitleTrimI0% > 0) = 0 THEN GOTO 2070
2040         IF (MID$(catalogTitleBuf$, searchbyauthorETitleTrimI0%, 1) = " ") = 0 THEN GOTO 2070
2050             searchbyauthorETitleTrimI0% = searchbyauthorETitleTrimI0% - 1
2060             GOTO 2030
2070         REM END WHILE
2080         searchbyauthorETitle0$ = LEFT$(catalogTitleBuf$, searchbyauthorETitleTrimI0%)
2090         searchbyauthorESubjectTrimI0% = LEN(catalogSubjectBuf$)
2100         IF (searchbyauthorESubjectTrimI0% > 0) = 0 THEN GOTO 2140
2110         IF (MID$(catalogSubjectBuf$, searchbyauthorESubjectTrimI0%, 1) = " ") = 0 THEN GOTO 2140
2120             searchbyauthorESubjectTrimI0% = searchbyauthorESubjectTrimI0% - 1
2130             GOTO 2100
2140         REM END WHILE
2150         searchbyauthorESubject0$ = LEFT$(catalogSubjectBuf$, searchbyauthorESubjectTrimI0%)
2160         IF (searchbyauthorEAuthor0$ = searchbyauthorAuthor0$) = 0 THEN GOTO 2180
2170             PRINT (((searchbyauthorEAuthor0$ + "  |  ") + searchbyauthorETitle0$) + "  |  ") + searchbyauthorESubject0$
2180         REM END IF
2190     NEXT searchbyauthorI0%
2200     RETURN
2210 ' end procedure searchbyauthor

2220 ' procedure searchbyauthortitle(author$, title$)
2230     ' let h = header[...]  (whole-record read)
2240     GET #1, 1
2250     searchbyauthortitleHSize0% = CVI(headerSizeBuf$)
2260     searchbyauthortitleHReservedTrimI0% = LEN(headerReservedBuf$)
2270     IF (searchbyauthortitleHReservedTrimI0% > 0) = 0 THEN GOTO 2310
2280     IF (MID$(headerReservedBuf$, searchbyauthortitleHReservedTrimI0%, 1) = " ") = 0 THEN GOTO 2310
2290         searchbyauthortitleHReservedTrimI0% = searchbyauthortitleHReservedTrimI0% - 1
2300         GOTO 2270
2310     REM END WHILE
2320     searchbyauthortitleHReserved0$ = LEFT$(headerReservedBuf$, searchbyauthortitleHReservedTrimI0%)
2330     FOR searchbyauthortitleI0% = 2 TO searchbyauthortitleHSize0%
2340         ' let e = catalog[...]  (whole-record read)
2350         GET #2, searchbyauthortitleI0%
2360         searchbyauthortitleEAuthorTrimI0% = LEN(catalogAuthorBuf$)
2370         IF (searchbyauthortitleEAuthorTrimI0% > 0) = 0 THEN GOTO 2410
2380         IF (MID$(catalogAuthorBuf$, searchbyauthortitleEAuthorTrimI0%, 1) = " ") = 0 THEN GOTO 2410
2390             searchbyauthortitleEAuthorTrimI0% = searchbyauthortitleEAuthorTrimI0% - 1
2400             GOTO 2370
2410         REM END WHILE
2420         searchbyauthortitleEAuthor0$ = LEFT$(catalogAuthorBuf$, searchbyauthortitleEAuthorTrimI0%)
2430         searchbyauthortitleETitleTrimI0% = LEN(catalogTitleBuf$)
2440         IF (searchbyauthortitleETitleTrimI0% > 0) = 0 THEN GOTO 2480
2450         IF (MID$(catalogTitleBuf$, searchbyauthortitleETitleTrimI0%, 1) = " ") = 0 THEN GOTO 2480
2460             searchbyauthortitleETitleTrimI0% = searchbyauthortitleETitleTrimI0% - 1
2470             GOTO 2440
2480         REM END WHILE
2490         searchbyauthortitleETitle0$ = LEFT$(catalogTitleBuf$, searchbyauthortitleETitleTrimI0%)
2500         searchbyauthortitleESubjectTrimI0% = LEN(catalogSubjectBuf$)
2510         IF (searchbyauthortitleESubjectTrimI0% > 0) = 0 THEN GOTO 2550
2520         IF (MID$(catalogSubjectBuf$, searchbyauthortitleESubjectTrimI0%, 1) = " ") = 0 THEN GOTO 2550
2530             searchbyauthortitleESubjectTrimI0% = searchbyauthortitleESubjectTrimI0% - 1
2540             GOTO 2510
2550         REM END WHILE
2560         searchbyauthortitleESubject0$ = LEFT$(catalogSubjectBuf$, searchbyauthortitleESubjectTrimI0%)
2570         IF (searchbyauthortitleEAuthor0$ = searchbyauthortitleAuthor0$) = 0 THEN GOTO 2600
2580         IF (searchbyauthortitleETitle0$ = searchbyauthortitleTitle0$) = 0 THEN GOTO 2600
2590             PRINT (((searchbyauthortitleEAuthor0$ + "  |  ") + searchbyauthortitleETitle0$) + "  |  ") + searchbyauthortitleESubject0$
2600         REM END IF
2610     NEXT searchbyauthortitleI0%
2620     RETURN
2630 ' end procedure searchbyauthortitle

2640 ' procedure deleteitem(author$, title$)
2650     ' let h = header[...]  (whole-record read)
2660     GET #1, 1
2670     deleteitemHSize0% = CVI(headerSizeBuf$)
2680     deleteitemHReservedTrimI0% = LEN(headerReservedBuf$)
2690     IF (deleteitemHReservedTrimI0% > 0) = 0 THEN GOTO 2730
2700     IF (MID$(headerReservedBuf$, deleteitemHReservedTrimI0%, 1) = " ") = 0 THEN GOTO 2730
2710         deleteitemHReservedTrimI0% = deleteitemHReservedTrimI0% - 1
2720         GOTO 2690
2730     REM END WHILE
2740     deleteitemHReserved0$ = LEFT$(headerReservedBuf$, deleteitemHReservedTrimI0%)
2750     deleteitemI0% = 1
2760     deleteitemStop0% = 0
2770     IF (deleteitemStop0% = 0) = 0 THEN GOTO 3100
2780         deleteitemI0% = deleteitemI0% + 1
2790         ' let e = catalog[...]  (whole-record read)
2800         GET #2, deleteitemI0%
2810         deleteitemEAuthorTrimI0% = LEN(catalogAuthorBuf$)
2820         IF (deleteitemEAuthorTrimI0% > 0) = 0 THEN GOTO 2860
2830         IF (MID$(catalogAuthorBuf$, deleteitemEAuthorTrimI0%, 1) = " ") = 0 THEN GOTO 2860
2840             deleteitemEAuthorTrimI0% = deleteitemEAuthorTrimI0% - 1
2850             GOTO 2820
2860         REM END WHILE
2870         deleteitemEAuthor0$ = LEFT$(catalogAuthorBuf$, deleteitemEAuthorTrimI0%)
2880         deleteitemETitleTrimI0% = LEN(catalogTitleBuf$)
2890         IF (deleteitemETitleTrimI0% > 0) = 0 THEN GOTO 2930
2900         IF (MID$(catalogTitleBuf$, deleteitemETitleTrimI0%, 1) = " ") = 0 THEN GOTO 2930
2910             deleteitemETitleTrimI0% = deleteitemETitleTrimI0% - 1
2920             GOTO 2890
2930         REM END WHILE
2940         deleteitemETitle0$ = LEFT$(catalogTitleBuf$, deleteitemETitleTrimI0%)
2950         deleteitemESubjectTrimI0% = LEN(catalogSubjectBuf$)
2960         IF (deleteitemESubjectTrimI0% > 0) = 0 THEN GOTO 3000
2970         IF (MID$(catalogSubjectBuf$, deleteitemESubjectTrimI0%, 1) = " ") = 0 THEN GOTO 3000
2980             deleteitemESubjectTrimI0% = deleteitemESubjectTrimI0% - 1
2990             GOTO 2960
3000         REM END WHILE
3010         deleteitemESubject0$ = LEFT$(catalogSubjectBuf$, deleteitemESubjectTrimI0%)
3020         IF (deleteitemEAuthor0$ = deleteitemAuthor0$) = 0 THEN GOTO 3050
3030         IF (deleteitemETitle0$ = deleteitemTitle0$) = 0 THEN GOTO 3050
3040             deleteitemStop0% = 1
3050         REM END IF
3060         IF (deleteitemI0% = deleteitemHSize0%) = 0 THEN GOTO 3080
3070             deleteitemStop0% = 1
3080         REM END IF
3090         GOTO 2770
3100     REM END DO
3110     IF (deleteitemEAuthor0$ = deleteitemAuthor0$) = 0 THEN GOTO 3200
3120     IF (deleteitemETitle0$ = deleteitemTitle0$) = 0 THEN GOTO 3200
3130         PRINT (("Deleting: " + deleteitemEAuthor0$) + "  |  ") + deleteitemETitle0$
3140         ' catalog[...] = { ... }  (whole-record write)
3150         LSET catalogAuthorBuf$ = ""
3160         LSET catalogTitleBuf$ = ""
3170         LSET catalogSubjectBuf$ = ""
3180         PUT #2, deleteitemI0%
3190         GOTO 3210
3200         PRINT (("Not found: " + deleteitemAuthor0$) + "  |  ") + deleteitemTitle0$
3210     REM END IF
3220     RETURN
3230 ' end procedure deleteitem

3240 ' procedure mainmenu()
3250     mainmenuRunning0% = 1
3260     IF (mainmenuRunning0% = 1) = 0 THEN GOTO 3750
3270         PRINT ""
3280         PRINT "MENU.          1 ) LIST ALL ITEMS"
3290         PRINT "               2 ) NEW ITEM"
3300         PRINT "               3 ) SEARCH BY AUTHOR"
3310         PRINT "               4 ) SEARCH BY AUTHOR + TITLE"
3320         PRINT "               5 ) DELETE ITEM"
3330         PRINT "               6 ) STOP"
3340         PRINT ""
3350         INPUT "CHOICE: "; mainmenuChoice0%

3360         BCCT34% = mainmenuChoice0%
3370         IF (BCCT34% = 1) <> 0 THEN GOTO 3440
3380         IF (BCCT34% = 2) <> 0 THEN GOTO 3460
3390         IF (BCCT34% = 3) <> 0 THEN GOTO 3540
3400         IF (BCCT34% = 4) <> 0 THEN GOTO 3580
3410         IF (BCCT34% = 5) <> 0 THEN GOTO 3640
3420         IF (BCCT34% = 6) <> 0 THEN GOTO 3700
3430         GOTO 3720
3440             GOSUB 1410
3450             GOTO 3730
3460             INPUT "AUTHOR  "; mainmenuAuthor0$
3470             INPUT "TITLE   "; mainmenuTitle0$
3480             INPUT "SUBJECT "; mainmenuSubject0$
3490             additemAuthor0$ = mainmenuAuthor0$
3500             additemTitle0$ = mainmenuTitle0$
3510             additemSubject0$ = mainmenuSubject0$
3520             GOSUB 840
3530             GOTO 3730
3540             INPUT "AUTHOR "; mainmenuAuthor0$
3550             searchbyauthorAuthor0$ = mainmenuAuthor0$
3560             GOSUB 1820
3570             GOTO 3730
3580             INPUT "AUTHOR "; mainmenuAuthor0$
3590             INPUT "TITLE  "; mainmenuTitle0$
3600             searchbyauthortitleAuthor0$ = mainmenuAuthor0$
3610             searchbyauthortitleTitle0$ = mainmenuTitle0$
3620             GOSUB 2230
3630             GOTO 3730
3640             INPUT "AUTHOR (to delete) "; mainmenuAuthor0$
3650             INPUT "TITLE  (to delete) "; mainmenuTitle0$
3660             deleteitemAuthor0$ = mainmenuAuthor0$
3670             deleteitemTitle0$ = mainmenuTitle0$
3680             GOSUB 2650
3690             GOTO 3730
3700             mainmenuRunning0% = 0
3710             GOTO 3730
3720             PRINT "Invalid choice"
3730         REM END SELECT
3740         GOTO 3260
3750     REM END DO
3760     RETURN
3770 ' end procedure mainmenu
