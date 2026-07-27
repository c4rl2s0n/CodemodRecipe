# Multi-language support

codemod-recipe applies tree-sitter queries per **`edit` step** using a lazy-loaded grammar registry. Dart remains the primary workflow, but Rust, Java, Kotlin, SQL/SQLite, and 300+ other languages are available via [tree-sitter-language-pack](https://docs.tree-sitter-language-pack.xberg.io/).

**Agent skill:** `.agents/skills/codemod-languages/` (installed by `bootstrap_project` from `export/.agents/skills/codemod-languages/`).

## Quick start

1. Add `language:` to `edit` steps when targeting non-Dart files (or to be explicit).
2. Write queries using node names for **that** grammar (queries do not port across languages).
3. Run `validate_recipes` → `preview_recipe` → `apply_recipe` as usual.

```yaml
- edit:
    path: "src/lib.rs"
    language: rust
    ops:
      - insert:
          query: "(function_item) @fn"
          capture: fn
          anchor: end
          text: "\n// codemod\n"
```

## Language resolution

| Priority | Source |
|----------|--------|
| 1 | `edit.language` if set (must be a known id) |
| 2 | File extension (`.rs` → `rust`, `.java` → `java`, `.dart` → `dart`, …) |

If `language:` is omitted and the extension cannot be mapped, preview/apply fails with `file type not supported`. An unknown explicit `language:` fails with `language not supported`.

**`.sql` files:** default dialect is **`sqlite`** (native grammar). Override with `language: sql` or host flag `--sql-default sql`.

## Priority languages

| ID | Extensions | Loader |
|----|------------|--------|
| `dart` | `.dart` | language-pack |
| `rust` | `.rs` | language-pack |
| `java` | `.java` | language-pack |
| `kotlin` | `.kt`, `.kts` | language-pack |
| `sqlite` | `.sql` (default) | native `tree-sitter-sqlite3` |
| `sql` | explicit | language-pack (generic) |
| `sql_bigquery` | `.bq` | language-pack |

## Grammar loading

- Integrated in the **Rust engine** (no separate CLI).
- **Lazy:** first preview/apply for a language downloads and caches the parser.
- **Cache:** `~/.cache/tree-sitter-language-pack`
- **Offline CI:** pre-download with `tree_sitter_language_pack::download(&["dart", ...])` and bake cache into the image.

Implementation: `rust/crates/engine/src/registry.rs`.

## Dart queries (important)

The pack Dart grammar uses **`class_definition`**, not `class_declaration`, and does not wrap members in `class_member`. Update legacy recipes accordingly — see the skill reference for before/after query examples.

## Host flags

```bash
cargo run -q --manifest-path rust/Cargo.toml -p codemod_recipe_host --bin codemod_mcp -- \
  --workspace-root . \
  --codemod-root .codemod \
  --sql-default sqlite
```

Environment: `CODEMOD_SQL_DEFAULT=sql`

## Further reading

- [export/.agents/skills/codemod-languages/reference.md](../export/.agents/skills/codemod-languages/reference.md) — full agent reference
- [docs/codemod-mcp.md](codemod-mcp.md) — MCP setup and workflow
- [tree-sitter-language-pack languages](https://docs.tree-sitter-language-pack.xberg.io/languages/)
