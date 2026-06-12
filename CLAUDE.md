# BASCAL — Agent Instructions

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

Documentation-only commits (`MANUAL.md`, `*.md`, `tutorial/`) and
compiled-output refreshes (`.bas` files) do not require a version bump.
