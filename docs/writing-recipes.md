# Writing recipes

How to author **Luau recipes** for codemod-recipe: the usual workflow, what
`query` / `capture` / `anchor` mean, the host API, and where to go next.

Day-1 product overview: [getting-started.md](getting-started.md).
Full host API: skill `codemod-luau-recipes` → `reference.md`.

## Authoring workflow

1. **Pick the target** — which file and AST node should change (method body,
   class member, import, …). Inspect the tree in Query Tools or iterate with preview.
2. **Write a query** — S-expression pattern with `@names` for nodes you filter
   on and the node you will edit.
3. **Set `capture`** — the `@name` whose **text span** the op edits (omit the
   `@` in the spec: `@body` → `capture = "body"`).
4. **For `c:insert` only** — set `anchor = "start"` or `"end"`.
5. **Use `c.args`** — declare dynamics as recipe args; build query/text strings in Luau
   or render with `c:applyTemplate`.
6. **Validate → preview → apply** — then re-preview; an idempotent recipe
   returns no patches.

Dart is the default language. For other files, set `language` on the edit spec —
see [language-support.md](language-support.md).

## Recipe shape

Recipes are Luau modules (`.luau` or `.lua`) under `.codemod/` that **return a table** with `id` and `apply = function(c)`. The `.codemod/recipes/` directory is a convention, not a requirement — the registry identifies a recipe by the `apply` function in its returned table.

```lua
return {
  id = "feature.area.recipe_id",  -- dotted ids nest in VS Code Recipes tab
  name = "Human-readable name",   -- optional
  description = "What it does",   -- optional
  args = {},                      -- optional parameters
  maps = {},                      -- optional recipe-local maps
  queries = {},                   -- optional inline query ids
  postExecution = { "dart format ." },  -- optional; apply only
  explorerMenu = {},              -- optional Explorer submenu
  apply = function(c)
    -- all mutations here
  end,
}
```

**`apply(c)` is the only mutation entrypoint.** Use Luau `if`/`for` for guards and
idempotency checks (e.g. `c:queryExists` before insert).

### Arguments (`args`)

```lua
args = {
  { name = "file", required = true, inputKind = "file" },
  { name = "className", required = true, inputKind = "text" },
  { name = "includeTests", inputKind = "bool", defaultsTo = "false" },
}
```

Builtin `inputKind` values: `text`, `bool`, `choice`, `file`, `directory`.
A registered type id (from a Luau module under `.codemod/` that returns `{ id, fields = { ... } }`) is one object; `{ "TypeId" }` is a
list of that type. `choice` needs `options` or `optionsFromMap`. `text` with options
is a combobox (suggestions plus free typing). `symbol` still loads as `text`.

```lua
-- .codemod/types/class_property.luau (conventional location)
return {
  id = "ClassProperty",
  fields = {
    { name = "type", inputKind = "text", required = true },
    { name = "name", inputKind = "text", required = true },
    { name = "initialValue", inputKind = "text" },
  },
}
```

```lua
args = {
  { name = "properties", inputKind = { "ClassProperty" } },
}
-- in apply:
for _, p in ipairs(c.args.properties or {}) do
  c:run("add_property", { type = p.type, name = p.name })
end
```
Defaults use `defaultsTo`. In VS Code, `from` can derive args from the editor —
see [recipe-shortcuts.md](recipe-shortcuts.md).

## Host API

Access arguments as `c.args.name`. All mutation methods are on `c`:

### AST edits

```lua
c:insert({
  path = c.args.file,
  language = "dart",       -- optional when extension is unambiguous
  query = "(class_definition ...) @body",
  capture = "body",
  anchor = "end",          -- "start" | "end" (default "end")
  text = "    print('codemod');\n",
})

c:replace({ path = c.args.file, query = "...", capture = "member", text = "  final int x = 0;\n" })
c:remove({ path = c.args.file, query = "...", capture = "member", includeLeadingTrivia = true })
```

### File create/delete

```lua
c:create({
  path = "lib/new_file.dart",
  content = c:renderFileTemplate({ path = "templates/widget.dart.template" }),
  ifExists = "fail",   -- or "skip"
})

c:delete({ path = "lib/legacy.dart", ifMissing = "skip" })
```

