# Maps, variables, query libraries

Shared Luau modules for maps, variables, and query libraries. They can live anywhere under `.codemod/` and are identified by top-level `id` plus table shape (`map`, `values`, or `queries`). The `.codemod/maps/`, `.codemod/variables/`, and `.codemod/queries/` directories are conventions, not requirements:

```lua
local types = c:map("column_type")
local defaults = c:var("project_defaults")
local q = c:queryLib("dart")
```

Plain helper modules (**no** top-level `id`) are not catalogued; `require` them from recipes/components under `.codemod/` (see [luau-lsp.md](../../05_components_and_stacks/04_luau_lsp/04_local_and_shared_types.md)). Maps/vars stay on `c:map` / `c:var` for now.
