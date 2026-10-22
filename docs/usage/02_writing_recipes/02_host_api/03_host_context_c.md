# Host context (`c`)

## `args`

Resolved recipe arguments: strings, bools, lists, and objects (`args.file`, `args.properties[1].name`).

## AST edits

All edit methods take a **spec table**:

| Field | insert | replace | remove |
|-------|--------|---------|--------|
| `path` | required | required | required |
| `query` | required | required | required |
| `capture` | required | required | required |
| `text` | required | required | — |
| `anchor` | `"start"` \| `"end"` (default `"end"`) | — | — |
| `language` | optional | optional | optional |
| `includeLeadingTrivia` | — | optional bool | optional bool |

**`query`** may be:

- An inline tree-sitter query string
- A registered query id (from recipe `queries`, `c:queryLib`, or `.codemod/queries/*.luau`)
- A list of query strings (chained evaluation)

```lua
c:insert({
  path = args.file,
  language = "dart",
  query = [[
    (class_definition
      name: (identifier) @className
      body: (class_body
        (function_body (block) @body))
      (#eq? @className "]] .. args.className .. [["))
  ]],
  capture = "body",
  anchor = "end",
  text = "    print('codemod');\n",
})

c:replace({ path = args.file, query = "...", capture = "member", text = "  final int count = 0;\n" })
c:remove({ path = args.file, query = "...", capture = "member" })
```

## Read-only query helpers

| Method | Use when |
|--------|----------|
| `c:query` + `capture` | Flat list of hits for **one** capture (most recipes/components) |
| `c:queryMatches` | One row per match with **all** captures together (e.g. name + type + default) |

```lua
local hits = c:query({ path = args.file, query = "...", capture = "body" })
-- hits[i] = { span = { from = byte, to = byte }, text = "...", kind = "..." }
-- Prefer hit.text — spans are 0-based bytes; do not source:sub(from+1, to) yourself.

local rows = c:queryMatches({ path = args.file, query = "..." })
-- rows[i].captureName = hit or { hit, hit, ... }

if c:queryExists({ path = args.file, query = "...", capture = "body" }) then ... end
local n = c:queryCount({ path = args.file, query = "..." })  -- optional capture
c:expectMatch({ ... })   -- error if zero matches
c:expectNoMatch({ ... }) -- error if any match
```

## Path helpers

```lua
local paths = c:listFiles("lib/models/*.dart")
local rows = c:matchFiles("lib/stores/{id}_store.dart")  -- { path, id }
-- resolve = {key} fill-in only (no MiniJinja). Prefer over template for paths.
local one = c:resolve("lib/stores/{id}_store.dart", { id = "recipe" })
local name = c:filename("lib/models/recipe.dart")  -- "recipe"
```

## File I/O

```lua
local source = c:read(args.file)  -- or c:read({ path = "..." })

-- Arbitrary 0-based byte span (when you don't already have a query hit):
local text = c:spanText({ path = args.file, span = { from = 10, to = 20 } })

c:create({
  path = "lib/new.dart",
  content = "class X {}\n",   -- or template = "..." (alias)
  ifExists = "fail",            -- or "skip"
})

c:delete({ path = "lib/legacy.dart", ifMissing = "skip" })  -- or "fail"
```

## Composition

```lua
c:run("dart.settings.add_counter_field")
c:run("dart.logging.add_log_line", { file = args.file, className = "Settings" })
```

Child recipe `postExecution` runs inline; parent args are restored afterward.

## Templates (MiniJinja)

```lua
local text = c:template({
  inline = "class {{ className | pascal_case }} {}\n",
  env = { className = args.className },  -- optional extra vars
})

local fileBody = c:template({
  path = "templates/widget.dart.template",
})
```

## Logging

```lua
c:log("exports", args.exports)
```

Appends a `log` event to the transcript. Visible on **`preview_recipe`** / **`component_dry_run_hook`** (and apply) as `transcript` plus top-level **`logs`** (string array of `c:log` details). **`component_scan` has no transcript/logs** — dry-run the hook to debug.

MiniJinja has access to recipe args plus registered maps/variables (`c:map`, `c:var`).
See [docs/recipe-templates.md](../../04_templates_and_patterns/01_recipe_templates/00_index.md).

## Asset lookups

Shared assets are Luau modules:

| Conventional location | Identified by | Return shape |
|-----------------------|-------------|--------------|
| `.codemod/maps/*.luau` | `map` table | `{ id = "...", map = { key = "value" } }` |
| `.codemod/variables/*.luau` | `values` table | `{ id = "...", values = { key = "value" } }` |
| `.codemod/queries/*.luau` | `queries` table | `{ id = "...", queries = { name = "(...) @x" } }` |

These files can live anywhere under `.codemod/`; the registry classifies them by top-level
`id` plus returned table shape, not by directory. Modules **without** `id` are libraries
(not catalogued); `require` them from recipes/components under `.codemod/`
([docs/luau-lsp.md](../../05_components_and_stacks/04_luau_lsp/04_local_and_shared_types.md)).
`.d.luau` stubs remain IDE-only.

```lua
local m = c:map("column_type")
local v = c:var("defaults")
local q = c:queryLib("dart")
```

Recipe-local `maps` / `queries` on the return table merge with registered assets.

## Post-execution

```lua
-- In apply (runs on apply only; transcript entry on preview):
c:postExec({ command = "dart format {{file}}" })

-- Or declare top-level (Jinja-rendered after apply):
postExecution = { "dart format ." },
```

On **preview**, postExec is recorded in the transcript with `includedInPreview: false`.
On **apply**, the WorkingTree flushes to disk before each postExec / postExecution command.

## Assertions

```lua
c:assert(condition, "optional message")
```
