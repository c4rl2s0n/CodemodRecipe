# New Project Quickstart (Rust MCP + YAML v2)

This guide shows how to use `codemod_recipe` in a brand-new project with the Rust MCP server and YAML `dslVersion: 2` recipes.

## 1) Prerequisites

- You have a target workspace (the project you want to codemod).
- You can run Rust/Cargo.
- You can connect an MCP client (Cursor or MCP Inspector).

## 2) Bootstrap agent skills (recommended)

After connecting the MCP server, call **`bootstrap_project`** once. It installs:

- `.agents/skills/` — five modular skills:
  - `codemod-overview` — orientation and layout
  - `codemod-yaml-dsl-v2` — YAML syntax (`reference.md` for templates/maps)
  - `codemod-recipe-design-patterns` — create vs modify taxonomy (`reference.md`)
  - `codemod-mcp-playbook` — MCP tool reference
  - `codemod-recipe-authoring` — tree-sitter queries
- `.cursor/rules/codemod-recipe.mdc` — always-on orientation + codebase-memory hint
- `.codemod/recipes/` and `.codemod/maps/` scaffolding

```json
{ "force": false }
```

Set `"force": true` to overwrite existing bootstrap files.

## 3) Create the `.codemod` layout

In your target workspace:

```text
.codemod/
  recipes/
  maps/        # optional
  templates/   # optional (for create.templateFile)
```

- Put registered recipes under `.codemod/recipes/*.yaml`.
- Put shared maps under `.codemod/maps/*.yaml` when needed.

## 4) Add your first YAML v2 recipe

Create `.codemod/recipes/add_log_line.yaml`:

```yaml
dslVersion: 2
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
              (class_declaration
                name: (identifier) @className
                body: (class_body
                  (class_member
                    (method_signature
                      (function_signature
                        name: (identifier) @methodName))
                    (function_body
                      (block) @body)))
                (#eq? @className "{{className}}")
                (#eq? @methodName "{{methodName}}"))
            capture: body
            anchor: end
            text: "    print('codemod');\n"

postExecution:
  - dartFormat
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

`edit.ops` supports `insert`, `replace`, and `remove`:
- `insert`: requires `query`, `capture`, `anchor` (`start` or `end`), `text`
- `replace`: requires `query`, `capture`, `text` (`includeLeadingTrivia` optional)
- `remove`: requires `query`, `capture` (`includeLeadingTrivia` optional)

## 10) Validation checklist

- `validate_recipes` returns `ok: true`.
- `preview_recipe` returns a `previewToken`.
- `apply_recipe` succeeds with that token.
- Re-running the same preview should produce no files if the recipe is idempotent.
