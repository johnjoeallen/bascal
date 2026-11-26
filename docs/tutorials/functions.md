[Home](../../) / [Tutorials](../) / Functions

<div class="prose" markdown="1">

Generated sources: [BCL](https://github.com/johnjoeallen/bascal/blob/main/tutorial/functions.bcl), [BASIC](https://github.com/johnjoeallen/bascal/blob/main/tutorial/functions.bas), [C](https://github.com/johnjoeallen/bascal/blob/main/tutorial/functions.c), and [JVM assembly](https://github.com/johnjoeallen/bascal/blob/main/tutorial/functions.j).

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
method shout[string]()
    return self$ + "!"
end method

print "  hello  ".ltrim().rtrim().ucase().shout()
```

Built-in methods such as `left`, `mid`, `len`, `abs`, and `sin` use the same syntax. Methods transpile to ordinary typed calls for both targets; they do not create runtime objects.

</div>



[← Select Case](select_case.md)  ·  [Arrays →](arrays.md)


<!-- BEGIN generated tutorial source -->

<details class="source-embed" markdown="1">

<summary><code>tutorial/functions.bcl</code></summary>



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
print "max(4, 9) = "; max%(4, 9) // 9
print "min(4, 9) = "; min%(4, 9) // 4
print "clamp(15,1,10) = "; clamp%(15, 1, 10) // 10
print "clamp(-3,1,10) = "; clamp%(-3, 1, 10) // 1
print "clamp(7,1,10)  = "; clamp%(7, 1, 10) // 7

print repeat$("ab", 4) // abababab
print titleCase$("bASCAL") // Bascal

/* Functions chained in expressions */
lo% = min%(max%(0, -5), 100) // max(0,-5)=0, min(0,100)=0
print "lo = "; lo%

/* Calling the same function twice — each result is captured separately */
a$ = repeat$("x", 3)
b$ = repeat$("y", 2)
print a$; " "; b$ // xxx yy

/* Local scoping: sumTo% and productTo% each use i% without conflict */
print "sumTo(5)     = "; sumTo%(5) // 15
print "productTo(5) = "; productTo%(5) // 120

/* Global variable shared across calls */
dummy% = addToTotal%(10)
dummy% = addToTotal%(5)
print "runningTotal = "; runningTotal% // 15

end

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/functions.bas</code></summary>



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
510 BCCT1$ = "max(4, 9) = "
520 maxA0% = 4
530 maxB0% = 9
540 GOSUB 1780
550 BCCT2% = maxResult0%
560 BCCT3% = BCCT2%
570 PRINT BCCT1$; BCCT3%
580 ' 9
590 BCCT4$ = "min(4, 9) = "
600 minA0% = 4
610 minB0% = 9
620 GOSUB 1880
630 BCCT5% = minResult0%
640 BCCT6% = BCCT5%
650 PRINT BCCT4$; BCCT6%
660 ' 4
670 BCCT7$ = "clamp(15,1,10) = "
680 clampValue0% = 15
690 clampLo0% = 1
700 clampHi0% = 10
710 GOSUB 1980
720 BCCT8% = clampResult0%
730 BCCT9% = BCCT8%
740 PRINT BCCT7$; BCCT9%
750 ' 10
760 BCCT10$ = "clamp(-3,1,10) = "
770 clampValue0% = -3
780 clampLo0% = 1
790 clampHi0% = 10
800 GOSUB 1980
810 BCCT11% = clampResult0%
820 BCCT12% = BCCT11%
830 PRINT BCCT10$; BCCT12%
840 ' 1
850 BCCT13$ = "clamp(7,1,10)  = "
860 clampValue0% = 7
870 clampLo0% = 1
880 clampHi0% = 10
890 GOSUB 1980
900 BCCT14% = clampResult0%
910 BCCT15% = BCCT14%
920 PRINT BCCT13$; BCCT15%
930 ' 7
940 repeatText0$ = "ab"
950 repeatN0% = 4
960 GOSUB 2120
970 PRINT repeatResult0$
980 ' abababab
990 titlecaseWord0$ = "bASCAL"
1000 GOSUB 2210
1010 PRINT titlecaseResult0$
1020 ' Bascal
1030 ' Functions chained in expressions
1040 maxA0% = 0
1050 maxB0% = -5
1060 GOSUB 1780
1070 BCCT16% = maxResult0%
1080 BCCT17% = BCCT16%
1090 minA0% = BCCT17%
1100 minB0% = 100
1110 GOSUB 1880
1120 lo% = minResult0%
1130 ' max(0,-5)=0, min(0,100)=0
1140 PRINT "lo = "; lo%
1150 ' Calling the same function twice — each result is captured separately
1160 repeatText0$ = "x"
1170 repeatN0% = 3
1180 GOSUB 2120
1190 a$ = repeatResult0$
1200 repeatText0$ = "y"
1210 repeatN0% = 2
1220 GOSUB 2120
1230 b$ = repeatResult0$
1240 PRINT a$; " "; b$
1250 ' xxx yy
1260 ' Local scoping: sumTo% and productTo% each use i% without conflict
1270 BCCT18$ = "sumTo(5)     = "
1280 sumtoN0% = 5
1290 GOSUB 2450
1300 BCCT19% = sumtoResult0%
1310 BCCT20% = BCCT19%
1320 PRINT BCCT18$; BCCT20%
1330 ' 15
1340 BCCT21$ = "productTo(5) = "
1350 producttoN0% = 5
1360 GOSUB 2540
1370 BCCT22% = producttoResult0%
1380 BCCT23% = BCCT22%
1390 PRINT BCCT21$; BCCT23%
1400 ' 120
1410 ' Global variable shared across calls
1420 addtototalX0% = 10
1430 GOSUB 2630
1440 dummy% = addtototalResult0%
1450 addtototalX0% = 5
1460 GOSUB 2630
1470 dummy% = addtototalResult0%
1480 PRINT "runningTotal = "; runningtotal%
1490 ' 15
1500 END

1510 ' function ucase$(self$)
1520     ucaseOut0$ = ""
1530     FOR ucaseI0% = 1 TO LEN(ucaseSelf0$)
1540         ucaseC0% = ASC(MID$(ucaseSelf0$, ucaseI0%, 1))
1550         IF (ucaseC0% >= 97) = 0 THEN GOTO 1580
1560         IF (ucaseC0% <= 122) = 0 THEN GOTO 1580
1570             ucaseC0% = ucaseC0% - 32
1580         REM END IF
1590         ucaseOut0$ = ucaseOut0$ + CHR$(ucaseC0%)
1600     NEXT ucaseI0%
1610     ucaseResult0$ = ucaseOut0$
1620     RETURN
1630 ' end function ucase$

1640 ' function lcase$(self$)
1650     lcaseOut0$ = ""
1660     FOR lcaseI0% = 1 TO LEN(lcaseSelf0$)
1670         lcaseC0% = ASC(MID$(lcaseSelf0$, lcaseI0%, 1))
1680         IF (lcaseC0% >= 65) = 0 THEN GOTO 1710
1690         IF (lcaseC0% <= 90) = 0 THEN GOTO 1710
1700             lcaseC0% = lcaseC0% + 32
1710         REM END IF
1720         lcaseOut0$ = lcaseOut0$ + CHR$(lcaseC0%)
1730     NEXT lcaseI0%
1740     lcaseResult0$ = lcaseOut0$
1750     RETURN
1760 ' end function lcase$

1770 ' function max%(a%, b%)
1780     IF (maxA0% > maxB0%) = 0 THEN GOTO 1820
1790         maxResult0% = maxA0%
1800         RETURN
1810     GOTO 1840
1820         maxResult0% = maxB0%
1830         RETURN
1840     REM END IF
1850     RETURN
1860 ' end function max%

1870 ' function min%(a%, b%)
1880     IF (minA0% < minB0%) = 0 THEN GOTO 1920
1890         minResult0% = minA0%
1900         RETURN
1910     GOTO 1940
1920         minResult0% = minB0%
1930         RETURN
1940     REM END IF
1950     RETURN
1960 ' end function min%

1970 ' function clamp%(value%, lo%, hi%)
1980     ' Constrain value to [lo, hi].
1990     minA0% = clampValue0%
2000     minB0% = clampHi0%
2010     GOSUB 1880
2020     BCCT32% = minResult0%
2030     BCCT33% = BCCT32%
2040     maxA0% = clampLo0%
2050     maxB0% = BCCT33%
2060     GOSUB 1780
2070     BCCT34% = maxResult0%
2080     clampResult0% = BCCT34%
2090     RETURN
2100 ' end function clamp%

2110 ' function repeat$(text$, n%)
2120     ' Concatenate text$ with itself n times.
2130     repeatAcc0$ = ""
2140     FOR repeatI0% = 1 TO repeatN0%
2150         repeatAcc0$ = repeatAcc0$ + repeatText0$
2160     NEXT repeatI0%
2170     repeatResult0$ = repeatAcc0$
2180     RETURN
2190 ' end function repeat$

2200 ' function titlecase$(word$)
2210     ' Capitalise first letter, lowercase remainder.
2220     ' UCASE$/LCASE$ aren't real MBASIC/BASCOM 2.00 builtins (verified
2230     ' against a real IBM BASIC Compiler 2.00 under dosbox-x), so this
2240     ' requires BASCAL's own com.bascal.stdlib implementations above.
2250     IF (LEN(titlecaseWord0$) = 0) = 0 THEN GOTO 2280
2260         titlecaseResult0$ = ""
2270         RETURN
2280     REM END IF
2290     BCCT37$ = titlecaseWord0$
2300     BCCT38% = 1
2310     BCCT39$ = LEFT$(BCCT37$, BCCT38%)
2320     ucaseSelf0$ = BCCT39$
2330     GOSUB 1520
2340     BCCT40$ = ucaseResult0$
2350     BCCT41$ = titlecaseWord0$
2360     BCCT42% = 2
2370     BCCT43$ = MID$(BCCT41$, BCCT42%)
2380     lcaseSelf0$ = BCCT43$
2390     GOSUB 1650
2400     BCCT44$ = lcaseResult0$
2410     titlecaseResult0$ = BCCT40$ + BCCT44$
2420     RETURN
2430 ' end function titlecase$

2440 ' function sumto%(n%)
2450     ' i% and acc% are local to sumTo%.
2460     sumtoAcc0% = 0
2470     FOR sumtoI0% = 1 TO sumtoN0%
2480         sumtoAcc0% = sumtoAcc0% + sumtoI0%
2490     NEXT sumtoI0%
2500     sumtoResult0% = sumtoAcc0%
2510     RETURN
2520 ' end function sumto%

2530 ' function productto%(n%)
2540     ' i% and acc% here are independent of sumTo%'s i% and acc%.
2550     producttoAcc0% = 1
2560     FOR producttoI0% = 1 TO producttoN0%
2570         producttoAcc0% = producttoAcc0% * producttoI0%
2580     NEXT producttoI0%
2590     producttoResult0% = producttoAcc0%
2600     RETURN
2610 ' end function productto%

2620 ' function addtototal%(x%)
2630     runningtotal% = runningtotal% + addtototalX0%
2640     addtototalResult0% = runningtotal%
2650     RETURN
2660 ' end function addtototal%

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/functions.c</code></summary>



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
    int bt_start_0 = 1;
    int bt_lim_0 = ((int)strlen(bv_s_self));
    int bt_step_0 = 1;
    for (bv_i_i = bt_start_0; bt_step_0 >= 0 ? bv_i_i <= bt_lim_0 : bv_i_i >= bt_lim_0; bv_i_i += bt_step_0) {
        bv_i_c = ((int)(unsigned char)bcc_mid(bv_s_self, bv_i_i, 1)[0]);
        if ((-(((-(bv_i_c >= 97))) != 0 && ((-(bv_i_c <= 122))) != 0))) {
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
    int bt_start_2 = 1;
    int bt_lim_2 = ((int)strlen(bv_s_self));
    int bt_step_2 = 1;
    for (bv_i_i = bt_start_2; bt_step_2 >= 0 ? bv_i_i <= bt_lim_2 : bv_i_i >= bt_lim_2; bv_i_i += bt_step_2) {
        bv_i_c = ((int)(unsigned char)bcc_mid(bv_s_self, bv_i_i, 1)[0]);
        if ((-(((-(bv_i_c >= 65))) != 0 && ((-(bv_i_c <= 90))) != 0))) {
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

<details class="source-embed" markdown="1">

<summary><code>tutorial/functions.j</code></summary>



```basic

.class public Functions
.super java/lang/Object

.field public static g1 Ljava/lang/String;
.field public static g2 Ljava/lang/String;
.field public static g3 I
.field public static g4 I
.field public static g5 I
.method public static ucase : (Ljava/lang/String;)Ljava/lang/String;

    iconst_0
    istore 1
    iconst_0
    istore 2
    ldc ""
    astore 3
    ldc ""
    astore 3
    ldc 1
    istore 4
    aload 0
    invokevirtual java/lang/String/length ()I
    istore 5
    iload 4
    istore 2
L_for_0_top:
    iload 2
    iload 5
    if_icmpgt L_for_0_end
    aload 0
    iload 2
    iconst_1
    isub
    dup
    ldc 1
    iadd
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    iconst_0
    invokevirtual java/lang/String/charAt (I)C
    istore 1
    iload 1
    ldc 97
    invokestatic java/lang/Integer/compare (II)I
    ineg
    iconst_1
    isub
    bipush 31
    ishr
    ifeq L_condition_1
    iload 1
    ldc 122
    invokestatic java/lang/Integer/compare (II)I
    iconst_1
    isub
    bipush 31
    ishr
    ifne L_condition_3
    iconst_0
    goto L_condition_4
L_condition_3:
    iconst_m1
L_condition_4:
    goto L_condition_2
L_condition_1:
    iconst_0
L_condition_2:
    ifeq L_condition_5
    iload 1
    ldc 32
    isub
    istore 1
L_condition_5:
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    aload 3
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    iload 1
    i2c
    invokestatic java/lang/String/valueOf (C)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    astore 3
L_for_0_continue:
    iload 2
    ldc 1
    iadd
    istore 2
    goto L_for_0_top
L_for_0_end:
    aload 3
    areturn
    ldc ""
    areturn
.end method

.method public static lcase : (Ljava/lang/String;)Ljava/lang/String;

    iconst_0
    istore 1
    iconst_0
    istore 2
    ldc ""
    astore 3
    ldc ""
    astore 3
    ldc 1
    istore 4
    aload 0
    invokevirtual java/lang/String/length ()I
    istore 5
    iload 4
    istore 2
L_for_0_top:
    iload 2
    iload 5
    if_icmpgt L_for_0_end
    aload 0
    iload 2
    iconst_1
    isub
    dup
    ldc 1
    iadd
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    iconst_0
    invokevirtual java/lang/String/charAt (I)C
    istore 1
    iload 1
    ldc 65
    invokestatic java/lang/Integer/compare (II)I
    ineg
    iconst_1
    isub
    bipush 31
    ishr
    ifeq L_condition_1
    iload 1
    ldc 90
    invokestatic java/lang/Integer/compare (II)I
    iconst_1
    isub
    bipush 31
    ishr
    ifne L_condition_3
    iconst_0
    goto L_condition_4
L_condition_3:
    iconst_m1
L_condition_4:
    goto L_condition_2
L_condition_1:
    iconst_0
L_condition_2:
    ifeq L_condition_5
    iload 1
    ldc 32
    iadd
    istore 1
L_condition_5:
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    aload 3
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    iload 1
    i2c
    invokestatic java/lang/String/valueOf (C)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    astore 3
L_for_0_continue:
    iload 2
    ldc 1
    iadd
    istore 2
    goto L_for_0_top
L_for_0_end:
    aload 3
    areturn
    ldc ""
    areturn
.end method

.method public static max : (II)I

    iload 0
    iload 1
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_condition_0
    iload 0
    ireturn
    goto L_condition_1
L_condition_0:
    iload 1
    ireturn
L_condition_1:
    iconst_0
    ireturn
.end method

.method public static min : (II)I

    iload 0
    iload 1
    invokestatic java/lang/Integer/compare (II)I
    bipush 31
    ishr
    ifeq L_condition_0
    iload 0
    ireturn
    goto L_condition_1
L_condition_0:
    iload 1
    ireturn
L_condition_1:
    iconst_0
    ireturn
.end method

.method public static clamp : (III)I

    ; Constrain value to [lo, hi].
    iload 1
    iload 0
    iload 2
    invokestatic Functions/min (II)I
    invokestatic Functions/max (II)I
    ireturn
    iconst_0
    ireturn
.end method

.method public static repeat : (Ljava/lang/String;I)Ljava/lang/String;

    ldc ""
    astore 2
    iconst_0
    istore 3
    ; Concatenate text$ with itself n times.
    ldc ""
    astore 2
    ldc 1
    istore 4
    iload 1
    istore 5
    iload 4
    istore 3
L_for_0_top:
    iload 3
    iload 5
    if_icmpgt L_for_0_end
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    aload 2
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    aload 0
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    astore 2
L_for_0_continue:
    iload 3
    ldc 1
    iadd
    istore 3
    goto L_for_0_top
L_for_0_end:
    aload 2
    areturn
    ldc ""
    areturn
.end method

.method public static titleCase : (Ljava/lang/String;)Ljava/lang/String;

    ; Capitalise first letter, lowercase remainder.
    ; UCASE$/LCASE$ aren't real MBASIC/BASCOM 2.00 builtins (verified
    ; against a real IBM BASIC Compiler 2.00 under dosbox-x), so this
    ; requires BASCAL's own com.bascal.stdlib implementations above.
    aload 0
    invokevirtual java/lang/String/length ()I
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    dup
    ineg
    ior
    bipush 31
    iushr
    iconst_1
    ixor
    ineg
    ifeq L_condition_0
    ldc ""
    areturn
