.version 50 0
.class public ShortCircuit
.super java/lang/Object

.field public static g1 I
.field public static g2 I
.field public static g3 I
.field public static g4 I
.field public static a0 [I
.method public static isPositive : (I)I
    .limit stack 16
    .limit locals 1

    ; A visible side effect, so the tutorial's own output proves whether
    ; this actually got called.
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  (checking element)"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    iload 0
    ireturn
    iconst_0
    ireturn
.end method

.method public static main : ([Ljava/lang/String;)V
    .limit stack 16
    .limit locals 5

    iconst_0
    putstatic ShortCircuit/g1 I
    iconst_0
    putstatic ShortCircuit/g2 I
    iconst_0
    putstatic ShortCircuit/g3 I
    iconst_0
    putstatic ShortCircuit/g4 I
    ldc 5
    iconst_1
    iadd
    multianewarray [I 1
    putstatic ShortCircuit/a0 [I
    ; Tutorial — Short-Circuit && and ||
    ; 
    ; Classic BASIC's AND/OR are bitwise and always evaluate both sides -- there
    ; is no short-circuit primitive in the generated BASIC at all. && and ||
    ; give BASCAL real short-circuit evaluation instead: the second operand is
    ; only evaluated once the first one hasn't already decided the answer.
    ; 
    ; a && b && c ...   -- true only if every operand is true; stops at the
    ; first false operand.
    ; a || b || c ...   -- true if any operand is true; stops at the first
    ; true operand.
    ; 
    ; && / || are only usable directly in the condition of if / elseif / while
    ; / do -- not as a general expression (can't be assigned to a variable or
    ; passed as a function argument). A condition may chain any number of the
    ; *same* operator; mixing && and || in one condition is a compile-time
    ; error -- split into nested if statements instead.

    ; ---- Guard clause: only check an array element when the index is valid ----

    ; n% -- value to test

    getstatic ShortCircuit/a0 [I
    ldc 0
    ldc 10
    iastore
    getstatic ShortCircuit/a0 [I
    ldc 1
    ldc 5
    ineg
    iastore
    getstatic ShortCircuit/a0 [I
    ldc 2
    ldc 30
    iastore

    ; Long way: nested IF, so isPositive%() is only called when ptr% is valid.
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Long way (nested if), ptr% = -1:"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ldc 1
    ineg
    putstatic ShortCircuit/g3 I
    getstatic ShortCircuit/g3 I
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    iconst_1
    isub
    bipush 31
    ishr
    ifeq L_if_0_else
    getstatic ShortCircuit/a0 [I
    getstatic ShortCircuit/g3 I
    iaload
    invokestatic ShortCircuit/isPositive (I)I
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_if_1_else
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  safe to read, value is positive"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_if_1_end
L_if_1_else:
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  value is not positive"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
L_if_1_end:
    goto L_if_0_end
L_if_0_else:
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  ptr% is out of range"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
L_if_0_end:

    ; Short way: && short-circuits -- same safety, one line, one IF. Watch for
    ; "(checking element)" in the output below: it does NOT print here, proving
    ; isPositive%() was never called for an out-of-range ptr%.
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Short way (&&), ptr% = -1:"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic ShortCircuit/g3 I
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    iconst_1
    isub
    bipush 31
    ishr
    ifeq L_if_2_else
    getstatic ShortCircuit/a0 [I
    getstatic ShortCircuit/g3 I
    iaload
    invokestatic ShortCircuit/isPositive (I)I
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_if_2_else
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  safe to read, value is positive"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_if_2_end
L_if_2_else:
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  ptr% is out of range or value is not positive"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
L_if_2_end:

    ; Same short form, this time with a valid, positive element -- now
    ; "(checking element)" DOES print, since ptr% >= 0 no longer stops it early.
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Short way (&&), ptr% = 2:"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ldc 2
    putstatic ShortCircuit/g3 I
    getstatic ShortCircuit/g3 I
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    iconst_1
    isub
    bipush 31
    ishr
    ifeq L_if_3_else
    getstatic ShortCircuit/a0 [I
    getstatic ShortCircuit/g3 I
    iaload
    invokestatic ShortCircuit/isPositive (I)I
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_if_3_else
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  safe to read, value is positive"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_if_3_end
L_if_3_else:
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  ptr% is out of range or value is not positive"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
L_if_3_end:

    ; ---- Retry loop: stop as soon as we succeed, or once out of attempts ----

    ; Long way: a bare DO with a separate exit for each stopping condition.
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Long way (nested checks), retry loop:"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ldc 0
    putstatic ShortCircuit/g1 I
    ldc 3
    putstatic ShortCircuit/g2 I
    ldc 0
    putstatic ShortCircuit/g4 I
L_do_4_top:
    getstatic ShortCircuit/g1 I
    ldc 1
    iadd
    putstatic ShortCircuit/g1 I
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  attempt "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic ShortCircuit/g1 I
    invokevirtual java/io/PrintStream/println (I)V
    getstatic ShortCircuit/g1 I
    ldc 2
    invokestatic java/lang/Integer/compare (II)I
    dup
    ineg
    ior
    bipush 31
    iushr
    iconst_1
    ixor
    ineg
    ifeq L_if_5_else
    ldc 1
    putstatic ShortCircuit/g4 I
L_if_5_else:
    getstatic ShortCircuit/g4 I
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    dup
    ineg
    ior
    bipush 31
    iushr
    ineg
    ifeq L_if_6_else
    goto L_do_4_end
L_if_6_else:
    getstatic ShortCircuit/g1 I
    getstatic ShortCircuit/g2 I
    invokestatic java/lang/Integer/compare (II)I
    ineg
    iconst_1
    isub
    bipush 31
    ishr
    ifeq L_if_7_else
    goto L_do_4_end
L_if_7_else:
    goto L_do_4_top
L_do_4_end:
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  stopped after "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic ShortCircuit/g1 I
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc " attempt(s), succeeded% = "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic ShortCircuit/g4 I
    invokevirtual java/io/PrintStream/println (I)V

    ; Short way: || short-circuits, so both stopping conditions live in the
    ; loop's own until-clause -- no scattered exit checks needed.
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Short way (||), retry loop:"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ldc 0
    putstatic ShortCircuit/g1 I
    ldc 0
    putstatic ShortCircuit/g4 I
L_do_8_top:
    getstatic ShortCircuit/g4 I
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    dup
    ineg
    ior
    bipush 31
    iushr
    ineg
    ifne L_do_8_end
    getstatic ShortCircuit/g1 I
    getstatic ShortCircuit/g2 I
    invokestatic java/lang/Integer/compare (II)I
    ineg
    iconst_1
    isub
    bipush 31
    ishr
    ifne L_do_8_end
    getstatic ShortCircuit/g1 I
    ldc 1
    iadd
    putstatic ShortCircuit/g1 I
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  attempt "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic ShortCircuit/g1 I
    invokevirtual java/io/PrintStream/println (I)V
    getstatic ShortCircuit/g1 I
    ldc 2
    invokestatic java/lang/Integer/compare (II)I
    dup
    ineg
    ior
    bipush 31
    iushr
    iconst_1
    ixor
    ineg
    ifeq L_if_9_else
    ldc 1
    putstatic ShortCircuit/g4 I
L_if_9_else:
    goto L_do_8_top
L_do_8_end:
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  stopped after "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic ShortCircuit/g1 I
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc " attempt(s), succeeded% = "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic ShortCircuit/g4 I
    invokevirtual java/io/PrintStream/println (I)V

    return
.end method
