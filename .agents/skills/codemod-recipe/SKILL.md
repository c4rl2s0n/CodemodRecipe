---
name: codemod-recipe
description: >-
  Top-level skill for codemod-recipe: deterministic, AST-safe code edits via
  sandboxed Luau recipes and the Rust MCP host. Use this skill when you are
  authoring, reviewing, or running codemod recipes, then read the relevant
  nested sub-skill for the specific topic.
---

# Codemod Recipe

## What it is

codemod-recipe applies deterministic, AST-safe edits using **sandboxed Luau recipes**
(any `.luau` or `.lua` module under `.codemod/` that returns `{ id, apply = function(c) ... }`).
The `.codemod/recipes/` directory is a convention, not a requirement. The Rust MCP server exposes
preview/apply tools for agents. Dart is the default language; set `language` on edit ops for non-Dart files.

## When to use

- Structural edits, multi-file scaffolds, or AST-boundary changes.
- Non-Dart files when you set `language` on the edit op.
- Validating, previewing, and applying Luau recipes.
- Creating or improving reusable recipes and templates.

## When not to use

- General codebase exploration → use **codebase-memory** first.
- Trivial text-only edits where a recipe would not help → edit directly.

## Core workflow

1. `validate_recipes` after editing Luau.
2. `list_recipes` / `describe_recipe` to learn recipe args.
3. `preview_recipe` (save the `previewToken`).
4. `apply_recipe` with the same args and token.
5. Re-preview to verify idempotency (`files: []`).

## Nested skills

Read the relevant sub-skill for details:

| Skill | Use for | Read |
|-------|---------|------|
| `codemod-overview` | Orientation and project layout | [SKILL.md](codemod-overview/SKILL.md) |
| `codemod-luau-recipes` | Recipe module shape and host API | [SKILL.md](codemod-luau-recipes/SKILL.md), [reference.md](codemod-luau-recipes/reference.md) |
| `codemod-mcp-playbook` | MCP tool workflow | [SKILL.md](codemod-mcp-playbook/SKILL.md) |
| `codemod-languages` | Language IDs, SQL dialects, grammar notes | [SKILL.md](codemod-languages/SKILL.md), [reference.md](codemod-languages/reference.md) |
| `codemod-tree-sitter-queries` | Query syntax, captures, predicates | [SKILL.md](codemod-tree-sitter-queries/SKILL.md), [reference.md](codemod-tree-sitter-queries/reference.md) |
| `codemod-recipe-design-patterns` | Create vs modify taxonomy and scaffolds | [SKILL.md](codemod-recipe-design-patterns/SKILL.md), [reference.md](codemod-recipe-design-patterns/reference.md) |
| `codemod-recipe-authoring` | Dart patterns, idempotency, testing | [SKILL.md](codemod-recipe-authoring/SKILL.md) |
| `recipe-generation` | Generate recipes from `@` references | [SKILL.md](recipe-generation/SKILL.md), [reference.md](recipe-generation/reference.md) |

## Human docs

- [docs/getting-started.md](../../../../docs/getting-started.md) — day-1 overview
- [docs/writing-recipes.md](../../../../docs/writing-recipes.md) — recipe authoring
