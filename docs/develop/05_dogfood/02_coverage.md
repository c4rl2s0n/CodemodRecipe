# Dogfood coverage

Component inventory for the self-hosting stack (`repo-stack` in the root
`.codemod/stack.luau`), what each verifies, and what it has found.

Expected layering and smells: [Target architecture](05_target_architecture.md).
Command-triangle scan view: [Architecture graph](04_architecture_graph.md).

Layout: components live under `.codemod/components/{rust,host,vscode,engine,lua}/`
(ids remain dotted `repo.*`).

## Span coverage program

| Phase | Goal | Status |
|-------|------|--------|
| A | Narrow `codebase` to `crates/*/src`; structural `rust_mod_*` + `ts_module`/`ts_export`; deepen existing leaves; nest component dirs | done |
| B | Host modules + component pipeline stages (pub fn/type leaves) | done |
| C | engine / lua / core / query_tools / lua_macros surfaces | done |
| D | VS Code via `ts_module` (controllers/views as optional non-root composers) | done (structural); specialized roots deferred to avoid triple-identify |
| E | Relation policy (child `lives_in` + inventory zone) + adapter `handles` edges + deepen hot partials; friction→D-017 clarity | done |
| F | Placement (D-020): drop inventory catch-all; zone modules by architecture; split component/dispatch mega-files; snapshot key order | done |
| K | Path-module edges: `declares_mod` / `uses` (Rust), `imports` / `exports` (TS barrels); host_module + pipeline domain edges | done |
| L | Span deepen (private fns, Vue `<script>`); span universe = `stack.codebase` only (no host path hardcodes) | done |
| M | Product test composers (`repo.rust_test` / `repo.ts_test`) + `tests` edges + `scaffoldTest` pilot | done |
| N | `relation.duplicate` hygiene (path-qualified child ids); `engine_surface` → `engine_edit` edges | done |

