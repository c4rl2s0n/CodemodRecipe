# Running tests

Run suites with the `run_tests` MCP tool. It is read-only with respect to the real
workspace (all writes land in temp dirs), so it is safe to allowlist for agents.

```json
{ "filter": "flutter.model" }
```

## Filter

`filter` is optional. It matches, by substring:

- a **suite id** (`"flutter.model.tests"`),
- a suite's **`target`** (`"flutter.model"`),
- a **`suite/test`** pair split on `/` (`"flutter.model/parses_fields"`),
- a bare **test name** (`"parses_fields"`).

Empty or omitted filter runs everything. No matching tests is a usage error
(`ok = false` with an `error` message listing the known suite ids), as is having no
suites registered at all.

## Report

```json
{
  "ok": true,
  "suites": [
    {
      "id": "flutter.model.tests",
      "path": ".codemod/components/flutter.model.test.luau",
      "tests": [
        { "name": "parses_fields", "ok": true },
        { "name": "add_store_field", "ok": false, "error": "expected 2, got 1", "line": 24 }
      ]
    }
  ],
  "totals": { "passed": 1, "failed": 1 }
}
```

- `ok` at the top is `true` only when every selected test passed.
- Failures carry the raised message and the **line number** in the suite file.
- Suite load errors (syntax errors, bad shape) surface through `validate_recipes`
  like any other asset — a suite that cannot load never reaches the runner.

## Authoring loop

1. Write/extend the `*.test.luau` suite next to the asset under test.
2. `validate_recipes` — the suite must load and classify.
3. `run_tests` with a narrow `filter` while iterating; full run before handing off.

## VS Code extension

The extension surfaces suites in the native **Testing** view (beaker icon) via a
`TestController` — no extra views are added to the Codemod Recipe container.

- **Discovery** rides the catalog reload (the `.codemod` watcher): suites appear
  grouped by their dot-separated id (`flutter.model.tests` → `flutter ▸ model ▸
  tests`), with test functions as leaves.
- **Run scopes**: every tree level has a run button — individual test, suite,
  group, or all (toolbar / `Codemod Recipe: Run Tests`). Gutter run buttons appear
  inside `*.test.luau` files.
- **Results** are color-coded per test; failures carry the raised message with a
  `file:line` location (editor peek + Test Results output), so clicking a red test
  jumps to the failing line.
- Suite files with load errors do not appear in the tree — they surface through
  the usual diagnostics (Problems panel / `validate_recipes`).
