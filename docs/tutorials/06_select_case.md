[Home](../) / [Tutorials](./) / Select Case

<div class="prose" markdown="1">

`select case` evaluates its expression once, stores it in a transpiler- generated temporary, and dispatches against each `case` clause in order. Patterns can be an exact value, a comma-separated list, an inclusive `lo to hi` range, or an `is <op> value` comparison; `case else` is the default and must come last. See the [control-flow comparison](../#control-flow) on the homepage for the GOTO dispatch chain this transpiles to.

</div>

<div class="snippet" markdown="1">

### IS comparisons

```bascal
select case temp%
case is < 0
    print "Below freezing ("; temp%; "deg)"
case is < 10
    print "Cold ("; temp%; "deg)"
case is < 20
    print "Cool ("; temp%; "deg)"
case else
    print "Hot ("; temp%; "deg)"
end select
```

</div>



[← Loops](05_loops.md)  ·  [Functions →](07_functions.md)


<!-- BEGIN generated tutorial source -->

### `tutorial/06_select_case.bcl`

```bascal

// Tutorial — SELECT CASE
//
// SELECT CASE tests one expression against multiple patterns.  The
// compiler evaluates the expression once, stores it in a temporary
// variable, and emits an IF/goto dispatch chain.
//
// Pattern forms:
//   case value               — exact match
//   case v1, v2, v3          — any of the listed values
//   case low to high         — inclusive range
//   case is <op> value       — comparison (=  <>  <  <=  >  >=)
//   case else                — default; must be the last clause
program selectCase

/* Integer select: convert numeric score to letter grade */
score% = 85

select case score%
    case 100
        print "Perfect!"
    case 90 to 99
        print "A  — Excellent"
    case 80 to 89
        print "B  — Good"      // score% = 85 matches here
    case 70 to 79
        print "C  — Satisfactory"
    case 60 to 69
        print "D  — Passing"
    case is >= 0
        print "F  — Fail"
    case else
        print "Invalid score"
end select

/* String select: day-of-week classification */
day$ = "Saturday"

select case day$
    case "Monday", "Tuesday", "Wednesday", "Thursday", "Friday"
        print day$ + " is a weekday"
    case "Saturday", "Sunday"
        print day$ + " is a weekend"   // matches here
    case else
        print "Unknown day: " + day$
end select

/* IS comparisons on temperature */
temp% = -3

select case temp%
    case is < 0
        print "Below freezing ("; temp%; "°)"  // matches here
    case is < 10
        print "Cold ("; temp%; "°)"
    case is < 20
        print "Cool ("; temp%; "°)"
    case is < 30
        print "Warm ("; temp%; "°)"
    case else
        print "Hot ("; temp%; "°)"
end select

/* Multi-value list on a menu choice */
choice% = 2

select case choice%
    case 1
        print "New game"
    case 2, 3
        print "Load game"   // choice% = 2 matches
    case 4
        print "Options"
    case else
        print "Quit"
end select

end

```

### `tutorial/06_select_case.bas`

```basic

10 ' BASCAL generated BASIC
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Tutorial — SELECT CASE
40 '
50 ' SELECT CASE tests one expression against multiple patterns.  The
60 ' compiler evaluates the expression once, stores it in a temporary
70 ' variable, and emits an IF/goto dispatch chain.
80 '
90 ' Pattern forms:
100 ' case value               — exact match
110 ' case v1, v2, v3          — any of the listed values
120 ' case low to high         — inclusive range
130 ' case is <op> value       — comparison (=  <>  <  <=  >  >=)
140 ' case else                — default; must be the last clause

150 ' Integer select: convert numeric score to letter grade
160 score% = 85

170 BCCT2% = score%
180 IF (BCCT2% = 100) <> 0 THEN GOTO 250
190 IF (BCCT2% >= 90 AND BCCT2% <= 99) <> 0 THEN GOTO 270
200 IF (BCCT2% >= 80 AND BCCT2% <= 89) <> 0 THEN GOTO 290
210 IF (BCCT2% >= 70 AND BCCT2% <= 79) <> 0 THEN GOTO 310
220 IF (BCCT2% >= 60 AND BCCT2% <= 69) <> 0 THEN GOTO 330
230 IF (BCCT2% >= 0) <> 0 THEN GOTO 350
240 GOTO 370
250     PRINT "Perfect!"
260     GOTO 380
270     PRINT "A  — Excellent"
280     GOTO 380
290     PRINT "B  — Good"
300     GOTO 380
310     PRINT "C  — Satisfactory"
320     GOTO 380
330     PRINT "D  — Passing"
340     GOTO 380
350     PRINT "F  — Fail"
360     GOTO 380
370     PRINT "Invalid score"
380 REM END SELECT

390 ' String select: day-of-week classification
400 day$ = "Saturday"

410 BCCT4$ = day$
420 IF (BCCT4$ = "Monday" OR BCCT4$ = "Tuesday" OR BCCT4$ = "Wednesday" OR BCCT4$ = "Thursday" OR BCCT4$ = "Friday") <> 0 THEN GOTO 450
430 IF (BCCT4$ = "Saturday" OR BCCT4$ = "Sunday") <> 0 THEN GOTO 470
440 GOTO 490
450     PRINT day$ + " is a weekday"
460     GOTO 500
470     PRINT day$ + " is a weekend"
480     GOTO 500
490     PRINT "Unknown day: " + day$
500 REM END SELECT

510 ' IS comparisons on temperature
520 temp% = -3

530 BCCT6% = temp%
540 IF (BCCT6% < 0) <> 0 THEN GOTO 590
550 IF (BCCT6% < 10) <> 0 THEN GOTO 610
560 IF (BCCT6% < 20) <> 0 THEN GOTO 630
570 IF (BCCT6% < 30) <> 0 THEN GOTO 650
580 GOTO 670
590     PRINT "Below freezing ("; temp%; "°)"
600     GOTO 680
610     PRINT "Cold ("; temp%; "°)"
620     GOTO 680
630     PRINT "Cool ("; temp%; "°)"
640     GOTO 680
650     PRINT "Warm ("; temp%; "°)"
660     GOTO 680
670     PRINT "Hot ("; temp%; "°)"
680 REM END SELECT

690 ' Multi-value list on a menu choice
700 choice% = 2

710 BCCT8% = choice%
720 IF (BCCT8% = 1) <> 0 THEN GOTO 760
730 IF (BCCT8% = 2 OR BCCT8% = 3) <> 0 THEN GOTO 780
740 IF (BCCT8% = 4) <> 0 THEN GOTO 800
750 GOTO 820
760     PRINT "New game"
770     GOTO 830
780     PRINT "Load game"
790     GOTO 830
800     PRINT "Options"
810     GOTO 830
820     PRINT "Quit"
830 REM END SELECT

840 END

```

<!-- END generated tutorial source -->
