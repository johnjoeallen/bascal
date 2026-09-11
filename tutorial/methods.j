.version 50 0
.class public Methods
.super java/lang/Object

.field public static g1 Ljava/lang/String;
.field public static g2 J
.field public static g4 Ljava/lang/String;
.field public static g5 Ljava/lang/String;
.field public static g6 I
.field public static g7 Ljava/lang/String;
.field public static g8 D
.field public static g10 Ljava/lang/String;
.field public static g11 I
.field public static g12 Ljava/lang/String;
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
    aload 0
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
    aload 0
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
    aload 0
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
    dload 0
    dreturn
    dconst_0
    dreturn
.end method

.method public static cardRestock : (Ljava/lang/String;Ljava/lang/String;JI)V
    .limit stack 16
    .limit locals 5

    lload 2
    iload 4
    i2l
    ladd
    lstore 2
    return
.end method

.method public static cardDisplay : (Ljava/lang/String;Ljava/lang/String;J)Ljava/lang/String;
    .limit stack 16
    .limit locals 4

    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    aload 0
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc " by "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    aload 1
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    areturn
    ldc ""
    areturn
.end method

.method public static main : ([Ljava/lang/String;)V
    .limit stack 16
    .limit locals 13

    ldc ""
    putstatic Methods/g1 Ljava/lang/String;
    lconst_0
    putstatic Methods/g2 J
    ldc ""
    putstatic Methods/g4 Ljava/lang/String;
    ldc ""
    putstatic Methods/g5 Ljava/lang/String;
    iconst_0
    putstatic Methods/g6 I
    ldc ""
    putstatic Methods/g7 Ljava/lang/String;
    dconst_0
    putstatic Methods/g8 D
    ldc ""
    putstatic Methods/g10 Ljava/lang/String;
    iconst_0
    putstatic Methods/g11 I
    ldc ""
    putstatic Methods/g12 Ljava/lang/String;
    ; Upper-cases self$. Not a real MBASIC/BASCOM 2.00 builtin -- verified
    ; against a real IBM BASIC Compiler 2.00 under dosbox-x -- so BASCAL ships
    ; its own. Declared as a scalar method (see GitHub issue #41 and
    ; ltrim.bcl's own doc comment for the reasoning) -- ucase$(s$) still works
    ; via ordinary-call syntax resolving to this same declaration.

    ; Tutorial — Methods
    ; 
    ; A method is a statically resolved callable with an implicit receiver,
    ; written in brackets after the method name: `method shout[string]()`
    ; receives a string. The return type follows the parameter list after
    ; `:` (or its suffix shorthand, `: $`); omitting it for a scalar receiver
    ; makes the method return the receiver's own type, falling through to an
    ; implicit `return self`. Dot calls chain when each result has the next
    ; receiver's type. Methods transpile to ordinary typed calls for every
    ; backend -- there is no runtime method object, virtual dispatch, or
    ; vtable of any kind.
    ; 
    ; A record type is a valid receiver too, declared either externally (in
    ; brackets, same as a scalar receiver) or inline, directly inside the
    ; record itself. `self.field` is then ordinary field access against the
    ; receiver, and mutating it is visible to the caller once the call
    ; returns -- the receiver is passed by reference, not by copy.






    ldc "bascal"
    putstatic Methods/g7 Ljava/lang/String;
    getstatic Methods/g7 Ljava/lang/String;
    ldc "["
    ldc "]"
    invokestatic Methods/surround (Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;
    putstatic Methods/g10 Ljava/lang/String;
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Methods/g10 Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic Methods/g7 Ljava/lang/String;
    invokestatic Methods/shout (Ljava/lang/String;)Ljava/lang/String;
    putstatic Methods/g12 Ljava/lang/String;
    getstatic Methods/g7 Ljava/lang/String;
    iconst_0
    ldc 5
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    invokevirtual java/lang/String/length ()I
    putstatic Methods/g6 I
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "length = "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Methods/g6 I
    invokevirtual java/io/PrintStream/println (I)V

    ldc 125
    putstatic Methods/g11 I
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "clamped score = "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Methods/g11 I
    ldc 0
    ldc 100
    invokestatic Methods/clamp (III)I
    invokevirtual java/io/PrintStream/println (I)V

    ldc 80
    i2d
    putstatic Methods/g8 D
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "discount amount = "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Methods/g8 D
    ldc 15
    i2d
    invokestatic Methods/percent (DD)D
    invokevirtual java/io/PrintStream/println (D)V

    getstatic Methods/g7 Ljava/lang/String;
    iconst_0
    ldc 3
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    invokestatic Methods/ucase (Ljava/lang/String;)Ljava/lang/String;
    putstatic Methods/g5 Ljava/lang/String;
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "first three = "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Methods/g5 Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V

    ; -------------------- Record methods --------------------

    ; Declared inline: the enclosing record supplies the receiver type, so
    ; there is no `[Card]` bracket here at all.

    ; Declared externally: same receiver, same callable identity as an
    ; inline method -- an external method just lets behavior be attached to
    ; a record without editing its own declaration.

    ldc "Dune"
    putstatic Methods/g4 Ljava/lang/String;
    ldc "Frank Herbert"
    putstatic Methods/g1 Ljava/lang/String;
    ldc 2
    i2l
    putstatic Methods/g2 J
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Methods/g4 Ljava/lang/String;
    getstatic Methods/g1 Ljava/lang/String;
    getstatic Methods/g2 J
    invokestatic Methods/cardDisplay (Ljava/lang/String;Ljava/lang/String;J)Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "copies on hand = "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Methods/g2 J
    invokevirtual java/io/PrintStream/println (J)V

    getstatic Methods/g4 Ljava/lang/String;
    getstatic Methods/g1 Ljava/lang/String;
    getstatic Methods/g2 J
    ldc 3
    invokestatic Methods/cardRestock (Ljava/lang/String;Ljava/lang/String;JI)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "copies after restock = "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Methods/g2 J
    invokevirtual java/io/PrintStream/println (J)V

    return
.end method
