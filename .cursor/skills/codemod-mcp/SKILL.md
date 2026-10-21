---
name: codemod-mcp
description: >-
  Use when applying deterministic codemod edits via the Rust codemod-mcp MCP
  server — preview/apply Luau recipes, components, and stacks. Thin pointer to
  the product playbook (do not maintain a parallel tool encyclopedia here).
---

# Codemod MCP

## When to use

- Applying **structural AST edits** (any supported language) with preview-before-apply
- Running registered `.codemod/recipes/*.luau` or component/stack hooks from an agent
- Building **inline recipes** (`inlineRecipe` as a Luau **source string**)
- Scanning a stack (`component_scan`) or dry-running hooks

## When not to use

- General codebase exploration → use **codebase-memory** first
- Text-only search/replace with no AST targeting → edit files directly
- Interactive full-file review UI → VS Code recipe runner; agents use `preview_recipe` / `diff_recipe` for hunk JSON

## Instructions

1. Read the product playbook: [`.agents/skills/codemod-recipe/codemod-mcp-playbook/SKILL.md`](../../../.agents/skills/codemod-recipe/codemod-mcp-playbook/SKILL.md)
2. Depth: [docs/usage/06_mcp_and_clients/01_codemod_mcp/00_index.md](../../../docs/usage/06_mcp_and_clients/01_codemod_mcp/00_index.md)
3. Apply workflow: [Builder apply](../../../.agents/skills/codemod-recipe/builder/apply/SKILL.md)
4. Always **preview / dry-run before apply**; pass `previewToken` to `apply_recipe`, `component_apply_hook`, and `stack_apply_operation`
5. Run server: `cargo run -q --manifest-path rust/Cargo.toml -p codemod_recipe_host --bin codemod_mcp -- --workspace-root . --codemod-root .codemod`

Do **not** duplicate the tool table here — inventory SSOT is `mcp_inventory.rs`; docs live under usage.
