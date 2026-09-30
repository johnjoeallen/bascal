#!/usr/bin/env python3
"""Keep the docs' "generated BASIC" blocks honest.

A block claims to be compiler output when it is tagged
`<span class="tag">Generated BASIC</span>` or introduced by a line reading
`Becomes:` or by an `<!-- generated-basic -->` marker line. Each such block is paired with its BASCAL source (the nearest
`<span class="tag">BASCAL</span>` block, or for `Becomes:` the fenced block just
before it). The source is compiled with `bcc` for the BASIC target, the lines
it contributes are extracted, and the displayed block is replaced by them.

    scripts/refresh_doc_generated.py           rewrite stale blocks in place
    scripts/refresh_doc_generated.py --check   list stale blocks, exit 1 if any
    scripts/refresh_doc_generated.py --check --programs
                                               also require every complete
                                               `program` example to compile
                                               (mark deliberate exceptions with
                                               `<!-- no-compile -->` above the fence)

`BCC` in the environment overrides the compiler path.
"""
import functools
import os
import re
import subprocess
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BCC = os.environ.get("BCC") or os.path.join(ROOT, "target", "debug", "bcc")
FENCE = re.compile(r"^```(\w*)\s*$")
MARKER = "<!-- generated-basic -->"
TAG = re.compile(r'<span class="tag">(.*?)</span>')

# Pages whose "Generated BASIC" tags are checked, and pages whose `Becomes:`
# blocks are. Other pages use the tag for hand-written comparisons.
TAGGED_PAGES = ["docs/home/record-files.md"]
BECOMES_PAGES = ["docs/manual/generated-basic-shape.md", "docs/home/features.md",
                 "docs/language/the-basic-beneath.md", "docs/manual/control-flow.md"]


def pages():
    for rel in TAGGED_PAGES:
        yield rel, "tagged"
    for rel in BECOMES_PAGES:
        yield rel, "becomes"


def blocks(lines):
    """Yield (fence_line_index, end_index, lang, text, tag, before) for each fence."""
    out = []
    tag = None
    i = 0
    prev_nonblank = ""
    while i < len(lines):
        line = lines[i]
        m = TAG.search(line)
        if m:
            tag = m.group(1)
        if line.startswith("#"):
            tag = None
        m = FENCE.match(line)
        if m:
            j = i + 1
            while j < len(lines) and not lines[j].startswith("```"):
                j += 1
            out.append({
                "open": i, "close": j, "lang": m.group(1),
                "text": "\n".join(lines[i + 1:j]), "tag": tag,
                "before": prev_nonblank,
            })
            tag = None
            i = j + 1
            prev_nonblank = ""
            continue
        if line.strip():
            prev_nonblank = line.strip()
        i += 1
    return out


def has_program_line(text):
    return any(re.match(r"\s*program\s+\w+", line) for line in text.split("\n"))


def wrap(text):
    if has_program_line(text):
        return text if text.endswith("\n") else text + "\n"
    body = text if text.endswith("\n") else text + "\n"
    if not re.search(r"^\s*end\s*$", body, re.M):
        body += "end\n"
    return "program docs\n" + body


@functools.lru_cache(maxsize=None)
def compile_basic_cached(source):
    return tuple(_compile_basic(source))


def compile_basic(source):
    return list(compile_basic_cached(source))


def _compile_basic(source):
    with tempfile.TemporaryDirectory() as directory:
        path = os.path.join(directory, "docs.bcl")
        with open(path, "w") as f:
            f.write(source)
        p = subprocess.run(
            [BCC, path, "-t", "basic", "--sparse-line-numbers", "-o", directory + "/", "--clean",
             "-L", os.path.join(ROOT, "tutorial", "lib"),
             "-L", os.path.join(ROOT, "examples", "sort_driver")],
            capture_output=True, text=True, timeout=120,
        )
        if p.returncode != 0:
            errors = [l for l in p.stderr.splitlines() if l.startswith("error")]
            raise RuntimeError(errors[0] if errors else p.stderr.strip()[:200])
        with open(os.path.join(directory, "docs.bas")) as f:
            return f.read().splitlines()


def body_lines(output):
    """Drop the two header comments and the final END."""
    lines = [l for l in output if l.strip()]
    lines = [l for l in lines if not re.match(r"^\d*\s*' (BASCAL generated BASIC|Functions are transpiled)", l)]
    while lines and re.match(r"^\d*\s*END\s*$", lines[-1]):
        lines.pop()
    return lines


