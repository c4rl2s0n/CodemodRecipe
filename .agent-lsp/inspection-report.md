# Inspection Report: codemod_recipe (full repo)

**Timestamp:** 2026-08-25T08:48:46Z  
**Files analyzed:** ~154 source files (Rust crates + `vscode_extension/src` + scripts; exclusions applied)  
**Checks applied:** `dead_symbol`, `redundancy` (SIMILAR_TO), `silent_failure`, `test_coverage` (via blast_radius), `doc_drift` (index freshness), compiler gates  
**Findings:** 0 errors, 6 warnings, 4 info  

## Tool status

| Gate | Result |
|------|--------|
| codebase-memory-mcp | Ready (4551 nodes / 16016 edges) — **partially stale** |
| agent-lsp Rust | rust-analyzer warm-up OK (`RecipeRegistry::reload` → 48 refs) |
| agent-lsp TypeScript | warm-up OK (`invokeRecipe` → 8 refs) |
| `cargo clippy --all-targets -- -D warnings` | **Pass** (exit 0) |
| `npx tsc -p ./ --noEmit` | **Pass** (exit 0) |

## Errors

_None._ Compiler gates are clean; no verified runtime-risk smells with high blast radius.

## Warnings

| # | Check | File:Line | Finding | Confidence | Fix |
|---|-------|-----------|---------|------------|-----|
| 1 | dead_symbol | `rust/crates/host/src/protocol/keys.rs:12-13` | `FILES` / `FILE` constants have **0 LSP refs**; `patch_selector.rs` uses literal `"files"` | verified (LSP + grep) | Use `protocol_keys::FILES` in `parse_selection`, or delete unused constants |
| 2 | dead_symbol | `vscode_extension/src/webview/src/lib/selection.ts:25` | `allFileRows` exported but never referenced (tests ignore it) | verified (LSP + grep) | Remove lines 25–29, or wire into UI/tests |
| 3 | dead_symbol | `vscode_extension/src/extension/language/dslSurface.ts:45` | `resetDslSurfaceCache` exported as “test helper” but unused even by tests | verified (LSP + grep) | Call from tests after cache mutation, or remove |
| 4 | redundancy | `rust/crates/engine/src/query/query_chain.rs:5` ↔ `query_tools/.../match_debug.rs:255` | Near-identical `node_contains` + LCA algorithms (SIMILAR_TO jaccard 1.0) | verified | Share helpers from engine (or a tiny common module) |
| 5 | redundancy | `host/tests/jinja_examples_test.rs:21` ↔ `scaffold_integration_test.rs:21` | Duplicate `copy_dir_all` (+ matching `repo_root` / `temp_workspace` preamble) | verified | Extract `tests/common` helpers |
| 6 | redundancy | `scripts/scaffold_integration.mjs:83` ↔ `vscode_extension/scripts/smoke.mjs:19` | Duplicate `extractResult` marker parsing | verified | Share helper, or keep intentional standalone smoke |

## Info

| # | Check | File:Line | Finding | Confidence | Fix |
|---|-------|-----------|---------|------------|-----|
| 1 | silent_failure | `recipeRepository.ts:60` | `describeCached` empty `catch` falls back to `findById` without updating `lastError` | suspected | Log or set `lastError` for diagnoseability |
| 2 | dead_symbol | `protocol/mod.rs:10` | Intentional `#[allow(dead_code)]` on `HostCommand` for v1 protocol-compat | verified | Keep until variants are implemented/removed |
| 3 | coverage_gap | `lua_runtime.rs` / `dispatch.rs` / `hostBridge.ts` | Large high-fan-in modules (751 / 524 / 566 LOC); architecture hotspots | advisory | Split only if growth hurts review |
| 4 | doc_drift | codebase-memory index | Graph still lists removed `rust/crates/yaml/` and old flat paths (`registry.rs`, `protocol.rs`) | advisory | `index_repository(mode=moderate)` |

## Cross-checks (not dead)

These looked unused in the graph but are **active**:

| Symbol | Why active |
|--------|------------|
| `protocol_keys::OK`, `RECIPE`, `PREVIEW_TOKEN`, … | Used from dispatch / preview_token / bins (LSP related set) |
| `RecipeRepository.refresh` / `reload` | Similar shapes but call different host commands (`list` vs `reload`) — not redundant |
| `path_basename` / `path_stem` | Related helpers, both used (explorer + templates) |
| `validate.error` / `validate.warning` | Parallel diagnostic builders — intentional |
| Webview client methods (`invokeRecipe`, `apply`, …) | Graph in_degree 0 often false; used from Vue via object methods |
| `#[test]` / cfg-test helpers with in_degree 0 | Expected; not production dead code |

## Exclusions applied

- `vscode_extension/media/` (generated bundle)  
- `export/`, `test/fixtures/`, `.agents/`, `.cursor/skills/`, `.vibe/`, `docs/`  

## Summary

The repo is in **good compiler health** (Clippy + `tsc` clean). Actionable debt is mostly **small dead exports** and **duplicated helpers** (test/script/LCA), not systemic unused production APIs. Prefer LSP + grep over sole reliance on the current codebase-memory index until it is refreshed.

**JSON artifact:** [`.agent-lsp/last-inspection.json`](last-inspection.json)
