.version 50 0
.class public SelectCase
.super java/lang/Object

.method public static main : ([Ljava/lang/String;)V
    .limit stack 16
    .limit locals 5

    iconst_0
    istore 1
    ldc ""
    astore 2
    iconst_0
    istore 3
    iconst_0
    istore 4
    ; Tutorial — SELECT CASE
    ; 
    ; SELECT CASE tests one expression against multiple patterns.  The
    ; compiler evaluates the expression once, stores it in a temporary
    ; variable, and emits an IF/goto dispatch chain.
    ; 
    ; Pattern forms:
    ; case value               — exact match
    ; case v1, v2, v3          — any of the listed values
    ; case low to high         — inclusive range
    ; case is <op> value       — comparison (=  <>  <  <=  >  >=)
    ; case else                — default; must be the last clause

    ; Integer select: convert numeric score to letter grade
    ldc 85
    istore 3

    iload 3
    dup
    ldc 100
    isub
    ifeq L_select_0_case_0
    goto L_select_0_next_0
L_select_0_next_0:
    dup
    ldc 90
    if_icmplt L_select_0_case_1_value_0
    dup
    ldc 99
    if_icmple L_select_0_case_1
L_select_0_case_1_value_0:
    goto L_select_0_next_1
L_select_0_next_1:
    dup
    ldc 80
    if_icmplt L_select_0_case_2_value_0
    dup
    ldc 89
    if_icmple L_select_0_case_2
L_select_0_case_2_value_0:
    goto L_select_0_next_2
L_select_0_next_2:
    dup
    ldc 70
    if_icmplt L_select_0_case_3_value_0
    dup
    ldc 79
    if_icmple L_select_0_case_3
L_select_0_case_3_value_0:
    goto L_select_0_next_3
L_select_0_next_3:
    dup
    ldc 60
    if_icmplt L_select_0_case_4_value_0
    dup
    ldc 69
    if_icmple L_select_0_case_4
L_select_0_case_4_value_0:
    goto L_select_0_next_4
L_select_0_next_4:
    dup
    ldc 0
    if_icmpge L_select_0_case_5
    goto L_select_0_next_5
L_select_0_next_5:
    pop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Invalid score"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_select_0_end
L_select_0_case_0:
    pop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Perfect!"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_select_0_end
L_select_0_case_1:
    pop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "A  — Excellent"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_select_0_end
L_select_0_case_2:
    pop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "B  — Good"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_select_0_end
L_select_0_case_3:
    pop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "C  — Satisfactory"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_select_0_end
L_select_0_case_4:
    pop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "D  — Passing"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_select_0_end
L_select_0_case_5:
    pop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "F  — Fail"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_select_0_end
L_select_0_end:

    ; String select: day-of-week classification
    ldc "Saturday"
    astore 2

    aload 2
    dup
    ldc "Monday"
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ifne L_select_1_case_0
    dup
    ldc "Tuesday"
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ifne L_select_1_case_0
    dup
    ldc "Wednesday"
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ifne L_select_1_case_0
    dup
    ldc "Thursday"
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ifne L_select_1_case_0
    dup
    ldc "Friday"
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ifne L_select_1_case_0
    goto L_select_1_next_0
L_select_1_next_0:
    dup
    ldc "Saturday"
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ifne L_select_1_case_1
    dup
    ldc "Sunday"
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ifne L_select_1_case_1
    goto L_select_1_next_1
L_select_1_next_1:
    pop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "Unknown day: "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    aload 2
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_select_1_end
L_select_1_case_0:
    pop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    aload 2
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc " is a weekday"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_select_1_end
L_select_1_case_1:
    pop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    aload 2
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc " is a weekend"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_select_1_end
L_select_1_end:

    ; IS comparisons on temperature
    ldc 3
    ineg
    istore 4

    iload 4
    dup
    ldc 0
    if_icmplt L_select_2_case_0
    goto L_select_2_next_0
L_select_2_next_0:
    dup
    ldc 10
    if_icmplt L_select_2_case_1
    goto L_select_2_next_1
L_select_2_next_1:
    dup
    ldc 20
    if_icmplt L_select_2_case_2
    goto L_select_2_next_2
L_select_2_next_2:
    dup
    ldc 30
    if_icmplt L_select_2_case_3
    goto L_select_2_next_3
L_select_2_next_3:
    pop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Hot ("
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 4
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "°)"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_select_2_end
L_select_2_case_0:
    pop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Below freezing ("
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 4
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "°)"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_select_2_end
L_select_2_case_1:
    pop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Cold ("
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 4
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "°)"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_select_2_end
L_select_2_case_2:
    pop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Cool ("
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 4
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "°)"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_select_2_end
L_select_2_case_3:
    pop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Warm ("
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iload 4
    invokevirtual java/io/PrintStream/print (I)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "°)"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_select_2_end
L_select_2_end:

    ; Multi-value list on a menu choice
    ldc 2
    istore 1

    iload 1
    dup
    ldc 1
    isub
    ifeq L_select_3_case_0
    goto L_select_3_next_0
L_select_3_next_0:
    dup
    ldc 2
    isub
    ifeq L_select_3_case_1
    dup
    ldc 3
    isub
    ifeq L_select_3_case_1
    goto L_select_3_next_1
L_select_3_next_1:
    dup
    ldc 4
    isub
    ifeq L_select_3_case_2
    goto L_select_3_next_2
L_select_3_next_2:
    pop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Quit"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_select_3_end
L_select_3_case_0:
    pop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "New game"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_select_3_end
L_select_3_case_1:
    pop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Load game"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_select_3_end
L_select_3_case_2:
    pop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Options"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_select_3_end
L_select_3_end:

    return
.end method
