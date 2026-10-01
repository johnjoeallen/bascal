.version 50 0
.class public CardCatalog
.super java/lang/Object

.field public static g1 Ljava/lang/String;
.field public static g2 Ljava/lang/String;
.field public static g3 Ljava/lang/String;
.field public static g4 Ljava/lang/String;
.field public static g5 Ljava/lang/String;
.field public static g6 Ljava/lang/String;
.field public static g7 Ljava/lang/String;
.field public static g8 I
.field public static g9 Ljava/lang/String;
.field public static g10 I
.field public static g11 I
.field public static bccFiles [Ljava/io/RandomAccessFile;
.field public static bccBufs [[B
.field public static bccRecLen [I
.field public static bccStdin Ljava/io/BufferedReader;
.method public static ltrim : (Ljava/lang/String;)Ljava/lang/String;
    .limit stack 16
    .limit locals 12

    ldc ""
    astore 1
    ldc ""
    astore 2
    ldc ""
    astore 3
    ldc ""
    astore 4
    ldc ""
    astore 5
    ldc ""
    astore 6
    ldc ""
    astore 7
    iconst_0
    istore 8
    ldc ""
    astore 9
    iconst_0
    istore 10
    iconst_0
    istore 11
    ldc 1
    istore 11
L_condition_0:
    iload 11
    aload 0
    invokevirtual java/lang/String/length ()I
    invokestatic java/lang/Integer/compare (II)I
    iconst_1
    isub
    bipush 31
    ishr
    ifeq L_condition_2
    aload 0
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
    ifne L_condition_4
    iconst_0
    goto L_condition_5
L_condition_4:
    iconst_m1
L_condition_5:
    goto L_condition_3
L_condition_2:
    iconst_0
L_condition_3:
    ifeq L_condition_1
    iload 11
    ldc 1
    iadd
    istore 11
    goto L_condition_0
L_condition_1:
    aload 0
    iload 11
    iconst_1
    isub
    invokevirtual java/lang/String/substring (I)Ljava/lang/String;
    areturn
    ldc ""
    areturn
.end method

.method public static rtrim : (Ljava/lang/String;)Ljava/lang/String;
    .limit stack 16
    .limit locals 12

    ldc ""
    astore 1
    ldc ""
    astore 2
    ldc ""
    astore 3
    ldc ""
    astore 4
    ldc ""
    astore 5
    ldc ""
    astore 6
    ldc ""
    astore 7
    iconst_0
    istore 8
    ldc ""
    astore 9
    iconst_0
    istore 10
    iconst_0
    istore 11
    aload 0
    invokevirtual java/lang/String/length ()I
    istore 11
L_condition_0:
    iload 11
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_condition_2
    aload 0
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
    ifne L_condition_4
    iconst_0
    goto L_condition_5
L_condition_4:
    iconst_m1
L_condition_5:
    goto L_condition_3
L_condition_2:
    iconst_0
L_condition_3:
    ifeq L_condition_1
    iload 11
    ldc 1
    isub
    istore 11
    goto L_condition_0
L_condition_1:
    aload 0
    iconst_0
    iload 11
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    areturn
    ldc ""
    areturn
.end method

.method public static initCatalog : ()V
    .limit stack 16
    .limit locals 13

    ldc ""
    astore 0
    ldc ""
    astore 1
    ldc ""
    astore 2
    ldc ""
    astore 3
    ldc ""
    astore 4
    ldc ""
    astore 5
    ldc ""
    astore 6
    iconst_0
    istore 7
    ldc ""
    astore 8
    iconst_0
    istore 9
    iconst_0
    istore 10
    ; global header
    ; global catalog
    ; header[...] = { ... }  (whole-record write)
    ldc 2
    invokestatic java/nio/ByteBuffer/allocate (I)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    getstatic CardCatalog/g11 I
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
    getstatic CardCatalog/bccBufs [[B
    ldc 0
    aaload
    ldc 0
    ldc 2
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    ldc ""
    ldc 58
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
    ldc 58
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    iconst_0
    getstatic CardCatalog/bccBufs [[B
    ldc 0
    aaload
    ldc 2
    ldc 58
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    getstatic CardCatalog/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    dup
    ldc 1
    i2l
    lconst_1
    lsub
    getstatic CardCatalog/bccRecLen [I
    ldc 1
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic CardCatalog/bccBufs [[B
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/write ([B)V
    ldc 2
    istore 11
    getstatic CardCatalog/g11 I
    istore 12
    iload 11
    istore 10
L_for_0_top:
    iload 10
    iload 12
    if_icmpgt L_for_0_end
    ; catalog[...] = { ... }  (whole-record write)
    ldc ""
    ldc 20
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
    ldc 20
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    iconst_0
    getstatic CardCatalog/bccBufs [[B
    ldc 1
    aaload
    ldc 0
    ldc 20
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    ldc ""
    ldc 20
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
    ldc 20
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    iconst_0
    getstatic CardCatalog/bccBufs [[B
    ldc 1
    aaload
    ldc 20
    ldc 20
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    ldc ""
    ldc 20
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
    ldc 20
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    iconst_0
    getstatic CardCatalog/bccBufs [[B
    ldc 1
    aaload
    ldc 40
    ldc 20
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    getstatic CardCatalog/bccFiles [Ljava/io/RandomAccessFile;
    ldc 2
    iconst_1
    isub
    aaload
    dup
    iload 10
    i2l
    lconst_1
    lsub
    getstatic CardCatalog/bccRecLen [I
    ldc 2
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic CardCatalog/bccBufs [[B
    ldc 2
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/write ([B)V
L_for_0_continue:
    iload 10
    ldc 1
    iadd
    istore 10
    goto L_for_0_top
L_for_0_end:
    return
.end method

.method public static addItem : (Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)V
    .limit stack 16
    .limit locals 15

    ldc ""
    astore 3
    ldc ""
    astore 4
    ldc ""
    astore 5
    ldc ""
    astore 6
    ldc ""
    astore 7
    ldc ""
    astore 8
    ldc ""
    astore 9
    iconst_0
    istore 10
    ldc ""
    astore 11
    iconst_0
    istore 12
    iconst_0
    istore 13
    iconst_0
    istore 14
    ; global header
    ; global catalog
    ; let h = header[...]  (whole-record read)
    getstatic CardCatalog/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    dup
    ldc 1
    i2l
    lconst_1
    lsub
    getstatic CardCatalog/bccRecLen [I
    ldc 1
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic CardCatalog/bccBufs [[B
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/read ([B)I
    pop
    getstatic CardCatalog/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 0
    ldc 2
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    invokestatic java/nio/ByteBuffer/wrap ([B)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/getShort ()S
    istore 12
    getstatic CardCatalog/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 2
    ldc 58
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    invokestatic CardCatalog/rtrim (Ljava/lang/String;)Ljava/lang/String;
    astore 11
    ldc 1
    istore 13
    ldc 0
    istore 14
L_do_0_top:
    iload 14
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
    ifeq L_do_0_end
    iload 13
    ldc 1
    iadd
    istore 13
    ; let e = catalog[...]  (whole-record read)
    getstatic CardCatalog/bccFiles [Ljava/io/RandomAccessFile;
    ldc 2
    iconst_1
    isub
    aaload
    dup
    iload 13
    i2l
    lconst_1
    lsub
    getstatic CardCatalog/bccRecLen [I
    ldc 2
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic CardCatalog/bccBufs [[B
    ldc 2
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/read ([B)I
    pop
    getstatic CardCatalog/bccBufs [[B
    ldc 1
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 0
    ldc 20
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    invokestatic CardCatalog/rtrim (Ljava/lang/String;)Ljava/lang/String;
    astore 6
    getstatic CardCatalog/bccBufs [[B
    ldc 1
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 20
    ldc 20
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    invokestatic CardCatalog/rtrim (Ljava/lang/String;)Ljava/lang/String;
    astore 8
    getstatic CardCatalog/bccBufs [[B
    ldc 1
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 40
    ldc 20
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    invokestatic CardCatalog/rtrim (Ljava/lang/String;)Ljava/lang/String;
    astore 7
    aload 6
    ldc ""
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ineg
    ifeq L_condition_1
    ldc 1
    istore 14
L_condition_1:
    iload 13
    iload 12
    invokestatic java/lang/Integer/compare (II)I
    dup
    ineg
    ior
    bipush 31
    iushr
    iconst_1
    ixor
    ineg
    ifeq L_condition_3
    ldc 1
    istore 14
L_condition_3:
L_do_0_continue:
    goto L_do_0_top
L_do_0_end:
    aload 6
    ldc ""
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ineg
    ifeq L_condition_5
    ; catalog[...] = { ... }  (whole-record write)
    aload 0
    ldc 20
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
    ldc 20
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    iconst_0
    getstatic CardCatalog/bccBufs [[B
    ldc 1
    aaload
    ldc 0
    ldc 20
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    aload 1
    ldc 20
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
    ldc 20
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    iconst_0
    getstatic CardCatalog/bccBufs [[B
    ldc 1
    aaload
    ldc 20
    ldc 20
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    aload 2
    ldc 20
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
    ldc 20
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    iconst_0
    getstatic CardCatalog/bccBufs [[B
    ldc 1
    aaload
    ldc 40
    ldc 20
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    getstatic CardCatalog/bccFiles [Ljava/io/RandomAccessFile;
    ldc 2
    iconst_1
    isub
    aaload
    dup
    iload 13
    i2l
    lconst_1
    lsub
    getstatic CardCatalog/bccRecLen [I
    ldc 2
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic CardCatalog/bccBufs [[B
    ldc 2
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/write ([B)V
    goto L_condition_6
L_condition_5:
    getstatic java/lang/System/out Ljava/io/PrintStream;
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    ldc "Catalog is full -- cannot add "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    aload 0
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
L_condition_6:
    return
.end method

.method public static listAll : ()V
    .limit stack 16
    .limit locals 13

    ldc ""
    astore 0
    ldc ""
    astore 1
    ldc ""
    astore 2
    ldc ""
    astore 3
    ldc ""
    astore 4
    ldc ""
    astore 5
    ldc ""
    astore 6
    iconst_0
    istore 7
    ldc ""
    astore 8
    iconst_0
    istore 9
    iconst_0
    istore 10
    ; global header
    ; global catalog
    ; let h = header[...]  (whole-record read)
    getstatic CardCatalog/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    dup
    ldc 1
    i2l
    lconst_1
    lsub
    getstatic CardCatalog/bccRecLen [I
    ldc 1
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic CardCatalog/bccBufs [[B
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/read ([B)I
    pop
    getstatic CardCatalog/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 0
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
    getstatic CardCatalog/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 2
    ldc 58
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    invokestatic CardCatalog/rtrim (Ljava/lang/String;)Ljava/lang/String;
    astore 8
    ldc 2
    istore 11
    iload 9
    istore 12
    iload 11
    istore 10
L_for_0_top:
    iload 10
    iload 12
    if_icmpgt L_for_0_end
    ; let e = catalog[...]  (whole-record read)
    getstatic CardCatalog/bccFiles [Ljava/io/RandomAccessFile;
    ldc 2
    iconst_1
    isub
    aaload
    dup
    iload 10
    i2l
    lconst_1
    lsub
    getstatic CardCatalog/bccRecLen [I
    ldc 2
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic CardCatalog/bccBufs [[B
    ldc 2
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/read ([B)I
    pop
    getstatic CardCatalog/bccBufs [[B
    ldc 1
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 0
    ldc 20
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    invokestatic CardCatalog/rtrim (Ljava/lang/String;)Ljava/lang/String;
    astore 3
    getstatic CardCatalog/bccBufs [[B
    ldc 1
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 20
    ldc 20
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    invokestatic CardCatalog/rtrim (Ljava/lang/String;)Ljava/lang/String;
    astore 5
    getstatic CardCatalog/bccBufs [[B
    ldc 1
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 40
    ldc 20
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    invokestatic CardCatalog/rtrim (Ljava/lang/String;)Ljava/lang/String;
    astore 4
    aload 3
    ldc ""
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    iconst_1
    ixor
    ineg
    ifeq L_condition_1
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
    aload 3
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc "  |  "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    aload 5
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc "  |  "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    aload 4
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
L_condition_1:
L_for_0_continue:
    iload 10
    ldc 1
    iadd
    istore 10
    goto L_for_0_top
L_for_0_end:
    return
.end method

.method public static searchByAuthor : (Ljava/lang/String;)V
    .limit stack 16
    .limit locals 14

    ldc ""
    astore 1
    ldc ""
    astore 2
    ldc ""
    astore 3
    ldc ""
    astore 4
    ldc ""
    astore 5
    ldc ""
    astore 6
    ldc ""
    astore 7
    iconst_0
    istore 8
    ldc ""
    astore 9
    iconst_0
    istore 10
    iconst_0
    istore 11
    ; global header
    ; global catalog
    ; let h = header[...]  (whole-record read)
    getstatic CardCatalog/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    dup
    ldc 1
    i2l
    lconst_1
    lsub
    getstatic CardCatalog/bccRecLen [I
    ldc 1
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic CardCatalog/bccBufs [[B
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/read ([B)I
    pop
    getstatic CardCatalog/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 0
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
    getstatic CardCatalog/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 2
    ldc 58
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    invokestatic CardCatalog/rtrim (Ljava/lang/String;)Ljava/lang/String;
    astore 9
    ldc 2
    istore 12
    iload 10
    istore 13
    iload 12
    istore 11
L_for_0_top:
    iload 11
    iload 13
    if_icmpgt L_for_0_end
    ; let e = catalog[...]  (whole-record read)
    getstatic CardCatalog/bccFiles [Ljava/io/RandomAccessFile;
    ldc 2
    iconst_1
    isub
    aaload
    dup
    iload 11
    i2l
    lconst_1
    lsub
    getstatic CardCatalog/bccRecLen [I
    ldc 2
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic CardCatalog/bccBufs [[B
    ldc 2
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/read ([B)I
    pop
    getstatic CardCatalog/bccBufs [[B
    ldc 1
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 0
    ldc 20
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    invokestatic CardCatalog/rtrim (Ljava/lang/String;)Ljava/lang/String;
    astore 4
    getstatic CardCatalog/bccBufs [[B
    ldc 1
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 20
    ldc 20
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    invokestatic CardCatalog/rtrim (Ljava/lang/String;)Ljava/lang/String;
    astore 6
    getstatic CardCatalog/bccBufs [[B
    ldc 1
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 40
    ldc 20
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    invokestatic CardCatalog/rtrim (Ljava/lang/String;)Ljava/lang/String;
    astore 5
    aload 4
    aload 0
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ineg
    ifeq L_condition_1
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
    aload 4
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc "  |  "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    aload 6
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc "  |  "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    aload 5
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
L_condition_1:
L_for_0_continue:
    iload 11
    ldc 1
    iadd
    istore 11
    goto L_for_0_top
L_for_0_end:
    return
.end method

.method public static searchByAuthorTitle : (Ljava/lang/String;Ljava/lang/String;)V
    .limit stack 16
    .limit locals 15

    ldc ""
    astore 2
    ldc ""
    astore 3
    ldc ""
    astore 4
    ldc ""
    astore 5
    ldc ""
    astore 6
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
    iconst_0
    istore 12
    ; global header
    ; global catalog
    ; let h = header[...]  (whole-record read)
    getstatic CardCatalog/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    dup
    ldc 1
    i2l
    lconst_1
    lsub
    getstatic CardCatalog/bccRecLen [I
    ldc 1
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic CardCatalog/bccBufs [[B
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/read ([B)I
    pop
    getstatic CardCatalog/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 0
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
    getstatic CardCatalog/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 2
    ldc 58
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    invokestatic CardCatalog/rtrim (Ljava/lang/String;)Ljava/lang/String;
    astore 10
    ldc 2
    istore 13
    iload 11
    istore 14
    iload 13
    istore 12
L_for_0_top:
    iload 12
    iload 14
    if_icmpgt L_for_0_end
    ; let e = catalog[...]  (whole-record read)
    getstatic CardCatalog/bccFiles [Ljava/io/RandomAccessFile;
    ldc 2
    iconst_1
    isub
    aaload
    dup
    iload 12
    i2l
    lconst_1
    lsub
    getstatic CardCatalog/bccRecLen [I
    ldc 2
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic CardCatalog/bccBufs [[B
    ldc 2
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/read ([B)I
    pop
    getstatic CardCatalog/bccBufs [[B
    ldc 1
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 0
    ldc 20
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    invokestatic CardCatalog/rtrim (Ljava/lang/String;)Ljava/lang/String;
    astore 5
    getstatic CardCatalog/bccBufs [[B
    ldc 1
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 20
    ldc 20
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    invokestatic CardCatalog/rtrim (Ljava/lang/String;)Ljava/lang/String;
    astore 7
    getstatic CardCatalog/bccBufs [[B
    ldc 1
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 40
    ldc 20
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    invokestatic CardCatalog/rtrim (Ljava/lang/String;)Ljava/lang/String;
    astore 6
    aload 5
    aload 0
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ineg
    ifeq L_condition_1
    aload 7
    aload 1
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ineg
    ifne L_condition_3
    iconst_0
    goto L_condition_4
L_condition_3:
    iconst_m1
L_condition_4:
    goto L_condition_2
L_condition_1:
    iconst_0
L_condition_2:
    ifeq L_condition_5
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
    aload 5
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc "  |  "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    aload 7
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc "  |  "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    aload 6
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
L_condition_5:
L_for_0_continue:
    iload 12
    ldc 1
    iadd
    istore 12
    goto L_for_0_top
L_for_0_end:
    return
.end method

.method public static deleteItem : (Ljava/lang/String;Ljava/lang/String;)V
    .limit stack 16
    .limit locals 14

    ldc ""
    astore 2
    ldc ""
    astore 3
    ldc ""
    astore 4
    ldc ""
    astore 5
    ldc ""
    astore 6
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
    iconst_0
    istore 12
    iconst_0
    istore 13
    ; global header
    ; global catalog
    ; let h = header[...]  (whole-record read)
    getstatic CardCatalog/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    dup
    ldc 1
    i2l
    lconst_1
    lsub
    getstatic CardCatalog/bccRecLen [I
    ldc 1
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic CardCatalog/bccBufs [[B
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/read ([B)I
    pop
    getstatic CardCatalog/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 0
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
    getstatic CardCatalog/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 2
    ldc 58
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    invokestatic CardCatalog/rtrim (Ljava/lang/String;)Ljava/lang/String;
    astore 10
    ldc 1
    istore 12
    ldc 0
    istore 13
L_do_0_top:
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
    ifeq L_do_0_end
    iload 12
    ldc 1
    iadd
    istore 12
    ; let e = catalog[...]  (whole-record read)
    getstatic CardCatalog/bccFiles [Ljava/io/RandomAccessFile;
    ldc 2
    iconst_1
    isub
    aaload
    dup
    iload 12
    i2l
    lconst_1
    lsub
    getstatic CardCatalog/bccRecLen [I
    ldc 2
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic CardCatalog/bccBufs [[B
    ldc 2
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/read ([B)I
    pop
    getstatic CardCatalog/bccBufs [[B
    ldc 1
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 0
    ldc 20
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    invokestatic CardCatalog/rtrim (Ljava/lang/String;)Ljava/lang/String;
    astore 5
    getstatic CardCatalog/bccBufs [[B
    ldc 1
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 20
    ldc 20
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    invokestatic CardCatalog/rtrim (Ljava/lang/String;)Ljava/lang/String;
    astore 7
    getstatic CardCatalog/bccBufs [[B
    ldc 1
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 40
    ldc 20
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    invokestatic CardCatalog/rtrim (Ljava/lang/String;)Ljava/lang/String;
    astore 6
    aload 5
    aload 0
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ineg
    ifeq L_condition_1
    aload 7
    aload 1
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ineg
    ifne L_condition_3
    iconst_0
    goto L_condition_4
L_condition_3:
    iconst_m1
L_condition_4:
    goto L_condition_2
L_condition_1:
    iconst_0
L_condition_2:
    ifeq L_condition_5
    ldc 1
    istore 13
L_condition_5:
    iload 12
    iload 11
    invokestatic java/lang/Integer/compare (II)I
    dup
    ineg
    ior
    bipush 31
    iushr
    iconst_1
    ixor
    ineg
    ifeq L_condition_7
    ldc 1
    istore 13
L_condition_7:
L_do_0_continue:
    goto L_do_0_top
L_do_0_end:
    aload 5
    aload 0
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ineg
    ifeq L_condition_9
    aload 7
    aload 1
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ineg
    ifne L_condition_11
    iconst_0
    goto L_condition_12
L_condition_11:
    iconst_m1
L_condition_12:
    goto L_condition_10
L_condition_9:
    iconst_0
L_condition_10:
    ifeq L_condition_13
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
    ldc "Deleting: "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    aload 5
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc "  |  "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    aload 7
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ; catalog[...] = { ... }  (whole-record write)
    ldc ""
    ldc 20
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
    ldc 20
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    iconst_0
    getstatic CardCatalog/bccBufs [[B
    ldc 1
    aaload
    ldc 0
    ldc 20
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    ldc ""
    ldc 20
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
    ldc 20
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    iconst_0
    getstatic CardCatalog/bccBufs [[B
    ldc 1
    aaload
    ldc 20
    ldc 20
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    ldc ""
    ldc 20
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
    ldc 20
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    iconst_0
    getstatic CardCatalog/bccBufs [[B
    ldc 1
    aaload
    ldc 40
    ldc 20
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    getstatic CardCatalog/bccFiles [Ljava/io/RandomAccessFile;
    ldc 2
    iconst_1
    isub
    aaload
    dup
    iload 12
    i2l
    lconst_1
    lsub
    getstatic CardCatalog/bccRecLen [I
    ldc 2
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic CardCatalog/bccBufs [[B
    ldc 2
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/write ([B)V
    goto L_condition_14
L_condition_13:
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
    ldc "Not found: "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    aload 0
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc "  |  "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    aload 1
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
L_condition_14:
    return
.end method

.method public static mainMenu : ()V
    .limit stack 16
    .limit locals 15

    ldc ""
    astore 0
    ldc ""
    astore 1
    ldc ""
    astore 2
    ldc ""
    astore 3
    iconst_0
    istore 4
    ldc ""
    astore 5
    ldc ""
    astore 6
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
    iconst_0
    istore 12
    ldc ""
    astore 13
    ldc ""
    astore 14
    ldc 1
    istore 12
L_do_0_top:
    iload 12
    ldc 1
    invokestatic java/lang/Integer/compare (II)I
    dup
    ineg
    ior
    bipush 31
    iushr
    iconst_1
    ixor
    ineg
    ifeq L_do_0_end
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc ""
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "MENU.          1 ) LIST ALL ITEMS"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "               2 ) NEW ITEM"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "               3 ) SEARCH BY AUTHOR"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "               4 ) SEARCH BY AUTHOR + TITLE"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "               5 ) DELETE ITEM"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "               6 ) STOP"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc ""
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "CHOICE: ? "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    invokevirtual java/io/PrintStream/flush ()V
    getstatic CardCatalog/bccStdin Ljava/io/BufferedReader;
    invokevirtual java/io/BufferedReader/readLine ()Ljava/lang/String;
    invokevirtual java/lang/String/trim ()Ljava/lang/String;
    invokestatic java/lang/Integer/parseInt (Ljava/lang/String;)I
    istore 4
    iload 4
    dup
    ldc 1
    isub
    ifeq L_select_1_case_0
    goto L_select_1_next_0
L_select_1_next_0:
    dup
    ldc 2
    isub
    ifeq L_select_1_case_1
    goto L_select_1_next_1
L_select_1_next_1:
    dup
    ldc 3
    isub
    ifeq L_select_1_case_2
    goto L_select_1_next_2
L_select_1_next_2:
    dup
    ldc 4
    isub
    ifeq L_select_1_case_3
    goto L_select_1_next_3
L_select_1_next_3:
    dup
    ldc 5
    isub
    ifeq L_select_1_case_4
    goto L_select_1_next_4
L_select_1_next_4:
    dup
    ldc 6
    isub
    ifeq L_select_1_case_5
    goto L_select_1_next_5
L_select_1_next_5:
    pop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Invalid choice"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    goto L_select_1_end
L_select_1_case_0:
    pop
    invokestatic CardCatalog/listAll ()V
    goto L_select_1_end
L_select_1_case_1:
    pop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "AUTHOR  ? "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    invokevirtual java/io/PrintStream/flush ()V
    getstatic CardCatalog/bccStdin Ljava/io/BufferedReader;
    invokevirtual java/io/BufferedReader/readLine ()Ljava/lang/String;
    astore 0
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "TITLE   ? "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    invokevirtual java/io/PrintStream/flush ()V
    getstatic CardCatalog/bccStdin Ljava/io/BufferedReader;
    invokevirtual java/io/BufferedReader/readLine ()Ljava/lang/String;
    astore 14
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "SUBJECT ? "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    invokevirtual java/io/PrintStream/flush ()V
    getstatic CardCatalog/bccStdin Ljava/io/BufferedReader;
    invokevirtual java/io/BufferedReader/readLine ()Ljava/lang/String;
    astore 13
    aload 0
    aload 14
    aload 13
    invokestatic CardCatalog/addItem (Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)V
    goto L_select_1_end
L_select_1_case_2:
    pop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "AUTHOR ? "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    invokevirtual java/io/PrintStream/flush ()V
    getstatic CardCatalog/bccStdin Ljava/io/BufferedReader;
    invokevirtual java/io/BufferedReader/readLine ()Ljava/lang/String;
    astore 0
    aload 0
    invokestatic CardCatalog/searchByAuthor (Ljava/lang/String;)V
    goto L_select_1_end
L_select_1_case_3:
    pop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "AUTHOR ? "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    invokevirtual java/io/PrintStream/flush ()V
    getstatic CardCatalog/bccStdin Ljava/io/BufferedReader;
    invokevirtual java/io/BufferedReader/readLine ()Ljava/lang/String;
    astore 0
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "TITLE  ? "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    invokevirtual java/io/PrintStream/flush ()V
    getstatic CardCatalog/bccStdin Ljava/io/BufferedReader;
    invokevirtual java/io/BufferedReader/readLine ()Ljava/lang/String;
    astore 14
    aload 0
    aload 14
    invokestatic CardCatalog/searchByAuthorTitle (Ljava/lang/String;Ljava/lang/String;)V
    goto L_select_1_end
L_select_1_case_4:
    pop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "AUTHOR (to delete) ? "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    invokevirtual java/io/PrintStream/flush ()V
    getstatic CardCatalog/bccStdin Ljava/io/BufferedReader;
    invokevirtual java/io/BufferedReader/readLine ()Ljava/lang/String;
    astore 0
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "TITLE  (to delete) ? "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    invokevirtual java/io/PrintStream/flush ()V
    getstatic CardCatalog/bccStdin Ljava/io/BufferedReader;
    invokevirtual java/io/BufferedReader/readLine ()Ljava/lang/String;
    astore 14
    aload 0
    aload 14
    invokestatic CardCatalog/deleteItem (Ljava/lang/String;Ljava/lang/String;)V
    goto L_select_1_end
L_select_1_case_5:
    pop
    ldc 0
    istore 12
    goto L_select_1_end
L_select_1_end:
L_do_0_continue:
    goto L_do_0_top
L_do_0_end:
    return
.end method

.method public static main : ([Ljava/lang/String;)V
    .limit stack 16
    .limit locals 12

    ldc ""
    putstatic CardCatalog/g1 Ljava/lang/String;
    ldc ""
    putstatic CardCatalog/g2 Ljava/lang/String;
    ldc ""
    putstatic CardCatalog/g3 Ljava/lang/String;
    ldc ""
    putstatic CardCatalog/g4 Ljava/lang/String;
    ldc ""
    putstatic CardCatalog/g5 Ljava/lang/String;
    ldc ""
    putstatic CardCatalog/g6 Ljava/lang/String;
    ldc ""
    putstatic CardCatalog/g7 Ljava/lang/String;
    iconst_0
    putstatic CardCatalog/g8 I
    ldc ""
    putstatic CardCatalog/g9 Ljava/lang/String;
    iconst_0
    putstatic CardCatalog/g10 I
    iconst_0
    putstatic CardCatalog/g11 I
    ldc 16
    anewarray java/io/RandomAccessFile
    putstatic CardCatalog/bccFiles [Ljava/io/RandomAccessFile;
    ldc 16
    anewarray [B
    putstatic CardCatalog/bccBufs [[B
    ldc 16
    newarray int
    putstatic CardCatalog/bccRecLen [I
    new java/io/BufferedReader
    dup
    new java/io/InputStreamReader
    dup
    getstatic java/lang/System/in Ljava/io/InputStream;
    invokespecial java/io/InputStreamReader/<init> (Ljava/io/InputStream;)V
    invokespecial java/io/BufferedReader/<init> (Ljava/io/Reader;)V
    putstatic CardCatalog/bccStdin Ljava/io/BufferedReader;
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
    ; Card Catalog — a flagship example for the record/file DSL + procedures
    ;
    ; Adapted from CLERK.BAS, a menu-driven card-catalog manager written by
    ; Carlos A. Lujan S. in February 1983 as an improved version of Alfred
    ; Fant's LIBRARIAN program (Microcomputing, December 1982). The original
    ; source lives in the PeatSoft GW-FILES collection, in the
    ; robhagemans/hoard-of-gwbasic archive on GitHub
    ; (PeatSoft/GWFILES/CLERK.BAS).
    ;
    ; What's carried over from CLERK.BAS:
    ; - One random-access file holding a header record (the catalog's
    ; capacity) in slot 1, followed by author/title/subject entry records
    ; in the remaining slots.
    ; - NEW ITEM: linear-scan the entries for the first empty slot.
    ; - Searches by author, and by author + title together.
    ; - DELETE ITEM: linear-scan for the first author+title match, blank it.
    ;
    ; What's adapted rather than ported line-for-line:
    ; - The menu is still interactive (INPUT-driven, like CLERK.BAS's own
    ; INKEY$/ON CHOICE GOSUB loop), but each menu action (NEW ITEM, list,
    ; the two searches, DELETE ITEM) is its own `procedure` — addItem,
    ; listAll, searchByAuthor, searchByAuthorTitle, deleteItem — called
    ; from a `mainMenu` dispatch procedure using `select case`, instead of
    ; CLERK.BAS's numbered GOTO/GOSUB sections. This is the canonical
    ; BASCAL style (see the manual's Procedures section at
    ; https://johnjoeallen.github.io/bascal/manual/), and specifically
    ; exercises record/file access from inside a procedure body, not just
    ; top-level code.
    ; - CLERK.BAS's original also supported a multi-diskette/multi-file
    ; registry (drive letter + FILEDAT), search-by-subject, and HARD COPY
    ; (LPRINT) output. This example keeps one catalog file and the two
    ; named searches, and drops the rest, to stay focused on what the
    ; record/file DSL and procedures are actually demonstrating here.
    ; The header occupies slot 1 of the same file, sized to match Entry's
    ; width (20+20+20 = 60 bytes) so both record types agree on where every
    ; slot starts. size is the last valid entry slot number, mirroring
    ; CLERK.BAS's own S = CVI(F$) header field.
    ldc 11
    putstatic CardCatalog/g11 I
    ; 10 usable entry slots + 1 -- slot 1 is the header, 2..11 are entries
    ; file header as Header = open(...)  [60 bytes/record]
    getstatic CardCatalog/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    dup_x1
    new java/io/RandomAccessFile
    dup
    ldc "catalog.dat"
    ldc "rw"
    invokespecial java/io/RandomAccessFile/<init> (Ljava/lang/String;Ljava/lang/String;)V
    aastore
    getstatic CardCatalog/bccRecLen [I
    swap
    dup_x1
    ldc 60
    iastore
    getstatic CardCatalog/bccBufs [[B
    swap
    dup_x1
    ldc 60
    newarray byte
    aastore
    pop
    ; file catalog as Entry = open(...)  [60 bytes/record]
    getstatic CardCatalog/bccFiles [Ljava/io/RandomAccessFile;
    ldc 2
    iconst_1
    isub
    dup_x1
    new java/io/RandomAccessFile
    dup
    ldc "catalog.dat"
    ldc "rw"
    invokespecial java/io/RandomAccessFile/<init> (Ljava/lang/String;Ljava/lang/String;)V
    aastore
    getstatic CardCatalog/bccRecLen [I
    swap
    dup_x1
    ldc 60
    iastore
    getstatic CardCatalog/bccBufs [[B
    swap
    dup_x1
    ldc 60
    newarray byte
    aastore
    pop
    ; ---- CHOICE=5 in CLERK.BAS: create/reset the catalog file ----
    ; ---- CHOICE=1 NEW ITEM in CLERK.BAS ----
    ; author$  -- new entry's author
    ; title$   -- new entry's title
    ; subject$ -- new entry's subject
    ; ---- MENU=1 subroutine in CLERK.BAS: list every non-empty entry ----
    ; ---- MENU=2 subroutine in CLERK.BAS: filter by author ----
    ; author$ -- author name to match
    ; ---- MENU=3 subroutine in CLERK.BAS: filter by author AND title ----
    ; author$ -- author name to match
    ; title$  -- title to match
    ; ---- CHOICE=3 DELETE ITEM in CLERK.BAS: first author+title match ----
    ; author$ -- author name to match
    ; title$  -- title to match
    ; ---- CLERK.BAS's own MENU / ON CHOICE GOSUB dispatch loop ----
    ; --- Drive the catalog ---
    invokestatic CardCatalog/initCatalog ()V
    invokestatic CardCatalog/mainMenu ()V
    ; header.close()
    getstatic CardCatalog/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/close ()V
    ; catalog.close()
    getstatic CardCatalog/bccFiles [Ljava/io/RandomAccessFile;
    ldc 2
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/close ()V
    return
.end method
