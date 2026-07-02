# codemod-mcp

MCP server that exposes the `CodemodHost` protocol as tools for AI agents. Use it to preview and apply AST-targeted Dart edits from registered YAML recipes or inline recipe bodies.

**Agent playbook:** `.cursor/skills/codemod-mcp/reference.md`

## Quick start

```bash
cd /path/to/your/workspace   # must contain .codemod/ OR use inline recipes only
dart pub get                  # in codemod_recipe package when running from source
dart run /path/to/codemod_recipe/bin/codemod_mcp.dart \
  --workspace-root . \
  --codemod-root .codemod
```

### Rust host (work-in-progress)

This repo now also contains an experimental Rust implementation under `rust/`.
It currently supports a subset of tools and does **not** support `generate_ast_path`.

```bash
cd /path/to/codemod_recipe
cargo run -q --manifest-path rust/Cargo.toml -p codemod_recipe_host --bin codemod_mcp -- \
  --workspace-root . \
  --codemod-root .codemod
```

The server speaks **MCP over stdio** — it will appear to hang when run in a terminal. That is normal. Attach an MCP client (Cursor, MCP Inspector) to interact with it.

**You do not need a separate Dart project.** Any directory can be a workspace:

- **This repo** — already has `.codemod/recipes/add_log_line.yaml`
- **This repo (Rust DSL)** — also has `.codemod/recipes/insert_log_line.yaml`
- **Temp fixture** — create `lib/foo.dart` + optional `.codemod/recipes/` (see [Manual testing](#manual-testing))
- **Inline only** — no recipes on disk; pass `inlineRecipe` to `preview_recipe`

## Setup

### Cursor

Create or edit `.cursor/mcp.json` in your **target workspace** (the project you want to codemod):

```json
{
  "mcpServers": {
    "codemod-mcp": {
      "command": "dart",
      "args": [
        "run",
        "/absolute/path/to/codemod_recipe/bin/codemod_mcp.dart",
        "--workspace-root",
        ".",
        "--codemod-root",
        ".codemod"
      ]
    }
  }
}
```

If `codemod_recipe` is a path dependency in your workspace, you can use a relative path to `bin/codemod_mcp.dart` instead.

Restart Cursor (or reload MCP servers). Invoke the **`codemod-mcp`** skill or ask the agent to use `list_recipes`, `preview_recipe`, etc.

### Vibe CLI

Already configured in this repo’s `.vibe/config.toml`:

```toml
[[mcp_servers]]
name = "codemod-mcp"
command = "dart run bin/codemod_mcp.dart"
args = ["--workspace-root", "."]
```

### MCP Inspector (recommended for manual testing)

Interactive UI to call tools without Cursor:

```bash
cd /path/to/codemod_recipe
npx @modelcontextprotocol/inspector \
  dart run bin/codemod_mcp.dart --workspace-root . --codemod-root .codemod
```

Opens a browser UI. Connect, then try:

1. **Tools → `list_recipes`** — `{}`
2. **Tools → `generate_ast_path`** — `{ "path": "/abs/path/lib/settings.dart", "offset": 28 }`
3. **Tools → `preview_recipe`** — inline remove example below
4. **Tools → `apply_recipe`** — same `inlineRecipe` + `previewToken` from step 3

Parse each tool result as JSON (the text content is `jsonEncode(hostResponse)`).

### Requirements

- Dart SDK on `PATH`
- `dart pub get` in the `codemod_recipe` package
- Target workspace path passed to `--workspace-root`
- For registered recipes: `.codemod/recipes/*.yaml` under that workspace
- For inline recipes: only `fromConfig` host (default for `bin/codemod_mcp.dart`)

## Manual testing

### Option A — use this repo as the workspace

```bash
cd /home/ikusa/workspace/Android/codemod_recipe
npx @modelcontextprotocol/inspector \
  dart run bin/codemod_mcp.dart --workspace-root . --codemod-root .codemod
```

Call `list_recipes` — you should see `add_log_line`.

### Option B — disposable temp workspace

```bash
WORK=/tmp/codemod_mcp_manual
mkdir -p "$WORK/lib" "$WORK/.codemod/recipes"

cat > "$WORK/lib/settings.dart" <<'EOF'
class Settings {
  final int count = 0;
  final String name = 'x';
}
EOF

# optional: copy a recipe
cp test/fixtures/yaml_recipes/remove_counter_field.yaml "$WORK/.codemod/recipes/"

npx @modelcontextprotocol/inspector \
  dart run bin/codemod_mcp.dart --workspace-root "$WORK" --codemod-root .codemod
```

### Option C — inline recipe (no `.codemod` needed)

Use **`preview_recipe`** with an absolute `edit.path`:

```json
{
  "inlineRecipe": {
    "id": "__inline_remove_count",
    "steps": [{
      "edit": {
        "path": "/tmp/codemod_mcp_manual/lib/settings.dart",
        "steps": [{
          "remove": {
            "at": [
              { "class": "Settings" },
              { "field": "count" }
            ]
          }
        }]
      }
    }]
  }
}
```

Then **`apply_recipe`** with the same `inlineRecipe` and the `previewToken` from the preview response.

### End-to-end checklist

1. `list_recipes` or skip if using inline only
2. `generate_ast_path` at a cursor offset (optional; builds `at:` steps)
3. `preview_recipe` → note `previewToken` and `files[]`
4. `apply_recipe` with matching recipe + token
5. `preview_recipe` again → `files` should be empty (idempotent)

## Tools

All tools return a **JSON string** (encoded host response). Parse it and check `ok` before using fields.

| Tool | Purpose |
|------|---------|
| `list_recipes` | Discover registered recipe ids |
| `describe_recipe` | Args + metadata for one recipe |
| `validate_recipes` | YAML/schema diagnostics |
| `generate_ast_path` | Offset → navigate steps |
| `preview_recipe` | Dry-run; returns `previewToken` |
| `apply_recipe` | Write changes; requires `previewToken` |

### `list_recipes`

**Input:** `{}`

**Response (success):**

```json
{
  "ok": true,
  "recipes": [{ "id": "add_log_line", "name": "...", "args": [...] }],
  "diagnostics": []
}
```

### `describe_recipe`

**Input:** `{ "recipe": "add_log_line" }`

**Response (success):** `{ "ok": true, "recipe": { "id", "name", "description", "args", ... } }`

### `validate_recipes`

Reload and validate all YAML recipes/maps.

**Input:** `{}`

**Response:** `{ "ok": true|false, "diagnostics": [{ "severity", "code", "message", ... }] }`

### `generate_ast_path`

Convert a byte offset in a Dart file into AST navigate steps.

**Input:**

```json
{
  "path": "/absolute/path/to/lib/settings.dart",
  "offset": 42
}
```

**Response (success):**

```json
{
  "ok": true,
  "path": {
    "navigate": [
      { "kind": "classDecl", "name": "Settings" },
      { "kind": "field", "name": "count" }
    ],
    "anchor": "Anchor(...)",
    "offset": 42
  }
}
```

For **remove** and **replace** inline steps, copy `navigate` into `at:` and **do not** include `anchor`. For **insert**, add `anchor` from the response (or choose explicitly, e.g. `stmt:last`).

### `preview_recipe`

Dry-run. Does not write files. Returns a **`previewToken`** required for apply.

**Registered recipe:**

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

**Inline recipe** (paths in `edit.path` should be **absolute**):

```json
{
  "inlineRecipe": {
    "id": "__inline_remove_count",
    "steps": [{
      "edit": {
        "path": "/abs/path/lib/settings.dart",
        "steps": [{
          "remove": {
            "at": [
              { "class": "Settings" },
              { "field": "count" }
            ]
          }
        }]
      }
    }]
  }
}
```

**Response (success):**

```json
{
  "ok": true,
  "recipe": "...",
  "previewToken": "<sha256>",
  "files": [{
    "path": "...",
    "patches": [{ "start", "end", "replacementPreview": "..." }]
  }]
}
```

Empty `files` means **no changes** (idempotent or already applied).

### `apply_recipe`

Writes selected changes atomically (rollback on failure). Runs `postExecution` after commit.

**Input:** Same `recipe` or `inlineRecipe` as preview, plus:

```json
{
  "previewToken": "<from preview_recipe>",
  "args": { }
}
```

Optional `selection` (same shape as VS Code host) to apply a subset of patches.

**Response (success):** `{ "ok": true, "recipe": "...", "applied": ["lib/settings.dart"] }`

**Common errors:**

| Error | Meaning |
|-------|---------|
| MCP schema: `Missing required property: previewToken` | Apply called without token |
| `Stale previewToken` | File changed after preview — preview again |
| `inlineRecipe requires a YAML-enabled host` | Should not happen for `bin/codemod_mcp.dart` |

## YAML edit steps

Under `edit.steps[]`:

| Step | `anchor` | Behavior |
|------|----------|----------|
| `insert` | **Required** | Insert text at anchor |
| `remove` | Optional | No anchor → full declaration span (incl. doc comment, trailing `;`) |
| `replace` | Optional | No anchor → replace full declaration; idempotent if whitespace-normalized text matches |

Navigate keys in YAML use short forms: `class`, `method`, `field`, `import`, etc. (see `yaml-dsl` skill).

## Agent workflow (with codebase-memory)

1. **Locate** — CBM `get_code_snippet` / `search_graph` (note file + offset)
2. **Impact** — CBM `trace_path` before `remove`
3. **Target** — `generate_ast_path` → `navigate` for `at:`
4. **Preview** — `preview_recipe`
5. **Apply** — `apply_recipe` with `previewToken`
6. **Verify** — CBM `detect_changes` / reindex; re-preview should show no changes

## Implementation

| File | Role |
|------|------|
| `bin/codemod_mcp.dart` | Stdio entrypoint |
| `lib/src/mcp/codemod_mcp_server.dart` | Tool registration (`createCodemodMcpServer`) |
| `lib/src/vscode/codemod_host.dart` | Host protocol |
| `test/mcp/codemod_mcp_test.dart` | MCP integration tests |

## Tests

```bash
# Full MCP protocol integration (in-process + subprocess)
dart test test/mcp/codemod_mcp_test.dart

# Host-level tests (same logic, no MCP wire)
dart test test/vscode/codemod_host_inline_test.dart
dart test test/vscode/generate_ast_path_test.dart

# Everything
dart test
```

Integration tests use:

- **In-process** — `IOStreamTransport` + `McpClient` (fast, no subprocess)
- **Subprocess** — spawns `dart run bin/codemod_mcp.dart` via `StdioClientTransport` (matches production)
