---
name: recipe-generation
description: >-
  Generate Luau codemod recipes from user-referenced code (@files, symbols,
  selections). Checks existing recipes for reuse or composition, infers
  parameters and confirms with the user, then writes under .codemod/recipes/
  and validates/previews. Use when the user runs /recipe-generation or asks
  to generate a recipe from referenced code.
---

# Recipe Generation

Turn `@`-referenced code into a registered **Luau recipe** under `.codemod/recipes/`.

## When to use

- User runs `/recipe-generation` with `@` file/symbol/selection refs
- User asks to generate a recipe from example or selected code

## When not to use

- Hand-tuning an existing recipe's query → `codemod-recipe-authoring`
- Running preview/apply on a known recipe id → `codemod-mcp-playbook`

## Workflow

```text
Recipe generation:
- [ ] 1. Ingest @refs / selection
- [ ] 2. Survey existing recipes (list_recipes)
- [ ] 3. Decide: reuse / compose / new
- [ ] 4. Infer args + taxonomy prefix
- [ ] 5. Confirm proposal with user (required)
- [ ] 6. Write .luau (or stop if reuse-only)
- [ ] 7. validate_recipes → preview_recipe
- [ ] 8. Hand off — do not apply unless asked
```

### 6. Author

Write `.codemod/recipes/<id>.luau` with `return { id, args, apply = function(c) ... end }`.

- Module shape / host API: skill `codemod-luau-recipes`
- Queries: `codemod-recipe-authoring`, `codemod-tree-sitter-queries`
- Non-Dart: set `language` on ops — skill `codemod-languages`

### 7. Verify

1. `validate_recipes`
2. `preview_recipe` with sample args
3. Fix until `files` / `transcript` look right

**Do not** call `apply_recipe` unless the user explicitly asks.

## Related skills

| Skill | Use for |
|-------|---------|
| `codemod-luau-recipes` | recipe module shape |
| `codemod-recipe-design-patterns` | taxonomy |
| `codemod-recipe-authoring` | query patterns |
| `codemod-mcp-playbook` | validate / preview tools |
