---
name: codemod-overview
description: Orientation for codemod-recipe — what it is, when to use MCP tools vs direct edits, project layout, and multi-language support.
---

# Codemod Recipe Overview

## What it is

**codemod-recipe** applies deterministic, AST-safe edits using declarative YAML recipes (dslVersion 2). The Rust MCP server exposes preview/apply tools for agents.

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
  recipes/    # registered YAML recipes (*.yaml)
  maps/       # optional string maps for {{ key | map('mapId') }}
  templates/  # optional files for create.templateFile
```

For recipe organization (create vs modify, scaffolds), invoke skill `codemod-recipe-design-patterns`.

Skills use thin SKILL.md routers; read `reference.md` in a skill directory when you need full detail.

## Core workflow

1. `validate_recipes` after editing recipe YAML
2. `list_recipes` / `describe_recipe` to learn args
3. `preview_recipe` (save `previewToken`)
4. `apply_recipe` with same recipe/args + token
5. Re-preview — expect empty `files` if idempotent

## Related skills

| Skill | Use for |
|-------|---------|
| `codemod-languages` | `language:` field, SQL dialects, grammar loading |
| `codemod-recipe-design-patterns` | create vs modify taxonomy |
| `codemod-yaml-dsl-v2` | recipe YAML syntax |
| `codemod-mcp-playbook` | MCP tool reference |
| `codemod-tree-sitter-queries` | query syntax, captures, predicates |
| `codemod-recipe-authoring` | Dart query patterns (pack grammar) |
