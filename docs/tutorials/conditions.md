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
    grade$ = "B"        // points% = 85 lands here
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
age%    = 25
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
290     GOTO 310
300     PRINT "Fail ("; score%; ")"
310 REM END IF

320 ' IF / ELSEIF / ELSE — grade classification
330 points% = 85

340 IF (points% >= 90) = 0 THEN GOTO 370
350     grade$ = "A"
360     GOTO 500
370     IF (points% >= 80) = 0 THEN GOTO 400
380         grade$ = "B"
390         GOTO 490
400         IF (points% >= 70) = 0 THEN GOTO 430
410             grade$ = "C"
420             GOTO 480
430             IF (points% >= 60) = 0 THEN GOTO 460
440                 grade$ = "D"
450                 GOTO 470
460                 grade$ = "F"
470             REM END IF
480         REM END IF
490     REM END IF
500 REM END IF

510 PRINT "Grade: " + grade$

520 ' Nested IF
530 x% = 15
540 IF (x% > 0) = 0 THEN GOTO 610
550     IF (x% > 10) = 0 THEN GOTO 580
560         PRINT x%; "is large and positive"
570         GOTO 590
580         PRINT x%; "is small and positive"
590     REM END IF
600     GOTO 620
610     PRINT x%; "is not positive"
620 REM END IF

630 ' Single-line IF -- no end if needed
640 temperature% = 23
650 IF (temperature% > 30) = 0 THEN GOTO 670
660     PRINT "Hot day (single-line)"
670 REM END IF
680 IF (temperature% > 100) = 0 THEN GOTO 710
690     PRINT "Scorching"
700     GOTO 720
710     PRINT "Not scorching"
720 REM END IF

730 ' Compound conditions
740 age% = 25
750 income% = 45000
760 IF ((age% >= 18) AND (income% >= 30000)) = 0 THEN GOTO 790
770     PRINT "Eligible"
780     GOTO 800
790     PRINT "Not eligible"
800 REM END IF

810 END

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

.version 50 0
.class public Conditions
.super java/lang/Object

.method public static main : ([Ljava/lang/String;)V
    .limit stack 16
    .limit locals 8

    iconst_0
    istore 1
    ldc ""
    astore 2
    iconst_0
    istore 3
    iconst_0
    istore 4
    iconst_0
    istore 5
    iconst_0
    istore 6
    iconst_0
    istore 7
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
    istore 6
    iload 6
    ldc 30
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_if_0_else
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Hot day"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
L_if_0_else:

    ; IF / ELSE
    ldc 72
    istore 5
    iload 5
    ldc 60
    invokestatic java/lang/Integer/compare (II)I
    ineg
    iconst_1
    isub
    bipush 31
    ishr
    ifeq L_if_1_else
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Pass ("
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 5
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc ")"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_if_1_end
L_if_1_else:
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Fail ("
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 5
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc ")"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
L_if_1_end:

    ; IF / ELSEIF / ELSE — grade classification
    ldc 85
    istore 4

    iload 4
    ldc 90
    invokestatic java/lang/Integer/compare (II)I
    ineg
    iconst_1
    isub
    bipush 31
    ishr
    ifeq L_if_2_else
    ldc "A"
    astore 2
    goto L_if_2_end
L_if_2_else:
    iload 4
    ldc 80
    invokestatic java/lang/Integer/compare (II)I
    ineg
    iconst_1
    isub
    bipush 31
    ishr
    ifeq L_if_3_else
    ldc "B"
    astore 2
    goto L_if_3_end
L_if_3_else:
    iload 4
    ldc 70
    invokestatic java/lang/Integer/compare (II)I
    ineg
    iconst_1
    isub
    bipush 31
    ishr
    ifeq L_if_4_else
    ldc "C"
    astore 2
    goto L_if_4_end
L_if_4_else:
    iload 4
    ldc 60
    invokestatic java/lang/Integer/compare (II)I
    ineg
    iconst_1
    isub
    bipush 31
    ishr
    ifeq L_if_5_else
    ldc "D"
    astore 2
    goto L_if_5_end
L_if_5_else:
    ldc "F"
    astore 2
L_if_5_end:
L_if_4_end:
L_if_3_end:
L_if_2_end:

    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "Grade: "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    aload 2
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V

    ; Nested IF
    ldc 15
    istore 7
    iload 7
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_if_6_else
    iload 7
    ldc 10
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_if_7_else
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 7
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "is large and positive"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_if_7_end
L_if_7_else:
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 7
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "is small and positive"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
L_if_7_end:
    goto L_if_6_end
L_if_6_else:
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 7
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "is not positive"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
L_if_6_end:

    ; Single-line IF -- no end if needed
    ldc 23
    istore 6
    iload 6
    ldc 30
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_if_8_else
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Hot day (single-line)"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
L_if_8_else:
    iload 6
    ldc 100
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_if_9_else
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Scorching"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_if_9_end
L_if_9_else:
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Not scorching"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
L_if_9_end:

    ; Compound conditions
    ldc 25
    istore 1
    ldc 45000
    istore 3
    iload 1
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
    iload 3
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
    ifeq L_if_10_else
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Eligible"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_if_10_end
L_if_10_else:
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Not eligible"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
L_if_10_end:

    return
.end method

```



</details>

<!-- END generated tutorial source -->
