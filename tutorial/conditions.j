.version 50 0
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
    .limit stack 16
    .limit locals 8

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
