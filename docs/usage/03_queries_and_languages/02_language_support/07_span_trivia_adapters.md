# Span / trivia adapters

| Languages | Adapter behavior |
|-----------|------------------|
| Default (dart, sql, …) | `///` and `//` leading trivia; trailing `;` + newline |
| `rust`, `java`, `kotlin` | C-style block comments + line comments |

Set `includeLeadingTrivia: true` on `remove` / `replace` to expand spans over doc comments.

---
