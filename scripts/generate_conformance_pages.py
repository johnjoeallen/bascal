#!/usr/bin/env python3
"""Generate every conformance page from the unified test index."""
from pathlib import Path
import tomllib

root = Path(__file__).resolve().parents[1]
entries = tomllib.loads((root / "conformance/test-index.toml").read_text())["test"]
results_path = root / "conformance/test-results.toml"
observed = {}
if results_path.exists():
    observed = {r["id"]: r["observed"] for r in tomllib.loads(results_path.read_text()).get("result", [])}
def resolved(entry, backend):
    expected = entry.get("expected", {}).get(backend, "UNKNOWN")
    actual = observed.get(entry["id"])
    if actual is None:
        return expected
    if actual == "PASS":
        return "PASS"
    if expected in ("DEFERRED", "UNSUPPORTED", "WILL NOT IMPLEMENT"):
        return expected
    return "FAIL"
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
        status = resolved(entry, group if group in ("basic", "c", "jvm") else "basic")
        if group in ("core", "tutorials"):
            cells = [resolved(entry, b) for b in ("basic", "c", "jvm")]
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
