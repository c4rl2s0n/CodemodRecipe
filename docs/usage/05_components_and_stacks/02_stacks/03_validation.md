# Validation

On reload, when any components exist:

1. `stack.luau` must be present.
2. Every id in `stack.components` must match a loaded component module.

Recipe-only projects (no components) do not require a stack.

## Scan phase order

At `componentScan` the host runs, in order:

1. **`identify`** — stack roots only; each returns freeform locators. (Roots still
   require `identify`; unchanged behavior.)
2. **`parse`** — every root locator; parents may nest children via `c:component(...)`.
3. **`relate`** — every instance in the forest (roots + nested children) whose
   component defines the hook; the host resolves refs, merges duplicate edges by
   `(from, kind, target)` into canonical edges with `sources`, and attaches
   `ast.relations`.
4. **`verify`** — every instance whose component defines the hook, now with the
   `graph` facade (forest + resolved edges).

Findings from phases 1–2 are parse-time findings; `relation.*` and `verify` findings
are judgment findings, recomputed over the whole merged forest on incremental scans.
