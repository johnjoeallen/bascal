[Home](../../) / [Tutorials](../) / Functions

<div class="prose" markdown="1">

Generated sources: [BCL](https://github.com/johnjoeallen/bascal/blob/main/tutorial/07_functions.bcl), [BASIC](https://github.com/johnjoeallen/bascal/blob/main/tutorial/07_functions.bas), [C](https://github.com/johnjoeallen/bascal/blob/main/tutorial/07_functions.c), and [JVM assembly](https://github.com/johnjoeallen/bascal/blob/main/tutorial/07_functions.j).

A function is declared `function name%(...) ... end function`; the name carries the return-type suffix and every path must reach a `return`. Variables declared inside a function are local by default — the transpiler prefixes them with the function name — so two functions can each use `i%` without conflict. To touch a global from inside a function, declare it with `global varname`. Functions can't recurse — directly (calling themselves) or indirectly (two or more functions calling each other in a cycle) — since any recursive call would overwrite its own parameter variables; the transpiler checks the whole call graph and rejects any cycle.

</div>

<div class="snippet" markdown="1">

### Typed return and calling one function from another

```bascal
function clamp%(value%, lo%, hi%)
    // Constrain value to [lo, hi].
    return max%(lo%, min%(value%, hi%))
end function
```

</div>

<div class="snippet" markdown="1">

### global for shared state

```bascal
runningTotal% = 0

function addToTotal%(x%)
    global runningTotal%
    runningTotal% = runningTotal% + x%
    return runningTotal%
end function
```

</div>

<div class="snippet" markdown="1">

### Scalar methods and chaining

A method is a typed function with an implicit scalar receiver named `self`. Calls use a dot and parentheses; the result can become the receiver of the next method.

```bascal
method$ shout$()
    return self$ + "!"
end method

print "  hello  ".ltrim().rtrim().ucase().shout()
```

Built-in methods such as `left`, `mid`, `len`, `abs`, and `sin` use the same syntax. Methods transpile to ordinary typed calls for both targets; they do not create runtime objects.

</div>



[← Select Case](06_select_case.md)  ·  [Arrays →](08_arrays.md)


<!-- BEGIN generated tutorial source -->

<details class="source-embed" markdown="1">

<summary><code>tutorial/07_functions.bcl</code></summary>



```bascal

// Tutorial — Functions
//
// A BASCAL function is declared with FUNCTION ... END FUNCTION.
// The function name carries the return type suffix.  Parameters
// also carry type suffixes.  Every function must reach a RETURN.
//
// Variables declared inside a function are local by default: the compiler
// prefixes them with the function name.  To access a global variable from
// inside a function, declare it with:  global varname
//
// Functions cannot recurse, directly or indirectly (parameters would be
// overwritten) -- the compiler checks the whole call graph and rejects
// any cycle.  Use an explicit stack array for recursive algorithms.
//
// Scalar methods are typed functions with an implicit receiver.  Calls use
// dot syntax and can chain: word$.left(1).ucase().  The existing titleCase$
// function below demonstrates this form; methods transpile to ordinary
// calls for both targets.
program functions

require com.bascal.stdlib.ucase
require com.bascal.stdlib.lcase

/* Integer arithmetic functions */
// a% -- first value to compare
// b% -- second value to compare
function max%(a%, b%)
    if a% > b% then
        return a%
    else
        return b%
    end if
end function

// a% -- first value to compare
// b% -- second value to compare
function min%(a%, b%)
    if a% < b% then
        return a%
    else
        return b%
    end if
end function

// value% -- number to constrain
// lo%    -- lower bound, inclusive
// hi%    -- upper bound, inclusive
function clamp%(value%, lo%, hi%)
    // Constrain value to [lo, hi].
    return max%(lo%, min%(value%, hi%))
end function

/* String functions */
// text$ -- string to repeat
// n%    -- number of times to repeat it
function repeat$(text$, n%)
    // Concatenate text$ with itself n times.
    acc$ = ""
    for i% = 1 to n%
        acc$ = acc$ + text$
    end for
    return acc$
end function

// word$ -- string to title-case
function titleCase$(word$)
    // Capitalise first letter, lowercase remainder.
    // UCASE$/LCASE$ aren't real MBASIC/BASCOM 2.00 builtins (verified
    // against a real IBM BASIC Compiler 2.00 under dosbox-x), so this
    // requires BASCAL's own com.bascal.stdlib implementations above.
    if word$.len() = 0 then
        return ""
    end if
    return word$.left(1).ucase() + word$.mid(2).lcase()
end function

/* Local variable scoping — each function has its own i% and acc% */
// n% -- upper bound of the sum, inclusive
function sumTo%(n%)
    // i% and acc% are local to sumTo%.
    acc% = 0
    for i% = 1 to n%
        acc% = acc% + i%
    end for
    return acc%
end function

// n% -- upper bound of the product, inclusive
function productTo%(n%)
    // i% and acc% here are independent of sumTo%'s i% and acc%.
    acc% = 1
    for i% = 1 to n%
        acc% = acc% * i%
    end for
    return acc%
end function

/* Global variable accessed inside a function with the global keyword */
runningTotal% = 0

// x% -- amount to add to the running total
function addToTotal%(x%)
    global runningTotal%
    runningTotal% = runningTotal% + x%
    return runningTotal%
end function

/* --- Exercise the functions --- */

// print mixes string labels and numeric results directly with ;
print "max(4, 9) = "; max%(4, 9)        // 9
print "min(4, 9) = "; min%(4, 9)        // 4
print "clamp(15,1,10) = "; clamp%(15, 1, 10)  // 10
print "clamp(-3,1,10) = "; clamp%(-3, 1, 10)  // 1
print "clamp(7,1,10)  = "; clamp%(7, 1, 10)   // 7

print repeat$("ab", 4)       // abababab
print titleCase$("bASCAL")   // Bascal

/* Functions chained in expressions */
lo% = min%(max%(0, -5), 100)   // max(0,-5)=0, min(0,100)=0
print "lo = "; lo%

/* Calling the same function twice — each result is captured separately */
a$ = repeat$("x", 3)
b$ = repeat$("y", 2)
print a$; " "; b$    // xxx yy

/* Local scoping: sumTo% and productTo% each use i% without conflict */
print "sumTo(5)     = "; sumTo%(5)      // 15
print "productTo(5) = "; productTo%(5)  // 120

/* Global variable shared across calls */
dummy% = addToTotal%(10)
dummy% = addToTotal%(5)
print "runningTotal = "; runningTotal%  // 15

end

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/07_functions.bas</code></summary>



```basic

10 ' BASCAL generated BASIC -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Upper-cases self$. Not a real MBASIC/BASCOM 2.00 builtin -- verified
40 ' against a real IBM BASIC Compiler 2.00 under dosbox-x -- so BASCAL ships
50 ' its own. Declared as a scalar method (see GitHub issue #41 and
60 ' ltrim.bcl's own doc comment for the reasoning) -- ucase$(s$) still works
70 ' via ordinary-call syntax resolving to this same declaration.

