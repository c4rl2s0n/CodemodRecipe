# Day-1 success: run a recipe

## In VS Code

1. Open **Codemod Recipe** → pick a recipe in the **Recipes** tab.
2. Fill args in **Recipe Runner** (live preview updates as you type).
3. Review **Files** and **Transcript**; uncheck anything you do not want.
4. **Apply Selected**.

Arg defaults use `defaultsTo`. Power features: [recipe-shortcuts.md](../04_templates_and_patterns/03_recipe_shortcuts/00_index.md).

## Over MCP

Typical tool order:

1. `bootstrap_project` (once, if needed)
2. `validate_recipes` (after editing Luau)
3. `list_recipes` / `describe_recipe`
4. `preview_recipe` — save `previewToken`
5. `apply_recipe` with the same recipe/args + token

Re-preview afterward: an idempotent recipe returns no files.
More detail: [codemod-mcp.md](../06_mcp_and_clients/01_codemod_mcp/00_index.md).
