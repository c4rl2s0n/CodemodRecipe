---
name: codemod-recipe-builder-intel
description: >-
  Understand a workspace with codemod-recipe: component scan, recipe discovery,
  AST/query debug tools, and optional codebase-memory. Read-only by default.
---

# Builder — Intel

Gather structure **before** editing. Prefer summaries over raw `ast` payloads.

## Workflow

1. **Locate** — if [codebase-memory-mcp](https://github.com/DeusData/codebase-memory-mcp) is configured, find symbols/files first; otherwise use recipes/components you already know
2. **Stack present?** (`.codemod/stack.luau`) → `component_scan` / `list_component_instances`
3. After known edits → `component_incremental_scan` with `dirtyPaths` (or `detectChanges: true`)
4. **Coverage blind spots?** → `component_coverage` (needs `stack.codebase`; span-only — see [coverage](../../docs/05_components_and_stacks/01_components/09_coverage.md))
5. **Recipes** → `list_recipes` / `describe_recipe` for args and descriptions
6. **Query stuck?** → `dump_ast` / `debug_query` / `generate_query` ([tree-sitter-queries.md](../../docs/03_queries_and_languages/01_tree_sitter_queries/00_index.md))

## Reasoning tips

- Use instance **`summary`** / shallow fields for planning
- Use **`ast`** / `source` spans only when you need exact edit sites
- Empty scan → check stack roots and `identify` (Architect may need to fix the component)

## Docs

- [components.md](../../docs/05_components_and_stacks/01_components/00_index.md) — scan / instances / findings
- [codemod-mcp.md](../../docs/06_mcp_and_clients/01_codemod_mcp/00_index.md) — tool parameters
- [stacks.md](../../docs/05_components_and_stacks/02_stacks/00_index.md) — stack manifest

## Next

Ready to change code → [../apply/SKILL.md](../apply/SKILL.md)
