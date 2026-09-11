.version 50 0
.class public Procedures
.super java/lang/Object

.field public static g1 I
.field public static g2 I
.field public static a0 [I
.method public static printSeparator : ()V
    .limit stack 16
    .limit locals 0

    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "----------------------------"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    return
.end method

.method public static printScore : (Ljava/lang/String;I)V
    .limit stack 16
    .limit locals 2

    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    aload 0
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc ": "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    iload 1
    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    return
.end method

.method public static printIfPass : (Ljava/lang/String;I)V
    .limit stack 16
    .limit locals 2

    iload 1
    ldc 60
    invokestatic java/lang/Integer/compare (II)I
    bipush 31
    ishr
    ifeq L_if_0_else
    return
L_if_0_else:
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    aload 0
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc " passed with "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    iload 1
    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    return
.end method

.method public static fillRange : ([II)V
    .limit stack 16
    .limit locals 3

    iconst_0
    istore 2
    ldc 0
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
    iload 1
    iastore
    iload 2
    ldc 1
    iadd
    istore 2
    goto L_for_0_top
L_for_0_end:
    return
.end method

.method public static increment : ()V
    .limit stack 16
    .limit locals 0

    getstatic Procedures/g1 I
    ldc 1
    iadd
    putstatic Procedures/g1 I
    return
.end method

.method public static main : ([Ljava/lang/String;)V
    .limit stack 16
    .limit locals 3

    iconst_0
    putstatic Procedures/g1 I
    iconst_0
    putstatic Procedures/g2 I
    ldc 5
    iconst_1
    iadd
    multianewarray [I 1
    putstatic Procedures/a0 [I
    ; Tutorial — Procedures
    ; 
    ; A procedure is like a function but returns no value.  Declare it with
    ; PROCEDURE ... END PROCEDURE.  The name must not carry a type suffix.
    ; 
    ; Variables inside a procedure are LOCAL by default: the compiler prefixes
    ; them with the procedure name.  To access a global variable, declare it
    ; inside the body with:  global varname
    ; 
    ; Use procedures for actions that produce side effects (output, file I/O,
    ; modifying arrays) rather than for computing a value.
    ; 
    ; A bare RETURN exits a procedure early.  Falling through to END PROCEDURE
    ; is also valid — an implicit RETURN is emitted.

    ; Procedure with no parameters

    ; Procedure that prints a labelled value
    ; label$ -- text shown before the score
    ; score% -- value to print

    ; Procedure with early exit
    ; name$  -- person's name
    ; score% -- score to test against the passing threshold

    ; Procedure that modifies an array in place -- byref copies the result
    ; back to the caller; the default byval would fill a private copy only.
    ; arr%   -- array to fill; byref because it's mutated in place
    ; value% -- value written into every element

    ; Procedure that uses a global variable
    ldc 0
    putstatic Procedures/g1 I


    ; --- Drive the procedures ---

    invokestatic Procedures/printSeparator ()V
    ldc "Alice"
    ldc 91
    invokestatic Procedures/printScore (Ljava/lang/String;I)V
    ldc "Bob"
    ldc 54
    invokestatic Procedures/printScore (Ljava/lang/String;I)V
    ldc "Carol"
    ldc 78
    invokestatic Procedures/printScore (Ljava/lang/String;I)V
    invokestatic Procedures/printSeparator ()V

    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Passes only:"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ldc "Alice"
    ldc 91
    invokestatic Procedures/printIfPass (Ljava/lang/String;I)V
    ldc "Bob"
    ldc 54
    invokestatic Procedures/printIfPass (Ljava/lang/String;I)V
    ldc "Carol"
    ldc 78
    invokestatic Procedures/printIfPass (Ljava/lang/String;I)V

    getstatic Procedures/a0 [I
    ldc 99
    invokestatic Procedures/fillRange ([II)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Filled array:"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ldc 0
    putstatic Procedures/g2 I
L_for_0_top:
    getstatic Procedures/g2 I
    ldc 5
    ldc 1
    isub
    if_icmpgt L_for_0_end
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
    ldc "  data%("
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    getstatic Procedures/g2 I
    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc ") = "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    getstatic Procedures/a0 [I
    getstatic Procedures/g2 I
    iaload
    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic Procedures/g2 I
    ldc 1
    iadd
    putstatic Procedures/g2 I
    goto L_for_0_top
L_for_0_end:

    invokestatic Procedures/increment ()V
    invokestatic Procedures/increment ()V
    invokestatic Procedures/increment ()V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "globalCount = "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    getstatic Procedures/g1 I
    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V

    return
.end method
