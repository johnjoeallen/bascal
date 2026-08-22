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
270 ' BASCAL style (see the manual's Procedures section at
280 ' https://johnjoeallen.github.io/bascal/manual/), and specifically
290 ' exercises record/file access from inside a procedure body, not just
300 ' top-level code.
310 ' - CLERK.BAS's original also supported a multi-diskette/multi-file
320 ' registry (drive letter + FILEDAT), search-by-subject, and HARD COPY
330 ' (LPRINT) output. This example keeps one catalog file and the two
340 ' named searches, and drops the rest, to stay focused on what the
350 ' record/file DSL and procedures are actually demonstrating here.

360 ' The header occupies slot 1 of the same file, sized to match Entry's
370 ' width (20+20+20 = 60 bytes) so both record types agree on where every
380 ' slot starts. size is the last valid entry slot number, mirroring
390 ' CLERK.BAS's own S = CVI(F$) header field.

400 lastslot% = 11

410 ' file header as Header = open(...)  [60 bytes/record]
420 OPEN "catalog.dat" FOR RANDOM AS #1 LEN = 60
430 FIELD #1, 2 AS headerSizeBuf$, 58 AS headerReservedBuf$
440 ' file catalog as Entry = open(...)  [60 bytes/record]
450 OPEN "catalog.dat" FOR RANDOM AS #2 LEN = 60
460 FIELD #2, 20 AS catalogAuthorBuf$, 20 AS catalogTitleBuf$, 20 AS catalogSubjectBuf$

470 ' ---- CHOICE=5 in CLERK.BAS: create/reset the catalog file ----

480 ' ---- CHOICE=1 NEW ITEM in CLERK.BAS ----
490 ' author$  -- new entry's author
500 ' title$   -- new entry's title
510 ' subject$ -- new entry's subject

520 ' ---- MENU=1 subroutine in CLERK.BAS: list every non-empty entry ----

530 ' ---- MENU=2 subroutine in CLERK.BAS: filter by author ----
540 ' author$ -- author name to match

550 ' ---- MENU=3 subroutine in CLERK.BAS: filter by author AND title ----
560 ' author$ -- author name to match
570 ' title$  -- title to match

580 ' ---- CHOICE=3 DELETE ITEM in CLERK.BAS: first author+title match ----
590 ' author$ -- author name to match
600 ' title$  -- title to match

610 ' ---- CLERK.BAS's own MENU / ON CHOICE GOSUB dispatch loop ----

620 ' --- Drive the catalog ---

630 GOSUB 710
640 GOSUB 3260

650 ' header.close()
660 CLOSE #1
670 ' catalog.close()
680 CLOSE #2

690 END

700 ' procedure initcatalog()
710     ' header[...] = { ... }  (whole-record write)
720     LSET headerSizeBuf$ = MKI$(lastslot%)
730     LSET headerReservedBuf$ = ""
740     PUT #1, 1
750     FOR initcatalogI0% = 2 TO lastslot%
760         ' catalog[...] = { ... }  (whole-record write)
770         LSET catalogAuthorBuf$ = ""
780         LSET catalogTitleBuf$ = ""
790         LSET catalogSubjectBuf$ = ""
800         PUT #2, initcatalogI0%
810     NEXT initcatalogI0%
820     RETURN
830 ' end procedure initcatalog

