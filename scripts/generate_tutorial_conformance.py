#!/usr/bin/env python3
"""Generate the tutorial conformance table from tutorial/conformance.toml."""
from pathlib import Path
import tomllib
import re

root = Path(__file__).resolve().parents[1]
meta = tomllib.loads(root.joinpath("tutorial/conformance.toml").read_text())
valid_statuses = {"PASS", "FAIL", "UNSUPPORTED", "DEFERRED"}
rows = []
for item in meta["tutorial"]:
    missing = not root.joinpath("tutorial", item["source"]).exists()
    if missing:
        raise SystemExit(f"missing tutorial source: {item['source']}")
    statuses = item.get("status", {})
    not_applicable = set(item.get("na", []))
    will_not_implement = set(item.get("wont", []))
    cells = []
    for backend in ("basic", "c", "jvm"):
        state = statuses.get(backend)
        if state is None:
            state = "UNSUPPORTED" if backend in not_applicable or backend in will_not_implement else (
                "PASS" if backend in item["backends"] else "DEFERRED"
            )
        state = state.upper()
        if state not in valid_statuses:
            raise SystemExit(f"{item['name']}: invalid {backend} status {state!r}")
        cells.append(state)
    display_name = re.sub(r"^\d+\s+", "", item["name"])
    rows.append(f"| {display_name} | {' | '.join(cells)} |")
page = """# [Conformance tests](../)\n\n## Tutorials\n\n| Tutorial | BASIC | C | JVM |\n| --- | :---: | :---: | :---: |\n""" + "\n".join(rows) + """\n\n<nav class=\"conformance-nav\" aria-label=\"Conformance results navigation\">\n  <a href=\"../\">← Previous: Core language</a>\n  <a href=\"basic/\">Next: BASIC-specific →</a>\n</nav>\n"""
target = root / "docs/conformance/tutorials.md"
page += "\n`FAIL` means a required check ran and failed. `UNSUPPORTED` means the backend will not implement the feature. `DEFERRED` means support is expected but validation is not yet in the suite.\n"
target.write_text(page)
print(f"generated {target} from {len(rows)} tutorial annotations")
