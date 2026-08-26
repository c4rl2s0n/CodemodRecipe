# codemod-mcp (Rust Host)

MCP server that exposes the Rust `CodemodHost` protocol as tools for AI agents.
Use it to preview and apply deterministic AST edits from registered **Luau
recipes** (`.luau` or `.lua` modules anywhere under `.codemod/` that return `{ id, apply = function(c) ... }`)
or `inlineRecipe` Luau source strings. The `.codemod/recipes/` directory is a convention.
Supports Dart, Rust, Java, Kotlin, SQL/SQLite, and 300+ languages via lazy-loaded
tree-sitter grammars.

**Agent playbook:** `.agents/skills/codemod-recipe/codemod-mcp-playbook/`  
**Recipe API:** `.agents/skills/codemod-recipe/codemod-luau-recipes/`  
**Templates:** [recipe-templates.md](recipe-templates.md)  
**Multi-language:** [language-support.md](language-support.md)

## Quick start

Build the Rust MCP server:

```bash
cd /path/to/codemod_recipe
cargo build -q --manifest-path rust/Cargo.toml -p codemod_recipe_host --bin codemod_mcp --release
```

Run the Rust MCP server:

```bash
cd /path/to/codemod_recipe
cargo run -q --manifest-path rust/Cargo.toml -p codemod_recipe_host --bin codemod_mcp -- \
  --workspace-root /absolute/path/to/target-workspace \
  --codemod-root .codemod
```

The server speaks MCP over stdio. It appears idle when healthy.

## Target workspace layout

```text
.codemod/
  recipes/    # Luau recipe modules (*.luau / *.lua)
  maps/       # Luau map modules (id + map table)
  variables/  # Luau variable modules (id + values table)
  queries/    # Luau query libraries (id + queries table)
  templates/  # optional (renderFileTemplate)
```

Discovery scans `.codemod/` for Luau modules with the expected return shapes.

You can also use inline-only workflows with `inlineRecipe` (Luau source string).

Path resolution (`rust/crates/core/src/resource_path.rs`):

- workspace mutation targets resolve exactly under the workspace root
- recipe resources (`.scm`, templates, postExecution scripts) resolve recipe-local first, then `.codemod/`
- bare query names also try `queries/` under each root

## Setup in Cursor

Create `.cursor/mcp.json` in the target workspace:

```json
{
  "mcpServers": {
    "codemod-mcp": {
      "command": "cargo",
      "args": [
        "run",
        "-q",
        "--manifest-path",
        "/absolute/path/to/codemod_recipe/rust/Cargo.toml",
        "-p",
        "codemod_recipe_host",
        "--bin",
        "codemod_mcp",
        "--",
        "--workspace-root",
        ".",
        "--codemod-root",
        ".codemod",
        "--sql-default",
        "sqlite"
      ]
    }
  }
}
```

Reload MCP servers and call `list_recipes` to verify connectivity.

## Multi-language support

- Set `language` on edit specs (`c:insert`, etc.) for non-Dart files.
- Queries must use node names for that grammar.
- `.sql` paths default to `sqlite`; pass `language = "sql"` or `--sql-default sql`.

See [language-support.md](language-support.md).

## Tools

All tools return a JSON string. Parse it, then check `ok`.

| Tool | Purpose |
|------|---------|
| `bootstrap_project` | Install agent skills, rules, `.codemod/` scaffolding |
| `list_recipes` | Discover registered recipe ids and argument schemas |
| `describe_recipe` | Show metadata/args for one recipe |
| `validate_recipes` | Reload + validate Luau recipes/maps/variables/queries |
| `preview_recipe` | Dry-run; returns `previewToken` + transcript |
| `apply_recipe` | Apply atomically; requires `previewToken` |
| `dump_ast` | Dump tree-sitter AST |
| `debug_query` | Run a query; return match roots and captures |
| `generate_query` | Starter query from byte `start`/`end` in a file |
| `resolve_static_path` | Jinja-render a path with empty args (static paths only) |

## End-to-end workflow

0. `bootstrap_project` (once per workspace)
1. `validate_recipes` after recipe edits
2. `list_recipes` / `describe_recipe`
3. `preview_recipe` (save `previewToken`; inspect `files` + `transcript`)
4. `apply_recipe` with the same recipe/args and that token
5. Optional: preview again to verify idempotency (`files: []`)

## Preview vs apply

- Both run `apply(c)`. Preview uses a WorkingTree; **postExec is skipped** (listed in transcript).
- Apply **re-runs** Luau and flushes before postExec / `postExecution`.
- `previewToken` = hash(recipe + args + starting file contents).

## Registered recipe examples

`preview_recipe`:

```json
{
  "recipe": "insert_log_line",
  "args": {
    "file": "lib/settings.dart"
  },
  "snippetLines": 5
}
```

`apply_recipe`:

```json
{
  "recipe": "insert_log_line",
  "args": {
    "file": "lib/settings.dart"
  },
  "previewToken": "<from preview_recipe>",
  "selection": {
    "files": {
      "lib/settings.dart": { "include": true }
    }
  }
}
```

## Inline recipe (`inlineRecipe`)

`inlineRecipe` is a **Luau source string** that `return`s a recipe table with `apply`:

```json
{
  "inlineRecipe": "return {\n  id = \"__inline_insert\",\n  apply = function(c)\n    c:insert({\n      path = \"lib/settings.dart\",\n      query = \"(class_definition name: (identifier) @className body: (class_body (method_signature (function_signature name: (identifier) @methodName)) (function_body (block) @body)) (#eq? @className \\\"Settings\\\") (#eq? @methodName \\\"update\\\"))\",\n      capture = \"body\",\n      anchor = \"end\",\n      text = \"    print('codemod');\\n\",\n    })\n  end,\n}"
}
```

Field rules on edit specs (same as registered recipes):

- `insert`: `path`, `query`, `capture`, `anchor` (`start`|`end`), `text`
- `replace`: `path`, `query`, `capture`, `text`
- `remove`: `path`, `query`, `capture`

## Troubleshooting

| Symptom | Action |
|---------|--------|
| `Unknown recipe` | Call `list_recipes`, verify id and workspace root |
| `Missing previewToken` | Call `preview_recipe` first |
| `Stale previewToken` | Re-run preview after file changes |
| `Invalid node type "class_declaration"` | Use `class_definition` in Dart queries |
| `language not supported` | Set valid `language` on the op |
| Empty `files` in preview | Query matched nothing, or recipe already applied |
| Apply differs from preview | Expected when postExec ran on apply only — check transcript |

## Related docs

- [writing-recipes.md](writing-recipes.md) — Luau authoring
- [new-project-rust-mcp.md](new-project-rust-mcp.md)
- [tree-sitter-queries.md](tree-sitter-queries.md)
- [language-support.md](language-support.md)
