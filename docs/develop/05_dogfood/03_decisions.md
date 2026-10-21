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

## Host-shipped Luau (`include_str!`)

**Host-shipped Luau that Rust evaluates stays as a real `.luau` file beside the
consumer `.rs`, loaded with `include_str!`.** Do not embed multi-line Luau in Rust
raw strings.

- Applies under `rust/crates/host/src/**` (example:
  [`graph_facade.luau`](../../../rust/crates/host/src/component/relations/facade/graph_facade.luau)
  next to `builders.rs`).
- Does **not** apply to workspace assets under `.codemod/**` (already files).
- Does **not** mean modeling those sibling `.luau` files in the dogfood component
  graph (same anti-recursion stance as `.codemod/components`).

Semantics: [Facades](../03_relations/01_semantics.md#facades-sibling-luau-via-include_str).

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
| Host domain modules (recipe, edit_session, host_api, components, query_intel) | matching `domain.*` | Architecture ([D-025](#domain-restructure-d-025)); legacy `rust.runtime` retired when path map lands |
| Domain dual types | `domain.types` | Architecture |
| TS under `extension/` / `webview/` / `shared/` | matching `vscode.*` | Architecture (path → zone) |
| Rust barrels / support / engine / lua surfaces | `structural.modules` | `ignoreRelations` (barrels only) |
| Coverage children (`*_item`, `ts_export`, …) | `lives_in` → parent | Inherit; do not park parents in inventory |

**Visualization:** kind graph = readable architecture story; instance graph = **complete** map (all membership + edges). Snapshot JSON top-level order: `meta`, `coverage`, `relations`, then instances/edges/…

### Architecture ratio eligibility (inferred)

Span coverage and architecture connectivity are **independent** (spans vs edges).
Architecture % does **not** use a per-component `metricRole`. Eligibility is inferred:

- **Eligible:** not `ignoreRelations`, and either zone-rooted / free root, **or** nested under a non-zone parent but present on a non-membership edge (`uses` / `feeds` / …).
- **Not eligible:** membership-only children of another component instance (structural / span members). They stay in the forest and still paint span; they are not `architectureIsolated`.
- **Zone catalog nodes** (`repo.zone/*`): always relation-ignored (membership hubs / labels). Do **not** set `ignoreRelations` on architecture `zoneRoles` — that would exempt every member under those zones.
- **Crate DAG:** `repo.rust_crate` emits `uses` from Cargo.toml `path = "../dep"` (`pathDeps`) to other crate instances.
- **Adapter inventory roots:** `repo.vscode_command` / `vscode_manifest` / `webview_message` / `mcp_tool` emit `defines` → each entry child so the surface root is architecture-connected (entries already carry triangle edges). Direct MCP tools still `lives_in` the parent even without `exposes`.
- **`repo.host_api_method` source globs:** after handlers/delegate grain splits, method discovery uses
  `host_api/handlers/**/*.rs`, `lua_runtime.rs`, and `component/delegate/**/*.rs`, with identify keyed
  on `handlers/mod.rs` (not the deleted flat `handlers.rs` / `delegate.rs`).
- Ratio: `architectureConnected / architectureEligible` (see `relations.aggregate` in the scan snapshot).

Amends the spirit of D-020: nested `lives_in` → parent unit means “part of that model,” not “architecture-isolated orphan.”

## Domain restructure (D-025)

Module-first restructure so dogfood can model **domains**, not bags of functions.
Companion policy: [Target architecture](05_target_architecture.md) domain table.

### Vocabulary

| Term | Meaning |
|------|---------|
| **Adapter** | Thin client of `HostCommand` (MCP, hostBridge, VS Code COMMANDS) |
| **Application** | `protocol/dispatch/*`; stack `createCommand` — orchestration |
| **Domain module** | Cohesive ownership inside host (or extracted crate): one public API, one zone |
| **Platform crate** | Leaf libraries per crate DAG (`lua`, `engine`, `core`, …) |
| **Surface** | Temporary pub-leaf composer; retire when domain kinds exist |

`HostCommand` (RPC) and `host_api` / `c:*` (in-sandbox) remain **two different cores**.

### Rename

`host/src/engine/` → `host/src/edit_session/` so it does not collide with
`codemod_recipe_engine`.

### Zones (evolve from flat `rust.runtime`)

| Zone id | Owns |
|---------|------|
| `command.adapters` | Protocol, MCP, bridge, VS Code commands (unchanged) |
| `domain.recipe` | Registry, args, validate |
| `domain.edit_session` | WorkingTree, preview/apply orchestration, postExec |
| `domain.host_api` | Luau `c:*` handlers |
| `domain.components` | Scan, relations, coverage, snapshot, hooks |
| `domain.query_intel` | Host wrappers over query_tools |
| `domain.types` | Shared / dual domain type sites |
| `structural.modules` | Barrels / support surfaces (`ignoreRelations`) |
| `vscode.*` | Path-mapped extension / webview / shared |

### Crate extraction criteria

Extract a **new crate** only if **all** hold:

1. Clear DAG role (extends the crate table in target architecture).
2. Independently testable without protocol/MCP/VS Code.
3. ≥~2k LOC cohesive domain **or** blocking a forbidden dependency.
4. Decision entry here + `repo.rust_crate` DAG update in the same change.

Default: **split modules first**; open an extraction ADR only when criteria are met.
Likely later candidates: `codemod_recipe_components`, `codemod_recipe_host_api`.

### D-016 type ownership (resolved)

- **`QuerySpec`:** owned in [`codemod_recipe_core::query_spec`](../../../rust/crates/core/src/query_spec.rs); `engine` re-exports; `lua` keeps a FromLua twin + `From` into core (orphan-rule adapters).
- **`IfExists` / `IfMissing`:** owned in `core::apply::file_change`; lua FromLua twins + `From` into core at the host_api boundary.

### D-018 stub distribution (resolved)

Generate once into `vscode_extension/stubs/codemod-context.d.luau` (SSOT).
`scripts/sync-luau-stubs.sh` copies into `export/.codemod/stubs/` and `.codemod/stubs/`
for bootstrap/IDE. Copies are deploy artifacts — never hand-edit.

### Crate extraction deferred (D-025 F)

`host/src/component/` (~5.5k LOC) meets size, but **fails** independent testability
without `SharedHost` / `WorkingTree` / protocol. Keep as `domain.components` modules
inside host until a thin facade API exists. Revisit when criterion 2 holds.

### HostInner facet sketch (toward D-025 F criterion 2)

`SharedHost = Rc<Mutex<HostInner>>` ([`host_api/context.rs`](../../../rust/crates/host/src/host_api/context.rs))
is the remaining concentration smell. **Do not extract a crate yet.** Facet
**views** live in [`host_api/facets.rs`](../../../rust/crates/host/src/host_api/facets.rs)
(same mutex; narrower call-site access):

| Facet | Owns (from HostInner) | Status |
|-------|------------------------|--------|
| `RecipeSession` | `kind`, `tree`, `sandbox`, args/maps/vars/queries/recipes, transcript, … | types ready; handlers not migrated yet |
| `ComponentSession` | `components`, `component_stack`, graph JSON, `codemod_root` | **Phase 1:** delegate stack + graph snapshot paths use it |
| `QueryIntel` | `language_registry`, `ast_cache` | types ready; query paths not migrated yet |

Extraction ADR only after a facet is testable without MCP/VS Code.

### Framework dogfood backlog (Track C2)

Shipped (D-029):

- `summary.productSpanRatio` — product-only span (excludes `**/tests/**`, `*.test.*`, …)
- `composer.identify_path_conflict` when two stack roots identify the same path
- CLI one-shot: `codemod_mcp --scan-summary` prints compact scan summary JSON
- Leaf-wipe audit: `repo.rust_mod_file`, `repo.vscode_manifest`, `repo.ts_test`, `lib/rust_test_children` keep partial children on `pcall` failure

**Rejected:** auto-appending stack `codebase` `!` excludes to `c:listFiles`. `codebase` is the coverage universe only; identify/listFiles take caller globs (composers mirror excludes in identify when needed). Both still share `resolve_globs` semantics.

### Path-module graph + product tests (K–M)

Path inventories (`repo.ts_module`, `repo.rust_mod_file`) stay architecture-eligible:
isolation means missing relates, not “drop from metrics.” Generation needs module
edges (`declares_mod` / `imports` / `exports` / `uses`).

**Span universe:** declared only by `stack.codebase` globs (shared `resolve_globs`;
optional `!pattern` excludes). `src/bin/*.rs` is **product** (host/MCP entrypoints) —
do not carve it out. Non-product trees (e.g. `target/`, `node_modules/`) stay out
via include globs or explicit `!` excludes — never host path hardcodes.

**Product tests in the graph:** `repo.rust_test` / `repo.ts_test` identify
`crates/*/tests/**/*.rs` and `*.test.ts` (not Luau `*.test.luau`). Edge `tests` →
production `rust_mod_file` / `ts_module`. Pilot scaffold:
`repo.ts_module` operation `scaffoldTest` creates a vitest stub beside a `.ts` module.

**Graph scope:** Rust + VS Code product code (+ product tests). Luau under
`.codemod/` is the modeling language — component defs and `*.test.luau` stay off
the architecture/span instance graph.
