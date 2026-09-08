# Luau LSP ergonomics

Typed authoring for recipes and components under `.codemod/`.

## Setup

1. Install [Luau Language Server](https://marketplace.visualstudio.com/items?itemName=JohnnyMorganz.luau-lsp) (`JohnnyMorganz.luau-lsp`).
2. Run **Codemod Recipe: Scaffold Project** (or bootstrap) so `.vscode/settings.json` includes:

```json
{
  "luau-lsp.types.definitionFiles": [
    ".codemod/stubs/codemod-context.d.luau"
  ],
  "luau-lsp.sourcemap.enabled": false
}
```

3. Add `--!strict` at the top of modules you want typechecked.

After host API changes, run **Update Project Scaffold** or `scripts/generate-host-api-surface.sh` to refresh stubs.

## Defining host API types (source of truth)

All Luau-facing host types live in [`rust/crates/lua/src/host_api/specs.rs`](../rust/crates/lua/src/host_api/specs.rs). Do **not** hardcode type shapes in the stub generator.

| Kind | How |
|------|-----|
| **Input specs** (`InsertSpec`, `QueryCaptureSpec`, …) | `JsonSchema` + `LuaSpec` (or manual `HostSpec`) + `#[host_method(spec = …)]` |
| **Return shapes** (`QueryHit`, `ByteSpan`, `MatchFileRow`, …) | `JsonSchema` + `HostType` + `#[host_method(return_type = …)]` / `Array(…)` |
| **Stubs / completions** | Generated from `HOST_API_SURFACE` by `scripts/generate-host-api-surface.sh` |

Optional fields in stubs use Luau syntax `field: T?` (not TypeScript `field?: T`).

### `c:query` vs helpers

- **`QueryCaptureSpec`** — `capture` required → `c:query`
- **`QueryTableSpec`** — `capture` optional → `queryExists` / `queryCount` / `queryMatches` / `expectMatch` / `expectNoMatch`

Query hits are `{ span = { from, to }, text, kind }` (`ByteSpan` uses `from`/`to` because Lua reserves `end`).

### `c:resolve` vs `c:template`

| | `c:resolve` | `c:template` |
|--|-------------|--------------|
| Job | Path string fill-in | Content rendering |
| Placeholders | `{key}` | MiniJinja `{{ key }}` |
| Side effects | None | File load / recipe args, maps, vars |

Prefer `resolve` for workspace-relative paths; use `template` for generated file bodies.

## Global types (definition files)

`.codemod/stubs/codemod-context.d.luau` exposes ambient types such as `CodemodContext`, `QueryCaptureSpec`, `QueryTableSpec`, `InsertSpec`, `QueryHit`, …

**Recipes:**

```lua
--!strict

return {
  id = "my.recipe",
  apply = function(c: CodemodContext)
    c:query({ path = "lib/foo.dart", query = "...", capture = "x" })
  end,
}
```

**Components** — same `c` type on every hook:

```lua
--!strict

return {
  id = "my.component",
  parse = function(c: CodemodContext, context: MyParseContext): MyNode?
    -- ...
  end,
  verify = function(c: CodemodContext, instance: ComponentInstance): { Finding }
    return {}
  end,
}
```

## Local and shared types

| Approach | When |
|----------|------|
| `export type` in the **same file** | Quick; annotate `parse` args/return |
| `export type` + **`require()`** from a helper under `.codemod/` | Shared parse-context / node types across components |

luau-lsp resolves `require` by filesystem path (sourcemap is off in the default scaffold). It does **not** automatically load every `export type` under `.codemod/` into every file — only **definitionFiles** and modules you **require**.

### Runtime `require` (library helpers)

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

## Runtime vs IDE

Luau types are **IDE-only**. The host passes plain tables at runtime. Components validate parse context with ordinary `if not context.source then return nil end` checks. Runtime `require` loads real library modules; it does not execute `.d.luau` stubs.

## Codemod extension IntelliSense

The VS Code extension also completes `c:` methods, recipe/map/var ids, and query captures under `.codemod/**/*.luau` — complementary to luau-lsp.

See [getting-started.md](./getting-started.md) and [component-contract.md](./component-contract.md).