L_condition_0:
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    aload 0
    iconst_0
    ldc 1
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    invokestatic Functions/ucase (Ljava/lang/String;)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    aload 0
    ldc 2
    iconst_1
    isub
    invokevirtual java/lang/String/substring (I)Ljava/lang/String;
    invokestatic Functions/lcase (Ljava/lang/String;)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    areturn
    ldc ""
    areturn
.end method

.method public static sumTo : (I)I

    iconst_0
    istore 1
    iconst_0
    istore 2
    ; i% and acc% are local to sumTo%.
    ldc 0
    istore 1
    ldc 1
    istore 3
    iload 0
    istore 4
    iload 3
    istore 2
L_for_0_top:
    iload 2
    iload 4
    if_icmpgt L_for_0_end
    iload 1
    iload 2
    iadd
    istore 1
L_for_0_continue:
    iload 2
    ldc 1
    iadd
    istore 2
    goto L_for_0_top
L_for_0_end:
    iload 1
    ireturn
    iconst_0
    ireturn
.end method

.method public static productTo : (I)I

    iconst_0
    istore 1
    iconst_0
    istore 2
    ; i% and acc% here are independent of sumTo%'s i% and acc%.
    ldc 1
    istore 1
    ldc 1
    istore 3
    iload 0
    istore 4
    iload 3
    istore 2
