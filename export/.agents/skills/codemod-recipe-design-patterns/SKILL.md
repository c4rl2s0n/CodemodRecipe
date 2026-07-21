---
name: codemod-recipe-design-patterns
description: Organize codemod-recipe YAML recipes by feature — create vs modify taxonomy, scaffold orchestrators, directory layout, and idempotency. Read reference.md for the full guide.
---

# Recipe Design Patterns

## When to use

- Organizing recipes around concepts (Bloc, repository, routing)
- Choosing between `create_*`, `add_*`, `scaffold_*`, or `remove_*` recipes
- Designing multi-file feature scaffolds or incremental modify workflows
- Naming recipes and structuring `.codemod/recipes/` directories

## When not to use

- YAML syntax or template helpers → `codemod-yaml-dsl-v2`
- Tree-sitter query authoring → `codemod-recipe-authoring`
- Running preview/apply → `codemod-mcp-playbook`

## Prefix summary

| Prefix | Purpose |
|--------|---------|
| `create_` | Greenfield files from templates |
| `add_` / `patch_` | Brownfield AST edits (one coherent change each) |
| `scaffold_` | Feature workflow (compose atomics) |
| `remove_` | Tear down members or files |

## Instructions

1. **Read and follow** [reference.md](reference.md) in this directory for the full guide.
2. Keep recipes atomic; compose with `scaffold_*` orchestrators.
3. For incremental changes after scaffolding, call `add_*` recipes — do not re-run scaffolds.

## Related skills

| Skill | Use for |
|-------|---------|
| `codemod-yaml-dsl-v2` | YAML syntax, templates, maps |
| `codemod-recipe-authoring` | Tree-sitter queries and captures |
| `codemod-mcp-playbook` | preview/apply workflow |
| `codemod-overview` | Project layout and orientation |
