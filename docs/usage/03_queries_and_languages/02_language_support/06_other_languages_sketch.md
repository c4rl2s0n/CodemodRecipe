# Other languages (sketch)

Queries must use node kinds from that language’s grammar. Examples:

**Rust — insert after function item:**

```lua
c:insert({
  path = "src/lib.rs",
  language = "rust",
  query = "(function_item) @fn",
  capture = "fn",
  anchor = "end",
  text = "\n// codemod\n",
})
```

**Java / Kotlin** — use `CStyleLanguageAdapter` trivia rules (`//`, `/* */`, `/** */`).

Use tree-sitter playground or `tree-sitter-language-pack` docs to discover node names for new languages. For query syntax (captures, predicates, operators), see skill `codemod-tree-sitter-queries`.

---
