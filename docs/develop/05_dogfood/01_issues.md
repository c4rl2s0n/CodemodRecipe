# Dogfood issues log

Running log of bugs, missing features, UX papercuts, and architectural
findings encountered while applying codemod-recipe to this repository.
Severity: `bug` > `gap` (missing feature) > `ux` (papercut) > `note`.
Status: `open` / `in-progress` / `fixed` / `wontfix(decision documented)`.

## 2026-09-29 — PoC kickoff (pre-flight research)

| ID | Severity | Area | Issue | Status |
|----|----------|------|-------|--------|
| D-001 | gap | setup | codemod-mcp was not configured for this workspace (no `.cursor/mcp.json`); dogfooding required adding one. | fixed (Phase 0: committed `.cursor/mcp.json`) |
| D-002 | bug | repo hygiene | Root `.codemod/` doubled as a test fixture: `query_chain_test.rs` loaded `RecipeRegistry` on the repo root (expecting `dart_queries.*`) and wrote `lib/foo.dart` into the real repo root; `mcp_test.rs`/`dispatch_test.rs` copied `.codemod/recipes/insert_log_line.luau`; `dispatch_test.rs::list_reports_loaded_maps_from_repo` asserted on root `.codemod/maps/column_type.luau`. Decision: root `.codemod` is self-hosting-only; test assets belong under `test/`. | fixed (Phase 0.5: assets moved to `test/fixtures/dart_demo/.codemod`, tests repointed, `query_chain_test` now uses `temp_workspace`) |
| D-003 | bug | protocol | Drift: `componentGraph` and `generateAstPath` exist in Rust `HostCommand` (`protocol/mod.rs`) but are missing from the TS mirror (`vscode_extension/src/shared/types.ts`). Re-identified by `repo.host_protocol` scan (exactly 2 findings, no false positives). | fixed (Phase 3: union members added; component now guards) |
| D-004 | bug | architecture | MCP tool inventory was duplicated 3x. **Policy**: MCP is a HostCommand-derived adapter ([05_target_architecture.md](05_target_architecture.md)). Phase 3: `protocol/mcp_inventory.rs` SSOT for tools/list; `repo.mcp_tool` rejects handwritten lists. tools/call arg extraction remains in `codemod_mcp.rs` but must match inventory. | fixed (Phase 3 adapter inventory) |
| D-005 | bug | tests | `HOST_API_SURFACE` expected-names consistency test incomplete — `queryMatches` missing from the hardcoded list (`host_api/registry.rs` ~L125–150). | fixed (Phase 3: added to expected list) |
| D-006 | note | setup | Grammars lazy-download from GitHub on first use; first scan of a new language needs network or a pre-warmed `~/.cache/tree-sitter-language-pack`. | open |
| D-007 | ux | docs | `docs/usage/.../06_examples_index.md` links to root `.codemod` demo recipes; repo-relative example links are fragile for consumers because `docs/usage` is bootstrapped. Repo-side links repointed to `test/fixtures/dart_demo/` in Phase 0.5; the consumer-side fragility (bootstrapped docs referencing repo-local example files) remains as a documented decision. | fixed (repo-side) / wontfix-documented (consumer-side) |
| D-008 | ux | mcp | Adding `.cursor/mcp.json` does not adopt the server into an already-running agent session — a reload/new session is required. Workaround: drive the same JSON-RPC over stdio (`codemod_mcp` binary) from the shell. | open |

## 2026-09-29 — Phase 1 (first components)

| ID | Severity | Area | Issue | Status |
|----|----------|------|-------|--------|
| D-009 | bug | mcp / query_tools | `debug_query` with default `instrument: true` breaks any query whose pattern ends with predicates: instrumentation appends `@__layer_1` *after* the trailing predicate → `Query error ... Invalid syntax`. Repro: any `repo_*` library query via `debug_query` without `instrument: false`. Workaround: pass `instrument: false`. | fixed (inject `(type @__layer_N …)` before predicates) |
| D-010 | ux | host api docs | `listFiles` surface description says "a `*` glob (one `*` per segment)", but `**` recursive segments are implemented (`workspace_glob.rs` `**` arm + `glob_double_star_recursive` test). Description undersells the feature. | open |
| D-011 | bug | extension | **Found by `repo.vscode_command` scan**: manifest command `codemodRecipe.queryTools.selectAstNode` has no `COMMANDS` entry and is registered by string literal (`queryToolsController.ts` L77, referenced in `astTreeProvider.ts` L85) — bypasses the COMMANDS SSOT. | fixed (Phase 3) |
| D-012 | ux | query authoring | Unbalanced-paren mistakes in `[[...]]` query libraries surface as `Query error at L:C. Invalid syntax` pointing at the trailing predicate line, with no hint that parens are unbalanced. Cost one debug cycle per new query library. | fixed (paren pre-check + Invalid syntax Hint) |

