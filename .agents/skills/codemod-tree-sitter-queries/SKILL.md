---
name: codemod-tree-sitter-queries
description: Tree-sitter query language for codemod-recipe — S-expression patterns, captures, predicates, operators, and codemod-specific capture/anchor semantics. Read reference.md for the full guide.
---

# Tree-sitter queries

## When to use

- Writing or debugging `query:` strings in `insert` / `replace` / `remove` ops
- Understanding `@captures`, `#eq?`, wildcards, quantifiers, or query anchors `.`
- Distinguishing tree-sitter query anchors from codemod `anchor: start|end`
- Moving queries into external `.scm` files
- Fixing `Invalid node type`, `query matched no nodes`, or `query matched multiple nodes`
- Using VS Code **Query Tools** or MCP `dump_ast` / `debug_query` / `generate_query`

## When not to use

- YAML recipe structure or templates → `codemod-yaml-dsl`
- Which `language:` id or grammar to pick → `codemod-languages`
- Dart-specific recipe patterns and testing workflow → `codemod-recipe-authoring`
- Running preview/apply → `codemod-mcp-playbook`

## Minimal pattern

```yaml
query: |
  (class_definition
    name: (identifier) @className
    body: (class_body
      (function_body
        (block) @body))
    (#eq? @className "{{className}}"))
capture: body
anchor: end   # codemod insert anchor — not the tree-sitter `.` operator
text: "    print('codemod');\n"
```

1. Parentheses match a node type (`class_definition`).
2. `field:` names constrain children (`name:`, `body:`).
3. `@name` captures a node for predicates or for the edit target.
4. `(#eq? @capture "literal")` filters matches.
5. `capture:` on the op selects which `@name` span is edited.

## Quick rules

1. Node type names come from the **grammar** for `edit.language` — see `codemod-languages`.
2. Each op edits **one** capture; tighten predicates until exactly one match.
3. `{{args}}` are expanded into `query` before tree-sitter parses it.
4. `query:` may be inline text or a path to a `.scm` file resolved by the shared safe resolver in `rust/crates/core/src/resource_path.rs`: recipe-local first, then `.codemod/`, with bare names also trying `queries/` under each root. YAML query libraries remain id-based under `.codemod/queries/*.yaml`.

## Instructions

1. **Read and follow** [reference.md](reference.md) for syntax, predicates, operators, and codemod integration.
2. Inspect the target AST (tree-sitter playground) before guessing node names.
3. `validate_recipes` → `preview_recipe` → tighten `#eq?` / `#match?` if matches are empty or ambiguous.

## Related skills

| Skill | Use for |
|-------|---------|
| `codemod-languages` | language ids, grammar node names |
| `codemod-recipe-authoring` | Dart patterns, idempotency, testing |
| `codemod-yaml-dsl` | `edit.ops` field shape |
| `codemod-mcp-playbook` | preview/apply workflow |

Official tree-sitter query docs: https://tree-sitter.github.io/tree-sitter/using-parsers/queries/1-syntax.html
