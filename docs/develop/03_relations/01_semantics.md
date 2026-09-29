# Relations semantics

## Scan phase order

```
identify (roots) → parse (all roots) → relate (all instances) → verify (all instances)
```

- `parse` stays pure: it extracts local facts (including references-by-name) and never
  sees other instances.
- `relate(c, instance, index)` turns references-by-name into edges. `index` = the forest
  **before** edges.
- `verify(c, instance, graph)` judges. `graph` = the forest **plus** resolved edges.

Both post-scan phases walk the **whole instance tree** (roots + nested children with a
`componentId`).

## Flattening and identity

The host flattens the instance forest before the relate phase. Every node with a
`componentId` becomes a flat instance:

```
{ id, componentId, node, locator, parentId, rootId }
```

`id` reuses the existing scheme: `{componentId}/{node.id}` when the node has an `id`,
else `{componentId}/{filename(source.path)}`. Child locators are synthesized from the
child's `source` (falling back to the parent locator). The same flattening backs edge
endpoints, the facades, and `node.relations` attachment, so identities always agree.

## Edge model

An edge as returned from `relate`:

```lua
{
  kind = "stores",                                   -- required, open vocabulary
  to = { componentId = "flutter.model", key = "recipe" },  -- see target forms
  source = { path = "lib/stores/recipe_store.dart" },      -- provenance (span optional)
  cardinality = "one",                               -- any extra fields pass through
}
```

Normalized host-side:

```json
{ "from": "flutter.state/recipe", "kind": "stores", "to": "flutter.model/recipe",
  "sources": [ { "path": "..." } ], "cardinality": "one" }
```

- `from` defaults to the relating instance; a relate hook may set it explicitly to emit
  third-party edges (e.g. a DI container emitting `depends_on` between two other
  instances). Unknown `from` → `relation.unresolved` finding.
- Target forms:
  - `"componentId/key"` string — exact flat instance id.
  - `{ componentId = "...", key = "..." }` — ref; host resolves to the flat id
    `componentId/key` or to an instance of that component whose `node.id` equals `key`.
    Unresolved → `relation.unresolved` finding (drift signal), edge dropped.
  - `{ symbol = "dart:core/StatelessWidget" }` — external target; serialized as
    `"symbol"`, never resolves, never a finding. Symbol edges appear in `outgoing` but
    not `incoming`.
- Dedupe = **merge into one canonical edge** by `(from, kind, resolved to|symbol)`.
  `kind` is part of the key: edges of *different kinds* between the same instances are
  different edges and never merge. The canonical edge always carries
  `sources = [ { path, span? }, ... ]` — a list even for a single derivation, one
  shape for all consumers. There is **no `count` field**: multiplicity is derived as
  `#sources` (stored derived data can drift; the list cannot). Visualizers render
  "×n" from `#sources`.
  - **Identical provenance** (same path+span — the relate hook emitted the same edge
    twice, a component bug) → `relation.duplicate` **info** finding, so the developer
    adjusts `relate`.
  - **Distinct provenance** (two code sites genuinely state the same relation, e.g.
    two import statements) → silent merge; multiplicity is real data, not a bug.
  - Resolution findings dedupe by the raw target descriptor, so duplicate dangling
    refs warn once.
- Extra fields (`cardinality`, …) pass through onto the edge JSON (first-wins on
  conflict within a duplicate group).

## Facades (embedded Luau chunk)

Plain data in, queryable tables out. The host evaluates an embedded chunk per sandbox
and calls `build_index(flat)` / `build_graph(flat, edges)`.

Terminology: **instances are the graph nodes** — the facade uses `nodes()`/`edges()`
vocabulary. The word "node" had three senses (tree-sitter node, component instance,
graph vertex); to disambiguate, the *parse-tree payload* field is renamed
`node` → **`ast`** (decided 2026-09-28): `instance.ast.name`, never `node.node.name`.
The vertex keeps the name "instance" in JSON (`id`, `componentId`, `locator`, `ast`,
`parentId`, `rootId`); the scan-response key `instances` stays (component-model wire
term, not graph context).

`index` (relate phase):

| Method | Returns |
|--------|---------|
| `index:byId(id)` | instance (graph node) or `nil` |
| `index:byComponent(componentId)` | array of instances |
| `index:nodes()` | all instances (flat array) |

`graph` (verify phase, operations) — all index methods plus:

