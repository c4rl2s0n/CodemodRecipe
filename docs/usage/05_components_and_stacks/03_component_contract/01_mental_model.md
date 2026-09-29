# Mental model

A **component instance** is a **node** in a tree — and the scan turns the forest of
trees into a **graph**:

- Root scan: each stack root’s **`identify(c)`** returns freeform **locators**; host calls **`parse(c, locator)`** per entry.
- Each node may nest **children** by calling `c:component(childId):parse(context)` during `parse`.
- After all roots are parsed, **`relate(c, instance, index)`** derives **edges** between instances (see [relations](../01_components/10_relations.md)); **`verify(c, instance, graph)`** then judges with the full forest plus resolved edges.
- Instance JSON: `{ id, componentId, locator, ast }` (plus optional `summary`) — `ast` is the parse-tree payload, so "node" only ever means a tree/graph vertex. Post-scan phases flatten the forest — every node with a `componentId` becomes a flat instance `{ id, componentId, locator, ast, parentId, rootId }`.
- **`source`** (`path` + optional `span`) anchors nodes for extract/edit — internal tooling, not agent summaries.

```lua
{
  id = "recipe",
  name = "Recipe",
  componentId = "flutter.model",
  source = { path = "lib/models/recipe.dart" },
  children = {
    { componentId = "flutter.property", name = "title", source = { ... }, ... },
  },
  -- attached post-relate: outgoing edges grouped by kind
  relations = { storedBy = { { to = "flutter.state/recipe", sources = { { ... } } } } },
}
```

Forest vs graph: **the forest is what `parse` built** (roots + nested children); **the
graph is the forest plus edges** (`relate` output). `index` = the forest before edges,
`graph` = the forest plus resolved edges. Both always cover the full instance set.
