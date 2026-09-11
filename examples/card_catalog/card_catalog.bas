10 ' BASCAL generated BASIC -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
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

400 lastSLOT% = 11

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
640 GOSUB 3380

650 ' header.close()
660 CLOSE #1
670 ' catalog.close()
680 CLOSE #2

690 END

700 ' procedure initcatalog()
710     ' global header
720     ' global catalog
730     ' header[...] = { ... }  (whole-record write)
740     LSET headerSizeBuf$ = MKI$(lastSLOT%)
750     LSET headerReservedBuf$ = ""
760     PUT #1, 1
770     FOR initcatalogI0% = 2 TO lastSLOT%
780         ' catalog[...] = { ... }  (whole-record write)
790         LSET catalogAuthorBuf$ = ""
800         LSET catalogTitleBuf$ = ""
810         LSET catalogSubjectBuf$ = ""
820         PUT #2, initcatalogI0%
830     NEXT initcatalogI0%
840     RETURN
850 ' end procedure initcatalog

860 ' procedure additem(author$, title$, subject$)
870     ' global header
880     ' global catalog
890     ' let h = header[...]  (whole-record read)
900     GET #1, 1
910     additemHSize0% = CVI(headerSizeBuf$)
920     additemHReservedTrimI0% = LEN(headerReservedBuf$)
930     IF (additemHReservedTrimI0% > 0) = 0 THEN GOTO 970
940     IF (MID$(headerReservedBuf$, additemHReservedTrimI0%, 1) = " ") = 0 THEN GOTO 970
950         additemHReservedTrimI0% = additemHReservedTrimI0% - 1
960         GOTO 930
970     REM END WHILE
980     additemHReserved0$ = LEFT$(headerReservedBuf$, additemHReservedTrimI0%)
990     additemI0% = 1
1000     additemStop0% = 0
1010     IF (additemStop0% = 0) = 0 THEN GOTO 1330
1020         additemI0% = additemI0% + 1
1030         ' let e = catalog[...]  (whole-record read)
1040         GET #2, additemI0%
1050         additemEAuthorTrimI0% = LEN(catalogAuthorBuf$)
1060         IF (additemEAuthorTrimI0% > 0) = 0 THEN GOTO 1100
1070         IF (MID$(catalogAuthorBuf$, additemEAuthorTrimI0%, 1) = " ") = 0 THEN GOTO 1100
1080             additemEAuthorTrimI0% = additemEAuthorTrimI0% - 1
1090             GOTO 1060
1100         REM END WHILE
1110         additemEAuthor0$ = LEFT$(catalogAuthorBuf$, additemEAuthorTrimI0%)
1120         additemETitleTrimI0% = LEN(catalogTitleBuf$)
1130         IF (additemETitleTrimI0% > 0) = 0 THEN GOTO 1170
1140         IF (MID$(catalogTitleBuf$, additemETitleTrimI0%, 1) = " ") = 0 THEN GOTO 1170
1150             additemETitleTrimI0% = additemETitleTrimI0% - 1
1160             GOTO 1130
1170         REM END WHILE
1180         additemETitle0$ = LEFT$(catalogTitleBuf$, additemETitleTrimI0%)
1190         additemESubjectTrimI0% = LEN(catalogSubjectBuf$)
1200         IF (additemESubjectTrimI0% > 0) = 0 THEN GOTO 1240
1210         IF (MID$(catalogSubjectBuf$, additemESubjectTrimI0%, 1) = " ") = 0 THEN GOTO 1240
1220             additemESubjectTrimI0% = additemESubjectTrimI0% - 1
1230             GOTO 1200
1240         REM END WHILE
1250         additemESubject0$ = LEFT$(catalogSubjectBuf$, additemESubjectTrimI0%)
1260         IF (additemEAuthor0$ = "") = 0 THEN GOTO 1280
1270             additemStop0% = 1
1280         REM END IF
1290         IF (additemI0% = additemHSize0%) = 0 THEN GOTO 1310
1300             additemStop0% = 1
1310         REM END IF
1320         GOTO 1010
1330     REM END DO
1340     IF (additemEAuthor0$ = "") = 0 THEN GOTO 1410
1350         ' catalog[...] = { ... }  (whole-record write)
1360         LSET catalogAuthorBuf$ = additemAuthor0$
1370         LSET catalogTitleBuf$ = additemTitle0$
1380         LSET catalogSubjectBuf$ = additemSubject0$
1390         PUT #2, additemI0%
1400         GOTO 1420
1410         PRINT "Catalog is full -- cannot add " + additemAuthor0$
1420     REM END IF
1430     RETURN
1440 ' end procedure additem

