' BASCAL generated BASIC
' Functions are lowered to global variables, labels, and GOSUB
' Simple integer function example.
' Function parameters lower to global variables named from the function.
' The compiler lowers this call to parameter assignments, GOSUB, and result copy.
add_left% = 10
add_right% = 20
GOSUB 10
total% = add_result%
PRINT "10 + 20 ="
PRINT total%
END
' ===== BEGIN FUNCTION add% =====
10     ' Return is explicit; BASCAL does not synthesize implicit function results.
    add_result% = add_left% + add_right%
    RETURN
' ===== END FUNCTION add% =====
