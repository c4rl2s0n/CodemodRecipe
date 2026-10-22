# Query, capture, and anchor

```text
source file  →  parse  →  query match  →  capture span  →  patch
```

## Captures in the query

```scheme
(class_definition
  name: (identifier) @className
  body: (class_body
    (function_body (block) @body))
  (#eq? @className "Settings"))
```

## The `capture` field

On the op spec, `capture` picks **which** named match is edited (name without `@`).

## The `anchor` field (insert only)

```text
  …[===== capture span =====]…
     ^                       ^
  anchor: start           anchor: end
```

Full query language: [tree-sitter-queries.md](../../03_queries_and_languages/01_tree_sitter_queries/00_index.md).
