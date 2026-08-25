[Home](../../) / [Tutorials](../) / Hello, World

<div class="prose" markdown="1">

Every BASCAL program ends with `end`. `print` writes a line to the screen. Three comment styles are available and all three are preserved in the generated output: a leading `'` (classic BASIC style), a leading `//` (C style), and `/* ... */` block comments.

Generated sources: [BCL](https://github.com/johnjoeallen/bascal/blob/main/tutorial/01_hello.bcl), [BASIC](https://github.com/johnjoeallen/bascal/blob/main/tutorial/01_hello.bas), [C](https://github.com/johnjoeallen/bascal/blob/main/tutorial/01_hello.c), and [JVM assembly](https://github.com/johnjoeallen/bascal/blob/main/tutorial/01_hello.j).

</div>

<div class="snippet" markdown="1">

### The whole program

```bascal
print "Hello, World!"        // inline comment after a statement
print "Welcome to BASCAL."
' print "This line is commented out."

end
```

</div>



[Variables and Constants →](02_variables.md)


<!-- BEGIN generated tutorial source -->

<details class="source-embed" markdown="1">

<summary><code>tutorial/01_hello.j</code></summary>



```basic

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

```



</details>

<!-- END generated tutorial source -->
