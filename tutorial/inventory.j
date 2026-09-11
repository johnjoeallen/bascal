.version 50 0
.class public Inventory
.super java/lang/Object

.field public static g1 I
.field public static g2 I
.field public static g3 Ljava/lang/String;
.field public static bccFiles [Ljava/io/RandomAccessFile;
.field public static bccBufs [[B
.field public static bccRecLen [I
.field public static bccStdin Ljava/io/BufferedReader;
.method public static error : (I)Ljava/lang/String;
    .limit stack 16
    .limit locals 5

    iload 0
    dup
    ldc 2
    isub
    ifeq L_select_0_case_0
    goto L_select_0_next_0
L_select_0_next_0:
    dup
    ldc 3
    isub
    ifeq L_select_0_case_1
    goto L_select_0_next_1
L_select_0_next_1:
    dup
    ldc 4
    isub
    ifeq L_select_0_case_2
    goto L_select_0_next_2
L_select_0_next_2:
    dup
    ldc 5
    isub
    ifeq L_select_0_case_3
    goto L_select_0_next_3
L_select_0_next_3:
    dup
    ldc 6
    isub
    ifeq L_select_0_case_4
    goto L_select_0_next_4
L_select_0_next_4:
    dup
    ldc 7
    isub
    ifeq L_select_0_case_5
    goto L_select_0_next_5
L_select_0_next_5:
    dup
    ldc 9
    isub
    ifeq L_select_0_case_6
    goto L_select_0_next_6
L_select_0_next_6:
    dup
    ldc 10
    isub
    ifeq L_select_0_case_7
    goto L_select_0_next_7
L_select_0_next_7:
    dup
    ldc 11
    isub
    ifeq L_select_0_case_8
    goto L_select_0_next_8
L_select_0_next_8:
    dup
    ldc 13
    isub
    ifeq L_select_0_case_9
    goto L_select_0_next_9
L_select_0_next_9:
    dup
    ldc 14
    isub
    ifeq L_select_0_case_10
    goto L_select_0_next_10
L_select_0_next_10:
    dup
    ldc 19
    isub
    ifeq L_select_0_case_11
    goto L_select_0_next_11
L_select_0_next_11:
    dup
    ldc 20
    isub
    ifeq L_select_0_case_12
    goto L_select_0_next_12
L_select_0_next_12:
    dup
    ldc 24
    isub
    ifeq L_select_0_case_13
    goto L_select_0_next_13
L_select_0_next_13:
    dup
    ldc 25
    isub
    ifeq L_select_0_case_14
    goto L_select_0_next_14
L_select_0_next_14:
    dup
    ldc 27
    isub
    ifeq L_select_0_case_15
    goto L_select_0_next_15
L_select_0_next_15:
    dup
    ldc 52
    isub
    ifeq L_select_0_case_16
    goto L_select_0_next_16
L_select_0_next_16:
    dup
    ldc 53
    isub
    ifeq L_select_0_case_17
    goto L_select_0_next_17
L_select_0_next_17:
    dup
    ldc 54
    isub
    ifeq L_select_0_case_18
    goto L_select_0_next_18
L_select_0_next_18:
    dup
    ldc 55
    isub
    ifeq L_select_0_case_19
    goto L_select_0_next_19
L_select_0_next_19:
    dup
    ldc 57
    isub
    ifeq L_select_0_case_20
    goto L_select_0_next_20
L_select_0_next_20:
    dup
    ldc 58
    isub
    ifeq L_select_0_case_21
    goto L_select_0_next_21
L_select_0_next_21:
    dup
    ldc 61
    isub
    ifeq L_select_0_case_22
    goto L_select_0_next_22
L_select_0_next_22:
    dup
    ldc 62
    isub
    ifeq L_select_0_case_23
    goto L_select_0_next_23
L_select_0_next_23:
    dup
    ldc 63
    isub
    ifeq L_select_0_case_24
    goto L_select_0_next_24
L_select_0_next_24:
    dup
    ldc 64
    isub
    ifeq L_select_0_case_25
    goto L_select_0_next_25
L_select_0_next_25:
    dup
    ldc 67
    isub
    ifeq L_select_0_case_26
    goto L_select_0_next_26
L_select_0_next_26:
    dup
    ldc 68
    isub
    ifeq L_select_0_case_27
    goto L_select_0_next_27
L_select_0_next_27:
    dup
    ldc 70
    isub
    ifeq L_select_0_case_28
    goto L_select_0_next_28
L_select_0_next_28:
    dup
    ldc 71
    isub
    ifeq L_select_0_case_29
    goto L_select_0_next_29
L_select_0_next_29:
    dup
    ldc 72
    isub
    ifeq L_select_0_case_30
    goto L_select_0_next_30
L_select_0_next_30:
    dup
    ldc 75
    isub
    ifeq L_select_0_case_31
    goto L_select_0_next_31
L_select_0_next_31:
    dup
    ldc 76
    isub
    ifeq L_select_0_case_32
    goto L_select_0_next_32
L_select_0_next_32:
    pop
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "Error "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    iload 0
    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    areturn
    goto L_select_0_end
L_select_0_case_0:
    pop
    ldc "Syntax error"
    areturn
    goto L_select_0_end
L_select_0_case_1:
    pop
    ldc "RETURN without GOSUB"
    areturn
    goto L_select_0_end
L_select_0_case_2:
    pop
    ldc "Out of DATA"
    areturn
    goto L_select_0_end
L_select_0_case_3:
    pop
    ldc "Illegal function call"
    areturn
    goto L_select_0_end
L_select_0_case_4:
    pop
    ldc "Overflow"
    areturn
    goto L_select_0_end
L_select_0_case_5:
    pop
    ldc "Out of memory"
    areturn
    goto L_select_0_end
L_select_0_case_6:
    pop
    ldc "Subscript out of range"
    areturn
    goto L_select_0_end
L_select_0_case_7:
    pop
    ldc "Duplicate Definition"
    areturn
    goto L_select_0_end
L_select_0_case_8:
    pop
    ldc "Division by zero"
    areturn
    goto L_select_0_end
L_select_0_case_9:
    pop
    ldc "Type mismatch"
    areturn
    goto L_select_0_end
L_select_0_case_10:
    pop
    ldc "Out of string space"
    areturn
    goto L_select_0_end
L_select_0_case_11:
    pop
    ldc "No RESUME"
    areturn
    goto L_select_0_end
L_select_0_case_12:
    pop
    ldc "RESUME without error"
    areturn
    goto L_select_0_end
L_select_0_case_13:
    pop
    ldc "Device timeout"
    areturn
    goto L_select_0_end
L_select_0_case_14:
    pop
    ldc "Device fault"
    areturn
    goto L_select_0_end
L_select_0_case_15:
    pop
    ldc "Out of paper"
    areturn
    goto L_select_0_end
L_select_0_case_16:
    pop
    ldc "Bad file number"
    areturn
    goto L_select_0_end
L_select_0_case_17:
    pop
    ldc "File not found"
    areturn
    goto L_select_0_end
L_select_0_case_18:
    pop
    ldc "Bad file mode"
    areturn
    goto L_select_0_end
L_select_0_case_19:
    pop
    ldc "File already open"
    areturn
    goto L_select_0_end
L_select_0_case_20:
    pop
    ldc "Device I/O error"
    areturn
    goto L_select_0_end
L_select_0_case_21:
    pop
    ldc "File already exists"
    areturn
    goto L_select_0_end
L_select_0_case_22:
    pop
    ldc "Disk full"
    areturn
    goto L_select_0_end
L_select_0_case_23:
    pop
    ldc "Input past end"
    areturn
    goto L_select_0_end
L_select_0_case_24:
    pop
    ldc "Bad record number"
    areturn
    goto L_select_0_end
L_select_0_case_25:
    pop
    ldc "Bad file name"
    areturn
    goto L_select_0_end
L_select_0_case_26:
    pop
    ldc "Too many files"
    areturn
    goto L_select_0_end
L_select_0_case_27:
    pop
    ldc "Device unavailable"
    areturn
    goto L_select_0_end
L_select_0_case_28:
    pop
    ldc "Disk write protected"
    areturn
    goto L_select_0_end
L_select_0_case_29:
    pop
    ldc "Disk not ready"
    areturn
    goto L_select_0_end
L_select_0_case_30:
    pop
    ldc "Disk media error"
    areturn
    goto L_select_0_end
L_select_0_case_31:
    pop
    ldc "Path/File access error"
    areturn
    goto L_select_0_end
L_select_0_case_32:
    pop
    ldc "Path not found"
    areturn
    goto L_select_0_end
L_select_0_end:
    ldc ""
    areturn
.end method

.method public static isEmpty : (Ljava/lang/String;)I
    .limit stack 16
    .limit locals 5

    aload 0
    iconst_0
    invokevirtual java/lang/String/charAt (I)C
    ldc 255
    invokestatic java/lang/Integer/compare (II)I
    dup
    ineg
    ior
    bipush 31
    iushr
    iconst_1
    ixor
    ineg
    ireturn
    iconst_0
    ireturn
.end method

.method public static partInRange : (I)I
    .limit stack 16
    .limit locals 5

    iload 0
    ldc 1
    invokestatic java/lang/Integer/compare (II)I
    ineg
    iconst_1
    isub
    bipush 31
    ishr
    ifeq L_if_0_else
    iload 0
    ldc 100
    invokestatic java/lang/Integer/compare (II)I
    iconst_1
    isub
    bipush 31
    ishr
    ifeq L_if_0_else
    ldc 1
    ireturn
L_if_0_else:
    ldc 0
    ireturn
    iconst_0
    ireturn
.end method

.method public static readPartNumberInput : ()Ljava/lang/String;
    .limit stack 16
    .limit locals 5

    ldc ""
    astore 0
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Input part number? "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    invokevirtual java/io/PrintStream/flush ()V
    new java/lang/ProcessBuilder
    dup
    ldc 2
    anewarray java/lang/String
    dup
    ldc 0
    ldc "stty"
    aastore
    dup
    ldc 1
    ldc "sane"
    aastore
    invokespecial java/lang/ProcessBuilder/<init> ([Ljava/lang/String;)V
    invokevirtual java/lang/ProcessBuilder/inheritIO ()Ljava/lang/ProcessBuilder;
    invokevirtual java/lang/ProcessBuilder/start ()Ljava/lang/Process;
    invokevirtual java/lang/Process/waitFor ()I
    pop
    getstatic Inventory/bccStdin Ljava/io/BufferedReader;
    invokevirtual java/io/BufferedReader/readLine ()Ljava/lang/String;
    new java/lang/ProcessBuilder
    dup
    ldc 7
    anewarray java/lang/String
    dup
    ldc 0
    ldc "stty"
    aastore
    dup
    ldc 1
    ldc "-icanon"
    aastore
    dup
    ldc 2
    ldc "-echo"
    aastore
    dup
    ldc 3
    ldc "min"
    aastore
    dup
    ldc 4
    ldc "0"
    aastore
    dup
    ldc 5
    ldc "time"
    aastore
    dup
    ldc 6
    ldc "0"
    aastore
    invokespecial java/lang/ProcessBuilder/<init> ([Ljava/lang/String;)V
    invokevirtual java/lang/ProcessBuilder/inheritIO ()Ljava/lang/ProcessBuilder;
    invokevirtual java/lang/ProcessBuilder/start ()Ljava/lang/Process;
    invokevirtual java/lang/Process/waitFor ()I
    pop
    astore 0
    aload 0
    areturn
    ldc ""
    areturn
.end method

.method public static readKey : ()Ljava/lang/String;
    .limit stack 16
    .limit locals 5

    ldc ""
    astore 0
L_do_0_top:
    getstatic java/lang/System/in Ljava/io/InputStream;
    invokevirtual java/io/InputStream/available ()I
    ifle L_condition_0
    getstatic java/lang/System/in Ljava/io/InputStream;
    invokevirtual java/io/InputStream/read ()I
    i2c
    invokestatic java/lang/String/valueOf (C)Ljava/lang/String;
    goto L_condition_1
L_condition_0:
    ldc ""
L_condition_1:
    astore 0
    aload 0
    ldc ""
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    iconst_1
    ixor
    ineg
    ifne L_do_0_end
    goto L_do_0_top
L_do_0_end:
    aload 0
    areturn
    ldc ""
    areturn
.end method

.method public static waitAnyKey : ()V
    .limit stack 16
    .limit locals 5

    ldc ""
    astore 0
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 25
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc ";"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 10
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "H"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Press the AnyKey to continue..."
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    invokevirtual java/io/PrintStream/flush ()V
L_do_0_top:
    getstatic java/lang/System/in Ljava/io/InputStream;
    invokevirtual java/io/InputStream/available ()I
    ifle L_condition_0
    getstatic java/lang/System/in Ljava/io/InputStream;
    invokevirtual java/io/InputStream/read ()I
    i2c
    invokestatic java/lang/String/valueOf (C)Ljava/lang/String;
    goto L_condition_1
L_condition_0:
    ldc ""
L_condition_1:
    astore 0
    aload 0
    ldc ""
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    iconst_1
    ixor
    ineg
    ifne L_do_0_end
    goto L_do_0_top
L_do_0_end:
    return
.end method

.method public static showMainMenu : ()V
    .limit stack 16
    .limit locals 4

    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "[2J[H"
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "[93;41m"
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "[2J[H"
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 6
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc ";"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 1
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "H"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    ; `tab(n)` passes straight through to real TAB(n), same as
    ; fhb's own `PRINT TAB(V) "..."` -- but only as a bare item in
    ; a PRINT list, juxtaposed or `;`-separated like here. Real
    ; BASCOM rejects `"literal" + tab(n) + ...` (TAB isn't a real
    ; string function you can concatenate); see printListHeader()
    ; and printReorderHeader() below, which need `;` between a
    ; preceding string and a `tab(n)` for exactly this reason.
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 30
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "G"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Inventory Program"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 20
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "G"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "1......C)heck a part"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 20
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "G"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "2......E)dit/overwrite/add a part"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 20
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "G"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "3......L)ist all"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 100
    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc "parts"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 20
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "G"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "4......A)dd stock"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 20
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "G"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "5......S)ubtract stock"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 20
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "G"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "6......R)eorder Report"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 20
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "G"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "7......eX)it to system"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    return
.end method

.method public static showBadPartNumber : ()V
    .limit stack 16
    .limit locals 4

    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "[2J[H"
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 10
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc ";"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 10
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "H"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "Part number is out of permissable range of 1 to"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 100
    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    return
.end method

.method public static showRangeRetryMessage : ()V
    .limit stack 16
    .limit locals 4

    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 10
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc ";"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 15
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "H"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "The Part number is out of permissable range of 1 to"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 100
    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 25
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc ";"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 15
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "H"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Press the Anykey to reenter part number..."
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    invokevirtual java/io/PrintStream/flush ()V
    return
.end method

.method public static showNullEntryMessage : (Ljava/lang/String;)V
    .limit stack 16
    .limit locals 5

    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 10
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc ";"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 20
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "H"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "Part number "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    aload 0
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc " is a null entry"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    return
.end method

.method public static showPartStatus : (ILjava/lang/String;IID)V
    .limit stack 16
    .limit locals 10

    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "[2J[H"
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 5
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc ";"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 1
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "H"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 20
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "G"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Inventory Status for Individual Part Number"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 20
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "G"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "==========================================="
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 20
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "G"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "     Part number:  "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    iload 0
    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 20
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "G"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "       Item name:  "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    aload 1
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 20
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "G"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "Quantity on hand:  "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    iload 2
    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 20
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "G"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "   Reorder level:  "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    iload 3
    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 20
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "G"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "      Unit price:  "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    dload 4
    invokestatic Inventory/bccStr (D)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    return
.end method

.method public static printListHeader : ()V
    .limit stack 16
    .limit locals 4

    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "[2J[H"
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 25
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "G"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "I N V E N T O R Y   L I S T I N G"
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 65
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "G"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc 100
    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc "items"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "                                          Quantity       Reorder"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc " Partno           Description             on hand         level"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    return
.end method

.method public static printInventoryLine : (ILjava/lang/String;II)V
    .limit stack 16
    .limit locals 8

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
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    iload 0
    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc "  "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    aload 1
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc "   "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    iload 2
    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc "          "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    iload 3
    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    return
.end method

.method public static printReorderHeader : ()V
    .limit stack 16
    .limit locals 4

    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "[2J[H"
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 1
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc ";"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 20
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "H"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Reorder Report"
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 55
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "G"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    invokestatic java/time/LocalDate/now ()Ljava/time/LocalDate;
    ldc "MM-dd-yyyy"
    invokestatic java/time/format/DateTimeFormatter/ofPattern (Ljava/lang/String;)Ljava/time/format/DateTimeFormatter;
    invokevirtual java/time/LocalDate/format (Ljava/time/format/DateTimeFormatter;)Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "                                             Quantity       Reorder"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "    Partno           Description             on hand         level"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "   =======  ==============================   ========       ======="
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    return
.end method

.method public static printReorderLine : (ILjava/lang/String;II)V
    .limit stack 16
    .limit locals 8

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
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "  "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    iload 0
    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc "  "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    aload 1
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc "   "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    iload 2
    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc "          "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    iload 3
    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    return
.end method

.method public static gatherPartDetails : (I[Ljava/lang/String;[I[I[D)V
    .limit stack 16
    .limit locals 14

    aload 1
    iconst_0
    aaload
    astore 5
    aload 2
    iconst_0
    iaload
    istore 6
    aload 3
    iconst_0
    iaload
    istore 7
    aload 4
    iconst_0
    daload
    dstore 8
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "[2J[H"
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 4
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc ";"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 20
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "H"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Adding or Overwriting a Record"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 8
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc ";"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 20
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "H"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "Record/Partno"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    iload 0
    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 11
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc ";"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 39
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "H"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "------------------------------"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 10
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc ";"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 20
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "H"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "      Description? "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    invokevirtual java/io/PrintStream/flush ()V
    new java/lang/ProcessBuilder
    dup
    ldc 2
    anewarray java/lang/String
    dup
    ldc 0
    ldc "stty"
    aastore
    dup
    ldc 1
    ldc "sane"
    aastore
    invokespecial java/lang/ProcessBuilder/<init> ([Ljava/lang/String;)V
    invokevirtual java/lang/ProcessBuilder/inheritIO ()Ljava/lang/ProcessBuilder;
    invokevirtual java/lang/ProcessBuilder/start ()Ljava/lang/Process;
    invokevirtual java/lang/Process/waitFor ()I
    pop
    getstatic Inventory/bccStdin Ljava/io/BufferedReader;
    invokevirtual java/io/BufferedReader/readLine ()Ljava/lang/String;
    new java/lang/ProcessBuilder
    dup
    ldc 7
    anewarray java/lang/String
    dup
    ldc 0
    ldc "stty"
    aastore
    dup
    ldc 1
    ldc "-icanon"
    aastore
    dup
    ldc 2
    ldc "-echo"
    aastore
    dup
    ldc 3
    ldc "min"
    aastore
    dup
    ldc 4
    ldc "0"
    aastore
    dup
    ldc 5
    ldc "time"
    aastore
    dup
    ldc 6
    ldc "0"
    aastore
    invokespecial java/lang/ProcessBuilder/<init> ([Ljava/lang/String;)V
    invokevirtual java/lang/ProcessBuilder/inheritIO ()Ljava/lang/ProcessBuilder;
    invokevirtual java/lang/ProcessBuilder/start ()Ljava/lang/Process;
    invokevirtual java/lang/Process/waitFor ()I
    pop
    astore 5
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 12
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc ";"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 20
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "H"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Quantity in stock? "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    invokevirtual java/io/PrintStream/flush ()V
    new java/lang/ProcessBuilder
    dup
    ldc 2
    anewarray java/lang/String
    dup
    ldc 0
    ldc "stty"
    aastore
    dup
    ldc 1
    ldc "sane"
    aastore
    invokespecial java/lang/ProcessBuilder/<init> ([Ljava/lang/String;)V
    invokevirtual java/lang/ProcessBuilder/inheritIO ()Ljava/lang/ProcessBuilder;
    invokevirtual java/lang/ProcessBuilder/start ()Ljava/lang/Process;
    invokevirtual java/lang/Process/waitFor ()I
    pop
    getstatic Inventory/bccStdin Ljava/io/BufferedReader;
    invokevirtual java/io/BufferedReader/readLine ()Ljava/lang/String;
    new java/lang/ProcessBuilder
    dup
    ldc 7
    anewarray java/lang/String
    dup
    ldc 0
    ldc "stty"
    aastore
    dup
    ldc 1
    ldc "-icanon"
    aastore
    dup
    ldc 2
    ldc "-echo"
    aastore
    dup
    ldc 3
    ldc "min"
    aastore
    dup
    ldc 4
    ldc "0"
    aastore
    dup
    ldc 5
    ldc "time"
    aastore
    dup
    ldc 6
    ldc "0"
    aastore
    invokespecial java/lang/ProcessBuilder/<init> ([Ljava/lang/String;)V
    invokevirtual java/lang/ProcessBuilder/inheritIO ()Ljava/lang/ProcessBuilder;
    invokevirtual java/lang/ProcessBuilder/start ()Ljava/lang/Process;
    invokevirtual java/lang/Process/waitFor ()I
    pop
    invokevirtual java/lang/String/trim ()Ljava/lang/String;
    invokestatic java/lang/Integer/parseInt (Ljava/lang/String;)I
    istore 6
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 14
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc ";"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 20
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "H"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "    Reorder level? "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    invokevirtual java/io/PrintStream/flush ()V
    new java/lang/ProcessBuilder
    dup
    ldc 2
    anewarray java/lang/String
    dup
    ldc 0
    ldc "stty"
    aastore
    dup
    ldc 1
    ldc "sane"
    aastore
    invokespecial java/lang/ProcessBuilder/<init> ([Ljava/lang/String;)V
    invokevirtual java/lang/ProcessBuilder/inheritIO ()Ljava/lang/ProcessBuilder;
    invokevirtual java/lang/ProcessBuilder/start ()Ljava/lang/Process;
    invokevirtual java/lang/Process/waitFor ()I
    pop
    getstatic Inventory/bccStdin Ljava/io/BufferedReader;
    invokevirtual java/io/BufferedReader/readLine ()Ljava/lang/String;
    new java/lang/ProcessBuilder
    dup
    ldc 7
    anewarray java/lang/String
    dup
    ldc 0
    ldc "stty"
    aastore
    dup
    ldc 1
    ldc "-icanon"
    aastore
    dup
    ldc 2
    ldc "-echo"
    aastore
    dup
    ldc 3
    ldc "min"
    aastore
    dup
    ldc 4
    ldc "0"
    aastore
    dup
    ldc 5
    ldc "time"
    aastore
    dup
    ldc 6
    ldc "0"
    aastore
    invokespecial java/lang/ProcessBuilder/<init> ([Ljava/lang/String;)V
    invokevirtual java/lang/ProcessBuilder/inheritIO ()Ljava/lang/ProcessBuilder;
    invokevirtual java/lang/ProcessBuilder/start ()Ljava/lang/Process;
    invokevirtual java/lang/Process/waitFor ()I
    pop
    invokevirtual java/lang/String/trim ()Ljava/lang/String;
    invokestatic java/lang/Integer/parseInt (Ljava/lang/String;)I
    istore 7
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 16
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc ";"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 20
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "H"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "       Unit price? "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    invokevirtual java/io/PrintStream/flush ()V
    new java/lang/ProcessBuilder
    dup
    ldc 2
    anewarray java/lang/String
    dup
    ldc 0
    ldc "stty"
    aastore
    dup
    ldc 1
    ldc "sane"
    aastore
    invokespecial java/lang/ProcessBuilder/<init> ([Ljava/lang/String;)V
    invokevirtual java/lang/ProcessBuilder/inheritIO ()Ljava/lang/ProcessBuilder;
    invokevirtual java/lang/ProcessBuilder/start ()Ljava/lang/Process;
    invokevirtual java/lang/Process/waitFor ()I
    pop
    getstatic Inventory/bccStdin Ljava/io/BufferedReader;
    invokevirtual java/io/BufferedReader/readLine ()Ljava/lang/String;
    new java/lang/ProcessBuilder
    dup
    ldc 7
    anewarray java/lang/String
    dup
    ldc 0
    ldc "stty"
    aastore
    dup
    ldc 1
    ldc "-icanon"
    aastore
    dup
    ldc 2
    ldc "-echo"
    aastore
    dup
    ldc 3
    ldc "min"
    aastore
    dup
    ldc 4
    ldc "0"
    aastore
    dup
    ldc 5
    ldc "time"
    aastore
    dup
    ldc 6
    ldc "0"
    aastore
    invokespecial java/lang/ProcessBuilder/<init> ([Ljava/lang/String;)V
    invokevirtual java/lang/ProcessBuilder/inheritIO ()Ljava/lang/ProcessBuilder;
    invokevirtual java/lang/ProcessBuilder/start ()Ljava/lang/Process;
    invokevirtual java/lang/Process/waitFor ()I
    pop
    invokevirtual java/lang/String/trim ()Ljava/lang/String;
    invokestatic java/lang/Double/parseDouble (Ljava/lang/String;)D
    dstore 8
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 18
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc ";"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 20
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "H"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Is information correct (Y/N)?"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    aload 1
    iconst_0
    aload 5
    aastore
    aload 2
    iconst_0
    iload 6
    iastore
    aload 3
    iconst_0
    iload 7
    iastore
    aload 4
    iconst_0
    dload 8
    dastore
    return
.end method

.method public static showAddStockScreen : (ILjava/lang/String;II)V
    .limit stack 16
    .limit locals 8

    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "[2J[H"
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 4
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc ";"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 25
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "H"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Add to an inventory part number"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 5
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc ";"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 25
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "H"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "==============================="
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 8
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc ";"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 20
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "H"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "     Part number: "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    iload 0
    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 9
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc ";"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 20
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "H"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "Item description: "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    aload 1
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 10
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc ";"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 20
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "H"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "Quantity on hand: "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    iload 2
    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 11
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc ";"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 20
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "H"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "   Reorder Level: "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    iload 3
    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    return
.end method

.method public static showNegativeQtyWarning : ()V
    .limit stack 16
    .limit locals 4

    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 17
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc ";"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 15
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "H"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "The quantity to add must NOT be a negative number"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 25
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc ";"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 1
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "H"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Please press the Anykey to reenter quantity to add..."
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    invokevirtual java/io/PrintStream/flush ()V
    return
.end method

.method public static showSubtractStockScreen : (ILjava/lang/String;II)V
    .limit stack 16
    .limit locals 8

    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "[2J[H"
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 4
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc ";"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 20
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "H"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Subtract an inventory part number"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 5
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc ";"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 20
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "H"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "================================="
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 8
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc ";"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 20
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "H"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "         Part number: "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    iload 0
    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 9
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc ";"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 20
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "H"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "    Item description: "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    aload 1
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 10
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc ";"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 20
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "H"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "    Quantity on hand: "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    iload 2
    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 11
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc ";"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 20
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "H"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "       Reorder Level: "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    iload 3
    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    return
.end method

.method public static showOverSubtractWarning : (I)V
    .limit stack 16
    .limit locals 5

    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 17
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc ";"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 5
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "H"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "The quantity to SUBTRACT must NOT result in NEGATIVE inventory"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 18
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc ";"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 5
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "H"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "Only"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    iload 0
    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc " IN STOCK"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 25
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc ";"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 1
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "H"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Please press the Anykey to reenter quantity to subtract..."
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    invokevirtual java/io/PrintStream/flush ()V
    return
.end method

.method public static checkPart : ()V
    .limit stack 16
    .limit locals 14

    iconst_0
    istore 0
    ldc ""
    astore 1
    ldc ""
    astore 2
    iconst_0
    istore 3
    ldc ""
    astore 4
    iconst_0
    istore 5
    dconst_0
    dstore 6
    iconst_0
    istore 8
    iconst_0
    istore 9
    ; global inv
    invokestatic Inventory/readPartNumberInput ()Ljava/lang/String;
    astore 1
    aload 1
    invokestatic java/lang/Double/parseDouble (Ljava/lang/String;)D
    dup2
    ldc2_w 0.5
    dup2_x2
    pop2
    invokestatic java/lang/Math/copySign (DD)D
    dadd
    d2l
    l2i
    istore 0
    iload 0
    invokestatic Inventory/partInRange (I)I
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
    invokestatic Inventory/showBadPartNumber ()V
    invokestatic Inventory/waitAnyKey ()V
    return
L_if_0_else:
    ; BASCAL-ism: `let p = inv[part%]` reads record `part%` of the
    ; `inv` file into a local record variable `p` -- one expression
    ; for what fhb's `GET #1, PART!` plus five separate field reads
    ; (F$, D$, CVI(Q$), CVI(R$), CVS(P$)) did by hand. The write
    ; side, `inv[part%] = { ... }` (see editRecord() below), is the
    ; same sugar for PUT plus the LSET/MKx$ packing it replaces.
    ; let p = inv[...]  (whole-record read)
    getstatic Inventory/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    dup
    iload 0
    i2l
    lconst_1
    lsub
    getstatic Inventory/bccRecLen [I
    ldc 1
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic Inventory/bccBufs [[B
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/read ([B)I
    pop
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 0
    ldc 1
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    invokevirtual java/lang/String/length ()I
    istore 5
L_while_1_top:
    iload 5
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_while_1_end
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 0
    ldc 1
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    iload 5
    iconst_1
    isub
    dup
    ldc 1
    iadd
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    ldc " "
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ineg
    ifeq L_while_1_end
    iload 5
    ldc 1
    isub
    istore 5
    goto L_while_1_top
L_while_1_end:
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 0
    ldc 1
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    iconst_0
    iload 5
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    astore 4
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 1
    ldc 30
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    invokevirtual java/lang/String/length ()I
    istore 3
L_while_2_top:
    iload 3
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_while_2_end
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 1
    ldc 30
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    iload 3
    iconst_1
    isub
    dup
    ldc 1
    iadd
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    ldc " "
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ineg
    ifeq L_while_2_end
    iload 3
    ldc 1
    isub
    istore 3
    goto L_while_2_top
L_while_2_end:
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 1
    ldc 30
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    iconst_0
    iload 3
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    astore 2
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 31
    ldc 2
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    invokestatic java/nio/ByteBuffer/wrap ([B)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/getShort ()S
    istore 8
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 33
    ldc 2
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    invokestatic java/nio/ByteBuffer/wrap ([B)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/getShort ()S
    istore 9
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 35
    ldc 4
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    invokestatic java/nio/ByteBuffer/wrap ([B)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/getFloat ()F
    f2d
    dstore 6
    aload 4
    invokestatic Inventory/isEmpty (Ljava/lang/String;)I
    ifeq L_if_3_else
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "[2J[H"
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 10
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc ";"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 18
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "H"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "Part number"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    iload 0
    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc "is still a null entry at this time"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    invokestatic Inventory/waitAnyKey ()V
    return
L_if_3_else:
    iload 0
    aload 2
    iload 8
    iload 9
    dload 6
    invokestatic Inventory/showPartStatus (ILjava/lang/String;IID)V
    invokestatic Inventory/waitAnyKey ()V
    return
.end method

.method public static editRecord : ()V
    .limit stack 16
    .limit locals 20

    ldc ""
    astore 0
    dconst_0
    dstore 1
    iconst_0
    istore 3
    iconst_0
    istore 4
    ldc ""
    astore 5
    iconst_0
    istore 6
    ldc ""
    astore 7
    ldc ""
    astore 8
    iconst_0
    istore 9
    ldc ""
    astore 10
    iconst_0
    istore 11
    dconst_0
    dstore 12
    iconst_0
    istore 14
    iconst_0
    istore 15
    ; global inv
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "[2J[H"
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 10
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc ";"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 20
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "H"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    invokestatic Inventory/readPartNumberInput ()Ljava/lang/String;
    astore 7
    aload 7
    invokestatic java/lang/Double/parseDouble (Ljava/lang/String;)D
    dup2
    ldc2_w 0.5
    dup2_x2
    pop2
    invokestatic java/lang/Math/copySign (DD)D
    dadd
    d2l
    l2i
    istore 6
    iload 6
    invokestatic Inventory/partInRange (I)I
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
    invokestatic Inventory/showBadPartNumber ()V
    invokestatic Inventory/waitAnyKey ()V
    return
L_if_0_else:
    ; let p = inv[...]  (whole-record read)
    getstatic Inventory/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    dup
    iload 6
    i2l
    lconst_1
    lsub
    getstatic Inventory/bccRecLen [I
    ldc 1
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic Inventory/bccBufs [[B
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/read ([B)I
    pop
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 0
    ldc 1
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    invokevirtual java/lang/String/length ()I
    istore 11
L_while_1_top:
    iload 11
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_while_1_end
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 0
    ldc 1
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    iload 11
    iconst_1
    isub
    dup
    ldc 1
    iadd
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    ldc " "
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ineg
    ifeq L_while_1_end
    iload 11
    ldc 1
    isub
    istore 11
    goto L_while_1_top
L_while_1_end:
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 0
    ldc 1
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    iconst_0
    iload 11
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    astore 10
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 1
    ldc 30
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    invokevirtual java/lang/String/length ()I
    istore 9
L_while_2_top:
    iload 9
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_while_2_end
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 1
    ldc 30
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    iload 9
    iconst_1
    isub
    dup
    ldc 1
    iadd
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    ldc " "
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ineg
    ifeq L_while_2_end
    iload 9
    ldc 1
    isub
    istore 9
    goto L_while_2_top
L_while_2_end:
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 1
    ldc 30
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    iconst_0
    iload 9
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    astore 8
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 31
    ldc 2
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    invokestatic java/nio/ByteBuffer/wrap ([B)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/getShort ()S
    istore 14
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 33
    ldc 2
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    invokestatic java/nio/ByteBuffer/wrap ([B)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/getShort ()S
    istore 15
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 35
    ldc 4
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    invokestatic java/nio/ByteBuffer/wrap ([B)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/getFloat ()F
    f2d
    dstore 12
    aload 10
    invokestatic Inventory/isEmpty (Ljava/lang/String;)I
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
    ifeq L_if_3_else
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 12
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc ";"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 20
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "H"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Overwrite existing part data?"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    invokestatic Inventory/readKey ()Ljava/lang/String;
    astore 5
    aload 5
    ldc "Y"
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    iconst_1
    ixor
    ineg
    ifeq L_if_4_else
    aload 5
    ldc "y"
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    iconst_1
    ixor
    ineg
    ifeq L_if_4_else
    return
L_if_4_else:
L_if_3_else:

L_do_5_top:
    iload 6
    iconst_1
    anewarray java/lang/String
    dup
    iconst_0
    aload 0
    aastore
    astore 16
    aload 16
    iconst_1
    newarray int
    dup
    iconst_0
    iload 3
    iastore
    astore 17
    aload 17
    iconst_1
    newarray int
    dup
    iconst_0
    iload 4
    iastore
    astore 18
    aload 18
    iconst_1
    newarray double
    dup
    iconst_0
    dload 1
    dastore
    astore 19
    aload 19
    invokestatic Inventory/gatherPartDetails (I[Ljava/lang/String;[I[I[D)V
    aload 16
    iconst_0
    aaload
    astore 0
    aload 17
    iconst_0
    iaload
    istore 3
    aload 18
    iconst_0
    iaload
    istore 4
    aload 19
    iconst_0
    daload
    dstore 1
    invokestatic Inventory/readKey ()Ljava/lang/String;
    astore 5
    aload 5
    ldc "Y"
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ineg
    ifne L_do_5_end
    aload 5
    ldc "y"
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ineg
    ifne L_do_5_end
    goto L_do_5_top
L_do_5_end:
    ; inv[...] = { ... }  (whole-record write)
    ldc "1"
    ldc 1
    newarray char
    dup
    bipush 32
    invokestatic java/util/Arrays/fill ([CC)V
    new java/lang/String
    dup_x1
    swap
    invokespecial java/lang/String/<init> ([C)V
    invokevirtual java/lang/String/concat (Ljava/lang/String;)Ljava/lang/String;
    iconst_0
    ldc 1
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    iconst_0
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    ldc 0
    ldc 1
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    aload 0
    ldc 30
    newarray char
    dup
    bipush 32
    invokestatic java/util/Arrays/fill ([CC)V
    new java/lang/String
    dup_x1
    swap
    invokespecial java/lang/String/<init> ([C)V
    invokevirtual java/lang/String/concat (Ljava/lang/String;)Ljava/lang/String;
    iconst_0
    ldc 30
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    iconst_0
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    ldc 1
    ldc 30
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    ldc 2
    invokestatic java/nio/ByteBuffer/allocate (I)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    iload 3
    i2s
    invokevirtual java/nio/ByteBuffer/putShort (S)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/array ()[B
    new java/lang/String
    dup_x1
    swap
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BLjava/nio/charset/Charset;)V
    ldc 2
    newarray char
    dup
    bipush 32
    invokestatic java/util/Arrays/fill ([CC)V
    new java/lang/String
    dup_x1
    swap
    invokespecial java/lang/String/<init> ([C)V
    invokevirtual java/lang/String/concat (Ljava/lang/String;)Ljava/lang/String;
    iconst_0
    ldc 2
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    iconst_0
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    ldc 31
    ldc 2
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    ldc 2
    invokestatic java/nio/ByteBuffer/allocate (I)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    iload 4
    i2s
    invokevirtual java/nio/ByteBuffer/putShort (S)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/array ()[B
    new java/lang/String
    dup_x1
    swap
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BLjava/nio/charset/Charset;)V
    ldc 2
    newarray char
    dup
    bipush 32
    invokestatic java/util/Arrays/fill ([CC)V
    new java/lang/String
    dup_x1
    swap
    invokespecial java/lang/String/<init> ([C)V
    invokevirtual java/lang/String/concat (Ljava/lang/String;)Ljava/lang/String;
    iconst_0
    ldc 2
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    iconst_0
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    ldc 33
    ldc 2
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    ldc 4
    invokestatic java/nio/ByteBuffer/allocate (I)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    dload 1
    d2f
    invokevirtual java/nio/ByteBuffer/putFloat (F)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/array ()[B
    new java/lang/String
    dup_x1
    swap
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BLjava/nio/charset/Charset;)V
    ldc 4
    newarray char
    dup
    bipush 32
    invokestatic java/util/Arrays/fill ([CC)V
    new java/lang/String
    dup_x1
    swap
    invokespecial java/lang/String/<init> ([C)V
    invokevirtual java/lang/String/concat (Ljava/lang/String;)Ljava/lang/String;
    iconst_0
    ldc 4
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    iconst_0
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    ldc 35
    ldc 4
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    getstatic Inventory/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    dup
    iload 6
    i2l
    lconst_1
    lsub
    getstatic Inventory/bccRecLen [I
    ldc 1
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic Inventory/bccBufs [[B
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/write ([B)V
    return
.end method

.method public static listAll : ()V
    .limit stack 16
    .limit locals 14

    iconst_0
    istore 0
    ldc ""
    astore 1
    iconst_0
    istore 2
    ldc ""
    astore 3
    iconst_0
    istore 4
    dconst_0
    dstore 5
    iconst_0
    istore 7
    iconst_0
    istore 8
    iconst_0
    istore 9
    ; global inv
    invokestatic Inventory/printListHeader ()V
    ldc 0
    istore 9
    ldc 1
    istore 0
L_for_0_top:
    iload 0
    ldc 100
    if_icmpgt L_for_0_end
    ; let p = inv[...]  (whole-record read)
    getstatic Inventory/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    dup
    iload 0
    i2l
    lconst_1
    lsub
    getstatic Inventory/bccRecLen [I
    ldc 1
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic Inventory/bccBufs [[B
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/read ([B)I
    pop
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 0
    ldc 1
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    invokevirtual java/lang/String/length ()I
    istore 4
L_while_1_top:
    iload 4
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_while_1_end
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 0
    ldc 1
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    iload 4
    iconst_1
    isub
    dup
    ldc 1
    iadd
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    ldc " "
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ineg
    ifeq L_while_1_end
    iload 4
    ldc 1
    isub
    istore 4
    goto L_while_1_top
L_while_1_end:
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 0
    ldc 1
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    iconst_0
    iload 4
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    astore 3
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 1
    ldc 30
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    invokevirtual java/lang/String/length ()I
    istore 2
L_while_2_top:
    iload 2
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_while_2_end
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 1
    ldc 30
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    iload 2
    iconst_1
    isub
    dup
    ldc 1
    iadd
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    ldc " "
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ineg
    ifeq L_while_2_end
    iload 2
    ldc 1
    isub
    istore 2
    goto L_while_2_top
L_while_2_end:
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 1
    ldc 30
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    iconst_0
    iload 2
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    astore 1
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 31
    ldc 2
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    invokestatic java/nio/ByteBuffer/wrap ([B)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/getShort ()S
    istore 7
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 33
    ldc 2
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    invokestatic java/nio/ByteBuffer/wrap ([B)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/getShort ()S
    istore 8
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 35
    ldc 4
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    invokestatic java/nio/ByteBuffer/wrap ([B)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/getFloat ()F
    f2d
    dstore 5
    iload 0
    aload 1
    iload 7
    iload 8
    invokestatic Inventory/printInventoryLine (ILjava/lang/String;II)V
    iload 9
    ldc 1
    iadd
    istore 9
    iload 9
    ldc 20
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
    invokestatic Inventory/waitAnyKey ()V
    ldc 0
    istore 9
    ; Redraw for the next page rather than let it keep scrolling
    ; past row 25 -- see printListHeader()'s own note on why a
    ; fixed-row prompt can't coexist with unbounded scrolling.
    iload 0
    ldc 100
    invokestatic java/lang/Integer/compare (II)I
    bipush 31
    ishr
    ifeq L_if_4_else
    invokestatic Inventory/printListHeader ()V
L_if_4_else:
L_if_3_else:
    iload 0
    ldc 1
    iadd
    istore 0
    goto L_for_0_top
L_for_0_end:
    return
.end method

.method public static addStock : ()V
    .limit stack 16
    .limit locals 17

    iconst_0
    istore 0
    ldc ""
    astore 1
    iconst_0
    istore 2
    ldc ""
    astore 3
    ldc ""
    astore 4
    iconst_0
    istore 5
    ldc ""
    astore 6
    iconst_0
    istore 7
    dconst_0
    dstore 8
    iconst_0
    istore 10
    iconst_0
    istore 11
    iconst_0
    istore 12
    ; global inv
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "[2J[H"
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 5
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc ";"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 25
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "H"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "A D D I N G   S T O C K"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V

L_do_0_top:
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 8
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc ";"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 25
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "H"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    invokestatic Inventory/readPartNumberInput ()Ljava/lang/String;
    astore 3
    aload 3
    invokestatic java/lang/Double/parseDouble (Ljava/lang/String;)D
    dup2
    ldc2_w 0.5
    dup2_x2
    pop2
    invokestatic java/lang/Math/copySign (DD)D
    dadd
    d2l
    l2i
    istore 2
    iload 2
    invokestatic Inventory/partInRange (I)I
    istore 12
    iload 12
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
    ifeq L_if_1_else
    invokestatic Inventory/showRangeRetryMessage ()V
    invokestatic Inventory/readKey ()Ljava/lang/String;
    pop
L_if_1_else:
    iload 12
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    dup
    ineg
    ior
    bipush 31
    iushr
    ineg
    ifne L_do_0_end
    goto L_do_0_top
L_do_0_end:

    ; let p = inv[...]  (whole-record read)
    getstatic Inventory/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    dup
    iload 2
    i2l
    lconst_1
    lsub
    getstatic Inventory/bccRecLen [I
    ldc 1
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic Inventory/bccBufs [[B
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/read ([B)I
    pop
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 0
    ldc 1
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    invokevirtual java/lang/String/length ()I
    istore 7
L_while_2_top:
    iload 7
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_while_2_end
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 0
    ldc 1
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    iload 7
    iconst_1
    isub
    dup
    ldc 1
    iadd
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    ldc " "
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ineg
    ifeq L_while_2_end
    iload 7
    ldc 1
    isub
    istore 7
    goto L_while_2_top
L_while_2_end:
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 0
    ldc 1
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    iconst_0
    iload 7
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    astore 6
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 1
    ldc 30
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    invokevirtual java/lang/String/length ()I
    istore 5
L_while_3_top:
    iload 5
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_while_3_end
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 1
    ldc 30
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    iload 5
    iconst_1
    isub
    dup
    ldc 1
    iadd
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    ldc " "
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ineg
    ifeq L_while_3_end
    iload 5
    ldc 1
    isub
    istore 5
    goto L_while_3_top
L_while_3_end:
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 1
    ldc 30
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    iconst_0
    iload 5
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    astore 4
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 31
    ldc 2
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    invokestatic java/nio/ByteBuffer/wrap ([B)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/getShort ()S
    istore 10
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 33
    ldc 2
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    invokestatic java/nio/ByteBuffer/wrap ([B)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/getShort ()S
    istore 11
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 35
    ldc 4
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    invokestatic java/nio/ByteBuffer/wrap ([B)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/getFloat ()F
    f2d
    dstore 8
    aload 6
    invokestatic Inventory/isEmpty (Ljava/lang/String;)I
    ifeq L_if_4_else
    aload 3
    invokestatic Inventory/showNullEntryMessage (Ljava/lang/String;)V
    invokestatic Inventory/readKey ()Ljava/lang/String;
    pop
    return
L_if_4_else:

L_do_5_top:
    iload 2
    aload 4
    iload 10
    iload 11
    invokestatic Inventory/showAddStockScreen (ILjava/lang/String;II)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 14
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc ";"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 20
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "H"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc " Quantity to add? "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    invokevirtual java/io/PrintStream/flush ()V
    new java/lang/ProcessBuilder
    dup
    ldc 2
    anewarray java/lang/String
    dup
    ldc 0
    ldc "stty"
    aastore
    dup
    ldc 1
    ldc "sane"
    aastore
    invokespecial java/lang/ProcessBuilder/<init> ([Ljava/lang/String;)V
    invokevirtual java/lang/ProcessBuilder/inheritIO ()Ljava/lang/ProcessBuilder;
    invokevirtual java/lang/ProcessBuilder/start ()Ljava/lang/Process;
    invokevirtual java/lang/Process/waitFor ()I
    pop
    getstatic Inventory/bccStdin Ljava/io/BufferedReader;
    invokevirtual java/io/BufferedReader/readLine ()Ljava/lang/String;
    new java/lang/ProcessBuilder
    dup
    ldc 7
    anewarray java/lang/String
    dup
    ldc 0
    ldc "stty"
    aastore
    dup
    ldc 1
    ldc "-icanon"
    aastore
    dup
    ldc 2
    ldc "-echo"
    aastore
    dup
    ldc 3
    ldc "min"
    aastore
    dup
    ldc 4
    ldc "0"
    aastore
    dup
    ldc 5
    ldc "time"
    aastore
    dup
    ldc 6
    ldc "0"
    aastore
    invokespecial java/lang/ProcessBuilder/<init> ([Ljava/lang/String;)V
    invokevirtual java/lang/ProcessBuilder/inheritIO ()Ljava/lang/ProcessBuilder;
    invokevirtual java/lang/ProcessBuilder/start ()Ljava/lang/Process;
    invokevirtual java/lang/Process/waitFor ()I
    pop
    astore 1
    aload 1
    invokestatic java/lang/Double/parseDouble (Ljava/lang/String;)D
    dup2
    ldc2_w 0.5
    dup2_x2
    pop2
    invokestatic java/lang/Math/copySign (DD)D
    dadd
    d2l
    l2i
    istore 0
    iload 0
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    bipush 31
    ishr
    ifeq L_if_6_else
    invokestatic Inventory/showNegativeQtyWarning ()V
    invokestatic Inventory/readKey ()Ljava/lang/String;
    pop
L_if_6_else:
    iload 0
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    iconst_1
    isub
    bipush 31
    ishr
    ifne L_do_5_end
    goto L_do_5_top
L_do_5_end:

    iload 10
    iload 0
    iadd
    istore 10
    ; inv[...] = p  (write back a let-bound record)
    aload 6
    ldc 1
    newarray char
    dup
    bipush 32
    invokestatic java/util/Arrays/fill ([CC)V
    new java/lang/String
    dup_x1
    swap
    invokespecial java/lang/String/<init> ([C)V
    invokevirtual java/lang/String/concat (Ljava/lang/String;)Ljava/lang/String;
    iconst_0
    ldc 1
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    iconst_0
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    ldc 0
    ldc 1
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    aload 4
    ldc 30
    newarray char
    dup
    bipush 32
    invokestatic java/util/Arrays/fill ([CC)V
    new java/lang/String
    dup_x1
    swap
    invokespecial java/lang/String/<init> ([C)V
    invokevirtual java/lang/String/concat (Ljava/lang/String;)Ljava/lang/String;
    iconst_0
    ldc 30
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    iconst_0
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    ldc 1
    ldc 30
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    ldc 2
    invokestatic java/nio/ByteBuffer/allocate (I)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    iload 10
    i2s
    invokevirtual java/nio/ByteBuffer/putShort (S)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/array ()[B
    new java/lang/String
    dup_x1
    swap
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BLjava/nio/charset/Charset;)V
    ldc 2
    newarray char
    dup
    bipush 32
    invokestatic java/util/Arrays/fill ([CC)V
    new java/lang/String
    dup_x1
    swap
    invokespecial java/lang/String/<init> ([C)V
    invokevirtual java/lang/String/concat (Ljava/lang/String;)Ljava/lang/String;
    iconst_0
    ldc 2
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    iconst_0
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    ldc 31
    ldc 2
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    ldc 2
    invokestatic java/nio/ByteBuffer/allocate (I)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    iload 11
    i2s
    invokevirtual java/nio/ByteBuffer/putShort (S)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/array ()[B
    new java/lang/String
    dup_x1
    swap
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BLjava/nio/charset/Charset;)V
    ldc 2
    newarray char
    dup
    bipush 32
    invokestatic java/util/Arrays/fill ([CC)V
    new java/lang/String
    dup_x1
    swap
    invokespecial java/lang/String/<init> ([C)V
    invokevirtual java/lang/String/concat (Ljava/lang/String;)Ljava/lang/String;
    iconst_0
    ldc 2
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    iconst_0
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    ldc 33
    ldc 2
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    ldc 4
    invokestatic java/nio/ByteBuffer/allocate (I)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    dload 8
    d2f
    invokevirtual java/nio/ByteBuffer/putFloat (F)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/array ()[B
    new java/lang/String
    dup_x1
    swap
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BLjava/nio/charset/Charset;)V
    ldc 4
    newarray char
    dup
    bipush 32
    invokestatic java/util/Arrays/fill ([CC)V
    new java/lang/String
    dup_x1
    swap
    invokespecial java/lang/String/<init> ([C)V
    invokevirtual java/lang/String/concat (Ljava/lang/String;)Ljava/lang/String;
    iconst_0
    ldc 4
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    iconst_0
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    ldc 35
    ldc 4
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    getstatic Inventory/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    dup
    iload 2
    i2l
    lconst_1
    lsub
    getstatic Inventory/bccRecLen [I
    ldc 1
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic Inventory/bccBufs [[B
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/write ([B)V
    return
.end method

.method public static subtractStock : ()V
    .limit stack 16
    .limit locals 18

    iconst_0
    istore 0
    iconst_0
    istore 1
    ldc ""
    astore 2
    ldc ""
    astore 3
    iconst_0
    istore 4
    ldc ""
    astore 5
    iconst_0
    istore 6
    dconst_0
    dstore 7
    iconst_0
    istore 9
    iconst_0
    istore 10
    iconst_0
    istore 11
    ldc ""
    astore 12
    iconst_0
    istore 13
    ; global inv
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "[2J[H"
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 5
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc ";"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 20
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "H"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "S U B T R A C T I N G    S T O C K"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V

L_do_0_top:
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 8
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc ";"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 25
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "H"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    invokestatic Inventory/readPartNumberInput ()Ljava/lang/String;
    astore 2
    aload 2
    invokestatic java/lang/Double/parseDouble (Ljava/lang/String;)D
    dup2
    ldc2_w 0.5
    dup2_x2
    pop2
    invokestatic java/lang/Math/copySign (DD)D
    dadd
    d2l
    l2i
    istore 1
    iload 1
    invokestatic Inventory/partInRange (I)I
    istore 13
    iload 13
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
    ifeq L_if_1_else
    invokestatic Inventory/showRangeRetryMessage ()V
    invokestatic Inventory/readKey ()Ljava/lang/String;
    pop
L_if_1_else:
    iload 13
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    dup
    ineg
    ior
    bipush 31
    iushr
    ineg
    ifne L_do_0_end
    goto L_do_0_top
L_do_0_end:

    ; let p = inv[...]  (whole-record read)
    getstatic Inventory/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    dup
    iload 1
    i2l
    lconst_1
    lsub
    getstatic Inventory/bccRecLen [I
    ldc 1
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic Inventory/bccBufs [[B
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/read ([B)I
    pop
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 0
    ldc 1
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    invokevirtual java/lang/String/length ()I
    istore 6
L_while_2_top:
    iload 6
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_while_2_end
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 0
    ldc 1
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    iload 6
    iconst_1
    isub
    dup
    ldc 1
    iadd
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    ldc " "
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ineg
    ifeq L_while_2_end
    iload 6
    ldc 1
    isub
    istore 6
    goto L_while_2_top
L_while_2_end:
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 0
    ldc 1
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    iconst_0
    iload 6
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    astore 5
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 1
    ldc 30
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    invokevirtual java/lang/String/length ()I
    istore 4
L_while_3_top:
    iload 4
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_while_3_end
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 1
    ldc 30
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    iload 4
    iconst_1
    isub
    dup
    ldc 1
    iadd
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    ldc " "
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ineg
    ifeq L_while_3_end
    iload 4
    ldc 1
    isub
    istore 4
    goto L_while_3_top
L_while_3_end:
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 1
    ldc 30
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    iconst_0
    iload 4
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    astore 3
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 31
    ldc 2
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    invokestatic java/nio/ByteBuffer/wrap ([B)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/getShort ()S
    istore 9
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 33
    ldc 2
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    invokestatic java/nio/ByteBuffer/wrap ([B)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/getShort ()S
    istore 10
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 35
    ldc 4
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    invokestatic java/nio/ByteBuffer/wrap ([B)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/getFloat ()F
    f2d
    dstore 7
    aload 5
    invokestatic Inventory/isEmpty (Ljava/lang/String;)I
    ifeq L_if_4_else
    aload 2
    invokestatic Inventory/showNullEntryMessage (Ljava/lang/String;)V
    invokestatic Inventory/readKey ()Ljava/lang/String;
    pop
    return
L_if_4_else:

L_do_5_top:
    iload 1
    aload 3
    iload 9
    iload 10
    invokestatic Inventory/showSubtractStockScreen (ILjava/lang/String;II)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 14
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc ";"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 20
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "H"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Quantity to subtract? "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    invokevirtual java/io/PrintStream/flush ()V
    new java/lang/ProcessBuilder
    dup
    ldc 2
    anewarray java/lang/String
    dup
    ldc 0
    ldc "stty"
    aastore
    dup
    ldc 1
    ldc "sane"
    aastore
    invokespecial java/lang/ProcessBuilder/<init> ([Ljava/lang/String;)V
    invokevirtual java/lang/ProcessBuilder/inheritIO ()Ljava/lang/ProcessBuilder;
    invokevirtual java/lang/ProcessBuilder/start ()Ljava/lang/Process;
    invokevirtual java/lang/Process/waitFor ()I
    pop
    getstatic Inventory/bccStdin Ljava/io/BufferedReader;
    invokevirtual java/io/BufferedReader/readLine ()Ljava/lang/String;
    new java/lang/ProcessBuilder
    dup
    ldc 7
    anewarray java/lang/String
    dup
    ldc 0
    ldc "stty"
    aastore
    dup
    ldc 1
    ldc "-icanon"
    aastore
    dup
    ldc 2
    ldc "-echo"
    aastore
    dup
    ldc 3
    ldc "min"
    aastore
    dup
    ldc 4
    ldc "0"
    aastore
    dup
    ldc 5
    ldc "time"
    aastore
    dup
    ldc 6
    ldc "0"
    aastore
    invokespecial java/lang/ProcessBuilder/<init> ([Ljava/lang/String;)V
    invokevirtual java/lang/ProcessBuilder/inheritIO ()Ljava/lang/ProcessBuilder;
    invokevirtual java/lang/ProcessBuilder/start ()Ljava/lang/Process;
    invokevirtual java/lang/Process/waitFor ()I
    pop
    astore 12
    aload 12
    invokestatic java/lang/Double/parseDouble (Ljava/lang/String;)D
    dup2
    ldc2_w 0.5
    dup2_x2
    pop2
    invokestatic java/lang/Math/copySign (DD)D
    dadd
    d2l
    l2i
    istore 11
    ldc 0
    istore 0
    iload 11
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    iconst_1
    isub
    bipush 31
    ishr
    ifeq L_if_6_else
    iload 9
    iload 11
    isub
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    bipush 31
    ishr
    ifeq L_if_6_else
    ldc 1
    istore 0
    iload 9
    invokestatic Inventory/showOverSubtractWarning (I)V
    invokestatic Inventory/readKey ()Ljava/lang/String;
    pop
L_if_6_else:
    iload 11
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    iconst_1
    isub
    bipush 31
    ishr
    ifeq L_condition_0
    iload 0
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
    ifne L_do_5_end
L_condition_0:
    goto L_do_5_top
L_do_5_end:

    iload 9
    iload 11
    isub
    istore 9
    iload 9
    iload 10
    invokestatic java/lang/Integer/compare (II)I
    iconst_1
    isub
    bipush 31
    ishr
    ifeq L_if_7_else
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 16
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc ";"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 20
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "H"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
L_if_7_else:
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
    ldc "quantity now"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    iload 9
    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc " reorder level"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    iload 10
    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ; inv[...] = p  (write back a let-bound record)
    aload 5
    ldc 1
    newarray char
    dup
    bipush 32
    invokestatic java/util/Arrays/fill ([CC)V
    new java/lang/String
    dup_x1
    swap
    invokespecial java/lang/String/<init> ([C)V
    invokevirtual java/lang/String/concat (Ljava/lang/String;)Ljava/lang/String;
    iconst_0
    ldc 1
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    iconst_0
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    ldc 0
    ldc 1
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    aload 3
    ldc 30
    newarray char
    dup
    bipush 32
    invokestatic java/util/Arrays/fill ([CC)V
    new java/lang/String
    dup_x1
    swap
    invokespecial java/lang/String/<init> ([C)V
    invokevirtual java/lang/String/concat (Ljava/lang/String;)Ljava/lang/String;
    iconst_0
    ldc 30
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    iconst_0
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    ldc 1
    ldc 30
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    ldc 2
    invokestatic java/nio/ByteBuffer/allocate (I)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    iload 9
    i2s
    invokevirtual java/nio/ByteBuffer/putShort (S)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/array ()[B
    new java/lang/String
    dup_x1
    swap
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BLjava/nio/charset/Charset;)V
    ldc 2
    newarray char
    dup
    bipush 32
    invokestatic java/util/Arrays/fill ([CC)V
    new java/lang/String
    dup_x1
    swap
    invokespecial java/lang/String/<init> ([C)V
    invokevirtual java/lang/String/concat (Ljava/lang/String;)Ljava/lang/String;
    iconst_0
    ldc 2
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    iconst_0
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    ldc 31
    ldc 2
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    ldc 2
    invokestatic java/nio/ByteBuffer/allocate (I)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    iload 10
    i2s
    invokevirtual java/nio/ByteBuffer/putShort (S)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/array ()[B
    new java/lang/String
    dup_x1
    swap
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BLjava/nio/charset/Charset;)V
    ldc 2
    newarray char
    dup
    bipush 32
    invokestatic java/util/Arrays/fill ([CC)V
    new java/lang/String
    dup_x1
    swap
    invokespecial java/lang/String/<init> ([C)V
    invokevirtual java/lang/String/concat (Ljava/lang/String;)Ljava/lang/String;
    iconst_0
    ldc 2
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    iconst_0
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    ldc 33
    ldc 2
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    ldc 4
    invokestatic java/nio/ByteBuffer/allocate (I)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    dload 7
    d2f
    invokevirtual java/nio/ByteBuffer/putFloat (F)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/array ()[B
    new java/lang/String
    dup_x1
    swap
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BLjava/nio/charset/Charset;)V
    ldc 4
    newarray char
    dup
    bipush 32
    invokestatic java/util/Arrays/fill ([CC)V
    new java/lang/String
    dup_x1
    swap
    invokespecial java/lang/String/<init> ([C)V
    invokevirtual java/lang/String/concat (Ljava/lang/String;)Ljava/lang/String;
    iconst_0
    ldc 4
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    iconst_0
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    ldc 35
    ldc 4
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    getstatic Inventory/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    dup
    iload 1
    i2l
    lconst_1
    lsub
    getstatic Inventory/bccRecLen [I
    ldc 1
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic Inventory/bccBufs [[B
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/write ([B)V
    return
.end method

.method public static reorderReport : ()V
    .limit stack 16
    .limit locals 14

    iconst_0
    istore 0
    ldc ""
    astore 1
    iconst_0
    istore 2
    ldc ""
    astore 3
    iconst_0
    istore 4
    dconst_0
    dstore 5
    iconst_0
    istore 7
    iconst_0
    istore 8
    iconst_0
    istore 9
    ; global inv
    invokestatic Inventory/printReorderHeader ()V
    ldc 0
    istore 9
    ldc 1
    istore 0
L_for_0_top:
    iload 0
    ldc 100
    if_icmpgt L_for_0_end
    ; let p = inv[...]  (whole-record read)
    getstatic Inventory/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    dup
    iload 0
    i2l
    lconst_1
    lsub
    getstatic Inventory/bccRecLen [I
    ldc 1
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic Inventory/bccBufs [[B
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/read ([B)I
    pop
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 0
    ldc 1
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    invokevirtual java/lang/String/length ()I
    istore 4
L_while_1_top:
    iload 4
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_while_1_end
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 0
    ldc 1
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    iload 4
    iconst_1
    isub
    dup
    ldc 1
    iadd
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    ldc " "
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ineg
    ifeq L_while_1_end
    iload 4
    ldc 1
    isub
    istore 4
    goto L_while_1_top
L_while_1_end:
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 0
    ldc 1
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    iconst_0
    iload 4
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    astore 3
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 1
    ldc 30
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    invokevirtual java/lang/String/length ()I
    istore 2
L_while_2_top:
    iload 2
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_while_2_end
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 1
    ldc 30
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    iload 2
    iconst_1
    isub
    dup
    ldc 1
    iadd
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    ldc " "
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ineg
    ifeq L_while_2_end
    iload 2
    ldc 1
    isub
    istore 2
    goto L_while_2_top
L_while_2_end:
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 1
    ldc 30
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    iconst_0
    iload 2
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    astore 1
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 31
    ldc 2
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    invokestatic java/nio/ByteBuffer/wrap ([B)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/getShort ()S
    istore 7
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 33
    ldc 2
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    invokestatic java/nio/ByteBuffer/wrap ([B)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/getShort ()S
    istore 8
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 35
    ldc 4
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    invokestatic java/nio/ByteBuffer/wrap ([B)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/getFloat ()F
    f2d
    dstore 5
    iload 7
    iload 8
    invokestatic java/lang/Integer/compare (II)I
    bipush 31
    ishr
    ifeq L_if_3_else
    iload 0
    aload 1
    iload 7
    iload 8
    invokestatic Inventory/printReorderLine (ILjava/lang/String;II)V
    iload 9
    ldc 1
    iadd
    istore 9
    iload 9
    ldc 15
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_if_4_else
    invokestatic Inventory/waitAnyKey ()V
    ldc 0
    istore 9
    ; Redraw for the next page rather than let it keep
    ; scrolling past row 25 -- see printListHeader()'s own
    ; note (same underlying issue, same fix) on why a
    ; fixed-row prompt can't coexist with unbounded scrolling.
    iload 0
    ldc 100
    invokestatic java/lang/Integer/compare (II)I
    bipush 31
    ishr
    ifeq L_if_5_else
    invokestatic Inventory/printReorderHeader ()V
L_if_5_else:
L_if_4_else:
L_if_3_else:
    iload 0
    ldc 1
    iadd
    istore 0
    goto L_for_0_top
L_for_0_end:
    invokestatic Inventory/waitAnyKey ()V
    return
.end method

.method public static initializeInventoryFileIfNew : ()V
    .limit stack 16
    .limit locals 13

    iconst_0
    istore 0
    ldc ""
    astore 1
    iconst_0
    istore 2
    ldc ""
    astore 3
    iconst_0
    istore 4
    dconst_0
    dstore 5
    iconst_0
    istore 7
    iconst_0
    istore 8
    ; global inv
    ; let p = inv[...]  (whole-record read)
    getstatic Inventory/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    dup
    ldc 1
    i2l
    lconst_1
    lsub
    getstatic Inventory/bccRecLen [I
    ldc 1
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic Inventory/bccBufs [[B
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/read ([B)I
    pop
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 0
    ldc 1
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    invokevirtual java/lang/String/length ()I
    istore 4
L_while_0_top:
    iload 4
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_while_0_end
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 0
    ldc 1
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    iload 4
    iconst_1
    isub
    dup
    ldc 1
    iadd
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    ldc " "
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ineg
    ifeq L_while_0_end
    iload 4
    ldc 1
    isub
    istore 4
    goto L_while_0_top
L_while_0_end:
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 0
    ldc 1
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    iconst_0
    iload 4
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    astore 3
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 1
    ldc 30
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    invokevirtual java/lang/String/length ()I
    istore 2
L_while_1_top:
    iload 2
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_while_1_end
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 1
    ldc 30
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    iload 2
    iconst_1
    isub
    dup
    ldc 1
    iadd
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    ldc " "
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ineg
    ifeq L_while_1_end
    iload 2
    ldc 1
    isub
    istore 2
    goto L_while_1_top
L_while_1_end:
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 1
    ldc 30
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    iconst_0
    iload 2
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    astore 1
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 31
    ldc 2
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    invokestatic java/nio/ByteBuffer/wrap ([B)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/getShort ()S
    istore 7
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 33
    ldc 2
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    invokestatic java/nio/ByteBuffer/wrap ([B)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/getShort ()S
    istore 8
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 35
    ldc 4
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    invokestatic java/nio/ByteBuffer/wrap ([B)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/getFloat ()F
    f2d
    dstore 5
    aload 3
    iconst_0
    invokevirtual java/lang/String/charAt (I)C
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
    ifeq L_if_2_else
    ldc 1
    istore 0
L_for_3_top:
    iload 0
    ldc 100
    if_icmpgt L_for_3_end
    ; inv[...] = { ... }  (whole-record write)
    ldc 255
    i2c
    invokestatic java/lang/String/valueOf (C)Ljava/lang/String;
    ldc 1
    newarray char
    dup
    bipush 32
    invokestatic java/util/Arrays/fill ([CC)V
    new java/lang/String
    dup_x1
    swap
    invokespecial java/lang/String/<init> ([C)V
    invokevirtual java/lang/String/concat (Ljava/lang/String;)Ljava/lang/String;
    iconst_0
    ldc 1
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    iconst_0
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    ldc 0
    ldc 1
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    ldc ""
    ldc 30
    newarray char
    dup
    bipush 32
    invokestatic java/util/Arrays/fill ([CC)V
    new java/lang/String
    dup_x1
    swap
    invokespecial java/lang/String/<init> ([C)V
    invokevirtual java/lang/String/concat (Ljava/lang/String;)Ljava/lang/String;
    iconst_0
    ldc 30
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    iconst_0
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    ldc 1
    ldc 30
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    ldc 2
    invokestatic java/nio/ByteBuffer/allocate (I)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    ldc 0
    i2s
    invokevirtual java/nio/ByteBuffer/putShort (S)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/array ()[B
    new java/lang/String
    dup_x1
    swap
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BLjava/nio/charset/Charset;)V
    ldc 2
    newarray char
    dup
    bipush 32
    invokestatic java/util/Arrays/fill ([CC)V
    new java/lang/String
    dup_x1
    swap
    invokespecial java/lang/String/<init> ([C)V
    invokevirtual java/lang/String/concat (Ljava/lang/String;)Ljava/lang/String;
    iconst_0
    ldc 2
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    iconst_0
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    ldc 31
    ldc 2
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    ldc 2
    invokestatic java/nio/ByteBuffer/allocate (I)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    ldc 0
    i2s
    invokevirtual java/nio/ByteBuffer/putShort (S)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/array ()[B
    new java/lang/String
    dup_x1
    swap
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BLjava/nio/charset/Charset;)V
    ldc 2
    newarray char
    dup
    bipush 32
    invokestatic java/util/Arrays/fill ([CC)V
    new java/lang/String
    dup_x1
    swap
    invokespecial java/lang/String/<init> ([C)V
    invokevirtual java/lang/String/concat (Ljava/lang/String;)Ljava/lang/String;
    iconst_0
    ldc 2
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    iconst_0
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    ldc 33
    ldc 2
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    ldc 4
    invokestatic java/nio/ByteBuffer/allocate (I)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    ldc 0
    i2d
    d2f
    invokevirtual java/nio/ByteBuffer/putFloat (F)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/array ()[B
    new java/lang/String
    dup_x1
    swap
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BLjava/nio/charset/Charset;)V
    ldc 4
    newarray char
    dup
    bipush 32
    invokestatic java/util/Arrays/fill ([CC)V
    new java/lang/String
    dup_x1
    swap
    invokespecial java/lang/String/<init> ([C)V
    invokevirtual java/lang/String/concat (Ljava/lang/String;)Ljava/lang/String;
    iconst_0
    ldc 4
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    iconst_0
    getstatic Inventory/bccBufs [[B
    ldc 0
    aaload
    ldc 35
    ldc 4
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    getstatic Inventory/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    dup
    iload 0
    i2l
    lconst_1
    lsub
    getstatic Inventory/bccRecLen [I
    ldc 1
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic Inventory/bccBufs [[B
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/write ([B)V
    iload 0
    ldc 1
    iadd
    istore 0
    goto L_for_3_top
L_for_3_end:
L_if_2_else:
    return
.end method

.method public static reportInventoryError : (II)V
    .limit stack 16
    .limit locals 7

    ldc ""
    astore 2
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 25
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc ";"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc 1
    invokevirtual java/lang/StringBuilder/append (I)Ljava/lang/StringBuilder;
    ldc "H"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
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
    ldc "There has been an error on line"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    iload 1
    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc ": "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    iload 0
    invokestatic Inventory/error (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    invokestatic Inventory/readKey ()Ljava/lang/String;
    astore 2
    return
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
    .limit locals 8

    iconst_0
    putstatic Inventory/g1 I
    iconst_0
    putstatic Inventory/g2 I
    ldc ""
    putstatic Inventory/g3 Ljava/lang/String;
    ldc 16
    anewarray java/io/RandomAccessFile
    putstatic Inventory/bccFiles [Ljava/io/RandomAccessFile;
    ldc 16
    anewarray [B
    putstatic Inventory/bccBufs [[B
    ldc 16
    newarray int
    putstatic Inventory/bccRecLen [I
    new java/io/BufferedReader
    dup
    new java/io/InputStreamReader
    dup
    getstatic java/lang/System/in Ljava/io/InputStream;
    invokespecial java/io/InputStreamReader/<init> (Ljava/io/InputStream;)V
    invokespecial java/io/BufferedReader/<init> (Ljava/io/Reader;)V
    putstatic Inventory/bccStdin Ljava/io/BufferedReader;
    new java/lang/ProcessBuilder
    dup
    ldc 7
    anewarray java/lang/String
    dup
    ldc 0
    ldc "stty"
    aastore
    dup
    ldc 1
    ldc "-icanon"
    aastore
    dup
    ldc 2
    ldc "-echo"
    aastore
    dup
    ldc 3
    ldc "min"
    aastore
    dup
    ldc 4
    ldc "0"
    aastore
    dup
    ldc 5
    ldc "time"
    aastore
    dup
    ldc 6
    ldc "0"
    aastore
    invokespecial java/lang/ProcessBuilder/<init> ([Ljava/lang/String;)V
    invokevirtual java/lang/ProcessBuilder/inheritIO ()Ljava/lang/ProcessBuilder;
    invokevirtual java/lang/ProcessBuilder/start ()Ljava/lang/Process;
    invokevirtual java/lang/Process/waitFor ()I
    pop
    ; Maps an ERR code to its classic MBASIC/GW-BASIC/BASCOM message. Compiles
    ; and links on a real IBM BASIC Compiler 2.00 as ERROR$, but silently
    ; returns an empty string at runtime (verified under dosbox-x) -- so BASCAL
    ; ships a working implementation.
    ; 
    ; The named constants below are the complete common subset supported by
    ; ERROR$: use them in THROW and filtered CATCH clauses instead of magic
    ; numbers.  Dialect-specific errors outside this shared MBASIC/GW-BASIC/
    ; BASCOM subset still fall through to ERROR$'s generic message.
    ; 
    ; Deliberately NOT a scalar method (see GitHub issue #41, which asked for
    ; this decision to be recorded either way): code% is an opaque lookup key,
    ; not a value the call is naturally "operating on" the way ltrim$/rtrim$/
    ; ucase$/lcase$ operate on their string -- code%.error() would read as if
    ; the *error code itself* has a message, when really this is a lookup
    ; table keyed by that code. Stays an ordinary function.


    ; ============================================================
    ; INVENTORY.BCL -- Random-Access Inventory Program
    ; 
    ; A BASCAL reconstruction of "Example program for RANDOM ACCESS
    ; FILE study", by fhb, 8/19/98, from Joseph Sixpack's GW-BASIC
    ; programs page (part of his "Last Book of GW-Basic" collection):
    ; http://www.geocities.ws/joseph_sixpack/binventory.html
    ; fhb's own header comment credits the original as "suggested
    ; from MS-BASIC manual".
    ; 
    ; This is a reconstruction, not a line-by-line port -- some
    ; original pieces have no BASCAL equivalent and were dropped
    ; rather than approximated:
    ; - The GOTO-driven "subroutine roadmap" dispatcher at the top
    ; of fhb's listing (a `LIST 110-320` etc. navigation aid for
    ; editing in the GW-BASIC interpreter) has no meaning once the
    ; program is structured into named function/procedure blocks.
    ; - `KEY OFF` / `KEY I,""` (clearing the function-key soft-label
    ; row) and `VIEW PRINT` (scroll-region windowing for the list
    ; screen) are interpreter/console features BASCAL doesn't
    ; expose.
    ; - fhb's own hand-rolled numeric-ERR-code-to-message lookup table
    ; (ERR=1 "Input value overflow", ERR=2 "Syntax error", ... ERR=25)
    ; is replaced below by BASCAL's com.bascal.stdlib.error library
    ; (ERROR$(code%)) -- same idea, BASCAL's own table; it still
    ; doesn't decode ERL, which errorTrap() reports as the raw line
    ; number.
    ; - fhb's one-time "hidden" datafile initializer (PUT-ing 100
    ; blank, CHR$(255)-flagged records) is reproduced below as
    ; initializeInventoryFileIfNew(), called once at program entry --
    ; inven.dat no longer has to be pre-populated by hand.
    ; - The three original tab-position constants (T=20, U=25,
    ; V=30) are collapsed into a single `TAB_COL = 20`; a couple of
    ; screens that used U=25 in the original (see showAddStockScreen
    ; below) keep 25 as a literal rather than reusing TAB_COL.
    ; 
    ; Tracks parts in a fixed 100-record file: check status, add,
    ; edit, add/subtract stock, and a reorder report.
    ; 
    ; Error handling uses try/catch (GitHub issue #60), not the raw `on
    ; error goto` / `resume next` fhb's original relies on: a failed menu
    ; action is abandoned outright and the program returns straight to the
    ; main menu, rather than resuming at the exact instruction after
    ; whatever failed -- see reportInventoryError() below and
    ; tutorial/inventory_try_catch.draft's own header comment for why. This
    ; is a real, deliberate behavior change from an earlier on-error-goto
    ; version of this file, which *was* verified against real BASCOM 2.00
    ; under dosbox-x (only with the /E and /X switches -- error trapping
    ; isn't linked in by default); the try/catch shape below transpiles to
    ; the same ON ERROR GOTO/RESUME primitives BASCOM accepts, but hasn't
    ; itself been independently re-verified against a real BASCOM compile.
    ; ============================================================


    ; BASCAL-ism: the record/file DSL. `record ... end record` plus
    ; `file ... as ... = open(...)` below replace fhb's manual
    ; FIELD #1,1 AS F$,30 AS D$,2 AS Q$,... buffer layout entirely --
    ; bcc computes the field widths and record LEN from this
    ; declaration and generates the FIELD statement itself. Named
    ; field access (`p.flag`, `p.qty`, ...) and whole-record
    ; read/write via `inv[n]` (see checkPart() below) replace fhb's
    ; manual GET/PUT plus LSET/RSET and MKI$/MKS$/CVI$/CVS$ packing.

    ; BASCAL-ism: `const` is a real compile-time constant, not a plain
    ; variable assignment like fhb's `N=100` / `T=20` -- it can never
    ; be reassigned, and resolves to the same value everywhere,
    ; including inside every function/procedure below, with no
    ; `global` declaration needed.

    ; `file ... = open(...)` is sugar for OPEN ... FOR RANDOM AS #n
    ; LEN = <record width> plus the FIELD statement fhb wrote out by
    ; hand at his line 550. Wrapped in its own try/catch: a file that
    ; exists but can't be opened for random access (permissions, a
    ; read-only inven.dat, disk full on the fallback create) is a real,
    ; trappable error (code 75, "Path/File access error") on both
    ; targets now, not a hard crash -- report it and exit cleanly
    ; instead of leaving the program to fail confusingly the first time
    ; something tries to use an `inv` that was never actually opened.
L_try_0_start:
    ; file inv as Part = open(...)  [39 bytes/record]
    getstatic Inventory/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    new java/io/RandomAccessFile
    dup
    ldc "inven.dat"
    ldc "rw"
    invokespecial java/io/RandomAccessFile/<init> (Ljava/lang/String;Ljava/lang/String;)V
    aastore
    getstatic Inventory/bccRecLen [I
    ldc 1
    iconst_1
    isub
    ldc 39
    iastore
    getstatic Inventory/bccBufs [[B
    ldc 1
    iconst_1
    isub
    ldc 39
    newarray byte
    aastore
    goto L_try_0_finish
L_try_0_end:
L_try_0_catch:
    invokevirtual java/lang/Throwable/getMessage ()Ljava/lang/String;
    invokestatic java/lang/Integer/parseInt (Ljava/lang/String;)I
    putstatic Inventory/g2 I
    iconst_0
    putstatic Inventory/g1 I
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "could not open inven.dat: "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    getstatic Inventory/g2 I
    invokestatic Inventory/error (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    new java/lang/ProcessBuilder
    dup
    ldc 2
    anewarray java/lang/String
    dup
    ldc 0
    ldc "stty"
    aastore
    dup
    ldc 1
    ldc "sane"
    aastore
    invokespecial java/lang/ProcessBuilder/<init> ([Ljava/lang/String;)V
    invokevirtual java/lang/ProcessBuilder/inheritIO ()Ljava/lang/ProcessBuilder;
    invokevirtual java/lang/ProcessBuilder/start ()Ljava/lang/Process;
    invokevirtual java/lang/Process/waitFor ()I
    pop
    return
    goto L_try_0_finish
L_try_0_finish:

    ; -------------------- Pure functions (no file access) --------------------

    ; BASCAL-ism: `function ... end function` with `return` replaces
    ; fhb's convention of a GOSUB target plus a bare RETURN -- there's
    ; no separate "subroutine label" and no shared/global result
    ; variable to manage by hand; `isEmpty%(...)` is called like an
    ; ordinary expression at every use below (e.g. `isEmpty%(p.flag)`).
    ; A record whose flag byte is CHR$(255) is an empty/never-used slot.

    ; BASCAL-ism: `&&` and `||` are short-circuit AND/OR -- real
    ; MBASIC/BASCOM only has bitwise AND/OR (which fhb relies on here
    ; too, since `PART!<1 OR PART!>N!` never short-circuits anyway).
    ; BASCAL lowers `&&`/`||` into the equivalent branching so the
    ; short-circuit *is* real at the generated-BASIC level; see the
    ; manual's "Short-Circuit && and ||" section
    ; (https://johnjoeallen.github.io/bascal/manual/).


    ; -------------------- Keyboard input --------------------

    ; BASCAL-ism: `do ... loop until` is a structured post-check loop
    ; replacing fhb's `730 KP$=INKEY$:IF KP$="" THEN 730` GOTO-polling
    ; idiom. `inkey$` itself is the real INKEY$ builtin passed straight
    ; through, resolving correctly from inside a function/procedure
    ; body like this one -- every menu action below calls
    ; readKey$()/waitAnyKey() rather than polling INKEY$ inline.


    ; -------------------- Display procedures --------------------






    ; BASCAL-ism: no `VIEW PRINT` (see the header note above), so this
    ; deliberately does NOT pin a "press any key" line to a fixed row the way
    ; fhb's original does -- a bare `LOCATE 25, ...` sitting under content
    ; that keeps printing past it (listAll()'s own items) collides with
    ; whatever's later written there, since nothing here scrolls a bounded
    ; region: waitAnyKey() is the only thing that ever touches row 25, and
    ; only right when it actually blocks (see listAll()'s own redraw-per-page
    ; structure below).




    ; byref scalar parameters: gatherPartDetails writes the four editable
    ; fields for a part directly back into the caller's variables.





    ; -------------------- Menu actions --------------------







    ; fhb's own one-time "hidden" datafile initializer PUT-ing 100 blank,
    ; CHR$(255)-flagged records (see the header note above) -- reproduced
    ; here so inven.dat no longer has to be pre-populated by hand before
    ; running this program. A brand-new file OPEN created just now (rather
    ; than one that already existed) reads back as all-zero bytes: record
    ; 1's flag byte is CHR$(0), never CHR$(255) -- the one signal an
    ; already-populated file (whose record 1 flag is always either
    ; CHR$(255), still an empty slot, or a real part's own "1") could never
    ; produce, so it's what isEmpty%() itself can't use (see its own
    ; header note) but this one-time check safely can.

    ; -------------------- Program entry --------------------

    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "[2J[H"
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    invokestatic Inventory/initializeInventoryFileIfNew ()V

L_do_1_top:
    invokestatic Inventory/showMainMenu ()V
    invokestatic Inventory/readKey ()Ljava/lang/String;
    putstatic Inventory/g3 Ljava/lang/String;
    ldc "1234567cCeElLaAsSrRxX"
    getstatic Inventory/g3 Ljava/lang/String;
    invokevirtual java/lang/String/indexOf (Ljava/lang/String;)I
    iconst_1
    iadd
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    dup
    ineg
    ior
    bipush 31
    iushr
    ineg
    ifeq L_if_2_else
    ; BASCAL-ism: `select case` replaces fhb's chain of eight
    ; `IF VAL(KP$)=n OR KP$="x" OR KP$="X" THEN GOTO ...` lines
    ; (his 770-840) with one multi-way dispatch.
    ; 
    ; BASCAL-ism: `try`/`catch` (issue #60) replaces fhb's own global
    ; `ON ERROR GOTO` trap. A failed menu action is abandoned outright
    ; here -- the `catch` below runs, then execution continues right
    ; after `end try`, back at `loop until` -- rather than resuming at
    ; the exact instruction after whatever failed inside checkPart()/
    ; editRecord()/etc. the way fhb's `RESUME NEXT` did. See
    ; reportInventoryError() below and tutorial/inventory_try_catch.
    ; draft's own header comment for why that arbitrary resume-point
    ; behavior isn't something try/catch reproduces.
L_try_3_start:
    getstatic Inventory/g3 Ljava/lang/String;
    dup
    ldc "1"
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ifne L_select_4_case_0
    dup
    ldc "c"
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ifne L_select_4_case_0
    dup
    ldc "C"
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ifne L_select_4_case_0
    goto L_select_4_next_0
L_select_4_next_0:
    dup
    ldc "2"
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ifne L_select_4_case_1
    dup
    ldc "e"
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ifne L_select_4_case_1
    dup
    ldc "E"
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ifne L_select_4_case_1
    goto L_select_4_next_1
L_select_4_next_1:
    dup
    ldc "3"
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ifne L_select_4_case_2
    dup
    ldc "l"
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ifne L_select_4_case_2
    dup
    ldc "L"
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ifne L_select_4_case_2
    goto L_select_4_next_2
L_select_4_next_2:
    dup
    ldc "4"
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ifne L_select_4_case_3
    dup
    ldc "a"
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ifne L_select_4_case_3
    dup
    ldc "A"
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ifne L_select_4_case_3
    goto L_select_4_next_3
L_select_4_next_3:
    dup
    ldc "5"
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ifne L_select_4_case_4
    dup
    ldc "s"
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ifne L_select_4_case_4
    dup
    ldc "S"
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ifne L_select_4_case_4
    goto L_select_4_next_4
L_select_4_next_4:
    dup
    ldc "6"
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ifne L_select_4_case_5
    dup
    ldc "r"
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ifne L_select_4_case_5
    dup
    ldc "R"
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ifne L_select_4_case_5
    goto L_select_4_next_5
L_select_4_next_5:
    dup
    ldc "7"
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ifne L_select_4_case_6
    dup
    ldc "x"
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ifne L_select_4_case_6
    dup
    ldc "X"
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ifne L_select_4_case_6
    goto L_select_4_next_6
L_select_4_next_6:
    pop
    goto L_select_4_end
L_select_4_case_0:
    pop
    invokestatic Inventory/checkPart ()V
    goto L_select_4_end
L_select_4_case_1:
    pop
    invokestatic Inventory/editRecord ()V
    goto L_select_4_end
L_select_4_case_2:
    pop
    invokestatic Inventory/listAll ()V
    goto L_select_4_end
L_select_4_case_3:
    pop
    invokestatic Inventory/addStock ()V
    goto L_select_4_end
L_select_4_case_4:
    pop
    invokestatic Inventory/subtractStock ()V
    goto L_select_4_end
L_select_4_case_5:
    pop
    invokestatic Inventory/reorderReport ()V
    goto L_select_4_end
L_select_4_case_6:
    pop
    ; BASCAL-ism: `inv.close()` is sugar for `CLOSE #1`,
    ; matching fhb's own `90 CLOSE:SYSTEM`. fhb's original
    ; also had a separate "Quit to BASIC" option (his own
    ; 7, returning to the interpreter's command prompt
    ; rather than exiting to DOS) -- dropped here: a
    ; compiled program has no interpreter to return to,
    ; so it was never anything but a second spelling of
    ; this same close-and-exit action.
    ; inv.close()
    getstatic Inventory/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/close ()V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "[37;40m"
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "[2J[H"
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    new java/lang/ProcessBuilder
    dup
    ldc 2
    anewarray java/lang/String
    dup
    ldc 0
    ldc "stty"
    aastore
    dup
    ldc 1
    ldc "sane"
    aastore
    invokespecial java/lang/ProcessBuilder/<init> ([Ljava/lang/String;)V
    invokevirtual java/lang/ProcessBuilder/inheritIO ()Ljava/lang/ProcessBuilder;
    invokevirtual java/lang/ProcessBuilder/start ()Ljava/lang/Process;
    invokevirtual java/lang/Process/waitFor ()I
    pop
    iconst_0
    invokestatic java/lang/System/exit (I)V
    goto L_select_4_end
L_select_4_end:
    goto L_try_3_finish
L_try_3_end:
L_try_3_catch:
    invokevirtual java/lang/Throwable/getMessage ()Ljava/lang/String;
    invokestatic java/lang/Integer/parseInt (Ljava/lang/String;)I
    putstatic Inventory/g2 I
    iconst_0
    putstatic Inventory/g1 I
    getstatic Inventory/g2 I
    getstatic Inventory/g1 I
    invokestatic Inventory/reportInventoryError (II)V
    goto L_try_3_finish
L_try_3_finish:
L_if_2_else:
    goto L_do_1_top
L_do_1_end:

    ; -------------------- Error handling --------------------
    ; err%/erl% are ordinary locals scoped to the `catch` block above, not
    ; aliases for the ambient (readable-anywhere) `err`/`erl` pseudo-
    ; variables `on error goto` uses -- see `Statement::TryCatch`'s own doc
    ; comment in ast.rs. Passed straight through to ERROR$ here like fhb's
    ; own ERR/ERL (his 3390: "an error on line";ERL), decoded through
    ; BASCAL's own com.bascal.stdlib.error (ERROR$) instead of fhb's
    ; hand-rolled lookup table -- see the header note above. try/catch
    ; itself isn't documented in the manual yet (GitHub issue #60 tracks
    ; the still-unfinished C-target work; the manual page can follow once
    ; that lands) -- see ast.rs's own `Statement::TryCatch` doc comment for
    ; the full semantics meanwhile.
    new java/lang/ProcessBuilder
    dup
    ldc 2
    anewarray java/lang/String
    dup
    ldc 0
    ldc "stty"
    aastore
    dup
    ldc 1
    ldc "sane"
    aastore
    invokespecial java/lang/ProcessBuilder/<init> ([Ljava/lang/String;)V
    invokevirtual java/lang/ProcessBuilder/inheritIO ()Ljava/lang/ProcessBuilder;
    invokevirtual java/lang/ProcessBuilder/start ()Ljava/lang/Process;
    invokevirtual java/lang/Process/waitFor ()I
    pop
    return
    .catch java/lang/RuntimeException from L_try_0_start to L_try_0_end using L_try_0_catch
    .catch java/lang/RuntimeException from L_try_3_start to L_try_3_end using L_try_3_catch
.end method
