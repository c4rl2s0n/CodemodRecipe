# How language resolution works

For each insert/replace/remove the host picks a language id in this order:

1. **`language`** on the op when set (trimmed, non-empty; must be a known id)
2. **File extension** via language-pack’s extension table (with overrides below)

If neither yields a known language, preview/apply fails (`language not supported` or `file type not supported`). There is no default language.

Then `LanguageRegistry` loads the grammar (once per process per id) and runs the op.

```text
op.language?  →  extension map  →  error if unresolved
       ↓
native override? (sqlite, postgres)
       ↓
tree-sitter-language-pack::get_language(id)  [lazy download + dlopen]
       ↓
Engine + LanguageAdapter
```

## Extension → language (common)

| Extension | Language ID | Notes |
|-----------|-------------|-------|
| `.dart` | `dart` | Pack grammar (UserNobody14/tree-sitter-dart) |
| `.rs` | `rust` | |
| `.java` | `java` | |
| `.kt`, `.kts` | `kotlin` | |
| `.sql` | **configurable** | Default `sqlite` (see below) |
| `.bq` | `sql_bigquery` | |

Any id in the [language-pack catalog](https://docs.tree-sitter-language-pack.xberg.io/languages/) can be used explicitly in `language` even if the extension is unusual.

---
