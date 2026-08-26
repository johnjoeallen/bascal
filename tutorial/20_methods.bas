10 ' BASCAL generated BASIC -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Upper-cases self$. Not a real MBASIC/BASCOM 2.00 builtin -- verified
40 ' against a real IBM BASIC Compiler 2.00 under dosbox-x -- so BASCAL ships
50 ' its own. Declared as a scalar method (see GitHub issue #41 and
60 ' ltrim.bcl's own doc comment for the reasoning) -- ucase$(s$) still works
70 ' via ordinary-call syntax resolving to this same declaration.

80 ' Tutorial — Scalar methods
90 ' 
100 ' A method declares its scalar receiver and result types in brackets after
110 ' its name. The receiver is available as self%/self!/self$
120 ' in the body. Dot calls can chain when each result has the next receiver's
130 ' type. Methods transpile to ordinary typed calls for both backends.

140 name$ = "bascal"
150 surroundSelf0$ = name$
160 surroundLeft0$ = "["
170 surroundRight0$ = "]"
180 GOSUB 620
190 result$ = surroundResult0$
200 PRINT result$
210 shoutSelf0$ = name$
220 GOSUB 560
230 shoutresult$ = shoutResult0$
240 length% = LEN(LEFT$(name$, 5))
250 PRINT "length = "; length%

260 score% = 125
270 clampSelf0% = score%
280 clampLow0% = 0
290 clampHigh0% = 100
300 GOSUB 660
310 PRINT "clamped score = "; clampResult0%

320 price! = 80
330 percentSelf0! = price!
340 percentRate0! = 15
350 GOSUB 790
360 PRINT "discount amount = "; percentResult0!

370 ucaseSelf0$ = LEFT$(name$, 3)
380 GOSUB 430
390 firstthree$ = ucaseResult0$
400 PRINT "first three = "; firstthree$

410 END

420 ' function ucase$()
430     ucaseOut0$ = ""
440     FOR ucaseI0% = 1 TO LEN(ucaseSelf0$)
450         ucaseC0% = ASC(MID$(ucaseSelf0$, ucaseI0%, 1))
460         IF (ucaseC0% >= 97) = 0 THEN GOTO 490
470         IF (ucaseC0% <= 122) = 0 THEN GOTO 490
480             ucaseC0% = ucaseC0% - 32
490         REM END IF
500         ucaseOut0$ = ucaseOut0$ + CHR$(ucaseC0%)
510     NEXT ucaseI0%
520     ucaseResult0$ = ucaseOut0$
530     RETURN
540 ' end function ucase$

550 ' function shout$()
560     ucaseSelf0$ = shoutSelf0$
570     GOSUB 430
580     shoutResult0$ = ucaseResult0$ + "!"
590     RETURN
600 ' end function shout$

610 ' function surround$(left$, right$)
620     surroundResult0$ = (surroundLeft0$ + surroundSelf0$) + surroundRight0$
630     RETURN
640 ' end function surround$

650 ' function clamp%(low%, high%)
660     IF (clampSelf0% < clampLow0%) = 0 THEN GOTO 700
670         clampResult0% = clampLow0%
680         RETURN
690         GOTO 740
700         IF (clampSelf0% > clampHigh0%) = 0 THEN GOTO 730
710             clampResult0% = clampHigh0%
720             RETURN
730         REM END IF
740     REM END IF
750     clampResult0% = clampSelf0%
760     RETURN
770 ' end function clamp%

780 ' function percent!(rate!)
790     percentResult0! = (percentSelf0! * percentRate0!) / 100
800     RETURN
810 ' end function percent!
