# Module shape

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

Classification: a module with top-level `id` plus `parse` and `verify` loads as `AssetKind::Component`. Helpers without `id` are libraries (`require`-able); see [luau-lsp.md](../04_luau_lsp/04_local_and_shared_types.md).
