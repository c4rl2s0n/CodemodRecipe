# Stacks (`stack.luau`)

A **stack** declares which components participate in **workspace root scan**. Child-only kinds are still loaded from `.codemod/components/` but are reached only through `c:component(id)` from a parent hook.

## Required manifest

Path: **`.codemod/stack.luau`** (detected by return-table shape, like recipes).

```lua
return {
  id = "reference-stack",
  description = "optional human text",
  components = {
    "example.entity",
    "example.store",
    -- root-scan ids only — NOT example.property
  },
}
```

| Field | Required |
|-------|----------|
| `id` | Yes |
| `components` | Yes — array of component ids for **root scan** |

Optional: `description`. There is **no** `globs` field — roots find candidates via **`identify(c)`**.

## Root scan vs child-only

| | In `stack.components` | At `componentScan` | Via `c:component` |
|--|----------------------|--------------------|-------------------|
| **Root** (entity, store, …) | Yes | identify → parse → verify | Yes |
| **Child-only** (property, column, …) | No | Skipped | Yes — parent passes locator |

All component modules are loaded into the registry on reload. The stack only gates **who runs `identify` at scan time**.

## Validation

On reload, when any components exist:

1. `stack.luau` must be present.
2. Every id in `stack.components` must match a loaded component module.

Recipe-only projects (no components) do not require a stack.

## Not in v1

- `stackVersion`, lockfile, migrations
- Global conventions table (use maps/vars + per-component `conventions`)
- Span-scoped / string-buffer queries

## Example

See `test/fixtures/reference_stack/.codemod/stack.luau` and [components.md](./components.md).
