# Luau Recipe Reference

Recipes are **sandboxed Luau** modules under `.codemod/recipes/*.luau` (also `.lua`).
They `return` a table; **`apply = function(c)`** is the only mutation entrypoint.

Rust implementation: `rust/crates/lua/src/load.rs` (parse/validate), `rust/crates/host/src/lua_runtime.rs` (host API).

## Module return table

| Field | Required | Role |
|-------|----------|------|
| `id` | yes | Unique recipe id (dotted ids nest in VS Code Recipes tab) |
| `apply` | yes | `function(c)` — all file mutations |
| `name` | no | Display name |
| `description` | no | Short description |
| `args` | no | List of arg definitions (see below) |
| `maps` | no | Recipe-local map tables `{ mapId = { key = "value" } }` |
| `queries` | no | Inline query strings `{ queryId = "(...) @cap" }` |
| `postExecution` | no | Shell commands after successful apply (Jinja-rendered) |
| `explorerMenu` | no | VS Code Explorer submenu entries |

## Arguments (`args`)

```lua
args = {
  {
    name = "file",
    required = true,
    inputKind = "file",       -- text | file | directory | choice | symbol
    defaultsTo = "lib/foo.dart",
    help = "Target Dart file",
    abbr = "f",
    options = { "a", "b" },   -- for choice
    from = "activeFile",      -- or a table for query-based derivation
  },
}
```

Arg defaults use `defaultsTo`. Unknown keys on an arg entry fail validation.

## Host context (`c`)

### `c.args`

String map of resolved recipe arguments. Access as `c.args.file`, `c.args.className`, etc.

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
  path = c.args.file,
  language = "dart",
  query = [[
    (class_definition
      name: (identifier) @className
      body: (class_body
        (function_body (block) @body))
      (#eq? @className "]] .. c.args.className .. [["))
  ]],
  capture = "body",
  anchor = "end",
  text = "    print('codemod');\n",
})

c:replace({ path = c.args.file, query = "...", capture = "member", text = "  final int count = 0;\n" })
c:remove({ path = c.args.file, query = "...", capture = "member" })
```

### Read-only query helpers

```lua
local spans = c:query({ path = c.args.file, query = "...", capture = "body" })
-- spans[i] = { start = byte, ["end"] = byte, text = "..." }

if c:queryExists({ path = c.args.file, query = "...", capture = "body" }) then ... end
local n = c:queryCount({ path = c.args.file, query = "..." })  -- optional capture
c:expectMatch({ ... })   -- error if zero matches
c:expectNoMatch({ ... }) -- error if any match
```

### File I/O

```lua
local source = c:read(c.args.file)  -- or c:read({ path = "..." })

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
c:run("dart.logging.add_log_line", { file = c.args.file, className = "Settings" })
```

Child recipe `postExecution` runs inline; parent args are restored afterward.

### Templates (MiniJinja)

```lua
local text = c:applyTemplate({
  template = "class {{ className | pascal_case }} {}\n",
  env = { className = c.args.className },  -- optional extra vars
})

local fileBody = c:renderFileTemplate({
  path = "templates/widget.dart.template",
})
```

MiniJinja has access to recipe args plus registered maps/variables (`c:map`, `c:var`).
See [docs/recipe-templates.md](../../../docs/recipe-templates.md).

### Asset lookups

Shared assets are Luau modules:

| Location | Return shape |
|----------|--------------|
| `.codemod/maps/*.luau` | `{ id = "...", map = { key = "value" } }` |
| `.codemod/variables/*.luau` | `{ id = "...", values = { key = "value" } }` |
| `.codemod/queries/*.luau` | `{ id = "...", queries = { name = "(...) @x" } }` |

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
Query libraries are Luau modules under `.codemod/queries/*.luau`.

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
  "inlineRecipe": "return {\n  id = \"__inline\",\n  apply = function(c)\n    c:insert({ path = \"lib/a.dart\", query = \"(program) @root\", capture = \"root\", anchor = \"end\", text = \"// x\\n\" })\n  end,\n}"
}
```

## Validation

`validate_recipes` reloads all Luau under `.codemod/` (recipes, maps, variables, query libs),
checks required fields, and verifies `apply` is present.

## Control flow

Use Luau `if` / `for` inside `apply`. MiniJinja is only for `c:applyTemplate` and `c:renderFileTemplate`.

## Maintainer notes

When changing recipe loading or host API:

- `rust/crates/lua/src/load.rs` — parse recipe/map/variable/query-lib tables
- `rust/crates/host/src/lua_runtime.rs` — `c:` method implementations
- `rust/crates/host/src/protocol_keys.rs` — transport keys (`inlineRecipe`, `previewToken`, …)
