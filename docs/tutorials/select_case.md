[Home](../../) / [Tutorials](../) / Select Case

Generated sources: [BCL](https://github.com/johnjoeallen/bascal/blob/main/tutorial/select_case.bcl), [BASIC](https://github.com/johnjoeallen/bascal/blob/main/tutorial/select_case.bas), [C](https://github.com/johnjoeallen/bascal/blob/main/tutorial/select_case.c), and [JVM assembly](https://github.com/johnjoeallen/bascal/blob/main/tutorial/select_case.j).

<div class="prose" markdown="1">

`select case` evaluates its expression once, stores it in a transpiler-generated temporary, and dispatches against each `case` clause in order. Patterns can be an exact value, a comma-separated list, an inclusive `lo to hi` range, or an `is <op> value` comparison; `case else` is the default and must come last. See the [structured control-flow chapter](../home/control-flow.md) for the GOTO dispatch chain this transpiles to.

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



[← Loops](loops.md)  ·  [Functions →](functions.md)


<!-- BEGIN generated tutorial source -->

<details class="source-embed" markdown="1">

<summary><code>tutorial/select_case.bcl</code></summary>



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
        print "B  — Good" // score% = 85 matches here
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
        print day$ + " is a weekend" // matches here
    case else
        print "Unknown day: " + day$
end select

/* IS comparisons on temperature */
temp% = -3

select case temp%
    case is < 0
        print "Below freezing ("; temp%; "°)" // matches here
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
        print "Load game" // choice% = 2 matches
    case 4
        print "Options"
    case else
        print "Quit"
end select

end

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/select_case.bas</code></summary>



```basic

10 ' BASCAL generated BASIC -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
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
210 IF (BCCT2% >= 70 AND BCCT2% <= 79) <> 0 THEN GOTO 320
220 IF (BCCT2% >= 60 AND BCCT2% <= 69) <> 0 THEN GOTO 340
230 IF (BCCT2% >= 0) <> 0 THEN GOTO 360
240 GOTO 380
250     PRINT "Perfect!"
260     GOTO 390
270     PRINT "A  — Excellent"
280     GOTO 390
290     PRINT "B  — Good"
300     ' score% = 85 matches here
310     GOTO 390
320     PRINT "C  — Satisfactory"
330     GOTO 390
340     PRINT "D  — Passing"
350     GOTO 390
360     PRINT "F  — Fail"
370     GOTO 390
380     PRINT "Invalid score"
390 REM END SELECT
400 ' String select: day-of-week classification
410 day$ = "Saturday"
420 BCCT4$ = day$
430 IF (BCCT4$ = "Monday" OR BCCT4$ = "Tuesday" OR BCCT4$ = "Wednesday" OR BCCT4$ = "Thursday" OR BCCT4$ = "Friday") <> 0 THEN GOTO 460
440 IF (BCCT4$ = "Saturday" OR BCCT4$ = "Sunday") <> 0 THEN GOTO 480
450 GOTO 510
460     PRINT day$ + " is a weekday"
470     GOTO 520
480     PRINT day$ + " is a weekend"
490     ' matches here
500     GOTO 520
510     PRINT "Unknown day: " + day$
520 REM END SELECT
530 ' IS comparisons on temperature
540 temp% = -3
550 BCCT6% = temp%
560 IF (BCCT6% < 0) <> 0 THEN GOTO 610
570 IF (BCCT6% < 10) <> 0 THEN GOTO 640
580 IF (BCCT6% < 20) <> 0 THEN GOTO 660
590 IF (BCCT6% < 30) <> 0 THEN GOTO 680
600 GOTO 700
610     PRINT "Below freezing ("; temp%; "°)"
620     ' matches here
630     GOTO 710
640     PRINT "Cold ("; temp%; "°)"
650     GOTO 710
660     PRINT "Cool ("; temp%; "°)"
670     GOTO 710
680     PRINT "Warm ("; temp%; "°)"
690     GOTO 710
700     PRINT "Hot ("; temp%; "°)"
710 REM END SELECT
720 ' Multi-value list on a menu choice
730 choice% = 2
740 BCCT8% = choice%
750 IF (BCCT8% = 1) <> 0 THEN GOTO 790
760 IF (BCCT8% = 2 OR BCCT8% = 3) <> 0 THEN GOTO 810
770 IF (BCCT8% = 4) <> 0 THEN GOTO 840
780 GOTO 860
790     PRINT "New game"
800     GOTO 870
810     PRINT "Load game"
820     ' choice% = 2 matches
830     GOTO 870
840     PRINT "Options"
850     GOTO 870
860     PRINT "Quit"
870 REM END SELECT
880 END

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/select_case.c</code></summary>



```c

// BASCAL generated C -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
#include <stdio.h>
#include <string.h>

static int bv_i_choice = 0;
static int bv_i_score = 0;
static int bv_i_temp = 0;
static char bv_s_day[256] = {0};

int main(void) {
    // Tutorial — SELECT CASE
    //
    // SELECT CASE tests one expression against multiple patterns.  The
    // compiler evaluates the expression once, stores it in a temporary
    // variable, and emits an IF/goto dispatch chain.
    //
    // Pattern forms:
    // case value               — exact match
    // case v1, v2, v3          — any of the listed values
    // case low to high         — inclusive range
    // case is <op> value       — comparison (=  <>  <  <=  >  >=)
    // case else                — default; must be the last clause

    // Integer select: convert numeric score to letter grade
    bv_i_score = 85;

    {
        int bt_sel_0 = bv_i_score;
        int bt_sel_match_1 = 0;
        if (!bt_sel_match_1) {
            if ((bt_sel_0 == 100)) {
                bt_sel_match_1 = 1;
                printf("Perfect!\n");
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 >= 90 && bt_sel_0 <= 99)) {
                bt_sel_match_1 = 1;
                printf("A  — Excellent\n");
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 >= 80 && bt_sel_0 <= 89)) {
                bt_sel_match_1 = 1;
                printf("B  — Good\n");
                // score% = 85 matches here
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 >= 70 && bt_sel_0 <= 79)) {
                bt_sel_match_1 = 1;
                printf("C  — Satisfactory\n");
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 >= 60 && bt_sel_0 <= 69)) {
                bt_sel_match_1 = 1;
                printf("D  — Passing\n");
            }
        }
        if (!bt_sel_match_1) {
            if ((bt_sel_0 >= 0)) {
                bt_sel_match_1 = 1;
                printf("F  — Fail\n");
            }
        }
        if (!bt_sel_match_1) {
            printf("Invalid score\n");
        }
    }

    // String select: day-of-week classification
    snprintf(bv_s_day, sizeof(bv_s_day), "%s", "Saturday");

    {
        char bt_sel_2[256];
        snprintf(bt_sel_2, sizeof(bt_sel_2), "%s", bv_s_day);
        int bt_sel_match_3 = 0;
        if (!bt_sel_match_3) {
            if ((strcmp(bt_sel_2, "Monday") == 0) || (strcmp(bt_sel_2, "Tuesday") == 0) || (strcmp(bt_sel_2, "Wednesday") == 0) || (strcmp(bt_sel_2, "Thursday") == 0) || (strcmp(bt_sel_2, "Friday") == 0)) {
                bt_sel_match_3 = 1;
                char bt_s_4[256];
                snprintf(bt_s_4, sizeof(bt_s_4), "%s%s", bv_s_day, " is a weekday");
                printf("%s\n", bt_s_4);
            }
        }
        if (!bt_sel_match_3) {
            if ((strcmp(bt_sel_2, "Saturday") == 0) || (strcmp(bt_sel_2, "Sunday") == 0)) {
                bt_sel_match_3 = 1;
                char bt_s_5[256];
                snprintf(bt_s_5, sizeof(bt_s_5), "%s%s", bv_s_day, " is a weekend");
                printf("%s\n", bt_s_5);
                // matches here
            }
        }
        if (!bt_sel_match_3) {
            char bt_s_6[256];
            snprintf(bt_s_6, sizeof(bt_s_6), "%s%s", "Unknown day: ", bv_s_day);
            printf("%s\n", bt_s_6);
        }
    }

    // IS comparisons on temperature
    bv_i_temp = -(3);

    {
        int bt_sel_7 = bv_i_temp;
        int bt_sel_match_8 = 0;
        if (!bt_sel_match_8) {
            if ((bt_sel_7 < 0)) {
                bt_sel_match_8 = 1;
                printf("Below freezing (%d°)\n", bv_i_temp);
                // matches here
            }
        }
        if (!bt_sel_match_8) {
            if ((bt_sel_7 < 10)) {
                bt_sel_match_8 = 1;
                printf("Cold (%d°)\n", bv_i_temp);
            }
        }
        if (!bt_sel_match_8) {
            if ((bt_sel_7 < 20)) {
                bt_sel_match_8 = 1;
                printf("Cool (%d°)\n", bv_i_temp);
            }
        }
        if (!bt_sel_match_8) {
            if ((bt_sel_7 < 30)) {
                bt_sel_match_8 = 1;
                printf("Warm (%d°)\n", bv_i_temp);
            }
        }
        if (!bt_sel_match_8) {
            printf("Hot (%d°)\n", bv_i_temp);
        }
    }

    // Multi-value list on a menu choice
    bv_i_choice = 2;

    {
        int bt_sel_9 = bv_i_choice;
        int bt_sel_match_10 = 0;
        if (!bt_sel_match_10) {
            if ((bt_sel_9 == 1)) {
                bt_sel_match_10 = 1;
                printf("New game\n");
            }
        }
        if (!bt_sel_match_10) {
            if ((bt_sel_9 == 2) || (bt_sel_9 == 3)) {
                bt_sel_match_10 = 1;
                printf("Load game\n");
                // choice% = 2 matches
            }
        }
        if (!bt_sel_match_10) {
            if ((bt_sel_9 == 4)) {
                bt_sel_match_10 = 1;
                printf("Options\n");
            }
        }
        if (!bt_sel_match_10) {
            printf("Quit\n");
        }
    }

    return 0;
}

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/select_case.j</code></summary>