80 ' Lower-cases self$. Not a real MBASIC/BASCOM 2.00 builtin -- verified
90 ' against a real IBM BASIC Compiler 2.00 under dosbox-x -- so BASCAL ships
100 ' its own. Declared as a scalar method (see GitHub issue #41 and
110 ' ltrim.bcl's own doc comment for the reasoning) -- lcase$(s$) still works
120 ' via ordinary-call syntax resolving to this same declaration.

130 ' Tutorial — Functions
140 '
150 ' A BASCAL function is declared with FUNCTION ... END FUNCTION.
160 ' The function name carries the return type suffix.  Parameters
170 ' also carry type suffixes.  Every function must reach a RETURN.
180 '
190 ' Variables declared inside a function are local by default: the compiler
200 ' prefixes them with the function name.  To access a global variable from
210 ' inside a function, declare it with:  global varname
220 '
230 ' Functions cannot recurse, directly or indirectly (parameters would be
240 ' overwritten) -- the compiler checks the whole call graph and rejects
250 ' any cycle.  Use an explicit stack array for recursive algorithms.
260 '
270 ' Scalar methods are typed functions with an implicit receiver.  Calls use
280 ' dot syntax and can chain: word$.left(1).ucase().  The existing titleCase$
290 ' function below demonstrates this form; methods transpile to ordinary
300 ' calls for both targets.

310 ' Integer arithmetic functions
320 ' a% -- first value to compare
330 ' b% -- second value to compare

340 ' a% -- first value to compare
350 ' b% -- second value to compare

