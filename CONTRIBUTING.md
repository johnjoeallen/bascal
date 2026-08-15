# Contributing to BASCAL

## Building and testing

```bash
cargo build
env -u RUSTC_WRAPPER cargo test
```

See `README.md`'s [Tests](README.md#tests) section for what the standard
suite covers.

## Optional: real-BASCOM conformance tests

BASCAL targets classic Microsoft BASIC dialects (MBASIC, BASCOM, GW-BASIC),
and most of the test suite checks that BASCAL's own understanding of that
target is internally consistent. `tests/dosbox_conformance.rs` goes a step
further: it compiles BASCAL's generated `.bas` output against a **real**
1986 IBM Personal Computer BASIC Compiler 2.00 ("BASCOM"), running under
[dosbox-x](https://dosbox-x.com/), and diffs the result against checked-in
golden output. This is how several real incompatibilities between BASCAL's
generated code and actual period compilers were originally found and fixed
(see `MANUAL.md`'s notes on `CONST`, `MKx$`/`CVx`, and identifier naming).

This suite is **entirely optional and does not run by default** -- neither
locally nor in CI. It requires two things that BASCAL does not, and will
not, install or fetch on your behalf:

1. **dosbox-x itself.** Install it for your platform following its own
   [download/install instructions](https://dosbox-x.com/wiki/Download-and-Installation).
2. **The BASCOM 2.00 compiler**, which is copyrighted IBM/Microsoft
   software and is therefore never committed to this repository. Pull it
   into `test-fixtures/ibm-basic-compiler/` (gitignored) by running:

   ```bash
   scripts/fetch-ibm-basic-compiler.sh
   ```

   This is a deliberate, manual step -- it is not run automatically by
   `git clone`, `cargo build`, or `cargo test`. See
   `test-fixtures/README.md` for exactly what it fetches, from where, and
   why it's structured this way.

Once both are present, `cargo test` picks up `tests/dosbox_conformance.rs`
automatically and runs it alongside everything else. If either is missing,
those tests print a one-line message pointing back here and skip cleanly --
the build never fails because of an absent optional fixture.

### Running it in CI

GitHub Actions does not fetch this fixture or run these tests as part of
the default workflow, for the same copyright reasons it isn't committed.
If you want to run the conformance suite in CI (e.g. before a release), add
a separate `workflow_dispatch`-triggered job that installs `dosbox-x` and
`mtools`, runs `scripts/fetch-ibm-basic-compiler.sh`, and then runs
`cargo test`. Do not wire it into the default `push`/`pull_request`
triggers that every contributor's PR runs through.

## Adding a new conformance fixture

Fixtures live in `tests/fixtures/conformance/*.bcl`, each paired with a
`*.expected.txt` file holding the exact stdout the compiled program should
produce when run under real BASCOM. Keep fixtures small, deterministic (no
`TIMER`-seeded randomness, no wall-clock-dependent output), and focused on
one behavior, the same way the existing fixture is scoped.
