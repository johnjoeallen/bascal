#!/usr/bin/env python3
"""Convert the old hand-written documentation HTML to MkDocs Markdown.

Run once while migrating a checkout.  The generated Markdown is the new
source of truth; MkDocs produces the deployable HTML in ``site/``.
"""
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DOCS = ROOT / "docs"


def markdown_from(html: Path, rel_path: Path) -> str:
    """Convert the old page's main content, leaving site chrome to MkDocs."""
    source = html.read_text() if html.exists() else subprocess.run(
        ["git", "show", f"HEAD:{rel_path.as_posix()}"],
        check=True, capture_output=True, text=True,
    ).stdout
    match = re.search(r"<main\b[^>]*>(.*?)</main>", source, re.DOTALL | re.IGNORECASE)
    if not match:
        raise ValueError(f"{rel_path}: no <main> element found")
    result = subprocess.run(
        ["pandoc", "--from=html", "--to=gfm", "--wrap=none"],
        check=True, input=match.group(1), capture_output=True, text=True,
    )
    text = result.stdout
    # MkDocs resolves links to source pages, then emits the existing directory
    # URLs (for example /manual/introduction/).
    text = re.sub(r"(?<=\])\(([^)#]+)\.html(#[^)]+)?\)", r"(\1.md\2)", text)
    text = re.sub(r"^```$", "```bascal", text, flags=re.MULTILINE)
    return text


def main() -> None:
    tracked = subprocess.run(
        ["git", "ls-tree", "-r", "--name-only", "HEAD", "docs"],
        check=True, capture_output=True, text=True,
    ).stdout.splitlines()
    pages = [ROOT / path for path in tracked if path.endswith(".html") and path != "docs/manual.html"]
    for page in pages:
        rel = page.relative_to(ROOT)
        target = page.with_suffix(".md")
        target.write_text(markdown_from(page, rel))
        if page.exists():
            page.unlink()
        print(target.relative_to(ROOT))


if __name__ == "__main__":
    main()
