# Troubleshooting

| Symptom | Likely cause | Fix |
|---------|--------------|-----|
| `Invalid node type "…"` | Wrong node name for grammar | Use **Query Tools** / `dump_ast`; see [language-support.md](../02_language_support/00_index.md) |
| `query matched no nodes` | Typo, wrong language, or predicates too strict | Run `debug_query` / Query Tools; loosen predicates; verify `language` |
| `query matched multiple nodes` | Pattern too broad | Add `#eq?`, `field:`, or structural constraints |
| Confused insert position | Mixed up anchors | tree-sitter `.` = tree position; codemod `anchor` = byte offset in capture |
| `Missing capture` | `capture` name not in query | Align `capture` with an `@name` in the query (recipe requires explicit `capture`) |
| `query file not found` | Bad `.scm` path | Place file next to recipe or under `.codemod/queries/` |
| Predicate ignored | Wrong capture or quantifier | Ensure predicate is inside the pattern that owns the capture |
| Preview looks like full-file wipe | Patch is whole-file replace | Use Query Tools to see capture spans; not a capture bug by itself |
| AST tree stale while editing | Query AST refreshes on save / view open, not every keystroke | Save to refresh tree; Run/Generate still use live buffer (host caches parse by hash) |
| Run fails with `@__layer_*` / `.` | Layer highlights instrumentation | Turn off **Layer highlights**, or rely on skip-inject for `.` patterns |

---
