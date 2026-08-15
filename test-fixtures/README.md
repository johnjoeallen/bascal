# test-fixtures/

This directory holds third-party test fixtures that support BASCAL's test
suite but are **never committed to this repository**.

## ibm-basic-compiler/

`test-fixtures/ibm-basic-compiler/` holds a local copy of the **IBM Personal
Computer BASIC Compiler 2.00** ("BASCOM"), a real, period-accurate 1986
MBASIC-family compiler. `tests/dosbox_conformance.rs` uses it to compile
BASCAL's generated `.bas` output under [dosbox-x](https://dosbox-x.com/) and
check the result against the genuine compiler, not just BASCAL's own idea of
what valid MBASIC/BASCOM looks like.

**This directory is gitignored and must never be committed.** The compiler
is copyrighted IBM/Microsoft software, sourced from the
[PCjs software archive](https://www.pcjs.org/software/pcx86/lang/ibm/basic/compiler/2.00/)
(specifically its
[pcjs-miscdisks](https://github.com/jeffpar/pcjs-miscdisks) repository, which
hosts it for use with PCjs's own PC emulator). PCjs's own hosting of it
doesn't make it open source or ours to redistribute -- so it stays local-only
by design, fetched fresh by each developer or CI run that wants it, never
checked into BASCAL's history.

### Setting it up

1. Install [dosbox-x](https://dosbox-x.com/) yourself first (see its
   [download/install docs](https://dosbox-x.com/wiki/Download-and-Installation)
   for your platform). This is a manual, one-time step -- BASCAL's build
   does not install it for you.
2. From the repo root, run:

   ```
   scripts/fetch-ibm-basic-compiler.sh
   ```

   This downloads BASCOM 2.00's disk image from PCjs, reconstructs it from
   PCjs's JSON-wrapped disk format into a raw floppy image, verifies it
   against the checksum PCjs itself publishes for that image (not a
   checksum BASCAL invented), and extracts the compiler files BASCAL's
   tests need into `test-fixtures/ibm-basic-compiler/c_drive/`.

   Requires `curl`, `python3`, and `mtools` (for `mcopy`) on your `PATH`.
   The script is idempotent -- re-running it is a no-op if the fixture is
   already present and verified; delete the directory first to force a
   re-fetch.

Once both are in place, `cargo test` automatically picks up and runs the
dosbox-x conformance tests in `tests/dosbox_conformance.rs` alongside the
rest of the suite. **This is entirely opt-in and local by default** -- if
either dosbox-x or the fixture is missing, those tests print a short message
explaining how to enable them and skip cleanly, without failing the build.
CI does not fetch this fixture or run these tests by default either; see
`CONTRIBUTING.md` for how to run them as an opt-in CI job.

### Why not fetch it automatically?

Pulling and running copyrighted third-party software as a side effect of
`git clone`, `cargo build`, or an unattended CI job isn't something BASCAL
does silently. Running `scripts/fetch-ibm-basic-compiler.sh` is a deliberate
choice a developer (or a CI job specifically configured to opt in) makes,
not a default.
