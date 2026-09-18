# Recipe shape

Recipes are Luau modules (`.luau` or `.lua`) under `.codemod/` that **return a table** with `id` and `apply = function(c, args)`. The `.codemod/recipes/` directory is a convention, not a requirement — the registry identifies a recipe by the `apply` function in its returned table.

```lua
return {
  id = "feature.area.recipe_id",  -- dotted ids nest in VS Code Recipes tab
  name = "Human-readable name",   -- optional
  description = "What it does",   -- optional
  args = {},                      -- optional parameters
  maps = {},                      -- optional recipe-local maps
  queries = {},                   -- optional inline query ids
  postExecution = { "dart format ." },  -- optional; apply only
  explorerMenu = {},              -- optional Explorer submenu
  apply = function(c, args)
    -- all mutations here
  end,
}
```

**`apply(c, args)` is the only mutation entrypoint.** Use Luau `if`/`for` for guards and
idempotency checks (e.g. `c:queryExists` before insert).

## Arguments (`args`)

```lua
args = {
  { name = "file", required = true, inputKind = "file" },
  { name = "className", required = true, inputKind = "text" },
  { name = "includeTests", inputKind = "bool", defaultsTo = "false" },
}
```

Builtin `inputKind` values: `text`, `bool`, `choice`, `file`, `directory`.
A registered type id (from a Luau module under `.codemod/` that returns `{ id, fields = { ... } }`) is one object; `{ "TypeId" }` is a
list of that type. `choice` needs `options` or `optionsFromMap`. `text` with options
is a combobox (suggestions plus free typing). `symbol` still loads as `text`.

```lua
-- .codemod/types/class_property.luau (conventional location)
return {
  id = "ClassProperty",
  fields = {
    { name = "type", inputKind = "text", required = true },
    { name = "name", inputKind = "text", required = true },
    { name = "initialValue", inputKind = "text" },
  },
}
```

```lua
args = {
  { name = "properties", inputKind = { "ClassProperty" } },
}
-- in apply:
for _, p in ipairs(args.properties or {}) do
  c:run("add_property", { type = p.type, name = p.name })
end
```
Defaults use `defaultsTo`. In VS Code, `from` can derive args from the editor —
see [recipe-shortcuts.md](../../04_templates_and_patterns/03_recipe_shortcuts/00_index.md).
