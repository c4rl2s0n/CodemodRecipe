#!/usr/bin/env python3
"""Split a markdown file: each ## becomes a sibling .md under a topic folder.

Usage:
  python3 scripts/split_md_by_headings.py path/to/file.md [--keep-stem]
  python3 scripts/split_md_by_headings.py path/to/00_index.md --inplace-chapter

--keep-stem: folder name = file stem (01_workflow.md → 01_workflow/)
--inplace-chapter: split ## out of 00_index.md in place (keep short landing)
"""

from __future__ import annotations

import argparse
import re
import sys
import unicodedata
from pathlib import Path

HEADING_RE = re.compile(r"^(#{1,6})\s+(.+?)\s*$")


def slugify(title: str) -> str:
    text = unicodedata.normalize("NFKD", title)
    text = "".join(c for c in text if not unicodedata.combining(c))
    text = text.lower()
    # curly apostrophes → strip
    text = text.replace("'", "").replace("'", "").replace("`", "")
    text = re.sub(r"[^a-z0-9]+", "_", text)
    text = re.sub(r"_+", "_", text).strip("_")
    return text or "section"


def demote_headings(body: str) -> str:
    """Move headings up one level (###→## etc). Leave # alone if any."""
    out: list[str] = []
    in_fence = False
    fence_marker = ""
    for line in body.splitlines(keepends=True):
        stripped = line.rstrip("\n")
        if not in_fence:
            m = re.match(r"^(```+|~~~+)(.*)$", stripped)
            if m:
                in_fence = True
                fence_marker = m.group(1)[0]
                out.append(line)
                continue
            hm = HEADING_RE.match(stripped)
            if hm and len(hm.group(1)) >= 2:
                level = len(hm.group(1)) - 1
                out.append("#" * level + " " + hm.group(2) + "\n")
                if line.endswith("\n") and not out[-1].endswith("\n"):
                    out[-1] += "\n"
                # preserve exact newline from original via rebuild
                continue
            out.append(line)
        else:
            if stripped.startswith(fence_marker * 3) or (
                fence_marker and stripped == fence_marker * len(
                    re.match(r"^(```+|~~~+)", stripped).group(1)
                    if re.match(r"^(```+|~~~+)", stripped)
                    else ""
                )
            ):
                # close fence
                close = re.match(r"^(```+|~~~+)\s*$", stripped)
                if close and close.group(1)[0] == fence_marker:
                    in_fence = False
                    fence_marker = ""
            out.append(line)
    return "".join(out)


def demote_headings_simple(body: str) -> str:
    """Demote ##+ headings by one; skip fenced code."""
    lines = body.splitlines(keepends=True)
    out: list[str] = []
    in_fence = False
    fence_char = ""
    for line in lines:
        stripped = line.rstrip("\r\n")
        fence = re.match(r"^(```+|~~~+)", stripped)
        if fence:
            marker = fence.group(1)
            if not in_fence:
                in_fence = True
                fence_char = marker[0]
            elif marker[0] == fence_char and re.match(r"^(```+|~~~+)\s*$", stripped):
                in_fence = False
                fence_char = ""
            out.append(line)
            continue
        if not in_fence:
            hm = HEADING_RE.match(stripped)
            if hm and len(hm.group(1)) >= 2:
                new_level = len(hm.group(1)) - 1
                nl = "\n" if line.endswith("\n") else ""
                if line.endswith("\r\n"):
                    nl = "\r\n"
                out.append("#" * new_level + " " + hm.group(2) + nl)
                continue
        out.append(line)
    return "".join(out)


