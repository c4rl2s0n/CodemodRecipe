# Required manifest

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
  -- optional: universe for component span coverage
  codebase = {
    "lib/**/*.dart",
  },
}
```

| Field | Required |
|-------|----------|
| `id` | Yes |
| `components` | Yes — array of component ids for **root scan** |
| `codebase` | No — array of workspace-relative globs (`*`, `**`, `*.ext`) defining files measured by [span coverage](../01_components/09_coverage.md) |

Optional: `description`. Root candidates still come from **`identify(c)`** / `matchFiles`; `codebase` is only the coverage universe. Without `codebase`, coverage tools skip with a warning.
