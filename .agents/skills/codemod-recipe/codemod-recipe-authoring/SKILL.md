---
name: codemod-recipe-authoring
description: >-
  Practical Luau recipe authoring — queries, captures, anchors, idempotency.
  Index into usage docs.
---

# Recipe authoring

| Doc | Contents |
|-----|----------|
| [writing-recipes.md](../docs/02_writing_recipes/01_workflow/00_index.md) | Workflow + worked examples |
| [tree-sitter-queries.md](../docs/03_queries_and_languages/01_tree_sitter_queries/00_index.md) | Query language |
| [host-api.md](../docs/02_writing_recipes/02_host_api/00_index.md) | Edit specs |

## Testing loop

`validate_recipes` → `preview_recipe` → adjust query/text → re-preview until empty when idempotent.

Parent: [Architect](../architect/SKILL.md)
