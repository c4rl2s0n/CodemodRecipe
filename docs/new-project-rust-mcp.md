# New Project Quickstart (Rust MCP + YAML)

This guide shows how to use `codemod_recipe` in a brand-new project with the Rust MCP server and YAML recipes.

## 1) Prerequisites

- You have a target workspace (the project you want to codemod).
- You can run Rust/Cargo.
- You can connect an MCP client (Cursor or MCP Inspector).

## 2) Bootstrap agent skills (recommended)

After connecting the MCP server, call **`bootstrap_project`** once. It installs:

- `.agents/skills/` — modular skills:
  - `codemod-overview` — orientation and layout
  - `codemod-yaml-dsl` — YAML syntax (`reference.md` for templates/maps)
  - `codemod-recipe-design-patterns` — create vs modify taxonomy (`reference.md`)
  - `codemod-mcp-playbook` — MCP tool reference
  - `codemod-languages` — multi-language support (`reference.md`)
  - `codemod-tree-sitter-queries` — query language (`reference.md`)
  - `codemod-recipe-authoring` — tree-sitter queries
  - `recipe-generation` — generate YAML recipes from `@` code refs
- `.cursor/rules/codemod-recipe.mdc` — always-on orientation (soft by default)
- `.codemod/recipes/`, `.codemod/maps/`, and `.codemod/variables/` scaffolding

```json
{ "force": false, "edit_policy": "recommend", "companions": [] }
```

- `edit_policy`: `"recommend"` (default) soft prefer+preview, or `"strict"` recipe-first (in-body direct edit only; discuss before new recipes/templates).
- `companions`: optional packs such as `["codebase-memory"]` when codebase-memory-mcp is configured (MCP-first navigation rule). Independent of `edit_policy`.
- Set `"force": true` to overwrite existing bootstrap files when switching packs.

Response echoes the profile: `{ "ok": true, "edit_policy": "...", "companions": [...], "written": [...], "skipped": [...] }`.

## 3) Create the `.codemod` layout

In your target workspace:

```text
.codemod/
  recipes/     # recommended for recipes (*.yaml with steps)
  maps/        # recommended for maps (id + map:)
  variables/   # recommended for variables (id + values:)
  templates/   # optional (for create.templateFile)
```

Discovery is schema-based under `.codemod/` (directory names are convention only).

- Put recipes in `.codemod/recipes/*.yaml` (or elsewhere under `.codemod/` if they have `steps`).
- Put shared maps as YAML with `id` + `map:` (e.g. `.codemod/maps/*.yaml`).
- Put shared variables as YAML with `id` + `values:` (e.g. `.codemod/variables/*.yaml`).

## 4) Add your first YAML recipe

Create `.codemod/recipes/add_log_line.yaml`:

```yaml
id: add_log_line
name: add_log_line
description: Inserts a log line at the end of a method

args:
  - name: file
    required: true
    inputKind: file
  - name: className
    required: true
    inputKind: symbol
  - name: methodName
    required: true
    inputKind: symbol

steps:
  - edit:
      path: "{{file}}"
      ops:
        - insert:
            query: |
              (class_definition
                name: (identifier) @className
                body: (class_body
                  (method_signature
                    (function_signature
                      name: (identifier) @methodName))
                  (function_body
                    (block) @body))
                (#eq? @className "{{className}}")
                (#eq? @methodName "{{methodName}}"))
            capture: body
            anchor: end
            text: "    print('codemod');\n"

postExecution:
  - "dart format ."
```

Reference recipes:
- [`.codemod/recipes/add_log_line.yaml`](../.codemod/recipes/add_log_line.yaml)
- [`.codemod/recipes/add_counter_field.yaml`](../.codemod/recipes/add_counter_field.yaml)

## 5) Optional: create and delete steps

Create-file example (`templateFile`):
- [`test/fixtures/scaffold_project/.codemod/recipes/create_repository.yaml`](../test/fixtures/scaffold_project/.codemod/recipes/create_repository.yaml)

Delete-file example (`ifMissing`):
- [`test/fixtures/rust_oracle/delete_legacy.recipe.yaml`](../test/fixtures/rust_oracle/delete_legacy.recipe.yaml)

## 6) Start the Rust MCP server

From this repository:

```bash
cargo run -q --manifest-path rust/Cargo.toml -p codemod_recipe_host --bin codemod_mcp -- \
  --workspace-root /absolute/path/to/your/target-workspace \
  --codemod-root .codemod
```

The process runs as an MCP stdio server, so it appears idle when healthy.

## 7) Run the core MCP workflow

Use Cursor or MCP Inspector and call tools in this order:

0. `bootstrap_project` (once, if not done yet)
1. `validate_recipes` (after editing YAML)
2. `list_recipes`
3. `describe_recipe`
4. `preview_recipe` (save `previewToken`)
5. `apply_recipe` (must use the same `previewToken`)

`selection` is optional on `apply_recipe` when you want partial patch apply.

## 8) Registered recipe request examples

Preview:

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

Apply:

```json
{
  "recipe": "add_log_line",
  "args": {
    "file": "lib/settings.dart",
    "className": "Settings",
    "methodName": "update"
  },
  "previewToken": "<from preview_recipe>"
}
```

## 9) Inline recipe support (Rust, v2 shape)

You can skip on-disk recipes and pass `inlineRecipe` directly:

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

`edit.ops` supports `insert`, `replace`, and `remove`:
- `insert`: requires `query`, `capture`, `anchor` (`start` or `end`), `text`
- `replace`: requires `query`, `capture`, `text` (`includeLeadingTrivia` optional)
- `remove`: requires `query`, `capture` (`includeLeadingTrivia` optional)

## 10) Multi-language edits

For non-Dart files, set `language:` on `edit` steps and write queries for that grammar’s node names. `.sql` paths default to `sqlite`.

See [language-support.md](language-support.md) and agent skill `codemod-languages` (installed via `bootstrap_project`).

## 11) Validation checklist

- `validate_recipes` returns `ok: true`.
- `preview_recipe` returns a `previewToken`.
- `apply_recipe` succeeds with that token.
- Re-running the same preview should produce no files if the recipe is idempotent.
