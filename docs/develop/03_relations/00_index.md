# Relations (design spike)

First-class relations between component instances: a post-scan `relate` phase derives
edges, a graph-aware `verify` judges them, and the host aggregates a kind-level
architecture graph. This doc is the design record; usage docs live under
`docs/usage/05_components_and_stacks/`.

| Page | |
|------|---|
| [Semantics](01_semantics.md) | |
| [Semantic delta in preview (spike)](02_semantic_delta_spike.md) | |

## Decisions (from design discussion)

- One concrete layer only: **edges** (instance → instance, derived from code). No rule
  entity, no declarative rule table. Judgment is ordinary Luau in `verify`, owned by the
  constrained component; a "rule" is reified only as a finding `code` when violated.
- One judgment hook: `verify(c, instance, graph)` moved **post-scan** (nothing consumed
  verify findings mid-scan; Luau ignores the extra arg, so existing components are
  unaffected). No separate `check` hook.
- `verify` is **optional** (omission ≡ `return {}`); components classify by `id` + `parse`.
- `identify` requirement is unchanged: optional in the loader, required in practice for
  stack roots (locator provenance: roots from `identify`, children from parent `parse`).
- `index`/`graph` are always the **full** forest, never the reachable part.
- The abstract kind-level graph is a **pure aggregation** over resolved edges — no
  `relate(instance = nil)`, no stack-level mechanism.
- Edge dedupe: **merge** by `(from, kind, target)` into one canonical edge that always
  carries `sources[]` (multiplicity derived as `#sources` — no stored `count`);
  `relation.duplicate` info finding only for identical-provenance redundancy
  (updated 2026-09-28 — was silent first-wins).
- Facade surface: plain JSON data + sibling `graph_facade.luau` via `include_str!` (no mlua closure
  plumbing); richer algorithms live in require-able helper libraries. Plus
  node-centric traversal on `byId`/`resolve*` results; graph vocabulary
  (`nodes()`/`edges()` — instances are the graph nodes) (2026-09-28).
- Naming: the parse-tree payload field is renamed `node` → `ast` so "node" means only
  the graph/tree vertex, never the payload (`instance.ast.name`) (2026-09-28).
- Operations receive the graph too: `apply(c, args, target, graph)` (4th arg; existing
  operations ignore it). Guards reject illegal requests at dry-run time. Scaffold gets
  it as 3rd arg for symmetry (2026-09-28).
- Dry-run of `verify`/`relate` triggers a full scan when no cached forest exists.

## Behavior changes vs v1.3

- `verify` runs after the full scan (was: immediately after each instance's `parse`).
- `verify`/`relate` run for **every node with a `componentId`**, including nested
  children (was: roots only; child verify previously ran only when a parent called it
  via the delegate — parents should stop doing that).
- Scan responses gain `edges` and `kindGraph`; instance nodes gain `relations`.
