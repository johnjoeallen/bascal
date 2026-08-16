## Preface: Why BASCAL Exists

My first professional programming job was at Ramtech Ireland, starting in May 1985.

By that point I had already spent time with Z80 assembly, 8086 assembly, C, and Pascal. I had been exposed to languages and environments that gave me a sense of structure, control, and abstraction. Then I found myself writing business software professionally in Microsoft BASIC.

I had also become very enamoured with Turbo Pascal. The language itself was fantastic, but the Turbo Pascal environment was amazing: an integrated editor, compiler, and run loop that made development feel lightning fast. Nothing in Ramtech BASIC brought that same fast edit-compile cycle to classic BASIC, but that experience shaped what I wanted: more structure, less friction, and a development process that felt more flexible.

BASIC was approachable and productive, but compared with the tools and languages I had already used, the working environment felt very limited. The language encouraged large global programs, weak structure, remembered line numbers, and repetitive copying of shared code. I wanted a better way to work. I wanted some of the discipline and convenience I was used to from Pascal and C, but without abandoning the BASIC system we actually had to deliver software with.

One thing that especially bothered me was the way shared routines were handled.

We had what amounted to a standard library of BASIC routines, starting around line 10000, which had to be copied into each program that needed them. We also had a distributed dev team, so every change to that shared library had to be merged by hand across each developer's copy, and then merged again into the full application suite. Even at the time, that felt prehistoric to me. It was not just inconvenient; it was a maintenance trap. Every copied routine was another chance for programs to drift out of sync.

There was also the practical problem of remembering what routine started at what line number. Calling a shared routine meant knowing where it lived. My first attack on that problem was not a sophisticated compiler. It was a preprocessor that could provide labels — written as `{label}` in place of a raw line number, giving routines symbolic names instead of remembered line numbers — and `@include`, a way to pull shared code into a program without manually copying it.

Because I was working for Ramtech Ireland, I called it **Ramtech BASIC**.

The first version was a weekend project, written in Pascal, and it supported only two things: `{label}` and `@include`. I demoed it to my boss, and he liked the idea enough to give me a week to rewrite it in BASIC so it could live inside the same environment as the programs it was helping to build.

The original tool was not a new runtime, and it was not a replacement for BASIC. It was a practical layer over the BASIC we had: a way to make the source more structured and then generate BASIC that could still be compiled and run in the existing environment.

That week is where the rest of the directive set came from. I added multiline `@IF` / `@ELSE`, `@CASE`, and `@FUNCTION` / `@PROCEDURE`, all prefixed with `@` to make the source easy for the preprocessor to scan. The `@` prefix was not there because I wanted a strange-looking language; it was there because it made the implementation practical. The tool could quickly distinguish preprocessor constructs from ordinary BASIC text.

The original tool was much simpler than BASCAL. It was a product of its time, built to solve immediate problems with the machines, compilers, and constraints we had. But the motivation was already there: stop remembering magic line numbers, make shared code easier to reuse, reduce repetitive copying, add some structure, and make BASIC source feel less fragile.

Later versions of the original tooling became more capable, including separate compilation and reuse of generated BASIC.

**BASCAL** is a modern reconstruction of that idea, written in Rust.

It is not intended to be exactly what I built in the 1980s. It is closer to what I wish I could have built then — the tool I wish I'd had the skills and tools for back in 1985. Building a real transpiler, rather than another preprocessor, was finally within reach with Claude and Codex doing the heavy lifting. The result is BASCAL: not a Microsoft BASIC compiler, but a structured language of its own that `bcc` transpiles into classic Microsoft BASIC, keeping the original global BASIC model while adding a more disciplined source language and a more practical build workflow. The language itself is significantly more advanced than the original: more structured, easier to read, easier to write, and organized into reusable files.

BASCAL source uses `.bcl` files and is transpiled by `bcc` into generated `.bas` output. It supports multiline `if` / `else` / `end if`, `for` / `next`, `while` / `wend`, `function` declarations with explicit `return`, BASIC type suffixes, comments preserved in generated output, and path-style `require` / `import` dependencies.

BASCAL is also a strict superset of that classic BASIC: bitwise `AND`/`OR`/`NOT` and hand-written `OPEN`/`FIELD`/`GET`/`PUT` still pass through unchanged. `GOTO`/`GOSUB` are raw BASIC too, but BASCAL manages line numbering itself, so their targets are always a `name:` label declared in source, never a raw line number. Beyond that, wherever BASCAL has its own construct for something, that construct is the canonical way to write it in `.bcl` source — the original BASIC syntax is what you're transpiling *away from*, not an equally-good alternative.

One visible difference from the original Ramtech BASIC preprocessor is that BASCAL does not need the old `@` prefix. The original prefix made sense for a small preprocessor scanning BASIC text. BASCAL is being built as a proper transpiler-style tool, so its structured constructs can be part of the language grammar rather than marked as special preprocessor commands.

The core idea remains the same:

> Make BASIC more pleasant to write, without pretending it is a different runtime.

BASCAL deliberately preserves BASIC's global symbol model — variables and functions are still global, still transpiled to `GOTO`/`GOSUB` — while adding enough structure that larger programs stay maintainable. Path-style names are dependency selectors, not runtime namespaces. Functions are transpiled to global parameter/result variables plus `GOSUB`. Every parameter is copied in before the call; `byref` copies the result back out afterward too (`byval`, the default, doesn't). Array parameters support any number of dimensions, matched against how the callee's body indexes them — a mismatch is a transpile-time error. Recursive functions are not supported, because a recursive call would overwrite its own global parameter state.

Generated BASIC is intentionally conservative. BASCAL transpiles structured source into line-numbered `GOTO` / `GOSUB` style output suitable for classic BASIC-oriented tooling, while still allowing the source program to be much clearer than the BASIC it generates.

In that sense, BASCAL is both a small transpiler project and a personal historical exercise: a chance to revisit the frustration that led to Ramtech BASIC, and to build the version of the tool I would have loved to have had at the start of my career.

I don't expect anyone to actually reach for BASCAL to ship a real application in 2026 — that was never really the point. It's a fun project: built mostly to see what's possible, and to scratch an old itch along the way.
