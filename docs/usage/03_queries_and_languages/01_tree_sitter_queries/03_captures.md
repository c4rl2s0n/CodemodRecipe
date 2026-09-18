# Captures

Attach `@name` after a matched node:

```scheme
(identifier) @className
(block) @body
```

A single pattern may define **multiple captures**. Use extra captures for filtering (`#eq?`) and pick the edit target with the op’s `capture` field:

```lua
c:replace({
  path = args.file,
  query = [[
    (class_definition
      name: (identifier) @className
      body: (class_body
        (declaration) @member)
      (#eq? @className "Settings"))
  ]],
  capture = "member",    -- edits @member, not @className
  text = "  final int count = 0;\n",
})
```

## codemod capture semantics

For each `insert` / `replace` / `remove` op:

1. The engine runs the query against the file.
2. It collects spans for the capture named in `capture:` (without the `@`).
3. **0 matches** → op skipped or preview shows empty `files`.
4. **1 match** → span is edited.
5. **2+ matches** → error (`query matched multiple nodes`) unless the engine takes the first (prefer tightening the query).

Always name captures explicitly (`@className`, `@body`, `@member`) — never rely on implicit ordering.

---
