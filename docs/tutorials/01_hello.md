[Home](../../) / [Tutorials](../) / Hello, World

<div class="prose" markdown="1">

Every BASCAL program ends with `end`. `print` writes a line to the screen. Three comment styles are available and all three are preserved in the generated output: a leading `'` (classic BASIC style), a leading `//` (C style), and `/* ... */` block comments.

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

<summary><code>tutorial/01_hello.bcl</code></summary>



```bascal

// Tutorial — Hello, World
//
// The simplest BASCAL program.  print writes a line to the screen.
// END marks the bottom of the main program body; every program needs one.
//
// Three comment styles are available:
//   '  single-line (BASIC style, passed through to generated output)
//   // single-line (C style, same behaviour as ')
//   /* ... */  block comment, each line becomes a ' comment in the output
program hello

print "Hello, World!"        // inline comment after a statement
print "Welcome to BASCAL."
' print "This line is commented out."

/*
 * Expected output:
 *   Hello, World!
 *   Welcome to BASCAL.
 */

end

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/01_hello.bas</code></summary>



```basic

10 ' BASCAL generated BASIC -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Tutorial — Hello, World
40 '
50 ' The simplest BASCAL program.  print writes a line to the screen.
60 ' END marks the bottom of the main program body; every program needs one.
70 '
80 ' Three comment styles are available:
90 ' '  single-line (BASIC style, passed through to generated output)
100 ' // single-line (C style, same behaviour as ')
110 ' /* ... */  block comment, each line becomes a ' comment in the output

120 PRINT "Hello, World!"
130 PRINT "Welcome to BASCAL."
140 ' print "This line is commented out."

150 ' Expected output:
160 ' Hello, World!
170 ' Welcome to BASCAL.

180 END

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/01_hello.c</code></summary>



```c

// BASCAL generated C -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
#include <stdio.h>

int main(void) {
    // Tutorial — Hello, World
    //
    // The simplest BASCAL program.  print writes a line to the screen.
    // END marks the bottom of the main program body; every program needs one.
    //
    // Three comment styles are available:
    // '  single-line (BASIC style, passed through to generated output)
    // // single-line (C style, same behaviour as ')
    // /* ... */  block comment, each line becomes a ' comment in the output

    printf("Hello, World!\n");
    printf("Welcome to BASCAL.\n");
    // print "This line is commented out."

    // Expected output:
    // Hello, World!
    // Welcome to BASCAL.

    return 0;
}

```



</details>

<!-- END generated tutorial source -->
