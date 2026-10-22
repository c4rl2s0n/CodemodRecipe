# Inline recipes (MCP / host)

`inlineRecipe` is a **Luau source string** that `return`s the same table shape (must include `apply`).

```json
{
  "inlineRecipe": "return {\n  id = \"__inline\",\n  apply = function(c, args)\n    c:insert({ path = \"lib/a.dart\", query = \"(program) @root\", capture = \"root\", anchor = \"end\", text = \"// x\\n\" })\n  end,\n}"
}
```
