.version 50 0
.class public Methods
.super java/lang/Object

.field public static g1 Ljava/lang/String;
.field public static g2 I
.field public static g3 Ljava/lang/String;
.field public static g4 D
.field public static g6 Ljava/lang/String;
.field public static g7 I
.field public static g8 Ljava/lang/String;
.method public static ucase : (Ljava/lang/String;)Ljava/lang/String;
    .limit stack 16
    .limit locals 4

    iconst_0
    istore 1
    iconst_0
    istore 2
    ldc ""
    astore 3
    ldc ""
    astore 3
    ldc 1
    istore 2
L_for_0_top:
    iload 2
    aload 0
    invokevirtual java/lang/String/length ()I
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
    ifeq L_if_1_else
    iload 1
    ldc 122
    invokestatic java/lang/Integer/compare (II)I
    iconst_1
    isub
    bipush 31
    ishr
    ifeq L_if_1_else
    iload 1
    ldc 32
    isub
    istore 1
L_if_1_else:
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

.method public static shout : (Ljava/lang/String;)Ljava/lang/String;
    .limit stack 16
    .limit locals 1

    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    aload 0
    invokestatic Methods/ucase (Ljava/lang/String;)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc "!"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    areturn
    ldc ""
    areturn
.end method

.method public static surround : (Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;
    .limit stack 16
    .limit locals 3

    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    aload 1
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    aload 0
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    aload 2
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    areturn
    ldc ""
    areturn
.end method

.method public static clamp : (III)I
    .limit stack 16
    .limit locals 3

    iload 0
    iload 1
    invokestatic java/lang/Integer/compare (II)I
    bipush 31
    ishr
    ifeq L_if_0_else
    iload 1
    ireturn
    goto L_if_0_end
L_if_0_else:
    iload 0
    iload 2
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_if_1_else
    iload 2
    ireturn
L_if_1_else:
L_if_0_end:
    iload 0
    ireturn
    iconst_0
    ireturn
.end method

.method public static percent : (DD)D
    .limit stack 16
    .limit locals 4

    dload 0
    dload 2
    dmul
    ldc 100
    i2d
    ddiv
    dreturn
    dconst_0
    dreturn
.end method

.method public static main : ([Ljava/lang/String;)V
    .limit stack 16
    .limit locals 9

    ldc ""
    putstatic Methods/g1 Ljava/lang/String;
    iconst_0
    putstatic Methods/g2 I
    ldc ""
    putstatic Methods/g3 Ljava/lang/String;
    dconst_0
    putstatic Methods/g4 D
    ldc ""
    putstatic Methods/g6 Ljava/lang/String;
    iconst_0
    putstatic Methods/g7 I
    ldc ""
    putstatic Methods/g8 Ljava/lang/String;
    ; Upper-cases self$. Not a real MBASIC/BASCOM 2.00 builtin -- verified
    ; against a real IBM BASIC Compiler 2.00 under dosbox-x -- so BASCAL ships
    ; its own. Declared as a scalar method (see GitHub issue #41 and
    ; ltrim.bcl's own doc comment for the reasoning) -- ucase$(s$) still works
    ; via ordinary-call syntax resolving to this same declaration.

    ; Tutorial — Scalar methods
    ;
    ; A method has a typed scalar receiver, written after `method`, and a typed
    ; result suffix on its name. The receiver is available as self%/self!/self$
    ; in the body. Dot calls can chain when each result has the next receiver's
    ; type. Methods transpile to ordinary typed calls for both backends.






    ldc "bascal"
    putstatic Methods/g3 Ljava/lang/String;
    getstatic Methods/g3 Ljava/lang/String;
    ldc "["
    ldc "]"
    invokestatic Methods/surround (Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;
    putstatic Methods/g6 Ljava/lang/String;
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Methods/g6 Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic Methods/g3 Ljava/lang/String;
    invokestatic Methods/shout (Ljava/lang/String;)Ljava/lang/String;
    putstatic Methods/g8 Ljava/lang/String;
    getstatic Methods/g3 Ljava/lang/String;
    iconst_0
    ldc 5
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    invokevirtual java/lang/String/length ()I
    putstatic Methods/g2 I
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "length = "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Methods/g2 I
    invokevirtual java/io/PrintStream/println (I)V

    ldc 125
    putstatic Methods/g7 I
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "clamped score = "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Methods/g7 I
    ldc 0
    ldc 100
    invokestatic Methods/clamp (III)I
    invokevirtual java/io/PrintStream/println (I)V

    ldc 80
    i2d
    putstatic Methods/g4 D
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "discount amount = "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Methods/g4 D
    ldc 15
    i2d
    invokestatic Methods/percent (DD)D
    invokevirtual java/io/PrintStream/println (D)V

    getstatic Methods/g3 Ljava/lang/String;
    iconst_0
    ldc 3
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    invokestatic Methods/ucase (Ljava/lang/String;)Ljava/lang/String;
    putstatic Methods/g1 Ljava/lang/String;
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "first three = "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Methods/g1 Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V

    return
.end method
