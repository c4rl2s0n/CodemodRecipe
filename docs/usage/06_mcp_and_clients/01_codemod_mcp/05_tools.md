# Tools

All tools return a JSON string. Parse it, then check `ok`.

| Tool | Purpose |
|------|---------|
| `bootstrap_project` | Install agent skills, rules, `.codemod/` scaffolding |
| `list_recipes` | Discover registered recipe ids and argument schemas |
| `describe_recipe` | Show metadata/args for one recipe |
| `validate_recipes` | Reload + validate Luau recipes/maps/variables/queries |
| `preview_recipe` | Dry-run; returns `previewToken` + transcript |
| `apply_recipe` | Apply atomically; requires `previewToken` |
| `dump_ast` | Dump tree-sitter AST |
| `debug_query` | Run a query; return match roots and captures |
| `generate_query` | Starter query from byte `start`/`end` in a file |
| `resolve_static_path` | Jinja-render a path with empty args (static paths only) |
| `component_scan` | Scan stack root components (instances, findings, summaries, relation `edges`, `kindGraph`); populates cache |
| `list_component_instances` | Alias of `component_scan` |
| `component_coverage` | Span coverage vs `stack.codebase` (per-file + aggregate); skips with warning if `codebase` missing |
| `component_graph` | Render the relations graph as mermaid text; `level`: `kinds` (default) or `instances` |
| `component_incremental_scan` | Rescan after edits: `dirtyPaths`, `detectChanges`, or `force` |
| `describe_component` | Metadata: hooks present, scaffold args, operations |
| `component_dry_run_hook` | Dry-run one hook (Preview; no disk writes); returns `result` + `transcript` |
| `component_apply_hook` | Apply one hook (flushes WorkingTree); same args as dry-run |
| `run_tests` | Run `*.test.luau` suites against temp fixture workspaces; optional `filter` (suite id, `target`, `suite/test`, or test name). Read-only for the real workspace — see [Testing](../../07_testing/00_index.md) |

When `.codemod/stack.luau` and components exist: `component_scan` first, then
`component_incremental_scan` after file edits. Optional: `component_coverage` to audit
blind spots (requires `codebase` on the stack — see [coverage](../../05_components_and_stacks/01_components/09_coverage.md)).
To author/debug hooks, use
`describe_component` and `component_dry_run_hook` (inspect `c:log` in `transcript`).
Authoring agents should be allowlisted for dry-run + scan/describe — withhold
`component_apply_hook` when agents must not write the workspace. Experiment fixture:
[flutter-poc.md](../../../develop/02_flutter_poc/00_index.md). See Builder skill apply checklist..

## Component dry-run (debug)

```json
{ "componentId": "example.entity", "hook": "identify" }
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