840 ' procedure additem(author$, title$, subject$)
850     ' let h = header[...]  (whole-record read)
860     GET #1, 1
870     additemHSize0% = CVI(headerSizeBuf$)
880     additemHReservedTrimI0% = LEN(headerReservedBuf$)
890     IF (additemHReservedTrimI0% > 0) = 0 THEN GOTO 930
900     IF (MID$(headerReservedBuf$, additemHReservedTrimI0%, 1) = " ") = 0 THEN GOTO 930
910         additemHReservedTrimI0% = additemHReservedTrimI0% - 1
920         GOTO 890
930     REM END WHILE
940     additemHReserved0$ = LEFT$(headerReservedBuf$, additemHReservedTrimI0%)
950     additemI0% = 1
960     additemStop0% = 0
970     IF (additemStop0% = 0) = 0 THEN GOTO 1290
980         additemI0% = additemI0% + 1
990         ' let e = catalog[...]  (whole-record read)
1000         GET #2, additemI0%
1010         additemEAuthorTrimI0% = LEN(catalogAuthorBuf$)
1020         IF (additemEAuthorTrimI0% > 0) = 0 THEN GOTO 1060
1030         IF (MID$(catalogAuthorBuf$, additemEAuthorTrimI0%, 1) = " ") = 0 THEN GOTO 1060
1040             additemEAuthorTrimI0% = additemEAuthorTrimI0% - 1
1050             GOTO 1020
1060         REM END WHILE
1070         additemEAuthor0$ = LEFT$(catalogAuthorBuf$, additemEAuthorTrimI0%)
1080         additemETitleTrimI0% = LEN(catalogTitleBuf$)
1090         IF (additemETitleTrimI0% > 0) = 0 THEN GOTO 1130
1100         IF (MID$(catalogTitleBuf$, additemETitleTrimI0%, 1) = " ") = 0 THEN GOTO 1130
1110             additemETitleTrimI0% = additemETitleTrimI0% - 1
1120             GOTO 1090
1130         REM END WHILE
1140         additemETitle0$ = LEFT$(catalogTitleBuf$, additemETitleTrimI0%)
1150         additemESubjectTrimI0% = LEN(catalogSubjectBuf$)
1160         IF (additemESubjectTrimI0% > 0) = 0 THEN GOTO 1200
1170         IF (MID$(catalogSubjectBuf$, additemESubjectTrimI0%, 1) = " ") = 0 THEN GOTO 1200
1180             additemESubjectTrimI0% = additemESubjectTrimI0% - 1
1190             GOTO 1160
1200         REM END WHILE
1210         additemESubject0$ = LEFT$(catalogSubjectBuf$, additemESubjectTrimI0%)
1220         IF (additemEAuthor0$ = "") = 0 THEN GOTO 1240
1230             additemStop0% = 1
1240         REM END IF
1250         IF (additemI0% = additemHSize0%) = 0 THEN GOTO 1270
1260             additemStop0% = 1
1270         REM END IF
1280         GOTO 970
1290     REM END DO
1300     IF (additemEAuthor0$ = "") = 0 THEN GOTO 1370
1310         ' catalog[...] = { ... }  (whole-record write)
1320         LSET catalogAuthorBuf$ = additemAuthor0$
1330         LSET catalogTitleBuf$ = additemTitle0$
1340         LSET catalogSubjectBuf$ = additemSubject0$
1350         PUT #2, additemI0%
1360         GOTO 1380
1370         PRINT "Catalog is full -- cannot add " + additemAuthor0$
1380     REM END IF
1390     RETURN
1400 ' end procedure additem

