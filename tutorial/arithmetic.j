.version 50 0
.class public Arithmetic
.super java/lang/Object

.method public static main : ([Ljava/lang/String;)V
    .limit stack 16
    .limit locals 5

    iconst_0
    istore 1
    iconst_0
    istore 2
    iconst_0
    istore 3
    iconst_0
    istore 4
    ; Tutorial — Operators and Expressions
    ; 
    ; Arithmetic:   +  -  *  /  \  MOD  ^
    ; Comparison:   =  <>  <  <=  >  >=   (result: -1 true, 0 false)
    ; Logical:      AND  OR  NOT  XOR  (bitwise — see note below)
    ; String:       + concatenates strings
    ; 
    ; Precedence (highest first):
    ; ^                 exponentiation (right-associative)
    ; unary -           negation
    ; * /               multiply / divide
    ; \                 integer (floor) division
    ; MOD               modulus (remainder)
    ; + -               add / subtract
    ; = <> < <= > >=    comparison
    ; NOT               bitwise NOT
    ; AND               bitwise AND
    ; OR                bitwise OR
    ; XOR               bitwise XOR
    ; 
    ; IMPORTANT: NOT is bitwise, so NOT 1 = -2, not 0.
    ; Test for false with (expr) = 0, not NOT expr.

    ; Arithmetic — mix labels and numbers with ;
    ldc 17
    istore 1
    ldc 5
    istore 2
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 1
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "+ "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 2
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "="
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 1
    iload 2
    iadd
    invokevirtual java/io/PrintStream/println (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 1
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "- "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 2
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "="
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 1
    iload 2
    isub
    invokevirtual java/io/PrintStream/println (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 1
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "* "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 2
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "="
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 1
    iload 2
    imul
    invokevirtual java/io/PrintStream/println (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 1
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "/ "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 2
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "="
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 1
    i2d
    iload 2
    i2d
    ddiv
    invokevirtual java/io/PrintStream/println (D)V

    ; Integer division and MOD
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 1
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "\\ "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 2
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "="
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 1
    i2d
    dup2
    ldc2_w 0.5
    dup2_x2
    pop2
    invokestatic java/lang/Math/copySign (DD)D
    dadd
    d2l
    iload 2
    i2d
    dup2
    ldc2_w 0.5
    dup2_x2
    pop2
    invokestatic java/lang/Math/copySign (DD)D
    dadd
    d2l
    ldiv
    invokevirtual java/io/PrintStream/println (J)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 1
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "MOD "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 2
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "="
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 1
    i2d
    dup2
    ldc2_w 0.5
    dup2_x2
    pop2
    invokestatic java/lang/Math/copySign (DD)D
    dadd
    d2l
    iload 2
    i2d
    dup2
    ldc2_w 0.5
    dup2_x2
    pop2
    invokestatic java/lang/Math/copySign (DD)D
    dadd
    d2l
    lrem
    invokevirtual java/io/PrintStream/println (J)V

    ; Exponentiation — right-associative
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "2 ^ 8 ="
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc 2
    i2d
    ldc 8
    i2d
    invokestatic java/lang/Math/pow (DD)D
    invokevirtual java/io/PrintStream/println (D)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "2 ^ 3 ^ 2 ="
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc 2
    i2d
    ldc 3
    i2d
    ldc 2
    i2d
    invokestatic java/lang/Math/pow (DD)D
    invokestatic java/lang/Math/pow (DD)D
    invokevirtual java/io/PrintStream/println (D)V

    ; Precedence
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc 2
    ldc 3
    ldc 4
    imul
    iadd
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc " (expect 14 — * before +)"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc 2
    ldc 3
    iadd
    ldc 4
    imul
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc " (expect 20 — parens first)"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V

    ; Comparison — -1 means true, 0 means false
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc 10
    ldc 3
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc " (expect -1)"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc 10
    ldc 3
    invokestatic java/lang/Integer/compare (II)I
    bipush 31
    ishr
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc " (expect  0)"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc 7
    ldc 7
    invokestatic java/lang/Integer/compare (II)I
    dup
    ineg
    ior
    bipush 31
    iushr
    iconst_1
    ixor
    ineg
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc " (expect -1)"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc 7
    ldc 8
    invokestatic java/lang/Integer/compare (II)I
    dup
    ineg
    ior
    bipush 31
    iushr
    ineg
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc " (expect -1)"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V

    ; Logical — AND, OR, XOR are bitwise but work correctly with 0/-1 values
    ldc 7
    istore 4
    iload 4
    ldc 0
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
    iload 4
    ldc 10
    invokestatic java/lang/Integer/compare (II)I
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
    ifeq L_if_0_else
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 4
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "is in 1..9"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
L_if_0_else:
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc 6
    i2d
    dup2
    ldc2_w 0.5
    dup2_x2
    pop2
    invokestatic java/lang/Math/copySign (DD)D
    dadd
    d2l
    ldc 3
    i2d
    dup2
    ldc2_w 0.5
    dup2_x2
    pop2
    invokestatic java/lang/Math/copySign (DD)D
    dadd
    d2l
    lxor
    invokevirtual java/io/PrintStream/print (J)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc " (expect 5 — 110 XOR 011 = 101)"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V

    ; String concatenation
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "Hello"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc ", "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc "World"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc "!"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V

    ; Unary negation
    ldc 42
    istore 3
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 3
    ineg
    invokevirtual java/io/PrintStream/println (I)V

    return
.end method
