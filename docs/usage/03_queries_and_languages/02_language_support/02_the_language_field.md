# The `language` field

Optional on each insert/replace/remove:

```lua
c:insert({
  path = "lib/app.dart",
  language = "dart",          -- optional for .dart; required when ambiguous
  query = "...",
  capture = "body",
  anchor = "end",
  text = "...",
})
```

**Validation:** if `language` is set, it must be a known id (`validate_recipes` checks against pack catalog + native overrides). Download still happens lazily at preview/apply time.

**Queries are per-language.** A Dart query will not work on Rust source. Do not reuse query strings across languages.

---
