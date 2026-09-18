# Worked insert example

```lua
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
```