### Read-only helpers

```lua
local text = c:read(c.args.file)
local spans = c:query({ path = c.args.file, query = "...", capture = "body" })
if c:queryExists({ path = c.args.file, query = "..." }) then return end
```

### Composition

```lua
c:run("dart.settings.add_counter_field")
c:run("dart.logging.add_log_line", { file = c.args.file, className = c.args.className })
```

### Templates

```lua
local rendered = c:applyTemplate({
  template = "class {{ name | pascal_case }} {}\n",
  env = { name = c.args.className },
})
```

See [recipe-templates.md](recipe-templates.md).

### Post-execution

Top-level `postExecution` or inline:

```lua
c:postExec({ command = "dart format {{file}}" })
```

Preview records postExec in the **transcript** but does not run it. Apply flushes
the WorkingTree first, then executes commands.

## Query, capture, and anchor

```text
source file  →  parse  →  query match  →  capture span  →  patch
```

### Captures in the query

```scheme
(class_definition
  name: (identifier) @className
  body: (class_body
    (function_body (block) @body))
  (#eq? @className "Settings"))
```

### The `capture` field

On the op spec, `capture` picks **which** named match is edited (name without `@`).

### The `anchor` field (insert only)

```text
  …[===== capture span =====]…
     ^                       ^
  anchor: start           anchor: end
```

Full query language: [tree-sitter-queries.md](tree-sitter-queries.md).

## Worked insert example

```lua
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
```

## Idempotency

Prefer recipes that produce **no patches** on a second apply:

- **replace** — idempotent when `text` is already the desired final state
- **remove** — idempotent when the target is already gone
- **insert** — guard with `c:queryExists` / `c:queryCount` or a query that only
  matches when the snippet is absent

```lua
if c:queryExists({ path = c.args.file, query = alreadyThereQuery }) then
  return
end
c:insert({ ... })
```

## Maps, variables, query libraries

Shared Luau modules for maps, variables, and query libraries. They can live anywhere under `.codemod/` and are identified by their returned table shape (`map`, `values`, or `queries`). The `.codemod/maps/`, `.codemod/variables/`, and `.codemod/queries/` directories are conventions, not requirements:

```lua
local types = c:map("column_type")
local defaults = c:var("project_defaults")
local q = c:queryLib("dart")
```

## Naming and organization

| Prefix | Purpose |
|--------|---------|
| `create_` | Greenfield files from templates |
| `add_` / `patch_` | Brownfield AST edits (one change each) |
| `scaffold_` | Feature workflow (`c:run` atomics) |
| `remove_` | Tear down members or files |

Details: [recipe-design-patterns.md](recipe-design-patterns.md).

## Validate and run

| Client | Flow |
|--------|------|
| VS Code | Recipes tab → fill args → review Files + Transcript → Apply Selected |
| MCP | `validate_recipes` → `preview_recipe` → `apply_recipe` with `previewToken` |

See [getting-started.md](getting-started.md#day-1-success-run-a-recipe) and
[codemod-mcp.md](codemod-mcp.md).

## Inline recipes (MCP)

`inlineRecipe` is a **Luau source string** that `return`s the same table (must include `apply`):

```json
{
  "inlineRecipe": "return { id = \"__inline\", apply = function(c) c:insert({ path = \"lib/a.dart\", query = \"(program) @root\", capture = \"root\", anchor = \"end\", text = \"// x\\n\" }) end }"
}
```

## Further reading

| Doc / skill | Contents |
|-------------|----------|
| `codemod-luau-recipes` → `reference.md` | Complete host API |
| [tree-sitter-queries.md](tree-sitter-queries.md) | Query syntax, captures, predicates |
| [recipe-templates.md](recipe-templates.md) | MiniJinja, maps, variables |
| [language-support.md](language-support.md) | `language` and grammars |
| [recipe-design-patterns.md](recipe-design-patterns.md) | create / add / scaffold taxonomy |
| [recipe-shortcuts.md](recipe-shortcuts.md) | Slots, `from`, explorer menu |
