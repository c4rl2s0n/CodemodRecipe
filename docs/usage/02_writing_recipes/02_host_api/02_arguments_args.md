# Arguments (`args`)

```lua
args = {
  {
    name = "file",
    required = true,
    inputKind = "file",       -- text | bool | choice | file | directory | TypeId | { TypeId }
    defaultsTo = "lib/foo.dart",
    help = "Target Dart file",
    abbr = "f",
    options = { "a", "b" },   -- choice = closed list; text = suggestions
    optionsFromMap = "column_type",
    from = "activeFile",      -- or a table for query-based derivation
  },
}
```

`inputKind` is the type. `{ "ClassProperty" }` means a list. Named types live in
Luau modules anywhere under `.codemod/` that return `{ id, fields }`, or as recipe-local `types`.
The `.codemod/types/` directory is a convention, not a requirement. Nested lists belong on a
type field, not as `{{ "text" }}`.

Arg defaults use `defaultsTo`. Unknown keys on an arg entry fail validation.
