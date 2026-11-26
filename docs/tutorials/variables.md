[Home](../../) / [Tutorials](../) / Variables and Constants

Generated sources: [BCL](https://github.com/johnjoeallen/bascal/blob/main/tutorial/variables.bcl), [BASIC](https://github.com/johnjoeallen/bascal/blob/main/tutorial/variables.bas), [C](https://github.com/johnjoeallen/bascal/blob/main/tutorial/variables.c), and [JVM assembly](https://github.com/johnjoeallen/bascal/blob/main/tutorial/variables.j).

<div class="prose" markdown="1">

Every name ends with a type suffix that tells the runtime how to store the value: `%` 16-bit integer, `$` string, `!` single, `#` double, `&` long. Variables are global and spring into existence on first use — `dim` is only needed for arrays or when you want to be explicit. `const` names a value that can't change.

</div>

<div class="snippet" markdown="1">

### Constants and assignment

```bascal
const MAX_SCORE  = 100
const APP_NAME   = "Grade Checker"
const TAX_RATE   = 0.2

playerName$  = "Alice"
score%       = 87
```

</div>

<div class="snippet" markdown="1">

### print mixes types directly with ;

No str$() needed just to print a number next to a label.

```bascal
print "Score:       "; score%; "/ "; MAX_SCORE

// str$() is still available when you need to build a string value
let greeting$ = "Score is " + str$(score%)
```

</div>



[← Hello, World](hello.md)  ·  [Operators and Expressions →](arithmetic.md)


<!-- BEGIN generated tutorial source -->

<details class="source-embed" markdown="1">

<summary><code>tutorial/variables.bcl</code></summary>



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

const MAX_SCORE = 100
const PASS_MARK = 60
const APP_NAME = "Grade Checker"
const TAX_RATE = 0.2

// Variable assignment uses =
playerName$ = "Alice"
score% = 87
temperature! = 36.6

// print mixes strings and numbers directly with ; (no str$() needed)
print APP_NAME
print "Player:      "; playerName$
print "Score:       "; score%; "/ "; MAX_SCORE
print "Pass mark:   "; PASS_MARK
print "Temperature: "; temperature!
print "Tax rate:    "; TAX_RATE

// str$() is still available when you need to build a string value
let greeting$ = "Score is " + str$(score%)
print greeting$

end

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/variables.bas</code></summary>



```basic

10 ' BASCAL generated BASIC -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
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
210 CONSTMAXSCORE% = 100
220 CONSTPASSMARK% = 60
230 CONSTAPPNAME$ = "Grade Checker"
240 CONSTTAXRATE! = 0.2
250 ' Variable assignment uses =
260 playername$ = "Alice"
270 score% = 87
280 temperature! = 36.6
290 ' print mixes strings and numbers directly with ; (no str$() needed)
300 PRINT CONSTAPPNAME$
310 PRINT "Player:      "; playername$
320 PRINT "Score:       "; score%; "/ "; CONSTMAXSCORE%
330 PRINT "Pass mark:   "; CONSTPASSMARK%
340 PRINT "Temperature: "; temperature!
350 PRINT "Tax rate:    "; CONSTTAXRATE!
360 ' str$() is still available when you need to build a string value
370 greeting$ = "Score is " + STR$(score%)
380 PRINT greeting$
390 END

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/variables.c</code></summary>



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

static float bv_f_tax_rate = 0;
static float bv_f_temperature = 0;
static int bv_i_max_score = 0;
static int bv_i_pass_mark = 0;
static int bv_i_score = 0;
static char bv_s_app_name[256] = {0};
static char bv_s_greeting[256] = {0};
static char bv_s_playername[256] = {0};

int main(void) {
    // Tutorial — Variables and Constants
    //
    // Every name in BASCAL ends with a type suffix that tells the runtime
    // how to store the value:
    //
    // %   integer   — 16-bit signed, -32768 to 32767
    // $   string    — variable-length text
    // !   single    — 32-bit floating-point
    // #   double    — 64-bit floating-point
    // &   long      — 32-bit signed integer
    //
    // All variables are global.  They spring into existence on first use;
    // dim (or its synonym declare) is needed only for arrays or when you
    // want to be explicit -- declare tends to read better for a plain
    // scalar, dim for an array.
    //
    // const names a value that cannot change.  Use it for magic numbers
    // so the intent is clear and the value lives in one place.

    bv_i_max_score = 100;
    bv_i_pass_mark = 60;
    snprintf(bv_s_app_name, sizeof(bv_s_app_name), "%s", "Grade Checker");
    bv_f_tax_rate = 0.2;

    // Variable assignment uses =
    snprintf(bv_s_playername, sizeof(bv_s_playername), "%s", "Alice");
    bv_i_score = 87;
    bv_f_temperature = 36.6;

    // print mixes strings and numbers directly with ; (no str$() needed)
    printf("%s\n", bv_s_app_name);
    printf("Player:      %s\n", bv_s_playername);
    printf("Score:       %d/ %d\n", bv_i_score, bv_i_max_score);
    printf("Pass mark:   %d\n", bv_i_pass_mark);
    printf("Temperature: %g\n", bv_f_temperature);
    printf("Tax rate:    %g\n", bv_f_tax_rate);

    // str$() is still available when you need to build a string value
    char bt_s_0[256];
    snprintf(bt_s_0, sizeof(bt_s_0), "%s%s", "Score is ", bcc_stri(bv_i_score));
    snprintf(bv_s_greeting, sizeof(bv_s_greeting), "%s", bt_s_0);
    printf("%s\n", bv_s_greeting);

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

<summary><code>tutorial/variables.j</code></summary>



```basic

.class public Variables
.super java/lang/Object

.field public static g1 Ljava/lang/String;
.field public static g2 Ljava/lang/String;
.field public static g3 I
.field public static g4 I
.field public static g5 Ljava/lang/String;
.field public static g6 I
.field public static g7 D
.field public static g9 D
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

    ldc ""
    putstatic Variables/g1 Ljava/lang/String;
    ldc ""
    putstatic Variables/g2 Ljava/lang/String;
    iconst_0
    putstatic Variables/g3 I
    iconst_0
    putstatic Variables/g4 I
    ldc ""
    putstatic Variables/g5 Ljava/lang/String;
    iconst_0
    putstatic Variables/g6 I
    dconst_0
    putstatic Variables/g7 D
    dconst_0
    putstatic Variables/g9 D
    ; Tutorial — Variables and Constants
    ;
    ; Every name in BASCAL ends with a type suffix that tells the runtime
    ; how to store the value:
    ;
    ; %   integer   — 16-bit signed, -32768 to 32767
    ; $   string    — variable-length text
    ; !   single    — 32-bit floating-point
    ; #   double    — 64-bit floating-point
    ; &   long      — 32-bit signed integer
    ;
    ; All variables are global.  They spring into existence on first use;
    ; dim (or its synonym declare) is needed only for arrays or when you
    ; want to be explicit -- declare tends to read better for a plain
    ; scalar, dim for an array.
    ;
    ; const names a value that cannot change.  Use it for magic numbers
    ; so the intent is clear and the value lives in one place.
    ldc 100
    putstatic Variables/g3 I
    ldc 60
    putstatic Variables/g4 I
    ldc "Grade Checker"
    putstatic Variables/g1 Ljava/lang/String;
    ldc2_w 0.2
    putstatic Variables/g7 D
    ; Variable assignment uses =
    ldc "Alice"
    putstatic Variables/g5 Ljava/lang/String;
    ldc 87
    putstatic Variables/g6 I
    ldc2_w 36.6
    d2f
    f2d
    putstatic Variables/g9 D
    ; print mixes strings and numbers directly with ; (no str$() needed)
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Variables/g1 Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Player:      "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Variables/g5 Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Score:       "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Variables/g6 I
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "/ "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Variables/g3 I
    invokevirtual java/io/PrintStream/println (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Pass mark:   "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Variables/g4 I
    invokevirtual java/io/PrintStream/println (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Temperature: "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Variables/g9 D
    d2f
    f2d
    invokestatic Variables/bccStr (D)Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Tax rate:    "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Variables/g7 D
    d2f
    f2d
    invokestatic Variables/bccStr (D)Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ; str$() is still available when you need to build a string value
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "Score is "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    getstatic Variables/g6 I
    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    putstatic Variables/g2 Ljava/lang/String;
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Variables/g2 Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    return
.end method

```



</details>

<!-- END generated tutorial source -->