360 ' value% -- number to constrain
370 ' lo%    -- lower bound, inclusive
380 ' hi%    -- upper bound, inclusive

390 ' String functions
400 ' text$ -- string to repeat
410 ' n%    -- number of times to repeat it

420 ' word$ -- string to title-case

430 ' Local variable scoping — each function has its own i% and acc%
440 ' n% -- upper bound of the sum, inclusive

450 ' n% -- upper bound of the product, inclusive

460 ' Global variable accessed inside a function with the global keyword
470 runningtotal% = 0

480 ' x% -- amount to add to the running total

490 ' --- Exercise the functions ---

500 ' print mixes string labels and numeric results directly with ;
510 maxA0% = 4
520 maxB0% = 9
530 GOSUB 1430
540 PRINT "max(4, 9) = "; maxResult0%
550 minA0% = 4
560 minB0% = 9
570 GOSUB 1530
580 PRINT "min(4, 9) = "; minResult0%
590 clampValue0% = 15
600 clampLo0% = 1
610 clampHi0% = 10
620 GOSUB 1630
630 PRINT "clamp(15,1,10) = "; clampResult0%
640 clampValue0% = -3
650 clampLo0% = 1
660 clampHi0% = 10
670 GOSUB 1630
680 PRINT "clamp(-3,1,10) = "; clampResult0%
690 clampValue0% = 7
700 clampLo0% = 1
710 clampHi0% = 10
720 GOSUB 1630
730 PRINT "clamp(7,1,10)  = "; clampResult0%

740 repeatText0$ = "ab"
750 repeatN0% = 4
760 GOSUB 1740
770 PRINT repeatResult0$
780 titlecaseWord0$ = "bASCAL"
790 GOSUB 1830
800 PRINT titlecaseResult0$

810 ' Functions chained in expressions
820 maxA0% = 0
830 maxB0% = -5
840 GOSUB 1430
850 minA0% = maxResult0%
860 minB0% = 100
870 GOSUB 1530
880 lo% = minResult0%
890 PRINT "lo = "; lo%

900 ' Calling the same function twice — each result is captured separately
910 repeatText0$ = "x"
920 repeatN0% = 3
930 GOSUB 1740
940 a$ = repeatResult0$
950 repeatText0$ = "y"
960 repeatN0% = 2
970 GOSUB 1740
980 b$ = repeatResult0$
990 PRINT a$; " "; b$

1000 ' Local scoping: sumTo% and productTo% each use i% without conflict
1010 sumtoN0% = 5
1020 GOSUB 1990
1030 PRINT "sumTo(5)     = "; sumtoResult0%
1040 producttoN0% = 5
1050 GOSUB 2080
1060 PRINT "productTo(5) = "; producttoResult0%

1070 ' Global variable shared across calls
1080 addtototalX0% = 10
1090 GOSUB 2170
1100 dummy% = addtototalResult0%
1110 addtototalX0% = 5
1120 GOSUB 2170
1130 dummy% = addtototalResult0%
1140 PRINT "runningTotal = "; runningtotal%

1150 END

1160 ' function ucase$()
1170     ucaseOut0$ = ""
1180     FOR ucaseI0% = 1 TO LEN(ucaseSelf0$)
1190         ucaseC0% = ASC(MID$(ucaseSelf0$, ucaseI0%, 1))
1200         IF (ucaseC0% >= 97) = 0 THEN GOTO 1230
1210         IF (ucaseC0% <= 122) = 0 THEN GOTO 1230
1220             ucaseC0% = ucaseC0% - 32
1230         REM END IF
1240         ucaseOut0$ = ucaseOut0$ + CHR$(ucaseC0%)
1250     NEXT ucaseI0%
1260     ucaseResult0$ = ucaseOut0$
1270     RETURN
1280 ' end function ucase$

1290 ' function lcase$()
1300     lcaseOut0$ = ""
1310     FOR lcaseI0% = 1 TO LEN(lcaseSelf0$)
1320         lcaseC0% = ASC(MID$(lcaseSelf0$, lcaseI0%, 1))
1330         IF (lcaseC0% >= 65) = 0 THEN GOTO 1360
1340         IF (lcaseC0% <= 90) = 0 THEN GOTO 1360
1350             lcaseC0% = lcaseC0% + 32
1360         REM END IF
1370         lcaseOut0$ = lcaseOut0$ + CHR$(lcaseC0%)
1380     NEXT lcaseI0%
1390     lcaseResult0$ = lcaseOut0$
1400     RETURN
1410 ' end function lcase$

