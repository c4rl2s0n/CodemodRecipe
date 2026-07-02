# codemod-mcp Playbook

MCP server **`codemod-mcp`** wraps `CodemodHost.dispatch()` and returns JSON text. Parse every response; branch on `ok`.

**Setup:** `dart run bin/codemod_mcp.dart --workspace-root <root> --codemod-root .codemod`  
**Human doc:** `docs/codemod-mcp.md`

---

## Response envelope

All tools return `jsonEncode(hostResponse)` as a single text blob.

```json
{ "ok": true, ... }
{ "ok": false, "error": "human-readable message" }
```

Success responses may include `_timingsMs` and `_hostMetrics` (ignore for logic).

---

## Tool reference

### `list_recipes`

Discover recipe ids before `describe_recipe` or `preview_recipe`.

| | |
|---|---|
| **Args** | none |
| **Returns** | `recipes[]` (id, name, description, args), `diagnostics[]` |

```json
{ "ok": true, "recipes": [{ "id": "add_log_line", "name": "add_log_line", "args": [...] }], "diagnostics": [] }
```

---

### `describe_recipe`

| | |
|---|---|
| **Args** | `recipe` (string, required) — recipe id from `list_recipes` |
| **Returns** | Full metadata: `args` with `inputKind`, `options`, `contextKey`, template previews |

Use to learn required `args` before preview.

---

### `validate_recipes`

Reload `.codemod/recipes/` and `.codemod/maps/`; report YAML/schema errors.

| | |
|---|---|
| **Args** | none |
| **Returns** | `ok: false` if any diagnostic has `severity: error` |

Call after adding or editing recipe YAML on disk.

---

### `generate_ast_path`

Bridge from **cursor offset** (e.g. from codebase-memory `get_code_snippet`) to inline-recipe `at:` steps.

| | |
|---|---|
| **Args** | `path` (string, absolute path recommended), `offset` (int, 0-based byte offset) |
| **Returns** | `path.navigate[]` with `kind`, `name`, `match`; `path.anchor` for inserts |

**Success:**

```json
{
  "ok": true,
  "path": {
    "navigate": [
      { "kind": "classDecl", "name": "Settings" },
      { "kind": "field", "name": "count" }
    ],
    "anchor": "Anchor(AnchorKind.bodyEnd)",
    "offset": 28
  }
}
```

**Mapping to YAML `at:`:**

| `navigate[].kind` | YAML key |
|-------------------|----------|
| `classDecl` | `class` |
| `method` | `method` |
| `field` | `field` |
| `import` | `import` |
| `function` | `function` |

**Remove / replace:** use `navigate` only — **omit `anchor`**.  
**Insert:** add `anchor` (e.g. `stmt:last`, `body:end`, `member:last`).

---

### `preview_recipe`

Dry-run. **Never skips this before apply.**

Provide **either** `recipe` **or** `inlineRecipe` (not both required, but one must identify the work).

| Arg | Type | Notes |
|-----|------|-------|
| `recipe` | string | Registered recipe id |
| `inlineRecipe` | object | Same shape as `.codemod/recipes/*.yaml` body (needs `id`, `steps`) |
| `args` | object | String values for recipe placeholders / registered recipe args |
| `snippetLines` | number | Patch preview context lines (1–20, default 5) |

**Registered recipe example:**

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

**Inline remove example** (`edit.path` must be absolute):

