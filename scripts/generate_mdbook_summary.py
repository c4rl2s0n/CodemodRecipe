#!/usr/bin/env python3
"""Generate mdBook SUMMARY.md files for usage (consumer) and docs (repo).

Walks nested numbered directories recursively so bite-sized topic folders appear
in the sidebar.
"""

from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOCS = ROOT / "docs"
USAGE = DOCS / "usage"
DEVELOP = DOCS / "develop"
USAGE_SUMMARY = USAGE / "SUMMARY.md"
DOCS_SUMMARY = DOCS / "SUMMARY.md"

TITLE_RE = re.compile(r"^#\s+(.+)$", re.M)
NUMBERED = re.compile(r"^\d+_")


def title_of(path: Path) -> str:
    text = path.read_text(encoding="utf-8")
    m = TITLE_RE.search(text)
    if m:
        return m.group(1).strip()
    stem = path.stem
    stem = re.sub(r"^\d+_", "", stem)
    return stem.replace("_", " ").strip() or path.name


def child_entries(folder: Path) -> list[Path]:
    """Numbered subdirs and .md pages (excluding 00_index / SUMMARY / book artifacts).

    Prefer a topic directory over a same-stem `.md` stub if both exist.
    """
    entries: list[Path] = []
    for p in folder.iterdir():
        if p.name in ("book", "SUMMARY.md") or p.name.startswith("."):
            continue
        if p.name.startswith("mermaid"):
            continue
        if p.suffix == ".js" or p.name == "book.toml":
            continue
        if p.is_dir() and NUMBERED.match(p.name):
            entries.append(p)
        elif p.is_file() and p.suffix == ".md" and p.name != "00_index.md":
            # skip ghost stubs left beside a topic folder of the same stem
            if (folder / p.stem).is_dir():
                continue
            entries.append(p)
    return sorted(entries, key=lambda p: p.name)


def emit_tree(folder: Path, *, rel: str, depth: int) -> list[str]:
    """Emit SUMMARY bullets for folder children (not the folder's own index)."""
    lines: list[str] = []
    indent = "  " * depth
    for entry in child_entries(folder):
        if entry.is_dir():
            index = entry / "00_index.md"
            child_rel = f"{rel}/{entry.name}" if rel else entry.name
            if index.is_file():
                lines.append(
                    f"{indent}- [{title_of(index)}]({child_rel}/00_index.md)"
                )
            else:
                lines.append(f"{indent}- [{entry.name}]()")
            lines.extend(emit_tree(entry, rel=child_rel, depth=depth + 1))
        else:
            page_rel = f"{rel}/{entry.name}" if rel else entry.name
            lines.append(f"{indent}- [{title_of(entry)}]({page_rel})")
    return lines


def usage_chapter_lines(*, path_prefix: str = "") -> list[str]:
    lines: list[str] = []
    chapters = sorted(
        p for p in USAGE.iterdir() if p.is_dir() and NUMBERED.match(p.name)
    )
    for chapter in chapters:
        index = chapter / "00_index.md"
        rel = f"{path_prefix}{chapter.name}"
        if index.is_file():
            lines.append(f"- [{title_of(index)}]({rel}/00_index.md)")
        else:
            lines.append(f"- [{chapter.name}]()")
        lines.extend(emit_tree(chapter, rel=rel, depth=1))
        lines.append("")
    return lines


def develop_lines(*, path_prefix: str = "develop") -> list[str]:
    lines: list[str] = []
    index = DEVELOP / "00_index.md"
    if index.is_file():
        lines.append(f"- [{title_of(index)}]({path_prefix}/00_index.md)")
        lines.extend(emit_tree(DEVELOP, rel=path_prefix, depth=1))
    else:
        for entry in child_entries(DEVELOP):
            if entry.is_dir():
                idx = entry / "00_index.md"
                rel = f"{path_prefix}/{entry.name}"
                if idx.is_file():
                    lines.append(f"- [{title_of(idx)}]({rel}/00_index.md)")
                lines.extend(emit_tree(entry, rel=rel, depth=1))
            else:
                lines.append(
                    f"- [{title_of(entry)}]({path_prefix}/{entry.name})"
                )
    lines.append("")
    return lines


def write_usage_summary() -> None:
    lines = ["# Summary", "", "[Usage docs](00_index.md)", ""]
    lines.extend(usage_chapter_lines())
    USAGE_SUMMARY.write_text("\n".join(lines).rstrip() + "\n", encoding="utf-8")
    print(f"wrote {USAGE_SUMMARY.relative_to(ROOT)}")


def write_docs_summary() -> None:
    lines = [
        "# Summary",
        "",
        "[Documentation](README.md)",
        "",
        "# Usage",
        "",
        "[Usage docs](usage/00_index.md)",
        "",
    ]
    lines.extend(usage_chapter_lines(path_prefix="usage/"))
    lines.extend(["# Develop", ""])
    lines.extend(develop_lines(path_prefix="develop"))
    DOCS_SUMMARY.write_text("\n".join(lines).rstrip() + "\n", encoding="utf-8")
    print(f"wrote {DOCS_SUMMARY.relative_to(ROOT)}")


def main() -> None:
    write_usage_summary()
    write_docs_summary()


if __name__ == "__main__":
    main()