1410 ' procedure listall()
1420     ' let h = header[...]  (whole-record read)
1430     GET #1, 1
1440     listallHSize0% = CVI(headerSizeBuf$)
1450     listallHReservedTrimI0% = LEN(headerReservedBuf$)
1460     IF (listallHReservedTrimI0% > 0) = 0 THEN GOTO 1500
1470     IF (MID$(headerReservedBuf$, listallHReservedTrimI0%, 1) = " ") = 0 THEN GOTO 1500
1480         listallHReservedTrimI0% = listallHReservedTrimI0% - 1
1490         GOTO 1460
1500     REM END WHILE
1510     listallHReserved0$ = LEFT$(headerReservedBuf$, listallHReservedTrimI0%)
1520     FOR listallI0% = 2 TO listallHSize0%
1530         ' let e = catalog[...]  (whole-record read)
1540         GET #2, listallI0%
1550         listallEAuthorTrimI0% = LEN(catalogAuthorBuf$)
1560         IF (listallEAuthorTrimI0% > 0) = 0 THEN GOTO 1600
1570         IF (MID$(catalogAuthorBuf$, listallEAuthorTrimI0%, 1) = " ") = 0 THEN GOTO 1600
1580             listallEAuthorTrimI0% = listallEAuthorTrimI0% - 1
1590             GOTO 1560
1600         REM END WHILE
1610         listallEAuthor0$ = LEFT$(catalogAuthorBuf$, listallEAuthorTrimI0%)
1620         listallETitleTrimI0% = LEN(catalogTitleBuf$)
1630         IF (listallETitleTrimI0% > 0) = 0 THEN GOTO 1670
1640         IF (MID$(catalogTitleBuf$, listallETitleTrimI0%, 1) = " ") = 0 THEN GOTO 1670
1650             listallETitleTrimI0% = listallETitleTrimI0% - 1
1660             GOTO 1630
1670         REM END WHILE
1680         listallETitle0$ = LEFT$(catalogTitleBuf$, listallETitleTrimI0%)
1690         listallESubjectTrimI0% = LEN(catalogSubjectBuf$)
1700         IF (listallESubjectTrimI0% > 0) = 0 THEN GOTO 1740
1710         IF (MID$(catalogSubjectBuf$, listallESubjectTrimI0%, 1) = " ") = 0 THEN GOTO 1740
1720             listallESubjectTrimI0% = listallESubjectTrimI0% - 1
1730             GOTO 1700
1740         REM END WHILE
1750         listallESubject0$ = LEFT$(catalogSubjectBuf$, listallESubjectTrimI0%)
1760         IF (listallEAuthor0$ <> "") = 0 THEN GOTO 1780
1770             PRINT (((listallEAuthor0$ + "  |  ") + listallETitle0$) + "  |  ") + listallESubject0$
1780         REM END IF
1790     NEXT listallI0%
1800     RETURN
1810 ' end procedure listall

1820 ' procedure searchbyauthor(author$)
1830     ' let h = header[...]  (whole-record read)
1840     GET #1, 1
1850     searchbyauthorHSize0% = CVI(headerSizeBuf$)
1860     searchbyauthorHReservedTrimI0% = LEN(headerReservedBuf$)
1870     IF (searchbyauthorHReservedTrimI0% > 0) = 0 THEN GOTO 1910
1880     IF (MID$(headerReservedBuf$, searchbyauthorHReservedTrimI0%, 1) = " ") = 0 THEN GOTO 1910
1890         searchbyauthorHReservedTrimI0% = searchbyauthorHReservedTrimI0% - 1
1900         GOTO 1870
1910     REM END WHILE
1920     searchbyauthorHReserved0$ = LEFT$(headerReservedBuf$, searchbyauthorHReservedTrimI0%)
1930     FOR searchbyauthorI0% = 2 TO searchbyauthorHSize0%
1940         ' let e = catalog[...]  (whole-record read)
1950         GET #2, searchbyauthorI0%
1960         searchbyauthorEAuthorTrimI0% = LEN(catalogAuthorBuf$)
1970         IF (searchbyauthorEAuthorTrimI0% > 0) = 0 THEN GOTO 2010
1980         IF (MID$(catalogAuthorBuf$, searchbyauthorEAuthorTrimI0%, 1) = " ") = 0 THEN GOTO 2010
1990             searchbyauthorEAuthorTrimI0% = searchbyauthorEAuthorTrimI0% - 1
2000             GOTO 1970
2010         REM END WHILE
2020         searchbyauthorEAuthor0$ = LEFT$(catalogAuthorBuf$, searchbyauthorEAuthorTrimI0%)
2030         searchbyauthorETitleTrimI0% = LEN(catalogTitleBuf$)
2040         IF (searchbyauthorETitleTrimI0% > 0) = 0 THEN GOTO 2080
2050         IF (MID$(catalogTitleBuf$, searchbyauthorETitleTrimI0%, 1) = " ") = 0 THEN GOTO 2080
2060             searchbyauthorETitleTrimI0% = searchbyauthorETitleTrimI0% - 1
2070             GOTO 2040
2080         REM END WHILE
2090         searchbyauthorETitle0$ = LEFT$(catalogTitleBuf$, searchbyauthorETitleTrimI0%)
2100         searchbyauthorESubjectTrimI0% = LEN(catalogSubjectBuf$)
2110         IF (searchbyauthorESubjectTrimI0% > 0) = 0 THEN GOTO 2150
2120         IF (MID$(catalogSubjectBuf$, searchbyauthorESubjectTrimI0%, 1) = " ") = 0 THEN GOTO 2150
2130             searchbyauthorESubjectTrimI0% = searchbyauthorESubjectTrimI0% - 1
2140             GOTO 2110
2150         REM END WHILE
2160         searchbyauthorESubject0$ = LEFT$(catalogSubjectBuf$, searchbyauthorESubjectTrimI0%)
2170         IF (searchbyauthorEAuthor0$ = searchbyauthorAuthor0$) = 0 THEN GOTO 2190
2180             PRINT (((searchbyauthorEAuthor0$ + "  |  ") + searchbyauthorETitle0$) + "  |  ") + searchbyauthorESubject0$
2190         REM END IF
2200     NEXT searchbyauthorI0%
2210     RETURN
2220 ' end procedure searchbyauthor

