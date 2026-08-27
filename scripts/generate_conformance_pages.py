#!/usr/bin/env python3
"""Generate every conformance page from the unified test index."""
from pathlib import Path
import tomllib
import re

root = Path(__file__).resolve().parents[1]
entries = tomllib.loads((root / "conformance/test-index.toml").read_text())["test"]
results_path = root / "conformance/test-results.toml"
observed = {}
if results_path.exists():
    observed = {r["id"]: r["observed"] for r in tomllib.loads(results_path.read_text()).get("result", [])}
def resolved(entry, backend):
    expected = entry.get("expected", {}).get(backend, "UNKNOWN")
    # A front-end-only test intentionally exercises parsing/loading without
    # selecting a backend. Its passing harness result must not imply that any
    # code generator implements the planned syntax.
    if entry.get("validation") == "FRONTEND_ONLY":
        return expected
    actual = observed.get(entry["id"])
    # A metadata FAIL deliberately describes an unsupported/invalidating
    # backend result.  Keep it visible as FAIL even when the harness test
    # itself passes by asserting the expected diagnostic; the test runner
    # remains green because that assertion is non-blocking.
    if expected == "FAIL":
        return "FAIL"
    if expected == "NOT APPLICABLE":
        return expected
    if actual is None:
        return expected
    if actual == "PASS":
        return "PASS"
    if expected in ("DEFERRED", "UNIMPLEMENTED", "UNSUPPORTED", "WILL NOT IMPLEMENT"):
        return expected
    return "FAIL"
pages = {
    "core": ("core-language.md", "Core language"),
    "tutorials": ("tutorials.md", "Tutorials"),
    "basic": ("basic.md", "BASIC-specific"),
    "c": ("c.md", "C-specific"),
    "jvm": ("jvm.md", "JVM-specific"),
    "records": ("records.md", "Files and records"),
}
order = ["core", "tutorials", "basic", "c", "jvm", "records"]
for group, (filename, title) in pages.items():
    selected = [e for e in entries if group in e.get("groups", [])]
    if group == "tutorials":
        selected = [e for e in selected if e.get("kind") == "tutorial"]
    rows = []
    for entry in selected:
        status = resolved(entry, group if group in ("basic", "c", "jvm") else "basic")
        description = re.sub(r"^\d+\s+", "", entry.get("description", entry["name"]))
        if group in ("core", "tutorials", "records"):
            cells = [resolved(entry, b) for b in ("basic", "c", "jvm")]
            rows.append(f"| {description} | {' | '.join(cells)} |")
        else:
            rows.append(f"| {description} | {status} |")
    if group in ("core", "tutorials", "records"):
        table = "| Test description | BASIC | C | JVM |\n| --- | :---: | :---: | :---: |"
    else:
        table = "| Test description | Result |\n| --- | :---: |"
    body = "\n".join(rows) if rows else "| No indexed tests | UNKNOWN |"
    index = order.index(group)
    links = ['  <a href="../">← Overview</a>']
    if index:
        links.insert(0, f'  <a href="../{order[index - 1]}/">← Previous: {pages[order[index - 1]][1]}</a>')
    if index + 1 < len(order):
        links.append(f'  <a href="../{order[index + 1]}/">Next: {pages[order[index + 1]][1]} →</a>')
    nav = "\n".join(links)
    page = f"# [Conformance tests](../)\n\n## {title}\n\n{table}\n{body}\n\n<nav class=\"conformance-nav\" aria-label=\"Conformance results navigation\">\n{nav}\n</nav>\n"
    (root / "docs/conformance" / filename).write_text(page)
    print(f"generated {filename} from {len(selected)} indexed tests")

# The conformance home is the core-language page, not a second link index.
core_page = (root / "docs/conformance/core-language.md").read_text()
core_page = core_page.replace("# [Conformance tests](../)", "# Conformance tests", 1)
core_page = core_page.replace('href="../tutorials/"', 'href="tutorials/"')
(root / "docs/conformance.md").write_text(core_page)
