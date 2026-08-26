#!/usr/bin/env python3
"""Generate every conformance page from the unified test index."""
from pathlib import Path
import tomllib

root = Path(__file__).resolve().parents[1]
entries = tomllib.loads((root / "conformance/test-index.toml").read_text())["test"]
overview = "# Conformance tests\n\nGenerated from the conformance test metadata and latest build run.\n\n" + "\n".join(
    f"- [{title}]({filename.removesuffix('.md')}/)" for filename, title in (value for value in {
        "core": ("core-language.md", "Core language"), "tutorials": ("tutorials.md", "Tutorials"),
        "basic": ("basic.md", "BASIC-specific"), "c": ("c.md", "C-specific"),
        "jvm": ("jvm.md", "JVM-specific"), "files": ("records.md", "Files and records")
    }.values())
) + "\n"
(root / "docs/conformance.md").write_text(overview)
pages = {
    "core": ("core-language.md", "Core language"),
    "tutorials": ("tutorials.md", "Tutorials"),
    "basic": ("basic.md", "BASIC-specific"),
    "c": ("c.md", "C-specific"),
    "jvm": ("jvm.md", "JVM-specific"),
    "files": ("records.md", "Files and records"),
}
for group, (filename, title) in pages.items():
    selected = [e for e in entries if group in e.get("groups", [])]
    rows = []
    for entry in selected:
        status = entry.get("expected", {}).get(group if group in ("basic", "c", "jvm") else "basic", "UNKNOWN")
        if group in ("core", "tutorials"):
            cells = [entry.get("expected", {}).get(b, "UNKNOWN") for b in ("basic", "c", "jvm")]
            rows.append(f"| {entry.get('description', entry['name'])} | {' | '.join(cells)} |")
        else:
            rows.append(f"| {entry.get('description', entry['name'])} | {status} |")
    if group in ("core", "tutorials"):
        table = "| Test description | BASIC | C | JVM |\n| --- | :---: | :---: | :---: |"
    else:
        table = "| Test description | Result |\n| --- | :---: |"
    body = "\n".join(rows) if rows else "| No indexed tests | UNKNOWN |"
    page = f"# [Conformance tests](../)\n\n## {title}\n\n{table}\n{body}\n\n<nav class=\"conformance-nav\" aria-label=\"Conformance results navigation\">\n  <a href=\"../\">← Back to conformance overview</a>\n</nav>\n"
    (root / "docs/conformance" / filename).write_text(page)
    print(f"generated {filename} from {len(selected)} indexed tests")
