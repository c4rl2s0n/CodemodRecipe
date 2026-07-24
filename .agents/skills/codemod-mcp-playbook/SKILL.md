---
name: codemod-mcp-playbook
description: MCP tool reference and agent workflow for codemod-recipe preview/apply with YAML v2 recipes and inlineRecipe.
---

# Codemod MCP Playbook

MCP server `codemod-mcp` returns JSON text. Parse output and branch on `ok`.

## Tools

| Tool | Purpose |
|------|---------|
| `bootstrap_project` | Install `.agents/skills/`, `.cursor/rules/`, `.codemod/` scaffolding (`edit_policy`, optional `companions`) |
| `list_recipes` | Discover registered recipe ids |
| `describe_recipe` | Args + metadata for one recipe |
| `validate_recipes` | Reload + validate recipes/maps/variables under `.codemod/` |
| `preview_recipe` | Dry-run; returns `previewToken` |
| `apply_recipe` | Atomic apply; requires `previewToken` |

## Response envelope

```json
{ "ok": true, "...": "..." }
{ "ok": false, "error": "message" }
```

## preview_recipe

Provide one of `recipe` (id) or `inlineRecipe` (v2 object).

Optional: `args` (string map), `snippetLines` (number).

Returns `previewToken` and `files[]` with patch previews.

## apply_recipe

Required: `previewToken` from matching preview.

Also: same `recipe` or `inlineRecipe`, same `args`.

Optional: `selection` for partial patch apply.

Optional host flags: `--sql-default` (default `sqlite` for `.sql` files). See skill `codemod-languages`.

## inlineRecipe shape (v2)

```json
{
  "id": "__inline_id",
  "steps": [{
    "edit": {
      "path": "lib/file.dart",
      "language": "dart",
      "ops": [{
        "insert": {
          "query": "(class_definition ... @body)",
          "capture": "body",
          "anchor": "end",
          "text": "    print('ok');\n"
        }
      }]
    }
  }]
}
```

- `insert`: `query`, `capture`, `anchor` (`start`|`end`), `text`
- `replace`: `query`, `capture`, `text`
- `remove`: `query`, `capture`

## Agent workflow

1. Locate target (codebase-memory if available)
2. Pick registered recipe or craft `inlineRecipe`
3. `preview_recipe` → inspect `files[]`
4. `apply_recipe` with token
5. Re-preview for idempotency

## Troubleshooting

- `Stale previewToken` — re-preview after file changes
- Empty `files` — query matched nothing or already applied
- `Missing required arguments` — check `describe_recipe`
- `Invalid node type "class_declaration"` — use `class_definition` (pack Dart); see `codemod-languages`
- `unknown language` — set valid `language:` id; see `codemod-languages`

## Related skills

| Skill | Use for |
|-------|---------|
| `codemod-languages` | `language:` field, SQL dialects, grammar loading |
| `codemod-yaml-dsl-v2` | inlineRecipe step shape |
| `codemod-recipe-authoring` | tree-sitter queries |
