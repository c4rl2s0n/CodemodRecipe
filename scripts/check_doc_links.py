#!/usr/bin/env python3
"""Check relative markdown links under docs/usage and usage skills (SKILL.md)."""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LINK_RE = re.compile(r"\[([^\]]*)\]\(([^)]+)\)")

SCAN_ROOTS = [
    ROOT / "docs" / "usage",
    ROOT / "docs" / "develop",
    ROOT / ".agents" / "skills" / "codemod-recipe",
]


def should_skip(path: Path) -> bool:
    parts = path.parts
    # Do not double-scan usage via the docs symlink under skills
    if "skills" in parts and "codemod-recipe" in parts and "docs" in parts:
        # path is under .agents/.../docs — skip (same files as docs/usage)
        try:
            path.relative_to(ROOT / ".agents" / "skills" / "codemod-recipe" / "docs")
            return True
        except ValueError:
            pass
    return path.name == "SUMMARY.md"


def check_file(path: Path) -> list[str]:
    errors: list[str] = []
    text = path.read_text(encoding="utf-8")
    for m in LINK_RE.finditer(text):
        url = m.group(2).strip()
        if url.startswith(("http://", "https://", "mailto:", "#")):
            continue
        path_part = url.split("#", 1)[0].split("?", 1)[0]
        if not path_part:
            continue
        target = (path.parent / path_part).resolve()
        if not target.exists():
            errors.append(f"{path.relative_to(ROOT)}: broken link ({url})")
    return errors


def main() -> int:
    errors: list[str] = []
    for root in SCAN_ROOTS:
        if not root.exists():
            errors.append(f"missing scan root: {root}")
            continue
        for md in root.rglob("*.md"):
            if should_skip(md):
                continue
            errors.extend(check_file(md))

    if errors:
        print("\n".join(errors))
        print(f"\n{len(errors)} broken link(s)", file=sys.stderr)
        return 1
    print("ok: no broken relative links")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
