# MCP

With `codemod-mcp` configured:

- `component_scan` / `list_component_instances` — full scan (populates cache)
- `component_incremental_scan` — `{ dirtyPaths?, detectChanges?, force? }` (see tiers above)

Point at a workspace that has a stack + components (fixture: `test/fixtures/flutter_poc/workspace` with `.codemod` beside it).
