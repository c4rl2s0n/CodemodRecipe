# Apply signatures (recipes + components)

Args are **never** on `c`. Host context is sandbox only.

| Surface | Signature | Target |
|---------|-----------|--------|
| Recipe `apply` | `apply(c, args)` | absent |
| Component `scaffold.apply` | `apply(c, args, graph)` | absent (create) |
| Component `operations.*.apply` | `apply(c, args, target, graph)` | scanned instance |

```lua
-- Unbound (recipe / scaffold)
apply = function(c, args)
  local name = args.name
  c:insert({ ... })
end

-- Bound mutate
operations.addStoreField.apply = function(c, args, target, graph)
  local path = target.ast.source and target.ast.source.path
  local fieldName = args.fieldName
end
```

`target` (mutate only) includes at least `id`, `componentId`, `locator`, and **`ast`** (the parse-tree payload). The `graph` arg on component applies is the pre-operation relations graph — see [relations](../01_components/10_relations.md).
