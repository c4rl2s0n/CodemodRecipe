---
name: codemod-recipe-builder-apply
description: >-
  Preview and apply codemod-recipe recipes or component hooks. Always dry-run
  first; pass previewToken; verify idempotency.
---

# Builder — Apply

Mutate the workspace via MCP. Full tool table: [codemod-mcp.md](../../docs/06_mcp_and_clients/01_codemod_mcp/00_index.md).

## Recipe checklist

1. `describe_recipe` — required args
2. `preview_recipe` (`recipe` id or `inlineRecipe`) → inspect `files[]`, `transcript[]`, save `previewToken`
3. `apply_recipe` with **same** args + `previewToken` (optional `selection`)
4. Re-`preview_recipe` — expect `files: []` when idempotent

## Component checklist

1. `describe_component` — scaffold args / operations
2. Ensure scan cache is warm (`component_scan` or prior intel)
3. `component_dry_run_hook` with `hook: "scaffold"` or `"operation"` (+ `operation` name, `target` / args)
4. Inspect `files` + `transcript` (`c:log` appears as `kind: "log"`)
5. `component_apply_hook` with the same arguments
6. `component_incremental_scan` with `dirtyPaths` when known

## Authoring hooks (dry-run only)

When debugging component Luau: `identify` → `parse` → `verify` / `toSummary` via `component_dry_run_hook` before any apply. See [Architect](../../architect/SKILL.md).

## Troubleshooting

| Symptom | Action |
|---------|--------|
| `Stale previewToken` | Re-preview after file changes |
| Empty `files` | Query matched nothing or already applied |
| `Missing required arguments` | `describe_recipe` / `describe_component` |
| `Invalid node type "class_declaration"` | Use `class_definition` (Dart pack) — [language-support.md](../../docs/03_queries_and_languages/02_language_support/00_index.md) |

## Related

- [codemod-mcp-playbook](../../codemod-mcp-playbook/SKILL.md) — compact tool list
- [intel](../intel/SKILL.md) — scan / locate first
