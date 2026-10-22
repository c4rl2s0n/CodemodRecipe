# Luau Recipe Reference

Recipes are **sandboxed Luau** modules (`.luau` or `.lua`) anywhere under `.codemod/`.
They `return` a table; **`apply = function(c, args)`** is the only mutation entrypoint and the
registry uses it to identify a recipe. The `.codemod/recipes/` directory is a convention, not a
requirement.

| Page | |
|------|---|
| [Module return table](01_module_return_table.md) | |
| [Arguments (`args`)](02_arguments_args.md) | |
| [Host context (`c`)](03_host_context_c.md) | |
| [Preview vs apply semantics](04_preview_vs_apply_semantics.md) | |
| [Tree-sitter queries](05_tree_sitter_queries.md) | |
| [Path resolution](06_path_resolution.md) | |
| [Inline recipes (MCP / host)](07_inline_recipes_mcp_host.md) | |
| [Validation](08_validation.md) | |
| [Control flow](09_control_flow.md) | |
| [Related docs](10_related_docs.md) | |

