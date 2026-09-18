---
name: codemod-recipe
description: >-
  Top-level skill for codemod-recipe: deterministic AST-safe edits via sandboxed
  Luau recipes and the Rust MCP host. Start here, then open Architect (create
  or modify .codemod assets) or Builder (understand a workspace / apply changes).
---

# Codemod Recipe

## What it is

codemod-recipe applies deterministic, AST-safe edits using **sandboxed Luau recipes**
and **components**. The Rust MCP server exposes validate / preview / apply tools.
Usage docs (product SSOT) ship beside these skills under [`docs/`](docs/).

## Pick a mode

| Mode | When | Open |
|------|------|------|
| **Architect** | Create or modify recipes, components, queries, templates, maps | [architect/SKILL.md](architect/SKILL.md) |
| **Builder** | Understand a workspace or apply registered recipes/components | [builder/SKILL.md](builder/SKILL.md) |

## Core apply workflow (Builder)

1. Locate target (Builder intel / codebase-memory if available)
2. `describe_recipe` / `describe_component` for args
3. `preview_recipe` or `component_dry_run_hook` — save token / inspect `files` + `transcript`
4. `apply_recipe` / `component_apply_hook` with the same args
5. Re-preview / incremental scan for idempotency

## Usage docs (read these for depth)

| Doc | Contents |
|-----|----------|
| [docs/01_getting_started/00_index.md](docs/01_getting_started/00_index.md) | Day-1 overview |
| [docs/02_writing_recipes/02_host_api/00_index.md](docs/02_writing_recipes/02_host_api/00_index.md) | Host API (`c:insert`, maps, templates) |
| [docs/02_writing_recipes/01_workflow/00_index.md](docs/02_writing_recipes/01_workflow/00_index.md) | Authoring workflow |
| [docs/06_mcp_and_clients/01_codemod_mcp/00_index.md](docs/06_mcp_and_clients/01_codemod_mcp/00_index.md) | MCP tool encyclopedia |
| [docs/05_components_and_stacks/01_components/00_index.md](docs/05_components_and_stacks/01_components/00_index.md) | Components and hooks |

## Related workflow skills

| Skill | Use for |
|-------|---------|
| [recipe-generation](recipe-generation/SKILL.md) | Generate a recipe from `@` code refs |
| [codemod-mcp-playbook](codemod-mcp-playbook/SKILL.md) | Short MCP tool checklist (links to docs) |
