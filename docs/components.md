# Components (platform layer)

Components are Luau modules under `.codemod/components/` that **parse**, **verify**, **scaffold**, and **mutate** abstract instances of code structures in your workspace. They complement recipes: recipes and component `scaffold` create or edit unbound; `operations` mutate an existing scanned identity. Both use the same host sandbox `c` (no `c.args` — args are an explicit parameter).

See [component-contract.md](./component-contract.md) for the parse-tree contract, apply signatures, and [luau-lsp.md](./luau-lsp.md) for typed authoring.

## Module shape

```lua
return {
  id = "example.entity",
  description = "optional",
  discover = function(c) ... end,   -- optional (legacy; prefer stack.globs)
  identify = function(c, context) ... end, -- optional
  parse = function(c, context) ... end,    -- required
  verify = function(c, instance) ... end,  -- required
  readSource = function(c, context) ... end, -- optional
  toSummary = function(c, instance) ... end, -- optional; host calls after verify

  -- CREATE: no identity yet
  scaffold = {
    args = { { name = "name", required = true } },
    apply = function(c, args)
      -- create files from args; may call c:component("child")...
    end,
  },

  -- MUTATE: bound to a scanned identity
  operations = {
    addField = {
      args = { { name = "name", required = true } },
      apply = function(c, args, identity) ... end,
    },
  },
}
```

Classification: a module with `parse` and `verify` loads as `AssetKind::Component`.

## Hooks

| Hook | When | Returns |
|------|------|---------|
| `discover(c)` | Root scan (optional) | Candidate parse contexts |
| `identify(c, context)` | Per candidate (optional) | Truthy if this component owns the context |
| `parse(c, context)` | Root or child | Node table; `nil` to skip; or `{ instances = { ... } }` |
| `verify(c, instance)` | After parse | Array of findings `{ severity, code, message, instanceId?, locator? }` |
| `scaffold.apply(c, args)` | `componentScaffold` | Side effects via `c:insert` / `c:create` / … |
| `operations.*.apply(c, args, identity)` | `componentInvokeOperation` | Side effects via `c:insert` / `c:replace` / … |

**`c`** is the host sandbox only (same as recipes) — **not** argument storage. **`args`** is always an explicit table. **`identity`** is only on mutate ops (scanned instance: `id`, `componentId`, `locator`, `parsed`).

**Child components** are invoked with `c:component("child.id"):parse(context)` inside parent `parse`. They need not be listed in `stack.components`. Compose scaffolds/ops the same way via `c:component(id)` — there is no “components registry” apply argument.

## Parse tree

Root `parse` returns a node with optional **`children`** (nested child parse results) and **`source`** (`path`, optional `span`) for tooling. Agents read **`summary`** / shallow fields; `source` spans are for extract/edit.

## Instance IDs

`{componentId}/{parse.id}` when `parse` returns `id`, otherwise `{componentId}/{stem(path)}`.

## Host APIs

| Method | Purpose |
|--------|---------|
| `c:glob(pattern)` | Workspace-relative glob |
| `c:stem(path)` | Basename without extension |
| `c:resolve(template, vars?)` | Replace `{key}` in a path template |
| `c:component(id)` | Delegate with `:parse`, `:verify`, `:readSource` (cycle-safe) |

All recipe host APIs (`c:read`, `c:insert`, `c:query`, `c:run`, maps/vars, …) are available in component hooks. Recipe and nested `c:run` use `apply(c, args)` (no `args`).

## Host commands

| Command | Description |
|---------|-------------|
| `componentScan` | Root scan: globs / discover → parse → verify → optional `toSummary`; populates in-process cache |
| `componentListInstances` | Same as scan |
| `componentIncrementalScan` | Rescan after edits: `dirtyPaths` (tier 1), `detectChanges` (stat + glob-diff, tier 2), or `force` (full scan). MCP: `component_incremental_scan` |
| `componentScaffold` | Run component `scaffold.apply(c, args)` (create; no instanceId) |
| `componentInvokeOperation` | Run `operations.*.apply(c, args, identity)`; preview or apply; incremental-rescans dirty paths after apply |

MCP tools: `component_scan`, `list_component_instances`, `component_incremental_scan`. See [flutter-poc.md](./flutter-poc.md) for a try-it path and [codemod-mcp.md](./codemod-mcp.md) for tool setup.

## Layout

```
.codemod/
  stack.luau              # root-scan component ids + optional globs
  components/
    example.entity.luau
    example.property.luau # child-only
  lib/                    # optional shared types/helpers for authoring
```

See [stacks.md](./stacks.md) for the stack manifest.

## Reference fixtures

- `test/fixtures/reference_stack/` — legacy discover/identify + operations
- `test/fixtures/flutter_poc/` — tree-sitter, parse tree, nested state child
