# codemod-mcp (Rust Host)

MCP server that exposes the Rust `CodemodHost` protocol as tools for AI agents.
Use it to preview and apply deterministic Dart AST edits from registered YAML
recipes (`.codemod/recipes/*.yaml`) or `inlineRecipe` payloads.

**Agent playbook:** `.cursor/skills/codemod-mcp/reference.md`

## Quick start

Run the Rust MCP server:

```bash
cd /path/to/codemod_recipe
cargo run -q --manifest-path rust/Cargo.toml -p codemod_recipe_host --bin codemod_mcp -- \
  --workspace-root /absolute/path/to/target-workspace \
  --codemod-root .codemod
```

The server speaks MCP over stdio. It appears idle when healthy.

## Target workspace layout

In the target workspace:

```text
.codemod/
  recipes/   # required for registered recipe flow
  maps/      # optional
  templates/ # optional (create.templateFile)
```

You can also use inline-only workflows with `inlineRecipe`.

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
        ".codemod"
      ]
    }
  }
}
```

Reload MCP servers and call `list_recipes` to verify connectivity.

## Tools

All tools return a JSON string. Parse it, then check `ok`.

| Tool | Purpose |
|------|---------|
| `bootstrap_project` | Install agent skills, rules, and `.codemod/` scaffolding |
| `list_recipes` | Discover registered recipe ids and argument schemas |
| `describe_recipe` | Show metadata/args for one recipe |
| `validate_recipes` | Reload + validate recipes/maps |
| `preview_recipe` | Dry-run recipe; returns `previewToken` |
| `apply_recipe` | Apply recipe atomically; requires `previewToken` |

## End-to-end workflow

0. `bootstrap_project` (once per workspace)
1. `validate_recipes` after recipe edits
2. `list_recipes`
3. `describe_recipe`
4. `preview_recipe` (save `previewToken`)
5. `apply_recipe` with the same recipe/args and that token
6. Optional: preview again to verify idempotency (`files: []`)

## Registered recipe examples

`preview_recipe`:

```json
{
  "recipe": "add_log_line",
  "args": {
    "file": "lib/settings.dart",
    "className": "Settings",
    "methodName": "update"
  },
  "snippetLines": 5
}
```

`apply_recipe`:

```json
{
  "recipe": "add_log_line",
  "args": {
    "file": "lib/settings.dart",
    "className": "Settings",
    "methodName": "update"
  },
  "previewToken": "<from preview_recipe>",
  "selection": {
    "files": {
      "lib/settings.dart": {
        "include": true
      }
    }
  }
}
```

`selection` is optional; omit to apply all patches.

## Inline recipe examples (v2 shape)

`inlineRecipe` follows the Rust YAML model:

- top-level: `id`, `steps`, optional `args`, `maps`, `postExecution`
- edit step: `edit.path`, `edit.ops[]`
- ops: `insert`, `replace`, `remove`

### Inline insert

```json
{
  "inlineRecipe": {
    "id": "__inline_insert_log",
    "steps": [
      {
        "edit": {
          "path": "lib/settings.dart",
          "ops": [
            {
              "insert": {
                "query": "(class_declaration name: (identifier) @className body: (class_body (class_member (method_signature (function_signature name: (identifier) @methodName)) (function_body (block) @body))) (#eq? @className \"Settings\") (#eq? @methodName \"update\"))",
                "capture": "body",
                "anchor": "end",
                "text": "    print('codemod');\\n"
              }
            }
          ]
        }
      }
    ]
  }
}
```

### Inline replace

```json
{
  "inlineRecipe": {
    "id": "__inline_replace_count",
    "steps": [
      {
        "edit": {
          "path": "lib/settings.dart",
          "ops": [
            {
              "replace": {
                "query": "(class_declaration name: (identifier) @className body: (class_body (class_member (declaration (initialized_identifier_list (initialized_identifier (identifier) @fieldName))) @member)) (#eq? @className \"Settings\") (#eq? @fieldName \"count\"))",
                "capture": "member",
                "text": "  final int count = 0;"
              }
            }
          ]
        }
      }
    ]
  }
}
```

### Inline remove

```json
{
  "inlineRecipe": {
    "id": "__inline_remove_count",
    "steps": [
      {
        "edit": {
          "path": "lib/settings.dart",
          "ops": [
            {
              "remove": {
                "query": "(class_declaration name: (identifier) @className body: (class_body (class_member (declaration (initialized_identifier_list (initialized_identifier (identifier) @fieldName))) @member)) (#eq? @className \"Settings\") (#eq? @fieldName \"count\"))",
                "capture": "member"
              }
            }
          ]
        }
      }
    ]
  }
}
```

### Field rules

- `insert`: requires `query`, `capture`, `anchor` (`start` or `end`), `text`
- `replace`: requires `query`, `capture`, `text`
- `remove`: requires `query`, `capture`
- `includeLeadingTrivia` is optional for `replace` and `remove`

## Create/delete in v2 recipes

You can combine file creation/deletion with edit ops in one recipe:

- Create example: `test/fixtures/scaffold_project/.codemod/recipes/create_repository.yaml`
- Delete example: `test/fixtures/rust_oracle/delete_legacy.recipe.yaml`
- Design patterns: bootstrap skill `.agents/skills/codemod-recipe-design-patterns/` (or [recipe-design-patterns.md](recipe-design-patterns.md) stub)

## Troubleshooting

| Symptom | Action |
|---------|--------|
| `Unknown recipe` | Call `list_recipes`, verify id and workspace root |
| `Missing previewToken` | Call `preview_recipe` first and pass token to apply |
| `Stale previewToken` | Re-run preview after any file change |
| Empty `files` in preview | Query/capture matched nothing, or recipe already applied |
| `Missing required arguments: ...` | Supply required args from `describe_recipe` |

## Related docs

- Bootstrap skills: `.agents/skills/codemod-recipe-design-patterns/` (after `bootstrap_project`)
- `docs/recipe-design-patterns.md` — human stub pointing to agent skills
- `docs/new-project-rust-mcp.md`
- `.cursor/skills/codemod-mcp/reference.md`
- `README.md`
