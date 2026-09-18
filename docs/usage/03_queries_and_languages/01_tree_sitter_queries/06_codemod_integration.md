# Codemod integration

## `query` field

Inline Luau long string:

```lua
query = [[
  (class_definition
    name: (identifier) @className
    body: (class_body (block) @body)
    (#eq? @className "]] .. args.className .. [["))
]]
```

Or path to a `.scm` file (using the shared resolver in
`rust/crates/core/src/resource_path.rs`):

```lua
query = "settings_update_body.scm"
```

Search order (engine): recipe directory → `queries/` under recipe →
`.codemod/` → `.codemod/queries/`. Query libraries are Luau modules under
`.codemod/queries/*.luau` (`{ id, queries = { name = "(...) @x" } }`); look them
up with `c:queryLib(id)` or by registered query id.

## Arg substitution

Interpolate into `query` / `text` with Luau or `c:template` **before** the
query is parsed. Use predicates with concrete literals:

```lua
local query = c:template({
  inline = [[
(class_definition
  name: (identifier) @className
  (#eq? @className "{{className}}"))
]],
})
```

Do not try to parameterize predicate names — only values.

## `capture` + `anchor` on ops

| Op | `capture` | `anchor` |
|----|-----------|----------|
| `c:insert` | required — node to insert beside/inside | `"start"` or `"end"` of capture span |
| `c:replace` | required — node span replaced by `text` | n/a |
| `c:remove` | required — node span deleted | n/a |

Optional `includeLeadingTrivia = true` on `c:replace` / `c:remove` expands the span over leading `///` or `//` comments (language-dependent; see `codemod-languages`).

## Parse errors

If the source file has syntax errors (`has_error()` on the tree root), the engine rejects the edit before matching. Fix or exclude broken files.

## Syntax errors in the query itself

Invalid node types for the grammar fail at preview time:

```text
Invalid node type "class_declaration"
```

Use node names from the grammar for `language` on the op (pack Dart uses `class_definition` — see `codemod-languages`).

---
