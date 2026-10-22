# Core concepts

## Recipes

A recipe is a Luau module:

```lua
return {
  id = "add_log_line",
  name = "Add log line",
  args = {
    { name = "file", required = true, inputKind = "file" },
    { name = "className", required = true, inputKind = "text" },
    { name = "methodName", required = true, inputKind = "text" },
  },
  postExecution = { "dart format ." },
  apply = function(c, args)
    c:insert({
      path = args.file,
      query = [[
        (class_definition
          name: (identifier) @className
          body: (class_body
            (method_signature
              (function_signature
                name: (identifier) @methodName))
            (function_body
              (block) @body))
          (#eq? @className "]] .. args.className .. [[")
          (#eq? @methodName "]] .. args.methodName .. [["))
      ]],
      capture = "body",
      anchor = "end",
      text = "    print('codemod');\n",
    })
  end,
}
```

**`apply(c, args)` is the only mutation entrypoint.** Use Luau `if` / `for` in `apply` for
guards and idempotency.

Args are JSON values, not only strings: builtins are `text`, `bool`, `choice`,
`file`, and `directory`. A registered type id is one object; `{ "TypeId" }` is a
list (VS Code repeats a field group). Details: [writing-recipes.md](../02_writing_recipes/01_workflow/00_index.md).

### How an insert finds its place

Three fields work together on `c:insert`:

1. **`query`** — tree-sitter pattern. Names like `@body` label matched nodes.
2. **`capture`** — which named match is the **edit target** (without `@`).
3. **`anchor`** — insert **before** (`start`) or **after** (`end`) that span:

```text
  …[===== @body span =====]…
     ^                     ^
  anchor: start         anchor: end
```

How to write recipes end to end: [writing-recipes.md](../02_writing_recipes/01_workflow/00_index.md).
Query language detail: [tree-sitter-queries.md](../03_queries_and_languages/01_tree_sitter_queries/00_index.md).

Optional `postExecution` (or `c:postExec` inside `apply`) runs shell commands after
a successful apply. Preview records these in the transcript but does not run them.

## Host API (summary)

| Method | Role |
|--------|------|
| `args` (2nd param to `apply`) | Resolved recipe arguments — **not** on `c` |
| `c:insert` / `c:replace` / `c:remove` | AST patches |
| `c:query` / `c:read` | Read-only inspection |
| `c:create` / `c:delete` | File create/delete |
| `c:run(id, args?)` | Call another recipe |
| `c:postExec` | Shell command (apply only) |
| `c:template` | MiniJinja (`inline` or `path`) |
| `c:log` | Append to preview/apply transcript |
| `c:map` / `c:var` / `c:queryLib` | Registered assets |

Full API: [writing-recipes.md](../02_writing_recipes/01_workflow/00_index.md) and [host-api.md](../02_writing_recipes/02_host_api/00_index.md).

## Preview vs apply

- **Preview** runs `apply()` against an in-memory **WorkingTree**; builds diffs + transcript.
- **Apply** **re-runs** `apply()` from original file hashes, flushes to disk, then runs postExec.
- **`previewToken`** binds recipe id/source, args, and starting file hashes.

## Tree-sitter queries

Ops find nodes with tree-sitter query S-expressions. Queries can be inline strings,
paths to `.scm` files, or ids from query libraries under `.codemod/queries/`.

Debug with **Query Tools** (VS Code) or MCP `dump_ast` / `debug_query` before
`preview_recipe`.

## Templates (MiniJinja)

String templating uses [MiniJinja](https://docs.rs/minijinja/) via `c:template`
only (`inline` string or file `path`).

```lua
local path = c:template({ inline = "lib/{{ feature | snake_case }}.dart" })
```

Full guide: [recipe-templates.md](../04_templates_and_patterns/01_recipe_templates/00_index.md).

## Maps, variables, types, and query libraries

Shared assets are Luau modules (`*.luau` or `*.lua`) anywhere under `.codemod/`. A top-level **`id`** marks asset intent; the registry classifies by shape (`apply`, `parse`+`verify`, `map`, …). Modules **without** `id` are libraries (not catalogued) and may be `require`d — see [luau-lsp.md](../05_components_and_stacks/04_luau_lsp/04_local_and_shared_types.md). Directories below are conventions, not requirements:

| Kind | Conventional path | Return shape |
|------|-------------------|--------------|
| Maps | `.codemod/maps/` | `{ id, map = { ... } }` |
| Types | `.codemod/types/` | `{ id, fields = { ... } }` |
| Variables | `.codemod/variables/` | `{ id, values = { ... } }` |
| Query libs | `.codemod/queries/` | `{ id, queries = { ... } }` |

Lookup in recipes: `c:map("id")`, `c:var("id")`, `c:queryLib("id")`.

## Composition

Call other recipes from `apply`:

```lua
c:run("dart.settings.add_counter_field")
c:run("dart.logging.add_log_line", { file = args.file })
```

Or pass a full module as MCP **`inlineRecipe`** (Luau source string, no file on disk).

## Languages

Default language is Dart. For other files, set `language = "rust"` (etc.) on the
edit spec. `.sql` defaults to `sqlite`.

Details: [language-support.md](../03_queries_and_languages/02_language_support/00_index.md).

## Naming and design patterns

| Prefix | Purpose |
|--------|---------|
| `create_` | Greenfield files from templates |
| `add_` / `patch_` | Brownfield AST edits (one change each) |
| `scaffold_` | Feature workflow (`c:run` atomics) |
| `remove_` | Tear down members or files |

**One recipe = one coherent change.** Summary: [recipe-design-patterns.md](../04_templates_and_patterns/02_recipe_design_patterns/00_index.md).
