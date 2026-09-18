# Authoring workflow

1. **Pick language** — set `language` on the op or rely on extension (`codemod-languages`).
2. **Inspect AST** — tree-sitter playground or `tree-sitter parse` / grammar docs for node kinds.
3. **Write pattern** — start broad, add `field:` and `@captures`.
4. **Filter** — add `#eq?` / `#match?` until one match per file.
5. **Choose edit capture** — set `capture` to the node you will insert beside, replace, or remove.
6. **Validate** — `validate_recipes` (Luau load + language id).
7. **Preview** — `preview_recipe`; inspect `files[].original` and `files[].modified`.
8. **Idempotency** — re-preview after apply; expect empty `files` when done (`codemod-recipe-authoring`).

---
