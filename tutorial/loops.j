.version 50 0
.class public Loops
.super java/lang/Object

.method public static main : ([Ljava/lang/String;)V
    .limit stack 16
    .limit locals 6

    iconst_0
    istore 1
    iconst_0
    istore 2
    iconst_0
    istore 3
    iconst_0
    istore 4
    iconst_0
    istore 5
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
    istore 1
L_for_0_top:
    iload 1
    ldc 5
    if_icmpgt L_for_0_end
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 1
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "^2 = "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 1
    iload 1
    imul
    invokevirtual java/io/PrintStream/println (I)V
    iload 1
    ldc 1
    iadd
    istore 1
    goto L_for_0_top
L_for_0_end:

    ; Negative STEP — count down
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Countdown:"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ldc 3
    istore 3
L_for_1_top:
    iload 3
    ldc 1
    if_icmplt L_for_1_end
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 3
    invokevirtual java/io/PrintStream/println (I)V
    iload 3
    ldc 1
    ineg
    iadd
    istore 3
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
    istore 1
L_for_2_top:
    iload 1
    ldc 20
    if_icmpgt L_for_2_end
    iload 1
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
    iload 1
    i2d
    ldc 2
    i2d
    ddiv
    ldc 2
    i2d
    dmul
    iload 1
    i2d
    invokestatic java/lang/Double/compare (DD)I
    dup
    ineg
    ior
    bipush 31
    iushr
    iconst_1
    ixor
    ineg
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
    ifeq L_if_3_else
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 1
    invokevirtual java/io/PrintStream/println (I)V
    goto L_for_2_end
L_if_3_else:
    iload 1
    ldc 1
    iadd
    istore 1
    goto L_for_2_top
L_for_2_end:

    ; --- WHILE / WEND ---
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Powers of 2 under 100:"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ldc 1
    istore 4
L_while_4_top:
    iload 4
    ldc 100
    invokestatic java/lang/Integer/compare (II)I
    bipush 31
    ishr
    ifeq L_while_4_end
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 4
    invokevirtual java/io/PrintStream/println (I)V
    iload 4
    ldc 2
    imul
    istore 4
    goto L_while_4_top
L_while_4_end:

    ; exit from a WHILE loop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Collatz from 27 (first 8 steps):"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ldc 27
    istore 3
    ldc 0
    istore 5
L_while_5_top:
    iload 3
    ldc 1
    invokestatic java/lang/Integer/compare (II)I
    dup
    ineg
    ior
    bipush 31
    iushr
    ineg
    ifeq L_while_5_end
    iload 5
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
    ifeq L_if_6_else
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  ..."
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_while_5_end
L_if_6_else:
    iload 3
    i2d
    ldc 2
    i2d
    ddiv
    ldc 2
    i2d
    dmul
    iload 3
    i2d
    invokestatic java/lang/Double/compare (DD)I
    dup
    ineg
    ior
    bipush 31
    iushr
    iconst_1
    ixor
    ineg
    ifeq L_if_7_else
    iload 3
    i2d
    ldc 2
    i2d
    ddiv
    dup2
    ldc2_w 0.5
    dup2_x2
    pop2
    invokestatic java/lang/Math/copySign (DD)D
    dadd
    d2l
    l2i
    istore 3
    goto L_if_7_end
L_if_7_else:
    iload 3
    ldc 3
    imul
    ldc 1
    iadd
    istore 3
L_if_7_end:
    iload 5
    ldc 1
    iadd
    istore 5
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 3
    invokevirtual java/io/PrintStream/println (I)V
    goto L_while_5_top
L_while_5_end:

    ; --- DO / LOOP variants ---

    ; DO WHILE — test before body
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "DO WHILE:"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ldc 1
    istore 2
L_do_8_top:
    iload 2
    ldc 3
    invokestatic java/lang/Integer/compare (II)I
    iconst_1
    isub
    bipush 31
    ishr
    ifeq L_do_8_end
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 2
    invokevirtual java/io/PrintStream/println (I)V
    iload 2
    ldc 1
    iadd
    istore 2
    goto L_do_8_top
L_do_8_end:

    ; DO UNTIL — enter while condition is false
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "DO UNTIL:"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ldc 1
    istore 2
L_do_9_top:
    iload 2
    ldc 3
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifne L_do_9_end
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 2
    invokevirtual java/io/PrintStream/println (I)V
    iload 2
    ldc 1
    iadd
    istore 2
    goto L_do_9_top
L_do_9_end:

    ; DO ... LOOP UNTIL — post-check, body runs at least once
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "DO...LOOP UNTIL (body runs once even though already false):"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ldc 99
    istore 2
L_do_10_top:
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 2
    invokevirtual java/io/PrintStream/println (I)V
    iload 2
    ldc 1
    iadd
    istore 2
    iload 2
    ldc 3
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifne L_do_10_end
    goto L_do_10_top
L_do_10_end:

    ; exit from the middle of a DO loop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "exit at k% = 3:"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ldc 1
    istore 2
L_do_11_top:
    iload 2
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
    ifeq L_if_12_else
    goto L_do_11_end
L_if_12_else:
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 2
    invokevirtual java/io/PrintStream/println (I)V
    iload 2
    ldc 1
    iadd
    istore 2
    goto L_do_11_top
L_do_11_end:

    return
.end method
