[Home](../) / [Manual](../manual/) / Getting Started

[← Introduction](introduction.md) [Program Structure →](program-structure.md)

<div class="prose" markdown="1">

### Building bcc

```bascal
env -u RUSTC_WRAPPER cargo build --release
```

The compiled binary is `target/release/bcc`.

### Your First Program

The file `tutorial/01_hello.bcl` demonstrates all three comment styles and a basic PRINT/END structure:

```bascal
// Tutorial 1 — Hello, World
' This is a classic single-quote comment (passes through to BASIC as-is).
// This is a double-slash end-of-line comment (same behaviour).

/*
 * Block comments span multiple lines.  Each line is emitted as a separate
 * ' comment in the generated output; blank lines are preserved as blank lines.
 */

PRINT "Hello, World!"
PRINT "Welcome to BASCAL."
END
```

Transpile it:

```bascal
bcc tutorial/01_hello.bcl
```

This produces `tutorial/01_hello.bas`. To compile and run with FreeBASIC:

```bascal
bcc tutorial/01_hello.bcl --binary
./tmp/01_hello
```

### A Simple Function

```bascal
' name$ -- who to greet
function greet$(name$)
    return "Hello, " + name$ + "!"
end function

msg$ = greet$("BASCOM")
PRINT msg$
END
```

</div>

[← Introduction](introduction.md) [Program Structure →](program-structure.md)
