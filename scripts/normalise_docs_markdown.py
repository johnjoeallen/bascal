#!/usr/bin/env python3
"""Normalise links emitted by the one-time HTML-to-Markdown conversion."""
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def main() -> None:
    for page in (ROOT / "docs").rglob("*.md"):
        if "theme" in page.parts:
            continue
        text = page.read_text()
        text = text.replace("]](", "](")
        text = re.sub(
            r'<div(\s[^>]*?)?>',
            lambda m: m.group(0) if "markdown=" in m.group(0) else m.group(0)[:-1] + ' markdown="1">',
            text,
        )
        text = re.sub(r"\]\(([^)\s]+)\.html(?=#[^)]+\)|\))", r"](\1.md", text)
        text = text.replace(
            "../../LICENSE-OUTPUT-EXCEPTION.md",
            "https://github.com/johnjoeallen/bascal/blob/main/LICENSE-OUTPUT-EXCEPTION.md",
        )
        page.write_text(text)


if __name__ == "__main__":
    main()
