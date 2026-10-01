10 ' BASCAL generated BASIC -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Upper-cases self$. Not a real MBASIC/BASCOM 2.00 builtin -- verified
40 ' against a real IBM BASIC Compiler 2.00 under dosbox-x -- so BASCAL ships
50 ' its own. Declared as a scalar method (see GitHub issue #41 and
60 ' ltrim.bcl's own doc comment for the reasoning) -- ucase$(s$) still works
70 ' via ordinary-call syntax resolving to this same declaration.
80 ' Tutorial — Methods
90 ' 
100 ' A method is a statically resolved callable with an implicit receiver,
110 ' written in brackets after the method name: `method shout[string]()`
120 ' receives a string. The return type follows the parameter list after
130 ' `:` (or its suffix shorthand, `: $`); omitting it for a scalar receiver
140 ' makes the method return the receiver's own type, falling through to an
150 ' implicit `return self`. Dot calls chain when each result has the next
160 ' receiver's type. Methods transpile to ordinary typed calls for every
170 ' backend -- there is no runtime method object, virtual dispatch, or
180 ' vtable of any kind.
190 ' 
200 ' A record type is a valid receiver too, declared either externally (in
210 ' brackets, same as a scalar receiver) or inline, directly inside the
220 ' record itself. `self.field` is then ordinary field access against the
230 ' receiver, and mutating it is visible to the caller once the call
240 ' returns -- the receiver is passed by reference, not by copy.
250 name$ = "bascal"
260 surroundSelf0$ = name$
270 BCCT1$ = "["
280 surroundLeft0$ = BCCT1$
290 BCCT2$ = "]"
300 surroundRight0$ = BCCT2$
310 GOSUB 1500
320 BCCT3$ = surroundResult0$
330 result$ = BCCT3$
340 PRINT result$
350 shoutSelf0$ = name$
360 GOSUB 1430
370 BCCT4$ = shoutResult0$
380 shoutresult$ = BCCT4$
390 length% = LEN(LEFT$(name$, 5))
400 PRINT "length = "; length%
410 score% = 125
420 BCCT5$ = "clamped score = "
430 clampSelf0% = score%
440 BCCT6% = 0
450 clampLow0% = BCCT6%
460 BCCT7% = 100
470 clampHigh0% = BCCT7%
480 GOSUB 1540
490 BCCT8% = clampResult0%
500 BCCT9% = BCCT8%
510 PRINT BCCT5$; BCCT9%
520 price! = 80
530 BCCT10$ = "discount amount = "
540 percentSelf0! = price!
550 BCCT11! = 15
560 percentRate0! = BCCT11!
570 GOSUB 1670
580 BCCT12! = percentResult0!
590 BCCT13! = BCCT12!
600 PRINT BCCT10$; BCCT13!
610 BCCT14$ = name$
620 BCCT15% = 3
630 BCCT16$ = LEFT$(BCCT14$, BCCT15%)
640 ucaseSelf0$ = BCCT16$
650 GOSUB 1300
660 BCCT17$ = ucaseResult0$
670 firstthree$ = BCCT17$
680 PRINT "first three = "; firstthree$
690 ' -------------------- Record methods --------------------
700 ' Declared inline: the enclosing record supplies the receiver type, so
710 ' there is no `[Card]` bracket here at all.
720 ' Declared externally: same receiver, same callable identity as an
730 ' inline method -- an external method just lets behavior be attached to
740 ' a record without editing its own declaration.
750 cardtitle$ = "Dune"
760 cardauthor$ = "Frank Herbert"
770 cardcopies& = 2
780 carddisplaySelfTitle0$ = cardtitle$
790 carddisplaySelfAuthor0$ = cardauthor$
800 carddisplaySelfCopies0& = cardcopies&
810 GOSUB 1790
820 cardtitle$ = carddisplaySelfTitle0$
830 cardauthor$ = carddisplaySelfAuthor0$
840 cardcopies& = carddisplaySelfCopies0&
850 PRINT carddisplayResult0$
860 PRINT "copies on hand = "; cardcopies&
870 cardrestockSelfTitle0$ = cardtitle$
880 cardrestockSelfAuthor0$ = cardauthor$
890 cardrestockSelfCopies0& = cardcopies&
900 cardrestockAmount0% = 3
910 GOSUB 1710
920 cardtitle$ = cardrestockSelfTitle0$
930 cardauthor$ = cardrestockSelfAuthor0$
940 cardcopies& = cardrestockSelfCopies0&
950 PRINT "copies after restock = "; cardcopies&
960 ' `combines` is structural field composition only, not inheritance:
970 ' SignedCard's own field list is Card's fields (title, author, copies)
980 ' followed by its own (signature), so its record literal accepts all
990 ' four -- but Card's methods aren't combined in. SignedCard needs its
1000 ' own display() (below); calling signed.restock(...) without declaring
1010 ' SignedCard's own restock() would be a compile error, since a method
1020 ' is only ever visible for the exact record type it was declared for.
1030 signedtitle$ = "Dune"
1040 signedauthor$ = "Frank Herbert"
1050 signedcopies& = 1
1060 signedsignature$ = "F.H."
1070 signedcarddisplaySelfTitle0$ = signedtitle$
1080 signedcarddisplaySelfAuthor0$ = signedauthor$
1090 signedcarddisplaySelfCopies0& = signedcopies&
1100 signedcarddisplaySelfSignature0$ = signedsignature$
1110 GOSUB 1830
1120 signedtitle$ = signedcarddisplaySelfTitle0$
1130 signedauthor$ = signedcarddisplaySelfAuthor0$
1140 signedcopies& = signedcarddisplaySelfCopies0&
1150 signedsignature$ = signedcarddisplaySelfSignature0$
1160 PRINT signedcarddisplayResult0$
1170 signedcardrestockSelfTitle0$ = signedtitle$
1180 signedcardrestockSelfAuthor0$ = signedauthor$
1190 signedcardrestockSelfCopies0& = signedcopies&
1200 signedcardrestockSelfSignature0$ = signedsignature$
1210 signedcardrestockAmount0% = 2
1220 GOSUB 1750
1230 signedtitle$ = signedcardrestockSelfTitle0$
1240 signedauthor$ = signedcardrestockSelfAuthor0$
1250 signedcopies& = signedcardrestockSelfCopies0&
1260 signedsignature$ = signedcardrestockSelfSignature0$
1270 PRINT "signed copies after restock = "; signedcopies&
1280 END