2230 ' procedure searchbyauthortitle(author$, title$)
2240     ' let h = header[...]  (whole-record read)
2250     GET #1, 1
2260     searchbyauthortitleHSize0% = CVI(headerSizeBuf$)
2270     searchbyauthortitleHReservedTrimI0% = LEN(headerReservedBuf$)
2280     IF (searchbyauthortitleHReservedTrimI0% > 0) = 0 THEN GOTO 2320
2290     IF (MID$(headerReservedBuf$, searchbyauthortitleHReservedTrimI0%, 1) = " ") = 0 THEN GOTO 2320
2300         searchbyauthortitleHReservedTrimI0% = searchbyauthortitleHReservedTrimI0% - 1
2310         GOTO 2280
2320     REM END WHILE
2330     searchbyauthortitleHReserved0$ = LEFT$(headerReservedBuf$, searchbyauthortitleHReservedTrimI0%)
2340     FOR searchbyauthortitleI0% = 2 TO searchbyauthortitleHSize0%
2350         ' let e = catalog[...]  (whole-record read)
2360         GET #2, searchbyauthortitleI0%
2370         searchbyauthortitleEAuthorTrimI0% = LEN(catalogAuthorBuf$)
2380         IF (searchbyauthortitleEAuthorTrimI0% > 0) = 0 THEN GOTO 2420
2390         IF (MID$(catalogAuthorBuf$, searchbyauthortitleEAuthorTrimI0%, 1) = " ") = 0 THEN GOTO 2420
2400             searchbyauthortitleEAuthorTrimI0% = searchbyauthortitleEAuthorTrimI0% - 1
2410             GOTO 2380
2420         REM END WHILE
2430         searchbyauthortitleEAuthor0$ = LEFT$(catalogAuthorBuf$, searchbyauthortitleEAuthorTrimI0%)
2440         searchbyauthortitleETitleTrimI0% = LEN(catalogTitleBuf$)
2450         IF (searchbyauthortitleETitleTrimI0% > 0) = 0 THEN GOTO 2490
2460         IF (MID$(catalogTitleBuf$, searchbyauthortitleETitleTrimI0%, 1) = " ") = 0 THEN GOTO 2490
2470             searchbyauthortitleETitleTrimI0% = searchbyauthortitleETitleTrimI0% - 1
2480             GOTO 2450
2490         REM END WHILE
2500         searchbyauthortitleETitle0$ = LEFT$(catalogTitleBuf$, searchbyauthortitleETitleTrimI0%)
2510         searchbyauthortitleESubjectTrimI0% = LEN(catalogSubjectBuf$)
2520         IF (searchbyauthortitleESubjectTrimI0% > 0) = 0 THEN GOTO 2560
2530         IF (MID$(catalogSubjectBuf$, searchbyauthortitleESubjectTrimI0%, 1) = " ") = 0 THEN GOTO 2560
2540             searchbyauthortitleESubjectTrimI0% = searchbyauthortitleESubjectTrimI0% - 1
2550             GOTO 2520
2560         REM END WHILE
2570         searchbyauthortitleESubject0$ = LEFT$(catalogSubjectBuf$, searchbyauthortitleESubjectTrimI0%)
2580         IF (searchbyauthortitleEAuthor0$ = searchbyauthortitleAuthor0$) = 0 THEN GOTO 2610
2590         IF (searchbyauthortitleETitle0$ = searchbyauthortitleTitle0$) = 0 THEN GOTO 2610
2600             PRINT (((searchbyauthortitleEAuthor0$ + "  |  ") + searchbyauthortitleETitle0$) + "  |  ") + searchbyauthortitleESubject0$
2610         REM END IF
2620     NEXT searchbyauthortitleI0%
2630     RETURN
2640 ' end procedure searchbyauthortitle

