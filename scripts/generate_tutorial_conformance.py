#!/usr/bin/env python3
"""Generate the tutorial conformance table from tutorial/conformance.toml."""
from pathlib import Path
import tomllib
import re

root = Path(__file__).resolve().parents[1]
meta = tomllib.loads(root.joinpath("tutorial/conformance.toml").read_text())
rows = []
for item in meta["tutorial"]:
    missing = not root.joinpath("tutorial", item["source"]).exists()
    if missing:
        raise SystemExit(f"missing tutorial source: {item['source']}")
    not_applicable = set(item.get("na", []))
    cells = [
        "N/A" if backend in not_applicable else ("PASS" if backend in item["backends"] else "FAIL")
        for backend in ("basic", "c", "jvm")
    ]
    display_name = re.sub(r"^\d+\s+", "", item["name"])
    rows.append(f"| {display_name} | {' | '.join(cells)} |")
page = """# [Conformance tests](../)\n\n## Tutorials\n\n| Tutorial | BASIC | C | JVM |\n| --- | :---: | :---: | :---: |\n""" + "\n".join(rows) + """\n\n<nav class=\"conformance-nav\" aria-label=\"Conformance results navigation\">\n  <a href=\"../\">← Previous: Core language</a>\n  <a href=\"basic/\">Next: BASIC-specific →</a>\n</nav>\n"""
target = root / "docs/conformance/tutorials.md"
page += "\n`FAIL` means the backend is required to support the tutorial but does not yet pass it. `N/A` is reserved for tests explicitly not applicable (such as an optional runtime).\n"
target.write_text(page)
print(f"generated {target} from {len(rows)} tutorial annotations")
