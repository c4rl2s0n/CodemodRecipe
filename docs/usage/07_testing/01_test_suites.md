# Test suites

A test suite is a Luau module returning `{ id, tests }` — classified as a **test
suite** asset (not a recipe, not a component) and registered alongside them.

## Module shape

```lua
return {
  id = "flutter.model.tests",        -- required, unique across all assets
  target = "flutter.model",          -- optional; grouping + filter matching
  description = "...",               -- optional
  tests = {
    parses_fields = function(t)
      -- ...
    end,
    store_relation_resolves = function(t)
      -- ...
    end,
  },
}
```

- `tests` is a table of **functions**; every value must be a function and at least
  one test is required. Test names run in sorted order.
- `id` must not collide with any recipe, component, or other suite id (duplicate ids
  are reported by `validate_recipes`).
- Suites may `require` shared helpers from `.codemod/lib/` — each test runs in a
  sandbox with the same `require` support as production hooks.

## Placement

Suites are discovered anywhere under `.codemod/`; the convention is to colocate by
name with what they exercise:

| Suite file | Exercises |
|-----------|-----------|
| `components/flutter.model.test.luau` | one component's hooks |
| `stack.test.luau` | stack-level scan, relations, verify |
| `recipes/<name>.test.luau` | a recipe's apply behavior |

Workspace fixtures for `t:workspace("...")` live under `.codemod/tests/fixtures/`
(see [Fixture workspaces](02_fixture_workspaces.md)). Files below that directory are
never loaded as assets, so a fixture may itself contain a `.codemod` tree.

## Pass/fail semantics

**A test fails by raising, and passes by returning normally.** Use plain Luau
`assert(cond, msg)` / `error(msg)`, or the `t` helpers below. Returning `false` or a
string does *not* fail a test. Each test runs in isolation — fresh sandbox, fresh
module eval, fresh fixture workspaces — and one failure does not stop the run; the
report carries the failing line number per test.

## The `t` context

| Method | Behavior |
|--------|----------|
| `t:workspace(spec)` | Materialize a fixture workspace → [`ws`](02_fixture_workspaces.md) |
| `t:assert(v, msg?)` | Fail with `msg` (default `"assertion failed"`) when `v` is falsy; returns `v` |
| `t:assertEq(got, want, msg?)` | Deep equality (tables compared structurally); on failure renders **both** values, which raw `assert(a == b)` does not; returns `got` |
| `t:fail(msg?)` | Unconditional failure |

Prefer `t:assertEq` over `assert(a == b)` for values you will want to see on failure.
