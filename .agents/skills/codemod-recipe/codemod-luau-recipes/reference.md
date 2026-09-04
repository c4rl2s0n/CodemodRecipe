# Luau Recipe Reference

Recipes are **sandboxed Luau** modules (`.luau` or `.lua`) anywhere under `.codemod/`.
They `return` a table; **`apply = function(c, args)`** is the only mutation entrypoint and the
registry uses it to identify a recipe. The `.codemod/recipes/` directory is a convention, not a
requirement.

Rust implementation: `rust/crates/lua/src/load.rs` (parse/validate), `rust/crates/host/src/host_api/handlers.rs` (host API source of truth), `rust/crates/host/src/host_api/registry.rs` + `lua_runtime.rs` (runtime binding).

## Module return table

| Field | Required | Role |
|-------|----------|------|
| `id` | yes | Unique recipe id (dotted ids nest in VS Code Recipes tab) |
| `apply` | yes | `function(c)` — all file mutations |
| `name` | no | Display name |
| `description` | no | Short description |
| `args` | no | List of arg definitions (see below) |
| `maps` | no | Recipe-local map tables `{ mapId = { key = "value" } }` |
| `types` | no | Recipe-local named types `{ TypeId = { fields = { ... } } }` |
| `queries` | no | Inline query strings `{ queryId = "(...) @cap" }` |
| `postExecution` | no | Shell commands after successful apply (Jinja-rendered) |
| `explorerMenu` | no | VS Code Explorer submenu entries |

## Arguments (`args`)

```lua
args = {
  {
    name = "file",
    required = true,
    inputKind = "file",       -- text | bool | choice | file | directory | TypeId | { TypeId }
    defaultsTo = "lib/foo.dart",
    help = "Target Dart file",
    abbr = "f",
    options = { "a", "b" },   -- choice = closed list; text = suggestions
    optionsFromMap = "column_type",
    from = "activeFile",      -- or a table for query-based derivation
  },
}
```

`inputKind` is the type. `{ "ClassProperty" }` means a list. Named types live in
Luau modules anywhere under `.codemod/` that return `{ id, fields }`, or as recipe-local `types`.
The `.codemod/types/` directory is a convention, not a requirement. Nested lists belong on a
type field, not as `{{ "text" }}`.

Arg defaults use `defaultsTo`. Unknown keys on an arg entry fail validation.

## Host context (`c`)

### `args`

Resolved recipe arguments: strings, bools, lists, and objects (`args.file`, `args.properties[1].name`).

### AST edits

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

### Read-only query helpers

```lua
local hits = c:query({ path = args.file, query = "...", capture = "body" })
-- hits[i] = { span = { from = byte, to = byte }, text = "...", kind = "..." }

local rows = c:queryMatches({ path = args.file, query = "..." })
-- rows[i].captureName = hit or { hit, hit, ... }

if c:queryExists({ path = args.file, query = "...", capture = "body" }) then ... end
local n = c:queryCount({ path = args.file, query = "..." })  -- optional capture
c:expectMatch({ ... })   -- error if zero matches
c:expectNoMatch({ ... }) -- error if any match
```

### Path helpers

```lua
local paths = c:listFiles("lib/models/*.dart")
local rows = c:matchFiles("lib/stores/{id}_store.dart")  -- { path, id }
-- resolve = {key} fill-in only (no MiniJinja). Prefer over template for paths.
local one = c:resolve("lib/stores/{id}_store.dart", { id = "recipe" })
local name = c:filename("lib/models/recipe.dart")  -- "recipe"
```

### File I/O

```lua
local source = c:read(args.file)  -- or c:read({ path = "..." })

c:create({
  path = "lib/new.dart",
  content = "class X {}\n",   -- or template = "..." (alias)
  ifExists = "fail",            -- or "skip"
})

c:delete({ path = "lib/legacy.dart", ifMissing = "skip" })  -- or "fail"
```

### Composition

```lua
c:run("dart.settings.add_counter_field")
c:run("dart.logging.add_log_line", { file = args.file, className = "Settings" })
```

Child recipe `postExecution` runs inline; parent args are restored afterward.

