---
name: codemod-mcp
description: Apply deterministic Dart AST edits via codemod-mcp MCP tools. Read the canonical playbook before preview/apply.
user-invocable: true
---

# Codemod MCP

## When to use

- Preview/apply codemod recipes (registered YAML or inline)
- `generate_ast_path` from file + byte offset
- Structural insert / remove / replace on Dart AST paths

## Instructions

1. **Read and follow** `.cursor/skills/codemod-mcp/reference.md` (canonical playbook).
2. Use MCP server **`codemod-mcp`** (`dart run bin/codemod_mcp.dart`).
3. Pair with **codebase-memory** for locate → impact → edit (playbook § Agent workflow).
4. Always preview before apply; pass `previewToken` to `apply_recipe`.

Human setup: `docs/codemod-mcp.md`
