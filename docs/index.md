<div id="why" class="section" markdown="1">

## Why BASCAL exists

BASIC was once one of the most widely used programming languages, but today it has largely disappeared from mainstream software development. Its descendants remain in use, particularly in legacy applications and automation, but classic BASIC belongs to an earlier generation of computing.

BASCAL revisits classic BASIC with a straightforward question: **what would BASIC look like with structured syntax and modern language tooling, while still remaining BASIC?**

From a programmer's perspective, this raises several questions. How can BASIC be made easier to structure, read and maintain? Which language features are useful without losing the simplicity that made BASIC accessible? How can larger applications be organised effectively? And how can the same source work with a classic Microsoft BASIC environment while also supporting C and the JVM?

These are the questions this guide sets out to answer.

BASCAL traces its origins to 1985 and a preprocessor I wrote for a commercial BASIC development environment. We maintained a shared set of library routines across an application suite developed by a distributed team. Changes to those routines had to be merged manually into each developer's copy and subsequently into the applications that used them.

The preprocessor was written to address those problems. It allowed applications to be divided into multiple source files which were assembled into a single BASIC program for compilation. Shared library components could be maintained separately rather than copied and edited within each application. It also provided directives such as `@include`, `@if`, `@case`, `@function` and `@procedure`, together with `{label}` as an alternative to working directly with BASIC line numbers.

This made it considerably easier to maintain a suite of BASIC applications and to work across multiple developers and locations. Source files and common components could be managed independently, while the preprocessor dealt with assembling them into the form expected by the BASIC compiler.

I always intended to develop the idea further, but the tools and experience available to me in 1985 limited what was practical. Four decades later, modern development tools, including Claude and Codex, made it possible to revisit the idea and implement BASCAL as a compiler rather than a text preprocessor.

BASCAL is that modern implementation, written in Rust.

The language has developed considerably beyond its 1985 predecessor. It retains the familiar foundations of BASIC while adding structured syntax, scoped variables, functions, procedures and reusable source files. Its compiler processes BASCAL as a language in its own right and can generate code for several target environments.

BASCAL builds on a direction that was already apparent in later BASIC dialects such as Microsoft QuickBASIC. QuickBASIC introduced structured procedures and functions, local variables, `SELECT CASE` and support for organising larger applications into separately compiled modules. BASCAL develops these ideas further while also drawing on concepts from languages such as Pascal, C, Java and Groovy where they provide a natural fit.

As a result, many BASCAL concepts will be familiar to programmers accustomed to more modern languages, while the resulting source remains recognisably BASIC. The purpose is not to turn BASIC into an unrecognisable language, but to provide a more structured and consistent way of writing it.

BASCAL began as a strict superset of classic BASIC, and the `basic` target remains the closest expression of that approach. A substantial amount of existing BASIC syntax can be used directly, including facilities for bitwise operations and traditional file handling.

`GOTO` and `GOSUB` are also supported, with one deliberate difference: BASCAL manages line numbering. Branch targets are therefore expressed as named labels rather than physical line numbers such as `GOTO 140`.

Where BASCAL provides its own construct, that construct is the preferred form for `.bcl` source. Classic BASIC syntax should be regarded as something BASCAL is intended to steer new code away from, rather than as an equally preferred alternative. For the C and JVM targets, some BASIC-specific syntax is not supported, and mixing BASCAL constructs with equivalent classic BASIC forms is deliberately restricted. This keeps source code consistent and makes its intended behaviour clear across different targets.

BASCAL also retains an important connection with the BASIC environment it targets. Its structured features do not require the underlying BASIC runtime to provide capabilities it does not have. BASCAL handles those details during translation while presenting a cleaner and more structured source language to the programmer.

The same BASCAL source can also target C or the JVM. These environments differ significantly from classic BASIC, so a small number of BASIC-specific constructs cannot be supported consistently across every target. BASCAL as a whole is therefore a **partial superset** of classic BASIC, while the `basic` target provides the greatest compatibility. Code intended to be portable across targets should use BASCAL's structured constructs wherever an equivalent is provided.

This distinction between the language and its targets is an important part of BASCAL's design. The BASIC backend maintains compatibility with the environment that inspired the original project. The C backend provides native compilation and portability, while the JVM backend provides access to a modern managed runtime. In each case, the programmer writes BASCAL; the compiler handles the requirements of the selected target.

The objectives remain much the same as they were in 1985: **make BASIC easier to write, structure and maintain, while respecting the environment in which it runs.** The original preprocessor allowed applications to be divided into multiple source files and assembled into a single BASIC program, making shared libraries easier to maintain and reducing the effort required to merge changes across multi-developer and distributed teams. BASCAL carries those ideas forward with structured source code, reusable components and a modern development model, while still producing code appropriate to its target environment.

The [full origin story](https://johnjoeallen.github.io/bascal/origin/) provides more background on the original preprocessor and the development of BASCAL. [Portability across backends](https://johnjoeallen.github.io/bascal/manual/command-line-reference/#portability-across-backends) describes the differences between the BASIC, C and JVM targets and the BASCAL constructs to use when writing portable programs.

</div>

---

[Next: What BASCAL adds →](home/features.md)
