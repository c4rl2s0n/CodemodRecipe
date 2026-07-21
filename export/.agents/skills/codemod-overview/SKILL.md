---
name: codemod-overview
description: Orientation for codemod-recipe — what it is, when to use MCP tools vs direct edits, and project layout.
---

# Codemod Recipe Overview

## What it is

**codemod-recipe** applies deterministic, AST-safe Dart edits using declarative YAML recipes (dslVersion 2). The Rust MCP server exposes preview/apply tools for agents.

## When to use

- Structural Dart edits: insert, replace, remove at AST boundaries
- Multi-file scaffolding: create files, patch existing files, delete legacy files
- Repeatable, idempotent transformations with preview before apply

## When not to use

- General codebase exploration (use codebase-memory-mcp)
- Non-Dart files or trivial text edits (edit directly)
- Guessing AST paths without a tree-sitter query

## Project layout

```text
.codemod/
  recipes/    # registered YAML recipes (*.yaml)
  maps/       # optional string maps for {{$map ...}}
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
| `codemod-recipe-design-patterns` | create vs modify taxonomy |
| `codemod-yaml-dsl-v2` | recipe YAML syntax |
| `codemod-mcp-playbook` | MCP tool reference |
| `codemod-recipe-authoring` | tree-sitter queries |