## 2026-10-09 — External MCP agent feedback (DataModel)

| ID | Severity | Area | Issue | Status |
|----|----------|------|-------|--------|
| D-021 | ux | docs | Capture `@` placement (after type vs after `)`) unclear to agents. | fixed (captures.md + troubleshooting) |
| D-022 | ux | docs | Wildcard / anonymous-token rules and dump-ast-first workflow underdocumented. | fixed (basic patterns + authoring workflow) |
| D-023 | gap | host api | Agents slice spans with Lua 1-based `sub`; need span helper + hit.text guidance. | fixed (`c:spanText` + docs) |
| D-024 | ux | mcp | `c:log` hard to find in MCP JSON; agents check `component_scan` (no transcript). | fixed (top-level `logs[]` on dry-run/preview; MCP docs/playbook) |

## 2026-09-29 — Phase 2 (relations, stack operations, snapshots)

| ID | Severity | Area | Issue | Status |
|----|----------|------|-------|--------|
| D-013 | gap | host / scan | Scan-cache warm-from-disk: reloading a session still pays full identify cost; warming the forest from a committed snapshot was deferred (correctness does not need it). | open (deferred — see [03_decisions.md](03_decisions.md)) |
| D-014 | gap | tests / CI | Test-layer golden assertions against `scan_snapshot.json` / mermaid artifacts deferred; committed artifacts are for humans and regen docs first. | open (deferred — see [03_decisions.md](03_decisions.md)) |
| D-015 | note | architecture | Junction `repo.command` component rejected; command concept = relation edges + stack `createCommand`. Graph-aware recipes deferred (stack ops cover the use case with guards). | wontfix-documented / deferred ([03_decisions.md](03_decisions.md)) |

## 2026-09-30 — Phase 3 (architecture-first)

| ID | Severity | Area | Issue | Status |
|----|----------|------|-------|--------|
| D-016 | gap | architecture | Dual domain types: `QuerySpec` / `IfExists` exist in both `lua` specs and `engine`/`core`; host bridges by hand. Expected: single ownership. **Enforced** by `repo.domain_type` (`domain_type.dual_ownership` warnings + `mirrors`). | open (architecture-first — [05_target_architecture.md](05_target_architecture.md)) |
| D-017 | gap | architecture | Host crate concentration: protocol, registry mega-state, WorkingTree, components, bootstrap all in `codemod_recipe_host`. Expected: thinner integrator. **Phase F**: split `component/{relations,scan,runtime}` into modules + `protocol/dispatch/` per-domain handlers (cleared `stage_oversized` / `dispatch_dual_model`). Remaining: `lua/src/load.rs` mega-file. | open (partial — [02_coverage.md](02_coverage.md)) |
| D-018 | gap | architecture | Generated stub fan-out under `.codemod/stubs`, `export/.codemod/stubs`, `vscode_extension/stubs`. Expected: one generated surface. | open (backlog) |
| D-019 | note | host / scan | Machine snapshot path is `{codemod_root}/.cache/scan-snapshot.json` (not docs/). Docs JSON dump removed; mermaid remains under `05_dogfood/`. | fixed (Phase 3) |
| D-020 | decision | architecture | Placement policy: no inventory catch-all. Module composers live in architecture zones (`rust.runtime`, `vscode.*` by path, `command.adapters`); structural barrels keep `ignoreRelations`; coverage leaves nest under parents. Snapshot keys ordered `meta` → `coverage` → `relations` → rest. Full instance graph retained for completeness. | fixed (Phase F) |

## Template for new entries

Append to the current date section (or start a new one):

```
| D-NNN | severity | area | What happened, repro, expected vs actual. | status |
```
