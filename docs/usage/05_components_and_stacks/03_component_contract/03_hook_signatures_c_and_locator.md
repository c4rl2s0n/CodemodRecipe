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
| `verify(c, instance)` | required | Findings array (`instance.node` is the parse tree) |
| `toSummary(c, instance)` | optional | Agent-facing summary (no `source` spans) |
| `scaffold.apply(c, args)` | optional | Create side effects |
| `operations.*.apply(c, args, target)` | optional | Mutate side effects |

**Required hooks:** `parse`, `verify`. Stack roots must also define **`identify`**. Child-only components typically omit `identify`.

Dry-run any hook via host `componentInvokeHook` / MCP `component_dry_run_hook` (Preview WorkingTree): returns the hook **`result`** JSON plus a recipe-style **`transcript`** (including `c:log`). Apply uses `component_apply_hook` / `preview: false`.

There is **no** `discover` hook and **no** `stack.globs`.

Compose other components with `c:component(id)` — do not pass a components registry into `apply`.
