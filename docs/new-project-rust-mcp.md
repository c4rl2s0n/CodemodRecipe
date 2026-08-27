# New Project Quickstart (Rust MCP + Luau)

New here? Start with [Getting Started](getting-started.md) for the full product walkthrough.

This guide shows how to use `codemod_recipe` in a brand-new project with the Rust MCP server and **Luau recipes**.

## 1) Prerequisites

- A target workspace (the project you want to codemod).
- Rust/Cargo and an MCP client (Cursor or MCP Inspector).

## 2) Bootstrap agent skills (recommended)

After connecting the MCP server, call **`bootstrap_project`** once. It installs:

- `.agents/skills/codemod-recipe/` — one parent skill with nested sub-skills:
  - `codemod-recipe` — top-level router (start here)
  - `codemod-recipe/codemod-overview` — orientation and layout
  - `codemod-recipe/codemod-luau-recipes` — recipe module shape and host API (`reference.md`)
  - `codemod-recipe/codemod-recipe-design-patterns` — create vs modify taxonomy
  - `codemod-recipe/codemod-mcp-playbook` — MCP tool reference
  - `codemod-recipe/codemod-languages` — multi-language support
  - `codemod-recipe/codemod-tree-sitter-queries` — query language
  - `codemod-recipe/codemod-recipe-authoring` — tree-sitter query patterns
  - `codemod-recipe/recipe-generation` — generate Luau recipes from `@` code refs
- `.cursor/rules/codemod-recipe.mdc` — always-on orientation
- `.codemod/recipes/`, `.codemod/maps/`, `.codemod/types/`, `.codemod/variables/` scaffolding

```json
{ "force": false, "edit_policy": "recommend", "companions": [] }
```

## 3) Create the `.codemod` layout

```text
.codemod/
  recipes/     # Luau recipe modules (*.luau or *.lua) — convention
  maps/        # Luau map modules — convention
  types/       # Named arg types — convention
  variables/   # Luau variable modules — convention
  queries/     # Luau query libraries — convention
  templates/   # optional (c:template path)
```

The registry identifies every `.luau` and `.lua` module under `.codemod/` by its returned table
shape (`apply`, `map`, `fields`, `values`, `queries`). Directories are conventions, not requirements.
File-backed resources resolve recipe-local first, then `.codemod/`
(`rust/crates/core/src/resource_path.rs`).

## 4) Add your first Luau recipe

Create `.codemod/recipes/add_log_line.luau`:

```lua
return {
  id = "add_log_line",
  name = "Add log line",
  description = "Inserts a log line at the end of a method",

  args = {
    { name = "file", required = true, inputKind = "file" },
    { name = "className", required = true, inputKind = "text" },
    { name = "methodName", required = true, inputKind = "text" },
  },

  postExecution = { "dart format ." },

  apply = function(c)
    c:insert({
      path = c.args.file,
      query = [[
        (class_definition
          name: (identifier) @className
          body: (class_body
            (method_signature
              (function_signature
                name: (identifier) @methodName))
            (function_body
              (block) @body))
          (#eq? @className "]] .. c.args.className .. [[")
          (#eq? @methodName "]] .. c.args.methodName .. [["))
      ]],
      capture = "body",
      anchor = "end",
      text = "    print('codemod');\n",
    })
  end,
}
```

## 5) Optional: create, delete, and composition

```lua
-- create
c:create({ path = "lib/new.dart", content = c:template({ path = "templates/x.template" }) })

-- delete
c:delete({ path = c.args.legacyPath, ifMissing = "skip" })

-- compose
c:run("other.recipe.id", { file = c.args.file })
```

## 6) Start the Rust MCP server

```bash
cargo run -q --manifest-path rust/Cargo.toml -p codemod_recipe_host --bin codemod_mcp -- \
  --workspace-root /absolute/path/to/your/target-workspace \
  --codemod-root .codemod
```

## 7) Run the core MCP workflow

0. `bootstrap_project` (once)
1. `validate_recipes`
2. `list_recipes` / `describe_recipe`
3. `preview_recipe` (save `previewToken`; read `transcript`)
4. `apply_recipe` (same args + token)

## 8) Inline recipe support

Pass Luau source as `inlineRecipe`:

```json
{
  "inlineRecipe": "return { id = \"__inline\", apply = function(c) c:create({ path = \"lib/x.dart\", content = \"// new\\n\" }) end }"
}
```

## 9) Multi-language edits

Set `language = "rust"` (etc.) on edit specs. See [language-support.md](language-support.md).

## 10) Validation checklist

- `validate_recipes` returns `ok: true`
- `preview_recipe` returns a `previewToken`
- `apply_recipe` succeeds with that token
- Re-preview produces no files if idempotent
