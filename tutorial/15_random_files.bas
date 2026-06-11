' BASCAL generated BASIC
' Functions are lowered to global variables, labels, and GOSUB

' Tutorial 15 — Random-Access File I/O
' 
' Random-access files store fixed-length records that can be read or written
' in any order without scanning from the beginning.
' 
' open filename$ for random as #n len = recLen%
' Open (or create) a random-access file.  len specifies the record length
' in bytes; every record occupies exactly that many bytes.
' 
' field #n, width1% as var1$, width2% as var2$, ...
' Bind string variables to regions of the file buffer.  The sum of all
' widths must equal the record length.  Only string variables may be used
' in a FIELD statement.
' 
' lset var$ = expr$   — copy into a field buffer, left-justified (padded)
' rset var$ = expr$   — copy into a field buffer, right-justified (padded)
' 
' put #n, recordNumber%   — write the current buffer as record n (1-based)
' get #n, recordNumber%   — read record n into the buffer variables
' 
' seek #n, recordNumber%  — position file pointer (affects next get/put
' when record number is omitted)
' 
' Packing helpers (BASIC builtins):
' mki%(n%)  — pack a 2-byte integer into a 2-character string
' mkl&(n&)  — pack a 4-byte long
' mks!(n!)  — pack a 4-byte single
' mkd#(n#)  — pack an 8-byte double
' cvi%(s$)  — unpack a 2-byte integer from a string
' cvl&(s$)  — unpack a 4-byte long
' cvs!(s$)  — unpack a 4-byte single
' cvd#(s$)  — unpack an 8-byte double

CONST rec_len% = 30
CONST num_recs% = 3
CONST db_file$ = "tutorial_students.dat"

' ---- Write three records ----

OPEN db_file$ FOR RANDOM AS #1 LEN = rec_len%

FIELD #1, 2 AS idbuf$, 20 AS namebuf$, 8 AS scorebuf$

' Record 1: Alice, 95
LSET idbuf$ = MKI%(1)
LSET namebuf$ = "Alice"
LSET scorebuf$ = MKD#(95)
PUT #1, 1

' Record 2: Bob, 54
LSET idbuf$ = MKI%(2)
LSET namebuf$ = "Bob"
LSET scorebuf$ = MKD#(54)
PUT #1, 2

' Record 3: Carol, 78
LSET idbuf$ = MKI%(3)
LSET namebuf$ = "Carol"
LSET scorebuf$ = MKD#(78)
PUT #1, 3

CLOSE #1

' ---- Read records in reverse order ----

PRINT "Reading records in reverse order:"
OPEN db_file$ FOR RANDOM AS #1 LEN = rec_len%

FIELD #1, 2 AS idbuf$, 20 AS namebuf$, 8 AS scorebuf$

FOR i% = num_recs% TO 1 STEP -1
    GET #1, i%
    id% = CVI%(idbuf$)
    score# = CVD#(scorebuf$)
    PRINT (((("  [" + STR$(id%)) + "] ") + RTRIM$(namebuf$)) + " — ") + STR$(score#)
NEXT i%

CLOSE #1

' ---- Update one record in place ----

OPEN db_file$ FOR RANDOM AS #1 LEN = rec_len%

FIELD #1, 2 AS idbuf$, 20 AS namebuf$, 8 AS scorebuf$

' Read record 2, update score, write it back
GET #1, 2
LSET scorebuf$ = MKD#(61.5)
PUT #1, 2

CLOSE #1

' ---- Verify the update ----

PRINT "After update:"
OPEN db_file$ FOR RANDOM AS #1 LEN = rec_len%

FIELD #1, 2 AS idbuf$, 20 AS namebuf$, 8 AS scorebuf$

FOR i% = 1 TO num_recs%
    GET #1, i%
    PRINT (("  " + RTRIM$(namebuf$)) + ": ") + STR$(CVD#(scorebuf$))
NEXT i%

CLOSE #1

END
