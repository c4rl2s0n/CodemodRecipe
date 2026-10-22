# Maps and templates

**Templates** (`c:template`) hold greenfield skeletons.
Use Jinja2 syntax (MiniJinja): `{{ arg }}`, filters like `{{ feature | pascal_case }}`,
and `{% if %}`. See [host-api.md](../../02_writing_recipes/02_host_api/00_index.md) and [recipe-templates.md](../01_recipe_templates/00_index.md).

**Maps** resolve symbolic names to types or snippets:

```lua
-- .codemod/maps/dart_types.luau
return {
  id = "dart_types",
  map = {
    email = "String",
    age = "int",
  },
}
```

```lua
local text = c:template({
  inline = "  final {{ fieldName | map('dart_types') }} {{ fieldName | camel_case }};\n\n",
})
```

Recipe-local `maps = { dart_types = { email = "String" } }` on the return table
merges with registered map modules.
