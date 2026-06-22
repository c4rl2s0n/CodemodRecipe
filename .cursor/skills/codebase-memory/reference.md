# Codebase-Memory MCP Playbook

[codebase-memory-mcp](https://github.com/DeusData/codebase-memory-mcp) indexes this repository into a persistent knowledge graph (functions, classes, call chains, HTTP routes, imports). MCP tools query that graph in milliseconds with far fewer tokens than file-by-file exploration.

**Setup:** not documented here. If MCP tools are unavailable, see the [GitHub repository](https://github.com/DeusData/codebase-memory-mcp).

## First-step rule

**Before grep, glob, or reading files to understand structure, use codebase-memory MCP tools.**

Fall back to manual analysis only when:

- MCP is unavailable
- The question is about unindexed paths
- You need exact line-level edit context **after** graph tools have oriented you

## When to use graph tools vs manual analysis

| Task | First step |
|------|------------|
| Where is X defined? / find symbols | `search_graph` |
| Who calls X? / impact / data flow | `trace_path` |
| How is this repo structured? | `get_architecture` |
| Read a known symbol's source | `search_graph` → `get_code_snippet` |
| Complex multi-hop queries | `query_graph` (Cypher) |
| Text in file bodies (after orienting) | `search_code` |
| Uncommitted change blast radius | `detect_changes` |
| First time in a session | `get_graph_schema` |
| Line-level edit / formatting | `read` / `grep` (after graph orientation) |

## Standard workflow

1. `index_status` / `list_projects` — confirm index exists (if not, `index_repository` with absolute repo path)
2. `get_architecture` — orient (packages, clusters, routes)
3. `search_graph` with `query=` (BM25) or `name_pattern=`
4. `trace_path` for call chains (`direction`, `depth`, `mode`)
5. `get_code_snippet` with qualified name from step 3
6. `detect_changes` before large refactors
7. Only then: targeted `read` / `grep` for edit sites

## Project name

**Indexed project name for this repo:** `home-ikusa-workspace-Android-codemod_recipe`

Pass as `project=` on tool calls when results are ambiguous. Discover via `list_projects`.

## Tool cheat sheet

### Indexing

| Tool | Use for |
|------|---------|
| `index_repository` | Index or refresh the repo graph (`mode`: `full`, `moderate`, `fast`) |
| `list_projects` | List indexed projects with node/edge counts |
| `index_status` | Check whether a project index is ready |
| `delete_project` | Remove a project from the graph store |

### Querying

| Tool | Use for |
|------|---------|
| `search_graph` | Find functions, classes, routes by name, BM25 `query=`, or `semantic_query=` |
| `trace_path` | Callers/callees, data flow, cross-service paths (`direction`, `depth`, `mode`) |
| `get_code_snippet` | Read source for a symbol — use `qualified_name` from `search_graph` first |
| `get_architecture` | Packages, clusters, routes, hotspots, layers (`aspects` optional) |
| `get_graph_schema` | Node labels, edge types — run early in a session |
| `query_graph` | Cypher-like read queries for multi-hop patterns |
| `search_code` | Grep-like search within indexed files |
| `detect_changes` | Map git diff to affected symbols and blast radius |

### Advanced

| Tool | Use for |
|------|---------|
| `manage_adr` | Store/retrieve architecture decision records |
| `ingest_traces` | Validate HTTP call edges from runtime traces |

### `search_graph` tips

- `query="recipe compiler"` — BM25 full-text (camelCase-aware)
- `name_pattern=".*Registry.*"` — regex on symbol names
- `label="Function"` — filter by node type
- `file_pattern=".*yaml.*"` — scope to paths
- Paginate with `limit` / `offset` when `has_more` is true

### `trace_path` tips

- `function_name` — short name or from `search_graph` results
- `direction`: `inbound`, `outbound`, or `both`
- `mode`: `calls` (default), `data_flow`, `cross_service`
- `depth`: 1–5 (default 3)

## This-repo examples

Use `project="home-ikusa-workspace-Android-codemod_recipe"` when needed.

```
search_graph(query="recipe compiler")
→ lib/src/yaml/recipe_compiler.dart

search_graph(query="recipe registry")
→ lib/src/yaml/recipe_registry.dart

trace_path(function_name="CodemodHost", direction="both")
→ VS Code host protocol (Dart ↔ TypeScript bridge)

get_architecture(aspects=["packages", "clusters"])
→ module seams across Dart core, VS Code extension, Vue webview
```

## Maintenance

Update this file when primary subsystems or example entry points change materially.
