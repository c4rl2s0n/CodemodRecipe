---
name: codemod-luau-recipes
description: Luau recipe authoring for codemod-recipe — recipe module shape, host API (c:insert/replace/remove/query/run), maps/variables/query libs, templates, and validation. Read reference.md for full API and examples.
---

# Luau Recipes

## When to use

- Writing or reviewing Luau recipe modules anywhere under `.codemod/` (`.codemod/recipes/` is a convention)
- Crafting `inlineRecipe` Luau source strings for MCP preview/apply
- Understanding the host context API (`c:insert`, `c:run`, `c:applyTemplate`, …)

## When not to use

- Recipe organization (create vs modify) → `codemod-recipe-design-patterns`
- Tree-sitter query syntax → `codemod-tree-sitter-queries`
- Dart-specific query patterns → `codemod-recipe-authoring`
- Language IDs, SQL dialects → `codemod-languages`
- Running MCP tools → `codemod-mcp-playbook`

## Recipe shape

Luau modules **return a table**. `apply = function(c)` is the only mutation entrypoint.

```lua
return {
  id = "feature.area.recipe_id",
  name = "Human-readable name",       -- optional
  description = "What it does",       -- optional
  args = {
    { name = "file", required = true, inputKind = "file" },
  },
  maps = { myMap = { key = "value" } },  -- optional recipe-local maps
  queries = { body = "(program) @root" }, -- optional inline query ids
  postExecution = { "dart format ." },    -- optional; runs after apply only
  explorerMenu = { { kind = "file" } },   -- optional VS Code Explorer menu
  apply = function(c)
    c:insert({
      path = c.args.file,
      query = "(class_definition ...) @body",
      capture = "body",
      anchor = "end",
      text = "    print('codemod');\n",
    })
  end,
}
```

## Host API (summary)

| Method | Role |
|--------|------|
| `c.args` | Recipe arguments (strings, bools, lists, objects) |
| `c:insert` / `c:replace` / `c:remove` | AST edits (table: `path`, `query`, `capture`, …) |
| `c:query` / `c:queryExists` / `c:queryCount` | Read-only query evaluation |
| `c:read` | Current file content (WorkingTree-aware) |
| `c:create` / `c:delete` | File create/delete |
| `c:run(id, args?)` | Invoke another registered recipe |
| `c:postExec({ command = "..." })` | Shell command (skipped on preview) |
| `c:applyTemplate` / `c:renderFileTemplate` | MiniJinja rendering |
| `c:map(id)` / `c:var(id)` / `c:queryLib(id)` | Lookup registered assets |

Set `language = "dart"` (etc.) on edit ops when the path is ambiguous.

## Preview vs apply

- Both run `apply()`. Preview keeps edits in a **WorkingTree** and records `postExec` in the **transcript** without running it.
- Apply **re-runs** `apply()` and flushes to disk before each `postExec` / `postExecution` entry.
- `previewToken` binds recipe + args + starting file hashes. When apply may diverge from preview (skipped postExec), the VS Code review panel shows an `applyMayDiverge` disclaimer.

## Instructions

1. **Read and follow** [reference.md](reference.md) for the full host API, asset modules, and examples.
2. Call `validate_recipes` after editing recipe Luau.
3. Path resolution: update `rust/crates/core/src/resource_path.rs` — do not add ad hoc joins in host/engine.
4. Recipe loading/validation: `rust/crates/lua/src/load.rs` and `rust/crates/host/src/lua_runtime.rs`.

## Related skills

| Skill | Use for |
|-------|---------|
| `codemod-recipe-design-patterns` | create vs modify taxonomy |
| `codemod-languages` | `language` on ops |
| `codemod-tree-sitter-queries` | query syntax, captures, predicates |
| `codemod-recipe-authoring` | Dart query patterns and testing |
| `codemod-mcp-playbook` | preview/apply workflow |

## Human docs

- [docs/writing-recipes.md](../../../../docs/writing-recipes.md) — authoring workflow
- [docs/getting-started.md](../../../../docs/getting-started.md) — day-1 overview
