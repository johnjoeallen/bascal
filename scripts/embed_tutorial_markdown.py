#!/usr/bin/env python3
"""Refresh generated tutorial source blocks in Markdown tutorial pages."""
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TUTORIALS = ROOT / "docs" / "tutorials"
GITHUB_BLOB = "https://github.com/johnjoeallen/bascal/blob/main/"
BEGIN = "<!-- BEGIN generated tutorial source -->"
END = "<!-- END generated tutorial source -->"
GENERATED = re.compile(re.escape(BEGIN) + r".*?" + re.escape(END), re.S)
SOURCE_EMBED = re.compile(r'<div class="source-embed">.*?</div>', re.S)
LINK = re.compile(re.escape(GITHUB_BLOB) + r"(tutorial/[^)\s]+)")
PATH_HEADER = re.compile(r"^### `?(tutorial/[^`\n]+)`?$", re.MULTILINE)
EMBED_PATH = re.compile(r"<summary><code>(tutorial/[^<\n]+)</code></summary>")


def regenerate(paths: list[str]) -> None:
    binary = ROOT / "target" / "debug" / "bcc"
    for path in paths:
        source = ROOT / path
        if source.suffix != ".bcl" or not re.search(r"^\s*program\b", source.read_text(), re.M):
            continue
        # Tutorial 12 deliberately keeps `require stats` short and documents
        # the library search path separately.  The docs build must provide the
        # same tutorial library directory as that documented command line.
        subprocess.run(
            [binary, "--target", "basic", "-L", ROOT / "tutorial" / "lib", source],
            check=True,
            cwd=ROOT,
        )
        if str(source.with_suffix(".c").relative_to(ROOT)) in paths:
            subprocess.run([binary, "--target", "c", source], check=True, cwd=ROOT)


def replacement(paths: list[str]) -> str:
    result = [BEGIN]
    for path in paths:
        source = ROOT / path
        language = "bascal" if source.suffix == ".bcl" else "c" if source.suffix == ".c" else "basic"
        # Trailing spaces in generated BASIC have no rendered meaning and make
        # the Markdown source noisy in diffs.
        content = "\n".join(line.rstrip() for line in source.read_text().splitlines())
        result.extend([
            '<details class="source-embed" markdown="1">',
            f"<summary><code>{path}</code></summary>",
            "",
            f"```{language}",
            content,
            "```",
            "",
            "</details>",
        ])
    result.append(END)
    return "\n\n".join(result)


def main() -> None:
    for page in sorted(TUTORIALS.glob("[0-9][0-9]_*.md")):
        text = page.read_text()
        existing = GENERATED.search(text)
        paths = list(
            dict.fromkeys(
                LINK.findall(text)
                or PATH_HEADER.findall(text)
                or EMBED_PATH.findall(text)
            )
        )
        if not paths:
            continue
        regenerate(paths)
        new = replacement(paths)
        # Pandoc preserved the old HTML source-embed wrapper.  Replace it
        # rather than leaving a stale copy beside the refreshed Markdown one.
        text = SOURCE_EMBED.sub("", text)
        existing = GENERATED.search(text)
        text = text[:existing.start()] + new + text[existing.end():] if existing else text + "\n\n" + new + "\n"
        page.write_text(text)


if __name__ == "__main__":
    main()
