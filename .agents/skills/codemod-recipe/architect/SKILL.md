---
name: codemod-recipe-architect
description: >-
  Create or modify codemod-recipe assets: Luau recipes, components, stacks,
  queries, templates, maps. Index into usage docs — not a second API manual.
---

# Architect

Author **`.codemod` assets**. Do not invent API — read the linked usage docs.

## When to use

- Write or edit a recipe / component / query library / template / map
- User `@`-references code and wants a suitable recipe (also see [recipe-generation](../recipe-generation/SKILL.md))
- Debug a failing query or component hook during authoring

## When not to use

- Applying an existing recipe/component to app code → [Builder](../builder/SKILL.md)
- Exploring application structure only → [Builder intel](../builder/intel/SKILL.md)

## Decision guide

| Goal | Prefer | Docs |
|------|--------|------|
| One coherent AST/file change | Recipe with `apply(c, args)` | [writing-recipes.md](../docs/02_writing_recipes/01_workflow/00_index.md), [host-api.md](../docs/02_writing_recipes/02_host_api/00_index.md) |
| Greenfield files / feature workflow | `create_*` / `scaffold_*` taxonomy | [recipe-design-patterns.md](../docs/04_templates_and_patterns/02_recipe_design_patterns/00_index.md) |
| Identify / parse / mutate domain instances | Component + stack | [components.md](../docs/05_components_and_stacks/01_components/00_index.md), [stacks.md](../docs/05_components_and_stacks/02_stacks/00_index.md), [component-contract.md](../docs/05_components_and_stacks/03_component_contract/00_index.md) |
| Query syntax / captures | Tree-sitter query | [tree-sitter-queries.md](../docs/03_queries_and_languages/01_tree_sitter_queries/00_index.md) |
| Non-Dart file | Set `language` on ops | [language-support.md](../docs/03_queries_and_languages/02_language_support/00_index.md) |
| MiniJinja text | `c:template` | [recipe-templates.md](../docs/04_templates_and_patterns/01_recipe_templates/00_index.md) |
| Typed authoring | Luau stubs | [luau-lsp.md](../docs/05_components_and_stacks/04_luau_lsp/00_index.md) |

## Authoring checklist

1. Survey existing recipes/components (`list_recipes`, `describe_component`) — reuse or compose first
2. Write under `.codemod/` (conventional dirs: `recipes/`, `components/`, `queries/`, …)
3. `validate_recipes`
4. Dry-run: `preview_recipe` or `component_dry_run_hook` (`identify` → `parse` → op)
5. Fix until `files` / `transcript` look right — **do not apply unless the user asks**

## Prompt templates

- `@selection — write a Luau recipe that makes this change idempotently; confirm args first`
- `Add a component operation addX for scanned instances of component Y; dry-run parse first`
- `Turn this query into a registered recipe under .codemod/recipes/`

## Leaf topic routers

| Skill | Topic |
|-------|-------|
| [codemod-luau-recipes](../codemod-luau-recipes/SKILL.md) | Module shape + host API links |
| [codemod-recipe-authoring](../codemod-recipe-authoring/SKILL.md) | Query patterns / idempotency |
| [codemod-recipe-design-patterns](../codemod-recipe-design-patterns/SKILL.md) | Taxonomy |
| [codemod-tree-sitter-queries](../codemod-tree-sitter-queries/SKILL.md) | Query language |
| [codemod-languages](../codemod-languages/SKILL.md) | Language ids |
| [recipe-generation](../recipe-generation/SKILL.md) | Generate from `@` refs |
