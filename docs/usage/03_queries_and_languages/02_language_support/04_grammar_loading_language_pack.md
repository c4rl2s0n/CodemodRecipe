# Grammar loading (language-pack)

- **No CLI required** — the Rust crate loads grammars in-process.
- **First use** of a language id may download a platform parser binary from GitHub releases.
- **Cache:** `~/.cache/tree-sitter-language-pack` (subsequent runs offline for cached langs).
- **Lazy:** no startup cost for languages your recipes never reference.
- **Any catalog id** works on first request (e.g. `language = "python"` on a `.py` file).

## Offline / CI

Pre-warm grammars in test setup or image build (Rust API, not CLI):

```rust
use tree_sitter_language_pack::download;
download(&["dart", "rust", "java", "kotlin", "sql"])?;
```

Copy the cache directory into CI images for sandboxed environments without network.

---
