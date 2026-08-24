# codemod-mcp Playbook (Rust + Luau)

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

- Args:
  - optional `force` (boolean, default false) — overwrite existing files
  - optional `edit_policy` (`"recommend"` | `"strict"`, default `"recommend"`) — soft prefer+preview vs recipe-first mandate
  - optional `companions` (string array, default `[]`) — e.g. `["codebase-memory"]` when that MCP is configured
- Writes: `.agents/skills/*`, `.cursor/rules/codemod-recipe.mdc` (selected policy), optional companion rules, `.codemod/recipes/`, `.codemod/maps/`, `.codemod/variables/`
- Returns: `{ "ok": true, "edit_policy": "...", "companions": [...], "written": [...], "skipped": [...] }`

```json
{ "force": false, "edit_policy": "recommend", "companions": [] }
```

Use `"edit_policy": "strict"` for cookin-style recipe-first edits. Add `"companions": ["codebase-memory"]` independently when codebase-memory-mcp is available. Re-bootstrap with `force: true` to switch packs; omitted companions are not deleted.

### `list_recipes`

Discover registered recipe ids and argument schemas.

- Args: none
- Returns: `recipes[]`, `diagnostics[]`, `mapsLoaded`

### `describe_recipe`

Show metadata for a registered recipe.

- Args: `recipe` (string, required)
- Returns: `recipe` schema (`args`, descriptions, etc.)

### `validate_recipes`

Reload and validate Luau assets under `.codemod/` (recipes, maps, variables, query libs).

- Args: optional `recipe` (string) — validate one recipe by id; omit to validate all
- Returns: `ok` + `diagnostics[]` (may include `hint`, `relatedRecipe`)

### `preview_recipe`

Dry-run and compute a `previewToken`.

Provide exactly one of:
- `recipe` (registered id), or
- `inlineRecipe` (Luau source string)

Optional:
- `args` (JSON: strings, bools, lists, objects)
- `snippetLines` (number)

Success includes:
- `previewToken`
- `files[]` with patch previews
- `transcript[]`

### `apply_recipe`

Apply a previously previewed recipe atomically.

Required:
- `previewToken` from the matching `preview_recipe`

Also provide:
- same `recipe` or same `inlineRecipe`
- same `args`

Optional:
- `selection` for partial patch apply

## inlineRecipe (Luau source)

`inlineRecipe` is a **Luau source string** that `return`s a recipe table with `apply`:

```json
{
  "inlineRecipe": "return { id = \"__inline\", apply = function(c) c:insert({ path = \"lib/a.dart\", query = \"(program) @root\", capture = \"root\", anchor = \"end\", text = \"// x\\n\" }) end }"
}
```

Edit-op rules on the spec table:

- `c:insert` requires: `path`, `query`, `capture`, `anchor`, `text`
- `c:replace` requires: `path`, `query`, `capture`, `text`
- `c:remove` requires: `path`, `query`, `capture`

Set `language` when the path is ambiguous. See skill `codemod-luau-recipes`.

## Recommended agent workflow

1. Locate target code context
2. Select a registered recipe or craft `inlineRecipe` (Luau source)
3. `preview_recipe` and inspect `files[]` / `transcript[]`
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
  "inlineRecipe": "return {\n  id = \"__inline_replace\",\n  apply = function(c)\n    c:replace({\n      path = \"lib/settings.dart\",\n      query = \"(class_definition name: (identifier) @className body: (class_body (declaration (initialized_identifier_list (initialized_identifier (identifier) @fieldName))) @member) (#eq? @className \\\"Settings\\\") (#eq? @fieldName \\\"count\\\"))\",\n      capture = \"member\",\n      text = \"  final int count = 0;\"\n    })\n  end,\n}"
}
```

## Troubleshooting

- `Missing recipe or inlineRecipe`: provide one of them
- `Provide either recipe or inlineRecipe, not both`: remove one
- `Missing previewToken`: apply needs token
- `Stale previewToken`: preview again after file changes
- `Missing required arguments: ...`: add required args