1420 ' function max%(a%, b%)
1430     IF (maxA0% > maxB0%) = 0 THEN GOTO 1470
1440         maxResult0% = maxA0%
1450         RETURN
1460         GOTO 1490
1470         maxResult0% = maxB0%
1480         RETURN
1490     REM END IF
1500     RETURN
1510 ' end function max%

1520 ' function min%(a%, b%)
1530     IF (minA0% < minB0%) = 0 THEN GOTO 1570
1540         minResult0% = minA0%
1550         RETURN
1560         GOTO 1590
1570         minResult0% = minB0%
1580         RETURN
1590     REM END IF
1600     RETURN
1610 ' end function min%

1620 ' function clamp%(value%, lo%, hi%)
1630     ' Constrain value to [lo, hi].
1640     minA0% = clampValue0%
1650     minB0% = clampHi0%
1660     GOSUB 1530
1670     maxA0% = clampLo0%
1680     maxB0% = minResult0%
1690     GOSUB 1430
1700     clampResult0% = maxResult0%
1710     RETURN
1720 ' end function clamp%

1730 ' function repeat$(text$, n%)
1740     ' Concatenate text$ with itself n times.
1750     repeatAcc0$ = ""
1760     FOR repeatI0% = 1 TO repeatN0%
1770         repeatAcc0$ = repeatAcc0$ + repeatText0$
1780     NEXT repeatI0%
1790     repeatResult0$ = repeatAcc0$
1800     RETURN
1810 ' end function repeat$

