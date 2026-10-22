# Inline recipes (MCP)

`inlineRecipe` is a **Luau source string** that `return`s the same table (must include `apply`):

```json
{
  "inlineRecipe": "return { id = \"__inline\", apply = function(c, args) c:insert({ path = \"lib/a.dart\", query = \"(program) @root\", capture = \"root\", anchor = \"end\", text = \"// x\\n\" }) end }"
}
```
