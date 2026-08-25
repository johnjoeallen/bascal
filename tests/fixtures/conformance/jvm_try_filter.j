.version 50 0
.class public JvmTryFilter
.super java/lang/Object

.field public static g1 I
.field public static g2 I
.field public static g3 Ljava/lang/String;
.method public static main : ([Ljava/lang/String;)V
    .limit stack 16
    .limit locals 4

    iconst_0
    putstatic JvmTryFilter/g1 I
    iconst_0
    putstatic JvmTryFilter/g2 I
    ldc ""
    putstatic JvmTryFilter/g3 Ljava/lang/String;

L_try_0_start:
    new java/lang/RuntimeException
    dup
    ldc 7
    invokestatic java/lang/Integer/toString (I)Ljava/lang/String;
    invokespecial java/lang/RuntimeException/<init> (Ljava/lang/String;)V
    athrow
    goto L_try_0_finish
L_try_0_end:
L_try_0_catch:
    invokevirtual java/lang/Throwable/getMessage ()Ljava/lang/String;
    invokestatic java/lang/Integer/parseInt (Ljava/lang/String;)I
    putstatic JvmTryFilter/g2 I
    iconst_0
    putstatic JvmTryFilter/g1 I
    ldc "tests/fixtures/conformance/jvm_try_filter.bcl"
    putstatic JvmTryFilter/g3 Ljava/lang/String;
    getstatic JvmTryFilter/g2 I
    ldc 6
    if_icmpeq L_try_0_matched
    getstatic JvmTryFilter/g2 I
    ldc 7
    if_icmpeq L_try_0_matched
    goto L_try_0_rethrow
L_try_0_matched:
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "caught "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    getstatic JvmTryFilter/g2 I
    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic JvmTryFilter/g3 Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_try_0_finish
L_try_0_rethrow:
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "finally"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    new java/lang/RuntimeException
    dup
    getstatic JvmTryFilter/g2 I
    invokestatic java/lang/Integer/toString (I)Ljava/lang/String;
    invokespecial java/lang/RuntimeException/<init> (Ljava/lang/String;)V
    athrow
L_try_0_finish:
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "finally"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V

    return
    .catch java/lang/RuntimeException from L_try_0_start to L_try_0_end using L_try_0_catch
.end method