1820 ' function titlecase$(word$)
1830     ' Capitalise first letter, lowercase remainder.
1840     ' UCASE$/LCASE$ aren't real MBASIC/BASCOM 2.00 builtins (verified
1850     ' against a real IBM BASIC Compiler 2.00 under dosbox-x), so this
1860     ' requires BASCAL's own com.bascal.stdlib implementations above.
1870     IF (LEN(titlecaseWord0$) = 0) = 0 THEN GOTO 1900
1880         titlecaseResult0$ = ""
1890         RETURN
1900     REM END IF
1910     ucaseSelf0$ = LEFT$(titlecaseWord0$, 1)
1920     GOSUB 1170
1930     lcaseSelf0$ = MID$(titlecaseWord0$, 2)
1940     GOSUB 1300
1950     titlecaseResult0$ = ucaseResult0$ + lcaseResult0$
1960     RETURN
1970 ' end function titlecase$

1980 ' function sumto%(n%)
1990     ' i% and acc% are local to sumTo%.
2000     sumtoAcc0% = 0
2010     FOR sumtoI0% = 1 TO sumtoN0%
2020         sumtoAcc0% = sumtoAcc0% + sumtoI0%
2030     NEXT sumtoI0%
2040     sumtoResult0% = sumtoAcc0%
2050     RETURN
2060 ' end function sumto%

2070 ' function productto%(n%)
2080     ' i% and acc% here are independent of sumTo%'s i% and acc%.
2090     producttoAcc0% = 1
2100     FOR producttoI0% = 1 TO producttoN0%
2110         producttoAcc0% = producttoAcc0% * producttoI0%
2120     NEXT producttoI0%
2130     producttoResult0% = producttoAcc0%
2140     RETURN
2150 ' end function productto%

2160 ' function addtototal%(x%)
2170     runningtotal% = runningtotal% + addtototalX0%
2180     addtototalResult0% = runningtotal%
2190     RETURN
2200 ' end function addtototal%

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/07_functions.c</code></summary>



```c

// BASCAL generated C -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
#include <stdio.h>
#include <string.h>

#define BCC_STRBUF_COUNT 8
static char bcc_strbuf[BCC_STRBUF_COUNT][256];
static int bcc_strbuf_next = 0;

static char* bcc_strbuf_take(void);
static const char* bcc_mid(const char* s, int start, int length);
static const char* bcc_chr(int code);
static const char* bcc_stri(int value);
static const char* bcc_strd(double value);

static int bv_i_dummy = 0;
static int bv_i_lo = 0;
static int bv_i_runningtotal = 0;
static char bv_s_a[256] = {0};
static char bv_s_b[256] = {0};

void bf_s_ucase_s(const char* bv_s_self_in, char* bcc_out);
void bf_s_lcase_s(const char* bv_s_self_in, char* bcc_out);
int bf_i_max(int bv_i_a, int bv_i_b);
int bf_i_min(int bv_i_a, int bv_i_b);
int bf_i_clamp(int bv_i_value, int bv_i_lo, int bv_i_hi);
void bf_s_repeat(const char* bv_s_text_in, int bv_i_n, char* bcc_out);
void bf_s_titlecase(const char* bv_s_word_in, char* bcc_out);
int bf_i_sumto(int bv_i_n);
int bf_i_productto(int bv_i_n);
int bf_i_addtototal(int bv_i_x);

void bf_s_ucase_s(const char* bv_s_self_in, char* bcc_out) {
    char bv_s_self[256];
    snprintf(bv_s_self, sizeof(bv_s_self), "%s", bv_s_self_in);
    int bv_i_c = 0;
    int bv_i_i = 0;
    char bv_s_out[256] = {0};

    snprintf(bv_s_out, sizeof(bv_s_out), "%s", "");
    int bt_lim_0 = ((int)strlen(bv_s_self));
    int bt_step_0 = 1;
    for (bv_i_i = 1; bt_step_0 >= 0 ? bv_i_i <= bt_lim_0 : bv_i_i >= bt_lim_0; bv_i_i += bt_step_0) {
        bv_i_c = ((int)(unsigned char)bcc_mid(bv_s_self, bv_i_i, 1)[0]);
        if (((-(bv_i_c >= 97)) && (-(bv_i_c <= 122)))) {
            bv_i_c = (bv_i_c - 32);
        }
        char bt_s_1[256];
        snprintf(bt_s_1, sizeof(bt_s_1), "%s%s", bv_s_out, bcc_chr(bv_i_c));
        snprintf(bv_s_out, sizeof(bv_s_out), "%s", bt_s_1);
    }
    snprintf(bcc_out, 256, "%s", bv_s_out);
    return;
}

void bf_s_lcase_s(const char* bv_s_self_in, char* bcc_out) {
    char bv_s_self[256];
    snprintf(bv_s_self, sizeof(bv_s_self), "%s", bv_s_self_in);
    int bv_i_c = 0;
    int bv_i_i = 0;
    char bv_s_out[256] = {0};

    snprintf(bv_s_out, sizeof(bv_s_out), "%s", "");
    int bt_lim_2 = ((int)strlen(bv_s_self));
    int bt_step_2 = 1;
    for (bv_i_i = 1; bt_step_2 >= 0 ? bv_i_i <= bt_lim_2 : bv_i_i >= bt_lim_2; bv_i_i += bt_step_2) {
        bv_i_c = ((int)(unsigned char)bcc_mid(bv_s_self, bv_i_i, 1)[0]);
        if (((-(bv_i_c >= 65)) && (-(bv_i_c <= 90)))) {
            bv_i_c = (bv_i_c + 32);
        }
        char bt_s_3[256];
        snprintf(bt_s_3, sizeof(bt_s_3), "%s%s", bv_s_out, bcc_chr(bv_i_c));
        snprintf(bv_s_out, sizeof(bv_s_out), "%s", bt_s_3);
    }
    snprintf(bcc_out, 256, "%s", bv_s_out);
    return;
}

int bf_i_max(int bv_i_a, int bv_i_b) {
    if ((-(bv_i_a > bv_i_b))) {
        return bv_i_a;
    } else {
        return bv_i_b;
    }
}

int bf_i_min(int bv_i_a, int bv_i_b) {
    if ((-(bv_i_a < bv_i_b))) {
        return bv_i_a;
    } else {
        return bv_i_b;
    }
}

int bf_i_clamp(int bv_i_value, int bv_i_lo, int bv_i_hi) {
    // Constrain value to [lo, hi].
    return bf_i_max(bv_i_lo, bf_i_min(bv_i_value, bv_i_hi));
}

void bf_s_repeat(const char* bv_s_text_in, int bv_i_n, char* bcc_out) {
    char bv_s_text[256];
    snprintf(bv_s_text, sizeof(bv_s_text), "%s", bv_s_text_in);
    int bv_i_i = 0;
    char bv_s_acc[256] = {0};

    // Concatenate text$ with itself n times.
    snprintf(bv_s_acc, sizeof(bv_s_acc), "%s", "");
    int bt_lim_4 = bv_i_n;
    int bt_step_4 = 1;
    for (bv_i_i = 1; bt_step_4 >= 0 ? bv_i_i <= bt_lim_4 : bv_i_i >= bt_lim_4; bv_i_i += bt_step_4) {
        char bt_s_5[256];
        snprintf(bt_s_5, sizeof(bt_s_5), "%s%s", bv_s_acc, bv_s_text);
        snprintf(bv_s_acc, sizeof(bv_s_acc), "%s", bt_s_5);
    }
    snprintf(bcc_out, 256, "%s", bv_s_acc);
    return;
}

void bf_s_titlecase(const char* bv_s_word_in, char* bcc_out) {
    char bv_s_word[256];
    snprintf(bv_s_word, sizeof(bv_s_word), "%s", bv_s_word_in);

    // Capitalise first letter, lowercase remainder.
    // UCASE$/LCASE$ aren't real MBASIC/BASCOM 2.00 builtins (verified
    // against a real IBM BASIC Compiler 2.00 under dosbox-x), so this
    // requires BASCAL's own com.bascal.stdlib implementations above.
    if ((-(((int)strlen(bv_s_word)) == 0))) {
        snprintf(bcc_out, 256, "%s", "");
        return;
    }
    char bt_s_6[256];
    bf_s_ucase_s(bcc_mid(bv_s_word, 1, 1), bt_s_6);
    char bt_s_7[256];
    bf_s_lcase_s(bcc_mid(bv_s_word, 2, 2147483647), bt_s_7);
    char bt_s_8[256];
    snprintf(bt_s_8, sizeof(bt_s_8), "%s%s", bt_s_6, bt_s_7);
    snprintf(bcc_out, 256, "%s", bt_s_8);
    return;
}

int bf_i_sumto(int bv_i_n) {
    int bv_i_acc = 0;
    int bv_i_i = 0;

    // i% and acc% are local to sumTo%.
    bv_i_acc = 0;
    int bt_lim_9 = bv_i_n;
    int bt_step_9 = 1;
    for (bv_i_i = 1; bt_step_9 >= 0 ? bv_i_i <= bt_lim_9 : bv_i_i >= bt_lim_9; bv_i_i += bt_step_9) {
        bv_i_acc = (bv_i_acc + bv_i_i);
    }
    return bv_i_acc;
}

int bf_i_productto(int bv_i_n) {
    int bv_i_acc = 0;
    int bv_i_i = 0;

    // i% and acc% here are independent of sumTo%'s i% and acc%.
    bv_i_acc = 1;
    int bt_lim_10 = bv_i_n;
    int bt_step_10 = 1;
    for (bv_i_i = 1; bt_step_10 >= 0 ? bv_i_i <= bt_lim_10 : bv_i_i >= bt_lim_10; bv_i_i += bt_step_10) {
        bv_i_acc = (bv_i_acc * bv_i_i);
    }
    return bv_i_acc;
}

int bf_i_addtototal(int bv_i_x) {
    bv_i_runningtotal = (bv_i_runningtotal + bv_i_x);
    return bv_i_runningtotal;
}

int main(void) {
    // Upper-cases self$. Not a real MBASIC/BASCOM 2.00 builtin -- verified
    // against a real IBM BASIC Compiler 2.00 under dosbox-x -- so BASCAL ships
    // its own. Declared as a scalar method (see GitHub issue #41 and
    // ltrim.bcl's own doc comment for the reasoning) -- ucase$(s$) still works
    // via ordinary-call syntax resolving to this same declaration.

    // Lower-cases self$. Not a real MBASIC/BASCOM 2.00 builtin -- verified
    // against a real IBM BASIC Compiler 2.00 under dosbox-x -- so BASCAL ships
    // its own. Declared as a scalar method (see GitHub issue #41 and
    // ltrim.bcl's own doc comment for the reasoning) -- lcase$(s$) still works
    // via ordinary-call syntax resolving to this same declaration.

    // Tutorial — Functions
    //
    // A BASCAL function is declared with FUNCTION ... END FUNCTION.
    // The function name carries the return type suffix.  Parameters
    // also carry type suffixes.  Every function must reach a RETURN.
    //
    // Variables declared inside a function are local by default: the compiler
    // prefixes them with the function name.  To access a global variable from
    // inside a function, declare it with:  global varname
    //
    // Functions cannot recurse, directly or indirectly (parameters would be
    // overwritten) -- the compiler checks the whole call graph and rejects
    // any cycle.  Use an explicit stack array for recursive algorithms.
    //
    // Scalar methods are typed functions with an implicit receiver.  Calls use
    // dot syntax and can chain: word$.left(1).ucase().  The existing titleCase$
    // function below demonstrates this form; methods transpile to ordinary
    // calls for both targets.


    // Integer arithmetic functions
    // a% -- first value to compare
    // b% -- second value to compare

    // a% -- first value to compare
    // b% -- second value to compare

    // value% -- number to constrain
    // lo%    -- lower bound, inclusive
    // hi%    -- upper bound, inclusive

    // String functions
    // text$ -- string to repeat
    // n%    -- number of times to repeat it

    // word$ -- string to title-case

    // Local variable scoping — each function has its own i% and acc%
    // n% -- upper bound of the sum, inclusive

    // n% -- upper bound of the product, inclusive

    // Global variable accessed inside a function with the global keyword
    bv_i_runningtotal = 0;

    // x% -- amount to add to the running total

    // --- Exercise the functions ---

    // print mixes string labels and numeric results directly with ;
    printf("max(4, 9) = %d\n", bf_i_max(4, 9));
    printf("min(4, 9) = %d\n", bf_i_min(4, 9));
    printf("clamp(15,1,10) = %d\n", bf_i_clamp(15, 1, 10));
    printf("clamp(-3,1,10) = %d\n", bf_i_clamp(-(3), 1, 10));
    printf("clamp(7,1,10)  = %d\n", bf_i_clamp(7, 1, 10));

    char bt_s_11[256];
    bf_s_repeat("ab", 4, bt_s_11);
    printf("%s\n", bt_s_11);
    char bt_s_12[256];
    bf_s_titlecase("bASCAL", bt_s_12);
    printf("%s\n", bt_s_12);

    // Functions chained in expressions
    bv_i_lo = bf_i_min(bf_i_max(0, -(5)), 100);
    printf("lo = %d\n", bv_i_lo);

    // Calling the same function twice — each result is captured separately
    char bt_s_13[256];
    bf_s_repeat("x", 3, bt_s_13);
    snprintf(bv_s_a, sizeof(bv_s_a), "%s", bt_s_13);
    char bt_s_14[256];
    bf_s_repeat("y", 2, bt_s_14);
    snprintf(bv_s_b, sizeof(bv_s_b), "%s", bt_s_14);
    printf("%s %s\n", bv_s_a, bv_s_b);

    // Local scoping: sumTo% and productTo% each use i% without conflict
    printf("sumTo(5)     = %d\n", bf_i_sumto(5));
    printf("productTo(5) = %d\n", bf_i_productto(5));

    // Global variable shared across calls
    bv_i_dummy = bf_i_addtototal(10);
    bv_i_dummy = bf_i_addtototal(5);
    printf("runningTotal = %d\n", bv_i_runningtotal);

    return 0;
}

static char* bcc_strbuf_take(void) {
    char* buf = bcc_strbuf[bcc_strbuf_next];
    bcc_strbuf_next = (bcc_strbuf_next + 1) % BCC_STRBUF_COUNT;
    return buf;
}

static const char* bcc_mid(const char* s, int start, int length) {
    char* out = bcc_strbuf_take();
    int len = (int)strlen(s);
    int from = start - 1;
    if (from < 0) from = 0;
    if (from > len) from = len;
    int avail = len - from;
    if (length < 0) length = 0;
    if (length > avail) length = avail;
    snprintf(out, 256, "%.*s", length, s + from);
    return out;
}

static const char* bcc_chr(int code) {
    char* out = bcc_strbuf_take();
    snprintf(out, 256, "%c", code);
    return out;
}

static const char* bcc_stri(int value) {
    char* out = bcc_strbuf_take();
    snprintf(out, 256, "% d", value);
    return out;
}

static const char* bcc_strd(double value) {
    char* out = bcc_strbuf_take();
    snprintf(out, 256, "% g", value);
    return out;
}


```



</details>

<!-- END generated tutorial source -->
