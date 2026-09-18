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
}
```

| Field | Required |
|-------|----------|
| `id` | Yes |
| `components` | Yes — array of component ids for **root scan** |

Optional: `description`. There is **no** `globs` field — roots find candidates via **`identify(c)`**.
