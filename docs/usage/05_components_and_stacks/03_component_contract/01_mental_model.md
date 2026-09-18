# Mental model

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
