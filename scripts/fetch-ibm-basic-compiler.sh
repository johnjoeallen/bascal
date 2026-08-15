#!/usr/bin/env bash
# Fetches the IBM Personal Computer BASIC Compiler 2.00 ("BASCOM") from the
# PCjs software archive and unpacks it into test-fixtures/ibm-basic-compiler/
# so the dosbox-x conformance test suite (tests/dosbox_conformance.rs) can
# compile BASCAL's generated BASIC against the real, period-accurate
# compiler.
#
# This compiler is copyrighted IBM/Microsoft software. It is never committed
# to this repository -- see test-fixtures/README.md for why. Run this script
# manually, once, to opt in to the conformance suite locally.
#
# Requires: curl, python3, mtools (mcopy).
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEST_DIR="$REPO_ROOT/test-fixtures/ibm-basic-compiler"
JSON_URL="https://raw.githubusercontent.com/jeffpar/pcjs-miscdisks/main/pcx86/lang/ibm/basic/compiler/2.00/BASCOM200-DISK1.json"

# Published by PCjs itself, inside BASCOM200-DISK1.json's own `imageInfo.hash`
# field -- this is PCjs's reference checksum for the reconstructed disk
# image, not one we computed ourselves. Hardcoded here (rather than trusted
# blindly from whatever the script just downloaded) so a corrupted or
# tampered download is caught instead of silently accepted.
EXPECTED_IMG_MD5="cdc2044fab51d33a65d17afa98011eb8"
EXPECTED_IMG_SIZE=368640

JSON_PATH="$DEST_DIR/BASCOM200-DISK1.json"
IMG_PATH="$DEST_DIR/BASCOM200-DISK1.img"
C_DRIVE_DIR="$DEST_DIR/c_drive"

# Files the conformance suite actually needs mounted as drive C: in dosbox-x.
FILES_TO_EXTRACT=(
  BASCOM.EXE
  BASCOM20.LIB
  BASCOM20.PIF
  BASRUN20.EXE
  BASRUN20.LIB
  IBMCAS.OBJ
  IBMCOM.OBJ
  LINK.EXE
)

log() { echo "[fetch-ibm-basic-compiler] $*"; }
die() { echo "[fetch-ibm-basic-compiler] error: $*" >&2; exit 1; }

already_fetched() {
  [[ -f "$IMG_PATH" ]] || return 1
  [[ -f "$C_DRIVE_DIR/BASCOM.EXE" ]] || return 1
  [[ "$(md5sum "$IMG_PATH" | cut -d' ' -f1)" == "$EXPECTED_IMG_MD5" ]]
}

if already_fetched; then
  log "already present and verified at $DEST_DIR -- nothing to do"
  log "(delete $DEST_DIR and re-run this script to force a re-fetch)"
  exit 0
fi

for tool in curl python3 mcopy; do
  command -v "$tool" >/dev/null 2>&1 || die "'$tool' is required but not found on PATH"
done

mkdir -p "$DEST_DIR"

log "downloading disk image JSON from PCjs..."
curl -fsSL "$JSON_URL" -o "$JSON_PATH.tmp"
mv "$JSON_PATH.tmp" "$JSON_PATH"

log "reconstructing raw floppy image from PCjs's JSON encoding..."
CONVERTER="$REPO_ROOT/scripts/lib/pcjs_disk_to_img.py"
CONVERT_OUTPUT="$(python3 "$CONVERTER" "$JSON_PATH" "$IMG_PATH.tmp")" \
  || die "failed to reconstruct disk image from $JSON_PATH"

PUBLISHED_HASH="$(cut -d' ' -f1 <<<"$CONVERT_OUTPUT")"
PUBLISHED_SIZE="$(cut -d' ' -f2 <<<"$CONVERT_OUTPUT")"

if [[ "$PUBLISHED_HASH" != "$EXPECTED_IMG_MD5" || "$PUBLISHED_SIZE" != "$EXPECTED_IMG_SIZE" ]]; then
  rm -f "$IMG_PATH.tmp"
  die "PCjs's own published hash/size ($PUBLISHED_HASH, $PUBLISHED_SIZE bytes) no longer" \
      " matches what this script expects ($EXPECTED_IMG_MD5, $EXPECTED_IMG_SIZE bytes)." \
      " The upstream disk image may have changed -- update EXPECTED_IMG_MD5/EXPECTED_IMG_SIZE" \
      " in this script after manually verifying the new image, or investigate before trusting it."
fi

ACTUAL_MD5="$(md5sum "$IMG_PATH.tmp" | cut -d' ' -f1)"
if [[ "$ACTUAL_MD5" != "$EXPECTED_IMG_MD5" ]]; then
  rm -f "$IMG_PATH.tmp"
  die "reconstructed image md5 ($ACTUAL_MD5) does not match the expected reference" \
      " checksum ($EXPECTED_IMG_MD5) -- refusing to use it"
fi

mv "$IMG_PATH.tmp" "$IMG_PATH"
log "verified: $IMG_PATH matches PCjs's published checksum"

log "extracting compiler files into $C_DRIVE_DIR..."
rm -rf "$C_DRIVE_DIR"
mkdir -p "$C_DRIVE_DIR"
for file in "${FILES_TO_EXTRACT[@]}"; do
  mcopy -n -i "$IMG_PATH" "::$file" "$C_DRIVE_DIR/" \
    || die "failed to extract $file from $IMG_PATH"
done

log "done. IBM BASIC Compiler 2.00 is ready at $C_DRIVE_DIR"
log "the dosbox-x conformance suite (tests/dosbox_conformance.rs) will now run instead of skipping."
