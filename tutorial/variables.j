.version 50 0
.class public Variables
.super java/lang/Object

.field public static g1 Ljava/lang/String;
.field public static g2 Ljava/lang/String;
.field public static g3 I
.field public static g4 D
.method public static bccStr : (D)Ljava/lang/String;
    .limit stack 6
    .limit locals 2

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
    .limit stack 16
    .limit locals 6

    ldc ""
    putstatic Variables/g1 Ljava/lang/String;
    ldc ""
    putstatic Variables/g2 Ljava/lang/String;
    iconst_0
    putstatic Variables/g3 I
    dconst_0
    putstatic Variables/g4 D
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


    ; Variable assignment uses =
    ldc "Alice"
    putstatic Variables/g2 Ljava/lang/String;
    ldc 87
    putstatic Variables/g3 I
    ldc2_w 36.6
    putstatic Variables/g4 D

    ; print mixes strings and numbers directly with ; (no str$() needed)
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Grade Checker"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Player:      "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Variables/g2 Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Score:       "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Variables/g3 I
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "/ "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc 100
    invokevirtual java/io/PrintStream/println (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Pass mark:   "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc 60
    invokevirtual java/io/PrintStream/println (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Temperature: "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Variables/g4 D
    invokestatic Variables/bccStr (D)Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Tax rate:    "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc2_w 0.2
    invokestatic Variables/bccStr (D)Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V

    ; str$() is still available when you need to build a string value
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "Score is "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    getstatic Variables/g3 I
    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    putstatic Variables/g1 Ljava/lang/String;
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Variables/g1 Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V

    return
.end method
