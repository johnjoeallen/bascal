[Home](../../) / [Tutorials](../) / Conditions

Generated sources: [BCL](https://github.com/johnjoeallen/bascal/blob/main/tutorial/conditions.bcl), [BASIC](https://github.com/johnjoeallen/bascal/blob/main/tutorial/conditions.bas), [C](https://github.com/johnjoeallen/bascal/blob/main/tutorial/conditions.c), and [JVM assembly](https://github.com/johnjoeallen/bascal/blob/main/tutorial/conditions.j).

<div class="prose" markdown="1">

BASCAL supports multi-line block `if` statements: `if cond then ... end if`, with optional `else` and any number of `elseif` clauses. It also supports classic BASIC's single-line form — a statement directly after `then` on the same line needs no `end if` at all. A newline right after `then` is what selects the block form instead; that's the only difference. The transpiler transpiles either form to numeric GOTO targets in the generated BASIC — you never write a line number yourself.

</div>

<div class="snippet" markdown="1">

### if / elseif / elseif / else

```bascal
if points% >= 90 then
    grade$ = "A"
elseif points% >= 80 then
    grade$ = "B"        // points% = 85 lands here
elseif points% >= 70 then
    grade$ = "C"
elseif points% >= 60 then
    grade$ = "D"
else
    grade$ = "F"
end if
```

</div>

<div class="snippet" markdown="1">

### Single-line if — no end if needed

`elseif` isn't available single-line, same as classic BASIC — it needs the block form above.

```bascal
if temperature% > 30 then print "Hot day (single-line)"
if temperature% > 100 then print "Scorching" else print "Not scorching"
```

</div>



[← Operators and Expressions](arithmetic.md)  ·  [Loops →](loops.md)


<!-- BEGIN generated tutorial source -->

<details class="source-embed" markdown="1">

<summary><code>tutorial/conditions.bcl</code></summary>



```bascal

// Tutorial — Conditions: IF / ELSEIF / ELSE / END IF
//
// BASCAL supports multi-line block IF statements.  The compiler transpiles
// them to numeric goto targets so the generated BASIC is compatible with
// 1980s BASCOM.  You never write line numbers yourself.
//
// Forms:
//   if cond then ... end if
//   if cond then ... else ... end if
//   if cond then ... elseif cond then ... else ... end if
//   if cond then statement                   (single-line, no end if)
//   if cond then statement else statement     (single-line, no end if)
//
// A newline right after `then` selects the block form; a statement
// directly after `then` on the same line selects the single-line form
// instead -- that's the only difference. elseif isn't available
// single-line, same as classic BASIC.
program conditions

/* Simple IF */
temperature% = 23
if temperature% > 30 then
    print "Hot day"
end if

/* IF / ELSE */
score% = 72
if score% >= 60 then
    print "Pass ("; score%; ")"
else
    print "Fail ("; score%; ")"
end if

/* IF / ELSEIF / ELSE — grade classification */
points% = 85

if points% >= 90 then
    grade$ = "A"
elseif points% >= 80 then
    grade$ = "B" // points% = 85 lands here
elseif points% >= 70 then
    grade$ = "C"
elseif points% >= 60 then
    grade$ = "D"
else
    grade$ = "F"
end if

print "Grade: " + grade$

/* Nested IF */
x% = 15
if x% > 0 then
    if x% > 10 then
        print x%; "is large and positive"
    else
        print x%; "is small and positive"
    end if
else
    print x%; "is not positive"
end if

/* Single-line IF -- no end if needed */
temperature% = 23
if temperature% > 30 then print "Hot day (single-line)"
if temperature% > 100 then print "Scorching" else print "Not scorching"

/* Compound conditions */
age% = 25
income% = 45000
if age% >= 18 and income% >= 30000 then
    print "Eligible"
else
    print "Not eligible"
end if

end

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/conditions.bas</code></summary>



```basic

10 ' BASCAL generated BASIC -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Tutorial — Conditions: IF / ELSEIF / ELSE / END IF
40 '
50 ' BASCAL supports multi-line block IF statements.  The compiler transpiles
60 ' them to numeric goto targets so the generated BASIC is compatible with
70 ' 1980s BASCOM.  You never write line numbers yourself.
80 '
90 ' Forms:
100 ' if cond then ... end if
110 ' if cond then ... else ... end if
120 ' if cond then ... elseif cond then ... else ... end if
130 ' if cond then statement                   (single-line, no end if)
140 ' if cond then statement else statement     (single-line, no end if)
150 '
160 ' A newline right after `then` selects the block form; a statement
170 ' directly after `then` on the same line selects the single-line form
180 ' instead -- that's the only difference. elseif isn't available
190 ' single-line, same as classic BASIC.
200 ' Simple IF
210 temperature% = 23
220 IF (temperature% > 30) = 0 THEN GOTO 240
230     PRINT "Hot day"
240 REM END IF
250 ' IF / ELSE
260 score% = 72
270 IF (score% >= 60) = 0 THEN GOTO 300
280     PRINT "Pass ("; score%; ")"
290 GOTO 310
300     PRINT "Fail ("; score%; ")"
310 REM END IF
320 ' IF / ELSEIF / ELSE — grade classification
330 points% = 85
340 IF (points% >= 90) = 0 THEN GOTO 370
350     grade$ = "A"
360 GOTO 510
370     IF (points% >= 80) = 0 THEN GOTO 410
380         grade$ = "B"
390         ' points% = 85 lands here
400     GOTO 500
410         IF (points% >= 70) = 0 THEN GOTO 440
420             grade$ = "C"
430         GOTO 490
440             IF (points% >= 60) = 0 THEN GOTO 470
450                 grade$ = "D"
460             GOTO 480
470                 grade$ = "F"
480             REM END IF
490         REM END IF
500     REM END IF
510 REM END IF
520 PRINT "Grade: " + grade$
530 ' Nested IF
540 x% = 15
550 IF (x% > 0) = 0 THEN GOTO 620
560     IF (x% > 10) = 0 THEN GOTO 590
570         PRINT x%; "is large and positive"
580     GOTO 600
590         PRINT x%; "is small and positive"
600     REM END IF
610 GOTO 630
620     PRINT x%; "is not positive"
630 REM END IF
640 ' Single-line IF -- no end if needed
650 temperature% = 23
660 IF (temperature% > 30) = 0 THEN GOTO 680
670     PRINT "Hot day (single-line)"
680 REM END IF
690 IF (temperature% > 100) = 0 THEN GOTO 720
700     PRINT "Scorching"
710 GOTO 730
720     PRINT "Not scorching"
730 REM END IF
740 ' Compound conditions
750 age% = 25
760 income% = 45000
770 IF ((age% >= 18) AND (income% >= 30000)) = 0 THEN GOTO 800
780     PRINT "Eligible"
790 GOTO 810
800     PRINT "Not eligible"
810 REM END IF
820 END

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/conditions.c</code></summary>



```c

// BASCAL generated C -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
#include <stdio.h>
#include <math.h>

static int bv_i_age = 0;
static int bv_i_income = 0;
static int bv_i_points = 0;
static int bv_i_score = 0;
static int bv_i_temperature = 0;
static int bv_i_x = 0;
static char bv_s_grade[256] = {0};

int main(void) {
    // Tutorial — Conditions: IF / ELSEIF / ELSE / END IF
    //
    // BASCAL supports multi-line block IF statements.  The compiler transpiles
    // them to numeric goto targets so the generated BASIC is compatible with
    // 1980s BASCOM.  You never write line numbers yourself.
    //
    // Forms:
    // if cond then ... end if
    // if cond then ... else ... end if
    // if cond then ... elseif cond then ... else ... end if
    // if cond then statement                   (single-line, no end if)
    // if cond then statement else statement     (single-line, no end if)
    //
    // A newline right after `then` selects the block form; a statement
    // directly after `then` on the same line selects the single-line form
    // instead -- that's the only difference. elseif isn't available
    // single-line, same as classic BASIC.

    // Simple IF
    bv_i_temperature = 23;
    if ((-(bv_i_temperature > 30))) {
        printf("Hot day\n");
    }

    // IF / ELSE
    bv_i_score = 72;
    if ((-(bv_i_score >= 60))) {
        printf("Pass (%d)\n", bv_i_score);
    } else {
        printf("Fail (%d)\n", bv_i_score);
    }

    // IF / ELSEIF / ELSE — grade classification
    bv_i_points = 85;

    if ((-(bv_i_points >= 90))) {
        snprintf(bv_s_grade, sizeof(bv_s_grade), "%s", "A");
    } else {
        if ((-(bv_i_points >= 80))) {
            snprintf(bv_s_grade, sizeof(bv_s_grade), "%s", "B");
            // points% = 85 lands here
        } else {
            if ((-(bv_i_points >= 70))) {
                snprintf(bv_s_grade, sizeof(bv_s_grade), "%s", "C");
            } else {
                if ((-(bv_i_points >= 60))) {
                    snprintf(bv_s_grade, sizeof(bv_s_grade), "%s", "D");
                } else {
                    snprintf(bv_s_grade, sizeof(bv_s_grade), "%s", "F");
                }
            }
        }
    }

    char bt_s_0[256];
    snprintf(bt_s_0, sizeof(bt_s_0), "%s%s", "Grade: ", bv_s_grade);
    printf("%s\n", bt_s_0);

    // Nested IF
    bv_i_x = 15;
    if ((-(bv_i_x > 0))) {
        if ((-(bv_i_x > 10))) {
            printf("%dis large and positive\n", bv_i_x);
        } else {
            printf("%dis small and positive\n", bv_i_x);
        }
    } else {
        printf("%dis not positive\n", bv_i_x);
    }

    // Single-line IF -- no end if needed
    bv_i_temperature = 23;
    if ((-(bv_i_temperature > 30))) {
        printf("Hot day (single-line)\n");
    }
    if ((-(bv_i_temperature > 100))) {
        printf("Scorching\n");
    } else {
        printf("Not scorching\n");
    }

    // Compound conditions
    bv_i_age = 25;
    bv_i_income = 45000;
    if (((int)((long)round((double)(-(bv_i_age >= 18))) & (long)round((double)(-(bv_i_income >= 30000)))))) {
        printf("Eligible\n");
    } else {
        printf("Not eligible\n");
    }

    return 0;
}

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/conditions.j</code></summary>



```basic

.class public Conditions
.super java/lang/Object

.field public static g1 I
.field public static g2 Ljava/lang/String;
.field public static g3 I
.field public static g4 I
.field public static g5 I
.field public static g6 I
.field public static g7 I
.method public static main : ([Ljava/lang/String;)V

    iconst_0
    putstatic Conditions/g1 I
    ldc ""
    putstatic Conditions/g2 Ljava/lang/String;
    iconst_0
    putstatic Conditions/g3 I
    iconst_0
    putstatic Conditions/g4 I
    iconst_0
    putstatic Conditions/g5 I
    iconst_0
    putstatic Conditions/g6 I
    iconst_0
    putstatic Conditions/g7 I
    ; Tutorial — Conditions: IF / ELSEIF / ELSE / END IF
    ;
    ; BASCAL supports multi-line block IF statements.  The compiler transpiles
    ; them to numeric goto targets so the generated BASIC is compatible with
    ; 1980s BASCOM.  You never write line numbers yourself.
    ;
    ; Forms:
    ; if cond then ... end if
    ; if cond then ... else ... end if
    ; if cond then ... elseif cond then ... else ... end if
    ; if cond then statement                   (single-line, no end if)
    ; if cond then statement else statement     (single-line, no end if)
    ;
    ; A newline right after `then` selects the block form; a statement
    ; directly after `then` on the same line selects the single-line form
    ; instead -- that's the only difference. elseif isn't available
    ; single-line, same as classic BASIC.
    ; Simple IF
    ldc 23
    putstatic Conditions/g6 I
    getstatic Conditions/g6 I
    ldc 30
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_condition_0
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Hot day"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
L_condition_0:
    ; IF / ELSE
    ldc 72
    putstatic Conditions/g5 I
    getstatic Conditions/g5 I
    ldc 60
    invokestatic java/lang/Integer/compare (II)I
    ineg
    iconst_1
    isub
    bipush 31
    ishr
    ifeq L_condition_2
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Pass ("
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Conditions/g5 I
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc ")"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_condition_3
L_condition_2:
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Fail ("
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Conditions/g5 I
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc ")"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
L_condition_3:
    ; IF / ELSEIF / ELSE — grade classification
    ldc 85
    putstatic Conditions/g4 I
    getstatic Conditions/g4 I
    ldc 90
    invokestatic java/lang/Integer/compare (II)I
    ineg
    iconst_1
    isub
    bipush 31
    ishr
    ifeq L_condition_4
    ldc "A"
    putstatic Conditions/g2 Ljava/lang/String;
    goto L_condition_5
L_condition_4:
    getstatic Conditions/g4 I
    ldc 80
    invokestatic java/lang/Integer/compare (II)I
    ineg
    iconst_1
    isub
    bipush 31
    ishr
    ifeq L_condition_6
    ldc "B"
    putstatic Conditions/g2 Ljava/lang/String;
    ; points% = 85 lands here
    goto L_condition_7
L_condition_6:
    getstatic Conditions/g4 I
    ldc 70
    invokestatic java/lang/Integer/compare (II)I
    ineg
    iconst_1
    isub
    bipush 31
    ishr
    ifeq L_condition_8
    ldc "C"
    putstatic Conditions/g2 Ljava/lang/String;
    goto L_condition_9
L_condition_8:
    getstatic Conditions/g4 I
    ldc 60
    invokestatic java/lang/Integer/compare (II)I
    ineg
    iconst_1
    isub
    bipush 31
    ishr
    ifeq L_condition_10
    ldc "D"
    putstatic Conditions/g2 Ljava/lang/String;
    goto L_condition_11
L_condition_10:
    ldc "F"
    putstatic Conditions/g2 Ljava/lang/String;
L_condition_11:
L_condition_9:
L_condition_7:
L_condition_5:
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "Grade: "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    getstatic Conditions/g2 Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ; Nested IF
    ldc 15
    putstatic Conditions/g7 I
    getstatic Conditions/g7 I
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_condition_12
    getstatic Conditions/g7 I
    ldc 10
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_condition_14
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Conditions/g7 I
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "is large and positive"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_condition_15
L_condition_14:
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Conditions/g7 I
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "is small and positive"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
L_condition_15:
    goto L_condition_13
L_condition_12:
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Conditions/g7 I
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "is not positive"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
L_condition_13:
    ; Single-line IF -- no end if needed
    ldc 23
    putstatic Conditions/g6 I
    getstatic Conditions/g6 I
    ldc 30
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_condition_16
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Hot day (single-line)"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
L_condition_16:
    getstatic Conditions/g6 I
    ldc 100
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_condition_18
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Scorching"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_condition_19
L_condition_18:
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Not scorching"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
L_condition_19:
    ; Compound conditions
    ldc 25
    putstatic Conditions/g1 I
    ldc 45000
    putstatic Conditions/g3 I
    getstatic Conditions/g1 I
    ldc 18
    invokestatic java/lang/Integer/compare (II)I
    ineg
    iconst_1
    isub
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
    getstatic Conditions/g3 I
    ldc 30000
    invokestatic java/lang/Integer/compare (II)I
    ineg
    iconst_1
    isub
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
    ifeq L_condition_20
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Eligible"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_condition_21
L_condition_20:
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Not eligible"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
L_condition_21:
    return
.end method

```



</details>

<!-- END generated tutorial source -->
