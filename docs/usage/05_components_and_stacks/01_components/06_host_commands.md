# Host commands

| Command | Description |
|---------|-------------|
| `componentScan` | Root scan: identify → parse (all roots) → relate → verify over the whole forest, with `toSummary` per root; populates in-process cache. Response includes `edges` + `kindGraph` (see [relations](10_relations.md)) |
| `componentListInstances` | Same as scan |
| `componentCoverage` | Post-scan span coverage vs `stack.codebase` (per-file + aggregate); skips with warning if `codebase` missing. MCP: `component_coverage` |
| `componentGraph` | Render the relations graph as mermaid text; `level`: `kinds` (default, roll-up with `×count`) or `instances` (all nodes + symbol leaves, response includes `edges`). Scan-on-demand. MCP: `component_graph` |
| `componentIncrementalScan` | Rescan after edits: `dirtyPaths` (tier 1), `detectChanges` (stat + re-identify, tier 2), or `force` (full scan). `relate`/`verify` re-run over the merged forest. MCP: `component_incremental_scan` |
| `componentInvokeHook` | Dry-run or apply one hook (`identify` / `parse` / `relate` / `verify` / `toSummary` / `scaffold` / `operation`); returns `result` + `transcript` (+ `files` when applicable). Host `preview` flag maps to Preview vs Apply. `relate`/`verify` dry-runs build their facade from the cached scan — triggering a full scan first when no cache exists |
| `componentScaffold` | Thin wrapper → scaffold via `componentInvokeHook` |
| `componentInvokeOperation` | Thin wrapper → operation via `componentInvokeHook`; operation `apply` receives the pre-operation graph as 4th arg |
| `describeComponent` | Metadata including `hasIdentify` / `hasVerify` / `hasRelate` / `hasToSummary` / `hasScaffold` and operation args |

MCP tools: `component_scan`, `list_component_instances`, `component_coverage`, `component_graph`, `component_incremental_scan`, `describe_component`, `component_dry_run_hook`, `component_apply_hook` (dry-run vs apply are separate tools for agent allowlisting; both call `componentInvokeHook`). See [flutter-poc.md](../../../develop/02_flutter_poc/00_index.md), [coverage](09_coverage.md), and [codemod-mcp.md](../../06_mcp_and_clients/01_codemod_mcp/00_index.md).
