# BASCAL

<img src="docs/hero-computer.svg" width="220" align="right" alt="A retro all-in-one computer showing green phosphor BASIC code, with a violet cursor blinking at the prompt">

**B**eginner's **A**ll-purpose **S**tructured **C**omputer **A**pplication
**L**anguage.

BASCAL is a structured superset of classic Microsoft BASIC, inspired mainly
by Pascal. Its `bcc` transpiler turns readable `.bcl` source — structured
control flow, functions and procedures, scalar methods, typed records, and
path-style `require` dependencies — into either plain 1980s BASIC (compiled
with a period compiler like BASCOM, or FreeBASIC's QB-compatible mode) or
native C, with no BASIC compiler involved.

*A fun project, built mostly to see what's possible — not really meant for
building a real 2026 application. See
[the origin story](https://johnjoeallen.github.io/bascal/origin.html).*

**[Browse the website](https://johnjoeallen.github.io/bascal/)** for the
[language book](https://johnjoeallen.github.io/bascal/language/), tutorials,
worked examples, side-by-side syntax comparisons, and the full
[language manual](https://johnjoeallen.github.io/bascal/manual/) — the
complete language reference.

## Licence and generated programs

BASCAL itself is GPLv3. Its [output exception](LICENSE-OUTPUT-EXCEPTION.md)
means that BASCAL source programs, generated BASIC or C, compiled binaries,
and C runtime-support functions included in those binaries are not subject to
the GPLv3 solely because BASCAL produced them. You may license them as you see
fit.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for building, testing, and
repository-layout details.
