#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
work_dir=$(mktemp -d "${TMPDIR:-/tmp}/rdgen-bascal-probe.XXXXXX")
trap 'rm -rf "$work_dir"' EXIT

cargo run --offline --manifest-path "$repo_root/Cargo.toml" -p rdgen -- \
    "$repo_root/rdgen/grammars/bascal.bcl.rdg" --emit-rust >"$work_dir/generated.rs"
RDGEN_GENERATED="$work_dir/generated.rs" \
    rustc --crate-name rdgen_bascal_probe "$repo_root/rdgen/scripts/bascal_probe.rs" \
    -o "$work_dir/probe"

mapfile -d '' files < <(
    find "$repo_root/tutorial" "$repo_root/examples" "$repo_root/tests/fixtures" \
        -type f -name '*.bcl' -print0 | sort -z
)
"$work_dir/probe" "${files[@]}"
