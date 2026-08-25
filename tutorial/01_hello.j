.version 50 0
.class public Hello
.super java/lang/Object

.method public static main : ([Ljava/lang/String;)V
    .limit stack 16
    .limit locals 1

    ; Tutorial — Hello, World
    ; 
    ; The simplest BASCAL program.  print writes a line to the screen.
    ; END marks the bottom of the main program body; every program needs one.
    ; 
    ; Three comment styles are available:
    ; '  single-line (BASIC style, passed through to generated output)
    ; // single-line (C style, same behaviour as ')
    ; /* ... */  block comment, each line becomes a ' comment in the output

    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Hello, World!"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Welcome to BASCAL."
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    ; print "This line is commented out."

    ; Expected output:
    ; Hello, World!
    ; Welcome to BASCAL.

    return
.end method
