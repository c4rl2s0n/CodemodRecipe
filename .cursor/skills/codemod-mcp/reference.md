# codemod-mcp Playbook (Rust + YAML v2)

MCP server `codemod-mcp` wraps the Rust host and returns JSON text.
Always parse tool output and branch on `ok`.

Server command:

```bash
cargo run -q --manifest-path rust/Cargo.toml -p codemod_recipe_host --bin codemod_mcp -- \
  --workspace-root <root> \
  --codemod-root .codemod
```

## Response envelope

```json
{ "ok": true, "...": "..." }
{ "ok": false, "error": "human-readable message" }
```

## Tool reference

### `bootstrap_project`

Install codemod-recipe agent guidance into the workspace.

- Args: optional `force` (boolean, default false) — overwrite existing files
- Writes: `.agents/skills/*`, `.cursor/rules/codemod-recipe.mdc`, `.codemod/recipes/`, `.codemod/maps/`
- Returns: `{ "ok": true, "written": [...], "skipped": [...] }`

### `list_recipes`

Discover registered recipe ids and argument schemas.

- Args: none
- Returns: `recipes[]`, `diagnostics[]`, `mapsLoaded`

### `describe_recipe`

Show metadata for a registered recipe.

- Args: `recipe` (string, required)
- Returns: `recipe` schema (`args`, descriptions, etc.)

### `validate_recipes`

Reload and validate `.codemod/recipes` and `.codemod/maps`.

- Args: none
- Returns: `ok` + `diagnostics[]`

### `preview_recipe`

Dry-run and compute a `previewToken`.

Provide exactly one of:
- `recipe` (registered id), or
- `inlineRecipe` (full v2 recipe object)

Optional:
- `args` (string map)
- `snippetLines` (number)

Success includes:
- `previewToken`
- `files[]` with patch previews

### `apply_recipe`

Apply a previously previewed recipe atomically.

Required:
- `previewToken` from the matching `preview_recipe`

Also provide:
- same `recipe` or same `inlineRecipe`
- same `args`

Optional:
- `selection` for partial patch apply

## v2 inlineRecipe schema (Rust model)

Inline recipe mirrors YAML v2 shape:

```json
{
  "id": "__inline_id",
  "args": [],
  "maps": {},
  "steps": [
    {
      "edit": {
        "path": "lib/file.dart",
        "ops": [
          { "insert":  { "query": "...", "capture": "x", "anchor": "start", "text": "..." } },
          { "replace": { "query": "...", "capture": "x", "text": "..." } },
          { "remove":  { "query": "...", "capture": "x" } }
        ]
      }
    },
    {
      "create": {
        "path": "lib/new_file.dart",
        "template": "class X {}",
        "ifExists": "fail",
        "format": true
      }
    },
    {
      "delete": {
        "path": "lib/legacy/old.dart",
        "ifMissing": "skip"
      }
    }
  ],
  "postExecution": ["dartFormat"]
}
```

## Edit-op rules (v2)

- `insert` requires: `query`, `capture`, `anchor`, `text`
  - `anchor` must be `start` or `end`
- `replace` requires: `query`, `capture`, `text`
  - optional: `includeLeadingTrivia`
- `remove` requires: `query`, `capture`
  - optional: `includeLeadingTrivia`

`query` is a tree-sitter query string. `capture` names the capture whose node
range is used for the operation.

## Recommended agent workflow

1. Locate target code context
2. Select a registered recipe or craft v2 `inlineRecipe`
3. `preview_recipe` and inspect `files[]`
4. Save `previewToken`
5. `apply_recipe` with same recipe/args and token
6. Re-preview to verify idempotency (`files: []` expected)

## Minimal examples

### Registered recipe preview

```json
{
  "recipe": "add_log_line",
  "args": {
    "file": "lib/settings.dart",
    "className": "Settings",
    "methodName": "update"
  }
}
```

### Registered recipe apply

```json
{
  "recipe": "add_log_line",
  "args": {
    "file": "lib/settings.dart",
    "className": "Settings",
    "methodName": "update"
  },
  "previewToken": "<from preview>"
}
```

### Inline replace preview

```json
{
  "inlineRecipe": {
    "id": "__inline_replace",
    "steps": [
      {
        "edit": {
          "path": "lib/settings.dart",
          "ops": [
            {
              "replace": {
                "query": "(class_definition name: (identifier) @className body: (class_body (declaration (initialized_identifier_list (initialized_identifier (identifier) @fieldName))) @member) (#eq? @className \"Settings\") (#eq? @fieldName \"count\"))",
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

## Troubleshooting

- `Missing recipe or inlineRecipe`: provide one of them
- `Provide either recipe or inlineRecipe, not both`: remove one
- `Missing previewToken`: apply needs token
- `Stale previewToken`: preview again after file changes
- `Missing required arguments: ...`: add required args
