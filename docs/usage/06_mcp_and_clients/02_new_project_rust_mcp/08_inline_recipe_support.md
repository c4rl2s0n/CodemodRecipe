# Inline recipe support

Pass Luau source as `inlineRecipe`:

```json
{
  "inlineRecipe": "return { id = \"__inline\", apply = function(c, args) c:create({ path = \"lib/x.dart\", content = \"// new\\n\" }) end }"
}
```
