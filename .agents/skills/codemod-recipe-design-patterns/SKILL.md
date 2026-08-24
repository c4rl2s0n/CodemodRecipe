---
name: codemod-recipe-design-patterns
description: Organize codemod-recipe Luau recipes by feature — create vs modify taxonomy, scaffold orchestrators, directory layout, and idempotency. Read reference.md for the full guide.
---

# Recipe Design Patterns

## When to use

- Organizing recipes around concepts (Bloc, repository, routing)
- Choosing between `create_*`, `add_*`, `scaffold_*`, or `remove_*` recipes
- Designing multi-file feature scaffolds or incremental modify workflows

## When not to use

- Host API / module shape → `codemod-luau-recipes`
- Tree-sitter query authoring → `codemod-recipe-authoring`
- Running preview/apply → `codemod-mcp-playbook`

## Prefix summary

| Prefix | Purpose |
|--------|---------|
| `create_` | Greenfield files from templates |
| `add_` / `patch_` | Brownfield AST edits |
| `scaffold_` | Feature workflow (`c:run` atomics) |
| `remove_` | Tear down members or files |

## Instructions

1. **Read and follow** [reference.md](reference.md).
2. Keep recipes atomic; compose scaffolds with `c:run`.
3. For incremental changes, call `add_*` recipes — do not re-run scaffolds.

## Related skills

| Skill | Use for |
|-------|---------|
| `codemod-luau-recipes` | host API, templates |
| `codemod-recipe-authoring` | queries and captures |
| `codemod-languages` | multi-language scaffolds |
| `codemod-mcp-playbook` | preview/apply |
