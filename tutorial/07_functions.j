.version 50 0
.class public Functions
.super java/lang/Object

.field public static g1 Ljava/lang/String;
.field public static g2 Ljava/lang/String;
.field public static g3 I
.field public static g4 I
.field public static g5 I
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

.method public static lcase : (Ljava/lang/String;)Ljava/lang/String;
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
    ldc 65
    invokestatic java/lang/Integer/compare (II)I
    ineg
    iconst_1
    isub
    bipush 31
    ishr
    ifeq L_if_1_else
    iload 1
    ldc 90
    invokestatic java/lang/Integer/compare (II)I
    iconst_1
    isub
    bipush 31
    ishr
    ifeq L_if_1_else
    iload 1
    ldc 32
    iadd
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

.method public static max : (II)I
    .limit stack 16
    .limit locals 2

    iload 0
    iload 1
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_if_0_else
    iload 0
    ireturn
    goto L_if_0_end
L_if_0_else:
    iload 1
    ireturn
L_if_0_end:
    iconst_0
    ireturn
.end method

.method public static min : (II)I
    .limit stack 16
    .limit locals 2

    iload 0
    iload 1
    invokestatic java/lang/Integer/compare (II)I
    bipush 31
    ishr
    ifeq L_if_0_else
    iload 0
    ireturn
    goto L_if_0_end
L_if_0_else:
    iload 1
    ireturn
L_if_0_end:
    iconst_0
    ireturn
.end method

.method public static clamp : (III)I
    .limit stack 16
    .limit locals 3

    ; Constrain value to [lo, hi].
    iload 1
    iload 0
    iload 2
    invokestatic Functions/min (II)I
    invokestatic Functions/max (II)I
    ireturn
    iconst_0
    ireturn
.end method

.method public static repeat : (Ljava/lang/String;I)Ljava/lang/String;
    .limit stack 16
    .limit locals 4

    ldc ""
    astore 2
    iconst_0
    istore 3
    ; Concatenate text$ with itself n times.
    ldc ""
    astore 2
    ldc 1
    istore 3
L_for_0_top:
    iload 3
    iload 1
    if_icmpgt L_for_0_end
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    aload 2
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    aload 0
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    astore 2
    iload 3
    ldc 1
    iadd
    istore 3
    goto L_for_0_top
L_for_0_end:
    aload 2
    areturn
    ldc ""
    areturn
.end method

.method public static titleCase : (Ljava/lang/String;)Ljava/lang/String;
    .limit stack 16
    .limit locals 1

    ; Capitalise first letter, lowercase remainder.
    ; UCASE$/LCASE$ aren't real MBASIC/BASCOM 2.00 builtins (verified
    ; against a real IBM BASIC Compiler 2.00 under dosbox-x), so this
    ; requires BASCAL's own com.bascal.stdlib implementations above.
    aload 0
    invokevirtual java/lang/String/length ()I
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    dup
    ineg
    ior
    bipush 31
    iushr
    iconst_1
    ixor
    ineg
    ifeq L_if_0_else
    ldc ""
    areturn
L_if_0_else:
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    aload 0
    iconst_0
    ldc 1
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    invokestatic Functions/ucase (Ljava/lang/String;)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    aload 0
    ldc 2
    iconst_1
    isub
    invokevirtual java/lang/String/substring (I)Ljava/lang/String;
    invokestatic Functions/lcase (Ljava/lang/String;)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    areturn
    ldc ""
    areturn
.end method

.method public static sumTo : (I)I
    .limit stack 16
    .limit locals 3

    iconst_0
    istore 1
    iconst_0
    istore 2
    ; i% and acc% are local to sumTo%.
    ldc 0
    istore 1
    ldc 1
    istore 2
L_for_0_top:
    iload 2
    iload 0
    if_icmpgt L_for_0_end
    iload 1
    iload 2
    iadd
    istore 1
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

.method public static productTo : (I)I
    .limit stack 16
    .limit locals 3

    iconst_0
    istore 1
    iconst_0
    istore 2
    ; i% and acc% here are independent of sumTo%'s i% and acc%.
    ldc 1
    istore 1
    ldc 1
    istore 2
L_for_0_top:
    iload 2
    iload 0
    if_icmpgt L_for_0_end
    iload 1
    iload 2
    imul
    istore 1
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

.method public static addToTotal : (I)I
    .limit stack 16
    .limit locals 1

    getstatic Functions/g5 I
    iload 0
    iadd
    putstatic Functions/g5 I
    getstatic Functions/g5 I
    ireturn
    iconst_0
    ireturn
.end method

