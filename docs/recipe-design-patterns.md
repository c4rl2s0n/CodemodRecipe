# Recipe Design Patterns (Create vs Modify)

How to organize codemod-recipe **Luau recipes** around **concepts or features**, with
a clear split between **creation** (greenfield scaffolding) and **modification**
(brownfield AST edits).

For the authoring workflow (queries, capture, anchor, step types), see
[writing-recipes.md](writing-recipes.md).

## Taxonomy

| Prefix | Purpose |
|--------|---------|
| `create_` | Greenfield files from templates |
| `add_` / `patch_` | Brownfield AST edits (one change each) |
| `scaffold_` | Feature workflow (compose atomics) |
| `remove_` | Tear down members or files |

**One recipe = one coherent change.** Compose upward with `scaffold_*`
orchestrators that `c:run` atomics. For incremental changes after scaffolding,
call atomic `add_*` recipes.

Prefer idempotent brownfield recipes: a second apply should produce no patches
([writing-recipes.md](writing-recipes.md#idempotency)).

## Layout tips

- Group recipes by feature under `.codemod/recipes/*.luau` (directory names are
  convention; discovery scans Luau modules).
- Use dotted `id`s (`feature.area.recipe_name`) so the VS Code Recipes tab nests
  them.
- Keep AST edits small; put multi-file workflows in a `scaffold_*` that
  calls atomics with `c:run("atomic.id")`.

## Related docs

| Doc | Contents |
|-----|----------|
| [writing-recipes.md](writing-recipes.md) | How to write recipes |
| [recipe-templates.md](recipe-templates.md) | Jinja / maps / variables |
| [codemod-mcp.md](codemod-mcp.md) | MCP preview/apply |
| [new-project-rust-mcp.md](new-project-rust-mcp.md) | Bootstrap a new project |

## For agents

After `bootstrap_project`, the full playbook also lives at:

```text
.agents/skills/codemod-recipe-design-patterns/
  SKILL.md       # router — load by default
  reference.md   # full playbook — read when designing recipes
```

Related modular skills: `codemod-luau-recipes`, `codemod-recipe-authoring`,
`codemod-languages`, `codemod-mcp-playbook`.