```json
{
  "inlineRecipe": {
    "id": "__inline_remove_count",
    "steps": [{
      "edit": {
        "path": "/workspace/lib/settings.dart",
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

**Inline insert example:**

```json
{
  "inlineRecipe": {
    "id": "__inline_insert_log",
    "steps": [{
      "edit": {
        "path": "/workspace/lib/settings.dart",
        "steps": [{
          "insert": {
            "at": [
              { "class": "Settings" },
              { "method": "update" }
            ],
            "anchor": "stmt:last",
            "text": "    print('codemod');\n"
          }
        }]
      }
    }]
  }
}
```

**Inline replace example:**

```json
{
  "inlineRecipe": {
    "id": "__inline_replace_count",
    "steps": [{
      "edit": {
        "path": "/workspace/lib/settings.dart",
        "steps": [{
          "replace": {
            "at": [
              { "class": "Settings" },
              { "field": "count" }
            ],
            "text": "  final int count = 99;\n"
          }
        }]
      }
    }]
  }
}
```

**Success:**

```json
{
  "ok": true,
  "recipe": "add_log_line",
  "previewToken": "a1b2c3...",
  "files": [{
    "path": "lib/settings.dart",
    "patches": [{
      "start": 42,
      "end": 42,
      "replacementPreview": "    print('codemod');\n"
    }]
  }]
}
```

**Idempotency:** second preview with same recipe after apply → `files: []` (no patches). Still returns a new `previewToken`.

**Save `previewToken`** for the matching `apply_recipe` call.

---

### `apply_recipe`

| Arg | Type | Notes |
|-----|------|-------|
| `previewToken` | string | **Required** — from immediately preceding preview of same recipe + args |
| `recipe` or `inlineRecipe` | | Must match what was previewed |
| `args` | object | Same args as preview |
| `selection` | object | Optional patch subset (`files[path].include`, `files[path].patches`) |

```json
{
  "recipe": "add_log_line",
  "previewToken": "<from preview>",
  "args": { "file": "lib/settings.dart", "className": "Settings", "methodName": "update" }
}
```

**Success:** `{ "ok": true, "applied": ["lib/settings.dart"] }`

**Failure modes:**

- `Missing previewToken`
- `Stale previewToken` — disk changed; preview again
- `Missing required arguments: <name>`
- Validation errors from inline YAML compile

Apply is **atomic** across files (rollback on write failure). `postExecution` (e.g. `dartFormat`) runs after successful commit.

---

## Agent workflow (codebase-memory + codemod)

```
CBM locate (file, offset)
    → CBM trace_path (impact, especially before remove)
    → generate_ast_path (navigate steps)
    → build inlineRecipe OR pick registered recipe
    → preview_recipe (save previewToken)
    → [user/agent confirms]
    → apply_recipe (same recipe/args + previewToken)
    → CBM verify / re-preview (expect empty files)
```

Do **not** hand-edit Dart when a codemod step can express the change — keeps AST alignment.

---

## YAML edit-step rules

| Step | Sibling of | `anchor` | Default span |
|------|------------|----------|--------------|
| `insert` | `remove`, `replace` under `edit.steps` | Required | Point anchor |
| `remove` | same | Optional | Full declaration (trivia + `;`) |
| `replace` | same | Optional | Full declaration |

**Backward compat:** `insert` may use a single `at` string with embedded anchor: `function:main @ body:end`.

**Limitation:** `{{template}}` in navigate step **names** inside on-disk YAML may not render at apply time. Prefer `generate_ast_path` + inline recipes with concrete names, or pass resolved values via `args` where the recipe supports it.

---

## Paths and workspace

- MCP resolves recipes from `--workspace-root` and `--codemod-root` (default `.codemod`).
- **Inline `edit.path`:** use absolute paths (host does not chdir).
- **Registered recipe `args.file`:** workspace-relative is fine (rendered through templates).

---

## What MCP does not expose

| Host command | MCP tool | Notes |
|--------------|----------|-------|
| `diff` | — | Use extension or host directly for full before/after |
| `reload` | — | `validate_recipes` reloads YAML |

---

## Troubleshooting

| Symptom | Action |
|---------|--------|
| `Unknown recipe` | `list_recipes`; check id spelling |
| `inlineRecipe requires...` | Use `bin/codemod_mcp.dart` with `fromConfig` (default) |
| Empty preview but edit expected | Wrong `at:` path; run `generate_ast_path`; check navigate kinds |
| `Stale previewToken` | Re-preview after any manual edit to target files |
| Patches look wrong | Preview again with higher `snippetLines`; validate `anchor` for insert |

---

## Tests

```bash
dart test test/mcp/codemod_mcp_test.dart
```

See `docs/codemod-mcp.md` for manual testing with MCP Inspector.
