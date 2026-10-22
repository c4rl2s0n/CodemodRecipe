# Path resolution

- Workspace mutation targets (`path` on edits/create/delete) resolve under workspace root only.
- Recipe resources (`.scm`, template files, postExecution scripts) resolve recipe-local first, then `.codemod/`.
- Bare query names also try `queries/` under each root.
