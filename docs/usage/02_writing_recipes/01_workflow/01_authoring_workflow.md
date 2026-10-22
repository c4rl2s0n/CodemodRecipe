# Authoring workflow

1. **Pick the target** — which file and AST node should change (method body,
   class member, import, …). Inspect the tree in Query Tools or iterate with preview.
2. **Write a query** — S-expression pattern with `@names` for nodes you filter
   on and the node you will edit.
3. **Set `capture`** — the `@name` whose **text span** the op edits (omit the
   `@` in the spec: `@body` → `capture = "body"`).
4. **For `c:insert` only** — set `anchor = "start"` or `"end"`.
5. **Use `args`** — declare dynamics as recipe args; build query/text strings in Luau
   or render with `c:template`.
6. **Validate → preview → apply** — then re-preview; an idempotent recipe
   returns no patches.

Dart is the default language. For other files, set `language` on the edit spec —
see [language-support.md](../../03_queries_and_languages/02_language_support/00_index.md).
