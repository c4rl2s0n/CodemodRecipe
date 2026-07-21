# Recipe Design Patterns (Create vs Modify)

This guide describes how to organize codemod-recipe YAML v2 recipes around **concepts or
features**, with a clear split between **creation** (greenfield scaffolding) and
**modification** (brownfield AST edits).

## For agents

After `bootstrap_project`, the full guide lives at:

```text
.agents/skills/codemod-recipe-design-patterns/
  SKILL.md       # router (~30 lines) — load by default
  reference.md   # full playbook — read when designing recipes
```

Invoke skill **`codemod-recipe-design-patterns`** when organizing recipes, choosing
`create_*` vs `add_*` vs `scaffold_*` naming, or designing feature packs.

Related modular skills (also under `.agents/skills/` after bootstrap):

| Skill | Contents |
|-------|----------|
| `codemod-yaml-dsl-v2` | YAML syntax, templates, maps (`reference.md`) |
| `codemod-recipe-authoring` | Tree-sitter queries |
| `codemod-languages` | Multi-language `language:` field, SQL dialects |
| `codemod-mcp-playbook` | preview/apply workflow |

## Quick summary

| Prefix | Purpose |
|--------|---------|
| `create_` | Greenfield files from templates |
| `add_` / `patch_` | Brownfield AST edits (one change each) |
| `scaffold_` | Feature workflow (compose atomics) |
| `remove_` | Tear down members or files |

**One recipe = one coherent change.** Compose upward with `scaffold_*` orchestrators.
For incremental changes after scaffolding, call atomic `add_*` recipes.

See [codemod-mcp.md](codemod-mcp.md) for MCP setup and [new-project-rust-mcp.md](new-project-rust-mcp.md) for bootstrap.