.method public static main : ([Ljava/lang/String;)V
    .limit stack 16
    .limit locals 6

    ldc ""
    putstatic Functions/g1 Ljava/lang/String;
    ldc ""
    putstatic Functions/g2 Ljava/lang/String;
    iconst_0
    putstatic Functions/g3 I
    iconst_0
    putstatic Functions/g4 I
    iconst_0
    putstatic Functions/g5 I
    ; Upper-cases self$. Not a real MBASIC/BASCOM 2.00 builtin -- verified
    ; against a real IBM BASIC Compiler 2.00 under dosbox-x -- so BASCAL ships
    ; its own. Declared as a scalar method (see GitHub issue #41 and
    ; ltrim.bcl's own doc comment for the reasoning) -- ucase$(s$) still works
    ; via ordinary-call syntax resolving to this same declaration.

    ; Lower-cases self$. Not a real MBASIC/BASCOM 2.00 builtin -- verified
    ; against a real IBM BASIC Compiler 2.00 under dosbox-x -- so BASCAL ships
    ; its own. Declared as a scalar method (see GitHub issue #41 and
    ; ltrim.bcl's own doc comment for the reasoning) -- lcase$(s$) still works
    ; via ordinary-call syntax resolving to this same declaration.

    ; Tutorial — Functions
    ;
    ; A BASCAL function is declared with FUNCTION ... END FUNCTION.
    ; The function name carries the return type suffix.  Parameters
    ; also carry type suffixes.  Every function must reach a RETURN.
    ;
    ; Variables declared inside a function are local by default: the compiler
    ; prefixes them with the function name.  To access a global variable from
    ; inside a function, declare it with:  global varname
    ;
    ; Functions cannot recurse, directly or indirectly (parameters would be
    ; overwritten) -- the compiler checks the whole call graph and rejects
    ; any cycle.  Use an explicit stack array for recursive algorithms.
    ;
    ; Scalar methods are typed functions with an implicit receiver.  Calls use
    ; dot syntax and can chain: word$.left(1).ucase().  The existing titleCase$
    ; function below demonstrates this form; methods transpile to ordinary
    ; calls for both targets.


    ; Integer arithmetic functions
    ; a% -- first value to compare
    ; b% -- second value to compare

    ; a% -- first value to compare
    ; b% -- second value to compare

    ; value% -- number to constrain
    ; lo%    -- lower bound, inclusive
    ; hi%    -- upper bound, inclusive

    ; String functions
    ; text$ -- string to repeat
    ; n%    -- number of times to repeat it

    ; word$ -- string to title-case

    ; Local variable scoping — each function has its own i% and acc%
    ; n% -- upper bound of the sum, inclusive

    ; n% -- upper bound of the product, inclusive

    ; Global variable accessed inside a function with the global keyword
    ldc 0
    putstatic Functions/g5 I

    ; x% -- amount to add to the running total

    ; --- Exercise the functions ---

    ; print mixes string labels and numeric results directly with ;
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "max(4, 9) = "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc 4
    ldc 9
    invokestatic Functions/max (II)I
    invokevirtual java/io/PrintStream/println (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "min(4, 9) = "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc 4
    ldc 9
    invokestatic Functions/min (II)I
    invokevirtual java/io/PrintStream/println (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "clamp(15,1,10) = "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc 15
    ldc 1
    ldc 10
    invokestatic Functions/clamp (III)I
    invokevirtual java/io/PrintStream/println (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "clamp(-3,1,10) = "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc 3
    ineg
    ldc 1
    ldc 10
    invokestatic Functions/clamp (III)I
    invokevirtual java/io/PrintStream/println (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "clamp(7,1,10)  = "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc 7
    ldc 1
    ldc 10
    invokestatic Functions/clamp (III)I
    invokevirtual java/io/PrintStream/println (I)V

    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "ab"
    ldc 4
    invokestatic Functions/repeat (Ljava/lang/String;I)Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "bASCAL"
    invokestatic Functions/titleCase (Ljava/lang/String;)Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V

    ; Functions chained in expressions
    ldc 0
    ldc 5
    ineg
    invokestatic Functions/max (II)I
    ldc 100
    invokestatic Functions/min (II)I
    putstatic Functions/g4 I
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "lo = "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Functions/g4 I
    invokevirtual java/io/PrintStream/println (I)V

    ; Calling the same function twice — each result is captured separately
    ldc "x"
    ldc 3
    invokestatic Functions/repeat (Ljava/lang/String;I)Ljava/lang/String;
    putstatic Functions/g1 Ljava/lang/String;
    ldc "y"
    ldc 2
    invokestatic Functions/repeat (Ljava/lang/String;I)Ljava/lang/String;
    putstatic Functions/g2 Ljava/lang/String;
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Functions/g1 Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc " "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Functions/g2 Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V

    ; Local scoping: sumTo% and productTo% each use i% without conflict
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "sumTo(5)     = "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc 5
    invokestatic Functions/sumTo (I)I
    invokevirtual java/io/PrintStream/println (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "productTo(5) = "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc 5
    invokestatic Functions/productTo (I)I
    invokevirtual java/io/PrintStream/println (I)V

    ; Global variable shared across calls
    ldc 10
    invokestatic Functions/addToTotal (I)I
    putstatic Functions/g3 I
    ldc 5
    invokestatic Functions/addToTotal (I)I
    putstatic Functions/g3 I
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "runningTotal = "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Functions/g5 I
    invokevirtual java/io/PrintStream/println (I)V

    return
.end method
