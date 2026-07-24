---
name: codemod-mcp
description: Use when applying deterministic Dart AST edits via the Rust codemod-mcp MCP server — preview/apply recipes and inline YAML v2 edits (insert/remove/replace). Read the playbook before calling tools.
---

# Codemod MCP

## When to use

- Applying **structural Dart edits** (insert, remove, replace) with AST-safe targeting
- Running registered `.codemod/recipes/*.yaml` recipes headlessly from an agent
- Building **inline recipes** without adding files on disk
- Previewing changes before apply (required `previewToken` flow)

## When not to use

- General codebase exploration → use **codebase-memory** first
- Non-Dart files or text-only search/replace → edit files directly
- Full-file diffs → MCP exposes preview snippets only, not the host `diff` command

## Instructions

1. **Read and follow** [reference.md](reference.md) in this directory (canonical tool playbook).
2. Use Rust MCP server **`codemod-mcp`**:
   - `cargo run -q --manifest-path rust/Cargo.toml -p codemod_recipe_host --bin codemod_mcp -- --workspace-root . --codemod-root .codemod`
3. Pair with **codebase-memory** for locate → impact → edit workflows (see playbook § Agent workflow).
4. Always **preview before apply**; pass `previewToken` from preview to `apply_recipe`.
5. For inline edits, use YAML v2 `edit.ops` with tree-sitter `query` + `capture`.

## Quick tool map

| MCP tool | Purpose |
|----------|---------|
| `bootstrap_project` | Install skills/rules and `.codemod/` scaffolding (`edit_policy`, optional `companions`) |
| `list_recipes` | Discover registered recipe ids |
| `describe_recipe` | Args + metadata for one recipe |
| `validate_recipes` | YAML/schema diagnostics |
| `preview_recipe` | Dry-run; returns `previewToken` |
| `apply_recipe` | Write changes; requires `previewToken` |

Human-oriented setup and examples: [docs/codemod-mcp.md](../../../docs/codemod-mcp.md).
