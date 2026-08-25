#!/usr/bin/env bash
# Builds bcc and installs it under ~/.local, laid out the same way
# scripts/fetch-krak2.sh installs krak2 (and the way this machine's other
# debkit-installed tools already work): a versioned build under
# ~/.local/share/bascal-versions/<version>/, a "current" symlink pointing
# at it, and ~/.local/bin/bcc symlinked through that.
#
# By default this builds from the repo checkout this script lives in (its
# own working tree, uncommitted changes included -- handy right after
# building/testing locally). Pass --pull [ref] to instead clone fresh from
# GitHub at the given ref (default: main) into a scratch directory and
# build that -- for installing on a machine with no local checkout, or to
# be sure of building exactly what's on GitHub rather than a possibly
# locally-modified working tree.
#
# The installed layout follows lib.rs's own stdlib_search_roots FHS
# convention (see its doc comment): bcc looks for com/ at ../share/bascal
# relative to its own binary's directory, so this installs to
# <version>/bin/bcc plus <version>/share/bascal/com/, not just the bare
# binary -- a bcc that can't find com.bascal.stdlib.* would fail on
# anything using the standard library.
#
# Requires: cargo (any toolchain able to build this repo); git only for
# --pull.
set -euo pipefail

REPO_URL="https://github.com/johnjoeallen/bascal.git"
PULL=0
REF="main"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --pull)
      PULL=1
      if [[ $# -ge 2 && "$2" != --* ]]; then
        REF="$2"
        shift
      fi
      shift
      ;;
    --force)
      FORCE=1
      shift
      ;;
    *)
      echo "usage: $0 [--pull [ref]] [--force]" >&2
      exit 2
      ;;
  esac
done
FORCE="${FORCE:-0}"

INSTALL_ROOT="$HOME/.local/share/bascal-versions"
CURRENT_LINK="$HOME/.local/share/bascal-versions/current"
BIN_DIR="$HOME/.local/bin"
BIN_LINK="$BIN_DIR/bcc"

log() { echo "[install-bcc] $*"; }
die() { echo "[install-bcc] error: $*" >&2; exit 1; }

command -v cargo >/dev/null 2>&1 || die "'cargo' is required but not found on PATH"

CLEANUP_DIR=""
cleanup() { [[ -n "$CLEANUP_DIR" ]] && rm -rf "$CLEANUP_DIR"; return 0; }
trap cleanup EXIT

if [[ "$PULL" -eq 1 ]]; then
  command -v git >/dev/null 2>&1 || die "'git' is required for --pull but not found on PATH"
  CLEANUP_DIR="$(mktemp -d)"
  log "cloning $REPO_URL @ $REF..."
  git clone --branch "$REF" --depth 1 --quiet "$REPO_URL" "$CLEANUP_DIR/bascal" \
    || die "failed to clone $REPO_URL @ $REF"
  SOURCE_DIR="$CLEANUP_DIR/bascal"
else
  SOURCE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
  [[ -f "$SOURCE_DIR/Cargo.toml" && -d "$SOURCE_DIR/com/bascal" ]] \
    || die "$SOURCE_DIR doesn't look like a bascal checkout (no Cargo.toml/com/bascal) -- use --pull instead"
  log "building from local checkout at $SOURCE_DIR (pass --pull to build from GitHub instead)"
fi

VERSION="$(git -C "$SOURCE_DIR" describe --always --dirty 2>/dev/null || echo unknown)"
VERSION_DIR="$INSTALL_ROOT/$VERSION"

if [[ -x "$VERSION_DIR/bin/bcc" && "$FORCE" -ne 1 ]]; then
  log "already built at $VERSION_DIR -- nothing to build"
  log "(pass --force, or delete $VERSION_DIR, to force a rebuild)"
else
  log "building (cargo build --release) at $VERSION..."
  ( cd "$SOURCE_DIR" && env -u RUSTC_WRAPPER cargo build --release --quiet ) \
    || die "cargo build failed"

  BUILT_BIN="$SOURCE_DIR/target/release/bcc"
  [[ -x "$BUILT_BIN" ]] || die "build succeeded but $BUILT_BIN is missing"

  log "installing to $VERSION_DIR..."
  rm -rf "$VERSION_DIR"
  mkdir -p "$VERSION_DIR/bin" "$VERSION_DIR/share/bascal"
  install -m 755 "$BUILT_BIN" "$VERSION_DIR/bin/bcc"
  cp -r "$SOURCE_DIR/com" "$VERSION_DIR/share/bascal/com"
fi

log "pointing $CURRENT_LINK -> $VERSION_DIR..."
ln -sfn "$VERSION_DIR" "$CURRENT_LINK"

mkdir -p "$BIN_DIR"
log "pointing $BIN_LINK -> $CURRENT_LINK/bin/bcc..."
ln -sfn "$CURRENT_LINK/bin/bcc" "$BIN_LINK"

if ! command -v bcc >/dev/null 2>&1; then
  log "done, but $BIN_DIR isn't on PATH in this shell -- add it to use 'bcc' directly, or run" \
      "$BIN_LINK"
else
  log "done: $(command -v bcc) -> $(readlink -f "$BIN_LINK") ($VERSION)"
fi
