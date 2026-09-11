.version 50 0
.class public Arithmetic
.super java/lang/Object

.field public static g1 I
.field public static g2 I
.field public static g3 I
.field public static g4 I
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
    .limit locals 5

    iconst_0
    putstatic Arithmetic/g1 I
    iconst_0
    putstatic Arithmetic/g2 I
    iconst_0
    putstatic Arithmetic/g3 I
    iconst_0
    putstatic Arithmetic/g4 I
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
    putstatic Arithmetic/g1 I
    ldc 5
    putstatic Arithmetic/g2 I
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Arithmetic/g1 I
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "+ "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Arithmetic/g2 I
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "="
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Arithmetic/g1 I
    getstatic Arithmetic/g2 I
    iadd
    invokevirtual java/io/PrintStream/println (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Arithmetic/g1 I
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "- "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Arithmetic/g2 I
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "="
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Arithmetic/g1 I
    getstatic Arithmetic/g2 I
    isub
    invokevirtual java/io/PrintStream/println (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Arithmetic/g1 I
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "* "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Arithmetic/g2 I
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "="
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Arithmetic/g1 I
    getstatic Arithmetic/g2 I
    imul
    invokevirtual java/io/PrintStream/println (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Arithmetic/g1 I
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "/ "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Arithmetic/g2 I
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "="
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Arithmetic/g1 I
    i2d
    getstatic Arithmetic/g2 I
    i2d
    ddiv
    invokestatic Arithmetic/bccStr (D)Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V

    ; Integer division and MOD
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Arithmetic/g1 I
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "\\ "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Arithmetic/g2 I
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "="
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Arithmetic/g1 I
    i2d
    dup2
    ldc2_w 0.5
    dup2_x2
    pop2
    invokestatic java/lang/Math/copySign (DD)D
    dadd
    d2l
    getstatic Arithmetic/g2 I
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
    getstatic Arithmetic/g1 I
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "MOD "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Arithmetic/g2 I
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "="
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Arithmetic/g1 I
    i2d
    dup2
    ldc2_w 0.5
    dup2_x2
    pop2
    invokestatic java/lang/Math/copySign (DD)D
    dadd
    d2l
    getstatic Arithmetic/g2 I
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
    invokestatic Arithmetic/bccStr (D)Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
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
    invokestatic Arithmetic/bccStr (D)Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V

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
    putstatic Arithmetic/g4 I
    getstatic Arithmetic/g4 I
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
    getstatic Arithmetic/g4 I
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
    getstatic Arithmetic/g4 I
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
    putstatic Arithmetic/g3 I
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Arithmetic/g3 I
    ineg
    invokevirtual java/io/PrintStream/println (I)V

    return
.end method