1290 ' function ucase$(self$)
1300     ucaseOut0$ = ""
1310     FOR ucaseI0% = 1 TO LEN(ucaseSelf0$)
1320         ucaseC0% = ASC(MID$(ucaseSelf0$, ucaseI0%, 1))
1330         IF (ucaseC0% >= 97) = 0 THEN GOTO 1360
1340         IF (ucaseC0% <= 122) = 0 THEN GOTO 1360
1350             ucaseC0% = ucaseC0% - 32
1360         REM END IF
1370         ucaseOut0$ = ucaseOut0$ + CHR$(ucaseC0%)
1380     NEXT ucaseI0%
1390     ucaseResult0$ = ucaseOut0$
1400     RETURN
1410 ' end function ucase$

1420 ' function shout$(self$)
1430     ucaseSelf0$ = shoutSelf0$
1440     GOSUB 1300
1450     BCCT21$ = ucaseResult0$
1460     shoutResult0$ = BCCT21$ + "!"
1470     RETURN
1480 ' end function shout$

1490 ' function surround$(self$, left$, right$)
1500     surroundResult0$ = (surroundLeft0$ + surroundSelf0$) + surroundRight0$
1510     RETURN
1520 ' end function surround$

1530 ' function clamp%(self%, low%, high%)
1540     IF (clampSelf0% < clampLow0%) = 0 THEN GOTO 1580
1550         clampResult0% = clampLow0%
1560         RETURN
1570     GOTO 1620
1580         IF (clampSelf0% > clampHigh0%) = 0 THEN GOTO 1610
1590             clampResult0% = clampHigh0%
1600             RETURN
1610         REM END IF
1620     REM END IF
1630     clampResult0% = clampSelf0%
1640     RETURN
1650 ' end function clamp%

1660 ' function percent!(self!, rate!)
1670     percentResult0! = (percentSelf0! * percentRate0!) / 100
1680     RETURN
1690 ' end function percent!

1700 ' procedure cardrestock(selfTitle$, selfAuthor$, selfCopies&, amount%)
1710     cardrestockSelfCopies0& = cardrestockSelfCopies0& + cardrestockAmount0%
1720     RETURN
1730 ' end procedure cardrestock

1740 ' procedure signedcardrestock(selfTitle$, selfAuthor$, selfCopies&, selfSignature$, amount%)
1750     signedcardrestockSelfCopies0& = signedcardrestockSelfCopies0& + signedcardrestockAmount0%
1760     RETURN
1770 ' end procedure signedcardrestock

1780 ' function carddisplay$(selfTitle$, selfAuthor$, selfCopies&)
1790     carddisplayResult0$ = (carddisplaySelfTitle0$ + " by ") + carddisplaySelfAuthor0$
1800     RETURN
1810 ' end function carddisplay$

1820 ' function signedcarddisplay$(selfTitle$, selfAuthor$, selfCopies&, selfSignature$)
1830     signedcarddisplayResult0$ = (((signedcarddisplaySelfTitle0$ + " by ") + signedcarddisplaySelfAuthor0$) + ", signed ") + signedcarddisplaySelfSignature0$
1840     RETURN
1850 ' end function signedcarddisplay$
