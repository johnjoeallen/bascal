' BASCAL generated BASIC
' Functions are lowered to global variables, labels, and GOSUB

' String function example using BASIC's $ suffix convention.

' The function result is copied from fullname_result$ after GOSUB.
fullname_first$ = "Ada"
fullname_last$ = "Lovelace"
GOSUB 10
name$ = fullname_result$
PRINT name$
END

' function fullName$(first$, last$)
10 ' String concatenation is preserved as BASIC +.
    fullname_result$ = (fullname_first$ + " ") + fullname_last$
    RETURN
' end function fullName$
