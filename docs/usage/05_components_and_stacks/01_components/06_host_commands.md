# Host commands

| Command | Description |
|---------|-------------|
| `componentScan` | Root scan: identify → parse → verify → optional `toSummary`; populates in-process cache |
| `componentListInstances` | Same as scan |
| `componentIncrementalScan` | Rescan after edits: `dirtyPaths` (tier 1), `detectChanges` (stat + re-identify, tier 2), or `force` (full scan). MCP: `component_incremental_scan` |
| `componentInvokeHook` | Dry-run or apply one hook (`identify` / `parse` / `verify` / `toSummary` / `scaffold` / `operation`); returns `result` + `transcript` (+ `files` when applicable). Host `preview` flag maps to Preview vs Apply |
| `componentScaffold` | Thin wrapper → scaffold via `componentInvokeHook` |
| `componentInvokeOperation` | Thin wrapper → operation via `componentInvokeHook` |
| `describeComponent` | Metadata including `hasIdentify` / `hasToSummary` / `hasScaffold` and operation args |

MCP tools: `component_scan`, `list_component_instances`, `component_incremental_scan`, `describe_component`, `component_dry_run_hook`, `component_apply_hook` (dry-run vs apply are separate tools for agent allowlisting; both call `componentInvokeHook`). See [flutter-poc.md](../../../develop/02_flutter_poc/00_index.md) and [codemod-mcp.md](../../06_mcp_and_clients/01_codemod_mcp/00_index.md).