2650 ' procedure deleteitem(author$, title$)
2660     ' let h = header[...]  (whole-record read)
2670     GET #1, 1
2680     deleteitemHSize0% = CVI(headerSizeBuf$)
2690     deleteitemHReservedTrimI0% = LEN(headerReservedBuf$)
2700     IF (deleteitemHReservedTrimI0% > 0) = 0 THEN GOTO 2740
2710     IF (MID$(headerReservedBuf$, deleteitemHReservedTrimI0%, 1) = " ") = 0 THEN GOTO 2740
2720         deleteitemHReservedTrimI0% = deleteitemHReservedTrimI0% - 1
2730         GOTO 2700
2740     REM END WHILE
2750     deleteitemHReserved0$ = LEFT$(headerReservedBuf$, deleteitemHReservedTrimI0%)
2760     deleteitemI0% = 1
2770     deleteitemStop0% = 0
2780     IF (deleteitemStop0% = 0) = 0 THEN GOTO 3110
2790         deleteitemI0% = deleteitemI0% + 1
2800         ' let e = catalog[...]  (whole-record read)
2810         GET #2, deleteitemI0%
2820         deleteitemEAuthorTrimI0% = LEN(catalogAuthorBuf$)
2830         IF (deleteitemEAuthorTrimI0% > 0) = 0 THEN GOTO 2870
2840         IF (MID$(catalogAuthorBuf$, deleteitemEAuthorTrimI0%, 1) = " ") = 0 THEN GOTO 2870
2850             deleteitemEAuthorTrimI0% = deleteitemEAuthorTrimI0% - 1
2860             GOTO 2830
2870         REM END WHILE
2880         deleteitemEAuthor0$ = LEFT$(catalogAuthorBuf$, deleteitemEAuthorTrimI0%)
2890         deleteitemETitleTrimI0% = LEN(catalogTitleBuf$)
2900         IF (deleteitemETitleTrimI0% > 0) = 0 THEN GOTO 2940
2910         IF (MID$(catalogTitleBuf$, deleteitemETitleTrimI0%, 1) = " ") = 0 THEN GOTO 2940
2920             deleteitemETitleTrimI0% = deleteitemETitleTrimI0% - 1
2930             GOTO 2900
2940         REM END WHILE
2950         deleteitemETitle0$ = LEFT$(catalogTitleBuf$, deleteitemETitleTrimI0%)
2960         deleteitemESubjectTrimI0% = LEN(catalogSubjectBuf$)
2970         IF (deleteitemESubjectTrimI0% > 0) = 0 THEN GOTO 3010
2980         IF (MID$(catalogSubjectBuf$, deleteitemESubjectTrimI0%, 1) = " ") = 0 THEN GOTO 3010
2990             deleteitemESubjectTrimI0% = deleteitemESubjectTrimI0% - 1
3000             GOTO 2970
3010         REM END WHILE
3020         deleteitemESubject0$ = LEFT$(catalogSubjectBuf$, deleteitemESubjectTrimI0%)
3030         IF (deleteitemEAuthor0$ = deleteitemAuthor0$) = 0 THEN GOTO 3060
3040         IF (deleteitemETitle0$ = deleteitemTitle0$) = 0 THEN GOTO 3060
3050             deleteitemStop0% = 1
3060         REM END IF
3070         IF (deleteitemI0% = deleteitemHSize0%) = 0 THEN GOTO 3090
3080             deleteitemStop0% = 1
3090         REM END IF
3100         GOTO 2780
3110     REM END DO
3120     IF (deleteitemEAuthor0$ = deleteitemAuthor0$) = 0 THEN GOTO 3210
3130     IF (deleteitemETitle0$ = deleteitemTitle0$) = 0 THEN GOTO 3210
3140         PRINT (("Deleting: " + deleteitemEAuthor0$) + "  |  ") + deleteitemETitle0$
3150         ' catalog[...] = { ... }  (whole-record write)
3160         LSET catalogAuthorBuf$ = ""
3170         LSET catalogTitleBuf$ = ""
3180         LSET catalogSubjectBuf$ = ""
3190         PUT #2, deleteitemI0%
3200         GOTO 3220
3210         PRINT (("Not found: " + deleteitemAuthor0$) + "  |  ") + deleteitemTitle0$
3220     REM END IF
3230     RETURN
3240 ' end procedure deleteitem

