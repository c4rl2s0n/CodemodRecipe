# Module shape

```lua
return {
  id = "example.entity",
  description = "optional",
  identify = function(c) ... end,  -- required for stack roots
  parse = function(c, locator) ... end,    -- required
  relate = function(c, instance, index) ... end, -- optional; post-scan edge derivation
  verify = function(c, instance, graph) ... end, -- optional; post-scan judgment
  readSource = function(c, context) ... end, -- optional
  toSummary = function(c, instance) ... end, -- optional; host calls after parse

  -- CREATE: no target yet (`graph` = pre-op forest snapshot; often unused)
  scaffold = {
    args = { { name = "name", required = true } },
    apply = function(c, args, graph)
      -- create files from args; may call c:component("child")...
    end,
  },

  -- MUTATE: bound to a scanned target
  operations = {
    addField = {
      args = { { name = "name", required = true } },
      apply = function(c, args, target, graph) ... end, -- graph optional 4th arg
    },
  },
}
```

Classification: a module with top-level `id` plus `parse` loads as `AssetKind::Component`. Helpers without `id` are libraries (`require`-able); see [luau-lsp.md](../04_luau_lsp/04_local_and_shared_types.md).

## Hook requirements

| Hook | Requirement |
|------|-------------|
| `id` + `parse` | **Always required** — this is what classifies the module as a component |
| `identify` | Required **iff the component is a stack root** (listed in `stack.components`). Locator provenance: roots get locators from `identify`; children get their parse context from the parent's `parse` via `c:component(childId):parse(context)` |
| `relate` | Optional; runs post-scan to derive relation edges (see [relations](10_relations.md)) |
| `verify` | Optional; omission ≡ `return {}` (no judgment) |
| `readSource`, `toSummary`, `scaffold`, `operations` | Optional |
