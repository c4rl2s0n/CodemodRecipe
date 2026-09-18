---
name: codemod-mcp
description: Use when applying deterministic codemod edits via the Rust codemod-mcp MCP server — preview/apply Luau recipes and inlineRecipe. Read the playbook before calling tools.
---

# Codemod MCP

## When to use

- Applying **structural Dart edits** (insert, remove, replace) with AST-safe targeting
- Running registered `.codemod/recipes/*.luau` recipes headlessly from an agent
- Building **inline recipes** (Luau source strings) without adding files on disk
- Previewing changes before apply (required `previewToken` flow)

## When not to use

- General codebase exploration → use **codebase-memory** first
- Non-Dart files or text-only search/replace → edit files directly, or set `language` on ops
- Full-file diffs → MCP exposes preview snippets only, not the host `diff` command

## Instructions

1. **Read** [docs/usage/06_mcp_and_clients/01_codemod_mcp/00_index.md](../../../docs/usage/06_mcp_and_clients/01_codemod_mcp/00_index.md) and Builder apply skill (`.agents/skills/codemod-recipe/builder/apply/`).
2. Use Rust MCP server **`codemod-mcp`**:
   - `cargo run -q --manifest-path rust/Cargo.toml -p codemod_recipe_host --bin codemod_mcp -- --workspace-root . --codemod-root .codemod`
3. Pair with **codebase-memory** for locate → impact → edit workflows.
4. Always **preview before apply**; pass `previewToken` from preview to `apply_recipe`.
5. For inline edits, pass `inlineRecipe` as a Luau source string that `return`s a table with `apply`.

## Quick tool map

| MCP tool | Purpose |
|----------|---------|
| `bootstrap_project` | Install skills/rules and `.codemod/` scaffolding (`edit_policy`, optional `companions`) |
| `list_recipes` | Discover registered recipe ids |
| `describe_recipe` | Args + metadata for one recipe |
| `validate_recipes` | Reload + validate Luau under `.codemod/` |
| `preview_recipe` | Dry-run; returns `previewToken` |
| `apply_recipe` | Write changes; requires `previewToken` |
| `component_scan` / `list_component_instances` | Scan stack roots |
| `component_incremental_scan` | Rescan after edits |
| `describe_component` | Component hooks / args |
| `component_dry_run_hook` | Dry-run one hook; `result` + `transcript` (`c:log`) |
| `component_apply_hook` | Apply one hook (flush); withhold from authoring-only agents |

Human-oriented setup and examples: [docs/usage/06_mcp_and_clients/01_codemod_mcp/00_index.md](../../../docs/usage/06_mcp_and_clients/01_codemod_mcp/00_index.md).
Full playbook: Builder apply skill and `.agents/skills/codemod-recipe/codemod-mcp-playbook/`.
