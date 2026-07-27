# Tree-sitter query language

codemod-recipe uses [tree-sitter](https://tree-sitter.github.io/) queries to match AST nodes before applying insert, replace, or remove patches. Queries are S-expression patterns written in recipe `edit.ops` or external `.scm` files.

**Agent skill:** `.agents/skills/codemod-tree-sitter-queries/` (installed by `bootstrap_project` from `export/.agents/skills/codemod-tree-sitter-queries/`).

## Quick start

1. Write a pattern matching the target node: `(node_type field: (child) @capture)`.
2. Filter with predicates: `(#eq? @capture "{{arg}}")`.
3. Set `capture:` on the op to the `@name` you want to edit.
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
| External query | `query: path/to/file.scm` | Resolved under recipe or `.codemod/queries/` |
| Args in queries | `{{className}}` | Expanded before tree-sitter parses the query |
| Edit guards | `edit.when`, `edit.whenNot` | Same query specs as ops; evaluated once before the edit (skip step if guards fail) |
| Step locals | `edit.let[].query` | Per-op bindings; `capture` + `extract` (`text`, `kind`, `exists`, `count`) feed Jinja in later ops |

## Edit-level `when` / `let`

Use tree-sitter queries on the **current file text** to gate an entire `edit` step or to bind locals that change between sequential ops. Guard queries use the same composition rules as op `query` (inline, `.scm`, library refs, chains). See `codemod-yaml-dsl` `reference.md` for YAML shape and template filters on `let.as`.

## Two kinds of “anchor”

- **Tree-sitter query anchor** (`.`) — constrains sibling/first/last child position in the pattern. See [official operators docs](https://tree-sitter.github.io/tree-sitter/using-parsers/queries/2-operators.html).
- **codemod insert anchor** (`anchor: start|end`) — where `text` is inserted relative to the captured node span.

## Grammar node names

Node type names are **per grammar**. Dart pack grammar uses `class_definition`, not `class_declaration`. See [language-support.md](language-support.md) and skill `codemod-languages`.

## Workflow

1. Inspect AST (tree-sitter playground) for node kinds.
2. `validate_recipes` → `preview_recipe` → inspect patches.
3. Tighten predicates if preview is empty or matches multiple nodes.

## Further reading

- [export/.agents/skills/codemod-tree-sitter-queries/reference.md](../export/.agents/skills/codemod-tree-sitter-queries/reference.md) — full agent reference
- [tree-sitter query syntax](https://tree-sitter.github.io/tree-sitter/using-parsers/queries/1-syntax.html) — official documentation
- [docs/codemod-mcp.md](codemod-mcp.md) — MCP preview/apply workflow
- [docs/language-support.md](language-support.md) — multi-language grammars