L_for_0_top:
    iload 2
    iload 4
    if_icmpgt L_for_0_end
    iload 1
    iload 2
    imul
    istore 1
L_for_0_continue:
    iload 2
    ldc 1
    iadd
    istore 2
    goto L_for_0_top
L_for_0_end:
    iload 1
    ireturn
    iconst_0
    ireturn
.end method

.method public static addToTotal : (I)I

    getstatic Functions/g5 I
    iload 0
    iadd
    putstatic Functions/g5 I
    getstatic Functions/g5 I
    ireturn
    iconst_0
    ireturn
.end method

.method public static main : ([Ljava/lang/String;)V

    ldc ""
    putstatic Functions/g1 Ljava/lang/String;
    ldc ""
    putstatic Functions/g2 Ljava/lang/String;
    iconst_0
    putstatic Functions/g3 I
    iconst_0
    putstatic Functions/g4 I
    iconst_0
    putstatic Functions/g5 I
    ; Upper-cases self$. Not a real MBASIC/BASCOM 2.00 builtin -- verified
    ; against a real IBM BASIC Compiler 2.00 under dosbox-x -- so BASCAL ships
    ; its own. Declared as a scalar method (see GitHub issue #41 and
    ; ltrim.bcl's own doc comment for the reasoning) -- ucase$(s$) still works
    ; via ordinary-call syntax resolving to this same declaration.
    ; Lower-cases self$. Not a real MBASIC/BASCOM 2.00 builtin -- verified
    ; against a real IBM BASIC Compiler 2.00 under dosbox-x -- so BASCAL ships
    ; its own. Declared as a scalar method (see GitHub issue #41 and
    ; ltrim.bcl's own doc comment for the reasoning) -- lcase$(s$) still works
    ; via ordinary-call syntax resolving to this same declaration.
    ; Tutorial — Functions
    ;
    ; A BASCAL function is declared with FUNCTION ... END FUNCTION.
    ; The function name carries the return type suffix.  Parameters
    ; also carry type suffixes.  Every function must reach a RETURN.
    ;
    ; Variables declared inside a function are local by default: the compiler
    ; prefixes them with the function name.  To access a global variable from
    ; inside a function, declare it with:  global varname
    ;
    ; Functions cannot recurse, directly or indirectly (parameters would be
    ; overwritten) -- the compiler checks the whole call graph and rejects
    ; any cycle.  Use an explicit stack array for recursive algorithms.
    ;
    ; Scalar methods are typed functions with an implicit receiver.  Calls use
    ; dot syntax and can chain: word$.left(1).ucase().  The existing titleCase$
    ; function below demonstrates this form; methods transpile to ordinary
    ; calls for both targets.
    ; Integer arithmetic functions
    ; a% -- first value to compare
    ; b% -- second value to compare
    ; a% -- first value to compare
    ; b% -- second value to compare
    ; value% -- number to constrain
    ; lo%    -- lower bound, inclusive
    ; hi%    -- upper bound, inclusive
    ; String functions
    ; text$ -- string to repeat
    ; n%    -- number of times to repeat it
    ; word$ -- string to title-case
    ; Local variable scoping — each function has its own i% and acc%
    ; n% -- upper bound of the sum, inclusive
    ; n% -- upper bound of the product, inclusive
    ; Global variable accessed inside a function with the global keyword
    ldc 0
    putstatic Functions/g5 I
    ; x% -- amount to add to the running total
    ; --- Exercise the functions ---
    ; print mixes string labels and numeric results directly with ;
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "max(4, 9) = "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc 4
    ldc 9
    invokestatic Functions/max (II)I
    invokevirtual java/io/PrintStream/println (I)V
    ; 9
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "min(4, 9) = "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc 4
    ldc 9
    invokestatic Functions/min (II)I
    invokevirtual java/io/PrintStream/println (I)V
    ; 4
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "clamp(15,1,10) = "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc 15
    ldc 1
    ldc 10
    invokestatic Functions/clamp (III)I
    invokevirtual java/io/PrintStream/println (I)V
    ; 10
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "clamp(-3,1,10) = "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc 3
    ineg
    ldc 1
    ldc 10
    invokestatic Functions/clamp (III)I
    invokevirtual java/io/PrintStream/println (I)V
    ; 1
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "clamp(7,1,10)  = "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc 7
    ldc 1
    ldc 10
    invokestatic Functions/clamp (III)I
    invokevirtual java/io/PrintStream/println (I)V
    ; 7
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "ab"
    ldc 4
    invokestatic Functions/repeat (Ljava/lang/String;I)Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ; abababab
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "bASCAL"
    invokestatic Functions/titleCase (Ljava/lang/String;)Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ; Bascal
    ; Functions chained in expressions
    ldc 0
    ldc 5
    ineg
    invokestatic Functions/max (II)I
    ldc 100
    invokestatic Functions/min (II)I
    putstatic Functions/g4 I
    ; max(0,-5)=0, min(0,100)=0
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "lo = "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Functions/g4 I
    invokevirtual java/io/PrintStream/println (I)V
    ; Calling the same function twice — each result is captured separately
    ldc "x"
    ldc 3
    invokestatic Functions/repeat (Ljava/lang/String;I)Ljava/lang/String;
    putstatic Functions/g1 Ljava/lang/String;
    ldc "y"
    ldc 2
    invokestatic Functions/repeat (Ljava/lang/String;I)Ljava/lang/String;
    putstatic Functions/g2 Ljava/lang/String;
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Functions/g1 Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc " "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Functions/g2 Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ; xxx yy
    ; Local scoping: sumTo% and productTo% each use i% without conflict
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "sumTo(5)     = "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc 5
    invokestatic Functions/sumTo (I)I
    invokevirtual java/io/PrintStream/println (I)V
    ; 15
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "productTo(5) = "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc 5
    invokestatic Functions/productTo (I)I
    invokevirtual java/io/PrintStream/println (I)V
    ; 120
    ; Global variable shared across calls
    ldc 10
    invokestatic Functions/addToTotal (I)I
    putstatic Functions/g3 I
    ldc 5
    invokestatic Functions/addToTotal (I)I
    putstatic Functions/g3 I
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "runningTotal = "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Functions/g5 I
    invokevirtual java/io/PrintStream/println (I)V
    ; 15
    return
.end method

```



</details>

<!-- END generated tutorial source -->
