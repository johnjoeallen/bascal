.version 50 0
.class public Stdlib
.super java/lang/Object

.method public static ltrim : (Ljava/lang/String;)Ljava/lang/String;
    .limit stack 16
    .limit locals 2

    iconst_0
    istore 1
    ldc 1
    istore 1
L_while_0_top:
    iload 1
    aload 0
    invokevirtual java/lang/String/length ()I
    invokestatic java/lang/Integer/compare (II)I
    iconst_1
    isub
    bipush 31
    ishr
    ifeq L_while_0_end
    aload 0
    iload 1
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
    iload 1
    ldc 1
    iadd
    istore 1
    goto L_while_0_top
L_while_0_end:
    aload 0
    iload 1
    iconst_1
    isub
    invokevirtual java/lang/String/substring (I)Ljava/lang/String;
    areturn
    ldc ""
    areturn
.end method

.method public static rtrim : (Ljava/lang/String;)Ljava/lang/String;
    .limit stack 16
    .limit locals 2

    iconst_0
    istore 1
    aload 0
    invokevirtual java/lang/String/length ()I
    istore 1
L_while_0_top:
    iload 1
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_while_0_end
    aload 0
    iload 1
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
    iload 1
    ldc 1
    isub
    istore 1
    goto L_while_0_top
L_while_0_end:
    aload 0
    iconst_0
    iload 1
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    areturn
    ldc ""
    areturn
.end method

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

.method public static error : (I)Ljava/lang/String;
    .limit stack 16
    .limit locals 1

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

.method public static main : ([Ljava/lang/String;)V
    .limit stack 16
    .limit locals 1

    ; Strips leading spaces from self$. Not a real MBASIC/BASCOM 2.00 builtin --
    ; verified against a real IBM BASIC Compiler 2.00 under dosbox-x -- so
    ; BASCAL ships its own. Declared as a scalar method (see GitHub issue #41)
    ; so a required stdlib call reads the same way as a built-in method call
    ; (docs/language/functions-and-procedures.html#built-in-methods). The
    ; ordinary call form (ltrim$(s$)) still works -- a method's receiver is an
    ; implicit first parameter, so ordinary-call syntax resolves straight to
    ; this same declaration, with no separate function needed (and no longer
    ; allowed: a function and a method sharing one name is a duplicate
    ; declaration, since they'd both claim the same callable identity).

    ; Strips trailing spaces from self$. Not a real MBASIC/BASCOM 2.00 builtin --
    ; verified against a real IBM BASIC Compiler 2.00 under dosbox-x -- so
    ; BASCAL ships its own. Declared as a scalar method (see GitHub issue #41
    ; and ltrim.bcl's own doc comment for the reasoning) -- rtrim$(s$) still
    ; works via ordinary-call syntax resolving to this same declaration.

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


    ; Tutorial — Standard library functions
    ;
    ; com.bascal.stdlib is an ordinary require-able library, resolved the same
    ; way as com.bascal.sort in tutorial 12 -- but bcc always adds its home
    ; directory to the search path automatically, so no -L flag is needed to
    ; reach it. It exists because LTRIM$, RTRIM$, UCASE$, LCASE$, and ERROR$
    ; either aren't real MBASIC/BASCOM 2.00 builtins or don't work at runtime
    ; (verified against a real IBM Personal Computer BASIC Compiler 2.00 under
    ; dosbox-x) -- see the manual's "String and error-message functions"
    ; section (https://johnjoeallen.github.io/bascal/manual/) for the full
    ; story.
    ;
    ; ltrim$/rtrim$/ucase$/lcase$ are declared as scalar methods (method$ ...
    ; end method), using self$ in place of an explicit s$ parameter -- see
    ; the "Declare and call a method" chapter. A method's receiver is really
    ; just an implicit first parameter, so the ordinary call form below
    ; (ltrim$("...")) keeps working exactly as before: it resolves straight to
    ; the same method declaration, with the first argument filling self$. The
    ; method-call form (below, chained) is the same declaration too -- just
    ; written as "...".ltrim() instead. error$ stays an ordinary function: an
    ; error code is a lookup key, not a value the call is naturally "operating
    ; on" the way the others operate on their string.
    ;
    ; Run with:
    ; bcc tutorial/stdlib.bcl


    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc "   padded left"
    invokestatic Stdlib/ltrim (Ljava/lang/String;)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc "]"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc "padded right   "
    invokestatic Stdlib/rtrim (Ljava/lang/String;)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc "]"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "shout this"
    invokestatic Stdlib/ucase (Ljava/lang/String;)Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "QUIET THIS DOWN"
    invokestatic Stdlib/lcase (Ljava/lang/String;)Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V

    ; Same four functions, called as chained methods instead.
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc "  padded both sides  "
    invokestatic Stdlib/ltrim (Ljava/lang/String;)Ljava/lang/String;
    invokestatic Stdlib/rtrim (Ljava/lang/String;)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc "]"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  shout this too"
    invokestatic Stdlib/ltrim (Ljava/lang/String;)Ljava/lang/String;
    invokestatic Stdlib/ucase (Ljava/lang/String;)Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V

    ; ERROR$ maps a classic MBASIC/GW-BASIC/BASCOM error code to a message;
    ; pair it with ERR inside an ON ERROR GOTO handler in real code.
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc 53
    invokestatic Stdlib/error (I)Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc 11
    invokestatic Stdlib/error (I)Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc 9999
    invokestatic Stdlib/error (I)Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V

    return
.end method
