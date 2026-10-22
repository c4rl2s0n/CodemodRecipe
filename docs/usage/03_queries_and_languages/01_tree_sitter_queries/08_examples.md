# Examples

## Insert at end of method body (Dart)

```lua
c:insert({
  path = args.file,
  query = c:template({
    inline = [[
(class_definition
  name: (identifier) @className
  body: (class_body
    (method_signature
      (function_signature
        name: (identifier) @methodName))
    (function_body
      (block) @body))
  (#eq? @className "{{className}}")
  (#eq? @methodName "{{methodName}}"))
]],
  }),
  capture = "body",
  anchor = "end",
  text = "    print('codemod');\n",
})
```

## Remove a field by name (Dart)

```lua
c:remove({
  path = args.file,
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

## Rust — any function item

```lua
c:insert({
  path = "src/main.rs",
  language = "rust",
  query = "(function_item) @fn",
  capture = "fn",
  anchor = "end",
  text = "\n// codemod\n",
})
```

## External `.scm` file

`settings_update_body.scm`:

```scheme
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
```

Recipe:

```lua
c:insert({
  path = args.file,
  query = "settings_update_body.scm",
  capture = "body",
  anchor = "end",
  text = "    print('codemod');\n",
})
```

---