### Templates (MiniJinja)

```lua
local text = c:template({
  inline = "class {{ className | pascal_case }} {}\n",
  env = { className = args.className },  -- optional extra vars
})

local fileBody = c:template({
  path = "templates/widget.dart.template",
})
```

### Logging

```lua
c:log("exports", args.exports)
```

Appends a `log` event to the recipe transcript (visible in preview even when later ops fail).

MiniJinja has access to recipe args plus registered maps/variables (`c:map`, `c:var`).
See [docs/recipe-templates.md](../../../../docs/recipe-templates.md).

### Asset lookups

Shared assets are Luau modules:

| Conventional location | Identified by | Return shape |
|-----------------------|-------------|--------------|
| `.codemod/maps/*.luau` | `map` table | `{ id = "...", map = { key = "value" } }` |
| `.codemod/variables/*.luau` | `values` table | `{ id = "...", values = { key = "value" } }` |
| `.codemod/queries/*.luau` | `queries` table | `{ id = "...", queries = { name = "(...) @x" } }` |

These files can live anywhere under `.codemod/`; the registry classifies them by their returned
table shape, not by directory.

```lua
local m = c:map("column_type")
local v = c:var("defaults")
local q = c:queryLib("dart")
```

Recipe-local `maps` / `queries` on the return table merge with registered assets.

### Post-execution

```lua
-- In apply (runs on apply only; transcript entry on preview):
c:postExec({ command = "dart format {{file}}" })

-- Or declare top-level (Jinja-rendered after apply):
postExecution = { "dart format ." },
```

On **preview**, postExec is recorded in the transcript with `includedInPreview: false`.
On **apply**, the WorkingTree flushes to disk before each postExec / postExecution command.

### Assertions

```lua
c:assert(condition, "optional message")
```

## Preview vs apply semantics

| Phase | Behavior |
|-------|----------|
| Preview | Runs `apply()` against WorkingTree; skips postExec; builds diff + transcript |
| Apply | Re-runs `apply()` from original file hashes; flushes; runs postExec |
| Token | `previewToken` = hash(recipe + args + starting file contents) |

When preview skipped postExec, apply output may differ from preview patches — the VS Code
review panel surfaces **Files** and **Transcript** views and an `applyMayDiverge` disclaimer.

## Tree-sitter queries

Queries stay as **inline strings** or **`.scm` files** (path resolved recipe-local, then `.codemod/`).
Query libraries are Luau modules (conventionally under `.codemod/queries/`) that return
`{ id = "...", queries = { ... } }`.

See skill `codemod-tree-sitter-queries`.

## Path resolution

Centralized in `rust/crates/core/src/resource_path.rs`:

- Workspace mutation targets (`path` on edits/create/delete) resolve under workspace root only.
- Recipe resources (`.scm`, template files, postExecution scripts) resolve recipe-local first, then `.codemod/`.
- Bare query names also try `queries/` under each root.

## Inline recipes (MCP / host)

`inlineRecipe` is a **Luau source string** that `return`s the same table shape (must include `apply`).

```json
{
  "inlineRecipe": "return {\n  id = \"__inline\",\n  apply = function(c, args)\n    c:insert({ path = \"lib/a.dart\", query = \"(program) @root\", capture = \"root\", anchor = \"end\", text = \"// x\\n\" })\n  end,\n}"
}
```

## Validation

`validate_recipes` reloads all Luau under `.codemod/` (recipes, maps, variables, query libs),
checks required fields, and verifies `apply` is present.

## Control flow

Use Luau `if` / `for` inside `apply`. MiniJinja is only for `c:template`.

## Maintainer notes

When changing recipe loading or host API:

- `rust/crates/lua/src/load.rs` — parse recipe/map/variable/query-lib tables
- `rust/crates/host/src/host_api/handlers.rs` — `c:` method implementations (source of truth)
- `rust/crates/host/src/host_api/registry.rs` + `lua_runtime.rs` — runtime binding
- `rust/crates/host/src/protocol_keys.rs` — transport keys (`inlineRecipe`, `previewToken`, …)
