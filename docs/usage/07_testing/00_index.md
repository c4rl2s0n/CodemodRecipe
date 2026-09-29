# Testing

Fixture-based regression tests for `.codemod` assets — recipes, components, and
stack-level relations — written as Luau `*.test.luau` modules and run by the host via
the `run_tests` MCP tool.

| Page | |
|------|---|
| [Test suites](01_test_suites.md) | |
| [Fixture workspaces](02_fixture_workspaces.md) | |
| [Running tests](03_running_tests.md) | |

## `verify` vs `tests`

These are complementary, not overlapping:

- **`verify`** is the *continuous judgment of the real workspace* — it runs on every
  scan and reports findings about the code as it is now.
- **`tests`** are *fixture-based regression tests of hook logic* — they run on demand
  against throwaway workspaces and fail loudly when a hook stops behaving.

Write `verify` for "this workspace should look like X"; write tests for "my
`parse`/`relate`/operation logic handles these cases".

## Quick example

```lua
-- .codemod/components/flutter.model.test.luau
return {
  id = "flutter.model.tests",
  target = "flutter.model",   -- optional; grouping/filtering
  tests = {
    parses_fields = function(t)
      local ws = t:workspace({ files = {
        ["lib/models/recipe.dart"] = "class Recipe { final String title; }",
      } })
      local node = ws:parse("flutter.model", { path = "lib/models/recipe.dart" })
      t:assertEq(node.name, "Recipe")
    end,
  },
}
```

Run it with the [`run_tests`](../06_mcp_and_clients/01_codemod_mcp/05_tools.md) MCP
tool (optional `filter` argument narrows to a suite or test). Everything executes
against fresh temp directories — the real workspace is never touched.
