.version 50 0
.class public Arrays
.super java/lang/Object

.field public static g1 I
.field public static g2 I
.field public static g3 I
.field public static g4 I
.field public static g5 I
.field public static a0 [I
.field public static a1 [[I
.method public static insertionSort : ()I
    .limit stack 16
    .limit locals 3

    iconst_0
    istore 0
    iconst_0
    istore 1
    iconst_0
    istore 2
    ldc 1
    istore 0
L_for_0_top:
    iload 0
    ldc 6
    iconst_1
    iadd
    ldc 1
    isub
    if_icmpgt L_for_0_end
    getstatic Arrays/a1 [I
    iload 0
    iaload
    istore 2
    iload 0
    ldc 1
    isub
    istore 1
L_while_1_top:
    iload 1
    ldc 0
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
    getstatic Arrays/a1 [I
    iload 1
    iaload
    iload 2
    invokestatic java/lang/Integer/compare (II)I
    ineg
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
    ifeq L_while_1_end
    getstatic Arrays/a1 [I
    iload 1
    ldc 1
    iadd
    getstatic Arrays/a1 [I
    iload 1
    iaload
    iastore
    iload 1
    ldc 1
    isub
    istore 1
    goto L_while_1_top
L_while_1_end:
    getstatic Arrays/a1 [I
    iload 1
    ldc 1
    iadd
    iload 2
    iastore
    iload 0
    ldc 1
    iadd
    istore 0
    goto L_for_0_top
L_for_0_end:
    ldc 0
    ireturn
    iconst_0
    ireturn
.end method

.method public static indexOf : (I)I
    .limit stack 16
    .limit locals 2

    iconst_0
    istore 1
    ldc 0
    istore 1
L_for_0_top:
    iload 1
    ldc 6
    iconst_1
    iadd
    ldc 1
    isub
    if_icmpgt L_for_0_end
    getstatic Arrays/a1 [I
    iload 1
    iaload
    iload 0
    invokestatic java/lang/Integer/compare (II)I
    dup
    ineg
    ior
    bipush 31
    iushr
    iconst_1
    ixor
    ineg
    ifeq L_if_1_else
    iload 1
    ireturn
L_if_1_else:
    iload 1
    ldc 1
    iadd
    istore 1
    goto L_for_0_top
L_for_0_end:
    ldc 1
    ineg
    ireturn
    iconst_0
    ireturn
.end method

.method public static printArray : ()I
    .limit stack 16
    .limit locals 2

    iconst_0
    istore 0
    ldc ""
    astore 1
    ldc "["
    astore 1
    ldc 0
    istore 0
L_for_0_top:
    iload 0
    ldc 6
    iconst_1
    iadd
    ldc 1
    isub
    if_icmpgt L_for_0_end
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    aload 1
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc " "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    getstatic Arrays/a1 [I
    iload 0
    iaload
    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    astore 1
    iload 0
    ldc 1
    iadd
    istore 0
    goto L_for_0_top
L_for_0_end:
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    aload 1
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc " ]"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ldc 0
    ireturn
    iconst_0
    ireturn
.end method

.method public static main : ([Ljava/lang/String;)V
    .limit stack 16
    .limit locals 6

    iconst_0
    putstatic Arrays/g1 I
    iconst_0
    putstatic Arrays/g2 I
    iconst_0
    putstatic Arrays/g3 I
    iconst_0
    putstatic Arrays/g4 I
    iconst_0
    putstatic Arrays/g5 I
    ldc 6
    iconst_1
    iadd
    multianewarray [I 1
    putstatic Arrays/a0 [I
    ldc 2
    iconst_1
    iadd
    ldc 2
    iconst_1
    iadd
    multianewarray [[I 2
    putstatic Arrays/a1 [[I
    ; Tutorial — Arrays
    ; 
    ; dim name%(size) declares a 1-D array of size+1 elements, indexed 0..size.
    ; dim name%(rows, cols) declares a 2-D array; more dimensions are allowed.
    ; Array elements are accessed with parentheses: arr%(i%) or grid%(r%, c%).
    ; 
    ; An array parameter must declare its rank with one ? per dimension:
    ; arr%(?) for 1-D, grid%(?, ?) for 2-D, and so on. At the call site, just
    ; write the plain array name -- no () and no size argument needed; the
    ; compiler already knows that parameter is an array from its declaration,
    ; and carries its size alongside it automatically. Use sizeof(arr%) inside
    ; the function body wherever the size is needed.
    ; 
    ; An array parameter defaults to byval: the function gets its own private
    ; copy, and changes never reach the caller.  Write byref to copy the
    ; result back out after the call -- insertionSort% below needs it, since
    ; its whole job is to mutate the caller's array in place.

    ; Declare and populate

    getstatic Arrays/a0 [I
    ldc 0
    ldc 64
    iastore
    getstatic Arrays/a0 [I
    ldc 1
    ldc 25
    iastore
    getstatic Arrays/a0 [I
    ldc 2
    ldc 12
    iastore
    getstatic Arrays/a0 [I
    ldc 3
    ldc 22
    iastore
    getstatic Arrays/a0 [I
    ldc 4
    ldc 3
    iastore
    getstatic Arrays/a0 [I
    ldc 5
    ldc 11
    iastore

    ; Insertion sort — sorts data%() in place
    ; arr% -- array to sort; byref because it's mutated in place

    ; Linear search — returns index or -1
    ; arr%    -- array to search; byval, since indexOf% only reads it
    ; target% -- value to search for

    ; Print the array on one line as  [ a b c ... ]
    ; arr% -- array to print; byval, since printArray% only reads it

    ; Before sort
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Before: "
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    invokestatic Arrays/printArray ()I
    putstatic Arrays/g2 I

    ; Sort and show
    invokestatic Arrays/insertionSort ()I
    putstatic Arrays/g2 I
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "After:  "
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    invokestatic Arrays/printArray ()I
    putstatic Arrays/g2 I

    ; Search
    ldc 22
    putstatic Arrays/g5 I
    getstatic Arrays/g5 I
    invokestatic Arrays/indexOf (I)I
    putstatic Arrays/g3 I
    getstatic Arrays/g3 I
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    iconst_1
    isub
    bipush 31
    ishr
    ifeq L_if_0_else
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    getstatic Arrays/g5 I
    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc " found at index "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    getstatic Arrays/g3 I
    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_if_0_end
L_if_0_else:
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    getstatic Arrays/g5 I
    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc " not found"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
L_if_0_end:

    ; 2-D array — 3×3 identity matrix
    ldc 0
    putstatic Arrays/g4 I
L_for_1_top:
    getstatic Arrays/g4 I
    ldc 2
    if_icmpgt L_for_1_end
    ldc 0
    putstatic Arrays/g1 I
L_for_2_top:
    getstatic Arrays/g1 I
    ldc 2
    if_icmpgt L_for_2_end
    getstatic Arrays/g4 I
    getstatic Arrays/g1 I
    invokestatic java/lang/Integer/compare (II)I
    dup
    ineg
    ior
    bipush 31
    iushr
    iconst_1
    ixor
    ineg
    ifeq L_if_3_else
    getstatic Arrays/a1 [[I
    getstatic Arrays/g4 I
    aaload
    getstatic Arrays/g1 I
    ldc 1
    iastore
    goto L_if_3_end
L_if_3_else:
    getstatic Arrays/a1 [[I
    getstatic Arrays/g4 I
    aaload
    getstatic Arrays/g1 I
    ldc 0
    iastore
L_if_3_end:
    getstatic Arrays/g1 I
    ldc 1
    iadd
    putstatic Arrays/g1 I
    goto L_for_2_top
L_for_2_end:
    getstatic Arrays/g4 I
    ldc 1
    iadd
    putstatic Arrays/g4 I
    goto L_for_1_top
L_for_1_end:

    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Identity matrix:"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ldc 0
    putstatic Arrays/g4 I
L_for_4_top:
    getstatic Arrays/g4 I
    ldc 2
    if_icmpgt L_for_4_end
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Arrays/a1 [[I
    getstatic Arrays/g4 I
    aaload
    ldc 0
    iaload
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Arrays/a1 [[I
    getstatic Arrays/g4 I
    aaload
    ldc 1
    iaload
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Arrays/a1 [[I
    getstatic Arrays/g4 I
    aaload
    ldc 2
    iaload
    invokevirtual java/io/PrintStream/println (I)V
    getstatic Arrays/g4 I
    ldc 1
    iadd
    putstatic Arrays/g4 I
    goto L_for_4_top
L_for_4_end:

    return
.end method
