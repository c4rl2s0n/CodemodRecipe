# Documentation

Human-oriented docs for **codemod-recipe**. Start here if you are new:

**[Getting Started](getting-started.md)** — mental model, setup, core concepts, first apply.

Agent skills and Cursor rules are separate (installed into consumer projects by
`bootstrap_project` from `export/`). They are **not** a substitute for these pages,
and these human docs are **not** copied by bootstrap.

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
| [recipe-templates.md](recipe-templates.md) | MiniJinja / Jinja2 templates, filters, maps/vars |
| [tree-sitter-queries.md](tree-sitter-queries.md) | Query language, captures, anchors, predicates |
| [language-support.md](language-support.md) | `language:` field, inference, SQL dialects |
| [recipe-design-patterns.md](recipe-design-patterns.md) | create / add / scaffold / remove taxonomy |

Root [README.md](../README.md) also shows a compact YAML example and feature status.

## Extension power features

| Doc | Contents |
|-----|----------|
| [recipe-shortcuts.md](recipe-shortcuts.md) | Slots, `from` / `contextKey`, explorer menu |

## Architecture and contributing

| Doc | Contents |
|-----|----------|
| [../ARCHITECTURE.md](../ARCHITECTURE.md) | Components and data flow |
| [../CONTRIBUTING.md](../CONTRIBUTING.md) | Build, test, skills/rules sync |

Maintainers editing this set: see skill `codemod-recipe-human-docs` under
`.cursor/skills/`.
