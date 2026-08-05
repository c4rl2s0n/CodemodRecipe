# Documentation

Human-oriented docs for **codemod-recipe**. Start here if you are new:

**[Getting Started](getting-started.md)** — mental model, setup, core concepts, first apply.

Then **[Writing recipes](writing-recipes.md)** — how to author YAML (query / capture / anchor,
step types, workflow).

Agent skills and Cursor rules are separate (installed into consumer projects by
`bootstrap_project` from `export/`). They are **not** a substitute for these pages,
and these human docs are **not** copied by bootstrap.

## Learning path

1. [getting-started.md](getting-started.md) — what it is, setup, first apply
2. [writing-recipes.md](writing-recipes.md) — author recipes end to end
3. [tree-sitter-queries.md](tree-sitter-queries.md) — query language depth
4. [recipe-templates.md](recipe-templates.md) — Jinja / maps / variables
5. [generated/dsl-vocabulary.md](generated/dsl-vocabulary.md) — every DSL field (generated)
6. [recipe-design-patterns.md](recipe-design-patterns.md) — naming and composition

## Setup and clients

| Doc | Contents |
|-----|----------|
| [getting-started.md](getting-started.md) | Day-1 walkthrough (VS Code + MCP) |
| [new-project-rust-mcp.md](new-project-rust-mcp.md) | New project MCP quickstart and bootstrap |
| [codemod-mcp.md](codemod-mcp.md) | MCP server setup, tools, preview → apply |
| [../vscode_extension/README.md](../vscode_extension/README.md) | Extension install, UI, language toolkit |

## Recipe authoring

| Doc | Contents |
|-----|----------|
| [writing-recipes.md](writing-recipes.md) | Authoring workflow, capture/anchor, step types |
| [generated/dsl-vocabulary.md](generated/dsl-vocabulary.md) | Complete field / enum catalog (from schema) |
| [recipe-templates.md](recipe-templates.md) | MiniJinja / Jinja2 templates, filters, maps/vars |
| [tree-sitter-queries.md](tree-sitter-queries.md) | Query language, captures, anchors, predicates; **Query Tools** (VS Code + MCP) |
| [language-support.md](language-support.md) | `language:` field, inference, SQL dialects |
| [recipe-design-patterns.md](recipe-design-patterns.md) | create / add / scaffold / remove taxonomy |

Root [README.md](../README.md) also shows a compact YAML example and feature status.

## Extension power features

| Doc | Contents |
|-----|----------|
| [tree-sitter-queries.md — Query Tools](tree-sitter-queries.md#query-tools-vs-code) | AST browse, generate/run queries, CodeLens, MCP debug tools |
| [recipe-shortcuts.md](recipe-shortcuts.md) | Slots, `from` / `contextKey`, explorer menu |

## Architecture and contributing

| Doc | Contents |
|-----|----------|
| [../ARCHITECTURE.md](../ARCHITECTURE.md) | Components and data flow |
| [../CONTRIBUTING.md](../CONTRIBUTING.md) | Build, test, skills/rules sync |

Maintainers editing this set: see skill `codemod-recipe-human-docs` under
`.cursor/skills/`.