```basic

.class public SelectCase
.super java/lang/Object

.field public static g1 I
.field public static g2 Ljava/lang/String;
.field public static g3 I
.field public static g4 I
.method public static main : ([Ljava/lang/String;)V

    iconst_0
    putstatic SelectCase/g1 I
    ldc ""
    putstatic SelectCase/g2 Ljava/lang/String;
    iconst_0
    putstatic SelectCase/g3 I
    iconst_0
    putstatic SelectCase/g4 I
    ; Tutorial — SELECT CASE
    ;
    ; SELECT CASE tests one expression against multiple patterns.  The
    ; compiler evaluates the expression once, stores it in a temporary
    ; variable, and emits an IF/goto dispatch chain.
    ;
    ; Pattern forms:
    ; case value               — exact match
    ; case v1, v2, v3          — any of the listed values
    ; case low to high         — inclusive range
    ; case is <op> value       — comparison (=  <>  <  <=  >  >=)
    ; case else                — default; must be the last clause
    ; Integer select: convert numeric score to letter grade
    ldc 85
    putstatic SelectCase/g3 I
    getstatic SelectCase/g3 I
    dup
    ldc 100
    isub
    ifeq L_select_0_case_0
    goto L_select_0_next_0
L_select_0_next_0:
    dup
    ldc 90
    if_icmplt L_select_0_case_1_value_0
    dup
    ldc 99
    if_icmple L_select_0_case_1
L_select_0_case_1_value_0:
    goto L_select_0_next_1
L_select_0_next_1:
    dup
    ldc 80
    if_icmplt L_select_0_case_2_value_0
    dup
    ldc 89
    if_icmple L_select_0_case_2
L_select_0_case_2_value_0:
    goto L_select_0_next_2
L_select_0_next_2:
    dup
    ldc 70
    if_icmplt L_select_0_case_3_value_0
    dup
    ldc 79
    if_icmple L_select_0_case_3
L_select_0_case_3_value_0:
    goto L_select_0_next_3
L_select_0_next_3:
    dup
    ldc 60
    if_icmplt L_select_0_case_4_value_0
    dup
    ldc 69
    if_icmple L_select_0_case_4
L_select_0_case_4_value_0:
    goto L_select_0_next_4
L_select_0_next_4:
    dup
    ldc 0
    if_icmpge L_select_0_case_5
    goto L_select_0_next_5
L_select_0_next_5:
    pop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Invalid score"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_select_0_end
L_select_0_case_0:
    pop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Perfect!"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_select_0_end
L_select_0_case_1:
    pop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "A  — Excellent"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_select_0_end
L_select_0_case_2:
    pop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "B  — Good"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ; score% = 85 matches here
    goto L_select_0_end
L_select_0_case_3:
    pop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "C  — Satisfactory"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_select_0_end
L_select_0_case_4:
    pop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "D  — Passing"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_select_0_end
L_select_0_case_5:
    pop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "F  — Fail"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_select_0_end
L_select_0_end:
    ; String select: day-of-week classification
    ldc "Saturday"
    putstatic SelectCase/g2 Ljava/lang/String;
    getstatic SelectCase/g2 Ljava/lang/String;
    dup
    ldc "Monday"
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ifne L_select_1_case_0
    dup
    ldc "Tuesday"
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ifne L_select_1_case_0
    dup
    ldc "Wednesday"
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ifne L_select_1_case_0
    dup
    ldc "Thursday"
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ifne L_select_1_case_0
    dup
    ldc "Friday"
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ifne L_select_1_case_0
    goto L_select_1_next_0
L_select_1_next_0:
    dup
    ldc "Saturday"
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ifne L_select_1_case_1
    dup
    ldc "Sunday"
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ifne L_select_1_case_1
    goto L_select_1_next_1
L_select_1_next_1:
    pop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "Unknown day: "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    getstatic SelectCase/g2 Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_select_1_end
L_select_1_case_0:
    pop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    getstatic SelectCase/g2 Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc " is a weekday"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_select_1_end
L_select_1_case_1:
    pop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    getstatic SelectCase/g2 Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc " is a weekend"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ; matches here
    goto L_select_1_end
L_select_1_end:
    ; IS comparisons on temperature
    ldc 3
    ineg
    putstatic SelectCase/g4 I
    getstatic SelectCase/g4 I
    dup
    ldc 0
    if_icmplt L_select_2_case_0
    goto L_select_2_next_0
L_select_2_next_0:
    dup
    ldc 10
    if_icmplt L_select_2_case_1
    goto L_select_2_next_1
L_select_2_next_1:
    dup
    ldc 20
    if_icmplt L_select_2_case_2
    goto L_select_2_next_2
L_select_2_next_2:
    dup
    ldc 30
    if_icmplt L_select_2_case_3
    goto L_select_2_next_3
L_select_2_next_3:
    pop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Hot ("
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic SelectCase/g4 I
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "°)"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_select_2_end
L_select_2_case_0:
    pop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Below freezing ("
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic SelectCase/g4 I
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "°)"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ; matches here
    goto L_select_2_end
L_select_2_case_1:
    pop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Cold ("
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic SelectCase/g4 I
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "°)"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_select_2_end
L_select_2_case_2:
    pop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Cool ("
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic SelectCase/g4 I
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "°)"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_select_2_end
L_select_2_case_3:
    pop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Warm ("
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic SelectCase/g4 I
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "°)"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_select_2_end
L_select_2_end:
    ; Multi-value list on a menu choice
    ldc 2
    putstatic SelectCase/g1 I
    getstatic SelectCase/g1 I
    dup
    ldc 1
    isub
    ifeq L_select_3_case_0
    goto L_select_3_next_0
L_select_3_next_0:
    dup
    ldc 2
    isub
    ifeq L_select_3_case_1
    dup
    ldc 3
    isub
    ifeq L_select_3_case_1
    goto L_select_3_next_1
L_select_3_next_1:
    dup
    ldc 4
    isub
    ifeq L_select_3_case_2
    goto L_select_3_next_2
L_select_3_next_2:
    pop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Quit"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_select_3_end
L_select_3_case_0:
    pop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "New game"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_select_3_end
L_select_3_case_1:
    pop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Load game"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ; choice% = 2 matches
    goto L_select_3_end
L_select_3_case_2:
    pop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Options"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_select_3_end
L_select_3_end:
    return
.end method

```



</details>

<!-- END generated tutorial source -->
