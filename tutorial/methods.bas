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
290 GOSUB 1040
300 result$ = surroundResult0$
310 PRINT result$
320 shoutSelf0$ = name$
330 GOSUB 960
340 shoutresult$ = shoutResult0$
350 length% = LEN(LEFT$(name$, 5))
360 PRINT "length = "; length%

370 score% = 125
380 clampSelf0% = score%
390 clampLow0% = 0
400 clampHigh0% = 100
410 GOSUB 1100
420 PRINT "clamped score = "; clampResult0%

430 price! = 80
440 percentSelf0! = price!
450 percentRate0! = 15
460 GOSUB 1250
470 PRINT "discount amount = "; percentResult0!

480 ucaseSelf0$ = LEFT$(name$, 3)
490 GOSUB 810
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
640 GOSUB 1350
650 cardtitle$ = carddisplaySelfTitle0$
660 cardauthor$ = carddisplaySelfAuthor0$
670 cardcopies& = carddisplaySelfCopies0&
680 PRINT carddisplayResult0$
690 PRINT "copies on hand = "; cardcopies&

700 cardrestockSelfTitle0$ = cardtitle$
710 cardrestockSelfAuthor0$ = cardauthor$
720 cardrestockSelfCopies0& = cardcopies&
730 cardrestockAmount0% = 3
740 GOSUB 1310
750 cardtitle$ = cardrestockSelfTitle0$
760 cardauthor$ = cardrestockSelfAuthor0$
770 cardcopies& = cardrestockSelfCopies0&
780 PRINT "copies after restock = "; cardcopies&

790 END

800 ' function ucase$()
810     ucaseOut0$ = ""
820     FOR ucaseI0% = 1 TO LEN(ucaseSelf0$)
830         ucaseC0% = ASC(MID$(ucaseSelf0$, ucaseI0%, 1))
840         IF (ucaseC0% >= 97) = 0 THEN GOTO 870
850         IF (ucaseC0% <= 122) = 0 THEN GOTO 870
860             ucaseC0% = ucaseC0% - 32
870         REM END IF
880         ucaseOut0$ = ucaseOut0$ + CHR$(ucaseC0%)
890     NEXT ucaseI0%
900     ucaseResult0$ = ucaseOut0$
910     RETURN
920     ucaseResult0$ = ucaseSelf0$
930     RETURN
940 ' end function ucase$

950 ' function shout$()
960     ucaseSelf0$ = shoutSelf0$
970     GOSUB 810
980     shoutResult0$ = ucaseResult0$ + "!"
990     RETURN
1000     shoutResult0$ = shoutSelf0$
1010     RETURN
1020 ' end function shout$

1030 ' function surround$(left$, right$)
1040     surroundResult0$ = (surroundLeft0$ + surroundSelf0$) + surroundRight0$
1050     RETURN
1060     surroundResult0$ = surroundSelf0$
1070     RETURN
1080 ' end function surround$

1090 ' function clamp%(low%, high%)
1100     IF (clampSelf0% < clampLow0%) = 0 THEN GOTO 1140
1110         clampResult0% = clampLow0%
1120         RETURN
1130         GOTO 1180
1140         IF (clampSelf0% > clampHigh0%) = 0 THEN GOTO 1170
1150             clampResult0% = clampHigh0%
1160             RETURN
1170         REM END IF
1180     REM END IF
1190     clampResult0% = clampSelf0%
1200     RETURN
1210     clampResult0% = clampSelf0%
1220     RETURN
1230 ' end function clamp%

1240 ' function percent!(rate!)
1250     percentResult0! = (percentSelf0! * percentRate0!) / 100
1260     RETURN
1270     percentResult0! = percentSelf0!
1280     RETURN
1290 ' end function percent!

1300 ' procedure cardrestock(selftitle$, selfauthor$, selfcopies&, amount%)
1310     cardrestockSelfCopies0& = cardrestockSelfCopies0& + cardrestockAmount0%
1320     RETURN
1330 ' end procedure cardrestock

1340 ' function carddisplay$(selftitle$, selfauthor$, selfcopies&)
1350     carddisplayResult0$ = (carddisplaySelfTitle0$ + " by ") + carddisplaySelfAuthor0$
1360     RETURN
1370 ' end function carddisplay$
