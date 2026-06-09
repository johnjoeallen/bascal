' BASCAL generated BASIC
' Functions are lowered to global variables, labels, and GOSUB
' Repeated function result assignment.
' Each call writes x_result$, and each assignment copies that result immediately.
GOSUB 10
a$ = x_result$
GOSUB 10
b$ = x_result$
PRINT a$
PRINT b$
END
' ===== BEGIN FUNCTION x$ =====
10     x_result$ = "result"
    RETURN
' ===== END FUNCTION x$ =====
