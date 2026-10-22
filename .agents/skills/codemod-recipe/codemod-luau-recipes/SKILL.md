---
name: codemod-luau-recipes
description: >-
  Luau recipe module shape and host API for codemod-recipe. Read usage docs —
  this skill is an index only.
---

# Luau recipes

## Docs (SSOT)

| Doc | Contents |
|-----|----------|
| [host-api.md](../docs/02_writing_recipes/02_host_api/00_index.md) | Module table, `c:*` methods, preview vs apply |
| [writing-recipes.md](../docs/02_writing_recipes/01_workflow/00_index.md) | Authoring workflow, capture/anchor |
| [recipe-templates.md](../docs/04_templates_and_patterns/01_recipe_templates/00_index.md) | MiniJinja |

## Checklist

1. `return { id, args?, apply = function(c, args) ... }`
2. Mutations only in `apply`
3. `validate_recipes` → `preview_recipe` → fix; apply only if asked

Parent: [Architect](../architect/SKILL.md)