1450 ' procedure listall()
1460     ' global header
1470     ' global catalog
1480     ' let h = header[...]  (whole-record read)
1490     GET #1, 1
1500     listallHSize0% = CVI(headerSizeBuf$)
1510     listallHReservedTrimI0% = LEN(headerReservedBuf$)
1520     IF (listallHReservedTrimI0% > 0) = 0 THEN GOTO 1560
1530     IF (MID$(headerReservedBuf$, listallHReservedTrimI0%, 1) = " ") = 0 THEN GOTO 1560
1540         listallHReservedTrimI0% = listallHReservedTrimI0% - 1
1550         GOTO 1520
1560     REM END WHILE
1570     listallHReserved0$ = LEFT$(headerReservedBuf$, listallHReservedTrimI0%)
1580     FOR listallI0% = 2 TO listallHSize0%
1590         ' let e = catalog[...]  (whole-record read)
1600         GET #2, listallI0%
1610         listallEAuthorTrimI0% = LEN(catalogAuthorBuf$)
1620         IF (listallEAuthorTrimI0% > 0) = 0 THEN GOTO 1660
1630         IF (MID$(catalogAuthorBuf$, listallEAuthorTrimI0%, 1) = " ") = 0 THEN GOTO 1660
1640             listallEAuthorTrimI0% = listallEAuthorTrimI0% - 1
1650             GOTO 1620
1660         REM END WHILE
1670         listallEAuthor0$ = LEFT$(catalogAuthorBuf$, listallEAuthorTrimI0%)
1680         listallETitleTrimI0% = LEN(catalogTitleBuf$)
1690         IF (listallETitleTrimI0% > 0) = 0 THEN GOTO 1730
1700         IF (MID$(catalogTitleBuf$, listallETitleTrimI0%, 1) = " ") = 0 THEN GOTO 1730
1710             listallETitleTrimI0% = listallETitleTrimI0% - 1
1720             GOTO 1690
1730         REM END WHILE
1740         listallETitle0$ = LEFT$(catalogTitleBuf$, listallETitleTrimI0%)
1750         listallESubjectTrimI0% = LEN(catalogSubjectBuf$)
1760         IF (listallESubjectTrimI0% > 0) = 0 THEN GOTO 1800
1770         IF (MID$(catalogSubjectBuf$, listallESubjectTrimI0%, 1) = " ") = 0 THEN GOTO 1800
1780             listallESubjectTrimI0% = listallESubjectTrimI0% - 1
1790             GOTO 1760
1800         REM END WHILE
1810         listallESubject0$ = LEFT$(catalogSubjectBuf$, listallESubjectTrimI0%)
1820         IF (listallEAuthor0$ <> "") = 0 THEN GOTO 1840
1830             PRINT (((listallEAuthor0$ + "  |  ") + listallETitle0$) + "  |  ") + listallESubject0$
1840         REM END IF
1850     NEXT listallI0%
1860     RETURN
1870 ' end procedure listall

