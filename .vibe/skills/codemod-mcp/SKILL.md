---
name: codemod-mcp
description: Apply deterministic codemod edits via codemod-mcp MCP tools. Read the canonical playbook before preview/apply.
user-invocable: true
---

# Codemod MCP

## When to use

- Preview/apply codemod recipes (registered YAML or inline)
- Multi-language tree-sitter edits via YAML recipes

## Instructions

1. **Read and follow** `.cursor/skills/codemod-mcp/reference.md` (canonical playbook).
2. Use MCP server **`codemod-mcp`** (Rust binary `codemod_mcp`; see `docs/codemod-mcp.md`).
3. Pair with **codebase-memory** for locate → impact → edit (playbook § Agent workflow).
4. Always preview before apply; pass `previewToken` to `apply_recipe`.

Human setup: `docs/codemod-mcp.md`
