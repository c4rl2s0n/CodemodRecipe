# Tree-sitter query language

codemod-recipe uses [tree-sitter](https://tree-sitter.github.io/) queries to match AST nodes before applying insert, replace, or remove patches. Queries are S-expression patterns passed to `c:insert` / `c:replace` / `c:remove` or external `.scm` files.

**How this fits recipe authoring:** [writing-recipes.md](writing-recipes.md) (workflow, `capture` / `anchor` mental model).

**Agent skill:** `.agents/skills/codemod-tree-sitter-queries/` (installed by `bootstrap_project` from `export/.agents/skills/codemod-tree-sitter-queries/`).

## Quick start

1. Write a pattern matching the target node: `(node_type field: (child) @capture)`.
2. Filter with predicates: `(#eq? @capture "{{arg}}")`.
3. Set `capture` on the op spec to the `@name` you want to edit (omit the `@`).
4. For `c:insert`, set `anchor = "start"` or `"end"` on the capture span.

```lua
c:insert({
  path = "lib/settings.dart",
  query = [[
    (class_definition
      name: (identifier) @className
      body: (class_body
        (function_body
          (block) @body))
      (#eq? @className "Settings"))
  ]],
  capture = "body",
  anchor = "end",
  text = "    print('codemod');\n",
})
```

## Key concepts

| Concept | Op spec / syntax | Notes |
|---------|------------------|-------|
| Pattern | `query` | Node types from the grammar for `language` |
| Capture | `@name` in query, `capture` on op | One edit target per op |
| Filter | `#eq?`, `#match?`, `#any-of?` | Predicates inside the pattern |
| Insert position | `anchor = "start"\|"end"` | Byte offset in capture span |
| External query | `query = "path/to/file.scm"` | Recipe-local first, `.codemod/` fallback |
| Args in queries | string concat or `c:applyTemplate` | Build query text in Luau |
| Idempotency guards | `c:queryExists`, `c:queryCount` | Luau control flow before mutating |

## Insert `anchor` (codemod)

Only `insert` uses `anchor`. It picks the **byte** edge of the node named by `capture:`:

```text
  …[===== capture span =====]…
     ^                       ^
  anchor: start           anchor: end
```

| Goal | `anchor` |
|------|----------|
| Insert before the captured node’s text | `start` |
| Insert after the captured node’s text | `end` |

`replace` and `remove` always act on the whole capture span (no `anchor`).

## Idempotency and guards

Use `c:queryExists`, `c:queryCount`, or `c:query` inside `apply` before mutating.
See skill `codemod-luau-recipes` for the host API.

## Two kinds of “anchor”

- **Tree-sitter query anchor** (`.`) — constrains sibling/first/last child position in the pattern. See [official operators docs](https://tree-sitter.github.io/tree-sitter/using-parsers/queries/2-operators.html).
- **codemod insert anchor** (`anchor: start|end`) — where `text` is inserted relative to the captured node span (diagram above).

## Grammar node names

Node type names are **per grammar**. Dart pack grammar uses `class_definition`, not `class_declaration`. See [language-support.md](language-support.md) and skill `codemod-languages`.

## Workflow

1. Inspect AST with **Query Tools** (VS Code Codemod sidebar → Query AST / Query Tools), or a tree-sitter playground, for node kinds.
2. `validate_recipes` → `preview_recipe` → inspect files (preview shows original vs modified; use Query Tools to see capture spans).
3. Tighten predicates if preview is empty or matches multiple nodes.

## Query Tools (VS Code)

Bidirectional helper (read-only; separate from recipe apply):

| Direction | Action |
|-----------|--------|
| **Build** | Click an AST node (or Generate from cursor) → starter `query` |
| **Match** | Paste/Run a query → decorations for root, layers, `@captures`, insert anchor |

CodeLens on recipe `query:` → **Open in Query Tools** (loads query; opens `edit.path` when it Jinja-resolves with empty args). **Go to edit path** on static `path:` values.

**AST refresh:** the Query AST tree updates when the view is visible and you switch files, save, or explicitly reveal — not on every keystroke. While the buffer is dirty, the tree shows a prominent **Stale (unsaved)** warning until you save. **Run** / **Generate** always use the current editor text; the Rust host caches parses by path + content hash (LRU, shared across Query Tools commands). Closing Query AST clears editor highlights.

**Cursor follow:** with Query AST open, moving the cursor reveals the deepest node and regenerates the Query Tools query (same as clicking an AST node). Close Query AST to edit a pasted query without overwrite.

**Generate helpers:** **Pin text (#eq?)** adds `(#eq? @capture "literal")` when generating (and regenerates when you toggle it). Without Pin, last list items get tree-sitter last-child `.`. **Layer highlights** (default on) injects temporary `@__layer_*` for depth coloring on Run; patterns that use `.` skip layer injection so queries stay valid. Turn Layer highlights off if needed.

**AST display:** view title → **Query AST Display Mode…** (`kind` / `content` / `kindPreview`). Default is kind + muted text preview (≤30 chars). Setting: `codemodRecipe.queryTools.astDisplayMode`.

MCP: `dump_ast`, `debug_query`, `generate_query`, `resolve_static_path`.

Recipe ops still require an explicit `capture:` — Query Tools may default the UI capture to the last `@name` for visualization only.

## Query file resolution

External `.scm` files use the shared resolver in
`rust/crates/core/src/resource_path.rs`.

- Exact path lookup: referencing recipe directory first, `.codemod/` fallback.
- Bare file names also try `queries/` under each root as a query-specific
  convention.
- Query libraries are Luau modules under `.codemod/queries/*.luau`
  (`c:queryLib("id")` or query id strings resolved via the registry).

## Further reading

- [writing-recipes.md](writing-recipes.md) — end-to-end recipe authoring
- skill `codemod-luau-recipes` — host API reference
- [export/.agents/skills/codemod-tree-sitter-queries/reference.md](../export/.agents/skills/codemod-tree-sitter-queries/reference.md) — full agent reference
- [tree-sitter query syntax](https://tree-sitter.github.io/tree-sitter/using-parsers/queries/1-syntax.html) — official documentation
- [docs/codemod-mcp.md](codemod-mcp.md) — MCP preview/apply workflow
- [docs/language-support.md](language-support.md) — multi-language grammars
