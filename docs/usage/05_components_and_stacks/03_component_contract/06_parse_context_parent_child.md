# Parse context (parent → child)

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
