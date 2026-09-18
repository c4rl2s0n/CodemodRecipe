# Tree-sitter queries

Queries stay as **inline strings** or **`.scm` files** (path resolved recipe-local, then `.codemod/`).
Query libraries are Luau modules (conventionally under `.codemod/queries/`) that return
`{ id = "...", queries = { ... } }`.

See [tree-sitter-queries.md](../../03_queries_and_languages/01_tree_sitter_queries/00_index.md).
