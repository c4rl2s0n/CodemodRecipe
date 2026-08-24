---
name: codemod-overview
description: Orientation for codemod-recipe — what it is, when to use MCP tools vs direct edits, project layout, and multi-language support.
---

# Codemod Recipe Overview

## What it is

**codemod-recipe** applies deterministic, AST-safe edits using **sandboxed Luau recipes**.
Each recipe `return`s metadata plus `apply(c)`. The Rust MCP server exposes preview/apply tools for agents.

**Languages:** Dart is the default, with lazy-loaded support for Rust, Java, Kotlin, SQL/SQLite, and 300+ grammars via tree-sitter-language-pack. See skill `codemod-languages`.

## When to use

- Structural edits: `c:insert`, `c:replace`, `c:remove` at AST boundaries
- Multi-file scaffolding: `c:create`, patch existing files, `c:delete`
- Repeatable, idempotent transformations with preview before apply
- Non-Dart files when you set `language` on edit ops and author per-language queries

## When not to use

- General codebase exploration (use codebase-memory-mcp)
- Trivial text edits without AST safety (edit directly)
- Guessing AST paths without a tree-sitter query

## Project layout

```text
.codemod/
  recipes/     # Luau recipe modules (*.luau / *.lua)
  maps/        # Luau map modules (id + map table)
  variables/   # Luau variable modules (id + values table)
  queries/     # Luau query libraries (id + queries table)
  templates/   # optional (renderFileTemplate)
```

Asset discovery scans `.codemod/` for Luau modules with expected return shapes.

Path resolution: `rust/crates/core/src/resource_path.rs` (recipe-local first, then `.codemod/`).

For recipe organization (create vs modify, scaffolds), invoke skill `codemod-recipe-design-patterns`.

## Core workflow

1. `validate_recipes` after editing recipe Luau
2. `list_recipes` / `describe_recipe` to learn args
3. `preview_recipe` (save `previewToken`; inspect transcript)
4. `apply_recipe` with same recipe/args + token
5. Re-preview — expect empty `files` if idempotent

## Related skills

| Skill | Use for |
|-------|---------|
| `codemod-luau-recipes` | recipe module shape and host API |
| `codemod-languages` | `language` on ops, SQL dialects |
| `codemod-recipe-design-patterns` | create vs modify taxonomy |
| `codemod-mcp-playbook` | MCP tool reference |
| `codemod-tree-sitter-queries` | query syntax, captures, predicates |
| `codemod-recipe-authoring` | Dart query patterns (pack grammar) |
| `recipe-generation` | Generate recipes from `@` code refs |

Skills use thin SKILL.md routers; read `reference.md` when you need full detail.
