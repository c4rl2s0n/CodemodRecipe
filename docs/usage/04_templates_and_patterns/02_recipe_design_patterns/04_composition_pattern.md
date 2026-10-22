# Composition pattern

Prefer small, reusable atomics composed by orchestrators:

```lua
-- scaffold_bloc_feature.luau
apply = function(c, args)
  c:run("create_bloc_state")
  c:run("create_bloc_event")
  c:run("create_bloc")
  c:run("patch_app_routes")
end
```

`c:run` executes the child `apply` with the given args (parent args restored afterward).
Recipe cycles are rejected.

Gate children with Luau `if` / `for` inside `apply` (bool args, `c:queryExists`,
`c:read` via `pcall` for file existence). See [host-api.md](../../02_writing_recipes/02_host_api/00_index.md).

**Same-path incremental ops** are supported via sequential staging: a scaffold may
`c:create` a file (with `ifExists = "skip"`) and then `c:insert` later (or via a
composed `add_*` recipe), or chain dependent edits (class then members). Templates for
greenfield creates must be query-compatible with the following edits.

For incremental changes after scaffolding, call atomic `add_*` recipes directly — do not
re-run the orchestrator unless the orchestrator is intentionally idempotent (`ifExists = "skip"`
plus edits that no-op when already applied).

## Ensure / barrel pattern

```lua
c:create({
  path = "lib/foo/foo.dart",
  content = "// Exports for foo.\n",
  ifExists = "skip",
})
c:insert({
  path = "lib/foo/foo.dart",
  query = "(program) @root",
  capture = "root",
  anchor = "end",
  text = "export 'bar.dart';\n",
})
```
