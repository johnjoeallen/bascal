#!/usr/bin/env python3
"""One-off script: splits docs/manual.html (a single giant page) into
per-chapter pages under docs/manual/, matching the docs/tutorials/ page
template. Rewrites in-body #anchor cross-references to point at the right
chapter page when the anchor now lives on a different page. Not meant to be
re-run routinely -- MANUAL.md -> docs/manual.html regeneration tooling
(pandoc, presumably) would need to target docs/manual/*.html chapter-by-
chapter to replace this permanently; this script is the one-time migration.
"""
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / "docs" / "manual.html"
OUT_DIR = ROOT / "docs" / "manual"

text = SRC.read_text()
lines = text.splitlines(keepends=True)

# Locate every top-level `<h2 id="...">Title</h2>` line -- these are the
# chapter boundaries. The very first one is "table-of-contents", which
# becomes the index page instead of a chapter.
h2_re = re.compile(r'<h2 id="([^"]+)">(.*?)</h2>')
chapter_starts = []  # (line_index, id, title_html)
for i, line in enumerate(lines):
    m = h2_re.search(line)
    if m:
        chapter_starts.append((i, m.group(1), m.group(2)))

assert chapter_starts[0][1] == "table-of-contents"
chapters = chapter_starts[1:]  # drop the TOC itself

# Body ends at the closing </div> of <div class="prose"> (right before </main>).
body_end = next(i for i, line in enumerate(lines) if line.strip() == "</main>") - 1
while lines[body_end].strip() != "</div>":
    body_end -= 1

# Slice out each chapter's raw HTML (from its own <h2> line up to, but not
# including, the next chapter's <h2> line, or body_end for the last one).
sections = {}
for idx, (start, cid, title) in enumerate(chapters):
    end = chapters[idx + 1][0] if idx + 1 < len(chapters) else body_end
    sections[cid] = {
        "title": title,
        # start+1: drops the chapter's own `<h2 id="...">Title</h2>` line --
        # the page template already shows the title once, as the hero <h1>;
        # the anchor itself is preserved by putting id="{cid}" on <main>
        # instead (see PAGE_TEMPLATE), so cross-chapter links to a whole
        # chapter (not a subsection within it) still resolve to something.
        "html": "".join(lines[start + 1:end]),
        "order": idx,
    }
    # Pandoc's `<hr />` was a between-sections divider in the single-page
    # manual; as the last thing on a standalone chapter page it just dangles
    # with nothing after it, so drop one trailing occurrence per chapter.
    sections[cid]["html"] = re.sub(r"<hr />\n$", "", sections[cid]["html"], count=1)

# Build a full anchor -> chapter map, covering every `id="..."` in each
# chapter's HTML (h2/h3/h4/whatever), not just the top-level h2 ids --
# in-body links like #exit or #byref--byval target a subsection, not a
# whole chapter, but still need to resolve to the right *page*.
id_re = re.compile(r'id="([^"]+)"')
# Seed with each chapter's own top-level id first -- its own `<h2 id="...">`
# line was stripped out of `sec["html"]` above (see the comment there), so
# the id_re scan below would otherwise never find it.
anchor_to_chapter = {cid: cid for cid in sections}
for cid, sec in sections.items():
    for m in id_re.finditer(sec["html"]):
        anchor_to_chapter[m.group(1)] = cid

href_re = re.compile(r'href="#([^"]+)"')


def rewrite_hrefs(html: str, current_chapter: str) -> str:
    def repl(m):
        anchor = m.group(1)
        target_chapter = anchor_to_chapter.get(anchor)
        if target_chapter is None or target_chapter == current_chapter:
            return f'href="#{anchor}"'
        return f'href="{target_chapter}.html#{anchor}"'

    return href_re.sub(repl, html)


def strip_tags(html: str) -> str:
    return re.sub(r"<[^>]+>", "", html)


PAGE_TEMPLATE = """<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title} — BASCAL Manual</title>
<meta name="description" content="{description}">
<link rel="icon" href="../favicon.svg" type="image/svg+xml">
<link rel="stylesheet" href="../style.css">
<script>try{{var t=localStorage.getItem("bascal-theme");if(t)document.documentElement.setAttribute("data-theme",t);}}catch(e){{}}</script>
</head>
<body>

<header class="hero compact">
  <div class="wrap">
    <h1>{title}</h1>
    <p class="tagline">BASCAL Language Manual, chapter {num} of {total}.</p>
  </div>
</header>

<nav class="subnav">
  <div class="wrap">
    <a href="../">Home</a>
    <a href="../manual/" class="current">Manual</a>
    <a href="../tutorials/">Tutorials</a>
    <a href="https://github.com/johnjoeallen/bascal">GitHub</a>
    <button id="theme-toggle" class="theme-toggle" aria-label="Toggle color theme"></button>
  </div>
</nav>

<main class="wrap" id="{cid}">
  <p class="crumbs"><a href="../">Home</a> / <a href="../manual/">Manual</a> / {title}</p>

  <div class="prose">
{body}  </div>

  <p class="crumbs">
{nav_links}  </p>
</main>

<footer>
  <div class="wrap">
    BASCAL is licensed under the
    <a href="https://github.com/johnjoeallen/bascal/blob/main/LICENSE">GNU GPLv3</a>.
    This page is generated from
    <a href="https://github.com/johnjoeallen/bascal/blob/main/MANUAL.md">MANUAL.md</a>
    in the repo — that file is the source of truth.
  </div>
</footer>

<script src="../bcl-highlight.js"></script>
<script src="../theme-toggle.js"></script>
</body>
</html>
"""

OUT_DIR.mkdir(exist_ok=True)
ordered_ids = [cid for _, cid, _ in chapters]
total = len(ordered_ids)

for idx, cid in enumerate(ordered_ids):
    sec = sections[cid]
    title = strip_tags(sec["title"])
    body_html = rewrite_hrefs(sec["html"], cid)
    # First <p> in the section, tags stripped, trimmed -- used as the meta description.
    p_match = re.search(r"<p>(.*?)</p>", sec["html"], re.S)
    description = strip_tags(p_match.group(1)).strip() if p_match else title
    description = re.sub(r"\s+", " ", description)
    if len(description) > 200:
        description = description[:197].rsplit(" ", 1)[0] + "..."
    description = description.replace('"', "&quot;")

    nav_bits = []
    if idx > 0:
        prev_id = ordered_ids[idx - 1]
        prev_title = strip_tags(sections[prev_id]["title"])
        nav_bits.append(f'    <a href="{prev_id}.html">&larr; {prev_title}</a>\n')
    if idx < total - 1:
        next_id = ordered_ids[idx + 1]
        next_title = strip_tags(sections[next_id]["title"])
        sep = "\n" if nav_bits else ""
        nav_bits.append(f'{sep}    <a href="{next_id}.html">{next_title} &rarr;</a>\n')
    nav_links = "".join(nav_bits) if nav_bits else "    <a href=\"./\">Back to Manual</a>\n"

    page = PAGE_TEMPLATE.format(
        title=title,
        description=description,
        num=idx + 1,
        total=total,
        body=body_html,
        nav_links=nav_links,
        cid=cid,
    )
    (OUT_DIR / f"{cid}.html").write_text(page)

print(f"wrote {total} chapter pages to {OUT_DIR}")
for cid in ordered_ids:
    print(" ", cid)
