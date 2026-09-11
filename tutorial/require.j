.version 50 0
.class public RequireDemo
.super java/lang/Object

.field public static a0 [I
.method public static mean : ([I)D
    .limit stack 16
    .limit locals 4

    iconst_0
    istore 1
    iconst_0
    istore 2
    iconst_0
    istore 3
    aload 0
    bipush 1
    bipush 0
    invokestatic RequireDemo/bccCopyArray (Ljava/lang/Object;II)Ljava/lang/Object;
    checkcast [I
    astore 0
    ; Arithmetic mean of data%(0..sizeof(data%)-1).
    ldc 0
    istore 3
    aload 0
    arraylength
    istore 1
    ldc 0
    istore 2
L_for_0_top:
    iload 2
    iload 1
    ldc 1
    isub
    if_icmpgt L_for_0_end
    iload 3
    aload 0
    iload 2
    iaload
    iadd
    istore 3
    iload 2
    ldc 1
    iadd
    istore 2
    goto L_for_0_top
L_for_0_end:
    iload 3
    i2d
    iload 1
    i2d
    ddiv
    dreturn
    dconst_0
    dreturn
.end method

.method public static maximum : ([I)I
    .limit stack 16
    .limit locals 3

    iconst_0
    istore 1
    iconst_0
    istore 2
    aload 0
    bipush 1
    bipush 0
    invokestatic RequireDemo/bccCopyArray (Ljava/lang/Object;II)Ljava/lang/Object;
    checkcast [I
    astore 0
    ; Largest element in data%(0..sizeof(data%)-1).
    aload 0
    ldc 0
    iaload
    istore 1
    ldc 1
    istore 2
L_for_0_top:
    iload 2
    aload 0
    arraylength
    ldc 1
    isub
    if_icmpgt L_for_0_end
    aload 0
    iload 2
    iaload
    iload 1
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_if_1_else
    aload 0
    iload 2
    iaload
    istore 1
L_if_1_else:
    iload 2
    ldc 1
    iadd
    istore 2
    goto L_for_0_top
L_for_0_end:
    iload 1
    ireturn
    iconst_0
    ireturn
.end method

.method public static minimum : ([I)I
    .limit stack 16
    .limit locals 3

    iconst_0
    istore 1
    iconst_0
    istore 2
    aload 0
    bipush 1
    bipush 0
    invokestatic RequireDemo/bccCopyArray (Ljava/lang/Object;II)Ljava/lang/Object;
    checkcast [I
    astore 0
    ; Smallest element in data%(0..sizeof(data%)-1).
    aload 0
    ldc 0
    iaload
    istore 1
    ldc 1
    istore 2
L_for_0_top:
    iload 2
    aload 0
    arraylength
    ldc 1
    isub
    if_icmpgt L_for_0_end
    aload 0
    iload 2
    iaload
    iload 1
    invokestatic java/lang/Integer/compare (II)I
    bipush 31
    ishr
    ifeq L_if_1_else
    aload 0
    iload 2
    iaload
    istore 1
L_if_1_else:
    iload 2
    ldc 1
    iadd
    istore 2
    goto L_for_0_top
L_for_0_end:
    iload 1
    ireturn
    iconst_0
    ireturn
.end method

.method public static rangeOf : ([I)I
    .limit stack 16
    .limit locals 1

    aload 0
    bipush 1
    bipush 0
    invokestatic RequireDemo/bccCopyArray (Ljava/lang/Object;II)Ljava/lang/Object;
    checkcast [I
    astore 0
    ; Difference between maximum and minimum.
    aload 0
    invokestatic RequireDemo/maximum ([I)I
    aload 0
    invokestatic RequireDemo/minimum ([I)I
    isub
    ireturn
    iconst_0
    ireturn
.end method

.method private static bccCopyArray : (Ljava/lang/Object;II)Ljava/lang/Object;
    .limit stack 5
    .limit locals 5

    iload 1
    iconst_1
    if_icmpne L_copy_nested
    iload 2
    tableswitch 0
        L_copy_int
        L_copy_long
        L_copy_double
        L_copy_object
L_copy_int:
    aload 0
    checkcast [I
    invokevirtual [I/clone ()Ljava/lang/Object;
    areturn
L_copy_long:
    aload 0
    checkcast [J
    invokevirtual [J/clone ()Ljava/lang/Object;
    areturn
L_copy_double:
    aload 0
    checkcast [D
    invokevirtual [D/clone ()Ljava/lang/Object;
    areturn
L_copy_object:
    aload 0
    checkcast [Ljava/lang/Object;
    invokevirtual [Ljava/lang/Object;/clone ()Ljava/lang/Object;
    areturn
L_copy_nested:
    aload 0
    checkcast [Ljava/lang/Object;
    invokevirtual [Ljava/lang/Object;/clone ()Ljava/lang/Object;
    checkcast [Ljava/lang/Object;
    astore 3
    iconst_0
    istore 4
L_copy_loop:
    iload 4
    aload 3
    arraylength
    if_icmpge L_copy_done
    aload 3
    iload 4
    aload 3
    iload 4
    aaload
    iload 1
    iconst_1
    isub
    iload 2
    invokestatic RequireDemo/bccCopyArray (Ljava/lang/Object;II)Ljava/lang/Object;
    aastore
    iinc 4 1
    goto L_copy_loop
L_copy_done:
    aload 3
    areturn
.end method

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
    .limit locals 1

    ldc 8
    ldc 1
    isub
    iconst_1
    iadd
    multianewarray [I 1
    putstatic RequireDemo/a0 [I
    ; stats.bcl — basic statistics library for the BASCAL tutorial.
    ; Loaded by tutorial/require.bcl via:
    ; require stats
    ; 
    ; Provides: mean!, maximum%, minimum%, rangeOf%

    ; data% -- array to average; byval, since mean! only reads it

    ; data% -- array to search; byval, since maximum% only reads it

    ; data% -- array to search; byval, since minimum% only reads it

    ; data% -- array to measure; byval, since rangeOf% only reads it
    ; Tutorial — REQUIRE and multi-file projects
    ; 
    ; REQUIRE loads another .bcl file and merges its functions into the
    ; generated output.  The path is dot-separated and maps to a file:
    ; 
    ; require stats   →  stats.bcl  (in the same directory or a -L path)
    ; require com.bascal.sort.bubbleSort
    ; →  com/bascal/sort/bubbleSort.bcl
    ; 
    ; All required functions become part of the single generated .bas file.
    ; The original require line is preserved as a comment in the output.
    ; 
    ; Run with:
    ; bcc tutorial/require.bcl -L tutorial/lib
    ; 
    ; The -L flag adds tutorial/lib/ to the search path so that
    ; require stats   resolves to  tutorial/lib/stats.bcl



    getstatic RequireDemo/a0 [I
    ldc 0
    ldc 74
    iastore
    getstatic RequireDemo/a0 [I
    ldc 1
    ldc 91
    iastore
    getstatic RequireDemo/a0 [I
    ldc 2
    ldc 63
    iastore
    getstatic RequireDemo/a0 [I
    ldc 3
    ldc 88
    iastore
    getstatic RequireDemo/a0 [I
    ldc 4
    ldc 55
    iastore
    getstatic RequireDemo/a0 [I
    ldc 5
    ldc 97
    iastore
    getstatic RequireDemo/a0 [I
    ldc 6
    ldc 72
    iastore
    getstatic RequireDemo/a0 [I
    ldc 7
    ldc 84
    iastore

    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Scores: 74 91 63 88 55 97 72 84"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "Mean:   "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    getstatic RequireDemo/a0 [I
    invokestatic RequireDemo/mean ([I)D
    invokestatic RequireDemo/bccStr (D)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "Max:    "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    getstatic RequireDemo/a0 [I
    invokestatic RequireDemo/maximum ([I)I
    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "Min:    "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    getstatic RequireDemo/a0 [I
    invokestatic RequireDemo/minimum ([I)I
    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "Range:  "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    getstatic RequireDemo/a0 [I
    invokestatic RequireDemo/rangeOf ([I)I
    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V

    return
.end method