3250 ' procedure mainmenu()
3260     mainmenuRunning0% = 1
3270     IF (mainmenuRunning0% = 1) = 0 THEN GOTO 3760
3280         PRINT ""
3290         PRINT "MENU.          1 ) LIST ALL ITEMS"
3300         PRINT "               2 ) NEW ITEM"
3310         PRINT "               3 ) SEARCH BY AUTHOR"
3320         PRINT "               4 ) SEARCH BY AUTHOR + TITLE"
3330         PRINT "               5 ) DELETE ITEM"
3340         PRINT "               6 ) STOP"
3350         PRINT ""
3360         INPUT "CHOICE: "; mainmenuChoice0%

3370         BCCT34% = mainmenuChoice0%
3380         IF (BCCT34% = 1) <> 0 THEN GOTO 3450
3390         IF (BCCT34% = 2) <> 0 THEN GOTO 3470
3400         IF (BCCT34% = 3) <> 0 THEN GOTO 3550
3410         IF (BCCT34% = 4) <> 0 THEN GOTO 3590
3420         IF (BCCT34% = 5) <> 0 THEN GOTO 3650
3430         IF (BCCT34% = 6) <> 0 THEN GOTO 3710
3440         GOTO 3730
3450             GOSUB 1420
3460             GOTO 3740
3470             INPUT "AUTHOR  "; mainmenuAuthor0$
3480             INPUT "TITLE   "; mainmenuTitle0$
3490             INPUT "SUBJECT "; mainmenuSubject0$
3500             additemAuthor0$ = mainmenuAuthor0$
3510             additemTitle0$ = mainmenuTitle0$
3520             additemSubject0$ = mainmenuSubject0$
3530             GOSUB 850
3540             GOTO 3740
3550             INPUT "AUTHOR "; mainmenuAuthor0$
3560             searchbyauthorAuthor0$ = mainmenuAuthor0$
3570             GOSUB 1830
3580             GOTO 3740
3590             INPUT "AUTHOR "; mainmenuAuthor0$
3600             INPUT "TITLE  "; mainmenuTitle0$
3610             searchbyauthortitleAuthor0$ = mainmenuAuthor0$
3620             searchbyauthortitleTitle0$ = mainmenuTitle0$
3630             GOSUB 2240
3640             GOTO 3740
3650             INPUT "AUTHOR (to delete) "; mainmenuAuthor0$
3660             INPUT "TITLE  (to delete) "; mainmenuTitle0$
3670             deleteitemAuthor0$ = mainmenuAuthor0$
3680             deleteitemTitle0$ = mainmenuTitle0$
3690             GOSUB 2660
3700             GOTO 3740
3710             mainmenuRunning0% = 0
3720             GOTO 3740
3730             PRINT "Invalid choice"
3740         REM END SELECT
3750         GOTO 3270
3760     REM END DO
3770     RETURN
3780 ' end procedure mainmenu
