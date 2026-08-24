#!/usr/bin/env python3
"""Normalise links emitted by the one-time HTML-to-Markdown conversion."""
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def fence_indented_code(text: str) -> str:
    """Turn Pandoc's indented preformatted blocks into typed BASCAL fences."""
    lines = text.splitlines(keepends=True)
    result: list[str] = []
    index = 0
    in_fence = False
    while index < len(lines):
        line = lines[index]
        if line.startswith("```"):
            in_fence = not in_fence
            result.append(line)
            index += 1
            continue
        if not in_fence and line.startswith("    "):
            block: list[str] = []
            while index < len(lines):
                candidate = lines[index]
                if candidate.startswith("    "):
                    block.append(candidate[4:])
                    index += 1
                elif candidate.strip() == "" and index + 1 < len(lines) and lines[index + 1].startswith("    "):
                    block.append(candidate)
                    index += 1
                else:
                    break
            result.extend(["```bascal\n", *block, "```\n"])
            continue
        result.append(line)
        index += 1
    return "".join(result)


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
        if page == ROOT / "docs" / "language" / "index.md":
            text = re.sub(
                r'(<span class="chapter-number">.*?</span>)(<span class="chapter-title">.*?</span><span class="chapter-summary">.*?</span>)',
                r'\1<span>\2</span>',
            text,
        )
        text = fence_indented_code(text)
        page.write_text(text)


if __name__ == "__main__":
    main()
