# Dart grammar note (language-pack)

The pack ships a **different Dart AST** than the old `tree-sitter-dart` crate. Node names changed — recipes must use pack names:

| Old (crates.io dart) | Pack dart |
|----------------------|-----------|
| `class_declaration` | `class_definition` |
| `class_member` wrapper | members directly under `class_body` |

**Insert at end of method body (pack dart):**

```lua
c:insert({
  path = args.file,
  language = "dart",
  query = [[
    (class_definition
      name: (identifier) @className
      body: (class_body
        (method_signature
          (function_signature
            name: (identifier) @methodName))
        (function_body
          (block) @body))
      (#eq? @className "Settings")
      (#eq? @methodName "update"))
  ]],
  capture = "body",
  anchor = "end",
  text = "    print('codemod');\n",
})
```

**Remove a field (pack dart):**

```lua
c:remove({
  path = args.file,
  language = "dart",
  query = [[
    (class_definition
      name: (identifier) @className
      body: (class_body
        (declaration
          (initialized_identifier_list
            (initialized_identifier
              (identifier) @fieldName))) @member)
      (#eq? @className "Settings")
      (#eq? @fieldName "count"))
  ]],
  capture = "member",
})
```

See skill `codemod-recipe-authoring` for more Dart patterns.

---
