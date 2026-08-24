## A language for writing BASIC more clearly

BASCAL starts from a simple observation: classic BASIC is immediate and practical, but line numbers and jumps can make a program hard to follow. BASCAL keeps BASIC’s familiar data and statements while giving the source a clearer shape.

It is a structured superset of BASIC, inspired mainly by Pascal. Blocks show choices and repetition. Functions and procedures give work a name. `require` lets one source file use another by path. A required library becomes part of the generated program. Typed records describe saved data without making you repeat its byte-level layout.

## The source and its targets

The BASCAL compiler (`bcc`) transpiles `.bcl` source files to line-numbered Microsoft BASIC by default, or to C with `--target c`. The C backend still has a small number of documented gaps. You can read, inspect, and compile the generated output with the appropriate toolchain.

The usual advice is simple: use a BASCAL construct when one fits the job. Prefer structured `if` statements and loops to hand-wired jumps, and `record`/`file` to hand-written random-file bookkeeping. Raw BASIC remains available when a program genuinely needs target-specific behaviour.

## Who should read this book

This book is for programmers learning BASCAL, whether they already know BASIC, are returning to it, or enjoy working with a small language and visible output. It assumes only that you can read a program and are willing to run short examples. Pascal experience is useful, but not required.

## How to use this book

Read the first part in order. Each chapter begins with an idea in a complete program and adds the detail needed to use it. Later chapters give the full syntax rules, behaviour, and command-line information, making this book the authoritative language reference.

<div class="aside" markdown="1">

This book shows how to think in BASCAL and says exactly what each form does.

</div>

Compile and alter the small programs as you read. You can inspect BASCAL’s generated code, but maintain the clear, structured source program.
