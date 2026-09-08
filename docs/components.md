# Components (platform layer)

Components are Luau modules under `.codemod/components/` that **identify**, **parse**, **verify**, **scaffold**, and **mutate** abstract instances of code structures in your workspace. They complement recipes: recipes and component `scaffold` create or edit unbound; `operations` mutate an existing scanned target. Both use the same host sandbox `c` (no `c.args` — args are an explicit parameter).

See [component-contract.md](./component-contract.md) for the parse-tree contract, apply signatures, and [luau-lsp.md](./luau-lsp.md) for typed authoring.

## Module shape

```lua
return {
  id = "example.entity",
  description = "optional",
  identify = function(c) ... end,  -- required for stack roots
  parse = function(c, locator) ... end,    -- required
  verify = function(c, instance) ... end,  -- required
  readSource = function(c, context) ... end, -- optional
  toSummary = function(c, instance) ... end, -- optional; host calls after verify

  -- CREATE: no target yet
  scaffold = {
    args = { { name = "name", required = true } },
    apply = function(c, args)
      -- create files from args; may call c:component("child")...
    end,
  },

  -- MUTATE: bound to a scanned target
  operations = {
    addField = {
      args = { { name = "name", required = true } },
      apply = function(c, args, target) ... end,
    },
  },
}
```

Classification: a module with top-level `id` plus `parse` and `verify` loads as `AssetKind::Component`. Helpers without `id` are libraries (`require`-able); see [luau-lsp.md](./luau-lsp.md#runtime-require-library-helpers).

## Hooks

| Hook | When | Returns |
|------|------|---------|
| `identify(c)` | Root scan (required for stack roots) | Freeform list of locators |
| `parse(c, locator)` | Per locator (or child context) | Node table; `nil` to skip; or `{ instances = { ... } }` |
| `verify(c, instance)` | After parse | Array of findings `{ severity, code, message, instanceId?, locator? }` |
| `scaffold.apply(c, args)` | `componentScaffold` | Side effects via `c:insert` / `c:create` / … |
| `operations.*.apply(c, args, target)` | `componentInvokeOperation` | Side effects via `c:insert` / `c:replace` / … |

**`c`** is the host sandbox only (same as recipes) — **not** argument storage. **`args`** is always an explicit table. **`target`** is only on mutate ops (scanned instance: `id`, `componentId`, `locator`, `node`).

**Child components** are invoked with `c:component("child.id"):parse(context)` inside parent `parse`. They need not be listed in `stack.components` and typically omit `identify`.

## Parse tree

Root `parse` returns a **node** with optional **`children`** and **`source`** (`path`, optional `span`) for tooling. Scan JSON exposes the tree as **`node`** (not `parsed`). Agents read **`summary`** / shallow fields; `source` spans are for extract/edit.

## Instance IDs

`{componentId}/{node.id}` when `parse` returns `id`, otherwise `{componentId}/{filename(path)}`.

## Host APIs

| Method | Purpose |
|--------|---------|
| `c:listFiles(pattern)` | Workspace-relative `*` glob → path strings |
| `c:matchFiles(pattern)` | `{placeholder}` path pattern → `{ path, …captures }` rows |
| `c:filename(path)` | Basename without extension |
| `c:resolve(template, vars?)` | Replace `{key}` in a path template (no FS / no MiniJinja; use `c:template` for content) |
| `c:query` / `c:queryMatches` | Hits `{ span = { from, to }, text, kind }` / per-match capture rows |
| `c:component(id)` | Delegate with `:parse`, `:verify`, `:readSource` (cycle-safe) |

All recipe host APIs (`c:read`, `c:insert`, `c:run`, maps/vars, …) are available in component hooks. Recipe and nested `c:run` use `apply(c, args)`.

## Host commands

| Command | Description |
|---------|-------------|
| `componentScan` | Root scan: identify → parse → verify → optional `toSummary`; populates in-process cache |
| `componentListInstances` | Same as scan |
| `componentIncrementalScan` | Rescan after edits: `dirtyPaths` (tier 1), `detectChanges` (stat + re-identify, tier 2), or `force` (full scan). MCP: `component_incremental_scan` |
| `componentInvokeHook` | Dry-run or apply one hook (`identify` / `parse` / `verify` / `toSummary` / `scaffold` / `operation`); returns `result` + `transcript` (+ `files` when applicable). Host `preview` flag maps to Preview vs Apply |
| `componentScaffold` | Thin wrapper → scaffold via `componentInvokeHook` |
| `componentInvokeOperation` | Thin wrapper → operation via `componentInvokeHook` |
| `describeComponent` | Metadata including `hasIdentify` / `hasToSummary` / `hasScaffold` and operation args |

MCP tools: `component_scan`, `list_component_instances`, `component_incremental_scan`, `describe_component`, `component_dry_run_hook`, `component_apply_hook` (dry-run vs apply are separate tools for agent allowlisting; both call `componentInvokeHook`). See [flutter-poc.md](./flutter-poc.md) and [codemod-mcp.md](./codemod-mcp.md).

## Layout

```
.codemod/
  stack.luau              # root-scan component ids
  components/
    example.entity.luau
    example.property.luau # child-only
  lib/                    # optional shared types/helpers for authoring
```

See [stacks.md](./stacks.md) for the stack manifest.

## Reference fixtures

- `test/fixtures/reference_stack/` — identify + matchFiles + operations
- `test/fixtures/flutter_poc/` — tree-sitter, node tree, nested state child
