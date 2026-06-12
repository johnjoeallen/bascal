# BASCAL — Agent Instructions

## Version bumping

Every commit that changes source code (`src/`) or tests (`tests/`, `src/lib.rs`)
must be preceded by a patch-level version bump:

1. Increment the patch number in `Cargo.toml` (e.g. `0.99.2` → `0.99.3`).
2. Run `cargo build -q` to propagate the change into `Cargo.lock`.
3. Stage both `Cargo.toml` and `Cargo.lock` and include them in the same commit
   as the code change (not a separate commit).

After committing, run `cargo test`. If all tests pass, create an annotated tag
matching the new version:

```
git tag -a v<new-version> -m "Release v<new-version>"
```

If any test fails, fix the failure before tagging — do not tag a broken build.

Documentation-only commits (`MANUAL.md`, `*.md`, `tutorial/`) and
compiled-output refreshes (`.bas` files) do **not** require a version bump or
a new tag.
