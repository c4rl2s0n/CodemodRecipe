# Overview

A query is one or more patterns. Each pattern is `(node_type child1 child2 …)` where children are nested patterns, captures, predicates, or operators.

```text
source file  →  tree-sitter parse  →  syntax tree  →  query match  →  capture span  →  codemod patch
```

Official chapters:

| Topic | URL |
|-------|-----|
| Basic syntax | https://tree-sitter.github.io/tree-sitter/using-parsers/queries/1-syntax.html |
| Operators | https://tree-sitter.github.io/tree-sitter/using-parsers/queries/2-operators.html |
| Predicates | https://tree-sitter.github.io/tree-sitter/using-parsers/queries/3-predicates-and-directives.html |

---