def contribution(setup, snippet):
    """The lines `snippet` adds on top of `setup` (both BASCAL source)."""
    base = body_lines(compile_basic(wrap(setup or "program docs\nend\n")))
    full = body_lines(compile_basic(wrap((setup + "\n" if setup else "") + snippet)))
    # strip the common prefix and suffix
    p = 0
    while p < len(base) and p < len(full) and strip_no(base[p]) == strip_no(full[p]):
        p += 1
    s = 0
    while s < len(base) - p and s < len(full) - p and strip_no(base[-1 - s]) == strip_no(full[-1 - s]):
        s += 1
    return full[p:len(full) - s] if s else full[p:]


def strip_no(line):
    return re.sub(r"^\d+\s+", "", line.strip())


def hidden_setup(lines, fence):
    """Declarations from a `<!-- setup ... -->` comment just above a source fence."""
    i = fence - 1
    while i >= 0 and not lines[i].strip():
        i -= 1
    if i < 0 or not lines[i].rstrip().endswith("-->"):
        return None
    j = i
    while j >= 0 and not lines[j].lstrip().startswith("<!-- setup"):
        if lines[j].startswith("```") or lines[j].startswith("#"):
            return None
        j -= 1
    if j < 0:
        return None
    body = "\n".join(lines[j:i + 1])
    return re.sub(r"^<!-- setup\s*|\s*-->$", "", body).strip("\n") + "\n"


def refresh(rel, kind, check):
    path = os.path.join(ROOT, rel)
    lines = open(path).read().split("\n")
    found = blocks(lines)
    stale = []
    replacements = []
    setup = None
    for idx, b in enumerate(found):
        if b["lang"] == "bascal" and b["tag"] == "BASCAL" and setup is None \
                and re.search(r"^\s*record\b", b["text"], re.M) and re.search(r"^\s*file\b", b["text"], re.M):
            setup_block = b["text"]
            setup = setup_block
        generated = (kind == "tagged" and b["tag"] == "Generated BASIC") or \
                    (kind == "becomes" and (b["before"].rstrip(":").lower() == "becomes"
                                            or b["before"] == MARKER))
        if not generated:
            continue
        if kind == "tagged":
            src = next((found[k] for k in (idx + 1, idx - 1, idx + 2, idx - 2)
                        if 0 <= k < len(found) and found[k]["tag"] == "BASCAL"), None)
        else:
            src = found[idx - 1] if idx > 0 else None
        if src is None:
            raise RuntimeError("%s:%d: no BASCAL source block for this generated block" % (rel, b["open"] + 1))
        use_setup = setup if (kind == "tagged" and src["text"] != setup
                              and not re.search(r"^\s*record\b", src["text"], re.M)) else None
        hidden = hidden_setup(lines, src["open"])
        if hidden:
            use_setup = hidden
        try:
            new = contribution(use_setup, src["text"])
        except RuntimeError as error:
            raise RuntimeError("%s:%d: cannot compile the BASCAL source: %s" % (rel, b["open"] + 1, error))
        new_text = "\n".join(new)
        if new_text != b["text"]:
            stale.append("%s:%d" % (rel, b["open"] + 1))
            replacements.append((b["open"], b["close"], new))
    if not check and replacements:
        for open_i, close_i, new in reversed(replacements):
            lines[open_i:close_i + 1] = ["```basic"] + new + ["```"]
        with open(path, "w") as f:
            f.write("\n".join(lines))
    return stale


NO_COMPILE = "<!-- no-compile -->"


def doc_pages():
    for base, _, files in os.walk(os.path.join(ROOT, "docs")):
        for name in sorted(files):
            if name.endswith(".md"):
                yield os.path.join(base, name)
    for name in ("README.md", "CONTRIBUTING.md"):
        yield os.path.join(ROOT, name)


def check_programs():
    """Every complete `program` example in the docs must compile."""
    failures = []
    for path in doc_pages():
        lines = open(path, errors="replace").read().split("\n")
        for b in blocks(lines):
            if b["lang"] != "bascal" or b["before"] == NO_COMPILE:
                continue
            if not has_program_line(b["text"]):
                continue
            try:
                compile_basic(b["text"] + "\n")
            except RuntimeError as error:
                failures.append("%s:%d: %s" % (os.path.relpath(path, ROOT), b["open"] + 1, error))
    return failures


def main():
    check = "--check" in sys.argv
    if not os.path.exists(BCC):
        print("compiler not found at %s (build it, or set BCC)" % BCC, file=sys.stderr)
        return 2
    stale = []
    for rel, kind in pages():
        stale += refresh(rel, kind, check)
    if "--programs" in sys.argv:
        failures = check_programs()
        if failures:
            print("doc programs that do not compile:")
            for f in failures:
                print("  " + f)
        if failures:
            return 1
    if stale:
        print(("stale" if check else "refreshed") + " generated blocks:")
        for s in stale:
            print("  " + s)
    return 1 if (check and stale) else 0


if __name__ == "__main__":
    sys.exit(main())