| Method | Returns |
|--------|---------|
| `graph:outgoing(id, kind?)` | edges whose `from == id` |
| `graph:incoming(id, kind?)` | edges whose `to == id` |
| `graph:resolve(edge)` | target instance (`edge.to`) or `nil` |
| `graph:resolveFrom(edge)` | source instance (`edge.from`) |
| `graph:reachable(fromId, kind?)` | set (`{ [id] = true }`) via BFS over outgoing edges |
| `graph:edges()` | all resolved edges |

Plain data is already reachable via `nodes()` / `edges()` — no separate `data()`
getter: all queries read the same in-sandbox tables, so no query is "more expensive"
than another (cost is at facade construction, one JSON→Luau conversion per phase).

**Node-centric traversal** (decided 2026-09-28): instances returned by `graph:byId` /
`graph:resolve*` carry bound methods via metatable `__index` (invisible to `pairs` and
JSON serialization; `setmetatable` works inside the sandbox — globals are frozen, user
tables are not):

| Instance method | Returns |
|-----------------|---------|
| `inst:outgoing(kind?)` / `inst:incoming(kind?)` | edges for this instance |
| `inst:follow(edge)` | the *other* endpoint of the edge |
| `inst:neighbors(kind?)` | deduped array of adjacent instances |

Fallback if metatables hit a sandbox edge case: explicit `graph:node(id)` wrapper.
The relate-phase `index` facade stays plain (no edges exist yet at relate time).

## Scan response additions

```json
{
  "instances": [ ... ],
  "edges": [ { "from": "...", "kind": "...", "to": "...", "source": { ... } } ],
  "kindGraph": [ { "from": "flutter.state", "kind": "stores", "to": "flutter.model", "count": 2 } ],
  "findings": [ ... ]
}
```

- `node.relations` is attached per node: `{ "<kind>": [ { "to": ..., "source": ... } ] }`
  (outgoing edges grouped by kind), so agent-facing instance trees stay self-contained.
- `kindGraph` aggregates resolved instance→instance edges by
  `(fromComponentId, kind, toComponentId)` with `count`; symbol edges aggregate with
  `external = true` and the symbol as `to`.

## Incremental scan

`relate` + `verify` always re-run over the **merged** forest (cached + reparsed dirty
roots): a dirty file can change edges between otherwise unchanged instances, so
judgment findings are recomputed wholesale. Parse-time findings keep per-instance
incremental merging. Instance-local verify re-runs are the v1 price; partitioning by
locality is a future optimization.

## Dry-run

`component_dry_run_hook` accepts `hook = "relate"` (requires `instance`, like `verify`).
For `relate`/`verify` the host builds the facade from the cached scan; when no cache
exists it runs a full scan first (populating the cache).

## Worked example (flutter_poc refactor)

Before: `flutter.model.parse` nests the store as a child (`role = "store"`), deriving
the link ad hoc from `context.parent` / filename convention; `verify` cross-checks
fields through the nested child.

After: `flutter.model` and `flutter.state` are both stack roots.

```lua
-- flutter.state
relate = function(c, instance, index)
  return {
    kind = "stores",
    to = { componentId = "flutter.model", key = instance.ast.modelId },
    source = instance.ast.source,
  }
end

-- flutter.model
verify = function(c, instance, graph)
  local findings = {}
  local stores = graph:incoming(instance.id, "stores")
  if #stores == 0 then
    findings[#findings + 1] = { severity = "warning", code = "model.missing_store",
      message = "model has no store", instanceId = instance.id }
    return findings
  end
  local store = graph:resolveFrom(stores[1]) -- incoming edge: from = the store
  -- cross-check store.ast.fields against own property children …
  return findings
end
```

The `addStoreField` operation receives `graph` as 4th arg and resolves the store via
`graph:incoming(target.id, "stores")` instead of `findChild`.

## Operations, guards, and scaffold

- `operations.*.apply(c, args, target, graph)` — implemented. The graph is the
  **pre-operation** graph (from the cached scan): it judges the *request*, it does not
  observe mid-apply edits. Use cases: guards rejected at dry-run (delete a model that
  still has incoming `stores` edges, layering rules, cycle checks via `reachable`),
  convention-free navigation, fan-out edits across linked instances.
- `scaffold.apply(c, args, graph)` — graph as 3rd arg for symmetry (decided
  2026-09-28): scaffolding often wires a new instance into the existing graph.

## Deferred

- Snapshot persistence / drift detection over time (Phase 3, with the semantic delta
  spike — format decided 2026-09-28, see
  [02_semantic_delta_spike.md](02_semantic_delta_spike.md)).
- Declarative rule sugar (only if many same-shape rules emerge).
- VS Code graph view (`kindGraph` rides the scan response; a `component_graph` MCP
  tool emitting mermaid is the interim PoC).
