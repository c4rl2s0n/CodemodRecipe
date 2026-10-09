---
name: codemod-mcp-playbook
description: >-
  Compact MCP tool checklist for codemod-recipe. Full encyclopedia is usage
  docs/06_mcp_and_clients/01_codemod_mcp/00_index.md; apply workflow is Builder apply.
---

# MCP playbook

Full tool reference: [codemod-mcp.md](../docs/06_mcp_and_clients/01_codemod_mcp/00_index.md)

## Tools (short)

| Tool | Purpose |
|------|---------|
| `bootstrap_project` | Install skills, rules, usage `docs/`, `.codemod/` scaffold |
| `list_recipes` / `describe_recipe` | Discover recipes |
| `validate_recipes` | Reload + validate Luau under `.codemod/` |
| `preview_recipe` / `apply_recipe` | Dry-run then apply (`previewToken`); read `logs` / `transcript` |
| `dump_ast` / `debug_query` / `generate_query` | Query Tools — **dump_ast before authoring queries** (`namedOnly: false` for keywords) |
| `component_scan` / `component_incremental_scan` | Stack instances (**no** `c:log` / transcript) |
| `component_coverage` | Span coverage vs `stack.codebase` (skips if missing) |
| `describe_component` | Hooks / ops |
| `component_dry_run_hook` / `component_apply_hook` | Dry-run / apply; `logs[]` + `transcript` for `c:log` |

## Debug tips

- `c:log` → dry-run / preview `logs` (not scan)
- Queries → `dump_ast` first, then `debug_query`
- Prefer `hit.text` / `c:spanText` over Lua `string.sub` on spans

## Workflows

- Apply recipes/components → [Builder apply](../builder/apply/SKILL.md)
- Scan / locate → [Builder intel](../builder/intel/SKILL.md)
- Author assets → [Architect](../architect/SKILL.md)
