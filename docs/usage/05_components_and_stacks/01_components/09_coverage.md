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

## VS Code extension

On the **Components** tab:

1. **Coverage chip** — aggregate % after scan (or “Coverage off” when `codebase` is missing). Click opens the coverage subview when enabled.
2. **Coverage subview** — file tree for the `stack.codebase` universe only (`files[]` from the host). Colors:
   - File: 100% green, 0% red, otherwise yellow
   - Directory: all-green children → green; all-red → red; otherwise yellow
3. **Click a file** — opens it in the editor and highlights gap ranges. Back / leaving the subview clears decorations.

## Authoring

Attach spans from query hits on the pieces you model. Prefer leaf spans (fields, args, methods); a large parent span is only useful when there are no spanned children. See [Parse tree](03_parse_tree.md).

## Span coverage vs snapshot relation metrics

Two different scores live next to each other in `.cache/scan-snapshot.json`:

| Block | Measures |
|-------|----------|
| `coverage` | Leaf `source.span` vs `stack.codebase` (compact: no `gaps`) |
| `relations` | Instance connectivity (membership vs architecture); see [relations](10_relations.md) |

`stack.zoneRoles.ignoreCoverage` applies **only** to snapshot compact `coverage`
(members under those zones do not paint covered bytes). Live `component_coverage`
ignores zone flags in v1. `ignoreRelations` affects `relations` metrics only.
