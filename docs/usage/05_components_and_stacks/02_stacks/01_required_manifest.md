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
  -- optional: universe for component span coverage (`!` excludes carve-outs)
  codebase = {
    "lib/**/*.dart",
    -- "!**/generated/**",
  },
  -- optional: named membership zones (identified by repo.zone)
  zones = {
    { id = "structural.modules", role = "structural", title = "Module barrels" },
    { id = "command.adapters", role = "architecture", title = "Command adapters" },
    { id = "domain.recipe", role = "architecture", title = "Recipe domain" },
    { id = "docs.surface", role = "documentation", title = "Docs surface" },
  },
  -- optional: metric flags per role string (missing role ⇒ title-only, no flags)
  zoneRoles = {
    structural = { ignoreRelations = true, ignoreCoverage = false },
  },
}
```

| Field | Required |
|-------|----------|
| `id` | Yes |
| `components` | Yes — array of component ids for **root scan** |
| `codebase` | No — array of workspace-relative globs (`*`, `**`, `*.ext`) defining files measured by [span coverage](../01_components/09_coverage.md). Patterns starting with `!` exclude (union all includes, then drop any path matching any exclude; order does not matter) |
| `zones` | No — catalog of membership zone instances (`id`, optional `role` / `title` / `parentId`). Empty ⇒ no `repo.zone` instances. See [relations](../01_components/10_relations.md) |
| `zoneRoles` | No — map `role` → `{ ignoreRelations?, ignoreCoverage? }` (`ZoneRole`). Omitted fields default `false`. A zone whose `role` has **no** entry stays title-only (no metric special-case) |
| `relations` | No — map of relation kind → `{ layout = "edge" \| "member" }`. Undeclared kinds default to `edge`. `member` kinds become opaque-string subgraphs in instance mermaid (not arrows); see [relations](../01_components/10_relations.md) |
| `operations` | No — stack-level ops (fan-out) |

### `ZoneRole` flags (v1)

| Flag | Effect |
|------|--------|
| `ignoreRelations` | Members under that role’s zones are `relationIgnored` in snapshot relation metrics (not `architectureIsolated`; out of that ratio’s denominator) |
| `ignoreCoverage` | Members’ leaf spans omitted from **snapshot** compact `coverage` only (live `component_coverage` unchanged) |

Optional: `description`. Root candidates still come from **`identify(c)`** / `matchFiles`; `codebase` is only the coverage universe. Without `codebase`, coverage tools skip with a warning.
