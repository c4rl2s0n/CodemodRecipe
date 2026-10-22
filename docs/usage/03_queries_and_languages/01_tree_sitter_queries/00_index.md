# Tree-sitter query language reference

Tree-sitter queries are **S-expression patterns** matched against a parsed syntax tree. codemod-recipe runs them per `c:insert` / `c:replace` / `c:remove` using the grammar for `language` on the op (or inferred from the file extension).

| Page | |
|------|---|
| [Overview](01_overview.md) | |
| [Basic patterns](02_basic_patterns.md) | |
| [Captures](03_captures.md) | |
| [Operators](04_operators.md) | |
| [Predicates](05_predicates.md) | |
| [Codemod integration](06_codemod_integration.md) | |
| [Authoring workflow](07_authoring_workflow.md) | |
| [Examples](08_examples.md) | |
| [Troubleshooting](09_troubleshooting.md) | |
| [Related docs](10_related_docs.md) | |

