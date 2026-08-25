#!/usr/bin/env bash
# Builds krak2, the assembler for Storyyeller/Krakatau's v2 (Rust) branch,
# and installs it under ~/.local the same way debkit-installed tools on this
# machine are laid out: a versioned build under ~/.local/share/krakatau/,
# a "current" symlink pointing at it, and ~/.local/bin/krak2 symlinked
# through that -- so re-running this script after bumping KRAK2_COMMIT
# upgrades cleanly and old versions stay on disk until removed by hand.
#
# krak2 assembles the Krakatau assembly text (.j) bcc's --target jvm backend
# (codegen_jvm.rs) emits into a real .class file; main.rs's invoke_krak2
# looks for it on PATH by default, so once ~/.local/bin is on PATH (it
# already is on this machine, going by the other tools installed there) no
# further configuration is needed. It can also be pointed at an install
# anywhere else via krak2=/path/to/krak2 in ~/.config/bascal/config or the
# BASCAL_KRAK2 env var -- see resolve_krak2_path in main.rs.
#
# Krakatau's v2 branch has no versioned releases, so this pins an exact,
# already-verified-working commit rather than tracking the branch tip.
#
# Requires: git, cargo (any toolchain able to build the pinned commit).
set -euo pipefail

KRAK2_REPO="https://github.com/Storyyeller/Krakatau.git"
KRAK2_REF="v2"
KRAK2_COMMIT="ae49743af92bbab1684b19bb1bee22b0a64d8ce5"

INSTALL_ROOT="$HOME/.local/share/krakatau"
VERSION_DIR="$INSTALL_ROOT/$KRAK2_COMMIT"
CURRENT_LINK="$INSTALL_ROOT/current"
BIN_DIR="$HOME/.local/bin"
BIN_LINK="$BIN_DIR/krak2"

log() { echo "[fetch-krak2] $*"; }
die() { echo "[fetch-krak2] error: $*" >&2; exit 1; }

already_built() {
  [[ -x "$VERSION_DIR/krak2" ]]
}

if already_built; then
  log "already built at $VERSION_DIR -- nothing to build"
  log "(delete $VERSION_DIR and re-run this script to force a rebuild)"
else
  for tool in git cargo; do
    command -v "$tool" >/dev/null 2>&1 || die "'$tool' is required but not found on PATH"
  done

  BUILD_DIR="$(mktemp -d)"
  trap 'rm -rf "$BUILD_DIR"' EXIT

  log "cloning $KRAK2_REPO @ $KRAK2_REF (pinned at $KRAK2_COMMIT)..."
  git clone --branch "$KRAK2_REF" --quiet "$KRAK2_REPO" "$BUILD_DIR/Krakatau"
  git -C "$BUILD_DIR/Krakatau" checkout --quiet "$KRAK2_COMMIT" \
    || die "pinned commit $KRAK2_COMMIT not found on $KRAK2_REF -- has it been rewritten upstream?"

  log "building (cargo build --release)..."
  ( cd "$BUILD_DIR/Krakatau" && env -u RUSTC_WRAPPER cargo build --release --quiet ) \
    || die "cargo build failed"

  BUILT_BIN="$BUILD_DIR/Krakatau/target/release/krak2"
  [[ -x "$BUILT_BIN" ]] || die "build succeeded but $BUILT_BIN is missing -- unexpected layout upstream?"

  log "installing to $VERSION_DIR..."
  mkdir -p "$VERSION_DIR"
  install -m 755 "$BUILT_BIN" "$VERSION_DIR/krak2"
fi

log "pointing $CURRENT_LINK -> $VERSION_DIR..."
ln -sfn "$VERSION_DIR" "$CURRENT_LINK"

mkdir -p "$BIN_DIR"
log "pointing $BIN_LINK -> $CURRENT_LINK/krak2..."
ln -sfn "$CURRENT_LINK/krak2" "$BIN_LINK"

if ! command -v krak2 >/dev/null 2>&1; then
  log "done, but $BIN_DIR isn't on PATH in this shell -- add it, or set" \
      "krak2=$BIN_LINK in ~/.config/bascal/config (see main.rs's resolve_krak2_path)"
else
  log "done: $(command -v krak2) -> $(readlink -f "$BIN_LINK")"
fi
