[Home](../../) / [Tutorials](../) / Operators and Expressions

Generated sources: [BCL](https://github.com/johnjoeallen/bascal/blob/main/tutorial/arithmetic.bcl), [BASIC](https://github.com/johnjoeallen/bascal/blob/main/tutorial/arithmetic.bas), [C](https://github.com/johnjoeallen/bascal/blob/main/tutorial/arithmetic.c), and [JVM assembly](https://github.com/johnjoeallen/bascal/blob/main/tutorial/arithmetic.j).

<div class="prose" markdown="1">

Arithmetic (`+ - * / \ MOD ^`), comparison (`= <> < <= > >=`, returning `-1`/`0`), and logical (`AND OR NOT XOR`) all work as in classic BASIC — including the one sharp edge worth knowing up front: `NOT` is bitwise, so `NOT 1 = -2`, not `0`. Test for false with `(expr) = 0`, never `NOT expr`.

</div>

<div class="snippet" markdown="1">

### Integer division vs MOD

```bascal
print a%; "/ "; b%; "="; a% / b%      // 17 / 5 = 3  (truncates)
print a%; "\ "; b%; "="; a% \ b%     // 17 \ 5 = 3  (integer quotient)
print a%; "MOD "; b%; "="; a% mod b%  // 17 MOD 5 = 2  (remainder)
```

</div>

<div class="snippet" markdown="1">

### The NOT gotcha

This is why BASCAL's own if/while transpilation inverts conditions with (cond) = 0 instead of NOT — see [If Transpilation](../manual/generated-basic-shape.md#if-transpilation) in the manual.

```bascal
x% = 7
if x% > 0 and x% < 10 then
    print x%; "is in 1..9"
end if
print 6 xor 3; " (expect 5 -- 110 XOR 011 = 101)"
```

</div>



[← Variables and Constants](variables.md)  ·  [Conditions →](conditions.md)


<!-- BEGIN generated tutorial source -->

<details class="source-embed" markdown="1">

<summary><code>tutorial/arithmetic.bcl</code></summary>



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
print a%; "+ "; b%; "="; a% + b% // 17 + 5 = 22
print a%; "- "; b%; "="; a% - b% // 17 - 5 = 12
print a%; "* "; b%; "="; a% * b% // 17 * 5 = 85
print a%; "/ "; b%; "="; a% / b% // 17 / 5 = 3  (truncates)

/* Integer division and MOD */
print a%; "\ "; b%; "="; a% \ b% // 17 \ 5 = 3  (integer quotient)
print a%; "MOD "; b%; "="; a% mod b% // 17 MOD 5 = 2  (remainder)

/* Exponentiation — right-associative */
print "2 ^ 8 ="; 2 ^ 8 // 256
print "2 ^ 3 ^ 2 ="; 2 ^ 3 ^ 2 // 512  (= 2 ^ (3^2) = 2^9)

/* Precedence */
print 2 + 3 * 4; " (expect 14 — * before +)"
print (2 + 3) * 4; " (expect 20 — parens first)"

/* Comparison — -1 means true, 0 means false */
print 10 > 3; " (expect -1)"
print 10 < 3; " (expect  0)"
print 7 = 7; " (expect -1)"
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
print -n% // -42

end

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/arithmetic.bas</code></summary>



```basic

10 ' BASCAL generated BASIC -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
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
290 ' 17 + 5 = 22
300 PRINT a%; "- "; b%; "="; a% - b%
310 ' 17 - 5 = 12
320 PRINT a%; "* "; b%; "="; a% * b%
330 ' 17 * 5 = 85
340 PRINT a%; "/ "; b%; "="; a% / b%
350 ' 17 / 5 = 3  (truncates)
360 ' Integer division and MOD
370 PRINT a%; "\ "; b%; "="; a% \ b%
380 ' 17 \ 5 = 3  (integer quotient)
390 PRINT a%; "MOD "; b%; "="; a% MOD b%
400 ' 17 MOD 5 = 2  (remainder)
410 ' Exponentiation — right-associative
420 PRINT "2 ^ 8 ="; 2 ^ 8
430 ' 256
440 PRINT "2 ^ 3 ^ 2 ="; 2 ^ (3 ^ 2)
450 ' 512  (= 2 ^ (3^2) = 2^9)
460 ' Precedence
470 PRINT 2 + (3 * 4); " (expect 14 — * before +)"
480 PRINT (2 + 3) * 4; " (expect 20 — parens first)"
490 ' Comparison — -1 means true, 0 means false
500 PRINT 10 > 3; " (expect -1)"
510 PRINT 10 < 3; " (expect  0)"
520 PRINT 7 = 7; " (expect -1)"
530 PRINT 7 <> 8; " (expect -1)"
540 ' Logical — AND, OR, XOR are bitwise but work correctly with 0/-1 values
550 x% = 7
560 IF ((x% > 0) AND (x% < 10)) = 0 THEN GOTO 580
570     PRINT x%; "is in 1..9"
580 REM END IF
590 PRINT 6 XOR 3; " (expect 5 — 110 XOR 011 = 101)"
600 ' String concatenation
610 PRINT (("Hello" + ", ") + "World") + "!"
620 ' Unary negation
630 n% = 42
640 PRINT -n%
650 ' -42
660 END

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/arithmetic.c</code></summary>



```c

// BASCAL generated C -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
#include <stdio.h>
#include <math.h>

static int bv_i_a = 0;
static int bv_i_b = 0;
static int bv_i_n = 0;
static int bv_i_x = 0;

int main(void) {
    // Tutorial — Operators and Expressions
    //
    // Arithmetic:   +  -  *  /  \  MOD  ^
    // Comparison:   =  <>  <  <=  >  >=   (result: -1 true, 0 false)
    // Logical:      AND  OR  NOT  XOR  (bitwise — see note below)
    // String:       + concatenates strings
    //
    // Precedence (highest first):
    // ^                 exponentiation (right-associative)
    // unary -           negation
    // * /               multiply / divide
    // \                 integer (floor) division
    // MOD               modulus (remainder)
    // + -               add / subtract
    // = <> < <= > >=    comparison
    // NOT               bitwise NOT
    // AND               bitwise AND
    // OR                bitwise OR
    // XOR               bitwise XOR
    //
    // IMPORTANT: NOT is bitwise, so NOT 1 = -2, not 0.
    // Test for false with (expr) = 0, not NOT expr.

    // Arithmetic — mix labels and numbers with ;
    bv_i_a = 17;
    bv_i_b = 5;
    printf("%d+ %d=%d\n", bv_i_a, bv_i_b, (bv_i_a + bv_i_b));
    printf("%d- %d=%d\n", bv_i_a, bv_i_b, (bv_i_a - bv_i_b));
    printf("%d* %d=%d\n", bv_i_a, bv_i_b, (bv_i_a * bv_i_b));
    printf("%d/ %d=%g\n", bv_i_a, bv_i_b, ((double)bv_i_a / (double)bv_i_b));

    // Integer division and MOD
    printf("%d\\ %d=%d\n", bv_i_a, bv_i_b, ((int)((long)round((double)bv_i_a) / (long)round((double)bv_i_b))));
    printf("%dMOD %d=%d\n", bv_i_a, bv_i_b, ((int)((long)round((double)bv_i_a) % (long)round((double)bv_i_b))));

    // Exponentiation — right-associative
    printf("2 ^ 8 =%g\n", pow((double)2, (double)8));
    printf("2 ^ 3 ^ 2 =%g\n", pow((double)2, (double)pow((double)3, (double)2)));

    // Precedence
    printf("%d (expect 14 — * before +)\n", (2 + (3 * 4)));
    printf("%d (expect 20 — parens first)\n", ((2 + 3) * 4));

    // Comparison — -1 means true, 0 means false
    printf("%d (expect -1)\n", (-(10 > 3)));
    printf("%d (expect  0)\n", (-(10 < 3)));
    printf("%d (expect -1)\n", (-(7 == 7)));
    printf("%d (expect -1)\n", (-(7 != 8)));

    // Logical — AND, OR, XOR are bitwise but work correctly with 0/-1 values
    bv_i_x = 7;
    if (((int)((long)round((double)(-(bv_i_x > 0))) & (long)round((double)(-(bv_i_x < 10)))))) {
        printf("%dis in 1..9\n", bv_i_x);
    }
    printf("%d (expect 5 — 110 XOR 011 = 101)\n", ((int)((long)round((double)6) ^ (long)round((double)3))));

    // String concatenation
    char bt_s_0[256];
    snprintf(bt_s_0, sizeof(bt_s_0), "%s%s", "Hello", ", ");
    char bt_s_1[256];
    snprintf(bt_s_1, sizeof(bt_s_1), "%s%s", bt_s_0, "World");
    char bt_s_2[256];
    snprintf(bt_s_2, sizeof(bt_s_2), "%s%s", bt_s_1, "!");
    printf("%s\n", bt_s_2);

    // Unary negation
    bv_i_n = 42;
    printf("%d\n", -(bv_i_n));

    return 0;
}

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/arithmetic.j</code></summary>



```basic

.class public Arithmetic
.super java/lang/Object

.field public static g1 I
.field public static g2 I
.field public static g3 I
.field public static g4 I
.method public static bccStr : (D)Ljava/lang/String;

    new java/math/BigDecimal
    dup
    dload 0
    invokespecial java/math/BigDecimal/<init> (D)V
    new java/math/MathContext
    dup
    bipush 6
    invokespecial java/math/MathContext/<init> (I)V
    invokevirtual java/math/BigDecimal/round (Ljava/math/MathContext;)Ljava/math/BigDecimal;
    invokevirtual java/math/BigDecimal/stripTrailingZeros ()Ljava/math/BigDecimal;
    invokevirtual java/math/BigDecimal/toPlainString ()Ljava/lang/String;
    areturn
.end method

.method public static main : ([Ljava/lang/String;)V

    iconst_0
    putstatic Arithmetic/g1 I
    iconst_0
    putstatic Arithmetic/g2 I
    iconst_0
    putstatic Arithmetic/g3 I
    iconst_0
    putstatic Arithmetic/g4 I
    ; Tutorial — Operators and Expressions
    ;
    ; Arithmetic:   +  -  *  /  \  MOD  ^
    ; Comparison:   =  <>  <  <=  >  >=   (result: -1 true, 0 false)
    ; Logical:      AND  OR  NOT  XOR  (bitwise — see note below)
    ; String:       + concatenates strings
    ;
    ; Precedence (highest first):
    ; ^                 exponentiation (right-associative)
    ; unary -           negation
    ; * /               multiply / divide
    ; \                 integer (floor) division
    ; MOD               modulus (remainder)
    ; + -               add / subtract
    ; = <> < <= > >=    comparison
    ; NOT               bitwise NOT
    ; AND               bitwise AND
    ; OR                bitwise OR
    ; XOR               bitwise XOR
    ;
    ; IMPORTANT: NOT is bitwise, so NOT 1 = -2, not 0.
    ; Test for false with (expr) = 0, not NOT expr.
    ; Arithmetic — mix labels and numbers with ;
    ldc 17
    putstatic Arithmetic/g1 I
    ldc 5
    putstatic Arithmetic/g2 I
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Arithmetic/g1 I
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "+ "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Arithmetic/g2 I
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "="
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Arithmetic/g1 I
    getstatic Arithmetic/g2 I
    iadd
    invokevirtual java/io/PrintStream/println (I)V
    ; 17 + 5 = 22
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Arithmetic/g1 I
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "- "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Arithmetic/g2 I
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "="
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Arithmetic/g1 I
    getstatic Arithmetic/g2 I
    isub
    invokevirtual java/io/PrintStream/println (I)V
    ; 17 - 5 = 12
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Arithmetic/g1 I
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "* "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Arithmetic/g2 I
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "="
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Arithmetic/g1 I
    getstatic Arithmetic/g2 I
    imul
    invokevirtual java/io/PrintStream/println (I)V
    ; 17 * 5 = 85
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Arithmetic/g1 I
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "/ "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Arithmetic/g2 I
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "="
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Arithmetic/g1 I
    i2d
    getstatic Arithmetic/g2 I
    i2d
    ddiv
    d2f
    f2d
    invokestatic Arithmetic/bccStr (D)Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ; 17 / 5 = 3  (truncates)
    ; Integer division and MOD
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Arithmetic/g1 I
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "\\ "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Arithmetic/g2 I
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "="
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Arithmetic/g1 I
    i2d
    dup2
    ldc2_w 0.5
    dup2_x2
    pop2
    invokestatic java/lang/Math/copySign (DD)D
    dadd
    d2l
    getstatic Arithmetic/g2 I
    i2d
    dup2
    ldc2_w 0.5
    dup2_x2
    pop2
    invokestatic java/lang/Math/copySign (DD)D
    dadd
    d2l
    ldiv
    invokevirtual java/io/PrintStream/println (J)V
    ; 17 \ 5 = 3  (integer quotient)
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Arithmetic/g1 I
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "MOD "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Arithmetic/g2 I
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "="
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Arithmetic/g1 I
    i2d
    dup2
    ldc2_w 0.5
    dup2_x2
    pop2
    invokestatic java/lang/Math/copySign (DD)D
    dadd
    d2l
    getstatic Arithmetic/g2 I
    i2d
    dup2
    ldc2_w 0.5
    dup2_x2
    pop2
    invokestatic java/lang/Math/copySign (DD)D
    dadd
    d2l
    lrem
    invokevirtual java/io/PrintStream/println (J)V
    ; 17 MOD 5 = 2  (remainder)
    ; Exponentiation — right-associative
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "2 ^ 8 ="
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc 2
    i2d
    ldc 8
    i2d
    invokestatic java/lang/Math/pow (DD)D
    invokestatic Arithmetic/bccStr (D)Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ; 256
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "2 ^ 3 ^ 2 ="
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc 2
    i2d
    ldc 3
    i2d
    ldc 2
    i2d
    invokestatic java/lang/Math/pow (DD)D
    invokestatic java/lang/Math/pow (DD)D
    invokestatic Arithmetic/bccStr (D)Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ; 512  (= 2 ^ (3^2) = 2^9)
    ; Precedence
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc 2
    ldc 3
    ldc 4
    imul
    iadd
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc " (expect 14 — * before +)"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc 2
    ldc 3
    iadd
    ldc 4
    imul
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc " (expect 20 — parens first)"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ; Comparison — -1 means true, 0 means false
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc 10
    ldc 3
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc " (expect -1)"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc 10
    ldc 3
    invokestatic java/lang/Integer/compare (II)I
    bipush 31
    ishr
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc " (expect  0)"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc 7
    ldc 7
    invokestatic java/lang/Integer/compare (II)I
    dup
    ineg
    ior
    bipush 31
    iushr
    iconst_1
    ixor
    ineg
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc " (expect -1)"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc 7
    ldc 8
    invokestatic java/lang/Integer/compare (II)I
    dup
    ineg
    ior
    bipush 31
    iushr
    ineg
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc " (expect -1)"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ; Logical — AND, OR, XOR are bitwise but work correctly with 0/-1 values
    ldc 7
    putstatic Arithmetic/g4 I
    getstatic Arithmetic/g4 I
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    i2d
    dup2
    ldc2_w 0.5
    dup2_x2
    pop2
    invokestatic java/lang/Math/copySign (DD)D
    dadd
    d2l
    getstatic Arithmetic/g4 I
    ldc 10
    invokestatic java/lang/Integer/compare (II)I
    bipush 31
    ishr
    i2d
    dup2
    ldc2_w 0.5
    dup2_x2
    pop2
    invokestatic java/lang/Math/copySign (DD)D
    dadd
    d2l
    land
    lconst_0
    lcmp
    ifeq L_condition_0
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Arithmetic/g4 I
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "is in 1..9"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
L_condition_0:
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc 6
    i2d
    dup2
    ldc2_w 0.5
    dup2_x2
    pop2
    invokestatic java/lang/Math/copySign (DD)D
    dadd
    d2l
    ldc 3
    i2d
    dup2
    ldc2_w 0.5
    dup2_x2
    pop2
    invokestatic java/lang/Math/copySign (DD)D
    dadd
    d2l
    lxor
    invokevirtual java/io/PrintStream/print (J)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc " (expect 5 — 110 XOR 011 = 101)"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ; String concatenation
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "Hello"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc ", "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc "World"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc "!"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ; Unary negation
    ldc 42
    putstatic Arithmetic/g3 I
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Arithmetic/g3 I
    ineg
    invokevirtual java/io/PrintStream/println (I)V
    ; -42
    return
.end method

```



</details>

<!-- END generated tutorial source -->