1880 ' procedure searchbyauthor(author$)
1890     ' global header
1900     ' global catalog
1910     ' let h = header[...]  (whole-record read)
1920     GET #1, 1
1930     searchbyauthorHSize0% = CVI(headerSizeBuf$)
1940     searchbyauthorHReservedTrimI0% = LEN(headerReservedBuf$)
1950     IF (searchbyauthorHReservedTrimI0% > 0) = 0 THEN GOTO 1990
1960     IF (MID$(headerReservedBuf$, searchbyauthorHReservedTrimI0%, 1) = " ") = 0 THEN GOTO 1990
1970         searchbyauthorHReservedTrimI0% = searchbyauthorHReservedTrimI0% - 1
1980         GOTO 1950
1990     REM END WHILE
2000     searchbyauthorHReserved0$ = LEFT$(headerReservedBuf$, searchbyauthorHReservedTrimI0%)
2010     FOR searchbyauthorI0% = 2 TO searchbyauthorHSize0%
2020         ' let e = catalog[...]  (whole-record read)
2030         GET #2, searchbyauthorI0%
2040         searchbyauthorEAuthorTrimI0% = LEN(catalogAuthorBuf$)
2050         IF (searchbyauthorEAuthorTrimI0% > 0) = 0 THEN GOTO 2090
2060         IF (MID$(catalogAuthorBuf$, searchbyauthorEAuthorTrimI0%, 1) = " ") = 0 THEN GOTO 2090
2070             searchbyauthorEAuthorTrimI0% = searchbyauthorEAuthorTrimI0% - 1
2080             GOTO 2050
2090         REM END WHILE
2100         searchbyauthorEAuthor0$ = LEFT$(catalogAuthorBuf$, searchbyauthorEAuthorTrimI0%)
2110         searchbyauthorETitleTrimI0% = LEN(catalogTitleBuf$)
2120         IF (searchbyauthorETitleTrimI0% > 0) = 0 THEN GOTO 2160
2130         IF (MID$(catalogTitleBuf$, searchbyauthorETitleTrimI0%, 1) = " ") = 0 THEN GOTO 2160
2140             searchbyauthorETitleTrimI0% = searchbyauthorETitleTrimI0% - 1
2150             GOTO 2120
2160         REM END WHILE
2170         searchbyauthorETitle0$ = LEFT$(catalogTitleBuf$, searchbyauthorETitleTrimI0%)
2180         searchbyauthorESubjectTrimI0% = LEN(catalogSubjectBuf$)
2190         IF (searchbyauthorESubjectTrimI0% > 0) = 0 THEN GOTO 2230
2200         IF (MID$(catalogSubjectBuf$, searchbyauthorESubjectTrimI0%, 1) = " ") = 0 THEN GOTO 2230
2210             searchbyauthorESubjectTrimI0% = searchbyauthorESubjectTrimI0% - 1
2220             GOTO 2190
2230         REM END WHILE
2240         searchbyauthorESubject0$ = LEFT$(catalogSubjectBuf$, searchbyauthorESubjectTrimI0%)
2250         IF (searchbyauthorEAuthor0$ = searchbyauthorAuthor0$) = 0 THEN GOTO 2270
2260             PRINT (((searchbyauthorEAuthor0$ + "  |  ") + searchbyauthorETitle0$) + "  |  ") + searchbyauthorESubject0$
2270         REM END IF
2280     NEXT searchbyauthorI0%
2290     RETURN
2300 ' end procedure searchbyauthor

2310 ' procedure searchbyauthortitle(author$, title$)
2320     ' global header
2330     ' global catalog
2340     ' let h = header[...]  (whole-record read)
2350     GET #1, 1
2360     searchbyauthortitleHSize0% = CVI(headerSizeBuf$)
2370     searchbyauthortitleHReservedTrimI0% = LEN(headerReservedBuf$)
2380     IF (searchbyauthortitleHReservedTrimI0% > 0) = 0 THEN GOTO 2420
2390     IF (MID$(headerReservedBuf$, searchbyauthortitleHReservedTrimI0%, 1) = " ") = 0 THEN GOTO 2420
2400         searchbyauthortitleHReservedTrimI0% = searchbyauthortitleHReservedTrimI0% - 1
2410         GOTO 2380
2420     REM END WHILE
2430     searchbyauthortitleHReserved0$ = LEFT$(headerReservedBuf$, searchbyauthortitleHReservedTrimI0%)
2440     FOR searchbyauthortitleI0% = 2 TO searchbyauthortitleHSize0%
2450         ' let e = catalog[...]  (whole-record read)
2460         GET #2, searchbyauthortitleI0%
2470         searchbyauthortitleEAuthorTrimI0% = LEN(catalogAuthorBuf$)
2480         IF (searchbyauthortitleEAuthorTrimI0% > 0) = 0 THEN GOTO 2520
2490         IF (MID$(catalogAuthorBuf$, searchbyauthortitleEAuthorTrimI0%, 1) = " ") = 0 THEN GOTO 2520
2500             searchbyauthortitleEAuthorTrimI0% = searchbyauthortitleEAuthorTrimI0% - 1
2510             GOTO 2480
2520         REM END WHILE
2530         searchbyauthortitleEAuthor0$ = LEFT$(catalogAuthorBuf$, searchbyauthortitleEAuthorTrimI0%)
2540         searchbyauthortitleETitleTrimI0% = LEN(catalogTitleBuf$)
2550         IF (searchbyauthortitleETitleTrimI0% > 0) = 0 THEN GOTO 2590
2560         IF (MID$(catalogTitleBuf$, searchbyauthortitleETitleTrimI0%, 1) = " ") = 0 THEN GOTO 2590
2570             searchbyauthortitleETitleTrimI0% = searchbyauthortitleETitleTrimI0% - 1
2580             GOTO 2550
2590         REM END WHILE
2600         searchbyauthortitleETitle0$ = LEFT$(catalogTitleBuf$, searchbyauthortitleETitleTrimI0%)
2610         searchbyauthortitleESubjectTrimI0% = LEN(catalogSubjectBuf$)
2620         IF (searchbyauthortitleESubjectTrimI0% > 0) = 0 THEN GOTO 2660
2630         IF (MID$(catalogSubjectBuf$, searchbyauthortitleESubjectTrimI0%, 1) = " ") = 0 THEN GOTO 2660
2640             searchbyauthortitleESubjectTrimI0% = searchbyauthortitleESubjectTrimI0% - 1
2650             GOTO 2620
2660         REM END WHILE
2670         searchbyauthortitleESubject0$ = LEFT$(catalogSubjectBuf$, searchbyauthortitleESubjectTrimI0%)
2680         IF (searchbyauthortitleEAuthor0$ = searchbyauthortitleAuthor0$) = 0 THEN GOTO 2710
2690         IF (searchbyauthortitleETitle0$ = searchbyauthortitleTitle0$) = 0 THEN GOTO 2710
2700             PRINT (((searchbyauthortitleEAuthor0$ + "  |  ") + searchbyauthortitleETitle0$) + "  |  ") + searchbyauthortitleESubject0$
2710         REM END IF
2720     NEXT searchbyauthortitleI0%
2730     RETURN
2740 ' end procedure searchbyauthortitle

