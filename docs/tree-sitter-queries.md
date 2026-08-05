# Tree-sitter query language

codemod-recipe uses [tree-sitter](https://tree-sitter.github.io/) queries to match AST nodes before applying insert, replace, or remove patches. Queries are S-expression patterns written in recipe `edit.ops` or external `.scm` files.

**How this fits recipe authoring:** [writing-recipes.md](writing-recipes.md) (workflow, `capture` / `anchor` mental model).

**Agent skill:** `.agents/skills/codemod-tree-sitter-queries/` (installed by `bootstrap_project` from `export/.agents/skills/codemod-tree-sitter-queries/`).

## Quick start

1. Write a pattern matching the target node: `(node_type field: (child) @capture)`.
2. Filter with predicates: `(#eq? @capture "{{arg}}")`.
3. Set `capture:` on the op to the `@name` you want to edit (omit the `@`).
4. For `insert`, set `anchor: start` or `end` on the capture span.

```yaml
- edit:
    path: "lib/settings.dart"
    ops:
      - insert:
          query: |
            (class_definition
              name: (identifier) @className
              body: (class_body
                (function_body
                  (block) @body))
              (#eq? @className "Settings"))
          capture: body
          anchor: end
          text: "    print('codemod');\n"
```

## Key concepts

| Concept | Recipe field / syntax | Notes |
|---------|----------------------|-------|
| Pattern | `query:` | Node types and children from the grammar for `edit.language` |
| Capture | `@name` in query, `capture:` on op | One edit target per op |
| Filter | `#eq?`, `#match?`, `#any-of?` | Predicates inside the pattern |
| Insert position | `anchor: start\|end` | Byte offset in capture span — **not** tree-sitter `.` anchor |
| External query | `query: path/to/file.scm` | Shared path resolver: recipe-local first, `.codemod/` fallback; bare names also check `queries/` under each root |
| Args in queries | `{{className}}` | Expanded before tree-sitter parses the query |
| Edit guards | `edit.when`, `edit.whenNot` | Same query specs as ops; evaluated once before the edit (skip step if guards fail) |
| Step locals | `edit.let[].query` | Per-op bindings; `capture` + `extract` (`text`, `kind`, `exists`, `count`) feed Jinja in later ops |

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

## Edit-level `when` / `let`

Use tree-sitter queries on the **current file text** to gate an entire `edit` step or to bind locals that change between sequential ops. Guard queries use the same composition rules as op `query` (inline, `.scm`, library refs, chains). See `codemod-yaml-dsl` `reference.md` for YAML shape and template filters on `let.as`.

## Two kinds of “anchor”

- **Tree-sitter query anchor** (`.`) — constrains sibling/first/last child position in the pattern. See [official operators docs](https://tree-sitter.github.io/tree-sitter/using-parsers/queries/2-operators.html).
- **codemod insert anchor** (`anchor: start|end`) — where `text` is inserted relative to the captured node span (diagram above).

## Grammar node names

Node type names are **per grammar**. Dart pack grammar uses `class_definition`, not `class_declaration`. See [language-support.md](language-support.md) and skill `codemod-languages`.

## Workflow

1. Inspect AST with **Query Tools** (VS Code Codemod sidebar → Query AST / Query Tools), or a tree-sitter playground, for node kinds.
2. `validate_recipes` → `preview_recipe` → inspect patches (preview may show a full-file replace; use Query Tools to see capture spans).
3. Tighten predicates if preview is empty or matches multiple nodes.

## Query Tools (VS Code)

Bidirectional helper (read-only; separate from recipe apply):

| Direction | Action |
|-----------|--------|
| **Build** | Click an AST node (or Generate from cursor) → starter `query` |
| **Match** | Paste/Run a query → decorations for root, layers, `@captures`, insert anchor |

CodeLens on recipe `query:` → **Open in Query Tools** (loads query; opens `edit.path` when it Jinja-resolves with empty args). **Go to edit path** on static `path:` values.

MCP: `dump_ast`, `debug_query`, `generate_query`, `resolve_static_path`.

Recipe ops still require an explicit `capture:` — Query Tools may default the UI capture to the last `@name` for visualization only.

## Query file resolution

External `.scm` files use the shared resolver in
`rust/crates/core/src/resource_path.rs`.

- Exact path lookup: referencing recipe directory first, `.codemod/` fallback.
- Bare file names also try `queries/` under each root as a query-specific
  convention.
- Query-library YAML stays registry-loaded and id-based
  (`.codemod/queries/*.yaml` + `library_id.query_key`), not direct
  `query: path/to/file.yaml`.

## Further reading

- [writing-recipes.md](writing-recipes.md) — end-to-end recipe authoring
- [generated/dsl-vocabulary.md](generated/dsl-vocabulary.md) — every DSL field
- [export/.agents/skills/codemod-tree-sitter-queries/reference.md](../export/.agents/skills/codemod-tree-sitter-queries/reference.md) — full agent reference
- [tree-sitter query syntax](https://tree-sitter.github.io/tree-sitter/using-parsers/queries/1-syntax.html) — official documentation
- [docs/codemod-mcp.md](codemod-mcp.md) — MCP preview/apply workflow
- [docs/language-support.md](language-support.md) — multi-language grammars
