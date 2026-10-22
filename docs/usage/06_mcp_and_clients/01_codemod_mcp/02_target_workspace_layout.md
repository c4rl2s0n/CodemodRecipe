# Target workspace layout

```text
.codemod/
  recipes/    # Luau recipe modules (*.luau / *.lua)
  maps/       # Luau map modules (id + map table)
  variables/  # Luau variable modules (id + values table)
  queries/    # Luau query libraries (id + queries table)
  templates/  # optional (c:template path)
```

Discovery scans `.codemod/` for Luau modules with the expected return shapes.

You can also use inline-only workflows with `inlineRecipe` (Luau source string).

Path resolution:

- workspace mutation targets resolve exactly under the workspace root
- recipe resources (`.scm`, templates, postExecution scripts) resolve recipe-local first, then `.codemod/`
- bare query names also try `queries/` under each root
