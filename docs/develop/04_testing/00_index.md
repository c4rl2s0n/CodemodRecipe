# Testing (design record)

Fixture-based test layer for `.codemod` assets: `*.test.luau` suites covering
recipes, components, and stack-level relations, run by the host
(`rust/crates/host/src/testing.rs`) and exposed as the `run_tests` MCP tool. Usage
docs live under `docs/usage/07_testing/`.

## Decisions (from design discussion, 2026-09-29)

- **Placement**: separate `*.test.luau` modules, colocated by naming convention
  (`components/flutter.model.test.luau`, `stack.test.luau`, `recipes/<name>.test.luau`).
  New `AssetKind::TestSuite`, classified by content (`id` + `tests` table of
  functions). Rationale: production hooks re-eval module source on every call
  (`runtime.rs` `eval_return_table`), so inline test code would couple test syntax
  errors to production `parse`/`apply`; separate files isolate failures and avoid
  clutter. No inline `tests` key sugar in v1.
- **Semantics**: tests fail by raising — plain Luau `assert(cond, msg)` /
  `error(msg)`. No bool, no string-as-failure. The runner catches per-test and
  continues; `load.rs` `map_lua_error`/`parse_lua_location`-style line extraction
  feeds the report (`parse_error_line` in `testing.rs`). A test passes by returning
  normally.
- **Scope**: all three levels — recipe tests, component tests, stack/relation
  integration tests — distinguished by what the suite exercises, not by module shape.
- **Fixtures**: hybrid, always materialized to a fresh temp dir per test (fixtures
  stay pristine against apply-mode tests, parallel runs, and scan-cache path
  bookkeeping; same `temp_workspace` pattern as `tests/common/mod.rs`). Three forms:
  - `t:workspace("fixtures/empty_app")` — copy an on-disk dir under `.codemod/tests/`.
  - `t:workspace({ files = { ... } })` — inline tree only.
  - `t:workspace({ base = "...", files = { ... } })` — copy base, inline files
    override/add (the fixture-variation pattern).
  - `t:workspace({})` is a deliberate empty workspace (scaffold tests).
- **Scan is explicit**: `t:workspace` only materializes + binds; `ws:scan()` is a
  separate call so unit-style `ws:parse` stays cheap and scan → apply → rescan
  sequences read clearly. Exception: `ws:verify` / `ws:relate` lazily scan on demand
  (same semantics as the existing dry-run path in `invoke_read_hook`).
- **Conceptual split**: `verify` remains the continuous judgment of the real
  workspace; `tests` are fixture-based regression tests of hook logic.

## Implementation notes

- Loader: `TestSuiteDef` + `AssetKind::TestSuite` in `rust/crates/lua`; the `tests`
  arm classifies before the `Unknown` fallthrough (a table with `tests` but no
  `apply`/`parse`/`components`/... is a suite). All `tests` values must be functions;
  ≥1 test required; names sorted.
- Registry: `test_suites: BTreeMap<String, TestSuiteDef>` with the usual `E_DUP_ID`
  diagnostic; `collect_lua_files` excludes `.codemod/tests/fixtures/` so fixture
  trees (which may contain their own `.codemod`) are never loaded as assets.
- Runner: per test — fresh sandbox (`new_sandbox_with_require`, so suites can
  `require` library helpers), fresh module eval, fresh temp workspace. The fixture
  registry is `RecipeRegistry::new(temp_root, real_codemod_root)` with mirrored
  language config — tests exercise the workspace's real `.codemod` assets against
  fixture files.
- `t` is built from a Luau bootstrap chunk (`TEST_CONTEXT_LUA`) so
  `error(msg, 2)` attributes failures to the calling test line; the only Rust
  primitive is `prims.workspace(spec)`. `ws` methods are thin Rust closures over the
  production scan/hook/operation/recipe machinery.
- Mutations (`ws:apply`, `ws:scaffold`, `ws:applyRecipe`) run in `RunKind::Apply`
  against the temp dir only, so rescan/`readFile` observe them; recipe postExec
  scripts are collected (`sawPostExec`) but never executed.
- MCP: `run_tests` is read-only for the real workspace (Preview kind + temp dirs) —
  allowlist-friendly.

## Framework bugs found via dogfooding

- Empty `identify` returning `{}` produced a phantom `component/unknown` instance:
  `table_to_json` maps the empty Luau table to a JSON object, which
  `collect_identify_locators` treated as one locator `{}`. Fixed in `scan.rs`
  (empty object → no locators); same latent case fixed in
  `relations.rs::normalize_raw_edges` (empty object → no edges).

## Out of scope (v1)

VS Code extension UI (test runner view), snapshot/golden-file assertions, inline
`tests` key sugar in production modules.
