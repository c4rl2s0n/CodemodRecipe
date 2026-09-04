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

## Global types (definition files)

`.codemod/stubs/codemod-context.d.luau` exposes ambient types such as `CodemodContext`, `QueryTableSpec`, `InsertSpec`, …

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
| `export type` + **`require()`** from `.codemod/lib/*.luau` | Shared parse-context / node types across components |

luau-lsp resolves `require` by filesystem path (sourcemap is off in the default scaffold). It does **not** automatically load every `export type` under `.codemod/` into every file — only **definitionFiles** and modules you **require**.

Example shared lib (author-maintained):

```lua
-- .codemod/lib/platform-types.luau
--!strict
export type Source = { path: string, span: { start: number, ["end"]: number }? }
export type ParseContext = { source: Source, parent: ParentIdentity?, role: string?, hints: { [string]: any }? }
```

**Note:** Component modules loaded by the host are evaluated as isolated chunks today; `require` inside component files is not yet wired for runtime. Use `require` for **editor types** in libs you also use from recipes, or keep types in the same file as the component.

## Runtime vs IDE

Luau types are **IDE-only**. The host passes plain tables at runtime. Components validate parse context with ordinary `if not context.source then return nil end` checks.

## Codemod extension IntelliSense

The VS Code extension also completes `c:` methods, recipe/map/var ids, and query captures under `.codemod/**/*.luau` — complementary to luau-lsp.

See [getting-started.md](./getting-started.md) and [component-contract.md](./component-contract.md).
