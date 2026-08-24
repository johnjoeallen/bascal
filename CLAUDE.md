# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

See also [AGENTS.md](AGENTS.md) for terminology conventions.

## What this is

BASCAL source files use the `.bcl` extension: a Pascal-inspired, structured
superset of classic Microsoft BASIC. `bcc`, BASCAL's Rust CLI, transpiles
that `.bcl` source into either plain 1980s Microsoft BASIC (`--target
basic`, the complete/default backend) or experimental native C (`--target
C`, narrow and still growing).

## Commands

```bash
env -u RUSTC_WRAPPER cargo build
env -u RUSTC_WRAPPER cargo test
```

- Run a single test: `cargo test <test_name>` (e.g. `cargo test compiles_every_example_bcl_file`).
- The default test suite covers lexer/parser/validation/codegen unit tests, plus transpiling every `tutorial/**/*.bcl` file (`tests/examples.rs`).
- Two extra suites are opt-in and skip cleanly when their prerequisites are absent: if `fbc` (FreeBASIC) is installed, it compiles/runs `sort_driver` and `remline` end-to-end; if `dosbox-x` is installed *and* `test-fixtures/ibm-basic-compiler/` has been populated via `scripts/fetch-ibm-basic-compiler.sh`, `tests/dosbox_conformance.rs` diffs generated `.bas` output against real IBM BASIC Compiler 2.00 output. See CONTRIBUTING.md for details — don't try to install/fetch these yourself.
- CLI usage: `bcc <input.bcl> [-o DIR] [-t basic|C] [-b|--binary] [-r|--run] [--strict-vars]` — run `cargo run -- --help` for the full flag list.

## Architecture

Pipeline, in `src/lib.rs`'s `compile_file`/`compile_source`: **lexer → parser → AST → resolver → codegen**.

- `lexer.rs` / `parser.rs` — tokenize and parse `.bcl` into `ast.rs`'s `Program`.
- `resolver.rs` — semantic checks (name resolution, `--strict-vars` enforcement, etc.) over the AST before codegen.
- `records.rs` — typed record (struct-like) support, shared across both backends.
- `scalar_builtins.rs` — built-in scalar methods available on values.
- `codegen.rs` — backend dispatch only; re-exports the BASIC backend's public surface (`CodeGenerator`) so callers never need to know the split into `codegen_basic`/`codegen_c` exists. `Target` (`Basic` default, `C`) selects the backend.
- `codegen_basic.rs` — the original, complete backend: plain BASCOM-compatible BASIC. Only numbers lines that need numbers (branch targets) by default; `--line-numbers` numbers every line for strict BASCOM compatibility.
- `codegen_c.rs` — experimental native-C backend, aiming eventually at Linux/macOS/Win32 binaries with no BASIC compiler involved. Narrower than the BASIC backend; check here before assuming a feature is supported for `--target C`.
- `main.rs` — the `bcc` CLI (clap). Handles `require` library search paths, output-path defaulting, optional post-transpile compile (`fbc`/`gcc`) and run.

`require`-based dependencies resolve against library search roots (`-L` dirs, `com/` stdlib, etc.) in `lib.rs`'s `search_roots`/`load_program_recursive` — this is how multi-file BASCAL programs and the `com/` standard library get pulled in.

### Testing conventions

- `tutorial/**/*.bcl` are both the tutorial content and the transpiler's regression corpus: `tests/examples.rs` compiles every one and writes `.bas`/`.c` output alongside the source (tracked in git — regenerate rather than hand-edit when behavior changes).
- New conformance fixtures (real-BASCOM-verified behavior) go in `tests/fixtures/conformance/*.bcl`, paired with a `*.expected.txt` of exact stdout; keep them small, deterministic, and scoped to one behavior.
- Generated BASIC/C output for one-off manual checks belongs in `output/`; temporary compiled binaries belong in `tmp/` (both git-ignored except where noted above).

## Conventions

- Say **"transpile"**, never "lower"/"lowering", in docs, commit messages, and code comments.
- In `codegen_c.rs`, implement AND/OR/XOR with C's bitwise operators on -1/0 values, not `&&`/`||`.

## Releasing

When the user says **"release"** (or "cut a release", "tag a release", etc.):

1. Increment the patch number in `Cargo.toml` (e.g. `0.99.3` → `0.99.4`).
2. Run `cargo build -q` to propagate the change into `Cargo.lock`.
3. Commit **only** `Cargo.toml` and `Cargo.lock` with the message
   `chore: bump version to <new-version>`.
4. Run `cargo test`. If any test fails, fix it before continuing — do not tag
   a broken build.
5. Create an annotated tag on that commit:
   ```
   git tag -a v<new-version> -m "Release v<new-version>"
   ```

## Regular commits

Source code and test commits (`src/`, `tests/`) do **not** automatically
trigger a version bump — the bump happens only at release time (step above).

Documentation-only commits (`*.md`, `tutorial/`, `docs/`) and
compiled-output refreshes (`.bas` files) do not require a version bump.
