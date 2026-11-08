#!/usr/bin/env python3
"""Check relative Markdown links in project documentation."""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DOCS = [ROOT / "README.md", ROOT / "AGENTS.md", *sorted((ROOT / "plan").glob("*.md"))]
LINK = re.compile(r"\[[^\]]*\]\(([^)]+)\)")


def strip_code(text: str) -> str:
    out: list[str] = []
    fenced = False
    for line in text.splitlines():
        if line.lstrip().startswith("```"):
            fenced = not fenced
            out.append("")
            continue
        out.append("" if fenced else line)
    return "\n".join(out)


def main() -> int:
    failures = 0
    for document in DOCS:
        if not document.exists():
            continue
        text = strip_code(document.read_text(encoding="utf-8"))
        for match in LINK.finditer(text):
            target = match.group(1).strip()
            if not target or target.startswith(("#", "http://", "https://", "mailto:")):
                continue
            target = target.split("#", 1)[0]
            if not target:
                continue
            resolved = (document.parent / target).resolve()
            if not resolved.exists():
                print(f"{document.relative_to(ROOT)}: broken link {target}", file=sys.stderr)
                failures += 1
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
