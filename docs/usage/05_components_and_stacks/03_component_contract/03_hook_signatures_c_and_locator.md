# Hook signatures: `c` and locator

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
| `relate(c, instance, index)` | optional | Edge table, array of edges, or `nil` |
| `verify(c, instance, graph)` | optional | Findings array (`instance.ast` is the parse-tree payload) |
| `toSummary(c, instance)` | optional | Agent-facing summary (no `source` spans) |
| `scaffold.apply(c, args, graph)` | optional | Create side effects |
| `operations.*.apply(c, args, target, graph)` | optional | Mutate side effects |

**Required hooks:** `parse` (plus `id`). Stack roots must also define **`identify`** — locator provenance: roots get locators from `identify`, children get their parse context from the parent's `parse`. Child-only components typically omit `identify`. `verify` is optional; omission ≡ no judgment.

`relate` and `verify` run **post-scan** (after all roots are parsed), walking the whole instance tree including nested children. `index` is the forest before edges; `graph` is the forest plus resolved edges — see [relations](../01_components/10_relations.md). Operations receive the pre-operation `graph` as 4th arg (guards, navigation) and `scaffold.apply` as 3rd arg; existing shorter signatures are unaffected (Luau ignores extra args). The graph judges the **request as of the last scan** — it does not observe mid-`apply` edits; the post-operation rescan yields the fresh graph.

Dry-run any hook via host `componentInvokeHook` / MCP `component_dry_run_hook` (Preview WorkingTree): returns the hook **`result`** JSON plus a recipe-style **`transcript`** (including `c:log`). Apply uses `component_apply_hook` / `preview: false`. For `relate`/`verify` the facade is built from the cached scan; when no cache exists, the dry-run triggers a full scan first.

There is **no** `discover` hook and **no** `stack.globs`.

Compose other components with `c:component(id)` — do not pass a components registry into `apply`.
