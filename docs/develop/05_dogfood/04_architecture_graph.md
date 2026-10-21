# Architecture graph

How VS Code commands, host-bridge methods, MCP tools, and `HostCommand` variants
connect. **Machine SSOT is under `.codemod/.cache/`** — do not treat historical
mermaid embeds in git history as current.

## Machine SSOT (read these)

| Artifact | Role |
|----------|------|
| [`.codemod/.cache/kind-graph.mmd`](../../../.codemod/.cache/kind-graph.mmd) | Kind-level architecture roll-up |
| [`.codemod/.cache/instance-graph.mmd`](../../../.codemod/.cache/instance-graph.mmd) | Complete instance map |
| [`.codemod/.cache/scan-snapshot.json`](../../../.codemod/.cache/scan-snapshot.json) | Instances, edges, coverage, relations, findings |

Optional committed copies under this folder (`_kinds.mmd` / `_instances.mmd`) are
convenience mirrors only — regenerate from cache after meaningful graph changes.

Policy companion: [Target architecture](05_target_architecture.md) (adapter
matrix H/B/M/V). Decisions: [03_decisions.md](03_decisions.md).

## How to read the kind graph

- **Command triangle:** vscode **invokes** (one-hop) bridge → **sends** HostCommand;
  MCP **exposes** the same commands; dispatch **handles** them. Many UI COMMANDS
  are multi-hop and only `presents` — see adapter matrix policy.
- **Domain internals (D-025):** `host_api_handlers` / `host_module` **drive** /
  **use** `engine_edit` and **stage_via** `working_tree`; `lua_load_module`
  **feeds** `recipe_registry`.

There is no junction `repo.command` node — the command concept lives in these
edges plus stack `createCommand` fan-out (see [Decisions](03_decisions.md)).

## Snapshot layout

`component_scan` / `component_snapshot` refresh the three cache artifacts
(content-stable). Snapshot JSON top-level order: `meta`, `coverage`,
`relations`, then instances/edges/…. Live `component_coverage` still returns
full per-file `gaps` for the UI. Scan responses also include a compact
`summary` (architecture ratio, dead composers, top product gaps).

## Regenerating

From a workspace with `.cursor/mcp.json` pointing at `codemod_mcp` (or drive
the same JSON-RPC over stdio):

1. `component_scan` (refreshes `{codemod_root}/.cache/scan-snapshot.json`,
   `kind-graph.mmd`, and `instance-graph.mmd`)
2. Optionally copy `.cache/*.mmd` into this folder for offline mirrors
3. Or `component_graph` with `out:` under `docs/develop/05_dogfood/` if you want
   workspace-relative exports without reading the cache

`component_graph` `out` is workspace-relative. Snapshot `path` (optional) is
codemod-root-relative; both reject `..` traversal outside their root.
