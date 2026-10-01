.version 50 0
.class public Loops
.super java/lang/Object

.field public static g1 I
.field public static g2 I
.field public static g3 I
.field public static g4 I
.field public static g5 I
.method public static main : ([Ljava/lang/String;)V
    .limit stack 16
    .limit locals 12

    iconst_0
    putstatic Loops/g1 I
    iconst_0
    putstatic Loops/g2 I
    iconst_0
    putstatic Loops/g3 I
    iconst_0
    putstatic Loops/g4 I
    iconst_0
    putstatic Loops/g5 I
    ; Tutorial — Loops: for, WHILE, DO
    ;
    ; BASCAL provides three loop constructs:
    ;
    ; for var = start to end [STEP n] ... for END  (or bare END)
    ; Counted loop.  STEP defaults to 1; use negative STEP to count down.
    ;
    ; WHILE condition ... WHILE END  (or bare END)
    ; Condition tested before each iteration.
    ;
    ; DO [WHILE/UNTIL cond] ... END DO  (or bare END)
    ; Pre-check: condition tested at the top, before the body runs at all.
    ; DO ... LOOP [WHILE/UNTIL cond]
    ; Post-check: condition tested at the bottom, so the body always runs
    ; at least once.
    ;
    ; All three loops share one early-exit statement: exit. It's unqualified --
    ; no "exit for"/"exit while"/"exit do" -- the compiler already knows which
    ; loop it's inside from context.
    ; --- for / NEXT ---
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Squares 1..5:"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ldc 1
    istore 6
    ldc 5
    istore 7
    iload 6
    putstatic Loops/g1 I
L_for_0_top:
    getstatic Loops/g1 I
    iload 7
    if_icmpgt L_for_0_end
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Loops/g1 I
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "^2 = "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Loops/g1 I
    getstatic Loops/g1 I
    imul
    invokevirtual java/io/PrintStream/println (I)V
L_for_0_continue:
    getstatic Loops/g1 I
    ldc 1
    iadd
    putstatic Loops/g1 I
    goto L_for_0_top
L_for_0_end:
    ; Negative STEP — count down
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Countdown:"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ldc 3
    istore 8
    ldc 1
    istore 9
    iload 8
    putstatic Loops/g3 I
L_for_1_top:
    getstatic Loops/g3 I
    iload 9
    if_icmplt L_for_1_end
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Loops/g3 I
    invokevirtual java/io/PrintStream/println (I)V
L_for_1_continue:
    getstatic Loops/g3 I
    ldc -1
    iadd
    putstatic Loops/g3 I
    goto L_for_1_top
L_for_1_end:
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  Go!"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ; exit — stop early
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "First even > 4:"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ldc 1
    istore 10
    ldc 20
    istore 11
    iload 10
    putstatic Loops/g1 I
L_for_2_top:
    getstatic Loops/g1 I
    iload 11
    if_icmpgt L_for_2_end
    getstatic Loops/g1 I
    ldc 4
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
    getstatic Loops/g1 I
    i2d
    ldc 2
    i2d
    ddiv
    d2f
    f2d
    d2f
    f2d
    ldc 2
    i2d
    dmul
    d2f
    f2d
    getstatic Loops/g1 I
    i2d
    dcmpl
    ifeq L_condition_3
    iconst_0
    goto L_condition_4
L_condition_3:
    iconst_m1
L_condition_4:
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
    ifeq L_condition_5
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Loops/g1 I
    invokevirtual java/io/PrintStream/println (I)V
    goto L_for_2_end
L_condition_5:
L_for_2_continue:
    getstatic Loops/g1 I
    ldc 1
    iadd
    putstatic Loops/g1 I
    goto L_for_2_top
L_for_2_end:
    ; --- WHILE / WEND ---
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Powers of 2 under 100:"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ldc 1
    putstatic Loops/g4 I
L_condition_7:
    getstatic Loops/g4 I
    ldc 100
    invokestatic java/lang/Integer/compare (II)I
    bipush 31
    ishr
    ifeq L_condition_8
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Loops/g4 I
    invokevirtual java/io/PrintStream/println (I)V
    getstatic Loops/g4 I
    ldc 2
    imul
    putstatic Loops/g4 I
    goto L_condition_7
L_condition_8:
    ; exit from a WHILE loop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Collatz from 27 (first 8 steps):"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ldc 27
    putstatic Loops/g3 I
    ldc 0
    putstatic Loops/g5 I
L_condition_9:
    getstatic Loops/g3 I
    ldc 1
    invokestatic java/lang/Integer/compare (II)I
    dup
    ineg
    ior
    bipush 31
    iushr
    ineg
    ifeq L_condition_10
    getstatic Loops/g5 I
    ldc 8
    invokestatic java/lang/Integer/compare (II)I
    dup
    ineg
    ior
    bipush 31
    iushr
    iconst_1
    ixor
    ineg
    ifeq L_condition_11
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  ..."
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_condition_10
L_condition_11:
    getstatic Loops/g3 I
    i2d
    ldc 2
    i2d
    ddiv
    d2f
    f2d
    d2f
    f2d
    ldc 2
    i2d
    dmul
    d2f
    f2d
    getstatic Loops/g3 I
    i2d
    dcmpl
    ifeq L_condition_13
    iconst_0
    goto L_condition_14
L_condition_13:
    iconst_m1
L_condition_14:
    ifeq L_condition_15
    getstatic Loops/g3 I
    i2d
    ldc 2
    i2d
    ddiv
    d2f
    f2d
    dup2
    ldc2_w 0.5
    dup2_x2
    pop2
    invokestatic java/lang/Math/copySign (DD)D
    dadd
    d2l
    l2i
    putstatic Loops/g3 I
    goto L_condition_16
L_condition_15:
    getstatic Loops/g3 I
    ldc 3
    imul
    ldc 1
    iadd
    putstatic Loops/g3 I
L_condition_16:
    getstatic Loops/g5 I
    ldc 1
    iadd
    putstatic Loops/g5 I
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Loops/g3 I
    invokevirtual java/io/PrintStream/println (I)V
    goto L_condition_9
L_condition_10:
    ; --- DO / LOOP variants ---
    ; DO WHILE — test before body
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "DO WHILE:"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ldc 1
    putstatic Loops/g2 I
L_do_17_top:
    getstatic Loops/g2 I
    ldc 3
    invokestatic java/lang/Integer/compare (II)I
    iconst_1
    isub
    bipush 31
    ishr
    ifeq L_do_17_end
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Loops/g2 I
    invokevirtual java/io/PrintStream/println (I)V
    getstatic Loops/g2 I
    ldc 1
    iadd
    putstatic Loops/g2 I
L_do_17_continue:
    goto L_do_17_top
L_do_17_end:
    ; DO UNTIL — enter while condition is false
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "DO UNTIL:"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ldc 1
    putstatic Loops/g2 I
L_do_18_top:
    getstatic Loops/g2 I
    ldc 3
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifne L_do_18_end
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Loops/g2 I
    invokevirtual java/io/PrintStream/println (I)V
    getstatic Loops/g2 I
    ldc 1
    iadd
    putstatic Loops/g2 I
L_do_18_continue:
    goto L_do_18_top
L_do_18_end:
    ; DO ... LOOP UNTIL — post-check, body runs at least once
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "DO...LOOP UNTIL (body runs once even though already false):"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ldc 99
    putstatic Loops/g2 I
L_do_19_top:
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Loops/g2 I
    invokevirtual java/io/PrintStream/println (I)V
    getstatic Loops/g2 I
    ldc 1
    iadd
    putstatic Loops/g2 I
L_do_19_continue:
    getstatic Loops/g2 I
    ldc 3
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifne L_do_19_end
    goto L_do_19_top
L_do_19_end:
    ; exit from the middle of a DO loop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "exit at k% = 3:"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ldc 1
    putstatic Loops/g2 I
L_do_20_top:
    getstatic Loops/g2 I
    ldc 3
    invokestatic java/lang/Integer/compare (II)I
    dup
    ineg
    ior
    bipush 31
    iushr
    iconst_1
    ixor
    ineg
    ifeq L_condition_21
    goto L_do_20_end
L_condition_21:
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Loops/g2 I
    invokevirtual java/io/PrintStream/println (I)V
    getstatic Loops/g2 I
    ldc 1
    iadd
    putstatic Loops/g2 I
L_do_20_continue:
    goto L_do_20_top
L_do_20_end:
    return
.end method
