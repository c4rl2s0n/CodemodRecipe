# Hooks

Scan phase order: **`identify` (roots) → `parse` (all roots) → `relate` (all instances) → `verify` (all instances)**. `relate` and `verify` walk the whole instance tree, including nested children with a `componentId`.

| Hook | When | Returns |
|------|------|---------|
| `identify(c)` | Root scan (required for stack roots) | Freeform list of locators |
| `parse(c, locator)` | Per locator (or child context) | Node table; `nil` to skip; or `{ instances = { ... } }` |
| `relate(c, instance, index)` | After all roots parsed (optional) | Edge table, array of edges, or `nil` — see [relations](10_relations.md) |
| `verify(c, instance, graph)` | After `relate` (optional) | Array of findings `{ severity, code, message, instanceId?, locator? }` |
| `scaffold.apply(c, args, graph)` | `componentScaffold` | Side effects via `c:insert` / `c:create` / … |
| `operations.*.apply(c, args, target, graph)` | `componentInvokeOperation` | Side effects via `c:insert` / `c:replace` / … |

**`c`** is the host sandbox only (same as recipes) — **not** argument storage. **`args`** is always an explicit table. **`target`** is only on mutate ops (scanned instance: `id`, `componentId`, `locator`, `ast`).

**`index`** (relate phase) is the instance forest *before* edges: `index:byId(id)`, `index:byComponent(componentId)`, `index:nodes()`. **`graph`** (verify phase, operations, scaffold) is the forest *plus* resolved edges — all index methods plus `graph:outgoing(id, kind?)`, `graph:incoming(id, kind?)`, `graph:resolve(edge)`, `graph:resolveFrom(edge)`, `graph:reachable(fromId, kind?)`, `graph:edges()`. Both always cover the **full** forest, never a reachable subset. Graph nodes also carry node-centric traversal: `node:outgoing(kind?)`, `node:incoming(kind?)`, `node:follow(edge)`, `node:neighbors(kind?)` — see [relations](10_relations.md).

The `graph` argument is a **pre-operation snapshot**: it judges the request as of the
last scan and does not observe edits made mid-`apply`; the post-operation rescan
yields the fresh graph.

**Child components** are invoked with `c:component("child.id"):parse(context)` inside parent `parse`. They need not be listed in `stack.components` and typically omit `identify`.