Latest snapshot (2026-10-20): span **~51.6%**; architecture-connected **~85.2%**
(`329 / 386` eligible — [inferred eligibility](03_decisions.md#architecture-ratio-eligibility-inferred)).
Path-module `declares_mod`/`imports`/`exports`; product `tests` edges; findings clean
(no `relation.duplicate`). Path modules stay eligible; scores move only with real edges.

**Metric:** snapshot `coverage.aggregate` + `relations.aggregate` after each scan. Aim for spanned architecture units and a trustworthy architecture ratio (not 100% private bodies).

### Framework friction → clarity (D-017 / D-025 grain)

Mega-files fight the framework: path-only composers + pub-fn leaves leave private/orchestration unscanned, and dual identities blur relate targets. **Deepening alone has diminishing returns** until grain improves.

| File | Status |
|------|--------|
| `component/{relations,scan,runtime}` | **Split** (Phase F) into `relations/`, `scan/`, `runtime/` modules — `stage_oversized` cleared |
| `protocol/dispatch` | **Split** (Phase F) into `dispatch/{catalog,query,recipe,components,common}.rs` + thin `mod.rs` — `dispatch_dual_model` cleared |
| `lua/src/load/` | **Split** (D-025 A1) by asset kind — `lua.load_oversized` cleared |

### Architecture adjustments (where modeling fights the tree)

| Area | Issue | Suggested product change |
|------|--------|--------------------------|
| D-016 dual types | **Fixed** — core owns `QuerySpec`; lua adapters + `From` | — |
| D-018 stubs | **Fixed** — generate once; sync copies | — |

## Components

| Component | Root? | Models | Verifies / notes |
|-----------|-------|--------|------------------|
| `repo.rust_crate` | yes | `Cargo.toml` crates | packaging DAG |
| `repo.rust_mod_file` / `repo.rust_mod_item` | yes / child | `lib.rs` / `mod.rs` barrels | mod + use leaves |
| `repo.support_crate_surface` | yes | core / query_tools / lua_macros | pub types/fns |
| `repo.host_protocol` (+ `host_command`, `host_dispatch_arm`) | yes | protocol + dispatch | rename/dispatch/TS parity |
| `repo.mcp_tool` / entry | yes / child | MCP inventory | adapter SSOT |
| `repo.host_bridge` / method | yes / child | hostBridge | sends |
| `repo.vscode_command` / entry | yes / child | package.json commands | COMMANDS parity |
| `repo.host_api_method` / entry | yes / child | `#[host_method]` | stub + TS surface |
| `repo.engine_edit` | yes | Engine + EditOp | produces FileChange |
| `repo.working_tree` | yes | WorkingTree | finalizes |
| `repo.recipe_registry` | yes | RecipeRegistry | feeds host API |
| `repo.domain_type` | yes | dual QuerySpec/IfExists | D-016 warnings |
| `repo.host_module` / item | yes / child | host `src/*` modules | pub leaves; `rust.runtime` |
| `repo.component_pipeline` | yes | component/* stages | pub fn/type leaves; post–Phase F module split |
| `repo.engine_surface` | yes | engine/src | pub leaves |
| `repo.lua_surface` | yes | lua/src | pub leaves (load split → `lua_load_module`) |
| `repo.lua_load_module` | yes | `lua/src/load/*.rs` | asset-kind load stages |
| `repo.host_api_handlers` | yes | `host_api/handlers/*` | capability groups |
| `repo.webview_message` / entry | yes / child | messages.ts | membership under webview |
| `repo.ts_module` / `ts_export` | yes / child | extension TS/Vue (not `*.test.ts`) | `imports`/`exports`; Vue script span; `scaffoldTest` |
| `repo.rust_test` / `repo.ts_test` | yes | product tests | `tests` → production module |
| `repo.vscode_controller` / `webview_view` | (files present, not stack roots) | specialized composers | avoid duplicate identify |
| `repo.zone` | yes | `stack.zones` catalog | membership subgraphs + role titles |

Stack ops: `createCommand` fan-out (see [Decisions](03_decisions.md)).
`relations`: edge vs `lives_in` member subgraphs; path-module `declares_mod`/`imports`/`exports`/`tests`; `handles` = dispatch→command.

**`stack.codebase`:** `rust/crates/*/src/**/*.rs`, `rust/crates/*/tests/**/*.rs`,
`Cargo.toml`, `vscode_extension/src/**/*.ts|.vue`, `package.json`. Span skips
`src/bin` entrypoints included. Optional `!` excludes for artifacts. Luau `.codemod/**` is not in the product graph.
Snapshot stores compact `coverage` (no gaps) + `relations` metrics; live `component_coverage` has gaps.

### Zones + relation metrics

Catalog in `.codemod/lib/zones.luau` (wired from `stack.zones` / `zoneRoles`):

| Zone id | Role | Notes |
|---------|------|--------|
| `structural.modules` | `structural` | `ignoreRelations` — Rust barrels / support surfaces |
| `command.adapters` / `domain.*` / `platform.engine` / `platform.lua` / `rust.runtime` / `vscode.*` | `architecture` | Title + grouping (no metric flags) |
| `docs.surface` | `documentation` | Title-only |

Coverage children (`host_module_item`, `ts_export`, `rust_mod_item`, `host_api_method_entry`) emit `lives_in` → parent so zone flags reach the leaf forest. Do **not** blanket-`ignoreRelations` on `command.adapters` / all `vscode.*` (that greenwashes the triangle).

Snapshot `relations.aggregate`: `relationIgnored`, `architectureIsolated`,
`ratios.architectureConnected = architectureConnected / (instanceCount - relationIgnored)`.

## Verified findings record

| Date | Component | Finding | Issue | Resolution |
|------|-----------|---------|-------|------------|
| 2026-09-29 | `repo.host_protocol` | `generateAstPath` + `componentGraph` missing from TS union | D-003 | fixed |
| 2026-09-29 | `repo.vscode_command` | selectAstNode bypass COMMANDS | D-011 | fixed |
| 2026-09-30 | `repo.domain_type` | dual ownership | D-016 | fixed (D-025 C) |
| 2026-09-30 | `repo.lua_surface` | load.rs oversized info | D-025 A1 | fixed (`lua/src/load/`) |
| 2026-09-30 | `repo.host_protocol` | dispatch dual-model info | D-017 | fixed (dispatch split) |
| 2026-09-30 | `repo.component_pipeline` | relations/scan/runtime oversized info | D-017 | fixed (module split) |

## Kind-graph shape (command triangle + runtime domain)

```
vscode_command_entry --invokes--> host_bridge_method --sends--> host_command
mcp_tool_entry --exposes--> host_command
host_dispatch_arm --handles--> host_command
host_api_method --drives--> engine_edit
host_api_method --stages_via--> working_tree
recipe_registry --feeds--> host_api_method
host_command --uses--> engine_edit / recipe_registry
domain_type --mirrors--> domain_type
rust_mod_file --declares_mod--> rust_mod_file
ts_module --imports--> ts_module
ts_module --exports--> ts_module   (barrels)
rust_test / ts_test --tests--> rust_mod_file / ts_module
```

## Backlog

1. Adapters codegen from HostCommand.
2. Retire blanket `repo.engine_surface` once engine domain kinds deepen (edges already → `engine_edit`).
3. `repo.rust_workspace` / docs / codemod_asset.
4. Full mermaid CI goldens (D-014 partial: schema test only).
5. Promote `vscode_controller` / `webview_view` to roots with verify-only if needed.
6. Crate extraction when [criteria](03_decisions.md#crate-extraction-deferred-d-025-f) hold (deferred).
7. Deeper product-test scaffolds (Rust `#[test]` / more than vitest stub).

### Restructure handoff log

| Date | Slice | Notes |
|------|-------|-------|
| 2026-10-19 | D-025 Phase 0 | Docs: domain table, zones, extraction criteria |
| 2026-10-19 | D-025 A1 | `lua/src/load/` split by asset kind; `lua.load_oversized` cleared |
| 2026-10-19 | D-025 A2 | `host_api/handlers/` split by capability |
| 2026-10-19 | D-025 B | `edit_session` rename; `domain.*` zones |
| 2026-10-19 | D-025 C | `QuerySpec` in core; lua adapters; D-016 fixed |
| 2026-10-19 | D-025 D | VS Code: recipeRunner/*, HostBridgeCore, useComponentsView |
| 2026-10-19 | D-025 E | `repo.lua_load_module`, `repo.host_api_handlers`; stub sync SSOT; snapshot schema test |
| 2026-10-19 | D-025 F | Crate extraction deferred (components still need SharedHost) |
| 2026-10-19 | D-025 rescan | Snapshot refreshed; span ~45.4%; domain edges; `lua→core` DAG allow; D-010 |
| 2026-10-19 | Arch eligibility | Inferred architecture ratio (exclude membership-only nested); pipeline `feeds`; ADR in decisions |
| 2026-10-20 | K–N gaps | Path-module relates; product tests + scaffoldTest; child id hygiene; span ~51.6% / arch ~85.2%; findings clear |
| 2026-10-20 | Glob excludes | Shared `resolve_globs` + `!` for listFiles/codebase; removed coverage `/bin/` hardcode; `src/bin` is product |
