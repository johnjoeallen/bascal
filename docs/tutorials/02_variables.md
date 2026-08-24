[Home](../) / [Tutorials](./) / Variables and Constants

<div class="prose" markdown="1">

Every name ends with a type suffix that tells the runtime how to store the value: `%` 16-bit integer, `$` string, `!` single, `#` double, `&` long. Variables are global and spring into existence on first use — `dim` is only needed for arrays or when you want to be explicit. `const` names a value that can't change.

</div>

<div class="snippet" markdown="1">

### Constants and assignment

    const max_score%  = 100
    const app_name$   = "Grade Checker"
    const tax_rate!   = 0.2

    playerName$  = "Alice"
    score%       = 87

</div>

<div class="snippet" markdown="1">

### print mixes types directly with ;

No str\$() needed just to print a number next to a label.

    print "Score:       "; score%; "/ "; max_score%

    // str$() is still available when you need to build a string value
    let greeting$ = "Score is " + str$(score%)

</div>



[← Hello, World](01_hello.md)  ·  [Operators and Expressions →](03_arithmetic.md)


<!-- BEGIN generated tutorial source -->

### `tutorial/02_variables.bcl`

```bascal

// Tutorial — Variables and Constants
//
// Every name in BASCAL ends with a type suffix that tells the runtime
// how to store the value:
//
//   %   integer   — 16-bit signed, -32768 to 32767
//   $   string    — variable-length text
//   !   single    — 32-bit floating-point
//   #   double    — 64-bit floating-point
//   &   long      — 32-bit signed integer
//
// All variables are global.  They spring into existence on first use;
// dim (or its synonym declare) is needed only for arrays or when you
// want to be explicit -- declare tends to read better for a plain
// scalar, dim for an array.
//
// const names a value that cannot change.  Use it for magic numbers
// so the intent is clear and the value lives in one place.
program variables

const maxScore%  = 100
const passMark%  = 60
const appName$   = "Grade Checker"
const taxRate!   = 0.2

// Variable assignment uses =
playerName$  = "Alice"
score%       = 87
temperature! = 36.6

// print mixes strings and numbers directly with ; (no str$() needed)
print appName$
print "Player:      "; playerName$
print "Score:       "; score%; "/ "; maxScore%
print "Pass mark:   "; passMark%
print "Temperature: "; temperature!
print "Tax rate:    "; taxRate!

// str$() is still available when you need to build a string value
let greeting$ = "Score is " + str$(score%)
print greeting$

end

```

### `tutorial/02_variables.bas`

```basic

10 ' BASCAL generated BASIC
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Tutorial — Variables and Constants
40 '
50 ' Every name in BASCAL ends with a type suffix that tells the runtime
60 ' how to store the value:
70 '
80 ' %   integer   — 16-bit signed, -32768 to 32767
90 ' $   string    — variable-length text
100 ' !   single    — 32-bit floating-point
110 ' #   double    — 64-bit floating-point
120 ' &   long      — 32-bit signed integer
130 '
140 ' All variables are global.  They spring into existence on first use;
150 ' dim (or its synonym declare) is needed only for arrays or when you
160 ' want to be explicit -- declare tends to read better for a plain
170 ' scalar, dim for an array.
180 '
190 ' const names a value that cannot change.  Use it for magic numbers
200 ' so the intent is clear and the value lives in one place.

210 maxscore% = 100
220 passmark% = 60
230 appname$ = "Grade Checker"
240 taxrate! = 0.2

250 ' Variable assignment uses =
260 playername$ = "Alice"
270 score% = 87
280 temperature! = 36.6

290 ' print mixes strings and numbers directly with ; (no str$() needed)
300 PRINT appname$
310 PRINT "Player:      "; playername$
320 PRINT "Score:       "; score%; "/ "; maxscore%
330 PRINT "Pass mark:   "; passmark%
340 PRINT "Temperature: "; temperature!
350 PRINT "Tax rate:    "; taxrate!

360 ' str$() is still available when you need to build a string value
370 greeting$ = "Score is " + STR$(score%)
380 PRINT greeting$

390 END

```

<!-- END generated tutorial source -->
