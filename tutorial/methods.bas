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
270 surroundLeft0$ = "["
280 surroundRight0$ = "]"
290 GOSUB 1360
300 result$ = surroundResult0$
310 PRINT result$
320 shoutSelf0$ = name$
330 GOSUB 1280
340 shoutresult$ = shoutResult0$
350 length% = LEN(LEFT$(name$, 5))
360 PRINT "length = "; length%

370 score% = 125
380 clampSelf0% = score%
390 clampLow0% = 0
400 clampHigh0% = 100
410 GOSUB 1420
420 PRINT "clamped score = "; clampResult0%

430 price! = 80
440 percentSelf0! = price!
450 percentRate0! = 15
460 GOSUB 1570
470 PRINT "discount amount = "; percentResult0!

480 ucaseSelf0$ = LEFT$(name$, 3)
490 GOSUB 1130
500 firstthree$ = ucaseResult0$
510 PRINT "first three = "; firstthree$

520 ' -------------------- Record methods --------------------

530 ' Declared inline: the enclosing record supplies the receiver type, so
540 ' there is no `[Card]` bracket here at all.

550 ' Declared externally: same receiver, same callable identity as an
560 ' inline method -- an external method just lets behavior be attached to
570 ' a record without editing its own declaration.

580 cardtitle$ = "Dune"
590 cardauthor$ = "Frank Herbert"
600 cardcopies& = 2
610 carddisplaySelfTitle0$ = cardtitle$
620 carddisplaySelfAuthor0$ = cardauthor$
630 carddisplaySelfCopies0& = cardcopies&
640 GOSUB 1710
650 cardtitle$ = carddisplaySelfTitle0$
660 cardauthor$ = carddisplaySelfAuthor0$
670 cardcopies& = carddisplaySelfCopies0&
680 PRINT carddisplayResult0$
690 PRINT "copies on hand = "; cardcopies&

700 cardrestockSelfTitle0$ = cardtitle$
710 cardrestockSelfAuthor0$ = cardauthor$
720 cardrestockSelfCopies0& = cardcopies&
730 cardrestockAmount0% = 3
740 GOSUB 1630
750 cardtitle$ = cardrestockSelfTitle0$
760 cardauthor$ = cardrestockSelfAuthor0$
770 cardcopies& = cardrestockSelfCopies0&
780 PRINT "copies after restock = "; cardcopies&

790 ' `combines` is structural field composition only, not inheritance:
800 ' SignedCard's own field list is Card's fields (title, author, copies)
810 ' followed by its own (signature), so its record literal accepts all
820 ' four -- but Card's methods aren't combined in. SignedCard needs its
830 ' own display() (below); calling signed.restock(...) without declaring
840 ' SignedCard's own restock() would be a compile error, since a method
850 ' is only ever visible for the exact record type it was declared for.

860 signedtitle$ = "Dune"
870 signedauthor$ = "Frank Herbert"
880 signedcopies& = 1
890 signedsignature$ = "F.H."
900 signedcarddisplaySelfTitle0$ = signedtitle$
910 signedcarddisplaySelfAuthor0$ = signedauthor$
920 signedcarddisplaySelfCopies0& = signedcopies&
930 signedcarddisplaySelfSignature0$ = signedsignature$
940 GOSUB 1750
950 signedtitle$ = signedcarddisplaySelfTitle0$
960 signedauthor$ = signedcarddisplaySelfAuthor0$
970 signedcopies& = signedcarddisplaySelfCopies0&
980 signedsignature$ = signedcarddisplaySelfSignature0$
990 PRINT signedcarddisplayResult0$
1000 signedcardrestockSelfTitle0$ = signedtitle$
1010 signedcardrestockSelfAuthor0$ = signedauthor$
1020 signedcardrestockSelfCopies0& = signedcopies&
1030 signedcardrestockSelfSignature0$ = signedsignature$
1040 signedcardrestockAmount0% = 2
1050 GOSUB 1670
1060 signedtitle$ = signedcardrestockSelfTitle0$
1070 signedauthor$ = signedcardrestockSelfAuthor0$
1080 signedcopies& = signedcardrestockSelfCopies0&
1090 signedsignature$ = signedcardrestockSelfSignature0$
1100 PRINT "signed copies after restock = "; signedcopies&

1110 END

1120 ' function ucase$()
1130     ucaseOut0$ = ""
1140     FOR ucaseI0% = 1 TO LEN(ucaseSelf0$)
1150         ucaseC0% = ASC(MID$(ucaseSelf0$, ucaseI0%, 1))
1160         IF (ucaseC0% >= 97) = 0 THEN GOTO 1190
1170         IF (ucaseC0% <= 122) = 0 THEN GOTO 1190
1180             ucaseC0% = ucaseC0% - 32
1190         REM END IF
1200         ucaseOut0$ = ucaseOut0$ + CHR$(ucaseC0%)
1210     NEXT ucaseI0%
1220     ucaseResult0$ = ucaseOut0$
1230     RETURN
1240     ucaseResult0$ = ucaseSelf0$
1250     RETURN
1260 ' end function ucase$

1270 ' function shout$()
1280     ucaseSelf0$ = shoutSelf0$
1290     GOSUB 1130
1300     shoutResult0$ = ucaseResult0$ + "!"
1310     RETURN
1320     shoutResult0$ = shoutSelf0$
1330     RETURN
1340 ' end function shout$

1350 ' function surround$(left$, right$)
1360     surroundResult0$ = (surroundLeft0$ + surroundSelf0$) + surroundRight0$
1370     RETURN
1380     surroundResult0$ = surroundSelf0$
1390     RETURN
1400 ' end function surround$

1410 ' function clamp%(low%, high%)
1420     IF (clampSelf0% < clampLow0%) = 0 THEN GOTO 1460
1430         clampResult0% = clampLow0%
1440         RETURN
1450         GOTO 1500
1460         IF (clampSelf0% > clampHigh0%) = 0 THEN GOTO 1490
1470             clampResult0% = clampHigh0%
1480             RETURN
1490         REM END IF
1500     REM END IF
1510     clampResult0% = clampSelf0%
1520     RETURN
1530     clampResult0% = clampSelf0%
1540     RETURN
1550 ' end function clamp%

1560 ' function percent!(rate!)
1570     percentResult0! = (percentSelf0! * percentRate0!) / 100
1580     RETURN
1590     percentResult0! = percentSelf0!
1600     RETURN
1610 ' end function percent!

1620 ' procedure cardrestock(selftitle$, selfauthor$, selfcopies&, amount%)
1630     cardrestockSelfCopies0& = cardrestockSelfCopies0& + cardrestockAmount0%
1640     RETURN
1650 ' end procedure cardrestock

1660 ' procedure signedcardrestock(selftitle$, selfauthor$, selfcopies&, selfsignature$, amount%)
1670     signedcardrestockSelfCopies0& = signedcardrestockSelfCopies0& + signedcardrestockAmount0%
1680     RETURN
1690 ' end procedure signedcardrestock

1700 ' function carddisplay$(selftitle$, selfauthor$, selfcopies&)
1710     carddisplayResult0$ = (carddisplaySelfTitle0$ + " by ") + carddisplaySelfAuthor0$
1720     RETURN
1730 ' end function carddisplay$

1740 ' function signedcarddisplay$(selftitle$, selfauthor$, selfcopies&, selfsignature$)
1750     signedcarddisplayResult0$ = (((signedcarddisplaySelfTitle0$ + " by ") + signedcarddisplaySelfAuthor0$) + ", signed ") + signedcarddisplaySelfSignature0$
1760     RETURN
1770 ' end function signedcarddisplay$
