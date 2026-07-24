# Multi-language support reference

codemod-recipe resolves a **tree-sitter grammar per `edit` step**, runs queries against that grammar, and applies byte-range patches. Grammars come from **tree-sitter-language-pack** (lazy download + cache) plus small **native addons** where the pack lacks dialect fidelity.

Rust implementation: `rust/crates/engine/src/registry.rs`, `native.rs`, `adapter.rs`.

Human-oriented copy also lives in [docs/language-support.md](../../../../docs/language-support.md).

---

## How language resolution works

For each `edit` step the host picks a language id in this order:

1. **`edit.language`** when set (trimmed, non-empty)
2. **File extension** via language-pack’s extension table (with overrides below)
3. **Default `dart`** when extension is unknown

Then `LanguageRegistry` loads the grammar (once per process per id) and runs all `ops` in that step.

```text
edit.language?  →  extension map  →  "dart"
       ↓
native override? (sqlite, postgres)
       ↓
tree-sitter-language-pack::get_language(id)  [lazy download + dlopen]
       ↓
Engine + LanguageAdapter
```

### Extension → language (common)

| Extension | Language ID | Notes |
|-----------|-------------|-------|
| `.dart` | `dart` | Pack grammar (UserNobody14/tree-sitter-dart) |
| `.rs` | `rust` | |
| `.java` | `java` | |
| `.kt`, `.kts` | `kotlin` | |
| `.sql` | **configurable** | Default `sqlite` (see below) |
| `.bq` | `sql_bigquery` | |

Any id in the [language-pack catalog](https://docs.tree-sitter-language-pack.xberg.io/languages/) can be used explicitly in `language:` even if the extension is unusual.

---

## The `language` field

Optional on `edit` steps (dslVersion 2):

```yaml
- edit:
    path: "lib/app.dart"
    language: dart          # optional for .dart; required when ambiguous
    ops: [ ... ]
```

**Validation:** if `language` is set, it must be a known id (`validate_recipes` checks against pack catalog + native overrides). Download still happens lazily at preview/apply time.

**Queries are per-language.** A Dart query will not work on Rust source. Do not reuse `query:` strings across languages.

---

## SQL dialects

SQL is **not** one grammar. Use separate language ids:

| Language ID | Source | When to use |
|-------------|--------|-------------|
| `sqlite` | native `tree-sitter-sqlite3` | SQLite DDL/DML (default for `.sql`) |
| `sql` | language-pack (DerekStride) | Generic ANSI-ish SQL |
| `sql_bigquery` | language-pack | BigQuery (`.bq`) |
| `postgres` | native addon (optional build) | PostgreSQL-specific |

### Examples

```yaml
# Generic SQL (explicit — .sql does NOT map to this by default)
- edit:
    path: migrations/001.sql
    language: sql
    ops: [ ... ]

# SQLite (explicit, or omit language for .sql with default config)
- edit:
    path: schema/init.sql
    language: sqlite
    ops: [ ... ]
```

### Default for `.sql` files

When `language:` is omitted and the path ends in `.sql`, the host uses **`sqlite`** by default.

Override via MCP / host flags:

```bash
--sql-default sql          # generic pack SQL
# or
CODEMOD_SQL_DEFAULT=postgres
```

---

## Grammar loading (language-pack)

- **No CLI required** — the Rust crate loads grammars in-process.
- **First use** of a language id may download a platform parser binary from GitHub releases.
- **Cache:** `~/.cache/tree-sitter-language-pack` (subsequent runs offline for cached langs).
- **Lazy:** no startup cost for languages your recipes never reference.
- **Any catalog id** works on first request (e.g. `language: python` on a `.py` file).

### Offline / CI

Pre-warm grammars in test setup or image build (Rust API, not CLI):

```rust
use tree_sitter_language_pack::download;
download(&["dart", "rust", "java", "kotlin", "sql"])?;
```

Copy the cache directory into CI images for sandboxed environments without network.

---

## Dart grammar note (language-pack)

The pack ships a **different Dart AST** than the old `tree-sitter-dart` crate. Node names changed — recipes must use pack names:

| Old (crates.io dart) | Pack dart |
|----------------------|-----------|
| `class_declaration` | `class_definition` |
| `class_member` wrapper | members directly under `class_body` |

**Insert at end of method body (pack dart):**

```yaml
query: |
  (class_definition
    name: (identifier) @className
    body: (class_body
      (method_signature
        (function_signature
          name: (identifier) @methodName))
      (function_body
        (block) @body))
    (#eq? @className "Settings")
    (#eq? @methodName "update"))
capture: body
anchor: end
```

**Remove a field (pack dart):**

```yaml
query: |
  (class_definition
    name: (identifier) @className
    body: (class_body
      (declaration
        (initialized_identifier_list
          (initialized_identifier
            (identifier) @fieldName))) @member)
    (#eq? @className "Settings")
    (#eq? @fieldName "count"))
capture: member
```

See skill `codemod-recipe-authoring` for more Dart patterns.

---

## Other languages (sketch)

Queries must use node kinds from that language’s grammar. Examples:

**Rust — insert after function item:**

```yaml
language: rust
query: "(function_item) @fn"
capture: fn
anchor: end
```

**Java / Kotlin** — use `CStyleLanguageAdapter` trivia rules (`//`, `/* */`, `/** */`).

Use tree-sitter playground or `tree-sitter-language-pack` docs to discover node names for new languages. For query syntax (captures, predicates, operators), see skill `codemod-tree-sitter-queries`.

---

## Span / trivia adapters

| Languages | Adapter behavior |
|-----------|------------------|
| Default (dart, sql, …) | `///` and `//` leading trivia; trailing `;` + newline |
| `rust`, `java`, `kotlin` | C-style block comments + line comments |

Set `includeLeadingTrivia: true` on `remove` / `replace` to expand spans over doc comments.

---

## Host configuration summary

| Flag / env | Purpose |
|------------|---------|
| `--workspace-root` | Target project root |
| `--codemod-root` | `.codemod` directory (default `<workspace>/.codemod`) |
| `--sql-default` | Default language id for `.sql` when `language:` omitted |
| `CODEMOD_SQL_DEFAULT` | Same as `--sql-default` |

---

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|--------------|-----|
| `unknown language: foo` | Invalid `language:` id | Pick id from pack catalog or native list |
| `Invalid node type "class_declaration"` | Old Dart query on pack grammar | Use `class_definition` (see above) |
| `failed to load language` / download error | Network blocked on first use | Pre-download/cache grammars in CI |
| `query matched no nodes` | Wrong language id or query for grammar | Set `language:` explicitly; fix node names |
| Generic SQL on SQLite file | Used `sql` instead of `sqlite` | `language: sqlite` or fix `--sql-default` |

---

## Related skills

- `codemod-yaml-dsl-v2` — `edit` step shape and validation rules
- `codemod-tree-sitter-queries` — query language syntax and codemod capture semantics
- `codemod-recipe-authoring` — Dart query authoring (pack grammar)
- `codemod-mcp-playbook` — MCP tools and workflow
- `codemod-overview` — project orientation
