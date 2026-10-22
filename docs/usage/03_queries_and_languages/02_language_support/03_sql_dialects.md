# SQL dialects

SQL is **not** one grammar. Use separate language ids:

| Language ID | Source | When to use |
|-------------|--------|-------------|
| `sqlite` | native `tree-sitter-sqlite3` | SQLite DDL/DML (default for `.sql`) |
| `sql` | language-pack (DerekStride) | Generic ANSI-ish SQL |
| `sql_bigquery` | language-pack | BigQuery (`.bq`) |
| `postgres` | native addon (optional build) | PostgreSQL-specific |

## Examples

```lua
-- Generic SQL (explicit — .sql does NOT map to this by default)
c:insert({
  path = "migrations/001.sql",
  language = "sql",
  query = "...",
  capture = "stmt",
  anchor = "end",
  text = "...",
})

-- SQLite (explicit, or omit language for .sql with default config)
c:insert({
  path = "schema/init.sql",
  language = "sqlite",
  query = "...",
  capture = "stmt",
  anchor = "end",
  text = "...",
})
```

## Default for `.sql` files

When `language` is omitted and the path ends in `.sql`, the host uses **`sqlite`** by default.

Override via MCP / host flags:

```bash
--sql-default sql          # generic pack SQL
# or
CODEMOD_SQL_DEFAULT=postgres
```

---
