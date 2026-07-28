---
name: codemod-overview
description: Orientation for codemod-recipe — what it is, when to use MCP tools vs direct edits, project layout, and multi-language support.
---

# Codemod Recipe Overview

## What it is

**codemod-recipe** applies deterministic, AST-safe edits using declarative YAML recipes . The Rust MCP server exposes preview/apply tools for agents.

**Languages:** Dart is the default, with lazy-loaded support for Rust, Java, Kotlin, SQL/SQLite, and 300+ grammars via tree-sitter-language-pack. See skill `codemod-languages`.

## When to use

- Structural edits: insert, replace, remove at AST boundaries
- Multi-file scaffolding: create files, patch existing files, delete legacy files
- Repeatable, idempotent transformations with preview before apply
- Non-Dart files when you set `edit.language` and author per-language queries

## When not to use

- General codebase exploration (use codebase-memory-mcp)
- Trivial text edits without AST safety (edit directly)
- Guessing AST paths without a tree-sitter query
- Reusing the same query across different languages

## Project layout

```text
.codemod/
  recipes/     # recommended location for recipes (*.yaml with steps)
  maps/        # recommended location for maps (id + map:)
  variables/   # recommended location for variables (id + values:)
  templates/   # optional files for create.templateFile
```

Asset discovery is **schema-based** anywhere under `.codemod/` (dirs are convention only).
Template access: `{{ map.<id>.<key> }}`, `{{ var.<id>.<key> }}`, `{{ key | map('id') }}`.

Centralized path/resource resolution lives in `rust/crates/core/src/resource_path.rs`:

- workspace file targets resolve exactly under the workspace root
- recipe resources (`query` `.scm`, `create.templateFile`, `postExecution`,
  template `extends` / `include`) resolve recipe-local first, then `.codemod/`
- bare query names also try `queries/` under each root
- YAML query libraries stay id-based under `.codemod/queries/*.yaml`

For recipe organization (create vs modify, scaffolds), invoke skill `codemod-recipe-design-patterns`.

Skills use thin SKILL.md routers; read `reference.md` in a skill directory when you need full detail.

## Core workflow

1. `validate_recipes` after editing recipe YAML
2. `list_recipes` / `describe_recipe` to learn args
3. `preview_recipe` (save `previewToken`)
4. `apply_recipe` with same recipe/args + token
5. Re-preview — expect empty `files` if idempotent

## Bootstrap profiles

`bootstrap_project` always installs skills + scaffolding. Rule packs:

- `edit_policy: "recommend"` (default) — soft prefer recipes + preview
- `edit_policy: "strict"` — recipe-first; in-body direct edit only; discuss before new recipes/templates
- `companions: ["codebase-memory"]` — optional MCP-first navigation (independent of edit policy)

## Related skills

| Skill | Use for |
|-------|---------|
| `codemod-languages` | `language:` field, SQL dialects, grammar loading |
| `codemod-recipe-design-patterns` | create vs modify taxonomy |
| `codemod-yaml-dsl` | recipe YAML syntax |
| `codemod-mcp-playbook` | MCP tool reference |
| `codemod-tree-sitter-queries` | query syntax, captures, predicates |
| `codemod-recipe-authoring` | Dart query patterns (pack grammar) |
| `recipe-generation` | Generate recipes from `@` code refs |