2750 ' procedure deleteitem(author$, title$)
2760     ' global header
2770     ' global catalog
2780     ' let h = header[...]  (whole-record read)
2790     GET #1, 1
2800     deleteitemHSize0% = CVI(headerSizeBuf$)
2810     deleteitemHReservedTrimI0% = LEN(headerReservedBuf$)
2820     IF (deleteitemHReservedTrimI0% > 0) = 0 THEN GOTO 2860
2830     IF (MID$(headerReservedBuf$, deleteitemHReservedTrimI0%, 1) = " ") = 0 THEN GOTO 2860
2840         deleteitemHReservedTrimI0% = deleteitemHReservedTrimI0% - 1
2850         GOTO 2820
2860     REM END WHILE
2870     deleteitemHReserved0$ = LEFT$(headerReservedBuf$, deleteitemHReservedTrimI0%)
2880     deleteitemI0% = 1
2890     deleteitemStop0% = 0
2900     IF (deleteitemStop0% = 0) = 0 THEN GOTO 3230
2910         deleteitemI0% = deleteitemI0% + 1
2920         ' let e = catalog[...]  (whole-record read)
2930         GET #2, deleteitemI0%
2940         deleteitemEAuthorTrimI0% = LEN(catalogAuthorBuf$)
2950         IF (deleteitemEAuthorTrimI0% > 0) = 0 THEN GOTO 2990
2960         IF (MID$(catalogAuthorBuf$, deleteitemEAuthorTrimI0%, 1) = " ") = 0 THEN GOTO 2990
2970             deleteitemEAuthorTrimI0% = deleteitemEAuthorTrimI0% - 1
2980             GOTO 2950
2990         REM END WHILE
3000         deleteitemEAuthor0$ = LEFT$(catalogAuthorBuf$, deleteitemEAuthorTrimI0%)
3010         deleteitemETitleTrimI0% = LEN(catalogTitleBuf$)
3020         IF (deleteitemETitleTrimI0% > 0) = 0 THEN GOTO 3060
3030         IF (MID$(catalogTitleBuf$, deleteitemETitleTrimI0%, 1) = " ") = 0 THEN GOTO 3060
3040             deleteitemETitleTrimI0% = deleteitemETitleTrimI0% - 1
3050             GOTO 3020
3060         REM END WHILE
3070         deleteitemETitle0$ = LEFT$(catalogTitleBuf$, deleteitemETitleTrimI0%)
3080         deleteitemESubjectTrimI0% = LEN(catalogSubjectBuf$)
3090         IF (deleteitemESubjectTrimI0% > 0) = 0 THEN GOTO 3130
3100         IF (MID$(catalogSubjectBuf$, deleteitemESubjectTrimI0%, 1) = " ") = 0 THEN GOTO 3130
3110             deleteitemESubjectTrimI0% = deleteitemESubjectTrimI0% - 1
3120             GOTO 3090
3130         REM END WHILE
3140         deleteitemESubject0$ = LEFT$(catalogSubjectBuf$, deleteitemESubjectTrimI0%)
3150         IF (deleteitemEAuthor0$ = deleteitemAuthor0$) = 0 THEN GOTO 3180
3160         IF (deleteitemETitle0$ = deleteitemTitle0$) = 0 THEN GOTO 3180
3170             deleteitemStop0% = 1
3180         REM END IF
3190         IF (deleteitemI0% = deleteitemHSize0%) = 0 THEN GOTO 3210
3200             deleteitemStop0% = 1
3210         REM END IF
3220         GOTO 2900
3230     REM END DO
3240     IF (deleteitemEAuthor0$ = deleteitemAuthor0$) = 0 THEN GOTO 3330
3250     IF (deleteitemETitle0$ = deleteitemTitle0$) = 0 THEN GOTO 3330
3260         PRINT (("Deleting: " + deleteitemEAuthor0$) + "  |  ") + deleteitemETitle0$
3270         ' catalog[...] = { ... }  (whole-record write)
3280         LSET catalogAuthorBuf$ = ""
3290         LSET catalogTitleBuf$ = ""
3300         LSET catalogSubjectBuf$ = ""
3310         PUT #2, deleteitemI0%
3320         GOTO 3340
3330         PRINT (("Not found: " + deleteitemAuthor0$) + "  |  ") + deleteitemTitle0$
3340     REM END IF
3350     RETURN
3360 ' end procedure deleteitem

