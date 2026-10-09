# Captures

Attach `@name` to a matched node. Tree-sitter accepts two placements:

## Preferred: after the node type

```scheme
(formal_parameter @param
  (constructor_param (identifier) @paramName))
```

`@param` is clearly on `formal_parameter`. Prefer this form — it survives nested parens.

## Also valid: after the closing paren

```scheme
(identifier) @className
(block) @body
```

`@` binds to the node closed by the **nearest** `)`. Miscounting parens captures a child instead of the parent:

```scheme
; WRONG — @param closes on constructor_param (one paren too early in nesting)
(formal_parameter (constructor_param (identifier) @paramName) @param)

; RIGHT — @param after formal_parameter's type, or after its outer closing paren
(formal_parameter @param (constructor_param (identifier) @paramName))
(formal_parameter (constructor_param (identifier) @paramName)) @param
```

## Multi-capture patterns

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
