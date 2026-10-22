# Recipe taxonomy

Convey recipe kind through the `id` prefix and `description` — there is no separate
`kind` field.

| Prefix | Purpose | Primary host calls | Typical idempotency |
|--------|---------|-------------------|---------------------|
| `create_` | Generate new files from templates | `c:create` | Fail if file exists (`ifExists = "fail"`) |
| `add_` / `patch_` | Insert or update existing code | `c:insert` / `c:replace` | Varies (see Idempotency) |
| `remove_` | Tear down code or files | `c:remove` or `c:delete` | Safe to re-run when target is gone |
| `scaffold_` | Feature-level workflow | `c:run("child.id", { ... })` (+ optional `c:delete`) | Depends on child recipes |

## `create_<artifact>` — greenfield files

Generate new source when nothing exists yet.

- Use `c:create` with `content` from `c:template`.
- Default `ifExists = "fail"` prevents accidental overwrites.
- Args drive paths and class names (`feature`, `className`).

```lua
c:create({
  path = "lib/counter/counter_repository.dart",
  content = c:template({ path = "templates/counter_repository.dart.template" }),
  ifExists = "fail",
})
```

## `add_<thing>_to_<target>` / `patch_<target>` — brownfield edits

Insert, replace, or remove AST nodes in existing files.

- Use `c:insert` / `c:replace` / `c:remove` with tree-sitter `query` + `capture`.
- Keep each recipe **atomic**: "add field to state class" is one recipe; "add matching
  handler to bloc" is a separate recipe (or a thin orchestrator that `c:run`s both).
- Args: `file` (or convention-derived path), `className`, domain symbols (`fieldName`,
  `eventName`).

## `scaffold_<feature>` — feature-level orchestrator

Run a "new feature" workflow end-to-end: create files, wire into existing code, optional
cleanup.

- Call children with `c:run("child.id")` or `c:run("child.id", { className = args.featureName })`.
- Parent args stay on the parent; pass a table to remap or hardcode child args.

```lua
return {
  id = "scaffold_feature",
  args = {
    { name = "featureName", required = true },
    { name = "fieldName", required = true },
  },
  apply = function(c, args)
    c:run("create_repository", { className = args.featureName })
    c:run("patch_counter", { className = args.featureName })
    c:run("patch_app")
    c:delete({ path = "lib/legacy/stale.dart", ifMissing = "skip" })
  end,
}
```

## `remove_<thing>_from_<target>` — teardown

Symmetric counterpart to `add_*`.

- Use `c:remove` for AST members, or `c:delete` for whole files.
- Prefer `ifMissing = "skip"` on file deletes.
- `c:remove` queries that match nothing produce an empty preview (safe re-run).
