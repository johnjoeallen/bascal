# BASCAL

<img src="docs/hero-computer.svg" width="220" align="right" alt="A retro all-in-one computer showing green phosphor BASIC code, with a violet cursor blinking at the prompt">

**B**eginner's **A**ll-purpose **S**tructured **C**omputer **A**pplication
**L**anguage.

BASCAL is a structured superset of classic Microsoft BASIC, inspired mainly
by Pascal. Its `bcc` compiler accepts readable `.bcl` source — structured
control flow, functions and procedures, methods, typed records, and
path-style `require` dependencies. The BASIC and C backends transpile it to
plain 1980s BASIC (for a period compiler such as BASCOM or FreeBASIC's
QB-compatible mode) or native C; the JVM backend emits low-level Krakatau
assembly for a JVM class file.

*A fun project, built mostly to see what's possible — not really meant for
building a real 2026 application. See
[the origin story](https://johnjoeallen.github.io/bascal/origin.html).*

**[Browse the website](https://johnjoeallen.github.io/bascal/)** for the
[language book](https://johnjoeallen.github.io/bascal/language/), tutorials,
worked examples, side-by-side syntax comparisons, and the full
[language manual](https://johnjoeallen.github.io/bascal/manual/) — the
complete language reference. Current backend and feature coverage is listed
in the [conformance results](https://johnjoeallen.github.io/bascal/conformance/).

## Licence and generated programs

BASCAL itself is GPLv3. Its [output exception](LICENSE-OUTPUT-EXCEPTION.md)
means that BASCAL source programs, generated BASIC or C, compiled binaries,
and C runtime-support functions included in those binaries are not subject to
the GPLv3 solely because BASCAL produced them. You may license them as you see
fit.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for building, testing, and
repository-layout details.
