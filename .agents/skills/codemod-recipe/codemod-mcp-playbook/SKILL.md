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
| `validate_recipes` / `reload_recipes` | Validate / reload catalog from disk |
| `preview_recipe` / `diff_recipe` / `apply_recipe` | Dry-run (full or one `path`), then apply (`previewToken` on apply); `inlineRecipe` = Luau **string** |
| `derive_args` / `filter_explorer_recipes` | Args from cursor/selection; explorerMenu path match |
| `dump_ast` / `debug_query` / `generate_query` / `query_captures` | Query Tools — **dump_ast before authoring queries** (`namedOnly: false` for keywords) |
| `resolve_static_path` | Jinja-render a static path template |
| `component_scan` / `component_incremental_scan` | Stack instances (**no** `c:log` / transcript) |
| `list_component_instances` | Alias of `component_scan` — prefer `component_scan` |
| `component_coverage` / `component_graph` / `component_snapshot` | Coverage, mermaid, snapshot JSON |
| `describe_component` | Hooks / ops |
| `component_dry_run_hook` / `component_apply_hook` | Dry-run / apply; apply needs `previewToken` |
| `stack_dry_run_operation` / `stack_apply_operation` | Stack fan-out ops; apply needs `previewToken` |
| `run_tests` | `*.test.luau` suites (temp fixtures) |

## Debug tips

- `c:log` → dry-run / preview `logs` (not scan)
- Queries → `dump_ast` first, then `debug_query` / `query_captures`
- Prefer `hit.text` / `c:spanText` over Lua `string.sub` on spans
- Mutating tools always need a matching dry-run `previewToken`

## Workflows

- Apply recipes/components → [Builder apply](../builder/apply/SKILL.md)
- Scan / locate → [Builder intel](../builder/intel/SKILL.md)
- Author assets → [Architect](../architect/SKILL.md)
