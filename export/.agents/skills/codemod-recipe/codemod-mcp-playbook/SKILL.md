---
name: codemod-mcp-playbook
description: MCP tool reference and agent workflow for codemod-recipe preview/apply with Luau recipes and inlineRecipe.
---

# Codemod MCP Playbook

MCP server `codemod-mcp` returns JSON text. Parse output and branch on `ok`.

## Tools

| Tool | Purpose |
|------|---------|
| `bootstrap_project` | Install `.agents/skills/`, `.cursor/rules/`, `.codemod/` scaffolding |
| `list_recipes` | Discover registered recipe ids |
| `describe_recipe` | Args + metadata for one recipe |
| `validate_recipes` | Reload + validate Luau under `.codemod/` |
| `preview_recipe` | Dry-run; returns `previewToken` + `transcript` |
| `apply_recipe` | Atomic apply; requires `previewToken` |
| `dump_ast` | Dump AST for Query Tools / debugging |
| `debug_query` | Run tree-sitter query; return captures |
| `generate_query` | Generate starter query from byte range |
| `resolve_static_path` | Probe whether a Jinja path needs args |
| `component_scan` | Scan stack root components (instances, findings, summaries); populates in-process cache |
| `component_incremental_scan` | Rescan after edits: `dirtyPaths` (tier 1), `detectChanges` (stat + glob-diff, tier 2), or `force` (full scan) |
| `list_component_instances` | Alias of `component_scan` |

## Response envelope

```json
{ "ok": true, "...": "..." }
{ "ok": false, "error": "message" }
```

## preview_recipe

Provide one of `recipe` (id) or `inlineRecipe` (Luau source string).

Optional: `args` (JSON values: strings, bools, lists, objects), `snippetLines` (number).

Returns `previewToken`, `files[]`, and `transcript[]`.

## apply_recipe

Required: `previewToken` from matching preview.

Also: same `recipe` or `inlineRecipe`, same `args`.

Optional: `selection` for partial patch apply.

## inlineRecipe

Luau source that `return`s a recipe table with `apply`:

```json
{
  "inlineRecipe": "return { id = \"__inline\", apply = function(c, args) c:insert({ path = \"lib/a.dart\", query = \"(program) @root\", capture = \"root\", anchor = \"end\", text = \"// x\\n\" }) end }"
}
```

## Preview vs apply

- Both run `apply(c, args)`. Preview skips postExec (transcript only).
- Apply re-runs Luau and flushes before postExec.
- When apply may diverge, UI shows `applyMayDiverge`.

## Agent workflow

1. Locate target (codebase-memory if available)
2. Pick registered recipe or craft `inlineRecipe`
3. `preview_recipe` → inspect `files[]` and `transcript[]`
4. `apply_recipe` with token
5. Re-preview for idempotency

## Platform components (stacks)

When `.codemod/stack.luau` and components exist:

1. `component_scan` (or `list_component_instances`) — initial read; populates host cache
2. After codemod apply or known file edits, prefer `component_incremental_scan` with `dirtyPaths` (or `detectChanges: true` when paths unknown)
3. Use summaries for reasoning; use `node` for detail
4. Create via host `componentScaffold` (`scaffold.apply(c, args)`); mutate via `componentInvokeOperation` (`operations.*.apply(c, args, target)`). MCP not wired for these yet; host incremental-rescans after apply

See `docs/components.md`, `docs/stacks.md`, and `docs/flutter-poc.md`.

## Troubleshooting

- `Stale previewToken` — re-preview after file changes
- Empty `files` — query matched nothing or already applied
- `Missing required arguments` — check `describe_recipe`
- `Invalid node type "class_declaration"` — use `class_definition` (pack Dart)

## Related skills

| Skill | Use for |
|-------|---------|
| `codemod-luau-recipes` | recipe shape and host API |
| `codemod-languages` | `language` on ops |
| `codemod-recipe-authoring` | tree-sitter queries |
