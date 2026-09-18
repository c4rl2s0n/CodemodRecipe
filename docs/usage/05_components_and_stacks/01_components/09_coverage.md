# Component span coverage

After a component scan, measure how much of the declared **codebase** is claimed by explicit **coverage-leaf** `source.span` ranges on parse-tree nodes.

## Semantics (leaf spans only)

- Only nodes with `source.path` **and** `source.span = { from, to }` that have **no descendant** with an explicit `source.span` paint coverage (via `children` / `fields`).
- A parent span that wraps spanned children is ignored — otherwise gaps between those children would look covered.
- Path-only `source` is for identity/linking — it does **not** cover the file and does not disqualify a parent span.
- Whitespace is excluded from the ratio denominator. Comments, imports, and any other non-blank text count; if components do not represent them, they appear as gaps.
- Ratio: `coveredNonblank / (coveredNonblank + gapNonblank)` (vacuous 1.0 when a file is all whitespace).

## Gaps

Each file returns the full `gaps` list (no truncation):

- One entry per contiguous **uncovered** run that contains any nonblank byte.
- `from` / `to` span from the first nonblank through past the last nonblank in that run (internal whitespace stays inside the range for editor highlighting).
- Whitespace-only uncovered stretches emit **no** gap entry.

## Prerequisites

1. `.codemod/stack.luau` with a non-empty **`codebase`** glob list (see [stacks](../02_stacks/01_required_manifest.md)).
2. A scan cache (`component_scan` / `component_coverage` runs scan first if needed).

If `codebase` is missing or empty, coverage is **skipped** (`skipped: true`) with a **warning** — the scan itself still succeeds.

## Host / MCP

| Surface | Name |
|---------|------|
| Host command | `componentCoverage` |
| MCP tool | `component_coverage` |

Response shape (extension-ready):

```json
{
  "ok": true,
  "aggregate": { "ratio": 0.5, "coveredNonblank": 10, "gapNonblank": 10 },
  "files": [
    {
      "path": "lib/a.dart",
      "ratio": 0.5,
      "coveredNonblank": 10,
      "gapNonblank": 10,
      "gaps": [{ "from": 20, "to": 30 }]
    }
  ]
}
```

Skipped:

```json
{
  "ok": true,
  "skipped": true,
  "warning": "stack.codebase is required to enable component coverage; …"
}
```

## Authoring

Attach spans from query hits on the pieces you model. Prefer leaf spans (fields, args, methods); a large parent span is only useful when there are no spanned children. See [Parse tree](03_parse_tree.md).

## VS Code UI

A Components-tab coverage badge and file-tree subview are a **follow-up** (not shipped in this host release). Agents and MCP clients already receive per-file data and full gap ranges.
