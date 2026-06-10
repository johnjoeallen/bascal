## Preface: Why BASCAL Exists

My first professional programming job was at Ramtech, starting in May 1985.

By that point I had already spent time with Z80 assembly, 8086 assembly, C, and Pascal. I had been exposed to languages and environments that gave me a sense of structure, control, and abstraction. Then I found myself writing business software professionally in Microsoft BASIC.

That was a shock.

BASIC was approachable and productive, but compared with the tools and languages I had already used, the working environment felt very limited. The language encouraged large global programs, weak structure, remembered line numbers, and repetitive copying of shared code. I wanted a better way to work. I wanted some of the discipline and convenience I was used to from Pascal and C, but without abandoning the BASIC system we actually had to deliver software with.

One thing that especially bothered me was the way shared routines were handled.

We had what amounted to a standard library of BASIC routines, starting around line 10000, which had to be copied into each program that needed them. If that shared code was updated, it then had to be copied all over again into every program that used it. Even at the time, that felt prehistoric to me. It was not just inconvenient; it was a maintenance trap. Every copied routine was another chance for programs to drift out of sync.

There was also the practical problem of remembering what routine started at what line number. Calling a shared routine meant knowing where it lived. My first attack on that problem was not a sophisticated compiler. It was a preprocessor that could provide labels and `@include`: symbolic names instead of remembered line numbers, and a way to pull shared code into a program without manually copying it.

Because I was working for Ramtech, I called it **Ramtech BASIC**.

The first version was a weekend project, written in Pascal. I demoed it to my boss, and he liked the idea enough to give me a week to rewrite it in BASIC so it could live inside the same environment as the programs it was helping to build.

The original tool was not a new runtime, and it was not a replacement for BASIC. It was a practical layer over the BASIC we had: a way to make the source more structured and then generate BASIC that could still be compiled and run in the existing environment.

The original preprocessor was deliberately simple. It added things like `@include`, multiline `@IF` / `@ELSE`, `@CASE`, and `@FUNCTION` / `@PROCEDURE`. These constructs were all prefixed with `@` to make the source easy for the preprocessor to scan. The `@` prefix was not there because I wanted a strange-looking language; it was there because it made the implementation practical. The tool could quickly distinguish preprocessor constructs from ordinary BASIC text.

The original tool was much simpler than BASCAL. It was a product of its time, built to solve immediate problems with the machines, compilers, and constraints we had. But the motivation was already there: stop remembering magic line numbers, make shared code easier to reuse, reduce repetitive copying, add some structure, and make BASIC source feel less fragile.

Later versions of the original tooling became more capable, including separate compilation and reuse of generated BASIC.

**BASCAL** is a modern reconstruction of that idea, written in Rust.

It is not intended to be exactly what I built in the 1980s. It is closer to what I wish I could have built then: a structured compiler for classic Microsoft BASIC that keeps the original global BASIC model, but adds a more disciplined source language and a more practical build workflow.

BASCAL source uses `.bcl` files and is compiled by `bcc` into generated `.bas` output. It supports multiline `if` / `else` / `end if`, `for` / `next`, `while` / `wend`, `function` declarations with explicit `return`, BASIC type suffixes, comments preserved in generated output, and path-style `require` / `import` dependencies.

One visible difference from the original Ramtech BASIC preprocessor is that BASCAL does not need the old `@` prefix. The original prefix made sense for a small preprocessor scanning BASIC text. BASCAL is being built as a proper compiler-style tool, so its structured constructs can be part of the language grammar rather than marked as special preprocessor commands.

The core idea remains the same:

> Make BASIC more pleasant to write, without pretending it is a different runtime.

BASCAL deliberately preserves BASIC's global symbol model. Variables and functions are global. Path-style names are dependency selectors, not runtime namespaces. Functions are lowered to global parameter/result variables plus `GOSUB`. Array arguments use copy-in/copy-out around the call. Recursive functions are not supported, because a recursive call would overwrite its own global parameter state.

Generated BASIC is intentionally conservative. BASCAL lowers structured source into line-numbered `GOTO` / `GOSUB` style output suitable for classic BASIC-oriented tooling, while still allowing the source program to be much clearer than the BASIC it generates.

In that sense, BASCAL is both a small compiler project and a personal historical exercise: a chance to revisit the frustration that led to Ramtech BASIC, and to build the version of the tool I would have loved to have had at the start of my career.
