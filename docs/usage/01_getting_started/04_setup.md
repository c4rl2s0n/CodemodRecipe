# Setup

## Prerequisites

- A target workspace (the project you want to edit).
- [Rust](https://rustup.rs/) (stable) if you build the host or MCP from source.
- [Node.js](https://nodejs.org/) 18+ to build the VS Code extension.

## Workspace layout

In the target project:

```text
.codemod/
  recipes/     # Luau recipe modules (*.luau or *.lua)
  maps/        # Luau map modules (id + map table)
  types/       # Named arg types (id + fields)
  variables/   # Luau variable modules (id + values table)
  queries/     # Luau query libraries (id + queries table)
  templates/   # optional (c:template path)
```

Discovery scans `.codemod/` for Luau modules that `return` the expected table shapes.

File-backed resources (`.scm` queries, template files, `postExecution` scripts)
resolve recipe-local first, then under `.codemod/`.

## VS Code extension

1. From this repo: `./build.sh` (regenerates host API stubs, release-builds
   `codemod_host` and `codemod_mcp`, packages the VSIX). For stub-only refresh
   after host API edits: `scripts/generate-host-api-surface.sh`.
2. Install the VSIX and open a workspace that has (or will have) `.codemod/`.
3. Use **Codemod Recipe: Scaffold Project** if `.codemod` is missing.
   After upgrading the extension, use **Codemod Recipe: Update Project
   Scaffold** to refresh stubs and other scaffold files (overwrites).
4. Open the **Codemod Recipe** activity bar: Recipes + Recipe Runner.

Details and settings: [vscode_extension/README.md](../../../vscode_extension/README.md).

**Recipe editor IntelliSense:** under `.codemod/**/*.luau` the extension
completes `c:` methods, spec keys, recipe/map/var/query ids (`c:map` /
`c:var` / `c:queryLib` / `c:run`), languages, captures, and `args.*` from
the recipe source; CodeLens on `id = "…"`; go-to-definition on `c:run("…")`.
Install **Luau Language Server** (`JohnnyMorganz.luau-lsp`) and point it at
`.codemod/stubs/codemod-context.d.luau` (via scaffolded
`luau-lsp.types.definitionFiles`) for typed `CodemodContext` on `apply(c, args)`.
Helper modules under `.codemod/` with **no** top-level `id` are libraries you can
`require` at runtime; `.d.luau` stubs stay IDE-only — see [luau-lsp.md](../05_components_and_stacks/04_luau_lsp/00_index.md).
If that stub is stale after upgrading the extension/host API, run
**Codemod Recipe: Update Project Scaffold** (or MCP `bootstrap_project` with
`force`) — soft Scaffold skips existing files. **Refresh Recipes** only
reloads the catalog; it does not rewrite stubs.

**Components and Luau types:** the platform layer uses the same `c` host context in
component hooks (`identify`, `parse`, `verify`, …). Stack roots return locators from
`identify(c)` (often via `c:listFiles` / `c:matchFiles`); `parse` yields a **node**;
ops receive `target = { locator, node, … }`. Query hits use
`{ span = { from, to }, text, kind }` (`c:queryMatches` for multi-capture rows).
See [components.md](../05_components_and_stacks/01_components/00_index.md), [component-contract.md](../05_components_and_stacks/03_component_contract/00_index.md),
and [luau-lsp.md](../05_components_and_stacks/04_luau_lsp/00_index.md).

**Platform experiment (MCP):** when a workspace has `.codemod/stack.luau` and
components, call `component_scan` (or `list_component_instances`) for instances /
findings / summaries, then `component_incremental_scan` after edits
(`dirtyPaths` when known, or `detectChanges: true`). Create with host
`componentScaffold`; mutate with `componentInvokeOperation` (not MCP tools yet).
Try the in-repo fixture: [flutter-poc.md](../../develop/02_flutter_poc/00_index.md)
(`test/fixtures/flutter_poc/`).

**Review panel:** after preview, inspect **Files** (diffs) and **Transcript**
(every `c:` call, including postExec entries skipped on preview). Edit rows
show `+N`/`-N` line badges when available; click a row to expand payloads or
pretty-print JSON logs. When apply may differ from preview because postExec
was skipped, a small **applyMayDiverge** disclaimer appears. The Runner also
shows recipe metadata (`explorerMenu`, context args, `postExecution`, shortcut
slots) when present.

**Query Tools** (same Codemod activity bar: **Query AST** + **Query Tools**):

1. Open a source file, then browse or click an AST node to generate a starter query.
2. Paste or edit the query, set `capture` / `anchor`, and **Run** to highlight matches.
3. Use Query Tools to debug queries before embedding them in recipe Luau.

Walkthrough: [tree-sitter-queries.md — Query Tools](../03_queries_and_languages/01_tree_sitter_queries/00_index.md).

## MCP

1. Build/run `codemod_mcp` pointing at your workspace root and `.codemod`.
2. Configure Cursor (or another MCP client) — see [codemod-mcp.md](../06_mcp_and_clients/01_codemod_mcp/00_index.md).
3. Call `bootstrap_project` once to install agent skills/rules and scaffolding.
4. End-to-end new-project checklist:
   [new-project-rust-mcp.md](../06_mcp_and_clients/02_new_project_rust_mcp/00_index.md).

Query debugging without the extension: MCP `dump_ast`, `debug_query`,
`generate_query`. Overview: [tree-sitter-queries.md](../03_queries_and_languages/01_tree_sitter_queries/00_index.md).
