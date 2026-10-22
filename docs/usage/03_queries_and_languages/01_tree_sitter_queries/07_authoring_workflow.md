# Authoring workflow

**Dump the AST first.** Do not assume parent/child layout from another language or from memory — grammars often put method bodies, modifiers, and keywords as **siblings**, not children.

1. **Pick language** — set `language` on the op or rely on extension ([language support](../02_language_support/00_index.md)).
2. **`dump_ast` on a real sample** — MCP `dump_ast` (or Query Tools). Use `namedOnly: false` when looking for keywords / punctuation.
3. **Confirm structure** — node types, fields, sibling vs child relationships, anonymous tokens.
4. **Write pattern** — start broad, add `field:` and `@captures` ([captures](03_captures.md)). Prefer `(type @cap …)`.
5. **Match anonymous siblings in the query** — e.g. `"required"` next to a parameter node — instead of reading source text before a span.
6. **`debug_query`** — validate captures and match count (pass `instrument: false` only if debugging instrumentation itself).
7. **Filter** — add `#eq?` / `#match?` until one match per edit target.
8. **Choose edit capture** — set `capture` to the node you will insert beside, replace, or remove.
9. **Validate** — `validate_recipes` (Luau load + language id).
10. **Preview / dry-run** — `preview_recipe` or `component_dry_run_hook`; read `logs` / `transcript` for `c:log` (`component_scan` has no logs).
11. **Idempotency** — re-preview after apply; expect empty `files` when done.

---
