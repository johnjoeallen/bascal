#!/usr/bin/env bash
set -euo pipefail

# Keep tutorial source and generated backend output in sync before MkDocs
# embeds them in the tutorial pages.
env -u RUSTC_WRAPPER cargo build --quiet
python3 scripts/embed_tutorial_markdown.py
mkdocs build --strict
