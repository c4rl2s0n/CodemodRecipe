# Local and shared types

| Approach | When |
|----------|------|
| `export type` in the **same file** | Quick; annotate `parse` args/return |
| `export type` + **`require()`** from a helper under `.codemod/` | Shared parse-context / node types across components |

luau-lsp resolves `require` by filesystem path (sourcemap is off in the default scaffold). It does **not** automatically load every `export type` under `.codemod/` into every file — only **definitionFiles** and modules you **require**.

## Runtime `require` (library helpers)

Top-level **`id`** marks **asset intent** (recipe, component, map, …). Modules **without** `id` (and without `apply`/`parse`) are **libraries**: not catalogued, loadable with Luau `require` from other `.codemod/` files.

| Returned table | Catalog / runtime |
|----------------|-------------------|
| Has `id` + valid asset shape | Catalogued asset |
| Has `id` but not a valid asset shape | Load diagnostic (`unknown asset shape`) |
| No `id`, but has `apply` or `parse` | Load diagnostic (`id` required) |
| No `id`, no `apply`/`parse` | Library — `require`-able |

Rules:

- Resolve relative to the requiring file; stay under `.codemod/`.
- Target must be a **library** (not a recipe/component/map/…). Use `c:run` / `c:component` / `c:map` for assets.
- **`.d.luau` / `.d.lua`** stay IDE-only (skipped by the catalog; cannot `require` them at runtime). Put helpers in ordinary `.luau` files **without** a top-level `id`.

```lua
-- .codemod/str.luau  (library — no id)
return {
  twice = function(s) return s .. s end,
}

-- .codemod/recipes/use.luau
local str = require("../str")
return {
  id = "use",
  apply = function(c, args)
    c:insert({ path = args.file, query = "(program) @root", capture = "root", anchor = "start", text = str.twice("--\n") })
  end,
}
```

Example shared types for the editor (often co-located with a runtime helper, or types-only in a `.luau` you `require` for LSP):

```lua
-- .codemod/lib/platform-types.luau
--!strict
export type Source = { path: string, span: { from: number, to: number }? }
export type ParseContext = { source: Source, parent: ParentIdentity?, role: string?, hints: { [string]: any }? }
```
