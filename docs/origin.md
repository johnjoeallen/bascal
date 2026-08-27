[Home](../) / Origin

<div class="prose" markdown="1">

## Ramtech BASIC

BASCAL grew from **Ramtech BASIC**, a small preprocessor written in 1985 for
the Microsoft BASIC applications developed at Ramtech Ireland.

It replaced remembered line numbers with `{label}` names and supported
`@include` for shared source. Later versions added `@if` / `@else`, `@case`,
and `@function` / `@procedure`, and supported separate compilation and reuse
of generated BASIC. Its purpose was practical: keep shared routines in one
place, assemble programs from source files, and make large BASIC programs
less fragile.

BASCAL is the modern Rust reconstruction of that idea. It is a structured
language whose `bcc` compiler transpiles `.bcl` source to classic BASIC, C,
or JVM bytecode. Unlike Ramtech BASIC, it is a compiler with its own grammar;
the old `@` directive prefix is not needed.

</div>

[← Back to Why BASCAL](index.md)  ·  [Read the development journey →](journey.md)
