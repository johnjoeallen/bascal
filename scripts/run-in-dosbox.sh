#!/usr/bin/env bash
# Compiles a .bcl file with bcc, then launches an *interactive* dosbox-x
# session that compiles the generated BASIC with the real IBM Personal
# Computer BASIC Compiler 2.00 ("BASCOM") and runs the resulting .EXE --
# so you can actually use the program, unlike tests/dosbox_conformance.rs
# which runs headlessly and just diffs captured output.
#
# Requires the same setup as that conformance suite:
#   1. dosbox-x on PATH.
#   2. test-fixtures/ibm-basic-compiler/c_drive/ populated by running
#      scripts/fetch-ibm-basic-compiler.sh (see test-fixtures/README.md).
#
# Usage: scripts/run-in-dosbox.sh path/to/program.bcl
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FIXTURE_DIR="$REPO_ROOT/test-fixtures/ibm-basic-compiler/c_drive"

log() { echo "[run-in-dosbox] $*"; }
die() { echo "[run-in-dosbox] error: $*" >&2; exit 1; }

[[ $# -eq 1 ]] || die "usage: $0 path/to/program.bcl"
BCL_PATH="$1"
[[ -f "$BCL_PATH" ]] || die "no such file: $BCL_PATH"

command -v dosbox-x >/dev/null 2>&1 \
  || die "dosbox-x not found on PATH -- see CONTRIBUTING.md"
[[ -f "$FIXTURE_DIR/BASCOM.EXE" ]] \
  || die "test-fixtures/ibm-basic-compiler/ not populated -- run scripts/fetch-ibm-basic-compiler.sh first"
command -v python3 >/dev/null 2>&1 || die "'python3' is required but not found on PATH"

log "building bcc..."
cargo build -q --release --bin bcc
BCC_BIN="$REPO_ROOT/target/release/bcc"

STEM="$(basename "$BCL_PATH" .bcl)"
WORK_DIR="${TMPDIR:-/tmp}/bascal-dosbox-run-$STEM"
rm -rf "$WORK_DIR"
mkdir -p "$WORK_DIR"

log "compiling $BCL_PATH..."
BAS_PATH="$WORK_DIR/${STEM}.bas"
"$BCC_BIN" "$BCL_PATH" -o "$BAS_PATH"

log "staging BASCOM/LINK fixture into $WORK_DIR..."
cp "$FIXTURE_DIR"/* "$WORK_DIR/"

# DOS text needs CRLF line endings and a trailing Ctrl-Z (0x1A) EOF marker
# to be read correctly by real DOS-era tools -- see to_dos_text() in
# tests/dosbox_conformance.rs, which this mirrors.
DOS_STEM="RUN"
DOS_BAS="$WORK_DIR/${DOS_STEM}.BAS"
sed 's/$/\r/' "$BAS_PATH" > "$DOS_BAS"
printf '\x1a' >> "$DOS_BAS"

# BASCOM doesn't link in ON ERROR GOTO / RESUME support by default --
# the /E (error trapping) and /X (RESUME) switches must be requested
# explicitly, or compilation/linking of such a program fails. Discovered
# on tutorial/inventory.bcl; see its header note and `errorTrap()`.
BASCOM_SWITCHES=""
if grep -qi 'ON ERROR GOTO' "$BAS_PATH"; then
    BASCOM_SWITCHES="/E/X"
    log "program uses ON ERROR GOTO -- compiling with BASCOM $BASCOM_SWITCHES"
fi

# tutorial/inventory.bcl's header note explains that fhb's original
# one-time "hidden" datafile initializer (PUT-ing 100 blank,
# CHR$(255)-flagged records) has no BASCAL equivalent and isn't
# reproduced -- inven.dat must already contain 100 such blank records
# before the program runs, or isEmpty%() reads uninitialized/zero-filled
# records as never-empty. Recognize that exact random-access layout
# (LEN = 39: 1-byte flag + 30-byte desc + 2+2+4-byte numeric fields) and
# seed it here instead. Any other random-access file this doesn't
# recognize is left for you to populate by hand in $WORK_DIR.
RANDOM_FILE="$(grep -oP 'OPEN "\K[^"]+(?=" FOR RANDOM)' "$BAS_PATH" | head -1 || true)"
RANDOM_LEN="$(grep -oP 'FOR RANDOM AS #\d+ LEN = \K\d+' "$BAS_PATH" | head -1 || true)"
if [[ "$RANDOM_FILE" == "inven.dat" && "$RANDOM_LEN" == "39" ]]; then
    log "pre-populating $RANDOM_FILE with 100 blank (CHR\$(255)-flagged) records"
    python3 -c "
with open('$WORK_DIR/$RANDOM_FILE', 'wb') as f:
    f.write((b'\xff' + b' ' * 30 + b'\x00' * 8) * 100)
"
elif [[ -n "$RANDOM_FILE" ]]; then
    log "note: $BCL_PATH opens random-access file '$RANDOM_FILE' (LEN=$RANDOM_LEN)" \
        "-- if it needs pre-existing data, populate $WORK_DIR/$RANDOM_FILE yourself" \
        "before continuing in dosbox-x"
fi

cat > "$WORK_DIR/RUN.BAT" <<EOF
BASCOM ${DOS_STEM}.BAS${BASCOM_SWITCHES},,;
LINK ${DOS_STEM}.OBJ;
${DOS_STEM}.EXE
EOF
sed -i 's/$/\r/' "$WORK_DIR/RUN.BAT"

log "launching dosbox-x (compiles with real BASCOM, links, then runs the .EXE)..."
log "close the dosbox-x window (or exit back to the C:\\> prompt) when you're done"
dosbox-x -c "MOUNT C: $WORK_DIR" -c "C:" -c "RUN.BAT"

log "session ended -- work directory left at $WORK_DIR for inspection"
