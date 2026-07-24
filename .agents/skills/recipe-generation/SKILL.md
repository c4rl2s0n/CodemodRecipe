---
name: recipe-generation
description: >-
  Generate YAML v2 codemod recipes from user-referenced code (@files, symbols,
  selections). Checks existing recipes for reuse or composition, infers
  parameters and confirms with the user, then writes under .codemod/recipes/
  and validates/previews. Use when the user runs /recipe-generation or asks
  to generate a recipe from referenced code.
---

# Recipe Generation

Turn `@`-referenced code into a registered YAML v2 recipe under `.codemod/recipes/`.

## When to use

- User runs `/recipe-generation` with `@` file/symbol/selection refs
- User asks to generate a recipe from example or selected code

## When not to use

- Hand-tuning an existing recipe’s query → `codemod-recipe-authoring`
- Running preview/apply on a known recipe id → `codemod-mcp-playbook`
- Choosing create vs modify taxonomy only → `codemod-recipe-design-patterns`

## Workflow

Copy this checklist and track progress:

```text
Recipe generation:
- [ ] 1. Ingest @refs / selection
- [ ] 2. Survey existing recipes (list_recipes)
- [ ] 3. Decide: reuse / compose / new
- [ ] 4. Infer args + taxonomy prefix
- [ ] 5. Confirm proposal with user (required)
- [ ] 6. Write YAML (or stop if reuse-only)
- [ ] 7. validate_recipes → preview_recipe
- [ ] 8. Hand off — do not apply unless asked
```

### 1. Ingest references

Treat `@` attachments and selections as the intended target shape (or before/after).
Ask only if intent is unclear: insert vs replace vs remove vs create file(s).

### 2–3. Survey and decide

1. Call `list_recipes`.
2. For promising ids, call `describe_recipe`.
3. Prefer, in order:
   - **Reuse** — existing recipe already performs the edit → propose that id + args; do not invent a duplicate
   - **Compose** — several atomics cover the change → `scaffold_*` with `- recipe: …` steps
   - **New** — nothing fits → one atomic (`create_` / `add_` / `patch_` / `remove_`) per coherent change

Follow skill `codemod-recipe-design-patterns` for prefixes and layout.

### 4. Infer parameters

Apply heuristics in [reference.md](reference.md). Always confirm before writing.

### 5. Confirmation gate (required)

Present a short proposal and **wait for user OK** (or an edited arg list) before writing files.
Use the confirmation template in [reference.md](reference.md).

If the decision is **reuse only**, stop after confirmation — no new YAML.

### 6. Author

Write `.codemod/recipes/<id>.yaml` (or concept subdirs when the project already groups recipes).

- YAML shape: skill `codemod-yaml-dsl-v2`
- Queries / captures: `codemod-recipe-authoring`, `codemod-tree-sitter-queries`
- Non-Dart: set `edit.language` — skill `codemod-languages`

### 7. Verify

1. `validate_recipes`
2. `preview_recipe` with sample args from the confirmation
3. Fix query/args until patches look right

**Do not** call `apply_recipe` unless the user explicitly asks.

### 8. Hand off

Report: recipe id (or reused ids), final args, preview summary (`files[]` / empty).

## Related skills

| Skill | Use for |
|-------|---------|
| `codemod-recipe-design-patterns` | create vs modify / scaffold taxonomy |
| `codemod-yaml-dsl-v2` | recipe YAML syntax |
| `codemod-recipe-authoring` | Dart query patterns and testing |
| `codemod-tree-sitter-queries` | query syntax, captures, predicates |
| `codemod-languages` | `language:` field |
| `codemod-mcp-playbook` | validate / preview / apply tools |
