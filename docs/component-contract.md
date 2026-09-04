# Component data contract (v1.2)

Parse trees, parse context, apply signatures, and hook conventions for the platform layer.

## Mental model

A **component instance** is a **node** in a tree:

- Root scan produces top-level nodes (listed in `stack.luau` `components`).
- Each node may nest **children** by calling `c:component(childId):parse(context)` during `parse`.
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

| Surface | Signature | Identity |
|---------|-----------|----------|
| Recipe `apply` | `apply(c, args)` | absent |
| Component `scaffold.apply` | `apply(c, args)` | absent (create) |
| Component `operations.*.apply` | `apply(c, args, identity)` | scanned instance |

```lua
-- Unbound (recipe / scaffold)
apply = function(c, args)
  local name = args.name
  c:insert({ ... })
end

-- Bound mutate
operations.addStoreField.apply = function(c, args, identity)
  local path = identity.parsed.source and identity.parsed.source.path
  local fieldName = args.fieldName
end
```

`identity` (mutate only) includes at least `id`, `componentId`, `locator`, and `parsed` (the parse tree).

## Hook signatures: `c` and `context`

Read/verify hooks are **`hook(c, arg?)`** — same host table as recipe `apply` (`CodemodContext` in stubs), without `args` on `c`.

| Param | Role |
|-------|------|
| **`c`** | Host context: `c:read`, `c:query`, `c:glob`, `c:resolve`, `c:component`, … |
| **`context`** (2nd arg to `parse`) | Parse input from host or parent. JSON scan output uses key **`locator`** for API stability. |

```lua
parse = function(c, context)
  local path = context.source and context.source.path or context.path
  if not path then return nil end
  -- ...
end
```

| Hook | Args | Returns |
|------|------|---------|
| `discover(c)` | optional | Candidate contexts (legacy; prefer `stack.globs` + `parse` only) |
| `identify(c, context)` | optional | `true` / `false` — skip before `parse` when present |
| `parse(c, context)` | required | Node table, `nil` / `{ skip = true }` to skip, or `{ instances = { ... } }` |
| `verify(c, instance)` | required | Findings array |
| `toSummary(c, instance)` | optional | Agent-facing summary (no `source` spans) |
| `scaffold.apply(c, args)` | optional | Create side effects |
| `operations.*.apply(c, args, identity)` | optional | Mutate side effects |

**Required hooks:** `parse`, `verify` only. `discover` / `identify` / `scaffold` / `operations` are optional.

Compose other components with `c:component(id)` — do not pass a components registry into `apply`.

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

## Stack globs (host-driven discover)

```lua
return {
  id = "my-stack",
  components = { "flutter.model" },
  globs = {
    ["flutter.model"] = "lib/models/*.dart",
  },
}
```

When `discover` is absent, the host globs each path and calls `parse(c, { source = { path } })`.

## Findings

```lua
{ severity = "warning", code = "model.field_missing_in_store", message = "...", instanceId = "..." }
```

## Reference

`test/fixtures/flutter_poc/` — tree-shaped model with nested property, method, and state children.

See also [components.md](./components.md), [luau-lsp.md](./luau-lsp.md).
