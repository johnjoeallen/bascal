.version 50 0
.class public CardCatalog
.super java/lang/Object

.field public static bccFiles [Ljava/io/RandomAccessFile;
.field public static bccBufs [[B
.field public static bccRecLen [I
.field public static bccStdin Ljava/io/BufferedReader;
.method public static initCatalog : ()V
    .limit stack 16
    .limit locals 1

    iconst_0
    istore 0
    ; global header
    ; global catalog
    ; header[...] = { ... }  (whole-record write)
    ldc 2
    invokestatic java/nio/ByteBuffer/allocate (I)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    ldc 11
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
    istore 0
L_for_0_top:
    iload 0
    ldc 11
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
    iload 0
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
    iload 0
    ldc 1
    iadd
    istore 0
    goto L_for_0_top
L_for_0_end:
    return
.end method

.method public static addItem : (Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)V
    .limit stack 16
    .limit locals 14

    ldc ""
    astore 3
    iconst_0
    istore 4
    ldc ""
    astore 5
    iconst_0
    istore 6
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
    invokevirtual java/io/RandomAccessFile/readFully ([B)V
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
    invokevirtual java/lang/String/length ()I
    istore 10
L_while_0_top:
    iload 10
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_while_0_end
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
    iload 10
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
    iload 10
    ldc 1
    isub
    istore 10
    goto L_while_0_top
L_while_0_end:
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
    iconst_0
    iload 10
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    astore 9
    ldc 1
    istore 12
    ldc 0
    istore 13
L_do_1_top:
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
    ifeq L_do_1_end
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
    invokevirtual java/io/RandomAccessFile/readFully ([B)V
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
    invokevirtual java/lang/String/length ()I
    istore 4
L_while_2_top:
    iload 4
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_while_2_end
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
    ifeq L_while_2_end
    iload 4
    ldc 1
    isub
    istore 4
    goto L_while_2_top
L_while_2_end:
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
    iconst_0
    iload 4
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
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
    invokevirtual java/lang/String/length ()I
    istore 8
L_while_3_top:
    iload 8
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_while_3_end
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
    iload 8
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
    iload 8
    ldc 1
    isub
    istore 8
    goto L_while_3_top
L_while_3_end:
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
    iconst_0
    iload 8
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
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
    invokevirtual java/lang/String/length ()I
    istore 6
L_while_4_top:
    iload 6
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_while_4_end
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
    ifeq L_while_4_end
    iload 6
    ldc 1
    isub
    istore 6
    goto L_while_4_top
L_while_4_end:
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
    iconst_0
    iload 6
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    astore 5
    aload 3
    ldc ""
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ineg
    ifeq L_if_5_else
    ldc 1
    istore 13
L_if_5_else:
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
    ifeq L_if_6_else
    ldc 1
    istore 13
L_if_6_else:
    goto L_do_1_top
L_do_1_end:
    aload 3
    ldc ""
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ineg
    ifeq L_if_7_else
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
    goto L_if_7_end
L_if_7_else:
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
L_if_7_end:
    return
.end method

.method public static listAll : ()V
    .limit stack 16
    .limit locals 10

    ldc ""
    astore 0
    iconst_0
    istore 1
    ldc ""
    astore 2
    iconst_0
    istore 3
    ldc ""
    astore 4
    iconst_0
    istore 5
    ldc ""
    astore 6
    iconst_0
    istore 7
    iconst_0
    istore 8
    iconst_0
    istore 9
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
    invokevirtual java/io/RandomAccessFile/readFully ([B)V
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
    istore 8
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
    invokevirtual java/lang/String/length ()I
    istore 7
L_while_0_top:
    iload 7
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_while_0_end
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
    ifeq L_while_0_end
    iload 7
    ldc 1
    isub
    istore 7
    goto L_while_0_top
L_while_0_end:
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
    iconst_0
    iload 7
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    astore 6
    ldc 2
    istore 9
L_for_1_top:
    iload 9
    iload 8
    if_icmpgt L_for_1_end
    ; let e = catalog[...]  (whole-record read)
    getstatic CardCatalog/bccFiles [Ljava/io/RandomAccessFile;
    ldc 2
    iconst_1
    isub
    aaload
    dup
    iload 9
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
    invokevirtual java/io/RandomAccessFile/readFully ([B)V
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
    invokevirtual java/lang/String/length ()I
    istore 1
L_while_2_top:
    iload 1
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_while_2_end
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
    ifeq L_while_2_end
    iload 1
    ldc 1
    isub
    istore 1
    goto L_while_2_top
L_while_2_end:
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
    iconst_0
    iload 1
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    astore 0
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
    iconst_0
    iload 5
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    astore 4
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
    invokevirtual java/lang/String/length ()I
    istore 3
L_while_4_top:
    iload 3
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_while_4_end
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
    ifeq L_while_4_end
    iload 3
    ldc 1
    isub
    istore 3
    goto L_while_4_top
L_while_4_end:
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
    iconst_0
    iload 3
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    astore 2
    aload 0
    ldc ""
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    iconst_1
    ixor
    ineg
    ifeq L_if_5_else
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
    aload 0
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc "  |  "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    aload 4
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc "  |  "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    aload 2
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
L_if_5_else:
    iload 9
    ldc 1
    iadd
    istore 9
    goto L_for_1_top
L_for_1_end:
    return
.end method

.method public static searchByAuthor : (Ljava/lang/String;)V
    .limit stack 16
    .limit locals 11

    ldc ""
    astore 1
    iconst_0
    istore 2
    ldc ""
    astore 3
    iconst_0
    istore 4
    ldc ""
    astore 5
    iconst_0
    istore 6
    ldc ""
    astore 7
    iconst_0
    istore 8
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
    invokevirtual java/io/RandomAccessFile/readFully ([B)V
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
    invokevirtual java/lang/String/length ()I
    istore 8
L_while_0_top:
    iload 8
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_while_0_end
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
    iload 8
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
    iload 8
    ldc 1
    isub
    istore 8
    goto L_while_0_top
L_while_0_end:
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
    iconst_0
    iload 8
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    astore 7
    ldc 2
    istore 10
L_for_1_top:
    iload 10
    iload 9
    if_icmpgt L_for_1_end
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
    invokevirtual java/io/RandomAccessFile/readFully ([B)V
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
    iconst_0
    iload 2
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    astore 1
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
    invokevirtual java/lang/String/length ()I
    istore 6
L_while_3_top:
    iload 6
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_while_3_end
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
    ifeq L_while_3_end
    iload 6
    ldc 1
    isub
    istore 6
    goto L_while_3_top
L_while_3_end:
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
    iconst_0
    iload 6
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
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
    invokevirtual java/lang/String/length ()I
    istore 4
L_while_4_top:
    iload 4
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_while_4_end
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
    ifeq L_while_4_end
    iload 4
    ldc 1
    isub
    istore 4
    goto L_while_4_top
L_while_4_end:
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
    iconst_0
    iload 4
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    astore 3
    aload 1
    aload 0
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ineg
    ifeq L_if_5_else
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
    aload 1
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
    aload 3
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
L_if_5_else:
    iload 10
    ldc 1
    iadd
    istore 10
    goto L_for_1_top
L_for_1_end:
    return
.end method

.method public static searchByAuthorTitle : (Ljava/lang/String;Ljava/lang/String;)V
    .limit stack 16
    .limit locals 12

    ldc ""
    astore 2
    iconst_0
    istore 3
    ldc ""
    astore 4
    iconst_0
    istore 5
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
    invokevirtual java/io/RandomAccessFile/readFully ([B)V
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
    invokevirtual java/lang/String/length ()I
    istore 9
L_while_0_top:
    iload 9
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_while_0_end
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
    ifeq L_while_0_end
    iload 9
    ldc 1
    isub
    istore 9
    goto L_while_0_top
L_while_0_end:
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
    iconst_0
    iload 9
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    astore 8
    ldc 2
    istore 11
L_for_1_top:
    iload 11
    iload 10
    if_icmpgt L_for_1_end
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
    invokevirtual java/io/RandomAccessFile/readFully ([B)V
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
    iconst_0
    iload 3
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    astore 2
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
    invokevirtual java/lang/String/length ()I
    istore 7
L_while_3_top:
    iload 7
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_while_3_end
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
    ifeq L_while_3_end
    iload 7
    ldc 1
    isub
    istore 7
    goto L_while_3_top
L_while_3_end:
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
    iconst_0
    iload 7
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
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
    invokevirtual java/lang/String/length ()I
    istore 5
L_while_4_top:
    iload 5
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_while_4_end
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
    ifeq L_while_4_end
    iload 5
    ldc 1
    isub
    istore 5
    goto L_while_4_top
L_while_4_end:
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
    iconst_0
    iload 5
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    astore 4
    aload 2
    aload 0
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ineg
    ifeq L_if_5_else
    aload 6
    aload 1
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ineg
    ifeq L_if_5_else
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
    aload 2
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
    aload 4
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
L_if_5_else:
    iload 11
    ldc 1
    iadd
    istore 11
    goto L_for_1_top
L_for_1_end:
    return
.end method

.method public static deleteItem : (Ljava/lang/String;Ljava/lang/String;)V
    .limit stack 16
    .limit locals 13

    ldc ""
    astore 2
    iconst_0
    istore 3
    ldc ""
    astore 4
    iconst_0
    istore 5
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
    invokevirtual java/io/RandomAccessFile/readFully ([B)V
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
    invokevirtual java/lang/String/length ()I
    istore 9
L_while_0_top:
    iload 9
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_while_0_end
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
    ifeq L_while_0_end
    iload 9
    ldc 1
    isub
    istore 9
    goto L_while_0_top
L_while_0_end:
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
    iconst_0
    iload 9
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    astore 8
    ldc 1
    istore 11
    ldc 0
    istore 12
L_do_1_top:
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
    ifeq L_do_1_end
    iload 11
    ldc 1
    iadd
    istore 11
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
    invokevirtual java/io/RandomAccessFile/readFully ([B)V
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
    iconst_0
    iload 3
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    astore 2
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
    invokevirtual java/lang/String/length ()I
    istore 7
L_while_3_top:
    iload 7
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_while_3_end
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
    ifeq L_while_3_end
    iload 7
    ldc 1
    isub
    istore 7
    goto L_while_3_top
L_while_3_end:
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
    iconst_0
    iload 7
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
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
    invokevirtual java/lang/String/length ()I
    istore 5
L_while_4_top:
    iload 5
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_while_4_end
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
    ifeq L_while_4_end
    iload 5
    ldc 1
    isub
    istore 5
    goto L_while_4_top
L_while_4_end:
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
    iconst_0
    iload 5
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    astore 4
    aload 2
    aload 0
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ineg
    ifeq L_if_5_else
    aload 6
    aload 1
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ineg
    ifeq L_if_5_else
    ldc 1
    istore 12
L_if_5_else:
    iload 11
    iload 10
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
    ldc 1
    istore 12
L_if_6_else:
    goto L_do_1_top
L_do_1_end:
    aload 2
    aload 0
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ineg
    ifeq L_if_7_else
    aload 6
    aload 1
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ineg
    ifeq L_if_7_else
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
    aload 2
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
    invokevirtual java/io/RandomAccessFile/write ([B)V
    goto L_if_7_end
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
L_if_7_end:
    return
.end method

.method public static mainMenu : ()V
    .limit stack 16
    .limit locals 5

    ldc ""
    astore 0
    iconst_0
    istore 1
    iconst_0
    istore 2
    ldc ""
    astore 3
    ldc ""
    astore 4
    ldc 1
    istore 2
L_do_0_top:
    iload 2
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
    getstatic CardCatalog/bccStdin Ljava/io/BufferedReader;
    invokevirtual java/io/BufferedReader/readLine ()Ljava/lang/String;
    invokevirtual java/lang/String/trim ()Ljava/lang/String;
    invokestatic java/lang/Integer/parseInt (Ljava/lang/String;)I
    istore 1

    iload 1
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
    getstatic CardCatalog/bccStdin Ljava/io/BufferedReader;
    invokevirtual java/io/BufferedReader/readLine ()Ljava/lang/String;
    astore 0
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "TITLE   ? "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic CardCatalog/bccStdin Ljava/io/BufferedReader;
    invokevirtual java/io/BufferedReader/readLine ()Ljava/lang/String;
    astore 4
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "SUBJECT ? "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic CardCatalog/bccStdin Ljava/io/BufferedReader;
    invokevirtual java/io/BufferedReader/readLine ()Ljava/lang/String;
    astore 3
    aload 0
    aload 4
    aload 3
    invokestatic CardCatalog/addItem (Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)V
    goto L_select_1_end
L_select_1_case_2:
    pop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "AUTHOR ? "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
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
    getstatic CardCatalog/bccStdin Ljava/io/BufferedReader;
    invokevirtual java/io/BufferedReader/readLine ()Ljava/lang/String;
    astore 0
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "TITLE  ? "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic CardCatalog/bccStdin Ljava/io/BufferedReader;
    invokevirtual java/io/BufferedReader/readLine ()Ljava/lang/String;
    astore 4
    aload 0
    aload 4
    invokestatic CardCatalog/searchByAuthorTitle (Ljava/lang/String;Ljava/lang/String;)V
    goto L_select_1_end
L_select_1_case_4:
    pop
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "AUTHOR (to delete) ? "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic CardCatalog/bccStdin Ljava/io/BufferedReader;
    invokevirtual java/io/BufferedReader/readLine ()Ljava/lang/String;
    astore 0
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "TITLE  (to delete) ? "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic CardCatalog/bccStdin Ljava/io/BufferedReader;
    invokevirtual java/io/BufferedReader/readLine ()Ljava/lang/String;
    astore 4
    aload 0
    aload 4
    invokestatic CardCatalog/deleteItem (Ljava/lang/String;Ljava/lang/String;)V
    goto L_select_1_end
L_select_1_case_5:
    pop
    ldc 0
    istore 2
    goto L_select_1_end
L_select_1_end:
    goto L_do_0_top
L_do_0_end:
    return
.end method

.method public static main : ([Ljava/lang/String;)V
    .limit stack 16
    .limit locals 1

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


    ; file header as Header = open(...)  [60 bytes/record]
    getstatic CardCatalog/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    new java/io/RandomAccessFile
    dup
    ldc "catalog.dat"
    ldc "rw"
    invokespecial java/io/RandomAccessFile/<init> (Ljava/lang/String;Ljava/lang/String;)V
    aastore
    getstatic CardCatalog/bccRecLen [I
    ldc 1
    iconst_1
    isub
    ldc 60
    iastore
    getstatic CardCatalog/bccBufs [[B
    ldc 1
    iconst_1
    isub
    ldc 60
    newarray byte
    aastore
    ; file catalog as Entry = open(...)  [60 bytes/record]
    getstatic CardCatalog/bccFiles [Ljava/io/RandomAccessFile;
    ldc 2
    iconst_1
    isub
    new java/io/RandomAccessFile
    dup
    ldc "catalog.dat"
    ldc "rw"
    invokespecial java/io/RandomAccessFile/<init> (Ljava/lang/String;Ljava/lang/String;)V
    aastore
    getstatic CardCatalog/bccRecLen [I
    ldc 2
    iconst_1
    isub
    ldc 60
    iastore
    getstatic CardCatalog/bccBufs [[B
    ldc 2
    iconst_1
    isub
    ldc 60
    newarray byte
    aastore

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
