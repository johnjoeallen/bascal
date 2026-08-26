.version 50 0
.class public Variables
.super java/lang/Object

.method public static main : ([Ljava/lang/String;)V
    .limit stack 16
    .limit locals 6

    ldc ""
    astore 1
    ldc ""
    astore 2
    iconst_0
    istore 3
    dconst_0
    dstore 4
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
    astore 2
    ldc 87
    istore 3
    ldc2_w 36.6
    dstore 4

    ; print mixes strings and numbers directly with ; (no str$() needed)
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Grade Checker"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Player:      "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    aload 2
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Score:       "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 3
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
    dload 4
    invokevirtual java/io/PrintStream/println (D)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Tax rate:    "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc2_w 0.2
    invokevirtual java/io/PrintStream/println (D)V

    ; str$() is still available when you need to build a string value
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "Score is "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    iload 3
    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    astore 1
    getstatic java/lang/System/out Ljava/io/PrintStream;
    aload 1
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V

    return
.end method
