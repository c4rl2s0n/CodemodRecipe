---
name: codemod-languages
description: Multi-language tree-sitter support in codemod-recipe — language IDs, extension inference, SQL dialects, lazy grammar loading, and per-language query authoring. Read reference.md for the full guide.
---

# Multi-language support

## When to use

- Choosing a `language:` id for an `edit` step (Dart, Rust, Java, Kotlin, SQL, SQLite, …)
- Editing non-Dart files with tree-sitter queries
- Configuring SQL dialect defaults (`.sql` → `sqlite` vs generic `sql`)
- Understanding first-use grammar download or offline/CI setup
- Debugging `language not supported`, `file type not supported`, or `Invalid node type` query errors

## When not to use

- YAML step syntax → `codemod-yaml-dsl-v2`
- Dart-specific query patterns → `codemod-recipe-authoring`
- Running preview/apply → `codemod-mcp-playbook`

## Quick rules

1. Set `language:` on every `edit` step when the file is **not** Dart, or when inference is ambiguous.
2. Queries are **not portable** across languages — write per-language node names.
3. `.sql` defaults to **`sqlite`** (native grammar). Use `language: sql` for generic ANSI-ish SQL.
4. Grammars load **lazily** on first use via `tree-sitter-language-pack` (cached under `~/.cache/tree-sitter-language-pack`).

## Priority language IDs

| ID | Typical extensions | Loader |
|----|-------------------|--------|
| `dart` | `.dart` | language-pack |
| `rust` | `.rs` | language-pack |
| `java` | `.java` | language-pack |
| `kotlin` | `.kt`, `.kts` | language-pack |
| `sql` | (explicit only) | language-pack |
| `sqlite` | `.sql` (default) | native addon |
| `sql_bigquery` | `.bq` | language-pack |

306+ additional IDs are available from language-pack on first request (e.g. `python`, `go`, `zig`).

## Minimal examples

**Rust edit (explicit language):**

```yaml
- edit:
    path: "src/main.rs"
    language: rust
    ops:
      - insert:
          query: "(function_item) @fn"
          capture: fn
          anchor: end
          text: "\n// codemod\n"
```

**SQLite schema (default for `.sql`):**

```yaml
- edit:
    path: "schema/init.sql"
    language: sqlite
    ops:
      - insert:
          query: "(create_table_statement) @stmt"
          capture: stmt
          anchor: end
          text: "\n-- migrated\n"
```

## Instructions

1. **Read and follow** [reference.md](reference.md) for resolution order, SQL dialects, Dart pack grammar notes, host flags, and troubleshooting.
2. Call `validate_recipes` after setting `language:` — unknown IDs fail at validate time.
3. Preview before apply; query node names must match the grammar for the chosen `language`.

## Related skills

| Skill | Use for |
|-------|---------|
| `codemod-yaml-dsl-v2` | `edit.language` field and step shape |
| `codemod-tree-sitter-queries` | query syntax, captures, predicates |
| `codemod-recipe-authoring` | tree-sitter queries (Dart pack grammar) |
| `codemod-mcp-playbook` | preview/apply workflow |
| `codemod-overview` | project layout and orientation |
