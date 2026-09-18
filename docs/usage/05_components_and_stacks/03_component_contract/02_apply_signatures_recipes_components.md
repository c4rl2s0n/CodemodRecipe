# Apply signatures (recipes + components)

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
