# Troubleshooting

| Symptom | Likely cause | Fix |
|---------|--------------|-----|
| Wrong node captured | `@` after the wrong `)` | Prefer `(type @cap …)`; see [captures](03_captures.md) |
| `Invalid syntax` / vague query error | Unbalanced `()`, capture after a predicate, bad `@` | Check paren balance; put captures before `(#…)`; read the Hint line on the error |
| `unbalanced parentheses (N unclosed` | Missing `)` in `[[…]]` query | Count grouping; predicates do not close patterns |
| Syntax error near `_ @…` | Bare `_` is not capturable | Use `(_) @name` for named wildcards, or `"token"` for anonymous |
| Keyword / `"required"` never matches | Anonymous token not in named-only dump | `dump_ast` with `namedOnly: false`; match with `"…"` |
| Pattern finds nothing; structure “should” nest | Assumed parent/child | **Dump AST** — bodies/modifiers are often **siblings** |
| `Invalid node type "…"` | Wrong node name for grammar | `dump_ast`; see [language-support](../02_language_support/00_index.md) |
| `query matched no nodes` | Typo, wrong language, or predicates too strict | `debug_query`; loosen predicates; verify `language` |
| `query matched multiple nodes` | Pattern too broad | Add `#eq?`, `field:`, or structural constraints |
| Confused insert position | Mixed up anchors | tree-sitter `.` = tree position; codemod `anchor` = byte offset in capture |
| `Missing capture` | `capture` name not in query | Align `capture` with an `@name` in the query |
| `query file not found` | Bad `.scm` path | Place file next to recipe or under `.codemod/queries/` |
| Predicate ignored | Wrong capture or quantifier | Ensure predicate is inside the pattern that owns the capture |
| Preview looks like full-file wipe | Patch is whole-file replace | Use Query Tools / `debug_query` to see capture spans |
| AST tree stale while editing | Query AST refreshes on save / view open | Save to refresh; Run/Generate use live buffer |
| Run fails with `@__layer_*` / `.` | Layer highlights instrumentation | Turn off layer highlights, or `instrument: false` on `debug_query` |
| `c:log` missing from scan | Scan has no transcript | Use `component_dry_run_hook` / `preview_recipe`; read `logs` |

---
