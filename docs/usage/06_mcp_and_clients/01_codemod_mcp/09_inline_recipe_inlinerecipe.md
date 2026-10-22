# Inline recipe (`inlineRecipe`)

`inlineRecipe` is a **Luau source string** that `return`s a recipe table with `apply`:

```json
{
  "inlineRecipe": "return {\n  id = \"__inline_insert\",\n  apply = function(c, args)\n    c:insert({\n      path = \"lib/settings.dart\",\n      query = \"(class_definition name: (identifier) @className body: (class_body (method_signature (function_signature name: (identifier) @methodName)) (function_body (block) @body)) (#eq? @className \\\"Settings\\\") (#eq? @methodName \\\"update\\\"))\",\n      capture = \"body\",\n      anchor = \"end\",\n      text = \"    print('codemod');\\n\",\n    })\n  end,\n}"
}
```

Field rules on edit specs (same as registered recipes):

- `insert`: `path`, `query`, `capture`, `anchor` (`start`|`end`), `text`
- `replace`: `path`, `query`, `capture`, `text`
- `remove`: `path`, `query`, `capture`
