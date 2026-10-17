<div id="what" class="section" markdown="1">

## What is BASCAL

BASIC was once one of the most widely used programming languages, but today it has largely disappeared from mainstream software development. Its descendants remain in use, particularly in legacy applications and automation, but classic BASIC belongs to an earlier generation of computing.

BASCAL revisits classic BASIC with a straightforward question: **what would BASIC look like with structured syntax and modern language tooling, while still remaining BASIC?**

BASCAL is a structured, compiled language: a thoroughly modernised dialect of BASIC, influenced by Pascal and by later languages such as Java and Groovy. Its compiler, `bcc`, is written in Rust. BASCAL retains the familiar foundations of BASIC while adding structured syntax, scoped variables, functions, procedures and reusable source files. `bcc` processes BASCAL as a language in its own right and can generate code for several target environments.

From a programmer's perspective, this raises several questions. How can BASIC be made easier to structure, read and maintain? Which language features are useful without losing the simplicity that made BASIC accessible? How can larger applications be organised effectively? And how can the same source work with a classic Microsoft BASIC environment while also supporting C and the JVM?

These are the questions this guide sets out to answer.

BASCAL began as a strict superset of classic BASIC, and the `basic` target remains the closest expression of that approach. A substantial amount of existing BASIC syntax can be used directly, including facilities for bitwise operations and traditional file handling.

`GOTO` and `GOSUB` are supported, with one deliberate difference: BASCAL manages line numbering. Branch targets are therefore expressed as named labels rather than physical line numbers such as `GOTO 140`.

Where BASCAL provides its own construct, that construct is the preferred form for `.bcl` source. Classic BASIC syntax should be regarded as something BASCAL is intended to steer new code away from, rather than as an equally preferred alternative. For the C and JVM targets, some BASIC-specific syntax is not supported, and mixing BASCAL constructs with equivalent classic BASIC forms is deliberately restricted. This keeps source code consistent and makes its intended behaviour clear across different targets.

BASCAL also retains an important connection with the BASIC environment it targets. Its structured features do not require the underlying BASIC runtime to provide capabilities it does not have. BASCAL handles those details during translation while presenting a cleaner and more structured source language to the programmer.

The same BASCAL source can also target C or the JVM. These environments differ significantly from classic BASIC, so a small number of BASIC-specific constructs cannot be supported consistently across every target. BASCAL as a whole is therefore a **partial superset** of classic BASIC, while the `basic` target provides the greatest compatibility. Code intended to be portable across targets should use BASCAL's structured constructs wherever an equivalent is provided.

This distinction between the language and its targets is an important part of BASCAL's design. The BASIC backend maintains compatibility with the environment that inspired the original project. The C backend provides native compilation and portability, while the JVM backend provides access to a modern managed runtime, built on the [Krakatau](https://github.com/Storyyeller/Krakatau) assembler — embedded directly into `bcc` as a library, with thanks to Storyyeller and the Krakatau project. In each case, the programmer writes BASCAL; the compiler handles the requirements of the selected target.

[Portability across backends](https://johnjoeallen.github.io/bascal/manual/command-line-reference/#portability-across-backends) describes the differences between the BASIC, C and JVM targets and the BASCAL constructs to use when writing portable programs.

</div>

---

[Next: Why was it created →](home/why.md)
