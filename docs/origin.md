[Home](../) / Origin

<div class="prose" markdown="1">

## Ramtech BASIC

BASCAL grew from **Ramtech BASIC**, a small preprocessor written in 1985 for
the Microsoft BASIC applications developed at Ramtech Ireland. The programs
were large business applications, built by a distributed team, with shared
routines copied into every program that used them.

That copying made every library change a manual merge in each developer's
work and again in the complete application. Calling a routine also meant
remembering its line number. Ramtech BASIC addressed those immediate problems:
`{label}` gave routines symbolic names, and `@include` assembled shared code
without copying it by hand.

The first version was a weekend Pascal project. It was then rewritten in
BASIC so it could be used in the existing development environment. Over time
it gained multiline `@if` / `@else`, `@case`, and `@function` /
`@procedure`, all marked with `@` so the preprocessor could distinguish its
directives from ordinary BASIC. Later versions also supported separate
compilation and reuse of generated BASIC.

Ramtech BASIC was not a replacement runtime or a new BASIC compiler. It
prepared structured source and generated ordinary BASIC for the tools already
in use. The point was simply to keep common routines in one place, assemble
programs from source files, and make larger BASIC programs less fragile.

BASCAL is the modern Rust reconstruction of that idea. It is a structured
language whose `bcc` compiler transpiles `.bcl` source to classic BASIC, C,
or JVM bytecode. It is not a recreation of the original tool, but follows the
same aim with a real language grammar, reusable source files, and structured
control flow. Unlike Ramtech BASIC, it does not need the old `@` directive
prefix: these constructs are part of the language itself.

</div>

[← Back to Why BASCAL](index.md)  ·  [Read the development journey →](journey.md)
