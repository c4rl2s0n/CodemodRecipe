# Codemod Recipe — Usage docs

Product documentation for using **codemod-recipe** (recipes, components, MCP).
Framework-contributor docs live under [`../develop/`](../develop/).

Agents receive these pages via bootstrap under `.agents/skills/codemod-recipe/docs/`.

## Browse with mdBook (optional)

Interactive HTML with previous/next navigation and Mermaid diagrams.

**This repository** (usage + [`develop/`](../develop/)):

```bash
cargo install mdbook mdbook-mermaid
cd docs
mdbook-mermaid install .
mdbook serve
```

**Consumer workspaces** (usage only, after `bootstrap_project`):

```bash
cd .agents/skills/codemod-recipe/docs
mdbook-mermaid install .
mdbook serve
```

You need **both** `mdbook` and `mdbook-mermaid` (preprocessor). The install step is local-only; do not commit `mermaid*.js` or the generated `book/` HTML directory.

## Learning path

1. [Getting started](01_getting_started/00_index.md)
2. [Writing recipes](02_writing_recipes/00_index.md) — [workflow](02_writing_recipes/01_workflow/00_index.md) + [host API](02_writing_recipes/02_host_api/00_index.md)
3. [Queries and languages](03_queries_and_languages/00_index.md)
4. [Templates and patterns](04_templates_and_patterns/00_index.md)
5. [Components and stacks](05_components_and_stacks/00_index.md)
6. [MCP and clients](06_mcp_and_clients/00_index.md)

## Chapters

| Chapter | Contents |
|---------|----------|
| [01_getting_started](01_getting_started/00_index.md) | Mental model, setup, first apply |
| [02_writing_recipes](02_writing_recipes/00_index.md) | Authoring workflow and host API |
| [03_queries_and_languages](03_queries_and_languages/00_index.md) | Tree-sitter queries, `language` |
| [04_templates_and_patterns](04_templates_and_patterns/00_index.md) | MiniJinja, taxonomy, shortcuts |
| [05_components_and_stacks](05_components_and_stacks/00_index.md) | Platform components |
| [06_mcp_and_clients](06_mcp_and_clients/00_index.md) | MCP tools and new-project quickstart |
