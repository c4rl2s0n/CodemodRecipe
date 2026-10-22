# Create the .codemod layout

```text
.codemod/
  recipes/     # Luau recipe modules (*.luau or *.lua) — convention
  maps/        # Luau map modules — convention
  types/       # Named arg types — convention
  variables/   # Luau variable modules — convention
  queries/     # Luau query libraries — convention
  templates/   # optional (c:template path)
```

The registry identifies every `.luau` and `.lua` module under `.codemod/` by its returned table
shape (`apply`, `map`, `fields`, `values`, `queries`). Directories are conventions, not requirements.
File-backed resources resolve recipe-local first, then `.codemod/`.
