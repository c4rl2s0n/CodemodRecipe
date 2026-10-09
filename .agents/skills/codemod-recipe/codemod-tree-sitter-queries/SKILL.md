---
name: codemod-tree-sitter-queries
description: >-
  Tree-sitter query language for codemod-recipe — patterns, captures, predicates.
  Index into usage docs.
---

# Tree-sitter queries

Full guide: [tree-sitter-queries.md](../docs/03_queries_and_languages/01_tree_sitter_queries/00_index.md)

**Dump AST first** (`dump_ast`, `namedOnly: false` for keywords) — do not assume parent/child layout.

- Captures: prefer `(type @cap …)` — [captures](../docs/03_queries_and_languages/01_tree_sitter_queries/03_captures.md)
- Wildcards / anonymous tokens: [basic patterns](../docs/03_queries_and_languages/01_tree_sitter_queries/02_basic_patterns.md)
- Workflow: [authoring](../docs/03_queries_and_languages/01_tree_sitter_queries/07_authoring_workflow.md)
- Codemod-specific: op `capture` + insert `anchor` (`start`/`end`). Debug with `dump_ast` / `debug_query`.

Languages: [language-support.md](../docs/03_queries_and_languages/02_language_support/00_index.md)

Parent: [Architect](../architect/SKILL.md)
