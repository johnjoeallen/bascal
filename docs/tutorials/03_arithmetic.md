[Home](../) / [Tutorials](./) / Operators and Expressions

<div class="prose" markdown="1">

Arithmetic (`+ - * / \ MOD ^`), comparison (`= <> < <= > >=`, returning `-1`/`0`), and logical (`AND OR NOT XOR`) all work as in classic BASIC — including the one sharp edge worth knowing up front: `NOT` is bitwise, so `NOT 1 = -2`, not `0`. Test for false with `(expr) = 0`, never `NOT expr`.

</div>

<div class="snippet" markdown="1">

### Integer division vs MOD

    print a%; "/ "; b%; "="; a% / b%      // 17 / 5 = 3  (truncates)
    print a%; "\ "; b%; "="; a% \ b%     // 17 \ 5 = 3  (integer quotient)
    print a%; "MOD "; b%; "="; a% mod b%  // 17 MOD 5 = 2  (remainder)

</div>

<div class="snippet" markdown="1">

### The NOT gotcha

This is why BASCAL's own if/while transpilation inverts conditions with (cond) = 0 instead of NOT — see [If Transpilation](../manual/generated-basic-shape.md#if-transpilation) in the manual.

    x% = 7
    if x% > 0 and x% < 10 then
        print x%; "is in 1..9"
    end if
    print 6 xor 3; " (expect 5 -- 110 XOR 011 = 101)"

</div>



[← Variables and Constants](02_variables.md)  ·  [Conditions →](04_conditions.md)


<!-- BEGIN generated tutorial source -->

### `tutorial/03_arithmetic.bcl`

```bascal

// Tutorial — Operators and Expressions
//
// Arithmetic:   +  -  *  /  \  MOD  ^
// Comparison:   =  <>  <  <=  >  >=   (result: -1 true, 0 false)
// Logical:      AND  OR  NOT  XOR  (bitwise — see note below)
// String:       + concatenates strings
//
// Precedence (highest first):
//   ^                 exponentiation (right-associative)
//   unary -           negation
//   * /               multiply / divide
//   \                 integer (floor) division
//   MOD               modulus (remainder)
//   + -               add / subtract
//   = <> < <= > >=    comparison
//   NOT               bitwise NOT
//   AND               bitwise AND
//   OR                bitwise OR
//   XOR               bitwise XOR
//
// IMPORTANT: NOT is bitwise, so NOT 1 = -2, not 0.
// Test for false with (expr) = 0, not NOT expr.
program arithmetic

/* Arithmetic — mix labels and numbers with ; */
a% = 17
b% = 5
print a%; "+ "; b%; "="; a% + b%   // 17 + 5 = 22
print a%; "- "; b%; "="; a% - b%   // 17 - 5 = 12
print a%; "* "; b%; "="; a% * b%   // 17 * 5 = 85
print a%; "/ "; b%; "="; a% / b%   // 17 / 5 = 3  (truncates)

/* Integer division and MOD */
print a%; "\ "; b%; "="; a% \ b%   // 17 \ 5 = 3  (integer quotient)
print a%; "MOD "; b%; "="; a% mod b%  // 17 MOD 5 = 2  (remainder)

/* Exponentiation — right-associative */
print "2 ^ 8 ="; 2 ^ 8                   // 256
print "2 ^ 3 ^ 2 ="; 2 ^ 3 ^ 2          // 512  (= 2 ^ (3^2) = 2^9)

/* Precedence */
print 2 + 3 * 4; " (expect 14 — * before +)"
print (2 + 3) * 4; " (expect 20 — parens first)"

/* Comparison — -1 means true, 0 means false */
print 10 > 3; " (expect -1)"
print 10 < 3; " (expect  0)"
print 7 = 7;  " (expect -1)"
print 7 <> 8; " (expect -1)"

/* Logical — AND, OR, XOR are bitwise but work correctly with 0/-1 values */
x% = 7
if x% > 0 and x% < 10 then
    print x%; "is in 1..9"
end if
print 6 xor 3; " (expect 5 — 110 XOR 011 = 101)"

/* String concatenation */
print "Hello" + ", " + "World" + "!"

/* Unary negation */
n% = 42
print -n%              // -42

end

```

### `tutorial/03_arithmetic.bas`

```basic

10 ' BASCAL generated BASIC
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Tutorial — Operators and Expressions
40 '
50 ' Arithmetic:   +  -  *  /  \  MOD  ^
60 ' Comparison:   =  <>  <  <=  >  >=   (result: -1 true, 0 false)
70 ' Logical:      AND  OR  NOT  XOR  (bitwise — see note below)
80 ' String:       + concatenates strings
90 '
100 ' Precedence (highest first):
110 ' ^                 exponentiation (right-associative)
120 ' unary -           negation
130 ' * /               multiply / divide
140 ' \                 integer (floor) division
150 ' MOD               modulus (remainder)
160 ' + -               add / subtract
170 ' = <> < <= > >=    comparison
180 ' NOT               bitwise NOT
190 ' AND               bitwise AND
200 ' OR                bitwise OR
210 ' XOR               bitwise XOR
220 '
230 ' IMPORTANT: NOT is bitwise, so NOT 1 = -2, not 0.
240 ' Test for false with (expr) = 0, not NOT expr.

250 ' Arithmetic — mix labels and numbers with ;
260 a% = 17
270 b% = 5
280 PRINT a%; "+ "; b%; "="; a% + b%
290 PRINT a%; "- "; b%; "="; a% - b%
300 PRINT a%; "* "; b%; "="; a% * b%
310 PRINT a%; "/ "; b%; "="; a% / b%

320 ' Integer division and MOD
330 PRINT a%; "\ "; b%; "="; a% \ b%
340 PRINT a%; "MOD "; b%; "="; a% MOD b%

350 ' Exponentiation — right-associative
360 PRINT "2 ^ 8 ="; 2 ^ 8
370 PRINT "2 ^ 3 ^ 2 ="; 2 ^ (3 ^ 2)

380 ' Precedence
390 PRINT 2 + (3 * 4); " (expect 14 — * before +)"
400 PRINT (2 + 3) * 4; " (expect 20 — parens first)"

410 ' Comparison — -1 means true, 0 means false
420 PRINT 10 > 3; " (expect -1)"
430 PRINT 10 < 3; " (expect  0)"
440 PRINT 7 = 7; " (expect -1)"
450 PRINT 7 <> 8; " (expect -1)"

460 ' Logical — AND, OR, XOR are bitwise but work correctly with 0/-1 values
470 x% = 7
480 IF ((x% > 0) AND (x% < 10)) = 0 THEN GOTO 500
490     PRINT x%; "is in 1..9"
500 REM END IF
510 PRINT 6 XOR 3; " (expect 5 — 110 XOR 011 = 101)"

520 ' String concatenation
530 PRINT (("Hello" + ", ") + "World") + "!"

540 ' Unary negation
550 n% = 42
560 PRINT -n%

570 END

```

<!-- END generated tutorial source -->
