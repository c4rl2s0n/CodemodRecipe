# Add your first Luau recipe

Create `.codemod/recipes/add_log_line.luau`:

```lua
return {
  id = "add_log_line",
  name = "Add log line",
  description = "Inserts a log line at the end of a method",

  args = {
    { name = "file", required = true, inputKind = "file" },
    { name = "className", required = true, inputKind = "text" },
    { name = "methodName", required = true, inputKind = "text" },
  },

  postExecution = { "dart format ." },

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
}
```
