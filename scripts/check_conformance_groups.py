#!/usr/bin/env python3
"""Validate conformance-group annotations on integration test modules."""
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
KNOWN = {"core", "tutorials", "basic", "c", "jvm", "files", "records"}
pattern = re.compile(r"^// Conformance groups:\s*(.+)$", re.MULTILINE)
files = sorted(ROOT.joinpath("tests").glob("*.rs"))
missing = []
for path in files:
    if "conformance" in path.name or path.name in {"examples.rs", "record_general_purpose.rs"}:
        match = pattern.search(path.read_text())
        if not match:
            missing.append(path.name)
            continue
        groups = {item.strip() for item in match.group(1).split(",")}
        unknown = groups - KNOWN
        if unknown:
            raise SystemExit(f"{path}: unknown conformance groups: {', '.join(sorted(unknown))}")
if missing:
    raise SystemExit("missing conformance groups: " + ", ".join(missing))
print(f"validated conformance groups for {len(files)} test modules")
