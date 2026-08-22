# BASCAL Manual

The full language reference now lives on the website:
https://johnjoeallen.github.io/bascal/manual/

This file is intentionally minimal so there's only one
place (the website) to keep the detailed reference in sync.

## Quick reference

- Build: `env -u RUSTC_WRAPPER cargo build`
- Usage: `bcc input.bcl [-o output.bas] [-L dir] [-l library] [--line-numbers] [--clean | -c] [--binary | -b] [--target | -t basic|c]`
- Targets: `basic` (default, complete), `c` (experimental)

See the website for everything else: syntax, statements,
operators, backends, and every tutorial.
