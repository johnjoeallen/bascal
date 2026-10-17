# Invalid-program corpus

Programs the compiler must reject (or, for the odd control, accept) with a
useful diagnostic. `invalid_program_corpus_reports_the_expected_diagnostics`
in `src/driver.rs` compiles every `*.bcl` here for the BASIC, C and JVM
targets through `compile_file` and checks it against `<name>.expected`.

## Adding a case

1. Add `<name>.bcl`. It must start with `program p` like any root file, and
   should be minimal and about one mistake.
2. Add `<name>.expected` with one line per expectation:

   ```
   all: expects 2 argument(s), got 1        # every target must fail with this
   basic: ok                                 # this target must accept it
   c: file channel #99 is out of range
   ```

   - The target is `basic`, `c`, `jvm` or `all`; a specific target overrides
     `all`, and all three targets need an expectation.
   - `ok` means the target compiles it. Anything else is a substring that some
     diagnostic must contain.
   - Blank lines and lines starting with `#` are ignored.
3. `cargo test --lib invalid_program_corpus`. To see what the compiler
   currently says for every file, run
   `BAD_DIR=$PWD/tests/fixtures/invalid cargo test --lib show_invalid_program_diagnostics -- --ignored --nocapture`.

## Known gaps

A `# gap:` comment marks an expectation that records a weakness rather than a
goal: a target that only says "X is not supported by the minimal ... backend
yet" where a specific message would help, or an invalid program nothing rejects
yet. When you improve one, update its `.expected` in the same change. The
suite is deliberately a ratchet: a diagnostic getting worse fails it.

The files in this directory are excluded from the valid-program corpora
(`corpus_programs` in `src/driver.rs` and the rdgen parser probe).