3370 ' procedure mainmenu()
3380     mainmenuRunning0% = 1
3390     IF (mainmenuRunning0% = 1) = 0 THEN GOTO 3880
3400         PRINT ""
3410         PRINT "MENU.          1 ) LIST ALL ITEMS"
3420         PRINT "               2 ) NEW ITEM"
3430         PRINT "               3 ) SEARCH BY AUTHOR"
3440         PRINT "               4 ) SEARCH BY AUTHOR + TITLE"
3450         PRINT "               5 ) DELETE ITEM"
3460         PRINT "               6 ) STOP"
3470         PRINT ""
3480         INPUT "CHOICE: "; mainmenuChoice0%

3490         BCCT34% = mainmenuChoice0%
3500         IF (BCCT34% = 1) <> 0 THEN GOTO 3570
3510         IF (BCCT34% = 2) <> 0 THEN GOTO 3590
3520         IF (BCCT34% = 3) <> 0 THEN GOTO 3670
3530         IF (BCCT34% = 4) <> 0 THEN GOTO 3710
3540         IF (BCCT34% = 5) <> 0 THEN GOTO 3770
3550         IF (BCCT34% = 6) <> 0 THEN GOTO 3830
3560         GOTO 3850
3570             GOSUB 1460
3580             GOTO 3860
3590             INPUT "AUTHOR  "; mainmenuAuthor0$
3600             INPUT "TITLE   "; mainmenuTitle0$
3610             INPUT "SUBJECT "; mainmenuSubject0$
3620             additemAuthor0$ = mainmenuAuthor0$
3630             additemTitle0$ = mainmenuTitle0$
3640             additemSubject0$ = mainmenuSubject0$
3650             GOSUB 870
3660             GOTO 3860
3670             INPUT "AUTHOR "; mainmenuAuthor0$
3680             searchbyauthorAuthor0$ = mainmenuAuthor0$
3690             GOSUB 1890
3700             GOTO 3860
3710             INPUT "AUTHOR "; mainmenuAuthor0$
3720             INPUT "TITLE  "; mainmenuTitle0$
3730             searchbyauthortitleAuthor0$ = mainmenuAuthor0$
3740             searchbyauthortitleTitle0$ = mainmenuTitle0$
3750             GOSUB 2320
3760             GOTO 3860
3770             INPUT "AUTHOR (to delete) "; mainmenuAuthor0$
3780             INPUT "TITLE  (to delete) "; mainmenuTitle0$
3790             deleteitemAuthor0$ = mainmenuAuthor0$
3800             deleteitemTitle0$ = mainmenuTitle0$
3810             GOSUB 2760
3820             GOTO 3860
3830             mainmenuRunning0% = 0
3840             GOTO 3860
3850             PRINT "Invalid choice"
3860         REM END SELECT
3870         GOTO 3390
3880     REM END DO
3890     RETURN
3900 ' end procedure mainmenu
