#!/usr/bin/env python3
"""Embeds full tutorial source files into docs/tutorials/*.html.

Each tutorial page ends with a "Full, real, transpiling source: ..."
paragraph linking out to the .bcl/.bas files on GitHub. This script reads
that paragraph's own links to find which tutorial/ files a page covers,
inlines each file's full contents as a collapsible, syntax-highlighted
<pre><code> block (bcl-highlight.js -- already loaded by every tutorial
page -- highlights it client-side, same as the hand-picked snippets
above it), and replaces the GitHub-only links with real embedded source
plus a "View on GitHub" line for permalinking/blame.

Idempotent: re-running it re-reads tutorial/ and refreshes the embedded
block between its own BEGIN/END markers, so it's safe (and expected) to
run again whenever a tutorial's .bcl/.bas changes.

Usage: scripts/embed-tutorial-sources.py
"""
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
TUTORIALS_DIR = REPO_ROOT / "docs" / "tutorials"
GITHUB_BLOB = "https://github.com/johnjoeallen/bascal/blob/main/"

BEGIN_MARKER = "<!-- BEGIN generated source embed (scripts/embed-tutorial-sources.py) -->"
END_MARKER = "<!-- END generated source embed -->"

LEDE_RE = re.compile(
    r'<p class="lede">\s*Full, real, transpiling source:.*?</p>', re.DOTALL
)
GENERATED_RE = re.compile(
    re.escape(BEGIN_MARKER) + r".*?" + re.escape(END_MARKER), re.DOTALL
)
LINK_RE = re.compile(r'href="' + re.escape(GITHUB_BLOB) + r'(tutorial/[^"]+)"')


def escape_html(text: str) -> str:
    return text.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")


def build_embed(paths: list[str]) -> str:
    parts = [BEGIN_MARKER, '  <div class="source-embed">']
    for rel_path in paths:
        file_path = REPO_ROOT / rel_path
        source = file_path.read_text()
        parts.append(
            f'    <details>\n'
            f'      <summary><code>{rel_path}</code></summary>\n'
            f'      <pre><code>{escape_html(source)}</code></pre>\n'
            f'    </details>'
        )
    links = ", ".join(
        f'<a href="{GITHUB_BLOB}{p}">{p}</a>' for p in paths
    )
    parts.append(f'    <p class="lede">View on GitHub: {links}.</p>')
    parts.append("  </div>")
    parts.append(END_MARKER)
    return "\n".join(parts)


def process(html_path: Path) -> bool:
    text = html_path.read_text()

    existing = GENERATED_RE.search(text)
    if existing:
        paths = LINK_RE.findall(existing.group(0))
        if not paths:
            return False
        new_embed = build_embed(paths)
        new_text = text[: existing.start()] + new_embed + text[existing.end() :]
    else:
        lede = LEDE_RE.search(text)
        if not lede:
            return False
        paths = LINK_RE.findall(lede.group(0))
        if not paths:
            return False
        new_embed = build_embed(paths)
        new_text = text[: lede.start()] + new_embed + text[lede.end() :]

    if new_text == text:
        return False
    html_path.write_text(new_text)
    return True


def main() -> int:
    changed = 0
    for html_path in sorted(TUTORIALS_DIR.glob("*.html")):
        if html_path.name == "index.html":
            continue
        if process(html_path):
            changed += 1
            print(f"embedded source into {html_path.relative_to(REPO_ROOT)}")
    print(f"done: {changed} page(s) updated")
    return 0


if __name__ == "__main__":
    sys.exit(main())
