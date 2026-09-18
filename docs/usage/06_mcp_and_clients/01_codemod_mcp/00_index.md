# codemod-mcp (Rust Host)

MCP server that exposes the Rust `CodemodHost` protocol as tools for AI agents.
Use it to preview and apply deterministic AST edits from registered **Luau
recipes** (`.luau` or `.lua` modules anywhere under `.codemod/` that return `{ id, apply = function(c, args) ... }`)
or `inlineRecipe` Luau source strings. The `.codemod/recipes/` directory is a convention.
Supports Dart, Rust, Java, Kotlin, SQL/SQLite, and 300+ languages via lazy-loaded
tree-sitter grammars.

| Page | |
|------|---|
| [Quick start](01_quick_start.md) | |
| [Target workspace layout](02_target_workspace_layout.md) | |
| [Setup in Cursor](03_setup_in_cursor.md) | |
| [Multi-language support](04_multi_language_support.md) | |
| [Tools](05_tools.md) | |
| [End-to-end workflow](06_end_to_end_workflow.md) | |
| [Preview vs apply](07_preview_vs_apply.md) | |
| [Registered recipe examples](08_registered_recipe_examples.md) | |
| [Inline recipe (`inlineRecipe`)](09_inline_recipe_inlinerecipe.md) | |
| [Troubleshooting](10_troubleshooting.md) | |
| [Related docs](11_related_docs.md) | |

