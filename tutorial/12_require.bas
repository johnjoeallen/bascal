' BASCAL generated BASIC
' Functions are lowered to global variables, labels, and GOSUB

' stats.bcl — basic statistics library for the BASCAL tutorial.
' Loaded by tutorial/12_require.bcl via:
' require stats
' 
' Provides: mean!, maximum%, minimum%, rangeOf%

' Tutorial 12 — REQUIRE and multi-file projects
' 
' REQUIRE loads another .bcl file and merges its functions into the
' generated output.  The path is dot-separated and maps to a file:
' 
' require stats   →  stats.bcl  (in the same directory or a -L path)
' require com.bascal.sort.bubbleSort
' →  com/bascal/sort/bubbleSort.bcl
' 
' All required functions become part of the single generated .bas file.
' The original require line is preserved as a comment in the output.
' 
' Run with:
' bcc tutorial/12_require.bcl -L tutorial/lib
' 
' The -L flag adds tutorial/lib/ to the search path so that
' require stats   resolves to  tutorial/lib/stats.bcl

CONST N% = 8
DIM scores%(N%)

scores%(0) = 74
scores%(1) = 91
scores%(2) = 63
scores%(3) = 88
scores%(4) = 55
scores%(5) = 97
scores%(6) = 72
scores%(7) = 84

PRINT "Scores: 74 91 63 88 55 97 72 84"
mean_count% = N%
DIM mean_data%(N%)

' copy array argument into lowered function storage: scores%() -> mean_data%()
FOR BCC_T1% = 1 TO N%
    mean_data%(BCC_T1%) = scores%(BCC_T1%)
NEXT BCC_T1%

GOSUB 10

' copy mutated array argument back to caller storage: mean_data%() -> scores%()
FOR BCC_T2% = 1 TO N%
    scores%(BCC_T2%) = mean_data%(BCC_T2%)
NEXT BCC_T2%

PRINT "Mean:   " + STR$(mean_result!)
maximum_count% = N%
DIM maximum_data%(N%)

' copy array argument into lowered function storage: scores%() -> maximum_data%()
FOR BCC_T3% = 1 TO N%
    maximum_data%(BCC_T3%) = scores%(BCC_T3%)
NEXT BCC_T3%

GOSUB 20

' copy mutated array argument back to caller storage: maximum_data%() -> scores%()
FOR BCC_T4% = 1 TO N%
    scores%(BCC_T4%) = maximum_data%(BCC_T4%)
NEXT BCC_T4%

PRINT "Max:    " + STR$(maximum_result%)
minimum_count% = N%
DIM minimum_data%(N%)

' copy array argument into lowered function storage: scores%() -> minimum_data%()
FOR BCC_T5% = 1 TO N%
    minimum_data%(BCC_T5%) = scores%(BCC_T5%)
NEXT BCC_T5%

GOSUB 40

' copy mutated array argument back to caller storage: minimum_data%() -> scores%()
FOR BCC_T6% = 1 TO N%
    scores%(BCC_T6%) = minimum_data%(BCC_T6%)
NEXT BCC_T6%

PRINT "Min:    " + STR$(minimum_result%)
rangeof_count% = N%
DIM rangeof_data%(N%)

' copy array argument into lowered function storage: scores%() -> rangeof_data%()
FOR BCC_T7% = 1 TO N%
    rangeof_data%(BCC_T7%) = scores%(BCC_T7%)
NEXT BCC_T7%

GOSUB 60

' copy mutated array argument back to caller storage: rangeof_data%() -> scores%()
FOR BCC_T8% = 1 TO N%
    scores%(BCC_T8%) = rangeof_data%(BCC_T8%)
NEXT BCC_T8%

PRINT "Range:  " + STR$(rangeof_result%)

END

' function mean!(data%, count%)
10 ' Arithmetic mean of data%(0..count%-1).
    sum% = 0
    FOR i% = 0 TO mean_count% - 1
        sum% = sum% + mean_data%(i%)
    NEXT i%
    mean_result! = sum% / mean_count%
    RETURN
' end function mean!

' function maximum%(data%, count%)
20 ' Largest element in data%(0..count%-1).
    best% = maximum_data%(0)
    FOR i% = 1 TO maximum_count% - 1
        IF (maximum_data%(i%) > best%) = 0 THEN GOTO 30
            best% = maximum_data%(i%)
30 REM END IF
    NEXT i%
    maximum_result% = best%
    RETURN
' end function maximum%

' function minimum%(data%, count%)
40 ' Smallest element in data%(0..count%-1).
    best% = minimum_data%(0)
    FOR i% = 1 TO minimum_count% - 1
        IF (minimum_data%(i%) < best%) = 0 THEN GOTO 50
            best% = minimum_data%(i%)
50 REM END IF
    NEXT i%
    minimum_result% = best%
    RETURN
' end function minimum%

' function rangeOf%(data%, count%)
60 ' Difference between maximum and minimum.
    maximum_count% = rangeof_count%
    DIM maximum_data%(rangeof_count%)

    ' copy array argument into lowered function storage: rangeof_data%() -> maximum_data%()
    FOR BCC_T11% = 1 TO rangeof_count%
        maximum_data%(BCC_T11%) = rangeof_data%(BCC_T11%)
    NEXT BCC_T11%

    GOSUB 20

    ' copy mutated array argument back to caller storage: maximum_data%() -> rangeof_data%()
    FOR BCC_T12% = 1 TO rangeof_count%
        rangeof_data%(BCC_T12%) = maximum_data%(BCC_T12%)
    NEXT BCC_T12%

    minimum_count% = rangeof_count%
    DIM minimum_data%(rangeof_count%)

    ' copy array argument into lowered function storage: rangeof_data%() -> minimum_data%()
    FOR BCC_T13% = 1 TO rangeof_count%
        minimum_data%(BCC_T13%) = rangeof_data%(BCC_T13%)
    NEXT BCC_T13%

    GOSUB 40

    ' copy mutated array argument back to caller storage: minimum_data%() -> rangeof_data%()
    FOR BCC_T14% = 1 TO rangeof_count%
        rangeof_data%(BCC_T14%) = minimum_data%(BCC_T14%)
    NEXT BCC_T14%

    rangeof_result% = maximum_result% - minimum_result%
    RETURN
' end function rangeOf%
