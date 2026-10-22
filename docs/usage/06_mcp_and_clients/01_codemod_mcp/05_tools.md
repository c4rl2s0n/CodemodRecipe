# Tools

All tools return a JSON string. Parse it, then check `ok`.

| Tool | Purpose |
|------|---------|
| `bootstrap_project` | Install agent skills, rules, `.codemod/` scaffolding |
| `list_recipes` | Discover registered recipe ids and argument schemas |
| `describe_recipe` | Show metadata/args for one recipe |
| `validate_recipes` | Reload + validate Luau recipes/maps/variables/queries |
| `reload_recipes` | Reload catalog from disk after editing `.codemod` assets |
| `preview_recipe` | Dry-run; returns `previewToken`, `transcript`, top-level `logs[]` (`c:log`) |
| `diff_recipe` | Single-path dry-run hunk (`path` required; same args as preview) |
| `apply_recipe` | Apply atomically; requires `previewToken` |
| `derive_args` | Derive recipe args from `source` + cursor/selection (+ optional `context`) |
| `filter_explorer_recipes` | Recipes whose `explorerMenu` matches a path (`kind`: `file` \| `folder`) |
| `dump_ast` | Dump tree-sitter AST — **call this before writing queries**; `namedOnly: false` to see keywords |
| `debug_query` | Run a query; return match roots and captures; `instrument` (default true) adds `@__layer_*` depth markers |
| `generate_query` | Starter query from byte `start`/`end` in a file |
| `query_captures` | List `@capture` names from inline `query` or registered `queryId` |
| `resolve_static_path` | Jinja-render a path with empty args (static paths only) |
| `component_scan` | Scan stack root components (instances, findings, summaries, relation `edges`, `kindGraph`); populates cache |
| `list_component_instances` | Alias of `component_scan` (same HostCommand family; prefer `component_scan`) |
| `component_coverage` | Span coverage vs `stack.codebase` (per-file + aggregate); skips with warning if `codebase` missing |
| `component_graph` | Render the relations graph as mermaid text; `level`: `kinds` (default) or `instances` |
| `component_snapshot` | Write deterministic scan snapshot JSON under the codemod root (default `.cache/scan-snapshot.json`) |
| `component_incremental_scan` | Rescan after edits: `dirtyPaths`, `detectChanges`, or `force` |
| `describe_component` | Metadata: hooks present, scaffold args, operations |
| `component_dry_run_hook` | Dry-run one hook; returns `result`, `transcript`, `logs[]`, **`previewToken`** — **scan has no logs** |
| `component_apply_hook` | Apply one hook (flushes WorkingTree); requires **`previewToken`** from dry-run with the same args |
| `stack_dry_run_operation` | Dry-run a stack-level op (e.g. `createCommand`); returns `previewToken` |
| `stack_apply_operation` | Apply a stack-level op; requires **`previewToken`** from dry-run |
| `run_tests` | Run `*.test.luau` suites against temp fixture workspaces; optional `filter` (suite id, `target`, `suite/test`, or test name). Read-only for the real workspace — see [Testing](../../07_testing/00_index.md) |

When `.codemod/stack.luau` and components exist: `component_scan` first, then
`component_incremental_scan` after file edits. Optional: `component_coverage` to audit
blind spots (requires `codebase` on the stack — see [coverage](../../05_components_and_stacks/01_components/09_coverage.md)).
To author/debug hooks, use
`describe_component` and `component_dry_run_hook` (read `logs` / `transcript` for `c:log`).
Authoring agents should be allowlisted for dry-run + scan/describe — withhold
`component_apply_hook` / `stack_apply_operation` when agents must not write the workspace.
Preview vs apply tokens: [07_preview_vs_apply.md](07_preview_vs_apply.md).
Experiment fixture: [flutter-poc.md](../../../develop/02_flutter_poc/00_index.md). See Builder skill apply checklist.

## Query Tools (MCP)

These replace the VS Code Query Tools panel for MCP-only agents:

1. **`dump_ast`** — `{ "path": "lib/foo.dart", "namedOnly": false }` before writing any query. Confirm node types, siblings vs children, anonymous tokens.
2. **`debug_query`** — `{ "path": "…", "query": "(…)", "instrument": true }`. Set `instrument: false` only if debugging instrumentation itself.
3. **`generate_query`** — starter pattern from a byte range.
4. **`query_captures`** — `{ "query": "(…)" }` or `{ "queryId": "lib.name" }` for declared `@capture` names.

Authoring workflow: [tree-sitter authoring](../../03_queries_and_languages/01_tree_sitter_queries/07_authoring_workflow.md).

## Component dry-run (debug)

```json
{ "componentId": "example.entity", "hook": "identify" }
```

Example success fragment (note top-level `logs`):

```json
{
  "ok": true,
  "result": [{ "path": "lib/x.dart", "name": "x" }],
  "logs": ["identify-start"],
  "transcript": [{ "kind": "log", "detail": "identify-start", "includedInPreview": true }]
}
```

```json
{
  "componentId": "example.entity",
  "hook": "parse",
  "locator": { "path": "src/entities/recipe.dart", "name": "recipe" }
}
```

`hook` is one of: `identify`, `parse`, `relate`, `verify`, `toSummary`, `scaffold`, `operation`
(for `operation`, also pass `operation` name + `instance` with `id`; for `relate`/`verify`,
pass `instance` with `id` — the facade is built from the cached scan, and when no cache
exists the dry-run triggers a full scan first, populating the cache).

Scan phase order is `identify → parse → relate → verify`; scan responses include the
resolved relation `edges` and the aggregated `kindGraph` — see
[relations](../../05_components_and_stacks/01_components/10_relations.md). Render that
graph as mermaid via `component_graph` (`level`: `kinds` default, `instances` for the
full node set).