def parse_sections(text: str) -> tuple[str, str, list[tuple[str, str]]]:
    """Return (h1_title, preamble_before_first_h2, [(h2_title, body), ...])."""
    lines = text.splitlines(keepends=True)
    in_fence = False
    fence_char = ""
    h1 = ""
    preamble_parts: list[str] = []
    sections: list[tuple[str, list[str]]] = []
    current_h2: str | None = None
    current_body: list[str] = []
    seen_h1 = False

    def flush() -> None:
        nonlocal current_h2, current_body
        if current_h2 is not None:
            sections.append((current_h2, current_body))
            current_body = []

    for line in lines:
        stripped = line.rstrip("\r\n")
        fence = re.match(r"^(```+|~~~+)", stripped)
        if fence:
            marker = fence.group(1)
            if not in_fence:
                in_fence = True
                fence_char = marker[0]
            elif marker[0] == fence_char and re.match(r"^(```+|~~~+)\s*$", stripped):
                in_fence = False
                fence_char = ""
            if current_h2 is None:
                if seen_h1:
                    preamble_parts.append(line)
            else:
                current_body.append(line)
            continue

        if not in_fence:
            hm = HEADING_RE.match(stripped)
            if hm:
                level = len(hm.group(1))
                title = hm.group(2).strip()
                if level == 1 and not seen_h1:
                    h1 = title
                    seen_h1 = True
                    continue
                if level == 2:
                    flush()
                    current_h2 = title
                    current_body = []
                    continue

        if current_h2 is None:
            if seen_h1:
                preamble_parts.append(line)
        else:
            current_body.append(line)

    flush()
    preamble = "".join(preamble_parts).strip()
    return h1, preamble, [(t, "".join(b).strip()) for t, b in sections]


def write_split(
    src: Path,
    *,
    keep_stem: bool,
    inplace_chapter: bool,
) -> Path:
    text = src.read_text(encoding="utf-8")
    h1, preamble, sections = parse_sections(text)
    if not sections:
        print(f"skip (no ##): {src}", file=sys.stderr)
        return src

    if inplace_chapter:
        folder = src.parent
        folder_title = h1 or title_from_dir(folder)
    elif keep_stem:
        folder = src.parent / src.stem
        folder_title = h1 or src.stem
    else:
        # numbered from sibling order — caller should use keep_stem for siblings
        slug = slugify(h1 or src.stem)
        folder = src.parent / slug
        folder_title = h1 or slug

    folder.mkdir(parents=True, exist_ok=True)

    child_rows: list[tuple[str, str]] = []
    for i, (title, body) in enumerate(sections, start=1):
        slug = slugify(title)
        fname = f"{i:02d}_{slug}.md"
        demoted = demote_headings_simple(body)
        content = f"# {title}\n"
        if demoted:
            content += "\n" + demoted
            if not content.endswith("\n"):
                content += "\n"
        else:
            content += "\n"
        (folder / fname).write_text(content, encoding="utf-8")
        child_rows.append((title, fname))
        print(f"  wrote {folder / fname}")

    # 00_index
    intro = preamble.strip()
    if not intro:
        intro = f"{folder_title}."
    index_lines = [f"# {folder_title}", "", intro, "", "| Page | |", "|------|---|"]
    for title, fname in child_rows:
        index_lines.append(f"| [{title}]({fname}) | |")
    index_lines.append("")
    (folder / "00_index.md").write_text("\n".join(index_lines), encoding="utf-8")
    print(f"  wrote {folder / '00_index.md'}")

    if inplace_chapter:
        # replace fat index with short landing pointing at children
        landing = [
            f"# {folder_title}",
            "",
            intro if preamble.strip() else f"Overview of {folder_title}.",
            "",
            "| Page | Topic |",
            "|------|-------|",
        ]
        for title, fname in child_rows:
            landing.append(f"| [{title}]({fname}) | |")
        landing.append("")
        src.write_text("\n".join(landing), encoding="utf-8")
        print(f"  rewrote landing {src}")
    else:
        src.unlink()
        print(f"  removed {src}")

    return folder


def title_from_dir(folder: Path) -> str:
    idx = folder / "00_index.md"
    if idx.is_file():
        m = re.search(r"^#\s+(.+)$", idx.read_text(encoding="utf-8"), re.M)
        if m:
            return m.group(1).strip()
    return folder.name


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("paths", nargs="+", type=Path)
    ap.add_argument("--keep-stem", action="store_true")
    ap.add_argument("--inplace-chapter", action="store_true")
    args = ap.parse_args()
    for path in args.paths:
        if not path.is_file():
            print(f"missing: {path}", file=sys.stderr)
            return 1
        print(f"splitting {path}")
        write_split(
            path,
            keep_stem=args.keep_stem,
            inplace_chapter=args.inplace_chapter,
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
