#!/usr/bin/env python3
"""One-off script: adds a top nav (mirroring the existing bottom one) to
every docs/manual/*.html chapter page, and restyles both as a flex row with
previous pinned left / next pinned right, instead of plain inline text.
Reads each chapter's *existing* bottom nav block to recover prev/next
href+title (no need to go back to the original docs/manual.html, which no
longer exists -- it's a redirect stub now). docs/manual/index.html is
untouched, per its own request to skip it. Not meant to be re-run
routinely.
"""
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MANUAL_DIR = ROOT / "docs" / "manual"

OLD_BOTTOM_RE = re.compile(
    r'\n  <p class="crumbs">\n(.*?)\n  </p>\n</main>', re.S
)
LINK_RE = re.compile(r'<a href="([^"]+)">(.*?)</a>')


def build_nav(links: list[str]) -> str:
    prev_html = ""
    next_html = ""
    for link in links:
        m = LINK_RE.search(link)
        if not m:
            continue
        href, text = m.group(1), m.group(2)
        if text.startswith("&larr;"):
            prev_html = f'<a href="{href}">{text}</a>'
        elif text.endswith("&rarr;"):
            next_html = f'<a href="{href}">{text}</a>'
    prev_slot = prev_html or "<span></span>"
    next_slot = next_html or "<span></span>"
    return f'  <nav class="chapter-nav">\n    {prev_slot}\n    {next_slot}\n  </nav>'


changed = 0
for path in sorted(MANUAL_DIR.glob("*.html")):
    if path.name == "index.html":
        continue
    text = path.read_text()

    m = OLD_BOTTOM_RE.search(text)
    if not m:
        raise SystemExit(f"{path}: couldn't find the expected bottom nav block")
    inner = m.group(1)
    links = [line.strip() for line in inner.splitlines() if line.strip()]
    nav_html = build_nav(links)

    # Replace the old bottom `<p class="crumbs">...</p>` block with the new nav.
    new_text = text[: m.start()] + "\n" + nav_html + "\n</main>" + text[m.end():]

    # Insert the same nav right after the top breadcrumb, before <div class="prose">.
    new_text = new_text.replace(
        '\n\n  <div class="prose">',
        f"\n{nav_html}\n\n  <div class=\"prose\">",
        1,
    )

    if new_text != text:
        path.write_text(new_text)
        changed += 1

print(f"updated {changed} chapter pages")
