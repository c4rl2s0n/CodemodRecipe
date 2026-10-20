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
| O | `rust_test_module` + case leaves; `protocol_key`; `vscode_bootstrap`; host/bin deepen | done |
| P | Exclude Cargo.toml from span; `vue_sfc` + sections/scaffold; command `presents`; host grain splits | done |
| Q | Host/lua grain deepen; Vue full section spans; `vscode_manifest` + `json_field`; `protocol_key` `defines` | done |
| R | Extension architecture grain: bootstrap / language context / recipe invoke; domain composers | done |
| S | Honest megas: lua `host_api/specs/*`, activate lifecycle; composers; import-resolve fix | done |
| T | Fix `component_pipeline` leaf wipe (`pub_types` → `type_items` + consts; no children wipe) | done |
| U | Deepen lua load/specs + engine/support `all_functions`; zone catalog out of arch ratio; crate `uses`; TS tooling/`context` barrel | done |
| V | Adapter root `defines` → entries; VS Code fn/method leaves; bootstrap `uses` App.vue | done |
| W | Restore `host_api_method` after handlers split; `lua_runtime` in `host_module`; TS imports + Rust attrs | done |

Latest snapshot (2026-10-20): span **~93.9%**; architecture-connected **~100%**
(`526 / 526` eligible — [inferred eligibility](03_decisions.md#architecture-ratio-eligibility-inferred)).
Phase W restored `repo.host_api_method` (glob `handlers/**` + `lua_runtime` + `delegate/**`; identify
`handlers/mod.rs`) — was zero instances after the handlers grain split. `host_module` deepens
`lua_runtime.rs` (~13% → ~99%). `ts_module` paints import statements; host/lua/pipeline composers
paint `attribute_item` leaves (`#[host_method]`, …). Zero architecture isolates.

**Rust host `src/component/**` is modeled** by `repo.component_pipeline` (fn / type / string-const /
attr leaves). Phase S wrongly called its residual an “accepted plateau”; the real bug was a nonexistent
`repo_rust.pub_types` query plus `children = {}` on `pcall` failure, which discarded all leaves.
Phase T fixed that — host component gap ~105k → ~10k; thin `mod.rs` barrels may still show
`0 leaves` (span from `rust_mod_file` use/mod only). **Do not model** `.codemod/components/**`
(Luau composers — recursive nonsense). Host-shipped facade Luau lives beside Rust as
`graph_facade.luau` + `include_str!` ([decision](03_decisions.md#host-shipped-luau-include_str));
sibling `.luau` under `host/src` is also not dogfood-modeled. Largest remaining host gaps are glue /
doc / fixture strings (`testing.rs`, `registry.rs`) — not unsplit `lua_runtime`.

**Metric:** snapshot `coverage.aggregate` + `relations.aggregate` after each scan. Aim for spanned architecture units and a trustworthy architecture ratio (not 100% private bodies).

### Framework friction → clarity (D-017 / D-025 grain)

Mega-files fight the framework: path-only composers + pub-fn leaves leave private/orchestration unscanned, and dual identities blur relate targets. **Deepening alone has diminishing returns** until grain improves.

| File | Status |
|------|--------|
| `component/{relations,scan,runtime}` | **Split** (Phase F) into `relations/`, `scan/`, `runtime/` modules — `stage_oversized` cleared |
| `protocol/dispatch` | **Split** (Phase F) into `dispatch/{catalog,query,recipe,components,common}.rs` + thin `mod.rs` — `dispatch_dual_model` cleared |
| `lua/src/load/` | **Split** (D-025 A1) by asset kind — `lua.load_oversized` cleared |
| `component/{coverage,delegate,types}` + `relations/resolve` | **Split** (Phase P) into module dirs — pipeline STAGES + `feeds` updated |
| `relations/{resolve,facade}`, `runtime`, `cache`, `snapshot`, `relation_metrics`, `graph_render` | **Split** (Phase Q) — further grain; STAGES/feeds updated |
| `lua/src/load/{ops,type_parse,explorer}` | **Split** (Phase Q) — shared helpers re-exported; `lua_load_module` stages |
| `extension/{bootstrap,language/context,recipeKeybindings}` | **Split** (Phase R) — activate helpers / Luau context / keybinding JSON |
| `lua/src/host_api/specs/{traits,query,common,edits,files,queries,template}` | **Split** (Phase S) — capability grain; `repo.lua_host_api_specs` |
| `extension/bootstrap/lifecycle.ts` | **Extracted** (Phase S) from fat `activate` |

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
| `repo.protocol_key` | yes | `protocol/keys.rs` consts | catalog; TS/MCP mirror verify; `addKey`; `defines` → `host_protocol` |
| `repo.mcp_tool` / entry | yes / child | MCP inventory | adapter SSOT |
| `repo.host_bridge` / method | yes / child | hostBridge | sends |
| `repo.vscode_command` / entry | yes / child | package.json commands | COMMANDS parity |
| `repo.vscode_manifest` / `json_field` | yes / child | package.json top-level + contributes.* | coarse field spans; `lives_in` extension |
| `repo.host_api_method` / entry | yes / child | `#[host_method]` | stub + TS surface |
| `repo.engine_edit` | yes | Engine + EditOp | produces FileChange |
| `repo.working_tree` | yes | WorkingTree | finalizes |
| `repo.recipe_registry` | yes | RecipeRegistry | feeds host API |
| `repo.domain_type` | yes | dual QuerySpec/IfExists | D-016 warnings |
| `repo.host_module` / item | yes / child | host `src/*` modules | pub leaves; `rust.runtime` |
| `repo.component_pipeline` | yes | `host/src/component/**` `.rs` stages | all_functions + type_items + pub_str_consts; not `.codemod/components` or sibling host `.luau` |
| `repo.engine_surface` | yes | engine/src | pub leaves |
| `repo.lua_surface` | yes | lua/src | pub leaves (load split → `lua_load_module`) |
| `repo.lua_load_module` | yes | `lua/src/load/*.rs` | asset-kind load stages |
| `repo.lua_host_api_specs` | yes | `lua/src/host_api/specs/*.rs` | capability specs; `uses`/`feeds` → handlers |
| `repo.host_api_handlers` | yes | `host_api/handlers/*` | capability groups |
| `repo.webview_message` / entry | yes / child | messages.ts | membership under webview |
| `repo.ts_module` / `ts_export` | yes / child | extension TS (excl. domain-owned paths / tests / main.ts / `.vue`) | `imports`/`exports`; `scaffoldTest` |
| `repo.vscode_language_context` | yes | `language/context/*.ts` | Luau completion/context; `uses` among modules |
| `repo.vscode_ext_bootstrap` | yes | `extension/bootstrap/*.ts` | activate workspace setup + pickers |
| `repo.recipe_invoke` | yes | `recipeInvoke` + `recipeKeybindings` | invoke path `uses` → `host_bridge` |
| `repo.vue_sfc` / `vue_section` | yes / child | `*.vue` script/template/style | full-block section spans (depth-aware); `kind` in summary; `imports`; `scaffoldSfc` |
| `repo.vscode_bootstrap` | yes | webview `main.ts` | classify + whole-file span |
| `repo.rust_test_module` | yes | `#[cfg(test)] mod tests` | test/helper leaves; `tests` → mod file |
| `repo.rust_test` / `repo.ts_test` | yes | integration / `*.test.ts` | case leaves; `tests` → production module |
| `repo.vscode_controller` / `webview_view` | (files present, not stack roots) | specialized composers | avoid duplicate identify |
| `repo.zone` | yes | `stack.zones` catalog | membership subgraphs + role titles |

Stack ops: `createCommand` fan-out (see [Decisions](03_decisions.md)).
`relations`: edge vs `lives_in` member subgraphs; path-module `declares_mod`/`imports`/`exports`/`tests`;
`handles` = dispatch→command; `presents` = UI-only command → vscode zone (not fake `invokes`).

**`stack.codebase`:** `rust/crates/*/src/**/*.rs`, `rust/crates/*/tests/**/*.rs`,
`vscode_extension/src/**/*.ts|.vue`, `package.json`, plus `!**/*.d.ts` and
`!**/*.config.ts` (tooling/ambient). `Cargo.toml` is **not** in the span universe
(`repo.rust_crate` still identifies crates via its own listFiles). `src/bin`
entrypoints included. Optional `!` excludes for artifacts. Luau `.codemod/**`
is not in the product graph. Mirror `*.d.ts` / `*.config.ts` skips in `ts_module`
identify (`listFiles` does not inherit codebase excludes).
Snapshot stores compact `coverage` (no gaps) + `relations` metrics; live `component_coverage` has gaps.

### Zones + relation metrics

Catalog in `.codemod/lib/zones.luau` (wired from `stack.zones` / `zoneRoles`):

| Zone id | Role | Notes |
|---------|------|--------|
| `structural.modules` | `structural` | `ignoreRelations` — Rust barrels / support surfaces |
| `command.adapters` / `domain.*` / `platform.engine` / `platform.lua` / `rust.runtime` / `vscode.*` | `architecture` | Title + grouping (no metric flags) |
| `docs.surface` | `documentation` | Title-only |

Coverage children (`host_module_item`, `ts_export`, `rust_mod_item`, `host_api_method_entry`) emit `lives_in` → parent so zone flags reach the leaf forest. Do **not** blanket-`ignoreRelations` on `command.adapters` / all `vscode.*` (that greenwashes the triangle).
**Zone catalog instances** (`repo.zone/*`) are always relation-ignored in metrics —
they are membership hubs, not architecture units (members still use `zoneRoles`).

Snapshot `relations.aggregate`: `relationIgnored`, `architectureIsolated`,
`ratios.architectureConnected = architectureConnected / architectureEligible`.

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
vscode_command_entry --presents--> zone (UI-only; no hostBridge)
mcp_tool_entry --exposes--> host_command
host_dispatch_arm --handles--> host_command
host_api_method --drives--> engine_edit
host_api_method --stages_via--> working_tree
recipe_registry --feeds--> host_api_method
host_command --uses--> engine_edit / recipe_registry
domain_type --mirrors--> domain_type
rust_mod_file --declares_mod--> rust_mod_file
ts_module / vue_sfc --imports--> ts_module / vue_sfc
ts_module --exports--> ts_module   (barrels)
rust_test_module / rust_test / ts_test --tests--> rust_mod_file / ts_module
protocol_key --defines--> host_protocol
vscode_manifest / json_field (package.json fields + contributes.*)
recipe_invoke --uses--> host_bridge
vscode_language_context / vscode_ext_bootstrap (extension domain grain)
```


## Backlog

1. Adapters codegen from HostCommand.
2. Retire blanket `repo.engine_surface` once engine domain kinds deepen (edges already → `engine_edit`).
3. `repo.rust_workspace` / docs / codemod_asset.
4. Full mermaid CI goldens (D-014 partial: schema test only).
5. Promote `vscode_controller` / `webview_view` to roots with verify-only if needed.
6. Crate extraction when [criteria](03_decisions.md#crate-extraction-deferred-d-025-f) hold (deferred).
7. Optional `wrapUnitTests` op for src files missing `mod tests`.
8. Finer test→symbol `tests` edges when AST makes them honest.

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
| 2026-10-20 | Modeling deepen | `rust_test_module` + case/helper leaves (`#[test]` sibling query); `protocol_key` + TS/MCP mirror allowlists + `addKey`; `vscode_bootstrap`; host `testing` + bin fn leaves; span ~73.8% / arch ~83.8% |
| 2026-10-20 | Gaps follow-on | Cargo.toml out of span; `vue_sfc`+`scaffoldSfc`; command local-follow + `presents`; split coverage/delegate/types/resolve; span ~75.8% / arch ~89.6% |
| 2026-10-20 | Phase Q grain+UI | Host resolve/runtime/cache/snapshot/facade/metrics/graph_render + lua load ops/type_parse/explorer; Vue full sections (13/13 @100%); `vscode_manifest`+`json_field` (package.json ~99.5%); `protocol_key` `defines`×15; span ~77.5% / arch ~93.2% |
| 2026-10-20 | Phase R ext grain | Split activate bootstrap + `language/context/*` + recipe keybindings; domain composers (not controller relabels); Vue import dedupe; span ~77.5% / arch ~92.9% (`447/481`) |
| 2026-10-20 | Phase S honest megas | Split lua `host_api/specs/*` + activate `lifecycle.ts`; `repo.lua_host_api_specs` 8/8; skip `lua_runtime` circular split; Vue/TS `.js` import resolve; span ~77.0% / arch ~93.7% (`459/490`) — host component residual later found to be leaf-wipe bug, not plateau |
| 2026-10-20 | Phase T pipeline leaves | Fix `component_pipeline`: `type_items`/`typeItem`, `pub_str_consts` (+ raw strings), per-family pcall (no wipe); retract accepted-plateau wording; span ~85.4% / arch ~93.7% (`459/490`); host `src/component` gap ~10k |
| 2026-10-20 | Host-shipped Luau | Extract `graph_facade.luau` + `include_str!` in `builders.rs`; delete `facade/script.rs`; drop `relations_facade_script` stage; decision in `03_decisions` |
| 2026-10-20 | Phase U deepen+arch | `lua_load_module`/`lua_host_api_specs`/`engine`/`support` all_functions; zone catalog ignored in arch ratio; crate `uses`; codebase `!*.d.ts`/`!*.config.ts`; language-context barrel; `ts_test` import fallback; span ~90.0% / arch ~98.7% (`466/472`) |
| 2026-10-20 | Phase V adapters+TS | Root `defines`→entries; mcp direct `lives_in` parent; bootstrap→App.vue; `ts_module` fn/method deepen; span ~91.1% / arch **100%** (`524/524`) |
| 2026-10-20 | Phase W span chase | Restore `host_api_method` globs after handlers split; `lua_runtime` in `host_module`; TS `import_statements` + Rust `attribute_items`; span ~93.9% / arch **100%** (`526/526`) |
