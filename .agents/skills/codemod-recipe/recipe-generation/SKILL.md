---
name: recipe-generation
description: >-
  Generate Luau codemod recipes from user-referenced code (@files, symbols,
  selections). Confirm args with the user; validate and preview; do not apply
  unless asked.
---

# Recipe generation

Turn `@`-referenced code into a registered Luau recipe under `.codemod/recipes/`.

## Workflow

1. Ingest `@` refs / selection
2. `list_recipes` — reuse / compose / new?
3. Infer args + taxonomy prefix ([recipe-design-patterns.md](../docs/04_templates_and_patterns/02_recipe_design_patterns/00_index.md))
4. **Confirm proposal with the user**
5. Write `.luau` using [host-api.md](../docs/02_writing_recipes/02_host_api/00_index.md) / [writing-recipes.md](../docs/02_writing_recipes/01_workflow/00_index.md)
6. `validate_recipes` → `preview_recipe`
7. Hand off — **do not apply unless asked**

Parent: [Architect](../architect/SKILL.md)
