# Dogfood decisions (Phase 2)

Architectural calls made while dogfooding the command triangle and cross-layer
mutation. Complements the [issues log](01_issues.md) and [coverage](02_coverage.md).

## Stack operations (chosen)

Cross-layer "add a command" is a **stack operation**, not a recipe and not a
junction component.

- `stack.luau` stays a thin manifest; behavior lives in a require-able module
  (`.codemod/stack_ops.luau` → `{ operations = { ... } }`).
- `operations.createCommand(name, layers)` fans out to per-layer component
  operations (`repo.host_command`, `repo.mcp_tool`, `repo.vscode_command`).
- Each layer op edits only its own files and guards its own preconditions via
  the graph; the stack op guards cross-layer concerns (name taken anywhere,
  valid layer subset).
- Child hooks run on the same `SharedHost` / working tree, so fan-out is one
  atomic dry-run → apply. Cycle detection already exists on the delegate.
- MCP: `stack_dry_run_operation` / `stack_apply_operation`. VS Code recipe
  runner surfaces the same flow.

First real use: scaffold `componentSnapshot` (host + mcp layers), then implement
the handler into the stubs (Phase C).

## No junction `repo.command` (rejected)

A synthetic command node that only exists to hang `realized_by` edges would be
a thin dummy — another place to drift, another identify surface to keep warm,
little judgment of its own. The command concept is already expressible as:

1. Relation edges (`exposes` / `sends` / `invokes`) between layer components
2. Stack `createCommand` for the mutation side

Completeness judgment stays on `repo.host_command.verify` reading incoming
edges (couples to edge *kinds*, not to sibling component ids).

## Graph-aware recipes (deferred)

Earlier design sketch: give recipes a read-only `c:graph()` facade (lazy
scan-on-demand; empty forest without a stack) so a recipe could orchestrate
cross-layer edits by traversing instances.

Deferred because stack operations subsume the motivating use case **with
guards intact**. Recipes remain unguarded single-purpose edits; graph access
there would add context without the component/stack guard story. Revisit only
if a batch migration over instance sets cannot honestly live as a stack or
component operation.

Sketch preserved for later: lazy scan, empty-forest without stack, reuse
`build_graph_facade`, opt-in only — never a backdoor around component guards.

## Composition rules (working)

| Concern | Owner |
|---------|--------|
| Layer facts (enum variant, MCP entry, COMMANDS const, …) | Layer component identify / verify |
| Cross-layer links | `relate` edges (`exposes`, `sends`, `invokes`) |
| Completeness / drift across layers | Graph-aware verify on the owning layer |
| Cross-layer mutation | Stack operation → layer operations |
| Single-file / single-purpose transform | Recipe (no graph required) |

## Deferred (logged, not built)

| Item | Why deferred | Tracking |
|------|--------------|----------|
| Graph-aware recipes | Stack ops cover the need; keep recipes unguarded | this page |
| D-004 MCP inventory SSOT refactor | Drift is **guarded** by `repo.mcp_tool`; eliminating the triple inventory is a larger host refactor | [D-004](01_issues.md) |
| Scan-cache warm-from-disk | Nice latency win; not needed for correctness | [D-013](01_issues.md) |
| Test-layer snapshot/golden assertions | Commit artifacts for humans first; CI golden check later | [D-014](01_issues.md) |

## Placement policy (D-020, Phase F)

Every composer has an architecture or structural home — **no inventory catch-all**:

| Kind / surface | Zone | Notes |
|----------------|------|-------|
| Commands, bridge, MCP, protocol, dispatch arms | `command.adapters` | Architecture |
| Host modules, component pipeline, API/registry/engine/tree | `rust.runtime` | Architecture |
| Domain dual types | `domain.types` | Architecture |
| TS under `extension/` / `webview/` / `shared/` | matching `vscode.*` | Architecture (path → zone) |
| Rust barrels / support / engine / lua surfaces | `structural.modules` | `ignoreRelations` (barrels only) |
| Coverage children (`*_item`, `ts_export`, …) | `lives_in` → parent | Inherit; do not park parents in inventory |

**Visualization:** kind graph = readable architecture story; instance graph = **complete** map (all membership + edges). Snapshot JSON top-level order: `meta`, `coverage`, `relations`, then instances/edges/…
