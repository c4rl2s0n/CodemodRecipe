# Component data contract (v1.3)

Parse trees, locators, apply signatures, and hook conventions for the platform layer.

## Mental model

A **component instance** is a **node** in a tree:

- Root scan: each stack root’s **`identify(c)`** returns freeform **locators**; host calls **`parse(c, locator)`** per entry.
- Each node may nest **children** by calling `c:component(childId):parse(context)` during `parse`.
- Instance JSON: `{ id, componentId, locator, node }` (plus optional `summary`).
- **`source`** (`path` + optional `span`) anchors nodes for extract/edit — internal tooling, not agent summaries.

```lua
{
  id = "recipe",
  name = "Recipe",
  componentId = "flutter.model",
  source = { path = "lib/models/recipe.dart" },
  children = {
    { componentId = "flutter.property", name = "title", source = { ... }, ... },
    { componentId = "flutter.state", role = "store", source = { path = "lib/stores/recipe_store.dart" }, ... },
  },
}
```

## Apply signatures (recipes + components)

Args are **never** on `c`. Host context is sandbox only.

| Surface | Signature | Target |
|---------|-----------|--------|
| Recipe `apply` | `apply(c, args)` | absent |
| Component `scaffold.apply` | `apply(c, args)` | absent (create) |
| Component `operations.*.apply` | `apply(c, args, target)` | scanned instance |

```lua
-- Unbound (recipe / scaffold)
apply = function(c, args)
  local name = args.name
  c:insert({ ... })
end

-- Bound mutate
operations.addStoreField.apply = function(c, args, target)
  local path = target.node.source and target.node.source.path
  local fieldName = args.fieldName
end
```

`target` (mutate only) includes at least `id`, `componentId`, `locator`, and **`node`** (the parse tree).

## Hook signatures: `c` and locator

Read/verify hooks are **`hook(c, arg?)`** — same host table as recipe `apply` (`CodemodContext` in stubs), without `args` on `c`.

| Param | Role |
|-------|------|
| **`c`** | Host context: `c:read`, `c:query`, `c:listFiles`, `c:matchFiles`, `c:resolve`, `c:component`, … |
| **`locator`** (2nd arg to `parse`) | Freeform table from `identify` or parent. No host field conventions. |

```lua
identify = function(c)
  local out = {}
  for _, row in ipairs(c:matchFiles("lib/models/{name}.dart")) do
    out[#out + 1] = { path = row.path, name = row.name }
  end
  return out
end

parse = function(c, locator)
  local path = locator.source and locator.source.path or locator.path
  if not path then return nil end
  -- ...
end
```

| Hook | Args | Returns |
|------|------|---------|
| `identify(c)` | **required on stack roots** | List of freeform locators |
| `parse(c, locator)` | required | Node table, `nil` / `{ skip = true }` to skip, or `{ instances = { ... } }` |
| `verify(c, instance)` | required | Findings array (`instance.node` is the parse tree) |
| `toSummary(c, instance)` | optional | Agent-facing summary (no `source` spans) |
| `scaffold.apply(c, args)` | optional | Create side effects |
| `operations.*.apply(c, args, target)` | optional | Mutate side effects |

**Required hooks:** `parse`, `verify`. Stack roots must also define **`identify`**. Child-only components typically omit `identify`.

Dry-run any hook via host `componentInvokeHook` / MCP `component_dry_run_hook` (Preview WorkingTree): returns the hook **`result`** JSON plus a recipe-style **`transcript`** (including `c:log`). Apply uses `component_apply_hook` / `preview: false`.

There is **no** `discover` hook and **no** `stack.globs`.

Compose other components with `c:component(id)` — do not pass a components registry into `apply`.

## Path helpers

| Method | Pattern | Returns |
|--------|---------|---------|
| `c:listFiles("lib/models/*.dart")` | shell `*` globs | list of path strings |
| `c:matchFiles("lib/models/{name}.dart")` | `{placeholder}` (one segment each) | `{ path, name, ... }` rows |
| `c:resolve(tmpl, vars)` | `{key}` fill-in | **one** path string (no FS search) |
| `c:filename(path)` | — | basename without extension |

## Query hits

`c:query` returns hits shaped:

```lua
{ span = { from = 10, to = 20 }, text = "...", kind = "identifier" }
```

(`from` / `to` because Lua reserves `end`.)

`c:queryMatches` returns one row per tree-sitter match; keys are `@capture` names. One capture → a hit; multiple same-name captures → array of hits.

## Parse context (parent → child)

```lua
c:component("flutter.state"):parse({
  source = { path = "lib/stores/recipe_store.dart" },
  parent = {
    componentId = "flutter.model",
    id = "recipe",
    name = "Recipe",
    path = "lib/models/recipe.dart",
  },
  role = "store",
  hints = { },  -- component-specific; not validated by host
})
```

Parent derives paths (`c:resolve`), builds identity, calls child `parse`. Child checks at runtime what it needs (`if not context.hints.name then return nil end`).

## Findings

```lua
{ severity = "warning", code = "model.field_missing_in_store", message = "...", instanceId = "..." }
```

## Reference

`test/fixtures/flutter_poc/` — tree-shaped model with nested property, method, and state children.

See also [components.md](./components.md), [luau-lsp.md](./luau-lsp.md).
