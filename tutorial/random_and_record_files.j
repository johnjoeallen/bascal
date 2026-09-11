.version 50 0
.class public RandomAndRecordFiles
.super java/lang/Object

.field public static g1 Ljava/lang/String;
.field public static g2 I
.field public static g3 I
.field public static g4 Ljava/lang/String;
.field public static g5 I
.field public static g6 D
.field public static g8 I
.field public static g9 I
.field public static g10 D
.field public static g12 Ljava/lang/String;
.field public static g13 I
.field public static g14 I
.field public static g15 Ljava/lang/String;
.field public static g16 I
.field public static g17 D
.field public static bccFiles [Ljava/io/RandomAccessFile;
.field public static bccBufs [[B
.field public static bccRecLen [I
.method public static trimmed : (Ljava/lang/String;)Ljava/lang/String;
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
    .limit locals 19

    ldc ""
    putstatic RandomAndRecordFiles/g1 Ljava/lang/String;
    iconst_0
    putstatic RandomAndRecordFiles/g2 I
    iconst_0
    putstatic RandomAndRecordFiles/g3 I
    ldc ""
    putstatic RandomAndRecordFiles/g4 Ljava/lang/String;
    iconst_0
    putstatic RandomAndRecordFiles/g5 I
    dconst_0
    putstatic RandomAndRecordFiles/g6 D
    iconst_0
    putstatic RandomAndRecordFiles/g8 I
    iconst_0
    putstatic RandomAndRecordFiles/g9 I
    dconst_0
    putstatic RandomAndRecordFiles/g10 D
    ldc ""
    putstatic RandomAndRecordFiles/g12 Ljava/lang/String;
    iconst_0
    putstatic RandomAndRecordFiles/g13 I
    iconst_0
    putstatic RandomAndRecordFiles/g14 I
    ldc ""
    putstatic RandomAndRecordFiles/g15 Ljava/lang/String;
    iconst_0
    putstatic RandomAndRecordFiles/g16 I
    dconst_0
    putstatic RandomAndRecordFiles/g17 D
    ldc 16
    anewarray java/io/RandomAccessFile
    putstatic RandomAndRecordFiles/bccFiles [Ljava/io/RandomAccessFile;
    ldc 16
    anewarray [B
    putstatic RandomAndRecordFiles/bccBufs [[B
    ldc 16
    newarray int
    putstatic RandomAndRecordFiles/bccRecLen [I
    ; Tutorial — Random-Access Files: hand-written, then with the record/file DSL
    ; 
    ; This tutorial writes the *same* program twice. Part 1 uses BASIC's raw
    ; random-access file primitives directly. Part 2 uses BASCAL's `record`/
    ; `file` DSL, which transpiles to exactly the same primitives — nothing about
    ; the *generated* BASIC changes, only how much of it you have to write by
    ; hand. Read Part 1 first; the comments between the two parts explain what
    ; the DSL is buying you and why.
    ; 
    ; ---- Part 1 primitives ----
    ; 
    ; open filename$ for random as #n len = REC_LEN
    ; Open (or create) a random-access file.  len specifies the record length
    ; in bytes; every record occupies exactly that many bytes.
    ; 
    ; field #n, width1% as var1$, width2% as var2$, ...
    ; Bind string variables to regions of the file buffer.  The sum of all
    ; widths must equal the record length.  Only string variables may be used
    ; in a FIELD statement.
    ; 
    ; lset var$ = expr$   — copy into a field buffer, left-justified (padded)
    ; rset var$ = expr$   — copy into a field buffer, right-justified (padded)
    ; 
    ; put #n, recordNumber%   — write the current buffer as record n (1-based)
    ; get #n, recordNumber%   — read record n into the buffer variables
    ; 
    ; Packing helpers (BASIC builtins):
    ; mki$(n%)  — pack a 2-byte integer into a 2-character string
    ; mkl$(n&)  — pack a 4-byte long
    ; mks$(n!)  — pack a 4-byte single
    ; mkd$(n#)  — pack an 8-byte double
    ; cvi(s$)   — unpack a 2-byte integer from a string
    ; cvl(s$)   — unpack a 4-byte long
    ; cvs(s$)   — unpack a 4-byte single
    ; cvd(s$)   — unpack an 8-byte double
    ; 
    ; Every MKx$ always returns a string (never a type-suffixed MKI%/MKD#/etc —
    ; those aren't real MBASIC/BASCOM functions), and every CVx takes no suffix
    ; at all. There's also no RTRIM$ builtin on real MBASIC/BASCOM -- trimming a
    ; fixed-width, space-padded FIELD buffer back down to its real length needs
    ; a hand-rolled loop, like trimmed$ below.

    ; trimmed$ -- right-trim trailing spaces from a fixed-width FIELD buffer.


    ; ============================================================
    ; Part 1 — random-access files, written by hand
    ; ============================================================

    ; ---- Write three records ----

    getstatic RandomAndRecordFiles/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    new java/io/RandomAccessFile
    dup
    ldc "tutorial_students.dat"
    ldc "rw"
    invokespecial java/io/RandomAccessFile/<init> (Ljava/lang/String;Ljava/lang/String;)V
    aastore
    getstatic RandomAndRecordFiles/bccRecLen [I
    ldc 1
    iconst_1
    isub
    ldc 50
    iastore
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 1
    iconst_1
    isub
    ldc 50
    newarray byte
    aastore

    ; Record 1: Alice, 95
    ldc 2
    invokestatic java/nio/ByteBuffer/allocate (I)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    ldc 1
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
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    ldc 0
    ldc 2
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    ldc "Alice"
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
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    ldc 2
    ldc 20
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    ldc 8
    invokestatic java/nio/ByteBuffer/allocate (I)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    ldc2_w 95.0
    invokevirtual java/nio/ByteBuffer/putDouble (D)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/array ()[B
    new java/lang/String
    dup_x1
    swap
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BLjava/nio/charset/Charset;)V
    ldc 8
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
    ldc 8
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    iconst_0
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    ldc 22
    ldc 8
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    ldc "Engineering"
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
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    ldc 30
    ldc 20
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    getstatic RandomAndRecordFiles/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    dup
    ldc 1
    i2l
    lconst_1
    lsub
    getstatic RandomAndRecordFiles/bccRecLen [I
    ldc 1
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/write ([B)V

    ; Record 2: Bob, 54
    ldc 2
    invokestatic java/nio/ByteBuffer/allocate (I)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    ldc 2
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
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    ldc 0
    ldc 2
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    ldc "Bob"
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
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    ldc 2
    ldc 20
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    ldc 8
    invokestatic java/nio/ByteBuffer/allocate (I)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    ldc2_w 54.0
    invokevirtual java/nio/ByteBuffer/putDouble (D)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/array ()[B
    new java/lang/String
    dup_x1
    swap
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BLjava/nio/charset/Charset;)V
    ldc 8
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
    ldc 8
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    iconst_0
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    ldc 22
    ldc 8
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    ldc "Arts"
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
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    ldc 30
    ldc 20
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    getstatic RandomAndRecordFiles/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    dup
    ldc 2
    i2l
    lconst_1
    lsub
    getstatic RandomAndRecordFiles/bccRecLen [I
    ldc 1
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/write ([B)V

    ; Record 3: Carol, 78
    ldc 2
    invokestatic java/nio/ByteBuffer/allocate (I)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    ldc 3
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
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    ldc 0
    ldc 2
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    ldc "Carol"
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
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    ldc 2
    ldc 20
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    ldc 8
    invokestatic java/nio/ByteBuffer/allocate (I)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    ldc2_w 78.0
    invokevirtual java/nio/ByteBuffer/putDouble (D)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/array ()[B
    new java/lang/String
    dup_x1
    swap
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BLjava/nio/charset/Charset;)V
    ldc 8
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
    ldc 8
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    iconst_0
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    ldc 22
    ldc 8
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    ldc "Science"
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
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    ldc 30
    ldc 20
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    getstatic RandomAndRecordFiles/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    dup
    ldc 3
    i2l
    lconst_1
    lsub
    getstatic RandomAndRecordFiles/bccRecLen [I
    ldc 1
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/write ([B)V

    getstatic RandomAndRecordFiles/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/close ()V

    ; ---- Read records in reverse order ----

    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Part 1 (hand-written) -- reading records in reverse order:"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic RandomAndRecordFiles/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    new java/io/RandomAccessFile
    dup
    ldc "tutorial_students.dat"
    ldc "rw"
    invokespecial java/io/RandomAccessFile/<init> (Ljava/lang/String;Ljava/lang/String;)V
    aastore
    getstatic RandomAndRecordFiles/bccRecLen [I
    ldc 1
    iconst_1
    isub
    ldc 50
    iastore
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 1
    iconst_1
    isub
    ldc 50
    newarray byte
    aastore

    ldc 3
    putstatic RandomAndRecordFiles/g8 I
L_for_0_top:
    getstatic RandomAndRecordFiles/g8 I
    ldc 1
    if_icmplt L_for_0_end
    getstatic RandomAndRecordFiles/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    dup
    getstatic RandomAndRecordFiles/g8 I
    i2l
    lconst_1
    lsub
    getstatic RandomAndRecordFiles/bccRecLen [I
    ldc 1
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/read ([B)I
    pop
    getstatic RandomAndRecordFiles/bccBufs [[B
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
    putstatic RandomAndRecordFiles/g9 I
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 22
    ldc 8
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    invokestatic java/nio/ByteBuffer/wrap ([B)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/getDouble ()D
    putstatic RandomAndRecordFiles/g10 D
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
    ldc "  ["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    getstatic RandomAndRecordFiles/g9 I
    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc "] "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 2
    ldc 20
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    invokestatic RandomAndRecordFiles/trimmed (Ljava/lang/String;)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc " -- "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    getstatic RandomAndRecordFiles/g10 D
    invokestatic RandomAndRecordFiles/bccStr (D)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic RandomAndRecordFiles/g8 I
    ldc 1
    ineg
    iadd
    putstatic RandomAndRecordFiles/g8 I
    goto L_for_0_top
L_for_0_end:

    getstatic RandomAndRecordFiles/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/close ()V

    ; ---- Update one field in place ----

    getstatic RandomAndRecordFiles/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    new java/io/RandomAccessFile
    dup
    ldc "tutorial_students.dat"
    ldc "rw"
    invokespecial java/io/RandomAccessFile/<init> (Ljava/lang/String;Ljava/lang/String;)V
    aastore
    getstatic RandomAndRecordFiles/bccRecLen [I
    ldc 1
    iconst_1
    isub
    ldc 50
    iastore
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 1
    iconst_1
    isub
    ldc 50
    newarray byte
    aastore

    ; Bob just scraped a pass on re-mark. Only scoreBuf$ changes, but PUT
    ; always writes the whole 50-byte buffer, so GET has to load the record
    ; first even though idBuf$/nameBuf$/facultyBuf$ are just being written straight back
    ; unchanged.
    getstatic RandomAndRecordFiles/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    dup
    ldc 2
    i2l
    lconst_1
    lsub
    getstatic RandomAndRecordFiles/bccRecLen [I
    ldc 1
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/read ([B)I
    pop
    ldc 8
    invokestatic java/nio/ByteBuffer/allocate (I)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    ldc2_w 61.5
    invokevirtual java/nio/ByteBuffer/putDouble (D)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/array ()[B
    new java/lang/String
    dup_x1
    swap
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BLjava/nio/charset/Charset;)V
    ldc 8
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
    ldc 8
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    iconst_0
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    ldc 22
    ldc 8
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    getstatic RandomAndRecordFiles/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    dup
    ldc 2
    i2l
    lconst_1
    lsub
    getstatic RandomAndRecordFiles/bccRecLen [I
    ldc 1
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/write ([B)V

    getstatic RandomAndRecordFiles/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/close ()V

    ; ---- Update two fields at once ----

    getstatic RandomAndRecordFiles/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    new java/io/RandomAccessFile
    dup
    ldc "tutorial_students.dat"
    ldc "rw"
    invokespecial java/io/RandomAccessFile/<init> (Ljava/lang/String;Ljava/lang/String;)V
    aastore
    getstatic RandomAndRecordFiles/bccRecLen [I
    ldc 1
    iconst_1
    isub
    ldc 50
    iastore
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 1
    iconst_1
    isub
    ldc 50
    newarray byte
    aastore

    ; Alice got married and re-sat the exam — `name` and `score` both change,
    ; `id` and `faculty` don't. Same problem as Bob's update, just with two fields instead
    ; of one: GET first (this is what preserves idBuf$ and facultyBuf$), LSET the two fields
    ; that actually changed, then PUT the whole buffer back. Nothing here is
    ; specific to "two" fields — five changed fields would look identical,
    ; just with five LSET lines between the GET and the PUT.
    getstatic RandomAndRecordFiles/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    dup
    ldc 1
    i2l
    lconst_1
    lsub
    getstatic RandomAndRecordFiles/bccRecLen [I
    ldc 1
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/read ([B)I
    pop
    ldc "Alice Smith"
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
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    ldc 2
    ldc 20
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    ldc 8
    invokestatic java/nio/ByteBuffer/allocate (I)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    ldc2_w 91.0
    invokevirtual java/nio/ByteBuffer/putDouble (D)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/array ()[B
    new java/lang/String
    dup_x1
    swap
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BLjava/nio/charset/Charset;)V
    ldc 8
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
    ldc 8
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    iconst_0
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    ldc 22
    ldc 8
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    getstatic RandomAndRecordFiles/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    dup
    ldc 1
    i2l
    lconst_1
    lsub
    getstatic RandomAndRecordFiles/bccRecLen [I
    ldc 1
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/write ([B)V

    getstatic RandomAndRecordFiles/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/close ()V

    ; ---- Same shape again ----

    getstatic RandomAndRecordFiles/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    new java/io/RandomAccessFile
    dup
    ldc "tutorial_students.dat"
    ldc "rw"
    invokespecial java/io/RandomAccessFile/<init> (Ljava/lang/String;Ljava/lang/String;)V
    aastore
    getstatic RandomAndRecordFiles/bccRecLen [I
    ldc 1
    iconst_1
    isub
    ldc 50
    iastore
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 1
    iconst_1
    isub
    ldc 50
    newarray byte
    aastore

    ; Carol changed her name and improved her score: the exact same
    ; GET / LSET / LSET / PUT shape as Alice's update above, just retyped by
    ; hand with Carol's record number and values.
    getstatic RandomAndRecordFiles/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    dup
    ldc 3
    i2l
    lconst_1
    lsub
    getstatic RandomAndRecordFiles/bccRecLen [I
    ldc 1
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/read ([B)I
    pop
    ldc "Carol Jones"
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
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    ldc 2
    ldc 20
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    ldc 8
    invokestatic java/nio/ByteBuffer/allocate (I)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    ldc2_w 88.0
    invokevirtual java/nio/ByteBuffer/putDouble (D)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/array ()[B
    new java/lang/String
    dup_x1
    swap
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BLjava/nio/charset/Charset;)V
    ldc 8
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
    ldc 8
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    iconst_0
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    ldc 22
    ldc 8
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    getstatic RandomAndRecordFiles/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    dup
    ldc 3
    i2l
    lconst_1
    lsub
    getstatic RandomAndRecordFiles/bccRecLen [I
    ldc 1
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/write ([B)V

    getstatic RandomAndRecordFiles/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/close ()V

    ; ---- Verify the updates ----

    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Part 1 (hand-written) -- after updates:"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic RandomAndRecordFiles/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    new java/io/RandomAccessFile
    dup
    ldc "tutorial_students.dat"
    ldc "rw"
    invokespecial java/io/RandomAccessFile/<init> (Ljava/lang/String;Ljava/lang/String;)V
    aastore
    getstatic RandomAndRecordFiles/bccRecLen [I
    ldc 1
    iconst_1
    isub
    ldc 50
    iastore
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 1
    iconst_1
    isub
    ldc 50
    newarray byte
    aastore

    ldc 1
    putstatic RandomAndRecordFiles/g8 I
L_for_1_top:
    getstatic RandomAndRecordFiles/g8 I
    ldc 3
    if_icmpgt L_for_1_end
    getstatic RandomAndRecordFiles/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    dup
    getstatic RandomAndRecordFiles/g8 I
    i2l
    lconst_1
    lsub
    getstatic RandomAndRecordFiles/bccRecLen [I
    ldc 1
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/read ([B)I
    pop
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
    ldc "  "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 2
    ldc 20
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    invokestatic RandomAndRecordFiles/trimmed (Ljava/lang/String;)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc ": "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 22
    ldc 8
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    invokestatic java/nio/ByteBuffer/wrap ([B)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/getDouble ()D
    invokestatic RandomAndRecordFiles/bccStr (D)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic RandomAndRecordFiles/g8 I
    ldc 1
    iadd
    putstatic RandomAndRecordFiles/g8 I
    goto L_for_1_top
L_for_1_end:

    getstatic RandomAndRecordFiles/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/close ()V

    ; ------------------------------------------------------------------------
    ; What Part 1 actually cost:
    ; 
    ; - idBuf$/nameBuf$/scoreBuf$ and the FIELD statement binding them had to
    ; be repeated, identically, in every OPEN block — get it wrong in one
    ; of the five and you're reading or writing the wrong bytes.
    ; - REC_LEN (50) is 2+20+8+20 computed by hand; add a field to the record
    ; and every one of those numbers has to be updated together, or the
    ; file silently gets corrupted.
    ; - Each field's pack/unpack call (mki$/cvi, mkd$/cvd, or nothing for
    ; strings) has to be matched to that field's type by hand, every time
    ; it's touched — nothing stops mkd$() being used on the id field.
    ; - There's no RTRIM$ builtin on real MBASIC/BASCOM, so reading a string
    ; field back means hand-rolling a trim loop (trimmed$, above) and
    ; remembering to call it, every time.
    ; - Alice's and Carol's updates are the identical GET/LSET/LSET/PUT
    ; pattern, typed out twice, with every buffer/field name repeated.
    ; 
    ; None of this is hard, exactly — it's just bookkeeping a compiler should
    ; be doing for you. Part 2 is the same program again, with BASCAL's
    ; record/file DSL doing that bookkeeping.
    ; ------------------------------------------------------------------------

    ; ============================================================
    ; Part 2 — the same program with the record / file DSL
    ; ============================================================
    ; 
    ; record <Name> ... end record
    ; Declares a fixed-layout record type. Supported field types: int16,
    ; int32, float32, float64, and string(N). The record's total byte width
    ; (used as Part 1's REC_LEN) is the sum of its field widths, computed
    ; automatically.
    ; 
    ; file <var> as <RecordType> = open(<path>)
    ; Opens (or creates) a random-access file sized for one record, and binds
    ; FIELD buffer variables for every field. File numbers are allocated
    ; automatically, starting at #1, in the order `file` declarations appear.
    ; This one line replaces Part 1's REC_LEN constant, OPEN, and FIELD.
    ; 
    ; <file>[<n>] = { field: value, ... }
    ; Whole-record write: packs every field (LSET, MKx$ for numeric fields)
    ; and writes record n. Every declared field must be given — a missing one
    ; is a compile-time error.
    ; 
    ; let <var> = <file>[<n>]
    ; Whole-record read: reads record n and unpacks every field (CVx for
    ; numeric fields, an inline trim loop like Part 1's trimmed$ for strings)
    ; into `<var>.<field>`.
    ; 
    ; <file>[<n>].<field> = value
    ; Partial update: GET, LSET just that one field, PUT. The one-field
    ; version of Part 1's Bob update, with no buffer names to get wrong.
    ; 
    ; <file>[<n>] = ?{ field: value, ... }
    ; Partial-record write: any subset of fields; unlisted ones are left
    ; untouched on disk. Whether a GET is needed is decided at *compile
    ; time* by comparing the given field names against the record's declared
    ; fields: some fields missing -> GET first, LSET just those fields, then
    ; PUT (this is Alice's update from Part 1, minus the GET/LSET/LSET/PUT
    ; spelled out by hand); every field given anyway -> no GET, same as a
    ; plain `{...}`. Unlike `{...}`, an *unknown* field name is still a
    ; compile-time error — only *missing* fields are allowed, not misspelled
    ; ones.
    ; 
    ; let <var> = <file>[<n>]
    ; <var>.<field> = value  (any number of times)
    ; <file>[<n>] = <var>
    ; Batched update: the `let` does one GET; each `<var>.<field> = value` is
    ; a pure in-memory assignment (no I/O); the final `<file>[<n>] = <var>`
    ; packs every field from `<var>` and does one PUT. This is Carol's update
    ; from Part 1 — same GET/LSET/LSET/PUT shape as `?{...}`, just spelled as
    ; read-mutate-write instead of a single literal, useful when the new
    ; values come from more than a one-line expression.
    ; 
    ; for <var> = <A> downto <B> ... end for
    ; Sugar for `for <var> = <A> to <B> step -1`.
    ; 
    ; <file>.close()
    ; Closes the file.


    ; file db as Student = open(...)  [50 bytes/record]
    getstatic RandomAndRecordFiles/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    new java/io/RandomAccessFile
    dup
    ldc "tutorial_records.dat"
    ldc "rw"
    invokespecial java/io/RandomAccessFile/<init> (Ljava/lang/String;Ljava/lang/String;)V
    aastore
    getstatic RandomAndRecordFiles/bccRecLen [I
    ldc 1
    iconst_1
    isub
    ldc 50
    iastore
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 1
    iconst_1
    isub
    ldc 50
    newarray byte
    aastore

    ; ---- Write three records ----

    ; Record 1: Alice, 95
    ; db[...] = { ... }  (whole-record write)
    ldc 2
    invokestatic java/nio/ByteBuffer/allocate (I)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    ldc 1
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
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    ldc 0
    ldc 2
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    ldc "Alice"
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
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    ldc 2
    ldc 20
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    ldc 8
    invokestatic java/nio/ByteBuffer/allocate (I)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    ldc2_w 95.0
    invokevirtual java/nio/ByteBuffer/putDouble (D)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/array ()[B
    new java/lang/String
    dup_x1
    swap
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BLjava/nio/charset/Charset;)V
    ldc 8
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
    ldc 8
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    iconst_0
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    ldc 22
    ldc 8
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    ldc "Engineering"
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
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    ldc 30
    ldc 20
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    getstatic RandomAndRecordFiles/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    dup
    ldc 1
    i2l
    lconst_1
    lsub
    getstatic RandomAndRecordFiles/bccRecLen [I
    ldc 1
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/write ([B)V

    ; Record 2: Bob, 54
    ; db[...] = { ... }  (whole-record write)
    ldc 2
    invokestatic java/nio/ByteBuffer/allocate (I)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    ldc 2
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
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    ldc 0
    ldc 2
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    ldc "Bob"
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
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    ldc 2
    ldc 20
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    ldc 8
    invokestatic java/nio/ByteBuffer/allocate (I)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    ldc2_w 54.0
    invokevirtual java/nio/ByteBuffer/putDouble (D)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/array ()[B
    new java/lang/String
    dup_x1
    swap
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BLjava/nio/charset/Charset;)V
    ldc 8
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
    ldc 8
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    iconst_0
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    ldc 22
    ldc 8
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    ldc "Arts"
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
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    ldc 30
    ldc 20
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    getstatic RandomAndRecordFiles/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    dup
    ldc 2
    i2l
    lconst_1
    lsub
    getstatic RandomAndRecordFiles/bccRecLen [I
    ldc 1
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/write ([B)V

    ; Record 3: Carol, 78
    ; db[...] = { ... }  (whole-record write)
    ldc 2
    invokestatic java/nio/ByteBuffer/allocate (I)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    ldc 3
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
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    ldc 0
    ldc 2
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    ldc "Carol"
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
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    ldc 2
    ldc 20
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    ldc 8
    invokestatic java/nio/ByteBuffer/allocate (I)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    ldc2_w 78.0
    invokevirtual java/nio/ByteBuffer/putDouble (D)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/array ()[B
    new java/lang/String
    dup_x1
    swap
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BLjava/nio/charset/Charset;)V
    ldc 8
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
    ldc 8
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    iconst_0
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    ldc 22
    ldc 8
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    ldc "Science"
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
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    ldc 30
    ldc 20
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    getstatic RandomAndRecordFiles/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    dup
    ldc 3
    i2l
    lconst_1
    lsub
    getstatic RandomAndRecordFiles/bccRecLen [I
    ldc 1
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/write ([B)V

    ; ---- Read records in reverse order ----

    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Part 2 (record/file DSL) -- reading records in reverse order:"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V

    ldc 3
    putstatic RandomAndRecordFiles/g8 I
L_for_2_top:
    getstatic RandomAndRecordFiles/g8 I
    ldc 1
    if_icmplt L_for_2_end
    ; let s = db[...]  (whole-record read)
    getstatic RandomAndRecordFiles/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    dup
    getstatic RandomAndRecordFiles/g8 I
    i2l
    lconst_1
    lsub
    getstatic RandomAndRecordFiles/bccRecLen [I
    ldc 1
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/read ([B)I
    pop
    getstatic RandomAndRecordFiles/bccBufs [[B
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
    putstatic RandomAndRecordFiles/g14 I
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 2
    ldc 20
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    invokevirtual java/lang/String/length ()I
    putstatic RandomAndRecordFiles/g16 I
L_while_3_top:
    getstatic RandomAndRecordFiles/g16 I
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_while_3_end
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 2
    ldc 20
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    getstatic RandomAndRecordFiles/g16 I
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
    getstatic RandomAndRecordFiles/g16 I
    ldc 1
    isub
    putstatic RandomAndRecordFiles/g16 I
    goto L_while_3_top
L_while_3_end:
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 2
    ldc 20
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    iconst_0
    getstatic RandomAndRecordFiles/g16 I
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    putstatic RandomAndRecordFiles/g15 Ljava/lang/String;
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 22
    ldc 8
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    invokestatic java/nio/ByteBuffer/wrap ([B)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/getDouble ()D
    putstatic RandomAndRecordFiles/g17 D
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 30
    ldc 20
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    invokevirtual java/lang/String/length ()I
    putstatic RandomAndRecordFiles/g13 I
L_while_4_top:
    getstatic RandomAndRecordFiles/g13 I
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_while_4_end
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 30
    ldc 20
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    getstatic RandomAndRecordFiles/g13 I
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
    getstatic RandomAndRecordFiles/g13 I
    ldc 1
    isub
    putstatic RandomAndRecordFiles/g13 I
    goto L_while_4_top
L_while_4_end:
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 30
    ldc 20
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    iconst_0
    getstatic RandomAndRecordFiles/g13 I
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    putstatic RandomAndRecordFiles/g12 Ljava/lang/String;
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
    ldc "  ["
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    getstatic RandomAndRecordFiles/g14 I
    invokestatic java/lang/String/valueOf (I)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc "] "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    getstatic RandomAndRecordFiles/g15 Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc " -- "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    getstatic RandomAndRecordFiles/g17 D
    invokestatic RandomAndRecordFiles/bccStr (D)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic RandomAndRecordFiles/g8 I
    ldc -1
    iadd
    putstatic RandomAndRecordFiles/g8 I
    goto L_for_2_top
L_for_2_end:

    ; ---- Update one field in place ----

    ; Bob just scraped a pass on re-mark. Compare to Part 1: no REC_LEN, no
    ; idBuf$/nameBuf$/scoreBuf$/facultyBuf$, no mkd$() — just the field that's changing.
    ; db[...].score = ...  (partial-field update)
    getstatic RandomAndRecordFiles/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    dup
    ldc 2
    i2l
    lconst_1
    lsub
    getstatic RandomAndRecordFiles/bccRecLen [I
    ldc 1
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/read ([B)I
    pop
    ldc 8
    invokestatic java/nio/ByteBuffer/allocate (I)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    ldc2_w 61.5
    invokevirtual java/nio/ByteBuffer/putDouble (D)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/array ()[B
    new java/lang/String
    dup_x1
    swap
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BLjava/nio/charset/Charset;)V
    ldc 8
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
    ldc 8
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    iconst_0
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    ldc 22
    ldc 8
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    getstatic RandomAndRecordFiles/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    dup
    ldc 2
    i2l
    lconst_1
    lsub
    getstatic RandomAndRecordFiles/bccRecLen [I
    ldc 1
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/write ([B)V

    ; ---- Update two fields at once, still one GET and one PUT ----

    ; Alice got married and re-sat the exam. `name` and `score` don't cover
    ; every field of Student, so this needs an implicit GET first (id and
    ; faculty are preserved from the existing record) -- exactly Part 1's GET / LSET /
    ; LSET / PUT for Alice, minus having to write out the GET, the buffer
    ; names, or the packing calls. Which fields need a GET is worked out by
    ; the compiler by comparing `name`/`score` against Student's declared
    ; fields — not decided at runtime.
    ; db[...] = ?{ ... }  (partial-record write)
    getstatic RandomAndRecordFiles/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    dup
    ldc 1
    i2l
    lconst_1
    lsub
    getstatic RandomAndRecordFiles/bccRecLen [I
    ldc 1
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/read ([B)I
    pop
    ldc "Alice Smith"
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
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    ldc 2
    ldc 20
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    ldc 8
    invokestatic java/nio/ByteBuffer/allocate (I)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    ldc2_w 91.0
    invokevirtual java/nio/ByteBuffer/putDouble (D)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/array ()[B
    new java/lang/String
    dup_x1
    swap
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BLjava/nio/charset/Charset;)V
    ldc 8
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
    ldc 8
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    iconst_0
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    ldc 22
    ldc 8
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    getstatic RandomAndRecordFiles/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    dup
    ldc 1
    i2l
    lconst_1
    lsub
    getstatic RandomAndRecordFiles/bccRecLen [I
    ldc 1
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/write ([B)V

    ; ---- Batched update: read once, mutate twice, write back once ----

    ; Carol changed her name and improved her score — the read-mutate-write
    ; spelling of the same one-GET-one-PUT update, useful when the new values
    ; aren't just a couple of literals.
    ; let carol = db[...]  (whole-record read)
    getstatic RandomAndRecordFiles/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    dup
    ldc 3
    i2l
    lconst_1
    lsub
    getstatic RandomAndRecordFiles/bccRecLen [I
    ldc 1
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/read ([B)I
    pop
    getstatic RandomAndRecordFiles/bccBufs [[B
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
    putstatic RandomAndRecordFiles/g3 I
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 2
    ldc 20
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    invokevirtual java/lang/String/length ()I
    putstatic RandomAndRecordFiles/g5 I
L_while_5_top:
    getstatic RandomAndRecordFiles/g5 I
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_while_5_end
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 2
    ldc 20
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    getstatic RandomAndRecordFiles/g5 I
    iconst_1
    isub
    dup
    ldc 1
    iadd
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    ldc " "
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ineg
    ifeq L_while_5_end
    getstatic RandomAndRecordFiles/g5 I
    ldc 1
    isub
    putstatic RandomAndRecordFiles/g5 I
    goto L_while_5_top
L_while_5_end:
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 2
    ldc 20
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    iconst_0
    getstatic RandomAndRecordFiles/g5 I
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    putstatic RandomAndRecordFiles/g4 Ljava/lang/String;
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 22
    ldc 8
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    invokestatic java/nio/ByteBuffer/wrap ([B)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/getDouble ()D
    putstatic RandomAndRecordFiles/g6 D
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 30
    ldc 20
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    invokevirtual java/lang/String/length ()I
    putstatic RandomAndRecordFiles/g2 I
L_while_6_top:
    getstatic RandomAndRecordFiles/g2 I
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_while_6_end
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 30
    ldc 20
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    getstatic RandomAndRecordFiles/g2 I
    iconst_1
    isub
    dup
    ldc 1
    iadd
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    ldc " "
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ineg
    ifeq L_while_6_end
    getstatic RandomAndRecordFiles/g2 I
    ldc 1
    isub
    putstatic RandomAndRecordFiles/g2 I
    goto L_while_6_top
L_while_6_end:
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 30
    ldc 20
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    iconst_0
    getstatic RandomAndRecordFiles/g2 I
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    putstatic RandomAndRecordFiles/g1 Ljava/lang/String;
    ldc "Carol Jones"
    putstatic RandomAndRecordFiles/g4 Ljava/lang/String;
    ldc2_w 88.0
    putstatic RandomAndRecordFiles/g6 D
    ; db[...] = carol  (write back a let-bound record)
    ldc 2
    invokestatic java/nio/ByteBuffer/allocate (I)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    getstatic RandomAndRecordFiles/g3 I
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
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    ldc 0
    ldc 2
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    getstatic RandomAndRecordFiles/g4 Ljava/lang/String;
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
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    ldc 2
    ldc 20
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    ldc 8
    invokestatic java/nio/ByteBuffer/allocate (I)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    getstatic RandomAndRecordFiles/g6 D
    invokevirtual java/nio/ByteBuffer/putDouble (D)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/array ()[B
    new java/lang/String
    dup_x1
    swap
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BLjava/nio/charset/Charset;)V
    ldc 8
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
    ldc 8
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    iconst_0
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    ldc 22
    ldc 8
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    getstatic RandomAndRecordFiles/g1 Ljava/lang/String;
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
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    ldc 30
    ldc 20
    invokestatic java/lang/System/arraycopy (Ljava/lang/Object;ILjava/lang/Object;II)V
    getstatic RandomAndRecordFiles/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    dup
    ldc 3
    i2l
    lconst_1
    lsub
    getstatic RandomAndRecordFiles/bccRecLen [I
    ldc 1
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/write ([B)V

    ; ---- Verify the updates ----

    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Part 2 (record/file DSL) -- after updates:"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V

    ldc 1
    putstatic RandomAndRecordFiles/g8 I
L_for_7_top:
    getstatic RandomAndRecordFiles/g8 I
    ldc 3
    if_icmpgt L_for_7_end
    ; let s = db[...]  (whole-record read)
    getstatic RandomAndRecordFiles/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    dup
    getstatic RandomAndRecordFiles/g8 I
    i2l
    lconst_1
    lsub
    getstatic RandomAndRecordFiles/bccRecLen [I
    ldc 1
    iconst_1
    isub
    iaload
    i2l
    lmul
    invokevirtual java/io/RandomAccessFile/seek (J)V
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/read ([B)I
    pop
    getstatic RandomAndRecordFiles/bccBufs [[B
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
    putstatic RandomAndRecordFiles/g14 I
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 2
    ldc 20
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    invokevirtual java/lang/String/length ()I
    putstatic RandomAndRecordFiles/g16 I
L_while_8_top:
    getstatic RandomAndRecordFiles/g16 I
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_while_8_end
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 2
    ldc 20
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    getstatic RandomAndRecordFiles/g16 I
    iconst_1
    isub
    dup
    ldc 1
    iadd
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    ldc " "
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ineg
    ifeq L_while_8_end
    getstatic RandomAndRecordFiles/g16 I
    ldc 1
    isub
    putstatic RandomAndRecordFiles/g16 I
    goto L_while_8_top
L_while_8_end:
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 2
    ldc 20
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    iconst_0
    getstatic RandomAndRecordFiles/g16 I
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    putstatic RandomAndRecordFiles/g15 Ljava/lang/String;
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 22
    ldc 8
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokevirtual java/lang/String/getBytes (Ljava/nio/charset/Charset;)[B
    invokestatic java/nio/ByteBuffer/wrap ([B)Ljava/nio/ByteBuffer;
    getstatic java/nio/ByteOrder/LITTLE_ENDIAN Ljava/nio/ByteOrder;
    invokevirtual java/nio/ByteBuffer/order (Ljava/nio/ByteOrder;)Ljava/nio/ByteBuffer;
    invokevirtual java/nio/ByteBuffer/getDouble ()D
    putstatic RandomAndRecordFiles/g17 D
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 30
    ldc 20
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    invokevirtual java/lang/String/length ()I
    putstatic RandomAndRecordFiles/g13 I
L_while_9_top:
    getstatic RandomAndRecordFiles/g13 I
    ldc 0
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_while_9_end
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 30
    ldc 20
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    getstatic RandomAndRecordFiles/g13 I
    iconst_1
    isub
    dup
    ldc 1
    iadd
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    ldc " "
    invokevirtual java/lang/String/equals (Ljava/lang/Object;)Z
    ineg
    ifeq L_while_9_end
    getstatic RandomAndRecordFiles/g13 I
    ldc 1
    isub
    putstatic RandomAndRecordFiles/g13 I
    goto L_while_9_top
L_while_9_end:
    getstatic RandomAndRecordFiles/bccBufs [[B
    ldc 0
    aaload
    new java/lang/String
    dup_x1
    swap
    ldc 30
    ldc 20
    getstatic java/nio/charset/StandardCharsets/ISO_8859_1 Ljava/nio/charset/Charset;
    invokespecial java/lang/String/<init> ([BIILjava/nio/charset/Charset;)V
    iconst_0
    getstatic RandomAndRecordFiles/g13 I
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    putstatic RandomAndRecordFiles/g12 Ljava/lang/String;
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
    ldc "  "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    getstatic RandomAndRecordFiles/g15 Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc ": "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    getstatic RandomAndRecordFiles/g17 D
    invokestatic RandomAndRecordFiles/bccStr (D)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic RandomAndRecordFiles/g8 I
    ldc 1
    iadd
    putstatic RandomAndRecordFiles/g8 I
    goto L_for_7_top
L_for_7_end:

    ; db.close()
    getstatic RandomAndRecordFiles/bccFiles [Ljava/io/RandomAccessFile;
    ldc 1
    iconst_1
    isub
    aaload
    invokevirtual java/io/RandomAccessFile/close ()V

    ; ------------------------------------------------------------------------
    ; Part 2 is the same three writes, the same reverse-order read, and the
    ; same three updates as Part 1 — Alice's and Bob's and Carol's updates
    ; still transpile to exactly one GET and one PUT each, nothing runs slower.
    ; What's gone is everything that was bookkeeping rather than logic: the
    ; hand-computed record width, the repeated buffer-variable/FIELD
    ; boilerplate in every block, the pack/unpack call picked by hand per
    ; field, and the GET-or-not decision for a partial write, which the
    ; compiler now makes for you at compile time by simply comparing field
    ; names -- get a field name wrong (`db[1] = ?{ nmae: ... }`) and it's a
    ; compile error instead of a silently corrupted record.
    ; ------------------------------------------------------------------------

    return
.end method
