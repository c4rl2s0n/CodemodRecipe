# Fixture workspaces

`t:workspace(spec)` materializes a **fresh temp directory** per call and binds a
fixture-bound host (`ws`) to it. The fixture registry loads the workspace's real
`.codemod/` assets (components, recipes, stack) but scans and writes only inside the
temp dir — the real workspace is never touched, fixtures stay pristine, and tests can
run in any order.

## Three forms

```lua
-- 1. Copy an on-disk fixture dir (under .codemod/tests/)
local ws = t:workspace("fixtures/empty_app")

-- 2. Inline tree only
local ws = t:workspace({ files = {
  ["lib/models/recipe.dart"] = "class Recipe { final String title; }",
} })

-- 3. Base dir + inline overrides/additions (the fixture-variation pattern)
local ws = t:workspace({
  base = "fixtures/model_with_store",
  files = { ["lib/models/recipe.dart"] = "class Recipe { final String id; }" },
})
```

- `base` is relative to `.codemod/tests/` and must not escape it (no absolute paths,
  no `..`); inline `files` paths are workspace-relative, also without `..`.
- `t:workspace({})` is a deliberate **empty workspace** — the starting point for
  scaffold tests.
- Temp dirs are removed after each test, including on materialization errors.

## Scan is explicit

`t:workspace` only materializes and binds. `ws:scan()` is a separate call so
unit-style `ws:parse` stays cheap and scan → apply → rescan sequences read clearly.
Exception: `ws:verify` / `ws:relate` need the scan graph, so they trigger a scan on
demand when none ran yet (same semantics as `component_dry_run_hook`).

## The `ws` surface

| Method | Returns |
|--------|---------|
| `ws:scan()` | Full pipeline (`identify → parse → relate → verify`): `{ instances, edges, findings, kindGraph, ... }` |
| `ws:identify(componentId)` | Raw locator list from `identify` |
| `ws:parse(componentId, locator)` | Parse-tree node for one locator |
| `ws:verify(componentId, instance)` | Findings (lazy scan-on-demand) |
| `ws:relate(componentId, instance)` | Raw edge(s) from `relate` (lazy scan-on-demand) |
| `ws:apply(componentId, operation, args, instanceId)` | Operation response; writes land in the temp dir |
| `ws:scaffold(componentId, args)` | Scaffold response; writes land in the temp dir |
| `ws:applyRecipe(recipeId, args)` | `{ files, applied, transcript, sawPostExec }` — postExec scripts are flagged, never executed |
| `ws:readFile(path)` | File contents, or `nil` when missing |

Mutations (`apply`, `scaffold`, `applyRecipe`) write to the fixture temp dir through
the production write path, so `ws:readFile` and a follow-up `ws:scan()` observe them:

```lua
apply_and_rescan = function(t)
  local ws = t:workspace({ base = "fixtures/model_with_store" })
  local scan = ws:scan()
  local model = scan.instances[1]
  ws:apply("flutter.model", "addField", { name = "id", type = "String" }, model.id)
  t:assert(ws:readFile("lib/models/recipe.dart"):find("final String id", 1, true) ~= nil)
  local rescan = ws:scan()
  t:assertEq(#rescan.findings, 0)
end,
```

The `componentId` argument to `ws:apply` is for call-site readability — the instance
id (`componentId/key`) already determines the component.
