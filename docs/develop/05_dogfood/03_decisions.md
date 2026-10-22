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

### Host facade then crate extract (D-025 F)

**Criterion 2 held** for the component facade: unit tests use
[`FakeComponentHost`](../../../rust/crates/components/src/host_facade.rs) only
(push/pop cycle, finalize empty/non-empty, graph JSON) — no RecipeRegistry /
MCP / VS Code.

| Phase | Result |
|-------|--------|
| A | `ComponentHost` trait + `FakeComponentHost` in `codemod_recipe_components` |
| B | Runtime finalize / delegate stack / graph snapshot via facade; `HostInner` impl in host; Luau `SharedHost` only at host `component/entry` + scan/runtime adapters |
| C | Fake unit tests in the components crate (24 lib tests incl. relations) |
| D | New crate `codemod_recipe_components` (`core`+`lua`); host depends on it; `repo.rust_crate` DAG + target-architecture table updated |

**Still in host (Luau/registry-coupled):** `component/{entry,runtime,delegate,scan,cache,snapshot,coverage}` — create `SharedHost` for `c:*`, call into the facade for stack/graph/finalize. Full scan/runtime move waits on a `ComponentWorkspace` / Luau-session trait (not this change).

### HostInner facets (still valid)

`SharedHost = Rc<Mutex<HostInner>>` remains the Luau session mutex. Facet
**views** in [`host_api/facets.rs`](../../../rust/crates/host/src/host_api/facets.rs);
`ComponentSession` delegates stack/graph to `ComponentHost`.

| Facet | Owns (from HostInner) | Status |
|-------|------------------------|--------|
| `RecipeSession` | `kind`, `tree`, `sandbox`, args/maps/vars/queries, transcript, … | Phase 2 handlers via `with_host` |
| `ComponentSession` | thin view → `ComponentHost` | Phase 1 + D-025 F facade |
| `QueryIntel` | `language_registry`, `ast_cache` (+ `cached_tree`) | Phase 2 |

### Framework dogfood backlog (Track C2)

Shipped (D-029):

- `summary.productSpanRatio` — product-only span (excludes `**/tests/**`, `*.test.*`, …)
- `composer.identify_path_conflict` when two stack roots identify the same path
- CLI one-shot: `codemod_mcp --scan-summary` prints compact scan summary JSON
- Leaf-wipe audit: `repo.rust_mod_file`, `repo.vscode_manifest`, `repo.ts_test`, `lib/rust_test_children` keep partial children on `pcall` failure

**Rejected:** auto-appending stack `codebase` `!` excludes to `c:listFiles`. `codebase` is the coverage universe only; identify/listFiles take caller globs (composers mirror excludes in identify when needed). Both still share `resolve_globs` semantics.

### Pre-extension cleanup (D-030)

Before extending the framework further:

- **Agent honesty:** one MCP skill path (`.agents` playbook + usage docs); Cursor `codemod-mcp` is a thin pointer; mutating component/stack MCP tools require `previewToken` like recipes; `inlineRecipe` schema is a string.
- **Domain honesty:** `hostZoneForPath` maps sandbox/config/template/explorer; legacy `rust.runtime` catalog entry removed; `preview` lives under `edit_session`, `lua_runtime` under `host_api`; protocol DTOs split (`command.rs` / `responses.rs`).

### Dogfood cleanliness (D-033)

Follow-up after D-029/D-030 path split and dual-root enforcement:

- **HostCommand locate:** protocol identify/parse/scaffold + fixtures use `protocol/command.rs` (not `mod.rs`). `host_module` still owns the `protocol/mod.rs` barrel.
- **Domain-owned path opt-outs:** `lib/rust_owned_paths.luau`; `repo.rust_mod_file` skips domain/platform trees (mirror `ts_module`). Broader surfaces yield to grain (`engine_surface` ↔ `engine_edit`, `support_crate_surface` ↔ `domain_type`, `lua_host_api_specs` ↔ domain leaves).
- **Test co-locate allowlist:** `composer.identify_path_conflict` skips when either root is `repo.rust_test_module` (inline `mod tests` must share the production path).
- **`tests` edges:** `lib/rust_test_target.luau` resolves production owners beyond `rust_mod_file` (domain composers after opt-outs; aliases for multi-file units like `ops.rs` → `engine_edit`, `mcp_inventory.rs` → `mcp_tool`; `ts_test` → `webview_message` for `messages.ts`).
- **VS Code anchors:** `vscode_command` identifies `constants.ts` (manifest stays `vscode_manifest`); `ts_module` skips `hostBridge.ts` / `messages.ts` / `constants.ts`; `vscode_ext_bootstrap` paints `index.ts`.
- **host_module ownership:** most-specific directory MODULES entry wins (`preview/` under `edit_session/`); exact leaf MODULES paths (`lua_runtime`, `host_api_registry`) own themselves.
- **Span residual:** pipeline/host/lua composers also paint `use_declaration`, `macro_invocation`, and `line_comment` (doc comments were the bulk of `testing.rs` gap).

### Facet Phase 2 + Sweep 3 grain (D-031)

- **Facets:** `RecipeSession` / `QueryIntel` migrated for the handler families above (same mutex; no crate extract).
- **Grain:** VS Code `HostCommand` → `shared/hostCommand.ts`; `recipe/{schema,fs_walk}.rs`; `template/{legacy,env}.rs`.
- Still deferred: crate extraction (D-025 F), D-013 warm-from-disk, golden mermaid CI.
- HostCommand TS/MCP codegen: done as D-034.

### MCP HostCommand parity (D-032)

Agent-useful HostCommands already used by the VS Code bridge are now MCP tools:
`reload_recipes`, `diff_recipe`, `derive_args`, `filter_explorer_recipes`, `query_captures`.
Dedicated scaffold/operation MCP tools remain omitted (covered by hook tools).

### HostCommand adapter sync (D-034)

- **SSOT:** hand `command.rs` (+ dispatch bodies, MCP `input_schema` / tools/call).
- **Generated:** `hostCommand.g.ts`, `mcp_inventory.g.rs` via shared Luau `host_command_adapters` (`expectedFiles`); hand wrappers re-export / `include!`.
- **Verify ≡ sync:** `adapter.generated_drift` diffs disk against the same renderers as `syncAdapters`.
- **MCP policy:** opt-in rows in `MCP_TOOL_SPECS` (not 1:1 every HostCommand).
- **Zones:** drop empty `docs.surface`; keep `support_crate_surface` (span painter).
- **Out of scope:** generating dispatch or tools/call arms. Host facade / crate extract: D-025 F.

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
production owner (`rust_mod_file`, domain composer, or `ts_module` — see D-033
`rust_test_target`). Pilot scaffold:
`repo.ts_module` operation `scaffoldTest` creates a vitest stub beside a `.ts` module.

**Graph scope:** Rust + VS Code product code (+ product tests). Luau under
`.codemod/` is the modeling language — component defs and `*.test.luau` stay off
the architecture/span instance graph.
